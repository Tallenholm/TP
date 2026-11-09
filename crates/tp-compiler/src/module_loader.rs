use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use crate::{
    Block, Diagnostic, Expr, ExprKind, FnDecl, ImportDecl, Item, Lexer, MatchArm, Module,
    ModuleGraph, ModuleId, ModuleImport, ModuleNode, Param, Parser, Pattern, PatternKind,
    SourceFile, SourceId, SourceManager, Stmt, StructDecl, TypeRef, VariantDecl,
};

pub struct ModuleLoader;

#[derive(Debug)]
pub struct ModuleLoadReport {
    pub module: Option<Module>,
    pub diagnostics: Vec<Diagnostic>,
    pub sources: SourceManager,
    pub graph: ModuleGraph,
}

impl ModuleLoader {
    pub fn load(path: &Path) -> Result<Module, Vec<Diagnostic>> {
        let report = Self::load_with_sources(path);
        if report.diagnostics.is_empty() {
            Ok(report.module.unwrap_or(Module { items: Vec::new() }))
        } else {
            Err(report.diagnostics)
        }
    }

    pub fn load_with_sources(path: &Path) -> ModuleLoadReport {
        let mut stack = Vec::new();
        let mut loaded = HashMap::new();
        let mut sources = SourceManager::default();
        let mut graph = ModuleGraph::default();
        let result = load_recursive(path, "", &mut stack, &mut loaded, &mut sources, &mut graph);
        graph.finish();
        match result {
            Ok(loaded_module) => ModuleLoadReport {
                module: Some(Module {
                    items: loaded_module.items,
                }),
                diagnostics: Vec::new(),
                sources,
                graph,
            },
            Err(diagnostics) => ModuleLoadReport {
                module: None,
                diagnostics,
                sources,
                graph,
            },
        }
    }
}

#[derive(Debug)]
struct LoadedModule {
    namespace: String,
    items: Vec<Item>,
}

fn load_recursive(
    path: &Path,
    requested_namespace: &str,
    stack: &mut Vec<PathBuf>,
    loaded: &mut HashMap<PathBuf, String>,
    sources: &mut SourceManager,
    graph: &mut ModuleGraph,
) -> Result<LoadedModule, Vec<Diagnostic>> {
    let canonical = canonicalize_for_load(path)?;
    if let Some(index) = stack.iter().position(|entry| entry == &canonical) {
        let mut chain: Vec<String> = stack[index..]
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        chain.push(canonical.display().to_string());
        return Err(vec![Diagnostic::error(
            "TP-E0200",
            format!("import cycle detected: {}", chain.join(" -> ")),
        )]);
    }

    if let Some(namespace) = loaded.get(&canonical) {
        return Ok(LoadedModule {
            namespace: namespace.clone(),
            items: Vec::new(),
        });
    }

    let namespace = requested_namespace.to_owned();
    loaded.insert(canonical.clone(), namespace.clone());

    let source = fs::read_to_string(&canonical).map_err(|error| {
        vec![Diagnostic::error(
            "TP-E0200",
            format!("unable to read module `{}`: {error}", canonical.display()),
        )]
    })?;
    let (module, source_id) = parse_file(&canonical, &source, sources)?;
    graph.add_node(ModuleNode {
        id: ModuleId::from_canonical_path(canonical.clone()),
        namespace: namespace.clone(),
        source: source_id,
    });

    stack.push(canonical.clone());
    let imports: Vec<ImportDecl> = module
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Import(import) => Some(import.clone()),
            _ => None,
        })
        .collect();

    let mut alias_bindings: HashMap<String, (String, crate::Span)> = HashMap::new();
    let mut import_namespaces = HashMap::new();
    let mut imported_items = Vec::new();
    for import in &imports {
        let alias = import.alias.as_deref().unwrap_or(&import.module);

        if let Some((previous_module, previous_span)) = alias_bindings.get(alias) {
            if previous_module == &import.module {
                continue;
            }

            stack.pop();
            return Err(vec![
                Diagnostic::error("TP-E0200", format!("duplicate import alias `{alias}`"))
                    .with_primary(import.span)
                    .with_label(*previous_span, "alias was first bound here"),
            ]);
        }
        alias_bindings.insert(alias.to_owned(), (import.module.clone(), import.span));

        let child_requested_namespace = qualify(&namespace, alias);
        let child_path = canonical
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(format!("{}.tp", import.module));
        let child_canonical = match canonicalize_for_load(&child_path) {
            Ok(path) => path,
            Err(diagnostics) => {
                stack.pop();
                return Err(diagnostics);
            }
        };
        match load_recursive(
            &child_canonical,
            &child_requested_namespace,
            stack,
            loaded,
            sources,
            graph,
        ) {
            Ok(mut child) => {
                graph.add_edge(ModuleImport {
                    from: ModuleId::from_canonical_path(canonical.clone()),
                    to: ModuleId::from_canonical_path(child_canonical),
                    alias: alias.to_owned(),
                    span: import.span,
                });
                import_namespaces.insert(alias.to_owned(), child.namespace.clone());
                imported_items.append(&mut child.items);
            }
            Err(diagnostics) => {
                stack.pop();
                return Err(diagnostics);
            }
        }
    }

    let current_items: Vec<Item> = module
        .items
        .into_iter()
        .filter(|item| !matches!(item, Item::Import(_)))
        .collect();
    let mut rewriter = NamespaceRewriter::new(&namespace, &current_items, import_namespaces);
    let mut rewritten = rewriter.rewrite_items(current_items);
    rewritten.append(&mut imported_items);
    stack.pop();

    Ok(LoadedModule {
        namespace,
        items: rewritten,
    })
}

