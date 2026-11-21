//! 崩溃放量的 GPU 核对（里程碑二「增补 4」第四项：数据从 KV 按批读出来交给 GPU 核对，写回，门禁一次读违例）。
//! 这一版 GPU 接单元级那一截：一批单元的头校验和与载荷 CRC 在 GPU 上算（CRC-32C，参数与 D18（块里携带什么信息） 已定项 17 相同），
//! CPU 参照（`check_unit_checksums_on_the_cpu`）是另一趟、由对拍用例逐个比，两边的输入都经库读回
//! （`gpu_test_start::cpu_reference_table_read_back_from_the_pairing_library`）；走树、恢复、记录核对器照旧在 CPU 上。
//! 着色器与多卡枚举取自 E163（GPU多卡算单元校验和） 的装置（那边两卡 2560/2560 与 CPU 逐个相同）。
//! 分工按显卡空闲显存（`nvidia-smi` 的 memory.free，按 PCI 总线号对上 wgpu 的适配器）；不走卡池的取卡入口照门槛判卡
//! （[`MINIMUM_MEMORY_BUDGET_MEBIBYTES`]，空闲加回这个进程自己占在卡上的再判，[`card_memory_for_a_judge`]），被这个进程自己占着的卡等它还，
//! 被别的进程占着的卡不用；GPU 用例共用的卡池（[`gpu_test_start::GpuCardPool`]）不看这个门槛，能不能借由这个进程的显存账在借的那一刻判
//! （`singlefs_verification::gpu_memory_ledger` 文件头：只记这个进程借的，借不到不等）。
//! 每个设备起的时候装上 wgpu 没接住的错误与设备丢失的回调（[`DeviceConditionOfACard`]）：别的线程上报的 Out of Memory
//! （判器内核在另起的线程上编）记在设备上，接 Out of Memory 的那一层照它认出是显存不够，
//! 按记的那条线程上正在做的那段活认（别的线程上一次 Out of Memory 不把这段活真正的 panic 说成显存不够；
//! 编判器内核另起的线程算在那一刻在这个设备上编内核的那段活上，[`GpuCard::pipelines_compiled_once_on_this_device`]）。
//!
//! 起设备与编管线各打一行 GPU 阶段现场（标准错误，行头同别的 `GPU_LIFECYCLE` 行：`GPU_LIFECYCLE unix_microseconds=… elapsed_microseconds=…
//! pid=<进程号> thread=… test=… phase=<阶段> <细节>`，[`gpu_work_trace`]），只供事后数一个进程起了几台设备、编了几套管线，不进判定、不改结局：
//! - 每起一台设备（卡池的与不走卡池的都算，起没起好都打）一行
//!   `phase=device_start card=<nvidia-smi 序号> pci="<总线号>" owner=<the-card-pool|outside-the-card-pool> outcome=<started|out-of-gpu-memory|device-not-started> start_microseconds=<起这一台用了多久>`
//!   （[`DEVICE_START_LIFECYCLE_PHASE`]）；
//! - 一个设备上按源码摘要缓存的那组管线没编过、这一次真编了（缓存没命中）一行
//!   `phase=pipeline_compile card=<序号> pci="<总线号>" source_digest=<源码摘要的十六进制> pipelines=<编出几条> compile_microseconds=<编用了多久>`
//!   （[`PIPELINE_COMPILE_LIFECYCLE_PHASE`]）；命中缓存不打。

pub use crate::gpu_memory_ledger::DeviceConditionOfACard;
use crate::gpu_memory_ledger::{
    card_memory_reader_of_one_report_per_reading, card_memory_readings_of_one_nvidia_smi_report,
    pci_bus_identifier_without_the_domain, BusesWithDevicesStartedOutsideTheBooks,
    CardMemoryReader, CardMemoryReading, CauseOfTheDeviceLoss, DeviceOutsideTheBooksMarkedOnTheBus,
    DeviceStartedOnTheBooks, DeviceWaitBeforeReturningTheLease, OccupiedByThisProcess,
    NVIDIA_SMI_MEMORY_AND_PROCESSES_REPORT_ARGUMENTS,
};
use crate::sha256::DIGEST_BYTES;
use singlefs_checker::{
    crc32_castagnoli_table, unit_checksum_layout, UNIT_HEADER_CHECKSUM_BYTE_OFFSET,
};

mod card_memory_for_the_judges;
mod discovering_and_starting_the_cards;
#[cfg(test)]
mod family_sources_for_the_scans;
/// GPU 对拍用例开跑前取卡，只给测试用（`tests/` 下的 GPU 对拍与库内要卡的计时用例都经它）。
/// 本机没有 NVIDIA 卡时打一行 `NOT_RUN_NO_GPU_ADAPTER test=<用例名> reason=<一句>`、用例当场返回成功（门禁按这一行逐条报「本次未跑」，
/// `.claude/rules/test-script-discipline.md`「分快慢两档」：要特定硬件的那几条在没有那种硬件的机器上报「本次未跑」）；有卡而用不上判红
/// （同一篇「用上现有的硬件加速」：有那种硬件而用不上要报错，不悄悄退回）。
/// 「没卡」看两处：Vulkan 枚举里有没有 NVIDIA 独显，PCI 上有没有 NVIDIA 显示控制器——Vulkan 驱动坏了枚举不出来时，PCI 上照样看得见那张卡
/// （C620：GPU 对拍认「没卡」只看 Vulkan 枚举，驱动坏了被当成本次未跑）。看不见 PCI 总线（[`gpu_test_start::PCI_BUS_DIRECTORY`] 整个不在）
/// 也算没卡；总线看得见而设备表读不出照旧判红（代码三方 test-script-align-code-r2 判决 J2 G1）。判定是纯函数 [`gpu_test_start::decide_gpu_test_start`]，
/// 读 Vulkan 与 PCI 在取卡那一层（[`gpu_test_start::usable_cards_or_report_not_run`]）。
pub mod gpu_test_start;
mod unit_checksums;
mod work_on_a_card_and_its_readback;

pub use card_memory_for_the_judges::*;
pub use discovering_and_starting_the_cards::*;
pub use unit_checksums::*;
pub use work_on_a_card_and_its_readback::*;

/// 卡池的借据丢掉时、崩溃放量在这一趟自己起的卡上判完还显存排队的占位之前，最多等设备多久（E174 跑前登记
/// `research/prompts/e174-preregistration.md` 第五节「等待」那一臂：`PollType::Wait { submission_index: None, timeout: Some(30 秒) }`；
/// E174 两段里这一臂每次都等完了，`wait_always_returns=true`）。两处等的是同一件事（丢掉的缓冲要等设备收拾才交回驱动），只留这一处定义。
pub(crate) const LONGEST_WAIT_FOR_THE_DEVICE_BEFORE_RETURNING_A_LEASE_OR_A_PLACE_IN_THE_MEMORY_QUEUE:
    std::time::Duration = std::time::Duration::from_secs(30);

/// 借据丢掉时那一次等（`polled` 是 `poll(Wait)` 的结局，wgpu panic 了是外层的 `Err`）算没算等完：队列空了或等到了才算；
/// 超时、wgpu 报错、panic 都算没等成。`Wait` 不交 [`wgpu::PollStatus::Poll`]（那是 `PollType::Poll` 的结局），交了照没等成算。
fn device_wait_before_returning_a_lease(
    polled: &std::thread::Result<Result<wgpu::PollStatus, wgpu::PollError>>,
) -> DeviceWaitBeforeReturningTheLease {
    match polled {
        Ok(Ok(wgpu::PollStatus::QueueEmpty | wgpu::PollStatus::WaitSucceeded)) => {
            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished
        }
        Ok(Ok(wgpu::PollStatus::Poll) | Err(_)) | Err(_) => {
            DeviceWaitBeforeReturningTheLease::TheWaitFailed
        }
    }
}

/// 卡池的借据丢掉时那一次等：等这张卡的设备上到此刻为止提交的活都做完（`submission_index: None`，E174 跑前登记第五节「等待」那一臂），
/// 最多等 [`LONGEST_WAIT_FOR_THE_DEVICE_BEFORE_RETURNING_A_LEASE_OR_A_PLACE_IN_THE_MEMORY_QUEUE`]（不设上限会让丢借据的那条线程卡在一台出了事的设备上，设 0 等于不等）。
fn poll_type_before_returning_a_lease() -> wgpu::PollType {
    wgpu::PollType::Wait {
        submission_index: None,
        timeout: Some(
            LONGEST_WAIT_FOR_THE_DEVICE_BEFORE_RETURNING_A_LEASE_OR_A_PLACE_IN_THE_MEMORY_QUEUE,
        ),
    }
}

/// 卡池借之前对一台设备 poll 一次（`PollType::Poll`，不等，wgpu panic 了是外层的 `Err`）交回的算不算那台设备上提交的活都做完了：
/// 交队列空了才算（wgpu-core 30 `Device::maintain` 先收完做完的提交、丢掉的缓冲交回驱动，再看队列空没空）；还有在飞的、报错、panic 都不算。
fn device_found_idle_by_a_poll(
    polled: &std::thread::Result<Result<wgpu::PollStatus, wgpu::PollError>>,
) -> bool {
    match polled {
        Ok(Ok(wgpu::PollStatus::QueueEmpty)) => true,
        Ok(Ok(wgpu::PollStatus::WaitSucceeded | wgpu::PollStatus::Poll) | Err(_)) | Err(_) => false,
    }
}

/// 按源码摘要编好的东西：每个摘要一格，第一个要它的线程编、同时要它的等那一次编完（[`compiled_once_for_the_source_digest`]）。
type CompiledOnceBySourceDigest<Compiled> = std::sync::Mutex<
    std::collections::BTreeMap<[u8; DIGEST_BYTES], std::sync::Arc<std::sync::OnceLock<Compiled>>>,
>;

/// 一个设备上按源码摘要编好的几组管线（[`GpuCard::pipelines_compiled_once_on_this_device`]）。
type CompiledPipelinesBySourceDigest = CompiledOnceBySourceDigest<Vec<wgpu::ComputePipeline>>;

/// 一张起好上下文的卡。克隆出来的与原来的共用同一个设备、队列与编好的管线（wgpu 的这几样克隆的是句柄）：
/// 一个进程里几条 GPU 用例共用一张卡就靠克隆（[`gpu_test_start::GpuCardPool`]），每张卡只起一个设备。
#[derive(Clone)]
pub struct GpuCard {
    pub name: String,
    /// 驱动的名字与版本（wgpu adapter info 的 `driver` 与 `driver_info`，空格接起来）：崩溃放量 GPU 结论的块属性照它写，不进版本
    /// （第 11 条：两台驱动不同不分版本，靠对照与小输入对拍抓差）。
    pub driver_version: String,
    pub pci_bus_identifier: String,
    /// `nvidia-smi` 给这张卡的序号（配置按它点名）。
    pub index: u32,
    /// 起上下文之前 `nvidia-smi` 报的空闲显存（MiB；卡池里的卡是起池时找卡那一刻报的，只当说明，借卡不看它）。
    /// 卡池借出去的卡改成借的量加驱动那一份：账判过卡上放得下这么多，不让找卡那一刻的读数把草稿区压得比借的小。
    pub free_memory_mebibytes: u64,
    /// 这张卡空着时能用的显存（MiB，`nvidia-smi` 报的总量减驱动保留的那一份，与 `free_memory_mebibytes` 出自同一份报告）：不走卡池的取卡入口
    /// （[`gpu_cards_by_priority`]）起卡时读；卡池里的卡不读（能借多少照账），是 `None`。
    pub usable_memory_of_the_empty_card_mebibytes: Option<u64>,
    /// 配置给这张卡的显存额度（MiB）：崩溃放量在这张卡上占的显存不超过它；没给额度的是 `None`。
    /// 卡池借出去的卡改成借的量加驱动那一份。
    pub memory_quota_mebibytes: Option<u64>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
    /// 这个设备上编好的管线，几个克隆共用：同一套源码在同一个设备上只编一次。
    compiled_pipelines_by_source_digest: std::sync::Arc<CompiledPipelinesBySourceDigest>,
    /// 这个设备上 wgpu 报过的 Out of Memory 与设备丢没丢，几个克隆共用（[`start_the_card`] 装的回调往里记）。
    device_condition: std::sync::Arc<DeviceConditionOfACard>,
    /// 卡池起的设备在账上的那一台（[`gpu_test_start::GpuCardPool`] 起好时放进来，不走卡池的是 `None`），几个克隆共用：最后一个克隆丢掉时
    /// 这一台放掉（`singlefs_verification::gpu_memory_ledger::DeviceStartedOnTheBooks`）。放在最后一个字段：字段照声明的次序丢，
    /// 这一份最后丢的时候，同一个克隆里的设备、队列与编好的管线已经先丢了。
    device_on_the_books: Option<std::sync::Arc<DeviceStartedOnTheBooks<'static>>>,
    /// 不走卡池的设备在它那条总线上的那一记（[`start_the_card`] 起好时放进来，卡池的是 `None`），几个克隆共用：最后一个克隆丢掉时那条总线上减一台
    /// （`singlefs_verification::gpu_memory_ledger::DeviceOutsideTheBooksMarkedOnTheBus`），减到 0 卡池读那张卡回到按账拆。
    /// 与 `device_on_the_books` 同理放在设备、队列与管线之后。
    device_outside_the_books: Option<std::sync::Arc<DeviceOutsideTheBooksMarkedOnTheBus<'static>>>,
}

/// 这一个设备归谁：卡池的设备走这个进程的显存账；别的（[`gpu_cards_by_priority`] 起的）不走，
/// 起之前就记下它在哪张卡上（[`has_devices_started_outside_the_card_pool_on_the_bus`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DeviceOwnerOnTheCard {
    TheCardPool,
    OutsideTheCardPool,
}

impl DeviceOwnerOnTheCard {
    /// 起设备那一行 GPU 阶段现场（[`DEVICE_START_LIFECYCLE_PHASE`]）里 `owner=` 的值。
    fn name_in_the_lifecycle_line(self) -> &'static str {
        match self {
            Self::TheCardPool => "the-card-pool",
            Self::OutsideTheCardPool => "outside-the-card-pool",
        }
    }
}

/// `nvidia-smi` 报的一张卡：序号与空闲显存（MiB）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ReportedCard {
    index: u32,
    free_memory_mebibytes: u64,
}

