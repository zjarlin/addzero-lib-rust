use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldInfoSimple {
    pub field_name: &'static str,
    pub description: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    let regex =
        Regex::new(r"(?i)\bfrom\s+([a-zA-Z0-9_]+)").expect("table extraction regex should compile");
    regex
        .captures(sql.as_ref())
        .and_then(|captures| captures.get(1).map(|table| table.as_str().to_owned()))
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
