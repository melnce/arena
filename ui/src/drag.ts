/** Pointer-drag session (mouse + touch). No HTML5 drag-and-drop. */

export type DropAccept = (payload: string) => boolean;
export type DropHandler = (payload: string, clientX: number, clientY: number) => void;

export interface DragSourceOptions {
  payload: string;
  kind: "hand" | "attacker" | "evo";
  onDragBegan?: () => void;
  onDragEnded?: () => void;
  onThresholdCrossed?: () => void;
}

export const DRAG_THRESHOLD_PX = 8;

type Registered = {
  element: HTMLElement;
  accepts: DropAccept;
  onDrop: DropHandler;
  highlightClass: string;
};

const dropTargets = new Set<Registered>();

let suppressClickUntilMs = 0;

let session: {
  pointerId: number;
  source: HTMLElement;
  options: DragSourceOptions;
  startX: number;
  startY: number;
  dragging: boolean;
  preview: HTMLElement | null;
  suppressClick: boolean;
} | null = null;

export function setDropTarget(
  element: HTMLElement,
  accepts: DropAccept,
  onDrop: DropHandler,
  highlightClass = "pointer-drop-highlight",
): void {
  for (const t of [...dropTargets]) {
    if (t.element === element) dropTargets.delete(t);
  }
  dropTargets.add({ element, accepts, onDrop, highlightClass });
  element.dataset.pointerDrop = "1";
}

export function clearDropTarget(element: HTMLElement): void {
  for (const t of [...dropTargets]) {
    if (t.element === element) {
      t.element.classList.remove(t.highlightClass);
      dropTargets.delete(t);
    }
  }
  delete element.dataset.pointerDrop;
}

export function shouldSuppressClickFromPointerDrag(el?: HTMLElement): boolean {
  if (el?.dataset.pointerDragSuppressClick === "1") return true;
  return performance.now() < suppressClickUntilMs;
}

export function forceCancelPointerDrag(): void {
  if (!session) return;
  if (session.dragging) session.suppressClick = true;
  endSession(true);
}

function prune() {
  for (const t of [...dropTargets]) {
    if (!t.element.isConnected) dropTargets.delete(t);
  }
}

function clearHighlights(payload: string | null) {
  prune();
  for (const t of dropTargets) {
    if (payload && t.accepts(payload) && t.element.isConnected) {
      t.element.classList.add(t.highlightClass);
    } else {
      t.element.classList.remove(t.highlightClass);
    }
  }
}

function makePreview(source: HTMLElement): HTMLElement {
  const preview = source.cloneNode(true) as HTMLElement;
  preview.classList.add("pointer-drag-preview");
  preview.classList.remove("pointer-drag-source", "pressed");
  preview.removeAttribute("id");
  preview.style.position = "fixed";
  preview.style.pointerEvents = "none";
  preview.style.zIndex = "10000";
  preview.style.margin = "0";
  preview.style.opacity = "0.92";
  const rect = source.getBoundingClientRect();
  preview.style.width = `${rect.width}px`;
  preview.style.height = `${rect.height}px`;
  preview.style.left = `${rect.left}px`;
  preview.style.top = `${rect.top}px`;
  document.body.appendChild(preview);
  return preview;
}

function movePreview(preview: HTMLElement, clientX: number, clientY: number) {
  preview.style.left = `${clientX - preview.offsetWidth / 2}px`;
  preview.style.top = `${clientY - preview.offsetHeight / 2}px`;
}

function findDropAt(
  clientX: number,
  clientY: number,
  payload: string,
  source: HTMLElement,
  preview: HTMLElement | null,
): Registered | null {
  prune();
  const prevVis = preview?.style.visibility;
  const prevPe = source.style.pointerEvents;
  if (preview) preview.style.visibility = "hidden";
  source.style.pointerEvents = "none";
  const hit = document.elementFromPoint(clientX, clientY) as HTMLElement | null;
  source.style.pointerEvents = prevPe;
  if (preview) preview.style.visibility = prevVis ?? "";
  if (!hit) return null;
  let node: HTMLElement | null = hit;
  while (node) {
    for (const t of dropTargets) {
      if (t.element === node && t.accepts(payload)) return t;
    }
    node = node.parentElement;
  }
  return null;
}

