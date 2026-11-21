use super::*;

/// 一块盘：读者交的 journal 候选槽是 `candidate_slots`（从环起点数），`non_zero_slots` 里那几槽读出不全零、不自证的字节，别处读出全零。
struct JournalCandidatesReachingPastTheRing {
    candidate_slots: Vec<u64>,
    non_zero_slots: Vec<u64>,
    journal_ring_start_slot: u64,
}

impl ImageReader for JournalCandidatesReachingPastTheRing {
    fn devices(&self) -> Vec<u32> {
        vec![0]
    }
    fn device_bytes(&self, _device: u32) -> Option<u64> {
        Some(1 << 30)
    }
    fn read(&self, _device: u32, offset: u64, length: usize) -> Option<Vec<u8>> {
        let ring_start = self.journal_ring_start_slot * SLOT_BYTES;
        let slot_from_the_ring_start = offset
            .checked_sub(ring_start)
            .map(|offset_in_the_ring| offset_in_the_ring / JOURNAL_RECORD_BYTES);
        let byte =
            if slot_from_the_ring_start.is_some_and(|slot| self.non_zero_slots.contains(&slot)) {
                0xA5
            } else {
                0
            };
        Some(vec![byte; length])
    }
    fn candidate_unit_slots(&self, _device: u32) -> Option<Vec<u64>> {
        Some(Vec::new())
    }
    fn candidate_journal_slots(&self, _device: u32) -> Option<Vec<u64>> {
        Some(self.candidate_slots.clone())
    }
}

/// I-8.14 眼里读不出的 journal 环槽只数环里的槽（
/// 「journal 环里有读不出的槽」）。读者交的候选宁多不少：盘上系统配置槽 0 自证不过时 harness 的读者从环起点罩到盘尾，候选里有环外单元区里写过的槽，
/// 那几槽的字节不全零、不自证。它们不进读不出的那一份（GPU 判表的 journal 记录落点也只在环里，两边才对得上）；环里不全零而不自证的槽照记。
#[test]
fn journal_candidate_slots_past_the_ring_are_not_counted_as_unreadable_ring_slots() {
    let geometry = geometry_of_a_ring_of_twelve();
    let ring_slot_count = geometry.journal_ring_bytes / JOURNAL_RECORD_BYTES;
    let first_slot_past_the_ring = ring_slot_count;
    let scanned_with_garbage_only_past_the_ring = scanned_journal_ring_of_device(
        &JournalCandidatesReachingPastTheRing {
            candidate_slots: vec![0, 1, first_slot_past_the_ring, first_slot_past_the_ring + 3],
            non_zero_slots: vec![first_slot_past_the_ring, first_slot_past_the_ring + 3],
            journal_ring_start_slot: geometry.journal_ring_start_slot,
        },
        &geometry,
        0,
        0,
    );
    assert_eq!(
            scanned_with_garbage_only_past_the_ring.slots_that_do_not_read_back,
            Vec::<u64>::new(),
            "环外那两槽（{first_slot_past_the_ring} 与其后第三槽）不是 journal 环的槽，不当读不出的环槽"
        );
    let scanned_with_garbage_inside_the_ring = scanned_journal_ring_of_device(
        &JournalCandidatesReachingPastTheRing {
            candidate_slots: vec![0, 7, first_slot_past_the_ring],
            non_zero_slots: vec![7, first_slot_past_the_ring],
            journal_ring_start_slot: geometry.journal_ring_start_slot,
        },
        &geometry,
        0,
        0,
    );
    assert_eq!(
        scanned_with_garbage_inside_the_ring.slots_that_do_not_read_back,
        vec![7],
        "环里不全零而不自证的槽 7 照记成读不出，环外那一槽不记"
    );
}

/// 一块盘的环里只坐着计数器 `counter` 上的这一条记录。
fn ring_of_a_device_holding(counter: u64, record: ScannedJournalRecord) -> JournalRecordsOfADevice {
    JournalRecordsOfADevice::from([(
        JournalRecordKeyOnADevice {
            counter,
            instance: record.instance,
        },
        record,
    )])
}

/// I-8.6 的本实例内逻辑前一条按 (实例代号, 计数器 − 1) 找（`records/2026-10-24-singlefs-checker-审计复核.md` 的 C02）：一块盘上计数器 3
/// 坐着实例 1 与实例 2 各一条（冲突布局，按键的次序实例 1 那条在前），实例 2 计数器 4 那一条的链值取实例 2 那一条的；
/// 实例 3 计数器 4 那一条前一个计数器上只有别的实例的记录 ⇒ 本实例第一条、链值 0；计数器 6 前一个计数器上一条都没有 ⇒ 判不了。
#[test]
fn the_back_chain_takes_the_predecessor_of_its_own_instance_when_two_instances_share_the_previous_counter(
) {
    let mut records = ring_of_a_device_holding(3, scanned_record_of(1, 0, 0x11));
    records.insert(
        JournalRecordKeyOnADevice {
            counter: 3,
            instance: 2,
        },
        scanned_record_of(2, 0, 0x22),
    );
    assert_eq!(
            expected_back_chain_of(
                &records,
                JournalRecordKeyOnADevice {
                    counter: 4,
                    instance: 2
                }
            ),
            ExpectedBackChain::Value(0x22),
            "实例 2 计数器 4 的本实例内逻辑前一条是实例 2 计数器 3 那一条，不是按键次序先到的实例 1 那一条"
        );
    assert_eq!(
        expected_back_chain_of(
            &records,
            JournalRecordKeyOnADevice {
                counter: 4,
                instance: 3
            }
        ),
        ExpectedBackChain::Value(0),
        "计数器 3 上只有别的实例的记录：实例 3 计数器 4 是本实例第一条，链值 0"
    );
    assert_eq!(
        expected_back_chain_of(
            &records,
            JournalRecordKeyOnADevice {
                counter: 6,
                instance: 2
            }
        ),
        ExpectedBackChain::PredecessorNotOnThisDevice,
        "计数器 5 上一条记录都没有：判不了"
    );
}

