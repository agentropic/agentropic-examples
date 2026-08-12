//! Three agents: Asker asks questions, Responder answers, Observer watches.
use agent_core::{Agent, AgentContext, AgentId, AgentResult};
use runtime::prelude::*;
use async_trait::async_trait;

struct AskerAgent { id: AgentId, asked: bool }
impl AskerAgent { fn new() -> Self { Self { id: AgentId::new(), asked: false } } }

#[async_trait]
impl Agent for AskerAgent {
    fn id(&self) -> &AgentId { &self.id }
    async fn initialize(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [Asker] Ready.");
        Ok(())
    }
    async fn execute(&mut self, ctx: &AgentContext) -> AgentResult<()> {
        if !self.asked {
            println!("  [Asker] → \"What patterns does RustyAI support?\"");
            ctx.send_message("responder", "query", "What patterns does RustyAI support?");
            self.asked = true;
        }
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        Ok(())
    }
    async fn shutdown(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [Asker] Goodbye!");
        Ok(())
    }
    async fn handle_message(&mut self, _ctx: &AgentContext, _s: &str, _p: &str, content: &str) -> AgentResult<()> {
        println!("  [Asker] ← Answer: \"{}\"", content);
        Ok(())
    }
}

struct ResponderAgent { id: AgentId }
impl ResponderAgent { fn new() -> Self { Self { id: AgentId::new() } } }

#[async_trait]
impl Agent for ResponderAgent {
    fn id(&self) -> &AgentId { &self.id }
    async fn initialize(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [Responder] Waiting for questions...");
        Ok(())
    }
    async fn execute(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        Ok(())
    }
    async fn shutdown(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [Responder] Goodbye!");
        Ok(())
    }
    async fn handle_message(&mut self, ctx: &AgentContext, sender: &str, _p: &str, content: &str) -> AgentResult<()> {
        println!("  [Responder] ← \"{}\"", content);
        let answer = "Hierarchy, Swarm, Coalition, Market, Federation, Team, Holarchy, and Blackboard.";
        println!("  [Responder] → \"{}\"", answer);
        ctx.send_message(sender, "inform", answer);
        Ok(())
    }
}

struct ObserverAgent { id: AgentId, ticks: u32 }
impl ObserverAgent { fn new() -> Self { Self { id: AgentId::new(), ticks: 0 } } }

#[async_trait]
impl Agent for ObserverAgent {
    fn id(&self) -> &AgentId { &self.id }
    async fn initialize(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [Observer] Watching...");
        Ok(())
    }
    async fn execute(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        self.ticks += 1;
        if self.ticks == 5 { println!("  [Observer] The agents are communicating!"); }
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        Ok(())
    }
    async fn shutdown(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [Observer] Observed {} ticks. Goodbye!", self.ticks);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), RuntimeError> {
    println!("=== Agents Talking ===\n");

    let runtime = Runtime::new();
    runtime.spawn(Box::new(ResponderAgent::new()), "responder").await?;
    runtime.spawn(Box::new(AskerAgent::new()), "asker").await?;
    runtime.spawn(Box::new(ObserverAgent::new()), "observer").await?;

    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    runtime.shutdown().await?;
    println!("\n=== Done ===");
    Ok(())
}
