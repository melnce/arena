import type { NeutralAction, PlayerId } from "./types.ts";

export function keyOf(a: NeutralAction): string {
  return JSON.stringify(a);
}

export function playsFor(legal: NeutralAction[], player: PlayerId): NeutralAction[] {
  return legal.filter(
    (a) => "play" in a && a.play.player === player,
  );
}

export function playAt(
  legal: NeutralAction[],
  player: PlayerId,
  handPos: number,
): NeutralAction | null {
  return (
    legal.find(
      (a) => "play" in a && a.play.player === player && a.play.hand_pos === handPos,
    ) ?? null
  );
}

export function attacksFrom(
  legal: NeutralAction[],
  player: PlayerId,
  slot: number,
): NeutralAction[] {
  return legal.filter(
    (a) =>
      "attack" in a && a.attack.player === player && a.attack.attacker_slot === slot,
  );
}

export function evolveFor(
  legal: NeutralAction[],
  player: PlayerId,
  superEvo: boolean,
): NeutralAction[] {
  return legal.filter(
    (a) =>
      "evolve" in a && a.evolve.player === player && a.evolve.super === superEvo,
  );
}

export function engageAt(
  legal: NeutralAction[],
  player: PlayerId,
  slot: number,
): NeutralAction | null {
  return (
    legal.find(
      (a) => "engage" in a && a.engage.player === player && a.engage.slot === slot,
    ) ?? null
  );
}

export function fuseAt(
  legal: NeutralAction[],
  player: PlayerId,
  host: number,
): NeutralAction | null {
  return (
    legal.find(
      (a) => "fuse" in a && a.fuse.player === player && a.fuse.host_pos === host,
    ) ?? null
  );
}

export function bonusPp(legal: NeutralAction[], player: PlayerId): NeutralAction | null {
  return legal.find((a) => "bonus_pp" in a && a.bonus_pp.player === player) ?? null;
}

export function endTurn(legal: NeutralAction[], player: PlayerId): NeutralAction | null {
  return legal.find((a) => "end_turn" in a && a.end_turn.player === player) ?? null;
}

export function confirm(legal: NeutralAction[]): NeutralAction | null {
  return legal.find((a) => "confirm" in a) ?? null;
}

export function mulligan(
  legal: NeutralAction[],
  player: PlayerId,
  swap: [boolean, boolean, boolean, boolean],
): NeutralAction | null {
  return (
    legal.find(
      (a) =>
        "mulligan" in a &&
        a.mulligan.player === player &&
        a.mulligan.swap[0] === swap[0] &&
        a.mulligan.swap[1] === swap[1] &&
        a.mulligan.swap[2] === swap[2] &&
        a.mulligan.swap[3] === swap[3],
    ) ?? null
  );
}

export function chooseMatching(
  legal: NeutralAction[],
  pred: (opt: unknown) => boolean,
): NeutralAction | null {
  return legal.find((a) => "choose" in a && pred(a.choose.option)) ?? null;
}

export function isPlay(a: NeutralAction): a is { play: { player: PlayerId; hand_pos: number; card: string } } {
  return "play" in a;
}

export function isAttack(a: NeutralAction): boolean {
  return "attack" in a;
}

export function isChoose(a: NeutralAction): boolean {
  return "choose" in a;
}
