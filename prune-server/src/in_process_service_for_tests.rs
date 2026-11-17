//! 单元测试共用的装置：一条用例独占的临时目录，与在这个进程里起的一份服务（组提交窗口、绑定地址、自证造错都由用例给）。
//! 只编进测试（`lib.rs` 里 `#[cfg(test)]`）。

use std::path::PathBuf;
use std::time::Duration;

use crate::break_switches::BreakSwitches;
use crate::client::{RouteAddress, SimpleClient};
use crate::committer::CommitterSettings;
use crate::configuration::create_directory_holding_a_socket;
use crate::server::{start_in_this_process_listening_on, ServiceInThisProcess};

/// socket 文件在用例临时目录里的文件名。
const SOCKET_FILE_NAME: &str = "socket";
/// 状态目录（故障日志与库在里面）在用例临时目录里的目录名。
const STATE_DIRECTORY_NAME: &str = "state";
/// 在这个进程里起的服务缺省绑的 TCP 地址：本机回环、端口由内核挑。
pub(crate) const LOOPBACK_NETWORK_ADDRESS: &str = "127.0.0.1:0";

/// 这条用例独占的临时目录（建法见 `create_directory_holding_a_socket`：里面的 socket 全路径不超限），丢掉时删。
pub(crate) struct TemporaryDirectoryOfTheTest {
    pub(crate) path: PathBuf,
}

impl TemporaryDirectoryOfTheTest {
    /// `name_prefix` 是目录名的前缀，写这条用例测什么；目录名另带进程号与本进程内的序号。
    pub(crate) fn create(name_prefix: &str) -> TemporaryDirectoryOfTheTest {
        let path = create_directory_holding_a_socket(
            std::env::var_os("TMPDIR"),
            name_prefix,
            SOCKET_FILE_NAME,
        )
        .unwrap_or_else(|message| panic!("{message}"));
        TemporaryDirectoryOfTheTest { path }
    }
}

impl Drop for TemporaryDirectoryOfTheTest {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

/// 提交线程的设置：组提交窗口与一组至多收几帧由用例给，自证造错的几项都不开。
pub(crate) fn committer_settings_grouping(
    group_window: Duration,
    group_maximum_frames: usize,
) -> CommitterSettings {
    CommitterSettings {
        group_window,
        group_maximum_frames,
        selftest_commit_delay: None,
        selftest_storage_failure_key: None,
        selftest_write_failure_file: None,
    }
}

/// 在这个进程里起的一份服务，连同它的临时目录；丢掉时先停服务、再删目录（字段按声明的次序放手）。
pub(crate) struct ServiceUnderTest {
    pub(crate) service: ServiceInThisProcess,
    pub(crate) directory: TemporaryDirectoryOfTheTest,
}

impl ServiceUnderTest {
    /// 起一份绑本机回环的服务，提交线程取 `committer`。
    pub(crate) fn start(name_prefix: &str, committer: CommitterSettings) -> ServiceUnderTest {
        ServiceUnderTest::start_listening_on(name_prefix, LOOPBACK_NETWORK_ADDRESS, committer)
    }

    /// 起一份 TCP 绑 `network_address` 的服务，提交线程取 `committer`。
    pub(crate) fn start_listening_on(
        name_prefix: &str,
        network_address: &str,
        committer: CommitterSettings,
    ) -> ServiceUnderTest {
        ServiceUnderTest::start_in(
            TemporaryDirectoryOfTheTest::create(name_prefix),
            network_address,
            committer,
        )
    }

    /// 起在用例已经建好的临时目录里（`committer` 的设置要指到目录里的文件时用）。
    pub(crate) fn start_in(
        directory: TemporaryDirectoryOfTheTest,
        network_address: &str,
        committer: CommitterSettings,
    ) -> ServiceUnderTest {
        let service = start_in_this_process_listening_on(
            &directory.path.join(STATE_DIRECTORY_NAME),
            &directory.path.join(SOCKET_FILE_NAME),
            network_address,
            committer,
        )
        .unwrap_or_else(|failure| panic!("在这个进程里起不来服务：{failure:?}"));
        ServiceUnderTest { service, directory }
    }

    /// 经 unix socket 连上这一份服务。
    pub(crate) fn connect(&self) -> SimpleClient {
        SimpleClient::connect(
            &RouteAddress::UnixSocket(self.service.socket_path().to_path_buf()),
            BreakSwitches::none(),
        )
        .unwrap_or_else(|message| panic!("连不上这一份服务：{message}"))
    }
}
