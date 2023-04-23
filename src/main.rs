// - 문자 순회:
//   - 문자 == 숫자:
//     - prev_num에 저장
//   - 문자 == 연산자:
//     - 이전 op_prec == 현재 op_prec:
//       ```
//       state.op_list[prec].op(state.op_list[prec].num, state.num)
//       ```
//     - 이전 op_prec > 현재 op_prec:
//       ```
//       for p in (0..=state.op_list.high).rev():
//         state.op_list[p].op(state.op_list[p].num, state.num)
//       ```
//     - state.op_list[prec]에 연산자를 저장
//     - 현재 op_prec을 state.op_prec에 저장

use std::{io::stdin, iter::Peekable, str::CharIndices};

// fn debug_print

#[derive(PartialEq)]
enum TokenType {
    None,
    Number,
    Operator,
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

fn calcuate_op_list_at(state: &mut State, prec: usize) {
    if let Some(op) = &mut state.op_list[prec] {
        state.num = match op.op {
            '+' => op.num + state.num,
            '-' => op.num - state.num,
            '*' => op.num * state.num,
            '/' => op.num / state.num,
            '^' => op.num.powf(state.num),
            _ => {
                println!("invalid operator \"{}\"", op.op);
                state.num
            }
        };
        state.op_list[prec] = None;
    }
}

fn calcuate_op_list_all(state: &mut State) {
    for p in (0..=state.op_list.len() - 1).rev() {
        calcuate_op_list_at(state, p);
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

fn parse_whitespace(line_iter: &mut Peekable<CharIndices>, index: &mut usize) {
    while let Some((_, c)) = line_iter.peek() {
        if !c.is_whitespace() {
            return;
        }

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

                line_iter.next();
                *index += 1;
            }
            '.' => {
                parsed = true;

                line_iter.next();
                *index += 1;

                while let Some((_, c)) = line_iter.peek() {
                    match c {
                        '0'..='9' => {
                            decimal *= 0.1;
                            result += c.to_digit(10).unwrap() as f32 * decimal;

                            line_iter.next();
                            *index += 1;
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
        apply_unary(state);

        println!("number [{}] {}", index, result);
    }
}

fn parse_operator(line_iter: &mut Peekable<CharIndices>, index: &mut usize, state: &mut State) {
    let mut parsed_prec: i8 = -1;
    if let Some((_, c)) = line_iter.peek() {
        match c {
            '+' | '-' => parsed_prec = 0,
            '*' | '/' => parsed_prec = 1,
            '^' => parsed_prec = 2,
            _ => {}
        }

        if parsed_prec >= 0 {
            let prec = parsed_prec as usize;

            // do calcuation
            if state.op_prec == parsed_prec {
                calcuate_op_list_at(state, prec);
            } else if state.op_prec > parsed_prec {
                calcuate_op_list_all(state);
            }

            // update state
            state.token = TokenType::Operator;
            state.op_prec = parsed_prec;
            state.op_list[prec] = Some(Opteration {
                num: state.num,
                op: c.clone(),
            });

            println!("operator [{}] {}", index, c);
            line_iter.next();
            *index += 1;
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

                println!("unary operator [{}] {}", index, c);

                line_iter.next();
                *index += 1;
            }
            _ => {}
        }
    }
}

fn parse_consume(line_iter: &mut Peekable<CharIndices>, index: &mut usize) {
    if let Some(_) = line_iter.peek() {
        line_iter.next();
        *index += 1;
    }
}

fn parse_unexpected(line_iter: &mut Peekable<CharIndices>, index: &mut usize) {
    if let Some((_, c)) = line_iter.peek() {
        println!("unexpected character [{}] {}", index, c);
        line_iter.next();
        *index += 1;
    }
}

fn parse_expression(line_iter: &mut Peekable<CharIndices>, index: &mut usize) -> State {
    let mut state = State {
        op_list: [None, None, None],
        token: TokenType::None,
        unary: 0 as char,
        num: 0.,
        op_prec: 0,
    };

    while let Some((_, c)) = line_iter.peek() {
        match c {
            _ if { c.is_whitespace() } => {
                parse_whitespace(line_iter, index);
            }
            '.' | '0'..='9' => {
                parse_number(line_iter, index, &mut state);
            }
            '+' | '-'
                if { state.token == TokenType::None || state.token == TokenType::Operator } =>
            {
                parse_unary(line_iter, index, &mut state);
            }
            '+' | '-' | '*' | '/' | '^' => {
                parse_operator(line_iter, index, &mut state);
            }
            '(' => {
                parse_consume(line_iter, index);
                state.num = parse_expression(line_iter, index).num;
                apply_unary(&mut state);
            }
            ')' => {
                parse_consume(line_iter, index);
                break;
            }
            _ => {
                parse_unexpected(line_iter, index);
            }
        }
    }

    calcuate_op_list_all(&mut state);

    state
}

fn main() {
    // read line from stdin
    let mut line = String::new();
    _ = stdin().read_line(&mut line).unwrap();
    let line = line.trim();
    // let line = "-2*((2+22)*2)*-2";
    // let line = "2.01 + 2.0";
    // println!("input = {}", line);

    let mut line_iter = line.char_indices().peekable();
    let mut index: usize = 0;

    let state = parse_expression(&mut line_iter, &mut index);
    println!("result = {}", state.num);
}
