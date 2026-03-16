//! Depth map rendering — render the scene as a depth map for preview generation.
//!
//! WG7.1: Captures depth buffer from a configurable camera angle, normalizes
//! to grayscale (near=white, far=black), and saves as PNG.

use serde::{Deserialize, Serialize};

/// Configuration for depth map rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepthRenderConfig {
    /// Camera angle preset.
    #[serde(default = "default_depth_camera")]
    pub camera_angle: DepthCameraAngle,
    /// Custom camera position (for Custom angle).
    pub custom_position: Option<[f32; 3]>,
    /// Custom look-at target (for Custom angle).
    pub custom_look_at: Option<[f32; 3]>,
    /// Output resolution [width, height].
    #[serde(default = "default_resolution")]
    pub resolution: [u32; 2],
    /// Near clipping plane distance.
    #[serde(default = "default_near")]
    pub near_plane: f32,
    /// Far clipping plane distance.
    #[serde(default = "default_far")]
    pub far_plane: f32,
    /// Whether to add Gaussian noise to reduce grid artifacts.
    #[serde(default = "default_true")]
    pub add_noise: bool,
}

fn default_depth_camera() -> DepthCameraAngle {
    DepthCameraAngle::Isometric
}
fn default_resolution() -> [u32; 2] {
    [1024, 1024]
}
fn default_near() -> f32 {
    0.1
}
fn default_far() -> f32 {
    200.0
}
fn default_true() -> bool {
    true
}

impl Default for DepthRenderConfig {
    fn default() -> Self {
        Self {
            camera_angle: DepthCameraAngle::Isometric,
            custom_position: None,
            custom_look_at: None,
            resolution: [1024, 1024],
            near_plane: 0.1,
            far_plane: 200.0,
            add_noise: true,
        }
    }
}

/// Camera angle presets for depth rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DepthCameraAngle {
    /// 45 degree elevation, orthographic projection.
    Isometric,
    /// Directly above, looking down, orthographic.
    TopDown,
    /// Ground level, perspective.
    Front,
    /// User-specified position and target.
    Custom,
}

/// Result of a depth render operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepthRenderResult {
    pub path: String,
    pub width: u32,
    pub height: u32,
    pub depth_range: [f32; 2],
}

/// Compute camera position and orientation for a depth render.
pub fn compute_depth_camera(
    config: &DepthRenderConfig,
    scene_center: [f32; 3],
    scene_extent: f32,
) -> DepthCameraSetup {
    match config.camera_angle {
        DepthCameraAngle::Isometric => {
            let dist = scene_extent * 2.0 + 20.0;
            DepthCameraSetup {
                position: [
                    scene_center[0] + dist * 0.707,
                    scene_center[1] + dist * 0.707,
                    scene_center[2] + dist * 0.707,
                ],
                look_at: scene_center,
                orthographic: true,
                ortho_scale: scene_extent * 1.5,
            }
        }
        DepthCameraAngle::TopDown => {
            let height = scene_extent * 2.0 + 20.0;
            DepthCameraSetup {
                position: [scene_center[0], scene_center[1] + height, scene_center[2]],
                look_at: scene_center,
                orthographic: true,
                ortho_scale: scene_extent * 1.2,
            }
        }
        DepthCameraAngle::Front => {
            let dist = scene_extent * 1.5 + 10.0;
            DepthCameraSetup {
                position: [scene_center[0], scene_center[1] + 2.0, scene_center[2] + dist],
                look_at: scene_center,
                orthographic: false,
                ortho_scale: 1.0,
            }
        }
        DepthCameraAngle::Custom => {
            let pos = config.custom_position.unwrap_or([0.0, 10.0, 10.0]);
            let look = config.custom_look_at.unwrap_or([0.0, 0.0, 0.0]);
            DepthCameraSetup {
                position: pos,
                look_at: look,
                orthographic: false,
                ortho_scale: 1.0,
            }
        }
    }
}

/// Camera setup for depth rendering.
#[derive(Debug, Clone)]
pub struct DepthCameraSetup {
    pub position: [f32; 3],
    pub look_at: [f32; 3],
    pub orthographic: bool,
    pub ortho_scale: f32,
}

