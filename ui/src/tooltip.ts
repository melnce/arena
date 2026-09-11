import { catalogIds, crestFile, getCatalog, lookupText } from "./catalog.ts";
import { publicUrl } from "./base.ts";
import { escapeHtml } from "./images.ts";
import { formatGateLine, isFormGate } from "./info.ts";
import type { CardInstance, CrestInstance, GateInfo } from "./types.ts";

const KEYWORD_LINE_RE =
  /^(Fanfare|Ward|Rush|Storm|Bane|Drain|Barrier|Aura|Ambush|Intimidate|Last Words|Super-Evolve|Super Evolve|Enhance(?: \(\d+\))?|Accelerate(?: \(\d+\))?|Crystallize(?: \(\d+\))?|Engage|Evolve|Countdown(?: \(\d+\))?|Necromancy(?: \(\d+\))?|Rally(?: \(\d+\))?|Combo(?: \(\d+\))?|Overflow|Earth Rite|Spellboost|Fusion|Fuse|Ongoing|Skybound Art)\s*:?\s*(.*)$/i;

const KEYWORD_INLINE_RE =
  /\b(Fanfare|Last Words|Evolve|Super-Evolve|Super Evolve|Enhance|Accelerate|Crystallize|Necromancy|Rally|Combo|Overflow|Earth Rite|Spellboost|Ward|Storm|Rush|Bane|Drain|Barrier|Ambush|Aura|Countdown|Fusion|Fuse|Ongoing|Skybound Art)\b/gi;

const RALLY_RE = /Rally(?:\s*\((\d+)\))?/i;
const SKYBOUND_RE = /Skybound Art(?:\s*\((\d+)\))?/i;

let knownNames: string[] | null = null;

function cardNames(): string[] {
  if (knownNames) return knownNames;
  const names = new Set<string>();
  for (const id of catalogIds()) {
    const n = getCatalog(id)?.name || lookupText(id).name;
    if (n && n.length >= 3) names.add(n);
  }
  knownNames = [...names].sort((a, b) => b.length - a.length);
  return knownNames;
}

export type TooltipPaint = {
  inst?: CardInstance | null;
  cardId: string;
  gates?: GateInfo[];
  displayCost?: number | null;
  blockedReason?: string | null;
  turn?: number;
  rallyHave?: number;
};

export function formatCardTooltip(opts: TooltipPaint): string {
  const info = lookupText(opts.cardId);
  const inst = opts.inst;
  const name = escapeHtml(info.name || inst?.name || opts.cardId);
  const clazz = info.class || inst?.class || getCatalog(opts.cardId)?.class || "Neutral";
  const tribes = (info.tribes?.length ? info.tribes : inst?.tribes) ?? [];
  const classLine = tribes.length ? `${clazz}/${tribes.join(", ")}` : clazz;
  const setLine = formatSetLine(info.set);
  const metaParts = [escapeHtml(classLine), setLine].filter(Boolean).join("<br>");

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
  const costLine = [costBit, stats, info.kind].filter(Boolean).join(" · ");

  const gates = formatGateBlock(opts.gates ?? []);
  const desc = formatTooltipDescription(info.text || "");
  const crests = formatCrestPanels(info.text || "", cat?.specificEffects);
  const extras = extraLines(opts, info.text || "");
  return (
    `<div class="tooltip-header-name">${name}</div>` +
    (metaParts ? `<div class="tooltip-header-meta">${metaParts}</div>` : "") +
    (costLine ? `<div class="tooltip-header-meta tooltip-cost-line">${costLine}</div>` : "") +
    gates +
    desc +
    crests +
    extras
  );
}

function formatSetLine(set: number | null | undefined): string {
  if (set == null) return "";
  const inRotation = set >= 8;
  const color = inRotation ? "#9aa3b2" : "#7a8494";
  return `<span class="card-set-line" style="color:${color};font-size:0.9em;">Set ${set}</span>`;
}

function extraLines(opts: TooltipPaint, text: string): string {
  let extra = "";
  extra += fusedBlock(opts.inst);
  extra += rallyLine(opts, text);
  extra += skyboundLine(opts, text);
  extra += buffDelta(opts.inst);
  if (opts.blockedReason) {
    extra +=
      `<div class="tooltip-play-blocked">Cannot play: ${escapeHtml(opts.blockedReason)}</div>`;
  }
  return extra;
}

function fusedBlock(inst?: CardInstance | null): string {
  if (!inst?.flags) return "";
  const kinds = inst.flags.fused_kinds ?? [];
  if (kinds.length) {
    const uniq = [...new Set(kinds.map((k) => String(k).trim()).filter(Boolean))];
    return `<br><br><span class="fused-loot" style="color:orange;">Fused Loot (unique): ${uniq.length}<br>${escapeHtml(uniq.join(", "))}</span>`;
  }
  if (inst.flags.was_fused) {
    return `<br><span class="fused-count" style="color:#aaa;font-size:0.8em;">Fused: 1 cards</span>`;
  }
  return "";
}

function rallyLine(opts: TooltipPaint, text: string): string {
  const gate = opts.gates?.find((g) => g.kind === "rally");
  const m = text.match(RALLY_RE);
  if (!gate && !m) return "";
  const need = gate?.need ?? (m?.[1] ? Number(m[1]) : 0);
  if (!need) return "";
  const have = gate?.have ?? opts.rallyHave ?? 0;
  return `<br><br><span class="rally-line" style="color:#7af;">Rally: <span class="rally-value">${have} / ${need}</span></span>`;
}

