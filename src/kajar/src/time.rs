use bevy::prelude::*;
use std::time::Duration;

/// Game play time
#[derive(Resource)]
pub struct GameTime(Duration);
