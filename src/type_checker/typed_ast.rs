use crate::environment::environment::Environment;
use crate::ir::ast::{Expression, Name, Pattern, Statement, Type};
use crate::type_checker::expression_type_checker::check_expr;

/// Representação de um padrão depois de receber a informação de tipo.
/// A AST original não é alterada; esta estrutura é usada somente para
/// inspeção e impressão após o type checking.
#[derive(Debug, Clone)]
pub struct TypedPattern {
    pub pattern: Pattern,
    pub pattern_type: Type,
    pub children: Vec<TypedPattern>,
}

/// Um caso de Match com o padrão anotado e seu corpo original.
#[derive(Debug, Clone)]
pub struct TypedCase {
    pub pattern: TypedPattern,
    pub body: Box<Statement>,
}

/// Representação anotada de um Match para visualização.
#[derive(Debug, Clone)]
pub struct TypedMatch {
    pub value: Expression,
    pub value_type: Type,
    pub cases: Vec<TypedCase>,
}

/// Anota os padrões de um Match usando o ambiente já validado pelo type checker.
pub fn annotate_match(
    value: &Expression,
    cases: &[(Pattern, Box<Statement>)],
    env: &Environment<Type>,
) -> Result<TypedMatch, String> {
    let value_type = check_expr(value.clone(), env)?;
    let mut typed_cases = Vec::with_capacity(cases.len());

    for (pattern, body) in cases {
        let typed_pattern = annotate_pattern(pattern, &value_type, env)?;
        typed_cases.push(TypedCase {
            pattern: typed_pattern,
            body: body.clone(),
        });
    }

    Ok(TypedMatch {
        value: value.clone(),
        value_type,
        cases: typed_cases,
    })
}

fn annotate_pattern(
    pattern: &Pattern,
    expected_type: &Type,
    env: &Environment<Type>,
) -> Result<TypedPattern, String> {
    match pattern {
        Pattern::Literal(expression) => {
            let pattern_type = check_expr(expression.clone(), env)?;
            Ok(TypedPattern {
                pattern: pattern.clone(),
                pattern_type,
                children: Vec::new(),
            })
        }
        Pattern::Variable(_) | Pattern::Wildcard => Ok(TypedPattern {
            pattern: pattern.clone(),
            pattern_type: expected_type.clone(),
            children: Vec::new(),
        }),
        Pattern::Tuple(patterns) => {
            let element_types = match expected_type {
                Type::TTuple(types) => types.clone(),
                Type::TAny => vec![Type::TAny; patterns.len()],
                other => {
                    return Err(format!(
                        "Não foi possível anotar tupla com tipo '{}'.",
                        other
                    ))
                }
            };

            let mut children = Vec::with_capacity(patterns.len());
            for (pattern, element_type) in patterns.iter().zip(element_types.iter()) {
                children.push(annotate_pattern(pattern, element_type, env)?);
            }

            Ok(TypedPattern {
                pattern: pattern.clone(),
                pattern_type: expected_type.clone(),
                children,
            })
        }
        Pattern::List(patterns) => {
            let element_type = match expected_type {
                Type::TList(element_type) => element_type.as_ref().clone(),
                Type::TAny => Type::TAny,
                other => {
                    return Err(format!(
                        "Não foi possível anotar lista com tipo '{}'.",
                        other
                    ))
                }
            };

            let mut children = Vec::with_capacity(patterns.len());
            for pattern in patterns {
                children.push(annotate_pattern(pattern, &element_type, env)?);
            }

            Ok(TypedPattern {
                pattern: pattern.clone(),
                pattern_type: expected_type.clone(),
                children,
            })
        }
        Pattern::Constructor(name, patterns) => {
            let argument_types = constructor_argument_types(name, expected_type, env)?;
            let mut children = Vec::with_capacity(patterns.len());

            for (pattern, argument_type) in patterns.iter().zip(argument_types.iter()) {
                children.push(annotate_pattern(pattern, argument_type, env)?);
            }

            Ok(TypedPattern {
                pattern: pattern.clone(),
                pattern_type: expected_type.clone(),
                children,
            })
        }
    }
}

fn constructor_argument_types(
    name: &Name,
    expected_type: &Type,
    env: &Environment<Type>,
) -> Result<Vec<Type>, String> {
    match name.as_str() {
        "Just" => match expected_type {
            Type::TMaybe(inner) => Ok(vec![inner.as_ref().clone()]),
            Type::TAny => Ok(vec![Type::TAny]),
            other => Err(format!("Just não pode analisar o tipo '{}'.", other)),
        },
        "Nothing" => Ok(Vec::new()),
        "Ok" => match expected_type {
            Type::TResult(ok, _) => Ok(vec![ok.as_ref().clone()]),
            Type::TAny => Ok(vec![Type::TAny]),
            other => Err(format!("Ok não pode analisar o tipo '{}'.", other)),
        },
        "Err" => match expected_type {
            Type::TResult(_, error) => Ok(vec![error.as_ref().clone()]),
            Type::TAny => Ok(vec![Type::TAny]),
            other => Err(format!("Err não pode analisar o tipo '{}'.", other)),
        },
        _ => {
            let (adt_name, constructor) = find_constructor(name, env).ok_or_else(|| {
                format!("Construtor '{}' não foi encontrado.", name)
            })?;

            let belongs_to_adt = match expected_type {
                Type::TAny => true,
                Type::TAlgebraicData(type_name, _) => type_name == &adt_name,
                _ => false,
            };

            if !belongs_to_adt {
                return Err(format!(
                    "Construtor '{}' não pertence ao tipo '{}'.",
                    name, expected_type
                ));
            }

            Ok(constructor.types)
        }
    }
}

fn find_constructor(
    name: &Name,
    env: &Environment<Type>,
) -> Option<(Name, crate::ir::ast::ValueConstructor)> {
    for scope in env.stack.iter() {
        for (adt_name, constructors) in &scope.adts {
            if let Some(constructor) = constructors.iter().find(|constructor| &constructor.name == name) {
                return Some((adt_name.clone(), constructor.clone()));
            }
        }
    }

    for (adt_name, constructors) in &env.globals.adts {
        if let Some(constructor) = constructors.iter().find(|constructor| &constructor.name == name) {
            return Some((adt_name.clone(), constructor.clone()));
        }
    }

    None
}
