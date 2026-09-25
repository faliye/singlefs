//! 里程碑「第二个事务」增补 3 第 5 件：坏盘输入。合法镜像由第 1 件的生成器造（`singlefs_harness::history`），
//! 坏法、三个读者与判定在 `singlefs_harness::bad_disk_input`；这里是快档（普通 `cargo test`）、逐个坏法的落点、
//! 判别力自证与大档（`#[ignore]`，规模从环境变量取）。
//!
//! 判据（里程碑「设想实现」第 5 件逐字）：恢复、挂载与 checker 三者都不许 panic；恢复读回的内容要么是模型允许的某一版，
//! 要么报错，**不许读回一版从没提交过的内容**。
//!
//! 「零 panic」那一半 2026-09-22 第三批改法落地之后**在这一档上成立**：`bad_disk_input::KNOWN_PANIC_SITES`
//! 空了（`records/2026-09-22-panic面普查-走得到的那些.md` 那三族里这批坏法打得到的十处，改法都是
//! 「盘上读来的值在边界上验一次、返回错误成员」），这一档三个读者一次都没 panic。清单空着仍然是判据：
//! 任何一处 panic 都落进「清单外的新发现」那一格、当场判红。清单只许缩，往里加一行要主 agent 点头。
//!
//! 种子基是这个测试周期写死的那一个（`SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE`，随机历史与崩溃注入用的是同一个）。
//!
//! cargo-fuzz（libFuzzer，要 nightly）那一路这一轮没做（里程碑自己标着「两路怎么分是预想」）。

use std::io::Write as _;

use singlefs_core::address::DeviceIdentity;
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::mount::{mount_writable, MountError};
use singlefs_core::transaction::TransactionUnit;
use singlefs_harness::bad_disk_input::{
    damage_image, feed_a_damaged_image, known_panic_site_among, known_panic_site_of,
    newest_root_has_the_trees_the_targeted_damages_need, run_bad_disk_campaign, BadDiskCampaign,
    BadDiskFinding, BadDiskInputWorkerThreads, BadDiskReport, DamageKind, KnownPanicSite,
    ReadBackVerdict, ReaderOutcome, BAD_DISK_INPUT_WORKER_THREADS_ENVIRONMENT_VARIABLE,
    EVERY_BASE_IMAGE_TIER, EVERY_DAMAGE_KIND, KNOWN_PANIC_SITES,
};
use singlefs_harness::crash::{MemoryPool, SparseBlockDevice};
use singlefs_harness::crash_injection::SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;
use singlefs_harness::history::{
    execute_history_with, generate_history_with_weights, CapturedPanic, GeneratedHistory,
    GenerationWeights, HistoryDeviceWidth, HistoryExecution, HistoryOperation, HistorySeed,
    HistoryStartingPoint, PerStepChecker, SeededRandomSource,
};
use singlefs_harness::SharedStream;

/// 快档的规模：段数与每段步数。规模与种子基都写死（种子基是这个测试周期的常量）。
const FAST_TIER_SEEDS: u64 = 12;
const FAST_TIER_OPERATIONS_PER_HISTORY: usize = 16;

/// 多线程对拍那一条的规模：段数取 6（切法在 1 与 4 个线程下不同，见那条用例的文档注释），每段 4 步（只要两边逐字相同，不要覆盖）。
const DETERMINISM_SEEDS: u64 = 6;
const DETERMINISM_OPERATIONS_PER_HISTORY: usize = 4;

/// 写死的那几条用例用种子基加这个偏移的那一段历史：它从第一个文件起，每一步之后最新那条根下面
/// extent / 分配记录 / 记账三棵树都在（2026-09-22 在种子基往后 24 个种子上数过：17 个有合格的步，
/// 偏移 0、1、7、10、14、15、19 一步都没有——那几段从 mkfs 起、一次文件都没发出来）。
const SEED_OFFSET_OF_THE_FIXED_HISTORY: u64 = 4;

/// 坏盘输入这一段历史本身怎么跑：不跑每一步的池级 checker（活盘面上每一步的 checker 由门禁 74 号那五段罩着，
/// 这一段的预算全给坏镜像），两块 4 GiB 的盘。
const UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES: HistoryExecution = HistoryExecution {
    per_step_checker: PerStepChecker::Skipped,
    device_width: HistoryDeviceWidth::FourGibibytes,
    space_admission: SpaceAdmission::JudgedByTheFormula,
};

/// 报告直接写进进程的标准输出，不经 libtest 的捕获：通过时计数照样出现在 `check.sh` 的输出里
/// （`test-discipline.md`「阴性结果要能和「代码没跑到」分开」）。
fn print_uncaptured(text: &str) {
    let mut standard_output = std::io::stdout();
    let _ = standard_output.write_all(text.as_bytes());
    let _ = standard_output.flush();
}

fn number_from_environment(name: &str, default: u64) -> u64 {
    std::env::var(name).map_or(default, |text| {
        text.parse()
            .unwrap_or_else(|error| panic!("{name}={text} 不是一个非负整数：{error}"))
    })
}

fn how_to_replay(report: &BadDiskReport) -> String {
    format!(
        "重放：SINGLEFS_BAD_DISK_FIRST_SEED={} SINGLEFS_BAD_DISK_SEEDS={} SINGLEFS_BAD_DISK_OPERATIONS={} 跑大档那条 #[ignore] 用例",
        report.first_seed, report.seed_count, report.operations_per_history
    )
}

fn fast_tier_campaign(worker_threads: BadDiskInputWorkerThreads) -> BadDiskCampaign {
    BadDiskCampaign {
        first_seed: SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        seed_count: FAST_TIER_SEEDS,
        operations_per_history: FAST_TIER_OPERATIONS_PER_HISTORY,
        weights: GenerationWeights::BROAD,
        execution: UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        worker_threads,
    }
}

