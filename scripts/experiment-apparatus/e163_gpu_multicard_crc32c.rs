//! E163（GPU多卡算单元校验和）：单机多卡、双机多卡能不能把一批 32768 字节单元的
//! CRC-32C 切开算、合并后与 CPU 参照逐个相同；比对本身会不会红。
//!
// admission: always 每一次跑都是新的适配器枚举、新的显存分配与新的计时，上一次的结论不替这一次作保
// run-condition: command cargo
// task-server-gpu-arguments: anchors run-shard
//!
//! 跑前登记：`research/prompts/e163-preregistration.md`。岔路单：`research/prompts/m2-gpu-multicard-r1-forks.md`。
//!
//! 子命令：`probe`（只枚举、不建设备）、`anchors --adapter-index N`、`gen-input`、`record`、`run-shard`、`merge`。
//! 先录后判（`.claude/rules/verification.md`「判定库只有一份」）：`record`、`run-shard`、`merge` 读写判定库，一律经
//! `scripts/experiment-apparatus/device-code-key.py run` 起，三个入口一起给：
//! `python3 scripts/experiment-apparatus/device-code-key.py run . --binary e163-gpu-multicard-crc32c --feature e163-gpu
//! --entry e7_index_bench::reference_values_of_the_batch --entry e7_index_bench::gpu_values_of_the_shard --entry e7_index_bench::merge_lines_of -- <子命令> …`；
//! 没有 `--code-key-file` 或没有转交的管道退 2。库是 prune-server 那一份（族 apparatus-items，种类是 bin 名），相对键：
//!   `items/reference-<键>.out`  整批的 CPU 参照：`record --seed <十六进制> --units N` 在 CPU 上算全部单元、写进库，不取卡；
//!                               键 = sha256（参照入口的代码键、种子、单元数）
//!   `items/shard-<键>.out`      一片在卡上算出的结果：`run-shard` 先在 CPU 上算锚点、读输入、翻位、算键，库里有就不取卡、照库里的写片结果文件；
//!                               没有才取卡、只跑 GPU、丢掉上下文再做覆盖断言与丢行；键 = sha256（片入口的代码键、起点、单元数、上传副本的 sha256、丢的单元号）
//!   `items/merge-<键>.out`      合并比对的结论（`merge` 打到标准输出的全部行）：键 = sha256（合并入口的代码键、参照那一条的键、批文件头、
//!                               各片记录的 sha256、抽算三单元的 sha256）；有就照打，没有才从库里读回参照（读不回的在 CPU 上现算、写回）再比
//! 卡（`run-shard` 与 `anchors`）只开服务交的：环境里有 `GPU_CARDS_BY_PRIORITY` 照它；没有而有领卡通道（服务交的五个变量）时到开设备前一刻才经
//! `task-server/task lease take` 领，进程退出时还；两样都没有退 78，不自己枚举。
//!
//! 独立手写模型，不与 `crates/` 共用代码（`.claude/rules/implementation-first.md` 第 4 条）：
//! 臂 C（CPU 参照）与臂 G（着色器）各自实现 CRC-32C，互不调用、不共用表。
//! 参数照 D18（块里携带什么信息） 已定项 17：多项式 0x1EDC6F41（反射 0x82F63B78）、
//! 输入输出反射、初值 0xFFFFFFFF、输出取反——两份实现都在本文件里，逐处标注行号供执行员核对。

use e7_index_bench::u32_from_usize;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::time::Instant;

// 判定库的口子（文件头「先录后判」）：包 `singlefs-device-library`（crates/singlefs-device-library），scripts/experiment-apparatus/Cargo.toml 里按 path 依赖引
use singlefs_device_library::DeviceLibrary;

// ============================== D18 已定项 17：CRC-32C 参数（A 类断言的出处） ==============================

/// 反射多项式 0x1EDC6F41 的反射形式；臂 C（下面的 `crc32c_reference`）与臂 G（WGSL 着色器源码
/// 里的同名常量）各自持有一份，不共用，用来在健康输入上互证。
const CASTAGNOLI_REFLECTED_POLYNOMIAL: u32 = 0x82F6_3B78;
/// 初值 0xFFFFFFFF。
const CASTAGNOLI_INITIAL_VALUE: u32 = 0xFFFF_FFFF;

/// 臂 C：按位、反射输入输出、初值 0xFFFFFFFF、输出取反的 CRC-32C 参照实现。
/// 不查表、不与臂 G（WGSL 着色器）共用任何代码或数据。
fn crc32c_reference(bytes: &[u8]) -> u32 {
    let mut register: u32 = CASTAGNOLI_INITIAL_VALUE;
    for &current_byte in bytes {
        register ^= u32::from(current_byte);
        for _ in 0..8 {
            let low_bit_is_set = register & 1 == 1;
            register = if low_bit_is_set {
                (register >> 1) ^ CASTAGNOLI_REFLECTED_POLYNOMIAL
            } else {
                register >> 1
            };
        }
    }
    register ^ CASTAGNOLI_INITIAL_VALUE
}

/// 跑前登记第七节 B 类锚点：八个输入与它们各自的 CRC-32C，命令核过（登记第十三节）。
const EXPECTED_ANCHOR_VALUES: [u32; 8] = [
    0xE306_9283, // ASCII "123456789"
    0x0000_0000, // 空串
    0x8A91_36AA, // 32 × 0x00
    0x62A8_AB43, // 32 × 0xFF
    0x46DD_794E, // 0x00..0x1F 升序
    0x113F_DB5C, // 0x1F..0x00 降序
    0xBC43_BAAD, // 32768 × 0x00
    0x434C_2368, // 32768 × 0xFF
];

const ANCHOR_VECTOR_NAMES: [&str; 8] = [
    "ascii_123456789",
    "empty",
    "thirty_two_zero_bytes",
    "thirty_two_ff_bytes",
    "thirty_two_ascending",
    "thirty_two_descending",
    "unit_length_zero_bytes",
    "unit_length_ff_bytes",
];

/// 跑前登记第七节 B 类的八个锚点向量，次序与 `EXPECTED_ANCHOR_VALUES`、`ANCHOR_VECTOR_NAMES` 一致。
fn anchor_vectors() -> [Vec<u8>; 8] {
    [
        b"123456789".to_vec(),
        Vec::new(),
        vec![0x00u8; 32],
        vec![0xFFu8; 32],
        (0u8..32).collect(),
        (0u8..32).rev().collect(),
        vec![0x00u8; UNIT_BYTE_LENGTH],
        vec![0xFFu8; UNIT_BYTE_LENGTH],
    ]
}

/// 逐个核对臂 C（或任意「按同一份参照实现算出来的」序列）与登记的八个锚点是否一致。
fn anchor_mismatches(computed: &[u32; 8]) -> Vec<usize> {
    (0..8)
        .filter(|&index| computed[index] != EXPECTED_ANCHOR_VALUES[index])
        .collect()
}

// ============================== 批的几何与生成 ==============================

/// 单元大小（登记「读法写死」：对整 32768 字节求 CRC-32C）。
const UNIT_BYTE_LENGTH: usize = 32_768;
/// 每个单元占的 u32 字数：输入缓冲区按 `array<u32>` 传给着色器（登记第五节「臂 G」）。
const WORDS_PER_UNIT_SLOT: usize = UNIT_BYTE_LENGTH / 4;

const SPLITMIX64_INCREMENT: u64 = 0x9E37_79B9_7F4A_7C15;
const SPLITMIX64_FIRST_MULTIPLIER: u64 = 0xBF58_476D_1CE4_E5B9;
const SPLITMIX64_SECOND_MULTIPLIER: u64 = 0x94D0_49BB_1331_11EB;

fn splitmix64_next(state: &mut u64) -> u64 {
    *state = state.wrapping_add(SPLITMIX64_INCREMENT);
    let mut mixed = *state;
    mixed = (mixed ^ (mixed >> 30)).wrapping_mul(SPLITMIX64_FIRST_MULTIPLIER);
    mixed = (mixed ^ (mixed >> 27)).wrapping_mul(SPLITMIX64_SECOND_MULTIPLIER);
    mixed ^ (mixed >> 31)
}

/// 单元 i 的内容 = splitmix64 以 `种子 XOR i·0x9E3779B97F4A7C15` 为初态的输出按小端铺满
/// 32768 字节（登记第五节「批」）。
fn generate_unit_bytes(seed: u64, global_unit_identifier: u64) -> Vec<u8> {
    let mut state = seed ^ global_unit_identifier.wrapping_mul(SPLITMIX64_INCREMENT);
    let mut bytes = Vec::with_capacity(UNIT_BYTE_LENGTH);
    while bytes.len() < UNIT_BYTE_LENGTH {
        bytes.extend_from_slice(&splitmix64_next(&mut state).to_le_bytes());
    }
    bytes.truncate(UNIT_BYTE_LENGTH);
    bytes
}

/// 往一份「上传副本」字节里翻一个位，不touch 磁盘上的输入文件（M3 的 F1 注入，登记第五节「跑 F1、F2」）。
fn apply_bit_flip(bytes: &mut [u8], byte_offset_within_unit: usize, bit_index_within_byte: u32) {
    bytes[byte_offset_within_unit] ^= 1u8 << bit_index_within_byte;
}

// ============================== 输入批文件格式 ==============================

const BATCH_FILE_MAGIC: &[u8; 8] = b"E163IN01";

fn write_batch_file(output_path: &PathBuf, unit_count: u64, seed: u64) -> std::io::Result<()> {
    let mut file = File::create(output_path)?;
    file.write_all(BATCH_FILE_MAGIC)?;
    file.write_all(&unit_count.to_le_bytes())?;
    file.write_all(&(UNIT_BYTE_LENGTH as u64).to_le_bytes())?;
    file.write_all(&seed.to_le_bytes())?;
    for global_unit_identifier in 0..unit_count {
        let unit_bytes = generate_unit_bytes(seed, global_unit_identifier);
        file.write_all(&unit_bytes)?;
    }
    file.flush()
}

struct BatchFileHeader {
    unit_count: u64,
    unit_byte_length: u64,
    seed: u64,
    data_start_offset: u64,
}

fn read_batch_header(input_path: &PathBuf) -> std::io::Result<BatchFileHeader> {
    let mut file = File::open(input_path)?;
    let mut magic = [0u8; 8];
    file.read_exact(&mut magic)?;
    assert_eq!(&magic, BATCH_FILE_MAGIC, "输入文件头魔数不对：{input_path:?}");
    let mut eight_bytes = [0u8; 8];
    file.read_exact(&mut eight_bytes)?;
    let unit_count = u64::from_le_bytes(eight_bytes);
    file.read_exact(&mut eight_bytes)?;
    let unit_byte_length = u64::from_le_bytes(eight_bytes);
    file.read_exact(&mut eight_bytes)?;
    let seed = u64::from_le_bytes(eight_bytes);
    Ok(BatchFileHeader { unit_count, unit_byte_length, seed, data_start_offset: 8 + 8 + 8 + 8 })
}

/// 从批文件里读出 `[start, start+count)` 这一段单元，拼成 `count * UNIT_BYTE_LENGTH` 字节的连续缓冲区。
fn read_unit_range(input_path: &PathBuf, header: &BatchFileHeader, start: u64, count: u64) -> std::io::Result<Vec<u8>> {
    assert_eq!(header.unit_byte_length as usize, UNIT_BYTE_LENGTH, "批文件的单元字节数与本装置写死的不一致");
    assert_eq!(UNIT_BYTE_LENGTH, WORDS_PER_UNIT_SLOT * 4, "字节数与字数的换算与 WGSL 着色器里那份写死的常量必须一致");
    assert!(start + count <= header.unit_count, "读的区间超出批文件的单元数");
    let mut file = File::open(input_path)?;
    file.seek(SeekFrom::Start(header.data_start_offset + start * UNIT_BYTE_LENGTH as u64))?;
    let mut bytes = vec![0u8; (count as usize) * UNIT_BYTE_LENGTH];
    file.read_exact(&mut bytes)?;
    Ok(bytes)
}

// ============================== 「卡」的定义与适配器枚举 ==============================

/// 「卡」= wgpu 在 Vulkan 后端枚举出的、`vendor == 0x10DE` 且 `device_type == DiscreteGpu`
/// 的适配器；软件适配器（llvmpipe 一类）与 CPU 类型不算卡（登记「读法写死」）。
const NVIDIA_VENDOR_IDENTIFIER: u32 = 0x10DE;

fn new_vulkan_instance() -> wgpu::Instance {
    wgpu::Instance::new(wgpu::InstanceDescriptor { backends: wgpu::Backends::VULKAN, ..wgpu::InstanceDescriptor::new_without_display_handle() })
}

fn nvidia_discrete_adapters(instance: &wgpu::Instance) -> Vec<wgpu::Adapter> {
    pollster::block_on(instance.enumerate_adapters(wgpu::Backends::VULKAN))
        .into_iter()
        .filter(|adapter| {
            let adapter_information = adapter.get_info();
            adapter_information.vendor == NVIDIA_VENDOR_IDENTIFIER && adapter_information.device_type == wgpu::DeviceType::DiscreteGpu
        })
        .collect()
}

fn adapter_information_fields(adapter_information: &wgpu::AdapterInfo) -> String {
    format!(
        "name={:?} vendor=0x{:X} device=0x{:X} device_type={:?} driver={:?} driver_info={:?} backend={:?}",
        adapter_information.name,
        adapter_information.vendor,
        adapter_information.device,
        adapter_information.device_type,
        adapter_information.driver,
        adapter_information.driver_info,
        adapter_information.backend
    )
}

/// 片结果文件头里的适配器那几行（`key=value`，一行一样）。
fn adapter_information_lines(adapter_information: &wgpu::AdapterInfo) -> Vec<String> {
    vec![
        format!("name={:?}", adapter_information.name),
        format!("vendor=0x{:X}", adapter_information.vendor),
        format!("device=0x{:X}", adapter_information.device),
        format!("device_type={:?}", adapter_information.device_type),
        format!("driver={:?}", adapter_information.driver),
        format!("driver_info={:?}", adapter_information.driver_info),
        format!("backend={:?}", adapter_information.backend),
    ]
}

// ============================== 服务交来的卡 ==============================
//
// 卡由统一任务服务分（`task-server/task_server.py` 文件头 ②「GPU」「按需领卡」）：起跑就交卡的，卡在环境变量 `GPU_CARDS_BY_PRIORITY`；
// 按需领卡的，环境里只有领卡通道（`CARD_LEASE_CHANNEL_VARIABLES` 那五个），这个进程到开设备前一刻经领卡客户端 `task-server/task lease take` 领，
// 领到的卡串与那个变量同格式，进程退出时还（`cards_leased_by_this_process`；写法照 `crates/singlefs-verification/src/gpu_card_configuration.rs` 的
// `CardLeaseChannelOfTheCaller`，这份装置不依赖 `crates/` 下的包，这里另写一份）。卡串是空白隔开的若干项，每项 `<HOSTS 里的地址>:<nvidia-smi 序号>:<额度>`。认卡的规则同
// `crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs` 的 `gpus_handed_to_this_host`
// （这份装置不依赖 `crates/` 下的包，这里另写一份等价的）：这一台是 `HOSTS` 里哪一项只由多机配置判法
// `scripts/crash/crash-point-replay-shard-configuration-check.sh --emit-addresses` 打的 `THIS_HOST_ADDRESS` 认；
// `nvidia-smi` 序号经它报的总线地址对到 wgpu 的适配器（去掉域、小写之后相同，恰好一个）。

