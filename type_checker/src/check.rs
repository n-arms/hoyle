use crate::env::*;
use crate::infer;
use tree::parsed;
use tree::typed::*;

pub fn expr(env: &Env, expr: &parsed::Expr, want: &Type) -> Result<Expr> {
    let typed = infer::expr(env, expr)?;
    if &typed.get_type() == want {
        Ok(typed)
    } else {
        Err(Error::TypeMismatch {
            got: typed.get_type(),
            expected: want.clone(),
        })
    }
}
