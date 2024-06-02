use crate::parser::parser;

pub struct Visitor<'a> {
    pub ast_tree: parser::ParserReturn<'a>,
}
