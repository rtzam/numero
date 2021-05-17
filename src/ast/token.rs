


// type TokTagRepr = usize;
// #[derive(Debug, Clone, Copy, PartialEq)]
// pub struct TokTag(TokTagRepr);
// mod tok_span;
// pub use tok_span::{TokenTable, TokenSpan, TokTag};


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineEndKind{
    Newline,  
    SemiColon,
}

#[derive(Debug, Clone, Copy,PartialEq)]
pub enum LitKind{
    Int,
    Char,
    Float,
    Str,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Delim{
    Paren,
    Square,
    Curly,
    Angle, 
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KwKind{
    Fun,
    Extern,
    End,
    Mod,
    Do,
    If,
    Else,
    Val,
    Let,
    In,
    Mut,
    // Loop,
    While,
    // modules
    // From,
    // Import,
    // Export
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Token{
    // EOF,
    EOLComment,
    Whitespace,
    Newline,  
    SemiColon,
    // LineEnd(LineEndKind),
    Ident,
    Assigner,
    Sigil,
    Dot,
    // TypeArrow,  // ->
    // WideArrow,  // =>
    
    // Comment,
    Literal(LitKind),

    // delimiter
    OpenParen,
    ClosedParen,
    // OpenSquare,
    // ClosedSquare,
    Comma,
    ColonSingle,
    // ColonDouble,
    // Pipe,

    Kw(KwKind),

    // Errors
    // UnclosedChar,
    // UnclosedStr,
    UnknownChunk,
}



#[derive(Debug, Clone, Copy)]
pub struct TokenData<'s>{
    pub kind: Token,
    pub span: &'s str,
    pub loc: TokenLoc,
}

impl<'s> TokenData<'s>{
    pub fn new(k: Token, s: &'s str, l: TokenLoc) -> TokenData<'s>{
        Self{
            kind: k,
            span: s,
            loc:  l,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TokenLoc{
    pub line: usize,
    pub column: usize,
}

impl TokenLoc{
    pub fn new() -> TokenLoc{
        TokenLoc{
            line: 1,
            column: 1,
        }
    }
    // pub fn from_pos(l: usize, c: usize) -> TokenLoc{
    //     TokenLoc{
    //         line:   l,
    //         column: c,
    //     }
    // }

    pub fn next_line(&self) -> Self{
        TokenLoc{
            line: self.line + 1,
            column: 1,
        }
    }

    pub fn next_col(&self) -> Self{
        TokenLoc{
            line: self.line,
            column: self.column + 1,
        }
    }
}