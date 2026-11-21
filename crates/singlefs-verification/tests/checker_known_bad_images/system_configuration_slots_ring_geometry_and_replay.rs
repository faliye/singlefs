//! 系统配置槽的代号轮换、日志环几何与根槽对齐、尾槽见证、重放出的版本与在途记录上限。

use super::*;

/// I-7.16 只破差值那一格（`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 13 行）：盘 0 没被择的那一槽世代号
/// 改成被择那个减 3（仍比它小、择哪一槽不变，与被择那个奇偶相反——住的槽对），整槽校验和重封：两槽相差 3。
fn lower_the_generation_of_the_unchosen_slot_of_device_zero_by_three(image: &mut MemoryPool) {
    let [(_, chosen_generation), (unchosen_offset, unchosen_generation)] =
        system_configuration_slots_chosen_first(image, 0);
    assert_eq!(
        chosen_generation,
        unchosen_generation + 1,
        "干净镜像上盘 0 两槽世代号相差 1"
    );
    assert!(
        chosen_generation >= 3,
        "盘 0 被择那个槽的世代号 {chosen_generation} ≥ 3，减 3 不越过 0"
    );
    mutate_system_configuration_slot(image, 0, unchosen_offset, |bytes| {
        set_u64(
            bytes,
            SYSTEM_CONFIGURATION_SLOT_GENERATION_OFFSET,
            chosen_generation - 3,
        );
    });
}

/// I-7.16 只破奇偶那一格：盘 0 被择那一槽世代号加 1、没被择那一槽改成原来被择的那个世代号（择哪一槽不变，两槽仍相差 1），
/// 整槽校验和各自重封：两槽的世代号都 ≥ 2 而都住在对方的槽里。
fn swap_the_parities_of_the_two_system_configuration_slots_of_device_zero(image: &mut MemoryPool) {
    let [(chosen_offset, chosen_generation), (unchosen_offset, unchosen_generation)] =
        system_configuration_slots_chosen_first(image, 0);
    assert_eq!(
        chosen_generation,
        unchosen_generation + 1,
        "干净镜像上盘 0 两槽世代号相差 1"
    );
    assert!(
        unchosen_generation >= 1,
        "没被择那个槽改成 {chosen_generation} 之后也 ≥ 2，奇偶那一格判得到"
    );
    mutate_system_configuration_slot(image, 0, chosen_offset, |bytes| {
        set_u64(
            bytes,
            SYSTEM_CONFIGURATION_SLOT_GENERATION_OFFSET,
            chosen_generation + 1,
        );
    });
    mutate_system_configuration_slot(image, 0, unchosen_offset, |bytes| {
        set_u64(
            bytes,
            SYSTEM_CONFIGURATION_SLOT_GENERATION_OFFSET,
            chosen_generation,
        );
    });
}

/// I-7.16 的两份坏镜像，交给「每条不变量至少有一份坏镜像」那一处数清单；判定各在一条用例里。
pub(super) fn known_bad_images_of_the_system_configuration_slot_generations(
) -> Vec<(&'static str, Mutation)> {
    vec![
        (
            SYSTEM_CONFIGURATION_SLOT_GENERATIONS_FOLLOW_THE_ROTATION,
            Box::new(lower_the_generation_of_the_unchosen_slot_of_device_zero_by_three),
        ),
        (
            SYSTEM_CONFIGURATION_SLOT_GENERATIONS_FOLLOW_THE_ROTATION,
            Box::new(swap_the_parities_of_the_two_system_configuration_slots_of_device_zero),
        ),
    ]
}

/// 一份 I-7.16 坏镜像只红 I-7.16，第一处违例落在 `cell_in_the_detail` 说的那一格上；干净镜像上 I-7.16 真被评估过且成立。
fn assert_only_the_rotation_invariant_reddens_on_the_cell(
    tag: &str,
    mutation: impl Fn(&mut MemoryPool),
    cell_in_the_detail: &str,
) {
    let clean = build_pool(tag).memory_pool();
    assert_eq!(
        verdict(
            &clean,
            SYSTEM_CONFIGURATION_SLOT_GENERATIONS_FOLLOW_THE_ROTATION
        ),
        InvariantVerdict::Holds,
        "干净镜像上 I-7.16 真被评估过且成立"
    );
    let mut image = clean.clone();
    mutation(&mut image);
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [SYSTEM_CONFIGURATION_SLOT_GENERATIONS_FOLLOW_THE_ROTATION],
        "判红的该只有 I-7.16：{verdicts:?}"
    );
    match verdict(
        &image,
        SYSTEM_CONFIGURATION_SLOT_GENERATIONS_FOLLOW_THE_ROTATION,
    ) {
        InvariantVerdict::Violated(detail) => assert!(
            detail.contains(cell_in_the_detail),
            "I-7.16 红在「{cell_in_the_detail}」那一格：{detail}"
        ),
        other @ (InvariantVerdict::Holds
        | InvariantVerdict::NotApplicable(_)
        | InvariantVerdict::PartiallyJudged(_)) => {
            panic!("I-7.16 该判违例，实际 {other:?}")
        }
    }
}

/// 两槽相差 3、各自住的槽都对：只红 I-7.16 的差值那一格（奇偶那一格照旧成立）。
#[test]
fn system_configuration_slot_generations_three_apart_in_their_own_slots_redden_only_the_rotation_invariant_on_the_difference(
) {
    assert_only_the_rotation_invariant_reddens_on_the_cell(
        "known-bad-system-configuration-generation-difference",
        lower_the_generation_of_the_unchosen_slot_of_device_zero_by_three,
        "相差不是 1",
    );
}

/// 两槽相差 1、都住在对方的槽里：只红 I-7.16 的奇偶那一格（差值那一格照旧成立）。
#[test]
fn system_configuration_slot_generations_one_apart_in_each_others_slot_redden_only_the_rotation_invariant_on_the_parity(
) {
    assert_only_the_rotation_invariant_reddens_on_the_cell(
        "known-bad-system-configuration-generation-parity",
        swap_the_parities_of_the_two_system_configuration_slots_of_device_zero,
        "该住槽",
    );
}

/// 系统配置字段表里在飞记录数上限（4 字节）、最坏占用（8 字节）、安全系数 F（4 字节）、io_min（4 字节）的偏移
/// （`.claude/kb/layout/01-first-txn.md` 一：环长 333 + 8 → 记录尺寸 341 + 4 → 在飞上限 345 → 最坏占用 349 → F 357 → R 361；io_min 在 425）。
const SYSTEM_CONFIGURATION_IN_FLIGHT_RECORD_LIMIT_OFFSET: usize = 345;

