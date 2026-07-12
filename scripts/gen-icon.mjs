// Generates the app icon (a database cylinder on a rounded dark tile)
// as a 1024x1024 PNG, without any native image dependencies.
// Usage: node scripts/gen-icon.mjs [out.png]
import zlib from "node:zlib";
import fs from "node:fs";

const OUT = process.argv[2] ?? "app-icon.png";
const SIZE = 1024;
const SS = 2; // supersampling factor for smooth edges
const W = SIZE * SS;

const buf = new Float32Array(W * W * 4);

function put(x, y, r, g, b, a) {
  const i = (y * W + x) * 4;
  // simple "source over" blend
  const da = buf[i + 3];
  const oa = a + da * (1 - a);
  if (oa <= 0) return;
  buf[i] = (r * a + buf[i] * da * (1 - a)) / oa;
  buf[i + 1] = (g * a + buf[i + 1] * da * (1 - a)) / oa;
  buf[i + 2] = (b * a + buf[i + 2] * da * (1 - a)) / oa;
  buf[i + 3] = oa;
}

const hex = (h) => [
  parseInt(h.slice(1, 3), 16) / 255,
  parseInt(h.slice(3, 5), 16) / 255,
  parseInt(h.slice(5, 7), 16) / 255,
];

// --- rounded tile with vertical gradient ---
const tileA = hex("#1b2437");
const tileB = hex("#0c111d");
const m = 64 * SS;
const R = 190 * SS;
for (let y = 0; y < W; y++) {
  const t = y / W;
  const col = tileA.map((c, i) => c + (tileB[i] - c) * t);
  for (let x = 0; x < W; x++) {
    const dx = Math.max(m + R - x, x - (W - m - R), 0);
    const dy = Math.max(m + R - y, y - (W - m - R), 0);
    const inside =
      x >= m && x < W - m && y >= m && y < W - m && dx * dx + dy * dy <= R * R;
    if (inside) put(x, y, col[0], col[1], col[2], 1);
  }
}

// --- database cylinder ---
const green = hex("#3ecf8e");
const greenDark = hex("#24996a");
const cx = W / 2;
const rx = 235 * SS;
const ry = 78 * SS;
const top = 300 * SS;
const bottom = 660 * SS;

function inEllipse(x, y, ex, ey, erx, ery) {
  const nx = (x - ex) / erx;
  const ny = (y - ey) / ery;
  return nx * nx + ny * ny <= 1;
}

for (let y = top - ry; y <= bottom + ry; y++) {
  for (let x = cx - rx; x <= cx + rx; x++) {
    const body = y >= top && y <= bottom && Math.abs(x - cx) <= rx;
    const cap = inEllipse(x, y, cx, top, rx, ry) || inEllipse(x, y, cx, bottom, rx, ry);
    if (!body && !cap) continue;
    // vertical shading
    const t = Math.min(Math.max((y - (top - ry)) / (bottom + ry - top + ry), 0), 1);
    let col = green.map((c, i) => c + (greenDark[i] - c) * t * 0.8);
    put(Math.round(x), Math.round(y), col[0], col[1], col[2], 1);
  }
}

// top cap highlight
const hi = hex("#7fe7bb");
for (let y = top - ry; y <= top + ry; y++) {
  for (let x = cx - rx; x <= cx + rx; x++) {
    if (inEllipse(x, y, cx, top, rx, ry)) {
      put(Math.round(x), Math.round(y), hi[0], hi[1], hi[2], 1);
    }
  }
}

// separation bands (darker elliptical arcs on the body)
const band = hex("#1a7a52");
for (const by of [420 * SS, 540 * SS]) {
  for (let y = by; y <= by + ry; y++) {
    for (let x = cx - rx; x <= cx + rx; x++) {
      const nx = (x - cx) / rx;
      const ny = (y - by) / ry;
      const d = nx * nx + ny * ny;
      if (d >= 0.86 && d <= 1.0) {
        put(Math.round(x), Math.round(y), band[0], band[1], band[2], 0.9);
      }
    }
  }
}

// --- downward arrow (backup to disk) in a dark circle, bottom right ---
const acx = 700 * SS;
const acy = 700 * SS;
const acr = 150 * SS;
const circle = hex("#0c111d");
for (let y = acy - acr; y <= acy + acr; y++) {
  for (let x = acx - acr; x <= acx + acr; x++) {
    if ((x - acx) ** 2 + (y - acy) ** 2 <= acr * acr) {
      put(x, y, circle[0], circle[1], circle[2], 1);
    }
  }
}
const white = hex("#eafff5");
const shaftW = 34 * SS;
for (let y = acy - 80 * SS; y <= acy + 10 * SS; y++) {
  for (let x = acx - shaftW; x <= acx + shaftW; x++) put(x, y, ...white, 1);
}
for (let y = acy + 10 * SS; y <= acy + 78 * SS; y++) {
  const half = ((acy + 78 * SS - y) / (68 * SS)) * 80 * SS;
  for (let x = acx - half; x <= acx + half; x++) put(Math.round(x), y, ...white, 1);
}

// --- downsample to SIZE and encode ---
const rgba = Buffer.alloc(SIZE * SIZE * 4);
for (let y = 0; y < SIZE; y++) {
  for (let x = 0; x < SIZE; x++) {
    let r = 0, g = 0, b = 0, a = 0;
    for (let sy = 0; sy < SS; sy++) {
      for (let sx = 0; sx < SS; sx++) {
        const i = ((y * SS + sy) * W + x * SS + sx) * 4;
        const al = buf[i + 3];
        r += buf[i] * al;
        g += buf[i + 1] * al;
        b += buf[i + 2] * al;
        a += al;
      }
    }
    const n = SS * SS;
    const o = (y * SIZE + x) * 4;
    rgba[o] = a > 0 ? Math.round((r / a) * 255) : 0;
    rgba[o + 1] = a > 0 ? Math.round((g / a) * 255) : 0;
    rgba[o + 2] = a > 0 ? Math.round((b / a) * 255) : 0;
    rgba[o + 3] = Math.round((a / n) * 255);
  }
}

// PNG encoding
const crcTable = Array.from({ length: 256 }, (_, n) => {
  let c = n;
  for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
  return c >>> 0;
});
const crc32 = (data) => {
  let c = 0xffffffff;
  for (const byte of data) c = crcTable[(c ^ byte) & 0xff] ^ (c >>> 8);
  return (c ^ 0xffffffff) >>> 0;
};
const chunk = (type, data) => {
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length);
  const typeBuf = Buffer.from(type, "ascii");
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(Buffer.concat([typeBuf, data])));
  return Buffer.concat([len, typeBuf, data, crc]);
};

const ihdr = Buffer.alloc(13);
ihdr.writeUInt32BE(SIZE, 0);
ihdr.writeUInt32BE(SIZE, 4);
ihdr[8] = 8; // bit depth
ihdr[9] = 6; // RGBA
const scanlines = Buffer.alloc(SIZE * (SIZE * 4 + 1));
for (let y = 0; y < SIZE; y++) {
  scanlines[y * (SIZE * 4 + 1)] = 0; // filter: none
  rgba.copy(scanlines, y * (SIZE * 4 + 1) + 1, y * SIZE * 4, (y + 1) * SIZE * 4);
}
const png = Buffer.concat([
  Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
  chunk("IHDR", ihdr),
  chunk("IDAT", zlib.deflateSync(scanlines, { level: 9 })),
  chunk("IEND", Buffer.alloc(0)),
]);
fs.writeFileSync(OUT, png);
console.log(`wrote ${OUT} (${png.length} bytes)`);
