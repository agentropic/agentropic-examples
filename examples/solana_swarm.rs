//! RustyAI × Solana — Autonomous Trading Swarm Demo
//!
//! 6 agents coordinate a simulated SOL/USDC trading session:
//!   • PriceOracle   — broadcasts SOL prices each round (5 ticks)
//!   • TraderAlpha   — BDI deliberation, sealed-bid auction
//!   • TraderBeta    — BDI deliberation, sealed-bid auction
//!   • TraderGamma   — BDI deliberation; crashes twice, Supervisor recovers it
//!   • RiskMonitor   — watches price trend, emits risk signal
//!   • Auctioneer    — runs a sealed-bid auction, accepts best bid
//!
//! Demonstrates: Market pattern · FIPA messaging · Supervision · BDI deliberation
//!
//! Run: cargo run --example solana_swarm

use agent_core::{Agent, AgentContext, AgentId, AgentError, AgentResult};
use patterns::market::{Auction, AuctionType, Bid};
use runtime::prelude::*;
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU32, Ordering};

// ─── Shared market state ──────────────────────────────────────────────────────

#[derive(Default)]
struct MarketState {
    sol_price:  f64,
    prev_price: f64,
    round:      u32,
    trades:     u32,
}

type Shared<T> = Arc<Mutex<T>>;

// ─── PriceOracle ─────────────────────────────────────────────────────────────

struct PriceOracle {
    id:     AgentId,
    state:  Shared<MarketState>,
    prices: Vec<f64>,
    tick:   usize,
}

impl PriceOracle {
    fn new(state: Shared<MarketState>) -> Self {
        Self {
            id: AgentId::new(),
            state,
            prices: vec![142.50, 145.20, 143.80, 148.60, 146.10],
            tick: 0,
        }
    }
}

#[async_trait]
impl Agent for PriceOracle {
    fn id(&self) -> &AgentId { &self.id }

    async fn initialize(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [PriceOracle]  Online — broadcasting SOL/USDC each round");
        Ok(())
    }

    async fn execute(&mut self, ctx: &AgentContext) -> AgentResult<()> {
        if self.tick >= self.prices.len() {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            return Ok(());
        }

        let price = self.prices[self.tick];
        let (pct, round) = {
            let mut s = self.state.lock().unwrap();
            s.prev_price = s.sol_price;
            s.sol_price  = price;
            s.round      = self.tick as u32 + 1;
            let pct = if s.prev_price == 0.0 { 0.0 }
                      else { ((price - s.prev_price) / s.prev_price) * 100.0 };
            (pct, s.round)
        };

        let sign = if pct >= 0.0 { "+" } else { "" };
        println!("\n━━━ Round {} ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━", round);
        println!("  [PriceOracle]  SOL/USDC: ${:.2}  ({}{:.2}%)", price, sign, pct);

        let msg = format!("{:.2}", price);
        ctx.send_message("trader_alpha", "inform", &msg);
        ctx.send_message("trader_beta",  "inform", &msg);
        ctx.send_message("trader_gamma", "inform", &msg);
        ctx.send_message("risk_monitor", "inform", &msg);
        ctx.send_message("auctioneer",   "inform", &msg);

        self.tick += 1;
        tokio::time::sleep(std::time::Duration::from_millis(1400)).await;
        Ok(())
    }

    async fn shutdown(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [PriceOracle]  Offline.");
        Ok(())
    }
}

// ─── TraderAgent (BDI deliberation) ──────────────────────────────────────────

struct TraderAgent {
    id:           AgentId,
    name:         String,
    budget:       f64,     // belief: available capital (USDC)
    target_price: f64,     // belief: desired buy-in price
    position:     f64,     // belief: SOL held
    trades:       u32,
    crash_count:  Arc<AtomicU32>,
    should_crash: bool,
}

impl TraderAgent {
    fn new(name: &str, budget: f64, target: f64) -> Self {
        Self {
            id: AgentId::new(),
            name: name.to_string(),
            budget,
            target_price: target,
            position: 0.0,
            trades: 0,
            crash_count: Arc::new(AtomicU32::new(0)),
            should_crash: false,
        }
    }

    fn with_crash_counter(mut self, counter: Arc<AtomicU32>) -> Self {
        self.crash_count = counter;
        self.should_crash = true;
        self
    }

    /// BDI deliberation: beliefs → desires → intention
    fn deliberate(&self, price: f64) -> (&'static str, Option<f64>) {
        let margin   = (self.target_price - price) / self.target_price;
        let has_room = self.budget > price;

        if margin > 0.005 && has_room {
            // Intention: BUY — price is below our target, bid aggressively
            let bid = price * (1.0 + margin * 0.5);
            ("BUY ", Some(bid))
        } else if self.position > 0.0 && price > self.target_price * 1.02 {
            // Intention: SELL — price rose past our target, lock profit
            ("SELL", Some(price * 0.998))
        } else {
            // Intention: HOLD — wait for a better entry
            ("HOLD", None)
        }
    }
}

