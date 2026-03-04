import fs from "node:fs";
import path from "node:path";
import zlib from "node:zlib";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const reactMDist = path.join(root, "dist");
const reactMiniDist = path.join(root, "dist-mini");
const leptosDist = path.resolve(root, "..", "example_motion_controller", "dist");

function gather(dir) {
  if (!fs.existsSync(dir)) {
    throw new Error(`Missing directory: ${dir}`);
  }

  const files = [];

  const visit = (current) => {
    const entries = fs.readdirSync(current, { withFileTypes: true });
    for (const entry of entries) {
      const fullPath = path.join(current, entry.name);
      if (entry.isDirectory()) {
        visit(fullPath);
      } else if (entry.isFile()) {
        files.push(fullPath);
      }
    }
  };

  visit(dir);
  files.sort();

  const rows = files.map((file) => {
    const data = fs.readFileSync(file);
    return {
      name: path.relative(dir, file),
      raw: data.length,
      gzip: zlib.gzipSync(data, { level: 9 }).length,
      brotli: zlib.brotliCompressSync(data, {
        params: {
          [zlib.constants.BROTLI_PARAM_QUALITY]: 11,
        },
      }).length,
    };
  });

  const total = rows.reduce(
    (acc, row) => ({
      raw: acc.raw + row.raw,
      gzip: acc.gzip + row.gzip,
      brotli: acc.brotli + row.brotli,
    }),
    { raw: 0, gzip: 0, brotli: 0 },
  );

  return { rows, total };
}

function formatBytes(bytes) {
  return `${(bytes / 1024).toFixed(2)} KB`;
}

function printBlock(label, report) {
  console.log(`\n${label}`);
  console.log("-".repeat(label.length));
  for (const row of report.rows) {
    console.log(
      `${row.name.padEnd(56)} raw=${formatBytes(row.raw).padStart(10)} gzip=${formatBytes(row.gzip).padStart(10)} brotli=${formatBytes(row.brotli).padStart(10)}`,
    );
  }
  console.log(
    `${"TOTAL".padEnd(56)} raw=${formatBytes(report.total.raw).padStart(10)} gzip=${formatBytes(report.total.gzip).padStart(10)} brotli=${formatBytes(report.total.brotli).padStart(10)}`,
  );
}

const reactMReport = gather(reactMDist);
const reactMiniReport = gather(reactMiniDist);
const leptosReport = gather(leptosDist);

printBlock("React + Motion m.* (Vite)", reactMReport);
printBlock("React + Motion useAnimate mini (Vite)", reactMiniReport);
printBlock("Leptos controller demo", leptosReport);

function printDelta(label, left, right) {
  const deltaRaw = left.total.raw - right.total.raw;
  const deltaGzip = left.total.gzip - right.total.gzip;
  const deltaBrotli = left.total.brotli - right.total.brotli;

  console.log(`\nDelta (${label})`);
  console.log("-".repeat(8 + label.length));
  console.log(`raw:    ${formatBytes(deltaRaw)} (${deltaRaw >= 0 ? "+" : ""}${deltaRaw} bytes)`);
  console.log(
    `gzip:   ${formatBytes(deltaGzip)} (${deltaGzip >= 0 ? "+" : ""}${deltaGzip} bytes)`,
  );
  console.log(
    `brotli: ${formatBytes(deltaBrotli)} (${deltaBrotli >= 0 ? "+" : ""}${deltaBrotli} bytes)`,
  );
}

printDelta("React m - Leptos", reactMReport, leptosReport);
printDelta("React mini - Leptos", reactMiniReport, leptosReport);
printDelta("React mini - React m", reactMiniReport, reactMReport);
