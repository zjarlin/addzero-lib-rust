
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldInfoSimple {
    pub field_name: &'static str,
    pub description: Option<&'static str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldInfo {
    pub field_name: &'static str,
    pub description: Option<&'static str>,
    pub column_name: Option<&'static str>,
    pub type_name: &'static str,
    pub is_nested_object: bool,
    pub children: Vec<FieldInfo>,
}

impl FieldInfo {
    pub fn leaf(
        field_name: &'static str,
        description: Option<&'static str>,
        column_name: Option<&'static str>,
        type_name: &'static str,
    ) -> Self {
        Self {
            field_name,
            description,
            column_name,
            type_name,
            is_nested_object: false,
            children: Vec::new(),
        }
    }

    pub fn nested(
        field_name: &'static str,
        description: Option<&'static str>,
        column_name: Option<&'static str>,
        type_name: &'static str,
        children: Vec<FieldInfo>,
    ) -> Self {
        Self {
            field_name,
            description,
            column_name,
            type_name,
            is_nested_object: true,
            children,
        }
    }

    pub fn to_simple(&self) -> FieldInfoSimple {
        FieldInfoSimple {
            field_name: self.field_name,
            description: self.description,
        }
    }

    pub fn to_simple_with_children(&self) -> Vec<FieldInfoSimple> {
        let mut simple = vec![self.to_simple()];
        for child in &self.children {
            simple.extend(child.to_simple_with_children());
        }
        simple
    }

    pub fn to_simple_string(&self) -> String {
        let current = match self.description {
            Some(description) => format!("{}: {}", self.field_name, description),
            None => format!("{}: No description", self.field_name),
        };

        if self.children.is_empty() {
            return current;
        }

        let children = self
            .children
            .iter()
            .map(FieldInfo::to_simple_string)
            .collect::<Vec<_>>()
            .join(" ,  ");
        format!("{current} ,  {children}")
    }
}

pub trait MetaInfo {
    fn type_description() -> Option<&'static str> {
        None
    }

    fn field_infos() -> Vec<FieldInfo>;
}

pub fn get_field_infos<T: MetaInfo>() -> Vec<FieldInfo> {
    T::field_infos()
}

pub fn get_simple_field_info_str<T: MetaInfo>() -> String {
    get_field_infos::<T>()
        .into_iter()
        .map(|field| field.to_simple_string())
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn extract_table_name(sql: impl AsRef<str>) -> Option<String> {
    let sql = sql.as_ref();
    let lower = sql.to_ascii_lowercase();
    let mut search_from = 0usize;

    while let Some(relative_index) = lower[search_from..].find("from") {
        let from_start = search_from + relative_index;
        let from_end = from_start + "from".len();
        if !is_sql_word_boundary(lower.as_bytes(), from_start, from_end) {
            search_from = from_end;
            continue;
        }

        let after_from = &sql[from_end..];
        let table_start_offset = after_from
            .char_indices()
            .find_map(|(index, ch)| (!ch.is_whitespace()).then_some(index))?;
        let table_start = from_end + table_start_offset;
        let table = sql[table_start..]
            .chars()
            .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
            .collect::<String>();

        if table.is_empty() {
            return None;
        }
        return Some(table);
    }

    None
}

fn is_sql_word_boundary(bytes: &[u8], start: usize, end: usize) -> bool {
    let before_ok = start == 0 || !is_sql_word_byte(bytes[start - 1]);
    let after_ok = end >= bytes.len() || !is_sql_word_byte(bytes[end]);
    before_ok && after_ok
}

fn is_sql_word_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

pub fn guess_column_name(field_name: impl AsRef<str>) -> String {
    let source = field_name.as_ref();
    let chars = source.chars().collect::<Vec<_>>();
    let mut snake = String::with_capacity(source.len() + source.len() / 3);

    for (index, ch) in chars.iter().copied().enumerate() {
        if ch.is_ascii_uppercase() {
            let has_prev = index > 0;
            let prev_is_word = chars
                .get(index.saturating_sub(1))
                .is_some_and(|prev| prev.is_ascii_lowercase() || prev.is_ascii_digit());
            let next_is_lower = chars
                .get(index + 1)
                .is_some_and(|next| next.is_ascii_lowercase());
            let needs_separator = has_prev && (prev_is_word || next_is_lower);

            if needs_separator && !snake.ends_with('_') {
                snake.push('_');
            }
            snake.push(ch.to_ascii_lowercase());
        } else {
            snake.push(ch);
        }
    }

    snake
}
