use crate::{prelude::*, FontAssets, Score};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::Ready).with_system(setup_ui));
        app.add_system_set(SystemSet::on_update(GameState::Ready).with_system(update_ui));
    }
}

#[derive(Component)]
pub struct ScoreText;

fn setup_ui(mut commands: Commands, font_assets: Res<FontAssets>, score: Res<Score>) {
    commands.spawn((
        TextBundle {
            text: Text {
                sections: vec![TextSection {
                    value: format!(" Score: {:?} ", score.score).to_string(),
                    style: TextStyle {
                        font: font_assets.fira_sans.clone(),
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                }],
                alignment: Default::default(),
            },
            transform: Transform::from_xyz(0.0, 100.0, 0.0),
            ..Default::default()
        },
        ScoreText,
    ));
}

fn update_ui(score: Res<Score>, mut score_text: Query<&mut Text, With<ScoreText>>) {
    for mut text in &mut score_text {
        text.sections[0].value = format!(" Score: {:?} ", score.score);
    }
}