/// Normalize a raw depth buffer to 0.0-1.0 range, inverted (near=1, far=0).
pub fn normalize_depth(
    raw_depths: &[f32],
    near: f32,
    far: f32,
) -> Vec<f32> {
    let range = far - near;
    if range <= 0.0 {
        return vec![0.0; raw_depths.len()];
    }

    raw_depths
        .iter()
        .map(|&d| {
            let normalized = (d - near) / range;
            1.0 - normalized.clamp(0.0, 1.0) // Invert: near = white (1.0), far = black (0.0)
        })
        .collect()
}

/// Convert normalized depth values to 8-bit grayscale pixels.
pub fn depth_to_grayscale(normalized: &[f32]) -> Vec<u8> {
    normalized
        .iter()
        .map(|&d| (d * 255.0).round() as u8)
        .collect()
}

/// Write grayscale pixel data to a PNG file.
pub fn write_grayscale_png(
    path: &str,
    pixels: &[u8],
    width: u32,
    height: u32,
) -> Result<(), String> {
    use image::{GrayImage, Luma};

    let mut img = GrayImage::new(width, height);
    for (i, &val) in pixels.iter().enumerate() {
        let x = (i as u32) % width;
        let y = (i as u32) / width;
        if x < width && y < height {
            img.put_pixel(x, y, Luma([val]));
        }
    }
    let parent = std::path::Path::new(path).parent();
    if let Some(dir) = parent {
        std::fs::create_dir_all(dir).map_err(|e| format!("create dir: {e}"))?;
    }
    img.save(path).map_err(|e| format!("save PNG: {e}"))
}

/// Add Gaussian noise to depth values to reduce grid artifacts.
pub fn add_depth_noise(depths: &mut [f32], sigma: f32, seed: u32) {
    // Simple hash-based pseudo-random noise (no external dependency)
    for (i, d) in depths.iter_mut().enumerate() {
        let hash = ((i as u32).wrapping_mul(2654435761).wrapping_add(seed)) as f32 / u32::MAX as f32;
        // Box-Muller approximation: map uniform [0,1] to approximate Gaussian
        let noise = (hash - 0.5) * 2.0 * sigma;
        *d = (*d + noise).clamp(0.0, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DepthRenderConfig::default();
        assert_eq!(config.resolution, [1024, 1024]);
        assert!(config.add_noise);
    }

    #[test]
    fn test_normalize_depth() {
        let raw = vec![0.1, 1.0, 100.0, 200.0];
        let norm = normalize_depth(&raw, 0.1, 200.0);
        assert!((norm[0] - 1.0).abs() < 0.01); // Near = 1.0 (white)
        assert!((norm[3] - 0.0).abs() < 0.01); // Far = 0.0 (black)
    }

    #[test]
    fn test_depth_to_grayscale() {
        let norm = vec![1.0, 0.5, 0.0];
        let pixels = depth_to_grayscale(&norm);
        assert_eq!(pixels, vec![255, 128, 0]);
    }

    #[test]
    fn test_noise_preserves_range() {
        let mut depths = vec![0.5; 100];
        add_depth_noise(&mut depths, 0.01, 42);
        assert!(depths.iter().all(|d| *d >= 0.0 && *d <= 1.0));
    }

    #[test]
    fn test_isometric_camera() {
        let config = DepthRenderConfig::default();
        let setup = compute_depth_camera(&config, [0.0, 0.0, 0.0], 50.0);
        assert!(setup.orthographic);
        assert!(setup.position[1] > 0.0); // Camera is above
    }

    #[test]
    fn test_top_down_camera() {
        let config = DepthRenderConfig {
            camera_angle: DepthCameraAngle::TopDown,
            ..Default::default()
        };
        let setup = compute_depth_camera(&config, [0.0, 0.0, 0.0], 50.0);
        assert!(setup.orthographic);
        // Camera should be directly above
        assert!((setup.position[0] - 0.0).abs() < 0.01);
        assert!((setup.position[2] - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_custom_camera() {
        let config = DepthRenderConfig {
            camera_angle: DepthCameraAngle::Custom,
            custom_position: Some([10.0, 20.0, 30.0]),
            custom_look_at: Some([0.0, 0.0, 0.0]),
            ..Default::default()
        };
        let setup = compute_depth_camera(&config, [0.0, 0.0, 0.0], 50.0);
        assert_eq!(setup.position, [10.0, 20.0, 30.0]);
        assert_eq!(setup.look_at, [0.0, 0.0, 0.0]);
    }
}
