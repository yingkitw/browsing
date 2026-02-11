//! Agent service implementation

use crate::agent::json_extractor::JSONExtractor;
use crate::agent::views::{
    ActionResult, AgentHistory, AgentHistoryList, AgentOutput, AgentSettings, AgentState,
};
use crate::error::{BrowsingError, Result};
use crate::llm::base::{ChatMessage, ChatModel};
use crate::traits::{BrowserClient, DOMProcessor};
use crate::tools::Tools;
use crate::tools::views::ActionModel;
use serde_json::Value;
use tracing::info;

/// Agent for autonomous web automation
pub struct Agent<L: ChatModel> {
    task: String,
    browser: Box<dyn BrowserClient>,
    llm: L,
    tools: Tools,
    dom_processor: Box<dyn DOMProcessor>,
    max_steps: u32,
    settings: AgentSettings,
    state: AgentState,
    history: AgentHistoryList,
    usage_tracker: UsageTracker,
}

/// Simple usage tracker that aggregates token counts
struct UsageTracker {
    total_prompt_tokens: u32,
    total_completion_tokens: u32,
    total_tokens: u32,
}

impl UsageTracker {
    fn new() -> Self {
        Self {
            total_prompt_tokens: 0,
            total_completion_tokens: 0,
            total_tokens: 0,
        }
    }

    fn add_usage(&mut self, usage: &crate::llm::base::ChatInvokeUsage) {
        self.total_prompt_tokens += usage.prompt_tokens;
        self.total_completion_tokens += usage.completion_tokens;
        self.total_tokens += usage.total_tokens;
    }

    fn to_summary(&self) -> crate::tokens::views::UsageSummary {
        crate::tokens::views::UsageSummary {
            prompt_tokens: Some(self.total_prompt_tokens),
            completion_tokens: Some(self.total_completion_tokens),
            total_tokens: Some(self.total_tokens),
            cost: None, // Cost calculation can be added later
        }
    }
}

impl<L: ChatModel> Agent<L> {
    /// Create a new Agent with the specified task, browser, DOM processor, and LLM
    pub fn new(
        task: String,
        browser: Box<dyn BrowserClient>,
        dom_processor: Box<dyn DOMProcessor>,
        llm: L,
    ) -> Self {
        Self {
            task: task.clone(),
            browser,
            llm,
            tools: Tools::default(),
            dom_processor,
            max_steps: 100,
            settings: AgentSettings::default(),
            state: AgentState::default(),
            history: AgentHistoryList {
                history: vec![],
                usage: None,
            },
            usage_tracker: UsageTracker::new(),
        }
    }

    /// Set the maximum number of steps the agent will take
    pub fn with_max_steps(mut self, max_steps: u32) -> Self {
        self.max_steps = max_steps;
        self
    }

    /// Set agent configuration settings
    pub fn with_settings(mut self, settings: AgentSettings) -> Self {
        self.settings = settings;
        self
    }

