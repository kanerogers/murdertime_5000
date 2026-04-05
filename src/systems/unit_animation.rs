use hotham::{
    components::{AnimationController, LocalTransform},
    Engine,
};

use crate::{components::unit::Unit, Simulation, DELTA_TIME};

pub fn unit_animation_system(engine: &mut Engine, simulation: &mut Simulation) {
    for (_, (unit, controller)) in engine
        .world
        .query::<(&Unit, &mut AnimationController)>()
        .iter()
    {
        controller.advance_animation();
        controller.set_current_animation("ID_11_Viking_Male_1_Walking");

        let Some(animation_state) = controller.current_animation() else {
            continue;
        };

        for target in &animation_state.targets {
            let local_transform = engine
                .world
                .get::<&mut LocalTransform>(target.target)
                .unwrap();
        }
    }
}
