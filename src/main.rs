#![windows_subsystem = "windows"]

mod simulation;
mod utils;
mod velocity;

use bevy::input::mouse::MouseMotion;
use bevy::{
    // diagnostic,
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    render::camera::{Camera, CameraProjection, OrthographicProjection},
};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_prototype_debug_lines::*;

use simulation::*;

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Heroes and Cowards Simulator".to_string(),
            width: 900.0,
            height: 600.0,
            vsync: true,
            ..Default::default()
        })
        .init_resource::<UiState>()
        .add_plugins(DefaultPlugins)
        // // Adds frame time diagnostics
        // .add_plugin(diagnostic::FrameTimeDiagnosticsPlugin::default())
        // // Adds a system that prints diagnostics to the console
        // .add_plugin(diagnostic::LogDiagnosticsPlugin::default())
        .add_plugin(DebugLinesPlugin)
        .add_plugin(EguiPlugin)
        .add_plugin(HeroesCowardSimulationPlugin)
        .add_startup_system(setup.system())
        .add_system(ui.system().label("ui"))
        .add_system(ui_stats.system().after("ui"))
        .add_system(scroll_zoom.system())
        .add_system(move_camera.system())
        .run();
}

// ===== resources =====

struct UiState {
    random_seed: bool,
    simulation_settings: SimulationSettings,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            random_seed: true,
            simulation_settings: Default::default(),
        }
    }
}

// ===== components =====

/// Tag for the camera
struct MainCamera;

// ===== systems =====

fn setup(mut commands: Commands) {
    // spawn camera for agents
    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(MainCamera);
}

fn ui(
    mut commands: Commands,
    egui_context: ResMut<EguiContext>,
    mut ui_state: ResMut<UiState>,
    mut simulation_speed: ResMut<SimulationSpeed>,
    mut simulation_state: ResMut<State<SimulationState>>,
    mut debug_settings: ResMut<SimulationDebug>,
    simulation_settings: Res<SimulationSettings>,
) {
    egui::Window::new("Paramètres").show(egui_context.ctx(), |ui| {
        ui.vertical_centered_justified(|ui| {
            egui::Grid::new("grid_param").show(ui, |ui| {
                ui.heading("Seed");
                ui.end_row();

                ui.label("Aléatoire");
                ui.checkbox(&mut ui_state.random_seed, "");
                ui.end_row();

                ui.label("Valeur");
                ui.scope(|ui| {
                    ui.set_enabled(!ui_state.random_seed);
                    ui.add(
                        egui::DragValue::new(&mut ui_state.simulation_settings.seed)
                            .clamp_range(0..=u64::MAX),
                    );
                });
                ui.end_row();

                if ui.button("générer").clicked() {
                    ui_state.simulation_settings.seed = rand::random();
                }
                ui.end_row();

                ui.add_space(10.0);
                ui.end_row();

                ui.heading("Arène");
                ui.end_row();

                ui.label("Taille");
                ui.add(
                    egui::DragValue::new(&mut ui_state.simulation_settings.arena_size)
                        .clamp_range(10.0..=1000.0),
                );
                ui.end_row();

                ui.add_space(10.0);
                ui.end_row();

                ui.heading("Agent");
                ui.end_row();

                ui.label("Nombre");
                ui.add(
                    egui::DragValue::new(&mut ui_state.simulation_settings.agent_count)
                        .clamp_range(3..=2000),
                );
                ui.end_row();

                ui.label("Proportion de héros");
                ui.add(egui::Slider::new(
                    &mut ui_state.simulation_settings.heroe_proportion,
                    0.0..=1.0,
                ));
                ui.end_row();

                ui.label("Vision limitée");
                ui.scope(|ui| {
                    ui.checkbox(&mut ui_state.simulation_settings.use_vision_limit, "");
                    ui.set_enabled(ui_state.simulation_settings.use_vision_limit);
                    let vision_max = ui_state.simulation_settings.arena_size;
                    ui.add(egui::Slider::new(
                        &mut ui_state.simulation_settings.vision_limit,
                        0.0..=vision_max,
                    ));
                });
                ui.end_row();

                ui.label("Comportement si aveugle");
                ui.end_row();
                ui.vertical_centered_justified(|ui| {
                    ui.selectable_value(
                        &mut ui_state.simulation_settings.blind_behaviour,
                        BlindBehavour::NoMove,
                        "Immobile",
                    );
                    ui.selectable_value(
                        &mut ui_state.simulation_settings.blind_behaviour,
                        BlindBehavour::RandomMove,
                        "Movement aléatoire",
                    );
                });
                ui.end_row();
            });
            ui.add_space(20.0);
            if ui.button("Start").clicked() {
                if ui_state.random_seed {
                    ui_state.simulation_settings.seed = rand::random();
                }

                // update the simulation settings
                commands.insert_resource(ui_state.simulation_settings.clone());
                // start the simulation
                simulation_state.set(SimulationState::Start).unwrap(); // todo: handle the error
            }
        });
    });

    egui::Window::new("Simulation").show(egui_context.ctx(), |ui| {
        ui.vertical_centered_justified(|ui| {
            egui::Grid::new("grid_sim").show(ui, |ui| {
                ui.label("seed");
                ui.label(simulation_settings.seed);
                ui.end_row();

                ui.label("Vitesse");
                ui.add(egui::Slider::new(&mut simulation_speed.0, 1.0..=1000.0));
                ui.end_row();
            });

            ui.vertical(|ui| {
                ui.checkbox(
                    &mut debug_settings.display_friend_links,
                    "Afficher les liens d'amitié ?",
                );
                ui.checkbox(
                    &mut debug_settings.display_foe_links,
                    "Afficher les liens d'hostilité ?",
                );
                ui.checkbox(
                    &mut debug_settings.center_of_mass,
                    "Afficher le centre de masse ?",
                );
                ui.checkbox(&mut debug_settings.deviation, "Afficher la deviation ?");
            });

            ui.add_space(20.0);
            match simulation_state.current() {
                SimulationState::Run => {
                    if ui.button("Pause").clicked() {
                        simulation_state.set(SimulationState::Pause).unwrap(); // todo: handle error
                    }
                }
                SimulationState::Pause => {
                    if ui.button("Play").clicked() {
                        simulation_state.set(SimulationState::Run).unwrap(); // todo: handle error
                    }
                }
                _ => {
                    ui.scope(|ui| {
                        ui.set_enabled(false);
                        let _ = ui.button("Play");
                    });
                }
            }
        });
    });
}

