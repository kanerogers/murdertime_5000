use hotham::{
    components::{animation_target::AnimationOutputs, AnimationController, LocalTransform},
    hecs, Engine,
};

use crate::{
    components::unit::{Unit, UnitStatus},
    Simulation,
};

pub fn unit_animation_system(engine: &mut Engine, _simulation: &mut Simulation) {
    for (entity, (unit, controller)) in engine
        .world
        .query::<(&Unit, &mut AnimationController)>()
        .iter()
    {
        let desired_animation = match unit.status {
            UnitStatus::Idle => "ID_10_Viking_Male_1_Idle",
            UnitStatus::Attacking { .. } => "ID_9_Viking_Male_1_Smash_Object",
            UnitStatus::Moving => "ID_11_Viking_Male_1_Walking",
            UnitStatus::Dying { .. } | UnitStatus::Dead => "ID_5_Viking_Male_1_Fall_Over",
        };

        controller.set_current_animation(desired_animation);

        match unit.status {
            UnitStatus::Dead => {
                // Final frame
                controller.current_animation_mut().unwrap().elapsed = 0.99;
            }
            _ => {
                controller.advance_animation();
            }
        }

        let animation_state = controller.current_animation().unwrap();

        for target in &animation_state.targets {
            if target.target == entity {
                continue;
            }

            apply_animation(
                &engine.world,
                target,
                animation_state.elapsed,
                animation_state.duration,
            );
        }
    }
}

/// Get the next value for an animation channel.
///
/// This implementation is loosely based on the glTF tutorial:
/// https://github.com/KhronosGroup/glTF-Tutorials/blob/main/gltfTutorial/gltfTutorial_007_Animations.md
fn apply_animation(
    world: &hecs::World,
    target: &hotham::components::AnimationTarget,
    elapsed: f32,
    duration: f32,
) {
    let mut previous_time = 0.;
    let mut next_time = f32::MAX;

    let mut previous_index = None;
    let mut next_index = None;

    let time_values = &target.times;

    if time_values.len() == 2 {
        previous_index = Some(0);
        next_index = Some(1);
    }

    for (index, time) in time_values.iter().enumerate() {
        let time = *time;

        // previous_time is the largest element from the times accessor that is smaller than elapsed
        if time > previous_time && time < elapsed {
            previous_time = time;
            previous_index = Some(index);
        }

        // next_time is the smallest element from the times accessor that is larger than elapsed
        if time < next_time && time > elapsed {
            next_time = time;
            next_index = Some(index);
        }
    }

    let Some(previous_index) = previous_index else {
        // log::debug!(
        //     "No previous index? Elapsed {elapsed:?}, Duration {duration:?}, Times: {}",
        //     time_values.len()
        // );
        return;
    };
    let Some(next_index) = next_index else {
        // log::debug!("No next index?");
        return;
    };

    // Compute the interpolation value. This is a value between 0.0 and 1.0 that describes the relative
    // position of the current_time, between the previous_time and the next_time:
    let interpolation_value = (elapsed - previous_time) / (next_time - previous_time);

    let mut transform = world.get::<&mut LocalTransform>(target.target).unwrap();

    let output_values = &target.outputs;
    match output_values {
        AnimationOutputs::Scales(scales) => {
            // Get the output values from the indices we found
            let previous_output = scales[previous_index];
            let next_output = scales[next_index];

            transform.scale = previous_output.lerp(next_output, interpolation_value);
        }
        AnimationOutputs::Translation(translations) => {
            // Get the output values from the indices we found
            let previous_output = translations[previous_index];
            let next_output = translations[next_index];

            transform.translation = previous_output.lerp(next_output, interpolation_value);
        }
        AnimationOutputs::Rotation(quats) => {
            // Get the output values from the indices we found
            let previous_output = quats[previous_index];
            let next_output = quats[next_index];

            transform.rotation = previous_output.slerp(next_output, interpolation_value);
        }
    }
}
