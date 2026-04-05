use hotham::glam::{Vec2, Vec3Swizzles};

use crate::{systems::unit_movement::HMD_DISTANCE, DELTA_TIME};

const MOVEMENT_SPEED: f32 = 3.0;
const ARRIVAL_SLOWDOWN_RADIUS: f32 = 2.0;
const ATTACK_COOLDOWN: f32 = 0.75;

#[derive(Debug, Clone)]
pub struct Unit {
    pub id: u32,
    pub position: Vec2,
    pub rotation: f32,
    pub status: UnitStatus,
    pub health: Health,
}

impl Unit {
    pub fn update_state(&mut self, simulation: &mut crate::Simulation) {
        let current_state = std::mem::take(&mut self.status);
        let hmd_position = simulation.head_pos.xz();
        self.status = match current_state {
            UnitStatus::Idle => UnitStatus::Moving,
            UnitStatus::Moving => {
                if self.move_towards_hmd(DELTA_TIME, hmd_position) {
                    UnitStatus::Attacking {
                        cooldown_left: ATTACK_COOLDOWN,
                    }
                } else {
                    UnitStatus::Moving
                }
            }
            UnitStatus::Attacking { cooldown_left } => {
                let cooldown_left = cooldown_left - DELTA_TIME;

                if !self.near_hmd(hmd_position) {
                    UnitStatus::Moving
                } else if cooldown_left < 0.0 {
                    // Do attack!
                    UnitStatus::Attacking {
                        cooldown_left: ATTACK_COOLDOWN,
                    }
                } else {
                    UnitStatus::Attacking { cooldown_left }
                }
            }
        };
    }

    pub fn near_hmd(&self, hmd_position: Vec2) -> bool {
        // Get a vector between us and our destination
        let to_destination = hmd_position - self.position;

        let distance_to_destination = to_destination.length();
        return distance_to_destination <= (HMD_DISTANCE + 1.0);
    }

    pub fn move_towards_hmd(&mut self, dt: f32, hmd_position: Vec2) -> bool {
        // Get a vector between us and our destination
        let to_destination = hmd_position - self.position;

        // Check the distance; if we're close, nothing to do
        let distance_to_destination = to_destination.length();
        if distance_to_destination <= (HMD_DISTANCE + 0.1) {
            return true;
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
        false
    }
}

#[derive(Debug, Clone, Default)]
pub enum UnitStatus {
    #[default]
    Idle,
    Attacking {
        cooldown_left: f32,
    },
    Moving,
}

#[derive(Default, Debug, Clone)]
pub struct Health {
    pub max: f32,
    pub current: f32,
}

impl Health {
    pub fn take_damage(&mut self, amount: f32) {
        self.current = (self.current - amount).max(0.);
    }

    pub fn is_dead(&self) -> bool {
        self.current >= 0.
    }
}