/// 服务交卡的环境变量。
const VARIABLE_OF_THE_CARDS_HANDED_OVER: &str = "GPU_CARDS_BY_PRIORITY";
/// 服务交给按需领卡那一次尝试的领卡通道：通道目录、凭据、尝试标签、任务号、第几次尝试（`task-server/task_server_lib.py` 文件头「GPU」的领卡通道），五个都在、都不空才算有通道。
const CARD_LEASE_CHANNEL_VARIABLES: [&str; 5] =
    ["TASK_SERVER_CARD_LEASE_DIRECTORY", "TASK_SERVER_CARD_LEASE_TOKEN", "TASK_SERVER_ATTEMPT_LABEL", "TASK_SERVER_TASK", "TASK_SERVER_ATTEMPT"];
/// 服务每次尝试交给命令的「提交方那一份仓的根」：领卡客户端取它下面的 `CARD_LEASE_CLIENT_IN_THE_REPOSITORY`。
const SOURCE_REPOSITORY_VARIABLE: &str = "TASK_SERVER_SOURCE_REPOSITORY";
/// 领卡客户端在仓里的路径（从仓根起）。
const CARD_LEASE_CLIENT_IN_THE_REPOSITORY: &str = "task-server/task";
/// 还卡子进程跑的 shell：标准输入读到收尾（领卡的进程退出，写端随之关掉）再起客户端 `lease return <卡串>`（`$0` 是客户端、`$1` 是卡串）。
const RETURN_THE_CARDS_WHEN_THE_STANDARD_INPUT_ENDS: &str = r#"while read -r line_never_written; do :; done; exec "$0" lease return "$1""#;

/// 调用方给这个进程的领卡通道：五个变量的名字与值，原样交给领卡客户端。
#[derive(Debug, PartialEq, Eq)]
struct CardLeaseChannel {
    variables: Vec<(&'static str, String)>,
}

/// 照 `value_of`（按名字取一个环境变量的值）认通道：五个都给了、都不空才是 `Some`。
fn card_lease_channel_of(value_of: &dyn Fn(&str) -> Option<String>) -> Option<CardLeaseChannel> {
    let mut variables = Vec::new();
    for name in CARD_LEASE_CHANNEL_VARIABLES {
        let value = value_of(name).filter(|value| !value.is_empty())?;
        variables.push((name, value));
    }
    Some(CardLeaseChannel { variables })
}

/// 这个进程的卡串从哪来：交卡变量有值（不是空白）照它、不领；没有而有领卡通道，照 `lease` 领（真要卡的这一刻才调）；两样都没有是 `None`。
fn cards_handed_over_or_leased(
    handover_value: Option<&str>,
    channel: Option<&CardLeaseChannel>,
    lease: &dyn Fn(&CardLeaseChannel) -> Result<String, String>,
) -> Result<Option<String>, String> {
    match (handover_value.filter(|value| !value.trim().is_empty()), channel) {
        (Some(handover_text), _) => Ok(Some(handover_text.to_string())),
        (None, Some(channel)) => lease(channel).map(Some),
        (None, None) => Ok(None),
    }
}

/// 领卡客户端在哪：服务交的仓根（`SOURCE_REPOSITORY_VARIABLE` 的值）给了就是它下面那一份；没给的从 `start_directory` 往上逐层找。
fn card_lease_client_from(source_repository: Option<PathBuf>, start_directory: &std::path::Path) -> Result<PathBuf, String> {
    if let Some(repository) = source_repository {
        let client = repository.join(CARD_LEASE_CLIENT_IN_THE_REPOSITORY);
        return if client.is_file() { Ok(client) } else { Err(format!("{SOURCE_REPOSITORY_VARIABLE} 点的仓根下没有领卡客户端：{} 不是文件", client.display())) };
    }
    start_directory
        .ancestors()
        .map(|directory| directory.join(CARD_LEASE_CLIENT_IN_THE_REPOSITORY))
        .find(|client| client.is_file())
        .ok_or_else(|| format!("从 {} 往上找不到领卡客户端 {CARD_LEASE_CLIENT_IN_THE_REPOSITORY}", start_directory.display()))
}

/// 领到的卡串，连同等着这个进程退出就还卡的那个子进程（它的标准输入写端只在这个进程手里，进程退出写端就关）。
#[derive(Debug)]
struct CardsLeasedUntilThisProcessExits {
    cards: String,
    /// 平常只为握着它标准输入的写端（进程退出才关），不读；单测经 `return_the_cards_now` 当场还。
    _return_waiting_for_the_exit: std::process::Child,
}

impl CardsLeasedUntilThisProcessExits {
    /// 不等进程退出，当场还：关掉还卡子进程的标准输入、等它起的客户端退出；交回客户端退没退 0（单测用）。
    #[cfg(test)]
    fn return_the_cards_now(mut self) -> bool {
        drop(self._return_waiting_for_the_exit.stdin.take());
        self._return_waiting_for_the_exit.wait().is_ok_and(|status| status.success())
    }
}

/// 经 `client` 领一回卡：`lease take` 的标准输出是卡串（客户端在等卡时往标准错误打的行照进这个进程的标准错误）；领到之后起还卡的子进程。
/// 客户端退非 0、退 0 而没交卡串、起不了还卡的子进程都交 `Err`（不重试；后两种领到的当场还）。
fn lease_cards_through_the_client(client: &std::path::Path, channel: &CardLeaseChannel) -> Result<CardsLeasedUntilThisProcessExits, String> {
    let variables = || channel.variables.iter().map(|(name, value)| (*name, value.as_str()));
    let taken = std::process::Command::new(client)
        .args(["lease", "take"])
        .envs(variables())
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::inherit())
        .output()
        .map_err(|error| format!("起不了领卡客户端 {}：{error}", client.display()))?;
    if !taken.status.success() {
        return Err(format!("按需领卡没成：{} lease take 退 {:?}（原因在它往标准错误打的那一行）", client.display(), taken.status.code()));
    }
    let cards = String::from_utf8_lossy(&taken.stdout).trim().to_string();
    let return_the_cards_at_once = || {
        // 领到了而往下走不通：当场还，结局不改这一句错（尝试结束时服务照样收回）
        let _returned = std::process::Command::new(client).args(["lease", "return"]).envs(variables()).stdin(std::process::Stdio::null()).status();
    };
    if cards.is_empty() {
        return_the_cards_at_once();
        return Err(format!("按需领卡没成：{} lease take 退 0 而没交卡串", client.display()));
    }
    match std::process::Command::new("sh")
        .arg("-c")
        .arg(RETURN_THE_CARDS_WHEN_THE_STANDARD_INPUT_ENDS)
        .arg(client)
        .arg(&cards)
        .envs(variables())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::inherit())
        .spawn()
    {
        Ok(return_waiting_for_the_exit) => Ok(CardsLeasedUntilThisProcessExits { cards, _return_waiting_for_the_exit: return_waiting_for_the_exit }),
        Err(error) => {
            return_the_cards_at_once();
            Err(format!("按需领卡领到了 {cards}，起不了等这个进程退出就还卡的子进程（sh）：{error}；领到的已当场还掉"))
        }
    }
}

/// 这个进程按需领卡那一回的结局：进程里只领一回，之后每次交同一份；被拒的那一回也记下，不重试。
static CARDS_LEASED_BY_THIS_PROCESS: std::sync::OnceLock<Result<CardsLeasedUntilThisProcessExits, String>> = std::sync::OnceLock::new();

/// 这个进程按需领到的卡串：头一回问的时候才领（客户端在等卡时这里照等），进程退出时还。
fn cards_leased_by_this_process(channel: &CardLeaseChannel) -> Result<&'static str, String> {
    CARDS_LEASED_BY_THIS_PROCESS
        .get_or_init(|| {
            let current_directory = env::current_dir().map_err(|error| format!("取不到当前目录（领卡客户端从这里往上找）：{error}"))?;
            let client = card_lease_client_from(env::var_os(SOURCE_REPOSITORY_VARIABLE).map(PathBuf::from), &current_directory)?;
            let leased = lease_cards_through_the_client(&client, channel)?;
            eprintln!("GPU_CARDS_LEASED_ON_DEMAND cards={}", leased.cards);
            Ok(leased)
        })
        .as_ref()
        .map(|leased| leased.cards.as_str())
        .map_err(Clone::clone)
}

/// 此刻这个进程的卡串：交卡变量、领卡通道都从此刻的环境读（`cards_handed_over_or_leased`）；要领的这一刻领。
fn cards_of_this_process() -> Result<Option<String>, String> {
    let handover_value = env::var(VARIABLE_OF_THE_CARDS_HANDED_OVER).ok();
    let channel = card_lease_channel_of(&|name| env::var(name).ok());
    cards_handed_over_or_leased(handover_value.as_deref(), channel.as_ref(), &|channel| cards_leased_by_this_process(channel).map(str::to_string))
}

/// 交卡变量的一项：`HOSTS` 里哪个地址那一台、`nvidia-smi` 序号（额度不用）。
#[derive(Debug, PartialEq, Eq)]
struct CardHandedOver {
    host_address: String,
    nvidia_smi_index: u32,
}

/// 读交卡变量的值：每项从右边切两刀（地址可以是 IPv6、自带冒号），三段都不许空、序号是数、同一台的同一张卡不许交两次；次序照写的次序。
fn cards_handed_over(handover_text: &str) -> Result<Vec<CardHandedOver>, String> {
    let mut handed_over: Vec<CardHandedOver> = Vec::new();
    for entry in handover_text.split_whitespace() {
        let without_three_fields = || format!("「{entry}」从右边切不出地址、序号、额度三段");
        let (address_and_index, quota) = entry.rsplit_once(':').ok_or_else(without_three_fields)?;
        let (host_address, index_text) = address_and_index.rsplit_once(':').ok_or_else(without_three_fields)?;
        if host_address.is_empty() || index_text.is_empty() || quota.is_empty() {
            return Err(without_three_fields());
        }
        let nvidia_smi_index: u32 = index_text.parse().map_err(|_| format!("「{entry}」的序号不是数"))?;
        let card = CardHandedOver { host_address: host_address.to_string(), nvidia_smi_index };
        if handed_over.contains(&card) {
            return Err(format!("「{entry}」同一台的同一张卡交了两次"));
        }
        handed_over.push(card);
    }
    Ok(handed_over)
}

/// 交给这一台的卡：这一次不跑（服务没交卡、或交的卡里没有这一台的，带一句原因），或这一台那几张的 `nvidia-smi` 序号（照交来的次序，至少一张）。
#[derive(Debug, PartialEq, Eq)]
enum CardsForThisHost {
    NotRun { reason: String },
    Handed(Vec<u32>),
}

/// 照交卡变量的值（没设是 `None`）挑这一台的卡：没设或是空白的不问这一台是谁（`this_host_address` 不调），交回不跑；
/// 交了卡的才调 `this_host_address`，只留地址逐字相同的那几项。值读不成、取不到这一台的地址交回 `Err`（判红）。
fn cards_handed_to_this_host(handover_value: Option<&str>, this_host_address: impl FnOnce() -> Result<String, String>) -> Result<CardsForThisHost, String> {
    let handover_text = match handover_value {
        Some(handover_text) if !handover_text.trim().is_empty() => handover_text,
        Some(_) | None => return Ok(CardsForThisHost::NotRun { reason: format!("服务没经 {VARIABLE_OF_THE_CARDS_HANDED_OVER} 交卡（没设或是空的）") }),
    };
    let handed_over = cards_handed_over(handover_text).map_err(|unreadable| format!("{VARIABLE_OF_THE_CARDS_HANDED_OVER}=「{handover_text}」读不成：{unreadable}"))?;
    let this_host = this_host_address()?;
    let indices_of_this_host: Vec<u32> = handed_over.iter().filter(|card| card.host_address == this_host).map(|card| card.nvidia_smi_index).collect();
    if indices_of_this_host.is_empty() {
        return Ok(CardsForThisHost::NotRun { reason: format!("服务交的卡（{VARIABLE_OF_THE_CARDS_HANDED_OVER}=「{handover_text}」）里没有这一台（HOSTS 里的 {this_host}）的") });
    }
    Ok(CardsForThisHost::Handed(indices_of_this_host))
}

/// bash `printf %q` 打的一个地址还原成原样：只认反斜杠转义与空串的 `''`，别的引号写法认不出就报错。
fn value_printed_by_printf_quoted(quoted: &str) -> Result<String, String> {
    if quoted == "''" {
        return Ok(String::new());
    }
    if quoted.starts_with('\'') || quoted.starts_with("$'") {
        return Err(format!("「{quoted}」的引号写法不是判法给地址打的那一种"));
    }
    let mut value = String::with_capacity(quoted.len());
    let mut characters = quoted.chars();
    while let Some(character) = characters.next() {
        if character == '\\' {
            value.push(characters.next().ok_or_else(|| format!("「{quoted}」结尾多了一个反斜杠"))?);
        } else {
            value.push(character);
        }
    }
    Ok(value)
}

/// 多机配置判法 `--emit-addresses` 打的那几行（`KEY=<printf %q 的值>`）里这一台的地址（`THIS_HOST_ADDRESS` 那一行）。
fn this_host_address_emitted(printed: &str) -> Result<String, String> {
    let quoted = printed.lines().find_map(|line| line.strip_prefix("THIS_HOST_ADDRESS=")).ok_or_else(|| format!("多机配置判法没打 THIS_HOST_ADDRESS 那一行（打的是「{printed}」）"))?;
    let address = value_printed_by_printf_quoted(quoted)?;
    if address.is_empty() {
        return Err("多机配置判法打的 THIS_HOST_ADDRESS 是空的".to_string());
    }
    Ok(address)
}

/// 起多机配置判法 `bash scripts/crash/crash-point-replay-shard-configuration-check.sh --emit-addresses`（判法自己照 `SINGLEFS_MULTI_HOST_CONFIG`、没设照主工作树找多机配置），读这一台的地址。
fn this_host_address_by_the_configuration_check() -> Result<String, String> {
    let script = concat!(env!("CARGO_MANIFEST_DIR"), "/../crash/crash-point-replay-shard-configuration-check.sh");
    let output = std::process::Command::new("bash")
        .arg(script)
        .arg("--emit-addresses")
        .stdin(std::process::Stdio::null())
        .output()
        .map_err(|error| format!("起不了多机配置判法 bash {script} --emit-addresses：{error}（这一台是 HOSTS 里哪一项只由它认）"))?;
    let printed = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        return Err(format!("多机配置判法认不出这一台（bash {script} --emit-addresses 退 {:?}）：{}{}", output.status.code(), printed.trim(), String::from_utf8_lossy(&output.stderr).trim()));
    }
    this_host_address_emitted(&printed)
}

/// PCI 总线地址只留 `总线:设备.功能` 那一段、小写：`nvidia-smi` 报的 `00000000:01:00.0` 与 wgpu 报的 `0000:01:00.0` 认成同一张卡。
fn pci_bus_identifier_without_the_domain(pci_bus_address: &str) -> String {
    let lowered = pci_bus_address.trim().to_ascii_lowercase();
    let parts: Vec<&str> = lowered.split(':').collect();
    parts[parts.len().saturating_sub(2)..].join(":")
}

