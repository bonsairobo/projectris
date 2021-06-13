use projectris::{
    create_game, create_scene_assets, send_drop_piece_events, send_move_piece_events,
    synchronize_grid_materials, update_falling_piece, Config, FallingPieceEvent,
};

use bevy::prelude::*;
use smooth_bevy_cameras::LookTransformPlugin;

fn main() -> Result<(), ron::Error> {
    let config = Config::read_file("config.ron")?;

    App::build()
        .add_event::<FallingPieceEvent>()
        .insert_resource(config)
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(LookTransformPlugin)
        .add_startup_system(create_scene_assets.exclusive_system())
        .add_startup_system(create_game.system())
        .add_system(send_drop_piece_events.system())
        .add_system(send_move_piece_events.system())
        .add_system(update_falling_piece.system().label("falling_piece"))
        .add_system(synchronize_grid_materials.system().after("falling_piece"))
        .run();

    Ok(())
}
