// TODO: support builtin function (sqrt, abs, floor, ceil, round, ...)
// TODO: support imaginary number
// TODO: support custom variable (declaration, assignment)
// TODO: support custom function (infix, unary)

mod parser;

use parser::*;
use rustyline::{error::ReadlineError, DefaultEditor};
use std::{
    f32::consts::{E, PI, TAU},
    str::FromStr,
};

impl Context<'_> {
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
                    TokenType::InfixOperator,
                    TokenType::Eof,
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
                    if parsed {
                        self.iter_next();
                    } else {
                        if let Some((_, c)) = self.iter.clone().nth(1) {
                            match c {
                                '0'..='9' => self.iter_next(),
                                _ => break,
                            }
                        } else {
                            break;
                        }
                    }

                    while let Some((_, c)) = self.iter.peek() {
                        // parse decimal
                        match c {
                            '0'..='9' => {
                                decimal *= 0.1;
                                result += c.to_digit(10).unwrap() as f32 * decimal;

                                parsed = true;
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
                    TokenType::InfixOperator,
                    TokenType::Eof,
                ]));
            }
        }

        Err(ParseError::ParsedFailed)
    }

    fn parse_number(&mut self, state: &mut State) -> ParseResult {
        return self.parse_one_of(state, &[Self::parse_constant, Self::parse_float]);
    }

    fn parse_unary_operator(&mut self, state: &mut State) -> ParseResult {
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

    fn parse_infix_operator(&mut self, state: &mut State) -> ParseResult {
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
        let mut next_tokens = make_next_tokens(&[
            TokenType::Number,
            TokenType::UnaryOperator,
            TokenType::ParenthesisOpen,
        ]);

        while next_tokens[0].is_some() {
            parse_result = Err(ParseError::Unreachable);

            for tok in next_tokens.iter() {
                match tok {
                    None => break,
                    Some(tok) => {
                        match tok {
                            TokenType::Number => {
                                parse_result = self.parse_number(&mut state);
                                if parse_result.is_ok() {
                                    state.apply_unary_operator();
                                }
                            }
                            TokenType::UnaryOperator => {
                                parse_result = self.parse_unary_operator(&mut state);
                            }
                            TokenType::InfixOperator => {
                                parse_result = self.parse_infix_operator(&mut state);
                            }
                            TokenType::ParenthesisOpen => {
                                parse_result = Err(ParseError::ParsedFailed);
                                if self.parse_char(&mut state, '(') {
                                    match self.parse_expression(true) {
                                        Ok(inner_state) => {
                                            state.num = inner_state.num;
                                            state.apply_unary_operator();
                                            parse_result = Ok(make_next_tokens(&[
                                                TokenType::InfixOperator,
                                                TokenType::ParenthesisClose,
                                                TokenType::Eof,
                                            ]));
                                        }
                                        Err(msg) => {
                                            err_msg = msg;
                                        }
                                    }
                                }
                            }
                            TokenType::ParenthesisClose => {
                                parse_result = if self.parse_char(&mut state, ')') {
                                    Ok(Default::default())
                                } else {
                                    Err(ParseError::ParsedFailed)
                                }
                            }
                            TokenType::Eof => {
                                parse_result = if let None = self.iter.peek() {
                                    Ok(Default::default())
                                } else {
                                    Err(ParseError::ParsedFailed)
                                }
                            }
                        }

                        // update next tokens
                        if let Ok(next) = &parse_result {
                            next_tokens = next.clone();
                            break;
                        }
                    }
                }
            }

            // check error
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
                            err_msg = String::from_str("Expected one of {").unwrap();
                            for (i, tok) in next_tokens.iter().enumerate() {
                                match tok {
                                    None => break,
                                    Some(tok) => {
                                        err_msg.push_str(&format!("{:?}", tok));
                                        if let Some(Some(_)) = next_tokens.iter().nth(i + 1) {
                                            err_msg.push_str(", ");
                                        }
                                    }
                                }
                            }
                            err_msg.push_str("} But found ");
                            if let Some((_, c)) = self.iter.peek() {
                                err_msg.push_str(&format!("\"{}\"", c));
                            } else {
                                err_msg.push_str("EOF");
                            }
                            err_msg.push_str(&format!(" at index {}", self.index));
                        }
                    }
                }
                Err(err_msg)
            }
        }
    }
}

fn main() {
    let mut editor = DefaultEditor::new().unwrap();
    loop {
        match editor.readline("rustcalc) ") {
            Ok(input) => {
                let input = input.trim();
                editor.add_history_entry(input).unwrap();

                // let input = "1 + 2 + 3 + ((1))-(((-2)))*-(+(2.3+22.7)+2*3^2)*-2"; // = 179
                // let input = "1 + -pi * tau"; // = -18.73921
                // println!("{}", input);

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
            Err(ReadlineError::Interrupted) => {
                break;
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}
