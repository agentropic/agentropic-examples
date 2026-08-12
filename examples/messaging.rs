//! Two agents exchanging messages through the Runtime's Router.
use agent_core::{Agent, AgentContext, AgentId, AgentResult};
use runtime::prelude::*;
use async_trait::async_trait;

struct PingAgent {
    id: AgentId,
    pinged: bool,
}

impl PingAgent {
    fn new() -> Self { Self { id: AgentId::new(), pinged: false } }
}

#[async_trait]
impl Agent for PingAgent {
    fn id(&self) -> &AgentId { &self.id }

    async fn initialize(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [Ping] Ready.");
        Ok(())
    }

    async fn execute(&mut self, ctx: &AgentContext) -> AgentResult<()> {
        if !self.pinged {
            println!("  [Ping] → Sending 'ping!' to Pong");
            ctx.send_message("pong", "request", "ping!");
            self.pinged = true;
        }
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        Ok(())
    }

    async fn shutdown(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [Ping] Done.");
        Ok(())
    }

    async fn handle_message(
        &mut self, _ctx: &AgentContext, _sender: &str, _perf: &str, content: &str,
    ) -> AgentResult<()> {
        println!("  [Ping] ← Received: \"{}\"", content);
        Ok(())
    }
}

struct PongAgent {
    id: AgentId,
}

impl PongAgent {
    fn new() -> Self { Self { id: AgentId::new() } }
}

#[async_trait]
impl Agent for PongAgent {
    fn id(&self) -> &AgentId { &self.id }

    async fn initialize(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [Pong] Ready.");
        Ok(())
    }

    async fn execute(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        Ok(())
    }

    async fn shutdown(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [Pong] Done.");
        Ok(())
    }

    async fn handle_message(
        &mut self, ctx: &AgentContext, sender: &str, _perf: &str, content: &str,
    ) -> AgentResult<()> {
        println!("  [Pong] ← Received: \"{}\"", content);
        println!("  [Pong] → Replying: \"pong!\"");
        ctx.send_message(sender, "inform", "pong!");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), RuntimeError> {
    println!("=== Messaging Example ===\n");

    let runtime = Runtime::new();
    runtime.spawn(Box::new(PongAgent::new()), "pong").await?;
    runtime.spawn(Box::new(PingAgent::new()), "ping").await?;

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    runtime.shutdown().await?;
    println!("\n=== Done ===");
    Ok(())
}
