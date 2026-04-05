use hotham::{components::LocalTransform, Engine};

use crate::{components::unit::Unit, Simulation};

pub fn unit_state_system(engine: &mut Engine, simulation: &mut Simulation) {
    for (entity, unit) in engine.world.query::<&mut Unit>().iter() {
        unit.update_state(simulation);
    }
}
