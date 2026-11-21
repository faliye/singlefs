//! 本包真卡对拍用例共用的判定库存取：产品测试自己的一族键（[`KEY_FAMILY_OF_THE_PRODUCT_TEST_PAIRINGS`]），经 prune-server 的容错客户端
//! （`prune_server::reliable_client`）读写全仓那一份库（`.claude/rules/verification.md`「判定库只有一份：全仓统一写进 prune-server 的那份 RocksDB」），
//! 与验证包崩溃判定库的键空间（键以表标签字节起头）互不相见：这一族的键都是以族名起头的 UTF-8 文字。
//!
//! 对拍分两段：录入段把一条用例的输入（一边是 CPU 参照的，连同参照在 CPU 上算的结果）按内容摘要写进库、按录入键登清单，只录那一趟到此返回、不开卡；
//! 判段照同一份输入现算录入键、读清单、按摘要读回，开卡之后交给 GPU 的与比的参照都是读回的那一份。段由环境变量 [`RUN_STAGE_ENVIRONMENT_VARIABLE`]
//! 定（`record` 只录、`judge` 只判、不设是同一个进程里先录后判）。判段判完的对拍结论也入库：判段先领卡、在这个进程的 Vulkan 实例上读交来的卡的
//! 适配器（只枚举、不开设备），这一台没交到卡的报「本次未跑」；库里有两边一致的复用、不开卡；没有、或记着红的，开卡、跑持卡判、照同一把键存
//! （红的照存：记红，下一趟现判）。
//!
//! 键照全仓同一套组成（D13（验证路线） 已定项 17）：
//! - 录入键 = SHA-256（场景标识 ‖ 录入输入摘要），与验证包的录入键同一种组成（[`recording_key_of_the_scenario_and_the_inputs`]）。场景标识是
//!   这条对拍用例的名字；录入输入摘要是输入的 SHA-256，一边是参照的另拼上参照一侧的判法单元指纹（参照是录入段在 CPU 上算好存进库的，
//!   它的代码改了，录入键变，库里旧的参照不再被读到）。除了经输入摘要拼进来的参照一侧指纹，存取代码、产品代码一个单元都不进录入键：
//!   只有输入的对拍，输入没变录入键就不变。
//!   参照一侧拒算的：键只拼场景与输入，照录、不凭命中跳过（[`RecordingKeyOfThisRun::skips_the_recording_on_a_hit`]）。
//! - 对拍结论的键 = 读回输入的摘要 ‖ 参照一侧的判法单元指纹 ‖ GPU 一侧的判法单元指纹（这条用例的持卡判入口那一组）‖ 交来的卡的适配器摘要
//!   （[`AdaptersOfTheHandedOverGpus`]：设备名、厂商号、设备号、驱动名、驱动信息、后端；判法单元的键只管代码，换一种卡或换一版驱动不复用）；
//!   哪一边拒算的、驱动没报驱动名或驱动信息的照判、不入库。
//!
//! 两张判法单元键表由测试支撑包 `singlefs-background-gpu-pairing-keys` 的构建脚本照 `tests/pairing_units_entries.tsv` 的登记逐组算好、写进它的 `OUT_DIR`，
//! 那个包编进来交出，这里读（产品 GPU 包自己的构建脚本只交特性、不算键）。
//!
//! 这一族在服务那份库的结论表里的行（键都是 `<族名>/<种类>/…`，十六进制是小写）：
//!
//! | 行 | 键 | 值 |
//! |---|---|---|
//! | 字节 | `<族名>/bytes/<SHA-256>` | 对拍的输入或参照的结果，键是这份字节的摘要；长过一帧的照分段存（`prune_server::reliable_client` 的大值分段） |
//! | 清单 | `<族名>/manifest/<录入键>` | `input=<摘要>` 一行，一边是参照的再一行 `reference=<摘要>` |
//! | 对拍结论 | `<族名>/conclusion/<输入摘要>/<参照一侧指纹>/<GPU 一侧指纹>/<适配器摘要>` | `agreement=both_sides_agree` 或 `agreement=sides_disagree` 一行，接结论的字节 |
//!
//! 连哪一处：本机那个 prune-server 的 socket（`prune_server::configuration::default_socket_path`）在就走它，不在走多机配置里的
//! `PRUNE_SERVER_TCP_ADDRESS`；服务读写不成照客户端的退出码退出进程（75 服务不可用），不当成「库里没有」。测这份存取本身的用例
//! 在这个进程里起一份临时服务（[`PairingLibraryOfTheProductTests::on_a_temporary_service_in_this_process`]），不连真服务；
//! 它的 TCP 绑多机配置 `HOSTS` 里这一台的地址（只由多机配置判法认，取不到就不起服务、报错带出路，不补缺省地址）。
//! 要先 `mod common_discrete_gpus_of_this_host;` 再 mod 进这一份。
#![allow(dead_code, reason = "几份测试文件各自 mod 进来，只用到其中一部分")]

use std::path::PathBuf;
use std::sync::{Mutex, PoisonError};

use prune_server::client::RouteAddress;
use prune_server::configuration::{
    create_directory_holding_a_socket, default_multi_host_configuration_path, default_socket_path,
    read_configuration_value, NETWORK_ADDRESS_KEY,
};
use prune_server::protocol::Table;
use prune_server::reliable_client::{
    ReliableClient, ReliableClientSettings, DEFAULT_JOURNAL_FILE_NAME,
    OTHER_CLIENT_FAILURE_EXIT_CODE,
};
use prune_server::server::{start_in_this_process, ServiceInThisProcess};
use singlefs_unit_graph::entry_groups::{parse_entry_group_table, ReadBackEntryGroupVerdict};
use singlefs_unit_graph::sha256::{sha256_of, Sha256Digest, DIGEST_BYTES};

use crate::common_discrete_gpus_of_this_host::{
    cards_leased_on_demand_unless_handed_over, handed_over_gpus_or_report_not_run,
    this_host_address_by_the_configuration_check,
};

/// 两张判法单元键表（参照一侧、GPU 一侧（组名就是用例名））：测试支撑包的构建脚本照 `tests/pairing_units_entries.tsv` 算好，
/// 那个包编进来交出。
pub use singlefs_background_gpu_pairing_keys::{
    GPU_SIDE_UNITS_KEYS_TEXT, REFERENCE_UNITS_KEYS_TEXT,
};

/// 产品测试这一族在服务那份库里的键前缀（族名）。
pub const KEY_FAMILY_OF_THE_PRODUCT_TEST_PAIRINGS: &str = "background-gpu-pairing";

/// 这一族的行都写在服务三张表里的结论那一张（与 Python 各族同一张，键以族名起头、互不相撞）。
const PRUNE_SERVER_TABLE: Table = Table::Conclusions;

/// 一帧写最多装多少字节的键值（服务的帧长上限 64 MiB，留出帧头与每条的表、长度字段；与验证包崩溃判定库切帧同一个数）。
const PRUNE_SERVER_WRITE_FRAME_BYTES: usize = 32 << 20;

/// 选段的那个环境变量（统一任务服务与门禁分段跑时设它，与验证包崩溃判定库读的是同一个）。
pub const RUN_STAGE_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_CRASH_AMPLIFICATION_STAGE";

/// 读写 prune-server 不成、照退出码退出进程之前在标准错误打的那一行：`<前缀> exit_code=<码> <说明>`。
pub const PRUNE_SERVER_FAILURE_LINE_PREFIX: &str =
    "PRODUCT_TEST_PAIRING_LIBRARY_PRUNE_SERVER_FAILURE";

// ============================== 选段 ==============================

/// 这一趟跑哪一段（[`RUN_STAGE_ENVIRONMENT_VARIABLE`]）：`record` 只跑录入段（CPU 上造输入、算参照、写进库），`judge` 只跑判段（从库里读），
/// 没设两段先后跑，中间照样经库交接、不在内存里交数据。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunStage {
    RecordOnly,
    JudgeOnly,
    RecordThenJudge,
}

impl RunStage {
    /// 变量的值（`None` 是没设）换成段。
    ///
    /// # Errors
    /// 设了而不是 `record`、`judge`。
    pub fn of_the_variable_value(value: Option<&str>) -> Result<Self, String> {
        match value {
            None => Ok(Self::RecordThenJudge),
            Some("record") => Ok(Self::RecordOnly),
            Some("judge") => Ok(Self::JudgeOnly),
            Some(other) => Err(format!(
                "{RUN_STAGE_ENVIRONMENT_VARIABLE}={other} 认不得：只认 record（只跑录入段）、judge（只跑判段），或不设（两段先后跑）"
            )),
        }
    }