function skyboundLine(opts: TooltipPaint, text: string): string {
  const gate = opts.gates?.find((g) => /skybound/i.test(g.kind));
  const hasText = SKYBOUND_RE.test(text);
  if (!gate && !hasText) return "";
  const need = gate?.need ?? (Number(text.match(SKYBOUND_RE)?.[1] ?? 10) || 10);
  const have = gate?.have ?? (opts.turn ?? 1) + (opts.inst?.skybound ?? 0);
  return `<br><br><span class="skybound-line" style="color:#ebd04f;">Skybound Art: <span class="skybound-value">${have} / ${need}</span></span>`;
}

function buffDelta(inst?: CardInstance | null): string {
  if (!inst || inst.kind !== "follower") return "";
  const cat = getCatalog(inst.card);
  const text = lookupText(inst.card);
  const pa = cat?.attack ?? text.attack ?? inst.attack;
  const pd = cat?.defense ?? text.defense ?? inst.defense;
  const a = inst.attack - (pa ?? inst.attack);
  const d = inst.defense - (pd ?? inst.defense);
  if (a === 0 && d === 0) return "";
  const color = a < 0 || d < 0 ? "#ff6666" : "#66ff66";
  const sa = `${a >= 0 ? "+" : ""}${a}`;
  const sd = `${d >= 0 ? "+" : ""}${d}`;
  return `<br><br><span class="buff-delta" style="color:${color};font-weight:700;">${sa}/${sd}</span>`;
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

function splitAbilityLines(text: string): string[] {
  const chunks = text
    .split(/\n+/)
    .flatMap((line) =>
      line.split(
        /(?=(?:Fanfare|Evolve|Super-Evolve|Super Evolve|Enhance(?: \(\d+\))?|Accelerate(?: \(\d+\))?|Crystallize(?: \(\d+\))?|Last Words|Engage|Necromancy(?: \(\d+\))?|Rally(?: \(\d+\))?|Combo(?: \(\d+\))?|Overflow|Earth Rite|Spellboost|Ongoing|Skybound Art):)/i,
      ),
    )
    .map((s) => s.trim())
    .filter(Boolean);
  return chunks;
}

function formatDescriptionLine(line: string): string {
  const trimmed = line.trim();
  if (!trimmed) return "";
  const names = cardNames();
  const kwMatch = trimmed.match(KEYWORD_LINE_RE);
  if (kwMatch) {
    const keyword = kwMatch[1] ?? "";
    const rest = kwMatch[2] ?? "";
    const restHtml = rest ? ` ${highlightCardNames(escapeHtml(rest), names)}` : "";
    const colon = rest ? ":" : "";
    return `<div class="tooltip-desc-line"><span class="tooltip-keyword">${escapeHtml(keyword)}${colon}</span>${restHtml}</div>`;
  }
  if (/^gain crest:/i.test(trimmed)) {
    return `<div class="tooltip-desc-line tooltip-crest-lead">${highlightCardNames(escapeHtml(trimmed), names)}</div>`;
  }
  return `<div class="tooltip-desc-line">${highlightCardNames(escapeHtml(trimmed), names)}</div>`;
}

function highlightCardNames(escaped: string, names: string[]): string {
  let out = escaped;
  for (const name of names) {
    if (!name || name.length < 3) continue;
    const re = new RegExp(`\\b${escapeRegExp(name)}\\b`, "g");
    out = out.replace(re, `<span class="tooltip-card-name">${escapeHtml(name)}</span>`);
  }
  return out.replace(KEYWORD_INLINE_RE, (m) => `<span class="tooltip-keyword">${m}</span>`);
}

function escapeRegExp(text: string): string {
  return text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function formatCrestPanels(text: string, specific?: string[]): string {
  const names: string[] = [];
  for (const line of text.split(/\n+/)) {
    const m = line.match(/^gain crest:\s*(.+)$/i);
    if (m?.[1]) names.push(m[1].trim());
  }
  const ids = specific ?? [];
  if (!names.length && !ids.length) return "";
  const panels: string[] = [];
  const seen = new Set<string>();
  for (const id of ids) {
    const info = lookupText(id);
    if (!info.name || seen.has(info.name)) continue;
    seen.add(info.name);
    panels.push(crestPanel(info.name, info.text || "", id));
  }
  for (const n of names) {
    if (seen.has(n)) continue;
    seen.add(n);
    panels.push(crestPanel(n, "", n));
  }
  if (!panels.length) return "";
  return `<div class="tooltip-crest-block">${panels.join("")}</div>`;
}

function crestPanel(name: string, text: string, id: string): string {
  const file = crestFile(id, name);
  const src = file ? publicUrl(`crests/${file}`) : "";
  const img = src
    ? `<img class="tooltip-crest-icon" src="${escapeHtml(src)}" alt="">`
    : `<div class="tooltip-crest-icon"></div>`;
  const body = text
    ? formatTooltipDescription(text)
    : "";
  return (
    `<div class="tooltip-crest-panel">${img}<div class="tooltip-crest-body">` +
    `<div class="tooltip-crest-name">${escapeHtml(name)}</div>` +
    `<div class="tooltip-crest-text">${body}</div></div></div>`
  );
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
