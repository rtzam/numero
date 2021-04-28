use super::Ptr;
use super::ops::{BinaryOp};
use super::token::{TokenData};
use super::node::NodeId;

// pub enum LitKind{
//     Str,
//     Char,
//     Int,
//     Float
// }

// pub enum PlaceKind<'s>{
//     Name(TokenData<'s>),
//     Wildcard
// }

// pub struct Place<'s>{
//     pub path: Vec<TokenData<'s>>,
// }

#[derive(Clone, Copy)]
pub enum MutKind{
    Mutable,
    Const,
}

pub struct VarDecl<'s>{
    pub mutable: MutKind,
    pub bound: TokenData<'s>, 
    pub value: Expr<'s>,
}

impl<'s> VarDecl<'s>{
    pub fn new(m: MutKind, b: TokenData<'s>, val: Expr<'s>) -> Self{
        Self{
            mutable: m,
            bound: b,
            value: val,
        }
    }
}

// pub enum CtrlFlow{
//     Continue,
//     Break,
// }

pub enum ExprKind<'s>{
    // Place(Place<'s>),
    Var(TokenData<'s>),
    Lit(f64),
    Binary{op: BinaryOp, lhs: Expr<'s>, rhs: Expr<'s>},
    Call{callee: Expr<'s>, args: Vec<Expr<'s>>},
    If{cond: Expr<'s>, if_body: Expr<'s>, else_body: Expr<'s>},
    Block(Vec<Expr<'s>>),
    Decl(VarDecl<'s>),
    Let{bound: Vec<Expr<'s>>, let_body: Expr<'s>},
    While{cond: Expr<'s>, while_body: Expr<'s>},
}

pub struct Expr<'s>{
    pub kind: Ptr<ExprKind<'s>>,
    pub nid: NodeId,
}

impl<'s> Expr<'s>{
    pub fn new(nid: NodeId, ek: ExprKind<'s>) -> Self{
        Expr{
            kind: Ptr::new(ek),
            nid: nid,
        }
    }
}