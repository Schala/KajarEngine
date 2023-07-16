use nom::{
	branch::alt,
	bytes::complete::{
		tag,
		take_till
	},
	character::complete::{
		char,
		line_ending
	},
	combinator::value,
	IResult,
	number::complete::{
		hex_u32,
		u8
	},
	sequence::{
		delimited,
		pair,
		preceded,
		terminated
	}
};

/// Token types that correspond to various variables
pub enum Token {
	AutoEnd,
	AutoPage,
	Ayla,
	Choice1(String),
	Choice2(String),
	Choice3(String),
	ConfigButton,
	Crono,
	DashButton,
	Epoch,
	Fire,
	Frog,
	Item,
	LButton,
	Light,
	LineBreak,
	Lucca,
	Magus,
	Marle,
	MenuButton,
	Name,
	Narrate,
	Number,
	Page,
	PartyCharacter1,
	PartyCharacter2,
	PartyCharacter3,
	RButton,
	Shadow,
	Sharp,
	Space(u8),
	Tech,
	Text(String),
	Wait(u8),
	WarpButton,
	Water,

	Unknown,
}

/// <C#> ... </C#>
fn choice(input: &str) -> IResult<&str, Token> {
	let (input, n) = preceded(tag("<C"), terminated(u8, char('>')))?;
	let (input, txt) = take_till(pair(tag("</C"), pair(u8, char('>'))))?;

	match n {
		1 => Ok((input, Token::Choice1(txt.to_owned()))),
		2 => Ok((input, Token::Choice2(txt.to_owned()))),
		3 => Ok((input, Token::Choice3(txt.to_owned()))),
		_ => Ok((input, Token::Unknown)),
	}
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

fn token(input: &str) -> IResult<&str, Token> {
	alt((
		choice,
		party_char,
		wait,
		value(tag("<AUTO_END>"), Token::AutoEnd),
		value(tag("<AUTO_PAGE>"), Token::AutoPage),
		value(tag("<NAME_AYL>"), Token::Ayla),
		value(tag("<BTN_CONF>"), Token::ConfigButton),
		value(alt((tag("<NAME_CRO>"), tag("<NICK_CRO>"), tag("<NAME_CNO>"))), Token::Crono),
		value(tag("<BTN_DASH>"), Token::DashButton),
		value(tag("<NAME_SIL>"), Token::Epoch),
		value(tag("<ICON_FIRE>"), Token::Fire),
		value(tag("<NAME_FRO>"), Token::Frog),
		value(tag("<NAME_ITM>"), Token::Item),
		value(tag("<BTN_L>"), Token::LButton),
		value(tag("<ICON_LIGHT>"), Token::Light),
		value(char('\\'), Token::LineBreak),
		value(tag("<NAME_LUC>"), Token::Lucca),
		value(tag("<NAME_MAG>"), Token::Magus),
		value(tag("<NAME_MAR>"), Token::Marle),
		value(tag("<BTN_MENU>"), Token::MenuButton),
		value(tag("<NAME>"), Token::Name),
		value(tag("<CT>"), Token::Narrate),
		value(tag("<NUMBER>"), Token::Number),
		value(tag("<PAGE>"), Token::Page),
		value(tag("<BTN_R>"), Token::RButton),
		value(tag("<ICON_SHADOW>"), Token::Shadow),
		value(tag("<SHARP>"), Token::Sharp),
		value(tag("<NAME_TEC>"), Token::Tech),
		value(tag("<BTN_WARP>"), Token::WarpButton),
		value(tag("<ICON_WATER>"), Token::Water),
	))(input)
}

/// <WAIT>##</WAIT>
fn wait(input: &str) -> IResult<&str, Token> {
	let (input, n) = delimited(tag("<WAIT>"), hex_u32, tag("</WAIT>"))?;

	Ok((input, Token::Wait(n as u8)))
}