    /// 这个进程的选段。
    ///
    /// # Panics
    /// [`RUN_STAGE_ENVIRONMENT_VARIABLE`] 设了而认不得。
    #[must_use]
    pub fn of_this_process() -> Self {
        Self::of_the_variable_value(
            std::env::var(RUN_STAGE_ENVIRONMENT_VARIABLE)
                .ok()
                .as_deref(),
        )
        .unwrap_or_else(|refusal| panic!("{refusal}"))
    }

    #[must_use]
    pub fn records(self) -> bool {
        matches!(self, Self::RecordOnly | Self::RecordThenJudge)
    }

    #[must_use]
    pub fn judges(self) -> bool {
        matches!(self, Self::JudgeOnly | Self::RecordThenJudge)
    }
}

// ============================== 判法单元键与录入键 ==============================

/// 一组判法单元的键：构建脚本照 `singlefs-unit-graph` 算出的那一把指纹（组成清单在键表里，这里只当键用）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PairingSideUnitsKey {
    pub fingerprint: Sha256Digest,
}

impl PairingSideUnitsKey {
    /// 构建脚本写下的键表（`table_text`）里组 `group_name` 那一把。
    ///
    /// # Errors
    /// 表认不回来；表里没有这一组；这一组拒算或没编进来。交回那一句（调用方当拒算处置：照录、照判、不复用）。
    pub fn of_the_group_in_the_table_text(
        table_text: &str,
        group_name: &str,
    ) -> Result<Self, String> {
        let table = parse_entry_group_table(table_text)
            .map_err(|reason| format!("判法单元键表认不回来：{reason}"))?;
        let group = table
            .groups
            .iter()
            .find(|group_of_the_table| group_of_the_table.group_name == group_name)
            .ok_or_else(|| format!("判法单元键表里没有组 {group_name}"))?;
        match &group.verdict {
            ReadBackEntryGroupVerdict::Keyed { key } => Ok(Self {
                fingerprint: Sha256Digest(*key),
            }),
            ReadBackEntryGroupVerdict::Refused { refusal_lines } => Err(format!(
                "组 {group_name} 的判法单元拒算：{}",
                refusal_lines.join("；")
            )),
            ReadBackEntryGroupVerdict::NotCompiled { reason } => {
                Err(format!("组 {group_name} 这一次没编进来：{reason}"))
            }
        }
    }
}

/// 录入键（32 字节摘要）：清单那一行的键里的那一段。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecordingKey(pub Sha256Digest);

/// 这一趟的录入键与凭它跳不跳过录入。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordingKeyOfThisRun {
    pub key: RecordingKey,
    /// 录入键下已有清单就跳过录入。只有参照一侧的判法单元拒算时是假（照录、不复用，[`recording_key_of_the_pairing_input_and_the_reference_units`]）。
    pub skips_the_recording_on_a_hit: bool,
    /// 参照一侧拒算时那一句。
    pub refusal: Option<String>,
}

/// 只有输入、没有参照的对拍的录入输入摘要：输入的 SHA-256。
#[must_use]
pub fn recording_inputs_digest_of_the_input(input_bytes: &[u8]) -> Sha256Digest {
    sha256_of(input_bytes)
}

/// 一边是参照的对拍的录入输入摘要：输入的摘要与参照一侧的判法单元指纹拼起来取 SHA-256。参照的代码变了、或输入变了，录入键都变。
#[must_use]
pub fn recording_inputs_digest_of_the_input_and_the_reference_units(
    input_bytes: &[u8],
    reference_units: &PairingSideUnitsKey,
) -> Sha256Digest {
    sha256_of(
        format!(
            "singlefs background gpu pairing input with the reference\nreference_units={}\ninput={}\n",
            reference_units.fingerprint.hexadecimal(),
            sha256_of(input_bytes).hexadecimal()
        )
        .as_bytes(),
    )
}

/// 录入键的组成只写在这一处：场景标识（`scenario`：这条对拍用例的名字）与录入输入摘要（`inputs_digest`：
/// [`recording_inputs_digest_of_the_input`] 或 [`recording_inputs_digest_of_the_input_and_the_reference_units`]）拼起来取 SHA-256，
/// 与验证包的录入键同一种组成。这一处不拼任何代码单元（存取代码、产品代码与测试服务的代码都不进；参照一侧的指纹只经 `inputs_digest` 进来）：
/// 输入摘要没变，录入键就不变、命中就不再录；输入摘要变了，或换一条场景，键就变、照录。交回的 `skips_the_recording_on_a_hit` 恒是真。
#[must_use]
pub fn recording_key_of_the_scenario_and_the_inputs(
    scenario: &str,
    inputs_digest: &Sha256Digest,
) -> RecordingKeyOfThisRun {
    RecordingKeyOfThisRun {
        key: RecordingKey(sha256_of(
            format!(
                "singlefs background gpu pairing recording key\nscenario={scenario}\ninputs={}\n",
                inputs_digest.hexadecimal()
            )
            .as_bytes(),
        )),
        skips_the_recording_on_a_hit: true,
        refusal: None,
    }
}

/// 一边是参照的对拍的录入键（`scenario` 是这条对拍用例的名字）：参照一侧算得出时输入摘要照
/// [`recording_inputs_digest_of_the_input_and_the_reference_units`]；参照一侧拒算时输入摘要只取输入、照录、不复用。
#[must_use]
pub fn recording_key_of_the_pairing_input_and_the_reference_units(
    scenario: &str,
    input_bytes: &[u8],
    reference_units: &Result<PairingSideUnitsKey, String>,
) -> RecordingKeyOfThisRun {
    match reference_units {
        Ok(reference_units) => recording_key_of_the_scenario_and_the_inputs(
            scenario,
            &recording_inputs_digest_of_the_input_and_the_reference_units(
                input_bytes,
                reference_units,
            ),
        ),
        Err(sentence) => RecordingKeyOfThisRun {
            skips_the_recording_on_a_hit: false,
            refusal: Some(format!("参照一侧的判法单元拒算：{sentence}")),
            ..recording_key_of_the_scenario_and_the_inputs(
                scenario,
                &recording_inputs_digest_of_the_input(input_bytes),
            )
        },
    }
}

// ============================== 这一族的键 ==============================

fn bytes_key(digest: &Sha256Digest) -> Vec<u8> {
    format!(
        "{KEY_FAMILY_OF_THE_PRODUCT_TEST_PAIRINGS}/bytes/{}",
        digest.hexadecimal()
    )
    .into_bytes()
}

fn manifest_key(recording_key: &RecordingKey) -> Vec<u8> {
    format!(
        "{KEY_FAMILY_OF_THE_PRODUCT_TEST_PAIRINGS}/manifest/{}",
        recording_key.0.hexadecimal()
    )
    .into_bytes()
}

/// 对拍结论那一行的键：读回输入的摘要、参照一侧与 GPU 一侧的判法单元指纹、交来的卡的适配器摘要。
#[must_use]
pub fn conclusion_key(
    input_digest: &Sha256Digest,
    reference_units: &PairingSideUnitsKey,
    gpu_side_units: &PairingSideUnitsKey,
    adapters: &AdaptersOfTheHandedOverGpus,
) -> Vec<u8> {
    format!(
        "{KEY_FAMILY_OF_THE_PRODUCT_TEST_PAIRINGS}/conclusion/{}/{}/{}/{}",
        input_digest.hexadecimal(),
        reference_units.fingerprint.hexadecimal(),
        gpu_side_units.fingerprint.hexadecimal(),
        adapters.digest.hexadecimal()
    )
    .into_bytes()
}

// ============================== 交来的卡的适配器 ==============================

/// 读交来的卡的适配器信息时等枚举的上限（只枚举、不开设备）。
const WAIT_LIMIT_OF_READING_THE_ADAPTERS: std::time::Duration = std::time::Duration::from_secs(60);

/// 交来的一张卡在 wgpu 的适配器信息里报的那几样：设备名、厂商号、设备号、驱动名、驱动信息（Vulkan 后端是
/// `VkPhysicalDeviceDriverProperties` 的 driverName 与 driverInfo，驱动版本在后者里）、后端。卡的总线地址不在里面：同型号、同驱动的两张卡算同一种。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AdapterOfAHandedOverGpu {
    pub name: String,
    pub vendor: u32,
    pub device: u32,
    pub driver: String,
    pub driver_information: String,
    pub backend: String,
}

/// 对拍结论键里交来的卡那一段：各卡适配器描述去重、排好之后的摘要。判法单元的键只管代码，不管跑在哪种卡、哪个版本的驱动上；
/// 同一份代码换一种卡或换一版驱动，GPU 一侧走的是另一份机器码，库里的结论不拿来复用。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AdaptersOfTheHandedOverGpus {
    pub digest: Sha256Digest,
}

