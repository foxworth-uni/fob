use std::fs;
use std::path::{Path, PathBuf};

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Class, Comment, Declaration, ExportDefaultDeclaration, ExportDefaultDeclarationKind,
    ExportNamedDeclaration, Function, Statement, TSExportAssignment, TSInterfaceDeclaration,
    TSTypeAliasDeclaration, VariableDeclarator,
};
use oxc_span::{GetSpan, Span};
use rustc_hash::{FxHashMap, FxHashSet};
use fob_gen::{parse, ParseOptions};

use crate::error::{DocsError, Result};
use crate::jsdoc::parse_jsdoc;
use crate::model::{Documentation, ExportedSymbol, ModuleDoc, SourceLocation, SymbolKind};

/// Options controlling documentation extraction.
#[derive(Debug, Clone)]
pub struct ExtractOptions {
    /// Include symbols marked with `@internal`.
    pub include_internal: bool,
}

impl Default for ExtractOptions {
    fn default() -> Self {
        Self {
            include_internal: false,
        }
    }
}

/// Extracts documentation from JavaScript / TypeScript modules using OXC.
#[derive(Debug, Clone)]
pub struct DocsExtractor {
    options: ExtractOptions,
}

impl DocsExtractor {
    /// Create a new extractor with the provided options.
    pub fn new(options: ExtractOptions) -> Self {
        Self { options }
    }

    /// Extract documentation from multiple files at once.
    pub fn extract_many<I>(&self, inputs: I) -> Result<Documentation>
    where
        I: IntoIterator<Item = PathBuf>,
    {
        let mut documentation = Documentation::default();
        for path in inputs {
            let module = self.extract_from_path(&path)?;
            documentation.add_module(module);
        }
        Ok(documentation)
    }

    /// Extract documentation from a file on disk.
    pub fn extract_from_path(&self, path: impl AsRef<Path>) -> Result<ModuleDoc> {
        let path = path.as_ref();
        let source = fs::read_to_string(path).map_err(|error| DocsError::Io {
            path: path.to_path_buf(),
            error,
        })?;
        self.extract_from_source(path, &source)
    }

    /// Extract documentation from an in-memory source string.
    pub fn extract_from_source(&self, path: impl AsRef<Path>, source: &str) -> Result<ModuleDoc> {
        let path = path.as_ref();
        
        // Use fob-gen's ParseOptions to infer source type from path
        let parse_opts = ParseOptions::from_path(
            path.to_str().unwrap_or("")
        );

        let allocator = Allocator::default();
        
        // Use fob-gen's parse function
        let parsed = parse(&allocator, source, parse_opts).map_err(|e| {
            DocsError::parse_error(
                path.to_path_buf(),
                &[format!("Parse error: {}", e)]
            )
        })?;

        // Check for parse diagnostics
        if parsed.has_errors() {
            let diagnostics: Vec<String> = parsed
                .diagnostics
                .iter()
                .map(|d| d.message.clone())
                .collect();
            return Err(DocsError::parse_error(path.to_path_buf(), &diagnostics));
        }

        let program = parsed.ast();
        let mut module = ModuleDoc::new(path.to_string_lossy());

        let comment_map = build_comment_map(program.comments.iter());
        let mut consumed_comments: FxHashSet<(u32, u32)> = FxHashSet::default();
        let line_index = LineIndex::new(parsed.source_text);

        let mut first_symbol_start: Option<u32> = None;

        for statement in program.body.iter() {
            let records = self.collect_symbols_for_statement(
                statement,
                &program.comments,
                &comment_map,
                &mut consumed_comments,
                &line_index,
            );

            for record in records {
                first_symbol_start = Some(
                    first_symbol_start
                        .map_or(record.node_start, |current| current.min(record.node_start)),
                );
                if let Some(span) = record.comment_span {
                    consumed_comments.insert(span);
                }
                module.symbols.push(record.symbol);
            }
        }

        derive_module_description(
            &mut module,
            parsed.source_text,
            program.comments.iter(),
            &consumed_comments,
            first_symbol_start,
            &self.options,
        );

        Ok(module)
    }

