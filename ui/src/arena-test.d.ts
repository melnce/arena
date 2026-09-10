export {};

declare global {
  interface Window {
    __arena?: {
      hash(): string;
      canUndo(): boolean;
      canRedo(): boolean;
      botAction(policy: string, seed: string | number | bigint): string;
    };
  }
}
