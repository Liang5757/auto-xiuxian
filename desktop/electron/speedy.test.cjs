const { test } = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs/promises');
const os = require('node:os');
const path = require('node:path');
const { Speedy, validFactor } = require('./speedy.cjs');

async function fixture(t) {
  const directory = await fs.mkdtemp(path.join(os.tmpdir(), 'xiuxian-speedy-'));
  for (const name of [
    'xiuxian-speed-controller32.exe',
    'xiuxian-speed-controller64.exe',
    'xiuxian-speed32.dll',
    'xiuxian-speed64.dll',
  ])
    await fs.writeFile(path.join(directory, name), '');
  const game = { id: 42, started: '100', exe: 'game.exe', arch: '64', modules: [], enabled: false };
  const commands = [];
  let games = [game];
  let failInjection = false;
  class FakeBridge {
    constructor(_directory, _arch, onExit) {
      this.onExit = onExit;
    }
    async command(command) {
      commands.push(command);
      if (command.startsWith('inject') && !failInjection) {
        game.modules = [path.join(directory, 'xiuxian-speed64.dll')];
        game.enabled = true;
        game.ready = true;
      }
    }
    close() {
      this.dead = true;
    }
  }
  const speedy = new Speedy(path.join(directory, 'settings.json'), directory, () => {}, {
    Bridge: FakeBridge,
    probe: async (target, disable) => {
      if (!target) return games;
      const matching = games.filter(
        (item) => item.id === target.id && item.started === target.started,
      );
      if (disable)
        matching.forEach((item) => {
          item.enabled = false;
        });
      return matching;
    },
  });
  t.after(async () => {
    await speedy.stop();
    // The test owns this exact temporary directory created by mkdtemp.
    await fs.rm(directory, { recursive: true, force: true });
  });
  return {
    speedy,
    game,
    commands,
    setGames: (value) => {
      games = value;
    },
    failInjection: () => {
      failInjection = true;
    },
  };
}

test('rejects invalid speed values before invoking native modules', async (t) => {
  const { speedy, commands } = await fixture(t);
  for (const value of [NaN, Infinity, 0, -1, 10.1, 1.25, '2', null]) {
    assert.equal(validFactor(value), false);
    await assert.rejects(speedy.set(true, value));
  }
  for (const value of [0.1, 0.3, 1, 2.5, 10]) assert.equal(validFactor(value), true);
  assert.deepEqual(commands, []);
});

test('requires a single game and rejects another module directory', async (t) => {
  const f = await fixture(t);
  f.setGames([]);
  await assert.rejects(f.speedy.set(true, 2), /先启动/);
  f.setGames([f.game, { ...f.game, id: 43 }]);
  await assert.rejects(f.speedy.set(true, 2), /一个游戏窗口/);
  f.setGames([f.game]);
  f.game.modules = ['C:\\other\\xiuxian-speed64.dll'];
  await assert.rejects(f.speedy.set(true, 2), /其他目录/);
  assert.deepEqual(f.commands, []);
});

test('serializes enable and stop, restores target speed, saves only preferences', async (t) => {
  const { speedy, game, commands } = await fixture(t);
  await Promise.all([speedy.set(true, 3), speedy.set(false, 3)]);
  assert.deepEqual(commands, ['inject 42', 'change 3']);
  assert.equal(game.enabled, false);
  assert.equal(speedy.state.enabled, false);
  const saved = JSON.parse(await fs.readFile(speedy.file, 'utf8'));
  assert.equal(saved.factor, 3);
  assert.equal('enabled' in saved, false);
  await speedy.load();
  assert.equal(speedy.state.enabled, false);
});

test('failed injection does not report enabled or apply multiplier', async (t) => {
  const f = await fixture(t);
  f.failInjection();
  await assert.rejects(f.speedy.set(true, 5), /注入失败/);
  assert.equal(f.speedy.state.enabled, false);
  assert.equal(f.speedy.target, null);
  assert.deepEqual(f.commands, ['inject 42']);
});

test('bridge crash restores the target speed', async (t) => {
  const f = await fixture(t);
  await f.speedy.set(true, 2);
  f.speedy.bridge.onExit(new Error('test crash'));
  await f.speedy.pending;
  assert.equal(f.game.enabled, false);
  assert.equal(f.speedy.state.enabled, false);
  assert.equal(f.speedy.state.status, 'test crash');
});

test('PID reuse does not disable a replacement process', async (t) => {
  const f = await fixture(t);
  await f.speedy.set(true, 2);
  const replacement = { ...f.game, started: '200', enabled: true };
  f.setGames([replacement]);
  await f.speedy.set(false, 2);
  assert.equal(replacement.enabled, true);
});

test('ignores legacy external directory and persists only multiplier', async (t) => {
  const { speedy } = await fixture(t);
  const bundled = speedy.directory;
  await fs.writeFile(
    speedy.file,
    JSON.stringify({ directory: 'C:\\old', factor: 4, enabled: true }),
  );
  await speedy.load();
  assert.equal(speedy.directory, bundled);
  assert.equal(speedy.state.enabled, false);
  assert.equal(speedy.state.factor, 4);
  await speedy.set(false, 4);
  assert.deepEqual(JSON.parse(await fs.readFile(speedy.file, 'utf8')), { factor: 4 });
});
