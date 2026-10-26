// Actual single-worker desktop QA. A launch failure is Blocked, never a headless pass.
import { createServer } from 'node:http';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { resolve, extname, sep } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { execFileSync } from 'node:child_process';
import assert from 'node:assert/strict';

const here = fileURLToPath(new URL('./', import.meta.url));
const dist = resolve(here, 'dist');
const evidence = resolve(here, 'qa');
await mkdir(evidence, { recursive: true });
const report = { status: 'Blocked', assertions: [], errors: [], headless: false, startedAt: new Date().toISOString() };
let browser, server, context;
const check = (name) => report.assertions.push(name);
const parseJSONC = (text) => JSON.parse(text.split('\n').filter(line => !line.startsWith('//')).join('\n'));
try {
  const playwrightPath = process.env.PLAYWRIGHT_MODULE || '/home/sinan/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright/index.mjs';
  const { chromium } = await import(playwrightPath.startsWith('/') ? pathToFileURL(playwrightPath).href : playwrightPath);
  server = createServer(async (req, res) => {
    const pathname = decodeURIComponent(new URL(req.url, 'http://localhost').pathname);
    let path = resolve(dist, '.' + (pathname === '/' ? '/index.html' : pathname));
    if (!path.startsWith(dist + sep)) return res.writeHead(403).end();
    try {
      let data;
      try { data = await readFile(path); }
      catch {
        if (extname(path)) return res.writeHead(404).end('Not found');
        path = resolve(dist, 'index.html'); data = await readFile(path);
      }
      const types = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css', '.svg': 'image/svg+xml', '.png': 'image/png', '.xml': 'application/xml', '.txt': 'text/plain', '.ico': 'image/x-icon' };
      res.writeHead(200, { 'Content-Type': types[extname(path)] || 'application/octet-stream' }).end(data);
    } catch { res.writeHead(500).end('Test fixture missing; run npm run build first'); }
  });
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  const base = process.env.BASE_URL || 'http://127.0.0.1:' + server.address().port;
  report.target = base;
  browser = await chromium.launch({
    headless: false,
    executablePath: process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE || '/home/sinan/.cache/ms-playwright/chromium-1234/chrome-linux64/chrome',
    args: process.platform === 'linux' ? ['--ozone-platform=x11'] : [],
  });
  context = await browser.newContext({ viewport: { width: 1440, height: 1100 }, acceptDownloads: true });
  await context.tracing.start({ screenshots: true, snapshots: true });
  const page = await context.newPage();
  page.on('pageerror', error => report.errors.push(error.message));
  page.on('console', message => { if (message.type() === 'error') report.errors.push(message.text()); });
  page.on('response', response => { if (response.status() >= 400 && response.url().startsWith(base)) report.errors.push(`${response.status()} ${response.url()}`); });
  await page.goto(base, { waitUntil: 'networkidle' });
  if (process.platform === 'linux') {
    const windows = execFileSync('xwininfo', ['-root', '-tree'], { encoding: 'utf8' });
    assert.match(windows, /Google Chrome for Testing|Chromium|chromium/i, 'A real headed Chromium window must exist');
    await writeFile(resolve(evidence, 'desktop-window.txt'), windows);
  }
  const studio = page.getByRole('region', { name: 'Interactive TermFetch Studio', exact: true });
  const section = name => page.getByRole('navigation', { name: 'Studio sections' }).getByRole('button', { name: new RegExp(name) });
  const output = async () => parseJSONC(await page.locator('.json-output').textContent());
  const showJSON = () => page.getByRole('button', { name: 'Generated JSONC', exact: true }).click();
  assert.equal(await page.getByLabel('Theme', { exact: true }).inputValue(), 'catppuccin-mocha');
  assert.equal(await page.getByLabel('Preset', { exact: true }).inputValue(), 'full');
  await page.getByLabel('Theme', { exact: true }).selectOption('nord');
  await page.getByLabel('Preset', { exact: true }).selectOption('minimal');
  await page.getByLabel('Logo', { exact: true }).selectOption('cat');
  assert.equal(await page.locator('.terminal-preview').evaluate(el => getComputedStyle(el).backgroundColor), 'rgb(46, 52, 64)');
  await showJSON();
  let config = await output();
  assert.equal(config.logo.type, 'data');
  assert.equal(config.modules.filter(m => m.keyWidth).length, 4);
  assert.ok(config.modules[2].format.includes('\x1b'), 'Title uses real ESC characters');
  assert.ok(!config.modules[2].format.includes('\\x1b'), 'No literal backslash-x escape in exported title');
  check('Rust defaults, reactive theme/preset/ASCII logo and valid ANSI JSONC');

  await page.getByLabel('Logo', { exact: true }).selectOption('pikachu');
  assert.equal((await output()).logo.type, 'auto');
  await page.getByLabel('Image path on your machine (optional)', { exact: true }).fill('/home/example/logo.png');
  await page.getByLabel('Image size', { exact: true }).selectOption('small');
  assert.equal((await output()).logo.width, 20);
  await page.getByLabel('Image size', { exact: true }).selectOption('large');
  assert.equal((await output()).logo.width, 52);
  await page.getByLabel('Image protocol', { exact: true }).selectOption('kitty-direct');
  assert.equal((await output()).logo.type, 'kitty-direct');
  await page.getByLabel('Logo', { exact: true }).selectOption('cat');
  check('Bundled image catalog, explicit image path/protocol, size and automatic fallback');

  await section('Modules').click();
  await page.getByRole('checkbox', { name: /CPU/ }).check();
  await page.getByRole('button', { name: 'Move CPU up', exact: true }).click();
  assert.deepEqual((await output()).modules.filter(m => m.keyWidth).map(m => m.type), ['os', 'kernel', 'shell', 'cpu', 'uptime']);
  await page.getByRole('checkbox', { name: /CPU/ }).uncheck();
  assert.equal((await output()).modules.filter(m => m.keyWidth).length, 4);
  await page.getByLabel('Preset', { exact: true }).selectOption('full');
  await page.getByRole('button', { name: 'Use current preset as a starting point', exact: true }).click();
  assert.equal((await output()).modules.filter(m => m.keyWidth).length, 25);
  await page.getByLabel('Preset', { exact: true }).selectOption('minimal');
  check('Module enable/disable, order and adopting current preset');

  await section('Settings').click();
  const originalOS = (await output()).modules.find(m => m.type === 'os').keyColor;
  const edit = () => page.getByRole('button', { name: 'Edit colors', exact: true }).click();
  await edit();
  await page.getByLabel('Custom hex for os', { exact: true }).fill('#ff8c00');
  assert.equal((await output()).modules.find(m => m.type === 'os').keyColor, '#ff8c00');
  assert.equal(await page.getByRole('button', { name: /Download JSONC/ }).isDisabled(), true);
  await page.getByRole('button', { name: 'Cancel colors', exact: true }).click();
  assert.equal((await output()).modules.find(m => m.type === 'os').keyColor, originalOS);
  await edit(); await page.getByLabel('Custom hex for os', { exact: true }).fill('#ff8c00');
  await page.getByRole('button', { name: 'Apply colors', exact: true }).click();
  assert.equal((await output()).modules.find(m => m.type === 'os').outputColor, '#ff8c00');
  await edit(); await page.getByRole('button', { name: 'Reset colors', exact: true }).click();
  assert.equal((await output()).modules.find(m => m.type === 'os').keyColor, originalOS);
  await page.getByRole('button', { name: 'Cancel colors', exact: true }).click();
  assert.equal((await output()).modules.find(m => m.type === 'os').keyColor, '#ff8c00');
  await edit(); await page.getByRole('button', { name: 'Reset colors', exact: true }).click();
  await page.getByRole('button', { name: 'Apply colors', exact: true }).click();
  assert.equal((await output()).modules.find(m => m.type === 'os').keyColor, originalOS);
  await page.getByRole('checkbox', { name: /Demo mode/ }).uncheck();
  assert.equal((await output()).modules.find(m => m.type === 'os').format, undefined);
  check('Semantic live edit/apply/cancel/reset, draft export guard and demo-off config');

  await page.getByRole('button', { name: /Apply locally/ }).click();
  assert.match(await page.getByRole('status').textContent(), /No host files/);
  const saved = await page.evaluate(() => localStorage.getItem('termfetch-browser-export'));
  assert.equal(JSON.parse(saved).theme, 'nord');
  await page.getByLabel('Theme', { exact: true }).selectOption('dracula');
  await page.getByRole('button', { name: 'Restore browser save', exact: true }).click();
  assert.equal(await page.getByLabel('Theme', { exact: true }).inputValue(), 'nord');
  await page.evaluate(raw => localStorage.setItem('termfetch-browser-export', JSON.stringify({ ...JSON.parse(raw), ui_follow_theme: 'invalid' })), saved);
  await page.getByRole('button', { name: 'Restore browser save', exact: true }).click();
  assert.match(await page.getByRole('status').textContent(), /invalid/);
  await page.evaluate(() => localStorage.setItem('termfetch-browser-export', '{'));
  await page.getByRole('button', { name: 'Restore browser save', exact: true }).click();
  assert.match(await page.getByRole('status').textContent(), /Could not read/);
  const downloaded = page.waitForEvent('download');
  await page.getByRole('button', { name: /Download JSONC/ }).click();
  const file = await downloaded; await file.saveAs(resolve(evidence, 'config.jsonc'));
  config = parseJSONC(await readFile(resolve(evidence, 'config.jsonc'), 'utf8'));
  assert.equal(config.modules.filter(m => m.keyWidth).length, 4);
  check('Local apply/restore, invalid and malformed storage rejection, actual JSONC download');

  await studio.focus(); await page.keyboard.press('2');
  await page.getByPlaceholder('Search palettes…').fill('aod?');
  assert.equal(await page.getByPlaceholder('Search palettes…').inputValue(), 'aod?');
  assert.equal(await page.locator('#studio-shortcuts').count(), 0);
  await studio.focus(); await page.keyboard.press('6');
  assert.equal(await page.getByRole('checkbox', { name: /Demo mode/ }).isChecked(), false);
  await page.keyboard.press('d');
  assert.equal(await page.getByRole('checkbox', { name: /Demo mode/ }).isChecked(), true);
  await page.getByRole('button', { name: 'Terminal preview', exact: true }).click();
  await studio.focus(); await page.keyboard.press('o'); assert.equal(await page.locator('.json-output').count(), 1);
  await page.keyboard.press('?'); assert.equal(await page.locator('#studio-shortcuts').count(), 1);
  await page.keyboard.press('Escape'); assert.equal(await page.locator('#studio-shortcuts').count(), 0);
  await page.keyboard.press('a'); assert.match(await page.getByRole('status').textContent(), /Applied to this browser/);
  await page.getByRole('button', { name: 'Terminal preview', exact: true }).click();
  await section('Dashboard').click();
  assert.match(await page.locator('.sidebar button.selected').innerText(), /Dashboard/);
  assert.equal(await page.locator('.editor h2').textContent(), 'Make it yours.');
  await page.evaluate(() => { document.activeElement?.blur(); window.scrollTo(0, 0); });
  await page.mouse.move(0, 0);
  await page.waitForTimeout(200);
  await page.screenshot({ path: resolve(evidence, 'studio-desktop.png'), fullPage: true });
  check('Scoped section/apply/demo/output/help shortcuts; input typing is preserved');

  const nav = page.getByRole('navigation', { name: 'Main navigation' });
  await nav.getByRole('link', { name: 'Downloads', exact: true }).click();
  assert.equal(new URL(page.url()).pathname, '/downloads');
  assert.equal(await page.locator('.download-card').count(), 6);
  assert.equal(await page.locator('tbody .pending').count(), 4);
  assert.equal(await page.locator('.download-card a[href*="/releases/download/v2.0.0/"]').count(), 6);
  assert.equal(await page.getByRole('link', { name: 'SHA256SUMS.txt ↗', exact: true }).getAttribute('href'), 'https://github.com/sinansarikaya/termfetch-studio/releases/download/v2.0.0/SHA256SUMS.txt');
  await page.reload(); await page.getByRole('heading', { name: 'Get TermFetch Studio.', exact: true }).waitFor();
  await nav.getByRole('link', { name: 'Docs', exact: true }).click();
  await page.getByRole('link', { name: 'Run the native studio', exact: true }).click();
  assert.equal(new URL(page.url()).pathname, '/docs'); assert.equal(new URL(page.url()).hash, '#native');
  assert.ok(await page.locator('#native').evaluate(el => scrollY > 0 && el.getBoundingClientRect().top >= 0 && el.getBoundingClientRect().top < innerHeight / 2), 'Anchor target is visible even when the document cannot scroll it to the exact top');
  await page.reload(); await page.locator('#native').waitFor();
  const skip = page.getByRole('link', { name: 'Skip to content', exact: true });
  await skip.focus(); await skip.click(); assert.equal(await page.evaluate(() => document.activeElement.id), 'main');
  assert.equal(new URL(page.url()).pathname, '/docs');
  await nav.getByRole('link', { name: 'Releases', exact: true }).click();
  assert.match(await page.locator('main').innerText(), /v2\.0\.0/); assert.match(await page.locator('main').innerText(), /v2\.1\.0/);
  await page.goto(base + '/docs/getting-started'); await page.getByRole('heading', { name: 'From studio to shell.', exact: true }).waitFor();
  assert.equal(await page.locator('link[rel="canonical"]').getAttribute('href'), 'https://termfetch-studio.sinansarikaya.dev/docs/getting-started');
  check('Direct/reload routes, released/source versions, six verified assets, four pending channels, Docs anchor and skip link');

  await nav.getByRole('link', { name: 'Studio', exact: true }).click();
  await section('Logos').click();
  for (const src of await page.locator('.logo-list img').evaluateAll(images => images.map(image => image.getAttribute('src')))) {
    assert.equal((await page.request.get(base + src)).status(), 200, src);
  }
  for (const asset of ['/brand/termfetch-symbol.svg', '/brand/termfetch.ico', '/brand/apple-touch-icon.png', '/brand/termfetch-512.png', '/robots.txt', '/sitemap.xml']) {
    assert.equal((await page.request.get(base + asset)).status(), 200, asset);
  }
  for (const width of [320, 390, 768, 1440]) {
    await page.setViewportSize({ width, height: 844 });
    assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), `No page overflow at ${width}px`);
  }
  await page.setViewportSize({ width: 390, height: 844 });
  await section('Themes').click(); await page.getByPlaceholder('Search palettes…').fill('no-such-theme');
  assert.match(await page.locator('.editor').innerText(), /No matching palettes/);
  await page.keyboard.press('Tab'); assert.notEqual(await page.evaluate(() => document.activeElement.tagName), 'BODY');
  assert.match(await page.locator('.sidebar button.selected').innerText(), /Themes/);
  assert.equal(await page.locator('.editor h2').textContent(), 'Themes');
  await page.evaluate(() => { document.activeElement?.blur(); window.scrollTo(0, 0); });
  await page.mouse.move(0, 0);
  await page.waitForTimeout(200);
  await page.screenshot({ path: resolve(evidence, 'studio-mobile.png'), fullPage: true });
  check('All bundled logo and shared brand assets, crawler files, 320/390/768/1440px overflow, mobile empty state and keyboard focus');
  assert.deepEqual(report.errors, [], 'No console, page or local network errors');
  report.status = 'Pass';
} catch (error) {
  report.errors.push(String(error)); report.status = browser ? 'Fail' : 'Blocked'; process.exitCode = 1;
} finally {
  if (context) await context.tracing.stop({ path: resolve(evidence, 'studio-trace.zip') }).catch(() => {});
  await browser?.close();
  await new Promise(resolve => server ? server.close(resolve) : resolve());
  report.completedAt = new Date().toISOString();
  await writeFile(resolve(evidence, 'browser-verification.json'), JSON.stringify(report, null, 2) + '\n');
  console.log(JSON.stringify(report, null, 2));
}
