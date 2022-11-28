use bevy::prelude::*;

use crate::{FontAssets, GameState, MainCamera, Score};

pub struct GameOverPlugin;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::GameOver).with_system(setup_menu))
            .add_system_set(SystemSet::on_exit(GameState::GameOver).with_system(cleanup_menu));
    }
}

#[derive(Component)]
pub struct Root;

fn setup_menu(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    score: Res<Score>,
    query: Query<Entity, With<MainCamera>>,
) {
    commands.spawn(Camera2dBundle::default());
    commands.entity(query.single()).despawn();

    commands
        .spawn(NodeBundle {
            style: Style {
                margin: UiRect::all(Val::Auto),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::ColumnReverse,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Root)
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text {
                    sections: vec![TextSection {
                        value: format!(" Score: {:?} ", score.score).to_string(),
                        style: TextStyle {
                            font: font_assets.montserrat.clone(),
                            font_size: 40.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                    }],
                    alignment: Default::default(),
                },
                transform: Transform::from_xyz(0.0, 100.0, 0.0),
                ..Default::default()
            });
        });
}

fn cleanup_menu(
    mut commands: Commands,
    root: Query<Entity, With<Root>>,
    cam: Query<Entity, With<Camera2d>>,
) {
    commands.entity(root.single()).despawn_recursive();
    commands.entity(cam.single()).despawn_recursive();
}
