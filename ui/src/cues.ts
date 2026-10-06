import { byId, visual } from "./render/ids.ts";
import { lookupText } from "./catalog.ts";
import { applyCardImage } from "./images.ts";
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

export type CueEndpoint =
  | { kind: "uid"; uid: number; player: PlayerId }
  | { kind: "leader"; player: PlayerId }
  | { kind: "hand"; uid: number; player: PlayerId };

export type CueArrowKind = "attack" | "choice" | "random" | "effect";

export type CuePlaySpotlight = {
  type: "play-spotlight";
  player: PlayerId;
  card: string;
};

export type CueArrow = {
  type: "arrow";
  kind: CueArrowKind;
  from: CueEndpoint;
  to: CueEndpoint;
  dashed?: boolean;
};

export type CueFade = { type: "fade"; endpoint: CueEndpoint; card: string };

export type Cue = CuePlaySpotlight | CueArrow | CueFade;

export type PlannedCue = { at: number; cue: Cue };

export type CuePlan = {
  cues: PlannedCue[];
  durationMs: number;
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
let lastCuePlan: CuePlan | null = null;
let cueClearTimer: number | null = null;
const cueTimers: number[] = [];

export function cueLog(): CueLogEntry[] {
  return cueLogEntries;
}

export function cues(): CuePlan | null {
  return lastCuePlan;
}

export function cueHoldMs(pace: PlayCuesPace = readPace()): number {
  if (pace === "off") return 0;
  return CUE_PACE[pace].holdMs;
}

function pushCueLog(step: CueStep, plan: CuePlan, duration: number): void {
  cueLogEntries.push({
    t: performance.now(),
    human: step.human,
    duration,
    plan,
  });
  lastCuePlan = plan;
}

export function endpointKey(ep: CueEndpoint): string {
  switch (ep.kind) {
    case "leader":
      return `leader:${ep.player}`;
    case "uid":
      return `uid:${ep.uid}`;
    case "hand":
      return `hand:${ep.uid}`;
  }
}

export function planCues(
  step: CueStep,
  afterState: FullState,
  config: SessionConfig,
  sessionEvents: EngineEvent[],
): CuePlan {
  const pace = readPace();
  const hold = pace === "off" ? 0 : CUE_PACE[pace].holdMs;
  if (pace === "off") {
    const plan = { cues: [], durationMs: 0 };
    pushCueLog(step, plan, 0);
    return plan;
  }

  const cues: PlannedCue[] = [];
  let at = 0;
  const stagger = CUE_PACE[pace].staggerMs;
  const botPlayers = nonHumanPlayers(config);
  const arrowPairs = new Set<string>();

  const push = (cue: Cue) => {
    if (!cueVisible(config, cue)) return;
    if (cue.type === "arrow") {
      const pair = `${endpointKey(cue.from)}->${endpointKey(cue.to)}`;
      if (arrowPairs.has(pair)) return;
      arrowPairs.add(pair);
    }
    if (!endpointOnBoard(cue, step.before, afterState)) return;
    cues.push({ at, cue });
    at += stagger;
  };

  if (!step.human) {
    const attackCue = planAttackArrow(step.action, step.before);
    if (attackCue) push(attackCue);

    const choiceCue = planChoiceArrow(
      step.action,
      step.before,
      sessionEvents,
      step.events,
    );
    if (choiceCue) push(choiceCue);
  }

  const matchedLwDestroys = new Set<string>();

  let lastResolve: ParsedSource | null = null;
  let lastResolveCombat = false;

  for (let i = 0; i < step.events.length; i++) {
    const ev = step.events[i]!;
    if ("resolve" in ev) {
      const src = parseResolveSource((ev.resolve as { source: unknown }).source);
      lastResolve = src;
      lastResolveCombat = src?.kind === "combat";
      continue;
    }

    if ("random_pick" in ev) {
      if (step.human) continue;
      const rp = ev.random_pick as { target: Record<string, unknown> };
      if (isHiddenRandomTarget(rp.target)) continue;
      const to = eventTargetToEndpoint(rp.target, step.before);
      const from = effectiveSource(
        step.events,
        i,
        lastResolve,
        step.before,
        config,
        matchedLwDestroys,
      );
      if (from && to) {
        push({ type: "arrow", kind: "random", from, to, dashed: true });
      }
      continue;
    }

    if ("damage" in ev || "restore" in ev) {
      if (lastResolveCombat) continue;
      const body = ("damage" in ev ? ev.damage : ev.restore) as {
        target: Record<string, unknown>;
      };
      const src = effectiveSource(
        step.events,
        i,
        lastResolve,
        step.before,
        config,
        matchedLwDestroys,
      );
      if (!src || !sourcePlayer(src) || !botPlayers.has(sourcePlayer(src)!)) continue;
      const to = combatTargetToEndpoint(body.target as CombatTargetJson);
      if (src && to) {
        push({ type: "arrow", kind: "effect", from: src, to });
      }
      continue;
    }

    if ("destroy" in ev) {
      const d = ev.destroy as { slot: number; card: string; player: PlayerId; id: number };
      const endpoint: CueEndpoint = { kind: "uid", uid: d.id, player: d.player };
      if (unitOnBoard(step.before, d.player, d.id)) {
        push({ type: "fade", endpoint, card: d.card });
      }
      continue;
    }

    if ("play" in ev && !step.human) {
      const p = ev.play as { player: PlayerId; card: string };
      push({ type: "play-spotlight", player: p.player, card: p.card });
    }
  }

  const plan = { cues, durationMs: hold };
  pushCueLog(step, plan, hold);
  return plan;
}

type CombatTargetJson = {
  leader?: PlayerId;
  slot?: number;
  player?: PlayerId;
  id?: number;
};

function planAttackArrow(action: NeutralAction, before: FullState): CueArrow | null {
  if (!("attack" in action)) return null;
  const a = action.attack;
  const attacker = before.players[a.player].field[a.attacker_slot];
  if (!attacker) return null;
  const from: CueEndpoint = { kind: "uid", uid: attacker.id, player: a.player };
  const enemy = a.player === "a" ? "b" : "a";
  let to: CueEndpoint;
  if (a.target === "leader") {
    to = { kind: "leader", player: enemy };
  } else {
    const def = before.players[enemy].field[a.target.slot];
    if (!def) return null;
    to = { kind: "uid", uid: def.id, player: enemy };
  }
  return { type: "arrow", kind: "attack", from, to };
}

function planChoiceArrow(
  action: NeutralAction,
  before: FullState,
  sessionEvents: EngineEvent[],
  stepEvents: EngineEvent[],
): CueArrow | null {
  if (!("choose" in action)) return null;
  const opt = action.choose.option;
  if (isNonTargetChoice(opt)) return null;
  const chooser = action.choose.player;
  const to = resolveChoiceTarget(opt, chooser, before, sessionEvents, stepEvents);
  if (!to) return null;
  const from = findChoiceSource(sessionEvents, stepEvents, chooser);
  if (!from) return null;
  return { type: "arrow", kind: "choice", from, to };
}

function isNonTargetChoice(opt: ChooseOption): boolean {
  if (typeof opt === "object" && opt !== null) {
    if ("card" in opt) return true;
    if ("mode" in opt) return true;
  }
  return false;
}

function findChoiceSource(
  sessionEvents: EngineEvent[],
  stepEvents: EngineEvent[],
  player: PlayerId,
): CueEndpoint | null {
  const prior = sessionEvents.slice(0, sessionEvents.length - stepEvents.length);
  for (let i = prior.length - 1; i >= 0; i--) {
    const ev = prior[i]!;
    if ("choice_offered" in ev) {
      const co = ev.choice_offered as { player: PlayerId; source?: Record<string, unknown> };
      if (co.player !== player) continue;
      if (!co.source) return null;
      const parsed = parseResolveSource(co.source);
      if (parsed) return sourceToEndpoint(parsed, null);
      return null;
    }
  }
  return null;
}

function resolveChoiceTarget(
  opt: ChooseOption,
  chooser: PlayerId,
  before: FullState,
  sessionEvents: EngineEvent[],
  stepEvents: EngineEvent[],
): CueEndpoint | null {
  if (opt === "leader") {
    const prior = sessionEvents.slice(0, sessionEvents.length - stepEvents.length);
    for (let i = prior.length - 1; i >= 0; i--) {
      const ev = prior[i]!;
      if (!("choice_offered" in ev)) continue;
      const co = ev.choice_offered as {
        player: PlayerId;
        node?: { options?: Array<Record<string, unknown>> };
      };
      if (co.player !== chooser) continue;
      const options = co.node?.options ?? [];
      for (const o of options) {
        if ("leader" in o || o.kind === "leader") {
          const player = (o.player ?? o.leader) as PlayerId;
          return { kind: "leader", player };
        }
      }
      return { kind: "leader", player: chooser === "a" ? "b" : "a" };
    }
    return { kind: "leader", player: chooser === "a" ? "b" : "a" };
  }
  if (typeof opt === "object" && opt !== null && "slot" in opt) {
    const enemy = chooser === "a" ? "b" : "a";
    if (opt.player) {
      const unit = before.players[opt.player].field[opt.slot];
      if (!unit) return null;
      return { kind: "uid", uid: unit.id, player: opt.player };
    }
    const tryOrder = [enemy, chooser] as PlayerId[];
    for (const p of tryOrder) {
      const unit = before.players[p].field[opt.slot];
      if (unit) return { kind: "uid", uid: unit.id, player: p };
    }
    return null;
  }
  return null;
}

function effectiveSource(
  events: EngineEvent[],
  index: number,
  lastResolve: ParsedSource | null,
  before: FullState,
  config: SessionConfig,
  matchedLwDestroys: Set<string>,
): CueEndpoint | null {
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
    const destroy = findMatchingDestroyBefore(
      events,
      index,
      src.player,
      src.card,
      matchedLwDestroys,
    );
    if (destroy) {
      return { kind: "uid", uid: destroy.id, player: destroy.player };
    }
  }

  return sourceToEndpoint(src, before, config);
}

