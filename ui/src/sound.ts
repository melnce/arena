let audioContext: AudioContext | null = null;
let turnPingCount = 0;
let unlockInstalled = false;

const NOTE1_HZ = 784;
const NOTE2_HZ = 1175;
const NOTE_S = 0.18;
const ATTACK_S = 0.008;
const GAP_S = 0.09;
const PEAK_GAIN = 0.2;

type UnlockListener = { type: string; fn: EventListener };

function removeUnlockListeners(listeners: UnlockListener[]): void {
  for (const { type, fn } of listeners) {
    window.removeEventListener(type, fn, true);
  }
}

function audioContextCtor(): typeof AudioContext | null {
  const w = window as typeof globalThis & { webkitAudioContext?: typeof AudioContext };
  return w.AudioContext ?? w.webkitAudioContext ?? null;
}

function iosSilentPrime(ctx: AudioContext): void {
  const buf = ctx.createBuffer(1, 1, ctx.sampleRate);
  const src = ctx.createBufferSource();
  src.buffer = buf;
  src.connect(ctx.destination);
  src.start();
}

export function installAudioUnlock(): void {
  if (unlockInstalled) return;
  unlockInstalled = true;
  const listeners: UnlockListener[] = [];

  const unlock = () => {
    try {
      if (!audioContext) {
        const Ctor = audioContextCtor();
        if (!Ctor) return;
        audioContext = new Ctor();
      }
      void audioContext.resume().then(() => {
        if (audioContext?.state === "running") {
          iosSilentPrime(audioContext);
        }
        if (audioContext?.state === "running") {
          removeUnlockListeners(listeners);
        }
      });
    } catch {
      /* gesture unlock may fail on some browsers */
    }
  };

  for (const type of ["pointerup", "touchend", "keydown", "click"]) {
    const fn = () => unlock();
    window.addEventListener(type, fn, true);
    listeners.push({ type, fn });
  }
}

export function turnPingOn(): boolean {
  const box = document.getElementById("turnPingToggle") as HTMLInputElement | null;
  if (box) return box.checked;
  try {
    const v = localStorage.getItem("svwb.turnPing");
    return v == null ? true : v !== "0";
  } catch {
    return true;
  }
}

function playNote(ctx: AudioContext, freq: number, startAt: number): void {
  const osc = ctx.createOscillator();
  const gain = ctx.createGain();
  osc.type = "sine";
  osc.frequency.setValueAtTime(freq, startAt);
  gain.gain.setValueAtTime(0, startAt);
  gain.gain.linearRampToValueAtTime(PEAK_GAIN, startAt + ATTACK_S);
  const decayEnd = startAt + NOTE_S;
  gain.gain.exponentialRampToValueAtTime(0.0001, decayEnd);
  osc.connect(gain);
  gain.connect(ctx.destination);
  osc.start(startAt);
  osc.stop(decayEnd);
}

function playChime(ctx: AudioContext): void {
  const t = ctx.currentTime;
  playNote(ctx, NOTE1_HZ, t);
  playNote(ctx, NOTE2_HZ, t + GAP_S);
}

export function playTurnPing(): void {
  turnPingCount += 1;
  try {
    if (!audioContext) {
      const Ctor = audioContextCtor();
      if (!Ctor) return;
      audioContext = new Ctor();
    }
    if (audioContext.state === "running") {
      playChime(audioContext);
      return;
    }
    void audioContext
      .resume()
      .then(() => {
        if (audioContext?.state === "running") {
          playChime(audioContext);
        }
      })
      .catch(() => {});
  } catch {
    /* autoplay policy or missing Web Audio */
  }
}

export function turnPings(): number {
  return turnPingCount;
}
