mod components;
mod graphics;
mod physics;
mod simulation;
mod systems;
mod terrain;
mod viking;

use std::collections::HashMap;

use crate::{
    components::{unit::Unit, Jetpack, KinematicPhysicsBody, Weapon, WeaponKind},
    graphics::renderer::Renderer,
    physics::Physics,
    simulation::Simulation,
    viking::spawn_vikings,
};
use hotham::{
    asset_importer,
    components::{hand::Handedness, LocalTransform},
    glam,
    hecs::{self, World},
    systems::{rendering, update_global_transform_system},
    xr, Engine, HothamResult, TickData,
};
use log::info;

pub const DELTA_TIME: f32 = 1. / 72.;
pub const UNIT_RADIUS: f32 = 0.5;
pub const UNIT_COUNT: usize = 20;
pub const UNIT_HEIGHT: f32 = 1.5;
pub const UNIT_HALF_HEIGHT: f32 = UNIT_HEIGHT / 2.0;
pub const UNIT_MAX_HEALTH: f32 = 80.;
pub const SEPARATION_STRENGTH: f32 = 5.0;
pub const UNIT_INITIAL_TARGET_RADIUS: f32 = 20.;
pub const SPAWN_RADIUS: f32 = 10.;

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
pub fn main() {
    println!("[FIREFLIES] MAIN!");
    real_main().expect("Error running app!");
    println!("[FIREFLIES] FINISHED! Goodbye!");
}

pub fn real_main() -> HothamResult<()> {
    let _ = env_logger::builder()
        .filter_module("murdertime_5000", log::LevelFilter::Trace)
        .try_init();

    info!("Initialising fireflies..");
    info!("Building engine..");
    let mut engine = Engine::new();
    info!("..done!");

    info!("Building physics..");
    let mut physics = Physics::new();
    info!("..done!");

    let mut renderer = Renderer::new(&mut engine);
    info!("Initialising app..");
    init(&mut engine)?;
    info!("Done! Entering main loop..");

    let mut simulation = Simulation::default();

    while let Ok(tick_data) = engine.update() {
        tick(
            tick_data,
            &mut engine,
            &mut physics,
            &mut renderer,
            &mut simulation,
        );
        engine.finish()?;
    }

    Ok(())
}

fn tick(
    tick_data: TickData,
    engine: &mut Engine,
    physics: &mut Physics,
    renderer: &mut Renderer,
    simulation: &mut Simulation,
) {
    // Gameplay loop
    renderer.allocator.transfers_complete();
    let mut debug_lines = Vec::new();

    if tick_data.current_state == xr::SessionState::FOCUSED {
        let mut command_buffer = hecs::CommandBuffer::new();
        // Custom physics
        systems::physics::physics_system(engine, physics, &mut command_buffer, &mut debug_lines);

        systems::jetpack_system::jetpack_system(engine);

        // Weapons
        systems::weapon_movement::weapon_movement_system(engine, simulation);
        systems::weapon_firing::weapon_firing_system(
            engine,
            simulation,
            &mut command_buffer,
            physics,
        );
        systems::hammer_hit::hammer_hit_system(engine, physics);

        // Projectiles
        systems::update_projectile::update_projectile_system(
            engine,
            simulation,
            physics,
            &mut command_buffer,
            &mut debug_lines,
        );

        // Units
        systems::unit_state::unit_state_system(engine, simulation);
        systems::unit_movement::unit_movement_system(engine, simulation);

        // Custom animations
        systems::unit_animation::unit_animation_system(engine, simulation);

        // Bzzzt
        hotham::systems::haptics_system(engine);
        hotham::systems::update_global_transform_system(engine);
        hotham::systems::skinning_system(engine);

        command_buffer.run_on(&mut engine.world);
    }

    let views = engine.xr_context.update_views().to_owned();
    simulation.update(engine, &views);

    // Rendering loop
    unsafe {
        let global_from_stage = hotham::components::stage::get_global_from_stage(&engine.world);
        renderer.update_lines(debug_lines, global_from_stage);
        renderer.execute_transfers(engine);
        rendering::begin(
            &mut engine.world,
            &mut engine.vulkan_context,
            &mut engine.render_context,
            &views,
            tick_data.swapchain_image_index,
        );

        // PBR Rendering
        rendering::draw_world(&mut engine.vulkan_context, &mut engine.render_context);

        // Debug lines
        renderer.render(engine, simulation);

        rendering::end(&mut engine.vulkan_context, &mut engine.render_context);
    }
}

