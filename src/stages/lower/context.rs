use std::collections::HashMap;

use crate::stages::semantic::context::SymbolId;

#[derive(Debug, Clone, Copy)]
pub struct WordId(pub usize);

impl WordId {
    pub fn id(&self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TypeId(pub usize);

impl TypeId {
    pub fn id(&self) -> usize {
        self.0
    }
}

#[derive(Debug)]
pub struct WordIRInfo {
    pub name: String,
}

#[derive(Debug)]
pub struct TypeIRInfo {
    pub name: String,
}

#[derive(Default)]
pub struct IRContext {
    pub symbol_id_to_word_id: HashMap<SymbolId, WordId>,
    pub words: Vec<WordIRInfo>,
    pub symbol_id_to_type_id: HashMap<SymbolId, TypeId>,
    pub types: Vec<TypeIRInfo>,
}

impl IRContext {
    pub fn register_word(&mut self, symbol_id: SymbolId, word: WordIRInfo) -> Option<WordId> {
        let word_id = WordId(self.words.len());
        self.words.push(word);
        self.symbol_id_to_word_id.insert(symbol_id, word_id);
        Some(word_id)
    }

    pub fn get_word(&self, word_id: WordId) -> Option<&WordIRInfo> {
        self.words.get(word_id.id())
    }

    pub fn register_type(&mut self, symbol_id: SymbolId, ty: TypeIRInfo) -> Option<TypeId> {
        let type_id = TypeId(self.types.len());
        self.types.push(ty);
        self.symbol_id_to_type_id.insert(symbol_id, type_id);
        Some(type_id)
    }

    pub fn get_type(&self, type_id: TypeId) -> Option<&TypeIRInfo> {
        self.types.get(type_id.id())
    }
}
