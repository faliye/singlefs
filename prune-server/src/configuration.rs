//! 服务的位置与地址。
//!
//! - TCP 地址：只读多机配置 `multi-host.env` 的 `PRUNE_SERVER_TCP_ADDRESS=<本机在有线直连段上的地址>:<端口>`，不读环境变量；配置里没有这一键由调用方报错，不补缺省。
//!   第二台的客户端连同一个地址。配置文件的位置（`default_multi_host_configuration_path`）：`SINGLEFS_MULTI_HOST_CONFIG` 设了照它；没设从这个进程的
//!   当前目录起照 git 找主工作树的根（`git rev-parse --git-common-dir` 的上一层；在 git worktree 里跑也取主工作树那一份，多机配置被 git 忽略、
//!   只在主工作树里有），取它下面的 `multi-host.env`；当前目录不在 git 工作树里、git 起不来时取当前目录下那一份。第二台在同一棵共享的树里跑，读的是同一份。
//!   与 `crates/singlefs-verification/src/gpu_card_configuration.rs` 的 `multi_host_configuration_file` 同一种找法。
//!   不从编译时的清单目录往上拼仓根：那样读的是仓根下一份本机私有、被 git 忽略的文件，经 prune-server 客户端连服务的测试的单元键都拒算
//!   runtime-path（`scripts/gate/lib_function_units.py` 文件头「数据目录」）。这份文件只定连哪一处（服务的 TCP 地址），判定库只有一份，
//!   连哪一处不改任何判定的结局；连不上退 75 或 3，不当判定。
//!   弄坏开关 multi-host-configuration-from-working-directory-only：不顺 git 找、只取起点目录下那一份（自证 multi-host-configuration-from-the-main-worktree 证红）。
//!   写法照模板 `multi-host.env.example` 文件头：一行一个 `KEY=值`，值不做 shell 展开，两头成对的引号去掉一层。
//! - socket：`${XDG_RUNTIME_DIR}/singlefs-prune-server/server.sock`（照 task-server：目录 700、socket 600，全路径至多 107 字节）。
//! - 状态目录：`~/.local/state/singlefs/prune-server`（照 task-server 的库放在 `~/.local/state/singlefs/` 下）；
//!   服务自己那一份库在它下面的 `library`（服务头一次起时建；布局在 `storage.rs` 文件头），
//!   故障日志是它下面的 `faults.log`，控制命令的停服务记录在它下面的 `stops/`。
//! - 用例与自证的临时目录（`create_directory_holding_a_socket`）：建在 `TMPDIR` 底下，里面的 socket 全路径超 107 字节就改建在 `SHORT_TEMPORARY_ROOT` 底下，
//!   退到那里仍超就报错、不建。用例与自证里要放 socket 的临时目录都经它建，不各写一份。
use std::ffi::OsString;
use std::os::unix::fs::DirBuilderExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

pub const NETWORK_ADDRESS_KEY: &str = "PRUNE_SERVER_TCP_ADDRESS";
pub const MULTI_HOST_CONFIGURATION_VARIABLE: &str = "SINGLEFS_MULTI_HOST_CONFIG";
/// 多机配置的文件名（git 忽略、每台机器各一份；模板是仓根的 `multi-host.env.example`）。
pub const MULTI_HOST_CONFIGURATION_FILE_NAME: &str = "multi-host.env";
/// sockaddr_un 的 sun_path 是 108 字节，留一个给结尾的 0。
pub const MAXIMUM_SOCKET_PATH_BYTES: usize = 107;
/// 临时根（`TMPDIR`）放不下 socket 全路径时改用的根，也是 `TMPDIR` 没设或为空时的临时根：路径短。
pub const SHORT_TEMPORARY_ROOT: &str = "/tmp";
/// 用例与自证的临时目录只许建它的进程读写进出（socket 在里面）。
const SOCKET_DIRECTORY_MODE: u32 = 0o700;

static SOCKET_DIRECTORY_COUNTER: AtomicU64 = AtomicU64::new(0);

/// 没给 `--multi-host-configuration` 时读的那一份多机配置（文件头「TCP 地址」）：从这个进程的当前目录起找。
#[must_use]
pub fn default_multi_host_configuration_path() -> PathBuf {
    let current_directory = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    multi_host_configuration_path_from(
        std::env::var_os(MULTI_HOST_CONFIGURATION_VARIABLE),
        &current_directory,
        ConfigurationLookup::FollowGitToTheMainWorktree,
    )
}