const SYSTEM_CONFIGURATION_WORST_CASE_OCCUPANCY_OFFSET: usize = 349;

const SYSTEM_CONFIGURATION_SAFETY_FACTOR_OFFSET: usize = 357;

const SYSTEM_CONFIGURATION_MINIMUM_INPUT_OUTPUT_OFFSET: usize = 425;

/// 根槽距 4096 的两倍：io_min 声明成它，判定宽度跟着成 8192，槽 1（区域起点 + 4096）不是它的整数倍。
const MINIMUM_INPUT_OUTPUT_TWICE_THE_ROOT_SLOT_SPACING: u32 = 8192;

/// 全池四个系统配置槽（两块盘各两槽）：(盘, 偏移)。槽 1 的偏移按槽 0 记的槽距取。
fn every_system_configuration_slot(image: &MemoryPool) -> Vec<(u32, u64)> {
    DEVICES
        .iter()
        .flat_map(|device| {
            [
                (*device, 0),
                (
                    *device,
                    system_configuration_slot_one_offset(image, *device),
                ),
            ]
        })
        .collect()
}

/// I-8.1（环几何够大）与 I-7.5（根槽按判定宽度对齐）的坏镜像，建在**写完新池新建文件**那份干净镜像上：每份改系统配置的字段、重封整槽校验和，
/// 只红它自己那一条（`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 4、5 行）。
pub(super) fn known_bad_images_of_the_ring_geometry_and_the_root_slot_alignment(
) -> Vec<(&'static str, Mutation)> {
    vec![
        // I-8.1 三个字段等式各一份，都只改盘 0 槽 0、重封，另外两个等式照样成立：
        // ① 在飞上限加 1、最坏占用跟着改成新上限 × 4096——只有「在飞上限 = ⌊环长 ÷ 4096 ÷ F⌋」不成立；
        (
            JOURNAL_RING_GEOMETRY_FIELDS_AGREE,
            Box::new(|image: &mut MemoryPool| {
                mutate_system_configuration_slot(image, 0, 0, |bytes| {
                    let limit =
                        get_u32(bytes, SYSTEM_CONFIGURATION_IN_FLIGHT_RECORD_LIMIT_OFFSET) + 1;
                    set_u32(
                        bytes,
                        SYSTEM_CONFIGURATION_IN_FLIGHT_RECORD_LIMIT_OFFSET,
                        limit,
                    );
                    set_u64(
                        bytes,
                        SYSTEM_CONFIGURATION_WORST_CASE_OCCUPANCY_OFFSET,
                        u64::from(limit) * 4096,
                    );
                });
            }),
        ),
        // ② 最坏占用多 4096——只有「最坏占用 = 在飞上限 × 4096」不成立；
        (
            JOURNAL_RING_GEOMETRY_FIELDS_AGREE,
            Box::new(|image: &mut MemoryPool| {
                mutate_system_configuration_slot(image, 0, 0, |bytes| {
                    let worst = get_u64(bytes, SYSTEM_CONFIGURATION_WORST_CASE_OCCUPANCY_OFFSET);
                    set_u64(
                        bytes,
                        SYSTEM_CONFIGURATION_WORST_CASE_OCCUPANCY_OFFSET,
                        worst + 4096,
                    );
                });
            }),
        ),
        // ③ F 写 1、在飞上限与最坏占用按 F = 1 算——两个等式成立，只有「F = 格式常量」那一格不成立。
        (
            JOURNAL_RING_GEOMETRY_FIELDS_AGREE,
            Box::new(|image: &mut MemoryPool| {
                mutate_system_configuration_slot(image, 0, 0, |bytes| {
                    set_the_safety_factor_and_the_two_fields_computed_from_it(bytes, 1);
                });
            }),
        ),
        // ④ ⑤ 四个槽的 F 都写 2（读者下界那一格成立）、都写 4，在飞上限与最坏占用按槽里的 F 算——两个等式成立，只有「F = 格式常量」
        // 那一格不成立（`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 3 行，攻方 C2-1 那两份镜像）。
        (
            JOURNAL_RING_GEOMETRY_FIELDS_AGREE,
            Box::new(|image: &mut MemoryPool| {
                for (device, offset) in every_system_configuration_slot(image) {
                    mutate_system_configuration_slot(image, device, offset, |bytes| {
                        set_the_safety_factor_and_the_two_fields_computed_from_it(bytes, 2);
                    });
                }
            }),
        ),
        (
            JOURNAL_RING_GEOMETRY_FIELDS_AGREE,
            Box::new(|image: &mut MemoryPool| {
                for (device, offset) in every_system_configuration_slot(image) {
                    mutate_system_configuration_slot(image, device, offset, |bytes| {
                        set_the_safety_factor_and_the_two_fields_computed_from_it(bytes, 4);
                    });
                }
            }),
        ),
        // ⑥ 只改盘 1 没被择的那一槽：F 写 2，另两个字段按它算。择到的槽照旧是 F = 3 那一份，那一槽照样整池拒。
        (
            JOURNAL_RING_GEOMETRY_FIELDS_AGREE,
            Box::new(|image: &mut MemoryPool| {
                let slots = [0, system_configuration_slot_one_offset(image, 1)].map(|offset| {
                    (
                        offset,
                        get_u64(
                            &read(image, 1, offset, 4096),
                            SYSTEM_CONFIGURATION_SLOT_GENERATION_OFFSET,
                        ),
                    )
                });
                let unchosen_offset = if slots[1].1 > slots[0].1 {
                    slots[0].0
                } else {
                    slots[1].0
                };
                mutate_system_configuration_slot(image, 1, unchosen_offset, |bytes| {
                    set_the_safety_factor_and_the_two_fields_computed_from_it(bytes, 2);
                });
            }),
        ),
        // I-7.5：四个槽的 io_min 都声明成 8192，判定宽度成 8192，根环里每个奇数号槽（区域起点 + 4096 × 奇数）不对齐。
        (
            ROOT_SLOTS_ALIGNED_TO_THE_DECISION_WIDTH,
            Box::new(|image: &mut MemoryPool| {
                for (device, offset) in every_system_configuration_slot(image) {
                    mutate_system_configuration_slot(image, device, offset, |bytes| {
                        set_u32(
                            bytes,
                            SYSTEM_CONFIGURATION_MINIMUM_INPUT_OUTPUT_OFFSET,
                            MINIMUM_INPUT_OUTPUT_TWICE_THE_ROOT_SLOT_SPACING,
                        );
                    });
                }
            }),
        ),
        // I-7.5 异构池那一格（`research/prompts/m3-batch4-code-r1-sonnet-output.md` 第五节盲区第 8 条）：只有盘 1 两槽的 io_min 声明成 8192，
        // 盘 0 仍是 512——判定宽度取各盘里最大的那个（8192），奇数号槽照样不对齐。
        (
            ROOT_SLOTS_ALIGNED_TO_THE_DECISION_WIDTH,
            Box::new(|image: &mut MemoryPool| {
                for offset in [0, system_configuration_slot_one_offset(image, 1)] {
                    mutate_system_configuration_slot(image, 1, offset, |bytes| {
                        set_u32(
                            bytes,
                            SYSTEM_CONFIGURATION_MINIMUM_INPUT_OUTPUT_OFFSET,
                            MINIMUM_INPUT_OUTPUT_TWICE_THE_ROOT_SLOT_SPACING,
                        );
                    });
                }
            }),
        ),
    ]
}

