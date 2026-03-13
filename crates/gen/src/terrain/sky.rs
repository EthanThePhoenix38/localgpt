//! Sky and atmosphere configuration.
//!
//! Sets sun direction, ambient light, fog, and sky appearance.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Sky preset.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "lowercase")]
pub enum SkyPreset {
    #[default]
    Day,
    Sunset,
    Night,
    Overcast,
    Custom,
}

/// Parameters for sky configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkyParams {
    /// Preset to use.
    #[serde(default)]
    pub preset: SkyPreset,
    /// Sun angle above horizon (0-90 degrees).
    #[serde(default)]
    pub sun_altitude: Option<f32>,
    /// Sun compass direction (0=north).
    #[serde(default)]
    pub sun_azimuth: Option<f32>,
    /// Sun brightness multiplier.
    #[serde(default)]
    pub sun_intensity: Option<f32>,
    /// Ambient light color (hex).
    #[serde(default)]
    pub ambient_color: Option<String>,
    /// Ambient light brightness.
    #[serde(default)]
    pub ambient_intensity: Option<f32>,
    /// Enable distance fog.
    #[serde(default)]
    pub fog_enabled: bool,
    /// Fog color (hex).
    #[serde(default = "default_fog_color")]
    pub fog_color: String,
    /// Fog start distance.
    #[serde(default = "default_fog_start")]
    pub fog_start: f32,
    /// Fog end distance.
    #[serde(default = "default_fog_end")]
    pub fog_end: f32,
}

fn default_fog_color() -> String {
    "#c8d0d8".to_string()
}
fn default_fog_start() -> f32 {
    50.0
}
fn default_fog_end() -> f32 {
    200.0
}

impl Default for SkyParams {
    fn default() -> Self {
        Self {
            preset: SkyPreset::Day,
            sun_altitude: None,
            sun_azimuth: None,
            sun_intensity: None,
            ambient_color: None,
            ambient_intensity: None,
            fog_enabled: false,
            fog_color: default_fog_color(),
            fog_start: default_fog_start(),
            fog_end: default_fog_end(),
        }
    }
}

/// Resource holding current sky configuration.
#[derive(Resource, Reflect, Clone)]
#[reflect(Resource)]
pub struct SkyConfig {
    /// Current preset.
    pub preset: SkyPreset,
    /// Sun altitude in degrees.
    pub sun_altitude: f32,
    /// Sun azimuth in degrees.
    pub sun_azimuth: f32,
    /// Sun intensity (lux multiplier).
    pub sun_intensity: f32,
    /// Ambient color (RGB).
    pub ambient_color: [f32; 3],
    /// Ambient intensity.
    pub ambient_intensity: f32,
    /// Fog enabled.
    pub fog_enabled: bool,
    /// Fog color (RGB).
    pub fog_color: [f32; 3],
    /// Fog start distance.
    pub fog_start: f32,
    /// Fog end distance.
    pub fog_end: f32,
}

impl Default for SkyConfig {
    fn default() -> Self {
        Self::from_preset(SkyPreset::Day)
    }
}

impl SkyConfig {
    /// Create config from preset.
    pub fn from_preset(preset: SkyPreset) -> Self {
        match preset {
            SkyPreset::Day => Self {
                preset,
                sun_altitude: 60.0,
                sun_azimuth: 180.0,
                sun_intensity: 1.0,
                ambient_color: [0.53, 0.81, 0.92], // Sky blue
                ambient_intensity: 0.3,
                fog_enabled: false,
                fog_color: [0.78, 0.82, 0.85],
                fog_start: 50.0,
                fog_end: 200.0,
            },
            SkyPreset::Sunset => Self {
                preset,
                sun_altitude: 5.0,
                sun_azimuth: 270.0,
                sun_intensity: 0.8,
                ambient_color: [1.0, 0.6, 0.3], // Warm amber
                ambient_intensity: 0.4,
                fog_enabled: true,
                fog_color: [0.9, 0.7, 0.6],
                fog_start: 30.0,
                fog_end: 150.0,
            },
            SkyPreset::Night => Self {
                preset,
                sun_altitude: -10.0,
                sun_azimuth: 0.0,
                sun_intensity: 0.1,
                ambient_color: [0.1, 0.1, 0.2], // Dark blue
                ambient_intensity: 0.05,
                fog_enabled: false,
                fog_color: [0.1, 0.1, 0.15],
                fog_start: 20.0,
                fog_end: 100.0,
            },
            SkyPreset::Overcast => Self {
                preset,
                sun_altitude: 45.0,
                sun_azimuth: 180.0,
                sun_intensity: 0.3,
                ambient_color: [0.7, 0.7, 0.7], // Gray
                ambient_intensity: 0.5,
                fog_enabled: true,
                fog_color: [0.78, 0.82, 0.85],
                fog_start: 30.0,
                fog_end: 120.0,
            },
            SkyPreset::Custom => Self {
                preset,
                sun_altitude: 45.0,
                sun_azimuth: 180.0,
                sun_intensity: 1.0,
                ambient_color: [1.0, 1.0, 1.0],
                ambient_intensity: 0.3,
                fog_enabled: false,
                fog_color: [0.78, 0.82, 0.85],
                fog_start: 50.0,
                fog_end: 200.0,
            },
        }
    }