fn canonicalize_for_load(path: &Path) -> Result<PathBuf, Vec<Diagnostic>> {
    path.canonicalize().map_err(|error| {
        vec![Diagnostic::error(
            "TP-E0200",
            format!("unable to resolve module `{}`: {error}", path.display()),
        )]
    })
}

fn parse_file(
    path: &Path,
    source: &str,
    sources: &mut SourceManager,
) -> Result<(Module, SourceId), Vec<Diagnostic>> {
    let source_file = SourceFile::new(path.display().to_string(), source);
    let source_id = sources.insert(source_file.clone());
    let lexed = Lexer::new(&source_file).lex();
    let mut diagnostics = lexed.diagnostics;
    let parsed = Parser::new(&source_file, lexed.tokens).parse_module();
    diagnostics.extend(parsed.diagnostics);
    if diagnostics.is_empty() {
        Ok((
            parsed.module.unwrap_or(Module { items: Vec::new() }),
            source_id,
        ))
    } else {
        Err(diagnostics)
    }
}

fn qualify(namespace: &str, name: &str) -> String {
    if namespace.is_empty() {
        name.to_owned()
    } else {
        format!("{namespace}.{name}")
    }
}

struct NamespaceRewriter {
    namespace: String,
    functions: HashSet<String>,
    structs: HashSet<String>,
    enums: HashSet<String>,
    variants: HashSet<String>,
    imports: HashMap<String, String>,
    scopes: Vec<HashSet<String>>,
}

impl NamespaceRewriter {
    fn new(namespace: &str, items: &[Item], imports: HashMap<String, String>) -> Self {
        let functions = items
            .iter()
            .filter_map(|item| match item {
                Item::Function(decl) => Some(decl.name.clone()),
                _ => None,
            })
            .collect();
        let structs = items
            .iter()
            .filter_map(|item| match item {
                Item::Struct(decl) => Some(decl.name.clone()),
                _ => None,
            })
            .collect();
        let enums = items
            .iter()
            .filter_map(|item| match item {
                Item::Enum(decl) => Some(decl.name.clone()),
                _ => None,
            })
            .collect();
        let variants = items
            .iter()
            .flat_map(|item| match item {
                Item::Enum(decl) => decl
                    .variants
                    .iter()
                    .map(|variant| variant.name.clone())
                    .collect::<Vec<_>>(),
                _ => Vec::new(),
            })
            .collect();
        Self {
            namespace: namespace.to_owned(),
            functions,
            structs,
            enums,
            variants,
            imports,
            scopes: Vec::new(),
        }
    }

