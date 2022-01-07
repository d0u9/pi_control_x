#[cfg(test)]
use std::time::Duration;
use test_log::test;
use tokio::time;

#[cfg(test)]
use super::router::*;
use super::wire::*;

#[test(tokio::test)]
async fn router_create_test() {
    let (epa0, epa1) = Wire::endpoints::<u32>();
    let (epb0, epb1) = Wire::endpoints::<u32>();

    let router = Router::new(epa0, epb0);

    time::sleep(Duration::from_millis(10)).await;
}
