use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::tools::ToolInvocationInput;

/// Error while extracting tool invocations
#[derive(Debug, thiserror::Error, Clone, Serialize, Deserialize)]
pub enum Error {
    /// Invalid yaml
    #[error("Invalid yaml: {0}")]
    InvalidYaml(String),
    /// No invocation found in the document
    #[error("No Action found")]
    NoInvocationFound,
    /// No valid invocation found in the document
    #[error("No valid Action found: {0}")]
    NoValidInvocationFound(String),
    /// Too many yaml blocks
    #[error("Too many ({0}) yaml blocks. Only one is expected.")]
    TooManyYamlBlocks(usize),
}

/// One of several T
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Invocation<T> {
    Single(T),
    Multiple(Vec<T>),
}

/// extract on or several T from a string.
///
/// `data` can be a single yaml of T or of a Vec<T> or a list of yaml
/// documents separated by `---`.
fn extract_from_yaml<T>(data: &str) -> Result<Vec<T>, Error>
where
    T: DeserializeOwned,
{
    let mut invocations = vec![];

    for doc in serde_yaml::Deserializer::from_str(data) {
        // is it a list of T or a single T?
        let attempt: Result<Invocation<T>, _> = Deserialize::deserialize(doc);
        match attempt {
            Ok(invocation) => match invocation {
                Invocation::Single(t) => {
                    invocations.push(t);
                }
                Invocation::Multiple(ts) => {
                    invocations.extend(ts);
                }
            },
            Err(e) => {
                debug!(error = %e, "Failed to deserialize as a list of T or a single T");
                return Err(Error::InvalidYaml(e.to_string()));
            }
        }
    }

    if invocations.is_empty() {
        Err(Error::NoInvocationFound)
    } else {
        Ok(invocations)
    }
}

/// Extracted invocations
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ExtractedInvocations {
    pub(crate) invocations: Vec<ToolInvocationInput>,
    pub(crate) yaml_block_count: usize,
}

