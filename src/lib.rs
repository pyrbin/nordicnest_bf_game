pub mod prelude;

use bevy::core_pipeline::clear_color::{self, ClearColorConfig};
use bevy_asset_loader::prelude::*;
use bevy_embedded_assets::EmbeddedAssetPlugin;
pub use prelude::*;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum GameState {
    Loading,
    Ready,
}

#[derive(AssetCollection, Resource)]
struct ImageAssets {
    #[asset(path = "textures/bevy.png")]
    icon: Handle<Image>,
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
    .add_plugin(RapierDebugRenderPlugin::default())
    .add_plugin(Sprite3dPlugin);

    app.add_state(GameState::Loading)
        .add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::Ready)
                .with_collection::<ImageAssets>(),
        )
        .add_system_set(
            SystemSet::on_enter(GameState::Ready)
                .with_system(setup)
                .with_system(setup_ground),
        )
        .add_system_set(SystemSet::on_update(GameState::Ready).with_system(player_movement));
    app
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct FaceCamera;

fn setup(mut commands: Commands, images: Res<ImageAssets>, mut sprite_params: Sprite3dParams) {
    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 6.0, 12.0).looking_at(Vec3::ZERO, Vec3::Y),
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
            image: images.icon.clone(),
            pixels_per_metre: 400.,
            partial_alpha: true,
            unlit: true,
            // transform: Transform::from_xyz(0., 0., 0.),
            // pivot: Some(Vec2::new(0.5, 0.5)),
            ..default()
        }
        .bundle(&mut sprite_params),
        Collider::ball(0.5),
        RigidBody::KinematicPositionBased,
        KinematicCharacterController::default(),
        FaceCamera,
        Player,
    ));
}

fn setup_ground(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // ground
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane {
                size: config::GROUND_SIZE,
            })),
            material: materials.add(Color::rgb(1.0, 0.5, 0.3).into()),
            transform: Transform::from_xyz(0.0, -0.5, 0.0),
            ..default()
        },
        Collider::cuboid(
            config::GROUND_SIZE / 2.0,
            config::GROUND_DEPTH,
            config::GROUND_SIZE / 2.0,
        ),
    ));
}

fn player_movement(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut KinematicCharacterController, &Transform), With<Player>>,
) {
    let (mut controller, tranform) = query.single_mut();
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

    let mut movement = delta.normalize_or_zero() * time.delta_seconds() * config::PLAYER_SPEED;
    let mut translation = tranform.translation + movement;

    if translation.x < -config::GROUND_SIZE / 2.0 {
        translation.x = -config::GROUND_SIZE / 2.0;
    }
    if translation.x > config::GROUND_SIZE / 2.0 {
        translation.x = config::GROUND_SIZE / 2.0;
    }
    if translation.z < -config::GROUND_SIZE / 2.0 {
        translation.z = -config::GROUND_SIZE / 2.0;
    }
    if translation.z > config::GROUND_SIZE / 2.0 {
        translation.z = config::GROUND_SIZE / 2.0;
    }

    movement = translation - tranform.translation;

    controller.translation = Some(movement);
}
