import { cp, mkdir, readdir, copyFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
const source = new URL('../../assets/brand/', import.meta.url);
const destination = new URL('../public/brand/', import.meta.url);
await mkdir(destination, { recursive: true });
for (const name of await readdir(source)) {
  if (/\.(svg|png|ico|icns)$/.test(name)) await cp(new URL(name, source), new URL(name, destination));
}
// Apple accepts a PNG touch icon and scales this shared 512px source as needed.
await copyFile(new URL('termfetch-512.png', source), new URL('apple-touch-icon.png', destination));
console.log('Shared brand synchronized to ' + fileURLToPath(destination));
