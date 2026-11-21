//! 第 2 次跑的 5.6 常量回比与第七节锚点。

use super::*;

// ============================================================================
// 十、第 2 次跑（`research/prompts/e158-r2-prereg.md`）：5.6 常量回比与第七节锚点（第 2 次跑那一份）、
//    H1d（5.5 末尾主 agent 跑前修订加的历史族）、Q1-0（H1c 的抛弃步，前提 1 在装置上的读数）、
//    实七两段复现历史。这一节只加：第一次跑的各模式（上面一至九）原样留着，它们的旧产物由
//    `scripts/experiment-apparatus/replay.sh` 里 E158 那几行复跑；第一节处理表里的删与改归后面的段。
// ============================================================================

/// 这一次跑在哪一条臂的副本上：只当标签打进结果行。装置源码各臂同一份（跑前登记「装置写在哪」），
/// 差别只在副本的 `crates/` 与跑的时候设的这个环境变量；没设就是工作树（今天那一臂）。
pub(super) fn second_run_arm_label() -> String {
    env::var("SINGLEFS_E158_ARM").unwrap_or_else(|_| "today".to_string())
}

/// 系统配置字段表合计今天的值（D22（单元原子性怎么合成） 已定项 9；跑前登记 5.6 表「系统配置字段表合计」一行）。
pub(super) const SYSTEM_CONFIGURATION_BYTES_TODAY_LOCAL: u64 = 489;
/// 系统配置槽宽（跑前登记 5.6；`singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES` 回比）。
pub(super) const SYSTEM_CONFIGURATION_SLOT_BYTES_LOCAL: u64 = 4096;
/// 字段表四档（D22 已定项 9 的分段表；跑前登记 7.2 第四行）：不可变 389、可变 4、运行配置 36、运行量 60（今天）。
const IMMUTABLE_TIER_BYTES_LOCAL: u64 = 389;
const MUTABLE_TIER_BYTES_LOCAL: u64 = 4;
const RUNTIME_CONFIGURATION_TIER_BYTES_LOCAL: u64 = 36;
const RUNTIME_QUANTITIES_TIER_BYTES_TODAY_LOCAL: u64 = 60;
/// 一次准入最多几次发布（D16（发布语义） 已定项 1「准入」：B = 4 + 2 k_tol，k_tol = 2；跑前登记 5.6）。
const PUBLISHES_PER_ADMISSION_AT_MOST_LOCAL: u64 = 4 + 2 * 2;
/// 512 字节物理写原子边界：`crates/` 里没有，出处 D22 已定项 9 正文，条款锚不回比（跑前登记 5.6）。
const PHYSICAL_WRITE_ATOMIC_BOUNDARY_BYTES: u64 = 512;
/// 落点粒度（跑前登记 5.6 `SLOT_BYTES`）：第 2 次跑算单元字节区间用的就是这一个。
pub(super) const SLOT_BYTES_LOCAL: u64 = 16384;
/// 这个测试周期的种子基（抄自 `crates/singlefs-verification/src/history.rs` 的 `SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE`，
/// 开跑时回比）；实七报告「交主 agent 的」第 1 条点名的两段历史都从它数（`research/prompts/m2-impl7-implementer-report.md`）。
pub(super) const SEED_BASE_OF_THIS_TEST_CYCLE_LOCAL: u64 = 7_463_871_032_432_355_113;
/// 故障注入 512 段那一批里的一段：种子基 + 110、每段 30 步、注入 6 次（当时设变量跑的那一批的规模，
/// `crates/singlefs-verification/tests/fault_injection_campaign.rs` 的故障注入入口）。
pub(super) const SEVENTH_BATCH_FAULT_REPRODUCTION_SEED_OFFSET: u64 = 110;
pub(super) const SEVENTH_BATCH_FAULT_REPRODUCTION_OPERATIONS: usize = 30;
pub(super) const SEVENTH_BATCH_FAULT_REPRODUCTION_FAULTS: usize = 6;
/// 崩溃注入默认批那一段：种子基 + 16、每段 24 步、每段抽 4 个崩溃状态（默认批规模，
/// `crates/singlefs-verification/tests/crash_injection_campaign.rs` 的 `DEFAULT_BATCH_*`）。
pub(super) const SEVENTH_BATCH_CRASH_REPRODUCTION_SEED_OFFSET: u64 = 16;
pub(super) const SEVENTH_BATCH_CRASH_REPRODUCTION_OPERATIONS: usize = 24;
pub(super) const SEVENTH_BATCH_CRASH_REPRODUCTION_CRASH_POINTS: usize = 4;

