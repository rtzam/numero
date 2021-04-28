

use super::{Syntax, Delimiter};
use crate::ast::token::{Token, KwKind};
use crate::parse::{Parser, ParseResult, RecoveryInfo};


// First parse F, then parse S
pub struct PairOf<F, S>(pub F, pub S);

impl<'s, F,S> Syntax<'s> for PairOf<F,S> 
where
    F: Syntax<'s>,
    S: Syntax<'s>
{
    type Parsed = (F::Parsed, S::Parsed);

    fn check(&self, p: &Parser<'s>) -> bool{
        p.check(&self.0)
    }

    fn expect(&self, p: &mut Parser<'s>) -> ParseResult<Self::Parsed>{
        let first = p.expect(&self.0)?;
        let second = p.expect(&self.1)?;
        Ok((first, second))
    }
}


// // representing regex *
// pub struct ManyOf<S>(pub S);

// impl<'s, S> Syntax<'s> for ManyOf<S> 
// where
//     S: Syntax<'s>
// {
//     type Parsed = Vec<S::Parsed>;

//     fn check(&self, p: &Parser<'s>) -> bool{
//         p.check(&self.0)
//     }

//     fn expect(&self, p: &mut Parser<'s>) -> ParseResult<Self::Parsed>{
//         let mut content = Vec::new();

//         while p.check(&self.0){
//             let next_item = p.expect(&self.0)?;
//             content.push(next_item);
//         }

//         Ok(content)
//     }
// }



// Try to parse first, if failed, try parse second
pub struct EitherOf<F,S>(pub F, pub S);

// Result type for EitherOf::expect
pub enum Either<F,S>
{
    First(F),
    Second(S)
}

impl<'s, F,S> Syntax<'s> for EitherOf<F,S> 
where
    F: Syntax<'s>,
    S: Syntax<'s>
{
    type Parsed = Either<F::Parsed, S::Parsed>;

    fn check(&self, p: &Parser<'s>) -> bool{
        p.check(&self.0) || p.check(&self.1)
    }

    fn expect(&self, p: &mut Parser<'s>) -> ParseResult<Self::Parsed>{
        let either_result = match p.expect(&self.0){
            Ok(f) => Either::First(f),
            Err(_) => Either::Second(p.expect(&self.1)?)
        };

        Ok(either_result)
    }
}


// Try to parse first, if failed, try parse second
pub struct AnyOf<'a, S>(pub &'a [S]);

impl<'a, 's, S> Syntax<'s> for AnyOf<'a, S> 
where
    S: Syntax<'s>
{
    type Parsed = S::Parsed;

    fn check(&self, p: &Parser<'s>) -> bool{
        self.0.iter().any(|x| p.check(x))
    }

    fn expect(&self, p: &mut Parser<'s>) -> ParseResult<Self::Parsed>{
        let mut current_error = RecoveryInfo::CompilerBug;
        
        for value in self.0{
            match p.expect(value){
                Ok(found) => return Ok(found),
                Err(e) => {
                    current_error = e;
                }
            }
        }

        Err(current_error)
    }
}



pub struct LineEnd;
impl<'s> Syntax<'s> for LineEnd{
    type Parsed = ();

    fn check(&self, p: &Parser<'s>) -> bool{
        p.check(AnyOf(&[
            Token::Newline,
            Token::SemiColon,
        ]))
    }

    fn expect(&self, p: &mut Parser<'s>) -> ParseResult<Self::Parsed>{
        p.expect(AnyOf(&[
            Token::Newline,
            Token::SemiColon,
        ]))?;
        Ok(())
    }
}

// sugar around PairOf newline
pub struct LineOf<S>(pub S);
impl<'s, S> Syntax<'s> for LineOf<S> 
where
    S: Syntax<'s>
{
    type Parsed = S::Parsed;

    fn check(&self, p: &Parser<'s>) -> bool{
        p.check(PairOf(&self.0, LineEnd))
    }

    fn expect(&self, p: &mut Parser<'s>) -> ParseResult<Self::Parsed>{
        let (content, _) = p.expect(PairOf(&self.0, LineEnd))?;
        Ok(content)
    }
}


pub struct SkipEmptyLines;

impl<'s> Syntax<'s> for SkipEmptyLines{
    type Parsed = ();

    fn check(&self, p: &Parser<'s>) -> bool{
        p.check(LineEnd)
    }

    fn expect(&self, p: &mut Parser<'s>) -> ParseResult<Self::Parsed>{
        while p.check(LineEnd){
            p.expect(LineEnd)?;
        }

        Ok(())
    }
}

pub struct SkipWhitespace<S>(pub S);

impl<'s, S> Syntax<'s> for SkipWhitespace<S>
where 
    S: Syntax<'s>
{
    type Parsed = S::Parsed;

    fn check(&self, p: &Parser<'s>) -> bool{
        p.check(LineEnd) || p.check(&self.0)
    }

    fn expect(&self, p: &mut Parser<'s>) -> ParseResult<Self::Parsed>{
        while p.check(LineEnd){
            p.expect(LineEnd)?;
        }

        p.expect(&self.0)
    }
}


pub struct LinesOf<S>(pub S);
impl<'s, S> Syntax<'s> for LinesOf<S> 
where
    S: Syntax<'s>
{
    type Parsed = Vec<S::Parsed>;

    fn check(&self, p: &Parser<'s>) -> bool{
        p.check(SkipWhitespace(&self.0))
    }

    fn expect(&self, p: &mut Parser<'s>) -> ParseResult<Self::Parsed>{

        let mut body = Vec::new();

        loop{ 
            p.expect(SkipEmptyLines)?;

            match p.parse_if_present(&self.0){
                Some(maybe_parsed) => {
                    body.push(maybe_parsed?);
                }
                None => break,
            }
        }

        Ok(body)
    }
}



pub struct DelimitedListOf<D, S>(pub D, pub S);

impl<'s, D, S> Syntax<'s> for DelimitedListOf<D, S> 
where
    D: Delimiter<'s>, 
    S: Syntax<'s>
{
    type Parsed = Vec<S::Parsed>;

    fn check(&self, p: &Parser<'s>) -> bool{
        p.check(self.0.open_syntax())
    }

    fn expect(&self, p: &mut Parser<'s>) -> ParseResult<Self::Parsed>{
        p.expect(self.0.open_syntax())?;

        let mut body = Vec::new();
        let seperator = EitherOf(Token::Comma, LineEnd);
        let inner_grammer = EitherOf(self.0.close_syntax(), &self.1);
        loop{
            p.expect(SkipEmptyLines)?;

            // this parses trailing commas before a close delimiter
            match p.expect(&inner_grammer)?{
                Either::First(_) => break,
                Either::Second(found) => {
                    body.push(found);

                    match p.expect(EitherOf(&seperator, self.0.close_syntax()))?{
                        Either::First(_) => (),
                        Either::Second(_) => break,
                    }
                }
            }
        }
        
        Ok(body)
    }
}



// sugar around PairOf newline
pub struct EndedOf<S>(pub S);
impl<'s, S> Syntax<'s> for EndedOf<S> 
where
    S: Syntax<'s>
{
    type Parsed = S::Parsed;

    fn check(&self, p: &Parser<'s>) -> bool{
        p.check(PairOf(&self.0, Token::Kw(KwKind::End)))
    }

    fn expect(&self, p: &mut Parser<'s>) -> ParseResult<Self::Parsed>{
        let (content, _) = p.expect(PairOf(&self.0, Token::Kw(KwKind::End)))?;
        Ok(content)
    }
}

