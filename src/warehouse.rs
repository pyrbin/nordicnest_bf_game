use std::time::Duration;

use bevy_tweening::lens::TransformScaleLens;

use crate::{prelude::*, ClosestParcel, Despawn, ImageAssets, ModelAssets, Parcel, Picked};
pub struct WarehousePlugin;

impl Plugin for WarehousePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ScoreEvent>();
        app.insert_resource(Score { score: 0 });
        app.add_system_set(SystemSet::on_enter(GameState::Ready).with_system(setup_ground));
        app.add_system_set(
            SystemSet::on_update(GameState::Ready)
                .with_system(move_truck)
                .with_system(collect_parcels)
                .with_system(update_score),
        );
    }
}

#[derive(Component)]
pub struct Ground;

#[derive(Component, PartialEq, Eq, Hash)]
pub enum AgentServiceCode {
    PostNord,
    DHL,
    Bring,
    Budbee,
}

impl AgentServiceCode {
    pub fn color(&self) -> Color {
        match self {
            AgentServiceCode::PostNord => Color::rgb(0.0, 0.62, 0.84),
            AgentServiceCode::DHL => Color::rgb(1.0, 0.8, 0.0),
            AgentServiceCode::Bring => Color::rgb(1., 1., 1.),
            AgentServiceCode::Budbee => Color::rgb(0.32, 0.75, 0.62),
        }
    }
}

#[derive(Component)]
pub struct Truck;

#[derive(Component)]
pub struct ShippingArea {
    pub score: i32,
    pub received_parcels: u64,
    pub truck: Entity,
}

fn setup_ground(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    assets: Res<ModelAssets>,
    texture_assets: Res<ImageAssets>,
) {
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 5500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.0, config::PARCEL_SPAWN_Y, 0.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // ground
    commands
        .spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Plane {
                    size: config::GROUND_SIZE,
                })),
                material: materials.add(Color::rgb(1.0, 0.5, 0.3).into()),
                transform: Transform::from_xyz(0.0, -0.5, 0.0),
                ..default()
            },
            RigidBody::Fixed,
        ))
        .with_children(|b| {
            b.spawn((
                TransformBundle {
                    local: Transform::from_xyz(0.0, -config::GROUND_DEPTH, 0.0),
                    ..default()
                },
                Ground,
                ActiveEvents::COLLISION_EVENTS,
                Collider::cuboid(
                    config::GROUND_SIZE / 2.0,
                    config::GROUND_DEPTH,
                    config::GROUND_SIZE / 2.0,
                ),
            ));
        });

    let agent_codes = vec![
        (AgentServiceCode::PostNord, Vec3::new(0.0, 1.0, -1.0), 0.0),
        (AgentServiceCode::DHL, Vec3::new(1.0, 1.0, 0.0), -90.0),
        (AgentServiceCode::Bring, Vec3::new(0.0, 1.0, 1.0), -180.0),
        (AgentServiceCode::Budbee, Vec3::new(-1.0, 1.0, 0.0), -260.0),
    ];

    for (code, pos, rot) in agent_codes {
        let offset = config::GROUND_SIZE;
        let truck_padding = 3.0;

        let truck = commands
            .spawn((
                SceneBundle {
                    scene: assets.truck.clone(),
                    transform: Transform {
                        translation: Vec3::new(
                            pos.x * (offset + truck_padding),
                            pos.y * 5.0,
                            pos.z * (offset + truck_padding),
                        ),
                        scale: Vec3::ONE * 0.05,
                        rotation: Quat::from_rotation_y((rot as f32).to_radians()),
                    },
                    ..Default::default()
                },
                Truck,
                Name::new("Truck".to_string()),
            ))
            .id();

        commands
            .spawn((
                PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Plane {
                        size: config::GROUND_SIZE,
                    })),
                    material: materials.add(StandardMaterial {
                        base_color: code.color(),
                        base_color_texture: match code {
                            AgentServiceCode::PostNord => Some(texture_assets.postnord.clone()),
                            AgentServiceCode::DHL => Some(texture_assets.dhl.clone()),
                            AgentServiceCode::Bring => Some(texture_assets.bring.clone()),
                            AgentServiceCode::Budbee => Some(texture_assets.budbee.clone()),
                        },
                        alpha_mode: AlphaMode::Blend,
                        depth_bias: -100.0,
                        ..default()
                    }),
                    transform: Transform::from_xyz(pos.x * offset, pos.y * -0.5, pos.z * offset),
                    ..default()
                },
                code,
                ShippingArea {
                    truck,
                    score: 0,
                    received_parcels: 0,
                },
                Sensor,
                ActiveEvents::COLLISION_EVENTS,
                Collider::cuboid(
                    config::GROUND_SIZE / 2.0,
                    config::GROUND_DEPTH / 2.0,
                    config::GROUND_SIZE / 2.0,
                ),
                Name::new("Shipping Area".to_string()),
            ))
            .with_children(|b| {
                b.spawn((
                    TransformBundle {
                        local: Transform::from_xyz(0.0, -config::GROUND_DEPTH, 0.0),
                        ..default()
                    },
                    Collider::cuboid(
                        config::GROUND_SIZE / 2.0,
                        config::GROUND_DEPTH,
                        config::GROUND_SIZE / 2.0,
                    ),
                ));
            });
    }
}