/// 跑前登记 5.6 表：第一次跑那一份（`local_constants_checks`）之外，第 2 次跑另要回比的几项。
pub(super) fn second_run_local_constants_checks() -> Vec<ConstantCheck> {
    let mut checks = local_constants_checks();
    checks.extend([
        ConstantCheck {
            name: "系统配置槽宽（第 2 次跑的锚点用的那一个）",
            local_value: SYSTEM_CONFIGURATION_SLOT_BYTES_LOCAL,
            crates_value: singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES,
        },
        ConstantCheck {
            name: "字段表不可变一档 SystemImmutableConfiguration::FIELD_TABLE_BYTES",
            local_value: IMMUTABLE_TIER_BYTES_LOCAL,
            crates_value: singlefs_core::system_configuration::SystemImmutableConfiguration::FIELD_TABLE_BYTES,
        },
        ConstantCheck {
            name: "字段表可变一档 SystemMutableConfiguration::FIELD_TABLE_BYTES",
            local_value: MUTABLE_TIER_BYTES_LOCAL,
            crates_value: singlefs_core::system_configuration::SystemMutableConfiguration::FIELD_TABLE_BYTES,
        },
        ConstantCheck {
            name: "字段表运行配置一档 SystemRuntimeConfiguration::FIELD_TABLE_BYTES",
            local_value: RUNTIME_CONFIGURATION_TIER_BYTES_LOCAL,
            crates_value: singlefs_core::system_configuration::SystemRuntimeConfiguration::FIELD_TABLE_BYTES,
        },
        ConstantCheck {
            name: "字段表运行量一档 SystemRuntimeQuantities::FIELD_TABLE_BYTES（乙 / 丁四臂按臂的定义多 8）",
            local_value: RUNTIME_QUANTITIES_TIER_BYTES_TODAY_LOCAL + local_system_configuration_bytes()
                - SYSTEM_CONFIGURATION_BYTES_TODAY_LOCAL,
            crates_value: singlefs_core::system_configuration::SystemRuntimeQuantities::FIELD_TABLE_BYTES,
        },
        ConstantCheck {
            name: "一次准入最多几次发布 PUBLISHES_PER_ADMISSION_AT_MOST",
            local_value: PUBLISHES_PER_ADMISSION_AT_MOST_LOCAL,
            crates_value: u64::try_from(singlefs_core::mount::PUBLISHES_PER_ADMISSION_AT_MOST)
                .expect("8 装得进 u64"),
        },
        ConstantCheck {
            name: "落点粒度（第 2 次跑算单元字节区间用的那一个）",
            local_value: SLOT_BYTES_LOCAL,
            crates_value: singlefs_format::SLOT_BYTES,
        },
        ConstantCheck {
            name: "这个测试周期的种子基 SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE",
            local_value: SEED_BASE_OF_THIS_TEST_CYCLE_LOCAL,
            crates_value: singlefs_verification::history::SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        },
    ]);
    checks
}

/// 在一份系统配置槽的字节里找一段小端字节恰好出现一次的位置；不是恰好一次交 None。
fn unique_offset_of(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    let mut found = None;
    for start in 0..=haystack.len().saturating_sub(needle.len()) {
        if haystack[start..start + needle.len()] == *needle {
            if found.is_some() {
                return None;
            }
            found = Some(start);
        }
    }
    found
}

