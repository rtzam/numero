
use std::fs;
use inkwell::OptimizationLevel;
// use crate::parse;
use crate::parse::{Parser, ModuleGrammer};
// use crate::codegen::{CodeGenerator};
use crate::ast_pass::ModulePass;
use crate::ast_pass::name_resolve::AstNameResolver;
use crate::ast_pass::to_llvm::LlvmBackend;

pub fn run_file(filename: &str, ol: OptimizationLevel){
    let src = fs::read_to_string(filename).unwrap();

    let mut p = Parser::default(src.as_str());
    
    let tried_module = p.expect(ModuleGrammer);

    let module = match tried_module{
        Ok(m) => m,
        Err(e) => return {
            eprintln!("{:?}", e);
            for err in &p.errors{
                eprintln!("{}", err)
            }
        },
    };

    let sym_table = match AstNameResolver::new().run_pass(&module){
        Ok(table) => table,
        Err(errs) => {
            for e in errs{
                eprintln!("{:?}", e);
            }
            return;
        }
    };

    // let printer = AstTermPrinter::default();
    // printer.print_module(&module);

    let ll = LlvmBackend::new(OptimizationLevel::None);

    let llmod = ll.compile_mod(module, &sym_table);

    let engine = llmod.create_jit_execution_engine(ol).unwrap();

    match llmod.get_function("main"){
        Some(fun) => {
            let ret_val = unsafe{
                engine.run_function(fun, &[]) // _as_main
            };
            let f64_ty = ll.context.f64_type();
            println!("{}", ret_val.as_float(&f64_ty));
        }
        None => {
            eprintln!("No function 'main' to to begin execution")
        }
    }
    
}