//! GPU 卡上的显存怎么分：一个判器一次派活最多带几个状态（草稿区按这条流的状态数封顶），与这个进程里 GPU 用例共用的卡池怎么记账。
//! 单放一个模块、只用 `std`：`crash_judge_dispatch`（草稿区封顶）与 `gpu_unit_checks::gpu_test_start` 的卡池
//! 都调它；这几样不碰 wgpu、不要显卡，单测不要卡就跑得到。
//! 挑卡照卡在池里的先后（配置的优先级）找第一张放得下的，前一张放满了才用下一张
//! （「一个显卡 任务满了才去分另外一个」「一边占一点绝对不合理」）；
//! 账只拦放不下的，不限同时借几份、也不限一台机器上同时跑几个 GPU 测试进程（「开到最大的并发和gpu」
//! 「仅仅管会不会对指定的电脑造成cpu或者gpu的oom」「不压并发」）。
//!
//! 只记这个进程借的（岔路 G1「二：去掉跨进程账」）：没有账本文件、不锁文件、
//! 不判别的进程活没活，也不等。每一次借都在这个进程的锁里现读一次每张卡此刻的显存（[`CardMemoryReading`]：空闲、空卡上能用的、
//! 这个进程实际占着的），照这个进程在这张卡上已经借着的算这一笔放不放得下（[`card_room_for_this_process`]）。
//! 卡池每读一次跑一次 `nvidia-smi -q -d MEMORY,PIDS`（[`NVIDIA_SMI_MEMORY_AND_PROCESSES_REPORT_ARGUMENTS`]）：空闲与这个进程占着的
//! 出自同一份报告（[`card_memory_reader_of_one_report_per_reading`]：分两次读时，两次之间这个进程别的线程
//! 建上卡的那一笔既在空闲里又在占着的里，算两遍）。
//!
//! 口径（主 agent 定「三者取最小」照第 10t、10v 批）：设 F 是现读的空闲，B 是这个进程在这张卡上借着的，
//! D 是驱动的固定开销（每个进程在每张卡上一份，起设备时付），M 是这个进程此刻能在这张卡上占的
//! （[`memory_this_process_may_hold_mebibytes`]：F 加上这个进程实际占着的里属于它借出去的那一部分）。这一笔能借的是
//! `card_ledger_capacity_mebibytes(…, M, D) − B`，展开就是下面几项里最小的：
//! - 额度变量 `SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES` − B（设了才有这一项）；
//! - 配置给卡的额度 − D − B；没配显卡优先级时由 M 的 3/4 − D − B 代替（这几项都是这个进程在这张卡上一共借的连驱动那一份不超过额度）；
//! - F − 这个进程许出去而还没占上卡的那一份：许出去的是 B 连驱动那一份 D（还没登记时 B 是 0、D 照样要放得下），
//!   减实际占着的里属于许出去的那部分。账上只有活着的借据：借据丢掉时卡池先等这张卡的设备把提交了的活都做完、丢掉的缓冲交回驱动，
//!   再销账（[`GpuMemoryBooking::take_in_the_wait_before_returning`]岔路 G2「改用乙」，E174 在两张卡上量过：
//!   等完之后下一次读数里那笔缓冲都已不在），销了账的那一份不再记、也不从空闲里扣（「甲」那一份「销了账、卡上可能还没还的」整个拿掉）。
//!   等失败（超时或报错）的那一笔照旧记在账上，只会少借；它等的那台设备上提交的活后来做完了就销（卡池借之前列出每张卡上那一刻照旧记着的那几笔、
//!   对那张卡 poll 一次交队列空了只销列出来的那几笔，[`DeviceStartedOnTheBooks::take_in_that_the_work_of_the_listed_leases_finished`]；
//!   或同一台设备上另一笔借据等成了），那台设备记成丢了、或放掉了也销
//!   （两腿都量到等超时的那几笔累加到整张卡借不出，到设备丢了才回来；
//!   列卡之后才记成没等成的那一笔不一起销，它的活可以是 poll 之后才提交的）。
//!   卡池借之前看到池里哪张卡留着的设备丢了，当场告诉账、照重起那一套记成丢了（[`GpuMemoryLedger::take_in_the_devices_kept_before_booking`]），
//!   不等借到之后走到重起（那几笔让后面的借都放不下时走不到重起，卡空着却再也借不出）；
//!   那张卡上没有没等成的借据也一样（账不知道丢了时把丢了的设备还占着的
//!   驱动那一份当成这个进程许出去的，多给，借走之后重起时被拒、报本次未跑）。
//!   读不到实际占着的（`nvidia-smi` 没列这个进程），或这个进程在这张卡上另有不走这份账的设备（[`OccupiedByThisProcess::AlsoOutsideTheBooks`]，
//!   分不出哪一份是借据建的），借着的当一点都没建、驱动那一份也当还没占上卡（攻方乙附：当它整份在卡上，实占的 x 比 D 小时
//!   多给 D − x），只会少借。这张卡上起好过的设备丢了、账照重起记了之后，到丢了的那几台都放掉之前也这样算
//!   （[`DriverShareOnTheCard::DeviceLostNotStartedAgain`]、[`DriverShareOnTheCard::DeviceStartedAgainWhileTheLostDeviceRemains`]：
//!   丢了的设备连它上面的缓冲可能还占着卡，这个进程占着的分不出哪一份是新设备的）。账上的每台设备是一个 [`DeviceStartedOnTheBooks`]，
//!   卡池把它放进那台设备的每一个克隆：最后一个克隆丢掉（这个进程里那台设备的句柄都没了，wgpu 收掉设备、它上面的缓冲交回驱动）才算放掉
//!   （丢了那一刻活着的借据没等成时这一段出不来；按「那一刻活着的借据都还了」判，
//!   旧设备另有不在账上的句柄时会多给它占着的那一份）。
//!   不走这份账的设备在起之前就记下它在哪张卡上（[`BusesWithDevicesStartedOutsideTheBooks::start_a_device_marked_first`]
//!   攻方乙形：起好了才记，起设备到记下之间那一段读数里已经有那个设备的占用，被当成借据建上的）；每条总线记几台，
//!   那一台的句柄都丢了就减一台（[`DeviceOutsideTheBooksMarkedOnTheBus`]），减到 0 这张卡回到按账拆，不再照分不出算。
//!
//! 借的要比这张卡空着时能借的（同一个算法拿空卡上能用的当 M、B 当 0；空卡上能用的是 `nvidia-smi` 报的总量减驱动保留的那一份，
//! 本机两张空卡上逐 MiB 等于报的空闲：5090 是 32607 − 497 = 32110，5060 Ti 是 16311 − 462 = 15849
//! 「上界不用总量」）还大，等多久都放不下，调用方判红（配了 GPU 就一定用 GPU）；
//! 比此刻能借的大而空着时放得下，是借不到：不等，调用方按借不到走下一条（几张卡一起借的换后面的卡，一张都借不到报本次未跑，
//! [`scratch_booking_refusal_outcome`]、[`GPU_NOT_RUN_MARKER`]）。
//! 账不防的：两个进程同一刻读到同一份空闲、一起借超了（各自只记自己借的）；一份 `nvidia-smi -q` 报告里空闲与这个进程占着的是驱动的两次查询，
//! 本机两张卡的两次读显存相隔约 1.3 毫秒（攻方量过 1347.9 µs），这个进程别的线程恰在两次之间建上的那一笔既在空闲里又在占着的里
//! （与两进程同读一份空闲同形，归运行时兜底，不另改）。这两形由运行时兜底：内存包装
//! `scripts/hosts/run-with-memory-cap.sh` 认出内核日志里的 `NVRM … NV_ERR_NO_MEMORY` 退 245，wgpu 报 Out of Memory 时
//! 崩溃放量产品路径与卡池起设备把这一趟报本次未跑（原因写显存不够，[`GPU_MEMORY_SHORT_REASON_PREFIX`]、
//! [`reason_for_running_out_of_gpu_memory`]）。
//!
//! 先登记、后建设备（这个进程里）：一个进程在一张卡上第一次借时，这一笔连同它的驱动那一份放得下才记、才把这张卡记成登记了；
//! 调用方拿到借据之后才在这张卡上起设备（[`GpuMemoryLedger::prepare_the_driver_share_before_starting_the_device`]），
//! 起好了记一笔（[`GpuMemoryLedger::record_the_device_started_on_the_card`]），从此驱动那一份算实际占在卡上。设备丢了要重起时，驱动那一份记回
//! 许出去而还没占上卡、这个进程占着的照分不出算（上一段），此刻放不下就不起（重起不另登记驱动那一份，
//! 新设备的上下文没留地方）；被拒之后新设备起好之前，每一次要起都照此刻再判一次（正推 S6：被拒之前已借到的
//! 并发用例再来起设备时不直接放行）。
//!
//! wgpu 在卡上报的 Out of Memory 记在设备上（[`DeviceConditionOfACard`]），按记的那条线程上正在做的那段活认（[`WorkOnTheCardOfThisThread`]）：
//! 别的线程上一次 Out of Memory 不把这条线程上真正的 panic 说成显存不够；判器内核在活另起的线程上编，
//! 那几条线程上记的算在那一刻这个设备上在编内核的那几段活上（[`DeviceConditionOfACard::compile_for_the_work_of_this_thread`]）。
//! 崩溃放量取卡在带 `gpu` 特性的每个进程里都经这个进程的卡池（`singlefs_verification::crash_amplification` 的文件头第 5 条），借卡走这份账。

use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};
use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};

mod blocks_for_the_cards;
mod card_memory_queue;
mod card_memory_readings_of_the_nvidia_smi_report;
mod card_room_and_booking_decisions;
mod leases_and_devices_started_on_demand;
mod not_run_and_out_of_memory_reasons;
mod works_ending_together;

pub use blocks_for_the_cards::{
    after_a_round_of_the_cards, judge_blocks_on_one_card, next_card_to_start,
    AfterARoundOfTheCards, BlockHandedToTheStore, BlockJudgedOnACard, BlocksForTheCards,
    BlocksStoredInOrder, CardLeftTheBlocks, RoundsOfTheCards, StopOfTheCards, StopOfTheRun,
    StopRequestRememberedInThisRun,
};
pub use card_memory_queue::{
    first_card_with_room_in_the_queue, CardMemoryQueue, GpuCardsSource, PlaceInTheCardMemoryQueue,
};
pub use card_memory_readings_of_the_nvidia_smi_report::{
    card_memory_reader_of_one_report_per_reading, card_memory_readings_of_one_nvidia_smi_report,
    pci_bus_identifier_without_the_domain, NVIDIA_SMI_MEMORY_AND_PROCESSES_REPORT_ARGUMENTS,
};
pub use card_room_and_booking_decisions::{
    attempt_on_a_card_without_room_for_one_state, card_ledger_capacity_mebibytes,
    card_room_for_this_process, decide_after_attempting_every_card, decide_booking_by_priority,
    decide_scratch_before_booking, mebibytes_of_buffers_held_at_once,
    memory_this_process_may_hold_mebibytes, most_states_whose_need_fits_the_capacity,
    occupied_part_of_what_this_process_promised_mebibytes, place_states_on_the_cards_that_book_now,
    states_in_one_dispatch_capped_by_the_state_count, BookedByThisProcessOnACard, BookingDecision,
    CardRoomForThisProcess, NeedLargerThanEveryCard, ScratchAttemptOnACard, ScratchBeforeBooking,
    ScratchPlacementDecision, PASSES_ALIVE_AT_ONCE_ON_EACH_THREAD,
};
use leases_and_devices_started_on_demand::LeaseWhenTheBookingIsDropped;
pub use leases_and_devices_started_on_demand::{
    wait_for_the_device_and_take_in_the_outcome, DeviceStartedOnDemand,
    DeviceWaitBeforeReturningTheLease, GpuMemoryBooking, LeasesKeptAfterAFailedWaitWhenListed,
    StatesBookedOnACard,
};
pub use not_run_and_out_of_memory_reasons::{
    gpu_not_run_line, reason_for_a_booking_without_room_now, reason_for_running_out_of_gpu_memory,
    scratch_booking_refusal_outcome, BookingRefusal, ScratchBookingRefusalOutcome,
    GPU_MEMORY_SHORT_REASON_PREFIX, GPU_NOT_RUN_MARKER,
};
pub use works_ending_together::{
    work_caught_noting_the_device_when_it_returned, WorkEndingTogether,
    WorksEndingTogetherOnADevice,
};

