/** Cancel in-flight `<img>` decodes so a teardown does not stall the next paint. */
export function releaseImageLoads(root: ParentNode | Element | null | undefined): void {
  if (!root) return;
  const imgs =
    root instanceof HTMLImageElement
      ? [root]
      : Array.from(root.querySelectorAll("img"));
  for (const img of imgs) {
    img.onload = null;
    img.onerror = null;
    img.src = "";
    img.removeAttribute("src");
    img.removeAttribute("srcset");
  }
}
