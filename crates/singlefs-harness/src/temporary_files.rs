//! 测试与装置在临时目录里建的文件与目录：交给守卫 [`TemporaryPathRemovedOnDrop`] 认领，守卫离开作用域时删掉——用例做完、断言失败、
//! panic 都删（`.claude/singlefs-ai-sop/rules/command-safety.md`「测试镜像一律放临时目录」，用户定案「自己用完自己删」）。
//! 要留现场看时显式设 [`KEEP_TEMPORARY_FILES_ENVIRONMENT_VARIABLE`]，留下的路径打到标准错误。
//! 守卫只在 Drop 里删：进程被 SIGKILL（超时、内存包装撞顶）时 Drop 不走，那一次的临时文件删不到。
//! harness 档的测试、共用搭建模块与 checker 档都经这一份认领，不各写一份删法。

use std::path::{Path, PathBuf};

/// 设成 `1` 时守卫不删、把留下的路径打到标准错误（要留现场看时显式打开）；没设、或设成别的值都照删。
/// 在 Drop 里读：读到认不得的值不 panic（panic 途中再 panic 会直接中止进程），当没设。
pub const KEEP_TEMPORARY_FILES_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_KEEP_TEST_TEMPORARY_FILES";

/// 守卫丢掉时临时路径怎么办。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TemporaryPathOnDrop {
    /// 删掉（默认）。
    Removed,
    /// 留着看现场，路径打到标准错误（[`KEEP_TEMPORARY_FILES_ENVIRONMENT_VARIABLE`] 设成 `1`）。
    KeptForInspection,
}

impl TemporaryPathOnDrop {
    /// [`KEEP_TEMPORARY_FILES_ENVIRONMENT_VARIABLE`] 的值（`None` 是没设）换成怎么办：只有 `1` 是留着。
    #[must_use]
    pub fn from_the_keep_switch_value(value: Option<&std::ffi::OsStr>) -> Self {
        if value.is_some_and(|text| text == "1") {
            Self::KeptForInspection
        } else {
            Self::Removed
        }
    }

    /// 读进程环境里的 [`KEEP_TEMPORARY_FILES_ENVIRONMENT_VARIABLE`]。
    #[must_use]
    pub fn from_the_environment() -> Self {
        Self::from_the_keep_switch_value(
            std::env::var_os(KEEP_TEMPORARY_FILES_ENVIRONMENT_VARIABLE).as_deref(),
        )
    }
}

/// 认领一条临时路径的守卫：丢掉时那条路径上有什么删什么（文件删文件，目录连同里面的一起删），路径不在就算了。
/// 建之前认领：建到一半 panic（写满盘、断言失败）留下的半份同样删掉。
pub struct TemporaryPathRemovedOnDrop {
    /// 交给别的主人之后（[`Self::into_path_handed_to_another_owner`]）是空路径，Drop 不碰它。
    path: PathBuf,
}

impl TemporaryPathRemovedOnDrop {
    /// 认领 `path`：它归这个守卫删，调用方不再另删。
    #[must_use]
    pub fn claiming(path: PathBuf) -> Self {
        Self { path }
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// 交出路径、这个守卫不再删：接手的主人（池的 Drop 这类）在它自己丢掉时经 [`remove_or_keep_the_temporary_path`] 删。
    #[must_use]
    pub fn into_path_handed_to_another_owner(mut self) -> PathBuf {
        std::mem::take(&mut self.path)
    }
}

/// 当路径用（`guard.join(…)`、交给收 `&Path` 的函数）：就是 [`TemporaryPathRemovedOnDrop::path`]。
impl std::ops::Deref for TemporaryPathRemovedOnDrop {
    type Target = Path;

    fn deref(&self) -> &Path {
        &self.path
    }
}

impl AsRef<Path> for TemporaryPathRemovedOnDrop {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}

impl Drop for TemporaryPathRemovedOnDrop {
    fn drop(&mut self) {
        if self.path.as_os_str().is_empty() {
            return;
        }
        remove_or_keep_the_temporary_path(&self.path, TemporaryPathOnDrop::from_the_environment());
    }
}

/// 照 `on_drop` 删掉或留着 `path`：留着时路径在就打一行到标准错误；删不掉只打一行，不 panic（这一份在 Drop 里调）。
pub fn remove_or_keep_the_temporary_path(path: &Path, on_drop: TemporaryPathOnDrop) {
    match on_drop {
        TemporaryPathOnDrop::Removed => remove_the_temporary_path(path),
        TemporaryPathOnDrop::KeptForInspection => {
            if std::fs::symlink_metadata(path).is_ok() {
                eprintln!(
                    "{KEEP_TEMPORARY_FILES_ENVIRONMENT_VARIABLE}=1：临时路径留着看现场：{}",
                    path.display()
                );
            }
        }
    }
}

/// 删 `path`：目录连同里面的一起删，别的（文件、符号链接本身）删它自己；不在不算错。
fn remove_the_temporary_path(path: &Path) {
    let removed = match std::fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return,
        Err(error) => Err(error),
        Ok(metadata) if metadata.is_dir() => std::fs::remove_dir_all(path),
        Ok(_not_a_directory) => std::fs::remove_file(path),
    };
    if let Err(error) = removed {
        if error.kind() != std::io::ErrorKind::NotFound {
            eprintln!("删不掉临时路径 {}：{error}", path.display());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_TEST_PATH_NUMBER: AtomicU64 = AtomicU64::new(0);

    fn fresh_temporary_path(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "singlefs-temporary-files-test-{label}-{}-{}",
            std::process::id(),
            NEXT_TEST_PATH_NUMBER.fetch_add(1, Ordering::SeqCst)
        ))
    }

