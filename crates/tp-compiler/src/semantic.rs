use std::collections::HashMap;

use crate::{HirGlobalKind, Span};

#[derive(Debug, Clone, Default)]
pub struct SemanticModel {
    globals: HashMap<Span, HirGlobalKind>,
}

impl SemanticModel {
    pub(crate) fn record_global(&mut self, span: Span, kind: HirGlobalKind) {
        self.globals.insert(span, kind);
    }

    pub(crate) fn global(&self, span: Span) -> Option<HirGlobalKind> {
        self.globals.get(&span).copied()
    }
}
