use mediawiki::api::Api;
use sapiens::tools::{
    Describe, Format, ProtoToolDescribe, ProtoToolInvoke, ToolDescription, ToolUseError,
};
use sapiens_derive::{Describe, ProtoToolDescribe, ProtoToolInvoke};
use serde::{Deserialize, Serialize};
use serde_json;

/// A Tool to query Wikidata using SPARQL.
///
/// Wikidata is a free and open knowledge base that can be read and edited by
/// both humans and machines.
///
/// Wikidata acts as central storage for the structured data of its Wikimedia
/// sister projects including Wikipedia, Wikivoyage, Wiktionary, Wikisource, and
/// others.
#[derive(ProtoToolInvoke, ProtoToolDescribe)]
#[tool(
    name = "Wikidata",
    input = "WikidataToolInput",
    output = "WikidataToolOutput"
)]
pub struct WikidataTool {
    client: Api,
}

/// [`WikidataTool`] input
#[derive(Debug, Deserialize, Serialize, Describe)]
pub struct WikidataToolInput {
    /// SPARQL query to execute.
    query: String,
}

/// [`WikidataTool`] output
#[derive(Debug, Deserialize, Serialize, Describe)]
pub struct WikidataToolOutput {
    /// SPARQL query result - in JSON.
    result: String,
}

impl WikidataTool {
    /// Create a new [`WikidataTool`]
    pub async fn new() -> Self {
        let client = Api::new("https://www.wikidata.org/w/api.php")
            .await
            .unwrap();
        Self { client }
    }

    async fn invoke_typed(
        &self,
        input: &WikidataToolInput,
    ) -> Result<WikidataToolOutput, ToolUseError> {
        let result = self
            .client
            .sparql_query(&input.query)
            .await
            .map_err(|e| ToolUseError::ToolInvocationFailed(e.to_string()))?;

        Ok(WikidataToolOutput {
            result: serde_json::to_string(&result).unwrap(),
        })
    }
}
