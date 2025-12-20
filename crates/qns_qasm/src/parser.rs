use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    character::complete::{alpha1, alphanumeric1, char, digit1, multispace1},
    combinator::{map, map_res, opt, recognize, value},
    multi::{many0, separated_list0},
    sequence::{delimited, pair, preceded, tuple},
    IResult,
};

use crate::ast::*;

// --- Whitespace & Comments ---

fn comment(input: &str) -> IResult<&str, ()> {
    value((), pair(tag("//"), take_while(|c| c != '\n')))(input)
}

fn sp(input: &str) -> IResult<&str, ()> {
    value((), many0(alt((value((), multispace1), comment))))(input)
}

// --- Identifiers & Numbers ---

fn identifier(input: &str) -> IResult<&str, String> {
    map(
        recognize(pair(
            alt((alpha1, tag("_"))),
            many0(alt((alphanumeric1, tag("_")))),
        )),
        |s: &str| s.to_string(),
    )(input)
}

fn usize_literal(input: &str) -> IResult<&str, usize> {
    map_res(digit1, |s: &str| s.parse::<usize>())(input)
}

fn float_literal(input: &str) -> IResult<&str, f64> {
    map_res(
        recognize(tuple((
            opt(char('-')),
            digit1,
            opt(pair(char('.'), digit1)),
            opt(pair(
                alt((char('e'), char('E'))),
                pair(opt(alt((char('+'), char('-')))), digit1),
            )),
        ))),
        |s: &str| s.parse::<f64>(),
    )(input)
}

// --- Arguments ---

fn argument(input: &str) -> IResult<&str, Argument> {
    alt((
        map(
            pair(identifier, delimited(char('['), usize_literal, char(']'))),
            |(name, idx)| Argument::Indexed(name, idx),
        ),
        map(identifier, Argument::Id),
    ))(input)
}

// --- Statements ---

fn version_decl(input: &str) -> IResult<&str, String> {
    delimited(
        tuple((tag("OPENQASM"), sp)),
        map(
            recognize(pair(digit1, pair(char('.'), digit1))),
            |s: &str| s.to_string(),
        ),
        tuple((sp, char(';'))),
    )(input)
}

fn qreg_decl(input: &str) -> IResult<&str, Statement> {
    map(
        tuple((
            tag("qreg"),
            sp,
            identifier,
            sp,
            delimited(char('['), usize_literal, char(']')),
            sp,
            char(';'),
        )),
        |(_, _, name, _, size, _, _)| Statement::QRegDecl { name, size },
    )(input)
}

fn creg_decl(input: &str) -> IResult<&str, Statement> {
    map(
        tuple((
            tag("creg"),
            sp,
            identifier,
            sp,
            delimited(char('['), usize_literal, char(']')),
            sp,
            char(';'),
        )),
        |(_, _, name, _, size, _, _)| Statement::CRegDecl { name, size },
    )(input)
}

fn gate_call(input: &str) -> IResult<&str, Statement> {
    map(
        tuple((
            identifier,
            sp,
            opt(delimited(
                char('('),
                separated_list0(tuple((sp, char(','), sp)), float_literal),
                char(')'),
            )),
            sp,
            separated_list0(tuple((sp, char(','), sp)), argument),
            sp,
            char(';'),
        )),
        |(name, _, params, _, args, _, _)| Statement::GateCall {
            name,
            params: params.unwrap_or_default(),
            args,
        },
    )(input)
}

fn measure_stmt(input: &str) -> IResult<&str, Statement> {
    map(
        tuple((
            tag("measure"),
            sp,
            argument,
            sp,
            tag("->"),
            sp,
            argument,
            sp,
            char(';'),
        )),
        |(_, _, qubit, _, _, _, target, _, _)| Statement::Measure { qubit, target },
    )(input)
}

fn reset_stmt(input: &str) -> IResult<&str, Statement> {
    map(
        tuple((tag("reset"), sp, argument, sp, char(';'))),
        |(_, _, qubit, _, _)| Statement::Reset { qubit },
    )(input)
}

fn barrier_stmt(input: &str) -> IResult<&str, Statement> {
    map(
        tuple((
            tag("barrier"),
            sp,
            separated_list0(tuple((sp, char(','), sp)), argument),
            sp,
            char(';'),
        )),
        |(_, _, args, _, _)| Statement::Barrier { args },
    )(input)
}

fn if_stmt(input: &str) -> IResult<&str, Statement> {
    map(
        tuple((
            tag("if"),
            sp,
            char('('),
            sp,
            identifier,
            sp,
            tag("=="),
            sp,
            usize_literal,
            sp,
            char(')'),
            sp,
            statement, // Recursive call
        )),
        |(_, _, _, _, condition, _, _, _, val, _, _, _, body)| Statement::If {
            condition,
            val,
            body: Box::new(body),
        },
    )(input)
}

// Include statement is handled by preprocessor, but we might want to parse it if it remains?
// For now, assume preprocessor handles it. But let's add a parser just in case.
fn include_stmt(input: &str) -> IResult<&str, Statement> {
    // We parse it but maybe ignore or return a placeholder?
    // Actually, if we use preprocessor, this shouldn't appear.
    // But for completeness:
    map(
        tuple((
            tag("include"),
            sp,
            char('"'),
            take_while(|c| c != '"'),
            char('"'),
            sp,
            char(';'),
        )),
        |(_, _, _, filename, _, _, _)| Statement::GateCall {
            name: "include".to_string(),
            params: vec![],
            args: vec![Argument::Id(filename.to_string())],
        }, // Hack: treat include as a gate call if not preprocessed
    )(input)
}

fn statement(input: &str) -> IResult<&str, Statement> {
    preceded(
        sp,
        alt((
            qreg_decl,
            creg_decl,
            measure_stmt,
            reset_stmt,
            barrier_stmt,
            if_stmt,
            include_stmt,
            gate_call, // Should be last as it matches generic identifiers
        )),
    )(input)
}

// --- Program ---

pub fn parse_qasm_str(input: &str) -> crate::Result<Program> {
    let (input, version) = opt(preceded(sp, version_decl))(input)
        .map_err(|e| crate::QasmError::ParseError(e.to_string()))?;

    let (input, statements) =
        many0(statement)(input).map_err(|e| crate::QasmError::ParseError(e.to_string()))?;

    // Check if there is remaining input (errors)
    let input = sp(input).map(|(i, _)| i).unwrap_or(input);
    if !input.is_empty() {
        return Err(crate::QasmError::ParseError(format!(
            "Unparsed input: {}",
            input
        )));
    }

    Ok(Program {
        version,
        statements,
    })
}