/// 快档：一批坏镜像喂给恢复、可写挂载与池级 checker。两条判据——
/// 一、**一次都不许读回一版从没提交过的内容**（里程碑那一句的硬判据，报错与读回提交过的某一版都算合法）；
/// 二、panic 只许落在 `KNOWN_PANIC_SITES` 里（清单外的算新发现）。
///
/// 再钉三样覆盖：每一种坏法都真造出过坏镜像、每一种坏法造出的坏镜像都被三个读者里至少一个看见过、
/// 三个读者各自都跑满了这么多次。缺了覆盖这三条，「一次都没读回过没提交的内容」与「没撞出清单外的 panic」
/// 就分不清是真的没有，还是根本没跑到。
#[test]
fn bad_disk_inputs_never_read_back_an_uncommitted_version_and_panic_only_at_known_sites() {
    let report = run_bad_disk_campaign(&fast_tier_campaign(
        BadDiskInputWorkerThreads::from_the_environment_variable_named(
            BAD_DISK_INPUT_WORKER_THREADS_ENVIRONMENT_VARIABLE,
        ),
    ));
    print_uncaptured(&report.render());

    let damaged_images: u64 = report.tally.damaged_images_by_kind.values().sum();
    assert!(
        damaged_images > 0,
        "一份坏镜像都没造出来：{}",
        report.tally.render()
    );
    for kind in EVERY_DAMAGE_KIND {
        assert!(
            report
                .tally
                .damaged_images_by_kind
                .get(kind.name())
                .copied()
                .unwrap_or(0)
                > 0,
            "坏法「{}」在这一档里一次都没造出坏镜像（{} 段历史都没有它要坏的对象）：{}",
            kind.name(),
            report.seed_count,
            report.tally.render()
        );
        assert!(
            report
                .tally
                .damages_seen_by_a_reader_by_kind
                .get(kind.name())
                .copied()
                .unwrap_or(0)
                > 0,
            "坏法「{}」造出的坏镜像三个读者一次都没看见（没 panic、没判红、恢复与挂载都若无其事）：\
             它在这个盘面上什么也没碰到，这一格是空的，{}",
            kind.name(),
            report.tally.render()
        );
    }
    // C481（坏盘输入的基线镜像取不到树表 0 条的盘面）：基线按档抽，每一档都要真抽到过镜像、真造出过坏镜像。
    // 报告里那一句「基线档 N / M 档抽到过镜像」的 N 由这里钉住——把树表 0 条那一档从抽样里拿掉，N 由 2 变 1，这里红。
    assert_eq!(
        report.tally.base_image_tiers_sampled(),
        EVERY_BASE_IMAGE_TIER.len(),
        "有基线档一段历史都没抽到镜像：{}",
        report.tally.render()
    );
    assert!(
        report.render().contains(&format!(
            "基线档 {} / {} 档抽到过镜像",
            EVERY_BASE_IMAGE_TIER.len(),
            EVERY_BASE_IMAGE_TIER.len()
        )),
        "报告里报基线档数的那一句：{}",
        report.render()
    );
    for tier in EVERY_BASE_IMAGE_TIER {
        assert!(
            report
                .tally
                .damaged_images_by_tier
                .get(tier.name())
                .copied()
                .unwrap_or(0)
                > 0,
            "基线档「{}」上一份坏镜像都没造出来：{}",
            tier.name(),
            report.tally.render()
        );
    }
    assert_eq!(
        report.tally.damaged_images_by_tier.values().sum::<u64>(),
        damaged_images,
        "逐档的坏镜像数加起来就是逐坏法的"
    );
    assert_eq!(
        report.tally.recoveries, damaged_images,
        "每一份坏镜像都要喂给恢复"
    );
    assert_eq!(
        report.tally.mounts, damaged_images,
        "每一份都要喂给可写挂载"
    );
    assert_eq!(
        report.tally.checker_runs, damaged_images,
        "每一份都要喂给池级 checker"
    );

    assert_eq!(
        report.tally.read_back_versions_never_committed,
        0,
        "恢复读回了一版从没提交过的内容（里程碑增补 3 第 5 件的硬判据）：\n{}\n{}",
        report
            .findings
            .iter()
            .map(BadDiskFinding::render)
            .collect::<Vec<_>>()
            .join("\n"),
        how_to_replay(&report)
    );
    assert_eq!(
        report.tally.panics_outside_the_known_list,
        0,
        "撞到 KNOWN_PANIC_SITES 之外的 panic：\n{}\n{}",
        report
            .findings
            .iter()
            .map(BadDiskFinding::render)
            .collect::<Vec<_>>()
            .join("\n"),
        how_to_replay(&report)
    );

    // 清单只缩不涨，而且每条的次数要与登记的**相等**（2026-09-22 用户定的两条规矩）：
    // 一条修好了这里红在「撞不到了」上，逼着把那一行删掉；次数涨了这里红在「涨了」上——
    // 清单只按「文件 + 消息片段」认，同一个文件里同形的新 panic 会被现成的行静默接走，带上次数才拦得住
    // （`show-me-test.md`「踩过的坑要做成会失败的检查，不要做成提醒句」）。
    for site in KNOWN_PANIC_SITES {
        let hits = report
            .tally
            .known_panics_by_registry_name
            .get(site.registry_name)
            .copied()
            .unwrap_or(0);
        assert_eq!(
            hits, site.expected_hits_in_the_fast_tier,
            "KNOWN_PANIC_SITES 里的「{}」（普查 {}）这一档撞到 {hits} 次，登记的是 {} 次：\
             撞到 0 次多半是它已经修好了（那就把这一行删掉），也可能是打到它的那条坏法不灵了（那就是这一件的缺口）；\
             次数涨了是同一个文件里又多了一处同形的 panic，被这一行静默接走了——它是清单外的新发现，不许并进来。{}",
            site.registry_name,
            site.panic_survey_item,
            site.expected_hits_in_the_fast_tier,
            report.tally.render()
        );
    }
    // 清单空了（2026-09-22 第三批）⇒ 这一档里任何一处 panic 都落进「清单外的新发现」那一格，上面那条断言就是
    // 「三个读者一次都不许 panic」。这里把两格的数都打出来，好让通过时也看得见它们是 0（阴性结果要能和「没跑到」分开）。
    let panics_in_this_tier = report
        .tally
        .known_panics_by_registry_name
        .values()
        .sum::<u64>()
        + report.tally.panics_outside_the_known_list;
    print_uncaptured(&format!(
        "增补 3 的验收「三个读者都不许 panic」在这一档（写死的种子基、{FAST_TIER_SEEDS} 段 × {FAST_TIER_OPERATIONS_PER_HISTORY} 步）上成立：\
         KNOWN_PANIC_SITES 空着（{} 条），三个读者 panic {panics_in_this_tier} 次。\n",
        KNOWN_PANIC_SITES.len(),
    ));
}

