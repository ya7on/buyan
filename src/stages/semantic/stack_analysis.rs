use std::collections::HashMap;

use crate::{
    error::CompileError,
    stages::semantic::{context::SymbolId, hir::HIRType},
};

#[derive(Default)]
pub struct Substitution {
    type_vars: HashMap<SymbolId, HIRType>,
    stack_vars: HashMap<SymbolId, Vec<HIRType>>,
}

#[derive(Default)]
pub struct StackAnalysis {
    stack: Vec<HIRType>,
}

impl StackAnalysis {
    pub fn new(initial_stack: Vec<HIRType>) -> Self {
        Self {
            stack: initial_stack,
        }
    }

    pub fn push(&mut self, value: HIRType) {
        self.stack.push(value);
    }

    pub fn unify_builtin(&mut self, expected: SymbolId) -> Result<(), CompileError> {
        let Some(top) = self.stack.pop() else {
            return Err(CompileError::Unknown {
                label: "stack underflow".to_string(),
            });
        };

        if top != HIRType::BuiltIn(expected) {
            return Err(CompileError::Unknown {
                label: "type mismatch".to_string(),
            });
        }

        Ok(())
    }

    pub fn unify_lambda(
        &mut self,
        substitution: &mut Substitution,
        expected_stack_in: Vec<HIRType>,
        expected_stack_out: Vec<HIRType>,
    ) -> Result<(), CompileError> {
        let Some(top) = self.stack.pop() else {
            return Err(CompileError::Unknown {
                label: "stack underflow".to_string(),
            });
        };

        if let HIRType::Lambda {
            stack_in,
            stack_out,
        } = top
        {
            let mut actual_in = StackAnalysis::new(stack_in);
            actual_in.apply_in(substitution, expected_stack_in)?;
            if !actual_in.stack.is_empty() {
                return Err(CompileError::Unknown {
                    label: "lambda input stack mismatch".to_string(),
                });
            }

            let mut actual_out = StackAnalysis::new(stack_out);
            actual_out.apply_in(substitution, expected_stack_out)?;
            if !actual_out.stack.is_empty() {
                return Err(CompileError::Unknown {
                    label: "lambda output stack mismatch".to_string(),
                });
            }
        } else {
            return Err(CompileError::Unknown {
                label: "type mismatch".to_string(),
            });
        }

        Ok(())
    }

    pub fn unify_type_var(
        &mut self,
        substitution: &mut Substitution,
        expected: SymbolId,
    ) -> Result<(), CompileError> {
        let Some(top) = self.stack.pop() else {
            return Err(CompileError::Unknown {
                label: "stack underflow".to_string(),
            });
        };

        if let Some(unified) = substitution.type_vars.get(&expected) {
            if unified != &top {
                return Err(CompileError::Unknown {
                    label: "type mismatch".to_string(),
                });
            }
        } else {
            substitution.type_vars.insert(expected, top.clone());
        }

        Ok(())
    }

    pub fn unify_stack_var(
        &mut self,
        substitution: &mut Substitution,
        symbol_id: SymbolId,
    ) -> Result<(), CompileError> {
        if let Some(expected) = substitution.stack_vars.get(&symbol_id) {
            if expected != &self.stack {
                return Err(CompileError::Unknown {
                    label: "stack mismatch".to_string(),
                });
            }
        } else {
            substitution
                .stack_vars
                .insert(symbol_id, self.stack.clone());
        }

        self.stack.clear();

        Ok(())
    }

    pub fn apply_in(
        &mut self,
        substitution: &mut Substitution,
        stack_in: Vec<HIRType>,
    ) -> Result<(), CompileError> {
        let mut stack_in = stack_in;

        while let Some(next) = stack_in.pop() {
            match next {
                HIRType::BuiltIn(symbol_id) => {
                    self.unify_builtin(symbol_id)?;
                }
                HIRType::Lambda {
                    stack_in,
                    stack_out,
                } => {
                    self.unify_lambda(substitution, stack_in, stack_out)?;
                }
                HIRType::TypeVar(symbol_id) => {
                    self.unify_type_var(substitution, symbol_id)?;
                }
                HIRType::StackVar(symbol_id) => {
                    self.unify_stack_var(substitution, symbol_id)?;
                }
            }
        }

        Ok(())
    }

