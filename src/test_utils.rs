//! This module provides utility function for tests.

use std::path::PathBuf;

/// Returns the path to the test resources.
pub fn test_resources() -> PathBuf {
    "./test_resources".into()
}