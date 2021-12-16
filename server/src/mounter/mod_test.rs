#[cfg(test)]
use super::*;

#[tokio::test]
async fn mounter_test() {
    let mounter = Builder::new()
        .commit();

    mounter.mount_as_label("/dev/vdb1").unwrap();
    println!("donw");
}
