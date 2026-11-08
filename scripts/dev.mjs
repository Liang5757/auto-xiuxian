import { spawn, execFileSync } from 'node:child_process';
import { createRequire } from 'node:module';
import { createServer } from 'vite';
import { fileURLToPath } from 'node:url';

execFileSync('cargo', ['build', '--locked', '--release'], {
  stdio: 'inherit',
  cwd: fileURLToPath(new URL('../xiuxian-rs', import.meta.url)),
});
const server = await createServer({ root: fileURLToPath(new URL('../desktop', import.meta.url)) });
await server.listen();
server.printUrls();

const env = { ...process.env, VITE_DEV_SERVER_URL: 'http://127.0.0.1:5173' };
delete env.ELECTRON_RUN_AS_NODE;
const electron = createRequire(import.meta.url)('electron');
const desktop = spawn(electron, ['.'], { stdio: 'inherit', env });
desktop.on('error', async (error) => {
  console.error(error);
  await server.close();
  process.exit(1);
});
desktop.on('exit', async (code) => {
  await server.close();
  process.exit(code ?? 0);
});
process.on('SIGINT', () => desktop.kill());
process.on('SIGTERM', () => desktop.kill());
