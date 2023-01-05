use ir::desugared::*;

pub struct Block<'names, 'expr> {
    statements: Vec<Statement<'expr>>,
    names: &'names mut NameSource,
}

impl<'names, 'expr> Block<'names, 'expr> {
    pub fn reset(&mut self) {
        self.statements.clear();
    }

    pub fn new(names: &'names mut NameSource) -> Self {
        Self {
            statements: Vec::new(),
            names,
        }
    }

    pub fn fresh_name(&mut self) -> Name {
        self.names.fresh()
    }

    pub fn with_statement(&mut self, statement: Statement<'expr>) -> &mut Self {
        self.statements.push(statement);
        self
    }
}
