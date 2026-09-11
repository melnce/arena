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
      full(): unknown;
      legal(): unknown[];
      paintMs?: number;
    };
  }
}
