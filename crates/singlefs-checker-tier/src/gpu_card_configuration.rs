//! 显卡配置怎么读：`GPU_CARDS_BY_PRIORITY` 点名的卡（在哪台机器、`nvidia-smi` 序号、显存额度）、这个进程在哪台机器、`键=值` 配置文件里一个键的值、
//! 点名的卡里哪几张用不上。崩溃放量取卡（`crate::crash_amplification`，随 `verdict-store` 编）与 GPU 用例共用的卡池
//! （`crate::gpu_unit_checks::gpu_test_start`，随 `gpu` 编）都照它挑卡，所以单放一份、只开其中一个特性也编得进；
//! `crate::crash_amplification` 再 `pub use` 回去，它的调用方与单测照旧经它引。
//! 显卡的两个键写在多机配置那一份里（用户 2026-10-10 定：多机配置合成一份，不零散配置；判决
//! `research/prompts/m3-crash-enumeration-merge-r3-main-verification.md` 第三节 J5、J11）：`GPU_CARDS_BY_PRIORITY`（不清场时）与
//! `GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED`（清场之后）；多机配置在哪照 [`multi_host_configuration_file`]（与
//! `scripts/crash/crash-amplification-lib.sh` 的 `crash_amplification_multi_host_configuration_file` 同一种找法）。私有显卡配置
//! （`SINGLEFS_GPU_CARDS_CONFIG` 指的那一份）退役，这里不读。

use std::path::{Path, PathBuf};

/// 配置里的一张卡在哪台机器上。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardHost {
    Local,
    Peer,
}

impl CardHost {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Peer => "peer",
        }
    }

    /// 这个进程在哪台机器上：环境变量 `SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE`，没设是本机。
    ///
    /// # Errors
    /// 写的不是 `local` 也不是 `peer`。
    pub fn of_this_process() -> Result<Self, String> {
        Self::of_the_role_variable(
            std::env::var(HOST_ROLE_ENVIRONMENT_VARIABLE)
                .ok()
                .as_deref(),
        )
    }

    /// 那个环境变量的值（没设是 `None`）说的是哪台机器。
    ///
    /// # Errors
    /// 写的不是 `local` 也不是 `peer`。
    fn of_the_role_variable(value: Option<&str>) -> Result<Self, String> {
        match value {
            None => Ok(Self::Local),
            Some(text) => Self::parse(text),
        }
    }

    fn parse(text: &str) -> Result<Self, String> {
        match text.trim() {
            "local" => Ok(Self::Local),
            "peer" => Ok(Self::Peer),
            other => Err(format!("机器要写 local 或 peer，实际「{other}」")),
        }
    }
}

/// 配置点名的一张卡（`GPU_CARDS_BY_PRIORITY` 里的一项）：在哪台机器、`nvidia-smi` 的序号、给崩溃放量的显存额度。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CardByPriority {
    pub host: CardHost,
    pub index: u32,
    pub memory_quota_mebibytes: u64,
}

/// 读 `GPU_CARDS_BY_PRIORITY` 的值：空白隔开的若干项，每项 `<local|peer>:<nvidia-smi 序号>:<显存额度>`，
/// 额度是正整数加 `G` 或 `M`（1024 进制）；写在前面的优先级高，任务先装它、装满了才给后面的。
///
/// # Errors
/// 一项不是三段、机器或序号或额度写法不对、同一张卡点名两次、一项都没有。
pub fn parse_cards_by_priority(text: &str) -> Result<Vec<CardByPriority>, String> {
    let mut cards: Vec<CardByPriority> = Vec::new();
    for entry in text.split_whitespace() {
        let fields: Vec<&str> = entry.split(':').collect();
        let [host, index, quota] = fields.as_slice() else {
            return Err(format!(
                "显卡优先级的一项要写 <local|peer>:<序号>:<显存额度>，实际「{entry}」"
            ));
        };
        let host = CardHost::parse(host)?;
        let index: u32 = index
            .parse()
            .map_err(|_| format!("显卡序号不是数：「{entry}」"))?;
        let (number, unit_mebibytes) = match quota.as_bytes().last() {
            Some(b'G') => (&quota[..quota.len() - 1], 1024u64),
            Some(b'M') => (&quota[..quota.len() - 1], 1u64),
            _ => {
                return Err(format!(
                    "显存额度要带单位 G 或 M（1024 进制），实际「{entry}」"
                ))
            }
        };
        let number: u64 = number
            .parse()
            .ok()
            .filter(|number| *number > 0)
            .ok_or_else(|| format!("显存额度不是正整数：「{entry}」"))?;
        if cards
            .iter()
            .any(|card| card.host == host && card.index == index)
        {
            return Err(format!("同一张卡点名了两次：「{entry}」"));
        }
        cards.push(CardByPriority {
            host,
            index,
            memory_quota_mebibytes: number * unit_mebibytes,
        });
    }
    if cards.is_empty() {
        return Err("显卡优先级一项都没写".to_string());
    }
    Ok(cards)
}