/// 多机配置的缺省位置怎么找：顺 git 找主工作树（平常跑），或只取起点目录（弄坏开关 multi-host-configuration-from-working-directory-only）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfigurationLookup {
    FollowGitToTheMainWorktree,
    StartDirectoryOnly,
}

/// [`default_multi_host_configuration_path`] 去掉读环境与取当前目录的那一层：`configured`（`SINGLEFS_MULTI_HOST_CONFIG` 的值）不空照它；
/// 不然 `start_directory` 所在 git 工作树的主工作树根下的 `multi-host.env`，不在 git 工作树里（或 `lookup` 只取起点目录）取 `start_directory` 下那一份。
#[must_use]
pub fn multi_host_configuration_path_from(
    configured: Option<OsString>,
    start_directory: &Path,
    lookup: ConfigurationLookup,
) -> PathBuf {
    if let Some(configured) = configured.filter(|configured| !configured.is_empty()) {
        return PathBuf::from(configured);
    }
    let root = match lookup {
        ConfigurationLookup::FollowGitToTheMainWorktree => main_worktree_root_of(start_directory),
        ConfigurationLookup::StartDirectoryOnly => None,
    };
    root.unwrap_or_else(|| start_directory.to_path_buf())
        .join(MULTI_HOST_CONFIGURATION_FILE_NAME)
}

