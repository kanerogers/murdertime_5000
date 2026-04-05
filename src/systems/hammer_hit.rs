use hotham::Engine;

use crate::{
    components::{Weapon, WeaponKind},
    physics::Physics,
    DamageEvent,
};

pub fn hammer_hit_system(engine: &mut Engine, physics: &mut Physics) {
    let mut damage_events = Vec::new();
    for (_, weapon) in engine.world.query::<&Weapon>().iter() {
        match weapon.kind {
            WeaponKind::Hammer { hit_entity } => {
                for other_entity in physics.check_for_insersecting(hit_entity, &engine.world) {
                    damage_events.push((
                        weapon.hand,
                        DamageEvent {
                            target: other_entity,
                            amount: 100.,
                        },
                    ))
                }
            }
            _ => {}
        }
    }

    for (hand, event) in damage_events {
        if event.apply(&engine.world) {
            engine.haptic_context.request_haptic_feedback(100., hand);
        }
    }
}
