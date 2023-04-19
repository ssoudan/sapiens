use std::collections::HashMap;

use mediawiki::api_sync::ApiSync;
use sapiens::tools::{
    Describe, Format, ProtoToolDescribe, ProtoToolInvoke, ToolDescription, ToolUseError,
};
use sapiens_derive::{Describe, ProtoToolDescribe, ProtoToolInvoke};
use serde::{Deserialize, Serialize};
use serde_json;

/// A Tool to query Wikipedia using SPARQL.
///
/// Wikipedia is a free online encyclopedia, created and edited by volunteers
/// around the world and hosted by the Wikimedia Foundation.
#[derive(ProtoToolInvoke, ProtoToolDescribe)]
#[tool(
    name = "Wikipedia",
    input = "WikipediaToolInput",
    output = "WikipediaToolOutput"
)]
pub struct WikipediaTool {
    client: ApiSync,
}

/// [`WikipediaTool`] input
#[derive(Debug, Deserialize, Serialize, Describe)]
pub struct WikipediaToolInput {
    /// query parameters
    query: HashMap<String, String>,
    /// maximum number of results to return - if not specified, all results are
    /// returned.
    limit: Option<usize>,
}

/// [`WikipediaTool`] output
#[derive(Debug, Deserialize, Serialize, Describe)]
pub struct WikipediaToolOutput {
    /// query result - in JSON.
    result: String,
}

impl Default for WikipediaTool {
    fn default() -> Self {
        Self {
            client: ApiSync::new("https://en.wikipedia.org/w/api.php").unwrap(),
        }
    }
}

impl WikipediaTool {
    fn invoke_typed(
        &self,
        input: &WikipediaToolInput,
    ) -> Result<WikipediaToolOutput, ToolUseError> {
        let result = self
            .client
            .get_query_api_json_limit(&input.query, input.limit)
            .map_err(|e| ToolUseError::ToolInvocationFailed(e.to_string()))?;

        Ok(WikipediaToolOutput {
            result: serde_json::to_string(&result).unwrap(),
        })
    }
}
