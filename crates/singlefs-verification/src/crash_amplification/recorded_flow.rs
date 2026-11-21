//! 录下的流与它的几样附件在库里的字节：流本身（[`RecordedFlow`]）、给定状态表、一趟的清单（[`RecordedRunManifest`]），
//! 录进库与读回（[`record_the_flow_into_the_library`]、[`read_back_the_recorded_run`]）。

use super::*;

// ============================== 录入段落库的流程（两段：录入段在 CPU 上跑数据、写进库；判段只从库里读） ==============================

/// 录入段写进库的一条流程：判段重建崩溃状态、排计划、判、核对要的全部输入，自己持有，不借录入段内存里的流程
/// （录入与判是两段，段与段之间不在内存里交数据）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordedFlow {
    pub base: MemoryPool,
    pub writes: Vec<RetainedWrite>,
    pub segments: Vec<Vec<usize>>,
    pub judged_root_index: usize,
    pub versions: Vec<PublishedVersion>,
    pub versions_without_a_file: Vec<PublishedVersionWithoutAFile>,
    /// 每一段照录入段那一趟的展开方式（按段登记改过的也在内），段号即下标。
    pub segment_expansions: Vec<CrashPointReplaySegmentExpansion>,
    pub crash_points: Vec<CrashPointSpan>,
    /// 每一段这一趟的片（录入段定的片起点与片长，C790；不分片的段是 `None`），段号即下标：判段照它排计划、判、核对，不另算。
    pub segment_slices: Vec<Option<CrashPointReplaySegmentSlice>>,
}

/// 录入段那份字节的头：换了编码就换头，读到别的头拒绝，不猜（`SFRFLOW2`：每段多一格片，C790）。
const RECORDED_FLOW_MAGIC: &[u8; 8] = b"SFRFLOW2";

impl RecordedFlow {
    /// 录入段：把这一趟要判的流程（按段登记改过展开的那一份）抄成自己持有的一份；一段都不分片。
    #[must_use]
    pub fn of(flow: &CrashFlow<'_>) -> Self {
        Self::of_with_slices(flow, &no_segment_slices(flow.segments.len()))
    }

    /// 同 [`Self::of`]，带上每段这一趟的片（[`CrashPlan::segment_slices`]）。
    #[must_use]
    pub fn of_with_slices(
        flow: &CrashFlow<'_>,
        segment_slices: &[Option<CrashPointReplaySegmentSlice>],
    ) -> Self {
        Self {
            base: flow.base.clone(),
            writes: flow.writes.to_vec(),
            segments: flow.segments.to_vec(),
            judged_root_index: flow.judged_root_index,
            versions: flow.versions.to_vec(),
            versions_without_a_file: flow.versions_without_a_file.to_vec(),
            segment_expansions: flow
                .segments
                .iter()
                .enumerate()
                .map(|(segment_index, segment)| (flow.expansion)(segment_index, segment))
                .collect(),
            crash_points: flow.crash_points.clone(),
            segment_slices: segment_slices.to_vec(),
        }
    }

    /// 判段从库里读回的这一份当成一条流程用：展开方式照录入段登下的逐段取。
    pub fn with_the_flow<Outcome>(
        &self,
        use_the_flow: impl FnOnce(&CrashFlow<'_>) -> Outcome,
    ) -> Outcome {
        let expansion =
            |segment_index: usize, _segment: &[usize]| self.segment_expansions[segment_index];
        let flow = CrashFlow {
            base: &self.base,
            writes: &self.writes,
            segments: &self.segments,
            judged_root_index: self.judged_root_index,
            versions: &self.versions,
            versions_without_a_file: &self.versions_without_a_file,
            expansion: &expansion,
            crash_points: self.crash_points.clone(),
        };
        use_the_flow(&flow)
    }

    /// 写进库的字节（小端，变长的先写长度）。
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = RECORDED_FLOW_MAGIC.to_vec();
        let put_u64 = |destination_bytes: &mut Vec<u8>, value: u64| {
            destination_bytes.extend_from_slice(&value.to_le_bytes())
        };
        let put_count = |destination_bytes: &mut Vec<u8>, count: usize| {
            destination_bytes
                .extend_from_slice(&u64::try_from(count).expect("长度装得进 u64").to_le_bytes());
        };
        let put_bytes = |destination_bytes: &mut Vec<u8>, content: &[u8]| {
            destination_bytes.extend_from_slice(
                &u64::try_from(content.len())
                    .expect("长度装得进 u64")
                    .to_le_bytes(),
            );
            destination_bytes.extend_from_slice(content);
        };
        put_pool_writes_and_segments(&mut bytes, &self.base, &self.writes, &self.segments);
        put_count(&mut bytes, self.judged_root_index);
        put_count(&mut bytes, self.versions.len());
        for version in &self.versions {
            bytes.extend_from_slice(&version.instance.0.to_le_bytes());
            put_u64(&mut bytes, version.checkpoint_txg.0);
            put_bytes(&mut bytes, &version.content);
        }
        put_count(&mut bytes, self.versions_without_a_file.len());
        for version in &self.versions_without_a_file {
            bytes.extend_from_slice(&version.instance.0.to_le_bytes());
            put_u64(&mut bytes, version.checkpoint_txg.0);
        }
        put_count(&mut bytes, self.segment_expansions.len());
        for expansion in &self.segment_expansions {
            bytes.push(segment_expansion_code(*expansion));
        }
        put_count(&mut bytes, self.crash_points.len());
        for point in &self.crash_points {
            put_bytes(&mut bytes, point.name.as_bytes());
            put_bytes(&mut bytes, point.code_file_in_repository.as_bytes());
            put_count(&mut bytes, point.writes.start);
            put_count(&mut bytes, point.writes.end);
            bytes.push(u8::from(point.ignored));
        }
        put_count(&mut bytes, self.segment_slices.len());
        for slice in &self.segment_slices {
            match slice {
                None => bytes.push(0),
                Some(slice) => {
                    bytes.push(1);
                    bytes.extend_from_slice(&slice.start_within_the_domain.to_le_bytes());
                    put_u64(&mut bytes, slice.state_count);
                }
            }
        }
        bytes
    }

