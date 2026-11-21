//! 从 harness 录一条流（[`CrashPointRecorder`]）：每次写归哪一段、写的类别。

use super::*;

// ============================== 在测试代码里给崩溃点写身份 ==============================

/// 一次写属于哪个周期：录制流只带写的种类；单元写再按单元头第 6 字节的类标签分成写数据（码 1）、更新索引树（码 2）、
/// 写打包记录（码 3）——单元自描述，不用核心层打标记（周期是「写入、写日志、更新索引树」这一级）。
/// 认不出类标签的单元写仍叫 `unit_write`。
#[must_use]
pub fn period_of_write(write: &RetainedWrite) -> &'static str {
    match (write.kind, &write.contents) {
        (StepKind::UnitWrite, WrittenContents::Bytes(bytes)) => {
            period_of_unit_class(bytes.get(6).copied())
        }
        (StepKind::UnitWrite, WrittenContents::Zeros { .. }) => "unit_write",
        (
            StepKind::ZeroFill
            | StepKind::Discard
            | StepKind::JournalRecord
            | StepKind::RootRecordFua
            | StepKind::SystemConfigurationSlot
            | StepKind::Barrier,
            _,
        ) => write.kind.name(),
    }
}

/// 单元头第 6 字节的类标签 → 周期名（与池级 checker `unit_checksum_layout` 认的三个码相同）。
#[must_use]
pub fn period_of_unit_class(unit_class: Option<u8>) -> &'static str {
    match unit_class {
        Some(1) => "data_unit",
        Some(2) => "index_node",
        Some(3) => "packed_records",
        Some(_) | None => "unit_write",
    }
}

/// 崩溃点记录器：测试代码调一个入口（取号、暖机、发布、回退、抬 F、卸载）时包一层，记下它发出的那段录制流与设它的文件；
/// 录制流自己只记卸载入口（`RecordedPublishEntry::Unmount`），别的入口的名字只有调它的测试代码知道，所以身份在这里写。
/// 换成崩溃点时每个屏障段一个点，名字写成 `<流程>.<周期>.<步>`（流程 = 入口的名字，周期 = 这一段里写的种类，步 = 这个入口里这一种周期的第几段；
/// 路径段按「测哪条流程的哪个周期的第几步」写、节点切到周期与步）。同一个入口调几次就是几段路径，名字相同、位置不同。
pub struct CrashPointRecorder {
    stream: crate::SharedStream,
    mkfs_operation_count: usize,
    /// （设它的文件，入口的名字，录制流下标的半开区间，绝对下标）。
    spans: Vec<(String, String, Range<usize>)>,
}

impl CrashPointRecorder {
    #[must_use]
    pub fn new(stream: crate::SharedStream, mkfs_operation_count: usize) -> Self {
        Self {
            stream,
            mkfs_operation_count,
            spans: Vec::new(),
        }
    }

    /// 已经发出去的一段（例如 harness 的 `build_pool` 里取号加暖机那一段），按录制流的绝对下标记；`code_file` 给 `file!()`，
    /// 归一成仓内路径（[`repository_relative_file`]）。
    ///
    /// # Panics
    /// 这一段没接上前一段（中间有没归任何点的操作，或倒退）。
    pub fn record_span(&mut self, code_file: &str, name: &str, operations: Range<usize>) {
        let expected_start = self
            .spans
            .last()
            .map_or(self.mkfs_operation_count, |(_, _, span)| span.end);
        assert_eq!(
            operations.start, expected_start,
            "崩溃点 {name} 那一段没接上前一个点"
        );
        assert!(operations.end >= operations.start);
        self.spans.push((
            repository_relative_file(code_file),
            name.to_string(),
            operations,
        ));
    }

    /// 调一个入口，把它发出的那段流记下来；`code_file` 给 `file!()`。
    pub fn step<Output>(
        &mut self,
        code_file: &str,
        name: &str,
        body: impl FnOnce() -> Output,
    ) -> Output {
        let start = self.stream.operation_count();
        let output = body();
        let end = self.stream.operation_count();
        self.record_span(code_file, name, start..end);
        output
    }

    /// 换成写表上的崩溃点：每个屏障段一个点，归它最后一次写落在的那个入口；名字 `<入口>.<这一段里写的种类，`+` 接>.<这个入口里这一种周期的第几段>`。
    /// `writes`、`segments`、`stream_indexes` 是 `writes_and_segments_with_stream_indexes` 交回的三样（下标都在 mkfs 之后那段录制流里）。
    ///
    /// # Panics
    /// 记下的段没有罩住整条流（流尾还有没归任何点的操作）；段不是首尾相接的写区间；有写不属于任何段。
    #[must_use]
    pub fn into_crash_points(
        self,
        writes: &[RetainedWrite],
        segments: &[Vec<usize>],
        stream_indexes: &[usize],
    ) -> Vec<CrashPointSpan> {
        let end = self
            .spans
            .last()
            .map_or(self.mkfs_operation_count, |(_, _, span)| span.end);
        assert_eq!(
            end,
            self.stream.operation_count(),
            "流尾还有没归任何崩溃点的操作"
        );
        assert_eq!(
            writes.len(),
            stream_indexes.len(),
            "每次写各有一个录制流下标"
        );
        // 每个入口管的写区间（首尾相接）
        let mut entries: Vec<(String, String, Range<usize>)> = Vec::new();
        let mut next_write = 0usize;
        for (code_file, name, operations) in self.spans {
            let relative_end = operations.end - self.mkfs_operation_count;
            let write_end = stream_indexes[next_write..]
                .iter()
                .position(|index| *index >= relative_end)
                .map_or(stream_indexes.len(), |offset| next_write + offset);
            entries.push((code_file, name, next_write..write_end));
            next_write = write_end;
        }
        assert_eq!(next_write, stream_indexes.len(), "每次写都归了某个入口");
        let mut points = Vec::new();
        let mut expected_start = 0usize;
        let mut steps_of_period_in_entry: BTreeMap<(usize, String), usize> = BTreeMap::new();
        for segment in segments {
            let first = *segment.first().expect("段里至少一次写");
            let last = *segment.last().expect("段里至少一次写");
            assert_eq!(first, expected_start, "段没有接上前一段的写区间");
            assert!(
                segment.windows(2).all(|pair| pair[1] == pair[0] + 1),
                "一段里的写下标连续"
            );
            expected_start = last + 1;
            let entry_index = entries
                .iter()
                .position(|(_, _, range)| range.contains(&last))
                .expect("每一段的最后一次写归某个入口");
            let mut kinds: Vec<&'static str> = Vec::new();
            for write_index in segment {
                let kind = period_of_write(&writes[*write_index]);
                if !kinds.contains(&kind) {
                    kinds.push(kind);
                }
            }
            let period = kinds.join("+");
            let step = steps_of_period_in_entry
                .entry((entry_index, period.clone()))
                .or_insert(0);
            *step += 1; // 这一种周期在这个入口里的第几段
            let step_in_period = *step;
            let (code_file, entry_name, _) = &entries[entry_index];
            points.push(CrashPointSpan {
                name: format!("{entry_name}.{period}.{step_in_period}"),
                code_file_in_repository: code_file.clone(),
                writes: first..last + 1,
                ignored: false,
            });
        }
        assert_eq!(expected_start, writes.len(), "每次写都在某一段里");
        points
    }
}

#[cfg(test)]
mod tests;
