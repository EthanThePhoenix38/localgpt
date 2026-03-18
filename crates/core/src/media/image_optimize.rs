//! Image optimization for vision models.
//!
//! Resizes and compresses images before sending to LLM providers to reduce
//! token cost and stay within API size limits. Inspired by Moltis's image_ops.

use anyhow::{Context, Result};
use image::imageops::FilterType;
use image::{DynamicImage, ImageFormat, ImageReader};
use std::io::Cursor;
use tracing::debug;

/// Default maximum dimension (width or height) in pixels.
pub const DEFAULT_MAX_DIMENSION: u32 = 1568;

/// Default maximum file size in bytes (5MB).
pub const DEFAULT_MAX_BYTES: usize = 5 * 1024 * 1024;

/// Default JPEG quality (0-100).
const DEFAULT_JPEG_QUALITY: u8 = 85;

/// Quality reduction steps when image is still over max_bytes.
const QUALITY_STEPS: &[u8] = &[80, 70, 60, 50, 40, 30];

/// Configuration for image optimization.
#[derive(Debug, Clone)]
pub struct ImageOptConfig {
    /// Maximum width or height in pixels. 0 = disabled.
    pub max_dimension: u32,
    /// Maximum file size in bytes.
    pub max_bytes: usize,
    /// Initial JPEG quality (0-100).
    pub jpeg_quality: u8,
}

impl Default for ImageOptConfig {
    fn default() -> Self {
        Self {
            max_dimension: DEFAULT_MAX_DIMENSION,
            max_bytes: DEFAULT_MAX_BYTES,
            jpeg_quality: DEFAULT_JPEG_QUALITY,
        }
    }
}

/// Result of image optimization.
#[derive(Debug)]
pub struct OptimizedImage {
    /// Optimized image data (raw bytes, not base64).
    pub data: Vec<u8>,
    /// MIME type of the output (may differ from input if PNG→JPEG).
    pub media_type: String,
    /// Whether the image was modified.
    pub was_resized: bool,
    /// Original dimensions (width, height).
    pub original_dimensions: (u32, u32),
    /// Final dimensions (width, height).
    pub final_dimensions: (u32, u32),
}

/// Optimize an image for LLM consumption.
///
/// - Decodes the image
/// - Resizes if dimensions exceed `max_dimension` (preserving aspect ratio)
/// - Converts opaque PNGs to JPEG for smaller size
/// - Reduces JPEG quality if still over `max_bytes`
///
/// Returns the original data unchanged if already within limits.
pub fn optimize_for_llm(
    data: &[u8],
    _media_type: &str,
    config: &ImageOptConfig,
) -> Result<OptimizedImage> {
    if config.max_dimension == 0 {
        // Optimization disabled
        return Ok(OptimizedImage {
            data: data.to_vec(),
            media_type: _media_type.to_string(),
            was_resized: false,
            original_dimensions: (0, 0),
            final_dimensions: (0, 0),
        });
    }

    let reader = ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .context("Failed to detect image format")?;

    let img = reader.decode().context("Failed to decode image")?;
    let (orig_w, orig_h) = (img.width(), img.height());

    let needs_resize = orig_w > config.max_dimension || orig_h > config.max_dimension;
    let needs_compress = data.len() > config.max_bytes;

    if !needs_resize && !needs_compress {
        return Ok(OptimizedImage {
            data: data.to_vec(),
            media_type: _media_type.to_string(),
            was_resized: false,
            original_dimensions: (orig_w, orig_h),
            final_dimensions: (orig_w, orig_h),
        });
    }

    // Resize if needed
    let resized = if needs_resize {
        let (new_w, new_h) = fit_dimensions(orig_w, orig_h, config.max_dimension);
        debug!(
            "Resizing image: {}x{} -> {}x{}",
            orig_w, orig_h, new_w, new_h
        );
        img.resize(new_w, new_h, FilterType::Lanczos3)
    } else {
        img
    };

    let (final_w, final_h) = (resized.width(), resized.height());
    let has_alpha = has_meaningful_alpha(&resized);

    // Encode: PNG for alpha, JPEG for everything else
    let (encoded, media_type) = if has_alpha {
        let mut buf = Vec::new();
        resized
            .write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)
            .context("Failed to encode PNG")?;
        (buf, "image/png".to_string())
    } else {
        encode_jpeg(&resized, config.jpeg_quality)?
    };

    // If still over max_bytes and JPEG, reduce quality
    let (final_data, final_type) = if encoded.len() > config.max_bytes && !has_alpha {
        reduce_jpeg_quality(&resized, config.max_bytes)?
    } else {
        (encoded, media_type)
    };

    debug!(
        "Image optimized: {}x{} -> {}x{}, {} -> {} bytes ({})",
        orig_w,
        orig_h,
        final_w,
        final_h,
        data.len(),
        final_data.len(),
        final_type
    );

    Ok(OptimizedImage {
        data: final_data,
        media_type: final_type,
        was_resized: true,
        original_dimensions: (orig_w, orig_h),
        final_dimensions: (final_w, final_h),
    })
}