    fn rewrite_items(&mut self, items: Vec<Item>) -> Vec<Item> {
        items
            .into_iter()
            .map(|item| match item {
                Item::Function(function) => Item::Function(self.rewrite_function(function)),
                Item::Struct(decl) => Item::Struct(self.rewrite_struct(decl)),
                Item::Enum(decl) => Item::Enum(self.rewrite_enum(decl)),
                Item::Import(import) => Item::Import(import),
            })
            .collect()
    }

    fn rewrite_function(&mut self, mut function: FnDecl) -> FnDecl {
        function.name = self.declaration_name(&function.name);
        for param in &mut function.params {
            param.ty = self.rewrite_type(param.ty.clone());
        }
        function.return_type = function.return_type.take().map(|ty| self.rewrite_type(ty));

        self.push_scope();
        for Param { name, .. } in &function.params {
            self.bind(name);
        }
        function.body = self.rewrite_block(function.body);
        self.pop_scope();
        function
    }

    fn rewrite_struct(&mut self, mut decl: StructDecl) -> StructDecl {
        decl.name = self.declaration_name(&decl.name);
        for field in &mut decl.fields {
            field.ty = self.rewrite_type(field.ty.clone());
        }
        decl
    }

    fn rewrite_enum(&mut self, mut decl: crate::EnumDecl) -> crate::EnumDecl {
        decl.name = self.declaration_name(&decl.name);
        for VariantDecl { name, payload, .. } in &mut decl.variants {
            *name = self.declaration_name(name);
            for ty in payload {
                *ty = self.rewrite_type(ty.clone());
            }
        }
        decl
    }

    fn rewrite_type(&self, mut ty: TypeRef) -> TypeRef {
        ty.args = ty
            .args
            .into_iter()
            .map(|arg| self.rewrite_type(arg))
            .collect();
        if self.structs.contains(&ty.name) || self.enums.contains(&ty.name) {
            ty.name = self.declaration_name(&ty.name);
        } else if let Some(imported) = self.rewrite_imported_name(&ty.name) {
            ty.name = imported;
        }
        ty
    }

    fn rewrite_block(&mut self, mut block: Block) -> Block {
        self.push_scope();
        let mut rewritten = Vec::with_capacity(block.statements.len());
        for statement in block.statements {
            let statement = match statement {
                Stmt::Let {
                    mutable,
                    name,
                    type_annotation,
                    value,
                    span,
                } => {
                    let value = self.rewrite_expr(value);
                    let type_annotation = type_annotation.map(|ty| self.rewrite_type(ty));
                    self.bind(&name);
                    Stmt::Let {
                        mutable,
                        name,
                        type_annotation,
                        value,
                        span,
                    }
                }
                Stmt::Return { value, span } => Stmt::Return {
                    value: value.map(|expr| self.rewrite_expr(expr)),
                    span,
                },
                Stmt::While {
                    condition,
                    body,
                    span,
                } => Stmt::While {
                    condition: self.rewrite_expr(condition),
                    body: self.rewrite_block(body),
                    span,
                },
                Stmt::Expr { expr, terminated } => Stmt::Expr {
                    expr: self.rewrite_expr(expr),
                    terminated,
                },
            };
            rewritten.push(statement);
        }
        block.statements = rewritten;
        self.pop_scope();
        block
    }

