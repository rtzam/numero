
use crate::ast::token::{Token, TokenData, LitKind, KwKind, TokenLoc};
use crate::ast::ops::BinaryOp;
use crate::parse::{Parser, ParseResult, RecoveryInfo, PeekOpPrec};
use crate::ast::{
    Expr, ExprKind, VarDecl, MutKind, // Place,
    FuncArg, FuncProto, Function,
    ItemKind, Item,
    Module, // ModDecl, 
    // ImportStmt,
};


// values can either be assignable or no assignable
fn parse_stmt<'s>(p: &mut Parser<'s>) -> ParseResult<Expr<'s>>{
    match p.peek(){
        Some(tok) => match tok{
            Token::Kw(kw) => match kw{
                KwKind::If => parse_if_expr(p),
                KwKind::Let => parse_let_expr(p),
                KwKind::While => parse_while_stmt(p),
                KwKind::Mut => {
                    let decl = parse_var_decl(p)?;
                    Ok(p.new_expr(ExprKind::Decl(decl)))
                },
                _ => unimplemented!("Bad Keyword"),
            }

            // either expression or 
            // const var_decl
            Token::Ident => {
                parse_expr_or_var_decl(p)
            }

            // try to parse expr
            _ => parse_expr(p),
        }

        _ => {
            let msg = format!("Early EOF when expecting Stmt");
            Err(p.report_error(RecoveryInfo::EarlyEOF, msg))
        }
    }
}


fn parse_expr_or_var_decl<'s>(p: &mut Parser<'s>) -> ParseResult<Expr<'s>>{
    match p.peek_next(){
        Some(td) => {
            if td.span == ":="{
                let decl = parse_const_var_decl(p)?;
                Ok(p.new_expr(ExprKind::Decl(decl)))
            } else{
                parse_expr(p)
            }
        }
        None => {
            let msg = format!("Early EOF after Ident expected VarDecl or Expression");
            Err(p.report_error(RecoveryInfo::EarlyEOF, msg))
        }
    }
}

// values must be assignable
fn parse_expr<'s>(p: &mut Parser<'s>) -> ParseResult<Expr<'s>>{
    
    match p.peek(){
        Some(tok) => match tok {
            Token::Kw(kw) => match kw{
                KwKind::If => parse_if_expr(p),
                KwKind::Let => parse_let_expr(p),
                KwKind::Do => parse_do_expr(p),
                _ => unimplemented!("Bad Keyword {:?}", kw),
            }
            _ => parse_binary_expr(p),
        }
        None => Err(p.report_error(
            RecoveryInfo::EarlyEOF, 
            format!("Early EOF while parsing expression"))),
    }
}


fn parse_binary_expr<'s>(p: &mut Parser<'s>) -> ParseResult<Expr<'s>>{
    let lhs = parse_binary_lhs(p)?;

    parse_binary_rhs(p, lhs, -1)
}

fn parse_binary_lhs<'s>(p: &mut Parser<'s>) -> ParseResult<Expr<'s>>{

    match p.peek(){
        Some(tok) => match tok{
            Token::OpenParen | Token::Ident | Token::Literal(_) 
                => parse_primary_expr(p), 
            Token::Sigil => unimplemented!("No unary ops supported"),
            _ => Err(p.report_error(
                RecoveryInfo::InvalidToken, 
                format!("Cannot parse token {:?} as part of binary expr", tok))),
        }

        _ => Err(p.report_error(
            RecoveryInfo::EarlyEOF, 
            format!("Early EOF while parseing binary expr"))),
    }
}

fn parse_primary_expr<'s>(p: &mut Parser<'s>) -> ParseResult<Expr<'s>>{
    // eprintln!("parse: primary expr");
    let lead = match p.peek(){
        Some(tok) => match tok{
            Token::OpenParen    => parse_paren_expr(p)?,
            Token::Ident        => parse_ident_expr(p)?, 
            Token::Literal(_)   => parse_literal_expr(p)?,
            Token::Sigil => unimplemented!("No unary ops supported"),
            _ => return Err(p.report_error(
                RecoveryInfo::InvalidToken, 
                format!("Cannot parse token {:?} as part of an expr", tok))),
        }
        _ => return Err(p.report_error(
            RecoveryInfo::EarlyEOF, 
            format!("Early EOF while parseing primary expr"))),
    };

    parse_trailing_expr(p, lead)
}

