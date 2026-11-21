//! 跑前登记第一段的 5.7 本地常量回比与第七节锚点（`run_constants_and_anchors`）：本地写死的常量逐个与 `crates::` 同名常量比对，不等就停（V3）。

use super::*;

// ============================================================================
// 一、5.7 装置要本地化的常量：本地写死，逐个与 `crates::` 同名常量比对，不等就停（V3）。
//    这些常量本身只在 `run_constants_and_anchors` 里与 `crates::` 比对；构造几何用的是下面
//    「二、几何」里的本地字面量，不从这里或 `crates::` 读。
// ============================================================================

/// 一条常量回比的结果行。
pub(super) struct ConstantCheck {
    pub(super) name: &'static str,
    pub(super) local_value: u64,
    pub(super) crates_value: u64,
}

impl ConstantCheck {
    pub(super) fn matches(&self) -> bool {
        self.local_value == self.crates_value
    }
}

/// 系统配置字段表合计本地按臂的定义现算（跑前登记 5.7「乙族各臂的系统配置字段表比 481 + 8」）：
/// 环境变量 `SINGLEFS_E158_LOCAL_SYSTEM_CONFIGURATION_BYTES` 给了就用它，没给按今天那一份取 489——
/// D22（单元原子性怎么合成） 已定项 9 的字段表合计 489（系统运行量末尾加了回退下界 F 8 字节）；
/// 跑前登记写的 481 是登记那天的今天。乙族各臂这时要传几，归 E158（择根与修复四岔路） 的重跑登记定。
/// 装置源码各臂同一份（跑前登记「装置写在哪」），差别只在跑的时候传不传这个环境变量，不是另分支代码。
pub(super) fn local_system_configuration_bytes() -> u64 {
    parse_local_system_configuration_bytes(env::var(
        "SINGLEFS_E158_LOCAL_SYSTEM_CONFIGURATION_BYTES",
    ))
}

/// `local_system_configuration_bytes` 的纯函数一半，与 `crash_injection.rs` 里
/// `CrashInjectingWorkerThreads::from_the_environment_value` 同一个理由把环境变量的读结果
/// 当参数传进来：单测不碰真环境变量（并发跑的单测共用同一个进程环境，互相踩）。
fn parse_local_system_configuration_bytes(value: Result<String, env::VarError>) -> u64 {
    match value {
        Ok(value) => value
            .parse()
            .unwrap_or_else(|error| panic!("SINGLEFS_E158_LOCAL_SYSTEM_CONFIGURATION_BYTES={value:?} 不是十进制正整数: {error}")),
        Err(env::VarError::NotPresent) => 489,
        Err(error) => panic!("SINGLEFS_E158_LOCAL_SYSTEM_CONFIGURATION_BYTES 读不出: {error}"),
    }
}

/// 5.7 表：本地常量与 `crates::` 那一份逐条比对（V3：任一不等，整轮不开跑）。
pub(super) fn local_constants_checks() -> Vec<ConstantCheck> {
    vec![
        ConstantCheck {
            name: "根环区域数 ROOT_RING_REGIONS",
            local_value: 3,
            crates_value: singlefs_format::ROOT_RING_REGIONS,
        },
        ConstantCheck {
            name: "每区槽数 mkfs 默认 ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM",
            local_value: 8,
            crates_value: singlefs_format::ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM,
        },
        ConstantCheck {
            name: "每区槽数下界 ROOT_RING_SLOTS_PER_REGION_MINIMUM",
            local_value: 4,
            crates_value: singlefs_format::ROOT_RING_SLOTS_PER_REGION_MINIMUM,
        },
        ConstantCheck {
            name: "每区槽数上界 ROOT_RING_SLOTS_PER_REGION_MAXIMUM",
            local_value: 16,
            crates_value: singlefs_format::ROOT_RING_SLOTS_PER_REGION_MAXIMUM,
        },
        ConstantCheck {
            name: "journal 默认环长 JOURNAL_RING_DEFAULT_BYTES",
            local_value: 768 * 1024 * 1024,
            crates_value: singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
        },
        ConstantCheck {
            name: "journal 记录宽 JOURNAL_RECORD_BYTES",
            local_value: 4096,
            crates_value: singlefs_format::JOURNAL_RECORD_BYTES,
        },
        ConstantCheck {
            name: "journal 环起点槽 JOURNAL_RING_START_SLOT",
            local_value: 1024,
            crates_value: singlefs_format::JOURNAL_RING_START_SLOT,
        },
        ConstantCheck {
            name: "落点粒度 SLOT_BYTES",
            local_value: 16384,
            crates_value: singlefs_format::SLOT_BYTES,
        },
        ConstantCheck {
            name: "节点宽 NODE_BYTES",
            local_value: 16384,
            crates_value: singlefs_format::NODE_BYTES,
        },
        ConstantCheck {
            name: "数据单元宽 DATA_UNIT_BYTES",
            local_value: 32768,
            crates_value: singlefs_format::DATA_UNIT_BYTES,
        },
        ConstantCheck {
            name: "系统配置字段表合计 SYSTEM_CONFIGURATION_BYTES",
            local_value: local_system_configuration_bytes(),
            crates_value: singlefs_format::SYSTEM_CONFIGURATION_BYTES,
        },
        ConstantCheck {
            name: "系统配置槽宽 SYSTEM_CONFIGURATION_SLOT_BYTES",
            local_value: 4096,
            crates_value: singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES,
        },
        ConstantCheck {
            name: "系统配置每盘槽数 SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE",
            local_value: 2,
            crates_value: singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE,
        },
        ConstantCheck {
            name: "根记录宽 ROOT_RECORD_BYTES",
            local_value: 466,
            crates_value: singlefs_format::ROOT_RECORD_BYTES,
        },
        ConstantCheck {
            name: "实例表行宽 INSTANCE_ROW_BYTES",
            local_value: 88,
            crates_value: singlefs_format::INSTANCE_ROW_BYTES,
        },
        ConstantCheck {
            name: "实例表一片记录数 INSTANCE_TABLE_PAGE_RECORDS",
            local_value: 370,
            crates_value: singlefs_format::INSTANCE_TABLE_PAGE_RECORDS,
        },
        ConstantCheck {
            name: "系统配置整槽校验和偏移（装置自己的算术 4+2+96+16+20+1+4+4+8，与 core 的私有算法比）",
            local_value: 4 + 2 + 96 + 16 + 20 + 1 + 4 + 4 + 8,
            crates_value: u64::try_from(singlefs_core::system_configuration::SYSTEM_CONFIGURATION_CHECKSUM_OFFSET)
                .expect("155 装得进 u64"),
        },
    ]
}

