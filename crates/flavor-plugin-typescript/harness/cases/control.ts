export function resolveItem(items: string[]): string {
  for (const item of items) {
    if (item === "skip") {
      continue;
    }

    switch (item) {
      case "done":
        return item;
      default:
        break;
    }
  }

  try {
    throw new Error("missing");
  } catch ({ message }) {
    return message;
  } finally {
    cleanup();
  }
}
