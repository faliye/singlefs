//! 大值分段：超过一段的值拆成几条写、读回时按版本拼回（`crate::segmented_values`），读缺的段、重读原键。

use super::*;

/// 拼分段存的键时经服务读段、重读原键（[`StoredSegmentsReader`]）：`key_in_the_service` 是原键在服务里的全名。
struct SegmentReadsOverTheService<'client> {
    client: &'client mut ReliableClient,
    table: Table,
    key_in_the_service: &'client [u8],
}

impl StoredSegmentsReader for SegmentReadsOverTheService<'_> {
    type ReadFailure = ClientFailure;

    fn read_missing_segments(
        &mut self,
        being_assembled: &SegmentedValueBeingAssembled,
    ) -> Result<SegmentsReadByPoint, ClientFailure> {
        self.client
            .read_missing_segments(self.table, self.key_in_the_service, being_assembled)
    }

    fn reread_the_original_key(&mut self) -> Result<Option<Vec<u8>>, ClientFailure> {
        Ok(self
            .client
            .read_as_many_values_as_keys(&[(self.table, self.key_in_the_service.to_vec())])?
            .pop()
            .flatten())
    }

    fn wait(&mut self, delay: Duration) {
        thread::sleep(delay);
    }
}

impl ReliableClient {
    /// 成批点读（照 [`Self::read`]），回的值个数要与问的键数相同。
    fn read_as_many_values_as_keys(
        &mut self,
        keys: &[(Table, Vec<u8>)],
    ) -> Result<Vec<Option<Vec<u8>>>, ClientFailure> {
        let values = self.read(keys)?;
        if values.len() != keys.len() {
            return Err(ClientFailure::Protocol(format!(
                "成批点读问了 {} 个键，服务回了 {} 个值",
                keys.len(),
                values.len()
            )));
        }
        Ok(values)
    }

    /// 成批写之前把大值分段（文件头「大值分段」；Python 客户端 `write_many` 的第 1 到第 3 步）：先点读这批键的版本登记键，登记了版本的再点读原键取此刻那一版；
    /// 登记了而不是此刻那一版的按前缀删掉（第 2 步）；要分段的拆成这一版的段（段键带整值的 sha256，不就地覆写旧段），版本登记键写成此刻那一版并上这一批写的各版，
    /// 按 `frame_byte_budget` 切帧先写登记键、等确认，再写各段、等确认；把清单连同不分段的条目交回，由调用方照它自己的切法最后写
    /// （一块一批落在一帧里的原子由调用方的切法保住）。清单写进之后的回收（Python 的第 5 步）留给这把键下一次写时的第 2 步。
    /// 弄坏开关 segments-overwritten-in-place 下段键不带版本、就地覆写，不读也不写登记键、不删（改前的样子）。
    ///
    /// # Errors
    /// 有一把原键的形状是段键或版本登记键、有一个值要的段数超过段号装得下的（[`ClientFailure::SegmentedValue`]，一条都没写）；登记键或原键读不成
    /// （照 [`Self::read`]）、登记键解不开（[`ClientFailure::SegmentedValue`]）；删前缀不成（照 [`Self::delete_prefix`]）；写不成（照 [`Self::submit_write_entries`] 与
    /// [`Self::wait_all_acknowledged`]：清单还没写，此刻那一版照旧读得回，下一次写这把键时第 2 步删掉这一次写下的段）。
    pub fn write_segments_of_large_values(
        &mut self,
        entries: Vec<TableEntry>,
        frame_byte_budget: usize,
    ) -> Result<Vec<TableEntry>, ClientFailure> {
        if entries.is_empty() {
            return Ok(entries);
        }
        let registry_keys = version_registry_keys_to_read_before_writing(&entries)
            .map_err(ClientFailure::SegmentedValue)?;
        let placement = if self.is_enabled(BreakSwitch::SegmentsOverwrittenInPlace) {
            SegmentPlacement::OverwrittenInPlace
        } else {
            SegmentPlacement::Versioned
        };
        let mut stored_versions_of_each_entry = vec![StoredVersions::default(); entries.len()];
        if placement == SegmentPlacement::Versioned {
            self.read_stored_versions(
                &entries,
                &registry_keys,
                frame_byte_budget,
                &mut stored_versions_of_each_entry,
            )?;
        }
        let plan = plan_segmented_write(entries, &stored_versions_of_each_entry, placement)
            .map_err(ClientFailure::SegmentedValue)?;
        let mut deleted_prefixes: Vec<Vec<u8>> = Vec::new();
        for ((_table, key, _value), stored_versions) in plan
            .entries_to_write_last
            .iter()
            .zip(&stored_versions_of_each_entry)
        {
            for version in stored_versions.stale_versions() {
                let prefix = segment_version_prefix(key, &version);
                if !deleted_prefixes.contains(&prefix) {
                    self.delete_prefix(&prefix, MAXIMUM_DELETE_PREFIX_KEYS_PER_FRAME, false)?;
                    deleted_prefixes.push(prefix);
                }
            }
        }
        for step in [&plan.version_registry_entries, &plan.segment_entries] {
            for frame in frame_ranges(
                step,
                |(_table, key, value)| WRITE_BYTES_PER_ENTRY + key.len() + value.len(),
                frame_byte_budget,
                SEGMENTED_FRAME_ENTRY_LIMIT,
            ) {
                self.submit_write_entries(&step[frame])?;
            }
            self.wait_all_acknowledged()?;
        }
        Ok(plan.entries_to_write_last)
    }

