import { getCatalog, lookupText } from "./catalog.ts";
import { escapeHtml } from "./images.ts";
import { formatGateLine } from "./info.ts";
import type { CardInstance, CrestInstance, GateInfo } from "./types.ts";

const KEYWORD_LINE_RE =
  /^(Fanfare|Ward|Rush|Storm|Bane|Drain|Barrier|Aura|Ambush|Intimidate|Last Words|Super-Evolve|Super Evolve|Enhance(?: \(\d+\))?|Accelerate(?: \(\d+\))?|Crystallize(?: \(\d+\))?|Engage|Evolve|Countdown(?: \(\d+\))?|Necromancy(?: \(\d+\))?|Rally(?: \(\d+\))?|Combo(?: \(\d+\))?|Overflow|Earth Rite|Spellboost|Fusion|Fuse)\s*:?\s*(.*)$/i;

const KEYWORD_INLINE_RE =
  /\b(Fanfare|Last Words|Evolve|Super-Evolve|Super Evolve|Enhance|Accelerate|Crystallize|Necromancy|Rally|Combo|Overflow|Earth Rite|Spellboost|Ward|Storm|Rush|Bane|Drain|Barrier|Ambush|Aura|Countdown|Fusion|Fuse)\b/gi;

export function formatCardTooltip(opts: {
  inst?: CardInstance | null;
  cardId: string;
  gates?: GateInfo[];
  displayCost?: number | null;
}): string {
  const info = lookupText(opts.cardId);
  const inst = opts.inst;
  const name = escapeHtml(info.name || inst?.name || opts.cardId);
  const baseCost = inst?.base_cost ?? info.cost;
  const paid = opts.displayCost;
  const costBit =
    paid != null && baseCost != null && paid !== baseCost
      ? `Cost ${paid} <span class="tooltip-base-cost">(base ${escapeHtml(String(baseCost))})</span>`
      : baseCost != null
        ? `Cost ${baseCost}`
        : "";
  const cat = getCatalog(opts.cardId);
  const kind = (inst?.kind || info.kind || cat?.kind || "").toLowerCase();
  const stats =
    kind === "follower"
      ? inst
        ? `${inst.attack}/${inst.defense}`
        : cat?.attack != null && cat?.defense != null
          ? `${cat.attack}/${cat.defense}`
          : ""
      : "";
  const meta = [costBit, stats, info.kind].filter(Boolean).join(" · ");
  const desc = formatTooltipDescription(info.text || "");
  const gates = formatGateBlock(opts.gates ?? []);
  return (
    `<div class="tooltip-header-name">${name}</div>` +
    (meta ? `<div class="tooltip-header-meta">${meta}</div>` : "") +
    desc +
    gates
  );
}

export function formatCrestTooltip(crest: CrestInstance): string {
  const info = lookupText(crest.id);
  const cd =
    crest.countdown != null ? `Countdown ${crest.countdown}` : crest.faith ? "Faith" : "Crest";
  return (
    `<div class="tooltip-header-name">${escapeHtml(info.name)}</div>` +
    `<div class="tooltip-header-meta">${cd}</div>` +
    formatTooltipDescription(info.text || "")
  );
}

export function formatTooltipDescription(raw: string): string {
  const desc = raw.trim();
  if (!desc) return "";
  const lines = splitAbilityLines(desc);
  return `<div class="tooltip-desc-block">${lines
    .map((line) => formatDescriptionLine(line))
    .join("")}</div>`;
}

function splitAbilityLines(text: string): string[] {
  const chunks = text
    .split(/\n+/)
    .flatMap((line) =>
      line.split(
        /(?=(?:Fanfare|Evolve|Super-Evolve|Super Evolve|Enhance(?: \(\d+\))?|Accelerate(?: \(\d+\))?|Crystallize(?: \(\d+\))?|Last Words|Engage|Necromancy(?: \(\d+\))?|Rally(?: \(\d+\))?|Combo(?: \(\d+\))?|Overflow|Earth Rite|Spellboost):)/i,
      ),
    )
    .map((s) => s.trim())
    .filter(Boolean);
  return chunks;
}

function formatDescriptionLine(line: string): string {
  const trimmed = line.trim();
  if (!trimmed) return "";
  const kwMatch = trimmed.match(KEYWORD_LINE_RE);
  if (kwMatch) {
    const keyword = kwMatch[1] ?? "";
    const rest = kwMatch[2] ?? "";
    const restHtml = rest ? ` ${boldKeywords(escapeHtml(rest))}` : "";
    return `<div class="tooltip-desc-line"><span class="tooltip-keyword">${escapeHtml(keyword)}:</span>${restHtml}</div>`;
  }
  return `<div class="tooltip-desc-line">${boldKeywords(escapeHtml(trimmed))}</div>`;
}

function boldKeywords(htmlEscaped: string): string {
  return htmlEscaped.replace(KEYWORD_INLINE_RE, (m) => `<span class="tooltip-keyword">${m}</span>`);
}

function formatGateBlock(gates: GateInfo[]): string {
  if (!gates.length) return "";
  const lines = gates
    .map(
      (g) =>
        `<div class="dynamic-counter-line${g.met ? " gate-met" : ""}">` +
        `<span class="dynamic-counter-label">${escapeHtml(formatGateLine(g))}</span>` +
        `</div>`,
    )
    .join("");
  return `<div class="tooltip-counter-block">${lines}</div>`;
}