/// `键=值` 配置文件里一个键的值（去掉两头的空白与一层引号）；文件不在、键没写是 `None`。
#[must_use]
pub fn configuration_value(path: &Path, key: &str) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    text.lines().find_map(|line| {
        let value = line.trim().strip_prefix(key)?.strip_prefix('=')?.trim();
        let value = value
            .strip_prefix('\'')
            .and_then(|rest| rest.strip_suffix('\''))
            .or_else(|| {
                value
                    .strip_prefix('"')
                    .and_then(|rest| rest.strip_suffix('"'))
            })
            .unwrap_or(value);
        Some(value.to_string())
    })
}

/// 多机配置里不清场时的显卡先后与额度那个键。
pub const CARDS_BY_PRIORITY_KEY: &str = "GPU_CARDS_BY_PRIORITY";
/// 多机配置里清场之后的显卡先后与额度那个键（同一种写法，[`parse_cards_by_priority`]）。
pub const CARDS_BY_PRIORITY_WHEN_QUIESCED_KEY: &str = "GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED";
/// 多机配置里开不开 GPU 那个键（写 `1` 是开）。
pub const GPU_SWITCH_KEY: &str = "ENABLE_GPU";

/// 这个进程在不在清场之下：清场的那一层（`scripts/crash/gpu-quiesce-run.sh`）给命令设 `GPU_QUIESCE_HOLDER=<这一趟的登记文件>`，
/// 文件在就是清过场的。cargo 起测试二进制时中间隔着 cargo 一层，不按父进程号核登记文件的名字，只看它在不在。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GpuQuiesce {
    QuiescedByTheWrapper,
    NotQuiesced,
}

impl GpuQuiesce {
    #[must_use]
    pub fn of_this_process() -> Self {
        match std::env::var_os(QUIESCE_HOLDER_ENVIRONMENT_VARIABLE) {
            Some(holder) if Path::new(&holder).is_file() => Self::QuiescedByTheWrapper,
            Some(_) | None => Self::NotQuiesced,
        }
    }
}

/// 这个进程在哪台机器上的那个环境变量（`local` 或 `peer`，没设是本机，[`CardHost::of_this_process`]）。
pub const HOST_ROLE_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE";
/// 多机配置在哪的那个环境变量（[`multi_host_configuration_file`]）。
pub const MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_MULTI_HOST_CONFIG";
/// 清场的那一层给命令设的登记文件那个环境变量（[`GpuQuiesce::of_this_process`]）。
pub const QUIESCE_HOLDER_ENVIRONMENT_VARIABLE: &str = "GPU_QUIESCE_HOLDER";

/// 配置的显卡优先级：环境变量 `GPU_CARDS_BY_PRIORITY` 设了就照它（分项驱动按清没清场从多机配置里挑一份送过来，第二台的树副本也靠它），
/// 没设读多机配置（[`multi_host_configuration_file`]）：清过场（[`GpuQuiesce::of_this_process`]）取 [`CARDS_BY_PRIORITY_WHEN_QUIESCED_KEY`]，
/// 没清场取 [`CARDS_BY_PRIORITY_KEY`]；没清场而都没写、清过场而 GPU 没开也都没写是 `None`（本机能用的卡按空闲显存从大到小用，不设额度）。
///
/// # Errors
/// 写了而写法不对（[`parse_cards_by_priority`]）；清过场、多机配置开了 GPU 或写了不清场那一份而没写清场之后那一份（[`cards_by_priority_from`]）。
pub fn cards_by_priority_from_configuration_or_environment(
    configuration_file: Option<&Path>,
) -> Result<Option<Vec<CardByPriority>>, String> {
    cards_by_priority_from(
        std::env::var(CARDS_BY_PRIORITY_KEY).ok(),
        configuration_file,
        GpuQuiesce::of_this_process(),
    )
}

