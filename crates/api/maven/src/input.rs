pub(crate) use az_str::transformation::trim_non_blank;

pub(crate) fn non_blank(value: Option<&str>) -> Option<&str> {
    trim_non_blank(value)
}
