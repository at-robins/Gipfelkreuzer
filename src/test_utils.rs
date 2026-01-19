//! This module provides utility function for tests.

use std::path::PathBuf;

/// Returns the path to the test resources.
pub fn test_resources() -> PathBuf {
    "./test_resources".into()
}

/// Returns the directory to use for test output.
pub fn test_output() -> PathBuf {
    let mut path = test_resources();
    path.push("tmp");
    path
}