    /// 从库里的字节读回；头不对、字节不够、多出字节、码认不得都交回 `None`（判段照「库里那份读不回」停下，不猜）。
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut reader = RecordedBytesReader { bytes, at: 0 };
        if reader.take(RECORDED_FLOW_MAGIC.len())? != RECORDED_FLOW_MAGIC {
            return None;
        }
        let (base, writes, segments) = read_pool_writes_and_segments(&mut reader)?;
        let judged_root_index = reader.count()?;
        let mut versions = Vec::new();
        for _ in 0..reader.count()? {
            versions.push(PublishedVersion {
                instance: InstanceGeneration(reader.u32()?),
                checkpoint_txg: CheckpointTxg(reader.u64()?),
                content: reader.bytes()?.to_vec(),
            });
        }
        let mut versions_without_a_file = Vec::new();
        for _ in 0..reader.count()? {
            versions_without_a_file.push(PublishedVersionWithoutAFile {
                instance: InstanceGeneration(reader.u32()?),
                checkpoint_txg: CheckpointTxg(reader.u64()?),
            });
        }
        let mut segment_expansions = Vec::new();
        for _ in 0..reader.count()? {
            segment_expansions.push(segment_expansion_of_code(reader.u8()?)?);
        }
        let mut crash_points = Vec::new();
        for _ in 0..reader.count()? {
            let name = String::from_utf8(reader.bytes()?.to_vec()).ok()?;
            let code_file_in_repository = String::from_utf8(reader.bytes()?.to_vec()).ok()?;
            let start = reader.count()?;
            let end = reader.count()?;
            let ignored = match reader.u8()? {
                0 => false,
                1 => true,
                _ => return None,
            };
            crash_points.push(CrashPointSpan {
                name,
                code_file_in_repository,
                writes: start..end,
                ignored,
            });
        }
        let mut segment_slices = Vec::new();
        for _ in 0..reader.count()? {
            segment_slices.push(match reader.u8()? {
                0 => None,
                1 => Some(CrashPointReplaySegmentSlice {
                    start_within_the_domain: StateCount1280Bits::from_le_bytes(
                        reader.take(STATE_COUNT_BYTES)?.try_into().ok()?,
                    ),
                    state_count: reader.u64()?,
                }),
                _ => return None,
            });
        }
        if reader.at != bytes.len()
            || segment_expansions.len() != segments.len()
            || segment_slices.len() != segments.len()
        {
            return None;
        }
        Some(Self {
            base,
            writes,
            segments,
            judged_root_index,
            versions,
            versions_without_a_file,
            segment_expansions,
            crash_points,
            segment_slices,
        })
    }

    /// 这一份的摘要（库里按它存、按它读回核）。
    #[must_use]
    pub fn digest(&self) -> [u8; DIGEST_BYTES] {
        sha256_digest(&self.to_bytes())
    }
}

/// 读回的流逐段的展开方式（录入段登下的那一份），跨线程共用。
type ExpansionOfEachSegment =
    Box<dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion + Sync + Send>;

/// 从判定库里读回、核过摘要的一份流：判法（崩溃点重放枚举与按序号段判、判块、GPU 判、第 ② 段核对）只收它借出的 [`FlowOfTheLibrary`]。
/// 字段私有，只有 [`RecordedFlowReadBack::read_from_the_library`] 与 [`RecordedFlowReadBack::read_outside_the_library_lock`] 造得出：内存里现备的流交不进判法，要判先写进库再读回
/// （测试用 [`record_the_flow_and_read_it_back`]）。
pub struct RecordedFlowReadBack {
    recorded: RecordedFlow,
    expansion_of_each_segment: ExpansionOfEachSegment,
}

impl std::fmt::Debug for RecordedFlowReadBack {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RecordedFlowReadBack")
            .field("recorded", &self.recorded)
            .finish_non_exhaustive()
    }
}

impl RecordedFlowReadBack {
    /// 按摘要从库里读录下的流：读字节、核摘要、解码。
    ///
    /// # Errors
    /// 库读失败；库里没有这个摘要；字节的摘要对不上（库里那份被改过）；解不开。
    pub fn read_from_the_library(
        store: &VerdictStore,
        digest: &[u8; DIGEST_BYTES],
    ) -> Result<Self, String> {
        Self::of_the_bytes_read_from_the_library(
            &recorded_flow_bytes_in_the_library(store, digest)?,
            digest,
        )
    }

    /// 同 [`Self::read_from_the_library`]，锁只包读字节那一次往返：核摘要与解码在锁外（一条流载荷可到几 MiB，别的线程不等它解）。
    ///
    /// # Errors
    /// 同 [`Self::read_from_the_library`]。
    pub(crate) fn read_outside_the_library_lock(
        library: &crate::crash_library::CrashLibrary,
        digest: &[u8; DIGEST_BYTES],
    ) -> Result<Self, String> {
        crate::crash_library::assert_this_thread_holds_no_library_lock("读回流之后的核摘要与解码");
        let bytes = recorded_flow_bytes_in_the_library(&library.lock(), digest)?;
        crate::crash_library::assert_this_thread_holds_no_library_lock("核流的摘要、解流");
        Self::of_the_bytes_read_from_the_library(&bytes, digest)
    }

    /// 从库里读到的字节核摘要、解码。
    fn of_the_bytes_read_from_the_library(
        flow_bytes: &[u8],
        digest: &[u8; DIGEST_BYTES],
    ) -> Result<Self, String> {
        if sha256_digest(flow_bytes) != *digest {
            return Err("录下的流程摘要与清单对不上：库里那份坏了，重跑录入段".to_string());
        }
        let recorded = RecordedFlow::from_bytes(flow_bytes)
            .ok_or_else(|| "录下的流程解不出（字节坏了或编码换了）：重跑录入段".to_string())?;
        let expansions = recorded.segment_expansions.clone();
        Ok(Self {
            recorded,
            expansion_of_each_segment: Box::new(move |segment_index, _segment| {
                expansions[segment_index]
            }),
        })
    }

    /// 读回的那一份（只读）。
    #[must_use]
    pub fn recorded(&self) -> &RecordedFlow {
        &self.recorded
    }

    /// 借出判法收的那一种流：展开方式照录入段登下的逐段取。
    #[must_use]
    pub fn flow(&self) -> FlowOfTheLibrary<'_> {
        FlowOfTheLibrary {
            flow: CrashFlow {
                base: &self.recorded.base,
                writes: &self.recorded.writes,
                segments: &self.recorded.segments,
                judged_root_index: self.recorded.judged_root_index,
                versions: &self.recorded.versions,
                versions_without_a_file: &self.recorded.versions_without_a_file,
                expansion: &*self.expansion_of_each_segment,
                crash_points: self.recorded.crash_points.clone(),
            },
        }
    }
}

/// 库里录下的流的字节（还没核摘要、没解码）。
fn recorded_flow_bytes_in_the_library(
    store: &VerdictStore,
    digest: &[u8; DIGEST_BYTES],
) -> Result<Vec<u8>, String> {
    store
        .read_recorded_flow(digest)
        .map_err(|error| format!("读录下的流程失败：{error:?}"))?
        .ok_or_else(|| "录下的流程不在库里：先跑录入段".to_string())
}

/// 从库里读回的流连它的计划：取样照一条流按序号段调几十次，这两样只由流定，按摘要留一份（[`crate::crash_library::CrashLibrary::recorded_flow_and_its_plan_read_back_once`]），
/// 各次、各线程共用。计划照读回的那一份排，不用录入段内存里的流。
pub(crate) struct RecordedFlowAndItsPlan {
    read_back: RecordedFlowReadBack,
    plan: CrashPlan,
}

impl RecordedFlowAndItsPlan {
    /// 从库里读回这个摘要的流、在锁外排计划。
    ///
    /// # Errors
    /// 同 [`RecordedFlowReadBack::read_outside_the_library_lock`]。
    pub(crate) fn read_outside_the_library_lock(
        library: &crate::crash_library::CrashLibrary,
        digest: &[u8; DIGEST_BYTES],
    ) -> Result<Self, String> {
        let read_back = RecordedFlowReadBack::read_outside_the_library_lock(library, digest)?;
        crate::crash_library::assert_this_thread_holds_no_library_lock("判段排计划");
        let plan = plan_crash_points(&read_back.flow());
        Ok(Self { read_back, plan })
    }

