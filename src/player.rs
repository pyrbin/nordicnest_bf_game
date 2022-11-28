use std::time::Duration;

use bevy_spatial::SpatialAccess;
use bevy_tweening::lens::TransformScaleLens;

use crate::{
    prelude::*, Despawn, FaceCamera, ImageAssets, Parcel, ParcelSpawner, ParcelsSpatialTree,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClosestParcel(None));
        app.insert_resource(MousePosition::default());
        app.add_event::<AddParcelToStack>();
        app.add_event::<PopParcelFromStack>();
        app.add_system_set(SystemSet::on_enter(GameState::Ready).with_system(setup))
            .add_system_set(
                SystemSet::on_update(GameState::Ready)
                    .with_system(update_mouse_hover_pos)
                    .with_system(player_movement)
                    .with_system(parcel_awarness)
                    .with_system(parcel_stack_events)
                    .with_system(maintain_parcel_stack)
                    .with_system(pickup_parcel)
                    .with_system(pop_parcel)
                    .with_system(pop_despawning_parcels_from_pick)
                    .with_system(remove_outline_from_picked),
            );
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerGfx;

#[derive(Component)]
struct ParcelStack {
    parcels_entries: Vec<Entity>,
}

#[derive(Component)]
struct ParcelStackEntry {
    parcel: Option<Entity>,
}

#[derive(Component)]
pub struct Picked;

fn setup(mut commands: Commands, images: Res<ImageAssets>, mut sprite_params: Sprite3dParams) {
    // player
    commands
        .spawn((
            SpatialBundle {
                transform: Transform::from_xyz(0., 0., 0.),
                ..Default::default()
            },
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
            Name::new("Player"),
        ))
        .add_children(|b| {
            b.spawn((
                SpatialBundle {
                    transform: Transform::from_translation(Vec3::Y / 2.),
                    ..Default::default()
                },
                Name::new("Parcel Stack"),
                ParcelStack {
                    parcels_entries: vec![],
                },
            ));
            b.spawn((
                Sprite3d {
                    image: images.bird.clone(),
                    pixels_per_metre: 600.,
                    partial_alpha: true,
                    unlit: true,
                    double_sided: true,
                    ..default()
                }
                .bundle(&mut sprite_params),
                Name::new("Player Gfx"),
                PlayerGfx,
                Animator::new(
                    Tween::new(
                        EaseFunction::QuadraticInOut,
                        Duration::from_millis(666),
                        TransformScaleLens {
                            start: Vec3::new(1., 1., 1.),
                            end: Vec3::new(0.85, 0.85, 0.85),
                        },
                    )
                    .with_repeat_count(RepeatCount::Infinite)
                    .with_repeat_strategy(RepeatStrategy::MirroredRepeat),
                ),
            ));
        });
}

pub struct AddParcelToStack {
    pub parcel: Entity,
}

pub struct PopParcelFromStack {
    pub parcel_entry: Entity,
    pub despawning: bool,
}

