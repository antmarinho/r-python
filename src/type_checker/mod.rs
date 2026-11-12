pub mod expression_type_checker;
pub mod statement_type_checker;
pub mod typed_ast;

pub use expression_type_checker::check_expr;
pub use statement_type_checker::check_stmt;
