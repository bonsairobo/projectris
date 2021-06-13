use crate::{FallingPieceEvent, Rotation};

use bevy::prelude::*;
use building_blocks::core::PointN;

pub fn send_move_piece_events(
    keyboard: Res<Input<KeyCode>>,
    mut events: EventWriter<FallingPieceEvent>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        events.send(FallingPieceEvent::FastDrop);
        return;
    }

    if keyboard.pressed(KeyCode::Z) {
        if keyboard.just_pressed(KeyCode::Left) {
            events.send(FallingPieceEvent::Rotate(Rotation::rotate_z_neg_90()));
        } else if keyboard.just_pressed(KeyCode::Right) {
            events.send(FallingPieceEvent::Rotate(Rotation::rotate_z_pos_90()));
        }
    } else if keyboard.pressed(KeyCode::V) {
        if keyboard.just_pressed(KeyCode::Left) {
            events.send(FallingPieceEvent::Rotate(Rotation::rotate_x_neg_90()));
        } else if keyboard.just_pressed(KeyCode::Right) {
            events.send(FallingPieceEvent::Rotate(Rotation::rotate_x_pos_90()));
        }
    } else if keyboard.pressed(KeyCode::X) {
        if keyboard.just_pressed(KeyCode::Left) {
            events.send(FallingPieceEvent::Translate(PointN([1, 0, 0])));
        } else if keyboard.just_pressed(KeyCode::Right) {
            events.send(FallingPieceEvent::Translate(PointN([-1, 0, 0])));
        }
    } else if keyboard.pressed(KeyCode::C) {
        if keyboard.just_pressed(KeyCode::Left) {
            events.send(FallingPieceEvent::Translate(PointN([0, 0, -1])));
        } else if keyboard.just_pressed(KeyCode::Right) {
            events.send(FallingPieceEvent::Translate(PointN([0, 0, 1])));
        }
    }
}
