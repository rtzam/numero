use super::{Syntax};
use super::comb::{
    EitherOf, Either,
    LineOf, LinesOf, DelimitedListOf, 
};
use super::delim::{ParenDelim, };

use crate::parse::{Parser, ParseResult};

use crate::ast::token::{Token, KwKind, TokenData};
use crate::ast;

use super::stmt::{StmtBlock, Statement};


struct FuncArg;
impl<'s> Syntax<'s> for FuncArg{
    type Parsed = ast::FuncArg<'s>;

    fn check(&self, p: &Parser<'s>) -> bool{
        p.check(Token::Ident)
    }

    fn expect(&self, p: &mut Parser<'s>) -> ParseResult<Self::Parsed>{
        let td = p.expect(Token::Ident)?;
        Ok(ast::FuncArg::new(p.nid.shift(), td))      
    }
}

struct FuncProto;
impl<'s> Syntax<'s> for FuncProto{
    type Parsed = ast::FuncProto<'s>;

    fn check(&self, p: &Parser<'s>) -> bool{
        p.check(Token::Kw(KwKind::Fun))
    }

    fn expect(&self, p: &mut Parser<'s>) -> ParseResult<Self::Parsed>{
        p.expect(Token::Kw(KwKind::Fun))?;

        let name = p.expect(Token::Ident)?;

        let args = p.expect(
           DelimitedListOf(ParenDelim, FuncArg) 
        )?;

        Ok(ast::FuncProto{name:name, args: args})
    }
}


struct Function;
impl<'s> Syntax<'s> for Function{
    type Parsed = ast::Function<'s>;

    fn check(&self, p: &Parser<'s>) -> bool{
        p.check(FuncProto)
    }

    fn expect(&self, p: &mut Parser<'s>) -> ParseResult<Self::Parsed>{
        let proto = p.expect(LineOf(FuncProto))?;
        let body = p.expect(StmtBlock)?;
        Ok(ast::Function{proto:proto, body:body})     
    }
}

struct ExternDecl;
impl<'s> Syntax<'s> for ExternDecl{
    type Parsed = ast::FuncProto<'s>;

    fn check(&self, p: &Parser<'s>) -> bool{
        p.check(Token::Kw(KwKind::Extern))
    }

    fn expect(&self, p: &mut Parser<'s>) -> ParseResult<Self::Parsed>{
        p.expect(Token::Kw(KwKind::Extern))?;
        p.expect(FuncProto)
    }
}


struct TopLevelItem;
impl<'s> Syntax<'s> for TopLevelItem{
    type Parsed = ast::Item<'s>;

    fn check(&self, p: &Parser<'s>) -> bool{
        p.check(EitherOf(ExternDecl, Function))
    }

    fn expect(&self, p: &mut Parser<'s>) -> ParseResult<Self::Parsed>{
        let new_item = match p.expect(EitherOf(ExternDecl, Function))?{
            Either::First(ext) =>  ast::ItemKind::Extern(ext),
            Either::Second(func) => ast::ItemKind::Func(func),
        };

        Ok(ast::Item::new(p.nid.shift(), new_item))
    }
}

struct ModDecl;
impl<'s> Syntax<'s> for ModDecl{
    type Parsed = TokenData<'s>;

    fn check(&self, p: &Parser<'s>) -> bool{
        p.check(Token::Kw(KwKind::Mod))
    }

    fn expect(&self, p: &mut Parser<'s>) -> ParseResult<Self::Parsed>{
        p.expect(Token::Kw(KwKind::Mod))?;
        p.expect(Token::Ident)
    }
}


pub struct ModuleGrammer;
impl<'s> Syntax<'s> for ModuleGrammer{
    type Parsed = ast::Module<'s>;

    fn check(&self, p: &Parser<'s>) -> bool{
        p.check(Token::Kw(KwKind::Mod))
    }

    fn expect(&self, p: &mut Parser<'s>) -> ParseResult<Self::Parsed>{
        let mod_decl = p.expect(LineOf(ModDecl))?;
        let body = p.expect(LinesOf(TopLevelItem))?;
        Ok(ast::Module{decl: mod_decl, body: body})
    }
}



fn build_anon_func<'s>(body: ast::Expr<'s>) -> ast::Function<'s>{
    let name_data = TokenData::new(Token::Ident, "", ast::token::TokenLoc::new());
    let proto = ast::FuncProto{name: name_data, args: Vec::new()};

    ast::Function{proto:proto, body:body}
}

struct AnonFunction;
impl<'s> Syntax<'s> for AnonFunction{
    type Parsed = ast::Function<'s>;

    fn check(&self, p: &Parser<'s>) -> bool{
        p.check(Statement)
    }

    fn expect(&self, p: &mut Parser<'s>) -> ParseResult<Self::Parsed>{
        let body = p.expect(Statement)?;
        Ok(build_anon_func(body))
    }
}

pub struct ReplGrammer;
impl<'s> Syntax<'s> for ReplGrammer{
    type Parsed = Vec<ast::Item<'s>>;

    fn check(&self, p: &Parser<'s>) -> bool{
        p.check(EitherOf(TopLevelItem, AnonFunction))
    }

    fn expect(&self, p: &mut Parser<'s>) -> ParseResult<Self::Parsed>{
        
        let body_parts = p.expect(LinesOf(EitherOf(TopLevelItem, AnonFunction)))?;

        let body = body_parts.into_iter()
            .map(|choice|
                match choice{
                    Either::First(f) => f,
                    Either::Second(s) => {
                        let ik = ast::ItemKind::Func(s);
                        ast::Item::new(p.nid.shift(), ik)
                    },
                })
            .collect();

        Ok(body)
    }
}
