
use super::{Syntax};
use super::comb::{
    PairOf, AnyOf, EitherOf, Either,
    LineEnd, LineOf, LinesOf, DelimitedListOf,
    EndedOf,
};
use super::delim::{ParenDelim};

use crate::parse::{Parser, ParseResult, RecoveryInfo, PeekOpPrec};

use crate::ast::token::{Token, KwKind, LitKind};
use crate::ast::{
    Expr, ExprKind, VarDecl, MutKind, 
};
use crate::ast::ops::BinaryOp;


struct LiteralExpr;
impl<'s> Syntax<'s> for LiteralExpr{
    type Parsed = Expr<'s>;

    fn check(&self, p: &Parser<'s>) -> bool{
        p.check(
            AnyOf(&[
                Token::Literal(LitKind::Float),
                Token::Literal(LitKind::Int),
                Token::Literal(LitKind::Char),
                Token::Literal(LitKind::Str),
            ])
        )
    }

    fn expect(&self, p: &mut Parser<'s>) -> ParseResult<Self::Parsed>{
        let found_lit = p.expect(
            AnyOf(&[
                Token::Literal(LitKind::Float),
                Token::Literal(LitKind::Int),
                Token::Literal(LitKind::Char),
                Token::Literal(LitKind::Str),
            ])
        )?;

        match found_lit.kind{
            Token::Literal(lit) => match lit{
                LitKind::Float => {
                    let fp = found_lit.span.parse().unwrap();
                    Ok(p.new_expr(ExprKind::Lit(fp)))
                }
                LitKind::Int => {
                    let fp = found_lit.span.parse().unwrap();
                    Ok(p.new_expr(ExprKind::Lit(fp)))
                }
                _ => unimplemented!("Invalid literal, only float and int supported")
            }
            _ => unreachable!()
        }
    }
}

// parentheses are always tuples
struct TupleExpr;
impl<'s> Syntax<'s> for TupleExpr{
    type Parsed = Expr<'s>;

    fn check(&self, p: &Parser<'s>) -> bool{
        p.check(DelimitedListOf(ParenDelim, Expression))
    }

    fn expect(&self, p: &mut Parser<'s>) -> ParseResult<Self::Parsed>{
        let body = p.expect(DelimitedListOf(ParenDelim, Expression))?;

        if body.len() == 1{
            Ok(body.into_iter().nth(0).unwrap())
        } else{
            unimplemented!("Tuples with multiple elements are not supported yet")
        }        
    }
}


struct LocalVar;
impl<'s> Syntax<'s> for LocalVar{
    type Parsed = Expr<'s>;

    fn check(&self, p: &Parser<'s>) -> bool{
        p.check(Token::Ident)
    }

    fn expect(&self, p: &mut Parser<'s>) -> ParseResult<Self::Parsed>{
        let ident = p.expect(Token::Ident)?;
        Ok(p.new_expr(ExprKind::Var(ident)))     
    }
}

struct UnaryExpr;
impl<'s> Syntax<'s> for UnaryExpr{
    type Parsed = Expr<'s>;
    
    fn check(&self, p: &Parser<'s>) -> bool{
        p.check(Token::Sigil)
    }

    fn expect(&self, p: &mut Parser<'s>) -> ParseResult<Self::Parsed>{
        let _op = p.expect(Token::Sigil)?;
        let _body = p.expect(PrimaryExpr)?;
        unimplemented!("unary expression not implemented")
    }
}


// Grammer fragments, not full syntax
// A struct implementation would require a Expr as a field
// which breaks the borrow-checking model of this implementation style
fn parse_call_expr<'s>(p: &mut Parser<'s>, callee: Expr<'s>) -> ParseResult<Expr<'s>>{
    let args = p.expect(DelimitedListOf(ParenDelim, Expression))?;
    Ok(p.new_expr(ExprKind::Call{callee:callee, args:args}))
}

// Grammer fragments, not full syntax
// A struct implementation would require a Expr as a field
// which breaks the borrow-checking model of this implementation style
fn parse_trailing_expr<'s>(p: &mut Parser<'s>, first_lead: Expr<'s>) -> ParseResult<Expr<'s>>{
    
    let mut new_lead = first_lead;
    loop{
        match p.peek(){
            Some(tok) => match tok.kind{
                Token::OpenParen  => {
                    new_lead = parse_call_expr(p, new_lead)?;
                },
                _ => return Ok(new_lead),
            }
            _ => return Ok(new_lead),
        }
    }
}




struct PrimaryExpr;
impl<'s> Syntax<'s> for PrimaryExpr{
    type Parsed = Expr<'s>;

