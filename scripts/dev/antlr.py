#!/usr/bin/env python3
"""Reference-check grammar files with ANTLR in Docker."""

from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
from collections import defaultdict
from pathlib import Path


DEFAULT_ANTLR_VERSION = "4.13.2"
DEFAULT_ANTLR_SHA256 = "eae2dfa119a64327444672aff63e9ec35a20180dc5b8090b7a6ab85125df4d76"


def main() -> int:
    args = parse_args(sys.argv[1:])
    root = repo_root()
    image = args.image or f"flavor-antlr:{args.antlr_version}"

    require_docker()
    ensure_image(
        root,
        image,
        antlr_version=args.antlr_version,
        antlr_sha256=args.antlr_sha256,
        rebuild=args.rebuild_image,
        require_image=args.require_image,
    )

    return check_grammars(root, image, args.g4_files)


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)

    check = subparsers.add_parser(
        "check",
        help="validate repo grammar files",
        description=(
            "Validate .g4 files under grammars/. When files are omitted, all "
            "repo grammar files are checked. ANTLR is run in dependency mode "
            "against a read-only repo mount, so no Java parser artifacts are "
            "generated."
        ),
    )
    check.add_argument(
        "g4_files",
        nargs="*",
        help="optional .g4 files under grammars/",
    )
    check.add_argument(
        "--antlr-version",
        default=os.environ.get("ANTLR_VERSION", DEFAULT_ANTLR_VERSION),
        help=f"ANTLR version to download into the Docker image (default: {DEFAULT_ANTLR_VERSION})",
    )
    check.add_argument(
        "--antlr-sha256",
        default=os.environ.get("ANTLR_SHA256", DEFAULT_ANTLR_SHA256),
        help="expected SHA256 of the ANTLR complete jar",
    )
    check.add_argument(
        "--image",
        default=os.environ.get("ANTLR_IMAGE"),
        help="Docker image tag to build/use (default: flavor-antlr:<version>)",
    )
    check.add_argument(
        "--rebuild-image",
        action="store_true",
        help="rebuild the Docker image even if it already exists",
    )
    check.add_argument(
        "--require-image",
        action="store_true",
        help="do not build the Docker image; fail if it is missing",
    )

    args = parser.parse_args(argv)
    if args.rebuild_image and args.require_image:
        raise SystemExit("--rebuild-image and --require-image cannot be combined")
    return args


def repo_root() -> Path:
    result = subprocess.run(
        ["git", "rev-parse", "--show-toplevel"],
        check=True,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    return Path(result.stdout.strip()).resolve()


def require_docker() -> None:
    if shutil.which("docker") is None:
        raise SystemExit("docker is required for ANTLR; install Docker and start it")
    result = subprocess.run(
        ["docker", "version", "--format", "{{.Server.Version}}"],
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if result.returncode != 0:
        detail = result.stderr.strip() or result.stdout.strip()
        raise SystemExit(f"docker is not available: {detail}")


def ensure_image(
    root: Path,
    image: str,
    *,
    antlr_version: str,
    antlr_sha256: str,
    rebuild: bool,
    require_image: bool,
) -> None:
    if not rebuild and image_exists(image):
        return
    if require_image:
        raise SystemExit(f"Docker image {image!r} is missing")
    build_image(root, image, antlr_version, antlr_sha256)


def image_exists(image: str) -> bool:
    result = subprocess.run(
        ["docker", "image", "inspect", image],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    return result.returncode == 0


def build_image(root: Path, image: str, antlr_version: str, antlr_sha256: str) -> None:
    dockerfile = root / "scripts" / "dev" / "antlr.Dockerfile"
    context_dir = root / "scripts" / "dev"
    run_checked(
        [
            "docker",
            "build",
            "--build-arg",
            f"ANTLR_VERSION={antlr_version}",
            "--build-arg",
            f"ANTLR_SHA256={antlr_sha256}",
            "-t",
            image,
            "-f",
            str(dockerfile),
            str(context_dir),
        ],
        label=f"building {image}",
        attempts=3,
    )


def check_grammars(root: Path, image: str, requested_files: list[str]) -> int:
    groups = grammar_groups(root, requested_files)
    if not groups:
        raise SystemExit("no .g4 files found under grammars/")

    for relative_dir, files in groups:
        result = run_antlr(root, image, relative_dir, files)
        if result != 0:
            return result

    print("antlr grammar validation passed")
    return 0


def grammar_groups(root: Path, requested_files: list[str]) -> list[tuple[Path, list[Path]]]:
    grammar_root = root / "grammars"
    files = (
        [resolve_requested_file(root, file) for file in requested_files]
        if requested_files
        else sorted(grammar_root.rglob("*.g4"))
    )

    groups: dict[Path, list[Path]] = defaultdict(list)
    for file in files:
        relative = grammar_relative_path(grammar_root, file)
        groups[relative.parent].append(file)

    return [
        (relative_dir, sorted(group_files, key=grammar_sort_key))
        for relative_dir, group_files in sorted(groups.items())
    ]


def resolve_requested_file(root: Path, raw_path: str) -> Path:
    path = Path(raw_path)
    if not path.is_absolute():
        path = Path.cwd() / path
    path = path.resolve()
    grammar_relative_path(root / "grammars", path)
    if path.suffix != ".g4":
        raise SystemExit(f"expected a .g4 file: {raw_path}")
    if not path.is_file():
        raise SystemExit(f"grammar file not found: {raw_path}")
    return path


def grammar_relative_path(grammar_root: Path, file: Path) -> Path:
    try:
        relative = file.relative_to(grammar_root)
    except ValueError as error:
        raise SystemExit(f"grammar file must be under grammars/: {file}") from error
    if relative == Path(".") or relative.name == "":
        raise SystemExit(f"expected a grammar file path: {file}")
    return relative


def grammar_sort_key(file: Path) -> tuple[bool, str]:
    return (not file.name.endswith("Lexer.g4"), file.name)


def run_antlr(root: Path, image: str, relative_dir: Path, files: list[Path]) -> int:
    display_dir = relative_dir.as_posix()
    workdir = f"/work/grammars/{display_dir}"
    print(f"==> validating grammars/{display_dir}", flush=True)
    command = [
        "docker",
        "run",
        "--rm",
        *docker_user_args(),
        "-v",
        f"{root}:/work:ro",
        "-w",
        workdir,
        image,
        "-Werror",
        "-depend",
        *[file.name for file in files],
    ]
    return subprocess.run(command, stdout=subprocess.DEVNULL).returncode


def docker_user_args() -> list[str]:
    if hasattr(os, "getuid") and hasattr(os, "getgid"):
        return ["--user", f"{os.getuid()}:{os.getgid()}"]
    return []


def run_checked(command: list[str], *, label: str, attempts: int = 1) -> None:
    for attempt in range(1, attempts + 1):
        print(f"==> {label}", flush=True)
        try:
            subprocess.run(command, check=True)
            return
        except subprocess.CalledProcessError:
            if attempt == attempts:
                raise
            print(f"retrying {label} ({attempt + 1}/{attempts})", flush=True)


if __name__ == "__main__":
    raise SystemExit(main())
