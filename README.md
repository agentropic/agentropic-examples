# RustyAI Examples

Working examples for the [RustyAI](https://rustyai.xyz) multi-agent framework in Rust.
Clone this repo and run any example with a single command — no config, no API keys required.

```bash
git clone https://github.com/RustyAI/examples
cd examples
cargo run --example hello_agent
```

---

## Quickstart demos

### Autonomous trading swarm on Solana
6 agents coordinate a SOL/USDC trading session. TraderGamma crashes twice — the Supervisor restarts it automatically.

```bash
cargo run --example solana_swarm
```

```
╔══════════════════════════════════════════════════╗
║  RustyAI × Solana - Autonomous Trading Swarm  ║
║  6 agents · BDI reasoning · Supervision         ║
╚══════════════════════════════════════════════════╝

  [PriceOracle]  SOL/USDC: $142.50
  [TraderGamma]  ⚡ Crash #1 - Supervisor restarting...
  [TraderGamma]  ⚡ Crash #2 - Supervisor restarting...
  [TraderAlpha]  sol_price=142.50  →  BUY  @ $143.24
  [TraderBeta]   sol_price=142.50  →  BUY  @ $145.15
  [Auctioneer]   ✓ Accept → TraderBeta wins @ $145.15
  [TraderGamma]  ✓ Recovered after 2 crash(es) - running stable

━━━ Session Summary ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Trades settled:  3
  Crashes handled: 2  (auto-recovered by Supervisor)
  Framework:       RustyAI v0.1
```

---

### Self-healing agent (Supervisor + restart policies)
An agent crashes 3 times. The Supervisor catches every failure and restarts it automatically.

```bash
cargo run --example supervised_agents
```

```
=== Supervised Agent ===

  [Flaky] Initialized (attempt 1)
  [Flaky] Crashing! (execution #1)
  [Flaky] Initialized (attempt 2)
  [Flaky] Crashing! (execution #2)
  [Flaky] Initialized (attempt 3)
  [Flaky] Crashing! (execution #3)
  [Flaky] Initialized (attempt 4)

  Total executions: 20 (3 crashes + 17 successful)
```

---

### Swarm consensus (decentralized voting)
5 agents vote on a route with no central leader. The swarm reaches consensus on its own.

```bash
cargo run --example swarm_pattern
```

```
  [Scout-1]  Voted: Route A
  [Scout-2]  Voted: Route B
  [Scout-3]  Voted: Route A
  [Scout-4]  Voted: Route A
  [Scout-5]  Voted: Route B

  [Tally] Consensus Results:
    Route A ███ (3)
    Route B ██  (2)
    → Swarm decision: Route A
```

---

### Hello Agent (start here)
The simplest possible agent — spawn it, watch it tick, shut it down.

```bash
cargo run --example hello_agent
```

```
  [Greeter] Hello! I'm alive.
  [Greeter] Tick #1
  [Greeter] Tick #2
  [Greeter] Goodbye after 6 ticks!
```

---

## All examples

| Example | What it demonstrates |
|---------|----------------------|
| `hello_agent` | Agent lifecycle: initialize → execute → shutdown |
| `messaging` | Two agents sending typed messages through the Router |
| `agents_talking` | Three agents: asker, responder, observer |
| `supervised_agents` | Crash recovery with Supervisor restart policies |
| `cognitive_agent` | BDI reasoning from BeliefBase with LLM fallback |
| `solana_swarm` | 6-agent Solana trading swarm with auction coordination |
| `full_system` | All five crates working together in one demo |

### Organizational patterns

| Pattern | What it demonstrates |
|---------|----------------------|
| `team_pattern` | Leader assigns tasks, Executors work, Coordinator tracks |
| `hierarchy_pattern` | Commander → Captain → Soldiers, orders down, reports up |
| `swarm_pattern` | Decentralized consensus voting, no central leader |
| `coalition_pattern` | Temporary alliance for a mission, then disband |
| `market_pattern` | Sealed-bid auction, highest bidder wins GPU cluster slot |
| `federation_pattern` | Weighted voting on governance proposals |
| `holarchy_pattern` | Nested autonomous units that delegate down |
| `blackboard_pattern` | Shared knowledge space for collaborative problem solving |

---

## Run any example

```bash
cargo run --example <name>
```

No setup required for most examples. The `cognitive_agent` and `full_system` examples use `data/beliefs.json` (included) and optionally an LLM — they work offline with just the belief base.

---

## License

MIT OR Apache-2.0
