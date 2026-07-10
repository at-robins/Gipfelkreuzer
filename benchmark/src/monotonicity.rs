//! This module calculates monotonicity metrics.

/// A peak with counts per base position.
pub struct PeakCounts {
    base_counts: Vec<u64>,
    total_counts: u64,
}

impl PeakCounts {
    pub fn new(counts: Vec<u64>) -> Result<Self, String> {
        if counts.is_empty() {
            Err("Cannot create  a peak from an empty count vector.".to_string())
        } else {
            let total_counts = counts.iter().sum();
            Ok(Self {
                base_counts: counts,
                total_counts,
            })
        }
    }

    /// Returns the difference between counts at index A and B divided by the total counts.
    fn relative_difference_at_indices(&self, index_a: usize, index_b: usize) -> f64 {
        ((self.base_counts[index_a] as f64) - (self.base_counts[index_b] as f64))
            / (self.total_counts as f64)
    }

    /// Returns the relative monotonicity at each given base position.
    fn monotonicity(&self) -> Vec<f64> {
        let max_value = self
            .base_counts
            .iter()
            .max()
            .expect("There must be a maximum value as the vector cannot be empty here.");
        let max_value_index = self
            .base_counts
            .iter()
            .position(|counts| counts == max_value)
            .expect(
                "There must be an index corresponding to the \
             maximum value as the vector cannot be empty here.",
            );
        let mut monotonicity = Vec::with_capacity(self.base_counts.len());
        for (count_index, _) in self.base_counts.iter().enumerate() {
            if count_index == max_value_index {
                monotonicity.push(0.0);
            } else if count_index < max_value_index {
                monotonicity
                    .push(self.relative_difference_at_indices(count_index, count_index + 1));
            } else {
                monotonicity
                    .push(self.relative_difference_at_indices(count_index, count_index - 1));
            }
        }
        monotonicity
    }

    /// Returns how much the peak count distribution deviates from a global maximum that decreases monoton at both flanks.
    pub fn monotonicity_deviation(&self) -> f64 {
        self.monotonicity()
            .into_iter()
            .filter(|value| *value > 0.0)
            .sum()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_peak_counts_new() {
        let counts = vec![0, 0, 1, 2, 1, 3, 6, 5, 4, 3, 0];
        let total_counts = counts.iter().sum();
        let peak_counts = PeakCounts::new(counts.clone()).unwrap();
        assert_eq!(peak_counts.base_counts, counts);
        assert_eq!(peak_counts.total_counts, total_counts);
    }

    #[test]
    fn test_peak_counts_new_empty() {
        assert!(PeakCounts::new(Vec::new()).is_err());
    }

    #[test]
    fn test_peak_counts_relative_difference_at_indices() {
        let counts = vec![0, 0, 1, 2, 1, 3, 6, 4, 4, 3, 1];
        let peak_counts = PeakCounts::new(counts).unwrap();
        assert_eq!(peak_counts.relative_difference_at_indices(3, 4), 0.04);
        assert_eq!(peak_counts.relative_difference_at_indices(7, 6), -0.08);
    }

    #[test]
    fn test_peak_counts_monotonicity() {
        let counts = vec![0, 0, 1, 2, 1, 3, 6, 4, 4, 3, 1];
        let total_counts: u64 = counts.iter().sum();
        let monotonicity_differences = vec![0, -1, -1, 1, -2, -3, 0, -2, 0, -1, -2];
        let monotonicity: Vec<f64> = monotonicity_differences
            .iter()
            .map(|value| (*value as f64) / (total_counts as f64))
            .collect();
        let peak_counts = PeakCounts::new(counts).unwrap();
        assert_eq!(peak_counts.monotonicity(), monotonicity);
    }

    #[test]
    fn test_monotonicity_deviation() {
        let peak_low_deviation = PeakCounts::new(vec![0, 0, 1, 2, 1, 3, 6, 5, 4, 3, 0]).unwrap();
        let peak_high_deviation =
            PeakCounts::new(vec![0, 1, 2, 3, 4, 1, 0, 2, 4, 6, 4, 3, 2, 0]).unwrap();
        assert!(
            peak_low_deviation.monotonicity_deviation()
                < peak_high_deviation.monotonicity_deviation()
        );
    }
}
