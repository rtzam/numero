
use std::{collections::HashMap};

use lazy_static::lazy_static;

use crate::ast::token::{TokenData, Token, KwKind, ReservedKind};

mod scan;


pub fn scan_source<'s>(source: &'s str) -> Vec<TokenData<'s>>{
    let tokenizer = scan::Tokenizer::new(source);
    tokenizer.map(|td| process_token_kw(td)).collect()
}


fn process_token_kw<'s>(tok: TokenData<'s>) -> TokenData<'s>{
    match tok.kind{
        Token::Ident => {
            // TODO: lazy_static HashMap
            let new_kind = match _TOKEN_KW_TABLE.get(tok.span){
                Some(kind) => kind.clone(),
                _ => tok.kind,
            };

            TokenData::new(new_kind, tok.span, tok.loc)
        }

        _ => tok,
    }
}

lazy_static!{
    static ref _TOKEN_KW_LIST: [(&'static str, Token); 75] = [        
        ("fun"      , Token::Kw(KwKind::Fun)),
        ("extern"   , Token::Kw(KwKind::Extern)),
        ("end"      , Token::Kw(KwKind::End)),
        ("do"       , Token::Kw(KwKind::Do)),
        ("mod"      , Token::Kw(KwKind::Mod)),
        ("if"       , Token::Kw(KwKind::If)),
        ("else"     , Token::Kw(KwKind::Else)),
        ("val"      , Token::Kw(KwKind::Val)),
        ("let"      , Token::Kw(KwKind::Let)),
        ("in"       , Token::Kw(KwKind::In)),
        ("mut"      , Token::Kw(KwKind::Mut)),
        ("while"    , Token::Kw(KwKind::While)),
        ("loop"     , Token::Reserved(ReservedKind::Loop)),
        ("from"     , Token::Reserved(ReservedKind::FromKw)),
        ("import"   , Token::Reserved(ReservedKind::Import)),
        ("export"   , Token::Reserved(ReservedKind::Export)),
        ("def"      , Token::Reserved(ReservedKind::Def)),
        ("fn"       , Token::Reserved(ReservedKind::Fn)),
        ("this"     , Token::Reserved(ReservedKind::This)),
        ("self"     , Token::Reserved(ReservedKind::KwSelf)),
        ("enum"     , Token::Reserved(ReservedKind::Enum)),
        ("struct"   , Token::Reserved(ReservedKind::Struct)),
        ("class"    , Token::Reserved(ReservedKind::Class)),
        ("rec"      , Token::Reserved(ReservedKind::Rec)),
        ("data"     , Token::Reserved(ReservedKind::Data)),
        ("type"     , Token::Reserved(ReservedKind::Type)),
        ("alias"    , Token::Reserved(ReservedKind::Alias)),
        ("use"      , Token::Reserved(ReservedKind::Use)),
        ("using"    , Token::Reserved(ReservedKind::Using)),
        ("as"       , Token::Reserved(ReservedKind::As)),
        ("pub"      , Token::Reserved(ReservedKind::Pub)),
        ("exposing" , Token::Reserved(ReservedKind::Exposing)),
        ("async"    , Token::Reserved(ReservedKind::Async)),
        ("await"    , Token::Reserved(ReservedKind::Await)),
        ("for"      , Token::Reserved(ReservedKind::For)),
        ("match"    , Token::Reserved(ReservedKind::Match)),
        ("case"     , Token::Reserved(ReservedKind::Case)),
        ("switch"   , Token::Reserved(ReservedKind::Switch)),
        ("and"      , Token::Reserved(ReservedKind::And)),
        ("or"       , Token::Reserved(ReservedKind::Or)),
        ("xor"      , Token::Reserved(ReservedKind::Xor)),
        ("ref"      , Token::Reserved(ReservedKind::Ref)),
        ("var"      , Token::Reserved(ReservedKind::Var)),
        ("const"    , Token::Reserved(ReservedKind::Const)),
        ("global"   , Token::Reserved(ReservedKind::Global)),
        ("local"    , Token::Reserved(ReservedKind::Local)),
        ("new"      , Token::Reserved(ReservedKind::New)),
        ("del"      , Token::Reserved(ReservedKind::Del)),
        ("delete"   , Token::Reserved(ReservedKind::Delete)),
        ("assert"   , Token::Reserved(ReservedKind::Assert)),
        ("defer"    , Token::Reserved(ReservedKind::Defer)),
        ("move"     , Token::Reserved(ReservedKind::Move)),
        ("go"       , Token::Reserved(ReservedKind::Go)),
        ("try"      , Token::Reserved(ReservedKind::Try)),
        ("catch"    , Token::Reserved(ReservedKind::Catch)),
        ("break"    , Token::Reserved(ReservedKind::Break)),
        ("continue" , Token::Reserved(ReservedKind::Continue)),
        ("goto"     , Token::Reserved(ReservedKind::GoTo)),
        ("impl"     , Token::Reserved(ReservedKind::Impl)),
        ("return"   , Token::Reserved(ReservedKind::Return)),
        ("yield"    , Token::Reserved(ReservedKind::Yield)),
        ("throw"    , Token::Reserved(ReservedKind::Throw)),
        ("raise"    , Token::Reserved(ReservedKind::Raise)),
        ("static"   , Token::Reserved(ReservedKind::Static)),
        ("trait"    , Token::Reserved(ReservedKind::Trait)),
        ("super"    , Token::Reserved(ReservedKind::Super)),
        ("unsafe"   , Token::Reserved(ReservedKind::Unsafe)),
        ("where"    , Token::Reserved(ReservedKind::Where)),
        ("final"    , Token::Reserved(ReservedKind::Final)),
        ("virtual"  , Token::Reserved(ReservedKind::Virtual)),
        ("override" , Token::Reserved(ReservedKind::Override)),
        ("except"   , Token::Reserved(ReservedKind::Except)),
        ("dyn"      , Token::Reserved(ReservedKind::Dyn)),
        ("bit"      , Token::Reserved(ReservedKind::Bit)),
        ("flag"      , Token::Reserved(ReservedKind::Flag)),
    ];
    static ref _TOKEN_KW_TABLE: HashMap<&'static str, Token> = _TOKEN_KW_LIST.iter().cloned().collect();
}