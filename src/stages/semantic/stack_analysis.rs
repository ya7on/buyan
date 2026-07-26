use std::collections::HashMap;

use crate::{
    common::Span,
    error::DiagnosticMessage,
    stages::semantic::{
        context::{HIRContext, SymbolId, SymbolKind},
        hir::HIRType,
    },
};

#[derive(Debug, Default)]
pub struct Substitution {
    type_vars: HashMap<SymbolId, HIRType>,
    stack_vars: HashMap<SymbolId, Vec<HIRType>>,
}

#[derive(Debug)]
pub struct CallAnalysis<'a> {
    hir_ctx: &'a HIRContext,
    stack: Vec<HIRType>,
    initial_stack: Vec<HIRType>,
    substitution: Substitution,
    expected_stack_in: Vec<HIRType>,
    expected_stack_out: Vec<HIRType>,
    span: Span,
}

impl<'a> CallAnalysis<'a> {
    pub fn new(
        hir_ctx: &'a HIRContext,
        initial_stack: Vec<HIRType>,
        expected_stack_in: Vec<HIRType>,
        expected_stack_out: Vec<HIRType>,
        span: Span,
    ) -> Self {
        Self {
            hir_ctx,
            stack: initial_stack.clone(),
            initial_stack,
            substitution: Substitution::default(),
            expected_stack_in,
            expected_stack_out,
            span,
        }
    }

    fn unify_builtin(&mut self, top: HIRType, expected: SymbolId) -> Result<(), DiagnosticMessage> {
        match top {
            HIRType::BuiltIn(symbol_id) => {
                if symbol_id != expected {
                    return Err(DiagnosticMessage::InvalidStack {
                        label: "type mismatch".to_string(),
                        expected_stack: self
                            .expected_stack_in
                            .iter()
                            .map(|item| self.hir_ctx.format_type(item))
                            .collect(),
                        actual_stack: self
                            .initial_stack
                            .iter()
                            .map(|item| self.hir_ctx.format_type(item))
                            .collect(),
                        additional_spans: Vec::new(),
                        span: self.span,
                    });
                }
            }
            // HIRType::TypeVar(symbol_id) => {
            //     if let Some(unified) = self.substitution.type_vars.get(&symbol_id) {
            //         if unified != &top {
            //             return Err(DiagnosticMessage::InvalidStack {
            //                 label: "type mismatch".to_string(),
            //                 expected_stack: self
            //                     .expected_stack_in
            //                     .iter()
            //                     .map(|item| self.hir_ctx.format_type(item))
            //                     .collect(),
            //                 actual_stack: self
            //                     .initial_stack
            //                     .iter()
            //                     .map(|item| self.hir_ctx.format_type(item))
            //                     .collect(),
            //                 additional_spans: Vec::new(),
            //                 span: self.span,
            //             });
            //         }
            //     } else {
            //         self.substitution
            //             .type_vars
            //             .insert(symbol_id, HIRType::BuiltIn(symbol_id));
            //     }
            // }
            _ => {
                return Err(DiagnosticMessage::InvalidStack {
                    label: "type mismatch".to_string(),
                    expected_stack: self
                        .expected_stack_in
                        .iter()
                        .map(|item| self.hir_ctx.format_type(item))
                        .collect(),
                    actual_stack: self
                        .initial_stack
                        .iter()
                        .map(|item| self.hir_ctx.format_type(item))
                        .collect(),
                    additional_spans: Vec::new(),
                    span: self.span,
                });
            }
        }

        Ok(())
    }

    fn unify_struct(&mut self, top: HIRType, expected: SymbolId) -> Result<(), DiagnosticMessage> {
        if top != HIRType::Struct(expected) {
            return Err(DiagnosticMessage::InvalidStack {
                label: "type mismatch".to_string(),
                expected_stack: self
                    .expected_stack_in
                    .iter()
                    .map(|item| self.hir_ctx.format_type(item))
                    .collect(),
                actual_stack: self
                    .initial_stack
                    .iter()
                    .map(|item| self.hir_ctx.format_type(item))
                    .collect(),
                additional_spans: Vec::new(),
                span: self.span,
            });
        }
        Ok(())
    }

