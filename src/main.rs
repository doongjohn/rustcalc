// TODO: support builtin function (sqrt, abs, floor, ceil, round, ...)
// TODO: support imaginary number
// TODO: support custom variable (declaration, assignment)
// TODO: support custom function (infix, unary)

// TODO: refactor parse_*_repeat

use std::{
    f32::consts::{E, PI, TAU},
    io::stdin,
    iter::Peekable,
    str::{CharIndices, FromStr},
};

struct Context<'a> {
    text: &'a str,
    iter: Peekable<CharIndices<'a>>,
    index: usize,
}

impl Context<'_> {
    fn iter_next(&mut self) {
        if let Some((_, _c)) = self.iter.peek() {
            self.iter.next();
            self.index += 1;
        }
    }

    fn iter_next_n(&mut self, n: usize) {
        let remainder = self.text.len() - self.index;
        let n = n.min(remainder);
        if n > 0 {
            self.iter.nth(n - 1);
            self.index += n;
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
enum TokenType {
    EOF,
    Number,
    InfixOperator,
    UnaryOperator,
    ParenthesisOpen,
    ParenthesisClose,
}

#[derive(Debug)]
enum ParseError {
    Unreachable,
    ParsedFailed,
}

type NextTokens = [Option<TokenType>; 6];
type ParseResult = Result<NextTokens, ParseError>;

fn make_next_tokens(tokens: &[TokenType]) -> NextTokens {
    let mut result: NextTokens = Default::default();
    for (i, tok) in tokens.iter().enumerate() {
        result[i] = Some(tok.clone());
    }
    result
}

struct Opteration {
    num: f32,
    op: char,
}

struct State {
    op_list: [Option<Opteration>; 3],
    unary: char,
    num: f32,
    op_prec: i8,
    paren_opened: bool,
}

impl State {
    fn calcuate_op_list_at(&mut self, prec: usize) {
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

    fn calcuate_op_list_all(&mut self) {
        for p in (0..=self.op_list.len() - 1).rev() {
            Self::calcuate_op_list_at(self, p);
        }
    }

    fn apply_unary(&mut self) {
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
    fn parse_char(&mut self, _state: &mut State, ch: char) -> bool {
        if let Some((_, c)) = self.iter.peek() {
            if *c == ch {
                self.iter_next();
                return true;
            }
        }
        false
    }

    fn parse_char_repeat(&mut self, _state: &mut State, ch: char) -> bool {
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

    fn parse_if(&mut self, _state: &mut State, func: fn(char) -> bool) -> bool {
        if let Some((_, c)) = self.iter.peek() {
            if func(*c) {
                self.iter_next();
                return true;
            }
        }
        false
    }

    fn parse_if_repeat(&mut self, _state: &mut State, func: fn(char) -> bool) -> bool {
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

    fn parse_one_of(
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

    fn parse_constant(&mut self, state: &mut State) -> ParseResult {
        const CONSTANTS: [&str; 3] = ["pi", "tau", "e"];
        const CONSTANTS_VALUE: [f32; 3] = [PI, TAU, E];

        let mut parsed = false;
        for (i, constant) in CONSTANTS.iter().enumerate() {
            if self.text[self.index..].starts_with(constant) {
                // update state
                state.num = CONSTANTS_VALUE[i];

                parsed = true;
                self.iter_next_n(constant.len());
                break;
            }
        }

        if parsed {
            // skip whitespace
            self.parse_if_repeat(state, char::is_whitespace);

            if state.paren_opened {
                return Ok(make_next_tokens(&[
                    TokenType::InfixOperator,
                    TokenType::ParenthesisClose,
                ]));
            } else {
                return Ok(make_next_tokens(&[
                    TokenType::EOF,
                    TokenType::InfixOperator,
                ]));
            }
        }

        Err(ParseError::ParsedFailed)
    }

    fn parse_float(&mut self, state: &mut State) -> ParseResult {
        let mut result: f32 = 0.;
        let mut decimal: f32 = 1.;

        let mut parsed = false;
        'outer: while let Some((_, c)) = self.iter.peek() {
            // parse int
            match c {
                '0'..='9' => {
                    result *= 10.0;
                    result += c.to_digit(10).unwrap() as f32;

                    parsed = true;
                    self.iter_next();
                }
                '.' => {
                    parsed = true;
                    self.iter_next();

                    while let Some((_, c)) = self.iter.peek() {
                        // parse decimal
                        match c {
                            '0'..='9' => {
                                decimal *= 0.1;
                                result += c.to_digit(10).unwrap() as f32 * decimal;

                                self.iter_next();
                            }
                            _ => {
                                break 'outer;
                            }
                        }
                    }
                }
                _ => {
                    break;
                }
            }
        }

        if parsed {
            // update state
            state.num = result;

            // skip whitespace
            self.parse_if_repeat(state, char::is_whitespace);

            if state.paren_opened {
                return Ok(make_next_tokens(&[
                    TokenType::InfixOperator,
                    TokenType::ParenthesisClose,
                ]));
            } else {
                return Ok(make_next_tokens(&[
                    TokenType::EOF,
                    TokenType::InfixOperator,
                ]));
            }
        }

        Err(ParseError::ParsedFailed)
    }

    fn parse_number(&mut self, state: &mut State) -> ParseResult {
        return self.parse_one_of(state, &[Self::parse_constant, Self::parse_float]);
    }

    fn parse_operator(&mut self, state: &mut State) -> ParseResult {
        let mut parsed_prec: i8 = -1;

        if let Some((_, c)) = self.iter.peek() {
            match c {
                '+' | '-' => {
                    parsed_prec = 0;
                }
                '*' | '/' => {
                    parsed_prec = 1;
                }
                '^' => {
                    parsed_prec = 2;
                }
                _ => {}
            }

            if parsed_prec >= 0 {
                let prec = parsed_prec as usize;

                // do calcuation
                if state.op_prec == parsed_prec {
                    state.calcuate_op_list_at(prec);
                } else if state.op_prec > parsed_prec {
                    state.calcuate_op_list_all();
                }

                // update state
                state.op_prec = parsed_prec;
                state.op_list[prec] = Some(Opteration {
                    num: state.num,
                    op: c.clone(),
                });

                self.iter_next();

                // skip whitespace
                self.parse_if_repeat(state, char::is_whitespace);

                return Ok(make_next_tokens(&[
                    TokenType::Number,
                    TokenType::UnaryOperator,
                    TokenType::ParenthesisOpen,
                ]));
            }
        }

        Err(ParseError::ParsedFailed)
    }

    fn parse_unary(&mut self, state: &mut State) -> ParseResult {
        if let Some((_, c)) = self.iter.peek() {
            match c {
                '+' | '-' => {
                    // update state
                    state.unary = c.clone();

                    self.iter_next();

                    return Ok(make_next_tokens(&[
                        TokenType::Number,
                        TokenType::ParenthesisOpen,
                    ]));
                }
                _ => {}
            }
        }

        Err(ParseError::ParsedFailed)
    }

    fn parse_expression(&mut self, paren_opened: bool) -> Result<State, String> {
        let mut state = State {
            op_list: Default::default(),
            unary: 0 as char,
            num: 0.,
            op_prec: 0,
            paren_opened,
        };

        let mut err_msg = String::new();
        let mut parse_result: ParseResult = Err(ParseError::Unreachable);
        let mut next_tokens: NextTokens = make_next_tokens(&[
            TokenType::Number,
            TokenType::UnaryOperator,
            TokenType::ParenthesisOpen,
        ]);

        while next_tokens[0].is_some() {
            parse_result = Err(ParseError::Unreachable);
            for tok in next_tokens.iter() {
                match tok {
                    None => {
                        break;
                    }
                    Some(tok) => {
                        match tok {
                            TokenType::EOF => {
                                if let None = self.iter.peek() {
                                    parse_result = Ok(Default::default());
                                } else {
                                    parse_result = Err(ParseError::ParsedFailed);
                                }
                            }
                            TokenType::Number => {
                                parse_result = self.parse_number(&mut state);
                                if parse_result.is_ok() {
                                    state.apply_unary();
                                }
                            }
                            TokenType::UnaryOperator => {
                                parse_result = self.parse_unary(&mut state);
                            }
                            TokenType::InfixOperator => {
                                parse_result = self.parse_operator(&mut state);
                            }
                            TokenType::ParenthesisOpen => {
                                if self.parse_char(&mut state, '(') {
                                    match self.parse_expression(true) {
                                        Ok(inner_state) => {
                                            state.num = inner_state.num;
                                            state.apply_unary();
                                            parse_result = Ok(make_next_tokens(&[
                                                TokenType::EOF,
                                                TokenType::InfixOperator,
                                                TokenType::ParenthesisClose,
                                            ]));
                                        }
                                        Err(msg) => {
                                            err_msg = msg;
                                            parse_result = Err(ParseError::ParsedFailed);
                                        }
                                    }
                                } else {
                                    parse_result = Err(ParseError::ParsedFailed);
                                }
                            }
                            TokenType::ParenthesisClose => {
                                if self.parse_char(&mut state, ')') {
                                    parse_result = Ok(Default::default());
                                } else {
                                    parse_result = Err(ParseError::ParsedFailed);
                                }
                            }
                        }

                        if let Ok(next) = &parse_result {
                            next_tokens = next.clone();
                            break;
                        }
                    }
                }
            }

            if parse_result.is_err() {
                break;
            }
        }

        match parse_result {
            Ok(_) => {
                state.calcuate_op_list_all();
                Ok(state)
            }
            Err(err) => {
                match err {
                    ParseError::Unreachable => unreachable!(),
                    ParseError::ParsedFailed => {
                        if err_msg.is_empty() {
                            err_msg = String::from_str("Expected one of [\n").unwrap();
                            for tok in next_tokens.iter() {
                                match tok {
                                    None => break,
                                    Some(tok) => err_msg.push_str(&format!("  {:?}\n", tok)),
                                }
                            }
                            err_msg.push_str("]\n");
                            if let Some((_, c)) = self.iter.peek() {
                                err_msg.push_str(&format!("But found \"{}\"", c))
                            } else {
                                err_msg.push_str(&format!("But found EOF"))
                            }
                            err_msg.push_str(&format!(" at index {}", self.index))
                        }
                    }
                }
                Err(err_msg)
            }
        }
    }
}

fn main() {
    let mut input = String::new();
    _ = stdin().read_line(&mut input).unwrap();

    // let input = "1 + 2 + 3 + ((1))-(((-2)))*-(+(2.3+22.7)+2*3^2)*-2"; // = 179
    // let input = "1 + -pi * tau"; // = -18.73921
    // println!("{}", input);

    let input = input.trim();
    let mut context = Context {
        text: input,
        iter: input.char_indices().peekable(),
        index: 0,
    };

    match context.parse_expression(false) {
        Ok(state) => println!("= {}", state.num),
        Err(err) => println!("Error: {}", err),
    }
}
