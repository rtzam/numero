
use self::ast::Ptr;

#[derive(Debug)]
pub enum IntWidth{
    Int8,
    Int16,
    Int32,
    Int64,
}

#[derive(Debug)]
pub enum FloatWidth{
    Float32,
    Float64
}

#[derive(Debug)]
pub enum TypeKind<'s>{
    Int(IntWidth),
    Float(FloatWidth),
    Char,
    Enum(&'s str),
    Array(Type<'s>), //Option<usize>, 
    Struct(&'s str),
}

pub struct Type<'s>{
    kind: Ptr<TypeKind<'s>>,
}