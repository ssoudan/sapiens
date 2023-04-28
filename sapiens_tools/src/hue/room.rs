use std::fmt::Debug;

use huelib2::resource::group::CreatableKind;
use huelib2::resource::group::Kind::Creatable;
use sapiens::tools::{Describe, ProtoToolDescribe, ProtoToolInvoke, ToolDescription, ToolUseError};
use sapiens_derive::{Describe, ProtoToolDescribe, ProtoToolInvoke};
use serde::{Deserialize, Serialize};

use crate::hue::Room;

/// A tool to use that the source of truth for the Lights of a Room.
#[derive(ProtoToolDescribe, ProtoToolInvoke)]
#[tool(name = "Room", input = "RoomToolInput", output = "RoomToolOutput")]
pub struct RoomTool {
    bridge: huelib2::bridge::Bridge,
}

impl Debug for RoomTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RoomTool").finish()
    }
}

impl RoomTool {
    /// Create a new RoomTool
    pub fn new(bridge: huelib2::bridge::Bridge) -> Self {
        RoomTool { bridge }
    }
}

impl Default for RoomTool {
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
pub struct RoomToolInput {
    /// The list of Room names (<string>) to get the lights for, e.g.
    /// `room_filter: ["Bedroom"]`. If unsure, use `[]` as `room_filter` to get
    /// all Rooms.
    pub room_filter: Vec<String>,
}

/// The output of the tool
#[derive(Debug, Serialize, Deserialize, PartialEq, Describe)]
pub struct RoomToolOutput {
    /// A list of Rooms with a name and a list of Light IDs in that
    /// room. E.g.: `[{"name": "Smoking room", "lights": ["1", "2", ...]},
    /// ...]`
    pub rooms: Vec<Room>,
}

impl RoomTool {
    #[tracing::instrument(skip(self))]
    async fn invoke_typed(&self, input: &RoomToolInput) -> Result<RoomToolOutput, ToolUseError> {
        let room_filter = &input.room_filter;

        self.bridge
            .get_all_groups()
            .map(|groups| {
                let mut rooms: Vec<Room> = Vec::new();
                for group in groups {
                    if group.kind == Creatable(CreatableKind::Room)
                        && (room_filter.is_empty() || room_filter.contains(&group.name))
                    {
                        rooms.push(Room {
                            name: group.name,
                            lights: group.lights,
                        });
                    }
                }
                Ok(RoomToolOutput { rooms })
            })
            .map_err(|e| ToolUseError::InvocationFailed(e.to_string()))?
    }
}

/// A fake RoomTool
pub mod fake {
    use sapiens::tools::{Describe, Tool, ToolDescription, ToolUseError};

    use crate::hue::room::{RoomToolInput, RoomToolOutput};
    use crate::hue::Room;

    /// a fake RoomTool
    #[derive(Default)]
    pub struct FakeRoomTool {}

    impl FakeRoomTool {
        #[tracing::instrument(skip(self))]
        async fn invoke_typed(
            &self,
            input: &RoomToolInput,
        ) -> Result<RoomToolOutput, ToolUseError> {
            let rooms = vec![
                Room {
                    name: "Bedroom".to_string(),
                    lights: vec!["1".to_string(), "2".to_string()],
                },
                Room {
                    name: "Living room".to_string(),
                    lights: vec!["3".to_string()],
                },
            ];

            let room_filter = &input.room_filter;

            let rooms = rooms
                .into_iter()
                .filter(|room| room_filter.is_empty() || room_filter.contains(&room.name))
                .collect();

            Ok(RoomToolOutput { rooms })
        }
    }

    #[async_trait::async_trait]
    impl Tool for FakeRoomTool {
        fn description(&self) -> ToolDescription {
            ToolDescription::new(
                "Room",
                "Use this to get the lights of a Room.",
                RoomToolInput::describe(),
                RoomToolOutput::describe(),
            )
        }

        async fn invoke(
            &self,
            input: serde_yaml::Value,
        ) -> Result<serde_yaml::Value, ToolUseError> {
            let input = serde_yaml::from_value(input)?;
            let output = self.invoke_typed(&input).await?;
            Ok(serde_yaml::to_value(output)?)
        }
    }

    #[cfg(test)]
    mod tests {

        use sapiens::tools::Tool;

        use super::*;

        #[tokio::test]
        async fn test_fake_room_tool() {
            let tool = FakeRoomTool::default();
            let input = RoomToolInput {
                room_filter: vec!["Bedroom".to_string()],
            };
            let input = serde_yaml::to_value(input).unwrap();
            let output = tool.invoke(input).await.unwrap();
            let output: RoomToolOutput = serde_yaml::from_value(output).unwrap();
            let expected = RoomToolOutput {
                rooms: vec![Room {
                    name: "Bedroom".to_string(),
                    lights: vec!["1".to_string(), "2".to_string()],
                }],
            };
            assert_eq!(output, expected);
        }

        #[tokio::test]
        async fn test_fake_room_tool_empty_filter() {
            let tool = FakeRoomTool::default();
            let input = RoomToolInput {
                room_filter: vec![],
            };
            let input = serde_yaml::to_value(input).unwrap();
            let output = tool.invoke(input).await.unwrap();
            let output: RoomToolOutput = serde_yaml::from_value(output).unwrap();
            let expected = RoomToolOutput {
                rooms: vec![
                    Room {
                        name: "Bedroom".to_string(),
                        lights: vec!["1".to_string(), "2".to_string()],
                    },
                    Room {
                        name: "Living room".to_string(),
                        lights: vec!["3".to_string()],
                    },
                ],
            };
            assert_eq!(output, expected);
        }
    }
}
