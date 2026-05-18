export function renderCard(
  { title, meta: { owner = "system" }, ...rest }: CardProps,
  [first, ...tail]: string[],
): string {
  const { id, flags: [primary] = [] } = rest;
  const [head, , fallback = first] = tail;
  return title;
}
