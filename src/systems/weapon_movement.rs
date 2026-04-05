use hotham::{
    components::{hand::Handedness, LocalTransform},
    glam, Engine,
};

use crate::{
    components::{Weapon, WeaponKind},
    Simulation,
};

pub fn weapon_movement_system(engine: &mut Engine, simulation: &mut Simulation) {
    let input = &engine.input_context;
    let left_input = &input.left;
    let right_input = &input.right;

    let stage_transform = engine
        .world
        .get::<&LocalTransform>(engine.stage_entity)
        .unwrap();

    for (_, (weapon, weapon_transform)) in engine
        .world
        .query::<(&Weapon, &mut LocalTransform)>()
        .iter()
    {
        match (weapon.hand, &weapon.kind) {
            (Handedness::Left, WeaponKind::GatlingGun { .. }) => {
                weapon_transform.translation =
                    stage_transform.translation + left_input.aim_position();
                weapon_transform.rotation = left_input.aim_rotation();
            }
            (Handedness::Right, WeaponKind::GatlingGun { .. }) => {
                weapon_transform.translation =
                    stage_transform.translation + right_input.aim_position();
                weapon_transform.rotation = right_input.aim_rotation();
            }
            (Handedness::Left, WeaponKind::Hammer { hit_entity }) => {
                weapon_transform.translation =
                    stage_transform.translation + left_input.grip_position();
                weapon_transform.rotation = left_input.grip_rotation();

                let mut hit_capsule_transform = engine
                    .world
                    .get::<&mut LocalTransform>(*hit_entity)
                    .unwrap();
                hit_capsule_transform.translation = weapon_transform.translation
                    + ((weapon_transform.rotation * glam::Vec3::NEG_Z) * 0.7);
                hit_capsule_transform.rotation = weapon_transform.rotation;
            }
            (Handedness::Right, WeaponKind::Hammer { hit_entity }) => {
                weapon_transform.translation =
                    stage_transform.translation + right_input.grip_position();
                weapon_transform.rotation = right_input.grip_rotation();

                let mut hit_capsule_transform = engine
                    .world
                    .get::<&mut LocalTransform>(*hit_entity)
                    .unwrap();
                hit_capsule_transform.translation = weapon_transform.translation
                    + ((weapon_transform.rotation * glam::Vec3::NEG_Z) * 0.7);
                hit_capsule_transform.rotation = weapon_transform.rotation;
            }
            _ => {}
        }
    }
}
