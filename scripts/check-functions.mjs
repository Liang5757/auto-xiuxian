import fs from 'node:fs';
import path from 'node:path';
import { parsers } from 'prettier/plugins/typescript';

const limit = 200;
let maximum = { lines: 0, file: '', line: 0 };
const failures = [];
const functionTypes = new Set([
  'FunctionDeclaration',
  'FunctionExpression',
  'ArrowFunctionExpression',
]);

async function checkFile(file) {
  const ast = await parsers.typescript.parse(fs.readFileSync(file, 'utf8'), { filepath: file });
  function visit(node) {
    if (!node || typeof node !== 'object' || typeof node.type !== 'string') return;
    if (functionTypes.has(node.type) && node.body) {
      const entry = {
        file,
        line: node.loc.start.line,
        lines: node.loc.end.line - node.loc.start.line + 1,
      };
      if (entry.lines > maximum.lines) maximum = entry;
      if (entry.lines > limit) failures.push(entry);
    }
    for (const value of Object.values(node)) {
      if (Array.isArray(value)) value.forEach(visit);
      else visit(value);
    }
  }
  visit(ast);
}

async function walk(directory) {
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    const file = path.join(directory, entry.name);
    if (entry.isDirectory()) await walk(file);
    else if (/\.(?:tsx?|cjs|mjs)$/.test(file)) await checkFile(file);
  }
}

for (const directory of ['desktop/src', 'desktop/electron', 'scripts']) await walk(directory);
for (const entry of failures) console.error(entry);
console.log('Largest JavaScript/TypeScript function:', maximum);
if (failures.length) process.exitCode = 1;
