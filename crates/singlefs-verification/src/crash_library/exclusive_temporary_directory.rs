//! 独占的一次性临时目录：一份库、一个用例各建一份，名字按标签截到路径段的上限，丢掉时删。

use super::*;

/// 这个进程里建过几份独占的一次性临时目录（[`ExclusiveTemporaryDirectory::create`] 每次取一个号，名字里带它）。
static EXCLUSIVE_TEMPORARY_DIRECTORIES_CREATED_IN_THIS_PROCESS: AtomicU64 = AtomicU64::new(0);

/// 建一份独占的一次性临时目录最多试几个名字：名字撞上已有的目录（同进程号的旧进程被杀时留下的）就换下一个号，试满还建不出就报错。
const MOST_EXCLUSIVE_TEMPORARY_DIRECTORY_NAMES_TRIED: u32 = 64;

/// 一段路径名（一级目录的名字）最多几个字节：Linux 的 `NAME_MAX`，再长 `mkdir` 报 `File name too long (os error 36)`。
const MOST_BYTES_OF_ONE_PATH_COMPONENT: usize = 255;

/// 标签放不下、截短之后接在后面的标签摘要取几位十六进制：标签整串 sha256 的前 16 位（64 比特），
/// 只在截掉的那一段不同的两个标签靠它分开，同一台机器上同时在的一次性临时目录撞上的机会不计。
const LABEL_DIGEST_HEXADECIMAL_DIGITS_OF_A_SHORTENED_NAME: usize = 16;

/// 一次性临时目录的单段名字 `singlefs-<标签>-<进程号>-<进程内序号>`，标签里的 `/` 换成 `-`。
/// 整段超 [`MOST_BYTES_OF_ONE_PATH_COMPONENT`] 字节的，标签截到放得下（按 UTF-8 字符边界截，不截半个字符），
/// 截短的标签后面接 `-` 与标签整串 sha256 十六进制的前 [`LABEL_DIGEST_HEXADECIMAL_DIGITS_OF_A_SHORTENED_NAME`] 位，
/// 再接 `-<进程号>-<进程内序号>`：只在截掉的那一段不同的两个标签照样得出不同的名字。放得下的照整串拼。
fn exclusive_temporary_directory_name(
    label: &str,
    process_identifier: u32,
    serial_in_this_process: u64,
) -> String {
    let label_without_slashes = label.replace('/', "-");
    let suffix = format!("-{process_identifier}-{serial_in_this_process}");
    let name_in_full = format!("singlefs-{label_without_slashes}{suffix}");
    if name_in_full.len() <= MOST_BYTES_OF_ONE_PATH_COMPONENT {
        return name_in_full;
    }
    let label_digest = hexadecimal_text(&sha256_digest(label.as_bytes()));
    let digest_prefix = &label_digest[..LABEL_DIGEST_HEXADECIMAL_DIGITS_OF_A_SHORTENED_NAME];
    let label_bytes_that_fit = MOST_BYTES_OF_ONE_PATH_COMPONENT
        .saturating_sub("singlefs-".len() + "-".len() + digest_prefix.len() + suffix.len());
    let mut label_bytes_kept = label_bytes_that_fit;
    // 迭代上界：3 次（UTF-8 一个字符最多 4 个字节，退到字符边界最多退 3 个字节；0 总是字符边界）
    while !label_without_slashes.is_char_boundary(label_bytes_kept) {
        label_bytes_kept -= 1;
    }
    format!(
        "singlefs-{}-{digest_prefix}{suffix}",
        &label_without_slashes[..label_bytes_kept]
    )
}

/// 调用方独占的一份一次性临时目录：`${TMPDIR:-/tmp}` 下 `singlefs-<标签>-<进程号>-<进程内序号>`（标签里的 `/` 换成 `-`，目录只占一层；
/// 整段超一段路径名的上限时标签截短、接标签摘要，[`exclusive_temporary_directory_name`]），`create_dir` 排他新建；
/// 丢掉时只删这一份（连同里面的东西）。同一个进程里两次用同一个标签各拿一份、互不删对方；两个进程同跑一个测试二进制，进程号不同。
/// 标签只为在临时目录里认得出是谁建的，不靠它区分：验证包的单测与集成测试要一份自己的临时目录都经它建
/// （照崩溃注入镜像目录的写法，`crate::crash_injection` 的 `create_the_image_directory_of_a_campaign_call`）。
pub struct ExclusiveTemporaryDirectory {
    path: PathBuf,
}