    pub(crate) fn flow_read_back(&self) -> FlowOfTheLibrary<'_> {
        self.read_back.flow()
    }

    pub(crate) fn plan_of_the_flow_read_back(&self) -> &CrashPlan {
        &self.plan
    }
}

/// 判法收的流：只能从 [`RecordedFlowReadBack::flow`] 借出（字段私有），所以判的一定是库里读回的那一份。读它照读 [`CrashFlow`]。
pub struct FlowOfTheLibrary<'flow> {
    pub(super) flow: CrashFlow<'flow>,
}

impl<'flow> std::ops::Deref for FlowOfTheLibrary<'flow> {
    type Target = CrashFlow<'flow>;

    fn deref(&self) -> &Self::Target {
        &self.flow
    }
}

/// 测试与单进程跑法用：把内存里备好的流抄成 [`RecordedFlow`] 按摘要写进 `store`，再从 `store` 读回。判法只拿读回的那一份。
///
/// # Errors
/// 写不进库；读回失败（[`RecordedFlowReadBack::read_from_the_library`]）。
pub fn record_the_flow_and_read_it_back(
    store: &mut VerdictStore,
    flow: &CrashFlow<'_>,
) -> Result<RecordedFlowReadBack, String> {
    let recorded = RecordedFlow::of(flow);
    let digest = recorded.digest();
    store
        .store_recorded_flow(&digest, &recorded.to_bytes())
        .map_err(|error| format!("录下的流程写不进库：{error:?}"))?;
    drop(recorded);
    RecordedFlowReadBack::read_from_the_library(store, &digest)
}

/// 录入段那几份字节里共用的一截：基础镜像（盘大小、每块盘写过的扇区）、写表（盘、种类、FUA、偏移、内容或整段清零）、切段。
pub(super) fn put_pool_writes_and_segments(
    bytes: &mut Vec<u8>,
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
) {
    let put_u64 = |destination_bytes: &mut Vec<u8>, value: u64| {
        destination_bytes.extend_from_slice(&value.to_le_bytes())
    };
    let put_count = |destination_bytes: &mut Vec<u8>, count: usize| {
        destination_bytes
            .extend_from_slice(&u64::try_from(count).expect("长度装得进 u64").to_le_bytes());
    };
    let put_bytes = |destination_bytes: &mut Vec<u8>, content: &[u8]| {
        destination_bytes.extend_from_slice(
            &u64::try_from(content.len())
                .expect("长度装得进 u64")
                .to_le_bytes(),
        );
        destination_bytes.extend_from_slice(content);
    };
    put_u64(bytes, base.device_size_in_bytes);
    put_count(bytes, base.devices.len());
    for (identity, device) in &base.devices {
        bytes.extend_from_slice(&identity.0.to_le_bytes());
        let sectors = device.written_sectors();
        put_count(bytes, sectors.len());
        for (sector, content) in sectors {
            put_u64(bytes, *sector);
            put_bytes(bytes, content);
        }
    }
    put_count(bytes, writes.len());
    for write in writes {
        bytes.extend_from_slice(&write.device.0.to_le_bytes());
        bytes.push(recorded_step_kind_byte(write.kind));
        bytes.push(u8::from(write.is_force_unit_access));
        put_u64(bytes, write.offset.0);
        match &write.contents {
            WrittenContents::Bytes(content) => {
                bytes.push(0);
                put_bytes(bytes, content);
            }
            WrittenContents::Zeros { length } => {
                bytes.push(1);
                put_u64(bytes, *length);
            }
        }
    }
    put_count(bytes, segments.len());
    for segment in segments {
        put_count(bytes, segment.len());
        for write_index in segment {
            put_count(bytes, *write_index);
        }
    }
}

/// [`put_pool_writes_and_segments`] 写下的那一截读回来；字节不够、码认不得交回 `None`。
pub(super) fn read_pool_writes_and_segments(
    reader: &mut RecordedBytesReader<'_>,
) -> Option<(MemoryPool, Vec<RetainedWrite>, Vec<Vec<usize>>)> {
    let device_size_in_bytes = reader.u64()?;
    let mut devices = BTreeMap::new();
    for _ in 0..reader.count()? {
        let identity = singlefs_core::address::DeviceIdentity(reader.u32()?);
        let mut device = crate::memory_pool::SparseDevice::default();
        for _ in 0..reader.count()? {
            let sector = reader.u64()?;
            let content = reader.bytes()?;
            device.write(
                singlefs_core::address::DeviceOffsetInBytes(
                    sector.checked_mul(crate::memory_pool::SECTOR_BYTES)?,
                ),
                content,
            );
        }
        devices.insert(identity, device);
    }
    let mut writes = Vec::new();
    for _ in 0..reader.count()? {
        let device = singlefs_core::address::DeviceIdentity(reader.u32()?);
        let kind = step_kind_of_code(reader.u8()?)?;
        let is_force_unit_access = match reader.u8()? {
            0 => false,
            1 => true,
            _ => return None,
        };
        let offset = singlefs_core::address::DeviceOffsetInBytes(reader.u64()?);
        let contents = match reader.u8()? {
            0 => WrittenContents::Bytes(reader.bytes()?.to_vec()),
            1 => WrittenContents::Zeros {
                length: reader.u64()?,
            },
            _ => return None,
        };
        writes.push(RetainedWrite {
            device,
            kind,
            is_force_unit_access,
            offset,
            contents,
        });
    }
    let mut segments = Vec::new();
    for _ in 0..reader.count()? {
        let mut segment = Vec::new();
        for _ in 0..reader.count()? {
            segment.push(reader.count()?);
        }
        segments.push(segment);
    }
    Some((
        MemoryPool {
            devices,
            device_size_in_bytes,
        },
        writes,
        segments,
    ))
}

/// 给定状态那一路写进库的一张表（崩溃注入、坏盘输入交来的状态表）：引擎先写进库、再从库里读回来判，判的是读回的这一份，
/// 不判调用方内存里的那一份（段与段之间不在内存里交数据）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordedGivenStatesTable {
    pub table_name: String,
    pub base: MemoryPool,
    pub writes: Vec<RetainedWrite>,
    pub segments: Vec<Vec<usize>>,
    pub seams: Vec<crate::memory_pool::RecordStreamSeam>,
    pub persisted_sets: Vec<Vec<bool>>,
}

const RECORDED_GIVEN_STATES_TABLE_MAGIC: &[u8; 8] = b"SFGSTAB1";

impl RecordedGivenStatesTable {
    #[must_use]
    pub fn of(table: &crate::crash_given_states::GivenStatesOfATable<'_>) -> Self {
        Self {
            table_name: table.table_name.clone(),
            base: table.base.clone(),
            writes: table.writes.to_vec(),
            segments: table.segments.to_vec(),
            seams: table.seams.to_vec(),
            persisted_sets: table.persisted_sets.clone(),
        }
    }