/// 系统配置槽里 F 写成 `safety_factor`，在飞上限与最坏占用按它算（⌊环长 ÷ 4096 ÷ F⌋、上限 × 4096）：I-8.1 两个等式照样成立。
fn set_the_safety_factor_and_the_two_fields_computed_from_it(bytes: &mut [u8], safety_factor: u32) {
    let ring_records = JOURNAL_RING_BYTES_OF_THE_POOL / 4096;
    let in_flight_record_limit = ring_records / u64::from(safety_factor);
    set_u32(
        bytes,
        SYSTEM_CONFIGURATION_SAFETY_FACTOR_OFFSET,
        safety_factor,
    );
    set_u32(
        bytes,
        SYSTEM_CONFIGURATION_IN_FLIGHT_RECORD_LIMIT_OFFSET,
        u32::try_from(in_flight_record_limit).expect("环里 196 608 条，装得进 4 字节"),
    );
    set_u64(
        bytes,
        SYSTEM_CONFIGURATION_WORST_CASE_OCCUPANCY_OFFSET,
        in_flight_record_limit * 4096,
    );
}

/// I-8.1 三个字段等式有一格不等的镜像，实现整池拒绝挂载（`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 3、4 行）：
/// 上面那批里 I-8.1 的六份（在飞上限、最坏占用、F = 1 / 2 / 4、只坏盘 1 没被择的那一槽）每一份都只红 I-8.1，别的每一条
/// （连同 I-7.13）都报不适用——与 I-7.13 那一支同口径，checker 不替挂不上的池作保。
#[test]
fn journal_ring_geometry_fields_that_disagree_redden_only_the_ring_geometry_invariant_and_every_other_is_not_applicable(
) {
    let clean =
        build_pool("journal-ring-geometry-disagreement-is-a-whole-pool-refusal").memory_pool();
    let mut images_judged = 0;
    for (invariant, mutation) in known_bad_images_of_the_ring_geometry_and_the_root_slot_alignment()
    {
        if invariant != JOURNAL_RING_GEOMETRY_FIELDS_AGREE {
            continue;
        }
        images_judged += 1;
        let mut image = clean.clone();
        mutation(&mut image);
        let verdicts = check_pool_image(&image);
        assert_eq!(
            violated_invariants(&verdicts),
            [JOURNAL_RING_GEOMETRY_FIELDS_AGREE],
            "判红的该只有 I-8.1：{verdicts:?}"
        );
        let judged_other_than_not_applicable: Vec<&(&str, InvariantVerdict)> = verdicts
            .iter()
            .filter(|(name, found)| {
                *name != JOURNAL_RING_GEOMETRY_FIELDS_AGREE
                    && !matches!(found, InvariantVerdict::NotApplicable(_))
            })
            .collect();
        assert!(
            judged_other_than_not_applicable.is_empty(),
            "I-8.1 三格有一格不等时别的每一条都该报不适用：{judged_other_than_not_applicable:?}"
        );
    }
    assert_eq!(images_judged, 6, "I-8.1 三个字段等式那几份坏镜像六份");
}

/// 上面那批坏镜像各自**只**红在自己那一条上；干净镜像上 I-8.1 与 I-7.5 真被评估过且成立。
#[test]
fn each_ring_geometry_and_root_slot_alignment_bad_image_reddens_only_its_own_invariant() {
    let clean = build_pool("known-bad-ring-geometry-and-root-slot-alignment").memory_pool();
    let verdicts_on_the_clean_image = check_pool_image(&clean);
    for invariant in [
        JOURNAL_RING_GEOMETRY_FIELDS_AGREE,
        ROOT_SLOTS_ALIGNED_TO_THE_DECISION_WIDTH,
    ] {
        assert_eq!(
            verdict(&clean, invariant),
            InvariantVerdict::Holds,
            "干净镜像上 {invariant} 要真被评估过且成立：{verdicts_on_the_clean_image:?}"
        );
    }
    for (invariant, mutation) in known_bad_images_of_the_ring_geometry_and_the_root_slot_alignment()
    {
        let mut image = clean.clone();
        mutation(&mut image);
        let verdicts = check_pool_image(&image);
        assert_eq!(
            violated_invariants(&verdicts),
            [invariant],
            "判红的该只有 {invariant}：{verdicts:?}"
        );
    }
}

/// 翻一个系统配置槽里的一个字节、不重封：magic 照旧，整槽校验和对不上，这一槽不自证。
fn break_the_system_configuration_slot_without_resealing(
    image: &mut MemoryPool,
    device: u32,
    offset: u64,
) {
    const BYTE_INSIDE_THE_CHECKSUMMED_SLOT: usize = 300;
    let mut bytes = read(image, device, offset, 4096);
    bytes[BYTE_INSIDE_THE_CHECKSUMMED_SLOT] ^= 0xff;
    write(image, device, offset, &bytes);
}

/// I-8.5（tail 槽有效集非空，V-池）的坏镜像：全池四个系统配置槽各翻一个字节、不重封——一个自证过的槽都不剩。
pub(super) fn known_bad_images_of_the_tail_slot_witnesses() -> Vec<(&'static str, Mutation)> {
    vec![(
        SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION,
        Box::new(|image: &mut MemoryPool| {
            for (device, offset) in every_system_configuration_slot(image) {
                break_the_system_configuration_slot_without_resealing(image, device, offset);
            }
        }),
    )]
}

