pub mod debug;

use hotham::{glam, hecs};
use joltc_sys::*;
use rolt::{
    BodyId, BroadPhaseLayer, BroadPhaseLayerInterface, ObjectLayer, ObjectLayerPairFilter,
    ObjectVsBroadPhaseLayerFilter, PhysicsSystem, Vec3,
};
use std::ffi::{CStr, CString};
use std::ptr;

use crate::components::{self, InsertedPhysicsBody, ShapeKind};

// use crate::components::{self, InsertedPhysicsBody, ShapeKind};
// use crate::{QUAD_SIZE, VERTEX_GRID_SIZE};

pub const OL_NON_MOVING: JPC_ObjectLayer = 0;
pub const OL_MOVING: JPC_ObjectLayer = 1;

pub const BPL_NON_MOVING: JPC_BroadPhaseLayer = 0;
pub const BPL_MOVING: JPC_BroadPhaseLayer = 1;
pub const BPL_COUNT: JPC_BroadPhaseLayer = 2;

pub struct Physics {
    pub system: PhysicsSystem,
    pub temp_allocator: *mut joltc_sys::JPC_TempAllocatorImpl,
    pub job_system: *mut joltc_sys::JPC_JobSystemThreadPool,
    pub terrain_body_id: Option<BodyId>,
}

