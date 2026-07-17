use bevy::prelude::*;

use crate::SimClock;
use crate::ui::{Focus, ScreenPanel};

const PANEL_BACKGROUND: Color = Color::srgba(0.055, 0.05, 0.04, 0.92);
const BUTTON_IDLE: Color = Color::srgb(0.20, 0.19, 0.16);
const BUTTON_HOVERED: Color = Color::srgb(0.32, 0.29, 0.22);
const BUTTON_ACTIVE: Color = Color::srgb(0.82, 0.57, 0.16);

#[derive(Component)]
pub struct GameClockText;

#[derive(Component, Clone, Copy)]
pub(crate) enum TimeControl {
    Paused,
    Normal,
    Fast,
}

pub(crate) type TimeControlQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Interaction,
        &'static TimeControl,
        &'static mut BackgroundColor,
    ),
    (With<Button>, Without<ScreenPanel>),
>;

impl TimeControl {
    fn apply(self, clock: &mut SimClock) {
        match self {
            Self::Paused => clock.pause(),
            Self::Normal => clock.play_normal(),
            Self::Fast => clock.play_fast(),
        }
    }

    fn is_active(self, clock: &SimClock) -> bool {
        match self {
            Self::Paused => clock.is_paused(),
            Self::Normal => clock.speed_label() == "1x",
            Self::Fast => clock.speed_label() == "3x",
        }
    }
}

pub fn render(commands: &mut Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(24.0),
                bottom: Val::Px(24.0),
                height: Val::Px(66.0),
                padding: UiRect::all(Val::Px(9.0)),
                column_gap: Val::Px(8.0),
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(PANEL_BACKGROUND),
            Interaction::default(),
            ScreenPanel,
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new("DAY 1  08:00"),
                GameClockText,
                TextFont {
                    font_size: 23.0,
                    ..default()
                },
                TextColor(Color::srgb(0.98, 0.78, 0.30)),
                Node {
                    width: Val::Px(176.0),
                    margin: UiRect::right(Val::Px(6.0)),
                    ..default()
                },
            ));
            spawn_button(panel, TimeControl::Paused, "Pause");
            spawn_button(panel, TimeControl::Normal, "1x");
            spawn_button(panel, TimeControl::Fast, "3x");
        });
}

fn spawn_button(panel: &mut ChildSpawnerCommands, control: TimeControl, label: &str) {
    panel
        .spawn((
            Button,
            control,
            Node {
                width: Val::Px(68.0),
                height: Val::Px(42.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(BUTTON_IDLE),
        ))
        .with_child((
            Text::new(label),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgb(0.96, 0.94, 0.88)),
        ));
}

pub fn update(
    mut clock: ResMut<SimClock>,
    mut focus: ResMut<Focus>,
    mut clock_text: Query<&mut Text, With<GameClockText>>,
    panels: Query<&Interaction, (With<ScreenPanel>, Without<Button>)>,
    mut controls: TimeControlQuery,
) {
    let pointer_over_panel = panels
        .iter()
        .any(|interaction| *interaction != Interaction::None);
    let pointer_over_control = controls
        .iter()
        .any(|(interaction, _, _)| *interaction != Interaction::None);
    focus.pointer_over_ui = pointer_over_panel || pointer_over_control;

    for (interaction, control, _) in controls.iter() {
        if *interaction == Interaction::Pressed {
            control.apply(&mut clock);
        }
    }
    for (interaction, control, mut color) in controls.iter_mut() {
        color.0 = if control.is_active(&clock) {
            BUTTON_ACTIVE
        } else if *interaction == Interaction::Hovered {
            BUTTON_HOVERED
        } else {
            BUTTON_IDLE
        };
    }

    let (day, hour, minute) = clock.day_time();
    for mut text in clock_text.iter_mut() {
        *text = Text::new(format!("DAY {day}  {hour:02}:{minute:02}"));
    }
}
