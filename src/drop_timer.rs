use crate::FallingPieceEvent;
use bevy::prelude::*;

// TODO: get faster as the player's score gets higher

#[derive(Resource)]
pub struct DropTimer {
    last_drop_time: f64,
    time_between_drops: f64,
}

impl DropTimer {
    pub fn new(time_between_drops: f64, time: &Time) -> Self {
        Self {
            last_drop_time: time.elapsed_secs_f64(),
            time_between_drops,
        }
    }

    pub fn should_drop(&mut self, time: &Time) -> bool {
        let now = time.elapsed_secs_f64();
        if now - self.last_drop_time > self.time_between_drops {
            self.last_drop_time = now;

            true
        } else {
            false
        }
    }
}

pub fn send_drop_piece_events(
    mut movement_events: EventWriter<FallingPieceEvent>,
    time: Res<Time>,
    mut drop_timer: ResMut<DropTimer>,
) {
    if drop_timer.should_drop(&time) {
        movement_events.send(FallingPieceEvent::Drop);
    }
}
