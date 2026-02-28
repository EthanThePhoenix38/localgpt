//! Behavior system — declarative, data-driven entity behaviors.
//!
//! Each behavior is a serializable definition that the tick system evaluates
//! every frame. Behaviors modify entity transforms (position, rotation, scale)
//! based on elapsed time, producing continuous animation from pure data.

use bevy::prelude::*;
use std::collections::HashMap;

use super::commands::*;
use super::registry::NameRegistry;

// ---------------------------------------------------------------------------
// Bevy components & resources
// ---------------------------------------------------------------------------

/// A single active behavior instance on an entity.
#[derive(Clone, Debug)]
pub struct BehaviorInstance {
    pub id: String,
    pub def: BehaviorDef,
    /// Base position captured when the behavior was added (for additive behaviors like Bob).
    pub base_position: Vec3,
    /// Base scale captured when added (for Pulse).
    pub base_scale: Vec3,
}

/// Component holding all behaviors for a gen entity.
#[derive(Component, Default, Clone, Debug)]
pub struct EntityBehaviors {
    pub behaviors: Vec<BehaviorInstance>,
}

/// Global behavior state.
#[derive(Resource)]
pub struct BehaviorState {
    pub paused: bool,
    /// Monotonic time accumulator (seconds). Not wall-clock — pauses freeze it.
    pub elapsed: f64,
    /// Counter for generating unique behavior IDs.
    next_id: u64,
}

impl Default for BehaviorState {
    fn default() -> Self {
        Self {
            paused: false,
            elapsed: 0.0,
            next_id: 1,
        }
    }
}

impl BehaviorState {
    pub fn next_id(&mut self) -> String {
        let id = format!("b{}", self.next_id);
        self.next_id += 1;
        id
    }
}

// ---------------------------------------------------------------------------
// Behavior tick system
// ---------------------------------------------------------------------------

/// Advance behavior time and apply all behaviors to their entities.
pub fn behavior_tick(
    time: Res<Time>,
    mut state: ResMut<BehaviorState>,
    registry: Res<NameRegistry>,
    mut query: Query<(Entity, &mut Transform, &mut EntityBehaviors)>,
) {
    if state.paused {
        return;
    }

    state.elapsed += time.delta_secs_f64();
    let t = state.elapsed;

    // Pre-collect center positions for orbit behaviors that reference entities.
    let mut center_positions: HashMap<String, Vec3> = HashMap::new();
    for (_, transform, _) in query.iter() {
        // We'll look up by name from registry on demand
        let _ = transform;
    }
    // Build name→position map for lookups
    for (name, entity) in registry.all_names() {
        if let Ok((_, transform, _)) = query.get(entity) {
            center_positions.insert(name.to_string(), transform.translation);
        }
    }

    for (_entity, mut transform, behaviors) in query.iter_mut() {
        if behaviors.behaviors.is_empty() {
            continue;
        }

        for behavior in &behaviors.behaviors {
            apply_behavior(
                &behavior.def,
                behavior,
                t,
                &center_positions,
                &mut transform,
            );
        }
    }
}

