mod parcels;
pub mod prelude;
mod state;
mod ui;
mod warehouse;

use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy_asset_loader::prelude::*;
use bevy_embedded_assets::EmbeddedAssetPlugin;

pub use crate::parcels::*;
pub use crate::ui::*;
pub use crate::warehouse::*;
use bevy_mod_fbx::FbxPlugin;
pub use prelude::*;

#[derive(AssetCollection, Resource)]
pub struct ImageAssets {
    #[asset(path = "textures/nordicnest_bird.png")]
    bird: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub struct FontAssets {
    #[asset(path = "fonts/FiraSans-Bold.ttf")]
    pub fira_sans: Handle<Font>,
}

pub fn setup_app(app: &mut App) -> &mut App {
    app.add_plugins(
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                window: WindowDescriptor {
                    width: 1080.0,
                    height: 1080.0 * 3. / 4.,
                    ..default()
                },

                ..default()
            })
            .build()
            .add_before::<AssetPlugin, EmbeddedAssetPlugin>(EmbeddedAssetPlugin),
    )
    .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
    //.add_plugin(RapierDebugRenderPlugin::default())
    .add_plugin(Sprite3dPlugin)
    .add_plugin(ParcelsPlugin)
    .add_plugin(WarehousePlugin)
    .add_plugin(UiPlugin)
    .add_plugin(FbxPlugin);

    app.add_state(GameState::Loading)
        .add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::Ready)
                .with_collection::<ImageAssets>()
                .with_collection::<FontAssets>(),
        )
        .add_system_set(
            SystemSet::on_enter(GameState::Ready)
                .with_system(setup)
                .with_system(setup_truck),
        )
        .add_system_set(SystemSet::on_enter(GameState::Ready).with_system(setup))
        .add_system_set(SystemSet::on_update(GameState::Ready).with_system(player_movement));
    app
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct FaceCamera;

fn setup_truck(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((SceneBundle {
        scene: asset_server.load("models/truck.fbx"),
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        ..default()
    },));
}

fn setup(mut commands: Commands, images: Res<ImageAssets>, mut sprite_params: Sprite3dParams) {
    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 12.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
        projection: bevy::prelude::Projection::Perspective(PerspectiveProjection::default()),
        camera: Camera { ..default() },
        camera_3d: Camera3d {
            clear_color: ClearColorConfig::Custom(Color::rgb(0.0, 0.0, 0.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    // player
    commands.spawn((
        Sprite3d {
            image: images.bird.clone(),
            pixels_per_metre: 800.,
            partial_alpha: true,
            unlit: true,
            ..default()
        }
        .bundle(&mut sprite_params),
        Collider::capsule(Vec3::Y / 2., Vec3::ZERO, 0.4),
        RigidBody::KinematicVelocityBased,
        Velocity::default(),
        KinematicCharacterController {
            slide: true,
            apply_impulse_to_dynamic_bodies: true,
            ..default()
        },
        FaceCamera,
        Player,
    ));
}

fn player_movement(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Velocity, With<Player>>,
) {
    let mut vel = query.single_mut();
    let mut delta = Vec3::ZERO;
    if keyboard_input.pressed(KeyCode::W) {
        delta -= Vec3::Z;
    }
    if keyboard_input.pressed(KeyCode::S) {
        delta += Vec3::Z;
    }
    if keyboard_input.pressed(KeyCode::A) {
        delta -= Vec3::X;
    }
    if keyboard_input.pressed(KeyCode::D) {
        delta += Vec3::X;
    }

    vel.linvel = delta.normalize_or_zero() * time.delta_seconds() * config::PLAYER_SPEED;
}
