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

    fn read_until<P>(&mut self, p: P) -> Option<Token>
    where
        P: Fn(&u8) -> bool,
    {
        let mut token = Vec::new();

        loop {
            match self.next() {
                Some(c) if p(&c) => {
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
                if let Some(token) = self.read_until(|c| c == &b'\'') {
                    tokens.push(token);
                    // skip the last one
                    self.next().unwrap();
                }
            } else if let Some(token) = self.read_until(is_whitespace) {
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
        let parser = Parser::new("hello");
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
    fn test_quote() {
        let parser = Parser::new("'hello' 'world'");
        let tokens = parser.into_tokens();
        assert_eq!(&tokens, &tokens_from_str(&["hello", " ", "world"]));
        assert_eq!(Token::to_string_no_whitespace(&tokens), ["hello", "world"]);
    }

    #[test]
    fn test_mixed() {
        let parser = Parser::new("hello 'world'");
        let tokens = parser.into_tokens();
        assert_eq!(&tokens, &tokens_from_str(&["hello", " ", "world"]));
        assert_eq!(Token::to_string_no_whitespace(&tokens), ["hello", "world"]);
    }

    #[test]
    fn test_connected_quotes() {
        let parser = Parser::new("hello 'test''world'");
        let tokens = parser.into_tokens();
        assert_eq!(&tokens, &tokens_from_str(&["hello", " ", "test", "world"]));
        assert_eq!(
            Token::to_string_no_whitespace(&tokens),
            ["hello", "testworld"]
        );
    }
}
