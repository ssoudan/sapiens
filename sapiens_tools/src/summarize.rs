use std::fmt::Debug;

use async_openai::types::{CreateCompletionRequest, Prompt};
use async_openai::Client;
use sapiens::tools::{Describe, ProtoToolDescribe, ProtoToolInvoke, ToolDescription, ToolUseError};
use sapiens_derive::{Describe, ProtoToolDescribe, ProtoToolInvoke};
use serde::{Deserialize, Serialize};

/// Text summarization tool
#[derive(ProtoToolDescribe, ProtoToolInvoke)]
#[tool(
    name = "Summarize",
    input = "SummarizeToolInput",
    output = "SummarizeToolOutput"
)]
pub struct SummarizeTool {
    openai_client: Client,
    model: String,
}

impl SummarizeTool {
    /// Create a new SummarizeTool
    pub fn with_model(openai_client: Client, model: String) -> Self {
        Self {
            openai_client,
            model,
        }
    }

    /// Create a new SummarizeTool with the default model
    pub fn new(openai_client: Client) -> Self {
        Self::with_model(openai_client, "text-babbage-001".to_string())
    }
}

impl Default for SummarizeTool {
    fn default() -> Self {
        Self {
            openai_client: Client::default(),
            model: "text-babbage-001".to_string(),
        }
    }
}

/// A tool that is called to test stuffs
#[derive(Debug, Serialize, Deserialize, Describe)]
pub struct SummarizeToolInput {
    /// The text to summarize (max 2000 characters)
    pub text: String,
}

/// SummarizeToolOutput not very significant
#[derive(Serialize, Deserialize, Describe)]
pub struct SummarizeToolOutput {
    /// The summary
    pub summary: String,
}

impl SummarizeTool {
    async fn invoke_typed(
        &self,
        input: &SummarizeToolInput,
    ) -> Result<SummarizeToolOutput, ToolUseError> {
        let prompt = Some(Prompt::String(format!("{}\n\nTl;dr", input.text)));

        if input.text.len() < 100 {
            return Ok(SummarizeToolOutput {
                summary: input.text.clone(),
            });
        }

        if input.text.len() > 2000 {
            return Err(ToolUseError::ToolInvocationFailed(
                "Text too long - limit is 2000.".to_string(),
            ));
        }

        let response = self
            .openai_client
            .completions()
            .create(CreateCompletionRequest {
                prompt,
                model: self.model.clone(),
                ..Default::default()
            })
            .await
            .map_err(|e| ToolUseError::ToolInvocationFailed(e.to_string()))?;

        let summary = response.choices[0].text.clone();
        Ok(SummarizeToolOutput { summary })
    }
}