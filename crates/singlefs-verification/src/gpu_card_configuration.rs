//! 显卡配置怎么读：`GPU_CARDS_BY_PRIORITY` 点名的卡（在 `HOSTS` 里哪个地址那一台、`nvidia-smi` 序号、显存额度）、这一台是 `HOSTS` 里哪一项、
//! `键=值` 配置文件里一个键的值、点名的卡里哪几张用不上。崩溃放量取卡（`crate::crash_amplification`）与 GPU 用例共用的卡池
//! （`crate::gpu_unit_checks::gpu_test_start`，随 `gpu` 编）都照它挑卡，所以单放一份、不挂特性；
//! `crate::crash_amplification` 再 `pub use` 回去，它的调用方与单测照旧经它引。
//! 显卡的两个键写在多机配置那一份里（多机配置合成一份，不零散配置；判决
//! `research/prompts/m3-crash-enumeration-merge-r3-main-verification.md` 第三节 J5、J11）：`GPU_CARDS_BY_PRIORITY`（不清场那一份）与
//! `GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED`（清场之后）：清没清场，这里两份都只看写没写、不照它们挑卡，卡由服务分、经环境变量交（[`cards_by_priority_from`]）；多机配置在哪照 [`multi_host_configuration_file`]（与
//! `scripts/crash/crash-amplification-lib.sh` 的 `crash_amplification_multi_host_configuration_file` 同一种找法）。GPU 开关与两个显卡键只写在多机配置里。
//! 两台对称，没有「主机角色」：卡项点的是 `HOSTS` 里的地址，这一台是 `HOSTS` 里哪一项只由多机配置判法
//! `scripts/crash/crash-point-replay-shard-configuration-check.sh --emit-addresses` 认（这一台网卡上的地址与 `HOSTS` 求交、恰好一项；全仓只这一份判法，
//! 这里起它、读它打的 `THIS_HOST_ADDRESS`，不另判，[`MultiHostAddresses::emitted_by_the_configuration_check`]）。

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
            .is_some_and(|val| val == "1")
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

/// 配置的显卡优先级：环境变量 `GPU_CARDS_BY_PRIORITY` 设了就照它（服务分给这一条的卡经环境交下来），
/// 没设时清没清场（[`GpuQuiesce::of_this_process`]）都不读多机配置（[`multi_host_configuration_file`]）里的卡（清场与分卡由服务做）：
/// GPU 没开、两份显卡先后都没写是 `None`（本机能用的卡按空闲显存从大到小用，不设额度），开了或写了报错。
/// 写了卡的，`HOSTS` 与这一台取调用方那份多机配置上判法认出的（[`multi_host_addresses_of_the_caller`]）。
///
/// # Errors
/// 写了而写法不对或地址不在 `HOSTS` 里（[`parse_cards_by_priority`]）；写了卡而判法认不出这一台；
/// 没经服务给卡而多机配置开了 GPU 或写了两份显卡先后里的一份（[`cards_by_priority_from`]）。
pub fn cards_by_priority_from_configuration_or_environment(
    configuration_file: Option<&Path>,
) -> Result<Option<ConfiguredCards>, String> {
    cards_by_priority_from(
        std::env::var(CARDS_BY_PRIORITY_KEY).ok(),
        configuration_file,
        GpuQuiesce::of_this_process(),
        &multi_host_addresses_of_the_caller,
    )
}