/// 「三个区域住哪块盘」：登记表说「照 `crates/` 那一份抄」——这里抄的是值（0, 1, 0），
/// 不是从 `crates::` 活着读这个数组来决定 mkfs 参数；构造 `MakeFilesystemParameters` 时
/// 用的是下面 `region_devices()` 里同一份字面量，两处各自维护、在这里比对。
pub(super) fn region_devices_matches_crates() -> (bool, [u32; 3], [u32; 3]) {
    let local: [u32; 3] = [0, 1, 0];
    let crates_value =
        singlefs_core::make_filesystem::FIRST_VERSION_REGION_DEVICES.map(|identity| identity.0);
    (local == crates_value, local, crates_value)
}

// ============================================================================
// 九、5.7 常量回比 + 第七节锚点：开跑前先做，任一常量不等就停（V3）。
// ============================================================================

pub(super) fn run_constants_and_anchors() -> bool {
    let mut all_ok = true;
    for check in local_constants_checks() {
        let ok = check.matches();
        all_ok &= ok;
        emit_result(&format!(
            "name=local_constant_check item={:?} verdict={} local_value={} crates_value={}",
            check.name,
            if ok { "pass" } else { "fail" },
            check.local_value,
            check.crates_value
        ));
    }
    let (region_ok, local_regions, crates_regions) = region_devices_matches_crates();
    all_ok &= region_ok;
    emit_result(&format!(
        "name=local_constant_check item=region_devices verdict={} local_value={local_regions:?} crates_value={crates_regions:?}",
        if region_ok { "pass" } else { "fail" }
    ));

    let arithmetic_checks: [(&str, i64, i64); 19] = [
        ("系统配置槽余量 4096-481", 4096 - 481, 3615),
        ("512-481", 512 - 481, 31),
        ("乙族字段表 481+8", 481 + 8, 489),
        ("512-489", 512 - 489, 23),
        ("4096-489", 4096 - 489, 3607),
        ("字段表四档 389+4+36+52", 389 + 4 + 36 + 52, 481),
        ("journal tail 与实例代号 469+8+4", 469 + 8 + 4, 481),
        (
            "系统配置整槽校验和偏移 4+2+96+16+20+1+4+4+8",
            4 + 2 + 96 + 16 + 20 + 1 + 4 + 4 + 8,
            155,
        ),
        ("实例表行宽 1+4+8+8+1+66", 1 + 4 + 8 + 8 + 1 + 66, 88),
        ("实例表一片行数 370-1", 370 - 1, 369),
        ("今天那一臂系统配置那一类写字节 4096*2*1", 4096 * 2, 8192),
        // 岔路单第 2 行 ③：四条修法与岔路 3 候选 2 抢不抢系统配置槽空间。
        // 候选 2 表形态宽度取 Q3-2 已量出的 N_w=4（S4/L=4 那一点，四个几何点里最大的一个）。
        // 甲（甲-txg、甲-jsn）与丙（今天）都不改系统配置字段表（仍是 481），与候选 2 同放时
        // 余量按「单放」算（`512 - 481 - W`，跑前登记 Q3-2 定义原句）；乙、丁都把字段表改成 489
        // （同一处 8 字节字段，只是取号时写占位 0 还是续传，写法宽度相同），余量按「与乙同放」算
        // （`512 - 481 - 8 - W`）。
        ("候选2表形态 1+16*N_w(N_w=4)", 1 + 16 * 4, 65),
        ("候选2表形态 1+12*N_w(N_w=4)", 1 + 12 * 4, 49),
        ("甲/丙同放候选2单条16 512-481-16", 512 - 481 - 16, 15),
        ("甲/丙同放候选2单条12 512-481-12", 512 - 481 - 12, 19),
        ("甲/丙同放候选2表形态65 512-481-65", 512 - 481 - 65, -34),
        ("甲/丙同放候选2表形态49 512-481-49", 512 - 481 - 49, -18),
        ("乙/丁同放候选2表形态65 512-489-65", 512 - 489 - 65, -42),
        ("乙/丁同放候选2表形态49 512-489-49", 512 - 489 - 49, -26),
    ];
    for (name, computed, expected) in arithmetic_checks {
        let ok = computed == expected;
        all_ok &= ok;
        emit_result(&format!(
            "name=section_seven_two_arithmetic item={name:?} verdict={} computed={computed} registered={expected}",
            if ok { "pass" } else { "fail" }
        ));
    }
    for slots_per_region in [4u64, 8, 16] {
        let total = 3 * slots_per_region;
        let expected = match slots_per_region {
            4 => 12,
            8 => 24,
            16 => 48,
            _ => unreachable!(),
        };
        let ok = total == expected;
        all_ok &= ok;
        emit_result(&format!(
            "name=root_ring_slot_total slots_per_region={slots_per_region} verdict={} computed={total} registered={expected}",
            if ok { "pass" } else { "fail" }
        ));
        let root_slots_with_a_root = total;
        let combinations =
            1 + root_slots_with_a_root + root_slots_with_a_root * (root_slots_with_a_root - 1) / 2;
        let expected_combinations = match slots_per_region {
            4 => 79,
            8 => 301,
            16 => 1177,
            _ => unreachable!(),
        };
        let combinations_ok = combinations == expected_combinations;
        all_ok &= combinations_ok;
        emit_result(&format!(
            "name=phi_three_fault_set_count slots_per_region={slots_per_region} root_slots_with_a_root={root_slots_with_a_root} verdict={} computed={combinations} registered={expected_combinations}",
            if combinations_ok { "pass" } else { "fail" }
        ));
    }
    for (ring_bytes, expected_records, expected_in_flight, label) in [
        (768u64 * 1024 * 1024, 196_608u64, 65_536u64, "768MiB"),
        (3 * 1024 * 1024, 768, 256, "3MiB"),
        (48 * 1024, 12, 4, "48KiB"),
    ] {
        let records = ring_bytes / 4096;
        let in_flight = records / 3;
        let ok = records == expected_records && in_flight == expected_in_flight;
        all_ok &= ok;
        emit_result(&format!(
            "name=journal_ring_records_and_in_flight_limit label={label} verdict={} computed_records={records} computed_in_flight={in_flight} registered_records={expected_records} registered_in_flight={expected_in_flight}",
            if ok { "pass" } else { "fail" }
        ));
    }

    // 写成加法不写减法（`.claude/rules/test-discipline.md`「常量断言写加法，别写减法」）：
    // 减法形态 `4096 - 481 == 3615` 把常量改大会在编译期溢出，变异记成「无效」而不是「被抓到」。
    // 两边都是字面量，clippy 判「等价表达式」——这条钉的正是「这三个字面量今天互相对得上」这件事本身
    // （M6/M7 变异表改的就是这一类字面量），不是一次运行时判断，允许它。
    #[allow(
        clippy::eq_op,
        reason = "两边都是字面量，钉的是这三个字面量互相对得上，不是运行时判断"
    )]
    let section_seven_one_anchor_holds = 4096 == 481 + 3615;
    all_ok &= section_seven_one_anchor_holds;
    emit_result(&format!(
        "name=section_seven_one_anchor item=\"D22 已定项 9：系统配置字段表合计 481 字节、槽内余 3615、今天没跨 512\" verdict={} checked_in_this_segment=true",
        if section_seven_one_anchor_holds { "pass" } else { "fail" }
    ));
    emit_result("name=section_seven_one_anchor item=\"C332 正文：2 个故障\" verdict=deferred checked_in_this_segment=false deferred_to=Q3-1_第二段");
    emit_result("name=section_seven_one_anchor item=\"C393 正文：被抛弃根的账读不出时今天只计数、不拒绝挂载\" verdict=deferred checked_in_this_segment=false deferred_to=Q1-1c_第四段");
    emit_result("name=section_seven_one_anchor item=\"D23 已定项 14：N_switch=3\" verdict=deferred checked_in_this_segment=false deferred_to=第五段_前置未满足");
    emit_result("name=section_seven_one_anchor item=\"C334 判别力自证：3/3/各1\" verdict=deferred checked_in_this_segment=false deferred_to=第五段_前置未满足");
    emit_result("name=section_seven_three_row item=\"PC2/PC1-a/PC1-b/Q1-3\" verdict=deferred checked_in_this_segment=false deferred_to=第三_第四段");

    all_ok
}

#[cfg(test)]
mod tests;
