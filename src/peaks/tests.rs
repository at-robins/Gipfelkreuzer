use super::*;

#[test]
fn test_is_continuous_range() {
    let a_start: u64 = 42;
    let a_end: u64 = a_start + 42;
    // B before A.
    {
        let b_start: u64 = a_start - 20;
        let b_end: u64 = a_start - 10;
        assert!(!is_continuous_range(a_start, a_end, b_start, b_end));
    }

    // B adjacent to A on the left side.
    {
        let b_start: u64 = a_start - 20;
        let b_end: u64 = a_start - 1;
        assert!(is_continuous_range(a_start, a_end, b_start, b_end));
    }

    // B is overlapping A by 1 base (same start / end) on the left side.
    {
        let b_start: u64 = a_start - 20;
        let b_end: u64 = a_start;
        assert!(is_continuous_range(a_start, a_end, b_start, b_end));
    }

    // B is overlapping A by multiple bases on the left side.
    {
        let b_start: u64 = a_start - 20;
        let b_end: u64 = a_start + 5;
        assert!(is_continuous_range(a_start, a_end, b_start, b_end));
    }

    // B identical to A.
    {
        let b_start: u64 = a_start;
        let b_end: u64 = a_end;
        assert!(is_continuous_range(a_start, a_end, b_start, b_end));
    }

    // B is overlapping A by multiple bases on the right side.
    {
        let b_start: u64 = a_end - 5;
        let b_end: u64 = a_end + 20;
        assert!(is_continuous_range(a_start, a_end, b_start, b_end));
    }

    // B is overlapping A by 1 base (same start / end) on the right side.
    {
        let b_start: u64 = a_end;
        let b_end: u64 = a_end + 20;
        assert!(is_continuous_range(a_start, a_end, b_start, b_end));
    }

    // B is overlapping A by 1 base (same start / end) on the right side.
    {
        let b_start: u64 = a_end;
        let b_end: u64 = a_end + 20;
        assert!(is_continuous_range(a_start, a_end, b_start, b_end));
    }

    // B adjacent to A on the right side.
    {
        let b_start: u64 = a_end + 1;
        let b_end: u64 = a_end + 20;
        assert!(is_continuous_range(a_start, a_end, b_start, b_end));
    }

    // B after A.
    {
        let b_start: u64 = a_end + 5;
        let b_end: u64 = a_end + 20;
        assert!(!is_continuous_range(a_start, a_end, b_start, b_end));
    }
}

#[test]
fn test_is_continuous_range_points() {
    let a_start: u64 = 42;
    let a_end: u64 = a_start;
    // B before A.
    {
        let b_start: u64 = a_start - 20;
        let b_end: u64 = b_start;
        assert!(!is_continuous_range(a_start, a_end, b_start, b_end));
    }

    // B adjacent to A on the left side.
    {
        let b_start: u64 = a_start - 1;
        let b_end: u64 = b_start;
        assert!(is_continuous_range(a_start, a_end, b_start, b_end));
    }

    // B identical to A.
    {
        let b_start: u64 = a_start;
        let b_end: u64 = b_start;
        assert!(is_continuous_range(a_start, a_end, b_start, b_end));
    }

    // B adjacent to A on the right side.
    {
        let b_start: u64 = a_end + 1;
        let b_end: u64 = b_start;
        assert!(is_continuous_range(a_start, a_end, b_start, b_end));
    }

    // B after A.
    {
        let b_start: u64 = a_end + 20;
        let b_end: u64 = b_start;
        assert!(!is_continuous_range(a_start, a_end, b_start, b_end));
    }
}

#[test]
#[should_panic]
fn test_is_continuous_range_a_invalid() {
    let a_start: u64 = 42;
    let a_end: u64 = a_start - 20;
    let b_start: u64 = a_start - 20;
    let b_end: u64 = a_start - 10;
    is_continuous_range(a_start, a_end, b_start, b_end);
}

#[test]
#[should_panic]
fn test_is_continuous_range_b_invalid() {
    let a_start: u64 = 42;
    let a_end: u64 = a_start + 42;
    let b_start: u64 = a_start - 20;
    let b_end: u64 = a_start - 30;
    is_continuous_range(a_start, a_end, b_start, b_end);
}