/// 这一刻读到的一张卡的显存（MiB）：几个量出自同一份 `nvidia-smi` 报告（[`card_memory_readings_of_one_nvidia_smi_report`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CardMemoryReading {
    pub free_mebibytes: u64,
    /// 空卡上能用的：报的总量减驱动保留的那一份（谁都借不走；空卡上读到的空闲就是它，文件头）。
    pub total_less_reserved_mebibytes: u64,
    pub occupied_by_this_process: OccupiedByThisProcess,
}

/// 这个进程此刻实际占在一张卡上的（`nvidia-smi` 按进程号报的），连同这一份能不能按这份账拆。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OccupiedByThisProcess {
    /// 报告里没列这个进程（没在这张卡上起设备，或驱动不按进程报），或列了而占的量读不出。
    NotListed,
    /// 列了：这个进程在这张卡上的设备都走这份账（卡池起的那一个）。
    OnlyThroughTheBooks { mebibytes: u64 },
    /// 列了，而这个进程在这张卡上另有不走这份账的设备（崩溃放量没起卡池时自己起的、经入口直接起的）：
    /// 占的量里分不出哪一份是借据建的（攻方乙形）。
    AlsoOutsideTheBooks { mebibytes: u64 },
}

/// 这个进程的驱动那一份在一张卡上走到哪一步。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DriverShareOnTheCard {
    /// 这个进程还没在这张卡上借过：下一笔连驱动那一份一起要放得下。
    NotRegistered,
    /// 借过、记成登记了，设备还没起好：驱动那一份许出去了、还没占在卡上。
    RegisteredDeviceNotStarted,
    /// 设备起好了：驱动那一份占在卡上。
    DeviceStarted,
    /// 起好过的设备丢了、新设备还没起好（重起时此刻放不下被拒过，或还没起）：驱动那一份许出去、还没占在卡上；
    /// 每一次要起新设备都照此刻再判一次放不放得下（[`GpuMemoryLedger::prepare_the_driver_share_before_starting_the_device`]）。
    /// 丢了的设备连它上面的缓冲可能还占着卡，这个进程占着的分不出哪一份是谁的：照读不到实际占着的算
    /// （[`occupied_part_of_what_this_process_promised_mebibytes`] 交 0）。
    DeviceLostNotStartedAgain,
    /// 丢了的设备重起好了，而丢了的那几台还有没放掉的（[`DeviceStartedOnTheBooks`] 还有克隆在：借据的卡、卡池那一格、调用方留着的卡）：
    /// 丢了的设备连它上面的缓冲可能还占着卡，照读不到实际占着的算；都放掉了才回到 [`Self::DeviceStarted`]（G2 乙：
    /// 拿掉「销了账、卡上可能还没还的」那一份之后，旧设备留在卡上的不再记成一份改按放没放掉判，不按那一刻活着的借据还没还判）。
    DeviceStartedAgainWhileTheLostDeviceRemains,
}

/// 卡超时、设备丢失那一行的记号：变异判「抓到」照它把这种红与断言红分开。
const GPU_DEVICE_LOST_MARKER: &str = "GPU_DEVICE_LOST";

/// 那一行的 `test=` 取当前线程的名字（libtest 多线程跑时是测试名）；线程没有名字时写这个。
const THREAD_WITHOUT_A_NAME: &str = "unnamed-thread";

/// 读日志的一头按行切时当换行的字符：控制字符（`\n`、`\r`、`\x0b`、`\x0c`、`\x1c` 到 `\x1e`、`\u{85}`）与 Unicode 的行、段分隔符
/// （Python 的 `str.splitlines` 全都切）。
fn is_a_line_break(character: char) -> bool {
    character.is_control() || matches!(character, '\u{2028}' | '\u{2029}')
}

/// 去掉换行、接成一行：连着的几个换行算一处，换成一个空格（wgpu 的错误文字常带 `\n\nCaused by:` 这样的几行）。
fn text_on_one_line(text: &str) -> String {
    text.split(is_a_line_break)
        .filter(|piece| !piece.is_empty())
        .collect::<Vec<&str>>()
        .join(" ")
}

/// 卡超时、设备丢失那一行：`GPU_DEVICE_LOST test=<当前线程名> card=<卡名> reason=<一句>`，一定是一行。
/// 卡名里可以有空格（`NVIDIA GeForce RTX 3090`），`reason=` 排最后、到行尾。
fn gpu_device_lost_line(test_name: &str, card_name: &str, reason: &str) -> String {
    format!(
        "{GPU_DEVICE_LOST_MARKER} test={} card={} reason={}",
        text_on_one_line(test_name),
        text_on_one_line(card_name),
        text_on_one_line(reason)
    )
}

/// 打到标准输出：用 `println!`，与 [`GPU_NOT_RUN_MARKER`] 那一行同样进 libtest 捕获的这条用例的输出，用例红了 libtest 把它交出来。
fn print_the_device_lost_marker_line(marker_line: &str) {
    println!("{marker_line}");
}

/// 读每张卡此刻的显存（次序与卡池一样）：卡池每读一次跑一次 `nvidia-smi`（[`card_memory_reader_of_one_report_per_reading`]），单测换成假的。
pub type CardMemoryReader = Box<dyn Fn() -> Result<Vec<CardMemoryReading>, String> + Send + Sync>;

/// 一张卡上这个进程一共能占多少（MiB）：`(卡在池里的次序, 这个进程能占的)` → 容量（这个进程自己的驱动那一份已经扣掉，
/// 卡池照 [`card_ledger_capacity_mebibytes`] 算）。
pub type CardCapacityForThisProcess = Box<dyn Fn(usize, u64) -> u64 + Send + Sync>;

/// 账上一台设备在它那张卡上是第几台（这个进程在每张卡上起好的设备各从 0 数，[`GpuMemoryLedger::record_the_device_started_on_the_card`] 发号）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct DeviceNumberOnTheCard(u64);

/// 这个进程在一张卡上记着的：每笔活着的借据（借据号 → MiB）、驱动那一份走到哪一步、账上的设备（在用的那一台、丢了还没放掉的那几台），
/// 与等设备失败而照旧记着的借据。
#[derive(Clone, Debug, PartialEq, Eq)]
struct BooksOfACard {
    leases: BTreeMap<u64, u64>,
    driver_share: DriverShareOnTheCard,
    /// 下一台起好的设备记成第几台。
    next_device_number: DeviceNumberOnTheCard,
    /// 起好了、还没记成丢了、也还没放掉的那一台。
    device_in_use: Option<DeviceNumberOnTheCard>,
    /// 记成丢了（[`Self::take_back_the_driver_share_of_a_lost_device`]）、还没放掉（[`DeviceStartedOnTheBooks`] 还有克隆在）的那几台：
    /// 它们连上面的缓冲可能还占着卡，不空时这个进程占着的照分不出算（[`DriverShareOnTheCard::DeviceStartedAgainWhileTheLostDeviceRemains`]）。
    lost_devices_not_released: BTreeSet<DeviceNumberOnTheCard>,
    /// 丢掉时等设备没等成（超时或报错）、照旧记在 `leases` 里的借据（借据号 → 它等的那一台，[`GpuMemoryBooking::take_in_the_wait_before_returning`]）：
    /// 那一台上提交的活后来做完了（[`Self::return_the_leases_kept_after_a_failed_wait_on`]、
    /// [`Self::return_the_listed_leases_kept_after_a_failed_wait_on`]）、那一台记成丢了、或放掉了，销账。
    /// 只记在用的那一台上的：等的是丢了还没放掉的那一台而没等成的，照常销账（那时这个进程占着的照分不出算，放掉之前它的缓冲交回驱动）。
    leases_kept_after_a_failed_wait: BTreeMap<u64, DeviceNumberOnTheCard>,
}

impl BooksOfACard {
    /// 一张还没借过的卡的账。
    fn empty() -> Self {
        Self {
            leases: BTreeMap::new(),
            driver_share: DriverShareOnTheCard::NotRegistered,
            next_device_number: DeviceNumberOnTheCard(0),
            device_in_use: None,
            lost_devices_not_released: BTreeSet::new(),
            leases_kept_after_a_failed_wait: BTreeMap::new(),
        }
    }

    /// 这张卡上起好过的设备丢了、要重起：驱动那一份记回许出去而还没占上卡；在用的那一台记成丢了（放掉之前这个进程占着的照分不出算）；
    /// 在它上面等设备没等成而照旧记着的那几笔这时销账（它们的缓冲在丢了的那一台上，放掉之前照分不出算，不再按借据记）。
    fn take_back_the_driver_share_of_a_lost_device(&mut self) {
        self.driver_share = DriverShareOnTheCard::DeviceLostNotStartedAgain;
        if let Some(lost_device) = self.device_in_use.take() {
            self.lost_devices_not_released.insert(lost_device);
        }
        for kept_lease in std::mem::take(&mut self.leases_kept_after_a_failed_wait).into_keys() {
            self.leases.remove(&kept_lease);
        }
    }

    fn booked(&self) -> BookedByThisProcessOnACard {
        BookedByThisProcessOnACard {
            leases_mebibytes: self.leases.values().sum(),
            driver_share: self.driver_share,
        }
    }

    /// 销一笔借据（借据丢掉时）：它的缓冲已经交回驱动（等完了设备），或它拿着的是丢了还没放掉的设备（这个进程占着的照分不出算），
    /// 账上不再记它。交回销没销到。
    fn return_a_lease(&mut self, lease_identifier: u64) -> bool {
        self.leases.remove(&lease_identifier).is_some()
    }

    /// 第 `device` 台上到某一刻为止提交的活都做完了：在它上面等设备没等成而照旧记着的那几笔销账（它们丢掉的缓冲在那之前提交过、
    /// 丢掉过，wgpu 收完那几次提交就交回驱动）。
    fn return_the_leases_kept_after_a_failed_wait_on(&mut self, device: DeviceNumberOnTheCard) {
        let leases = &mut self.leases;
        self.leases_kept_after_a_failed_wait
            .retain(|kept_lease, kept_on_the_device| {
                if *kept_on_the_device == device {
                    leases.remove(kept_lease);
                    return false;
                }
                true
            });
    }

    /// 第 `device` 台上到列卡之后的某一刻为止提交的活都做完了（卡池在列卡之后 poll 交回队列空了）：`listed`（列卡那一刻这张卡上照旧记着的，
    /// 借据号 → 它等的那一台）里等的是这一台、此刻还照旧记着的那几笔销账。列卡之后才记成没等成的不销：它的活可以是 poll 之后才提交的，
    /// 缓冲可能还在卡上（攻方：poll 之后、销账之前另一笔刚记成没等成，一起销掉，账多给）。
    fn return_the_listed_leases_kept_after_a_failed_wait_on(
        &mut self,
        device: DeviceNumberOnTheCard,
        listed: &BTreeMap<u64, DeviceNumberOnTheCard>,
    ) {
        // 一笔借据照旧记着时等的是哪一台，记上之后不变：列出来时等的是这一台、此刻还照旧记着的，就是还没销的那几笔
        for (listed_lease, listed_on_the_device) in listed {
            if *listed_on_the_device == device
                && self
                    .leases_kept_after_a_failed_wait
                    .remove(listed_lease)
                    .is_some()
            {
                self.leases.remove(listed_lease);
            }
        }
    }

