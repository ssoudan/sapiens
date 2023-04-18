use serde::{Deserialize, Serialize};

/// Tools to get information about rooms and their lights.
pub mod room;
/// Tools to get information about the lights
pub mod status;

/// State of a light.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct State {
    /// Whether the light is on.
    pub on: Option<bool>,
    /// Brightness of the light.
    ///
    /// The maximum brightness is 254 and 1 is the minimum brightness.
    pub brightness: Option<u8>,
    /// Hue of the light.
    ///
    /// Both 0 and 65535 are red, 25500 is green and 46920 is blue.
    pub hue: Option<u16>,
    /// Saturation of the light.
    ///
    /// The most saturated (colored) is 254 and 0 is the least saturated
    /// (white).
    pub saturation: Option<u8>,
    /// X and y coordinates of a color in CIE color space. Both values must be
    /// between 0 and 1.
    // pub color_space_coordinates: Option<(f32, f32)>,
    /// Mired color temperature of the light.
    pub color_temperature: Option<u16>,
    // /// Alert effect of the light.
    // pub alert: Option<Alert>,
    // /// Dynamic effect of the light.
    // pub effect: Option<Effect>,
    // /// Color mode of the light.
    // pub color_mode: Option<ColorMode>,
    // Whether the light can be reached by the bridge.
    // pub reachable: bool,
}

impl From<huelib::resource::light::State> for State {
    fn from(value: huelib::resource::light::State) -> Self {
        Self {
            on: value.on,
            brightness: value.brightness,
            hue: value.hue,
            saturation: value.saturation,
            color_temperature: value.color_temperature,
        }
    }
}

/// A light.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Light {
    /// Identifier of the light.
    pub id: String,
    /// Name of the light.
    pub name: String,
    /// Type of the light.
    // #[serde(rename = "type")]
    // pub kind: String,
    /// Current state of the light.
    #[serde(flatten)]
    pub state: State,
}

impl From<huelib::resource::light::Light> for Light {
    fn from(value: huelib::resource::light::Light) -> Self {
        Self {
            id: value.id,
            name: value.name,
            state: value.state.into(),
        }
    }
}

/// A group of lights.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Room {
    /// Name of the group.
    pub name: String,
    /// Identifiers of lights that are in this group.
    pub lights: Vec<String>,
}

impl From<huelib::resource::group::Group> for Room {
    fn from(value: huelib::resource::group::Group) -> Self {
        Self {
            name: value.name,
            lights: value.lights,
        }
    }
}