/// `adapters`（交来的每张卡一项，次序与重复不论）的摘要。
///
/// # Errors
/// 一张卡都没有；有一张的驱动名或驱动信息是空的（驱动没报，读不出是哪一版）：交回那一句，调用方照判、结论不入库。
pub fn adapters_of_the_handed_over_gpus(
    adapters: &[AdapterOfAHandedOverGpu],
) -> Result<AdaptersOfTheHandedOverGpus, String> {
    if adapters.is_empty() {
        return Err("一张交来的卡都没有，读不出适配器".to_string());
    }
    if let Some(not_reported) = adapters
        .iter()
        .find(|adapter| adapter.driver.is_empty() || adapter.driver_information.is_empty())
    {
        return Err(format!(
            "交来的卡「{}」的驱动没报驱动名或驱动信息（driver={:?} driver_info={:?}），读不出是哪一版驱动",
            not_reported.name, not_reported.driver, not_reported.driver_information
        ));
    }
    let distinct: std::collections::BTreeSet<&AdapterOfAHandedOverGpu> = adapters.iter().collect();
    let mut text = String::from("singlefs background gpu pairing adapters\n");
    for adapter in distinct {
        text.push_str(&format!(
            "name={:?}\tvendor={}\tdevice={}\tdriver={:?}\tdriver_info={:?}\tbackend={}\n",
            adapter.name,
            adapter.vendor,
            adapter.device,
            adapter.driver,
            adapter.driver_information,
            adapter.backend
        ));
    }
    Ok(AdaptersOfTheHandedOverGpus {
        digest: sha256_of(text.as_bytes()),
    })
}

/// 判段问库里的对拍结论之前，看到的交来的卡。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AdaptersSeenBeforeTheConclusion {
    /// 这一台交到了卡：适配器的摘要，或读不全的那一句（照判、不入库）。
    Seen(Result<AdaptersOfTheHandedOverGpus, String>),
    /// 这一台没交到卡（打过 `NOT_RUN_NO_GPU_ADAPTER`）：结论的键算不出，既不复用也不判。
    NoGpuHandedOver,
}

/// 交给这个进程、属于这一台的卡，在这个进程的那一份 Vulkan 实例上读各自的适配器信息（只枚举，不开设备）。
///
/// # Panics
/// 同 `crate::common_discrete_gpus_of_this_host::handed_over_gpus_or_report_not_run`；枚举超过等的上限；刚对上的卡再枚举不见了。
fn adapters_seen_through_the_handed_over_gpus(test_name: &str) -> AdaptersSeenBeforeTheConclusion {
    let Some(handed_over) =
        handed_over_gpus_or_report_not_run(test_name, WAIT_LIMIT_OF_READING_THE_ADAPTERS)
    else {
        return AdaptersSeenBeforeTheConclusion::NoGpuHandedOver;
    };
    let enumerated = handed_over
        .vulkan_instance
        .vulkan_adapters_enumerated_within(WAIT_LIMIT_OF_READING_THE_ADAPTERS)
        .unwrap_or_else(|cause| panic!("读交来的卡的适配器，枚举超过等的上限：{cause:?}"));
    let adapters: Vec<AdapterOfAHandedOverGpu> = handed_over
        .gpus
        .iter()
        .map(|gpu| {
            let information = enumerated
                .iter()
                .map(wgpu::Adapter::get_info)
                .find(|information| information.device_pci_bus_id == gpu.pci_bus_address)
                .unwrap_or_else(|| {
                    panic!(
                        "交来的卡 {} 刚对上过本机的适配器，再枚举不见了",
                        gpu.pci_bus_address
                    )
                });
            AdapterOfAHandedOverGpu {
                name: information.name,
                vendor: information.vendor,
                device: information.device,
                driver: information.driver,
                driver_information: information.driver_info, // naming-lint:external wgpu 的 AdapterInfo 定的字段名，这里只读它
                backend: format!("{:?}", information.backend),
            }
        })
        .collect();
    AdaptersSeenBeforeTheConclusion::Seen(adapters_of_the_handed_over_gpus(&adapters))
}

/// 64 个小写或大写十六进制字符读回成摘要；别的写法是 `None`。
fn digest_of_the_hexadecimal(text: &str) -> Option<Sha256Digest> {
    if text.len() != DIGEST_BYTES * 2 || !text.is_ascii() {
        return None;
    }
    let mut digest = [0_u8; DIGEST_BYTES];
    for (position, byte) in digest.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[position * 2..position * 2 + 2], 16).ok()?;
    }
    Some(Sha256Digest(digest))
}

/// 一条用例登的清单：输入的摘要，一边是参照的另有参照的摘要。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PairingManifest {
    pub input: Sha256Digest,
    pub reference: Option<Sha256Digest>,
}

impl PairingManifest {
    fn to_bytes(self) -> Vec<u8> {
        let mut text = format!("input={}\n", self.input.hexadecimal());
        if let Some(reference) = self.reference {
            text.push_str(&format!("reference={}\n", reference.hexadecimal()));
        }
        text.into_bytes()
    }

    /// [`Self::to_bytes`] 的逆。
    ///
    /// # Errors
    /// 不是 UTF-8；开头不是 `input=<摘要>` 那一行；接着的那一行不是 `reference=<摘要>`；还有别的行。
    fn of_the_bytes(bytes: &[u8]) -> Result<Self, String> {
        let text = std::str::from_utf8(bytes).map_err(|_| "清单不是 UTF-8".to_string())?;
        let mut lines = text.lines();
        let input = lines
            .next()
            .and_then(|line| line.strip_prefix("input="))
            .and_then(digest_of_the_hexadecimal)
            .ok_or_else(|| format!("清单开头不是 input=<摘要> 那一行：{text:?}"))?;
        let reference = match lines.next() {
            None => None,
            Some(line) => Some(
                line.strip_prefix("reference=")
                    .and_then(digest_of_the_hexadecimal)
                    .ok_or_else(|| {
                        format!("清单 input 那一行之后不是 reference=<摘要>：{text:?}")
                    })?,
            ),
        };
        if lines.next().is_some() {
            return Err(format!("清单多出了行：{text:?}"));
        }
        Ok(Self { input, reference })
    }
}

// ============================== 库：连服务、读写 ==============================

/// 读写 prune-server 不成时怎么办。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PruneServerFailureHandling {
    /// 连全仓那一份（真卡用例）：打一行 [`PRUNE_SERVER_FAILURE_LINE_PREFIX`]、照退出码退出进程，服务修好重起之后由提交它的会话重交；
    /// 读失败不当成「库里没有」去现判。
    ExitTheProcess,
    /// 连这个进程里起的临时服务（测这份存取本身的用例）：当场 panic，这条用例红。
    PanicInTheTest,
}

/// 照 `handling` 处置一次读写不成。
fn prune_server_failed(
    handling: PruneServerFailureHandling,
    exit_code: i32,
    description: String,
) -> ! {
    match handling {
        PruneServerFailureHandling::ExitTheProcess => {
            eprintln!("{PRUNE_SERVER_FAILURE_LINE_PREFIX} exit_code={exit_code} {description}");
            std::process::exit(exit_code)
        }
        PruneServerFailureHandling::PanicInTheTest => {
            panic!("{PRUNE_SERVER_FAILURE_LINE_PREFIX} exit_code={exit_code} {description}")
        }
    }
}

/// 这一份库独占的一次性目录（容错客户端的追加日志；临时服务的 socket 与状态目录也在里面），丢掉时删。
struct ExclusiveDirectoryOfTheLibrary {
    path: PathBuf,
}

impl ExclusiveDirectoryOfTheLibrary {
    /// 建在 `TMPDIR` 底下（放 socket 嫌长时退到 `/tmp` 底下，`prune_server::configuration::create_directory_holding_a_socket`），
    /// 目录名 `<name_prefix>-<进程号>-<本进程内的序号>`，只有本用户进得去。
    ///
    /// # Panics
    /// 建不了。
    fn create(name_prefix: &str) -> Self {
        let path = create_directory_holding_a_socket(
            std::env::var_os("TMPDIR"),
            name_prefix,
            TEMPORARY_PRUNE_SERVER_SOCKET_FILE_NAME,
        )
        .unwrap_or_else(|problem| panic!("建不了判定库存取的一次性目录：{problem}"));
        Self { path }
    }
}

