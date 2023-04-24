// - if char is number:
//   - state.num = number
// - if char is operator:
//   - if state.op_prec == current op_prec:
//     ```
//     state.op_list[prec].op(state.op_list[prec].num, state.num)
//     state.op_list[prec] = None
//     ```
//   - if state.op_prec > current op_prec:
//     ```
//     for p in (0..=state.op_list.high).rev():
//       state.op_list[p].op(state.op_list[p].num, state.num)
//       state.op_list[p] = None
//     ```
//   - store operator to state.op_list[prec].op
//   - store state.num to state.op_list[prec].num
//   - store current op_prec to state.op_prec

// TODO: parser should return error
// TODO: support constants (pi, tau, e, ...)
// TODO: support imaginary number
// TODO: support custom variable (declaration, assignment)
// TODO: support builtin function (sqrt, abs, floor, ceil, round, ...)
// TODO: support custom function (infix, unary)

use std::{io::stdin, iter::Peekable, str::CharIndices};

const INFIX_OPERATORS: [char; 5] = ['+', '-', '*', '/', '^'];

#[derive(Debug, PartialEq)]
enum TokenType {
    None,
    Number,
    InfixOperator,
    UnaryOperator,
}

struct Opteration {
    num: f32,
    op: char,
}

struct State {
    op_list: [Option<Opteration>; 3],
    token: TokenType,
    unary: char,
    num: f32,
    op_prec: i8,
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
}

fn apply_unary(state: &mut State) {
    match state.unary {
        '+' => {
            state.unary = 0 as char;
        }
        '-' => {
            state.num *= -1.;
            state.unary = 0 as char;
        }
        _ => {}
    }
}

fn consume(line_iter: &mut Peekable<CharIndices>, index: &mut usize) {
    if let Some((_, _c)) = line_iter.peek() {
        line_iter.next();
        *index += 1;
    }
}

fn parse_number(line_iter: &mut Peekable<CharIndices>, index: &mut usize, state: &mut State) {
    let mut parsed = false;
    let mut result: f32 = 0.;
    let mut decimal: f32 = 1.;

    while let Some((_, c)) = line_iter.peek() {
        match c {
            '0'..='9' => {
                parsed = true;

                result *= 10.0;
                result += c.to_digit(10).unwrap() as f32;

                consume(line_iter, index);
            }
            '.' => {
                parsed = true;

                consume(line_iter, index);

                while let Some((_, c)) = line_iter.peek() {
                    match c {
                        '0'..='9' => {
                            decimal *= 0.1;
                            result += c.to_digit(10).unwrap() as f32 * decimal;

                            consume(line_iter, index);
                        }
                        _ => {
                            break;
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
        state.token = TokenType::Number;
        state.num = result;

        // println!("number [{}] {}", index, result);
    }
}

fn parse_operator(line_iter: &mut Peekable<CharIndices>, index: &mut usize, state: &mut State) {
    let mut parsed_prec: i8 = -1;

    if let Some((_, c)) = line_iter.peek() {
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
            state.token = TokenType::InfixOperator;
            state.op_prec = parsed_prec;
            state.op_list[prec] = Some(Opteration {
                num: state.num,
                op: c.clone(),
            });

            // println!("operator [{}] {}", index, c);
            consume(line_iter, index);
        }
    }
}

fn parse_unary(line_iter: &mut Peekable<CharIndices>, index: &mut usize, state: &mut State) {
    if let Some((_, c)) = line_iter.peek() {
        match c {
            '+' | '-' => {
                // update state
                state.token = TokenType::UnaryOperator;
                state.unary = c.clone();

                // println!("unary operator [{}] {}", index, c);
                consume(line_iter, index);
            }
            _ => {}
        }
    }
}

fn parse_expression(
    line_iter: &mut Peekable<CharIndices>,
    index: &mut usize,
    mut paren: bool,
) -> Result<State, String> {
    let mut state = State {
        op_list: Default::default(),
        token: TokenType::None,
        unary: 0 as char,
        num: 0.,
        op_prec: 0,
    };

    while let Some((_, c)) = line_iter.peek() {
        match c {
            _ if { c.is_whitespace() && state.token != TokenType::UnaryOperator } => {
                consume(line_iter, index);
            }
            '+' | '-'
                if {
                    state.token == TokenType::None || state.token == TokenType::InfixOperator
                } =>
            {
                parse_unary(line_iter, index, &mut state);
            }
            _ if { INFIX_OPERATORS.contains(c) && state.token == TokenType::Number } => {
                parse_operator(line_iter, index, &mut state);
            }
            '.' | '0'..='9' if { state.token != TokenType::Number } => {
                parse_number(line_iter, index, &mut state);
                apply_unary(&mut state);
            }
            '(' if { state.token != TokenType::Number } => {
                consume(line_iter, index);
                match parse_expression(line_iter, index, true) {
                    Ok(inner_state) => {
                        state.token = TokenType::Number;
                        state.num = inner_state.num;
                        apply_unary(&mut state);
                    }
                    Err(msg) => {
                        return Err(msg);
                    }
                }
            }
            ')' if { paren } => {
                paren = false;
                consume(line_iter, index);
                break;
            }
            _ => match state.token {
                TokenType::None | TokenType::InfixOperator => {
                    return Err(format!(
                        "error: expect {{ `Number` | `UnaryOperator` | `(` }} at index {} but found \"{}\"",
                        index, c
                    ));
                }
                TokenType::UnaryOperator => {
                    return Err(format!(
                        "error: expect {{ `Number` | `(` }} at index {} but found \"{}\"",
                        index, c
                    ));
                }
                TokenType::Number => {
                    return Err(format!(
                        "error: expect {{ `InfixOperator` }} at index {} but found \"{}\"",
                        index, c
                    ));
                }
            },
        }
    }

    match state.token {
        TokenType::None => {
            return Err(format!(
                "error: expect {{ `Number` | `UnaryOperator` | `(` }} at index {}",
                index
            ));
        }
        TokenType::InfixOperator | TokenType::UnaryOperator => {
            return Err(format!("error: expect {{ `Number` }} at index {}", index));
        }
        TokenType::Number => {
            state.calcuate_op_list_all();
        }
    }

    if paren {
        return Err(format!("error: expect {{ `)` }} at index {}", index));
    }

    Ok(state)
}

fn main() {
    let mut line = String::new();
    _ = stdin().read_line(&mut line).unwrap();

    // let line = "1 + 2 + 3 + (1)--2*-(+(2.3+22.7)+2*3^2)*-2"; // = 179
    // println!("{}", line);

    let mut line_iter = line.trim().char_indices().peekable();
    let mut index: usize = 0;

    match parse_expression(&mut line_iter, &mut index, false) {
        Ok(state) => println!("= {}", state.num),
        Err(msg) => println!("{}", msg),
    }
}
