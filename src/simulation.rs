use hotham::{glam, rendering::camera::Frustum, xr, Engine};

use crate::terrain::Terrain;

#[derive(Debug, Clone, Default)]
pub struct Simulation {
    pub left_hand_pos: glam::Vec3,
    pub right_hand_pos: glam::Vec3,
    pub head_pos: glam::Vec3,
    pub view_proj: [glam::Mat4; 2],
    pub camera_up: [glam::Vec3; 2],
    pub camera_right: [glam::Vec3; 2],
    pub viewport_size: glam::Vec2,
    pub terrain: Terrain,
}

impl Simulation {
    pub fn update(&mut self, engine: &mut Engine, views: &[xr::View]) {
        self.left_hand_pos = engine.input_context.left.grip_position();
        self.right_hand_pos = engine.input_context.right.grip_position();

        // Create transformations to globally oriented stage space
        let global_from_stage = hotham::components::stage::get_global_from_stage(&engine.world);
        self.head_pos =
            glam::Vec3::from(global_from_stage.translation) + engine.input_context.hmd.position();

        // `gos_from_global` is just the inverse of `global_from_stage`'s translation - rotation is ignored.
        let gos_from_global =
            glam::Affine3A::from_translation(glam::Vec3::from(global_from_stage.translation))
                .inverse();

        let gos_from_stage: glam::Affine3A = gos_from_global * global_from_stage;

        let view_matrices = &engine
            .render_context
            .cameras
            .iter_mut()
            .enumerate()
            .map(|(n, c)| c.update(&views[n], &gos_from_stage))
            .collect::<Vec<_>>();

        let znear = 0.05;
        let view_proj =
            [0, 1].map(|i| Frustum::from(views[i].fov).projection(znear) * view_matrices[i]);
        let camera_rotations = [0, 1].map(|i| {
            engine.render_context.cameras[i]
                .gos_from_view
                .to_scale_rotation_translation()
                .1
        });

        let camera_right = [0, 1].map(|i| camera_rotations[i] * glam::Vec3::X);
        let camera_up = [0, 1].map(|i| camera_rotations[i] * glam::Vec3::Y);

        self.view_proj = view_proj;
        self.camera_right = camera_right;
        self.camera_up = camera_up;

        let resolution = engine.xr_context.swapchain_resolution;
        self.viewport_size = glam::Vec2::new(resolution.width as f32, resolution.height as f32);
    }
}
