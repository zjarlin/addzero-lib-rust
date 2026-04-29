//! Array and slice utility extensions.
//!
//! Provides convenience functions for common operations on slices and vectors
//! that aren't available in the standard library:
//!
//! - [`chunk`] — split into fixed-size chunks
//! - [`unique`] — deduplicate preserving order
//! - [`flatten_nested`] — flatten nested vectors
//! - [`zip_longest`] — zip with fill value for uneven lengths
//! - [`rotate_left`] / [`rotate_right`] — in-place rotation
//! - [`window`] — sliding windows
//! - [`frequencies`] — count occurrences
//! - [`partition`] — split by predicate
//! - [`pad_left`] — pad to desired length

use std::collections::HashMap;
use std::hash::Hash;

/// Splits `slice` into chunks of at most `size` elements.
///
/// The last chunk may contain fewer than `size` elements.
/// Returns an empty `Vec` if `size` is 0.
///
/// # Examples
///
/// ```
/// use addzero_array::chunk;
///
/// assert_eq!(chunk(&[1, 2, 3, 4, 5], 2), vec![vec![1, 2], vec![3, 4], vec![5]]);
/// assert_eq!(chunk(&[1, 2, 3], 10), vec![vec![1, 2, 3]]);
/// assert_eq!(chunk::<i32>(&[], 3), Vec::<Vec<i32>>::new());
/// ```
#[must_use]
pub fn chunk<T: Clone>(slice: &[T], size: usize) -> Vec<Vec<T>> {
    if size == 0 || slice.is_empty() {
        return Vec::new();
    }
    slice
        .chunks(size)
        .map(<[T]>::to_vec)
        .collect()
}

/// Returns a new `Vec` with duplicate elements removed, preserving the
/// original order of first occurrence.
///
/// # Examples
///
/// ```
/// use addzero_array::unique;
///
/// assert_eq!(unique(&[1, 2, 3, 2, 1]), vec![1, 2, 3]);
/// assert_eq!(unique(&[1, 1, 1]), vec![1]);
/// ```
#[must_use]
pub fn unique<T: Eq + Hash + Clone>(slice: &[T]) -> Vec<T> {
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();
    for item in slice {
        if seen.insert(item) {
            result.push(item.clone());
        }
    }
    result
}

/// Flattens a nested `Vec<Vec<T>>` into a single `Vec<T>`.
///
/// # Examples
///
/// ```
/// use addzero_array::flatten_nested;
///
/// assert_eq!(flatten_nested(&[vec![1, 2], vec![3], vec![4, 5]]), vec![1, 2, 3, 4, 5]);
/// ```
#[must_use]
pub fn flatten_nested<T: Clone>(nested: &[Vec<T>]) -> Vec<T> {
    nested.iter().flat_map(|v| v.iter().cloned()).collect()
}

/// Zips two slices together, using `fill` to pad the shorter slice.
///
/// # Examples
///
/// ```
/// use addzero_array::zip_longest;
///
/// assert_eq!(zip_longest(&[1, 2, 3], &[10, 20], 0), vec![(1, 10), (2, 20), (3, 0)]);
/// ```
#[must_use]
pub fn zip_longest<T: Clone>(a: &[T], b: &[T], fill: T) -> Vec<(T, T)> {
    let len = a.len().max(b.len());
    (0..len)
        .map(|i| {
            let av = a.get(i).cloned().unwrap_or_else(|| fill.clone());
            let bv = b.get(i).cloned().unwrap_or_else(|| fill.clone());
            (av, bv)
        })
        .collect()
}

/// Rotates `slice` left by `mid` positions in-place.
///
/// The element at index `mid` becomes the first element.
///
/// # Examples
///
/// ```
/// use addzero_array::rotate_left;
///
/// let mut v = vec![1, 2, 3, 4, 5];
/// rotate_left(&mut v, 2);
/// assert_eq!(v, vec![3, 4, 5, 1, 2]);
/// ```
pub fn rotate_left<T>(slice: &mut [T], mid: usize) {
    let mid = mid.min(slice.len());
    slice.rotate_left(mid);
}

/// Rotates `slice` right by `mid` positions in-place.
///
/// # Examples
///
/// ```
/// use addzero_array::rotate_right;
///
/// let mut v = vec![1, 2, 3, 4, 5];
/// rotate_right(&mut v, 2);
/// assert_eq!(v, vec![4, 5, 1, 2, 3]);
/// ```
pub fn rotate_right<T>(slice: &mut [T], mid: usize) {
    let mid = mid.min(slice.len());
    slice.rotate_right(mid);
}

/// Returns sliding windows of `size` elements from `slice`.
///
/// Returns an empty `Vec` if `size` is 0 or greater than the slice length.
///
/// # Examples
///
/// ```
/// use addzero_array::window;
///
/// assert_eq!(window(&[1, 2, 3, 4], 2), vec![vec![1, 2], vec![2, 3], vec![3, 4]]);
/// assert_eq!(window(&[1, 2], 3), Vec::<Vec<i32>>::new());
/// ```
#[must_use]
pub fn window<T: Clone>(slice: &[T], size: usize) -> Vec<Vec<T>> {
    if size == 0 || size > slice.len() {
        return Vec::new();
    }
    slice
        .windows(size)
        .map(<[T]>::to_vec)
        .collect()
}

/// Counts the occurrences of each element in `slice`.
///
/// # Examples
///
/// ```
/// use addzero_array::frequencies;
///
/// let freq = frequencies(&['a', 'b', 'a', 'c', 'b', 'a']);
/// assert_eq!(freq[&'a'], 3);
/// assert_eq!(freq[&'b'], 2);
/// assert_eq!(freq[&'c'], 1);
/// ```
#[must_use]
pub fn frequencies<T: Eq + Hash>(slice: &[T]) -> HashMap<&T, usize> {
    let mut map = HashMap::new();
    for item in slice {
        *map.entry(item).or_insert(0) += 1;
    }
    map
}