/// [`cards_by_priority_from_configuration_or_environment`] 去掉读环境的那一层：环境变量的值、多机配置、清没清场都由调用方给。
/// 清过场而多机配置开了 GPU（[`GPU_SWITCH_KEY`]）或写了不清场那一份显卡先后（[`CARDS_BY_PRIORITY_KEY`]）、没写清场之后那一份：报错，
/// 不退回不清场那一份、也不照本机空闲显存挑卡（清场之下要显卡先后；用不上照「配了 GPU 卡用不上退 78、不降级」报错；与
/// `scripts/crash/crash-amplification-lib.sh` 的 `crash_amplification_prepare` 同一处、同一个判据拒）。清过场而 GPU 没开、两份都没写的是 `None`。
///
/// # Errors
/// 写了而写法不对（[`parse_cards_by_priority`]）；清过场、开了 GPU 或写了不清场那一份而没写清场之后那一份。
pub fn cards_by_priority_from(
    environment_value: Option<String>,
    configuration_file: Option<&Path>,
    quiesce: GpuQuiesce,
) -> Result<Option<Vec<CardByPriority>>, String> {
    let configured_value =
        |key: &str| configuration_file.and_then(|path| configuration_value(path, key));
    let written =
        |value: &Option<String>| value.as_deref().is_some_and(|text| !text.trim().is_empty());
    let text = match (environment_value, quiesce) {
        (Some(text), GpuQuiesce::QuiescedByTheWrapper | GpuQuiesce::NotQuiesced) => Some(text),
        (None, GpuQuiesce::QuiescedByTheWrapper) => {
            let when_quiesced = configured_value(CARDS_BY_PRIORITY_WHEN_QUIESCED_KEY);
            let gpu_is_enabled = configured_value(GPU_SWITCH_KEY)
                .is_some_and(|value| value.trim().trim_matches('"') == "1");
            if !written(&when_quiesced)
                && (gpu_is_enabled || written(&configured_value(CARDS_BY_PRIORITY_KEY)))
            {
                return Err(format!(
                    "说是清过场，多机配置{}开了 GPU（{GPU_SWITCH_KEY}=1）或写了 {CARDS_BY_PRIORITY_KEY}、却没写 {CARDS_BY_PRIORITY_WHEN_QUIESCED_KEY}：\
                     不退回不清场那一份显卡先后，也不照本机空闲显存挑卡（照仓根 multi-host.env.example 写清场之后的显卡先后与额度；运行条件不满足，退 78）",
                    configuration_file.map_or_else(String::new, |path| format!(" {}", path.display()))
                ));
            }
            when_quiesced
        }
        (None, GpuQuiesce::NotQuiesced) => configured_value(CARDS_BY_PRIORITY_KEY),
    };
    match text {
        Some(text) if !text.trim().is_empty() => parse_cards_by_priority(&text).map(Some),
        Some(_) | None => Ok(None),
    }
}

/// 调用方给这个进程的显卡配置：挑卡读的四个环境变量（多机配置在哪 [`MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE`]、显卡先后
/// [`CARDS_BY_PRIORITY_KEY`]、这个进程在哪台机器 [`HOST_ROLE_ENVIRONMENT_VARIABLE`]、清没清场 [`QUIESCE_HOLDER_ENVIRONMENT_VARIABLE`]）
/// 在这个进程第一次换环境之前的值（[`GpuConfigurationOfTheCaller::of_this_process`]）。
/// GPU 用例共用的卡池（`crate::gpu_unit_checks::gpu_test_start`）照它挑卡，用例抄显卡先后（[`card_priority_lines_of_the_caller`]）也照它：
/// 同一个测试二进制里别的用例拿着改环境的锁、把这几个变量换成它自己的临时配置时，这两处读到的仍是调用方那一份，不是那条用例的
/// （主 agent 2026-10-10 定：卡池只认进程起来时调用方的那份配置，不跟用例临时改的环境走）。
/// 崩溃放量一趟（`crate::crash_amplification::run_from_environment`）照旧读此刻的环境：用例给这一趟设的临时配置就是要它照着跑的。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GpuConfigurationOfTheCaller {
    multi_host_configuration: Option<PathBuf>,
    cards_by_priority: Option<String>,
    host_role: Option<String>,
    quiesce: GpuQuiesce,
}

