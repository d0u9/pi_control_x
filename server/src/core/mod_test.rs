#[cfg(test)]
use super::*;

#[tokio::test]
async fn core_test() {
    let core = Core::new();
    core.enable_source("udev");
}

