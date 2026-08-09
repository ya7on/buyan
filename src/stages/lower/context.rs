use std::collections::HashMap;

use crate::stages::{
    lower::ir::IRType,
    semantic::{context::SymbolId, hir::HIRType},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct WordId(pub usize);

impl WordId {
    #[must_use]
    pub const fn id(&self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypeId(pub usize);

impl TypeId {
    #[must_use]
    pub const fn id(&self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct WordIRInfo {
    pub name: String,
    pub source_word: SymbolId,
    pub type_args: Vec<HIRType>,
}

#[derive(Debug, Default)]
pub struct IRContext {
    pub words: Vec<WordIRInfo>,
    pub external_symbols: HashMap<SymbolId, String>,
    pub symbol_id_to_type_id: HashMap<SymbolId, TypeId>,
    pub types: Vec<IRType>,
}

impl IRContext {
    pub fn register_word(&mut self, word: WordIRInfo) -> Option<WordId> {
        if self
            .get_word_id(word.source_word, &word.type_args)
            .is_some()
        {
            return None;
        }
        let word_id = WordId(self.words.len());
        self.words.push(word);
        Some(word_id)
    }

    #[must_use]
    pub fn get_word_id(&self, source_word: SymbolId, type_args: &[HIRType]) -> Option<WordId> {
        self.words
            .iter()
            .position(|word| word.source_word == source_word && word.type_args == type_args)
            .map(WordId)
    }

    pub fn register_type(&mut self, symbol_id: SymbolId, ty: IRType) -> Option<TypeId> {
        let type_id = TypeId(self.types.len());
        self.types.push(ty);
        self.symbol_id_to_type_id.insert(symbol_id, type_id);
        Some(type_id)
    }

    pub fn register_external_symbol(&mut self, symbol_id: SymbolId, symbol: String) {
        self.external_symbols.insert(symbol_id, symbol);
    }

    #[must_use]
    pub fn get_type(&self, type_id: TypeId) -> Option<&IRType> {
        self.types.get(type_id.id())
    }
}
