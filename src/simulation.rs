use crate::velocity::*;
use bevy::{math::swizzles::*, prelude::*};
use bevy_prototype_debug_lines::DebugLines;
use rand::prelude::*;
use rand_pcg::Pcg32;

use crate::utils::*;

pub struct HeroesCowardSimulationPlugin;

impl Plugin for HeroesCowardSimulationPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            // state
            .add_system_set_to_stage(
                CoreStage::PostUpdate,
                State::<SimulationState>::get_driver(),
            )
            .add_state(SimulationState::NotInit)
            // resource
            .init_resource::<SimulationSpeed>()
            .init_resource::<SimulationSettings>()
            .init_resource::<SimulationDebug>()
            .init_resource::<SimStats>()
            // systems
            .add_startup_system(setup.system())
            .add_system_set(
                SystemSet::on_enter(SimulationState::Start)
                    .with_system(initialize_simulation.system()),
            )
            .add_system_set(
                SystemSet::on_update(SimulationState::Run).with_system(update_agents.system()),
            )
            .add_system_set_to_stage(
                CoreStage::PostUpdate,
                SystemSet::on_update(SimulationState::Run)
                    .with_system(update_velocities.system().label("update_velocity"))
                    .with_system(
                        keep_in_arena
                            .system()
                            .label("keep_in_arena")
                            .after("update_velocity"),
                    )
                    .with_system(compute_stats.system().after("keep_in_arena")),
            )
            .add_system_to_stage(
                CoreStage::PostUpdate,
                display_lines.system().after("keep_in_arena"),
            )
            .add_system_to_stage(
                CoreStage::PostUpdate,
                display_center_of_mass.system().after("keep_in_arena"),
            );
    }
}

// ===== states =====

/// Simulation statistiques
#[derive(Default)]
pub struct SimStats {
    pub center_of_mass: Vec2,
    pub deviation: f32,
}

/// State of the simulation
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum SimulationState {
    NotInit,
    Start,
    Run,
    Pause,
}

// ===== resources =====

#[derive(Debug, Clone, PartialEq)]
pub enum BlindBehavour {
    NoMove,
    RandomMove,
}

/// Settings for the simulation.
#[derive(Debug, Clone)]
pub struct SimulationSettings {
    pub seed: u64,
    pub agent_count: usize,
    pub heroe_proportion: f64,
    pub blind_behaviour: BlindBehavour,
    pub arena_size: f32,
    pub use_vision_limit: bool,
    pub vision_limit: f32,
}

impl Default for SimulationSettings {
    fn default() -> Self {
        Self {
            seed: 0,
            agent_count: 30,
            heroe_proportion: 0.5,
            blind_behaviour: BlindBehavour::NoMove,
            arena_size: 300.0,
            use_vision_limit: false,
            vision_limit: 30.0,
        }
    }
}

/// Define the speed of the agents
pub struct SimulationSpeed(pub f32);

/// Settings for debug display
#[derive(Default)]
pub struct SimulationDebug {
    pub display_friend_links: bool,
    pub display_foe_links: bool,
    pub center_of_mass: bool,
    pub deviation: bool,
}

impl Default for SimulationSpeed {
    fn default() -> Self {
        Self(32.0)
    }
}

/// Hold a list of agent entity.
struct Agents(pub Vec<Entity>);

/// Random number generator used by the simulation
struct SimRng(Pcg32);

// ===== components =====

/// Tag for the agents.
pub struct Agent;

/// Tag for the arena sprite.
struct Arena;

/// Component that define the agent behaviour.
enum AgentBehaviour {
    Heroe,
    Coward,
}

/// Component that hold witch entity is the friend and foe of the agent.
struct FriendFoe(Entity, Entity);

/// Resource to hold materials for the agents.
struct AgentMaterials {
    heroe_material: Handle<ColorMaterial>,
    coward_material: Handle<ColorMaterial>,
}

/// Bundle for agent.
/// [`FriendFoe`] isn't included.
#[derive(Bundle)]
struct AgentBundle {
    #[bundle]
    sprite: SpriteBundle,
    behaviour: AgentBehaviour,
    velocity: Velocity,
    agent: Agent,
}

impl AgentBundle {
    fn new(material: Handle<ColorMaterial>, x: f32, y: f32, behaviour: AgentBehaviour) -> Self {
        let mut transform = Transform::from_xyz(x, y, 0.0);
        transform.scale = Vec3::splat(1.0 / 8.0);

        Self {
            sprite: SpriteBundle {
                material,
                transform,
                ..Default::default()
            },
            behaviour,
            velocity: Velocity::default(),
            agent: Agent,
        }
    }
}

