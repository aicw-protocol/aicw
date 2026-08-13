/**
 * Start Next.js production server on the canonical port from aicw/ports.json.
 *
 * Usage: node ../aicw/scripts/next-start.mjs aicw_home
 */
import { readFileSync } from "fs";
import { spawn } from "child_process";
import { dirname, join } from "path";
import { fileURLToPath } from "url";

const serviceKey = process.argv[2];
if (!serviceKey) {
  console.error("Usage: node next-start.mjs <service-key>");
  process.exit(1);
}

const scriptDir = dirname(fileURLToPath(import.meta.url));
const portsPath = join(scriptDir, "..", "ports.json");
const registry = JSON.parse(readFileSync(portsPath, "utf8"));
const service = registry.services?.[serviceKey];

if (!service?.port) {
  console.error(`Unknown service key "${serviceKey}" in ${portsPath}`);
  process.exit(1);
}

const port = Number(process.env.PORT || service.port);

const child = spawn("npx", ["next", "start", "-p", String(port)], {
  stdio: "inherit",
  env: process.env,
  shell: true,
});

child.on("exit", (code, signal) => {
  if (signal) process.kill(process.pid, signal);
  process.exit(code == null ? 1 : code);
});
