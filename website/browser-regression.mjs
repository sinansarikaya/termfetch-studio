// Single-worker, real headed desktop QA of the native-faithful browser studio.
import { createServer } from 'node:http';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { resolve, extname, sep } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { execFileSync } from 'node:child_process';
import assert from 'node:assert/strict';
const here = fileURLToPath(new URL('./', import.meta.url)), dist = resolve(here, 'dist'), evidence = resolve(here, 'qa/revision-2');
await mkdir(evidence, { recursive: true });
const report = { status: 'Blocked', assertions: [], errors: [], headless: false, startedAt: new Date().toISOString(), limitations: ['Browser recreation shares the Rust catalog and JSONC rules; it does not execute the Rust TUI engine.', 'Local headed performance snapshot is not a field metric, SEO score, ranking or native platform guarantee.'] };
let browser, server, context;
const check = name => report.assertions.push(name);
const parse = text => JSON.parse(text.split('\n').filter(line => !line.startsWith('//')).join('\n'));
try {
 const module = process.env.PLAYWRIGHT_MODULE || '/home/sinan/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright/index.mjs';
 const { chromium } = await import(module.startsWith('/') ? pathToFileURL(module).href : module);
 server = createServer(async (req, res) => {
  const pathname = decodeURIComponent(new URL(req.url, 'http://localhost').pathname);
  let path = resolve(dist, '.' + (pathname === '/' ? '/index.html' : pathname));
  if (!path.startsWith(dist + sep)) return res.writeHead(403).end();
  if (!extname(path)) path = resolve(path, 'index.html');
  let status = 200, data;
  try { data = await readFile(path); } catch { if (extname(pathname)) return res.writeHead(404).end('Not found'); path = resolve(dist, '404.html'); status = 404; data = await readFile(path); }
  const types = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css', '.svg': 'image/svg+xml', '.png': 'image/png', '.xml': 'application/xml', '.txt': 'text/plain', '.woff2': 'font/woff2' };
  res.writeHead(status, { 'Content-Type': types[extname(path)] || 'application/octet-stream' }).end(data);
 });
 await new Promise(done => server.listen(0, '127.0.0.1', done));
 const base = process.env.BASE_URL || 'http://127.0.0.1:' + server.address().port; report.target = base;
 browser = await chromium.launch({ headless: false, executablePath: process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE || '/home/sinan/.cache/ms-playwright/chromium-1234/chrome-linux64/chrome', args: process.platform === 'linux' ? ['--ozone-platform=x11'] : [] });
 context = await browser.newContext({ viewport: { width: 1660, height: 1200 }, acceptDownloads: true });
 await context.addInitScript(() => { window.__localVitals = { cls: 0, lcp: null }; try { new PerformanceObserver(list => list.getEntries().forEach(e => { if (!e.hadRecentInput) window.__localVitals.cls += e.value; })).observe({ type: 'layout-shift', buffered: true }); new PerformanceObserver(list => list.getEntries().forEach(e => { window.__localVitals.lcp = e.startTime; })).observe({ type: 'largest-contentful-paint', buffered: true }); } catch { /* Unsupported metrics remain unavailable. */ } });
 await context.tracing.start({ screenshots: true, snapshots: true });
 const page = await context.newPage();
 page.on('pageerror', error => report.errors.push(error.message));
 page.on('console', msg => { if (msg.type() === 'error') report.errors.push(msg.text()); });
 page.on('response', response => { if (response.status() >= 400 && response.url().startsWith(base)) report.errors.push(response.status() + ' ' + response.url()); });
 await page.goto(base, { waitUntil: 'networkidle' }); await page.evaluate(() => document.fonts.ready); await page.waitForTimeout(200);
 report.localPerformance = await page.evaluate(() => { const n = performance.getEntriesByType('navigation')[0]; return { context: 'Fresh local headed Chromium navigation, 1660×1200, brief observation before interaction', navigationDurationMs: n?.duration ?? null, responseStartMs: n?.responseStart ?? null, lcpMs: window.__localVitals?.lcp ?? null, cls: window.__localVitals?.cls ?? null }; });
 if (process.platform === 'linux') { const windows = execFileSync('xwininfo', ['-root', '-tree'], { encoding: 'utf8' }); assert.match(windows, /Chromium|Chrome/i); await writeFile(resolve(evidence, 'desktop-window.txt'), windows); }
 const app = page.locator('.native-window'), section = name => app.getByRole('navigation', { name: 'Studio sections' }).getByRole('button', { name: new RegExp(name) });
 const output = async () => parse(await app.locator('.native-json-output').textContent());
 const select = label => app.getByLabel(label, { exact: true });
 assert.equal(await select('Theme').inputValue(), 'synthwave'); assert.equal(await select('Preset').inputValue(), 'full'); assert.equal(await select('Logo').inputValue(), 'pikachu');
 assert.equal(await app.locator('.native-info-line').count(), 25); assert.equal(await app.locator('.native-info-group').count(), 5);
 assert.ok(await page.evaluate(() => document.fonts.check('13px "TermFetch Mono"')));
 assert.ok(await app.locator('.module-glyph').evaluateAll(items => items.every(el => [...el.textContent].length === 1 && el.textContent.codePointAt(0) > 0xe000)));
 const geometry = await app.evaluate(el => { const rect = s => el.querySelector(s).getBoundingClientRect(); return { window: el.getBoundingClientRect().width, side: rect('.native-sidebar').width, preview: rect('.native-preview-column').width, centerLeft: rect('.native-stage').left, sideRight: rect('.native-sidebar').right }; });
 assert.ok(geometry.side >= 170 && geometry.side <= 230); assert.ok(geometry.preview / geometry.window > .43 && geometry.preview / geometry.window < .46); assert.equal(geometry.centerLeft, geometry.sideRight);
 assert.equal(await app.locator('.native-header').evaluate(el => getComputedStyle(el).borderBottomColor), 'rgb(90, 99, 128)');
 assert.ok(await app.locator('[data-module="media"]').evaluate(el => { const pane = el.closest('.native-preview-output'); return el.getBoundingClientRect().bottom <= pane.getBoundingClientRect().bottom + 1; }), 'All 25 Full Info modules fit the initial desktop preview');
 const groupColors = await app.locator('.native-info-group').evaluateAll(items => items.map(el => getComputedStyle(el.querySelector('.module-value')).color)); assert.equal(new Set(groupColors).size, 5);
 await app.screenshot({ path: resolve(evidence, 'studio-native-desktop.png') });
 await page.screenshot({ path: resolve(evidence, 'landing-desktop.png'), fullPage: true });
 check('Native navy/indigo geometry, 44% preview, licensed glyph font, image artwork, all25 icon/tree modules and5 semantic colors');

 await select('Output').selectOption('jsonc'); let config = await output(); assert.equal(config.modules.filter(m => m.keyWidth).length, 25); assert.ok(config.modules[2].format.includes('\x1b'));
 await app.getByRole('button', { name: 'Copy preset', exact: true }).click(); assert.equal(await select('Preset').inputValue(), 'user-1'); assert.equal((await output()).modules.filter(m => m.keyWidth).length, 25);
 await app.getByRole('button', { name: 'Rename preset', exact: true }).click(); await app.getByLabel('Preset name', { exact: true }).fill('Neon Full Info'); await app.getByRole('button', { name: 'Save preset name', exact: true }).click(); assert.match(await app.locator('.native-json-output').textContent(), /Neon Full Info/);
 await app.getByRole('button', { name: /Apply setup locally/ }).click();
 const downloaded = page.waitForEvent('download'); await page.getByRole('button', { name: 'Download JSONC ↓', exact: true }).click(); const file = await downloaded; await file.saveAs(resolve(evidence, 'copied-preset.jsonc')); config = parse(await readFile(resolve(evidence, 'copied-preset.jsonc'), 'utf8')); assert.equal(config.modules.filter(m => m.keyWidth).length, 25); assert.equal(config.modules.filter(m => m.type === 'custom').length, 0);
 await select('Preset').selectOption('minimal'); await section('Settings').click(); await app.getByRole('button', { name: 'Restore browser save', exact: true }).click(); assert.equal(await select('Preset').inputValue(), 'user-1');
 await section('Presets').click(); await app.getByRole('button', { name: 'Delete preset', exact: true }).click(); await app.getByLabel('Type yes to delete preset', { exact: true }).fill('yes'); await app.getByRole('button', { name: 'Confirm delete preset', exact: true }).click(); assert.equal(await select('Preset').inputValue(), 'full'); assert.equal(await select('Preset').locator('option').count(), 6);
 check('Native-style preset copy/rename/confirmeddelete, section/color-dot fidelity, persisted browser library and real exportable25module JSONC');

 // Custom Modules is an editable preset, but structural module actions must
 // restore its definition before activating it after deletion.
 await select('Preset').selectOption('custom');
 await app.getByRole('button', { name: 'Delete preset', exact: true }).click();
 await app.getByLabel('Type yes to delete preset', { exact: true }).fill('yes');
 await app.getByRole('button', { name: 'Confirm delete preset', exact: true }).click();
 assert.equal(await select('Preset').locator('option[value="custom"]').count(), 0);
 await section('Modules').click();
 await app.getByRole('checkbox', { name: /CPU/ }).check();
 assert.equal(await select('Preset').inputValue(), 'custom');
 assert.equal(await select('Preset').locator('option[value="custom"]').count(), 1);
 await app.getByRole('button', { name: 'Move CPU up', exact: true }).click();
 assert.deepEqual((await output()).modules.filter(m => m.keyWidth).map(m => m.type), ['os', 'kernel', 'shell', 'cpu', 'uptime']);
 await select('Preset').selectOption('full');
 await app.getByRole('button', { name: 'Use current preset as a starting point', exact: true }).click();
 assert.equal(await select('Preset').inputValue(), 'custom');
 assert.equal((await output()).modules.filter(m => m.keyWidth).length, 25);
 await app.getByRole('button', { name: /Apply setup locally/ }).click();
 const customDownload = page.waitForEvent('download');
 await page.getByRole('button', { name: 'Download JSONC ↓', exact: true }).click();
 await (await customDownload).saveAs(resolve(evidence, 'restored-custom-preset.jsonc'));
 const customConfig = parse(await readFile(resolve(evidence, 'restored-custom-preset.jsonc'), 'utf8'));
 assert.equal(customConfig.modules.filter(m => m.keyWidth).length, 25);
 await select('Preset').selectOption('minimal');
 await section('Settings').click();
 await app.getByRole('button', { name: 'Restore browser save', exact: true }).click();
 assert.equal(await select('Preset').inputValue(), 'custom');
 assert.equal((await output()).modules.filter(m => m.keyWidth).length, 25);
 const restoredLibrary = JSON.parse(await page.evaluate(() => localStorage.getItem('termfetch-browser-export')));
 assert.equal(restoredLibrary.preset_library.filter(item => item.id === 'custom').length, 1);
 assert.equal(restoredLibrary.custom_modules.length, 25);
 // Keep the following independent module tests on their documented defaults.
 await app.getByRole('button', { name: 'Reset to native defaults', exact: true }).click();
 check('Deleting Custom Modules then toggling/reordering/adopting restores its definition; apply/export/restore retains all25modules without duplicate options or runtime errors');

 // Exercise capacity through real copy actions, then verify saved data can be
 // restored and the reserved Custom slot never forces an oversized library.
 await section('Presets').click();
 while (await select('Preset').locator('option').count() < 40) {
  await app.getByRole('button', { name: 'Copy preset', exact: true }).click();
 }
 assert.equal(await app.getByRole('button', { name: 'Copy preset', exact: true }).isDisabled(), true);
 assert.match(await app.getByRole('status').textContent(), /capacity reached/);
 const copiedAtCapacity = await select('Preset').inputValue();
 await app.getByRole('button', { name: /Apply setup locally/ }).click();
 await select('Preset').selectOption('minimal'); await section('Settings').click();
 await app.getByRole('button', { name: 'Restore browser save', exact: true }).click();
 assert.equal(await select('Preset').inputValue(), copiedAtCapacity);
 assert.match(await app.getByRole('status').textContent(), /capacity reached/);
 assert.equal(await select('Preset').locator('option').count(), 40);
 const beforeCapacityDelete = JSON.parse(await page.evaluate(() => localStorage.getItem('termfetch-browser-export')));
 await section('Presets').click(); await select('Preset').selectOption('custom');
 await app.getByRole('button', { name: 'Delete preset', exact: true }).click();
 await app.getByLabel('Type yes to delete preset', { exact: true }).fill('yes');
 await app.getByRole('button', { name: 'Confirm delete preset', exact: true }).click();
 assert.equal(await select('Preset').locator('option').count(), 39);
 assert.equal(await app.getByRole('button', { name: 'Copy preset', exact: true }).isDisabled(), true);
 await app.locator('.native-preset-list > button[aria-pressed="true"]').focus();
 await page.keyboard.press('d');
 assert.equal(await select('Preset').locator('option').count(), 39);
 assert.match(await app.getByRole('status').textContent(), /slot is reserved for Custom Modules/);
 await section('Modules').click(); await app.getByRole('checkbox', { name: /CPU/ }).check();
 assert.equal(await select('Preset').inputValue(), 'custom');
 assert.equal(await select('Preset').locator('option').count(), 40);
 assert.equal(await select('Preset').locator('option[value="custom"]').count(), 1);
 await select('Preset').selectOption('full');
 await app.getByRole('button', { name: 'Use current preset as a starting point', exact: true }).click();
 await app.getByRole('button', { name: /Apply setup locally/ }).click();
 const restoredAtCapacity = JSON.parse(await page.evaluate(() => localStorage.getItem('termfetch-browser-export')));
 assert.equal(restoredAtCapacity.preset_library.length, 40);
 assert.deepEqual(restoredAtCapacity.preset_library.filter(p => p.id !== 'custom').map(p => p.id).sort(), beforeCapacityDelete.preset_library.filter(p => p.id !== 'custom').map(p => p.id).sort());
 await section('Settings').click(); await select('Preset').selectOption('minimal');
 await app.getByRole('button', { name: 'Restore browser save', exact: true }).click();
 assert.equal(await select('Preset').inputValue(), 'custom');
 assert.equal((await output()).modules.filter(m => m.keyWidth).length, 25);
 // An older/external full library with no Custom reservation is rejected,
 // preserving the current valid library instead of dropping any user preset.
 await page.evaluate(raw => { const s = JSON.parse(raw); s.preset = 'full'; s.preset_library = s.preset_library.filter(p => p.id !== 'custom'); s.preset_library.push({ ...s.preset_library[0], id: 'external-extra', name: 'External extra' }); localStorage.setItem('termfetch-browser-export', JSON.stringify(s)); }, JSON.stringify(restoredAtCapacity));
 await app.getByRole('button', { name: 'Restore browser save', exact: true }).click();
 assert.match(await app.getByRole('status').textContent(), /invalid/);
 assert.equal(await select('Preset').inputValue(), 'custom');
 assert.equal(await select('Preset').locator('option').count(), 40);
 await app.getByRole('button', { name: 'Reset to native defaults', exact: true }).click();
 check('Realcopycapacity40 applies/restores; deletingCustomreservesaslot, keyboardcopyguardholds39, moduleactivationrestores40withoutdroppinguserpresets; external40-withoutCustomrejected');

 await select('Theme').selectOption('nord'); await select('Preset').selectOption('minimal'); await select('Logo').selectOption('cat');
 assert.equal((await output()).logo.type, 'data'); assert.equal((await output()).modules.filter(m => m.keyWidth).length, 4);
 await section('Modules').click(); await app.getByRole('checkbox', { name: /CPU/ }).check(); await app.getByRole('button', { name: 'Move CPU up', exact: true }).click(); assert.deepEqual((await output()).modules.filter(m => m.keyWidth).map(m => m.type), ['os', 'kernel', 'shell', 'cpu', 'uptime']);
 await app.getByRole('checkbox', { name: /CPU/ }).uncheck(); assert.equal((await output()).modules.filter(m => m.keyWidth).length, 4);
 await select('Preset').selectOption('full'); await app.getByRole('button', { name: 'Use current preset as a starting point', exact: true }).click(); assert.equal((await output()).modules.filter(m => m.keyWidth).length, 25);
 check('Theme/ASCII/preset selections, module toggles/reorder and adoptingfullpreset are functional');

 await section('Settings').click(); const originalOS = (await output()).modules.find(m => m.type === 'os').keyColor;
 await app.getByRole('button', { name: 'Edit colors', exact: true }).click(); await app.getByLabel('Custom hex for os', { exact: true }).fill('#ff8c00'); assert.equal((await output()).modules.find(m => m.type === 'os').outputColor, '#ff8c00'); assert.equal(await page.getByRole('button', { name: 'Download JSONC ↓', exact: true }).isDisabled(), true);
 await app.getByRole('button', { name: 'Cancel colors', exact: true }).click(); assert.equal((await output()).modules.find(m => m.type === 'os').keyColor, originalOS);
 await app.getByRole('button', { name: 'Edit colors', exact: true }).click(); await app.getByLabel('Custom hex for os', { exact: true }).fill('#ff8c00'); await app.getByRole('button', { name: 'Apply colors', exact: true }).click(); assert.equal((await output()).modules.find(m => m.type === 'os').outputColor, '#ff8c00');
 await app.getByRole('button', { name: 'Edit colors', exact: true }).click(); await app.getByRole('button', { name: 'Reset colors', exact: true }).click(); await app.getByRole('button', { name: 'Apply colors', exact: true }).click(); assert.equal((await output()).modules.find(m => m.type === 'os').keyColor, originalOS);
 await app.getByRole('checkbox', { name: 'Demo mode', exact: true }).uncheck(); assert.equal((await output()).modules.find(m => m.type === 'os').format, undefined);
 await select('Logo').selectOption('pikachu'); await app.getByLabel('Image path on your machine', { exact: true }).fill('/home/example/logo.png'); await select('Image size').selectOption('large'); await select('Image protocol').selectOption('kitty-direct'); assert.equal((await output()).logo.width, 52); assert.equal((await output()).logo.type, 'kitty-direct');
 await select('Image size').selectOption('small'); assert.equal((await output()).logo.width, 20);
 await app.getByRole('button', { name: /Apply setup locally/ }).click(); assert.match(await app.getByRole('status').textContent(), /host files unchanged/);
 const saved = await page.evaluate(() => localStorage.getItem('termfetch-browser-export')); await page.evaluate(raw => localStorage.setItem('termfetch-browser-export', JSON.stringify({ ...JSON.parse(raw), ui_follow_theme: 'bad' })), saved); await app.getByRole('button', { name: 'Restore browser save', exact: true }).click(); assert.match(await app.getByRole('status').textContent(), /invalid/);
 check('Semantic liveedit/apply/cancel/reset, draftexportguard, demo-off export, imageprotocol/size/path and invalidstorage rejection');

 await app.focus(); await page.keyboard.press('2'); await app.getByRole('searchbox', { name: 'Search themes', exact: true }).fill('aod?'); assert.equal(await app.getByRole('dialog').count(), 0); await app.focus(); await page.keyboard.press('6'); await page.keyboard.press('d'); assert.equal(await app.getByRole('checkbox', { name: 'Demo mode', exact: true }).isChecked(), true); await page.keyboard.press('?'); assert.equal(await app.getByRole('dialog', { name: 'Keyboard shortcuts', exact: true }).count(), 1); await page.keyboard.press('Escape');
 await select('Output').selectOption('preview'); await app.getByRole('button', { name: 'Expand preview', exact: true }).click(); assert.equal(await app.evaluate(el => el.classList.contains('preview-expanded')), true); await app.getByRole('button', { name: 'Restore studio layout', exact: true }).click();
 await page.getByRole('button', { name: /Gaming.*master-chief/i }).click(); assert.equal(await select('Theme').inputValue(), 'tokyo-night'); assert.equal(await select('Preset').inputValue(), 'gaming'); assert.equal(await select('Logo').inputValue(), 'master-chief');
 await page.locator('.marketing-theme-card').filter({ hasText: 'Nord Aurora' }).getByRole('button').click(); assert.equal(await select('Theme').inputValue(), 'nord');
 check('Scoped keyboard/input behavior, previewexpand/restore and actualmarketinglook/theme buttons update studio');

 await page.goto(base + '/?theme=nord#studio', { waitUntil: 'networkidle' }); await page.waitForFunction(() => document.querySelector('.native-options select[aria-label="Theme"]')?.value === 'nord'); assert.equal(await select('Preset').inputValue(), 'full');
 await page.goto(base + '/?look=minimal#studio', { waitUntil: 'networkidle' }); await page.waitForFunction(() => document.querySelector('.native-options select[aria-label="Preset"]')?.value === 'minimal');

 for (const [presetId, name, count] of [['focused', 'Focused', 9], ['custom', 'Custom Modules', 4]]) {
  await page.goto(base + '/presets', { waitUntil: 'networkidle' });
  await page.locator('.preset-comparison > article').filter({ has: page.getByRole('heading', { name, exact: true }) }).getByRole('link', { name: 'Explore in the studio →', exact: true }).click();
  assert.equal(new URL(page.url()).searchParams.get('preset'), presetId);
  await page.waitForFunction(id => document.querySelector('.native-options select[aria-label="Preset"]')?.value === id, presetId);
  await select('Output').selectOption('jsonc');
  assert.equal((await output()).modules.filter(m => m.keyWidth).length, count);
  await page.reload({ waitUntil: 'networkidle' });
  await page.waitForFunction(id => document.querySelector('.native-options select[aria-label="Preset"]')?.value === id, presetId);
  await select('Output').selectOption('jsonc');
  assert.equal((await output()).modules.filter(m => m.keyWidth).length, count);
 }
 await page.goto(base + '/?preset=not-a-valid-preset', { waitUntil: 'networkidle' });
 assert.equal(await select('Preset').inputValue(), 'full');
 assert.match(await page.locator('.platform-cards > a').filter({ hasText: 'Windows' }).textContent(), /extract\/run EXE.*Fastfetch separate/);
 assert.equal(await page.getByText('installer included', { exact: false }).count(), 0);
 check('Focused/Customcardsnavigateexplicitvalidatedpresetqueryandexport9/4modules; directreloadhydratescleanly, invalidpresetignored, WindowsZIPmanualrun/Fastfetchcopytruthful');
 for (const route of ['/themes', '/presets', '/downloads', '/docs', '/docs/getting-started', '/releases', '/faq']) { await page.goto(base + route, { waitUntil: 'networkidle' }); await page.getByRole('heading', { level: 1 }).waitFor(); assert.equal(new URL(await page.locator('link[rel=canonical]').getAttribute('href')).pathname, route); }
 await page.goto(base + '/themes'); assert.equal(await page.locator('.marketing-theme-card').count(), 30);
 await page.goto(base + '/downloads'); assert.equal(await page.locator('.download-card').count(), 6); assert.equal(await page.locator('tbody .pending').count(), 4);
 await page.goto(base + '/docs/getting-started'); await page.getByRole('link', { name: 'Run the native studio', exact: true }).click(); assert.equal(new URL(page.url()).hash, '#native'); const skip = page.getByRole('link', { name: 'Skip to content', exact: true }); await skip.focus(); await skip.click(); assert.equal(await page.evaluate(() => document.activeElement.id), 'main');
 check('Nohydrationmismatch on directqueryshowcases orall8SSGroutes;30themegallery, verifieddownloads/pendingmanagers, docsanchor andskiplink');

 await page.goto(base, { waitUntil: 'networkidle' }); await page.evaluate(() => document.fonts.ready); await section('Logos').click();
 for (const src of await app.locator('.native-logo-grid img').evaluateAll(images => images.map(image => image.getAttribute('src')))) assert.equal((await page.request.get(base + src)).status(), 200, src);
 for (const path of ['/fonts/termfetch-mono.woff2', '/fonts/OFL.txt', '/brand/termfetch-symbol.svg', '/brand/termfetch.ico', '/brand/apple-touch-icon.png', '/robots.txt', '/sitemap.xml']) assert.equal((await page.request.get(base + path)).status(), 200, path);
 for (const width of [320, 390, 768, 1440]) { await page.setViewportSize({ width, height: 844 }); assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), `No bodyoverflow at${width}px`); }
 await page.setViewportSize({ width: 390, height: 844 }); await section('Presets').click();
 assert.equal(await app.evaluate(el => getComputedStyle(el).fontSize), '13px'); assert.ok(await page.locator('.studio-pan').evaluate(el => el.scrollWidth > el.clientWidth), 'Nativegeometry containedpan on mobile, not microscopic stacking');
 await page.getByRole('button', { name: '⛶ Expand studio', exact: true }).click(); assert.equal(await page.locator('.studio-shell').evaluate(el => el.classList.contains('studio-expanded')), true); await page.getByRole('button', { name: '↙ Close expanded studio', exact: true }).click();
 await page.evaluate(() => { document.activeElement?.blur(); window.scrollTo(0, 0); }); await page.mouse.move(0, 0); await page.waitForTimeout(200); await page.screenshot({ path: resolve(evidence, 'landing-mobile.png'), fullPage: true });
 await page.locator('.studio-pan').evaluate(el => el.scrollLeft = el.scrollWidth); await page.locator('.studio-pan').screenshot({ path: resolve(evidence, 'studio-mobile-pan.png') });
 check('Alllocalart/font/brand assets, bodyoverflow320/390/768/1440, readablemobilefont, containedpan and expandedstudio');
 assert.deepEqual(report.errors, [], 'Zero runtime/console/network/hydration errors'); report.status = 'Pass';
} catch (error) { report.errors.push(String(error)); report.status = browser ? 'Fail' : 'Blocked'; process.exitCode = 1; }
finally { if (context) await context.tracing.stop({ path: resolve(evidence, 'studio-trace.zip') }).catch(() => {}); await browser?.close(); await new Promise(done => server ? server.close(done) : done()); report.completedAt = new Date().toISOString(); await writeFile(resolve(evidence, 'browser-verification.json'), JSON.stringify(report, null, 2) + '\n'); console.log(JSON.stringify(report, null, 2)); }
