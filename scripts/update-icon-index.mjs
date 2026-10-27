import { writeFile } from 'node:fs/promises';

const response = await fetch('https://api.github.com/repos/selfhst/icons/git/trees/HEAD?recursive=1');
if (!response.ok) throw new Error(`Icon index request failed: ${response.status}`);
const tree = await response.json();
if (tree.truncated || !Array.isArray(tree.tree)) throw new Error('The icon tree is incomplete.');
const slugs = tree.tree.map(entry => entry.path)
  .filter(path => /^png\/[a-z0-9][a-z0-9-]*\.png$/.test(path) && !/-(dark|light)\.png$/.test(path))
  .map(path => path.slice(4, -4)).sort();
await writeFile(new URL('../src-tauri/catalog/icon-index.json', import.meta.url), JSON.stringify(slugs, null, 2) + '\n');
console.log(`Updated ${slugs.length} icon names from selfh.st/icons. Image downloads remain lazy.`);
