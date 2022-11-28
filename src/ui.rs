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
    commands.spawn((
        TextBundle::from_section(
            format!("{:.1$}", score.score, 0),
            TextStyle {
                font: font_assets.montserrat.clone(),
                font_size: 60.0,
                color: Color::rgb(1., 1., 1.),
            },
        )
        .with_text_alignment(TextAlignment::CENTER)
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                top: Val::Px(5.0),
                left: Val::Percent(48.0),
                ..default()
            },
            max_size: Size {
                width: Val::Px(400.),
                height: Val::Undefined,
            },
            ..default()
        }),
        GameTime,
    ));

    commands.spawn((
        TextBundle::from_section(
            format!("{:.1$}", score.score, 0),
            TextStyle {
                font: font_assets.montserrat.clone(),
                font_size: 60.0,
                color: Color::rgb(1., 1., 1.),
            },
        )
        .with_text_alignment(TextAlignment::CENTER)
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                bottom: Val::Px(5.0),
                left: Val::Percent(46.0),
                ..default()
            },
            max_size: Size {
                width: Val::Px(400.),
                height: Val::Undefined,
            },
            ..default()
        }),
        ScoreText,
    ));
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
        text.sections[0].value = format!("{:.1$}", time_remaining.timer.remaining_secs(), 0);
    }
}
