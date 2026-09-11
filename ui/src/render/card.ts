import { publicUrl } from "../base.ts";
import { crestFile, lookupText } from "../catalog.ts";
import { applyCardImage, escapeHtml } from "../images.ts";
import { formLetter } from "../info.ts";
import type { CardInstance, CrestInstance, HandCardInfo } from "../types.ts";

const GLOW_CLASSES = [
  "legal-play",
  "legal-attack",
  "can-attack",
  "playable-glow",
  "enhance-ready",
  "condition-ready",
  "alternate-ready",
  "rush-glow",
  "fuse-ready",
  "engage-ready",
  "selectable",
  "selected",
  "legal-target",
];

export function cardSignature(c: CardInstance | null, extras = ""): string {
  if (!c) return `empty|${extras}`;
  return [
    c.id,
    c.card,
    c.cost,
    c.attack,
    c.defense,
    c.max_defense,
    c.evolved ? 1 : 0,
    c.super_evolved ? 1 : 0,
    (c.traits || []).join(","),
    (c.printed_tags || []).join(","),
    c.countdown ?? "",
    c.flags?.attacks_left ?? "",
    c.flags?.ambush_active ? 1 : 0,
    extras,
  ].join("|");
}

export type CardPaintOpts = {
  inst: CardInstance;
  elementId: string;
  faceDown?: boolean;
  onBoard?: boolean;
  glow?: string;
  selected?: boolean;
  selectable?: boolean;
  displayCost?: number | null;
  form?: HandCardInfo["form"];
  entering?: boolean;
};

export function glowFor(opts: {
  playable?: boolean;
  canAttack?: boolean;
  gateMet?: boolean;
  form?: HandCardInfo["form"] | null;
}): string {
  const yellow = !!opts.gateMet;
  const green = !!opts.playable || !!opts.canAttack;
  const bits: string[] = [];
  if (yellow) bits.push("enhance-ready", "condition-ready");
  else if (green) bits.push("playable-glow");
  // Keep legal-* even under yellow so existing smokes / click handlers still match.
  if (opts.playable) bits.push("legal-play");
  if (opts.canAttack) bits.push("can-attack", "legal-attack");
  if (opts.form === "accelerate" || opts.form === "crystallize") bits.push("alternate-ready");
  return bits.join(" ");
}

export function renderCard(opts: CardPaintOpts): HTMLElement {
  const existing = document.getElementById(opts.elementId);
  if (
    existing &&
    existing.dataset.uid === String(opts.inst.id) &&
    existing.dataset.faceDown === (opts.faceDown ? "1" : undefined) &&
    !opts.faceDown
  ) {
    updateCard(existing, opts);
    return existing;
  }
  const el = buildCard(opts);
  if (opts.entering) el.classList.add("card-enter");
  return el;
}

function buildCard(opts: CardPaintOpts): HTMLElement {
  const { inst, elementId } = opts;
  if (opts.faceDown) {
    const div = document.createElement("div");
    div.className = "card card-back";
    div.id = elementId;
    div.dataset.uid = String(inst.id);
    div.dataset.faceDown = "1";
    const wrap = document.createElement("div");
    wrap.className = "card-image-wrapper";
    const back = document.createElement("div");
    back.className = "card-back-face";
    back.setAttribute("aria-label", "Face-down card");
    wrap.appendChild(back);
    div.appendChild(wrap);
    return div;
  }

  const div = document.createElement("div");
  div.className = "card hidpi";
  div.id = elementId;
  paintIdentity(div, opts);
  const wrap = document.createElement("div");
  wrap.className = "card-image-wrapper";
  const img = document.createElement("img");
  applyCardImage(img, inst.card, inst.evolved || inst.super_evolved, wrap);
  wrap.appendChild(img);
  const cost = document.createElement("div");
  cost.className = "card-stats top-left cost-badge";
  cost.textContent = String(opts.displayCost ?? inst.cost);
  wrap.appendChild(cost);
  paintStats(wrap, inst);
  applyOverlays(wrap, inst, !!opts.onBoard);
  paintFormBadge(wrap, opts.form);
  div.appendChild(wrap);
  div.dataset.name = lookupText(inst.card).name;
  applyChrome(div, opts);
  return div;
}

export function updateCard(div: HTMLElement, opts: CardPaintOpts): void {
  const { inst } = opts;
  if (opts.faceDown) return;
  paintIdentity(div, opts);
  applyChrome(div, opts);
  const wrap = div.querySelector<HTMLElement>(".card-image-wrapper");
  if (!wrap) return;
  const cost = wrap.querySelector<HTMLElement>(".cost-badge, .card-stats.top-left");
  if (cost) {
    const next = String(opts.displayCost ?? inst.cost);
    if (cost.textContent !== next) {
      cost.textContent = next;
      cost.classList.add("stat-flash");
      window.setTimeout(() => cost.classList.remove("stat-flash"), 160);
    }
  }
  const atk = wrap.querySelector<HTMLElement>(".card-stats.bottom-left");
  if (atk && inst.kind === "follower") {
    if (atk.textContent !== String(inst.attack)) {
      atk.textContent = String(inst.attack);
      atk.classList.add("stat-flash");
      window.setTimeout(() => atk.classList.remove("stat-flash"), 160);
    }
  }
  const def = wrap.querySelector<HTMLElement>(".card-stats.bottom-right:not(.countdown-badge)");
  if (def && inst.kind === "follower") {
    if (def.textContent !== String(inst.defense)) {
      def.textContent = String(inst.defense);
      def.classList.add("stat-flash");
      window.setTimeout(() => def.classList.remove("stat-flash"), 160);
    }
    def.classList.toggle("stat-damaged", inst.defense < inst.max_defense);
  }
  paintFormBadge(wrap, opts.form);
}

