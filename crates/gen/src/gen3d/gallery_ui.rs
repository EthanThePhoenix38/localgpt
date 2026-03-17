//! Gallery UI — egui overlay for browsing generated worlds.
//!
//! Triggered by the `G` keybind or `/gallery` slash command.
//! Displays world cards with thumbnails, filtering, and load actions.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use super::gallery::{self, WorldGalleryEntry};
use super::plugin::GenWorkspace;

/// Gallery visibility and data state.
#[derive(Resource, Default)]
pub struct GalleryState {
    /// Whether the gallery overlay is visible.
    pub visible: bool,
    /// Scanned world entries.
    pub entries: Vec<WorldGalleryEntry>,
    /// Filter text for searching by name or tag.
    pub filter: String,
    /// Index of the selected entry (for load action).
    pub selected: Option<usize>,
    /// Path of world to load (set by "Load" button, consumed by command handler).
    pub load_request: Option<String>,
}

/// Bevy plugin that adds the gallery UI systems.
pub struct GalleryPlugin;

impl Plugin for GalleryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GalleryState>()
            .add_systems(Update, gallery_toggle_system)
            .add_systems(Update, gallery_ui_system);
    }
}

/// Toggle gallery visibility with G key (when not typing in egui).
fn gallery_toggle_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut gallery: ResMut<GalleryState>,
    workspace: Res<GenWorkspace>,
    mut contexts: EguiContexts,
) {
    // Don't toggle if egui wants keyboard input (text fields)
    if let Ok(ctx) = contexts.ctx_mut()
        && ctx.wants_keyboard_input()
    {
        return;
    }

    if keys.just_pressed(KeyCode::KeyG) {
        gallery.visible = !gallery.visible;
        if gallery.visible && gallery.entries.is_empty() {
            gallery.entries = gallery::scan_world_gallery(&workspace.path);
        }
    }
}

/// Main gallery UI system — draws the egui window.
fn gallery_ui_system(
    mut contexts: EguiContexts,
    mut gallery: ResMut<GalleryState>,
    workspace: Res<GenWorkspace>,
) {
    if !gallery.visible {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    // Collect filtered indices upfront to avoid borrow conflicts
    let filter_lower = gallery.filter.to_lowercase();
    let filtered_indices: Vec<usize> = gallery
        .entries
        .iter()
        .enumerate()
        .filter(|(_, e)| {
            filter_lower.is_empty()
                || e.name.to_lowercase().contains(&filter_lower)
                || e.style_tags
                    .iter()
                    .any(|t| t.to_lowercase().contains(&filter_lower))
                || e.source.to_lowercase().contains(&filter_lower)
        })
        .map(|(i, _)| i)
        .collect();

    let entry_count = gallery.entries.len();
    let filtered_count = filtered_indices.len();

    let mut still_open = true;

    egui::Window::new("World Gallery")
        .open(&mut still_open)
        .resizable(true)
        .default_size([800.0, 600.0])
        .show(ctx, |ui| {
            // Filter bar
            ui.horizontal(|ui| {
                ui.label("Filter:");
                ui.text_edit_singleline(&mut gallery.filter);
                if ui.button("Refresh").clicked() {
                    gallery.entries = gallery::scan_world_gallery(&workspace.path);
                }
                ui.label(format!("{} worlds", entry_count));
            });

            ui.separator();

            if filtered_count == 0 {
                ui.centered_and_justified(|ui| {
                    ui.label("No worlds found. Generate some first!");
                });
                return;
            }

            // Scrollable list of world cards
            egui::ScrollArea::vertical().show(ui, |ui| {
                for &idx in &filtered_indices {
                    let is_selected = gallery.selected == Some(idx);

                    // Read entry data before the mutable borrow
                    let name = gallery.entries[idx].name.clone();
                    let entity_count = gallery.entries[idx].entity_count;
                    let source = gallery.entries[idx].source.clone();
                    let description = gallery.entries[idx].description.clone();
                    let style_tags = gallery.entries[idx].style_tags.clone();
                    let has_thumb = gallery.entries[idx].thumbnail_path.is_some();
                    let path = gallery.entries[idx].path.to_string_lossy().to_string();
                    let date = gallery.entries[idx]
                        .created_at
                        .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
                        .unwrap_or_default();

                    let frame = if is_selected {
                        egui::Frame::new()
                            .fill(egui::Color32::from_rgba_premultiplied(60, 80, 120, 200))
                            .corner_radius(4.0)
                            .inner_margin(8.0)
                    } else {
                        egui::Frame::new()
                            .fill(egui::Color32::from_rgba_premultiplied(40, 40, 40, 200))
                            .corner_radius(4.0)
                            .inner_margin(8.0)
                    };

                    frame.show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // Thumbnail placeholder (100x75)
                            let thumb_size = egui::vec2(100.0, 75.0);
                            let (rect, _) =
                                ui.allocate_exact_size(thumb_size, egui::Sense::hover());
                            let fill = if has_thumb {
                                egui::Color32::from_gray(60)
                            } else {
                                egui::Color32::from_gray(30)
                            };
                            ui.painter().rect_filled(rect, 4.0, fill);
                            if !has_thumb {
                                ui.painter().text(
                                    rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    "No Preview",
                                    egui::FontId::proportional(11.0),
                                    egui::Color32::GRAY,
                                );
                            }

                            // Info column
                            ui.vertical(|ui| {
                                ui.strong(&name);
                                ui.label(format!(
                                    "{} entities | {} | {}",
                                    entity_count, source, date
                                ));

                                if let Some(ref desc) = description {
                                    ui.label(
                                        egui::RichText::new(desc)
                                            .small()
                                            .color(egui::Color32::GRAY),
                                    );
                                }

                                if !style_tags.is_empty() {
                                    ui.horizontal(|ui| {
                                        for tag in &style_tags {
                                            ui.label(
                                                egui::RichText::new(tag)
                                                    .small()
                                                    .background_color(egui::Color32::from_gray(50)),
                                            );
                                        }
                                    });
                                }

                                ui.horizontal(|ui| {
                                    if ui.button("Load").clicked() {
                                        gallery.selected = Some(idx);
                                        gallery.load_request = Some(path.clone());
                                    }
                                    if ui.button("Select").clicked() {
                                        gallery.selected = Some(idx);
                                    }
                                });
                            });
                        });
                    });

                    ui.add_space(4.0);
                }
            });
        });

    if !still_open {
        gallery.visible = false;
    }
}
