/// Leap second handling module for Hourcoin
///
/// Chrono does not natively handle leap seconds. This module provides
/// leap second awareness for accurate time-based consensus.
///
/// Leap seconds are occasional 1-second adjustments to UTC to account
/// for variations in Earth's rotation. Without proper handling, they can cause:
/// - Time to appear to go backwards
/// - Duplicate timestamps
/// - Consensus disagreements between nodes

use chrono::{DateTime, Utc, NaiveDateTime};
use std::sync::OnceLock;

/// Leap second offset data
/// Source: IERS Bulletin C (International Earth Rotation Service)
/// https://www.iers.org/IERS/EN/Publications/Bulletins/bulletins.html
#[derive(Debug, Clone)]
struct LeapSecond {
    /// Unix timestamp (in seconds) when leap second is introduced
    timestamp: i64,
    /// TAI - UTC offset after this leap second (in seconds)
    tai_offset: i32,
}

/// Leap second table (updated as of January 2025)
/// Format: (Unix timestamp, TAI-UTC offset)
static LEAP_SECONDS: OnceLock<Vec<LeapSecond>> = OnceLock::new();

fn get_leap_seconds() -> &'static Vec<LeapSecond> {
    LEAP_SECONDS.get_or_init(|| {
        vec![
            // Historical leap seconds since 1972
            LeapSecond { timestamp: 63072000, tai_offset: 10 },   // 1972-01-01
            LeapSecond { timestamp: 78796800, tai_offset: 11 },   // 1972-07-01
            LeapSecond { timestamp: 94694400, tai_offset: 12 },   // 1973-01-01
            LeapSecond { timestamp: 126230400, tai_offset: 13 },  // 1974-01-01
            LeapSecond { timestamp: 157766400, tai_offset: 14 },  // 1975-01-01
            LeapSecond { timestamp: 189302400, tai_offset: 15 },  // 1976-01-01
            LeapSecond { timestamp: 220924800, tai_offset: 16 },  // 1977-01-01
            LeapSecond { timestamp: 252460800, tai_offset: 17 },  // 1978-01-01
            LeapSecond { timestamp: 283996800, tai_offset: 18 },  // 1979-01-01
            LeapSecond { timestamp: 315532800, tai_offset: 19 },  // 1980-01-01
            LeapSecond { timestamp: 362793600, tai_offset: 20 },  // 1981-07-01
            LeapSecond { timestamp: 394329600, tai_offset: 21 },  // 1982-07-01
            LeapSecond { timestamp: 425865600, tai_offset: 22 },  // 1983-07-01
            LeapSecond { timestamp: 489024000, tai_offset: 23 },  // 1985-07-01
            LeapSecond { timestamp: 567993600, tai_offset: 24 },  // 1988-01-01
            LeapSecond { timestamp: 631152000, tai_offset: 25 },  // 1990-01-01
            LeapSecond { timestamp: 662688000, tai_offset: 26 },  // 1991-01-01
            LeapSecond { timestamp: 709948800, tai_offset: 27 },  // 1992-07-01
            LeapSecond { timestamp: 741484800, tai_offset: 28 },  // 1993-07-01
            LeapSecond { timestamp: 773020800, tai_offset: 29 },  // 1994-07-01
            LeapSecond { timestamp: 820454400, tai_offset: 30 },  // 1996-01-01
            LeapSecond { timestamp: 867715200, tai_offset: 31 },  // 1997-07-01
            LeapSecond { timestamp: 915148800, tai_offset: 32 },  // 1999-01-01
            LeapSecond { timestamp: 1136073600, tai_offset: 33 }, // 2006-01-01
            LeapSecond { timestamp: 1230768000, tai_offset: 34 }, // 2009-01-01
            LeapSecond { timestamp: 1341100800, tai_offset: 35 }, // 2012-07-01
            LeapSecond { timestamp: 1435708800, tai_offset: 36 }, // 2015-07-01
            LeapSecond { timestamp: 1483228800, tai_offset: 37 }, // 2017-01-01
            // Note: Update this table when new leap seconds are announced
            // Check IERS Bulletin C: https://www.iers.org/IERS/EN/Publications/Bulletins/bulletins.html
        ]
    })
}

/// Get the TAI-UTC offset for a given Unix timestamp
fn get_tai_offset(unix_seconds: i64) -> i32 {
    let leap_seconds = get_leap_seconds();

    // Find the most recent leap second before this timestamp
    let mut offset = 10; // Initial offset in 1972

    for leap in leap_seconds {
        if unix_seconds >= leap.timestamp {
            offset = leap.tai_offset;
        } else {
            break;
        }
    }

    offset
}

/// Convert UTC timestamp to TAI (International Atomic Time)
/// TAI does not have leap seconds and is monotonically increasing
pub fn utc_to_tai_millis(utc_millis: i64) -> i64 {
    let utc_seconds = utc_millis / 1000;
    let offset = get_tai_offset(utc_seconds);
    utc_millis + (offset as i64 * 1000)
}

/// Convert TAI timestamp to UTC
pub fn tai_to_utc_millis(tai_millis: i64) -> i64 {
    // Approximate UTC seconds for lookup
    let approx_utc_seconds = (tai_millis / 1000) - 37; // 37 is current max offset
    let offset = get_tai_offset(approx_utc_seconds);
    tai_millis - (offset as i64 * 1000)
}

/// Get current time in TAI milliseconds
/// TAI is monotonically increasing and has no leap seconds
pub fn now_tai_millis() -> i64 {
    let utc_millis = Utc::now().timestamp_millis();
    utc_to_tai_millis(utc_millis)
}