fn parse_ident_expr<'s>(p: &mut Parser<'s>) -> ParseResult<Expr<'s>>{
    let ident = p.expect_tok(Token::Ident)?;
    // eprintln!("parse: {:?}", ident);
    Ok(p.new_expr(ExprKind::Var(ident)))
}

fn parse_literal_expr<'s>(p: &mut Parser<'s>) -> ParseResult<Expr<'s>>{
    // eprintln!("parse: literal");
    match p.peek(){
        Some(Token::Literal(lit)) => match lit{
            LitKind::Float => {
                let lit = p.expect_tok(Token::Literal(LitKind::Float))?;
                let fp = lit.span.parse().unwrap();
                Ok(p.new_expr(ExprKind::Lit(fp)))
            }
            LitKind::Int => {
                let lit = p.expect_tok(Token::Literal(LitKind::Int))?;
                let fp = lit.span.parse().unwrap();
                Ok(p.new_expr(ExprKind::Lit(fp)))
            }
            _ => unimplemented!("Invalid literal, only float and int supported")
        }
        _ => unreachable!()
    }
}

fn parse_trailing_expr<'s>(p: &mut Parser<'s>, first_lead: Expr<'s>) -> ParseResult<Expr<'s>>{
    
    let mut new_lead = first_lead;
    loop{
        match p.peek(){
            Some(tok) => match tok{
                Token::OpenParen  => {
                    let call = parse_call_expr(p, new_lead)?;
                    new_lead = p.new_expr(call)
                },
                _ => return Ok(new_lead),
            }
            _ => return Ok(new_lead),
        }
    }
}

// fn get_precedence_from_span_wrapper<'s>(p: &mut Parser<'s>, span: &'s str) -> ParseResult<i32>{
//     match p.get_precedence_from_span(span){
//         Some(prec) => Ok(prec),
//         _ => return Err(p.report_error(
//             RecoveryInfo::InvalidOp,
//             format!("unsupported operator {:?}", span))),
//     }
// } 

// TODO: refactor, change algo, make this cleaner
fn parse_binary_rhs<'s>(p: &mut Parser<'s>, lhs: Expr<'s>, old_prec: i32) -> ParseResult<Expr<'s>>{
    
    // TODO: refactor to avoid doubling lexing effort
    let prec = match p.peek_op_precedence(){
        PeekOpPrec::Prec(prec) => prec,
        PeekOpPrec::BadOp => return {
            let tok = p.peek_data();
            match tok {
                Some(td) =>{
                    let msg = format!("unsupported operator {:?}", td.span);
                    Err(p.report_error(RecoveryInfo::InvalidOp, msg))
                }
                None => {
                    let msg = format!("Early EOF while parsing BinaryExpr");
                    Err(p.report_error(RecoveryInfo::InvalidOp, msg))
                }
            }
        },
        PeekOpPrec::ExprEnd => return Ok(lhs),
    };

    // first iteration CANNOT satisfy this condition
    // old_prec = -1
    if prec < old_prec{
        return Ok(lhs);
    }

    let op  = parse_binary_op(p)?;
    let mut rhs = parse_binary_lhs(p)?;

    let next_prec = match p.peek_op_precedence(){
        PeekOpPrec::Prec(prec) => prec,
        PeekOpPrec::BadOp => return {
            let tok = p.peek_data();
            match tok {
                Some(td) =>{
                    let msg = format!("unsupported operator {:?}", td.span);
                    Err(p.report_error(RecoveryInfo::InvalidOp, msg))
                }
                None => {
                    let msg = format!("Early EOF while parsing BinaryExpr");
                    Err(p.report_error(RecoveryInfo::InvalidOp, msg))
                }
            }
        },
        PeekOpPrec::ExprEnd => return Ok(p.new_expr(ExprKind::Binary{op:op, lhs:lhs, rhs:rhs})),
    };

    // if this op binds less tightly, compute next expr first
    if prec < next_prec{
        rhs = parse_binary_rhs(p, rhs, prec+1)?;
    }

    let new_lhs = p.new_expr(ExprKind::Binary{op:op, lhs:lhs, rhs:rhs});

    // TODO: Tail Call Opt
    parse_binary_rhs(p, new_lhs, old_prec)
}

