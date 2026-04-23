use std::collections::HashMap;

use crate::{
    common::Span,
    error::CompileError,
    stages::semantic::{context::SymbolId, hir::HIRType},
};

#[derive(Debug, Default)]
pub struct Substitution {
    type_vars: HashMap<SymbolId, HIRType>,
    stack_vars: HashMap<SymbolId, Vec<HIRType>>,
}

#[derive(Debug)]
pub struct CallAnalysis {
    stack: Vec<HIRType>,
    initial_stack: Vec<HIRType>,
    substitution: Substitution,
    expected_stack_in: Vec<HIRType>,
    expected_stack_out: Vec<HIRType>,
    span: Span,
}

impl CallAnalysis {
    pub fn new(
        initial_stack: Vec<HIRType>,
        expected_stack_in: Vec<HIRType>,
        expected_stack_out: Vec<HIRType>,
        span: Span,
    ) -> Self {
        Self {
            stack: initial_stack.clone(),
            initial_stack,
            substitution: Substitution::default(),
            expected_stack_in,
            expected_stack_out,
            span,
        }
    }

    fn unify_builtin(&mut self, top: HIRType, expected: SymbolId) -> Result<(), CompileError> {
        match top {
            HIRType::BuiltIn(symbol_id) => {
                if symbol_id != expected {
                    return Err(CompileError::InvalidStack {
                        label: "type mismatch".to_string(),
                        expected_stack: self
                            .expected_stack_in
                            .iter()
                            .map(|item| format!("{item:?}"))
                            .collect(),
                        actual_stack: self
                            .initial_stack
                            .iter()
                            .map(|item| format!("{item:?}"))
                            .collect(),
                        span: self.span,
                    });
                }
            }
            // HIRType::TypeVar(symbol_id) => {
            //     if let Some(unified) = self.substitution.type_vars.get(&symbol_id) {
            //         if unified != &top {
            //             return Err(CompileError::InvalidStack {
            //                 label: "type mismatch".to_string(),
            //                 expected_stack: self
            //                     .expected_stack_in
            //                     .iter()
            //                     .map(|item| format!("{item:?}"))
            //                     .collect(),
            //                 actual_stack: self
            //                     .initial_stack
            //                     .iter()
            //                     .map(|item| format!("{item:?}"))
            //                     .collect(),
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
                return Err(CompileError::InvalidStack {
                    label: "type mismatch".to_string(),
                    expected_stack: self
                        .expected_stack_in
                        .iter()
                        .map(|item| format!("{item:?}"))
                        .collect(),
                    actual_stack: self
                        .initial_stack
                        .iter()
                        .map(|item| format!("{item:?}"))
                        .collect(),
                    span: self.span,
                });
            }
        }