    fn rewrite_expr(&mut self, mut expr: Expr) -> Expr {
        let kind = match expr.kind {
            ExprKind::Integer(value) => ExprKind::Integer(value),
            ExprKind::Float(value) => ExprKind::Float(value),
            ExprKind::String(value) => ExprKind::String(value),
            ExprKind::Bool(value) => ExprKind::Bool(value),
            ExprKind::Name(name) => {
                if !self.is_local(&name)
                    && (self.functions.contains(&name) || self.variants.contains(&name))
                {
                    ExprKind::Name(self.declaration_name(&name))
                } else {
                    ExprKind::Name(name)
                }
            }
            ExprKind::Unary { op, expr } => ExprKind::Unary {
                op,
                expr: Box::new(self.rewrite_expr(*expr)),
            },
            ExprKind::Binary { op, left, right } => ExprKind::Binary {
                op,
                left: Box::new(self.rewrite_expr(*left)),
                right: Box::new(self.rewrite_expr(*right)),
            },
            ExprKind::Assign { target, value } => ExprKind::Assign {
                target,
                value: Box::new(self.rewrite_expr(*value)),
            },
            ExprKind::Call { callee, args } => ExprKind::Call {
                callee: Box::new(self.rewrite_expr(*callee)),
                args: args.into_iter().map(|arg| self.rewrite_expr(arg)).collect(),
            },
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => ExprKind::If {
                condition: Box::new(self.rewrite_expr(*condition)),
                then_branch: self.rewrite_block(then_branch),
                else_branch: else_branch.map(|block| self.rewrite_block(block)),
            },
            ExprKind::StructLiteral { type_name, fields } => ExprKind::StructLiteral {
                type_name: if self.structs.contains(&type_name) {
                    self.declaration_name(&type_name)
                } else {
                    self.rewrite_imported_name(&type_name).unwrap_or(type_name)
                },
                fields: fields
                    .into_iter()
                    .map(|(name, value)| (name, self.rewrite_expr(value)))
                    .collect(),
            },
            ExprKind::Field { base, field } => {
                if let ExprKind::Name(alias) = &base.kind {
                    if !self.is_local(alias) {
                        if let Some(namespace) = self.imports.get(alias) {
                            return Expr {
                                kind: ExprKind::Name(format!("{namespace}.{field}")),
                                span: expr.span,
                            };
                        }
                    }
                }
                ExprKind::Field {
                    base: Box::new(self.rewrite_expr(*base)),
                    field,
                }
            }
            ExprKind::Match { value, arms } => ExprKind::Match {
                value: Box::new(self.rewrite_expr(*value)),
                arms: arms
                    .into_iter()
                    .map(|arm| self.rewrite_match_arm(arm))
                    .collect(),
            },
        };
        expr.kind = kind;
        expr
    }

    fn rewrite_match_arm(&mut self, mut arm: MatchArm) -> MatchArm {
        self.push_scope();
        arm.pattern = self.rewrite_pattern(arm.pattern);
        arm.body = self.rewrite_expr(arm.body);
        self.pop_scope();
        arm
    }

    fn rewrite_pattern(&mut self, mut pattern: Pattern) -> Pattern {
        let kind = match pattern.kind {
            PatternKind::Wildcard => PatternKind::Wildcard,
            PatternKind::Name(name) => {
                if self.variants.contains(&name) {
                    PatternKind::Name(self.declaration_name(&name))
                } else if let Some(imported) = self.rewrite_imported_name(&name) {
                    PatternKind::Name(imported)
                } else {
                    self.bind(&name);
                    PatternKind::Name(name)
                }
            }
            PatternKind::Integer(value) => PatternKind::Integer(value),
            PatternKind::Bool(value) => PatternKind::Bool(value),
            PatternKind::String(value) => PatternKind::String(value),
            PatternKind::Variant { name, args } => PatternKind::Variant {
                name: if self.variants.contains(&name) {
                    self.declaration_name(&name)
                } else {
                    self.rewrite_imported_name(&name).unwrap_or(name)
                },
                args: args
                    .into_iter()
                    .map(|arg| self.rewrite_pattern(arg))
                    .collect(),
            },
        };
        pattern.kind = kind;
        pattern
    }

    fn rewrite_imported_name(&self, name: &str) -> Option<String> {
        let (alias, member) = name.split_once('.')?;
        let namespace = self.imports.get(alias)?;
        Some(format!("{namespace}.{member}"))
    }

    fn declaration_name(&self, name: &str) -> String {
        qualify(&self.namespace, name)
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashSet::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn bind(&mut self, name: &str) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_owned());
        }
    }

    fn is_local(&self, name: &str) -> bool {
        self.scopes.iter().rev().any(|scope| scope.contains(name))
    }
}
