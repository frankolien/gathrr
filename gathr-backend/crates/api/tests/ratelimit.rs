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

#[test]
fn buckets_do_not_borrow_from_each_other() {
    let limiter = RateLimiter::default();
    let other = Quota {
        bucket: "other",
        ..TIGHT
    };

    for _ in 0..TIGHT.allowance {
        let _ = limiter.admit("198.51.100.7", TIGHT);
    }

    assert!(
        limiter.admit("198.51.100.7", other).is_ok(),
        "exhausting one route class must not lock a caller out of the rest of the api"
    );
}

#[test]
fn the_tightest_quotas_guard_code_issuing_and_invite_guessing() {
    let verification = quota_for(&Method::POST, "/v1/auth/otp/request");
    let invites = quota_for(&Method::GET, "/v1/invites/ABCDEFGHJK");
    let reads = quota_for(&Method::GET, "/v1/events");

    assert_eq!(verification.bucket, "verification");
    assert_eq!(invites.bucket, "invite_lookup");
    assert!(
        verification.allowance < invites.allowance,
        "issuing codes must be scarcer than looking up invites"
    );
    assert!(
        invites.allowance < reads.allowance,
        "invite lookups are the enumeration surface and must be tighter than ordinary reads"
    );
}

#[test]
fn writes_are_scarcer_than_reads_on_the_same_path() {
    assert!(
        quota_for(&Method::POST, "/v1/events/x/messages").allowance
            < quota_for(&Method::GET, "/v1/events/x/messages").allowance
    );
}
