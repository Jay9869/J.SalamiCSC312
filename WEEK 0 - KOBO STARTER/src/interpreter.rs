use crate::ast::{Expr, Program, Resolved, Stmt};
use crate::environment::Environment;
use crate::token::{Token, TokenType};
use crate::value::Value;

pub struct RuntimeError {
    pub line: usize,
    pub message: String,
}

/// Why a statement stopped early. An error unwinds to `main`; a `return` unwinds only as
/// far as the call that started the function, which is why both travel the same way. Rust
/// gives `?` for free on a `Result`, and a `return` is exactly a non-local exit.
enum Flow {
    Error(RuntimeError),
    Return(Value),
}

type Exec<T> = Result<T, Flow>;

/// Kobo's call stack is the Rust call stack: `call` calls `execute` calls `evaluate`
/// calls `call`. Running off the end of it aborts the process with no diagnostic, so the
/// depth is capped instead. Measured on this toolchain, the real limit is near 1,500
/// frames; 500 leaves room for a deeply nested expression inside every one of them.
pub const MAX_FRAMES: usize = 500;

pub fn interpret(program: &Program) -> Result<(), RuntimeError> {
    let mut env = Environment::new();
    for stmt in &program.stmts {
        match execute(stmt, &mut env, program) {
            Ok(()) => {}
            // `return` outside any function ends the program (spec section 3.1).
            Err(Flow::Return(_)) => return Ok(()),
            Err(Flow::Error(e)) => return Err(e),
        }
    }
    Ok(())
}

fn execute(stmt: &Stmt, env: &mut Environment, program: &Program) -> Exec<()> {
    // TODO(you): spec 4: run one statement.

    match stmt {
        Stmt::Expression(expr) => {
            evaluate(expr, env, program)?;
            Ok(())
        }
        Stmt::Print(expr) => {
            let value = evaluate(expr, env, program)?;
            // println!(
            //     "value {}",
            //     value
            // );
            println!("{}", value);
            Ok(())
        }
        Stmt::Var {
            name,
            initialiser,
            slot: _,
        } => {
            // println!(
            //     "var {}",
            //     name.lexeme
            // );

            let value = match initialiser {
                Some(expr) => {
                    // println!(
                    //     "var {}",
                    //     name.lexeme
                    // );
                    evaluate(expr, env, program)?
                }
                None => {
                    // println!(
                    //     "var {}",
                    //     name.lexeme
                    // );
                    Value::Nil
                }
            };
            // println!(
            //     "var {}, value {}",
            //     name.lexeme,
            //     value
            // );
            env.declare(&name.lexeme, value);
            Ok(())
        }
        Stmt::Block { body, .. } => {
            // println!(
            //     "stmts {} ",
            //     body.len()
            // );
            env.push_scope();

            let mut outcome = Ok(());
            for statement in body {
                if let Err(flow) = execute(statement, env, program) {
                    outcome = Err(flow);
                    break;
                }
            }

            env.pop_scope();

            outcome
        }
        Stmt::If {
            condition,
            then_branch,
            else_branch,
        } => {
            if evaluate(condition, env, program)?.is_truthy() {
                execute(then_branch, env, program)
            } else if let Some(else_branch) = else_branch {
                execute(else_branch, env, program)
            } else {
                Ok(())
            }
        }
        Stmt::While { condition, body } => {
            while evaluate(condition, env, program)?.is_truthy() {
                execute(body, env, program)?;
            }

            Ok(())
        }
        Stmt::Function { index, name, .. } => {
            // println!(
            //     "stmt {}",
            //     name
            // );

            // println!(
            //     "function {}, index {}",
            //     name,
            //     index
            // );
            env.declare(
                name,
                Value::Function {
                    index: *index,
                    name: name.to_string(),
                },
            );
            Ok(())
        }
        Stmt::Return(value) => {
            let value = match value {
                Some(expr) => evaluate(expr, env, program)?,
                None => Value::Nil,
            };

            Err(Flow::Return(value))
        }
    }
}