/// 一份 `nvidia-smi --query-gpu=index,pci.bus_id --format=csv,noheader` 的输出读成「序号 → 去掉域的总线地址」；一行切不出两段、序号不是数、同一个序号报两次交回 `Err`。
fn pci_buses_by_nvidia_smi_index(report: &str) -> Result<std::collections::BTreeMap<u32, String>, String> {
    let mut buses = std::collections::BTreeMap::new();
    for line in report.lines().filter(|line| !line.trim().is_empty()) {
        let (index_text, pci_bus_address) = line.split_once(',').ok_or_else(|| format!("nvidia-smi 的一行「{line}」切不出序号与总线地址"))?;
        let nvidia_smi_index: u32 = index_text.trim().parse().map_err(|_| format!("nvidia-smi 的一行「{line}」序号不是数"))?;
        if buses.insert(nvidia_smi_index, pci_bus_identifier_without_the_domain(pci_bus_address)).is_some() {
            return Err(format!("nvidia-smi 把序号 {nvidia_smi_index} 报了两次"));
        }
    }
    Ok(buses)
}

/// 跑一次 `nvidia-smi --query-gpu=index,pci.bus_id --format=csv,noheader` 读每张卡的序号与总线地址。
fn pci_buses_reported_by_nvidia_smi() -> Result<std::collections::BTreeMap<u32, String>, String> {
    let output = std::process::Command::new("nvidia-smi")
        .args(["--query-gpu=index,pci.bus_id", "--format=csv,noheader"])
        .stdin(std::process::Stdio::null())
        .output()
        .map_err(|error| format!("nvidia-smi 起不来：{error}"))?;
    if !output.status.success() {
        return Err(format!("nvidia-smi 退了 {}：{}", output.status, String::from_utf8_lossy(&output.stderr).trim()));
    }
    pci_buses_by_nvidia_smi_index(&String::from_utf8_lossy(&output.stdout))
}

/// 交来的一张卡（`nvidia-smi` 序号）是本机 NVIDIA 独显里的第几个：序号经 `nvidia-smi` 报的总线地址、去掉域之后与适配器报的（`adapter_pci_buses`，wgpu 报的原样，
/// 次序同 `nvidia_discrete_adapters`）相同，恰好一个；`nvidia-smi` 没报这个序号、那条总线上没有独显或有几个，交回 `Err`（判红，不退回别的卡）。
fn position_of_the_handed_over_card(nvidia_smi_index: u32, pci_buses_by_index: &std::collections::BTreeMap<u32, String>, adapter_pci_buses: &[String]) -> Result<usize, String> {
    let pci_bus = pci_buses_by_index.get(&nvidia_smi_index).ok_or_else(|| format!("交来的卡 nvidia-smi 序号 {nvidia_smi_index}，nvidia-smi 没报这个序号（报的是 {pci_buses_by_index:?}）"))?;
    let positions: Vec<usize> =
        adapter_pci_buses.iter().enumerate().filter(|(_, adapter_pci_bus)| pci_bus_identifier_without_the_domain(adapter_pci_bus) == *pci_bus).map(|(position, _)| position).collect();
    match positions.as_slice() {
        [position] => Ok(*position),
        [] => Err(format!("交来的卡 nvidia-smi 序号 {nvidia_smi_index}（总线 {pci_bus}）对不上本机任何一张 NVIDIA 独显（本机独显的总线 {adapter_pci_buses:?}）")),
        [_, _, ..] => Err(format!("交来的卡 nvidia-smi 序号 {nvidia_smi_index}（总线 {pci_bus}）对上了本机 {} 个独显适配器，点哪一个不定", positions.len())),
    }
}

/// 交给这一台的卡里照交来的次序第 `position` 张，对上的本机 NVIDIA 独显适配器；交来的不够、对不上交回 `Err`（判红）。
fn handed_over_adapter(instance: &wgpu::Instance, cards_of_this_host: &[u32], position: usize) -> Result<wgpu::Adapter, String> {
    let nvidia_smi_index = *cards_of_this_host
        .get(position)
        .ok_or_else(|| format!("服务交给这一台 {} 张卡（nvidia-smi 序号 {cards_of_this_host:?}），要的是交来的第 {position} 张（从 0 数）", cards_of_this_host.len()))?;
    let pci_buses_by_index = pci_buses_reported_by_nvidia_smi()?;
    let adapters = nvidia_discrete_adapters(instance);
    let adapter_pci_buses: Vec<String> = adapters.iter().map(|adapter| adapter.get_info().device_pci_bus_id).collect();
    let adapter_position = position_of_the_handed_over_card(nvidia_smi_index, &pci_buses_by_index, &adapter_pci_buses)?;
    Ok(adapters.into_iter().nth(adapter_position).expect("位置取自同一份枚举"))
}

/// 没拿到服务的卡：打 ✗ 与出路，退 78（运行条件不满足），不自己枚举、不退回别的卡。
fn refuse_without_the_cards_of_the_service(reason: &str) -> ! {
    eprintln!("  ✗ e163：只开服务交的卡，这一次拿不到：{reason}");
    eprintln!("     → 怎么办：整条经 task-server/task submit 交（服务起跑交卡串，或交领卡通道、到开设备前一刻再领）");
    std::process::exit(78)
}

/// `anchors` 与 `run-shard` 的 `--adapter-index N` 点的卡：只开服务交的卡（`cards_of_this_process`：环境里有卡串照它，没有而有领卡通道就在这一刻领），
/// 不论起跑交卡串还是按需领卡，都取服务交给这一个进程的那一张（`handed_over_adapter` 的第 0 张；装置不按 N 自己挑卡）；两样都没有、领卡没成、卡串里没有这一台的、交来的不够、对不上本机独显都退 78，不退回枚举序。
fn adapter_for_index(instance: &wgpu::Instance, adapter_index: usize) -> wgpu::Adapter {
    // 装置不自己挑卡（哪张卡由统一任务服务分）：起跑交的卡串与按需领到的卡，都用服务交给这一个进程的那一张（交来的第 0 张）；--adapter-index 只当片号，不选卡
    let _shard_number_not_a_card_choice = adapter_index;
    let adapter_index = 0;
    let cards = match cards_of_this_process() {
        Ok(Some(cards)) => cards,
        Ok(None) => refuse_without_the_cards_of_the_service(&format!("环境里没有 {VARIABLE_OF_THE_CARDS_HANDED_OVER}，也没有领卡通道（{}）", CARD_LEASE_CHANNEL_VARIABLES.join("、"))),
        Err(reason) => refuse_without_the_cards_of_the_service(&reason),
    };
    match cards_handed_to_this_host(Some(cards.as_str()), this_host_address_by_the_configuration_check) {
        Ok(CardsForThisHost::Handed(cards_of_this_host)) => handed_over_adapter(instance, &cards_of_this_host, adapter_index).unwrap_or_else(|reason| refuse_without_the_cards_of_the_service(&reason)),
        Ok(CardsForThisHost::NotRun { reason }) | Err(reason) => refuse_without_the_cards_of_the_service(&reason),
    }
}

// ============================== 臂 G：GPU 上下文与派发 ==============================

/// WGSL 计算着色器：每个调用算一个单元，字节从 `array<u32>` 里按
/// `(word >> 8*(k%4)) & 0xFF` 取（登记第五节「臂 G」），CRC-32C 参数与臂 C 逐一对得上：
/// 反射多项式 0x82F63B78、初值 0xFFFFFFFF、输出取反——这四样与本文件顶部 `crc32c_reference`
/// 用的是同一组常量值，两处各自持有一份（A 类断言，执行员贴这两处源码行核对）。
const COMPUTE_SHADER_SOURCE: &str = r#"
struct Params {
    unit_count: u32,
    reserved0: u32,
    reserved1: u32,
    reserved2: u32,
};

struct ResultEntry {
    crc: u32,
    local_offset: u32,
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> unit_words: array<u32>;
@group(0) @binding(2) var<storage, read> unit_lengths: array<u32>;
@group(0) @binding(3) var<storage, read_write> results: array<ResultEntry>;

const CRC32C_REFLECTED_POLYNOMIAL: u32 = 0x82F63B78u;
const CRC32C_INITIAL_VALUE: u32 = 0xFFFFFFFFu;
const WORDS_PER_UNIT_SLOT: u32 = 8192u; // 32768 字节 / 4

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let local_offset = global_id.x;
    if (local_offset >= params.unit_count) {
        return;
    }
    let length_in_bytes = unit_lengths[local_offset];
    let base_word = local_offset * WORDS_PER_UNIT_SLOT;
    var crc_register: u32 = CRC32C_INITIAL_VALUE;
    var byte_index: u32 = 0u;
    loop {
        if (byte_index >= length_in_bytes) {
            break;
        }
        let word_index = base_word + byte_index / 4u;
        let word = unit_words[word_index];
        let shift = 8u * (byte_index % 4u);
        let current_byte = (word >> shift) & 0xFFu;
        crc_register = crc_register ^ current_byte;
        var bit_index: u32 = 0u;
        loop {
            if (bit_index >= 8u) {
                break;
            }
            let low_bit_is_set = (crc_register & 1u) == 1u;
            if (low_bit_is_set) {
                crc_register = (crc_register >> 1u) ^ CRC32C_REFLECTED_POLYNOMIAL;
            } else {
                crc_register = crc_register >> 1u;
            }
            bit_index = bit_index + 1u;
        }
        byte_index = byte_index + 1u;
    }
    results[local_offset].crc = crc_register ^ CRC32C_INITIAL_VALUE;
    results[local_offset].local_offset = local_offset;
}
"#;

const RESULT_ENTRY_BYTE_LENGTH: usize = 8; // crc: u32 + local_offset: u32
const WORKGROUP_SIZE: u32 = 64;
const SENTINEL_WORD: u32 = 0xFFFF_FFFF;

struct GpuContext {
    adapter_information: wgpu::AdapterInfo,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
}

/// 起得来上下文＝`request_device` 成功且第一块缓冲区分配成功（登记「读法写死」）。
/// 失败时把 wgpu 原样错误报回去，不换配置重试（登记第五节「对面那条臂」）。
fn build_gpu_context(adapter: wgpu::Adapter) -> Result<GpuContext, String> {
    let adapter_information = adapter.get_info();
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("e163-gpu-multicard-crc32c"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
    }))
    .map_err(|error| format!("request_device 失败：{error:?}"))?;

    // 「第一块缓冲区分配成功」：起一个很小的 storage 缓冲区探一次，探不到就是这张卡起不来上下文。
    let probe_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("e163-probe-buffer"),
        size: 4,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });
    drop(probe_buffer);

    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("e163-crc32c-shader"),
        source: wgpu::ShaderSource::Wgsl(COMPUTE_SHADER_SOURCE.into()),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("e163-crc32c-pipeline"),
        layout: None,
        module: &shader_module,
        entry_point: Some("main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });
    Ok(GpuContext { adapter_information, device, queue, pipeline })
}

fn u32_slice_to_le_bytes(values: &[u32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(values.len() * 4);
    for value in values {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes
}

/// 派发一批单元给 GPU 算 CRC-32C。`unit_slots` 是 `count * UNIT_BYTE_LENGTH` 字节的连续缓冲区
/// （每个单元占一个满槽，不足的用 `lengths[i]` 截断，供 `anchors` 短输入复用同一条路径）；
/// 返回长度为 `count` 的 `(crc, local_offset)`，`local_offset` 取自着色器里的
/// `global_invocation_id.x`（登记第五节「臂 G」）。
///
/// 派发前结果缓冲区整片填哨兵 `0xFFFFFFFF`（登记同节），调用方用 `assert_full_coverage_no_sentinel`
/// 核「没有哨兵残留、`local_offset` 等于数组下标」这两条钉死的断言（登记第七节「另三条」）。
fn run_gpu_crc(context: &GpuContext, unit_slots: &[u8], lengths: &[u32]) -> Vec<(u32, u32)> {
    let count = lengths.len();
    assert_eq!(unit_slots.len(), count * UNIT_BYTE_LENGTH, "输入槽长度与单元数不匹配");
    let device = &context.device;
    let queue = &context.queue;

    let dispatch_parameters = [count as u32, 0u32, 0u32, 0u32];
    let parameter_buffer = wgpu::util::DeviceExt::create_buffer_init(
        device,
        &wgpu::util::BufferInitDescriptor {
            label: Some("e163-params"),
            contents: &u32_slice_to_le_bytes(&dispatch_parameters),
            usage: wgpu::BufferUsages::UNIFORM,
        },
    );
    let input_buffer = wgpu::util::DeviceExt::create_buffer_init(
        device,
        &wgpu::util::BufferInitDescriptor { label: Some("e163-input"), contents: unit_slots, usage: wgpu::BufferUsages::STORAGE },
    );
    let lengths_buffer = wgpu::util::DeviceExt::create_buffer_init(
        device,
        &wgpu::util::BufferInitDescriptor { label: Some("e163-lengths"), contents: &u32_slice_to_le_bytes(lengths), usage: wgpu::BufferUsages::STORAGE },
    );
    let sentinel_bytes = vec![0xFFu8; count * RESULT_ENTRY_BYTE_LENGTH];
    let result_buffer = wgpu::util::DeviceExt::create_buffer_init(
        device,
        &wgpu::util::BufferInitDescriptor {
            label: Some("e163-results"),
            contents: &sentinel_bytes,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        },
    );
    let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("e163-staging"),
        size: (count * RESULT_ENTRY_BYTE_LENGTH) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let bind_group_layout = context.pipeline.get_bind_group_layout(0);
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("e163-bind-group"),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: parameter_buffer.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: input_buffer.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 2, resource: lengths_buffer.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 3, resource: result_buffer.as_entire_binding() },
        ],
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("e163-encoder") });
    {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("e163-pass"), timestamp_writes: None });
        compute_pass.set_pipeline(&context.pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);
        let workgroup_count = count.div_ceil(WORKGROUP_SIZE as usize) as u32;
        compute_pass.dispatch_workgroups(workgroup_count, 1, 1);
    }
    encoder.copy_buffer_to_buffer(&result_buffer, 0, &staging_buffer, 0, (count * RESULT_ENTRY_BYTE_LENGTH) as u64);
    queue.submit(std::iter::once(encoder.finish()));

    let slice = staging_buffer.slice(..);
    let (sender, receiver) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        sender.send(result).expect("回读结果的 map_async 回调发送失败");
    });
    device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None }).expect("device.poll 失败");
    receiver.recv().expect("map_async 回调没有送达").expect("map_async 报告失败");

    let mapped = slice.get_mapped_range().expect("读回结果缓冲区失败");
    let mut results = Vec::with_capacity(count);
    for chunk in mapped.chunks_exact(RESULT_ENTRY_BYTE_LENGTH) {
        let crc = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        let local_offset = u32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]);
        results.push((crc, local_offset));
    }
    drop(mapped);
    staging_buffer.unmap();
    results
}