    /// 第 `device` 台放掉了（这个进程里它的句柄都丢了，wgpu 收掉设备、它上面的缓冲交回驱动）：在它上面没等成的那几笔销账；
    /// 它记成丢了的，不再让这张卡照分不出算，丢了的都放掉了、新设备已经起好，回到 [`DriverShareOnTheCard::DeviceStarted`]。
    fn release_a_device(&mut self, device: DeviceNumberOnTheCard) {
        self.return_the_leases_kept_after_a_failed_wait_on(device);
        self.lost_devices_not_released.remove(&device);
        if self.device_in_use == Some(device) {
            self.device_in_use = None;
        }
        if self.lost_devices_not_released.is_empty() {
            match self.driver_share {
                DriverShareOnTheCard::DeviceStartedAgainWhileTheLostDeviceRemains => {
                    self.driver_share = DriverShareOnTheCard::DeviceStarted;
                }
                DriverShareOnTheCard::NotRegistered
                | DriverShareOnTheCard::RegisteredDeviceNotStarted
                | DriverShareOnTheCard::DeviceStarted
                | DriverShareOnTheCard::DeviceLostNotStartedAgain => {}
            }
        }
    }

    /// 记一笔借据；这个进程还没在这张卡上登记的，一并登记（设备还没起）。
    fn book_a_lease(&mut self, lease_identifier: u64, mebibytes: u64) {
        let previous = self.leases.insert(lease_identifier, mebibytes);
        assert!(
            previous.is_none(),
            "借据号 {lease_identifier} 记了两次：借据号由原子计数器发，一个号只发一次"
        );
        match self.driver_share {
            DriverShareOnTheCard::NotRegistered => {
                self.driver_share = DriverShareOnTheCard::RegisteredDeviceNotStarted;
            }
            DriverShareOnTheCard::RegisteredDeviceNotStarted
            | DriverShareOnTheCard::DeviceStarted
            | DriverShareOnTheCard::DeviceLostNotStartedAgain
            | DriverShareOnTheCard::DeviceStartedAgainWhileTheLostDeviceRemains => {}
        }
    }
}

/// 这个进程里几张卡的显存账（MiB）：借的时候在锁里现读每张卡的显存、照 [`card_room_for_this_process`] 判，放得下就记上，
/// 放不下不等、交借不到；借据丢掉时销账。只记这个进程借的（文件头）。
pub struct GpuMemoryLedger {
    read_card_memory: CardMemoryReader,
    capacity_for_this_process: CardCapacityForThisProcess,
    mebibytes_each_process_keeps_on_a_card: u64,
    books: Mutex<Vec<BooksOfACard>>,
    next_lease_identifier: AtomicU64,
}

/// 记上的那一笔：在池里第几张卡、借据号、记了多少（MiB）、按状态数借的借了几个状态、记的那一刻这张卡能借给这个进程的。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BookedOnACard {
    card_position: usize,
    lease_identifier: u64,
    mebibytes_booked: u64,
    states_booked: Option<u64>,
    room_mebibytes_when_booked: u64,
}

impl GpuMemoryLedger {
    /// 开这个进程的账：`card_count` 张卡（次序就是优先级），每次借照 `read_card_memory` 现读、照 `capacity_for_this_process` 算容量，
    /// 每个进程在每张卡上付 `mebibytes_each_process_keeps_on_a_card` 的驱动开销。不读卡、不登记。
    #[must_use]
    pub fn new(
        card_count: usize,
        read_card_memory: CardMemoryReader,
        capacity_for_this_process: CardCapacityForThisProcess,
        mebibytes_each_process_keeps_on_a_card: u64,
    ) -> Self {
        Self {
            read_card_memory,
            capacity_for_this_process,
            mebibytes_each_process_keeps_on_a_card,
            books: Mutex::new((0..card_count).map(|_| BooksOfACard::empty()).collect()),
            next_lease_identifier: AtomicU64::new(0),
        }
    }

    /// 账的锁：持锁的那几段只改这几张表、不 panic 到一半（借据号重了才断言，那时账已经坏了），别的线程 panic 留下的毒照用。
    fn books_locked(&self) -> MutexGuard<'_, Vec<BooksOfACard>> {
        self.books.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// 在锁里现读每张卡（次序与池一样，`books` 是锁里的账）：读到的卡数与池对不上是读不出。
    fn readings_of_every_card(
        &self,
        books: &[BooksOfACard],
    ) -> Result<Vec<CardMemoryReading>, BookingRefusal> {
        let readings = (self.read_card_memory)()
            .map_err(|reason| BookingRefusal::CardMemoryUnreadable { reason })?;
        if readings.len() != books.len() {
            return Err(BookingRefusal::CardMemoryUnreadable {
                reason: format!(
                    "读到 {} 张卡的显存，池里是 {} 张",
                    readings.len(),
                    books.len()
                ),
            });
        }
        Ok(readings)
    }

    /// 在锁里现读每张卡（[`Self::readings_of_every_card`]），算这几张卡此刻各能借给这个进程的（与 `card_positions` 逐张对上）。
    fn rooms_of_the_cards(
        &self,
        books: &[BooksOfACard],
        card_positions: &[usize],
    ) -> Result<Vec<CardRoomForThisProcess>, BookingRefusal> {
        let readings = self.readings_of_every_card(books)?;
        Ok(card_positions
            .iter()
            .map(|card_position| {
                card_room_for_this_process(
                    readings[*card_position],
                    books[*card_position].booked(),
                    self.mebibytes_each_process_keeps_on_a_card,
                    &|may_hold_mebibytes| {
                        (self.capacity_for_this_process)(*card_position, may_hold_mebibytes)
                    },
                )
            })
            .collect())
    }

    /// 在锁里记一笔借据，交回记上的那一笔。
    fn book_in_the_lock(
        &self,
        books: &mut [BooksOfACard],
        card_position: usize,
        mebibytes: u64,
        states_booked: Option<u64>,
        room_mebibytes_when_booked: u64,
    ) -> BookedOnACard {
        let lease_identifier = self.next_lease_identifier.fetch_add(1, Ordering::Relaxed);
        books[card_position].book_a_lease(lease_identifier, mebibytes);
        BookedOnACard {
            card_position,
            lease_identifier,
            mebibytes_booked: mebibytes,
            states_booked,
            room_mebibytes_when_booked,
        }
    }

    /// 这几张卡此刻各能借给这个进程多少（MiB，现读一次，与 `card_positions` 逐张对上）。
    ///
    /// # Errors
    /// 卡上的显存读不出（[`BookingRefusal::CardMemoryUnreadable`]）。
    #[cfg(test)]
    pub fn card_rooms_mebibytes_now(
        &self,
        card_positions: &[usize],
    ) -> Result<Vec<u64>, BookingRefusal> {
        Ok(self
            .card_rooms_now_and_when_empty(card_positions)?
            .iter()
            .map(|room| room.room_now_mebibytes)
            .collect())
    }

    /// 这几张卡此刻与空着时各能借给这个进程多少（现读一次，与 `card_positions` 逐张对上）。
    ///
    /// # Errors
    /// 卡上的显存读不出（[`BookingRefusal::CardMemoryUnreadable`]）。
    pub fn card_rooms_now_and_when_empty(
        &self,
        card_positions: &[usize],
    ) -> Result<Vec<CardRoomForThisProcess>, BookingRefusal> {
        let books = self.books_locked();
        self.rooms_of_the_cards(&books, card_positions)
    }

