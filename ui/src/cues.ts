import { hostFor, type CombatTarget } from "./fct.ts";
import { byId, visual } from "./render/ids.ts";
import type {
  ChooseOption,
  EngineEvent,
  FullState,
  NeutralAction,
  PlayerId,
  SessionConfig,
} from "./types.ts";

export type PlayCuesPace = "off" | "fast" | "normal" | "slow";

export const CUE_PACE: Record<
  Exclude<PlayCuesPace, "off">,
  { staggerMs: number; holdMs: number }
> = {
  fast: { staggerMs: 120, holdMs: 600 },
  normal: { staggerMs: 120, holdMs: 1000 },
  slow: { staggerMs: 120, holdMs: 1800 },
};

export type CueTarget =
  | { kind: "leader"; player: PlayerId }
  | { kind: "slot"; player: PlayerId; slot?: number; id?: number }
  | { kind: "hand"; player: PlayerId; pos?: number; id?: number }
  | { kind: "spell"; player: PlayerId; card: string };

export type CueArrowKind = "attack" | "choice" | "random" | "effect";

export type CueSpotlight = { type: "spotlight"; target: CueTarget };
export type CueArrow = { type: "arrow"; kind: CueArrowKind; from: CueTarget; to: CueTarget };
export type CueFade = { type: "fade"; target: CueTarget; card: string };

export type Cue = CueSpotlight | CueArrow | CueFade;

export type PlannedCue = { at: number; cue: Cue };

export type CuePlan = {
  cues: PlannedCue[];
  totalMs: number;
};

export type CueStep = {
  action: NeutralAction;
  events: EngineEvent[];
  human: boolean;
  before: FullState;
};

type ParsedSource =
  | { kind: "field"; player: PlayerId; id: number; card?: string }
  | { kind: "spell"; player: PlayerId; card: string }
  | { kind: "combat"; player: PlayerId; id: number }
  | { kind: "leader"; player: PlayerId }
  | { kind: "hand"; player: PlayerId; id: number }
  | { kind: "crest"; player: PlayerId; index: number };

export type CueLogEntry = {
  t: number;
  human: boolean;
  duration: number;
  plan: CuePlan;
};

let cueLogEntries: CueLogEntry[] = [];
const cueTimers: number[] = [];

export function cueLog(): CueLogEntry[] {
  return cueLogEntries;
}

function pushCueLog(step: CueStep, plan: CuePlan): void {
  cueLogEntries.push({
    t: Date.now(),
    human: step.human,
    duration: plan.totalMs,
    plan,
  });
}

export function cueTargetKey(t: CueTarget): string {
  switch (t.kind) {
    case "leader":
      return `leader:${t.player}`;
    case "slot":
      return typeof t.id === "number"
        ? `uid:${t.player}:${t.id}`
        : `slot:${t.player}:${t.slot ?? "?"}`;
    case "hand":
      return typeof t.id === "number"
        ? `handuid:${t.player}:${t.id}`
        : `hand:${t.player}:${t.pos ?? "?"}`;
    case "spell":
      return `spell:${t.player}:${t.card}`;
  }
}