/// 判别力自证（里程碑增补 3 第 5 件的验收逐字：「去掉恢复里一道校验和比对，判出「读回了没提交过的内容」」）。
///
/// 坏法是「把现行版本的数据单元整个重写成 0xA5，只重算这个单元自己的两道校验和（头校验和与载荷 CRC），
/// extent 记录里位置条目上的整单元校验和原样留着」。今天挡着它的只有
/// `crates/singlefs-core/src/recovery.rs` 的 `read_unit_via_locations` 那一句
/// `if crc32_castagnoli(&bytes) == location.unit_checksum`——两条位置条目都对不上，恢复报 `UnitUnreadable`，
/// 这一条判「报错」合法。把那一句改成恒真（`crates/mutations.tsv` 里那一条变异），恢复就把 0xA5 那一版读回来，
/// 这条用例判红在「读回了一版从没提交过的内容」上。
///
/// 内容那一句另外单钉一次：光看 `ReadBackVerdict` 分不出「读回的是别的提交过的版本」与「读回的是 0xA5」。
#[test]
fn the_location_entry_checksum_refuses_the_unit_that_was_resealed_with_only_its_own_checksums() {
    let base = base_image_for(HistorySeed(
        SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE.wrapping_add(SEED_OFFSET_OF_THE_FIXED_HISTORY),
    ));
    let mut random = SeededRandomSource::from_seed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE);
    let damaged = damage_image(
        &base.image,
        DamageKind::RewrittenDataUnitResealedWithItsOwnChecksums,
        &mut random,
    )
    .expect("这段历史的盘面上有 extent 树与它指的数据单元（起点就发过第一个文件）");
    print_uncaptured(&format!("坏在哪：{}\n", damaged.what));
    let observation = feed_a_damaged_image(
        &damaged,
        &base.committed_contents,
        &HistoryDeviceWidth::FourGibibytes.parameters(),
        HistoryDeviceWidth::FourGibibytes,
    );
    print_uncaptured(&format!(
        "恢复：{:?}\n可写挂载：{:?}\n读回判定：{:?}\n",
        observation.recovery, observation.mount, observation.read_back
    ));
    assert!(
        observation.recovery.panic().is_none(),
        "恢复在这份坏镜像上 panic 了：{:?}",
        observation.recovery
    );
    assert!(
        !matches!(
            observation.read_back,
            ReadBackVerdict::ReadBackAVersionNeverCommitted { .. }
        ),
        "恢复读回了一版从没提交过的内容：{:?}（坏在哪：{}）",
        observation.read_back,
        damaged.what
    );
    let ReaderOutcome::Finished { how } = &observation.recovery else {
        unreachable!("上一条断言已经排掉了 panic 那一支")
    };
    assert!(
        how.starts_with("Failed"),
        "位置条目里的整单元校验和是唯一挡着这份重写的东西，恢复该报错：{how}"
    );
}

