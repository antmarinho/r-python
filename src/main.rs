//use crate::ir::ast::Expression;
//use crate::ir::ast::Statement;
//use crate::interpreter::interpreter::eval;

/*
use crate::interpreter::interpreter::{execute, ControlFlow};
use crate::ir::ast::{Statement, Type};
use crate::parser::parser::parse;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;*/

pub mod environment;
pub mod interpreter;
pub mod ir;
pub mod parser;
pub mod pretty_print;
pub mod stdlib;
pub mod type_checker;

use crate::environment::environment::Environment;
use crate::interpreter::statement_execute::execute_block;
use crate::ir::ast::{Statement, Type};
use crate::parser::parse;
use crate::type_checker::statement_type_checker::check_stmt;
use crate::type_checker::typed_ast::annotate_match;

/// Pequeno driver para executar um programa R-Python de um arquivo.
/// Uso: `cargo run -- <caminho_para_arquivo.rpy>`
/// Se nenhum arquivo for passado, executa um programa de demonstração.
fn main() {
    let mut args = std::env::args().skip(1);
    // AST VIEWER: use `cargo run -- --ast arquivo.rpy` para mostrar
    // a conversão do código-fonte para a AST depois do parsing.
    let first_arg = args.next();
    let show_ast = first_arg.as_deref() == Some("--ast");
    let show_typed_ast = first_arg.as_deref() == Some("--typed-ast");
    let source_path = if show_ast || show_typed_ast {
        args.next()
    } else {
        first_arg
    };

    let source = if let Some(path) = source_path {
        std::fs::read_to_string(&path).unwrap_or_else(|e| {
            eprintln!("Erro ao ler {}: {}", path, e);
            std::process::exit(1);
        })
    } else {
        // Programa de demonstração básico
        "var x = 10; var y = 5; assert(x + y == 15, \"soma falhou\");".to_string()
    };

    let (rest, stmts) = match parse(&source) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Erro de parsing: {}", e);
            std::process::exit(1);
        }
    };

    if !rest.trim().is_empty() {
        eprintln!("Aviso: código não consumido: '{}'", rest);
    }

    if show_ast {
        println!("=== AST gerada pelo parser ===");
        println!("{:#?}", stmts);
        return;
    }

    if show_typed_ast {
        let mut type_env: Environment<Type> = Environment::new();
        let mut typed_matches = Vec::new();

        for statement in &stmts {
            // Primeiro valida a AST com o type checker. Só uma AST válida
            // pode ser convertida para a representação anotada com tipos.
            type_env = check_stmt(statement.clone(), &type_env).unwrap_or_else(|error| {
                eprintln!("Erro de type checking: {}", error);
                std::process::exit(1);
            });

            if let Statement::Match { value, cases } = statement {
                let typed_match = annotate_match(value, cases, &type_env).unwrap_or_else(|error| {
                    eprintln!("Erro ao anotar a AST: {}", error);
                    std::process::exit(1);
                });
                typed_matches.push(typed_match);
            }
        }

        println!("=== AST do Match aprovada pelo type checker ===");
        println!("{:#?}", typed_matches);
        return;
    }

    let env = Environment::new();
    match execute_block(stmts, &env) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Erro em tempo de execução: {}", e);
            std::process::exit(1);
        }
    }
}
