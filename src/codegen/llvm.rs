
use std::collections::HashMap;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::{Module, Linkage};
use inkwell::passes::{PassManager, PassManagerBuilder};
use inkwell::{OptimizationLevel, FloatPredicate};
use inkwell::types::{FunctionType};
use inkwell::values::{FunctionValue, BasicValue, BasicValueEnum, FloatValue, PointerValue};
use inkwell::basic_block::BasicBlock;

// use crate::codegen::CodeGenerator;
use crate::ast::ops::BinaryOp;
use crate::ast;
use crate::ast::{
    ItemKind, ExprKind, NodeId, // Place
};
use crate::ast::symbol::{ModSymTable, SymId};

pub enum CompileError{
    TypeErr,
}

pub type BuildResult<T> = Result<T, CompileError>;

pub struct LlvmBackend{
    pub context: Context,
    opt: OptimizationLevel
}

impl LlvmBackend{
    pub fn default() -> Self{
        Self::new(OptimizationLevel::None)
    }

    pub fn new(opt: OptimizationLevel) -> Self{
        Self{
            context: Context::create(),
            opt: opt,
        }
    }
}

pub enum NameMangleOptions{
    Pretty,
    // Safe,
}


fn mangle_mod_name<'s>(p: &ast::token::TokenData<'s>, opts: NameMangleOptions) -> String{
    match opts{
        NameMangleOptions::Pretty => mangle_into_pretty_name(p),
        // NameMangleOptions::Safe => mangle_into_safe_name(p),
    }
    
}

fn mangle_into_pretty_name<'s>(p: &ast::token::TokenData<'s>) -> String{
    let mangled_name = String::from(p.span);
    // let mut mangled_name = String::with_capacity(p.path.len());
    // mangled_name.push_str(p.path.first().unwrap().span);

    // for idx in 1..p.path.len(){
    //     mangled_name.push('.');
    //     mangled_name.push_str(p.path[idx].span);
    // }
    mangled_name
}

// fn mangle_into_safe_name<'s>(p: &Place<'s>) -> String{
//     let mangled_name = String::new();
//     for part in p.path{
//         mangled_name.push_str(part.span);
//         mangled_name.push('.');
//     }
//     mangled_name
// }


impl<'s> LlvmBackend{
    pub fn compile_mod(&'s self, module: ast::Module<'s>, mst: &'s ModSymTable) -> Module<'s>{
        // let mod_name = mangle_mod_name(&module.decl.name, NameMangleOptions::Pretty);
        let mod_name = mangle_mod_name(&module.decl, NameMangleOptions::Pretty);
        let llvm_module = self.context.create_module(mod_name.as_str());

        let pass_manager_builder = PassManagerBuilder::create();
        pass_manager_builder.set_optimization_level(self.opt);
        let fpm = PassManager::create(&llvm_module);
        pass_manager_builder.populate_function_pass_manager(&fpm);

        let mut art = BuildState::new(&self.context, llvm_module, fpm, mst);
        
        for item in module.body{
            art.build_item(&item)
        }

        art.module
    }
}

pub struct BuildState<'c>{
    context: &'c Context,
    module: Module<'c>,
    mod_syms: &'c ModSymTable,
    builder: Builder<'c>,
    // func_sym: HashMap<&'c str, FunctionValue<'c>>
    // var_sym: HashMap<&'c str, PointerValue<'c>>,
    fpm: PassManager<FunctionValue<'c>>,
}

