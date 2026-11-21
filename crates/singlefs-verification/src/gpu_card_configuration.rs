//! 显卡配置怎么读：`GPU_CARDS_BY_PRIORITY` 点名的卡（在 `HOSTS` 里哪个地址那一台、`nvidia-smi` 序号、显存额度）、这一台是 `HOSTS` 里哪一项、
//! `键=值` 配置文件里一个键的值、点名的卡里哪几张用不上。崩溃放量取卡（`crate::crash_amplification`）与 GPU 用例共用的卡池
//! （`crate::gpu_unit_checks::gpu_test_start`，随 `gpu` 编）都照它挑卡，所以单放一份、不挂特性；
//! `crate::crash_amplification` 再 `pub use` 回去，它的调用方与单测照旧经它引。
//! 显卡的两个键写在多机配置那一份里（多机配置合成一份，不零散配置；判决
//! `research/prompts/m3-crash-enumeration-merge-r3-main-verification.md` 第三节 J5、J11）：`GPU_CARDS_BY_PRIORITY`（不清场那一份）与
//! `GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED`（清场之后）：清没清场，这里两份都只看写没写、不照它们挑卡（[`cards_by_priority_from`]）；卡归这一台的 GPU 批处理验证服务，
//! 环境变量交了卡串照它；验证包的测试与判段不借卡，调用方没交卡串（没设、空串或只有空白，[`card_string_handed_over_by_the_service`]）就当没有卡
//! （[`GpuConfigurationOfTheCaller::cards_for_the_gpu_tests`]）；
//! 借卡客户端（[`cards_handed_over_or_leased_on_demand`]）留给别的包里 GPU 服务借卡白名单放行的装置；多机配置在哪照 [`multi_host_configuration_file`]（与
//! `scripts/crash/crash-amplification-lib.sh` 的 `crash_amplification_multi_host_configuration_file` 同一种找法）。GPU 开关与两个显卡键只写在多机配置里。
//! 两台对称，没有「主机角色」：卡项点的是 `HOSTS` 里的地址，这一台是 `HOSTS` 里哪一项只由多机配置判法
//! `scripts/crash/crash-point-replay-shard-configuration-check.sh --emit-addresses` 认（这一台网卡上的地址与 `HOSTS` 求交、恰好一项；全仓只这一份判法，
//! 这里起它、读它打的 `THIS_HOST_ADDRESS`，不另判，[`MultiHostAddresses::emitted_by_the_configuration_check`]）。

use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// 主机数组 `HOSTS` 里的一项：多机配置判法认过的一台机器的有线直连段地址（IPv4 点分或 IPv6，照判法打的原样）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostAddress(String);

impl HostAddress {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 多机配置判法 `--emit-addresses` 认出的主机数组与这一台。只从判法打的那几行来（[`Self::emitted_by_the_configuration_check`]），
/// 构造时核过：主机数组至少一项，这一台是其中一项。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MultiHostAddresses {
    hosts: Vec<HostAddress>,
    this_host: HostAddress,
}

impl MultiHostAddresses {
    #[must_use]
    pub fn hosts(&self) -> &[HostAddress] {
        &self.hosts
    }

    #[must_use]
    pub fn this_host(&self) -> &HostAddress {
        &self.this_host
    }

    /// 起多机配置判法 `bash <仓根>/scripts/crash/crash-point-replay-shard-configuration-check.sh --emit-addresses --configuration <configuration_file>`
    /// （仓根是编这份二进制的那棵树），读它打的 `HOSTS`、`THIS_HOST_ADDRESS` 两行。判法整份判那份配置、不连任何一台、不看双机开关。
    ///
    /// # Errors
    /// 判法起不来；判法判不过（退出码不是 0：`HOSTS` 没写或写坏、这一台网卡上的地址在 `HOSTS` 里不是恰好一项、配置别处写坏，原话带上）；打的行认不出。
    pub fn emitted_by_the_configuration_check(configuration_file: &Path) -> Result<Self, String> {
        // 判法从编这份二进制的那棵树里起，与编进来的读法同一版；拼成包目录接字面路径，判法这份文件的字节随这一处进单元键
        let script = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../scripts/crash/crash-point-replay-shard-configuration-check.sh");
        let output = std::process::Command::new("bash")
            .arg(&script)
            .arg("--emit-addresses")
            .arg("--configuration")
            .arg(configuration_file)
            .stdin(std::process::Stdio::null())
            .output()
            .map_err(|error| {
                format!(
                    "起不了多机配置判法 bash {} --emit-addresses：{error}（这一台是 HOSTS 里哪一项只由它认）",
                    script.display()
                )
            })?;
        let printed = String::from_utf8_lossy(&output.stdout);
        if !output.status.success() {
            let printed_on_standard_error = String::from_utf8_lossy(&output.stderr);
            let reason: Vec<&str> = printed
                .lines()
                .chain(printed_on_standard_error.lines())
                .map(str::trim)
                .filter(|line| line.starts_with('✗') || line.starts_with('→'))
                .collect();
            return Err(format!(
                "多机配置判法认不出这一台（bash {} --emit-addresses --configuration {} 退 {}）：{}",
                script.display(),
                configuration_file.display(),
                output
                    .status
                    .code()
                    .map_or_else(|| "被信号杀掉".to_string(), |code| code.to_string()),
                reason.join(" ")
            ));
        }
        Self::from_the_emitted_assignments(&printed)
    }

    /// 判法打的 `KEY=<printf %q 的值>` 几行认成主机数组与这一台（`PEER_HOST_ADDRESS` 那一行不用）。
    ///
    /// # Errors
    /// 少了 `HOSTS` 或 `THIS_HOST_ADDRESS` 那一行、值里有认不出的写法、`HOSTS` 是空的、`THIS_HOST_ADDRESS` 不是 `HOSTS` 里的一项。
    pub(crate) fn from_the_emitted_assignments(printed: &str) -> Result<Self, String> {
        let emitted_value = |key: &str| -> Result<String, String> {
            let quoted = printed
                .lines()
                .find_map(|line| line.strip_prefix(key)?.strip_prefix('='))
                .ok_or_else(|| format!("多机配置判法没打 {key} 那一行（打的是「{printed}」）"))?;
            value_printed_by_printf_quoted(quoted)
                .map_err(|reason| format!("多机配置判法打的 {key}={quoted} 认不出：{reason}"))
        };
        let hosts: Vec<HostAddress> = emitted_value("HOSTS")?
            .split_whitespace()
            .map(|address| HostAddress(address.to_string()))
            .collect();
        let this_host = HostAddress(emitted_value("THIS_HOST_ADDRESS")?);
        if !hosts.contains(&this_host) {
            return Err(format!(
                "多机配置判法打的这一台「{}」不是 HOSTS（{}）里的一项",
                this_host.as_str(),
                joined_host_addresses(&hosts)
            ));
        }
        Ok(Self { hosts, this_host })
    }
}

/// 几个地址空格隔开（报错里抄 `HOSTS` 用）。
fn joined_host_addresses(hosts: &[HostAddress]) -> String {
    hosts
        .iter()
        .map(HostAddress::as_str)
        .collect::<Vec<_>>()
        .join(" ")
}

/// bash `printf %q` 打出来的一个值还原成原样：判法只打地址（十六进制数字、点、冒号）与隔开它们的空格，`%q` 只给空格前面加反斜杠，空串打成 `''`；
/// 别的引号写法（`'…'`、`$'…'`）判法不会打给这几行，认不出就报错，不猜。
fn value_printed_by_printf_quoted(quoted: &str) -> Result<String, String> {
    if quoted == "''" {
        return Ok(String::new());
    }
    if quoted.starts_with('\'') || quoted.starts_with("$'") {
        return Err("引号写法不是判法给地址打的那一种".to_string());
    }
    let mut value = String::with_capacity(quoted.len());
    let mut characters = quoted.chars();
    while let Some(character) = characters.next() {
        if character == '\\' {
            value.push(
                characters
                    .next()
                    .ok_or_else(|| "结尾多了一个反斜杠".to_string())?,
            );
        } else {
            value.push(character);
        }
    }
    Ok(value)
}

/// [`multi_host_addresses_of_the_caller`] 判出的那一份：进程里只判一次。
static MULTI_HOST_ADDRESSES_OF_THE_CALLER: std::sync::OnceLock<Result<MultiHostAddresses, String>> =
    std::sync::OnceLock::new();

/// 调用方给这个进程的那份多机配置（[`GpuConfigurationOfTheCaller::multi_host_configuration_file`]，从这个进程的当前目录起找）上，
/// 多机配置判法认出的主机数组与这一台（[`MultiHostAddresses::emitted_by_the_configuration_check`]）：进程里只判一次，之后交同一份。
/// 用例在环境守卫里换上的临时配置只写开关与显卡先后、不写全多机配置的键，判法整份判不过；这一台是哪台、`HOSTS` 有哪几项是这对机器的事，
/// 只认调用方那一份（与卡池只认调用方那份配置同一个道理，[`GpuConfigurationOfTheCaller`]）。
///
/// # Errors
/// 取不到当前目录；调用方那一份设了路径而文件不在，或找不到多机配置（`HOSTS` 只写在多机配置里，没写就是没有）；判法认不出这一台。
pub fn multi_host_addresses_of_the_caller() -> Result<MultiHostAddresses, String> {
    MULTI_HOST_ADDRESSES_OF_THE_CALLER
        .get_or_init(|| {
            let current_directory = std::env::current_dir().map_err(|error| {
                format!("取不到这个进程的当前目录（多机配置从这里起找）：{error}")
            })?;
            let configuration = GpuConfigurationOfTheCaller::of_this_process()
                .multi_host_configuration_file(&current_directory)?
                .ok_or_else(|| {
                    format!(
                        "找不到多机配置（{MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE} 没设，从 {} 起找、主工作树根下也没有 {MULTI_HOST_CONFIGURATION_FILE_NAME}）：\
                         显卡先后点名的地址要对多机配置的 HOSTS，这一台是 HOSTS 里哪一项也只由多机配置判法认，没写就是没有。\
                         出路：照仓根 multi-host.env.example 写一份，或设 {MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE} 指到它",
                        current_directory.display()
                    )
                })?;
            MultiHostAddresses::emitted_by_the_configuration_check(&configuration)
        })
        .clone()
}

/// 配置点名的一张卡（`GPU_CARDS_BY_PRIORITY` 里的一项）：在 `HOSTS` 里哪个地址那一台、`nvidia-smi` 的序号、给崩溃放量的显存额度。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardByPriority {
    pub host: HostAddress,
    pub index: u32,
    pub memory_quota_mebibytes: u64,
}

/// 配置点名的卡（[`parse_cards_by_priority`] 读出来、每项的地址都在 `HOSTS` 里），连同这一台是 `HOSTS` 里哪一项：选卡只取这一台那几项。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfiguredCards {
    this_host: HostAddress,
    cards: Vec<CardByPriority>,
}

impl ConfiguredCards {
    #[must_use]
    pub fn this_host(&self) -> &HostAddress {
        &self.this_host
    }

    /// 两台的卡都在里面，按配置的次序。
    #[must_use]
    pub fn cards(&self) -> &[CardByPriority] {
        &self.cards
    }

    /// 这一台的那几项（序号，额度 MiB），按配置的次序。
    #[must_use]
    pub fn cards_of_this_host(&self) -> Vec<(u32, u64)> {
        self.cards
            .iter()
            .filter(|card| card.host == self.this_host)
            .map(|card| (card.index, card.memory_quota_mebibytes))
            .collect()
    }
}

