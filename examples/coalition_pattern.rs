//! Coalition: Agents form a temporary alliance to tackle a big task, then disband.
use agent_core::{Agent, AgentContext, AgentId, AgentResult};
use patterns::coalition::{Coalition, Strategy, StrategyType};
use runtime::prelude::*;
use async_trait::async_trait;

struct RecruiterAgent { id: AgentId, recruited: bool }
impl RecruiterAgent { fn new() -> Self { Self { id: AgentId::new(), recruited: false } } }

#[async_trait]
impl Agent for RecruiterAgent {
    fn id(&self) -> &AgentId { &self.id }
    async fn initialize(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [Recruiter] Need a coalition for the big job.");
        Ok(())
    }
    async fn execute(&mut self, ctx: &AgentContext) -> AgentResult<()> {
        if !self.recruited {
            println!("  [Recruiter] → Recruiting specialists...");
            ctx.send_message("hacker", "propose", "Join coalition: infiltrate target system");
            ctx.send_message("analyst", "propose", "Join coalition: analyze defenses");
            ctx.send_message("extractor", "propose", "Join coalition: extract the data");
            self.recruited = true;
        }
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        Ok(())
    }
    async fn shutdown(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [Recruiter] Coalition disbanded.");
        Ok(())
    }
    async fn handle_message(&mut self, _ctx: &AgentContext, _s: &str, _p: &str, content: &str) -> AgentResult<()> {
        println!("  [Recruiter] ← \"{}\"", content);
        Ok(())
    }
}

struct SpecialistAgent { id: AgentId, name: String, skill: String }
impl SpecialistAgent {
    fn new(name: &str, skill: &str) -> Self {
        Self { id: AgentId::new(), name: name.to_string(), skill: skill.to_string() }
    }
}

#[async_trait]
impl Agent for SpecialistAgent {
    fn id(&self) -> &AgentId { &self.id }
    async fn initialize(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [{}] Specialist ({}) standing by.", self.name, self.skill);
        Ok(())
    }
    async fn execute(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        Ok(())
    }
    async fn shutdown(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [{}] Leaving coalition.", self.name);
        Ok(())
    }
    async fn handle_message(&mut self, ctx: &AgentContext, _s: &str, perf: &str, content: &str) -> AgentResult<()> {
        if perf == "Propose" || perf == "propose" {
            println!("  [{}] ← Proposal: \"{}\"", self.name, content);
            println!("  [{}] ✓ Accepted! Applying {} skills...", self.name, self.skill);
            tokio::time::sleep(std::time::Duration::from_millis(600)).await;
            let result = format!("{} complete: {} phase done", self.name, self.skill);
            println!("  [{}] → \"{}\"", self.name, result);
            ctx.send_message("recruiter", "inform", &result);
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), RuntimeError> {
    println!("╔═══════════════════════════════════════════════╗");
    println!("║   RustyAI — Coalition Pattern              ║");
    println!("║   Temporary alliance for a mission            ║");
    println!("╚═══════════════════════════════════════════════╝\n");

    let mut coalition = Coalition::new("Operation Nightfall");
    let strategy = Strategy::new(StrategyType::MaximizeUtility)
        .with_parameter("risk_tolerance", 0.8);
    coalition.set_strategy(strategy);

    println!("  Coalition: \"{}\"", coalition.name());
    println!("  Strategy: {:?}\n", coalition.strategy().unwrap().strategy_type());

    let runtime = Runtime::new();
    runtime.spawn(Box::new(SpecialistAgent::new("Hacker", "infiltration")), "hacker").await?;
    runtime.spawn(Box::new(SpecialistAgent::new("Analyst", "analysis")), "analyst").await?;
    runtime.spawn(Box::new(SpecialistAgent::new("Extractor", "extraction")), "extractor").await?;
    runtime.spawn(Box::new(RecruiterAgent::new()), "recruiter").await?;

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    runtime.shutdown().await?;
    println!("\n✓ Coalition demo complete.");
    Ok(())
}