    /// 读回的这一份当成一张给定状态表用。
    pub fn with_the_table<Outcome>(
        &self,
        use_the_table: impl FnOnce(&crate::crash_given_states::GivenStatesOfATable<'_>) -> Outcome,
    ) -> Outcome {
        use_the_table(&crate::crash_given_states::GivenStatesOfATable {
            table_name: self.table_name.clone(),
            base: &self.base,
            writes: &self.writes,
            segments: &self.segments,
            seams: &self.seams,
            persisted_sets: self.persisted_sets.clone(),
        })
    }

    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        use crate::memory_pool::RecordStreamSeam;
        let mut bytes = RECORDED_GIVEN_STATES_TABLE_MAGIC.to_vec();
        let put_u64 = |destination_bytes: &mut Vec<u8>, value: u64| {
            destination_bytes.extend_from_slice(&value.to_le_bytes())
        };
        let put_count = |destination_bytes: &mut Vec<u8>, count: usize| {
            destination_bytes
                .extend_from_slice(&u64::try_from(count).expect("长度装得进 u64").to_le_bytes());
        };
        put_count(&mut bytes, self.table_name.len());
        bytes.extend_from_slice(self.table_name.as_bytes());
        put_pool_writes_and_segments(&mut bytes, &self.base, &self.writes, &self.segments);
        put_count(&mut bytes, self.seams.len());
        for seam in &self.seams {
            match seam {
                RecordStreamSeam::AfterAPowerLoss {
                    first_write_after_the_power_loss,
                } => {
                    bytes.push(0);
                    put_count(&mut bytes, *first_write_after_the_power_loss);
                }
                RecordStreamSeam::AfterACrash {
                    first_write_after_the_crash,
                    version_landed_on_after_the_crash,
                } => {
                    bytes.push(1);
                    put_count(&mut bytes, *first_write_after_the_crash);
                    match version_landed_on_after_the_crash {
                        None => bytes.push(0),
                        Some((instance, checkpoint_txg)) => {
                            bytes.push(1);
                            bytes.extend_from_slice(&instance.0.to_le_bytes());
                            put_u64(&mut bytes, checkpoint_txg.0);
                        }
                    }
                }
            }
        }
        put_count(&mut bytes, self.persisted_sets.len());
        for persisted in &self.persisted_sets {
            put_count(&mut bytes, persisted.len());
            bytes.extend(persisted.iter().map(|is_persisted| u8::from(*is_persisted)));
        }
        bytes
    }

    /// 头不对、字节不够或多出、码认不得交回 `None`。
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        use crate::memory_pool::RecordStreamSeam;
        let mut reader = RecordedBytesReader { bytes, at: 0 };
        if reader.take(RECORDED_GIVEN_STATES_TABLE_MAGIC.len())?
            != RECORDED_GIVEN_STATES_TABLE_MAGIC
        {
            return None;
        }
        let table_name = String::from_utf8(reader.bytes()?.to_vec()).ok()?;
        let (base, writes, segments) = read_pool_writes_and_segments(&mut reader)?;
        let mut seams = Vec::new();
        for _ in 0..reader.count()? {
            seams.push(match reader.u8()? {
                0 => RecordStreamSeam::AfterAPowerLoss {
                    first_write_after_the_power_loss: reader.count()?,
                },
                1 => RecordStreamSeam::AfterACrash {
                    first_write_after_the_crash: reader.count()?,
                    version_landed_on_after_the_crash: match reader.u8()? {
                        0 => None,
                        1 => Some((
                            InstanceGeneration(reader.u32()?),
                            CheckpointTxg(reader.u64()?),
                        )),
                        _ => return None,
                    },
                },
                _ => return None,
            });
        }
        let mut persisted_sets = Vec::new();
        for _ in 0..reader.count()? {
            let length = reader.count()?;
            let mut persisted = Vec::with_capacity(length);
            for byte in reader.take(length)? {
                persisted.push(match byte {
                    0 => false,
                    1 => true,
                    _ => return None,
                });
            }
            persisted_sets.push(persisted);
        }
        if reader.at != bytes.len() {
            return None;
        }
        Some(Self {
            table_name,
            base,
            writes,
            segments,
            seams,
            persisted_sets,
        })
    }
}

/// 给定状态表那一类的库里录下的一张表连同它的两栏（引擎录入段写进库的就是这一份，判段读回之后按两栏判）：
/// - 容器容量旋钮值（「给定状态表加旋钮栏」）：CPU 判器调池级 checker 带旋钮值的入口，GPU 判器把旋钮值编进扫描内核。
///   旋钮不落盘，所以不在表的镜像里，跟着表另记一栏；没填的表是 [`ContainerCapacityKnobs::PRODUCT`]。
/// - 每个状态恢复自称的那一版（「照字面改：表里记下根的来路」，[`ClaimedVersionOfAGivenState`]）：记录核对器第二条判据
///   照它取那一版（CPU 判器、GPU 恢复内核都照它）。逐状态一个，与表的持久集合同序同长；没填的表每个都是
///   [`ClaimedVersionOfAGivenState::LandedOnByTheConsultedRecovery`]。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordedGivenStatesTableWithItsColumns {
    pub table: RecordedGivenStatesTable,
    pub container_capacity_knobs: ContainerCapacityKnobs,
    pub claimed_versions: Vec<ClaimedVersionOfAGivenState>,
}

/// 带两栏那一份字节的头（之后是三棵容器树各一个 8 字节小端的每片记录数，0 是没装旋钮；再是恢复自称的那一版的个数与逐个
/// （[`put_a_claimed_version`]）；再之后是 [`RecordedGivenStatesTable::to_bytes`]）。换了布局就换头：只带旋钮栏的那一种（`SFGSKNB1`）不认。
const RECORDED_GIVEN_STATES_TABLE_WITH_ITS_COLUMNS_MAGIC: &[u8; 8] = b"SFGSCOL1";

/// 恢复自称的那一版在录进库的字节里的码：看 journal 那一遍落到的那一版、写了而那一版没有根、写了一条根（后接实例代号 4 字节与 checkpoint_txg 8 字节，小端）。
const CLAIMED_VERSION_CODE_LANDED_ON_BY_THE_CONSULTED_RECOVERY: u8 = 0;
const CLAIMED_VERSION_CODE_NEWEST_ROOT_WRITTEN_WITHOUT_A_ROOT: u8 = 1;
const CLAIMED_VERSION_CODE_NEWEST_ROOT_WRITTEN_WITH_A_ROOT: u8 = 2;

/// 一个状态那一栏恢复自称的那一版写进录进库的字节（码见 [`CLAIMED_VERSION_CODE_LANDED_ON_BY_THE_CONSULTED_RECOVERY`] 一族）。
fn put_a_claimed_version(bytes: &mut Vec<u8>, claimed_version: ClaimedVersionOfAGivenState) {
    match claimed_version {
        ClaimedVersionOfAGivenState::LandedOnByTheConsultedRecovery => {
            bytes.push(CLAIMED_VERSION_CODE_LANDED_ON_BY_THE_CONSULTED_RECOVERY);
        }
        ClaimedVersionOfAGivenState::NewestRootWrittenByTheMountAndItsPublish(None) => {
            bytes.push(CLAIMED_VERSION_CODE_NEWEST_ROOT_WRITTEN_WITHOUT_A_ROOT);
        }
        ClaimedVersionOfAGivenState::NewestRootWrittenByTheMountAndItsPublish(Some((
            instance,
            checkpoint_txg,
        ))) => {
            bytes.push(CLAIMED_VERSION_CODE_NEWEST_ROOT_WRITTEN_WITH_A_ROOT);
            bytes.extend_from_slice(&instance.0.to_le_bytes());
            bytes.extend_from_slice(&checkpoint_txg.0.to_le_bytes());
        }
    }
}