/// [`GpuConfigurationOfTheCaller::of_this_process`] 记下的那一份：进程里只记一次。
static GPU_CONFIGURATION_OF_THE_CALLER: std::sync::OnceLock<GpuConfigurationOfTheCaller> =
    std::sync::OnceLock::new();

impl GpuConfigurationOfTheCaller {
    /// 调用方给这个进程的那一份：进程里第一次问的时候从环境记下，之后每次交同一份。
    /// 记的那一刻环境还是调用方的：这个进程里改这几个变量只经用例的环境守卫
    /// （`crate::crash_amplification::EnvironmentRestoredOnDrop::saved_then_cleared`），它在改任何一个变量之前先调这里；
    /// 有守卫改着环境的时候，这一份早在那个守卫建的时候记下了。
    #[must_use]
    pub fn of_this_process() -> &'static Self {
        GPU_CONFIGURATION_OF_THE_CALLER.get_or_init(Self::read_from_the_environment_now)
    }

    /// 此刻进程环境里的那四个变量。只给 [`Self::of_this_process`] 记那一份用：别处在守卫改着环境时读，读到的是那条用例的临时配置。
    pub(crate) fn read_from_the_environment_now() -> Self {
        Self {
            multi_host_configuration: std::env::var_os(
                MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE,
            )
            .map(PathBuf::from),
            cards_by_priority: std::env::var(CARDS_BY_PRIORITY_KEY).ok(),
            host_role: std::env::var(HOST_ROLE_ENVIRONMENT_VARIABLE).ok(),
            quiesce: GpuQuiesce::of_this_process(),
        }
    }

    /// 这个进程在哪台机器（同 [`CardHost::of_this_process`]，读的是这一份）。
    ///
    /// # Errors
    /// 写的不是 `local` 也不是 `peer`。
    pub fn host(&self) -> Result<CardHost, String> {
        CardHost::of_the_role_variable(self.host_role.as_deref())
    }

    /// 多机配置在哪（同 [`multi_host_configuration_file`]，读的是这一份）。
    ///
    /// # Errors
    /// 这一份设了多机配置的路径而那份文件不在。
    pub fn multi_host_configuration_file(
        &self,
        repository_root: &Path,
    ) -> Result<Option<PathBuf>, String> {
        multi_host_configuration_file_given(
            self.multi_host_configuration.clone(),
            repository_root,
            git_common_directory_of(repository_root),
        )
    }

    /// 配置的显卡优先级（同 [`cards_by_priority_from_configuration_or_environment`]，环境那一层读的是这一份）。
    ///
    /// # Errors
    /// 同 [`cards_by_priority_from`]。
    pub fn cards_by_priority(
        &self,
        configuration_file: Option<&Path>,
    ) -> Result<Option<Vec<CardByPriority>>, String> {
        cards_by_priority_from(
            self.cards_by_priority.clone(),
            configuration_file,
            self.quiesce,
        )
    }

    /// 用例自己写一份开 GPU 的多机配置时，照抄调用方这一份的显卡先后（「只写 `ENABLE_GPU=1` 的用例在清场之下要显卡先后的，用例自己在配置里写上」）：
    /// 这一份设了 `GPU_CARDS_BY_PRIORITY`（入口挑好清没清场那一份送过来）就两个键都写它；没设照这一份的多机配置
    /// （[`Self::multi_host_configuration_file`]）里两个键各自写了的抄。交回要接在配置后面的那几行（一个键一行，没得抄是空串）。
    ///
    /// # Errors
    /// 同 [`Self::multi_host_configuration_file`]。
    pub fn card_priority_lines(&self, repository_root: &Path) -> Result<String, String> {
        if let Some(text) = &self.cards_by_priority {
            if !text.trim().is_empty() {
                return Ok(format!(
                    "{CARDS_BY_PRIORITY_KEY}='{text}'\n{CARDS_BY_PRIORITY_WHEN_QUIESCED_KEY}='{text}'\n"
                ));
            }
        }
        let Some(configuration) = self.multi_host_configuration_file(repository_root)? else {
            return Ok(String::new());
        };
        Ok([CARDS_BY_PRIORITY_KEY, CARDS_BY_PRIORITY_WHEN_QUIESCED_KEY]
            .iter()
            .filter_map(|key| {
                configuration_value(&configuration, key)
                    .filter(|value| !value.trim().is_empty())
                    .map(|value| format!("{key}='{value}'\n"))
            })
            .collect())
    }
}

