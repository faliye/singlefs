//! GPU 卡上的显存怎么分：一个判器一次派活最多带几个状态（草稿区按这条流的状态数封顶），与这个进程里 GPU 用例共用的卡池怎么记账。
//! 单放在 harness 档、只用 `std`：checker 档的 `crash_judge_dispatch`（草稿区封顶）与 `gpu_unit_checks::gpu_test_start` 的卡池
//! 都调它；这几样不碰 wgpu、不要显卡，单测在 harness 档随时跑得到（checker 档的单测只在提交时跑）。
//! 挑卡照卡在池里的先后（配置的优先级）找第一张放得下的，前一张放满了才用下一张
//! （用户 2026-09-29 定「一个显卡 任务满了才去分另外一个」「一边占一点绝对不合理」）；
//! 账只拦放不下的，不限同时借几份、也不限一台机器上同时跑几个 GPU 测试进程（用户 2026-10-06 定「开到最大的并发和gpu」
//! 「仅仅管会不会对指定的电脑造成cpu或者gpu的oom」「不压并发」）。
//!
//! 只记这个进程借的（用户 2026-10-07 定第 10t 批代码轮判决岔路 G1「二：去掉跨进程账」）：没有账本文件、不锁文件、
//! 不判别的进程活没活，也不等。每一次借都在这个进程的锁里现读一次每张卡此刻的显存（[`CardMemoryReading`]：空闲、空卡上能用的、
//! 这个进程实际占着的），照这个进程在这张卡上已经借着的算这一笔放不放得下（[`card_room_for_this_process`]）。
//! 卡池每读一次跑一次 `nvidia-smi -q -d MEMORY,PIDS`（[`NVIDIA_SMI_MEMORY_AND_PROCESSES_REPORT_ARGUMENTS`]）：空闲与这个进程占着的
//! 出自同一份报告（[`card_memory_reader_of_one_report_per_reading`]，第 10x 批代码轮判决 K1-1：分两次读时，两次之间这个进程别的线程
//! 建上卡的那一笔既在空闲里又在占着的里，算两遍）。
//!
//! 口径（第 10x 批，主 agent 定「三者取最小」照第 10t、10v 批）：设 F 是现读的空闲，B 是这个进程在这张卡上借着的，
//! D 是驱动的固定开销（每个进程在每张卡上一份，起设备时付），M 是这个进程此刻能在这张卡上占的
//! （[`memory_this_process_may_hold_mebibytes`]：F 加上这个进程实际占着的里属于它借出去的那一部分）。这一笔能借的是
//! `card_ledger_capacity_mebibytes(…, M, D) − B`，展开就是下面几项里最小的：
//! - 额度变量 `SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES` − B（设了才有这一项）；
//! - 配置给卡的额度 − D − B；没配显卡优先级时由 M 的 3/4 − D − B 代替（这几项都是这个进程在这张卡上一共借的连驱动那一份不超过额度）；
//! - F − 这个进程许出去而还没占上卡的那一份：许出去的是 B 连驱动那一份 D（还没登记时 B 是 0、D 照样要放得下），
//!   减实际占着的里属于许出去的那部分。账上只有活着的借据：借据丢掉时卡池先等这张卡的设备把提交了的活都做完、丢掉的缓冲交回驱动，
//!   再销账（[`GpuMemoryBooking::take_in_the_wait_before_returning`]；用户 2026-10-08 定第 10z 批代码轮岔路 G2「改用乙」，E174 在两张卡上量过：
//!   等完之后下一次读数里那笔缓冲都已不在），销了账的那一份不再记、也不从空闲里扣（第 10z 批「甲」那一份「销了账、卡上可能还没还的」整个拿掉）。
//!   等失败（超时或报错）的那一笔照旧记在账上，只会少借；它等的那台设备上提交的活后来做完了就销（卡池借之前列出每张卡上那一刻照旧记着的那几笔、
//!   对那张卡 poll 一次交队列空了只销列出来的那几笔，[`DeviceStartedOnTheBooks::take_in_that_the_work_of_the_listed_leases_finished`]；
//!   或同一台设备上另一笔借据等成了），那台设备记成丢了、或放掉了也销
//!   （第 10z 批第八块，第 10z 批第六块代码轮判决 K1-2：两腿都量到等超时的那几笔累加到整张卡借不出，到设备丢了才回来；
//!   第 10z 批第十一块，第 10z 批第七八九块代码轮判决 K1-1：列卡之后才记成没等成的那一笔不一起销，它的活可以是 poll 之后才提交的）。
//!   卡池借之前看到池里哪张卡留着的设备丢了，当场告诉账、照重起那一套记成丢了（[`GpuMemoryLedger::take_in_the_devices_kept_before_booking`]），
//!   不等借到之后走到重起（第 10z 批第十三块，第 10z 批第十、十一块代码轮判决 K4-2：那几笔让后面的借都放不下时走不到重起，卡空着却再也借不出）；
//!   那张卡上没有没等成的借据也一样（第 10z 批第十五块，第 10z 批第十二、十三块代码轮判决 K3-3：账不知道丢了时把丢了的设备还占着的
//!   驱动那一份当成这个进程许出去的，多给，借走之后重起时被拒、报本次未跑）。
//!   读不到实际占着的（`nvidia-smi` 没列这个进程），或这个进程在这张卡上另有不走这份账的设备（[`OccupiedByThisProcess::AlsoOutsideTheBooks`]，
//!   分不出哪一份是借据建的），借着的当一点都没建、驱动那一份也当还没占上卡（第 10y 批代码轮判决 K1-1 攻方乙附：当它整份在卡上，实占的 x 比 D 小时
//!   多给 D − x），只会少借。这张卡上起好过的设备丢了、账照重起记了之后，到丢了的那几台都放掉之前也这样算
//!   （[`DriverShareOnTheCard::DeviceLostNotStartedAgain`]、[`DriverShareOnTheCard::DeviceStartedAgainWhileTheLostDeviceRemains`]：
//!   丢了的设备连它上面的缓冲可能还占着卡，这个进程占着的分不出哪一份是新设备的）。账上的每台设备是一个 [`DeviceStartedOnTheBooks`]，
//!   卡池把它放进那台设备的每一个克隆：最后一个克隆丢掉（这个进程里那台设备的句柄都没了，wgpu 收掉设备、它上面的缓冲交回驱动）才算放掉
//!   （第 10z 批第八块，第 10z 批第六块代码轮判决 K1-3：丢了那一刻活着的借据没等成时这一段出不来；按「那一刻活着的借据都还了」判，
//!   旧设备另有不在账上的句柄时会多给它占着的那一份）。
//!   不走这份账的设备在起之前就记下它在哪张卡上（[`BusesWithDevicesStartedOutsideTheBooks::start_a_device_marked_first`]，第 10y 批代码轮判决 K1-1
//!   攻方乙形：起好了才记，起设备到记下之间那一段读数里已经有那个设备的占用，被当成借据建上的）。
//!
//! 借的要比这张卡空着时能借的（同一个算法拿空卡上能用的当 M、B 当 0；空卡上能用的是 `nvidia-smi` 报的总量减驱动保留的那一份，
//! 2026-10-07 本机两张空卡上逐 MiB 等于报的空闲：5090 是 32607 − 497 = 32110，5060 Ti 是 16311 − 462 = 15849；第 10x 批代码轮判决 K1-1
//! 「上界不用总量」）还大，等多久都放不下，调用方判红（配了 GPU 就一定用 GPU）；
//! 比此刻能借的大而空着时放得下，是借不到：不等，调用方按借不到走下一条（几张卡一起借的换后面的卡，一张都借不到报本次未跑，
//! [`scratch_booking_refusal_outcome`]、[`GPU_NOT_RUN_MARKER`]）。
//! 账不防的：两个进程同一刻读到同一份空闲、一起借超了（各自只记自己借的）；一份 `nvidia-smi -q` 报告里空闲与这个进程占着的是驱动的两次查询，
//! 本机两张卡的两次读显存相隔约 1.3 毫秒（第 10y 批代码轮攻方量过 1347.9 µs），这个进程别的线程恰在两次之间建上的那一笔既在空闲里又在占着的里
//! （第 10y 批代码轮判决 K1-1：与两进程同读一份空闲同形，归运行时兜底，不另改）。这两形由运行时兜底：内存包装
//! `scripts/hosts/run-with-memory-cap.sh` 认出内核日志里的 `NVRM … NV_ERR_NO_MEMORY` 退 245，wgpu 报 Out of Memory 时
//! 崩溃放量产品路径与卡池起设备把这一趟报本次未跑（原因写显存不够，[`GPU_MEMORY_SHORT_REASON_PREFIX`]、
//! [`reason_for_running_out_of_gpu_memory`]）。
//!
//! 先登记、后建设备（这个进程里）：一个进程在一张卡上第一次借时，这一笔连同它的驱动那一份放得下才记、才把这张卡记成登记了；
//! 调用方拿到借据之后才在这张卡上起设备（[`GpuMemoryLedger::prepare_the_driver_share_before_starting_the_device`]），
//! 起好了记一笔（[`GpuMemoryLedger::record_the_device_started_on_the_card`]），从此驱动那一份算实际占在卡上。设备丢了要重起时，驱动那一份记回
//! 许出去而还没占上卡、这个进程占着的照分不出算（上一段），此刻放不下就不起（第 10y 批代码轮判决 K1-3：重起不另登记驱动那一份，
//! 新设备的上下文没留地方）；被拒之后新设备起好之前，每一次要起都照此刻再判一次（第 10z 批代码轮判决 K1-3 正推 S6：被拒之前已借到的
//! 并发用例再来起设备时不直接放行）。
//!
//! wgpu 在卡上报的 Out of Memory 记在设备上（[`DeviceConditionOfACard`]），按记的那条线程上正在做的那段活认（[`WorkOnTheCardOfThisThread`]）：
//! 别的线程上一次 Out of Memory 不把这条线程上真正的 panic 说成显存不够（第 10y 批代码轮判决 K1-3 攻方）；判器内核在活另起的线程上编，
//! 那几条线程上记的算在那一刻这个设备上在编内核的那几段活上（[`DeviceConditionOfACard::compile_for_the_work_of_this_thread`]）。
//! 崩溃放量产品路径在起过卡池的进程里也走这份账：第 ① 段每张装了状态的卡按装的状态数借草稿区（[`scratch_states_to_book_on_each_card`]，
//! 只记在那张卡上），几张卡当场各试一次（[`GpuMemoryLedger::book_states_on_the_cards_with_room`]），每张在锁里照此刻能借的压状态数，
//! 借不到的卡不装状态、它那一份留给后面的卡（[`place_states_on_the_cards_that_book_now`]）；第 ② 段核对与单元校验和在卡上建的缓冲
//! 照 [`mebibytes_of_buffers_held_at_once`] 算，借第一张放得下的。没起过卡池的进程照旧不进账，是哪几种调用方写在
//! `singlefs_checker_tier::crash_amplification` 的文件头。

use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};
use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};

/// 一个判器一次派活最多带几个状态：显存装得下的、调用方要的、这条流一共几个状态，三者取小，至少 1。
/// 草稿区按它开：状态数之外开的一个状态都用不上（一条 45 个状态的流按显存开满一张空着的 5090 是约 21 GiB，用得上的约 54 MiB）。
#[must_use]
pub fn states_in_one_dispatch_capped_by_the_state_count(
    states_the_memory_holds: usize,
    requested_states_per_dispatch: usize,
    state_count: u64,
) -> usize {
    let state_count_as_usize = usize::try_from(state_count).unwrap_or(usize::MAX);
    states_the_memory_holds
        .min(requested_states_per_dispatch)
        .min(state_count_as_usize)
        .max(1)
}

/// 一张卡上这个进程一共能占多少（MiB，这个进程自己的驱动那一份已经扣掉；减去它借着的才是这一笔能借的，文件头「口径」）。
/// 设了 `SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES`（`card_quota_from_the_environment_mebibytes`）就取三者里最小的：这个额度、
/// 配置给卡的额度减驱动的固定开销、这个进程此刻能在这张卡上占的减驱动的固定开销（第 10t 批 K4-1：额度变量不越过配置给卡的额度）。
/// 配置给卡的额度：`GPU_CARDS_BY_PRIORITY` 配给这张卡的（`configured_card_quota_mebibytes`）；没配显卡优先级（`None`）时由
/// 没配额度时默认的那一份代替，即 `budget_mebibytes`（那份显存的 3/4，第 10v 批：主 agent 定没配优先级时 3/4 那一份也并进最小值）。
/// 没设额度变量照旧是这张卡给崩溃放量的额度（`budget_mebibytes`，配置的额度与这个进程能占的里小的那个，或这个进程能占的 3/4）
/// 减驱动的固定开销（主 agent 2026-10-06 定：它是每张卡的上限，不是每条用例的上限，不判红）。
#[must_use]
pub fn card_ledger_capacity_mebibytes(
    card_quota_from_the_environment_mebibytes: Option<u64>,
    configured_card_quota_mebibytes: Option<u64>,
    budget_mebibytes: u64,
    memory_this_process_may_hold_mebibytes: u64,
    mebibytes_left_for_the_driver: u64,
) -> u64 {
    let default_share_without_a_configured_card_quota_mebibytes =
        match configured_card_quota_mebibytes {
            Some(_) => u64::MAX,
            None => budget_mebibytes.saturating_sub(mebibytes_left_for_the_driver),
        };
    match card_quota_from_the_environment_mebibytes {
        Some(quota) => quota.min(
            configured_card_quota_mebibytes
                .map_or(u64::MAX, |configured| {
                    configured.saturating_sub(mebibytes_left_for_the_driver)
                })
                .min(default_share_without_a_configured_card_quota_mebibytes)
                .min(
                    memory_this_process_may_hold_mebibytes
                        .saturating_sub(mebibytes_left_for_the_driver),
                ),
        ),
        None => budget_mebibytes.saturating_sub(mebibytes_left_for_the_driver),
    }
}

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
    /// 占的量里分不出哪一份是借据建的（第 10x 批代码轮判决 K1-1，攻方乙形）。
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
    /// 丢了的设备连它上面的缓冲可能还占着卡，照读不到实际占着的算；都放掉了才回到 [`Self::DeviceStarted`]（第 10z 批第六块，G2 乙：
    /// 拿掉「销了账、卡上可能还没还的」那一份之后，旧设备留在卡上的不再记成一份；第 10z 批第八块改按放没放掉判，不按那一刻活着的借据还没还判）。
    DeviceStartedAgainWhileTheLostDeviceRemains,
}

/// 这个进程在一张卡上记着的：活着的借据合计（MiB）与驱动那一份走到哪一步。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BookedByThisProcessOnACard {
    pub leases_mebibytes: u64,
    pub driver_share: DriverShareOnTheCard,
}

/// 这个进程此刻能在一张卡上占多少（MiB，文件头「口径」里的 M）：现读的空闲，加上这个进程实际占着的里属于它许出去的那一部分
/// （[`occupied_part_of_what_this_process_promised_mebibytes`]）。
#[must_use]
pub fn memory_this_process_may_hold_mebibytes(
    reading: CardMemoryReading,
    booked: BookedByThisProcessOnACard,
    mebibytes_left_for_the_driver: u64,
) -> u64 {
    reading
        .free_mebibytes
        .saturating_add(occupied_part_of_what_this_process_promised_mebibytes(
            reading,
            booked,
            mebibytes_left_for_the_driver,
        ))
}

/// 这个进程实际占在一张卡上的里，属于它许出去的那一部分（MiB；许出去的是活着的借据，加登记了时驱动那一份）：比许出去的多（分配器按块拿）
/// 只算到许出去的那么多；读不到实际占着的、这个进程在这张卡上另有不走这份账的设备、或这张卡上丢了的设备可能还占着
/// （[`DriverShareOnTheCard::DeviceLostNotStartedAgain`]、[`DriverShareOnTheCard::DeviceStartedAgainWhileTheLostDeviceRemains`]），
/// 是 0：借着的当一点都没建，驱动那一份也当还没占上卡（驱动实占多少、哪一份是旧设备的都分不出）。
/// 销了账的借据不在许出去的里，不再从实际占着的里扣：它的缓冲在销账之前已经交回驱动（等完了设备，或那台设备上提交的活后来做完了，
/// 文件头「口径」，G2 乙），或者它拿着的是丢了还没放掉的设备，那时这里本来就交 0、放掉之前缓冲交回驱动。
#[must_use]
pub fn occupied_part_of_what_this_process_promised_mebibytes(
    reading: CardMemoryReading,
    booked: BookedByThisProcessOnACard,
    mebibytes_left_for_the_driver: u64,
) -> u64 {
    let promised_mebibytes = match booked.driver_share {
        DriverShareOnTheCard::NotRegistered => booked.leases_mebibytes,
        DriverShareOnTheCard::RegisteredDeviceNotStarted | DriverShareOnTheCard::DeviceStarted => {
            booked
                .leases_mebibytes
                .saturating_add(mebibytes_left_for_the_driver)
        }
        // 旧设备留在卡上的那一堆读数里与新设备的分不出：当一点都不是许出去的（只会少借）
        DriverShareOnTheCard::DeviceLostNotStartedAgain
        | DriverShareOnTheCard::DeviceStartedAgainWhileTheLostDeviceRemains => return 0,
    };
    match reading.occupied_by_this_process {
        OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes } => {
            mebibytes.min(promised_mebibytes)
        }
        // 驱动那一份当整份在卡上，实占的 x 比 D 小时多给 D − x（第 10y 批代码轮判决 K1-1 攻方乙附）：驱动之后还会长到 D
        OccupiedByThisProcess::NotListed | OccupiedByThisProcess::AlsoOutsideTheBooks { .. } => 0,
    }
}

/// 一张卡此刻对这个进程：这一笔最多能借多少，与这张卡空着时最多能借多少（MiB）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CardRoomForThisProcess {
    pub room_now_mebibytes: u64,
    /// 同一个算法拿空卡上能用的（[`CardMemoryReading::total_less_reserved_mebibytes`]）当这个进程能占的、借着的当 0：
    /// 要的比它大，等多久都放不下。
    pub ceiling_mebibytes: u64,
}

/// 一张卡此刻能借给这个进程的这一笔（文件头「口径」）：`capacity_of_memory_this_process_may_hold` 把这个进程能占的
/// （[`memory_this_process_may_hold_mebibytes`]）换成它一共能占的（卡池照 [`card_ledger_capacity_mebibytes`] 算），减去借着的。
#[must_use]
pub fn card_room_for_this_process(
    reading: CardMemoryReading,
    booked: BookedByThisProcessOnACard,
    mebibytes_left_for_the_driver: u64,
    capacity_of_memory_this_process_may_hold: &dyn Fn(u64) -> u64,
) -> CardRoomForThisProcess {
    let may_hold_mebibytes =
        memory_this_process_may_hold_mebibytes(reading, booked, mebibytes_left_for_the_driver);
    CardRoomForThisProcess {
        room_now_mebibytes: capacity_of_memory_this_process_may_hold(may_hold_mebibytes)
            .saturating_sub(booked.leases_mebibytes),
        ceiling_mebibytes: capacity_of_memory_this_process_may_hold(
            reading.total_less_reserved_mebibytes,
        ),
    }
}

/// 借一次显存的结局（[`decide_booking_by_priority`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BookingDecision {
    /// 记在这张卡上：照卡的先后，第一张此刻放得下的卡（值是它在给的表里的次序）。
    BookOnTheCardAt { card_position: usize },
    /// 此刻每张卡都放不下，而至少一张卡空着时放得下：借不到（不等）。
    NoRoomNow,
    /// 要的比每张卡空着时能借的都大（一张卡都没有也算）：等多久都放不下。
    NeverFits { largest_ceiling_mebibytes: u64 },
}

/// 一次借 `need_mebibytes` 记在哪张卡上：`rooms` 是每张卡此刻对这个进程的，次序就是优先级。
#[must_use]
pub fn decide_booking_by_priority(
    rooms: &[CardRoomForThisProcess],
    need_mebibytes: u64,
) -> BookingDecision {
    let largest_ceiling_mebibytes = rooms
        .iter()
        .map(|room| room.ceiling_mebibytes)
        .max()
        .unwrap_or(0);
    if need_mebibytes > largest_ceiling_mebibytes {
        return BookingDecision::NeverFits {
            largest_ceiling_mebibytes,
        };
    }
    match rooms
        .iter()
        .position(|room| need_mebibytes <= room.room_now_mebibytes)
    {
        Some(card_position) => BookingDecision::BookOnTheCardAt { card_position },
        None => BookingDecision::NoRoomNow,
    }
}

/// 要的比每张卡空着时能借的都大：配了 GPU 就一定用 GPU，调用方据此判红，不悄悄换小。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NeedLargerThanEveryCard {
    pub need_mebibytes: u64,
    pub largest_ceiling_mebibytes: u64,
}

/// 在 `0..=most_states` 里找最多的状态数，它要的显存（`mebibytes_for_states`，MiB）不超过 `capacity_mebibytes`：
/// 0 个状态不借、恒放得下；`mebibytes_for_states` 要随状态数单调不减（判器按状态数借的那一族都是）。二分，最多问 65 次。
#[must_use]
pub fn most_states_whose_need_fits_the_capacity(
    most_states: usize,
    capacity_mebibytes: u64,
    mebibytes_for_states: &dyn Fn(usize) -> u64,
) -> usize {
    if most_states == 0 || mebibytes_for_states(most_states) <= capacity_mebibytes {
        return most_states;
    }
    // 不变量：`fitting` 放得下（0 恒放得下），`not_fitting` 放不下；两者之间每轮砍半
    let mut fitting = 0usize;
    let mut not_fitting = most_states;
    while not_fitting - fitting > 1 {
        let middle = fitting + (not_fitting - fitting) / 2;
        if mebibytes_for_states(middle) <= capacity_mebibytes {
            fitting = middle;
        } else {
            not_fitting = middle;
        }
    }
    fitting
}

/// 产品路径在一张卡上当场借草稿区的结局（[`place_states_on_the_cards_that_book_now`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScratchAttemptOnACard {
    /// 前面的卡已经装下了全部要判的状态：没去借。调用方算出这张卡一个状态都带不了的不是这一种，是下面两种之一
    /// （第 10x 批代码轮判决 K2-2：「此刻带不了」不并进「没有要装的」，否则一张都借不到时落进判红那一支）。
    NothingLeftToPlace,
    /// 借到了 `states` 个状态的草稿区（在锁里照此刻能借的压过，不多于要装的）。
    Booked { states: u64 },
    /// 此刻连一个状态都放不下（这个进程别的借据或卡上别的用户占着），空着时放得下：这张卡这一趟不装状态；
    /// `need_mebibytes` 是一个状态要借的量。
    NoRoomNow { need_mebibytes: u64 },
    /// 这张卡空着时连一个状态的草稿区都放不下，等多久都一样。
    CannotHoldOneState { ceiling_mebibytes: u64 },
}

/// 产品路径按卡的先后把 `pending_states` 个状态装到此刻借得到的卡上：每张卡要装的是剩下的与它带得了的（`states_each_card_holds`）
/// 里小的那个，交 `attempt_now(卡在表里的次序, 要装的状态数)` 当场借一次；借到几个就从剩下的里减几个，
/// 借不到的卡不装，它那一份留给后面的卡。装满了的卡之后剩下的状态排队等前面的卡判完（不算在任何一张卡上），
/// 后面的卡没有要装的，不去借（[`ScratchAttemptOnACard::NothingLeftToPlace`]）。还有要装的而调用方算出这张卡一个状态都带不了，
/// 不去借，交 `classify_a_card_holding_no_state(卡在表里的次序)` 判是此刻带不了还是空着时也带不了。
/// 交回每张卡的结局，与 `states_each_card_holds` 逐张对上。
///
/// # Panics
/// `attempt_now` 交回的借到的状态数是 0、或比要装的多；`classify_a_card_holding_no_state` 交回的不是此刻借不到、空着时也放不下两种之一。
pub fn place_states_on_the_cards_that_book_now(
    pending_states: u64,
    states_each_card_holds: &[u64],
    attempt_now: &mut dyn FnMut(usize, u64) -> ScratchAttemptOnACard,
    classify_a_card_holding_no_state: &mut dyn FnMut(usize) -> ScratchAttemptOnACard,
) -> Vec<ScratchAttemptOnACard> {
    let mut states_left_to_place = pending_states;
    let mut attempts = Vec::with_capacity(states_each_card_holds.len());
    for (card_slot, states_the_card_holds) in states_each_card_holds.iter().enumerate() {
        if states_left_to_place == 0 {
            attempts.push(ScratchAttemptOnACard::NothingLeftToPlace);
            continue;
        }
        if *states_the_card_holds == 0 {
            let classified = classify_a_card_holding_no_state(card_slot);
            match classified {
                ScratchAttemptOnACard::NoRoomNow { .. }
                | ScratchAttemptOnACard::CannotHoldOneState { .. } => {}
                ScratchAttemptOnACard::NothingLeftToPlace
                | ScratchAttemptOnACard::Booked { .. } => {
                    panic!(
                        "第 {card_slot} 张卡一个状态都带不了，判出来的是 {classified:?}：只能是此刻借不到或空着时也放不下"
                    )
                }
            }
            attempts.push(classified);
            continue;
        }
        let states_to_place = states_left_to_place.min(*states_the_card_holds);
        let attempt = attempt_now(card_slot, states_to_place);
        match attempt {
            ScratchAttemptOnACard::Booked { states } => {
                assert!(
                    states > 0 && states <= states_to_place,
                    "第 {card_slot} 张卡要装 {states_to_place} 个状态，借到的是 {states} 个：借到的要在 1 到要装的之间"
                );
                states_left_to_place -= states;
            }
            ScratchAttemptOnACard::NothingLeftToPlace
            | ScratchAttemptOnACard::NoRoomNow { .. }
            | ScratchAttemptOnACard::CannotHoldOneState { .. } => {}
        }
        attempts.push(attempt);
    }
    attempts
}

/// 一张此刻连一个状态（`need_of_one_state_mebibytes`）都借不出的卡算哪一种：空着时也放不下是 [`ScratchAttemptOnACard::CannotHoldOneState`]，
/// 空着时放得下是此刻借不到（[`ScratchAttemptOnACard::NoRoomNow`]）。
#[must_use]
pub fn attempt_on_a_card_without_room_for_one_state(
    need_of_one_state_mebibytes: u64,
    room: CardRoomForThisProcess,
) -> ScratchAttemptOnACard {
    if need_of_one_state_mebibytes > room.ceiling_mebibytes {
        return ScratchAttemptOnACard::CannotHoldOneState {
            ceiling_mebibytes: room.ceiling_mebibytes,
        };
    }
    ScratchAttemptOnACard::NoRoomNow {
        need_mebibytes: need_of_one_state_mebibytes,
    }
}

/// 每张卡当场试过一次之后怎么办（[`decide_after_attempting_every_card`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScratchPlacementDecision {
    /// 至少一张卡借到了：就在借到的那几张上判。
    JudgeOnTheCardsBooked,
    /// 一张都没借到，而至少一张卡空着时放得下：借不到（不等），调用方报本次未跑（`need_mebibytes` 是第一张放不下的卡要借的量）。
    NoRoomOnAnyCardNow { need_mebibytes: u64 },
    /// 一张都没借到，每张卡空着时连一个状态都放不下：等多久都一样，调用方判红。
    NoCardHoldsOneState { largest_ceiling_mebibytes: u64 },
}

/// 一张借到了就判；一张都没借到时，有空着时放得下的就是此刻借不到，没有就判成没有一张卡装得下一个状态。
#[must_use]
pub fn decide_after_attempting_every_card(
    attempts: &[ScratchAttemptOnACard],
) -> ScratchPlacementDecision {
    let mut first_need_without_room = None;
    let mut largest_ceiling_mebibytes = 0u64;
    for attempt in attempts {
        match attempt {
            ScratchAttemptOnACard::Booked { .. } => {
                return ScratchPlacementDecision::JudgeOnTheCardsBooked;
            }
            ScratchAttemptOnACard::NoRoomNow { need_mebibytes } => {
                first_need_without_room = first_need_without_room.or(Some(*need_mebibytes));
            }
            ScratchAttemptOnACard::CannotHoldOneState { ceiling_mebibytes } => {
                largest_ceiling_mebibytes = largest_ceiling_mebibytes.max(*ceiling_mebibytes);
            }
            ScratchAttemptOnACard::NothingLeftToPlace => {}
        }
    }
    match first_need_without_room {
        Some(need_mebibytes) => ScratchPlacementDecision::NoRoomOnAnyCardNow { need_mebibytes },
        None => ScratchPlacementDecision::NoCardHoldsOneState {
            largest_ceiling_mebibytes,
        },
    }
}

/// 产品路径第 ① 段借草稿区之前怎么走（[`decide_scratch_before_booking`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScratchBeforeBooking {
    /// 至少一张卡此刻带得了一个状态：去借。
    BookOnTheCardsThatHoldAStateNow,
    /// 每张卡此刻都带不了一个状态，至少一张空着时带得了：此刻借不到，调用方报本次未跑（不等、不判红）。
    NoCardHoldsAStateNow,
    /// 每张卡空着时也带不了一个状态（一张卡都没有也算）：等多久都一样，调用方判红。
    NoCardHoldsAStateEvenWhenEmpty,
}

/// 照每张卡此刻带得了几个状态（`states_each_card_holds_now`）与空着时带得了几个（`states_each_card_holds_when_empty`，逐张对上）
/// 定借草稿区之前怎么走（第 10x 批代码轮判决 K2-2：每张卡此刻都放不下一个状态而空着时放得下，是借不到，不是放不下）。
///
/// # Panics
/// 两张表的卡数不同。
#[must_use]
pub fn decide_scratch_before_booking(
    states_each_card_holds_now: &[u64],
    states_each_card_holds_when_empty: &[u64],
) -> ScratchBeforeBooking {
    assert_eq!(
        states_each_card_holds_now.len(),
        states_each_card_holds_when_empty.len(),
        "此刻与空着时两张表要逐张对上"
    );
    if states_each_card_holds_now.iter().any(|states| *states > 0) {
        return ScratchBeforeBooking::BookOnTheCardsThatHoldAStateNow;
    }
    if states_each_card_holds_when_empty
        .iter()
        .any(|states| *states > 0)
    {
        return ScratchBeforeBooking::NoCardHoldsAStateNow;
    }
    ScratchBeforeBooking::NoCardHoldsAStateEvenWhenEmpty
}

/// 一条线程上同时还活着几趟派活的缓冲：wgpu 丢掉的缓冲要等设备下一次收拾才还给驱动，这一趟的缓冲建起来时
/// 上一趟丢掉的可能还占着（推的，没上卡量过）。
pub const PASSES_ALIVE_AT_ONCE_ON_EACH_THREAD: u64 = 2;

/// 一段在卡上建的缓冲合起来占多少显存（MiB，向上取整）：上卡一次、整段都在的 `bytes_uploaded_once`，
/// 加每一趟派活建、派完就丢的 `bytes_of_one_pass` 乘同时还活着的趟数 `passes_alive_at_once`
/// （几条线程同时派活时是线程数乘 [`PASSES_ALIVE_AT_ONCE_ON_EACH_THREAD`]）。
#[must_use]
pub fn mebibytes_of_buffers_held_at_once(
    bytes_uploaded_once: u64,
    bytes_of_one_pass: u64,
    passes_alive_at_once: u64,
) -> u64 {
    bytes_uploaded_once
        .saturating_add(bytes_of_one_pass.saturating_mul(passes_alive_at_once))
        .div_ceil(1024 * 1024)
}

/// 崩溃放量产品路径（`singlefs_checker_tier::crash_amplification`）一趟用的卡从哪来；草稿区进不进显存账也照它
/// （[`scratch_states_to_book_on_each_card`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GpuCardsSource {
    /// 这一趟没有一类计算走 GPU，不要卡。
    NoCardNeeded,
    /// 这个进程起过 GPU 用例共用的卡池（`singlefs_checker_tier::gpu_unit_checks::gpu_test_start::GpuCardPool`）：照池里的卡用，
    /// 每张卡这个进程只有一个设备、只在借到的那张卡上起；第 ① 段判器开的草稿区按装到这张卡上的状态数借
    /// （第 10v 批：主 agent 定照 state_count 记）。
    TheCardPoolOfThisProcess,
    /// 这个进程没起过卡池：这一趟自己在点名的卡上起设备（`gpu_unit_checks::gpu_cards_by_priority` 或 `usable_gpu_cards`），不进显存账。
    DevicesStartedForThisRun,
}

/// 崩溃放量产品路径第 ① 段每张卡的草稿区在显存账上记几个状态。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScratchStatesToBook {
    /// 不进账：这一趟不要卡，或这个进程没起过卡池、自己起的设备（[`GpuCardsSource::DevicesStartedForThisRun`]）。
    NotBooked,
    /// 起过卡池：按卡的次序每张卡借几个状态的草稿区（0 是这张卡没装状态、不借）。
    OnEachCard(Vec<usize>),
}

/// 产品路径第 ① 段的草稿区记几个状态的账：起过卡池的进程，每张装了状态的卡记它装的状态数，与这条流的状态数取小
/// （[`states_in_one_dispatch_capped_by_the_state_count`]，与 GPU 对拍用例借卡那一路同一个封顶：判器的草稿区不为这条流没有的状态开）；
/// 没起过卡池的照旧不进账。调用方照这里的状态数按 `crash_judge_dispatch::mebibytes_a_judge_needs` 算显存去借。
#[must_use]
pub fn scratch_states_to_book_on_each_card(
    cards_taken_from: GpuCardsSource,
    states_placed_on_each_card: &[u64],
    state_count: u64,
) -> ScratchStatesToBook {
    match cards_taken_from {
        GpuCardsSource::NoCardNeeded | GpuCardsSource::DevicesStartedForThisRun => {
            ScratchStatesToBook::NotBooked
        }
        GpuCardsSource::TheCardPoolOfThisProcess => ScratchStatesToBook::OnEachCard(
            states_placed_on_each_card
                .iter()
                .map(|states_placed| {
                    if *states_placed == 0 {
                        return 0;
                    }
                    let states_placed_as_usize =
                        usize::try_from(*states_placed).unwrap_or(usize::MAX);
                    states_in_one_dispatch_capped_by_the_state_count(
                        states_placed_as_usize,
                        states_placed_as_usize,
                        state_count,
                    )
                })
                .collect(),
        ),
    }
}

/// GPU 用例与产品路径报「本次未跑」那一行的记号（`singlefs_checker_tier::gpu_unit_checks::gpu_test_start::not_run_line` 照它打；
/// 门禁 `checker-tier-layer0-replay` 的 feature-gated-tests 格按它逐条报本次未跑）。
pub const GPU_NOT_RUN_MARKER: &str = "NOT_RUN_NO_GPU_ADAPTER";

/// 「本次未跑」那一行：`<记号> test=<用例或产品路径的名字> reason=<为什么>`。
#[must_use]
pub fn gpu_not_run_line(test_name: &str, reason: &str) -> String {
    format!("{GPU_NOT_RUN_MARKER} test={test_name} reason={reason}")
}

/// 显存不够而报「本次未跑」时原因的开头（借不到、卡上报 Out of Memory 都是它）：分项驱动与门禁按 [`GPU_NOT_RUN_MARKER`] 认这一行，
/// 人照这几个字认出是显存不够（第 10x 批代码轮判决 K1-3：记号照 `NOT_RUN_NO_GPU_ADAPTER` 一族，原因写显存不够）。
pub const GPU_MEMORY_SHORT_REASON_PREFIX: &str = "显存不够：";

/// 此刻借不到的那一句（GPU 用例借卡与产品路径借缓冲同一句）。
#[must_use]
pub fn reason_for_a_booking_without_room_now(
    need_mebibytes: u64,
    card_rooms_mebibytes: &[u64],
) -> String {
    format!(
        "{GPU_MEMORY_SHORT_REASON_PREFIX}此刻池里没有一张卡借得出 {need_mebibytes} MiB（这个进程看每张卡此刻能借的 {card_rooms_mebibytes:?}：\
         现读的空闲减这个进程借了还没占的；卡上别的 GPU 用户或这个进程里别的用例占着）"
    )
}

/// wgpu 在卡上报 Out of Memory 时「本次未跑」那一句（`what_ran_out` 说是哪一段、哪张卡）：账只记这个进程借的，
/// 两个进程同一刻读到同一份空闲、一起借超了的那一形由这一句兜底（文件头）。
#[must_use]
pub fn reason_for_running_out_of_gpu_memory(what_ran_out: &str) -> String {
    format!(
        "{GPU_MEMORY_SHORT_REASON_PREFIX}{what_ran_out}上 wgpu 报了 Out of Memory（账只记这个进程借的，卡上别的进程同一刻借走了；判过的块在库里）"
    )
}

/// 产品路径借缓冲借不到的结局（[`scratch_booking_refusal_outcome`]）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScratchBookingRefusalOutcome {
    /// 空着时放得下而此刻借不到：打这一行，这一趟照产品路径的「本次未跑」交回（判过的块在库里，没判的留给下一趟）。
    NotRun { not_run_line: String },
    /// 要的比那张卡空着时能借的都大、卡上的显存读不出：判红。
    Red { reason: String },
}

/// 产品路径借缓冲被拒时怎么报：此刻借不到照 GPU 对拍用例那一路报「本次未跑」（同一个记号、同一句），别的判红。
#[must_use]
pub fn scratch_booking_refusal_outcome(
    run_name: &str,
    card_position: usize,
    refusal: &BookingRefusal,
) -> ScratchBookingRefusalOutcome {
    match refusal {
        BookingRefusal::NoCardHasRoomNow {
            need_mebibytes,
            card_rooms_mebibytes,
        } => ScratchBookingRefusalOutcome::NotRun {
            not_run_line: gpu_not_run_line(
                run_name,
                &reason_for_a_booking_without_room_now(*need_mebibytes, card_rooms_mebibytes),
            ),
        },
        BookingRefusal::NeedLargerThanEveryCard(refusal) => ScratchBookingRefusalOutcome::Red {
            reason: format!(
                "{run_name}：第 {card_position} 张卡要借 {} MiB，那张卡空着时也只借得出 {} MiB",
                refusal.need_mebibytes, refusal.largest_ceiling_mebibytes
            ),
        },
        BookingRefusal::CardMemoryUnreadable { reason } => ScratchBookingRefusalOutcome::Red {
            reason: format!("{run_name}：卡上的显存读不出：{reason}"),
        },
    }
}

/// 借不到的几种结局。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BookingRefusal {
    /// 要的比每张卡空着时能借的都大：调用方判红。
    NeedLargerThanEveryCard(NeedLargerThanEveryCard),
    /// 空着时放得下而此刻每张卡都借不出：调用方报「本次未跑」（不等）。带着这一刻每张卡能借的。
    NoCardHasRoomNow {
        need_mebibytes: u64,
        card_rooms_mebibytes: Vec<u64>,
    },
    /// 卡上的显存读不出、读到的卡数与池对不上：调用方判红。
    CardMemoryUnreadable { reason: String },
}

/// 读每张卡此刻的显存（次序与卡池一样）：卡池每读一次跑一次 `nvidia-smi`（[`card_memory_reader_of_one_report_per_reading`]），单测换成假的。
pub type CardMemoryReader = Box<dyn Fn() -> Result<Vec<CardMemoryReading>, String> + Send + Sync>;

/// 一次读出每张卡的显存与每个进程占的那一份报告：`nvidia-smi` 带这几个参数（文本版，每张卡一段：`FB Memory Usage` 下的
/// `Total`、`Reserved`、`Free`，`Processes` 下每个进程的 `Process ID` 与 `Used GPU Memory`）。一次调用里两样都有，空闲与这个进程占着的
/// 是同一次读的（`--query-gpu` 与 `--query-compute-apps` 不能在一次调用里一起用，`nvidia-smi` 报「Only one --query-* switch」）。
/// 格式照本机驱动 595.84 的 `nvidia-smi` 文本模板（`strings` 读出的 `Process ID : %u`、`Used GPU Memory : %llu MiB`）。
pub const NVIDIA_SMI_MEMORY_AND_PROCESSES_REPORT_ARGUMENTS: [&str; 3] = ["-q", "-d", "MEMORY,PIDS"];

/// PCI 总线号只留 `总线:设备.功能` 那一段、小写：`00000000:01:00.0` 与 `0000:01:00.0` 认成同一张卡（卡池按它对上 wgpu 的适配器与
/// `nvidia-smi` 报的卡）。
#[must_use]
pub fn pci_bus_identifier_without_the_domain(text: &str) -> String {
    let lowered = text.trim().to_ascii_lowercase();
    let parts: Vec<&str> = lowered.split(':').collect();
    parts[parts.len().saturating_sub(2)..].join(":")
}

/// 一份报告里一张卡那一段读到的。
#[derive(Default)]
struct OneCardOfTheReport {
    total_mebibytes: Option<u64>,
    reserved_mebibytes: Option<u64>,
    free_mebibytes: Option<u64>,
    /// 这个进程那几行占的合计；列了这个进程而有一行读不出是 `Err`。
    used_by_this_process_mebibytes: Option<Result<u64, ()>>,
}

/// 报告里一张卡那一段眼下读到哪一小节。
#[derive(Clone, Copy, PartialEq, Eq)]
enum SectionOfTheReport {
    FrameBufferMemory,
    Processes,
    Other,
}

/// `32607 MiB` 那样的量（MiB）；读不出交 `None`。
fn mebibytes_of_a_report_value(value: &str) -> Option<u64> {
    value.strip_suffix("MiB")?.trim().parse().ok()
}

/// 一份 `nvidia-smi -q -d MEMORY,PIDS` 报告（[`NVIDIA_SMI_MEMORY_AND_PROCESSES_REPORT_ARGUMENTS`]）里 `buses` 这几张卡此刻的显存
/// （与 `buses` 逐张对上，`buses` 照 [`pci_bus_identifier_without_the_domain`] 写）：空闲、空卡上能用的（总量减保留），
/// 与进程号是 `this_process_identifier` 的那几行占的合计。这个进程在一张卡上另有不走这份账的设备
/// （`has_devices_outside_the_books_on_the_bus(总线号)` 为真）时，占的那一份记成 [`OccupiedByThisProcess::AlsoOutsideTheBooks`]。
///
/// # Errors
/// 报告里没有某张卡、那张卡的总量、保留、空闲读不出，或一行进程号认不出。
pub fn card_memory_readings_of_one_nvidia_smi_report(
    report: &str,
    this_process_identifier: u32,
    buses: &[String],
    has_devices_outside_the_books_on_the_bus: &dyn Fn(&str) -> bool,
) -> Result<Vec<CardMemoryReading>, String> {
    let mut cards: BTreeMap<String, OneCardOfTheReport> = BTreeMap::new();
    let mut current_bus: Option<String> = None;
    let mut section = SectionOfTheReport::Other;
    let mut the_listed_process_is_this_one = false;
    for line in report.lines() {
        if let Some(bus) = line.strip_prefix("GPU ") {
            let bus = pci_bus_identifier_without_the_domain(bus);
            cards.insert(bus.clone(), OneCardOfTheReport::default());
            current_bus = Some(bus);
            section = SectionOfTheReport::Other;
            the_listed_process_is_this_one = false;
            continue;
        }
        let Some(card) = current_bus.as_ref().and_then(|bus| cards.get_mut(bus)) else {
            continue;
        };
        let indentation = line.len() - line.trim_start().len();
        let (key, value) = match line.split_once(':') {
            Some((key, value)) => (key.trim(), value.trim()),
            None => (line.trim(), ""),
        };
        if indentation <= 4 {
            section = match key {
                "FB Memory Usage" => SectionOfTheReport::FrameBufferMemory,
                "Processes" => SectionOfTheReport::Processes,
                _ => SectionOfTheReport::Other,
            };
            continue;
        }
        match (section, key) {
            (SectionOfTheReport::FrameBufferMemory, "Total") => {
                card.total_mebibytes = mebibytes_of_a_report_value(value);
            }
            (SectionOfTheReport::FrameBufferMemory, "Reserved") => {
                card.reserved_mebibytes = mebibytes_of_a_report_value(value);
            }
            (SectionOfTheReport::FrameBufferMemory, "Free") => {
                card.free_mebibytes = mebibytes_of_a_report_value(value);
            }
            (SectionOfTheReport::Processes, "Process ID") => {
                let process_identifier: u32 = value
                    .parse()
                    .map_err(|_| format!("nvidia-smi 的报告里一行进程号认不出：{line:?}"))?;
                the_listed_process_is_this_one = process_identifier == this_process_identifier;
            }
            (SectionOfTheReport::Processes, "Used GPU Memory")
                if the_listed_process_is_this_one =>
            {
                card.used_by_this_process_mebibytes = Some(
                    match (
                        card.used_by_this_process_mebibytes,
                        mebibytes_of_a_report_value(value),
                    ) {
                        (None, Some(used)) => Ok(used),
                        (Some(Ok(earlier)), Some(used)) => Ok(earlier.saturating_add(used)),
                        (Some(Err(())), Some(_)) | (_, None) => Err(()),
                    },
                );
            }
            (
                SectionOfTheReport::FrameBufferMemory
                | SectionOfTheReport::Processes
                | SectionOfTheReport::Other,
                _,
            ) => {}
        }
    }
    buses
        .iter()
        .map(|bus| {
            let card = cards
                .get(bus)
                .ok_or_else(|| format!("nvidia-smi 的报告里没有总线 {bus} 那张卡"))?;
            let unreadable =
                |what: &str| format!("nvidia-smi 的报告里总线 {bus} 那张卡的 {what} 读不出");
            let total_mebibytes = card.total_mebibytes.ok_or_else(|| unreadable("Total"))?;
            let reserved_mebibytes = card
                .reserved_mebibytes
                .ok_or_else(|| unreadable("Reserved"))?;
            let free_mebibytes = card.free_mebibytes.ok_or_else(|| unreadable("Free"))?;
            let occupied_by_this_process = match card.used_by_this_process_mebibytes {
                None | Some(Err(())) => OccupiedByThisProcess::NotListed,
                Some(Ok(mebibytes)) => {
                    if has_devices_outside_the_books_on_the_bus(bus) {
                        OccupiedByThisProcess::AlsoOutsideTheBooks { mebibytes }
                    } else {
                        OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes }
                    }
                }
            };
            Ok(CardMemoryReading {
                free_mebibytes,
                total_less_reserved_mebibytes: total_mebibytes.saturating_sub(reserved_mebibytes),
                occupied_by_this_process,
            })
        })
        .collect()
}

/// 卡池读显存的那一份：每读一次照 `run_the_report` 取一份报告（卡池是跑一次 `nvidia-smi`），整张表照这一份读
/// （[`card_memory_readings_of_one_nvidia_smi_report`]）：空闲与这个进程占着的不分两次读（文件头）。
#[must_use]
pub fn card_memory_reader_of_one_report_per_reading(
    run_the_report: Box<dyn Fn() -> Result<String, String> + Send + Sync>,
    this_process_identifier: u32,
    buses: Vec<String>,
    has_devices_outside_the_books_on_the_bus: Box<dyn Fn(&str) -> bool + Send + Sync>,
) -> CardMemoryReader {
    Box::new(move || {
        let report = run_the_report()?;
        card_memory_readings_of_one_nvidia_smi_report(
            &report,
            this_process_identifier,
            &buses,
            &*has_devices_outside_the_books_on_the_bus,
        )
    })
}

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
    /// 缓冲可能还在卡上（第 10z 批第十一块，第 10z 批第七八九块代码轮判决 K1-1 攻方：poll 之后、销账之前另一笔刚记成没等成，一起销掉，账多给）。
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
    /// 在锁里现读一次：借着的连新设备的驱动那一份照此刻的空闲放不下，交借不到，调用方不起（第 10y 批代码轮判决 K1-3 攻方：重起不另登记驱动那一份，
    /// 触发重起的那一笔是按驱动那一份已在卡上借的，新设备的上下文没留地方）。交借不到之后驱动那一份照旧记着许出去而没占上卡
    /// （[`DriverShareOnTheCard::DeviceLostNotStartedAgain`]），新设备起好之前每一次再来都照此刻再判一次（第 10z 批代码轮判决 K1-3 正推 S6：
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
    /// 只销列出来的那几笔，[`DeviceStartedOnTheBooks::take_in_that_the_work_of_the_listed_leases_finished`]；第 10z 批第十一块，
    /// 第 10z 批第七八九块代码轮判决 K1-1；留着的设备丢了的告诉账，[`Self::take_in_the_devices_kept_before_booking`]）。按卡在池里的先后。
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

    /// 卡池借之前那一步（`singlefs_checker_tier::gpu_unit_checks::gpu_test_start::GpuCardPool` 借之前只调它）：先列卡
    /// （[`Self::leases_kept_after_a_failed_wait_on_each_card`]），再对池里每张卡取卡池那一格留着的设备（`device_kept_on_the_card`，
    /// 不等正在起的设备；没有的跳过）：
    /// - 丢了（`is_lost`）：不 poll（wgpu 在丢了的设备上 poll 会 panic），告诉账这台设备丢了，照重起那一套记
    ///   （[`DeviceStartedOnTheBooks::take_in_that_the_device_was_found_lost`]：它上面没等成的那几笔销账、这张卡照分不出算）。
    ///   不等借到之后走到重起才记：那几笔记着的量让后面的借都放不下时，借不到就走不到重起，这张卡在这个进程里再也借不出
    ///   （第 10z 批第十三块，第 10z 批第十、十一块代码轮判决 K4-2，正推账层探针：丢了之后能借的 1 000、借 4 000 被拒）。
    ///   没有没等成借据的卡（列不出来）也看：丢了的设备的驱动那一份与缓冲还占着卡，账不知道丢了时照读数把它们算成这个进程许出去的、
    ///   多给，借走之后重起时照此刻再判被拒、报本次未跑（第 10z 批第十五块，第 10z 批第十二、十三块代码轮判决 K3-3，正推 T4：
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

/// 列卡那一刻一张卡上等设备没等成而照旧记着的那几笔（[`GpuMemoryLedger::leases_kept_after_a_failed_wait_on_each_card`]）：
/// 卡池列卡之后对这张卡留着的设备 poll，交回队列空了只销这几笔（[`DeviceStartedOnTheBooks::take_in_that_the_work_of_the_listed_leases_finished`]）。
/// 列卡之后才记成没等成的那一笔不在里面：它的活可以是 poll 之后才提交的。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeasesKeptAfterAFailedWaitWhenListed {
    card_position: usize,
    /// 借据号 → 它等的那一台。
    leases_kept_on_each_device: BTreeMap<u64, DeviceNumberOnTheCard>,
}

impl LeasesKeptAfterAFailedWaitWhenListed {
    /// 在池里第几张卡上。
    #[must_use]
    pub fn card_position(&self) -> usize {
        self.card_position
    }

    /// 列出来的借据号（从小到大）。
    #[must_use]
    pub fn lease_identifiers(&self) -> Vec<u64> {
        self.leases_kept_on_each_device.keys().copied().collect()
    }
}

/// 按状态数在一张卡上借到的草稿区：借了几个状态（在锁里照此刻能借的压过）与那一笔借据（丢掉时还）。
pub struct StatesBookedOnACard<'ledger> {
    pub states: u64,
    pub booking: GpuMemoryBooking<'ledger>,
}

/// 账上的一笔：记在哪张卡、多少 MiB、记的那一刻这张卡能借给这个进程的；丢掉时销账（等设备没等成的照旧记着，
/// [`Self::take_in_the_wait_before_returning`]）。
pub struct GpuMemoryBooking<'ledger> {
    ledger: &'ledger GpuMemoryLedger,
    card_position: usize,
    mebibytes: u64,
    lease_identifier: u64,
    card_room_mebibytes_when_booked: u64,
    when_dropped: LeaseWhenTheBookingIsDropped,
}

/// 借据丢掉之前等设备的结局（卡池的借据丢掉时等这张卡的设备把提交了的活都做完，`singlefs_checker_tier::gpu_unit_checks::gpu_test_start::GpuCardLease`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviceWaitBeforeReturningTheLease {
    /// 提交了的活都做完了：丢掉的缓冲已经交回驱动，可以销账。
    TheSubmittedWorkFinished,
    /// 没等成（超时、wgpu 报错或 panic）：缓冲可能还占着卡，等的是在用的那一台时这一笔先不销账（文件头「口径」）。
    TheWaitFailed,
}

/// 一笔借据丢掉时账上怎么办。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LeaseWhenTheBookingIsDropped {
    /// 销账（等完设备了，这一笔没在卡上建过缓冲，或等的是丢了还没放掉的那一台）。
    ReturnedToTheDriverAndOffTheBooks,
    /// 等在用的那一台没等成：照旧记着，那一台上提交的活后来做完了、记成丢了、或放掉了时销（`BooksOfACard::leases_kept_after_a_failed_wait`）。
    KeptOnTheBooksAfterAFailedWait,
}

impl GpuMemoryBooking<'_> {
    /// 记在池里第几张卡上。
    #[must_use]
    pub fn card_position(&self) -> usize {
        self.card_position
    }

    #[must_use]
    pub fn mebibytes(&self) -> u64 {
        self.mebibytes
    }

    /// 这一笔的借据号（[`GpuMemoryLedger::is_the_lease_on_the_books`] 照它查）。
    #[must_use]
    pub fn lease_identifier(&self) -> u64 {
        self.lease_identifier
    }

    /// 记的那一刻这张卡能借给这个进程的（MiB，这一笔之前）。
    #[must_use]
    pub fn card_room_mebibytes_when_booked(&self) -> u64 {
        self.card_room_mebibytes_when_booked
    }

    /// 这一笔此刻还记在账上没有（[`GpuMemoryLedger::is_the_lease_on_the_books`]；借据活着时恒为真，核账用）。
    #[must_use]
    pub fn is_on_the_books(&self) -> bool {
        self.ledger
            .is_the_lease_on_the_books(self.card_position, self.lease_identifier)
    }

    /// 丢掉这一笔之前等过设备（卡池的借据丢掉时调，第 10z 批第六块，用户 2026-10-08 定 G2「改用乙」；`device` 是这笔借据那张卡上的设备）：
    /// 等完了，丢掉时照常销账，同一台上之前没等成而照旧记着的那几笔也一起销（这一次等的是到此刻为止的全部提交，它们的活也做完了）；
    /// 没等成而等的是在用的那一台，这一笔丢掉时不销账、照旧算借着，到那一台上提交的活后来做完了、或那一台记成丢了、放掉了才销
    /// （文件头「口径」：账不多给，代价是这一段少借这一笔）；没等成而等的是丢了还没放掉的那一台，照常销账（这个进程占着的那时照分不出算，
    /// 它的缓冲在那一台放掉之前交回驱动，第 10z 批第八块，第 10z 批第六块代码轮判决 K1-3）。
    ///
    /// # Panics
    /// `device` 不是这本账、这张卡上的（别处已经在 panic 时不断言：丢借据是收尾的一部分，两次 panic 会让整个进程中止）。
    pub fn take_in_the_wait_before_returning(
        &mut self,
        device: &DeviceStartedOnTheBooks<'_>,
        wait: DeviceWaitBeforeReturningTheLease,
    ) {
        assert!(
            (std::ptr::eq(device.ledger, self.ledger) && device.card_position == self.card_position)
                || std::thread::panicking(),
            "第 {} 张卡上的借据号 {} 等的设备记在第 {} 张卡上（或别的账上）：借据只等它那张卡上的设备",
            self.card_position,
            self.lease_identifier,
            device.card_position
        );
        let mut books = self.ledger.books_locked();
        let books_of_the_card = &mut books[self.card_position];
        match wait {
            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished => {
                books_of_the_card.return_the_leases_kept_after_a_failed_wait_on(device.device);
            }
            DeviceWaitBeforeReturningTheLease::TheWaitFailed => {
                if !books_of_the_card
                    .lost_devices_not_released
                    .contains(&device.device)
                {
                    books_of_the_card
                        .leases_kept_after_a_failed_wait
                        .insert(self.lease_identifier, device.device);
                    self.when_dropped =
                        LeaseWhenTheBookingIsDropped::KeptOnTheBooksAfterAFailedWait;
                }
            }
        }
    }
}

impl Drop for GpuMemoryBooking<'_> {
    fn drop(&mut self) {
        match self.when_dropped {
            LeaseWhenTheBookingIsDropped::ReturnedToTheDriverAndOffTheBooks => {
                let removed = self.ledger.books_locked()[self.card_position]
                    .return_a_lease(self.lease_identifier);
                // 别处已经在 panic（丢借据是收尾的一部分）时不再断言：两次 panic 会让整个进程中止
                assert!(
                    removed || std::thread::panicking(),
                    "第 {} 张卡上借据号 {} 不在账上：一笔借据只在它丢掉时销一次账",
                    self.card_position,
                    self.lease_identifier
                );
            }
            LeaseWhenTheBookingIsDropped::KeptOnTheBooksAfterAFailedWait => {}
        }
    }
}

/// 账上一台起好的设备（[`GpuMemoryLedger::record_the_device_started_on_the_card`] 交回）：卡池把它放进这台设备的每一个克隆
/// （`singlefs_checker_tier::gpu_unit_checks::GpuCard` 的最后一个字段，借据的卡、卡池那一格、调用方留着的卡都是克隆），
/// 最后一个克隆丢掉时这一台放掉：这个进程里它的句柄都没了，wgpu 收掉设备、它上面的缓冲交回驱动（wgpu-core 30 的 `Queue::drop` 先等设备空了
/// 再收拾），它记成丢了的不再让这张卡照分不出算，在它上面没等成的借据销账（第 10z 批第八块，第 10z 批第六块代码轮判决 K1-3）。
/// 管不到的：调用方把 wgpu 的设备、队列、缓冲从克隆里另拿出来、比卡活得久（卡池这边没有这种拿法，借据先于它的缓冲丢的那一条同一个前提）。
pub struct DeviceStartedOnTheBooks<'ledger> {
    ledger: &'ledger GpuMemoryLedger,
    card_position: usize,
    device: DeviceNumberOnTheCard,
}

impl DeviceStartedOnTheBooks<'_> {
    /// 列卡（`listed`，[`GpuMemoryLedger::leases_kept_after_a_failed_wait_on_each_card`]）之后对这台设备 poll 一次交回队列空了：到 poll 那一刻为止提交的活
    /// 都做完了，wgpu 收完那几次提交、丢掉的缓冲交回了驱动。列出来的、等的是这一台、此刻还照旧记着的那几笔销账（它们的缓冲在列卡之前就丢了、
    /// 活在那之前提交）；列卡之后才记成没等成的照旧记着（第 10z 批第八块，第 10z 批第六块代码轮判决 K1-2：两次超时之间没有别的等成时也出得来；
    /// 第 10z 批第十一块，第 10z 批第七八九块代码轮判决 K1-1）。
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

/// 卡池的一笔借据丢掉时那一步（`singlefs_checker_tier::gpu_unit_checks::gpu_test_start::GpuCardLease` 的 `drop` 只调它）：
/// 照 `wait_for_the_device` 等这笔借据那张卡上的设备 `device`（wgpu 在等的时候 panic 了接住），`wait_of` 照等的结局判等没等成，
/// 没等成时先交 `report_the_failed_wait` 打一行，再把结局记进这一笔（[`GpuMemoryBooking::take_in_the_wait_before_returning`]）。
/// 等的时候 panic 了：不在展开中，记完照样往外抛；已经在展开中（这笔借据是别处 panic 收尾时丢掉的），只记没等成、不再抛——展开中再抛会让
/// 整个进程中止，同一进程里别的用例都没了结局（第 10z 批第八块，第 10z 批第六块代码轮判决 K1-1 攻方 H3，改法 F-H3）。
pub fn wait_for_the_device_and_take_in_the_outcome<Polled>(
    booking: &mut GpuMemoryBooking<'_>,
    device: &DeviceStartedOnTheBooks<'_>,
    wait_for_the_device: impl FnOnce() -> Polled,
    wait_of: impl FnOnce(&std::thread::Result<Polled>) -> DeviceWaitBeforeReturningTheLease,
    report_the_failed_wait: impl FnOnce(&std::thread::Result<Polled>),
) {
    let polled = std::panic::catch_unwind(std::panic::AssertUnwindSafe(wait_for_the_device));
    let wait = wait_of(&polled);
    match wait {
        DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished => {}
        DeviceWaitBeforeReturningTheLease::TheWaitFailed => report_the_failed_wait(&polled),
    }
    booking.take_in_the_wait_before_returning(device, wait);
    if let Err(panic) = polled {
        if !std::thread::panicking() {
            std::panic::resume_unwind(panic);
        }
    }
}

/// 一张卡上这个进程的设备（卡池每张卡一格）：要时有留着的、没丢的就拿它；没有（第一次要、上一次起不来）或丢了就起一个，
/// 起好了留着，起不来交为什么、不留着（第 10x 批代码轮判决 K1-3：起设备失败不缓存成这个进程永久的错；设备丢了之后借这张卡的
/// 不拿丢了的那个）。不碰 wgpu：设备、怎么起、怎么算丢了由卡池给，单测换成假的。
/// 两把锁：起设备时只拿 `starting`（同一张卡上同时要起的等这一次起完），留着的那一份只在取、换的那一下拿 `started`，
/// 取留着的设备（[`Self::started_now`]，卡池借之前 poll 用）不等正在起的设备（第 10z 批第十一块，第 10z 批第七八九块代码轮判决 K4-2：
/// 正推量到一次 300 毫秒的重起里借之前那一下等了 250 毫秒，哪怕借的是别的卡）。
pub struct DeviceStartedOnDemand<Device> {
    /// 留着的设备：持锁的那几段只取克隆、换一份，不起设备。
    started: Mutex<Option<Device>>,
    /// 起设备的那一段拿着。
    starting: Mutex<()>,
}

/// 空着（还没起设备）就是起点，没有别的不变量。
impl<Device: Clone> Default for DeviceStartedOnDemand<Device> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Device: Clone> DeviceStartedOnDemand<Device> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            started: Mutex::new(None),
            starting: Mutex::new(()),
        }
    }

    /// 留着的那一份的克隆（`started` 只在这一下拿）。持锁的这一段只克隆，不 panic；别的线程 panic 留下的毒照用。
    fn clone_of_the_device_kept(&self) -> Option<Device> {
        self.started
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// 留着的设备 `is_lost` 不为真就交它的克隆；否则调 `start` 起一个，起好了留着、交克隆，起不来交 `start` 的错（不留着）。
    /// 起的时候拿着这张卡起设备的那把锁：同一张卡上同时要起的等这一次起完、再看一次留着的（别的卡不等）；
    /// 取留着的设备（[`Self::started_now`]、[`Self::is_started`]）不等。
    ///
    /// # Errors
    /// `start` 的错。
    pub fn started_or_start<Failure>(
        &self,
        is_lost: impl Fn(&Device) -> bool,
        start: impl FnOnce() -> Result<Device, Failure>,
    ) -> Result<Device, Failure> {
        if let Some(device) = self.clone_of_the_device_kept() {
            if !is_lost(&device) {
                return Ok(device);
            }
        }
        // 持这把锁时 panic 只会在 `start` 里（起设备的断言）：那时留着的还是旧的（没有或丢了的），别的线程照用、照起
        let _starting = self.starting.lock().unwrap_or_else(PoisonError::into_inner);
        // 等这把锁的那一段里别的线程可能已经起好了：再看一次，起好了就用它，不再起
        if let Some(device) = self.clone_of_the_device_kept() {
            if !is_lost(&device) {
                return Ok(device);
            }
        }
        let device = start()?;
        // 换下来的旧设备在放开 `started` 之后才丢：它可能是这个进程里那台设备的最后一个克隆，丢的时候 wgpu 要等设备空了再收拾
        let replaced = self
            .started
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .replace(device.clone());
        drop(replaced);
        Ok(device)
    }

    /// 起过、留着一个设备没有（丢了的也算留着，下一次要时重起）；不等正在起的设备。
    #[must_use]
    pub fn is_started(&self) -> bool {
        self.started
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .is_some()
    }

    /// 留着的设备的克隆（丢了的也交；没起过、上一次没起来是 `None`）；不起设备，也不等正在起的设备：起着的时候交的是起之前留着的那一份
    /// （没有或丢了的；卡池借之前 poll 留着的设备用，丢了的不 poll）。
    #[must_use]
    pub fn started_now(&self) -> Option<Device> {
        self.clone_of_the_device_kept()
    }
}

/// 这个进程在哪几张卡（PCI 总线号，[`pci_bus_identifier_without_the_domain`] 写法）上起过不走这份账的设备（崩溃放量没起卡池时自己起的、
/// 经入口直接起的）：卡池读这张卡时这个进程占着的那一份分不出哪些是借据建的（[`OccupiedByThisProcess::AlsoOutsideTheBooks`]）。
/// 只增不减：那几个设备什么时候丢掉、卡上什么时候还，这里不知道，记着就照「分不出」算（只会少借）。
pub struct BusesWithDevicesStartedOutsideTheBooks {
    buses: Mutex<BTreeSet<String>>,
}

/// 空着（一个不走账的设备都没起过）就是起点，没有别的不变量。
impl Default for BusesWithDevicesStartedOutsideTheBooks {
    fn default() -> Self {
        Self::new()
    }
}

impl BusesWithDevicesStartedOutsideTheBooks {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            buses: Mutex::new(BTreeSet::new()),
        }
    }

    /// 先把 `bus` 记成起过不走账的设备，再调 `start` 起，交 `start` 的结果：起设备到记下之间那一段，卡池读到的这个进程占着的已经有
    /// 那个设备的那一份，后记就被当成借据建上的（第 10y 批代码轮判决 K1-1 攻方乙形）。起不来的照样记着（分不出，只会少借）。
    ///
    /// # Errors
    /// `start` 的错。
    pub fn start_a_device_marked_first<Device, Failure>(
        &self,
        bus: &str,
        start: impl FnOnce() -> Result<Device, Failure>,
    ) -> Result<Device, Failure> {
        // 持锁只为插一个总线号，没有会 panic 到一半的写：别的线程 panic 留下的毒照用
        self.buses
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(bus.to_string());
        start()
    }

    /// 这个进程在 `bus` 那张卡上起过（或正在起）不走账的设备没有。
    #[must_use]
    pub fn has_devices_on_the_bus(&self, bus: &str) -> bool {
        self.buses
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .contains(bus)
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
/// （崩溃放量产品路径 `singlefs_checker_tier::crash_amplification`）照它认出：活 panic 了、或设备丢了，是不是这段活的显存不够
/// （第 10x 批代码轮判决 K1-3：wgpu 的错误范围按线程分，判器内核在另起的线程上编，那里的 Out of Memory 范围接不住）。
/// 每一次按记的那条线程认是哪段活的（第 10y 批代码轮判决 K1-3 攻方：别的线程上一次 Out of Memory 不把这段活真正的 panic 说成显存不够）：
/// 那条线程上正在做一段活就是那一段；没有（活另起的编内核的线程）就是那一刻这个设备上在编内核的那几段（[`Self::compile_for_the_work_of_this_thread`]）。
#[derive(Default)]
pub struct DeviceConditionOfACard {
    out_of_memory_reports: Mutex<Vec<OutOfMemoryReport>>,
    /// 此刻在这个设备上编内核的活，各有几处在编。
    works_compiling: Mutex<BTreeMap<WorkOnTheCardIdentifier, usize>>,
    /// 设备丢了的话：记了就不抹（重起的是新设备、新的一份）。
    lost: Mutex<Option<LossOfTheDevice>>,
}

/// 设备为什么丢的，按接 Out of Memory 的那一层要做的决定分（设备丢失的回调里认：wgpu 给的那一句是不是它的「显存不够」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CauseOfTheDeviceLoss {
    /// wgpu 因显存不够丢了设备（它把建管线、提交这类调用上的 Out of Memory 当致命）：因设备丢了而失败的活都说得成显存不够，
    /// 不必等别的活把它范围里接到的那一句交回（第 10z 批代码轮判决 K1-4 攻方甲形：同一个设备上不在同一组的活）。
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

    /// 记设备丢了（wgpu 的设备丢失回调调它）：`why` 是 wgpu 给的原因与那一句，`cause` 是回调认出来的缘由。
    pub fn record_the_device_lost(&self, why: String, cause: CauseOfTheDeviceLoss) {
        *self.lost.lock().unwrap_or_else(PoisonError::into_inner) =
            Some(LossOfTheDevice { why, cause });
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
    /// （第 10z 批代码轮判决 K1-4 攻方乙形：活自己真 panic 的那一刻设备没丢，之后同组别的段撞上致命的 Out of Memory、设备丢了，
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
    /// [`Self::device_when_the_work_returns`] 记的）。先记再报到、先等再认（第 10z 批代码轮判决 K1-3 攻方：一块的几段并发，一段因设备丢了先失败，
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

/// 在卡上做一段活 `work`、接住它的 panic，并在它交回的那一刻（接住之后、做别的任何事之前）记下设备丢没丢
/// （[`DeviceConditionOfACard::device_when_the_work_returns`]）：崩溃放量接 Out of Memory 的那一层
/// （`singlefs_checker_tier::crash_amplification::on_the_card_catching_out_of_memory_ending_together`）照它认这段活的失败
/// （第 10z 批第五块代码轮判决 K3，攻方 F4：这一步抽到 harness 档，交回那一刻的读法与位置不要卡就测得到）。
pub fn work_caught_noting_the_device_when_it_returned<Outcome>(
    condition: &DeviceConditionOfACard,
    work: impl FnOnce() -> Outcome,
) -> (std::thread::Result<Outcome>, DeviceWhenTheWorkReturned) {
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(work));
    let device_when_the_work_returned = condition.device_when_the_work_returns();
    (outcome, device_when_the_work_returned)
}

/// 同一个设备上一起开的几段活（崩溃放量单元校验和对拍时一块切成的几段，`singlefs_checker_tier::crash_amplification`）：每段收完报到一次，
/// 认它是不是显存不够之前等其余几段都报到（[`DeviceConditionOfACard::end_the_work_and_explain_its_failure`]）。先把每一段的报到都领出来
/// （[`Self::join`]），再起那几段：领出来的没报到就丢掉（那一段 panic 了）也算报到，别的段不会等它一辈子。
pub struct WorksEndingTogetherOnADevice {
    works_not_yet_ended: Mutex<usize>,
    every_work_ended: std::sync::Condvar,
}

/// 一段都没领（还没开活）就是起点，没有别的不变量。
impl Default for WorksEndingTogetherOnADevice {
    fn default() -> Self {
        Self::new()
    }
}

impl WorksEndingTogetherOnADevice {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            works_not_yet_ended: Mutex::new(0),
            every_work_ended: std::sync::Condvar::new(),
        }
    }

    /// 领一段活的报到（这一段起之前领）。
    #[must_use]
    pub fn join(&self) -> WorkEndingTogether<'_> {
        // 持锁只为改一个计数，没有会 panic 到一半的写：别的线程 panic 留下的毒照用
        *self
            .works_not_yet_ended
            .lock()
            .unwrap_or_else(PoisonError::into_inner) += 1;
        WorkEndingTogether {
            together: self,
            has_ended: false,
        }
    }

    /// 领了报到、还没报到的段数。
    #[must_use]
    pub fn works_not_yet_ended(&self) -> usize {
        *self
            .works_not_yet_ended
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    fn end_one_work(&self) {
        let mut works_not_yet_ended = self
            .works_not_yet_ended
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        *works_not_yet_ended = works_not_yet_ended
            .checked_sub(1)
            .expect("每一份报到只报一次：领一次加一，报一次减一");
        if *works_not_yet_ended == 0 {
            self.every_work_ended.notify_all();
        }
    }
}

/// 一段活的报到（[`WorksEndingTogetherOnADevice::join`]）：[`Self::end_and_wait_for_the_others`] 报到；没报到就丢掉，丢的时候补报（不等）。
pub struct WorkEndingTogether<'together> {
    together: &'together WorksEndingTogetherOnADevice,
    has_ended: bool,
}

impl WorkEndingTogether<'_> {
    /// 这一段收完了：报到，等一起开的其余几段都报到（或丢掉了报到）再交回。
    pub fn end_and_wait_for_the_others(mut self) {
        self.has_ended = true;
        self.together.end_one_work();
        let mut works_not_yet_ended = self
            .together
            .works_not_yet_ended
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        while *works_not_yet_ended > 0 {
            works_not_yet_ended = self
                .together
                .every_work_ended
                .wait(works_not_yet_ended)
                .unwrap_or_else(PoisonError::into_inner);
        }
    }
}

impl Drop for WorkEndingTogether<'_> {
    fn drop(&mut self) {
        if !self.has_ended {
            self.together.end_one_work();
        }
    }
}

/// 卡线程这一刻要不要停（每领一块之前问一次，[`judge_blocks_on_one_card`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StopOfTheCards {
    /// 照常领。
    NotRequested,
    /// 调用方叫停：不领新的块，只领判完的块在等的那几块（比判完的块号小、交回队里的），判完的块照块序都写得进库、库里不留空洞；
    /// 没判的留给下一趟（第 10z 批代码轮判决 K1-5：收尾再起的那一轮里叫停，上一轮判完的块在等交回的那一块）。
    RequestedByTheCaller,
    /// 写库失败或有卡拒判（这一趟判红）：一块都不再领。
    TheRunFailed,
}

/// 几张卡一起判的那一队块（崩溃放量产品路径第 ① 段，`singlefs_checker_tier::crash_amplification`；块号是要判的块在那一趟 `pending` 里的次序）：
/// 每张卡领序号最小的、还没领走的那一块；一张卡报了 Out of Memory 把手里那一块交回，交回的序号比没领过的都小，下一张来领的卡先领到它
/// （第 10z 批，第 10y 批代码轮判决 K2-2 P2：一张卡 Out of Memory 不停整趟）。记着判完的块里块号最大的（几轮接着算）：调用方叫停之后
/// 只领比它小的（[`StopOfTheCards::RequestedByTheCaller`]）。不碰卡：单测在 harness 档随时跑（文件头）。
pub struct BlocksForTheCards {
    queue: Mutex<QueueOfTheBlocks>,
}

struct QueueOfTheBlocks {
    blocks_not_taken: BTreeSet<usize>,
    /// 判完的块里块号最大的（[`judge_blocks_on_one_card`] 判完一块就记）：队里比它小的，判完的块在等。
    largest_judged: Option<usize>,
}

impl QueueOfTheBlocks {
    /// 队头那一块 `first` 此刻照 `stop` 领不领得。
    fn may_take(&self, first: usize, stop: StopOfTheCards) -> bool {
        match stop {
            StopOfTheCards::NotRequested => true,
            StopOfTheCards::RequestedByTheCaller => {
                self.largest_judged.is_some_and(|largest| first < largest)
            }
            StopOfTheCards::TheRunFailed => false,
        }
    }
}

impl BlocksForTheCards {
    /// 一队块：`pending_indices` 里的每一块，还没有判完的。
    #[must_use]
    pub fn new(pending_indices: impl IntoIterator<Item = usize>) -> Self {
        Self {
            queue: Mutex::new(QueueOfTheBlocks {
                blocks_not_taken: pending_indices.into_iter().collect(),
                largest_judged: None,
            }),
        }
    }

    fn queue_locked(&self) -> MutexGuard<'_, QueueOfTheBlocks> {
        // 持锁只为取、放一个序号，没有会 panic 到一半的写：别的线程 panic 留下的毒照用
        self.queue.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// 照 `stop` 领一块：没叫停领序号最小的、还没领走的（交回的也算）；调用方叫停了，那一块比判完的块号小（判完的块在等它）才领；
    /// 判红了不领。领不得或都领走了是 `None`。
    #[must_use]
    pub fn take_a_block(&self, stop: StopOfTheCards) -> Option<usize> {
        let mut queue = self.queue_locked();
        let first = queue.blocks_not_taken.first().copied()?;
        if queue.may_take(first, stop) {
            queue.blocks_not_taken.pop_first()
        } else {
            None
        }
    }

    /// 照 `stop` 此刻还有没有领得的块（起下一张卡之前问：没有就不起）。
    #[must_use]
    pub fn any_left_to_take(&self, stop: StopOfTheCards) -> bool {
        let queue = self.queue_locked();
        queue
            .blocks_not_taken
            .first()
            .is_some_and(|first| queue.may_take(*first, stop))
    }

    /// 判完了一块（交去写库之前记）。
    fn record_a_judged_block(&self, pending_index: usize) {
        let mut queue = self.queue_locked();
        queue.largest_judged = Some(
            queue
                .largest_judged
                .map_or(pending_index, |largest| largest.max(pending_index)),
        );
    }

    /// 领了没判完的一块交回队里（报了 Out of Memory 的卡交）。
    ///
    /// # Panics
    /// 这一块此刻就在队里（交了两次，或没领过）。
    pub fn give_back(&self, pending_index: usize) {
        let is_newly_given_back = self.queue_locked().blocks_not_taken.insert(pending_index);
        assert!(
            is_newly_given_back,
            "块 {pending_index} 交回时已经在队里：一块领一次只交回一次"
        );
    }

    /// 还有没领走的块没有（没领过的或交回的）。
    #[must_use]
    pub fn any_left(&self) -> bool {
        !self.queue_locked().blocks_not_taken.is_empty()
    }

    /// 此刻还没领走的块，按序号（几张卡都收工之后，就是这一轮没判的那几块）。
    #[must_use]
    pub fn blocks_not_taken(&self) -> Vec<usize> {
        self.queue_locked()
            .blocks_not_taken
            .iter()
            .copied()
            .collect()
    }

    /// 一轮卡都收工之后，下一轮的那一队：这一轮没领走的，连同判完的块里块号最大的（下一轮叫停了照样领判完的块在等的那几块）。
    #[must_use]
    pub fn left_for_another_round(&self) -> Self {
        let queue = self.queue_locked();
        Self {
            queue: Mutex::new(QueueOfTheBlocks {
                blocks_not_taken: queue.blocks_not_taken.clone(),
                largest_judged: queue.largest_judged,
            }),
        }
    }
}

/// 一张卡判一块的结局（[`judge_blocks_on_one_card`] 的 `judge_a_block` 交）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlockJudgedOnACard<Judged> {
    Judged(Judged),
    /// 卡上 wgpu 报了 Out of Memory（`reason` 是本次未跑那一行的原因）：这一块交回，这张卡收工。
    OutOfMemory {
        reason: String,
    },
    /// 这张卡判不了这一块里的某个状态（表越过了内核的容量这类，`reason` 说哪一个）：这一块不交回，调用方叫所有卡停下。
    Refused {
        reason: String,
    },
}

/// 判完的一块交没交到写库那一头（[`judge_blocks_on_one_card`] 的 `hand_to_the_store` 交）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockHandedToTheStore {
    Handed,
    /// 写库那一头已经不收了（写库那条线程走了）。
    TheStoreIsGone,
}

/// 一张卡怎么收的工（[`judge_blocks_on_one_card`]）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CardLeftTheBlocks {
    /// 队里没有领得的了（领完了，或照叫停领不得）。
    NothingLeftOrStopped,
    /// 判一块时报了 Out of Memory：那一块已经交回队里。
    OutOfMemory { reason: String },
    /// 判不了一块：那一块没交回。
    Refused { reason: String },
    /// 判完的一块交不到写库那一头。
    TheStoreIsGone,
}

/// 一张卡的线程判块的那一圈（崩溃放量产品路径第 ① 段的卡线程，判器建好之后）：每领一块之前问 `stop_of_the_cards`、照它领一块
/// （[`BlocksForTheCards::take_a_block`]，叫停了只领判完的块在等的），交 `judge_a_block` 判；判完记下、交 `hand_to_the_store`
/// （写库按块序，[`BlocksStoredInOrder`]）；报了 Out of Memory 就把这一块交回队里、这张卡收工，不再领（只停这一张卡，第 10z 批）；
/// 判不了就收工、这一块不交回。循环的次数不超过队里的块数加交回的次数（每次领走一块，每张卡至多交回一次）。
pub fn judge_blocks_on_one_card<Judged>(
    blocks: &BlocksForTheCards,
    stop_of_the_cards: &dyn Fn() -> StopOfTheCards,
    judge_a_block: &mut dyn FnMut(usize) -> BlockJudgedOnACard<Judged>,
    hand_to_the_store: &mut dyn FnMut(usize, Judged) -> BlockHandedToTheStore,
) -> CardLeftTheBlocks {
    while let Some(pending_index) = blocks.take_a_block(stop_of_the_cards()) {
        match judge_a_block(pending_index) {
            BlockJudgedOnACard::Judged(judged) => {
                blocks.record_a_judged_block(pending_index);
                match hand_to_the_store(pending_index, judged) {
                    BlockHandedToTheStore::Handed => {}
                    BlockHandedToTheStore::TheStoreIsGone => {
                        return CardLeftTheBlocks::TheStoreIsGone
                    }
                }
            }
            BlockJudgedOnACard::OutOfMemory { reason } => {
                blocks.give_back(pending_index);
                return CardLeftTheBlocks::OutOfMemory { reason };
            }
            BlockJudgedOnACard::Refused { reason } => return CardLeftTheBlocks::Refused { reason },
        }
    }
    CardLeftTheBlocks::NothingLeftOrStopped
}

/// 第 `position` 张卡备好了（或建判器时报了 Out of Memory、不判）之后起哪一张卡的线程（崩溃放量产品路径第 ① 段，卡一张一张备、不同时编内核）：
/// 后面第一张装了状态的卡（`states_placed` 按这一轮卡的次序，0 是没装），而且队里照此刻的叫停还有领得的块
/// （[`BlocksForTheCards::any_left_to_take`]：叫停了只剩判完的块不在等的，不起）；否则不起（第 10z 批第五块代码轮判决 K3：
/// 这一步抽到 harness 档，起下一张卡时看不看叫停不要卡就测得到）。
/// 叫停只在后面有装了状态的卡时才问 `stop_of_the_cards_now`、问一次，没有下一张卡一次都不问（第 10z 批第七块：
/// 抽出来那一版不管有没有下一张卡都先问，每起一张卡多问一次；调用方的叫停可以按「问第几次」变，多问一次卡上就少领一块）。
#[must_use]
pub fn next_card_to_start(
    states_placed: &[u64],
    position: usize,
    blocks: &BlocksForTheCards,
    stop_of_the_cards_now: impl FnOnce() -> StopOfTheCards,
) -> Option<usize> {
    let next_position =
        (position + 1..states_placed.len()).find(|candidate| states_placed[*candidate] > 0)?;
    if blocks.any_left_to_take(stop_of_the_cards_now()) {
        Some(next_position)
    } else {
        None
    }
}

/// 判完的块按块序写库（判定向量的编号按块序登记，与用了几张卡无关）：块号从 0 起连着，后面的块先到就等着，前面那一块到了一起写；
/// 写库失败之后到的不再写。几轮卡（[`after_a_round_of_the_cards`]）共用一份：上一轮判完、等着前面那一块的，下一轮那一块判完照块序一起写，
/// 不再判一遍。
pub struct BlocksStoredInOrder<Block> {
    blocks_stored: usize,
    waiting: BTreeMap<usize, Block>,
    storing_error: Option<String>,
}

/// 一块都没写、没有等着的就是起点，没有别的不变量。
impl<Block> Default for BlocksStoredInOrder<Block> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Block> BlocksStoredInOrder<Block> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            blocks_stored: 0,
            waiting: BTreeMap::new(),
            storing_error: None,
        }
    }

    /// 块号 `pending_index` 的一块判完到了：照块序交 `store_block` 写（它和前面连着的、等着的一起）；写失败记下那一句。
    pub fn arrive(
        &mut self,
        pending_index: usize,
        block: Block,
        store_block: &mut dyn FnMut(Block) -> Result<(), String>,
    ) {
        self.waiting.insert(pending_index, block);
        while let Some(next_block) = self.waiting.remove(&self.blocks_stored) {
            if self.storing_error.is_some() {
                continue;
            }
            match store_block(next_block) {
                Ok(()) => self.blocks_stored += 1,
                Err(error) => self.storing_error = Some(error),
            }
        }
    }

    /// 照块序写进库了几块（块号 `0..` 这几块）。
    #[must_use]
    pub fn blocks_stored(&self) -> usize {
        self.blocks_stored
    }

    /// 判完了、等着前面那一块的有几块。
    #[must_use]
    pub fn blocks_waiting(&self) -> usize {
        self.waiting.len()
    }

    /// 写库失败的那一句（没失败是 `None`）。
    #[must_use]
    pub fn storing_error(&self) -> Option<&str> {
        self.storing_error.as_deref()
    }
}

/// 调用方叫没叫停（一轮卡都收工之后读的那一次，[`RoundsOfTheCards::after_the_round`]；卡线程每领一块之前问的是 [`StopOfTheCards`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StopOfTheRun {
    Requested,
    NotRequested,
}

/// 一趟里调用方叫没叫停，见过一次就记住（第 10z 批第五块代码轮判决 K2-2，攻方 F2：叫停是一个文件在不在，外层循环、卡线程、一轮收尾、
/// 整趟收尾各自现读时文件中途被删，四处读到的不一样，轮末断言 panic、或照叫停收尾而整趟没记成提早收尾）。这一趟里问叫停的每一处都问它；
/// 头一次读到叫停之后不再读 `read_the_request_now`，之后恒答叫停。几条线程一起问（卡线程与收写库那一条）。
pub struct StopRequestRememberedInThisRun<'reader> {
    read_the_request_now: &'reader (dyn Fn() -> bool + Sync),
    has_seen_the_request: AtomicBool,
}

impl<'reader> StopRequestRememberedInThisRun<'reader> {
    /// 这一趟开头：还没见过叫停，要问时照 `read_the_request_now` 现读（崩溃放量产品路径是叫停文件在不在）。
    #[must_use]
    pub fn new(read_the_request_now: &'reader (dyn Fn() -> bool + Sync)) -> Self {
        Self {
            read_the_request_now,
            has_seen_the_request: AtomicBool::new(false),
        }
    }

    /// 这一刻叫没叫停：这一趟里见过就是叫了；没见过就现读一次，读到叫停记下。
    #[must_use]
    pub fn is_requested(&self) -> bool {
        if self.has_seen_the_request.load(Ordering::SeqCst) {
            return true;
        }
        if (self.read_the_request_now)() {
            self.has_seen_the_request.store(true, Ordering::SeqCst);
            return true;
        }
        false
    }

    /// 同 [`Self::is_requested`]，写成一轮收尾交 [`RoundsOfTheCards::after_the_round`] 的那一种。
    #[must_use]
    pub fn stop_of_the_run(&self) -> StopOfTheRun {
        if self.is_requested() {
            StopOfTheRun::Requested
        } else {
            StopOfTheRun::NotRequested
        }
    }
}

/// 一轮卡都收工之后这一趟怎么走（[`after_a_round_of_the_cards`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AfterARoundOfTheCards {
    /// 要判的块都写进了库。
    EveryBlockStored,
    /// 被叫停，判完的块都写进了库（没有在等的），没判的留给下一趟；这一轮有卡报了 Out of Memory 也一样（第 10z 批代码轮判决 K1-5：
    /// 交回的块是卡上领走之前报的还是之后报的，结局不随先后变）。
    StoppedOnRequest,
    /// 这一轮有卡报了 Out of Memory、还有块没写、没被叫停，而还有没报过的卡：收尾这一步再起一轮，只用没报过的卡判剩下的块
    /// （第 10z 批代码轮判决 K2-1 第 5 条：交回的块在别的卡都收工之后也要有人判，状态全装在第一张卡时也要起第二张）。
    AnotherRoundWithoutTheCardsShortOfMemory,
    /// 被叫停，这一轮有卡报了 Out of Memory、交回的块前面判完的块在等它，而还有没报过的卡：再起一轮，只用没报过的卡判判完的块在等的那几块
    /// （叫停照旧：卡只领那几块，[`StopOfTheCards::RequestedByTheCaller`]），判完的都入库再照叫停收尾（第 10z 批代码轮判决 K1-5）。
    AnotherRoundOnlyForTheBlocksTheJudgedBlocksWaitFor,
    /// 有卡报了 Out of Memory、还有块没写，而报过的卡之外没有卡了（被叫停的，是判完的块在等的那几块没有卡能判）：这一趟报本次未跑，
    /// 写进库的块在库里。
    NotRunTheGpuMemoryWasShort,
}

/// 一轮卡都收工之后：`blocks_left_unstored` 块还没写进库（共要判的减照块序写进的，[`BlocksStoredInOrder::blocks_stored`]），其中
/// `blocks_waiting` 块判完了在等前面的块，这一轮 `cards_short_of_memory_this_round` 张卡报了 Out of Memory，到这一轮为止没报过的卡还有
/// `cards_not_short_of_memory` 张。每再起一轮至少少一张卡，轮数不超过卡数。
#[must_use]
pub fn after_a_round_of_the_cards(
    blocks_left_unstored: usize,
    blocks_waiting: usize,
    cards_short_of_memory_this_round: usize,
    stop: StopOfTheRun,
    cards_not_short_of_memory: usize,
) -> AfterARoundOfTheCards {
    if blocks_left_unstored == 0 {
        return AfterARoundOfTheCards::EveryBlockStored;
    }
    if cards_short_of_memory_this_round == 0 {
        return AfterARoundOfTheCards::StoppedOnRequest;
    }
    match stop {
        StopOfTheRun::Requested if blocks_waiting == 0 => AfterARoundOfTheCards::StoppedOnRequest,
        StopOfTheRun::Requested if cards_not_short_of_memory == 0 => {
            AfterARoundOfTheCards::NotRunTheGpuMemoryWasShort
        }
        StopOfTheRun::Requested => {
            AfterARoundOfTheCards::AnotherRoundOnlyForTheBlocksTheJudgedBlocksWaitFor
        }
        StopOfTheRun::NotRequested if cards_not_short_of_memory == 0 => {
            AfterARoundOfTheCards::NotRunTheGpuMemoryWasShort
        }
        StopOfTheRun::NotRequested => {
            AfterARoundOfTheCards::AnotherRoundWithoutTheCardsShortOfMemory
        }
    }
}

/// 第 ① 段几轮卡之间带着的（崩溃放量产品路径，`singlefs_checker_tier::crash_amplification` 的外层循环；不碰卡，单测在 harness 档随时跑）：
/// 按块序写库的那一份（几轮共用：上一轮判完、等着前面那一块的，下一轮那一块判完照块序一起写）、下一轮的那一队块、报过 Out of Memory 的卡
/// 与头一句、收过工的卡。一轮怎么走：[`Self::take_the_blocks_of_the_round`] 交出这一轮的那一队，卡上的线程判、写库那条线程照
/// [`Self::stored_in_order`] 写，卡都收工之后交 [`Self::after_the_round`]。
pub struct RoundsOfTheCards<Block> {
    pending_blocks: usize,
    stored_in_order: BlocksStoredInOrder<Block>,
    blocks_of_the_next_round: BlocksForTheCards,
    cards_short_of_memory: BTreeSet<u32>,
    first_out_of_memory: Option<String>,
    cards_that_finished: BTreeSet<u32>,
}

impl<Block> RoundsOfTheCards<Block> {
    /// 头一轮之前：`pending_blocks` 块要判（块号 `0..pending_blocks`），一张卡都没用过。
    #[must_use]
    pub fn new(pending_blocks: usize) -> Self {
        Self {
            pending_blocks,
            stored_in_order: BlocksStoredInOrder::new(),
            blocks_of_the_next_round: BlocksForTheCards::new(0..pending_blocks),
            cards_short_of_memory: BTreeSet::new(),
            first_out_of_memory: None,
            cards_that_finished: BTreeSet::new(),
        }
    }

    /// 这张卡（`card_index` 是 `nvidia-smi` 给它的序号）这一趟报过 Out of Memory 没有：报过的不再用（每再起一轮至少少一张卡）。
    #[must_use]
    pub fn is_short_of_memory(&self, card_index: u32) -> bool {
        self.cards_short_of_memory.contains(&card_index)
    }

    /// 这一趟报过 Out of Memory 的卡（序号，进度行用）。
    #[must_use]
    pub fn cards_short_of_memory(&self) -> &BTreeSet<u32> {
        &self.cards_short_of_memory
    }

    /// 下一轮要判的块号（按序号）。
    #[must_use]
    pub fn blocks_of_the_next_round(&self) -> Vec<usize> {
        self.blocks_of_the_next_round.blocks_not_taken()
    }

    /// 交出这一轮的那一队（一轮一次；交出之后这里留一队空的，[`Self::after_the_round`] 换上这一轮剩下的）。
    #[must_use]
    pub fn take_the_blocks_of_the_round(&mut self) -> BlocksForTheCards {
        std::mem::replace(
            &mut self.blocks_of_the_next_round,
            BlocksForTheCards::new([]),
        )
    }

    /// 写库那条线程照它写（几轮共用一份）。
    pub fn stored_in_order(&mut self) -> &mut BlocksStoredInOrder<Block> {
        &mut self.stored_in_order
    }

    /// 照块序写进库了几块。
    #[must_use]
    pub fn blocks_stored(&self) -> usize {
        self.stored_in_order.blocks_stored()
    }

    /// 判完了、等着前面那一块的有几块。
    #[must_use]
    pub fn blocks_waiting(&self) -> usize {
        self.stored_in_order.blocks_waiting()
    }

    /// 这一趟头一张报 Out of Memory 的卡报的那一句（没有卡报过是 `None`）。
    #[must_use]
    pub fn first_out_of_memory(&self) -> Option<&str> {
        self.first_out_of_memory.as_deref()
    }

    /// 照叫停收尾（[`AfterARoundOfTheCards::StoppedOnRequest`]）时要打的那一行：这一趟有卡报过 Out of Memory，头一句照「本次未跑」那一行的
    /// 形态（[`gpu_not_run_line`]，`run_name` 是那一格的名字）；没有卡报过是 `None`（第 10z 批第五块代码轮判决 K1-3：叫停加有卡报
    /// Out of Memory 而照叫停收尾时，这一行不打，显存不够的缘由外面读不到）。
    #[must_use]
    pub fn not_run_line_of_a_stop_after_an_out_of_memory(&self, run_name: &str) -> Option<String> {
        self.first_out_of_memory
            .as_deref()
            .map(|reason| gpu_not_run_line(run_name, reason))
    }

    /// 收过工的卡有几张：一张卡在几轮里各收工一次只算一张（第 10z 批代码轮判决 K1-5：`cards_used` 算两遍）。
    #[must_use]
    pub fn cards_used(&self) -> usize {
        self.cards_that_finished.len()
    }

    /// 一轮卡都收工（写库没出错、没有卡拒判）之后：`blocks` 是这一轮的那一队，`cards_of_the_round` 是这一轮挑卡时没报过的卡（装没装状态都算），
    /// `cards_finished` 是这一轮收了工的卡，`cards_out_of_memory` 是这一轮报了 Out of Memory 的卡与那一句（按收到的次序），`stop` 是这一刻
    /// 调用方叫没叫停。记下报了的卡、收工的卡，换上下一轮的那一队（[`BlocksForTheCards::left_for_another_round`]），照
    /// [`after_a_round_of_the_cards`] 交这一趟怎么走。
    ///
    /// # Panics
    /// 这一轮没有卡报 Out of Memory，却还有判完的块在等前面的块（判完的块在等的那几块卡都该领了，叫停了也一样），或没人叫停却还有块没领。
    pub fn after_the_round(
        &mut self,
        blocks: &BlocksForTheCards,
        cards_of_the_round: &[u32],
        cards_finished: impl IntoIterator<Item = u32>,
        cards_out_of_memory: &[(u32, String)],
        stop: StopOfTheRun,
    ) -> AfterARoundOfTheCards {
        self.cards_that_finished.extend(cards_finished);
        if cards_out_of_memory.is_empty() {
            assert_eq!(
                self.stored_in_order.blocks_waiting(),
                0,
                "没有卡报 Out of Memory：判完的块每一块都写进了库（叫停了卡也领判完的块在等的那几块）"
            );
            assert!(
                !blocks.any_left() || stop == StopOfTheRun::Requested,
                "没人叫停，却有 {} 块没判",
                blocks.blocks_not_taken().len()
            );
        }
        for (index, reason) in cards_out_of_memory {
            self.cards_short_of_memory.insert(*index);
            self.first_out_of_memory
                .get_or_insert_with(|| reason.clone());
        }
        self.blocks_of_the_next_round = blocks.left_for_another_round();
        let cards_not_short_of_memory = cards_of_the_round
            .iter()
            .filter(|index| !self.cards_short_of_memory.contains(index))
            .count();
        after_a_round_of_the_cards(
            self.pending_blocks - self.stored_in_order.blocks_stored(),
            self.stored_in_order.blocks_waiting(),
            cards_out_of_memory.len(),
            stop,
            cards_not_short_of_memory,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::DeviceStartedOnDemand;
    use super::{
        after_a_round_of_the_cards, judge_blocks_on_one_card, AfterARoundOfTheCards,
        BlockHandedToTheStore, BlockJudgedOnACard, BlocksForTheCards, BlocksStoredInOrder,
        CardLeftTheBlocks, CauseOfTheDeviceLoss, DeviceWhenTheWorkReturned, RoundsOfTheCards,
        StopOfTheCards, StopOfTheRun, WorksEndingTogetherOnADevice,
    };
    use super::{
        attempt_on_a_card_without_room_for_one_state, card_memory_reader_of_one_report_per_reading,
        card_memory_readings_of_one_nvidia_smi_report, decide_scratch_before_booking,
        gpu_not_run_line, pci_bus_identifier_without_the_domain,
        reason_for_running_out_of_gpu_memory, BusesWithDevicesStartedOutsideTheBooks,
        DeviceConditionOfACard, WorkOnTheCardOfThisThread, GPU_MEMORY_SHORT_REASON_PREFIX,
    };
    use super::{
        card_ledger_capacity_mebibytes, card_room_for_this_process,
        decide_after_attempting_every_card, decide_booking_by_priority,
        mebibytes_of_buffers_held_at_once, memory_this_process_may_hold_mebibytes,
        most_states_whose_need_fits_the_capacity, place_states_on_the_cards_that_book_now,
        scratch_booking_refusal_outcome, scratch_states_to_book_on_each_card,
        states_in_one_dispatch_capped_by_the_state_count, BookedByThisProcessOnACard,
        BookingDecision, BookingRefusal, CardMemoryReading, CardRoomForThisProcess,
        DriverShareOnTheCard, GpuCardsSource, GpuMemoryBooking, GpuMemoryLedger,
        NeedLargerThanEveryCard, OccupiedByThisProcess, ScratchAttemptOnACard,
        ScratchBeforeBooking, ScratchBookingRefusalOutcome, ScratchPlacementDecision,
        ScratchStatesToBook,
    };
    use super::{
        next_card_to_start, wait_for_the_device_and_take_in_the_outcome,
        work_caught_noting_the_device_when_it_returned, DeviceStartedOnTheBooks,
        DeviceWaitBeforeReturningTheLease, LeasesKeptAfterAFailedWaitWhenListed,
        StopRequestRememberedInThisRun,
    };
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    /// 驱动的固定开销，照卡池用的那个数（`crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER`）。
    const DRIVER: u64 = 2560;
    /// 一张 5090 的显存总量与驱动保留的那一份（2026-10-07 本机 `nvidia-smi -q -d MEMORY` 报的 Total、Reserved）。
    const TOTAL_OF_AN_RTX_5090: u64 = 32_607;
    const RESERVED_ON_AN_RTX_5090: u64 = 497;
    /// 一张空着的 5090 上能用的：总量减保留（同一刻报的空闲就是这个数）。
    const AN_EMPTY_RTX_5090_HAS: u64 = TOTAL_OF_AN_RTX_5090 - RESERVED_ON_AN_RTX_5090;

    /// 一张 5090 此刻空着 `free_mebibytes`，这个进程实际占着的读不到。
    fn reading_of_an_rtx_5090(free_mebibytes: u64) -> CardMemoryReading {
        CardMemoryReading {
            free_mebibytes,
            total_less_reserved_mebibytes: AN_EMPTY_RTX_5090_HAS,
            occupied_by_this_process: OccupiedByThisProcess::NotListed,
        }
    }

    /// 换得了的假读数：几张卡此刻的显存，用例中途改它就是卡上的空闲变了。
    type ReplaceableReadings = Arc<Mutex<Vec<CardMemoryReading>>>;

    fn replaceable_readings(readings: Vec<CardMemoryReading>) -> ReplaceableReadings {
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
    fn ledger_reading(readings: &ReplaceableReadings) -> GpuMemoryLedger {
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

    /// 第 10x 批（G1 二）：这个进程借了 1 000 之后，一个不经这份账的 GPU 用户占了 20 000（卡上空闲从 30 000 掉到 9 000）。
    /// 下一笔照这一刻现读的借：9 000 减这个进程借了还没占的 1 000、减还没起设备的驱动那一份，正好 5 440，多 1 MiB 就借不到。
    #[test]
    fn a_booking_after_an_outsider_took_memory_follows_the_free_memory_read_at_that_booking() {
        let readings = replaceable_readings(vec![reading_of_an_rtx_5090(30_000)]);
        let ledger = ledger_reading(&readings);
        assert_eq!(
            booking_kept_on_the_books(&ledger, 1_000),
            Ok(1_000),
            "空着的卡借得到"
        );
        replace_the_reading(&readings, 0, reading_of_an_rtx_5090(9_000));
        assert_eq!(
            booking_kept_on_the_books(&ledger, 5_441),
            Err(BookingRefusal::NoCardHasRoomNow {
                need_mebibytes: 5_441,
                card_rooms_mebibytes: vec![9_000 - 1_000 - DRIVER],
            }),
            "外人占了 20 000：这一笔照现读的 9 000 算，借不到（不判红、不等）"
        );
        assert_eq!(
            booking_kept_on_the_books(&ledger, 5_440),
            Ok(5_440),
            "现读的空闲减借了还没占的、减驱动那一份，正好借得到"
        );
    }

    /// 第 10x 批（G1 二）：外人占着卡（空 5 000）时借 10 000 借不到，不判红（卡空着时借得出 29 550）；
    /// 外人走了（空 30 000），下一笔照现读的借到。
    #[test]
    fn a_booking_after_an_outsider_left_gets_the_memory_it_returned() {
        let readings = replaceable_readings(vec![reading_of_an_rtx_5090(5_000)]);
        let ledger = ledger_reading(&readings);
        assert_eq!(
            booking_kept_on_the_books(&ledger, 10_000),
            Err(BookingRefusal::NoCardHasRoomNow {
                need_mebibytes: 10_000,
                card_rooms_mebibytes: vec![5_000 - DRIVER],
            }),
            "外人占着：借不到，不当成比卡还大"
        );
        replace_the_reading(&readings, 0, reading_of_an_rtx_5090(30_000));
        assert_eq!(
            booking_kept_on_the_books(&ledger, 10_000),
            Ok(10_000),
            "外人走了：下一笔照现读的借到"
        );
    }

    /// 第 10x 批（G1 二）：这个进程在同一张卡上先后两笔，卡上读数一直是空 30 000（借的都还没建）：第一笔 20 000，
    /// 第二笔最多 7 440，两笔加驱动那一份正好是现读的空闲，多 1 MiB 就借不到。
    #[test]
    fn two_bookings_of_this_process_on_one_card_stay_within_the_free_memory_read_now() {
        let readings = replaceable_readings(vec![reading_of_an_rtx_5090(30_000)]);
        let ledger = ledger_reading(&readings);
        assert_eq!(booking_kept_on_the_books(&ledger, 20_000), Ok(20_000));
        assert!(
            matches!(
                booking_kept_on_the_books(&ledger, 7_441),
                Err(BookingRefusal::NoCardHasRoomNow { .. })
            ),
            "两笔加驱动那一份超过现读的空闲：借不到"
        );
        assert_eq!(booking_kept_on_the_books(&ledger, 7_440), Ok(7_440));
        assert_eq!(
            20_000 + 7_440 + DRIVER,
            30_000,
            "两笔加驱动那一份正好是现读的空闲"
        );
    }

    /// 第 10x 批：这个进程能占的是现读的空闲加它实际占着的里属于它许出去的那一部分：没登记、登记了没起设备都只算空闲；
    /// 起了设备而读不到实际占着的，驱动那一份也不当已经在卡上（第 10z 批：实占多少分不出）；读得到的只算到许出去的那么多（分配器多拿的块不算）。
    #[test]
    fn the_memory_this_process_may_hold_counts_only_what_it_occupies_of_what_it_promised() {
        let free = reading_of_an_rtx_5090(20_000);
        let booked = |leases_mebibytes: u64, driver_share: DriverShareOnTheCard| {
            BookedByThisProcessOnACard {
                leases_mebibytes,
                driver_share,
            }
        };
        assert_eq!(
            memory_this_process_may_hold_mebibytes(
                free,
                booked(0, DriverShareOnTheCard::NotRegistered),
                DRIVER
            ),
            20_000,
            "没登记：只有现读的空闲"
        );
        assert_eq!(
            memory_this_process_may_hold_mebibytes(
                free,
                booked(1_000, DriverShareOnTheCard::RegisteredDeviceNotStarted),
                DRIVER
            ),
            20_000,
            "登记了、设备没起、实际占着的读不到：借的当一点都没建"
        );
        assert_eq!(
            memory_this_process_may_hold_mebibytes(
                free,
                booked(1_000, DriverShareOnTheCard::DeviceStarted),
                DRIVER
            ),
            20_000,
            "起了设备、实际占着的读不到：驱动那一份实占多少分不出，不当已经在卡上"
        );
        let occupied = |mebibytes: u64| CardMemoryReading {
            occupied_by_this_process: OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes },
            ..free
        };
        assert_eq!(
            memory_this_process_may_hold_mebibytes(
                occupied(DRIVER + 500),
                booked(1_000, DriverShareOnTheCard::DeviceStarted),
                DRIVER
            ),
            20_000 + DRIVER + 500,
            "读得到实际占着的：驱动那一份与借的里建了的 500"
        );
        assert_eq!(
            memory_this_process_may_hold_mebibytes(
                occupied(9_000),
                booked(1_000, DriverShareOnTheCard::DeviceStarted),
                DRIVER
            ),
            20_000 + DRIVER + 1_000,
            "实际占着的比许出去的多：只算到许出去的那么多"
        );
    }

    /// 第 10x 批 K4-3：额度变量 20 000、配置给卡 24 G 都设了，这个进程此刻能占的只有 10 000：容量照能占的那一项压到
    /// 10 000 减驱动那一份，不取两个额度里小的那个。
    #[test]
    fn the_memory_this_process_may_hold_caps_the_capacity_even_when_both_quotas_are_set() {
        const CONFIGURED_CARD_QUOTA: u64 = 24 * 1024;
        const MAY_HOLD: u64 = 10_000;
        assert_eq!(
            card_ledger_capacity_mebibytes(
                Some(20_000),
                Some(CONFIGURED_CARD_QUOTA),
                CONFIGURED_CARD_QUOTA.min(MAY_HOLD),
                MAY_HOLD,
                DRIVER
            ),
            MAY_HOLD - DRIVER,
            "能占的那一项最小：取它"
        );
    }

    /// 第 10x 批（先登记、后建设备，这个进程里）：没在一张卡上借过就起设备要断言；借过之后起得了，起好了驱动那一份算在卡上：
    /// 读数里这个进程占着的驱动那一份从空闲里少了，不再从能借的里扣第二次；读不到实际占着的时候照旧扣（第 10z 批：实占多少分不出）。
    #[test]
    fn the_device_starts_only_on_a_card_this_process_booked_and_then_counts_the_driver_share_as_on_the_card(
    ) {
        let readings = replaceable_readings(vec![reading_of_an_rtx_5090(30_000)]);
        let ledger = ledger_reading(&readings);
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                ledger.check_registered_before_starting_the_device(0);
            }))
            .is_err(),
            "没借过就起设备：先登记、后建设备的次序被打乱，要断言"
        );
        assert!(!ledger.is_this_process_registered_on_the_card(0));
        let booking = ledger.book_on_the_card(0, 1_000).expect("空着的卡借得到");
        assert!(
            ledger.is_this_process_registered_on_the_card(0),
            "借过就登记了"
        );
        assert_eq!(
            ledger.card_rooms_mebibytes_now(&[0]),
            Ok(vec![30_000 - 1_000 - DRIVER]),
            "设备没起：驱动那一份还要从空闲里扣"
        );
        assert_eq!(
            ledger.prepare_the_driver_share_before_starting_the_device(0),
            Ok(()),
            "头一回起：驱动那一份在登记的那一笔里借过"
        );
        let _device = ledger.record_the_device_started_on_the_card(0);
        assert_eq!(
            ledger.card_rooms_mebibytes_now(&[0]),
            Ok(vec![30_000 - 1_000 - DRIVER]),
            "设备起好了而读不到实际占着的：驱动那一份照旧扣"
        );
        replace_the_reading(
            &readings,
            0,
            CardMemoryReading {
                free_mebibytes: 30_000 - DRIVER,
                total_less_reserved_mebibytes: AN_EMPTY_RTX_5090_HAS,
                occupied_by_this_process: OccupiedByThisProcess::OnlyThroughTheBooks {
                    mebibytes: DRIVER,
                },
            },
        );
        assert_eq!(
            ledger.card_rooms_mebibytes_now(&[0]),
            Ok(vec![30_000 - DRIVER - 1_000]),
            "设备起好了、读到占着驱动那一份：现读的空闲里少了它，不再扣第二次"
        );
        drop(booking);
    }

    /// 第 10x 批（第 10t 批正推探针 G 那一形）：卡上只空 1 000，不到一份驱动开销：借 1 MiB 也借不到，这个进程不登记（调用方不起设备）。
    #[test]
    fn a_card_with_less_free_memory_than_one_driver_share_lends_nothing_and_registers_nothing() {
        let readings = replaceable_readings(vec![reading_of_an_rtx_5090(1_000)]);
        let ledger = ledger_reading(&readings);
        assert_eq!(
            booking_kept_on_the_books(&ledger, 1),
            Err(BookingRefusal::NoCardHasRoomNow {
                need_mebibytes: 1,
                card_rooms_mebibytes: vec![0],
            })
        );
        assert!(
            !ledger.is_this_process_registered_on_the_card(0),
            "借不到就不登记"
        );
    }

    /// 第 10x 批 K4-1 用的核账：一笔借据丢掉之前一直在账上，丢掉就不在了。
    #[test]
    fn a_lease_is_on_the_books_until_it_is_dropped() {
        let readings = replaceable_readings(vec![reading_of_an_rtx_5090(30_000)]);
        let ledger = ledger_reading(&readings);
        let booking = ledger.book_on_the_card(0, 1_000).expect("空着的卡借得到");
        let lease_identifier = booking.lease_identifier();
        assert!(ledger.is_the_lease_on_the_books(0, lease_identifier));
        drop(booking);
        assert!(
            !ledger.is_the_lease_on_the_books(0, lease_identifier),
            "丢掉就销账"
        );
        assert_eq!(
            ledger.card_rooms_mebibytes_now(&[0]),
            Ok(vec![30_000 - DRIVER]),
            "销了账，这一笔不再从空闲里扣"
        );
    }

    /// 第 10x 批（第 10t 批正推探针 C、攻方 S3 那一形）：卡上空 20 000，这个进程已经借着 12 000（没建）。按状态数借 30 个、
    /// 每个 1 000 MiB：照此刻能借的 5 440 压成 5 个，当场借到，不按卡的整份容量压成 17 个再判放不下。
    #[test]
    fn a_scratch_booking_compresses_its_states_to_the_room_now_and_books_them() {
        let readings = replaceable_readings(vec![reading_of_an_rtx_5090(20_000)]);
        let ledger = ledger_reading(&readings);
        assert_eq!(booking_kept_on_the_books(&ledger, 12_000), Ok(12_000));
        let bookings = ledger
            .book_states_on_the_cards_with_room(&[0], 30, &[30], &|states| states * 1_000)
            .expect("此刻放得下 5 个状态");
        let booked = bookings[0].as_ref().expect("只有这一张卡，借到了");
        assert_eq!(
            booked.states, 5,
            "照此刻能借的 20 000 − 12 000 − 驱动那一份压"
        );
        assert_eq!(booked.booking.mebibytes(), 5_000);
    }

    /// 第 10x 批（第 10t 批攻方 S4b、正推探针 D 那一形）：两张卡，第一张这个进程借着 12 000（空 20 000），第二张空 16 000。
    /// 30 个状态、每个 1 000 MiB、每张带得了 30 个：第一张照此刻能借的装 5 个，剩下的 25 个里第二张装得下 13 个，两张都用上。
    #[test]
    fn the_states_a_card_cannot_take_now_go_to_the_next_card_with_room() {
        let readings = replaceable_readings(vec![
            reading_of_an_rtx_5090(20_000),
            reading_of_an_rtx_5090(16_000),
        ]);
        let ledger = ledger_reading(&readings);
        std::mem::forget(ledger.book_on_the_card(0, 12_000).expect("第一张借得到"));
        let bookings = ledger
            .book_states_on_the_cards_with_room(&[0, 1], 30, &[30, 30], &|states| states * 1_000)
            .expect("两张都借得到");
        let states_booked: Vec<Option<u64>> = bookings
            .iter()
            .map(|booked| booked.as_ref().map(|booked| booked.states))
            .collect();
        assert_eq!(states_booked, vec![Some(5), Some(13)]);
    }

    /// 第 10x 批：要的比卡空着时能借的（空卡上能用的减驱动那一份，29 550）还大，判成比每张卡都大（调用方判红）；不超过它、只是此刻
    /// 放不下的是借不到（调用方报本次未跑）。按状态数借时一个状态都放不下同理。
    #[test]
    fn a_need_larger_than_the_card_when_empty_never_fits_and_a_smaller_one_has_no_room_now() {
        let readings = replaceable_readings(vec![reading_of_an_rtx_5090(5_000)]);
        let ledger = ledger_reading(&readings);
        let ceiling = AN_EMPTY_RTX_5090_HAS - DRIVER;
        assert_eq!(
            booking_kept_on_the_books(&ledger, ceiling + 1),
            Err(BookingRefusal::NeedLargerThanEveryCard(
                NeedLargerThanEveryCard {
                    need_mebibytes: ceiling + 1,
                    largest_ceiling_mebibytes: ceiling,
                }
            ))
        );
        assert!(matches!(
            booking_kept_on_the_books(&ledger, ceiling),
            Err(BookingRefusal::NoCardHasRoomNow { .. })
        ));
        assert!(matches!(
            ledger.book_states_on_the_cards_with_room(&[0], 3, &[3], &|states| states
                * (ceiling + 1)),
            Err(BookingRefusal::NeedLargerThanEveryCard(_))
        ));
        assert!(matches!(
            ledger.book_states_on_the_cards_with_room(&[0], 3, &[3], &|states| states * 6_000),
            Err(BookingRefusal::NoCardHasRoomNow {
                need_mebibytes: 6_000,
                ..
            })
        ));
        assert!(
            !ledger.is_this_process_registered_on_the_card(0),
            "一笔都没借到：不登记"
        );
    }

    /// 读不出卡上的显存、读到的卡数与池对不上：判红（[`BookingRefusal::CardMemoryUnreadable`]），什么都不记。
    #[test]
    fn an_unreadable_card_memory_is_refused_and_nothing_is_booked() {
        let unreadable = GpuMemoryLedger::new(
            1,
            Box::new(|| Err("nvidia-smi 起不来".to_string())),
            Box::new(|_position, may_hold_mebibytes| may_hold_mebibytes),
            DRIVER,
        );
        assert_eq!(
            booking_kept_on_the_books(&unreadable, 1),
            Err(BookingRefusal::CardMemoryUnreadable {
                reason: "nvidia-smi 起不来".to_string()
            })
        );
        let too_few = GpuMemoryLedger::new(
            2,
            Box::new(|| Ok(vec![reading_of_an_rtx_5090(30_000)])),
            Box::new(|_position, may_hold_mebibytes| may_hold_mebibytes),
            DRIVER,
        );
        assert!(matches!(
            booking_kept_on_the_books(&too_few, 1),
            Err(BookingRefusal::CardMemoryUnreadable { .. })
        ));
        assert!(!unreadable.is_this_process_registered_on_the_card(0));
        assert!(!too_few.is_this_process_registered_on_the_card(0));
    }

    /// 第 10x 批「估计在压与借之间变小」那一形换了形态：调用方照空 30 000 时能借的压出 274 个每个 100 MiB 的状态；
    /// 借之前外人占了卡（空 10 000），借的时候在锁里照现读的重压成 74 个（7 400 ≤ 10 000 − 驱动那一份），借到。
    #[test]
    fn states_compressed_before_an_outsider_took_memory_are_compressed_again_in_the_booking() {
        let readings = replaceable_readings(vec![reading_of_an_rtx_5090(30_000)]);
        let ledger = ledger_reading(&readings);
        let need_of = |states: u64| states * 100;
        let room_when_compressed = ledger.card_rooms_mebibytes_now(&[0]).expect("读得到")[0];
        let states_compressed =
            most_states_whose_need_fits_the_capacity(1_000, room_when_compressed, &|states| {
                need_of(u64::try_from(states).expect("装得进 u64"))
            });
        assert_eq!(states_compressed, 274, "照 30 000 − 驱动那一份压出 274 个");
        replace_the_reading(&readings, 0, reading_of_an_rtx_5090(10_000));
        let bookings = ledger
            .book_states_on_the_cards_with_room(&[0], 274, &[274], &need_of)
            .expect("照现读的重压一次，借得到");
        let booked = bookings[0].as_ref().expect("只有这一张卡，借到了");
        assert_eq!(booked.states, 74, "照 10 000 − 驱动那一份重压成 74 个");
        assert_eq!(booked.booking.mebibytes(), 7_400);
    }

    /// 第 10v 批：点名一张卡借，只记在那张卡上：第一张空着也不挪过去；要的比点名的那张空着时能借的大、比第一张小的，
    /// 判成比卡大（判红），不记到第一张上。
    #[test]
    fn a_booking_on_a_named_card_stays_on_that_card_even_when_an_earlier_card_has_room() {
        let ledger = GpuMemoryLedger::new(
            2,
            Box::new(|| {
                Ok(vec![
                    reading_of_an_rtx_5090(30_000),
                    reading_of_an_rtx_5090(30_000),
                ])
            }),
            Box::new(|position, may_hold_mebibytes| match position {
                0 => may_hold_mebibytes.min(29_000),
                _ => may_hold_mebibytes.min(12_000),
            }),
            DRIVER,
        );
        let booking = ledger
            .book_on_the_card(1, 4_000)
            .expect("第二张卡空着，借得到");
        assert_eq!(booking.card_position(), 1, "点名第二张卡就记在第二张卡上");
        drop(booking);
        let refusal = ledger
            .book_on_the_card(1, 20_000)
            .map(|too_large_booking| too_large_booking.card_position())
            .expect_err("要的比第二张卡空着时能借的大");
        assert!(
            matches!(
                scratch_booking_refusal_outcome("crash_amplification::run", 1, &refusal),
                ScratchBookingRefusalOutcome::Red { .. }
            ),
            "比点名的那张大要判红，不挪到第一张卡上：{refusal:?}"
        );
    }

    /// 第 10x 批：产品路径此刻借不到照 GPU 对拍用例那一路报「本次未跑」，同一个记号、原因写显存不够；比卡大、读不出卡判红。
    #[test]
    fn a_booking_without_room_now_is_reported_not_run_with_the_gpu_not_run_marker() {
        let no_room = BookingRefusal::NoCardHasRoomNow {
            need_mebibytes: 4_000,
            card_rooms_mebibytes: vec![1_000],
        };
        let ScratchBookingRefusalOutcome::NotRun { not_run_line } =
            scratch_booking_refusal_outcome("crash_amplification::run", 0, &no_room)
        else {
            panic!("此刻借不到要报本次未跑");
        };
        assert!(
            not_run_line.starts_with(
                "NOT_RUN_NO_GPU_ADAPTER test=crash_amplification::run reason=显存不够："
            ),
            "本次未跑那一行用 GPU 用例的记号（门禁 feature-gated-tests 格认的就是这一串）、原因写显存不够：{not_run_line}"
        );
        for red in [
            BookingRefusal::NeedLargerThanEveryCard(NeedLargerThanEveryCard {
                need_mebibytes: 40_000,
                largest_ceiling_mebibytes: 30_047,
            }),
            BookingRefusal::CardMemoryUnreadable {
                reason: "nvidia-smi 起不来".to_string(),
            },
        ] {
            assert!(
                matches!(
                    scratch_booking_refusal_outcome("crash_amplification::run", 0, &red),
                    ScratchBookingRefusalOutcome::Red { .. }
                ),
                "{red:?} 判红"
            );
        }
    }

    /// 第 10t 批 K4-1：配置给这张卡 8 G、额度变量 20 000：容量不越过配置的额度（减驱动那一份），额度变量更小时取额度变量；
    /// 没设额度变量照旧。数照一张清场后空 30 000 MiB 的 5090。
    #[test]
    fn an_environment_quota_larger_than_the_configured_card_quota_stays_within_the_configured_card_quota(
    ) {
        const CONFIGURED_CARD_QUOTA: u64 = 8 * 1024;
        let budget = CONFIGURED_CARD_QUOTA.min(30_000);
        let capacity = card_ledger_capacity_mebibytes(
            Some(20_000),
            Some(CONFIGURED_CARD_QUOTA),
            budget,
            30_000,
            DRIVER,
        );
        assert!(
            capacity <= CONFIGURED_CARD_QUOTA,
            "额度变量 20 000 不越过配置给卡的 8 G：实际 {capacity}"
        );
        assert_eq!(
            capacity,
            CONFIGURED_CARD_QUOTA - DRIVER,
            "取配置额度减驱动那一份"
        );
        assert_eq!(
            card_ledger_capacity_mebibytes(
                Some(4_000),
                Some(CONFIGURED_CARD_QUOTA),
                budget,
                30_000,
                DRIVER
            ),
            4_000,
            "额度变量更小：取额度变量"
        );
        assert_eq!(
            card_ledger_capacity_mebibytes(
                None,
                Some(CONFIGURED_CARD_QUOTA),
                budget,
                30_000,
                DRIVER
            ),
            CONFIGURED_CARD_QUOTA - DRIVER,
            "没设额度变量：照旧是给崩溃放量的额度减驱动那一份"
        );
    }

    /// 一张空着的 5090 照 3/4 额度装得下 17 867 个状态（2026-10-06 日志里那一行）：45 个状态的流只开 45 个，
    /// 131 195 个状态的大流照旧按显存开，调用方要得更少就照它的，一点显存都不给、流是空的也带一个。
    #[test]
    fn a_small_flow_opens_scratch_for_its_own_states_and_a_large_flow_still_follows_the_memory() {
        const STATES_A_WHOLE_RTX_5090_HOLDS: usize = 17_867;
        assert_eq!(
            states_in_one_dispatch_capped_by_the_state_count(
                STATES_A_WHOLE_RTX_5090_HOLDS,
                usize::MAX,
                45
            ),
            45,
            "45 个状态的流只开 45 个状态的草稿区"
        );
        assert_eq!(
            states_in_one_dispatch_capped_by_the_state_count(
                STATES_A_WHOLE_RTX_5090_HOLDS,
                usize::MAX,
                131_195
            ),
            STATES_A_WHOLE_RTX_5090_HOLDS,
            "比显存装得下的还多的流照显存开"
        );
        assert_eq!(
            states_in_one_dispatch_capped_by_the_state_count(
                STATES_A_WHOLE_RTX_5090_HOLDS,
                256,
                131_195
            ),
            256,
            "调用方要的更少就照它的"
        );
        assert_eq!(
            states_in_one_dispatch_capped_by_the_state_count(0, usize::MAX, 0),
            1,
            "至少带一个"
        );
    }

    /// 设了卡的额度变量：容量取额度与「这个进程能占的 − 驱动那一份」里小的那个（额度小取额度，额度大取能占的那一边）；
    /// 没设：照旧是这张卡给崩溃放量的额度减驱动那一份。数照一张 5090（空 30 000 MiB，3/4 是 22 500）。
    #[test]
    fn a_card_quota_from_the_environment_caps_each_card_ledger_and_without_it_the_capacity_is_unchanged(
    ) {
        assert_eq!(
            card_ledger_capacity_mebibytes(Some(4096), None, 22_500, 30_000, DRIVER),
            4096,
            "额度比能占的减驱动那一份小：取额度"
        );
        assert_eq!(
            card_ledger_capacity_mebibytes(Some(40_000), None, 22_500, 30_000, DRIVER),
            22_500 - DRIVER,
            "额度比 3/4 那一份减驱动那一份大：没配显卡优先级时 3/4 那一份代替配置的额度，取它（第 10v 批）"
        );
        assert_eq!(
            card_ledger_capacity_mebibytes(None, None, 22_500, 30_000, DRIVER),
            22_500 - DRIVER,
            "没设额度：照旧是给崩溃放量的额度减驱动那一份"
        );
    }

    /// 第 10v 批：没配显卡优先级、设了额度变量 20 000、能占 30 000 MiB：没配额度时默认的 3/4 那一份（22 500）代替配置的额度，
    /// 并进最小值，容量不超过它减驱动那一份。
    #[test]
    fn an_environment_quota_without_a_card_priority_configured_stays_within_the_default_three_quarter_share(
    ) {
        const MAY_HOLD: u64 = 30_000;
        let three_quarter_share = MAY_HOLD / 4 * 3;
        let capacity = card_ledger_capacity_mebibytes(
            Some(20_000),
            None,
            three_quarter_share,
            MAY_HOLD,
            DRIVER,
        );
        assert!(
            capacity <= three_quarter_share - DRIVER,
            "额度变量 20 000 不越过 3/4 那一份 {three_quarter_share} 减驱动那一份：实际 {capacity}"
        );
        assert_eq!(
            capacity,
            three_quarter_share - DRIVER,
            "三者里最小的是 3/4 那一份减驱动那一份"
        );
    }

    /// 第 10v 批：起过卡池的进程，产品路径第 ① 段每张装了状态的卡按装的状态数借草稿区（45 个状态的小流只借 45 个，
    /// 一张 5090 装满 17 867 个、余下 3 000 个给下一张的大流各借各的），没装状态的卡不借；没起过卡池、不要卡的照旧不进账。
    #[test]
    fn a_process_that_started_the_card_pool_books_the_scratch_of_each_card_for_the_states_placed_on_it(
    ) {
        assert_eq!(
            scratch_states_to_book_on_each_card(
                GpuCardsSource::TheCardPoolOfThisProcess,
                &[45, 0],
                45
            ),
            ScratchStatesToBook::OnEachCard(vec![45, 0]),
            "45 个状态的流只在第一张卡上借 45 个状态的草稿区，第二张不借"
        );
        assert_eq!(
            scratch_states_to_book_on_each_card(
                GpuCardsSource::TheCardPoolOfThisProcess,
                &[17_867, 3_000],
                131_195
            ),
            ScratchStatesToBook::OnEachCard(vec![17_867, 3_000]),
            "大流按每张卡装的状态数借"
        );
        assert_eq!(
            scratch_states_to_book_on_each_card(
                GpuCardsSource::DevicesStartedForThisRun,
                &[45, 0],
                45
            ),
            ScratchStatesToBook::NotBooked,
            "没起过卡池的进程自己起设备，照旧不进账"
        );
        assert_eq!(
            scratch_states_to_book_on_each_card(GpuCardsSource::NoCardNeeded, &[], 45),
            ScratchStatesToBook::NotBooked,
            "不要卡的一趟不借"
        );
    }

    /// 第 10v 批：按判器那一族的算法（表 1 MiB，每个状态 1 261 616 字节）在 56 MiB 里装得下 45 个状态（45 个要 56、46 个要 57）；
    /// 上限内全放得下就取上限，一个都放不下取 0。
    #[test]
    fn the_most_states_whose_need_fits_is_the_largest_count_whose_need_is_within_the_capacity() {
        let judge_needs = |states: usize| -> u64 {
            1 + (u64::try_from(states).expect("装得进 u64") * 1_261_616).div_ceil(1024 * 1024)
        };
        assert_eq!(judge_needs(45), 56);
        assert_eq!(judge_needs(46), 57);
        assert_eq!(
            most_states_whose_need_fits_the_capacity(100_000, 56, &judge_needs),
            45,
            "56 MiB 装 45 个状态"
        );
        assert_eq!(
            most_states_whose_need_fits_the_capacity(30, 56, &judge_needs),
            30,
            "上限内全放得下取上限"
        );
        assert_eq!(
            most_states_whose_need_fits_the_capacity(100_000, 0, &judge_needs),
            0,
            "一个都放不下取 0"
        );
    }

    /// 这几条用例里每张卡都带得了状态：调用方算出一个都带不了的那一支走不到。
    fn every_card_here_holds_a_state(card_slot: usize) -> ScratchAttemptOnACard {
        panic!("第 {card_slot} 张卡带得了状态，不该问它一个都带不了时算哪一种")
    }

    /// 第 10w 批「几张卡一张借不到时不拿着别的卡干等」：100 个状态、两张卡各带得了 60 个；第一张当场借到 60 个，第二张此刻借不到。
    /// 第二张只试一次、要装的是剩下的 40 个，借不到就不装它，判成就在第一张上判（剩下的 40 个排队等第一张）。
    #[test]
    fn a_second_card_without_room_leaves_every_state_to_the_first_card() {
        let mut attempts_made = Vec::new();
        let attempts = place_states_on_the_cards_that_book_now(
            100,
            &[60, 60],
            &mut |card_slot, states_to_place| {
                attempts_made.push((card_slot, states_to_place));
                match card_slot {
                    0 => ScratchAttemptOnACard::Booked {
                        states: states_to_place,
                    },
                    _ => ScratchAttemptOnACard::NoRoomNow {
                        need_mebibytes: 2_000,
                    },
                }
            },
            &mut every_card_here_holds_a_state,
        );
        assert_eq!(
            attempts_made,
            vec![(0, 60), (1, 40)],
            "每张卡当场试一次：第一张装 60、第二张装剩下的 40"
        );
        assert_eq!(
            attempts,
            vec![
                ScratchAttemptOnACard::Booked { states: 60 },
                ScratchAttemptOnACard::NoRoomNow {
                    need_mebibytes: 2_000
                },
            ],
            "第二张借不到就不装状态"
        );
        assert_eq!(
            decide_after_attempting_every_card(&attempts),
            ScratchPlacementDecision::JudgeOnTheCardsBooked,
            "第一张借到了：就在第一张上判"
        );
    }

    /// 第 10w 批：借不到的卡那一份留给后面的卡。第一张此刻借不到，100 个状态里第二张装满它带得了的 60 个；
    /// 第一张借到的比要装的少（在锁里照此刻能借的压过，60 个里借到 30 个），第二张装的是剩下的 70 个与 60 里小的那个。
    #[test]
    fn the_states_a_card_did_not_book_go_to_the_next_card() {
        let first_card_without_room = place_states_on_the_cards_that_book_now(
            100,
            &[60, 60],
            &mut |card_slot, states_to_place| match card_slot {
                0 => ScratchAttemptOnACard::NoRoomNow {
                    need_mebibytes: 3_000,
                },
                _ => ScratchAttemptOnACard::Booked {
                    states: states_to_place,
                },
            },
            &mut every_card_here_holds_a_state,
        );
        assert_eq!(
            first_card_without_room[1],
            ScratchAttemptOnACard::Booked { states: 60 },
            "第一张借不到：第二张装满它带得了的 60 个"
        );
        let mut second_card_asked_for = 0;
        place_states_on_the_cards_that_book_now(
            100,
            &[60, 60],
            &mut |card_slot, states_to_place| match card_slot {
                0 => ScratchAttemptOnACard::Booked { states: 30 },
                _ => {
                    second_card_asked_for = states_to_place;
                    ScratchAttemptOnACard::Booked {
                        states: states_to_place,
                    }
                }
            },
            &mut every_card_here_holds_a_state,
        );
        assert_eq!(
            second_card_asked_for, 60,
            "第一张只借到 30 个：剩下的 70 个里第二张装它带得了的 60 个"
        );
    }

    /// 第 10x 批：一张都没借到时，有空着时放得下的卡就是此刻借不到（要的是第一张放不下的卡那一笔，调用方报本次未跑），
    /// 每张卡空着时连一个状态都放不下就判红。
    #[test]
    fn with_no_card_booked_a_card_that_fits_when_empty_means_no_room_now_and_none_means_red() {
        assert_eq!(
            decide_after_attempting_every_card(&[
                ScratchAttemptOnACard::CannotHoldOneState {
                    ceiling_mebibytes: 40
                },
                ScratchAttemptOnACard::NoRoomNow {
                    need_mebibytes: 3_000
                },
                ScratchAttemptOnACard::NoRoomNow {
                    need_mebibytes: 1_000
                },
            ]),
            ScratchPlacementDecision::NoRoomOnAnyCardNow {
                need_mebibytes: 3_000
            },
            "第二、三张空着时放得下：此刻借不到"
        );
        assert_eq!(
            decide_after_attempting_every_card(&[
                ScratchAttemptOnACard::CannotHoldOneState {
                    ceiling_mebibytes: 40
                },
                ScratchAttemptOnACard::CannotHoldOneState {
                    ceiling_mebibytes: 90
                },
            ]),
            ScratchPlacementDecision::NoCardHoldsOneState {
                largest_ceiling_mebibytes: 90
            },
            "每张都连一个状态都放不下：判红，带最大的那张空着时能借的"
        );
    }

    /// 第 10w 批：第 ② 段核对与单元校验和的缓冲进账的量。事实表 3 MiB 多 1 字节上卡一次，一趟派活建 512 KiB 多 48 字节、
    /// 同时两趟活着：合起来 4 MiB 多 97 字节，向上取整 5 MiB；什么都不建是 0。
    #[test]
    fn buffers_held_at_once_are_the_uploaded_tables_plus_every_pass_still_alive_rounded_up() {
        const MEBIBYTE: u64 = 1024 * 1024;
        assert_eq!(
            mebibytes_of_buffers_held_at_once(3 * MEBIBYTE + 1, MEBIBYTE / 2 + 48, 2),
            5,
            "3 MiB + 1 + 2 × (512 KiB + 48) = 4 MiB + 97 字节，向上取整"
        );
        assert_eq!(
            mebibytes_of_buffers_held_at_once(0, MEBIBYTE, 64),
            64,
            "32 条线程各两趟：64 趟各 1 MiB"
        );
        assert_eq!(mebibytes_of_buffers_held_at_once(0, 0, 0), 0);
    }

    /// 两张卡：前一张此刻放得下就一直记在前一张上，前一张放不下这一笔才记到下一张；每张都放不下而空着时放得下的是此刻借不到，
    /// 比每张空着时都大的等多久都放不下。
    #[test]
    fn bookings_fill_the_first_card_before_the_next_card_takes_any() {
        let room = |room_now_mebibytes: u64, ceiling_mebibytes: u64| CardRoomForThisProcess {
            room_now_mebibytes,
            ceiling_mebibytes,
        };
        assert_eq!(
            decide_booking_by_priority(&[room(29_000, 29_000), room(12_000, 12_000)], 600),
            BookingDecision::BookOnTheCardAt { card_position: 0 },
            "两张都空着：记在优先的那张"
        );
        assert_eq!(
            decide_booking_by_priority(&[room(600, 29_000), room(12_000, 12_000)], 600),
            BookingDecision::BookOnTheCardAt { card_position: 0 },
            "前一张剩的正好放得下：照旧记在前一张"
        );
        assert_eq!(
            decide_booking_by_priority(&[room(500, 29_000), room(12_000, 12_000)], 600),
            BookingDecision::BookOnTheCardAt { card_position: 1 },
            "前一张放不下这一笔：才记到下一张"
        );
        assert_eq!(
            decide_booking_by_priority(&[room(20, 100), room(10, 50)], 30),
            BookingDecision::NoRoomNow,
            "两张此刻都放不下 30，而第一张空着时放得下：此刻借不到"
        );
        assert_eq!(
            decide_booking_by_priority(&[room(100, 100), room(50, 50)], 101),
            BookingDecision::NeverFits {
                largest_ceiling_mebibytes: 100
            },
            "比最大的那张空着时还大：永远放不下"
        );
        assert_eq!(
            decide_booking_by_priority(&[], 1),
            BookingDecision::NeverFits {
                largest_ceiling_mebibytes: 0
            },
            "一张卡都没有：永远放不下"
        );
    }

    /// 一张卡此刻能借给这个进程的照 [`card_room_for_this_process`]：容量拿这个进程能占的算、减借着的；空着时能借的拿空卡上能用的
    /// （总量减保留）算、不减借着的。
    #[test]
    fn the_room_now_subtracts_the_leases_and_the_ceiling_takes_what_an_empty_card_has() {
        let room = card_room_for_this_process(
            CardMemoryReading {
                free_mebibytes: 20_000,
                total_less_reserved_mebibytes: AN_EMPTY_RTX_5090_HAS,
                occupied_by_this_process: OccupiedByThisProcess::NotListed,
            },
            BookedByThisProcessOnACard {
                leases_mebibytes: 3_000,
                driver_share: DriverShareOnTheCard::DeviceStarted,
            },
            DRIVER,
            &|may_hold_mebibytes| may_hold_mebibytes.saturating_sub(DRIVER),
        );
        assert_eq!(
            room,
            CardRoomForThisProcess {
                room_now_mebibytes: 20_000 - DRIVER - 3_000,
                ceiling_mebibytes: AN_EMPTY_RTX_5090_HAS - DRIVER,
            },
            "读不到实际占着的：驱动那一份不当已经在卡上，从现读的空闲里扣"
        );
    }

    /// 测试里进程号写死的「这个进程」与「别的进程」。
    const THIS_PROCESS: u32 = 4_242;
    const ANOTHER_PROCESS: u32 = 7_777;
    /// 测试里那张 5090 的总线号（`nvidia-smi` 写法，去掉域之前）。
    const BUS_OF_THE_RTX_5090: &str = "00000000:01:00.0";
    const BUS_OF_THE_RTX_5060_TI: &str = "00000000:10:00.0";

    /// 一张卡在假报告里的那一段：空闲与每个进程（进程号、占的 MiB）。总量、保留照 5090 的两个数写。
    struct CardInTheReport<'processes> {
        bus: &'processes str,
        total_mebibytes: u64,
        reserved_mebibytes: u64,
        free_mebibytes: u64,
        processes: &'processes [(u32, u64)],
    }

    /// 照本机驱动 595.84 的 `nvidia-smi -q -d MEMORY,PIDS` 文本写一份假报告（BAR1、Conf Compute 两小节也带着，
    /// 它们的 Total、Free 不该被当成显存读）。
    fn nvidia_smi_report(cards: &[CardInTheReport<'_>]) -> String {
        let mut report = format!(
            "\n==============NVSMI LOG==============\n\nTimestamp                                              : Wed Oct  7 14:38:07 2026\n\
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
    fn an_rtx_5090_in_the_report(
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

    fn no_device_outside_the_books(_bus: &str) -> bool {
        false
    }

    /// 第 10x 批代码轮判决 K1-1：一份报告里读出每张卡 FB 那一小节的空闲与空卡上能用的（BAR1 的不算），与这个进程那几行占的合计
    /// （别的进程的不算）；这个进程在一张卡上另有不走账的设备时那一份记成分不出；没列这个进程是没列；报告里没有点名的卡是读不出。
    #[test]
    fn one_nvidia_smi_report_gives_the_frame_buffer_memory_and_this_process_occupancy_of_each_card()
    {
        let report = nvidia_smi_report(&[
            an_rtx_5090_in_the_report(
                20_000,
                &[
                    (ANOTHER_PROCESS, 9_000),
                    (THIS_PROCESS, 3_000),
                    (THIS_PROCESS, 211),
                ],
            ),
            CardInTheReport {
                bus: BUS_OF_THE_RTX_5060_TI,
                total_mebibytes: 16_311,
                reserved_mebibytes: 462,
                free_mebibytes: 15_849,
                processes: &[(ANOTHER_PROCESS, 500)],
            },
        ]);
        let buses = vec![
            pci_bus_identifier_without_the_domain(BUS_OF_THE_RTX_5090),
            pci_bus_identifier_without_the_domain(BUS_OF_THE_RTX_5060_TI),
        ];
        assert_eq!(buses, vec!["01:00.0".to_string(), "10:00.0".to_string()]);
        assert_eq!(
            card_memory_readings_of_one_nvidia_smi_report(
                &report,
                THIS_PROCESS,
                &buses,
                &no_device_outside_the_books
            ),
            Ok(vec![
                CardMemoryReading {
                    free_mebibytes: 20_000,
                    total_less_reserved_mebibytes: AN_EMPTY_RTX_5090_HAS,
                    occupied_by_this_process: OccupiedByThisProcess::OnlyThroughTheBooks {
                        mebibytes: 3_211
                    },
                },
                CardMemoryReading {
                    free_mebibytes: 15_849,
                    total_less_reserved_mebibytes: 16_311 - 462,
                    occupied_by_this_process: OccupiedByThisProcess::NotListed,
                },
            ]),
            "FB 那一小节的空闲与总量减保留；这个进程两行相加，别的进程不算；第二张卡没列这个进程"
        );
        assert_eq!(
            card_memory_readings_of_one_nvidia_smi_report(&report, THIS_PROCESS, &buses, &|bus| {
                bus == "01:00.0"
            })
            .map(|readings| readings[0].occupied_by_this_process),
            Ok(OccupiedByThisProcess::AlsoOutsideTheBooks { mebibytes: 3_211 }),
            "这个进程在这张卡上另有不走账的设备：占的那一份分不出"
        );
        assert!(
            card_memory_readings_of_one_nvidia_smi_report(
                &report,
                THIS_PROCESS,
                &["02:00.0".to_string()],
                &no_device_outside_the_books
            )
            .is_err(),
            "报告里没有点名的卡：读不出"
        );
    }

    /// 第 10x 批代码轮判决 K1-1（两腿甲形）：每借一次只取一份报告，空闲与这个进程占着的出自那一份。假的报告按次序给：
    /// 第一份卡空着（这一笔借 3 000）；第二份是第一笔建上卡之后同一刻的空闲与占着的；第三份是外人又占了 10 000。
    /// 两次借各取一份、借的那一刻能借的照各自那一份算；取两份拼一份（空闲取前一份、占着的取后一份，或整份取后一份）都对不上。
    #[test]
    fn each_booking_reads_the_free_memory_and_this_process_occupancy_from_one_report() {
        let reports = [
            nvidia_smi_report(&[an_rtx_5090_in_the_report(30_000, &[])]),
            nvidia_smi_report(&[an_rtx_5090_in_the_report(
                30_000 - DRIVER - 3_000,
                &[(THIS_PROCESS, DRIVER + 3_000)],
            )]),
            nvidia_smi_report(&[an_rtx_5090_in_the_report(
                30_000 - DRIVER - 3_000 - 10_000,
                &[(THIS_PROCESS, DRIVER + 3_000), (ANOTHER_PROCESS, 10_000)],
            )]),
        ];
        let reports_taken = Arc::new(AtomicUsize::new(0));
        let taken_by_the_reader = Arc::clone(&reports_taken);
        let ledger = GpuMemoryLedger::new(
            1,
            card_memory_reader_of_one_report_per_reading(
                Box::new(move || {
                    let taken = taken_by_the_reader.fetch_add(1, Ordering::SeqCst);
                    reports
                        .get(taken)
                        .cloned()
                        .ok_or_else(|| format!("第 {} 份报告用例里没有", taken + 1))
                }),
                THIS_PROCESS,
                vec![pci_bus_identifier_without_the_domain(BUS_OF_THE_RTX_5090)],
                Box::new(no_device_outside_the_books),
            ),
            Box::new(|_position, may_hold_mebibytes| may_hold_mebibytes.saturating_sub(DRIVER)),
            DRIVER,
        );
        let first = ledger.book_on_the_card(0, 3_000).expect("空着的卡借得到");
        assert_eq!(
            first.card_room_mebibytes_when_booked(),
            30_000 - DRIVER,
            "第一笔照第一份报告：空 30 000，减还没起设备的驱动那一份"
        );
        let _device = ledger.record_the_device_started_on_the_card(0);
        let second = ledger
            .book_on_the_card(0, 1_000)
            .expect("第二份报告里还空着 24 440");
        assert_eq!(
            second.card_room_mebibytes_when_booked(),
            30_000 - DRIVER - 3_000,
            "第二笔照第二份报告：第一笔建上卡的 3 000 只从空闲里扣一次，不既在空闲里又在占着的里"
        );
        assert_eq!(
            reports_taken.load(Ordering::SeqCst),
            2,
            "借两次各取一份报告：空闲与占着的不分两次读"
        );
        drop(second);
        drop(first);
    }

    /// 第 10x 批代码轮判决 K1-1（攻方乙形）：这个进程在卡上占着 9 000，其中有不走这份账的设备（读数记成分不出）。
    /// 起了设备、借着 4 000 还没建：能借的照读不到实际占着的算，不把那 9 000 当成借据已经建上的那一份。
    #[test]
    fn memory_of_this_process_outside_the_books_is_not_taken_for_leases_already_built() {
        let booked = BookedByThisProcessOnACard {
            leases_mebibytes: 4_000,
            driver_share: DriverShareOnTheCard::DeviceStarted,
        };
        let reading = |occupied_by_this_process| CardMemoryReading {
            free_mebibytes: 15_000,
            total_less_reserved_mebibytes: AN_EMPTY_RTX_5090_HAS,
            occupied_by_this_process,
        };
        assert_eq!(
            memory_this_process_may_hold_mebibytes(
                reading(OccupiedByThisProcess::AlsoOutsideTheBooks { mebibytes: 9_000 }),
                booked,
                DRIVER
            ),
            15_000,
            "分不出：借着的 4 000 当一点都没建，起好了设备的驱动那一份也不当在卡上（第 10z 批：实占多少分不出）"
        );
        assert_eq!(
            memory_this_process_may_hold_mebibytes(
                reading(OccupiedByThisProcess::NotListed),
                booked,
                DRIVER
            ),
            15_000,
            "与没列这个进程同样算"
        );
        assert_eq!(
            memory_this_process_may_hold_mebibytes(
                reading(OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes: 9_000 }),
                booked,
                DRIVER
            ),
            15_000 + DRIVER + 4_000,
            "都走这份账时那 9 000 封顶到许出去的 6 560，才算借据建上了"
        );
    }

    /// 第 10z 批第六块（G2 乙，用户 2026-10-08 定「改用乙」）：第一笔 4 000 建上卡，第二笔 3 000 借着还没建；第一笔丢掉之前等完设备
    /// （缓冲交回驱动，读数里这个进程只剩驱动那一份）再销账：之后能借的是卡上空着的减第二笔还要建的、减驱动那一份，
    /// 销了账的 4 000 不再从空闲里扣（甲在这里多扣驱动那一份 2 560）。
    #[test]
    fn a_lease_returned_after_the_device_finished_is_no_longer_taken_off_the_free_memory() {
        const EMPTY: u64 = 30_000;
        const FIRST_LEASE: u64 = 4_000;
        const SECOND_LEASE: u64 = 3_000;
        let readings = replaceable_readings(vec![reading_of_an_rtx_5090(EMPTY)]);
        let ledger = ledger_reading(&readings);
        let mut first = ledger
            .book_on_the_card(0, FIRST_LEASE)
            .expect("空着的卡借得到");
        let device = ledger.record_the_device_started_on_the_card(0);
        replace_the_reading(
            &readings,
            0,
            CardMemoryReading {
                free_mebibytes: EMPTY - DRIVER - FIRST_LEASE,
                total_less_reserved_mebibytes: AN_EMPTY_RTX_5090_HAS,
                occupied_by_this_process: OccupiedByThisProcess::OnlyThroughTheBooks {
                    mebibytes: DRIVER + FIRST_LEASE,
                },
            },
        );
        let second = ledger
            .book_on_the_card(0, SECOND_LEASE)
            .expect("卡上空着 23 440，借得到");
        first.take_in_the_wait_before_returning(
            &device,
            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
        );
        drop(first);
        replace_the_reading(
            &readings,
            0,
            CardMemoryReading {
                free_mebibytes: EMPTY - DRIVER,
                total_less_reserved_mebibytes: AN_EMPTY_RTX_5090_HAS,
                occupied_by_this_process: OccupiedByThisProcess::OnlyThroughTheBooks {
                    mebibytes: DRIVER,
                },
            },
        );
        let room_now = ledger.card_rooms_mebibytes_now(&[0]).expect("假读数读得出")[0];
        assert_eq!(
            room_now + SECOND_LEASE + DRIVER,
            EMPTY,
            "能借的 {room_now} 加第二笔还要建的、加驱动那一份，是卡上空着的：销了账的第一笔不再扣"
        );
        assert!(
            matches!(
                ledger.book_on_the_card(0, room_now + 1),
                Err(BookingRefusal::NoCardHasRoomNow { .. })
            ),
            "多 1 MiB 就借不到"
        );
        drop(second);
    }

    /// 第 10x 批代码轮判决 K1-1（攻方丙形）：报告里一张空着的 5090 空 32 110（总量 32 607 减保留 497）。要的比空卡上能用的减驱动那一份
    /// （29 550）大 1 MiB：判成比每张卡都大（判红），不是永远借不到的本次未跑。
    #[test]
    fn a_need_beyond_what_an_empty_card_has_never_fits_even_below_the_total_memory() {
        let report = nvidia_smi_report(&[an_rtx_5090_in_the_report(5_000, &[])]);
        let ledger = GpuMemoryLedger::new(
            1,
            card_memory_reader_of_one_report_per_reading(
                Box::new(move || Ok(report.clone())),
                THIS_PROCESS,
                vec![pci_bus_identifier_without_the_domain(BUS_OF_THE_RTX_5090)],
                Box::new(no_device_outside_the_books),
            ),
            Box::new(|_position, may_hold_mebibytes| may_hold_mebibytes.saturating_sub(DRIVER)),
            DRIVER,
        );
        let ceiling = AN_EMPTY_RTX_5090_HAS - DRIVER;
        assert!(ceiling + 1 < TOTAL_OF_AN_RTX_5090 - DRIVER);
        assert_eq!(
            booking_kept_on_the_books(&ledger, ceiling + 1),
            Err(BookingRefusal::NeedLargerThanEveryCard(
                NeedLargerThanEveryCard {
                    need_mebibytes: ceiling + 1,
                    largest_ceiling_mebibytes: ceiling,
                }
            )),
            "空卡上能用的装不下：判红，不照总量判成此刻借不到"
        );
    }

    /// 第 10x 批代码轮判决 K2-2：调用方照此刻能借的算出这张卡一个状态都带不了（带得了 0 个），而空着时放得下：
    /// 判成此刻借不到（调用方报本次未跑），不当成前面的卡装完了、一张都没借到就判红。
    #[test]
    fn a_card_the_caller_found_holding_no_state_now_is_no_room_now_not_nothing_left_to_place() {
        let readings = replaceable_readings(vec![reading_of_an_rtx_5090(DRIVER + 1_000)]);
        let ledger = ledger_reading(&readings);
        assert_eq!(
            ledger
                .book_states_on_the_cards_with_room(&[0], 10, &[0], &|states| states * 2_000)
                .map(|bookings| bookings.len()),
            Err(BookingRefusal::NoCardHasRoomNow {
                need_mebibytes: 2_000,
                card_rooms_mebibytes: vec![1_000],
            }),
            "此刻带不了、空着时放得下：此刻借不到"
        );
        assert_eq!(
            place_states_on_the_cards_that_book_now(
                10,
                &[4, 0],
                &mut |_card_slot, states_to_place| ScratchAttemptOnACard::Booked {
                    states: states_to_place
                },
                &mut |_card_slot| ScratchAttemptOnACard::NoRoomNow {
                    need_mebibytes: 2_000
                },
            ),
            vec![
                ScratchAttemptOnACard::Booked { states: 4 },
                ScratchAttemptOnACard::NoRoomNow {
                    need_mebibytes: 2_000
                },
            ],
            "还剩 6 个要装、第二张此刻带不了：是此刻借不到，不是没有要装的"
        );
    }

    /// 第 10x 批代码轮判决 K2-2（白红那一处）：借草稿区之前，每张卡此刻都带不了一个状态而至少一张空着时带得了，是此刻借不到
    /// （报本次未跑）；空着时也带不了（一张卡都没有也算）才判红；有一张此刻带得了就去借。
    #[test]
    fn with_no_card_holding_a_state_now_a_card_holding_one_when_empty_means_not_run_not_red() {
        assert_eq!(
            decide_scratch_before_booking(&[0, 0], &[0, 3]),
            ScratchBeforeBooking::NoCardHoldsAStateNow,
            "第二张空着时带得了 3 个：此刻借不到"
        );
        assert_eq!(
            decide_scratch_before_booking(&[0, 0], &[0, 0]),
            ScratchBeforeBooking::NoCardHoldsAStateEvenWhenEmpty,
            "空着时也带不了：判红"
        );
        assert_eq!(
            decide_scratch_before_booking(&[], &[]),
            ScratchBeforeBooking::NoCardHoldsAStateEvenWhenEmpty,
            "一张卡都没有：判红"
        );
        assert_eq!(
            decide_scratch_before_booking(&[0, 2], &[5, 5]),
            ScratchBeforeBooking::BookOnTheCardsThatHoldAStateNow,
            "第二张此刻带得了 2 个：去借"
        );
    }

    /// 第 10x 批代码轮判决 K1-3、K3：卡上 wgpu 报 Out of Memory 那一句以「显存不够：」起头，打成本次未跑那一行时跟着 GPU 用例的记号
    /// （门禁 feature-gated-tests 格认记号，人认这几个字）。
    #[test]
    fn the_out_of_memory_reason_says_the_gpu_memory_was_short_in_the_not_run_line() {
        let reason = reason_for_running_out_of_gpu_memory("序号 0 的卡建判器时");
        assert!(
            reason.starts_with(GPU_MEMORY_SHORT_REASON_PREFIX),
            "原因写显存不够：{reason}"
        );
        assert!(
            gpu_not_run_line("crash_amplification::run", &reason).starts_with(
                "NOT_RUN_NO_GPU_ADAPTER test=crash_amplification::run reason=显存不够：序号 0 的卡建判器时"
            ),
            "本次未跑那一行：GPU 用例的记号、原因写显存不够"
        );
    }

    /// 第 10x 批代码轮判决 K3：按状态数借时同一张卡给了两次要断言（两笔记在一张卡上、各按那张卡此刻能借的压，会借超）。
    #[test]
    fn the_same_card_given_twice_to_a_scratch_booking_is_refused_by_an_assertion() {
        let readings = replaceable_readings(vec![reading_of_an_rtx_5090(30_000)]);
        let ledger = ledger_reading(&readings);
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                ledger
                    .book_states_on_the_cards_with_room(&[0, 0], 4, &[2, 2], &|states| states * 100)
                    .map(|bookings| bookings.len())
            }))
            .is_err(),
            "同一张卡给了两次：断言"
        );
        assert!(
            !ledger.is_this_process_registered_on_the_card(0),
            "断言在借之前：什么都没记"
        );
    }

    /// 第 10x 批代码轮判决 K3：读到的卡比池里多（一张卡的池读到两张）也判读不出，什么都不记。
    #[test]
    fn a_reading_of_more_cards_than_the_pool_is_refused_and_nothing_is_booked() {
        let too_many = GpuMemoryLedger::new(
            1,
            Box::new(|| {
                Ok(vec![
                    reading_of_an_rtx_5090(30_000),
                    reading_of_an_rtx_5090(30_000),
                ])
            }),
            Box::new(|_position, may_hold_mebibytes| may_hold_mebibytes),
            DRIVER,
        );
        assert!(matches!(
            booking_kept_on_the_books(&too_many, 1),
            Err(BookingRefusal::CardMemoryUnreadable { .. })
        ));
        assert!(!too_many.is_this_process_registered_on_the_card(0));
    }

    /// 第 10x 批代码轮判决 K1-3：卡池每张卡的设备第一次起不来不留着，下一次要时再起；起好了留着给后面要的；留着的那个丢了就重起一个。
    #[test]
    fn a_device_that_failed_to_start_is_started_again_and_a_lost_one_is_replaced() {
        let slot: DeviceStartedOnDemand<u32> = DeviceStartedOnDemand::new();
        let starts = AtomicUsize::new(0);
        let start_as = |device: u32| {
            starts.fetch_add(1, Ordering::SeqCst);
            Ok::<u32, String>(device)
        };
        let never_lost = |_device: &u32| false;
        assert_eq!(
            slot.started_or_start(never_lost, || Err::<u32, String>("显存不够".to_string())),
            Err("显存不够".to_string()),
            "第一次起不来：交起不来的那一句"
        );
        assert!(!slot.is_started(), "起不来不留着");
        assert_eq!(
            slot.started_or_start(never_lost, || start_as(1)),
            Ok(1),
            "下一次要时再起，起得来"
        );
        assert_eq!(
            slot.started_or_start(never_lost, || start_as(2)),
            Ok(1),
            "起好了留着：后面要的拿同一个，不再起"
        );
        assert_eq!(starts.load(Ordering::SeqCst), 1);
        assert_eq!(
            slot.started_or_start(|device: &u32| *device == 1, || start_as(3)),
            Ok(3),
            "留着的那个丢了：重起一个"
        );
        assert_eq!(starts.load(Ordering::SeqCst), 2);
        assert!(slot.is_started());
    }

    /// 第 10z 批第十一块（第 10z 批第七八九块代码轮判决 K4-2，正推 `probe_started_now_blocks.rs`：取留着的设备与起设备共用一把锁，
    /// 300 毫秒的重起里借之前那一下等了 250 毫秒）：留着的设备 1 丢了，一条线程在重起、起到一半停住；这时取留着的设备不等，
    /// 交起之前留着的那一台（丢了的 1，卡池借之前不 poll 丢了的）；重起完了交新起的 2。
    #[test]
    fn taking_the_device_kept_does_not_wait_for_a_restart_in_progress() {
        let slot: DeviceStartedOnDemand<u32> = DeviceStartedOnDemand::new();
        assert_eq!(
            slot.started_or_start(|_device: &u32| false, || Ok::<u32, String>(1)),
            Ok(1)
        );
        let slot_shared = &slot;
        let (restart_began_sender, restart_began) = std::sync::mpsc::channel::<()>();
        let (finish_the_restart, restart_may_finish) = std::sync::mpsc::channel::<()>();
        let (taken_sender, taken) = std::sync::mpsc::channel::<Option<u32>>();
        std::thread::scope(|scope| {
            let restarting = scope.spawn(move || {
                slot_shared.started_or_start(
                    |device: &u32| *device == 1,
                    move || {
                        restart_began_sender.send(()).expect("用例在等重起起头");
                        // 用例断言失败、放开之前就丢了发的那一头时也收尾
                        let _finished_or_the_test_gave_up = restart_may_finish.recv();
                        Ok::<u32, String>(2)
                    },
                )
            });
            restart_began.recv().expect("重起起头了");
            scope.spawn(move || {
                taken_sender
                    .send(slot_shared.started_now())
                    .expect("用例在等取到的那一台");
            });
            let taken_during_the_restart = taken.recv_timeout(std::time::Duration::from_secs(10));
            finish_the_restart.send(()).expect("重起在等放开");
            assert_eq!(
                taken_during_the_restart,
                Ok(Some(1)),
                "重起着的时候取留着的设备不等：交起之前留着的那一台（丢了的 1）"
            );
            assert_eq!(restarting.join().expect("重起不 panic"), Ok(2));
        });
        assert_eq!(
            slot.started_now(),
            Some(2),
            "重起完了：留着的是新起的那一台"
        );
    }

    /// 第 10z 批第十一块（K4-2 拆了锁之后）：同一张卡上一条线程在重起，另一条也要这张卡的设备：等这一次起完、拿它起好的那一台，不另起一台。
    #[test]
    fn a_second_start_on_the_same_card_waits_for_the_restart_in_progress_and_takes_its_device() {
        let slot: DeviceStartedOnDemand<u32> = DeviceStartedOnDemand::new();
        assert_eq!(
            slot.started_or_start(|_device: &u32| false, || Ok::<u32, String>(1)),
            Ok(1)
        );
        let slot_shared = &slot;
        let starts_by_the_second = AtomicUsize::new(0);
        let starts_by_the_second_shared = &starts_by_the_second;
        let (restart_began_sender, restart_began) = std::sync::mpsc::channel::<()>();
        let (finish_the_restart, restart_may_finish) = std::sync::mpsc::channel::<()>();
        let (second_sender, second) = std::sync::mpsc::channel::<Result<u32, String>>();
        std::thread::scope(|scope| {
            let restarting = scope.spawn(move || {
                slot_shared.started_or_start(
                    |device: &u32| *device == 1,
                    move || {
                        restart_began_sender.send(()).expect("用例在等重起起头");
                        let _finished_or_the_test_gave_up = restart_may_finish.recv();
                        Ok::<u32, String>(2)
                    },
                )
            });
            restart_began.recv().expect("重起起头了");
            scope.spawn(move || {
                second_sender
                    .send(slot_shared.started_or_start(
                        |device: &u32| *device == 1,
                        || {
                            starts_by_the_second_shared.fetch_add(1, Ordering::SeqCst);
                            Ok::<u32, String>(3)
                        },
                    ))
                    .expect("用例在等第二条拿到的那一台");
            });
            let second_before_the_restart_finished =
                second.recv_timeout(std::time::Duration::from_millis(500));
            finish_the_restart.send(()).expect("重起在等放开");
            assert_eq!(
                second_before_the_restart_finished,
                Err(std::sync::mpsc::RecvTimeoutError::Timeout),
                "第一条还在起：第二条等它起完，不先交回"
            );
            assert_eq!(restarting.join().expect("重起不 panic"), Ok(2));
            assert_eq!(
                second.recv_timeout(std::time::Duration::from_secs(10)),
                Ok(Ok(2)),
                "第二条拿的是第一条起好的那一台"
            );
        });
        assert_eq!(
            starts_by_the_second.load(Ordering::SeqCst),
            0,
            "第二条没另起一台"
        );
    }

    /// 第 10z 批（第 10y 批代码轮判决 K1-1 攻方乙形）：不走账的设备起之前就记下它在哪张卡上：起的那一段里卡池读这张卡已经认得出
    /// 这个进程另有不走账的设备；没起过的卡不记。
    #[test]
    fn a_device_outside_the_books_is_marked_on_its_bus_before_it_starts() {
        let buses = BusesWithDevicesStartedOutsideTheBooks::new();
        assert_eq!(
            buses.start_a_device_marked_first("01:00.0", || {
                Ok::<bool, String>(buses.has_devices_on_the_bus("01:00.0"))
            }),
            Ok(true),
            "起的那一段里已经记着"
        );
        assert!(buses.has_devices_on_the_bus("01:00.0"));
        assert!(!buses.has_devices_on_the_bus("10:00.0"), "没起过的卡不记");
    }

    /// 第 10z 批：不走账的设备起不来照样记着（分不出它占没占，只会少借）。
    #[test]
    fn a_device_outside_the_books_that_failed_to_start_stays_marked_on_its_bus() {
        let buses = BusesWithDevicesStartedOutsideTheBooks::new();
        assert_eq!(
            buses
                .start_a_device_marked_first("10:00.0", || Err::<(), String>("起不来".to_string())),
            Err("起不来".to_string())
        );
        assert!(buses.has_devices_on_the_bus("10:00.0"));
    }

    /// 第 10z 批（第 10y 批代码轮判决 K1-1 攻方乙附，数照攻方模型那一行：空 21 010，借着 8 000 还没建，驱动实占 600）：
    /// 设备起好了而报告里没列这个进程，驱动那一份不当整份在卡上：能借的是空闲减驱动那一份减借着的 10 450，不超过卡上此刻真能再给的
    /// 11 050（空闲减驱动还要长的 1 960 减借着的）；当整份在卡上是 13 010，多给 D − x。
    #[test]
    fn with_this_process_not_listed_a_started_device_leaves_room_for_its_whole_driver_share() {
        const FREE: u64 = 21_010;
        const LEASES: u64 = 8_000;
        const DRIVER_HOLDS_NOW: u64 = 600;
        let room = card_room_for_this_process(
            reading_of_an_rtx_5090(FREE),
            BookedByThisProcessOnACard {
                leases_mebibytes: LEASES,
                driver_share: DriverShareOnTheCard::DeviceStarted,
            },
            DRIVER,
            &|may_hold_mebibytes| may_hold_mebibytes.saturating_sub(DRIVER),
        );
        assert_eq!(room.room_now_mebibytes, FREE - DRIVER - LEASES);
        assert!(
            room.room_now_mebibytes <= FREE - (DRIVER - DRIVER_HOLDS_NOW) - LEASES,
            "不超过卡上此刻真能再给的：{room:?}"
        );
    }

    /// 第 10z 批第六块（G2 乙；数照第 10y 批代码轮判决 K1-1 攻方丙形那一行：5090 空卡 32 110，外人占 10 000，借 4 000、设备起好、建满，
    /// 驱动实占 600）：那 4 000 丢掉之前等完设备、交回驱动再销账，读数里这个进程只剩驱动实占的 600；下一笔能借的是卡上此刻真能再给的
    /// 19 550（空 21 510 减驱动还要长的 1 960），不把驱动还没长到的那一截当成能借的（多给 D − x）。
    #[test]
    fn a_lease_returned_after_the_device_finished_leaves_room_only_for_what_the_driver_still_grows_to(
    ) {
        const EMPTY: u64 = 32_110;
        const OUTSIDER: u64 = 10_000;
        const DRIVER_HOLDS_NOW: u64 = 600;
        const LEASE: u64 = 4_000;
        let readings = replaceable_readings(vec![CardMemoryReading {
            free_mebibytes: EMPTY - OUTSIDER,
            total_less_reserved_mebibytes: EMPTY,
            occupied_by_this_process: OccupiedByThisProcess::NotListed,
        }]);
        let shared = Arc::clone(&readings);
        let ledger = GpuMemoryLedger::new(
            1,
            Box::new(move || Ok(shared.lock().expect("用例里只有一条线程改读数").clone())),
            Box::new(|_position, may_hold_mebibytes| {
                card_ledger_capacity_mebibytes(
                    None,
                    Some(32_000),
                    may_hold_mebibytes.min(32_000),
                    may_hold_mebibytes,
                    DRIVER,
                )
            }),
            DRIVER,
        );
        let mut lease = ledger.book_on_the_card(0, LEASE).expect("卡上空着 22 110");
        let device = ledger.record_the_device_started_on_the_card(0);
        lease.take_in_the_wait_before_returning(
            &device,
            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
        );
        drop(lease);
        let free_now = EMPTY - OUTSIDER - DRIVER_HOLDS_NOW;
        replace_the_reading(
            &readings,
            0,
            CardMemoryReading {
                free_mebibytes: free_now,
                total_less_reserved_mebibytes: EMPTY,
                occupied_by_this_process: OccupiedByThisProcess::OnlyThroughTheBooks {
                    mebibytes: DRIVER_HOLDS_NOW,
                },
            },
        );
        let room_now = ledger.card_rooms_mebibytes_now(&[0]).expect("假读数读得出")[0];
        assert_eq!(
            room_now + (DRIVER - DRIVER_HOLDS_NOW),
            free_now,
            "能借的 {room_now} 加驱动还要长的 1 960 是卡上空着的 21 510：不多给"
        );
        assert_eq!(
            ledger.card_rooms_mebibytes_now(&[0]),
            Ok(vec![room_now]),
            "再读一次照旧"
        );
    }

    /// 第 10z 批第六块（G2 乙；数照第 10y 批代码轮判决 K2-2 P1、正推在真账上造出的那一组 S10）：一笔 10 000 的长借据建满之后，十笔 1 000 的短借据
    /// 各借、建、等完设备再丢（丢了之后读数里这个进程占的回到驱动那一份加 10 000）。销了账的不再记：第 k 笔还了之后能借的照旧是真空闲 19 550，
    /// 第 10 笔之后要 18 550 照样借得到（甲在这里每笔多扣 1 000，第 10 笔之后只给 9 550、报本次未跑）。
    #[test]
    fn returned_short_leases_while_a_long_lease_stays_built_leave_the_room_as_it_was() {
        const EMPTY: u64 = 32_110;
        const LONG_LEASE: u64 = 10_000;
        const SHORT_LEASE: u64 = 1_000;
        let readings = replaceable_readings(vec![CardMemoryReading {
            free_mebibytes: EMPTY,
            total_less_reserved_mebibytes: EMPTY,
            occupied_by_this_process: OccupiedByThisProcess::NotListed,
        }]);
        let shared = Arc::clone(&readings);
        let ledger = GpuMemoryLedger::new(
            1,
            Box::new(move || Ok(shared.lock().expect("用例里只有一条线程改读数").clone())),
            Box::new(|_position, may_hold_mebibytes| {
                card_ledger_capacity_mebibytes(
                    None,
                    Some(40_000),
                    may_hold_mebibytes.min(40_000),
                    may_hold_mebibytes,
                    DRIVER,
                )
            }),
            DRIVER,
        );
        let occupied = |free_mebibytes: u64, mebibytes: u64| CardMemoryReading {
            free_mebibytes,
            total_less_reserved_mebibytes: EMPTY,
            occupied_by_this_process: OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes },
        };
        let long_lease = ledger.book_on_the_card(0, LONG_LEASE).expect("空卡借得到");
        let device = ledger.record_the_device_started_on_the_card(0);
        let free_while_the_long_lease_is_built = EMPTY - DRIVER - LONG_LEASE;
        replace_the_reading(
            &readings,
            0,
            occupied(free_while_the_long_lease_is_built, DRIVER + LONG_LEASE),
        );
        assert_eq!(
            ledger.card_rooms_mebibytes_now(&[0]),
            Ok(vec![free_while_the_long_lease_is_built]),
            "长借据建满：能借的是真空闲"
        );
        for short_lease_number in 1..=10 {
            let mut short_lease = ledger
                .book_on_the_card(0, SHORT_LEASE)
                .expect("卡上空着 19 550");
            replace_the_reading(
                &readings,
                0,
                occupied(
                    free_while_the_long_lease_is_built - SHORT_LEASE,
                    DRIVER + LONG_LEASE + SHORT_LEASE,
                ),
            );
            short_lease.take_in_the_wait_before_returning(
                &device,
                DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
            );
            drop(short_lease);
            replace_the_reading(
                &readings,
                0,
                occupied(free_while_the_long_lease_is_built, DRIVER + LONG_LEASE),
            );
            let room_now = ledger.card_rooms_mebibytes_now(&[0]).expect("假读数读得出")[0];
            assert_eq!(
                room_now + DRIVER + LONG_LEASE,
                EMPTY,
                "第 {short_lease_number} 笔短借据等完设备、还了：能借的 {room_now} 加长借据与驱动那一份是空卡，销了账的几笔不再扣"
            );
        }
        let need = free_while_the_long_lease_is_built - SHORT_LEASE;
        assert_eq!(
            booking_kept_on_the_books(&ledger, need),
            Ok(need),
            "要 18 550、真空闲 19 550：借得到"
        );
        drop(long_lease);
    }

    /// 第 10z 批的两条重起用例共用：一张 5090 上空 30 000，借一笔 10 000、设备起好、建满（卡上空 17 440），再借 `second_lease_mebibytes`
    /// 还没建。交账与换读数的那一份（卡上的读数停在建满那一刻：丢了的设备还占着）。
    fn ledger_whose_started_device_holds_a_built_lease_and_another_unbuilt(
        second_lease_mebibytes: u64,
    ) -> (GpuMemoryLedger, ReplaceableReadings) {
        let readings = replaceable_readings(vec![reading_of_an_rtx_5090(30_000)]);
        let ledger = ledger_reading(&readings);
        booking_kept_on_the_books(&ledger, 10_000).expect("空着的卡借得到");
        assert_eq!(
            ledger.prepare_the_driver_share_before_starting_the_device(0),
            Ok(())
        );
        record_the_device_started_and_kept_by_the_pool(&ledger, 0);
        replace_the_reading(
            &readings,
            0,
            CardMemoryReading {
                free_mebibytes: 30_000 - DRIVER - 10_000,
                total_less_reserved_mebibytes: AN_EMPTY_RTX_5090_HAS,
                occupied_by_this_process: OccupiedByThisProcess::OnlyThroughTheBooks {
                    mebibytes: DRIVER + 10_000,
                },
            },
        );
        booking_kept_on_the_books(&ledger, second_lease_mebibytes)
            .expect("建满之后卡上还空着 17 440");
        (ledger, readings)
    }

    /// 第 10z 批（第 10y 批代码轮判决 K1-3 攻方）：起过的设备丢了要重起，驱动那一份重新登记：卡上空 17 440（丢了的设备还占着），
    /// 第二笔 17 440 还没建，再起一个设备连它的驱动那一份要 20 000，放不下：交借不到，这个进程照旧登记着（驱动那一份许出去而没占上卡）。
    #[test]
    fn a_lost_device_started_again_is_refused_when_its_driver_share_no_longer_fits_now() {
        let (ledger, _readings) =
            ledger_whose_started_device_holds_a_built_lease_and_another_unbuilt(
                30_000 - DRIVER - 10_000,
            );
        assert_eq!(
            ledger.prepare_the_driver_share_before_starting_the_device(0),
            Err(BookingRefusal::NoCardHasRoomNow {
                need_mebibytes: DRIVER,
                card_rooms_mebibytes: vec![0],
            }),
            "新设备的驱动那一份放不下：不起"
        );
        assert!(ledger.is_this_process_registered_on_the_card(0));
    }

    /// 第 10z 批：同上，第二笔只有 2 000：再起的设备连驱动那一份照此刻的空闲放得下，起；之后能借的照新设备的驱动那一份还没占上卡、
    /// 这个进程占着的分不出算（第 10z 批第六块 G2 乙：17 440 减驱动那一份、减借着的 12 000，建上了的 10 000 也照没建扣），
    /// 不照丢了的那一份当已在卡上。
    #[test]
    fn a_lost_device_started_again_books_its_driver_share_as_not_yet_on_the_card() {
        let (ledger, _readings) =
            ledger_whose_started_device_holds_a_built_lease_and_another_unbuilt(2_000);
        assert_eq!(
            ledger.card_rooms_mebibytes_now(&[0]),
            Ok(vec![30_000 - DRIVER - 10_000 - 2_000]),
            "丢之前：驱动那一份在卡上"
        );
        assert_eq!(
            ledger.prepare_the_driver_share_before_starting_the_device(0),
            Ok(()),
            "新设备连驱动那一份放得下"
        );
        let room_now = ledger.card_rooms_mebibytes_now(&[0]).expect("假读数读得出")[0];
        assert_eq!(
            room_now + DRIVER + 10_000 + 2_000,
            30_000 - DRIVER - 10_000,
            "重起之前：能借的 {room_now} 加新设备的驱动那一份、加借着的两笔，是此刻空着的 17 440"
        );
    }

    /// 第 10z 批（第 10y 批代码轮判决 K1-3 攻方）：同一个设备上，别的线程上那段活记的 Out of Memory 说不成这段活的显存不够
    /// （这段活真正的 panic 照样往外抛）；这段活自己这条线程上记的说得成。
    #[test]
    fn an_out_of_memory_in_the_work_of_another_thread_does_not_explain_a_failure_of_this_work() {
        let condition = DeviceConditionOfACard::default();
        let this_work = WorkOnTheCardOfThisThread::begin();
        let reports_before = condition.out_of_memory_reports();
        std::thread::scope(|scope| {
            scope.spawn(|| {
                let _another_work = WorkOnTheCardOfThisThread::begin();
                condition.record_an_out_of_memory("别的用例在这张卡上撞满".to_string());
            });
        });
        assert_eq!(condition.out_of_memory_reports(), reports_before + 1);
        assert_eq!(
            condition.out_of_memory_explaining_a_failure_of_the_work(
                this_work.identifier(),
                reports_before,
                DeviceWhenTheWorkReturned::NotLost,
            ),
            None,
            "别的线程上那段活的 Out of Memory：说不成这段活的显存不够"
        );
        condition.record_an_out_of_memory("这段活自己判一块时".to_string());
        assert_eq!(
            condition.out_of_memory_explaining_a_failure_of_the_work(
                this_work.identifier(),
                reports_before,
                DeviceWhenTheWorkReturned::NotLost,
            ),
            Some("这段活自己判一块时".to_string()),
            "这段活自己这条线程上记的：说得成"
        );
    }

    /// 第 10z 批：判器内核在活另起的线程上编，那几条线程上没有活；在这个设备上为这段活编内核的那一刻记的 Out of Memory 算这段活的，
    /// 编完之后没有活的线程上记的哪段活都说不成。
    #[test]
    fn an_out_of_memory_on_a_thread_without_work_explains_only_the_work_compiling_on_the_device_then(
    ) {
        let condition = DeviceConditionOfACard::default();
        let compiling_work = WorkOnTheCardOfThisThread::begin();
        let reports_before = condition.out_of_memory_reports();
        condition.compile_for_the_work_of_this_thread(|| {
            std::thread::scope(|scope| {
                scope.spawn(|| {
                    condition
                        .record_an_out_of_memory("另起的线程上编恢复内核时没接住的".to_string());
                });
            });
        });
        assert_eq!(
            condition.out_of_memory_explaining_a_failure_of_the_work(
                compiling_work.identifier(),
                reports_before,
                DeviceWhenTheWorkReturned::NotLost,
            ),
            Some("另起的线程上编恢复内核时没接住的".to_string()),
            "编内核的那一刻另起的线程上记的：算这段活的"
        );
        let reports_after_compiling = condition.out_of_memory_reports();
        std::thread::scope(|scope| {
            scope.spawn(|| {
                condition.record_an_out_of_memory("编完之后没有活的线程上记的".to_string());
            });
        });
        assert_eq!(
            condition.out_of_memory_explaining_a_failure_of_the_work(
                compiling_work.identifier(),
                reports_after_compiling,
                DeviceWhenTheWorkReturned::NotLost,
            ),
            None,
            "编完之后没有活的线程上记的：说不成"
        );
    }

    /// 第 10z 批（第 10y 批代码轮正推）：设备为别的原因丢了，而这段活开始之后没记过 Out of Memory（早先一段活上处理过的一次不算），
    /// 说不成显存不够；这段活开始之后记过（别的活上的也算：丢了设备谁都做不成），说成设备因它丢了。
    #[test]
    fn a_lost_device_explains_a_failure_only_with_an_out_of_memory_since_the_work_began() {
        let condition = DeviceConditionOfACard::default();
        condition.record_an_out_of_memory("早先处理过的一次".to_string());
        let work = WorkOnTheCardOfThisThread::begin();
        let reports_before = condition.out_of_memory_reports();
        condition
            .record_the_device_lost("Unknown：驱动重置".to_string(), CauseOfTheDeviceLoss::Other);
        let device_when_the_work_returned = condition.device_when_the_work_returns();
        assert_eq!(
            device_when_the_work_returned,
            DeviceWhenTheWorkReturned::Lost
        );
        assert_eq!(
            condition.out_of_memory_explaining_a_failure_of_the_work(
                work.identifier(),
                reports_before,
                device_when_the_work_returned
            ),
            None,
            "设备为别的原因丢了而这段活开始之后没报过：不是显存不够"
        );
        std::thread::scope(|scope| {
            scope.spawn(|| {
                let _another_work = WorkOnTheCardOfThisThread::begin();
                condition.record_an_out_of_memory("别的活上撞满".to_string());
            });
        });
        assert!(
            condition
                .out_of_memory_explaining_a_failure_of_the_work(
                    work.identifier(),
                    reports_before,
                    device_when_the_work_returned
                )
                .is_some_and(|reason| reason.starts_with("设备丢了")),
            "设备丢了、这段活开始之后报过：说成设备因它丢了"
        );
    }

    /// 第 10z 批（第 10y 批代码轮判决 K3，正推 X16）：报告里这个进程那一行占的量读不出（`N/A`），记成没列（少借），不当成占了极大。
    #[test]
    fn an_unreadable_memory_of_this_process_in_the_report_is_not_listed_not_a_huge_occupancy() {
        let report = nvidia_smi_report(&[an_rtx_5090_in_the_report(
            20_000,
            &[(ANOTHER_PROCESS, 9_000), (THIS_PROCESS, 3_141)],
        )])
        .replace(": 3141 MiB", ": N/A");
        assert_eq!(
            card_memory_readings_of_one_nvidia_smi_report(
                &report,
                THIS_PROCESS,
                &[pci_bus_identifier_without_the_domain(BUS_OF_THE_RTX_5090)],
                &no_device_outside_the_books
            )
            .map(|readings| readings[0].occupied_by_this_process),
            Ok(OccupiedByThisProcess::NotListed)
        );
    }

    /// 第 10z 批（第 10y 批代码轮判决 K3，正推 X17）：报告里一行进程号认不出，报错（判红），不悄悄当成 0 号进程。
    #[test]
    fn an_unreadable_process_identifier_in_the_report_is_an_error_not_process_zero() {
        let report = nvidia_smi_report(&[an_rtx_5090_in_the_report(
            20_000,
            &[(ANOTHER_PROCESS, 9_000), (THIS_PROCESS, 3_141)],
        )])
        .replace(": 7777\n", ": abc\n");
        let readings = card_memory_readings_of_one_nvidia_smi_report(
            &report,
            THIS_PROCESS,
            &[pci_bus_identifier_without_the_domain(BUS_OF_THE_RTX_5090)],
            &no_device_outside_the_books,
        );
        assert!(
            readings
                .as_ref()
                .is_err_and(|reason| reason.contains("进程号认不出")),
            "{readings:?}"
        );
    }

    /// 第 10z 批（第 10y 批代码轮判决 K3，正推 X18）：报告里点名的卡的 Free 读不出，报错（判红），不悄悄当成空 0。
    #[test]
    fn an_unreadable_free_memory_in_the_report_is_an_error_not_zero() {
        let report = nvidia_smi_report(&[an_rtx_5090_in_the_report(20_000, &[])])
            .replace(": 20000 MiB", ": N/A");
        let readings = card_memory_readings_of_one_nvidia_smi_report(
            &report,
            THIS_PROCESS,
            &[pci_bus_identifier_without_the_domain(BUS_OF_THE_RTX_5090)],
            &no_device_outside_the_books,
        );
        assert!(
            readings
                .as_ref()
                .is_err_and(|reason| reason.contains("Free 读不出")),
            "{readings:?}"
        );
    }

    /// 第 10z 批（第 10y 批代码轮判决 K3，正推 X19）：报告里点名的卡没有 Reserved 那一行（老驱动的样子），报错（判红），不悄悄当成保留 0
    /// （那样空卡上能用的就成了总量）。
    #[test]
    fn a_missing_reserved_memory_in_the_report_is_an_error_not_zero() {
        let report = nvidia_smi_report(&[an_rtx_5090_in_the_report(20_000, &[])])
            .lines()
            .filter(|line| !line.trim_start().starts_with("Reserved"))
            .collect::<Vec<&str>>()
            .join("\n");
        let readings = card_memory_readings_of_one_nvidia_smi_report(
            &report,
            THIS_PROCESS,
            &[pci_bus_identifier_without_the_domain(BUS_OF_THE_RTX_5090)],
            &no_device_outside_the_books,
        );
        assert!(
            readings
                .as_ref()
                .is_err_and(|reason| reason.contains("Reserved 读不出")),
            "{readings:?}"
        );
    }

    /// 第 10z 批（第 10y 批代码轮判决 K3，正推 X4）：一个状态要的恰好等于空着时能借的，是此刻借不到（报本次未跑），不是放不下；
    /// 多 1 MiB 才是空着时也放不下。
    #[test]
    fn a_state_needing_exactly_what_an_empty_card_lends_is_no_room_now() {
        let room = CardRoomForThisProcess {
            room_now_mebibytes: 0,
            ceiling_mebibytes: 1_000,
        };
        assert_eq!(
            attempt_on_a_card_without_room_for_one_state(1_000, room),
            ScratchAttemptOnACard::NoRoomNow {
                need_mebibytes: 1_000
            }
        );
        assert_eq!(
            attempt_on_a_card_without_room_for_one_state(1_001, room),
            ScratchAttemptOnACard::CannotHoldOneState {
                ceiling_mebibytes: 1_000
            }
        );
    }

    /// 第 10z 批第六块（G2 乙，等失败那一支；数照第 10z 批代码轮判决 K2-1 第 2 条攻方模型「丁」：5090 空卡 32 110，驱动实占 600）：长借据 10 000、
    /// C 8 000、V 3 000 都建满、读过一次；C 丢掉之前等设备没等成（超时或报错），卡上还没还，V 这一趟派完、缓冲缩回驱动（V 的借据还活着）。
    /// C 照旧记在账上：能借的是配给卡的额度 32 000 减驱动那一份、减三笔借据的 8 440，不超过卡上此刻真能再给的 8 550（空 13 510 减驱动还要长的
    /// 1 960、减 V 还要长回的 3 000）；C 照常销了账是 13 510，照账借满卡上就超了。这张卡的设备丢了、账照重起记，C 才销账。
    #[test]
    fn a_lease_whose_wait_failed_stays_booked_until_the_device_is_taken_as_lost_and_gives_no_more()
    {
        const EMPTY: u64 = 32_110;
        const DRIVER_HOLDS_NOW: u64 = 600;
        const LONG_LEASE: u64 = 10_000;
        const RETURNED_LEASE: u64 = 8_000;
        const SHRINKING_LEASE: u64 = 3_000;
        const CONFIGURED_CARD_QUOTA: u64 = 32_000;
        let readings = replaceable_readings(vec![CardMemoryReading {
            free_mebibytes: EMPTY,
            total_less_reserved_mebibytes: EMPTY,
            occupied_by_this_process: OccupiedByThisProcess::NotListed,
        }]);
        let shared = Arc::clone(&readings);
        let ledger = GpuMemoryLedger::new(
            1,
            Box::new(move || Ok(shared.lock().expect("用例里只有一条线程改读数").clone())),
            Box::new(|_position, may_hold_mebibytes| {
                card_ledger_capacity_mebibytes(
                    None,
                    Some(CONFIGURED_CARD_QUOTA),
                    may_hold_mebibytes.min(CONFIGURED_CARD_QUOTA),
                    may_hold_mebibytes,
                    DRIVER,
                )
            }),
            DRIVER,
        );
        let occupied = |mebibytes: u64| CardMemoryReading {
            free_mebibytes: EMPTY - mebibytes,
            total_less_reserved_mebibytes: EMPTY,
            occupied_by_this_process: OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes },
        };
        let long_lease = ledger.book_on_the_card(0, LONG_LEASE).expect("空卡借得到");
        let device = ledger.record_the_device_started_on_the_card(0);
        let mut returned_lease = ledger
            .book_on_the_card(0, RETURNED_LEASE)
            .expect("卡上空着 19 440");
        let returned_lease_identifier = returned_lease.lease_identifier();
        let shrinking_lease = ledger
            .book_on_the_card(0, SHRINKING_LEASE)
            .expect("卡上空着 11 440");
        replace_the_reading(
            &readings,
            0,
            occupied(DRIVER_HOLDS_NOW + LONG_LEASE + RETURNED_LEASE + SHRINKING_LEASE),
        );
        assert!(
            ledger.card_rooms_mebibytes_now(&[0]).is_ok(),
            "三笔都建满时读一次"
        );
        returned_lease.take_in_the_wait_before_returning(
            &device,
            DeviceWaitBeforeReturningTheLease::TheWaitFailed,
        );
        drop(returned_lease);
        replace_the_reading(
            &readings,
            0,
            occupied(DRIVER_HOLDS_NOW + LONG_LEASE + RETURNED_LEASE),
        );
        let free_now = EMPTY - DRIVER_HOLDS_NOW - LONG_LEASE - RETURNED_LEASE;
        let room_now = ledger.card_rooms_mebibytes_now(&[0]).expect("假读数读得出")[0];
        assert_eq!(
            room_now + DRIVER + LONG_LEASE + RETURNED_LEASE + SHRINKING_LEASE,
            CONFIGURED_CARD_QUOTA,
            "C 没等成、照旧记着：能借的 {room_now} 加驱动那一份与三笔借据，是配给卡的额度"
        );
        assert!(
            room_now + (DRIVER - DRIVER_HOLDS_NOW) + SHRINKING_LEASE <= free_now,
            "C 卡上还没还，V 缩回驱动：能借的 {room_now} 加驱动还要长的、V 还要长回的，不超过卡上空着的 13 510（不多给）"
        );
        assert!(
            ledger.is_the_lease_on_the_books(0, returned_lease_identifier),
            "没等成的那一笔丢掉之后照旧记在账上"
        );
        let _restart_judged = ledger.prepare_the_driver_share_before_starting_the_device(0);
        assert!(
            !ledger.is_the_lease_on_the_books(0, returned_lease_identifier),
            "设备丢了、账照重起记：没等成的那一笔这时销账"
        );
        drop(shrinking_lease);
        drop(long_lease);
    }

    /// 第 10z 批第三块（第 10z 批代码轮判决 K1-3 正推 S6）：丢了的设备重起被拒过一次（卡上空 17 440，第二笔 17 440 还没建），被拒之前已借到的
    /// 并发用例再来起设备，照此刻再判、照样拒，不直接放行；旧设备与它上面的缓冲都还给驱动之后（卡上空 30 000、这个进程不在报告里）再来，放得下，起。
    #[test]
    fn a_lost_device_refused_a_restart_is_judged_again_on_every_later_attempt() {
        let (ledger, readings) =
            ledger_whose_started_device_holds_a_built_lease_and_another_unbuilt(
                30_000 - DRIVER - 10_000,
            );
        let refused = Err(BookingRefusal::NoCardHasRoomNow {
            need_mebibytes: DRIVER,
            card_rooms_mebibytes: vec![0],
        });
        assert_eq!(
            ledger.prepare_the_driver_share_before_starting_the_device(0),
            refused,
            "头一次重起：放不下，不起"
        );
        assert_eq!(
            ledger.prepare_the_driver_share_before_starting_the_device(0),
            refused,
            "被拒之前已借到的用例再来：照此刻再判，照样放不下"
        );
        assert_eq!(
            ledger.prepare_the_driver_share_before_starting_the_device(0),
            refused,
            "再来一次照旧"
        );
        replace_the_reading(&readings, 0, reading_of_an_rtx_5090(30_000));
        assert_eq!(
            ledger.prepare_the_driver_share_before_starting_the_device(0),
            Ok(()),
            "旧设备还给驱动之后：借着的 27 440 连新设备的驱动那一份放得下，起"
        );
        let _replacement = ledger.record_the_device_started_on_the_card(0);
        assert!(ledger.is_this_process_registered_on_the_card(0));
    }

    /// 第 10z 批第三块（第 10z 批代码轮正推 N10、攻方第三节第 1310 行；第 10z 批第六块 G2 乙改数）：重起时借着的连新设备的驱动那一份照此刻的空闲
    /// 恰好放得下（空 17 440 = 驱动那一份 2 560 + 借着的 10 000 + 4 880，建上了的那 10 000 照没建算），起，不拒；多 1 MiB 就拒。
    #[test]
    fn a_lost_device_started_again_fits_when_its_driver_share_fits_exactly() {
        let (ledger, _readings) =
            ledger_whose_started_device_holds_a_built_lease_and_another_unbuilt(4_880);
        assert_eq!(DRIVER + 10_000 + 4_880, 30_000 - DRIVER - 10_000);
        assert_eq!(
            ledger.prepare_the_driver_share_before_starting_the_device(0),
            Ok(()),
            "恰好放得下：起"
        );
        let (ledger_one_mebibyte_short, _readings_one_mebibyte_short) =
            ledger_whose_started_device_holds_a_built_lease_and_another_unbuilt(4_881);
        assert!(
            ledger_one_mebibyte_short
                .prepare_the_driver_share_before_starting_the_device(0)
                .is_err(),
            "多 1 MiB：不起"
        );
    }

    /// 第 10z 批第六块（G2 乙：「销了账、卡上可能还没还的」那一份拿掉之后，丢了的设备留在卡上的不再记成一份；第 10z 批第八块改按放没放掉判）：
    /// 空 30 000 的卡上借 A 10 000、设备起好、建满（驱动实占 600）；设备丢了，B 5 000 借到、照重起记，新设备起好（也实占 600），卡池那一格换成新设备。
    /// A 的卡还拿着旧设备：旧设备的 600 与 A 的 10 000 留在卡上，读数分不出哪一份是新设备的，能借的照分不出算（空 18 800 减驱动那一份、减 A、B，
    /// 1 240），不超过卡上此刻真能再给的 11 840（空 18 800 减新设备驱动还要长的 1 960、减 B 还要建的 5 000）；把旧设备的 600 当新设备的驱动那一份算是
    /// 12 440，多给 600。A 等完设备、它的卡丢掉，旧设备的句柄都没了（wgpu 收掉、卡上还给驱动），这一台放掉：照读数分得出算，能借的加 B 与驱动那一份
    /// 是空卡（B 是设备丢了那一刻就借着的，照「那一刻活着的借据都还了」判时要等 B 也还了，少给 600）。之后借 C 3 000、B 也还了，照旧。
    #[test]
    fn a_lost_device_left_on_the_card_is_not_taken_for_the_driver_share_of_its_replacement() {
        const EMPTY: u64 = 30_000;
        const DRIVER_HOLDS_NOW: u64 = 600;
        const LEASE_ON_THE_LOST_DEVICE: u64 = 10_000;
        const LEASE_STARTING_THE_REPLACEMENT: u64 = 5_000;
        const LEASE_AFTER_THE_RESTART: u64 = 3_000;
        let readings = replaceable_readings(vec![reading_of_an_rtx_5090(EMPTY)]);
        let ledger = ledger_reading(&readings);
        let occupied = |mebibytes: u64| CardMemoryReading {
            free_mebibytes: EMPTY - mebibytes,
            total_less_reserved_mebibytes: AN_EMPTY_RTX_5090_HAS,
            occupied_by_this_process: OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes },
        };
        let mut lease_on_the_lost_device = ledger
            .book_on_the_card(0, LEASE_ON_THE_LOST_DEVICE)
            .expect("空着的卡借得到");
        assert_eq!(
            ledger.prepare_the_driver_share_before_starting_the_device(0),
            Ok(())
        );
        let lost_device_held_by_the_pool =
            Arc::new(ledger.record_the_device_started_on_the_card(0));
        let lost_device_held_by_the_lease = Arc::clone(&lost_device_held_by_the_pool);
        replace_the_reading(
            &readings,
            0,
            occupied(DRIVER_HOLDS_NOW + LEASE_ON_THE_LOST_DEVICE),
        );
        let mut lease_starting_the_replacement = ledger
            .book_on_the_card(0, LEASE_STARTING_THE_REPLACEMENT)
            .expect("卡上空着 19 400");
        assert_eq!(
            ledger.prepare_the_driver_share_before_starting_the_device(0),
            Ok(()),
            "设备丢了要重起：空 19 400 放得下驱动那一份与两笔"
        );
        let replacement = ledger.record_the_device_started_on_the_card(0);
        drop(lost_device_held_by_the_pool);
        let occupied_with_the_lost_device_left =
            DRIVER_HOLDS_NOW + LEASE_ON_THE_LOST_DEVICE + DRIVER_HOLDS_NOW;
        replace_the_reading(&readings, 0, occupied(occupied_with_the_lost_device_left));
        let free_now = EMPTY - occupied_with_the_lost_device_left;
        let room_while_the_lost_device_remains =
            ledger.card_rooms_mebibytes_now(&[0]).expect("假读数读得出")[0];
        assert_eq!(
            room_while_the_lost_device_remains
                + DRIVER
                + LEASE_ON_THE_LOST_DEVICE
                + LEASE_STARTING_THE_REPLACEMENT,
            free_now,
            "旧设备还在卡上：能借的 {room_while_the_lost_device_remains} 加驱动那一份与两笔，是空着的 18 800"
        );
        assert!(
            room_while_the_lost_device_remains
                + (DRIVER - DRIVER_HOLDS_NOW)
                + LEASE_STARTING_THE_REPLACEMENT
                <= free_now,
            "不超过卡上此刻真能再给的（不多给）"
        );
        lease_on_the_lost_device.take_in_the_wait_before_returning(
            &lost_device_held_by_the_lease,
            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
        );
        replace_the_reading(&readings, 0, occupied(DRIVER_HOLDS_NOW));
        drop(lost_device_held_by_the_lease);
        drop(lease_on_the_lost_device);
        let room_after_the_lost_device_is_released =
            ledger.card_rooms_mebibytes_now(&[0]).expect("假读数读得出")[0];
        assert_eq!(
            room_after_the_lost_device_is_released + DRIVER + LEASE_STARTING_THE_REPLACEMENT,
            EMPTY,
            "丢了的那一台放掉了：照读数分得出算，能借的 {room_after_the_lost_device_is_released} 加驱动那一份与 B 是空卡"
        );
        let mut lease_after_the_restart = ledger
            .book_on_the_card(0, LEASE_AFTER_THE_RESTART)
            .expect("卡上空着 29 400");
        lease_starting_the_replacement.take_in_the_wait_before_returning(
            &replacement,
            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
        );
        drop(lease_starting_the_replacement);
        let room_after_the_lease_starting_the_replacement_is_returned =
            ledger.card_rooms_mebibytes_now(&[0]).expect("假读数读得出")[0];
        assert_eq!(
            room_after_the_lease_starting_the_replacement_is_returned
                + LEASE_AFTER_THE_RESTART
                + DRIVER,
            EMPTY,
            "B 也还了：能借的 {room_after_the_lease_starting_the_replacement_is_returned} 加重起之后借的与驱动那一份是空卡"
        );
        lease_after_the_restart.take_in_the_wait_before_returning(
            &replacement,
            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
        );
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

    /// 借一笔建满 `mebibytes`、等设备超时（设备上还排着别的活）、丢掉：交回它的借据号；缓冲还在卡上，由调用方后来还给驱动。
    fn a_lease_built_and_dropped_after_a_timed_out_wait(
        card: &Mutex<CardUnderTheLedger>,
        ledger: &GpuMemoryLedger,
        device: &DeviceStartedOnTheBooks<'_>,
        mebibytes: u64,
    ) -> u64 {
        let mut lease = ledger.book_on_the_card(0, mebibytes).expect("借得到");
        let lease_identifier = lease.lease_identifier();
        card.lock().expect("假卡不 panic").built_mebibytes += mebibytes;
        lease.take_in_the_wait_before_returning(
            device,
            DeviceWaitBeforeReturningTheLease::TheWaitFailed,
        );
        drop(lease);
        lease_identifier
    }

    /// 第 10z 批第八块（第 10z 批第六块代码轮判决 K1-2：两腿都量到等超时的那几笔永久少借，正推每笔 4 000、第 7 笔起借不到）：设备起好（驱动实占 600），
    /// 一笔笔借 4 000、建满、等设备超时、丢掉；活后来做完、缓冲还给驱动，卡池借下一笔之前对这张卡 poll 一次交回队列空了：没等成的那一笔销账，
    /// 能借的回到卡上真能再给的（空卡减驱动那一份）。八笔笔笔借得到，中间没有别的等成。
    #[test]
    fn a_lease_whose_wait_timed_out_leaves_the_books_when_a_poll_finds_its_device_idle() {
        let (card, ledger) = ledger_on_a_card_under_the_ledger();
        let mut first = ledger.book_on_the_card(0, 1).expect("空卡借得到");
        let device = ledger.record_the_device_started_on_the_card(0);
        {
            let mut card = card.lock().expect("假卡不 panic");
            card.device_started = true;
            card.driver_occupied_mebibytes = 600;
        }
        first.take_in_the_wait_before_returning(
            &device,
            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
        );
        drop(first);
        for timed_out_lease_number in 1..=8 {
            let timed_out_lease =
                a_lease_built_and_dropped_after_a_timed_out_wait(&card, &ledger, &device, 4_000);
            assert!(
                ledger.is_the_lease_on_the_books(0, timed_out_lease),
                "第 {timed_out_lease_number} 笔没等成：缓冲可能还在卡上，照旧记着"
            );
            assert!(
                room_now(&ledger) <= room_truly_left(&card, 0),
                "第 {timed_out_lease_number} 笔的缓冲还在卡上时不多给"
            );
            let listed = ledger.leases_kept_after_a_failed_wait_on_each_card();
            assert_eq!(
                listed
                    .iter()
                    .map(|listed_on_a_card| (
                        listed_on_a_card.card_position(),
                        listed_on_a_card.lease_identifiers()
                    ))
                    .collect::<Vec<(usize, Vec<u64>)>>(),
                vec![(0, vec![timed_out_lease])],
                "第 {timed_out_lease_number} 笔：这张卡上有没等成而照旧记着的，卡池借之前要 poll 它"
            );
            card.lock().expect("假卡不 panic").built_mebibytes = 0;
            device.take_in_that_the_work_of_the_listed_leases_finished(&listed[0]);
            assert!(
                !ledger.is_the_lease_on_the_books(0, timed_out_lease),
                "第 {timed_out_lease_number} 笔：poll 交回队列空了，那一笔销账"
            );
            assert_eq!(
                room_now(&ledger),
                room_truly_left(&card, 0),
                "第 {timed_out_lease_number} 笔销了账：能借的是卡上真能再给的"
            );
        }
        assert!(ledger
            .leases_kept_after_a_failed_wait_on_each_card()
            .is_empty());
    }

    /// 第 10z 批第十一块（第 10z 批第七八九块代码轮判决 K1-1，攻方 `opus_k11_poll_race.rs` 的 take-in-between：卡池列卡、poll 交回队列空了，
    /// 销账之前另一条线程借了一笔、建满、提交了活、等设备没等成；销账时它被一起销了，它的缓冲还在卡上，账多给）：X 10 000 建满、没等成、照旧记着；
    /// 卡池列卡（列出 X）、poll 交回队列空了（X 的缓冲还给驱动）；A 借 8 000、建满、没等成；卡池照列出来的销：X 销账，A 照旧记着，不多给。
    /// 下一次列卡列出 A，A 的缓冲还给驱动之后再 poll 交回队列空了，A 才销账。
    #[test]
    fn a_lease_kept_after_the_listing_stays_on_the_books_when_the_poll_after_the_listing_finds_the_device_idle(
    ) {
        let (card, ledger) = ledger_on_a_card_under_the_ledger();
        let mut first = ledger.book_on_the_card(0, 1).expect("空卡借得到");
        let device = ledger.record_the_device_started_on_the_card(0);
        {
            let mut card = card.lock().expect("假卡不 panic");
            card.device_started = true;
            card.driver_occupied_mebibytes = 600;
        }
        first.take_in_the_wait_before_returning(
            &device,
            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
        );
        drop(first);
        let lease_x =
            a_lease_built_and_dropped_after_a_timed_out_wait(&card, &ledger, &device, 10_000);
        let listed = ledger.leases_kept_after_a_failed_wait_on_each_card();
        assert_eq!(
            listed
                .iter()
                .map(LeasesKeptAfterAFailedWaitWhenListed::lease_identifiers)
                .collect::<Vec<Vec<u64>>>(),
            vec![vec![lease_x]],
            "列卡那一刻照旧记着的只有 X"
        );
        // poll 交回队列空了：X 的活做完、缓冲还给驱动
        card.lock().expect("假卡不 panic").built_mebibytes = 0;
        let lease_a =
            a_lease_built_and_dropped_after_a_timed_out_wait(&card, &ledger, &device, 8_000);
        device.take_in_that_the_work_of_the_listed_leases_finished(&listed[0]);
        assert!(
            !ledger.is_the_lease_on_the_books(0, lease_x),
            "X 列出来了、poll 那一刻之前提交的活都做完了：销账"
        );
        assert!(
            ledger.is_the_lease_on_the_books(0, lease_a),
            "A 是列卡之后才记成没等成的：它的活可以是 poll 之后才提交的，照旧记着"
        );
        assert!(
            room_now(&ledger) <= room_truly_left(&card, 0),
            "A 的缓冲还在卡上：不多给（{} 对 {}）",
            room_now(&ledger),
            room_truly_left(&card, 0)
        );
        let listed_again = ledger.leases_kept_after_a_failed_wait_on_each_card();
        card.lock().expect("假卡不 panic").built_mebibytes = 0;
        device.take_in_that_the_work_of_the_listed_leases_finished(&listed_again[0]);
        assert!(
            !ledger.is_the_lease_on_the_books(0, lease_a),
            "下一次列卡列出 A、poll 交回队列空了：A 销账"
        );
        assert_eq!(room_now(&ledger), room_truly_left(&card, 0));
    }

    /// 第 10z 批第十一块（第 10z 批第七八九块代码轮攻方 `scripts/mut.py` 的 M1、M2 照绿：放掉的是在用的那一台没有用例）：在用的那一台上 X 没等成、
    /// 照旧记着，这台设备的克隆都丢了（卡池整个丢掉那一类；今天卡池活到进程退出，产品路径走不到，账的接口上走得到）：X 销账、这张卡上不再有在用的那一台，
    /// 下一台起得来，能借的是卡上真能再给的。
    #[test]
    fn releasing_the_device_in_use_returns_the_leases_kept_on_it_and_a_new_device_starts_on_the_card(
    ) {
        let (card, ledger) = ledger_on_a_card_under_the_ledger();
        let mut first = ledger.book_on_the_card(0, 1).expect("空卡借得到");
        let device = ledger.record_the_device_started_on_the_card(0);
        {
            let mut card = card.lock().expect("假卡不 panic");
            card.device_started = true;
            card.driver_occupied_mebibytes = 600;
        }
        first.take_in_the_wait_before_returning(
            &device,
            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
        );
        drop(first);
        let kept_lease =
            a_lease_built_and_dropped_after_a_timed_out_wait(&card, &ledger, &device, 4_000);
        {
            // 放掉：wgpu 收掉设备，它上面的缓冲与驱动那一份交回驱动
            let mut card = card.lock().expect("假卡不 panic");
            card.device_started = false;
            card.built_mebibytes = 0;
        }
        drop(device);
        assert!(
            !ledger.is_the_lease_on_the_books(0, kept_lease),
            "在用的那一台放掉了：它上面没等成的那一笔销账"
        );
        let next_lease = ledger.book_on_the_card(0, 1_000).expect("借得到 1 000");
        let next_device = ledger.record_the_device_started_on_the_card(0);
        {
            let mut card = card.lock().expect("假卡不 panic");
            card.device_started = true;
            card.driver_occupied_mebibytes = 600;
        }
        assert_eq!(
            room_now(&ledger),
            room_truly_left(&card, 1_000),
            "下一台起好了：能借的是卡上真能再给的"
        );
        drop(next_lease);
        drop(next_device);
    }

    /// 正推账层探针那一形的起头：设备起好（驱动实占 600）、头一笔等完设备还了；再借此刻能借的减 1 000、建满、等设备超时、丢掉（照旧记着）。
    /// 交回账上的那一台、照旧记着的那一笔的借据号与它借的量（MiB）。
    fn a_started_device_with_all_but_one_thousand_kept_after_a_timed_out_wait<'ledger>(
        card: &Mutex<CardUnderTheLedger>,
        ledger: &'ledger GpuMemoryLedger,
    ) -> (DeviceStartedOnTheBooks<'ledger>, u64, u64) {
        let mut first = ledger.book_on_the_card(0, 1).expect("空卡借得到");
        let device = ledger.record_the_device_started_on_the_card(0);
        {
            let mut card = card.lock().expect("假卡不 panic");
            card.device_started = true;
            card.driver_occupied_mebibytes = 600;
        }
        first.take_in_the_wait_before_returning(
            &device,
            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
        );
        drop(first);
        let kept_mebibytes = room_now(ledger) - 1_000;
        let kept_lease =
            a_lease_built_and_dropped_after_a_timed_out_wait(card, ledger, &device, kept_mebibytes);
        (device, kept_lease, kept_mebibytes)
    }

    /// 第 10z 批第十三块（第 10z 批第十、十一块代码轮判决 K4-2，正推账层探针 `probe_kept_leases_on_a_lost_device.rs`：丢了的设备上没等成的那一笔
    /// 记在账上，账到重起才知道设备丢了，卡池只在借到之后才走到重起；能借的 1 000、借 4 000 被拒，这张卡在这个进程里再也借不出）：设备丢了，
    /// 它上面的缓冲与驱动那一份交回了驱动（卡上空着）；卡池借之前列卡、看到留着的设备丢了：不 poll 它、告诉账，那一笔销账，借 4 000 借得到，
    /// 能借的不超过卡上真能再给的；之后的重起照常（驱动那一份照此刻判、新的一台记得上）。
    #[test]
    fn a_lease_kept_on_a_device_found_lost_before_booking_leaves_the_books_and_four_thousand_fit_again(
    ) {
        let (card, ledger) = ledger_on_a_card_under_the_ledger();
        let (device, kept_lease, _kept_mebibytes) =
            a_started_device_with_all_but_one_thousand_kept_after_a_timed_out_wait(&card, &ledger);
        {
            let mut card = card.lock().expect("假卡不 panic");
            card.device_started = false;
            card.driver_occupied_mebibytes = 0;
            card.built_mebibytes = 0;
        }
        assert_eq!(
            room_now(&ledger),
            1_000,
            "账还不知道设备丢了：照旧记着的那一笔占着，能借的 1 000（正推探针 room_now_after_lost）"
        );
        let polls_of_the_lost_device = AtomicUsize::new(0);
        ledger.take_in_the_devices_kept_before_booking(
            |card_position| (card_position == 0).then_some(&device),
            |_kept| true,
            |_kept| {
                polls_of_the_lost_device.fetch_add(1, Ordering::SeqCst);
                true
            },
            |kept| *kept,
        );
        assert_eq!(
            polls_of_the_lost_device.load(Ordering::SeqCst),
            0,
            "丢了的设备不 poll（wgpu 在丢了的设备上 poll 会 panic）"
        );
        assert!(
            !ledger.is_the_lease_on_the_books(0, kept_lease),
            "借之前账就知道设备丢了：它上面没等成的那一笔照重起记、销账"
        );
        assert!(
            room_now(&ledger) <= room_truly_left(&card, 0),
            "不多给（{} 对 {}）",
            room_now(&ledger),
            room_truly_left(&card, 0)
        );
        let booking = ledger
            .book_on_the_card(0, 4_000)
            .expect("卡上空着：借 4 000 借得到（改之前 NoCardHasRoomNow，能借的 1 000）");
        ledger
            .prepare_the_driver_share_before_starting_the_device(0)
            .expect("重起：驱动那一份照此刻放得下");
        let replacement = ledger.record_the_device_started_on_the_card(0);
        drop(booking);
        drop(replacement);
        drop(device);
    }

    /// 第 10z 批第十三块（K4-2 那一改不多给）：同上，设备丢了而它上面那一笔的缓冲与驱动那一份还占着卡（旧设备还没收拾）；卡池借之前告诉账之后，
    /// 那一笔出了账、这张卡照分不出算：能借的不超过卡上真能再给的，借 4 000 照旧借不到。
    #[test]
    fn a_device_found_lost_with_its_buffers_still_on_the_card_lends_no_more_than_the_card_has_left()
    {
        let (card, ledger) = ledger_on_a_card_under_the_ledger();
        let (device, kept_lease, kept_mebibytes) =
            a_started_device_with_all_but_one_thousand_kept_after_a_timed_out_wait(&card, &ledger);
        {
            let mut card = card.lock().expect("假卡不 panic");
            card.device_started = false;
            card.driver_occupied_mebibytes = 0;
            card.built_mebibytes = 0;
            card.left_by_a_lost_device_mebibytes = 600 + kept_mebibytes;
        }
        ledger.take_in_the_devices_kept_before_booking(
            |card_position| (card_position == 0).then_some(&device),
            |_kept| true,
            |_kept| panic!("丢了的设备不 poll"),
            |kept| *kept,
        );
        assert!(
            !ledger.is_the_lease_on_the_books(0, kept_lease),
            "告诉账之后照重起记：那一笔出了账（缓冲在丢了的那一台上，放掉之前这张卡照分不出算）"
        );
        let room = room_now(&ledger);
        let room_truly_left_on_the_card = room_truly_left(&card, 0);
        assert!(
            room <= room_truly_left_on_the_card,
            "旧设备连那一笔的缓冲还占着卡：能借的 {room} 不超过卡上真能再给的 {room_truly_left_on_the_card}（不多给）"
        );
        assert!(
            ledger.book_on_the_card(0, 4_000).is_err(),
            "卡上真没有 4 000：照旧借不到"
        );
        drop(device);
    }

    /// 第 10z 批第十三块（K4-2 那一改，卡池看到的是重起之前留着的那一份）：丢了的那一台重起过了（重起那一步已经把它记成丢了），
    /// 新的一台上有一笔没等成而照旧记着；卡池借之前取到的还是丢了的那一台、告诉账它丢了：账不动，新的一台没丢，它上面那一笔照旧记着。
    #[test]
    fn a_device_found_lost_after_its_restart_leaves_the_replacement_and_its_kept_lease_alone() {
        let (card, ledger) = ledger_on_a_card_under_the_ledger();
        let mut first = ledger.book_on_the_card(0, 1).expect("空卡借得到");
        let lost_device = ledger.record_the_device_started_on_the_card(0);
        {
            let mut card = card.lock().expect("假卡不 panic");
            card.device_started = true;
            card.driver_occupied_mebibytes = 600;
        }
        first.take_in_the_wait_before_returning(
            &lost_device,
            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
        );
        drop(first);
        ledger
            .prepare_the_driver_share_before_starting_the_device(0)
            .expect("重起：驱动那一份放得下");
        let replacement = ledger.record_the_device_started_on_the_card(0);
        let kept_on_the_replacement =
            a_lease_built_and_dropped_after_a_timed_out_wait(&card, &ledger, &replacement, 4_000);
        let room_before = room_now(&ledger);
        ledger.take_in_the_devices_kept_before_booking(
            |card_position| (card_position == 0).then_some(&lost_device),
            |_kept| true,
            |_kept| panic!("丢了的设备不 poll"),
            |kept| *kept,
        );
        assert!(
            ledger.is_the_lease_on_the_books(0, kept_on_the_replacement),
            "新的一台没丢：它上面没等成的那一笔照旧记着"
        );
        assert_eq!(room_now(&ledger), room_before, "账不动");
        assert_eq!(
            ledger
                .leases_kept_after_a_failed_wait_on_each_card()
                .iter()
                .map(LeasesKeptAfterAFailedWaitWhenListed::lease_identifiers)
                .collect::<Vec<Vec<u64>>>(),
            vec![vec![kept_on_the_replacement]],
            "下一次列卡照旧列出新的一台上那一笔"
        );
        drop(replacement);
        drop(lost_device);
    }

    /// 第 10z 批第十三块（第 10z 批第十、十一块代码轮判决 K5，正推 Y1：卡池借之前 poll 的结局改成恒真照绿）：留着的设备没丢，poll 交回还有在飞的提交：
    /// poll 了一次，那一笔照旧记着（它的缓冲可能还在卡上），账不动。
    #[test]
    fn a_device_still_busy_at_the_poll_before_booking_keeps_its_kept_lease_on_the_books() {
        let (card, ledger) = ledger_on_a_card_under_the_ledger();
        let (device, kept_lease, _kept_mebibytes) =
            a_started_device_with_all_but_one_thousand_kept_after_a_timed_out_wait(&card, &ledger);
        let room_before = room_now(&ledger);
        let polls = AtomicUsize::new(0);
        ledger.take_in_the_devices_kept_before_booking(
            |card_position| (card_position == 0).then_some(&device),
            |_kept| false,
            |_kept| {
                polls.fetch_add(1, Ordering::SeqCst);
                false
            },
            |kept| *kept,
        );
        assert_eq!(polls.load(Ordering::SeqCst), 1, "没丢：poll 一次");
        assert!(
            ledger.is_the_lease_on_the_books(0, kept_lease),
            "还有在飞的提交：那一笔照旧记着"
        );
        assert_eq!(room_now(&ledger), room_before, "账不动");
        assert!(room_now(&ledger) <= room_truly_left(&card, 0), "不多给");
        drop(device);
    }

    /// 第 10z 批第十三块（K5，正推 Y2、Y3：poll 的结局改成恒假、列卡的循环只看第一张卡照绿）：两张卡上各有一笔没等成而照旧记着，
    /// 两台设备都没丢、poll 都交回队列空了：每张卡 poll 一次，两笔都销账。
    #[test]
    fn every_listed_card_whose_kept_device_is_idle_at_the_poll_returns_its_kept_lease() {
        let readings = replaceable_readings(vec![
            reading_of_an_rtx_5090(AN_EMPTY_RTX_5090_HAS),
            reading_of_an_rtx_5090(AN_EMPTY_RTX_5090_HAS),
        ]);
        let ledger = ledger_reading(&readings);
        let mut devices = Vec::new();
        let mut kept_leases = Vec::new();
        for card_position in 0..2 {
            let mut lease = ledger
                .book_on_the_card(card_position, 1_000)
                .expect("空卡借得到");
            let device = ledger.record_the_device_started_on_the_card(card_position);
            lease.take_in_the_wait_before_returning(
                &device,
                DeviceWaitBeforeReturningTheLease::TheWaitFailed,
            );
            kept_leases.push(lease.lease_identifier());
            drop(lease);
            devices.push(device);
        }
        let polls_on_each_card = [AtomicUsize::new(0), AtomicUsize::new(0)];
        ledger.take_in_the_devices_kept_before_booking(
            |card_position| devices.get(card_position),
            |_kept| false,
            |kept| {
                polls_on_each_card[kept.card_position].fetch_add(1, Ordering::SeqCst);
                true
            },
            |kept| *kept,
        );
        assert_eq!(
            polls_on_each_card
                .iter()
                .map(|polls| polls.load(Ordering::SeqCst))
                .collect::<Vec<usize>>(),
            vec![1, 1],
            "列出来的每张卡 poll 一次"
        );
        for (card_position, kept_lease) in kept_leases.iter().enumerate() {
            assert!(
                !ledger.is_the_lease_on_the_books(card_position, *kept_lease),
                "第 {card_position} 张卡的设备队列空了：那一笔销账"
            );
        }
    }

    /// 第 10z 批第十三块（K5，正推 G1：去掉销列出来的借据时「等的是 poll 的那一台」那一条照绿）：丢了的那一台重起过、还没放掉，新的一台上
    /// 有一笔没等成而照旧记着；拿丢了的那一台交「列出来的那几笔的活做完了」：列出来的那一笔等的是新的一台，不销。
    #[test]
    fn a_finished_poll_of_a_lost_device_left_alive_does_not_return_the_lease_kept_on_its_replacement(
    ) {
        let (card, ledger) = ledger_on_a_card_under_the_ledger();
        let mut first = ledger.book_on_the_card(0, 1).expect("空卡借得到");
        let lost_device = ledger.record_the_device_started_on_the_card(0);
        {
            let mut card = card.lock().expect("假卡不 panic");
            card.device_started = true;
            card.driver_occupied_mebibytes = 600;
        }
        first.take_in_the_wait_before_returning(
            &lost_device,
            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
        );
        drop(first);
        ledger
            .prepare_the_driver_share_before_starting_the_device(0)
            .expect("重起：驱动那一份放得下");
        let replacement = ledger.record_the_device_started_on_the_card(0);
        let kept_on_the_replacement =
            a_lease_built_and_dropped_after_a_timed_out_wait(&card, &ledger, &replacement, 4_000);
        let listed = ledger.leases_kept_after_a_failed_wait_on_each_card();
        lost_device.take_in_that_the_work_of_the_listed_leases_finished(&listed[0]);
        assert!(
            ledger.is_the_lease_on_the_books(0, kept_on_the_replacement),
            "poll 的是丢了的那一台：等新的一台的那一笔不销"
        );
        drop(replacement);
        drop(lost_device);
    }

    /// 第 10z 批第十三块（K5，正推 G4：去掉「列的是这台设备那张卡」那一条断言照绿）：列的是第 1 张卡，交给第 0 张卡上的设备销：断言拦下，
    /// 第 1 张卡上那一笔照旧记着。
    #[test]
    fn the_listed_leases_of_another_card_given_to_a_device_are_refused_by_an_assertion() {
        let readings = replaceable_readings(vec![
            reading_of_an_rtx_5090(AN_EMPTY_RTX_5090_HAS),
            reading_of_an_rtx_5090(AN_EMPTY_RTX_5090_HAS),
        ]);
        let ledger = ledger_reading(&readings);
        let booking_on_the_first_card = ledger.book_on_the_card(0, 1_000).expect("空卡借得到");
        let device_on_the_first_card = ledger.record_the_device_started_on_the_card(0);
        let mut lease_on_the_second_card = ledger.book_on_the_card(1, 1_000).expect("空卡借得到");
        let device_on_the_second_card = ledger.record_the_device_started_on_the_card(1);
        lease_on_the_second_card.take_in_the_wait_before_returning(
            &device_on_the_second_card,
            DeviceWaitBeforeReturningTheLease::TheWaitFailed,
        );
        let kept_on_the_second_card = lease_on_the_second_card.lease_identifier();
        drop(lease_on_the_second_card);
        let listed = ledger.leases_kept_after_a_failed_wait_on_each_card();
        let refused = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            device_on_the_first_card
                .take_in_that_the_work_of_the_listed_leases_finished(&listed[0]);
        }));
        assert!(
            refused.is_err(),
            "列的是第 1 张卡、交给第 0 张卡上的设备：断言拦下"
        );
        assert!(ledger.is_the_lease_on_the_books(1, kept_on_the_second_card));
        drop(booking_on_the_first_card);
        drop(device_on_the_first_card);
        drop(device_on_the_second_card);
    }

    /// 第 10z 批第十三块（K4-2 那一改的接线）：卡池那一格交的设备在账上记在别的卡上（列的是第 1 张卡，取到的是第 0 张卡上的设备）：
    /// 断言拦下。
    #[test]
    fn a_kept_device_on_the_books_of_another_card_is_refused_by_an_assertion() {
        let readings = replaceable_readings(vec![
            reading_of_an_rtx_5090(AN_EMPTY_RTX_5090_HAS),
            reading_of_an_rtx_5090(AN_EMPTY_RTX_5090_HAS),
        ]);
        let ledger = ledger_reading(&readings);
        let booking_on_the_first_card = ledger.book_on_the_card(0, 1_000).expect("空卡借得到");
        let device_on_the_first_card = ledger.record_the_device_started_on_the_card(0);
        let mut lease_on_the_second_card = ledger.book_on_the_card(1, 1_000).expect("空卡借得到");
        let device_on_the_second_card = ledger.record_the_device_started_on_the_card(1);
        lease_on_the_second_card.take_in_the_wait_before_returning(
            &device_on_the_second_card,
            DeviceWaitBeforeReturningTheLease::TheWaitFailed,
        );
        let kept_on_the_second_card = lease_on_the_second_card.lease_identifier();
        drop(lease_on_the_second_card);
        let refused = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            ledger.take_in_the_devices_kept_before_booking(
                |_card_position| Some(&device_on_the_first_card),
                |_kept| true,
                |_kept| panic!("丢了的设备不 poll"),
                |kept| *kept,
            );
        }));
        assert!(refused.is_err(), "取到的设备不在列出来的那张卡上：断言拦下");
        assert!(
            ledger.is_the_lease_on_the_books(1, kept_on_the_second_card),
            "第 1 张卡上那一笔照旧记着"
        );
        drop(booking_on_the_first_card);
        drop(device_on_the_first_card);
        drop(device_on_the_second_card);
    }

    /// 第 10z 批第十五块（第 10z 批第十二、十三块代码轮判决 K4，正推 T1 拦 Z8：借之前没丢的那一支 poll 之后销这张卡此刻全部没等成的、
    /// 不只列出来的，照绿）：poll 的当口（不在账的锁里）又有一笔借据记成没等成；poll 交回队列空了：列出来的那一笔销账，
    /// 列卡之后才记成没等成的那一笔照旧记着（它的活可以是 poll 之后才提交的）。
    #[test]
    fn a_lease_kept_after_the_listing_survives_the_poll_before_booking() {
        let (card, ledger) = ledger_on_a_card_under_the_ledger();
        let (device, listed_lease, _kept_mebibytes) =
            a_started_device_with_all_but_one_thousand_kept_after_a_timed_out_wait(&card, &ledger);
        let lease_kept_during_the_poll = std::cell::Cell::new(None);
        ledger.take_in_the_devices_kept_before_booking(
            |card_position| (card_position == 0).then_some(&device),
            |_kept| false,
            |kept| {
                let mut lease = ledger
                    .book_on_the_card(0, 500)
                    .expect("还剩 1 000：借得到 500");
                lease.take_in_the_wait_before_returning(
                    kept,
                    DeviceWaitBeforeReturningTheLease::TheWaitFailed,
                );
                lease_kept_during_the_poll.set(Some(lease.lease_identifier()));
                drop(lease);
                true
            },
            |kept| *kept,
        );
        assert!(
            !ledger.is_the_lease_on_the_books(0, listed_lease),
            "列出来的那一笔：poll 交回队列空了，销账"
        );
        assert!(
            ledger.is_the_lease_on_the_books(
                0,
                lease_kept_during_the_poll
                    .get()
                    .expect("poll 的当口记了一笔没等成的")
            ),
            "列卡之后才记成没等成的那一笔照旧记着"
        );
        drop(device);
    }

    /// 第 10z 批第十五块（K4，正推 T2 拦 Z6：告诉账丢了时在用的那一台不进丢了还没放掉的那一堆，照绿）：借之前告诉账设备丢了、
    /// 之后重起好了，丢了的那一台连它的缓冲与驱动那一份还占着卡（还没放掉）：能借的不超过卡上真能再给的。
    #[test]
    fn the_lost_device_still_on_the_card_after_the_early_loss_and_the_restart_is_not_lent_again() {
        let (card, ledger) = ledger_on_a_card_under_the_ledger();
        let (device, _kept_lease, kept_mebibytes) =
            a_started_device_with_all_but_one_thousand_kept_after_a_timed_out_wait(&card, &ledger);
        {
            let mut card = card.lock().expect("假卡不 panic");
            card.device_started = false;
            card.driver_occupied_mebibytes = 0;
            card.built_mebibytes = 0;
            card.left_by_a_lost_device_mebibytes = 600 + kept_mebibytes;
        }
        ledger.take_in_the_devices_kept_before_booking(
            |card_position| (card_position == 0).then_some(&device),
            |_kept| true,
            |_kept| panic!("丢了的设备不 poll"),
            |kept| *kept,
        );
        ledger
            .prepare_the_driver_share_before_starting_the_device(0)
            .expect("重起：驱动那一份放得下");
        let replacement = ledger.record_the_device_started_on_the_card(0);
        {
            let mut card = card.lock().expect("假卡不 panic");
            card.device_started = true;
            card.driver_occupied_mebibytes = 600;
        }
        let room = room_now(&ledger);
        let room_truly_left_on_the_card = room_truly_left(&card, 0);
        assert!(
            room <= room_truly_left_on_the_card,
            "丢了的那一台还在卡上：能借的 {room} 不超过卡上真能再给的 {room_truly_left_on_the_card}"
        );
        drop(replacement);
        drop(device);
    }

    /// 第 10z 批第十五块（K4，正推 T3 拦 Z1：取留着的设备缺了的那张卡就停、不看后面的卡，照绿）：第 0 张卡那一格没有留着的设备
    /// （正在起），第 1 张卡的设备队列空了：第 1 张卡上那一笔照样销账，第 0 张卡上那一笔照旧记着。
    #[test]
    fn a_card_whose_device_is_still_starting_does_not_keep_the_next_card_from_returning_its_kept_lease(
    ) {
        let readings = replaceable_readings(vec![
            reading_of_an_rtx_5090(AN_EMPTY_RTX_5090_HAS),
            reading_of_an_rtx_5090(AN_EMPTY_RTX_5090_HAS),
        ]);
        let ledger = ledger_reading(&readings);
        let mut devices = Vec::new();
        let mut kept_leases = Vec::new();
        for card_position in 0..2 {
            let mut lease = ledger
                .book_on_the_card(card_position, 1_000)
                .expect("空卡借得到");
            let device = ledger.record_the_device_started_on_the_card(card_position);
            lease.take_in_the_wait_before_returning(
                &device,
                DeviceWaitBeforeReturningTheLease::TheWaitFailed,
            );
            kept_leases.push(lease.lease_identifier());
            drop(lease);
            devices.push(device);
        }
        ledger.take_in_the_devices_kept_before_booking(
            |card_position| (card_position == 1).then(|| &devices[1]),
            |_kept| false,
            |_kept| true,
            |kept| *kept,
        );
        assert!(
            ledger.is_the_lease_on_the_books(0, kept_leases[0]),
            "第 0 张卡那一格没有留着的设备可 poll：那一笔照旧记着"
        );
        assert!(
            !ledger.is_the_lease_on_the_books(1, kept_leases[1]),
            "第 1 张卡的设备队列空了：那一笔销账"
        );
    }

    /// 第 10z 批第十五块（第 10z 批第十二、十三块代码轮判决 K3-3，正推 T4）：设备起好（驱动实占 600）、头一笔等完设备还了，之后设备丢了，
    /// 它的驱动那一份还占着卡，账上没有没等成的借据（列卡列不出这张卡）。账还不知道丢了时照读数把那 600 当成这个进程许出去的：
    /// 能借的 29 550、卡上真能再给的 28 950，借走之后重起被拒。借之前那一步看池里每张卡：丢了的照重起记，能借的就是卡上真能再给的，
    /// 借走它之后重起放得下。
    #[test]
    fn a_device_lost_without_a_kept_lease_is_taken_in_before_booking_so_the_room_lent_leaves_space_for_the_restart(
    ) {
        let (card, ledger) = ledger_on_a_card_under_the_ledger();
        let mut first = ledger.book_on_the_card(0, 1).expect("空卡借得到");
        let device = ledger.record_the_device_started_on_the_card(0);
        {
            let mut card = card.lock().expect("假卡不 panic");
            card.device_started = true;
            card.driver_occupied_mebibytes = 600;
        }
        first.take_in_the_wait_before_returning(
            &device,
            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
        );
        drop(first);
        {
            let mut card = card.lock().expect("假卡不 panic");
            card.device_started = false;
            card.driver_occupied_mebibytes = 0;
            card.left_by_a_lost_device_mebibytes = 600;
        }
        assert_eq!(
            room_now(&ledger),
            29_550,
            "账还不知道设备丢了：把丢了的设备还占着的驱动那一份 600 当成许出去的（正推 T4 room_now）"
        );
        assert_eq!(
            room_truly_left(&card, 0),
            28_950,
            "卡上真能再给的（正推 T4 truly_left）"
        );
        let losses_asked = AtomicUsize::new(0);
        ledger.take_in_the_devices_kept_before_booking(
            |card_position| (card_position == 0).then_some(&device),
            |_kept| {
                losses_asked.fetch_add(1, Ordering::SeqCst);
                true
            },
            |_kept| panic!("丢了的设备不 poll"),
            |kept| *kept,
        );
        assert_eq!(
            losses_asked.load(Ordering::SeqCst),
            1,
            "没有没等成借据的卡也问一次丢没丢"
        );
        let room = room_now(&ledger);
        assert_eq!(
            room,
            room_truly_left(&card, 0),
            "借之前账就知道丢了：能借的就是卡上真能再给的（不多给，也不少给）"
        );
        let booking = ledger
            .book_on_the_card(0, room)
            .expect("能借的那么多借得到");
        assert_eq!(
            ledger.prepare_the_driver_share_before_starting_the_device(0),
            Ok(()),
            "借走能借的全部之后重起：驱动那一份照此刻放得下"
        );
        let replacement = ledger.record_the_device_started_on_the_card(0);
        drop(booking);
        drop(replacement);
        drop(device);
    }

    /// 第 10z 批第十五块（K3-3 那一改：借之前看池里每张卡，只 poll 列出来的）：第 0 张卡的设备起着、没丢、没有没等成的借据，
    /// 第 1 张卡上一笔没等成：只 poll 第 1 张卡，那一笔销账，第 0 张卡的账不动。
    #[test]
    fn a_card_without_a_kept_lease_is_not_polled_before_booking() {
        let readings = replaceable_readings(vec![
            reading_of_an_rtx_5090(AN_EMPTY_RTX_5090_HAS),
            reading_of_an_rtx_5090(AN_EMPTY_RTX_5090_HAS),
        ]);
        let ledger = ledger_reading(&readings);
        let booking_on_the_first_card = ledger.book_on_the_card(0, 1_000).expect("空卡借得到");
        let device_on_the_first_card = ledger.record_the_device_started_on_the_card(0);
        let mut lease_on_the_second_card = ledger.book_on_the_card(1, 1_000).expect("空卡借得到");
        let device_on_the_second_card = ledger.record_the_device_started_on_the_card(1);
        lease_on_the_second_card.take_in_the_wait_before_returning(
            &device_on_the_second_card,
            DeviceWaitBeforeReturningTheLease::TheWaitFailed,
        );
        let kept_on_the_second_card = lease_on_the_second_card.lease_identifier();
        drop(lease_on_the_second_card);
        let devices = [&device_on_the_first_card, &device_on_the_second_card];
        let rooms_before = ledger.card_rooms_mebibytes_now(&[0]).expect("假读数读得出");
        let polls_on_each_card = [AtomicUsize::new(0), AtomicUsize::new(0)];
        ledger.take_in_the_devices_kept_before_booking(
            |card_position| devices.get(card_position).copied(),
            |_kept| false,
            |kept| {
                polls_on_each_card[kept.card_position].fetch_add(1, Ordering::SeqCst);
                true
            },
            |kept| *kept,
        );
        assert_eq!(
            polls_on_each_card
                .iter()
                .map(|polls| polls.load(Ordering::SeqCst))
                .collect::<Vec<usize>>(),
            vec![0, 1],
            "只 poll 列出来的那张卡"
        );
        assert!(
            !ledger.is_the_lease_on_the_books(1, kept_on_the_second_card),
            "第 1 张卡的设备队列空了：那一笔销账"
        );
        assert_eq!(
            ledger.card_rooms_mebibytes_now(&[0]).expect("假读数读得出"),
            rooms_before,
            "第 0 张卡的账不动"
        );
        drop(booking_on_the_first_card);
        drop(device_on_the_first_card);
        drop(device_on_the_second_card);
    }

    /// 随机操作串里的一步（[`random_operation_strings_never_lend_more_than_the_card_has_left_once_the_ledger_knows_of_the_loss`]）。
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum RandomStepOnTheCard {
        /// 借 1 000、4 000、9 000 或 15 000（借不到就不借）。
        Book,
        /// 设备没丢时，把头一笔没建的借据建满。
        BuildTheFirstUnbuiltLease,
        /// 随机丢一笔借据：设备没丢就等成（它与之前没等成的缓冲都交回驱动），丢了就没等成。
        DropALeaseAfterTheWait,
        /// 随机丢一笔借据，没等成（建过的缓冲留在卡上）。
        DropALeaseAfterAFailedWait,
        /// 设备丢了：它的驱动那一份与卡上的缓冲都成了丢了的设备留在卡上的。
        LoseTheDevice,
        /// 卡池借之前那一步（设备丢了就交丢了，没丢的 poll 交还有在飞的）。
        StepBeforeBooking,
        /// 设备丢了就重起（驱动那一份照此刻判，放得下才起）。
        RestartTheLostDevice,
        /// 放掉最早的那一台丢了的设备（它留在卡上的都交回驱动要等最后一台放掉）。
        ReleaseTheEarliestLostDevice,
    }

    /// 随机操作串每一步从这里按下标取（与正推腿 T5 的操作号同序：0 借、1 建、2 等成丢、3 没等成丢、4 丢设备、5 借之前那一步、6 重起、7 放掉）。
    const RANDOM_STEPS_ON_THE_CARD: [RandomStepOnTheCard; 8] = [
        RandomStepOnTheCard::Book,
        RandomStepOnTheCard::BuildTheFirstUnbuiltLease,
        RandomStepOnTheCard::DropALeaseAfterTheWait,
        RandomStepOnTheCard::DropALeaseAfterAFailedWait,
        RandomStepOnTheCard::LoseTheDevice,
        RandomStepOnTheCard::StepBeforeBooking,
        RandomStepOnTheCard::RestartTheLostDevice,
        RandomStepOnTheCard::ReleaseTheEarliestLostDevice,
    ];

    /// 随机操作串借的那几种量（MiB）。
    const RANDOM_BOOKING_SIZES_MEBIBYTES: [u64; 4] = [1_000, 4_000, 9_000, 15_000];

    /// 线性同余的随机数（正推腿 T5 那一个，同一个种子出同一串）：种子先乘加混一次，之后每取一次乘加一次、取高 31 位。
    struct OperationStringGenerator {
        state: u64,
    }

    impl OperationStringGenerator {
        const SEED_MULTIPLIER: u64 = 2_862_933_555_777_941_757;
        const SEED_INCREMENT: u64 = 3_037_000_493;
        const STEP_MULTIPLIER: u64 = 6_364_136_223_846_793_005;
        const STEP_INCREMENT: u64 = 1_442_695_040_888_963_407;
        const DISCARDED_LOW_BITS: u32 = 33;

        fn from_seed(seed: u64) -> Self {
            Self {
                state: seed
                    .wrapping_mul(Self::SEED_MULTIPLIER)
                    .wrapping_add(Self::SEED_INCREMENT),
            }
        }

        /// `0..bound` 里的一个（`bound` 不为 0）。
        fn next_below(&mut self, bound: usize) -> usize {
            self.state = self
                .state
                .wrapping_mul(Self::STEP_MULTIPLIER)
                .wrapping_add(Self::STEP_INCREMENT);
            let bound_as_u64 = u64::try_from(bound).expect("下标的上界装得进 u64");
            usize::try_from((self.state >> Self::DISCARDED_LOW_BITS) % bound_as_u64)
                .expect("比 bound 小，装得进 usize")
        }
    }

    /// 随机操作串里一笔活着的借据建到哪一步。
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum LeaseBuiltOnTheCard {
        NotBuilt,
        BuiltOnTheDeviceInUse,
        /// 建在后来丢了的那一台上（缓冲已经算进丢了的设备留在卡上的）。
        BuiltOnALostDevice,
    }

    /// 第 10z 批第十五块（第 10z 批第十二、十三块代码轮判决 K4，正推 T5：随机操作串拦「告诉账丢了只销那几笔、不照重起记」与 Z6；
    /// K3-3 改了之后借之前那一步看见丢了就记）：一张假卡、一台设备，4 000 个种子各 14 步随机的借、建、丢借据（等成或没等成）、设备丢了、
    /// 借之前那一步、重起、放掉旧设备。设备没丢、或丢了而账已经知道（丢了之后走过借之前那一步，或重起那一步判过）的每一步之后：
    /// 能借的不超过卡上真能再给的（建过没还的缓冲、丢了的设备留在卡上的都不在空闲里，活着而没建的借据还要建）。
    #[test]
    fn random_operation_strings_never_lend_more_than_the_card_has_left_once_the_ledger_knows_of_the_loss(
    ) {
        const SEEDS: u64 = 4_000;
        const STEPS_IN_A_STRING: usize = 14;
        const DRIVER_HOLDS_ONCE_STARTED: u64 = 600;
        let mut states_checked = 0usize;
        let mut first_overgiving_strings: Vec<String> = Vec::new();
        let mut overgiving_states = 0usize;
        for seed in 0..SEEDS {
            let mut generator = OperationStringGenerator::from_seed(seed);
            let (card, ledger) = ledger_on_a_card_under_the_ledger();
            let mut first = ledger.book_on_the_card(0, 1).expect("空卡借得到");
            let first_device = ledger.record_the_device_started_on_the_card(0);
            {
                let mut card = card.lock().expect("假卡不 panic");
                card.device_started = true;
                card.driver_occupied_mebibytes = DRIVER_HOLDS_ONCE_STARTED;
            }
            first.take_in_the_wait_before_returning(
                &first_device,
                DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
            );
            drop(first);
            let mut device_in_use: std::rc::Rc<DeviceStartedOnTheBooks<'_>> =
                std::rc::Rc::new(first_device);
            let mut lost_devices_kept: Vec<std::rc::Rc<DeviceStartedOnTheBooks<'_>>> = Vec::new();
            let mut alive: Vec<(GpuMemoryBooking<'_>, LeaseBuiltOnTheCard, u64)> = Vec::new();
            let mut is_the_device_lost = false;
            let mut does_the_ledger_know_of_the_loss = false;
            let mut built_by_leases_dropped_after_a_failed_wait_mebibytes: u64 = 0;
            let mut steps_taken = String::new();
            for _step_in_the_string in 0..STEPS_IN_A_STRING {
                let step =
                    RANDOM_STEPS_ON_THE_CARD[generator.next_below(RANDOM_STEPS_ON_THE_CARD.len())];
                match step {
                    RandomStepOnTheCard::Book => {
                        let mebibytes = RANDOM_BOOKING_SIZES_MEBIBYTES
                            [generator.next_below(RANDOM_BOOKING_SIZES_MEBIBYTES.len())];
                        if let Ok(booking) = ledger.book_on_the_card(0, mebibytes) {
                            alive.push((booking, LeaseBuiltOnTheCard::NotBuilt, mebibytes));
                            steps_taken.push_str(&format!("借{mebibytes} "));
                        }
                    }
                    RandomStepOnTheCard::BuildTheFirstUnbuiltLease => {
                        if !is_the_device_lost {
                            if let Some(unbuilt) =
                                alive.iter_mut().find(|(_booking, built, _mebibytes)| {
                                    *built == LeaseBuiltOnTheCard::NotBuilt
                                })
                            {
                                card.lock().expect("假卡不 panic").built_mebibytes += unbuilt.2;
                                unbuilt.1 = LeaseBuiltOnTheCard::BuiltOnTheDeviceInUse;
                                steps_taken.push_str("建 ");
                            }
                        }
                    }
                    RandomStepOnTheCard::DropALeaseAfterTheWait
                    | RandomStepOnTheCard::DropALeaseAfterAFailedWait => {
                        if !alive.is_empty() {
                            let (mut booking, built, mebibytes) =
                                alive.remove(generator.next_below(alive.len()));
                            let is_the_wait_finished = step
                                == RandomStepOnTheCard::DropALeaseAfterTheWait
                                && !is_the_device_lost;
                            booking.take_in_the_wait_before_returning(
                                &device_in_use,
                                if is_the_wait_finished {
                                    DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished
                                } else {
                                    DeviceWaitBeforeReturningTheLease::TheWaitFailed
                                },
                            );
                            let built_on_the_device_in_use =
                                if built == LeaseBuiltOnTheCard::BuiltOnTheDeviceInUse {
                                    mebibytes
                                } else {
                                    0
                                };
                            if is_the_wait_finished {
                                card.lock().expect("假卡不 panic").built_mebibytes -=
                                    built_by_leases_dropped_after_a_failed_wait_mebibytes
                                        + built_on_the_device_in_use;
                                built_by_leases_dropped_after_a_failed_wait_mebibytes = 0;
                            } else {
                                built_by_leases_dropped_after_a_failed_wait_mebibytes +=
                                    built_on_the_device_in_use;
                            }
                            drop(booking);
                            steps_taken.push_str(if is_the_wait_finished {
                                "等成丢 "
                            } else {
                                "没等成丢 "
                            });
                        }
                    }
                    RandomStepOnTheCard::LoseTheDevice => {
                        if !is_the_device_lost {
                            is_the_device_lost = true;
                            does_the_ledger_know_of_the_loss = false;
                            let mut card = card.lock().expect("假卡不 panic");
                            card.left_by_a_lost_device_mebibytes +=
                                card.driver_occupied_mebibytes + card.built_mebibytes;
                            card.built_mebibytes = 0;
                            card.driver_occupied_mebibytes = 0;
                            card.device_started = false;
                            for (_booking, built, _mebibytes) in &mut alive {
                                if *built == LeaseBuiltOnTheCard::BuiltOnTheDeviceInUse {
                                    *built = LeaseBuiltOnTheCard::BuiltOnALostDevice;
                                }
                            }
                            built_by_leases_dropped_after_a_failed_wait_mebibytes = 0;
                            steps_taken.push_str("丢设备 ");
                        }
                    }
                    RandomStepOnTheCard::StepBeforeBooking => {
                        ledger.take_in_the_devices_kept_before_booking(
                            |_card_position| Some(std::rc::Rc::clone(&device_in_use)),
                            |_kept| is_the_device_lost,
                            |_kept| false,
                            |kept| &**kept,
                        );
                        if is_the_device_lost {
                            does_the_ledger_know_of_the_loss = true;
                        }
                        steps_taken.push_str("借之前 ");
                    }
                    RandomStepOnTheCard::RestartTheLostDevice => {
                        if is_the_device_lost {
                            does_the_ledger_know_of_the_loss = true;
                            if ledger
                                .prepare_the_driver_share_before_starting_the_device(0)
                                .is_ok()
                            {
                                let replacement = ledger.record_the_device_started_on_the_card(0);
                                lost_devices_kept.push(std::mem::replace(
                                    &mut device_in_use,
                                    std::rc::Rc::new(replacement),
                                ));
                                is_the_device_lost = false;
                                let mut card = card.lock().expect("假卡不 panic");
                                card.device_started = true;
                                card.driver_occupied_mebibytes = DRIVER_HOLDS_ONCE_STARTED;
                                steps_taken.push_str("重起 ");
                            } else {
                                steps_taken.push_str("重起被拒 ");
                            }
                        }
                    }
                    RandomStepOnTheCard::ReleaseTheEarliestLostDevice => {
                        if !lost_devices_kept.is_empty() {
                            drop(lost_devices_kept.remove(0));
                            if lost_devices_kept.is_empty() {
                                card.lock()
                                    .expect("假卡不 panic")
                                    .left_by_a_lost_device_mebibytes = 0;
                            }
                            steps_taken.push_str("放掉 ");
                        }
                    }
                }
                let unbuilt_mebibytes: u64 = alive
                    .iter()
                    .filter(|(_booking, built, _mebibytes)| *built == LeaseBuiltOnTheCard::NotBuilt)
                    .map(|(_booking, _built, mebibytes)| *mebibytes)
                    .sum();
                if !is_the_device_lost || does_the_ledger_know_of_the_loss {
                    states_checked += 1;
                    let room = room_now(&ledger);
                    let room_truly_left_on_the_card = room_truly_left(&card, unbuilt_mebibytes);
                    if room > room_truly_left_on_the_card {
                        overgiving_states += 1;
                        if first_overgiving_strings.len() < 5 {
                            first_overgiving_strings.push(format!(
                                "种子 {seed}：{steps_taken}能借 {room}，卡上真能再给 {room_truly_left_on_the_card}"
                            ));
                        }
                    }
                }
            }
            drop(alive);
            drop(lost_devices_kept);
            drop(device_in_use);
        }
        assert!(states_checked > 0, "随机串里至少有一步落在账知道的状态上");
        assert_eq!(
            overgiving_states, 0,
            "核了 {states_checked} 步，能借的都不超过卡上真能再给的；头几条多给的：{first_overgiving_strings:#?}"
        );
    }

    /// 一台假设备：最后一个克隆丢掉时（wgpu 收设备那一下要等设备空了）先报一声、等用例放开才收完；`number` 认是哪一台。
    struct DeviceWhoseLastDropWaits {
        number: u32,
        dropping_began: Mutex<Option<std::sync::mpsc::Sender<()>>>,
        drop_may_finish: Mutex<Option<std::sync::mpsc::Receiver<()>>>,
    }

    impl Drop for DeviceWhoseLastDropWaits {
        fn drop(&mut self) {
            if let Some(dropping_began) =
                self.dropping_began.lock().expect("只在丢的这一下拿").take()
            {
                // 用例已经不等了（断言失败、先丢了收的那一头）时照样往下收
                let _sent_or_the_test_gave_up = dropping_began.send(());
            }
            if let Some(drop_may_finish) = self
                .drop_may_finish
                .lock()
                .expect("只在丢的这一下拿")
                .take()
            {
                let _finished_or_the_test_gave_up = drop_may_finish.recv();
            }
        }
    }

    /// 第 10z 批第十三块（第 10z 批第十、十一块代码轮判决 K5，正推 G2：换下来的旧设备改回在 `started` 的锁里丢照绿）：留着的设备 1 丢了，
    /// 一条线程重起、换上 2，丢 1 时卡住（等设备空了）；这时取留着的设备不等丢旧设备的那一下，交新起的 2。
    #[test]
    fn taking_the_device_kept_does_not_wait_for_the_replaced_device_to_be_dropped() {
        let slot: DeviceStartedOnDemand<Arc<DeviceWhoseLastDropWaits>> =
            DeviceStartedOnDemand::new();
        let (dropping_began_sender, dropping_began) = std::sync::mpsc::channel::<()>();
        let (finish_the_drop, drop_may_finish) = std::sync::mpsc::channel::<()>();
        let first_started = slot.started_or_start(
            |_device: &Arc<DeviceWhoseLastDropWaits>| false,
            move || {
                Ok::<Arc<DeviceWhoseLastDropWaits>, String>(Arc::new(DeviceWhoseLastDropWaits {
                    number: 1,
                    dropping_began: Mutex::new(Some(dropping_began_sender)),
                    drop_may_finish: Mutex::new(Some(drop_may_finish)),
                }))
            },
        );
        assert_eq!(first_started.map(|device| device.number), Ok(1));
        let slot_shared = &slot;
        let (taken_sender, taken) = std::sync::mpsc::channel::<Option<u32>>();
        std::thread::scope(|scope| {
            let restarting = scope.spawn(move || {
                slot_shared
                    .started_or_start(
                        |device: &Arc<DeviceWhoseLastDropWaits>| device.number == 1,
                        || {
                            Ok::<Arc<DeviceWhoseLastDropWaits>, String>(Arc::new(
                                DeviceWhoseLastDropWaits {
                                    number: 2,
                                    dropping_began: Mutex::new(None),
                                    drop_may_finish: Mutex::new(None),
                                },
                            ))
                        },
                    )
                    .map(|device| device.number)
            });
            let dropping_the_replaced_device =
                dropping_began.recv_timeout(std::time::Duration::from_secs(10));
            scope.spawn(move || {
                // 用例已经不等了时照样收尾
                let _sent_or_the_test_gave_up =
                    taken_sender.send(slot_shared.started_now().map(|device| device.number));
            });
            let taken_while_the_replaced_device_drops =
                taken.recv_timeout(std::time::Duration::from_secs(10));
            finish_the_drop.send(()).expect("丢旧设备那一下在等放开");
            assert_eq!(
                dropping_the_replaced_device,
                Ok(()),
                "重起换上 2 之后丢旧设备 1"
            );
            assert_eq!(
                taken_while_the_replaced_device_drops,
                Ok(Some(2)),
                "丢旧设备的那一下不拿着留着的那一份：取留着的设备不等，交新起的 2"
            );
            assert_eq!(restarting.join().expect("重起不 panic"), Ok(2));
        });
    }

    /// 第 10z 批第八块（第 10z 批第六块代码轮攻方改法 F-K1）：没等成的那一笔记着，同一台设备上后来另一笔借据等成了（等的是到那一刻为止的全部提交）：
    /// 没等成的那一笔一起销账。
    #[test]
    fn a_lease_whose_wait_timed_out_leaves_the_books_when_another_lease_on_its_device_finishes_its_wait(
    ) {
        let (card, ledger) = ledger_on_a_card_under_the_ledger();
        let mut finishing_later = ledger.book_on_the_card(0, 2_000).expect("空卡借得到");
        let device = ledger.record_the_device_started_on_the_card(0);
        {
            let mut card = card.lock().expect("假卡不 panic");
            card.device_started = true;
            card.driver_occupied_mebibytes = 600;
            card.built_mebibytes = 2_000;
        }
        let timed_out_lease =
            a_lease_built_and_dropped_after_a_timed_out_wait(&card, &ledger, &device, 6_000);
        card.lock().expect("假卡不 panic").built_mebibytes = 0;
        finishing_later.take_in_the_wait_before_returning(
            &device,
            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
        );
        assert!(
            !ledger.is_the_lease_on_the_books(0, timed_out_lease),
            "同一台上另一笔等成了：没等成的那一笔的活也做完了，销账"
        );
        drop(finishing_later);
        assert_eq!(room_now(&ledger), room_truly_left(&card, 0));
    }

    /// 第 10z 批第八块（第 10z 批第六块代码轮判决 K1-3，正推：触发重起的那一笔拿的是新设备）：A 建满在旧设备上，设备丢了；B 借到、照重起记、
    /// 新设备起好、B 建满，B 等新设备超时、丢掉（缓冲还在卡上）。A 在旧设备上等成了：不销 B（B 等的是新设备）。旧设备放掉之后照读数分，
    /// B 照旧记着，不多给；新设备后来 poll 交队列空了、B 的缓冲还给驱动，B 才销账。
    #[test]
    fn a_finished_wait_on_a_lost_device_leaves_a_lease_kept_on_its_replacement_on_the_books() {
        let (card, ledger) = ledger_on_a_card_under_the_ledger();
        let (mut lease_on_the_lost_device, lost_device) =
            ten_thousand_built_with_the_device_held(&card, &ledger);
        let mut lease_starting_the_replacement =
            ledger.book_on_the_card(0, 5_000).expect("借得到 5 000");
        let starting_the_replacement = lease_starting_the_replacement.lease_identifier();
        {
            let mut card = card.lock().expect("假卡不 panic");
            card.device_started = false;
            card.built_mebibytes = 0;
            card.left_by_a_lost_device_mebibytes = DRIVER + 10_000;
        }
        assert_eq!(
            ledger.prepare_the_driver_share_before_starting_the_device(0),
            Ok(())
        );
        let replacement = ledger.record_the_device_started_on_the_card(0);
        {
            // 新设备的驱动还没长满（实占 600）：B 的缓冲要是被当成驱动还要长的那一份，就看得出多给
            let mut card = card.lock().expect("假卡不 panic");
            card.device_started = true;
            card.driver_occupied_mebibytes = 600;
            card.built_mebibytes = 5_000;
        }
        lease_starting_the_replacement.take_in_the_wait_before_returning(
            &replacement,
            DeviceWaitBeforeReturningTheLease::TheWaitFailed,
        );
        drop(lease_starting_the_replacement);
        lease_on_the_lost_device.take_in_the_wait_before_returning(
            &lost_device,
            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
        );
        assert!(
            ledger.is_the_lease_on_the_books(0, starting_the_replacement),
            "旧设备上等成了，说明不了新设备上 B 的活做完了：B 照旧记着"
        );
        card.lock()
            .expect("假卡不 panic")
            .left_by_a_lost_device_mebibytes = 0;
        drop(lost_device);
        drop(lease_on_the_lost_device);
        assert!(
            room_now(&ledger) <= room_truly_left(&card, 0),
            "旧设备放掉、照读数分：B 的缓冲还在卡上，不多给（{} 对 {}）",
            room_now(&ledger),
            room_truly_left(&card, 0)
        );
        card.lock().expect("假卡不 panic").built_mebibytes = 0;
        let listed = ledger.leases_kept_after_a_failed_wait_on_each_card();
        assert_eq!(listed.len(), 1, "B 照旧记在这张卡上：列得出");
        replacement.take_in_that_the_work_of_the_listed_leases_finished(&listed[0]);
        assert!(!ledger.is_the_lease_on_the_books(0, starting_the_replacement));
        assert_eq!(room_now(&ledger), room_truly_left(&card, 0));
    }

    /// 借一笔 10 000、起设备、建满（驱动实占 [`DRIVER`]），交回借据与账上的这一台（`Arc`：借据的卡拿着一份；卡池那一格的那一份
    /// 由调用方丢的时候另克隆，这里不留）。
    fn ten_thousand_built_with_the_device_held<'ledger>(
        card: &Mutex<CardUnderTheLedger>,
        ledger: &'ledger GpuMemoryLedger,
    ) -> (
        GpuMemoryBooking<'ledger>,
        Arc<DeviceStartedOnTheBooks<'ledger>>,
    ) {
        let lease = ledger
            .book_on_the_card(0, 10_000)
            .expect("空卡借得到 10 000");
        assert_eq!(
            ledger.prepare_the_driver_share_before_starting_the_device(0),
            Ok(())
        );
        let device = Arc::new(ledger.record_the_device_started_on_the_card(0));
        {
            let mut card = card.lock().expect("假卡不 panic");
            card.device_started = true;
            card.driver_occupied_mebibytes = DRIVER;
            card.built_mebibytes = 10_000;
        }
        (lease, device)
    }

    /// 第 10z 批第八块（第 10z 批第六块代码轮判决 K1-3、攻方 H2 与改法 F-K3 自报的风险）：旧设备上借着 A 10 000（建满，驱动实占 600），
    /// 设备丢了；L 5 000 借到、照重起记、新设备起好、L 建满，卡池那一格换成新设备。A 等旧设备没等成、丢掉：照常销账（这时照分不出算）。
    /// 调用方另留着旧设备的一个克隆（产品路径的 `Arc<GpuCard>` 一类）：旧设备连 A 的缓冲还占着卡，L 也还了，这张卡照旧照分不出算，不多给
    /// （照「那一刻活着的借据都还了」判，这时回到照读数分，旧设备的那一份被当成新借据建上的）。那个克隆也丢了、旧设备放掉，照读数分：
    /// 长借据 Q 15 000 建满之后再借 10 000，借得到，能借的就是卡上真能再给的（攻方 `k1_3` 那六段的形状）。
    #[test]
    fn a_lost_device_still_held_outside_the_leases_keeps_the_card_unsplittable_until_it_is_released(
    ) {
        const DRIVER_HOLDS_NOW: u64 = 600;
        let (card, ledger) = ledger_on_a_card_under_the_ledger();
        let mut lease_a = ledger.book_on_the_card(0, 10_000).expect("空卡借得到");
        let lease_a_identifier = lease_a.lease_identifier();
        assert_eq!(
            ledger.prepare_the_driver_share_before_starting_the_device(0),
            Ok(())
        );
        let held_by_the_pool = Arc::new(ledger.record_the_device_started_on_the_card(0));
        let held_by_lease_a = Arc::clone(&held_by_the_pool);
        let held_by_the_caller = Arc::clone(&held_by_the_pool);
        {
            let mut card = card.lock().expect("假卡不 panic");
            card.device_started = true;
            card.driver_occupied_mebibytes = DRIVER_HOLDS_NOW;
            card.built_mebibytes = 10_000;
        }
        {
            let mut card = card.lock().expect("假卡不 panic");
            card.device_started = false;
            card.built_mebibytes = 0;
            card.left_by_a_lost_device_mebibytes = DRIVER_HOLDS_NOW + 10_000;
        }
        let mut lease_l = ledger.book_on_the_card(0, 5_000).expect("借得到 5 000");
        assert_eq!(
            ledger.prepare_the_driver_share_before_starting_the_device(0),
            Ok(()),
            "重起放得下"
        );
        let replacement = ledger.record_the_device_started_on_the_card(0);
        {
            let mut card = card.lock().expect("假卡不 panic");
            card.device_started = true;
            card.driver_occupied_mebibytes = DRIVER_HOLDS_NOW;
            card.built_mebibytes = 5_000;
        }
        drop(held_by_the_pool);
        lease_a.take_in_the_wait_before_returning(
            &held_by_lease_a,
            DeviceWaitBeforeReturningTheLease::TheWaitFailed,
        );
        drop(held_by_lease_a);
        drop(lease_a);
        assert!(
            !ledger.is_the_lease_on_the_books(0, lease_a_identifier),
            "A 等的是丢了还没放掉的那一台：没等成也照常销账，不卡在账上"
        );
        assert!(
            room_now(&ledger) <= room_truly_left(&card, 0),
            "旧设备还被调用方拿着、连 A 的缓冲占着卡：不多给（{} 对 {}）",
            room_now(&ledger),
            room_truly_left(&card, 0)
        );
        card.lock().expect("假卡不 panic").built_mebibytes = 0;
        lease_l.take_in_the_wait_before_returning(
            &replacement,
            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
        );
        drop(lease_l);
        assert!(
            room_now(&ledger) <= room_truly_left(&card, 0),
            "L 也还了、旧设备还在：照旧照分不出算，不多给（{} 对 {}）",
            room_now(&ledger),
            room_truly_left(&card, 0)
        );
        card.lock()
            .expect("假卡不 panic")
            .left_by_a_lost_device_mebibytes = 0;
        drop(held_by_the_caller);
        assert_eq!(
            room_now(&ledger),
            room_truly_left(&card, 0),
            "旧设备放掉了：照读数分，能借的是卡上真能再给的"
        );
        let _long_lease = ledger.book_on_the_card(0, 15_000).expect("借得到 15 000");
        card.lock().expect("假卡不 panic").built_mebibytes = 15_000;
        assert_eq!(room_now(&ledger), room_truly_left(&card, 0));
        assert!(
            ledger.book_on_the_card(0, 10_000).is_ok(),
            "Q 建满之后卡上放得下 10 000：借得到"
        );
    }

    /// 第 10z 批第八块（第 10z 批第六块代码轮正推探针 P5、攻方 H5 的 X7：重起好了、旧设备还在时又丢一次，不重新记账）：第一台丢了、调用方还拿着它，
    /// 第二台起好、L 在它上面建满之后也丢了，M 借到、照重起记、第三台起好。第一台放掉了，第二台还被 L 的卡与调用方拿着、连 L 的缓冲占着卡：
    /// 照旧照分不出算，不多给；L 还了、第二台也放掉，照读数分。
    #[test]
    fn a_second_loss_while_the_first_lost_device_remains_keeps_the_card_unsplittable_until_both_are_released(
    ) {
        const DRIVER_HOLDS_NOW: u64 = 600;
        let (card, ledger) = ledger_on_a_card_under_the_ledger();
        let mut lease_on_the_first = ledger.book_on_the_card(0, 1_000).expect("空卡借得到");
        assert_eq!(
            ledger.prepare_the_driver_share_before_starting_the_device(0),
            Ok(())
        );
        let first_device = ledger.record_the_device_started_on_the_card(0);
        {
            let mut card = card.lock().expect("假卡不 panic");
            card.device_started = true;
            card.driver_occupied_mebibytes = DRIVER_HOLDS_NOW;
        }
        lease_on_the_first.take_in_the_wait_before_returning(
            &first_device,
            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
        );
        drop(lease_on_the_first);
        {
            let mut card = card.lock().expect("假卡不 panic");
            card.device_started = false;
            card.left_by_a_lost_device_mebibytes = DRIVER_HOLDS_NOW;
        }
        let mut lease_l = ledger.book_on_the_card(0, 5_000).expect("借得到 5 000");
        assert_eq!(
            ledger.prepare_the_driver_share_before_starting_the_device(0),
            Ok(()),
            "第一次重起"
        );
        let second_device = Arc::new(ledger.record_the_device_started_on_the_card(0));
        let second_held_by_lease_l = Arc::clone(&second_device);
        {
            let mut card = card.lock().expect("假卡不 panic");
            card.device_started = true;
            card.driver_occupied_mebibytes = DRIVER_HOLDS_NOW;
            card.built_mebibytes = 5_000;
        }
        {
            let mut card = card.lock().expect("假卡不 panic");
            card.device_started = false;
            card.built_mebibytes = 0;
            card.left_by_a_lost_device_mebibytes = DRIVER_HOLDS_NOW + DRIVER_HOLDS_NOW + 5_000;
        }
        let _lease_m = ledger.book_on_the_card(0, 1_000).expect("借得到 1 000");
        assert_eq!(
            ledger.prepare_the_driver_share_before_starting_the_device(0),
            Ok(()),
            "第二次重起：第一台还没放掉时又丢一次"
        );
        let _third_device = ledger.record_the_device_started_on_the_card(0);
        {
            let mut card = card.lock().expect("假卡不 panic");
            card.device_started = true;
            card.driver_occupied_mebibytes = DRIVER_HOLDS_NOW;
            card.left_by_a_lost_device_mebibytes = DRIVER_HOLDS_NOW + 5_000;
        }
        drop(first_device);
        let unbuilt_lease_m = 1_000;
        assert!(
            room_now(&ledger) <= room_truly_left(&card, unbuilt_lease_m),
            "第一台放掉了、第二台连 L 的缓冲还在卡上：照分不出算，不多给（{} 对 {}）",
            room_now(&ledger),
            room_truly_left(&card, unbuilt_lease_m)
        );
        card.lock()
            .expect("假卡不 panic")
            .left_by_a_lost_device_mebibytes = DRIVER_HOLDS_NOW;
        lease_l.take_in_the_wait_before_returning(
            &second_held_by_lease_l,
            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
        );
        drop(second_held_by_lease_l);
        drop(lease_l);
        assert!(
            room_now(&ledger) <= room_truly_left(&card, unbuilt_lease_m),
            "L 还了、第二台还被调用方拿着：不多给"
        );
        card.lock()
            .expect("假卡不 panic")
            .left_by_a_lost_device_mebibytes = 0;
        drop(second_device);
        assert_eq!(
            room_now(&ledger),
            room_truly_left(&card, unbuilt_lease_m),
            "丢了的两台都放掉了：照读数分，能借的是卡上真能再给的"
        );
    }

    /// 第 10z 批第八块（第 10z 批第六块代码轮正推探针 P3、攻方 H5 的 X1 那一形：重起时没等成的借据要一起出账，不然这张卡到下一次丢设备都少给）：
    /// C 在设备上没等成、照旧记着（缓冲还在卡上）；设备丢了、重起：C 这时销账；旧设备连 C 的缓冲放掉之后，能借的就是卡上真能再给的。
    #[test]
    fn a_lease_kept_after_a_failed_wait_leaves_the_books_at_the_restart_and_the_card_is_not_left_short(
    ) {
        let (card, ledger) = ledger_on_a_card_under_the_ledger();
        let (lease, lost_device) = ten_thousand_built_with_the_device_held(&card, &ledger);
        let kept_lease =
            a_lease_built_and_dropped_after_a_timed_out_wait(&card, &ledger, &lost_device, 4_000);
        {
            let mut card = card.lock().expect("假卡不 panic");
            card.device_started = false;
            card.built_mebibytes = 0;
            card.left_by_a_lost_device_mebibytes = DRIVER + 10_000 + 4_000;
        }
        let mut lease_starting_the_replacement =
            ledger.book_on_the_card(0, 1_000).expect("借得到 1 000");
        assert_eq!(
            ledger.prepare_the_driver_share_before_starting_the_device(0),
            Ok(())
        );
        assert!(
            !ledger.is_the_lease_on_the_books(0, kept_lease),
            "设备丢了、账照重起记：没等成的那一笔这时销账"
        );
        let replacement = ledger.record_the_device_started_on_the_card(0);
        {
            let mut card = card.lock().expect("假卡不 panic");
            card.device_started = true;
            card.driver_occupied_mebibytes = DRIVER;
            card.left_by_a_lost_device_mebibytes = 0;
        }
        drop(lost_device);
        drop(lease);
        assert_eq!(
            room_now(&ledger),
            room_truly_left(&card, 1_000),
            "旧设备放掉了：能借的是卡上真能再给的，没有一笔卡在账上"
        );
        lease_starting_the_replacement.take_in_the_wait_before_returning(
            &replacement,
            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
        );
    }

    /// 借据丢掉时那几条用例的等法：`true` 是等完了、`false` 是没等成；等的时候 panic 了也是没等成。
    fn wait_of_a_fake_device(
        polled: &std::thread::Result<bool>,
    ) -> DeviceWaitBeforeReturningTheLease {
        if matches!(polled, Ok(true)) {
            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished
        } else {
            DeviceWaitBeforeReturningTheLease::TheWaitFailed
        }
    }

    /// 第 10z 批第八块（第 10z 批第六块代码轮判决 K4，攻方 H6、正推第 4 点：`GpuCardLease::drop` 记没记结局没有不要卡的测试）：
    /// 借据丢掉时那一步没等成，打一行、这一笔照旧记着；等完了不打、照常销账，之前没等成的那一笔也一起销。
    #[test]
    fn the_wait_before_returning_a_lease_reports_a_failed_wait_and_takes_in_either_outcome() {
        let readings = replaceable_readings(vec![reading_of_an_rtx_5090(30_000)]);
        let ledger = ledger_reading(&readings);
        let mut timed_out = ledger.book_on_the_card(0, 1_000).expect("空卡借得到");
        let timed_out_identifier = timed_out.lease_identifier();
        let device = ledger.record_the_device_started_on_the_card(0);
        let mut finished = ledger.book_on_the_card(0, 1_000).expect("借得到");
        let finished_identifier = finished.lease_identifier();
        let reports = std::cell::Cell::new(0);
        wait_for_the_device_and_take_in_the_outcome(
            &mut timed_out,
            &device,
            || false,
            wait_of_a_fake_device,
            |_polled| reports.set(reports.get() + 1),
        );
        drop(timed_out);
        assert_eq!(reports.get(), 1, "没等成：打一行");
        assert!(
            ledger.is_the_lease_on_the_books(0, timed_out_identifier),
            "没等成：结局记进了这一笔，丢掉之后照旧记着"
        );
        wait_for_the_device_and_take_in_the_outcome(
            &mut finished,
            &device,
            || true,
            wait_of_a_fake_device,
            |_polled| reports.set(reports.get() + 1),
        );
        drop(finished);
        assert_eq!(reports.get(), 1, "等完了：不打");
        assert!(!ledger.is_the_lease_on_the_books(0, finished_identifier));
        assert!(
            !ledger.is_the_lease_on_the_books(0, timed_out_identifier),
            "同一台上等完了：之前没等成的那一笔一起销"
        );
    }

    /// 第 10z 批第八块（第 10z 批第六块代码轮判决 K4，攻方 H6：`catch_unwind` 拿掉照绿）：不在展开中，等的时候 wgpu panic 了，
    /// 这一笔记成没等成（丢掉之后照旧记着），那个 panic 照样往外抛。
    #[test]
    fn a_panic_while_waiting_before_returning_a_lease_is_taken_as_a_failed_wait_and_goes_on() {
        let readings = replaceable_readings(vec![reading_of_an_rtx_5090(30_000)]);
        let ledger = ledger_reading(&readings);
        let mut lease = ledger.book_on_the_card(0, 1_000).expect("空卡借得到");
        let lease_identifier = lease.lease_identifier();
        let device = ledger.record_the_device_started_on_the_card(0);
        let unwound = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            wait_for_the_device_and_take_in_the_outcome(
                &mut lease,
                &device,
                || -> bool { panic!("等的时候 wgpu panic 了") },
                wait_of_a_fake_device,
                |_polled| {},
            );
        }));
        let panic = unwound.expect_err("不在展开中：等的时候的 panic 照样往外抛");
        assert_eq!(
            panic.downcast_ref::<&str>(),
            Some(&"等的时候 wgpu panic 了"),
            "抛出去的是等的时候那一个"
        );
        drop(lease);
        assert!(
            ledger.is_the_lease_on_the_books(0, lease_identifier),
            "等的时候 panic 了：记成没等成，丢掉之后照旧记着"
        );
    }

    /// 子进程跑 [`a_lease_dropped_while_its_thread_unwinds_takes_in_a_panicking_wait_without_panicking_again`] 那一形时设的变量。
    const LEASE_DROPPED_WHILE_UNWINDING_CHILD_VARIABLE: &str =
        "SINGLEFS_HARNESS_LEASE_DROPPED_WHILE_UNWINDING_CHILD";

    /// 第 10z 批第八块（第 10z 批第六块代码轮判决 K1-1 攻方 H3，改法 F-H3，照攻方 `unwind_in_drop.rs` 写，不要卡）：用例本身 panic 了，
    /// 借据在展开中丢掉，等的时候 wgpu 又 panic：只记没等成、不再抛（展开中再抛整个进程中止）。在子进程里跑那一形（中止的是子进程，
    /// 这条用例照常判红），子进程要正常退出、这一笔记成没等成。
    #[test]
    fn a_lease_dropped_while_its_thread_unwinds_takes_in_a_panicking_wait_without_panicking_again()
    {
        const TEST_FUNCTION: &str =
            "a_lease_dropped_while_its_thread_unwinds_takes_in_a_panicking_wait_without_panicking_again";
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
        let test_name = format!(
            "{}::{TEST_FUNCTION}",
            module_path!()
                .split_once("::")
                .expect("模块路径以包名起头")
                .1
        );
        let child = std::process::Command::new(std::env::current_exe().expect("测试二进制的路径"))
            .args([
                test_name.as_str(),
                "--exact",
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

    /// 第 10z 批第三块（第 10z 批代码轮正推 N1）：同一条线程上活里又起一段活，里面那段丢掉之后，这条线程上记的 Out of Memory 算回外面那段。
    #[test]
    fn a_work_begun_inside_another_hands_the_thread_back_to_the_outer_work_when_it_ends() {
        let condition = DeviceConditionOfACard::default();
        let outer_work = WorkOnTheCardOfThisThread::begin();
        let reports_before = condition.out_of_memory_reports();
        drop(WorkOnTheCardOfThisThread::begin());
        condition.record_an_out_of_memory("里面那段收完之后外面那段撞满".to_string());
        assert_eq!(
            condition.out_of_memory_explaining_a_failure_of_the_work(
                outer_work.identifier(),
                reports_before,
                DeviceWhenTheWorkReturned::NotLost,
            ),
            Some("里面那段收完之后外面那段撞满".to_string())
        );
    }

    /// 第 10z 批第三块（第 10z 批代码轮正推 N2、N3）：同一段活在这个设备上同时两处编内核，里面那一处编完、外面那一处还在编时，
    /// 没有活的线程上记的 Out of Memory 照样算这段活的。
    #[test]
    fn a_work_compiling_in_two_places_at_once_stays_compiling_until_both_end() {
        let condition = DeviceConditionOfACard::default();
        let compiling_work = WorkOnTheCardOfThisThread::begin();
        let reports_before = condition.out_of_memory_reports();
        condition.compile_for_the_work_of_this_thread(|| {
            condition.compile_for_the_work_of_this_thread(|| {});
            std::thread::scope(|scope| {
                scope.spawn(|| {
                    condition.record_an_out_of_memory("外面那一处还在编时没接住的".to_string());
                });
            });
        });
        assert_eq!(
            condition.out_of_memory_explaining_a_failure_of_the_work(
                compiling_work.identifier(),
                reports_before,
                DeviceWhenTheWorkReturned::NotLost,
            ),
            Some("外面那一处还在编时没接住的".to_string())
        );
    }

    /// 第 10z 批第三块（第 10z 批代码轮正推 K3 第 3 条）：记了丢失的设备是丢了的，没记的不是（卡池照它判留着的设备要不要重起）；
    /// 一段活交回那一刻照它记设备丢没丢。
    #[test]
    fn a_device_recorded_lost_is_lost_and_another_is_not() {
        let condition = DeviceConditionOfACard::default();
        assert!(!condition.is_lost(), "没记丢失");
        assert_eq!(
            condition.device_when_the_work_returns(),
            DeviceWhenTheWorkReturned::NotLost
        );
        condition
            .record_the_device_lost("Unknown：驱动重置".to_string(), CauseOfTheDeviceLoss::Other);
        assert!(condition.is_lost(), "记了丢失");
        assert_eq!(
            condition.device_when_the_work_returns(),
            DeviceWhenTheWorkReturned::Lost
        );
    }

    /// 第 10z 批第三块（第 10z 批代码轮判决 K1-3 攻方）：一块的两段在同一个设备上并发，一段因设备丢了（wgpu 说设备丢了，不是它的显存不够）
    /// 先失败、先来认；另一段范围里接到的 Out of Memory 要等那一段先收完才记进设备。先失败的那一段等两段都报到再认：认得出设备丢之前报过
    /// Out of Memory，不判红。
    #[test]
    fn a_piece_failing_on_a_lost_device_is_told_after_another_piece_records_its_out_of_memory() {
        let condition = &DeviceConditionOfACard::default();
        let together = &WorksEndingTogetherOnADevice::new();
        let failed_piece_ending = together.join();
        let caught_piece_ending = together.join();
        let explained = std::thread::scope(|scope| {
            let failed_piece = scope.spawn(move || {
                let work = WorkOnTheCardOfThisThread::begin();
                let reports_before = condition.out_of_memory_reports();
                condition.record_the_device_lost(
                    "Unknown：Device is lost".to_string(),
                    CauseOfTheDeviceLoss::Other,
                );
                condition.end_the_work_and_explain_its_failure(
                    &work,
                    reports_before,
                    condition.device_when_the_work_returns(),
                    None,
                    failed_piece_ending,
                )
            });
            scope.spawn(move || {
                let work = WorkOnTheCardOfThisThread::begin();
                let reports_before = condition.out_of_memory_reports();
                let waiting_since = std::time::Instant::now();
                while together.works_not_yet_ended() > 1 {
                    assert!(
                        waiting_since.elapsed() < std::time::Duration::from_secs(60),
                        "先失败的那一段一分钟都没报到"
                    );
                    std::thread::yield_now();
                }
                condition.end_the_work_and_explain_its_failure(
                    &work,
                    reports_before,
                    condition.device_when_the_work_returns(),
                    Some("序号 0 的卡单元校验和对拍时报了 Out of Memory".to_string()),
                    caught_piece_ending,
                )
            });
            failed_piece.join().expect("先失败的那一段不 panic")
        });
        assert!(
            explained
                .as_deref()
                .is_some_and(|reason| reason.starts_with("设备丢了")),
            "等另一段记进设备之后再认：说成设备丢之前报过 Out of Memory，{explained:?}"
        );
    }

    /// wgpu 因显存不够丢设备时给的那一句：`wgpu_hal::DeviceError::OutOfMemory` 说的（wgpu-hal 30.0.1 `src/lib.rs` 第 382 行），
    /// 丢失回调照 `{reason:?}：{message}` 记（`singlefs_checker_tier::gpu_unit_checks`）。
    const LOST_FOR_LACK_OF_MEMORY: &str = "Unknown：Out of memory";

    /// 第 10z 批第五块（第 10z 批第三块代码轮判决 K1-4 攻方甲形，`attack_10zc_endings.rs` 的 `cross-group`）：同一个设备上不在同一组的两段活
    /// （同一测试进程里两条用例共用卡池的设备）：X 的范围里接到 Out of Memory、wgpu 因它丢了设备，X 还没收完；Y 因设备丢了先失败、先收完，
    /// 等的只是它自己。设备是因显存不够丢的，Y 认成显存不够、不判红；X 收完认它自己那一句。同组对照照旧认得出。
    #[test]
    fn a_work_failing_on_a_device_another_group_lost_for_lack_of_memory_is_told_out_of_memory() {
        for same_group in [false, true] {
            let condition = &DeviceConditionOfACard::default();
            let shared = &WorksEndingTogetherOnADevice::new();
            let alone_x = &WorksEndingTogetherOnADevice::new();
            let alone_y = &WorksEndingTogetherOnADevice::new();
            let (ending_x, ending_y) = if same_group {
                (shared.join(), shared.join())
            } else {
                (alone_x.join(), alone_y.join())
            };
            let (y_began_sender, y_began) = std::sync::mpsc::channel::<()>();
            let (y_ended_sender, y_ended) = std::sync::mpsc::channel::<()>();
            let (x_told, y_told) = std::thread::scope(|scope| {
                let x = scope.spawn(move || {
                    let work = WorkOnTheCardOfThisThread::begin();
                    let reports_before = condition.out_of_memory_reports();
                    y_began
                        .recv_timeout(std::time::Duration::from_secs(60))
                        .expect("Y 一分钟之内开了活");
                    condition.record_the_device_lost(
                        LOST_FOR_LACK_OF_MEMORY.to_string(),
                        CauseOfTheDeviceLoss::GpuMemoryShort,
                    );
                    let device_when_the_work_returned = condition.device_when_the_work_returns();
                    if !same_group {
                        // 不在同一组：Y 先收完，X 收得比 Y 晚
                        y_ended
                            .recv_timeout(std::time::Duration::from_secs(60))
                            .expect("Y 一分钟之内收完");
                    }
                    condition.end_the_work_and_explain_its_failure(
                        &work,
                        reports_before,
                        device_when_the_work_returned,
                        Some("序号 0 的卡判一块时报了 Out of Memory".to_string()),
                        ending_x,
                    )
                });
                let y = scope.spawn(move || {
                    let work = WorkOnTheCardOfThisThread::begin();
                    let reports_before = condition.out_of_memory_reports();
                    y_began_sender.send(()).expect("X 在等 Y 开活");
                    let waiting_since = std::time::Instant::now();
                    while !condition.is_lost() {
                        assert!(
                            waiting_since.elapsed() < std::time::Duration::from_secs(60),
                            "X 一分钟都没丢设备"
                        );
                        std::thread::yield_now();
                    }
                    let explained = condition.end_the_work_and_explain_its_failure(
                        &work,
                        reports_before,
                        condition.device_when_the_work_returns(),
                        None,
                        ending_y,
                    );
                    // 同组时 X 不等这一句，发不出去也无妨
                    let _ = y_ended_sender.send(());
                    explained
                });
                (x.join().expect("X 不 panic"), y.join().expect("Y 不 panic"))
            });
            assert_eq!(
                x_told,
                Some("序号 0 的卡判一块时报了 Out of Memory".to_string()),
                "同组 {same_group}：X 认它自己那一句"
            );
            assert!(
                y_told.is_some(),
                "同组 {same_group}：设备因显存不够丢了，Y 认成显存不够、不判红"
            );
        }
    }

    /// 第 10z 批第五块（第 10z 批第三块代码轮判决 K1-4 攻方乙形 `genuine-panic-widened`、正推 `explain_demo.rs`）：一块两段 A、B 同组。
    /// A 的活自己真 panic 了，交回那一刻设备没丢、设备上一条 Out of Memory 都没有；A 报到之后 B 的范围里接到 Out of Memory（设备丢没丢两样都试），
    /// B 收完把它记进设备。A 等齐再认：只认算在它自己头上的，说不成显存不够（照样往外抛、判红）；B 认得出它自己那一句。
    #[test]
    fn a_genuine_panic_while_the_device_was_there_is_not_told_out_of_memory_after_another_piece_runs_out(
    ) {
        for b_loses_the_device in [false, true] {
            let condition = &DeviceConditionOfACard::default();
            let together = &WorksEndingTogetherOnADevice::new();
            let ending_a = together.join();
            let ending_b = together.join();
            let (a_told, b_told) = std::thread::scope(|scope| {
                let a = scope.spawn(move || {
                    let work = WorkOnTheCardOfThisThread::begin();
                    let reports_before = condition.out_of_memory_reports();
                    // A 的活 panic 了（真毛病）：交回这一刻设备没丢
                    let device_when_the_work_returned = condition.device_when_the_work_returns();
                    condition.end_the_work_and_explain_its_failure(
                        &work,
                        reports_before,
                        device_when_the_work_returned,
                        None,
                        ending_a,
                    )
                });
                let b = scope.spawn(move || {
                    let work = WorkOnTheCardOfThisThread::begin();
                    let reports_before = condition.out_of_memory_reports();
                    let waiting_since = std::time::Instant::now();
                    while together.works_not_yet_ended() > 1 {
                        assert!(
                            waiting_since.elapsed() < std::time::Duration::from_secs(60),
                            "A 一分钟都没报到"
                        );
                        std::thread::yield_now();
                    }
                    if b_loses_the_device {
                        condition.record_the_device_lost(
                            LOST_FOR_LACK_OF_MEMORY.to_string(),
                            CauseOfTheDeviceLoss::GpuMemoryShort,
                        );
                    }
                    condition.end_the_work_and_explain_its_failure(
                        &work,
                        reports_before,
                        condition.device_when_the_work_returns(),
                        Some("序号 0 的卡单元校验和对拍时报了 Out of Memory".to_string()),
                        ending_b,
                    )
                });
                (a.join().expect("A 不 panic"), b.join().expect("B 不 panic"))
            });
            assert_eq!(
                a_told, None,
                "B 丢设备 {b_loses_the_device}：A 交回那一刻设备没丢，它的 panic 说不成显存不够"
            );
            assert_eq!(
                b_told,
                Some("序号 0 的卡单元校验和对拍时报了 Out of Memory".to_string()),
                "B 丢设备 {b_loses_the_device}：B 认它自己那一句"
            );
        }
    }

    /// 第 10z 批第五块（第 10z 批第三块代码轮判决 K3 攻方 `record-the-caught-after-waiting`）：一段范围里接到的 Out of Memory 先记进设备再报到：
    /// 它报到的那一刻（还在等同组别的段）设备上已经记着，先收完的别的段这时来认就认得出。改成等完之后再记，报到的那一刻设备上一条都没有。
    #[test]
    fn a_piece_records_what_its_scope_caught_before_it_reports_its_ending() {
        let condition: &'static DeviceConditionOfACard =
            Box::leak(Box::new(DeviceConditionOfACard::default()));
        let together: &'static WorksEndingTogetherOnADevice =
            Box::leak(Box::new(WorksEndingTogetherOnADevice::new()));
        let held_ending = together.join();
        let caught_piece_ending = together.join();
        let caught_piece = std::thread::spawn(move || {
            let work = WorkOnTheCardOfThisThread::begin();
            let reports_before = condition.out_of_memory_reports();
            condition.end_the_work_and_explain_its_failure(
                &work,
                reports_before,
                condition.device_when_the_work_returns(),
                Some("序号 0 的卡单元校验和对拍时报了 Out of Memory".to_string()),
                caught_piece_ending,
            )
        });
        let waiting_since = std::time::Instant::now();
        while together.works_not_yet_ended() > 1 {
            assert!(
                waiting_since.elapsed() < std::time::Duration::from_secs(60),
                "接到 Out of Memory 的那一段一分钟都没报到"
            );
            std::thread::yield_now();
        }
        let reports_when_it_reported = condition.out_of_memory_reports();
        drop(held_ending);
        let explained = caught_piece.join().expect("那一段不 panic");
        assert_eq!(
            reports_when_it_reported, 1,
            "报到的那一刻范围里接到的已经记进设备"
        );
        assert_eq!(
            explained,
            Some("序号 0 的卡单元校验和对拍时报了 Out of Memory".to_string())
        );
    }

    /// 第 10z 批第五块（第 10z 批第三块代码轮判决 K3 攻方 `endings-notify-one`）：一组三段，两段先报到、在等；最后一段报到时叫醒等着的每一段，
    /// 两段都在一分钟之内等完（一块的段数是核对线程数，本机 32）。只叫醒一段时另一段一直等（这条用例一分钟之后判红）。
    #[test]
    fn the_last_piece_to_report_wakes_every_piece_waiting_for_it() {
        let together: &'static WorksEndingTogetherOnADevice =
            Box::leak(Box::new(WorksEndingTogetherOnADevice::new()));
        let waiting_endings = [together.join(), together.join()];
        let last_ending = together.join();
        let (ended_sender, ended_receiver) = std::sync::mpsc::channel();
        for ending in waiting_endings {
            let ended_sender = ended_sender.clone();
            std::thread::spawn(move || {
                ending.end_and_wait_for_the_others();
                let _ = ended_sender.send(());
            });
        }
        drop(ended_sender);
        let waiting_since = std::time::Instant::now();
        while together.works_not_yet_ended() > 1 {
            assert!(
                waiting_since.elapsed() < std::time::Duration::from_secs(60),
                "先报到的两段一分钟都没报到"
            );
            std::thread::yield_now();
        }
        // 报到之后到进等待之间还有几条指令：给那两段时间都等上
        std::thread::sleep(std::time::Duration::from_millis(200));
        last_ending.end_and_wait_for_the_others();
        for waiting_piece in 0..2 {
            assert_eq!(
                ended_receiver.recv_timeout(std::time::Duration::from_secs(60)),
                Ok(()),
                "等着的第 {waiting_piece} 段一分钟之内被叫醒、等完"
            );
        }
    }

    /// 第 10z 批第三块：一段领了报到、没报到就丢掉（那一段 panic 了），算它报到了，别的段不等它。
    #[test]
    fn a_piece_dropping_its_ending_without_ending_does_not_keep_the_others_waiting() {
        let together: &'static WorksEndingTogetherOnADevice =
            Box::leak(Box::new(WorksEndingTogetherOnADevice::new()));
        let dropped_ending = together.join();
        let waiting_ending = together.join();
        let (ended_sender, ended_receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            waiting_ending.end_and_wait_for_the_others();
            let _ = ended_sender.send(());
        });
        drop(dropped_ending);
        assert_eq!(
            ended_receiver.recv_timeout(std::time::Duration::from_secs(60)),
            Ok(()),
            "丢掉的那一份算报到：另一段一分钟之内等完"
        );
        assert_eq!(together.works_not_yet_ended(), 0);
    }

    /// 第 10z 批（第 10y 批代码轮判决 K2-2 P2）：几张卡一起领的那一队块，报了 Out of Memory 的卡交回的那一块先给下一张来领的卡；
    /// 交回的都领走、队里也领完才算没有剩下的。
    #[test]
    fn a_block_given_back_by_a_card_out_of_memory_goes_to_the_next_card_that_takes_one() {
        let go_on = StopOfTheCards::NotRequested;
        let blocks = BlocksForTheCards::new(0..3);
        assert_eq!(blocks.take_a_block(go_on), Some(0));
        assert_eq!(blocks.take_a_block(go_on), Some(1));
        blocks.give_back(0);
        assert!(blocks.any_left());
        assert_eq!(blocks.take_a_block(go_on), Some(0), "交回的先领");
        assert_eq!(blocks.take_a_block(go_on), Some(2));
        assert_eq!(blocks.take_a_block(go_on), None);
        assert!(!blocks.any_left());
        blocks.give_back(1);
        assert!(blocks.any_left(), "队里领完了、还有交回的");
        assert_eq!(blocks.blocks_not_taken(), vec![1]);
        assert_eq!(blocks.take_a_block(go_on), Some(1));
        assert_eq!(blocks.take_a_block(go_on), None);
    }

    /// 第 10z 批第五块（第 10z 批第三块代码轮判决 K1-5、K3 攻方 `stop-checked-after-taking-a-block`、正推 M18）：调用方叫停之后卡只领判完的块
    /// 在等的那几块（比判完的块号小、交回队里的），别的留在队里给下一趟；一块都没判完时一块都不领、队里原样；判红了连判完的块在等的也不领；
    /// 下一轮的那一队记着判完的块号，叫停了照样领判完的块在等的。
    #[test]
    fn a_card_asked_to_stop_takes_only_the_blocks_judged_blocks_wait_for_and_leaves_the_rest() {
        let blocks = BlocksForTheCards::new(0..5);
        let mut judged_while_stopped = Vec::new();
        let left_while_stopped = judge_blocks_on_one_card(
            &blocks,
            &|| StopOfTheCards::RequestedByTheCaller,
            &mut |pending_index| {
                judged_while_stopped.push(pending_index);
                BlockJudgedOnACard::Judged(pending_index)
            },
            &mut |_pending_index, _judged| BlockHandedToTheStore::Handed,
        );
        assert_eq!(left_while_stopped, CardLeftTheBlocks::NothingLeftOrStopped);
        assert!(
            judged_while_stopped.is_empty(),
            "叫停了、没有判完的块在等：一块都不领"
        );
        assert_eq!(
            blocks.blocks_not_taken(),
            vec![0, 1, 2, 3, 4],
            "叫停时没领的块原样留在队里"
        );
        // 第一张卡领了块 0；第二张判完块 1、2 之后被叫停
        assert_eq!(blocks.take_a_block(StopOfTheCards::NotRequested), Some(0));
        let judged_by_the_second_card = std::cell::RefCell::new(Vec::new());
        let left_by_the_second_card = judge_blocks_on_one_card(
            &blocks,
            &|| {
                if judged_by_the_second_card.borrow().len() >= 2 {
                    StopOfTheCards::RequestedByTheCaller
                } else {
                    StopOfTheCards::NotRequested
                }
            },
            &mut |pending_index| {
                judged_by_the_second_card.borrow_mut().push(pending_index);
                BlockJudgedOnACard::Judged(pending_index)
            },
            &mut |_pending_index, _judged| BlockHandedToTheStore::Handed,
        );
        assert_eq!(
            left_by_the_second_card,
            CardLeftTheBlocks::NothingLeftOrStopped
        );
        assert_eq!(
            judged_by_the_second_card.into_inner(),
            vec![1, 2],
            "叫停之后块 3 比判完的块 2 大：不领"
        );
        // 第一张卡在块 0 上报了 Out of Memory、交回
        blocks.give_back(0);
        assert!(
            blocks.any_left_to_take(StopOfTheCards::RequestedByTheCaller),
            "交回的块 0 比判完的块 2 小：叫停了也领得"
        );
        assert!(
            !blocks.any_left_to_take(StopOfTheCards::TheRunFailed),
            "判红了一块都不领"
        );
        assert_eq!(blocks.take_a_block(StopOfTheCards::TheRunFailed), None);
        let next_round = blocks.left_for_another_round();
        assert_eq!(next_round.blocks_not_taken(), vec![0, 3, 4]);
        assert_eq!(
            next_round.take_a_block(StopOfTheCards::RequestedByTheCaller),
            Some(0),
            "下一轮的那一队记着判完的块 2：叫停了照样领块 0"
        );
        assert_eq!(
            next_round.take_a_block(StopOfTheCards::RequestedByTheCaller),
            None,
            "块 3 留给下一趟"
        );
        assert_eq!(next_round.blocks_not_taken(), vec![3, 4]);
    }

    /// 第 10z 批第五块（第 10z 批第三块代码轮正推 M30、M31）：判不了一块的卡收工、那一块不交回，不再领；判完的一块交不到写库那一头，
    /// 这张卡收工、不再领。
    #[test]
    fn a_card_stops_taking_blocks_after_a_refusal_and_after_the_store_is_gone() {
        let blocks_of_the_refusing_card = BlocksForTheCards::new(0..5);
        let mut taken = Vec::new();
        let left_by_the_refusing_card = judge_blocks_on_one_card(
            &blocks_of_the_refusing_card,
            &|| StopOfTheCards::NotRequested,
            &mut |pending_index| {
                taken.push(pending_index);
                if pending_index == 1 {
                    BlockJudgedOnACard::<usize>::Refused {
                        reason: "判不了块 1".to_string(),
                    }
                } else {
                    BlockJudgedOnACard::Judged(pending_index)
                }
            },
            &mut |_pending_index, _judged| BlockHandedToTheStore::Handed,
        );
        assert_eq!(
            left_by_the_refusing_card,
            CardLeftTheBlocks::Refused {
                reason: "判不了块 1".to_string()
            }
        );
        assert_eq!(taken, vec![0, 1], "判不了的那一块之后不再领");
        assert_eq!(
            blocks_of_the_refusing_card.blocks_not_taken(),
            vec![2, 3, 4],
            "判不了的那一块不交回"
        );
        let blocks_of_the_card_without_a_store = BlocksForTheCards::new(0..5);
        let mut handed = 0;
        let left_without_a_store = judge_blocks_on_one_card(
            &blocks_of_the_card_without_a_store,
            &|| StopOfTheCards::NotRequested,
            &mut |pending_index| BlockJudgedOnACard::Judged(pending_index),
            &mut |_pending_index, _judged| {
                handed += 1;
                BlockHandedToTheStore::TheStoreIsGone
            },
        );
        assert_eq!(left_without_a_store, CardLeftTheBlocks::TheStoreIsGone);
        assert_eq!(handed, 1, "写库那一头走了：不再领");
    }

    /// 第 10z 批第三块：判完的块按块序写库，后面的先到就等着；写库失败之后到的不再写。
    #[test]
    fn judged_blocks_are_stored_in_block_order_and_none_after_a_storing_error() {
        let mut stored_in_order: BlocksStoredInOrder<usize> = BlocksStoredInOrder::new();
        let mut stored = Vec::new();
        for (pending_index, arrived_block) in [(2, 2), (0, 0), (1, 1), (3, 3)] {
            stored_in_order.arrive(pending_index, arrived_block, &mut |block| {
                stored.push(block);
                Ok(())
            });
        }
        assert_eq!(stored, vec![0, 1, 2, 3], "按块序写");
        assert_eq!(stored_in_order.blocks_stored(), 4);
        assert_eq!(stored_in_order.blocks_waiting(), 0);
        let mut failing: BlocksStoredInOrder<usize> = BlocksStoredInOrder::new();
        for (pending_index, arrived_block) in [(1, 1), (0, 0), (2, 2)] {
            failing.arrive(pending_index, arrived_block, &mut |block| match block {
                1 => Err("写判定块失败".to_string()),
                _ => Ok(()),
            });
        }
        assert_eq!(failing.blocks_stored(), 1, "块 1 写失败：只写进了块 0");
        assert_eq!(failing.storing_error(), Some("写判定块失败"));
    }

    /// 第 10z 批第三块、第五块：一轮卡收工之后这一趟怎么走，逐种（参数：没写的块、其中判完在等的、这一轮报了的卡、叫没叫停、没报过的卡）。
    #[test]
    fn after_a_round_another_round_starts_only_when_a_card_ran_out_and_another_card_is_left() {
        assert_eq!(
            after_a_round_of_the_cards(0, 0, 1, StopOfTheRun::NotRequested, 1),
            AfterARoundOfTheCards::EveryBlockStored,
            "有卡报了而别的卡判完了：照常交回"
        );
        assert_eq!(
            after_a_round_of_the_cards(3, 0, 0, StopOfTheRun::Requested, 2),
            AfterARoundOfTheCards::StoppedOnRequest
        );
        assert_eq!(
            after_a_round_of_the_cards(3, 0, 1, StopOfTheRun::NotRequested, 1),
            AfterARoundOfTheCards::AnotherRoundWithoutTheCardsShortOfMemory
        );
        assert_eq!(
            after_a_round_of_the_cards(3, 2, 1, StopOfTheRun::NotRequested, 1),
            AfterARoundOfTheCards::AnotherRoundWithoutTheCardsShortOfMemory,
            "没叫停：等着的块随剩下的一起判"
        );
        assert_eq!(
            after_a_round_of_the_cards(3, 0, 1, StopOfTheRun::NotRequested, 0),
            AfterARoundOfTheCards::NotRunTheGpuMemoryWasShort,
            "报过的卡之外没有卡了"
        );
        assert_eq!(
            after_a_round_of_the_cards(3, 0, 1, StopOfTheRun::Requested, 1),
            AfterARoundOfTheCards::StoppedOnRequest,
            "叫停了、判完的块都写进了库：照叫停收尾，不再起一轮"
        );
        assert_eq!(
            after_a_round_of_the_cards(3, 2, 1, StopOfTheRun::Requested, 1),
            AfterARoundOfTheCards::AnotherRoundOnlyForTheBlocksTheJudgedBlocksWaitFor,
            "叫停了、判完的块在等交回的那一块：再起一轮只判它"
        );
        assert_eq!(
            after_a_round_of_the_cards(3, 2, 1, StopOfTheRun::Requested, 0),
            AfterARoundOfTheCards::NotRunTheGpuMemoryWasShort,
            "叫停了、判完的块在等而没有卡能判了"
        );
    }

    /// 第 10z 批第五块（第 10z 批第三块代码轮判决 K1-5 攻方乙、正推第 4 点）：几轮之间带着的那几样：报了 Out of Memory 的卡记下、下一轮不再用，
    /// 下一轮只判这一轮没领走的，头一句 Out of Memory 留着；一张卡两轮各收工一次只算一张。
    #[test]
    fn rounds_keep_the_cards_short_of_memory_out_and_count_a_card_finishing_twice_once() {
        let mut rounds: RoundsOfTheCards<usize> = RoundsOfTheCards::new(2);
        assert_eq!(rounds.blocks_of_the_next_round(), vec![0, 1]);
        let blocks_of_the_first_round = rounds.take_the_blocks_of_the_round();
        // 头一轮：卡 0 领了块 0、报了 Out of Memory 交回；卡 1 判完块 1（等着块 0）收工
        assert_eq!(
            blocks_of_the_first_round.take_a_block(StopOfTheCards::NotRequested),
            Some(0)
        );
        assert_eq!(
            blocks_of_the_first_round.take_a_block(StopOfTheCards::NotRequested),
            Some(1)
        );
        blocks_of_the_first_round.record_a_judged_block(1);
        rounds.stored_in_order().arrive(1, 1, &mut |_block| Ok(()));
        blocks_of_the_first_round.give_back(0);
        assert_eq!(
            rounds.after_the_round(
                &blocks_of_the_first_round,
                &[0, 1],
                [1],
                &[(0, "显存不够：卡 0 判块 0".to_string())],
                StopOfTheRun::NotRequested,
            ),
            AfterARoundOfTheCards::AnotherRoundWithoutTheCardsShortOfMemory
        );
        assert!(rounds.is_short_of_memory(0), "报了的卡 0 不再用");
        assert!(!rounds.is_short_of_memory(1));
        assert_eq!(rounds.first_out_of_memory(), Some("显存不够：卡 0 判块 0"));
        assert_eq!(
            rounds.blocks_of_the_next_round(),
            vec![0],
            "下一轮只判没领走的块 0，判完的块 1 不再判"
        );
        let blocks_of_the_second_round = rounds.take_the_blocks_of_the_round();
        assert_eq!(
            blocks_of_the_second_round.take_a_block(StopOfTheCards::NotRequested),
            Some(0)
        );
        rounds.stored_in_order().arrive(0, 0, &mut |_block| Ok(()));
        assert_eq!(rounds.blocks_stored(), 2);
        assert_eq!(
            rounds.after_the_round(
                &blocks_of_the_second_round,
                &[1],
                [1],
                &[],
                StopOfTheRun::NotRequested
            ),
            AfterARoundOfTheCards::EveryBlockStored
        );
        assert_eq!(rounds.cards_used(), 1, "卡 1 两轮各收工一次只算一张");
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

    /// 第 10z 批第五块（第 10z 批第三块代码轮正推 M01，`probes/killers.rs`；第 10z 批第六块 G2 乙改数）：长借据建满、等完设备还了
    /// （卡上还给驱动），新借一笔建满：能借的照读数算，新借的建上的只扣一次，还了的那一笔不再扣（甲在这里多扣驱动那一份，21 990）。
    #[test]
    fn a_lease_booked_after_another_was_returned_counts_its_built_buffers_once() {
        let (card, ledger) = ledger_on_a_card_under_the_ledger();
        let mut first = ledger.book_on_the_card(0, 10_000).expect("空卡借得到");
        {
            let mut card = card.lock().expect("假卡不 panic");
            card.device_started = true;
            card.driver_occupied_mebibytes = DRIVER;
            card.built_mebibytes = 10_000;
        }
        let device = ledger.record_the_device_started_on_the_card(0);
        first.take_in_the_wait_before_returning(
            &device,
            DeviceWaitBeforeReturningTheLease::TheSubmittedWorkFinished,
        );
        card.lock().expect("假卡不 panic").built_mebibytes = 0;
        drop(first);
        let _ = room_now(&ledger);
        let _second = ledger.book_on_the_card(0, 5_000).expect("借得到 5 000");
        card.lock().expect("假卡不 panic").built_mebibytes = 5_000;
        let room_after_the_second_is_built = room_now(&ledger);
        assert_eq!(
            room_after_the_second_is_built + 5_000 + DRIVER,
            AN_EMPTY_RTX_5090_HAS,
            "第二笔建满：能借的 {room_after_the_second_is_built} 加第二笔与驱动那一份是空卡"
        );
    }

    /// 第 10z 批第五块（第 10z 批第三块代码轮正推 M05）：设备丢了、重起被拒一次之后，再被问三次而卡上读数没变，能借的不变
    /// （被拒之后记成「丢了、还没重起」，不记回「起着」而每问一次多扣一份）。
    #[test]
    fn a_refused_restart_asked_again_takes_no_further_driver_share_off_the_room() {
        let (card, ledger) = ledger_on_a_card_under_the_ledger();
        let _lease = ten_thousand_built_on_a_started_device(&card, &ledger);
        // 新设备放不下：卡上 32 110 − 19 000 − 2 560 − 10 000 = 550，驱动那一份要 2 560
        the_device_is_lost_with_others_taking(&card, 19_000);
        assert_eq!(
            ledger.prepare_the_driver_share_before_starting_the_device(0),
            Err(BookingRefusal::NoCardHasRoomNow {
                need_mebibytes: DRIVER,
                card_rooms_mebibytes: vec![0],
            })
        );
        card.lock().expect("假卡不 panic").others_mebibytes = 0;
        let room_after_one_refusal = room_now(&ledger);
        for _ in 0..3 {
            let _ = ledger.prepare_the_driver_share_before_starting_the_device(0);
        }
        assert_eq!(
            room_now(&ledger),
            room_after_one_refusal,
            "再问三次、读数没变：能借的不变"
        );
    }

    /// 第 10z 批第五块（第 10z 批第三块代码轮正推 M13）：设备丢了、重起被拒之后又借了一笔，之后卡上又被占：再来起设备照此刻判、放不下就拒
    /// （借一笔不把驱动那一份记回「登记了、没起」而直接放行）。
    #[test]
    fn a_lease_booked_after_a_refused_restart_keeps_the_restart_judged_again() {
        let (card, ledger) = ledger_on_a_card_under_the_ledger();
        let _lease = ten_thousand_built_on_a_started_device(&card, &ledger);
        the_device_is_lost_with_others_taking(&card, 19_000);
        assert!(ledger
            .prepare_the_driver_share_before_starting_the_device(0)
            .is_err());
        card.lock().expect("假卡不 panic").others_mebibytes = 0;
        let _another = ledger
            .book_on_the_card(0, 5_000)
            .expect("别的进程走了，借得到");
        card.lock().expect("假卡不 panic").others_mebibytes = 19_000;
        assert!(
            ledger
                .prepare_the_driver_share_before_starting_the_device(0)
                .is_err(),
            "被拒之后借了一笔，再来起设备照此刻判：放不下"
        );
    }

    /// 第 10z 批第五块（第 10z 批第三块代码轮正推 M10；第 10z 批第六块 G2 乙改数）：设备丢了要重起，借着的 10 000 连新设备的驱动那一份
    /// 照此刻的空闲恰好放得下就起，差 1 MiB 就拒（丢了的设备与建上的 10 000 留在卡上、不从读数里分出来：空 32 110 − 6 990 − 12 560 = 12 560）。
    #[test]
    fn a_restart_is_allowed_when_it_fits_exactly_and_refused_one_mebibyte_short() {
        assert_eq!(
            AN_EMPTY_RTX_5090_HAS - 6_990 - (DRIVER + 10_000),
            DRIVER + 10_000
        );
        for (others_mebibytes, fits) in [(6_990u64, true), (6_991, false)] {
            let (card, ledger) = ledger_on_a_card_under_the_ledger();
            let _lease = ten_thousand_built_on_a_started_device(&card, &ledger);
            the_device_is_lost_with_others_taking(&card, others_mebibytes);
            assert_eq!(
                ledger
                    .prepare_the_driver_share_before_starting_the_device(0)
                    .is_ok(),
                fits,
                "别的进程占着 {others_mebibytes}"
            );
        }
    }

    /// P2 那几条用例的假卡按拍走的挂钟（次序照攻方模型 `research/prompts/m3-batch10z-a-code-r1-opus-model/p2_blocks_10z.rs`）：每张卡判一块占几拍；
    /// 挂钟只在没有一张卡在跑（都停在判块上、或收了工）时往前走，最先判完的那一张先往下，同一拍卡号大的先。起卡的那一方也算在跑，
    /// 卡一张一张起完它才让开，卡领第一块的次序因此照卡号。
    struct CardsOnAClockOfTicks {
        clock: Mutex<ClockOfTicks>,
        changed: std::sync::Condvar,
    }

    struct ClockOfTicks {
        now: u64,
        running: usize,
        judging_until: std::collections::BTreeSet<(u64, std::cmp::Reverse<usize>)>,
    }

    impl CardsOnAClockOfTicks {
        fn started_by_the_one_starting_the_cards() -> Self {
            Self {
                clock: Mutex::new(ClockOfTicks {
                    now: 0,
                    running: 1,
                    judging_until: std::collections::BTreeSet::new(),
                }),
                changed: std::sync::Condvar::new(),
            }
        }

        fn locked(&self) -> std::sync::MutexGuard<'_, ClockOfTicks> {
            self.clock
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
        }

        fn wait<'clock>(
            &self,
            clock: std::sync::MutexGuard<'clock, ClockOfTicks>,
        ) -> std::sync::MutexGuard<'clock, ClockOfTicks> {
            self.changed
                .wait(clock)
                .unwrap_or_else(std::sync::PoisonError::into_inner)
        }

        fn a_card_joins(&self) {
            self.locked().running += 1;
        }

        /// 起卡的那一方等它刚起的那张卡停在判块上或收了工。
        fn wait_until_only_the_one_starting_the_cards_runs(&self) {
            let mut clock = self.locked();
            while clock.running > 1 {
                clock = self.wait(clock);
            }
        }

        fn one_running_stops(&self) {
            let mut clock = self.locked();
            clock.running -= 1;
            self.changed.notify_all();
        }

        fn judge_for(&self, card: usize, ticks: u64) {
            let mut clock = self.locked();
            let until = (clock.now + ticks, std::cmp::Reverse(card));
            clock.judging_until.insert(until);
            clock.running -= 1;
            self.changed.notify_all();
            while !(clock.running == 0 && clock.judging_until.first() == Some(&until)) {
                clock = self.wait(clock);
            }
            clock.judging_until.pop_first();
            clock.now = until.0;
            clock.running += 1;
            self.changed.notify_all();
        }
    }

    /// 一张假卡的线程收了工（正常收或 panic 了）就不再算在跑。
    struct CardRunningOnTheClock<'clock>(&'clock CardsOnAClockOfTicks);

    impl Drop for CardRunningOnTheClock<'_> {
        fn drop(&mut self) {
            self.0.one_running_stops();
        }
    }

    /// 假卡那一趟里调用方什么时候叫停。
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum CallerStopOfTheFakeRun {
        Never,
        /// 几轮合计判完第 n 块（从 1 数）的那一刻。
        WhenTheBlocksJudgedReach(usize),
        /// 第 0 张卡报 Out of Memory 的那一刻（交回那一块之前）。
        WhenCard0RunsOutOfMemory,
        /// 第 r 轮（从 0 数）起卡之前。
        BeforeRound(usize),
    }

    /// 假卡上几轮判完一队块的结局：照块序写进库的块、每块判了几遍、最后一轮之后怎么走、留给下一趟的块、用到几张卡。
    struct BlocksJudgedOnFakeCards {
        stored: Vec<usize>,
        judged_times: Vec<usize>,
        after_the_last_round: AfterARoundOfTheCards,
        blocks_left_for_the_next_run: Vec<usize>,
        cards_used: usize,
    }

    /// 几张假卡几轮判 `pending_blocks` 块（照崩溃放量产品路径第 ① 段的走法：几轮之间带着的交 [`RoundsOfTheCards`]，一轮里卡一张一张起，
    /// 从一队块里照叫停领、判、交去按块序写库；一轮收工之后照 [`RoundsOfTheCards::after_the_round`] 走）：第 `card` 张判一块占
    /// `ticks_of_each_card[card]` 拍；头一轮只起前 `cards_holding_states_in_the_first_round` 张（装了状态的），再起的一轮起没报过 Out of Memory
    /// 的第一张（收尾那一步再起一张还有空的卡）；第 0 张判它领的第 `out_of_memory_at_block_of_card_0` 块（从 1 数）到末尾时报 Out of Memory，
    /// 报了之后再给它块就 panic（报了就收工）；调用方照 `caller_stop` 叫停。
    fn blocks_judged_on_fake_cards(
        pending_blocks: usize,
        ticks_of_each_card: &[u64],
        cards_holding_states_in_the_first_round: usize,
        out_of_memory_at_block_of_card_0: usize,
        caller_stop: CallerStopOfTheFakeRun,
    ) -> BlocksJudgedOnFakeCards {
        let card_index = |card: usize| u32::try_from(card).expect("假卡没几张");
        let mut rounds: RoundsOfTheCards<usize> = RoundsOfTheCards::new(pending_blocks);
        let stored = Mutex::new(Vec::new());
        let judged_times = Mutex::new(vec![0usize; pending_blocks]);
        let blocks_taken_by_card_0 = AtomicUsize::new(0);
        let blocks_judged = AtomicUsize::new(0);
        let stop_is_requested = std::sync::atomic::AtomicBool::new(false);
        let mut round = 0;
        loop {
            if caller_stop == CallerStopOfTheFakeRun::BeforeRound(round) {
                stop_is_requested.store(true, Ordering::SeqCst);
            }
            let cards_not_short_of_memory: Vec<usize> = (0..ticks_of_each_card.len())
                .filter(|card| !rounds.is_short_of_memory(card_index(*card)))
                .collect();
            let cards_of_the_round: Vec<usize> = if round == 0 {
                (0..cards_holding_states_in_the_first_round).collect()
            } else {
                cards_not_short_of_memory.iter().copied().take(1).collect()
            };
            let blocks = rounds.take_the_blocks_of_the_round();
            let clock = CardsOnAClockOfTicks::started_by_the_one_starting_the_cards();
            // 写库那一份只借给这一轮的卡：这一轮收工之后才交 `after_the_round`
            let stored_in_order = Mutex::new(rounds.stored_in_order());
            let cards_left: Vec<(usize, CardLeftTheBlocks)> = std::thread::scope(|scope| {
                let mut card_threads = Vec::new();
                for card in cards_of_the_round.iter().copied() {
                    clock.a_card_joins();
                    let (blocks, clock, stored_in_order, stored, judged_times) =
                        (&blocks, &clock, &stored_in_order, &stored, &judged_times);
                    let (blocks_taken_by_card_0, blocks_judged, stop_is_requested) =
                        (&blocks_taken_by_card_0, &blocks_judged, &stop_is_requested);
                    card_threads.push((
                        card,
                        scope.spawn(move || {
                            let _running = CardRunningOnTheClock(clock);
                            judge_blocks_on_one_card(
                                blocks,
                                &|| {
                                    if stop_is_requested.load(Ordering::SeqCst) {
                                        StopOfTheCards::RequestedByTheCaller
                                    } else {
                                        StopOfTheCards::NotRequested
                                    }
                                },
                                &mut |pending_index| {
                                    let taken = match card {
                                        0 => blocks_taken_by_card_0.fetch_add(1, Ordering::SeqCst) + 1,
                                        _ => 0,
                                    };
                                    assert!(
                                        card != 0 || taken <= out_of_memory_at_block_of_card_0,
                                        "第 0 张卡报了 Out of Memory 之后又领了块 {pending_index}：报了就收工"
                                    );
                                    clock.judge_for(card, ticks_of_each_card[card]);
                                    if card == 0 && taken == out_of_memory_at_block_of_card_0 {
                                        if caller_stop == CallerStopOfTheFakeRun::WhenCard0RunsOutOfMemory {
                                            stop_is_requested.store(true, Ordering::SeqCst);
                                        }
                                        return BlockJudgedOnACard::OutOfMemory {
                                            reason: format!("显存不够：第 0 张卡判块 {pending_index}"),
                                        };
                                    }
                                    judged_times.lock().expect("判块的线程不 panic")[pending_index] += 1;
                                    let judged_so_far = blocks_judged.fetch_add(1, Ordering::SeqCst) + 1;
                                    if caller_stop
                                        == CallerStopOfTheFakeRun::WhenTheBlocksJudgedReach(judged_so_far)
                                    {
                                        stop_is_requested.store(true, Ordering::SeqCst);
                                    }
                                    BlockJudgedOnACard::Judged(pending_index)
                                },
                                &mut |pending_index, judged| {
                                    stored_in_order.lock().expect("写库不 panic").arrive(
                                        pending_index,
                                        judged,
                                        &mut |block| {
                                            stored.lock().expect("写库不 panic").push(block);
                                            Ok(())
                                        },
                                    );
                                    BlockHandedToTheStore::Handed
                                },
                            )
                        }),
                    ));
                    clock.wait_until_only_the_one_starting_the_cards_runs();
                }
                clock.one_running_stops();
                card_threads
                    .into_iter()
                    .map(|(card, thread)| (card, thread.join().expect("假卡的线程不 panic")))
                    .collect()
            });
            let cards_out_of_memory: Vec<(u32, String)> = cards_left
                .iter()
                .filter_map(|(card, left)| match left {
                    CardLeftTheBlocks::OutOfMemory { reason } => {
                        Some((card_index(*card), reason.clone()))
                    }
                    CardLeftTheBlocks::NothingLeftOrStopped
                    | CardLeftTheBlocks::Refused { .. }
                    | CardLeftTheBlocks::TheStoreIsGone => None,
                })
                .collect();
            let cards_finished: Vec<u32> = cards_left
                .iter()
                .filter(|(_, left)| *left == CardLeftTheBlocks::NothingLeftOrStopped)
                .map(|(card, _)| card_index(*card))
                .collect();
            let stop = if stop_is_requested.load(Ordering::SeqCst) {
                StopOfTheRun::Requested
            } else {
                StopOfTheRun::NotRequested
            };
            let cards_of_the_round_by_index: Vec<u32> = cards_not_short_of_memory
                .iter()
                .map(|card| card_index(*card))
                .collect();
            match rounds.after_the_round(
                &blocks,
                &cards_of_the_round_by_index,
                cards_finished,
                &cards_out_of_memory,
                stop,
            ) {
                AfterARoundOfTheCards::AnotherRoundWithoutTheCardsShortOfMemory
                | AfterARoundOfTheCards::AnotherRoundOnlyForTheBlocksTheJudgedBlocksWaitFor => {
                    round += 1;
                }
                after_the_last_round @ (AfterARoundOfTheCards::EveryBlockStored
                | AfterARoundOfTheCards::StoppedOnRequest
                | AfterARoundOfTheCards::NotRunTheGpuMemoryWasShort) => {
                    return BlocksJudgedOnFakeCards {
                        stored: stored.into_inner().expect("写库不 panic"),
                        judged_times: judged_times.into_inner().expect("判块的线程不 panic"),
                        after_the_last_round,
                        blocks_left_for_the_next_run: rounds.blocks_of_the_next_round(),
                        cards_used: rounds.cards_used(),
                    };
                }
            }
            assert!(
                round <= ticks_of_each_card.len(),
                "每再起一轮至少少一张卡：轮数不超过卡数"
            );
        }
    }

    /// 第 10z 批第三块（第 10z 批代码轮判决 K2-1 第 5 条，数照攻方模型那三组快慢：20 块，第 0 张卡判一块是第 1 张的 1、2、3 倍）：第 0 张卡在它领的
    /// 第 1 到 20 块中任一块末尾报 Out of Memory、交回那一块，别的卡都已收工时也由收尾那一步再起的一轮判掉；每一格 20 块都照块序写进库、
    /// 没有一块判两遍。改前快慢 3:1、第 5 块那一格交回的没人判、它后面判完的 2 块也不入库。
    #[test]
    fn a_block_a_card_gives_back_is_judged_once_and_every_block_is_stored_at_any_speed_of_two_cards(
    ) {
        const PENDING_BLOCKS: usize = 20;
        for ticks_of_each_card in [[1u64, 1], [2, 1], [3, 1]] {
            for out_of_memory_at_block_of_card_0 in 1..=PENDING_BLOCKS {
                let judged = blocks_judged_on_fake_cards(
                    PENDING_BLOCKS,
                    &ticks_of_each_card,
                    2,
                    out_of_memory_at_block_of_card_0,
                    CallerStopOfTheFakeRun::Never,
                );
                let cell = format!(
                    "快慢 {ticks_of_each_card:?}、第 0 张卡第 {out_of_memory_at_block_of_card_0} 块报"
                );
                assert_eq!(
                    judged.after_the_last_round,
                    AfterARoundOfTheCards::EveryBlockStored,
                    "{cell}"
                );
                assert_eq!(
                    judged.stored,
                    (0..PENDING_BLOCKS).collect::<Vec<usize>>(),
                    "{cell}：每一块都照块序写进库"
                );
                assert_eq!(
                    judged.judged_times,
                    vec![1; PENDING_BLOCKS],
                    "{cell}：每一块判一遍"
                );
            }
        }
    }

    /// 第 10z 批第三块（第 10z 批代码轮判决 K2-1 第 5 条攻方另一形）：状态全装在第一张卡（第二张头一轮没起），第一张在它的第 5 块报了：
    /// 收尾那一步再起一轮、第二张判剩下的，20 块都照块序写进库、没有一块判两遍。改前第二张闲着、这一趟报本次未跑。
    #[test]
    fn with_every_state_on_the_first_card_the_blocks_it_leaves_are_judged_on_the_second_card() {
        let judged = blocks_judged_on_fake_cards(20, &[1, 1], 1, 5, CallerStopOfTheFakeRun::Never);
        assert_eq!(
            judged.after_the_last_round,
            AfterARoundOfTheCards::EveryBlockStored
        );
        assert_eq!(judged.stored, (0..20).collect::<Vec<usize>>());
        assert_eq!(judged.judged_times, vec![1; 20]);
    }

    /// 第 10z 批第三块：只有一张卡，它在第 5 块报了：没有卡能判了，这一趟报本次未跑，前 4 块在库里。
    #[test]
    fn with_one_card_only_a_block_it_gives_back_leaves_the_run_not_run() {
        let judged = blocks_judged_on_fake_cards(20, &[1], 1, 5, CallerStopOfTheFakeRun::Never);
        assert_eq!(
            judged.after_the_last_round,
            AfterARoundOfTheCards::NotRunTheGpuMemoryWasShort
        );
        assert_eq!(judged.stored, vec![0, 1, 2, 3]);
    }

    /// 第 10z 批第五块（第 10z 批第三块代码轮判决 K1-5，攻方 `attack_10zc_rounds.rs` 的 `stop-in-the-closing-round` 四格）：两张卡、5 块，
    /// 第 0 张领块 0、判得慢（10 拍）、判到末尾报 Out of Memory 交回；第 1 张先判完块 1–4、收工（块 1–4 等着块 0）。叫停落在四个时刻：
    /// 不叫停、头一轮第 1 张判完它最后一块那一刻、收尾再起的那一轮起卡之前、收尾那一轮判完块 0 那一刻。每一格都不 panic，判完的块照块序都入库、
    /// 没有一块判两遍，第 1 张卡两轮各收工一次只算一张。改前「收尾那一轮起卡之前」那一格在轮末的断言上 panic（等着的 4 块）、
    /// 「头一轮最后一块」那一格报本次未跑、4 块判完不入库。
    #[test]
    fn a_stop_around_the_closing_round_stores_every_block_judged_before_it() {
        for caller_stop in [
            CallerStopOfTheFakeRun::Never,
            CallerStopOfTheFakeRun::WhenTheBlocksJudgedReach(4),
            CallerStopOfTheFakeRun::BeforeRound(1),
            CallerStopOfTheFakeRun::WhenTheBlocksJudgedReach(5),
        ] {
            let judged = blocks_judged_on_fake_cards(5, &[10, 1], 2, 1, caller_stop);
            assert_eq!(
                judged.after_the_last_round,
                AfterARoundOfTheCards::EveryBlockStored,
                "{caller_stop:?}"
            );
            assert_eq!(
                judged.stored,
                vec![0, 1, 2, 3, 4],
                "{caller_stop:?}：判完的块都入库"
            );
            assert_eq!(
                judged.judged_times,
                vec![1; 5],
                "{caller_stop:?}：每块判一遍"
            );
            assert_eq!(judged.cards_used, 1, "{caller_stop:?}：第 1 张卡只算一张");
        }
    }

    /// 第 10z 批第五块（第 10z 批第三块代码轮正推 `rounds_sim4.rs` 的 `STOPSIM stop_from_round=Some(2)`）：两张卡、10 块，第 0 张很慢、
    /// 判第一块时报 Out of Memory，第 1 张把别的 9 块都判完先收工；收尾再起的那一轮起卡之前叫停：那一轮照样判交回的块 0，10 块都入库、不 panic。
    #[test]
    fn a_stop_before_the_closing_round_still_judges_the_block_nine_judged_blocks_wait_for() {
        let judged = blocks_judged_on_fake_cards(
            10,
            &[300, 1],
            2,
            1,
            CallerStopOfTheFakeRun::BeforeRound(1),
        );
        assert_eq!(
            judged.after_the_last_round,
            AfterARoundOfTheCards::EveryBlockStored
        );
        assert_eq!(judged.stored, (0..10).collect::<Vec<usize>>());
        assert_eq!(judged.judged_times, vec![1; 10]);
    }

    /// 第 10z 批第五块（第 10z 批第三块代码轮判决 K1-5「没判的留给下一趟」）：20 块，第 1 张卡判完块 1–3 时被叫停，第 0 张在块 0 上报了交回：
    /// 收尾那一轮只判块 0，块 0–3 照块序入库，块 4–19 一块都不判、留给下一趟，这一趟照叫停收尾。
    #[test]
    fn a_stop_leaves_the_blocks_not_judged_to_the_next_run_after_storing_the_judged_ones() {
        let judged = blocks_judged_on_fake_cards(
            20,
            &[10, 1],
            2,
            1,
            CallerStopOfTheFakeRun::WhenTheBlocksJudgedReach(3),
        );
        assert_eq!(
            judged.after_the_last_round,
            AfterARoundOfTheCards::StoppedOnRequest
        );
        assert_eq!(judged.stored, vec![0, 1, 2, 3]);
        assert_eq!(
            judged.judged_times,
            [vec![1; 4], vec![0; 16]].concat(),
            "叫停之后只判判完的块在等的块 0"
        );
        assert_eq!(
            judged.blocks_left_for_the_next_run,
            (4..20).collect::<Vec<usize>>()
        );
    }

    /// 第 10z 批第五块（第 10z 批第三块代码轮判决 K1-5）：同一轮里第 0 张卡在块 0 上报了 Out of Memory、交回，那一刻叫停；第 1 张还在判块 1：
    /// 判完块 1 之后照样领交回的块 0（块 1 在等它），块 0、1 入库，这一趟照叫停收尾、不报本次未跑（与交回的块落到收尾那一轮时同一个结局）。
    #[test]
    fn a_stop_with_a_block_given_back_in_the_same_round_stores_the_judged_block_and_stops() {
        let judged = blocks_judged_on_fake_cards(
            4,
            &[1, 2],
            2,
            1,
            CallerStopOfTheFakeRun::WhenCard0RunsOutOfMemory,
        );
        assert_eq!(
            judged.after_the_last_round,
            AfterARoundOfTheCards::StoppedOnRequest
        );
        assert_eq!(judged.stored, vec![0, 1]);
        assert_eq!(judged.judged_times, vec![1, 1, 0, 0]);
        assert_eq!(judged.blocks_left_for_the_next_run, vec![2, 3]);
    }

    /// 第 10z 批第六块（第 10z 批第五块代码轮判决 K2-2，攻方 F2）：一趟里见过一次叫停就记住：叫停文件头一次读到之后被删，之后每一处问都答叫停，
    /// 也不再读文件；没见过叫停时每问一次现读一次。
    #[test]
    fn a_stop_request_seen_once_stays_requested_after_the_request_is_gone() {
        let reads = AtomicUsize::new(0);
        let request_is_there_on_the_second_read = || reads.fetch_add(1, Ordering::SeqCst) == 1;
        let stop = StopRequestRememberedInThisRun::new(&request_is_there_on_the_second_read);
        assert!(!stop.is_requested(), "头一次读：叫停文件还没有");
        assert!(stop.is_requested(), "第二次读：叫停文件在");
        assert_eq!(
            stop.stop_of_the_run(),
            StopOfTheRun::Requested,
            "叫停文件已经删了：见过一次就记住"
        );
        assert!(stop.is_requested(), "再问照旧");
        assert_eq!(reads.load(Ordering::SeqCst), 2, "见过叫停之后不再读文件");
    }

    /// 第 10z 批第六块（第 10z 批第五块代码轮判决 K1-3）：叫停、这一轮有卡报了 Out of Memory、判完的块都入了库而照叫停收尾：头一句 Out of Memory
    /// 照「本次未跑」那一行的形态打得出来；这一趟没有卡报过的，照叫停收尾不打这一行。
    #[test]
    fn a_stop_after_an_out_of_memory_prints_the_first_out_of_memory_in_the_not_run_line() {
        let mut rounds: RoundsOfTheCards<usize> = RoundsOfTheCards::new(3);
        let blocks = rounds.take_the_blocks_of_the_round();
        assert_eq!(blocks.take_a_block(StopOfTheCards::NotRequested), Some(0));
        rounds.stored_in_order().arrive(0, 0, &mut |_block| Ok(()));
        assert_eq!(
            rounds.after_the_round(
                &blocks,
                &[0, 1],
                [1],
                &[(0, "序号 0 的卡判一块时报了 Out of Memory".to_string())],
                StopOfTheRun::Requested
            ),
            AfterARoundOfTheCards::StoppedOnRequest
        );
        assert_eq!(
            rounds
                .not_run_line_of_a_stop_after_an_out_of_memory(
                    "crash_amplification::run_from_environment"
                )
                .as_deref(),
            Some(
                "NOT_RUN_NO_GPU_ADAPTER test=crash_amplification::run_from_environment reason=序号 0 的卡判一块时报了 Out of Memory"
            )
        );
        let mut rounds_without_out_of_memory: RoundsOfTheCards<usize> = RoundsOfTheCards::new(3);
        let blocks_without_out_of_memory =
            rounds_without_out_of_memory.take_the_blocks_of_the_round();
        assert_eq!(
            rounds_without_out_of_memory.after_the_round(
                &blocks_without_out_of_memory,
                &[0],
                [0],
                &[],
                StopOfTheRun::Requested
            ),
            AfterARoundOfTheCards::StoppedOnRequest
        );
        assert_eq!(
            rounds_without_out_of_memory.not_run_line_of_a_stop_after_an_out_of_memory(
                "crash_amplification::run_from_environment"
            ),
            None,
            "没有卡报过 Out of Memory：照叫停收尾不打本次未跑那一行"
        );
    }

    /// 第 10z 批第六块（第 10z 批第五块代码轮判决 K3，攻方 F4）：一段活交回的那一刻记设备丢没丢：活里设备丢了，交回那一刻记成丢了；
    /// 活 panic 时设备没丢、之后才丢，交回那一刻记成没丢（之后丢的不把这段活真正的 panic 说成显存不够）。
    #[test]
    fn a_work_notes_the_device_at_the_moment_it_returns_not_before_and_not_after() {
        let lost_during_the_work = DeviceConditionOfACard::default();
        let (outcome, device_when_the_work_returned) =
            work_caught_noting_the_device_when_it_returned(&lost_during_the_work, || {
                lost_during_the_work.record_the_device_lost(
                    "Unknown：Out of memory".to_string(),
                    CauseOfTheDeviceLoss::GpuMemoryShort,
                );
                7
            });
        assert!(matches!(outcome, Ok(7)));
        assert_eq!(
            device_when_the_work_returned,
            DeviceWhenTheWorkReturned::Lost,
            "活里设备丢了：交回那一刻记成丢了"
        );
        let lost_after_the_work = DeviceConditionOfACard::default();
        let (panicked_outcome, device_when_the_panicked_work_returned) =
            work_caught_noting_the_device_when_it_returned(&lost_after_the_work, || -> u32 {
                panic!("这段活自己真 panic 了")
            });
        lost_after_the_work.record_the_device_lost(
            "Unknown：Out of memory".to_string(),
            CauseOfTheDeviceLoss::GpuMemoryShort,
        );
        assert!(panicked_outcome.is_err(), "panic 接住了");
        assert_eq!(
            device_when_the_panicked_work_returned,
            DeviceWhenTheWorkReturned::NotLost,
            "活 panic 时设备没丢、之后才丢：交回那一刻记成没丢"
        );
    }

    /// 第 10z 批第六块（第 10z 批第五块代码轮判决 K3：起下一张卡的看叫停没有用例）：后面装了状态的那一张，队里照此刻的叫停还有领得的块才起；
    /// 叫停了、判完的块都不在等，不起；判红了不起；后面没有装了状态的卡，不起。
    #[test]
    fn the_next_card_starts_only_when_it_holds_states_and_a_block_is_left_to_take_under_the_stop() {
        let blocks = BlocksForTheCards::new([0usize, 1, 2]);
        assert_eq!(
            next_card_to_start(&[1, 0, 1], 0, &blocks, || StopOfTheCards::NotRequested),
            Some(2),
            "跳过没装状态的那一张"
        );
        assert_eq!(
            next_card_to_start(&[1, 0, 1], 0, &blocks, || {
                StopOfTheCards::RequestedByTheCaller
            }),
            None,
            "叫停了、还没有判完的块：没有判完的块在等的，不起"
        );
        assert_eq!(
            next_card_to_start(&[1, 0, 1], 0, &blocks, || StopOfTheCards::TheRunFailed),
            None,
            "判红了：不起"
        );
        assert_eq!(
            next_card_to_start(&[1, 0, 1], 2, &blocks, || StopOfTheCards::NotRequested),
            None,
            "后面没有卡了"
        );
        assert_eq!(
            next_card_to_start(&[1, 0], 0, &blocks, || StopOfTheCards::NotRequested),
            None,
            "后面的卡没装状态"
        );
    }

    /// 第 10z 批第七块：起下一张卡时叫停只在后面有装了状态的卡时问、问一次；后面没有卡、后面的卡都没装状态，一次都不问
    /// （崩溃放量产品路径的叫停可以按「问第几次」变，多问一次卡上就少领一块）。
    #[test]
    fn the_next_card_asks_for_the_stop_once_only_when_a_later_card_holds_states() {
        let blocks = BlocksForTheCards::new([0usize, 1, 2]);
        let times_asked = std::cell::Cell::new(0usize);
        let stop_counting_the_asks = || {
            times_asked.set(times_asked.get() + 1);
            StopOfTheCards::NotRequested
        };
        assert_eq!(
            next_card_to_start(&[1, 0, 1], 2, &blocks, stop_counting_the_asks),
            None,
            "最后一张卡之后没有卡"
        );
        assert_eq!(times_asked.get(), 0, "后面没有卡：叫停一次都不问");
        assert_eq!(
            next_card_to_start(&[1, 0, 0], 0, &blocks, stop_counting_the_asks),
            None,
            "后面的卡都没装状态"
        );
        assert_eq!(times_asked.get(), 0, "后面的卡都没装状态：叫停一次都不问");
        assert_eq!(
            next_card_to_start(&[1, 0, 1], 0, &blocks, stop_counting_the_asks),
            Some(2),
            "后面有装了状态的卡、没叫停：起它"
        );
        assert_eq!(times_asked.get(), 1, "后面有装了状态的卡：叫停恰好问一次");
    }

    /// 接住一段会 panic 的活，交它 panic 了没有。
    fn panics_when_run(work: impl FnOnce()) -> bool {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(work)).is_err()
    }

    /// 第 10z 批第六块（第 10z 批第五块代码轮判决 K3，正推 `probes/killers_new.rs` M05）：三张卡各领一块 5、6、7；7 先判完、5 后判完、
    /// 6 最后报 Out of Memory 交回。叫停之后下一张卡领得到 6：判完的块里最大的是 7（在等 6），不是最后判完的 5。
    #[test]
    fn the_largest_judged_block_not_the_last_judged_lets_a_stopped_card_take_the_block_it_waits_for(
    ) {
        let blocks = BlocksForTheCards::new([5usize, 6, 7]);
        let (block_five_taken_sender, block_five_taken) = std::sync::mpsc::channel::<()>();
        let (block_six_taken_sender, block_six_taken) = std::sync::mpsc::channel::<()>();
        let (block_seven_taken_sender, block_seven_taken) = std::sync::mpsc::channel::<()>();
        let (finish_block_five_sender, finish_block_five) = std::sync::mpsc::channel::<()>();
        let (finish_block_six_sender, finish_block_six) = std::sync::mpsc::channel::<()>();
        let (finish_block_seven_sender, finish_block_seven) = std::sync::mpsc::channel::<()>();
        let (block_five_stored_sender, block_five_stored) = std::sync::mpsc::channel::<()>();
        let (block_seven_stored_sender, block_seven_stored) = std::sync::mpsc::channel::<()>();
        let enough_for_any_step = std::time::Duration::from_secs(20);
        // 每张卡只领一块：头一次照常领，之后判红（不再领）
        let one_block_then_stop = || {
            let is_the_first_ask = std::cell::Cell::new(true);
            move || {
                if is_the_first_ask.replace(false) {
                    StopOfTheCards::NotRequested
                } else {
                    StopOfTheCards::TheRunFailed
                }
            }
        };
        let card_left_with_block_six = std::thread::scope(|scope| {
            let blocks = &blocks;
            scope.spawn(move || {
                let stop_of_the_cards = one_block_then_stop();
                let _left = judge_blocks_on_one_card(
                    blocks,
                    &stop_of_the_cards,
                    &mut |block| {
                        assert_eq!(block, 5);
                        block_five_taken_sender.send(()).expect("主线程等着");
                        finish_block_five.recv().expect("主线程会放行");
                        BlockJudgedOnACard::Judged(block)
                    },
                    &mut |_block, _judged: usize| {
                        block_five_stored_sender.send(()).expect("主线程等着");
                        BlockHandedToTheStore::Handed
                    },
                );
            });
            block_five_taken
                .recv_timeout(enough_for_any_step)
                .expect("块 5 领走了");
            let card_with_block_six = scope.spawn(move || {
                let stop_of_the_cards = one_block_then_stop();
                judge_blocks_on_one_card(
                    blocks,
                    &stop_of_the_cards,
                    &mut |block| {
                        assert_eq!(block, 6);
                        block_six_taken_sender.send(()).expect("主线程等着");
                        finish_block_six.recv().expect("主线程会放行");
                        BlockJudgedOnACard::<usize>::OutOfMemory {
                            reason: "块 6 上报了 Out of Memory".to_string(),
                        }
                    },
                    &mut |_block, _judged: usize| BlockHandedToTheStore::Handed,
                )
            });
            block_six_taken
                .recv_timeout(enough_for_any_step)
                .expect("块 6 领走了");
            scope.spawn(move || {
                let stop_of_the_cards = one_block_then_stop();
                let _left = judge_blocks_on_one_card(
                    blocks,
                    &stop_of_the_cards,
                    &mut |block| {
                        assert_eq!(block, 7);
                        block_seven_taken_sender.send(()).expect("主线程等着");
                        finish_block_seven.recv().expect("主线程会放行");
                        BlockJudgedOnACard::Judged(block)
                    },
                    &mut |_block, _judged: usize| {
                        block_seven_stored_sender.send(()).expect("主线程等着");
                        BlockHandedToTheStore::Handed
                    },
                );
            });
            block_seven_taken
                .recv_timeout(enough_for_any_step)
                .expect("块 7 领走了");
            finish_block_seven_sender.send(()).expect("块 7 那张卡等着");
            block_seven_stored
                .recv_timeout(enough_for_any_step)
                .expect("块 7 判完");
            finish_block_five_sender.send(()).expect("块 5 那张卡等着");
            block_five_stored
                .recv_timeout(enough_for_any_step)
                .expect("块 5 判完");
            finish_block_six_sender.send(()).expect("块 6 那张卡等着");
            card_with_block_six.join().expect("块 6 那张卡不 panic")
        });
        assert!(matches!(
            card_left_with_block_six,
            CardLeftTheBlocks::OutOfMemory { .. }
        ));
        assert_eq!(
            blocks.take_a_block(StopOfTheCards::RequestedByTheCaller),
            Some(6),
            "7 判完了在等 6：叫停之后下一张卡领得到 6"
        );
    }

    /// 第 10z 批第六块（正推 `probes/killers_new.rs` M10）：叫停、一张卡报了 Out of Memory、判完的块只有 1 块在等：还有别的卡就再起一轮，
    /// 没有别的卡是本次未跑，都不是照叫停收尾；一块都不在等才照叫停收尾。
    #[test]
    fn a_stop_with_one_judged_block_waiting_needs_another_round_or_is_not_run() {
        assert_eq!(
            after_a_round_of_the_cards(3, 1, 1, StopOfTheRun::Requested, 1),
            AfterARoundOfTheCards::AnotherRoundOnlyForTheBlocksTheJudgedBlocksWaitFor
        );
        assert_eq!(
            after_a_round_of_the_cards(3, 1, 1, StopOfTheRun::Requested, 0),
            AfterARoundOfTheCards::NotRunTheGpuMemoryWasShort
        );
        assert_eq!(
            after_a_round_of_the_cards(3, 0, 1, StopOfTheRun::Requested, 1),
            AfterARoundOfTheCards::StoppedOnRequest
        );
    }

    /// 第 10z 批第六块（正推 `probes/killers_new.rs` M13）：没有卡报 Out of Memory 而判完的块还有在等前面那一块的：轮末断言 panic（叫停了也一样）。
    #[test]
    fn a_round_without_out_of_memory_leaving_a_judged_block_waiting_panics() {
        let mut rounds: RoundsOfTheCards<usize> = RoundsOfTheCards::new(3);
        let blocks = rounds.take_the_blocks_of_the_round();
        assert_eq!(blocks.take_a_block(StopOfTheCards::NotRequested), Some(0));
        rounds.stored_in_order().arrive(2, 2, &mut |_block| Ok(()));
        assert_eq!(rounds.blocks_waiting(), 1);
        assert!(panics_when_run(|| {
            let _after =
                rounds.after_the_round(&blocks, &[0], [0u32], &[], StopOfTheRun::Requested);
        }));
    }

    /// 第 10z 批第六块（正推 `probes/killers_new.rs` M14）：没人叫停、没有卡报 Out of Memory，队里却还有没领走的块：轮末断言 panic；叫停了就可以留。
    #[test]
    fn a_round_without_a_stop_leaving_a_block_untaken_panics() {
        let mut rounds: RoundsOfTheCards<usize> = RoundsOfTheCards::new(3);
        let blocks = rounds.take_the_blocks_of_the_round();
        assert!(panics_when_run(|| {
            let _after =
                rounds.after_the_round(&blocks, &[0], [0u32], &[], StopOfTheRun::NotRequested);
        }));
        let mut stopped_rounds: RoundsOfTheCards<usize> = RoundsOfTheCards::new(3);
        let stopped_blocks = stopped_rounds.take_the_blocks_of_the_round();
        assert_eq!(
            stopped_rounds.after_the_round(
                &stopped_blocks,
                &[0],
                [0u32],
                &[],
                StopOfTheRun::Requested
            ),
            AfterARoundOfTheCards::StoppedOnRequest
        );
    }

    /// 第 10z 批第六块（正推 `probes/killers_new.rs` M17）：这一趟头一句 Out of Memory 是第一张报的那一句，后面报的不盖掉它。
    #[test]
    fn the_first_out_of_memory_of_the_run_stays_the_first_reported() {
        let mut rounds: RoundsOfTheCards<usize> = RoundsOfTheCards::new(2);
        let blocks = rounds.take_the_blocks_of_the_round();
        let _after = rounds.after_the_round(
            &blocks,
            &[0, 1],
            [],
            &[(0, "第一句".to_string()), (1, "第二句".to_string())],
            StopOfTheRun::NotRequested,
        );
        assert_eq!(rounds.first_out_of_memory(), Some("第一句"));
    }

    /// 第 10z 批第六块（正推 `probes/killers_new.rs` M22）：一块交回两次要 panic。
    #[test]
    fn giving_a_block_back_twice_panics() {
        let blocks = BlocksForTheCards::new([0usize, 1]);
        let first = blocks
            .take_a_block(StopOfTheCards::NotRequested)
            .expect("队里有块");
        blocks.give_back(first);
        assert!(panics_when_run(|| blocks.give_back(first)));
    }

    /// 第 10z 批第六块（正推 `probes/killers_new.rs` M23）：同一段活上记了两次 Out of Memory，认出来的是最后那一次。
    #[test]
    fn the_last_out_of_memory_recorded_on_a_work_is_the_one_told() {
        let condition = DeviceConditionOfACard::default();
        let work = WorkOnTheCardOfThisThread::begin();
        let reports_before = condition.out_of_memory_reports();
        condition.record_an_out_of_memory("先报的".to_string());
        condition.record_an_out_of_memory("后报的".to_string());
        assert_eq!(
            condition
                .out_of_memory_explaining_a_failure_of_the_work(
                    work.identifier(),
                    reports_before,
                    DeviceWhenTheWorkReturned::NotLost
                )
                .as_deref(),
            Some("后报的")
        );
    }
}