/// Vulkan 下的一张 NVIDIA 独显与 `nvidia-smi` 报的它（序号、那一刻的空闲显存）：还没起设备。
/// 卡池先找到点名的卡、在账上登记了这个进程，才在卡上起设备（[`gpu_test_start::GpuCardPool`]）；别的取卡入口找到就起。
#[derive(Clone)]
struct DiscoveredCard {
    adapter: wgpu::Adapter,
    name: String,
    driver_version: String,
    pci_bus_identifier: String,
    reported: ReportedCard,
}

/// GPU 阶段现场：直接写 stderr，绕过测试框架的输出捕获；卡住时也能看到最后进入的阶段。
/// 写日志失败不影响工作，尤其不能让析构期间发生第二次 panic。
pub(crate) fn gpu_lifecycle_trace(card: &GpuCard, phase: &str, detail: std::fmt::Arguments<'_>) {
    gpu_work_trace(
        phase,
        format_args!(
            "card={} pci={:?} {detail}",
            card.index, card.pci_bus_identifier
        ),
    );
}

/// 借卡前的数据库与任务阶段也使用同一时钟；那时尚无显卡序号。
pub(crate) fn gpu_work_trace(phase: &str, detail: std::fmt::Arguments<'_>) {
    use std::io::Write;
    static ORIGIN: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    let elapsed_microseconds = ORIGIN
        .get_or_init(std::time::Instant::now)
        .elapsed()
        .as_micros();
    let unix_microseconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros();
    let _ = writeln!(
        std::io::stderr().lock(),
        "{}",
        gpu_lifecycle_line(
            unix_microseconds,
            elapsed_microseconds,
            &std::thread::current(),
            phase,
            detail
        ),
    );
}

/// GPU 阶段现场那一行：`GPU_LIFECYCLE unix_microseconds=<自 1970 年起的微秒> elapsed_microseconds=<自这个进程头一次打这一行起的微秒>
/// pid=<进程号> thread=<线程号> test=<线程名，即测试名> phase=<阶段> <细节>`（键名写全称，门禁 code-source-discipline 的 data-names 格判）。
fn gpu_lifecycle_line(
    unix_microseconds: u128,
    elapsed_microseconds: u128,
    thread: &std::thread::Thread,
    phase: &str,
    detail: std::fmt::Arguments<'_>,
) -> String {
    format!(
        "GPU_LIFECYCLE unix_microseconds={unix_microseconds} elapsed_microseconds={elapsed_microseconds} pid={} thread={:?} test={:?} phase={phase} {detail}",
        std::process::id(),
        thread.id(),
        thread.name(),
    )
}

/// 按源码摘要只编一次（[`GpuCard::pipelines_compiled_once_on_this_device`] 的缓存那一半，不碰 wgpu）：`compiled_by_source_digest` 里
/// `source_digest` 那一格没编过，这条线程调 `compile` 编、编好调 `note_the_compile`（编出来的与编用了几微秒），同时要同一个摘要的等它编完；
/// 编过的直接交那一份，`compile` 与 `note_the_compile` 都不调。
///
/// # Panics
/// `compile` panic：原样传出，这一格照旧空着，下一个要这个摘要的再编一次。
fn compiled_once_for_the_source_digest<Compiled: Clone>(
    compiled_by_source_digest: &CompiledOnceBySourceDigest<Compiled>,
    source_digest: [u8; DIGEST_BYTES],
    compile: impl FnOnce() -> Compiled,
    note_the_compile: impl FnOnce(&Compiled, u128),
) -> Compiled {
    // 持锁只为取出这个摘要那一格（不在锁里编），没有会 panic 到一半的写：别的线程 panic 留下的毒不影响这张表，照用
    let compiled_once = {
        let mut by_source_digest = compiled_by_source_digest
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        std::sync::Arc::clone(by_source_digest.entry(source_digest).or_default())
    };
    compiled_once
        .get_or_init(|| {
            let compile_began = std::time::Instant::now();
            let compiled = compile();
            note_the_compile(&compiled, compile_began.elapsed().as_micros());
            compiled
        })
        .clone()
}

#[cfg(test)]
mod tests {
    use super::family_sources_for_the_scans::{
        source_in_the_family_table, source_of_the_family, RoleInTheFamily,
        GPU_UNIT_CHECKS_FAMILY_FILES, ROOT_OF_THE_FAMILY,
    };
    use super::*;

    /// GPU 阶段现场那一行的两个时刻键写全称：`unix_microseconds=`、`elapsed_microseconds=` 紧跟在行头之后、各带交进来的数，
    /// 阶段与细节接在行尾。判别力（`crates/mutations.tsv`「GPU 阶段现场那一行的时刻键写回缩写」）：键名换回 `unix_us`，这条红。
    #[test]
    fn the_gpu_lifecycle_line_names_its_time_keys_in_full() {
        let line = super::gpu_lifecycle_line(
            1_793_843_585_212_043,
            12_792_699,
            &std::thread::current(),
            "queue_submit_end",
            format_args!("card=0"),
        );
        assert!(
            line.starts_with(
                "GPU_LIFECYCLE unix_microseconds=1793843585212043 elapsed_microseconds=12792699 pid="
            ),
            "{line}"
        );
        assert!(line.ends_with(" phase=queue_submit_end card=0"), "{line}");
    }

    // 扫源码的用例里，串里的花括号一律写成 \u 转义：门禁 code-source-discipline 的 mutation-tables 格配花括号时不跳字符串，
    // 串里落单的半边括号会让它把后面的用例算到别的模块下。
    /// 根这一份测试模块的头两行（扫源码的用例从这里切掉测试模块）。
    const GPU_UNIT_CHECKS_TESTS_MODULE_OPENING: &str = "#[cfg(test)]\nmod tests \u{7b}";
    /// 顶格的右括号自成一行（函数体在这里收尾）。
    const GPU_UNIT_CHECKS_TOP_LEVEL_CLOSING_BRACE_LINE: &str = "\n\u{7d}\n";

    /// 一个设备上按源码摘要缓存的管线：这个摘要头一次要时真编、记一次编（编管线那一行 GPU 阶段现场照它打），再要同一个摘要直接交那一份、
    /// 不编也不记；换一个摘要再编、再记一次。不要卡。判别力（`crates/mutations.tsv`「GPU 阶段现场 编管线不打那一行」）：编完不记，这条红。
    #[test]
    fn the_pipelines_of_a_source_digest_note_a_compile_only_when_the_cache_misses() {
        let compiled_by_source_digest: CompiledOnceBySourceDigest<u32> =
            std::sync::Mutex::new(std::collections::BTreeMap::new());
        let compiles = std::cell::Cell::new(0_usize);
        let noted_compiles = std::cell::RefCell::new(Vec::<u32>::new());
        let compiled_for = |source_digest: [u8; crate::sha256::DIGEST_BYTES],
                            compiled_value: u32| {
            compiled_once_for_the_source_digest(
                &compiled_by_source_digest,
                source_digest,
                || {
                    compiles.set(compiles.get() + 1);
                    compiled_value
                },
                |compiled, _compile_microseconds| noted_compiles.borrow_mut().push(*compiled),
            )
        };
        assert_eq!(compiled_for([1; crate::sha256::DIGEST_BYTES], 11), 11);
        assert_eq!(
            compiled_for([1; crate::sha256::DIGEST_BYTES], 12),
            11,
            "同一个摘要第二次要：交头一次编的那一份"
        );
        assert_eq!(compiled_for([2; crate::sha256::DIGEST_BYTES], 21), 21);
        assert_eq!(compiles.get(), 2, "两个摘要各编一次");
        assert_eq!(
            noted_compiles.into_inner(),
            vec![11, 21],
            "真编了才记，命中缓存不记"
        );
    }

    /// 不走卡池的取卡入口（[`super::gpu_cards_by_priority`]）起设备照不归卡池起（起之前记下总线号），卡池起设备只有一处、
    /// 照归卡池起。扫卡池这一族的源码（[`source_of_the_family`]，测试那一段之外），不要卡。
    ///
    /// 拦不住的写法，一种一条（写明）：
    /// - 入口取第一处：前面放一份挂 `#[cfg(any())]` 的干净副本、后面写真生效的另一份（照不归卡池起的那一句换成先 `let` 一个
    ///   `DeviceOwnerOnTheCard` 再交进去），读到的是编译时去掉的那一份；与正推 K4-1 G14、G16 同一类，借据丢掉时那一步、
    ///   四个借卡入口与卡池模块按词恰好一处、不挂 `#[cfg]`（[`index_of_the_one_pinned_item_without_a_conditional_attribute`]），这个入口不是。
    #[test]
    fn the_entry_outside_the_card_pool_starts_its_devices_as_outside_the_card_pool() {
        let source: &str = &source_of_the_family();
        let code = code_before_the_tests_module(source);
        let body_of = |signature: &str| {
            let start = code
                .find(signature)
                .unwrap_or_else(|| panic!("源码里有 {signature}"));
            let after = &code[start + signature.len()..];
            &after[..after
                .find(GPU_UNIT_CHECKS_TOP_LEVEL_CLOSING_BRACE_LINE)
                .expect("函数体以顶格的右括号收尾")]
        };
        let outside_the_pool = concat!(
            "start_the_card(discovered, DeviceOwnerOnTheCard::",
            "OutsideTheCardPool)"
        );
        assert_eq!(
            body_of("pub fn gpu_cards_by_priority(")
                .matches(outside_the_pool)
                .count(),
            1,
            "按优先级取卡的入口起设备照不归卡池起"
        );
        assert_eq!(
            code.matches(concat!("DeviceOwnerOnTheCard::", "TheCardPool)"))
                .count(),
            1,
            "照归卡池起设备的只有卡池那一处"
        );
    }

    /// （G2 乙）：借据丢掉时那一次等，队列空了或等到了才算等完、照常销账；超时、wgpu 报错、等的时候 panic、`Wait` 交回
    /// `Poll`（那是不等的结局）都算没等成，这一笔照旧记在账上。不要卡。
    #[test]
    fn lease_is_returned_only_when_the_wait_for_the_device_finished() {
        let finished = DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished;
        let failed = DeviceWaitBeforeReturningTheLease::TheWaitFailed;
        assert_eq!(
            device_wait_before_returning_a_lease(&Ok(Ok(wgpu::PollStatus::QueueEmpty))),
            finished
        );
        assert_eq!(
            device_wait_before_returning_a_lease(&Ok(Ok(wgpu::PollStatus::WaitSucceeded))),
            finished
        );
        assert_eq!(
            device_wait_before_returning_a_lease(&Ok(Ok(wgpu::PollStatus::Poll))),
            failed,
            "Wait 交回 Poll：没说等完"
        );
        assert_eq!(
            device_wait_before_returning_a_lease(&Ok(Err(wgpu::PollError::Timeout))),
            failed,
            "超时"
        );
        assert_eq!(
            device_wait_before_returning_a_lease(&Err(Box::new("等的时候 wgpu panic 了"))),
            failed,
            "等的时候 panic"
        );
    }

    /// （超时 30 秒改成 0 或不设都照绿）：借据丢掉时那一次等的是到此刻为止的全部提交，
    /// 最多等 E174 跑前登记的 30 秒（`research/prompts/e174-preregistration.md` 第五节「等待」那一臂）。不要卡。
    #[test]
    fn lease_waits_for_every_submission_so_far_for_at_most_the_thirty_seconds_registered_by_e174() {
        match poll_type_before_returning_a_lease() {
            wgpu::PollType::Wait {
                submission_index: None,
                timeout: Some(timeout),
            } => assert_eq!(
                timeout,
                // 钉的是 E174 登记的那个值，所以写字面量：拿常量比它自己，常量改成别的秒数也照绿
                std::time::Duration::from_secs(30),
                "最多等 E174 登记的 30 秒：不设会让丢借据的线程卡在出了事的设备上，0 等于不等"
            ),
            wgpu::PollType::Wait {
                submission_index: Some(_),
                ..
            }
            | wgpu::PollType::Wait { timeout: None, .. }
            | wgpu::PollType::Poll => {
                panic!(
                    "借据丢掉时等到此刻为止的全部提交、有上限：{:?}",
                    poll_type_before_returning_a_lease()
                )
            }
        }
    }

    /// （的出路）：卡池借之前对有没等成借据的那张卡 poll 一次，交回队列空了才算那台设备上
    /// 提交的活都做完了（销那几笔）；还有在飞的、`Wait` 才交的那一种、报错、panic 都不算。不要卡。
    #[test]
    fn poll_before_booking_takes_a_device_as_idle_only_when_its_queue_is_empty() {
        assert!(device_found_idle_by_a_poll(&Ok(Ok(
            wgpu::PollStatus::QueueEmpty
        ))));
        assert!(
            !device_found_idle_by_a_poll(&Ok(Ok(wgpu::PollStatus::Poll))),
            "还有在飞的提交"
        );
        assert!(
            !device_found_idle_by_a_poll(&Ok(Ok(wgpu::PollStatus::WaitSucceeded))),
            "Poll 不交这一种，交了也说不清全部提交做完了"
        );
        assert!(
            !device_found_idle_by_a_poll(&Ok(Err(wgpu::PollError::Timeout))),
            "报错"
        );
        assert!(
            !device_found_idle_by_a_poll(&Err(Box::new("poll 的时候 wgpu panic 了"))),
            "panic"
        );
    }

    /// （正推第 4 点：`GpuCardLease::drop` 等不等、记没记结局没有不要卡的测试）：
    /// 借据丢掉时那一步整个交给 `gpu_memory_ledger` 的 `wait_for_the_device_and_take_in_the_outcome`（等、接住 panic、记结局、展开中不再抛都在那边测），
    /// 等的是 [`poll_type_before_returning_a_lease`]、结局照 [`device_wait_before_returning_a_lease`] 判、记进这一笔借据；卡池的三个入口借之前
    /// 都先 poll 没等成的那几张卡（K1-2 的出路）。扫卡池这一族的源码（[`source_of_the_family`]，测试那一段之外），不要卡。
    /// （G14、G16）：借据丢掉时那一步（按词 `Drop for GpuCardLease`）
    /// 与三个入口（按词 `fn 名字`）各恰好一处、前面不挂 `#[cfg]`、`#[cfg_attr]`，挂 `#[cfg(any())]` 的干净副本在前、坏版本在后判红
    /// （`the_card_pool_scans_refuse_a_second_copy_or_a_conditional_attribute_on_the_lease_return_the_booking_entries_and_the_pool_module`）。
    ///
    /// 拦不住的写法，一种一条：
    /// - 坏版本的借据丢掉时那一步把类型写成路径或别名（`impl Drop for self::GpuCardLease`、`type 别名 = GpuCardLease;`），干净的那一份
    ///   不靠挂在它上面的属性去掉（包进一个展开成空的宏调用、放在函数体里）：按词只数得到干净的那一份；
    /// - 宏（过程宏、拼名字的宏）展开出来的同名方法或 `impl Drop`，干净的那一份照上一条去掉。
    #[test]
    fn the_card_lease_hands_its_return_to_the_ledger_and_every_booking_entry_polls_first() {
        assert_the_lease_return_and_every_booking_entry_are_pinned(&source_of_the_family());
    }