/// Find all the invocations in a markdown document.
pub(crate) fn find_all(data: &str) -> Result<ExtractedInvocations, Error> {
    let mut err: Option<Error> = None;

    let mut invocations = vec![];
    let mut yaml_block_count = 0;

    let mut lines = data.lines();

    while let Some(line) = lines.next() {
        if line.trim().is_empty() {
            continue;
        }

        // we have start of a yaml block
        if line.trim().starts_with("```yaml") {
            // collect the lines until the end of the yaml block
            let mut yaml = vec![];

            for line in lines.by_ref() {
                if line.trim().starts_with("```") {
                    break;
                }

                yaml.push(line);
            }

            // put them together
            let yaml = yaml.join("\n");

            yaml_block_count += 1;

            // does that make valid invocations?
            match extract_from_yaml(&yaml) {
                Ok(more) => {
                    invocations.extend(more);
                }
                Err(e) => {
                    // debug!(error = %e, "Failed to extract invocation from yaml");

                    err = Some(e);
                }
            }
        }
    }

    if invocations.is_empty() {
        err.map_or_else(|| Err(Error::NoInvocationFound), Err)
    } else {
        Ok(ExtractedInvocations {
            invocations,
            yaml_block_count,
        })
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use insta::{assert_snapshot, assert_yaml_snapshot};
    use serde_yaml::Number;

    #[tokio::test]
    async fn test_find_invocations_one_yaml() {
        let data = indoc! {r"# Some text
    ```yaml
    tool_name: Search
    parameters:
      q: Marcel Deneuve
      excluded_terms: Resident Evil
      num_results: 10
    ```        
    Some other text
    "};

        let invocations = super::find_all(data).unwrap();

        assert_eq!(invocations.invocations.len(), 1);
    }

    #[tokio::test]
    async fn test_find_multiple_invocations() {
        let data = indoc! {r"# Some text
    ```yaml
    tool_name: Search1
    parameters:
      q: Marcel Deneuve
      excluded_terms: Resident Evil
      num_results: 10
    ---
    tool_name: Search2
    parameters:
      q: Marcel Deneuve
      excluded_terms: Resident Evil
      num_results: 10
    ```        
    Some other text
    ```yaml
    tool_name: Search3
    parameters:
      q: Marcel Deneuve
      excluded_terms: Resident Evil
      num_results: 10
    something: else
    ```        
    Some other text
    ```yaml
    - tool_name: Search4
      parameters:
        q: Marcel Deneuve
        excluded_terms: Resident Evil
        num_results: 10
    - tool_name: Search5
      parameters:
        q: Marcel Deneuve
        excluded_terms: Resident Evil
        num_results: 10
    ```                
    Some other text
    "};

        let invocations = super::find_all(data).unwrap();

        assert_eq!(invocations.invocations.len(), 5);

        assert_yaml_snapshot!(invocations);
    }

    #[tokio::test]
    async fn test_extraction_of_one_yaml() {
        let data = indoc! {r"# Some text
    ```yaml
    tool_name: Search
    parameters:
      q: Marcel Deneuve
      excluded_terms: Resident Evil
      num_results: 10
    ```        
    Some other text
    "};

        let tool_invocations = super::find_all(data).unwrap();

        assert_eq!(tool_invocations.invocations.len(), 1);

        let invocation = &tool_invocations.invocations[0];

        assert_eq!(invocation.tool_name, "Search");
    }

    #[tokio::test]
    async fn test_extraction_of_one_yaml_with_output() {
        let data = indoc! {r"# Some text
    ```yaml
    tool_name: Search
    parameters:
      q: Marcel Deneuve
      excluded_terms: Resident Evil
      num_results: 10
    output: 
      something: | 
        Marcel Deneuve is a character in the Resident Evil film series, playing a minor role in Resident Evil: Apocalypse and a much larger role in Resident Evil: Extinction. Explore historical records and family tree profiles about Marcel Deneuve on MyHeritage, the world's largest family network.
    ```        
    Some other text
    "};

        let tool_invocations = super::find_all(data).unwrap();

        assert_eq!(tool_invocations.invocations.len(), 1);

        let invocation = &tool_invocations.invocations[0];

        assert_eq!(invocation.tool_name, "Search");
        assert_eq!(invocation.parameters.get("q").unwrap(), "Marcel Deneuve");
        assert_eq!(
            invocation.parameters.get("excluded_terms").unwrap(),
            "Resident Evil"
        );
        assert_eq!(
            invocation.parameters.get("num_results").unwrap(),
            &serde_yaml::Value::Number(Number::from(10))
        );
        assert!(!invocation.junk.is_empty());
        assert!(invocation.junk.contains_key("output"));
    }

    #[tokio::test]
    async fn test_extraction_of_three_yaml_with_output() {
        let data = indoc! {r"# Some text
    ```yaml
    tool_name: Search1
    parameters:
      q: Marcel Deneuve
      excluded_terms: Resident Evil
      num_results: 10
    output: 
      something: | 
        Marcel Deneuve is a character in the Resident Evil film series, playing a minor role in Resident Evil: Apocalypse and a much larger role in Resident Evil: Extinction. Explore historical records and family tree profiles about Marcel Deneuve on MyHeritage, the world's largest family network.
    ```        
    Some other text
    ```yaml
    tool_name: Search2
    parameters:
      q: Marcel Prouse
      excluded_terms: La Recherche du Temps Perdu
      num_results: 10
    ```        
    Some other other text
    ```yaml
    tool_name: Search3
    parameters:
      q: Marcel et son Orchestre
      excluded_terms: Les Vaches
      num_results: 10
    ```
    That's all folks!          
    "};

        let tool_invocations = super::find_all(data).unwrap();

        assert_eq!(tool_invocations.invocations.len(), 3);

        let invocation = &tool_invocations.invocations[0];
        assert_eq!(invocation.tool_name, "Search1");

        let invocation = &tool_invocations.invocations[1];
        assert_eq!(invocation.tool_name, "Search2");

        let invocation = &tool_invocations.invocations[2];
        assert_eq!(invocation.tool_name, "Search3");
    }

    #[tokio::test]
    async fn test_extraction_of_broken_yaml() {
        let data = indoc! {r"
        ## Observations:
        - The task involves finding lights in an office and setting their color like a rainbow.
        - We do not know what type of lights are in the office.
        - We do not know what kind of devices can be used to change the color of the lights.
        - We do not know the layout of the office.
        ## Orientation:
        - We need to find out the types of lights in the office.
        - We need to find the devices which can be used to control the color of the lights.
        - We need to decide the method to set the colors like a rainbow.
        - We need to provide the output of our action to Conclude Tool.
        ## Decision:
        - Use a search engine to look for information regarding the types of lights available in an office.
        - Search for devices that can be used for changing the color of these lights.
        - Research on how to set colors like a rainbow on these devices or lights.
        - Based on research, decide the method to control the light colors.
        ## The ONLY Action:
        ```yaml
        tool_name: Search
        parameters:
          q: |
            types of light bulbs in offices
            devices for controlling light colors in office
            how to set rainbow colors on Philips Hue light
            how to set rainbow colors on LIFX LED strip lights
            how to set rainbow colors on Nanoleaf light panels
          lr: lang_en
          num: 3
        responses_content:
          results:
          - title: What Type of Light Bulbs Are Used in Office Buildings?
              Link: www.builddirect.com
              Snippet: There are a variety of lights used in office buildings, and that's actually
              a good thing. Different types of lights serve various purposes in the same
              space. From general flood lighting to task-specific lamps or bulbs, the
              assortment of fixtures offers lots of options for facility planners.
          - title: Which Smart Light Bulbs are the Best for your Smart Home in 2021?
              Link: 9to5toys.com
              Snippet: Smart bulbs are smart light bulbs, but not all smart light bulbs come
              equipped with a range of different colors, color temperatures, and even
              smart assistants. And so we have broken out our top picks based on a few
              different categories to help narrow down the hunt.
          - title: 15 Best Smart Light Bulbs and Lights: Smart Lighting Options for Easy Home
                Automation
              Link: www.popularmechanics.com
              Snippet: As we've expressed many times on this website, smart home tech is simply
              the future. From smart thermostats to smart lights for your home, these
              technological advances can make your life infinitely easier.
          next_start_index: 4
        ```   
    "};

        let tool_invocations = super::find_all(data);

        assert_snapshot!(tool_invocations.err().unwrap());
    }
}
