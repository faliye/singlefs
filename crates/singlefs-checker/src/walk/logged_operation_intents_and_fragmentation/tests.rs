use super::*;

/// 一块 1 GiB、单元区里一个单元都没有的盘：I-3.16 那一判只读盘容量。
struct OneEmptyDeviceOfOneGibibyte;

impl ImageReader for OneEmptyDeviceOfOneGibibyte {
    fn devices(&self) -> Vec<u32> {
        vec![0]
    }
    fn device_bytes(&self, _device: u32) -> Option<u64> {
        Some(1 << 30)
    }
    fn read(&self, _device: u32, _offset: u64, _length: usize) -> Option<Vec<u8>> {
        None
    }
    fn candidate_unit_slots(&self, _device: u32) -> Option<Vec<u64>> {
        Some(Vec::new())
    }
    fn candidate_journal_slots(&self, _device: u32) -> Option<Vec<u64>> {
        Some(Vec::new())
    }
}

/// I-3.16：最新根的记账里没有 runs 那一行——没有别的不变量判它，判红，不报部分没判。
/// 已分配那一行与占着的槽对得上（都是 0），盘容量读得出，走到 runs 那一判。
#[test]
fn an_accounting_without_the_fragmentation_runs_row_reddens_the_fragmentation_statistics_invariant()
{
    let mut judgements = Judgements::default();
    judge_fragmentation_and_empty_segment_statistics(
        &OneEmptyDeviceOfOneGibibyte,
        UNIT_AREA_START_SLOT_OF_THE_DEFAULT_JOURNAL_RING,
        &BTreeMap::from([((STATISTIC_ALLOCATED_BYTES, 0), 0)]),
        &BTreeSet::new(),
        &Ok(AllocationRecordsOfAVersion {
            records: Vec::new(),
            nodes_of_the_tree_the_root_record_holds: Vec::new(),
            allocation_record_trees_read: 1,
        }),
        &mut judgements,
    );
    let verdict = verdict_of(
        judgements,
        FRAGMENTATION_AND_EMPTY_SEGMENT_STATISTICS_EQUAL_A_FULL_SCAN,
    );
    assert!(
        matches!(&verdict, InvariantVerdict::Violated(detail) if detail.contains("没有 runs 那一行")),
        "记账里缺 runs 那一行：I-3.16 应当判红，实际 {verdict:?}"
    );
}