/// Partitions `slice` into two groups based on `pred`.
///
/// Returns `(matching, not_matching)`.
///
/// # Examples
///
/// ```
/// use addzero_array::partition;
///
/// let (evens, odds) = partition(&[1, 2, 3, 4, 5], |x| x % 2 == 0);
/// assert_eq!(evens, vec![&2, &4]);
/// assert_eq!(odds, vec![&1, &3, &5]);
/// ```
pub fn partition<T>(slice: &[T], pred: impl Fn(&T) -> bool) -> (Vec<&T>, Vec<&T>) {
    let mut pass = Vec::new();
    let mut fail = Vec::new();
    for item in slice {
        if pred(item) {
            pass.push(item);
        } else {
            fail.push(item);
        }
    }
    (pass, fail)
}

/// Pads `slice` to `len` elements by prepending `fill` values.
///
/// If `slice` is already at least `len` elements, returns a clone of it.
///
/// # Examples
///
/// ```
/// use addzero_array::pad_left;
///
/// assert_eq!(pad_left(&[3, 4], 5, 0), vec![0, 0, 0, 3, 4]);
/// assert_eq!(pad_left(&[1, 2, 3], 2, 0), vec![1, 2, 3]);
/// ```
#[must_use]
pub fn pad_left<T: Clone>(slice: &[T], len: usize, fill: T) -> Vec<T> {
    if slice.len() >= len {
        return slice.to_vec();
    }
    let diff = len - slice.len();
    let mut result = Vec::with_capacity(len);
    for _ in 0..diff {
        result.push(fill.clone());
    }
    result.extend_from_slice(slice);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_normal() {
        assert_eq!(chunk(&[1, 2, 3, 4, 5], 2), vec![vec![1, 2], vec![3, 4], vec![5]]);
    }

    #[test]
    fn test_chunk_uneven() {
        assert_eq!(chunk(&[1, 2, 3], 2), vec![vec![1, 2], vec![3]]);
    }

    #[test]
    fn test_chunk_size_zero() {
        assert!(chunk(&[1, 2, 3], 0).is_empty());
    }

    #[test]
    fn test_chunk_empty() {
        assert!(chunk::<i32>(&[], 3).is_empty());
    }

    #[test]
    fn test_unique() {
        assert_eq!(unique(&[1, 2, 3, 2, 1]), vec![1, 2, 3]);
        assert_eq!(unique(&[1, 1, 1]), vec![1]);
        assert_eq!(unique::<i32>(&[]), vec![]);
    }

    #[test]
    fn test_flatten_nested() {
        assert_eq!(flatten_nested(&[vec![1, 2], vec![3], vec![4, 5]]), vec![1, 2, 3, 4, 5]);
        assert_eq!(flatten_nested::<i32>(&[]), vec![]);
    }

    #[test]
    fn test_zip_longest() {
        assert_eq!(
            zip_longest(&[1, 2, 3], &[10, 20], 0),
            vec![(1, 10), (2, 20), (3, 0)]
        );
        assert_eq!(
            zip_longest(&[1], &[10, 20, 30], 0),
            vec![(1, 10), (0, 20), (0, 30)]
        );
    }

    #[test]
    fn test_rotate_left() {
        let mut v = vec![1, 2, 3, 4, 5];
        rotate_left(&mut v, 2);
        assert_eq!(v, vec![3, 4, 5, 1, 2]);
    }

    #[test]
    fn test_rotate_right() {
        let mut v = vec![1, 2, 3, 4, 5];
        rotate_right(&mut v, 2);
        assert_eq!(v, vec![4, 5, 1, 2, 3]);
    }

    #[test]
    fn test_window() {
        assert_eq!(
            window(&[1, 2, 3, 4], 2),
            vec![vec![1, 2], vec![2, 3], vec![3, 4]]
        );
    }

    #[test]
    fn test_window_too_large() {
        assert!(window(&[1, 2], 3).is_empty());
    }

    #[test]
    fn test_window_zero() {
        assert!(window(&[1, 2, 3], 0).is_empty());
    }

    #[test]
    fn test_frequencies() {
        let freq = frequencies(&['a', 'b', 'a', 'c', 'b', 'a']);
        assert_eq!(freq[&'a'], 3);
        assert_eq!(freq[&'b'], 2);
        assert_eq!(freq[&'c'], 1);
    }

    #[test]
    fn test_partition() {
        let (evens, odds) = partition(&[1, 2, 3, 4, 5], |x| x % 2 == 0);
        assert_eq!(evens, vec![&2, &4]);
        assert_eq!(odds, vec![&1, &3, &5]);
    }

    #[test]
    fn test_partition_all_match() {
        let (pass, fail) = partition(&[2, 4, 6], |x| x % 2 == 0);
        assert_eq!(pass, vec![&2, &4, &6]);
        assert!(fail.is_empty());
    }

    #[test]
    fn test_pad_left() {
        assert_eq!(pad_left(&[3, 4], 5, 0), vec![0, 0, 0, 3, 4]);
    }

    #[test]
    fn test_pad_left_already_long_enough() {
        assert_eq!(pad_left(&[1, 2, 3], 2, 0), vec![1, 2, 3]);
    }

    #[test]
    fn test_pad_left_exact() {
        assert_eq!(pad_left(&[1, 2], 2, 0), vec![1, 2]);
    }
}