    /// 场景：守卫认领一个文件与一个带子目录的目录，作用域照常走完。预期：两条路径都不在了。
    /// 判别力（`crates/mutations.tsv`）：删法里不删，两条都还在，这条红。
    #[test]
    fn a_claimed_file_and_a_claimed_directory_are_gone_once_the_scope_ends_normally() {
        let file_path = fresh_temporary_path("file");
        let directory_path = fresh_temporary_path("directory");
        {
            let file = TemporaryPathRemovedOnDrop::claiming(file_path.clone());
            std::fs::write(&*file, b"image bytes").expect("临时目录写得进文件");
            let directory = TemporaryPathRemovedOnDrop::claiming(directory_path.clone());
            std::fs::create_dir_all(directory.join("nested")).expect("临时目录建得了目录");
            std::fs::write(directory.join("nested").join("image"), b"image bytes")
                .expect("目录里写得进文件");
            assert!(file.exists() && directory.exists(), "作用域里两条路径都在");
        }
        assert!(
            !file_path.exists(),
            "作用域走完文件 {} 不在了",
            file_path.display()
        );
        assert!(
            !directory_path.exists(),
            "作用域走完目录 {} 连同里面的不在了",
            directory_path.display()
        );
    }

    /// 场景：守卫认领一个目录、写进文件之后作用域里 panic。预期：panic 接住之后那个目录不在了。
    /// 判别力（`crates/mutations.tsv`）：删法里不删，目录还在，这条红。
    #[test]
    fn a_claimed_directory_is_gone_after_a_panic_inside_the_scope() {
        let directory_path = fresh_temporary_path("panic");
        let panicked = std::panic::catch_unwind(|| {
            let directory = TemporaryPathRemovedOnDrop::claiming(directory_path.clone());
            std::fs::create_dir(&*directory).expect("临时目录建得了目录");
            std::fs::write(directory.join("image"), b"image bytes").expect("目录里写得进文件");
            assert!(directory.exists(), "panic 之前目录在");
            panic!("守卫作用域里造的 panic");
        });
        assert!(panicked.is_err(), "作用域里真 panic 了");
        assert!(
            !directory_path.exists(),
            "panic 之后目录 {} 不在了",
            directory_path.display()
        );
    }

    /// 场景：留现场的开关打开（`KeptForInspection`）时收尾一条临时路径；开关的值只有 `1` 算打开。
    /// 预期：路径还在；没设、空串、`0`、`yes` 都是删。
    #[test]
    fn the_keep_switch_set_to_one_keeps_the_path_and_any_other_value_removes_it() {
        let kept_path = fresh_temporary_path("kept");
        std::fs::write(&kept_path, b"image bytes").expect("临时目录写得进文件");
        remove_or_keep_the_temporary_path(&kept_path, TemporaryPathOnDrop::KeptForInspection);
        assert!(kept_path.exists(), "开关打开时路径留着");
        remove_or_keep_the_temporary_path(&kept_path, TemporaryPathOnDrop::Removed);
        assert!(!kept_path.exists(), "开关没开时路径删掉");
        assert_eq!(
            TemporaryPathOnDrop::from_the_keep_switch_value(Some(std::ffi::OsStr::new("1"))),
            TemporaryPathOnDrop::KeptForInspection
        );
        for value in [None, Some(""), Some("0"), Some("yes")] {
            assert_eq!(
                TemporaryPathOnDrop::from_the_keep_switch_value(value.map(std::ffi::OsStr::new)),
                TemporaryPathOnDrop::Removed,
                "开关值 {value:?} 不是 1：照删"
            );
        }
    }

    /// 场景：守卫把路径交给别的主人之后丢掉。预期：路径还在（归接手的主人删），接手的主人收尾之后不在了。
    #[test]
    fn a_path_handed_to_another_owner_is_left_for_that_owner_to_remove() {
        let image_path = fresh_temporary_path("handed-over");
        let image = TemporaryPathRemovedOnDrop::claiming(image_path.clone());
        std::fs::write(&*image, b"image bytes").expect("临时目录写得进文件");
        let handed_over = image.into_path_handed_to_another_owner();
        assert_eq!(handed_over, image_path);
        assert!(image_path.exists(), "交出去之后守卫不删");
        remove_or_keep_the_temporary_path(&handed_over, TemporaryPathOnDrop::Removed);
        assert!(!image_path.exists(), "接手的主人删掉");
    }
}
