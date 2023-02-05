use arena_alloc::General;
use ir::desugared::{self, Atom, FunctionDefinition, Statement, StructDefinition};
use ir::qualified::{LocalTagSource, Tag};

pub struct Program<'names, 'expr> {
    structs: Vec<StructDefinition<'expr>>,
    functions: Vec<FunctionDefinition<'expr>>,
    pub names: LocalTagSource<'names>,
}

impl<'names, 'expr> Program<'names, 'expr> {
    #[must_use]
    pub const fn new(names: LocalTagSource<'names>) -> Self {
        Self {
            structs: Vec::new(),
            functions: Vec::new(),
            names,
        }
    }

    pub fn build(&mut self, alloc: &General<'expr>) -> desugared::Program<'expr> {
        let program = desugared::Program {
            functions: alloc.alloc_slice_fill_iter(self.functions.iter().copied()),
            structs: alloc.alloc_slice_fill_iter(self.structs.iter().copied()),
        };

        self.structs.clear();
        self.functions.clear();

        program
    }

    pub fn with_function(&mut self, function: FunctionDefinition<'expr>) -> &mut Self {
        self.functions.push(function);
        self
    }

    pub fn with_struct(&mut self, r#struct: StructDefinition<'expr>) -> &mut Self {
        self.structs.push(r#struct);
        self
    }
}

pub struct Block<'names, 'expr> {
    statements: Vec<Statement<'expr>>,
    names: LocalTagSource<'names>,
}

impl<'names, 'expr> Block<'names, 'expr> {
    #[must_use]
    pub const fn new(names: LocalTagSource<'names>) -> Self {
        Self {
            statements: Vec::new(),
            names,
        }
    }

    pub fn fresh_tag(&mut self) -> Tag {
        self.names.fresh_tag()
    }

    pub fn with_statement(&mut self, statement: Statement<'expr>) -> &mut Self {
        self.statements.push(statement);
        self
    }

    pub fn build(&mut self, result: Atom, alloc: &General<'expr>) -> desugared::Block<'expr> {
        let block = desugared::Block {
            statements: alloc.alloc_slice_fill_iter(self.statements.iter().copied()),
            result,
        };

        self.statements.clear();

        block
    }
}