    /// 借 `need_mebibytes`：照卡的优先级记在第一张此刻放得下的卡上（这个进程在那张卡上还没登记的，它的驱动那一份也要放得下，
    /// 一并登记），交回借据（丢掉时销账）；此刻每张都放不下就交借不到、什么都不记，不等。
    ///
    /// # Errors
    /// 见 [`BookingRefusal`]。
    ///
    /// # Panics
    /// `need_mebibytes` 是 0（借 0 MiB 照样要登记驱动那一份，这一笔说不清要什么）。
    pub fn book_on_the_first_card_with_room(
        &self,
        need_mebibytes: u64,
    ) -> Result<GpuMemoryBooking<'_>, BookingRefusal> {
        let every_card: Vec<usize> = (0..self.books_locked().len()).collect();
        self.book_on_one_of_these_cards(&every_card, need_mebibytes)
    }

    /// 借 `need_mebibytes`，只记在第 `card_position` 张卡上（卡由调用方定、不由账挑）；别的同 [`Self::book_on_the_first_card_with_room`]。
    ///
    /// # Errors
    /// 见 [`BookingRefusal`]。
    ///
    /// # Panics
    /// 同 [`Self::book_on_the_first_card_with_room`]。
    #[cfg(test)]
    pub fn book_on_the_card(
        &self,
        card_position: usize,
        need_mebibytes: u64,
    ) -> Result<GpuMemoryBooking<'_>, BookingRefusal> {
        self.book_on_one_of_these_cards(&[card_position], need_mebibytes)
    }

    /// 照 `card_positions` 的先后挑第一张此刻放得下的卡记 `need_mebibytes`（在一次锁里读、判、记）。
    fn book_on_one_of_these_cards(
        &self,
        card_positions: &[usize],
        need_mebibytes: u64,
    ) -> Result<GpuMemoryBooking<'_>, BookingRefusal> {
        assert!(
            need_mebibytes > 0,
            "借 0 MiB 照样要登记驱动那一份：调用方按实际要建的缓冲借，至少 1 MiB"
        );
        let mut books = self.books_locked();
        let rooms = self.rooms_of_the_cards(&books, card_positions)?;
        match decide_booking_by_priority(&rooms, need_mebibytes) {
            BookingDecision::BookOnTheCardAt { card_position } => {
                let booked = self.book_in_the_lock(
                    &mut books,
                    card_positions[card_position],
                    need_mebibytes,
                    None,
                    rooms[card_position].room_now_mebibytes,
                );
                drop(books);
                Ok(self.booking_of(booked))
            }
            BookingDecision::NoRoomNow => Err(BookingRefusal::NoCardHasRoomNow {
                need_mebibytes,
                card_rooms_mebibytes: rooms.iter().map(|room| room.room_now_mebibytes).collect(),
            }),
            BookingDecision::NeverFits {
                largest_ceiling_mebibytes,
            } => Err(BookingRefusal::NeedLargerThanEveryCard(
                NeedLargerThanEveryCard {
                    need_mebibytes,
                    largest_ceiling_mebibytes,
                },
            )),
        }
    }

    /// 崩溃放量产品路径第 ① 段借草稿区：按卡的先后（`card_positions`）把 `pending_states` 个状态装到此刻借得到的卡上
    /// （[`place_states_on_the_cards_that_book_now`]，每张卡最多装 `states_each_card_holds` 个，与 `card_positions` 逐张对上），
    /// 在一次锁里现读每张卡，每张照此刻能借给这个进程的压状态数再借（几个状态要多少显存照 `mebibytes_for_states`，随状态数单调不减、
    /// 一个状态至少 1 MiB）；借不到的卡不装。交回每张卡借到的（与 `card_positions` 逐张对上，没借到的是 `None`，至少一张借到了）。
    ///
    /// # Errors
    /// 一张都没借到而空着时放得下（[`BookingRefusal::NoCardHasRoomNow`]，要的是一个状态的量）；每张卡空着时连一个状态都放不下
    /// （[`BookingRefusal::NeedLargerThanEveryCard`]）；卡上的显存读不出。出错时什么都没记。
    ///
    /// # Panics
    /// 两张表的卡数不同；同一张卡给了两次；一个状态要 0 MiB。
    pub fn book_states_on_the_cards_with_room(
        &self,
        card_positions: &[usize],
        pending_states: u64,
        states_each_card_holds: &[u64],
        mebibytes_for_states: &dyn Fn(u64) -> u64,
    ) -> Result<Vec<Option<StatesBookedOnACard<'_>>>, BookingRefusal> {
        assert_eq!(
            card_positions.len(),
            states_each_card_holds.len(),
            "卡表与每张卡带得了几个状态的表要逐张对上"
        );
        assert!(
            card_positions
                .iter()
                .enumerate()
                .all(|(slot, position)| !card_positions[..slot].contains(position)),
            "同一张卡给了两次：{card_positions:?}"
        );
        let need_of_one_state_mebibytes = mebibytes_for_states(1);
        assert!(
            need_of_one_state_mebibytes > 0,
            "一个状态要 0 MiB：判器按状态数借的那一族至少有表与一个状态的草稿区"
        );
        let mut books = self.books_locked();
        let rooms = self.rooms_of_the_cards(&books, card_positions)?;
        let mut booked_on_each_card: Vec<Option<BookedOnACard>> =
            card_positions.iter().map(|_| None).collect();
        let attempts = place_states_on_the_cards_that_book_now(
            pending_states,
            states_each_card_holds,
            &mut |card_slot, states_to_place| {
                let room = rooms[card_slot];
                let states_that_fit = most_states_whose_need_fits_the_capacity(
                    usize::try_from(states_to_place).unwrap_or(usize::MAX),
                    room.room_now_mebibytes,
                    &|states| {
                        mebibytes_for_states(u64::try_from(states).expect("状态数装得进 u64"))
                    },
                );
                let states =
                    u64::try_from(states_that_fit).expect("压出来的不多于要装的，装得进 u64");
                if states > 0 {
                    booked_on_each_card[card_slot] = Some(self.book_in_the_lock(
                        &mut books,
                        card_positions[card_slot],
                        mebibytes_for_states(states),
                        Some(states),
                        room.room_now_mebibytes,
                    ));
                    return ScratchAttemptOnACard::Booked { states };
                }
                attempt_on_a_card_without_room_for_one_state(need_of_one_state_mebibytes, room)
            },
            &mut |card_slot| {
                attempt_on_a_card_without_room_for_one_state(
                    need_of_one_state_mebibytes,
                    rooms[card_slot],
                )
            },
        );
        drop(books);
        match decide_after_attempting_every_card(&attempts) {
            ScratchPlacementDecision::JudgeOnTheCardsBooked => Ok(booked_on_each_card
                .into_iter()
                .map(|booked| {
                    booked.map(|booked| StatesBookedOnACard {
                        states: booked
                            .states_booked
                            .expect("按状态数借的那一笔记上就带着借了几个状态"),
                        booking: self.booking_of(booked),
                    })
                })
                .collect()),
            ScratchPlacementDecision::NoRoomOnAnyCardNow { need_mebibytes } => {
                Err(BookingRefusal::NoCardHasRoomNow {
                    need_mebibytes,
                    card_rooms_mebibytes: rooms
                        .iter()
                        .map(|room| room.room_now_mebibytes)
                        .collect(),
                })
            }
            ScratchPlacementDecision::NoCardHoldsOneState {
                largest_ceiling_mebibytes,
            } => Err(BookingRefusal::NeedLargerThanEveryCard(
                NeedLargerThanEveryCard {
                    need_mebibytes: need_of_one_state_mebibytes,
                    largest_ceiling_mebibytes,
                },
            )),
        }
    }

    /// 记上的一笔借据做成借据（丢掉时销账）。
    fn booking_of(&self, booked: BookedOnACard) -> GpuMemoryBooking<'_> {
        GpuMemoryBooking {
            ledger: self,
            card_position: booked.card_position,
            mebibytes: booked.mebibytes_booked,
            lease_identifier: booked.lease_identifier,
            card_room_mebibytes_when_booked: booked.room_mebibytes_when_booked,
            when_dropped: LeaseWhenTheBookingIsDropped::ReturnedToTheDriverAndOffTheBooks,
        }
    }

    /// 这个进程在这张卡上登记了没有（在它上面借过一笔就登记了）。
    #[must_use]
    pub fn is_this_process_registered_on_the_card(&self, card_position: usize) -> bool {
        match self.books_locked()[card_position].driver_share {
            DriverShareOnTheCard::NotRegistered => false,
            DriverShareOnTheCard::RegisteredDeviceNotStarted
            | DriverShareOnTheCard::DeviceStarted
            | DriverShareOnTheCard::DeviceLostNotStartedAgain
            | DriverShareOnTheCard::DeviceStartedAgainWhileTheLostDeviceRemains => true,
        }
    }

    /// 在第 `card_position` 张卡上起设备之前调（先登记、后建设备）：这个进程要已经在这张卡上借过一笔。
    ///
    /// # Panics
    /// 这个进程还没在这张卡上登记（起设备的次序被打乱了）。
    pub fn check_registered_before_starting_the_device(&self, card_position: usize) {
        assert!(
            self.is_this_process_registered_on_the_card(card_position),
            "第 {card_position} 张卡：这个进程在它的账上登记之前就要起设备（先登记、后建设备）"
        );
    }

    /// 在第 `card_position` 张卡上起设备之前调（先登记、后建设备）：这个进程要已经在这张卡上借过一笔。设备是头一回起（或上一回没起来），
    /// 驱动那一份在登记的那一笔里一起借过，照旧；起过的设备丢了要重起（驱动那一份记着已经占在卡上；重起好了、丢了的还没放掉时又丢一次也是），
    /// 把它记回许出去而还没占上卡、在用的那一台记成丢了、这个进程占着的照分不出算
    /// （`BooksOfACard::take_back_the_driver_share_of_a_lost_device`：丢了的设备连它上面的缓冲可能还占着，放掉之前一直这样算），
    /// 在锁里现读一次：借着的连新设备的驱动那一份照此刻的空闲放不下，交借不到，调用方不起（攻方：重起不另登记驱动那一份，
    /// 触发重起的那一笔是按驱动那一份已在卡上借的，新设备的上下文没留地方）。交借不到之后驱动那一份照旧记着许出去而没占上卡
    /// （[`DriverShareOnTheCard::DeviceLostNotStartedAgain`]），新设备起好之前每一次再来都照此刻再判一次（正推 S6：
    /// 被拒之前已借到、借据还在手上的并发用例是按丢之前的账借的，再来起设备时不直接放行）。
    ///
    /// # Errors
    /// 重起而此刻放不下（[`BookingRefusal::NoCardHasRoomNow`]，要的是驱动那一份，带着这张卡此刻能借的）；卡上的显存读不出。
    ///
    /// # Panics
    /// 同 [`Self::check_registered_before_starting_the_device`]。
    pub fn prepare_the_driver_share_before_starting_the_device(
        &self,
        card_position: usize,
    ) -> Result<(), BookingRefusal> {
        let mut books = self.books_locked();
        match books[card_position].driver_share {
            DriverShareOnTheCard::NotRegistered => panic!(
                "第 {card_position} 张卡：这个进程在它的账上登记之前就要起设备（先登记、后建设备）"
            ),
            DriverShareOnTheCard::RegisteredDeviceNotStarted => Ok(()),
            DriverShareOnTheCard::DeviceStarted
            | DriverShareOnTheCard::DeviceStartedAgainWhileTheLostDeviceRemains => {
                books[card_position].take_back_the_driver_share_of_a_lost_device();
                self.driver_share_of_a_new_device_fits_now(&books, card_position)
            }
            DriverShareOnTheCard::DeviceLostNotStartedAgain => {
                self.driver_share_of_a_new_device_fits_now(&books, card_position)
            }
        }
    }

    /// 第 `card_position` 张卡上丢了的设备要重起（驱动那一份记着许出去而还没占上卡）：在锁里现读一次，借着的连新设备的驱动那一份此刻放不放得下。
    ///
    /// # Errors
    /// 同 [`Self::prepare_the_driver_share_before_starting_the_device`]。
    fn driver_share_of_a_new_device_fits_now(
        &self,
        books: &[BooksOfACard],
        card_position: usize,
    ) -> Result<(), BookingRefusal> {
        let readings = self.readings_of_every_card(books)?;
        let booked = books[card_position].booked();
        let capacity_mebibytes = (self.capacity_for_this_process)(
            card_position,
            memory_this_process_may_hold_mebibytes(
                readings[card_position],
                booked,
                self.mebibytes_each_process_keeps_on_a_card,
            ),
        );
        if capacity_mebibytes < booked.leases_mebibytes {
            return Err(BookingRefusal::NoCardHasRoomNow {
                need_mebibytes: self.mebibytes_each_process_keeps_on_a_card,
                card_rooms_mebibytes: vec![
                    capacity_mebibytes.saturating_sub(booked.leases_mebibytes)
                ],
            });
        }
        Ok(())
    }

    /// 第 `card_position` 张卡上的设备起好了：从此这个进程的驱动那一份算实际占在卡上（读到实际占着的时候照它算）；起的是丢了的设备的替身、
    /// 丢了的那几台还有没放掉的，到它们都放掉之前照分不出算（[`DriverShareOnTheCard::DeviceStartedAgainWhileTheLostDeviceRemains`]）。
    /// 交回账上的这一台：调用方把它放进这台设备的每一个克隆，最后一个克隆丢掉时放掉（[`DeviceStartedOnTheBooks`]）。
    ///
    /// # Panics
    /// 同 [`Self::check_registered_before_starting_the_device`]；这张卡上在用的那一台还没记成丢了、也没放掉（起设备只在没起过、
    /// 上一次没起来、或丢了时）。
    #[must_use]
    pub fn record_the_device_started_on_the_card(
        &self,
        card_position: usize,
    ) -> DeviceStartedOnTheBooks<'_> {
        self.check_registered_before_starting_the_device(card_position);
        let mut books = self.books_locked();
        let books_of_the_card = &mut books[card_position];
        assert_eq!(
            books_of_the_card.device_in_use, None,
            "第 {card_position} 张卡：在用的那一台没记成丢了就又起了一台（起设备只在没起过、上一次没起来、或丢了时）"
        );
        let device = books_of_the_card.next_device_number;
        books_of_the_card.next_device_number = DeviceNumberOnTheCard(device.0 + 1);
        books_of_the_card.device_in_use = Some(device);
        books_of_the_card.driver_share = if books_of_the_card.lost_devices_not_released.is_empty() {
            DriverShareOnTheCard::DeviceStarted
        } else {
            DriverShareOnTheCard::DeviceStartedAgainWhileTheLostDeviceRemains
        };
        DeviceStartedOnTheBooks {
            ledger: self,
            card_position,
            device,
        }
    }

    /// 列卡：此刻每张有等设备没等成而照旧记着的借据的卡，连同那一刻照旧记着的那几笔（卡池借之前对这几张卡留着的设备 poll 一次，交回队列空了
    /// 只销列出来的那几笔，[`DeviceStartedOnTheBooks::take_in_that_the_work_of_the_listed_leases_finished`]，
    /// 留着的设备丢了的告诉账，[`Self::take_in_the_devices_kept_before_booking`]）。按卡在池里的先后。
    #[must_use]
    pub fn leases_kept_after_a_failed_wait_on_each_card(
        &self,
    ) -> Vec<LeasesKeptAfterAFailedWaitWhenListed> {
        self.books_locked()
            .iter()
            .enumerate()
            .filter(|(_card_position, books_of_the_card)| {
                !books_of_the_card.leases_kept_after_a_failed_wait.is_empty()
            })
            .map(
                |(card_position, books_of_the_card)| LeasesKeptAfterAFailedWaitWhenListed {
                    card_position,
                    leases_kept_on_each_device: books_of_the_card
                        .leases_kept_after_a_failed_wait
                        .clone(),
                },
            )
            .collect()
    }

    /// 卡池借之前那一步（`singlefs_verification::gpu_unit_checks::gpu_test_start::GpuCardPool` 借之前只调它）：先列卡
    /// （[`Self::leases_kept_after_a_failed_wait_on_each_card`]），再对池里每张卡取卡池那一格留着的设备（`device_kept_on_the_card`，
    /// 不等正在起的设备；没有的跳过）：
    /// - 丢了（`is_lost`）：不 poll（wgpu 在丢了的设备上 poll 会 panic），告诉账这台设备丢了，照重起那一套记
    ///   （[`DeviceStartedOnTheBooks::take_in_that_the_device_was_found_lost`]：它上面没等成的那几笔销账、这张卡照分不出算）。
    ///   不等借到之后走到重起才记：那几笔记着的量让后面的借都放不下时，借不到就走不到重起，这张卡在这个进程里再也借不出
    ///   （正推账层探针：丢了之后能借的 1 000、借 4 000 被拒）。
    ///   没有没等成借据的卡（列不出来）也看：丢了的设备的驱动那一份与缓冲还占着卡，账不知道丢了时照读数把它们算成这个进程许出去的、
    ///   多给，借走之后重起时照此刻再判被拒、报本次未跑（
    ///   能借的 29 550、卡上真能再给的 28 950）；
    /// - 没丢、这张卡列出来了：poll 一次（`found_idle_by_a_poll`，不等），交回队列空了只销列出来的那几笔
    ///   （[`DeviceStartedOnTheBooks::take_in_that_the_work_of_the_listed_leases_finished`]）；没列出来的不 poll。
    ///
    /// 不在账的锁里 poll；`on_the_books` 交那台设备在账上的那一台。
    ///
    /// # Panics
    /// `on_the_books` 交的那一台不在这本账、或不在取它的那张卡上（卡池那一格接错了卡）。
    pub fn take_in_the_devices_kept_before_booking<Device>(
        &self,
        device_kept_on_the_card: impl Fn(usize) -> Option<Device>,
        is_lost: impl Fn(&Device) -> bool,
        found_idle_by_a_poll: impl Fn(&Device) -> bool,
        on_the_books: impl Fn(&Device) -> &DeviceStartedOnTheBooks<'_>,
    ) {
        let listed_cards = self.leases_kept_after_a_failed_wait_on_each_card();
        let card_count = self.books_locked().len();
        for card_position in 0..card_count {
            let Some(kept) = device_kept_on_the_card(card_position) else {
                continue;
            };
            let kept_on_the_books = on_the_books(&kept);
            assert!(
                std::ptr::eq(kept_on_the_books.ledger, self)
                    && kept_on_the_books.card_position == card_position,
                "取的是第 {card_position} 张卡那一格留着的设备，它在账上记在第 {} 张卡上（或别的账上）",
                kept_on_the_books.card_position
            );
            if is_lost(&kept) {
                kept_on_the_books.take_in_that_the_device_was_found_lost();
                continue;
            }
            let Some(listed) = listed_cards
                .iter()
                .find(|listed| listed.card_position == card_position)
            else {
                continue;
            };
            if found_idle_by_a_poll(&kept) {
                kept_on_the_books.take_in_that_the_work_of_the_listed_leases_finished(listed);
            }
        }
    }

    /// 借据号是 `lease_identifier` 的那一笔此刻还记在第 `card_position` 张卡上没有（还了就不在了）。
    #[must_use]
    pub fn is_the_lease_on_the_books(&self, card_position: usize, lease_identifier: u64) -> bool {
        self.books_locked()[card_position]
            .leases
            .contains_key(&lease_identifier)
    }
}