impl Drop for ExclusiveDirectoryOfTheLibrary {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

/// 临时服务的 socket 在一次性目录里的文件名。
const TEMPORARY_PRUNE_SERVER_SOCKET_FILE_NAME: &str = "socket";
/// 临时服务的状态目录（故障日志与库）在一次性目录里的名字。
const TEMPORARY_PRUNE_SERVER_STATE_DIRECTORY_NAME: &str = "state";

/// 在这个进程里起的一份临时 prune-server；丢掉时停服，停不干净是缺陷（还有线程拿着服务），不在 panic 中就 panic。
struct TemporaryPruneServer {
    service: Option<ServiceInThisProcess>,
}

impl Drop for TemporaryPruneServer {
    fn drop(&mut self) {
        let Some(service) = self.service.take() else {
            return;
        };
        if let Err(problem) = service.stop() {
            if std::thread::panicking() {
                eprintln!("  ✗ 临时 prune-server 没停干净：{problem}");
            } else {
                panic!("临时 prune-server 没停干净：{problem}");
            }
        }
    }
}

/// 连哪一处：本机那个 prune-server 的 socket 在就走它；不在走多机配置（`SINGLEFS_MULTI_HOST_CONFIG`，没设照主工作树找）里的
/// `PRUNE_SERVER_TCP_ADDRESS`。没写就是没有，不补缺省。
///
/// # Errors
/// socket 不在而多机配置读不了、或没写地址。
fn prune_server_route_of_the_configuration() -> Result<RouteAddress, String> {
    let socket_path = default_socket_path().ok();
    if let Some(socket_path) = socket_path.as_ref().filter(|path| path.exists()) {
        return Ok(RouteAddress::UnixSocket(socket_path.clone()));
    }
    let configuration_path = default_multi_host_configuration_path();
    match read_configuration_value(&configuration_path, NETWORK_ADDRESS_KEY)? {
        Some(address) if !address.is_empty() => Ok(RouteAddress::Network(address)),
        Some(_) | None => Err(format!(
            "没有 prune-server 的 socket（{}），多机配置 {} 里也没有 {NETWORK_ADDRESS_KEY}。出路：本机要有服务的 socket（prune-server/control check 看服务在不在跑），\
             第二台要有多机配置里的 {NETWORK_ADDRESS_KEY}；判定库没有不走服务的那条路",
            socket_path.map_or_else(
                || "XDG_RUNTIME_DIR 没设".to_string(),
                |path| path.display().to_string()
            ),
            configuration_path.display()
        )),
    }
}

/// 临时服务的 TCP 绑哪个地址：多机配置 `HOSTS` 里这一台的地址 `this_host_address` 接内核挑的端口 `:0`（几份同时起互不撞端口，
/// 两台的 GPU 批处理服务都连得到），IPv6 地址带方括号（同验证包 `crash_library` 的 `network_address_of_a_temporary_service_on`；
/// 本包测试不依赖验证包，各写各的）。
#[must_use]
pub fn network_address_of_a_temporary_service_on(this_host_address: &str) -> String {
    if this_host_address.contains(':') {
        format!("[{this_host_address}]:0")
    } else {
        format!("{this_host_address}:0")
    }
}

/// 产品测试这一族的判定库：一个容错客户端连着服务（全仓那一份，或这个进程里的临时服务）。读写都经它；读失败不当成「库里没有」。
/// 字段按放手的次序排：先放客户端，再停临时服务，最后删一次性目录。
pub struct PairingLibraryOfTheProductTests {
    client: Mutex<ReliableClient>,
    library_name: String,
    failure_handling: PruneServerFailureHandling,
    temporary_service: Option<TemporaryPruneServer>,
    directory: ExclusiveDirectoryOfTheLibrary,
}

impl PairingLibraryOfTheProductTests {
    /// 连全仓那一份（[`prune_server_route_of_the_configuration`]）。连不上、读写不成照退出码退出进程。
    #[must_use]
    pub fn open_the_one_library() -> Self {
        let route = prune_server_route_of_the_configuration().unwrap_or_else(|problem| {
            prune_server_failed(
                PruneServerFailureHandling::ExitTheProcess,
                OTHER_CLIENT_FAILURE_EXIT_CODE,
                problem,
            )
        });
        Self::connected(
            route,
            ExclusiveDirectoryOfTheLibrary::create("product-test-pairing-library"),
            None,
            PruneServerFailureHandling::ExitTheProcess,
        )
    }

    /// 在这个进程里起一份临时 prune-server（状态目录、库与 socket 都在这一份独占的一次性目录里），TCP 绑多机配置 `HOSTS` 里这一台的地址
    /// （[`this_host_address_by_the_configuration_check`]）上内核挑的端口，连它；从空库起，不连真服务。
    ///
    /// # Panics
    /// 取不到这一台的地址（[`Self::on_a_temporary_service_bound_to_this_host`] 交回的说明连同出路）；起不来、连不上、读写不成。
    #[must_use]
    pub fn on_a_temporary_service_in_this_process(name_prefix: &str) -> Self {
        Self::on_a_temporary_service_bound_to_this_host(
            name_prefix,
            this_host_address_by_the_configuration_check,
        )
        .unwrap_or_else(|problem| panic!("{problem}"))
    }

    /// 同 [`Self::on_a_temporary_service_in_this_process`]，这一台的地址由 `this_host_address` 取。先取地址：取不到就交回错误，
    /// 不建一次性目录、不起服务；取到的地址照用（[`network_address_of_a_temporary_service_on`]），不补缺省、不写死。
    ///
    /// # Errors
    /// `this_host_address` 交回错误：说明接上它的原话与出路。
    ///
    /// # Panics
    /// 一次性目录建不了；服务起不来（说明带 socket 路径与 TCP 地址）、连不上、读写不成。
    pub fn on_a_temporary_service_bound_to_this_host(
        name_prefix: &str,
        this_host_address: impl FnOnce() -> Result<String, String>,
    ) -> Result<Self, String> {
        let this_host = this_host_address().map_err(|problem| {
            format!(
                "临时 prune-server 的 TCP 要绑多机配置 HOSTS 里这一台的地址，取不到：{problem}。出路：多机配置（SINGLEFS_MULTI_HOST_CONFIG，\
                 没设照主工作树找）的 HOSTS 里写上这一台网卡上的地址，bash scripts/crash/crash-point-replay-shard-configuration-check.sh \
                 --emit-addresses 打得出 THIS_HOST_ADDRESS 再跑；不补缺省地址"
            )
        })?;
        let network_address = network_address_of_a_temporary_service_on(&this_host);
        let directory = ExclusiveDirectoryOfTheLibrary::create(name_prefix);
        let socket_path = directory.path.join(TEMPORARY_PRUNE_SERVER_SOCKET_FILE_NAME);
        let service = start_in_this_process(
            &directory
                .path
                .join(TEMPORARY_PRUNE_SERVER_STATE_DIRECTORY_NAME),
            &socket_path,
            &network_address,
        )
        .unwrap_or_else(|failure| {
            let (problem, next_step) = failure.describe();
            panic!(
                "在这个进程里起不来临时 prune-server（{}，TCP {network_address}）：{problem}；{next_step}",
                socket_path.display()
            )
        });
        Ok(Self::connected(
            RouteAddress::UnixSocket(socket_path),
            directory,
            Some(TemporaryPruneServer {
                service: Some(service),
            }),
            PruneServerFailureHandling::PanicInTheTest,
        ))
    }

    fn connected(
        route: RouteAddress,
        directory: ExclusiveDirectoryOfTheLibrary,
        temporary_service: Option<TemporaryPruneServer>,
        failure_handling: PruneServerFailureHandling,
    ) -> Self {
        let library_name = match &route {
            RouteAddress::UnixSocket(socket_path) => format!("unix:{}", socket_path.display()),
            RouteAddress::Network(address) => format!("tcp:{address}"),
        };
        let settings =
            ReliableClientSettings::new(route, directory.path.join(DEFAULT_JOURNAL_FILE_NAME));
        let client = ReliableClient::open(settings).unwrap_or_else(|failure| {
            prune_server_failed(
                failure_handling,
                failure.exit_code(),
                format!("连 prune-server（{library_name}）：{failure}"),
            )
        });
        Self {
            client: Mutex::new(client),
            library_name,
            failure_handling,
            temporary_service,
            directory,
        }
    }

    fn failed(&self, exit_code: i32, description: String) -> ! {
        prune_server_failed(self.failure_handling, exit_code, description)
    }