/// 登记第七节「另三条」：`gid` = 片内偏移（没有重排）、没有哨兵残留。
/// 违反就是作废 2（装置或这一次派发本身错了），panic 让调用方（run-shard / 单测）都能看见。
fn assert_full_coverage_no_sentinel(results: &[(u32, u32)]) {
    for (index, &(crc, local_offset)) in results.iter().enumerate() {
        assert_eq!(local_offset as usize, index, "第 {index} 条结果的 local_offset 不等于数组下标，gid 断言不过");
        assert!(!(crc == SENTINEL_WORD && local_offset == SENTINEL_WORD), "第 {index} 条结果仍是哨兵值，覆盖数不足");
    }
}

/// 一片的判段（片那一条的代码键入口，文件头「先录后判」）：卡上只跑 GPU——建上下文、八锚点与整片各派一次、回读，上下文在那一块末尾丢掉；
/// 之后锚点核对、覆盖 / gid 断言（作废 2，不过就 panic、不进库）与按 `drops` 丢行都在 CPU 上做。`inputs` 在取卡之前已在 CPU 上备好。
fn gpu_values_of_the_shard(adapter: wgpu::Adapter, inputs: &ShardInputs, start: u64, drops: &[u64]) -> Result<ShardOnTheCard, String> {
    let (anchor_gpu_results, gpu_results, adapter_lines, build_device_milliseconds, transfer_compute_readback_milliseconds) = {
        let build_device_start = Instant::now();
        let context = build_gpu_context(adapter)?;
        let build_device_milliseconds = build_device_start.elapsed().as_secs_f64() * 1000.0;
        let anchor_gpu_results = run_gpu_crc(&context, &inputs.anchor_slots, &inputs.anchor_lengths);
        let transfer_compute_readback_start = Instant::now();
        let gpu_results = run_gpu_crc(&context, &inputs.unit_slots, &inputs.lengths);
        let transfer_compute_readback_milliseconds = transfer_compute_readback_start.elapsed().as_secs_f64() * 1000.0;
        (anchor_gpu_results, gpu_results, adapter_information_lines(&context.adapter_information), build_device_milliseconds, transfer_compute_readback_milliseconds)
    };
    assert_full_coverage_no_sentinel(&anchor_gpu_results);
    let gpu_anchor_values: [u32; 8] = std::array::from_fn(|index| anchor_gpu_results[index].0);
    let anchors_gpu_match = anchor_mismatches(&gpu_anchor_values).is_empty();

    let mut sentinel_and_gid_ok = true;
    for (index, &(crc, local_offset)) in gpu_results.iter().enumerate() {
        if local_offset as usize != index || (crc == SENTINEL_WORD && local_offset == SENTINEL_WORD) {
            sentinel_and_gid_ok = false;
        }
    }
    assert!(sentinel_and_gid_ok, "这一片的覆盖 / gid 断言不过（作废 2）");

    let records: Vec<ShardResultRecord> = gpu_results
        .iter()
        .enumerate()
        .map(|(local_offset, &(crc, _))| ShardResultRecord { global_unit_identifier: start + local_offset as u64, crc })
        .filter(|record| !drops.contains(&record.global_unit_identifier))
        .collect();
    Ok(ShardOnTheCard { anchors_gpu_match, sentinel_and_gid_ok, records, adapter_lines, build_device_milliseconds, transfer_compute_readback_milliseconds })
}

// ============================== 一片（shard）跑出来的结果文件 ==============================

/// 一条注入的翻位：全局单元号、单元内字节偏移、字节内位号（登记第五节「跑 F1、F2」）。
#[derive(Clone, Copy)]
struct BitFlipInjection {
    global_unit_identifier: u64,
    byte_offset_within_unit: usize,
    bit_index_within_byte: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ShardResultRecord {
    global_unit_identifier: u64,
    crc: u32,
}

/// 一片的结果文件：文本格式，头部若干 `key=value` 行，一个 `# results` 分隔行，随后每行
/// 一条 `<global_unit_identifier> <crc_hex>`（登记第五节「计时」「传输」，第九节变异 V9 的截断检测在
/// merge 阶段按行数与 sha256 一起判）。
fn write_shard_result_file(
    output_path: &PathBuf,
    library_item: &str,
    reused_from_the_library: bool,
    adapter_lines: &[String],
    shard_start: u64,
    shard_count: u64,
    build_device_milliseconds: f64,
    transfer_compute_readback_milliseconds: f64,
    total_milliseconds: f64,
    injected_flips: &[BitFlipInjection],
    injected_drops: &[u64],
    sentinel_and_gid_ok: bool,
    anchors_cpu_match: bool,
    anchors_gpu_match: bool,
    records: &[ShardResultRecord],
) -> std::io::Result<()> {
    let mut file = File::create(output_path)?;
    writeln!(file, "library_item={library_item}")?;
    writeln!(file, "reused_from_the_library={reused_from_the_library}")?;
    for adapter_line in adapter_lines {
        writeln!(file, "{adapter_line}")?;
    }
    writeln!(file, "start={shard_start}")?;
    writeln!(file, "count={shard_count}")?;
    writeln!(file, "build_device_ms={build_device_milliseconds:.3}")?;
    writeln!(file, "transfer_compute_readback_ms={transfer_compute_readback_milliseconds:.3}")?;
    writeln!(file, "total_ms={total_milliseconds:.3}")?;
    if injected_flips.is_empty() {
        writeln!(file, "injected_flip=none")?;
    } else {
        let flips: Vec<String> = injected_flips
            .iter()
            .map(|flip| format!("{}:{}:{}", flip.global_unit_identifier, flip.byte_offset_within_unit, flip.bit_index_within_byte))
            .collect();
        writeln!(file, "injected_flip={}", flips.join(","))?;
    }
    if injected_drops.is_empty() {
        writeln!(file, "injected_drop=none")?;
    } else {
        let drops: Vec<String> = injected_drops.iter().map(u64::to_string).collect();
        writeln!(file, "injected_drop={}", drops.join(","))?;
    }
    writeln!(file, "sentinel_and_gid_ok={sentinel_and_gid_ok}")?;
    writeln!(file, "anchors_cpu_match={anchors_cpu_match}")?;
    writeln!(file, "anchors_gpu_match={anchors_gpu_match}")?;
    writeln!(file, "record_count={}", records.len())?;
    writeln!(file, "# results")?;
    for record in records {
        writeln!(file, "{} {:08X}", record.global_unit_identifier, record.crc)?;
    }
    file.flush()
}

fn read_shard_result_file(path: &PathBuf) -> std::io::Result<Vec<ShardResultRecord>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    let mut past_header = false;
    for line in reader.lines() {
        let line = line?;
        if !past_header {
            if line == "# results" {
                past_header = true;
            }
            continue;
        }
        let mut parts = line.split_whitespace();
        let global_unit_identifier: u64 = parts.next().expect("结果行缺全局单元号").parse().expect("全局单元号不是数字");
        let crc_hex = parts.next().expect("结果行缺 CRC 十六进制值");
        let crc = u32::from_str_radix(crc_hex, 16).expect("CRC 不是合法的十六进制");
        records.push(ShardResultRecord { global_unit_identifier, crc });
    }
    Ok(records)
}

// ============================== 合并比对（arm C 对 arm G，登记第六节 Q2/Q5/Q8/Q9） ==============================

#[derive(Default, Debug)]
struct MergeReport {
    matched_count: usize,
    mismatched_global_unit_identifiers: Vec<u64>,
    missing_global_unit_identifiers: Vec<u64>,
    duplicated_global_unit_identifiers: Vec<u64>,
}

/// 纯函数，不碰 GPU、不碰文件：拿臂 C 算好的参照值（下标即全局单元号）与几片臂 G 的结果，
/// 逐个全局单元号比对。这是 V1（合并比对恒判相同）与 V2（不按 0..N 查缺）两条变异要抓的函数。
fn compare_reference_and_shards(cpu_reference: &[u32], shard_records: &[&[ShardResultRecord]]) -> MergeReport {
    let total_units = cpu_reference.len() as u64;
    let mut seen_crc_by_unit: std::collections::HashMap<u64, Vec<u32>> = std::collections::HashMap::new();
    for shard in shard_records {
        for record in *shard {
            seen_crc_by_unit.entry(record.global_unit_identifier).or_default().push(record.crc);
        }
    }
    let mut report = MergeReport::default();
    for global_unit_identifier in 0..total_units {
        match seen_crc_by_unit.get(&global_unit_identifier) {
            None => report.missing_global_unit_identifiers.push(global_unit_identifier),
            Some(values) if values.len() > 1 => report.duplicated_global_unit_identifiers.push(global_unit_identifier),
            Some(values) => {
                if values[0] == cpu_reference[global_unit_identifier as usize] {
                    report.matched_count += 1;
                } else {
                    report.mismatched_global_unit_identifiers.push(global_unit_identifier);
                }
            }
        }
    }
    report
}

/// 臂 C：整批 `units` 个单元照 `generate_unit_bytes` 现造（不读批文件），逐个求 `crc32c_reference`，下标即全局单元号。
/// 参照那一条的代码键入口（文件头「先录后判」）：生成器或臂 C 一改，参照的键跟着变。
fn reference_values_of_the_batch(seed: u64, units: u64) -> Vec<u32> {
    (0..units).map(|global_unit_identifier| crc32c_reference(&generate_unit_bytes(seed, global_unit_identifier))).collect()
}

/// 抽算的三个单元（登记第七节「另三条」：N = 2560 时的首、中、末）。
fn sampled_unit_indices(total_units: u64) -> [u64; 3] {
    let middle_sample_index = if total_units > 1279 { 1279 } else { total_units / 2 };
    [0, middle_sample_index, total_units - 1]
}

/// `merge` 打到标准输出的全部行：参照两两不同（作废 2，真碰撞概率约 7.6e-4，登记第七节「另三条」）、逐全局单元号比对、抽算三单元与参照同值。
/// 合并那一条的代码键入口（文件头「先录后判」）；`sampled_units` 是 `sampled_unit_indices` 那三个单元号与从批文件读出的原字节。
fn merge_lines_of(
    header_seed: u64,
    header_unit_count: u64,
    total_units: u64,
    cpu_reference: &[u32],
    shard_records: &[&[ShardResultRecord]],
    sampled_units: &[(u64, Vec<u8>)],
) -> Vec<String> {
    let mut sorted_reference = cpu_reference.to_vec();
    sorted_reference.sort_unstable();
    let distinct_count = sorted_reference.windows(2).filter(|pair| pair[0] != pair[1]).count() + usize::from(!sorted_reference.is_empty());
    let all_distinct = distinct_count == cpu_reference.len();
    let report = compare_reference_and_shards(cpu_reference, shard_records);
    let mut lines = vec![
        format!("MERGE_INPUT_HEADER seed=0x{header_seed:016X} unit_count={header_unit_count}"),
        format!("MERGE total_units={total_units} shards={}", shard_records.len()),
        format!("MERGE matched_count={}", report.matched_count),
        format!("MERGE mismatched_count={} mismatched_global_unit_identifiers={:?}", report.mismatched_global_unit_identifiers.len(), report.mismatched_global_unit_identifiers),
        format!("MERGE missing_count={} missing_global_unit_identifiers={:?}", report.missing_global_unit_identifiers.len(), report.missing_global_unit_identifiers),
        format!("MERGE duplicated_count={} duplicated_global_unit_identifiers={:?}", report.duplicated_global_unit_identifiers.len(), report.duplicated_global_unit_identifiers),
        format!("MERGE all_reference_values_distinct={all_distinct}"),
    ];
    for (sample_index, recomputed_bytes) in sampled_units {
        let recomputed = crc32c_reference(recomputed_bytes);
        let reference = cpu_reference[*sample_index as usize];
        lines.push(format!("MERGE sample_unit={sample_index} recomputed={recomputed:08X} cpu_reference={reference:08X} equal={}", recomputed == reference));
    }
    lines
}

// ============================== 判定库：参照、片、合并三种项（文件头「先录后判」） ==============================

/// `device-code-key.py` 打的组成清单的头一行。
const COMPOSITION_FORM_LINE: &str = "form\tdevice-code-key-v1";
/// 组成清单里三把代码键各自的入口（以这一段收尾）：参照、片、合并。
const REFERENCE_ENTRY_SUFFIX: &str = "::reference_values_of_the_batch";
const SHARD_ENTRY_SUFFIX: &str = "::gpu_values_of_the_shard";
const MERGE_ENTRY_SUFFIX: &str = "::merge_lines_of";

/// 这一趟的三把代码键（`device-code-key.py run` 交的组成清单里读）。
#[derive(Debug, PartialEq, Eq)]
struct CodeKeys {
    reference: String,
    shard: String,
    merge: String,
}

/// 读组成清单：头一行是 form，恰好三行 code_key，入口各以 `REFERENCE_ENTRY_SUFFIX`、`SHARD_ENTRY_SUFFIX`、`MERGE_ENTRY_SUFFIX` 收尾、各恰好一行，键是十六进制。
fn code_keys_from_composition(composition: &str) -> Result<CodeKeys, String> {
    if composition.lines().next() != Some(COMPOSITION_FORM_LINE) {
        return Err("--code-key-file 不是 device-code-key.py 打的组成清单（头一行不是 form\tdevice-code-key-v1）".to_string());
    }
    let code_keys: Vec<(&str, &str)> = composition.lines().filter_map(|line| line.strip_prefix("code_key\t")).filter_map(|rest| rest.split_once('\t')).collect();
    if code_keys.len() != 3 {
        return Err(format!("组成清单里要恰好三行 code_key（参照、片、合并三个入口），读到 {} 行", code_keys.len()));
    }
    let key_of = |suffix: &str| -> Result<String, String> {
        let found: Vec<&str> = code_keys.iter().filter(|(entry, _)| entry.ends_with(suffix)).map(|(_, code_key)| *code_key).collect();
        let [code_key] = found.as_slice() else {
            return Err(format!("组成清单里以 {suffix} 收尾的 code_key 要恰好一行，读到 {} 行", found.len()));
        };
        if code_key.is_empty() || !code_key.chars().all(|character| character.is_ascii_hexdigit()) {
            return Err(format!("组成清单里的代码键不是十六进制：{code_key}"));
        }
        Ok((*code_key).to_string())
    };
    Ok(CodeKeys { reference: key_of(REFERENCE_ENTRY_SUFFIX)?, shard: key_of(SHARD_ENTRY_SUFFIX)?, merge: key_of(MERGE_ENTRY_SUFFIX)? })
}

/// 读写判定库的子命令缺了库的口子：打 ✗ 与出路，退 2（用法错）。
fn refuse_without_the_library(subcommand: &str, problem: &str) -> ! {
    eprintln!("  ✗ e163 {subcommand}：{problem}");
    eprintln!("     → 怎么办：经 scripts/experiment-apparatus/device-code-key.py run 起，三个入口一起给（文件头「先录后判」那一条命令）");
    std::process::exit(2)
}

/// 读写判定库的子命令开头调：读 `--code-key-file` 的组成清单、接上 `device-code-key.py run` 开的转交管道；缺一样、读不成退 2（不起卡、不现算）。
fn library_and_code_keys(arguments: &[String], subcommand: &str) -> (DeviceLibrary, CodeKeys) {
    let Some(code_key_path) = parse_named_argument(arguments, "--code-key-file") else {
        refuse_without_the_library(subcommand, "要 --code-key-file（参照、片、合并三个入口的代码键）")
    };
    let composition = std::fs::read_to_string(&code_key_path).unwrap_or_else(|error| refuse_without_the_library(subcommand, &format!("读不了 --code-key-file {code_key_path}：{error}")));
    let code_keys = code_keys_from_composition(&composition).unwrap_or_else(|problem| refuse_without_the_library(subcommand, &problem));
    let library = DeviceLibrary::from_environment().unwrap_or_else(|problem| refuse_without_the_library(subcommand, &problem));
    (library, code_keys)
}

