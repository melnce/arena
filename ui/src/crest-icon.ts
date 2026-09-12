import { publicUrl } from "./base.ts";
import crops from "./data/crest-crops.json" with { type: "json" };
import { cardImageUrl, escapeHtml } from "./images.ts";

export const FOLLOWER_CROP = { fx: 0.478, fy: 0.338, w: 0.185 } as const;
export const SPELL_CROP = { fx: 0.5, fy: 0.4, w: 0.36 } as const;

/** Spells grant a scene, not a face — wider default crop. */
export const SPELL_CRESTS = new Set([
  "crest:10412310",
  "crest:10441310",
  "crest:10451310",
  "crest:10453310",
  "crest:10553310",
  "crest:10622310",
  "crest:10712310",
  "crest:10713310",
]);

export const FRAME_APERTURE = {
  left: 0.1055,
  top: 0.1016,
  width: 0.7798,
  height: 0.7891,
} as const;

export const CARD_ART = { width: 368, height: 473 } as const;
export const FRAME_ART = { width: 218, height: 256 } as const;
export const TOOLTIP_ICON_W = 56;

export type CrestCrop = { fx: number; fy: number; w: number };

const OVERRIDES = crops as Record<string, CrestCrop>;

export function crestCrop(id: string): CrestCrop {
  const over = OVERRIDES[id];
  if (over && Number.isFinite(over.fx) && Number.isFinite(over.fy) && Number.isFinite(over.w)) {
    return over;
  }
  return SPELL_CRESTS.has(id) ? SPELL_CROP : FOLLOWER_CROP;
}

export function grantingCardId(
  crestId: string,
  grantedBy?: string | string[] | null,
): string | null {
  if (Array.isArray(grantedBy) && grantedBy[0]) return String(grantedBy[0]);
  if (typeof grantedBy === "string" && grantedBy) return grantedBy;
  const m = crestId.match(/^(?:crest|faith):(\d{8})$/);
  return m?.[1] ?? null;
}

export function cropStyle(crop: CrestCrop): string {
  return `--crest-fx:${crop.fx};--crest-fy:${crop.fy};--crest-zoom:${crop.w}`;
}

export function crestIconHtml(
  crestId: string,
  grantedBy?: string | string[] | null,
  alt = "",
): string {
  const crop = crestCrop(crestId);
  const grant = grantingCardId(crestId, grantedBy);
  const art = grant ? cardImageUrl(grant, false) : null;
  const frame = publicUrl("crests/crest_frame.png");
  const portrait = art
    ? `<img class="crest-icon-portrait crest-image" src="${escapeHtml(art)}" alt="" referrerpolicy="no-referrer">`
    : "";
  return (
    `<span class="crest-icon" style="${cropStyle(crop)}">` +
    `<span class="crest-icon-aperture">${portrait}</span>` +
    `<img class="crest-icon-frame" src="${escapeHtml(frame)}" alt="${escapeHtml(alt)}">` +
    `</span>`
  );
}

export function bindCrestIconFallbacks(root: ParentNode): void {
  for (const img of root.querySelectorAll<HTMLImageElement>(".crest-icon-portrait")) {
    const drop = () => img.remove();
    img.addEventListener("error", drop, { once: true });
    if (img.complete && img.naturalWidth === 0 && img.getAttribute("src")) drop();
  }
}

export function mountCrestIcon(
  host: HTMLElement,
  crestId: string,
  grantedBy?: string | string[] | null,
  alt = "",
): void {
  const wrap = document.createElement("span");
  wrap.innerHTML = crestIconHtml(crestId, grantedBy, alt);
  const icon = wrap.firstElementChild;
  if (icon) host.appendChild(icon);
  bindCrestIconFallbacks(host);
}
