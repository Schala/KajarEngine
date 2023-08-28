use nom::{
    branch::alt,
    bytes::complete::{tag, take_till, take_until},
    character::complete::{char, hex_digit1, u16, u8},
    combinator::value,
    multi::many0,
    sequence::{delimited, pair, preceded},
    IResult,
};

use indexmap::IndexMap;

/// Token types that correspond to various variables
#[derive(Clone, Debug)]
pub(crate) enum Token {
    AccessoryIcon,
    ArmorIcon,
    AutoEnd,
    AutoPage,
    AylaIcon,
    AylaName,
    Choice1(String),
    Choice2(String),
    Choice3(String),
    Choice4(String),
    Config,
    CronoIcon,
    CronoName,
    Dash,
    EpochName,
    FireIcon,
    FrogIcon,
    FrogName,
    HelmIcon,
    ItemIcon,
    ItemName,
    L,
    LightIcon,
    LineBreak,
    LuccaIcon,
    LuccaName,
    MagusIcon,
    MagusName,
    MarleIcon,
    MarleName,
    Menu,
    Name,
    Narrate,
    NonIcon,
    Number,
    Page,
    PartyCharacter1,
    PartyCharacter2,
    PartyCharacter3,
    R,
    RoboIcon,
    RoboName,
    ShadowIcon,
    Sharp,
    Space(u8),
    TechName,
    Text(String),
    Wait(u8),
    Warp,
    WaterIcon,
}

/// <C#> ... </C#>
fn choice(input: &str) -> IResult<&str, Token> {
    let (input, n) = delimited(tag("<C"), alt((u8, char('E'))), char('>'))(input)?;
    let (input, txt) = take_until("</C")(input)?;
    let (input, _) = pair(alt((u8, char('E'))), char('>'))(input)?; // nyomp!

    match n {
        1 => Ok((input, Token::Choice1(txt.to_owned()))),
        2 => Ok((input, Token::Choice2(txt.to_owned()))),
        3 => Ok((input, Token::Choice3(txt.to_owned()))),
        4 => Ok((input, Token::Choice4(txt.to_owned()))),
        _ => unreachable!(),
    }
}

/// Parses a dialogue entry
fn entry(input: &str) -> IResult<&str, (u16, Vec<Token>)> {
    let (input, i) = ident(input)?;
    let (input, txt) = preceded(char(','), take_till(|c| c == '\n' || c == '\r'))(input)?;
    let (txt, toks) = token_split(txt)?;

    Ok((input, (i, toks)))
}

/// Parses a dialogue identifier (ie. XXX_001) and returns the array index
fn ident(input: &str) -> IResult<&str, u16> {
    let (input, _) = take_until("_")(input)?;
    let (input, i) = u16(input)?;

    Ok((input, i))
}

/// Parses an array of dialogue entries into an indexed map
pub(crate) fn ident_array(input: &str) -> IResult<&str, IndexMap<u16, Vec<Token>>> {
    let (input, entries) = many0(entry)(input)?;
    let mut entmap = IndexMap::new();

    // todo: drain_filter when stabilised
    entries.iter_mut().enumerate().for_each(|(i, e)| {
        if !e.1.is_empty() {
            entmap.insert(e.0, e.1);
        }
    });

    Ok((input, entmap))
}

/// <PT#>
fn party_char(input: &str) -> IResult<&str, Token> {
    let (input, n) = delimited(alt((tag("<PT"), tag("<NAME_PT"))), u8, char('>'))(input)?;

    match n {
        1 => Ok((input, Token::PartyCharacter1)),
        2 => Ok((input, Token::PartyCharacter2)),
        3 => Ok((input, Token::PartyCharacter3)),
        _ => unreachable!(),
    }
}

/// <S##>
fn space(input: &str) -> IResult<&str, Token> {
    let (input, n) = delimited(tag("<S"), u8, char('>'))(input)?;

    Ok((input, Token::Space(n)))
}