    /// Apply parameter overrides.
    pub fn with_overrides(mut self, params: &SkyParams) -> Self {
        if let Some(alt) = params.sun_altitude {
            self.sun_altitude = alt;
        }
        if let Some(az) = params.sun_azimuth {
            self.sun_azimuth = az;
        }
        if let Some(int) = params.sun_intensity {
            self.sun_intensity = int;
        }
        if let Some(rgb) = params
            .ambient_color
            .as_deref()
            .and_then(parse_hex_color_to_rgb)
        {
            self.ambient_color = rgb;
        }
        if let Some(int) = params.ambient_intensity {
            self.ambient_intensity = int;
        }
        self.fog_enabled = params.fog_enabled;
        if let Some(rgb) = parse_hex_color_to_rgb(&params.fog_color) {
            self.fog_color = rgb;
        }
        self.fog_start = params.fog_start;
        self.fog_end = params.fog_end;
        self
    }

    /// Get sun direction from altitude and azimuth.
    pub fn sun_direction(&self) -> Vec3 {
        let alt_rad = self.sun_altitude.to_radians();
        let az_rad = self.sun_azimuth.to_radians();

        // Convert spherical to Cartesian
        // azimuth 0 = north (positive Z), 90 = east (positive X)
        Vec3::new(
            -alt_rad.sin() * az_rad.sin(),
            alt_rad.cos(),
            -alt_rad.sin() * az_rad.cos(),
        )
        .normalize_or_zero()
    }
}

/// Parse hex color string to RGB values.
pub fn parse_hex_color_to_rgb(hex: &str) -> Option<[f32; 3]> {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;
        Some([r, g, b])
    } else {
        None
    }
}

/// Parse hex color string to Bevy Color.
pub fn parse_hex_color(hex: &str) -> Option<Color> {
    let rgb = parse_hex_color_to_rgb(hex)?;
    Some(Color::srgb(rgb[0], rgb[1], rgb[2]))
}

/// Marker component for directional light (sun).
#[derive(Component)]
pub struct SunLight;

/// Plugin for sky systems.
pub struct SkyPlugin;

impl Plugin for SkyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SkyConfig>();
        // Note: Light/fog updates are handled by existing systems in gen3d/plugin.rs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_day_preset() {
        let config = SkyConfig::from_preset(SkyPreset::Day);
        assert_eq!(config.sun_altitude, 60.0);
        assert!(!config.fog_enabled);
    }

    #[test]
    fn test_sunset_preset() {
        let config = SkyConfig::from_preset(SkyPreset::Sunset);
        assert_eq!(config.sun_altitude, 5.0);
        assert!(config.fog_enabled);
    }

    #[test]
    fn test_sun_direction() {
        let config = SkyConfig::from_preset(SkyPreset::Day);
        let dir = config.sun_direction();
        // At 60° altitude, Y component should be positive
        assert!(dir.y > 0.0);
    }

    #[test]
    fn test_parse_hex_color() {
        let color = parse_hex_color("#ff0000");
        assert!(color.is_some());

        let rgb = parse_hex_color_to_rgb("#00ff00");
        assert!(rgb.is_some());
        assert!((rgb.unwrap()[1] - 1.0).abs() < 0.01);

        let color = parse_hex_color("invalid");
        assert!(color.is_none());
    }
}