fn parse_binary_op<'s>(p: &mut Parser<'s>) -> ParseResult<BinaryOp>{
    let td = p.expect_tok(Token::Sigil)?;
    let op = match p.get_op_from_span(td.span){
        Some(op) => op,
        _ => return Err(p.report_error(
            RecoveryInfo::InvalidOp,
            format!("unsupported operator {:?}", td.span))),
    };
    Ok(op)
}

fn parse_paren_expr<'s>(p: &mut Parser<'s>) -> ParseResult<Expr<'s>>{
    p.expect_tok(Token::OpenParen)?;
    let subexpr = parse_expr(p)?;
    p.expect_tok(Token::ClosedParen)?;

    Ok(subexpr)
}

fn parse_call_expr<'s>(p: &mut Parser<'s>, callee: Expr<'s>) -> ParseResult<ExprKind<'s>>{
    p.expect_tok(Token::OpenParen)?;
    let args = parse_call_args(p)?;
    p.expect_tok(Token::ClosedParen)?;
    // eprintln!("parse: parsed Call");
    Ok(ExprKind::Call{callee:callee, args:args})
}

// TODO: refactor to comma list of
fn parse_call_args<'s>(p: &mut Parser<'s>) -> ParseResult<Vec<Expr<'s>>>{
    let mut args = Vec::new();
    loop{
        match p.peek(){
            Some(Token::Comma) => {
                p.shift();
            },
            Some(Token::ClosedParen) => break,
            Some(_) => args.push(parse_expr(p)?),
            None => return Err(p.report_error(
                RecoveryInfo::EarlyEOF, 
                format!("EOF while parsing call args"))),
        }
    }

    Ok(args)
}




fn parse_func_arg<'s>(p: &mut Parser<'s>) -> ParseResult<FuncArg<'s>>{
    let td = p.expect_tok(Token::Ident)?;
    // optional type

    Ok(FuncArg::new(p.nid.shift(), td))
}

// TODO: refactor to comma list of
fn parse_many_func_args<'s>(p: &mut Parser<'s>) -> ParseResult<Vec<FuncArg<'s>>>{
    
    let mut args = Vec::new();
    loop{
        match p.peek(){
            Some(Token::Ident) => args.push(parse_func_arg(p)?),
            Some(Token::Comma) => {
                p.shift();
            },
            _ => break,
        }
    }

    Ok(args)
}

fn parse_func_proto<'s>(p: &mut Parser<'s>) -> ParseResult<FuncProto<'s>>{
    p.expect_kw(KwKind::Fun)?;
    let name = p.expect_tok(Token::Ident)?;

    // TODO: refactor to a paren list of comma
    p.expect_tok(Token::OpenParen)?;
    let args = parse_many_func_args(p)?;
    p.expect_tok(Token::ClosedParen)?;

    // TODO: optional return type

    // TODO: expect line end
    p.expect_line_end()?;

    // eprintln!("parse: parsed FuncProto");
    Ok(FuncProto{name:name, args: args})
}


fn parse_function<'s>(p: &mut Parser<'s>) -> ParseResult<Function<'s>>{
    let proto = parse_func_proto(p)?;
    let body = parse_func_body(p)?;
    Ok(Function{proto:proto, body:body})
}

fn parse_extern_func_proto<'s>(p: &mut Parser<'s>) -> ParseResult<FuncProto<'s>>{
    p.expect_kw(KwKind::Extern)?;
    parse_func_proto(p)
}