/// 全池皆废：I-8.5 判红，别的每一条都报不适用（这不是一个挂得上的镜像）；干净镜像上 I-8.5 真被评估过且成立。
#[test]
fn no_self_verified_system_configuration_slot_on_any_device_reddens_the_tail_slot_invariant_and_every_other_is_not_applicable(
) {
    let clean = build_pool("known-bad-tail-slot-witnesses").memory_pool();
    assert_eq!(
        verdict(
            &clean,
            SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION
        ),
        InvariantVerdict::Holds,
        "干净镜像上 I-8.5 要真被评估过且成立"
    );
    for (invariant, mutation) in known_bad_images_of_the_tail_slot_witnesses() {
        let mut image = clean.clone();
        mutation(&mut image);
        let verdicts = check_pool_image(&image);
        assert_eq!(
            violated_invariants(&verdicts),
            [invariant],
            "判红的该只有 {invariant}：{verdicts:?}"
        );
        let judged_other_than_not_applicable: Vec<&(&str, InvariantVerdict)> = verdicts
            .iter()
            .filter(|(name, found)| {
                *name != invariant && !matches!(found, InvariantVerdict::NotApplicable(_))
            })
            .collect();
        assert!(
            judged_other_than_not_applicable.is_empty(),
            "全池皆废时别的每一条都该报不适用：{judged_other_than_not_applicable:?}"
        );
    }
}

/// 单盘全废不进违例通道：盘 1 两槽都翻坏，I-8.5 照样成立（盘 0 有自证过的槽），旁路报告点名盘 1 两槽都废、少于两槽自证过
/// （这类盘的可写挂载带读故障时可能判不出而拒；没有读故障时挂得上，取号写两轮补回那一槽）。
/// 只坏盘 0 的槽 0 时盘 0 少一槽自证过，旁路报告只在少于两槽自证过那一样里报它；干净镜像上旁路报告两样都空。
#[test]
fn a_device_short_of_self_verified_system_configuration_slots_is_reported_aside_while_the_tail_slot_invariant_holds(
) {
    use singlefs_checker::image::{
        system_configuration_redundancy_report, SystemConfigurationRedundancyReport,
    };
    let clean = build_pool("tail-slot-witnesses-reported-aside").memory_pool();
    assert_eq!(
        system_configuration_redundancy_report(&clean),
        SystemConfigurationRedundancyReport {
            devices_without_a_self_verified_slot: Vec::new(),
            devices_with_fewer_than_two_self_verified_slots: Vec::new(),
        },
        "干净镜像上每块盘两槽都自证过"
    );
    let mut device_one_lost = clean.clone();
    let slot_one_offset_of_device_one = system_configuration_slot_one_offset(&clean, 1);
    for offset in [0, slot_one_offset_of_device_one] {
        break_the_system_configuration_slot_without_resealing(&mut device_one_lost, 1, offset);
    }
    assert_eq!(
        verdict(
            &device_one_lost,
            SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION
        ),
        InvariantVerdict::Holds,
        "盘 0 两槽都还自证过：单盘全废不判违例"
    );
    assert_eq!(
        system_configuration_redundancy_report(&device_one_lost),
        SystemConfigurationRedundancyReport {
            devices_without_a_self_verified_slot: vec![1],
            devices_with_fewer_than_two_self_verified_slots: vec![1],
        }
    );
    let mut one_slot_of_device_zero_lost = clean.clone();
    break_the_system_configuration_slot_without_resealing(&mut one_slot_of_device_zero_lost, 0, 0);
    assert_eq!(
        system_configuration_redundancy_report(&one_slot_of_device_zero_lost),
        SystemConfigurationRedundancyReport {
            devices_without_a_self_verified_slot: Vec::new(),
            devices_with_fewer_than_two_self_verified_slots: vec![0],
        }
    );
}

/// 干净镜像（最新根之上没有记录）上 I-4.1 报的不适用理由。
pub(super) const NOTHING_APPLIED_ABOVE_THE_CHOSEN_ROOT: &str =
    "所选根之上一次发布都不施加：重放之后那一版就是所选根，它可不可遍历归 I-7.2";

/// 记录头里新根段（99 起）的树表指针（新根段 [0..86)）两条位置条目的整单元校验和（指针 50 + 10、64 + 10）。
const JOURNAL_RECORD_NEW_TREE_TABLE_POINTER_CHECKSUMS: [usize; 2] = [99 + 50 + 10, 99 + 64 + 10];

/// 发布 B 那条记录的计数器（新池新建文件写到 3，B 是第 4 条）。
pub(super) const COUNTER_OF_THE_OVERWRITE_RECORD: u64 = 4;

/// I-4.1（重放后树可遍历）的坏镜像，建在「发布 B 的记录两盘都落了、根槽没落」那份崩溃镜像上：B 那条记录新根段里树表指针的两条位置条目的
/// 校验和各加 1、重封记录头（新根段在头里，载荷校验和不动）。点名项一个字节不动 ⇒ 点名单元照样两份都验得过、记录照样施加，
/// 施加出来那一版的树表两份都读不对。
pub(super) fn known_bad_images_of_the_replayed_version() -> Vec<(&'static str, Mutation)> {
    vec![(
        TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE,
        Box::new(|image: &mut MemoryPool| {
            for device in DEVICES {
                break_the_new_tree_table_pointer_of_the_overwrite_record_on(image, device);
            }
        }),
    )]
}

/// 崩在发布 B 的记录落了、根槽没落：下一次挂载施加 B 那条记录，重放之后那一版走得完 ⇒ I-4.1 真被评估过且成立、一条都不红；
/// B 的新根段里树表指针读不对 ⇒ **只**红 I-4.1。
#[test]
fn a_replayed_version_whose_tree_table_does_not_read_back_reddens_only_the_replayed_version_invariant(
) {
    let clean =
        crash_image_with_the_overwrite_record_but_not_its_root("known-bad-replayed-version");
    let verdicts_on_the_clean_image = check_pool_image(&clean);
    assert_eq!(
        violated_invariants(&verdicts_on_the_clean_image),
        Vec::<&str>::new(),
        "干净的崩溃镜像上一条都不红：{verdicts_on_the_clean_image:?}"
    );
    assert_eq!(
        verdict(&clean, TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE),
        InvariantVerdict::Holds,
        "B 那条记录被重放、施加出来的那一版走得完：I-4.1 真被评估过且成立"
    );
    for (invariant, mutation) in known_bad_images_of_the_replayed_version() {
        let mut image = clean.clone();
        mutation(&mut image);
        let verdicts = check_pool_image(&image);
        assert_eq!(
            violated_invariants(&verdicts),
            [invariant],
            "判红的该只有 {invariant}：{verdicts:?}"
        );
    }
}