    /// 点读一把键，分段存的拼回整值（`ReliableClient::read_assembling_segmented_values`）。
    fn read_value(&self, key: Vec<u8>) -> Option<Vec<u8>> {
        let mut client = self.client.lock().unwrap_or_else(PoisonError::into_inner);
        let mut values = client
            .read_assembling_segmented_values(&[(PRUNE_SERVER_TABLE, key)])
            .unwrap_or_else(|failure| {
                self.failed(
                    failure.exit_code(),
                    format!("读库（{}）：{failure}", self.library_name),
                )
            });
        match (values.pop(), values.pop()) {
            (Some(value), None) => value,
            (None, _) | (Some(_), Some(_)) => self.failed(
                OTHER_CLIENT_FAILURE_EXIT_CODE,
                format!("读库（{}）问了一把键，回的值不是一个", self.library_name),
            ),
        }
    }

    /// 成批写键值：长过一段的值先分段写下（`ReliableClient::write_segments_of_large_values`），再把它交回的清单与别的行按
    /// [`PRUNE_SERVER_WRITE_FRAME_BYTES`] 切帧，一帧一帧等确认。
    fn write_values(&self, rows: Vec<(Vec<u8>, Vec<u8>)>) {
        let mut client = self.client.lock().unwrap_or_else(PoisonError::into_inner);
        let entries = rows
            .into_iter()
            .map(|(key, value)| (PRUNE_SERVER_TABLE, key, value))
            .collect();
        let entries_to_write_last = client
            .write_segments_of_large_values(entries, PRUNE_SERVER_WRITE_FRAME_BYTES)
            .unwrap_or_else(|failure| {
                self.failed(
                    failure.exit_code(),
                    format!("写库（{}）的大值分段：{failure}", self.library_name),
                )
            });
        let mut frame: Vec<(Table, Vec<u8>, Vec<u8>)> = Vec::new();
        let mut frame_bytes = 0_usize;
        for (table, key, value) in entries_to_write_last {
            let entry_bytes = key.len() + value.len();
            if !frame.is_empty() && frame_bytes + entry_bytes > PRUNE_SERVER_WRITE_FRAME_BYTES {
                self.write_one_frame(&mut client, &std::mem::take(&mut frame));
                frame_bytes = 0;
            }
            frame.push((table, key, value));
            frame_bytes += entry_bytes;
        }
        if !frame.is_empty() {
            self.write_one_frame(&mut client, &frame);
        }
    }

    /// 写一帧、等它确认。
    fn write_one_frame(&self, client: &mut ReliableClient, entries: &[(Table, Vec<u8>, Vec<u8>)]) {
        client
            .submit_write_entries(entries)
            .unwrap_or_else(|failure| {
                self.failed(
                    failure.exit_code(),
                    format!("写库（{}）：{failure}", self.library_name),
                )
            });
        client.wait_all_acknowledged().unwrap_or_else(|failure| {
            self.failed(
                failure.exit_code(),
                format!("等写库（{}）的确认：{failure}", self.library_name),
            )
        });
    }

    /// 录入键下登的清单；没登过交回 `None`。
    ///
    /// # Panics
    /// 清单解不出（字节坏了或写法换了：出路是重跑录入段）。
    fn manifest_under(&self, recording_key: &RecordingKey) -> Option<PairingManifest> {
        self.read_value(manifest_key(recording_key)).map(|bytes| {
            PairingManifest::of_the_bytes(&bytes)
                .unwrap_or_else(|reason| panic!("录入键下的清单解不出（重跑录入段）：{reason}"))
        })
    }

    /// 按摘要读回一份字节（输入或参照）、核摘要。
    ///
    /// # Panics
    /// 库里没有（录入段没跑完）；读回的字节摘要对不上。
    fn bytes_read_back(&self, digest: &Sha256Digest, what: &str) -> Vec<u8> {
        let bytes = self.read_value(bytes_key(digest)).unwrap_or_else(|| {
            panic!(
                "库（{}）里没有清单点名的{what}（{}）：先跑录入段（{RUN_STAGE_ENVIRONMENT_VARIABLE}=record，或不设、两段先后跑）",
                self.library_name,
                digest.hexadecimal()
            )
        });
        assert_eq!(
            sha256_of(&bytes),
            *digest,
            "库（{}）里读回的{what}摘要对不上清单点名的那一个",
            self.library_name
        );
        bytes
    }
}

// ============================== 两段：录入、读回 ==============================

/// 录入段要写进库的一条用例的字节：输入，一边是参照的另有参照在 CPU 上算的结果。
pub struct BytesOfThePairing {
    pub input: Vec<u8>,
    pub reference: Option<Vec<u8>>,
}

/// 两段过一次库：录入段照录入键找清单，有而凭命中可跳过（[`RecordingKeyOfThisRun::skips_the_recording_on_a_hit`]）的打 `PRODUCT_TEST_PAIRING_RECORDING_REUSED`、不再录；
/// 没有的调 `bytes_of_the_pairing` 造字节，按摘要写进库，再按录入键登清单（清单最后写：写到一半死掉的那一趟没有清单，下一趟照录），
/// 打 `PRODUCT_TEST_PAIRING_RECORDED`。只录那一趟交回 `None`；判段照录入键读回清单交回。
///
/// # Panics
/// 判段的录入键下没有清单（录入段没跑，或输入、参照一侧变了）；读写不成照 [`PairingLibraryOfTheProductTests`] 的处置。
pub fn manifest_recorded_or_reused_under_the_recording_key(
    library: &PairingLibraryOfTheProductTests,
    stage: RunStage,
    test_name: &str,
    recording: &RecordingKeyOfThisRun,
    bytes_of_the_pairing: impl FnOnce() -> BytesOfThePairing,
) -> Option<PairingManifest> {
    let recording_key_text = recording.key.0.hexadecimal();
    if let Some(refusal) = &recording.refusal {
        println!("PRODUCT_TEST_PAIRING_RECORDING_KEY_REFUSED test={test_name} {refusal}");
    }
    if stage.records() {
        let stored = if recording.skips_the_recording_on_a_hit {
            library.manifest_under(&recording.key)
        } else {
            None
        };
        match stored {
            Some(_manifest) => println!(
                "PRODUCT_TEST_PAIRING_RECORDING_REUSED test={test_name} recording_key={recording_key_text}"
            ),
            None => {
                let bytes = bytes_of_the_pairing();
                let manifest = PairingManifest {
                    input: sha256_of(&bytes.input),
                    reference: bytes.reference.as_deref().map(sha256_of),
                };
                let mut rows = vec![(bytes_key(&manifest.input), bytes.input)];
                if let (Some(reference_digest), Some(reference)) =
                    (manifest.reference, bytes.reference)
                {
                    rows.push((bytes_key(&reference_digest), reference));
                }
                library.write_values(rows);
                library.write_values(vec![(manifest_key(&recording.key), manifest.to_bytes())]);
                println!(
                    "PRODUCT_TEST_PAIRING_RECORDED test={test_name} recording_key={recording_key_text}"
                );
            }
        }
    }
    if !stage.judges() {
        return None;
    }
    Some(library.manifest_under(&recording.key).unwrap_or_else(|| {
        panic!(
            "库（{}）里没有用例「{test_name}」这一份输入的清单（录入键 {recording_key_text}）：先跑录入段（{RUN_STAGE_ENVIRONMENT_VARIABLE}=record，或不设、两段先后跑）",
            library.library_name
        )
    }))
}

/// 从库里读回的对拍输入（连同它的摘要，对拍结论的键用它）与参照一侧在录入段算好的结果。
pub struct PairingInputAndReferenceReadBack {
    pub input: Vec<u8>,
    pub input_digest: Sha256Digest,
    pub reference: Vec<u8>,
}

/// 一边是参照的对拍过一次库：录入段把 `input_bytes` 与 `reference_of_the_input` 在 CPU 上算的结果写进库、登清单；判段读回两份交回。
/// 只录那一趟交回 `None`。
///
/// # Panics
/// 同 [`manifest_recorded_or_reused_under_the_recording_key`]；清单里没有参照（录入键撞上了只有输入的那一种清单）。
pub fn pairing_input_and_reference_through_the_library_under_the_recording_key(
    library: &PairingLibraryOfTheProductTests,
    stage: RunStage,
    test_name: &str,
    recording: &RecordingKeyOfThisRun,
    input_bytes: &[u8],
    reference_of_the_input: impl FnOnce(&[u8]) -> Vec<u8>,
) -> Option<PairingInputAndReferenceReadBack> {
    let manifest = manifest_recorded_or_reused_under_the_recording_key(
        library,
        stage,
        test_name,
        recording,
        || BytesOfThePairing {
            input: input_bytes.to_vec(),
            reference: Some(reference_of_the_input(input_bytes)),
        },
    )?;
    let reference_digest = manifest.reference.unwrap_or_else(|| {
        panic!("用例「{test_name}」录入键下的清单里没有参照：录入键的输入摘要两种对拍不该撞上")
    });
    Some(PairingInputAndReferenceReadBack {
        input: library.bytes_read_back(&manifest.input, "对拍输入"),
        input_digest: manifest.input,
        reference: library.bytes_read_back(&reference_digest, "对拍参照"),
    })
}

// ============================== 对拍结论入库 ==============================

/// 两边比下来一致还是不一致（调用方照它自己的比法判）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PairingAgreement {
    BothSidesAgree,
    SidesDisagree,
}

