use std::collections::HashMap;

use hotham::{asset_importer, hecs, Engine};

use crate::components::KinematicPhysicsBody;

pub fn spawn_viking(engine: &mut Engine, models: &HashMap<String, hecs::World>) {
    let entity = asset_importer::add_model_to_world("Skeleton", models, &mut engine.world, None)
        .expect("Could not find Viking");

    engine
        .world
        .insert_one(entity, KinematicPhysicsBody::new_capsule(0.75, 0.50))
        .unwrap();
}
