// Extract each <script> block from index.html and parse it as JS to detect syntax errors.
const fs = require('fs');
const vm = require('vm');
const src = fs.readFileSync('index.html', 'utf8');

function extractBlocks(html) {
  const blocks = [];
  const re = /<script(?:\s+id="([^"]+)")?[^>]*>([\s\S]*?)<\/script>/g;
  let m;
  while ((m = re.exec(html)) !== null) {
    blocks.push({ id: m[1] || '(main)', code: m[2] });
  }
  return blocks;
}

const blocks = extractBlocks(src);
let ok = true;
for (const b of blocks) {
  try {
    new vm.Script(b.code, { filename: `block:${b.id}.js` });
    console.log(`OK ${b.id} (${b.code.split('\n').length} lines)`);
  } catch (e) {
    ok = false;
    console.error(`FAIL ${b.id}: ${e.message}`);
    // Extract line/col from stack
    const m = /block:[^:]+:(\d+)/.exec(e.stack || '');
    if (m) {
      const lineNo = parseInt(m[1], 10);
      const lines = b.code.split('\n');
      const ctx = lines.slice(Math.max(0, lineNo - 3), lineNo + 2)
        .map((l, i) => `  ${lineNo - 2 + i}: ${l}`)
        .join('\n');
      console.error(ctx);
    }
  }
}
process.exit(ok ? 0 : 1);