/// 读 `GPU_CARDS_BY_PRIORITY` 的值：空白隔开的若干项，每项 `<HOSTS 里的地址>:<nvidia-smi 序号>:<显存额度>`（例 `192.0.2.1:0:32G`），
/// 从右边切两刀（地址可以是 IPv6、自带冒号）；地址要逐字是 `addresses` 主机数组里的一项；额度是正整数加 `G` 或 `M`（1024 进制）；
/// 写在前面的优先级高，任务先装它、装满了才给后面的。
///
/// # Errors
/// 一项切不出三段、地址不在 `HOSTS` 里、序号或额度写法不对、同一张卡点名两次、一项都没有。
pub fn parse_cards_by_priority(
    text: &str,
    addresses: &MultiHostAddresses,
) -> Result<ConfiguredCards, String> {
    let mut cards: Vec<CardByPriority> = Vec::new();
    for entry in text.split_whitespace() {
        let malformed = || {
            format!(
                "显卡优先级的一项要写 <HOSTS 里的地址>:<序号>:<显存额度>（例 192.0.2.1:0:32G），实际「{entry}」"
            )
        };
        let (address_and_index, quota) = entry.rsplit_once(':').ok_or_else(malformed)?;
        let (address, index) = address_and_index.rsplit_once(':').ok_or_else(malformed)?;
        let host = addresses
            .hosts()
            .iter()
            .find(|host| host.as_str() == address)
            .cloned()
            .ok_or_else(|| {
                format!(
                    "显卡优先级的一项「{entry}」点的地址「{address}」不在多机配置的 HOSTS（{}）里，配置写坏了：\
                     每项写成 <HOSTS 里的地址>:<序号>:<显存额度>（例 192.0.2.1:0:32G），地址照 HOSTS 里的写法逐字抄",
                    joined_host_addresses(addresses.hosts())
                )
            })?;
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
    Ok(ConfiguredCards {
        this_host: addresses.this_host().clone(),
        cards,
    })
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
        if std::env::var_os(QUIESCE_ORIGIN_READ_BACK_ENVIRONMENT_VARIABLE)
            .is_some_and(|value| value == "1")
        {
            return Self::QuiescedByTheWrapper;
        }
        match std::env::var_os(QUIESCE_HOLDER_ENVIRONMENT_VARIABLE) {
            Some(holder) if Path::new(&holder).is_file() => Self::QuiescedByTheWrapper,
            Some(_) | None => Self::NotQuiesced,
        }
    }
}

/// 多机配置在哪的那个环境变量（[`multi_host_configuration_file`]）。
pub const MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_MULTI_HOST_CONFIG";
/// 清场的那一层给命令设的登记文件那个环境变量（[`GpuQuiesce::of_this_process`]）。
pub const QUIESCE_HOLDER_ENVIRONMENT_VARIABLE: &str = "GPU_QUIESCE_HOLDER";
/// 发起端已清场并回读后透传的环境变量（[`GpuQuiesce::of_this_process`]）。
pub const QUIESCE_ORIGIN_READ_BACK_ENVIRONMENT_VARIABLE: &str = "GPU_QUIESCE_ORIGIN_READ_BACK";

/// 借卡通道的三个环境变量：统一任务服务交给每次尝试的尝试标签、任务号、第几次尝试（`task-server/task_server.py` 起尝试时设）。
/// 三个都在、都不空就是有通道（[`CardLeaseChannelOfTheCaller::of_the_variable_values`]），原样交给借卡客户端。
pub const CARD_LEASE_ATTEMPT_ENVIRONMENT_VARIABLES: [&str; 3] = [
    "TASK_SERVER_ATTEMPT_LABEL",
    "TASK_SERVER_TASK",
    "TASK_SERVER_ATTEMPT",
];
/// 统一任务服务每次尝试交给命令的「提交方那一份仓的根」那个环境变量（`task-server/task_server_lib.py`）：借卡客户端取它下面的
/// [`CARD_LEASE_CLIENT_PATH_IN_THE_REPOSITORY`]（[`card_lease_client_from`]）。
pub const TASK_SERVER_SOURCE_REPOSITORY_ENVIRONMENT_VARIABLE: &str =
    "TASK_SERVER_SOURCE_REPOSITORY";
/// 借卡客户端在仓里的路径（从仓根起）：GPU 批处理验证服务的控制命令，借卡是它的 `lease take`，还卡是 `lease return <卡串>`。
pub const CARD_LEASE_CLIENT_PATH_IN_THE_REPOSITORY: &str = "gpu-verification-server/control";
/// 起借卡客户端的解释器：客户端是 python 脚本，照 `python3 <客户端> lease …` 起，不靠它的执行位。
pub const CARD_LEASE_CLIENT_INTERPRETER: &str = "python3";
/// 借卡客户端认借的一方的那个环境变量：`<借的进程号>:<它的启动刻>`（[`CardLeaseBorrower`]）。还卡那一条在借的进程退出之后才起，
/// 那时它的父进程已经换了，客户端只认这个变量、不看自己的父进程。
pub const CARD_LEASE_BORROWER_ENVIRONMENT_VARIABLE: &str = "GPU_CARD_LEASE_BORROWER";

/// 调用方给这个进程的借卡通道：统一任务服务交给每次尝试的三个环境变量（[`CARD_LEASE_ATTEMPT_ENVIRONMENT_VARIABLES`]），
/// 三个都在、都不空才算有通道，经任务服务交的命令都有。有通道而环境里没有 `GPU_CARDS_BY_PRIORITY` 时，GPU 批处理验证服务借卡白名单放行的装置
/// （在别的包里）第一次真要卡的那一刻才向这一台的服务借（[`CardLeaseChannelOfTheCaller::cards_leased_by_this_process`]），
/// 编译、录入这些不上卡的段不持卡；验证包不在白名单里，它的判段取卡（[`cards_by_priority_from_configuration_or_environment`]、给定状态引擎）、
/// 卡池起池与 GPU 用例取卡（[`GpuConfigurationOfTheCaller::cards_for_the_gpu_tests`]）都不问通道、不借卡。
/// 认通道那一刻的多机配置路径（`SINGLEFS_MULTI_HOST_CONFIG`，设了才记）一并记下，借卡与还卡时照它交给客户端。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardLeaseChannelOfTheCaller {
    /// 交给借卡客户端的那三个变量：名字与调用方给的值原样。
    variables_for_the_client: Vec<(&'static str, OsString)>,
    /// 认通道那一刻的 `SINGLEFS_MULTI_HOST_CONFIG`（没设是 `None`，那时客户端的环境里也不设它）。
    multi_host_configuration_for_the_client: Option<OsString>,
}

impl CardLeaseChannelOfTheCaller {
    /// 照 `value_of`（按名字取一个环境变量的值）认通道：三个变量（[`CARD_LEASE_ATTEMPT_ENVIRONMENT_VARIABLES`]）都给了、都不空才是 `Some`；
    /// 别的变量给不给都不改结局。`SINGLEFS_MULTI_HOST_CONFIG` 给了就记下交给客户端。
    #[must_use]
    pub fn of_the_variable_values(value_of: &dyn Fn(&str) -> Option<OsString>) -> Option<Self> {
        let mut variables_for_the_client = Vec::new();
        for name in CARD_LEASE_ATTEMPT_ENVIRONMENT_VARIABLES {
            let value = value_of(name).filter(|value| !value.is_empty())?;
            variables_for_the_client.push((name, value));
        }
        Some(Self {
            variables_for_the_client,
            multi_host_configuration_for_the_client: value_of(
                MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE,
            ),
        })
    }

    /// 此刻进程环境里的那三个变量认出的通道。
    #[must_use]
    pub fn read_from_the_environment_now() -> Option<Self> {
        Self::of_the_variable_values(&|name| std::env::var_os(name))
    }

    /// 交给借卡客户端的那三个变量。
    #[must_use]
    pub fn variables_for_the_client(&self) -> &[(&'static str, OsString)] {
        &self.variables_for_the_client
    }

    /// 这个进程按需借到的卡（与 `GPU_CARDS_BY_PRIORITY` 同格式）：进程里只借一回，第一次问的时候才借（客户端在等卡时这里照等），
    /// 之后每次交同一份；这个进程退出时还（[`CardsLeasedUntilThisProcessExits`]）。被拒的那一回记下来，之后每次交同一句，不重试。
    ///
    /// # Errors
    /// 找不到借卡客户端（[`card_lease_client_from`]）；读不出这个进程的启动刻（[`CardLeaseBorrower::of_this_process`]）；
    /// 借卡没成（[`lease_cards_through_the_client`]）。
    pub fn cards_leased_by_this_process(&self) -> Result<&'static str, String> {
        CARDS_LEASED_BY_THIS_PROCESS
            .get_or_init(|| {
                let current_directory = std::env::current_dir().map_err(|error| {
                    format!("取不到这个进程的当前目录（借卡客户端从这里往上找）：{error}")
                })?;
                let client = card_lease_client_from(
                    std::env::var_os(TASK_SERVER_SOURCE_REPOSITORY_ENVIRONMENT_VARIABLE)
                        .map(PathBuf::from),
                    &current_directory,
                )?;
                let borrower = CardLeaseBorrower::of_this_process()?;
                let leased = lease_cards_through_the_client(&client, self, &borrower)?;
                eprintln!("GPU_CARDS_LEASED_ON_DEMAND cards={}", leased.cards());
                Ok(leased)
            })
            .as_ref()
            .map(CardsLeasedUntilThisProcessExits::cards)
            .map_err(Clone::clone)
    }
}

/// 借卡的一方是哪个进程：进程号连同它的启动刻（`/proc/<进程号>/stat` 第 22 项，开机以来的时钟滴答数），进程号被别的进程重用也分得开。
/// 交给借卡客户端的写法是 `<进程号>:<启动刻>`（[`CARD_LEASE_BORROWER_ENVIRONMENT_VARIABLE`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CardLeaseBorrower {
    process_identifier: u32,
    start_time_since_boot_in_clock_ticks: u64,
}

/// `/proc/<进程号>/stat` 里启动刻是第几项（从 1 数，`man 5 proc`）。
const START_TIME_FIELD_NUMBER_IN_THE_PROCESS_STATUS_LINE: usize = 22;
/// `/proc/<进程号>/stat` 里命令名（第 2 项，带括号、可以有空格与右括号）之后的头一项是第几项。
const FIRST_FIELD_NUMBER_AFTER_THE_COMMAND_NAME_IN_THE_PROCESS_STATUS_LINE: usize = 3;

impl CardLeaseBorrower {
    /// 这个进程：进程号取 [`std::process::id`]，启动刻读 `/proc/self/stat`。
    ///
    /// # Errors
    /// `/proc/self/stat` 读不了，或读出来认不出启动刻（[`start_time_since_boot_in_clock_ticks_from_the_process_status_line`]）。
    pub fn of_this_process() -> Result<Self, String> {
        let process_status_line = std::fs::read_to_string("/proc/self/stat").map_err(|error| {
            format!("读不了 /proc/self/stat（借卡客户端要这个进程的启动刻认借的一方）：{error}")
        })?;
        Ok(Self {
            process_identifier: std::process::id(),
            start_time_since_boot_in_clock_ticks:
                start_time_since_boot_in_clock_ticks_from_the_process_status_line(
                    &process_status_line,
                )?,
        })
    }

    /// 交给借卡客户端的写法：`<进程号>:<启动刻>`。
    #[must_use]
    pub fn value_for_the_client(&self) -> String {
        format!(
            "{}:{}",
            self.process_identifier, self.start_time_since_boot_in_clock_ticks
        )
    }
}

/// `/proc/<进程号>/stat` 那一行里的启动刻（第 22 项）：命令名（第 2 项）带括号，里面可以有空格与右括号，从最后一个右括号之后起数，
/// 那之后的头一项是第 3 项。
///
/// # Errors
/// 没有右括号；右括号之后不够 22 项；第 22 项不是数。
pub fn start_time_since_boot_in_clock_ticks_from_the_process_status_line(
    process_status_line: &str,
) -> Result<u64, String> {
    let (_, fields_after_the_command_name) = process_status_line
        .rsplit_once(')')
        .ok_or_else(|| format!("进程状态文件里没有命令名的右括号：「{process_status_line}」"))?;
    let start_time = fields_after_the_command_name
        .split_whitespace()
        .nth(
            START_TIME_FIELD_NUMBER_IN_THE_PROCESS_STATUS_LINE
                - FIRST_FIELD_NUMBER_AFTER_THE_COMMAND_NAME_IN_THE_PROCESS_STATUS_LINE,
        )
        .ok_or_else(|| {
            format!(
                "进程状态文件里不够 {START_TIME_FIELD_NUMBER_IN_THE_PROCESS_STATUS_LINE} 项：「{process_status_line}」"
            )
        })?;
    start_time.parse().map_err(|_| {
        format!(
            "进程状态文件第 {START_TIME_FIELD_NUMBER_IN_THE_PROCESS_STATUS_LINE} 项（启动刻）不是数：「{start_time}」"
        )
    })
}

/// 这个进程按需借卡那一回的结局：进程里只借一回（[`CardLeaseChannelOfTheCaller::cards_leased_by_this_process`]）。
static CARDS_LEASED_BY_THIS_PROCESS: std::sync::OnceLock<
    Result<CardsLeasedUntilThisProcessExits, String>,