    fn collect_symbols_for_statement<'a>(
        &self,
        statement: &Statement<'a>,
        _comments: &'a oxc_allocator::Vec<'a, Comment>,
        comment_map: &CommentMap<'a>,
        consumed_comments: &mut FxHashSet<(u32, u32)>,
        line_index: &LineIndex,
    ) -> Vec<SymbolRecord> {
        match statement {
            Statement::ExportNamedDeclaration(export) => {
                self.handle_export_named(export, comment_map, consumed_comments, line_index)
            }
            Statement::ExportDefaultDeclaration(export) => {
                self.handle_export_default(export, comment_map, consumed_comments, line_index)
            }
            Statement::TSExportAssignment(ts_export) => self.handle_ts_export_assignment(
                ts_export,
                comment_map,
                consumed_comments,
                line_index,
            ),
            Statement::TSNamespaceExportDeclaration(namespace_export) => {
                let span = namespace_export.span;
                let location = line_index.location(span.start);
                let mut symbol = ExportedSymbol::new(
                    namespace_export.id.name.to_string(),
                    SymbolKind::Other,
                    location,
                );
                if let Some(doc_info) =
                    self.comment_for_span(span, comment_map, consumed_comments, line_index)
                {
                    apply_parsed_doc(&mut symbol, doc_info.parsed);
                    consumed_comments.insert(doc_info.comment_span);
                }
                vec![SymbolRecord::new(symbol, span.start, None)]
            }
            Statement::ExportAllDeclaration(_) => Vec::new(),
            _ => Vec::new(),
        }
    }

    fn handle_export_named<'a>(
        &self,
        export: &ExportNamedDeclaration<'a>,
        comment_map: &CommentMap<'a>,
        consumed_comments: &mut FxHashSet<(u32, u32)>,
        line_index: &LineIndex,
    ) -> Vec<SymbolRecord> {
        let mut records = Vec::new();
        if let Some(declaration) = &export.declaration {
            let span = export.span;
            let comment_info =
                self.comment_for_span(span, comment_map, consumed_comments, line_index);

            if let Some(info) = &comment_info {
                if info.should_skip(&self.options) {
                    return Vec::new();
                }
            }

            match declaration {
                Declaration::VariableDeclaration(variable) => {
                    for declarator in variable.declarations.iter() {
                        if let Some(record) = self.symbol_from_variable_declarator(
                            declarator,
                            SymbolKind::Variable,
                            line_index,
                            comment_info.as_ref(),
                        ) {
                            records.push(record);
                        }
                    }
                }
                Declaration::FunctionDeclaration(function) => {
                    if let Some(record) = self.symbol_from_function(
                        function,
                        SymbolKind::Function,
                        export.span.start,
                        line_index,
                        comment_info.as_ref(),
                    ) {
                        records.push(record);
                    }
                }
                Declaration::ClassDeclaration(class) => {
                    if let Some(record) = self.symbol_from_class(
                        class,
                        SymbolKind::Class,
                        export.span.start,
                        line_index,
                        comment_info.as_ref(),
                    ) {
                        records.push(record);
                    }
                }
                Declaration::TSTypeAliasDeclaration(type_alias) => {
                    if let Some(record) = self.symbol_from_type_alias(
                        type_alias,
                        export.span.start,
                        line_index,
                        comment_info.as_ref(),
                    ) {
                        records.push(record);
                    }
                }
                Declaration::TSInterfaceDeclaration(interface) => {
                    if let Some(record) = self.symbol_from_interface(
                        interface,
                        export.span.start,
                        line_index,
                        comment_info.as_ref(),
                    ) {
                        records.push(record);
                    }
                }
                Declaration::TSEnumDeclaration(enumeration) => {
                    let location = line_index.location(enumeration.span.start);
                    let mut symbol = ExportedSymbol::new(
                        enumeration.id.name.to_string(),
                        SymbolKind::Enum,
                        location,
                    );
                    if let Some(info) = comment_info.as_ref() {
                        apply_parsed_doc(&mut symbol, info.parsed.clone());
                    }
                    records.push(SymbolRecord::new(
                        symbol,
                        export.span.start,
                        comment_info.as_ref().map(|info| info.comment_span),
                    ));
                }
                Declaration::TSModuleDeclaration(module_decl) => {
                    let location = line_index.location(module_decl.span.start);
                    let mut symbol = ExportedSymbol::new(
                        module_decl.id.name().to_string(),
                        SymbolKind::Other,
                        location,
                    );
                    if let Some(info) = comment_info.as_ref() {
                        apply_parsed_doc(&mut symbol, info.parsed.clone());
                    }
                    records.push(SymbolRecord::new(
                        symbol,
                        export.span.start,
                        comment_info.as_ref().map(|info| info.comment_span),
                    ));
                }
                Declaration::TSImportEqualsDeclaration(_) => {}
            }

            if let Some(info) = comment_info {
                consumed_comments.insert(info.comment_span);
            }
        }
        records
    }

    fn handle_export_default<'a>(
        &self,
        export: &ExportDefaultDeclaration<'a>,
        comment_map: &CommentMap<'a>,
        consumed_comments: &mut FxHashSet<(u32, u32)>,
        line_index: &LineIndex,
    ) -> Vec<SymbolRecord> {
        let span = export.span;
        let comment_info = self.comment_for_span(span, comment_map, consumed_comments, line_index);

        if let Some(info) = &comment_info {
            if info.should_skip(&self.options) {
                return Vec::new();
            }
        }

        let mut result = Vec::new();
        match &export.declaration {
            ExportDefaultDeclarationKind::FunctionDeclaration(function) => {
                if let Some(record) = self.symbol_from_function(
                    function,
                    SymbolKind::DefaultExport,
                    span.start,
                    line_index,
                    comment_info.as_ref(),
                ) {
                    result.push(record);
                }
            }
            ExportDefaultDeclarationKind::ClassDeclaration(class) => {
                if let Some(record) = self.symbol_from_class(
                    class,
                    SymbolKind::DefaultExport,
                    span.start,
                    line_index,
                    comment_info.as_ref(),
                ) {
                    result.push(record);
                }
            }
            _ => {
                let location = line_index.location(span.start);
                let mut symbol =
                    ExportedSymbol::new("default", SymbolKind::DefaultExport, location);
                if let Some(info) = comment_info.as_ref() {
                    apply_parsed_doc(&mut symbol, info.parsed.clone());
                }
                result.push(SymbolRecord::new(
                    symbol,
                    span.start,
                    comment_info.as_ref().map(|info| info.comment_span),
                ));
            }
        }

        if let Some(info) = comment_info {
            consumed_comments.insert(info.comment_span);
        }

        result
    }

    fn handle_ts_export_assignment<'a>(
        &self,
        export: &TSExportAssignment<'a>,
        comment_map: &CommentMap<'a>,
        consumed_comments: &mut FxHashSet<(u32, u32)>,
        line_index: &LineIndex,
    ) -> Vec<SymbolRecord> {
        let span = export.span;
        let comment_info = self.comment_for_span(span, comment_map, consumed_comments, line_index);

        if let Some(info) = &comment_info {
            if info.should_skip(&self.options) {
                return Vec::new();
            }
        }

        let location = line_index.location(span.start);
        let mut symbol = ExportedSymbol::new("default", SymbolKind::DefaultExport, location);
        if let Some(info) = comment_info {
            apply_parsed_doc(&mut symbol, info.parsed);
            consumed_comments.insert(info.comment_span);
            return vec![SymbolRecord::new(
                symbol,
                span.start,
                Some(info.comment_span),
            )];
        }

        vec![SymbolRecord::new(symbol, span.start, None)]
    }

    fn symbol_from_function(
        &self,
        function: &Function,
        kind: SymbolKind,
        export_start: u32,
        line_index: &LineIndex,
        comment_info: Option<&CommentInfo>,
    ) -> Option<SymbolRecord> {
        let name = function
            .id
            .as_ref()
            .map(|id| id.name.to_string())
            .unwrap_or_else(|| "default".to_string());
        let location = line_index.location(function.span.start);
        let mut symbol = ExportedSymbol::new(name, kind, location);
        if let Some(info) = comment_info {
            apply_parsed_doc(&mut symbol, info.parsed.clone());
        }
        Some(SymbolRecord::new(
            symbol,
            export_start,
            comment_info.map(|info| info.comment_span),
        ))
    }

    fn symbol_from_class(
        &self,
        class: &Class,
        kind: SymbolKind,
        export_start: u32,
        line_index: &LineIndex,
        comment_info: Option<&CommentInfo>,
    ) -> Option<SymbolRecord> {
        let name = class
            .id
            .as_ref()
            .map(|id| id.name.to_string())
            .unwrap_or_else(|| "default".to_string());
        let location = line_index.location(class.span.start);
        let mut symbol = ExportedSymbol::new(name, kind, location);
        if let Some(info) = comment_info {
            apply_parsed_doc(&mut symbol, info.parsed.clone());
        }
        Some(SymbolRecord::new(
            symbol,
            export_start,
            comment_info.map(|info| info.comment_span),
        ))
    }

    fn symbol_from_variable_declarator(
        &self,
        declarator: &VariableDeclarator,
        kind: SymbolKind,
        line_index: &LineIndex,
        comment_info: Option<&CommentInfo>,
    ) -> Option<SymbolRecord> {
        let name = binding_pattern_to_name(&declarator.id)?;
        let location = line_index.location(declarator.id.span().start);
        let mut symbol = ExportedSymbol::new(name, kind, location);
        if let Some(info) = comment_info {
            apply_parsed_doc(&mut symbol, info.parsed.clone());
        }
        Some(SymbolRecord::new(
            symbol,
            declarator.span.start,
            comment_info.map(|info| info.comment_span),
        ))
    }

    fn symbol_from_type_alias(
        &self,
        type_alias: &TSTypeAliasDeclaration,
        export_start: u32,
        line_index: &LineIndex,
        comment_info: Option<&CommentInfo>,
    ) -> Option<SymbolRecord> {
        let location = line_index.location(type_alias.span.start);
        let mut symbol = ExportedSymbol::new(
            type_alias.id.name.to_string(),
            SymbolKind::TypeAlias,
            location,
        );
        if let Some(info) = comment_info {
            apply_parsed_doc(&mut symbol, info.parsed.clone());
        }
        Some(SymbolRecord::new(
            symbol,
            export_start,
            comment_info.map(|info| info.comment_span),
        ))
    }

    fn symbol_from_interface(
        &self,
        interface: &TSInterfaceDeclaration,
        export_start: u32,
        line_index: &LineIndex,
        comment_info: Option<&CommentInfo>,
    ) -> Option<SymbolRecord> {
        let location = line_index.location(interface.span.start);
        let mut symbol = ExportedSymbol::new(
            interface.id.name.to_string(),
            SymbolKind::Interface,
            location,
        );
        if let Some(info) = comment_info {
            apply_parsed_doc(&mut symbol, info.parsed.clone());
        }
        Some(SymbolRecord::new(
            symbol,
            export_start,
            comment_info.map(|info| info.comment_span),
        ))
    }

    fn comment_for_span<'a>(
        &self,
        span: Span,
        comment_map: &CommentMap<'a>,
        consumed_comments: &FxHashSet<(u32, u32)>,
        line_index: &LineIndex,
    ) -> Option<CommentInfo> {
        comment_map.get(&span.start).and_then(|comment| {
            if consumed_comments.contains(&(comment.span.start, comment.span.end)) {
                return None;
            }
            let content_span = comment.content_span();
            let parsed = parse_jsdoc(slice_source(line_index.source, content_span));
            Some(CommentInfo {
                comment_span: (comment.span.start, comment.span.end),
                parsed,
            })
        })
    }
}