/// 这一批修掉的每一处，各钉一格「拿到的是**哪一个**错误成员」——不是「没 panic」。
/// 判据是 `code-discipline.md`「错误」那一节：盘上读来的条目宽、槽号、跨度、设备身份可以是任何值，
/// 不是不变量，要在边界上验一次并返回错误成员；成员按调用方要做的决定分，所以这里比的是成员加它带的 `what`，
/// 不比消息文字。
///
/// 用的是写死的那一段历史（种子基 + [`SEED_OFFSET_OF_THE_FIXED_HISTORY`]）加写死的坏法，
/// 每一格的坏法与随机源都是定死的，重跑逐字相同。
///
/// **两个读者各钉一格，因为它们走的不是同一条路**：恢复走 `walk_to_file`，它在解条目之前先有
/// `read_tree_root` 那一道「根的 key 区间贴紧首末条目」（`recovery.rs` 的 `InvariantViolated { invariant: "I-1.1" }`），
/// 凡是动了首末条目 key 的坏法（条目数从 94 / 15 缩成 1、第一条记录的 key 被换掉）都先红在那里；
/// 可写挂载重建分配器走的是 `allocation_records_under_root`，不经那一道，于是落在这一批新加的边界判上。
/// 两格都钉，才看得出**这一批的判定真的被走到过**——只钉恢复那一格的话，把新加的判全删掉，
/// 恢复照样红在 I-1.1 上，这条用例不会响。
#[test]
fn every_fixed_panic_site_reports_its_own_error_member_instead_of_panicking() {
    let base = base_image_for(HistorySeed(
        SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE.wrapping_add(SEED_OFFSET_OF_THE_FIXED_HISTORY),
    ));
    // (坏法, 恢复那一侧交回里必然出现的那一段, 可写挂载那一侧交回里必然出现的那一段)。
    // 前四条是条目宽（普查 R1 / R3 / R4），中间四条是喂进分配器的结构值（普查 R6 / R8 / R9），
    // 第九条是第四批的 R2（中央映射树根的条目宽，它的根住根记录、不经树表），
    // 第十条是第四批的 R10（两块盘的分配记录树不对称：实例表那个落点只在盘 1 上写成已释放），
    // 第十一、十二条是第三批修的那两族——记账行的统计量标签（R11）与一条指针的两条位置条目槽号（R5），
    // 末一条是 C504（树表条目宽在走读里无守卫，今天没坏法打得到）：树表单元自己的条目宽；
    // `what` 那一段就是错误成员里带的那个字段，逐字照 `crates/singlefs-core/src/recovery.rs` 写。
    //
    // ⚠️ 末两条的恢复那一侧本来就不该报错，照实钉：
    // 记账行改标签的那份镜像上，恢复走 `walk_to_file`，先红在 I-1.1（标签在记账 key 里，根的 key 区间跟着对不上）；
    // 位置条目那份上恢复**读回了文件**（那条指针指的实例表不在读路径上，两条位置条目的第一条又还是好的）——
    // 这一条钉的正是「改法没有把读路径也一起拒掉」。
    //
    // ⚠️ 这段写死的历史第 4 步是「回退到环里第 3 新的根」，那是一条暖机根、而环里还留着带文件的根：C511（回退到无文件那一版之后诞生代怎么接）
    // 第 3 步之前那一步在写之前被拒、之后几步没有会话；拿掉那道拒绝之后它照常回退，后面接着再发第一个文件版本、覆盖写、重开（实例 3），
    // 盘面跟着变：实例表那个落点从槽 50304 挪到 50368，恢复择到的根从 (2, 9) 变成 (3, 13)。下面几处槽号与根照今天的盘面写，
    // 钉的错误成员一个没变。
    //
    // ⚠️ 「跨度越过单元区末尾」那条坏法在**两块 4 GiB 的盘**上够不着它名字里说的那一格：跨度写成 0x7FFF = 32767 槽，
    // 从 50176 起到 82943，而单元区末尾是槽 262144（4 GiB ÷ 16 KiB）。分配记录树按位置寻址之后，它先撞上的是叶判
    // （末槽越过它所在叶的末槽）。照实钉，不按坏法的名字钉。
    let expected_error_member: [(DamageKind, &str, &str); 13] = [
        // extent 树按位置寻址（D8（核心索引结构） 已定项 14）：第一个文件那一版的 extent 树根是上段根兼叶，条目是上段叶条目 113。
        (
            DamageKind::NarrowedEntryWidthOfTheExtentTreeRoot,
            "EntryNarrowerThanItsFieldTable { what: \"extent 上段叶条目\"",
            "EntryNarrowerThanItsFieldTable { what: \"extent 上段叶条目\"",
        ),
        (
            DamageKind::NarrowedEntryWidthOfTheInodeTreeRoot,
            "EntryNarrowerThanItsFieldTable { what: \"inode 树内部条目\"",
            "EntryNarrowerThanItsFieldTable { what: \"inode 树内部条目\"",
        ),
        // 分配记录树按位置寻址、根恒在第 1 层以上（D8（核心索引结构） 已定项 14）：根的条目是内部条目 96，两个读者都走整棵读，
        // 先红在内部条目的宽度上。
        (
            DamageKind::NarrowedEntryWidthOfTheAllocationRecordsTreeRoot,
            "EntryNarrowerThanItsFieldTable { what: \"分配记录树内部条目\"",
            "EntryNarrowerThanItsFieldTable { what: \"分配记录树内部条目\"",
        ),
        (
            DamageKind::NarrowedEntryWidthOfTheAccountingTreeRoot,
            "InvariantViolated { invariant: \"I-1.1\", detail: \"根 key 区间与条目不符\" }",
            "EntryNarrowerThanItsFieldTable { what: \"记账条目\"",
        ),
        // 这四条坏的是最左那片叶的记录（分配记录树按位置寻址，D8（核心索引结构） 已定项 14）：记录的槽号、跨度、设备改了，
        // 它就不在它所在叶按位置罩的那一段里，读树那一步的叶判先接走，两个读者同一个成员；两条记录的 key 撞在一起时
        // 先红在「叶里的记录不按 key 严格递增」。`recovery::allocation_records_fit_the_pool_geometry` 那四样判在这四条上被叶判遮住，
        // 它们只在罩着盘末尾的那片叶上够得着（交回里写明）。
        (
            DamageKind::AllocationRecordSlotBelowTheUnitArea,
            "AllocationRecordOutsideThePoolGeometry { what: \"分配记录不在它所在叶按位置罩的那一段里，或末槽越过叶的末槽\" }",
            "AllocationRecordOutsideThePoolGeometry { what: \"分配记录不在它所在叶按位置罩的那一段里，或末槽越过叶的末槽\" }",
        ),
        (
            DamageKind::AllocationRecordSpanPastTheEndOfTheUnitArea,
            "AllocationRecordOutsideThePoolGeometry { what: \"分配记录不在它所在叶按位置罩的那一段里，或末槽越过叶的末槽\" }",
            "AllocationRecordOutsideThePoolGeometry { what: \"分配记录不在它所在叶按位置罩的那一段里，或末槽越过叶的末槽\" }",
        ),
        (
            DamageKind::TwoAllocationRecordsCoveringTheSameSlot,
            "InvariantViolated { invariant: \"I-1.1\", detail: \"分配记录树叶里的记录不按 key 严格递增\" }",
            "InvariantViolated { invariant: \"I-1.1\", detail: \"分配记录树叶里的记录不按 key 严格递增\" }",
        ),
        (
            DamageKind::AllocationRecordOnADeviceOutsideThePool,
            "AllocationRecordOutsideThePoolGeometry { what: \"分配记录不在它所在叶按位置罩的那一段里，或末槽越过叶的末槽\" }",
            "AllocationRecordOutsideThePoolGeometry { what: \"分配记录不在它所在叶按位置罩的那一段里，或末槽越过叶的末槽\" }",
        ),
        (
            DamageKind::NarrowedEntryWidthOfTheCentralMappingTreeRoot,
            "InvariantViolated { invariant: \"I-1.1\", detail: \"根 key 区间与条目不符\" }",
            "EntryNarrowerThanItsFieldTable { what: \"映射条目\"",
        ),
        (
            DamageKind::AllocationRecordReleasedOnTheSecondDeviceOnly,
            "InvariantViolated { invariant: \"E142 走读同款\", detail: \"分配记录不是每个落点每盘各一条",
            "ReleaseTargetAlreadyReleased { unit: InstanceTable, device: DeviceIdentity(1), \
             slot: SlotNumber(50368) }",
        ),
        (
            DamageKind::RelabelledInodeWatermarkAccountingRow,
            "InvariantViolated { invariant: \"I-1.1\", detail: \"根 key 区间与条目不符\" }",
            "InodeNumberWatermarkRowMissingFromTheAccountingTree",
        ),
        (
            DamageKind::TwoLocationEntriesOfOnePointerDisagreeingOnTheSlot,
            "FileRead（实例 3 第 13 代根",
            "ReleaseTargetLocationsOnDifferentSlots { unit: InstanceTable, disagreement: \
             LocationEntriesOnDifferentSlots { devices: [DeviceIdentity(0), DeviceIdentity(1)], \
             slots: [SlotNumber(50368), SlotNumber(50369)] } }",
        ),
        // 第十三条是 C504（树表条目宽在走读里无守卫，今天没坏法打得到）：坏的是**树表单元自己**，不是它指着的某棵树的根。
        // 两个读者都走 `TreeTableEntry::parse`（它先判「这条条目正好 200 字节」），所以两侧同一个成员；
        // 池级 checker 走的是另一条路（`walk.rs` 的 `walk_tree_table_entry`），它今天交回的形态是「走读断了」不是违例，
        // 由上面那条「三个读者都不许 panic」与快档的 panic 计数钉住。
        (
            DamageKind::NarrowedEntryWidthOfTheTreeTableUnit,
            "UnitMalformed { what: \"树表条目\" }",
            "UnitMalformed { what: \"树表条目\" }",
        ),
    ];
    for (kind_index, (kind, member_from_recovery, member_from_mount)) in
        expected_error_member.into_iter().enumerate()
    {
        let mut random = SeededRandomSource::from_seed(
            SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE
                .wrapping_add(u64::try_from(kind_index).expect("坏法序号")),
        );
        let damaged = damage_image(&base.image, kind, &mut random)
            .unwrap_or_else(|| panic!("坏法「{}」在这段历史的盘面上没有可坏的对象", kind.name()));
        let observation = feed_a_damaged_image(
            &damaged,
            &base.committed_contents,
            &HistoryDeviceWidth::FourGibibytes.parameters(),
            HistoryDeviceWidth::FourGibibytes,
        );
        print_uncaptured(&format!(
            "{}\n  坏在哪：{}\n  恢复：{:?}\n  可写挂载：{:?}\n",
            kind.name(),
            damaged.what,
            observation.recovery,
            observation.mount
        ));
        for (reader, outcome) in [
            ("恢复", &observation.recovery),
            ("可写挂载", &observation.mount),
            ("池级 checker", &observation.checker),
        ] {
            assert!(
                outcome.panic().is_none(),
                "坏法「{}」把{reader}打 panic 了：{outcome:?}",
                kind.name()
            );
        }
        for (reader, outcome, member) in [
            ("恢复", &observation.recovery, member_from_recovery),
            ("可写挂载", &observation.mount, member_from_mount),
        ] {
            let ReaderOutcome::Finished { how } = outcome else {
                unreachable!("上一条断言已经排掉了 panic 那一支")
            };
            assert!(
                how.contains(member),
                "坏法「{}」该让{reader}交回 {member}，实际交回的是 {how}（坏在哪：{}）",
                kind.name(),
                damaged.what
            );
        }
    }
}