export function planCues(
  step: CueStep,
  _afterState: FullState,
  config: SessionConfig,
  _sessionEvents: EngineEvent[],
): CuePlan {
  const pace = readPace();
  if (pace === "off") return emptyPlan(step);

  const cues: PlannedCue[] = [];
  let at = 0;
  const stagger = CUE_PACE[pace].staggerMs;

  const push = (cue: Cue) => {
    if (!cueVisible(config, cue)) return;
    cues.push({ at, cue });
    at += stagger;
  };

  const attackCue = planAttackArrow(step.action);
  if (attackCue) push(attackCue);

  const choiceCue = planChoiceArrow(step.action, step.events);
  if (choiceCue) push(choiceCue);

  let lastResolve: ParsedSource | null = null;
  for (let i = 0; i < step.events.length; i++) {
    const ev = step.events[i]!;
    if ("resolve" in ev) {
      const src = parseResolveSource((ev.resolve as { source: unknown }).source);
      if (src) {
        lastResolve = src;
        const target = sourceToTarget(src, step.before);
        if (target) push({ type: "spotlight", target });
      }
      continue;
    }

    if ("random_pick" in ev) {
      const rp = ev.random_pick as { target: Record<string, unknown> };
      const to = eventTargetToCue(rp.target, step.before);
      const from = effectiveSource(step.events, i, lastResolve, step.before);
      if (from && to) push({ type: "arrow", kind: "random", from, to });
      continue;
    }

    if ("damage" in ev || "restore" in ev) {
      const body = ("damage" in ev ? ev.damage : ev.restore) as {
        target: CombatTarget;
      };
      const to = combatTargetToCue(body.target);
      const from = effectiveSource(step.events, i, lastResolve, step.before);
      if (from && to) push({ type: "arrow", kind: "effect", from, to });
      continue;
    }

    if ("destroy" in ev) {
      const d = ev.destroy as { slot: number; card: string; player: PlayerId; id: number };
      const target: CueTarget = { kind: "slot", player: d.player, slot: d.slot, id: d.id };
      push({ type: "fade", target, card: d.card });
      continue;
    }

    if ("play" in ev && !step.human) {
      const p = ev.play as { player: PlayerId; card: string };
      if (!handHidden(config, p.player)) {
        const hand = step.before.players[p.player].hand;
        const idx = hand.findIndex((c) => c.card === p.card);
        if (idx >= 0) {
          push({
            type: "spotlight",
            target: { kind: "hand", player: p.player, pos: idx, id: hand[idx]!.id },
          });
        }
      }
    }
  }

  const hold = cues.length ? CUE_PACE[pace].holdMs : 0;
  const tail = cues.length ? cues[cues.length - 1]!.at : 0;
  const plan = { cues, totalMs: tail + hold };
  pushCueLog(step, plan);
  return plan;
}

function emptyPlan(step: CueStep): CuePlan {
  const plan = { cues: [], totalMs: 0 };
  pushCueLog(step, plan);
  return plan;
}

function planAttackArrow(action: NeutralAction): CueArrow | null {
  if (!("attack" in action)) return null;
  const a = action.attack;
  const from: CueTarget = { kind: "slot", player: a.player, slot: a.attacker_slot };
  const to: CueTarget =
    a.target === "leader"
      ? { kind: "leader", player: a.player === "a" ? "b" : "a" }
      : { kind: "slot", player: a.player === "a" ? "b" : "a", slot: a.target.slot };
  return { type: "arrow", kind: "attack", from, to };
}

function planChoiceArrow(action: NeutralAction, events: EngineEvent[]): CueArrow | null {
  if (!("choose" in action)) return null;
  const opt = action.choose.option;
  const to = chooseOptionToTarget(opt, action.choose.player);
  if (!to) return null;
  const from = findChoiceSource(events, action.choose.player);
  if (!from) return null;
  return { type: "arrow", kind: "choice", from, to };
}

function findChoiceSource(events: EngineEvent[], player: PlayerId): CueTarget | null {
  for (let i = events.length - 1; i >= 0; i--) {
    const ev = events[i]!;
    if ("choice_offered" in ev) {
      const co = ev.choice_offered as { player: PlayerId; source?: Record<string, unknown> };
      if (co.player !== player) continue;
      if (co.source) {
        const parsed = parseResolveSource(co.source);
        if (parsed) return sourceToTarget(parsed, null);
      }
      break;
    }
    if ("resolve" in ev) {
      const parsed = parseResolveSource((ev.resolve as { source: unknown }).source);
      if (parsed) return sourceToTarget(parsed, null);
      break;
    }
  }
  return null;
}

