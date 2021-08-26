use std::fs;
use std::io::{self, Write};
use std::path::Path;

use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
use inkwell::OptimizationLevel;

use crate::cli::NoshEmit;
// use crate::parse;
use crate::ast::Ptr;
use crate::ast_pass::debug::AstTermPrinter;
use crate::ast_pass::name_resolve::AstNameResolver;
use crate::ast_pass::ModulePass;
use crate::lex;
use crate::parse::ModuleGrammer;
use crate::parse::Parser;
// use crate::codegen::{CodeGenerator};
use crate::ast_pass::to_llvm::LlvmBackend;

pub fn build_file(filename: &str, opt_level: OptimizationLevel, emit: Option<NoshEmit>) {
    let path = Path::new(filename);
    let src = fs::read_to_string(&path).unwrap();

    let token_buffer = lex::scan_source(src.as_str());
    let tokens = Ptr::new(token_buffer);

    let mut p = match Parser::default(tokens) {
        Some(parser) => parser,
        None => {
            eprintln!("File {} is empty, nothing to do...", filename);
            return;
        }
    };

    let tried_module = p.expect(ModuleGrammer);

    let module = match tried_module {
        Ok(m) => m,
        Err(e) => {
            return {
                eprintln!("{:?}", e);
                for err in &p.errors {
                    eprintln!("{}", err)
                }
            }
        }
    };

    let sym_table = match AstNameResolver::default().run_pass(&module) {
        Ok(table) => table,
        Err(errs) => {
            for e in errs {
                eprintln!("{:?}", e);
            }
            return;
        }
    };

    match emit {
        Some(NoshEmit::Ast) => {
            let printer = AstTermPrinter::default();
            printer.print_module(&module);
        }
        Some(NoshEmit::Llvm) => {
            let ll = LlvmBackend::new(opt_level);
            let llmod = ll.compile_mod(module, &sym_table);
            let buffered_llvm = llmod.print_to_string();

            let mut stdout = io::stdout();
            stdout.write_all(buffered_llvm.to_bytes()).unwrap();
        }
        Some(NoshEmit::Asm) => {
            let ll = LlvmBackend::new(opt_level);
            let llmod = ll.compile_mod(module, &sym_table);

            let tm = prep_llvm_target_machine(opt_level);
            let buffered_asm = tm
                .write_to_memory_buffer(&llmod, FileType::Assembly)
                .expect("Failed to write ASM to memorry buffer");

            let mut stdout = io::stdout();
            stdout.write_all(buffered_asm.as_slice()).unwrap();
        }

        None => {
            let ll = LlvmBackend::new(opt_level);
            let llmod = ll.compile_mod(module, &sym_table);

            let savename_buf = path.with_extension("o");
            let savename = Path::new("./").join(savename_buf.file_name().unwrap());

            let tm = prep_llvm_target_machine(opt_level);
            tm.write_to_file(&llmod, FileType::Object, &savename)
                .expect("Failed to write object file");
        }
    }
}

fn prep_llvm_target_machine(opt_level: OptimizationLevel) -> TargetMachine {
    let init_config = InitializationConfig::default();
    Target::initialize_native(&init_config).expect("Failed to initalize default target");

    let reloc = RelocMode::Default;
    let model = CodeModel::Default;
    let triple = TargetMachine::get_default_triple();
    let cpu = TargetMachine::get_host_cpu_name();
    let cpu_info = TargetMachine::get_host_cpu_features();
    let target = Target::from_triple(&triple).expect("Failed to generate default triple");

    let target_machine = target
        .create_target_machine(
            &triple,
            cpu.to_str()
                .expect("TargetMachine::get_host_cpu_name() failed"),
            cpu_info
                .to_str()
                .expect("TargetMachine::get_host_cpu_features() failed"),
            opt_level,
            reloc,
            model,
        )
        .expect("Failed to generate default target machine");

    target_machine
}
