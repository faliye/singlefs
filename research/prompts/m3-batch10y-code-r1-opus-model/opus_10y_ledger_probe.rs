//! 攻方原型（m3-batch10y-code-r1 云端攻方，只在冻结副本的拷贝上跑，不是入库用例）：K1-1 / K2-1 那几形在
//! `singlefs_harness::gpu_memory_ledger` 的真函数上算出这一笔账给多少、实际能用多少。不碰显卡：报告是造的文本（格式照本机
//! `nvidia-smi -q -d MEMORY,PIDS` 原样输出与 595.84 的模板），卡上「真能用的」照造的那一刻的物理量算。输出每行以 `OPUS10Y-` 开头。
use singlefs_harness::gpu_memory_ledger::*;
use std::sync::{Arc, Mutex};

const D: u64 = 2560; // crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER
const TOTAL: u64 = 32607;
const RESERVED: u64 = 497;
const BUS: &str = "00000000:01:00.0";
const ME: u32 = 4242;

/// 一张 5090 的报告：FB 的空闲照给的，进程段照给的（`None` 是 `Processes : None`）。
fn report(free: u64, processes: &[(u32, &str)]) -> String {
    let field = |indent: usize, key: &str, value: &str| format!("{}{:<width$}: {value}\n", " ".repeat(indent), key, width = 55 - indent);
    let mut text = String::from("\n==============NVSMI LOG==============\n\n");
    text += &field(0, "Timestamp", "Wed Oct  7 15:36:04 2026");
    text += &field(0, "Driver Version", "595.84");
    text += &field(0, "CUDA Version", "13.2");
    text += "\n";
    text += &field(0, "Attached GPUs", "1");
    text += &format!("GPU {BUS}\n    FB Memory Usage\n");
    text += &field(8, "Total", &format!("{TOTAL} MiB"));
    text += &field(8, "Reserved", &format!("{RESERVED} MiB"));
    text += &field(8, "Used", "2 MiB");
    text += &field(8, "Free", &format!("{free} MiB"));
    text += "    BAR1 Memory Usage\n";
    text += &field(8, "Total", "32768 MiB");
    text += &field(8, "Used", "1 MiB");
    text += &field(8, "Free", "32767 MiB");
    text += "    Conf Compute Protected Memory Usage\n";
    text += &field(8, "Total", "0 MiB");
    text += &field(8, "Used", "0 MiB");
    text += &field(8, "Free", "0 MiB");
    if processes.is_empty() {
        text += &field(4, "Processes", "None");
        text += "\n";
        return text;
    }
    text += "    Processes\n";
    for (pid, used) in processes {
        text += &field(8, "GPU instance ID", "N/A");
        text += &field(8, "Compute instance ID", "N/A");
        text += &field(8, "Process ID", &pid.to_string());
        text += &field(12, "Type", "C+G");
        text += &field(12, "Name", "/tmp/a_gpu_test");
        text += &field(12, "Used GPU Memory", used);
    }
    text += "\n";
    text
}

fn parse(text: &str, outside: bool) -> Result<Vec<CardMemoryReading>, String> {
    card_memory_readings_of_one_nvidia_smi_report(
        text,
        ME,
        &[pci_bus_identifier_without_the_domain(BUS)],
        &move |_bus: &str| outside,
    )
}

/// 卡池的容量式子（`gpu_unit_checks::gpu_test_start::open_the_card_pool` 那一个闭包；额度变量不设，配置的额度照给的）。
fn capacity(configured: Option<u64>) -> impl Fn(u64) -> u64 {
    move |may_hold: u64| {
        let budget = match configured {
            Some(q) => q.min(may_hold),
            None => may_hold / 4 * 3,
        };
        card_ledger_capacity_mebibytes(None, configured, budget, may_hold, D)
    }
}

fn room(reading: CardMemoryReading, booked: BookedByThisProcessOnACard, configured: Option<u64>) -> CardRoomForThisProcess {
    let cap = capacity(configured);
    card_room_for_this_process(reading, booked, D, &cap)
}

fn booked(leases: u64, returned: u64) -> BookedByThisProcessOnACard {
    BookedByThisProcessOnACard {
        leases_mebibytes: leases,
        driver_share: DriverShareOnTheCard::DeviceStarted,
        returned_possibly_still_on_the_card_mebibytes: returned,
    }
}

fn mib(n: u64) -> String {
    format!("{n} MiB")
}

