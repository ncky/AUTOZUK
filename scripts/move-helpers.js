// Move 5 helper functions from main into sim-core: isUnderMob, canSetLastAttacker,
// setPlayerLastAttacker, startDig, playerBFS.
const fs = require('fs');
const src = fs.readFileSync('index.html', 'utf8');
const lines = src.split('\n');

// 1-indexed inclusive ranges to lift from main into sim-core (matching current file state)
const moveRanges = [
  [1122, 1126], // canSetLastAttacker + setPlayerLastAttacker
  [1249, 1256], // startDig
  [1282, 1282], // isUnderMob
  [1309, 1347], // comment + playerBFS
];

const movedText = moveRanges.map(([a, b]) => lines.slice(a - 1, b).join('\n')).join('\n');

const drop = new Set();
for (const [a, b] of moveRanges) for (let i = a; i <= b; i++) drop.add(i);

// Find sim-core closing tag line
let simCoreClose = -1;
for (let i = 0; i < lines.length; i++) {
  if (lines[i].trim() === '</script>' && i > 0) { simCoreClose = i + 1; break; }
}

// Rebuild: insert movedText just before the sim-core </script>, and drop the
// original lines from the main block.
const out = [];
for (let i = 1; i <= lines.length; i++) {
  if (i === simCoreClose) {
    out.push('// ===== Phase-1-shared helpers (also called from hl* in sim-core) =====');
    out.push(movedText);
  }
  if (drop.has(i)) continue;
  out.push(lines[i - 1]);
}

fs.writeFileSync('index.html', out.join('\n'));
console.log(`Moved ${movedText.split('\n').length} lines into sim-core; dropped ${drop.size} lines from main`);