    fn unify_lambda(
        &mut self,
        top: HIRType,
        expected_stack_in: &[HIRType],
        expected_stack_out: &[HIRType],
    ) -> Result<(), DiagnosticMessage> {
        let HIRType::Lambda {
            stack_in: actual_stack_in,
            stack_out: actual_stack_out,
        } = top
        else {
            return Err(DiagnosticMessage::InvalidStack {
                label: "type mismatch".to_string(),
                expected_stack: self
                    .expected_stack_in
                    .iter()
                    .map(|item| self.hir_ctx.format_type(item))
                    .collect(),
                actual_stack: self
                    .initial_stack
                    .iter()
                    .map(|item| self.hir_ctx.format_type(item))
                    .collect(),
                additional_spans: Vec::new(),
                span: self.span,
            });
        };
        self.unify_stack_exact(actual_stack_in, expected_stack_in.to_vec())?;
        self.unify_stack_exact(actual_stack_out, expected_stack_out.to_vec())?;
        Ok(())
    }

    fn unify_typevar(&mut self, top: HIRType, expected: SymbolId) -> Result<(), DiagnosticMessage> {
        let has_required_traits = match self.hir_ctx.get(expected) {
            Some(SymbolKind::TypeVar {
                traits: required_traits,
                ..
            }) => required_traits.iter().all(|required_trait| match &top {
                HIRType::BuiltIn(symbol_id) => {
                    matches!(
                        self.hir_ctx.get(*symbol_id),
                        Some(SymbolKind::Type { traits, .. })
                            if traits.contains(&required_trait.value)
                    )
                }
                HIRType::TypeVar(symbol_id) => {
                    matches!(
                        self.hir_ctx.get(*symbol_id),
                        Some(SymbolKind::TypeVar { traits, .. })
                            if traits.iter().any(
                                |available_trait| available_trait.value == required_trait.value
                            )
                    )
                }
                HIRType::Struct(_) | HIRType::StackVar(_) | HIRType::Lambda { .. } => false,
            }),
            _ => false,
        };
        if !has_required_traits {
            return Err(DiagnosticMessage::InvalidStack {
                label: "trait constraint not satisfied".to_string(),
                expected_stack: self
                    .expected_stack_in
                    .iter()
                    .map(|item| self.hir_ctx.format_type(item))
                    .collect(),
                actual_stack: self
                    .initial_stack
                    .iter()
                    .map(|item| self.hir_ctx.format_type(item))
                    .collect(),
                additional_spans: Vec::new(),
                span: self.span,
            });
        }

        if let Some(unified) = self.substitution.type_vars.get(&expected) {
            if unified != &top {
                return Err(DiagnosticMessage::InvalidStack {
                    label: "type mismatch".to_string(),
                    expected_stack: self
                        .expected_stack_in
                        .iter()
                        .map(|item| self.hir_ctx.format_type(item))
                        .collect(),
                    actual_stack: self
                        .initial_stack
                        .iter()
                        .map(|item| self.hir_ctx.format_type(item))
                        .collect(),
                    additional_spans: Vec::new(),
                    span: self.span,
                });
            }
        } else {
            self.substitution.type_vars.insert(expected, top.clone());
        }
        Ok(())
    }

    fn unify_stackvar(
        &mut self,
        stack: &[HIRType],
        symbol_id: SymbolId,
    ) -> Result<(), DiagnosticMessage> {
        if let Some(unified) = self.substitution.stack_vars.get(&symbol_id) {
            if unified != stack {
                return Err(DiagnosticMessage::InvalidStack {
                    label: "stack mismatch".to_string(),
                    expected_stack: self
                        .expected_stack_in
                        .iter()
                        .map(|item| self.hir_ctx.format_type(item))
                        .collect(),
                    actual_stack: self
                        .initial_stack
                        .iter()
                        .map(|item| self.hir_ctx.format_type(item))
                        .collect(),
                    additional_spans: Vec::new(),
                    span: self.span,
                });
            }
        } else {
            self.substitution
                .stack_vars
                .insert(symbol_id, stack.to_vec());
        }
        Ok(())
    }

