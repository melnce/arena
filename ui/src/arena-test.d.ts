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
      };
      full(): unknown;
      legal(): unknown[];
      paintMs?: number;
    };
  }
}
