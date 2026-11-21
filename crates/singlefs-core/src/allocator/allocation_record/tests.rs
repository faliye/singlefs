use super::*;

/// 一条分配记录的 20 字节，跨度段换成 `span_field`（别的段照一条盘 1、槽 50180、分配代 3 的记录）。
fn allocation_record_bytes_with_the_span_field(span_field: u16) -> Vec<u8> {
    let mut bytes = AllocationRecord {
        device: DeviceIdentity(1),
        slot: SlotNumber(50180),
        span_slots: 2,
        generation: CheckpointTxg(3),
        is_released: false,
        quarantine_mark: AllocationRecordQuarantineMark::NotQuarantined,
    }
    .to_bytes();
    bytes[10..12].copy_from_slice(&span_field.to_le_bytes());
    bytes
}

/// 跨度段次高位 0x4000 是隔离记号（D19 已定项 5「隔离的盘上记号」）：0x4001、0x4002 读成跨度 1、2 且带记号，
/// 写回去逐字节相同；0x4000 与 0x8000 同时置（0xC001）拒。
#[test]
fn span_field_with_the_quarantine_mark_parses_to_the_span_in_the_low_bits_and_the_mark() {
    for span_slots in [1u16, 2] {
        let bytes = allocation_record_bytes_with_the_span_field(
            ALLOCATION_RECORD_QUARANTINE_MARK | span_slots,
        );
        let parsed = AllocationRecord::parse(&bytes).expect("带隔离记号、未释放的记录解得开");
        assert_eq!(parsed.span_slots, span_slots, "跨度取去掉记号之后的低位");
        assert_eq!(
            parsed.quarantine_mark,
            AllocationRecordQuarantineMark::QuarantinedAfterTheReleaseChecksumCheck,
            "0x4000 读成隔离记号"
        );
        assert!(!parsed.is_released, "带记号的记录仍分配");
        assert_eq!(parsed.generation, CheckpointTxg(3), "分配代照读");
        assert_eq!(parsed.to_bytes(), bytes, "记号写回跨度段次高位");
    }
    assert_eq!(
        AllocationRecord::parse(&allocation_record_bytes_with_the_span_field(
            ALLOCATION_RECORD_RELEASED_FLAG | ALLOCATION_RECORD_QUARANTINE_MARK | 1
        )),
        Err(AllocationRecordEntryRefusal::QuarantineMarkOnAReleasedRecord),
        "0xC001：隔离记号与已释放标志同时置，判坏"
    );
}
