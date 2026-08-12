//! Simplest possible agent — spawns with Runtime, ticks, shuts down.
use agent_core::{Agent, AgentContext, AgentId, AgentResult};
use runtime::prelude::*;
use async_trait::async_trait;

struct GreeterAgent {
    id: AgentId,
    count: u32,
}

impl GreeterAgent {
    fn new() -> Self {
        Self { id: AgentId::new(), count: 0 }
    }
}

#[async_trait]
impl Agent for GreeterAgent {
    fn id(&self) -> &AgentId { &self.id }

    async fn initialize(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [Greeter] Hello! I'm alive.");
        Ok(())
    }

    async fn execute(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        self.count += 1;
        println!("  [Greeter] Tick #{}", self.count);
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        Ok(())
    }

    async fn shutdown(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [Greeter] Goodbye after {} ticks!", self.count);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), RuntimeError> {
    println!("=== Hello Agent ===\n");

    let runtime = Runtime::new();
    runtime.spawn(Box::new(GreeterAgent::new()), "greeter").await?;

    println!("  Agent running for 3 seconds...\n");
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    runtime.shutdown().await?;
    println!("\n=== Done ===");
    Ok(())
}
