//! Swarm: Decentralized agents vote to reach consensus without a leader.
use agent_core::{Agent, AgentContext, AgentId, AgentResult};
use patterns::swarm::Swarm;
use runtime::prelude::*;
use async_trait::async_trait;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct VoteBoard {
    votes: Arc<Mutex<Vec<(String, String)>>>,
}

impl VoteBoard {
    fn new() -> Self { Self { votes: Arc::new(Mutex::new(Vec::new())) } }
    fn cast(&self, agent: &str, choice: &str) {
        self.votes.lock().unwrap().push((agent.to_string(), choice.to_string()));
    }
    fn tally(&self) -> Vec<(String, usize)> {
        let votes = self.votes.lock().unwrap();
        let mut counts = std::collections::HashMap::new();
        for (_, choice) in votes.iter() {
            *counts.entry(choice.clone()).or_insert(0) += 1;
        }
        let mut result: Vec<_> = counts.into_iter().collect();
        result.sort_by(|a, b| b.1.cmp(&a.1));
        result
    }
    fn count(&self) -> usize { self.votes.lock().unwrap().len() }
}

struct SwarmAgent {
    id: AgentId,
    name: String,
    preference: String,
    voted: bool,
    board: VoteBoard,
}

impl SwarmAgent {
    fn new(name: &str, preference: &str, board: VoteBoard) -> Self {
        Self { id: AgentId::new(), name: name.to_string(), preference: preference.to_string(), voted: false, board }
    }
}

#[async_trait]
impl Agent for SwarmAgent {
    fn id(&self) -> &AgentId { &self.id }
    async fn initialize(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [{}] Joined swarm. Preference: {}", self.name, self.preference);
        Ok(())
    }
    async fn execute(&mut self, ctx: &AgentContext) -> AgentResult<()> {
        if !self.voted {
            // Broadcast vote to all peers
            let msg = format!("vote:{}", self.preference);
            for peer in &["scout_1", "scout_2", "scout_3", "scout_4", "scout_5"] {
                if *peer != self.name.to_lowercase().replace("-", "_").replace(" ", "_") {
                    ctx.send_message(peer, "inform", &msg);
                }
            }
            self.board.cast(&self.name, &self.preference);
            println!("  [{}]  Voted: {}", self.name, self.preference);
            self.voted = true;
        }
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        Ok(())
    }
    async fn shutdown(&mut self, _ctx: &AgentContext) -> AgentResult<()> { Ok(()) }
    async fn handle_message(&mut self, _ctx: &AgentContext, _s: &str, _p: &str, _content: &str) -> AgentResult<()> {
        Ok(())
    }
}

struct TallyAgent {
    id: AgentId,
    board: VoteBoard,
    reported: bool,
    expected: usize,
}

impl TallyAgent {
    fn new(board: VoteBoard, expected: usize) -> Self {
        Self { id: AgentId::new(), board, reported: false, expected }
    }
}

#[async_trait]
impl Agent for TallyAgent {
    fn id(&self) -> &AgentId { &self.id }
    async fn initialize(&mut self, _ctx: &AgentContext) -> AgentResult<()> { Ok(()) }
    async fn execute(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        if !self.reported && self.board.count() >= self.expected {
            println!("\n  [Tally] Consensus Results:");
            for (choice, count) in self.board.tally() {
                let bar = "█".repeat(count);
                println!("    {} {} ({})", choice, bar, count);
            }
            let winner = self.board.tally().first().map(|(c, _)| c.clone()).unwrap_or_default();
            println!("    → Swarm decision: {}", winner);
            self.reported = true;
        }
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        Ok(())
    }
    async fn shutdown(&mut self, _ctx: &AgentContext) -> AgentResult<()> { Ok(()) }
}

#[tokio::main]
async fn main() -> Result<(), RuntimeError> {
    println!("╔═══════════════════════════════════════════════╗");
    println!("║   RustyAI — Swarm Pattern                  ║");
    println!("║   Decentralized consensus voting              ║");
    println!("╚═══════════════════════════════════════════════╝\n");

    let mut swarm = Swarm::new("Scout Swarm");
    let board = VoteBoard::new();

    let agents_data = vec![
        ("Scout-1", "Route A"),
        ("Scout-2", "Route B"),
        ("Scout-3", "Route A"),
        ("Scout-4", "Route A"),
        ("Scout-5", "Route B"),
    ];

    let runtime = Runtime::new();

    for (name, pref) in &agents_data {
        let agent = SwarmAgent::new(name, pref, board.clone());
        swarm.add_member(*agent.id());
        let runtime_name = name.to_lowercase().replace("-", "_");
        runtime.spawn(Box::new(agent), &runtime_name).await?;
    }

    runtime.spawn(Box::new(TallyAgent::new(board.clone(), 5)), "tally").await?;

    println!("  Swarm: \"{}\" ({} members)\n", swarm.name(), swarm.size());

    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    runtime.shutdown().await?;
    println!("\n✓ Swarm demo complete.");
    Ok(())
}
