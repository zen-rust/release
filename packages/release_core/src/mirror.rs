//! Helpers for the optional crates.io mirror publish (Phase 3).
//!
//! These are pure building blocks — rate-safe batching and rate-limit detection —
//! consumed by the mirror publish orchestration once it is wired into the release flow.
//! Kept separate so they can be unit-tested without a real registry.

/// Split `items` into batches of at most `batch_size`, preserving order.
///
/// A `batch_size` of `0` means "no batching": all items are returned as a single
/// batch (or no batches at all when `items` is empty).
pub fn into_batches<T>(items: Vec<T>, batch_size: usize) -> Vec<Vec<T>> {
  if batch_size == 0 {
    return if items.is_empty() {
      vec![]
    } else {
      vec![items]
    };
  }
  let mut batches = Vec::new();
  let mut current = Vec::new();
  for item in items {
    current.push(item);
    if current.len() == batch_size {
      batches.push(std::mem::take(&mut current));
    }
  }
  if !current.is_empty() {
    batches.push(current);
  }
  batches
}

/// Detect a registry rate-limit (HTTP 429) from `cargo publish` output.
///
/// crates.io surfaces its publish burst limit as a 429; the message wording varies,
/// so we match the status code and the common phrasings.
pub fn is_rate_limited(output: &str) -> bool {
  let s = output.to_ascii_lowercase();
  s.contains("429") || s.contains("too many requests") || s.contains("rate limit")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn batches_preserve_order_and_size() {
    let items = vec![1, 2, 3, 4, 5];
    let batches = into_batches(items, 2);
    assert_eq!(batches, vec![vec![1, 2], vec![3, 4], vec![5]]);
  }

  #[test]
  fn batch_size_dividing_evenly() {
    assert_eq!(
      into_batches(vec![1, 2, 3, 4], 2),
      vec![vec![1, 2], vec![3, 4]]
    );
  }

  #[test]
  fn batch_size_larger_than_len_is_single_batch() {
    assert_eq!(into_batches(vec![1, 2], 10), vec![vec![1, 2]]);
  }

  #[test]
  fn batch_size_zero_is_no_batching() {
    assert_eq!(into_batches(vec![1, 2, 3], 0), vec![vec![1, 2, 3]]);
    assert_eq!(into_batches(Vec::<i32>::new(), 0), Vec::<Vec<i32>>::new());
  }

  #[test]
  fn empty_input_yields_no_batches() {
    assert_eq!(into_batches(Vec::<i32>::new(), 3), Vec::<Vec<i32>>::new());
  }

  #[test]
  fn detects_rate_limit_phrasings() {
    assert!(is_rate_limited(
      "error: failed to get a 200 OK response, got 429"
    ));
    assert!(is_rate_limited("429 Too Many Requests"));
    assert!(is_rate_limited(
      "You have hit the rate limit for publishing"
    ));
    assert!(!is_rate_limited("error: failed to verify package tarball"));
    assert!(!is_rate_limited("Uploading foo v0.0.1"));
  }
}
