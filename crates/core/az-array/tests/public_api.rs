use az_array::api::{
    chunk, flatten_nested, frequencies, pad_left, partition, rotate_left, rotate_right, unique,
    window, zip_longest,
};

#[test]
fn chunk_splits_slices_by_requested_size() {
    assert_eq!(
        chunk(&[1, 2, 3, 4, 5], 2),
        vec![vec![1, 2], vec![3, 4], vec![5]]
    );
    assert_eq!(chunk(&[1, 2, 3], 2), vec![vec![1, 2], vec![3]]);
    assert!(chunk(&[1, 2, 3], 0).is_empty());
    assert!(chunk::<i32>(&[], 3).is_empty());
}

#[test]
fn unique_preserves_first_seen_order() {
    assert_eq!(unique(&[1, 2, 3, 2, 1]), vec![1, 2, 3]);
    assert_eq!(unique(&[1, 1, 1]), vec![1]);
    assert_eq!(unique::<i32>(&[]), vec![]);
}

#[test]
fn flatten_nested_concatenates_vectors() {
    assert_eq!(
        flatten_nested(&[vec![1, 2], vec![3], vec![4, 5]]),
        vec![1, 2, 3, 4, 5]
    );
    assert_eq!(flatten_nested::<i32>(&[]), vec![]);
}

#[test]
fn zip_longest_pads_shorter_side_with_fill_value() {
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
fn rotate_helpers_apply_in_place_rotation() {
    let mut left = vec![1, 2, 3, 4, 5];
    rotate_left(&mut left, 2);
    assert_eq!(left, vec![3, 4, 5, 1, 2]);

    let mut right = vec![1, 2, 3, 4, 5];
    rotate_right(&mut right, 2);
    assert_eq!(right, vec![4, 5, 1, 2, 3]);
}

#[test]
fn window_returns_fixed_width_slices() {
    assert_eq!(
        window(&[1, 2, 3, 4], 2),
        vec![vec![1, 2], vec![2, 3], vec![3, 4]]
    );
    assert!(window(&[1, 2], 3).is_empty());
    assert!(window(&[1, 2, 3], 0).is_empty());
}

#[test]
fn frequencies_counts_each_element_reference() {
    let freq = frequencies(&['a', 'b', 'a', 'c', 'b', 'a']);
    assert_eq!(freq[&'a'], 3);
    assert_eq!(freq[&'b'], 2);
    assert_eq!(freq[&'c'], 1);
}

#[test]
fn partition_splits_items_by_predicate() {
    let (evens, odds) = partition(&[1, 2, 3, 4, 5], |x| x % 2 == 0);
    assert_eq!(evens, vec![&2, &4]);
    assert_eq!(odds, vec![&1, &3, &5]);

    let (pass, fail) = partition(&[2, 4, 6], |x| x % 2 == 0);
    assert_eq!(pass, vec![&2, &4, &6]);
    assert!(fail.is_empty());
}

#[test]
fn pad_left_adds_fill_until_target_length() {
    assert_eq!(pad_left(&[3, 4], 5, 0), vec![0, 0, 0, 3, 4]);
    assert_eq!(pad_left(&[1, 2, 3], 2, 0), vec![1, 2, 3]);
    assert_eq!(pad_left(&[1, 2], 2, 0), vec![1, 2]);
}