> = std::sync::OnceLock::new();

/// 借到的卡，连同等着这个进程退出就还卡的那个子进程：子进程读自己的标准输入读到收尾才起客户端还卡，标准输入的写端只在这个进程手里，
/// 这个进程退出（正常结束、panic、被杀都算）写端就关。不靠析构、不靠退出钩子：测试二进制经 `std::process::exit` 退，静态量不析构。
/// 还卡那一条起的时候这个进程已经不在，客户端认借的一方只靠交给它的 [`CARD_LEASE_BORROWER_ENVIRONMENT_VARIABLE`]。
#[derive(Debug)]
pub struct CardsLeasedUntilThisProcessExits {
    cards: String,
    return_waiting_for_the_exit: std::process::Child,
}

impl CardsLeasedUntilThisProcessExits {
    /// 借到的卡串（与 `GPU_CARDS_BY_PRIORITY` 同格式）。
    #[must_use]
    pub fn cards(&self) -> &str {
        &self.cards
    }

    /// 不等进程退出，当场还：关掉还卡子进程的标准输入、等它起的客户端退出。交回客户端退没退 0。
    ///
    /// # Errors
    /// 等不到还卡子进程。
    pub fn return_the_cards_now(mut self) -> Result<bool, String> {
        drop(self.return_waiting_for_the_exit.stdin.take());
        self.return_waiting_for_the_exit
            .wait()
            .map(|status| status.success())
            .map_err(|error| format!("等不到还卡的子进程：{error}"))
    }
}

/// 借卡客户端在哪：`source_repository`（[`TASK_SERVER_SOURCE_REPOSITORY_ENVIRONMENT_VARIABLE`] 的值，任务服务交的仓根）给了就是它下面的
/// [`CARD_LEASE_CLIENT_PATH_IN_THE_REPOSITORY`]；没给（手起的、有通道而没交仓根）从 `start_directory` 起往上逐层找
/// （cargo 把测试进程的当前目录设成包目录，仓根在它上面）。
///
/// # Errors
/// 任务服务交的仓根下那一份不是文件；往上找到根都没有。
pub fn card_lease_client_from(
    source_repository: Option<PathBuf>,
    start_directory: &Path,
) -> Result<PathBuf, String> {
    if let Some(repository) = source_repository {
        let client = repository.join(CARD_LEASE_CLIENT_PATH_IN_THE_REPOSITORY);
        return if client.is_file() {
            Ok(client)
        } else {
            Err(format!(
                "{TASK_SERVER_SOURCE_REPOSITORY_ENVIRONMENT_VARIABLE} 点的仓根下没有借卡客户端：{} 不是文件。\
                 卡归 GPU 批处理验证服务，任务里要卡的进程经它的控制命令借。出路：从带 {CARD_LEASE_CLIENT_PATH_IN_THE_REPOSITORY} 的那份仓交这条命令",
                client.display()
            ))
        };
    }
    start_directory
        .ancestors()
        .map(|directory| directory.join(CARD_LEASE_CLIENT_PATH_IN_THE_REPOSITORY))
        .find(|client| client.is_file())
        .ok_or_else(|| {
            format!(
                "从 {} 往上找不到借卡客户端 {CARD_LEASE_CLIENT_PATH_IN_THE_REPOSITORY}。卡归 GPU 批处理验证服务，任务里要卡的进程经它的控制命令借。\
                 出路：在带这个控制命令的仓里起这条命令，或设 {TASK_SERVER_SOURCE_REPOSITORY_ENVIRONMENT_VARIABLE} 指到那份仓的根",
                start_directory.display()
            )
        })
}

/// 还卡子进程跑的 shell：标准输入读到收尾（借卡的进程退出），再起 `<解释器> <客户端> lease return <卡串>`
/// （`$0` 是解释器、`$1` 是客户端、`$2` 是卡串）。
const RETURN_THE_CARDS_WHEN_THE_STANDARD_INPUT_ENDS: &str =
    r#"while read -r line_never_written; do :; done; exec "$0" "$1" lease return "$2""#;

/// 借卡客户端的环境里拿掉的三个变量：改连哪一处 prune-server 的两个与临时服务的标记（`crate::crash_library` 那三个常量）。
/// 测试进程在临时 prune-server 底下（变异门禁、证红每一行起的那种）时它们指着临时服务；借卡客户端要连的是 GPU 批处理服务连的那一份真服务，
/// 拿掉之后它照平常跑的取法连（`XDG_RUNTIME_DIR` 下的 socket，不在走多机配置的 TCP 地址）。
pub const PRUNE_SERVER_CONNECTION_VARIABLES_KEPT_FROM_THE_CARD_LEASE_CLIENT: [&str; 3] = [
    crate::crash_library::PRUNE_SERVER_SOCKET_ENVIRONMENT_VARIABLE,
    crate::crash_library::PRUNE_SERVER_TCP_ADDRESS_ENVIRONMENT_VARIABLE,
    crate::crash_library::PRUNE_SERVER_TEMPORARY_SERVICE_SOCKET_ENVIRONMENT_VARIABLE,
];

/// 给起借卡客户端的命令（借、还、等着还的 shell 都算）设环境：通道的三个变量、借的一方（[`CARD_LEASE_BORROWER_ENVIRONMENT_VARIABLE`]），
/// 认通道那一刻设了多机配置路径的照它设、没设的从子进程环境里拿掉；改连哪一处 prune-server 的变量一律拿掉
/// （[`PRUNE_SERVER_CONNECTION_VARIABLES_KEPT_FROM_THE_CARD_LEASE_CLIENT`]：借卡固定落到真服务）。都照 `channel` 与 `borrower` 交，不靠此刻的进程环境。
fn give_the_client_the_environment_of_the_lease(
    command: &mut std::process::Command,
    channel: &CardLeaseChannelOfTheCaller,
    borrower: &CardLeaseBorrower,
) {
    for name in PRUNE_SERVER_CONNECTION_VARIABLES_KEPT_FROM_THE_CARD_LEASE_CLIENT {
        command.env_remove(name);
    }
    command.envs(
        channel
            .variables_for_the_client
            .iter()
            .map(|(name, value)| (*name, value.as_os_str())),
    );
    command.env(
        CARD_LEASE_BORROWER_ENVIRONMENT_VARIABLE,
        borrower.value_for_the_client(),
    );
    match &channel.multi_host_configuration_for_the_client {
        Some(configuration) => {
            command.env(MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE, configuration)
        }
        None => command.env_remove(MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE),
    };
}

/// 起借卡客户端的命令：`<解释器> <客户端>`，环境照 [`give_the_client_the_environment_of_the_lease`]，子命令由调用方接。
fn card_lease_client_command(
    client: &Path,
    channel: &CardLeaseChannelOfTheCaller,
    borrower: &CardLeaseBorrower,
) -> std::process::Command {
    let mut command = std::process::Command::new(CARD_LEASE_CLIENT_INTERPRETER);
    command.arg(client);
    give_the_client_the_environment_of_the_lease(&mut command, channel, borrower);
    command
}

/// 经 `client`（GPU 批处理验证服务的控制命令）向这一台的服务借一回卡：`python3 <客户端> lease take` 的标准输出整段去空白是卡串，
/// 客户端在等卡时往标准错误打的行照进这个进程的标准错误（日志照涨）；借到之后起还卡的子进程（[`CardsLeasedUntilThisProcessExits`]），
/// 它在这个进程退出时起 `python3 <客户端> lease return <卡串>`。环境照 `channel` 与 `borrower` 交（[`give_the_client_the_environment_of_the_lease`]）。
///
/// # Errors
/// 起不了客户端；客户端退非 0（借不成：原因在客户端的标准错误，不重试、不退回 CPU）；退 0 而没交卡串；起不了还卡的子进程（借到的当场还掉）。
pub fn lease_cards_through_the_client(
    client: &Path,
    channel: &CardLeaseChannelOfTheCaller,
    borrower: &CardLeaseBorrower,
) -> Result<CardsLeasedUntilThisProcessExits, String> {
    let taken = card_lease_client_command(client, channel, borrower)
        .args(["lease", "take"])
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::inherit())
        .output()
        .map_err(|error| {
            format!(
                "起不了借卡客户端 {CARD_LEASE_CLIENT_INTERPRETER} {}：{error}",
                client.display()
            )
        })?;
    if !taken.status.success() {
        return Err(format!(
            "按需借卡没成：{CARD_LEASE_CLIENT_INTERPRETER} {} lease take 退 {}；不重试、不退回 CPU（运行条件不满足，退 78）。\
             卡归 GPU 批处理验证服务，任务里要卡的进程向它借；借不成的原因在客户端往标准错误打的那几行，出路照那几行办",
            client.display(),
            taken
                .status
                .code()
                .map_or_else(|| "（被信号停了）".to_string(), |code| code.to_string())
        ));
    }
    let cards = String::from_utf8_lossy(&taken.stdout).trim().to_string();
    let return_the_cards_at_once = |cards_to_return: &str| {
        // 借到了而往下走不通：交回的卡串当场还，结局不改这一句错
        let _returned = card_lease_client_command(client, channel, borrower)
            .args(["lease", "return"])
            .arg(cards_to_return)
            .stdin(std::process::Stdio::null())
            .status();
    };
    if cards.is_empty() {
        return_the_cards_at_once(&cards);
        return Err(format!(
            "按需借卡没成：{CARD_LEASE_CLIENT_INTERPRETER} {} lease take 退 0 而没交卡串，已当场 lease return。\
             卡归 GPU 批处理验证服务；原因看客户端往标准错误打的那几行",
            client.display()
        ));
    }
    let mut command_returning_the_cards_at_the_exit = std::process::Command::new("sh");
    command_returning_the_cards_at_the_exit
        .arg("-c")
        .arg(RETURN_THE_CARDS_WHEN_THE_STANDARD_INPUT_ENDS)
        .arg(CARD_LEASE_CLIENT_INTERPRETER)
        .arg(client)
        .arg(&cards);
    give_the_client_the_environment_of_the_lease(
        &mut command_returning_the_cards_at_the_exit,
        channel,
        borrower,
    );
    match command_returning_the_cards_at_the_exit
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::inherit())
        .spawn()
    {
        Ok(return_waiting_for_the_exit) => Ok(CardsLeasedUntilThisProcessExits {
            cards,
            return_waiting_for_the_exit,
        }),
        Err(error) => {
            return_the_cards_at_once(&cards);
            Err(format!(
                "按需借卡借到了 {cards}，起不了等这个进程退出就还卡的子进程（sh）：{error}；借到的已当场还掉"
            ))
        }
    }
}

/// 这个进程的卡串从哪来：环境变量 `GPU_CARDS_BY_PRIORITY` 给了照它（外层已经借好交下来的），不借；
/// 没给而有借卡通道（经任务服务交的命令都有），照 `lease` 向 GPU 批处理验证服务借（真要卡的这一刻才借）；两样都没有是 `None`（没经服务给卡）。
///
/// # Errors
/// `lease` 的错。
pub fn cards_handed_over_or_leased_on_demand(
    environment_value: Option<String>,
    channel: Option<&CardLeaseChannelOfTheCaller>,
    lease: &dyn Fn(&CardLeaseChannelOfTheCaller) -> Result<String, String>,
) -> Result<Option<String>, String> {
    match (environment_value, channel) {
        (Some(text), Some(_) | None) => Ok(Some(text)),
        (None, Some(channel)) => lease(channel).map(Some),
        (None, None) => Ok(None),
    }
}

/// 服务交的卡串（环境变量 `GPU_CARDS_BY_PRIORITY` 的值）算不算交了：去掉两头空白之后不空才算，交回原样那一串；没设、设成空串或只有空白
/// 都是 `None`，与没交同样当这台机器没有卡，不当「本机能用的卡全用」（卡只归 GPU 批处理验证服务，`.claude/rules/verification.md`
/// 「卡只归 GPU 服务，任务不拿卡」）。读卡串的每一处（判段取卡、卡池起池、GPU 用例取卡、实验包里要卡的装置）都先过这一道。
#[must_use]
pub fn card_string_handed_over_by_the_service<CardString: AsRef<str>>(
    environment_value: Option<CardString>,
) -> Option<CardString> {
    environment_value.filter(|card_string| !card_string.as_ref().trim().is_empty())
}

