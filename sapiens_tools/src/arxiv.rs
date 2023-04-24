use std::fmt::Display;

use arxiv;
use arxiv::{Arxiv, ArxivQuery};
use sapiens::tools::ToolUseError::ToolInvocationFailed;
use sapiens::tools::{Describe, ProtoToolDescribe, ProtoToolInvoke, ToolDescription, ToolUseError};
use sapiens_derive::{Describe, ProtoToolDescribe, ProtoToolInvoke};
use serde::{Deserialize, Serialize};

/// A Tool to query arXiv.
///
/// arXiv is a free distribution service and an open-access archive for
/// millions scholarly articles in the fields of physics, mathematics, computer
/// science, quantitative biology, quantitative finance, statistics, electrical
/// engineering and systems science, and economics. Materials on this site are
/// not peer-reviewed by arXiv.
#[derive(ProtoToolInvoke, ProtoToolDescribe)]
#[tool(name = "ArXiv", input = "ArXivToolInput", output = "ArXivToolOutput")]
pub struct ArXivTool {}

/// Sort order
#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub enum SortOrder {
    /// Ascending
    #[serde(rename = "ascending")]
    Ascending,
    /// Descending
    #[serde(rename = "descending")]
    #[default]
    Descending,
}

impl Display for SortOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SortOrder::Ascending => write!(f, "ascending"),
            SortOrder::Descending => write!(f, "descending"),
        }
    }
}

/// Sort by
#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub enum SortBy {
    /// Relevance
    #[serde(rename = "relevance")]
    #[default]
    Relevance,
    /// Last updated date
    #[serde(rename = "lastUpdatedDate")]
    LastUpdatedDate,
    /// Submitted date
    #[serde(rename = "submittedDate")]
    SubmittedDate,
}

impl Display for SortBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SortBy::Relevance => write!(f, "relevance"),
            SortBy::LastUpdatedDate => write!(f, "lastUpdatedDate"),
            SortBy::SubmittedDate => write!(f, "submittedDate"),
        }
    }
}

/// [`ArXivTool`] input
///
/// ArXiv API documentation query specification
#[derive(Debug, Deserialize, Serialize, Describe)]
pub struct ArXivToolInput {
    /// search_query: Search query - see https://info.arxiv.org/help/api/user-manual.html
    /// for details. E.g. `cs.AI` or `cat:cs.AI` or `au:John Smith`
    /// The fields that can be searched are: `ti` (title), `au` (author), `abs`
    /// (abstract), `co` (comment), `jr` (journal reference), `cat` (subject
    /// category), `rn` (report number), `id` (id (use id_list instead)),
    /// `all` (all of the above). Operators: `AND`, `OR`, `ANDNOT`.
    /// You cannot search on publication or last update date.
    pub search_query: String,

    /// id_list: Comma-separated list of arXiv IDs to return
    pub id_list: Option<String>,

    /// start: Result offset for pagination
    pub start: Option<i32>,

    /// max_results: Maximum number of results to return in a single response.
    /// Default is 10.
    pub max_results: Option<i32>,

    /// Sort by. Can be either `relevance`, `lastUpdatedDate` or
    /// `submittedDate`. Default is `relevance`.
    pub sort_by: Option<SortBy>,

    /// Sort order. Can be either `ascending` or `descending`.
    /// Default is `descending`.
    pub sort_order: Option<SortOrder>,

    /// show PDF url - default is false
    pub show_pdf_url: Option<bool>,

    /// show authors - default is false
    pub show_authors: Option<bool>,

    /// show comments - default is false
    pub show_comments: Option<bool>,

    /// show summary - default is false
    pub show_summary: Option<bool>,
}

impl From<&ArXivToolInput> for ArxivQuery {
    fn from(input: &ArXivToolInput) -> Self {
        ArxivQuery {
            base_url: "https://export.arxiv.org/api/query?".to_string(),
            search_query: input.search_query.clone(),
            id_list: input.id_list.clone().unwrap_or_default(),
            start: input.start,
            max_results: input.max_results,
            sort_by: input.sort_by.clone().unwrap_or_default().to_string(),
            sort_order: input.sort_order.clone().unwrap_or_default().to_string(),
        }
    }
}

