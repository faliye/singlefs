//! checker 档模块：无（样本：真跑实现的可写挂载之后判一张镜像，带豁免标记）
// gpu-exempt: writable-mount
#[test]
fn the_writable_mount_is_judged_on_the_cpu() { assert!(check_pool_image(&mounted).is_empty()); }
