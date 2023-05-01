use std::{iter::Peekable, str::CharIndices};

pub struct Context<'a> {
    pub text: &'a str,
    pub iter: Peekable<CharIndices<'a>>,
    pub index: usize,
    pub err_msg: String,
}

impl Context<'_> {
    pub fn iter_next(&mut self) {
        if let Some((_, _c)) = self.iter.peek() {
            self.iter.next();
            self.index += 1;
        }
    }

    pub fn iter_next_n(&mut self, n: usize) {
        let remainder = self.text.len() - self.index;
        let n = n.min(remainder);
        if n > 0 {
            self.iter.nth(n - 1);
            self.index += n;
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    Number,
    InfixOperator,
    UnaryOperator,
    ParenthesisOpen,
    ParenthesisClose,
    Eof,
}

#[derive(Debug)]
pub enum ParseError {
    Unreachable,
    ParsedFailed,
}

pub type NextTokens = [Option<TokenType>; TokenType::Eof as usize];
pub type ParseResult = Result<NextTokens, ParseError>;

pub fn make_next_tokens(tokens: &[TokenType]) -> NextTokens {
    let mut result: NextTokens = Default::default();
    for (i, tok) in tokens.iter().enumerate() {
        result[i] = Some(tok.clone());
    }
    result
}

pub fn get_valid_tokens(tokens: &[Option<TokenType>]) -> &[Option<TokenType>] {
    let mut high: usize = 0;
    for (i, tok) in tokens.iter().enumerate() {
        if tok.is_none() {
            break;
        }
        high = i;
    }
    &tokens[0..=high]
}

pub struct Opteration {
    pub num: f32,
    pub op: char,
}

pub struct State {
    pub op_list: [Option<Opteration>; 3],
    pub unary: char,
    pub num: f32,
    pub op_prec: i8,
    pub paren_opened: bool,
}

impl State {
    pub fn calcuate_op_list_at(&mut self, prec: usize) {
        if let Some(op) = &mut self.op_list[prec] {
            self.num = match op.op {
                '+' => op.num + self.num,
                '-' => op.num - self.num,
                '*' => op.num * self.num,
                '/' => op.num / self.num,
                '^' => op.num.powf(self.num),
                _ => panic!("operator \"{}\" unimplemented!", op.op),
            };
            self.op_list[prec] = None;
        }
    }

    pub fn calcuate_op_list_all(&mut self) {
        for p in (0..=self.op_list.len() - 1).rev() {
            Self::calcuate_op_list_at(self, p);
        }
    }

    pub fn apply_unary_operator(&mut self) {
        match self.unary {
            '+' => {
                self.unary = 0 as char;
            }
            '-' => {
                self.num *= -1.;
                self.unary = 0 as char;
            }
            _ => {}
        }
    }
}

impl Context<'_> {
    pub fn parse_char(&mut self, _state: &mut State, ch: char) -> bool {
        if let Some((_, c)) = self.iter.peek() {
            if *c == ch {
                self.iter_next();
                return true;
            }
        }
        false
    }

    pub fn parse_char_repeat(&mut self, _state: &mut State, ch: char) -> bool {
        let mut parsed = false;
        while let Some((_, c)) = self.iter.peek() {
            if *c != ch {
                break;
            }
            parsed = true;
            self.iter_next();
        }
        parsed
    }

    pub fn parse_if(&mut self, _state: &mut State, func: fn(char) -> bool) -> bool {
        if let Some((_, c)) = self.iter.peek() {
            if func(*c) {
                self.iter_next();
                return true;
            }
        }
        false
    }

    pub fn parse_if_repeat(&mut self, _state: &mut State, func: fn(char) -> bool) -> bool {
        let mut parsed = false;
        while let Some((_, c)) = self.iter.peek() {
            if !func(*c) {
                break;
            }
            parsed = true;
            self.iter_next();
        }
        parsed
    }

    pub fn parse_one_of(
        &mut self,
        state: &mut State,
        parsers: &[fn(&mut Self, &mut State) -> ParseResult],
    ) -> ParseResult {
        for f in parsers {
            if let Ok(next_tokens) = f(self, state) {
                return Ok(next_tokens);
            }
        }

        Err(ParseError::ParsedFailed)
    }
}
