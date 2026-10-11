declare module "../pkg/arena_wasm.js" {
  export default function init(module?: unknown): Promise<unknown>;
  export class Game {
    constructor(
      seed: number | bigint | string,
      deckA: string,
      deckB: string,
      first: string,
    );
    legal(): string;
    apply(action: string): string;
    snapshot(): string;
    full(): string;
    hash(): string;
    phase(): string;
    clone(): Game;
    free(): void;
    acting(): string;
    active(): string;
    turn(): number;
    winner(): string | null;
    botAction(policy: string, seed: number | bigint | string): string;
    botActionValue(policy: string, seed: number | bigint | string): string;
    h0JobStart(policy: string, seed: number | bigint | string): string;
    h0KillDet(policy: string, seed: number | bigint | string): string;
    h0KillRoll(policy: string, seed: number | bigint | string): string;
    h0JobKillDet(outcomeJson: string): string;
    h0JobKillRoll(outcomeJson: string): string;
    h0JobWorld(
      policy: string,
      seed: number | bigint | string,
      r: number,
      budget: number,
    ): string;
    h0JobFinish(outcomesJson: string): string;
    handInfo(player: string): string;
    boardInfo(player: string): string;
    playerInfo(player: string): string;
    modeChoiceInfo(): string;
    reseed(seed: number | bigint | string): void;
    debugGrantCantAttackLeader(player: string, slot: number): void;
  }
  export function cardText(id: string): string;
  export function bundleInfo(): string;
  export function version(): string;
  export function botPolicies(): string;
}