/// [`ArXivTool`] output
#[derive(Debug, Deserialize, Serialize, Describe)]
pub struct ArXivToolOutput {
    /// query result - in JSON.
    result: Vec<ArXivResult>,
}

/// ArXiv result
#[derive(Debug, Deserialize, Serialize, Describe)]
pub struct ArXivResult {
    /// arXiv ID
    pub id: String,
    /// last updated date
    pub updated: String,
    /// published date
    pub published: String,
    /// title
    pub title: String,
    /// summary
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// authors
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub authors: Vec<String>,
    /// PDF URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pdf_url: Option<String>,
    /// Comments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

impl From<Arxiv> for ArXivResult {
    fn from(arxiv: Arxiv) -> Self {
        ArXivResult {
            id: arxiv.id,
            updated: arxiv.updated,
            published: arxiv.published,
            title: arxiv.title,
            summary: Some(arxiv.summary),
            authors: arxiv.authors,
            pdf_url: Some(arxiv.pdf_url),
            comment: arxiv.comment,
        }
    }
}

impl ArXivTool {
    /// Create a new [`ArXivTool`]
    pub async fn new() -> ArXivTool {
        ArXivTool {}
    }

    async fn invoke_typed(&self, input: &ArXivToolInput) -> Result<ArXivToolOutput, ToolUseError> {
        let query = ArxivQuery::from(input);
        let result = arxiv::fetch_arxivs(query)
            .await
            .map_err(|e| ToolInvocationFailed(e.to_string()))?;

        let vec = result
            .into_iter()
            .map(|x| x.into())
            .map(|mut x: ArXivResult| {
                if !(input.show_pdf_url.unwrap_or(false)) {
                    x.pdf_url = None;
                }

                if !(input.show_comments.unwrap_or(false)) {
                    x.comment = None;
                }

                if !(input.show_summary.unwrap_or(false)) {
                    x.summary = None;
                }

                if !(input.show_authors.unwrap_or(false)) {
                    x.authors = vec![];
                }

                x
            })
            .collect();

        Ok(ArXivToolOutput { result: vec })
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use insta::assert_yaml_snapshot;

    use super::*;

    #[tokio::test]
    async fn test_arxiv() {
        let tool = ArXivTool::new().await;
        let input = ArXivToolInput {
            search_query: "cat:cs.AI".to_string(),
            id_list: None,
            start: None,
            max_results: None,
            sort_by: Some(SortBy::Relevance),
            sort_order: Some(SortOrder::Ascending),
            show_authors: None,
            show_comments: None,
            show_summary: Some(false),
            show_pdf_url: Some(false),
        };
        let output = tool.invoke_typed(&input).await.unwrap();

        assert_eq!(output.result.len(), 10);
    }

    #[tokio::test]
    async fn test_arxiv_from_yaml() {
        let tool = ArXivTool::new().await;
        let input = indoc! {"
            search_query: cat:cs.AI
            show_authors: true
        "};

        let input: ArXivToolInput = serde_yaml::from_str(input).unwrap();

        assert_yaml_snapshot!(input);

        let output = tool.invoke_typed(&input).await.unwrap();

        assert_eq!(output.result.len(), 10);
        assert!(!output.result[0].authors.is_empty());
    }

    #[tokio::test]
    async fn test_arxiv_from_yaml_2() {
        let tool = ArXivTool::new().await;
        let input = indoc! {"
            search_query: cat:cs.DB
            max_results: 4
            show_authors: true
            show_pdf_url: true  
        "};

        let input: ArXivToolInput = serde_yaml::from_str(input).unwrap();

        assert_yaml_snapshot!(input);

        let output = tool.invoke_typed(&input).await.unwrap();

        assert_eq!(output.result.len(), 4);
        assert!(!output.result[0].authors.is_empty());

        let yaml = serde_yaml::to_value(&output).unwrap();
        assert_yaml_snapshot!(yaml);
    }
}
