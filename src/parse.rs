


use crate::ast::{Expr, ExprKind};
use crate::ast::node::NodeId;
use crate::ast::ops::BinaryOp;
use crate::ast::token::{TokenData, Token, };
use crate::lex::{TokenStream, TokTag};

// mod gram;
mod op_prec;
mod syntax;

pub use syntax::{ModuleGrammer, ReplGrammer};

use syntax::{Syntax,};
use op_prec::BinOpPrec;
// pub use gram::{parse_module, parse_repl_line};

pub type SyntaxErrorMsg = String;

// information needed for caller function to recover from
// parse error
// example: REPL may use EarlyEOF to wait for next line of text
#[derive(Debug, Clone,)]
pub enum RecoveryInfo{
    EarlyEOF,
    InvalidOp,
    InvalidToken(String),
    CompilerBug,
}


pub type ParseResult<T> = Result<T, RecoveryInfo>;


#[derive(Debug, Clone, Copy)]
pub enum ParseMode{
    File,
    Repl,
}

#[derive(Debug, Clone)]
pub struct ParseConfig<'s>{
    op_prec: BinOpPrec<'s>,
    mode: ParseMode,
}

impl<'s> ParseConfig<'s>{
    pub fn new(m: ParseMode) -> Self{
        Self{
            op_prec: BinOpPrec::init(),
            mode: m,
        }
    }
    pub fn default() -> Self{
        Self::new(ParseMode::File)
    }
}



pub struct Parser<'s>{
    nid: NodeId,
    current_tag: Option<TokTag>,
    next_tag: Option<TokTag>,
    tok_stream: TokenStream<'s>,
    config: ParseConfig<'s>,
    pub errors: Vec<SyntaxErrorMsg>,
}

impl<'s> Parser<'s>{
    pub fn new(src: &'s str, config: ParseConfig<'s>) -> Self{

        let mut ts = TokenStream::new(src);
        Self{
            nid: NodeId::new(),
            current_tag: ts.next(),
            next_tag: ts.next(),
            tok_stream: ts,
            config: config,
            errors: Vec::new(),
        }
    }

    pub fn default(src: &'s str) -> Self{
        Self::new(src, ParseConfig::default())
    }

    fn report_error(&mut self, rs: RecoveryInfo, msg: String) -> RecoveryInfo{
        self.errors.push(msg);
        rs
    }

    fn peek(&self) -> Option<Token>{
        // let td = self.current_tok?;
        let td = self.peek_data()?;
        Some(td.kind)
    }
    
    fn peek_data(&self) -> Option<&TokenData<'s>>{
        // &self.current_tok
        self.tok_stream.get_token(self.current_tag?)
    }

    fn copy_current_tok(&mut self) -> Option<TokenData<'s>>{
        let tok = self.peek_data()?;
        Some(*tok)
    }

    fn peek_next(&mut self) -> Option<&TokenData<'s>>{
        self.tok_stream.get_token(self.next_tag?)
    }

    fn shift(&mut self) -> Option<TokenData<'s>>{
        let temp = self.copy_current_tok();
        self.current_tag = self.next_tag;
        self.advance();
        temp
    }

    fn advance(&mut self) -> Option<()>{
        let current_td = *self.peek_next()?;

        loop {
            self.next_tag = self.tok_stream.next();

            let next_tt = self.next_tag?;
            let next_td = self.tok_stream.get_token(next_tt)?;
            // eprintln!("lex: {:?}, {:?}", next_td, next_tt);
            match (current_td.kind, next_td.kind) {
                // skip new line
                // TODO: binary ops only
                (Token::Sigil, Token::Newline) =>{
                    continue
                }
                _ => {
                    // eprintln!("lex: {:?}", self.tok_stream.get_current_tok_tag());
                    break Some(());
                }
            }
        }
    }

    fn get_precedence(&self, bo: BinaryOp) -> i32{
        self.config.op_prec.get_precedence(bo)
    }

    fn get_op_from_span(&self, span: &'s str) -> Option<BinaryOp>{
        let op = self.config.op_prec.get_op_from_span(span)?;
        Some(op.clone())
    }

    fn get_precedence_from_span(&self, span: &'s str) -> Option<i32>{
        let op = self.get_op_from_span(span)?;
        Some(self.get_precedence(op))
    }

    fn peek_op_precedence(&mut self) -> PeekOpPrec{
        match self.peek_data(){
            Some(tok) if tok.kind == Token::Sigil || tok.kind == Token::Assigner => 
            {
                let span = tok.span;
                match self.get_precedence_from_span(span){
                    Some(prec) => PeekOpPrec::Prec(prec),
                    _ => PeekOpPrec::BadOp,
                }
            }
            _ => PeekOpPrec::ExprEnd,
        }
    }


    fn new_expr(&mut self, kind: ExprKind<'s>) -> Expr<'s>{
        Expr::new(self.nid.shift(), kind)
    }

    fn check<S: Syntax<'s>>(&self, s: S) -> bool{
        s.check(self)
    }
    pub fn expect<S: Syntax<'s>>(&mut self, s: S) -> ParseResult<S::Parsed>{
        s.expect(self)
    }
    fn parse_if_present<S: Syntax<'s>>(&mut self, s: S) -> Option<ParseResult<S::Parsed>>{
        if s.check(self){
            let parsed = s.expect(self); 
            Some(parsed)
        } else{
            None
        }
    }
}


pub enum PeekOpPrec{
    Prec(i32),
    BadOp,
    ExprEnd,
}