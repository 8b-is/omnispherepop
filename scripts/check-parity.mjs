// Prove the browser kernel agrees with the Rust kernel.
// Usage: node scripts/check-parity.mjs            (needs cargo on PATH)
// Tolerance: sparsity must match exactly; rmse within 1e-3 relative.
import { execFileSync } from 'node:child_process';
import { demoWeights, stats } from '../site/assets/ospop.js';

const cli = (...args) =>
  JSON.parse(execFileSync('cargo', ['run', '-q', '-p', 'ospop-cli', '--', '--json', ...args], { encoding: 'utf8' }));

const failures = [];
for (const groupSize of [32, 128, 256]) {
  const rust = cli('quantize', '--group-size', String(groupSize));
  const js = stats(demoWeights(), groupSize, null);
  const sparsityDelta = Math.abs(rust.sparsity - js.sparsity);
  const scaleDelta = Math.abs(rust.mse - js.rmse * js.rmse);
  const ok = sparsityDelta < 1e-6 && scaleDelta < 1e-3;
  console.log(
    `${ok ? 'ok  ' : 'FAIL'} group=${groupSize} sparsity rust=${rust.sparsity.toFixed(6)} js=${js.sparsity.toFixed(6)} mse rust=${rust.mse.toExponential(3)} js=${(js.rmse * js.rmse).toExponential(3)}`,
  );
  if (!ok) failures.push(groupSize);
}

// Fixed threshold path too.
const rustFixed = cli('quantize', '--group-size', '64', '--threshold', '0.5');
const jsFixed = stats(demoWeights(), 64, 0.5);
const fixedOk = Math.abs(rustFixed.sparsity - jsFixed.sparsity) < 1e-6;
console.log(`${fixedOk ? 'ok  ' : 'FAIL'} threshold=0.5 sparsity rust=${rustFixed.sparsity.toFixed(6)} js=${jsFixed.sparsity.toFixed(6)}`);
if (!fixedOk) failures.push('fixed-threshold');

if (failures.length) {
  console.error(`parity failed: ${failures.join(', ')}`);
  process.exit(1);
}
console.log('parity ok: browser kernel matches ospop-cli');