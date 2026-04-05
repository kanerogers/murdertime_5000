use hotham::{
    components::{hand::Handedness, LocalTransform},
    glam,
    hecs::{self, CommandBuffer},
    Engine,
};

use crate::{
    components::{Projectile, Weapon, WeaponKind, PROJECTILE_SPEED},
    Simulation, DELTA_TIME,
};

pub const FIRING_COOLDOWN: f32 = 0.1;

pub fn weapon_firing_system(
    engine: &mut Engine,
    simulation: &mut Simulation,
    command_buffer: &mut CommandBuffer,
) {
    let input = &engine.input_context;
    let left_input = &input.left;
    let right_input = &input.right;

    for (_, (weapon, transform)) in engine
        .world
        .query::<(&mut Weapon, &mut LocalTransform)>()
        .iter()
    {
        match (weapon.hand, &mut weapon.kind) {
            (Handedness::Left, WeaponKind::GatlingGun { cooldown }) => {
                if left_input.trigger_button() {
                    *cooldown -= DELTA_TIME;
                    if *cooldown <= 0. {
                        // Fire
                        let position = left_input.aim_position();
                        let velocity =
                            (left_input.aim_rotation() * glam::Vec3::NEG_Z) * PROJECTILE_SPEED;
                        command_buffer
                            .spawn((transform.clone(), Projectile::new(position, velocity)));
                        *cooldown = FIRING_COOLDOWN;
                    }
                } else {
                    *cooldown = 0.;
                }
            }
            (Handedness::Right, WeaponKind::GatlingGun { cooldown }) => {
                if right_input.trigger_button() {
                    *cooldown -= DELTA_TIME;
                    if *cooldown <= 0. {
                        // Fire
                        let position = right_input.aim_position();
                        let velocity =
                            (right_input.aim_rotation() * glam::Vec3::NEG_Z) * PROJECTILE_SPEED;
                        command_buffer
                            .spawn((transform.clone(), Projectile::new(position, velocity)));
                        *cooldown = FIRING_COOLDOWN;
                    }
                } else {
                    *cooldown = 0.;
                }
            }
        }
    }
}