fn evaluate(expr: &Expr, env: &mut Environment, program: &Program) -> Exec<Value> {
    // TODO(you): spec 2 and 4.1: evaluate one expression to a Value. Truthiness, equality
    //            across kinds and short-circuiting are all specified.

    match expr {
        Expr::Literal { value, .. } => {
            // println!(
            //     "literal is{}",
            //     value
            // );
            Ok(value.clone())
        }
        Expr::Grouping(inner) => evaluate(inner, env, program),
        Expr::Variable { name, at } => {
            match at {
                Resolved::Local { depth, .. } => {
                    // println!(
                    //     "local{}, depth {}",
                    //     name.lexeme,
                    //     depth
                    // );
                    env.get_at(*depth, &name.lexeme)
                        .cloned()
                        .ok_or_else(|| undefined(name))
                }
                Resolved::Global => {
                    // println!(
                    //     "global {}",
                    //     name.lexeme
                    // );
                    env.get_global(&name.lexeme)
                        .cloned()
                        .ok_or_else(|| undefined(name))
                }
            }
        }
        Expr::Assign { name, value, at } => {
            let value = evaluate(value, env, program)?;

            let stored = match at {
                Resolved::Local { depth, .. } => env.assign_at(*depth, &name.lexeme, value.clone()),
                Resolved::Global => env.assign_global(&name.lexeme, value.clone()),
            };

            if stored {
                Ok(value)
            } else {
                Err(undefined(name))
            }
        }
        Expr::Call {
            callee,
            paren,
            args,
        } => {
            // println!(
            //     "call at {}",
            //     paren.line
            // );
            let callee = evaluate(callee, env, program)?;

            let mut arguments = Vec::with_capacity(args.len());
            for arg in args {
                arguments.push(evaluate(arg, env, program)?);
            }

            match callee {
                Value::Function { index, .. } => call(index, arguments, paren.line, env, program),
                _ => Err(err(paren.line, "This value is not a function.")),
            }
        }
        Expr::Unary { op, right } => {
            // println!(
            //     "unary {}",
            //     op.lexeme
            // );

            // println!(
            //     "unary operator {}",
            //     op.lexeme
            // );
            let right = evaluate(right, env, program)?;
            match op.kind {
                TokenType::Bang => {
                    // println!(
                    //     "unary bang {}",
                    //     !right.is_truthy()
                    // );
                    Ok(Value::Bool(!right.is_truthy()))
                }
                TokenType::Minus => match right {
                    Value::Number(number) => Ok(Value::Number(-number)),
                    _ => Err(err(op.line, "Operand must be a number.")),
                },
                _ => unreachable!("the parser only produces ! and - as unary operators"),
            }
        }
        Expr::Binary { left, op, right } => {
            let left = evaluate(left, env, program)?;
            let right = evaluate(right, env, program)?;

            // println!(
            //     "binary op {}, left {}, right {}",
            //     op.lexeme,
            //     left,
            //     right
            // );

            match op.kind {
                TokenType::Minus => arith(left, right, op.line, |a, b| a - b),

                TokenType::Slash => {
                    let (a, b) = numbers(left, right, op.line)?;

                    if b == 0.0 {
                        Err(err(op.line, "Division by zero."))
                    } else {
                        Ok(Value::Number(a / b))
                    }
                }

                TokenType::Star => arith(left, right, op.line, |a, b| a * b),

                TokenType::Plus => match (left, right) {
                    (Value::Number(a), Value::Number(b)) => {
                        // println!(
                        //     "added numbers {} and {}",
                        //     a,
                        //     b
                        // );
                        Ok(Value::Number(a + b))
                    }
                    (Value::Str(a), Value::Str(b)) => {
                        // println!(
                        //     "joined strings {} and {}",
                        //     a,
                        //     b
                        // );
                        Ok(Value::Str(a + &b))
                    }
                    _ => Err(err(op.line, "Operands must be two numbers or two strings.")),
                },

                TokenType::Greater => compare(left, right, op.line, |a, b| a > b),
                TokenType::GreaterEqual => compare(left, right, op.line, |a, b| a >= b),
                TokenType::Less => compare(left, right, op.line, |a, b| a < b),
                TokenType::LessEqual => compare(left, right, op.line, |a, b| a <= b),
                TokenType::EqualEqual => Ok(Value::Bool(left == right)),
                TokenType::BangEqual => Ok(Value::Bool(left != right)),

                _ => unreachable!("the parser only produces binary operators here"),
            }
        }
        Expr::Logical { left, op, right } => {
            // println!(
            //     "logical {}",
            //     op.lexeme
            // );

            let left = evaluate(left, env, program)?;

            // println!(
            //     "logical op {}, left value {}",
            //     op.lexeme,
            //     left
            // );

            match op.kind {
                TokenType::Or => {
                    if left.is_truthy() {
                        Ok(left)
                    } else {
                        evaluate(right, env, program)
                    }
                }
                TokenType::And => {
                    if !left.is_truthy() {
                        Ok(left)
                    } else {
                        evaluate(right, env, program)
                    }
                }
                _ => unreachable!("the parser only produces and/or as logical operators"),
            }
        }
    }
}

