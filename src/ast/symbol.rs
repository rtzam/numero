use std::collections::HashMap;

// use std::num::NonZeroU32;
// use std::num::NonZeroUsize;

use crate::ast::node::NodeId;

type SymIdRepr = u32;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymId(SymIdRepr);

impl SymId {
    fn new() -> Self {
        // Self(SymIdRepr::new(1).unwrap())
        Self(0)
    }
    // fn inc(&self) -> Self{
    //     // let new_id = self.0.get() + 1;
    //     // let non_z_id = SymIdRepr::new(new_id).unwrap();
    //     // Self(non_z_id)
    //     Self(self.0 + 1)
    // }
    fn shift(&mut self) -> Self {
        self.0 += 1;
        *self
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ScopedSymEntry {
    nid: NodeId,
    pub sid: SymId,
    sscope: SymScope,
}

#[derive(Debug, Clone)]
pub enum ScopedInsertErr {
    RedefinedLocal(ScopedSymEntry), // TODO: support shadowing
    CompilerBug(String),
}

pub type ScopedInsertResult = Result<SymId, ScopedInsertErr>;

struct ScopedSymTable<'s> {
    symbols: HashMap<&'s str, ScopedSymEntry>,
}

impl<'s> Default for ScopedSymTable<'s> {
    fn default() -> Self {
        Self {
            symbols: HashMap::new(),
        }
    }
}

impl<'s> ScopedSymTable<'s> {
    fn lookup(&self, s: &'s str) -> Option<&SymId> {
        let entry = self.symbols.get(s)?;
        Some(&entry.sid)
    }

    fn insert(
        &mut self,
        nid: NodeId,
        s: &'s str,
        sid: SymId,
        kind: SymScope,
    ) -> ScopedInsertResult {
        match self.symbols.get(s) {
            Some(entry) => Err(ScopedInsertErr::RedefinedLocal(*entry)),
            None => {
                let entry = ScopedSymEntry {
                    nid,
                    sid,
                    sscope: kind,
                };
                self.symbols.insert(s, entry);
                Ok(sid)
            }
        }
    }
}

pub struct ScopedSymbolStack<'s> {
    sid: SymId,
    mod_table: ModSymTable,
    scope_stack: Vec<ScopedSymTable<'s>>,
}

impl<'s> Default for ScopedSymbolStack<'s> {
    fn default() -> Self {
        Self {
            sid: SymId::new(),
            mod_table: ModSymTable::new(),
            scope_stack: Vec::new(),
        }
    }
}

impl<'s> ScopedSymbolStack<'s> {
    // pub fn with_root() -> Self{
    //     let mut sss = Self::new();
    //     sss.push_scope();
    //     sss
    // }
    pub fn push_scope(&mut self) {
        self.scope_stack.push(ScopedSymTable::default());
    }

    pub fn pop_scope(&mut self) {
        // unpack symbols into global table
        if let Some(scope) = self.scope_stack.pop() {
            for (_, entry) in scope.symbols.into_iter() {
                if let Some(old_sid) = self.mod_table.insert(entry.nid, entry.sid) {
                    unimplemented!(
                        "remap NodeID {:?} from {:?} to second {:?}",
                        entry.nid,
                        old_sid,
                        entry.sid
                    )
                }
            }
        }
    }

    pub fn finish_resolve(self) -> ModSymTable {
        self.mod_table
    }

    pub fn lookup(&self, sym: &'s str) -> Option<&SymId> {
        for scope in self.scope_stack.iter().rev() {
            match scope.lookup(sym) {
                Some(sid) => {
                    // eprintln!("Resolved {:?} to {:?}", sym, sid);
                    return Some(sid);
                }
                _ => continue,
            }
        }
        None
    }

    pub fn insert_local(&mut self, nid: NodeId, sym: &'s str) -> ScopedInsertResult {
        self.insert(nid, sym, SymScope::Local)
    }

    // create nid => sid mapping
    pub fn insert_local_reuse(&mut self, nid: NodeId, sid: SymId) -> Option<ScopedInsertErr> {
        let _old_entry = self.mod_table.insert(nid, sid)?;

        // Error, a variable use is being mapped to multiple
        unimplemented!("AST node being mapped to multiple SID")
    }

    pub fn insert_func(&mut self, nid: NodeId, sym: &'s str) -> ScopedInsertResult {
        self.insert(nid, sym, SymScope::Module)
    }

    fn insert(&mut self, nid: NodeId, sym: &'s str, kind: SymScope) -> ScopedInsertResult {
        let sid = self.sid.shift();

        // eprintln!("Creating {:?} from {:?}", sid, sym);

        match self.scope_stack.last_mut() {
            Some(scope) => scope.insert(nid, sym, sid, kind),
            None => {
                let msg = format!("No root scope when inserting scoped symbol");
                Err(ScopedInsertErr::CompilerBug(msg))
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SymScope {
    Module, // function, extern, global var
    Local,  // inside function
}

#[derive(Debug, Clone, Copy)]
pub struct ModSymEntry {
    pub scope: SymScope,
}

pub struct ModSymTable {
    symbols: HashMap<NodeId, SymId>,
}

impl<'s> ModSymTable {
    pub fn lookup(&self, nid: &NodeId) -> Option<&SymId> {
        self.symbols.get(nid) //.expect("Bug: Global symbol table lookup failed")
    }

    fn new() -> Self {
        Self {
            symbols: HashMap::new(),
        }
    }

    fn insert(&mut self, nid: NodeId, entry: SymId) -> Option<SymId> {
        self.symbols.insert(nid, entry)
    }
}
