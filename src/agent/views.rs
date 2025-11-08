//! Agent view types and data structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration options for the Agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSettings {
    pub use_vision: VisionMode,
    pub vision_detail_level: VisionDetailLevel,
    pub save_conversation_path: Option<String>,
    pub save_conversation_path_encoding: Option<String>,
    pub max_failures: u32,
    pub generate_gif: bool,
    pub override_system_message: Option<String>,
    pub extend_system_message: Option<String>,
    pub include_attributes: Option<Vec<String>>,
    pub max_actions_per_step: u32,
    pub use_thinking: bool,
    pub flash_mode: bool,
    pub use_judge: bool,
    pub max_history_items: Option<u32>,
    pub calculate_cost: bool,
    pub include_tool_call_examples: bool,
    pub llm_timeout: u32,
    pub step_timeout: u32,
    pub final_response_after_failure: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VisionMode {
    Auto,
    Enabled(bool),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VisionDetailLevel {
    Auto,
    Low,
    High,
}

impl Default for AgentSettings {
    fn default() -> Self {
        Self {
            use_vision: VisionMode::Enabled(true),
            vision_detail_level: VisionDetailLevel::Auto,
            save_conversation_path: None,
            save_conversation_path_encoding: Some("utf-8".to_string()),
            max_failures: 3,
            generate_gif: false,
            override_system_message: None,
            extend_system_message: None,
            include_attributes: None,
            max_actions_per_step: 4,
            use_thinking: true,
            flash_mode: false,
            use_judge: true,
            max_history_items: None,
            calculate_cost: false,
            include_tool_call_examples: false,
            llm_timeout: 60,
            step_timeout: 180,
            final_response_after_failure: true,
        }
    }
}

/// Holds all state information for an Agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    pub agent_id: String,
    pub n_steps: u32,
    pub consecutive_failures: u32,
    pub last_result: Option<Vec<ActionResult>>,
    pub last_plan: Option<String>,
    pub last_model_output: Option<AgentOutput>,
    pub paused: bool,
    pub stopped: bool,
    pub session_initialized: bool,
    pub follow_up_task: bool,
}

impl Default for AgentState {
    fn default() -> Self {
        Self {
            agent_id: uuid::Uuid::now_v7().to_string(),
            n_steps: 1,
            consecutive_failures: 0,
            last_result: None,
            last_plan: None,
            last_model_output: None,
            paused: false,
            stopped: false,
            session_initialized: false,
            follow_up_task: false,
        }
    }
}

/// Information about a single step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStepInfo {
    pub step_number: u32,
    pub max_steps: u32,
}

impl AgentStepInfo {
    pub fn is_last_step(&self) -> bool {
        self.step_number >= self.max_steps - 1
    }
}

/// LLM judgement of agent trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgementResult {
    pub reasoning: Option<String>,
    pub verdict: bool,
    pub failure_reason: Option<String>,
    pub impossible_task: bool,
    pub reached_captcha: bool,
}

/// Result of executing an action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    pub is_done: Option<bool>,
    pub success: Option<bool>,
    pub judgement: Option<JudgementResult>,
    pub error: Option<String>,
    pub attachments: Option<Vec<String>>,
    pub images: Option<Vec<ImageData>>,
    pub long_term_memory: Option<String>,
    pub extracted_content: Option<String>,
    pub include_extracted_content_only_once: bool,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageData {
    pub name: String,
    pub data: String, // base64 encoded
}

impl Default for ActionResult {
    fn default() -> Self {
        Self {
            is_done: Some(false),
            success: None,
            judgement: None,
            error: None,
            attachments: None,
            images: None,
            long_term_memory: None,
            extracted_content: None,
            include_extracted_content_only_once: false,
            metadata: None,
        }
    }
}

/// Metadata for a single step including timing and token information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepMetadata {
    pub step_start_time: f64,
    pub step_end_time: f64,
    pub step_number: u32,
}

impl StepMetadata {
    pub fn duration_seconds(&self) -> f64 {
        self.step_end_time - self.step_start_time
    }
}

/// Agent's reasoning process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentBrain {
    pub thinking: Option<String>,
    pub evaluation_previous_goal: String,
    pub memory: String,
    pub next_goal: String,
}

/// Agent output from LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOutput {
    pub thinking: Option<String>,
    pub evaluation_previous_goal: Option<String>,
    pub memory: Option<String>,
    pub next_goal: Option<String>,
    pub action: Vec<serde_json::Value>, // ActionModel - will be properly typed later
}

impl AgentOutput {
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

impl AgentOutput {
    pub fn current_state(&self) -> AgentBrain {
        AgentBrain {
            thinking: self.thinking.clone(),
            evaluation_previous_goal: self.evaluation_previous_goal.clone().unwrap_or_default(),
            memory: self.memory.clone().unwrap_or_default(),
            next_goal: self.next_goal.clone().unwrap_or_default(),
        }
    }
}

/// History item for agent actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentHistory {
    pub model_output: Option<AgentOutput>,
    pub result: Vec<ActionResult>,
    pub state: crate::browser::views::BrowserStateHistory,
    pub metadata: Option<StepMetadata>,
    pub state_message: Option<String>,
}

/// List of AgentHistory messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentHistoryList {
    pub history: Vec<AgentHistory>,
    pub usage: Option<crate::tokens::views::UsageSummary>,
}

impl AgentHistoryList {
    pub fn total_duration_seconds(&self) -> f64 {
        self.history
            .iter()
            .filter_map(|h| h.metadata.as_ref())
            .map(|m| m.duration_seconds())
            .sum()
    }

    pub fn number_of_steps(&self) -> usize {
        self.history.len()
    }

    pub fn is_done(&self) -> bool {
        self.history
            .last()
            .and_then(|h| h.result.last())
            .and_then(|r| r.is_done)
            .unwrap_or(false)
    }

    pub fn is_successful(&self) -> Option<bool> {
        self.history
            .last()
            .and_then(|h| h.result.last())
            .and_then(|r| r.success)
    }

    pub fn has_errors(&self) -> bool {
        self.history
            .iter()
            .any(|h| h.result.iter().any(|r| r.error.is_some()))
    }
}
