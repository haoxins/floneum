#![allow(unused_macros)]

pub use crate::exports::plugins::main::definitions::{
    Definition, Definitions, IoDefinition, PrimitiveValue, PrimitiveValueType, Value, ValueType,
};
use crate::plugins::main::imports::*;
pub use crate::plugins::main::imports::{print, GptNeoXType, LlamaType, ModelType, MptType};
pub use plugins::main::types::Embedding;
use plugins::main::types::{
    EitherStructure, NumberParameters, SequenceParameters, Structure, ThenStructure, UnsignedRange,
};
use std::ops::RangeInclusive;

wit_bindgen::generate!({path: "../wit", macro_export});

pub struct VectorDatabase {
    id: EmbeddingDbId,
    drop: bool,
}

impl VectorDatabase {
    pub fn new(embeddings: &[&plugins::main::types::Embedding], documents: &[&str]) -> Self {
        let id = create_embedding_db(embeddings, documents);

        VectorDatabase { id, drop: true }
    }

    pub fn find_closest_documents(
        &self,
        embedding: &plugins::main::types::Embedding,
        count: usize,
    ) -> Vec<String> {
        find_closest_documents(self.id, embedding, count as u32)
    }

    pub fn find_documents_within(
        &self,
        embedding: &plugins::main::types::Embedding,
        distance: f32,
    ) -> Vec<String> {
        find_documents_within(self.id, embedding, distance)
    }

    pub fn from_id(id: EmbeddingDbId) -> Self {
        VectorDatabase { id, drop: false }
    }

    pub fn leak(self) -> EmbeddingDbId {
        let id = self.id;
        std::mem::forget(self);
        id
    }

    pub fn manually_drop(self) {
        remove_embedding_db(self.id);
    }
}

impl Drop for VectorDatabase {
    fn drop(&mut self) {
        if self.drop {
            println!("Dropping vector database {}", self.id.id);
            remove_embedding_db(self.id);
        }
    }
}

pub struct ModelInstance {
    id: ModelId,
}

impl ModelInstance {
    pub fn new(ty: ModelType) -> Self {
        let id = load_model(ty);

        ModelInstance { id }
    }

    pub fn infer(&self, input: &str, max_tokens: Option<u32>, stop_on: Option<&str>) -> String {
        infer(self.id, input, max_tokens, stop_on)
    }

    pub fn infer_structured(
        &self,
        input: &str,
        max_tokens: Option<u32>,
        structure: Structured,
    ) -> String {
        infer_structured(self.id, input, max_tokens, structure.id)
    }

    pub fn get_embedding(&self, text: &str) -> Embedding {
        get_embedding(self.id, text)
    }
}

impl Drop for ModelInstance {
    fn drop(&mut self) {
        let id = self.id;
        unload_model(id);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Structured {
    id: StructureId,
}

impl Structured {
    pub fn literal(text: &str) -> Self {
        let inner = Structure::Literal(text);

        let id = create_structure(inner);
        Structured { id }
    }

    pub fn sequence_of(
        item: Structured,
        seperator: Structured,
        range: RangeInclusive<u64>,
    ) -> Self {
        let inner = Structure::Sequence(SequenceParameters {
            item: item.id,
            seperator: seperator.id,
            min_len: *range.start(),
            max_len: *range.end(),
        });
        let id = create_structure(inner);
        Structured { id }
    }

    pub fn float() -> Self {
        Self::ranged_float(f64::MIN..=f64::MAX)
    }

    pub fn ranged_float(range: RangeInclusive<f64>) -> Self {
        Self::number(range, false)
    }

    pub fn int() -> Self {
        Self::ranged_int(f64::MIN..=f64::MAX)
    }

    pub fn ranged_int(range: RangeInclusive<f64>) -> Self {
        Self::number(range, true)
    }

    pub fn number(range: RangeInclusive<f64>, int: bool) -> Self {
        let inner = Structure::Num(NumberParameters {
            min: *range.start(),
            max: *range.end(),
            integer: int,
        });
        let id = create_structure(inner);
        Structured { id }
    }

    pub fn str() -> Self {
        Self::ranged_str(0, u64::MAX)
    }

    pub fn ranged_str(min_len: u64, max_len: u64) -> Self {
        let inner = Structure::Str(UnsignedRange {
            min: min_len,
            max: max_len,
        });
        let id = create_structure(inner);
        Structured { id }
    }

    pub fn boolean() -> Self {
        Self::literal("true").or(Self::literal("false"))
    }

    pub fn null() -> Self {
        Self::literal("null")
    }

    pub fn or_not(self) -> Self {
        self.or(Self::null())
    }

    pub fn or(self, second: Structured) -> Self {
        let inner = Structure::Or(EitherStructure {
            first: self.id,
            second: second.id,
        });
        let id = create_structure(inner);
        Structured { id }
    }

    pub fn then(self, then: Structured) -> Self {
        let inner = Structure::Then(ThenStructure {
            first: self.id,
            second: then.id,
        });
        let id = create_structure(inner);
        Structured { id }
    }
}
