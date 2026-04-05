use std::collections::HashMap;

use hotham::{asset_importer, components::LocalTransform, glam, hecs, Engine};

use crate::{
    components::{unit::Unit, KinematicPhysicsBody},
    SPAWN_RADIUS, UNIT_COUNT, UNIT_HALF_HEIGHT, UNIT_RADIUS,
};

pub fn spawn_vikings(engine: &mut Engine, models: &HashMap<String, hecs::World>) {
    for (id, position) in circle_points(SPAWN_RADIUS, UNIT_COUNT)
        .into_iter()
        .enumerate()
    {
        // Create the unit entity
        let unit_entity =
            asset_importer::add_model_to_world("Skeleton", models, &mut engine.world, None)
                .expect("Could not find Viking");

        // Move it
        {
            let mut local_transform = engine
                .world
                .get::<&mut LocalTransform>(unit_entity)
                .unwrap();
            local_transform.translation.x = position.x;
            local_transform.translation.z = position.y;
        }

        // Create a Unit component
        let unit = Unit {
            id: id as u32,
            position,
            rotation: Default::default(),
            status: Default::default(),
            health: Default::default(),
        };

        // Create a physics body component
        let mut capsule = KinematicPhysicsBody::new_capsule(UNIT_HALF_HEIGHT, UNIT_RADIUS);
        capsule.y_offset = glam::Vec3::ZERO;

        // Insert them into the world
        engine.world.insert(unit_entity, (unit, capsule)).unwrap();
    }
}

fn circle_points(radius: f32, num_points: usize) -> Vec<glam::Vec2> {
    use std::f32::consts::{FRAC_PI_2, TAU};

    if num_points == 0 {
        return Vec::new();
    }

    (0..num_points)
        .map(|i| {
            let t = i as f32 / num_points as f32;
            let angle = t * TAU - FRAC_PI_2;
            glam::Vec2::new(angle.cos() * radius, angle.sin() * radius)
        })
        .collect()
}