/// Run a function body in a fresh frame. Everything about Kobo's scope rules is in these
/// twenty lines: the parameters are declared in the frame's own scope, the body can reach
/// past it only to the globals, and the frame is popped on every path out: normal
/// completion, `return`, or a runtime error.
fn call(
    index: usize,
    args: Vec<Value>,
    line: usize,
    env: &mut Environment,
    program: &Program,
) -> Exec<Value> {
    // TODO(you): spec 4.3: check arity, push a frame, declare the parameters, run the body, and
    //            pop the frame on all three ways out.

    let function = &program.functions[index];

    // println!(
    //     "calling function {}, expected params {}, args {}",
    //     function.name,
    //     function.params.len(),
    //     args.len()
    // );

    if args.len() != function.params.len() {
        return Err(err(
            line,
            &format!(
                "Expected {} arguments but got {}.",
                function.params.len(),
                args.len()
            ),
        ));
    }

    if env.frame_depth() >= MAX_FRAMES {
        return Err(err(line, "Stack overflow."));
    }

    env.push_frame();
    for (param, arg) in function.params.iter().zip(args) {
        // println!(
        //     "binding param {} to value {}",
        //     param,
        //     arg
        // );
        env.declare(param, arg);
    }

    // println!(
    //     "function body{}, statements",
    //     function.body.len()
    // );
    let mut outcome = Ok(Value::Nil);
    for statement in &function.body {
        match execute(statement, env, program) {
            Ok(()) => {}
            Err(Flow::Return(value)) => {
                outcome = Ok(value);
                break;
            }
            Err(Flow::Error(error)) => {
                outcome = Err(Flow::Error(error));
                break;
            }
        }
    }

    env.pop_frame();
    // println!(
    //     "finished function call {}",
    //     function.name
    // );
    outcome
}

fn numbers(l: Value, r: Value, line: usize) -> Exec<(f64, f64)> {
    match (l, r) {
        (Value::Number(a), Value::Number(b)) => Ok((a, b)),
        _ => Err(err(line, "Operands must be numbers.")),
    }
}

fn arith(l: Value, r: Value, line: usize, f: fn(f64, f64) -> f64) -> Exec<Value> {
    let (a, b) = numbers(l, r, line)?;
    Ok(Value::Number(f(a, b)))
}

fn compare(l: Value, r: Value, line: usize, f: fn(f64, f64) -> bool) -> Exec<Value> {
    let (a, b) = numbers(l, r, line)?;
    Ok(Value::Bool(f(a, b)))
}

fn undefined(name: &Token) -> Flow {
    err(
        name.line,
        &format!("Variable '{}' is not defined.", name.lexeme),
    )
}

fn err(line: usize, message: &str) -> Flow {
    Flow::Error(RuntimeError {
        line,
        message: message.to_string(),
    })
}