fn apply_behavior(
    def: &BehaviorDef,
    instance: &BehaviorInstance,
    t: f64,
    centers: &HashMap<String, Vec3>,
    transform: &mut Transform,
) {
    match def {
        BehaviorDef::Orbit {
            center,
            center_point,
            radius,
            speed,
            axis,
            phase,
            tilt,
        } => {
            let center_pos = if let Some(name) = center {
                centers.get(name).copied().unwrap_or(Vec3::ZERO)
            } else if let Some(point) = center_point {
                Vec3::from_array(*point)
            } else {
                Vec3::ZERO
            };

            let angle_rad = (speed * t as f32 + phase).to_radians();
            let orbit_axis = Vec3::from_array(*axis).normalize_or(Vec3::Y);

            // Build an orthonormal basis around the orbit axis
            let (tangent, bitangent) = orbit_axis.any_orthonormal_pair();

            // Apply tilt (rotation of the orbit plane)
            let tilt_rad = tilt.to_radians();
            let tilted_tangent =
                tangent * tilt_rad.cos() + orbit_axis.cross(tangent) * tilt_rad.sin();

            let offset = (tilted_tangent * angle_rad.cos() + bitangent * angle_rad.sin()) * *radius;

            transform.translation = center_pos + offset;
        }

        BehaviorDef::Spin { axis, speed } => {
            let spin_axis = Vec3::from_array(*axis).normalize_or(Vec3::Y);
            let delta_angle = speed.to_radians() * (t as f32 - (t - 1.0 / 60.0).max(0.0) as f32);
            // Accumulate rotation directly from total elapsed time for stability
            let total_angle = speed.to_radians() * t as f32;
            let base_rotation = Quat::from_axis_angle(spin_axis, total_angle);
            transform.rotation = base_rotation;
            let _ = delta_angle; // used total angle approach instead
        }

        BehaviorDef::Bob {
            axis,
            amplitude,
            frequency,
            phase,
        } => {
            let bob_axis = Vec3::from_array(*axis).normalize_or(Vec3::Y);
            let phase_rad = phase.to_radians();
            let offset =
                (2.0 * std::f32::consts::PI * frequency * t as f32 + phase_rad).sin() * *amplitude;
            // Apply bob relative to base position
            // Only modify the component along the bob axis, preserving other position changes
            let current_along_axis = transform.translation.dot(bob_axis);
            let base_along_axis = instance.base_position.dot(bob_axis);
            let diff = (base_along_axis + offset) - current_along_axis;
            transform.translation += bob_axis * diff;
        }

        BehaviorDef::LookAt { target } => {
            if let Some(target_pos) = centers.get(target) {
                let direction = *target_pos - transform.translation;
                if direction.length_squared() > 0.0001 {
                    transform.look_at(*target_pos, Vec3::Y);
                }
            }
        }

        BehaviorDef::Pulse {
            min_scale,
            max_scale,
            frequency,
        } => {
            let t_val = (2.0 * std::f32::consts::PI * frequency * t as f32).sin();
            let scale_factor = *min_scale + (*max_scale - *min_scale) * (t_val * 0.5 + 0.5);
            transform.scale = instance.base_scale * scale_factor;
        }
    }
}

// ---------------------------------------------------------------------------
// Command handlers
// ---------------------------------------------------------------------------

pub fn handle_add_behavior(
    cmd: AddBehaviorCmd,
    state: &mut BehaviorState,
    commands: &mut Commands,
    registry: &NameRegistry,
    transforms: &Query<&Transform>,
    behaviors_query: &mut Query<&mut EntityBehaviors>,
) -> GenResponse {
    let Some(entity) = registry.get_entity(&cmd.entity) else {
        return GenResponse::Error {
            message: format!("Entity '{}' not found", cmd.entity),
        };
    };

    let behavior_id = cmd.behavior_id.unwrap_or_else(|| state.next_id());

    let base_transform = transforms.get(entity).copied().unwrap_or_default();

    let instance = BehaviorInstance {
        id: behavior_id.clone(),
        def: cmd.behavior,
        base_position: base_transform.translation,
        base_scale: base_transform.scale,
    };

    if let Ok(mut behaviors) = behaviors_query.get_mut(entity) {
        behaviors.behaviors.push(instance);
    } else {
        commands.entity(entity).insert(EntityBehaviors {
            behaviors: vec![instance],
        });
    }

    GenResponse::BehaviorAdded {
        entity: cmd.entity,
        behavior_id,
    }
}

pub fn handle_remove_behavior(
    entity_name: &str,
    behavior_id: Option<&str>,
    registry: &NameRegistry,
    behaviors_query: &mut Query<&mut EntityBehaviors>,
) -> GenResponse {
    let Some(entity) = registry.get_entity(entity_name) else {
        return GenResponse::Error {
            message: format!("Entity '{}' not found", entity_name),
        };
    };

    let Ok(mut behaviors) = behaviors_query.get_mut(entity) else {
        return GenResponse::BehaviorRemoved {
            entity: entity_name.to_string(),
            count: 0,
        };
    };

    let before = behaviors.behaviors.len();
    if let Some(id) = behavior_id {
        behaviors.behaviors.retain(|b| b.id != id);
    } else {
        behaviors.behaviors.clear();
    }
    let removed = before - behaviors.behaviors.len();

    GenResponse::BehaviorRemoved {
        entity: entity_name.to_string(),
        count: removed,
    }
}

