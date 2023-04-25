use std::collections::HashMap;

use mediawiki::api::Api;
use sapiens::tools::{Describe, ProtoToolDescribe, ProtoToolInvoke, ToolDescription, ToolUseError};
use sapiens_derive::{Describe, ProtoToolDescribe, ProtoToolInvoke};
use serde::{Deserialize, Serialize};
use serde_json;
use serde_yaml::Value;

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
    client: Api,
}

/// [`WikipediaTool`] input
#[derive(Debug, Deserialize, Serialize, Describe)]
pub struct WikipediaToolInput {
    /// query parameters. E.g.
    /// ```yaml
    ///   parameters:
    ///     action: query
    ///     prop:
    ///       - extracts
    ///       - exintro
    ///       - explaintext
    ///     titles: Albert Einstein
    /// ```
    /// - Values can be either strings or numbers. Or lists of them.
    /// - The output size is limited. Be specific and use limits where possible.
    parameters: HashMap<String, Value>,
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

impl WikipediaTool {
    /// Create a new [`WikipediaTool`]
    pub async fn new() -> WikipediaTool {
        let client = Api::new("https://en.wikipedia.org/w/api.php")
            .await
            .unwrap();

        WikipediaTool { client }
    }

    async fn invoke_typed(
        &self,
        input: &WikipediaToolInput,
    ) -> Result<WikipediaToolOutput, ToolUseError> {
        let query: HashMap<String, String> = input
            .parameters
            .clone()
            .into_iter()
            .map(|(k, v)| match v {
                Value::Sequence(s) => Ok((
                    k.clone(),
                    s.into_iter()
                        .map(|v| match v {
                            Value::String(s) => Ok(s),
                            Value::Number(n) => Ok(n.to_string()),
                            _ => Err(ToolUseError::ToolInvocationFailed(format!(
                                "Unsupported value type for parameter: {:?}. Only <str> or <number>
        and list of them supported.",
                                k
                            ))),
                        })
                        .collect::<Result<Vec<_>, _>>()?
                        .join("|"),
                )),
                Value::String(s) => Ok((k, s)),
                Value::Number(n) => Ok((k, n.to_string())),
                _ => Err(ToolUseError::ToolInvocationFailed(format!(
                    "Unsupported value type for parameter: {:?}. Only <str>
        or <number> and list of them supported.",
                    k
                ))),
            })
            .collect::<Result<_, _>>()?;

        let result = self
            .client
            .get_query_api_json_limit(&query, input.limit)
            .await
            .map_err(|e| ToolUseError::ToolInvocationFailed(e.to_string()))?;

        Ok(WikipediaToolOutput {
            result: serde_json::to_string(&result).unwrap(),
        })
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use insta::assert_yaml_snapshot;

    use super::*;

    #[tokio::test]
    async fn test_wikipedia_tool_description() {
        let tool = WikipediaTool::new().await;

        let description = tool.description();

        assert_yaml_snapshot!(description);
    }

    #[tokio::test]
    async fn test_wikipedia_tool() {
        let mut settings = insta::Settings::clone_current();
        settings.set_sort_maps(true);
        settings
            .bind_async(async {
                let tool = WikipediaTool::new().await;
                let input = WikipediaToolInput {
                    parameters: vec![
                        ("action".to_string(), Value::String("query".to_string())),
                        (
                            "prop".to_string(),
                            Value::Sequence(vec![
                                Value::String("extracts".to_string()),
                                Value::String("exintro".to_string()),
                                Value::String("explaintext".to_string()),
                            ]),
                        ),
                        (
                            "titles".to_string(),
                            Value::String("Albert Einstein".to_string()),
                        ),
                    ]
                    .into_iter()
                    .collect(),
                    limit: None,
                };
                let input = serde_yaml::to_string(&input).unwrap();
                let input = serde_yaml::from_str::<WikipediaToolInput>(&input).unwrap();

                assert_yaml_snapshot!(input);

                let _output = tool.invoke_typed(&input).await.unwrap();
            })
            .await;
    }

    #[tokio::test]
    async fn test_wikipedia_tool_from_yaml() {
        let tool = WikipediaTool::new().await;

        let input = indoc! {
            r#"
               parameters:
                 action: query
                 prop:
                   - extracts
                   - exintro
                   - explaintext
                 titles: Albert Einstein
            "#
        };
        let input = serde_yaml::from_str::<WikipediaToolInput>(input).unwrap();

        let _output = tool.invoke_typed(&input).await.unwrap();
    }

    #[tokio::test]
    async fn test_wikipedia_input_format() {
        let mut settings = insta::Settings::clone_current();
        settings.set_sort_maps(true);
        settings
            .bind_async(async {
                let input = WikipediaToolInput {
                    parameters: vec![
                        ("action".to_string(), Value::String("query".to_string())),
                        (
                            "prop".to_string(),
                            Value::Sequence(vec![
                                Value::String("extracts".to_string()),
                                Value::String("exintro".to_string()),
                                Value::String("explaintext".to_string()),
                            ]),
                        ),
                        (
                            "titles".to_string(),
                            Value::String("Albert Einstein".to_string()),
                        ),
                    ]
                    .into_iter()
                    .collect(),
                    limit: None,
                };
                let input = serde_yaml::to_string(&input).unwrap();
                let input = serde_yaml::from_str::<WikipediaToolInput>(&input).unwrap();

                assert_yaml_snapshot!(input);
            })
            .await;
    }
}
