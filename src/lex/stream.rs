


use crate::lex::scan::Tokenizer;
use crate::ast::token::{Token, TokenData, KwKind};


pub type TokenBuffer<'s> = Vec<TokenData<'s>>;


type TokTagRepr = usize;
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TokTag(TokTagRepr);

/**
 * Basic Token transformation to provide parser more information 
 */
#[derive(Clone)]
pub struct TokenStream<'s>{
    tokens: TokenBuffer<'s>,
    idx: TokTagRepr,
}

impl<'s> TokenStream<'s>{
    pub fn new(source: &'s str) -> TokenStream<'s>{
        let tokenizer = Tokenizer::new(source);
        TokenStream{
            tokens: tokenizer.map(|td| process_token_kw(td)).collect(),
            idx: 0,
        }
    }

    pub fn get_token(&self, tag: TokTag) -> Option<&TokenData<'s>>{
        self.tokens.get(tag.0)
    }
    
    pub fn get_token_range(&mut self, start: TokTag, end: TokTag) -> Option<&[TokenData<'s>]>{
        self.tokens.get(start.0..end.0)
    }
}

impl<'s> Iterator for TokenStream<'s>{
    type Item = TokTag;
    fn next(&mut self) -> Option<Self::Item> {
        loop{
            let tag_idx = self.idx.clone();
            self.idx += 1;
            let current_token = self.tokens.get(tag_idx)?;
            match current_token.kind{
                Token::Whitespace | 
                Token::EOLComment => (),
                _ => return Some(TokTag(tag_idx)),
            }
        }
    } 
}


fn process_token_kw<'s>(tok: TokenData<'s>) -> TokenData<'s>{
    match tok.kind{
        Token::Ident => {
            // TODO: lazy_static HashMap
            let new_kind = match tok.span{
                "fun"   => Token::Kw(KwKind::Fun),   
                "extern"=> Token::Kw(KwKind::Extern),
                "end"   => Token::Kw(KwKind::End),
                "do"    => Token::Kw(KwKind::Do),
                "mod"   => Token::Kw(KwKind::Mod),
                "if"    => Token::Kw(KwKind::If),
                "else"  => Token::Kw(KwKind::Else),
                "val"   => Token::Kw(KwKind::Val),
                "let"   => Token::Kw(KwKind::Let),
                "in"    => Token::Kw(KwKind::In),
                "mut"   => Token::Kw(KwKind::Mut),
                // "loop"  => Token::Kw(KwKind::Loop),
                "while" => Token::Kw(KwKind::While),
                // "from"  => Token::Kw(KwKind::From),
                // "import"=> Token::Kw(KwKind::Export),
                // "export"=> Token::Kw(KwKind::Export),
                _        => tok.kind,
            };

            TokenData::new(new_kind, tok.span, tok.loc)
        }

        _ => tok,
    }
}

#[test]
fn test_tok_tag_gen() {
    let src = "if test while 1.0 2 ";
    let tt: Vec<_> = TokenStream::new(src).into_iter().collect();

    assert_eq!(tt[0], TokTag(0));
    assert_eq!(tt[1], TokTag(2));
    assert_eq!(tt[2], TokTag(4));
    assert_eq!(tt[3], TokTag(6));
    assert_eq!(tt[4], TokTag(8));
}