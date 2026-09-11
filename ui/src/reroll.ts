/**
 * Deterministic F8 / Reroll seed.
 *
 * `nextSeed = splitmix64(gameSeed XOR (n * GOLDEN))` where `n` is the
 * 1-based reroll count since the last F6 checkpoint and GOLDEN is
 * `0x9E3779B97F4A7C15`. Same game seed + same `n` always reseeds the
 * same way; the checkpoint board is unchanged (RNG sits outside hash).
 */
export const REROLL_GOLDEN = 0x9e3779b97f4a7c15n;

export function rerollSeed(gameSeed: bigint, n: number): bigint {
  const count = BigInt(Math.max(1, n | 0));
  return splitmix64(gameSeed ^ (count * REROLL_GOLDEN));
}

function splitmix64(seed: bigint): bigint {
  const mask = 0xffffffffffffffffn;
  let z = (seed + REROLL_GOLDEN) & mask;
  z = ((z ^ (z >> 30n)) * 0xbf58476d1ce4e5b9n) & mask;
  z = ((z ^ (z >> 27n)) * 0x94d049bb133111ebn) & mask;
  return (z ^ (z >> 31n)) & mask;
}
