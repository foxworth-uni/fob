//! JSX/React element building support
//!
//! This module provides ergonomic builders for JSX elements using OXC's AST.

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_ast::{AstBuilder, NONE};
use oxc_span::{Atom, SPAN};

/// JSX element builder
///
/// Provides methods for building JSX elements, attributes, and children.
pub struct JsxBuilder<'a> {
    ast: AstBuilder<'a>,
}

impl<'a> JsxBuilder<'a> {
    /// Create a new JSX builder
    pub fn new(alloc: &'a Allocator) -> Self {
        Self {
            ast: AstBuilder::new(alloc),
        }
    }

    /// Get the underlying AST builder
    pub fn ast(&self) -> &AstBuilder<'a> {
        &self.ast
    }

    /// Create a JSX element: `<Tag attr="value">children</Tag>`
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let jsx = JsxBuilder::new(&allocator);
    /// let element = jsx.element(
    ///     "div",
    ///     vec![jsx.attr("className", jsx.string_attr("container"))],
    ///     vec![jsx.text("Hello")],
    ///     false,
    /// );
    /// ```
    pub fn element(
        &self,
        name: impl Into<Atom<'a>>,
        attributes: Vec<JSXAttributeItem<'a>>,
        children: Vec<JSXChild<'a>>,
        self_closing: bool,
    ) -> JSXElement<'a> {
        let name_atom = name.into();

        // Create opening element name
        let opening_ident = self.ast.jsx_identifier(SPAN, name_atom);
        let opening_name = JSXElementName::Identifier(self.ast.alloc(opening_ident));

        let attrs_vec = self.ast.vec_from_iter(attributes);

        // Build JSXOpeningElement using AstBuilder
        // Note: self_closing is inferred from absence of closing_element
        let opening_element = self
            .ast
            .jsx_opening_element(SPAN, opening_name, NONE, attrs_vec);

        // Create closing element if not self-closing (None = self-closing)
        let closing_element: Option<JSXClosingElement> = if self_closing {
            None
        } else {
            let closing_ident = self.ast.jsx_identifier(SPAN, name_atom);
            let closing_name = JSXElementName::Identifier(self.ast.alloc(closing_ident));
            Some(self.closing_element(closing_name))
        };

        let children_vec = self.ast.vec_from_iter(children);
        self.ast.jsx_element(
            SPAN,
            opening_element,
            children_vec,
            closing_element.map(|e| self.ast.alloc(e)),
        )
    }

    /// Create a JSX closing element
    fn closing_element(&self, name: JSXElementName<'a>) -> JSXClosingElement<'a> {
        JSXClosingElement { span: SPAN, name }
    }

    /// Create a JSX attribute: `name="value"`
    pub fn attr(
        &self,
        name: impl Into<Atom<'a>>,
        value: Option<JSXAttributeValue<'a>>,
    ) -> JSXAttributeItem<'a> {
        let attr_name = self.ast.jsx_attribute_name_identifier(SPAN, name);
        let attr = self.ast.jsx_attribute(SPAN, attr_name, value);
        JSXAttributeItem::Attribute(self.ast.alloc(attr))
    }

    /// Create a string attribute value: `"value"`
    pub fn string_attr(&self, value: impl Into<Atom<'a>>) -> JSXAttributeValue<'a> {
        JSXAttributeValue::StringLiteral(self.ast.alloc(self.ast.string_literal(SPAN, value, None)))
    }

    /// Create an expression attribute value: `{expr}`
    pub fn expr_attr(&self, expr: Expression<'a>) -> JSXAttributeValue<'a> {
        JSXAttributeValue::ExpressionContainer(self.ast.alloc(JSXExpressionContainer {
            span: SPAN,
            expression: JSXExpression::from(expr),
        }))
    }

    /// Create a JSX text child
    pub fn text(&self, value: impl Into<Atom<'a>>) -> JSXChild<'a> {
        let value_atom = value.into();
        let text = self
            .ast
            .jsx_text(SPAN, value_atom, Some(value_atom));
        JSXChild::Text(self.ast.alloc(text))
    }

    /// Create a JSX element child
    pub fn child(&self, element: JSXElement<'a>) -> JSXChild<'a> {
        JSXChild::Element(self.ast.alloc(element))
    }

    /// Create a JSX expression child: `{expr}`
    pub fn expr_child(&self, expr: Expression<'a>) -> JSXChild<'a> {
        JSXChild::ExpressionContainer(self.ast.alloc(JSXExpressionContainer {
            span: SPAN,
            expression: JSXExpression::from(expr),
        }))
    }

    /// Create a JSX fragment: `<>children</>`
    pub fn fragment(&self, children: Vec<JSXChild<'a>>) -> JSXFragment<'a> {
        JSXFragment {
            span: SPAN,
            opening_fragment: JSXOpeningFragment { span: SPAN },
            closing_fragment: JSXClosingFragment { span: SPAN },
            children: self.ast.vec_from_iter(children),
        }
    }

    /// Convert JSX element to expression
    pub fn jsx_expr(&self, element: JSXElement<'a>) -> Expression<'a> {
        Expression::JSXElement(self.ast.alloc(element))
    }

    /// Convert JSX fragment to expression
    pub fn jsx_fragment_expr(&self, fragment: JSXFragment<'a>) -> Expression<'a> {
        Expression::JSXFragment(self.ast.alloc(fragment))
    }
}
