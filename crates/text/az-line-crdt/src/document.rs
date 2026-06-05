use crate::error::{LineCrdtError, LineCrdtResult, engine_error};
use crate::line_index::{LineSpan, line_spans, unicode_len};
use crate::wire::{
    LineCrdtImportReport, LineCrdtPendingRange, LineCrdtSnapshot, LineCrdtUpdate, LineCrdtVersion,
};
use loro::{ExportMode, LoroDoc, LoroText, UpdateOptions, VersionVector};

const CONTENT_CONTAINER: &str = "content";

#[derive(Debug, Clone)]
pub struct LineCrdtDocument {
    doc: LoroDoc,
}

impl Default for LineCrdtDocument {
    fn default() -> Self {
        Self::new()
    }
}

impl LineCrdtDocument {
    pub fn new() -> Self {
        Self {
            doc: LoroDoc::new(),
        }
    }

    pub fn with_peer_id(peer_id: u64) -> LineCrdtResult<Self> {
        let document = Self::new();
        document
            .doc
            .set_peer_id(peer_id)
            .map_err(|error| engine_error("set peer id", error))?;
        Ok(document)
    }

    pub fn from_text(text: &str) -> LineCrdtResult<Self> {
        let document = Self::new();
        document.set_text(text)?;
        Ok(document)
    }

    pub fn from_text_with_peer_id(text: &str, peer_id: u64) -> LineCrdtResult<Self> {
        let document = Self::with_peer_id(peer_id)?;
        document.set_text(text)?;
        Ok(document)
    }

    pub fn from_snapshot(snapshot: impl AsRef<[u8]>) -> LineCrdtResult<Self> {
        let doc = LoroDoc::from_snapshot(snapshot.as_ref())
            .map_err(|error| engine_error("import snapshot", error))?;
        Ok(Self { doc })
    }

    pub fn from_snapshot_with_peer_id(
        snapshot: impl AsRef<[u8]>,
        peer_id: u64,
    ) -> LineCrdtResult<Self> {
        let document = Self::with_peer_id(peer_id)?;
        document.import_blob(snapshot)?;
        Ok(document)
    }

    pub fn peer_id(&self) -> u64 {
        self.doc.peer_id()
    }

    pub fn text(&self) -> String {
        self.content().to_string()
    }

    pub fn lines(&self) -> Vec<String> {
        self.text().split('\n').map(str::to_owned).collect()
    }

    pub fn line_count(&self) -> usize {
        line_spans(&self.text()).len()
    }

    pub fn set_text(&self, text: &str) -> LineCrdtResult<()> {
        let content = self.content();
        let current_len = content.len_unicode();
        if current_len > 0 {
            content
                .delete(0, current_len)
                .map_err(|error| engine_error("delete current text", error))?;
        }
        if !text.is_empty() {
            content
                .insert(0, text)
                .map_err(|error| engine_error("insert text", error))?;
        }
        self.doc.commit();
        Ok(())
    }

    pub fn apply_text_by_line(&self, text: &str) -> LineCrdtResult<()> {
        self.content()
            .update_by_line(text, UpdateOptions::default())
            .map_err(|error| engine_error("update text by line", error))?;
        self.doc.commit();
        Ok(())
    }

    pub fn apply_text_precise(&self, text: &str) -> LineCrdtResult<()> {
        self.content()
            .update(text, UpdateOptions::default())
            .map_err(|error| engine_error("update text", error))?;
        self.doc.commit();
        Ok(())
    }

    pub fn insert_line(&self, index: usize, line: &str) -> LineCrdtResult<()> {
        validate_single_line(line)?;

        let current = self.text();
        let spans = line_spans(&current);
        if index > spans.len() {
            return Err(LineCrdtError::LineIndexOutOfBounds {
                index,
                line_count: spans.len(),
            });
        }

        let position = spans
            .get(index)
            .map(|span| span.content_start)
            .unwrap_or_else(|| unicode_len(&current));
        let inserted = insertion_text_for_line(&current, index, spans.len(), line);
        if !inserted.is_empty() {
            self.content()
                .insert(position, &inserted)
                .map_err(|error| engine_error("insert line", error))?;
            self.doc.commit();
        }

        Ok(())
    }

    pub fn append_line(&self, line: &str) -> LineCrdtResult<()> {
        self.insert_line(self.line_count(), line)
    }

    pub fn replace_line(&self, index: usize, line: &str) -> LineCrdtResult<()> {
        validate_single_line(line)?;

        let current = self.text();
        let spans = line_spans(&current);
        let span = span_at(&spans, index)?;
        let content = self.content();
        let existing_len = span.content_len();

        if existing_len > 0 {
            content
                .delete(span.content_start, existing_len)
                .map_err(|error| engine_error("delete line content", error))?;
        }
        if !line.is_empty() {
            content
                .insert(span.content_start, line)
                .map_err(|error| engine_error("insert line content", error))?;
        }

        self.doc.commit();
        Ok(())
    }

