use hotham::{glam::Vec2, hecs};

const STOP_DISTANCE: f32 = 0.1;
const MOVEMENT_SPEED: f32 = 3.0;
const ARRIVAL_SLOWDOWN_RADIUS: f32 = 2.0;

#[derive(Debug, Clone)]
pub struct Unit {
    pub id: u32,
    pub position: Vec2,
    pub rotation: f32,
    pub target_position: Vec2,
    pub combat_status: CombatStatus,
    pub health: Health,
}

impl Unit {
    pub fn move_towards_target(&mut self, dt: f32, target_position: Vec2) {
        // Get a vector between us and our destination
        let to_destination = target_position - self.position;

        // Check the distance; if we're close, nothing to do
        let distance_to_destination = to_destination.length();
        if distance_to_destination <= STOP_DISTANCE {
            return;
        }

        // Normalise the direction
        let direction_towards_destination = to_destination.normalize();

        // Face towards the direction
        self.rotation = direction_towards_destination
            .x
            .atan2(direction_towards_destination.y);

        // Scale our speed by how close we are to the destination
        let speed_scale = (distance_to_destination / ARRIVAL_SLOWDOWN_RADIUS).clamp(0., 1.);
        let scaled_speed = MOVEMENT_SPEED * speed_scale;

        // Figure out how far we can move
        let maximum_step_distance_this_frame = scaled_speed * dt;
        let step_distance_this_frame =
            maximum_step_distance_this_frame.min(distance_to_destination);

        // Apply the displacement
        self.position += direction_towards_destination * step_distance_this_frame;
    }
}

#[derive(Debug, Clone, Default)]
pub enum CombatStatus {
    Idle,
    SearchingForTarget {
        radius: f32,
    },
    Attacking {
        target_entity: hecs::Entity,
        cooldown_left: f32,
    },
    // FSM nonsense
    #[default]
    Moving,
}

#[derive(Default, Debug, Clone)]
pub struct Health {
    pub max: u32,
    pub current: u32,
}
