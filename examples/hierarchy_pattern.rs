//! Hierarchy: Commander → Captain → Soldiers. Orders flow down, reports flow up.
use agent_core::{Agent, AgentContext, AgentId, AgentResult};
use patterns::hierarchy::{Hierarchy, Level, LevelType};
use runtime::prelude::*;
use async_trait::async_trait;

struct CommanderAgent { id: AgentId, issued: bool, reports: u32 }
impl CommanderAgent { fn new() -> Self { Self { id: AgentId::new(), issued: false, reports: 0 } } }

#[async_trait]
impl Agent for CommanderAgent {
    fn id(&self) -> &AgentId { &self.id }
    async fn initialize(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [Commander] Strategic level online.");
        Ok(())
    }
    async fn execute(&mut self, ctx: &AgentContext) -> AgentResult<()> {
        if !self.issued {
            println!("  [Commander] ↓ Order: \"Secure sector 7\"");
            ctx.send_message("captain", "request", "Secure sector 7");
            self.issued = true;
        }
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        Ok(())
    }
    async fn shutdown(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [Commander] {} reports received. Standing down.", self.reports);
        Ok(())
    }
    async fn handle_message(&mut self, _ctx: &AgentContext, _s: &str, _p: &str, content: &str) -> AgentResult<()> {
        self.reports += 1;
        println!("  [Commander] ← Report: \"{}\"", content);
        Ok(())
    }
}

struct CaptainAgent { id: AgentId, delegated: bool }
impl CaptainAgent { fn new() -> Self { Self { id: AgentId::new(), delegated: false } } }

#[async_trait]
impl Agent for CaptainAgent {
    fn id(&self) -> &AgentId { &self.id }
    async fn initialize(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [Captain] Tactical level online.");
        Ok(())
    }
    async fn execute(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        Ok(())
    }
    async fn shutdown(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [Captain] Standing down.");
        Ok(())
    }
    async fn handle_message(&mut self, ctx: &AgentContext, _s: &str, _p: &str, content: &str) -> AgentResult<()> {
        if !self.delegated {
            // First message = order from Commander. Delegate to soldiers.
            println!("  [Captain] ← Order: \"{}\"", content);
            let zone = content.replace("Secure ", "");
            println!("  [Captain] ↓ soldier_1: \"Scout {}\"", zone);
            ctx.send_message("soldier_1", "request", &format!("Scout {}", zone));
            println!("  [Captain] ↓ soldier_2: \"Hold perimeter of {}\"", zone);
            ctx.send_message("soldier_2", "request", &format!("Hold perimeter of {}", zone));
            self.delegated = true;
        } else {
            // Subsequent messages = reports from soldiers. Forward up.
            println!("  [Captain] ← Report: \"{}\"", content);
            ctx.send_message("commander", "inform", content);
        }
        Ok(())
    }
}

struct SoldierAgent { id: AgentId, name: String }
impl SoldierAgent { fn new(name: &str) -> Self { Self { id: AgentId::new(), name: name.to_string() } } }

#[async_trait]
impl Agent for SoldierAgent {
    fn id(&self) -> &AgentId { &self.id }
    async fn initialize(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [{}] Operational level online.", self.name);
        Ok(())
    }
    async fn execute(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        Ok(())
    }
    async fn shutdown(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [{}] Standing down.", self.name);
        Ok(())
    }
    async fn handle_message(&mut self, ctx: &AgentContext, _s: &str, _p: &str, content: &str) -> AgentResult<()> {
        println!("  [{}] ← Task: \"{}\"", self.name, content);
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        println!("  [{}] ✓ Completed: \"{}\"", self.name, content);
        ctx.send_message("captain", "inform", &format!("{}: {} done", self.name, content));
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), RuntimeError> {
    println!("╔═══════════════════════════════════════════════╗");
    println!("║   RustyAI — Hierarchy Pattern              ║");
    println!("║   Commander → Captain → Soldiers              ║");
    println!("╚═══════════════════════════════════════════════╝\n");

    let cmd_id = AgentId::new();
    let cap_id = AgentId::new();
    let s1_id = AgentId::new();
    let s2_id = AgentId::new();

    let mut hierarchy = Hierarchy::new("Military Operations");
    let strategic = Level::new("Command", LevelType::Strategic, 3);
    let tactical = Level::new("Tactical", LevelType::Tactical, 2);
    let operational = Level::new("Field", LevelType::Operational, 1);
    hierarchy.add_level(strategic.clone());
    hierarchy.add_level(tactical.clone());
    hierarchy.add_level(operational.clone());
    hierarchy.assign_agent(cmd_id, strategic);
    hierarchy.assign_agent(cap_id, tactical);
    hierarchy.assign_agent(s1_id, operational.clone());
    hierarchy.assign_agent(s2_id, operational);

    println!("  Hierarchy: \"{}\" ({} levels)\n", hierarchy.name(), hierarchy.levels().len());

    let runtime = Runtime::new();
    runtime.spawn(Box::new(SoldierAgent::new("Soldier-1")), "soldier_1").await?;
    runtime.spawn(Box::new(SoldierAgent::new("Soldier-2")), "soldier_2").await?;
    runtime.spawn(Box::new(CaptainAgent::new()), "captain").await?;
    runtime.spawn(Box::new(CommanderAgent::new()), "commander").await?;

    tokio::time::sleep(std::time::Duration::from_secs(4)).await;
    runtime.shutdown().await?;
    println!("\n✓ Hierarchy demo complete.");
    Ok(())
}
