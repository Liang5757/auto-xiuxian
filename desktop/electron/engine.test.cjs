const assert = require('node:assert/strict');
const path = require('node:path');
const test = require('node:test');
const { Engine } = require('./engine.cjs');
const { validate } = require('./commands.cjs');

async function waitFor(predicate) {
  const deadline = Date.now() + 3000;
  while (!predicate()) {
    if (Date.now() > deadline) throw new Error('等待后台状态超时');
    await new Promise((resolve) => setTimeout(resolve, 10));
  }
}

test('IPC preserves valid settings and rejects invalid updates', { timeout: 5000 }, async (t) => {
  let state;
  const errors = [];
  const engine = new Engine(
    { isPackaged: false, getAppPath: () => path.resolve(__dirname, '..') },
    (next) => {
      state = next;
    },
    (error) => errors.push(error),
  );
  t.after(() => engine.stop());
  await waitFor(() => state);
  await engine.command({ type: 'configure_clicker', interval: 250, right: true, key: 118 });
  await waitFor(() => state.clicker.interval === 250);
  await assert.rejects(
    engine.command({ type: 'configure_clicker', interval: 1, right: false, key: 117 }),
  );
  assert.equal(state.clicker.interval, 250);
  assert.equal(state.clicker.right, true);
  assert.equal(state.dispatchRunning, false);
  await engine.stop();
  assert.deepEqual(errors, []);
  assert.equal(engine.pending.size, 0);
});

test(
  'missing engine reports the original error once and settles shutdown',
  { timeout: 5000 },
  async () => {
    const errors = [];
    const engine = new Engine(
      { isPackaged: false, getAppPath: () => path.join(__dirname, 'missing-application') },
      () => assert.fail('Missing engine must not publish state'),
      (error) => errors.push(error),
    );
    await engine.finished;
    assert.equal(errors.length, 1);
    assert.equal(errors[0].code, 'ENOENT');
    await assert.rejects(engine.command({ type: 'start_dispatch' }), /ENOENT/);
    await engine.stop();
  },
);

test(
  'protocol failure stops the child and does not replace the first error',
  { timeout: 5000 },
  async (t) => {
    let state;
    const errors = [];
    const engine = new Engine(
      { isPackaged: false, getAppPath: () => path.resolve(__dirname, '..') },
      (next) => {
        state = next;
      },
      (error) => errors.push(error),
    );
    t.after(() => engine.stop());
    await waitFor(() => state);
    engine.child.stdin.write('invalid-json\n');
    await engine.finished;
    assert.equal(errors.length, 1);
    assert.match(errors[0].message, /expected|invalid/i);
    assert.equal(engine.pending.size, 0);
  },
);

test('renderer commands are validated and shutdown stays internal', () => {
  assert.deepEqual(validate({ type: 'start_dispatch', extra: 'discarded' }), {
    type: 'start_dispatch',
  });
  assert.throws(() => validate({ type: 'shutdown' }));
  assert.throws(() => validate({ type: 'set_clicker', enabled: 'yes' }));
  assert.throws(() =>
    validate({ type: 'configure_clicker', interval: 100, right: false, key: 119 }),
  );
});
