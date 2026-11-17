// Omnispherepop browser reference kernel.
// Mirrors crates/ospop-core (ternary PTQ, sparsity, reconstruction error).
// This is a deliberately small reference implementation, not the Rust engine;
// parity is checked against `ospop quantize --json` by scripts/check-parity.mjs.

export const ZERO = 0;
export const POS = 1;
export const NEG = -1;

/** Deterministic PRNG so the browser and the CLI can share a snapshot. */
export function demoWeights(count = 512, seed = 0x9e3779b9) {
  let state = seed >>> 0;
  const out = new Array(count);
  for (let i = 0; i < count; i++) {
    state = (Math.imul(state, 1664525) + 1013904223) >>> 0;
    const unit = (state >>> 8) / 0x1000000;
    const centred = unit * 4 - 2;
    out[i] = centred * Math.abs(centred) * (1 - (i % 7) * 0.1);
  }
  return out;
}

/** Δ = threshold ?? 0.7 * mean|w|; scale = mean|w| over selected trits. */
export function quantize(weights, groupSize = 128, threshold = null) {
  const groups = [];
  for (let start = 0; start < weights.length; start += groupSize) {
    const slice = weights.slice(start, start + groupSize);
    const mean = slice.reduce((a, w) => a + Math.abs(w), 0) / slice.length;
    const delta = threshold === null ? 0.7 * mean : threshold;
    const trits = slice.map((w) => (w < -delta ? NEG : w > delta ? POS : ZERO));
    let picked = 0;
    let sum = 0;
    trits.forEach((t, i) => {
      if (t !== ZERO) {
        picked++;
        sum += Math.abs(slice[i]);
      }
    });
    groups.push({ trits, scale: picked === 0 ? 0 : sum / picked });
  }
  return groups;
}

export function stats(weights, groupSize = 128, threshold = null) {
  const groups = quantize(weights, groupSize, threshold);
  const total = groups.reduce((n, g) => n + g.trits.length, 0);
  let zeros = 0;
  let sqErr = 0;
  let index = 0;
  for (const g of groups) {
    for (const t of g.trits) {
      if (t === ZERO) zeros++;
      const value = t * g.scale;
      const d = value - weights[index++];
      sqErr += d * d;
    }
  }
  return {
    values: total,
    groups: groups.length,
    sparsity: total === 0 ? 0 : zeros / total,
    packedBytes: Math.ceil(total / 4),
    floatBytes: total * 4,
    rmse: total === 0 ? 0 : Math.sqrt(sqErr / total),
    scales: groups.map((g) => g.scale),
  };
}

export function tritsOf(weights, groupSize = 128, threshold = null) {
  return quantize(weights, groupSize, threshold).flatMap((g) => g.trits);
}

export function glyph(t) {
  return t === NEG ? '\u2212' : t === POS ? '+' : '\u00b7';
}