type CommentMap<'a> = FxHashMap<u32, &'a Comment>;

struct CommentInfo {
    comment_span: (u32, u32),
    parsed: crate::jsdoc::ParsedJsDoc,
}

impl CommentInfo {
    fn should_skip(&self, options: &ExtractOptions) -> bool {
        !options.include_internal && self.parsed.is_internal
    }
}

struct SymbolRecord {
    symbol: ExportedSymbol,
    node_start: u32,
    comment_span: Option<(u32, u32)>,
}

impl SymbolRecord {
    fn new(symbol: ExportedSymbol, node_start: u32, comment_span: Option<(u32, u32)>) -> Self {
        Self {
            symbol,
            node_start,
            comment_span,
        }
    }
}

fn build_comment_map<'a, I>(comments: I) -> CommentMap<'a>
where
    I: IntoIterator<Item = &'a Comment>,
{
    let mut map = CommentMap::default();
    for comment in comments {
        if comment.is_jsdoc() {
            map.insert(comment.attached_to, comment);
        }
    }
    map
}

fn derive_module_description<'a>(
    module: &mut ModuleDoc,
    source: &'a str,
    comments: impl IntoIterator<Item = &'a Comment>,
    consumed: &FxHashSet<(u32, u32)>,
    first_symbol_start: Option<u32>,
    options: &ExtractOptions,
) {
    if module.description.is_some() {
        return;
    }

    for comment in comments {
        if !comment.is_jsdoc() {
            continue;
        }
        if consumed.contains(&(comment.span.start, comment.span.end)) {
            continue;
        }
        if let Some(limit) = first_symbol_start {
            if comment.span.start >= limit {
                continue;
            }
        }
        let parsed = parse_jsdoc(slice_source(source, comment.content_span()));
        if parsed.is_internal && !options.include_internal {
            continue;
        }
        if let Some(summary) = parsed.summary.map(|s| s.trim().to_string()) {
            if !summary.is_empty() {
                module.description = Some(summary);
                break;
            }
        }
    }
}

