use crate::ast::ops::BinaryOp;
use crate::ast::token::TokenData;
use crate::ast::*;

// TODO: create const ref printer struct for public use
// internally create mutable state
#[derive(Clone)]
pub struct AstTermPrinter<'s> {
    tab: &'s str,
}

struct PrinterState<'s> {
    depth: u32,
    tab: &'s str,
}

impl<'s> AstTermPrinter<'s> {
    pub fn new(t: &'s str) -> Self {
        Self { tab: t }
    }
    pub fn default() -> Self {
        Self::new("    ")
    }

    pub fn print_module(&self, m: &Module<'s>) {
        let mut printer = PrinterState::new(self.tab);
        printer.print_module(m)
    }
    pub fn print_item(&self, item: &Item<'s>) {
        let mut printer = PrinterState::new(self.tab);
        printer.print_item(item)
    }
}

impl<'s> PrinterState<'s> {
    pub fn new(t: &'s str) -> Self {
        Self { depth: 0, tab: t }
    }

    // fn print_place(&mut self, p: &Place<'s>){
    //     eprint!("{}", p.path.first().unwrap().span);
    //     for idx in 1..p.path.len(){
    //         eprint!(".");
    //         eprint!("{}", p.path[idx].span);
    //     }
    // }

    pub fn print_module(&mut self, m: &Module<'s>) {
        eprintln!("Module {}", m.decl.span);
        // self.print_place(&m.decl.name);

        self.dive();
        for item in &m.body {
            self.print_depth();
            self.print_item(item);
        }
        self.rise();
    }
    pub fn print_item(&mut self, item: &Item<'s>) {
        match &item.kind {
            ItemKind::Func(f) => self.print_function(f),
            ItemKind::Extern(e) => self.print_extern(e),
            // ItemKind::Import(im) => self.print_imports(im),
        }
    }

    fn dive(&mut self) {
        self.depth += 1;
    }
    fn rise(&mut self) {
        self.depth -= 1;
    }
    fn print_depth(&self) {
        for _ in 0..self.depth {
            eprint!("{}", self.tab)
        }
    }

    // fn print_imports(&mut self, im: &ImportStmt<'s>){
    //     eprint!("Import");
    //     self.dive();
    //     match im{
    //         ImportStmt::SimpleImport(p) =>{
    //             eprint!("\n");
    //             self.print_place(p);
    //         }
    //         ImportStmt::FromImport(p, take) =>{
    //             self.print_place(p);
    //             eprint!("(");
    //             self.dive();
    //             for t in take{
    //                 eprintln!("{},",t.span);
    //             }
    //             self.rise();
    //         }
    //     }
    //     self.rise();
    // }

    fn print_func_proto(&mut self, fp: &FuncProto<'s>) {
        let tok = fp.name;
        let loc = tok.loc;
        eprintln!(
            "Function {:?} @ line {}, col {}",
            tok.span, loc.line, loc.column
        );
    }

    fn print_extern(&mut self, e: &FuncProto<'s>) {
        eprint!("Extern ");
        self.print_func_proto(e);
    }

    fn print_function(&mut self, f: &Function<'s>) {
        // eprint!("- ");
        self.print_func_proto(&f.proto);

        // self.print_depth();
        // eprintln!("func body:");
        // self.dive();
        self.print_depth();
        self.print_expr(&f.body);
        // self.rise();
    }

    fn print_expr(&mut self, expr: &Expr<'s>) {
        match &*expr.kind {
            // ExprKind::Place(p) => self.print_place(p),
            ExprKind::Block(block) => self.print_expr_block(block),
            ExprKind::Call { callee, args } => self.print_call(callee, args),
            ExprKind::Binary { op, lhs, rhs } => self.print_binary_expr(op, lhs, rhs),
            ExprKind::Lit(f) => self.print_literal(f),
            ExprKind::Var(v) => self.print_var(v),
            ExprKind::If {
                cond,
                if_body,
                else_body,
            } => self.print_if(cond, if_body, else_body),
            ExprKind::Decl(vd) => self.print_assignment(vd),
            ExprKind::Let { bound, let_body } => self.print_let(bound, let_body),
            ExprKind::While { cond, while_body } => self.print_while(cond, while_body),
        }
    }

    fn print_while(&mut self, cond: &Expr<'s>, body: &Expr<'s>) {
        eprintln!("While:");
        self.dive();
        self.print_depth();
        self.print_expr(cond);
        self.rise();
        self.print_depth();
        eprintln!("Do:");
        self.dive();
        self.print_depth();
        self.print_expr(body);
        self.rise();
    }

    fn print_assignment(&mut self, decl: &VarDecl<'s>) {
        eprintln!("Bind {:?} with:", decl.bound.span);
        self.dive();
        self.print_depth();
        self.print_expr(&decl.value);
        self.rise();
    }

    fn print_let(&mut self, b: &[Expr<'s>], lb: &Expr<'s>) {
        eprintln!("Let:");
        self.dive();
        for binding in b {
            self.print_depth();
            self.print_expr(binding);
        }

        self.rise();
        self.print_depth();
        eprintln!("In:");
        self.dive();
        self.print_depth();
        self.print_expr(lb);
        self.rise();
    }

    fn print_if(&mut self, cond: &Expr<'s>, body: &Expr<'s>, else_body: &Expr<'s>) {
        eprintln!("If: ");

        self.print_depth();
        eprintln!("cond: ");
        self.dive();
        self.print_depth();
        self.print_expr(cond);
        self.rise();

        self.print_depth();
        eprintln!("body: ");
        self.dive();
        self.print_depth();
        self.print_expr(body);
        self.rise();

        self.print_depth();
        eprintln!("else: ");
        self.dive();
        self.print_depth();
        self.print_expr(else_body);
        self.rise();

        // self.rise();
    }

    fn print_expr_block(&mut self, b: &[Expr<'s>]) {
        eprintln!("Block with:");
        self.dive();
        for expr in b {
            self.print_depth();
            eprint!("- ");
            self.print_expr(expr);
        }
        self.rise();
    }

    // TODO: print args
    fn print_call(&mut self, ce: &Expr<'s>, args: &[Expr<'s>]) {
        eprint!("Call of ");
        self.print_expr(ce);

        self.print_depth();
        eprintln!("args:");
        self.dive();
        for arg in args {
            self.print_depth();
            self.print_expr(arg);
        }
        self.rise();
    }

    fn print_binary_expr(&mut self, op: &BinaryOp, lhs: &Expr<'s>, rhs: &Expr<'s>) {
        eprintln!("Binary {:?}", op);
        self.dive();
        self.print_depth();
        self.print_expr(lhs);
        self.print_depth();
        self.print_expr(rhs);
        self.rise();
    }

    fn print_literal(&mut self, lit: &f64) {
        eprintln!("Literal {}", lit);
    }

    fn print_var(&mut self, td: &TokenData<'s>) {
        eprintln!("Var {:?}", td.span)
    }
}
