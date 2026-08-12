//! Blackboard: Agents read and write to shared knowledge space to solve a problem.
use agent_core::{Agent, AgentContext, AgentId, AgentResult};
use patterns::blackboard::Blackboard;
use runtime::prelude::*;
use async_trait::async_trait;
use std::sync::{Arc, Mutex};

struct SensorAgent { id: AgentId, name: String, data: Vec<(&'static str, &'static str)>, index: usize, board: Arc<Mutex<Blackboard>> }
impl SensorAgent {
    fn new(name: &str, data: Vec<(&'static str, &'static str)>, board: Arc<Mutex<Blackboard>>) -> Self {
        Self { id: AgentId::new(), name: name.to_string(), data, index: 0, board }
    }
}

#[async_trait]
impl Agent for SensorAgent {
    fn id(&self) -> &AgentId { &self.id }
    async fn initialize(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [{}] Sensor online. {} readings to report.", self.name, self.data.len());
        Ok(())
    }
    async fn execute(&mut self, ctx: &AgentContext) -> AgentResult<()> {
        if self.index < self.data.len() {
            let (key, value) = self.data[self.index];
            self.board.lock().unwrap().write(key, value);
            println!("  [{}]  Wrote: {} = {}", self.name, key, value);
            ctx.send_message("analyzer", "inform", &format!("updated:{}", key));
            self.index += 1;
        }
        tokio::time::sleep(std::time::Duration::from_millis(600)).await;
        Ok(())
    }
    async fn shutdown(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [{}] Sensor offline.", self.name);
        Ok(())
    }
}

struct AnalyzerAgent { id: AgentId, board: Arc<Mutex<Blackboard>>, analyses: u32 }
impl AnalyzerAgent {
    fn new(board: Arc<Mutex<Blackboard>>) -> Self {
        Self { id: AgentId::new(), board, analyses: 0 }
    }
}

#[async_trait]
impl Agent for AnalyzerAgent {
    fn id(&self) -> &AgentId { &self.id }
    async fn initialize(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [Analyzer] Watching blackboard for data.");
        Ok(())
    }
    async fn execute(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        Ok(())
    }
    async fn shutdown(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [Analyzer] Performed {} analyses.", self.analyses);
        let board = self.board.lock().unwrap();
        println!("\n  Final Blackboard State ({} entries):", board.size());
        let mut entries: Vec<_> = board.knowledge().iter().collect();
        entries.sort_by_key(|(k, _)| k.clone());
        for (key, value) in entries {
            println!("    {} = {}", key, value);
        }
        Ok(())
    }
    async fn handle_message(&mut self, ctx: &AgentContext, _s: &str, _p: &str, content: &str) -> AgentResult<()> {
        if content.starts_with("updated:") {
            let key = &content[8..];
            let board = self.board.lock().unwrap();
            if let Some(value) = board.read(key) {
                println!("  [Analyzer] Read: {} = {}", key, value);
                self.analyses += 1;

                // Write analysis result back to blackboard
                drop(board);
                let analysis_key = format!("analysis:{}", key);
                let analysis = format!("Processed {}", key);
                self.board.lock().unwrap().write(&analysis_key, &analysis);
                println!("  [Analyzer]  Wrote: {} = {}", analysis_key, analysis);
                ctx.send_message("decision_maker", "inform", &format!("analyzed:{}", key));
            }
        }
        Ok(())
    }
}

struct DecisionAgent { id: AgentId, board: Arc<Mutex<Blackboard>>, decided: bool }
impl DecisionAgent {
    fn new(board: Arc<Mutex<Blackboard>>) -> Self {
        Self { id: AgentId::new(), board, decided: false }
    }
}

#[async_trait]
impl Agent for DecisionAgent {
    fn id(&self) -> &AgentId { &self.id }
    async fn initialize(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [DecisionMaker] Waiting for enough data.");
        Ok(())
    }
    async fn execute(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        if !self.decided {
            let board = self.board.lock().unwrap();
            let analysis_count = board.knowledge().keys().filter(|k| k.starts_with("analysis:")).count();
            if analysis_count >= 3 {
                println!("\n  [DecisionMaker] Enough data. Making decision...");
                println!("  [DecisionMaker] Decision: All systems nominal. Proceed with launch.");
                drop(board);
                self.board.lock().unwrap().write("decision", "Proceed with launch");
                self.decided = true;
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        Ok(())
    }
    async fn shutdown(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [DecisionMaker] Done.");
        Ok(())
    }
    async fn handle_message(&mut self, _ctx: &AgentContext, _s: &str, _p: &str, _content: &str) -> AgentResult<()> {
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), RuntimeError> {
    println!("╔═══════════════════════════════════════════════╗");
    println!("║   RustyAI — Blackboard Pattern             ║");
    println!("║   Shared knowledge for collaborative solving  ║");
    println!("╚═══════════════════════════════════════════════╝\n");

    let board = Arc::new(Mutex::new(Blackboard::new("Mission Control")));
    println!("  Blackboard: \"{}\"\n", board.lock().unwrap().name());

    let runtime = Runtime::new();

    runtime.spawn(Box::new(DecisionAgent::new(board.clone())), "decision_maker").await?;
    runtime.spawn(Box::new(AnalyzerAgent::new(board.clone())), "analyzer").await?;
    runtime.spawn(Box::new(SensorAgent::new("TempSensor", vec![
        ("temperature", "72°F"),
        ("humidity", "45%"),
    ], board.clone())), "temp_sensor").await?;
    runtime.spawn(Box::new(SensorAgent::new("PressureSensor", vec![
        ("pressure", "1013 hPa"),
        ("wind_speed", "12 mph"),
    ], board.clone())), "pressure_sensor").await?;

    tokio::time::sleep(std::time::Duration::from_secs(6)).await;
    runtime.shutdown().await?;
    println!("\n✓ Blackboard demo complete.");
    Ok(())
}
