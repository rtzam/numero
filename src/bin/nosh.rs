
use std::io::stdout;

use nosh::cli;
use nosh::cli::build;
use nosh::cli::repl;
use nosh::cli::run;

fn main() {
    let cmd = cli::make_cli();

    // parse args here
    let matches = cmd.get_matches();

    // could panic but thats fine here
    // let verbose_level: i32 = matches.value_of("verbose").unwrap().parse().unwrap();

    match matches.subcommand() {
        ("build", Some(subm)) => {
            let filename = subm.value_of("FILE").unwrap();
            let opt_str = subm.value_of("optlevel").unwrap().parse().unwrap();
            let opt_level = cli::int_to_opt_level(opt_str);
            let emitter = cli::stdout_emission(subm.value_of("emit"));
            build::build_file(filename, opt_level, emitter)
        }
        ("repl", Some(_subm)) => repl::begin_repl(),
        ("run", Some(subm)) => {
            let filename = subm.value_of("FILE").unwrap();
            let opt_str = subm.value_of("optlevel").unwrap().parse().unwrap();
            let opt_level = cli::int_to_opt_level(opt_str);
            run::run_file(filename, opt_level)
        }
        ("check", Some(_subm)) => unimplemented!("No Checking yet..."),
        ("shell", Some(subm)) => {
            use clap::Shell;
            use std::str::FromStr;
            let shell_str = subm.value_of("shell").unwrap();
            let for_shell = Shell::from_str(shell_str).unwrap();
            let mut cmd = cli::make_cli();

            cmd.gen_completions_to(env!("CARGO_BIN_NAME"), for_shell, &mut stdout());
        },
        _ => unreachable!(),
    }
}
