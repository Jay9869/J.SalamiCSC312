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
    todo!("evaluate")
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
    todo!("call")
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
