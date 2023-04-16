use std::rc::Rc;

use huelib::resource::group::CreatableKind;
use huelib::resource::group::Kind::Creatable;
use llm_chain::tools::{Describe, Format, Tool, ToolDescription, ToolUseError};
use serde::{Deserialize, Serialize};

use crate::tools::hue::Room;

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
#[derive(Serialize, Deserialize)]
pub struct RoomToolInput {
    room_filter: Option<Vec<String>>,
}

/// The output of the tool
#[derive(Serialize, Deserialize)]
pub struct RoomToolOutput {
    rooms: Vec<Room>,
}

impl Describe for RoomToolInput {
    fn describe() -> Format {
        vec![(
            "room_filter",
            "The list of Room names (<string>) to get the lights for, e.g. `room_filter: [\"Bedroom\"]`. If unsure, use `[]` to get all Rooms.",
        )
            .into()]
        .into()
    }
}

impl Describe for RoomToolOutput {
    fn describe() -> Format {
        vec![(
            "rooms",
            r#"A list of Rooms with a name and a list of lights (their IDs) in that room. E.g.: [{"name": "Smoking room", "lights": ["light_ID1", ...]}, ...]"#,
        )
            .into()]
        .into()
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
                    if group.kind == Creatable(CreatableKind::Room) {
                        if let Some(room_filter) = room_filter {
                            if room_filter.is_empty() || room_filter.contains(&group.name) {
                                rooms.push(group.into());
                            }
                        } else {
                            rooms.push(group.into());
                        }
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
