//! 一张卡给判器的显存：额度怎么算（配了额度的取额度与空闲里小的那个，没配的取空闲的一部分）、低于门槛的卡用不了，
//! 照此刻空闲放不下而加回这个进程自己占着的就放得下时等这个进程还。

use super::*;

/// 最小一份判表上卡占多少（MiB）：打包判表一个字 4 字节，上卡按 MiB 向上取整（`crate::crash_judge_dispatch::PackedJudgeTables::mebibytes`），
/// 表里有一个字就占这么多。
pub const MEBIBYTES_OF_THE_SMALLEST_JUDGE_TABLES: u64 =
    ((u32::BITS / u8::BITS) as u64).div_ceil(1024 * 1024);

/// 一张卡给崩溃放量的额度（[`memory_budget_mebibytes_of`]）低于这个数（MiB）就用不了：驱动那一份
/// （[`crate::crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER`]）加最小一份判表（[`MEBIBYTES_OF_THE_SMALLEST_JUDGE_TABLES`]）。
/// 额度比它小的卡连最小一份表也放不下、一个状态都派不出去（`crash_amplification::states_in_flight_within_the_budget` 交 0）；
/// 本机有卡被别的服务占着时只剩几百 MiB，起上下文都难（E163 M2 那张卡 141 MiB 起不来）。
/// 只管 [`gpu_cards_by_priority`]（照 [`card_memory_for_a_judge`] 判）；卡池按账判，此刻被别的进程占着的卡照样进池，
/// 借的那一刻借不到就报本次未跑（不等）。
pub const MINIMUM_MEMORY_BUDGET_MEBIBYTES: u64 =
    crate::crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER
        + MEBIBYTES_OF_THE_SMALLEST_JUDGE_TABLES;

/// 一张卡照此刻空闲判放不下、空闲加回这个进程自己占着的就放得下时，最多等这个进程还多久（[`card_memory_once_a_judge_fits`]）：
/// 同一个测试进程里别的 GPU 用例判完就还；等到头还没还够算用不了（这个进程自己的这条线程占着、或别的用例一直占着，等多久都一样）。
const LONGEST_WAIT_FOR_THIS_PROCESS_TO_RETURN_CARD_MEMORY: std::time::Duration =
    std::time::Duration::from_secs(600);

/// 等这个进程还显存时两次读卡之间停多久。
const PAUSE_BETWEEN_CARD_MEMORY_READINGS: std::time::Duration = std::time::Duration::from_secs(1);

/// 一张卡此刻在这个进程看来能不能给判器（[`card_memory_for_a_judge`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CardMemoryForAJudge {
    /// 照此刻空闲算的额度不低于 [`MINIMUM_MEMORY_BUDGET_MEBIBYTES`]：起设备。
    RoomNow,
    /// 照此刻空闲算的额度不够，空闲加回这个进程自己此刻占在这张卡上的就够：同一个进程里别的用例还了才放得下，等它还。
    RoomOnceThisProcessReturnsWhatItHolds,
    /// 加回这个进程自己占着的也不够：别的进程占着，或卡本身太小。用不了。
    NoRoomEvenWithWhatThisProcessHolds,
}

/// 这个进程此刻自己占在一张卡上的（MiB，`nvidia-smi` 按进程号报的；没列是 0）：走不走卡池的账都算，判这张卡用不用得了时
/// 同一个进程里别的用例起的设备与缓冲不算别人占的。
fn mebibytes_this_process_occupies(reading: CardMemoryReading) -> u64 {
    match reading.occupied_by_this_process {
        OccupiedByThisProcess::NotListed => 0,
        OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes }
        | OccupiedByThisProcess::AlsoOutsideTheBooks { mebibytes } => mebibytes,
    }
}