/// 起点含介质损坏不在 I-4.1 判的合法镜像里：
/// B 那条记录第一个点名单元在盘 1 上那一份翻一个字节，前缀停在 B 那条（两份都验过才施加）⇒ I-4.1 报不适用，不报成立也不报违例。
#[test]
fn a_replay_stopped_by_a_named_unit_that_does_not_verify_leaves_the_replayed_version_invariant_not_applicable(
) {
    let mut image =
        crash_image_with_the_overwrite_record_but_not_its_root("replay-stopped-by-a-named-unit");
    let offset = singlefs_core::journal::record_offset(
        COUNTER_OF_THE_OVERWRITE_RECORD,
        singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
    )
    .0;
    let record = read(&image, 0, offset, 4096);
    assert!(
        get_u32(&record, JOURNAL_RECORD_NAMED_COUNT_OFFSET) > 0,
        "B 那条记录有点名项"
    );
    let second_location = JOURNAL_RECORD_HEADER_BYTES + 14;
    let named_device = get_u32(&record, second_location);
    let mut named_slot_bytes = [0u8; 8];
    named_slot_bytes[..6].copy_from_slice(&record[second_location + 4..second_location + 10]);
    let named_slot = u64::from_le_bytes(named_slot_bytes);
    let mut unit = read(&image, named_device, named_slot * SLOT, 16384);
    unit[1000] ^= 0xff;
    write(&mut image, named_device, named_slot * SLOT, &unit);
    match verdict(&image, TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE) {
        InvariantVerdict::NotApplicable(reason) => assert!(
            reason.contains("介质损坏"),
            "报不适用的理由要说是起点含介质损坏：{reason}"
        ),
        other @ (InvariantVerdict::Holds
        | InvariantVerdict::Violated(_)
        | InvariantVerdict::PartiallyJudged(_)) => {
            panic!("点名单元一份验不过时 I-4.1 该报不适用，实际 {other:?}")
        }
    }
}

/// 「发布 B 的记录两盘都落了、根槽没落」那份崩溃镜像上 B 那条记录照恢复的读法施加、I-4.1 真被评估过且成立；`change` 改过之后，
/// D23（journal 的角色与格式） 已定项 14 的前缀规则里有一条该把 B 那条挡在前缀之外 ⇒ 一次发布都不施加，I-4.1 报不适用
/// （`research/prompts/m3-batch4-code-r1-sonnet-output.md` 第五节盲区第 10 条：重放规则各配一份只靠它挡住的镜像）。
fn assert_the_replay_stops_before_the_overwrite_record(
    tag: &str,
    change: impl Fn(&mut MemoryPool),
) {
    let mut image = crash_image_with_the_overwrite_record_but_not_its_root(tag);
    assert_eq!(
        verdict(&image, TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE),
        InvariantVerdict::Holds,
        "改之前 B 那条记录被重放、施加出来的那一版走得完"
    );
    change(&mut image);
    assert_eq!(
        verdict(&image, TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE),
        InvariantVerdict::NotApplicable(NOTHING_APPLIED_ABOVE_THE_CHOSEN_ROOT),
        "B 那条记录该被挡在重放前缀之外、一次发布都不施加"
    );
}

/// 带末条标志的 B 那条之后，同一次发布（实例 1、txg 4）还有一条读得出的记录（计数器 5，序号 2，反向链接在 B 上）：
/// 读者规则「带末条标志的那条之后还有记录」断在 B 那条，B 不施加。
#[test]
fn a_readable_record_of_the_same_publish_after_the_last_record_flag_stops_the_replay_before_it() {
    assert_the_replay_stops_before_the_overwrite_record(
        "replay-stops-at-a-record-after-the-last-record-flag",
        |image| {
            let offset_of_the_overwrite_record =
                record_offset(COUNTER_OF_THE_OVERWRITE_RECORD, JOURNAL_RING_DEFAULT_BYTES).0;
            let offset_after_it = record_offset(
                COUNTER_OF_THE_OVERWRITE_RECORD + 1,
                JOURNAL_RING_DEFAULT_BYTES,
            )
            .0;
            for device in DEVICES {
                let overwrite_bytes = read(image, device, offset_of_the_overwrite_record, 4096);
                let overwrite = JournalRecord::parse(
                    &overwrite_bytes,
                    get_u64(
                        &overwrite_bytes,
                        JOURNAL_RECORD_FILESYSTEM_IDENTIFIER_OFFSET,
                    ),
                )
                .expect("B 那条记录自证过");
                let mut follower = overwrite.clone();
                follower.counter = COUNTER_OF_THE_OVERWRITE_RECORD + 1;
                follower.ordinal_within_publish = JournalRecordOrdinalWithinPublish(2);
                follower.place_in_publish =
                    JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow;
                follower.is_commit = false;
                follower.named = Vec::new();
                follower.back_chain = back_chain_of(&overwrite_bytes);
                write(image, device, offset_after_it, &follower.to_bytes());
            }
        },
    );
}

/// B 那条（带末条标志）摘掉提交标记：已定项 14 第三条「末条要带提交标记」不成，B 不施加。
#[test]
fn a_last_record_without_its_commit_marker_stops_the_replay_before_it() {
    assert_the_replay_stops_before_the_overwrite_record(
        "replay-stops-at-a-last-record-without-commit-marker",
        |image| {
            rewrite_journal_record_and_rechain_the_records_after_it(
                image,
                COUNTER_OF_THE_OVERWRITE_RECORD,
                |record| record.is_commit = false,
            );
        },
    );
}

/// B 那条（一次发布的第一条）本次发布内序号 1 → 2：已定项 4 读者规则「一次发布的首条序号不是 1」断在这一条，B 不施加。
#[test]
fn a_first_record_of_a_publish_whose_ordinal_is_not_one_stops_the_replay_before_it() {
    assert_the_replay_stops_before_the_overwrite_record(
        "replay-stops-at-a-publish-starting-past-ordinal-one",
        |image| {
            rewrite_journal_record_and_rechain_the_records_after_it(
                image,
                COUNTER_OF_THE_OVERWRITE_RECORD,
                |record| record.ordinal_within_publish = JournalRecordOrdinalWithinPublish(2),
            );
        },
    );
}

/// B 那条的反向链翻一位：同实例逻辑前一条（计数器 3，A 那条）在盘上、链值对不上（I-8.6「不等的记录不进重放前缀」），B 不施加。
#[test]
fn a_record_whose_back_chain_does_not_match_its_predecessor_stops_the_replay_before_it() {
    assert_the_replay_stops_before_the_overwrite_record(
        "replay-stops-at-a-broken-back-chain",
        |image| {
            rewrite_journal_record_and_rechain_the_records_after_it(
                image,
                COUNTER_OF_THE_OVERWRITE_RECORD,
                |record| record.back_chain ^= 1,
            );
        },
    );
}

