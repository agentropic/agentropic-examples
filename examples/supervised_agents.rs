//! Agent crashes on purpose, Supervisor restarts it automatically.
use agent_core::{Agent, AgentContext, AgentId, AgentError, AgentResult};
use runtime::prelude::*;
use async_trait::async_trait;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

struct FlakyAgent {
    id: AgentId,
    counter: Arc<AtomicU32>,
    fail_until: u32,
}

impl FlakyAgent {
    fn new(counter: Arc<AtomicU32>, fail_until: u32) -> Self {
        Self { id: AgentId::new(), counter, fail_until }
    }
}

#[async_trait]
impl Agent for FlakyAgent {
    fn id(&self) -> &AgentId { &self.id }

    async fn initialize(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [Flaky] Initialized (attempt {})", self.counter.load(Ordering::SeqCst) + 1);
        Ok(())
    }

    async fn execute(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        let count = self.counter.fetch_add(1, Ordering::SeqCst);

        if count < self.fail_until {
            println!("  [Flaky] Crashing! (execution #{})", count + 1);
            return Err(AgentError::ExecutionFailed(format!("Intentional crash #{}", count + 1)));
        }

        println!("  [Flaky] ✓ Running normally (execution #{})", count + 1);
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        Ok(())
    }

    async fn shutdown(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [Flaky] Shutdown.");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), RuntimeError> {
    println!("=== Supervised Agent ===\n");
    println!("  Agent will crash 3 times, then run normally.\n");

    let runtime = Runtime::new();
    let counter = Arc::new(AtomicU32::new(0));

    let agent = FlakyAgent::new(counter.clone(), 3);
    let policy = RestartPolicy::new(RestartStrategy::OnFailure)
        .with_max_retries(5)
        .with_backoff_seconds(1);

    runtime.spawn_with_policy(Box::new(agent), "flaky", policy).await?;

    // Wait for crashes + restarts + normal execution
    tokio::time::sleep(std::time::Duration::from_secs(8)).await;

    let total = counter.load(Ordering::SeqCst);
    println!("\n  Total executions: {} (3 crashes + {} successful)", total, total - 3);

    runtime.shutdown().await?;
    println!("\n=== Done ===");
    Ok(())
}
