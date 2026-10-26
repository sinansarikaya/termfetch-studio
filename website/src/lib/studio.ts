import catalog from './catalog.json';
export { catalog };
export const groups = ['os', 'desktop', 'hardware', 'network', 'time'] as const;
export type Group = typeof groups[number];
export type Settings = Omit<typeof catalog.defaults, 'group_colors'> & { group_colors: Record<Group, string>; image_path: string; image_protocol: string; custom_art: string; recent_themes: string[]; preset_library?: typeof catalog.presets };
export type Theme = typeof catalog.themes[number];
export const defaults: Settings = { ...catalog.defaults, group_colors: { ...catalog.defaults.group_colors }, custom_modules: [...catalog.defaults.custom_modules], image_path: '', image_protocol: 'chafa', custom_art: '', recent_themes: [] };
export function getTheme(s: Settings) { return catalog.themes.find(t => t.id === s.theme)!; }
export function getLogo(s: Settings) { return catalog.logos.find(l => l.id === s.logo)!; }
export function getPreset(s: Settings) {
 const p = (s.preset_library ?? catalog.presets).find(p => p.id === s.preset)!;
 if (p.id !== 'custom') return p;
 const sections: string[][] = [];
 for (const id of s.custom_modules) {
  const prev = sections.at(-1);
  const group = catalog.modules.find(m => m.id === id)!.group;
  if (prev && catalog.modules.find(m => m.id === prev.at(-1))!.group === group) prev.push(id);
  else sections.push([id]);
 }
 return { ...p, sections };
}
export function resolve(value: string, fallback: string) { return value === 'theme' || value === 'inherit' ? fallback : value === 'default' ? '39' : value; }
export function colors(s: Settings, group: string) {
 const t = getTheme(s), g = group as Group, override = s.group_colors[g];
 if (override !== 'inherit') { const color = resolve(override, t.groups[g]); return { key: color, output: color }; }
 return { key: resolve(s.icon_color, t.groups[g]), output: resolve(s.text_color, t.text) };
}
export function sgr(color: string) { return color.startsWith('#') ? '38;2;' + [1,3,5].map(i => parseInt(color.slice(i,i+2),16)).join(';') : color; }
const esc = (color: string, text: string) => '\x1b[1;' + sgr(color) + 'm' + text + '\x1b[0m';
export function dots(s: Settings) { const t=getTheme(s); return [t.primary,t.secondary,t.accent,...groups.map(g=>colors(s,g).key)]; }
export function buildConfig(s: Settings) {
 const t=getTheme(s), p=getPreset(s), logo=getLogo(s), padding={top:2,right:4};
 // ASCII is embedded with Fastfetch's "data" source so exported files are self-contained.
 // Rust uses "file" after writing the same art to its managed logo directory.
 const art=logo.kind === 'custom' ? s.custom_art : logo.art;
 const logoConfig = logo.kind === 'ascii' ? {type:'data',source:art,color:{'1':t.primary},padding}
  : s.image_path.trim() ? {type:s.image_protocol,source:s.image_path.trim(),width:({small:20,medium:36,large:52} as Record<string,number>)[s.logo_size],padding}
  : logo.kind === 'custom' && s.custom_art.trim() ? {type:'data',source:s.custom_art,color:{'1':t.primary},padding}
  : {type:'auto',padding};
 const [user,host]=s.demo_mode ? catalog.demo.user_host.split('@') : ['{user-name}','{host-name}'];
 const title=esc(t.primary,user)+'\x1b['+sgr(resolve(s.text_color,t.text))+'m@\x1b[0m'+esc(t.primary,host);
 const dotEntry=()=>({type:'custom',format:dots(s).map(c=>esc(c,'●')).join(' ')});
 const modules: (string | Record<string, unknown>)[]=['break','break',{type:'title',format:title},'break'];
 if(p.color_dots) modules.push(dotEntry(),'break');
 p.sections.forEach((section,i)=>{
  if(i) modules.push('break');
  section.forEach(id=>{ const m=catalog.modules.find(m=>m.id===id)!, c=colors(s,m.group); modules.push({type:m.id,key:' ├ '+m.icon+' ',...(s.demo_mode?{format:m.demo}:{}),keyColor:c.key,outputColor:c.output,keyWidth:10}); });
 });
 modules.push('break',p.color_dots?dotEntry():'break');
 return {$schema:'https://github.com/fastfetch-cli/fastfetch/raw/dev/doc/json_schema.json',logo:logoConfig,display:{separator:' ➜ '},modules};
}
export function jsonc(s: Settings) {
 const t=getTheme(s),p=getPreset(s);
 return '// TermFetch Studio v2 - generated fastfetch configuration\n// Browser export: ASCII embedded; image paths must exist on your machine.\n// Preset: '+p.name+' | Theme: '+t.name+' ('+t.category+') | Demo mode: '+(s.demo_mode?'on':'off')+'\n'+JSON.stringify(buildConfig(s),null,4)+'\n';
}
export function validColor(value: string) { return /^(theme|inherit|default|3[0-7]|9[0-7]|#[0-9a-fA-F]{6})$/.test(value); }
const ansi: Record<string,string> = {'30':'#000000','31':'#cd0000','32':'#00cd00','33':'#cdcd00','34':'#0000ee','35':'#cd00cd','36':'#00cdcd','37':'#e5e5e5','90':'#7f7f7f','91':'#ff0000','92':'#00ff00','93':'#ffff00','94':'#5c5cff','95':'#ff00ff','96':'#00ffff','97':'#ffffff'};
export function cssColor(value: string, fallback: string) { return /^#[0-9a-f]{6}$/i.test(value)?value:ansi[value]??fallback; }
export function download(text: string, name: string) {
 const url=URL.createObjectURL(new Blob([text],{type:'application/json'})), a=document.createElement('a');
 a.href=url; a.download=name; a.click(); setTimeout(()=>URL.revokeObjectURL(url),1000);
}
