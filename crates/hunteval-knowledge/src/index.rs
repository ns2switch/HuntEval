use std::{collections::BTreeSet, fs, path::Path};

use crate::{
    Citation, KnowledgeError, KnowledgeManifest, LocalIndex, RetrievalRequest, RetrievalResult,
    RetrievedDocument,
};

const MAX_DOCUMENT_BYTES: u64 = 1_048_576;

#[derive(Debug, Clone)]
struct IndexedDocument {
    definition: crate::KnowledgeDocument,
    content: String,
    tokens: BTreeSet<String>,
}

#[derive(Debug, Clone)]
pub struct LocalKnowledgeIndex {
    manifest: KnowledgeManifest,
    documents: Vec<IndexedDocument>,
}

impl LocalKnowledgeIndex {
    pub fn open(root: &Path, manifest: KnowledgeManifest) -> Result<Self, KnowledgeError> {
        manifest.validate()?;
        if !manifest.enabled {
            return Err(KnowledgeError::Disabled);
        }
        let root = root
            .canonicalize()
            .map_err(|_| KnowledgeError::InvalidManifest)?;
        let corpus = root
            .join(&manifest.corpus_root)
            .canonicalize()
            .map_err(|_| KnowledgeError::InvalidManifest)?;
        if !corpus.starts_with(&root) {
            return Err(KnowledgeError::InvalidManifest);
        }
        let mut documents = Vec::with_capacity(manifest.documents.len());
        for definition in &manifest.documents {
            let path = corpus.join(&definition.path);
            let metadata =
                fs::symlink_metadata(&path).map_err(|_| KnowledgeError::DocumentUnavailable)?;
            if metadata.file_type().is_symlink()
                || !metadata.is_file()
                || metadata.len() > MAX_DOCUMENT_BYTES
            {
                return Err(KnowledgeError::DocumentUnavailable);
            }
            let canonical = path
                .canonicalize()
                .map_err(|_| KnowledgeError::DocumentUnavailable)?;
            if !canonical.starts_with(&corpus) {
                return Err(KnowledgeError::DocumentUnavailable);
            }
            let bytes = fs::read(canonical).map_err(|_| KnowledgeError::DocumentUnavailable)?;
            let content = String::from_utf8(bytes).map_err(|_| KnowledgeError::InvalidEncoding)?;
            documents.push(IndexedDocument {
                definition: definition.clone(),
                tokens: tokenize(&content),
                content,
            });
        }
        Ok(Self {
            manifest,
            documents,
        })
    }
}

impl LocalIndex for LocalKnowledgeIndex {
    fn retrieve(&self, request: &RetrievalRequest) -> Result<RetrievalResult, KnowledgeError> {
        let query_tokens = tokenize(&request.query);
        if request.query.trim().is_empty()
            || token_count(&request.query) > self.manifest.max_query_tokens as usize
            || request.max_documents == 0
            || request.max_documents > self.manifest.max_documents
            || request.max_tokens == 0
            || request.max_tokens > self.manifest.max_output_tokens
        {
            return Err(KnowledgeError::BudgetExceeded);
        }
        let mut ranked: Vec<_> = self
            .documents
            .iter()
            .filter_map(|document| {
                let score = query_tokens.intersection(&document.tokens).count();
                (score > 0).then_some((score, document))
            })
            .collect();
        ranked.sort_by(|(left_score, left), (right_score, right)| {
            right_score.cmp(left_score).then_with(|| {
                left.definition
                    .id
                    .as_str()
                    .cmp(right.definition.id.as_str())
            })
        });
        let mut remaining = request.max_tokens as usize;
        let mut documents = Vec::new();
        let mut citations = Vec::new();
        for (_, indexed) in ranked.into_iter().take(request.max_documents as usize) {
            let document_tokens = token_count(&indexed.content);
            if document_tokens > remaining {
                continue;
            }
            remaining -= document_tokens;
            let end = excerpt_end(&indexed.content, 240);
            citations.push(Citation {
                document_id: indexed.definition.id.clone(),
                start_byte: 0,
                end_byte: end,
                quote: indexed.content[..end].into(),
            });
            documents.push(RetrievedDocument {
                id: indexed.definition.id.clone(),
                title: indexed.definition.title.clone(),
                content: indexed.content.clone(),
                untrusted: true,
            });
        }
        Ok(RetrievalResult {
            query: request.query.clone(),
            documents,
            citations,
            latency_ms: 0,
            cost_microunits: 0,
        })
    }
}

fn tokenize(value: &str) -> BTreeSet<String> {
    value
        .split(|character: char| !character.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(str::to_lowercase)
        .collect()
}

fn token_count(value: &str) -> usize {
    value
        .split(|character: char| !character.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .count()
}

fn excerpt_end(value: &str, maximum: usize) -> usize {
    if value.len() <= maximum {
        return value.len();
    }
    let mut end = maximum;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    end
}
