

use rustyline::error::ReadlineError;
use rustyline::Editor;


use crate::parse;
use crate::parse::{Parser, RecoveryInfo};
use crate::ast::debug::AstTermPrinter;


pub struct ReplState{
    line: String,
    collecting: bool,
}

impl ReplState{
    pub fn new() -> Self{
        Self{
            line: String::new(),
            collecting: false,
        }
    }
    pub fn append_line(&mut self, s: &str){
        self.line.push('\n');
        self.line.push_str(s);
        eprintln!("Buffered {:?}", self.line);
    }
    fn clear_line(&mut self){
        self.line.clear()
    }
    pub fn as_str(&self) -> &str{
        self.line.as_str()
    }
    pub fn reset(&mut self){
        self.clear_line();
        self.collecting = false;
    }

    pub fn collect(&mut self){
        self.collecting = true;
    }
}


pub fn begin_repl(){
    eprintln!("Nosh REPL");
    
    let mut rl = Editor::<()>::new();
    // if rl.load_history("history.txt").is_err() {
    //     println!("No previous history.");
    // }
    let printer = AstTermPrinter::default();
    let config = parse::ParseConfig::new(parse::ParseMode::Repl);
    
    let mut repl = ReplState::new();

    loop {
        let linefeed = match repl.collecting{
            true => "| ",
            false => ">> ",
        };
        match rl.readline(linefeed) {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                repl.append_line(line.as_str());
                // read from REPL stored buffer
                let mut p = Parser::new(repl.as_str(), config.clone());

                let tried_items = parse::parse_repl_line(&mut p);
                
                match tried_items{
                    Ok(items) => {
                        for item in items{
                            printer.print_item(&item);
                        }
                    },
                    Err(RecoveryInfo::EarlyEOF) => {
                        repl.collect();
                        continue
                    },
                    _ => {
                        for err in &p.errors{
                            eprintln!("{}", err)
                        }
                    }
                };

                repl.reset();
            },
            Err(ReadlineError::Interrupted) => {
                eprintln!("CTRL-C");
                break
            },
            Err(ReadlineError::Eof) => {
                eprintln!("CTRL-D");
                break
            },
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break
            }
        }
        
    }
}