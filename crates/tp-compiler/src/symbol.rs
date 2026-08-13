use std::collections::HashMap;

use crate::{Span, Type};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolId(u32);

impl SymbolId {
    pub const fn raw(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct Binding {
    pub id: SymbolId,
    pub ty: Type,
    pub mutable: bool,
    pub span: Span,
}

#[derive(Debug, Default)]
pub struct Scopes {
    next_id: u32,
    scopes: Vec<HashMap<String, Binding>>,
}

impl Scopes {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            scopes: vec![HashMap::new()],
        }
    }

    pub fn push(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    pub fn insert(
        &mut self,
        name: String,
        ty: Type,
        mutable: bool,
        span: Span,
    ) -> Result<SymbolId, Binding> {
        let scope = self.scopes.last_mut().expect("scope stack is never empty");
        if let Some(existing) = scope.get(&name) {
            return Err(existing.clone());
        }
        let id = SymbolId(self.next_id);
        self.next_id += 1;
        scope.insert(
            name,
            Binding {
                id,
                ty,
                mutable,
                span,
            },
        );
        Ok(id)
    }

    pub fn get(&self, name: &str) -> Option<&Binding> {
        self.scopes.iter().rev().find_map(|scope| scope.get(name))
    }
}
