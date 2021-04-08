
use unicode_xid::UnicodeXID;

use crate::ast::token::{Token, TokenData, TokenLoc, LitKind};


#[derive(Clone, Copy)]
pub struct Tokenizer<'s>{
    source: &'s str,
    current_loc: TokenLoc,
    rest: &'s str,
    tok_len: usize,
    // comment_depth: u8,
    current_state: LexState,
}

impl<'s> Tokenizer<'s>{

    pub fn new(source: &'s str) -> Tokenizer<'s>{
        Tokenizer{
            source: source,
            rest:   source,
            tok_len: 0,
            current_state: LexState::Top,
            current_loc: TokenLoc::new(),
        }
    }

    // pub fn at_loc(source: &'s str, loc: TokenLoc) -> Tokenizer<'s>{
    //     Tokenizer{
    //         source: source,
    //         rest:   source,
    //         tok_len: 0,
    //         current_state: LexState::Top,
    //         current_loc: loc,
    //     }
    // }

    fn reset_for_new_lex(&mut self){
        self.source  = self.rest;
        self.tok_len = 0; 
    }

    fn complete_token(&mut self, tk: Token, loc: TokenLoc) -> Option<TokenData<'s>>{
        let result = self.tokenize(tk, loc);
        self.reset_for_new_lex(); 
        result
    }

    fn tokenize(&self, tk: Token, loc: TokenLoc) -> Option<TokenData<'s>>{
        Some(TokenData::new(tk, self.get_tok_slice()?, loc))
    }

    fn get_tok_slice(&self) -> Option<&'s str>{
        self.source.get(0..self.tok_len)
    }

    fn update_state(&mut self, action: LexAction){
        match action{
            LexAction::Continue => (),
            LexAction::Transition(s) => {self.current_state = s;},
            // LexAction::PopState => {
            //     let prev = self.state_stack.pop();
            //     match prev {
            //         Some(s) => self.current_state = s;
            //         None => panic!("Attempted to Pop LexState with None remaining"),
            //     }
            // }
            // LexAction::PushState(s) => {
            //     self.state_stack.push(self.current_state);
            //     self.current_state = s;
            // }
        }
    }

    fn update_location(&mut self, c: char){
        self.current_loc = match c {
            '\n' => self.current_loc.next_line(),
            _    => self.current_loc.next_col(),
        };
    }

    fn update_pos_by_char(&mut self, step: Option<char>){
        match step{
            Some(c) => {
                self.update_location(c);

                let n = c.len_utf8();
                self.tok_len += n;
                self.rest = &self.rest[n..];
            }
            _ => (),
        }
    } 
}

impl<'s> Iterator for Tokenizer<'s>{
    type Item = TokenData<'s>;

