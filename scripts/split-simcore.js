// One-shot script: extract sim-core ranges into a dedicated <script id="sim-core">
// block before the existing <script>, and delete those ranges from the main script.
const fs = require('fs');
const path = require('path');
const src = fs.readFileSync(path.join(__dirname, '..', 'index.html'), 'utf8');
const lines = src.split('\n'); // 0-indexed, line N in editor = lines[N-1]

// 1-indexed, inclusive ranges to lift into the sim-core block, in source order.
const simCoreRanges = [
  [251, 255],   // arena constants
  [258, 269],   // MOB_DEFS + PLAYER_ATK_SPEED/RANGE/DAMAGE
  [271, 315],   // LOADOUTS section header + LOADOUTS
  [390, 527],   // UTILITY FUNCTIONS + PLAYER PATHING
  [530, 530],   // DEATH_ANIM_TICKS
  [533, 546],   // createRegion
  [547, 560],   // spawnNibblers
  [577, 597],   // parseSpawnCode
  [925, 925],   // findRespawnLocation
  [927, 1406],  // mulberry32 + Phase 2 headless engine + checkTrappedValid + checkTileExcluded + optimizePrayer + calcSimDamage
];

// Build sim-core text
const simCoreParts = simCoreRanges.map(([a, b]) => lines.slice(a - 1, b).join('\n'));
const simCoreText = simCoreParts.join('\n\n');

// Compute the set of lines to drop from the main script
const drop = new Set();
for (const [a, b] of simCoreRanges) for (let i = a; i <= b; i++) drop.add(i);

// Find the <script> opening line (the existing main script tag we want to convert)
let scriptOpenLine = -1;
for (let i = 0; i < lines.length; i++) {
  if (lines[i].trim() === '<script>') { scriptOpenLine = i + 1; break; }
}
if (scriptOpenLine === -1) { console.error('Could not find <script> tag'); process.exit(1); }

// Find the closing </script>
let scriptCloseLine = -1;
for (let i = lines.length - 1; i >= 0; i--) {
  if (lines[i].trim() === '</script>') { scriptCloseLine = i + 1; break; }
}
if (scriptCloseLine === -1) { console.error('Could not find </script>'); process.exit(1); }

// Rebuild output:
//   prefix (everything up to and including the opening <script> line)
//   becomes:
//     prefix-up-to-but-not-including-script-tag
//     <script id="sim-core">
//     <sim-core text>
//     </script>
//     <script>
//   then main script body with drop lines removed
//   then </script> and the rest

const preScript = lines.slice(0, scriptOpenLine - 1).join('\n'); // up to line before <script>
const mainBodyLines = [];
for (let lineNo = scriptOpenLine + 1; lineNo <= scriptCloseLine - 1; lineNo++) {
  if (drop.has(lineNo)) continue;
  mainBodyLines.push(lines[lineNo - 1]);
}
const postScript = lines.slice(scriptCloseLine).join('\n'); // after </script>

const out = [
  preScript,
  '<script id="sim-core">',
  '// =====================================================',
  '// SIM CORE — pure engine (shared between main thread and workers)',
  '// =====================================================',
  simCoreText,
  '</script>',
  '<script>',
  mainBodyLines.join('\n'),
  '</script>',
  postScript,
].join('\n');

fs.writeFileSync(path.join(__dirname, '..', 'index.html'), out);
console.log(`Wrote index.html: sim-core block has ${simCoreText.split('\n').length} lines, main body has ${mainBodyLines.length} lines`);