/// 一条恢复的读者收得下的记录视图（提交标记 1、只带末条标志、序号 1、新根段全 0），给 [`journal_record_is_read_by_the_recovery_reader`] 用。
fn journal_record_view_the_recovery_reader_reads() -> crate::JournalRecordView {
    crate::JournalRecordView {
        instance: 1,
        counter: 1,
        checkpoint_txg: 1,
        transaction: 1,
        is_commit: true,
        commit_marker_byte: 1,
        ordinal_within_publish: 1,
        record_flags_byte: RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH,
        back_chain: 0,
        new_root_segment: vec![0u8; NEW_ROOT_SEGMENT_PINNED_RING_SLOT.end],
        new_tree_identifier_watermark: 0,
        new_rollback_floor: 0,
        named: Vec::new(),
    }
}

/// C897 那一行后半：恢复的记录读者（core 的 `JournalRecord::parse`）另拒的四种，池级 checker 照另写的一份逐种认得出——提交标记字节 2、
/// 记录标志位 1 置位、序号 0、新根段里实例表指针的 MAC 第 0 字节非 0；一样都不沾的那条收。
#[test]
fn the_recovery_reader_refuses_each_of_the_four_kinds_the_pool_checker_reader_accepts() {
    assert!(
        journal_record_is_read_by_the_recovery_reader(
            &journal_record_view_the_recovery_reader_reads()
        ),
        "提交标记 1、只带末条标志、序号 1、新根段指针头部全 0：恢复的读者收"
    );
    let mut commit_marker_two = journal_record_view_the_recovery_reader_reads();
    commit_marker_two.commit_marker_byte = 2;
    assert!(
        !journal_record_is_read_by_the_recovery_reader(&commit_marker_two),
        "提交标记字节 2：不收"
    );
    let mut record_flag_bit_one = journal_record_view_the_recovery_reader_reads();
    record_flag_bit_one.record_flags_byte |= 0b0000_0010;
    assert!(
        !journal_record_is_read_by_the_recovery_reader(&record_flag_bit_one),
        "记录标志位 1 置位：不收"
    );
    let mut ordinal_zero = journal_record_view_the_recovery_reader_reads();
    ordinal_zero.ordinal_within_publish = 0;
    assert!(
        !journal_record_is_read_by_the_recovery_reader(&ordinal_zero),
        "本次发布内序号 0：不收"
    );
    let mut instance_table_pointer_with_a_mac_byte =
        journal_record_view_the_recovery_reader_reads();
    instance_table_pointer_with_a_mac_byte.new_root_segment
        [NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER.start] = 1;
    assert!(
        !journal_record_is_read_by_the_recovery_reader(&instance_table_pointer_with_a_mac_byte),
        "新根段里实例表指针的 MAC 第 0 字节非 0：不收"
    );
}

/// 两块盘同一计数器坐着不同实例的记录（盘 0 计数器 4 是实例 2 的，盘 1 计数器 4 是实例 1 的）：恢复眼里的环按 (实例代号, 计数器)
/// 建键，两条都留下，不拿先读到的盘 0 那条挤掉实例 1 的（I-4.1 那一行：重放读的环先改、这一处同改，C897 起两处是同一个函数）。
#[test]
fn the_ring_as_the_recovery_reads_it_keeps_both_records_of_two_instances_at_the_same_counter() {
    let records_by_device = vec![
        (
            0,
            ring_of_a_device_holding(4, scanned_record_of(2, 0, 0x22)),
        ),
        (
            1,
            ring_of_a_device_holding(4, scanned_record_of(1, 0x0b, 0x11)),
        ),
    ];
    let ring = records_as_the_recovery_reads_them(&records_by_device);
    assert_eq!(
        ring.keys().copied().collect::<Vec<_>>(),
        vec![(1, 4), (2, 4)],
        "两条都在，按 (实例代号, 计数器) 认"
    );
    assert_eq!(
        ring[&(1, 4)].chain_value_the_next_record_must_carry,
        0x11,
        "实例 1 那一条是盘 1 上那一份"
    );
}

/// 反向链那一关按 (实例代号, 计数器 − 1) 找本实例内逻辑前一条：计数器 3 上先读到的是实例 2 的记录、实例 1 的那一条在盘 1 上，
/// 实例 1 计数器 4 那一条的反向链与实例 1 计数器 3 那条的链值不等 ⇒ 进不了前缀；对得上 ⇒ 进得去。
/// 按计数器找时先读到的是实例 2 的那条，当成「前一条是别的实例」放过去。
#[test]
fn the_back_chain_gate_looks_up_the_predecessor_of_the_same_instance_past_another_instance_at_that_counter(
) {
    let records_by_device = vec![
        (
            0,
            ring_of_a_device_holding(3, scanned_record_of(2, 0, 0x22)),
        ),
        (
            1,
            ring_of_a_device_holding(3, scanned_record_of(1, 0, 0x11)),
        ),
    ];
    let ring = records_as_the_recovery_reads_them(&records_by_device);
    assert!(
        !record_can_enter_a_replay_prefix(&ring, 4, &scanned_record_of(1, 0x12, 0)),
        "实例 1 计数器 4 的反向链 0x12 与实例 1 计数器 3 的链值 0x11 不等：不进前缀"
    );
    assert!(
        record_can_enter_a_replay_prefix(&ring, 4, &scanned_record_of(1, 0x11, 0)),
        "对得上：进前缀"
    );
}