    /// Run the agent to complete the task
    pub async fn run(&mut self) -> Result<AgentHistoryList> {
        // Start browser
        self.browser.start().await?;

        // Initialize DOM processor with browser's CDP client
        let cdp_client = self.browser.get_cdp_client()?;
        let session_info = self.browser.get_session_info().await?;

        // Create a new DOM processor with the CDP client and target ID
        let dom_processor = Box::new(
            crate::dom::DOMProcessorImpl::new()
                .with_cdp_client(cdp_client, session_info.session_id)
                .with_target_id(session_info.target_id)
        );
        self.dom_processor = dom_processor;

        // Extract URL from task if present
        let initial_url = crate::utils::extract_urls(&self.task).first().cloned();

        // Navigate to initial URL if found
        if let Some(url) = initial_url {
            self.browser.navigate(&url).await?;
        }

        // Set up signal handler for graceful shutdown
        let signal_handler = crate::utils::signal::SignalHandler::new();
        let _shutdown_listener = signal_handler.spawn_shutdown_listener();

        // Main execution loop
        for step in 0..self.max_steps {
            // Check for shutdown request
            if signal_handler.is_shutdown_requested()
                || crate::utils::signal::is_shutdown_requested()
            {
                info!("🛑 Shutdown requested, stopping agent execution");
                break;
            }

            self.state.n_steps = step + 1;

            // Get page state
            let page_state = self.get_page_state().await?;

            // Build messages for LLM
            let messages = self.build_messages(&page_state)?;

            // Get next action from LLM
            let response = self.llm.chat(&messages).await?;

            // Track token usage if available
            if let Some(ref usage) = response.usage {
                self.track_usage(usage);
            }

            // Parse AgentOutput from LLM response
            let agent_output = self.parse_agent_output(&response.completion)?;

            // Execute actions
            let mut results = vec![];
            for action_value in &agent_output.action {
                // Convert serde_json::Value to ActionModel
                let action: ActionModel =
                    serde_json::from_value(action_value.clone()).map_err(|e| {
                        BrowsingError::Agent(format!("Failed to parse action: {e}"))
                    })?;

                match self.execute_action(&action).await {
                    Ok(result) => results.push(result),
                    Err(e) => {
                        results.push(ActionResult {
                            error: Some(e.to_string()),
                            ..Default::default()
                        });
                    }
                }
            }

            // Record step in history
            let history_item = AgentHistory {
                model_output: Some(agent_output.clone()),
                result: results.clone(),
                state: crate::browser::views::BrowserStateHistory {
                    url: self.browser.get_current_url().await.unwrap_or_default(),
                    title: "Unknown".to_string(),
                    tabs: vec![],
                    interacted_element: vec![],
                    screenshot_path: None,
                },
                metadata: None,
                state_message: None,
            };
            self.history.history.push(history_item);

            // Check if task is complete
            if self.is_task_complete(&results) {
                break;
            }
        }

        // Update history with final usage summary
        self.history.usage = Some(self.usage_tracker.to_summary());

        // Gracefully close browser session
        if let Err(e) = self.browser.stop().await {
            info!("⚠ Browser stop warning: {e}");
        }

        Ok(self.history.clone())
    }

    /// Track token usage from an LLM response
    fn track_usage(&mut self, usage: &crate::llm::base::ChatInvokeUsage) {
        self.usage_tracker.add_usage(usage);
    }

    async fn get_page_state(&self) -> Result<String> {
        // Get page state from DOM processor
        self.dom_processor.get_page_state_string().await
    }

    fn build_messages(&self, page_state: &str) -> Result<Vec<ChatMessage>> {
        let mut messages = vec![];

        // System message
        if let Some(ref system_prompt) = self.settings.override_system_message {
            messages.push(ChatMessage::system(system_prompt.clone()));
        } else {
            // Default system prompt
            messages.push(ChatMessage::system(
                "You are a browser automation agent. Help the user complete their task."
                    .to_string(),
            ));
        }

        // Add task
        messages.push(ChatMessage::user(format!(
            "Task: {}\n\nPage state:\n{}",
            self.task, page_state
        )));

        Ok(messages)
    }

    fn parse_agent_output(&self, response: &str) -> Result<AgentOutput> {
        // Use JSONExtractor to extract JSON from response
        let extractor = JSONExtractor::new();
        let json_str = extractor.extract_from_response(response);

        tracing::debug!("Raw LLM response: {}", response);
        tracing::debug!("Extracted JSON: {}", json_str);

        // Try to repair JSON if needed using anyrepair
        // First try to parse directly, if that fails, try to repair
        let repaired = match serde_json::from_str::<Value>(&json_str) {
            Ok(_) => json_str.clone(), // Already valid JSON
            Err(e) => {
                tracing::debug!("JSON parse error, attempting repair: {}", e);
                // Try to repair using anyrepair
                anyrepair::repair(&json_str).unwrap_or_else(|_| json_str.clone())
            }
        };

        tracing::debug!("Repaired JSON: {}", repaired);

        // Parse JSON
        let value: Value = serde_json::from_str(&repaired)
            .map_err(|e| BrowsingError::Agent(format!("Failed to parse agent output: {e}")))?;

        // Convert to AgentOutput
        let agent_output = serde_json::from_value(value.clone()).map_err(|e| {
            tracing::error!("Failed to deserialize agent output. Value: {}", value);
            BrowsingError::Agent(format!("Failed to deserialize agent output: {e}"))
        })?;

        Ok(agent_output)
    }

    async fn execute_action(&mut self, action: &ActionModel) -> Result<ActionResult> {
        // Get selector map from DOM processor
        let selector_map = self.dom_processor.get_selector_map().await.ok();

        // Execute action via tools
        self.tools
            .act(action.clone(), &mut *self.browser, selector_map.as_ref())
            .await
    }

    fn is_task_complete(&self, results: &[ActionResult]) -> bool {
        // Check if any result indicates task is done
        results.iter().any(|r| r.is_done == Some(true))
    }
}