impl Physics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&self, dt: f32) {
        unsafe {
            self.system
                .update(dt, 1, self.temp_allocator, self.job_system)
        };
    }

    // pub(crate) fn create_terrain(&mut self, terrain: &super::terrain::Terrain) {
    //     let body_interface = self.system.body_interface();

    //     let mut heightfield_settings: JPC_HeightFieldShapeSettings = unsafe { std::mem::zeroed() };
    //     unsafe {
    //         JPC_HeightFieldShapeSettings_default(&mut heightfield_settings);
    //     }
    //     heightfield_settings.SampleCount = VERTEX_GRID_SIZE as _;
    //     heightfield_settings.HeightSamples = terrain.heights.as_slice().as_ptr();

    //     // If you have N samples along an axis, you have (N - 1) intervals between them.
    //     // The total width in world units is (N - 1) * cell_size.
    //     // Half of that is how far we must shift left/back to center the grid at origin.
    //     let interval_count_per_axis = heightfield_settings.SampleCount.saturating_sub(1) as f32;
    //     let half_extent_in_world_units = (interval_count_per_axis * QUAD_SIZE) * 0.5;

    //     // Offset moves the (0,0) sample corner to (-half_extent, -half_extent) in XZ,
    //     // so the grid center ends up at (0,0,0).
    //     let heightfield_offset_in_world_space = Vec3::new(
    //         -half_extent_in_world_units, // x
    //         0.0,                         // y
    //         -half_extent_in_world_units, // z
    //     );

    //     heightfield_settings.Offset = heightfield_offset_in_world_space.to_jpc();

    //     // Scale converts "one sample step" into meters in world space.
    //     // Y scale is 1.0 if your height samples are already meters.
    //     let heightfield_scale_in_world_space = Vec3::new(
    //         QUAD_SIZE, // x
    //         1.0,       // y
    //         QUAD_SIZE, // z
    //     );

    //     heightfield_settings.Scale = heightfield_scale_in_world_space.to_jpc();

    //     let mut heightfield_shape: *mut JPC_Shape = ptr::null_mut();
    //     let mut err: *mut JPC_String = ptr::null_mut();

    //     if unsafe {
    //         JPC_HeightFieldShapeSettings_Create(
    //             &heightfield_settings,
    //             &mut heightfield_shape,
    //             &mut err,
    //         )
    //     } {
    //     } else {
    //         let error = unsafe { CStr::from_ptr(JPC_String_c_str(err)).to_owned() };
    //         eprintln!("Error creating heightfield {error:?}");
    //     }

    //     // From this point onwards, we can guarantee that we've regenrated the heightfield
    //     terrain.set_needs_heightmap_regeneration(false);

    //     // If we already have a body for the terrain, update its shape
    //     if let Some(body_id) = self.terrain_body_id {
    //         unsafe {
    //             body_interface.set_shape(
    //                 body_id,
    //                 heightfield_shape,
    //                 false,
    //                 JPC_ACTIVATION_DONT_ACTIVATE,
    //             )
    //         };
    //         return;
    //     }

    //     let terrain_body = unsafe {
    //         body_interface
    //             .create_body(&JPC_BodyCreationSettings {
    //                 Position: Vec3::new(0.0, 0.0, 0.0).to_jpc(),
    //                 MotionType: JPC_MOTION_TYPE_STATIC,
    //                 ObjectLayer: OL_NON_MOVING,
    //                 Shape: heightfield_shape,
    //                 ..Default::default()
    //             })
    //             .unwrap()
    //     };

    //     body_interface.add_body(terrain_body.id(), JPC_ACTIVATION_DONT_ACTIVATE);
    //     self.terrain_body_id = Some(terrain_body.id());
    // }

    pub fn create_kinematic_body(
        &self,
        initial_position: Vec3,
        body: &components::KinematicPhysicsBody,
    ) -> InsertedPhysicsBody {
        let body_interface = &self.system.body_interface();
        let body_id = match body.shape_kind {
            ShapeKind::Cube { half_extents } => build_box(
                body_interface,
                initial_position,
                glam::Vec3::splat(half_extents),
                JPC_MOTION_TYPE_KINEMATIC,
            ),
            ShapeKind::Box {
                half_x,
                half_y,
                half_z,
            } => build_box(
                body_interface,
                initial_position,
                Vec3::new(half_x, half_y, half_z),
                JPC_MOTION_TYPE_KINEMATIC,
            ),
            ShapeKind::Sphere { radius } => build_sphere(
                body_interface,
                initial_position,
                radius,
                JPC_MOTION_TYPE_KINEMATIC,
            ),
            ShapeKind::Capsule {
                half_height,
                radius,
            } => build_capsule(
                body_interface,
                initial_position,
                radius,
                half_height,
                JPC_MOTION_TYPE_KINEMATIC,
            ),
        };

        InsertedPhysicsBody { body_id }
    }

    pub fn create_dynamic_body(
        &self,
        initial_position: Vec3,
        body: &components::DynamicPhysicsBody,
    ) -> InsertedPhysicsBody {
        let body_interface = &self.system.body_interface();
        let body_id = match body.shape_kind {
            ShapeKind::Cube { half_extents } => build_box(
                body_interface,
                initial_position,
                glam::Vec3::splat(half_extents),
                JPC_MOTION_TYPE_DYNAMIC,
            ),
            ShapeKind::Box {
                half_x,
                half_y,
                half_z,
            } => build_box(
                body_interface,
                initial_position,
                Vec3::new(half_x, half_y, half_z),
                JPC_MOTION_TYPE_DYNAMIC,
            ),
            ShapeKind::Sphere { radius } => build_sphere(
                body_interface,
                initial_position,
                radius,
                JPC_MOTION_TYPE_DYNAMIC,
            ),
            ShapeKind::Capsule {
                half_height,
                radius,
            } => build_capsule(
                body_interface,
                initial_position,
                radius,
                half_height,
                JPC_MOTION_TYPE_KINEMATIC,
            ),
        };

        InsertedPhysicsBody { body_id }
    }

    pub fn raycast(&self, origin: Vec3, direction: Vec3) -> Option<RayHit> {
        let narrow_phase = self.system.narrow_phase_query();
        let result = narrow_phase.cast_ray(rolt::RayCastArgs {
            ray: rolt::RRayCast { origin, direction },
            broad_phase_layer_filter: None,
            object_layer_filter: None,
            body_filter: None,
            shape_filter: None,
        })?;

        let user_data = self.system.body_interface().user_data(result.body_id);
        let entity = hecs::Entity::from_bits(user_data).unwrap();

        Some(RayHit { entity })
    }
}

impl Default for Physics {
    fn default() -> Self {
        rolt::register_default_allocator();
        rolt::factory_init();
        rolt::register_types();

        let temp_allocator = unsafe { joltc_sys::JPC_TempAllocatorImpl_new(10 * 1024 * 1024) };
        let job_system = unsafe {
            joltc_sys::JPC_JobSystemThreadPool_new2(
                joltc_sys::JPC_MAX_PHYSICS_JOBS as _,
                joltc_sys::JPC_MAX_PHYSICS_BARRIERS as _,
            )
        };

        Self {
            system: init_physics_system(),
            temp_allocator,
            job_system,
            terrain_body_id: None,
        }
    }
}

