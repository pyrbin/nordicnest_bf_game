mod debug;
mod game_over;
mod parcels;
mod player;
pub mod prelude;
mod state;
mod ui;
mod warehouse;

use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::window::PresentMode;
use bevy_asset_loader::prelude::*;
use bevy_embedded_assets::EmbeddedAssetPlugin;

pub use crate::game_over::*;
pub use crate::parcels::*;
pub use crate::player::*;
pub use crate::ui::*;
pub use crate::warehouse::*;
pub use prelude::*;

#[derive(AssetCollection, Resource)]
pub struct ModelAssets {
    #[asset(path = "models/truck.glb#Scene0")]
    truck: Handle<Scene>,
}

#[derive(AssetCollection, Resource)]
pub struct AudioAssets {
    #[asset(path = "audio/anthem.ogg")]
    anthem: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
pub struct ImageAssets {
    #[asset(path = "images/nordicnest_bird.png")]
    bird: Handle<Image>,
    #[asset(path = "images/dhl.png")]
    dhl: Handle<Image>,
    #[asset(path = "images/postnord.png")]
    postnord: Handle<Image>,
    #[asset(path = "images/bring.png")]
    bring: Handle<Image>,
    #[asset(path = "images/budbee.png")]
    budbee: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub struct FontAssets {
    #[asset(path = "fonts/Montserrat-Regular.ttf")]
    pub montserrat: Handle<Font>,
}

pub fn setup_app(app: &mut App) -> &mut App {
    app.add_plugins(
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                window: WindowDescriptor {
                    title: "Black Friday".to_string(),
                    present_mode: PresentMode::AutoNoVsync,
                    width: 1280.0,
                    height: 720.0,
                    position: WindowPosition::Automatic,
                    scale_factor_override: Some(1.0),
                    resizable: true,
                    decorations: true,
                    cursor_visible: true,
                    mode: WindowMode::Windowed,
                    transparent: false,
                    canvas: Some("#bevy".to_string()),
                    fit_canvas_to_parent: true,
                    ..default()
                },

                ..default()
            })
            .build()
            .add_before::<AssetPlugin, EmbeddedAssetPlugin>(EmbeddedAssetPlugin),
    )
    //.add_plugin(WorldInspectorPlugin::new())
    .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
    //.add_plugin(RapierDebugRenderPlugin::default())
    .add_plugin(DebugLinesPlugin::with_depth_test(true))
    .add_plugin(Sprite3dPlugin)
    .add_plugin(TweeningPlugin)
    .add_plugin(PlayerPlugin)
    .add_plugin(ParcelsPlugin)
    .add_plugin(WarehousePlugin)
    .add_plugin(OutlinePlugin)
    .add_plugin(UiPlugin);

    app.add_state(GameState::Loading)
        .add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::Ready)
                .with_collection::<ImageAssets>()
                .with_collection::<FontAssets>()
                .with_collection::<AudioAssets>()
                .with_collection::<ModelAssets>(),
        )
        .add_system_set(SystemSet::on_enter(GameState::Ready).with_system(setup))
        .add_system_set(SystemSet::on_update(GameState::Ready).with_system(check_game_over));
    app
}

#[derive(Resource)]
pub struct TimeRemaining {
    pub timer: Timer,
}

#[derive(Component)]
struct FaceCamera;

#[derive(Component)]
pub struct MainCamera;

fn setup(mut commands: Commands, audio_assets: Res<AudioAssets>, audio: Res<Audio>) {
    // camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 13.0, 22.0).looking_at(Vec3::ZERO, Vec3::Y),
            projection: bevy::prelude::Projection::Perspective(PerspectiveProjection::default()),
            camera: Camera { ..default() },
            camera_3d: Camera3d {
                clear_color: ClearColorConfig::Custom(Color::rgb(0.0, 0.0, 0.0)),
                ..Default::default()
            },
            tonemapping: Tonemapping::Enabled {
                deband_dither: true,
            },
            ..Default::default()
        },
        MainCamera,
    ));

    audio.play(audio_assets.anthem.clone());

    commands.insert_resource(TimeRemaining {
        timer: Timer::from_seconds(config::GAME_TIME, TimerMode::Once),
    });
}

fn check_game_over(
    mut app_state: ResMut<State<GameState>>,
    time: Res<Time>,
    mut time_remaining: ResMut<TimeRemaining>,
) {
    time_remaining.timer.tick(time.delta());
    if time_remaining.timer.just_finished() {
        app_state.set(GameState::GameOver).unwrap();
    }
}
