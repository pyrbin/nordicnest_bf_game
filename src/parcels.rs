use std::time::Duration;

use bevy_spatial::{RTreeAccess3D, RTreePlugin3D};

use crate::{prelude::*, AgentServiceCode, ImageAssets, PopParcelFromStack};
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
    pub fn from_millis(milli: u64) -> Self {
        Self {
            timer: Timer::new(Duration::from_millis(milli), TimerMode::Once),
        }
    }
}

#[derive(Resource)]
pub struct ParcelSpawner {
    pub timer: Timer,
    pub parent: Entity,
    pub count: u64,
}

fn setup_parcel_spawner(mut commands: Commands) {
    let parcel_parent = commands
        .spawn((
            Name::new("Parcels Container"),
            SpatialBundle {
                transform: Transform::from_xyz(0., 0., 0.),
                ..Default::default()
            },
        ))
        .id();
    commands.insert_resource(ParcelSpawner {
        timer: Timer::new(
            Duration::from_millis(config::PARCEL_SPAWN_RATE),
            TimerMode::Repeating,
        ),
        parent: parcel_parent,
        count: 0,
    })
}

fn spawn_parcels(
    mut commands: Commands,
    time: Res<Time>,
    mut spawner: ResMut<ParcelSpawner>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    texture_assets: Res<ImageAssets>,
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

    let mut cube_mesh = Mesh::from(shape::Cube {
        size: config::PARCEL_SIZE,
    });
    cube_mesh.generate_outline_normals().unwrap();

    commands
        .spawn((
            PbrBundle {
                mesh: meshes.add(cube_mesh),
                material: materials.add(StandardMaterial {
                    base_color: agent_code.color().into(),
                    base_color_texture: match agent_code {
                        AgentServiceCode::PostNord => Some(texture_assets.postnord.clone()),
                        AgentServiceCode::DHL => Some(texture_assets.dhl.clone()),
                        AgentServiceCode::Bring => Some(texture_assets.bring.clone()),
                        AgentServiceCode::Budbee => Some(texture_assets.budbee.clone()),
                    },
                    alpha_mode: AlphaMode::Blend,
                    depth_bias: 5.0,
                    ..default()
                }),
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
                coefficient: 2.0,
                combine_rule: CoefficientCombineRule::Average,
            },
            OutlineBundle {
                outline: OutlineVolume {
                    visible: false,
                    colour: Color::rgba(1.0, 1.0, 1.0, 0.8),
                    width: 4.0,
                },
                ..default()
            },
            Name::new(format!("Parcel")),
        ))
        .set_parent(spawner.parent);

    spawner.count += 1;

    if spawner.count % config::PARCEL_LEVEL_UP == 0 {
        let new_rate = (config::PARCEL_LEVEL_UP_DECR * (spawner.count / config::PARCEL_LEVEL_UP))
            .min(config::PARCEL_LEVEL_UP_MIN);

        spawner.timer = Timer::new(
            Duration::from_millis(config::PARCEL_SPAWN_RATE - new_rate),
            TimerMode::Repeating,
        );
    }
}

fn despawn_out_of_bounds(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform), (With<Parcel>, Without<Despawn>)>,
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
    mut query: Query<(Entity, &mut OutlineVolume, &mut Despawn)>,
    mut events: EventWriter<PopParcelFromStack>,
) {
    for (entity, mut volume, mut despawn) in query.iter_mut() {
        despawn.timer.tick(time.delta());
        volume.visible = false;
        if despawn.timer.just_finished() {
            commands.entity(entity).despawn();
            events.send(PopParcelFromStack {
                parcel_entry: entity,
                despawning: true,
            });
        }
    }
}

fn rand_parcel_spawn() -> Vec3 {
    const PADDING: f32 = 1.25;
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

pub fn rand_parcel_linvel() -> Vec3 {
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
