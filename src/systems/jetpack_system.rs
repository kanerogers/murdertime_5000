use hotham::{
    components::{hand::Handedness, LocalTransform},
    glam, Engine,
};

use crate::{components::Jetpack, DELTA_TIME};
const DEADZONE: f32 = 0.15;
const THRUST_ACCEL: f32 = 12.0;
const UPWARD_ACCEL: f32 = 20.0;
const GRAVITY: f32 = 9.8;
const DRAG: f32 = 2.5;
const MAX_SPEED: f32 = 8.0;

pub fn jetpack_system(engine: &mut Engine) {
    let input = &engine.input_context;
    let hmd_rotation = input.hmd.rotation();
    let move_stick = input.left.thumbstick_xy();

    let input = apply_deadzone(move_stick, DEADZONE);
    let input_mag = input.length().min(1.0);

    // HMD yaw-only basis
    let hmd_forward = hmd_rotation * glam::Vec3::NEG_Z;
    let forward = glam::Vec3::new(hmd_forward.x, 0.0, hmd_forward.z).normalize_or_zero();
    let right = forward.cross(glam::Vec3::Y).normalize_or_zero();

    // Horizontal move intent from stick
    let move_dir = (right * input.x + forward * input.y).normalize_or_zero();

    let mut acceleration = glam::Vec3::new(0.0, -GRAVITY, 0.0);

    if input_mag > 0.0 {
        acceleration += move_dir * THRUST_ACCEL * input_mag;
        acceleration += glam::Vec3::Y * UPWARD_ACCEL * input_mag;
        engine
            .haptic_context
            .request_haptic_feedback(10., Handedness::Left);
        engine
            .haptic_context
            .request_haptic_feedback(10., Handedness::Right);
    }

    let mut state = engine
        .world
        .get::<&mut Jetpack>(engine.stage_entity)
        .unwrap();
    let mut stage_transform = engine
        .world
        .get::<&mut LocalTransform>(engine.stage_entity)
        .unwrap();

    // Integrate
    state.velocity += acceleration * DELTA_TIME;

    // Drag
    state.velocity *= 1.0 / (1.0 + DRAG * DELTA_TIME);

    // Clamp
    let speed = state.velocity.length();
    if speed > MAX_SPEED {
        state.velocity = state.velocity / speed * MAX_SPEED;
    }

    // Move stage opposite player motion
    stage_transform.translation += state.velocity * DELTA_TIME;
    stage_transform.translation.y = stage_transform.translation.y.max(0.);
}

fn apply_deadzone(v: glam::Vec2, deadzone: f32) -> glam::Vec2 {
    let len = v.length();
    if len <= deadzone {
        glam::Vec2::ZERO
    } else {
        let scaled = (len - deadzone) / (1.0 - deadzone);
        v / len * scaled.min(1.0)
    }
}
