automod::dir!(pub(crate) "src/models");

pub(crate) fn knowledge_models() -> toasty::ModelSet {
    toasty::models!(
        knowledge_source::KnowledgeSourceRecord,
        knowledge_document::KnowledgeDocumentRecord
    )
}
