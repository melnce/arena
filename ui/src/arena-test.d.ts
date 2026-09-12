export {};

declare global {
  interface Window {
    __arena?: {
      hash(): string;
      canUndo(): boolean;
      canRedo(): boolean;
      botAction(policy: string, seed: string | number | bigint): string;
      apply(action: unknown): unknown;
      handInfo(player: string): unknown[];
      boardInfo(player: string): unknown[];
      playerInfo(player: string): {
        evolve_unlocked: boolean;
        super_evolve_unlocked: boolean;
        evolve_unlock_in: number;
        super_evolve_unlock_in: number;
        has_leader_barrier?: boolean;
      };
      full(): unknown;
      legal(): unknown[];
      actions(): unknown[];
      paintMs?: number;
      watchDelayMs(): number;
      humanSide(): "a" | "b" | null;
      reseed(seed: string | number | bigint): void;
      exportLog(): unknown;
      loadLog(log: unknown): void;
      setCheckpoint(): void;
      restoreCheckpoint(): boolean;
      reroll(): boolean;
      rerollSeed(seed: string | number | bigint, n: number): string;
      namedCounterValue(inst: { kind: string; countdown: number | null; vars: Record<string, number> }): number | null;
      rematchSame(): void;
      savedPosition(): {
        name?: string;
        turn?: number;
        savedAt?: string;
        actions: unknown[];
      } | null;
      mountNamedCounter(vars: Record<string, number>): string;
      debugGrantCantAttackLeader(player: string, slot: number): void;
    };
  }
}
