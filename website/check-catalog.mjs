import { readFileSync, writeFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import assert from 'node:assert/strict';
import ts from 'typescript';
import { extract } from './catalog-parser.mjs';
const root=fileURLToPath(new URL('../',import.meta.url)), here=fileURLToPath(new URL('./',import.meta.url));
const hashes=JSON.parse(readFileSync(here+'catalog-sources.json','utf8'));
const sources={};
const update=process.argv.includes('--update');
for(const [path,hash] of Object.entries(hashes)) {
 const text=readFileSync(root+path,'utf8');sources[path]=text;
 const actual=createHash('sha256').update(text).digest('hex');
 if(update) hashes[path]=actual;
 else if(actual!==hash) throw Error('Rust source drift: '+path+'. Review writer/defaults and regenerate catalog-sources.json; do not silently accept drift.');
}
const actual=extract(sources);
if(update) {
 writeFileSync(here+'catalog-sources.json',JSON.stringify(hashes,null,2)+'\n');
 writeFileSync(here+'src/lib/catalog.json',JSON.stringify(actual,null,2)+'\n');
}
const snapshot=JSON.parse(readFileSync(here+'src/lib/catalog.json','utf8'));
assert.deepEqual(actual,snapshot,'Catalog snapshot must match Rust');
const text=readFileSync(here+'src/lib/studio.ts','utf8').replace("import catalog from './catalog.json';",'const catalog = '+JSON.stringify(snapshot)+';');
const js=ts.transpileModule(text,{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.ESNext}}).outputText;
const studio=await import('data:text/javascript;base64,'+Buffer.from(js).toString('base64'));
const {defaults,buildConfig,getPreset,colors,jsonc}=studio;
assert.equal(snapshot.themes.length,40);assert.equal(snapshot.modules.length,25);assert.equal(snapshot.logos.length,31);
assert.deepEqual([defaults.theme,defaults.preset,defaults.logo,defaults.logo_size,defaults.demo_mode],['catppuccin-mocha','full','arch','medium',true]);
let combinations=0;
for(const theme of snapshot.themes)for(const preset of snapshot.presets)for(const demo_mode of [true,false]){
 const s={...structuredClone(defaults),theme:theme.id,preset:preset.id,demo_mode}, cfg=buildConfig(s), p=getPreset(s);
 assert.equal(cfg.display.separator,' ➜ ');assert.deepEqual(cfg.logo.padding,{top:2,right:4});
 assert.deepEqual(cfg.modules.slice(0,2),['break','break']);assert.equal(cfg.modules[2].type,'title');assert.equal(cfg.modules[2].key,undefined);
 const modules=cfg.modules.filter(m=>typeof m==='object'&&m.keyWidth===10);
 assert.deepEqual(modules.map(m=>m.type),p.sections.flat());
 modules.forEach(m=>{const def=snapshot.modules.find(d=>d.id===m.type);assert.equal(m.key,' ├ '+def.icon+' ');assert.equal(m.format,demo_mode?def.demo:undefined);assert.equal(m.keyColor,theme.groups[def.group]);assert.equal(m.outputColor,theme.text);});
 assert.equal(cfg.modules.filter(m=>typeof m==='object'&&m.type==='custom').length,p.color_dots?2:0);
 assert.equal(cfg.modules.filter(m=>m==='break').length,3+(p.color_dots?1:0)+Math.max(0,p.sections.length-1)+1+(p.color_dots?0:1));
 assert.deepEqual(JSON.parse(jsonc(s).split('\n').filter(l=>!l.startsWith('//')).join('\n')),cfg);
 assert.ok(cfg.modules[2].format.includes(demo_mode?'cyberdeck':'{user-name}'));combinations++;
}
const s=structuredClone(defaults);s.group_colors.network='#ff8c00';s.icon_color='#abcdef';s.text_color='default';
assert.deepEqual(colors(s,'network'),{key:'#ff8c00',output:'#ff8c00'});assert.deepEqual(colors(s,'os'),{key:'#abcdef',output:'39'});
s.group_colors.os='theme';assert.deepEqual(colors(s,'os'),{key:snapshot.themes[0].groups.os,output:snapshot.themes[0].groups.os});
s.preset='custom';s.custom_modules=['cpu','os','gpu','memory'];assert.deepEqual(getPreset(s).sections,[['cpu'],['os'],['gpu','memory']]);
s.custom_modules=[];assert.equal(buildConfig(s).modules.filter(m=>typeof m==='object'&&m.keyWidth).length,0);
s.logo='pikachu';assert.equal(buildConfig(s).logo.type,'auto');s.image_path='/home/example/logo.png';assert.equal(buildConfig(s).logo.source,s.image_path);assert.equal(buildConfig(s).logo.width,36);
s.logo='arch';assert.equal(buildConfig(s).logo.type,'data');assert.equal(buildConfig(s).logo.source,snapshot.logos[0].art);
s.logo='custom-user-image';s.image_path='';s.custom_art=' /\\_/\\ ';assert.equal(buildConfig(s).logo.source,s.custom_art);
assert.equal(studio.validColor('#123abc'),true);assert.equal(studio.validColor('red;}body{display:none'),false);
const report={status:'Pass',assertions:['Rust source hashes unchanged','Catalog extraction matches snapshot','480 theme/preset/demo combinations preserve writer module order, breaks, key width, separator, title and values','Group override precedence and default/ANSI resolution','Custom regrouping and empty modules','Self-contained ASCII and explicit image path/auto fallback','Export JSONC parses to generated object'],combinations,limitations:['Rust executable was not run: writer compatibility is source-derived, with explicit ASCII data/image sizing adaptations.','Headed browser verification requires a separately authorized QA command.']};
writeFileSync(here+'verification.json',JSON.stringify(report,null,2)+'\n');console.log('Catalog drift checks and '+combinations+' Fastfetch configuration combinations passed.');
