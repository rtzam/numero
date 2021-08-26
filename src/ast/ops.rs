#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum BinaryOp {
    // memory
    Assign,
    // AccessFieldOp,

    // arithmatic
    Add,
    Sub,
    Mul,
    Div,

    // logical
    LogicalAnd,
    LogicalOr,

    // comparison
    Et,
    Lt,
    LtEt,
}

// #[derive(Debug)]
// pub enum UnaryOp{
//     LogicalNotOp,

//     // memory ops
//     AddrOfOp,
//     DerefOp,
// }