fn ui_stats(egui_context: ResMut<EguiContext>, stats: Res<SimStats>) {
    egui::Window::new("Stats").show(egui_context.ctx(), |ui| {
        egui::Grid::new("grid_stats").show(ui, |ui| {
            ui.label("Centre de masse");
            ui.label(format!(
                "{:.2} - {:.2}",
                stats.center_of_mass.x, stats.center_of_mass.y
            ));
            ui.end_row();

            ui.label("Déviation");
            ui.label(format!("{:.4}", stats.deviation));
            ui.end_row();
        });
    });
}

fn scroll_zoom(
    time: Res<Time>,
    mut scroll_evr: EventReader<MouseWheel>,
    mut camera: Query<(&mut Camera, &mut OrthographicProjection), With<MainCamera>>,
) {
    const ZOOM_SPEED: f32 = 4.0;
    let scroll: f32 = scroll_evr
        .iter()
        .map(|ev| match ev.unit {
            MouseScrollUnit::Line => ev.y * 3.0,
            MouseScrollUnit::Pixel => ev.y,
        })
        .sum();

    let (mut camera, mut projection) = camera.single_mut().unwrap();
    projection.scale =
        (projection.scale - scroll * time.delta_seconds() * ZOOM_SPEED).clamp(0.5, 20.0);

    camera.projection_matrix = projection.get_projection_matrix();
    camera.depth_calculation = projection.depth_calculation();
}

fn move_camera(
    time: Res<Time>,
    buttons: Res<Input<MouseButton>>,
    mut motion_evr: EventReader<MouseMotion>,
    mut camera: Query<(&mut Transform, &OrthographicProjection), With<MainCamera>>,
) {
    const CAMERA_SPEED: f32 = 55.0;
    if buttons.pressed(MouseButton::Right) {
        let movement: Vec2 = motion_evr
            .iter()
            .map(|ev| ev.delta)
            .fold(Default::default(), |agg, cur| agg + cur);
        let movement = CAMERA_SPEED * time.delta_seconds() * movement;
        let movement = Vec3::new(-movement.x, movement.y, 0.0);

        for (mut cam, projection) in camera.iter_mut() {
            cam.translation += movement * projection.scale;
        }
    }
}
