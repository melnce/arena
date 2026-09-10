/** Public-dir URL under Vite's `base` (`/` locally, `/arena/` on Pages). */
export function publicUrl(path: string): string {
  return `${import.meta.env.BASE_URL}${path.replace(/^\//, "")}`;
}
