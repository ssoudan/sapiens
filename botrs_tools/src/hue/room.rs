use std::rc::Rc;

use botrs_derive::Describe;
use huelib::resource::group::CreatableKind;
use huelib::resource::group::Kind::Creatable;
use llm_chain::tools::{Describe, Format, Tool, ToolDescription, ToolUseError};
use serde::{Deserialize, Serialize};

use crate::hue::Room;

/// A tool that get the lights of a Room
pub struct RoomTool {
    bridge: Rc<huelib::bridge::Bridge>,
}

impl RoomTool {
    /// Create a new RoomTool
    pub fn new(bridge: Rc<huelib::bridge::Bridge>) -> Self {
        RoomTool { bridge }
    }
}

impl Default for RoomTool {
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
#[derive(Serialize, Deserialize, Describe)]
pub struct RoomToolInput {
    /// The list of Room names (<string>) to get the lights for, e.g.
    /// `room_filter: ["Bedroom"]`. If unsure, use `[]` to get all Rooms.
    pub room_filter: Vec<String>,
}

/// The output of the tool
#[derive(Serialize, Deserialize, PartialEq, Debug, Describe)]
pub struct RoomToolOutput {
    /// A list of Rooms with a name and a list of lights (their IDs) in that
    /// room. E.g.: `[{"name": "Smoking room", "lights": ["light_ID1", ...]},
    /// ...]`
    pub rooms: Vec<Room>,
}

/// A fake RoomTool
pub mod fake {
    use llm_chain::tools::{Describe, Tool, ToolDescription, ToolUseError};

    use crate::hue::room::{RoomToolInput, RoomToolOutput};
    use crate::hue::Room;

    /// a fake RoomTool
    #[derive(Default)]
    pub struct FakeRoomTool {}

    impl FakeRoomTool {
        fn invoke_typed(&self, input: &RoomToolInput) -> Result<RoomToolOutput, ToolUseError> {
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

    impl Tool for FakeRoomTool {
        fn description(&self) -> ToolDescription {
            ToolDescription::new(
                "Room",
                "A fake tool to get the lights of a Room.",
                "Use this to get the lights of a Room.",
                RoomToolInput::describe(),
                RoomToolOutput::describe(),
            )
        }

        fn invoke(&self, input: serde_yaml::Value) -> Result<serde_yaml::Value, ToolUseError> {
            let input = serde_yaml::from_value(input)?;
            let output = self.invoke_typed(&input)?;
            Ok(serde_yaml::to_value(output)?)
        }
    }

    #[cfg(test)]
    mod tests {

        use llm_chain::tools::Tool;

        use super::*;

        #[test]
        fn test_fake_room_tool() {
            let tool = FakeRoomTool::default();
            let input = RoomToolInput {
                room_filter: vec!["Bedroom".to_string()],
            };
            let input = serde_yaml::to_value(input).unwrap();
            let output = tool.invoke(input).unwrap();
            let output: RoomToolOutput = serde_yaml::from_value(output).unwrap();
            let expected = RoomToolOutput {
                rooms: vec![Room {
                    name: "Bedroom".to_string(),
                    lights: vec!["1".to_string(), "2".to_string()],
                }],
            };
            assert_eq!(output, expected);
        }

        #[test]
        fn test_fake_room_tool_empty_filter() {
            let tool = FakeRoomTool::default();
            let input = RoomToolInput {
                room_filter: vec![],
            };
            let input = serde_yaml::to_value(input).unwrap();
            let output = tool.invoke(input).unwrap();
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

impl RoomTool {
    fn invoke_typed(&self, input: &RoomToolInput) -> Result<RoomToolOutput, ToolUseError> {
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
            .map_err(|e| ToolUseError::ToolInvocationFailed(e.to_string()))?
    }
}

impl Tool for RoomTool {
    fn description(&self) -> ToolDescription {
        ToolDescription::new(
            "Room",
            "A tool to use that the source of truth for the Lights of a Room.",
            "Use this to fetch the Lights of Rooms.",
            RoomToolInput::describe(),
            RoomToolOutput::describe(),
        )
    }

    fn invoke(&self, input: serde_yaml::Value) -> Result<serde_yaml::Value, ToolUseError> {
        let input = serde_yaml::from_value(input)?;
        let output = self.invoke_typed(&input)?;
        Ok(serde_yaml::to_value(output)?)
    }
}