/// 普查 R5 的**第二个落点**：树表 0 条的一版（只做过 mkfs 的池）上，这一版的落点就是根记录直接指着的实例表与树表
/// 两个单元，槽号从那两条指针的位置条目里取（`mount.rs` 的 `format_time_allocator`）。上一条用例那段历史有文件版本，
/// 走的是释放判定那条路，够不着这一处——两处各钉一格，两格才都被走到过。
///
/// 盘面写死：只做过 mkfs 的那份镜像（起点观测，一步操作都不做）。坏法写死：那条根记录里实例表指针第二条位置条目的
/// 槽号 +1、整槽自证校验和重算。判两样——交回的是 `FormatTimeUnitLocationsOnDifferentSlots`（不是别的成员、更不是 panic），
/// 以及**盘上逐字节不变**：判定在动分配器与取号之前，这次挂载一个字节都不许写。
#[test]
fn a_format_time_pointer_with_two_location_entries_on_different_slots_refuses_the_writable_mount_without_writing_a_byte(
) {
    let base = image_of_a_pool_that_has_only_been_made();
    let mut random = SeededRandomSource::from_seed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE);
    let damaged = damage_image(
        &base,
        DamageKind::TwoLocationEntriesOfOnePointerDisagreeingOnTheSlot,
        &mut random,
    )
    .expect("只做过 mkfs 的盘面上有一条根记录，它的实例表指针不是全零");
    let mut devices = block_devices_of(&damaged.image);
    let before = memory_pool_of(&devices);
    let error = mount_writable(
        &HistoryDeviceWidth::FourGibibytes.parameters(),
        &mut devices,
    )
    .expect_err("两条位置条目落在不同的槽，这份镜像该被拒");
    print_uncaptured(&format!("坏在哪：{}\n可写挂载：{error:?}\n", damaged.what));
    let MountError::FormatTimeUnitLocationsOnDifferentSlots { unit, disagreement } = &error else {
        panic!(
            "该交回 FormatTimeUnitLocationsOnDifferentSlots，实际交回的是 {error:?}（坏在哪：{}）",
            damaged.what
        )
    };
    assert_eq!(
        *unit,
        TransactionUnit::InstanceTable,
        "坏的是实例表那条指针"
    );
    assert_eq!(
        disagreement.slots[1].0,
        disagreement.slots[0].0 + 1,
        "坏法把第二条位置条目的槽号加了一：{disagreement:?}"
    );
    assert_eq!(
        memory_pool_of(&devices),
        before,
        "拒绝发生在动分配器与取号之前，这次挂载一个字节都不许写"
    );
}

