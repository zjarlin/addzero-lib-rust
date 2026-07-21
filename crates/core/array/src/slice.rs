//! 数组和切片的实用工具扩展。
//!
//! 提供标准库中不可用的切片和向量常用操作便捷函数：
//!
//! - [`chunk`] — 按固定大小分块
//! - [`unique`] — 保持顺序的去重
//! - [`flatten_nested`] — 展平嵌套向量
//! - [`zip_longest`] — 不等长时以填充值压缩
//! - [`rotate_left`] / [`rotate_right`] — 原地旋转
//! - [`window`] — 滑动窗口
//! - [`frequencies`] — 出现次数统计
//! - [`partition`] — 按谓词分割
//! - [`pad_left`] — 左填充至指定长度

use std::collections::HashMap;
use std::hash::Hash;

/// 将切片按最多 `size` 个元素分成多个块。
///
/// 最后一个块可能少于 `size` 个元素。`size` 为 0 时返回空向量。
///
/// # Examples
///
/// ```
/// use az_array::slice::chunk;
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
    slice.chunks(size).map(<[T]>::to_vec).collect()
}

/// 返回保持首次出现顺序的去重结果。
///
/// # Examples
///
/// ```
/// use az_array::slice::unique;
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

/// 将嵌套向量展平成单层向量。
///
/// # Examples
///
/// ```
/// use az_array::slice::flatten_nested;
///
/// assert_eq!(flatten_nested(&[vec![1, 2], vec![3], vec![4, 5]]), vec![1, 2, 3, 4, 5]);
/// ```
#[must_use]
pub fn flatten_nested<T: Clone>(nested: &[Vec<T>]) -> Vec<T> {
    nested.iter().flat_map(|v| v.iter().cloned()).collect()
}

/// 将两个切片按位置压缩，较短的一侧用 `fill` 补齐。
///
/// # Examples
///
/// ```
/// use az_array::slice::zip_longest;
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

/// 将切片原地左旋 `mid` 个位置。
///
/// 原索引 `mid` 处的元素会成为旋转后的第一个元素；`mid` 超过长度时按长度截断。
///
/// # Examples
///
/// ```
/// use az_array::slice::rotate_left;
///
/// let mut v = vec![1, 2, 3, 4, 5];
/// rotate_left(&mut v, 2);
/// assert_eq!(v, vec![3, 4, 5, 1, 2]);
/// ```
pub fn rotate_left<T>(slice: &mut [T], mid: usize) {
    let mid = mid.min(slice.len());
    slice.rotate_left(mid);
}

/// 将切片原地右旋 `mid` 个位置。
///
/// `mid` 超过长度时按长度截断，因此不会因为过大的旋转量 panic。
///
/// # Examples
///
/// ```
/// use az_array::slice::rotate_right;
///
/// let mut v = vec![1, 2, 3, 4, 5];
/// rotate_right(&mut v, 2);
/// assert_eq!(v, vec![4, 5, 1, 2, 3]);
/// ```
pub fn rotate_right<T>(slice: &mut [T], mid: usize) {
    let mid = mid.min(slice.len());
    slice.rotate_right(mid);
}

/// 返回由 `size` 个元素组成的滑动窗口。
///
/// `size` 为 0 或大于切片长度时返回空向量。
///
/// # Examples
///
/// ```
/// use az_array::slice::window;
///
/// assert_eq!(window(&[1, 2, 3, 4], 2), vec![vec![1, 2], vec![2, 3], vec![3, 4]]);
/// assert_eq!(window(&[1, 2], 3), Vec::<Vec<i32>>::new());
/// ```
#[must_use]
pub fn window<T: Clone>(slice: &[T], size: usize) -> Vec<Vec<T>> {
    if size == 0 || size > slice.len() {
        return Vec::new();
    }
    slice.windows(size).map(<[T]>::to_vec).collect()
}

/// 统计切片中每个元素出现的次数。
///
/// 返回的键是原切片中元素的引用，不会克隆元素本身。
///
/// # Examples
///
/// ```
/// use az_array::slice::frequencies;
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

/// 按谓词将切片划分为两个引用分组。
///
/// 返回值为 `(matching, not_matching)`，分别表示满足和不满足谓词的元素引用。
///
/// # Examples
///
/// ```
/// use az_array::slice::partition;
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

/// 在切片左侧补入 `fill`，直到结果长度达到 `len`。
///
/// 如果原切片长度已经不小于 `len`，则直接返回它的克隆。
///
/// # Examples
///
/// ```
/// use az_array::slice::pad_left;
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