#[async_trait]
impl Agent for TraderAgent {
    fn id(&self) -> &AgentId { &self.id }

    async fn initialize(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [{}]  Budget: ${:.0} USDC  Target: ${:.2}", self.name, self.budget, self.target_price);
        Ok(())
    }

    async fn execute(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        if self.should_crash {
            let n = self.crash_count.fetch_add(1, Ordering::SeqCst);
            if n < 2 {
                println!("  [{}]  ⚡ Crash #{} — Supervisor restarting...", self.name, n + 1);
                return Err(AgentError::ExecutionFailed("simulated network fault".into()));
            }
            if n == 2 {
                println!("  [{}]  ✓ Recovered after {} crash(es) — running stable", self.name, n);
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        Ok(())
    }

    async fn shutdown(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [{}]  Shutdown. Trades this session: {}", self.name, self.trades);
        Ok(())
    }

    async fn handle_message(&mut self, ctx: &AgentContext, _sender: &str,
                             _perf: &str, content: &str) -> AgentResult<()> {
        if let Ok(price) = content.parse::<f64>() {
            let (intention, bid_opt) = self.deliberate(price);
            match bid_opt {
                Some(bid_price) => {
                    println!("  [{}]  sol_price={:.2}  →  {} @ ${:.2}",
                        self.name, price, intention, bid_price);
                    ctx.send_message("auctioneer", "propose",
                        &format!("bid:{:.2}:{}", bid_price, self.name));
                    self.trades += 1;
                }
                None => {
                    println!("  [{}]  sol_price={:.2}  →  {} (target ${:.2})",
                        self.name, price, intention, self.target_price);
                }
            }
        }
        Ok(())
    }
}

// ─── RiskMonitor ─────────────────────────────────────────────────────────────

struct RiskMonitor {
    id:    AgentId,
    state: Shared<MarketState>,
}

impl RiskMonitor {
    fn new(state: Shared<MarketState>) -> Self {
        Self { id: AgentId::new(), state }
    }
}

#[async_trait]
impl Agent for RiskMonitor {
    fn id(&self) -> &AgentId { &self.id }

    async fn initialize(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [RiskMonitor]  Online — monitoring price trend");
        Ok(())
    }

    async fn execute(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        Ok(())
    }

    async fn shutdown(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [RiskMonitor]  Offline.");
        Ok(())
    }

    async fn handle_message(&mut self, _ctx: &AgentContext, _sender: &str,
                             _perf: &str, content: &str) -> AgentResult<()> {
        if content.parse::<f64>().is_ok() {
            let (price, prev) = {
                let s = self.state.lock().unwrap();
                (s.sol_price, s.prev_price)
            };
            if price > 0.0 {
                let trend = if prev == 0.0 { "NEUTRAL →" }
                else {
                    let pct = ((price - prev) / prev) * 100.0;
                    if pct > 0.5 { "BULLISH ↑" } else if pct < -0.5 { "BEARISH ↓" } else { "NEUTRAL →" }
                };
                println!("  [RiskMonitor]  trend={}", trend);
            }
        }
        Ok(())
    }
}

// ─── AuctioneerAgent ─────────────────────────────────────────────────────────

struct AuctioneerAgent {
    id:           AgentId,
    auction:      Shared<Auction>,
    state:        Shared<MarketState>,
    last_round:   u32,
    bids_this_round: Vec<(String, f64)>,
}

impl AuctioneerAgent {
    fn new(auction: Shared<Auction>, state: Shared<MarketState>) -> Self {
        Self { id: AgentId::new(), auction, state, last_round: 0, bids_this_round: vec![] }
    }

    fn resolve_round(&mut self) {
        if self.bids_this_round.is_empty() {
            println!("  [Auctioneer]   No bids this round.");
            return;
        }
        // Pick winner — highest bid wins
        self.bids_this_round.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let (winner_name, winner_price) = &self.bids_this_round[0];
        println!("  [Auctioneer]   ✓ Accept → {} wins @ ${:.2}", winner_name, winner_price);
        self.state.lock().unwrap().trades += 1;
        self.bids_this_round.clear();
    }
}

#[async_trait]
impl Agent for AuctioneerAgent {
    fn id(&self) -> &AgentId { &self.id }

    async fn initialize(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [Auctioneer]   Online — sealed-bid auction ready");
        Ok(())
    }

    async fn execute(&mut self, ctx: &AgentContext) -> AgentResult<()> {
        tokio::time::sleep(std::time::Duration::from_millis(700)).await;
        let current_round = self.state.lock().unwrap().round;
        if current_round > self.last_round && current_round > 0 {
            // Resolve previous round if there were bids, then open new CFP
            if self.last_round > 0 {
                self.resolve_round();
            }
            self.last_round = current_round;
            let price = self.state.lock().unwrap().sol_price;
            if price > 0.0 {
                println!("  [Auctioneer]   CFP → Sealed-bid: BUY 10 SOL @ market (${:.2})", price);
                ctx.send_message("trader_alpha", "cfp", "10 SOL");
                ctx.send_message("trader_beta",  "cfp", "10 SOL");
                ctx.send_message("trader_gamma", "cfp", "10 SOL");
            }
        }
        Ok(())
    }

    async fn shutdown(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        if !self.bids_this_round.is_empty() {
            self.resolve_round();
        }
        Ok(())
    }

    async fn handle_message(&mut self, _ctx: &AgentContext, _sender: &str,
                             _perf: &str, content: &str) -> AgentResult<()> {
        if content.starts_with("bid:") {
            let parts: Vec<&str> = content.splitn(3, ':').collect();
            if parts.len() == 3 {
                if let Ok(price) = parts[1].parse::<f64>() {
                    let trader = parts[2].to_string();
                    let bid = Bid::new(self.id, price, "SOL/USDC");
                    println!("  [Auctioneer]   ← Bid: ${:.2} from {}", price, trader);
                    self.auction.lock().unwrap().add_bid(bid);
                    self.bids_this_round.push((trader, price));
                }
            }
        }
        Ok(())
    }
}

// ─── Main ────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<(), RuntimeError> {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║   RustyAI × Solana — Autonomous Trading Swarm Demo     ║");
    println!("║   6 agents · BDI reasoning · Auction · Supervision      ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    let state   = Arc::new(Mutex::new(MarketState::default()));
    let auction = Arc::new(Mutex::new(
        Auction::new(AuctionType::SealedBid, "SOL/USDC").with_reserve_price(130.0)
    ));
    let crash_counter = Arc::new(AtomicU32::new(0));

    println!("  Spawning agents...\n");

    let runtime = Runtime::new();

    runtime.spawn_with_policy(
        Box::new(PriceOracle::new(state.clone())),
        "price_oracle",
        RestartPolicy::new(RestartStrategy::Always),
    ).await?;

    runtime.spawn_with_policy(
        Box::new(TraderAgent::new("TraderAlpha", 15_000.0, 144.00)),
        "trader_alpha",
        RestartPolicy::new(RestartStrategy::OnFailure),
    ).await?;

    runtime.spawn_with_policy(
        Box::new(TraderAgent::new("TraderBeta", 20_000.0, 148.00)),
        "trader_beta",
        RestartPolicy::new(RestartStrategy::OnFailure),
    ).await?;

    // TraderGamma crashes twice — demonstrates ExponentialBackoff supervision
    runtime.spawn_with_policy(
        Box::new(
            TraderAgent::new("TraderGamma", 10_000.0, 146.00)
                .with_crash_counter(crash_counter.clone())
        ),
        "trader_gamma",
        RestartPolicy::new(RestartStrategy::ExponentialBackoff)
            .with_max_retries(5)
            .with_backoff_seconds(1),
    ).await?;

    runtime.spawn_with_policy(
        Box::new(RiskMonitor::new(state.clone())),
        "risk_monitor",
        RestartPolicy::new(RestartStrategy::OnFailure),
    ).await?;

    runtime.spawn_with_policy(
        Box::new(AuctioneerAgent::new(auction.clone(), state.clone())),
        "auctioneer",
        RestartPolicy::new(RestartStrategy::Always),
    ).await?;

    println!("  ✓ 6 agents registered with Supervisor\n");

    // Run for 5 price rounds (~7s per round = ~35s total, but rounds are 1.4s each = ~10s)
    tokio::time::sleep(std::time::Duration::from_secs(12)).await;

    let crashes = crash_counter.load(Ordering::SeqCst).min(2);
    let trades  = state.lock().unwrap().trades;

    println!("\n━━━ Session Summary ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  Price rounds:    5  (SOL/USDC simulated feed)");
    println!("  Trades settled:  {}", trades);
    println!("  Crashes handled: {}  (TraderGamma — auto-recovered by Supervisor)", crashes);
    println!("  Framework:       RustyAI v0.1 · github.com/RustyAI");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    runtime.shutdown().await?;

    println!("  ✓ Demo complete.\n");
    println!("  More examples:");
    println!("    cargo run --example market_pattern   # Sealed-bid GPU cluster auction");
    println!("    cargo run --example swarm_pattern    # Swarm coordination");
    println!("    cargo run --example full_system      # Complete framework demo");
    Ok(())
}