/// journal tail、实例代号、回退下界 F 三项在系统配置槽里住哪（跑前登记 5.6「私有常量引不到 ⇒ 装置写一槽已知的 tail、
/// 实例代号与 F，自己解码找偏移」）：mkfs 一个池，按 `crates` 的解析读回盘 0 第 0 槽，改三项为已知值、再按 `crates` 写成槽字节，
/// 按小端字节找它们。交回 (tail, 实例代号, F) 三个偏移。
pub(super) fn decoded_tail_instance_and_floor_offsets() -> Result<(usize, usize, usize), String> {
    let parameters = parameters_for(&GEOMETRY_PRIMARY);
    let mut devices = new_devices(IMAGE_BYTES);
    make_filesystem(&parameters, &mut devices).map_err(|error| format!("mkfs: {error:?}"))?;
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES_LOCAL).expect("4096");
    let mut slot = vec![0u8; slot_bytes];
    devices[0]
        .1
        .read_at(DeviceOffsetInBytes(0), &mut slot)
        .map_err(|error| format!("读系统配置槽: {error:?}"))?;
    let mut parsed = singlefs_core::system_configuration::SystemConfiguration::parse_slot(&slot)
        .map_err(|error| format!("解系统配置槽: {error:?}"))?;
    let known_tail: u64 = 0x1122_3344_5566_7788;
    let known_instance: u32 = 0xA1B2_C3D4;
    let known_floor: u64 = 0x0102_0304_0506_0708;
    parsed.quantities.journal_tail = known_tail;
    parsed.quantities.journal_instance = InstanceGeneration(known_instance);
    parsed.quantities.rollback_floor = CheckpointTxg(known_floor);
    let written = parsed.to_slot();
    let tail = unique_offset_of(&written, &known_tail.to_le_bytes())
        .ok_or("已知 tail 在槽里不是恰好一处")?;
    let instance = unique_offset_of(&written, &known_instance.to_le_bytes())
        .ok_or("已知实例代号在槽里不是恰好一处")?;
    let floor = unique_offset_of(&written, &known_floor.to_le_bytes())
        .ok_or("已知 F 在槽里不是恰好一处")?;
    Ok((tail, instance, floor))
}

/// 跑前登记 7.2：独立算出、用命令核过的值（登记第十三节那条 python 的输出），写成「由本地常量现算 == 字面值」，
/// 常量断言写加法不写减法（`test-discipline.md`「常量断言写加法」）。乙 / 丁四臂（本地字段表 497）那几行换成 7.2 第三行的数。
fn second_run_arithmetic_anchors(local_system_configuration: u64) -> Vec<(&'static str, u64, u64)> {
    let mut anchors = vec![
        (
            "字段表四档 389+4+36+60（7.2 第四行）",
            IMMUTABLE_TIER_BYTES_LOCAL
                + MUTABLE_TIER_BYTES_LOCAL
                + RUNTIME_CONFIGURATION_TIER_BYTES_LOCAL
                + RUNTIME_QUANTITIES_TIER_BYTES_TODAY_LOCAL,
            489,
        ),
        (
            "运行量一档 8+32+8+4+8（7.2 第四行）",
            8 + 32 + 8 + 4 + 8,
            60,
        ),
        (
            "乙 / 丁字段表 489+8（7.2 第三行）",
            SYSTEM_CONFIGURATION_BYTES_TODAY_LOCAL + 8,
            497,
        ),
        (
            "系统配置整槽校验和偏移 4+2+96+16+20+1+4+4+8（7.2 第六行）",
            4 + 2 + 96 + 16 + 20 + 1 + 4 + 4 + 8,
            155,
        ),
        ("根槽总数 3×S，S=4（7.2 第七行）", 3 * 4, 12),
        (
            "根槽总数 3×S，S=8（7.2 第七行）",
            3 * GEOMETRY_PRIMARY.slots_per_region,
            24,
        ),
        (
            "根槽总数 3×S，S=16（7.2 第七行）",
            3 * GEOMETRY_LARGER_ROOT_RING.slots_per_region,
            48,
        ),
        (
            "环槽数 768MiB÷4096（7.2 第八行）",
            (768 * 1024 * 1024) / JOURNAL_RECORD_BYTES,
            196_608,
        ),
        (
            "环槽数 3MiB÷4096（7.2 第八行）",
            GEOMETRY_PRIMARY.journal_ring_bytes / JOURNAL_RECORD_BYTES,
            768,
        ),
        (
            "环槽数 48KiB÷4096（7.2 第八行）",
            (48 * 1024) / JOURNAL_RECORD_BYTES,
            12,
        ),
        (
            "实例表行宽 1+4+8+8+1+66（7.2 第九行）",
            1 + 4 + 8 + 8 + 1 + 66,
            88,
        ),
        (
            "两个指称各一个索引节点两份 16384×2（7.2 第十一行）",
            SLOT_BYTES_LOCAL * 2,
            32768,
        ),
        ("Φ1 每条根 T、L 都有时 3+3+1（7.2 第十二行）", 3 + 3 + 1, 7),
        (
            "今天那一臂每次发布系统配置那一类写字节 4096×2（7.2 第十行）",
            SYSTEM_CONFIGURATION_SLOT_BYTES_LOCAL * 2,
            8192,
        ),
    ];
    if local_system_configuration == SYSTEM_CONFIGURATION_BYTES_TODAY_LOCAL {
        anchors.push((
            "系统配置槽余量：槽宽 == 字段表 + 3607（7.1 第一行、7.2 第一行）",
            SYSTEM_CONFIGURATION_SLOT_BYTES_LOCAL,
            local_system_configuration + 3607,
        ));
        anchors.push((
            "512 == 字段表 + 23（7.2 第二行）",
            PHYSICAL_WRITE_ATOMIC_BOUNDARY_BYTES,
            local_system_configuration + 23,
        ));
    } else {
        anchors.push((
            "乙 / 丁：槽宽 == 字段表 + 3599（7.2 第三行）",
            SYSTEM_CONFIGURATION_SLOT_BYTES_LOCAL,
            local_system_configuration + 3599,
        ));
        anchors.push((
            "乙 / 丁：512 == 字段表 + 15（7.2 第三行）",
            PHYSICAL_WRITE_ATOMIC_BOUNDARY_BYTES,
            local_system_configuration + 15,
        ));
    }
    anchors
}

