from __future__ import annotations

from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[3]
LOCAL_DIR = REPO_ROOT / ".local"
SECRETS_DIR = LOCAL_DIR / "secrets"
TMP_DIR = LOCAL_DIR / "tmp"