pub fn parse_module<'s>(p: &mut Parser<'s>) -> ParseResult<Module<'s>>{
    // let mod_decl = parse_module_decl(p)?;
    p.expect_kw(KwKind::Mod)?;
    let mod_decl = p.expect_tok(Token::Ident)?;
    let body = parse_module_body(p)?;
    // eprintln!("parse: completed Mod {:?}", mod_name.span);
    Ok(Module{decl: mod_decl, body: body})
}


// fn parse_module_decl<'s>(p: &mut Parser<'s>) -> ParseResult<ModDecl<'s>>{
//     p.expect_kw(KwKind::Mod)?;
//     let mod_name = parse_place(p)?;
//     let exports = parse_mod_exports(p)?;

//     Ok(ModDecl{name: mod_name, exports:exports})
// }

// fn parse_mod_exports<'s>(p: &mut Parser<'s>) -> ParseResult<Vec<TokenData<'s>>>{

//     p.expect_kw(KwKind::Export)?;
//     p.expect_tok(Token::OpenParen)?;
//     let mut exports = Vec::new();
//     loop{
//         match p.peek(){
//             Some(Token::Ident) => exports.push(p.shift().unwrap()),
//             Some(Token::Comma) | Some(Token::Newline) => {
//                 p.shift();
//             },
//             Some(Token::ClosedParen) => {
//                 p.shift();
//                 break;
//             }
//             // ERROR
//             _ => unimplemented!("Invalid token in module exports"),
//         }
//     }

//     Ok(exports)
// }

fn parse_module_body<'s>(p: &mut Parser<'s>) -> ParseResult<Vec<Item<'s>>>{
    let mut body = Vec::new();
    loop{
        let new_item = match p.peek(){
            Some(Token::Newline) => {
                // eprintln!("parse: Skipping newline");
                p.shift();
                continue
            },
            Some(Token::Kw(KwKind::Fun)) => {
                // eprintln!("parse: Function");
                let func = parse_function(p)?;
                // eprintln!("parse: parsed function {:?}", func.proto.name);
                ItemKind::Func(func)
            }
            Some(Token::Kw(KwKind::Extern)) => {
                // eprintln!("parse: Extern");
                let proto = parse_extern_func_proto(p)?;
                ItemKind::Extern(proto)
            }
            Some(tok) => return Err(p.report_error(
                RecoveryInfo::InvalidToken, 
                format!("Unsupported Top-Level expr beginning with {:?}", tok))),
            _ => break,
        };

        body.push(Item::new(p.nid.shift(), new_item));
    }

    Ok(body)
}


pub fn parse_repl_line<'s>(p: &mut Parser<'s>) -> ParseResult<Vec<Item<'s>>>{
    let mut body = Vec::new();
    loop{
        let new_item = match p.peek(){
            Some(Token::Newline) => {
                p.shift();
                continue
            },
            Some(Token::Kw(KwKind::Fun)) => { 
                let func = parse_function(p)?;
                ItemKind::Func(func)
            }
            Some(Token::Kw(KwKind::Extern)) => {
                let proto = parse_extern_func_proto(p)?;
                ItemKind::Extern(proto)
            }
            Some(_) => {
                // Construct anonymous function
                let expr = parse_expr(p)?;
                let func = build_anon_func(expr);
                ItemKind::Func(func)
            },
            _ => break,
        };

        body.push(Item::new(p.nid.shift(), new_item));
    }

    Ok(body)
}

// TODO: add name
fn build_anon_func<'s>(body: Expr<'s>) -> Function<'s>{
    let name_data = TokenData::new(Token::Ident, "", TokenLoc::new());
    let proto = FuncProto{name: name_data, args: Vec::new()};

    Function{proto:proto, body:body}
}

fn parse_func_body<'s>(p: &mut Parser<'s>) -> ParseResult<Expr<'s>>{
   let body = parse_ended_stmt_block(p)?;
   Ok(p.new_expr(ExprKind::Block(body))) 
}

fn parse_do_expr<'s>(p: &mut Parser<'s>) -> ParseResult<Expr<'s>>{
    p.expect_kw(KwKind::Do)?;
    let body = parse_ended_stmt_block(p)?;
    Ok(p.new_expr(ExprKind::Block(body)))
}