pub fn handle_list_behaviors(
    entity_filter: Option<&str>,
    state: &BehaviorState,
    registry: &NameRegistry,
    behaviors_query: &Query<&mut EntityBehaviors>,
) -> GenResponse {
    let mut entities_summary = Vec::new();

    for (name, entity) in registry.all_names() {
        if let Some(filter) = entity_filter
            && name != filter
        {
            continue;
        }

        let Ok(behaviors) = behaviors_query.get(entity) else {
            continue;
        };

        if behaviors.behaviors.is_empty() {
            continue;
        }

        let summaries: Vec<BehaviorSummary> = behaviors
            .behaviors
            .iter()
            .map(|b| BehaviorSummary {
                id: b.id.clone(),
                behavior_type: behavior_type_name(&b.def),
                description: behavior_description(&b.def),
            })
            .collect();

        entities_summary.push(EntityBehaviorsSummary {
            entity: name.to_string(),
            behaviors: summaries,
        });
    }

    GenResponse::BehaviorList(BehaviorListResponse {
        paused: state.paused,
        entities: entities_summary,
    })
}

fn behavior_type_name(def: &BehaviorDef) -> String {
    match def {
        BehaviorDef::Orbit { .. } => "orbit".to_string(),
        BehaviorDef::Spin { .. } => "spin".to_string(),
        BehaviorDef::Bob { .. } => "bob".to_string(),
        BehaviorDef::LookAt { .. } => "look_at".to_string(),
        BehaviorDef::Pulse { .. } => "pulse".to_string(),
    }
}

fn behavior_description(def: &BehaviorDef) -> String {
    match def {
        BehaviorDef::Orbit {
            center,
            center_point,
            radius,
            speed,
            ..
        } => {
            let around = center
                .as_deref()
                .map(|s| format!("'{}'", s))
                .or_else(|| center_point.map(|p| format!("[{},{},{}]", p[0], p[1], p[2])))
                .unwrap_or_else(|| "origin".to_string());
            format!(
                "Orbit around {} at radius {:.1}, {:.1} deg/s",
                around, radius, speed
            )
        }
        BehaviorDef::Spin { axis, speed } => {
            format!(
                "Spin [{:.1},{:.1},{:.1}] at {:.1} deg/s",
                axis[0], axis[1], axis[2], speed
            )
        }
        BehaviorDef::Bob {
            amplitude,
            frequency,
            ..
        } => format!("Bob amp={:.2} freq={:.2}Hz", amplitude, frequency),
        BehaviorDef::LookAt { target } => format!("Look at '{}'", target),
        BehaviorDef::Pulse {
            min_scale,
            max_scale,
            frequency,
        } => format!(
            "Pulse {:.2}-{:.2}x at {:.2}Hz",
            min_scale, max_scale, frequency
        ),
    }
}

// ---------------------------------------------------------------------------
// Serialization helpers (for world save/load)
// ---------------------------------------------------------------------------

/// Collect all behaviors from the ECS for serialization.
pub fn collect_all_behaviors(
    registry: &NameRegistry,
    behaviors_query: &Query<&mut EntityBehaviors>,
) -> Vec<(String, Vec<BehaviorDef>)> {
    let mut result = Vec::new();
    for (name, entity) in registry.all_names() {
        if let Ok(behaviors) = behaviors_query.get(entity)
            && !behaviors.behaviors.is_empty()
        {
            let defs: Vec<BehaviorDef> =
                behaviors.behaviors.iter().map(|b| b.def.clone()).collect();
            result.push((name.to_string(), defs));
        }
    }
    result
}
