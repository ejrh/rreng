use bevy::{
    ecs::system::EntityCommands,
    prelude::*,
    ui::{BackgroundColor, FlexDirection, PositionType},
    utils::default,
};

#[derive(Default)]
pub struct ToolbarPlugin {
}

impl Plugin for ToolbarPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<ToolbarButton>()
            .add_systems(Update, toolbar_interaction)
            .add_systems(Update, button_changed);
    }
}

#[derive(Component)]
pub struct Toolbar;

#[derive(Component)]
pub struct ToolbarLine;

#[derive(Component, Reflect)]
pub struct ToolbarButton {
    pub enabled: bool,
    pub hovered: bool,
    pub selected: bool,
}

pub fn create<'a>(commands: &'a mut Commands) -> EntityCommands<'a>  {
    let toolbar_id = commands.spawn((
        Toolbar,
        Node {
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            ..default()
        }))
        .id();

    commands.entity(toolbar_id)
}

pub fn create_line<'a>(commands: &'a mut Commands, toolbar_id: Entity) -> EntityCommands<'a> {
    let toolbar_line_id = commands
        .spawn((
            ToolbarLine,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            }))
        .set_parent(toolbar_id)
        .id();

    commands.entity(toolbar_line_id)
}

pub fn create_button<'a>(commands: &'a mut Commands, toolbar_line_id: Entity, enabled: bool) -> EntityCommands<'a> {
    let button_id = commands
        .spawn((
            ToolbarButton {
                enabled,
                hovered: false,
                selected: false,
            },
            Button,
            Node {
                position_type: PositionType::Relative,
                width: Val::Px(80.0),
                height: Val::Px(80.0),
                margin: UiRect::axes(Val::Px(10.0), Val::Px(5.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BorderColor(Color::WHITE),
            BorderRadius::all(Val::Px(10.0)),
            BackgroundColor(Color::BLACK),
        ))
        .set_parent(toolbar_line_id)
        .id();

    commands.entity(button_id)
}

pub fn create_label(button_font: Handle<Font>, label: &str) -> (Text, TextFont, TextColor) {
    (
        Text(label.to_owned()),
        TextFont {
            font: button_font.clone(),
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.9, 0.9))
    )
}

const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const SELECTED_BUTTON: Color = Color::srgb(0.5, 0.5, 0.1);
const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const DISABLED_BUTTON: Color = Color::srgb(0.1, 0.1, 0.1);

pub fn toolbar_interaction(
    mut interaction_query: Query<(&mut ToolbarButton, &Interaction), Changed<Interaction>>,
) {
    for (mut button, interaction) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
            }
            Interaction::Hovered => {
                button.hovered = true;
            }
            Interaction::None => {
                button.hovered = false;
            }
        }
    }
}

pub fn button_changed(
    mut query: Query<(Entity, &ToolbarButton, &mut BackgroundColor), Changed<ToolbarButton>>,
) {
    for (_entity, button, mut color) in &mut query {
        if !button.enabled {
            *color = DISABLED_BUTTON.into();
        } else if button.selected {
            *color = SELECTED_BUTTON.into();
        } else if button.hovered {
            *color = HOVERED_BUTTON.into();
        } else {
            *color = NORMAL_BUTTON.into();
        }
    }
}
