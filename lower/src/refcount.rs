use im::HashSet;
use ir::bridge::{Block, Instr};

pub fn insert_counts(block: Block) -> Block {
    let mut seen = HashSet::new();
    let mut instrs = Vec::new();
    for instr in block.instrs {
        match instr.clone() {
            Instr::Copy {
                target,
                value,
                witness,
            } => {
                if seen.contains(&target.name) {
                    seen.insert(value.name.clone());
                    if !seen.contains(&value.name) {
                        instrs.push(Instr::Destory {
                            value: value.clone(),
                        });
                        seen.insert(value.name.clone());
                    }
                    if let Some(witness) = witness.as_ref() {
                        seen.insert(witness.name.clone());
                    }
                    instrs.push(Instr::Copy {
                        target,
                        value,
                        witness,
                    });
                }
            }
            Instr::Set { target, .. } => {
                if seen.contains(&target.name) {
                    instrs.push(instr);
                }
            }
            _ => {
                instrs.push(instr);
            }
        }
    }
    Block { instrs }
}