    fn check(&self, p: &Parser<'s>) -> bool{
        match p.peek(){
            Some(td) => match td.kind{
                Token::OpenParen | 
                Token::Literal(_) | 
                Token::Ident
                    => true,
                _ => false,
            }
            _ => false,
        }
    }

    fn expect(&self, p: &mut Parser<'s>) -> ParseResult<Self::Parsed>{
        let lead = match p.peek(){
            Some(tok) => match tok.kind{
                Token::OpenParen    => p.expect(TupleExpr)?,
                Token::Ident        => p.expect(LocalVar)?, 
                Token::Literal(_)   => p.expect(LiteralExpr)?,
                Token::Sigil => unimplemented!("No unary ops supported"),
                _ => return Err(RecoveryInfo::InvalidToken(format!("Cannot parse token {:?} as part of an expr", tok)))
                // ,p.report_error(format!("Cannot parse token {:?} as part of an expr", tok))),
            }
            _ => return Err(RecoveryInfo::EarlyEOF)
            // p.report_error(, format!("Early EOF while parseing primary expr"))),
        };
    
        parse_trailing_expr(p, lead)
    }
}


// Traditional Binary Expr (1+2) without trailing-if-clause
struct PlainBinaryExpr;
impl<'s> Syntax<'s> for PlainBinaryExpr{
    type Parsed = Expr<'s>;

    fn check(&self, p: &Parser<'s>) -> bool{
        p.check(PrimaryExpr)
    }

    fn expect(&self, p: &mut Parser<'s>) -> ParseResult<Self::Parsed>{
        let first_lhs = p.expect(PrimaryExpr)?;
        parse_binary_rhs(p, first_lhs, -1)
    }
}

// TODO: refactor, change algo, make this cleaner
fn parse_binary_rhs<'s>(p: &mut Parser<'s>, lhs: Expr<'s>, old_prec: i32) -> ParseResult<Expr<'s>>{
    
    // TODO: refactor to avoid doubling lexing effort
    let prec = match p.peek_op_precedence(){
        PeekOpPrec::Prec(prec) => prec,
        PeekOpPrec::BadOp => return {
            let tok = p.peek();
            match tok {
                Some(td) =>{
                    let msg = format!("unsupported operator {:?}", td.span);
                    Err(p.report_error(RecoveryInfo::InvalidOp, msg))
                }
                None => {
                    let msg = format!("Early EOF while parsing BinaryExpr");
                    Err(p.report_error(RecoveryInfo::InvalidOp, msg))
                }
            }
        },
        PeekOpPrec::ExprEnd => return Ok(lhs),
    };

    // first iteration CANNOT satisfy this condition
    // old_prec = -1
    if prec < old_prec{
        return Ok(lhs);
    }

    let op  = parse_binary_op(p)?;
    let mut rhs = p.expect(PrimaryExpr)?;

    let next_prec = match p.peek_op_precedence(){
        PeekOpPrec::Prec(prec) => prec,
        PeekOpPrec::BadOp => return {
            let tok = p.peek();
            match tok {
                Some(td) =>{
                    let msg = format!("unsupported operator {:?}", td.span);
                    Err(p.report_error(RecoveryInfo::InvalidOp, msg))
                }
                None => {
                    let msg = format!("Early EOF while parsing BinaryExpr");
                    Err(p.report_error(RecoveryInfo::InvalidOp, msg))
                }
            }
        },
        PeekOpPrec::ExprEnd => return Ok(p.new_expr(ExprKind::Binary{op:op, lhs:lhs, rhs:rhs})),
    };

    // if this op binds less tightly, compute next expr first
    if prec < next_prec{
        rhs = parse_binary_rhs(p, rhs, prec+1)?;
    }

    let new_lhs = p.new_expr(ExprKind::Binary{op:op, lhs:lhs, rhs:rhs});

    // TODO: Tail Call Opt
    parse_binary_rhs(p, new_lhs, old_prec)
}

fn parse_binary_op<'s>(p: &mut Parser<'s>) -> ParseResult<BinaryOp>{
    let td = p.expect(AnyOf(&[
        Token::Sigil, 
        Token::Assigner
    ]))?;

    let op = match p.get_op_from_span(td.span){
        Some(op) => op,
        _ => return Err(RecoveryInfo::InvalidOp),
            // p.report_error(format!("unsupported operator {:?}", td.span))),
    };
    Ok(op)
}




// Python style trailing If syntax
// this is a clause on a BinaryExpr, not a PimaryExpr
// mutually-recursive nature allows for parsing nested-if-clause
fn parse_trailing_if_expr<'s>(p: &mut Parser<'s>, if_body: Expr<'s>) -> ParseResult<Expr<'s>>{
    p.expect(Token::Kw(KwKind::If))?;
    let cond = p.expect(BinaryExpr)?;
    
    p.expect(Token::Kw(KwKind::Else))?;
    let else_body = p.expect(BinaryExpr)?;

    Ok(p.new_expr(ExprKind::If{cond, if_body, else_body}))
}


