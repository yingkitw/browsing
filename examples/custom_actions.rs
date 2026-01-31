//! Example of registering custom actions
//!
//! This example demonstrates how to create and register custom actions
//! that extend the browser automation capabilities.

use browsing::agent::views::ActionResult;
use browsing::error::Result;
use browsing::tools::views::{ActionContext, ActionHandler, ActionParams};
use browsing::tools::Tools;

/// Custom action handler for sending messages
struct SendMessageHandler;

#[async_trait::async_trait]
impl ActionHandler for SendMessageHandler {
    async fn execute(
        &self,
        params: &ActionParams,
        _context: &mut ActionContext<'_>,
    ) -> Result<ActionResult> {
        let message = params.inner()
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("Hello, World!");

        println!("📧 Sending message: {}", message);

        Ok(ActionResult {
            extracted_content: Some(format!("Message sent: {}", message)),
            is_done: Some(false),
            ..Default::default()
        })
    }
}

/// Custom action handler for calculations
struct CalculatorHandler;

#[async_trait::async_trait]
impl ActionHandler for CalculatorHandler {
    async fn execute(
        &self,
        params: &ActionParams,
        _context: &mut ActionContext<'_>,
    ) -> Result<ActionResult> {
        let a = params.get_optional_f64("a").unwrap_or(0.0);
        let b = params.get_optional_f64("b").unwrap_or(0.0);
        let operation = params.inner()
            .get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("add");

        let result = match operation {
            "add" => a + b,
            "subtract" => a - b,
            "multiply" => a * b,
            "divide" if b != 0.0 => a / b,
            _ => 0.0,
        };

        println!("🧮 Calculation: {} {} {} = {}", a, operation, b, result);

        Ok(ActionResult {
            extracted_content: Some(format!("Result: {}", result)),
            is_done: Some(false),
            ..Default::default()
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Custom Actions Example ===\n");

    // Create tools service
    let mut tools = Tools::default();

    // Register custom actions
    tools.register_custom_action(
        "send_message".to_string(),
        "Send a message with custom content".to_string(),
        None, // Available on all domains
        SendMessageHandler,
    );

    tools.register_custom_action(
        "calculate".to_string(),
        "Perform mathematical calculations".to_string(),
        Some(vec!["calculator.com".to_string()]), // Only on calculator.com
        CalculatorHandler,
    );

    println!("✓ Registered custom actions:");
    println!("  - send_message: Send a message with custom content");
    println!("  - calculate: Perform mathematical calculations (calculator.com only)");

    println!("\nCustom actions are now registered and available!");
    println!("They can be used by the agent during autonomous browsing.");
    println!("\nTo use these actions:");
    println!("1. Create an Agent with the tools instance");
    println!("2. The agent will automatically have access to these custom actions");
    println!("3. The LLM can call them like built-in actions");

    Ok(())
}
