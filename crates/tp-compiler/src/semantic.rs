use std::collections::HashMap;

use crate::{HirGlobalKind, Span, SymbolId, Type};

#[derive(Debug, Clone)]
struct SemanticFunction {
    params: Vec<Type>,
    result: Type,
}

#[derive(Debug, Clone)]
struct SemanticBinding {
    symbol: SymbolId,
    ty: Type,
}

#[derive(Debug, Clone, Default)]
pub struct SemanticModel {
    globals: HashMap<Span, HirGlobalKind>,
    expr_types: HashMap<Span, Type>,
    block_types: HashMap<Span, Type>,
    bindings: HashMap<Span, SemanticBinding>,
    local_uses: HashMap<Span, SymbolId>,
    functions: HashMap<String, SemanticFunction>,
}

impl SemanticModel {
    pub(crate) fn record_global(&mut self, span: Span, kind: HirGlobalKind) {
        self.globals.insert(span, kind);
    }

    pub(crate) fn global(&self, span: Span) -> Option<HirGlobalKind> {
        self.globals.get(&span).copied()
    }

    pub(crate) fn record_expr_type(&mut self, span: Span, ty: Type) {
        self.expr_types.insert(span, ty);
    }

    pub fn expr_type(&self, span: Span) -> Option<&Type> {
        self.expr_types.get(&span)
    }

    pub(crate) fn record_block_type(&mut self, span: Span, ty: Type) {
        self.block_types.insert(span, ty);
    }

    pub(crate) fn block_type(&self, span: Span) -> Option<&Type> {
        self.block_types.get(&span)
    }

    pub(crate) fn record_binding(&mut self, span: Span, symbol: SymbolId, ty: Type) {
        self.bindings.insert(span, SemanticBinding { symbol, ty });
    }

    pub fn binding(&self, span: Span) -> Option<(SymbolId, &Type)> {
        self.bindings
            .get(&span)
            .map(|binding| (binding.symbol, &binding.ty))
    }

    pub(crate) fn record_local_use(&mut self, span: Span, symbol: SymbolId) {
        self.local_uses.insert(span, symbol);
    }

    pub(crate) fn local_use(&self, span: Span) -> Option<SymbolId> {
        self.local_uses.get(&span).copied()
    }

    pub(crate) fn record_function(&mut self, name: String, params: Vec<Type>, result: Type) {
        self.functions
            .insert(name, SemanticFunction { params, result });
    }

    pub(crate) fn function(&self, name: &str) -> Option<(&[Type], &Type)> {
        self.functions
            .get(name)
            .map(|function| (function.params.as_slice(), &function.result))
    }
}
