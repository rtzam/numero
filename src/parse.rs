


use crate::ast::{Expr, ExprKind, Ptr};
use crate::ast::node::NodeId;
use crate::ast::ops::BinaryOp;
use crate::ast::token::{TokenData, Token, };

// mod gram;
mod op_prec;
mod syntax;

pub use syntax::{ModuleGrammer, ReplGrammer};

use syntax::{Syntax,};
use op_prec::BinOpPrec;

pub type SyntaxErrorMsg = String;


// pub enum SyntaxError{
//     InvalidBinaryOp(TokTag),
//     InvalidUnaryOp(TokTag),
//     AssignOpMustBeFirstOp(TokTag),
//     UseOfVarNotVal(TokTag),
//     NoLineEndAfterLet(TokTag),
//     KwInvalidStartOfStmt(TokTag),
// }

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
    current_tag: usize,
    // next_tag: usize, // currently unneeded to parse grammer
    tokens: Ptr<Vec<TokenData<'s>>>,
    config: ParseConfig<'s>,
    pub errors: Vec<SyntaxErrorMsg>,
}

impl<'s> Parser<'s>{
    pub fn new(config: ParseConfig<'s>, tokens: Ptr<Vec<TokenData<'s>>>) -> Option<Self>{

        let current_tag = find_starting_token_idx(&tokens)?;
        // let next_tag = find_next_token_idx(current_tag, &tokens)?;

        Some(Self{
            nid: NodeId::new(),
            current_tag: current_tag,
            // next_tag: next_tag,
            tokens: tokens,
            config: config,
            errors: Vec::new(),
        })
    }

    pub fn default(tokens: Ptr<Vec<TokenData<'s>>>) -> Option<Self>{
        Self::new( ParseConfig::default(), tokens)
    }

    fn report_error(&mut self, rs: RecoveryInfo, msg: String) -> RecoveryInfo{
        self.errors.push(msg);
        rs
    }

    fn peek(&self) -> Option<&TokenData<'s>>{
        self.tokens.get(self.current_tag)
    }

    // fn peek_next(&mut self) -> Option<&TokenData<'s>>{
    //     self.tokens.get(self.next_tag)
    // }

    fn shift(&mut self){
        // let current_idx = self.next_tag;

        // self.current_tag = current_idx;
        // self.next_tag = match find_next_token_idx(current_idx, &self.tokens){
        //     Some(idx) => idx,
        //     None => self.tokens.len() // ensures out of range index so all lookups will return EOF
        // };

        let next_idx = match find_next_token_idx(self.current_tag, &self.tokens){
            Some(idx) => idx,
            None => self.tokens.len() // ensures out of range index so all lookups will return EOF
        };

        self.current_tag = next_idx;
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
        match self.peek(){
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
            Some(s.expect(self))
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


fn find_starting_token_idx<'s>(tokens: &Vec<TokenData<'s>>) -> Option<usize>{
    let mut index = 0;

    loop {
        let next_td = tokens.get(index)?;

        // eprintln!("lex: {:?}, {:?}", next_td, next_tt);
        match next_td.kind{
            // skip new line for multiline expresssions
            // TODO: binary ops only
            Token::Newline | Token::Whitespace | Token::EOLComment => {
                index += 1;
            },
            _ => break,
        }
    }

    Some(index)
}

fn find_next_token_idx<'s>(current_idx: usize, tokens: &Vec<TokenData<'s>>) -> Option<usize>{
    let mut idx_offset = 0;
    let current_td = tokens.get(current_idx)?;

    loop {
        idx_offset += 1;
        let next_td = tokens.get(current_idx + idx_offset)?;

        // eprintln!("lex: {:?}, {:?}", next_td, next_tt);
        match next_td.kind{
            // skip new line for multiline expresssions
            // TODO: binary ops only
            Token::Newline => match current_td.kind{
                Token::Sigil | Token::Assigner => continue,
                _ => break,
            }
            Token::Whitespace | Token::EOLComment => continue,
            _ => break,
        }
    }

    Some(current_idx + idx_offset)
}