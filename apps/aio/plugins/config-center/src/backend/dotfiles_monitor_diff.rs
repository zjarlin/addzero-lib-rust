use std::ops::RangeInclusive;


#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LineChange {
    pub start: usize,
    pub end: usize,
}

impl LineChange {
    fn range(&self) -> RangeInclusive<usize> {
        self.start..=self.end
    }
}

pub fn changed_ranges(base: &str, next: &str) -> Vec<LineChange> {
    let base_lines = split_lines(base);
    let next_lines = split_lines(next);
    let matches = lcs_matches(&base_lines, &next_lines);
    let mut ranges = Vec::new();
    let mut base_index = 0;
    let mut next_index = 0;

    for (matched_base, matched_next) in matches {
        push_gap(
            &mut ranges,
            base_index,
            matched_base,
            next_index,
            matched_next,
        );
        base_index = matched_base + 1;
        next_index = matched_next + 1;
    }
    push_gap(
        &mut ranges,
        base_index,
        base_lines.len(),
        next_index,
        next_lines.len(),
    );

    ranges
}

pub fn first_overlap(left: &[LineChange], right: &[LineChange]) -> Option<LineChange> {
    for left_range in left {
        for right_range in right {
            if intersects(left_range.range(), right_range.range()) {
                return Some(LineChange {
                    start: left_range.start.min(right_range.start),
                    end: left_range.end.max(right_range.end),
                });
            }
        }
    }
    None
}

pub fn snippet(content: &str, start: usize, end: usize) -> String {
    split_lines(content)
        .into_iter()
        .enumerate()
        .filter_map(|(index, line)| {
            let number = index + 1;
            (number >= start && number <= end).then(|| format!("{number:>4}  {line}"))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn push_gap(
    ranges: &mut Vec<LineChange>,
    base_start: usize,
    base_end: usize,
    next_start: usize,
    next_end: usize,
) {
    if base_start == base_end && next_start == next_end {
        return;
    }
    let start = if base_start < base_end {
        base_start + 1
    } else {
        base_start.max(1)
    };
    let end = if base_start < base_end {
        base_end
    } else {
        start.max(next_end).max(next_start + 1)
    };
    ranges.push(LineChange { start, end });
}

fn lcs_matches(left: &[&str], right: &[&str]) -> Vec<(usize, usize)> {
    let mut table = vec![vec![0usize; right.len() + 1]; left.len() + 1];
    for left_index in (0..left.len()).rev() {
        for right_index in (0..right.len()).rev() {
            table[left_index][right_index] = if left[left_index] == right[right_index] {
                table[left_index + 1][right_index + 1] + 1
            } else {
                table[left_index + 1][right_index].max(table[left_index][right_index + 1])
            };
        }
    }

    let mut matches = Vec::new();
    let mut left_index = 0;
    let mut right_index = 0;
    while left_index < left.len() && right_index < right.len() {
        if left[left_index] == right[right_index] {
            matches.push((left_index, right_index));
            left_index += 1;
            right_index += 1;
        } else if table[left_index + 1][right_index] >= table[left_index][right_index + 1] {
            left_index += 1;
        } else {
            right_index += 1;
        }
    }
    matches
}

fn split_lines(content: &str) -> Vec<&str> {
    if content.is_empty() {
        Vec::new()
    } else {
        content.lines().collect()
    }
}

fn intersects(left: RangeInclusive<usize>, right: RangeInclusive<usize>) -> bool {
    left.start() <= right.end() && right.start() <= left.end()
}

#[cfg(test)]
mod tests {
    use super::{changed_ranges, first_overlap};

    #[test]
    fn detects_same_line_replacements() {
        let base = "a\nb\nc\n";
        let left = "a\nleft\nc\n";
        let right = "a\nright\nc\n";

        let overlap = first_overlap(&changed_ranges(base, left), &changed_ranges(base, right))
            .expect("same baseline line should conflict");

        assert_eq!(overlap.start, 2);
        assert_eq!(overlap.end, 2);
    }
}
