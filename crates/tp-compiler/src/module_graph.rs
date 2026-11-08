use std::path::{Path, PathBuf};

use crate::{SourceId, Span};

/// Canonical filesystem identity. This remains stable across independent
/// sessions when canonical paths refer to the same file.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ModuleId(PathBuf);

impl ModuleId {
    pub fn path(&self) -> &Path {
        &self.0
    }

    pub(crate) fn from_canonical_path(path: PathBuf) -> Self {
        Self(path)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleNode {
    pub id: ModuleId,
    pub namespace: String,
    /// Session-scoped source ID belonging to this canonical module.
    pub source: SourceId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleImport {
    pub from: ModuleId,
    pub to: ModuleId,
    pub alias: String,
    pub span: Span,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModuleGraph {
    nodes: Vec<ModuleNode>,
    edges: Vec<ModuleImport>,
}

impl ModuleGraph {
    pub fn nodes(&self) -> &[ModuleNode] {
        &self.nodes
    }

    pub fn edges(&self) -> &[ModuleImport] {
        &self.edges
    }

    pub(crate) fn add_node(&mut self, node: ModuleNode) {
        if !self.nodes.iter().any(|existing| existing.id == node.id) {
            self.nodes.push(node);
        }
    }

    pub(crate) fn add_edge(&mut self, edge: ModuleImport) {
        self.edges.push(edge);
    }

    pub(crate) fn finish(&mut self) {
        self.nodes.sort_by(|left, right| left.id.cmp(&right.id));
        self.edges.sort_by(|left, right| {
            (
                &left.from,
                &left.to,
                left.alias.as_str(),
                left.span.start,
            )
                .cmp(&(
                    &right.from,
                    &right.to,
                    right.alias.as_str(),
                    right.span.start,
                ))
        });
    }
}