    /// `source`（卡池这一族拼成的那一段，或用例在它上面改过的一份）测试那一段之外，借据丢掉时那一步与卡池的三个借卡入口照钉住的那样写
    /// （判法在 `the_card_lease_hands_its_return_to_the_ledger_and_every_booking_entry_polls_first`）。
    ///
    /// # Panics
    /// 有一样没照钉住的那样写。
    fn assert_the_lease_return_and_every_booking_entry_are_pinned(source: &str) {
        let code = code_before_the_tests_module(source);
        let tokens_with_their_offsets = rust_tokens_with_their_offsets(code);
        let tokens: Vec<&str> = tokens_with_their_offsets
            .iter()
            .map(|(_offset, token)| *token)
            .collect();
        let body_until = |signature: &[&str], closing_line: &str| -> String {
            let signature_index =
                index_of_the_one_pinned_item_without_a_conditional_attribute(&tokens, signature);
            let (last_offset, last_token) =
                tokens_with_their_offsets[signature_index + signature.len() - 1];
            let after = &code[last_offset + last_token.len()..];
            after[..after
                .find(closing_line)
                .unwrap_or_else(|| panic!("{signature:?} 收得了尾"))]
                .to_string()
        };
        let drop_body = body_until(&["Drop", "for", "GpuCardLease"], "\n    \u{7d}\n");
        for expected in [
            "wait_for_the_device_and_take_in_the_outcome(",
            "&mut self.booking,",
            "&self.device_on_the_books,",
            "|| device.poll(poll_type_before_returning_a_lease()),",
            "device_wait_before_returning_a_lease,",
        ] {
            assert_eq!(
                drop_body.matches(expected).count(),
                1,
                "借据丢掉时那一步要有一处 {expected}：{drop_body}"
            );
        }
        for entry in [
            "lease_card",
            "lease_the_first_card_with_room_now",
            "card_rooms_now_and_when_empty_of",
        ] {
            // （正推六·二：那一句注释掉、包进 `if false` 照绿）：注释行不算，
            // 那一句要整行、缩进十二格（方法体顶层的语句），包进别的块里的缩进更深、同一行里另有别的不是整行
            let body = body_until(&["fn", entry], "\n        \u{7d}\n");
            let code_lines: Vec<&str> = body
                .lines()
                .filter(|line| !line.trim_start().starts_with("//"))
                .collect();
            let polled = code_lines
                .iter()
                .position(|line| *line == "            self.take_in_the_devices_found_idle();")
                .unwrap_or_else(|| {
                    panic!("{entry} 借之前先 poll 没等成的那几张卡（方法体顶层整行一句）：{code_lines:#?}")
                });
            let booked = code_lines
                .iter()
                .position(|line| line.contains("ledger"))
                .unwrap_or_else(|| panic!("{entry} 照账借：{code_lines:#?}"));
            assert!(
                polled < booked,
                "{entry} 先 poll、再照账借：{code_lines:#?}"
            );
        }
    }

    /// （K5：借之前那一步里面没有用例，丢了的设备到借到之后的重起才告诉账）：
    /// 借之前那一步整个交给 `gpu_memory_ledger` 的 `GpuMemoryLedger::take_in_the_devices_kept_before_booking`（看池里每张卡、丢了的告诉账不 poll、
    /// 列出来的 poll、只销列出来的，都在那边不要卡地测，`singlefs_verification::gpu_memory_ledger` 的库单测），卡池这一侧只接四样：那一格留着的设备
    /// （不等正在起的）、丢没丢照设备的情况、poll 一次不等且照 [`device_found_idle_by_a_poll`] 判、账上的那一台。这四样要真卡上起好的设备才走得到，
    /// 换不成不要卡的行为用例，这里扫卡池这一族的源码（测试那一段之外、去掉注释行与空白）：方法体整个等于钉住的那一句。不要卡。
    /// 只数几段文本各出现一处、闭包三个的判法，会被这几种改了行为而编得过的写法骗过：
    /// 方法体最前面 `if true { return; }`（正推 A2）、丢没丢与 poll 两个同型闭包对调（正推 A3）、整个包进 `if false { … }`（攻方 gpu-2）、
    /// 方法体里另起一个同名的 `device_found_idle_by_a_poll` 恒交真遮住外面那个（攻方 gpu-3）；整句比之后这四种都红。
    /// 整句比管不到（文本都在这个方法体之外）、由别的用例管的：
    /// - 改 [`device_found_idle_by_a_poll`] 本身：`poll_before_booking_takes_a_device_as_idle_only_when_its_queue_is_empty`；
    /// - 卡池模块里另起模块（攻方 `mod wgpu`、`mod std`）、经 `use` 改名或通配、在条目这一层展开宏，遮住方法体里的 `std`、`wgpu`；去掉 `use`
    ///   另起一个同名的 `device_found_idle_by_a_poll`：
    ///   `the_card_pool_module_shadows_neither_std_nor_wgpu_nor_the_functions_its_pinned_bodies_call`，它也只看文本；
    /// - `GpuCard::device`、`GpuCard::device_condition` 交的是哪一台、哪一份：
    ///   `card_hands_out_its_own_device_and_its_own_device_condition`，它也只看文本；
    /// - 三个入口不调这一步：`the_card_lease_hands_its_return_to_the_ledger_and_every_booking_entry_polls_first`，它也只看文本。
    ///
    /// 拦不住的写法，一种一条：
    /// - `device_on_the_books` 放进去的是哪一台（`started_card`）；
    /// - `GpuCard` 的 `device`、`device_condition` 两格里放的是哪一台、哪一份（`start_the_card` 起设备时放进去）；
    /// - `gpu_memory_ledger` 的 `DeviceStartedOnDemand::started_now` 与 `DeviceConditionOfACard::is_lost` 改了；
    /// - 在方法上面加 `#[cfg]` 让另一份同名方法编进来。
    #[test]
    fn the_card_pool_hands_its_kept_devices_to_the_ledger_before_booking() {
        let source: &str = &source_of_the_family();
        let code = code_before_the_tests_module(source);
        let signature = "fn take_in_the_devices_found_idle(&self) \u{7b}";
        assert_eq!(
            code.matches(signature).count(),
            1,
            "源码里恰好一处 {signature}"
        );
        let start = code
            .find(signature)
            .unwrap_or_else(|| panic!("源码里有 {signature}"));
        let after = &code[start + signature.len()..];
        let body = &after[..after
            .find("\n        \u{7d}\n")
            .expect("借之前那一步以缩进八格的右括号收尾")];
        let body_without_comments_and_spaces: String = body
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .flat_map(str::chars)
            .filter(|character| !character.is_whitespace())
            .collect();
        let expected_body = concat!(
            "self.ledger.take_in_the_devices_kept_before_booking(",
            "|card_position|self.cards[card_position].started.started_now(),",
            "|started:&GpuCard|started.device_condition().is_lost(),",
            "|started:&GpuCard|\u{7b}device_found_idle_by_a_poll(&std::panic::catch_unwind(",
            "std::panic::AssertUnwindSafe(||\u{7b}started.device().poll(wgpu::PollType::Poll)\u{7d}),))\u{7d},",
            "|started:&GpuCard|\u{7b}started.device_on_the_books.as_deref().expect(",
            "\"卡池起的设备起好时放进了账上的那一台（started_card）\")\u{7d},",
            ");"
        );
        assert_eq!(
            body_without_comments_and_spaces, expected_body,
            "借之前那一步整个交给账、四样各接一处，次序是那一格留着的设备、丢没丢、poll、账上的那一台，此外什么都没有"
        );
    }

    /// `text` 以 `"` 起头时，那一个字符串字面量连两头引号的长度（反斜杠连同它转义的那个字跳过）。
    fn length_of_the_quoted_literal_at_the_start(text: &str) -> usize {
        let bytes = text.as_bytes();
        let mut position = 1;
        loop {
            let byte = *bytes.get(position).expect("字符串字面量有收尾的引号");
            if byte == b'"' {
                return position + 1;
            }
            position += if byte == b'\\' { 2 } else { 1 };
        }
    }

    /// `text` 以 `'` 起头时，那一个字符字面量连两头引号的长度；是生命周期（`'static`、`'_`）交 `None`。
    fn length_of_the_character_literal_at_the_start(text: &str) -> Option<usize> {
        let after_the_quote = &text[1..];
        if after_the_quote.starts_with('\\') {
            // 转义的（`'\''`、`'\n'`、`'\u{7b}'`）：收尾的引号在反斜杠与它后面那个字之后
            let closing_quote = 3 + text[3..].find('\'').expect("转义的字符字面量有收尾的引号");
            return Some(closing_quote + 1);
        }
        let first_character = after_the_quote.chars().next().expect("引号后面还有字");
        after_the_quote[first_character.len_utf8()..]
            .starts_with('\'')
            .then(|| first_character.len_utf8() + 2)
    }

    /// `text` 是原始字符串去掉 `r`、`br`、`cr` 之后那一截（以若干个 `#` 与一个 `"` 起头）时，到收尾的 `"` 与同样多个 `#` 为止的长度。
    fn length_of_the_raw_string_at_the_start(text: &str) -> usize {
        let hashes = text.len() - text.trim_start_matches('#').len();
        let closing = format!("\"{}", "#".repeat(hashes));
        let body_start = hashes + 1;
        body_start + text[body_start..].find(&closing).expect("原始字符串有收尾") + closing.len()
    }

    /// `text` 以 `/*` 起头时，那一段块注释（可以嵌套）的长度。
    fn length_of_the_block_comment_at_the_start(text: &str) -> usize {
        let mut depth = 0_usize;
        let mut position = 0;
        loop {
            let rest = &text[position..];
            if rest.starts_with("/*") {
                depth += 1;
                position += 2;
            } else if rest.starts_with("*/") {
                depth -= 1;
                position += 2;
                if depth == 0 {
                    return position;
                }
            } else {
                position += rest.chars().next().expect("块注释有收尾").len_utf8();
            }
        }
    }

    /// 拼成关键字的原始标识符（`r#mod`、`r#use`）是名字，不是关键字：分词时这几个连着 `r#` 出词，免得被当成关键字（
    /// 正推 K4-2 L14）；别的原始标识符（`r#std`）与不带 `r#` 的同名，取它后面的名字。
    const KEYWORDS_A_RAW_IDENTIFIER_CAN_SPELL: [&str; 48] = [
        "abstract", "as", "async", "await", "become", "box", "break", "const", "continue", "do",
        "dyn", "else", "enum", "extern", "false", "final", "fn", "for", "gen", "if", "impl", "in",
        "let", "loop", "macro", "match", "mod", "move", "mut", "override", "priv", "pub", "ref",
        "return", "static", "struct", "trait", "true", "try", "type", "typeof", "unsafe",
        "unsized", "use", "virtual", "where", "while", "yield",
    ];

    /// `code` 拆成的词：标识符与关键字（`r#` 原始标识符取它后面的名字，拼成关键字的连着 `r#`）、生命周期（连着它前面的 `'`）、
    /// 数字（照标识符那样收一段）、`::`、`!=`，其余标点一个字一个词；注释（行注释、可以嵌套的块注释）、字符串与字符字面量
    /// （普通的、原始的 `r#"…"#`、`br`、`cr`，字节的、C 的）不出词。认原始字符串的写法与崩溃放量那一份数花括号
    /// （`crash_amplification.rs` 的 `lines_read_past_comments_and_literals`）相同：`r`、`br`、`cr` 后面紧跟若干个 `#` 与一个 `"`。
    /// 扫源码的用例照它找条目、`use` 引进来的名字与花括号的层数：字面量与注释里的 `mod`、`use`、花括号都不算。
    /// 非 ASCII 的字只落在注释、字面量与标识符里，切出来的词因此总在字符边界上。
    fn rust_tokens_without_comments_and_literals(code: &str) -> Vec<&str> {
        rust_tokens_with_their_offsets(code)
            .into_iter()
            .map(|(_offset, token)| token)
            .collect()
    }

