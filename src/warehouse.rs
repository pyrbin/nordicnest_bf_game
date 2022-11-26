use crate::{prelude::*, Despawn, Parcel};
pub struct WarehousePlugin;

impl Plugin for WarehousePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ScoreEvent>();
        app.insert_resource(Score { score: 0 });
        app.add_system_set(SystemSet::on_enter(GameState::Ready).with_system(setup_ground));
        app.add_system_set(
            SystemSet::on_update(GameState::Ready)
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
            AgentServiceCode::PostNord => Color::rgb(0.0, 0.0, 1.0),
            AgentServiceCode::DHL => Color::rgb(1.0, 0.0, 0.0),
            AgentServiceCode::Bring => Color::rgb(0.0, 1.0, 0.0),
            AgentServiceCode::Budbee => Color::rgb(0.32, 0.75, 0.62),
        }
    }
}

#[derive(Component)]
pub struct ShippingArea;

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
        transform: Transform::from_xyz(
            -config::GROUND_SIZE / 2.0,
            config::PARCEL_SPAWN_Y,
            -config::GROUND_SIZE / 2.0,
        )
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
        (AgentServiceCode::PostNord, Vec3::new(0.0, 1.0, -1.0)),
        (AgentServiceCode::DHL, Vec3::new(1.0, 1.0, 0.0)),
        (AgentServiceCode::Bring, Vec3::new(0.0, 1.0, 1.0)),
        (AgentServiceCode::Budbee, Vec3::new(-1.0, 1.0, 0.0)),
    ];

    for (code, pos) in agent_codes {
        let offset = config::GROUND_SIZE;
        commands
            .spawn((
                PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Plane {
                        size: config::GROUND_SIZE,
                    })),
                    material: materials.add(code.color().into()),
                    transform: Transform::from_xyz(pos.x * offset, pos.y * -0.5, pos.z * offset),
                    ..default()
                },
                code,
                ShippingArea,
                Sensor,
                ActiveEvents::COLLISION_EVENTS,
                Collider::cuboid(
                    config::GROUND_SIZE / 2.0,
                    config::GROUND_DEPTH,
                    config::GROUND_SIZE / 2.0,
                ),
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
    parcels: Query<(Entity, &Transform, &Collider, &AgentServiceCode), With<Parcel>>,
    shipping_areas: Query<(Entity, &Transform, &Collider, &AgentServiceCode), With<ShippingArea>>,
) {
    for event in collisions.iter() {
        match event {
            CollisionEvent::Started(e1, e2, _) => {
                // check if collision is between a parcel and a shipping area

                let parcel = if let Ok(parcel) = parcels.get(*e1) {
                    parcel
                } else if let Ok(parcel) = parcels.get(*e2) {
                    parcel
                } else {
                    continue;
                };

                let shipping_area = if let Ok(shipping_area) = shipping_areas.get(*e1) {
                    shipping_area
                } else if let Ok(shipping_area) = shipping_areas.get(*e2) {
                    shipping_area
                } else {
                    continue;
                };

                let (score, despawn_timer) = if (*parcel.3) != (*shipping_area.3) {
                    (-1, 1)
                } else {
                    (1, 5)
                };

                // despawn parcel
                commands
                    .entity(parcel.0)
                    .remove::<Parcel>()
                    .insert(Despawn::from_secs(despawn_timer));

                // emit score event
                score_events.send(ScoreEvent { score });
            }
            _ => {}
        }
    }
}

fn update_score(mut score: ResMut<Score>, mut score_events: EventReader<ScoreEvent>) {
    for event in score_events.iter() {
        score.score += event.score;
    }
}