// ===== systems =====

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // load agent materials
    let heroe_material = materials.add(asset_server.load("heroe.png").into());
    let coward_material = materials.add(asset_server.load("coward.png").into());

    commands.insert_resource(AgentMaterials {
        heroe_material,
        coward_material,
    });

    // spawn the arena
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(asset_server.load("arena.png").into()),
            transform: Transform::from_scale(Vec3::splat(1.0)),
            ..Default::default()
        })
        .insert(Arena);
}

/// Initialize the simulation.
fn initialize_simulation(
    mut commands: Commands,
    mut simulation_state: ResMut<State<SimulationState>>,
    materials: Res<AgentMaterials>,
    simulation_settings: Res<SimulationSettings>,
    agents: Option<Res<Agents>>,
    mut arena: Query<&mut Transform, With<Arena>>,
) {
    println!("INIT SIM {:?}", *simulation_settings);

    let mut arena_transform = arena.single_mut().unwrap();
    arena_transform.scale = Vec3::splat(2.0 * simulation_settings.arena_size);

    // depsawn previous agents
    if let Some(agents) = agents {
        for e in &agents.0 {
            commands.entity(*e).despawn();
        }
    }

    let mut rng = Pcg32::seed_from_u64(simulation_settings.seed);
    let mut entities = Vec::with_capacity(simulation_settings.agent_count);

    // create agent entities
    for _ in 0..simulation_settings.agent_count {
        // Randomly define the agent behaviour
        let (behaviour, material) = if rng.gen_bool(simulation_settings.heroe_proportion) {
            (AgentBehaviour::Heroe, materials.heroe_material.clone())
        } else {
            (AgentBehaviour::Coward, materials.coward_material.clone())
        };
        // Get a random position in the arena
        let x = rng.gen_range(-simulation_settings.arena_size..simulation_settings.arena_size);
        let y = rng.gen_range(-simulation_settings.arena_size..simulation_settings.arena_size);
        entities.push(
            commands
                .spawn_bundle(AgentBundle::new(material, x, y, behaviour))
                .id(),
        );
    }

    // set a random friend and foe to each agents
    for entity in &entities {
        let mut friend: &Entity;
        let mut foe: &Entity;
        loop {
            friend = entities
                .get(rng.gen_range(0..simulation_settings.agent_count))
                .unwrap();
            if friend != entity {
                break;
            }
        }
        loop {
            foe = entities
                .get(rng.gen_range(0..simulation_settings.agent_count))
                .unwrap();
            if foe != entity && foe != friend {
                break;
            }
        }
        commands
            .entity(entity.clone())
            .insert(FriendFoe(friend.clone(), foe.clone()));
    }

    // keep trace of agent entities to despawn them
    // if the simulation restart.
    commands.insert_resource(Agents(entities));

    commands.insert_resource(SimRng(rng));

    simulation_state.set(SimulationState::Run).unwrap();
}

/// Update agents velocity component.
fn update_agents(
    time: Res<Time>,
    simulation_settings: Res<SimulationSettings>,
    simulation_speed: Res<SimulationSpeed>,
    mut rng: ResMut<SimRng>,
    agents: Query<(&Transform, &AgentBehaviour, &FriendFoe)>,
    mut velocities: Query<
        (Entity, &mut Velocity),
        (With<Transform>, With<AgentBehaviour>, With<FriendFoe>),
    >,
) {
    for (entity, mut velocity) in velocities.iter_mut() {
        let (transform, behaviour, FriendFoe(friend, foe)) = agents.get(entity).unwrap();
        let friend_pos = agents.get(*friend).unwrap().0.translation;
        let foe_pos = agents.get(*foe).unwrap().0.translation;

        // compute a velocity vector to fiend and to (or from) foe
        // based on the bahaviour of the agent.
        let (to_friend, to_foe) = match behaviour {
            // move toward its friend and its foe
            AgentBehaviour::Heroe => (
                friend_pos - transform.translation,
                foe_pos - transform.translation,
            ),
            // move in the direction of its friend and
            // in the opposite direction of its foe
            AgentBehaviour::Coward => (
                friend_pos - transform.translation,
                transform.translation - foe_pos,
            ),
        };

        let desired_velocity = if simulation_settings.use_vision_limit {
            let can_see_friend = to_friend.length() < simulation_settings.vision_limit;
            let can_see_foe = to_foe.length() < simulation_settings.vision_limit;
            match (can_see_friend, can_see_foe) {
                (true, true) => to_friend + to_foe,
                (true, false) => to_friend,
                (false, true) => to_foe,
                (false, false) => match simulation_settings.blind_behaviour {
                    BlindBehavour::NoMove => Vec3::ZERO,
                    BlindBehavour::RandomMove => {
                        let rng = &mut rng.0;
                        let a = rng.gen_range(0.0..std::f32::consts::TAU);

                        velocity
                            .0
                            .lerp(Vec3::new(a.cos(), a.sin(), 0.0), time.delta_seconds())
                    }
                },
            }
        } else {
            to_friend + to_foe
        };

        velocity.0 = desired_velocity.normalize_or_zero() * simulation_speed.0;
    }
}

