//! World Inspector Panel — egui overlay for inspecting 100% of world state.
//!
//! F1 cycles: Hidden → OutlinerOnly → Full → Hidden.
//! Outliner (left) shows the entity tree. Detail (right) shows all components.
//! World Info (bottom) shows global state.

pub mod detail;
pub mod outliner;
pub mod selection;
pub mod world_info;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin};

use crate::gen3d::audio::AudioEngine;
use crate::gen3d::behaviors::{BehaviorState, EntityBehaviors};
use crate::gen3d::plugin::CurrentWorld;
use crate::gen3d::registry::*;

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Inspector visibility mode.
#[derive(Default, PartialEq, Eq, Clone, Copy, Debug)]
pub enum InspectorMode {
    #[default]
    Hidden,
    OutlinerOnly,
    Full,
}

/// Current inspector state.
#[derive(Resource, Default)]
pub struct InspectorState {
    pub mode: InspectorMode,
}

/// Currently selected entity in the inspector.
#[derive(Resource, Default)]
pub struct InspectorSelection {
    pub entity: Option<Entity>,
}

/// Marker resource: present when the mouse is over an inspector panel.
/// Camera systems should check `not(resource_exists::<UiHovered>)`.
#[derive(Resource)]
pub struct UiHovered;

// ---------------------------------------------------------------------------
// SystemParam for component queries
// ---------------------------------------------------------------------------

#[derive(SystemParam)]
pub struct InspectorQueries<'w, 's> {
    pub gen_entities: Query<'w, 's, &'static GenEntity>,
    pub transforms: Query<'w, 's, &'static Transform>,
    pub shapes: Query<'w, 's, &'static ParametricShape>,
    pub material_handles: Query<'w, 's, &'static MeshMaterial3d<StandardMaterial>>,
    pub materials: Res<'w, Assets<StandardMaterial>>,
    pub point_lights: Query<'w, 's, &'static PointLight>,
    pub dir_lights: Query<'w, 's, &'static DirectionalLight>,
    pub spot_lights: Query<'w, 's, &'static SpotLight>,
    pub behaviors_q: Query<'w, 's, &'static EntityBehaviors>,
    pub gltf_sources: Query<'w, 's, &'static GltfSource>,
    pub visibility_q: Query<'w, 's, &'static Visibility>,
    pub children_q: Query<'w, 's, &'static Children>,
    pub parent_q: Query<'w, 's, &'static ChildOf>,
    pub behavior_state: Res<'w, BehaviorState>,
    pub audio_engine: Option<Res<'w, AudioEngine>>,
    pub current_world: Res<'w, CurrentWorld>,
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .init_resource::<InspectorState>()
            .init_resource::<InspectorSelection>()
            .init_resource::<outliner::OutlinerCache>()
            .add_systems(
                Update,
                (
                    inspector_toggle,
                    inspector_ui,
                    selection::highlight_selected,
                )
                    .chain(),
            );
    }
}

/// Run condition: true when mouse is NOT over an inspector panel.
pub fn not_ui_hovered(hovered: Option<Res<UiHovered>>) -> bool {
    hovered.is_none()
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// F1 cycles through inspector modes.
fn inspector_toggle(keys: Res<ButtonInput<KeyCode>>, mut state: ResMut<InspectorState>) {
    if keys.just_pressed(KeyCode::F1) {
        state.mode = match state.mode {
            InspectorMode::Hidden => InspectorMode::OutlinerOnly,
            InspectorMode::OutlinerOnly => InspectorMode::Full,
            InspectorMode::Full => InspectorMode::Hidden,
        };
    }
}

/// Main inspector UI system — draws all egui panels and updates UiHovered.
fn inspector_ui(
    mut commands: Commands,
    mut contexts: EguiContexts,
    state: Res<InspectorState>,
    mut selection: ResMut<InspectorSelection>,
    registry: Res<NameRegistry>,
    mut outliner_cache: ResMut<outliner::OutlinerCache>,
    params: InspectorQueries,
) {
    if state.mode == InspectorMode::Hidden {
        commands.remove_resource::<UiHovered>();
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    // --- Outliner (left panel) ---
    outliner::draw_outliner(
        ctx,
        &registry,
        &mut selection,
        &mut outliner_cache,
        &params.gen_entities,
        &params.visibility_q,
        &params.children_q,
        &params.parent_q,
    );

    if state.mode == InspectorMode::Full {
        // --- Detail panel (right) ---
        detail::draw_detail(ctx, &selection, &registry, &params);

        // --- World info bar (bottom) ---
        world_info::draw_world_info(
            ctx,
            &registry,
            &params.behavior_state,
            params.audio_engine.as_deref(),
            &params.current_world,
        );
    }

    // --- Update UiHovered resource ---
    if ctx.is_pointer_over_area() {
        commands.insert_resource(UiHovered);
    } else {
        commands.remove_resource::<UiHovered>();
    }
}
