#![allow(dead_code, unused)]
pub mod segments {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub enum StepKind { ZeroFill, Discard, UnitWrite, JournalRecord, RootRecordFua, SystemConfigurationSlot, Barrier }
}
pub mod memory_pool {
    use std::collections::BTreeMap;
    use crate::segments::StepKind;
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct DeviceIdentity(pub u32);
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct DeviceOffsetInBytes(pub u64);
    #[derive(Clone)]
    pub enum WrittenContents { Bytes(Vec<u8>), Zero(u64) }
    impl WrittenContents {
        pub fn length_in_bytes(&self) -> u64 { match self { WrittenContents::Bytes(bytes) => bytes.len() as u64, WrittenContents::Zero(length) => *length } }
        pub fn as_bytes(&self) -> Option<&[u8]> { match self { WrittenContents::Bytes(bytes) => Some(bytes), WrittenContents::Zero(_) => None } }
    }
    #[derive(Clone)]
    pub struct RetainedWrite { pub device: DeviceIdentity, pub kind: StepKind, pub is_force_unit_access: bool, pub offset: DeviceOffsetInBytes, pub contents: WrittenContents }
    impl RetainedWrite {
        pub fn length_in_bytes(&self) -> u64 { self.contents.length_in_bytes() }
        pub fn bytes(&self) -> Option<&[u8]> { self.contents.as_bytes() }
    }
    #[derive(Default)]
    pub struct SparseDevice { pub sectors: BTreeMap<u64, Vec<u8>> }
    impl SparseDevice { pub fn written_sectors(&self) -> &BTreeMap<u64, Vec<u8>> { &self.sectors } }
    pub struct MemoryPool { pub devices: BTreeMap<DeviceIdentity, SparseDevice> }
    pub struct PublishWrites { pub units: Vec<usize>, pub records: Vec<usize>, pub root: usize, pub instance: u32, pub checkpoint_txg: u64 }
    pub enum RecordStreamContinuity<'a> { OneRecording, BrokenAtSeams(&'a [u8]) }
    pub fn publishes_in(writes: &[RetainedWrite], persisted: &[bool], continuity: RecordStreamContinuity<'_>) -> Vec<PublishWrites> { Vec::new() }
    pub struct CrashImage<'base> { pub base: &'base MemoryPool, pub writes: &'base [RetainedWrite], pub persisted: Vec<bool> }
}
pub mod crash {
    use crate::memory_pool::*;
    pub struct PublishedVersion;
    pub struct CrashStateJudgement;
    impl CrashStateJudgement { pub fn red_items(&self) -> Vec<String> { Vec::new() } }
    pub fn judge_crash_image(image: &CrashImage<'_>, judged_root_index: usize, versions: &[PublishedVersion]) -> CrashStateJudgement { CrashStateJudgement }
    pub enum CrashPointReplaySegmentExpansion { EveryProperSubset, NotExpanded }
    pub struct CrashStateEnumerationTables { pub enumerated_writes: Vec<RetainedWrite>, pub state_ranges_by_segment: Vec<std::ops::Range<u64>>, pub state_count: u64 }
    pub fn crash_state_enumeration_tables(base: &MemoryPool, writes: &[RetainedWrite], segments: &[Vec<usize>], expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion) -> CrashStateEnumerationTables { todo!() }
    pub fn full_expansion(_segment_index: usize, _segment: &[usize]) -> CrashPointReplaySegmentExpansion { CrashPointReplaySegmentExpansion::EveryProperSubset }
}
pub mod crash_state_enumeration {
    pub fn crash_point_replay_worker_threads() -> (usize, &'static str) { (4, "stub") }
}
