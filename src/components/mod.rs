use hotham::glam;
use rolt::BodyId;

#[derive(Debug, Clone)]
pub struct DynamicPhysicsBody {
    pub shape_kind: ShapeKind,
}
impl DynamicPhysicsBody {
    pub fn new_sphere(radius: f32) -> DynamicPhysicsBody {
        DynamicPhysicsBody {
            shape_kind: ShapeKind::Sphere { radius },
        }
    }

    pub fn y_offset(&self) -> f32 {
        match self.shape_kind {
            ShapeKind::Cube { half_extents } => half_extents,
            ShapeKind::Box { half_y, .. } => half_y,
            ShapeKind::Sphere { radius } => radius,
            ShapeKind::Capsule { half_height, .. } => half_height,
        }
    }
}

#[derive(Debug, Clone)]
pub struct KinematicPhysicsBody {
    pub shape_kind: ShapeKind,
}

impl KinematicPhysicsBody {
    pub fn new_box(half_x: f32, half_y: f32, half_z: f32) -> Self {
        Self {
            shape_kind: ShapeKind::Box {
                half_x,
                half_y,
                half_z,
            },
        }
    }

    pub fn y_offset(&self) -> f32 {
        match self.shape_kind {
            ShapeKind::Cube { half_extents } => half_extents,
            ShapeKind::Box { half_y, .. } => half_y,
            ShapeKind::Sphere { radius } => radius,
            ShapeKind::Capsule { half_height, .. } => half_height,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ShapeKind {
    Cube {
        half_extents: f32,
    },
    Box {
        half_x: f32,
        half_y: f32,
        half_z: f32,
    },
    Sphere {
        radius: f32,
    },
    Capsule {
        half_height: f32,
        radius: f32,
    },
}

#[derive(Debug, Clone)]
pub struct InsertedPhysicsBody {
    pub body_id: BodyId,
}

#[derive(Clone, Copy, Debug)]
pub struct LocalAABB {
    pub min: glam::Vec3,
    pub max: glam::Vec3,
}

impl Default for LocalAABB {
    fn default() -> Self {
        Self {
            min: glam::Vec3::splat(f32::INFINITY),
            max: glam::Vec3::splat(f32::NEG_INFINITY),
        }
    }
}

impl LocalAABB {
    pub fn center(&self) -> glam::Vec3 {
        (self.min + self.max) * 0.5
    }

    pub fn half_extents(&self) -> glam::Vec3 {
        (self.max - self.min) * 0.5
    }

    #[allow(unused)]
    pub fn expand_to_include_point(&mut self, point_in_local_space: glam::Vec3) {
        self.min = self.min.min(point_in_local_space);
        self.max = self.max.max(point_in_local_space);
    }
}
