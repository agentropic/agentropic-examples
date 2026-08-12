//! Market: Agents bid in an auction. Highest bidder wins the resource.
use agent_core::{Agent, AgentContext, AgentId, AgentResult};
use patterns::market::{Auction, AuctionType, Bid};
use runtime::prelude::*;
use async_trait::async_trait;
use std::sync::{Arc, Mutex};

struct AuctioneerAgent {
    id: AgentId,
    auction: Arc<Mutex<Auction>>,
    announced: bool,
    resolved: bool,
    expected_bids: usize,
}

impl AuctioneerAgent {
    fn new(auction: Arc<Mutex<Auction>>, expected: usize) -> Self {
        Self { id: AgentId::new(), auction, announced: false, resolved: false, expected_bids: expected }
    }
}

#[async_trait]
impl Agent for AuctioneerAgent {
    fn id(&self) -> &AgentId { &self.id }
    async fn initialize(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [Auctioneer] Opening auction.");
        Ok(())
    }
    async fn execute(&mut self, ctx: &AgentContext) -> AgentResult<()> {
        if !self.announced {
            let resource = self.auction.lock().unwrap().resource().to_string();
            println!("  [Auctioneer] Now auctioning: \"{}\"", resource);
            ctx.send_message("trader_1", "cfp", &resource);
            ctx.send_message("trader_2", "cfp", &resource);
            ctx.send_message("trader_3", "cfp", &resource);
            self.announced = true;
        }

        if !self.resolved {
            let auction = self.auction.lock().unwrap();
            let bid_count = auction.bids().len();
            if bid_count >= self.expected_bids {
                println!("\n  [Auctioneer] Bidding closed! {} bids received.", bid_count);
                for bid in auction.bids() {
                    println!("    ${:.0}", bid.amount());
                }
                if let Some(winner) = auction.winner() {
                    println!("  [Auctioneer] WINNER: ${:.0}!", winner.amount());
                } else {
                    println!("  [Auctioneer] No bids met the reserve price.");
                }
                self.resolved = true;
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        Ok(())
    }
    async fn shutdown(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [Auctioneer] Auction closed.");
        Ok(())
    }
    async fn handle_message(&mut self, _ctx: &AgentContext, _s: &str, _p: &str, content: &str) -> AgentResult<()> {
        if content.starts_with("bid:") {
            if let Ok(amount) = content[4..].parse::<f64>() {
                let bid = Bid::new(AgentId::new(), amount, "GPU Cluster");
                println!("  [Auctioneer] <- Bid received: ${:.0}", amount);
                self.auction.lock().unwrap().add_bid(bid);
            }
        }
        Ok(())
    }
}

struct TraderAgent { id: AgentId, name: String, budget: f64, bid_placed: bool }
impl TraderAgent {
    fn new(name: &str, budget: f64) -> Self {
        Self { id: AgentId::new(), name: name.to_string(), budget, bid_placed: false }
    }
}

#[async_trait]
impl Agent for TraderAgent {
    fn id(&self) -> &AgentId { &self.id }
    async fn initialize(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [{}] Budget: ${:.0}", self.name, self.budget);
        Ok(())
    }
    async fn execute(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        Ok(())
    }
    async fn shutdown(&mut self, _ctx: &AgentContext) -> AgentResult<()> { Ok(()) }
    async fn handle_message(&mut self, ctx: &AgentContext, _s: &str, _p: &str, content: &str) -> AgentResult<()> {
        if !self.bid_placed {
            let bid_pct = 0.7 + (self.budget % 100.0) / 333.0;
            let bid_amount = (self.budget * bid_pct).round();
            println!("  [{}] Bidding ${:.0} for \"{}\"", self.name, bid_amount, content);
            ctx.send_message("auctioneer", "propose", &format!("bid:{}", bid_amount));
            self.bid_placed = true;
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), RuntimeError> {
    println!("=== RustyAI — Market Pattern ===");
    println!("    Sealed-bid auction for GPU cluster\n");

    let auction = Arc::new(Mutex::new(
        Auction::new(AuctionType::SealedBid, "GPU Cluster (8x A100)")
            .with_reserve_price(5000.0)
    ));

    {
        let a = auction.lock().unwrap();
        println!("  Auction: {} ({:?})", a.resource(), a.auction_type());
    }
    println!("  Reserve price: $5,000\n");

    let runtime = Runtime::new();
    runtime.spawn(Box::new(TraderAgent::new("Trader-1", 8000.0)), "trader_1").await?;
    runtime.spawn(Box::new(TraderAgent::new("Trader-2", 12000.0)), "trader_2").await?;
    runtime.spawn(Box::new(TraderAgent::new("Trader-3", 6500.0)), "trader_3").await?;
    runtime.spawn(Box::new(AuctioneerAgent::new(auction.clone(), 3)), "auctioneer").await?;

    tokio::time::sleep(std::time::Duration::from_secs(4)).await;
    runtime.shutdown().await?;
    println!("\n=== Market demo complete ===");
    Ok(())
}
