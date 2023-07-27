use nom::{
	branch::alt,
	bytes::complete::{
		tag,
		take_till,
		take_until
	},
	character::complete::{
		char,
		hex_digit1,
		line_ending,
		u8,
		u16
	},
	combinator::value,
	IResult,
	multi::many0,
	sequence::{
		delimited,
		pair,
		preceded,
		terminated
	}
};

use indexmap::IndexMap;

use crate::ws;

/// Token types that correspond to various variables
pub enum Token {
	AutoEnd,
	AutoPage,
	Ayla,
	Choice1(String),
	Choice2(String),
	Choice3(String),
	Choice4(String),
	Config,
	Crono,
	Dash,
	Epoch,
	Fire,
	Frog,
	Item,
	L,
	Light,
	LineBreak,
	Lucca,
	Magus,
	Marle,
	Menu,
	Name,
	Narrate,
	Number,
	Page,
	PartyCharacter1,
	PartyCharacter2,
	PartyCharacter3,
	R,
	Robo,
	Shadow,
	Sharp,
	Space(u8),
	Tech,
	Text(String),
	Wait(u8),
	Warp,
	Water,

	Unknown,
}

/// <C#> ... </C#>
fn choice(input: &str) -> IResult<&str, Token> {
	let (input, n) = preceded(tag("<C"), terminated(u8, char('>')))?;
	let (input, txt) = take_till(delimited(tag("</C"), u8, char('>')))?;

	match n {
		1 => Ok((input, Token::Choice1(txt.to_owned()))),
		2 => Ok((input, Token::Choice2(txt.to_owned()))),
		3 => Ok((input, Token::Choice3(txt.to_owned()))),
		4 => Ok((input, Token::Choice4(txt.to_owned()))),
		_ => Ok((input, Token::Unknown)),
	}
}

/// Parses a dialogue entry
fn entry(input: &str) -> IResult<&str, (u16, Vec<Token>)> {
	let (input, i) = ident?;
	let (input, txt) = preceded(char(','), take_till(line_ending))?;
	let (txt, toks) = token_split(txt)?;

	Ok((input, (i, toks)))
}

/// Parses a dialogue identifier (ie. XXX_001) and returns the array index
fn ident(input: &str) -> IResult<&str, u16> {
	let (input, _) = take_until("_")?;
	let (input, i) = u16?;

	Ok((input, i))
}

/// Parses an array of dialogue entries into an indexed map
pub fn ident_array(input: &str) -> IResult<&str, IndexMap<u16, Vec<Token>>> {
	let mut (input, entries) = many0(entry)?;
	let mut entmap = IndexMap::new();

	entries.iter().for_each(|(i, toks) if !toks.is_empty() {
		entmap.insert(i, toks)
	});
	
	Ok((entmap))
}

/// <PT#>
fn party_char(input: &str) -> IResult<&str, Token> {
	let (input, n) = delimited(alt((tag("<PT"), tag("<NAME_PT"))), u8, char('>'))?;

	match n {
		1 => Ok((input, Token::PartyCharacter1)),
		2 => Ok((input, Token::PartyCharacter2)),
		3 => Ok((input, Token::PartyCharacter3)),
		_ => Ok((input, Token::Unknown)),
	}
}

/// <S##>
fn space(input: &str) -> IResult<&str, Token> {
	let (input, n) = delimited(tag("<S"), u8, char('>'))?;

	Ok((input, Token::Space(n)))
}

/// Non-markup dialogue text
fn text(input: &str) -> IResult<&str, Token> {
	let (input, txt) = take_till(token)?;

	Ok((input, Token::Text(txt.to_owned())))
}

/// Any special token in text
fn token(input: &str) -> IResult<&str, Token> {
	alt((
		choice,
		party_char,
		space,
		wait,
		value(tag("<AUTO_END>"), Token::AutoEnd),
		value(tag("<AUTO_PAGE>"), Token::AutoPage),
		value(tag("<NAME_AYL>"), Token::Ayla),
		value(tag("<BTN_CONF>"), Token::Config),
		value(alt((tag("<NAME_CRO>"), tag("<NICK_CRO>"), tag("<NAME_CNO>"))), Token::Crono),
		value(tag("<BTN_DASH>"), Token::Dash),
		value(tag("<NAME_SIL>"), Token::Epoch),
		value(tag("<ICON_FIRE>"), Token::Fire),
		value(tag("<NAME_FRO>"), Token::Frog),
		value(tag("<NAME_ITM>"), Token::Item),
		value(tag("<BTN_L>"), Token::L),
		value(tag("<ICON_LIGHT>"), Token::Light),
		value(char('\\'), Token::LineBreak),
		value(tag("<NAME_LUC>"), Token::Lucca),
		value(tag("<NAME_MAG>"), Token::Magus),
		value(tag("<NAME_MAR>"), Token::Marle),
		value(tag("<BTN_MENU>"), Token::Menu),
		value(tag("<NAME>"), Token::Name),
		value(tag("<CT>"), Token::Narrate),
		value(tag("<NUMBER>"), Token::Number),
		value(tag("<PAGE>"), Token::Page),
		value(tag("<BTN_R>"), Token::R),
		value(tag("<NAME_ROB>"), Token::Robo),
		value(tag("<ICON_SHADOW>"), Token::Shadow),
		value(tag("<SHARP>"), Token::Sharp),
		value(tag("<NAME_TEC>"), Token::Tech),
		value(tag("<BTN_WARP>"), Token::Warp),
		value(tag("<ICON_WATER>"), Token::Water),
	))(input)
}

/// Splits parsed dialogue into tokens, gathering all into a Vec
fn token_split(input: &str) -> IResult<&str, Vec<Token>> {
	many0(alt((text, token)))(input)
}

/// <WAIT>##</WAIT>
fn wait(input: &str) -> IResult<&str, Token> {
	let (input, hex) = delimited(tag("<WAIT>"), hex_digit1, tag("</WAIT>"))?;
	let n = hex.parse::<u8>()?;

	Ok((input, Token::Wait(n)))
}
