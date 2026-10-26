// Generated catalog extraction is deliberately strict: unknown Rust shapes fail the build.
export function extract(sources) {
 const theme = sources['src/domain/theme.rs'], preset = sources['src/domain/preset.rs'], logo = sources['src/domain/logo.rs'];
 const field = (body, name) => { const m = body.match(new RegExp('\\b'+name+':\\s*"([^"]*)"')); if (!m) throw Error('Missing '+name); return m[1]; };
 const themes = [...theme.matchAll(/Theme \{\s*id:([\s\S]*?)\n        \},/g)].map(m => {
 const b='id:'+m[1], out={}; for (const k of ['id','name','primary','secondary','accent','background','text']) out[k]=field(b,k);
 out.category=b.match(/category: ThemeCategory::(\w+)/)[1]; out.groups={}; for (const k of ['os','desktop','hardware','network','time']) out.groups[k]=field(b,k); return out; });
 const demoBody=preset.match(/pub const DEMO_DATA: DemoData = DemoData \{([\s\S]*?)\n\};/)[1];
 const demo=Object.fromEntries([...demoBody.matchAll(/(\w+): "([^"]*)"/g)].map(m=>[m[1],m[2]]));
 const arm=(method)=>preset.match(new RegExp('pub fn '+method+'\\(self\\)[\\s\\S]*?match self \\{([\\s\\S]*?)\\n        \\}'))[1];
 const pairs=(method)=>Object.fromEntries([...arm(method).matchAll(/Self::(\w+) => "([^"]*)"/g)].map(m=>[m[1],m[2]]));
 const ids=pairs('fastfetch_type'), labels=pairs('label'), icons=pairs('icon'), descriptions=pairs('description');
 const values=Object.fromEntries([...arm('demo_value').matchAll(/Self::(\w+) => (?:"([^"]*)"|DEMO_DATA\.(\w+))/g)].map(m=>[m[1],m[2]??demo[m[3]]]));
 const groups={}; for(const m of arm('group').matchAll(/((?:Self::\w+\s*\|?\s*)+) => SemanticGroup::(\w+)/g)) for(const n of m[1].matchAll(/Self::(\w+)/g)) groups[n[1]]=m[2].toLowerCase();
 const modules=Object.entries(ids).map(([rust,id])=>({id,label:labels[rust],icon:icons[rust],description:descriptions[rust],group:groups[rust],demo:values[rust]}));
 const presets=[...preset.matchAll(/Preset \{\s*id: "([^"]+)"([\s\S]*?)color_dots: (true|false),/g)].map(m=>({id:m[1],name:field(m[2],'name'),description:field(m[2],'description'),sections:[...m[2].matchAll(/vec!\[([A-Za-z,\s]+)\]/g)].map(v=>v[1].split(',').map(x=>x.trim()).filter(Boolean).map(x=>ids[x])),color_dots:m[3]==='true'}));
 const custom=['os','kernel','shell','uptime']; presets.push({id:'custom',name:'Custom Modules',description:'Your own module selection and order from the Module Builder',sections:[custom.slice(0,3),['uptime']],color_dots:true});
 const logos=[...logo.matchAll(/LogoItem \{\s*id: "([^"]+)"([\s\S]*?)file_path: None,/g)].filter(m=>m[2].includes('art: r#"')).map(m=>({id:m[1],name:field(m[2],'name'),group:m[2].match(/group: LogoGroup::(\w+)/)[1],kind:'ascii',art:m[2].match(/art: r#"([\s\S]*?)"#/)[1]}));
 for(const m of logo.matchAll(/\("([^"]+\.png)", "([^"]+)"\)/g)) logos.push({id:m[1].slice(0,-4),name:m[2],group:'PresetImage',kind:'image',file:m[1]});
 logos.push({id:'custom-user-image',name:'Custom Image (custom.png)',group:'Custom',kind:'custom'});
 const settings=sources['src/domain/settings.rs']; const defaults={theme:field(settings,'theme'),preset:field(settings,'preset'),logo:field(settings,'logo'),logo_size:'medium',demo_mode:true,custom_modules:custom,icon_color:'theme',text_color:'theme',group_colors:{os:'inherit',desktop:'inherit',hardware:'inherit',network:'inherit',time:'inherit'},apply_terminal_colors:true,run_fastfetch_on_exit:true,ui_follow_theme:false};
 const palette=[...sources['src/domain/color.rs'].matchAll(/\("([^"]+)", "([^"]+)"\)/g)].map(m=>({value:m[1],label:m[2]}));
 if(themes.length<20||presets.length!==6||modules.length!==25||logos.length!==30||modules.some(m=>!m.group||!m.demo)) throw Error('Catalog shape changed; update extractor.');
 return {version:sources['Cargo.toml'].match(/version = "([^"]+)"/)[1],themes,presets,modules,logos,demo,defaults,palette};
}
