import { publicUrl } from "../base.ts";
import { crestFile, getCatalog, lookupText } from "../catalog.ts";
import { applyCardImage, escapeHtml } from "../images.ts";
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
    c.spellboost_count ?? "",
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
  cannotAttack?: boolean;
};

export function glowFor(opts: {
  playable?: boolean;
  canAttack?: boolean;
  rushOnly?: boolean;
  yellow?: boolean;
  form?: HandCardInfo["form"] | null;
}): string {
  const bits: string[] = [];
  if (opts.playable) {
    bits.push("legal-play");
    if (opts.yellow || (opts.form && opts.form !== "normal")) {
      bits.push("enhance-ready");
      if (opts.form === "accelerate" || opts.form === "crystallize") bits.push("alternate-ready");
    } else {
      bits.push("playable-glow");
    }
  }
  if (opts.canAttack) {
    bits.push("legal-attack");
    if (opts.rushOnly) bits.push("rush-glow");
    else bits.push("can-attack");
  }
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
  paintWrapper(wrap, opts);
  div.appendChild(wrap);
  div.dataset.name = lookupText(inst.card).name;
  div.dataset.sig = cardSignature(inst, String(opts.displayCost ?? ""));
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
  const nextSig = cardSignature(inst, String(opts.displayCost ?? ""));
  const prevAtk = wrap.querySelector<HTMLElement>(".card-stats.bottom-left")?.textContent;
  const prevDef = wrap.querySelector<HTMLElement>(
    ".card-stats.bottom-right:not(.countdown-badge)",
  )?.textContent;
  const prevCost = wrap.querySelector<HTMLElement>(".cost-badge, .card-stats.top-left")?.textContent;
  if (div.dataset.sig !== nextSig) {
    paintWrapper(wrap, opts);
    div.dataset.sig = nextSig;
    const cost = wrap.querySelector<HTMLElement>(".cost-badge, .card-stats.top-left");
    if (cost && prevCost != null && cost.textContent !== prevCost) flash(cost);
    const atk = wrap.querySelector<HTMLElement>(".card-stats.bottom-left");
    if (atk && prevAtk != null && atk.textContent !== prevAtk) flash(atk);
    const def = wrap.querySelector<HTMLElement>(".card-stats.bottom-right:not(.countdown-badge)");
    if (def && prevDef != null && def.textContent !== prevDef) flash(def);
  }
}

function flash(el: HTMLElement): void {
  el.classList.add("stat-flash");
  window.setTimeout(() => el.classList.remove("stat-flash"), 160);
}

function paintWrapper(wrap: HTMLElement, opts: CardPaintOpts): void {
  const { inst } = opts;
  wrap.replaceChildren();
  const img = document.createElement("img");
  applyCardImage(img, inst.card, inst.evolved || inst.super_evolved, wrap);
  wrap.appendChild(img);
  const cost = document.createElement("div");
  cost.className = "card-stats top-left cost-badge";
  cost.textContent = String(opts.displayCost ?? inst.cost);
  wrap.appendChild(cost);
  if (inst.spellboost_count >= 1) {
    const badge = document.createElement("div");
    badge.className = "spellboost-badge";
    badge.textContent = String(inst.spellboost_count);
    wrap.appendChild(badge);
  }
  paintStats(wrap, inst);
  applyOverlays(wrap, inst, !!opts.onBoard, !!opts.cannotAttack);
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
  const keepFlash = div.classList.contains("floating-combat-flash");
  for (const cls of GLOW_CLASSES) div.classList.remove(cls);
  if (keepFlash) div.classList.add("floating-combat-flash");
  if (opts.glow) {
    for (const cls of opts.glow.split(/\s+/).filter(Boolean)) div.classList.add(cls);
  }
  if (opts.selectable) div.classList.add("selectable");
  if (opts.selected) div.classList.add("selected");
  let check = div.querySelector<HTMLElement>(".selected-check");
  if (opts.selected) {
    if (!check) {
      check = document.createElement("span");
      check.className = "selected-check";
      check.textContent = "✓";
      check.setAttribute("aria-hidden", "true");
      div.appendChild(check);
    }
  } else {
    check?.remove();
  }
}

function printedStats(inst: CardInstance): { attack: number; defense: number } {
  const cat = getCatalog(inst.card);
  const text = lookupText(inst.card);
  return {
    attack: cat?.attack ?? text.attack ?? inst.attack,
    defense: cat?.defense ?? text.defense ?? inst.defense,
  };
}

export function applyStatColors(el: HTMLElement, inst: CardInstance, which: "atk" | "def"): void {
  const printed = printedStats(inst);
  el.classList.remove("stat-buffed", "stat-damaged");
  if (which === "atk") {
    if (inst.attack > printed.attack) el.classList.add("stat-buffed");
    else if (inst.attack < printed.attack) el.classList.add("stat-damaged");
    return;
  }
  if (inst.defense < inst.max_defense) el.classList.add("stat-damaged");
  else if (inst.defense > printed.defense) el.classList.add("stat-buffed");
}

function paintStats(wrap: HTMLElement, inst: CardInstance): void {
  if (inst.kind === "follower") {
    const atk = document.createElement("div");
    atk.className = "card-stats bottom-left";
    atk.textContent = String(inst.attack);
    applyStatColors(atk, inst, "atk");
    wrap.appendChild(atk);
    const def = document.createElement("div");
    def.className = "card-stats bottom-right";
    def.textContent = String(inst.defense);
    applyStatColors(def, inst, "def");
    wrap.appendChild(def);
  } else if (inst.countdown != null) {
    const cd = document.createElement("div");
    cd.className = "card-stats bottom-right countdown-badge";
    cd.textContent = String(inst.countdown);
    wrap.appendChild(cd);
  }
}

function applyOverlays(
  wrap: HTMLElement,
  inst: CardInstance,
  onBoard: boolean,
  cannotAttack: boolean,
): void {
  const traits = new Set(inst.traits || []);
  const tags = new Set((inst.printed_tags || []).map((t) => String(t).toLowerCase()));
  const textTags = new Set((lookupText(inst.card).tags || []).map((t) => String(t).toLowerCase()));
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
  const locked =
    cannotAttack ||
    traits.has("cantAttackFollowers") ||
    traits.has("cantAttackLeader") ||
    traits.has("cantAttack");
  if (onBoard && locked) {
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
  if (
    textTags.has("ongoing") ||
    tags.has("ongoing") ||
    tags.has("static") ||
    /ongoing/i.test(lookupText(inst.card).text || "")
  ) {
    add(publicUrl("images/icon_ongoing.png"), "ongoing-icon");
  }
  const n = stack.childElementCount;
  if (n >= 4) stack.classList.add("swap-4");
  else if (n === 3) stack.classList.add("swap-3");
  else if (n === 2) stack.classList.add("swap-2");
  if (n) wrap.appendChild(stack);
}

export function renderCrestSlot(
  slot: HTMLElement,
  crest: CrestInstance | undefined,
  faithValue?: number,
): void {
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
  if (crest.faith) {
    const fb = document.createElement("div");
    fb.className = "crest-faith";
    fb.textContent = String(Math.max(0, faithValue ?? 0));
    slot.appendChild(fb);
  }
  if (crest.countdown != null) {
    const badge = document.createElement("div");
    badge.className = "crest-countdown";
    badge.textContent = String(Math.max(0, crest.countdown));
    slot.appendChild(badge);
  }
}
