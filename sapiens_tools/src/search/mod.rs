use std::fmt::Debug;

use reqwest::{Client, Response, Url};
use sapiens::tools::{Describe, ProtoToolDescribe, ProtoToolInvoke, ToolDescription, ToolUseError};
use sapiens_derive::{Describe, ProtoToolDescribe, ProtoToolInvoke};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::error;

use crate::search::gce::SearchResultNumber::Four;
use crate::search::gce::{ErrorBody, Item, Lr, QueryParameters, SearchResults};

/// A Tool to search the web - powered by Google Custom Search Engine.
#[cfg(feature = "search")]
pub mod gce;

/// A Tool to search the web - powered by Google Custom Search Engine.
///
/// Returns a list of [`Item`] with: `title`, `link`, `snippet`.
#[derive(ProtoToolInvoke, ProtoToolDescribe)]
#[tool(
    name = "Search",
    input = "SearchToolInput",
    output = "SearchToolOutput"
)]
pub struct SearchTool {
    /// API key to use
    api_key: String,
    /// CSE ID to use
    cse_id: String,
    /// HTTP client
    client: Mutex<Client>,
}

impl Debug for SearchTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SearchTool").finish()
    }
}

/// [`SearchTool`] input
#[derive(Debug, Deserialize, Serialize, Describe)]
pub struct SearchToolInput {
    /// query to search. `q` parameter of the Google Custom Search Engine API
    /// Use `exclude_terms` and `exact_terms` to refine your search.
    q: String,

    /// word or phrase to exclude from search
    exclude_terms: Option<String>,

    /// word or phrase that must be in the search results
    exact_terms: Option<String>,

    /// number of results to return (max 10, default 4)
    num: Option<u32>,

    /// language restriction (default 'lang_en') - see https://developers.google.com/custom-search/v1/reference/rest/v1/cse/list#query-parameters
    lr: Option<String>,

    /// start index (default 1)
    start_index: Option<u32>,
}

/// [`SearchTool`] output
#[derive(Debug, Deserialize, Serialize, Describe)]
pub struct SearchToolOutput {
    /// result items. An [`Item`] has the following format: `{'title':
    /// '...', 'link': '...', 'snippet': '...'}`
    results: Vec<Item>,
    /// Next page start index if any
    next_start_index: Option<u32>,
}

impl From<&SearchToolInput> for QueryParameters {
    fn from(value: &SearchToolInput) -> Self {
        let mut q = QueryParameters::builder();

        let q = q
            .q(&value.q)
            .num(value.num.and_then(|x| x.try_into().ok()).unwrap_or(Four))
            .lr(value
                .lr
                .as_ref()
                .and_then(|x| x.as_str().try_into().ok())
                .unwrap_or(Lr::LangEn))
            .start(value.start_index.unwrap_or(1));

        if let Some(exclude_terms) = &value.exclude_terms {
            q.exclude_terms(exclude_terms);
        }

        if let Some(exact_terms) = &value.exact_terms {
            q.exact_terms(exact_terms);
        }

        q.build()
    }
}

impl Default for SearchTool {
    fn default() -> Self {
        let api_key = std::env::var("GOOGLE_API_KEY").expect("GOOGLE_API_KEY env not set");
        let cse_id = std::env::var("GOOGLE_CSE_ID").expect("GOOGLE_CSE_ID env not set");

        SearchTool {
            api_key,
            cse_id,
            client: Mutex::new(Client::builder().build().unwrap()),
        }
    }
}

impl SearchTool {
    /// Create a new [`SearchTool`]
    ///
    /// # Arguments
    ///
    /// * `api_key` - API key to use
    /// * `cse_id` - CSE ID to use
    pub async fn new(api_key: String, cse_id: String) -> SearchTool {
        let client = Client::builder().build().unwrap();

        SearchTool {
            api_key,
            cse_id,
            client: Mutex::new(client),
        }
    }

    #[tracing::instrument]
    async fn invoke_typed(
        &self,
        input: &SearchToolInput,
    ) -> Result<SearchToolOutput, ToolUseError> {
        let query_params = QueryParameters::from(input);

        let resp = self.do_query(query_params).await;

        let resp = resp.map_err(|e| ToolUseError::ToolInvocationFailed(e.to_string()))?;

        if resp.status().is_success() {
            let body = resp
                .text()
                .await
                .map_err(|e| ToolUseError::ToolInvocationFailed(e.to_string()))?;

            let resp = match serde_json::from_str::<SearchResults>(&body) {
                Ok(resp) => resp,
                Err(e) => {
                    error!(body = body, "Error parsing response: {}", e);

                    return Err(ToolUseError::ToolInvocationFailed(e.to_string()));
                }
            };

            // let resp = resp
            //     .json::<SearchResults>()
            //     .await
            //     .map_err(|e| ToolUseError::ToolInvocationFailed(e.to_string()))?;

            let next_start_index = resp.next_page().map(|x| x.start);

            Ok(SearchToolOutput {
                results: resp.items,
                next_start_index,
            })
        } else {
            let code = resp.status();

            let body = resp
                .json::<ErrorBody>()
                .await
                .map_err(|e| ToolUseError::ToolInvocationFailed(e.to_string()))?;

            Err(ToolUseError::ToolInvocationFailed(format!(
                "Error code {}: {}",
                code, body.error.message
            )))
        }
    }

    async fn do_query(&self, mut query_params: QueryParameters) -> Result<Response, ToolUseError> {
        let url =
            Url::parse(gce::URL).map_err(|e| ToolUseError::ToolInvocationFailed(e.to_string()))?;

        let query_params = query_params.key(&self.api_key).cx(&self.cse_id);

        let resp = {
            let guard = self.client.lock().await;

            guard
                .get(url.clone())
                .header("Accept", "application/json")
                .query(&query_params.to_parameters())
                .send()
                .await
        };

        resp.map_err(|e| ToolUseError::ToolInvocationFailed(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_search_tool() {
        let _ = dotenvy::dotenv();

        let tool = SearchTool::default();

        let input = SearchToolInput {
            q: "Best Rust programming language website".to_string(),
            exclude_terms: None,
            exact_terms: None,
            num: Some(3),
            lr: None,
            start_index: Some(1),
        };

        let output = tool.invoke_typed(&input).await.unwrap();

        // assert_yaml_snapshot!(output);

        assert_eq!(output.results.len(), 3);
    }

    #[tokio::test]
    async fn test_search_tool_2() {
        let _ = dotenvy::dotenv();

        let tool = SearchTool::default();

        let input = SearchToolInput {
            q: "Alain Prost -motorcycle".to_string(),
            exclude_terms: None,
            exact_terms: None,
            num: Some(1),
            lr: None,
            start_index: None,
        };

        let query_params = QueryParameters::from(&input);

        let resp = tool.do_query(query_params).await.unwrap();

        println!("{:#?}", resp);

        let body = resp.text().await.unwrap();
        println!("{}", body);
    }
}