/// 一份还没有文件版本的盘面（树表 0 条）：mkfs 之后可写挂载一次，取最后那次观测的镜像。
///
/// 为什么不停在 mkfs 那一步：mkfs 把创世根写在**每块盘**上，坏法只改其中一块盘上的那一份，择根会挑另一块盘上
/// 完好的那一份，挂载照样成功（2026-09-22 实测：挂载交回 `Ok`，所选根的两条位置条目都还是槽 50176）。
/// 可写挂载之后最新那条根是暖机那次零单元发布写的，只落在一块盘上，坏它才是择根真会挑中的那一条；
/// 这一版的树表仍是 0 条、实例表与树表指针仍是 mkfs 那两个（诞生 txg 0），走的正是 `format_time_allocator`。
fn image_of_a_pool_that_has_only_been_made() -> MemoryPool {
    let history = GeneratedHistory {
        seed: HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE),
        starting_point: HistoryStartingPoint::AfterMakeFilesystem,
        operations: vec![HistoryOperation::CloseAndMountWritable],
    };
    let mut image = None;
    execute_history_with(
        &history,
        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        &SharedStream::new(),
        &mut |observation| image = Some(observation.image.clone()),
    );
    image.expect("mkfs 做完之后有一次起点观测")
}

/// 把一份镜像接成可写挂载要的那一串块设备（每块盘一份内存里的稀疏盘）。
fn block_devices_of(image: &MemoryPool) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    image
        .devices
        .iter()
        .map(|(identity, sparse)| {
            let mut device = SparseBlockDevice::new(
                HistoryDeviceWidth::FourGibibytes.device_bytes(),
                PhysicalBlockSizeInBytes(512),
            );
            device.image = sparse.clone();
            (*identity, device)
        })
        .collect()
}

/// 这几块盘此刻的整份镜像（拿来与挂载之前的逐字节比）。
fn memory_pool_of(devices: &[(DeviceIdentity, SparseBlockDevice)]) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: HistoryDeviceWidth::FourGibibytes.device_bytes(),
    }
}

/// 系统配置自述的区域数越过字段表那三个逐区域设备身份字段（普查 R12）时，池级 checker 不许 panic、也不许把它夹成 3
/// 往下走：`geometry_of` 交回 `Verdict::RegionCountPastTheRegionDeviceFields`、这一槽不可择，于是整片报不适用。
/// 「不夹成 3」那一半单钉在 `checker_known_bad_images.rs`
/// （`a_region_count_past_the_region_device_fields_is_refused_by_the_geometry_reader`）。
#[test]
fn a_region_count_past_the_three_region_array_leaves_the_checker_standing() {
    let base = base_image_for(HistorySeed(
        SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE.wrapping_add(SEED_OFFSET_OF_THE_FIXED_HISTORY),
    ));
    let mut random = SeededRandomSource::from_seed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE);
    let damaged = damage_image(
        &base.image,
        DamageKind::RegionCountPastTheThreeRegionArray,
        &mut random,
    )
    .expect("这段历史的盘面上有系统配置槽");
    let observation = feed_a_damaged_image(
        &damaged,
        &base.committed_contents,
        &HistoryDeviceWidth::FourGibibytes.parameters(),
        HistoryDeviceWidth::FourGibibytes,
    );
    print_uncaptured(&format!(
        "坏在哪：{}\n池级 checker：{:?}\n判红：{:?}\n",
        damaged.what, observation.checker, observation.invariants_violated
    ));
    assert!(
        observation.checker.panic().is_none(),
        "池级 checker 在这份坏镜像上 panic 了：{:?}",
        observation.checker
    );
    assert_eq!(
        observation.invariants_violated,
        Vec::<&'static str>::new(),
        "每块盘的系统配置都读不出几何，checker 该整片报不适用、不判任何一条违例"
    );
}

/// C504（树表条目宽在走读里无守卫，今天没坏法打得到）：树表单元**自己**自述的条目宽缩到 8 时，池级 checker 不许 panic，
/// 判红的形态要是「走读断了」——走读失败让 I-4.8（近 K 代根校验和自洽） 与 I-7.4（近 K 代块未被复用） 判红，
/// 而**不是**在树表上判 I-1.10（码 2 条目宽等于字段表宽）（用户 2026-09-23 定案「只加走读守卫，不动条款」，
/// I-1.10（码 2 条目宽等于字段表宽） 的射程一个字不扩；它判的是树表**指着的**那棵树的根节点，住在这两处切片之后）。
///
/// 坏法写死、盘面写死（种子基 + [`SEED_OFFSET_OF_THE_FIXED_HISTORY`] 那一段历史），重跑逐字相同。
#[test]
fn a_tree_table_narrower_than_a_registered_entry_breaks_the_walk_instead_of_panicking_or_judging_the_entry_width(
) {
    let base = base_image_for(HistorySeed(
        SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE.wrapping_add(SEED_OFFSET_OF_THE_FIXED_HISTORY),
    ));
    let mut random = SeededRandomSource::from_seed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE);
    let damaged = damage_image(
        &base.image,
        DamageKind::NarrowedEntryWidthOfTheTreeTableUnit,
        &mut random,
    )
    .expect("这段历史的盘面上有树表单元");
    let observation = feed_a_damaged_image(
        &damaged,
        &base.committed_contents,
        &HistoryDeviceWidth::FourGibibytes.parameters(),
        HistoryDeviceWidth::FourGibibytes,
    );
    print_uncaptured(&format!(
        "坏在哪：{}\n池级 checker：{:?}\n判红：{:?}\n",
        damaged.what, observation.checker, observation.invariants_violated
    ));
    assert!(
        observation.checker.panic().is_none(),
        "池级 checker 在这份坏镜像上 panic 了（走读守卫没拦住 read_u16(entry, 10) 那一步）：{:?}",
        observation.checker
    );
    assert!(
        observation.invariants_violated.contains(&"I-4.8")
            && observation.invariants_violated.contains(&"I-7.4"),
        "走读断了要由 I-4.8 与 I-7.4 说话，实际判红的是 {:?}",
        observation.invariants_violated
    );
    assert!(
        !observation.invariants_violated.contains(&"I-1.10"),
        "树表自己的条目宽不许算成 I-1.10 的违例（那一条的射程是树表指着的那棵树的根节点）：{:?}",
        observation.invariants_violated
    );
}