#[derive(Resource)]
pub struct Score {
    pub score: i32,
}

pub struct ScoreEvent {
    pub score: i32,
}

fn collect_parcels(
    mut commands: Commands,
    mut collisions: EventReader<CollisionEvent>,
    mut score_events: EventWriter<ScoreEvent>,
    mut closest_parcel: ResMut<ClosestParcel>,
    mut parcels: Query<
        (
            Entity,
            &Transform,
            &Collider,
            &AgentServiceCode,
            &mut OutlineVolume,
        ),
        With<Parcel>,
    >,
    trucks: Query<(&Transform, Option<&MoveTruck>), With<Truck>>,
    mut shipping_areas: Query<(
        Entity,
        &Transform,
        &Collider,
        &AgentServiceCode,
        &mut ShippingArea,
    )>,
) {
    for event in collisions.iter() {
        if let CollisionEvent::Started(e1, e2, _) = event {
            // check if collision is between a parcel and a shipping area

            let mut parcel = if let Ok(parcel) = parcels.get_mut(*e1) {
                parcel
            } else if let Ok(parcel) = parcels.get_mut(*e2) {
                parcel
            } else {
                continue;
            };

            let mut shipping_area = if let Ok(shipping_area) = shipping_areas.get_mut(*e1) {
                shipping_area
            } else if let Ok(shipping_area) = shipping_areas.get_mut(*e2) {
                shipping_area
            } else {
                continue;
            };

            let (score, despawn_timer) = if (*parcel.3) != (*shipping_area.3) {
                (-1, 600)
            } else {
                (1, 600)
            };

            // despawn parcel
            commands
                .entity(parcel.0)
                .remove::<Parcel>()
                .insert(Picked)
                .insert(Animator::new(
                    Tween::new(
                        EaseFunction::QuadraticInOut,
                        Duration::from_millis(despawn_timer),
                        TransformScaleLens {
                            start: Vec3::new(1., 1., 1.),
                            end: Vec3::new(0.0, 0.0, 0.0),
                        },
                    )
                    .with_repeat_count(RepeatCount::Finite(1)),
                ))
                .insert(Despawn::from_millis(despawn_timer));

            if let Some(p) = closest_parcel.0 {
                if p == parcel.0 {
                    closest_parcel.0 = None;
                }
            }

            parcel.4.visible = false;

            // emit score event
            score_events.send(ScoreEvent { score });

            shipping_area.4.score += score;
            shipping_area.4.received_parcels += 1;

            let (truck_transform, move_truck) = trucks.get(shipping_area.4.truck).unwrap();
            if move_truck.is_none() {
                commands.entity(shipping_area.4.truck).insert(MoveTruck {
                    origin: truck_transform.translation,
                    timer: Timer::from_seconds(3.0, TimerMode::Once),
                });
            }
        }
    }
}

fn update_score(mut score: ResMut<Score>, mut score_events: EventReader<ScoreEvent>) {
    for event in score_events.iter() {
        score.score += event.score;
    }
}

#[derive(Component)]
pub struct MoveTruck {
    pub origin: Vec3,
    pub timer: Timer,
}

fn move_truck(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut MoveTruck), With<Truck>>,
    time: Res<Time>,
) {
    const SPEED: f32 = 10.0;

    for (entity, mut transform, mut truck) in query.iter_mut() {
        truck.timer.tick(time.delta());
        if truck.timer.just_finished() {
            transform.translation = truck.origin;
            commands.entity(entity).remove::<MoveTruck>();
        } else {
            let forward = transform.right();
            transform.translation += forward * SPEED * time.delta_seconds();
        }
    }
}
