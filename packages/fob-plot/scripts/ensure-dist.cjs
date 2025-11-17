#!/usr/bin/env node
const { copyFileSync, mkdirSync, existsSync } = require("node:fs");
const { resolve, join } = require("node:path");

const distDir = resolve(__dirname, "..", "dist");
const source = resolve(
  __dirname,
  "..",
  "..",
  "..",
  "crates",
  "joy-cli",
  "assets",
  "plot",
  "standalone.html",
);
const target = join(distDir, "standalone.html");

if (!existsSync(source)) {
  console.warn("[joy-plot] Missing standalone template at", source);
  process.exit(0);
}

mkdirSync(distDir, { recursive: true });
copyFileSync(source, target);
console.log("[joy-plot] Synchronized standalone dashboard →", target);