/// [`put_a_claimed_version`] 的逆；码认不得、字节不够交回 `None`。
fn read_a_claimed_version(
    reader: &mut RecordedBytesReader<'_>,
) -> Option<ClaimedVersionOfAGivenState> {
    match reader.u8()? {
        CLAIMED_VERSION_CODE_LANDED_ON_BY_THE_CONSULTED_RECOVERY => {
            Some(ClaimedVersionOfAGivenState::LandedOnByTheConsultedRecovery)
        }
        CLAIMED_VERSION_CODE_NEWEST_ROOT_WRITTEN_WITHOUT_A_ROOT => {
            Some(ClaimedVersionOfAGivenState::NewestRootWrittenByTheMountAndItsPublish(None))
        }
        CLAIMED_VERSION_CODE_NEWEST_ROOT_WRITTEN_WITH_A_ROOT => {
            let instance = InstanceGeneration(reader.u32()?);
            let checkpoint_txg = CheckpointTxg(reader.u64()?);
            Some(
                ClaimedVersionOfAGivenState::NewestRootWrittenByTheMountAndItsPublish(Some((
                    instance,
                    checkpoint_txg,
                ))),
            )
        }
        _ => None,
    }
}

/// 一个状态那一栏恢复自称的那一版在块键输入里的一行（与录进库的字节同一组码）。
pub(super) fn claimed_version_line_of_the_block_key(
    claimed_version: ClaimedVersionOfAGivenState,
) -> String {
    match claimed_version {
        ClaimedVersionOfAGivenState::LandedOnByTheConsultedRecovery => {
            "claimed_version=landed_on_by_the_consulted_recovery".to_string()
        }
        ClaimedVersionOfAGivenState::NewestRootWrittenByTheMountAndItsPublish(None) => {
            "claimed_version=newest_root_written:none".to_string()
        }
        ClaimedVersionOfAGivenState::NewestRootWrittenByTheMountAndItsPublish(Some((
            instance,
            checkpoint_txg,
        ))) => format!(
            "claimed_version=newest_root_written:{},{}",
            instance.0, checkpoint_txg.0
        ),
    }
}

impl RecordedGivenStatesTableWithItsColumns {
    #[must_use]
    pub fn of(table_with_its_columns: GivenStatesOfATableWithItsColumns<'_, '_>) -> Self {
        Self {
            table: RecordedGivenStatesTable::of(table_with_its_columns.table),
            container_capacity_knobs: table_with_its_columns.container_capacity_knobs,
            claimed_versions: table_with_its_columns
                .claimed_versions
                .of_every_state(table_with_its_columns.table.persisted_sets.len()),
        }
    }

    /// 读回的这一份当成一张带两栏的给定状态表用。
    pub fn with_the_table<Outcome>(
        &self,
        use_the_table: impl FnOnce(GivenStatesOfATableWithItsColumns<'_, '_>) -> Outcome,
    ) -> Outcome {
        let container_capacity_knobs = self.container_capacity_knobs;
        let claimed_versions = ClaimedVersionsOfTheStates::OfEachState(&self.claimed_versions);
        self.table.with_the_table(|table| {
            use_the_table(GivenStatesOfATableWithItsColumns {
                table,
                container_capacity_knobs,
                claimed_versions,
            })
        })
    }

    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = RECORDED_GIVEN_STATES_TABLE_WITH_ITS_COLUMNS_MAGIC.to_vec();
        for records_per_container in
            records_per_container_of_each_knob(self.container_capacity_knobs)
        {
            bytes.extend_from_slice(&records_per_container.to_le_bytes());
        }
        bytes.extend_from_slice(
            &u64::try_from(self.claimed_versions.len())
                .expect("状态数装得进 u64")
                .to_le_bytes(),
        );
        for claimed_version in &self.claimed_versions {
            put_a_claimed_version(&mut bytes, *claimed_version);
        }
        bytes.extend_from_slice(&self.table.to_bytes());
        bytes
    }

    /// 头不对、旋钮值越出它那一棵树的范围、恢复自称的那一版码认不得或个数与持久集合不等、表那一份解不出交回 `None`。
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut reader = RecordedBytesReader { bytes, at: 0 };
        if reader.take(RECORDED_GIVEN_STATES_TABLE_WITH_ITS_COLUMNS_MAGIC.len())?
            != RECORDED_GIVEN_STATES_TABLE_WITH_ITS_COLUMNS_MAGIC
        {
            return None;
        }
        let tombstone = reader.u64()?;
        let lineage = reader.u64()?;
        let retirement = reader.u64()?;
        let container_capacity_knobs = ContainerCapacityKnobs {
            tombstone: match tombstone {
                0 => None,
                records_per_container => {
                    Some(TombstoneContainerCapacityKnob::new(records_per_container)?)
                }
            },
            lineage: match lineage {
                0 => None,
                records_per_container => {
                    Some(LineageContainerCapacityKnob::new(records_per_container)?)
                }
            },
            retirement: match retirement {
                0 => None,
                records_per_container => {
                    Some(RetirementContainerCapacityKnob::new(records_per_container)?)
                }
            },
        };
        let mut claimed_versions = Vec::new();
        for _ in 0..reader.count()? {
            claimed_versions.push(read_a_claimed_version(&mut reader)?);
        }
        let table = RecordedGivenStatesTable::from_bytes(&bytes[reader.at..])?;
        if claimed_versions.len() != table.persisted_sets.len() {
            return None;
        }
        Some(Self {
            table,
            container_capacity_knobs,
            claimed_versions,
        })
    }
}
/// 三棵容器树的旋钮值，按墓碑树、谱系树、退役树的次序各一个每片记录数，0 是没装旋钮（录进库的那一栏与块键都照这一种写法）。
fn records_per_container_of_each_knob(knobs: ContainerCapacityKnobs) -> [u64; 3] {
    [
        knobs
            .tombstone
            .map_or(0, TombstoneContainerCapacityKnob::records_per_container),
        knobs
            .lineage
            .map_or(0, LineageContainerCapacityKnob::records_per_container),
        knobs
            .retirement
            .map_or(0, RetirementContainerCapacityKnob::records_per_container),
    ]
}

/// 读录入段那份字节的游标：读一样往前走一样，不够就交回 `None`。
pub(super) struct RecordedBytesReader<'bytes> {
    pub(super) bytes: &'bytes [u8],
    pub(super) at: usize,
}

