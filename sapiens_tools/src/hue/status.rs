use std::fmt::Debug;

use huelib2::resource::Adjust;
use sapiens::tools::{Describe, ProtoToolDescribe, ProtoToolInvoke, ToolDescription, ToolUseError};
use sapiens_derive::{Describe, ProtoToolDescribe, ProtoToolInvoke};
use serde::{Deserialize, Serialize};

use crate::hue::Light;

/// A tool to use as the source of truth for the Light statuses.
#[derive(ProtoToolDescribe, ProtoToolInvoke)]
#[tool(
    name = "LightStatus",
    input = "StatusToolInput",
    output = "StatusToolOutput"
)]
#[allow(clippy::module_name_repetitions)]
pub struct StatusTool {
    bridge: huelib2::bridge::Bridge,
}

impl Default for StatusTool {
    fn default() -> Self {
        let bridge_ip = huelib2::bridge::discover_nupnp()
            .expect("Failed to discover bridge")
            .pop()
            .expect("No bridges found");

        let username = std::env::var("HUE_USERNAME").expect("HUE_USERNAME not set");

        let bridge = huelib2::bridge::Bridge::new(bridge_ip, username);

        Self::new(bridge)
    }
}

/// The input of the tool
#[derive(Debug, Serialize, Deserialize, Describe)]
#[allow(clippy::module_name_repetitions)]
pub struct StatusToolInput {
    /// The list of Lights IDs (<string>) to get the status for, e.g.: `["1",
    /// "2"]`. To get all the lights: `[]`
    pub light_filter: Option<Vec<String>>,
}

/// The output of the tool
#[derive(Debug, Serialize, Deserialize, Describe)]
#[allow(clippy::module_name_repetitions)]
pub struct StatusToolOutput {
    /// A list of Lights with their statuses. E.g.: `[{"id": "1", "name":
    /// "Corridor", "on": True, "brightness": 126, "hue": 2456, "saturation":
    /// 55, "color_temperature": 2500}]`
    pub lights: Vec<Light>,
}

impl StatusTool {
    /// Create a new `StatusTool`
    #[must_use]
    pub const fn new(bridge: huelib2::bridge::Bridge) -> Self {
        Self { bridge }
    }

    #[tracing::instrument(skip(self))]
    async fn invoke_typed(
        &self,
        input: &StatusToolInput,
    ) -> Result<StatusToolOutput, ToolUseError> {
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
            .map_err(|e| ToolUseError::InvocationFailed(e.to_string()))?
    }
}

/// A tool to use as the set the Light statuses.
#[derive(ProtoToolDescribe, ProtoToolInvoke)]
#[tool(
    name = "SetLightStatus",
    input = "SetStatusToolInput",
    output = "StatusToolOutput"
)]
#[allow(clippy::module_name_repetitions)]
pub struct SetStatusTool {
    bridge: huelib2::bridge::Bridge,
}

impl Debug for SetStatusTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SetStatusTool").finish()
    }
}

impl SetStatusTool {
    /// Create a new `StatusTool`
    #[must_use]
    pub const fn new(bridge: huelib2::bridge::Bridge) -> Self {
        Self { bridge }
    }

