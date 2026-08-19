use std::time::Duration;

use actix_web::http::Method;
use gathr_api::ratelimit::{quota_for, Quota, RateLimiter};

const TIGHT: Quota = Quota {
    bucket: "test",
    allowance: 3,
    window: Duration::from_secs(60),
};

#[test]
fn a_caller_is_admitted_up_to_the_allowance_and_then_told_to_wait() {
    let limiter = RateLimiter::default();

    for attempt in 1..=TIGHT.allowance {
        assert!(
            limiter.admit("198.51.100.7", TIGHT).is_ok(),
            "attempt {attempt} is inside the allowance"
        );
    }

    let refused = limiter.admit("198.51.100.7", TIGHT);
    assert!(refused.is_err());
    assert!(
        refused.unwrap_err() <= TIGHT.window,
        "the wait we advertise cannot exceed the window itself"
    );
}

#[test]
fn one_noisy_caller_does_not_spend_another_callers_allowance() {
    let limiter = RateLimiter::default();

    for _ in 0..TIGHT.allowance {
        let _ = limiter.admit("198.51.100.7", TIGHT);
    }

    assert!(
        limiter.admit("203.0.113.9", TIGHT).is_ok(),
        "buckets are per caller, not global"
    );
}