struct BinaryExpr;
impl<'s> Syntax<'s> for BinaryExpr{
    type Parsed = Expr<'s>;
    fn check(&self, p: &Parser<'s>) -> bool{
        p.check(PlainBinaryExpr)
    }

    fn expect(&self, p: &mut Parser<'s>) -> ParseResult<Self::Parsed>{
        let mut new_lead = p.expect(PlainBinaryExpr)?;
        loop{
            match p.peek(){
                Some(tok) => match tok.kind{
                    // Token::OpenParen  => {
                    //     new_lead = parse_call_expr(p, new_lead)?;
                    // },
                    Token::Kw(KwKind::If) => {
                        new_lead = parse_trailing_if_expr(p, new_lead)?;
                    }
                    _ => return Ok(new_lead),
                }
                _ => return Ok(new_lead),
            }
        }
    }
}



struct IfExpr;
impl<'s> Syntax<'s> for IfExpr{
    type Parsed = Expr<'s>;
    fn check(&self, p: &Parser<'s>) -> bool{
        p.check(PairOf(Token::Kw(KwKind::If), LineOf(BinaryExpr)))
    }

    fn expect(&self, p: &mut Parser<'s>) -> ParseResult<Self::Parsed>{
        let block_start_grammer = PairOf(
            Token::Kw(KwKind::If), 
            LineOf(BinaryExpr)
        );
        let (_, cond) = p.expect(block_start_grammer)?;

        let if_body = p.expect(BlockBody)?;

        p.expect(Token::Kw(KwKind::Else))?;

        // sugar to avoid end cascade in "else if" stmts

        // order needs to be careful here
        let rec_body_grammer = EitherOf(
            Expression,
            PairOf(LineEnd, StmtBlock),
        );

        let else_body = match p.expect(rec_body_grammer)?{
            Either::First(f) => f,
            Either::Second((_, s)) => s,
        };

        Ok(p.new_expr(ExprKind::If{cond, if_body, else_body}))
    }
}


// typically args violate the borrow-checking model in the parser
// but this arg is trivially copy-able so we can just clone it
struct VarDeclBody;
impl<'s> Syntax<'s> for VarDeclBody{
    type Parsed = VarDecl<'s>;
    
    fn check(&self, p: &Parser<'s>) -> bool{
        p.check(EitherOf(Token::Ident, Token::Kw(KwKind::Mut)))
    }

    fn expect(&self, p: &mut Parser<'s>) -> ParseResult<Self::Parsed>{

        let found_mut = match p.parse_if_present(Token::Kw(KwKind::Mut)){
            Some(_) => MutKind::Mutable,
            None => MutKind::Const,
        };

        let bound = p.expect(Token::Ident)?;

        // TODO optionally parse type
        
        p.expect(Token::Assigner)?;

        let value = p.expect(Expression)?;

        Ok(VarDecl{
            mutable: found_mut,
            bound: bound,
            value: value,
        })
    }
}

struct ValMutDecl;
impl<'s> Syntax<'s> for ValMutDecl{
    type Parsed = Expr<'s>;
    
    fn check(&self, p: &Parser<'s>) -> bool{
        p.check(Token::Kw(KwKind::Val))
    }

    fn expect(&self, p: &mut Parser<'s>) -> ParseResult<Self::Parsed>{
        p.expect(Token::Kw(KwKind::Val))?;

        let decl = p.expect(VarDeclBody)?;

        Ok(p.new_expr(ExprKind::Decl(decl)))
    }
}


struct LetExpr;
impl<'s> Syntax<'s> for LetExpr{
    type Parsed = Expr<'s>;

    fn check(&self, p: &Parser<'s>) -> bool{
        p.check(LineOf(Token::Kw(KwKind::Let)))
    }

    fn expect(&self, p: &mut Parser<'s>) -> ParseResult<Self::Parsed>{
        p.expect(LineOf(Token::Kw(KwKind::Let)))?;

        let binding_parts = p.expect(LinesOf(VarDeclBody))?;

        
        p.expect(Token::Kw(KwKind::In))?;

        /* fancy sugar here to avoid end cascade
         * 
         * When code looks like
         * let
         *      <many var decl>
         * in <expression BEFORE newline> 
         * 
         * We can unambiguously skip the "end" keyword
         */
        let sugar_grammer = EitherOf(
            Expression, 
            PairOf(LineEnd, StmtBlock)
        );

        let let_body = match p.expect(sugar_grammer)?{
            Either::First(f) => f,
            Either::Second((_, s)) => s,
        };

        let bound = binding_parts.into_iter()
            .map(|b| p.new_expr(ExprKind::Decl(b)))
            .collect();
        
        Ok(p.new_expr(ExprKind::Let{bound, let_body}))
    }
}



