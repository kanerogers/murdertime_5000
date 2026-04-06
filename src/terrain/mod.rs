use glam::{UVec2, Vec2, Vec3};
use hotham::glam::{self, Vec3Swizzles};

pub const SIDE_METRES: f32 = 64.0;
pub const HALF_SIDE_METRES: f32 = SIDE_METRES * 0.5;

pub const CELLS_PER_SIDE: u32 = SIDE_METRES as u32 * 2;
pub const POINTS_PER_SIDE: u32 = CELLS_PER_SIDE + 1;
pub const CELL_SIZE_METRES: f32 = SIDE_METRES / CELLS_PER_SIDE as f32;

pub const BASIN_SIZE_METRES: f32 = 5.0;
pub const BASIN_HALF_EXTENT_METRES: f32 = BASIN_SIZE_METRES * 0.5;

pub const RIM_HEIGHT_METRES: f32 = 14.0;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TerrainPushConstants {
    pub world_to_clip: glam::Mat4,
}

impl TerrainPushConstants {
    pub fn new(world_to_clip: glam::Mat4) -> Self {
        Self { world_to_clip }
    }
}

unsafe impl bytemuck::Pod for TerrainPushConstants {}
unsafe impl bytemuck::Zeroable for TerrainPushConstants {}

#[derive(Debug, Clone)]
pub struct Terrain {
    heights: Vec<f32>,
}

impl Default for Terrain {
    fn default() -> Self {
        Self::generate_bowl()
    }
}

impl Terrain {
    pub fn generate_bowl() -> Self {
        let mut heights = vec![0.0; (POINTS_PER_SIDE * POINTS_PER_SIDE) as usize];

        for z in 0..POINTS_PER_SIDE {
            for x in 0..POINTS_PER_SIDE {
                let p = UVec2::new(x, z);
                let world_xz = Self::grid_to_world_xz(p);
                heights[Self::flatten(p)] = bowl_height(world_xz);
            }
        }

        Self { heights }
    }

    pub fn heights(&self) -> &[f32] {
        &self.heights
    }

    pub fn vertex_count(&self) -> u32 {
        CELLS_PER_SIDE * CELLS_PER_SIDE * 6
    }

    pub fn push_constants(&self, world_to_clip: glam::Mat4) -> TerrainPushConstants {
        TerrainPushConstants::new(world_to_clip)
    }

    pub fn get_height_at(&self, position_in_world: Vec3) -> Option<f32> {
        self.sample_height_at_world_xz(position_in_world.xz())
    }

    pub fn flatten(p: UVec2) -> usize {
        (p.y * POINTS_PER_SIDE + p.x) as usize
    }

    pub fn grid_to_world_xz(p: UVec2) -> Vec2 {
        Vec2::new(
            p.x as f32 * CELL_SIZE_METRES - HALF_SIDE_METRES,
            p.y as f32 * CELL_SIZE_METRES - HALF_SIDE_METRES,
        )
    }

    pub fn contains_world_xz(world_xz: Vec2) -> bool {
        world_xz.x >= -HALF_SIDE_METRES
            && world_xz.x <= HALF_SIDE_METRES
            && world_xz.y >= -HALF_SIDE_METRES
            && world_xz.y <= HALF_SIDE_METRES
    }

    fn world_xz_to_grid_continuous(world_xz: Vec2) -> Option<Vec2> {
        if !Self::contains_world_xz(world_xz) {
            return None;
        }

        let u = (world_xz.x + HALF_SIDE_METRES) / SIDE_METRES;
        let v = (world_xz.y + HALF_SIDE_METRES) / SIDE_METRES;

        Some(Vec2::new(
            u * CELLS_PER_SIDE as f32,
            v * CELLS_PER_SIDE as f32,
        ))
    }

    fn sample_height_at_world_xz(&self, world_xz: Vec2) -> Option<f32> {
        let grid = Self::world_xz_to_grid_continuous(world_xz)?;

        let x0 = grid.x.floor() as u32;
        let z0 = grid.y.floor() as u32;

        let x1 = (x0 + 1).min(POINTS_PER_SIDE - 1);
        let z1 = (z0 + 1).min(POINTS_PER_SIDE - 1);

        let tx = grid.x - x0 as f32;
        let tz = grid.y - z0 as f32;

        let h00 = self.heights[Self::flatten(UVec2::new(x0, z0))];
        let h10 = self.heights[Self::flatten(UVec2::new(x1, z0))];
        let h01 = self.heights[Self::flatten(UVec2::new(x0, z1))];
        let h11 = self.heights[Self::flatten(UVec2::new(x1, z1))];

        let hx0 = lerp(h00, h10, tx);
        let hx1 = lerp(h01, h11, tx);

        Some(lerp(hx0, hx1, tz))
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn smoothstep01(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn bowl_height(world_xz: Vec2) -> f32 {
    // Flat 5x5m square basin in the middle.
    let r = world_xz.length();

    if r <= BASIN_HALF_EXTENT_METRES {
        return 0.0;
    }

    let t = (r - BASIN_HALF_EXTENT_METRES) / (HALF_SIDE_METRES - BASIN_HALF_EXTENT_METRES);

    let s = smoothstep01(t);

    // Gentle in the middle, steeper near the rim.
    RIM_HEIGHT_METRES * s * s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(a: f32, b: f32) {
        let diff = (a - b).abs();
        assert!(diff < 1e-4, "expected {a} ~= {b}, diff={diff}");
    }

    #[test]
    fn grid_corners_are_correct() {
        let min = Terrain::grid_to_world_xz(UVec2::new(0, 0));
        let max = Terrain::grid_to_world_xz(UVec2::new(POINTS_PER_SIDE - 1, POINTS_PER_SIDE - 1));

        assert_close(min.x, -HALF_SIDE_METRES);
        assert_close(min.y, -HALF_SIDE_METRES);
        assert_close(max.x, HALF_SIDE_METRES);
        assert_close(max.y, HALF_SIDE_METRES);
    }

    #[test]
    fn center_is_flat() {
        let terrain = Terrain::generate_bowl();
        assert_close(terrain.get_height_at(Vec3::ZERO).unwrap(), 0.0);
    }

    #[test]
    fn basin_is_flat() {
        let terrain = Terrain::generate_bowl();
        assert_close(
            terrain.get_height_at(Vec3::new(2.0, 999.0, -2.0)).unwrap(),
            0.0,
        );
    }

    #[test]
    fn outside_returns_none() {
        let terrain = Terrain::generate_bowl();
        assert!(terrain.get_height_at(Vec3::new(100.0, 0.0, 0.0)).is_none());
        assert!(terrain.get_height_at(Vec3::new(0.0, 0.0, -100.0)).is_none());
    }

    #[test]
    fn rim_is_higher_than_center() {
        let terrain = Terrain::generate_bowl();

        let center = terrain.get_height_at(Vec3::new(0.0, 0.0, 0.0)).unwrap();
        let rim = terrain.get_height_at(Vec3::new(31.0, 0.0, 0.0)).unwrap();

        assert!(rim > center);
        assert!(rim > 10.0);
    }
}