impl ExclusiveTemporaryDirectory {
    /// # Errors
    /// 临时目录里建不了目录；试满 [`MOST_EXCLUSIVE_TEMPORARY_DIRECTORY_NAMES_TRIED`] 个名字都撞上已有的。
    pub fn create(label: &str) -> Result<Self, String> {
        Self::create_in(&std::env::temp_dir(), label, &std::fs::DirBuilder::new())
    }

    /// 同 [`Self::create`]，只是建在 `root` 下、照 `directory_builder` 建（权限由它定）。
    ///
    /// # Errors
    /// 同 [`Self::create`]。
    pub(super) fn create_in(
        root: &Path,
        label: &str,
        directory_builder: &std::fs::DirBuilder,
    ) -> Result<Self, String> {
        let mut last_name_tried = PathBuf::new();
        // 迭代上界：MOST_EXCLUSIVE_TEMPORARY_DIRECTORY_NAMES_TRIED 次；只有名字撞上已有目录时才试下一个
        for _name_tried in 0..MOST_EXCLUSIVE_TEMPORARY_DIRECTORY_NAMES_TRIED {
            let serial_in_this_process = EXCLUSIVE_TEMPORARY_DIRECTORIES_CREATED_IN_THIS_PROCESS
                .fetch_add(1, Ordering::SeqCst);
            let path = root.join(exclusive_temporary_directory_name(
                label,
                std::process::id(),
                serial_in_this_process,
            ));
            match directory_builder.create(&path) {
                Ok(()) => return Ok(Self { path }),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    last_name_tried = path;
                }
                Err(error) => {
                    return Err(format!("建不了一次性临时目录 {}：{error}", path.display()))
                }
            }
        }
        Err(format!(
            "一次性临时目录试了 {MOST_EXCLUSIVE_TEMPORARY_DIRECTORY_NAMES_TRIED} 个名字都已经在（最后一个 {}）",
            last_name_tried.display()
        ))
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// 当路径用（`directory.join(…)`、交给收 `&Path` 的函数）：就是 [`ExclusiveTemporaryDirectory::path`]。
impl std::ops::Deref for ExclusiveTemporaryDirectory {
    type Target = Path;

