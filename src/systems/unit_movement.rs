use hotham::{
    components::LocalTransform,
    glam::{Vec2, Vec3, Vec3Swizzles},
    hecs::{self, With},
};
use rolt::Quat;

use crate::{components::unit::Unit, Simulation, DELTA_TIME, SEPARATION_STRENGTH, UNIT_RADIUS};
pub const UNIT_GAP: f32 = 1.5;
pub const HMD_DISTANCE: f32 = 1.0;

pub fn unit_movement_system(engine: &mut hotham::Engine, simulation: &mut Simulation) {
    // 1) Gather unit entity ids so we can do pairwise work safely.
    let mut unit_entities: Vec<hecs::Entity> = Vec::new();
    for (entity, ()) in engine.world.query::<With<(), &Unit>>().iter() {
        unit_entities.push(entity);
    }

    let unit_count = unit_entities.len();
    if unit_count == 0 {
        return;
    }

    // 2) Snapshot positions (XZ) so pairwise math isn't fighting mutable borrows.
    let mut positions: Vec<Vec2> = Vec::with_capacity(unit_count);
    for entity in unit_entities.iter() {
        let unit = engine.world.get::<&Unit>(*entity).unwrap();
        positions.push(unit.position);
    }

    let hmd_position = simulation.head_pos.xz();

    // 3) Compute separation pushes.
    let minimum_distance = UNIT_RADIUS + UNIT_GAP;
    let mut separation_velocities: Vec<Vec2> = vec![Vec2::ZERO; unit_count];

    for first_index in 0..unit_count {
        // Compute velocity to separate from other units
        for second_index in (first_index + 1)..unit_count {
            let from_second_to_first = positions[first_index] - positions[second_index];
            let distance = from_second_to_first.length();

            if distance < 0.0001 {
                continue; // same position; ignore this frame
            }

            if distance < minimum_distance {
                let direction_away = from_second_to_first / distance;
                let overlap_amount = minimum_distance - distance;

                // Push them apart equally in opposite directions.
                let push = direction_away * overlap_amount * SEPARATION_STRENGTH;

                separation_velocities[first_index] += push;
                separation_velocities[second_index] -= push;
            }
        }

        // Velocity to separate from headset
        let from_first_to_headset = positions[first_index] - hmd_position;
        let distance = from_first_to_headset.length();

        if distance < 0.0001 {
            continue; // same position; ignore this frame
        }

        if distance < HMD_DISTANCE {
            let direction_away = from_first_to_headset / distance;

            // Move away from headset
            let push = direction_away * (SEPARATION_STRENGTH * 5.0);

            separation_velocities[first_index] += push;
        }
    }

    // 4) Apply movement: move-to-target + separation.
    for (unit_index, entity) in unit_entities.iter().enumerate() {
        let mut unit = engine.world.get::<&mut Unit>(*entity).unwrap();

        // Now apply the separation as an extra displacement.
        unit.position += separation_velocities[unit_index] * DELTA_TIME;

        // And now write it to the unit's transform.
        let mut transform = engine.world.get::<&mut LocalTransform>(*entity).unwrap();
        transform.translation = Vec3::new(unit.position.x, 0., unit.position.y);
        // transform.translation.y = state.terrain.height_at(transform.position); // TODO, pin to terrain
        transform.translation.y = 0.0;

        // Apply rotation
        transform.rotation = Quat::from_rotation_y(unit.rotation);
    }
}