/// 照一次读数（`reading`）判一张卡能不能给判器，`memory_quota_mebibytes` 是配置给它的额度（没配是 `None`）：照此刻空闲算的额度
/// （[`memory_budget_mebibytes_of`]）不低于 [`MINIMUM_MEMORY_BUDGET_MEBIBYTES`] 是此刻放得下；不够时空闲加回这个进程自己占着的
/// （不超过空卡上能用的）再算一次，够是等这个进程还，还不够是用不了。
fn card_memory_for_a_judge(
    memory_quota_mebibytes: Option<u64>,
    reading: CardMemoryReading,
) -> CardMemoryForAJudge {
    if memory_budget_mebibytes_of(memory_quota_mebibytes, reading.free_mebibytes)
        >= MINIMUM_MEMORY_BUDGET_MEBIBYTES
    {
        return CardMemoryForAJudge::RoomNow;
    }
    let free_with_what_this_process_holds_mebibytes = reading
        .free_mebibytes
        .saturating_add(mebibytes_this_process_occupies(reading))
        .min(reading.total_less_reserved_mebibytes);
    if memory_budget_mebibytes_of(
        memory_quota_mebibytes,
        free_with_what_this_process_holds_mebibytes,
    ) >= MINIMUM_MEMORY_BUDGET_MEBIBYTES
    {
        CardMemoryForAJudge::RoomOnceThisProcessReturnsWhatItHolds
    } else {
        CardMemoryForAJudge::NoRoomEvenWithWhatThisProcessHolds
    }
}

/// 一张卡等到此刻放得下为止（[`card_memory_for_a_judge`]）：读一次（`read_the_card`），此刻放得下、或加回这个进程占着的也放不下，
/// 当场交；要等这个进程还的，停一会儿（`pause`）再读，最多读 `most_readings` 次（0 当 1）。交最后一次的判定与读数。
///
/// # Errors
/// 读不出卡上的显存（`read_the_card` 的错）。
fn card_memory_for_a_judge_waiting_for_this_process(
    memory_quota_mebibytes: Option<u64>,
    read_the_card: &mut dyn FnMut() -> Result<CardMemoryReading, String>,
    pause: &mut dyn FnMut(),
    most_readings: u64,
) -> Result<(CardMemoryForAJudge, CardMemoryReading), String> {
    let mut readings_left = most_readings.max(1);
    loop {
        let reading = read_the_card()?;
        let judged = card_memory_for_a_judge(memory_quota_mebibytes, reading);
        readings_left -= 1;
        match judged {
            CardMemoryForAJudge::RoomOnceThisProcessReturnsWhatItHolds if readings_left > 0 => {
                pause();
            }
            CardMemoryForAJudge::RoomNow
            | CardMemoryForAJudge::RoomOnceThisProcessReturnsWhatItHolds
            | CardMemoryForAJudge::NoRoomEvenWithWhatThisProcessHolds => {
                return Ok((judged, reading));
            }
        }
    }
}

/// 这一刻 `nvidia-smi` 报的总线 `bus` 那张卡的显存：空闲、空卡上能用的与这个进程占着的出自同一份报告
/// （[`nvidia_smi_memory_and_processes_report`]、[`card_memory_readings_of_one_nvidia_smi_report`]）。
///
/// # Errors
/// `nvidia-smi` 起不来、报告里没有这张卡或读不出。
fn card_memory_reading_now(bus: &str) -> Result<CardMemoryReading, String> {
    let report = nvidia_smi_memory_and_processes_report()?;
    let readings = card_memory_readings_of_one_nvidia_smi_report(
        &report,
        std::process::id(),
        &[bus.to_string()],
        &has_devices_started_outside_the_card_pool_on_the_bus,
    )?;
    readings
        .into_iter()
        .next()
        .ok_or_else(|| format!("nvidia-smi 的报告里读不出总线 {bus} 那张卡"))
}

