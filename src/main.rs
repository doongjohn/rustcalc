mod parser;

use parser::*;
use rustyline::error::ReadlineError;
use std::f64::consts::{E, PI, TAU};

impl Context<'_> {
    fn parse_constant(&mut self, state: &mut State) -> ParseResult {
        const CONSTANTS_STR: [&str; 3] = ["tau", "pi", "e"];
        const CONSTANTS_VAL: [f64; 3] = [TAU, PI, E];

        let mut parsed = false;
        for (i, constant) in CONSTANTS_STR.iter().enumerate() {
            if self.text[self.index..].starts_with(constant) {
                // update state
                state.num = CONSTANTS_VAL[i];

                parsed = true;
                self.iter_next_n(constant.len());
                break;
            }
        }

        if parsed {
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

        Err(ParseError::ParseFailed)
    }

    fn parse_float(&mut self, state: &mut State) -> ParseResult {
        let mut result: f64 = 0.;
        let mut decimal: f64 = 1.;

        let mut is_parsed = false;
        'outer: while let Some((_, c)) = self.iter.peek() {
            match c {
                // matches `.0` and `0.` but not `.`
                '0'..='9' => {
                    // parse int
                    result *= 10.0;
                    result += c.to_digit(10).unwrap() as f64;

                    is_parsed = true;
                    self.iter_next();
                }
                '.' => {
                    if is_parsed {
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
                                result += c.to_digit(10).unwrap() as f64 * decimal;

                                is_parsed = true;
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

        if is_parsed {
            // update state
            state.num = result;

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

        Err(ParseError::ParseFailed)
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

        Err(ParseError::ParseFailed)
    }

    fn parse_infix_operator(&mut self, state: &mut State) -> ParseResult {
        let mut parsed_prec: i8 = -1;

        if let Some((_, c)) = self.iter.peek() {
            match c {
                '+' | '-' => parsed_prec = 0,
                '*' | '/' => parsed_prec = 1,
                '^' => parsed_prec = 2,
                _ => {}
            }

            if parsed_prec != -1 {
                let prec = parsed_prec as usize;

                // do calcuation
                if state.op_prec == parsed_prec {
                    state.calcuate_op_list_prec(prec);
                } else if state.op_prec > parsed_prec {
                    state.calcuate_op_list_all();
                }

                // update state
                state.op_prec = parsed_prec;
                state.op_list[prec] = Some(Operation {
                    num: state.num,
                    op: c.clone(),
                });

                self.iter_next();
                self.parse_if_repeat(state, char::is_whitespace);

                return Ok(make_next_tokens(&[
                    TokenType::Number,
                    TokenType::UnaryOperator,
                    TokenType::ParenthesisOpen,
                ]));
            }
        }

        Err(ParseError::ParseFailed)
    }

    fn parse_parenthesis_open(&mut self, state: &mut State) -> ParseResult {
        if self.parse_char(state, '(') {
            match self.eval_expression(true) {
                Ok(inner_state) => {
                    // update state
                    state.num = inner_state.num;

                    if state.paren_opened {
                        Ok(make_next_tokens(&[
                            TokenType::InfixOperator,
                            TokenType::ParenthesisClose,
                        ]))
                    } else {
                        Ok(make_next_tokens(&[
                            TokenType::InfixOperator,
                            TokenType::Eof,
                        ]))
                    }
                }
                Err(_) => Err(ParseError::ParseFailed),
            }
        } else {
            Err(ParseError::ParseFailed)
        }
    }

    fn parse_parenthesis_close(&mut self, state: &mut State) -> ParseResult {
        if self.parse_char(state, ')') {
            Ok(Default::default())
        } else {
            Err(ParseError::ParseFailed)
        }
    }

    fn eval_expression(&mut self, paren_opened: bool) -> Result<State, ()> {
        let mut state = State {
            op_list: Default::default(),
            unary: 0 as char,
            num: 0.,
            op_prec: 0,
            paren_opened,
        };

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
                        parse_result = match tok {
                            TokenType::Number => self.parse_number(&mut state),
                            TokenType::UnaryOperator => self.parse_unary_operator(&mut state),
                            TokenType::InfixOperator => self.parse_infix_operator(&mut state),
                            TokenType::ParenthesisOpen => self.parse_parenthesis_open(&mut state),
                            TokenType::ParenthesisClose => self.parse_parenthesis_close(&mut state),
                            TokenType::Eof => {
                                if let None = self.iter.peek() {
                                    Ok(Default::default())
                                } else {
                                    Err(ParseError::ParseFailed)
                                }
                            }
                        };

                        if let Ok(next) = &parse_result {
                            // apply unary operator
                            if [TokenType::Number, TokenType::ParenthesisOpen].contains(tok) {
                                state.apply_unary_operator();
                            }

                            // update next tokens
                            next_tokens = next.clone();
                            break;
                        }
                    }
                }
            }

            // break on error
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
                    ParseError::ParseFailed => {
                        if self.err_msg.is_empty() {
                            self.generate_parse_failed_err_msg(&next_tokens);
                        }
                    }
                }
                Err(())
            }
        }
    }
}

fn main() {
    let mut editor = rustyline::DefaultEditor::new().unwrap();
    loop {
        match editor.readline("rustcalc) ") {
            Ok(input) => {
                let input = input.trim();
                editor.add_history_entry(input).unwrap();

                let mut context = Context {
                    text: input,
                    iter: input.char_indices().peekable(),
                    index: 0,
                    err_msg: String::new(),
                };

                match context.eval_expression(false) {
                    Ok(state) => println!("= {}", state.num),
                    Err(_) => println!("Error: {}", context.err_msg),
                }
            }
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}
