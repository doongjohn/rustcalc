/*
simple math expression parser

- no heap allocation except `stdin().read_line()` and `format!()`
- f32 number
- predefined constatns
    - 'pi', 'tau', 'e'
- unary opeartor
    - '+', '-'
- infix operator
    - '+', '-', '*', '/', '^'
    - operator precedence
- nested parentheses
- simple error message
*/

/*
- if char is number:
    - state.num = number
- if char is operator:
    - if state.op_prec == current op_prec:
      ```
      state.op_list[prec].op(state.op_list[prec].num, state.num)
      state.op_list[prec] = None
      ```
    - if state.op_prec > current op_prec:
      ```
      for p in (0..=state.op_list.high).rev():
          state.op_list[p].op(state.op_list[p].num, state.num)
          state.op_list[p] = None
      ```
    - store operator to state.op_list[prec].op
    - store state.num to state.op_list[prec].num
    - store current op_prec to state.op_prec
*/

// TODO: support builtin function (sqrt, abs, floor, ceil, round, ...)
// TODO: support imaginary number
// TODO: support custom variable (declaration, assignment)
// TODO: support custom function (infix, unary)

use std::{
    f32::consts::{E, PI, TAU},
    io::stdin,
    iter::Peekable,
    str::CharIndices,
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

#[derive(Debug, PartialEq)]
enum TokenType {
    None,
    UnkownConstant,
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
    fn parse_constant(&mut self, state: &mut State) {
        const CONSTANTS: [&str; 3] = ["pi", "tau", "e"];
        const CONSTANTS_VALUE: [f32; 3] = [PI, TAU, E];

        let mut parsed = false;
        for (i, constant) in CONSTANTS.iter().enumerate() {
            if self.text[self.index..].starts_with(constant) {
                // update state
                state.token = TokenType::Number;
                state.num = CONSTANTS_VALUE[i];

                parsed = true;
                self.iter_next_n(constant.len());
                break;
            }
        }

        if !parsed {
            state.token = TokenType::UnkownConstant;
        }
    }

    fn parse_number(&mut self, state: &mut State) {
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
            state.token = TokenType::Number;
            state.num = result;

            // println!("number [{}] {}", index, result);
        }
    }

    fn parse_operator(&mut self, state: &mut State) {
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
                state.token = TokenType::InfixOperator;
                state.op_prec = parsed_prec;
                state.op_list[prec] = Some(Opteration {
                    num: state.num,
                    op: c.clone(),
                });

                // println!("operator [{}] {}", index, c);
                self.iter_next();
            }
        }
    }

    fn parse_unary(&mut self, state: &mut State) {
        if let Some((_, c)) = self.iter.peek() {
            match c {
                '+' | '-' => {
                    // update state
                    state.token = TokenType::UnaryOperator;
                    state.unary = c.clone();

                    // println!("unary operator [{}] {}", index, c);
                    self.iter_next();
                }
                _ => {}
            }
        }
    }

    fn parse_expression(&mut self, mut paren_opened: bool) -> Result<State, String> {
        let mut state = State {
            op_list: Default::default(),
            token: TokenType::None,
            unary: 0 as char,
            num: 0.,
            op_prec: 0,
        };

        while let Some((_, c)) = self.iter.peek() {
            match c {
                _ if { c.is_whitespace() && state.token != TokenType::UnaryOperator } => {
                    self.iter_next();
                }
                '+' | '-'
                    if {
                        state.token == TokenType::None || state.token == TokenType::InfixOperator
                    } =>
                {
                    self.parse_unary(&mut state);
                }
                'a'..='z'
                    if {
                        state.token != TokenType::Number && state.token != TokenType::UnkownConstant
                    } =>
                {
                    self.parse_constant(&mut state);
                    state.apply_unary();
                }
                '.' | '0'..='9' if { state.token != TokenType::Number } => {
                    self.parse_number(&mut state);
                    state.apply_unary();
                }
                '(' if { state.token != TokenType::Number } => {
                    self.iter_next();
                    match self.parse_expression(true) {
                        Ok(inner_state) => {
                            state.token = TokenType::Number;
                            state.num = inner_state.num;
                            state.apply_unary();
                        }
                        Err(msg) => {
                            return Err(msg);
                        }
                    }
                }
                ')' if { paren_opened } => {
                    paren_opened = false;
                    self.iter_next();
                    break;
                }
                '+' | '-' | '*' | '/' | '^' if { state.token == TokenType::Number } => {
                    self.parse_operator(&mut state);
                }
                _ => match state.token {
                    TokenType::UnkownConstant => {
                        return Err(format!(
                            "error: expect {{ `pi` | 'tau' | 'e' }} at index {} but found \"{}\"",
                            self.index, c
                        ));
                    }
                    TokenType::None | TokenType::InfixOperator => {
                        return Err(format!(
                        "error: expect {{ `Number` | `UnaryOperator` | `(` }} at index {} but found \"{}\"",
                        self.index, c
                    ));
                    }
                    TokenType::UnaryOperator => {
                        return Err(format!(
                            "error: expect {{ `Number` | `(` }} at index {} but found \"{}\"",
                            self.index, c
                        ));
                    }
                    TokenType::Number => {
                        return Err(format!(
                            "error: expect {{ `InfixOperator` }} at index {} but found \"{}\"",
                            self.index, c
                        ));
                    }
                },
            }
        }

        match state.token {
            TokenType::None => {
                return Err(format!(
                    "error: expect {{ `Number` | `UnaryOperator` | `(` }} at index {}",
                    self.index
                ));
            }
            TokenType::InfixOperator | TokenType::UnaryOperator => {
                return Err(format!(
                    "error: expect {{ `Number` }} at index {}",
                    self.index
                ));
            }
            TokenType::Number => {
                state.calcuate_op_list_all();
            }
            _ => {}
        }

        if paren_opened {
            return Err(format!("error: expect {{ `)` }} at index {}", self.index));
        }

        Ok(state)
    }
}

fn main() {
    let mut text = String::new();
    _ = stdin().read_line(&mut text).unwrap();

    // let text = "1 + 2 + 3 + (1)--2*-(+(2.3+22.7)+2*3^2)*-2"; // = 179
    // let text = "1 + -pi * tau"; // = -18.73921
    // println!("{}", text);

    let text = text.trim();
    let mut context = Context {
        text,
        iter: text.char_indices().peekable(),
        index: 0,
    };

    match context.parse_expression(false) {
        Ok(state) => println!("= {}", state.num),
        Err(msg) => println!("{}", msg),
    }
}