        Ok(())
    }

    fn unify_lambda(
        &mut self,
        top: HIRType,
        expected_stack_in: &[HIRType],
        expected_stack_out: &[HIRType],
    ) -> Result<(), CompileError> {
        let HIRType::Lambda {
            stack_in: actual_stack_in,
            stack_out: actual_stack_out,
        } = top
        else {
            return Err(CompileError::InvalidStack {
                label: "type mismatch".to_string(),
                expected_stack: self
                    .expected_stack_in
                    .iter()
                    .map(|item| format!("{item:?}"))
                    .collect(),
                actual_stack: self
                    .initial_stack
                    .iter()
                    .map(|item| format!("{item:?}"))
                    .collect(),
                span: self.span,
            });
        };
        self.unify_stack_exact(actual_stack_in, expected_stack_in.to_vec())?;
        self.unify_stack_exact(actual_stack_out, expected_stack_out.to_vec())?;
        Ok(())
    }

    fn unify_typevar(&mut self, top: HIRType, expected: SymbolId) -> Result<(), CompileError> {
        if let Some(unified) = self.substitution.type_vars.get(&expected) {
            if unified != &top {
                return Err(CompileError::InvalidStack {
                    label: "type mismatch".to_string(),
                    expected_stack: self
                        .expected_stack_in
                        .iter()
                        .map(|item| format!("{item:?}"))
                        .collect(),
                    actual_stack: self
                        .initial_stack
                        .iter()
                        .map(|item| format!("{item:?}"))
                        .collect(),
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
    ) -> Result<(), CompileError> {
        if let Some(unified) = self.substitution.stack_vars.get(&symbol_id) {
            if unified != stack {
                return Err(CompileError::InvalidStack {
                    label: "stack mismatch".to_string(),
                    expected_stack: self
                        .expected_stack_in
                        .iter()
                        .map(|item| format!("{item:?}"))
                        .collect(),
                    actual_stack: self
                        .initial_stack
                        .iter()
                        .map(|item| format!("{item:?}"))
                        .collect(),
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

    fn unify_type_pair(&mut self, actual: HIRType, expected: HIRType) -> Result<(), CompileError> {
        match expected {
            HIRType::BuiltIn(symbol_id) => {
                self.unify_builtin(actual, symbol_id)?;
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
                return Err(CompileError::InvalidStack {
                    label: "stack underflow".to_string(),
                    expected_stack: self
                        .expected_stack_in
                        .iter()
                        .map(|item| format!("{item:?}"))
                        .collect(),
                    actual_stack: self
                        .initial_stack
                        .iter()
                        .map(|item| format!("{item:?}"))
                        .collect(),
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
    ) -> Result<(), CompileError> {
        if let HIRType::StackVar(symbol_id) = expected {
            let rest = std::mem::take(actual_stack);
            self.unify_stackvar(&rest, symbol_id)?;
        } else {
            let Some(top) = actual_stack.pop() else {
                return Err(CompileError::InvalidStack {
                    label: "stack underflow".to_string(),
                    expected_stack: self
                        .expected_stack_in
                        .iter()
                        .map(|item| format!("{item:?}"))
                        .collect(),
                    actual_stack: self
                        .initial_stack
                        .iter()
                        .map(|item| format!("{item:?}"))
                        .collect(),
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
    ) -> Result<Vec<HIRType>, CompileError> {
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
    ) -> Result<(), CompileError> {
        let rest = self.unify_stack_pair(actual_stack, expected_stack.clone())?;

        if !rest.is_empty() {
            return Err(CompileError::InvalidStack {
                label: "stack mismatch".to_string(),
                expected_stack: expected_stack
                    .iter()
                    .map(|item| format!("{item:?}"))
                    .collect(),
                actual_stack: rest.iter().map(|item| format!("{item:?}")).collect(),
                span: self.span,
            });
        }

        Ok(())
    }

    fn unify(&mut self) -> Result<(), CompileError> {
        self.stack = self.unify_stack_pair(self.stack.clone(), self.expected_stack_in.clone())?;
        Ok(())
    }

    fn resolve_typevar(&self, symbol_id: SymbolId) -> Result<HIRType, CompileError> {
        self.substitution
            .type_vars
            .get(&symbol_id)
            .cloned()
            .ok_or_else(|| CompileError::InvalidStack {
                label: "cannot infer type variable".to_string(),
                expected_stack: self
                    .expected_stack_out
                    .iter()
                    .map(|item| format!("{item:?}"))
                    .collect(),
                actual_stack: self
                    .initial_stack
                    .iter()
                    .map(|item| format!("{item:?}"))
                    .collect(),
                span: self.span,
            })
    }

    fn resolve_stackvar(&self, symbol_id: SymbolId) -> Result<Vec<HIRType>, CompileError> {
        self.substitution
            .stack_vars
            .get(&symbol_id)
            .cloned()
            .ok_or_else(|| CompileError::InvalidStack {
                label: "cannot infer stack variable".to_string(),
                expected_stack: self
                    .expected_stack_out
                    .iter()
                    .map(|item| format!("{item:?}"))
                    .collect(),
                actual_stack: self
                    .initial_stack
                    .iter()
                    .map(|item| format!("{item:?}"))
                    .collect(),
                span: self.span,
            })
    }

    fn substitute_type_value(&self, ty: &HIRType) -> Result<HIRType, CompileError> {
        match ty {
            HIRType::BuiltIn(symbol_id) => Ok(HIRType::BuiltIn(*symbol_id)),

            HIRType::TypeVar(symbol_id) => self.resolve_typevar(*symbol_id),

            HIRType::Lambda {
                stack_in,
                stack_out,
            } => Ok(HIRType::Lambda {
                stack_in: self.substitute_stack_value(stack_in)?,
                stack_out: self.substitute_stack_value(stack_out)?,
            }),

            HIRType::StackVar(_) => Err(CompileError::InvalidStack {
                label: "stack variable cannot be substituted as a single type".to_string(),
                expected_stack: self
                    .expected_stack_out
                    .iter()
                    .map(|item| format!("{item:?}"))
                    .collect(),
                actual_stack: self
                    .initial_stack
                    .iter()
                    .map(|item| format!("{item:?}"))
                    .collect(),
                span: self.span,
            }),
        }
    }

    fn substitute_stack_value(&self, stack: &[HIRType]) -> Result<Vec<HIRType>, CompileError> {
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

    fn substitute(&mut self) -> Result<(), CompileError> {
        let substituted = self.substitute_stack_value(&self.expected_stack_out)?;

        for ty in substituted {
            self.stack.push(ty);
        }

        Ok(())
    }

    pub fn apply(&mut self) -> Result<Vec<HIRType>, CompileError> {
        self.unify()?;
        self.substitute()?;
        Ok(self.stack.clone())
    }
}

#[derive(Default)]
pub struct StackAnalysis {
    stack: Vec<HIRType>,
}

impl StackAnalysis {
    pub fn new(initial_stack: Vec<HIRType>) -> Self {
        Self {
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
    ) -> Result<(), CompileError> {
        let mut call_analysis = CallAnalysis::new(self.stack.clone(), stack_in, stack_out, span);
        self.stack = call_analysis.apply()?;
        Ok(())
    }

    pub fn match_stack(
        &self,
        expected_stack: Vec<HIRType>,
        span: Span,
    ) -> Result<(), CompileError> {
        if self.stack != expected_stack {
            return Err(CompileError::InvalidStack {
                label: "stack mismatch".to_string(),
                expected_stack: expected_stack
                    .iter()
                    .map(|item| format!("{item:?}"))
                    .collect(),
                actual_stack: self.stack.iter().map(|item| format!("{item:?}")).collect(),
                span,
            });
        }
        Ok(())
    }
}
