use bytemuck::{
	bytes_of_mut,
	Pod,
	Zeroable
};

/// HP less than half
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
#[repr(C)]
struct HPLessThanHalf 

/// Check for status
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
#[repr(C)]
struct CheckForStatus {
	target: u8,
	offs: u8,
	check_bits: u8,
}

/// Check if something moved
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
#[repr(C)]
struct CheckIfMoved {
	target: u8,
	entity: u8,
	_2: u8,
}

/// Check status of entity
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
#[repr(C)]
struct CheckEntityStatus {
	_0: u8,
	entity: u8,
	is_dead: bool,
}

/// Checks for max number of living enemies
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
#[repr(C)]
struct CheckMaxLivingEntities {
	n: u8,
	_1: [u8; 2],
}