    /// [`rust_tokens_without_comments_and_literals`] 的词，各连同它在 `code` 里起头的字节位置。
    fn rust_tokens_with_their_offsets(code: &str) -> Vec<(usize, &str)> {
        let is_identifier_character =
            |character: char| character.is_alphanumeric() || character == '_';
        let mut tokens = Vec::new();
        let mut position = 0;
        while position < code.len() {
            let rest = &code[position..];
            let first_character = rest.chars().next().expect("没到结尾就还有字");
            if first_character.is_whitespace() {
                position += first_character.len_utf8();
            } else if rest.starts_with("//") {
                position += rest.find('\n').unwrap_or(rest.len());
            } else if rest.starts_with("/*") {
                position += length_of_the_block_comment_at_the_start(rest);
            } else if first_character == '"' {
                position += length_of_the_quoted_literal_at_the_start(rest);
            } else if first_character == '\'' {
                match length_of_the_character_literal_at_the_start(rest) {
                    Some(literal_length) => position += literal_length,
                    None => {
                        let lifetime_length = 1 + rest[1..]
                            .find(|character: char| !is_identifier_character(character))
                            .unwrap_or(rest.len() - 1);
                        tokens.push((position, &rest[..lifetime_length]));
                        position += lifetime_length;
                    }
                }
            } else if is_identifier_character(first_character) {
                let word_length = rest
                    .find(|character: char| !is_identifier_character(character))
                    .unwrap_or(rest.len());
                let word = &rest[..word_length];
                let after_the_word = &rest[word_length..];
                if matches!(word, "r" | "br" | "cr")
                    && after_the_word.trim_start_matches('#').starts_with('"')
                {
                    position += word_length + length_of_the_raw_string_at_the_start(after_the_word);
                } else if word == "r" && after_the_word.starts_with('#') {
                    // 原始标识符 `r#名字`：拼成关键字的连着 `r#` 出词（`r#mod` 是名字，不是关键字），别的名字下一圈照标识符收
                    let name = &after_the_word[1..];
                    let name_length = name
                        .find(|character: char| !is_identifier_character(character))
                        .unwrap_or(name.len());
                    if KEYWORDS_A_RAW_IDENTIFIER_CAN_SPELL.contains(&&name[..name_length]) {
                        tokens.push((position, &rest[..word_length + 1 + name_length]));
                        position += word_length + 1 + name_length;
                    } else {
                        position += word_length + 1;
                    }
                } else if matches!(word, "b" | "c") && after_the_word.starts_with('"') {
                    position +=
                        word_length + length_of_the_quoted_literal_at_the_start(after_the_word);
                } else if word == "b" && after_the_word.starts_with('\'') {
                    position += word_length
                        + length_of_the_character_literal_at_the_start(after_the_word)
                            .expect("b 后面紧跟引号的是字节字面量");
                } else {
                    tokens.push((position, word));
                    position += word_length;
                }
            } else if rest.starts_with("::") {
                tokens.push((position, "::"));
                position += 2;
            } else if rest.starts_with("!=") {
                // 不等号整个一个词：拆成 `!`、`=` 时，条目层 `const 名: bool = 甲 != 乙;` 的 `甲 !` 会被认成调宏
                tokens.push((position, "!="));
                position += 2;
            } else {
                tokens.push((position, &rest[..first_character.len_utf8()]));
                position += first_character.len_utf8();
            }
        }
        tokens
    }

    /// `source`（卡池这一族拼成的那一段 [`source_of_the_family`]，或用例在它上面改过的一份）测试模块头两行之前的那一段。头两行只认这一种，与崩溃放量那一份
    /// （`crash_amplification.rs` 的 `text_outside_the_tests_module`）同一口径、按行逐字：
    /// - 相邻两整行逐字是 [`GPU_UNIT_CHECKS_TESTS_MODULE_OPENING`] 那两行（`#[cfg(test)]`，与 `mod tests`、一个空格、一个左花括号；行首起、行尾止，不带别的空白与注释，换行是 `\n`），在代码里（注释与字面量之外）；
    /// - 在花括号的第 0 层，也在圆括号、方括号的第 0 层；
    /// - 前一个词（空白、注释、字面量都跳过）是分号、右花括号、右方括号或文件开头。
    ///
    /// 照原文找第一处时，注释或字符串里写一份头两行就把切口挪到那里，它与真测试模块之间的代码哪一条扫源码的用例都看不见（
    /// 攻方 K4-2）；拿一个吃掉 `#[cfg(test)]` 的宏把测试模块包进一次宏调用的圆括号里，那个「测试模块」在非测试构建里照编，头两行却仍在
    /// 花括号第 0 层，宏调用的括号里头两行前一个词是 `(`，前面先放一条以分号收尾的条目也还在
    /// 圆括号里。卡池这一份按行逐字比：这两份文件是 rustfmt 出的，头两行就是这一种写法，
    /// 别的写法（`mod tests` 后面不空格就接左花括号、写在一行、`#[ cfg ( test ) ]`、多空格、缩进、行尾带空白或注释、CRLF 换行）一律判红，是故意拒。
    ///
    /// # Panics
    /// 照上面那样认的头两行不是恰好一处。
    fn code_before_the_tests_module(source: &str) -> &str {
        let tokens = rust_tokens_with_their_offsets(source);
        let mut brace_depth = 0_usize;
        let mut parenthesis_and_bracket_depth = 0_usize;
        let mut openings = Vec::new();
        for (index, (offset, token)) in tokens.iter().enumerate() {
            let token_before_the_attribute = index
                .checked_sub(1)
                .map(|index_before| tokens[index_before].1);
            if *token == "#"
                && is_the_tests_module_opening_as_two_whole_lines_at(source, *offset)
                && brace_depth == 0
                && parenthesis_and_bracket_depth == 0
                && token_before_the_attribute
                    .is_none_or(|token_before| matches!(token_before, ";" | "\u{7d}" | "]"))
            {
                openings.push(*offset);
            }
            if *token == "\u{7b}" {
                brace_depth += 1;
            } else if *token == "\u{7d}" {
                brace_depth = brace_depth.checked_sub(1).expect("右花括号不多于左花括号");
            } else if matches!(*token, "(" | "[") {
                parenthesis_and_bracket_depth += 1;
            } else if matches!(*token, ")" | "]") {
                parenthesis_and_bracket_depth = parenthesis_and_bracket_depth
                    .checked_sub(1)
                    .expect("右圆括号、右方括号不多于左的");
            }
        }
        assert_eq!(
            openings.len(),
            1,
            "代码里测试模块的头两行恰好一处（两整行逐字、括号第 0 层、前一个词是条目收尾；注释、字符串、宏调用括号里的不算），读到的字节位置：{openings:?}"
        );
        &source[..openings[0]]
    }

    /// `source` 从 `offset` 起是不是测试模块头两行那两整行：`offset` 在行首（文件开头或紧跟 `\n`），往后逐字是 [`GPU_UNIT_CHECKS_TESTS_MODULE_OPENING`]，
    /// 再往后是 `\n` 或文件结尾。
    fn is_the_tests_module_opening_as_two_whole_lines_at(source: &str, offset: usize) -> bool {
        let starts_a_line = offset == 0 || source[..offset].ends_with('\n');
        starts_a_line
            && source[offset..]
                .strip_prefix(GPU_UNIT_CHECKS_TESTS_MODULE_OPENING)
                .is_some_and(|after_the_opening| {
                    after_the_opening.is_empty() || after_the_opening.starts_with('\n')
                })
    }

    /// 卡池这一族的源码（测试那一段之外）里卡池模块（`mod card_pool`）的词，各带它在模块里第几层花括号里：从模块的左花括号之后
    /// 到与它配对的右花括号之前，模块的条目在第 0 层（左花括号算它外面那一层，右花括号算它收回到的那一层）。
    /// 卡池模块按词恰好一处、不挂 `#[cfg]`（[`index_of_the_one_pinned_item_without_a_conditional_attribute`]）：取第一处时，前面放一份挂 `#[cfg(any())]`、
    /// 只有那四条 `use` 的干净卡池模块，扫的就是它，真生效的那一份里另起 `mod wgpu` 也看不见（与正推 K4-1 G14 同形）。
    fn card_pool_module_tokens_with_their_brace_depth(code: &str) -> Vec<(usize, &str)> {
        let tokens = rust_tokens_without_comments_and_literals(code);
        let module_keyword = index_of_the_one_pinned_item_without_a_conditional_attribute(
            &tokens,
            &["mod", "card_pool"],
        );
        assert_eq!(
            tokens.get(module_keyword + 2),
            Some(&"\u{7b}"),
            "卡池模块写在这一段里（mod card_pool 后面紧跟左花括号，不是 mod card_pool; 另起文件）"
        );
        let opening = module_keyword + 3;
        let mut depth = 0_usize;
        let mut module_tokens = Vec::new();
        for token in &tokens[opening..] {
            if *token == "\u{7b}" {
                module_tokens.push((depth, *token));
                depth += 1;
            } else if *token == "\u{7d}" {
                let Some(depth_after_the_closing) = depth.checked_sub(1) else {
                    return module_tokens;
                };
                depth = depth_after_the_closing;
                module_tokens.push((depth, *token));
            } else {
                module_tokens.push((depth, *token));
            }
        }
        panic!("卡池模块有与它的左花括号配对的右花括号")
    }

    /// `source` 里花括号第 0 层按文件声明的子模块（`mod 名字;`），各带它是不是只给测试编的（声明前面紧挨着挂 `#[cfg(test)]`）。
    /// 注释与字面量里的不算；行内模块（`mod 名字` 后面跟花括号）不是另一份文件，不算。
    ///
    /// # Panics
    /// 花括号第 0 层之外有按文件声明的子模块、声明前面挂着 `#[path …]`、或挂的 `cfg` 不是紧挨着的 `#[cfg(test)]`：
    /// 文件表照 `<父模块的路径去掉 .rs>/<名字>.rs` 认文件、照 `#[cfg(test)]` 认归哪一种，别的写法认不出。
    fn submodule_files_declared_in(source: &str) -> Vec<(String, bool)> {
        let tokens = rust_tokens_without_comments_and_literals(source);
        let mut declared = Vec::new();
        let mut brace_depth = 0_usize;
        for (index, token) in tokens.iter().enumerate() {
            if *token == "\u{7b}" {
                brace_depth += 1;
            } else if *token == "\u{7d}" {
                brace_depth = brace_depth.checked_sub(1).expect("右花括号不多于左花括号");
            } else if *token == "mod" && tokens.get(index + 2) == Some(&";") {
                let name = tokens[index + 1];
                assert_eq!(
                    brace_depth, 0,
                    "按文件声明的子模块 {name} 写在花括号第 0 层"
                );
                let attributes = names_of_the_attributes_right_before(&tokens, index);
                assert!(
                    !attributes.contains(&"path"),
                    "子模块 {name} 前面挂着 #[path …]"
                );
                let is_only_for_tests = attributes.contains(&"cfg");
                assert!(
                    !is_only_for_tests
                        || tokens[..index].ends_with(&["#", "[", "cfg", "(", "test", ")", "]"]),
                    "子模块 {name} 挂的 cfg 只认紧挨着声明的 #[cfg(test)]：{attributes:?}"
                );
                declared.push((name.to_string(), is_only_for_tests));
            }
        }
        declared
    }

    /// 场景：卡池这一族的文件表（[`GPU_UNIT_CHECKS_FAMILY_FILES`]）对着从根起逐层 `mod` 声明到的文件（[`submodule_files_declared_in`]：
    /// 子模块的文件是 `<父模块的路径去掉 .rs>/<名字>.rs`；挂 `#[cfg(test)]` 的与只给测试编的模块下面的，都只给测试编）。
    /// 预期：两边逐个相同，归的那一种也相同。少一份（拆出一份子模块而文件表没跟上，扫源码的用例看不见它）、多一份、归错一份都红。不要卡。
    /// 判别力（`crates/mutations.tsv`「卡池这一族文件表 一份 tests.rs 归成编进库」）：文件表里一份 `tests.rs` 归成编进库，这条红。
    #[test]
    fn the_family_table_holds_every_file_the_module_declarations_reach_from_the_root() {
        let listed: std::collections::BTreeMap<&str, RoleInTheFamily> =
            GPU_UNIT_CHECKS_FAMILY_FILES
                .iter()
                .map(|(path, role, _source)| (*path, *role))
                .collect();
        assert_eq!(
            listed.len(),
            GPU_UNIT_CHECKS_FAMILY_FILES.len(),
            "文件表里一份文件只写一行"
        );
        let mut reached = std::collections::BTreeMap::new();
        let mut pending = vec![(
            ROOT_OF_THE_FAMILY.to_string(),
            RoleInTheFamily::CompiledIntoTheLibrary,
        )];
        while let Some((path, role)) = pending.pop() {
            let directory = path.strip_suffix(".rs").expect("族里的文件以 .rs 收尾");
            for (name, is_only_for_tests) in
                submodule_files_declared_in(source_in_the_family_table(&path))
            {
                let role_of_the_submodule = if is_only_for_tests {
                    RoleInTheFamily::CompiledOnlyForTests
                } else {
                    role
                };
                pending.push((format!("{directory}/{name}.rs"), role_of_the_submodule));
            }
            reached.insert(path, role);
        }
        let reached: std::collections::BTreeMap<&str, RoleInTheFamily> = reached
            .iter()
            .map(|(path, role)| (path.as_str(), *role))
            .collect();
        assert_eq!(
            reached, listed,
            "文件表与根起逐层 mod 声明到的文件（连同归哪一种）不一样"
        );
    }

    /// 一条 `use` 引进来的一样东西。
    enum BroughtInByAUse {
        /// 引进一个名字：`source` 是被引的那一处的全路径（`甲::乙`；花括号里的 `self` 取花括号之前那一段），
        /// `name` 是它在这里叫什么（有 `as` 的取改成的那个名字）。
        Name { source: String, name: String },
        /// 通配（`甲::乙::*`）：引进来哪些名字，扫这一个模块的源码看不见。
        Glob { source: String },
    }

    /// `outer_path` 后面接上 `segments`，用 `::` 连起来（`outer_path` 空着就只是 `segments`）。
    fn path_joined(outer_path: &str, segments: &[&str]) -> String {
        outer_path
            .split("::")
            .filter(|segment| !segment.is_empty())
            .chain(segments.iter().copied())
            .collect::<Vec<&str>>()
            .join("::")
    }

    /// `statement`（一条 `use` 从 `use` 那个词到分号的词）从 `start` 起的一棵 use 树（`甲::乙`、`甲::乙 as 丙`、`甲::self`、`甲::*`、
    /// `甲::{…}`，花括号可以套几层）引进来的东西，路径前面接着外层花括号之前那一段 `outer_path`；连同这棵树收尾之后那个词的位置
    /// （逗号、右花括号或分号）。
    fn brought_in_by_a_use_tree(
        statement: &[&str],
        start: usize,
        outer_path: &str,
    ) -> (Vec<BroughtInByAUse>, usize) {
        let mut segments: Vec<&str> = Vec::new();
        let mut position = start;
        loop {
            let token = *statement.get(position).expect("use 树在分号之前收尾");
            if token == "::" {
                position += 1;
            } else if token == "*" {
                let glob = BroughtInByAUse::Glob {
                    source: path_joined(outer_path, &segments),
                };
                return (vec![glob], position + 1);
            } else if token == "\u{7b}" {
                let group_path = path_joined(outer_path, &segments);
                let mut brought_in = Vec::new();
                position += 1;
                while statement[position] != "\u{7d}" {
                    let (brought_in_by_the_inner_tree, after_the_inner_tree) =
                        brought_in_by_a_use_tree(statement, position, &group_path);
                    brought_in.extend(brought_in_by_the_inner_tree);
                    position =
                        after_the_inner_tree + usize::from(statement[after_the_inner_tree] == ",");
                }
                return (brought_in, position + 1);
            } else {
                segments.push(token);
                position += 1;
                if statement.get(position) == Some(&"::") {
                    continue;
                }
                let is_self = token == "self";
                let source = path_joined(
                    outer_path,
                    &segments[..segments.len() - usize::from(is_self)],
                );
                let name_without_a_rename = if is_self {
                    source
                        .rsplit("::")
                        .next()
                        .expect("rsplit 至少交一段")
                        .to_string()
                } else {
                    token.to_string()
                };
                let (name, after_the_tree) = if statement.get(position) == Some(&"as") {
                    (statement[position + 1].to_string(), position + 2)
                } else {
                    (name_without_a_rename, position)
                };
                return (vec![BroughtInByAUse::Name { source, name }], after_the_tree);
            }
        }
    }

