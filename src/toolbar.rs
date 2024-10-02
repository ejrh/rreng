use bevy::{
    asset::AssetServer,
    ecs::system::EntityCommands,
    prelude::*,
    text::TextStyle,
    ui::{AlignContent, AlignSelf, BackgroundColor, FlexDirection, PositionType},
    utils::default,
};
use bevy::prelude::{};

#[derive(Component)]
pub struct Toolbar {
    button_font: Handle<Font>,
}

#[derive(Component)]
pub struct ToolbarButton;

impl Toolbar {
    fn add_button(&self, name: &str, commands: &mut ChildBuilder) {
        info!("add button {name}");
        commands
            .spawn(ToolbarButton)
            .insert(ButtonBundle {
                style: Style {
                    position_type: PositionType::Relative,
                    width: Val::Px(80.0),
                    height: Val::Px(80.0),
                    margin: UiRect::all(Val::Px(10.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                border_color: BorderColor(Color::WHITE),
                border_radius: BorderRadius::all(Val::Px(10.0)),
                background_color: BackgroundColor(Color::BLACK),
                ..default()
            })
            .with_children(|p| {
                p.spawn(TextBundle::from_section(name, TextStyle {
                    font: self.button_font.clone(),
                    font_size: 20.0,
                    color: Color::srgb(0.9, 0.9, 0.9),
                    ..default()
                }));
            });
    }
}

pub fn create_toolbar(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let button_font = asset_server.load("fonts/FiraMono-Medium.ttf");

    commands.spawn(Toolbar {
        button_font
    }).insert(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            // align_items: AlignItems::Center,
            // align_self: AlignSelf::Start,
            // justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        ..default()
    });
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);

pub fn toolbar_interaction(
    mut interaction_query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                // *color = PRESSED_BUTTON.into();
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }

}

pub fn update_toolbar(
    toolbar: Query<&Toolbar>,
    toolbar_buttons: Query<(&ToolbarButton, )>,
    commands: Commands,
) {

}

pub fn create_buttons(
    toolbar_query: Query<(Entity, &Toolbar)>,
    mut commands: Commands,
) {
    let (toolbar_id, toolbar) = toolbar_query.single();
    commands.entity(toolbar_id).with_children(|p| {
        toolbar.add_button("Select", p);
        toolbar.add_button("Terraform", p);
        toolbar.add_button("Track", p);
        toolbar.add_button("Train", p);
    });
}
