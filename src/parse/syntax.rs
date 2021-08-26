use crate::ast::token::{Token, TokenData};
use crate::parse::{ParseResult, Parser, RecoveryInfo};

mod comb;
mod delim;
mod stmt;
mod top;

pub use top::{ModuleGrammer, ReplGrammer};

// modified from
// https://github.com/lark-exploration/lark/blob/master/components/lark-parser/src/syntax.rs

// Modified from Lark Compiler
pub trait Syntax<'s> {
    type Parsed;
    fn check(&self, p: &Parser<'s>) -> bool;
    fn expect(&self, p: &mut Parser<'s>) -> ParseResult<Self::Parsed>;
}

// Known Invalid Grammer to be parsed for error recovery
pub trait InvalidSyntax<'s>: Syntax<'s> {}

// A Syntax whose `expect` method, when `test` returns true, always
// consumes at least one token.
pub trait NonEmptySyntax<'s>: Syntax<'s> {}

pub trait Delimiter<'s> {
    type Open: NonEmptySyntax<'s>;
    type Close: NonEmptySyntax<'s>;
    fn open_syntax(&self) -> Self::Open;
    fn close_syntax(&self) -> Self::Close;
}

impl<'s, T> Syntax<'s> for &T
where
    T: Syntax<'s>,
{
    type Parsed = T::Parsed;

    fn check(&self, parser: &Parser<'s>) -> bool {
        T::check(self, parser)
    }

    fn expect(&self, parser: &mut Parser<'s>) -> ParseResult<Self::Parsed> {
        T::expect(self, parser)
    }
}

impl<'s> Syntax<'s> for Token {
    type Parsed = TokenData<'s>;

    fn check(&self, p: &Parser<'s>) -> bool {
        match p.peek() {
            Some(td) => &td.kind == self,
            _ => false,
        }
    }

    fn expect(&self, p: &mut Parser<'s>) -> ParseResult<Self::Parsed> {
        let current_token = p.peek();
        match current_token {
            Some(td) => {
                if td.kind == *self {
                    let final_token = *td;
                    p.shift();
                    Ok(final_token)
                } else {
                    let msg = format!(
                        "Expected token {:?} But Found {:?} at {:?}",
                        self, td.kind, td.loc
                    );
                    Err(RecoveryInfo::InvalidToken(msg))
                }
            }
            _ => Err(RecoveryInfo::EarlyEOF),
        }
    }
}

impl<'s> NonEmptySyntax<'s> for Token {}
