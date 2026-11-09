const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs/promises');
const os = require('node:os');
const path = require('node:path');
const { Settings } = require('./settings.cjs');
test('configuration updates survive restart and concurrent changes preserve each task', async (t) => {
  const directory = await fs.mkdtemp(path.join(os.tmpdir(), 'xiuxian-settings-'));
  t.after(() => fs.rm(directory, { recursive: true, force: true }));
  const file = path.join(directory, 'settings.json');
  const settings = new Settings(file);
  await settings.load();
  const engine = { command: async () => {} };
  await Promise.all([
    settings.apply(engine, { type: 'configure_polling', farming: false, interval: 5 }),
    settings.apply(engine, { type: 'configure_polling', farming: true, interval: 60 }),
    settings.apply(engine, { type: 'configure_clicker', interval: 250, right: true, key: 118 }),
    settings.apply(engine, { type: 'configure_polling', farming: false, interval: 10 }),
  ]);
  const restored = new Settings(file);
  await restored.load();
  assert.equal(restored.commands.length, 3);
  assert.deepEqual(
    restored.commands.find((c) => c.type === 'configure_polling' && !c.farming),
    { type: 'configure_polling', farming: false, interval: 10 },
  );
  assert.deepEqual(
    restored.commands.find((c) => c.type === 'configure_clicker'),
    { type: 'configure_clicker', interval: 250, right: true, key: 118 },
  );
  await assert.rejects(
    settings.apply(
      {
        command: async () => {
          throw Error('正在连点');
        },
      },
      { type: 'configure_clicker', interval: 300, right: false, key: 117 },
    ),
  );
  const unchanged = new Settings(file);
  await unchanged.load();
  assert.deepEqual(unchanged.commands, restored.commands);
});