/// Non-markup dialogue text
fn text(input: &str) -> IResult<&str, Token> {
    let (input, txt) = take_till(|c| c == '\\' || c == '\r' || c == '\n' || c == '<')(input)?;

    Ok((input, Token::Text(txt.to_owned())))
}

/// Any special token in text
fn token(input: &str) -> IResult<&str, Token> {
    alt((
        choice,
        party_char,
        space,
        wait,
        value(Token::AccessoryIcon, tag("<ICON_ACCE>")),
        value(Token::ArmorIcon, tag("<ICON_ARMO>")),
        value(Token::AutoEnd, tag("<AUTO_END>")),
        value(Token::AutoPage, tag("<AUTO_PAGE>")),
        value(Token::AylaIcon, tag("<ICON_AYL>")),
        value(Token::AylaName, tag("<NAME_AYL>")),
        value(Token::Config, tag("<BTN_CONF>")),
        value(Token::CronoIcon, tag("<ICON_CRO>")),
        value(
            Token::CronoName,
            alt((tag("<NAME_CRO>"), tag("<NICK_CRO>"), tag("<NAME_CNO>"))),
        ),
        value(Token::Dash, tag("<BTN_DASH>")),
        value(Token::EpochName, tag("<NAME_SIL>")),
        value(Token::FireIcon, tag("<ICON_FIRE>")),
        value(Token::FrogName, tag("<NAME_FRO>")),
        value(Token::HelmIcon, tag("<ICON_HELM>")),
        value(Token::ItemIcon, tag("<ICON_ITEM>")),
        value(Token::ItemName, tag("<NAME_ITM>")),
        value(Token::L, tag("<BTN_L>")),
        value(Token::LightIcon, tag("<ICON_LIGHT>")),
        value(Token::LineBreak, char('\\')),
        value(Token::LuccaIcon, tag("<ICON_LUC>")),
        value(Token::LuccaName, tag("<NAME_LUC>")),
        value(Token::MagusIcon, tag("<ICON_MAG>")),
        value(Token::MagusName, tag("<NAME_MAG>")),
        value(Token::MarleIcon, tag("<ICON_MAR>")),
        value(Token::MarleName, tag("<NAME_MAR>")),
        value(Token::Menu, tag("<BTN_MENU>")),
        value(Token::Name, tag("<NAME>")),
        value(Token::Narrate, tag("<CT>")),
        value(Token::NonIcon, tag("<NON_ICON>")),
        value(Token::Number, tag("<NUMBER>")),
        value(Token::Page, tag("<PAGE>")),
        value(Token::R, tag("<BTN_R>")),
        value(Token::RoboName, tag("<NAME_ROB>")),
        value(Token::ShadowIcon, tag("<ICON_SHADOW>")),
        value(Token::Sharp, tag("<SHARP>")),
        value(Token::TechName, tag("<NAME_TEC>")),
        value(Token::Warp, tag("<BTN_WARP>")),
        value(Token::WaterIcon, tag("<ICON_WATER>")),
    ))(input)
}

/// Splits parsed dialogue into tokens, gathering all into a Vec
fn token_split(input: &str) -> IResult<&str, Vec<Token>> {
    many0(alt((text, token)))(input)
}

/// <WAIT>##</WAIT>
fn wait(input: &str) -> IResult<&str, Token> {
    let (input, hex) = delimited(tag("<WAIT>"), hex_digit1, tag("</WAIT>"))(input)?;
    let n = hex.parse::<u8>()?;

    Ok((input, Token::Wait(n)))
}

#[cfg(test)]
mod test {
    #[test]
    fn test_dlg_parse() {
        let demo = "DEMO_01,<NAME_MAR>: My <NAME_ITM> brings all\
the <NAME_CNO>s to the<SP5>yard ";
        let out = super::ident_array(&demo).unwrap();
        println!("{:?}", out);
    }
}
