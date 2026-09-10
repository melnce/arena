import { getCatalog, lookupText } from "./catalog.ts";

const CARD_HOST = "https://shadowverse-wb.com/uploads/card_image/eng/card";

export function cardImageUrl(id: string, evolved: boolean): string | null {
  const cat = getCatalog(id);
  const hash = evolved && cat?.evoCard ? cat.evoCard : cat?.card;
  if (!hash) return null;
  return `${CARD_HOST}/${hash}.png`;
}

export function bannerImageUrl(id: string): string | null {
  const cat = getCatalog(id);
  if (!cat?.banner) return null;
  return `https://shadowverse-wb.com/uploads/card_image/eng/list/${cat.banner}.png`;
}

export function applyCardImage(
  img: HTMLImageElement,
  id: string,
  evolved: boolean,
  host: HTMLElement,
): void {
  const url = cardImageUrl(id, evolved);
  img.referrerPolicy = "no-referrer";
  img.alt = lookupText(id).name;
  img.draggable = false;
  const showFallback = () => {
    img.style.display = "none";
    if (host.querySelector(".card-fallback")) return;
    const cat = getCatalog(id);
    const text = lookupText(id);
    const fb = document.createElement("div");
    fb.className = "card-fallback";
    const atk = cat?.attack ?? "";
    const def = cat?.defense ?? "";
    const stats =
      text.kind === "follower" || cat?.kind === "follower"
        ? `${atk}/${def}`
        : "";
    fb.innerHTML = `<div class="fb-name">${escapeHtml(text.name)}</div><div class="fb-stats">${text.cost ?? cat?.cost ?? "?"} ${stats}</div>`;
    host.appendChild(fb);
  };
  if (!url) {
    showFallback();
    return;
  }
  img.onerror = () => {
    img.onerror = null;
    showFallback();
  };
  img.src = url;
}

export function escapeHtml(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}
