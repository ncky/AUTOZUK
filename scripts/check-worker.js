// Simulate what buildWorkerBlobUrl will do: concat sim-core + worker glue and verify
// it parses + handles each message type correctly in a worker-like sandbox.
const fs = require('fs');
const vm = require('vm');
const src = fs.readFileSync('index.html', 'utf8');
const simCore = /<script id="sim-core">([\s\S]*?)<\/script>/.exec(src)[1];
const workerSrc = /<script id="autozuk-worker"[^>]*>([\s\S]*?)<\/script>/.exec(src)[1];
const combined = simCore + ';\n' + workerSrc;

// Fake `self` object that records postMessage calls
let posted = [];
const self = {
  postMessage(m){ posted.push(m); },
  onmessage: null,
};
const ctx = vm.createContext({ console, Math, Set, Map, Uint8Array, Infinity, NaN, isNaN, parseInt, parseFloat, JSON, self });

try { vm.runInContext(combined, ctx); } catch (e) {
  console.error('Worker combined source failed to load:', e.message);
  process.exit(1);
}
if (typeof self.onmessage !== 'function') { console.error('self.onmessage not registered'); process.exit(1); }

// Run probe inside the vm so it can see const-bound LOADOUTS
const probe = `
self._test = {};
self.onmessage({ data: { type:'init', pillarConfig:{S:true,W:true,N:true}, loadout: LOADOUTS.blowpipe } });
self.onmessage({ data: { type:'exclude', tiles:[{x:5,y:5},{x:15,y:15},{x:11,y:24}], spawnCode:'MRYBXOOOO' } });
self.onmessage({ data: { type:'simulate', tile:{x:15,y:15}, spawnCode:'MRYBXOOOO', loadout: LOADOUTS.blowpipe, maxTicks:400, maxSims:20, seedBase:42 } });
`;
vm.runInContext(probe, ctx);
console.log('OK init:', posted[0]);
console.log('OK exclude:', posted[1].excluded.length, 'excluded,', posted[1].eligible.length, 'eligible');
console.log('OK simulate:', posted[2].summary
  ? `avgDamage=${posted[2].summary.avgDamage.toFixed(1)}, totalSims=${posted[2].summary.totalSims}, prayer=${JSON.stringify(posted[2].summary.prayer)}`
  : 'null summary');
