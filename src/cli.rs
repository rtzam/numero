pub mod build;
pub mod repl;
pub mod run;

use inkwell::OptimizationLevel;

use clap::{App, AppSettings, Arg, SubCommand, Shell};

pub fn int_to_opt_level(level: u8) -> OptimizationLevel {
    match level {
        0 => OptimizationLevel::None,
        1 => OptimizationLevel::Less,
        2 => OptimizationLevel::Default,
        3 => OptimizationLevel::Aggressive,
        _ => OptimizationLevel::Aggressive,
    }
}

// values to print to stdout
// instead of completely compiling
pub enum NoshEmit {
    Ast,
    Asm,
    Llvm,
}

pub fn stdout_emission(s: Option<&str>) -> Option<NoshEmit> {
    match s? {
        "ast" => Some(NoshEmit::Ast),
        "asm" => Some(NoshEmit::Asm),
        "llvm" => Some(NoshEmit::Llvm),
        _ => None,
    }
}

trait BuildFileCli<'a, 'b> {
    fn append_build_file_args(self) -> Self;
}

impl<'a, 'b> BuildFileCli<'a, 'b> for App<'a, 'b> {
    fn append_build_file_args(self) -> Self {
        self.arg(
            Arg::with_name("FILE")
                .required(true)
                .help("input file to use"),
        )
        .arg(
            Arg::with_name("optlevel")
                .long("opt-level")
                .short("O")
                .default_value("0")
                .possible_values(&["0", "1", "2", "3"]),
        )
    }
}


pub fn make_cli<'a, 'b>() -> App<'a, 'b> {
    App::new("The Nosh Compiler")
        .version(env!("CARGO_PKG_VERSION"))
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .takes_value(true)
                .value_name("level")
                .default_value("0")
                .help("display increasing levels of output"))
        .subcommand(
            SubCommand::with_name("build")
                .alias("b")
                .about("Compile program to object file")
                .arg(
                    Arg::with_name("emit")
                        .long("emit")
                        .takes_value(true)
                        .possible_values(&["ast", "asm", "llvm"]),
                )
                .append_build_file_args())
        .subcommand(
            SubCommand::with_name("repl")
                    .about("Run Interactive REPL"))
        .subcommand(
            SubCommand::with_name("run")
                .alias("r")
                .about("Execute program using LLVM JIT")
                .append_build_file_args())
        .subcommand(
            SubCommand::with_name("check")
                .about("Check program correctness without generating object file"))
        .subcommand(
            SubCommand::with_name("shell")
                .about("Generate shell completion instructions for nosh")
                .arg(
                    Arg::with_name("shell")
                    .required(true)
                    .possible_values(&Shell::variants())))
}
