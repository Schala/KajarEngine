use bevy::prelude::*;

/// Playable character experience points
#[derive(Component)]
pub struct Experience {
	current: i32,
	next: i32,
}

/// Player gold
#[derive(Resource)]
pub struct Gold(u32);

/// Player silver points for Millennial Faire
#[derive(Resource)]
pub struct SilverPoints(u16);

/// Player character talent points
#[derive(Component)]
pub struct TalentPoints(u16);

/// Playable character weapon
#[derive(Component)]
pub struct Weapon {
	class: u8,
	hp: i16,
	mp: i16,
	strength: i16,
	spd: i16,
	eva: i16,
	acc: i16,
	def: i16,
	mdef: i16,
	mag: i16,
	atk: i16,
	sta: i16,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialHash)]
#[repr(u8)]
pub enum ArmorClass {
	Male = 0,
	Female,

	#[default]
	Unisex,
}

/// Playable character armor
#[derive(Component)]
pub struct Armor {
	class: ArmorClass,
	hp: i16,
	mp: i16,
	strength: i16,
	spd: i16,
	eva: i16,
	acc: i16,
	def: i16,
	mdef: i16,
	mag: i16,
	atk: i16,
	sta: i16,
}

/// Playable character
#[derive(Bundle)]
pub struct PlayerCharacter {
	tp: TalentPoints,
	xp: Experience,
}