    /// 写之前读各键的版本（Python 客户端 `write_many` 第 1 步）：版本登记键逐帧点读，登记了版本的再点读原键取此刻那份清单的版本，填进与 `entries` 同序的
    /// `stored_versions_of_each_entry`。
    fn read_stored_versions(
        &mut self,
        entries: &[TableEntry],
        registry_keys: &[(Table, Vec<u8>)],
        frame_byte_budget: usize,
        stored_versions_of_each_entry: &mut [StoredVersions],
    ) -> Result<(), ClientFailure> {
        for frame in frame_ranges(
            registry_keys,
            |(_table, registry_key)| POINT_READ_BYTES_PER_KEY + registry_key.len(),
            frame_byte_budget,
            SEGMENTED_FRAME_ENTRY_LIMIT,
        ) {
            let stored_registries =
                self.read_as_many_values_as_keys(&registry_keys[frame.clone()])?;
            for (position, stored_registry) in frame.zip(stored_registries) {
                stored_versions_of_each_entry[position].registered =
                    parse_version_registry(&entries[position].1, stored_registry.as_deref())
                        .map_err(ClientFailure::SegmentedValue)?;
            }
        }
        let registered_positions: Vec<usize> = (0..entries.len())
            .filter(|position| {
                !stored_versions_of_each_entry[*position]
                    .registered
                    .is_empty()
            })
            .collect();
        let original_keys: Vec<(Table, Vec<u8>)> = registered_positions
            .iter()
            .map(|position| (entries[*position].0, entries[*position].1.clone()))
            .collect();
        for frame in frame_ranges(
            &original_keys,
            |(_table, key)| POINT_READ_BYTES_PER_KEY + key.len(),
            frame_byte_budget,
            SEGMENTED_FRAME_ENTRY_LIMIT,
        ) {
            let stored_originals =
                self.read_as_many_values_as_keys(&original_keys[frame.clone()])?;
            for (position, stored_original) in frame.zip(stored_originals) {
                let entry_position = registered_positions[position];
                stored_versions_of_each_entry[entry_position].current = current_version_of_stored(
                    &entries[entry_position].1,
                    stored_original.as_deref(),
                );
            }
        }
        Ok(())
    }

