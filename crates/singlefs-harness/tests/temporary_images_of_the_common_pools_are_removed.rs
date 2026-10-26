//! 共用搭建模块（`tests/common/mod.rs`）建的镜像文件跑完就删：池照常丢掉时删，搭池途中 panic 时由认领镜像的守卫删
//! （`.claude/singlefs-ai-sop/rules/command-safety.md`「测试镜像一律放临时目录」）。

mod common;

use std::path::PathBuf;

use common::{format_pool, formatted_recorded_images};

/// 场景：只做过 mkfs 的池（两块盘的镜像文件）照常丢掉。预期：丢掉之前两份镜像都在，丢掉之后都不在。
/// 判别力（`crates/mutations.tsv`）：池的 Drop 不删镜像，两份都还在，这条红。
#[test]
fn the_image_files_of_a_formatted_pool_are_gone_once_the_pool_is_dropped() {
    let formatted = format_pool("temporary-images-dropped");
    let paths = formatted.paths.clone();
    assert_eq!(paths.len(), 2, "两块盘各一份镜像");
    for path in &paths {
        assert!(path.exists(), "池在时镜像 {} 在", path.display());
    }
    drop(formatted);
    for path in &paths {
        assert!(!path.exists(), "池丢掉之后镜像 {} 不在了", path.display());
    }
}

/// 场景：两块盘的镜像建好、mkfs 做完，池还没搭成就 panic（搭池途中某一步的 `expect` 失败那一类）。
/// 预期：panic 接住之后两份镜像都不在了。
/// 判别力（`crates/mutations.tsv`）：认领镜像的守卫不删，两份都还在，这条红。
#[test]
fn the_image_files_are_gone_after_a_panic_before_the_pool_is_built() {
    // panic 之前看到的镜像路径与那一刻在不在：断言放在 catch_unwind 外面，里面的断言失败会被接住的 panic 吞掉
    let mut images_seen_before_the_panic: Vec<(PathBuf, bool)> = Vec::new();
    let panicked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let (images, _devices, _stream, _genesis, _mkfs_operation_count) =
            formatted_recorded_images("temporary-images-panic");
        images_seen_before_the_panic = images
            .iter()
            .map(|image| (image.to_path_buf(), image.exists()))
            .collect();
        panic!("池搭成之前造的 panic");
    }));
    assert!(panicked.is_err(), "搭池途中真 panic 了");
    assert_eq!(images_seen_before_the_panic.len(), 2, "两块盘各一份镜像");
    for (path, existed_before_the_panic) in &images_seen_before_the_panic {
        assert!(
            *existed_before_the_panic,
            "panic 之前镜像 {} 在",
            path.display()
        );
        assert!(!path.exists(), "panic 之后镜像 {} 不在了", path.display());
    }
}
