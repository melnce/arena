export {};

declare global {
  interface Window {
    __arena?: {
      hash(): string;
      canUndo(): boolean;
      canRedo(): boolean;
    };
  }
}