/// 每一种坏法都真的改了它名字里说的那个字段：坏法只说「改了点什么」不够，报告里写的是「哪块盘哪个偏移的哪个字段」，
/// 那句话要对得上盘面。这里对写死的一段历史逐个坏法跑一遍，核两样——坏完的镜像与坏之前逐字节不同，
/// 而且坏之前那份镜像一个字节都没被动过（坏法拿的是拷贝）。
#[test]
fn every_damage_kind_changes_the_image_and_leaves_the_base_image_untouched() {
    let base = base_image_for(HistorySeed(
        SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE.wrapping_add(SEED_OFFSET_OF_THE_FIXED_HISTORY),
    ));
    let before = base.image.clone();
    for (kind_index, kind) in EVERY_DAMAGE_KIND.into_iter().enumerate() {
        let mut random = SeededRandomSource::from_seed(
            SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE
                .wrapping_add(u64::try_from(kind_index).expect("坏法序号")),
        );
        let damaged = damage_image(&base.image, kind, &mut random)
            .unwrap_or_else(|| panic!("坏法「{}」在这段历史的盘面上没有可坏的对象", kind.name()));
        // 两份镜像各几十 MiB，判不等只报一句话，不把 `MemoryPool` 的 Debug 打出来。
        assert!(
            damaged.image != before,
            "坏法「{}」说改了「{}」，镜像却逐字节没变",
            kind.name(),
            damaged.what
        );
        assert!(
            base.image == before,
            "坏法「{}」改到了基线镜像自己，后面每一种坏法就都坏在上一种的结果上了",
            kind.name()
        );
        print_uncaptured(&format!("{} → {}\n", kind.name(), damaged.what));
    }
}

/// 多线程那一条（`.claude/rules/implementation-workflow.md`「测试与崩溃检测优先多线程」）：
/// 计数按片的次序相加、新发现按种子从小到大留第一个、按种子次序的摘要按片号并，所以报告与线程数无关。
/// 1 与 4 逐字相同。
///
/// 种子数取 [`DETERMINISM_SEEDS`] 是为了让两边的**切法**真的不同：片数是 min(种子数, 4 × 线程数)，
/// 6 个种子配 1 个线程切成 4 片（各 1、2、1、2 个种子）、配 4 个线程切成 6 片（每片 1 个）。
/// 种子数 ≤ 4 时两边都是「每片 1 个种子」，切法一样，这条用例就只剩「到达次序」一个变量，
/// 而到达次序是掷骰子——按它判红的变异有时红有时绿，钉不住。
#[test]
fn the_report_is_the_same_with_one_worker_thread_and_four() {
    let single = run_bad_disk_campaign(&BadDiskCampaign {
        seed_count: DETERMINISM_SEEDS,
        operations_per_history: DETERMINISM_OPERATIONS_PER_HISTORY,
        ..fast_tier_campaign(BadDiskInputWorkerThreads::GivenByCaller(1))
    });
    let four = run_bad_disk_campaign(&BadDiskCampaign {
        seed_count: DETERMINISM_SEEDS,
        operations_per_history: DETERMINISM_OPERATIONS_PER_HISTORY,
        ..fast_tier_campaign(BadDiskInputWorkerThreads::GivenByCaller(4))
    });
    assert_eq!(
        single.render(),
        four.render(),
        "1 个工作线程与 4 个跑出来的报告不一样：并起来的次序或者留第一个的规则与线程数有关"
    );
    assert_eq!(single.tally, four.tally, "计数与线程数有关");
    assert_eq!(
        single.in_order_digest, four.in_order_digest,
        "按种子次序的摘要与线程数有关：各段是按到达次序并起来的，不是按片号"
    );
}

/// 「已知红」清单按「文件 + 消息片段」认人，不按行号认：`crates/singlefs-core/` 这几轮另有会话在改，
/// 行号一漂，按行号写的清单会悄悄失效（撞到的 panic 变成「清单外」，用例红在一个假的新发现上）。
///
/// `KNOWN_PANIC_SITES` 2026-09-22 第三批之后空了，空清单怎么认都交回 `None` ⇒ 认人那条规矩在真清单上
/// 没有对象可判。所以这里拿**第三批删掉的那三条**当样本清单喂 `known_panic_site_among`：认人的规矩照旧有东西守着，
/// 同时钉住第二件事——这三处在**真清单**上一条都认不出来了（清单只许缩，`KNOWN_PANIC_SITES` 的第 2 条规矩；
/// 谁把它们加回去，这里先红）。
#[test]
fn the_known_panic_list_matches_by_file_and_message_not_by_line_number() {
    for site in SITES_DELETED_BY_THE_THIRD_BATCH {
        assert!(
            !site.file.is_empty() && !site.message_fragment.is_empty(),
            "样本清单里有一条没写文件或消息片段：{site:?}"
        );
        assert!(
            site.file
                .chars()
                .all(|character| !character.is_ascii_digit()),
            "样本清单里的文件写了行号（{}）：行号会漂，按「文件 + 消息片段」认",
            site.file
        );
        let same_site_at_another_line = CapturedPanic {
            location: format!("{}:99999", site.file),
            message: format!("{} 后面还跟着别的话", site.message_fragment),
        };
        assert_eq!(
            known_panic_site_among(
                &SITES_DELETED_BY_THE_THIRD_BATCH,
                &same_site_at_another_line
            )
            .map(|found| found.panic_survey_item),
            Some(site.panic_survey_item),
            "同一处 panic 换了行号就认不出来了：{site:?}"
        );
        assert!(
            known_panic_site_of(&same_site_at_another_line).is_none(),
            "「{}」（普查 {}）又回到 KNOWN_PANIC_SITES 里了：清单只许缩，\
             加行等于把「写死的种子数上零 panic」那条验收撤回，要主 agent 点头",
            site.registry_name,
            site.panic_survey_item
        );
    }
    let never_seen = CapturedPanic {
        location: "crates/singlefs-core/src/somewhere_new.rs:1".to_string(),
        message: "这条消息不在清单里".to_string(),
    };
    assert!(
        known_panic_site_among(&SITES_DELETED_BY_THE_THIRD_BATCH, &never_seen).is_none(),
        "清单外的 panic 被认成了清单里的某一条"
    );
    assert!(
        KNOWN_PANIC_SITES.is_empty(),
        "KNOWN_PANIC_SITES 不空了：{KNOWN_PANIC_SITES:?}"
    );
}

