//! Full system demo: agents run, talk, reason, and self-heal.
use agent_core::{Agent, AgentContext, AgentId, AgentError, AgentResult};
use cognition::Rule;
use runtime::prelude::*;
use runtime::CognitiveAgent;
use async_trait::async_trait;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

/// Worker that does tasks and reports to manager
struct WorkerAgent {
    id: AgentId,
    name: String,
    tasks_done: u32,
}

impl WorkerAgent {
    fn new(name: &str) -> Self {
        Self { id: AgentId::new(), name: name.to_string(), tasks_done: 0 }
    }
}

#[async_trait]
impl Agent for WorkerAgent {
    fn id(&self) -> &AgentId { &self.id }

    async fn initialize(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [{}] Online.", self.name);
        Ok(())
    }

    async fn execute(&mut self, ctx: &AgentContext) -> AgentResult<()> {
        self.tasks_done += 1;
        if self.tasks_done <= 3 {
            let report = format!("{} completed task #{}", self.name, self.tasks_done);
            ctx.send_message("manager", "inform", &report);
            println!("  [{}] {}", self.name, report);
        }
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        Ok(())
    }

    async fn shutdown(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [{}] Finished {} tasks.", self.name, self.tasks_done);
        Ok(())
    }
}

/// Manager that receives reports from workers
struct ManagerAgent {
    id: AgentId,
    reports: Vec<String>,
}

impl ManagerAgent {
    fn new() -> Self {
        Self { id: AgentId::new(), reports: Vec::new() }
    }
}

#[async_trait]
impl Agent for ManagerAgent {
    fn id(&self) -> &AgentId { &self.id }

    async fn initialize(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [Manager] Overseeing workers.");
        Ok(())
    }

    async fn execute(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        Ok(())
    }

    async fn shutdown(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [Manager] Received {} reports total.", self.reports.len());
        Ok(())
    }

    async fn handle_message(&mut self, _ctx: &AgentContext, _s: &str, _p: &str, content: &str) -> AgentResult<()> {
        println!("  [Manager] ← Report: \"{}\"", content);
        self.reports.push(content.to_string());
        Ok(())
    }
}

/// Unreliable agent that crashes, gets restarted by supervisor
struct UnreliableAgent {
    id: AgentId,
    counter: Arc<AtomicU32>,
}

impl UnreliableAgent {
    fn new(counter: Arc<AtomicU32>) -> Self {
        Self { id: AgentId::new(), counter }
    }
}

#[async_trait]
impl Agent for UnreliableAgent {
    fn id(&self) -> &AgentId { &self.id }
    async fn initialize(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [Unreliable] Starting...");
        Ok(())
    }
    async fn execute(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        let count = self.counter.fetch_add(1, Ordering::SeqCst);
        if count < 2 {
            println!("  [Unreliable] Crash #{}", count + 1);
            return Err(AgentError::ExecutionFailed("boom".into()));
        }
        println!("  [Unreliable] ✓ Stable now (tick {})", count + 1);
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        Ok(())
    }
    async fn shutdown(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [Unreliable] Shutdown.");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), RuntimeError> {
    println!("╔═══════════════════════════════════════════════╗");
    println!("║   RustyAI — Full System Demo               ║");
    println!("║   Running + Messaging + Reasoning + Recovery   ║");
    println!("╚═══════════════════════════════════════════════╝\n");

    let runtime = Runtime::new();

    // --- 1. Workers report to Manager ---
    println!("--- Stage 1: Workers & Manager ---\n");

    runtime.spawn(Box::new(ManagerAgent::new()), "manager").await?;
    runtime.spawn(Box::new(WorkerAgent::new("Alpha")), "alpha").await?;
    runtime.spawn(Box::new(WorkerAgent::new("Beta")), "beta").await?;

    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    // --- 2. Cognitive Agent answers questions ---
    println!("\n--- Stage 2: Cognitive Reasoning ---\n");

    let mut thinker = CognitiveAgent::from_config("data/beliefs.json", "data/config.json");
    thinker.add_rule(Rule::new("topic:what_is")
        .with_condition("what").with_condition("rustyai")
        .with_conclusion("what_is_rustyai"));
    thinker.add_rule(Rule::new("topic:patterns")
        .with_condition("pattern").with_condition("support")
        .with_conclusion("patterns"));

    runtime.spawn(Box::new(thinker), "thinker").await?;

    // Alpha asks Thinker a question
    // (We simulate by spawning a quick asker)
    struct QuickAsker { id: AgentId, asked: bool }
    impl QuickAsker { fn new() -> Self { Self { id: AgentId::new(), asked: false } } }

    #[async_trait]
    impl Agent for QuickAsker {
        fn id(&self) -> &AgentId { &self.id }
        async fn initialize(&mut self, _ctx: &AgentContext) -> AgentResult<()> { Ok(()) }
        async fn execute(&mut self, ctx: &AgentContext) -> AgentResult<()> {
            if !self.asked {
                println!("  [QuickAsker] → \"What is RustyAI?\"");
                ctx.send_message("thinker", "query", "What is RustyAI?");
                self.asked = true;
            }
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            Ok(())
        }
        async fn shutdown(&mut self, _ctx: &AgentContext) -> AgentResult<()> { Ok(()) }
        async fn handle_message(&mut self, _ctx: &AgentContext, _s: &str, _p: &str, content: &str) -> AgentResult<()> {
            println!("  [QuickAsker] ← \"{}\"", content);
            Ok(())
        }
    }

    runtime.spawn(Box::new(QuickAsker::new()), "quick_asker").await?;
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    // --- 3. Self-healing agent ---
    println!("\n--- Stage 3: Self-Healing ---\n");

    let crash_counter = Arc::new(AtomicU32::new(0));
    let policy = RestartPolicy::new(RestartStrategy::OnFailure)
        .with_max_retries(5)
        .with_backoff_seconds(1);

    runtime.spawn_with_policy(
        Box::new(UnreliableAgent::new(crash_counter.clone())),
        "unreliable",
        policy,
    ).await?;

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    // --- Summary ---
    println!("\n--- Summary ---");
    println!("  Agents running: {}", runtime.agent_count().await);
    println!("  Crash counter: {} (2 crashes + recovery)", crash_counter.load(Ordering::SeqCst));

    println!("\n--- Shutting down ---\n");
    runtime.shutdown().await?;

    println!("\n✓ Full system demo complete.");
    Ok(())
}