impl<'bytes> RecordedBytesReader<'bytes> {
    pub(super) fn take(&mut self, length: usize) -> Option<&'bytes [u8]> {
        let end = self.at.checked_add(length)?;
        let taken = self.bytes.get(self.at..end)?;
        self.at = end;
        Some(taken)
    }
    pub(super) fn u8(&mut self) -> Option<u8> {
        self.take(1).map(|taken| taken[0])
    }
    pub(super) fn u32(&mut self) -> Option<u32> {
        Some(u32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }
    pub(super) fn u64(&mut self) -> Option<u64> {
        Some(u64::from_le_bytes(self.take(8)?.try_into().ok()?))
    }
    pub(super) fn count(&mut self) -> Option<usize> {
        usize::try_from(self.u64()?).ok()
    }
    pub(super) fn bytes(&mut self) -> Option<&'bytes [u8]> {
        let length = self.count()?;
        self.take(length)
    }
}

/// 录进库的流里一次写的种类那一个字节（[`step_kind_of_code`] 读回）；与 GPU 表里的码（`crate::crash_judge_tables::step_kind_code`）
/// 不同名：同一个编译目标里同名，单元图按裸名两份都连，判器键收进流水线的单元（「不得撞名」）。
fn recorded_step_kind_byte(kind: StepKind) -> u8 {
    match kind {
        StepKind::ZeroFill => 0,
        StepKind::UnitWrite => 1,
        StepKind::JournalRecord => 2,
        StepKind::RootRecordFua => 3,
        StepKind::SystemConfigurationSlot => 4,
        StepKind::Barrier => 5,
        StepKind::Discard => 6,
    }
}

fn step_kind_of_code(code: u8) -> Option<StepKind> {
    match code {
        0 => Some(StepKind::ZeroFill),
        1 => Some(StepKind::UnitWrite),
        2 => Some(StepKind::JournalRecord),
        3 => Some(StepKind::RootRecordFua),
        4 => Some(StepKind::SystemConfigurationSlot),
        5 => Some(StepKind::Barrier),
        6 => Some(StepKind::Discard),
        _ => None,
    }
}

fn segment_expansion_code(expansion: CrashPointReplaySegmentExpansion) -> u8 {
    match expansion {
        CrashPointReplaySegmentExpansion::NotExpanded => 0,
        CrashPointReplaySegmentExpansion::EveryProperSubset => 1,
        CrashPointReplaySegmentExpansion::InPlaceSubsetsWithCopyOnWriteNoneOrAll => 2,
    }
}

fn segment_expansion_of_code(code: u8) -> Option<CrashPointReplaySegmentExpansion> {
    match code {
        0 => Some(CrashPointReplaySegmentExpansion::NotExpanded),
        1 => Some(CrashPointReplaySegmentExpansion::EveryProperSubset),
        2 => Some(CrashPointReplaySegmentExpansion::InPlaceSubsetsWithCopyOnWriteNoneOrAll),
        _ => None,
    }
}

/// 录入段一趟登在库里的清单（键是录入键 [`RecordingKey`]，不是流名）：这条流录下的几样各自的摘要，加上录入段定下的按段登记与全域判定。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordedRunManifest {
    pub recorded_flow_digest: [u8; DIGEST_BYTES],
    pub facts_digest: [u8; DIGEST_BYTES],
    /// 配了 GPU 时录入段打包好的判表的摘要；没配 GPU 是 `None`。
    pub packed_judge_tables_digest: Option<[u8; DIGEST_BYTES]>,
    pub registered_segments: Vec<RegisteredSegment>,
    pub exhaustive: bool,
}

const RECORDED_RUN_MANIFEST_MAGIC: &[u8; 8] = b"SFRRUN01";

impl RecordedRunManifest {
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = RECORDED_RUN_MANIFEST_MAGIC.to_vec();
        bytes.extend_from_slice(&self.recorded_flow_digest);
        bytes.extend_from_slice(&self.facts_digest);
        match &self.packed_judge_tables_digest {
            Some(digest) => {
                bytes.push(1);
                bytes.extend_from_slice(digest);
            }
            None => bytes.push(0),
        }
        bytes.push(u8::from(self.exhaustive));
        bytes.extend_from_slice(
            &u64::try_from(self.registered_segments.len())
                .expect("段数装得进 u64")
                .to_le_bytes(),
        );
        for registered in &self.registered_segments {
            bytes.extend_from_slice(
                &u64::try_from(registered.segment_index)
                    .expect("段号装得进 u64")
                    .to_le_bytes(),
            );
            bytes.extend_from_slice(
                &u64::try_from(registered.write_count)
                    .expect("段写数装得进 u64")
                    .to_le_bytes(),
            );
            let text = registered.write_table_digest_text.as_bytes();
            bytes.extend_from_slice(&u64::try_from(text.len()).expect("装得进 u64").to_le_bytes());
            bytes.extend_from_slice(text);
        }
        bytes
    }

    /// 头不对、字节不够或多出、码认不得交回 `None`。
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut reader = RecordedBytesReader { bytes, at: 0 };
        if reader.take(RECORDED_RUN_MANIFEST_MAGIC.len())? != RECORDED_RUN_MANIFEST_MAGIC {
            return None;
        }
        let recorded_flow_digest: [u8; DIGEST_BYTES] =
            reader.take(DIGEST_BYTES)?.try_into().ok()?;
        let facts_digest: [u8; DIGEST_BYTES] = reader.take(DIGEST_BYTES)?.try_into().ok()?;
        let packed_judge_tables_digest = match reader.u8()? {
            0 => None,
            1 => Some(reader.take(DIGEST_BYTES)?.try_into().ok()?),
            _ => return None,
        };
        let exhaustive = match reader.u8()? {
            0 => false,
            1 => true,
            _ => return None,
        };
        let mut registered_segments = Vec::new();
        for _ in 0..reader.count()? {
            registered_segments.push(RegisteredSegment {
                segment_index: reader.count()?,
                write_count: reader.count()?,
                write_table_digest_text: String::from_utf8(reader.bytes()?.to_vec()).ok()?,
            });
        }
        if reader.at != bytes.len() {
            return None;
        }
        Some(Self {
            recorded_flow_digest,
            facts_digest,
            packed_judge_tables_digest,
            registered_segments,
            exhaustive,
        })
    }
}

/// 录入段一趟的输入摘要（录入键里写流那一半，`crash_library::recording_key_of_the_scenario_and_the_inputs`）：录下的流程的字节
/// （屏障切出的段在里面）、按段登记的段、全域判定与判表开不开（配了 GPU 才打包判表）。流名不进这一半（它是录入键里场景那一半）。流程的字节由调用方给不带片的那一份（[`recording_inputs_digest_of_the_flow_whatever_its_slices`]）：
/// 分片的段这一趟判哪一片是按库里的续点定的，判了一片之后续点就往后挪，片进了录入键，判段那几趟另定的片与录入时的不同、键就对不上。
#[must_use]
pub fn recording_inputs_digest_of_the_run(
    recorded_flow_bytes: &[u8],
    registered_segments: &[RegisteredSegment],
    exhaustive: bool,
    gpu_judges: bool,
) -> crate::crash_library::RecordingInputsDigest {
    let mut message = b"singlefs crash amplification recording inputs\n".to_vec();
    message.extend_from_slice(&sha256_digest(recorded_flow_bytes));
    for registered in registered_segments {
        message.extend_from_slice(
            format!(
                "registered_segment={} writes={} table={}\n",
                registered.segment_index,
                registered.write_count,
                registered.write_table_digest_text
            )
            .as_bytes(),
        );
    }
    message.extend_from_slice(
        format!("exhaustive={exhaustive}\ngpu_judges={gpu_judges}\n").as_bytes(),
    );
    crate::crash_library::RecordingInputsDigest(sha256_digest(&message))
}

