use std::cmp::Ord;

/// Checks if a vector is sorted and has unique elements.
///
/// # Arguments
///
/// * `v` - A reference to a vector of elements that implement the `Ord` trait.
///
/// # Returns
///
/// Returns `true` if the vector is sorted and has unique elements, `false` otherwise.
pub fn is_sorted_and_unique<T: Ord>(v: &[T]) -> bool {
    if v.is_empty() {
        return true;
    }

    let mut prev = &v[0];
    for (i, item) in v.iter().enumerate() {
        if i == 0 {
            continue;
        }
        if prev >= item {
            return false;
        }
        prev = item;
    }
    true
}

/// Checks if a vector is sorted (but not necessarily unique).
///
/// # Arguments
///
/// * `v` - A reference to a vector of elements that implement the `Ord` trait.
///
/// # Returns
///
/// Returns `true` if the vector is sorted, `false` otherwise.
pub fn is_sorted<T: Ord>(v: &[T]) -> bool {
    if v.is_empty() {
        return true;
    }

    let mut prev = &v[0];
    for (i, item) in v.iter().enumerate() {
        if i == 0 {
            continue;
        }
        if prev > item {
            return false;
        }
        prev = item;
    }
    true
}

/// Compares two vectors for equality.
///
/// # Arguments
///
/// * `a` - A reference to the first vector.
/// * `b` - A reference to the second vector.
///
/// # Returns
///
/// Returns `true` if the vectors are equal, `false` otherwise.
pub fn eq_vectors<T: PartialEq>(a: &[T], b: &[T]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    for (i, item) in a.iter().enumerate() {
        if item != &b[i] {
            return false;
        }
    }
    true
}
