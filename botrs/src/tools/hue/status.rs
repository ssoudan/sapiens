use std::rc::Rc;

use llm_chain::tools::{Describe, Format, Tool, ToolDescription, ToolUseError};
use serde::{Deserialize, Serialize};

use crate::tools::hue::Light;

/// A tool that get the light statuses
pub struct StatusTool {
    bridge: Rc<huelib::bridge::Bridge>,
}

impl StatusTool {
    /// Create a new StatusTool
    pub fn new(bridge: Rc<huelib::bridge::Bridge>) -> Self {
        StatusTool { bridge }
    }
}

impl Default for StatusTool {
    fn default() -> Self {
        let bridge_ip = huelib::bridge::discover_nupnp()
            .expect("Failed to discover bridge")
            .pop()
            .expect("No bridges found");

        let username = std::env::var("HUE_USERNAME").expect("HUE_USERNAME not set");

        let bridge = huelib::bridge::Bridge::new(bridge_ip, username);

        Self::new(Rc::new(bridge))
    }
}

/// The input of the tool
#[derive(Serialize, Deserialize)]
pub struct StatusToolInput {
    light_filter: Option<Vec<String>>,
}

/// The output of the tool
#[derive(Serialize, Deserialize)]
pub struct StatusToolOutput {
    lights: Vec<Light>,
}

impl Describe for StatusToolInput {
    fn describe() -> Format {
        vec![(
            "light_filter",
            "The list of Lights IDs (<string>) to get the status for, e.g.: [\"1\", \"2\"]. To get all the lights: []",
        )
            .into()]
        .into()
    }
}

impl Describe for StatusToolOutput {
    fn describe() -> Format {
        vec![("lights", r#"A list of Lights with their status. E.g.: [{"id": "1", "name": "Corridor", "on": True, "brightness": 126, "hue": 2456, "saturation": 55, "color_temperature": 2500}]"#).into()].into()
    }
}

/// A fake StatusTool
pub mod fake {
    use llm_chain::tools::{Describe, Tool, ToolDescription, ToolUseError};

    use crate::tools::hue::status::{StatusToolInput, StatusToolOutput};
    use crate::tools::hue::{Light, State};

    /// A fake StatusTool
    pub struct FakeStatusTool {}

    impl FakeStatusTool {
        /// Create a new StatusTool
        pub fn new() -> Self {
            FakeStatusTool {}
        }
    }

    impl Default for FakeStatusTool {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Tool for FakeStatusTool {
        fn description(&self) -> ToolDescription {
            ToolDescription::new(
                "LightStatus",
                "A tool to use that the source of truth for the Light statuses.",
                "Use this to fetch the Light statuses",
                StatusToolInput::describe(),
                StatusToolOutput::describe(),
            )
        }

        fn invoke(&self, input: serde_yaml::Value) -> Result<serde_yaml::Value, ToolUseError> {
            let input = serde_yaml::from_value(input)?;
            let output = self.invoke_typed(&input)?;
            Ok(serde_yaml::to_value(output)?)
        }
    }

    impl FakeStatusTool {
        fn invoke_typed(&self, input: &StatusToolInput) -> Result<StatusToolOutput, ToolUseError> {
            let lights = vec![
                Light {
                    id: "1".to_string(),
                    name: "Bed".to_string(),
                    state: State {
                        on: Option::from(true),
                        brightness: Some(126),
                        hue: Some(2456),
                        saturation: Some(55),
                        color_temperature: Some(2500),
                    },
                },
                Light {
                    id: "2".to_string(),
                    name: "Closet".to_string(),
                    state: State {
                        on: Option::from(false),
                        brightness: Some(0),
                        hue: Some(0),
                        saturation: Some(0),
                        color_temperature: Some(0),
                    },
                },
                Light {
                    id: "3".to_string(),
                    name: "Ceiling".to_string(),
                    state: State {
                        on: Option::from(false),
                        brightness: Some(0),
                        hue: Some(0),
                        saturation: Some(0),
                        color_temperature: Some(0),
                    },
                },
            ];

            let light_filter = &input.light_filter;

            let lights = lights
                .into_iter()
                .filter(|l| {
                    if let Some(light_filter) = light_filter {
                        light_filter.is_empty() || light_filter.contains(&l.id)
                    } else {
                        true
                    }
                })
                .collect();

            Ok(StatusToolOutput { lights })
        }
    }
}

impl StatusTool {
    fn invoke_typed(&self, input: &StatusToolInput) -> Result<StatusToolOutput, ToolUseError> {
        let light_filter = &input.light_filter;

        self.bridge
            .get_all_lights()
            .map(|lights| {
                let mut res: Vec<Light> = Vec::new();
                for l in lights {
                    if let Some(light_filter) = light_filter {
                        if light_filter.is_empty() || light_filter.contains(&l.id) {
                            res.push(l.into());
                        }
                    } else {
                        res.push(l.into());
                    }
                }
                Ok(StatusToolOutput { lights: res })
            })
            .map_err(|e| ToolUseError::ToolInvocationFailed(e.to_string()))?
    }
}

impl Tool for StatusTool {
    fn description(&self) -> ToolDescription {
        ToolDescription::new(
            "LightStatus",
            "A tool to use that the source of truth for the Light statuses.",
            "Use this to fetch the Light statuses",
            StatusToolInput::describe(),
            StatusToolOutput::describe(),
        )
    }

    fn invoke(&self, input: serde_yaml::Value) -> Result<serde_yaml::Value, ToolUseError> {
        let input = serde_yaml::from_value(input)?;
        let output = self.invoke_typed(&input)?;
        Ok(serde_yaml::to_value(output)?)
    }
}
