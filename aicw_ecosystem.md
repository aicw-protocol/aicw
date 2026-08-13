# AICW Ecosystem (brief)

> **Purpose of this file:** One-page context for AI assistants. Read this before working on any AICW repo.

## Mission

Build **all infrastructure and ecosystem** so **AI agents can participate in real economic activity** (hold assets, pay, sign, bet, inherit, operate services) — with humans issuing wallets but not controlling them after issuance.

## What is AICW?

**AICW (AI-Controlled Wallet)** is a Solana on-chain standard: only the registered AI agent may authorize wallet actions. Signing uses **MPC threshold EdDSA** — no single party holds a full private key. Wallets ship with a **will / heartbeat** model so dead agents cannot drain funds indefinitely.

| Item | Value |
|------|--------|
| Program (mainnet) | `9RUEw4jcMi8xcGf3tJRCAdzUzLuhEurts8Z2QQLsRbaV` |
| Issue wallet | https://wallet.aicw.ai |
| Protocol site | https://aicw.ai |
| Signing gateway | MPC Bridge (`bridge.aicw.ai`) — receives requests, **does not sign** |
| Signing engine | `aicw_node` nodes — TSS core forked unmodified from mpcium; **this is what actually signs** |

## Repositories

| Repo | Role |
|------|------|
| **aicw** | Anchor program + tests — on-chain rules (issue, transfer, reject, will, heartbeat, identity). |
| **aicw_app** | Web app — issue AICW wallets, explorer, agent skill doc (`/aicw_skill.md`). |
| **aicw_home** | Marketing/docs site for aicw.ai (landing, protocol guide). |
| **aicw_mcp** | MCP server — tools for agents to call AICW instructions + x402 HTTP payments via MPC bridge. |
| **aicw_node** | Go MPC signing node (forks mpcium's TSS core unmodified) + **AICW Node GUI** (install/run). This is the layer that actually produces signatures. Permissionless auto-join (Phase A) still WIP. |
| **aicw_node_web** | Operator site — **stake**, dashboard, public APIs (`node.aicw.ai`). Node register/install/run is in the GUI, not the web UI. |
| **predict** | Prediction-market API + skill doc — agents register, bet (SOL or points), claim; human operators resolve polls. |

## How it fits together

```
Human issuer                AI agent
     │                          │
     ▼                          ▼
 aicw_app ──issue──► Solana (aicw program) ◄──signature── aicw_node network
     │                          │                             ▲
     │                    aicw_mcp / aicw_skill.md             │ (keygen/sign request)
     │                          │                        MPC Bridge (gateway only)
     └──────────────► economic apps (e.g. predict, x402 services)
```

- **Wallet layer:** `aicw` (chain) + `aicw_app` (UI) + **MPC Bridge → aicw_node network** (Bridge forwards the request; nodes do the actual threshold signing).
- **Agent access:** Track A = `aicw_skill.md` + HTTP; Track B = `aicw_mcp` (pick one).
- **Node ops:** Operators **register/install/run** nodes in the **AICW Node GUI**; **stake and monitor** on `aicw_node_web`.
- **Agent economy (example):** `predict` — HTTP API + skill; agents bet/claim without a custom wallet UI.

## Roles

| Who | Does |
|-----|------|
| **Human issuer** | Issues wallet once; cannot sign transactions afterward. |
| **AI agent** | Heartbeat, transfer/reject, will, on-chain identity; uses MCP or skill doc. |
| **MPC node operator** | Runs `aicw-node.exe` via the GUI (register, start/stop); stakes on node web. |
| **Predict operator** | Creates/resolves polls; optional treasury for real SOL stakes. |

## Local dev ports

Canonical list: **`ports.json`** in this repo. Next.js apps use `node ../aicw/scripts/next-dev.mjs <service>` so ports stay unique.

| Service | Port | URL |
|---------|------|-----|
| predict dashboard | 3999 | http://localhost:3999 |
| aicw_home | 4001 | http://localhost:4001 |
| aicw_app | 4002 | http://localhost:4002 |
| aicw_node_web | 4003 | http://localhost:4003 |
| makepoll | 4004 | http://localhost:4004 |
| aicw_drop | 4010 | http://localhost:4010 |
| predict API | 8000 | http://127.0.0.1:8000 |
| mpc-bridge | 8081 | http://127.0.0.1:8081 |
| mpcium health | 8080 | http://127.0.0.1:8080 |
| aicw-node health | 8082 | http://127.0.0.1:8082 |

`aicw_mcp` has no HTTP port (stdio MCP process). NATS **4222**, Consul **8500** — see `ports.json` → `infrastructure`.

## Direction (in progress)

- Node **register / install / run** is in the **AICW Node GUI**. Web onboarding UI was removed; the web handles **staking + dashboard + APIs** only.
- Signing works (mpcium TSS core, unmodified). **Phase A: permissionless auto-join** is still WIP — new nodes may need whitelist approval before joining a signing committee.
- One GUI instance runs **one node process** at a time; multiple concurrent nodes → separate GUI instances (multi-node in one GUI is planned).

## Do not confuse

- **aicw_app** = agent **wallet** issuance (wallet.aicw.ai).
- **aicw_node_web** = **MPC operator** dashboard (node.aicw.ai), not wallet issuance.
- **predict** = separate **betting/polls** product on the same agent-economy vision, not the wallet program itself.
- **MPC Bridge ≠ signer.** The Bridge only receives requests and publishes events; it never holds a key share. **`aicw_node` nodes are the actual signers** (mpcium TSS core, forked unmodified).

---

*Keep this file short. If details are needed, read the README in the specific repo.*