fn parse_ended_stmt_block<'s>(p: &mut Parser<'s>) -> ParseResult<Vec<Expr<'s>>>{
    let body = parse_many_stmt(p)?;
    
    p.expect_kw(KwKind::End)?;
    // cannot require newline here
    // because an ended block is an expression that returns a value
    // p.expect_line_end()?;

    Ok(body)
}

fn parse_many_stmt<'s>(p: &mut Parser<'s>) -> ParseResult<Vec<Expr<'s>>>{
    let mut body = Vec::new();
    loop{
        p.skip_newlines();
        match p.peek(){
            Some(tok) => match tok{
                // kw else is only valid
                // after an if
                Token::Kw(KwKind::End) | Token::Kw(KwKind::Else) => break,
                _ => {
                    let n = parse_stmt(p)?;
                    p.expect_line_end()?;
                    body.push(n)
                }
            }
            _ => break,
        }
    }

    Ok(body)
}


fn parse_if_expr<'s>(p: &mut Parser<'s>) -> ParseResult<Expr<'s>>{
    let if_block = parse_if_expr_rec(p, 0)?;
    p.expect_kw(KwKind::End)?;
    Ok(if_block)
}

fn parse_if_expr_rec<'s>(p: &mut Parser<'s>, depth: u32) -> ParseResult<Expr<'s>>{
    p.expect_kw(KwKind::If)?;
    let cond = parse_expr(p)?;
    p.expect_line_end()?;
    let stmts = parse_many_stmt(p)?;
    let if_body = p.new_expr(ExprKind::Block(stmts));
    p.expect_kw(KwKind::Else)?;

    let else_body = match p.peek(){
        Some(Token::Kw(KwKind::If)) => {
            parse_if_expr_rec(p, depth+1)?
        }
        _ => {
            p.expect_line_end()?;
            let stmts = parse_many_stmt(p)?;
            p.new_expr(ExprKind::Block(stmts))
        } 
    };

    Ok(p.new_expr(ExprKind::If{cond, if_body, else_body}))
}


fn parse_var_decl<'s>(p: &mut Parser<'s>) -> ParseResult<VarDecl<'s>>{
    let try_mut = p.optional_tok(Token::Kw(KwKind::Mut));
    let mutability = match try_mut{
        Some(_) => MutKind::Mutable,
        None => MutKind::Const,
    };

    _parse_var_decl_with_mut(p, mutability)
}

fn parse_const_var_decl<'s>(p: &mut Parser<'s>) -> ParseResult<VarDecl<'s>>{
    _parse_var_decl_with_mut(p, MutKind::Const)
}


fn _parse_var_decl_operator<'s>(p: &mut Parser<'s>) -> ParseResult<TokenData<'s>>{
    // expect decl operator :=
    match p.peek_data(){
        Some(td) => {
            if td.span == ":="{
                Ok(p.shift().unwrap())
            } else{
                let msg = format!("Expected := operator but found {:?} instead", td);
                let report = p.report_error(RecoveryInfo::InvalidToken, msg);
                return Err(report)
            }
        }
        _ => return Err(p.report_error(RecoveryInfo::EarlyEOF, 
            format!("Early EOF while parsing var decl, expected := operator")))
    }
}

fn _parse_var_decl_with_mut<'s>(p: &mut Parser<'s>, mutability: MutKind) -> ParseResult<VarDecl<'s>>{
    let bound = p.expect_tok(Token::Ident)?;
    
    _parse_var_decl_operator(p)?;
    
    let value = parse_expr(p)?;

    Ok(VarDecl{
        mutable: mutability,
        bound: bound,
        value: value,
    })
}


fn parse_let_expr<'s>(p: &mut Parser<'s>) -> ParseResult<Expr<'s>>{
    p.expect_kw(KwKind::Let)?;
    p.expect_line_end()?;

    let bound_parts = parse_let_var_decl_block(p)?;
    let bound = bound_parts.into_iter().map(|e| p.new_expr(ExprKind::Decl(e))).collect();

    p.expect_kw(KwKind::In)?;
    p.expect_line_end()?;

    let let_body_part = parse_ended_stmt_block(p)?;
    let let_body = p.new_expr(ExprKind::Block(let_body_part));

    Ok(p.new_expr(ExprKind::Let{bound, let_body}))
}


