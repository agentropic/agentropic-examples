//! Team Pattern: Leader assigns tasks, Executors work, Coordinator tracks progress.
//! Uses Team, Role, and RoleType from patterns with real running agents.
use agent_core::{Agent, AgentContext, AgentId, AgentResult};
use patterns::team::{Team, Role, RoleType};
use runtime::prelude::*;
use async_trait::async_trait;

// ── Leader Agent ────────────────────────────────────────────────────

struct LeaderAgent {
    id: AgentId,
    tasks: Vec<&'static str>,
    task_index: usize,
    assigned: u32,
    completed: u32,
}

impl LeaderAgent {
    fn new() -> Self {
        Self {
            id: AgentId::new(),
            tasks: vec![
                "Analyze market data",
                "Generate risk report",
                "Optimize portfolio weights",
                "Run backtest simulation",
                "Prepare client summary",
                "Validate compliance rules",
            ],
            task_index: 0,
            assigned: 0,
            completed: 0,
        }
    }
}

#[async_trait]
impl Agent for LeaderAgent {
    fn id(&self) -> &AgentId { &self.id }

    async fn initialize(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [Leader] Online. {} tasks to assign.", self.tasks.len());
        Ok(())
    }

    async fn execute(&mut self, ctx: &AgentContext) -> AgentResult<()> {
        if self.task_index < self.tasks.len() {
            let task = self.tasks[self.task_index];
            // Round-robin between executors
            let executor = match self.task_index % 3 {
                0 => "executor_1",
                1 => "executor_2",
                _ => "executor_3",
            };

            println!("  [Leader] → Assigning to {}: \"{}\"", executor, task);
            ctx.send_message(executor, "request", task);
            ctx.send_message("coordinator", "inform", &format!("assigned:{}", task));
            self.task_index += 1;
            self.assigned += 1;
        }

        tokio::time::sleep(std::time::Duration::from_millis(800)).await;
        Ok(())
    }

    async fn shutdown(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [Leader] Assigned: {}, Completed: {}", self.assigned, self.completed);
        Ok(())
    }

    async fn handle_message(
        &mut self, _ctx: &AgentContext, sender: &str, _perf: &str, content: &str,
    ) -> AgentResult<()> {
        if content.starts_with("done:") {
            self.completed += 1;
            let task = &content[5..];
            println!("  [Leader] ← {} finished: \"{}\" ({}/{})",
                sender, task, self.completed, self.assigned);
        }
        Ok(())
    }
}

// ── Executor Agent ──────────────────────────────────────────────────

struct ExecutorAgent {
    id: AgentId,
    name: String,
    tasks_done: u32,
}

impl ExecutorAgent {
    fn new(name: &str) -> Self {
        Self { id: AgentId::new(), name: name.to_string(), tasks_done: 0 }
    }
}

#[async_trait]
impl Agent for ExecutorAgent {
    fn id(&self) -> &AgentId { &self.id }

    async fn initialize(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [{}] Ready for tasks.", self.name);
        Ok(())
    }

    async fn execute(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        Ok(())
    }

    async fn shutdown(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [{}] Completed {} tasks.", self.name, self.tasks_done);
        Ok(())
    }

    async fn handle_message(
        &mut self, ctx: &AgentContext, _sender: &str, _perf: &str, content: &str,
    ) -> AgentResult<()> {
        println!("  [{}] ← Received task: \"{}\"", self.name, content);

        // Simulate work
        let work_ms = 300 + (content.len() as u64 * 20);
        println!("  [{}]  Working... ({}ms)", self.name, work_ms);
        tokio::time::sleep(std::time::Duration::from_millis(work_ms)).await;

        self.tasks_done += 1;
        println!("  [{}] ✓ Done: \"{}\"", self.name, content);

        // Report back to leader and coordinator
        ctx.send_message("leader", "inform", &format!("done:{}", content));
        ctx.send_message("coordinator", "inform", &format!("completed:{}", content));

        Ok(())
    }
}