/// 用例写开 GPU 的多机配置时抄的显卡先后：调用方给这个进程的那一份（[`GpuConfigurationOfTheCaller::card_priority_lines`]），
/// 在不在环境守卫里调、别的用例此刻改没改着环境都一样。经 `run_from_environment` 跑、自己写配置的用例都走这一个口子。
///
/// # Errors
/// 同 [`GpuConfigurationOfTheCaller::multi_host_configuration_file`]。
pub fn card_priority_lines_of_the_caller(repository_root: &Path) -> Result<String, String> {
    GpuConfigurationOfTheCaller::of_this_process().card_priority_lines(repository_root)
}

/// 多机配置的文件名（git 忽略、每台机器各一份；模板是仓根的 `multi-host.env.example`）。
pub const MULTI_HOST_CONFIGURATION_FILE_NAME: &str = "multi-host.env";

/// 多机配置在哪：`SINGLEFS_MULTI_HOST_CONFIG` 设了照它（相对路径按仓根解析），文件要在；没设按 `repository_root` 的 git common-dir
/// 找主工作树的根（common-dir 的上一层）下的 [`MULTI_HOST_CONFIGURATION_FILE_NAME`]——在 HEAD + 暂存区的临时 worktree 里跑也读主工作树
/// 那一份（判决 J11、M4 第 4 条）；`repository_root` 不是 git 工作树（取不到 common-dir）时退回它自己下面那一份。没设时文件不在是 `None`
/// （两个开关都关、一张卡都没写）。
///
/// # Errors
/// `SINGLEFS_MULTI_HOST_CONFIG` 设了而那份文件不在。
pub fn multi_host_configuration_file(repository_root: &Path) -> Result<Option<PathBuf>, String> {
    multi_host_configuration_file_given(
        std::env::var_os(MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE).map(PathBuf::from),
        repository_root,
        git_common_directory_of(repository_root),
    )
}

/// [`multi_host_configuration_file`] 去掉读环境与问 git 的那一层。
///
/// # Errors
/// 给了 `configured` 而那份文件不在。
pub fn multi_host_configuration_file_given(
    configured: Option<PathBuf>,
    repository_root: &Path,
    git_common_directory: Option<PathBuf>,
) -> Result<Option<PathBuf>, String> {
    if let Some(configured) = configured {
        let configuration = if configured.is_absolute() {
            configured
        } else {
            repository_root.join(configured)
        };
        if !configuration.is_file() {
            return Err(format!(
                "SINGLEFS_MULTI_HOST_CONFIG 指的配置不在：{}",
                configuration.display()
            ));
        }
        return Ok(Some(configuration));
    }
    let configuration = match git_common_directory.as_deref().and_then(Path::parent) {
        Some(main_worktree_root) => main_worktree_root.join(MULTI_HOST_CONFIGURATION_FILE_NAME),
        None => repository_root.join(MULTI_HOST_CONFIGURATION_FILE_NAME),
    };
    Ok(configuration.is_file().then_some(configuration))
}

