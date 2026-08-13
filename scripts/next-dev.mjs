/**
 * Start Next.js dev server on the canonical port from aicw/ports.json.
 *
 * Usage (from any sibling repo): node ../aicw/scripts/next-dev.mjs aicw_home
 * Override: PORT=4011 node ../aicw/scripts/next-dev.mjs aicw_home
 */
import { readFileSync } from "fs";
import net from "net";
import { spawn } from "child_process";
import { dirname, join } from "path";
import { fileURLToPath } from "url";

const serviceKey = process.argv[2];
if (!serviceKey) {
  console.error("Usage: node next-dev.mjs <service-key>");
  console.error("Example: node next-dev.mjs aicw_home");
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

function isPortInUse(targetPort, host = "127.0.0.1") {
  return new Promise((resolve) => {
    const socket = new net.Socket();
    let settled = false;

    socket.setTimeout(500);
    socket.once("connect", () => {
      settled = true;
      socket.destroy();
      resolve(true);
    });
    socket.once("timeout", () => {
      if (!settled) {
        settled = true;
        socket.destroy();
        resolve(false);
      }
    });
    socket.once("error", () => {
      if (!settled) {
        settled = true;
        resolve(false);
      }
    });
    socket.connect(targetPort, host);
  });
}

const busy = await isPortInUse(port);
if (busy) {
  console.error(
    `[${serviceKey}] Port ${port} is already in use (${service.url}).`,
  );
  console.error("Stop the existing process or set PORT to a free port.");
  console.error(`Windows: netstat -ano | findstr :${port}`);
  process.exit(1);
}

console.log(`[${serviceKey}] Starting Next.js on ${service.url}`);
const child = spawn("npx", ["next", "dev", "-p", String(port)], {
  stdio: "inherit",
  env: process.env,
  shell: true,
});

child.on("exit", (code, signal) => {
  if (signal) process.kill(process.pid, signal);
  process.exit(code == null ? 1 : code);
});