/// 判段取卡读的显卡优先级（崩溃放量观察者那一段取卡，`crate::crash_amplification::gpu_cards_of_this_run_from_the_multi_host_configuration`）：
/// 环境变量 `GPU_CARDS_BY_PRIORITY` 交了卡串（[`card_string_handed_over_by_the_service`]）就照它；没交时不问借卡通道、不借卡（判段的结论归 GPU 批处理验证服务判），
/// 清没清场（[`GpuQuiesce::of_this_process`]）都不读多机配置（[`multi_host_configuration_file`]）里的卡（卡归 GPU 批处理验证服务）：
/// GPU 没开、两份显卡先后都没写是 `None`（没配显卡先后），开了或写了报错。
/// 写了卡的，`HOSTS` 与这一台取调用方那份多机配置上判法认出的（[`multi_host_addresses_of_the_caller`]）。
///
/// # Errors
/// 写了而写法不对或地址不在 `HOSTS` 里（[`parse_cards_by_priority`]）；写了卡而判法认不出这一台；
/// 没经服务给卡而多机配置开了 GPU 或写了两份显卡先后里的一份（[`cards_by_priority_from`]）。
pub fn cards_by_priority_from_configuration_or_environment(
    configuration_file: Option<&Path>,
) -> Result<Option<ConfiguredCards>, String> {
    let cards_handed_over_in_the_environment = std::env::var(CARDS_BY_PRIORITY_KEY).ok();
    cards_by_priority_from(
        cards_handed_over_in_the_environment,
        configuration_file,
        GpuQuiesce::of_this_process(),
        &multi_host_addresses_of_the_caller,
    )
}

/// [`cards_by_priority_from_configuration_or_environment`] 去掉读环境的那一层：环境变量的值、多机配置、清没清场、主机数组与这一台都由调用方给
/// （`addresses_of_the_hosts` 只在写了卡、要读卡项时调）。
/// 环境变量没交卡串（没经服务给卡：没设、空串或只有空白，[`card_string_handed_over_by_the_service`]）而多机配置开了 GPU（[`GPU_SWITCH_KEY`]）或写了两份显卡先后
/// （[`CARDS_BY_PRIORITY_KEY`]、[`CARDS_BY_PRIORITY_WHEN_QUIESCED_KEY`]）里的一份：清没清场都报错（运行条件不满足，退 78），
/// 不照多机配置自己挑卡，也不照本机空闲显存挑卡（卡归 GPU 批处理验证服务，`.claude/rules/verification.md`「卡只归 GPU 服务，任务不拿卡」）；
/// 出路写卡只归它：判段的结论交它判，判段之外用卡的测试交它代跑，验证包的测试与判段不借卡；GPU 没开、两份都没写的是 `None`。
///
/// # Errors
/// 写了而写法不对或地址不在 `HOSTS` 里（[`parse_cards_by_priority`]）；写了卡而 `addresses_of_the_hosts` 报错；
/// 没经服务给卡而开了 GPU 或写了两份显卡先后里的一份。
pub fn cards_by_priority_from(
    environment_value: Option<String>,
    configuration_file: Option<&Path>,
    quiesce: GpuQuiesce,
    addresses_of_the_hosts: &dyn Fn() -> Result<MultiHostAddresses, String>,
) -> Result<Option<ConfiguredCards>, String> {
    let configured_value =
        |key: &str| configuration_file.and_then(|path| configuration_value(path, key));
    let written =
        |value: &Option<String>| value.as_deref().is_some_and(|text| !text.trim().is_empty());
    let configured_gpu_is_enabled = || {
        configured_value(GPU_SWITCH_KEY).is_some_and(|value| value.trim().trim_matches('"') == "1")
    };
    let text = match (
        card_string_handed_over_by_the_service(environment_value),
        quiesce,
    ) {
        (Some(text), GpuQuiesce::QuiescedByTheWrapper | GpuQuiesce::NotQuiesced) => Some(text),
        (None, GpuQuiesce::QuiescedByTheWrapper | GpuQuiesce::NotQuiesced) => {
            if configured_gpu_is_enabled()
                || written(&configured_value(CARDS_BY_PRIORITY_KEY))
                || written(&configured_value(CARDS_BY_PRIORITY_WHEN_QUIESCED_KEY))
            {
                let quiesce_said = match quiesce {
                    GpuQuiesce::QuiescedByTheWrapper => "说是清过场",
                    GpuQuiesce::NotQuiesced => "也没清场",
                };
                return Err(format!(
                    "没经服务给卡（环境里没有 {CARDS_BY_PRIORITY_KEY}，或是空白）、{quiesce_said}，多机配置{}却开了 GPU（{GPU_SWITCH_KEY}=1）或写了 {CARDS_BY_PRIORITY_KEY} 或 {CARDS_BY_PRIORITY_WHEN_QUIESCED_KEY}：\
                     不照多机配置自己挑卡，也不照本机空闲显存挑卡（运行条件不满足，退 78）。\
                     出路：卡只归 GPU 批处理验证服务（清场、开卡都归它）：判段的结论交它判，判段之外用卡的测试交它代跑；\
                     验证包的测试与判段不借卡（借卡只许它的借卡白名单里的装置，.claude/rules/verification.md「卡只归 GPU 服务，任务不拿卡」）",
                    configuration_file.map_or_else(String::new, |path| format!(" {}", path.display()))
                ));
            }
            None
        }
    };
    match text {
        Some(text) => parse_cards_by_priority(&text, &addresses_of_the_hosts()?).map(Some),
        None => Ok(None),
    }
}

/// 调用方给这个进程的显卡配置：挑卡读的三个环境变量（多机配置在哪 [`MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE`]、显卡先后
/// [`CARDS_BY_PRIORITY_KEY`]、清没清场 [`QUIESCE_HOLDER_ENVIRONMENT_VARIABLE`]）
/// 在这个进程第一次换环境之前的值（[`GpuConfigurationOfTheCaller::of_this_process`]）。
/// GPU 用例共用的卡池（`crate::gpu_unit_checks::gpu_test_start`）照它挑卡，用例抄显卡先后（[`card_priority_lines_of_the_caller`]）也照它：
/// 同一个测试二进制里别的用例拿着改环境的锁、把这几个变量换成它自己的临时配置时，这两处读到的仍是调用方那一份，不是那条用例的
/// （卡池只认进程起来时调用方的那份配置，不跟用例临时改的环境走）。
/// 崩溃放量一趟（`crate::crash_amplification::run_from_environment`）照旧读此刻的环境：用例给这一趟设的临时配置就是要它照着跑的。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GpuConfigurationOfTheCaller {
    multi_host_configuration: Option<PathBuf>,
    cards_by_priority: Option<String>,
    quiesce: GpuQuiesce,
}

