
use std::collections::HashMap;

use crate::ast::ops::BinaryOp;

// Lifetime will be useful for parsing custom ops
#[derive(Debug, Clone)]
pub struct BinOpPrec<'s>{
    // prec: HashMap<BinaryOp, i32>,
    ops: HashMap<&'s str, BinaryOp>,
}

impl<'s> BinOpPrec<'s>{
    pub fn init() -> Self{
        let num_ops = 10;
        // let mut prec_map = HashMap::with_capacity(num_ops);
        // let mut count = 1;
        
        // // must start from 1
        // // 0 is reserved for user defined 
        // prec_map.insert(BinaryOp::Assign, count);
        // count += 1; 

        // prec_map.insert(BinaryOp::LogicalAnd, count);
        // prec_map.insert(BinaryOp::LogicalOr, count);
        // count += 1;

        // prec_map.insert(BinaryOp::Et, count);
        // prec_map.insert(BinaryOp::Lt, count);
        // prec_map.insert(BinaryOp::LtEt, count);
        // count += 1;

        // prec_map.insert(BinaryOp::Add, count);
        // prec_map.insert(BinaryOp::Sub, count);
        // count += 1;
        // prec_map.insert(BinaryOp::Mul, count);
        // prec_map.insert(BinaryOp::Div, count);
        // count += 1;

        let mut ops_map = HashMap::with_capacity(num_ops);
        ops_map.insert("=", BinaryOp::Assign);
        ops_map.insert("+", BinaryOp::Add);
        ops_map.insert("-", BinaryOp::Sub);
        ops_map.insert("*", BinaryOp::Mul);
        ops_map.insert("/", BinaryOp::Div);
        ops_map.insert("&&", BinaryOp::LogicalAnd);
        ops_map.insert("||", BinaryOp::LogicalOr);
        ops_map.insert("<=", BinaryOp::LtEt);
        ops_map.insert("<", BinaryOp::Lt);
        ops_map.insert("==", BinaryOp::Et);

        Self{
            ops: ops_map,
            // prec: prec_map, 
        }
    }

    pub fn get_precedence(&self, bo: BinaryOp) -> i32{
        use BinaryOp::*;
        match bo{
            Assign => 1,
            LogicalAnd | LogicalOr => 2,
            Et | Lt| LtEt => 3,
            Add | Sub => 4,
            Mul | Div => 5,
        }
    }
    pub fn get_op_from_span(&self, op: &'s str) -> Option<&BinaryOp>{
        self.ops.get(op)
    }
}