/// This system ensure agents don't move out the arena.
fn keep_in_arena(
    simulation_settings: Res<SimulationSettings>,
    mut agents: Query<&mut Transform, With<AgentBehaviour>>,
) {
    for mut transform in agents.iter_mut() {
        transform.translation = transform
            .translation
            .max(Vec3::new(
                -simulation_settings.arena_size,
                -simulation_settings.arena_size,
                0.0,
            ))
            .min(Vec3::new(
                simulation_settings.arena_size,
                simulation_settings.arena_size,
                0.0,
            ));
    }
}

fn display_lines(
    settings: Res<SimulationDebug>,
    mut lines: ResMut<DebugLines>,
    agents: Query<(&Transform, &FriendFoe), With<Agent>>,
) {
    const ARROW_POS_OFFSET: f32 = 10.0;
    if settings.display_friend_links || settings.display_foe_links {
        for (transform, FriendFoe(friend, foe)) in agents.iter() {
            let friend_pos = agents.get(*friend).unwrap().0.translation;
            let foe_pos = agents.get(*foe).unwrap().0.translation;
            let pos = transform.translation;

            if settings.display_friend_links {
                lines.arrow_colored(
                    pos + ARROW_POS_OFFSET * Vec3::Y,
                    friend_pos + ARROW_POS_OFFSET * Vec3::Y,
                    0.0,
                    Color::GREEN,
                );
            }
            if settings.display_foe_links {
                lines.arrow_colored(
                    pos - ARROW_POS_OFFSET * Vec3::Y,
                    foe_pos - ARROW_POS_OFFSET * Vec3::Y,
                    0.0,
                    Color::RED,
                );
            }
        }
    }
}

fn display_center_of_mass(
    settings: Res<SimulationDebug>,
    mut lines: ResMut<DebugLines>,
    stats: Res<SimStats>,
) {
    const SIZE: f32 = 5.0;
    if settings.center_of_mass {
        let a1 = stats.center_of_mass + Vec2::new(-SIZE, -SIZE);
        let a2 = stats.center_of_mass + Vec2::new(SIZE, SIZE);

        let b1 = stats.center_of_mass + Vec2::new(0.0, -SIZE);
        let b2 = stats.center_of_mass + Vec2::new(0.0, SIZE);

        let c1 = stats.center_of_mass + Vec2::new(SIZE, -SIZE);
        let c2 = stats.center_of_mass + Vec2::new(-SIZE, SIZE);

        let d1 = stats.center_of_mass + Vec2::new(SIZE, 0.0);
        let d2 = stats.center_of_mass + Vec2::new(-SIZE, 0.0);

        let color = Color::rgb(0.4824, 0.0, 0.1725);

        lines.line_colored(a1.extend(0.0), a2.extend(0.0), 0.0, color);
        lines.line_colored(b1.extend(0.0), b2.extend(0.0), 0.0, color);
        lines.line_colored(c1.extend(0.0), c2.extend(0.0), 0.0, color);
        lines.line_colored(d1.extend(0.0), d2.extend(0.0), 0.0, color);
    }

    if settings.deviation {
        let color = Color::rgb(0.0, 0.4824, 0.3059);
        lines.circle_colored(
            stats.center_of_mass.extend(0.0),
            stats.deviation,
            0.0,
            color,
        );
    }
}
fn compute_stats(mut stats: ResMut<SimStats>, agents: Query<&Transform, With<Agent>>) {
    let mut agent_count: u32 = 0;

    let center_of_mass = {
        let mut sum = Vec2::ZERO;
        for transform in agents.iter() {
            sum += transform.translation.xy();
            agent_count += 1;
        }

        sum / agent_count as f32
    };

    let deviation = {
        let mut sum: f32 = 0.0;
        for transform in agents.iter() {
            sum += (transform.translation.xy() - center_of_mass).length();
        }
        sum / agent_count as f32
    };

    stats.center_of_mass = center_of_mass;
    stats.deviation = deviation;
}