/// GPU 用例与卡池照调用方那一份取的卡（[`GpuConfigurationOfTheCaller::cards_for_the_gpu_tests`]）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CardsOfTheCallerForTheGpuTests {
    /// 服务交了卡串（[`card_string_handed_over_by_the_service`]），读成的卡：GPU 用例只在其中这一台的那几张上跑。
    HandedOverByTheService(ConfiguredCards),
    /// 没交卡串（没设、空串或只有空白）：当这台机器没有卡，不借卡、不找本机的卡，GPU 用例照这一句报「本次未跑」。
    NotHandedOver { reason_not_run: String },
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

    /// 此刻进程环境里的那三个变量。只给 [`Self::of_this_process`] 记那一份用：别处在守卫改着环境时读，读到的是那条用例的临时配置。
    pub(crate) fn read_from_the_environment_now() -> Self {
        Self {
            multi_host_configuration: std::env::var_os(
                MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE,
            )
            .map(PathBuf::from),
            cards_by_priority: std::env::var(CARDS_BY_PRIORITY_KEY).ok(),
            quiesce: GpuQuiesce::of_this_process(),
        }
    }

    /// 单测造的一份：多机配置路径不设、没清场，卡串照 `cards_by_priority` 原样（`None` 是没设）。
    #[cfg(test)]
    pub(crate) fn for_the_unit_tests(cards_by_priority: Option<&str>) -> Self {
        Self {
            multi_host_configuration: None,
            cards_by_priority: cards_by_priority.map(str::to_string),
            quiesce: GpuQuiesce::NotQuiesced,
        }
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

    /// 调用方给这个进程的多机配置在哪：这一份记下的环境变量 `SINGLEFS_MULTI_HOST_CONFIG` 原样（调用方没设是 `None`，那时照 git common-dir 找）。
    /// 仓副本不是 git 工作树、主工作树根下那一份找不到，证红与合入验证在副本里跑时由调用方经这个变量交配置；环境守卫清掉了这个变量、
    /// 照调用方的配置跑的用例照它设回去（`crate::crash_amplification::CrashAmplificationEnvironment::set_the_multi_host_configuration_and_the_cards_of_the_caller`）。
    #[must_use]
    pub fn multi_host_configuration_handed_in(&self) -> Option<&Path> {
        self.multi_host_configuration.as_deref()
    }

    /// 外层交给这个进程的卡串：这一份记下的环境变量 `GPU_CARDS_BY_PRIORITY` 原样（没设是 `None`；空白的照原样交，读的那一处照
    /// [`card_string_handed_over_by_the_service`] 当没交）。
    /// 环境守卫清掉了这个变量、要上卡的用例照它设回去（`crate::crash_amplification::CrashAmplificationEnvironment::set_the_multi_host_configuration_and_the_cards_of_the_caller`）。
    #[must_use]
    pub fn cards_handed_out_by_the_service(&self) -> Option<&str> {
        self.cards_by_priority.as_deref()
    }

    /// 配置的显卡优先级（同 [`cards_by_priority_from_configuration_or_environment`]，环境那一层读的是这一份；`HOSTS` 与这一台同样取
    /// [`multi_host_addresses_of_the_caller`]）。
    ///
    /// 这一份没交卡串的不借卡（卡只归 GPU 批处理验证服务，验证包的测试与判段不借卡），照 [`cards_by_priority_from`] 判没经服务给卡。
    ///
    /// # Errors
    /// 同 [`cards_by_priority_from`]。
    pub fn cards_by_priority(
        &self,
        configuration_file: Option<&Path>,
    ) -> Result<Option<ConfiguredCards>, String> {
        cards_by_priority_from(
            self.cards_by_priority.clone(),
            configuration_file,
            self.quiesce,
            &multi_host_addresses_of_the_caller,
        )
    }

    /// GPU 用例与卡池取的卡（[`Self::cards_for_the_gpu_tests_given`]，`HOSTS` 与这一台取 [`multi_host_addresses_of_the_caller`]）。
    ///
    /// # Errors
    /// 同 [`Self::cards_for_the_gpu_tests_given`]。
    pub fn cards_for_the_gpu_tests(&self) -> Result<CardsOfTheCallerForTheGpuTests, String> {
        self.cards_for_the_gpu_tests_given(&multi_host_addresses_of_the_caller)
    }

    /// GPU 用例与卡池取的卡：验证包的 GPU 用例只在服务交的卡上跑。这一份交了卡串（[`card_string_handed_over_by_the_service`]）照它读成卡
    /// （`addresses_of_the_hosts` 只在这时调）；没交卡串（没设、空串或只有空白）交 [`CardsOfTheCallerForTheGpuTests::NotHandedOver`]：
    /// 当这台机器没有卡，不借卡、不读多机配置、不照本机空闲显存自己挑卡。
    ///
    /// # Errors
    /// 交了卡串而读不成（[`parse_cards_by_priority`]），或 `addresses_of_the_hosts` 报错。
    pub fn cards_for_the_gpu_tests_given(
        &self,
        addresses_of_the_hosts: &dyn Fn() -> Result<MultiHostAddresses, String>,
    ) -> Result<CardsOfTheCallerForTheGpuTests, String> {
        match card_string_handed_over_by_the_service(self.cards_by_priority.as_deref()) {
            Some(card_string) => parse_cards_by_priority(card_string, &addresses_of_the_hosts()?)
                .map(CardsOfTheCallerForTheGpuTests::HandedOverByTheService),
            None => Ok(CardsOfTheCallerForTheGpuTests::NotHandedOver {
                reason_not_run: format!(
                    "调用方没交卡串（{CARDS_BY_PRIORITY_KEY} 没设，或是空白）：卡只归 GPU 批处理验证服务，验证包的 GPU 用例不借卡，当这台机器没有卡"
                ),
            }),
        }
    }

    /// 用例自己写一份开 GPU 的多机配置时，照抄调用方这一份的显卡先后：
    /// 这一份设了 `GPU_CARDS_BY_PRIORITY`（服务分的卡）就两个键都写它；没设照这一份的多机配置
    /// （[`Self::multi_host_configuration_file`]）里两个键各自写了的抄。交回要接在配置后面的那几行（一个键一行，没得抄是空串）。
    /// 抄进配置的卡不拿来挑卡（[`cards_by_priority_from`] 只照环境变量取卡）：那一趟要卡时照服务交的环境变量取，用例在环境守卫里另设
    /// （[`Self::cards_handed_out_by_the_service`]）。
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

/// 单测造的临时显卡配置点名的卡（[`while_another_thread_holds_a_temporary_gpu_configuration`]）：调用方的配置里不会有这一张
/// （地址取文档用的地址段 192.0.2.0/24，不是哪一台的地址；只当字面比，不读卡项）。
#[cfg(test)]
pub(crate) const CARDS_OF_THE_TEMPORARY_GPU_CONFIGURATION: &str = "192.0.2.93:4093:1M";

/// 单测造 E5 报告第三节第 3 条那种交错，确定地造：另一条线程拿着改环境的锁（`crate::crash_amplification::CrashAmplificationEnvironment`），
/// 把多机配置换成一份临时的（开 GPU、两个显卡先后的键都点 [`CARDS_OF_THE_TEMPORARY_GPU_CONFIGURATION`]），显卡先后的环境变量也点它；
/// 设好之后这边才跑 `work`，`work` 跑完那条线程才还原、放锁。交回 `work` 的结果与那份临时配置的路径
/// （跑完已删）。
#[cfg(test)]
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

/// 单测用的主机数组与这一台：判法 `--emit-addresses` 打的那几行原样（这一台是 127.0.0.1，另一台是文档用的地址段 192.0.2.81；不起判法，
/// 只钉读卡项的那几步）。
#[cfg(test)]
pub(crate) fn multi_host_addresses_for_the_unit_tests() -> Result<MultiHostAddresses, String> {
    MultiHostAddresses::from_the_emitted_assignments(
        "HOSTS=127.0.0.1\\ 192.0.2.81\nTHIS_HOST_ADDRESS=127.0.0.1\nPEER_HOST_ADDRESS=192.0.2.81\n",
    )
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

    /// 没经服务给卡（环境变量 `GPU_CARDS_BY_PRIORITY` 没给）而说是清过场：多机配置开了 GPU、写了不清场那一份或清场之后那一份显卡先后的，
    /// 报错、出路写卡只归 GPU 批处理验证服务、判段的结论交它判、判段之外用卡的测试交它代跑、验证包不借卡（不写借卡通道），不照多机配置清场之后那一份自己挑卡，
    /// 也不照本机空闲显存挑卡（`.claude/rules/verification.md`「卡只归 GPU 服务，任务不拿卡」）；GPU 没开、两份都没写的、没有多机配置的是没配；
    /// 环境变量给了卡照它。判别力（`crates/mutations.tsv`「显卡配置 没经服务给卡的出路照旧写经借卡通道借」「显卡配置 没经服务给卡的出路写整条经任务服务交」）。
    #[test]
    fn quiesced_without_cards_handed_out_by_the_service_refuses_the_configured_gpu_with_the_way_out(
    ) {
        let directory = temporary_directory("quiesced-without-cards-of-the-service");
        let only_the_gpu_switch = directory.join("only-the-gpu-switch.env");
        std::fs::write(&only_the_gpu_switch, "ENABLE_GPU=1\n").expect("写得了");
        let the_gpu_and_the_cards_when_quiesced = directory.join("gpu-and-cards-when-quiesced.env");
        std::fs::write(
            &the_gpu_and_the_cards_when_quiesced,
            "ENABLE_GPU=1\nGPU_CARDS_BY_PRIORITY_WHEN_QUIESCED='127.0.0.1:0:28G 127.0.0.1:1:13G'\n",
        )
        .expect("写得了");
        let only_the_cards_not_quiesced = directory.join("only-the-cards-not-quiesced.env");
        std::fs::write(
            &only_the_cards_not_quiesced,
            "GPU_CARDS_BY_PRIORITY='127.0.0.1:1:8G'\n",
        )
        .expect("写得了");
        let gpu_switched_off = directory.join("gpu-switched-off.env");
        std::fs::write(&gpu_switched_off, "ENABLE_GPU=0\n").expect("写得了");
        for configuration in [
            &only_the_gpu_switch,
            &the_gpu_and_the_cards_when_quiesced,
            &only_the_cards_not_quiesced,
        ] {
            let refusal = cards_by_priority_from(
                None,
                Some(configuration.as_path()),
                GpuQuiesce::QuiescedByTheWrapper,
                &multi_host_addresses_for_the_unit_tests,
            )
            .expect_err("没经服务给卡、清过场而多机配置要 GPU：报错，不照多机配置自己挑卡");
            assert!(
                refusal.contains("说是清过场")
                    && refusal.contains(GPU_SWITCH_KEY)
                    && refusal.contains(CARDS_BY_PRIORITY_WHEN_QUIESCED_KEY)
                    && refusal.contains("出路：卡只归 GPU 批处理验证服务")
                    && refusal.contains("判段的结论交它判")
                    && refusal.contains("判段之外用卡的测试交它代跑")
                    && refusal.contains("验证包的测试与判段不借卡")
                    && !refusal.contains("借卡通道")
                    && !refusal.contains("task-server/task submit")
                    && !refusal.contains("task-server-gpu-arguments"),
                "报错写明清过场、点名 GPU 开关与清场之后那个键，出路写卡只归 GPU 批处理验证服务、判段的结论交它判、判段之外用卡的测试交它代跑、验证包不借卡（{}）：{refusal}",
                configuration.display()
            );
        }
        for configuration in [Some(gpu_switched_off.as_path()), None] {
            assert_eq!(
                cards_by_priority_from(
                    None,
                    configuration,
                    GpuQuiesce::QuiescedByTheWrapper,
                    &multi_host_addresses_for_the_unit_tests
                ),
                Ok(None),
                "GPU 没开、两份都没写，或没有多机配置：没配显卡先后，不报错（{configuration:?}）"
            );
        }
        assert_eq!(
            cards_by_priority_from(
                Some("127.0.0.1:0:4G 192.0.2.81:1:4G".to_string()),
                Some(the_gpu_and_the_cards_when_quiesced.as_path()),
                GpuQuiesce::QuiescedByTheWrapper,
                &multi_host_addresses_for_the_unit_tests
            )
            .map(|cards| cards.map(|cards| cards.cards_of_this_host())),
            Ok(Some(vec![(0, 4 * 1024)])),
            "服务交了卡（环境变量给了）照它，不看配置里清场之后那一份"
        );
        drop(directory);
    }

    /// 没经服务给卡（环境变量 `GPU_CARDS_BY_PRIORITY` 没给）、也没清场：多机配置开了 GPU 或写了不清场那一份显卡先后的，报错、出路写卡只归
    /// GPU 批处理验证服务（判段的结论交它判，判段之外用卡的测试交它代跑），不照多机配置自己挑卡（`.claude/rules/verification.md`「卡只归 GPU 服务，任务不拿卡」）；
    /// GPU 没开、没写不清场那一份的是没配；服务交了卡（环境变量给了）照它。
    #[test]
    fn not_quiesced_without_cards_handed_out_by_the_service_refuses_the_configured_gpu_with_the_way_out(
    ) {
        let directory = temporary_directory("not-quiesced-without-cards-of-the-service");
        let only_the_gpu_switch = directory.join("only-the-gpu-switch.env");
        std::fs::write(&only_the_gpu_switch, "ENABLE_GPU=1\n").expect("写得了");
        let only_the_cards_not_quiesced = directory.join("only-the-cards-not-quiesced.env");
        std::fs::write(
            &only_the_cards_not_quiesced,
            "GPU_CARDS_BY_PRIORITY='127.0.0.1:1:8G'\n",
        )
        .expect("写得了");
        let gpu_switched_off = directory.join("gpu-switched-off.env");
        std::fs::write(&gpu_switched_off, "ENABLE_GPU=0\n").expect("写得了");
        for configuration in [&only_the_gpu_switch, &only_the_cards_not_quiesced] {
            let refusal = cards_by_priority_from(
                None,
                Some(configuration.as_path()),
                GpuQuiesce::NotQuiesced,
                &multi_host_addresses_for_the_unit_tests,
            )
            .expect_err("没经服务给卡、没清场而多机配置要 GPU：报错，不照多机配置自己挑卡");
            assert!(
                refusal.contains(GPU_SWITCH_KEY)
                    && refusal.contains(CARDS_BY_PRIORITY_KEY)
                    && refusal.contains("出路：卡只归 GPU 批处理验证服务")
                    && refusal.contains("判段的结论交它判")
                    && refusal.contains("判段之外用卡的测试交它代跑")
                    && !refusal.contains("task-server/task submit")
                    && !refusal.contains("task-server-gpu-arguments"),
                "报错点名 GPU 开关与显卡先后那个键，出路写卡只归 GPU 批处理验证服务、判段的结论交它判、判段之外用卡的测试交它代跑（{}）：{refusal}",
                configuration.display()
            );
        }
        assert_eq!(
            cards_by_priority_from(
                None,
                Some(gpu_switched_off.as_path()),
                GpuQuiesce::NotQuiesced,
                &multi_host_addresses_for_the_unit_tests
            ),
            Ok(None),
            "GPU 没开、没写不清场那一份：没配显卡先后，不报错"
        );
        assert_eq!(
            cards_by_priority_from(
                Some("127.0.0.1:0:4G".to_string()),
                Some(only_the_gpu_switch.as_path()),
                GpuQuiesce::NotQuiesced,
                &multi_host_addresses_for_the_unit_tests
            )
            .map(|cards| cards.map(|cards| cards.cards_of_this_host())),
            Ok(Some(vec![(0, 4 * 1024)])),
            "服务交了卡（环境变量给了）照它"
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
    #[test]
    fn the_card_priority_lines_copied_while_another_thread_holds_a_temporary_configuration_are_those_of_the_caller(
    ) {
        // 仓根用一次性临时目录当夹具，不读真仓：调用方环境里没给显卡先后、也没点多机配置时抄的是这份夹具里写的，
        // 与那条线程的临时配置不同；判别力在那条线程换上的环境变量上，不在仓根下有没有配置。
        let fixture_directory = temporary_directory("copied-lines-repository-root");
        std::fs::write(
            fixture_directory.join(MULTI_HOST_CONFIGURATION_FILE_NAME),
            format!(
                "{GPU_SWITCH_KEY}=1\n{CARDS_BY_PRIORITY_KEY}='127.0.0.1:0:1M'\n\
                 {CARDS_BY_PRIORITY_WHEN_QUIESCED_KEY}='127.0.0.1:0:1M'\n"
            ),
        )
        .expect("写得了夹具里的多机配置");
        let repository_root: &Path = &fixture_directory;
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

    /// 卡项的地址从右边切两刀：IPv4 与自带冒号的 IPv6 都切得对（`fd00::2:1:512M` 是 `fd00::2` 的 1 号卡），地址照 `HOSTS` 里那一项、序号与额度照写的读。
    #[test]
    fn card_entries_are_split_from_the_right_so_ipv4_and_ipv6_addresses_both_read() {
        let addresses = MultiHostAddresses::from_the_emitted_assignments(
            "HOSTS=192.0.2.1\\ fd00::2\nTHIS_HOST_ADDRESS=192.0.2.1\nPEER_HOST_ADDRESS=fd00::2\n",
        )
        .expect("判法打的行认得出");
        let configured = parse_cards_by_priority("192.0.2.1:0:32G fd00::2:1:512M", &addresses)
            .expect("两项的地址都在 HOSTS 里");
        let read: Vec<(&str, u32, u64)> = configured
            .cards()
            .iter()
            .map(|card| (card.host.as_str(), card.index, card.memory_quota_mebibytes))
            .collect();
        assert_eq!(
            read,
            vec![("192.0.2.1", 0, 32 * 1024), ("fd00::2", 1, 512)],
            "IPv4 那一项与 IPv6 那一项都从右边切两刀"
        );
        assert_eq!(configured.this_host().as_str(), "192.0.2.1");
    }

    /// 卡项点的地址不在 `HOSTS` 里：判配置写坏，报错点名那个地址、抄出 `HOSTS`、给出写法（出路）。
    #[test]
    fn a_card_entry_whose_address_is_not_in_the_hosts_is_refused_with_the_way_out() {
        let addresses = multi_host_addresses_for_the_unit_tests().expect("判法打的行认得出");
        let refusal = parse_cards_by_priority("127.0.0.1:0:8G 192.0.2.99:1:8G", &addresses)
            .expect_err("192.0.2.99 不在 HOSTS 里");
        assert!(
            refusal.contains("「192.0.2.99」")
                && refusal.contains("127.0.0.1 192.0.2.81")
                && refusal.contains("<HOSTS 里的地址>:<序号>:<显存额度>"),
            "报错点名不在 HOSTS 里的地址、抄出 HOSTS、给出写法：{refusal}"
        );
    }

    /// 选卡只取这一台地址那几项，按配置的次序；另一台的卡不进这一台的卡池。
    #[test]
    fn only_the_cards_of_this_host_are_taken_in_the_configured_order() {
        let addresses = multi_host_addresses_for_the_unit_tests().expect("判法打的行认得出");
        let configured = parse_cards_by_priority(
            "192.0.2.81:0:30G 127.0.0.1:2:20G 192.0.2.81:1:10G 127.0.0.1:0:5G",
            &addresses,
        )
        .expect("地址都在 HOSTS 里");
        assert_eq!(
            configured.cards_of_this_host(),
            vec![(2, 20 * 1024), (0, 5 * 1024)],
            "只取 127.0.0.1 那两项，照写的次序"
        );
        assert_eq!(
            configured.cards().len(),
            4,
            "另一台的卡留在配置里（排派活计划要按额度算它）"
        );
    }

    /// 单测自己写的一份写全键的多机配置（判法整份判；`NFS_SHARED_ROOTS` 指这条用例的临时目录，HOSTS 照给的写）。
    /// 按台的三个键 `CARGO_BIN_DIRECTORY_BY_HOST`、`BUILD_ROOT_BY_HOST`、`SLICE_MEMORY_CAP_BY_HOST` 给 HOSTS 里每一台恰好一项 `<地址>=<值>`
    /// （判法文件头「按台的三个键」）：目录是绝对路径、上限带单位；双机关着，判法只判写法、不去各台查 cargo，目录不必在盘上。
    fn complete_multi_host_configuration(
        directory: &crate::crash_library::ExclusiveTemporaryDirectory,
        name: &str,
        hosts: &str,
    ) -> PathBuf {
        let one_item_per_host = |value_for_the_host: fn(&str) -> String| -> String {
            hosts
                .split_whitespace()
                .map(|address| format!("{address}={}", value_for_the_host(address)))
                .collect::<Vec<_>>()
                .join(" ")
        };
        let cargo_bin_directory_by_host =
            one_item_per_host(|address| format!("/unit-test/cargo-bin-of-{address}"));
        let build_root_by_host =
            one_item_per_host(|address| format!("/unit-test/build-root-of-{address}"));
        let slice_memory_cap_by_host = one_item_per_host(|_| "2G".to_string());
        let configuration = directory.join(name);
        std::fs::write(
            &configuration,
            format!(
                "CARGO_BIN_DIRECTORY_BY_HOST={cargo_bin_directory_by_host}\nBUILD_ROOT_BY_HOST={build_root_by_host}\n\
                 SLICE_MEMORY_CAP_BY_HOST={slice_memory_cap_by_host}\n\
                 QUIESCE_STOP_COMMAND=\nQUIESCE_STOPPED_CHECK_COMMAND=\nQUIESCE_START_COMMAND=\nQUIESCE_STARTED_CHECK_COMMAND=\n\
                 ENABLE_ACROSS_MACHINES=0\nENABLE_GPU=0\nNFS_SHARED_ROOTS={}\nHOSTS={hosts}\n",
                directory.display()
            ),
        )
        .expect("写得了单测的多机配置");
        configuration
    }

    /// 这一台取自多机配置判法 `--emit-addresses` 打的 `THIS_HOST_ADDRESS`：自造一份写全键的多机配置，HOSTS 里 127.0.0.1 在每一台的网卡上、
    /// 写在后面；192.0.2.81 是文档用的地址段、不在任何一台上，写在前面（照数组次序当头一项是自己就认错）。主机数组照判法打的原样、次序不变。
    #[test]
    fn this_host_is_the_address_the_configuration_check_emits() {
        let directory = temporary_directory("this-host-from-the-configuration-check");
        let configuration = complete_multi_host_configuration(
            &directory,
            "this-host-written-last.env",
            "192.0.2.81 127.0.0.1",
        );
        let addresses = MultiHostAddresses::emitted_by_the_configuration_check(&configuration)
            .expect("写全键的多机配置判得过、认得出这一台");
        assert_eq!(
            addresses.this_host().as_str(),
            "127.0.0.1",
            "这一台是网卡上有的那一项"
        );
        assert_eq!(
            addresses
                .hosts()
                .iter()
                .map(HostAddress::as_str)
                .collect::<Vec<_>>(),
            vec!["192.0.2.81", "127.0.0.1"],
            "主机数组照判法打的原样"
        );
        drop(directory);
    }

    /// HOSTS 里没有一项是这一台网卡上的地址（命中 0 项，不是恰好一项）：判法判不过，报错是「判法认不出这一台」、带判法的原话，不猜哪一台是自己。
    #[test]
    fn hosts_naming_no_address_of_this_machine_are_refused_by_the_configuration_check() {
        let directory = temporary_directory("hosts-without-this-machine");
        let configuration = complete_multi_host_configuration(
            &directory,
            "hosts-without-this-machine.env",
            "192.0.2.82 192.0.2.81",
        );
        let refusal = MultiHostAddresses::emitted_by_the_configuration_check(&configuration)
            .expect_err("HOSTS 里没有这一台：判法判不过");
        assert!(
            refusal.starts_with("多机配置判法认不出这一台")
                && refusal.contains("这一台网卡上的地址"),
            "报错是判法认不出这一台、带判法的原话：{refusal}"
        );
        drop(directory);
    }

    /// 单测用的借卡通道的三个变量：取认得出的字面（尝试标签、任务号、第几次尝试）。
    fn card_lease_variable_values_for_the_unit_tests() -> Vec<(&'static str, OsString)> {
        vec![
            (
                CARD_LEASE_ATTEMPT_ENVIRONMENT_VARIABLES[0],
                OsString::from("label-of-the-unit-test"),
            ),
            (
                CARD_LEASE_ATTEMPT_ENVIRONMENT_VARIABLES[1],
                OsString::from("4093"),
            ),
            (
                CARD_LEASE_ATTEMPT_ENVIRONMENT_VARIABLES[2],
                OsString::from("1"),
            ),
        ]
    }

    /// 单测用的借卡通道：三个变量照 [`card_lease_variable_values_for_the_unit_tests`]，`SINGLEFS_MULTI_HOST_CONFIG` 照 `multi_host_configuration` 给不给。
    fn card_lease_channel_for_the_unit_tests(
        multi_host_configuration: Option<&str>,
    ) -> CardLeaseChannelOfTheCaller {
        let values = card_lease_variable_values_for_the_unit_tests();
        CardLeaseChannelOfTheCaller::of_the_variable_values(&|name| {
            if name == MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE {
                return multi_host_configuration.map(OsString::from);
            }
            values
                .iter()
                .find(|(known, _)| *known == name)
                .map(|(_, value)| value.clone())
        })
        .expect("三个变量都给了")
    }

    /// 假的借卡客户端：`directory` 下一份假仓里的 `gpu-verification-server/control`，是 python 脚本、不设执行位（只有经 `python3 <它>` 起才跑得起来）。
    /// 每次调往 `directory` 下的 `calls.log` 记一行，`|` 隔开：脚本路径、参数（JSON）、通道的三个变量、`GPU_CARD_LEASE_BORROWER`、
    /// `SINGLEFS_MULTI_HOST_CONFIG`（没设写 `<unset>`），末一段是它自己从 `/proc/<父进程号>/stat` 读出的 `<父进程号>:<父进程启动刻>`
    /// （借的进程直接起它、还卡的 shell 经 `exec` 换成它，父进程都是借的进程）。`lease take` 再照 `take_behaviour`（一段 python）办，别的调用退 0。
    /// 交回（客户端，调用记录）。
    fn fake_card_lease_client(directory: &Path, take_behaviour: &str) -> (PathBuf, PathBuf) {
        use std::os::unix::fs::PermissionsExt;
        let client = directory
            .join("fake-repository")
            .join("gpu-verification-server")
            .join("control");
        std::fs::create_dir_all(client.parent().expect("客户端在假仓的一个目录下"))
            .expect("建得了假客户端的目录");
        let calls = directory.join("calls.log");
        std::fs::write(
            &client,
            format!(
                "import json, os, sys\n\
                 def identity_of_the_process(process_identifier):\n    \
                     with open('/proc/%d/stat' % process_identifier) as process_status_file:\n        \
                         fields_after_the_command_name = process_status_file.read().rsplit(')', 1)[1].split()\n    \
                     return '%d:%s' % (process_identifier, fields_after_the_command_name[19])\n\
                 record = [sys.argv[0], json.dumps(sys.argv[1:])]\n\
                 record += [os.environ.get(name, '<unset>') for name in ('TASK_SERVER_ATTEMPT_LABEL', 'TASK_SERVER_TASK', 'TASK_SERVER_ATTEMPT', \
                 'GPU_CARD_LEASE_BORROWER', 'SINGLEFS_MULTI_HOST_CONFIG')]\n\
                 record.append(identity_of_the_process(os.getppid()))\n\
                 with open('{}', 'a') as calls:\n    \
                     calls.write('|'.join(record) + '\\n')\n\
                 if sys.argv[1:3] != ['lease', 'take']:\n    \
                     sys.exit(0)\n\
                 {take_behaviour}\n",
                calls.display()
            ),
        )
        .expect("写得了假客户端");
        std::fs::set_permissions(&client, std::fs::Permissions::from_mode(0o644))
            .expect("假客户端设得了不带执行位");
        (client, calls)
    }

    /// 假客户端（[`fake_card_lease_client`]）照这一次调用该记的那一行：通道的三个变量是单测的字面，借的一方是这个进程，
    /// 末一段（假客户端自己从 `/proc` 读的父进程）也该是这个进程。
    fn expected_call_of_the_fake_card_lease_client(
        client: &Path,
        arguments_as_json: &str,
        multi_host_configuration_seen_by_the_client: &str,
        borrower: &CardLeaseBorrower,
    ) -> String {
        format!(
            "{}|{arguments_as_json}|label-of-the-unit-test|4093|1|{borrower_value}|{multi_host_configuration_seen_by_the_client}|{borrower_value}\n",
            client.display(),
            borrower_value = borrower.value_for_the_client()
        )
    }

    /// 借卡通道认的是统一任务服务交给每次尝试的三个变量（尝试标签、任务号、第几次尝试）：只给这三个、都不空就是 `Some`，记下的就是这三个、值原样；
    /// 少一个、有一个是空串都是 `None`。
    /// 判别力（`crates/mutations.tsv`「按需领卡 通道缺变量也算有」「借卡通道 还要三个尝试变量之外的仓根变量」）：缺的当空串收，少一个变量也认成有通道；
    /// 三个之外还要别的变量才认，只给这三个就认不出通道。
    #[test]
    fn a_card_lease_channel_needs_the_three_attempt_variables_present_and_not_empty() {
        let values = card_lease_variable_values_for_the_unit_tests();
        let channel = CardLeaseChannelOfTheCaller::of_the_variable_values(&|name| {
            values
                .iter()
                .find(|(known, _)| *known == name)
                .map(|(_, value)| value.clone())
        })
        .unwrap_or_else(|| panic!("只给三个尝试变量、都不空：认得出通道"));
        assert_eq!(
            channel.variables_for_the_client(),
            values.as_slice(),
            "三个都给：通道记下的就是这三个、值原样"
        );
        for (missing, _) in &values {
            let without_one = CardLeaseChannelOfTheCaller::of_the_variable_values(&|name| {
                values
                    .iter()
                    .find(|(known, _)| *known == name && known != missing)
                    .map(|(_, value)| value.clone())
            });
            assert_eq!(without_one, None, "少了 {missing}：不算有通道");
            let with_one_empty = CardLeaseChannelOfTheCaller::of_the_variable_values(&|name| {
                values
                    .iter()
                    .find(|(known, _)| *known == name)
                    .map(|(known, value)| {
                        if known == missing {
                            OsString::new()
                        } else {
                            value.clone()
                        }
                    })
            });
            assert_eq!(with_one_empty, None, "{missing} 是空串：不算有通道");
        }
    }

    /// 启动刻从命令名最后一个右括号之后数（那之后的头一项是第 3 项，启动刻是第 22 项）：命令名里带空格与右括号也数得对；
    /// 没有右括号、不够 22 项、第 22 项不是数都报错。判别力（`crates/mutations.tsv`「借卡 启动刻从命令名的头一个右括号数」）。
    #[test]
    fn the_start_time_is_read_after_the_last_closing_parenthesis_of_the_command_name() {
        // 第 3 项起逐项写成 1000 加它的项号，只有第 22 项写 777：数错一项、从头一个右括号数都取不到 777
        let fields_from_the_state_on =
            (FIRST_FIELD_NUMBER_AFTER_THE_COMMAND_NAME_IN_THE_PROCESS_STATUS_LINE..=52)
                .map(|field_number| {
                    if field_number == START_TIME_FIELD_NUMBER_IN_THE_PROCESS_STATUS_LINE {
                        "777".to_string()
                    } else {
                        (1000 + field_number).to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");
        let process_status_line =
            format!("4093 (odd name) (with spaces) {fields_from_the_state_on}\n");
        assert_eq!(
            start_time_since_boot_in_clock_ticks_from_the_process_status_line(&process_status_line),
            Ok(777),
            "命令名里带空格与右括号：从最后一个右括号之后数到第 22 项"
        );
        assert!(
            start_time_since_boot_in_clock_ticks_from_the_process_status_line(
                "4093 no parenthesis S 1"
            )
            .is_err_and(|error| error.contains("右括号")),
            "没有右括号报错"
        );
        assert!(
            start_time_since_boot_in_clock_ticks_from_the_process_status_line(
                "4093 (short) S 1 2 3"
            )
            .is_err_and(|error| error.contains("不够 22 项")),
            "不够 22 项报错"
        );
        let not_a_number = process_status_line.replace(" 777 ", " started ");
        assert!(
            start_time_since_boot_in_clock_ticks_from_the_process_status_line(&not_a_number)
                .is_err_and(|error| error.contains("不是数")),
            "第 22 项不是数报错"
        );
    }

    /// 环境里已经有交下来的卡：照它，有没有借卡通道都不借；两样都没有是没经服务给卡。
    /// 判别力（`crates/mutations.tsv`「按需领卡 环境里有卡也去领」）：有通道时不管环境里有没有卡都借，借的次数是 1、交的是借来的那一张。
    #[test]
    fn cards_already_in_the_environment_are_used_without_leasing() {
        let channel = card_lease_channel_for_the_unit_tests(None);
        let leases = std::cell::Cell::new(0_u32);
        let lease = |_channel: &CardLeaseChannelOfTheCaller| -> Result<String, String> {
            leases.set(leases.get() + 1);
            Ok("192.0.2.93:7:1M".to_string())
        };
        assert_eq!(
            cards_handed_over_or_leased_on_demand(
                Some("192.0.2.93:4:2M".to_string()),
                Some(&channel),
                &lease
            ),
            Ok(Some("192.0.2.93:4:2M".to_string())),
            "环境里有卡、也有通道：照环境里的"
        );
        assert_eq!(
            cards_handed_over_or_leased_on_demand(
                Some("192.0.2.93:4:2M".to_string()),
                None,
                &lease
            ),
            Ok(Some("192.0.2.93:4:2M".to_string())),
            "环境里有卡、没有通道：照环境里的"
        );
        assert_eq!(
            cards_handed_over_or_leased_on_demand(None, None, &lease),
            Ok(None),
            "两样都没有：没经服务给卡"
        );
        assert_eq!(leases.get(), 0, "三种都不借");
    }

    /// 环境里没有卡而有借卡通道：这一刻借，交借来的那一张；借卡的错原样交回。
    #[test]
    fn without_cards_in_the_environment_the_cards_are_leased_through_the_channel() {
        let channel = card_lease_channel_for_the_unit_tests(None);
        assert_eq!(
            cards_handed_over_or_leased_on_demand(None, Some(&channel), &|_channel| Ok(
                "192.0.2.93:7:1M".to_string()
            )),
            Ok(Some("192.0.2.93:7:1M".to_string())),
            "交借来的那一张"
        );
        assert_eq!(
            cards_handed_over_or_leased_on_demand(None, Some(&channel), &|_channel| Err(
                "借卡没成".to_string()
            )),
            Err("借卡没成".to_string()),
            "借卡的错原样交回"
        );
    }

    /// 场景：多机配置开着 GPU、环境里没有卡串，一回此刻环境里有借卡通道的三个尝试变量（任务服务交仓根的那个变量指到假仓，假客户端
    /// `lease take` 退 3、每回调用记一行），一回没有；两回都照判段取卡读显卡优先级（[`cards_by_priority_from_configuration_or_environment`]，
    /// 崩溃放量观察者那一段取卡走它）。
    /// 预期：有通道的那一回不起借卡客户端（调用记录一行都没有），结局与没有通道的那一回逐字相同，都是没经服务给卡那一句拒绝。
    /// 判别力（`crates/mutations.tsv`「判段取卡 观察者那一段照此刻环境里的借卡通道借」）：照通道借，假客户端被调、结局成了借卡没成那一句。
    #[test]
    fn the_cards_of_the_observers_stage_ask_no_card_lease_channel_and_refuse_as_without_one() {
        let directory = temporary_directory("observers-stage-asks-no-card-lease-channel");
        let configuration = directory.join("gpu-switched-on.env");
        std::fs::write(&configuration, format!("{GPU_SWITCH_KEY}=1\n")).expect("写得了");
        let (_client, calls) = fake_card_lease_client(directory.path(), "sys.exit(3)");
        let fake_repository = directory.path().join("fake-repository");
        let without_the_channel = {
            let _environment =
                crate::crash_amplification::CrashAmplificationEnvironment::cleared_then_set(&[]);
            assert_eq!(
                CardLeaseChannelOfTheCaller::read_from_the_environment_now(),
                None,
                "前提：守卫里环境里没有借卡通道"
            );
            cards_by_priority_from_configuration_or_environment(Some(configuration.as_path()))
        };
        let attempt_values = card_lease_variable_values_for_the_unit_tests();
        let with_the_channel = {
            let settings: Vec<(&str, &std::ffi::OsStr)> = attempt_values
                .iter()
                .map(|(name, value)| (*name, value.as_os_str()))
                .collect();
            let _environment =
                crate::crash_amplification::CrashAmplificationEnvironment::cleared_then_set(
                    &settings,
                );
            let mut source_repository =
                crate::crash_amplification::EnvironmentRestoredOnDrop::saved_then_cleared(&[
                    TASK_SERVER_SOURCE_REPOSITORY_ENVIRONMENT_VARIABLE,
                ]);
            source_repository.set_variable(
                TASK_SERVER_SOURCE_REPOSITORY_ENVIRONMENT_VARIABLE,
                fake_repository.as_os_str(),
            );
            assert!(
                CardLeaseChannelOfTheCaller::read_from_the_environment_now().is_some(),
                "前提：此刻环境里认得出借卡通道"
            );
            cards_by_priority_from_configuration_or_environment(Some(configuration.as_path()))
        };
        assert!(
            !calls.exists(),
            "判段取卡起了借卡客户端：{}",
            std::fs::read_to_string(&calls).unwrap_or_default()
        );
        assert_eq!(
            with_the_channel, without_the_channel,
            "有没有借卡通道，判段取卡的结局逐字相同"
        );
        assert!(
            without_the_channel
                .as_ref()
                .is_err_and(|refusal| refusal.contains("没经服务给卡")),
            "两回都是没经服务给卡那一句拒绝：{without_the_channel:?}"
        );
        drop(directory);
    }

    /// 场景：调用方那一份的卡串没设、是空串、只有空白，照它取 GPU 用例与卡池的卡（[`GpuConfigurationOfTheCaller::cards_for_the_gpu_tests_given`]）。
    /// 预期：都是没交卡串，交回 GPU 用例报「本次未跑」的原因（点名没交卡串、不借卡），不当「本机能用的卡全用」；交了卡串的照它读成这一台的卡。
    /// 判别力（`crates/mutations.tsv`「显卡卡串 只有空白也当交了卡」）：只有空白的卡串当交了，读卡项读不成，这条红。
    #[test]
    fn a_missing_or_blank_card_string_of_the_caller_is_not_handed_over_and_the_gpu_tests_are_not_run(
    ) {
        for card_string in [None, Some(""), Some("   "), Some("\t\n")] {
            let cards = GpuConfigurationOfTheCaller::for_the_unit_tests(card_string)
                .cards_for_the_gpu_tests_given(&multi_host_addresses_for_the_unit_tests);
            assert!(
                matches!(
                    &cards,
                    Ok(CardsOfTheCallerForTheGpuTests::NotHandedOver { reason_not_run })
                        if reason_not_run.contains("调用方没交卡串") && reason_not_run.contains("不借卡")
                ),
                "卡串 {card_string:?}：当没交卡串，交回本次未跑的原因（点名没交卡串、不借卡）：{cards:?}"
            );
        }
        assert_eq!(
            GpuConfigurationOfTheCaller::for_the_unit_tests(Some("127.0.0.1:0:4G 192.0.2.81:1:4G"))
                .cards_for_the_gpu_tests_given(&multi_host_addresses_for_the_unit_tests)
                .map(|cards| match cards {
                    CardsOfTheCallerForTheGpuTests::HandedOverByTheService(handed_over) => {
                        Some(handed_over.cards_of_this_host())
                    }
                    CardsOfTheCallerForTheGpuTests::NotHandedOver { .. } => None,
                }),
            Ok(Some(vec![(0, 4 * 1024)])),
            "交了卡串：照它读成这一台的卡"
        );
    }

    /// 场景：判段取卡（[`cards_by_priority_from`]）时环境里的卡串是空串或只有空白，多机配置开着 GPU；另一份配置 GPU 没开。
    /// 预期：开着 GPU 的照没经服务给卡那一句拒绝（运行条件不满足），不当「本机能用的卡全用」；GPU 没开的是没配（`None`）。
    /// 判别力（`crates/mutations.tsv`「显卡配置 判段取卡空白卡串当交了卡」）：空白卡串当交了，读卡项报一项都没写，不是没经服务给卡那一句，这条红。
    #[test]
    fn a_blank_card_string_in_the_environment_is_refused_as_not_handed_over_when_the_configuration_enables_the_gpu(
    ) {
        let directory = temporary_directory("blank-card-string-in-the-environment");
        let gpu_switched_on = directory.join("gpu-switched-on.env");
        std::fs::write(&gpu_switched_on, format!("{GPU_SWITCH_KEY}=1\n")).expect("写得了");
        let gpu_switched_off = directory.join("gpu-switched-off.env");
        std::fs::write(&gpu_switched_off, format!("{GPU_SWITCH_KEY}=0\n")).expect("写得了");
        for card_string in ["", "  "] {
            let refused = cards_by_priority_from(
                Some(card_string.to_string()),
                Some(gpu_switched_on.as_path()),
                GpuQuiesce::NotQuiesced,
                &multi_host_addresses_for_the_unit_tests,
            );
            assert!(
                refused
                    .as_ref()
                    .is_err_and(|refusal| refusal.contains("没经服务给卡")),
                "卡串 {card_string:?}、配置开着 GPU：没经服务给卡那一句拒绝：{refused:?}"
            );
            assert_eq!(
                cards_by_priority_from(
                    Some(card_string.to_string()),
                    Some(gpu_switched_off.as_path()),
                    GpuQuiesce::NotQuiesced,
                    &multi_host_addresses_for_the_unit_tests,
                ),
                Ok(None),
                "卡串 {card_string:?}、GPU 没开：没配显卡先后"
            );
        }
        drop(directory);
    }

    /// 经客户端借一回：起的是 `python3 <客户端> lease take`（客户端不带执行位，只有经解释器起才跑得起来），标准输出就是卡串；
    /// 客户端的环境里有通道的三个变量、通道记下的多机配置路径、`GPU_CARD_LEASE_BORROWER=<这个进程号>:<这个进程的启动刻>`
    /// （与假客户端自己从 `/proc/<父进程号>/stat` 读出的父进程相同）；借到之后不还，还卡子进程的标准输入一收尾
    /// （借卡的进程退出时写端关掉，这里用当场还代它）才起 `python3 <客户端> lease return <卡串>`，环境与借的那一条相同。
    /// 判别力（`crates/mutations.tsv`「按需领卡 进程退出不还卡」「借卡 借的一方的启动刻取第 21 项」「借卡 不交借的一方」「借卡 客户端不经 python3 起」
    /// 「借卡 还卡那一条不带借卡的环境」「借卡 通道记下的多机配置不交给客户端」）。
    #[test]
    fn a_lease_through_the_client_takes_the_cards_and_returns_them_only_when_the_standard_input_of_the_return_ends(
    ) {
        let directory = temporary_directory("lease-take-then-return");
        let multi_host_configuration = "/unit-test/multi-host-of-the-channel.env";
        let channel = card_lease_channel_for_the_unit_tests(Some(multi_host_configuration));
        let borrower = CardLeaseBorrower::of_this_process().expect("读得出这个进程的启动刻");
        let (client, calls) =
            fake_card_lease_client(directory.path(), "print('192.0.2.93:7:1M')\nsys.exit(0)");
        let leased = lease_cards_through_the_client(&client, &channel, &borrower)
            .expect("经 python3 起的假客户端交了卡");
        assert_eq!(
            leased.cards(),
            "192.0.2.93:7:1M",
            "卡串是客户端标准输出那一行"
        );
        let take = expected_call_of_the_fake_card_lease_client(
            &client,
            r#"["lease", "take"]"#,
            multi_host_configuration,
            &borrower,
        );
        assert!(
            borrower
                .value_for_the_client()
                .starts_with(&format!("{}:", std::process::id())),
            "借的一方是这个进程：{borrower:?}"
        );
        assert_eq!(
            std::fs::read_to_string(&calls).expect("读得了调用记录"),
            take,
            "借到之后只调过一回 python3 <客户端> lease take：通道的三个变量、借的一方、多机配置都到了客户端，借的一方就是客户端的父进程；还没还"
        );
        assert_eq!(leased.return_the_cards_now(), Ok(true), "还卡的客户端退 0");
        let return_after_the_take = expected_call_of_the_fake_card_lease_client(
            &client,
            r#"["lease", "return", "192.0.2.93:7:1M"]"#,
            multi_host_configuration,
            &borrower,
        );
        assert_eq!(
            std::fs::read_to_string(&calls).expect("读得了调用记录"),
            format!("{take}{return_after_the_take}"),
            "标准输入收尾之后还的就是借到的那一张，环境与借的那一条相同"
        );
        drop(directory);
    }

    /// 认通道那一刻没设 `SINGLEFS_MULTI_HOST_CONFIG`：借卡客户端的环境里也没有它，此刻进程环境里设了（这里在环境守卫里设一个诱饵）也不交。
    /// 判别力（`crates/mutations.tsv`「借卡 通道没记多机配置也照此刻的环境交」）。
    #[test]
    fn a_lease_through_a_channel_read_without_the_multi_host_configuration_hands_none_to_the_client(
    ) {
        let directory = temporary_directory("lease-without-the-multi-host-configuration");
        let channel = card_lease_channel_for_the_unit_tests(None);
        let borrower = CardLeaseBorrower::of_this_process().expect("读得出这个进程的启动刻");
        let (client, calls) =
            fake_card_lease_client(directory.path(), "print('192.0.2.93:7:1M')\nsys.exit(0)");
        let _decoy_in_the_process_environment =
            crate::crash_amplification::CrashAmplificationEnvironment::cleared_then_set(&[(
                MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE,
                std::ffi::OsStr::new("/unit-test/decoy-multi-host-of-the-process.env"),
            )]);
        let leased = lease_cards_through_the_client(&client, &channel, &borrower)
            .expect("经 python3 起的假客户端交了卡");
        assert_eq!(leased.return_the_cards_now(), Ok(true), "还卡的客户端退 0");
        assert_eq!(
            std::fs::read_to_string(&calls).expect("读得了调用记录"),
            format!(
                "{}{}",
                expected_call_of_the_fake_card_lease_client(
                    &client,
                    r#"["lease", "take"]"#,
                    "<unset>",
                    &borrower
                ),
                expected_call_of_the_fake_card_lease_client(
                    &client,
                    r#"["lease", "return", "192.0.2.93:7:1M"]"#,
                    "<unset>",
                    &borrower
                )
            ),
            "通道没记多机配置：借与还两条都不交，不照此刻进程环境里的诱饵"
        );
        drop(directory);
    }

    /// 客户端退非 0（借不成）：报错点名退出码、写明不重试不退回 CPU、原因在客户端的标准错误，不写「整条重交」；不起还卡（没借到）。
    /// 判别力（`crates/mutations.tsv`「按需领卡 被拒当成领到」「借卡 借不成的出路照旧写整条重交」）。
    #[test]
    fn a_refused_lease_is_an_error_naming_the_exit_code_of_the_client_and_returns_nothing() {
        let directory = temporary_directory("lease-refused");
        let channel = card_lease_channel_for_the_unit_tests(None);
        let borrower = CardLeaseBorrower::of_this_process().expect("读得出这个进程的启动刻");
        let (client, calls) = fake_card_lease_client(directory.path(), "sys.exit(3)");
        let refusal = lease_cards_through_the_client(&client, &channel, &borrower)
            .expect_err("客户端退 3：借卡没成");
        assert!(
            refusal.contains("lease take 退 3")
                && refusal.contains("不重试、不退回 CPU")
                && refusal.contains("借不成的原因在客户端往标准错误打的那几行")
                && !refusal.contains("重交"),
            "报错点名退出码与不重试、原因在客户端的标准错误：{refusal}"
        );
        assert_eq!(
            std::fs::read_to_string(&calls).expect("读得了调用记录"),
            expected_call_of_the_fake_card_lease_client(
                &client,
                r#"["lease", "take"]"#,
                "<unset>",
                &borrower
            ),
            "没借到：不调 lease return"
        );
        drop(directory);
    }

    /// 客户端退 0 而没交卡串：报错，交回的空卡串当场 `lease return`（带着那个空卡串，环境与借的那一条相同）。
    /// 判别力（`crates/mutations.tsv`「借卡 没交卡串当场还时不带卡串」）。
    #[test]
    fn a_lease_granting_no_card_is_an_error_and_is_returned_at_once() {
        let directory = temporary_directory("lease-granting-no-card");
        let channel = card_lease_channel_for_the_unit_tests(None);
        let borrower = CardLeaseBorrower::of_this_process().expect("读得出这个进程的启动刻");
        let (client, calls) = fake_card_lease_client(directory.path(), "sys.exit(0)");
        let refusal = lease_cards_through_the_client(&client, &channel, &borrower)
            .expect_err("退 0 而没交卡串：借卡没成");
        assert!(
            refusal.contains("退 0 而没交卡串"),
            "报错写明没交卡串：{refusal}"
        );
        assert_eq!(
            std::fs::read_to_string(&calls).expect("读得了调用记录"),
            format!(
                "{}{}",
                expected_call_of_the_fake_card_lease_client(
                    &client,
                    r#"["lease", "take"]"#,
                    "<unset>",
                    &borrower
                ),
                expected_call_of_the_fake_card_lease_client(
                    &client,
                    r#"["lease", "return", ""]"#,
                    "<unset>",
                    &borrower
                )
            ),
            "当场还了一回，带着交回的那个空卡串"
        );
        drop(directory);
    }

    /// 借卡客户端：任务服务交了仓根就是它下面的 `gpu-verification-server/control`（不是文件就拒）；没交从起点往上逐层找仓里那个路径，
    /// 找到根都没有就拒。夹具照字面路径摆客户端，不照常量摆。
    /// 判别力（`crates/mutations.tsv`「按需领卡 客户端只在起点那一层找」「借卡 客户端仍找 task-server/task」）。
    #[test]
    fn the_card_lease_client_is_the_named_file_or_the_first_one_found_going_up_from_the_start() {
        let directory = temporary_directory("card-lease-client-lookup");
        let repository = directory.path().join("repository");
        let package = repository.join("crates").join("some-package");
        std::fs::create_dir_all(&package).expect("建得了包目录");
        let client_in_the_repository = repository.join("gpu-verification-server").join("control");
        std::fs::create_dir_all(
            client_in_the_repository
                .parent()
                .expect("客户端在仓里的一个目录下"),
        )
        .expect("建得了客户端的目录");
        std::fs::write(&client_in_the_repository, "import sys\n").expect("写得了客户端");
        assert_eq!(
            card_lease_client_from(None, &package),
            Ok(client_in_the_repository.clone()),
            "没点名：从包目录往上找到仓根下那一份"
        );
        let elsewhere = directory.path().join("elsewhere");
        std::fs::create_dir_all(&elsewhere).expect("建得了仓外的目录");
        assert_eq!(
            card_lease_client_from(Some(repository.clone()), &elsewhere),
            Ok(client_in_the_repository.clone()),
            "任务服务交了仓根：取它下面那一份，不看起点"
        );
        let refusal_of_the_source_repository =
            card_lease_client_from(Some(elsewhere.clone()), &package)
                .expect_err("任务服务交的仓根下没有客户端：拒，不改从起点往上找");
        assert!(
            refusal_of_the_source_repository
                .contains(TASK_SERVER_SOURCE_REPOSITORY_ENVIRONMENT_VARIABLE)
                && refusal_of_the_source_repository.contains("不是文件"),
            "仓根下没有客户端：{refusal_of_the_source_repository}"
        );
        std::fs::remove_file(&client_in_the_repository).expect("删得了客户端");
        let refusal_of_the_search =
            card_lease_client_from(None, &package).expect_err("往上找不到：拒");
        assert!(
            refusal_of_the_search.contains("往上找不到借卡客户端")
                && refusal_of_the_search
                    .contains(TASK_SERVER_SOURCE_REPOSITORY_ENVIRONMENT_VARIABLE),
            "找不到的出路写怎么点名：{refusal_of_the_search}"
        );
        drop(directory);
    }

    /// 场景：测试进程在临时 prune-server 底下（改连哪一处的两个变量与临时服务的标记都设着，变异门禁每一行起的那种），照借卡通道起借卡客户端。
    /// 预期：起客户端的命令把这三个变量都从子进程环境里拿掉（`get_envs` 里记成拿掉），借卡落到平常跑连的那一份真服务；
    /// 通道的三个变量与借的一方照设。只看命令的环境，不起客户端。
    /// 判别力（`crates/mutations.tsv`）：不拿掉改连哪一处的变量，这条红。
    #[test]
    fn the_card_lease_client_is_routed_to_the_real_prune_server_under_a_temporary_service() {
        let channel = CardLeaseChannelOfTheCaller::of_the_variable_values(&|name| {
            CARD_LEASE_ATTEMPT_ENVIRONMENT_VARIABLES
                .contains(&name)
                .then(|| OsString::from(format!("{name}-of-the-unit-test")))
        })
        .expect("前提：三个尝试变量都给了，认得出通道");
        let borrower = CardLeaseBorrower {
            process_identifier: 4321,
            start_time_since_boot_in_clock_ticks: 8765,
        };
        let command = card_lease_client_command(
            Path::new("/nonexistent-singlefs-card-lease-client/control"),
            &channel,
            &borrower,
        );
        let environment: std::collections::BTreeMap<OsString, Option<OsString>> = command
            .get_envs()
            .map(|(name, value)| {
                (
                    name.to_os_string(),
                    value.map(std::ffi::OsStr::to_os_string),
                )
            })
            .collect();
        for name in PRUNE_SERVER_CONNECTION_VARIABLES_KEPT_FROM_THE_CARD_LEASE_CLIENT {
            assert_eq!(
                environment.get(std::ffi::OsStr::new(name)),
                Some(&None),
                "{name} 从借卡客户端的环境里拿掉：借卡落到真服务"
            );
        }
        assert_eq!(
            environment.get(std::ffi::OsStr::new(
                CARD_LEASE_BORROWER_ENVIRONMENT_VARIABLE
            )),
            Some(&Some(OsString::from("4321:8765"))),
            "借的一方照设"
        );
        for attempt_variable in CARD_LEASE_ATTEMPT_ENVIRONMENT_VARIABLES {
            assert_eq!(
                environment.get(std::ffi::OsStr::new(attempt_variable)),
                Some(&Some(OsString::from(format!(
                    "{attempt_variable}-of-the-unit-test"
                )))),
                "通道的 {attempt_variable} 照设"
            );
        }
    }
}
