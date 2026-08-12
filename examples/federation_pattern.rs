//! Federation: Agents vote on proposals with weighted voting.
use agent_core::{Agent, AgentContext, AgentId, AgentResult};
use patterns::federation::{Federation, Policy, PolicyType};
use runtime::prelude::*;
use async_trait::async_trait;
use std::sync::{Arc, Mutex};

struct VoteResult { yes: f64, no: f64, total_weight: f64 }

struct ChairAgent {
    id: AgentId,
    proposed: bool,
    votes: Arc<Mutex<VoteResult>>,
    resolved: bool,
    threshold: f64,
}

impl ChairAgent {
    fn new(votes: Arc<Mutex<VoteResult>>, threshold: f64) -> Self {
        Self { id: AgentId::new(), proposed: false, votes, resolved: false, threshold }
    }
}

#[async_trait]
impl Agent for ChairAgent {
    fn id(&self) -> &AgentId { &self.id }
    async fn initialize(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [Chair] Federation session open.");
        Ok(())
    }
    async fn execute(&mut self, ctx: &AgentContext) -> AgentResult<()> {
        if !self.proposed {
            let proposal = "Allocate $50k to AI research fund";
            println!("  [Chair] Proposal: \"{}\"", proposal);
            ctx.send_message("delegate_1", "cfp", proposal);
            ctx.send_message("delegate_2", "cfp", proposal);
            ctx.send_message("delegate_3", "cfp", proposal);
            self.proposed = true;
        }

        if !self.resolved {
            let v = self.votes.lock().unwrap();
            if v.yes + v.no >= v.total_weight {
                let pct = v.yes / (v.yes + v.no);
                println!("\n  [Chair] Vote Results:");
                println!("    Yes: {:.1} weight", v.yes);
                println!("    No:  {:.1} weight", v.no);
                println!("    Approval: {:.0}% (threshold: {:.0}%)", pct * 100.0, self.threshold * 100.0);
                if pct >= self.threshold {
                    println!("  [Chair] PASSED");
                } else {
                    println!("  [Chair] REJECTED");
                }
                self.resolved = true;
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        Ok(())
    }
    async fn shutdown(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [Chair] Session closed.");
        Ok(())
    }
    async fn handle_message(&mut self, _ctx: &AgentContext, _s: &str, _p: &str, content: &str) -> AgentResult<()> {
        if content.starts_with("yes:") {
            let weight: f64 = content[4..].parse().unwrap_or(1.0);
            self.votes.lock().unwrap().yes += weight;
            println!("  [Chair] ← Vote YES (weight {:.1})", weight);
        } else if content.starts_with("no:") {
            let weight: f64 = content[3..].parse().unwrap_or(1.0);
            self.votes.lock().unwrap().no += weight;
            println!("  [Chair] ← Vote NO (weight {:.1})", weight);
        }
        Ok(())
    }
}

struct DelegateAgent { id: AgentId, name: String, weight: f64, votes_yes: bool }
impl DelegateAgent {
    fn new(name: &str, weight: f64, votes_yes: bool) -> Self {
        Self { id: AgentId::new(), name: name.to_string(), weight, votes_yes }
    }
}

#[async_trait]
impl Agent for DelegateAgent {
    fn id(&self) -> &AgentId { &self.id }
    async fn initialize(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [{}] Delegate present (weight: {:.1})", self.name, self.weight);
        Ok(())
    }
    async fn execute(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        Ok(())
    }
    async fn shutdown(&mut self, _ctx: &AgentContext) -> AgentResult<()> { Ok(()) }
    async fn handle_message(&mut self, ctx: &AgentContext, _s: &str, _p: &str, content: &str) -> AgentResult<()> {
        println!("  [{}] Considering: \"{}\"", self.name, content);
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        if self.votes_yes {
            println!("  [{}]  Voting YES", self.name);
            ctx.send_message("chair", "inform", &format!("yes:{}", self.weight));
        } else {
            println!("  [{}]  Voting NO", self.name);
            ctx.send_message("chair", "inform", &format!("no:{}", self.weight));
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), RuntimeError> {
    println!("╔═══════════════════════════════════════════════╗");
    println!("║   RustyAI — Federation Pattern             ║");
    println!("║   Weighted voting on proposals                ║");
    println!("╚═══════════════════════════════════════════════╝\n");

    let mut federation = Federation::new("Research Council");
    let d1 = AgentId::new();
    let d2 = AgentId::new();
    let d3 = AgentId::new();
    federation.add_member(d1);
    federation.add_member(d2);
    federation.add_member(d3);
    federation.set_weight(d1, 2.0);
    federation.set_weight(d2, 1.5);
    federation.set_weight(d3, 1.0);

    let policy = Policy::new("budget_approval", PolicyType::WeightedVote)
        .with_threshold(0.6)
        .with_rule("Requires 60% weighted approval");
    federation.add_policy(policy);

    let total_weight = 2.0 + 1.5 + 1.0;
    let votes = Arc::new(Mutex::new(VoteResult { yes: 0.0, no: 0.0, total_weight }));

    println!("  Federation: \"{}\" ({} members, policy: WeightedVote)\n", federation.name(), federation.size());

    let runtime = Runtime::new();
    runtime.spawn(Box::new(DelegateAgent::new("Delegate-1", 2.0, true)), "delegate_1").await?;
    runtime.spawn(Box::new(DelegateAgent::new("Delegate-2", 1.5, true)), "delegate_2").await?;
    runtime.spawn(Box::new(DelegateAgent::new("Delegate-3", 1.0, false)), "delegate_3").await?;
    runtime.spawn(Box::new(ChairAgent::new(votes, 0.6)), "chair").await?;

    tokio::time::sleep(std::time::Duration::from_secs(4)).await;
    runtime.shutdown().await?;
    println!("\n✓ Federation demo complete.");
    Ok(())
}
