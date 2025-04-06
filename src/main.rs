use bevy::prelude::*;
use projectris::{
    create_game, create_scene_assets, send_drop_piece_events, send_move_piece_events,
    synchronize_grid_materials, update_falling_piece, Config, FallingPieceEvent,
};

fn main() -> Result<(), ron::Error> {
    let default_plugins = DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Projectris".into(),
            name: Some("projectris.app".into()),
            resolution: (800., 600.).into(),
            // Tells Wasm to resize the window according to the available canvas
            fit_canvas_to_parent: true,
            // Tells Wasm not to override default event handling, like F5, Ctrl+R etc.
            prevent_default_event_handling: false,
            ..default()
        }),
        ..default()
    });

    App::new()
        .add_event::<FallingPieceEvent>()
        .insert_resource(Config::read_file("config.ron")?)
        .add_plugins(default_plugins)
        .add_systems(Startup, create_scene_assets)
        .add_systems(Startup, create_game.after(create_scene_assets))
        .add_systems(Update, send_drop_piece_events)
        .add_systems(Update, send_move_piece_events)
        .add_systems(Update, update_falling_piece)
        .add_systems(
            Update,
            synchronize_grid_materials.after(update_falling_piece),
        )
        .run();

    Ok(())
}