function paintIdentity(div: HTMLElement, opts: CardPaintOpts): void {
  const { inst, elementId } = opts;
  div.id = elementId;
  div.dataset.uid = String(inst.id);
  div.dataset.card = inst.card;
  delete div.dataset.faceDown;
  div.classList.toggle("spell", inst.kind === "spell");
  div.classList.toggle("super-evo", !!inst.super_evolved);
  div.classList.toggle("evolved", !!inst.evolved && !inst.super_evolved);
}

function applyChrome(div: HTMLElement, opts: CardPaintOpts): void {
  for (const cls of GLOW_CLASSES) div.classList.remove(cls);
  if (opts.glow) {
    for (const cls of opts.glow.split(/\s+/).filter(Boolean)) div.classList.add(cls);
  }
  if (opts.selectable) div.classList.add("selectable");
  if (opts.selected) div.classList.add("selected");
}

function paintStats(wrap: HTMLElement, inst: CardInstance): void {
  if (inst.kind === "follower") {
    const atk = document.createElement("div");
    atk.className = "card-stats bottom-left";
    atk.textContent = String(inst.attack);
    wrap.appendChild(atk);
    const def = document.createElement("div");
    def.className = "card-stats bottom-right";
    def.textContent = String(inst.defense);
    if (inst.defense < inst.max_defense) def.classList.add("stat-damaged");
    wrap.appendChild(def);
  } else if (inst.countdown != null) {
    const cd = document.createElement("div");
    cd.className = "card-stats bottom-right countdown-badge";
    cd.textContent = String(inst.countdown);
    wrap.appendChild(cd);
  }
}

function paintFormBadge(wrap: HTMLElement, form: HandCardInfo["form"] | undefined): void {
  wrap.querySelector(".alternate-form-badge")?.remove();
  const letter = formLetter(form ?? null);
  if (!letter) return;
  const badge = document.createElement("div");
  badge.className = "alternate-form-badge";
  badge.textContent = letter;
  badge.dataset.form = form ?? "";
  wrap.appendChild(badge);
}

function applyOverlays(wrap: HTMLElement, inst: CardInstance, onBoard: boolean): void {
  const traits = new Set(inst.traits || []);
  const tags = new Set((inst.printed_tags || []).map((t) => String(t).toLowerCase()));
  if (onBoard && traits.has("ward")) {
    const o = document.createElement("div");
    o.className = "ward-overlay";
    wrap.appendChild(o);
  }
  if (onBoard && (traits.has("ambush") || inst.flags?.ambush_active)) {
    const o = document.createElement("div");
    o.className = "ambush-overlay";
    wrap.appendChild(o);
  }
  if (onBoard && traits.has("aura")) {
    const o = document.createElement("div");
    o.className = "aura-overlay";
    wrap.appendChild(o);
  }
  if (onBoard && traits.has("intimidate")) {
    const o = document.createElement("div");
    o.className = "intimidate-overlay";
    wrap.appendChild(o);
  }
  if (onBoard && traits.has("cantAttackFollowers") && traits.has("cantAttackLeader")) {
    const o = document.createElement("div");
    o.className = "cant_attack-overlay";
    wrap.appendChild(o);
  }
  const stack = document.createElement("div");
  stack.className = "keyword-icon-stack";
  const add = (src: string, cls: string) => {
    const img = document.createElement("img");
    img.src = src;
    img.className = `keyword-icon ${cls}`;
    img.alt = cls;
    stack.appendChild(img);
  };
  if (traits.has("bane")) add(publicUrl("images/icon_bane.png"), "bane-icon");
  if (traits.has("drain")) add(publicUrl("images/icon_drain.png"), "drain-icon");
  if (tags.has("lastwords") || tags.has("last_words") || tags.has("lastWords")) {
    add(publicUrl("images/icon_last-words.png"), "lastwords-icon");
  }
  if (stack.childElementCount) wrap.appendChild(stack);
}

export function renderCrestSlot(slot: HTMLElement, crest: CrestInstance | undefined): void {
  slot.innerHTML = "";
  slot.onmouseenter = null;
  slot.onmousemove = null;
  slot.onmouseleave = null;
  if (!crest) return;
  const info = lookupText(crest.id);
  const file = crestFile(crest.id, info.name);
  if (file) {
    const img = document.createElement("img");
    img.className = "crest-image";
    img.src = publicUrl(`crests/${file}`);
    img.alt = info.name;
    img.referrerPolicy = "no-referrer";
    slot.appendChild(img);
  } else {
    const fb = document.createElement("div");
    fb.className = "card-fallback";
    fb.innerHTML = `<div class="fb-name">${escapeHtml(info.name)}</div>`;
    slot.appendChild(fb);
  }
  if (crest.countdown != null) {
    const badge = document.createElement("div");
    badge.className = "crest-countdown";
    badge.textContent = String(Math.max(0, crest.countdown));
    slot.appendChild(badge);
  }
}
