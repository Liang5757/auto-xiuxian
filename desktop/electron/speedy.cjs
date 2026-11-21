const fs = require('node:fs/promises');
const path = require('node:path');
const { spawn, execFile } = require('node:child_process');
const { promisify } = require('node:util');
const { randomUUID } = require('node:crypto');
const run = promisify(execFile);

async function probe(target, disable = false) {
  const args = [
    '-NoProfile',
    '-NonInteractive',
    '-ExecutionPolicy',
    'Bypass',
    '-File',
    path.join(__dirname, 'speedy-probe.ps1').replace(/app\.asar([\\/])/, 'app.asar.unpacked$1'),
  ];
  if (target) args.push('-TargetId', String(target.id), '-Started', target.started);
  if (disable) args.push('-Disable');
  try {
    const { stdout } = await run('powershell.exe', args, {
      windowsHide: true,
      timeout: 10000,
      maxBuffer: 256 * 1024,
    });
    return JSON.parse(stdout.replace(/^\uFEFF/, '').trim());
  } catch (error) {
    throw new Error(error.stderr?.trim().split(/\r?\n/)[0] || '读取游戏变速状态失败，请确认权限');
  }
}

class Bridge {
  constructor(directory, arch, onExit) {
    this.pending = null;
    this.child = spawn(path.join(directory, `xiuxian-speed-controller${arch}.exe`), [], {
      cwd: directory,
      windowsHide: true,
      stdio: ['pipe', 'ignore', 'pipe'],
    });
    let output = '';
    this.child.stderr.on('data', (data) => {
      output = (output + data.toString()).slice(-8192);
      if (this.pending && output.includes(this.pending.marker)) {
        const pending = this.pending;
        this.pending = null;
        clearTimeout(pending.timer);
        pending.resolve();
      }
    });
    const fail = (error) => {
      if (this.dead) return;
      this.dead = true;
      if (this.pending) {
        clearTimeout(this.pending.timer);
        this.pending.reject(error);
        this.pending = null;
      }
      if (!this.closing) onExit(error);
    };
    this.child.on('error', fail);
    this.child.on('exit', () => fail(new Error('变速模块已退出')));
    this.child.stdin.on('error', () => {});
  }
  command(command) {
    if (this.dead) return Promise.reject(new Error('变速模块不可用'));
    return new Promise((resolve, reject) => {
      const marker = `assistant_marker_${randomUUID()}`;
      const timer = setTimeout(() => {
        this.pending = null;
        this.child.kill();
        reject(new Error('变速模块响应超时'));
      }, 10000);
      this.pending = { marker, resolve, reject, timer };
      this.child.stdin.write(`${command}\n${marker}\n`);
    });
  }
  close() {
    this.closing = true;
    if (!this.dead) this.child.stdin.end('exit\n');
  }
}