function effectiveSource(
  events: EngineEvent[],
  index: number,
  lastResolve: ParsedSource | null,
  before: FullState,
): CueTarget | null {
  let src = lastResolve;
  for (let j = index - 1; j >= 0; j--) {
    const ev = events[j]!;
    if ("resolve" in ev) {
      src = parseResolveSource((ev.resolve as { source: unknown }).source);
      break;
    }
  }
  if (!src) return null;

  if (src.kind === "spell") {
    const destroy = findMatchingDestroyBefore(events, index, src.player, src.card);
    if (destroy) {
      return { kind: "slot", player: destroy.player, slot: destroy.slot, id: destroy.id };
    }
  }

  return sourceToTarget(src, before);
}

function findMatchingDestroyBefore(
  events: EngineEvent[],
  beforeIndex: number,
  player: PlayerId,
  card: string,
): { player: PlayerId; slot: number; id: number } | null {
  for (let j = beforeIndex - 1; j >= 0; j--) {
    const ev = events[j]!;
    if ("destroy" in ev) {
      const d = ev.destroy as { player: PlayerId; card: string; slot: number; id: number };
      if (d.player === player && d.card === card) return d;
    }
    if ("resolve" in ev) break;
  }
  return null;
}

function parseResolveSource(raw: unknown): ParsedSource | null {
  if (!raw || typeof raw !== "object") return null;
  const o = raw as Record<string, unknown>;
  if ("field" in o && o.field && typeof o.field === "object") {
    const f = o.field as { player: PlayerId; id: number; card?: string };
    return { kind: "field", player: f.player, id: f.id, card: f.card };
  }
  if ("spell" in o && o.spell && typeof o.spell === "object") {
    const s = o.spell as { player: PlayerId; card: string };
    return { kind: "spell", player: s.player, card: s.card };
  }
  if ("combat" in o && o.combat && typeof o.combat === "object") {
    const c = o.combat as { player: PlayerId; id: number };
    return { kind: "combat", player: c.player, id: c.id };
  }
  if ("leader" in o) return { kind: "leader", player: o.leader as PlayerId };
  if ("hand" in o && o.hand && typeof o.hand === "object") {
    const h = o.hand as { player: PlayerId; id: number };
    return { kind: "hand", player: h.player, id: h.id };
  }
  if ("crest" in o && o.crest && typeof o.crest === "object") {
    const c = o.crest as { player: PlayerId; index: number };
    return { kind: "crest", player: c.player, index: c.index };
  }
  return null;
}

function sourceToTarget(src: ParsedSource, before: FullState | null): CueTarget | null {
  switch (src.kind) {
    case "field":
    case "combat":
      return { kind: "slot", player: src.player, id: src.id };
    case "spell":
      return { kind: "spell", player: src.player, card: src.card };
    case "leader":
      return { kind: "leader", player: src.player };
    case "hand":
      return { kind: "hand", player: src.player, id: src.id };
    case "crest":
      if (!before) return { kind: "leader", player: src.player };
      return { kind: "leader", player: src.player };
  }
}

function combatTargetToCue(target: CombatTarget): CueTarget | null {
  if (target.leader) return { kind: "leader", player: target.leader };
  if (!target.player) return null;
  if (typeof target.id === "number") {
    return { kind: "slot", player: target.player, id: target.id, slot: target.slot };
  }
  if (typeof target.slot === "number") {
    return { kind: "slot", player: target.player, slot: target.slot };
  }
  return null;
}

function eventTargetToCue(raw: Record<string, unknown>, before: FullState): CueTarget | null {
  if ("leader" in raw) return { kind: "leader", player: raw.leader as PlayerId };
  if ("slot" in raw) {
    const player = raw.player as PlayerId;
    const slot = raw.slot as number;
    const id = raw.id as number | undefined;
    return { kind: "slot", player, slot, id };
  }
  if ("hand" in raw && raw.hand && typeof raw.hand === "object") {
    const h = raw.hand as { player: PlayerId; pos: number };
    return { kind: "hand", player: h.player, pos: h.pos };
  }
  if ("deck" in raw && raw.deck && typeof raw.deck === "object") {
    const d = raw.deck as { player: PlayerId; id: number };
    const inst = before.players[d.player].deck.find((c) => c.id === d.id);
    if (inst) return { kind: "spell", player: d.player, card: inst.card };
  }
  return null;
}

