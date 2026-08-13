/**
 * Load aicw/ports.json for dev scripts (predict, drop, etc.).
 */
const { readFileSync } = require("fs");
const { join } = require("path");

const portsPath = join(__dirname, "..", "ports.json");

function loadPorts() {
  return JSON.parse(readFileSync(portsPath, "utf8"));
}

function getServicePort(serviceKey, fallback) {
  const registry = loadPorts();
  return registry.services?.[serviceKey]?.port ?? fallback;
}

function getServiceUrl(serviceKey, fallback) {
  const registry = loadPorts();
  return registry.services?.[serviceKey]?.url ?? fallback;
}

module.exports = { loadPorts, getServicePort, getServiceUrl };