fn parse_let_var_decl_block<'s>(p: &mut Parser<'s>) -> ParseResult<Vec<VarDecl<'s>>>{
    let mut buffer = Vec::new();
    loop{
        match p.peek(){
            Some(Token::Ident) | Some(Token::Kw(KwKind::Mut)) => {
                buffer.push(parse_const_var_decl(p)?);
                p.expect_line_end()?;
            }
            None => {
                // this will always be an error
                return Err(p.report_error(
                    RecoveryInfo::EarlyEOF, 
                    format!("Hit EOF while parsing let expr")))
            }
            // TODO: catch Mut keyword in LetIn
            // TODO: this is an error unless
            // we hit token KwKind::In
            _ => break,
        }
    }

    Ok(buffer)
}


fn parse_while_stmt<'s>(p: &mut Parser<'s>) -> ParseResult<Expr<'s>>{
    p.expect_kw(KwKind::While)?;
    let cond = parse_binary_expr(p)?;
    p.expect_line_end()?;
    let while_body_part = parse_ended_stmt_block(p)?;
    let while_body = p.new_expr(ExprKind::Block(while_body_part));

    Ok(p.new_expr(ExprKind::While{cond, while_body}))
}


// fn parse_place<'s>(p: &mut Parser<'s>) -> ParseResult<Place<'s>>{
//     let mut body = Vec::new();

//     let top_level = p.expect_tok(Token::Ident)?;
//     body.push(top_level);

//     loop{
//         match p.peek(){
//             Some(Token::Dot) => {
//                 p.shift();
//                 let next_mod = p.expect_tok(Token::Ident)?;
//                 body.push(next_mod);
//             }
//             _ => break,
//         }
//     }
//     Ok(Place{path: body})
// }

// fn parse_place_as_expr<'s>(p: &mut Parser<'s>) -> ParseResult<Expr<'s>>{
//     let place_data = parse_place(p)?;
//     Ok(p.new_expr(ExprKind::Place(place_data)))
// }


// fn parse_from_import_stmt<'s>(p: &mut Parser<'s>) -> ParseResult<ImportStmt<'s>>{
//     p.expect_kw(KwKind::From)?;
//     let root_import = parse_place(p)?;
//     let imports = parse_from_import_clause(p)?;

//     Ok(ImportStmt::FromImport(root_import, imports))
// }


// fn parse_import_stmt<'s>(p: &mut Parser<'s>) -> ParseResult<ImportStmt<'s>>{
//     p.expect_kw(KwKind::Import)?;
//     let path = parse_place(p)?;
//     Ok(ImportStmt::SimpleImport(path))
// }


// fn parse_from_import_clause<'s>(p: &mut Parser<'s>) -> ParseResult<Vec<TokenData<'s>>>{
//     p.expect_kw(KwKind::Import)?;

//     let mut imports = Vec::new();
//     loop{
//         match p.peek(){
//             Some(Token::Ident) => imports.push(p.shift().unwrap()),
//             Some(Token::Comma) | Some(Token::Newline) => {
//                 p.shift();
//             },
//             Some(Token::ClosedParen) => {
//                 p.shift();
//                 break;
//             }
//             // ERROR
//             _ => unimplemented!("Invalid token in import clause"),
//         }
//     }

//     Ok(imports)
// }


// fn parse_import_list<'s>(p: &mut Parser<'s>) -> ParseResult<ImportStmt<'s>>{

//     p.expect_kw(KwKind::Import)?;

//     let mut imports = Vec::new();
//     loop{
//         match p.peek(){
//             Some(Token::Ident) => imports.push(parse_place(p)?),
//             Some(Token::Comma) | Some(Token::Newline) => {
//                 p.shift();
//             },
//             Some(Token::ClosedParen) => {
//                 p.shift();
//                 break;
//             }
//             // ERROR
//             _ => unimplemented!("Invalid token in import clause"),
//         }
//     }

//     Ok(imports)
// }