/// SHA-256（FIPS 180-4）：三种项的键与上传副本、片记录、抽算单元的摘要由它算。独立手写，不与别的装置共用；单测钉空串与 "abc" 的标准值。
fn sha256_hex(message: &[u8]) -> String {
    const ROUND_CONSTANTS: [u32; 64] = [
        0x428a_2f98, 0x7137_4491, 0xb5c0_fbcf, 0xe9b5_dba5, 0x3956_c25b, 0x59f1_11f1, 0x923f_82a4, 0xab1c_5ed5, 0xd807_aa98, 0x1283_5b01,
        0x2431_85be, 0x550c_7dc3, 0x72be_5d74, 0x80de_b1fe, 0x9bdc_06a7, 0xc19b_f174, 0xe49b_69c1, 0xefbe_4786, 0x0fc1_9dc6, 0x240c_a1cc,
        0x2de9_2c6f, 0x4a74_84aa, 0x5cb0_a9dc, 0x76f9_88da, 0x983e_5152, 0xa831_c66d, 0xb003_27c8, 0xbf59_7fc7, 0xc6e0_0bf3, 0xd5a7_9147,
        0x06ca_6351, 0x1429_2967, 0x27b7_0a85, 0x2e1b_2138, 0x4d2c_6dfc, 0x5338_0d13, 0x650a_7354, 0x766a_0abb, 0x81c2_c92e, 0x9272_2c85,
        0xa2bf_e8a1, 0xa81a_664b, 0xc24b_8b70, 0xc76c_51a3, 0xd192_e819, 0xd699_0624, 0xf40e_3585, 0x106a_a070, 0x19a4_c116, 0x1e37_6c08,
        0x2748_774c, 0x34b0_bcb5, 0x391c_0cb3, 0x4ed8_aa4a, 0x5b9c_ca4f, 0x682e_6ff3, 0x748f_82ee, 0x78a5_636f, 0x84c8_7814, 0x8cc7_0208,
        0x90be_fffa, 0xa450_6ceb, 0xbef9_a3f7, 0xc671_78f2,
    ];
    let mut state: [u32; 8] = [0x6a09_e667, 0xbb67_ae85, 0x3c6e_f372, 0xa54f_f53a, 0x510e_527f, 0x9b05_688c, 0x1f83_d9ab, 0x5be0_cd19];
    let message_bits = u64::try_from(message.len()).expect("消息长度装得进 u64") * 8;
    let mut padded = message.to_vec();
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&message_bits.to_be_bytes());
    for block in padded.chunks_exact(64) {
        let mut schedule = [0_u32; 64];
        for (word_position, word) in block.chunks_exact(4).enumerate() {
            schedule[word_position] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for word_position in 16..64 {
            let earlier = schedule[word_position - 15];
            let recent = schedule[word_position - 2];
            let small_sigma_zero = earlier.rotate_right(7) ^ earlier.rotate_right(18) ^ (earlier >> 3);
            let small_sigma_one = recent.rotate_right(17) ^ recent.rotate_right(19) ^ (recent >> 10);
            schedule[word_position] = schedule[word_position - 16].wrapping_add(small_sigma_zero).wrapping_add(schedule[word_position - 7]).wrapping_add(small_sigma_one);
        }
        let mut working = state;
        for (round_constant, scheduled_word) in ROUND_CONSTANTS.iter().zip(schedule) {
            let big_sigma_one = working[4].rotate_right(6) ^ working[4].rotate_right(11) ^ working[4].rotate_right(25);
            let choice = (working[4] & working[5]) ^ (!working[4] & working[6]);
            let first_sum = working[7].wrapping_add(big_sigma_one).wrapping_add(choice).wrapping_add(*round_constant).wrapping_add(scheduled_word);
            let big_sigma_zero = working[0].rotate_right(2) ^ working[0].rotate_right(13) ^ working[0].rotate_right(22);
            let majority = (working[0] & working[1]) ^ (working[0] & working[2]) ^ (working[1] & working[2]);
            let second_sum = big_sigma_zero.wrapping_add(majority);
            working = [first_sum.wrapping_add(second_sum), working[0], working[1], working[2], working[3].wrapping_add(first_sum), working[4], working[5], working[6]];
        }
        for (word, added) in state.iter_mut().zip(working) {
            *word = word.wrapping_add(added);
        }
    }
    state.iter().map(|word| format!("{word:08x}")).collect()
}

/// 参照那一条的相对键：sha256（参照入口的代码键、种子、单元数）。
fn reference_item_of(code_keys: &CodeKeys, seed: u64, units: u64) -> String {
    let key_text = format!("reference\t{}\nseed\t0x{seed:016X}\nunits\t{units}\n", code_keys.reference);
    format!("items/reference-{}.out", sha256_hex(key_text.as_bytes()))
}

/// 参照那一条存的文本：头一行 `reference seed=<十六进制> units=<N>`，之后每单元一行八位十六进制（下标即全局单元号）。
fn reference_item_text(seed: u64, values: &[u32]) -> String {
    let mut text = format!("reference seed=0x{seed:016X} units={}\n", values.len());
    for value in values {
        text.push_str(&format!("{value:08X}\n"));
    }
    text
}

/// 读回参照那一条：头一行与种子、单元数对得上、行数恰好、每行是八位十六进制才算数，不然交 `None`。
fn reference_values_from_item_text(text: &str, seed: u64, units: u64) -> Option<Vec<u32>> {
    let mut lines = text.lines();
    if lines.next()? != format!("reference seed=0x{seed:016X} units={units}") {
        return None;
    }
    let values: Vec<u32> = lines.map(|line| if line.len() == 8 { u32::from_str_radix(line, 16).ok() } else { None }).collect::<Option<Vec<u32>>>()?;
    (values.len() as u64 == units).then_some(values)
}

/// 参照经库（先录）：库里有、认得出就读回；没有（或撤了、强制现判抽中）在 CPU 上现算、写回。交回（参照, 是不是从库里读回的）。不碰卡。
fn reference_through_the_library(library: &DeviceLibrary, code_keys: &CodeKeys, seed: u64, units: u64) -> (Vec<u32>, bool) {
    let item = reference_item_of(code_keys, seed, units);
    if let Some(text) = library.read(&item) {
        match reference_values_from_item_text(&text, seed, units) {
            Some(values) => return (values, true),
            None => eprintln!("e163：库里的 {item} 认不出（头一行或行数对不上），在 CPU 上现算、写回"),
        }
    }
    let values = reference_values_of_the_batch(seed, units);
    library.write(&item, &reference_item_text(seed, &values));
    (values, false)
}

/// 一片在卡上之前、在 CPU 上备好的输入：八锚点的槽与长度、这一片的上传副本（翻位已打进去）与长度。
struct ShardInputs {
    anchor_slots: Vec<u8>,
    anchor_lengths: [u32; 8],
    unit_slots: Vec<u8>,
    lengths: Vec<u32>,
}

/// 八个锚点向量拼成 8 个满槽的连续缓冲区与各自的长度（`run_gpu_crc` 的输入）。
fn anchor_slots_of(vectors: &[Vec<u8>; 8]) -> (Vec<u8>, [u32; 8]) {
    let anchor_lengths: [u32; 8] = std::array::from_fn(|index| u32_from_usize(vectors[index].len()));
    let mut anchor_slots = vec![0u8; 8 * UNIT_BYTE_LENGTH];
    for (index, vector) in vectors.iter().enumerate() {
        anchor_slots[index * UNIT_BYTE_LENGTH..index * UNIT_BYTE_LENGTH + vector.len()].copy_from_slice(vector);
    }
    (anchor_slots, anchor_lengths)
}

/// 一片在卡上算出、进库的那几样；适配器与两段计时只在 `meta ` 行里（复用哨兵比强制现判时不比这几行：换一张卡、换一次计时都不同）。
#[derive(Clone, Debug, PartialEq)]
struct ShardOnTheCard {
    anchors_gpu_match: bool,
    sentinel_and_gid_ok: bool,
    records: Vec<ShardResultRecord>,
    adapter_lines: Vec<String>,
    build_device_milliseconds: f64,
    transfer_compute_readback_milliseconds: f64,
}

/// 片那一条的相对键：sha256（片入口的代码键、起点、单元数、锚点槽与上传副本各自的 sha256、丢的单元号）；全在 CPU 上算，取卡之前就定。
fn shard_item_of(code_keys: &CodeKeys, start: u64, count: u64, inputs: &ShardInputs, drops: &[u64]) -> String {
    let drops_text = if drops.is_empty() { "none".to_string() } else { drops.iter().map(u64::to_string).collect::<Vec<String>>().join(",") };
    let key_text = format!(
        "shard\t{}\nstart\t{start}\ncount\t{count}\nanchors\t{}\ninput\t{}\ndrops\t{drops_text}\n",
        code_keys.shard,
        sha256_hex(&inputs.anchor_slots),
        sha256_hex(&inputs.unit_slots)
    );
    format!("items/shard-{}.out", sha256_hex(key_text.as_bytes()))
}

/// 片那一条存的文本：判定的几行在前，`meta ` 行（计时、适配器）在中间，`records <N>` 之后每条一行 `<全局单元号> <CRC 八位十六进制>`。
fn shard_item_text(start: u64, count: u64, shard: &ShardOnTheCard) -> String {
    let mut text = format!("shard start={start} count={count}\nanchors_gpu_match={}\nsentinel_and_gid_ok={}\n", shard.anchors_gpu_match, shard.sentinel_and_gid_ok);
    text.push_str(&format!("meta build_device_ms={:.3}\nmeta transfer_compute_readback_ms={:.3}\n", shard.build_device_milliseconds, shard.transfer_compute_readback_milliseconds));
    for adapter_line in &shard.adapter_lines {
        text.push_str(&format!("meta adapter {adapter_line}\n"));
    }
    text.push_str(&format!("records {}\n", shard.records.len()));
    for record in &shard.records {
        text.push_str(&format!("{} {:08X}\n", record.global_unit_identifier, record.crc));
    }
    text
}

/// 读回片那一条：头一行与起点、单元数对得上，两行判定、`records <N>` 与恰好 N 条记录都认得出才算数，不然交 `None`。
fn shard_from_item_text(text: &str, start: u64, count: u64) -> Option<ShardOnTheCard> {
    let mut lines = text.lines();
    if lines.next()? != format!("shard start={start} count={count}") {
        return None;
    }
    let anchors_gpu_match = lines.next()?.strip_prefix("anchors_gpu_match=")?.parse().ok()?;
    let sentinel_and_gid_ok = lines.next()?.strip_prefix("sentinel_and_gid_ok=")?.parse().ok()?;
    let (mut build_device_milliseconds, mut transfer_compute_readback_milliseconds, mut adapter_lines) = (0.0, 0.0, Vec::new());
    let record_count: usize = loop {
        let line = lines.next()?;
        if let Some(value) = line.strip_prefix("meta build_device_ms=") {
            build_device_milliseconds = value.parse().ok()?;
        } else if let Some(value) = line.strip_prefix("meta transfer_compute_readback_ms=") {
            transfer_compute_readback_milliseconds = value.parse().ok()?;
        } else if let Some(adapter_line) = line.strip_prefix("meta adapter ") {
            adapter_lines.push(adapter_line.to_string());
        } else {
            break line.strip_prefix("records ")?.parse().ok()?;
        }
    };
    let records: Vec<ShardResultRecord> = lines
        .map(|line| {
            let (identifier_text, crc_text) = line.split_once(' ')?;
            Some(ShardResultRecord { global_unit_identifier: identifier_text.parse().ok()?, crc: u32::from_str_radix(crc_text, 16).ok()? })
        })
        .collect::<Option<Vec<ShardResultRecord>>>()?;
    (records.len() == record_count).then_some(ShardOnTheCard { anchors_gpu_match, sentinel_and_gid_ok, records, adapter_lines, build_device_milliseconds, transfer_compute_readback_milliseconds })
}

/// 一片经库：库里有、认得出就读回，`compute_on_the_card` 不调（不取卡）；没有才调它（取卡、只跑 GPU）、写回。交回（这一片, 是不是从库里读回的）。
fn shard_through_the_library(library: &DeviceLibrary, item: &str, start: u64, count: u64, compute_on_the_card: impl FnOnce() -> ShardOnTheCard) -> (ShardOnTheCard, bool) {
    if let Some(text) = library.read(item) {
        match shard_from_item_text(&text, start, count) {
            Some(stored) => return (stored, true),
            None => eprintln!("e163：库里的 {item} 认不出，取卡现算、写回"),
        }
    }
    let computed = compute_on_the_card();
    library.write(item, &shard_item_text(start, count, &computed));
    (computed, false)
}

/// 合并那一条的相对键：sha256（合并入口的代码键、参照那一条的键、批文件头、比到的单元数、各片记录的 sha256（照给的次序）、抽算三单元的原字节各自的 sha256）。
fn merge_item_of(code_keys: &CodeKeys, reference_item: &str, header_seed: u64, header_unit_count: u64, total_units: u64, shard_records: &[&[ShardResultRecord]], sampled_units: &[(u64, Vec<u8>)]) -> String {
    let mut key_text = format!(
        "merge\t{}\nreference\t{reference_item}\nheader\tseed=0x{header_seed:016X} unit_count={header_unit_count}\ntotal_units\t{total_units}\n",
        code_keys.merge
    );
    for shard in shard_records {
        let records_text: String = shard.iter().map(|record| format!("{} {:08X}\n", record.global_unit_identifier, record.crc)).collect();
        key_text.push_str(&format!("shard\t{}\n", sha256_hex(records_text.as_bytes())));
    }
    for (sample_index, sample_bytes) in sampled_units {
        key_text.push_str(&format!("sample\t{sample_index}\t{}\n", sha256_hex(sample_bytes)));
    }
    format!("items/merge-{}.out", sha256_hex(key_text.as_bytes()))
}

/// 合并经库（后判）：结论那一条在库里就照读回的行打、不读参照；没有才从库里读回参照（`reference_through_the_library`）再比、把打的行写回。
/// 交回（要打的行, 是不是从库里读回的结论）。不碰卡。
fn merge_through_the_library(
    library: &DeviceLibrary,
    code_keys: &CodeKeys,
    header_seed: u64,
    header_unit_count: u64,
    total_units: u64,
    shard_records: &[&[ShardResultRecord]],
    sampled_units: &[(u64, Vec<u8>)],
) -> (Vec<String>, bool) {
    let reference_item = reference_item_of(code_keys, header_seed, total_units);
    let item = merge_item_of(code_keys, &reference_item, header_seed, header_unit_count, total_units, shard_records, sampled_units);
    if let Some(text) = library.read(&item) {
        return (text.lines().map(str::to_string).collect(), true);
    }
    let (cpu_reference, _) = reference_through_the_library(library, code_keys, header_seed, total_units);
    let lines = merge_lines_of(header_seed, header_unit_count, total_units, &cpu_reference, shard_records, sampled_units);
    library.write(&item, &lines.iter().map(|line| format!("{line}\n")).collect::<String>());
    (lines, false)
}