/// 找到的一张卡能不能在上面起设备给判器（`memory_quota_mebibytes` 是配置给它的额度，没配是 `None`）：照 `nvidia-smi` 现读、
/// 照 [`card_memory_for_a_judge_waiting_for_this_process`] 判，被这个进程自己占着的最多等
/// [`LONGEST_WAIT_FOR_THIS_PROCESS_TO_RETURN_CARD_MEMORY`]。此刻放得下交读到的那一份。
///
/// # Errors
/// 用不了：交为什么（读不出显存；加回这个进程占着的也放不下；等这个进程还等到头也没还够）。
pub(super) fn card_memory_once_a_judge_fits(
    discovered: &DiscoveredCard,
    memory_quota_mebibytes: Option<u64>,
) -> Result<CardMemoryReading, String> {
    let most_readings = u64::try_from(
        LONGEST_WAIT_FOR_THIS_PROCESS_TO_RETURN_CARD_MEMORY.as_millis()
            / PAUSE_BETWEEN_CARD_MEMORY_READINGS.as_millis(),
    )
    .unwrap_or(u64::MAX);
    let (judged, reading) = card_memory_for_a_judge_waiting_for_this_process(
        memory_quota_mebibytes,
        &mut || card_memory_reading_now(&discovered.pci_bus_identifier),
        &mut || std::thread::sleep(PAUSE_BETWEEN_CARD_MEMORY_READINGS),
        most_readings,
    )?;
    let budget_mebibytes =
        memory_budget_mebibytes_of(memory_quota_mebibytes, reading.free_mebibytes);
    match judged {
        CardMemoryForAJudge::RoomNow => Ok(reading),
        CardMemoryForAJudge::RoomOnceThisProcessReturnsWhatItHolds => Err(format!(
            "照此刻空闲算的额度 {budget_mebibytes} MiB 低于门槛 {MINIMUM_MEMORY_BUDGET_MEBIBYTES} MiB，空闲加回这个进程自己占着的 {} MiB 就够，\
             等这个进程还等了 {} 秒也没还够（读数 {reading:?}）",
            mebibytes_this_process_occupies(reading),
            LONGEST_WAIT_FOR_THIS_PROCESS_TO_RETURN_CARD_MEMORY.as_secs()
        )),
        CardMemoryForAJudge::NoRoomEvenWithWhatThisProcessHolds => Err(format!(
            "照此刻空闲算的额度 {budget_mebibytes} MiB 低于门槛 {MINIMUM_MEMORY_BUDGET_MEBIBYTES} MiB，空闲加回这个进程自己占着的 {} MiB 也不够\
             （别的进程占着或卡太小；读数 {reading:?}）",
            mebibytes_this_process_occupies(reading)
        )),
    }
}

/// 没给额度的卡，崩溃放量最多用它空闲显存的几分之几（分子、分母）：剩下的留给卡上别的进程。
const SHARE_OF_FREE_MEMORY_WITHOUT_A_QUOTA: (u64, u64) = (3, 4);

/// 一张卡给崩溃放量的额度（MiB）：给了额度（`memory_quota_mebibytes`）的取额度与空闲显存里小的那个，
/// 没给的取空闲显存的 [`SHARE_OF_FREE_MEMORY_WITHOUT_A_QUOTA`]（[`GpuCard::memory_budget_mebibytes`] 照它算；卡池照它、
/// 拿这个进程此刻能在卡上占的算每张卡的账，`singlefs_verification::gpu_memory_ledger` 文件头「口径」）。
#[must_use]
pub fn memory_budget_mebibytes_of(
    memory_quota_mebibytes: Option<u64>,
    free_memory_mebibytes: u64,
) -> u64 {
    match memory_quota_mebibytes {
        Some(quota) => quota.min(free_memory_mebibytes),
        None => {
            let (numerator, denominator) = SHARE_OF_FREE_MEMORY_WITHOUT_A_QUOTA;
            free_memory_mebibytes / denominator * numerator
        }
    }
}

/// 一张卡空着时给崩溃放量的额度（MiB）：照 [`memory_budget_mebibytes_of`] 算，空闲换成空卡上能用的
/// （`usable_memory_of_the_empty_card_mebibytes`，[`GpuCard::usable_memory_of_the_empty_card_mebibytes`]）；没读过空卡上能用的（卡池里的卡）
/// 照 `free_memory_mebibytes` 算，与此刻的额度相同。
#[must_use]
pub fn memory_budget_when_the_card_is_empty_mebibytes_of(
    memory_quota_mebibytes: Option<u64>,
    free_memory_mebibytes: u64,
    usable_memory_of_the_empty_card_mebibytes: Option<u64>,
) -> u64 {
    memory_budget_mebibytes_of(
        memory_quota_mebibytes,
        usable_memory_of_the_empty_card_mebibytes.unwrap_or(free_memory_mebibytes),
    )
}

#[cfg(test)]
mod tests;