/// 一条流的录入输入摘要，不随分片的段这一趟判哪一片变（崩溃点重放 position-addressed-trees 双机那一趟的红：本机先录先判、续点挪到下一片，
/// `SHARE=0/2`、第二台与导入那三趟只跑判段、照续点另定的片换了录入键，「库里没有流 … 的录入段清单」）：按不分片的那一份流（[`RecordedFlow::of`]）
/// 的字节算。录入段按续点定片，片与录入键下那份清单录的不同就重录、同一把键下换上新的清单；判段照清单里录下的片判（读回的流带着它）。
#[must_use]
pub fn recording_inputs_digest_of_the_flow_whatever_its_slices(
    flow: &CrashFlow<'_>,
    registered_segments: &[RegisteredSegment],
    exhaustive: bool,
    gpu_judges: bool,
) -> crate::crash_library::RecordingInputsDigest {
    recording_inputs_digest_of_the_run(
        &RecordedFlow::of(flow).to_bytes(),
        registered_segments,
        exhaustive,
        gpu_judges,
    )
}

/// 这一趟要录的流：录进库的字节（带每段这一趟的片）与录入键（场景是流名；写流那一半不带片，[`recording_inputs_digest_of_the_flow_whatever_its_slices`]）。
pub struct RecordingOfTheRun {
    pub recorded_flow_bytes: Vec<u8>,
    pub recording_key: crate::crash_library::RecordingKeyOfThisRun,
}

/// [`run_from_environment`] 录入段那一份：流的字节带片（判段照它判哪一片），录入键是流名与不带片的写流（只判那几趟照同一把键找到清单）。
#[must_use]
pub fn recording_of_the_run(
    flow_name: &str,
    flow: &CrashFlow<'_>,
    segment_slices: &[Option<CrashPointReplaySegmentSlice>],
    registered_segments: &[RegisteredSegment],
    exhaustive: bool,
    gpu_judges: bool,
) -> RecordingOfTheRun {
    RecordingOfTheRun {
        recorded_flow_bytes: RecordedFlow::of_with_slices(flow, segment_slices).to_bytes(),
        recording_key: crate::crash_library::recording_key_of_the_scenario_and_the_inputs(
            flow_name,
            &recording_inputs_digest_of_the_flow_whatever_its_slices(
                flow,
                registered_segments,
                exhaustive,
                gpu_judges,
            ),
        ),
    }
}

/// 登了分片段的流照区间判的那一趟（[`BLOCK_RANGE_ENVIRONMENT_VARIABLE`] 的份的写法）录的那一份：流的字节带片，录入键也照带片的字节算。
/// 片由区间定、不随库里的进度挪，只判那一段照同一段区间算出同一把键、读到的是自己这一段的清单；别的区间、不设区间那几趟的清单在别的键下，
/// 不互相换掉。块键不看录入键（[`PlannedCrashPoint::block_key`]），判过的块照旧互相复用。
#[must_use]
pub fn recording_of_the_run_keyed_by_its_slices(
    flow_name: &str,
    flow: &CrashFlow<'_>,
    segment_slices: &[Option<CrashPointReplaySegmentSlice>],
    registered_segments: &[RegisteredSegment],
    exhaustive: bool,
    gpu_judges: bool,
) -> RecordingOfTheRun {
    let recorded_flow_bytes = RecordedFlow::of_with_slices(flow, segment_slices).to_bytes();
    let recording_key = crate::crash_library::recording_key_of_the_scenario_and_the_inputs(
        flow_name,
        &recording_inputs_digest_of_the_run(
            &recorded_flow_bytes,
            registered_segments,
            exhaustive,
            gpu_judges,
        ),
    );
    RecordingOfTheRun {
        recorded_flow_bytes,
        recording_key,
    }
}

/// 录入段（CPU 跑数据、写进库）：写录入表；把这条流程（按段登记改过展开的那一份，字节由调用方先排好）、照计划抽的事实表、配了 GPU 时
/// 打包好的判表各写进库；最后按录入键登这一趟的清单。判段只认清单指到的那几样。
///
/// # Errors
/// 库写失败；配了 GPU 而判器建不出这条流的表（不退回 CPU 判器）。
#[allow(
    clippy::too_many_arguments,
    reason = "库、流、它录下的字节、计划、按段登记、全域判定、判表开关与录入键，各是各的东西"
)]
pub fn record_the_flow_into_the_library(
    library: &LibraryTakenAtEachReadOrWrite<'_>,
    flow: &CrashFlow<'_>,
    recorded_bytes: &[u8],
    plan: &CrashPlan,
    registered_segments: &[RegisteredSegment],
    exhaustive: bool,
    gpu_judges: bool,
    recording_key: &RecordingKey,
) -> Result<RecordedRunManifest, String> {
    write_the_recorded_blocks(&mut library.take(), plan)
        .map_err(|error| format!("录入失败：{error:?}"))?;
    let recorded_flow_digest = sha256_digest(recorded_bytes);
    library
        .take()
        .store_recorded_flow(&recorded_flow_digest, recorded_bytes)
        .map_err(|error| format!("录下的流程写不进库：{error:?}"))?;
    let expansion = |segment_index: usize, segment: &[usize]| {
        plan.segment_expansion(flow, segment_index, segment)
    };
    let facts = extract_flow_facts_with_slices(
        flow.base,
        flow.writes,
        flow.segments,
        &expansion,
        plan.segment_slices(),
    );
    let facts_digest = facts.digest();
    let facts_bytes = facts.to_bytes();
    library
        .take()
        .store_flow_facts(&facts_digest, &facts_bytes)
        .map_err(|error| format!("事实表写不进库：{error:?}"))?;
    drop(facts_bytes);
    #[cfg(not(feature = "gpu"))]
    if gpu_judges {
        return Err(
            "配置写了 ENABLE_GPU=1，这份二进制没有编进 gpu 特性（cargo … --features gpu）"
                .to_string(),
        );
    }
    #[cfg(feature = "gpu")]
    let packed_judge_tables_digest = if gpu_judges {
        let packed_bytes = packed_judge_tables_of_the_plan(flow, plan)?.to_bytes();
        let digest = sha256_digest(&packed_bytes);
        library
            .take()
            .store_packed_judge_tables(&digest, &packed_bytes)
            .map_err(|error| format!("GPU 判表写不进库：{error:?}"))?;
        Some(digest)
    } else {
        None
    };
    #[cfg(not(feature = "gpu"))]
    let packed_judge_tables_digest = None;
    let manifest = RecordedRunManifest {
        recorded_flow_digest,
        facts_digest,
        packed_judge_tables_digest,
        registered_segments: registered_segments.to_vec(),
        exhaustive,
    };
    let manifest_bytes = manifest.to_bytes();
    library
        .take()
        .store_recorded_run_manifest(recording_key, &manifest_bytes)
        .map_err(|error| format!("录入段的清单写不进库：{error:?}"))?;
    Ok(manifest)
}