/// 甲：一份报告里 FB 空闲与进程那一行不是同一刻（两次 NVML 查询之间这个进程别的线程建上了它借着的那一笔）。
/// 卡上：别的 GPU 用户占 `outsider`，这个进程驱动实占 `x`，借着一笔 `lease` 还没建；建上发生在两次查询之间。
#[test]
fn opus_10y_a_torn_report_counts_a_lease_built_between_the_two_queries_twice() {
    for (configured, outsider, x, lease) in [
        (Some(32000u64), 10000u64, 600u64, 8000u64),
        (Some(32000), 0, 600, 8000),
        (None, 10000, 600, 8000),
        (Some(32000), 16000, 600, 4000),
    ] {
        let free_before = TOTAL - RESERVED - outsider - x;
        let free_after = free_before - lease;
        let before = parse(&report(free_before, &[(ME, &mib(x))]), false).unwrap()[0];
        let after = parse(&report(free_after, &[(ME, &mib(x + lease))]), false).unwrap()[0];
        let torn = parse(&report(free_before, &[(ME, &mib(x + lease))]), false).unwrap()[0];
        let b = booked(lease, 0);
        let (r_before, r_after, r_torn) = (room(before, b, configured), room(after, b, configured), room(torn, b, configured));
        // 卡上此刻真能再给这个进程的：建完之后的空闲，再留驱动长到 D 的那一份（与账的口径同一个：D 算许出去的）
        let truly = free_after.saturating_sub(D - x);
        println!(
            "OPUS10Y-TORN configured={configured:?} outsider={outsider} x={x} lease={lease} room_before={} room_after={} room_torn={} over_by={} free_after_build={free_after} truly_usable={truly} torn_minus_truly={}",
            r_before.room_now_mebibytes, r_after.room_now_mebibytes, r_torn.room_now_mebibytes,
            r_torn.room_now_mebibytes.saturating_sub(r_after.room_now_mebibytes),
            i128::from(r_torn.room_now_mebibytes) - i128::from(truly)
        );
    }
}

/// 乙：不走卡池的设备起了、还没记进 `BUSES_WITH_DEVICES_STARTED_OUTSIDE_THE_CARD_POOL`（`start_the_card` 编完管线才记）的那一刻：
/// 报告里这个进程那一行含那个设备的 `y`，被认成 OnlyThroughTheBooks、当成借据已建上的。与记上之后（AlsoOutsideTheBooks）、
/// 与那个设备不存在时各算一遍；另把「读不到 / 分不出时驱动那一份当已在卡上，只会少借」那一句与列了时比。
#[test]
fn opus_10y_an_outside_device_not_yet_marked_and_the_driver_share_taken_as_on_the_card() {
    for (outsider, x, y, lease) in [(10000u64, 600u64, 500u64, 8000u64), (10000, 600, 1200, 8000), (10000, 2560, 500, 8000)] {
        let configured = Some(32000);
        let free = TOTAL - RESERVED - outsider - x - y;
        let b = booked(lease, 0);
        let unmarked = room(parse(&report(free, &[(ME, &mib(x + y))]), false).unwrap()[0], b, configured);
        let marked = room(parse(&report(free, &[(ME, &mib(x + y))]), true).unwrap()[0], b, configured);
        let not_listed = room(parse(&report(free, &[]), false).unwrap()[0], b, configured);
        // 真能再给的：空闲减还没建的借据，再留驱动长到 D 的那一份
        let truly = free.saturating_sub(lease).saturating_sub(D.saturating_sub(x));
        println!(
            "OPUS10Y-OUTSIDE outsider={outsider} x={x} y={y} lease={lease} free={free} room_unmarked={} room_marked={} room_not_listed={} truly_usable={truly} unmarked_minus_truly={} marked_minus_truly={}",
            unmarked.room_now_mebibytes, marked.room_now_mebibytes, not_listed.room_now_mebibytes,
            i128::from(unmarked.room_now_mebibytes) - i128::from(truly),
            i128::from(marked.room_now_mebibytes) - i128::from(truly)
        );
    }
}

