use crate::{Config, FallingPieceEvent, Rotation};

use bevy::prelude::*;
use building_blocks::core::PointN;

pub fn send_move_piece_events(
    config: Res<Config>,
    keyboard: Res<Input<KeyCode>>,
    mut events: EventWriter<FallingPieceEvent>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        events.send(FallingPieceEvent::FastDrop);
        return;
    }

    if keyboard.pressed(config.left_rotate_modifier) {
        if keyboard.just_pressed(KeyCode::Left) {
            events.send(FallingPieceEvent::Rotate(Rotation::rotate_z_neg_90()));
        } else if keyboard.just_pressed(KeyCode::Right) {
            events.send(FallingPieceEvent::Rotate(Rotation::rotate_z_pos_90()));
        }
    } else if keyboard.pressed(config.right_rotate_modifier) {
        if keyboard.just_pressed(KeyCode::Left) {
            events.send(FallingPieceEvent::Rotate(Rotation::rotate_x_neg_90()));
        } else if keyboard.just_pressed(KeyCode::Right) {
            events.send(FallingPieceEvent::Rotate(Rotation::rotate_x_pos_90()));
        }
    } else if keyboard.pressed(config.left_translate_modifier) {
        if keyboard.just_pressed(KeyCode::Left) {
            events.send(FallingPieceEvent::Translate(PointN([1, 0, 0])));
        } else if keyboard.just_pressed(KeyCode::Right) {
            events.send(FallingPieceEvent::Translate(PointN([-1, 0, 0])));
        }
    } else if keyboard.pressed(config.right_translate_modifier) {
        if keyboard.just_pressed(KeyCode::Left) {
            events.send(FallingPieceEvent::Translate(PointN([0, 0, -1])));
        } else if keyboard.just_pressed(KeyCode::Right) {
            events.send(FallingPieceEvent::Translate(PointN([0, 0, 1])));
        }
    }
}