fn parcel_stack_events(
    mut commands: Commands,
    mut events: EventReader<AddParcelToStack>,
    mut pop_events: EventReader<PopParcelFromStack>,
    mut parcel_stack: Query<(&mut ParcelStack, Entity)>,
    spawner: Res<ParcelSpawner>,
    mouse_pos: Res<MousePosition>,
    mut parcel_stack_entries: Query<&mut ParcelStackEntry>,
    mut parcels: Query<
        (
            &mut Transform,
            &mut Velocity,
            &mut OutlineVolume,
            &GlobalTransform,
        ),
        With<Parcel>,
    >,
) {
    for event in events.iter() {
        let (mut stack, stack_entity) = parcel_stack.single_mut();

        let index = stack.parcels_entries.len();

        let entry = commands
            .spawn((
                TransformBundle {
                    local: Transform::from_translation(get_parcel_stack_pos(index)),
                    ..Default::default()
                },
                VisibilityBundle::default(),
                ParcelStackEntry {
                    parcel: Some(event.parcel),
                },
            ))
            .set_parent(stack_entity)
            .id();

        commands
            .entity(event.parcel)
            .insert(Picked)
            .set_parent(entry)
            .insert(GravityScale(0.0))
            .insert(Sensor);

        if let Ok((mut transform, mut velocity, mut outline, _)) = parcels.get_mut(event.parcel) {
            transform.translation = Vec3::ZERO;
            velocity.linvel = Vec3::ZERO;
            velocity.angvel = Vec3::ZERO;
            outline.visible = false;
        }

        stack.parcels_entries.push(entry);
    }

    for event in pop_events.iter() {
        let (mut stack, _) = parcel_stack.single_mut();

        let index = stack
            .parcels_entries
            .iter()
            .position(|e| *e == event.parcel_entry);

        if let Some(index) = index {
            stack.parcels_entries.remove(index);
            stack.parcels_entries.shrink_to_fit();

            if let Ok(entry) = parcel_stack_entries.get_mut(event.parcel_entry) {
                if let Some(parcel) = entry.parcel {
                    commands
                        .entity(parcel)
                        .remove::<Picked>()
                        .remove::<Sensor>()
                        .insert(GravityScale(1.))
                        .set_parent(spawner.parent);

                    if let Ok((mut transform, mut velocity, _, global)) = parcels.get_mut(parcel) {
                        transform.translation = global.translation();

                        if let Some(pos) = mouse_pos.0 {
                            let linvel = ((pos - transform.translation)
                                * config::PLAYER_THROW_FACTOR)
                                .clamp_length_max(config::PLAYER_MAX_THROW_MAQ)
                                .clamp_length_min(2.0);

                            velocity.linvel = linvel + Vec3::Y * 7.;
                        } else {
                            velocity.linvel = Vec3::NEG_Y * 0.05;
                        }
                    }
                }
            }

            commands.entity(event.parcel_entry).despawn();
        }
    }
}

fn pop_despawning_parcels_from_pick(
    mut commands: Commands,
    mut events: EventWriter<PopParcelFromStack>,
    parcels: Query<(Entity, &Picked, &Despawn)>,
) {
    for (entity, _, _) in parcels.iter() {
        commands.entity(entity).remove::<Picked>();
        events.send(PopParcelFromStack {
            parcel_entry: entity,
            despawning: true,
        });
    }
}

fn maintain_parcel_stack(
    parcel_stack: Query<&ParcelStack>,
    mut parcel_stack_entry: Query<&mut Transform>,
) {
    let stack = parcel_stack.single();

    for (i, entity) in stack.parcels_entries.iter().enumerate() {
        if let Ok(mut transform) = parcel_stack_entry.get_mut(*entity) {
            transform.translation = get_parcel_stack_pos(i);
        }
    }
}

fn get_parcel_stack_pos(index: usize) -> Vec3 {
    Vec3::Y * (index as f32 + 1.0) * config::PARCEL_SIZE * 0.95
}

#[derive(Resource)]
pub struct ClosestParcel(pub Option<Entity>);

fn parcel_awarness(
    player: Query<&Transform, With<Player>>,
    mut closest_parcel: ResMut<ClosestParcel>,
    mut parcels: Query<&mut OutlineVolume, (With<Parcel>, Without<Picked>, Without<Despawn>)>,
    closest: Res<ParcelsSpatialTree>,
) {
    let player_transform = player.single();

    const PLAYER_PICKUP_RADIUS: f32 = 3.0;

    let mut min: Option<(Entity, f32)> = None;

    for (pos, entity) in closest.within_distance(player_transform.translation, PLAYER_PICKUP_RADIUS)
    {
        let dist = pos.distance(player_transform.translation);
        if parcels.get_mut(entity).is_ok() {
            if let Some((_, min_dist)) = min {
                if dist < min_dist {
                    min = Some((entity, dist));
                }
            } else {
                min = Some((entity, dist));
            }
        }
    }

    if let Some(entity) = closest_parcel.0 {
        if let Ok(mut outline) = parcels.get_mut(entity) {
            outline.visible = false;
        }
    }

    if let Some((entity, _)) = min {
        if let Ok(mut outline) = parcels.get_mut(entity) {
            outline.visible = true;
        }
        closest_parcel.0 = Some(entity);
    } else {
        closest_parcel.0 = None;
    }
}

fn pop_parcel(
    mut events: EventWriter<PopParcelFromStack>,
    mouse: Res<Input<MouseButton>>,
    parcel_stack: Query<&ParcelStack>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        let stack = parcel_stack.single();

        if let Some(entry) = stack.parcels_entries.last() {
            events.send(PopParcelFromStack {
                parcel_entry: *entry,
                despawning: false,
            });
        }
    }
}