impl PairingAgreement {
    /// 存进库那一行开头的写法。
    fn stored_name(self) -> &'static str {
        match self {
            Self::BothSidesAgree => "both_sides_agree",
            Self::SidesDisagree => "sides_disagree",
        }
    }
}

/// 判一次对拍交回的：结论的字节（写法由调用方定），与两边比下来一致还是不一致。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PairingJudged {
    pub conclusion: Vec<u8>,
    pub agreement: PairingAgreement,
}

impl PairingJudged {
    fn to_stored_bytes(&self) -> Vec<u8> {
        let mut bytes = format!("agreement={}\n", self.agreement.stored_name()).into_bytes();
        bytes.extend_from_slice(&self.conclusion);
        bytes
    }

    /// [`Self::to_stored_bytes`] 的逆。
    ///
    /// # Errors
    /// 没有开头那一行，或那一行认不出。
    fn of_the_stored_bytes(bytes: &[u8]) -> Result<Self, String> {
        let line_end = bytes
            .iter()
            .position(|byte| *byte == b'\n')
            .ok_or_else(|| "对拍结论那一行没有 agreement=… 开头那一行".to_string())?;
        let agreement_line = String::from_utf8_lossy(&bytes[..line_end]);
        let agreement = match agreement_line.as_ref() {
            "agreement=both_sides_agree" => PairingAgreement::BothSidesAgree,
            "agreement=sides_disagree" => PairingAgreement::SidesDisagree,
            other => return Err(format!("对拍结论开头那一行认不出：{other:?}")),
        };
        Ok(Self {
            conclusion: bytes[line_end + 1..].to_vec(),
            agreement,
        })
    }
}

/// 一条对拍结论是从哪来的。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PairingConclusionSource {
    /// 库里同一把键下存着两边一致的，这一趟没判。
    ReusedFromTheLibrary,
    /// 这一趟判的（库里没有、记着红、或哪一边拒算），判完照键存进库（拒算的不存）。
    JudgedNow,
}

/// 对拍结论入库：键是（读回输入的摘要，参照一侧与 GPU 一侧的判法单元指纹，交来的卡的适配器摘要，[`conclusion_key`]），库里有两边一致的就复用、不调 `judge`；
/// 没有、或记着不一致（红）的，调 `judge` 判一遍、照同一把键存进库（红的照存：同一把键原有的那一行被换掉，下一趟照判）。
/// 两边哪一边拒算、适配器读不全：照判、不入库。交回判定与它从哪来。
///
/// # Panics
/// 库里那一行认不出；读写不成照 [`PairingLibraryOfTheProductTests`] 的处置。
pub fn pairing_conclusion_reused_or_judged(
    library: &PairingLibraryOfTheProductTests,
    input_digest: &Sha256Digest,
    reference_units: &Result<PairingSideUnitsKey, String>,
    gpu_side_units: &Result<PairingSideUnitsKey, String>,
    adapters: &Result<AdaptersOfTheHandedOverGpus, String>,
    judge: impl FnOnce() -> PairingJudged,
) -> (PairingJudged, PairingConclusionSource) {
    let (Ok(reference_units), Ok(gpu_side_units), Ok(adapters)) =
        (reference_units, gpu_side_units, adapters)
    else {
        let refusals: Vec<&String> = [
            reference_units.as_ref().err(),
            gpu_side_units.as_ref().err(),
        ]
        .into_iter()
        .flatten()
        .chain(adapters.as_ref().err())
        .collect();
        println!(
            "PRODUCT_TEST_PAIRING_CONCLUSION_KEY_REFUSED inputs={}：判了、不入库：{refusals:?}",
            input_digest.hexadecimal()
        );
        return (judge(), PairingConclusionSource::JudgedNow);
    };
    let key = conclusion_key(input_digest, reference_units, gpu_side_units, adapters);
    if let Some(stored) = library.read_value(key.clone()) {
        let stored = PairingJudged::of_the_stored_bytes(&stored)
            .unwrap_or_else(|reason| panic!("库里的对拍结论认不出：{reason}"));
        match stored.agreement {
            PairingAgreement::BothSidesAgree => {
                return (stored, PairingConclusionSource::ReusedFromTheLibrary)
            }
            PairingAgreement::SidesDisagree => println!(
                "PRODUCT_TEST_PAIRING_STORED_RED inputs={}：库里这一把键记着两边不一致，这一趟现判",
                input_digest.hexadecimal()
            ),
        }
    }
    let judged = judge();
    library.write_values(vec![(key, judged.to_stored_bytes())]);
    (judged, PairingConclusionSource::JudgedNow)
}

// ============================== 持卡判与真卡对拍用例的入口 ==============================

/// 判段读回输入之后、开卡之前调：环境变量没给卡而有领卡通道时，这一刻才领（`common_discrete_gpus_of_this_host` 的
/// `cards_leased_on_demand_unless_handed_over`，进程里只领一回、进程退出时还）。只录那一趟走不到这里，不领卡。
///
/// # Panics
/// 领卡没成（被拒、找不到领卡客户端）：运行条件不满足，不退回「本次未跑」。
fn lease_the_cards_on_demand_before_the_card_step(test_name: &str) {
    cards_leased_on_demand_unless_handed_over().unwrap_or_else(|refusal| {
        panic!("对拍用例「{test_name}」判段要开卡，按需领卡没成：{refusal}")
    });
}

/// 持卡判那一段在这一台的卡上没跑成（这一台没交到卡、或这张卡撑不起这一条，打过 `NOT_RUN_NO_GPU_ADAPTER`）时带出去的记号：
/// [`judgement_of_the_assertions_on_the_cards`] 照它 `panic_any`，[`pairing_concluded_on_the_cards_in_the_library`] 认出它就当这一趟没判、结论不入库。
pub struct PairingNotRunOnTheseCards;

/// 持卡判那一段（开卡、跑 GPU、比）跑下来的样子。
pub enum ComparisonOnTheCards {
    /// 开了卡、跑完、每一处比都过了。
    EveryComparisonHeld,
    /// 这一台没交到卡、或这张卡撑不起这一条（打过 `NOT_RUN_NO_GPU_ADAPTER`）：没比。
    NotRunOnTheseCards,
}

/// panic 带出来的文字（`panic!` 与断言带的是 `String` 或 `&str`）。
fn text_of_the_panic_payload(payload: &(dyn std::any::Any + Send)) -> String {
    payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| {
            payload
                .downcast_ref::<&'static str>()
                .map(|text| (*text).to_string())
        })
        .unwrap_or_else(|| "panic 带的不是文字".to_string())
}

/// 每条用例的持卡判入口（登记在 `tests/pairing_units_entries.tsv` 的 GPU 一侧那几行）都经它交回判定：`run_and_compare_on_the_cards`
/// 照常开卡、跑 GPU、用断言比，断言都过了交回两边一致（结论是一句「每一处比都过了」）；有一处 panic（断言不过、开卡判红这一类）交回两边不一致，
/// 结论是 panic 带的那一句；交回 [`ComparisonOnTheCards::NotRunOnTheseCards`] 的照 [`PairingNotRunOnTheseCards`] `panic_any`（没判，不入库）。
/// 它在每条持卡判入口的闭包里，比法改了 GPU 一侧的键跟着变。
///
/// # Panics
/// 没跑成时照上面带记号 panic。
pub fn judgement_of_the_assertions_on_the_cards(
    test_name: &str,
    run_and_compare_on_the_cards: impl FnOnce() -> ComparisonOnTheCards,
) -> PairingJudged {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(run_and_compare_on_the_cards)) {
        Ok(ComparisonOnTheCards::EveryComparisonHeld) => PairingJudged {
            conclusion: format!("{test_name}：持卡那一段每一处比都过了").into_bytes(),
            agreement: PairingAgreement::BothSidesAgree,
        },
        Ok(ComparisonOnTheCards::NotRunOnTheseCards) => {
            std::panic::panic_any(PairingNotRunOnTheseCards)
        }
        Err(payload) => PairingJudged {
            conclusion: format!("{test_name}：{}", text_of_the_panic_payload(&*payload))
                .into_bytes(),
            agreement: PairingAgreement::SidesDisagree,
        },
    }
}