    fn unify_type_pair(
        &mut self,
        actual: HIRType,
        expected: HIRType,
    ) -> Result<(), DiagnosticMessage> {
        match expected {
            HIRType::BuiltIn(symbol_id) => {
                self.unify_builtin(actual, symbol_id)?;
            }
            HIRType::Struct(symbol_id) => {
                self.unify_struct(actual, symbol_id)?;
            }
            HIRType::TypeVar(symbol_id) => {
                self.unify_typevar(actual, symbol_id)?;
            }
            HIRType::Lambda {
                stack_in,
                stack_out,
            } => {
                self.unify_lambda(actual, &stack_in, &stack_out)?;
            }
            HIRType::StackVar(_) => {
                return Err(DiagnosticMessage::InvalidStack {
                    label: "stack underflow".to_string(),
                    expected_stack: self
                        .expected_stack_in
                        .iter()
                        .map(|item| self.hir_ctx.format_type(item))
                        .collect(),
                    actual_stack: self
                        .initial_stack
                        .iter()
                        .map(|item| self.hir_ctx.format_type(item))
                        .collect(),
                    additional_spans: Vec::new(),
                    span: self.span,
                });
            }
        }
        Ok(())
    }

    fn unify_type(
        &mut self,
        actual_stack: &mut Vec<HIRType>,
        expected: HIRType,
    ) -> Result<(), DiagnosticMessage> {
        if let HIRType::StackVar(symbol_id) = expected {
            let rest = std::mem::take(actual_stack);
            self.unify_stackvar(&rest, symbol_id)?;
        } else {
            let Some(top) = actual_stack.pop() else {
                return Err(DiagnosticMessage::InvalidStack {
                    label: "stack underflow".to_string(),
                    expected_stack: self
                        .expected_stack_in
                        .iter()
                        .map(|item| self.hir_ctx.format_type(item))
                        .collect(),
                    actual_stack: self
                        .initial_stack
                        .iter()
                        .map(|item| self.hir_ctx.format_type(item))
                        .collect(),
                    additional_spans: Vec::new(),
                    span: self.span,
                });
            };
            self.unify_type_pair(top, expected)?;
        }
        Ok(())
    }

    fn unify_stack_pair(
        &mut self,
        actual: Vec<HIRType>,
        expected: Vec<HIRType>,
    ) -> Result<Vec<HIRType>, DiagnosticMessage> {
        let mut actual = actual;
        for ty in expected.iter().rev() {
            self.unify_type(actual.as_mut(), ty.clone())?;
        }
        Ok(actual)
    }

    fn unify_stack_exact(
        &mut self,
        actual_stack: Vec<HIRType>,
        expected_stack: Vec<HIRType>,
    ) -> Result<(), DiagnosticMessage> {
        let original_actual_stack = actual_stack.clone();
        let rest = self.unify_stack_pair(actual_stack, expected_stack.clone())?;

        if !rest.is_empty() {
            return Err(DiagnosticMessage::InvalidStack {
                label: "stack mismatch".to_string(),
                expected_stack: expected_stack
                    .iter()
                    .map(|item| self.hir_ctx.format_type(item))
                    .collect(),
                actual_stack: original_actual_stack
                    .iter()
                    .map(|item| self.hir_ctx.format_type(item))
                    .collect(),
                additional_spans: Vec::new(),
                span: self.span,
            });
        }

        Ok(())
    }

    fn unify(&mut self) -> Result<(), DiagnosticMessage> {
        self.stack = self.unify_stack_pair(self.stack.clone(), self.expected_stack_in.clone())?;
        Ok(())
    }

    fn resolve_typevar(&self, symbol_id: SymbolId) -> Result<HIRType, DiagnosticMessage> {
        self.substitution
            .type_vars
            .get(&symbol_id)
            .cloned()
            .ok_or_else(|| DiagnosticMessage::InvalidStack {
                label: "cannot infer type variable".to_string(),
                expected_stack: self
                    .expected_stack_out
                    .iter()
                    .map(|item| self.hir_ctx.format_type(item))
                    .collect(),
                actual_stack: self
                    .initial_stack
                    .iter()
                    .map(|item| self.hir_ctx.format_type(item))
                    .collect(),
                additional_spans: Vec::new(),
                span: self.span,
            })
    }

