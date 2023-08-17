use bevy::prelude::*;
use bevy_mod_scripting::prelude::*;
use bitflags::bitflags;

bitflags! {
	/// Enemy attribute flags
	#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq)]
	pub struct EnemyFlags: u16 {
		const BOSS = 1;

		/// Do not remove from the map after death
		const NO_DESPAWN = 2;
	}
}

/// Attack stat for battle entities
#[derive(Clone, Copy, Component, Debug, Reflect)]
#[reflect(Component, LuaProxyable)]
pub struct Attack {
	current: i16,
	normal: i16,
}

impl

/// Defense stat for battle entities
#[derive(Clone, Copy, Component, Debug, Reflect)]
#[reflect(Component, LuaProxyable)]
pub struct Defense {
	current: i16,
	normal: i16,
}

/// Strength stat for battle entities
#[derive(Clone, Copy, Component, Debug, Reflect)]
#[reflect(Component, LuaProxyable)]
pub struct Strength {
	current: i16,
	normal: i16,
}

/// Speed stat for battle entities
#[derive(Clone, Copy, Component, Debug, Reflect)]
#[reflect(Component, LuaProxyable)]
pub struct Speed {
	current: i16,
	normal: i16,
}

/// Accuracy stat for battle entities
#[derive(Clone, Copy, Component, Debug, Reflect)]
#[reflect(Component, LuaProxyable)]
pub struct Accuracy {
	current: i16,
	normal: i16,
}

/// Evasion stat for battle entities
#[derive(Clone, Copy, Component, Debug, Reflect)]
#[reflect(Component, LuaProxyable)]
pub struct Evasion {
	current: i16,
	normal: i16,
}

/// Magic stat for battle entities
#[derive(Clone, Copy, Component, Debug, Reflect)]
#[reflect(Component, LuaProxyable)]
pub struct Magic {
	current: i16,
	normal: i16,
}

/// Stamina stat for battle entities
#[derive(Clone, Copy, Component, Debug, Reflect)]
#[reflect(Component, LuaProxyable)]
pub struct Stamina {
	current: i16,
	normal: i16,
}

/// Magic defense stat for battle entities
#[derive(Clone, Copy, Component, Debug, Reflect)]
#[reflect(Component, LuaProxyable)]
pub struct MagicDefense {
	current: i16,
	normal: i16,
}

/// Hit points for battle entities
#[derive(Clone, Copy, Component, Debug, Reflect)]
#[reflect(Component, LuaProxyable)]
pub struct HitPoints {
	current: i16,
	max: i16,
}

/// Magic points for player entities
#[derive(Clone, Copy, Component, Debug, Reflect)]
#[reflect(Component, LuaProxyable)]
pub struct MagicPoints {
	current: i16,
	max: i16,
}

/// Enemy entity
#[derive(Clone, Copy, Component, Debug, Reflect)]
#[reflect(Component, LuaProxyable)]
pub struct Enemy {
	flags: EnemyFlags,
	name: String,
	tp: u16,
	xp: u32,
}

/// Battle entity
#[derive(Bundle)]
pub struct BattleUnit {
	hp: HitPoints,
	mp: MagicPoints,
	strength: Strength,
	spd: Speed,
	eva: Evasion,
	acc: Accuracy,
	def: Defense,
	mdef: MagicDefense,
	mag: Magic,
	atk: Attack,
	sta: Stamina,
}
