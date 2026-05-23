// Mirror of window.__autozukDevTest, runs against the extracted sim-core block.
const fs = require('fs');
const vm = require('vm');
const src = fs.readFileSync('index.html', 'utf8');
const code = /<script id="sim-core">([\s\S]*?)<\/script>/.exec(src)[1];

const ctx = vm.createContext({ console, Math, Set, Map, Uint8Array, Infinity, NaN, isNaN, parseInt, parseFloat, JSON });
vm.runInContext(code, ctx);

const probe = `
function __fnv1a(str){
  let h = 0x811c9dc5;
  for(let i=0;i<str.length;i++){h ^= str.charCodeAt(i); h = Math.imul(h, 0x01000193);}
  return (h >>> 0).toString(16).padStart(8,'0');
}
const scenarios = [
  {code:'MRYBXOOOO', tile:{x:15,y:15}, sims:20, seed:1},
  {code:'MMRRX',     tile:{x:10,y:10}, sims:20, seed:2},
  {code:'XXXBB',     tile:{x:20,y:20}, sims:20, seed:3},
];
const pillarConfig = {S:true,W:true,N:true};
const loadout = LOADOUTS.blowpipe;
const region = createRegion(pillarConfig);
let parts = [];
for(const sc of scenarios){
  let results = [];
  for(let s=0;s<sc.sims;s++){
    const r = hlRunSim(sc.code, sc.tile, pillarConfig, loadout, 400, region, sc.seed*1000 + s);
    if(!r){parts.push('null'); continue;}
    results.push({
      t: r.completedTick,
      st: r.status,
      n: r.attacks.length,
      a: r.attacks.map(a=>a.tick+'|'+(a.mobId??'')+'|'+(a.style??'')+'|'+(a.isPlayerAttack?'P':'')+'|'+(a.hitTick??'')+'|'+(a.dmgRoll?Math.floor(a.dmgRoll*1000):'')+'|'+(a.accRoll?Math.floor(a.accRoll*1000):'')).join(','),
    });
  }
  parts.push(JSON.stringify(results));
}
globalThis.__hash = __fnv1a(parts.join('|'));
`;
vm.runInContext(probe, ctx);
console.log(ctx.__hash);