    fn next(&mut self) -> Option<Self::Item>{
        let tok_start = self.current_loc.clone();
        loop {
            let mut char_stream = self.rest.chars();
            let next_char = char_stream.next();
            let next_action = lex_next_chunk(self.current_state, next_char, char_stream.as_str());

            match next_action.next_bite{
                LexBite::Comsume => self.update_pos_by_char(next_char),
                LexBite::Reconsume => (),
            }

            self.update_state(next_action.next_action);

            match next_action.emit_token{
                Some(tok)   => return self.complete_token(tok, tok_start),
                None        => match self.current_state{
                    LexState::EOF => return None,
                    _ => (),
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
enum LexState{
    // MaybeFloat,
    MaybeComment,
    MaybeColon,
    Unknown,

    Top,
    Ident,
    // Indent,
    Whitespace,
    Int,
    Float,
    // Char,
    // Str,
    // Comment,
    EOLComment,
    Sigil,
    EOF,
}

enum LexBite{
    Comsume,
    Reconsume,
}



fn begin_lex_from_top(chunk: Option<char>) -> LexNext{
    
    use LexState::*;

    if let Some(c) = chunk{
        match c {
            '/'  => consume().and_trans(MaybeComment),
            // '\'' => consume().and_trans(Char),
            // '"'  => consume().and_trans(Str),
            '_'  => consume().and_trans(Ident),
            '\n' => consume().and_emit(Token::Newline),//consume().and_trans(Indent),
            ';' => consume().and_emit(Token::SemiColon),

            // '.' => consume().and_emit(Token::Dot),
            '(' => consume().and_emit(Token::OpenParen),
            ')' => consume().and_emit(Token::ClosedParen),
            // '[' => consume().and_emit(Token::OpenSquare),
            // ']' => consume().and_emit(Token::ClosedSquare),
            ',' => consume().and_emit(Token::Comma),
            ':' => consume().and_trans(MaybeColon),
            // '|' => consume().and_emit(Token::Pipe),
            '.' => consume().and_emit(Token::Dot),
            

            '0'..='9'        => consume().and_trans(Int),
            c if is_sigil(c) => consume().and_trans(Sigil),
            c if UnicodeXID::is_xid_start(c) => consume().and_trans(Ident),
            c if c.is_whitespace() => consume().and_trans(Whitespace),

            _ => consume().and_trans(Unknown),
        }
    } else{
        reconsume().and_eof()
    }
}


fn lex_next_chunk<'s>(state: LexState, chunk: Option<char>, rest: &'s str) -> LexNext{

    use LexState::*;

    match state{
        Top => begin_lex_from_top(chunk),
        Ident => match chunk{
            Some('\'') | Some('_') => consume().and_continue(),
            Some(c) if c.is_numeric() => consume().and_continue(),
            Some(c) if UnicodeXID::is_xid_start(c) => consume().and_continue(),

            _ => reconsume().and_emit(Token::Ident),
        }

        Whitespace => match chunk{
            Some('\n') => reconsume().and_emit(Token::Whitespace),
            Some(c) if c.is_whitespace() => consume().and_continue(),
            _ => reconsume().and_emit(Token::Whitespace),
        }
        // Indent => match chunk{
        //     Some('\n') | None => reconsume().and_emit(Token::Whitespace),
        //     Some(c) if c.is_whitespace() => consume().and_continue(),
        //     _ => reconsume().and_emit(Token::Indent),
        // }

        // Char => match chunk{
        //     Some('\'') => consume().and_emit(Token::Char),
        //     None    => reconsume().and_emit(Token::UnclosedChar),
        //     _       => consume().and_continue(),
        // }
        // Str => match chunk{
        //     Some('"') => consume().and_emit(Token::Str),
        //     None    => reconsume().and_emit(Token::UnclosedStr),
        //     _       => consume().and_continue(),
        // }
        LexState::Int => match chunk{
            Some('.') => match rest.chars().next() {
                Some('0'..='9') => consume().and_trans(Float),
                _ => reconsume().and_emit(Token::Literal(LitKind::Int)),
            }
            Some('0'..='9') => consume().and_continue(),
            _ => reconsume().and_emit(Token::Literal(LitKind::Int)),
        }
        Float => match chunk{
            Some('0'..='9') => consume().and_continue(),
            _ => reconsume().and_emit(Token::Literal(LitKind::Float)),
        }
        Sigil => match chunk{
            Some(c) if is_sigil(c) => consume().and_continue(),
            _ => reconsume().and_emit(Token::Sigil),
        }
        MaybeColon => match chunk{
            Some(c) if is_sigil(c) => consume().and_trans(Sigil),
            _ => reconsume().and_emit(Token::ColonSingle), 
        }
        MaybeComment => match chunk{
            Some('/') => consume().and_trans(EOLComment),
            // Some('>') => consume().and_emit(Token::TypeArrow),
            Some(c) if is_sigil(c) => consume().and_trans(Sigil),
            _ => reconsume().and_emit(Token::Sigil),
        }
        EOLComment => match chunk{
            None | Some('\n') => reconsume().and_emit(Token::EOLComment), 
            _ => consume().and_continue(),
        }

        Unknown => {
            let retry = begin_lex_from_top(chunk);

            match retry.next_action{
                LexAction::Transition(Unknown) => consume().and_continue(),
                _ => reconsume().and_emit(Token::UnknownChunk),
            }
        }

        EOF => {
            reconsume().and_eof()
            // unreachable!()
        },
    }
}



fn consume() -> LexBite{
    LexBite::Comsume
}
fn reconsume() -> LexBite{
    LexBite::Reconsume
}

impl LexBite{
    fn and_emit(self, tk: Token) -> LexNext{
        LexNext{
            next_bite: self,
            next_action: LexAction::Transition(LexState::Top),
            emit_token: Some(tk),
        }
    }

    // fn and_push(self, s: LexState) -> LexNext{
    //     LexNext{
    //         next_action: LexAction::PushState(s),
    //         next_bite  : self,
    //         emit_token : None,
    //     }
    // }
    // fn and_pop(self) -> LexNext{
    //     LexNext{
    //         next_action: LexAction::PopState,
    //         next_bite  : self,
    //         emit_token : None,
    //     }
    // }
    fn and_trans(self, s: LexState) -> LexNext{
        LexNext{
            next_action: LexAction::Transition(s),
            next_bite : self,
            emit_token : None,
        }
    }
    fn and_continue(self) -> LexNext{
        LexNext{
            next_action: LexAction::Continue,
            next_bite : self,
            emit_token : None,
        }
    }

    fn and_eof(self) -> LexNext{
        self.and_trans(LexState::EOF)
    }
}


enum LexAction{
    // PushState(LexState), // record current and transition to next
    // PopState,
    Transition(LexState),
    Continue,
}


// @todo: eliminate invalid state of LexAction::Continue and emit_token: Some(Token)
struct LexNext{
    next_action: LexAction,
    next_bite:   LexBite,
    emit_token:  Option<Token>,
}

// impl LexNext{
//     fn and_emit(self, tk: Token) -> LexNext{
//         LexNext{
//             next_action: self.next_action,
//             next_bite:  self.next_bite,
//             emit_token: Some(tk),
//         }
//     }
// }


fn is_sigil(c: char) -> bool{
    match c{
        '<'
        | '>'
        | '=' 
        | '|' 
        | '+'
        | '-'
        | '*' 
        | '/'
        | '^'
        | '%'
        | '?'
        | '!'
        | '&'
        | '@' => true,
        _ => false,
    }
}


// fn is_keyword(word: &str) -> bool{
//     match word{
//         "fun"
//         | "extern"
//         | "export"
//         | "mod"
//         | "exposing"
//         | "hiding"
//         | "from"
//         | "import"
//         | "use"
//         | "as"
//         | "val"
//         | "mut"
//         | "let"
//         | "in"
//         | "do"
//         | "case"
//         | "of"
//         | "if"
//         | "then"
//         | "else"
//         | "loop"
//         | "continue"
//         | "break"
//         | "type"
//         | "alias"
//         | "enum"
//         | "data"
//         | "flag"
//         | "pub"
//         | "trait"
//             => true,
//         _   => false,
//     }
// }



// #[test]
// fn test_lex_char() {
    
//     let src = "'a''b''\u{0306}'";
//     let stream = Tokenizer::new(src);

//     let tokens: Vec<_> = stream.into_iter().collect(); 

//     assert_eq!(tokens[0].kind, Token::Char);
//     assert_eq!(tokens[0].span, "'a'");
//     assert_eq!(tokens[1].kind, Token::Char);
//     assert_eq!(tokens[1].span, "'b'");
//     assert_eq!(tokens[2].kind, Token::Char);
//     assert_eq!(tokens[2].span, "'\u{0306}'");
// }

// #[test]
// fn test_lex_whitespace() {
    
//     let src = "  \t\n  \n\t\r\n\n -- a comment";
//     let stream = Tokenizer::new(src);

//     let tokens: Vec<_> = stream.into_iter().collect(); 

//     assert_eq!(tokens[0].kind, Token::Whitespace);
//     assert_eq!(tokens[1].kind, Token::Whitespace);

//     // assert_eq!(tokens[2].kind, Token::Whitespace);
//     assert_eq!(tokens[2].kind, Token::Whitespace);
//     // assert_eq!(tokens[4].kind, Token::Whitespace);
//     assert_eq!(tokens[3].kind, Token::Whitespace);
//     assert_eq!(tokens[4].kind, Token::Whitespace);
// }

// #[test]
// fn test_lex_basic_int() {
    
//     let src = "123123400";
//     let stream = Tokenizer::new(src);

//     let tokens: Vec<_> = stream.into_iter().collect(); 

//     assert_eq!(tokens[0].kind, Token::Int);
//     assert_eq!(tokens[0].span, src);
// }

// #[test]
// fn test_lex_basic_identifier() {
    
//     let answer = "_a_Name'";
//     let src = "_a_Name' \n ";
//     let stream = Tokenizer::new(src);

//     let tokens: Vec<_> = stream.into_iter().collect(); 

//     assert_eq!(tokens[0].kind, Token::Ident);
//     assert_eq!(tokens[0].span, answer);
// }

// #[test]
// fn test_lex_basic_float() {
    
//     let src = "123123400.123123400";
//     let stream = Tokenizer::new(src);

//     let tokens: Vec<_> = stream.into_iter().collect(); 

//     assert_eq!(tokens[0].kind, Token::Float);
//     assert_eq!(tokens[0].span, src);
// }

// #[test]
// fn test_lex_basic_string() {
    
//     let src = "\"ab\u{0306}'\"";
//     let stream = Tokenizer::new(src);

//     let tokens: Vec<_> = stream.into_iter().collect(); 

//     assert_eq!(tokens[0].kind, Token::Str);
//     assert_eq!(tokens[0].span, src);
// }

// #[test]
// fn test_lex_unknown_error() {
    
//     let src = "??&%";
//     let stream = Tokenizer::new(src);

//     let tokens: Vec<_> = stream.into_iter().collect(); 

//     assert_eq!(tokens[0].kind, Token::Sigil);
//     assert_eq!(tokens[0].span, src);
// }

#[test]
fn test_lex_basic_ops_and_line_comment() {
    
    let sigil_answer = "=*/<=+==";
    let src = "=*/<=+== // a giant comment\n";
    let stream = Tokenizer::new(src);

    let tokens: Vec<_> = stream.into_iter().collect(); 

    assert_eq!(tokens[0].kind, Token::Sigil);
    assert_eq!(tokens[0].span, sigil_answer);
    assert_eq!(tokens[2].kind, Token::EOLComment);
    assert_eq!(tokens[3].kind, Token::Newline);
}

#[test]
fn test_lex_basic_ops_and_colon() {
    
    let src = ":=,:";
    let stream = Tokenizer::new(src);

    let tokens: Vec<_> = stream.into_iter().collect(); 

    assert_eq!(tokens[0].kind, Token::Sigil);
    assert_eq!(tokens[1].kind, Token::Comma);
    assert_eq!(tokens[2].kind, Token::ColonSingle);
}

// #[test]
// fn test_lex_basic_delim() {
    
//     let src = "()[],:::->=>;";
//     let stream = Tokenizer::new(src);

//     let tokens: Vec<_> = stream.into_iter().collect();

//     assert_eq!(tokens[0].kind, Token::OpenParen);
//     assert_eq!(tokens[1].kind, Token::ClosedParen);
//     assert_eq!(tokens[2].kind, Token::OpenSquare);
//     assert_eq!(tokens[3].kind, Token::ClosedSquare);

//     assert_eq!(tokens[4].kind, Token::Comma);
//     assert_eq!(tokens[5].kind, Token::ColonSingle);  
//     assert_eq!(tokens[6].kind, Token::ColonSingle);
//     assert_eq!(tokens[7].kind, Token::ColonSingle);
//     assert_eq!(tokens[8].kind, Token::TypeArrow);
//     assert_eq!(tokens[9].kind, Token::Sigil);
//     // assert_eq!(tokens[10].kind, Token::SemiColon);
//     assert_eq!(tokens[10].kind, Token::UnknownChunk);
// }