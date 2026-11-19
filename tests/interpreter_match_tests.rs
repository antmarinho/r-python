use r_python::environment::environment::Environment;
use r_python::interpreter::statement_execute::{execute, Computation};
use r_python::ir::ast::{Expression, Pattern, Statement};

// IMPLEMENTAÇÃO DO MATCH - TESTES DO INTERPRETADOR:
// estes testes não validam parser nem type checker; eles executam a AST
// diretamente e verificam a seleção dos casos e as capturas em runtime.

fn resulting_env(computation: Computation) -> Environment<Expression> {
    match computation {
        Computation::Continue(env)
        | Computation::Return(_, env)
        | Computation::PropagateError(_, env)
        | Computation::BreakLoop(env)
        | Computation::ContinueLoop(env) => env,
    }
}

fn env_with_result() -> Environment<Expression> {
    let mut env = Environment::new();
    env.map_variable("result".to_string(), true, Expression::CInt(0));
    env
}

#[test]
fn executes_first_matching_literal_case() {
    let stmt = Statement::Match {
        value: Box::new(Expression::CInt(2)),
        cases: vec![
            (
                Pattern::Literal(Expression::CInt(1)),
                Box::new(Statement::Assignment(
                    "result".to_string(),
                    Box::new(Expression::CInt(10)),
                )),
            ),
            (
                Pattern::Literal(Expression::CInt(2)),
                Box::new(Statement::Assignment(
                    "result".to_string(),
                    Box::new(Expression::CInt(20)),
                )),
            ),
        ],
    };

    let env = resulting_env(execute(stmt, &env_with_result()).unwrap());
    assert_eq!(env.lookup(&"result".to_string()).unwrap().1, Expression::CInt(20));
}

#[test]
fn captures_tuple_variable_for_case_body() {
    let stmt = Statement::Match {
        value: Box::new(Expression::Tuple(vec![
            Expression::CInt(4),
            Expression::CString("ok".to_string()),
        ])),
        cases: vec![
            (
                Pattern::Tuple(vec![
                    Pattern::Variable("number".to_string()),
                    Pattern::Wildcard,
                ]),
                Box::new(Statement::Assignment(
                    "result".to_string(),
                    Box::new(Expression::Add(
                        Box::new(Expression::Var("number".to_string())),
                        Box::new(Expression::CInt(1)),
                    )),
                )),
            ),
        ],
    };

    let env = resulting_env(execute(stmt, &env_with_result()).unwrap());
    assert_eq!(env.lookup(&"result".to_string()).unwrap().1, Expression::CInt(5));
    assert!(env.lookup(&"number".to_string()).is_none());
}

#[test]
fn matches_constructor_and_captures_argument() {
    let stmt = Statement::Match {
        value: Box::new(Expression::Constructor(
            "Circle".to_string(),
            vec![Box::new(Expression::CInt(10))],
        )),
        cases: vec![
            (
                Pattern::Constructor(
                    "Circle".to_string(),
                    vec![Pattern::Variable("radius".to_string())],
                ),
                Box::new(Statement::Assignment(
                    "result".to_string(),
                    Box::new(Expression::Add(
                        Box::new(Expression::Var("radius".to_string())),
                        Box::new(Expression::CInt(1)),
                    )),
                )),
            ),
        ],
    };

    let env = resulting_env(execute(stmt, &env_with_result()).unwrap());
    assert_eq!(env.lookup(&"result".to_string()).unwrap().1, Expression::CInt(11));
    assert!(env.lookup(&"radius".to_string()).is_none());
}

#[test]
fn returns_error_when_no_case_matches() {
    let stmt = Statement::Match {
        value: Box::new(Expression::CInt(3)),
        cases: vec![(
            Pattern::Literal(Expression::CInt(1)),
            Box::new(Statement::Block(vec![])),
        )],
    };

    let result = execute(stmt, &Environment::new());
    match result {
        Err(error) => assert_eq!(error, "No match case matched the value"),
        Ok(_) => panic!("expected the match to fail"),
    }
}