function findMatchingDestroyBefore(
  events: EngineEvent[],
  beforeIndex: number,
  player: PlayerId,
  card: string,
  matched: Set<string>,
): { player: PlayerId; id: number } | null {
  for (let j = beforeIndex - 1; j >= 0; j--) {
    const ev = events[j]!;
    if ("destroy" in ev) {
      const d = ev.destroy as { player: PlayerId; card: string; id: number };
      const key = `${d.player}:${d.id}`;
      if (d.player === player && d.card === card && !matched.has(key)) {
        matched.add(key);
        return { player: d.player, id: d.id };
      }
    }
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

function sourceToEndpoint(
  src: ParsedSource,
  _before: FullState | null,
  config?: SessionConfig,
): CueEndpoint | null {
  switch (src.kind) {
    case "field":
    case "combat":
      return { kind: "uid", uid: src.id, player: src.player };
    case "spell":
      if (config && isHiddenSide(config, src.player)) {
        return { kind: "leader", player: src.player };
      }
      return { kind: "leader", player: src.player };
    case "leader":
      return { kind: "leader", player: src.player };
    case "hand":
      if (config && isHiddenSide(config, src.player)) {
        return { kind: "leader", player: src.player };
      }
      return { kind: "hand", uid: src.id, player: src.player };
    case "crest":
      return { kind: "leader", player: src.player };
  }
}

function combatTargetToEndpoint(target: CombatTargetJson): CueEndpoint | null {
  if (target.leader) return { kind: "leader", player: target.leader };
  if (!target.player) return null;
  if (typeof target.id === "number") {
    return { kind: "uid", uid: target.id, player: target.player };
  }
  return null;
}

function eventTargetToEndpoint(
  raw: Record<string, unknown>,
  before: FullState,
): CueEndpoint | null {
  if ("leader" in raw) return { kind: "leader", player: raw.leader as PlayerId };
  if ("slot" in raw && "player" in raw) {
    const player = raw.player as PlayerId;
    const id = raw.id as number | undefined;
    if (typeof id === "number") return { kind: "uid", uid: id, player };
    const slot = raw.slot as number;
    const unit = before.players[player].field[slot];
    if (!unit) return null;
    return { kind: "uid", uid: unit.id, player };
  }
  if ("hand" in raw && raw.hand && typeof raw.hand === "object") {
    return null;
  }
  if ("deck" in raw && raw.deck && typeof raw.deck === "object") {
    return null;
  }
  if ("card" in raw) return null;
  return null;
}

function isHiddenRandomTarget(raw: Record<string, unknown>): boolean {
  if ("hand" in raw) return true;
  if ("deck" in raw) return true;
  if ("card" in raw) return true;
  return false;
}

function sourcePlayer(ep: CueEndpoint): PlayerId | null {
  return ep.player;
}

function unitOnBoard(state: FullState, player: PlayerId, uid: number): boolean {
  return state.players[player].field.some((c) => c?.id === uid);
}

function endpointOnBoard(cue: Cue, before: FullState, after: FullState): boolean {
  if (cue.type === "play-spotlight") return true;
  if (cue.type === "fade" && cue.endpoint.kind === "uid") {
    return unitOnBoard(before, cue.endpoint.player, cue.endpoint.uid);
  }
  if (cue.type === "arrow") {
    return endpointValid(cue.from, before, after) && endpointValid(cue.to, before, after);
  }
  return true;
}

function endpointValid(ep: CueEndpoint, before: FullState, after: FullState): boolean {
  if (ep.kind === "leader") return true;
  if (ep.kind === "hand") return true;
  if (ep.kind === "uid") {
    return unitOnBoard(before, ep.player, ep.uid) || unitOnBoard(after, ep.player, ep.uid);
  }
  return false;
}

function nonHumanPlayers(config: SessionConfig): Set<PlayerId> {
  if (config.mode === "hotseat") return new Set();
  if (config.mode === "watch") return new Set(["a", "b"]);
  return new Set([config.humanSide === "a" ? "b" : "a"]);
}

function isHiddenSide(config: SessionConfig, player: PlayerId): boolean {
  return config.mode === "vs-bot" && config.hideBotHand && player !== config.humanSide;
}

function cueVisible(config: SessionConfig, cue: Cue): boolean {
  if (cue.type === "play-spotlight") return true;
  if (cue.type === "fade") {
    return !isHiddenSide(config, cue.endpoint.player);
  }
  if (cue.type === "arrow") {
    if (cue.kind === "random" && isHiddenHandDeckEndpoint(cue.to)) return false;
    return true;
  }
  return true;
}

function isHiddenHandDeckEndpoint(ep: CueEndpoint): boolean {
  return ep.kind === "hand";
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

export function collectPlanEndpointKeys(plan: CuePlan): Set<string> {
  const keys = new Set<string>();
  for (const { cue } of plan.cues) {
    if (cue.type === "fade") keys.add(endpointKey(cue.endpoint));
    if (cue.type === "arrow") {
      keys.add(endpointKey(cue.from));
      keys.add(endpointKey(cue.to));
    }
  }
  return keys;
}

function elementForEndpoint(ep: CueEndpoint): HTMLElement | null {
  if (ep.kind === "leader") {
    return document.getElementById(`${visual(ep.player)}Leader`);
  }
  if (ep.kind === "uid") {
    const board = document.getElementById(`${visual(ep.player)}Board`);
    return board?.querySelector<HTMLElement>(`.card[data-uid="${ep.uid}"]`) ?? null;
  }
  if (ep.kind === "hand") {
    const hand = document.getElementById(`${visual(ep.player)}Hand`);
    return hand?.querySelector<HTMLElement>(`.card[data-uid="${ep.uid}"]`) ?? null;
  }
  return null;
}

export function captureCueRectsBeforePaint(
  plan: CuePlan,
  before: FullState,
): Map<string, DOMRect> {
  const rects = new Map<string, DOMRect>();
  for (const key of collectPlanEndpointKeys(plan)) {
    const ep = keyToEndpoint(key, plan, before);
    if (!ep) continue;
    if (ep.kind === "uid" && !unitOnBoard(before, ep.player, ep.uid)) continue;
    const el = elementForEndpoint(ep);
    if (el) rects.set(key, el.getBoundingClientRect());
  }
  return rects;
}

export function captureCueRects(
  plan: CuePlan,
  before: FullState,
  after: FullState,
  prePaint?: Map<string, DOMRect>,
): Map<string, DOMRect> {
  const rects = new Map(prePaint ?? []);
  for (const key of collectPlanEndpointKeys(plan)) {
    const ep = keyToEndpoint(key, plan, before);
    if (!ep) continue;
    const afterEl = elementForEndpoint(ep);
    const afterOnBoard =
      ep.kind === "uid" ? unitOnBoard(after, ep.player, ep.uid) : ep.kind === "leader";
    if (afterEl && (ep.kind === "leader" || ep.kind === "hand" || afterOnBoard)) {
      rects.set(key, afterEl.getBoundingClientRect());
    }
  }
  return rects;
}

function keyToEndpoint(
  key: string,
  plan: CuePlan,
  before: FullState,
): CueEndpoint | null {
  for (const { cue } of plan.cues) {
    if (cue.type === "arrow") {
      if (endpointKey(cue.from) === key) return cue.from;
      if (endpointKey(cue.to) === key) return cue.to;
    }
    if (cue.type === "fade" && endpointKey(cue.endpoint) === key) return cue.endpoint;
  }
  if (key.startsWith("leader:")) {
    return { kind: "leader", player: key.slice(7) as PlayerId };
  }
  if (key.startsWith("uid:")) {
    const uid = Number(key.slice(4));
    for (const p of ["a", "b"] as PlayerId[]) {
      if (unitOnBoard(before, p, uid)) return { kind: "uid", uid, player: p };
    }
  }
  return null;
}

function centerOf(rect: DOMRect): { x: number; y: number } {
  return { x: rect.left + rect.width / 2, y: rect.top + rect.height / 2 };
}

export function clearCueLog(): void {
  cueLogEntries = [];
  lastCuePlan = null;
}

export function clearCues(): void {
  for (const t of cueTimers) window.clearTimeout(t);
  cueTimers.length = 0;
  if (cueClearTimer !== null) {
    window.clearTimeout(cueClearTimer);
    cueClearTimer = null;
  }
  const layer = byId("cueLayer");
  if (layer) layer.replaceChildren();
}

export function playCuePlan(
  plan: CuePlan,
  pace: PlayCuesPace,
  rects: Map<string, DOMRect>,
): void {
  clearCues();
  if (pace === "off" || !plan.cues.length) return;
  const layer = byId("cueLayer");
  if (!layer) return;

  let svg = layer.querySelector<SVGSVGElement>(".cue-arrows");
  if (!svg) {
    svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
    svg.classList.add("cue-arrows");
    svg.setAttribute("aria-hidden", "true");
    const defs = document.createElementNS("http://www.w3.org/2000/svg", "defs");
    const marker = document.createElementNS("http://www.w3.org/2000/svg", "marker");
    marker.setAttribute("id", "cueArrowHead");
    marker.setAttribute("markerWidth", "8");
    marker.setAttribute("markerHeight", "8");
    marker.setAttribute("refX", "6");
    marker.setAttribute("refY", "4");
    marker.setAttribute("orient", "auto");
    const head = document.createElementNS("http://www.w3.org/2000/svg", "path");
    head.setAttribute("d", "M0,0 L8,4 L0,8 Z");
    head.setAttribute("fill", "context-stroke");
    marker.appendChild(head);
    defs.appendChild(marker);
    svg.appendChild(defs);
    layer.appendChild(svg);
  }
  const syncSvg = () => {
    svg!.setAttribute("width", String(window.innerWidth));
    svg!.setAttribute("height", String(window.innerHeight));
  };
  syncSvg();

  const hold = plan.durationMs;
  for (const { at, cue } of plan.cues) {
    const show = window.setTimeout(() => {
      syncSvg();
      if (cue.type === "play-spotlight") renderPlaySpotlight(layer, cue);
      if (cue.type === "arrow") renderArrow(svg!, cue, rects);
      if (cue.type === "fade") renderFade(layer, cue, rects);
    }, at);
    cueTimers.push(show);
  }

  cueClearTimer = window.setTimeout(() => clearCues(), hold);
  cueTimers.push(cueClearTimer);
}

function renderPlaySpotlight(layer: HTMLElement, cue: CuePlaySpotlight): void {
  const board = document.getElementById(`${visual(cue.player)}Board`);
  if (!board) return;
  const boardRect = board.getBoundingClientRect();
  const text = lookupText(cue.card);
  const el = document.createElement("div");
  el.className = "cue-play-spotlight";
  const side = cue.player === "a" ? "left" : "right";
  el.dataset.side = side;
  el.dataset.card = cue.card;

  const imgWrap = document.createElement("div");
  imgWrap.className = "cue-play-spotlight__art";
  const img = document.createElement("img");
  img.className = "cue-play-spotlight__img";
  applyCardImage(img, cue.card, false, imgWrap);
  imgWrap.appendChild(img);

  const name = document.createElement("div");
  name.className = "cue-play-spotlight__name";
  name.textContent = text.name;

  const chip = document.createElement("div");
  chip.className = "cue-play-spotlight__chip";
  chip.textContent = text.kind;

  el.append(imgWrap, name, chip);
  const width = 108;
  const height = 148;
  const gap = 12;
  const top = boardRect.top + boardRect.height / 2 - height / 2;
  const left =
    side === "left" ? boardRect.left - width - gap : boardRect.right + gap;
  el.style.left = `${left}px`;
  el.style.top = `${top}px`;
  el.style.width = `${width}px`;
  layer.appendChild(el);
}

function renderArrow(svg: SVGSVGElement, cue: CueArrow, rects: Map<string, DOMRect>): void {
  const fromRect = rects.get(endpointKey(cue.from));
  const toRect = rects.get(endpointKey(cue.to));
  if (!fromRect || !toRect) return;
  const a = centerOf(fromRect);
  const b = centerOf(toRect);
  const path = document.createElementNS("http://www.w3.org/2000/svg", "path");
  const owner = cue.from.player;
  path.setAttribute("class", `cue-arrow cue-arrow--${owner}`);
  path.dataset.why = cue.kind;
  path.dataset.style = cue.dashed ? "dashed" : "solid";
  path.dataset.from = endpointKey(cue.from);
  path.dataset.to = endpointKey(cue.to);
  path.dataset.x1 = String(Math.round(a.x));
  path.dataset.y1 = String(Math.round(a.y));
  path.dataset.x2 = String(Math.round(b.x));
  path.dataset.y2 = String(Math.round(b.y));
  const dx = b.x - a.x;
  const dy = b.y - a.y;
  const cx = a.x + dx * 0.35;
  const cy = a.y + dy * 0.15 - Math.abs(dx) * 0.08;
  path.setAttribute("d", `M ${a.x} ${a.y} Q ${cx} ${cy} ${b.x} ${b.y}`);
  if (cue.dashed) path.classList.add("cue-arrow--dashed");
  svg.appendChild(path);
}

function renderFade(layer: HTMLElement, cue: CueFade, rects: Map<string, DOMRect>): void {
  const rect = rects.get(endpointKey(cue.endpoint));
  if (!rect) return;
  const el = document.createElement("div");
  el.className = "cue-fade";
  el.dataset.card = cue.card;
  if (cue.endpoint.kind === "uid") el.dataset.uid = String(cue.endpoint.uid);
  el.style.left = `${rect.left}px`;
  el.style.top = `${rect.top}px`;
  el.style.width = `${rect.width}px`;
  el.style.height = `${rect.height}px`;
  layer.appendChild(el);
}

export function cuesActiveMs(): number {
  if (cueClearTimer === null) return 0;
  const last = cueLogEntries[cueLogEntries.length - 1];
  return last?.duration ?? 0;
}

export function waitForCues(): Promise<void> {
  return new Promise((resolve) => {
    const poll = () => {
      if (cueClearTimer === null) resolve();
      else window.setTimeout(poll, 40);
    };
    poll();
  });
}