/// Calculate new dimensions that fit within max_dimension, preserving aspect ratio.
fn fit_dimensions(width: u32, height: u32, max_dim: u32) -> (u32, u32) {
    if width <= max_dim && height <= max_dim {
        return (width, height);
    }
    let ratio = width as f64 / height as f64;
    if width >= height {
        let new_w = max_dim;
        let new_h = (new_w as f64 / ratio).round() as u32;
        (new_w, new_h.max(1))
    } else {
        let new_h = max_dim;
        let new_w = (new_h as f64 * ratio).round() as u32;
        (new_w.max(1), new_h)
    }
}

/// Check if an image has meaningful alpha (not all-opaque).
fn has_meaningful_alpha(img: &DynamicImage) -> bool {
    match img {
        DynamicImage::ImageRgba8(rgba) => rgba.pixels().any(|p| p.0[3] < 255),
        DynamicImage::ImageRgba16(rgba) => rgba.pixels().any(|p| p.0[3] < 65535),
        DynamicImage::ImageRgba32F(rgba) => rgba.pixels().any(|p| p.0[3] < 1.0),
        DynamicImage::ImageLumaA8(la) => la.pixels().any(|p| p.0[1] < 255),
        DynamicImage::ImageLumaA16(la) => la.pixels().any(|p| p.0[1] < 65535),
        _ => false, // RGB formats have no alpha
    }
}

/// Encode as JPEG with given quality.
fn encode_jpeg(img: &DynamicImage, quality: u8) -> Result<(Vec<u8>, String)> {
    let rgb = img.to_rgb8();
    let mut buf = Vec::new();
    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, quality);
    rgb.write_with_encoder(encoder)
        .context("Failed to encode JPEG")?;
    Ok((buf, "image/jpeg".to_string()))
}

/// Progressively reduce JPEG quality to fit within max_bytes.
fn reduce_jpeg_quality(img: &DynamicImage, max_bytes: usize) -> Result<(Vec<u8>, String)> {
    for &quality in QUALITY_STEPS {
        let (encoded, media_type) = encode_jpeg(img, quality)?;
        if encoded.len() <= max_bytes {
            debug!(
                "Quality reduced to {} to fit {} bytes (got {} bytes)",
                quality,
                max_bytes,
                encoded.len()
            );
            return Ok((encoded, media_type));
        }
    }
    // Return lowest quality even if still over limit (best effort)
    let last_quality = *QUALITY_STEPS.last().unwrap_or(&30);
    encode_jpeg(img, last_quality)
}