#[test]
fn test_peak_data_new() {
    let id: usize = 42;
    let start: u64 = 2004402;
    let end: u64 = 5090960056;
    let summit: u64 = 48946040;

    let peak = PeakData::new(id, start, end, summit).unwrap();
    assert_eq!(peak.id(), id);
    assert_eq!(peak.start(), start);
    assert_eq!(peak.end(), end);
    assert_eq!(peak.summit(), summit);
}

#[test]
fn test_peak_data_new_start_end_summit_equal() {
    let id: usize = 42;
    let start: u64 = 2004402;
    let end: u64 = 2004402;
    let summit: u64 = 2004402;

    let peak = PeakData::new(id, start, end, summit).unwrap();
    assert_eq!(peak.start(), start);
    assert_eq!(peak.end(), end);
    assert_eq!(peak.summit(), summit);
}

#[test]
fn test_peak_data_new_start_after_end() {
    let id: usize = 42;
    let start: u64 = 5090960056;
    let end: u64 = 2004402;
    let summit: u64 = 48946040;

    assert!(PeakData::new(id, start, end, summit).is_err());
}

#[test]
fn test_peak_data_new_summit_before_start() {
    let id: usize = 42;
    let start: u64 = 2004402;
    let end: u64 = 5090960056;
    let summit: u64 = start - 4435;

    assert!(PeakData::new(id, start, end, summit).is_err());
}

#[test]
fn test_peak_data_new_summit_after_end() {
    let id: usize = 42;
    let start: u64 = 2004402;
    let end: u64 = 5090960056;
    let summit: u64 = end + 4435;

    assert!(PeakData::new(id, start, end, summit).is_err());
}

#[test]
fn test_peak_data_length() {
    let id: usize = 42;
    let start: u64 = 20;
    let end: u64 = 40;
    let summit: u64 = 38;

    let peak = PeakData::new(id, start, end, summit).unwrap();
    assert_eq!(peak.length(), 21);
}

#[test]
fn test_peak_bin_new() {
    let id: usize = 42;
    let start: u64 = 2004402;
    let end: u64 = 5090960056;
    let summit: u64 = 48946040;

    let peak = PeakData::new(id, start, end, summit).unwrap();
    let peak_bin = PeakBin::new(peak);
    assert_eq!(peak_bin.start(), start);
    assert_eq!(peak_bin.end(), end);
    assert_eq!(peak_bin.peaks(), &vec![peak]);
}

#[test]
fn test_peak_bin_insert() {
    let peaks = vec![
        PeakData::new(0, 12u64, 24u64, 18u64).unwrap(),
        PeakData::new(1, 11u64, 21u64, 17u64).unwrap(),
        PeakData::new(2, 23u64, 26u64, 24u64).unwrap(),
        PeakData::new(3, 27u64, 29u64, 27u64).unwrap(),
    ];

    let mut peak_bin = PeakBin::new(peaks[0]);
    peak_bin.insert(peaks[1]);
    assert_eq!(peak_bin.start(), peaks[0..=1].iter().map(PeakData::start).min().unwrap());
    assert_eq!(peak_bin.end(), peaks[0..=1].iter().map(PeakData::end).max().unwrap());
    assert_eq!(peak_bin.peaks(), &peaks[0..=1]);

    peak_bin.insert(peaks[2]);
    assert_eq!(peak_bin.start(), peaks[0..=2].iter().map(PeakData::start).min().unwrap());
    assert_eq!(peak_bin.end(), peaks[0..=2].iter().map(PeakData::end).max().unwrap());
    assert_eq!(peak_bin.peaks(), &peaks[0..=2]);

    peak_bin.insert(peaks[3]);
    assert_eq!(peak_bin.start(), peaks.iter().map(PeakData::start).min().unwrap());
    assert_eq!(peak_bin.end(), peaks.iter().map(PeakData::end).max().unwrap());
    assert_eq!(peak_bin.peaks(), &peaks);
}

