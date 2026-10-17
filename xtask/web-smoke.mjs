// Dev tool for `cargo xtask web-smoke`: drive headless Chromium over CDP, load the web build,
// press Enter twice (main menu -> New run -> start), screenshot, and fail if errors were logged.
// Usage: node xtask/web-smoke.mjs <chrome-binary> <url> <out.png> <profile-dir>
import { spawn } from 'node:child_process';
import { writeFileSync } from 'node:fs';
const [chrome, url, out, profile] = process.argv.slice(2);
const proc = spawn(chrome, ['--headless=new', `--user-data-dir=${profile}`, '--enable-unsafe-webgpu',
  '--use-angle=metal', '--window-size=1280,720', '--remote-debugging-port=9333', 'about:blank'],
  { stdio: ['ignore', 'ignore', 'pipe'] });
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
let target;
for (let i = 0; i < 50 && !target; i++) {
  await sleep(200);
  try { target = (await (await fetch('http://127.0.0.1:9333/json')).json()).find((t) => t.type === 'page'); } catch {}
}
const ws = new WebSocket(target.webSocketDebuggerUrl);
await new Promise((r) => ws.addEventListener('open', r));
let id = 0; const pending = new Map(); const logs = [];
ws.addEventListener('message', (e) => {
  const m = JSON.parse(e.data);
  if (m.id && pending.has(m.id)) { pending.get(m.id)(m); pending.delete(m.id); }
  if (m.method === 'Runtime.consoleAPICalled') logs.push(m.params.args.map((a) => a.value ?? a.description).join(' '));
  if (m.method === 'Runtime.exceptionThrown') logs.push('EXCEPTION ' + JSON.stringify(m.params.exceptionDetails.text));
});
const send = (method, params = {}) => new Promise((r) => { const i = ++id; pending.set(i, r); ws.send(JSON.stringify({ id: i, method, params })); });
await send('Runtime.enable'); await send('Page.enable');
await send('Page.navigate', { url });
await sleep(12000);
const key = async (k, code, vk) => {
  await send('Input.dispatchKeyEvent', { type: 'keyDown', key: k, code, windowsVirtualKeyCode: vk });
  await send('Input.dispatchKeyEvent', { type: 'keyUp', key: k, code, windowsVirtualKeyCode: vk });
  await sleep(600);
};
await send('Runtime.evaluate', { expression: "document.querySelector('canvas')?.focus()" });
await key('Enter', 'Enter', 13);   // New run
await key('Enter', 'Enter', 13);   // start Showcase
await sleep(4000);
const shot = await send('Page.captureScreenshot', { format: 'png' });
writeFileSync(out, Buffer.from(shot.result.data, 'base64'));
const bad = logs.filter((l) => /error|panic|EXCEPTION|warn/i.test(l)).slice(0, 15);
console.log(bad.join('\n') || 'no errors logged');
ws.close(); proc.kill();
process.exit(bad.length ? 1 : 0);