/// 一条真卡对拍用例过库与结论入库用的三把键：录入键（输入连同参照一侧的判法单元）、参照一侧与 GPU 一侧的判法单元键（拒算的是 `Err`）。
pub struct KeysOfAPairingOnTheCards {
    pub recording: RecordingKeyOfThisRun,
    pub reference_units: Result<PairingSideUnitsKey, String>,
    pub gpu_side_units: Result<PairingSideUnitsKey, String>,
}

/// 一条真卡对拍用例这一趟的结局（判出两边不一致的不交回：结论记红之后 panic）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PairingOnTheCardsOutcome {
    /// 只录那一趟：输入与参照写进库，不领卡、不开卡。
    RecordedOnly,
    /// 库里这一把键下有两边一致的结论：领了卡、读了适配器，不开卡、不判。
    ConclusionReusedFromTheLibrary,
    /// 这一趟领卡判的，两边一致，结论照这一把键存进库（哪一边拒算的不存）。
    JudgedNowAndBothSidesAgree,
    /// 持卡判没跑成（打过 `NOT_RUN_NO_GPU_ADAPTER`）：没判，不入库，下一趟照判。
    NotRunOnTheseCards,
}

/// [`pairing_concluded_on_the_cards_through_the_pairing_library`] 的本体：库、选段与三把键由调用方给（测这份存取本身的用例在临时服务上跑）。
/// 输入与参照照录入键过库（[`pairing_input_and_reference_through_the_library_under_the_recording_key`]）；只录那一趟打一行
/// `RECORDED_ONLY test=<用例名> input_bytes=<字节数>`、到此返回。判段先调 `adapters_of_the_cards` 看交来的卡（这一台没交到卡的交回没跑成、不问库），再拿读回输入的摘要、参照一侧与 GPU 一侧的键、适配器摘要问库里的对拍结论
/// （[`pairing_conclusion_reused_or_judged`]）：两边一致的复用、打一行 `PAIRING_CONCLUSION_REUSED test=<用例名>`，`judge_on_the_cards` 不调；
/// 要现判的才调它（收读回的输入与参照），判出一致打一行 `PAIRING_CONCLUSION_JUDGED test=<用例名> agreement=both_sides_agree`。
///
/// # Panics
/// 库读写不成、读不回；判出两边不一致（结论已照这一把键记红，消息带用例名与结论的文字）。
#[expect(
    clippy::too_many_arguments,
    reason = "八样各自独立、凑不成带不变量的概念：三把键已收在 KeysOfAPairingOnTheCards 里；库与选段、输入与算参照的函数、看卡与持卡判两个回调之间没有构造时查得了的约束，测这份存取的用例要逐样单独换掉，收成结构体只是挪个地方"
)]
pub fn pairing_concluded_on_the_cards_in_the_library(
    library: &PairingLibraryOfTheProductTests,
    stage: RunStage,
    test_name: &str,
    keys: &KeysOfAPairingOnTheCards,
    input_bytes: &[u8],
    reference_of_the_input: impl FnOnce(&[u8]) -> Vec<u8>,
    adapters_of_the_cards: impl FnOnce() -> AdaptersSeenBeforeTheConclusion,
    judge_on_the_cards: impl FnOnce(&[u8], &[u8]) -> PairingJudged,
) -> PairingOnTheCardsOutcome {
    let Some(read_back) = pairing_input_and_reference_through_the_library_under_the_recording_key(
        library,
        stage,
        test_name,
        &keys.recording,
        input_bytes,
        reference_of_the_input,
    ) else {
        println!(
            "RECORDED_ONLY test={test_name} input_bytes={}",
            input_bytes.len()
        );
        return PairingOnTheCardsOutcome::RecordedOnly;
    };
    let adapters = match adapters_of_the_cards() {
        AdaptersSeenBeforeTheConclusion::NoGpuHandedOver => {
            return PairingOnTheCardsOutcome::NotRunOnTheseCards;
        }
        AdaptersSeenBeforeTheConclusion::Seen(adapters) => adapters,
    };
    let concluded = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        pairing_conclusion_reused_or_judged(
            library,
            &read_back.input_digest,
            &keys.reference_units,
            &keys.gpu_side_units,
            &adapters,
            || judge_on_the_cards(&read_back.input, &read_back.reference),
        )
    }));
    let (judged, source) = match concluded {
        Ok(concluded) => concluded,
        Err(payload) if payload.is::<PairingNotRunOnTheseCards>() => {
            return PairingOnTheCardsOutcome::NotRunOnTheseCards;
        }
        Err(payload) => std::panic::resume_unwind(payload),
    };
    match (source, judged.agreement) {
        (
            PairingConclusionSource::ReusedFromTheLibrary,
            PairingAgreement::BothSidesAgree | PairingAgreement::SidesDisagree,
        ) => {
            println!("PAIRING_CONCLUSION_REUSED test={test_name}");
            PairingOnTheCardsOutcome::ConclusionReusedFromTheLibrary
        }
        (PairingConclusionSource::JudgedNow, PairingAgreement::BothSidesAgree) => {
            println!("PAIRING_CONCLUSION_JUDGED test={test_name} agreement=both_sides_agree");
            PairingOnTheCardsOutcome::JudgedNowAndBothSidesAgree
        }
        (PairingConclusionSource::JudgedNow, PairingAgreement::SidesDisagree) => panic!(
            "对拍用例「{test_name}」判出两边不一致（结论已照这一把键记红，下一趟现判）：{}",
            String::from_utf8_lossy(&judged.conclusion)
        ),
    }
}

/// GPU 一侧登记表里这条用例那一组的判法单元键（组名就是用例名）；组算不出（拒算）交回 `Err`，照判、不入库。
///
/// # Panics
/// 登记表里没有这条用例那一组：每条真卡对拍用例都要登记自己的持卡判入口。
fn gpu_side_units_key_of_the_test(test_name: &str) -> Result<PairingSideUnitsKey, String> {
    let group_line_start = format!("group\t{test_name}\t");
    assert!(
        GPU_SIDE_UNITS_KEYS_TEXT
            .lines()
            .any(|line| line.starts_with(&group_line_start)),
        "GPU 一侧登记表（tests/pairing_units_entries.tsv）里没有用例「{test_name}」那一组：每条真卡对拍用例都要登记自己的持卡判入口"
    );
    PairingSideUnitsKey::of_the_group_in_the_table_text(GPU_SIDE_UNITS_KEYS_TEXT, test_name)
}

/// 一边是 CPU 参照、一边要上卡的真卡对拍用例走这一条：`input_bytes_of_the_test` 造的输入连同 `reference_of_the_input` 在 CPU 上算的参照过判定库
/// （录入键拼进登记表里 `reference_units_group` 那一组的判法单元键），判段读回之后对拍结论入库、按键复用（[`pairing_concluded_on_the_cards_in_the_library`]）。
/// 要现判的那一刻才按需领卡，再调 `judged_on_the_cards`（这条用例的持卡判入口：收用例名、读回的输入与参照，开卡、跑 GPU、比，交回判定）；
/// 判段问库之前先按需领卡、读交来的卡的适配器（结论的键要它）：复用的那一趟也领卡、读适配器，只是不开卡；这一台没交到卡的报「本次未跑」。
///
/// # Panics
/// 库读写不成、读不回；GPU 一侧登记表里没有这条用例；判出两边不一致；领卡没成。
pub fn pairing_concluded_on_the_cards_through_the_pairing_library(
    test_name: &str,
    reference_units_group: &str,
    input_bytes_of_the_test: impl FnOnce() -> Vec<u8>,
    reference_of_the_input: impl FnOnce(&[u8]) -> Vec<u8>,
    judged_on_the_cards: impl FnOnce(&str, &[u8], &[u8]) -> PairingJudged,
) {
    let input_bytes = input_bytes_of_the_test();
    let reference_units = PairingSideUnitsKey::of_the_group_in_the_table_text(
        REFERENCE_UNITS_KEYS_TEXT,
        reference_units_group,
    );
    let gpu_side_units = gpu_side_units_key_of_the_test(test_name);
    let library = PairingLibraryOfTheProductTests::open_the_one_library();
    let keys = KeysOfAPairingOnTheCards {
        recording: recording_key_of_the_pairing_input_and_the_reference_units(
            test_name,
            &input_bytes,
            &reference_units,
        ),
        reference_units,
        gpu_side_units,
    };
    pairing_concluded_on_the_cards_in_the_library(
        &library,
        RunStage::of_this_process(),
        test_name,
        &keys,
        &input_bytes,
        reference_of_the_input,
        || {
            lease_the_cards_on_demand_before_the_card_step(test_name);
            adapters_seen_through_the_handed_over_gpus(test_name)
        },
        |input_read_back, reference_read_back| {
            lease_the_cards_on_demand_before_the_card_step(test_name);
            judged_on_the_cards(test_name, input_read_back, reference_read_back)
        },
    );
}