// ============================== 命令行 ==============================

fn parse_named_argument(arguments: &[String], name: &str) -> Option<String> {
    arguments.iter().position(|argument| argument == name).map(|index| arguments[index + 1].clone())
}

fn parse_repeated_named_argument(arguments: &[String], name: &str) -> Vec<String> {
    arguments
        .iter()
        .zip(arguments.iter().skip(1))
        .filter(|(flag, _)| *flag == name)
        .map(|(_, value)| value.clone())
        .collect()
}

fn parse_bit_flip_specification(specification: &str) -> BitFlipInjection {
    let mut parts = specification.split(':');
    let global_unit_identifier: u64 = parts.next().expect("翻位注入缺全局单元号").parse().expect("全局单元号不是数字");
    let byte_offset_within_unit: usize = parts.next().expect("翻位注入缺字节偏移").parse().expect("字节偏移不是数字");
    let bit_index_within_byte: u32 = parts.next().expect("翻位注入缺位号").parse().expect("位号不是数字");
    BitFlipInjection { global_unit_identifier, byte_offset_within_unit, bit_index_within_byte }
}

/// `probe`：只枚举、不建设备（登记第六节 Q4：另一台上「--probe」的退出码、stderr、列出的 NVIDIA 卡数）。
fn command_probe() {
    let instance = new_vulkan_instance();
    let adapters = nvidia_discrete_adapters(&instance);
    for adapter in &adapters {
        println!("PROBE {}", adapter_information_fields(&adapter.get_info()));
    }
    println!("PROBE_COUNT={}", adapters.len());
}

/// `anchors --adapter-index N`：算臂 C 与第 N 张卡（`adapter_for_index`：服务给这一台的第 N 张；没给卡退 78）的第七节 B 类八锚点，
/// 都与登记的 `EXPECTED_ANCHOR_VALUES` 比对（PC-A，登记第五节「阳性对照」）。
fn command_anchors(arguments: &[String]) {
    let adapter_index: usize = parse_named_argument(arguments, "--adapter-index").expect("anchors 需要 --adapter-index").parse().expect("adapter-index 不是数字");

    let vectors = anchor_vectors();
    let cpu_values: [u32; 8] = std::array::from_fn(|index| crc32c_reference(&vectors[index]));
    let cpu_mismatches = anchor_mismatches(&cpu_values);
    for index in 0..8 {
        println!("ANCHOR_CPU name={} expected={:08X} got={:08X}", ANCHOR_VECTOR_NAMES[index], EXPECTED_ANCHOR_VALUES[index], cpu_values[index]);
    }
    println!("ANCHOR_CPU_MATCH={}", cpu_mismatches.is_empty());
    let (unit_slots, lengths) = anchor_slots_of(&vectors);

    // 上面全在 CPU 上：到这里才取卡、开设备；GPU 跑完丢上下文，再核
    let gpu_results = {
        let instance = new_vulkan_instance();
        let adapter = adapter_for_index(&instance, adapter_index);
        let context = build_gpu_context(adapter).unwrap_or_else(|error| panic!("第 {adapter_index} 张卡建不起上下文：{error}"));
        println!("ANCHOR_ADAPTER {}", adapter_information_fields(&context.adapter_information));
        run_gpu_crc(&context, &unit_slots, &lengths)
    };
    assert_full_coverage_no_sentinel(&gpu_results);
    let gpu_values: [u32; 8] = std::array::from_fn(|index| gpu_results[index].0);
    let gpu_mismatches = anchor_mismatches(&gpu_values);
    for index in 0..8 {
        println!("ANCHOR_GPU name={} expected={:08X} got={:08X}", ANCHOR_VECTOR_NAMES[index], EXPECTED_ANCHOR_VALUES[index], gpu_values[index]);
    }
    println!("ANCHOR_GPU_MATCH={}", gpu_mismatches.is_empty());
    if !cpu_mismatches.is_empty() || !gpu_mismatches.is_empty() {
        std::process::exit(1);
    }
}

/// 登记的输入单元数（登记第七节 N = 2560）；`gen-input` 不给 `--units` 就是它。
const REGISTERED_UNIT_COUNT: u64 = 2560;

/// 给了与登记不同的单元数就是缩过规模的一趟：`GEN_INPUT` 行接这一段，产物不当正式产物。
fn reduced_scale_note(units: u64) -> String {
    if units == REGISTERED_UNIT_COUNT {
        String::new()
    } else {
        format!(" reduced=yes registered_units={REGISTERED_UNIT_COUNT} not_official_product=yes")
    }
}

/// `--units`（不给是登记的 N）与 `--seed <十六进制，可带 0x>`：`gen-input` 与 `record` 同一种写法。
fn units_and_seed_arguments(arguments: &[String], subcommand: &str) -> (u64, u64) {
    let units: u64 = parse_named_argument(arguments, "--units").map(|text| text.parse().expect("units 不是数字")).unwrap_or(REGISTERED_UNIT_COUNT);
    let seed_text = parse_named_argument(arguments, "--seed").unwrap_or_else(|| panic!("{subcommand} 需要 --seed"));
    let seed_text = seed_text.strip_prefix("0x").unwrap_or(&seed_text);
    let seed = u64::from_str_radix(seed_text, 16).expect("seed 不是合法的十六进制");
    (units, seed)
}

fn command_generate_input(arguments: &[String]) {
    let output_path = PathBuf::from(parse_named_argument(arguments, "--output").expect("gen-input 需要 --output"));
    let (units, seed) = units_and_seed_arguments(arguments, "gen-input");
    write_batch_file(&output_path, units, seed).expect("写批文件失败");
    println!("GEN_INPUT path={output_path:?} units={units} seed=0x{seed:016X} bytes={}{}", units as usize * UNIT_BYTE_LENGTH + 32, reduced_scale_note(units));
}

/// `record --seed <十六进制> [--units N]`：先录——在 CPU 上算整批参照、写进库（`reference_through_the_library`；库里已有就读回、不重算），不建 wgpu 实例、不取卡。
fn command_record(arguments: &[String]) {
    let (library, code_keys) = library_and_code_keys(arguments, "record");
    let (units, seed) = units_and_seed_arguments(arguments, "record");
    let (_, reused_from_the_library) = reference_through_the_library(&library, &code_keys, seed, units);
    println!(
        "RECORD reference_item={} units={units} seed=0x{seed:016X} reused_from_the_library={reused_from_the_library}{}",
        reference_item_of(&code_keys, seed, units),
        reduced_scale_note(units)
    );
}

/// `run-shard`：一片的完整流程——在 CPU 上算 PC-A 臂 C 锚点、读输入区间、按需翻位（只改上传副本）、算片那一条的键；
/// 库里有就照库里的写结果文件、不取卡；没有才取卡、只跑 GPU（`gpu_values_of_the_shard`）、写回库；再写结果文件
/// （登记第五节「跑 R1」「跑 R2」「跑 F1、F2」）。`total_ms` 从参数解析完算到写结果文件之前，含取卡前的 CPU 活与等卡。
fn command_run_shard(arguments: &[String]) {
    let input_path = PathBuf::from(parse_named_argument(arguments, "--input").expect("run-shard 需要 --input"));
    let start: u64 = parse_named_argument(arguments, "--start").expect("run-shard 需要 --start").parse().expect("start 不是数字");
    let count: u64 = parse_named_argument(arguments, "--count").expect("run-shard 需要 --count").parse().expect("count 不是数字");
    let adapter_index: usize = parse_named_argument(arguments, "--adapter-index").expect("run-shard 需要 --adapter-index").parse().expect("adapter-index 不是数字");
    let output_path = PathBuf::from(parse_named_argument(arguments, "--output").expect("run-shard 需要 --output"));
    let flips: Vec<BitFlipInjection> = parse_repeated_named_argument(arguments, "--inject-flip").iter().map(|specification| parse_bit_flip_specification(specification)).collect();
    let drops: Vec<u64> = parse_repeated_named_argument(arguments, "--inject-drop").iter().map(|text| text.parse().expect("drop 的全局单元号不是数字")).collect();

    let (library, code_keys) = library_and_code_keys(arguments, "run-shard");

    let total_start = Instant::now();

    // 取卡之前全在 CPU 上：PC-A 臂 C 的八锚点、读输入区间、翻位（只改上传副本）、拼锚点槽、算片那一条的键
    let vectors = anchor_vectors();
    let cpu_anchor_values: [u32; 8] = std::array::from_fn(|index| crc32c_reference(&vectors[index]));
    let anchors_cpu_match = anchor_mismatches(&cpu_anchor_values).is_empty();
    let (anchor_slots, anchor_lengths) = anchor_slots_of(&vectors);

    let header = read_batch_header(&input_path).expect("读批文件头失败");
    eprintln!("RUN_SHARD_INPUT_HEADER seed=0x{:016X} unit_count={}", header.seed, header.unit_count);
    let mut unit_slots = read_unit_range(&input_path, &header, start, count).expect("读输入区间失败");
    for flip in &flips {
        assert!(flip.global_unit_identifier >= start && flip.global_unit_identifier < start + count, "翻位注入的单元号不在这一片范围内");
        let local_index = (flip.global_unit_identifier - start) as usize;
        let slot_start = local_index * UNIT_BYTE_LENGTH;
        apply_bit_flip(&mut unit_slots[slot_start..slot_start + UNIT_BYTE_LENGTH], flip.byte_offset_within_unit, flip.bit_index_within_byte);
    }
    let lengths = vec![UNIT_BYTE_LENGTH as u32; count as usize];
    let inputs = ShardInputs { anchor_slots, anchor_lengths, unit_slots, lengths };
    let item = shard_item_of(&code_keys, start, count, &inputs, &drops);

    // 库里有这一片就不取卡；没有才到这一刻取卡（服务交的，或这一刻领）、只跑 GPU
    let (shard, reused_from_the_library) = shard_through_the_library(&library, &item, start, count, || {
        let instance = new_vulkan_instance();
        let adapter = adapter_for_index(&instance, adapter_index);
        gpu_values_of_the_shard(adapter, &inputs, start, &drops).unwrap_or_else(|error| panic!("第 {adapter_index} 张卡建不起上下文：{error}"))
    });
    eprintln!("RUN_SHARD_ADAPTER index={adapter_index} reused_from_the_library={reused_from_the_library} {}", shard.adapter_lines.join(" "));

    let total_milliseconds = total_start.elapsed().as_secs_f64() * 1000.0;
    write_shard_result_file(
        &output_path,
        &item,
        reused_from_the_library,
        &shard.adapter_lines,
        start,
        count,
        shard.build_device_milliseconds,
        shard.transfer_compute_readback_milliseconds,
        total_milliseconds,
        &flips,
        &drops,
        shard.sentinel_and_gid_ok,
        anchors_cpu_match,
        shard.anchors_gpu_match,
        &shard.records,
    )
    .expect("写结果文件失败");
    println!(
        "RUN_SHARD start={start} count={count} adapter_index={adapter_index} build_device_ms={:.3} \
transfer_compute_readback_ms={:.3} total_ms={total_milliseconds:.3} \
anchors_cpu_match={anchors_cpu_match} anchors_gpu_match={} sentinel_and_gid_ok={} \
record_count={} reused_from_the_library={reused_from_the_library} output={output_path:?}",
        shard.build_device_milliseconds,
        shard.transfer_compute_readback_milliseconds,
        shard.anchors_gpu_match,
        shard.sentinel_and_gid_ok,
        shard.records.len()
    );
}

/// `merge`（后判，不取卡）：读各片结果文件与抽算三单元的原字节，合并那一条在库里就照读回的行打；没有才从库里读回 `record` 录下的整批参照
/// （读不回的在 CPU 上现算、写回），与各片结果比对，逐量打印登记第六节 Q2/Q3/Q5/Q7/Q8/Q9 需要的计数（作废 2 的另两条断言：2560 个值两两不同、
/// 抽三单元与臂 C 同值，在这里一并做；`merge_lines_of`），打的行写回库。标准输出只有这几行（复跑表逐字比）；读库的结局打标准错误。
fn command_merge(arguments: &[String]) {
    let original_input_path = PathBuf::from(parse_named_argument(arguments, "--original-input").expect("merge 需要 --original-input"));
    let total_units: u64 = parse_named_argument(arguments, "--total-units").expect("merge 需要 --total-units").parse().expect("total-units 不是数字");
    let shard_paths: Vec<PathBuf> = parse_repeated_named_argument(arguments, "--shard").iter().map(PathBuf::from).collect();
    assert!(!shard_paths.is_empty(), "merge 至少要给一个 --shard");
    let (library, code_keys) = library_and_code_keys(arguments, "merge");

    let header = read_batch_header(&original_input_path).expect("读批文件头失败");
    assert!(header.unit_count >= total_units, "批文件的单元数比 --total-units 还少");
    let shard_records: Vec<Vec<ShardResultRecord>> = shard_paths.iter().map(|path| read_shard_result_file(path).unwrap_or_else(|error| panic!("读结果文件 {path:?} 失败：{error}"))).collect();
    let shard_record_slices: Vec<&[ShardResultRecord]> = shard_records.iter().map(Vec::as_slice).collect();
    let sampled_units: Vec<(u64, Vec<u8>)> = sampled_unit_indices(total_units)
        .into_iter()
        .map(|sample_index| (sample_index, read_unit_range(&original_input_path, &header, sample_index, 1).expect("抽算失败")))
        .collect();

    let (lines, reused_from_the_library) = merge_through_the_library(&library, &code_keys, header.seed, header.unit_count, total_units, &shard_record_slices, &sampled_units);
    for line in &lines {
        println!("{line}");
    }
    eprintln!("MERGE_LIBRARY reference_item={} reused_from_the_library={reused_from_the_library}", reference_item_of(&code_keys, header.seed, total_units));
}

/// 开跑前判准入与运行条件（`.claude/singlefs-ai-sop/rules/preflight-discipline.md`「开头先判」Rust 那一行）。
/// `run-shard` 要在另一台上跑（那边只有这个二进制，没有源文件与规范副本），不判；别的子命令都在本机、先判。
fn preflight() {
    let arguments: Vec<String> = env::args().skip(1).collect();
    if arguments.first().is_some_and(|subcommand| subcommand == "run-shard") {
        return;
    }
    let source = concat!(env!("CARGO_MANIFEST_DIR"), "/e163_gpu_multicard_crc32c.rs");
    let script = concat!(env!("CARGO_MANIFEST_DIR"), "/../../.claude/singlefs-ai-sop/scripts/preflight.py");
    let mut command = std::process::Command::new("python3");
    command.arg(script).arg("check").arg(source);
    if arguments.iter().any(|argument| argument == "--force") {
        command.arg("--force");
    }
    command.arg("--");
    command.args(arguments.iter().filter(|argument| *argument != "--force"));
    let output = command.output().unwrap_or_else(|error| {
        eprintln!("  ✗ 起不了 python3 判准入：{error}");
        eprintln!("  → 怎么办：装上 python3，或在有 python3 的机器上跑");
        std::process::exit(78)
    });
    if !output.status.success() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
        print!("{}", String::from_utf8_lossy(&output.stdout));
        std::process::exit(output.status.code().unwrap_or(1));
    }
}