function endSession(cancelled: boolean) {
  if (!session) return;
  const s = session;
  detachWindow();
  s.source.removeEventListener("lostpointercapture", onLost);
  clearHighlights(null);
  s.preview?.remove();
  if (s.source.isConnected) {
    s.source.classList.remove("pointer-drag-source", "pressed");
    delete s.source.dataset.pointerDragId;
  }
  try {
    s.source.releasePointerCapture(s.pointerId);
  } catch {
    /* already released */
  }
  if (s.dragging) s.options.onDragEnded?.();
  if (s.suppressClick && s.source.isConnected) {
    suppressClickUntilMs = performance.now() + 100;
    s.source.dataset.pointerDragSuppressClick = "1";
    setTimeout(() => {
      if (s.source.isConnected) delete s.source.dataset.pointerDragSuppressClick;
    }, 0);
  }
  session = null;
  void cancelled;
}

let windowOn = false;

function attachWindow() {
  if (windowOn) return;
  window.addEventListener("pointermove", onMove, true);
  window.addEventListener("pointerup", onUp, true);
  window.addEventListener("pointercancel", onCancel, true);
  windowOn = true;
}

function detachWindow() {
  if (!windowOn) return;
  window.removeEventListener("pointermove", onMove, true);
  window.removeEventListener("pointerup", onUp, true);
  window.removeEventListener("pointercancel", onCancel, true);
  windowOn = false;
}

function onLost(ev: PointerEvent) {
  if (!session || ev.pointerId !== session.pointerId) return;
  if (!session.source.isConnected) endSession(true);
}

function onMove(ev: PointerEvent) {
  if (!session || ev.pointerId !== session.pointerId) return;
  const dx = ev.clientX - session.startX;
  const dy = ev.clientY - session.startY;
  if (!session.dragging) {
    if (dx * dx + dy * dy < DRAG_THRESHOLD_PX * DRAG_THRESHOLD_PX) return;
    session.dragging = true;
    session.suppressClick = true;
    session.source.classList.add("pointer-drag-source");
    session.preview = makePreview(session.source);
    session.options.onThresholdCrossed?.();
    session.options.onDragBegan?.();
    clearHighlights(session.options.payload);
  }
  if (session.preview) movePreview(session.preview, ev.clientX, ev.clientY);
  clearHighlights(session.options.payload);
  ev.preventDefault();
}

function onUp(ev: PointerEvent) {
  if (!session || ev.pointerId !== session.pointerId) return;
  if (!session.source.isConnected) {
    endSession(true);
    return;
  }
  const s = session;
  if (!s.dragging) {
    endSession(true);
    return;
  }
  const target = findDropAt(ev.clientX, ev.clientY, s.options.payload, s.source, s.preview);
  if (target) {
    target.onDrop(s.options.payload, ev.clientX, ev.clientY);
    endSession(false);
    return;
  }
  endSession(false);
}

function onCancel(ev: PointerEvent) {
  if (!session || ev.pointerId !== session.pointerId) return;
  if (session.dragging) session.suppressClick = true;
  endSession(true);
}

export function attachPointerDragSource(
  el: HTMLElement,
  options: DragSourceOptions,
  enabled: boolean,
): void {
  const prev = (el as HTMLElement & { __pointerDragCleanup?: () => void }).__pointerDragCleanup;
  if (prev) prev();
  if (!enabled) {
    el.dataset.pointerDraggable = "false";
    el.removeAttribute("draggable");
    return;
  }
  el.dataset.pointerDraggable = "true";
  el.removeAttribute("draggable");
  const onDown = (ev: PointerEvent) => {
    if (ev.button !== 0) return;
    if (session) {
      if (!session.source.isConnected || session.pointerId === ev.pointerId) endSession(true);
      else return;
    }
    delete el.dataset.pointerDragSuppressClick;
    suppressClickUntilMs = 0;
    el.classList.add("pressed");
    session = {
      pointerId: ev.pointerId,
      source: el,
      options,
      startX: ev.clientX,
      startY: ev.clientY,
      dragging: false,
      preview: null,
      suppressClick: false,
    };
    el.dataset.pointerDragId = String(ev.pointerId);
    try {
      el.setPointerCapture(ev.pointerId);
    } catch {
      /* optional */
    }
    el.addEventListener("lostpointercapture", onLost);
    attachWindow();
  };
  el.addEventListener("pointerdown", onDown);
  (el as HTMLElement & { __pointerDragCleanup?: () => void }).__pointerDragCleanup = () => {
    el.removeEventListener("pointerdown", onDown);
    el.removeEventListener("lostpointercapture", onLost);
  };
}

document.addEventListener("visibilitychange", () => {
  if (document.visibilityState === "hidden") forceCancelPointerDrag();
});
window.addEventListener("blur", () => forceCancelPointerDrag());
