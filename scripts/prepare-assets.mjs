// Prépare les images dérivées de Magic.ico avant chaque build.
// - src-tauri/icons/icon.ico : icône de l'exécutable Windows
// - src-tauri/icons/icon.png : icône de fenêtre (Linux, développement)
// - public/panda.png        : image du panda utilisée dans l'interface
// Aucune dépendance : on décode le bitmap 32 bits de l'ICO et on écrit un PNG avec zlib.
import { readFileSync, writeFileSync, mkdirSync, copyFileSync } from 'node:fs';
import { deflateSync } from 'node:zlib';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const icoPath = join(root, 'Magic.ico');
const ico = readFileSync(icoPath);

function largestEntry(buf) {
  const count = buf.readUInt16LE(4);
  let best = null;
  for (let i = 0; i < count; i++) {
    const o = 6 + i * 16;
    const w = buf[o] || 256;
    const size = buf.readUInt32LE(o + 8);
    const offset = buf.readUInt32LE(o + 12);
    if (!best || w > best.w) best = { w, size, offset };
  }
  return buf.subarray(best.offset, best.offset + best.size);
}

function dibToRgba(dib) {
  if (dib.subarray(0, 8).equals(Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]))) {
    return { png: dib };
  }
  const headerSize = dib.readUInt32LE(0);
  const width = dib.readInt32LE(4);
  const height = Math.abs(dib.readInt32LE(8)) / 2; // XOR + masque AND
  const bpp = dib.readUInt16LE(14);
  if (bpp !== 32) throw new Error(`Icône ${bpp} bits non prise en charge (32 bits attendus)`);
  const rgba = Buffer.alloc(width * height * 4);
  const rowSize = width * 4;
  for (let y = 0; y < height; y++) {
    const src = headerSize + (height - 1 - y) * rowSize; // lignes stockées de bas en haut
    for (let x = 0; x < width; x++) {
      const s = src + x * 4;
      const d = (y * width + x) * 4;
      rgba[d] = dib[s + 2];
      rgba[d + 1] = dib[s + 1];
      rgba[d + 2] = dib[s];
      rgba[d + 3] = dib[s + 3];
    }
  }
  return { width, height, rgba };
}

const CRC_TABLE = (() => {
  const table = new Uint32Array(256);
  for (let n = 0; n < 256; n++) {
    let c = n;
    for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    table[n] = c >>> 0;
  }
  return table;
})();

function crc32(buf) {
  let c = 0xffffffff;
  for (const b of buf) c = CRC_TABLE[(c ^ b) & 0xff] ^ (c >>> 8);
  return (c ^ 0xffffffff) >>> 0;
}

function chunk(type, data) {
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length);
  const body = Buffer.concat([Buffer.from(type, 'ascii'), data]);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(body));
  return Buffer.concat([len, body, crc]);
}

function encodePng({ width, height, rgba }) {
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(width, 0);
  ihdr.writeUInt32BE(height, 4);
  ihdr[8] = 8; // profondeur
  ihdr[9] = 6; // RGBA
  const raw = Buffer.alloc((width * 4 + 1) * height);
  for (let y = 0; y < height; y++) {
    raw[y * (width * 4 + 1)] = 0;
    rgba.copy(raw, y * (width * 4 + 1) + 1, y * width * 4, (y + 1) * width * 4);
  }
  return Buffer.concat([
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
    chunk('IHDR', ihdr),
    chunk('IDAT', deflateSync(raw, { level: 9 })),
    chunk('IEND', Buffer.alloc(0)),
  ]);
}

const decoded = dibToRgba(largestEntry(ico));
const png = decoded.png ?? encodePng(decoded);

for (const dir of ['public', 'src-tauri/icons']) mkdirSync(join(root, dir), { recursive: true });
writeFileSync(join(root, 'public/panda.png'), png);
writeFileSync(join(root, 'src-tauri/icons/icon.png'), png);
copyFileSync(icoPath, join(root, 'src-tauri/icons/icon.ico'));
console.log('Icônes prêtes (panda.png, icon.png, icon.ico).');
