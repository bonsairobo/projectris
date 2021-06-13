use crate::{create_grids, Config, DropTimer, FallingPieceEvent, GrabBag, SceneAssets};

use bevy::prelude::*;
use smooth_bevy_cameras::LookTransform;

pub fn create_game(
    config: Res<Config>,
    time: Res<Time>,
    scene_assets: Res<SceneAssets>,
    mut commands: Commands,
    mut piece_events: EventWriter<FallingPieceEvent>,
) {
    create_grids(&config, &scene_assets, &mut commands);
    create_camera(&config, &mut commands);

    commands.insert_resource(DropTimer::new(0.75, &time));
    commands.insert_resource(GrabBag::new(config.repeats_per_bag));

    piece_events.send(FallingPieceEvent::Spawn);
}

fn create_camera(config: &Config, commands: &mut Commands) -> Entity {
    commands
        .spawn_bundle(PerspectiveCameraBundle::default())
        .insert(LookTransform {
            eye: config.camera_position,
            target: config.camera_target,
        })
        .id()
}
