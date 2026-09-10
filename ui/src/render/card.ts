import { crestFile, lookupText } from "../catalog.ts";
import { applyCardImage, escapeHtml } from "../images.ts";
import type { CardInstance, CrestInstance } from "../types.ts";

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

export function renderCard(opts: {
  inst: CardInstance;
  elementId: string;
  faceDown?: boolean;
  onBoard?: boolean;
  glow?: string;
  selected?: boolean;
  selectable?: boolean;
}): HTMLElement {
  const { inst, elementId } = opts;
  if (opts.faceDown) {
    const div = document.createElement("div");
    div.className = "card card-back";
    div.id = elementId;
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
  div.dataset.uid = String(inst.id);
  div.dataset.card = inst.card;
  if (inst.kind === "spell") div.classList.add("spell");
  if (inst.super_evolved) div.classList.add("super-evo");
  else if (inst.evolved) div.classList.add("evolved");
  if (opts.glow) {
    for (const cls of opts.glow.split(/\s+/).filter(Boolean)) div.classList.add(cls);
  }
  if (opts.selectable) div.classList.add("selectable");
  if (opts.selected) div.classList.add("selected");

  const wrap = document.createElement("div");
  wrap.className = "card-image-wrapper";
  const img = document.createElement("img");
  applyCardImage(img, inst.card, inst.evolved || inst.super_evolved, wrap);
  wrap.appendChild(img);

  const cost = document.createElement("div");
  cost.className = "card-stats top-left";
  cost.textContent = String(inst.cost);
  wrap.appendChild(cost);

  if (inst.kind === "follower") {
    const atk = document.createElement("div");
    atk.className = "card-stats bottom-left";
    atk.textContent = String(inst.attack);
    if (inst.attack > (lookupAtkBase(inst))) atk.classList.add("stat-buffed");
    else if (inst.attack < lookupAtkBase(inst)) atk.classList.add("stat-damaged");
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

  applyOverlays(wrap, inst, !!opts.onBoard);
  div.appendChild(wrap);
  div.dataset.name = lookupText(inst.card).name;
  return div;
}

function lookupAtkBase(inst: CardInstance): number {
  // Printed ATK is not always on the instance; treat max as current when unevolved.
  return inst.evolved || inst.super_evolved ? inst.attack : inst.attack;
}

function applyOverlays(wrap: HTMLElement, inst: CardInstance, onBoard: boolean): void {
  const traits = new Set(inst.traits || []);
  const tags = new Set(
    (inst.printed_tags || []).map((t) => String(t).toLowerCase()),
  );
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
  if (traits.has("bane")) add("/images/icon_bane.png", "bane-icon");
  if (traits.has("drain")) add("/images/icon_drain.png", "drain-icon");
  if (tags.has("lastwords") || tags.has("last_words") || tags.has("lastWords")) {
    add("/images/icon_last-words.png", "lastwords-icon");
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
    img.src = `/crests/${file}`;
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
