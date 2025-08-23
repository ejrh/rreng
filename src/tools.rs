use bevy::asset::AssetServer;
use bevy::prelude::*;
use crate::screens::Screen;
use crate::terrain;
use crate::ui::toolbar;
use crate::ui::toolbar::{Toolbar, ToolbarButton, ToolbarLine, ToolbarPlugin};

pub struct ToolsPlugin;

impl Plugin for ToolsPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<Tool>()
            .init_state::<TerraformTool>()
            .init_state::<TrackTool>()
            .add_plugins(ToolbarPlugin::default())
            .init_resource::<Tools>()
            .add_systems(Update, update_tool_buttons)
            .add_systems(Update, update_terraform_tool_buttons)
            .add_systems(Update, update_track_tool_buttons)
            .add_systems(Update, (
                terrain::edit::click_point.run_if(in_state(TerraformTool::Height)),
                terrain::edit::drag_point.run_if(in_state(TerraformTool::Level))
            ).run_if(in_state(Tool::Terraform)));
    }
}

#[derive(Resource)]
pub struct Tools {
    terraform_line_id: Entity,
    track_line_id: Entity,
}

impl Default for Tools {
    fn default() -> Self {
        Tools {
            terraform_line_id: Entity::PLACEHOLDER,
            track_line_id: Entity::PLACEHOLDER,
        }
    }
}


#[derive(Clone, Component, Copy, Debug, Default, Eq, Hash, PartialEq, States)]
enum Tool {
    #[default]
    Select,
    Terraform,
    Track,
    Train,
}

#[derive(Clone, Component, Copy, Debug, Default, Eq, Hash, PartialEq, States)]
enum TerraformTool {
    #[default]
    Height,
    Level,
    Flatten,
    Smooth,
}

#[derive(Clone, Component, Copy, Debug, Default, Eq, Hash, PartialEq, States)]
enum TrackTool {
    #[default]
    Create,
    Edit,
}

pub(crate) fn create_tools(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let button_font = asset_server.load("fonts/FiraMono-Medium.ttf");

    let toolbar_id= toolbar::create(&mut commands).id();
    commands.entity(toolbar_id).insert(StateScoped(Screen::Playing));

    let toolbar_line_id = toolbar::create_line(&mut commands, toolbar_id).id();

    for (tool, label, enabled) in [
        (Tool::Select, "Select", true),
        (Tool::Terraform, "Terra-\nform", true),
        (Tool::Track, "Track", true),
        (Tool::Train, "Train", false),
    ] {
        toolbar::create_button(&mut commands, toolbar_line_id, enabled)
            .insert(tool)
            .with_children(|p| {
                p.spawn(toolbar::create_label(button_font.clone(), label));
            });
    }
}

pub fn create_terraform_tools(
    mut tools: ResMut<Tools>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    toolbar_id: Single<Entity, With<Toolbar>>,
) {
    let button_font = asset_server.load("fonts/FiraMono-Medium.ttf");

    let toolbar_line_id = toolbar::create_line(&mut commands, *toolbar_id)
        .insert(Visibility::Hidden)
        .id();

    for (tool, label, enabled) in [
        (TerraformTool::Height, "Height", true),
        (TerraformTool::Level, "Level", true),
        (TerraformTool::Flatten, "Flatten", false),
        (TerraformTool::Smooth, "Smooth", false),
    ] {
        toolbar::create_button(&mut commands, toolbar_line_id, enabled)
            .insert(tool)
            .with_children(|p| {
                p.spawn(toolbar::create_label(button_font.clone(), label));
            });
    }

    tools.terraform_line_id = toolbar_line_id;
}


pub fn create_track_tools(
    mut tools: ResMut<Tools>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    toolbar_id: Single<Entity, With<Toolbar>>,
) {
    let button_font = asset_server.load("fonts/FiraMono-Medium.ttf");

    let toolbar_line_id = toolbar::create_line(&mut commands, *toolbar_id)
        .insert(Visibility::Hidden)
        .id();

    for (tool, label, enabled) in [
        (TrackTool::Create, "Create", true),
        (TrackTool::Edit, "Edit", true),
    ] {
        toolbar::create_button(&mut commands, toolbar_line_id, enabled)
            .insert(tool)
            .with_children(|p| {
                p.spawn(toolbar::create_label(button_font.clone(), label));
            });
    }

    tools.track_line_id = toolbar_line_id;
}

fn update_tool_buttons(
    tools: ResMut<Tools>,
    query: Query<(&Tool, &Interaction), Changed<Interaction>>,
    mut state: ResMut<NextState<Tool>>,
    mut toolbar_lines: Query<(Entity, &mut Visibility), With<ToolbarLine>>,
) {
    let Some(tool) = query.iter()
        .filter_map(|(tool, interaction)|
            if let Interaction::Pressed = interaction { Some(*tool) } else { None })
        .next()
    else { return };

    state.set(tool);
    info!("Tool: {tool:?}");

    for (line_id, mut vis) in toolbar_lines.iter_mut() {
        if line_id == tools.terraform_line_id {
            *vis = if matches!(tool, Tool::Terraform) { Visibility::Visible } else { Visibility::Hidden };
        }
        if line_id == tools.track_line_id {
            *vis = if matches!(tool, Tool::Track) { Visibility::Visible } else { Visibility::Hidden };
        }
    }
}

fn update_terraform_tool_buttons(
    mut query: Query<(&TerraformTool, &mut ToolbarButton, Ref<Interaction>), With<Button>>,
    mut state: ResMut<NextState<TerraformTool>>,
) {
    let Some(tool) = query.iter_mut()
        .filter_map(|(tool, mut button, interaction)| {
            if !interaction.is_changed() { return None; }
            if let Interaction::Pressed = *interaction {
                button.selected = true;
                Some(*tool)
            } else { None }
        })
        .next()
    else { return };

    for (tool2, mut button, _) in query.iter_mut() {
        if *tool2 != tool {
            button.selected = false;
        }
    }

    state.set(tool);
    info!("TerraformTool: {tool:?}");
}
fn update_track_tool_buttons(
    mut query: Query<(&TrackTool, &mut ToolbarButton, Ref<Interaction>), With<Button>>,
    mut state: ResMut<NextState<TrackTool>>,
) {
    let Some(tool) = query.iter_mut()
        .filter_map(|(tool, mut button, interaction)| {
            if !interaction.is_changed() { return None; }
            if let Interaction::Pressed = *interaction {
                button.selected = true;
                Some(*tool)
            } else { None }
        })
        .next()
    else { return };

    for (tool2, mut button, _) in query.iter_mut() {
        if *tool2 != tool {
            button.selected = false;
        }
    }

    state.set(tool);
    info!("TrackTool: {tool:?}");
}
