use super::comb::{AnyOf, DelimitedListOf, Either, EitherOf, LineOf, LinesOf};
use super::delim::ParenDelim;
use super::Syntax;

use crate::parse::{ParseResult, Parser};

use crate::ast;
use crate::ast::token::{KwKind, ReservedKind, Token, TokenData};

use super::stmt::{Statement, StmtBlock};

struct FuncArg;
impl<'s> Syntax<'s> for FuncArg {
    type Parsed = ast::FuncArg<'s>;

    fn check(&self, p: &Parser<'s>) -> bool {
        p.check(Token::Ident)
    }

    fn expect(&self, p: &mut Parser<'s>) -> ParseResult<Self::Parsed> {
        let td = p.expect(Token::Ident)?;
        Ok(ast::FuncArg::new(p.nid.shift(), td))
    }
}

struct FuncProto;
impl<'s> Syntax<'s> for FuncProto {
    type Parsed = ast::FuncProto<'s>;

    fn check(&self, p: &Parser<'s>) -> bool {
        p.check(AnyOf(&[
            Token::Kw(KwKind::Fun),
            Token::Reserved(ReservedKind::Def),
            Token::Reserved(ReservedKind::Fn),
        ]))
    }

    fn expect(&self, p: &mut Parser<'s>) -> ParseResult<Self::Parsed> {
        let starting_tokens = AnyOf(&[
            Token::Kw(KwKind::Fun),
            Token::Reserved(ReservedKind::Def),
            Token::Reserved(ReservedKind::Fn),
        ]);
        let td = p.expect(starting_tokens)?;
        let name = p.expect(Token::Ident)?;

        let args = p.expect(DelimitedListOf(ParenDelim, FuncArg))?;

        match td.kind {
            Token::Kw(KwKind::Fun) => (),
            Token::Reserved(ReservedKind::Def) | Token::Reserved(ReservedKind::Fn) => {
                unimplemented!("Use of bad keyword to start function")
            }
            _ => unreachable!("Parsed function without acceptable function-start keyword"),
        }

        Ok(ast::FuncProto { name, args })
    }
}

struct Function;
impl<'s> Syntax<'s> for Function {
    type Parsed = ast::Function<'s>;

    fn check(&self, p: &Parser<'s>) -> bool {
        p.check(FuncProto)
    }

    fn expect(&self, p: &mut Parser<'s>) -> ParseResult<Self::Parsed> {
        let proto = p.expect(LineOf(FuncProto))?;
        let body = p.expect(StmtBlock)?;
        Ok(ast::Function { proto, body })
    }
}

struct ExternDecl;
impl<'s> Syntax<'s> for ExternDecl {
    type Parsed = ast::FuncProto<'s>;

    fn check(&self, p: &Parser<'s>) -> bool {
        p.check(Token::Kw(KwKind::Extern))
    }

    fn expect(&self, p: &mut Parser<'s>) -> ParseResult<Self::Parsed> {
        p.expect(Token::Kw(KwKind::Extern))?;
        p.expect(FuncProto)
    }
}

struct TopLevelItem;
impl<'s> Syntax<'s> for TopLevelItem {
    type Parsed = ast::Item<'s>;

    fn check(&self, p: &Parser<'s>) -> bool {
        p.check(EitherOf(ExternDecl, Function))
    }

    fn expect(&self, p: &mut Parser<'s>) -> ParseResult<Self::Parsed> {
        let new_item = match p.expect(EitherOf(ExternDecl, Function))? {
            Either::First(ext) => ast::ItemKind::Extern(ext),
            Either::Second(func) => ast::ItemKind::Func(func),
        };

        Ok(ast::Item::new(p.nid.shift(), new_item))
    }
}

struct ModDecl;
impl<'s> Syntax<'s> for ModDecl {
    type Parsed = TokenData<'s>;

    fn check(&self, p: &Parser<'s>) -> bool {
        p.check(Token::Kw(KwKind::Mod))
    }

    fn expect(&self, p: &mut Parser<'s>) -> ParseResult<Self::Parsed> {
        p.expect(Token::Kw(KwKind::Mod))?;
        p.expect(Token::Ident)
    }
}

pub struct ModuleGrammer;
impl<'s> Syntax<'s> for ModuleGrammer {
    type Parsed = ast::Module<'s>;

    fn check(&self, p: &Parser<'s>) -> bool {
        p.check(Token::Kw(KwKind::Mod))
    }

    fn expect(&self, p: &mut Parser<'s>) -> ParseResult<Self::Parsed> {
        let mod_decl = p.expect(LineOf(ModDecl))?;
        let body = p.expect(LinesOf(TopLevelItem))?;
        Ok(ast::Module {
            decl: mod_decl,
            body,
        })
    }
}

fn build_anon_func(body: ast::Expr) -> ast::Function {
    let name_data = TokenData::new(Token::Ident, "", ast::token::TokenLoc::default());
    let proto = ast::FuncProto {
        name: name_data,
        args: Vec::new(),
    };

    ast::Function { proto, body }
}

struct AnonFunction;
impl<'s> Syntax<'s> for AnonFunction {
    type Parsed = ast::Function<'s>;

    fn check(&self, p: &Parser<'s>) -> bool {
        p.check(Statement)
    }

    fn expect(&self, p: &mut Parser<'s>) -> ParseResult<Self::Parsed> {
        let body = p.expect(Statement)?;
        Ok(build_anon_func(body))
    }
}

pub struct ReplGrammer;
impl<'s> Syntax<'s> for ReplGrammer {
    type Parsed = Vec<ast::Item<'s>>;

    fn check(&self, p: &Parser<'s>) -> bool {
        p.check(EitherOf(TopLevelItem, AnonFunction))
    }

    fn expect(&self, p: &mut Parser<'s>) -> ParseResult<Self::Parsed> {
        let body_parts = p.expect(LinesOf(EitherOf(TopLevelItem, AnonFunction)))?;

        let body = body_parts
            .into_iter()
            .map(|choice| match choice {
                Either::First(f) => f,
                Either::Second(s) => {
                    let ik = ast::ItemKind::Func(s);
                    ast::Item::new(p.nid.shift(), ik)
                }
            })
            .collect();

        Ok(body)
    }
}