struct DoExpr;
impl<'s> Syntax<'s> for DoExpr{
    type Parsed = Expr<'s>;

    fn check(&self, p: &Parser<'s>) -> bool{
        p.check(LineOf(Token::Kw(KwKind::Do)))
    }

    fn expect(&self, p: &mut Parser<'s>) -> ParseResult<Self::Parsed>{
        p.expect(LineOf(Token::Kw(KwKind::Do)))?;
        p.expect(StmtBlock)
    }
}


struct Expression;
impl<'s> Syntax<'s> for Expression{
    type Parsed = Expr<'s>;

    fn check(&self, p: &Parser<'s>) -> bool{
        match p.peek(){
            Some(tok) => match tok.kind {
                Token::Kw(kw) => match kw{
                    KwKind::If  | 
                    KwKind::Let | 
                    KwKind::Do  => true,
                    _ => unimplemented!("Bad Keyword {:?}", kw),
                }
                _ => p.check(BinaryExpr),
            }
            None => false,
        }
    }

    fn expect(&self, p: &mut Parser<'s>) -> ParseResult<Self::Parsed>{
        match p.peek(){
            Some(tok) => match tok.kind {
                Token::Kw(kw) => match kw{
                    KwKind::If  => p.expect(IfExpr),
                    KwKind::Let => p.expect(LetExpr),
                    KwKind::Do  => p.expect(DoExpr),
                    _ => unimplemented!("Bad Keyword {:?}", kw),
                }
                _ => p.expect(BinaryExpr),
            }
            None => Err(RecoveryInfo::EarlyEOF),
        }
    }
}



struct WhileStmt;
impl<'s> Syntax<'s> for WhileStmt{
    type Parsed = Expr<'s>;
    fn check(&self, p: &Parser<'s>) -> bool{
        p.check(PairOf(Token::Kw(KwKind::While), LineOf(BinaryExpr)))
    }

    fn expect(&self, p: &mut Parser<'s>) -> ParseResult<Self::Parsed>{
        p.expect(Token::Kw(KwKind::While))?;
        let cond = p.expect(LineOf(BinaryExpr))?;
        let while_body = p.expect(StmtBlock)?;
        Ok(p.new_expr(ExprKind::While{cond, while_body}))
    }
}


struct BlockBody;
impl<'s> Syntax<'s> for BlockBody{
    type Parsed = Expr<'s>;
    fn check(&self, p: &Parser<'s>) -> bool{
        p.check(LinesOf(Statement))
    }

    fn expect(&self, p: &mut Parser<'s>) -> ParseResult<Self::Parsed>{
        let body = p.expect(LinesOf(Statement))?;
        Ok(p.new_expr(ExprKind::Block(body)))
    }
}


pub struct Statement;
impl<'s> Syntax<'s> for Statement{
    type Parsed = Expr<'s>;

    fn check(&self, p: &Parser<'s>) -> bool{
        match p.peek(){
            Some(tok) => match tok.kind {
                Token::Kw(kw) => match kw{
                    KwKind::If  |
                    KwKind::Let |
                    KwKind::Val |
                    KwKind::While |
                    KwKind::Do  => true,
                    _ => false,
                }
                _ => p.check(BinaryExpr),
            }
            None => false,
        }
    }

    fn expect(&self, p: &mut Parser<'s>) -> ParseResult<Self::Parsed>{
        match p.peek(){
            Some(tok) => match tok.kind {
                Token::Kw(kw) => match kw{
                    KwKind::If  => p.expect(IfExpr),
                    KwKind::Let => p.expect(LetExpr),
                    KwKind::Val => p.expect(ValMutDecl),
                    KwKind::Do  => p.expect(DoExpr),
                    KwKind::While => p.expect(WhileStmt),
                    _ => unimplemented!("Bad Keyword {:?}", kw),
                }
                _ => p.expect(BinaryExpr),
            }
            None => Err(RecoveryInfo::EarlyEOF),
        }
    }
}

pub struct StmtBlock;
impl<'s> Syntax<'s> for StmtBlock{
    type Parsed = Expr<'s>;
    fn check(&self, p: &Parser<'s>) -> bool{
        p.check(EndedOf(BlockBody))
    }

    fn expect(&self, p: &mut Parser<'s>) -> ParseResult<Self::Parsed>{
        Ok(p.expect(EndedOf(BlockBody))?)
    }
}