impl<'a, 'c> BuildState<'c>{
    fn new(
        context: &'c Context, 
        module: Module<'c>, 
        fpm: PassManager<FunctionValue<'c>>,
        mst: &'c ModSymTable,     
    ) -> Self{
        Self{
            context: context,
            module: module,
            mod_syms: mst,
            builder: context.create_builder(),
            // func_sym: HashMap::new(),
            // var_sym: HashMap::new(),
            fpm: fpm,
        }
    }

    fn build_item(&mut self, it: &ast::Item<'c>){
        match &it.kind{
            ItemKind::Func(f) => {
                self.build_function(f)
            }
            ItemKind::Extern(fp) => {
                self.build_extern(fp)
            }
            // ItemKind::Import(_) => (),
        }
    }

    fn build_function(&mut self, f: &'a ast::Function<'c>){
        let func = self.build_func_decl(&f.proto, None); 
        
        FuncBuild::new(self, func, f).build();

        if func.verify(true) {
            self.fpm.run_on(&func);
        } else {
            // unsafe {
            //     function.delete();
            // }
            self.module.print_to_stderr();
            panic!("Invalid generated function.")
        }
    }

    fn build_extern(&mut self, fp: &ast::FuncProto<'c>){
        self.build_func_decl(fp, None); // Some(Linkage::ExternalWeak)
    }

    fn build_func_decl(&mut self, fp: &ast::FuncProto<'c>, link: Option<Linkage>) -> FunctionValue<'c>{
        let ty = self.build_func_type(fp);

        let func_val = self.module.add_function(
            fp.name.span, 
            ty, 
            link
        );

        for (i, arg) in func_val.get_param_iter().enumerate(){
            arg.into_float_value().set_name(fp.args[i].name.span);
        }

        func_val
    }

    fn build_func_type(&mut self, fp: &ast::FuncProto<'c>) -> FunctionType<'c>{
        let f64_type = self.context.f64_type();

        let arg_types: Vec<_> = fp.args.iter()
            .map(|_| f64_type.into())
            .collect();
        
        f64_type.fn_type(arg_types.as_slice(), false)
    }
}

struct FuncBuild<'a, 'c>{
    state: &'a BuildState<'c>,
    func: FunctionValue<'c>,
    entry: BasicBlock<'c>,
    ast_func: &'a ast::Function<'c>,
    var_sym: HashMap<SymId, BasicValueEnum<'c>>,
}

impl<'a, 'c> FuncBuild<'a, 'c>{

    fn new(state: &'a BuildState<'c>, func: FunctionValue<'c>, ast_f: &'a ast::Function<'c>) -> Self{
        let entry = state.context.append_basic_block(func, "entry");
        Self{
            state: state,
            func: func,
            entry: entry,
            ast_func: ast_f,
            var_sym: HashMap::new(),
        }
    }

    fn builder(&self) -> &'a Builder<'c>{
        &self.state.builder
    }
    fn context(&self) -> &'c Context{
        &self.state.context
    }
    fn module(&self) -> &'a Module<'c>{
        &self.state.module
    }

    fn build(mut self) -> FunctionValue<'c>{
        self.builder().position_at_end(self.entry);

        // const arg only supported
        self.var_sym.reserve(self.ast_func.proto.args.len());
        for (i, arg_val) in self.func.get_param_iter().enumerate() {
            let arg = &self.ast_func.proto.args[i];

            let sid = match self.state.mod_syms.lookup(&arg.nid){
                Some(s) => *s,
                None => unimplemented!("NodeId => SymId lookup failed"),
            };
            
            // let alloca = self.create_entry_block_alloca(arg_name);
            // self.builder().build_store(alloca, arg);

            self.var_sym.insert(
                sid,
                arg_val, // BasicValueEnum::PointerValue(alloca),
            );
        }

        // compile body
        let body = self.build_expr(&self.ast_func.body);

        self.builder().build_return(Some(&body));

        self.func
    }

    fn create_entry_block_alloca(&mut self, name: &str) -> PointerValue<'c>{
        
        // not sure why this is needed
        // but Context and Builder manipulate global variables
        // bad LLVM design
        let builder = self.context().create_builder();

        match self.entry.get_first_instruction() {
            Some(first_instr) => builder.position_before(&first_instr),
            None => builder.position_at_end(self.entry)
        }

        builder.build_alloca(self.context().f64_type(), name)
    }

    fn build_expr(&mut self, expr: &ast::Expr<'c>) -> FloatValue<'c>{
        match &*expr.kind{
            // Place Can only be used in function calls
            // ExprKind::Place(_) => unreachable!("Place cannot be a valid expr yet..."),
            ExprKind::Var(td) => {
                let sid = match self.state.mod_syms.lookup(&expr.nid){
                    Some(s) => s,
                    None => unimplemented!("NodeId => SymId lookup failed"),
                };

                self.build_variable(sid, td.span)
            },
            ExprKind::Binary{op, lhs, rhs} => self.build_binary_expr(*op, &lhs, &rhs),
            ExprKind::Lit(flp) => self.build_literal(&flp),
            ExprKind::Block(block) => {
                let mut val = self.context().f64_type().const_float(0.0);
                for expr in block{
                    val = self.build_expr(expr);
                }
                val
            }
            ExprKind::Call{callee, args} =>{
                self.build_call(callee, args)
            }
            ExprKind::If{cond, if_body, else_body} => self.build_if_expr(cond, if_body, else_body),
            // TODO: not well defined behavior returning this statment
            ExprKind::Decl(vd) => {
                use std::f64;
                self.build_var_decl(&expr.nid, vd);
                self.context().f64_type().const_float(f64::NAN)
            },
            ExprKind::Let{bound, let_body} => self.build_let_expr(bound, let_body),
            
            // TODO: behaviro not well defined
            ExprKind::While{cond, while_body} => {
                self.build_while_stmt(cond, while_body);
                self.context().f64_type().const_float(f64::NAN)
            }
            // _ => unimplemented!()
        }
    }


    fn build_while_stmt(&mut self, cond: &ast::Expr<'c>, body: &ast::Expr<'c>){
        // create basic block
        let while_cond_block = self.context().append_basic_block(self.func, "while.cond");
        let while_body_block = self.context().append_basic_block(self.func, "while.body");
        let while_after_block = self.context().append_basic_block(self.func, "while.after");

        // jump from prev bb to the condition
        self.builder().build_unconditional_branch(while_cond_block);
        
        // build conditional LLVM
        self.builder().position_at_end(while_cond_block);
        let zero_const = self.context().f64_type().const_float(0.0);
        let built_cond = self.build_expr(cond);
        let truth_switch = self.builder().build_float_compare(
            FloatPredicate::ONE, built_cond, zero_const, "while.switch"
        );

        self.builder().build_conditional_branch(truth_switch, while_body_block, while_after_block);

        self.builder().position_at_end(while_body_block);
        self.build_expr(body);
        
        // loop back to top
        self.builder().build_unconditional_branch(while_cond_block);

        // next stmts append to this bb
        self.builder().position_at_end(while_after_block);
    }


    fn build_var_decl(&mut self, nid: &NodeId, vd: &ast::VarDecl<'c>){

        let sid = match self.state.mod_syms.lookup(nid){
            Some(s) => *s,
            None => unimplemented!("NodeId => SymId lookup failed"),
        };

        match vd.mutable{
            ast::MutKind::Mutable => {
                // must alloc on stack
                let ptr = self.create_entry_block_alloca(vd.bound.span);
                self.var_sym.insert(sid, BasicValueEnum::PointerValue(ptr));

                let val = self.build_expr(&vd.value);
                self.builder().build_store(ptr, val);
            }
            ast::MutKind::Const => {
                let val = self.build_expr(&vd.value);
                self.var_sym.insert(sid, BasicValueEnum::FloatValue(val));
            }
        }
    }

    fn build_let_expr(&mut self, bindings: &Vec<ast::Expr<'c>>, lb: &ast::Expr<'c>) -> FloatValue<'c>{

        // declare const variables
        for decl in bindings{
            self.build_expr(decl);
        }

        // define body
        self.build_expr(lb)
    }


    fn build_if_expr(&mut self, cond: &ast::Expr<'c>, if_body: &ast::Expr<'c>, else_body: &ast::Expr<'c>) -> FloatValue<'c>{
        let parent = self.func;
        let zero_const = self.context().f64_type().const_float(0.0);
        
        let built_cond = self.build_expr(cond);
        let truth_switch = self.builder().build_float_compare(
            FloatPredicate::ONE, built_cond, zero_const, "if.cond"
        );
        
        let then_block = self.context().append_basic_block(parent, "if.then");
        let else_block = self.context().append_basic_block(parent, "if.else");
        let aif_block = self.context().append_basic_block(parent, "if.after");

        // jump to branch
        self.builder().build_conditional_branch(truth_switch, then_block, else_block);

        // then branch
        self.builder().position_at_end(then_block);
        let then_val = self.build_expr(if_body);
        self.builder().build_unconditional_branch(aif_block);

        let then_bb = self.builder().get_insert_block().unwrap();

        // else branch
        self.builder().position_at_end(else_block);
        let else_val = self.build_expr(else_body);
        self.builder().build_unconditional_branch(aif_block);

        let else_bb = self.builder().get_insert_block().unwrap();

        // after if, merge value
        self.builder().position_at_end(aif_block);
        let phi_node = self.builder().build_phi(self.context().f64_type(), "if_phi");
        phi_node.add_incoming(&[
            (&then_val, then_bb), 
            (&else_val, else_bb)
        ]);

        phi_node.as_basic_value().into_float_value()
    }

    fn build_assign_to_var(&mut self, sid: &SymId, name: &'c str, rhs_val: FloatValue<'c>) -> FloatValue<'c>{
        match self.var_sym.get(sid){
            Some(val) => match val{
                BasicValueEnum::PointerValue(ptr) => {
                    let _stored_instr = self.builder().build_store(*ptr, rhs_val);
                    rhs_val
                }
                _ => unimplemented!("Local Variable of invalid type",)
            },
            None => unimplemented!("Reference unknown variable {:?} in Assign", name),
        }
    }

    fn build_binary_expr(&mut self, op: BinaryOp, lhs: &ast::Expr<'c>, rhs: &ast::Expr<'c>) -> FloatValue<'c>{
        
        match op {
            BinaryOp::Assign => {
                let rhs_val = self.build_expr(rhs);
                match &*lhs.kind{
                    ast::ExprKind::Var(td) => {
                        let sid = match self.state.mod_syms.lookup(&lhs.nid){
                            Some(s) => s,
                            None => unimplemented!("NodeId => SymId lookup failed"),
                        };
                        return self.build_assign_to_var(sid, td.span, rhs_val);
                    }
                    _ => unimplemented!("Cannot assign to non-variable")
                }
            }
            _ => (),
        }
        
        let lhs_val = self.build_expr(lhs);
        let rhs_val = self.build_expr(rhs);

        match op{
            BinaryOp::Add => {
                self.builder().build_float_add(lhs_val, rhs_val, "tmpdadd")
            }
            BinaryOp::Sub => {
                self.builder().build_float_sub(lhs_val, rhs_val, "tmpdsub")
            }
            BinaryOp::Mul => {
                self.builder().build_float_mul(lhs_val, rhs_val, "tmpdmul")
            }
            BinaryOp::Div => {
                self.builder().build_float_div(lhs_val, rhs_val, "tmpddiv")
            }
            BinaryOp::Lt => {
                let bool_val = self.builder().build_float_compare(
                    FloatPredicate::OLT, 
                    lhs_val, rhs_val, 
                    "tmpflt"
                );
                // using unsigned because
                // signed 1-bit int only represents -1, 0
                // but the program semantic require 0, 1
                self.builder().build_unsigned_int_to_float(bool_val, self.context().f64_type(), "tmpbool")
            }
            BinaryOp::LtEt => {
                let bool_val = self.builder().build_float_compare(
                    FloatPredicate::OLE, 
                    lhs_val, rhs_val, 
                    "tmpfltet"
                );
                self.builder().build_unsigned_int_to_float(bool_val, self.context().f64_type(), "tmpbool")
                // self.builder().build_signed_int_to_float(bool_val, self.context().f64_type(), "tmpbool")
            }
            BinaryOp::Et => {
                let bool_val = self.builder().build_float_compare(
                    FloatPredicate::OEQ, 
                    lhs_val, rhs_val, 
                    "tmpfet"
                );
                self.builder().build_unsigned_int_to_float(bool_val, self.context().f64_type(), "tmpbool")
            }

            BinaryOp::Assign => unreachable!(),
            _ => unimplemented!()
        }
    }

    fn build_call(&mut self, callee: &ast::Expr<'c>, args: &Vec<ast::Expr<'c>>) -> FloatValue<'c>{
        let named_callee = match &*callee.kind{
            ExprKind::Var(td) => td.span,
            _ => unimplemented!("Cannot call function from general expression. must use a name"),
        };

        match self.module().get_function(named_callee){
            Some(func) =>{
                let compiled_args: Vec<_> = args.iter().map(|e| self.build_expr(e).into()).collect();
                let call_value = self.builder().build_call(func, compiled_args.as_slice(), "tmpcall");
                
                match call_value.try_as_basic_value().left(){
                    Some(value) => value.into_float_value(),
                    None => unimplemented!("Invalid function call in LLVM"),
                }
            }
            None => {
                unimplemented!("Unknown function call")
            }
        }
    }

    fn build_variable(&mut self, sid: &SymId, name: &'c str) -> FloatValue<'c>{
        match self.var_sym.get(sid){
            Some(val) => match val{
                BasicValueEnum::PointerValue(ptr) => {
                    self.builder().build_load(*ptr, name).into_float_value()
                }
                BasicValueEnum::FloatValue(float) => {
                    *float
                }
                _ => unimplemented!("Local Variable of invalid type",)
            },
            None => unimplemented!("Reference unknown variable {:?}", name),
        }
    }

    fn build_literal(&mut self, lit: &f64) -> FloatValue<'c>{
        self.context().f64_type().const_float(lit.clone())
    }
}

