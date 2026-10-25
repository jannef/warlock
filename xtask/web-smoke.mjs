// Dev tool for `cargo xtask web-smoke`: drive headless Chromium over CDP, load the web build,
// press Enter nine times (main menu -> New run -> race -> the stars -> gender -> class -> talents -> name ->
// summary -> start, the scenario's character preselected, the name left empty: a random one),
// check the new
// character's save is in localStorage (`warlock/saves/<id>.sav`, written as the run starts), wait
// five turns, reload the page (its pagehide hook must write the run to that save), choose Continue
// and check the run resumed at turn 5; then pause, Quit and confirm: the page must reload to the
// main menu (a web page cannot quit), where Continue resumes the run again; screenshot, and fail if
// errors were logged.
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
await key('Enter', 'Enter', 13);   // Showcase: on to the race
await key('Enter', 'Enter', 13);   // the race (the showcase's, preselected)
await key('Enter', 'Enter', 13);   // born under the stars: on to the gender
await key('Enter', 'Enter', 13);   // the gender
await key('Enter', 'Enter', 13);   // the class
await key('Enter', 'Enter', 13);   // the talents (Random, preselected)
await key('Enter', 'Enter', 13);   // the name (empty: a random one)
await key('Enter', 'Enter', 13);   // start from the summary
await sleep(4000);
// The characters' saves: [key, length] of each `warlock/saves/*.sav`, and every key.
const stored = async () => (await send('Runtime.evaluate', {
  expression: "JSON.stringify([Object.keys(localStorage).filter((k) => /^warlock\\/saves\\/[^/]+\\.sav$/.test(k)).map((k) => [k, localStorage.getItem(k).length]), Object.keys(localStorage)])",
  returnByValue: true })).result.result.value;
const before = JSON.parse(await stored());
const failures = [];
if (before[0].length !== 1) failures.push('the new character was not saved as the run started: ' + before[1]);
for (let i = 0; i < 5; i++) await key('.', 'Period', 190);   // wait: five turns
await sleep(500);
await send('Page.reload');
await sleep(12000);
const after = JSON.parse(await stored());
if (after[0].length !== 1) failures.push('expected one character save after reload; keys: ' + after[1]);
else console.log(`${after[0][0][0]} in localStorage after reload: ${after[0][0][1]} base64 chars`);
await send('Runtime.evaluate', { expression: "document.querySelector('canvas')?.focus()" });
await key('Enter', 'Enter', 13);   // Continue
await sleep(3000);
// Turn 5: the page-hide hook wrote the run (autosave wrote it at turn 0 and next at turn 50).
const resumed = (l) => /resumed \S+ at turn 5\b/.test(l);
if (!logs.some(resumed)) failures.push('Continue did not resume at turn 5; logs: ' + logs.join(' | '));
// Quit from the pause menu (Esc, Up wraps to Quit, Enter asks, y confirms): the page reloads.
await send('Runtime.evaluate', { expression: 'window.__beforeQuit = 1' });
await key('Escape', 'Escape', 27);
await key('ArrowUp', 'ArrowUp', 38);
await key('Enter', 'Enter', 13);
await key('y', 'KeyY', 89);
await sleep(12000);
const alive = (await send('Runtime.evaluate', { expression: "String(window.__beforeQuit)", returnByValue: true })).result.result.value;
if (alive !== 'undefined') failures.push('Quit did not reload the page');
await send('Runtime.evaluate', { expression: "document.querySelector('canvas')?.focus()" });
await key('Enter', 'Enter', 13);   // Continue
await sleep(3000);
if (logs.filter(resumed).length < 2) failures.push('Continue after Quit did not resume at turn 5; logs: ' + logs.join(' | '));
const shot = await send('Page.captureScreenshot', { format: 'png' });
writeFileSync(out, Buffer.from(shot.result.data, 'base64'));
const bad = [...failures, ...logs.filter((l) => /error|panic|EXCEPTION|warn/i.test(l))].slice(0, 15);
console.log(bad.join('\n') || 'no errors logged; reload and Quit both resumed the run');
ws.close(); proc.kill();
process.exit(bad.length ? 1 : 0);