/// 从库里读回的对拍输入与参照一侧在只录那一段算好的结果。
pub struct PairingInputAndReference {
    pub input: Vec<u8>,
    pub reference: Vec<u8>,
}

/// 一边是 CPU 参照、一边要上卡、结论不入库的对拍用例的头一步：`input_bytes_of_the_test` 造的输入过判定库，只录那一段另调 `reference_of_the_input`
/// 在 CPU 上算参照一侧的结果、一起写进库（录入键拼进登记表里 `reference_units_group` 那一组的判法单元键）。只录那一趟打一行 `RECORDED_ONLY`、
/// 交回 `None`（用例到此返回，不开卡）；判段按需领卡，交回从库里读回的输入与参照，持卡的那一段不再算参照。
///
/// # Panics
/// 库读写不成、读不回；领卡没成。
#[must_use]
pub fn pairing_input_and_reference_read_back_through_the_pairing_library(
    test_name: &str,
    reference_units_group: &str,
    input_bytes_of_the_test: impl FnOnce() -> Vec<u8>,
    reference_of_the_input: impl FnOnce(&[u8]) -> Vec<u8>,
) -> Option<PairingInputAndReference> {
    let input_bytes = input_bytes_of_the_test();
    let reference_units = PairingSideUnitsKey::of_the_group_in_the_table_text(
        REFERENCE_UNITS_KEYS_TEXT,
        reference_units_group,
    );
    let library = PairingLibraryOfTheProductTests::open_the_one_library();
    let recording = recording_key_of_the_pairing_input_and_the_reference_units(
        test_name,
        &input_bytes,
        &reference_units,
    );
    let Some(read_back) = pairing_input_and_reference_through_the_library_under_the_recording_key(
        &library,
        RunStage::of_this_process(),
        test_name,
        &recording,
        &input_bytes,
        reference_of_the_input,
    ) else {
        println!(
            "RECORDED_ONLY test={test_name} input_bytes={} reference_units={reference_units_group}",
            input_bytes.len()
        );
        return None;
    };
    lease_the_cards_on_demand_before_the_card_step(test_name);
    Some(PairingInputAndReference {
        input: read_back.input,
        reference: read_back.reference,
    })
}

/// 只有输入、没有参照的对拍用例的头一步：`input_bytes_of_the_test` 造的输入过判定库（录入键只拼场景与输入）。只录那一趟打一行
/// `RECORDED_ONLY test=<用例名> input_bytes=<字节数>`、交回 `None`（用例到此返回，不开卡）；判段按需领卡，交回从库里读回的字节。
///
/// # Panics
/// 库读写不成、读不回；领卡没成。
#[must_use]
pub fn pairing_input_read_back_through_the_pairing_library(
    test_name: &str,
    input_bytes_of_the_test: impl FnOnce() -> Vec<u8>,
) -> Option<Vec<u8>> {
    let input_bytes = input_bytes_of_the_test();
    let library = PairingLibraryOfTheProductTests::open_the_one_library();
    let recording = recording_key_of_the_scenario_and_the_inputs(
        test_name,
        &recording_inputs_digest_of_the_input(&input_bytes),
    );
    let Some(manifest) = manifest_recorded_or_reused_under_the_recording_key(
        &library,
        RunStage::of_this_process(),
        test_name,
        &recording,
        || BytesOfThePairing {
            input: input_bytes.clone(),
            reference: None,
        },
    ) else {
        println!(
            "RECORDED_ONLY test={test_name} input_bytes={}",
            input_bytes.len()
        );
        return None;
    };
    let input_read_back = library.bytes_read_back(&manifest.input, "对拍输入");
    lease_the_cards_on_demand_before_the_card_step(test_name);
    Some(input_read_back)
}

/// 往对拍输入的字节里追加一个数（8 字节小端）。
pub fn append_number(input_bytes: &mut Vec<u8>, number: u64) {
    input_bytes.extend_from_slice(&number.to_le_bytes());
}

/// 往对拍输入的字节里追加一个个数或长度（8 字节小端）。
pub fn append_count(input_bytes: &mut Vec<u8>, count: usize) {
    append_number(
        input_bytes,
        u64::try_from(count).expect("个数与长度装得进 u64"),
    );
}

/// 往对拍输入的字节里追加一段带长度的字节：长度（8 字节小端）接字节本身。
pub fn append_length_prefixed_bytes(input_bytes: &mut Vec<u8>, bytes: &[u8]) {
    append_count(input_bytes, bytes.len());
    input_bytes.extend_from_slice(bytes);
}

/// 从库里读回的对拍输入，从头往后取；取完调 [`PairingInputBeingRead::finish`] 核一个字节都没剩。
pub struct PairingInputBeingRead<'input> {
    remaining: &'input [u8],
}

impl<'input> PairingInputBeingRead<'input> {
    #[must_use]
    pub fn from_the_start_of(input_bytes: &'input [u8]) -> Self {
        Self {
            remaining: input_bytes,
        }
    }

    /// 取 `byte_count` 个字节。
    ///
    /// # Panics
    /// 剩下的不够。
    pub fn take_bytes(&mut self, byte_count: usize) -> &'input [u8] {
        assert!(
            byte_count <= self.remaining.len(),
            "读回的对拍输入不够：要取 {byte_count} 字节，只剩 {} 字节",
            self.remaining.len()
        );
        let (taken, rest) = self.remaining.split_at(byte_count);
        self.remaining = rest;
        taken
    }

    /// 取一个数（[`append_number`] 写下的）。
    pub fn take_number(&mut self) -> u64 {
        u64::from_le_bytes(self.take_bytes(8).try_into().expect("刚取的正好 8 字节"))
    }

    /// 取一个个数或长度（[`append_count`] 写下的）。
    pub fn take_count(&mut self) -> usize {
        usize::try_from(self.take_number()).expect("写下的个数装得进 usize")
    }

    /// 取一段带长度的字节（[`append_length_prefixed_bytes`] 写下的）。
    pub fn take_length_prefixed_bytes(&mut self) -> &'input [u8] {
        let length = self.take_count();
        self.take_bytes(length)
    }

    /// # Panics
    /// 取完还有剩：录的与读的布局对不上。
    pub fn finish(self) {
        assert!(
            self.remaining.is_empty(),
            "读回的对拍输入取完还剩 {} 字节：录的与读的布局对不上",
            self.remaining.len()
        );
    }
}

/// 一个池每块盘写过的扇区编成对拍输入的字节：盘数，逐块盘是盘号、扇区数，逐个扇区是扇区号与带长度的字节。数都是 8 字节小端。
#[must_use]
pub fn pairing_input_bytes_of_the_written_sectors(
    written_sectors: &[(
        singlefs_core::address::DeviceIdentity,
        std::collections::BTreeMap<u64, Vec<u8>>,
    )],
) -> Vec<u8> {
    let mut input_bytes = Vec::new();
    append_count(&mut input_bytes, written_sectors.len());
    for (device, sectors) in written_sectors {
        append_number(&mut input_bytes, u64::from(device.0));
        append_count(&mut input_bytes, sectors.len());
        for (sector, sector_bytes) in sectors {
            append_number(&mut input_bytes, *sector);
            append_length_prefixed_bytes(&mut input_bytes, sector_bytes);
        }
    }
    input_bytes
}

/// [`pairing_input_bytes_of_the_written_sectors`] 的反向。
///
/// # Panics
/// 字节与这份布局对不上（短了、取完还有剩）。
#[must_use]
pub fn written_sectors_of_the_pairing_input_bytes(
    input_bytes: &[u8],
) -> Vec<(
    singlefs_core::address::DeviceIdentity,
    std::collections::BTreeMap<u64, Vec<u8>>,
)> {
    let mut input = PairingInputBeingRead::from_the_start_of(input_bytes);
    let device_count = input.take_count();
    let written_sectors = (0..device_count)
        .map(|_device| {
            let device = singlefs_core::address::DeviceIdentity(
                u32::try_from(input.take_number()).expect("写下的盘号装得进 u32"),
            );
            let sector_count = input.take_count();
            let sectors = (0..sector_count)
                .map(|_sector| {
                    let sector = input.take_number();
                    (sector, input.take_length_prefixed_bytes().to_vec())
                })
                .collect();
            (device, sectors)
        })
        .collect();
    input.finish();
    written_sectors
}
