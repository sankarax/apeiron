export function lerp(from: number, to: number, alpha: number): number {
  return from + (to - from) * alpha;
}

export function wrap(value: number, length: number): number {
  return ((value % length) + length) % length;
}

export function wrappedDelta(from: number, to: number, length: number): number {
  return wrap(to - from + length / 2, length) - length / 2;
}

export function lerpWrapped(
  from: number,
  to: number,
  alpha: number,
  length: number,
): number {
  const delta = wrappedDelta(from, to, length);

  return wrap(from + delta * alpha, length);
}

export function clamp(value: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, value));
}
