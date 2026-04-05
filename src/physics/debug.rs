use crate::graphics::line_renderer::{DebugLine, MAX_LINES};
use joltc_sys::{JPC_Color, JPC_DebugRendererSimpleFns, JPC_DebugRendererSimple_new, JPC_Vec3};
use rolt::IntoRolt;

struct RenderData<'a> {
    debug_lines: &'a mut Vec<DebugLine>,
}

pub fn draw_physics_lines(debug_lines: &mut Vec<DebugLine>, physics: &super::Physics) {
    let mut settings: joltc_sys::JPC_BodyManager_DrawSettings = unsafe { std::mem::zeroed() };
    settings.mDrawShape = true;
    settings.mDrawShapeWireframe = true;
    // settings.mDrawBoundingBox = true;
    settings.mDrawCenterOfMassTransform = true;
    settings.mDrawWorldTransform = true;

    let mut data = RenderData { debug_lines };

    let renderer = unsafe {
        JPC_DebugRendererSimple_new(
            &mut data as *mut _ as *mut _,
            JPC_DebugRendererSimpleFns {
                DrawLine: Some(draw_line_fn),
            },
        )
    };

    unsafe { physics.system.draw_bodies(&mut settings, renderer) };
}

unsafe extern "C" fn draw_line_fn(
    data_ptr: *const std::ffi::c_void,
    start: JPC_Vec3,
    end: JPC_Vec3,
    colour: JPC_Color,
) {
    let data = unsafe { &mut *(data_ptr as *mut RenderData) };
    let colour: rolt::Color = colour.into_rolt();
    let line = DebugLine::new(start.into_rolt(), end.into_rolt(), colour.into());
    if data.debug_lines.len() < MAX_LINES as usize {
        data.debug_lines.push(line);
    }
}
