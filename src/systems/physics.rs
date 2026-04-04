use hotham::{
    components::LocalTransform,
    hecs::{self, CommandBuffer},
    Engine,
};

use crate::{
    components::{DynamicPhysicsBody, InsertedPhysicsBody, KinematicPhysicsBody},
    physics::Physics,
    DELTA_TIME,
};

pub fn physics_system(
    engine: &mut Engine,
    physics: &mut Physics,
    command_buffer: &mut CommandBuffer,
) {
    physics.update(DELTA_TIME);
    sync_body_positions(&engine.world, physics, command_buffer);
}

pub fn sync_body_positions(
    world: &hecs::World,
    physics: &Physics,
    command_buffer: &mut CommandBuffer,
) {
    let body_interface = physics.system.body_interface();

    // Update Kinematic Bodies
    for (entity, (maybe_inserted_body, transform, body)) in world
        .query::<(
            Option<&InsertedPhysicsBody>,
            &mut LocalTransform,
            &KinematicPhysicsBody,
        )>()
        .iter()
    {
        if let Some(inserted_body) = maybe_inserted_body {
            body_interface.set_position(
                inserted_body.body_id,
                transform.translation - body.y_offset(),
            );
            body_interface.set_rotation(inserted_body.body_id, transform.rotation);
            continue;
        }

        // No body exists, create one
        let inserted_body = physics.create_kinematic_body(transform.translation, body);

        body_interface.set_user_data(inserted_body.body_id, entity.to_bits().get());

        command_buffer.insert_one(entity, inserted_body);
    }

    // Update Dynamic Bodies
    for (entity, (maybe_inserted_body, transform, body)) in world
        .query::<(
            Option<&InsertedPhysicsBody>,
            &mut LocalTransform,
            &DynamicPhysicsBody,
        )>()
        .iter()
    {
        if let Some(inserted_body) = maybe_inserted_body {
            transform.translation =
                body_interface.position(inserted_body.body_id) + body.y_offset();
            transform.rotation = body_interface.rotation(inserted_body.body_id);
            continue;
        }

        // No body exists, create one
        let inserted_body = physics.create_dynamic_body(transform.translation, body);

        body_interface.set_user_data(inserted_body.body_id, entity.to_bits().get());

        command_buffer.insert_one(entity, inserted_body);
    }
}