/// 录入键下已经登过的清单（录入段命中就跳过，L11）；没登过、或录入键拒算（不复用）交回 `None`。
///
/// # Errors
/// 库读失败；清单解不出。
pub fn recorded_run_manifest_reused_under(
    store: &VerdictStore,
    recording: &crate::crash_library::RecordingKeyOfThisRun,
) -> Result<Option<RecordedRunManifest>, String> {
    if !recording.skips_the_recording_on_a_hit {
        return Ok(None);
    }
    let Some(bytes) = store
        .read_recorded_run_manifest(&recording.key)
        .map_err(|error| format!("读录入键下的清单失败：{error:?}"))?
    else {
        return Ok(None);
    };
    RecordedRunManifest::from_bytes(&bytes)
        .map(Some)
        .ok_or_else(|| "录入键下的清单解不出（字节坏了或编码换了）：重跑录入段".to_string())
}

/// 判段从库里读回的 GPU 判表（GPU 判器只收它）：编进 `gpu` 特性时是读回的打包判表；没编进时不会有（配置开 GPU 而没编进就在选路时报错），占个类型。
#[cfg(feature = "gpu")]
pub type PackedJudgeTablesOfTheRun = crate::crash_judge_dispatch::PackedJudgeTablesReadBack;
/// 没编进 `gpu` 特性：不会有 GPU 判表，造不出这个类型的值。
#[cfg(not(feature = "gpu"))]
#[derive(Debug)]
pub enum PackedJudgeTablesOfTheRun {}

/// 照这份计划的展开把一条流的 GPU 判表建好、打包（录入段写进库的就是这一份；对拍用例直接调卡上判的入口时也照它建）。
///
/// # Errors
/// 判器建不出这条流的表（不退回 CPU 判器）。
#[cfg(feature = "gpu")]
pub fn packed_judge_tables_of_the_plan(
    flow: &CrashFlow<'_>,
    plan: &CrashPlan,
) -> Result<crate::crash_judge_dispatch::PackedJudgeTables, String> {
    let expansion = |segment_index: usize, segment: &[usize]| {
        plan.segment_expansion(flow, segment_index, segment)
    };
    let judge_flow = crate::crash_judge_tables::JudgeFlow {
        base: flow.base,
        writes: flow.writes,
        segments: flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: flow.versions,
        expansion: &expansion,
    };
    crate::crash_judge_tables::build_judge_tables_against_the_version_table_with_slices(
        &judge_flow,
        flow.versions_without_a_file,
        plan.segment_slices(),
    )
    .map(|tables| crate::crash_judge_dispatch::PackedJudgeTables::of(&tables))
    .map_err(|refusal| {
        format!(
            "GPU 判器建不出这条流的表（{refusal:?}）。配置开了 GPU，不退回 CPU 判器：让判器的表认得这种写再跑"
        )
    })
}

/// 判段按摘要从库里读回 GPU 判表：库只在读字节那一次往返里取，核摘要与解码在锁外
/// （[`crate::crash_library::read_packed_judge_tables_back_outside_the_lock`]）。
#[cfg(feature = "gpu")]
pub(crate) fn packed_judge_tables_read_back(
    library: &LibraryTakenAtEachReadOrWrite<'_>,
    digest: &[u8; DIGEST_BYTES],
) -> Result<PackedJudgeTablesOfTheRun, String> {
    crate::crash_library::read_packed_judge_tables_back_outside_the_lock(library, digest)
}

/// 没编进 `gpu` 特性读不出 GPU 判表（也不会去读：要 GPU 判时选路先报错）。
#[cfg(not(feature = "gpu"))]
pub(crate) fn packed_judge_tables_read_back(
    _library: &LibraryTakenAtEachReadOrWrite<'_>,
    _digest: &[u8; DIGEST_BYTES],
) -> Result<PackedJudgeTablesOfTheRun, String> {
    Err("这个二进制没编进 gpu 特性，读不了 GPU 判表".to_string())
}

/// 判段从库里读回的这一趟：清单、流程、事实表、配了 GPU 时的判表。都是从库里的字节解出来的，不借录入段内存里的任何东西。
pub struct RecordedRunReadBack {
    pub manifest: RecordedRunManifest,
    pub flow: RecordedFlowReadBack,
    pub facts: crate::crash_facts::FlowFactsReadBack,
    pub packed_judge_tables: Option<PackedJudgeTablesOfTheRun>,
}

/// 判段：凭录入键（判段照同一份输入现算）读录入段登的清单，再读清单指到的几样，逐样核摘要。`flow_name` 只进报错的句子。
/// 库只在每一次读字节的往返里取，解码与核摘要在锁外。
///
/// # Errors
/// 库里没有这一份输入的清单（录入段没跑，或输入变了）、清单指到的哪一样不在、字节解不出、摘要对不上；要 GPU 判而清单里没有判表。
pub fn read_back_the_recorded_run(
    library: &LibraryTakenAtEachReadOrWrite<'_>,
    recording_key: &RecordingKey,
    flow_name: &str,
    gpu_judges: bool,
) -> Result<RecordedRunReadBack, String> {
    let manifest_bytes = library
        .take()
        .read_recorded_run_manifest(recording_key)
        .map_err(|error| format!("读录入段的清单失败：{error:?}"))?
        .ok_or_else(|| {
            format!(
                "库里没有流 {flow_name} 的录入段清单：先跑录入段（{RUN_STAGE_ENVIRONMENT_VARIABLE}=record，或不设、两段先后跑）"
            )
        })?;
    let manifest = RecordedRunManifest::from_bytes(&manifest_bytes).ok_or_else(|| {
        format!("流 {flow_name} 的录入段清单解不出（字节坏了或编码换了）：重跑录入段")
    })?;
    let flow_bytes =
        recorded_flow_bytes_in_the_library(&library.take(), &manifest.recorded_flow_digest)
            .map_err(|refusal| format!("流 {flow_name}：{refusal}"))?;
    let flow = RecordedFlowReadBack::of_the_bytes_read_from_the_library(
        &flow_bytes,
        &manifest.recorded_flow_digest,
    )
    .map_err(|refusal| format!("流 {flow_name}：{refusal}"))?;
    drop(flow_bytes);
    let facts = crate::crash_library::read_flow_facts_back_outside_the_lock(
        library,
        &manifest.facts_digest,
    )
    .map_err(|refusal| format!("流 {flow_name} 的清单指到的事实表：{refusal}"))?;
    let packed_judge_tables = match (&manifest.packed_judge_tables_digest, gpu_judges) {
        (Some(digest), true) => Some(
            packed_judge_tables_read_back(library, digest)
                .map_err(|refusal| format!("流 {flow_name} 的清单指到的 GPU 判表：{refusal}"))?,
        ),
        (None, true) => {
            return Err(format!(
                "配置开了 GPU，流 {flow_name} 的录入段清单里却没有 GPU 判表（录入段那一趟没开 GPU）：照这份配置重跑录入段"
            ));
        }
        (_, false) => None,
    };
    Ok(RecordedRunReadBack {
        manifest,
        flow,
        facts,
        packed_judge_tables,
    })
}

#[cfg(test)]
mod tests;
