
use crate::ast::Module;

pub mod name_resolve;

// pub trait GlobalPass{}
pub trait ModulePass<'s>{
    type Output;
    fn run_pass(self, m: &Module<'s>) -> Self::Output;
}
// pub trait FunctionPass{}