    /// 成批点读，分段存的键拼回整值（文件头「大值分段」；Python 客户端 `read_many`）：原键照 [`Self::read`] 点读（回应越过帧长上限就拆键），
    /// 读回的是清单就按清单点读各段（[`Self::assemble_segmented_value`]）。
    ///
    /// # Errors
    /// 同 [`Self::read`]；回的值个数与问的键数不同（[`ClientFailure::Protocol`]）；清单解不开、段缺了、长度或 sha256 对不上
    /// （[`ClientFailure::SegmentedValue`]），不当成键不在。
    pub fn read_assembling_segmented_values(
        &mut self,
        keys: &[(Table, Vec<u8>)],
    ) -> Result<Vec<Option<Vec<u8>>>, ClientFailure> {
        let stored_values = self.read_as_many_values_as_keys(keys)?;
        let mut values = Vec::with_capacity(keys.len());
        for ((table, key), stored) in keys.iter().zip(stored_values) {
            let Some(stored) = stored else {
                values.push(None);
                continue;
            };
            match SegmentManifest::parse_stored_value(key, &stored)
                .map_err(ClientFailure::SegmentedValue)?
            {
                None => values.push(Some(stored)),
                Some(manifest) => {
                    let (_owner_key, whole) = self.assemble_segmented_value(
                        *table,
                        key,
                        SegmentedValueBeingAssembled::read_by_point(key.clone(), manifest),
                    )?;
                    values.push(Some(whole));
                }
            }
        }
        Ok(values)
    }

    /// 一把分段存的键拼回整值（Python 客户端 `assembled`）：按清单点读还缺的段（每帧至多 [`SEGMENT_READ_FRAME_BYTES`]）、拼回、核每段长度与
    /// 整值 sha256，交 (调用方的原键, 整值)；拼不回照 [`assemble_rereading_the_original_key`] 重读原键（清单换过照新清单重拼）。
    /// `key_in_the_service` 是这把原键在服务里的全名（按前缀扫的一方交过来的原键去掉过自己的键空间，读段与重读原键要接回去）。
    ///
    /// # Errors
    /// 读段不成（照 [`Self::read`]）；等完清单没换、段缺了、长度或 sha256 对不上（[`ClientFailure::SegmentedValue`]）。
    pub fn assemble_segmented_value(
        &mut self,
        table: Table,
        key_in_the_service: &[u8],
        being_assembled: SegmentedValueBeingAssembled,
    ) -> Result<(Vec<u8>, Vec<u8>), ClientFailure> {
        let mut reader = SegmentReadsOverTheService {
            client: self,
            table,
            key_in_the_service,
        };
        assemble_rereading_the_original_key(being_assembled, &mut reader)?
            .map_err(ClientFailure::SegmentedValue)
    }

    /// 点读一把分段存的键清单点名、还缺的段（每帧至多 [`SEGMENT_READ_FRAME_BYTES`]），段键取清单的版本。
    fn read_missing_segments(
        &mut self,
        table: Table,
        key_in_the_service: &[u8],
        being_assembled: &SegmentedValueBeingAssembled,
    ) -> Result<SegmentsReadByPoint, ClientFailure> {
        let missing_segment_indexes = being_assembled.missing_segment_indexes();
        let mut segments_read_by_point = Vec::with_capacity(missing_segment_indexes.len());
        // 段键带清单的版本；弄坏开关 segments-overwritten-in-place 下不带（与写的一侧同一个开关）
        let version = (!self.is_enabled(BreakSwitch::SegmentsOverwrittenInPlace))
            .then(|| being_assembled.version());
        for frame in frame_ranges(
            &missing_segment_indexes,
            |segment_index| {
                being_assembled.segment_length_in_bytes(*segment_index)
                    + key_in_the_service.len()
                    + SEGMENT_KEY_TAIL_BYTES
                    + SEGMENT_READ_OVERHEAD_BYTES_PER_KEY
            },
            SEGMENT_READ_FRAME_BYTES,
            SEGMENTED_FRAME_ENTRY_LIMIT,
        ) {
            let segment_indexes = &missing_segment_indexes[frame];
            let segment_keys: Vec<(Table, Vec<u8>)> = segment_indexes
                .iter()
                .map(|segment_index| {
                    (
                        table,
                        segment_key(key_in_the_service, version, *segment_index),
                    )
                })
                .collect();
            let pieces = self.read_as_many_values_as_keys(&segment_keys)?;
            segments_read_by_point.extend(segment_indexes.iter().copied().zip(pieces));
        }
        Ok(segments_read_by_point)
    }
}
