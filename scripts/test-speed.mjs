import { spawn, execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { createInterface } from 'node:readline';
import { once } from 'node:events';
import assert from 'node:assert/strict';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../', import.meta.url));
const delay = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
const run = promisify(execFile);

async function verify(arch) {
  const target = spawn(path.join(root, `native/target/speed${arch}/clock-test.exe`), [], {
    windowsHide: true,
  });
  const lines = createInterface({ input: target.stdout });
  const pid = Number((await once(lines, 'line'))[0]);
  let latest = '';
  const discontinuities = [];
  lines.on('line', (line) => {
    latest = line;
    const values = line.split(' ').map(Number);
    if (values.some((value) => !Number.isFinite(value) || value < 0 || value > 1e8))
      discontinuities.push(line);
  });
  async function probe(disable = false) {
    const args = [
      '-NoProfile',
      '-NonInteractive',
      '-ExecutionPolicy',
      'Bypass',
      '-File',
      path.join(root, 'desktop/electron/speedy-probe.ps1'),
      '-TargetId',
      String(pid),
    ];
    if (disable) args.push('-Disable');
    const { stdout } = await run('powershell.exe', args, { windowsHide: true, timeout: 10000 });
    return JSON.parse(stdout.trim())[0];
  }
  const controller = spawn(
    path.join(root, `desktop/resources/native-speed/xiuxian-speed-controller${arch}.exe`),
    [],
    { windowsHide: true },
  );
  const replies = createInterface({ input: controller.stderr });
  let timeout;
  async function command(value) {
    const ack = once(replies, 'line');
    controller.stdin.write(`${value}\nassistant_marker_test\n`);
    const result = await Promise.race([
      ack,
      new Promise((_, reject) => {
        timeout = setTimeout(() => reject(new Error(`native command timeout: ${value}`)), 10000);
      }),
    ]);
    clearTimeout(timeout);
    assert.equal(result[0], 'assistant_marker_test');
  }
  async function sample(factor) {
    // Read live events, avoiding stale buffered samples while PowerShell probes run.
    await delay(1250);
    const values = latest.split(' ').map(Number);
    assert.equal(values.length, 4);
    for (const value of values)
      assert.ok(
        value > 400 * factor * 0.65 && value < 400 * factor * 1.4,
        `${arch}: ${values}, factor ${factor}`,
      );
    console.log(`${arch}-bit factor ${factor}: ${values.join(', ')} ms`);
  }
  try {
    await command(`inject ${pid}`);
    assert.equal((await probe()).ready, true);
    assert.equal((await probe()).enabled, false);
    for (const factor of [1, 3, 0.5, 10, 1]) {
      await command(`change ${factor}`);
      await sample(factor);
    }
    // Abrupt controller death must restore the rate without another IPC command.
    await command('change 3');
    assert.equal((await probe()).enabled, true);
    assert.equal((await probe(true)).enabled, false);
    await sample(1);
    await command('change 3');
    await sample(3);
    controller.kill();
    await sample(1);
    assert.deepEqual(discontinuities, [], 'clock jumps during multiplier changes or shutdown');
  } finally {
    clearTimeout(timeout);
    controller.kill();
    target.kill();
    lines.close();
    replies.close();
  }
}
for (const arch of ['32', '64']) await verify(arch);