/// 第 2 次跑的开跑检查：5.6 常量回比（V3）、7.2 算术（V4）、7.1 第一行里「字段表不跨 512」与偏移解码（S3）。
/// 交回 true = 全部过；任一不过交 false，调用方整轮不开跑。
pub(super) fn run_second_run_constants_and_anchors(arm: &str) -> bool {
    let mut all_passed = true;
    for check in second_run_local_constants_checks() {
        let passed = check.matches();
        all_passed &= passed;
        emit_result(&format!(
            "name=r2_local_constant_check arm={arm} item={:?} verdict={} local_value={} crates_value={}",
            check.name,
            if passed { "pass" } else { "fail" },
            check.local_value,
            check.crates_value
        ));
    }
    let (regions_match, local_regions, crates_regions) = region_devices_matches_crates();
    all_passed &= regions_match;
    emit_result(&format!(
        "name=r2_local_constant_check arm={arm} item=region_devices verdict={} local_value={local_regions:?} crates_value={crates_regions:?}",
        if regions_match { "pass" } else { "fail" }
    ));
    let local_system_configuration = local_system_configuration_bytes();
    for (item, computed, registered) in second_run_arithmetic_anchors(local_system_configuration) {
        let passed = computed == registered;
        all_passed &= passed;
        emit_result(&format!(
            "name=r2_section_seven_two_anchor arm={arm} item={item:?} verdict={} computed={computed} registered={registered}",
            if passed { "pass" } else { "fail" }
        ));
    }
    let within_one_physical_write =
        local_system_configuration <= PHYSICAL_WRITE_ATOMIC_BOUNDARY_BYTES;
    all_passed &= within_one_physical_write;
    emit_result(&format!(
        "name=r2_section_seven_one_anchor arm={arm} item=\"D22 已定项 9：字段表不跨 512\" verdict={} local_system_configuration_bytes={local_system_configuration}",
        if within_one_physical_write { "pass" } else { "fail" }
    ));
    match decoded_tail_instance_and_floor_offsets() {
        Ok((tail, instance, floor)) => {
            let tail_passed = tail == 469;
            let floor_passed = floor == 481 && instance + 4 == floor && tail + 8 == instance;
            all_passed &= tail_passed && floor_passed;
            emit_result(&format!(
                "name=r2_decoded_offsets arm={arm} verdict={} tail_offset={tail} instance_offset={instance} floor_offset={floor} registered_tail=469 registered_floor=481",
                if tail_passed && floor_passed { "pass" } else { "fail" }
            ));
        }
        Err(error) => {
            all_passed = false;
            emit_result(&format!(
                "name=r2_decoded_offsets arm={arm} verdict=fail reason={error:?}"
            ));
        }
    }
    all_passed
}

#[cfg(test)]
mod tests;
