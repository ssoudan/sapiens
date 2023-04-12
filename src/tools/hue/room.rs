use llm_chain_tools::{Describe, Format, Tool, ToolDescription};
use serde::{Deserialize, Serialize};

use crate::tools::hue::Group;

/// A tool that get the lights of a Room
pub struct RoomTool {
    bridge: huelib::bridge::Bridge,
}

impl RoomTool {
    pub fn new(bridge: huelib::bridge::Bridge) -> Self {
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

        Self::new(bridge)
    }
}

#[derive(Serialize, Deserialize)]
pub struct RoomToolInput {
    room_names: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct RoomToolOutput {
    rooms: Vec<Group>,
}

impl Describe for RoomToolInput {
    fn describe() -> Format {
        vec![(
            "room_names",
            "The name of the Rooms to get the lights for. If empty, returns all the lights for all the Rooms.",
        )
            .into()]
        .into()
    }
}

impl Describe for RoomToolOutput {
    fn describe() -> Format {
        vec![(
            "rooms",
            "A list of Rooms with a name and a list of lights in that room.",
        )
            .into()]
        .into()
    }
}

impl RoomTool {
    fn invoke_typed(&self, input: &RoomToolInput) -> Result<RoomToolOutput, String> {
        let room_names = &input.room_names;

        self.bridge
            .get_all_groups()
            .map(|groups| {
                let mut rooms: Vec<Group> = Vec::new();
                for group in groups {
                    if room_names.is_empty() || room_names.contains(&group.name) {
                        rooms.push(Group::from(group));
                    }
                }
                Ok(RoomToolOutput { rooms })
            })
            .map_err(|e| format!("Failed to get groups: {}", e))?
    }
}

impl Tool for RoomTool {
    fn description(&self) -> ToolDescription {
        ToolDescription::new(
            "RoomTool",
            "A tool that get the Lights of a Room.",
            "Use this to fetch the Lights of Rooms.",
            RoomToolInput::describe(),
            RoomToolOutput::describe(),
        )
    }

    fn invoke(&self, input: serde_yaml::Value) -> Result<serde_yaml::Value, String> {
        let input = serde_yaml::from_value(input).unwrap();
        let output = self.invoke_typed(&input).unwrap();
        Ok(serde_yaml::to_value(output).unwrap())
    }
}