pub fn init_physics_system() -> PhysicsSystem {
    let mut physics_system = PhysicsSystem::new();
    let broad_phase_layer_interface = BroadPhaseLayers;
    let object_vs_broad_phase_layer_filter = ObjectVsBroadPhase;
    let object_layer_pair_filter = ObjectLayerPair;

    let max_bodies = 1024;
    let num_body_mutexes = 64;
    let max_body_pairs = 1024;
    let max_contact_constraints = 1024;

    physics_system.init(
        max_bodies,
        num_body_mutexes,
        max_body_pairs,
        max_contact_constraints,
        broad_phase_layer_interface,
        object_vs_broad_phase_layer_filter,
        object_layer_pair_filter,
    );
    physics_system
}

pub fn build_sphere(
    body_interface: &rolt::BodyInterface<'_>,
    initial_position: impl Into<glam::Vec3>,
    radius: f32,
    motion_type: JPC_MotionType,
) -> BodyId {
    let initial_position = initial_position.into();
    let sphere_shape = create_sphere(&JPC_SphereShapeSettings {
        Radius: radius,
        ..Default::default()
    })
    .unwrap();

    let object_layer = match motion_type {
        JPC_MOTION_TYPE_STATIC => OL_NON_MOVING,
        _ => OL_MOVING,
    };

    let sphere = unsafe {
        body_interface
            .create_body(&JPC_BodyCreationSettings {
                Position: initial_position.to_jpc(),
                MotionType: JPC_MOTION_TYPE_DYNAMIC,
                ObjectLayer: object_layer,
                Shape: sphere_shape,
                Restitution: 0.8,
                ..Default::default()
            })
            .unwrap()
    };

    let sphere_id = sphere.id();
    body_interface.add_body(sphere_id, JPC_ACTIVATION_ACTIVATE);

    sphere_id
}

pub fn build_capsule(
    body_interface: &rolt::BodyInterface<'_>,
    initial_position: impl Into<glam::Vec3>,
    radius: f32,
    half_height_of_cylinder: f32,
    motion_type: JPC_MotionType,
) -> BodyId {
    let initial_position = initial_position.into();
    let capsule_shape = create_capsule(&JPC_CapsuleShapeSettings {
        Radius: radius,
        HalfHeightOfCylinder: half_height_of_cylinder,
        ..Default::default()
    })
    .unwrap();

    let object_layer = match motion_type {
        JPC_MOTION_TYPE_STATIC => OL_NON_MOVING,
        _ => OL_MOVING,
    };

    // Offset the position by half the height of the capsule
    let initial_position = initial_position + glam::Vec3::Y * half_height_of_cylinder;

    let sphere = unsafe {
        body_interface
            .create_body(&JPC_BodyCreationSettings {
                Position: initial_position.to_jpc(),
                MotionType: motion_type,
                ObjectLayer: object_layer,
                Shape: capsule_shape,
                Restitution: 0.8,
                ..Default::default()
            })
            .unwrap()
    };

    let body_id = sphere.id();
    body_interface.add_body(body_id, JPC_ACTIVATION_ACTIVATE);

    body_id
}

pub fn build_box(
    body_interface: &rolt::BodyInterface<'_>,
    initial_position: impl Into<glam::Vec3>,
    half_extents: impl Into<glam::Vec3>,
    motion_type: JPC_MotionType,
) -> BodyId {
    let initial_position = initial_position.into();
    let box_shape = create_box(&JPC_BoxShapeSettings {
        HalfExtent: half_extents.into().to_jpc(),
        ..Default::default()
    })
    .unwrap();

    let object_layer = match motion_type {
        JPC_MOTION_TYPE_STATIC => OL_NON_MOVING,
        _ => OL_MOVING,
    };

    let dat_box = unsafe {
        body_interface
            .create_body(&JPC_BodyCreationSettings {
                Position: initial_position.to_jpc(),
                MotionType: motion_type,
                ObjectLayer: object_layer,
                Shape: box_shape,
                ..Default::default()
            })
            .unwrap()
    };

    let box_id = dat_box.id();
    body_interface.add_body(box_id, JPC_ACTIVATION_ACTIVATE);

    box_id
}