fn pickup_parcel(
    mut events: EventWriter<AddParcelToStack>,
    keyboard_input: Res<Input<KeyCode>>,
    mut closest_parcel: ResMut<ClosestParcel>,
    parcel_stack: Query<&ParcelStack>,
) {
    let stack = parcel_stack.single();
    if let Some(entity) = closest_parcel.0 {
        if keyboard_input.just_pressed(KeyCode::E) && stack.parcels_entries.len() < 3 {
            closest_parcel.0 = None;
            events.send(AddParcelToStack { parcel: entity });
        }
    }
}

fn remove_outline_from_picked(mut query: Query<&mut OutlineVolume, With<Picked>>) {
    for mut volume in query.iter_mut() {
        volume.visible = false;
    }
}

#[derive(Resource, Default, Debug, Clone)]
pub struct MousePosition(pub Option<Vec3>);

fn update_mouse_hover_pos(
    mut commands: Commands,
    windows: Res<Windows>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera>>,
    mut lines: ResMut<DebugLines>,
) {
    let (camera, camera_transform) = cameras.single();
    let (ray_pos, ray_dir) =
        ray_from_mouse_position(windows.get_primary().unwrap(), camera, camera_transform);
    let (plane_pos, plane_normal) = (Vec3::ZERO, Vec3::Y);
    let point = plane_intersection(ray_pos, ray_dir, plane_pos, plane_normal);

    if point.is_finite() {
        commands.insert_resource(MousePosition(Some(point)));
        lines.circle(point, 0.5, 0.0, Color::WHITE);
    } else {
        commands.insert_resource(MousePosition(None));
    }
}

/// Calculate the intersection point of a vector and a plane defined as a point and normal vector
/// where `pv` is the vector point, `dv` is the vector direction, `pp` is the plane point
/// and `np` is the planes' normal vector
pub fn plane_intersection(pv: Vec3, dv: Vec3, pp: Vec3, np: Vec3) -> Vec3 {
    let d = dv.dot(np);
    let t = (pp.dot(np) - pv.dot(np)) / d;
    pv + dv * t
}

/// Calculates origin and direction of a ray from cursor to world space.
pub fn ray_from_mouse_position(
    window: &Window,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> (Vec3, Vec3) {
    let mouse_position = window.cursor_position().unwrap_or(Vec2::new(0.0, 0.0));

    let x = 2.0 * (mouse_position.x / window.width() as f32) - 1.0;
    let y = 2.0 * (mouse_position.y / window.height() as f32) - 1.0;

    let camera_inverse_matrix =
        camera_transform.compute_matrix() * camera.projection_matrix().inverse();
    let near = camera_inverse_matrix * Vec3::new(x, y, -1.0).extend(1.0);
    let far = camera_inverse_matrix * Vec3::new(x, y, 1.0).extend(1.0);

    let near = near.truncate() / near.w;
    let far = far.truncate() / far.w;
    let dir: Vec3 = far - near;
    (near, dir)
}

fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut player: Query<(&mut Velocity, &Transform), (With<Player>, Without<PlayerGfx>)>,
    mut player_gfx: Query<&mut Transform, With<PlayerGfx>>,
) {
    let (mut vel, transform) = player.single_mut();
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

    // check if new position is in bounds of ground
    let new_pos = transform.translation + delta;
    if new_pos.x <= -config::GROUND_SIZE / 2.0 - 2.0
        || new_pos.x >= config::GROUND_SIZE / 2.0 + 2.0
        || new_pos.z <= -config::GROUND_SIZE / 2.0 - 2.0
        || new_pos.z >= config::GROUND_SIZE / 2.0 + 2.0
    {
        delta = Vec3::ZERO;
    }

    let mut gfx_transform = player_gfx.single_mut();

    if delta.x > 0.0 {
        gfx_transform.rotation = Quat::from_rotation_y(180.0_f32.to_radians());
    } else if delta.x < 0.0 {
        gfx_transform.rotation = Quat::from_rotation_y(0.0_f32.to_radians());
    }

    vel.linvel = delta.normalize_or_zero() * config::PLAYER_SPEED;
}