    /// 钉住那两段（借之前那一步、借据丢掉时那一步）按名字调的函数，各连同它该从哪一处 `use` 进卡池模块。
    const FUNCTIONS_THE_PINNED_BODIES_CALL_BY_NAME: [(&str, &str); 4] = [
        ("super::super", "device_found_idle_by_a_poll"),
        ("super::super", "poll_type_before_returning_a_lease"),
        ("super::super", "device_wait_before_returning_a_lease"),
        (
            "crate::gpu_memory_ledger",
            "wait_for_the_device_and_take_in_the_outcome",
        ),
    ];

    /// 钉住那两段（借之前那一步、借据丢掉时那一步）的签名，按词。
    const SIGNATURES_OF_THE_PINNED_BODIES: [&[&str]; 2] = [
        &["fn", "take_in_the_devices_found_idle"],
        &["impl", "Drop", "for", "GpuCardLease"],
    ];

    /// 卡池模块的词（`module_tokens`，各带花括号层数）里第 `index` 个词所在的作用域罩不罩得到钉住那两段：在模块这一层（第 0 层）的罩得到；
    /// 在花括号里的（函数体、块）只罩它那一对花括号里，那一对花括号里有钉住那两段的签名才罩得到。
    fn does_the_scope_of_the_token_reach_the_pinned_bodies(
        module_tokens: &[(usize, &str)],
        index: usize,
    ) -> bool {
        let Some(enclosing_depth) = module_tokens[index].0.checked_sub(1) else {
            return true;
        };
        let opening = module_tokens[..index]
            .iter()
            .rposition(|(depth, token)| *depth == enclosing_depth && *token == "\u{7b}")
            .expect("第 1 层往里的词前面有开这一层的左花括号");
        let closing = index
            + module_tokens[index..]
                .iter()
                .position(|(depth, token)| *depth == enclosing_depth && *token == "\u{7d}")
                .expect("第 1 层往里的词后面有收这一层的右花括号");
        let block: Vec<&str> = module_tokens[opening..=closing]
            .iter()
            .map(|(_depth, token)| *token)
            .collect();
        SIGNATURES_OF_THE_PINNED_BODIES.iter().any(|signature| {
            block
                .windows(signature.len())
                .any(|window| window == *signature)
        })
    }

