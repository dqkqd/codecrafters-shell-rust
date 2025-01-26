pub(super) const WHITESPACE: u8 = b' ';
pub(super) const SINGLE_QUOTE: u8 = b'\'';
pub(super) const BACKSLASH: u8 = b'\\';
pub(super) const DOUBLE_QUOTE: u8 = b'"';

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RawToken(pub Vec<u8>);

impl RawToken {
    pub fn into_inner(self) -> Vec<u8> {
        self.0
    }

    fn merge(self, other: RawToken) -> RawToken {
        let (RawToken(lhs), RawToken(rhs)) = (self, other);
        RawToken([lhs, rhs].concat())
    }
}

pub(super) struct RawTokenParser<'a> {
    input: &'a [u8],
    pos: usize,
}

impl RawTokenParser<'_> {
    pub fn new(s: &str) -> RawTokenParser {
        RawTokenParser {
            input: s.as_bytes(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<&u8> {
        self.input.get(self.pos)
    }

    fn next(&mut self) -> Option<u8> {
        let c = self.input.get(self.pos).cloned()?;
        self.pos += 1;
        Some(c)
    }

    fn skip_whitespace(&mut self) {
        loop {
            match self.peek() {
                Some(c) if is_whitespace(c) => self.pos += 1,
                _ => break,
            }
        }
    }

    fn expect(&mut self, c: &u8) -> Option<()> {
        if self.peek() == Some(c) {
            self.pos += 1;
            Some(())
        } else {
            None
        }
    }

    fn read_escape(&mut self) -> Option<RawToken> {
        let mut token = Vec::new();

        let mut read_byte = || -> Option<u8> {
            let c = self.next()?;
            if c == BACKSLASH {
                self.next()
            } else if is_whitespace(&c) {
                None
            } else {
                Some(c)
            }
        };

        while let Some(c) = read_byte() {
            token.push(c);
        }

        if token.is_empty() {
            None
        } else {
            Some(RawToken(token))
        }
    }

    fn read_single_quote(&mut self) -> Option<RawToken> {
        self.expect(&SINGLE_QUOTE)?;
        let mut token = Vec::new();

        while let Some(c) = self.next() {
            if c == SINGLE_QUOTE {
                break;
            }
            token.push(c);
        }
        Some(RawToken(token))
    }

    fn read_double_quote(&mut self) -> Option<RawToken> {
        self.expect(&DOUBLE_QUOTE)?;
        let mut token = Vec::new();

        loop {
            match self.next() {
                Some(DOUBLE_QUOTE) => break,
                Some(BACKSLASH) => match self.next() {
                    Some(c) if b"$`\"\\\n".contains(&c) => token.push(c),
                    Some(c) => token.extend_from_slice(&[BACKSLASH, c]),
                    None => break,
                },
                Some(c) => token.push(c),
                None => break,
            }
        }
        Some(RawToken(token))
    }

    pub fn parse(mut self) -> Vec<RawToken> {
        let mut tokens = Vec::new();

        let mut add_token = |ts: &mut Vec<RawToken>| {
            let ts = std::mem::take(ts);
            if !ts.is_empty() {
                let token = ts.into_iter().fold(RawToken(vec![]), |acc, t| acc.merge(t));
                tokens.push(token);
            }
        };

        let mut processing_tokens: Vec<RawToken> = Vec::new();

        loop {
            if self.peek().is_some_and(is_whitespace) {
                add_token(&mut processing_tokens);
                self.skip_whitespace();
            }

            if let Some(token) = self
                .read_double_quote()
                .or_else(|| self.read_single_quote())
            {
                processing_tokens.push(token);
            } else if let Some(token) = self.read_escape() {
                processing_tokens.push(token);
                add_token(&mut processing_tokens);
                self.skip_whitespace();
            } else {
                break;
            }
        }

        add_token(&mut processing_tokens);

        tokens
    }
}

fn is_whitespace(c: &u8) -> bool {
    c == &b' ' || c == &b'\t' || c == &b'\r' || c == &b'\n'
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! RT {
        ($s:literal) => {
            RawToken($s.into())
        };
    }

    #[test]
    fn test_single() {
        let parser = RawTokenParser::new(r#"hello"#);
        let tokens = parser.parse();
        assert_eq!(tokens, [RT!("hello")])
    }

    #[test]
    fn test_no_quote() {
        let parser = RawTokenParser::new("hello world!!");
        let tokens = parser.parse();
        assert_eq!(tokens, [RT!("hello"), RT!("world!!")]);
    }

    #[test]
    fn test_single_quote() {
        let parser = RawTokenParser::new("'hello'");
        let tokens = parser.parse();
        assert_eq!(tokens, [RT!("hello")]);
    }

    #[test]
    fn test_double_quote() {
        let parser = RawTokenParser::new(r#""hello""#);
        let tokens = parser.parse();
        assert_eq!(tokens, [RT!("hello")]);
    }

    #[test]
    fn test_quote() {
        let parser = RawTokenParser::new("'hello' 'world'");
        let tokens = parser.parse();
        assert_eq!(tokens, [RT!("hello"), RT!("world")]);
    }

    #[test]
    fn test_mixed_single_quote() {
        let parser = RawTokenParser::new("hello 'world'");
        let tokens = parser.parse();
        assert_eq!(tokens, [RT!("hello"), RT!("world")]);
    }

    #[test]
    fn test_mixed_double_quote() {
        let parser = RawTokenParser::new(r#""bar"  "shell's"  "foo""#);
        let tokens = parser.parse();
        assert_eq!(tokens, [RT!("bar"), RT!("shell's"), RT!("foo")]);
    }

    #[test]
    fn test_connected_single_quotes() {
        let parser = RawTokenParser::new("hello 'test''world'");
        let tokens = parser.parse();
        assert_eq!(tokens, [RT!("hello"), RT!("testworld")]);
    }

    #[test]
    fn test_connected_double_quotes() {
        let parser = RawTokenParser::new(r#""world  shell"  "hello""test""#);
        let tokens = parser.parse();
        assert_eq!(tokens, [RT!("world  shell"), RT!("hellotest")]);
    }

    #[test]
    fn test_escape() {
        let parser = RawTokenParser::new(r#"world\ \ \ \ \ \ script"#);
        let tokens = parser.parse();
        assert_eq!(tokens, [RT!("world      script")]);
    }

    #[test]
    fn test_double_quote_with_escape() {
        let parser = RawTokenParser::new(r#""before\   after""#);
        let tokens = parser.parse();
        assert_eq!(tokens, [RT!(r#"before\   after"#)]);
    }

    #[test]
    fn test_escape_in_double_quote() {
        let parser = RawTokenParser::new(r#""hello'script'\\n'world""#);
        let tokens = parser.parse();
        assert_eq!(tokens, [RT!(r#"hello'script'\n'world"#)]);
    }

    #[test]
    fn test_escape_in_double_quote_trailing() {
        let parser = RawTokenParser::new(r#""hello\"insidequotes"script\""#);
        let tokens = parser.parse();
        assert_eq!(tokens, [RT!(r#"hello"insidequotesscript""#)]);
    }
}