fn binding_pattern_to_name(pattern: &oxc_ast::ast::BindingPattern) -> Option<String> {
    match &pattern.kind {
        oxc_ast::ast::BindingPatternKind::BindingIdentifier(ident) => Some(ident.name.to_string()),
        _ => None,
    }
}

fn apply_parsed_doc(symbol: &mut ExportedSymbol, parsed: crate::jsdoc::ParsedJsDoc) {
    if let Some(summary) = parsed.summary {
        let trimmed = summary.trim();
        if !trimmed.is_empty() {
            symbol.summary = Some(trimmed.to_string());
        }
    }
    if !parsed.parameters.is_empty() {
        symbol.parameters = parsed.parameters;
    }
    if let Some(returns) = parsed.returns {
        let trimmed = returns.trim();
        if !trimmed.is_empty() {
            symbol.returns = Some(trimmed.to_string());
        }
    }
    symbol.deprecated = parsed.deprecated;
    if !parsed.examples.is_empty() {
        symbol.examples = parsed.examples;
    }
    if !parsed.tags.is_empty() {
        symbol.tags = parsed.tags;
    }
}

#[derive(Debug)]
struct LineIndex<'a> {
    source: &'a str,
    line_starts: Vec<u32>,
}

impl<'a> LineIndex<'a> {
    fn new(source: &'a str) -> Self {
        let mut line_starts = Vec::with_capacity(128);
        line_starts.push(0);
        for (idx, byte) in source.bytes().enumerate() {
            if byte == b'\n' {
                line_starts.push((idx + 1) as u32);
            }
        }
        Self {
            source,
            line_starts,
        }
    }

    fn location(&self, offset: u32) -> SourceLocation {
        let idx = match self.line_starts.binary_search(&offset) {
            Ok(index) => index,
            Err(index) => index.saturating_sub(1),
        };
        let line = idx as u32 + 1;
        let column = offset - self.line_starts[idx] + 1;
        SourceLocation::new(line, column)
    }
}

fn slice_source(source: &str, span: Span) -> &str {
    let start = span.start as usize;
    let end = span.end as usize;
    &source[start..end]
}