/// B 那条挪到计数器 5（环里那一格），计数器 4 那一格两盘都清零：链首锚在 A 那条（计数器 3）之后要接 4，断号即止，B 不施加。
#[test]
fn a_record_one_counter_past_the_anchor_successor_stops_the_replay_before_it() {
    assert_the_replay_stops_before_the_overwrite_record("replay-stops-at-a-counter-gap", |image| {
        let offset_of_the_overwrite_record =
            record_offset(COUNTER_OF_THE_OVERWRITE_RECORD, JOURNAL_RING_DEFAULT_BYTES).0;
        let offset_after_it = record_offset(
            COUNTER_OF_THE_OVERWRITE_RECORD + 1,
            JOURNAL_RING_DEFAULT_BYTES,
        )
        .0;
        for device in DEVICES {
            let overwrite_bytes = read(image, device, offset_of_the_overwrite_record, 4096);
            let mut moved = JournalRecord::parse(
                &overwrite_bytes,
                get_u64(
                    &overwrite_bytes,
                    JOURNAL_RECORD_FILESYSTEM_IDENTIFIER_OFFSET,
                ),
            )
            .expect("B 那条记录自证过");
            moved.counter = COUNTER_OF_THE_OVERWRITE_RECORD + 1;
            write(image, device, offset_after_it, &moved.to_bytes());
            write(image, device, offset_of_the_overwrite_record, &[0u8; 4096]);
        }
    });
}

/// B 那条记录新根段里树表指针两条位置条目的校验和各加 1、重封记录头（只改 `device` 那一份）。
fn break_the_new_tree_table_pointer_of_the_overwrite_record_on(
    image: &mut MemoryPool,
    device: u32,
) {
    let offset = record_offset(COUNTER_OF_THE_OVERWRITE_RECORD, JOURNAL_RING_DEFAULT_BYTES).0;
    let mut record = read(image, device, offset, 4096);
    assert_eq!(
        get_u64(&record, JOURNAL_RECORD_CHECKPOINT_TXG_OFFSET),
        4,
        "盘 {device} 计数器 4 那条记录是发布 B（txg 4）"
    );
    for checksum_offset in JOURNAL_RECORD_NEW_TREE_TABLE_POINTER_CHECKSUMS {
        let checksum = get_u32(&record, checksum_offset);
        set_u32(&mut record, checksum_offset, checksum.wrapping_add(1));
    }
    let digest = wide_checksum_with_field_zeroed(&record, 4096, 46);
    record[46..78].copy_from_slice(&digest);
    write(image, device, offset, &record);
}

/// 攻方 C2-2 的手造镜像（`research/prompts/m3-batch4-code-r1-opus-output.md` C2-2）：「发布 B 的记录两盘都落了、根槽没落」那份
/// 崩溃镜像上先取号到实例 2（两盘系统配置跟着轮换，I-7.7 不先红），再把盘 0 计数器 4 那一格换成实例 2、txg 5 的一条记录
/// （照 B 那条改实例、txg、反向链 0、事务号 1、带提交标记与末条标志、点名项清空），盘 1 那一格仍是 B。
/// 第一版两盘池上推为不可达（同一计数器要被两个实例各写一次），checker 与实现同样按 (实例, 计数器) 读环。
fn image_with_another_instance_on_device_zero_at_the_counter_of_the_overwrite_record(
    tag: &str,
) -> MemoryPool {
    let crash = crash_image_with_the_overwrite_record_but_not_its_root(tag);
    let stream = SharedStream::retaining_contents();
    let mut devices = crash_state_devices(&crash, &[], &[], &stream);
    let parameters_of_the_pool = parameters();
    {
        let mut writer = PoolWriter::new(&parameters_of_the_pool, devices.as_mut_slice());
        let instance = acquire_instance(&mut writer).expect("取号");
        assert_eq!(instance, InstanceGeneration(2), "崩溃镜像上下一个实例是 2");
    }
    let mut forged = memory_pool_of_sparse_devices(&devices);
    let offset = record_offset(COUNTER_OF_THE_OVERWRITE_RECORD, JOURNAL_RING_DEFAULT_BYTES).0;
    let overwrite_bytes = read(&forged, 1, offset, 4096);
    let mut another_instance = JournalRecord::parse(
        &overwrite_bytes,
        get_u64(
            &overwrite_bytes,
            JOURNAL_RECORD_FILESYSTEM_IDENTIFIER_OFFSET,
        ),
    )
    .expect("盘 1 计数器 4 上是 B");
    another_instance.instance = InstanceGeneration(2);
    another_instance.checkpoint_txg = CheckpointTxg(5);
    another_instance.back_chain = 0;
    another_instance.transaction = 1;
    another_instance.is_commit = true;
    another_instance.place_in_publish = JournalRecordPlaceInPublish::LastRecordOfThePublish;
    another_instance.named = Vec::new();
    write(&mut forged, 0, offset, &another_instance.to_bytes());
    forged
}

/// 两块盘同一计数器坐着不同实例的记录：实现扫环按 (实例, 计数器) 两条都收、从实例 1 的 A 那条接到 B、施加 B（落在 (1, 4)），
/// checker 的重放同样施加 B（I-4.1 真被评估过且成立），不拿盘 0 那条实例 2 的记录挤掉 B。
#[test]
fn a_record_of_another_instance_at_the_same_counter_on_the_first_device_does_not_hide_the_overwrite_record_from_the_replay(
) {
    let forged = image_with_another_instance_on_device_zero_at_the_counter_of_the_overwrite_record(
        "replay-reads-the-ring-by-instance-and-counter",
    );
    assert_eq!(
        recover(&forged, JournalPolicy::Consult).effective_root,
        Some((InstanceGeneration(1), CheckpointTxg(4))),
        "实现的恢复施加 B、落在 (1, 4)"
    );
    assert_eq!(
        verdict(&forged, TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE),
        InvariantVerdict::Holds,
        "checker 的重放同样施加 B、施加出来那一版走得完"
    );
}

