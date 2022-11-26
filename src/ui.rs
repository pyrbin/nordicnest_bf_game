use crate::{prelude::*, FontAssets, Score, TimeRemaining};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::Ready).with_system(setup_ui));
        app.add_system_set(SystemSet::on_update(GameState::Ready).with_system(update_ui));
    }
}

#[derive(Component)]
pub struct ScoreText;

#[derive(Component)]
pub struct GameTime;

fn setup_ui(mut commands: Commands, font_assets: Res<FontAssets>, score: Res<Score>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                size: Size {
                    width: Val::Percent(100.),
                    height: Val::Percent(30.),
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn((
                TextBundle {
                    text: Text {
                        sections: vec![TextSection {
                            value: format!("Remaining: {:.1$}", score.score, 0).to_string(),
                            style: TextStyle {
                                font: font_assets.fira_sans.clone(),
                                font_size: 20.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                            },
                        }],
                        alignment: Default::default(),
                    },
                    style: Style {
                        flex_grow: 1.0,
                        ..default()
                    },
                    ..Default::default()
                },
                GameTime,
            ));

            parent.spawn((
                TextBundle {
                    text: Text {
                        sections: vec![TextSection {
                            value: format!("Score: {:?}", score.score).to_string(),
                            style: TextStyle {
                                font: font_assets.fira_sans.clone(),
                                font_size: 20.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                            },
                        }],
                        alignment: Default::default(),
                    },
                    style: Style {
                        flex_grow: 1.0,
                        ..default()
                    },
                    ..Default::default()
                },
                ScoreText,
            ));
        });
}

fn update_ui(
    score: Res<Score>,
    time_remaining: Res<TimeRemaining>,
    mut score_text: Query<&mut Text, (With<ScoreText>, Without<GameTime>)>,
    mut game_text: Query<&mut Text, (With<GameTime>, Without<ScoreText>)>,
) {
    for mut text in &mut score_text {
        text.sections[0].value = format!("Score: {:?}", score.score);
    }

    for mut text in &mut game_text {
        text.sections[0].value = format!(
            "Remaining: {:.1$}",
            time_remaining.timer.remaining_secs(),
            0
        );
    }
}
