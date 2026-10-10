//! harness 判法复算判据 N-配置 时，系统配置见证判不出改取 R2（D22（单元原子性怎么合成） 已定项 8 射程，用户 2026-10-03 定「改取 R2」；
//! D23（journal 的角色与格式） 已定项 14 那一段；代码轮判决 `research/prompts/m3-root-ring-code-r1-main-verification.md` 第二节 RR8）：
//! 判不出 ⟺ 有槽设备报读错，且没有一块盘两槽都自证过。撕裂的轮换（读得出、自证不过、设备没报错）不算判不出，c_见证 跳过撕裂的槽照常比；
//! 某块盘两槽一份自证过的都没有（C554 乙-配置续 Q1，这块盘不「可见」）：实现拒它报的是「读阶段判完之后系统配置又读不出」，
//! 那一拒照它自己的判据复算为真；拿判据 N-配置 当理由拒它是拒错了（C664（挂载拒绝合法复算把两种拒因报成一个））。
//! 小池是单元区 384 槽的两块内存盘，第一个文件之后的盘面（mkfs → 取号 → 暖机 → 第一个文件）。
//! 判的拒都是用例自己拼的一份「判据 N-配置 为真、重读仍为真」：判法自己在崩溃镜像上复算判据，不看拒里填的读数。
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::mount::{
    MountError, NewerPublishWitness, RollbackTarget, SelectedVersionAgainstTheWitness,
    StillUnreadableAfterOneReread, SystemConfigurationUnreadableAfterTheReadStage,
    SystemConfigurationUnreadableWhen, WitnessedCounterComparison,
};
use singlefs_core::recovery::verified_system_configuration_slots;
use singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE;
use singlefs_harness::history::{
    execute_history_with, GeneratedHistory, HistoryDeviceWidth, HistoryExecution, HistorySeed,
    HistoryStartingPoint, PerStepChecker,
};
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::mount_transparency::{
    judge_writable_mount_refusal, read_recovery_fields, ComparedVersion, DiskAfterTheRefusal,
    ReadOnlyReadBackOfTheCrashImage, RefusalConditionNotMet, RefusalReasonOnTheD23List,
    WritableMountRefusalVerdict,
};
use singlefs_harness::SharedStream;

const SMALL: HistoryDeviceWidth = HistoryDeviceWidth::UnitAreaOf384Slots;
const DEVICES: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];
/// 第一个文件那次系统配置轮换写的世代号：每块盘世代 5 那一槽（槽号 5 mod 2 = 1）。
const GENERATION_OF_THE_FIRST_FILE_ROTATION: u64 = 5;
/// 撕裂一个系统配置槽时翻的那个字节在槽里的下标：槽中间，落在整槽校验和罩着的范围里。
const TORN_BYTE_INSIDE_THE_SYSTEM_CONFIGURATION_SLOT: u64 = 2048;

fn pool_after_the_first_file() -> MemoryPool {
    let stream = SharedStream::retaining_contents();
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: Vec::new(),
    };
    execute_history_with(
        &history,
        HistoryExecution {
            per_step_checker: PerStepChecker::Skipped,
            device_width: SMALL,
            space_admission: SpaceAdmission::JudgedByTheFormula,
        },
        &stream,
        &mut |_observation| {},
    );
    let mut pool = MemoryPool::with_devices(&DEVICES, SMALL.device_bytes());
    pool.apply(&stream.retained_operations());
    pool
}

fn offset_of_system_configuration_slot(slot_index: u64) -> DeviceOffsetInBytes {
    DeviceOffsetInBytes(
        slot_index * u64::from(SMALL.parameters().geometry.fixed_structure_slot_spacing),
    )
}

/// 判据 N-配置 为真、重读一次仍为真的一拒（判法自己复算判据，不看这里填的读数）。
fn witness_refusal_claimed_on(selected_version: (InstanceGeneration, CheckpointTxg)) -> MountError {
    let reading = SelectedVersionAgainstTheWitness {
        selected_version: RollbackTarget {
            instance: selected_version.0,
            checkpoint_txg: selected_version.1,
        },
        witness: NewerPublishWitness {
            witnessed_journal_counter: 1,
            comparison: WitnessedCounterComparison::Undecidable,
        },
    };
    MountError::NewerStateStillUnreadableAfterOneReread(Box::new(
        StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
            first_read: reading,
            reread: reading,
        },
    ))
}

/// 取号之前的逐盘核点名盘 0 一份自证过的系统配置都没有的一拒（判法自己复算判据，不看这里填的读数）。
fn refusal_after_the_read_stage_claimed_on(
    selected_version: (InstanceGeneration, CheckpointTxg),
) -> MountError {
    MountError::SystemConfigurationUnreadableAfterTheReadStage(Box::new(
        SystemConfigurationUnreadableAfterTheReadStage {
            read_stage: SelectedVersionAgainstTheWitness {
                selected_version: RollbackTarget {
                    instance: selected_version.0,
                    checkpoint_txg: selected_version.1,
                },
                witness: NewerPublishWitness {
                    witnessed_journal_counter: 1,
                    comparison: WitnessedCounterComparison::NothingWitnessed,
                },
            },
            unreadable: SystemConfigurationUnreadableWhen::CheckingEachDeviceBeforeTheAcquisition {
                device: DEVICES[0],
            },
        },
    ))
}

