use hotham::{
    glam,
    hecs::{self, CommandBuffer},
    Engine,
};

use crate::{
    components::{unit::Unit, Projectile},
    graphics::line_renderer::DebugLine,
    physics::Physics,
    DamageEvent, Simulation, DELTA_TIME,
};

pub fn update_projectile_system(
    engine: &mut Engine,
    simulation: &mut Simulation,
    physics: &mut Physics,
    command_buffer: &mut CommandBuffer,
    debug_lines: &mut Vec<DebugLine>,
) {
    let mut to_despawn = Vec::new();
    let mut damage_events = Vec::new();

    let mut count = 3;
    for (entity, projectile) in engine.world.query::<&mut Projectile>().iter() {
        let start = projectile.position;
        let end = start + projectile.velocity * DELTA_TIME;

        projectile.previous_position = start;
        projectile.position = end;
        projectile.lifetime -= DELTA_TIME;

        if projectile.lifetime <= 0.0 {
            to_despawn.push(entity);
            continue;
        }

        let direction = end - start;

        if let Some(hit) = physics.raycast(start, direction) {
            damage_events.push(DamageEvent {
                target: hit.entity,
                amount: projectile.damage,
            });

            to_despawn.push(entity);
        }

        count = (count - 1) % 3;
        if count == 0 {
            debug_lines.push(DebugLine {
                start,
                end,
                colour: glam::Vec3::Z.lerp(glam::Vec3::X, projectile.lifetime / 2.0),
            })
        }
    }

    for event in damage_events {
        event.apply(&engine.world);
    }

    for entity in to_despawn {
        command_buffer.despawn(entity);
    }
}