fn main() {
    preflight();
    let arguments: Vec<String> = env::args().collect();
    let subcommand = arguments.get(1).map(String::as_str).unwrap_or("");
    let rest = &arguments[1.min(arguments.len())..];
    match subcommand {
        "probe" => command_probe(),
        "anchors" => command_anchors(rest),
        "gen-input" => command_generate_input(rest),
        "record" => command_record(rest),
        "run-shard" => command_run_shard(rest),
        "merge" => command_merge(rest),
        other => {
            eprintln!("认不出的子命令「{other}」，要 probe / anchors / gen-input / record / run-shard / merge 之一");
            std::process::exit(2);
        }
    }
}

// ============================== 单测 ==============================
//
// 不碰 GPU 的几条（含认交来的卡、取卡来路、经内存库先录后判的几条）`cargo test --release --features e163-gpu --bin e163-gpu-multicard-crc32c` 就能跑；
// 三条 GPU 用例先在 CPU 上备好输入，再只开服务给这一台的头一张卡（`handed_over_card_context`：环境里有卡串照它，没有而有领卡通道就在这一刻领）：
// 两样都没有、或卡串里没有这一台的，每条打 `NOT_RUN_NO_GPU_ADAPTER test=<用例名> reason=<一句>` 报本次未跑、不开卡
// （门禁 research-unit-tests 格把这一行判红：那一格给了卡才跑这个目标）；领卡没成、交来的卡对不上本机任何一张独显、建不起上下文都 panic，不退回别的卡。

#[cfg(test)]
mod tests {
    use super::*;

    /// `cargo test` 默认并行跑各条测试；三条要 GPU 的测试都对同一张（交来的那一张）物理卡
    /// 各建一份 `wgpu::Instance` / `Device` 并派发计算，三个线程同时撞上时会卡死在某个
    /// GPU 侧的阻塞调用里（`request_device` 或 `device.poll(Wait)`），既不报错也不超时——
    /// 实测：同一套代码单独跑全套测试 0.58 秒通过，另一次三线程撞上后挂起 39
    /// 分钟没有任何输出。这把锁把「碰不碰同一张卡」的测试序列化，让默认并行的 `cargo test`
    /// 也不再互相踩踏。
    static GPU_TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// 服务给这一台的头一张卡上建的 GPU 上下文（`cards_of_this_process`：卡串在环境里照它，没有而有领卡通道就在这一刻领，整个测试进程只领一回）；
    /// 两样都没有、给的卡里没有这一台的打 `NOT_RUN_NO_GPU_ADAPTER test=<用例名> reason=<一句>`（写进 `std::io::stdout()` 本身，libtest 不截）、交 `None`，不开卡；
    /// 领卡没成、卡串读不成、取不到这一台的地址、交来的卡对不上本机独显都 panic。调用方先在 CPU 上备好输入再调它。
    fn handed_over_card_context(test_name: &str) -> Option<GpuContext> {
        let cards = cards_of_this_process().unwrap_or_else(|reason| panic!("取服务的卡没成：{reason}"));
        let cards_of_this_host = match cards_handed_to_this_host(cards.as_deref(), this_host_address_by_the_configuration_check).unwrap_or_else(|reason| panic!("服务交的卡认不成：{reason}"))
        {
            CardsForThisHost::NotRun { reason } => {
                writeln!(std::io::stdout().lock(), "NOT_RUN_NO_GPU_ADAPTER test={test_name} reason={reason}").expect("写不出本次未跑那一行");
                return None;
            }
            CardsForThisHost::Handed(cards_of_this_host) => cards_of_this_host,
        };
        let instance = new_vulkan_instance();
        let adapter = handed_over_adapter(&instance, &cards_of_this_host, 0).unwrap_or_else(|reason| panic!("{reason}"));
        Some(build_gpu_context(adapter).expect("建 GPU 上下文失败"))
    }

    /// 本机地址的假取法：认交来的卡那几条不起多机配置判法。
    fn this_host_is(address: &'static str) -> impl FnOnce() -> Result<String, String> {
        move || Ok(address.to_string())
    }

    #[test]
    fn two_cards_handed_to_this_host_take_the_first() {
        let cards = cards_handed_to_this_host(Some("10.0.0.1:2:8G 10.0.0.1:0:8G"), this_host_is("10.0.0.1")).expect("两张卡的串读得成");
        assert_eq!(cards, CardsForThisHost::Handed(vec![2, 0]), "照交来的次序：单测开的头一张是序号 2，不是本机枚举序的头一张");
        let mixed = cards_handed_to_this_host(Some("10.0.0.2:0:8G 10.0.0.1:1:8G"), this_host_is("10.0.0.1")).expect("两台的卡混着交读得成");
        assert_eq!(mixed, CardsForThisHost::Handed(vec![1]), "别的台的卡不算这一台交来的头一张");
    }

    #[test]
    fn empty_or_unset_handover_is_not_run_without_asking_the_host() {
        for handover_value in [None, Some(""), Some("   ")] {
            let outcome = cards_handed_to_this_host(handover_value, || panic!("没交卡就不该问这一台是谁")).expect("没交卡不是读不成");
            assert!(matches!(outcome, CardsForThisHost::NotRun { .. }), "{handover_value:?} 应当判本次未跑，实际 {outcome:?}");
        }
    }

    #[test]
    fn cards_of_another_host_or_unreadable_entries_are_not_taken() {
        let elsewhere = cards_handed_to_this_host(Some("10.0.0.2:0:8G"), this_host_is("10.0.0.1")).expect("别的台的卡读得成");
        assert!(matches!(elsewhere, CardsForThisHost::NotRun { .. }), "交的卡里没有这一台的应当判本次未跑，实际 {elsewhere:?}");
        for unreadable in ["sample-card", "10.0.0.1:x:8G", "10.0.0.1:0:", "10.0.0.1:0:8G 10.0.0.1:0:4G"] {
            assert!(cards_handed_to_this_host(Some(unreadable), this_host_is("10.0.0.1")).is_err(), "「{unreadable}」应当读不成、判红");
        }
        let ipv6 = cards_handed_over("fe80::1:3:8G").expect("IPv6 地址自带冒号，从右边切两刀");
        assert_eq!(ipv6, vec![CardHandedOver { host_address: "fe80::1".to_string(), nvidia_smi_index: 3 }]);
    }

    #[test]
    fn handed_over_card_matches_the_adapter_on_its_bus() {
        let buses = pci_buses_by_nvidia_smi_index("0, 00000000:01:00.0\n1, 00000000:41:00.0\n").expect("nvidia-smi 的两行读得成");
        let adapter_pci_buses = vec!["0000:41:00.0".to_string(), "0000:01:00.0".to_string()];
        assert_eq!(position_of_the_handed_over_card(0, &buses, &adapter_pci_buses), Ok(1), "序号 0 在总线 01:00.0，是枚举里的第 1 个");
        assert_eq!(position_of_the_handed_over_card(1, &buses, &adapter_pci_buses), Ok(0));
        assert!(position_of_the_handed_over_card(2, &buses, &adapter_pci_buses).is_err(), "nvidia-smi 没报的序号判红");
        assert!(position_of_the_handed_over_card(0, &buses, &["0000:81:00.0".to_string()]).is_err(), "总线上没有独显判红");
        assert_eq!(this_host_address_emitted("HOSTS=(a)\nTHIS_HOST_ADDRESS=10.0.0.1\n"), Ok("10.0.0.1".to_string()));
        assert!(this_host_address_emitted("THIS_HOST_ADDRESS=''\n").is_err(), "空地址判红");
    }

    #[test]
    fn crc32c_reference_matches_prereg_anchor_vectors() {
        let vectors = anchor_vectors();
        for index in 0..8 {
            let computed = crc32c_reference(&vectors[index]);
            assert_eq!(computed, EXPECTED_ANCHOR_VALUES[index], "锚点「{}」算出来的值与登记的不一致", ANCHOR_VECTOR_NAMES[index]);
        }
    }

    #[test]
    fn compare_reference_and_shards_reports_injected_mismatch() {
        let cpu_reference = vec![0x1111_1111u32, 0x2222_2222u32, 0x3333_3333u32, 0x4444_4444u32];
        let shard = vec![
            ShardResultRecord { global_unit_identifier: 0, crc: 0x1111_1111 },
            ShardResultRecord { global_unit_identifier: 1, crc: 0xDEAD_BEEF }, // 故意错
            ShardResultRecord { global_unit_identifier: 2, crc: 0x3333_3333 },
            ShardResultRecord { global_unit_identifier: 3, crc: 0x4444_4444 },
        ];
        let report = compare_reference_and_shards(&cpu_reference, &[shard.as_slice()]);
        assert_eq!(report.mismatched_global_unit_identifiers, vec![1], "V1 要抓的场景：合并比对必须报出这一个不一致");
        assert_eq!(report.matched_count, 3);
        assert!(report.missing_global_unit_identifiers.is_empty());
    }

    #[test]
    fn compare_reference_and_shards_reports_missing_unit() {
        let cpu_reference = vec![0x1111_1111u32, 0x2222_2222u32, 0x3333_3333u32];
        let shard = vec![ShardResultRecord { global_unit_identifier: 0, crc: 0x1111_1111 }, ShardResultRecord { global_unit_identifier: 2, crc: 0x3333_3333 }];
        let report = compare_reference_and_shards(&cpu_reference, &[shard.as_slice()]);
        assert_eq!(report.missing_global_unit_identifiers, vec![1], "V2 要抓的场景：合并比对必须按 0..N 查出缺的那个");
        assert_eq!(report.matched_count, 2);
    }

    #[test]
    fn compare_reference_and_shards_reports_duplicate_unit() {
        let cpu_reference = vec![0x1111_1111u32];
        let shard_one = vec![ShardResultRecord { global_unit_identifier: 0, crc: 0x1111_1111 }];
        let shard_two = vec![ShardResultRecord { global_unit_identifier: 0, crc: 0x1111_1111 }];
        let report = compare_reference_and_shards(&cpu_reference, &[shard_one.as_slice(), shard_two.as_slice()]);
        assert_eq!(report.duplicated_global_unit_identifiers, vec![0]);
    }

    #[test]
    fn generate_unit_bytes_is_deterministic_and_full_length() {
        let first = generate_unit_bytes(0xE163_0927_0000_0001, 42);
        let second = generate_unit_bytes(0xE163_0927_0000_0001, 42);
        assert_eq!(first, second, "同样的种子与全局单元号，两次生成必须逐字节相同");
        assert_eq!(first.len(), UNIT_BYTE_LENGTH);
        let different_unit = generate_unit_bytes(0xE163_0927_0000_0001, 43);
        assert_ne!(first, different_unit, "相邻单元号的内容不该恰好相同");
    }

    #[test]
    fn apply_bit_flip_changes_exactly_one_bit() {
        let mut bytes = vec![0u8; 8];
        apply_bit_flip(&mut bytes, 3, 5);
        assert_eq!(bytes[3], 0b0010_0000);
        for (index, &value) in bytes.iter().enumerate() {
            if index != 3 {
                assert_eq!(value, 0, "只该有第 3 个字节被改动");
            }
        }
    }