fn init(engine: &mut Engine) -> Result<HashMap<String, hecs::World>, hotham::HothamError> {
    let render_context = &mut engine.render_context;
    let vulkan_context = &mut engine.vulkan_context;
    let world = &mut engine.world;

    let glb_buffers: Vec<&[u8]> = vec![
        include_bytes!("../assets/floor.glb"),
        include_bytes!("../assets/1_Viking_Male_1.glb"),
        include_bytes!("../assets/gatling_gun.glb"),
        include_bytes!("../assets/hammer.glb"),
        // include_bytes!("../assets/2_Viking_Male_2.glb"),
        // include_bytes!("../assets/3_Viking_Male_3.glb"),
        // include_bytes!("../assets/4_Viking_Male_4.glb"),
        // include_bytes!("../assets/5_Viking_Male_5.glb"),
        // include_bytes!("../assets/6_Viking_Female_1.glb"),
        // include_bytes!("../assets/7_Viking_Female_2.glb"),
        // include_bytes!("../assets/8_Viking_Female_3.glb"),
        // include_bytes!("../assets/9_Viking_Female_4.glb"),
        // include_bytes!("../assets/10_Viking_Female_5.glb"),
    ];
    let models =
        asset_importer::load_models_from_glb(&glb_buffers, vulkan_context, render_context)?;

    add_floor(&models, world);
    spawn_vikings(engine, &models);
    // add_guns(engine, &models);
    add_hammers(engine, &models);

    // Update global transforms from local transforms before physics_system gets confused
    update_global_transform_system(engine);

    engine
        .world
        .insert_one(engine.stage_entity, Jetpack::default())
        .unwrap();

    Ok(models)
}

fn add_guns(engine: &mut Engine, models: &HashMap<String, World>) {
    let world = &mut engine.world;
    let left_gun = asset_importer::add_model_to_world("SM_Wep_GattlingGun_01", models, world, None)
        .expect("Could not find gatling gun");

    world
        .insert_one(
            left_gun,
            Weapon {
                hand: Handedness::Left,
                kind: WeaponKind::GatlingGun { cooldown: 0. },
            },
        )
        .unwrap();

    let right_gun =
        asset_importer::add_model_to_world("SM_Wep_GattlingGun_01", models, world, None)
            .expect("Could not find gatling gun");

    world
        .insert_one(
            right_gun,
            Weapon {
                hand: Handedness::Right,
                kind: WeaponKind::GatlingGun { cooldown: 0. },
            },
        )
        .unwrap();
}

fn add_hammers(engine: &mut Engine, models: &HashMap<String, World>) {
    for hand in [Handedness::Left, Handedness::Right] {
        let world = &mut engine.world;
        let hammer = asset_importer::add_model_to_world("SM_Wep_Hammer_02", models, world, None)
            .expect("Could not find hammer");

        // Create a physics body component
        let mut capsule = KinematicPhysicsBody::new_capsule(0.2, 0.1);
        capsule.y_offset = glam::Vec3::ZERO;
        let hit_entity = world.spawn((LocalTransform::default(), capsule));

        world
            .insert_one(
                hammer,
                Weapon {
                    hand,
                    kind: WeaponKind::Hammer { hit_entity },
                },
            )
            .unwrap();
    }
}

fn add_floor(models: &std::collections::HashMap<String, World>, world: &mut World) {
    let entity = asset_importer::add_model_to_world("Floor", models, world, None)
        .expect("Could not find Floor");
    // let collider = Collider::new(SharedShape::halfspace(na::Vector3::y_axis()));
    // let rigid_body = RigidBody {
    //     body_type: BodyType::Fixed,
    //     ..Default::default()
    // };
    //
    {
        let mut transform = world.get::<&mut LocalTransform>(entity).unwrap();
        transform.scale = glam::Vec3::new(2.0, 1.0, 2.0);
    }

    world
        .insert_one(entity, KinematicPhysicsBody::new_box(10.0, 0.5, 10.0))
        .unwrap();
}

pub struct DamageEvent {
    pub target: hecs::Entity,
    pub amount: f32,
}

impl DamageEvent {
    pub fn apply(self, world: &hecs::World) -> bool {
        let Ok(mut unit) = world.get::<&mut Unit>(self.target) else {
            return false;
        };

        if unit.health.is_dead() {
            return false;
        }

        unit.health.take_damage(self.amount);
        println!(
            "Unit {} took {} damage (now {})",
            unit.id, self.amount, unit.health.current
        );

        true
    }
}