/// 丙：销了账、卡上还没还的那一份 R 照 `m − D` 往下压：驱动实占 x < D 时压过头，被压掉的那截当成活着的借据已建上。
/// 走真的 `GpuMemoryLedger`：借 L1、记设备起好、丢 L1、读一次（压 R）、再借 L2 那一刻能借多少；与 x = D 时比。
#[test]
fn opus_10y_the_returned_share_is_pressed_below_what_is_still_on_the_card_when_the_driver_holds_less_than_d() {
    for (x, l1, l2_probe) in [(600u64, 4000u64, 1u64), (2560, 4000, 1), (300, 6000, 1)] {
        let outsider = 10000u64;
        // 卡上的物理量：每一步改它，读数照它造报告
        let state = Arc::new(Mutex::new((TOTAL - RESERVED - outsider, 0u64))); // (空闲, 这个进程实占)
        let s = Arc::clone(&state);
        let reader: CardMemoryReader = card_memory_reader_of_one_report_per_reading(
            Box::new(move || {
                let (free, used) = *s.lock().unwrap();
                Ok(if used == 0 { report(free, &[]) } else { report(free, &[(ME, &mib(used))]) })
            }),
            ME,
            vec![pci_bus_identifier_without_the_domain(BUS)],
            Box::new(|_bus: &str| false),
        );
        let cap = capacity(Some(32000));
        let ledger = GpuMemoryLedger::new(1, reader, Box::new(move |_p, may_hold| cap(may_hold)), D);
        let first = ledger.book_on_the_card(0, l1).expect("L1");
        ledger.record_the_device_started_on_the_card(0);
        {
            let mut st = state.lock().unwrap();
            st.0 -= x + l1; // 驱动 x 与 L1 建上
            st.1 = x + l1;
        }
        drop(first); // 销账；卡上那 l1 还没还（wgpu 等收拾）
        let pressed = ledger.card_rooms_mebibytes_now(&[0]).unwrap()[0]; // 这一读把 R 压到 min(l1, x + l1 − D)
        let probe = ledger.book_on_the_card(0, l2_probe).expect("探一笔");
        let room_after_press = probe.card_room_mebibytes_when_booked();
        drop(probe);
        let (free, _) = *state.lock().unwrap();
        let truly = free.saturating_sub(D - x.min(D)); // 那 l1 还占着；留驱动长到 D
        println!(
            "OPUS10Y-RETURNED x={x} l1={l1} free_now={free} room_first_read={pressed} room_second_read={room_after_press} truly_usable_while_l1_still_on_card={truly} over_by={}",
            i128::from(room_after_press) - i128::from(truly)
        );
    }
}

/// 丁：空着时能借的上界取 Total − Reserved；卡上有一个常驻的别的用户（显示服务）时，要的量落在「常驻之外真能借到的」与上界之间，
/// 每一次都是此刻借不到（本次未跑），永远不判红。
#[test]
fn opus_10y_a_resident_outsider_makes_a_need_below_the_ceiling_never_fit_and_never_red() {
    let resident = 400u64;
    let empty_but_resident = parse(&report(TOTAL - RESERVED - resident, &[]), false).unwrap()[0];
    let fresh = BookedByThisProcessOnACard { leases_mebibytes: 0, driver_share: DriverShareOnTheCard::NotRegistered, returned_possibly_still_on_the_card_mebibytes: 0 };
    let r = room(empty_but_resident, fresh, None);
    let need = r.room_now_mebibytes + 1;
    println!(
        "OPUS10Y-CEILING resident={resident} room_with_only_the_resident={} ceiling={} need={need} decision={:?}",
        r.room_now_mebibytes, r.ceiling_mebibytes, decide_booking_by_priority(&[r], need)
    );
}

/// 戊：解析的几种变体：同一个进程号列两行（两份上下文或两种类型分开列）加起来；「Not available」；没有 Reserved 那一行；
/// 这个进程号在容器里撞上宿主机上别的进程。
#[test]
fn opus_10y_parser_variants() {
    let twice = parse(&report(20000, &[(ME, "3000 MiB"), (ME, "3000 MiB")]), false);
    let not_available = parse(&report(20000, &[(ME, "Not available in WDDM driver model")]), false);
    let no_reserved = parse(&report(20000, &[]).lines().filter(|line| !line.trim_start().starts_with("Reserved")).map(|line| format!("{line}\n")).collect::<String>(), false);
    let real = std::fs::read_to_string(std::env::var("OPUS10Y_REAL_REPORT").unwrap_or_default()).ok().map(|text| {
        card_memory_readings_of_one_nvidia_smi_report(
            &text,
            ME,
            &["01:00.0".to_string(), "10:00.0".to_string()],
            &|_bus: &str| false,
        )
    });
    println!("OPUS10Y-PARSE twice={twice:?}");
    println!("OPUS10Y-PARSE not_available={not_available:?}");
    println!("OPUS10Y-PARSE no_reserved={no_reserved:?}");
    println!("OPUS10Y-PARSE real_report={real:?}");
}
