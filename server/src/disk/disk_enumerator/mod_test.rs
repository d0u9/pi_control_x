#[cfg(test)]
use super::*;

#[tokio::test]
async fn disk_enumerator_test() {
    let enumerator = Builder::new().mount_point_prefix("/mnt").commit();

    let all_mounts = enumerator.get();
    dbg!(all_mounts);
}
