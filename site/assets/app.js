import { demoWeights, stats, tritsOf, glyph } from './ospop.js';

const WEIGHTS = demoWeights();
const threshold = document.getElementById('threshold');
const groupSize = document.getElementById('group-size');
const readout = document.getElementById('threshold-readout');
const readouts = document.getElementById('readouts');
const grid = document.getElementById('trit-grid');
const hist = document.getElementById('histogram');
const hero = document.getElementById('hero-trits');

function autoDelta(weights) {
  let sum = 0;
  for (const w of weights) sum += Math.abs(w);
  return (0.7 * sum) / weights.length;
}

function render() {
  const auto = threshold.value === '0.7';
  const delta = auto ? null : Number(threshold.value);
  readout.textContent = auto ? `auto (Δ=${autoDelta(WEIGHTS).toFixed(3)})` : Number(threshold.value).toFixed(2);
  const size = Number(groupSize.value);
  const s = stats(WEIGHTS, size, delta);

  readouts.innerHTML = [
    ['sparsity', `${(s.sparsity * 100).toFixed(1)}%`],
    ['trits', `${s.packedBytes} B`],
    ['float32', `${s.floatBytes} B`],
    ['smaller', `${(100 * (1 - s.packedBytes / s.floatBytes)).toFixed(1)}%`],
    ['rmse', s.rmse.toFixed(4)],
  ]
    .map(([k, v]) => `<div><b>${v}</b><span>${k}</span></div>`)
    .join('');

  const trits = tritsOf(WEIGHTS, size, delta);
  grid.innerHTML = trits
    .map((t) => `<span class="${t < 0 ? 'neg' : t > 0 ? 'pos' : 'zero'}">${glyph(t)}</span>`)
    .join('');

  const buckets = new Array(10).fill(0);
  const perGroup = trits.length / s.groups;
  for (let g = 0; g < s.groups; g++) {
    let zeros = 0;
    for (let i = g * perGroup; i < (g + 1) * perGroup; i++) if (trits[i] === 0) zeros++;
    buckets[Math.min(9, Math.floor((zeros / perGroup) * 10))]++;
  }
  const max = Math.max(1, ...buckets);
  hist.innerHTML = buckets
    .map(
      (n, i) =>
        `<div class="row"><span>${i * 10}\u2013${i * 10 + 10}%</span><span class="bar"><i style="width:${(n / max) * 100}%"></i></span><span>${n}</span></div>`,
    )
    .join('');
}

if (hero) {
  hero.textContent = tritsOf(WEIGHTS.slice(0, 96), 128, null)
    .map(glyph)
    .join(' ');
}

threshold.addEventListener('input', render);
groupSize.addEventListener('change', render);

fetch('assets/curriculum.json')
  .then((r) => (r.ok ? r.json() : Promise.reject(new Error('no curriculum'))))
  .then((data) => {
    const list = document.getElementById('lesson-list');
    list.innerHTML = data.lessons
      .map(
        (l) =>
          `<li><a href="academy/${l.id}.html"><span class="id">${l.id}<span class="track">${l.track}</span></span><span>${l.title}</span></a></li>`,
      )
      .join('');
  })
  .catch(() => {
    document.getElementById('lesson-list').innerHTML =
      '<li class="muted">Run scripts/build-site.sh to generate the academy from the CLI.</li>';
  });

render();