use winnow::{
    ascii::multispace0,
    combinator::{alt, delimited, opt, preceded, repeat},
    token::{any, take_till, take_until},
    ModalResult, Parser,
};

use super::{CommandToken, Stream};

pub(super) fn command_token(stream: &mut Stream) -> ModalResult<CommandToken> {
    preceded(
        multispace0,
        repeat(
            1..,
            alt((single_quote_stream, double_quote_stream, no_quote_stream)),
        )
        .fold(String::new, |acc, item| acc + &item)
        .map(CommandToken),
    )
    .parse_next(stream)
}

fn single_quote_stream(stream: &mut Stream) -> ModalResult<String> {
    delimited('\'', take_until(1.., "'").map(String::from), '\'').parse_next(stream)
}

fn double_quote_stream(stream: &mut Stream) -> ModalResult<String> {
    delimited(
        '"',
        repeat(0.., double_quote_inner_stream)
            .fold(String::new, |acc, item| acc + &item)
            .verify(|s: &str| !s.is_empty()),
        '"',
    )
    .parse_next(stream)
}

fn double_quote_inner_stream(stream: &mut Stream) -> ModalResult<String> {
    let token = take_till(0.., |c: char| "\"\\".contains(c)).map(String::from);
    let backslash = opt(alt(((preceded("\\", any)).map(|c| match c {
        '$' | '`' | '\"' | '\\' | '\n' => c.to_string(),
        c => format!("\\{c}"),
    }),)));

    (token, backslash)
        .map(|(token, s)| match s {
            Some(s) => token + &s,
            None => token,
        })
        .verify(|s: &str| !s.is_empty())
        .parse_next(stream)
}

fn no_quote_stream(stream: &mut Stream) -> ModalResult<String> {
    repeat(0.., no_quote_inner_stream)
        .fold(String::new, |acc, item| acc + &item)
        .verify(|s: &str| !s.is_empty())
        .parse_next(stream)
}

fn no_quote_inner_stream(stream: &mut Stream) -> ModalResult<String> {
    let token = take_till(0.., |c: char| " \t\r\n\\\'\"".contains(c)).map(String::from);
    let backslash = opt(preceded("\\", any));

    (token, backslash)
        .map(|(mut token, ch)| {
            if let Some(ch) = ch {
                token.push(ch);
            };
            token
        })
        .verify(|s: &str| !s.is_empty())
        .parse_next(stream)
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_single_quote() {
        assert_eq!(
            single_quote_stream.parse_next(&mut Stream::new("'hello'")),
            Ok("hello".to_string())
        );
        assert!(single_quote_stream
            .parse_next(&mut Stream::new("'hello"))
            .is_err());
    }

    #[test]
    fn test_double_quote() {
        assert_eq!(
            double_quote_stream.parse_next(&mut Stream::new("\"hello\"")),
            Ok("hello".to_string())
        );
        assert_eq!(
            double_quote_stream.parse_next(&mut Stream::new("\"hello\\$\"")),
            Ok("hello$".to_string())
        );
        assert_eq!(
            double_quote_stream.parse_next(&mut Stream::new("\"hello\\`\"")),
            Ok("hello`".to_string())
        );
        assert_eq!(
            double_quote_stream.parse_next(&mut Stream::new("\"hello\\\"\"")),
            Ok("hello\"".to_string())
        );
        assert_eq!(
            double_quote_stream.parse_next(&mut Stream::new("\"hello\\\\\"")),
            Ok("hello\\".to_string())
        );
        assert_eq!(
            double_quote_stream.parse_next(&mut Stream::new("\"hello\\\n\"")),
            Ok("hello\n".to_string())
        );
        assert!(double_quote_stream
            .parse_next(&mut Stream::new("\"hello"))
            .is_err());
        assert!(double_quote_stream
            .parse_next(&mut Stream::new("\"hello\\"))
            .is_err(),);
    }

    #[test]
    fn test_no_quote() {
        assert_eq!(
            no_quote_stream.parse_next(&mut Stream::new("hello ")),
            Ok("hello".to_string())
        );
        assert_eq!(
            no_quote_stream.parse_next(&mut Stream::new("hello\t")),
            Ok("hello".to_string())
        );
        assert_eq!(
            no_quote_stream.parse_next(&mut Stream::new("hello\r")),
            Ok("hello".to_string())
        );
        assert_eq!(
            no_quote_stream.parse_next(&mut Stream::new("hello\n")),
            Ok("hello".to_string())
        );
        assert_eq!(
            no_quote_stream.parse_next(&mut Stream::new("hello world")),
            Ok("hello".to_string())
        );
        assert_eq!(
            no_quote_stream.parse_next(&mut Stream::new("hello\\ world\n")),
            Ok("hello world".to_string())
        );
        assert!(no_quote_stream
            .parse_next(&mut Stream::new("hello"))
            .is_err(),);
    }

    #[test]
    fn test_command_arg() {
        assert_eq!(
            command_token(&mut Stream::new("hello\n")).unwrap(),
            CommandToken("hello".into())
        );
        assert_eq!(
            command_token(&mut Stream::new("hello world\n")).unwrap(),
            CommandToken("hello".into())
        );
        assert_eq!(
            command_token(&mut Stream::new("'hello world'\n")).unwrap(),
            CommandToken("hello world".into())
        );
        assert_eq!(
            command_token(&mut Stream::new("'hello' world\n")).unwrap(),
            CommandToken("hello".into())
        );
        assert_eq!(
            command_token(&mut Stream::new("hello'world'\n")).unwrap(),
            CommandToken("helloworld".into())
        );
        assert_eq!(
            command_token(&mut Stream::new("\"hello world\"\n")).unwrap(),
            CommandToken("hello world".into())
        );
        assert_eq!(
            command_token(&mut Stream::new("\"hello\" world\"\n")).unwrap(),
            CommandToken("hello".into())
        );
        assert_eq!(
            command_token(&mut Stream::new("\"hello\\\" world\"\n")).unwrap(),
            CommandToken("hello\" world".into())
        );
        assert_eq!(
            command_token(&mut Stream::new("\"hello\\$ world\"\n")).unwrap(),
            CommandToken("hello$ world".into())
        );
        assert_eq!(
            command_token(&mut Stream::new("\"hello\\` world\"\n")).unwrap(),
            CommandToken("hello` world".into())
        );
        assert_eq!(
            command_token(&mut Stream::new("\"hello\\\n world\"\n")).unwrap(),
            CommandToken("hello\n world".into())
        );
        assert_eq!(
            command_token(&mut Stream::new("\"hello\\x world\"\n")).unwrap(),
            CommandToken("hello\\x world".into())
        );
        assert_eq!(
            command_token(&mut Stream::new("\"hello\\$\"\n")).unwrap(),
            CommandToken("hello$".into())
        );
        assert_eq!(
            command_token(&mut Stream::new("hello\\ world\n")).unwrap(),
            CommandToken("hello world".into())
        );
        assert_eq!(
            command_token(&mut Stream::new("'hello\\\\world'\n")).unwrap(),
            CommandToken("hello\\\\world".into())
        );
        assert!(command_token(&mut Stream::new(" ")).is_err())
    }
}
