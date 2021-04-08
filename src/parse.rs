
use std::collections::HashMap;

use crate::ast::{Expr, ExprKind};
use crate::ast::node::NodeId;
use crate::ast::ops::BinaryOp;
use crate::ast::token::{TokenData, Token, KwKind};
use crate::lex::{TokenStream, TokTag};

mod comb;
mod gram;

pub use gram::{parse_module, parse_repl_line};

pub type SyntaxErrorMsg = String;

// information needed for caller function to recover from
// parse error
// example: REPL may use EarlyEOF to wait for next line of text
#[derive(Debug, Clone, Copy)]
pub enum RecoveryInfo{
    EarlyEOF,
    InvalidOp,
    InvalidToken,
}


pub type ParseResult<T> = Result<T, RecoveryInfo>;


// Lifetime will be useful for parsing custom ops
#[derive(Debug, Clone)]
struct BinOpPrec<'s>{
    prec: HashMap<BinaryOp, i32>,
    ops: HashMap<&'s str, BinaryOp>,
}

impl<'s> BinOpPrec<'s>{
    fn init() -> Self{
        let num_ops = 10;
        let mut prec_map = HashMap::with_capacity(num_ops);
        let mut count = 1;
        
        // must start from 1
        // 0 is reserved for user defined 
        prec_map.insert(BinaryOp::Assign, count);
        count += 1; 

        prec_map.insert(BinaryOp::LogicalAnd, count);
        prec_map.insert(BinaryOp::LogicalOr, count);
        count += 1;

        prec_map.insert(BinaryOp::Et, count);
        prec_map.insert(BinaryOp::Lt, count);
        prec_map.insert(BinaryOp::LtEt, count);
        count += 1;

        prec_map.insert(BinaryOp::Add, count);
        prec_map.insert(BinaryOp::Sub, count);
        count += 1;
        prec_map.insert(BinaryOp::Mul, count);
        prec_map.insert(BinaryOp::Div, count);
        // count += 1;

        let mut ops_map = HashMap::with_capacity(num_ops);
        ops_map.insert("=", BinaryOp::Assign);
        ops_map.insert("+", BinaryOp::Add);
        ops_map.insert("-", BinaryOp::Sub);
        ops_map.insert("*", BinaryOp::Mul);
        ops_map.insert("/", BinaryOp::Div);
        ops_map.insert("&&", BinaryOp::LogicalAnd);
        ops_map.insert("||", BinaryOp::LogicalOr);
        ops_map.insert("<=", BinaryOp::LtEt);
        ops_map.insert("<", BinaryOp::Lt);
        ops_map.insert("==", BinaryOp::Et);

        Self{
            ops: ops_map,
            prec: prec_map, 
        }
    }

    fn get_precedence(&self, bo: BinaryOp) -> i32{
        self.prec.get(&bo).unwrap().clone()
    }
    fn get_op_from_span(&self, op: &'s str) -> Option<&BinaryOp>{
        self.ops.get(op)
    }
}

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

    fn peek(&mut self) -> Option<Token>{
        // let td = self.current_tok?;
        let td = self.peek_data()?;
        Some(td.kind)
    }
    // fn peek_some(&mut self, ) -> ParseResult<Token>{
    //     match self.peek_data(){
    //         Some(td) => Ok(td.kind),
    //         None => {
    //             Err
    //         }
    //     }
    // }
    fn peek_data(&mut self) -> Option<&TokenData<'s>>{
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

    fn skip_newlines(&mut self){
        loop{
            match self.peek(){
                Some(Token::Newline) => {
                    self.shift();
                },
                _ => break,
            }
        }
    }

    // if token is found, return it
    // otherwise do nothing
    fn optional_tok(&mut self, t:Token) -> Option<TokenData<'s>>{
        let current = self.peek()?;
        if current == t{
            self.shift()
        } else{
            None
        }
    }

    fn expect_tok(&mut self, t: Token) -> ParseResult<TokenData<'s>>{
        let current_tok = self.peek_data();
        if let Some(tok) = current_tok{
            if tok.kind == t{
                Ok(self.shift().unwrap()) // TODO: refactor away unwrap
            } else{
                let msg = format!("Expected token {:?} but found {:?} instead", t, current_tok);
                Err(self.report_error(RecoveryInfo::InvalidToken, msg))
            }
        } else{
            Err(self.report_error(
                RecoveryInfo::EarlyEOF,
                format!("Expected token {:?} but found EOF instead", t)))
        }
    }

    fn expect_line_end(&mut self) -> ParseResult<()>{
        let current_tok = self.peek_data();
        if let Some(tok) = current_tok{
            match tok.kind{
                Token::Newline | Token::SemiColon => {
                    self.shift();
                    Ok(())
                }
                _ => {
                    let msg = format!("Expected Newline or SemiColon but found {:?} instead", current_tok);
                    Err(self.report_error(RecoveryInfo::InvalidToken, msg))
                }
            }
        } else{
            Ok(())
        }
    }

    fn expect_kw(&mut self, kw: KwKind) -> ParseResult<TokenData<'s>>{
        self.expect_tok(Token::Kw(kw))
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
            Some(tok) if tok.kind == Token::Sigil => {
                // TODO: refactor, avoid borrow checker errors
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
}


pub enum PeekOpPrec{
    Prec(i32),
    BadOp,
    ExprEnd,
}