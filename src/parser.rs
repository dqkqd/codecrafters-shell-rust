#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Token {
    Whitespace,
    Value(String),
}

impl Token {
    pub fn to_string_no_whitespace(tokens: &[Token]) -> Vec<String> {
        tokens
            .split(|token| token == &Token::Whitespace)
            .map(|tokens| {
                let strings: Vec<String> = tokens.iter().map(String::from).collect();
                strings.join("")
            })
            .filter(|s| !s.is_empty())
            .collect()
    }
}

impl From<&Token> for String {
    fn from(token: &Token) -> String {
        match token {
            Token::Whitespace => " ".into(),
            Token::Value(v) => v.clone(),
        }
    }
}

pub struct Parser<'a> {
    input: &'a [u8],
    pos: usize,
}

impl Parser<'_> {
    pub fn new(s: &str) -> Parser {
        Parser {
            input: s.as_bytes(),
            pos: 0,
        }
    }

    fn peek(&mut self) -> Option<&u8> {
        self.input.get(self.pos)
    }

    fn next(&mut self) -> Option<u8> {
        let c = self.input.get(self.pos).cloned()?;
        self.pos += 1;
        Some(c)
    }

    fn prev(&mut self) {
        self.pos -= 1;
    }

    fn read_all<P>(&mut self, until: P) -> Option<Token>
    where
        P: Fn(&u8) -> bool,
    {
        let mut token = Vec::new();

        loop {
            match self.next() {
                Some(c) if until(&c) => {
                    self.prev();
                    break;
                }
                None => break,
                Some(c) => token.push(c),
            }
        }

        let token = String::from_utf8(token).ok()?;
        Some(Token::Value(token))
    }

    fn read_raw<P>(&mut self, until: P) -> Option<Token>
    where
        P: Fn(&u8) -> bool,
    {
        let mut token = Vec::new();

        loop {
            match self.next() {
                Some(c) if until(&c) => {
                    self.prev();
                    break;
                }
                None => break,
                Some(b'\\') => match self.next() {
                    Some(c) => token.push(c),
                    None => break,
                },
                Some(c) => token.push(c),
            }
        }

        let token = String::from_utf8(token).ok()?;
        Some(Token::Value(token))
    }

    fn read_in_double_quote<P>(&mut self, until: P) -> Option<Token>
    where
        P: Fn(&u8) -> bool,
    {
        let mut token = Vec::new();

        loop {
            match self.next() {
                Some(c) if until(&c) => {
                    self.prev();
                    break;
                }
                None => break,
                Some(b'\\') => match self.next() {
                    Some(c) if c == b'$' || c == b'`' || c == b'"' || c == b'\\' || c == b'\n' => {
                        token.push(c)
                    }
                    Some(c) => {
                        token.push(b'\\');
                        token.push(c);
                    }
                    None => break,
                },
                Some(c) => token.push(c),
            }
        }

        let token = String::from_utf8(token).ok()?;
        Some(Token::Value(token))
    }

    fn skip_whitespace(&mut self) {
        loop {
            match self.peek() {
                Some(c) if is_whitespace(c) => self.pos += 1,
                _ => break,
            }
        }
    }

    pub fn into_tokens(mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while let Some(c) = self.peek() {
            if is_whitespace(c) {
                tokens.push(Token::Whitespace);
                self.skip_whitespace();
            } else if c == &b'\'' {
                self.next().unwrap();
                if let Some(token) = self.read_all(|c| c == &b'\'') {
                    tokens.push(token);
                    // skip the last one
                    self.next().unwrap();
                }
            } else if c == &b'"' {
                self.next().unwrap();
                if let Some(token) = self.read_in_double_quote(|c| c == &b'"') {
                    tokens.push(token);
                    // skip the last one
                    self.next().unwrap();
                }
            } else if let Some(token) = self.read_raw(is_whitespace) {
                tokens.push(token);
            }
        }

        tokens
    }
}

fn is_whitespace(c: &u8) -> bool {
    c == &b' ' || c == &b'\t' || c == &b'\r' || c == &b'\n'
}

#[cfg(test)]
mod test {
    use super::*;

    fn tokens_from_str(s: &[&str]) -> Vec<Token> {
        s.iter()
            .map(|v| {
                if v == &" " {
                    Token::Whitespace
                } else {
                    Token::Value(v.to_string())
                }
            })
            .collect()
    }

