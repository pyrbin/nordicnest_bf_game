use std::time::Duration;

use bevy_spatial::{RTreeAccess3D, RTreePlugin3D};

use crate::{prelude::*, AgentServiceCode};
pub struct ParcelsPlugin;

impl Plugin for ParcelsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(RTreePlugin3D::<Parcel> { ..default() });
        app.add_system_set(SystemSet::on_enter(GameState::Ready).with_system(setup_parcel_spawner))
            .add_system_set(
                SystemSet::on_update(GameState::Ready)
                    .with_system(spawn_parcels)
                    .with_system(despawn_out_of_bounds)
                    .with_system(despawn_with_timer),
            );
    }
}

#[derive(Component)]
pub struct Parcel;

#[derive(Component)]
pub struct Despawn {
    pub timer: Timer,
}

pub type ParcelsSpatialTree = RTreeAccess3D<Parcel>; // type alias for brevity

impl Despawn {
    pub fn from_secs(sec: u64) -> Self {
        Self {
            timer: Timer::new(Duration::from_secs(sec), TimerMode::Once),
        }
    }
}

#[derive(Resource)]
struct ParcelSpawner {
    timer: Timer,
}

fn setup_parcel_spawner(mut commands: Commands) {
    commands.insert_resource(ParcelSpawner {
        timer: Timer::new(
            Duration::from_millis(config::PARCEL_SPAWN_RATE),
            TimerMode::Repeating,
        ),
    })
}

fn spawn_parcels(
    mut commands: Commands,
    time: Res<Time>,
    mut spawner: ResMut<ParcelSpawner>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    spawner.timer.tick(time.delta());

    if !spawner.timer.just_finished() {
        return;
    }

    // random agent code
    let agent_code = match rand::random::<u8>() % 4 {
        0 => AgentServiceCode::PostNord,
        1 => AgentServiceCode::DHL,
        2 => AgentServiceCode::Bring,
        3 => AgentServiceCode::Budbee,
        _ => unreachable!(),
    };

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube {
                size: config::PARCEL_SIZE,
            })),
            material: materials.add(agent_code.color().into()),
            transform: Transform::from_translation(rand_parcel_spawn()),
            ..Default::default()
        },
        agent_code,
        RigidBody::Dynamic,
        Velocity {
            linvel: rand_parcel_linvel(),
            angvel: Vec3::new(1.0, 0.0, 0.0),
        },
        Collider::cuboid(
            config::PARCEL_SIZE / 2.,
            config::PARCEL_SIZE / 2.,
            config::PARCEL_SIZE / 2.,
        ),
        Parcel,
        Friction {
            coefficient: 1.5,
            combine_rule: CoefficientCombineRule::Average,
        },
    ));
}

fn despawn_out_of_bounds(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform), With<Parcel>>,
) {
    const DESPAWN_HEIGHT: f32 = -30.0;
    const DISABLE_COLLISION_HEIGHT: f32 = -5.0;

    for (entity, transform) in query.iter_mut() {
        if transform.translation.y <= DISABLE_COLLISION_HEIGHT {
            commands.entity(entity).remove::<Collider>();
        }
        if transform.translation.y <= DESPAWN_HEIGHT {
            commands.entity(entity).despawn();
        }
    }
}

fn despawn_with_timer(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Despawn)>,
) {
    for (entity, mut despawn) in query.iter_mut() {
        despawn.timer.tick(time.delta());
        if despawn.timer.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn rand_parcel_spawn() -> Vec3 {
    const PADDING: f32 = 0.5;
    let point = random_point_in_area(
        Vec3::new(
            -config::GROUND_SIZE / 2.0 + PADDING,
            0.0,
            -config::GROUND_SIZE / 2.0 + PADDING,
        ),
        Vec3::new(
            config::GROUND_SIZE / 2.0 - PADDING,
            0.0,
            config::GROUND_SIZE / 2.0 - PADDING,
        ),
    );

    Vec3::new(point.x, config::PARCEL_SPAWN_Y, point.z)
}

fn rand_parcel_linvel() -> Vec3 {
    let x = rand::random::<f32>() * config::PARCEL_MAX_LINVEL_X;
    let y = 0.0;
    let z = rand::random::<f32>() * config::PARCEL_MAX_LINVEL_Z;

    Vec3::new(
        x.max(config::PARCEL_MIN_LINVEL_X),
        y,
        z.max(config::PARCEL_MIN_LINVEL_Z),
    )
}

fn random_point_in_area(a: Vec3, b: Vec3) -> Vec3 {
    let x = rand::random::<f32>() * (b.x - a.x) + a.x;
    let y = rand::random::<f32>() * (b.y - a.y) + a.y;
    let z = rand::random::<f32>() * (b.z - a.z) + a.z;

    Vec3::new(x, y, z)
}
