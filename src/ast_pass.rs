
use crate::ast::Module;

pub mod name_resolve;
pub mod debug;
pub mod to_llvm;

// pub trait GlobalPass{}
pub trait ModulePass<'s>{
    type Output;
    fn run_pass(self, m: &Module<'s>) -> Self::Output;
}
// pub trait FunctionPass{}