/// Convenience: optimize a base64-encoded image, returning base64 output.
/// This is the main entry point for provider integration.
pub fn optimize_base64_image(
    base64_data: &str,
    media_type: &str,
    config: &ImageOptConfig,
) -> Result<(String, String)> {
    use base64::Engine;
    let raw = base64::engine::general_purpose::STANDARD
        .decode(base64_data)
        .context("Invalid base64 image data")?;

    let result = optimize_for_llm(&raw, media_type, config)?;

    let encoded = base64::engine::general_purpose::STANDARD.encode(&result.data);
    Ok((encoded, result.media_type))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a small test PNG (2x2 red pixels, opaque).
    fn make_small_png() -> Vec<u8> {
        let img = DynamicImage::ImageRgb8(image::RgbImage::from_fn(2, 2, |_, _| {
            image::Rgb([255, 0, 0])
        }));
        let mut buf = Vec::new();
        img.write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)
            .unwrap();
        buf
    }

    /// Create a large test PNG (4000x3000 gradient).
    fn make_large_png() -> Vec<u8> {
        let img = DynamicImage::ImageRgb8(image::RgbImage::from_fn(4000, 3000, |x, y| {
            image::Rgb([(x % 256) as u8, (y % 256) as u8, 128])
        }));
        let mut buf = Vec::new();
        img.write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)
            .unwrap();
        buf
    }

    /// Create a PNG with actual transparency.
    fn make_alpha_png() -> Vec<u8> {
        let img = DynamicImage::ImageRgba8(image::RgbaImage::from_fn(100, 100, |x, _| {
            image::Rgba([255, 0, 0, if x < 50 { 128 } else { 255 }])
        }));
        let mut buf = Vec::new();
        img.write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)
            .unwrap();
        buf
    }

    #[test]
    fn test_small_image_passthrough() {
        let data = make_small_png();
        let config = ImageOptConfig::default();
        let result = optimize_for_llm(&data, "image/png", &config).unwrap();
        assert!(!result.was_resized);
        assert_eq!(result.data, data);
        assert_eq!(result.original_dimensions, (2, 2));
        assert_eq!(result.final_dimensions, (2, 2));
    }

    #[test]
    fn test_large_image_resized() {
        let data = make_large_png();
        let config = ImageOptConfig {
            max_dimension: 800,
            ..Default::default()
        };
        let result = optimize_for_llm(&data, "image/png", &config).unwrap();
        assert!(result.was_resized);
        assert_eq!(result.original_dimensions, (4000, 3000));
        assert!(result.final_dimensions.0 <= 800);
        assert!(result.final_dimensions.1 <= 800);
        // Aspect ratio preserved (4:3)
        let ratio = result.final_dimensions.0 as f64 / result.final_dimensions.1 as f64;
        assert!((ratio - 4.0 / 3.0).abs() < 0.05);
    }

    #[test]
    fn test_opaque_png_converts_to_jpeg() {
        let data = make_large_png();
        let config = ImageOptConfig {
            max_dimension: 800,
            ..Default::default()
        };
        let result = optimize_for_llm(&data, "image/png", &config).unwrap();
        assert_eq!(result.media_type, "image/jpeg");
        assert!(result.data.len() < data.len()); // JPEG should be smaller
    }

    #[test]
    fn test_alpha_png_stays_png() {
        let data = make_alpha_png();
        let config = ImageOptConfig {
            max_dimension: 50,
            ..Default::default()
        };
        let result = optimize_for_llm(&data, "image/png", &config).unwrap();
        assert_eq!(result.media_type, "image/png");
    }

    #[test]
    fn test_corrupt_data_returns_error() {
        let result = optimize_for_llm(b"not an image", "image/png", &ImageOptConfig::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_disabled_passthrough() {
        let data = make_large_png();
        let config = ImageOptConfig {
            max_dimension: 0, // disabled
            ..Default::default()
        };
        let result = optimize_for_llm(&data, "image/png", &config).unwrap();
        assert!(!result.was_resized);
        assert_eq!(result.data, data);
    }

    #[test]
    fn test_fit_dimensions() {
        // Landscape
        assert_eq!(fit_dimensions(4000, 3000, 1568), (1568, 1176));
        // Portrait
        assert_eq!(fit_dimensions(3000, 4000, 1568), (1176, 1568));
        // Square
        assert_eq!(fit_dimensions(2000, 2000, 1568), (1568, 1568));
        // Already fits
        assert_eq!(fit_dimensions(800, 600, 1568), (800, 600));
    }

    #[test]
    fn test_quality_reduction() {
        let data = make_large_png();
        let config = ImageOptConfig {
            max_dimension: 1568,
            max_bytes: 50_000, // Very tight limit
            jpeg_quality: 85,
        };
        let result = optimize_for_llm(&data, "image/png", &config).unwrap();
        // Should have attempted quality reduction
        assert_eq!(result.media_type, "image/jpeg");
        assert!(result.was_resized);
    }

    #[test]
    fn test_optimize_base64_roundtrip() {
        use base64::Engine;
        let data = make_small_png();
        let b64 = base64::engine::general_purpose::STANDARD.encode(&data);

        let (result_b64, result_type) =
            optimize_base64_image(&b64, "image/png", &ImageOptConfig::default()).unwrap();

        // Small image should pass through (same base64)
        assert_eq!(result_b64, b64);
        assert_eq!(result_type, "image/png");
    }
}
