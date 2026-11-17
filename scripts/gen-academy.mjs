// Render one Academy page per lesson from `ospop learn --json`.
// Usage: node scripts/gen-academy.mjs curriculum.json outDir
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { join } from 'node:path';

const [, , curriculumPath, outDir] = process.argv;
const { lessons } = JSON.parse(readFileSync(curriculumPath, 'utf8'));
mkdirSync(outDir, { recursive: true });

const tracks = [...new Set(lessons.map((l) => l.track))];
const esc = (s) => String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');

for (const l of lessons) {
  const prereq = l.prerequisites.length
    ? `<p class="muted">Prerequisites: ${l.prerequisites.map((p) => `<a href="${p}.html">${p}</a>`).join(', ')}</p>`
    : '<p class="muted">No prerequisites — start here.</p>';
  const html = `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width, initial-scale=1" />
<title>${esc(l.title)} — Omnispherepop Academy</title>
<link rel="stylesheet" href="../assets/site.css" />
</head>
<body>
<header class="masthead">
  <a class="mark" href="../index.html">omnispherepop</a>
  <nav><a href="../index.html#sandbox">Sandbox</a><a href="../index.html#academy">Academy</a></nav>
</header>
<main>
<section class="hero">
  <p class="kicker">${esc(l.track)} · ${esc(l.id)}</p>
  <h1>${esc(l.title)}</h1>
  ${prereq}
</section>
<section class="panel">
  <h2>Concept</h2>
  <p>${esc(l.concept ?? 'Run the lab in your terminal: <code>ospop learn ' + l.id + '</code>.')}</p>
  <h2>ELI5</h2>
  <p id="eli5">${esc(l.eli5 ?? 'Run the lesson to see the plain-language version.')}</p>
  <h2>Lab</h2>
  <p class="muted">Lab target <code>${esc(l.lab_target ?? 'layer.weight')}</code> through the
  <code>${esc(l.lab_view)}</code> lens. The live lens below uses the same kernel as the CLI.</p>
  <div id="lens"></div>
  <pre class="install">ospop learn ${esc(l.id)}</pre>
  <p id="checkpoint" class="muted">checkpoint: pending</p>
</section>
</main>
<script type="module">
import { demoWeights, stats, tritsOf, glyph } from '../assets/ospop.js';
const lens = document.getElementById('lens');
const s = stats(demoWeights(), 128, null);
const trits = tritsOf(demoWeights(), 128, null);
lens.innerHTML =
  '<div class="readouts">' +
  [['sparsity', (s.sparsity * 100).toFixed(1) + '%'], ['trits', s.packedBytes + ' B'], ['rmse', s.rmse.toFixed(4)]]
    .map(([k, v]) => '<div><b>' + v + '</b><span>' + k + '</span></div>')
    .join('') +
  '</div><div class="trit-grid">' +
  trits.map((t) => '<span class="' + (t < 0 ? 'neg' : t > 0 ? 'pos' : 'zero') + '">' + glyph(t) + '</span>').join('') +
  '</div>';
document.getElementById('checkpoint').textContent =
  'checkpoint: ' + (s.sparsity >= 0.4 ? 'satisfied in-browser' : 'run in the CLI for the authoritative verdict');
</script>
</body>
</html>
`;
  writeFileSync(join(outDir, `${l.id}.html`), html);
}

const index = `<!doctype html>
<html lang="en"><head><meta charset="utf-8" /><meta name="viewport" content="width=device-width, initial-scale=1" />
<title>Academy — Omnispherepop</title><link rel="stylesheet" href="../assets/site.css" /></head>
<body>
<header class="masthead"><a class="mark" href="../index.html">omnispherepop</a>
<nav><a href="../index.html#sandbox">Sandbox</a><a href="../index.html#academy">Academy</a></nav></header>
<main>
${tracks.map((t) => `<section class="panel"><h2>${esc(t)}</h2><ul class="lessons">${lessons
  .filter((l) => l.track === t)
  .map((l) => `<li><a href="${l.id}.html"><span class="id">${esc(l.id)}</span><span>${esc(l.title)}</span></a></li>`)
  .join('')}</ul></section>`).join('\n')}
</main></body></html>
`;
writeFileSync(join(outDir, 'index.html'), index);
console.log(`wrote ${lessons.length} lessons across ${tracks.length} tracks`);