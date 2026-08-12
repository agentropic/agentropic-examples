//! Holarchy: Nested autonomous units. Parent delegates, children act independently.
use agent_core::{Agent, AgentContext, AgentId, AgentResult};
use patterns::holarchy::{Holarchy, Holon};
use runtime::prelude::*;
use async_trait::async_trait;

struct HolonAgent { id: AgentId, name: String, is_composite: bool, delegated: bool }
impl HolonAgent {
    fn new(name: &str, is_composite: bool) -> Self {
        Self { id: AgentId::new(), name: name.to_string(), is_composite, delegated: false }
    }
}

#[async_trait]
impl Agent for HolonAgent {
    fn id(&self) -> &AgentId { &self.id }
    async fn initialize(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        let kind = if self.is_composite { "composite" } else { "atomic" };
        println!("  [{}] Online ({})", self.name, kind);
        Ok(())
    }
    async fn execute(&mut self, ctx: &AgentContext) -> AgentResult<()> {
        if self.is_composite && !self.delegated {
            match self.name.as_str() {
                "Company" => {
                    println!("  [Company] ↓ Delegating to departments...");
                    ctx.send_message("engineering", "request", "Build the product");
                    ctx.send_message("marketing", "request", "Launch the campaign");
                    self.delegated = true;
                }
                "Engineering" => {
                    println!("  [Engineering] ↓ Delegating to teams...");
                    ctx.send_message("frontend", "request", "Build the UI");
                    ctx.send_message("backend", "request", "Build the API");
                    self.delegated = true;
                }
                _ => {}
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        Ok(())
    }
    async fn shutdown(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        println!("  [{}] Shutdown.", self.name);
        Ok(())
    }
    async fn handle_message(&mut self, ctx: &AgentContext, _s: &str, _p: &str, content: &str) -> AgentResult<()> {
        println!("  [{}] ← \"{}\"", self.name, content);

        if self.is_composite {
            // Composite holons delegate further (handled in execute)
        } else {
            // Atomic holons do the work
            tokio::time::sleep(std::time::Duration::from_millis(400)).await;
            let result = format!("{} done: {}", self.name, content);
            println!("  [{}] ✓ \"{}\"", self.name, result);
            // Report up
            ctx.send_message("company", "inform", &result);
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), RuntimeError> {
    println!("╔═══════════════════════════════════════════════╗");
    println!("║   RustyAI — Holarchy Pattern               ║");
    println!("║   Nested autonomous units                     ║");
    println!("╚═══════════════════════════════════════════════╝\n");

    let company_id = AgentId::new();
    let eng_id = AgentId::new();
    let mkt_id = AgentId::new();
    let fe_id = AgentId::new();
    let be_id = AgentId::new();

    let mut holarchy = Holarchy::new("TechCorp");
    let mut company = Holon::composite(company_id);
    company.add_child(eng_id);
    company.add_child(mkt_id);

    let mut eng = Holon::composite(eng_id);
    eng.set_parent(company_id);
    eng.add_child(fe_id);
    eng.add_child(be_id);

    let mut mkt = Holon::atomic(mkt_id);
    mkt.set_parent(company_id);

    let mut fe = Holon::atomic(fe_id);
    fe.set_parent(eng_id);

    let mut be = Holon::atomic(be_id);
    be.set_parent(eng_id);

    holarchy.add_holon(company);
    holarchy.add_holon(eng);
    holarchy.add_holon(mkt);
    holarchy.add_holon(fe);
    holarchy.add_holon(be);

    println!("  Holarchy: \"{}\" ({} holons)", holarchy.name(), holarchy.size());
    println!("  Company → [Engineering → [Frontend, Backend], Marketing]\n");

    let runtime = Runtime::new();
    runtime.spawn(Box::new(HolonAgent::new("Frontend", false)), "frontend").await?;
    runtime.spawn(Box::new(HolonAgent::new("Backend", false)), "backend").await?;
    runtime.spawn(Box::new(HolonAgent::new("Marketing", false)), "marketing").await?;
    runtime.spawn(Box::new(HolonAgent::new("Engineering", true)), "engineering").await?;
    runtime.spawn(Box::new(HolonAgent::new("Company", true)), "company").await?;

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    runtime.shutdown().await?;
    println!("\n✓ Holarchy demo complete.");
    Ok(())
}