/// `git -C <repository_root> rev-parse --path-format=absolute --git-common-dir` 交回的目录；不是 git 工作树、git 起不来是 `None`。
fn git_common_directory_of(repository_root: &Path) -> Option<PathBuf> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(repository_root)
        .args(["rev-parse", "--path-format=absolute", "--git-common-dir"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?;
    let directory = text.trim();
    (!directory.is_empty()).then(|| PathBuf::from(directory))
}

/// 配置点名的卡里哪几张（序号）不在用得上的卡里，按配置的次序。
#[must_use]
pub fn configured_cards_not_usable(configured: &[(u32, u64)], usable: &[u32]) -> Vec<u32> {
    configured
        .iter()
        .map(|(index, _)| *index)
        .filter(|index| !usable.contains(index))
        .collect()
}

/// 单测造的临时显卡配置点名的卡（[`while_another_thread_holds_a_temporary_gpu_configuration`]）：调用方的配置里不会有这一张。
#[cfg(all(test, feature = "verdict-store"))]
pub(crate) const CARDS_OF_THE_TEMPORARY_GPU_CONFIGURATION: &str = "peer:4093:1M";

/// 单测造 E5 报告第三节第 3 条那种交错，确定地造：另一条线程拿着改环境的锁（`crate::crash_amplification::CrashAmplificationEnvironment`），
/// 把多机配置换成一份临时的（开 GPU、两个显卡先后的键都点 [`CARDS_OF_THE_TEMPORARY_GPU_CONFIGURATION`]），显卡先后的环境变量也点它、
/// 这台机器的角色设成 `peer`；设好之后这边才跑 `work`，`work` 跑完那条线程才还原、放锁。交回 `work` 的结果与那份临时配置的路径
/// （跑完已删）。
#[cfg(all(test, feature = "verdict-store"))]
pub(crate) fn while_another_thread_holds_a_temporary_gpu_configuration<Outcome>(
    tag: &str,
    work: impl FnOnce() -> Outcome,
) -> (Outcome, PathBuf) {
    let directory = crate::crash_library::ExclusiveTemporaryDirectory::create(&format!(
        "temporary-gpu-configuration-{tag}"
    ))
    .expect("建得了一次性临时目录");
    let temporary_configuration = directory.join("temporary-multi-host.env");
    std::fs::write(
        &temporary_configuration,
        format!(
            "{GPU_SWITCH_KEY}=1\n{CARDS_BY_PRIORITY_KEY}='{CARDS_OF_THE_TEMPORARY_GPU_CONFIGURATION}'\n\
             {CARDS_BY_PRIORITY_WHEN_QUIESCED_KEY}='{CARDS_OF_THE_TEMPORARY_GPU_CONFIGURATION}'\n"
        ),
    )
    .expect("写得了临时配置");
    let (temporary_configuration_is_set, wait_until_it_is_set) = std::sync::mpsc::channel::<()>();
    let (work_is_done, wait_until_the_work_is_done) = std::sync::mpsc::channel::<()>();
    let configuration_of_the_holder = temporary_configuration.clone();
    let holder = std::thread::spawn(move || {
        let _environment =
            crate::crash_amplification::CrashAmplificationEnvironment::cleared_then_set(&[
                (
                    MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE,
                    configuration_of_the_holder.as_os_str(),
                ),
                (
                    CARDS_BY_PRIORITY_KEY,
                    std::ffi::OsStr::new(CARDS_OF_THE_TEMPORARY_GPU_CONFIGURATION),
                ),
                (HOST_ROLE_ENVIRONMENT_VARIABLE, std::ffi::OsStr::new("peer")),
            ]);
        temporary_configuration_is_set
            .send(())
            .expect("这边等着这一声才跑");
        wait_until_the_work_is_done.recv().expect("这边跑完才放");
    });
    wait_until_it_is_set
        .recv()
        .expect("拿着锁的那条线程设好了临时配置");
    let outcome = work();
    work_is_done.send(()).expect("拿着锁的那条线程在等");
    holder.join().expect("拿着锁的那条线程没 panic");
    drop(directory);
    (outcome, temporary_configuration)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temporary_directory(tag: &str) -> crate::crash_library::ExclusiveTemporaryDirectory {
        crate::crash_library::ExclusiveTemporaryDirectory::create(&format!(
            "gpu-card-configuration-{tag}"
        ))
        .expect("建得了一次性临时目录")
    }

    /// 显卡先后从多机配置读（判决 J5、J11）：清过场取清场之后那一份，没写它退回不清场那一份；没清场取不清场那一份；
    /// 环境变量 `GPU_CARDS_BY_PRIORITY` 给了就照它（驱动挑好的那一份）。
    #[test]
    fn the_cards_by_priority_come_from_the_multi_host_configuration_by_the_quiesce() {
        let directory = temporary_directory("cards-by-priority");
        let both = directory.join("both.env");
        std::fs::write(
            &both,
            "ENABLE_GPU=1\nGPU_CARDS_BY_PRIORITY='local:1:8G'\nGPU_CARDS_BY_PRIORITY_WHEN_QUIESCED='local:0:28G local:1:13G'\n",
        )
        .expect("写得了");
        let only_the_one_not_quiesced = directory.join("only-not-quiesced.env");
        std::fs::write(
            &only_the_one_not_quiesced,
            "GPU_CARDS_BY_PRIORITY='local:1:8G'\n",
        )
        .expect("写得了");
        let card_count = |result: Result<Option<Vec<CardByPriority>>, String>| {
            result.expect("写法对").map(|cards| cards.len())
        };
        assert_eq!(
            card_count(cards_by_priority_from(
                None,
                Some(both.as_path()),
                GpuQuiesce::QuiescedByTheWrapper
            )),
            Some(2),
            "清过场取清场之后那一份"
        );
        assert_eq!(
            card_count(cards_by_priority_from(
                None,
                Some(both.as_path()),
                GpuQuiesce::NotQuiesced
            )),
            Some(1),
            "没清场取不清场那一份"
        );
        assert_eq!(
            card_count(cards_by_priority_from(
                None,
                Some(only_the_one_not_quiesced.as_path()),
                GpuQuiesce::NotQuiesced
            )),
            Some(1),
            "没清场取不清场那一份"
        );
        assert_eq!(
            card_count(cards_by_priority_from(
                Some("local:0:4G peer:1:4G peer:2:4G".to_string()),
                Some(both.as_path()),
                GpuQuiesce::QuiescedByTheWrapper
            )),
            Some(3),
            "环境变量给了就照它"
        );
        assert_eq!(
            cards_by_priority_from(None, None, GpuQuiesce::QuiescedByTheWrapper),
            Ok(None),
            "没有多机配置、没给环境变量：没配显卡先后"
        );
        drop(directory);
    }

    /// 清过场而多机配置只写了不清场那一份显卡先后：报错、点名两个键，不退回不清场那一份（「配了 GPU 卡用不上退 78、不降级」，
    /// 与 `crash-amplification-lib.sh` 同一处拒）；GPU 没开、两份都没写是没配；环境变量给了照它。
    #[test]
    fn quiesced_with_only_the_cards_by_priority_not_quiesced_written_is_refused() {
        let directory = temporary_directory("quiesced-without-its-cards");
        let only_the_one_not_quiesced = directory.join("only-not-quiesced.env");
        std::fs::write(
            &only_the_one_not_quiesced,
            "ENABLE_GPU=1\nGPU_CARDS_BY_PRIORITY='local:1:8G'\n",
        )
        .expect("写得了");
        let neither_without_the_gpu = directory.join("neither-without-the-gpu.env");
        std::fs::write(&neither_without_the_gpu, "ENABLE_GPU=0\n").expect("写得了");
        let refusal = cards_by_priority_from(
            None,
            Some(only_the_one_not_quiesced.as_path()),
            GpuQuiesce::QuiescedByTheWrapper,
        )
        .expect_err("清过场而没写清场之后那一份：报错");
        assert!(
            refusal.contains(CARDS_BY_PRIORITY_WHEN_QUIESCED_KEY)
                && refusal.contains(CARDS_BY_PRIORITY_KEY),
            "报错点名两个键：{refusal}"
        );
        assert_eq!(
            cards_by_priority_from(
                None,
                Some(neither_without_the_gpu.as_path()),
                GpuQuiesce::QuiescedByTheWrapper
            ),
            Ok(None),
            "GPU 没开、两份都没写：没配显卡先后，不报错"
        );
        assert_eq!(
            cards_by_priority_from(
                Some("local:0:4G".to_string()),
                Some(only_the_one_not_quiesced.as_path()),
                GpuQuiesce::QuiescedByTheWrapper
            )
            .map(|cards| cards.map(|cards| cards.len())),
            Ok(Some(1)),
            "环境变量给了照它（驱动挑好的那一份）"
        );
        drop(directory);
    }

    /// 清场之后两份显卡先后都没写而多机配置开了 GPU（只写 `ENABLE_GPU=1` 的那种）：报错、点名两个键与 GPU 开关，不照本机空闲显存挑卡
    /// （脚本 `crash_amplification_prepare` 开了 GPU 而清场之后那一份没写时也拒，不管不清场那一份写没写；照「配了 GPU 卡用不上退 78、不降级」）；
    /// 同一份配置没清场时照旧是没配（本机能用的卡照空闲显存用）。
    #[test]
    fn quiesced_with_the_gpu_enabled_and_neither_cards_by_priority_written_is_refused() {
        let directory = temporary_directory("quiesced-gpu-without-any-cards");
        let only_the_gpu_switch = directory.join("only-the-gpu-switch.env");
        std::fs::write(&only_the_gpu_switch, "ENABLE_GPU=1\n").expect("写得了");
        let refusal = cards_by_priority_from(
            None,
            Some(only_the_gpu_switch.as_path()),
            GpuQuiesce::QuiescedByTheWrapper,
        )
        .expect_err("清过场、开了 GPU、两份显卡先后都没写：报错");
        assert!(
            refusal.contains(CARDS_BY_PRIORITY_WHEN_QUIESCED_KEY)
                && refusal.contains(GPU_SWITCH_KEY),
            "报错点名清场之后那个键与 GPU 开关：{refusal}"
        );
        assert_eq!(
            cards_by_priority_from(
                None,
                Some(only_the_gpu_switch.as_path()),
                GpuQuiesce::NotQuiesced
            ),
            Ok(None),
            "没清场：没配显卡先后，不报错"
        );
        drop(directory);
    }

    /// 多机配置在哪（判决 J11、M4 第 4 条）：给了 `SINGLEFS_MULTI_HOST_CONFIG` 照它（相对路径按仓根，文件不在报错）；没给按 git common-dir
    /// 找主工作树的根下那一份（worktree 里也读主工作树的），取不到 common-dir 退回仓根下那一份；文件不在是 `None`。
    #[test]
    fn the_multi_host_configuration_is_the_one_configured_or_the_one_of_the_main_worktree() {
        let directory = temporary_directory("multi-host");
        let main_worktree = directory.join("main");
        let worktree = directory.join("worktree");
        std::fs::create_dir_all(main_worktree.join(".git")).expect("建得了");
        std::fs::create_dir_all(&worktree).expect("建得了");
        std::fs::write(
            main_worktree.join(MULTI_HOST_CONFIGURATION_FILE_NAME),
            "ENABLE_GPU=1\n",
        )
        .expect("写得了");
        assert_eq!(
            multi_host_configuration_file_given(None, &worktree, Some(main_worktree.join(".git"))),
            Ok(Some(main_worktree.join(MULTI_HOST_CONFIGURATION_FILE_NAME))),
            "worktree 里读主工作树那一份"
        );
        assert_eq!(
            multi_host_configuration_file_given(None, &worktree, None),
            Ok(None),
            "不是 git 工作树、它自己下面没有那一份"
        );
        assert_eq!(
            multi_host_configuration_file_given(None, &main_worktree, None),
            Ok(Some(main_worktree.join(MULTI_HOST_CONFIGURATION_FILE_NAME))),
            "取不到 common-dir 退回仓根下那一份"
        );
        assert_eq!(
            multi_host_configuration_file_given(
                Some(PathBuf::from(MULTI_HOST_CONFIGURATION_FILE_NAME)),
                &main_worktree,
                None
            ),
            Ok(Some(main_worktree.join(MULTI_HOST_CONFIGURATION_FILE_NAME))),
            "相对路径按仓根解析"
        );
        assert!(
            multi_host_configuration_file_given(
                Some(directory.join("missing.env")),
                &worktree,
                None
            )
            .is_err_and(|error| error.contains("SINGLEFS_MULTI_HOST_CONFIG")),
            "给了而不在报错"
        );
        drop(directory);
    }

    /// E5 报告第三节第 3 条（用例在拿进程级环境锁之前读本机配置，读到的是拿着锁的别的用例换上的临时配置），确定地造出那种交错
    /// （[`while_another_thread_holds_a_temporary_gpu_configuration`]：那条线程设好之后才抄、抄完才放）：用例抄的显卡先后
    /// （[`card_priority_lines_of_the_caller`]，经 `run_from_environment` 跑、自己写开 GPU 配置的四份测试文件都走它）仍是调用方给这个进程的那一份，
    /// 不是那条线程的临时配置。不要卡。
    #[cfg(feature = "verdict-store")]
    #[test]
    fn the_card_priority_lines_copied_while_another_thread_holds_a_temporary_configuration_are_those_of_the_caller(
    ) {
        let repository_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("包目录在 crates/ 下，往上两层是仓根");
        let (copied, temporary_configuration) =
            while_another_thread_holds_a_temporary_gpu_configuration("copied-lines", || {
                card_priority_lines_of_the_caller(repository_root)
            });
        let copied = copied.expect("调用方给这个进程的多机配置读得了");
        assert!(
            !copied.contains(CARDS_OF_THE_TEMPORARY_GPU_CONFIGURATION),
            "抄的是拿着锁那条线程换上的临时配置 {}：{copied}",
            temporary_configuration.display()
        );
        assert_eq!(
            copied,
            GpuConfigurationOfTheCaller::of_this_process()
                .card_priority_lines(repository_root)
                .expect("调用方给这个进程的多机配置读得了"),
            "抄的是调用方给这个进程的那一份"
        );
    }
}
