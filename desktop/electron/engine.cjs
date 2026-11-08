const { spawn } = require('node:child_process');
const { createInterface } = require('node:readline');
const path = require('node:path');

class Engine {
  constructor(app, onState, onError) {
    this.pending = new Map();
    this.nextId = 0;
    this.onState = onState;
    this.onError = onError;
    this.failure = null;
    this.stopping = false;
    const binary = app.isPackaged
      ? path.join(process.resourcesPath, 'engine.exe')
      : path.join(app.getAppPath(), '../xiuxian-rs/target/release/xiuxian.exe');
    this.child = spawn(binary, [], { windowsHide: true, stdio: ['pipe', 'pipe', 'pipe'] });
    this.finished = new Promise((resolve) => {
      this.resolveFinished = resolve;
    });
    this.child.on('error', (error) => this.fail(error));
    this.child.on('close', (code) => {
      this.resolveFinished();
      this.rejectPending(new Error('自动化进程已退出'));
      if (!this.stopping) this.fail(new Error('自动化进程意外退出，代码 ' + code));
    });
    this.child.stdin.on('error', (error) => {
      if (!this.stopping) this.fail(error);
    });
    this.child.stderr.on('data', (data) => console.error(String(data)));
    createInterface({ input: this.child.stdout }).on('line', (line) => this.receive(line));
  }

  receive(line) {
    if (this.failure) return;
    try {
      const message = JSON.parse(line);
      switch (message.type) {
        case 'state':
          this.onState(message.state);
          break;
        case 'response': {
          const request = this.pending.get(message.id);
          if (!request) return;
          clearTimeout(request.timer);
          this.pending.delete(message.id);
          if (message.ok) request.resolve();
          else request.reject(new Error(message.error || '操作失败'));
          break;
        }
        case 'protocol_error':
          this.fail(new Error(message.error));
          break;
      }
    } catch (error) {
      this.fail(error);
    }
  }

  command(command) {
    if (this.failure) return Promise.reject(this.failure);
    if (this.stopping && command.type !== 'shutdown') return Promise.reject(new Error('正在退出'));
    if (this.child.exitCode !== null || !this.child.stdin.writable)
      return Promise.reject(new Error('自动化进程不可用'));
    const id = ++this.nextId;
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        this.pending.delete(id);
        reject(new Error('自动化进程响应超时'));
      }, 5000);
      this.pending.set(id, { resolve, reject, timer });
      this.child.stdin.write(JSON.stringify({ id, command }) + '\n', (error) => {
        if (error) this.fail(error);
      });
    });
  }

  rejectPending(error) {
    for (const request of this.pending.values()) {
      clearTimeout(request.timer);
      request.reject(error);
    }
    this.pending.clear();
  }

  fail(error) {
    if (this.failure) return;
    this.failure = error;
    this.rejectPending(error);
    this.onError(error);
    // Stop accepting work when communication fails.
    this.child.stdin.end();
  }

  async stop() {
    if (this.stopping) return this.finished;
    this.stopping = true;
    try {
      await this.command({ type: 'shutdown' });
    } catch {
      this.child.stdin.end();
    }
    await this.finished;
  }
}
module.exports = { Engine };
