# simple math expression parser

## features

- no heap allocation except
  - reading an input
  - generating the error message
- all numbers are f32
- predefined constatns
  - `['pi', 'tau', 'e']`
- unary opeartor
  - `['+', '-']`
- infix operator
  - `['+', '-', '*', '/', '^']`
  - operator precedence
- nested parentheses
- simple error message

## operator precedence algorithm

- if char is number:
  - state.num = number
- if char is operator:
  - if state.op_prec == current op_prec:
    ```rust
    let op_fn = get_op_fn(state.op_list[prec].op)
    state.num = op_fn(state.op_list[prec].num, state.num)
    state.op_list[prec] = None
    ```
  - if state.op_prec > current op_prec:
    ```rust
    for p in (0..=state.op_list.high).rev() {
        state.op_list[p].op(state.op_list[p].num, state.num)
        state.op_list[p] = None
    }
    ```
    - store operator to state.op_list[prec].op
    - store state.num to state.op_list[prec].num
    - store current op_prec to state.op_prec
