import { getCatalog, lookupText } from "./catalog.ts";
import { escapeHtml } from "./images.ts";
import { formatGateLine, isFormGate } from "./info.ts";
import type { CardInstance, CrestInstance, GateInfo } from "./types.ts";

const KEYWORD_LINE_RE =
  /^(Fanfare|Ward|Rush|Storm|Bane|Drain|Barrier|Aura|Ambush|Intimidate|Last Words|Super-Evolve|Super Evolve|Enhance(?: \(\d+\))?|Accelerate(?: \(\d+\))?|Crystallize(?: \(\d+\))?|Engage|Evolve|Countdown(?: \(\d+\))?|Necromancy(?: \(\d+\))?|Rally(?: \(\d+\))?|Combo(?: \(\d+\))?|Overflow|Earth Rite|Spellboost|Fusion|Fuse)\s*:?\s*(.*)$/i;

const KEYWORD_INLINE_RE =
  /\b(Fanfare|Last Words|Evolve|Super-Evolve|Super Evolve|Enhance|Accelerate|Crystallize|Necromancy|Rally|Combo|Overflow|Earth Rite|Spellboost|Ward|Storm|Rush|Bane|Drain|Barrier|Ambush|Aura|Countdown|Fusion|Fuse)\b/gi;

/** Official set id → name, matching the old practice-tool tooltip. */
const SET_LABELS: Record<string, string> = {
  "10000": "Basic",
  "10001": "Legends Rise",
  "10002": "Infinity Evolved",
  "10003": "Heirs of the Omen",
  "10004": "Skybound Dragons",
  "10005": "Blossoming Fate",
  "10006": "Apocalypse Pact",
  "10007": "Anathema's Gambit",
  "10008": "Chronicle of Destiny",
  "10009": "Revenants of Azvaldt",
};

const ROTATION_SET_IDS = new Set(["10000", "10004", "10005", "10006", "10007", "10008", "10009"]);

export function formatCardTooltip(opts: {
  inst?: CardInstance | null;
  cardId: string;
  gates?: GateInfo[];
  displayCost?: number | null;
}): string {
  const info = lookupText(opts.cardId);
  const inst = opts.inst;
  const name = escapeHtml(info.name || inst?.name || opts.cardId);
  const meta = formatClassTribeSet(opts.cardId, inst);
  const desc = formatTooltipDescription(info.text || "");
  const gates = formatGateBlock(opts.gates ?? []);
  return (
    `<div class="tooltip-header-name">${name}</div>` +
    (meta ? `<div class="tooltip-header-meta">${meta}</div>` : "") +
    gates +
    desc
  );
}

export function formatCrestTooltip(crest: CrestInstance, faithValue?: number): string {
  const info = lookupText(crest.id);
  const faith =
    crest.faith && faithValue != null ? `Faith: ${faithValue}` : crest.faith ? "Faith" : "";
  const cd = crest.countdown != null ? `Countdown ${crest.countdown}` : "";
  const meta = [faith, cd].filter(Boolean).join(" · ") || "Crest";
  return (
    `<div class="tooltip-header-name">${escapeHtml(info.name)}</div>` +
    `<div class="tooltip-header-meta">${meta}</div>` +
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

function formatClassTribeSet(cardId: string, inst?: CardInstance | null): string {
  const cat = getCatalog(cardId);
  const clazz = titleCase(inst?.class || cat?.class || "");
  const tribes = (inst?.tribes ?? []).map((t) => titleCase(String(t))).filter(Boolean);
  const classLine = clazz ? (tribes.length ? `${clazz}/${tribes.join(", ")}` : clazz) : "";
  const setLine = formatSetLine(cardId);
  return [classLine, setLine].filter(Boolean).join("<br>");
}

function formatSetLine(cardId: string): string {
  const setId = setIdFromCardId(cardId);
  if (!setId) return "";
  const name = SET_LABELS[setId];
  if (!name) return "";
  const inRotation = ROTATION_SET_IDS.has(setId);
  const text = inRotation ? name : `${name} · older set`;
  const cls = inRotation ? "card-set-line" : "card-set-line older";
  return `<span class="${cls}">${escapeHtml(text)}</span>`;
}

/** Collectible ids `1XX…` encode set `100XX`. Tokens (`9…`) have no set label. */
function setIdFromCardId(cardId: string): string | null {
  if (!/^\d{8}$/.test(cardId) || cardId.startsWith("9")) return null;
  const n = Number(cardId.slice(1, 3));
  if (!Number.isFinite(n)) return null;
  return String(10000 + n);
}

function titleCase(raw: string): string {
  return raw
    .trim()
    .split(/[\s_]+/)
    .filter(Boolean)
    .map((w) => w.charAt(0).toUpperCase() + w.slice(1).toLowerCase())
    .join(" ");
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
    const colon = rest ? ":" : "";
    return `<div class="tooltip-desc-line"><span class="tooltip-keyword">${escapeHtml(keyword)}${colon}</span>${restHtml}</div>`;
  }
  return `<div class="tooltip-desc-line">${boldKeywords(escapeHtml(trimmed))}</div>`;
}

function boldKeywords(htmlEscaped: string): string {
  return htmlEscaped.replace(KEYWORD_INLINE_RE, (m) => `<span class="tooltip-keyword">${m}</span>`);
}

function formatGateBlock(gates: GateInfo[]): string {
  const lines = gates
    .filter((g) => !isFormGate(g.kind))
    .map((g) => {
      const text = formatGateLine(g);
      if (!text) return "";
      return (
        `<div class="dynamic-counter-line${g.met ? " gate-met" : ""}">` +
        `<span class="dynamic-counter-label">${escapeHtml(text)}</span>` +
        `</div>`
      );
    })
    .filter(Boolean)
    .join("");
  if (!lines) return "";
  return `<div class="tooltip-counter-block">${lines}</div>`;
}