class Speedy {
  constructor(file, defaultDirectory, publish, dependencies = {}) {
    this.file = file;
    this.publish = publish;
    this.probe = dependencies.probe || probe;
    this.Bridge = dependencies.Bridge || Bridge;
    this.directory = defaultDirectory;
    this.state = { factor: 2, enabled: false, status: '已关闭' };
    this.pending = Promise.resolve();
  }
  update(change) {
    Object.assign(this.state, change);
    this.publish({ ...this.state });
  }
  async load() {
    try {
      const saved = JSON.parse(await fs.readFile(this.file, 'utf8'));
      if (saved && typeof saved === 'object') {
        if (validFactor(saved.factor)) this.state.factor = saved.factor;
      }
    } catch (error) {
      if (error.code !== 'ENOENT' && !(error instanceof SyntaxError)) throw error;
    }
    this.update({});
    this.timer = setInterval(() => {
      if (!this.target || this.polling) return;
      this.polling = true;
      void this.enqueue(async () => {
        if (!this.target) return;
        const [game] = await this.probe(this.target);
        if (!game || !game.enabled) {
          this.target = null;
          this.update({
            enabled: false,
            status: game ? '游戏变速已关闭' : '游戏已退出，请重新开启变速',
          });
        }
      })
        .catch((error) => this.update({ status: error.message }))
        .finally(() => {
          this.polling = false;
        });
    }, 3000);
  }
  enqueue(action) {
    const result = this.pending.then(action);
    this.pending = result.catch(() => {});
    return result;
  }
  async save() {
    const temporary = this.file + '.tmp';
    await fs.writeFile(temporary, JSON.stringify({ factor: this.state.factor }));
    await fs.rename(temporary, this.file);
  }
  async validateDirectory(directory) {
    if (typeof directory !== 'string' || !path.isAbsolute(directory))
      throw new Error('内置变速模块路径无效，请重新安装助手');
    for (const arch of ['32', '64']) {
      for (const name of [`xiuxian-speed-controller${arch}.exe`, `xiuxian-speed${arch}.dll`])
        await fs.access(path.join(directory, name)).catch(() => {
          throw new Error('内置变速模块缺失，请重新安装助手');
        });
    }
  }
  set(enabled, factor) {
    return this.enqueue(async () => {
      if (typeof enabled !== 'boolean' || !validFactor(factor))
        throw new Error('倍率须为 0.1～10，最多一位小数');
      if (!enabled) await this.disable();
      else {
        const games = await this.probe();
        if (games.length !== 1)
          throw new Error(games.length ? '请只保留一个游戏窗口' : '请先启动懒人修仙传');
        const game = games[0];
        if (this.target && (game.id !== this.target.id || game.started !== this.target.started))
          await this.disable();
        await this.enable(game, factor);
      }
      this.update({ factor });
      await this.save();
      return { ...this.state };
    });
  }
  async enable(game, factor) {
    await this.validateDirectory(this.directory);
    const arch = game.arch;
    if (!['32', '64'].includes(arch)) throw new Error('无法识别游戏进程位数');
    const dll = path.join(this.directory, `xiuxian-speed${arch}.dll`);
    if (game.modules.some((name) => name.toLowerCase() !== dll.toLowerCase()))
      throw new Error('游戏已载入其他目录的变速模块，请重启游戏');
    if (!this.bridge || this.bridge.dead || this.arch !== arch) {
      this.bridge?.close();
      const bridge = new this.Bridge(this.directory, arch, (error) => {
        if (this.bridge !== bridge) return;
        void this.enqueue(async () => {
          await this.disable();
          this.update({ status: error.message });
        }).catch((failure) => this.update({ status: failure.message }));
      });
      this.bridge = bridge;
      this.arch = arch;
    }
    this.target = game;
    try {
      // The controller acknowledges only after native initialization completes.
      await this.bridge.command(`inject ${game.id}`);
      const [injected] = await this.probe(game);
      if (
        !injected?.ready ||
        !injected.modules.some((name) => name.toLowerCase() === dll.toLowerCase())
      )
        throw new Error('变速注入失败，请确认助手与游戏权限一致');
      await this.bridge.command(`change ${factor}`);
      this.update({ enabled: true, status: `已开启 · ${factor} 倍速` });
    } catch (error) {
      await this.disable();
      throw error;
    }
  }
  async disable() {
    if (this.target) {
      const [game] = await this.probe(this.target, true);
      if (game?.enabled) throw new Error('未能恢复原速，请重试或重启游戏');
    }
    this.target = null;
    this.update({ enabled: false, status: '已关闭 · 游戏原速' });
  }
  async stop() {
    clearInterval(this.timer);
    await this.enqueue(() => this.disable());
    this.bridge?.close();
  }
}
function validFactor(value) {
  return (
    typeof value === 'number' &&
    Number.isFinite(value) &&
    value >= 0.1 &&
    value <= 10 &&
    Math.abs(value * 10 - Math.round(value * 10)) < 1e-8
  );
}
module.exports = { Speedy, validFactor, probe };