/// 账上一台起好的设备（[`GpuMemoryLedger::record_the_device_started_on_the_card`] 交回）：卡池把它放进这台设备的每一个克隆
/// （`singlefs_verification::gpu_unit_checks::GpuCard` 的最后一个字段，借据的卡、卡池那一格、调用方留着的卡都是克隆），
/// 最后一个克隆丢掉时这一台放掉：这个进程里它的句柄都没了，wgpu 收掉设备、它上面的缓冲交回驱动（wgpu-core 30 的 `Queue::drop` 先等设备空了
/// 再收拾），它记成丢了的不再让这张卡照分不出算，在它上面没等成的借据销账。
/// 管不到的：调用方把 wgpu 的设备、队列、缓冲从克隆里另拿出来、比卡活得久（卡池这边没有这种拿法，借据先于它的缓冲丢的那一条同一个前提）。
pub struct DeviceStartedOnTheBooks<'ledger> {
    ledger: &'ledger GpuMemoryLedger,
    card_position: usize,
    device: DeviceNumberOnTheCard,
}

impl DeviceStartedOnTheBooks<'_> {
    /// 列卡（`listed`，[`GpuMemoryLedger::leases_kept_after_a_failed_wait_on_each_card`]）之后对这台设备 poll 一次交回队列空了：到 poll 那一刻为止提交的活
    /// 都做完了，wgpu 收完那几次提交、丢掉的缓冲交回了驱动。列出来的、等的是这一台、此刻还照旧记着的那几笔销账（它们的缓冲在列卡之前就丢了、
    /// 活在那之前提交）；列卡之后才记成没等成的照旧记着（两次超时之间没有别的等成时也出得来）。
    ///
    /// # Panics
    /// `listed` 列的不是这台设备那张卡。
    pub fn take_in_that_the_work_of_the_listed_leases_finished(
        &self,
        listed: &LeasesKeptAfterAFailedWaitWhenListed,
    ) {
        assert_eq!(
            listed.card_position, self.card_position,
            "列的是第 {} 张卡，poll 的设备在第 {} 张卡上：只销这张卡上列出来的",
            listed.card_position, self.card_position
        );
        self.ledger.books_locked()[self.card_position]
            .return_the_listed_leases_kept_after_a_failed_wait_on(
                self.device,
                &listed.leases_kept_on_each_device,
            );
    }

    /// 卡池借之前看到这台设备丢了（[`GpuMemoryLedger::take_in_the_devices_kept_before_booking`]，还没走到重起）：它还是这张卡上在用的那一台时，
    /// 照重起那一套记（`BooksOfACard::take_back_the_driver_share_of_a_lost_device`：驱动那一份记回许出去而还没占上卡、这一台记成丢了、
    /// 放掉之前这个进程占着的照分不出算，它上面没等成而照旧记着的那几笔销账）；之后重起时驱动那一份照此刻再判一次
    /// （[`GpuMemoryLedger::prepare_the_driver_share_before_starting_the_device`]）。已经记成丢了、或这张卡已经起了别的一台（卡池看到的
    /// 是重起之前留着的那一份）时什么都不改：起好的那一台没丢，它上面没等成的照旧记着。
    pub fn take_in_that_the_device_was_found_lost(&self) {
        let mut books = self.ledger.books_locked();
        let books_of_the_card = &mut books[self.card_position];
        if books_of_the_card.device_in_use == Some(self.device) {
            books_of_the_card.take_back_the_driver_share_of_a_lost_device();
        }
    }
}

impl Drop for DeviceStartedOnTheBooks<'_> {
    fn drop(&mut self) {
        self.ledger.books_locked()[self.card_position].release_a_device(self.device);
    }
}

/// 这个进程在哪几张卡（PCI 总线号，[`pci_bus_identifier_without_the_domain`] 写法）上有不走这份账的设备（经入口直接起的：
/// `gpu_unit_checks::gpu_cards_by_priority`），每条总线记几台：卡池读这张卡时，记着的总线上这个进程占着的那一份
/// 分不出哪些是借据建的（[`OccupiedByThisProcess::AlsoOutsideTheBooks`]，只会少借）。
/// 起好的每台各拿一个 [`DeviceOutsideTheBooksMarkedOnTheBus`]，调用方把它放进那台设备的每一个克隆：最后一个克隆丢掉（这个进程里那台设备的句柄
/// 都没了）时这条总线上减一台，减到 0 的总线摘掉，卡池读那张卡回到按账拆（`OnlyThroughTheBooks`）。
/// 起不来的那一台照样记着、不减（起设备的半路这个进程在卡上占了什么、驱动什么时候还，这里不知道：分不出，只会少借）。
pub struct BusesWithDevicesStartedOutsideTheBooks {
    devices_marked_by_bus: Mutex<BTreeMap<String, usize>>,
}

/// 空着（一个不走账的设备都没起过）就是起点，没有别的不变量。
impl Default for BusesWithDevicesStartedOutsideTheBooks {
    fn default() -> Self {
        Self::new()
    }
}

/// 不走账的一台起好的设备在它那条总线上的那一记（[`BusesWithDevicesStartedOutsideTheBooks::start_a_device_marked_first`] 交的）：
/// 丢掉时那条总线上减一台。调用方把它放进那台设备的每一个克隆、放在最后一个字段（字段照声明的次序丢：同一个克隆里的设备、队列与管线先丢）。
pub struct DeviceOutsideTheBooksMarkedOnTheBus<'table> {
    table: &'table BusesWithDevicesStartedOutsideTheBooks,
    bus: String,
}

impl Drop for DeviceOutsideTheBooksMarkedOnTheBus<'_> {
    fn drop(&mut self) {
        self.table.take_one_device_off_the_bus(&self.bus);
    }
}

impl BusesWithDevicesStartedOutsideTheBooks {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            devices_marked_by_bus: Mutex::new(BTreeMap::new()),
        }
    }

    /// 持锁只为改一格计数，没有会 panic 到一半的写：别的线程 panic 留下的毒照用。
    fn devices_marked_by_bus_locked(&self) -> MutexGuard<'_, BTreeMap<String, usize>> {
        self.devices_marked_by_bus
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    /// `bus` 上多记一台。
    fn mark_one_more_device_on_the_bus(&self, bus: &str) {
        let mut devices_marked_by_bus = self.devices_marked_by_bus_locked();
        let devices_on_the_bus = devices_marked_by_bus.entry(bus.to_string()).or_insert(0);
        *devices_on_the_bus = devices_on_the_bus
            .checked_add(1)
            .expect("一条总线上记着的设备数装得进 usize：每台都是这个进程起的");
    }

    /// `bus` 上减一台，减到 0 摘掉这条总线。
    ///
    /// # Panics
    /// `bus` 上一台都没记着：每一记（[`DeviceOutsideTheBooksMarkedOnTheBus`]）都是先在它那条总线上加过一台才交出的，丢一记只减一台，
    /// 没记着就是账被别处改坏了。
    fn take_one_device_off_the_bus(&self, bus: &str) {
        let mut devices_marked_by_bus = self.devices_marked_by_bus_locked();
        let devices_on_the_bus = devices_marked_by_bus
            .get_mut(bus)
            .expect("丢掉的这一记在它那条总线上加过一台，那条总线还记着");
        *devices_on_the_bus = devices_on_the_bus
            .checked_sub(1)
            .expect("丢掉的这一记在它那条总线上加过一台，这条总线上至少还记着它那一台");
        if *devices_on_the_bus == 0 {
            devices_marked_by_bus.remove(bus);
        }
    }

    /// 先在 `bus` 上多记一台，再调 `start` 起，交 `start` 起好的设备连同这一台的那一记：起设备到记下之间那一段，卡池读到的这个进程占着的
    /// 已经有那个设备的那一份，后记就被当成借据建上的（攻方乙形）。起不来的那一台照样记着、不交那一记（分不出，只会少借）。
    ///
    /// # Errors
    /// `start` 的错。
    pub fn start_a_device_marked_first<Device, Failure>(
        &self,
        bus: &str,
        start: impl FnOnce() -> Result<Device, Failure>,
    ) -> Result<(Device, DeviceOutsideTheBooksMarkedOnTheBus<'_>), Failure> {
        self.mark_one_more_device_on_the_bus(bus);
        let started_device = start()?;
        Ok((
            started_device,
            DeviceOutsideTheBooksMarkedOnTheBus {
                table: self,
                bus: bus.to_string(),
            },
        ))
    }

    /// 这个进程在 `bus` 那张卡上有（或正在起、起不来过）不走账的设备没有。
    #[must_use]
    pub fn has_devices_on_the_bus(&self, bus: &str) -> bool {
        self.devices_marked_by_bus_locked().contains_key(bus)
    }

    /// 每条总线上记着的设备加起来几台（还活着的，加起不来过的）。
    #[must_use]
    pub fn devices_marked_on_every_bus(&self) -> usize {
        self.devices_marked_by_bus_locked().values().sum()
    }
}