function chooseOptionToTarget(opt: ChooseOption, acting: PlayerId): CueTarget | null {
  if (opt === "leader") return { kind: "leader", player: acting === "a" ? "b" : "a" };
  if (typeof opt === "object" && opt !== null) {
    if ("slot" in opt) {
      const player = opt.player ?? (acting === "a" ? "b" : "a");
      return { kind: "slot", player, slot: opt.slot };
    }
    if ("card" in opt) return { kind: "spell", player: acting, card: opt.card };
  }
  return null;
}

function botSide(config: SessionConfig): PlayerId {
  return config.humanSide === "a" ? "b" : "a";
}

function handHidden(config: SessionConfig, player: PlayerId): boolean {
  return config.mode === "vs-bot" && config.hideBotHand && player === botSide(config);
}

function targetHidden(config: SessionConfig, target: CueTarget): boolean {
  if (handHidden(config, target.player)) {
    if (target.kind === "hand") return true;
    if (target.kind === "spell") return true;
  }
  return false;
}

function cueVisible(config: SessionConfig, cue: Cue): boolean {
  switch (cue.type) {
    case "spotlight":
      return !targetHidden(config, cue.target);
    case "fade":
      return !targetHidden(config, cue.target);
    case "arrow":
      return !targetHidden(config, cue.from) && !targetHidden(config, cue.to);
  }
}

export function readPace(): PlayCuesPace {
  try {
    const v = localStorage.getItem("svwb.playCues");
    if (v === "off" || v === "fast" || v === "normal" || v === "slow") return v;
  } catch {
    /* private mode */
  }
  return "normal";
}

export function collectPlanTargetKeys(plan: CuePlan): Set<string> {
  const keys = new Set<string>();
  for (const { cue } of plan.cues) {
    if (cue.type === "spotlight" || cue.type === "fade") keys.add(cueTargetKey(cue.target));
    if (cue.type === "arrow") {
      keys.add(cueTargetKey(cue.from));
      keys.add(cueTargetKey(cue.to));
    }
  }
  return keys;
}

export function captureCueRects(plan: CuePlan): Map<string, DOMRect> {
  const rects = new Map<string, DOMRect>();
  for (const key of collectPlanTargetKeys(plan)) {
    const el = elementForKey(key);
    if (el) rects.set(key, el.getBoundingClientRect());
  }
  return rects;
}

function elementForKey(key: string): HTMLElement | null {
  if (key.startsWith("leader:")) {
    const player = key.slice(7) as PlayerId;
    return document.getElementById(`${visual(player)}Leader`);
  }
  if (key.startsWith("uid:")) {
    const [, player, id] = key.split(":");
    const board = document.getElementById(`${visual(player as PlayerId)}Board`);
    return board?.querySelector<HTMLElement>(`.card[data-uid="${id}"]`) ?? null;
  }
  if (key.startsWith("slot:")) {
    const [, player, slot] = key.split(":");
    return hostFor({ player: player as PlayerId, slot: Number(slot) });
  }
  if (key.startsWith("handuid:")) {
    const [, player, id] = key.split(":");
    const hand = document.getElementById(`${visual(player as PlayerId)}Hand`);
    return hand?.querySelector<HTMLElement>(`.card[data-uid="${id}"]`) ?? null;
  }
  if (key.startsWith("hand:")) {
    const [, player, pos] = key.split(":");
    const hand = document.getElementById(`${visual(player as PlayerId)}Hand`);
    const cards = hand?.querySelectorAll<HTMLElement>(".card");
    const idx = Number(pos);
    return cards?.[idx] ?? null;
  }
  if (key.startsWith("spell:")) {
    const [, player] = key.split(":");
    return document.getElementById(`${visual(player as PlayerId)}Leader`);
  }
  return null;
}

function centerOf(rect: DOMRect): { x: number; y: number } {
  return { x: rect.left + rect.width / 2, y: rect.top + rect.height / 2 };
}

