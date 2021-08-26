use super::Delimiter;
use crate::ast::token::Token;

macro_rules! delimiter_pair {
    ($(($v:vis struct $name:ident, $open_kind:path, $close_kind:path),)*) => {
        $(
            $v struct $name;

            impl<'s> Delimiter<'s> for $name{
                type Open = Token;
                type Close = Token;
                fn open_syntax(&self) -> Self::Open{
                    $open_kind
                }
                fn close_syntax(&self) -> Self::Close{
                    $close_kind
                }
            }
        )*
    };
}

delimiter_pair! {
    (pub struct ParenDelim, Token::OpenParen, Token::ClosedParen),
}