    fn apply_out(
        &mut self,
        substitution: &mut Substitution,
        stack_out: Vec<HIRType>,
    ) -> Result<(), CompileError> {
        for next in &stack_out {
            match next {
                HIRType::BuiltIn(symbol_id) => {
                    self.push(HIRType::BuiltIn(*symbol_id));
                }
                HIRType::Lambda {
                    stack_in,
                    stack_out,
                } => {
                    self.push(HIRType::Lambda {
                        stack_in: stack_in.clone(),
                        stack_out: stack_out.clone(),
                    });
                }
                HIRType::TypeVar(symbol_id) => {
                    let Some(unified) = substitution.type_vars.get(symbol_id) else {
                        return Err(CompileError::Unknown {
                            label: "type variable".to_string(),
                        });
                    };
                    self.push(unified.clone());
                }
                HIRType::StackVar(symbol_id) => {
                    let Some(unified) = substitution.stack_vars.get(symbol_id) else {
                        return Err(CompileError::Unknown {
                            label: "stack variable".to_string(),
                        });
                    };

                    for item in unified {
                        self.push(item.clone());
                    }
                }
            }
        }

        Ok(())
    }

    pub fn apply(
        &mut self,
        stack_in: Vec<HIRType>,
        stack_out: Vec<HIRType>,
    ) -> Result<(), CompileError> {
        let mut substitution = Substitution::default();

        self.apply_in(&mut substitution, stack_in)?;
        self.apply_out(&mut substitution, stack_out)?;

        Ok(())
    }

    pub fn check_output(&self, stack_out: Vec<HIRType>) -> Result<(), CompileError> {
        // TODO
        if self.stack.len() != stack_out.len() {
            return Err(CompileError::Unknown {
                label: "Stack output wrong".to_string(),
            });
        }

        for (a, b) in self.stack.iter().zip(stack_out.iter()) {
            if a != b {
                return Err(CompileError::Unknown {
                    label: "Stack output wrong".to_string(),
                });
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin() {
        let mut stack_analysis = StackAnalysis::new(vec![]);
        stack_analysis.push(HIRType::BuiltIn(SymbolId(6)));

        stack_analysis
            .apply(
                vec![HIRType::BuiltIn(SymbolId(6))],
                vec![HIRType::BuiltIn(SymbolId(7))],
            )
            .unwrap();

        assert_eq!(stack_analysis.stack, vec![HIRType::BuiltIn(SymbolId(7))])
    }

    #[test]
    fn test_drop() {
        let mut stack_analysis = StackAnalysis::new(vec![]);
        stack_analysis.push(HIRType::BuiltIn(SymbolId(6)));
        stack_analysis.push(HIRType::BuiltIn(SymbolId(7)));

        stack_analysis
            .apply(vec![HIRType::TypeVar(SymbolId(1))], vec![])
            .unwrap();

        assert_eq!(stack_analysis.stack, vec![HIRType::BuiltIn(SymbolId(6))])
    }

    #[test]
    fn test_dup() {
        let mut stack_analysis = StackAnalysis::new(vec![]);
        stack_analysis.push(HIRType::BuiltIn(SymbolId(6)));

        stack_analysis
            .apply(
                vec![HIRType::TypeVar(SymbolId(1))],
                vec![HIRType::TypeVar(SymbolId(1)), HIRType::TypeVar(SymbolId(1))],
            )
            .unwrap();

        assert_eq!(
            stack_analysis.stack,
            vec![HIRType::BuiltIn(SymbolId(6)), HIRType::BuiltIn(SymbolId(6))]
        )
    }

    #[test]
    fn test_swap() {
        let mut stack_analysis = StackAnalysis::new(vec![]);
        stack_analysis.push(HIRType::BuiltIn(SymbolId(6)));
        stack_analysis.push(HIRType::BuiltIn(SymbolId(7)));

        stack_analysis
            .apply(
                vec![HIRType::TypeVar(SymbolId(1)), HIRType::TypeVar(SymbolId(2))],
                vec![HIRType::TypeVar(SymbolId(2)), HIRType::TypeVar(SymbolId(1))],
            )
            .unwrap();

        assert_eq!(
            stack_analysis.stack,
            vec![HIRType::BuiltIn(SymbolId(7)), HIRType::BuiltIn(SymbolId(6))]
        )
    }
}