export function clearCueLog(): void {
  cueLogEntries = [];
}

export function clearCues(): void {
  for (const t of cueTimers) window.clearTimeout(t);
  cueTimers.length = 0;
  const layer = byId("cueLayer");
  if (layer) layer.replaceChildren();
}

export function playCuePlan(plan: CuePlan, pace: PlayCuesPace, rects: Map<string, DOMRect>): void {
  clearCues();
  if (pace === "off" || !plan.cues.length) return;
  const layer = byId("cueLayer");
  if (!layer) return;

  let svg = layer.querySelector<SVGSVGElement>(".cue-arrows");
  if (!svg) {
    svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
    svg.classList.add("cue-arrows");
    svg.setAttribute("aria-hidden", "true");
    layer.appendChild(svg);
  }
  const syncSvg = () => {
    svg!.setAttribute("width", String(window.innerWidth));
    svg!.setAttribute("height", String(window.innerHeight));
  };
  syncSvg();

  for (const { at, cue } of plan.cues) {
    const show = window.setTimeout(() => {
      syncSvg();
      if (cue.type === "spotlight") renderSpotlight(layer, cue, rects);
      if (cue.type === "arrow") renderArrow(svg!, cue, rects);
      if (cue.type === "fade") renderFade(layer, cue, rects);
    }, at);
    cueTimers.push(show);
  }

  const done = window.setTimeout(() => clearCues(), plan.totalMs);
  cueTimers.push(done);
}

function renderSpotlight(layer: HTMLElement, cue: CueSpotlight, rects: Map<string, DOMRect>): void {
  const rect = rects.get(cueTargetKey(cue.target));
  if (!rect) return;
  const el = document.createElement("div");
  el.className = "cue-spotlight";
  el.style.left = `${rect.left}px`;
  el.style.top = `${rect.top}px`;
  el.style.width = `${rect.width}px`;
  el.style.height = `${rect.height}px`;
  layer.appendChild(el);
  window.setTimeout(() => el.remove(), 700);
}

function renderArrow(svg: SVGSVGElement, cue: CueArrow, rects: Map<string, DOMRect>): void {
  const fromRect = rects.get(cueTargetKey(cue.from));
  const toRect = rects.get(cueTargetKey(cue.to));
  if (!fromRect || !toRect) return;
  const a = centerOf(fromRect);
  const b = centerOf(toRect);
  const path = document.createElementNS("http://www.w3.org/2000/svg", "path");
  path.setAttribute("class", `cue-arrow cue-arrow--${cue.kind}`);
  path.dataset.why = cue.kind;
  path.dataset.style = cue.kind;
  path.dataset.from = cueTargetKey(cue.from);
  path.dataset.to = cueTargetKey(cue.to);
  path.dataset.x1 = String(Math.round(a.x));
  path.dataset.y1 = String(Math.round(a.y));
  path.dataset.x2 = String(Math.round(b.x));
  path.dataset.y2 = String(Math.round(b.y));
  const dx = b.x - a.x;
  const dy = b.y - a.y;
  const cx = a.x + dx * 0.35;
  const cy = a.y + dy * 0.15 - Math.abs(dx) * 0.08;
  path.setAttribute("d", `M ${a.x} ${a.y} Q ${cx} ${cy} ${b.x} ${b.y}`);
  svg.appendChild(path);
  window.setTimeout(() => path.remove(), 900);
}

function renderFade(layer: HTMLElement, cue: CueFade, rects: Map<string, DOMRect>): void {
  const rect = rects.get(cueTargetKey(cue.target));
  if (!rect) return;
  const el = document.createElement("div");
  el.className = "cue-fade";
  el.dataset.card = cue.card;
  el.style.left = `${rect.left}px`;
  el.style.top = `${rect.top}px`;
  el.style.width = `${rect.width}px`;
  el.style.height = `${rect.height}px`;
  layer.appendChild(el);
  window.setTimeout(() => el.remove(), 1100);
}