    /// `tokens` 里第 `index` 个词（一个条目的头一个词）前面挂着的属性，各取 `#[` 之后的头一个词（`cfg`、`cfg_attr`、`allow`……），
    /// 照源码倒着；属性与条目之间的 `pub`、`pub(…)` 跳过。
    fn names_of_the_attributes_right_before<'token>(
        tokens: &[&'token str],
        index: usize,
    ) -> Vec<&'token str> {
        let mut item_start = index;
        if item_start > 0 && tokens[item_start - 1] == "pub" {
            item_start -= 1;
        } else if item_start > 1 && tokens[item_start - 1] == ")" {
            let visibility_opening = tokens[..item_start - 1]
                .iter()
                .rposition(|token| *token == "(");
            if let Some(parenthesis) = visibility_opening.filter(|parenthesis| {
                parenthesis.checked_sub(1).map(|before| tokens[before]) == Some("pub")
            }) {
                item_start = parenthesis - 1;
            }
        }
        let mut names = Vec::new();
        while item_start > 0 && tokens[item_start - 1] == "]" {
            let mut unclosed_brackets = 0_usize;
            let mut attribute_opening = None;
            for token_index in (0..item_start).rev() {
                if tokens[token_index] == "]" {
                    unclosed_brackets += 1;
                } else if tokens[token_index] == "[" {
                    unclosed_brackets -= 1;
                    if unclosed_brackets == 0 {
                        attribute_opening = Some(token_index);
                        break;
                    }
                }
            }
            let Some(bracket) = attribute_opening
                .filter(|bracket| bracket.checked_sub(1).map(|before| tokens[before]) == Some("#"))
            else {
                break;
            };
            names.extend(tokens.get(bracket + 1).copied());
            item_start = bracket - 1;
        }
        names
    }

    /// `tokens`（一段代码的词）里钉住的那一个条目，交 `signature`（按词：`["fn", 名字]`、`["mod", 名字]`、`["Drop", "for", 类型]`）
    /// 在 `tokens` 里那一处的下标。按词恰好一处，那个条目（往前到最近的 `impl`、`fn`、`mod`，再往前跳过 `const`、`async`、`unsafe`、
    /// `extern`、`safe`）前面不挂 `#[cfg]`、`#[cfg_attr]`：取第一处时，前面先放一份挂 `#[cfg(any())]` 的干净副本、后面写真生效的坏版本，
    /// 读到的是编译时去掉的那一份；只有一份而挂着 `#[cfg]` 的，编译时整个去掉。
    /// 与第二十三块 `use` 上挂 `#[cfg]` 判红是同一个判法（[`names_of_the_attributes_right_before`]）。
    ///
    /// # Panics
    /// 按词不是恰好一处，或那一处的条目挂着 `#[cfg]`、`#[cfg_attr]`。
    fn index_of_the_one_pinned_item_without_a_conditional_attribute(
        tokens: &[&str],
        signature: &[&str],
    ) -> usize {
        let places: Vec<usize> = tokens
            .windows(signature.len())
            .enumerate()
            .filter(|(_index, window)| *window == signature)
            .map(|(index, _window)| index)
            .collect();
        assert_eq!(
            places.len(),
            1,
            "钉住的 {signature:?} 按词恰好一处（挂 #[cfg(any())] 的干净副本在前、真生效的在后，取第一处就读到副本），读到的词下标：{places:?}"
        );
        let item_keyword = tokens[..=places[0]]
            .iter()
            .rposition(|token| matches!(*token, "impl" | "fn" | "mod"))
            .expect("钉住的条目由 impl、fn 或 mod 起头");
        let qualifiers = tokens[..item_keyword]
            .iter()
            .rev()
            .take_while(|token| matches!(**token, "const" | "async" | "unsafe" | "extern" | "safe"))
            .count();
        let attributes = names_of_the_attributes_right_before(tokens, item_keyword - qualifiers);
        assert!(
            !attributes
                .iter()
                .any(|attribute| matches!(*attribute, "cfg" | "cfg_attr")),
            "钉住的 {signature:?} 前面挂着 {attributes:?}：#[cfg] 让它编译时去掉"
        );
        places[0]
    }

    /// `code`（卡池这一族的源码测试那一段之外，或用例造的一段）的卡池模块里，能把钉住那两段按路径首段解析的 `std`、`wgpu` 与按名字调的
    /// 那几个函数换掉的写法，一处一句；一处没有交空。判法与名单在 `the_card_pool_module_shadows_neither_std_nor_wgpu_nor_the_functions_its_pinned_bodies_call`。
    fn ways_the_card_pool_module_could_replace_what_its_pinned_bodies_call(
        code: &str,
    ) -> Vec<String> {
        let module_tokens = card_pool_module_tokens_with_their_brace_depth(code);
        let tokens: Vec<&str> = module_tokens.iter().map(|(_depth, token)| *token).collect();
        let mut found = Vec::new();
        if tokens.contains(&"mod") {
            found.push(
                "卡池模块里另起模块：叫 std、wgpu 的直接遮住外部 crate，别的名字的经 #[macro_use] 把宏漏到这一层、mod 名字; 与 #[path] 的源码不在这一段里"
                    .to_string(),
            );
        }
        for pair in tokens.windows(2) {
            if pair == ["extern", "crate"] {
                found.push(
                    "卡池模块里写 extern crate：extern crate … as std 遮住外部 crate".to_string(),
                );
            }
        }
        for (index, pair) in module_tokens.windows(2).enumerate() {
            // 条目这一层的宏前面是上一个条目的收尾、属性的右方括号，或者模块的开头；跟在 = 后面、属性方括号里的是表达式宏
            // （`const _: () = assert!(…);`、`#[doc = include_str!(…)]`），展开不出条目
            let token_before = index
                .checked_sub(1)
                .map(|index_before| module_tokens[index_before].1);
            if pair[0].0 == 0
                && pair[1].1 == "!"
                && pair[0]
                    .1
                    .starts_with(|character: char| character.is_alphanumeric() || character == '_')
                && token_before.is_none_or(|token| matches!(token, ";" | "\u{7d}" | "]"))
            {
                found.push(format!(
                    "卡池模块的条目这一层调宏或定义宏 {}!：宏展开出来的条目也能叫 std、wgpu",
                    pair[0].1
                ));
            }
        }
        // 一条 use 后面不会紧跟 <：`impl Trait + use<'a>` 里的 use 是精确捕获；
        // 块里的 use 只在那一对花括号里有效，块里没有钉住那两段的签名就罩不到它们（攻方 K4-2 g2）
        let use_indices: Vec<usize> = tokens
            .iter()
            .enumerate()
            .filter(|(use_index, token)| {
                **token == "use"
                    && tokens.get(use_index + 1) != Some(&"<")
                    && does_the_scope_of_the_token_reach_the_pinned_bodies(
                        &module_tokens,
                        *use_index,
                    )
            })
            .map(|(use_index, _token)| use_index)
            .collect();
        let use_statement_at = |use_index: usize| -> &[&str] {
            let length = tokens[use_index..]
                .iter()
                .position(|token| *token == ";")
                .expect("use 以分号收尾")
                + 1;
            &tokens[use_index..use_index + length]
        };
        let use_statements: Vec<&[&str]> = use_indices
            .iter()
            .map(|use_index| use_statement_at(*use_index))
            .collect();
        let mut brought_in = Vec::new();
        for statement in &use_statements {
            let (brought_in_by_the_statement, end_of_the_tree) =
                brought_in_by_a_use_tree(statement, 1, "");
            assert_eq!(
                end_of_the_tree,
                statement.len() - 1,
                "use 树读到分号收尾：{statement:?}"
            );
            brought_in.extend(brought_in_by_the_statement);
        }
        for use_index in &use_indices {
            let attributes = names_of_the_attributes_right_before(&tokens, *use_index);
            let Some(conditional_attribute) = attributes
                .iter()
                .find(|attribute| matches!(**attribute, "cfg" | "cfg_attr"))
            else {
                continue;
            };
            let (brought_in_by_the_statement, _end_of_the_tree) =
                brought_in_by_a_use_tree(use_statement_at(*use_index), 1, "");
            for brought_in_by_a_use in &brought_in_by_the_statement {
                match brought_in_by_a_use {
                    BroughtInByAUse::Name { source: _, name }
                        if FUNCTIONS_THE_PINNED_BODIES_CALL_BY_NAME
                            .iter()
                            .any(|(_prefix, function)| function == name) =>
                    {
                        found.push(format!(
                            "卡池模块里挂 #[{conditional_attribute}] 的 use 引进钉住那两段按名字调的 {name}：编译时这条 use 可以被去掉，按名字调的就落到模块里另起的同名函数上"
                        ));
                    }
                    BroughtInByAUse::Name { .. } | BroughtInByAUse::Glob { .. } => {}
                }
            }
        }
        for brought_in_by_a_use in &brought_in {
            match brought_in_by_a_use {
                BroughtInByAUse::Glob { source } => found.push(format!(
                    "卡池模块里通配 use {source}::*：引进来的名字（叫 std、wgpu 的模块也算）扫这一段看不见"
                )),
                BroughtInByAUse::Name { source, name } => {
                    // use wgpu; 引的就是外部 crate 自己，遮不住它（攻方 K4-2 g1）
                    if matches!(name.as_str(), "std" | "wgpu") && source != name {
                        found.push(format!(
                            "卡池模块经 use 引进叫 {name} 的名字（引的是 {source}），遮住外部 crate"
                        ));
                    }
                }
            }
        }
        for (expected_prefix, function) in FUNCTIONS_THE_PINNED_BODIES_CALL_BY_NAME {
            let sources_brought_in_under_the_name: Vec<&str> = brought_in
                .iter()
                .filter_map(|brought_in_by_a_use| match brought_in_by_a_use {
                    BroughtInByAUse::Name { source, name } if name == function => {
                        Some(source.as_str())
                    }
                    BroughtInByAUse::Name { .. } | BroughtInByAUse::Glob { .. } => None,
                })
                .collect();
            let expected_source = format!("{expected_prefix}::{function}");
            if sources_brought_in_under_the_name != [expected_source.as_str()] {
                found.push(format!(
                    "钉住那两段按名字调的 {function} 要由一条 use 从 {expected_source} 引进卡池模块，引进来的是 {sources_brought_in_under_the_name:?}"
                ));
            }
        }
        found
    }

    /// （攻方：卡池模块里加 `mod wgpu { … }` 或 `mod std { … }` 遮住方法体里的
    /// `wgpu::PollType::Poll`、`std::panic::catch_unwind`，方法体一个字不动、`cargo check` 0 警告编过，整句比照绿）：钉住的两段（借之前那一步、
    /// 借据丢掉时那一步）里按路径首段解析的 `std`、`wgpu` 与按名字调的那几个函数，在卡池模块这一层只落到外部 crate 与 `use` 点名的那一处。
    /// 这一层能把一个名字换掉的写法，逐样拒：`use` 引进叫 `std` 或 `wgpu` 的名字（改名 `as`、花括号里的 `self`、几层花括号里的都照引进来的名字认）、
    /// `extern crate`；那几个函数各由一条 `use` 从钉住的那一处引进来（引了它，模块里再起同名的条目编译不过；去掉 `use` 另起一个、
    /// 或 `use 别处::某函数 as 那个名字` 就是这里红）。扫卡池这一族的源码（测试那一段之外，注释与字面量不算），不要卡。
    /// （第十六至十九块代码轮判决 K5，正推：以后合法的写法被判红）逐类判过：`!=` 被当成调宏是误判，改了；
    /// `use … as 别的名字`、两层花括号、`use 甲::{self, …}`、`extern "C" { … }` 换不掉钉住那两段用的名字，是误判，改了
    /// （`the_card_pool_scan_reads_an_item_level_not_equal_as_a_comparison_and_still_finds_an_item_level_macro`、
    /// `the_card_pool_scan_accepts_renames_nested_groups_and_self_that_bring_in_neither_std_nor_wgpu`、
    /// `the_card_pool_scan_finds_std_wgpu_or_a_pinned_function_brought_in_under_a_rename_a_nested_group_or_self`、
    /// `the_card_pool_scan_accepts_an_extern_block_and_still_finds_extern_crate`）。
    /// 挂 `#[cfg]`、`#[cfg_attr]` 的 `use` 引进钉住那两段调的函数判红——编译时
    /// 那条 `use` 去掉，按名字调的就落到模块里另起的同名函数上（正推 K4-1 B12、B13，
    /// `the_card_pool_scan_finds_a_pinned_function_brought_in_by_a_use_under_a_configuration`）；换不掉那几个名字的几样判绿
    /// （`the_card_pool_scan_accepts_writings_that_cannot_replace_what_the_pinned_bodies_call`）：`use wgpu;` 引的就是外部 crate 自己；
    /// 块里的 `use`（函数体里 `use core as std;`）只罩那一对花括号，里面没有钉住那两段的签名就不拒，有就照拒
    /// （`the_card_pool_scan_finds_a_rename_in_a_block_holding_a_pinned_body_and_an_item_level_macro_under_an_attribute`）；
    /// 属性与常量、静态量初始值里的表达式宏（`include_str!`、`assert!`、`concat!`）展开不出条目；`impl … + use<'a>` 的 `use` 是精确捕获；
    /// 分词认 `cr#"…"#`、`c"…"`（与崩溃放量那一份认法相同），拼成关键字的原始标识符（`r#mod`）是名字。测试模块只从代码里那一处头两行切
    /// （`the_card_pool_scans_cut_the_tests_module_at_its_opening_in_code_not_at_a_copy_in_a_comment_or_a_string`）。
    ///
    /// 故意拒的写法（换不换得掉那几个名字，扫这一个模块的源码判不出），一种一条，各写为什么
    /// （`the_card_pool_scan_refuses_globs_item_level_macros_and_inner_modules_even_when_they_name_nothing_pinned`）：
    /// - 通配 `use 甲::*`：引进来哪些名字要看 `甲` 里有什么，`甲` 里有叫 `std`、`wgpu` 的模块就遮住外部 crate；要用就逐个点名；
    /// - 条目这一层调宏或定义宏（`thread_local!` 也算）：宏名按文本作用域解析，卡池模块外面先定义的同名 `macro_rules!` 会顶替标准库那一个，
    ///   展开出来的条目也能叫 `std`、`wgpu`；要用就放进函数体里；
    /// - 另起模块（`#[cfg(test)] mod …` 也算）：叫 `std`、`wgpu` 的直接遮住外部 crate，别的名字的经 `#[macro_use]` 把里面的宏漏到这一层，
    ///   `mod 名字;` 与 `#[path]` 的源码不在这一段里；单测写在卡池所在那一份的测试模块里。函数体里的 `mod` 也拒（写明，
    ///   正推 K4-2 N33）：它只罩那个函数体、遮不住钉住那两段，是合法写法，判红是故意的，判法不分模块写在哪一层；
    /// - `extern crate`，挂 `#[cfg(any)]` 的也拒（写明，正推 K4-2）：编译时去掉的那一条遮不住什么，是合法写法，
    ///   判红是故意的，判法不看 `extern crate` 上挂的属性。
    ///
    /// 钉住的条目（卡池模块按词 `mod card_pool`、借据丢掉时那一步按词 `Drop for GpuCardLease`、三个借卡入口按词 `fn 名字`）在测试那一段之外
    /// 按词恰好一处、前面不挂 `#[cfg]`、`#[cfg_attr]`（[`index_of_the_one_pinned_item_without_a_conditional_attribute`]），下面几种合法写法因此判红，
    /// 也是故意拒，一种一条（写明第二节 K3）：
    /// - 卡池模块挂 `#[cfg(feature = …)]`（攻方 `g16`），红在「前面挂着」：判法不求 `cfg` 的谓词在跑的那一次构建里真不真，挂了就当它可能
    ///   编译时去掉、换成别处另一份；这一族在 `lib.rs` 里整个已经挂特性 `gpu`，要再挂特性就挂在整个模块那一级；
    /// - 测试那一段之外另有一个与卡池无关、也叫 `card_pool` 的模块（攻方 `g17`），红在「按词恰好一处」：判法按词数整族，不解析路径，
    ///   分不出钉住那几段的 `card_pool` 指的是哪一个；改成只在卡池所在的父模块里数，前面先放干净副本、取第一处那一类绕法就在父模块之外放回来；
    ///   另起的模块换个名字；
    /// - 卡池之外另一个类型上也有 `fn lease_card`（攻方 `g19`），红在「按词恰好一处」：同上按词数整族，调到的是哪一个 `lease_card`
    ///   要看类型推断，扫源码判不出；另一个方法换个名字；
    /// - 借据丢掉时那一步挂 `#[cfg_attr(…)]`（攻方 `g20`，如 `#[cfg_attr(feature = "trace", allow(dead_code))]`），红在「前面挂着」：
    ///   `cfg_attr` 能展开出 `cfg`（`#[cfg_attr(谓词, cfg(any()))]`），判法不拆它里面写的是什么，一律当成可能编译时去掉；要挂就直接写属性；
    /// - 唯一一份借据丢掉时那一步写成 `impl Drop for self::GpuCardLease`（正推 `G14h`），红在「按词恰好一处」：判法按词认
    ///   `Drop for GpuCardLease`，不解析路径，写成路径就数不到这一份；类型照 `use` 进来的名字直接写。
    ///
    /// 测试模块头两行换写法（`mod tests` 后面不空格就接左花括号、写在一行、`#[ cfg ( test ) ]`、多空格、缩进、行尾带空白或注释）与 CRLF 换行，切口当场判红，
    /// 也是故意拒（[`code_before_the_tests_module`]）。
    ///
    /// 拦不住的写法（扫这一个模块的源码看不见），一种一条：
    /// - 过程宏（派生宏、属性宏）在卡池模块里展开出来的条目；
    /// - crate 根上改 `std` 指什么（`#![no_std]` 加 `extern crate … as std`）；
    /// - `Cargo.toml` 里把 `wgpu` 换成另一个包（`package = …`）；
    /// - 被引的那一处自己换了：`super::super` 那一层的同名函数改了，或改成从别处 `pub use` 进来；
    /// - 过程宏（属性宏）挂在测试模块上、或把它包住，把测试模块编进非测试构建：切口（[`code_before_the_tests_module`]）只认头两行的写法与
    ///   所在的层，宏做了什么扫源码看不见。
    #[test]
    fn the_card_pool_module_shadows_neither_std_nor_wgpu_nor_the_functions_its_pinned_bodies_call()
    {
        let source: &str = &source_of_the_family();
        let code = code_before_the_tests_module(source);
        assert_eq!(
            ways_the_card_pool_module_could_replace_what_its_pinned_bodies_call(code),
            Vec::<String>::new(),
            "卡池模块这一层不换掉钉住那两段按路径首段解析的 std、wgpu 与按名字调的那几个函数"
        );
    }

    /// 用例造的卡池模块：头上照真源码那样把钉住那两段按名字调的四个函数各 `use` 进来，再接 `items`（花括号写成 \u 转义）。
    fn card_pool_module_with_the_pinned_uses_and(items: &str) -> String {
        let mut module = String::from(concat!(
            "mod card_pool \u{7b}\n",
            "    use super::super::\u{7b}\n",
            "        device_found_idle_by_a_poll, device_wait_before_returning_a_lease,\n",
            "        poll_type_before_returning_a_lease,\n",
            "    \u{7d};\n",
            "    use crate::gpu_memory_ledger::wait_for_the_device_and_take_in_the_outcome;\n",
        ));
        module.push_str(items);
        module.push_str("\n\u{7d}\n");
        module
    }

    /// （第十六至十九块代码轮判决 K5，正推 `K5_const_ne`、`K5_const_ne_ident`）：条目这一层的
    /// `const 名: bool = 甲 != 乙;` 是比较，不是调宏 `甲!`；条目这一层真调宏（`thread_local!`）照旧拒。不要卡。
    /// 条目层的宏另看前一个词（跟在 `=` 后面的不算）之后，`!=` 拆成两个词也不再被认成调宏，两道各自拦得住这一形；
    /// 分词把 `!=` 当一个词这一道另由头一句断言钉住。
    #[test]
    fn the_card_pool_scan_reads_an_item_level_not_equal_as_a_comparison_and_still_finds_an_item_level_macro(
    ) {
        assert_eq!(
            rust_tokens_without_comments_and_literals("FIRST_LIMIT!=SECOND_LIMIT"),
            ["FIRST_LIMIT", "!=", "SECOND_LIMIT"],
            "分词把不等号当一个词：拆成 ! 与 = 时 甲 ! 读起来像调宏"
        );
        let comparisons = card_pool_module_with_the_pinned_uses_and(concat!(
            "    const FIRST_LIMIT: u8 = 1;\n",
            "    const SECOND_LIMIT: u8 = 2;\n",
            "    const THE_LIMITS_DIFFER: bool = FIRST_LIMIT != SECOND_LIMIT;\n",
            "    const THE_LITERALS_DIFFER: bool = 1 != 2;\n",
            "    const THE_LIMITS_DIFFER_WITHOUT_SPACES: bool = FIRST_LIMIT!=SECOND_LIMIT;",
        ));
        assert_eq!(
            ways_the_card_pool_module_could_replace_what_its_pinned_bodies_call(&comparisons),
            Vec::<String>::new(),
            "条目这一层的 != 是比较，不算调宏：{comparisons}"
        );
        let item_level_macro = card_pool_module_with_the_pinned_uses_and(
            "    thread_local! \u{7b} static COUNT: std::cell::Cell<u8> = std::cell::Cell::new(0); \u{7d}",
        );
        assert_ne!(
            ways_the_card_pool_module_could_replace_what_its_pinned_bodies_call(&item_level_macro),
            Vec::<String>::new(),
            "条目这一层调宏照旧拒：{item_level_macro}"
        );
    }

    /// （第十六至十九块代码轮判决 K5，正推 `K5_use_as`、`K5_nested_use_braces`、`K5_pub_use_self_group`）：
    /// 改成别的名字、两层花括号、花括号里的 `self` 引进来的名字既不叫 `std`、`wgpu`，也不是钉住那两段调的函数，不拒。不要卡。
    #[test]
    fn the_card_pool_scan_accepts_renames_nested_groups_and_self_that_bring_in_neither_std_nor_wgpu(
    ) {
        let module = card_pool_module_with_the_pinned_uses_and(concat!(
            "    use std::sync::Arc as SharedArc;\n",
            "    use std::collections::\u{7b}BTreeMap, btree_map::\u{7b}Entry\u{7d}\u{7d};\n",
            "    use std::sync::\u{7b}self, Mutex\u{7d};\n",
            "    use ::std::fmt::\u{7b}Display as Shown,\u{7d};",
        ));
        assert_eq!(
            ways_the_card_pool_module_could_replace_what_its_pinned_bodies_call(&module),
            Vec::<String>::new(),
            "这几条 use 引进来的是 SharedArc、BTreeMap、Entry、sync、Mutex、Shown：{module}"
        );
    }

    /// （同上一条的反面）：改名、套在里层花括号里、花括号里的 `self` 引进叫 `std`、`wgpu` 的名字照旧拒；
    /// 钉住那两段调的函数换成 `use 别处::某函数 as 那个名字` 也拒。不要卡。
    #[test]
    fn the_card_pool_scan_finds_std_wgpu_or_a_pinned_function_brought_in_under_a_rename_a_nested_group_or_self(
    ) {
        for (form, items) in [
            (
                "改名成 std",
                "    use super::a_module_shadowing_the_crate as std;",
            ),
            (
                "里层花括号里的 wgpu",
                "    use super::\u{7b}outer::\u{7b}wgpu, other\u{7d}\u{7d};",
            ),
            (
                "花括号里的 self 引进 wgpu",
                "    use super::shadowing::wgpu::\u{7b}self\u{7d};",
            ),
            (
                "花括号里的 self 改名成 std",
                "    use super::\u{7b}shadowing::\u{7b}self as std\u{7d}\u{7d};",
            ),
        ] {
            let module = card_pool_module_with_the_pinned_uses_and(items);
            assert_ne!(
                ways_the_card_pool_module_could_replace_what_its_pinned_bodies_call(&module),
                Vec::<String>::new(),
                "{form} 拒：{module}"
            );
        }
        let pinned_function_renamed_in = card_pool_module_with_the_pinned_uses_and("")
            .replacen("device_found_idle_by_a_poll, ", "", 1)
            .replacen(
                "    use crate",
                "    use super::super::a_poll_that_always_finds_the_device_idle as device_found_idle_by_a_poll;\n    use crate",
                1,
            );
        assert!(
            !pinned_function_renamed_in.contains("device_found_idle_by_a_poll, "),
            "造的模块去掉了从钉住那一处引的 device_found_idle_by_a_poll：{pinned_function_renamed_in}"
        );
        assert_eq!(
            ways_the_card_pool_module_could_replace_what_its_pinned_bodies_call(
                &pinned_function_renamed_in
            ),
            vec![
                "钉住那两段按名字调的 device_found_idle_by_a_poll 要由一条 use 从 super::super::device_found_idle_by_a_poll 引进卡池模块，引进来的是 [\"super::super::a_poll_that_always_finds_the_device_idle\"]"
                    .to_string()
            ],
            "别处的函数改名成钉住那两段调的名字引进来，拒：{pinned_function_renamed_in}"
        );
    }

    /// （第十六至十九块代码轮判决 K5，正推 `K5_extern_c_block`）：`extern "C" { … }` 里声明的外部函数
    /// 在值的名字空间，遮不住按路径首段解析的 `std`、`wgpu`，与 `use` 进来的同名函数撞了编译不过，不拒；`extern crate` 照旧拒。不要卡。
    #[test]
    fn the_card_pool_scan_accepts_an_extern_block_and_still_finds_extern_crate() {
        let extern_block = card_pool_module_with_the_pinned_uses_and(
            "    extern \"C\" \u{7b} fn abs(input: i32) -> i32; \u{7d}",
        );
        assert_eq!(
            ways_the_card_pool_module_could_replace_what_its_pinned_bodies_call(&extern_block),
            Vec::<String>::new(),
            "extern 块不拒：{extern_block}"
        );
        let extern_crate =
            card_pool_module_with_the_pinned_uses_and("    extern crate alloc as std;");
        assert_ne!(
            ways_the_card_pool_module_could_replace_what_its_pinned_bodies_call(&extern_crate),
            Vec::<String>::new(),
            "extern crate 照旧拒：{extern_crate}"
        );
    }

    /// （第十六至十九块代码轮判决 K5，正推 `K5_use_wildcard_enum`、`K5_thread_local_macro`、
    /// `K5_cfg_test_mod_inside`）：故意拒的三类（为什么写在
    /// `the_card_pool_module_shadows_neither_std_nor_wgpu_nor_the_functions_its_pinned_bodies_call` 的名单里），
    /// 写的东西本身不叫 `std`、`wgpu` 也拒。不要卡。
    /// （合法写法判红的写明故意拒）：函数体里的 `mod`、
    /// 挂 `#[cfg(any())]` 的 `extern crate` 两样也在这里钉成拒。
    #[test]
    fn the_card_pool_scan_refuses_globs_item_level_macros_and_inner_modules_even_when_they_name_nothing_pinned(
    ) {
        for (form, items) in [
            ("通配", "    use std::cmp::Ordering::*;"),
            (
                "条目这一层调宏",
                "    thread_local! \u{7b} static COUNT: std::cell::Cell<u8> = std::cell::Cell::new(0); \u{7d}",
            ),
            (
                "另起模块",
                "    #[cfg(test)]\n    mod unit_checks_of_the_pool \u{7b} \u{7d}",
            ),
            (
                "函数体里另起模块",
                "    fn a_helper_with_an_inner_module() -> u8 \u{7b} mod inner \u{7b}\u{7d} 0 \u{7d}",
            ),
            (
                "挂 cfg 的 extern crate",
                "    #[cfg(any())]\n    extern crate alloc;",
            ),
        ] {
            let module = card_pool_module_with_the_pinned_uses_and(items);
            assert_ne!(
                ways_the_card_pool_module_could_replace_what_its_pinned_bodies_call(&module),
                Vec::<String>::new(),
                "{form} 故意拒：{module}"
            );
        }
    }

    /// 卡池模块之前写一份测试模块的头两行在块注释里（或跨行的字符串里），
    /// 扫源码的用例照旧切到真测试模块的头两行：切出来的那一段只多了写进去的那几行。不要卡。
    #[test]
    fn the_card_pool_scans_cut_the_tests_module_at_its_opening_in_code_not_at_a_copy_in_a_comment_or_a_string(
    ) {
        let source: &str = &source_of_the_family();
        assert_eq!(
            source.matches(GPU_UNIT_CHECKS_TESTS_MODULE_OPENING).count(),
            1,
            "卡池这一族的源码里测试模块的头两行原文只有代码里那一处"
        );
        let code = code_before_the_tests_module(source);
        assert!(
            source[code.len()..].starts_with(GPU_UNIT_CHECKS_TESTS_MODULE_OPENING),
            "切口在真测试模块的头两行"
        );
        let card_pool_opening = "mod card_pool \u{7b}\n";
        assert_eq!(
            code.matches(card_pool_opening).count(),
            1,
            "测试那一段之外恰好一处卡池模块"
        );
        for (form, copy_of_the_opening) in [
            (
                "块注释",
                format!("    /*\n{GPU_UNIT_CHECKS_TESTS_MODULE_OPENING}\n    */\n"),
            ),
            (
                "跨行的字符串",
                format!(
                    "    pub const A_COPY_OF_THE_OPENING: &str = \"\n{GPU_UNIT_CHECKS_TESTS_MODULE_OPENING}\n\";\n"
                ),
            ),
        ] {
            let with_a_copy = source.replacen(
                card_pool_opening,
                &format!("{copy_of_the_opening}{card_pool_opening}"),
                1,
            );
            assert_eq!(
                code_before_the_tests_module(&with_a_copy).len(),
                code.len() + copy_of_the_opening.len(),
                "卡池模块之前{form}里写一份测试模块的头两行，切口不挪"
            );
        }
    }

    /// 跑 `check`，它 panic 时交 panic 的消息，不 panic 交 `None`：扫源码的用例看判法在改过的源码上红没红、红在哪一句。
    fn panic_message_of_the_source_check(
        check: impl FnOnce() + std::panic::UnwindSafe,
    ) -> Option<String> {
        std::panic::catch_unwind(check).err().map(|payload| {
            payload
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| {
                    payload
                        .downcast_ref::<&str>()
                        .map(|message| (*message).to_string())
                })
                .unwrap_or_default()
        })
    }

    /// （K4，正推 K3-5 `W8`、`W9`，K4-1 `S14`、`S19`）：真测试模块头两行之前
    /// 写一份头两行，认不认照 [`code_before_the_tests_module`] 那几条。宏调用圆括号里的（前一个词是左括号，`S19`；前面先放一条以分号收尾的条目）、
    /// 花括号里的（前面先放一条以分号收尾的条目）、前一个词不是条目收尾的，都不认，切口照旧在真测试模块；代码里两处都认的（恰好一处的上界）、
    /// 真测试模块包进宏调用的圆括号（`S14`）、头两行换写法与 CRLF 换行（按行逐字的口径，故意拒），都红在「恰好一处」那一句。不要卡。
    /// （第二节 K2，攻方 B3：括号那一道只数圆括号照绿）：真测试模块包进方括号的
    /// 宏调用（`keep_hidden![` 起、前面先放一条以分号收尾的条目、`];` 收尾，攻方 `g09`；rustc 编得过、非测试构建里照编），也红在「恰好一处」。
    #[test]
    fn the_card_pool_scans_cut_only_at_one_opening_of_two_whole_lines_at_the_top_level_after_an_item(
    ) {
        let source: &str = &source_of_the_family();
        let code = code_before_the_tests_module(source);
        for (form, copy_of_the_opening) in [
            (
                "宏调用圆括号里的一份（S19）",
                "keep_hidden!(\n#[cfg(test)]\nmod tests \u{7b}\n    fn hidden_in_the_wrapped_copy() \u{7b}\u{7d}\n\u{7d}\n);\n",
            ),
            (
                "宏调用圆括号里、前面先放一条以分号收尾的条目的一份",
                "keep_hidden!(\nconst BEFORE_THE_COPY: u8 = 0;\n#[cfg(test)]\nmod tests \u{7b}\n\u{7d}\n);\n",
            ),
            (
                "花括号里、前面先放一条以分号收尾的条目的一份",
                "mod a_module_holding_a_copy \u{7b}\nconst BEFORE_THE_COPY: u8 = 0;\n#[cfg(test)]\nmod tests \u{7b}\n\u{7d}\n\u{7d}\n",
            ),
            (
                "前一个词不是条目收尾的一份",
                "static NOT_ENDED_BY_A_SEMICOLON: u8 = 0\n#[cfg(test)]\nmod tests \u{7b}\n\u{7d}\n",
            ),
        ] {
            let with_a_copy = source.replacen(
                GPU_UNIT_CHECKS_TESTS_MODULE_OPENING,
                &format!("{copy_of_the_opening}{GPU_UNIT_CHECKS_TESTS_MODULE_OPENING}"),
                1,
            );
            assert_eq!(
                code_before_the_tests_module(&with_a_copy).len(),
                code.len() + copy_of_the_opening.len(),
                "{form}不认，切口照旧在真测试模块"
            );
        }
        let opening_offset = source
            .find(GPU_UNIT_CHECKS_TESTS_MODULE_OPENING)
            .expect("卡池这一族的源码里有测试模块的头两行");
        let end_of_the_tests_module = opening_offset
            + source[opening_offset..]
                .find(GPU_UNIT_CHECKS_TOP_LEVEL_CLOSING_BRACE_LINE)
                .expect("测试模块在顶格的右括号行收尾")
            + GPU_UNIT_CHECKS_TOP_LEVEL_CLOSING_BRACE_LINE.len();
        let tests_wrapped_in_square_brackets_after_an_item =
            String::from(&source[..opening_offset])
                + "keep_hidden![\nconst BEFORE_THE_TESTS: u8 = 0;\n"
                + &source[opening_offset..end_of_the_tests_module]
                + "];\n"
                + &source[end_of_the_tests_module..];
        let mut refused_sources: Vec<(&str, String)> = vec![
            (
                "代码里两处都认的头两行",
                source.replacen(
                    GPU_UNIT_CHECKS_TESTS_MODULE_OPENING,
                    &format!(
                        "{GPU_UNIT_CHECKS_TESTS_MODULE_OPENING}{GPU_UNIT_CHECKS_TOP_LEVEL_CLOSING_BRACE_LINE}{GPU_UNIT_CHECKS_TESTS_MODULE_OPENING}"
                    ),
                    1,
                ),
            ),
            (
                "真测试模块包进宏调用的圆括号（S14）",
                source.replacen(
                    GPU_UNIT_CHECKS_TESTS_MODULE_OPENING,
                    &format!("keep_hidden!(\n{GPU_UNIT_CHECKS_TESTS_MODULE_OPENING}"),
                    1,
                ),
            ),
            (
                "真测试模块包进宏调用的圆括号、前面先放一条以分号收尾的条目",
                source.replacen(
                    GPU_UNIT_CHECKS_TESTS_MODULE_OPENING,
                    &(String::from("keep_hidden!(\nconst BEFORE_THE_TESTS: u8 = 0;\n")
                        + GPU_UNIT_CHECKS_TESTS_MODULE_OPENING),
                    1,
                ),
            ),
            (
                "真测试模块包进宏调用的花括号、前面先放一条以分号收尾的条目",
                source.replacen(
                    GPU_UNIT_CHECKS_TESTS_MODULE_OPENING,
                    &(String::from("keep_hidden! \u{7b}\nconst BEFORE_THE_TESTS: u8 = 0;\n")
                        + GPU_UNIT_CHECKS_TESTS_MODULE_OPENING),
                    1,
                ),
            ),
            (
                "真测试模块包进宏调用的方括号、前面先放一条以分号收尾的条目（攻方 g09）",
                tests_wrapped_in_square_brackets_after_an_item,
            ),
            ("CRLF 换行", source.replace('\n', "\r\n")),
        ];
        for (form, other_writing) in [
            ("mod 行少一个空格", "#[cfg(test)]\nmod tests\u{7b}"),
            ("写在一行", "#[cfg(test)] mod tests \u{7b}"),
            ("属性里多空格", "#[ cfg ( test ) ]\nmod tests \u{7b}"),
            ("mod 行多空格", "#[cfg(test)]\nmod  tests  \u{7b}"),
            ("头一行缩进", "    #[cfg(test)]\nmod tests \u{7b}"),
            ("头一行行尾带空白", "#[cfg(test)]  \nmod tests \u{7b}"),
            ("头一行行尾带注释", "#[cfg(test)] // 单测\nmod tests \u{7b}"),
            ("第二行行尾带注释", "#[cfg(test)]\nmod tests \u{7b} // 单测"),
        ] {
            refused_sources.push((
                form,
                source.replacen(GPU_UNIT_CHECKS_TESTS_MODULE_OPENING, other_writing, 1),
            ));
        }
        for (form, refused_source) in &refused_sources {
            let message = panic_message_of_the_source_check(|| {
                code_before_the_tests_module(refused_source);
            });
            assert!(
                message
                    .as_deref()
                    .is_some_and(|message| message.contains("测试模块的头两行恰好一处")),
                "{form}：切口红在「恰好一处」那一句，读到 {message:?}"
            );
        }
    }

    /// （G14、G16：借据丢掉时那一步、四个借卡入口取第一处，前面先放一份挂
    /// `#[cfg(any())]` 的干净副本、后面写真生效的坏版本，14 条卡池用例全绿）：借据丢掉时那一步、借卡入口、卡池模块各按词恰好一处、不挂 `#[cfg]`
    /// （[`index_of_the_one_pinned_item_without_a_conditional_attribute`]）。干净副本在前、坏版本在后（G14、G16；干净副本不挂 `#[cfg]`、包进函数体里一次展开成空的
    /// 宏调用的也算）与干净的卡池模块副本在前，红在「恰好一处」；
    /// 唯一的一份借据丢掉时那一步挂 `#[cfg(any())]`，红在「前面挂着」；只有坏版本（G15、G17），照旧红在钉住的那几句。扫卡池这一族的源码改过的一份，不要卡。
    #[test]
    fn the_card_pool_scans_refuse_a_second_copy_or_a_conditional_attribute_on_the_lease_return_the_booking_entries_and_the_pool_module(
    ) {
        let source: &str = &source_of_the_family();
        let piece_of_the_source = |opening: &str, closing: &str| -> (usize, usize) {
            let start = source
                .find(opening)
                .unwrap_or_else(|| panic!("源码里有 {opening}"));
            let end = start
                + source[start..]
                    .find(closing)
                    .unwrap_or_else(|| panic!("{opening} 收得了尾"))
                + closing.len();
            (start, end)
        };
        let replaced = |start: usize, end: usize, replacement: &str| {
            format!("{}{replacement}{}", &source[..start], &source[end..])
        };
        let (drop_start, drop_end) =
            piece_of_the_source("    impl Drop for GpuCardLease \u{7b}\n", "\n    \u{7d}\n");
        let pinned_drop = &source[drop_start..drop_end];
        let drop_doing_nothing = "    impl Drop for GpuCardLease \u{7b}\n        fn drop(&mut self) \u{7b}\u{7d}\n    \u{7d}\n";
        let (entry_start, entry_end) = piece_of_the_source(
            "        pub fn lease_the_first_card_with_room_now(",
            "\n        \u{7d}\n",
        );
        let pinned_entry = &source[entry_start..entry_end];
        let entry_without_the_poll = pinned_entry.replacen(
            "            self.take_in_the_devices_found_idle();\n",
            "",
            1,
        );
        assert_ne!(
            entry_without_the_poll, pinned_entry,
            "造的坏借卡入口少了借之前那一步"
        );
        for (form, changed_source, expected_red) in [
            (
                "干净的借据丢掉时那一步挂 cfg 在前、坏版本在后（G14）",
                replaced(
                    drop_start,
                    drop_end,
                    &format!("    #[cfg(any())]\n{pinned_drop}{drop_doing_nothing}"),
                ),
                "按词恰好一处",
            ),
            (
                "干净的借据丢掉时那一步包进函数体里一次展开成空的宏调用在前、坏版本在后",
                replaced(
                    drop_start,
                    drop_end,
                    &(String::from(
                        "    fn a_place_for_the_clean_copy() \u{7b}\n        expands_to_nothing! \u{7b}\n",
                    ) + pinned_drop
                        + "        \u{7d}\n    \u{7d}\n"
                        + drop_doing_nothing),
                ),
                "按词恰好一处",
            ),
            (
                "只有坏的借据丢掉时那一步（G15）",
                replaced(drop_start, drop_end, drop_doing_nothing),
                "借据丢掉时那一步要有一处",
            ),
            (
                "唯一的一份借据丢掉时那一步挂 cfg",
                replaced(
                    drop_start,
                    drop_end,
                    &format!("    #[cfg(any())]\n{pinned_drop}"),
                ),
                "前面挂着",
            ),
            (
                "干净的借卡入口挂 cfg 在前、坏版本在后（G16）",
                replaced(
                    entry_start,
                    entry_end,
                    &format!("        #[cfg(any())]\n{pinned_entry}{entry_without_the_poll}"),
                ),
                "按词恰好一处",
            ),
            (
                "只有坏的借卡入口（G17）",
                replaced(entry_start, entry_end, &entry_without_the_poll),
                "借之前先 poll 没等成的那几张卡",
            ),
        ] {
            let message = panic_message_of_the_source_check(|| {
                assert_the_lease_return_and_every_booking_entry_are_pinned(&changed_source);
            });
            assert!(
                message
                    .as_deref()
                    .is_some_and(|message| message.contains(expected_red)),
                "{form}：红在「{expected_red}」那一句，读到 {message:?}"
            );
        }
        let pool_opening = "mod card_pool \u{7b}\n";
        let with_a_clean_pool_copy_first = source.replacen(
            pool_opening,
            &format!(
                "#[cfg(any())]\n{}{pool_opening}",
                card_pool_module_with_the_pinned_uses_and("")
            ),
            1,
        );
        let message = panic_message_of_the_source_check(|| {
            let found = ways_the_card_pool_module_could_replace_what_its_pinned_bodies_call(
                code_before_the_tests_module(&with_a_clean_pool_copy_first),
            );
            drop(found);
        });
        assert!(
            message
                .as_deref()
                .is_some_and(|message| message.contains("按词恰好一处")),
            "干净的卡池模块挂 cfg 在前：红在「恰好一处」那一句，读到 {message:?}"
        );
    }

    /// （B12、B13：钉住那两段调的 `device_found_idle_by_a_poll` 改由一条挂
    /// `#[cfg(any())]` 的 `use` 引进、模块里另起一个同名函数，编译时那条 `use` 去掉、按名字调的是另起的那一个，扫源码照绿）：挂 `#[cfg]`、
    /// 展开成 `cfg` 的 `#[cfg_attr]`、`pub(crate)` 之前的 `#[cfg]`，都红。不要卡。
    #[test]
    fn the_card_pool_scan_finds_a_pinned_function_brought_in_by_a_use_under_a_configuration() {
        for (form, conditional_use) in [
            (
                "cfg",
                "    #[cfg(any())]\n    use super::super::device_found_idle_by_a_poll;\n",
            ),
            (
                "展开成 cfg 的 cfg_attr",
                "    #[cfg_attr(all(), cfg(any()))]\n    use super::super::device_found_idle_by_a_poll;\n",
            ),
            (
                "pub(crate) 之前的 cfg，另隔一条别的属性",
                "    #[cfg(any())]\n    #[allow(unused_imports, reason = \"造的\")]\n    pub(crate) use super::super::device_found_idle_by_a_poll;\n",
            ),
        ] {
            let module = card_pool_module_with_the_pinned_uses_and(
                "    fn device_found_idle_by_a_poll(_outcome: &u8) -> bool \u{7b} true \u{7d}",
            )
            .replacen("device_found_idle_by_a_poll, ", "", 1)
            .replacen(
                "    use crate",
                &format!("{conditional_use}    use crate"),
                1,
            );
            let found = ways_the_card_pool_module_could_replace_what_its_pinned_bodies_call(&module);
            assert!(
                found.iter().any(|finding| finding
                    .contains("的 use 引进钉住那两段按名字调的 device_found_idle_by_a_poll")),
                "挂 {form} 的 use 引进钉住那两段调的函数，红：{found:#?}\n{module}"
            );
        }
    }

    /// （g1、g2，正推 K4-2 L7–L10、L13b、L14）：换不掉钉住那两段用的名字的写法，
    /// 一样一个模块，都判绿。不要卡。
    #[test]
    fn the_card_pool_scan_accepts_writings_that_cannot_replace_what_the_pinned_bodies_call() {
        for (form, items) in [
            ("引外部 crate 自己", "    use wgpu;\n    use ::std;"),
            (
                "别的函数体里块作用域的改名",
                "    fn a_helper_with_its_own_std() -> u8 \u{7b} use core as std; std::convert::identity(0) \u{7d}",
            ),
            (
                "属性里的表达式宏",
                "    #[doc = include_str!(\"card_pool.md\")]\n    const POOL_SIZE: u8 = 2;",
            ),
            ("常量初始值里的表达式宏", "    const _: () = assert!(1 < 2);"),
            (
                "静态量初始值里的表达式宏",
                "    static JOINED_NAME: &str = concat!(\"card\", \"pool\");",
            ),
            (
                "精确捕获",
                "    fn borrowed_names<'pool>(names: &'pool [u8]) -> impl Iterator<Item = &'pool u8> + use<'pool> \u{7b} names.iter() \u{7d}",
            ),
            (
                "C 风格原始字符串里有引号与花括号",
                "    const QUOTED: &core::ffi::CStr = cr#\"say \"\u{7b}\" here\"#;\n    const PLAIN: &core::ffi::CStr = c\"say \u{7d} here\";",
            ),
            (
                "拼成关键字的原始标识符",
                "    fn r#mod() -> u8 \u{7b} 0 \u{7d}\n    fn r#use(r#extern: u8) -> u8 \u{7b} r#extern \u{7d}",
            ),
            (
                "挂 cfg 的 use 引的不是钉住那两段调的函数",
                "    #[cfg(any())]\n    use std::sync::Arc as SharedArcWhenConfigured;",
            ),
        ] {
            let module = card_pool_module_with_the_pinned_uses_and(items);
            assert_eq!(
                ways_the_card_pool_module_could_replace_what_its_pinned_bodies_call(&module),
                Vec::<String>::new(),
                "{form} 换不掉钉住那两段用的名字，判绿：{module}"
            );
        }
    }

    /// （上一条的反面）：块里的改名所在那一对花括号里有钉住那两段的签名（`impl Drop for GpuCardLease` 挪进函数体）照拒；
    /// 属性下面的条目层宏照拒。不要卡。
    #[test]
    fn the_card_pool_scan_finds_a_rename_in_a_block_holding_a_pinned_body_and_an_item_level_macro_under_an_attribute(
    ) {
        for (form, items) in [
            (
                "块里改名、同一块里有借据丢掉时那一步",
                "    fn holding_the_lease_drop() \u{7b}\n        use core as std;\n        impl Drop for GpuCardLease \u{7b} fn drop(&mut self) \u{7b}\u{7d} \u{7d}\n    \u{7d}",
            ),
            (
                "属性下面的条目层宏",
                "    #[allow(unused, reason = \"造的\")]\n    thread_local! \u{7b} static COUNT: std::cell::Cell<u8> = std::cell::Cell::new(0); \u{7d}",
            ),
        ] {
            let module = card_pool_module_with_the_pinned_uses_and(items);
            assert_ne!(
                ways_the_card_pool_module_could_replace_what_its_pinned_bodies_call(&module),
                Vec::<String>::new(),
                "{form} 拒：{module}"
            );
        }
    }

    /// （钉住的那一句里 `started.device` 交的是哪一台不在整句里，
    /// `started.device_condition()` 同理）：一张卡交出去的设备与设备的情况就是它自己那两格。要真卡上起好的设备才造得出 `GpuCard`，
    /// 换不成不要卡的行为用例，这里扫卡池这一族的源码（测试那一段之外、去掉注释行与空白）：两个方法各恰好一处，方法体整个是那一格。不要卡。
    ///
    /// 拦不住的写法，一种一条：
    /// - 这两格里放的是哪一台、哪一份（`start_the_card` 起设备时放进去，克隆共用同一台）；
    /// - 在方法上面加 `#[cfg]`，让签名写法与钉住的不同的另一份同名方法编进来。
    #[test]
    fn card_hands_out_its_own_device_and_its_own_device_condition() {
        let source: &str = &source_of_the_family();
        let code = code_before_the_tests_module(source);
        for (signature, expected_body) in [
            (
                "pub fn device(&self) -> &wgpu::Device \u{7b}",
                "&self.device",
            ),
            (
                "pub fn device_condition(&self) -> &DeviceConditionOfACard \u{7b}",
                "&self.device_condition",
            ),
        ] {
            assert_eq!(
                code.matches(signature).count(),
                1,
                "源码里恰好一处 {signature}"
            );
            let start = code
                .find(signature)
                .unwrap_or_else(|| panic!("源码里有 {signature}"));
            let after = &code[start + signature.len()..];
            let body = &after[..after
                .find("\n    \u{7d}\n")
                .expect("GpuCard 的方法以缩进四格的右括号收尾")];
            let body_without_comments_and_spaces: String = body
                .lines()
                .filter(|line| !line.trim_start().starts_with("//"))
                .flat_map(str::chars)
                .filter(|character| !character.is_whitespace())
                .collect();
            assert_eq!(
                body_without_comments_and_spaces, expected_body,
                "{signature} 交的是这张卡自己那一格"
            );
        }
    }

    /// Cargo.lock 里 wgpu-core 的版本（`name = "wgpu-core"` 下一行的 `version = "…"`）。
    fn wgpu_core_version_in_the_lock(lock: &str) -> Option<String> {
        let mut lines = lock.lines();
        lines.find(|line| line.trim() == "name = \"wgpu-core\"")?;
        let version_line = lines.next()?.trim();
        Some(
            version_line
                .strip_prefix("version = \"")?
                .strip_suffix('"')?
                .to_string(),
        )
    }

    /// cargo 缓存（`CARGO_HOME`，没设是 `~/.cargo`）的 `registry/src/*/wgpu-core-<版本>/src/device/resource.rs`；取不到是 `None`。
    fn wgpu_core_device_source_in_the_cargo_registry(version: &str) -> Option<String> {
        let cargo_home = std::env::var_os("CARGO_HOME")
            .map(std::path::PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME").map(|home| std::path::PathBuf::from(home).join(".cargo"))
            })?;
        std::fs::read_dir(cargo_home.join("registry").join("src"))
            .ok()?
            .filter_map(Result::ok)
            .map(|sources_of_one_registry| {
                sources_of_one_registry
                    .path()
                    .join(format!("wgpu-core-{version}"))
                    .join("src")
                    .join("device")
                    .join("resource.rs")
            })
            .find_map(|path| std::fs::read_to_string(path).ok())
    }

    /// wgpu-core 因 hal 的错丢设备时，交给设备丢失回调的那一句是那个 hal 错的
    /// `to_string()`（[`cause_of_the_device_loss`] 照「Out of memory」那一句比，上一条用例钉 hal 那一句）：扫 cargo 缓存里这一版 wgpu-core 的源码，
    /// `handle_hal_error` 里 `OutOfMemory` 那一支走 `self.lose(&error.to_string());`。版本取仓根 Cargo.lock；源码取不到打「本次未跑」那一行、
    /// 不报绿。不要卡。
    #[test]
    fn wgpu_core_loses_the_device_with_the_sentence_of_the_hal_error_it_failed_with() {
        const TEST_NAME: &str = "gpu_unit_checks::tests::wgpu_core_loses_the_device_with_the_sentence_of_the_hal_error_it_failed_with";
        const LOSS_WITH_THE_SENTENCE_OF_THE_HAL_ERROR: &str = "self.lose(&error.to_string());";
        let lock = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.lock"),
        )
        .expect("仓根有 Cargo.lock");
        let version =
            wgpu_core_version_in_the_lock(&lock).expect("Cargo.lock 里有 wgpu-core（wgpu 依赖它）");
        let Some(source) = wgpu_core_device_source_in_the_cargo_registry(&version) else {
            println!(
                "{}",
                crate::gpu_memory_ledger::gpu_not_run_line(
                    TEST_NAME,
                    &format!(
                        "cargo 缓存里没有 wgpu-core {version} 的 src/device/resource.rs（CARGO_HOME 或 ~/.cargo 下的 registry/src）"
                    )
                )
            );
            return;
        };
        let handle_start = source
            .find("pub fn handle_hal_error(")
            .expect("wgpu-core 的设备上有 handle_hal_error：没有就是这一版改了丢设备的路，要重看");
        let handle_body = &source[handle_start..];
        let handle_body = &handle_body[..handle_body
            .find("\n    \u{7d}\n")
            .expect("handle_hal_error 以缩进四格的右括号收尾")];
        assert!(
            handle_body.contains("hal::DeviceError::OutOfMemory"),
            "wgpu-core {version} 的 handle_hal_error 不再认 hal 的 OutOfMemory：{handle_body}"
        );
        assert_eq!(
            handle_body
                .matches(LOSS_WITH_THE_SENTENCE_OF_THE_HAL_ERROR)
                .count(),
            1,
            "wgpu-core {version} 丢设备时交给回调的不再是 hal 错的那一句：{handle_body}"
        );
    }
}
