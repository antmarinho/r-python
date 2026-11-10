use r_python::ir::ast::{Expression, Pattern, Statement, Type, ValueConstructor};
use r_python::type_checker::check_stmt;
use r_python::environment::environment::Environment;

// ================================================================
// TESTES DO TYPE CHECKER DO MATCH
// Estes testes validam tipos dos padrões, capturas, duplicidade de
// variáveis e construtores ADT.
// ================================================================

fn empty_block() -> Box<Statement> {
    Box::new(Statement::Block(vec![]))
}

#[test]
fn accepts_literal_pattern_with_matching_type() {
    let mut env: Environment<Type> = Environment::new();
    env = check_stmt(
        Statement::VarDeclaration("value".to_string(), Box::new(Expression::CInt(1))),
        &env,
    )
    .unwrap();

    let stmt = Statement::Match {
        value: Box::new(Expression::Var("value".to_string())),
        cases: vec![(
            Pattern::Literal(Expression::CInt(1)),
            empty_block(),
        )],
    };

    assert!(check_stmt(stmt, &env).is_ok());
}

#[test]
fn rejects_literal_pattern_with_incompatible_type() {
    let mut env: Environment<Type> = Environment::new();
    env = check_stmt(
        Statement::VarDeclaration("value".to_string(), Box::new(Expression::CInt(1))),
        &env,
    )
    .unwrap();

    let stmt = Statement::Match {
        value: Box::new(Expression::Var("value".to_string())),
        cases: vec![(
            Pattern::Literal(Expression::CString("one".to_string())),
            empty_block(),
        )],
    };

    assert!(check_stmt(stmt, &env).is_err());
}

#[test]
fn makes_tuple_capture_available_inside_case_body() {
    let mut env: Environment<Type> = Environment::new();
    env = check_stmt(
        Statement::VarDeclaration(
            "value".to_string(),
            Box::new(Expression::Tuple(vec![
                Expression::CInt(1),
                Expression::CString("ok".to_string()),
            ])),
        ),
        &env,
    )
    .unwrap();

    let stmt = Statement::Match {
        value: Box::new(Expression::Var("value".to_string())),
        cases: vec![(
            Pattern::Tuple(vec![
                Pattern::Variable("number".to_string()),
                Pattern::Wildcard,
            ]),
            Box::new(Statement::Block(vec![Statement::ExprStmt(Box::new(
                Expression::Add(
                    Box::new(Expression::Var("number".to_string())),
                    Box::new(Expression::CInt(1)),
                ),
            ))])),
        )],
    };

    assert!(check_stmt(stmt, &env).is_ok());
}

#[test]
fn rejects_duplicate_capture_in_one_pattern() {
    let mut env: Environment<Type> = Environment::new();
    env = check_stmt(
        Statement::VarDeclaration(
            "value".to_string(),
            Box::new(Expression::Tuple(vec![Expression::CInt(1), Expression::CInt(2)])),
        ),
        &env,
    )
    .unwrap();

    let stmt = Statement::Match {
        value: Box::new(Expression::Var("value".to_string())),
        cases: vec![(
            Pattern::Tuple(vec![
                Pattern::Variable("x".to_string()),
                Pattern::Variable("x".to_string()),
            ]),
            empty_block(),
        )],
    };

    assert!(check_stmt(stmt, &env).is_err());
}

#[test]
fn accepts_adt_constructor_pattern_and_types_capture() {
    let mut env: Environment<Type> = Environment::new();
    env = check_stmt(
        Statement::TypeDeclaration(
            "Shape".to_string(),
            vec![ValueConstructor::new(
                "Circle".to_string(),
                vec![Type::TInteger],
            )],
        ),
        &env,
    )
    .unwrap();

    env = check_stmt(
        Statement::VarDeclaration(
            "shape".to_string(),
            Box::new(Expression::Constructor(
                "Circle".to_string(),
                vec![Box::new(Expression::CInt(10))],
            )),
        ),
        &env,
    )
    .unwrap();

    let stmt = Statement::Match {
        value: Box::new(Expression::Var("shape".to_string())),
        cases: vec![(
            Pattern::Constructor(
                "Circle".to_string(),
                vec![Pattern::Variable("radius".to_string())],
            ),
            Box::new(Statement::Block(vec![Statement::ExprStmt(Box::new(
                Expression::Add(
                    Box::new(Expression::Var("radius".to_string())),
                    Box::new(Expression::CInt(1)),
                ),
            ))])),
        )],
    };

    assert!(check_stmt(stmt, &env).is_ok());
}