    #[test]
    fn batch_file_round_trip_matches_generator() {
        let path = std::env::temp_dir().join(format!("e163-batch-round-trip-{}.bin", std::process::id()));
        write_batch_file(&path, 5, 0xABCD_1234_0000_0001).expect("写批文件失败");
        let header = read_batch_header(&path).expect("读头失败");
        assert_eq!(header.unit_count, 5);
        assert_eq!(header.unit_byte_length as usize, UNIT_BYTE_LENGTH);
        let read_back = read_unit_range(&path, &header, 2, 1).expect("读区间失败");
        let expected = generate_unit_bytes(0xABCD_1234_0000_0001, 2);
        assert_eq!(read_back, expected, "从批文件读出的第 2 个单元应当与生成函数直接算出的一致");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn gpu_matches_cpu_reference_on_anchor_vectors() {
        let _gpu_test_guard = GPU_TEST_MUTEX.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let (unit_slots, lengths) = anchor_slots_of(&anchor_vectors());
        let Some(context) = handed_over_card_context("gpu_matches_cpu_reference_on_anchor_vectors") else {
            return;
        };
        let gpu_results = run_gpu_crc(&context, &unit_slots, &lengths);
        drop(context);
        assert_full_coverage_no_sentinel(&gpu_results);
        for index in 0..8 {
            assert_eq!(gpu_results[index].0, EXPECTED_ANCHOR_VALUES[index], "GPU 在锚点「{}」上应当与登记的期望值一致", ANCHOR_VECTOR_NAMES[index]);
        }
    }

    #[test]
    fn gpu_dispatch_covers_every_unit_no_sentinel_residue() {
        let _gpu_test_guard = GPU_TEST_MUTEX.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let count = 200usize;
        let unit_slots = vec![0u8; count * UNIT_BYTE_LENGTH];
        let lengths = vec![UNIT_BYTE_LENGTH as u32; count];
        let Some(context) = handed_over_card_context("gpu_dispatch_covers_every_unit_no_sentinel_residue") else {
            return;
        };
        let results = run_gpu_crc(&context, &unit_slots, &lengths);
        drop(context);
        assert_eq!(results.len(), count);
        assert_full_coverage_no_sentinel(&results);
    }

    #[test]
    fn injected_bit_flip_changes_gpu_result_but_not_original_cpu_reference() {
        let _gpu_test_guard = GPU_TEST_MUTEX.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        // 先在 CPU 上：造原单元、算参照、翻位只进上传副本；之后才取卡，卡上只跑 GPU
        let original = generate_unit_bytes(0xE163_0927_0000_0002u64, 7);
        let cpu_reference_on_original = crc32c_reference(&original);
        let mut uploaded_copy = original.clone();
        apply_bit_flip(&mut uploaded_copy, 4099, 1);
        let Some(context) = handed_over_card_context("injected_bit_flip_changes_gpu_result_but_not_original_cpu_reference") else {
            return;
        };
        let results = run_gpu_crc(&context, &uploaded_copy, &[UNIT_BYTE_LENGTH as u32]);
        drop(context);
        assert_full_coverage_no_sentinel(&results);
        assert_ne!(results[0].0, cpu_reference_on_original, "V7 要抓的场景：翻位只进了上传副本，GPU 结果必须与未翻位的 CPU 参照不同");
    }

    /// 给了缩规模的单元数，`GEN_INPUT` 行要写缩过、不当正式产物；登记的 2560 不加这一段。
    #[test]
    fn reduced_unit_count_is_marked_and_registered_one_is_not() {
        assert_eq!(REGISTERED_UNIT_COUNT, 2560);
        assert_eq!(reduced_scale_note(REGISTERED_UNIT_COUNT), "");
        assert_eq!(reduced_scale_note(64), " reduced=yes registered_units=2560 not_official_product=yes");
    }

    // ---------- 取卡来路（不起真卡、不连服务） ----------

    /// 单测的领卡通道：五个变量都给、值是名字本身。
    fn card_lease_channel_for_the_unit_tests() -> CardLeaseChannel {
        card_lease_channel_of(&|name| Some(format!("value-of-{name}"))).expect("五个变量都给了")
    }

    #[test]
    fn handed_over_cards_are_used_without_leasing() {
        let channel = card_lease_channel_for_the_unit_tests();
        let leases = std::cell::Cell::new(0_u32);
        let lease = |_channel: &CardLeaseChannel| -> Result<String, String> {
            leases.set(leases.get() + 1);
            Ok("10.0.0.1:1:8G".to_string())
        };
        assert_eq!(cards_handed_over_or_leased(Some("10.0.0.1:0:8G"), Some(&channel), &lease), Ok(Some("10.0.0.1:0:8G".to_string())));
        assert_eq!(cards_handed_over_or_leased(Some("10.0.0.1:0:8G"), None, &lease), Ok(Some("10.0.0.1:0:8G".to_string())));
        assert_eq!(leases.get(), 0, "环境里已有卡串就不领");
    }

    #[test]
    fn without_handed_over_cards_a_lease_channel_leases_at_that_moment() {
        let channel = card_lease_channel_for_the_unit_tests();
        let leases = std::cell::Cell::new(0_u32);
        let lease = |_channel: &CardLeaseChannel| -> Result<String, String> {
            leases.set(leases.get() + 1);
            Ok("10.0.0.1:1:8G".to_string())
        };
        for handover_value in [None, Some(""), Some("  ")] {
            assert_eq!(cards_handed_over_or_leased(handover_value, Some(&channel), &lease), Ok(Some("10.0.0.1:1:8G".to_string())), "{handover_value:?} 当没交卡串");
        }
        assert_eq!(leases.get(), 3, "每问一次领一回（进程里只领一回由 cards_leased_by_this_process 管）");
    }

    #[test]
    fn without_cards_or_a_lease_channel_nothing_is_taken() {
        let lease = |_channel: &CardLeaseChannel| -> Result<String, String> { panic!("没有领卡通道就不该领") };
        assert_eq!(cards_handed_over_or_leased(None, None, &lease), Ok(None));
        assert_eq!(cards_handed_over_or_leased(Some(" "), None, &lease), Ok(None));
    }

    #[test]
    fn a_refused_lease_is_an_error_not_a_fallback() {
        let channel = card_lease_channel_for_the_unit_tests();
        let refused = |_channel: &CardLeaseChannel| -> Result<String, String> { Err("服务回 refused".to_string()) };
        assert_eq!(cards_handed_over_or_leased(None, Some(&channel), &refused), Err("服务回 refused".to_string()));
    }

    #[test]
    fn a_lease_channel_needs_all_five_variables_present_and_not_empty() {
        let channel = card_lease_channel_for_the_unit_tests();
        assert_eq!(channel.variables.len(), CARD_LEASE_CHANNEL_VARIABLES.len());
        for missing in CARD_LEASE_CHANNEL_VARIABLES {
            assert_eq!(card_lease_channel_of(&|name| (name != missing).then(|| format!("value-of-{name}"))), None, "缺 {missing} 不算有通道");
            assert_eq!(card_lease_channel_of(&|name| Some(if name == missing { String::new() } else { format!("value-of-{name}") })), None, "{missing} 是空的不算有通道");
        }
    }

    /// 这一条用例自己的临时目录（照进程号与名字分开，用完删）。
    fn temporary_directory_for(name: &str) -> PathBuf {
        let directory = std::env::temp_dir().join(format!("e163-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).expect("建得了临时目录");
        directory
    }

    #[test]
    fn the_lease_client_is_under_the_source_repository_or_found_upward() {
        let directory = temporary_directory_for("lease-client-path");
        let repository = directory.join("repository");
        let started_in = repository.join("scripts").join("experiment-apparatus");
        std::fs::create_dir_all(&started_in).expect("建得了起点目录");
        std::fs::create_dir_all(repository.join("task-server")).expect("建得了 task-server 目录");
        std::fs::write(repository.join(CARD_LEASE_CLIENT_IN_THE_REPOSITORY), "#!/bin/sh\n").expect("写得了假客户端");
        let client = repository.join(CARD_LEASE_CLIENT_IN_THE_REPOSITORY);
        assert_eq!(card_lease_client_from(Some(repository.clone()), &directory), Ok(client.clone()), "服务交的仓根下那一份");
        assert_eq!(card_lease_client_from(None, &started_in), Ok(client), "没交仓根时从起点往上找");
        assert!(card_lease_client_from(Some(directory.join("elsewhere")), &started_in).is_err(), "交的仓根下没有就判错，不往上找");
        std::fs::remove_dir_all(&directory).expect("删得了临时目录");
    }

    #[test]
    fn leasing_through_a_fake_client_takes_then_returns_the_cards() {
        use std::os::unix::fs::PermissionsExt;
        let directory = temporary_directory_for("lease-through-a-fake-client");
        let calls = directory.join("calls.log");
        let client = directory.join("fake-card-lease-client");
        std::fs::write(
            &client,
            format!(
                "#!/bin/sh\nprintf '%s|%s\\n' \"$*\" \"$TASK_SERVER_CARD_LEASE_TOKEN\" >> '{}'\n[ \"$1 $2\" = 'lease take' ] || exit 0\nprintf '10.0.0.1:0:8G\\n'\n",
                calls.display()
            ),
        )
        .expect("写得了假客户端");
        std::fs::set_permissions(&client, std::fs::Permissions::from_mode(0o755)).expect("设得了执行位");
        let channel = card_lease_channel_for_the_unit_tests();
        let leased = lease_cards_through_the_client(&client, &channel).expect("假客户端交了卡串");
        assert_eq!(leased.cards, "10.0.0.1:0:8G");
        assert!(leased.return_the_cards_now(), "还卡的客户端退 0");
        let logged = std::fs::read_to_string(&calls).expect("假客户端记了调用");
        assert_eq!(logged, "lease take|value-of-TASK_SERVER_CARD_LEASE_TOKEN\nlease return 10.0.0.1:0:8G|value-of-TASK_SERVER_CARD_LEASE_TOKEN\n", "先领、通道变量交给客户端，再照卡串还");
        std::fs::remove_dir_all(&directory).expect("删得了临时目录");
    }

    // ---------- 先录后判（内存库，不起卡） ----------

    fn code_keys_for_the_unit_tests() -> CodeKeys {
        CodeKeys { reference: "a1".to_string(), shard: "b2".to_string(), merge: "c3".to_string() }
    }

    #[test]
    fn sha256_matches_the_standard_values() {
        assert_eq!(sha256_hex(b""), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        assert_eq!(sha256_hex(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    }

    #[test]
    fn the_composition_needs_exactly_the_three_entries() {
        let reference = "code_key\te7_index_bench::reference_values_of_the_batch\ta1\n";
        let shard = "code_key\te7_index_bench::gpu_values_of_the_shard\tb2\n";
        let merge = "code_key\te7_index_bench::merge_lines_of\tc3\n";
        assert_eq!(code_keys_from_composition(&format!("{COMPOSITION_FORM_LINE}\n{reference}{shard}{merge}")), Ok(code_keys_for_the_unit_tests()));
        assert!(code_keys_from_composition(&format!("{reference}{shard}{merge}")).is_err(), "缺 form 行");
        assert!(code_keys_from_composition(&format!("{COMPOSITION_FORM_LINE}\n{reference}{shard}")).is_err(), "缺合并入口");
        assert!(code_keys_from_composition(&format!("{COMPOSITION_FORM_LINE}\n{reference}{shard}{shard}")).is_err(), "片入口两行、合并入口没有");
        assert!(code_keys_from_composition(&format!("{COMPOSITION_FORM_LINE}\n{reference}{shard}code_key\te7_index_bench::merge_lines_of\tnot-hex\n")).is_err(), "键不是十六进制");
    }

    #[test]
    fn the_reference_is_recorded_on_the_cpu_and_read_back_from_the_library() {
        let library = DeviceLibrary::in_memory();
        let code_keys = code_keys_for_the_unit_tests();
        let (recorded, recorded_reused) = reference_through_the_library(&library, &code_keys, 0xE163, 4);
        assert!(!recorded_reused, "库里没有：现算、写回");
        assert_eq!(recorded, reference_values_of_the_batch(0xE163, 4));
        assert_eq!(library.count_entries("items/reference-", ".out"), 1);
        let (read_back, reused) = reference_through_the_library(&library, &code_keys, 0xE163, 4);
        assert!(reused, "键相同：读回、不重算");
        assert_eq!(read_back, recorded);
        let item = reference_item_of(&code_keys, 0xE163, 4);
        assert_ne!(item, reference_item_of(&code_keys, 0xE164, 4), "种子进键");
        assert_ne!(item, reference_item_of(&code_keys, 0xE163, 5), "单元数进键");
        assert_ne!(item, reference_item_of(&CodeKeys { reference: "a2".to_string(), ..code_keys_for_the_unit_tests() }, 0xE163, 4), "参照入口的代码键进键");
    }

    /// 合并的输入：整批参照原样当一片（全对得上），抽算三单元照生成器造。
    fn merge_inputs_for_the_unit_tests(seed: u64, reference: &[u32]) -> (Vec<ShardResultRecord>, Vec<(u64, Vec<u8>)>) {
        let shard = reference.iter().enumerate().map(|(index, crc)| ShardResultRecord { global_unit_identifier: index as u64, crc: *crc }).collect();
        let sampled = sampled_unit_indices(reference.len() as u64).into_iter().map(|index| (index, generate_unit_bytes(seed, index))).collect();
        (shard, sampled)
    }

    #[test]
    fn merge_compares_against_the_reference_read_back_from_the_library() {
        let library = DeviceLibrary::in_memory();
        let code_keys = code_keys_for_the_unit_tests();
        let (seed, units) = (0xE163_0927_0000_0003_u64, 4_u64);
        let (reference, _) = reference_through_the_library(&library, &code_keys, seed, units);
        let (shard, sampled) = merge_inputs_for_the_unit_tests(seed, &reference);
        // 库里的参照改掉单元 1：合并若在 GPU 结果之后现算参照就看不出，从库里读回的才报单元 1 不一致
        let mut altered = reference.clone();
        altered[1] ^= 1;
        library.write(&reference_item_of(&code_keys, seed, units), &reference_item_text(seed, &altered));
        let (lines, reused) = merge_through_the_library(&library, &code_keys, seed, units, units, &[shard.as_slice()], &sampled);
        assert!(!reused, "合并的结论库里还没有：现判");
        assert!(lines.contains(&"MERGE mismatched_count=1 mismatched_global_unit_identifiers=[1]".to_string()), "比的是库里读回的参照：{lines:?}");
        assert_eq!(library.count_entries("items/merge-", ".out"), 1, "结论写回库");
    }

    #[test]
    fn a_merge_verdict_in_the_library_is_reused_and_changed_shard_records_are_judged_again() {
        let library = DeviceLibrary::in_memory();
        let code_keys = code_keys_for_the_unit_tests();
        let (seed, units) = (0xE163_0927_0000_0004_u64, 4_u64);
        let (reference, _) = reference_through_the_library(&library, &code_keys, seed, units);
        let (shard, sampled) = merge_inputs_for_the_unit_tests(seed, &reference);
        let (first, first_reused) = merge_through_the_library(&library, &code_keys, seed, units, units, &[shard.as_slice()], &sampled);
        assert!(!first_reused);
        assert!(first.contains(&"MERGE matched_count=4".to_string()), "{first:?}");
        // 结论入库之后改掉库里的参照：键相同的合并照读回的结论打，不再读参照
        let mut altered = reference.clone();
        altered[0] ^= 1;
        library.write(&reference_item_of(&code_keys, seed, units), &reference_item_text(seed, &altered));
        let (second, second_reused) = merge_through_the_library(&library, &code_keys, seed, units, units, &[shard.as_slice()], &sampled);
        assert!(second_reused, "键相同：读回结论");
        assert_eq!(second, first);
        // 片的记录变了：键跟着变、现判
        let mut changed_shard = shard.clone();
        changed_shard[3].crc ^= 1;
        let (third, third_reused) = merge_through_the_library(&library, &code_keys, seed, units, units, &[changed_shard.as_slice()], &sampled);
        assert!(!third_reused, "片的记录进键");
        assert_ne!(third, first);
        assert_eq!(library.count_entries("items/merge-", ".out"), 2);
    }

    /// 单测的一片：两条记录、一行适配器、两段计时。
    fn shard_on_the_card_for_the_unit_tests() -> ShardOnTheCard {
        ShardOnTheCard {
            anchors_gpu_match: true,
            sentinel_and_gid_ok: true,
            records: vec![ShardResultRecord { global_unit_identifier: 10, crc: 0x1111_1111 }, ShardResultRecord { global_unit_identifier: 11, crc: 0x2222_2222 }],
            adapter_lines: vec!["name=\"fake adapter\"".to_string()],
            build_device_milliseconds: 1.5,
            transfer_compute_readback_milliseconds: 2.25,
        }
    }

    #[test]
    fn a_shard_item_reads_back_what_was_written() {
        let shard = shard_on_the_card_for_the_unit_tests();
        let text = shard_item_text(10, 2, &shard);
        assert_eq!(shard_from_item_text(&text, 10, 2), Some(shard));
        assert_eq!(shard_from_item_text(&text, 12, 2), None, "起点对不上不认");
        assert_eq!(shard_from_item_text(&text.replace("records 2", "records 3"), 10, 2), None, "条数对不上不认");
    }

    #[test]
    fn a_shard_in_the_library_does_not_take_a_card() {
        let library = DeviceLibrary::in_memory();
        let cards_taken = std::cell::Cell::new(0_u32);
        let (first, first_reused) = shard_through_the_library(&library, "items/shard-unit-test.out", 10, 2, || {
            cards_taken.set(cards_taken.get() + 1);
            shard_on_the_card_for_the_unit_tests()
        });
        assert!(!first_reused);
        let (second, second_reused) = shard_through_the_library(&library, "items/shard-unit-test.out", 10, 2, || panic!("库里有这一片就不该取卡"));
        assert!(second_reused);
        assert_eq!(second, first);
        assert_eq!(cards_taken.get(), 1);
    }

    #[test]
    fn the_shard_key_follows_the_upload_copy_the_drops_and_the_code_key() {
        let code_keys = code_keys_for_the_unit_tests();
        let (anchor_slots, anchor_lengths) = anchor_slots_of(&anchor_vectors());
        let inputs_of = |unit_slots: Vec<u8>| ShardInputs { anchor_slots: anchor_slots.clone(), anchor_lengths, unit_slots, lengths: vec![UNIT_BYTE_LENGTH as u32] };
        let original = generate_unit_bytes(0xE163_0927_0000_0005, 0);
        let mut flipped = original.clone();
        apply_bit_flip(&mut flipped, 4099, 1);
        let item = shard_item_of(&code_keys, 0, 1, &inputs_of(original.clone()), &[]);
        assert_eq!(item, shard_item_of(&code_keys, 0, 1, &inputs_of(original.clone()), &[]), "同样的输入同一把键");
        assert_ne!(item, shard_item_of(&code_keys, 0, 1, &inputs_of(flipped), &[]), "翻位进了上传副本，键要变");
        assert_ne!(item, shard_item_of(&code_keys, 0, 1, &inputs_of(original.clone()), &[0]), "丢的单元号进键");
        assert_ne!(item, shard_item_of(&CodeKeys { shard: "b3".to_string(), ..code_keys_for_the_unit_tests() }, 0, 1, &inputs_of(original), &[]), "片入口的代码键进键");
    }
}