/// 卡上一段活的号：接 Out of Memory 的那一层每起一段活发一个（[`WorkOnTheCardOfThisThread::begin`]），进程里不重。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct WorkOnTheCardIdentifier(u64);

static NEXT_WORK_ON_THE_CARD_IDENTIFIER: AtomicU64 = AtomicU64::new(0);

thread_local! {
    /// 这条线程上此刻正在做的那段卡上的活（[`WorkOnTheCardOfThisThread`]）。
    static WORK_ON_THE_CARD_OF_THIS_THREAD: Cell<Option<WorkOnTheCardIdentifier>> = const { Cell::new(None) };
}

/// 这条线程上正在做的一段卡上的活：活着时，这条线程上记的 Out of Memory 算它的（[`DeviceConditionOfACard::record_an_out_of_memory`]）；
/// 丢掉时这条线程换回原来那一段（没有就是没有）。只在起它的那条线程上丢（不跨线程送）。
pub struct WorkOnTheCardOfThisThread {
    identifier: WorkOnTheCardIdentifier,
    work_of_this_thread_before: Option<WorkOnTheCardIdentifier>,
    stays_on_the_thread_that_began_it: PhantomData<*const ()>,
}

impl WorkOnTheCardOfThisThread {
    /// 在这条线程上起一段新的活（发一个新号）。
    #[must_use]
    pub fn begin() -> Self {
        let identifier = WorkOnTheCardIdentifier(
            NEXT_WORK_ON_THE_CARD_IDENTIFIER.fetch_add(1, Ordering::Relaxed),
        );
        let work_of_this_thread_before =
            WORK_ON_THE_CARD_OF_THIS_THREAD.with(|work| work.replace(Some(identifier)));
        Self {
            identifier,
            work_of_this_thread_before,
            stays_on_the_thread_that_began_it: PhantomData,
        }
    }

    #[must_use]
    pub fn identifier(&self) -> WorkOnTheCardIdentifier {
        self.identifier
    }
}

impl Drop for WorkOnTheCardOfThisThread {
    fn drop(&mut self) {
        WORK_ON_THE_CARD_OF_THIS_THREAD.with(|work| work.set(self.work_of_this_thread_before));
    }
}

/// 一次记下的 Out of Memory：wgpu 报的那一句，与它说得成哪几段活的显存不够。
#[derive(Clone, Debug, PartialEq, Eq)]
struct OutOfMemoryReport {
    what: String,
    works_it_explains: Vec<WorkOnTheCardIdentifier>,
}

/// 一个设备上 wgpu 报过的 Out of Memory（错误范围接住的与没接住的都记）与设备丢没丢（几个克隆共用一份）。接 Out of Memory 的那一层
/// （崩溃放量产品路径 `singlefs_verification::crash_amplification`）照它认出：活 panic 了、或设备丢了，是不是这段活的显存不够
/// （wgpu 的错误范围按线程分，判器内核在另起的线程上编，那里的 Out of Memory 范围接不住）。
/// 每一次按记的那条线程认是哪段活的（攻方：别的线程上一次 Out of Memory 不把这段活真正的 panic 说成显存不够）：
/// 那条线程上正在做一段活就是那一段；没有（活另起的编内核的线程）就是那一刻这个设备上在编内核的那几段（[`Self::compile_for_the_work_of_this_thread`]）。
/// `Default` 是不在哪张真卡上的一份（测试里假的设备情况）；真卡上起的设备用 [`Self::on_the_card`]。
#[derive(Default)]
pub struct DeviceConditionOfACard {
    out_of_memory_reports: Mutex<Vec<OutOfMemoryReport>>,
    /// 此刻在这个设备上编内核的活，各有几处在编。
    works_compiling: Mutex<BTreeMap<WorkOnTheCardIdentifier, usize>>,
    /// 设备丢了的话：记了就不抹（重起的是新设备、新的一份）。
    lost: Mutex<Option<LossOfTheDevice>>,
    /// 真卡的名字。`None` 的一份不打 [`GPU_DEVICE_LOST_MARKER`] 那一行：假的设备情况上记丢失的用例，因断言红时输出里不许有这一行，
    /// 否则变异判「抓到」把断言红认成卡出了事。
    card_name_of_the_device: Option<String>,
    /// [`GPU_DEVICE_LOST_MARKER`] 那一行打过没有。
    is_the_device_lost_marker_line_told: AtomicBool,
}

/// 设备为什么丢的，按接 Out of Memory 的那一层要做的决定分（设备丢失的回调里认：wgpu 给的那一句是不是它的「显存不够」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CauseOfTheDeviceLoss {
    /// wgpu 因显存不够丢了设备（它把建管线、提交这类调用上的 Out of Memory 当致命）：因设备丢了而失败的活都说得成显存不够，
    /// 不必等别的活把它范围里接到的那一句交回（攻方甲形：同一个设备上不在同一组的活）。
    GpuMemoryShort,
    /// 别的原因（驱动重置、wgpu 说设备丢了、驱动出了意外）。
    Other,
}

/// 记下的一次设备丢失：wgpu 给的原因与那一句（`why`），与认出来的缘由。
#[derive(Clone, Debug, PartialEq, Eq)]
struct LossOfTheDevice {
    why: String,
    cause: CauseOfTheDeviceLoss,
}

/// 一段活交回那一刻（活做完或 panic 之后，等同一组的其余几段之前）设备丢没丢（[`DeviceConditionOfACard::device_when_the_work_returns`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviceWhenTheWorkReturned {
    Lost,
    NotLost,
}

/// 编内核那一段活着时这个设备上记着「在给这段活编」，丢掉时（编完或 panic）抹掉。
struct CompilingForAWork<'condition> {
    works_compiling: &'condition Mutex<BTreeMap<WorkOnTheCardIdentifier, usize>>,
    work: WorkOnTheCardIdentifier,
}

impl Drop for CompilingForAWork<'_> {
    fn drop(&mut self) {
        // 持锁只为改一个计数，没有会 panic 到一半的写：别的线程 panic 留下的毒照用
        let mut works_compiling = self
            .works_compiling
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let still_compiling = works_compiling.get(&self.work).copied().unwrap_or(0);
        if still_compiling <= 1 {
            works_compiling.remove(&self.work);
        } else {
            works_compiling.insert(self.work, still_compiling - 1);
        }
    }
}

impl DeviceConditionOfACard {
    /// 真卡 `card_name` 上起的一台设备的情况，一样都还没记。
    #[must_use]
    pub fn on_the_card(card_name: String) -> Self {
        Self {
            out_of_memory_reports: Mutex::new(Vec::new()),
            works_compiling: Mutex::new(BTreeMap::new()),
            lost: Mutex::new(None),
            card_name_of_the_device: Some(card_name),
            is_the_device_lost_marker_line_told: AtomicBool::new(false),
        }
    }

    /// 记一次 Out of Memory（`what` 说是在哪、wgpu 报的那一句），算在记的这条线程上正在做的那段活上；这条线程上没有活，
    /// 算在此刻这个设备上在编内核的那几段活上（都没有就哪段活都说不成）。
    pub fn record_an_out_of_memory(&self, what: String) {
        let works_it_explains = match WORK_ON_THE_CARD_OF_THIS_THREAD.with(Cell::get) {
            Some(work) => vec![work],
            None => self
                .works_compiling
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .keys()
                .copied()
                .collect(),
        };
        // 持锁只为追加一条，没有会 panic 到一半的写：别的线程 panic 留下的毒照用
        self.out_of_memory_reports
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(OutOfMemoryReport {
                what,
                works_it_explains,
            });
    }

    /// 到此刻一共记了几次 Out of Memory。
    #[must_use]
    pub fn out_of_memory_reports(&self) -> usize {
        self.out_of_memory_reports
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .len()
    }

    /// 在这个设备上为这条线程上正在做的那段活编内核（`compile`，里面另起的线程上记的 Out of Memory 算这段活的），交 `compile` 的结果；
    /// 这条线程上没有活就只是调 `compile`。`compile` panic 照样往外抛。
    pub fn compile_for_the_work_of_this_thread<Output>(
        &self,
        compile: impl FnOnce() -> Output,
    ) -> Output {
        let Some(work) = WORK_ON_THE_CARD_OF_THIS_THREAD.with(Cell::get) else {
            return compile();
        };
        *self
            .works_compiling
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .entry(work)
            .or_insert(0) += 1;
        let _compiling = CompilingForAWork {
            works_compiling: &self.works_compiling,
            work,
        };
        compile()
    }

    /// 记设备丢了（wgpu 的设备丢失回调与等卡没等成的那几处调它）：`why` 是丢失的原因与那一句，`cause` 是认出来的缘由。
    /// 这个设备头一次出事时往标准输出打 [`GPU_DEVICE_LOST_MARKER`] 那一行。
    pub fn record_the_device_lost(&self, why: String, cause: CauseOfTheDeviceLoss) {
        self.record_the_device_lost_telling_the_marker_line(
            why,
            cause,
            print_the_device_lost_marker_line,
        );
    }

    /// 等卡没等成、用例因此要红，而那一处不把设备记成丢了（记了会改卡池与显存不够的认法）：只打 [`GPU_DEVICE_LOST_MARKER`] 那一行，
    /// 与 [`Self::record_the_device_lost`] 合起来这个设备只打一次。
    pub fn print_the_device_lost_marker_line_the_first_time(&self, reason: &str) {
        self.tell_the_device_lost_marker_line_the_first_time(
            reason,
            print_the_device_lost_marker_line,
        );
    }

    fn record_the_device_lost_telling_the_marker_line(
        &self,
        why: String,
        cause: CauseOfTheDeviceLoss,
        tell_the_marker_line: impl FnOnce(&str),
    ) {
        let reason_for_the_marker_line = why.clone();
        *self.lost.lock().unwrap_or_else(PoisonError::into_inner) =
            Some(LossOfTheDevice { why, cause });
        self.tell_the_device_lost_marker_line_the_first_time(
            &reason_for_the_marker_line,
            tell_the_marker_line,
        );
    }

    fn tell_the_device_lost_marker_line_the_first_time(
        &self,
        reason: &str,
        tell_the_marker_line: impl FnOnce(&str),
    ) {
        let Some(card_name) = self.card_name_of_the_device.as_deref() else {
            return;
        };
        if self
            .is_the_device_lost_marker_line_told
            .swap(true, Ordering::Relaxed)
        {
            return;
        }
        let current_thread = std::thread::current();
        let test_name = current_thread.name().unwrap_or(THREAD_WITHOUT_A_NAME);
        tell_the_marker_line(&gpu_device_lost_line(test_name, card_name, reason));
    }

