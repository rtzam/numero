use super::expr::Expr;
use super::node::NodeId;
use super::token::TokenData;

pub struct FuncArg<'s> {
    pub name: TokenData<'s>,
    // ty: Option<Type>,
    pub nid: NodeId,
}

impl<'s> FuncArg<'s> {
    pub fn as_str(&self) -> &'s str {
        self.name.span
    }
}

impl<'s> FuncArg<'s> {
    pub fn new(nid: NodeId, t: TokenData<'s>) -> Self {
        FuncArg { name: t, nid }
    }
}

pub struct FuncProto<'s> {
    pub name: TokenData<'s>,
    pub args: Vec<FuncArg<'s>>,
}

pub struct Function<'s> {
    pub proto: FuncProto<'s>,
    pub body: Expr<'s>,
}

// pub enum ImportStmt<'s>{
//     SimpleImport(Place<'s>),
//     FromImport(Place<'s>, Vec<TokenData<'s>>),
// }

pub enum ItemKind<'s> {
    Func(Function<'s>),
    Extern(FuncProto<'s>),
    // Import(ImportStmt<'s>),
}

pub struct Item<'s> {
    pub kind: ItemKind<'s>,
    pub nid: NodeId,
}

impl<'s> Item<'s> {
    pub fn new(nid: NodeId, ik: ItemKind<'s>) -> Self {
        Item { kind: ik, nid }
    }
}

// pub struct ModDecl<'s>{
//     pub name: Place<'s>,
//     pub exports: Vec<TokenData<'s>>,
// }

pub struct Module<'s> {
    pub decl: TokenData<'s>, //ModDecl<'s>,
    pub body: Vec<Item<'s>>,
}