/// Check if a timestamp is near a leap second boundary
/// Returns true if within 1 second of a known leap second
pub fn is_near_leap_second(utc_millis: i64) -> bool {
    let utc_seconds = utc_millis / 1000;
    let leap_seconds = get_leap_seconds();

    for leap in leap_seconds {
        let diff = (utc_seconds - leap.timestamp).abs();
        if diff <= 1 {
            return true;
        }
    }

    false
}

/// Get information about the next scheduled leap second
/// Returns None if no leap second is scheduled
pub fn next_leap_second() -> Option<DateTime<Utc>> {
    // In practice, this would query IERS Bulletin C for announcements
    // Leap seconds are announced at least 8 weeks in advance
    // For now, return None as no future leap seconds are in the table
    None
}

/// Validate that a time difference is monotonically increasing
/// accounting for potential leap seconds
pub fn validate_time_ordering(prev_tai_millis: i64, curr_tai_millis: i64) -> bool {
    curr_tai_millis > prev_tai_millis
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_tai_offset() {
        // Before first leap second (1972)
        assert_eq!(get_tai_offset(0), 10);

        // After 1972-01-01 leap second
        assert_eq!(get_tai_offset(63072001), 10);

        // After 2017-01-01 leap second (most recent)
        assert_eq!(get_tai_offset(1483228801), 37);

        // Current time should have offset 37
        let now = Utc::now().timestamp();
        assert_eq!(get_tai_offset(now), 37);
    }

    #[test]
    fn test_utc_to_tai_conversion() {
        // 2020-01-01 00:00:00 UTC (after 2017 leap second)
        let utc_millis = 1577836800000_i64;
        let tai_millis = utc_to_tai_millis(utc_millis);

        // Should add 37 seconds
        assert_eq!(tai_millis, utc_millis + 37000);
    }

    #[test]
    fn test_tai_to_utc_conversion() {
        // TAI milliseconds
        let tai_millis = 1577836837000_i64;
        let utc_millis = tai_to_utc_millis(tai_millis);

        // Should subtract 37 seconds
        assert_eq!(utc_millis, tai_millis - 37000);
    }

    #[test]
    fn test_round_trip_conversion() {
        let utc_millis = 1577836800000_i64;
        let tai_millis = utc_to_tai_millis(utc_millis);
        let back_to_utc = tai_to_utc_millis(tai_millis);

        // Should be identical after round trip
        assert_eq!(utc_millis, back_to_utc);
    }

    #[test]
    fn test_now_tai_millis() {
        let tai_now = now_tai_millis();
        let utc_now = Utc::now().timestamp_millis();

        // TAI should be about 37 seconds ahead of UTC
        let diff_seconds = (tai_now - utc_now) / 1000;
        assert_eq!(diff_seconds, 37);
    }

    #[test]
    fn test_is_near_leap_second() {
        // 2017-01-01 00:00:00 UTC (leap second boundary)
        let leap_second = 1483228800000_i64;
        assert!(is_near_leap_second(leap_second));

        // One second before
        assert!(is_near_leap_second(leap_second - 1000));

        // One second after
        assert!(is_near_leap_second(leap_second + 1000));

        // Two seconds away (should be false)
        assert!(!is_near_leap_second(leap_second + 2000));

        // Random time not near leap second
        assert!(!is_near_leap_second(1577836800000));
    }

    #[test]
    fn test_validate_time_ordering() {
        let time1 = now_tai_millis();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let time2 = now_tai_millis();

        // Later time should be greater
        assert!(validate_time_ordering(time1, time2));

        // Same time should fail
        assert!(!validate_time_ordering(time1, time1));

        // Earlier time should fail
        assert!(!validate_time_ordering(time2, time1));
    }

    #[test]
    fn test_tai_is_monotonic_during_leap_second() {
        // Simulate timestamps around 2017 leap second
        let before_leap = 1483228799000_i64; // 2016-12-31 23:59:59 UTC (offset = 36)
        let at_leap = 1483228800000_i64;     // 2017-01-01 00:00:00 UTC (offset = 37)
        let after_leap = 1483228801000_i64;  // 2017-01-01 00:00:01 UTC (offset = 37)

        let tai_before = utc_to_tai_millis(before_leap);
        let tai_at = utc_to_tai_millis(at_leap);
        let tai_after = utc_to_tai_millis(after_leap);

        // TAI should be strictly increasing
        assert!(tai_at > tai_before);
        assert!(tai_after > tai_at);

        // During leap second boundary, TAI advances by 2 seconds while UTC advances by 1
        // This is because the TAI-UTC offset increases by 1
        // before: UTC + 36s, at: UTC + 37s
        // Difference: (at_utc + 37s) - (before_utc + 36s) = 1s + 1s = 2s
        assert_eq!(tai_at - tai_before, 2000); // 2 seconds (leap second!)

        // After the leap second, normal 1-second progression
        assert_eq!(tai_after - tai_at, 1000);  // 1 second
    }

    #[test]
    fn test_leap_second_table_is_sorted() {
        let leap_seconds = get_leap_seconds();

        for i in 1..leap_seconds.len() {
            // Timestamps should be increasing
            assert!(leap_seconds[i].timestamp > leap_seconds[i-1].timestamp);

            // Offsets should be increasing by 1
            assert_eq!(leap_seconds[i].tai_offset, leap_seconds[i-1].tai_offset + 1);
        }
    }
}