// ── Coordinator Agent ───────────────────────────────────────────────

struct CoordinatorAgent {
    id: AgentId,
    assigned: Vec<String>,
    completed: Vec<String>,
}

impl CoordinatorAgent {
    fn new() -> Self {
        Self { id: AgentId::new(), assigned: Vec::new(), completed: Vec::new() }
    }
}

#[async_trait]
impl Agent for CoordinatorAgent {
    fn id(&self) -> &AgentId { &self.id }

    async fn initialize(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [Coordinator] Tracking team progress.");
        Ok(())
    }

    async fn execute(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        Ok(())
    }

    async fn shutdown(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("\n  [Coordinator] === Team Report ===");
        println!("    Tasks assigned:  {}", self.assigned.len());
        println!("    Tasks completed: {}", self.completed.len());
        let pending = self.assigned.len() - self.completed.len();
        if pending > 0 {
            println!("    Tasks pending:   {}", pending);
        } else {
            println!("    Status: All tasks complete! ✓");
        }
        Ok(())
    }

    async fn handle_message(
        &mut self, _ctx: &AgentContext, _sender: &str, _perf: &str, content: &str,
    ) -> AgentResult<()> {
        if content.starts_with("assigned:") {
            let task = content[9..].to_string();
            self.assigned.push(task);
        } else if content.starts_with("completed:") {
            let task = content[10..].to_string();
            self.completed.push(task);
            println!("  [Coordinator] Progress: {}/{}", self.completed.len(), self.assigned.len());
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), RuntimeError> {
    println!("╔═══════════════════════════════════════════════╗");
    println!("║   RustyAI — Team Pattern Demo              ║");
    println!("║   Leader → Executors → Coordinator            ║");
    println!("╚═══════════════════════════════════════════════╝\n");

    // --- Define the team structure using patterns crate ---
    let executor_1_id = AgentId::new();
    let executor_2_id = AgentId::new();
    let executor_3_id = AgentId::new();
    let leader_id = AgentId::new();
    let coord_id = AgentId::new();

    let mut team = Team::new("Trading Operations");

    team.assign_role(leader_id, Role::new("Team Lead", RoleType::Leader)
        .with_responsibility("Assign tasks")
        .with_responsibility("Track completion"));

    team.assign_role(coord_id, Role::new("Progress Tracker", RoleType::Coordinator)
        .with_responsibility("Monitor assigned vs completed")
        .with_responsibility("Generate team report"));

    team.assign_role(executor_1_id, Role::new("Analyst", RoleType::Executor)
        .with_responsibility("Data analysis"));
    team.assign_role(executor_2_id, Role::new("Risk Manager", RoleType::Executor)
        .with_responsibility("Risk assessment"));
    team.assign_role(executor_3_id, Role::new("Quant", RoleType::Executor)
        .with_responsibility("Quantitative modeling"));

    team.set_leader(leader_id);

    println!("  Team: \"{}\" ({} members)", team.name(), team.members().len());
    for member in team.members() {
        if let Some(role) = team.get_role(member) {
            println!("    {:?} — {} ({})",
                role.role_type(), role.name(),
                role.responsibilities().join(", "));
        }
    }
    println!();

    // --- Spawn real agents with the Runtime ---
    let runtime = Runtime::new();

    runtime.spawn(Box::new(CoordinatorAgent::new()), "coordinator").await?;
    runtime.spawn(Box::new(ExecutorAgent::new("Executor-1")), "executor_1").await?;
    runtime.spawn(Box::new(ExecutorAgent::new("Executor-2")), "executor_2").await?;
    runtime.spawn(Box::new(ExecutorAgent::new("Executor-3")), "executor_3").await?;
    runtime.spawn(Box::new(LeaderAgent::new()), "leader").await?;

    // Wait for all tasks to be assigned and completed
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    println!("\n--- Shutting down ---\n");
    runtime.shutdown().await?;

    println!("\n✓ Team pattern demo complete.");
    Ok(())
}