    fn resolve_stackvar(&self, symbol_id: SymbolId) -> Result<Vec<HIRType>, DiagnosticMessage> {
        self.substitution
            .stack_vars
            .get(&symbol_id)
            .cloned()
            .ok_or_else(|| DiagnosticMessage::InvalidStack {
                label: "cannot infer stack variable".to_string(),
                expected_stack: self
                    .expected_stack_out
                    .iter()
                    .map(|item| self.hir_ctx.format_type(item))
                    .collect(),
                actual_stack: self
                    .initial_stack
                    .iter()
                    .map(|item| self.hir_ctx.format_type(item))
                    .collect(),
                additional_spans: Vec::new(),
                span: self.span,
            })
    }

    fn substitute_type_value(&self, ty: &HIRType) -> Result<HIRType, DiagnosticMessage> {
        match ty {
            HIRType::BuiltIn(symbol_id) => Ok(HIRType::BuiltIn(*symbol_id)),
            HIRType::Struct(symbol_id) => Ok(HIRType::Struct(*symbol_id)),

            HIRType::TypeVar(symbol_id) => self.resolve_typevar(*symbol_id),

            HIRType::Lambda {
                stack_in,
                stack_out,
            } => Ok(HIRType::Lambda {
                stack_in: self.substitute_stack_value(stack_in)?,
                stack_out: self.substitute_stack_value(stack_out)?,
            }),

            HIRType::StackVar(_) => Err(DiagnosticMessage::InvalidStack {
                label: "stack variable cannot be substituted as a single type".to_string(),
                expected_stack: self
                    .expected_stack_out
                    .iter()
                    .map(|item| self.hir_ctx.format_type(item))
                    .collect(),
                actual_stack: self
                    .initial_stack
                    .iter()
                    .map(|item| self.hir_ctx.format_type(item))
                    .collect(),
                additional_spans: Vec::new(),
                span: self.span,
            }),
        }
    }

    fn substitute_stack_value(&self, stack: &[HIRType]) -> Result<Vec<HIRType>, DiagnosticMessage> {
        let mut result = Vec::new();

        for ty in stack {
            match ty {
                HIRType::StackVar(symbol_id) => {
                    let resolved = self.resolve_stackvar(*symbol_id)?;
                    result.extend(resolved);
                }

                _ => {
                    let resolved = self.substitute_type_value(ty)?;
                    result.push(resolved);
                }
            }
        }

        Ok(result)
    }

    fn substitute(&mut self) -> Result<(), DiagnosticMessage> {
        let substituted = self.substitute_stack_value(&self.expected_stack_out)?;

        for ty in substituted {
            self.stack.push(ty);
        }

        Ok(())
    }

    pub fn apply(&mut self) -> Result<Vec<HIRType>, DiagnosticMessage> {
        self.unify()?;
        self.substitute()?;
        Ok(self.stack.clone())
    }
}

pub struct StackAnalysis<'a> {
    hir_ctx: &'a HIRContext,
    stack: Vec<HIRType>,
}

impl<'a> StackAnalysis<'a> {
    pub fn new(hir_ctx: &'a HIRContext, initial_stack: Vec<HIRType>) -> Self {
        Self {
            hir_ctx,
            stack: initial_stack.clone(),
        }
    }

    pub fn push(&mut self, ty: HIRType) {
        self.stack.push(ty);
    }

    pub fn apply_call(
        &mut self,
        stack_in: Vec<HIRType>,
        stack_out: Vec<HIRType>,
        span: Span,
    ) -> Result<(), DiagnosticMessage> {
        let mut call_analysis =
            CallAnalysis::new(self.hir_ctx, self.stack.clone(), stack_in, stack_out, span);
        self.stack = call_analysis.apply()?;
        Ok(())
    }

    pub fn match_stack(
        &self,
        expected_stack: Vec<HIRType>,
        span: Span,
        additional_spans: Vec<Span>,
    ) -> Result<(), DiagnosticMessage> {
        if self.stack != expected_stack {
            return Err(DiagnosticMessage::InvalidStack {
                label: "stack mismatch".to_string(),
                expected_stack: expected_stack
                    .iter()
                    .map(|item| self.hir_ctx.format_type(item))
                    .collect(),
                actual_stack: self
                    .stack
                    .iter()
                    .map(|item| self.hir_ctx.format_type(item))
                    .collect(),
                span,
                additional_spans,
            });
        }
        Ok(())
    }
}
