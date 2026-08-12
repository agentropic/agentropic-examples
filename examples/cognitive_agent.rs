//! CognitiveAgent reasons from BeliefBase, falls back to LLM for unknown questions.
use agent_core::{Agent, AgentContext, AgentId, AgentResult};
use cognition::Rule;
use runtime::prelude::*;
use runtime::CognitiveAgent;
use async_trait::async_trait;

struct CuriousAgent {
    id: AgentId,
    questions: Vec<&'static str>,
    index: usize,
    waiting: bool,
}

impl CuriousAgent {
    fn new() -> Self {
        Self {
            id: AgentId::new(),
            questions: vec![
                "What is RustyAI?",
                "What patterns does it support?",
                "Can RustyAI agents collaborate with external APIs?",
            ],
            index: 0,
            waiting: false,
        }
    }
}

#[async_trait]
impl Agent for CuriousAgent {
    fn id(&self) -> &AgentId { &self.id }
    async fn initialize(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [Curious] {} questions ready!", self.questions.len());
        Ok(())
    }
    async fn execute(&mut self, ctx: &AgentContext) -> AgentResult<()> {
        if !self.waiting && self.index < self.questions.len() {
            println!("\n  [Curious] → \"{}\"", self.questions[self.index]);
            ctx.send_message("thinker", "query", self.questions[self.index]);
            self.waiting = true;
        }
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        Ok(())
    }
    async fn shutdown(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [Curious] Done!");
        Ok(())
    }
    async fn handle_message(&mut self, _ctx: &AgentContext, _s: &str, _p: &str, content: &str) -> AgentResult<()> {
        println!("  [Curious] ← \"{}\"", content);
        self.index += 1;
        self.waiting = false;
        if self.index >= self.questions.len() {
            println!("\n  [Curious] All questions answered! ✓");
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), RuntimeError> {
    println!("=== Cognitive Agent ===\n");

    let mut thinker = CognitiveAgent::from_config("data/beliefs.json", "data/config.json");

    thinker.add_rule(Rule::new("topic:what_is")
        .with_condition("what").with_condition("rustyai").with_condition("about")
        .with_conclusion("what_is_rustyai"));
    thinker.add_rule(Rule::new("topic:patterns")
        .with_condition("pattern").with_condition("support")
        .with_conclusion("patterns"));

    println!("  CognitiveAgent: {} beliefs, {} rules\n", thinker.belief_count(), thinker.rule_count());

    let runtime = Runtime::new();
    runtime.spawn(Box::new(thinker), "thinker").await?;
    runtime.spawn(Box::new(CuriousAgent::new()), "curious").await?;

    tokio::time::sleep(std::time::Duration::from_secs(120)).await;

    runtime.shutdown().await?;
    println!("\n=== Done ===");
    Ok(())
}