    fn loss_of_the_device(&self) -> Option<LossOfTheDevice> {
        self.lost
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// 设备丢了的话为什么丢（没丢是 `None`）。
    #[must_use]
    pub fn lost_reason(&self) -> Option<String> {
        self.loss_of_the_device().map(|loss| loss.why)
    }

    /// 设备丢了没有（卡池照它判留着的设备还能不能拿：丢了就重起）。
    #[must_use]
    pub fn is_lost(&self) -> bool {
        self.lost
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .is_some()
    }

    /// 一段活交回的那一刻（活做完或 panic 了、错误范围弹出之前）记下设备丢没丢：之后认它的失败时，设备丢了那一支只给这一刻设备已经丢了的活
    /// （攻方乙形：活自己真 panic 的那一刻设备没丢，之后同组别的段撞上致命的 Out of Memory、设备丢了，
    /// 等齐再认不把这段活说成显存不够）。
    #[must_use]
    pub fn device_when_the_work_returns(&self) -> DeviceWhenTheWorkReturned {
        if self.is_lost() {
            DeviceWhenTheWorkReturned::Lost
        } else {
            DeviceWhenTheWorkReturned::NotLost
        }
    }

    /// 一段活收完：`caught_in_the_scope`（这条线程上的错误范围接到的 Out of Memory，`None` 是没接到）记进设备，交 `ending` 报到、等同一个设备上
    /// 一起开的其余几段都报到，再认这段活从记到 `reports_before` 次那一刻起出的坏事能不能说成显存不够
    /// （[`Self::out_of_memory_explaining_a_failure_of_the_work`]，`device_when_the_work_returned` 是这段活交回那一刻
    /// [`Self::device_when_the_work_returns`] 记的）。先记再报到、先等再认（攻方：一块的几段并发，一段因设备丢了先失败，
    /// 同块另一段范围里接到的 Out of Memory 要等那一段收完才记进设备；先认就认不出、判红）。
    pub fn end_the_work_and_explain_its_failure(
        &self,
        work: &WorkOnTheCardOfThisThread,
        reports_before: usize,
        device_when_the_work_returned: DeviceWhenTheWorkReturned,
        caught_in_the_scope: Option<String>,
        ending: WorkEndingTogether<'_>,
    ) -> Option<String> {
        if let Some(caught) = caught_in_the_scope {
            self.record_an_out_of_memory(caught);
        }
        ending.end_and_wait_for_the_others();
        self.out_of_memory_explaining_a_failure_of_the_work(
            work.identifier(),
            reports_before,
            device_when_the_work_returned,
        )
    }

    /// 段活 `work` 从记到 `reports_before` 次那一刻起出了一样坏事、能不能说成是它的显存不够：这之后记了算在它上面的 Out of Memory，
    /// 交最后那一句。别的只在这段活交回那一刻设备已经丢了（`device_when_the_work_returned`）时才说得成：设备因显存不够丢的
    /// （[`CauseOfTheDeviceLoss::GpuMemoryShort`]，这段活之后记没记过都一样：同一个设备上不在同一组的活撞上的，范围里那一句要等那段活交回），
    /// 交说它的一句；为别的丢的而这之后记过 Out of Memory（算在哪段活上都行：wgpu 把建管线、提交这类调用上的 Out of Memory 当致命、丢掉设备，
    /// 丢了设备谁都做不成），交两样都说的一句；别的交 `None`（别的活上的 Out of Memory、这段活开始之前的、交回那一刻设备还没丢的，都说不成）。
    #[must_use]
    pub fn out_of_memory_explaining_a_failure_of_the_work(
        &self,
        work: WorkOnTheCardIdentifier,
        reports_before: usize,
        device_when_the_work_returned: DeviceWhenTheWorkReturned,
    ) -> Option<String> {
        let reports = self
            .out_of_memory_reports
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let reports_since = reports.get(reports_before..).unwrap_or(&[]);
        if let Some(report) = reports_since
            .iter()
            .rev()
            .find(|report| report.works_it_explains.contains(&work))
        {
            return Some(report.what.clone());
        }
        match device_when_the_work_returned {
            DeviceWhenTheWorkReturned::NotLost => None,
            DeviceWhenTheWorkReturned::Lost => {
                // 交回那一刻照 `device_when_the_work_returns` 记成丢了的，丢失一定还在（只记不抹）；没记过丢失就说不成
                let loss = self.loss_of_the_device()?;
                match (loss.cause, reports_since.last()) {
                    (
                        CauseOfTheDeviceLoss::GpuMemoryShort | CauseOfTheDeviceLoss::Other,
                        Some(report),
                    ) => Some(format!(
                        "设备丢了（{}），丢之前报过 {}",
                        loss.why, report.what
                    )),
                    (CauseOfTheDeviceLoss::GpuMemoryShort, None) => {
                        Some(format!("设备因显存不够丢了（{}）", loss.why))
                    }
                    (CauseOfTheDeviceLoss::Other, None) => None,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        card_ledger_capacity_mebibytes, wait_for_the_device_and_take_in_the_outcome,
        BookingRefusal, CardMemoryReading, DeviceStartedOnTheBooks,
        DeviceWaitBeforeReturningTheLease, GpuMemoryBooking, GpuMemoryLedger,
        OccupiedByThisProcess,
    };
    use std::sync::{Arc, Mutex};

    mod bookings_on_the_cards;
    mod device_condition_and_out_of_memory;
    mod devices_on_the_books_and_outside_them;
    mod leases_kept_after_a_failed_wait;
    mod random_operation_strings;

    /// 驱动的固定开销，照卡池用的那个数（`crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER`）。
    pub(super) const DRIVER: u64 = 2560;
    /// 一张 5090 的显存总量与驱动保留的那一份（本机 `nvidia-smi -q -d MEMORY` 报的 Total、Reserved）。
    const TOTAL_OF_AN_RTX_5090: u64 = 32_607;
    const RESERVED_ON_AN_RTX_5090: u64 = 497;
    /// 一张空着的 5090 上能用的：总量减保留（同一刻报的空闲就是这个数）。
    pub(super) const AN_EMPTY_RTX_5090_HAS: u64 = TOTAL_OF_AN_RTX_5090 - RESERVED_ON_AN_RTX_5090;

    /// 一张 5090 此刻空着 `free_mebibytes`，这个进程实际占着的读不到。
    pub(super) fn reading_of_an_rtx_5090(free_mebibytes: u64) -> CardMemoryReading {
        CardMemoryReading {
            free_mebibytes,
            total_less_reserved_mebibytes: AN_EMPTY_RTX_5090_HAS,
            occupied_by_this_process: OccupiedByThisProcess::NotListed,
        }
    }

    /// 换得了的假读数：几张卡此刻的显存，用例中途改它就是卡上的空闲变了。
    type ReplaceableReadings = Arc<Mutex<Vec<CardMemoryReading>>>;

    pub(super) fn replaceable_readings(readings: Vec<CardMemoryReading>) -> ReplaceableReadings {
        Arc::new(Mutex::new(readings))
    }

    /// 把第 `card_position` 张卡此刻的读数换成 `reading`。
    fn replace_the_reading(
        readings: &ReplaceableReadings,
        card_position: usize,
        reading: CardMemoryReading,
    ) {
        readings.lock().expect("用例里只有一条线程改读数")[card_position] = reading;
    }

    /// 照假读数记账的一本账：容量照卡池没配额度、没设额度变量时的算法里不打 3/4 折扣的那一份
    /// （这个进程能占的减驱动那一份 [`DRIVER`]）。
    pub(super) fn ledger_reading(readings: &ReplaceableReadings) -> GpuMemoryLedger {
        let card_count = readings.lock().expect("还没有别的线程").len();
        let shared = Arc::clone(readings);
        GpuMemoryLedger::new(
            card_count,
            Box::new(move || Ok(shared.lock().expect("用例里只有一条线程改读数").clone())),
            Box::new(|_position, may_hold_mebibytes| {
                card_ledger_capacity_mebibytes(
                    None,
                    None,
                    may_hold_mebibytes,
                    may_hold_mebibytes,
                    DRIVER,
                )
            }),
            DRIVER,
        )
    }

    /// 第 `card_position` 张卡上的设备起好了，账上这一台一直不放掉（卡池那一格一直留着它；用例不看它放掉之后的事）。
    fn record_the_device_started_and_kept_by_the_pool(
        ledger: &GpuMemoryLedger,
        card_position: usize,
    ) {
        std::mem::forget(ledger.record_the_device_started_on_the_card(card_position));
    }

    /// 借一笔：借到了交借的量（借据不还，留在账上），借不到交结局。
    fn booking_kept_on_the_books(
        ledger: &GpuMemoryLedger,
        need_mebibytes: u64,
    ) -> Result<u64, BookingRefusal> {
        ledger
            .book_on_the_first_card_with_room(need_mebibytes)
            .map(|booking| {
                let mebibytes = booking.mebibytes();
                std::mem::forget(booking);
                mebibytes
            })
    }

    /// 测试里进程号写死的「这个进程」与「别的进程」。
    pub(super) const THIS_PROCESS: u32 = 4_242;
    pub(super) const ANOTHER_PROCESS: u32 = 7_777;
    /// 测试里那张 5090 的总线号（`nvidia-smi` 写法，去掉域之前）。
    pub(super) const BUS_OF_THE_RTX_5090: &str = "00000000:01:00.0";
    pub(super) const BUS_OF_THE_RTX_5060_TI: &str = "00000000:10:00.0";

    /// 一张卡在假报告里的那一段：空闲与每个进程（进程号、占的 MiB）。总量、保留照 5090 的两个数写。
    pub(super) struct CardInTheReport<'processes> {
        pub(super) bus: &'processes str,
        pub(super) total_mebibytes: u64,
        pub(super) reserved_mebibytes: u64,
        pub(super) free_mebibytes: u64,
        pub(super) processes: &'processes [(u32, u64)],
    }

    /// 照本机驱动 595.84 的 `nvidia-smi -q -d MEMORY,PIDS` 文本写一份假报告（BAR1、Conf Compute 两小节也带着，
    /// 它们的 Total、Free 不该被当成显存读）。
    pub(super) fn nvidia_smi_report(cards: &[CardInTheReport<'_>]) -> String {
        let mut report = format!(
            "\n==============NVSMI LOG==============\n\nTimestamp                                              : Wed Oct  7 2026\n\
             Driver Version                                         : 595.84\nCUDA Version                                           : 13.2\n\n\
             Attached GPUs                                          : {}\n",
            cards.len()
        );
        for card in cards {
            report.push_str(&format!(
                "GPU {}\n    FB Memory Usage\n        Total                                          : {} MiB\n\
                 \x20       Reserved                                       : {} MiB\n        Used                                           : 2 MiB\n\
                 \x20       Free                                           : {} MiB\n    BAR1 Memory Usage\n\
                 \x20       Total                                          : 32768 MiB\n        Used                                           : 1 MiB\n\
                 \x20       Free                                           : 32767 MiB\n    Conf Compute Protected Memory Usage\n\
                 \x20       Total                                          : 0 MiB\n        Used                                           : 0 MiB\n\
                 \x20       Free                                           : 0 MiB\n",
                card.bus, card.total_mebibytes, card.reserved_mebibytes, card.free_mebibytes
            ));
            if card.processes.is_empty() {
                report
                    .push_str("    Processes                                          : None\n\n");
                continue;
            }
            report.push_str("    Processes\n");
            for (process_identifier, used_mebibytes) in card.processes {
                report.push_str(&format!(
                    "        GPU instance ID                                : N/A\n        Compute instance ID                            : N/A\n\
                     \x20       Process ID                                     : {process_identifier}\n\
                     \x20           Type                                       : C+G\n\
                     \x20           Name                                       : /tmp/target/debug/deps/a_gpu_test\n\
                     \x20           Used GPU Memory                            : {used_mebibytes} MiB\n"
                ));
            }
            report.push('\n');
        }
        report
    }

    /// 一张 5090 的那一段：空 `free_mebibytes`，进程照给的。
    pub(super) fn an_rtx_5090_in_the_report(
        free_mebibytes: u64,
        processes: &[(u32, u64)],
    ) -> CardInTheReport<'_> {
        CardInTheReport {
            bus: BUS_OF_THE_RTX_5090,
            total_mebibytes: TOTAL_OF_AN_RTX_5090,
            reserved_mebibytes: RESERVED_ON_AN_RTX_5090,
            free_mebibytes,
            processes,
        }
    }

    pub(super) fn no_device_outside_the_books(_bus: &str) -> bool {
        false
    }

    /// [`CardUnderTheLedger`] 此刻真能再借给这个进程的（MiB）：空闲减起着的设备驱动还要长到 [`DRIVER`] 的、减活着的借据还没建的
    /// （`unbuilt_mebibytes`，由用例给）。
    fn room_truly_left(card: &Mutex<CardUnderTheLedger>, unbuilt_mebibytes: u64) -> u64 {
        let card = card.lock().expect("假卡不 panic");
        let driver_still_grows = if card.device_started {
            DRIVER - card.driver_occupied_mebibytes
        } else {
            DRIVER
        };
        card.reading()
            .free_mebibytes
            .saturating_sub(driver_still_grows)
            .saturating_sub(unbuilt_mebibytes)
    }

    /// 借据丢掉时那几条用例的等法：`true` 是等完了、`false` 是没等成；等的时候 panic 了也是没等成。
    pub(super) fn wait_of_a_fake_device(
        polled: &std::thread::Result<bool>,
    ) -> DeviceWaitBeforeReturningTheLease {
        if matches!(polled, Ok(true)) {
            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished
        } else {
            DeviceWaitBeforeReturningTheLease::TheWaitFailed
        }
    }

    /// 子进程跑 [`lease_dropped_while_its_thread_unwinds_takes_in_a_panicking_wait_without_panicking_again`] 那一形时设的变量。
    const LEASE_DROPPED_WHILE_UNWINDING_CHILD_VARIABLE: &str =
        "SINGLEFS_HARNESS_LEASE_DROPPED_WHILE_UNWINDING_CHILD";

    /// 子进程里 `--exact` 点名跑的那条用例的 libtest 全名。写成字面串，算复用键的工具（D13（验证路线） 已定项 17）才折得出它、
    /// 把起子进程的用例连到被点名的那条；与模块路径拼出的逐字相同由用例里那条 `assert_eq!` 钉住。
    const LEASE_DROPPED_WHILE_UNWINDING_CHILD_TEST_NAME: &str = "gpu_memory_ledger::tests::lease_dropped_while_its_thread_unwinds_takes_in_a_panicking_wait_without_panicking_again";

    /// （攻方 H3，改法 F-H3，照攻方 `unwind_in_drop.rs` 写，不要卡）：用例本身 panic 了，
    /// 借据在展开中丢掉，等的时候 wgpu 又 panic：只记没等成、不再抛（展开中再抛整个进程中止）。在子进程里跑那一形（中止的是子进程，
    /// 这条用例照常判红），子进程要正常退出、这一笔记成没等成。
    #[test]
    fn lease_dropped_while_its_thread_unwinds_takes_in_a_panicking_wait_without_panicking_again() {
        const TEST_FUNCTION: &str =
            "lease_dropped_while_its_thread_unwinds_takes_in_a_panicking_wait_without_panicking_again";
        if std::env::var_os(LEASE_DROPPED_WHILE_UNWINDING_CHILD_VARIABLE).is_some() {
            struct LeaseReturnedWhenDropped<'ledger, 'device> {
                lease: Option<GpuMemoryBooking<'ledger>>,
                device: &'device DeviceStartedOnTheBooks<'ledger>,
            }
            impl Drop for LeaseReturnedWhenDropped<'_, '_> {
                fn drop(&mut self) {
                    if let Some(mut lease) = self.lease.take() {
                        wait_for_the_device_and_take_in_the_outcome(
                            &mut lease,
                            self.device,
                            || -> bool { panic!("等的时候 wgpu panic 了：设备丢了") },
                            wait_of_a_fake_device,
                            |_polled| {},
                        );
                    }
                }
            }
            let readings = replaceable_readings(vec![reading_of_an_rtx_5090(30_000)]);
            let ledger = ledger_reading(&readings);
            let lease = ledger.book_on_the_card(0, 1_000).expect("空卡借得到");
            let lease_identifier = lease.lease_identifier();
            let device = ledger.record_the_device_started_on_the_card(0);
            let unwound = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _returned_when_dropped = LeaseReturnedWhenDropped {
                    lease: Some(lease),
                    device: &device,
                };
                panic!("用例本身 panic 了");
            }));
            assert!(unwound.is_err(), "用例本身那一个 panic 照样展开到外面");
            println!(
                "LEASE_DROPPED_WHILE_UNWINDING kept_on_the_books={}",
                ledger.is_the_lease_on_the_books(0, lease_identifier)
            );
            return;
        }
        assert_eq!(
            LEASE_DROPPED_WHILE_UNWINDING_CHILD_TEST_NAME,
            format!(
                "{}::{TEST_FUNCTION}",
                module_path!()
                    .split_once("::")
                    .expect("模块路径以包名起头")
                    .1
            ),
            "子进程点名的测试名与这条用例的模块路径（去掉包名）加函数名逐字相同"
        );
        let child = std::process::Command::new(std::env::current_exe().expect("测试二进制的路径"))
            .args([
                "--exact",
                LEASE_DROPPED_WHILE_UNWINDING_CHILD_TEST_NAME,
                "--nocapture",
                "--test-threads",
                "1",
            ])
            .env(LEASE_DROPPED_WHILE_UNWINDING_CHILD_VARIABLE, "1")
            .output()
            .expect("起得了子进程");
        let standard_output = String::from_utf8_lossy(&child.stdout);
        assert!(
            child.status.success(),
            "展开中丢借据、等的时候又 panic：子进程 {}（中止就是展开中又抛了）：{}",
            child.status,
            String::from_utf8_lossy(&child.stderr)
        );
        assert!(
            standard_output.contains("LEASE_DROPPED_WHILE_UNWINDING kept_on_the_books=true"),
            "这一笔记成没等成、丢掉之后照旧记着：{standard_output}"
        );
    }

