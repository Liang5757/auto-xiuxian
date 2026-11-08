import { createHash } from 'node:crypto';
import { createReadStream } from 'node:fs';
import { copyFile, mkdir, readFile, writeFile, appendFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('..', import.meta.url));
const rootPackage = JSON.parse(await readFile(path.join(root, 'package.json'), 'utf8'));
const desktopPackage = JSON.parse(await readFile(path.join(root, 'desktop/package.json'), 'utf8'));
const cargo = await readFile(path.join(root, 'xiuxian-rs/Cargo.toml'), 'utf8');
const rustVersion = cargo.match(/^version\s*=\s*"([^"]+)"/m)?.[1];
const [tag = 'v' + rootPackage.version, flag] = process.argv.slice(2);
const version = tag.slice(1);

if (!/^v\d+\.\d+\.\d+$/.test(tag) || (flag && flag !== '--check')) {
  throw new Error('使用 npm run release:prepare -- v0.2.0 [--check]');
}
if ([rootPackage.version, desktopPackage.version, rustVersion].some((value) => value !== version)) {
  throw new Error('标签、根目录 package.json、desktop/package.json 和 Rust workspace 版本必须一致');
}

const changelog = await readFile(path.join(root, 'CHANGELOG.md'), 'utf8');
const lines = changelog.split(/\r?\n/);
const start = lines.findIndex((line) => line.startsWith('## [' + version + '] - '));
if (start < 0 || !/ - \d{4}-\d{2}-\d{2}$/.test(lines[start])) {
  throw new Error('CHANGELOG.md 缺少当前版本及发布日期');
}
let end = start + 1;
while (end < lines.length && !lines[end].startsWith('## ') && !lines[end].startsWith('[未发布]:'))
  end++;
const section = lines
  .slice(start + 1, end)
  .join('\n')
  .trim();
if (!section) throw new Error('当前版本更新记录为空');

async function checksum(file) {
  const hash = createHash('sha256');
  for await (const chunk of createReadStream(file)) hash.update(chunk);
  return hash.digest('hex');
}

async function prepare() {
  const directory = path.join(root, 'release/github');
  const name = 'xiuxian-assistant-' + tag + '-windows-x64.exe';
  const asset = path.join(directory, name);
  const sums = path.join(directory, 'SHA256SUMS.txt');
  const notes = path.join(directory, 'release-notes.md');
  await mkdir(directory, { recursive: true });
  await copyFile(path.join(root, 'release/修仙助手.exe'), asset);
  const sha256 = await checksum(asset);
  await writeFile(sums, sha256 + '  ' + name + '\n');
  await writeFile(
    notes,
    section +
      '\n\n下载 EXE 即可运行，文件适用于 Windows x64。SHA256SUMS.txt 用于核验下载完整性。\n',
  );
  if (process.env.GITHUB_OUTPUT) {
    await appendFile(
      process.env.GITHUB_OUTPUT,
      ['tag=' + tag, 'asset=' + asset, 'sums=' + sums, 'notes=' + notes].join('\n') + '\n',
    );
  }
  console.log(JSON.stringify({ tag, asset, sums, notes, sha256 }, null, 2));
}

if (flag === '--check') console.log('版本与更新记录校验通过 ' + tag);
else await prepare();
