use hotham::{
    components::{hand::Handedness, LocalTransform},
    Engine,
};

use crate::{
    components::{Weapon, WeaponKind},
    Simulation,
};

pub fn weapon_movement_system(engine: &mut Engine, simulation: &mut Simulation) {
    let input = &engine.input_context;
    let left_input = &input.left;
    let right_input = &input.right;

    for (_, (weapon, transform)) in engine
        .world
        .query::<(&Weapon, &mut LocalTransform)>()
        .iter()
    {
        match (weapon.hand, &weapon.kind) {
            (Handedness::Left, WeaponKind::GatlingGun { .. }) => {
                transform.translation = left_input.aim_position();
                transform.rotation = left_input.aim_rotation();
            }
            (Handedness::Right, WeaponKind::GatlingGun { .. }) => {
                transform.translation = right_input.aim_position();
                transform.rotation = right_input.aim_rotation();
            }
        }
    }
}