/// `directory` 所在 git 工作树的主工作树根：`git -C <directory> rev-parse --path-format=absolute --git-common-dir` 交回的目录的上一层；
/// 不是 git 工作树、git 起不来是 `None`。
fn main_worktree_root_of(directory: &Path) -> Option<PathBuf> {
    let output = Command::new("git")
        .arg("-C")
        .arg(directory)
        .args(["rev-parse", "--path-format=absolute", "--git-common-dir"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?;
    let common_directory = text.trim();
    if common_directory.is_empty() {
        return None;
    }
    Path::new(common_directory).parent().map(Path::to_path_buf)
}

/// 取一个键的值；只读文件、不读环境变量：文件读不了是错，键不在回 `None`，由调用方报错、不补缺省。
pub fn read_configuration_value(
    configuration_path: &Path,
    key: &str,
) -> Result<Option<String>, String> {
    let text = std::fs::read_to_string(configuration_path)
        .map_err(|error| format!("读不了多机配置 {}：{error}", configuration_path.display()))?;
    Ok(configuration_value_in_text(&text, key))
}

fn configuration_value_in_text(text: &str, key: &str) -> Option<String> {
    let mut found = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((line_key, raw_value)) = trimmed.split_once('=') else {
            continue;
        };
        if line_key.trim() == key {
            found = Some(strip_one_pair_of_quotes(raw_value.trim()).to_string());
        }
    }
    found
}

fn strip_one_pair_of_quotes(value: &str) -> &str {
    for quote in ['"', '\''] {
        if value.len() >= 2 && value.starts_with(quote) && value.ends_with(quote) {
            return &value[1..value.len() - 1];
        }
    }
    value
}

pub fn default_socket_path() -> Result<PathBuf, String> {
    match std::env::var_os("XDG_RUNTIME_DIR") {
        Some(directory) if !directory.is_empty() => Ok(PathBuf::from(directory)
            .join("singlefs-prune-server")
            .join("server.sock")),
        Some(_) | None => Err("XDG_RUNTIME_DIR 没设，算不出缺省的 socket 路径".to_string()),
    }
}

fn home_directory(what: &str) -> Result<PathBuf, String> {
    match std::env::var_os("HOME") {
        Some(home) if !home.is_empty() => Ok(PathBuf::from(home)),
        Some(_) | None => Err(format!("HOME 没设，算不出缺省的{what}")),
    }
}

pub fn default_state_directory() -> Result<PathBuf, String> {
    Ok(home_directory("状态目录")?.join(".local/state/singlefs/prune-server"))
}

/// 状态目录里的库目录。
pub fn library_directory_in(state_directory: &Path) -> PathBuf {
    state_directory.join("library")
}

/// socket 全路径的字节数超了上限时回说明。
pub fn socket_path_problem(socket_path: &Path) -> Option<String> {
    let byte_count = socket_path.as_os_str().len();
    if byte_count > MAXIMUM_SOCKET_PATH_BYTES {
        return Some(format!(
            "socket 路径 {} 有 {byte_count} 字节，超过上限 {MAXIMUM_SOCKET_PATH_BYTES}",
            socket_path.display()
        ));
    }
    None
}

/// 建一个独占的临时目录，里面要放相对路径为 `socket_file_name` 的 unix socket，回目录的路径。
/// 目录叫 `<name_prefix>-<进程号>-<本进程内的序号>`，建在 `temporary_directory_variable`（`TMPDIR` 的值，没设或为空取 [`SHORT_TEMPORARY_ROOT`]）底下；
/// socket 全路径超过 [`MAXIMUM_SOCKET_PATH_BYTES`] 就改建在 [`SHORT_TEMPORARY_ROOT`] 底下；退到那里仍超回错、什么都不建。
/// 权限 0700；放手时删目录是调用方的事。
pub fn create_directory_holding_a_socket(
    temporary_directory_variable: Option<OsString>,
    name_prefix: &str,
    socket_file_name: &str,
) -> Result<PathBuf, String> {
    create_directory_holding_a_socket_with_short_root(
        temporary_directory_variable,
        Path::new(SHORT_TEMPORARY_ROOT),
        name_prefix,
        socket_file_name,
    )
}

/// [`create_directory_holding_a_socket`] 把短根交出来的那一层：自证里换一个短根，测退到短根仍放不下的那条路。
pub fn create_directory_holding_a_socket_with_short_root(
    temporary_directory_variable: Option<OsString>,
    short_root: &Path,
    name_prefix: &str,
    socket_file_name: &str,
) -> Result<PathBuf, String> {
    let temporary_root = temporary_directory_variable
        .filter(|value| !value.is_empty())
        .map_or_else(|| short_root.to_path_buf(), PathBuf::from);
    let directory_name = format!(
        "{name_prefix}-{}-{}",
        std::process::id(),
        SOCKET_DIRECTORY_COUNTER.fetch_add(1, Ordering::SeqCst)
    );
    let mut directory = temporary_root.join(&directory_name);
    if socket_path_problem(&directory.join(socket_file_name)).is_some() {
        directory = short_root.join(&directory_name);
    }
    if let Some(problem) = socket_path_problem(&directory.join(socket_file_name)) {
        return Err(format!(
            "{problem}；临时根 {} 底下放不下，退到短根 {} 底下仍放不下",
            temporary_root.display(),
            short_root.display()
        ));
    }
    std::fs::DirBuilder::new()
        .mode(SOCKET_DIRECTORY_MODE)
        .create(&directory)
        .map_err(|error| format!("建不了临时目录 {}：{error}", directory.display()))?;
    Ok(directory)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use std::os::unix::net::UnixListener;

    const NAME_PREFIX: &str = "prune-socket-directory-test";
    const SOCKET_FILE_NAME: &str = "test.sock";

    /// 测试自己的一个短根：放手时连里面建出来的目录一起删。
    struct ScratchRoot {
        path: PathBuf,
    }

    impl ScratchRoot {
        fn create() -> ScratchRoot {
            let path = Path::new(SHORT_TEMPORARY_ROOT).join(format!(
                "prune-configuration-test-{}-{}",
                std::process::id(),
                SOCKET_DIRECTORY_COUNTER.fetch_add(1, Ordering::SeqCst)
            ));
            std::fs::create_dir(&path)
                .unwrap_or_else(|error| panic!("建不了测试的短根 {}：{error}", path.display()));
            ScratchRoot { path }
        }
    }

    impl Drop for ScratchRoot {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.path).unwrap_or_else(|error| {
                eprintln!("删不了测试的短根 {}：{error}", self.path.display());
            });
        }
    }

    /// 比 socket 路径上限还长、没有真建出来的临时根：用到它建目录会因为它不存在而失败，所以用到了就看得出。
    fn temporary_root_longer_than_a_socket_path_may_be(name: &str) -> OsString {
        Path::new(SHORT_TEMPORARY_ROOT)
            .join(name.repeat(MAXIMUM_SOCKET_PATH_BYTES + 20))
            .into_os_string()
    }

    #[test]
    fn socket_directory_is_built_under_the_temporary_root_when_the_socket_path_fits() {
        let temporary_root = ScratchRoot::create();
        let short_root = ScratchRoot::create();

        let directory = create_directory_holding_a_socket_with_short_root(
            Some(temporary_root.path.clone().into_os_string()),
            &short_root.path,
            NAME_PREFIX,
            SOCKET_FILE_NAME,
        )
        .expect("临时根短，放得下");

        assert!(
            directory.starts_with(&temporary_root.path),
            "socket 全路径放得下，目录该建在临时根 {} 底下，实际是 {}",
            temporary_root.path.display(),
            directory.display()
        );
    }

    #[test]
    fn socket_directory_falls_back_to_the_short_root_when_the_temporary_root_is_too_long_and_a_socket_binds_there(
    ) {
        let short_root = ScratchRoot::create();
        let long_root = temporary_root_longer_than_a_socket_path_may_be("r");

        let directory = create_directory_holding_a_socket_with_short_root(
            Some(long_root.clone()),
            &short_root.path,
            NAME_PREFIX,
            SOCKET_FILE_NAME,
        )
        .expect("临时根放不下，短根放得下");

        assert!(
            directory.starts_with(&short_root.path),
            "临时根 {} 放不下 socket 全路径，目录该改建在短根 {} 底下，实际是 {}",
            Path::new(&long_root).display(),
            short_root.path.display(),
            directory.display()
        );
        let socket_path = directory.join(SOCKET_FILE_NAME);
        UnixListener::bind(&socket_path)
            .unwrap_or_else(|error| panic!("绑不上 {}：{error}", socket_path.display()));
    }

    #[test]
    fn socket_directory_is_refused_and_nothing_is_built_when_even_the_short_root_cannot_hold_the_socket_path(
    ) {
        let long_temporary_root = temporary_root_longer_than_a_socket_path_may_be("r");
        let long_short_root = PathBuf::from(temporary_root_longer_than_a_socket_path_may_be("s"));
        std::fs::create_dir_all(&long_short_root).unwrap_or_else(|error| {
            panic!("建不了过长的短根 {}：{error}", long_short_root.display())
        });
        let outcome = create_directory_holding_a_socket_with_short_root(
            Some(long_temporary_root),
            &long_short_root,
            NAME_PREFIX,
            SOCKET_FILE_NAME,
        );
        let built = std::fs::read_dir(&long_short_root)
            .unwrap_or_else(|error| panic!("读不了 {}：{error}", long_short_root.display()))
            .count();
        std::fs::remove_dir_all(&long_short_root).unwrap_or_else(|error| {
            panic!("删不了过长的短根 {}：{error}", long_short_root.display())
        });

        let message = outcome.expect_err("短根也放不下，该回错");
        assert!(
            message.contains("超过上限") && message.contains("仍放不下"),
            "错误说明该点名超了上限、短根也放不下：{message}"
        );
        assert_eq!(built, 0, "放不下就什么都不建，短根里却建出了 {built} 样");
    }

    #[test]
    fn socket_directory_is_readable_and_writable_by_its_owner_only() {
        let short_root = ScratchRoot::create();

        let directory = create_directory_holding_a_socket_with_short_root(
            Some(short_root.path.clone().into_os_string()),
            &short_root.path,
            NAME_PREFIX,
            SOCKET_FILE_NAME,
        )
        .expect("短根放得下");

        let mode = std::fs::metadata(&directory)
            .unwrap_or_else(|error| panic!("读不了 {}：{error}", directory.display()))
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, SOCKET_DIRECTORY_MODE, "目录权限该是 0700");
    }

    #[test]
    fn an_empty_temporary_directory_variable_is_read_as_unset_and_the_short_root_is_used() {
        let short_root = ScratchRoot::create();

        let outcome = create_directory_holding_a_socket_with_short_root(
            Some(OsString::new()),
            &short_root.path,
            NAME_PREFIX,
            SOCKET_FILE_NAME,
        );

        // 空值被当成根时目录建在当前目录底下：先删掉再判，不留垃圾。
        if let Ok(path) = &outcome {
            if !path.starts_with(&short_root.path) {
                std::fs::remove_dir(path)
                    .unwrap_or_else(|error| eprintln!("删不了建错地方的目录：{error}"));
            }
        }
        let directory = outcome.expect("空值当没设，短根放得下");
        assert!(
            directory.starts_with(&short_root.path),
            "TMPDIR 为空该当没设、用短根 {}，实际建在 {}",
            short_root.path.display(),
            directory.display()
        );
    }
}
