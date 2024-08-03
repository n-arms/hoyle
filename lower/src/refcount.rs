use im::{hashset, HashSet};
use ir::bridge::{Block, Convention, Expr, Function, Instr, Value, Variable, Witness};

pub fn count_function(function: Function) -> Function {
    let args = function
        .arguments
        .iter()
        .map(|arg| arg.name.clone())
        .collect();
    let block = count_block(
        function.body.clone(),
        function.body.instrs.last().unwrap().target.clone(),
        args,
    );
    Function {
        name: function.name,
        arguments: function.arguments,
        body: block,
        names: function.names,
    }
}

fn count_block(mut block: Block, result: Variable, written: HashSet<Variable>) -> Block {
    dbg!(&result);
    let size = block.instrs.len();
    let uses: Vec<_> = block
        .instrs
        .iter()
        .map(|instr| find_instr_uses(instr))
        .collect();
    dbg!(&uses);
    let mut read = HashSet::new();
    let mut first_writes = vec![HashSet::new(); size];
    first_writes[0] = uses[0].writes.clone().union(written);
    for i in 1..size {
        first_writes[i] = first_writes[i - 1].clone().union(uses[i].writes.clone());
    }
    dbg!(&first_writes);

    let mut last_reads: Vec<_> = uses
        .into_iter()
        .zip(first_writes)
        .rev()
        .map(|(uses, written)| {
            let mut first_read = HashSet::new();
            for to_read in uses.reads {
                if !read.contains(&to_read) && written.contains(&to_read) {
                    first_read.insert(to_read.clone());
                    read.insert(to_read);
                }
            }
            first_read
        })
        .collect();
    last_reads.reverse();
    for (i, reads) in last_reads.into_iter().enumerate().rev() {
        let instr = &mut block.instrs[i];
        if let Expr::If {
            true_branch,
            false_branch,
            ..
        } = &mut instr.value
        {
            *true_branch = count_block(true_branch.clone(), instr.target.clone(), reads.clone());
            *false_branch = count_block(false_branch.clone(), instr.target.clone(), reads);
        } else {
            for to_destroy in reads {
                block
                    .instrs
                    .insert(i + 1, Instr::new(to_destroy, Expr::Destroy));
            }
        }
    }
    block
}

#[derive(Clone, Debug, Default)]
struct VariableUses {
    writes: HashSet<Variable>,
    reads: HashSet<Variable>,
    destroys: HashSet<Variable>,
}

impl VariableUses {
    pub fn extend(&mut self, other: Self) {
        self.writes.extend(other.writes);
        self.reads.extend(other.reads);
        self.destroys.extend(other.destroys);
    }

    pub fn read(&mut self, variable: Variable) {
        self.read_witness(&variable);
        self.reads.insert(variable);
    }

    pub fn destroy(&mut self, variable: Variable) {
        self.read_witness(&variable);
        self.destroys.insert(variable);
    }

    pub fn write(&mut self, variable: Variable) {
        self.read_witness(&variable);
        self.writes.insert(variable);
    }

    pub fn remove_write(&mut self, variable: &Variable) {
        self.writes.remove(variable);
    }

    fn read_witness(&mut self, variable: &Variable) {
        if let Witness::Dynamic { location } = variable.witness.as_ref() {
            self.read(location.clone());
        }
    }
}

fn find_uses(block: &Block, result: Variable) -> VariableUses {
    let mut uses = VariableUses::default();
    for instr in &block.instrs {
        uses.extend(find_instr_uses(instr));
    }
    uses.read(result);
    uses
}

fn find_instr_uses(instr: &Instr) -> VariableUses {
    let mut uses = VariableUses {
        writes: hashset![instr.target.clone()],
        ..Default::default()
    };
    match &instr.value {
        Expr::Literal(_) => {}
        Expr::Primitive(_, arguments) => {
            for arg in arguments {
                uses.read(arg.clone());
            }
        }
        Expr::CallDirect { arguments, .. } => {
            for arg in arguments {
                match (arg.convention, &arg.value) {
                    (Convention::In, Value::Move(src)) => {
                        uses.destroy(src.clone());
                    }
                    (Convention::In, Value::Copy(src)) => {
                        uses.read(src.clone());
                    }
                    (Convention::Inout, Value::Move(_)) => todo!(),
                    (Convention::Inout, Value::Copy(_)) => todo!(),
                    (Convention::Out, Value::Move(_)) => todo!(),
                    (Convention::Out, Value::Copy(src)) => {
                        uses.write(src.clone());
                    }
                }
            }
        }
        Expr::Value(value) => {
            uses.extend(find_value_uses(&value));
        }
        Expr::Destroy => {
            uses.destroy(instr.target.clone());
            uses.remove_write(&instr.target);
        }
        Expr::StructPack { arguments, .. } => {
            for arg in arguments {
                uses.extend(find_value_uses(&arg.value));
            }
        }
        Expr::If {
            predicate,
            true_branch,
            false_branch,
        } => {
            uses.read(predicate.clone());
            let true_uses = find_uses(true_branch, instr.target.clone());
            let false_uses = find_uses(false_branch, instr.target.clone());
            uses.extend(true_uses);
            uses.extend(false_uses);
        }
        Expr::Unpack { value, .. } => {
            uses.read(value.clone());
        }
        Expr::MakeClosure { env, witness, .. } => {
            find_value_uses(env);
            find_value_uses(witness);
        }
    }
    uses
}

fn find_value_uses(value: &Value) -> VariableUses {
    let mut uses = VariableUses::default();
    match value {
        Value::Move(src) => {
            uses.destroy(src.clone());
        }
        Value::Copy(src) => {
            uses.read(src.clone());
        }
    }
    uses
}