struct BroadPhaseLayers;

impl BroadPhaseLayerInterface for BroadPhaseLayers {
    fn get_num_broad_phase_layers(&self) -> u32 {
        BPL_COUNT as u32
    }

    fn get_broad_phase_layer(&self, layer: ObjectLayer) -> BroadPhaseLayer {
        match layer.raw() {
            OL_NON_MOVING => BroadPhaseLayer::new(BPL_NON_MOVING),
            OL_MOVING => BroadPhaseLayer::new(BPL_MOVING),
            _ => unreachable!(),
        }
    }
}

struct ObjectVsBroadPhase;

impl ObjectVsBroadPhaseLayerFilter for ObjectVsBroadPhase {
    fn should_collide(&self, layer1: ObjectLayer, layer2: BroadPhaseLayer) -> bool {
        match layer1.raw() {
            OL_NON_MOVING => layer2.raw() == BPL_MOVING,
            OL_MOVING => true,
            _ => unreachable!(),
        }
    }
}

struct ObjectLayerPair;

impl ObjectLayerPairFilter for ObjectLayerPair {
    fn should_collide(&self, layer1: ObjectLayer, layer2: ObjectLayer) -> bool {
        match layer1.raw() {
            OL_NON_MOVING => layer2.raw() == OL_MOVING,
            OL_MOVING => true,
            _ => unreachable!(),
        }
    }
}

#[allow(unused)]
pub fn create_box(settings: &JPC_BoxShapeSettings) -> Result<*mut JPC_Shape, CString> {
    let mut shape: *mut JPC_Shape = ptr::null_mut();
    let mut err: *mut JPC_String = ptr::null_mut();

    unsafe {
        if JPC_BoxShapeSettings_Create(settings, &mut shape, &mut err) {
            Ok(shape)
        } else {
            Err(CStr::from_ptr(JPC_String_c_str(err)).to_owned())
        }
    }
}

#[allow(unused)]
pub fn create_sphere(settings: &JPC_SphereShapeSettings) -> Result<*mut JPC_Shape, CString> {
    let mut shape: *mut JPC_Shape = ptr::null_mut();
    let mut err: *mut JPC_String = ptr::null_mut();

    unsafe {
        if JPC_SphereShapeSettings_Create(settings, &mut shape, &mut err) {
            Ok(shape)
        } else {
            Err(CStr::from_ptr(JPC_String_c_str(err)).to_owned())
        }
    }
}

pub fn create_capsule(settings: &JPC_CapsuleShapeSettings) -> Result<*mut JPC_Shape, CString> {
    let mut shape: *mut JPC_Shape = ptr::null_mut();
    let mut err: *mut JPC_String = ptr::null_mut();

    unsafe {
        if JPC_CapsuleShapeSettings_Create(settings, &mut shape, &mut err) {
            Ok(shape)
        } else {
            Err(CStr::from_ptr(JPC_String_c_str(err)).to_owned())
        }
    }
}

pub trait ToJPC {
    type Output;

    fn to_jpc(self) -> Self::Output;
}

impl ToJPC for glam::Vec4 {
    type Output = JPC_Vec4;

    fn to_jpc(self) -> Self::Output {
        JPC_Vec4 {
            x: self.x,
            y: self.y,
            z: self.z,
            w: self.w,
        }
    }
}

impl ToJPC for glam::Vec3A {
    type Output = JPC_Vec3;

    fn to_jpc(self) -> Self::Output {
        vec3(self.x, self.y, self.z)
    }
}

impl ToJPC for glam::Vec3 {
    type Output = JPC_Vec3;

    fn to_jpc(self) -> Self::Output {
        vec3(self.x, self.y, self.z)
    }
}

impl ToJPC for glam::Quat {
    type Output = JPC_Quat;

    fn to_jpc(self) -> Self::Output {
        JPC_Quat {
            x: self.x,
            y: self.y,
            z: self.z,
            w: self.w,
        }
    }
}

fn vec3(x: f32, y: f32, z: f32) -> JPC_Vec3 {
    JPC_Vec3 { x, y, z, _w: z }
}

pub struct RayHit {
    pub entity: hecs::Entity,
}
