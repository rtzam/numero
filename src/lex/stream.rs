


use crate::lex::scan::Tokenizer;
use crate::ast::token::{Token, TokenData, KwKind};


pub type TokenBuffer<'s> = Vec<TokenData<'s>>;


type TokTagRepr = usize;
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TokTag(TokTagRepr);

// impl TokTag{
//     fn inc(self) -> Self{
//         self.inc_by(1)
//     }
//     fn inc_by(self, by: TokTagRepr) -> Self{
//         Self(self.0 + by)
//     }
// }

/**
 * Basic Token transformation to provide parser more information 
 */
#[derive(Clone)]
pub struct TokenStream<'s>{
    tokenizer: Tokenizer<'s>,
    tokens: TokenBuffer<'s>,
    // idx: TokTag,
}

impl<'s> TokenStream<'s>{
    pub fn new(source: &'s str) -> TokenStream<'s>{
        TokenStream{
            tokenizer: Tokenizer::new(source),
            tokens: TokenBuffer::new(),
            // idx: TokTag(0),
        }
    }

    // fn shift_token(&mut self) -> Option<TokenData<'s>>{
    //     let current_token = *self.peek_current()?;
    //     self.advance();
    //     Some(current_token)
    // }

    fn buffer_next_valid_token(&mut self) -> usize{
        let mut count = 0;
        loop {
            let try_tok = self.tokenizer.next();

            match try_tok{
                Some(td) => {
                    count += 1;
                    let next_td = process_token_kw(td);
                    self.tokens.push(next_td);

                    match next_td.kind{
                        Token::Whitespace | Token::EOLComment => continue,
                        _ => break
                    }
                }
                None => break,
            }
        }

        count
    }

    // fn buffered_tokens(&self) -> usize{
    //     self.tokens.len()
    // }
    // fn advance(&mut self){
    //     eprintln!("{:?}", self.idx);
    //     self.idx = self.idx.inc();
    //     eprintln!("{:?}", self.idx);
    // }
    fn buffer_next_token(&mut self){
        match self.tokenizer.next(){
            Some(td) => {
                self.tokens.push(process_token_kw(td));
            }
            None => (),
        }
    }

    fn ensure_tokens_buffered_until(&mut self, stop: TokTag){
        if self.tokens.len() < 1 + stop.0{
            // number of tokens to buffer
            let n_iters = 1 + stop.0 - self.tokens.len();
            for _ in 0..n_iters{
                self.buffer_next_token()
            }
        }
    }

    pub fn get_current_tok_tag(&self) -> TokTag{
        // TODO: underflow
        TokTag(self.tokens.len() - 1)
    }
    pub fn get_token(&mut self, tag: TokTag) -> Option<&TokenData<'s>>{
        self.ensure_tokens_buffered_until(tag);
        self.tokens.get(tag.0)
    }
    // pub fn get_tokens_until_current(&mut self, start: TokTag) -> Option<&[TokenData<'s>]>{
    //     let end_tag = TokTag(self.tokens.len());
    //     self.get_token_range(start, end_tag)
    // }
    pub fn get_token_range(&mut self, start: TokTag, end: TokTag) -> Option<&[TokenData<'s>]>{
        self.ensure_tokens_buffered_until(end);
        self.tokens.get(start.0..end.0)
    }
    // pub fn peek_current(&mut self) -> Option<&TokenData<'s>>{
    //     self.peek_by(0)
    // }
    // pub fn peek_by(&mut self, offset: u8) -> Option<&TokenData<'s>>{
    //     let index = self.idx.0 + offset as usize;
    //     self.ensure_tokens_buffered_until(TokTag(index));
    //     self.tokens.get(index)
    // }

    // pub fn reset_to(&mut self, tag: TokTag){
    //     self.idx = tag;
    // }
}

impl<'s> Iterator for TokenStream<'s>{
    type Item = TokTag;
    fn next(&mut self) -> Option<Self::Item> {
        let toks_buffered = self.buffer_next_valid_token();
        match toks_buffered{
            0 => None,
            _ => Some(self.get_current_tok_tag()),
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