use crate::{
    ast::stmt::Stmt,
    lexer::Lexer,
    parser::{Parser, RizonResParser},
};

use super::test_parser::{StmtInfos, ExprInfos, TestParser};

pub fn lex_and_parse(code: &str) -> Result<Vec<Stmt>, Vec<RizonResParser>> {
    let mut lexer = Lexer::new();
    let tokens = lexer.tokenize(code).unwrap();
    let mut parser = Parser::default();
    parser.parse(tokens)
}

pub fn get_nodes_infos(code: &str) -> StmtInfos {
    let nodes = lex_and_parse(code).unwrap();
    let mut test_parser = TestParser::default();
    test_parser.get_all_infos(&nodes).unwrap().clone()
}

pub fn get_stmt_nodes_infos(code: &str) -> StmtInfos {
    let nodes = lex_and_parse(code).unwrap();
    let mut test_parser = TestParser::default();
    test_parser.get_all_infos(&nodes).unwrap().clone()
}

pub fn get_expr_nodes_infos(code: &str) -> ExprInfos {
    let nodes = lex_and_parse(code).unwrap();
    let mut test_parser = TestParser::default();
    test_parser.get_all_infos(&nodes).unwrap().expr.clone()
}