    pub fn delete_line(&self, index: usize) -> LineCrdtResult<()> {
        let current = self.text();
        let spans = line_spans(&current);
        let span = span_at(&spans, index)?;
        let (delete_start, delete_end) = delete_bounds_for_line(&spans, index, span);

        if delete_end > delete_start {
            self.content()
                .delete(delete_start, delete_end - delete_start)
                .map_err(|error| engine_error("delete line", error))?;
            self.doc.commit();
        }

        Ok(())
    }

    pub fn insert_text(&self, unicode_index: usize, text: &str) -> LineCrdtResult<()> {
        if text.is_empty() {
            return Ok(());
        }

        self.content()
            .insert(unicode_index, text)
            .map_err(|error| engine_error("insert text", error))?;
        self.doc.commit();
        Ok(())
    }

    pub fn delete_text(&self, unicode_index: usize, unicode_len: usize) -> LineCrdtResult<()> {
        if unicode_len == 0 {
            return Ok(());
        }

        self.content()
            .delete(unicode_index, unicode_len)
            .map_err(|error| engine_error("delete text", error))?;
        self.doc.commit();
        Ok(())
    }

    pub fn splice_text(
        &self,
        unicode_index: usize,
        unicode_len: usize,
        replacement: &str,
    ) -> LineCrdtResult<()> {
        self.content()
            .splice(unicode_index, unicode_len, replacement)
            .map_err(|error| engine_error("splice text", error))?;
        self.doc.commit();
        Ok(())
    }

    pub fn version(&self) -> LineCrdtVersion {
        LineCrdtVersion::from_bytes(self.doc.oplog_vv().encode())
    }

    pub fn export_snapshot(&self) -> LineCrdtResult<LineCrdtSnapshot> {
        self.doc
            .export(ExportMode::Snapshot)
            .map(LineCrdtSnapshot::from_bytes)
            .map_err(|error| engine_error("export snapshot", error))
    }

    pub fn export_all_updates(&self) -> LineCrdtResult<LineCrdtUpdate> {
        self.doc
            .export(ExportMode::all_updates())
            .map(LineCrdtUpdate::from_bytes)
            .map_err(|error| engine_error("export all updates", error))
    }

    pub fn export_updates_since(
        &self,
        version: &LineCrdtVersion,
    ) -> LineCrdtResult<LineCrdtUpdate> {
        let version = decode_version(version)?;
        self.doc
            .export(ExportMode::updates(&version))
            .map(LineCrdtUpdate::from_bytes)
            .map_err(|error| engine_error("export updates", error))
    }

    pub fn import_update(&self, update: impl AsRef<[u8]>) -> LineCrdtResult<LineCrdtImportReport> {
        self.import_blob(update)
    }

    pub fn import_snapshot(
        &self,
        snapshot: impl AsRef<[u8]>,
    ) -> LineCrdtResult<LineCrdtImportReport> {
        self.import_blob(snapshot)
    }

    fn content(&self) -> LoroText {
        self.doc.get_text(CONTENT_CONTAINER)
    }

    fn import_blob(&self, blob: impl AsRef<[u8]>) -> LineCrdtResult<LineCrdtImportReport> {
        let status = self
            .doc
            .import(blob.as_ref())
            .map_err(|error| engine_error("import CRDT blob", error))?;
        let pending_ranges = status
            .pending
            .map(|pending| {
                pending
                    .iter()
                    .map(|(peer_id, (start, end))| LineCrdtPendingRange {
                        peer_id: *peer_id,
                        start: *start,
                        end: *end,
                    })
                    .collect()
            })
            .unwrap_or_default();
        self.doc.commit();

        Ok(LineCrdtImportReport { pending_ranges })
    }
}

fn decode_version(version: &LineCrdtVersion) -> LineCrdtResult<VersionVector> {
    VersionVector::decode(version.as_bytes()).map_err(|error| LineCrdtError::InvalidVersion {
        reason: format!("{error:?}"),
    })
}

fn span_at(spans: &[LineSpan], index: usize) -> LineCrdtResult<LineSpan> {
    spans
        .get(index)
        .copied()
        .ok_or(LineCrdtError::LineIndexOutOfBounds {
            index,
            line_count: spans.len(),
        })
}

fn validate_single_line(line: &str) -> LineCrdtResult<()> {
    if line.contains('\n') {
        return Err(LineCrdtError::LineContainsNewline);
    }

    Ok(())
}

fn insertion_text_for_line(current: &str, index: usize, line_count: usize, line: &str) -> String {
    if current.is_empty() {
        return line.to_owned();
    }

    if index < line_count {
        return format!("{line}\n");
    }

    if current.ends_with('\n') {
        line.to_owned()
    } else {
        format!("\n{line}")
    }
}

fn delete_bounds_for_line(spans: &[LineSpan], index: usize, span: LineSpan) -> (usize, usize) {
    if spans.len() == 1 {
        return (span.content_start, span.content_end);
    }

    if span.has_trailing_newline {
        return (span.content_start, span.content_end + 1);
    }

    if index > 0 {
        (span.content_start - 1, span.content_end)
    } else {
        (span.content_start, span.content_end)
    }
}