/// 同一张镜像再把盘 1 那份 B 的新根段里树表指针改坏：实现施加 B 之后读不出树表、恢复失败；checker 的重放同样施加 B（末条计数器 4），
/// 施加出来那一版走不完 ⇒ 红 I-4.1。
#[test]
fn a_record_of_another_instance_at_the_same_counter_does_not_hide_a_broken_overwrite_record_from_the_replayed_version_invariant(
) {
    let mut forged =
        image_with_another_instance_on_device_zero_at_the_counter_of_the_overwrite_record(
            "replay-by-instance-and-counter-reaches-a-broken-record",
        );
    break_the_new_tree_table_pointer_of_the_overwrite_record_on(&mut forged, 1);
    assert!(
        matches!(
            recover(&forged, JournalPolicy::Consult).outcome,
            RecoveryOutcome::Failed { .. }
        ),
        "实现施加 B 之后读不出它的树表、恢复失败"
    );
    match verdict(&forged, TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE) {
        InvariantVerdict::Violated(detail) => assert!(
            detail.contains("（末条计数器 4）"),
            "checker 施加的是 B 那条（计数器 4）：{detail}"
        ),
        other @ (InvariantVerdict::Holds
        | InvariantVerdict::NotApplicable(_)
        | InvariantVerdict::PartiallyJudged(_)) => {
            panic!("checker 施加 B 之后那一版走不完，I-4.1 该判违例，实际 {other:?}")
        }
    }
}

/// 1 MiB 的 journal 环：64 槽，环末尾的下一个槽 1088 = 17 × 64 落在聚簇段边界上，mkfs 收它；256 条记录，在飞上限 ⌊256 ÷ 3⌋ = 85 条
/// （`core_review_unit_area_start_and_publish_limits.rs` 用的同一个环）。
const JOURNAL_RING_OF_ONE_MEBIBYTE: u64 = 1 << 20;

const RECORD_LIMIT_ON_THE_ONE_MEBIBYTE_RING: u64 =
    JOURNAL_RING_OF_ONE_MEBIBYTE / JOURNAL_RECORD_BYTES / JOURNAL_SAFETY_FACTOR;

/// 两块内存盘、1 MiB 环的池：mkfs → 取号 → 暖机 → 新池新建文件（txg 3）→ 从文件偏移 0 顺序写 85 个数据单元（txg 4）：
/// 那次发布切成恰好在飞上限那么多条记录，末条带末条标志。交回镜像与那次发布末条的计数器。
fn image_with_a_publish_of_as_many_records_as_the_in_flight_limit_on_a_one_mebibyte_ring(
) -> (MemoryPool, u64) {
    let default_parameters = parameters();
    let parameters_on_the_one_mebibyte_ring = MakeFilesystemParameters {
        filesystem_identifier: default_parameters.filesystem_identifier,
        region_devices: default_parameters.region_devices,
        geometry: SystemImmutableSizes {
            physical_block_size: default_parameters.geometry.physical_block_size,
            minimum_input_output_bytes: default_parameters.geometry.minimum_input_output_bytes,
            fixed_structure_slot_spacing: default_parameters.geometry.fixed_structure_slot_spacing,
            journal_ring_bytes: JOURNAL_RING_OF_ONE_MEBIBYTE,
            root_ring_slots_per_region: default_parameters.geometry.root_ring_slots_per_region,
        },
    };
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> =
        [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .map(|identity| {
                (
                    identity,
                    SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                )
            })
            .collect();
    let genesis = make_filesystem(&parameters_on_the_one_mebibyte_ring, &mut devices)
        .expect("1 MiB 的环上 mkfs");
    let mut allocator =
        allocator_after_make_filesystem(&parameters_on_the_one_mebibyte_ring, &devices, &genesis);
    let data_unit_payload_bytes = u64::try_from(singlefs_core::unit::data_unit_payload_capacity())
        .expect("一个数据单元的净荷装得进 u64");
    let sequential_content: Vec<u8> =
        (0..(RECORD_LIMIT_ON_THE_ONE_MEBIBYTE_RING - 1) * data_unit_payload_bytes + 1)
            .map(|index| u8::try_from(index % 251).expect("小于 256"))
            .collect();
    let sequential_write = {
        let mut writer =
            PoolWriter::new(&parameters_on_the_one_mebibyte_ring, devices.as_mut_slice());
        let instance = acquire_instance(&mut writer).expect("取号");
        let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        let content = common::file_content();
        let first_file = publish_first_file(
            &mut writer,
            &mut allocator,
            warmed.roots.last().expect("暖机两代根"),
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warmed.last_record_bytes,
        )
        .expect("新池新建文件");
        publish_sequential_write(
            &mut writer,
            &mut allocator,
            &first_file,
            FirstFile {
                content: &sequential_content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
            },
            instance,
        )
        .expect("85 条记录不多于在飞上限 85 条")
    };
    assert_eq!(
        u64::try_from(sequential_write.earlier_records_of_this_publish.len()).expect("条数") + 1,
        RECORD_LIMIT_ON_THE_ONE_MEBIBYTE_RING,
        "顺序写那次发布切成恰好 85 条记录"
    );
    let image = MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: IMAGE_BYTES,
    };
    let last_counter = (1..=JOURNAL_RING_OF_ONE_MEBIBYTE / JOURNAL_RECORD_BYTES)
        .take_while(|counter| {
            let bytes = read(
                &image,
                0,
                record_offset(*counter, JOURNAL_RING_OF_ONE_MEBIBYTE).0,
                4096,
            );
            JournalRecord::parse(
                &bytes,
                get_u64(&bytes, JOURNAL_RECORD_FILESYSTEM_IDENTIFIER_OFFSET),
            )
            .is_some()
        })
        .last()
        .expect("环里有记录");
    (image, last_counter)
}

/// 上面那份镜像上把顺序写那次发布接长一条：原来的末条（计数器 `last_counter`）摘掉末条标志，后面接一条同实例、同 txg 的记录
/// （计数器加 1、序号 86、带末条标志与提交标记、事务号接着往上、点名项清空、反向链接在改过的原末条上），两盘各一份：
/// 那次发布 86 条记录，多于在飞上限 85 条，别的读者规则都还成立。
fn lengthen_the_publish_on_the_one_mebibyte_ring_past_the_in_flight_limit(
    image: &mut MemoryPool,
    last_counter: u64,
) {
    for device in DEVICES {
        let last_offset = record_offset(last_counter, JOURNAL_RING_OF_ONE_MEBIBYTE).0;
        let last_bytes = read(image, device, last_offset, 4096);
        let mut last = JournalRecord::parse(
            &last_bytes,
            get_u64(&last_bytes, JOURNAL_RECORD_FILESYSTEM_IDENTIFIER_OFFSET),
        )
        .expect("顺序写那次发布的末条自证过");
        assert_eq!(
            (
                last.ordinal_within_publish,
                last.place_in_publish,
                last.is_commit
            ),
            (
                JournalRecordOrdinalWithinPublish(85),
                JournalRecordPlaceInPublish::LastRecordOfThePublish,
                true
            ),
            "盘 {device} 计数器 {last_counter} 是那次发布的第 85 条、末条"
        );
        last.place_in_publish = JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow;
        let rewritten_last_bytes = last.to_bytes();
        write(image, device, last_offset, &rewritten_last_bytes);
        let mut appended = last.clone();
        appended.counter = last_counter + 1;
        appended.ordinal_within_publish = JournalRecordOrdinalWithinPublish(86);
        appended.place_in_publish = JournalRecordPlaceInPublish::LastRecordOfThePublish;
        appended.transaction = last.transaction + 1;
        appended.is_commit = true;
        appended.named = Vec::new();
        appended.back_chain = back_chain_of(&rewritten_last_bytes);
        write(
            image,
            device,
            record_offset(appended.counter, JOURNAL_RING_OF_ONE_MEBIBYTE).0,
            &appended.to_bytes(),
        );
    }
}

