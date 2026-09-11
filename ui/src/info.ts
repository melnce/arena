import type { Session } from "./session.ts";
import type { BoardCardInfo, HandCardInfo, PlayerId } from "./types.ts";

export function sessionHandInfo(s: Session, player: PlayerId): HandCardInfo[] {
  return JSON.parse(s.game.handInfo(player)) as HandCardInfo[];
}

export function sessionBoardInfo(s: Session, player: PlayerId): BoardCardInfo[] {
  return JSON.parse(s.game.boardInfo(player)) as BoardCardInfo[];
}

export function usablePp(pp: number, active: boolean): number {
  return pp + (active ? 1 : 0);
}

export function formatGateLine(gate: {
  kind: string;
  need: number;
  have: number;
  met: boolean;
}): string {
  switch (gate.kind) {
    case "necromancy":
      return `Necromancy ${gate.have}/${gate.need}`;
    case "enhance":
      return `Enhance ${gate.need} (have ${gate.have})`;
    case "accelerate":
      return `Accelerate ${gate.need} (have ${gate.have})`;
    case "crystallize":
      return `Crystallize ${gate.need} (have ${gate.have})`;
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
      return `${gate.kind} ${gate.have}/${gate.need}`;
  }
}

export function formLetter(
  form: HandCardInfo["form"],
): "E" | "A" | "C" | null {
  if (form === "enhance") return "E";
  if (form === "accelerate") return "A";
  if (form === "crystallize") return "C";
  return null;
}

/** Cost badge: engine-paid PP, else the selected form's need, else printed cost. */
export function badgeCost(info: HandCardInfo | undefined, fallback: number): number {
  if (info?.cost != null) return info.cost;
  if (info?.form && info.form !== "normal") {
    const g = info.gates.find((x) => x.kind === info.form);
    if (g) return g.need;
  }
  return fallback;
}