/// 在崩溃镜像 `image` 上判 `claimed` 拼出的那一拒：所选那一版取只读恢复落到的那一版，盘上逐字节不变、只读读出在允许集合里（U6 另两条都满足）。
fn verdict_of_a_claimed_refusal_on(
    image: &MemoryPool,
    claimed: fn((InstanceGeneration, CheckpointTxg)) -> MountError,
) -> WritableMountRefusalVerdict {
    let fields = read_recovery_fields(image, ComparedVersion::TheRecoveredVersion);
    let selected_version = fields
        .report
        .effective_root
        .expect("撕裂的系统配置槽上只读恢复照样落到第一个文件那一版");
    judge_writable_mount_refusal(
        image,
        fields.report.effective_root,
        &claimed(selected_version),
        DiskAfterTheRefusal::ByteForByteUnchanged,
        ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
    )
}

fn verdict_of_the_claimed_witness_refusal_on(image: &MemoryPool) -> WritableMountRefusalVerdict {
    verdict_of_a_claimed_refusal_on(image, witness_refusal_claimed_on)
}

/// 场景：两块盘第一个文件那次轮换写的那一槽（世代 5，槽 1）都撕裂——读得出、自证不过，设备不报错。
/// 预期：R2 下这不是判不出，c_见证 取两块盘世代 4 那一槽、不比第一个文件那条记录的计数器大，判据复算为假：拿它当理由拒可写是拒错了，判非法。
/// 取甲（哪块盘自证过的槽少于两份就按真）时这一拒判成合法。
#[test]
fn a_witness_refusal_on_a_torn_system_configuration_rotation_is_a_refusal_for_the_wrong_reason() {
    let mut image = pool_after_the_first_file();
    let slot_of_the_first_file_rotation = offset_of_system_configuration_slot(
        GENERATION_OF_THE_FIRST_FILE_ROTATION % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE,
    );
    for device in DEVICES {
        image.flip_byte(
            device,
            slot_of_the_first_file_rotation,
            TORN_BYTE_INSIDE_THE_SYSTEM_CONFIGURATION_SLOT,
        );
    }
    assert_eq!(
        verdict_of_the_claimed_witness_refusal_on(&image),
        WritableMountRefusalVerdict::Illegal {
            reason: RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration,
            conditions_not_met: vec![
                RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage
            ],
        },
        "撕裂的轮换不是判不出：判据复算为假"
    );
}

/// 场景：盘 0 两个系统配置槽都撕裂（一份本池自证过的都没有），盘 1 两槽都自证过；设备不报错。
/// 预期：R2 判得出（没有槽报读错），但盘 0 不「可见」（C554 乙-配置续 Q1）：实现在读阶段判完之后的逐盘核里以「系统配置又读不出」拒，
/// 判法照那一拒的判据（判不出或某块盘一份自证过的都没有）复算为真，合法；同一份盘面上拿判据 N-配置 当理由拒，判据照它自己的判法复算
/// （c_见证 取盘 1 自证过的槽，不比第一个文件那条记录的计数器大）为假，是拒错了（C664（挂载拒绝合法复算把两种拒因报成一个）：两种拒因各按各的判据）。
/// 判别力：判法复算判据 N-配置 时仍把「某块盘一份自证过的都没有」按真（`crates/mutations.tsv` 一行）时第二条断言红；
/// 那一拒的判据不认「某块盘一份自证过的都没有」时第一条红。
#[test]
fn on_a_device_without_any_self_verified_system_configuration_slot_the_refusal_after_the_read_stage_is_legal_and_a_witness_refusal_is_not(
) {
    let mut image = pool_after_the_first_file();
    for slot_index in 0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE {
        image.flip_byte(
            DEVICES[0],
            offset_of_system_configuration_slot(slot_index),
            TORN_BYTE_INSIDE_THE_SYSTEM_CONFIGURATION_SLOT,
        );
    }
    let slot_spacing_in_bytes = u64::from(SMALL.parameters().geometry.fixed_structure_slot_spacing);
    let filesystem_identifier = SMALL.parameters().filesystem_identifier;
    assert_eq!(
        DEVICES.map(|device| {
            verified_system_configuration_slots(
                &image,
                device,
                slot_spacing_in_bytes,
                &filesystem_identifier,
            )
            .len()
        }),
        [0, 2],
        "前提：盘 0 一份自证过的系统配置都没有，盘 1 两槽都自证过"
    );
    assert_eq!(
        verdict_of_a_claimed_refusal_on(&image, refusal_after_the_read_stage_claimed_on),
        WritableMountRefusalVerdict::Legal(
            RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage
        ),
        "一块盘一份自证过的系统配置都没有：读阶段之后系统配置又读不出那一拒的判据为真"
    );
    assert_eq!(
        verdict_of_the_claimed_witness_refusal_on(&image),
        WritableMountRefusalVerdict::Illegal {
            reason: RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration,
            conditions_not_met: vec![
                RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage
            ],
        },
        "同一份盘面上判据 N-配置 复算为假：拿它当理由拒是拒错了"
    );
}