/// I-8.1「同一次发布的记录数不多于在飞上限」那一格的边界（`research/prompts/m3-batch4-code-r1-sonnet-output.md` 第五节盲区第 7 条）：
/// 一次发布恰好 85 条记录、在飞上限 85 条 ⇒ I-8.1 真被评估过且成立，一条都不红。
#[test]
fn a_publish_of_exactly_the_in_flight_limit_of_records_holds_the_ring_geometry_invariant() {
    let (image, _) =
        image_with_a_publish_of_as_many_records_as_the_in_flight_limit_on_a_one_mebibyte_ring();
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        Vec::<&str>::new(),
        "一次发布 85 条记录、在飞上限 85 条：一条都不红：{verdicts:?}"
    );
    assert_eq!(
        verdict(&image, JOURNAL_RING_GEOMETRY_FIELDS_AGREE),
        InvariantVerdict::Holds,
        "I-8.1 真被评估过且成立"
    );
}

/// 同一次发布接长到 86 条（多于在飞上限 85 条），系统配置三个字段照样成立 ⇒ **只**红 I-8.1 记录条数那一格。
#[test]
fn a_publish_of_one_record_more_than_the_in_flight_limit_reddens_only_the_ring_geometry_invariant()
{
    let (mut image, last_counter) =
        image_with_a_publish_of_as_many_records_as_the_in_flight_limit_on_a_one_mebibyte_ring();
    lengthen_the_publish_on_the_one_mebibyte_ring_past_the_in_flight_limit(
        &mut image,
        last_counter,
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [JOURNAL_RING_GEOMETRY_FIELDS_AGREE],
        "判红的该只有 I-8.1：{verdicts:?}"
    );
}

/// 86 条那一份再把那次发布的根（txg 4）改坏：所选根退到 txg 3，它之上那次发布 86 条，重放数到在飞上限 85 条就停（已定项 18），
/// 那次发布的末条没到、整体不施加 ⇒ I-4.1 报不适用（`research/prompts/m3-batch4-code-r1-sonnet-output.md` 第五节盲区第 10 条 L1）。
#[test]
fn a_replay_stops_at_the_in_flight_limit_and_a_publish_longer_than_it_is_not_applied() {
    let (mut image, last_counter) =
        image_with_a_publish_of_as_many_records_as_the_in_flight_limit_on_a_one_mebibyte_ring();
    lengthen_the_publish_on_the_one_mebibyte_ring_past_the_in_flight_limit(
        &mut image,
        last_counter,
    );
    make_the_root_unreadable(&mut image, CheckpointTxg(4));
    assert_eq!(
        verdict(&image, TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE),
        InvariantVerdict::NotApplicable(NOTHING_APPLIED_ABOVE_THE_CHOSEN_ROOT),
        "86 条的那次发布超过在飞上限，重放一次发布都不施加"
    );
}

/// 系统配置槽里 incompat 位图第一个字节的偏移（magic 4、格式版本 2 之后，D22（单元原子性怎么合成） 已定项 9 的字段表）。
const SYSTEM_CONFIGURATION_FIRST_INCOMPAT_BYTE_OFFSET: usize = 6;

/// incompat 位 3（第一个字节的 0x08）没登记：带上它的槽自证过，而这一版读者不认得这一位、不可择（`UnknownIncompatBit`）。
/// R、S 越界归读者不收之值、整池拒（C663），不再能造「自证过而不可择」的槽。
const INCOMPAT_UNREGISTERED_BIT_THREE: u8 = 0x08;

/// I-8.5（tail 槽有效集非空）数的是自证过的槽，「自证过而不可择」的也算（`research/prompts/m3-batch4-code-r1-sonnet-output.md`
/// 第五节盲区第 9 条）：全池四个槽的 incompat 都带一个不认得的位、整槽校验和重封 ⇒ 一槽都择不出、每一槽都自证过 ⇒ 全池皆废那一格不成立、I-8.5 不判红；
/// 而一块盘都交不出可择的系统配置，这不是一个挂得上的镜像：I-8.5 与别的每一条一样报不适用、带理由，不判成立
/// （`InvariantVerdict`「不适用必须带理由，不许静默算成立」；I-8.5 不在 `chosen.is_empty()` 早退之前判成成立）。一条都不红。
#[test]
fn self_verified_system_configuration_slots_no_reader_can_choose_leave_every_invariant_not_applicable(
) {
    let clean = build_pool("tail-slot-witnesses-self-verified-but-not-choosable").memory_pool();
    let mut image = clean.clone();
    for (device, offset) in every_system_configuration_slot(&clean) {
        mutate_system_configuration_slot(&mut image, device, offset, |bytes| {
            assert_eq!(
                bytes[SYSTEM_CONFIGURATION_FIRST_INCOMPAT_BYTE_OFFSET]
                    & INCOMPAT_UNREGISTERED_BIT_THREE,
                0,
                "干净镜像不带位 3"
            );
            bytes[SYSTEM_CONFIGURATION_FIRST_INCOMPAT_BYTE_OFFSET] |=
                INCOMPAT_UNREGISTERED_BIT_THREE;
        });
    }
    let verdicts = check_pool_image(&image);
    let judged: Vec<&(&str, InvariantVerdict)> = verdicts
        .iter()
        .filter(|(_, verdict)| !matches!(verdict, InvariantVerdict::NotApplicable(_)))
        .collect();
    assert_eq!(
        judged,
        Vec::<&(&str, InvariantVerdict)>::new(),
        "一槽都择不出：每一条都报不适用，I-8.5 也不判成立"
    );
    assert!(
        matches!(
            verdict(
                &image,
                SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION
            ),
            InvariantVerdict::NotApplicable(_)
        ),
        "自证过而不可择的槽在：I-8.5 不判红，报不适用"
    );
}