/// 2026-09-22 第三批改法删掉的三条。留在用例里当样本清单，不是登记位——
/// 登记位是 [`KNOWN_PANIC_SITES`]，它现在空着（三处都改成了返回错误成员）。
const SITES_DELETED_BY_THE_THIRD_BATCH: [KnownPanicSite; 3] = [
    KnownPanicSite {
        registry_name: "记账树里没有 inode 号水位那一行",
        panic_survey_item: "R11",
        file: "crates/singlefs-core/src/transaction.rs",
        message_fragment: "每次发布都写 inode 号水位那一行",
        how_it_is_reached:
            "「记账树里 inode 号水位那一行被改挂到别的标签上」；挂载之后第一次发布取水位",
        expected_hits_in_the_fast_tier: 8,
    },
    KnownPanicSite {
        registry_name: "释放判定里的两盘同槽断言",
        panic_survey_item: "R5",
        file: "crates/singlefs-core/src/transaction.rs",
        message_fragment: "两盘同槽",
        how_it_is_reached:
            "「一条指针的两条位置条目槽号不等」；`placements_to_release_via_mapping`",
        expected_hits_in_the_fast_tier: 8,
    },
    KnownPanicSite {
        registry_name: "挂载重建分配器时的两盘同槽断言",
        panic_survey_item: "R5",
        file: "crates/singlefs-core/src/mount.rs",
        message_fragment: "两盘同槽",
        how_it_is_reached:
            "「一条指针的两条位置条目槽号不等」；`format_time_allocator` 的 `placement_of`",
        expected_hits_in_the_fast_tier: 4,
    },
];

/// 大档：规模从环境变量取，后台跑。
#[test]
#[ignore = "大档按环境变量跑，不进每次的 cargo test"]
fn the_large_tier_of_bad_disk_inputs() {
    let first_seed = number_from_environment(
        "SINGLEFS_BAD_DISK_FIRST_SEED",
        SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
    );
    let seed_count = number_from_environment("SINGLEFS_BAD_DISK_SEEDS", 500);
    let operations_per_history = usize::try_from(number_from_environment(
        "SINGLEFS_BAD_DISK_OPERATIONS",
        u64::try_from(FAST_TIER_OPERATIONS_PER_HISTORY).expect("步数"),
    ))
    .expect("步数装得进 usize");
    let report = run_bad_disk_campaign(&BadDiskCampaign {
        first_seed,
        seed_count,
        operations_per_history,
        weights: GenerationWeights::BROAD,
        execution: UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        worker_threads: BadDiskInputWorkerThreads::from_the_environment_variable_named(
            BAD_DISK_INPUT_WORKER_THREADS_ENVIRONMENT_VARIABLE,
        ),
    });
    print_uncaptured(&report.render());
    assert_eq!(
        report.tally.read_back_versions_never_committed,
        0,
        "恢复读回了一版从没提交过的内容：{}",
        how_to_replay(&report)
    );
    assert_eq!(
        report.tally.panics_outside_the_known_list,
        0,
        "撞到 KNOWN_PANIC_SITES 之外的 panic：{}",
        how_to_replay(&report)
    );
}

/// 一段写死的历史跑完之后手里那份合法镜像，连同理想模型提交过的每一版内容。
struct BaseImageForOneHistory {
    image: singlefs_harness::crash::MemoryPool,
    committed_contents: std::collections::BTreeSet<Option<Vec<u8>>>,
}

/// 跑一段写死的历史，留下**最后一步「该有的那几棵树都在」的那份**合法镜像与模型提交过的每一版内容。
/// 写死的那几条用例要的是一份确定的、指着普查那几族的坏法都做得出来的盘面，所以这里取最后一份合格的，不按种子抽
/// （按种子抽是 campaign 那一路的事）。
fn base_image_for(seed: HistorySeed) -> BaseImageForOneHistory {
    let history = generate_history_with_weights(
        seed,
        FAST_TIER_OPERATIONS_PER_HISTORY,
        &GenerationWeights::BROAD,
    );
    let mut committed_contents = std::collections::BTreeSet::new();
    let mut image = None;
    execute_history_with(
        &history,
        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        &SharedStream::new(),
        &mut |observation| {
            for (_, file) in observation.model.committed_versions() {
                committed_contents.insert(file.map(|content| content.to_vec()));
            }
            if newest_root_has_the_trees_the_targeted_damages_need(observation.image) {
                image = Some(observation.image.clone());
            }
        },
    );
    BaseImageForOneHistory {
        image: image.expect(
            "这段写死的历史里至少有一步之后，最新那条根下面 extent / 分配记录 / 记账三棵树都在",
        ),
        committed_contents,
    }
}