#[test]
fn test_peak_bin_try_insert() {
    let peaks = vec![
        PeakData::new(0, 12u64, 22u64, 18u64).unwrap(),
        PeakData::new(1, 11u64, 21u64, 17u64).unwrap(),
        PeakData::new(2, 23u64, 26u64, 24u64).unwrap(),
        PeakData::new(3, 27u64, 29u64, 27u64).unwrap(),
        PeakData::new(3, 270u64, 290u64, 277u64).unwrap(),
    ];

    let mut peak_bin = PeakBin::new(peaks[0]);
    assert!(peak_bin.try_insert(peaks[1]).is_none());
    assert_eq!(peak_bin.start(), peaks[0..=1].iter().map(PeakData::start).min().unwrap());
    assert_eq!(peak_bin.end(), peaks[0..=1].iter().map(PeakData::end).max().unwrap());
    assert_eq!(peak_bin.peaks(), &peaks[0..=1]);

    assert!(peak_bin.try_insert(peaks[2]).is_none());
    assert_eq!(peak_bin.start(), peaks[0..=2].iter().map(PeakData::start).min().unwrap());
    assert_eq!(peak_bin.end(), peaks[0..=2].iter().map(PeakData::end).max().unwrap());
    assert_eq!(peak_bin.peaks(), &peaks[0..=2]);

    assert!(peak_bin.try_insert(peaks[3]).is_none());
    assert_eq!(peak_bin.start(), peaks[0..=3].iter().map(PeakData::start).min().unwrap());
    assert_eq!(peak_bin.end(), peaks[0..=3].iter().map(PeakData::end).max().unwrap());
    assert_eq!(peak_bin.peaks(), &peaks[0..=3]);

    assert_eq!(peak_bin.try_insert(peaks[4]), Some(peaks[4]));
    assert_eq!(peak_bin.start(), peaks[0..=3].iter().map(PeakData::start).min().unwrap());
    assert_eq!(peak_bin.end(), peaks[0..=3].iter().map(PeakData::end).max().unwrap());
    assert_eq!(peak_bin.peaks(), &peaks[0..=3]);
}

#[test]
fn test_peak_bin_into_peak_vec() {
    let peaks = vec![
        PeakData::new(0, 12u64, 22u64, 18u64).unwrap(),
        PeakData::new(1, 11u64, 21u64, 17u64).unwrap(),
        PeakData::new(2, 23u64, 26u64, 24u64).unwrap(),
        PeakData::new(3, 27u64, 29u64, 27u64).unwrap(),
    ];

    let mut peak_bin = PeakBin::new(peaks[0]);
    assert!(peak_bin.try_insert(peaks[1]).is_none());
    assert!(peak_bin.try_insert(peaks[2]).is_none());
    assert!(peak_bin.try_insert(peaks[3]).is_none());

    let peaks_in_bin: Vec<PeakData> = peak_bin.into();
    assert_eq!(peaks_in_bin, peaks);
}

#[test]
fn test_peak_bin_bin_peaks() {
    let peaks = vec![
        PeakData::new(0, 12u64, 22u64, 18u64).unwrap(),
        PeakData::new(1, 11u64, 21u64, 17u64).unwrap(),
        PeakData::new(2, 23u64, 26u64, 24u64).unwrap(),
        PeakData::new(3, 27u64, 29u64, 27u64).unwrap(),
        PeakData::new(4, 270u64, 290u64, 277u64).unwrap(),
        PeakData::new(5, 271u64, 291u64, 276u64).unwrap(),
        PeakData::new(6, 2700u64, 2900u64, 2770u64).unwrap(),
    ];

    let bins = PeakBin::bin_peaks(peaks.clone());

    assert_eq!(bins.len(), 3);

    assert_eq!(bins[0].peaks().len(), 4);
    for peak in bins[0].peaks() {
        assert!(&peaks[0..=3].contains(peak));
    }

    assert_eq!(bins[1].peaks().len(), 2);
    for peak in bins[1].peaks() {
        assert!(&peaks[4..=5].contains(peak));
    }

    assert_eq!(bins[2].peaks().len(), 1);
    for peak in bins[2].peaks() {
        assert!(&peaks[6..].contains(peak));
    }
}
