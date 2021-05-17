
use crate::ast::token::{TokenData, Token, KwKind};

mod scan;


pub fn scan_source<'s>(source: &'s str) -> Vec<TokenData<'s>>{
    let tokenizer = scan::Tokenizer::new(source);
    tokenizer.map(|td| process_token_kw(td)).collect()
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