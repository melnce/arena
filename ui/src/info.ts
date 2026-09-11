import type { Session } from "./session.ts";
import type { BoardCardInfo, GateInfo, HandCardInfo, PlayerId } from "./types.ts";

export function sessionHandInfo(s: Session, player: PlayerId): HandCardInfo[] {
  return JSON.parse(s.game.handInfo(player)) as HandCardInfo[];
}

export function sessionBoardInfo(s: Session, player: PlayerId): BoardCardInfo[] {
  return JSON.parse(s.game.boardInfo(player)) as BoardCardInfo[];
}

export function usablePp(pp: number, active: boolean): number {
  return pp + (active ? 1 : 0);
}

const FORM_GATES = new Set(["enhance", "accelerate", "crystallize"]);

export function isFormGate(kind: string): boolean {
  return FORM_GATES.has(kind);
}

export function formatGateLine(gate: GateInfo): string {
  if (isFormGate(gate.kind)) return "";
  const raw = (gate.label || gate.kind).trim();
  const label = raw ? raw.charAt(0).toUpperCase() + raw.slice(1) : gate.kind;
  switch (gate.kind) {
    case "necromancy":
      return `Necromancy ${gate.have}/${gate.need}`;
    case "combo":
      return `Combo ${gate.have}/${gate.need}`;
    case "rally":
      return `Rally ${gate.have}/${gate.need}`;
    case "earth_rite":
      return `Earth Rite ${gate.have}/${gate.need}`;
    case "overflow":
      return gate.met ? "Overflow ✓" : "Overflow";
    case "spellboost":
      return `Spellboost ${gate.have}`;
    default:
      break;
  }
  if (gate.need === 0 && gate.have === 0) {
    return `${label} ${gate.met ? "✓" : "✗"}`;
  }
  return `${label} ${gate.have}/${gate.need}${gate.met ? " ✓" : ""}`;
}

/** Cost badge: engine-paid PP, else printed cost. */
export function badgeCost(info: HandCardInfo | undefined, fallback: number): number {
  if (info?.cost != null) return info.cost;
  return fallback;
}

export function conditionGateMet(gates: GateInfo[] | undefined): boolean {
  return !!gates?.some((g) => g.met && !isFormGate(g.kind) && g.kind !== "spellboost");
}
