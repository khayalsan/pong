use std::net::IpAddr;
use std::num::NonZeroU32;
use std::time::Duration;
use governor::clock::DefaultClock;
use governor::{Quota, RateLimiter};
use governor::state::keyed::DefaultKeyedStateStore;

pub type IpRateLimiter = RateLimiter<
    IpAddr,
    DefaultKeyedStateStore<IpAddr>,
    DefaultClock
>;

pub fn create_rate_limiter() -> IpRateLimiter {
    let quota = Quota::per_minute(
        NonZeroU32::new(5).unwrap()
    );

    RateLimiter::keyed(quota)
}