    #[tracing::instrument(skip(self))]
    async fn invoke_typed(
        &self,
        input: &SetStatusToolInput,
    ) -> Result<StatusToolOutput, ToolUseError> {
        if let Some(lights) = &input.lights {
            for light in lights {
                let state = huelib2::resource::light::StateModifier {
                    on: light.state.on,
                    brightness: light.state.brightness.map(Adjust::Override),
                    hue: light.state.hue.map(Adjust::Override),
                    saturation: light.state.saturation.map(Adjust::Override),
                    color_space_coordinates: None,
                    color_temperature: light.state.color_temperature.map(Adjust::Override),
                    alert: None,
                    effect: None,
                    transition_time: None,
                };

                self.bridge
                    .set_light_state(&light.id, &state)
                    .map_err(|e| {
                        ToolUseError::InvocationFailed(format!(
                            "Failed to set light state for light {}: {}",
                            light.id, e
                        ))
                    })?;
            }
        }

        // collect light IDs
        let light_ids: Vec<String> = input
            .lights
            .as_ref()
            .map(|lights| lights.iter().map(|l| l.id.clone()).collect())
            .unwrap_or_default();

        if light_ids.is_empty() {
            return Err(ToolUseError::InvocationFailed(
                "No lights to set status for".to_string(),
            ));
        }

        // get all lights
        let lights_status: Vec<Light> = self
            .bridge
            .get_all_lights()
            .map_err(|e| ToolUseError::InvocationFailed(e.to_string()))?
            .into_iter()
            .map(std::convert::Into::into)
            .collect();

        // filter lights
        let lights = if light_ids.is_empty() {
            lights_status
        } else {
            lights_status
                .into_iter()
                .filter(|l| light_ids.contains(&l.id))
                .collect()
        };

        Ok(StatusToolOutput { lights })
    }
}

impl Default for SetStatusTool {
    fn default() -> Self {
        let bridge_ip = huelib2::bridge::discover_nupnp()
            .expect("Failed to discover bridge")
            .pop()
            .expect("No bridges found");

        let username = std::env::var("HUE_USERNAME").expect("HUE_USERNAME not set");

        let bridge = huelib2::bridge::Bridge::new(bridge_ip, username);

        Self::new(bridge)
    }
}

/// The input of the tool
#[derive(Debug, Serialize, Deserialize, Describe)]
pub struct SetStatusToolInput {
    /// The list of Lights statuses to set for, e.g.: `[{"id": "1", "on": True,
    /// "brightness": 126, "hue": 2456, "saturation":
    /// 55, "color_temperature": 2500}]`. Omitted fields will not be changed.
    pub lights: Option<Vec<Light>>,
}

/// A fake `StatusTool`
pub mod fake {
    use sapiens::tools::{
        Describe, ProtoToolDescribe, ProtoToolInvoke, ToolDescription, ToolUseError,
    };

    use crate::hue::status::{StatusToolInput, StatusToolOutput};
    use crate::hue::{Light, State};

    /// A fake `StatusTool`
    #[allow(clippy::module_name_repetitions)]
    pub struct FakeStatusTool {}

    impl FakeStatusTool {
        /// Create a new `StatusTool`
        #[must_use]
        pub const fn new() -> Self {
            Self {}
        }
    }

    impl Default for FakeStatusTool {
        fn default() -> Self {
            Self::new()
        }
    }

    impl ProtoToolDescribe for FakeStatusTool {
        fn description(&self) -> ToolDescription {
            ToolDescription::new(
                "LightStatus",
                "A tool to use that the source of truth for the Light statuses.",
                StatusToolInput::describe(),
                StatusToolOutput::describe(),
            )
        }
    }

    #[async_trait::async_trait]
    impl ProtoToolInvoke for FakeStatusTool {
        async fn invoke(
            &self,
            input: serde_yaml::Value,
        ) -> Result<serde_yaml::Value, ToolUseError> {
            let input = serde_yaml::from_value(input)
                .map_err(|e| ToolUseError::InvalidInput(e.to_string()))?;
            let output = self.invoke_typed(&input).await?;
            Ok(serde_yaml::to_value(output)
                .map_err(|e| ToolUseError::InvalidOutput(e.to_string()))?)
        }
    }

    impl FakeStatusTool {
        #[tracing::instrument(skip(self))]
        async fn invoke_typed(
            &self,
            input: &StatusToolInput,
        ) -> Result<StatusToolOutput, ToolUseError> {
            let lights = vec![
                Light {
                    id: "1".to_string(),
                    name: Some("Bed".to_string()),
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
                    name: Some("Closet".to_string()),
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
                    name: Some("Ceiling".to_string()),
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