    fn deref(&self) -> &Path {
        &self.path
    }
}

impl AsRef<Path> for ExclusiveTemporaryDirectory {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}

impl Drop for ExclusiveTemporaryDirectory {
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_dir_all(&self.path) {
            if error.kind() != std::io::ErrorKind::NotFound {
                eprintln!("删不掉一次性临时目录 {}：{error}", self.path.display());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 标签里带 `/`（场景名、随机历史名这类）：照样建在临时目录下一层、`/` 换成 `-`，不当成多级路径（排他新建不递归，上一级不在就建不出）。
    /// 判别力（`crates/mutations.tsv`）：拼目录名时不换标签里的 `/`，这条红（ENOENT）。
    #[test]
    fn a_label_with_slashes_is_created_one_level_under_the_temporary_directory() {
        let directory = ExclusiveTemporaryDirectory::create("scenario/with/slashes")
            .expect("带 / 的标签也建得出");
        assert_eq!(
            directory.path().parent(),
            Some(std::env::temp_dir().as_path()),
            "建在临时目录下一层"
        );
        assert!(
            directory
                .path()
                .file_name()
                .expect("目录有名字")
                .to_string_lossy()
                .starts_with("singlefs-scenario-with-slashes-"),
            "标签里的 / 换成 -：{}",
            directory.path().display()
        );
    }

    /// 标签长到整串拼出的单段名超一段路径名的上限（带三个字节一个的中文）：照样建在临时目录下一层、单段名不超上限；
    /// 只在截掉的那一段不同的两个标签，名字去掉 `-<进程号>-<进程内序号>` 之后仍不同；截短点落在中文字符中间时退到字符边界。
    /// 判别力（`crates/mutations.tsv`）：不截短、整串照拼，建不出（ENAMETOOLONG）这条红；截短后不接标签摘要，两个名字相同这条红；
    /// 截短点不退到字符边界，切片切在字符中间 panic，这条红。
    #[test]
    fn overlong_multibyte_label_is_cut_to_the_path_component_limit_and_labels_differing_only_beyond_the_cut_get_distinct_directories(
    ) {
        let shared_head = "崩溃判定库一次性目录".repeat(30);
        let label_ending_in_east = format!("{shared_head}东");
        let label_ending_in_west = format!("{shared_head}西");
        assert!(
            format!("singlefs-{label_ending_in_east}").len() > MOST_BYTES_OF_ONE_PATH_COMPONENT,
            "标签整串拼出的名字超一段路径名的上限"
        );
        let east =
            ExclusiveTemporaryDirectory::create(&label_ending_in_east).expect("超长的标签也建得出");
        let west = ExclusiveTemporaryDirectory::create(&label_ending_in_west)
            .expect("另一个超长的标签也建得出");
        let name_of = |directory: &ExclusiveTemporaryDirectory| -> String {
            directory
                .path()
                .file_name()
                .expect("目录有名字")
                .to_str()
                .expect("名字是 UTF-8")
                .to_owned()
        };
        for directory in [&east, &west] {
            assert_eq!(
                directory.path().parent(),
                Some(std::env::temp_dir().as_path()),
                "建在临时目录下一层：{}",
                directory.path().display()
            );
            assert!(
                name_of(directory).len() <= MOST_BYTES_OF_ONE_PATH_COMPONENT,
                "单段名不超一段路径名的上限：{} 字节",
                name_of(directory).len()
            );
            assert!(
                name_of(directory).starts_with("singlefs-崩溃判定库"),
                "截短的名字留着标签开头：{}",
                name_of(directory)
            );
        }
        let without_process_and_serial = |name: &str| -> String {
            name.rsplitn(3, '-')
                .nth(2)
                .expect("名字以 -<进程号>-<进程内序号> 收尾")
                .to_owned()
        };
        assert_ne!(
            without_process_and_serial(&name_of(&east)),
            without_process_and_serial(&name_of(&west)),
            "只在截掉的那一段不同的两个标签，名字不同"
        );
        // 进程号各差一位，后缀各差一个字节：截短点落在三个字节一个的中文的三个位置上（字符边界与字符中间两处）
        for process_identifier in [7, 70, 700] {
            let name =
                exclusive_temporary_directory_name(&label_ending_in_east, process_identifier, 1);
            assert!(
                name.len() <= MOST_BYTES_OF_ONE_PATH_COMPONENT,
                "进程号 {process_identifier}：单段名不超上限，{} 字节",
                name.len()
            );
            assert!(
                name.ends_with(&format!("-{process_identifier}-1")),
                "进程号 {process_identifier}：名字以 -<进程号>-<进程内序号> 收尾：{name}"
            );
        }
    }

    /// 同一个进程里两次用同一个标签建一次性临时目录：各拿一份不同的、新建的空目录；丢掉一份只删那一份，另一份连同里面的文件还在。
    /// 判别力（`crates/mutations.tsv`）：名字退回只由标签与进程号定，第二次建撞上头一份、试满名字建不出，这条红。
    #[test]
    fn two_exclusive_temporary_directories_of_one_label_in_one_process_differ_and_dropping_one_keeps_the_other(
    ) {
        const LABEL: &str = "exclusive-temporary-directory-same-label";
        let one = ExclusiveTemporaryDirectory::create(LABEL).expect("头一份建得出");
        let another = ExclusiveTemporaryDirectory::create(LABEL)
            .expect("同一个标签的第二份也建得出：名字里带进程内序号");
        assert_ne!(
            one.path(),
            another.path(),
            "同一进程、同一标签的两份是两个目录"
        );
        for directory in [&one, &another] {
            assert!(
                std::fs::read_dir(directory.path())
                    .expect("建好的目录读得出")
                    .next()
                    .is_none(),
                "{} 是新建的空目录",
                directory.path().display()
            );
        }
        let file_of_the_other = another.join("written-by-the-other-holder");
        std::fs::write(&file_of_the_other, b"kept").expect("另一份里写得进文件");
        let path_of_the_dropped = one.path().to_path_buf();
        drop(one);
        assert!(!path_of_the_dropped.exists(), "丢掉的那一份连目录一起删掉");
        assert_eq!(
            std::fs::read(&file_of_the_other).expect("另一份里的文件还在"),
            b"kept",
            "丢掉一份不碰另一份"
        );
    }
}
