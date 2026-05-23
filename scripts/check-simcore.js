// Confirm sim-core block actually defines what the worker needs and doesn't touch DOM globals.
const fs = require('fs');
const vm = require('vm');
const src = fs.readFileSync('index.html', 'utf8');
const m = /<script id="sim-core">([\s\S]*?)<\/script>/.exec(src);
if (!m) { console.error('No sim-core block'); process.exit(1); }
const code = m[1];

// Create a worker-like sandbox: no `document`, no `window`, no `navigator`.
// Provide globalThis, Math, console.
const sandbox = { console, Math, Set, Map, Uint8Array, Infinity, NaN, isNaN, parseInt, parseFloat, JSON };
try {
  vm.runInNewContext(code, sandbox);
} catch (e) {
  console.error('Sim-core failed to load in worker-like sandbox:', e.message);
  process.exit(1);
}

// Smoke: append a call that uses the sim-core globals from within its own scope.
// `const`/`let` declarations are in scope to code in the same script but not on the sandbox object.
const probe = `
;(function(){
  const region = createRegion({S:true,W:true,N:true});
  globalThis.__probeResult = hlRunSim('MRYBXOOOO', {x:15,y:15}, {S:true,W:true,N:true}, LOADOUTS.blowpipe, 400, region, 42);
  globalThis.__probeExcluded = checkTileExcluded(15, 15, [], region);
  let r = globalThis.__probeResult;
  globalThis.__probePrayer = optimizePrayer([r], 'MRYBXOOOO', {S:true,W:true,N:true}, LOADOUTS.blowpipe);
  globalThis.__probeDmg = calcSimDamage(r.attacks, globalThis.__probePrayer.sequence, LOADOUTS.blowpipe, r.mobInitHP);
})();
`;
try { vm.runInContext(probe, vm.createContext(sandbox)); } catch (e) {
  console.error('Probe failed:', e.message);
  process.exit(1);
}
// Use a single context that has access to both the sim-core code and our probe
const ctx = vm.createContext({ console, Math, Set, Map, Uint8Array, Infinity, NaN, isNaN, parseInt, parseFloat, JSON });
try { vm.runInContext(code + '\n' + probe, ctx); } catch (e) {
  console.error('Combined run failed:', e.message);
  process.exit(1);
}
const r = ctx.__probeResult;
if (!r) { console.error('hlRunSim returned null'); process.exit(1); }
console.log('OK sim-core: hlRunSim ->', r.status, r.completedTick, 'ticks,', r.attacks.length, 'attacks');
console.log('OK checkTileExcluded ->', ctx.__probeExcluded);
console.log('OK optimizePrayer ->', JSON.stringify(ctx.__probePrayer.sequence));
console.log('OK calcSimDamage ->', JSON.stringify(ctx.__probeDmg));