    #[test]
    fn test_single() {
        let parser = Parser::new(r#"hello"#);
        let tokens = parser.into_tokens();

        assert_eq!(&tokens, &tokens_from_str(&["hello"]));
        assert_eq!(Token::to_string_no_whitespace(&tokens), ["hello"]);
    }

    #[test]
    fn test_no_quote() {
        let parser = Parser::new("hello world!!");
        let tokens = parser.into_tokens();
        assert_eq!(&tokens, &tokens_from_str(&["hello", " ", "world!!"]));
        assert_eq!(
            Token::to_string_no_whitespace(&tokens),
            ["hello", "world!!"]
        );
    }

    #[test]
    fn test_single_quote() {
        let parser = Parser::new("'hello'");
        let tokens = parser.into_tokens();
        assert_eq!(&tokens, &tokens_from_str(&["hello"]));
        assert_eq!(Token::to_string_no_whitespace(&tokens), ["hello"]);
    }

    #[test]
    fn test_double_quote() {
        let parser = Parser::new(r#""hello""#);
        let tokens = parser.into_tokens();
        assert_eq!(&tokens, &tokens_from_str(&["hello"]));
        assert_eq!(Token::to_string_no_whitespace(&tokens), ["hello"]);
    }

    #[test]
    fn test_quote() {
        let parser = Parser::new("'hello' 'world'");
        let tokens = parser.into_tokens();
        assert_eq!(&tokens, &tokens_from_str(&["hello", " ", "world"]));
        assert_eq!(Token::to_string_no_whitespace(&tokens), ["hello", "world"]);
    }

    #[test]
    fn test_mixed_single_quote() {
        let parser = Parser::new("hello 'world'");
        let tokens = parser.into_tokens();
        assert_eq!(&tokens, &tokens_from_str(&["hello", " ", "world"]));
        assert_eq!(Token::to_string_no_whitespace(&tokens), ["hello", "world"]);
    }

    #[test]
    fn test_mixed_double_quote() {
        let parser = Parser::new(r#""bar"  "shell's"  "foo""#);
        let tokens = parser.into_tokens();
        assert_eq!(
            &tokens,
            &tokens_from_str(&["bar", " ", "shell's", " ", "foo"])
        );
        assert_eq!(
            Token::to_string_no_whitespace(&tokens),
            ["bar", "shell's", "foo"]
        );
    }

    #[test]
    fn test_connected_single_quotes() {
        let parser = Parser::new("hello 'test''world'");
        let tokens = parser.into_tokens();
        assert_eq!(&tokens, &tokens_from_str(&["hello", " ", "test", "world"]));
        assert_eq!(
            Token::to_string_no_whitespace(&tokens),
            ["hello", "testworld"]
        );
    }

    #[test]
    fn test_connected_double_quotes() {
        let parser = Parser::new(r#""world  shell"  "hello""test""#);
        let tokens = parser.into_tokens();
        assert_eq!(
            &tokens,
            &tokens_from_str(&["world  shell", " ", "hello", "test"])
        );
        assert_eq!(
            Token::to_string_no_whitespace(&tokens),
            ["world  shell", "hellotest"]
        );
    }

    #[test]
    fn test_escape() {
        let parser = Parser::new(r#"world\ \ \ \ \ \ script"#);
        let tokens = parser.into_tokens();
        assert_eq!(&tokens, &tokens_from_str(&["world      script"]));
        assert_eq!(
            Token::to_string_no_whitespace(&tokens),
            ["world      script"]
        );
    }

    #[test]
    fn test_double_quote_with_escape() {
        let parser = Parser::new(r#""before\   after""#);
        let tokens = parser.into_tokens();
        assert_eq!(&tokens, &tokens_from_str(&[r#"before\   after"#]));
        assert_eq!(
            Token::to_string_no_whitespace(&tokens),
            [r#"before\   after"#]
        );
    }

    #[test]
    fn test_escape_in_double_quote() {
        let parser = Parser::new(r#""hello'script'\\n'world""#);
        let tokens = parser.into_tokens();
        assert_eq!(&tokens, &tokens_from_str(&[r#"hello'script'\n'world"#]));
        assert_eq!(
            Token::to_string_no_whitespace(&tokens),
            [r#"hello'script'\n'world"#]
        );
    }

    #[test]
    fn test_escape_in_double_quote_trailing() {
        let parser = Parser::new(r#""hello\"insidequotes"script\""#);
        let tokens = parser.into_tokens();
        assert_eq!(
            &tokens,
            &tokens_from_str(&[r#"hello"insidequotes"#, r#"script""#])
        );
        assert_eq!(
            Token::to_string_no_whitespace(&tokens),
            [r#"hello"insidequotesscript""#]
        );
    }
}