/// [`cards_by_priority_from_configuration_or_environment`] 去掉读环境的那一层：环境变量的值、多机配置、清没清场、主机数组与这一台都由调用方给
/// （`addresses_of_the_hosts` 只在写了卡、要读卡项时调）。
/// 环境变量没给（没经服务给卡）而多机配置开了 GPU（[`GPU_SWITCH_KEY`]）或写了两份显卡先后（[`CARDS_BY_PRIORITY_KEY`]、
/// [`CARDS_BY_PRIORITY_WHEN_QUIESCED_KEY`]）里的一份：清没清场都报错（运行条件不满足，退 78）、出路写经服务交，不照多机配置自己挑卡，
/// 也不照本机空闲显存挑卡（服务清场时一并交卡，清场与分卡由服务做，`.claude/rules/verification.md`「凡用 GPU 先清场，跑完不复原」）；
/// GPU 没开、两份都没写的是 `None`。
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
    let text = match (environment_value, quiesce) {
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
                    "没经服务给卡（环境里没有 {CARDS_BY_PRIORITY_KEY}）、{quiesce_said}，多机配置{}却开了 GPU（{GPU_SWITCH_KEY}=1）或写了 {CARDS_BY_PRIORITY_KEY} 或 {CARDS_BY_PRIORITY_WHEN_QUIESCED_KEY}：\
                     不照多机配置自己挑卡，也不照本机空闲显存挑卡，清场与分卡由服务做（运行条件不满足，退 78）。\
                     出路：整条经 task-server/task submit 交，服务清场、分卡；要卡的入库装置二进制在 src/bin/<名字>.rs 文件头写 `// task-server-gpu-arguments: …`",
                    configuration_file.map_or_else(String::new, |path| format!(" {}", path.display()))
                ));
            }
            None
        }
    };
    match text {
        Some(text) if !text.trim().is_empty() => {
            parse_cards_by_priority(&text, &addresses_of_the_hosts()?).map(Some)
        }
        Some(_) | None => Ok(None),
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

    /// 服务分给这个进程的卡：这一份记下的环境变量 `GPU_CARDS_BY_PRIORITY` 原样（没经服务给卡是 `None`）。
    /// 环境守卫清掉了这个变量、要上卡的用例照它设回去（`crate::crash_amplification::CrashAmplificationEnvironment::set_the_multi_host_configuration_and_the_cards_of_the_caller`）。
    #[must_use]
    pub fn cards_handed_out_by_the_service(&self) -> Option<&str> {
        self.cards_by_priority.as_deref()
    }

    /// 配置的显卡优先级（同 [`cards_by_priority_from_configuration_or_environment`]，环境那一层读的是这一份；`HOSTS` 与这一台同样取
    /// [`multi_host_addresses_of_the_caller`]）。
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
    /// 报错、出路写经服务交，不照多机配置清场之后那一份自己挑卡，也不照本机空闲显存挑卡（服务清场时一并交卡，
    /// `.claude/rules/verification.md`「凡用 GPU 先清场，跑完不复原」）；GPU 没开、两份都没写的、没有多机配置的是没配；服务交了卡（环境变量给了）照它。
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
                    && refusal.contains("出路：整条经 task-server/task submit 交，服务清场、分卡")
                    && refusal.contains(
                        "要卡的入库装置二进制在 src/bin/<名字>.rs 文件头写 `// task-server-gpu-arguments: …`"
                    ),
                "报错写明清过场、点名 GPU 开关与清场之后那个键，出路写经服务交、要卡的二进制在文件头声明（{}）：{refusal}",
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

    /// 没经服务给卡（环境变量 `GPU_CARDS_BY_PRIORITY` 没给）、也没清场：多机配置开了 GPU 或写了不清场那一份显卡先后的，报错、出路写经服务交
    /// （服务清场、分卡；要卡的入库装置二进制在文件头声明），不照多机配置自己挑卡（`.claude/rules/verification.md`「凡用 GPU 先清场，跑完不复原」）；
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
                    && refusal.contains("出路：整条经 task-server/task submit 交，服务清场、分卡")
                    && refusal.contains(
                        "要卡的入库装置二进制在 src/bin/<名字>.rs 文件头写 `// task-server-gpu-arguments: …`"
                    ),
                "报错点名 GPU 开关与显卡先后那个键，出路写经服务交、要卡的二进制在文件头声明（{}）：{refusal}",
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
}