    fn source_without_whitespace(source: &str) -> String {
        source
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect()
    }

    /// 真打到标准输出的那一步：libtest 的捕获没有稳定的接口接到用例里，这里扫源码钉住两个入口都交 `println!` 那一个函数。
    #[test]
    fn both_entries_of_the_device_lost_marker_line_print_it_on_standard_output() {
        let (code_before_the_tests, _tests) = include_str!("gpu_memory_ledger.rs")
            .split_once("#[cfg(test)]\nmod tests {")
            .expect("这份文件有测试模块");
        let code = source_without_whitespace(code_before_the_tests);
        for expected in [
            "fnprint_the_device_lost_marker_line(marker_line:&str){println!(\"{marker_line}\");}",
            "pubfnrecord_the_device_lost(&self,why:String,cause:CauseOfTheDeviceLoss){self.record_the_device_lost_telling_the_marker_line(why,cause,print_the_device_lost_marker_line,);}",
            "pubfnprint_the_device_lost_marker_line_the_first_time(&self,reason:&str){self.tell_the_device_lost_marker_line_the_first_time(reason,print_the_device_lost_marker_line,);}",
        ] {
            assert_eq!(
                code.matches(expected).count(),
                1,
                "源码里恰好一处 {expected}"
            );
        }
    }

    /// 真卡上起的设备带卡名（不带就一行都不打），等卡没等成而不记丢失的那几处都打那一行。这几处要真卡、真超时才走得到，这里扫源码钉住。
    #[test]
    fn started_card_has_its_name_and_the_failed_waits_that_record_no_loss_tell_the_marker_line() {
        for (file_name, source, expected) in [
            (
                "gpu_unit_checks/discovering_and_starting_the_cards.rs 起设备",
                include_str!("gpu_unit_checks/discovering_and_starting_the_cards.rs"),
                "std::sync::Arc::new(DeviceConditionOfACard::on_the_card(discovered.name.clone()))",
            ),
            (
                "gpu_unit_checks/gpu_test_start.rs 借据丢掉时那一次等",
                include_str!("gpu_unit_checks/gpu_test_start.rs"),
                "self.card.device_condition().print_the_device_lost_marker_line_the_first_time(",
            ),
            (
                "crash_amplification/images_after_the_observers.rs 还占位之前那一次等",
                include_str!("crash_amplification/images_after_the_observers.rs"),
                "card_with_the_admitted_quota.device_condition().print_the_device_lost_marker_line_the_first_time(",
            ),
            (
                "标定空转内核那一次等",
                include_str!(
                    "../tests/gpu_unit_checks_a_returned_card_lease_leaves_no_buffer_on_the_card.rs"
                ),
                "condition.print_the_device_lost_marker_line_the_first_time(",
            ),
        ] {
            assert_eq!(
                source_without_whitespace(source).matches(expected).count(),
                1,
                "{file_name} 里恰好一处 {expected}"
            );
        }
    }

    /// 正推 `probes/killers.rs` 那几条用的一张假 5090：照它此刻卡上有什么给读数（这个进程只经账占卡），驱动那一份 [`DRIVER`]。
    #[derive(Clone, Default)]
    struct CardUnderTheLedger {
        device_started: bool,
        /// 起着的设备驱动实占的。
        driver_occupied_mebibytes: u64,
        /// 别的进程占的。
        others_mebibytes: u64,
        /// 活着的借据建上卡的合计。
        built_mebibytes: u64,
        /// 丢了的设备留在卡上的。
        left_by_a_lost_device_mebibytes: u64,
    }

    impl CardUnderTheLedger {
        fn occupied_by_this_process_mebibytes(&self) -> u64 {
            (if self.device_started {
                self.driver_occupied_mebibytes
            } else {
                0
            }) + self.built_mebibytes
                + self.left_by_a_lost_device_mebibytes
        }

        fn reading(&self) -> CardMemoryReading {
            let occupied = self.occupied_by_this_process_mebibytes();
            CardMemoryReading {
                free_mebibytes: AN_EMPTY_RTX_5090_HAS - self.others_mebibytes - occupied,
                total_less_reserved_mebibytes: AN_EMPTY_RTX_5090_HAS,
                occupied_by_this_process: if occupied > 0 {
                    OccupiedByThisProcess::OnlyThroughTheBooks {
                        mebibytes: occupied,
                    }
                } else {
                    OccupiedByThisProcess::NotListed
                },
            }
        }
    }

    /// 账接一张 [`CardUnderTheLedger`]：没配额度、没配优先级（驱动之外照能占的算）。
    fn ledger_on_a_card_under_the_ledger() -> (Arc<Mutex<CardUnderTheLedger>>, GpuMemoryLedger) {
        let card = Arc::new(Mutex::new(CardUnderTheLedger::default()));
        let read_card = Arc::clone(&card);
        let ledger = GpuMemoryLedger::new(
            1,
            Box::new(move || Ok(vec![read_card.lock().expect("假卡不 panic").reading()])),
            Box::new(|_card_position, may_hold| {
                card_ledger_capacity_mebibytes(None, None, may_hold, may_hold, DRIVER)
            }),
            DRIVER,
        );
        (card, ledger)
    }

    fn room_now(ledger: &GpuMemoryLedger) -> u64 {
        ledger.card_rooms_mebibytes_now(&[0]).expect("假卡读得出")[0]
    }

    /// 借一笔 10 000、起设备、建满：之后的几条从这里起。
    fn ten_thousand_built_on_a_started_device<'ledger>(
        card: &Mutex<CardUnderTheLedger>,
        ledger: &'ledger GpuMemoryLedger,
    ) -> GpuMemoryBooking<'ledger> {
        let lease = ledger
            .book_on_the_card(0, 10_000)
            .expect("空卡借得到 10 000");
        {
            let mut card = card.lock().expect("假卡不 panic");
            card.device_started = true;
            card.driver_occupied_mebibytes = DRIVER;
            card.built_mebibytes = 10_000;
        }
        record_the_device_started_and_kept_by_the_pool(ledger, 0);
        lease
    }

    /// 设备丢了：驱动那一份与建上的留在卡上当旧设备那一堆，别的进程占了 `others_mebibytes`。
    fn the_device_is_lost_with_others_taking(
        card: &Mutex<CardUnderTheLedger>,
        others_mebibytes: u64,
    ) {
        let mut card = card.lock().expect("假卡不 panic");
        card.device_started = false;
        card.driver_occupied_mebibytes = 0;
        card.left_by_a_lost_device_mebibytes = DRIVER;
        card.others_mebibytes = others_mebibytes;
    }
}
