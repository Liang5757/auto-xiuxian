const fs = require('node:fs/promises');
const { validate } = require('./commands.cjs');
class Settings {
  constructor(file) {
    this.file = file;
    this.commands = [];
    this.pending = Promise.resolve();
  }
  async load() {
    let data;
    try {
      data = JSON.parse(await fs.readFile(this.file, 'utf8'));
    } catch (error) {
      if (error.code === 'ENOENT' || error instanceof SyntaxError) return;
      throw error;
    }
    if (!Array.isArray(data)) return;
    for (const raw of data) {
      try {
        const command = validate(raw);
        if (['configure_polling', 'configure_clicker'].includes(command.type))
          this.remember(command);
      } catch {
        /* Ignore invalid stored settings and keep the default for that field. */
      }
    }
  }
  remember(command) {
    this.commands = this.commands.filter(
      (old) =>
        old.type !== command.type ||
        (command.type === 'configure_polling' && old.farming !== command.farming),
    );
    this.commands.push(command);
  }
  apply(engine, command) {
    const result = this.pending.then(async () => {
      await engine.command(command);
      if (!['configure_polling', 'configure_clicker'].includes(command.type)) return;
      this.remember(command);
      const temporary = this.file + '.tmp';
      await fs.writeFile(temporary, JSON.stringify(this.commands, null, 2), 'utf8');
      await fs.rename(temporary, this.file);
    });
    this.pending = result.catch(() => {});
    return result;
  }
}
module.exports = { Settings };
