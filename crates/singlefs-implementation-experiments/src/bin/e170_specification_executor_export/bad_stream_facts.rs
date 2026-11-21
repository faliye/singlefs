//! 第三段：一条流上造坏状态要的事实（基底状态、系统配置、最新一版与伪记录）。

use super::*;

pub(super) fn bad_stream_facts(stream: &PreparedStream) -> BadStreamFacts<'_> {
    let operation_index_of_write: Vec<usize> = stream
        .operations
        .iter()
        .enumerate()
        .filter(|(_, retained)| retained.operation.kind != RecordedOperationKind::Barrier)
        .map(|(operation_index, _)| operation_index)
        .collect();
    assert_eq!(
        operation_index_of_write.len(),
        stream.writes.len(),
        "写表与流里的非屏障操作一一对应（第一段 S1 的 write_order_is_the_non_barrier_operation_order）"
    );
    let publishes = publishes_of_one_recording(&stream.writes, 0);
    let root_issuing_calls = stream
        .calls
        .iter()
        .enumerate()
        .filter(|(_, call)| {
            publishes.iter().any(|publish| {
                (call.first_operation..call.return_point)
                    .contains(&operation_index_of_write[publish.root])
            })
        })
        .map(|(call_index, _)| call_index)
        .collect();
    BadStreamFacts {
        stream,
        operation_index_of_write,
        publishes,
        root_issuing_calls,
    }
}

/// 第 ⌈K/2⌉ 个（从 1 数）在下标里是 ⌈K/2⌉ − 1（登记 5.5 Mk「共 K 个时取第 ⌈K/2⌉ 个」，Mk⁻ 同一取法）。
fn middle_position(count: usize) -> usize {
    assert!(count > 0, "至少一个可取");
    count.div_ceil(2) - 1
}

/// 只认 core 写者写出的根记录（fsid 取本地常量，第一段 S1 回比过）。
fn parse_root_write(write: &RetainedWrite) -> RootRecord {
    RootRecord::parse_slot(
        write.bytes().expect("根槽写带字节"),
        &LOCAL_FILESYSTEM_IDENTIFIER,
    )
    .expect("录制流里的根槽写是 core 写者写的、自证得过")
}

fn parse_record_write(write: &RetainedWrite) -> JournalRecord {
    JournalRecord::parse(
        write.bytes().expect("journal 记录写带字节"),
        unit_filesystem_identifier(&LOCAL_FILESYSTEM_IDENTIFIER),
    )
    .expect("录制流里的记录写是 core 写者写的、自证得过")
}

/// 一版的投影写成执行器判定行里 `landed=` 的同一种写法 (实例, txg, wm=树 ID 水位, floor=F)，比对器按文本比（指针两段不进这串）。
pub(super) fn projection_text_of_root(root: &RootRecord) -> String {
    format!(
        "({},{},wm={},floor={})",
        root.instance.0,
        root.checkpoint_txg.0,
        root.tree_identifier_watermark,
        root.rollback_floor.0
    )
}

impl BadStreamFacts<'_> {
    fn persisted_before_operation(&self, operation_bound: usize) -> Vec<bool> {
        self.operation_index_of_write
            .iter()
            .map(|operation_index| *operation_index < operation_bound)
            .collect()
    }

    pub(super) fn all_persisted(&self) -> BaseState {
        BaseState {
            label: "F".to_string(),
            call: None,
            persisted: vec![true; self.stream.writes.len()],
            notes: "all_writes_persisted".to_string(),
        }
    }

    /// 调用边界：流序下标小于这次调用返回点的写全记持久、其余全没持久（登记 5.5 Mk 的定义；盲测族的 Mk 型底态同一取法）。
    pub(super) fn call_boundary(&self, call_index: usize, label: String) -> BaseState {
        let call = &self.stream.calls[call_index];
        BaseState {
            label,
            call: Some(call_index),
            persisted: self.persisted_before_operation(call.return_point),
            notes: format!(
                "call={call_index}:{}:return_point={}",
                call.label, call.return_point
            ),
        }
    }

    pub(super) fn mk(&self) -> BaseState {
        let call_index = self.root_issuing_calls[middle_position(self.root_issuing_calls.len())];
        self.call_boundary(call_index, "Mk".to_string())
    }

    /// 底态里最新的那一版：根槽写持久了的发布里下标最大的那一次。
    fn newest_publish(&self, persisted: &[bool]) -> usize {
        self.publishes
            .iter()
            .rposition(|publish| persisted[publish.root])
            .expect("底态里至少一条根槽写持久")
    }

    /// Mk⁺：Mk 再加它之后第一次发布的第一次单元写持久；那次发布与 Mk 的最后一版同实例、且有单元写，不满足就顺延到下一个发过根槽写的调用。
    /// 一个都不满足时交 None（照报，P9 不造）。
    pub(super) fn mk_plus(&self) -> Option<BaseState> {
        let middle = middle_position(self.root_issuing_calls.len());
        for (shifted, call_index) in self.root_issuing_calls[middle..].iter().enumerate() {
            let boundary = self.call_boundary(*call_index, "Mk+".to_string());
            let newest = &self.publishes[self.newest_publish(&boundary.persisted)];
            let return_point = self.stream.calls[*call_index].return_point;
            let Some(next) = self
                .publishes
                .iter()
                .find(|publish| self.operation_index_of_write[publish.root] >= return_point)
            else {
                continue;
            };
            let Some(first_unit_write) = next.units.iter().min().copied() else {
                continue;
            };
            if next.instance != newest.instance || boundary.persisted[first_unit_write] {
                continue;
            }
            let mut persisted = boundary.persisted;
            persisted[first_unit_write] = true;
            return Some(BaseState {
                label: "Mk+".to_string(),
                call: Some(*call_index),
                persisted,
                notes: format!(
                    "{}:shifted={shifted}:next_publish=({},{}):first_unit_write={first_unit_write}",
                    boundary.notes, next.instance, next.checkpoint_txg
                ),
            });
        }
        None
    }

    /// 一次发布带不带点名单元：有单元写，且它的记录里至少一条点名项数非 0。
    fn publish_names_units(&self, publish: &PublishWrites) -> bool {
        !publish.units.is_empty()
            && publish.records.iter().any(|record_write| {
                !parse_record_write(&self.stream.writes[*record_write])
                    .named
                    .is_empty()
            })
    }

    /// Mk⁻：带点名单元的发布里取居中那一次 p，流序在 p 的根槽写之前的写全记持久、其余全没持久。交回 (底态, p 在发布表里的下标)。
    pub(super) fn mk_minus(&self) -> (BaseState, usize) {
        let naming: Vec<usize> = (0..self.publishes.len())
            .filter(|publish_index| self.publish_names_units(&self.publishes[*publish_index]))
            .collect();
        let chosen = naming[middle_position(naming.len())];
        let publish = &self.publishes[chosen];
        let root_operation = self.operation_index_of_write[publish.root];
        let owning_call =
            self.stream.calls.iter().position(|call| {
                (call.first_operation..call.return_point).contains(&root_operation)
            });
        (
            BaseState {
                label: "Mk-".to_string(),
                call: owning_call,
                persisted: self.persisted_before_operation(root_operation),
                notes: format!(
                    "publish=({},{}):publishes_naming_units={}:root_operation={root_operation}",
                    publish.instance,
                    publish.checkpoint_txg,
                    naming.len()
                ),
            },
            chosen,
        )
    }

    /// 底态镜像上读一段字节（基镜像 + 持久了的写，按写表顺序叠）。
    fn read_base_image(
        &self,
        persisted: &[bool],
        device: DeviceIdentity,
        offset: u64,
        length: usize,
    ) -> Vec<u8> {
        let image = CrashImage {
            base: &self.stream.base,
            writes: &self.stream.writes,
            persisted: persisted.to_vec(),
        };
        PoolReader::read(&image, device, DeviceOffsetInBytes(offset), length)
            .expect("底态镜像在盘内的区间读得出")
    }

    /// 每块盘两槽系统配置里自证得过、世代号最大的那一槽（D22（单元原子性怎么合成） 已定项 16 的择槽；解析与自证走 core 的读者）。
    pub(super) fn chosen_system_configurations(
        &self,
        persisted: &[bool],
    ) -> Vec<ChosenSystemConfiguration> {
        let slot_bytes = usize::try_from(singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES)
            .expect("4096 装得进 usize");
        self.stream
            .base
            .devices
            .keys()
            .map(|device| {
                let slot_zero = self.read_base_image(persisted, *device, 0, slot_bytes);
                let parsed_zero = SystemConfiguration::parse_slot(&slot_zero)
                    .expect("底态只有持久与没持久两值、系统配置槽是整槽写：槽 0 自证得过");
                let slot_one_offset =
                    u64::from(parsed_zero.immutable.sizes.fixed_structure_slot_spacing);
                let slot_one =
                    self.read_base_image(persisted, *device, slot_one_offset, slot_bytes);
                let mut chosen = ChosenSystemConfiguration {
                    device: *device,
                    offset: 0,
                    bytes: slot_zero,
                    parsed: parsed_zero,
                };
                if let Ok(parsed_one) = SystemConfiguration::parse_slot(&slot_one) {
                    if parsed_one.quantities.slot_generation
                        > chosen.parsed.quantities.slot_generation
                    {
                        chosen = ChosenSystemConfiguration {
                            device: *device,
                            offset: slot_one_offset,
                            bytes: slot_one,
                            parsed: parsed_one,
                        };
                    }
                }
                chosen
            })
            .collect()
    }

    /// 底态里最新那一版 v_n：它的发布下标、根槽写、根记录、带末条标志的那条记录（记录 n）。
    pub(super) fn newest_version(&self, persisted: &[bool]) -> NewestVersion {
        let publish_index = self.newest_publish(persisted);
        let publish = &self.publishes[publish_index];
        let record_write = publish
            .records
            .iter()
            .copied()
            .find(|record_write| {
                parse_record_write(&self.stream.writes[*record_write]).place_in_publish
                    == JournalRecordPlaceInPublish::LastRecordOfThePublish
            })
            .expect("每次发布都有一条带末条标志的记录（D23 已定项 17）");
        NewestVersion {
            publish_index,
            root_write: publish.root,
            root: parse_root_write(&self.stream.writes[publish.root]),
            record_write,
            record: parse_record_write(&self.stream.writes[record_write]),
        }
    }

    /// 发布表里第 `publish_index` 次发布之前那一版的投影文本；它是第一次发布时，前一版是基镜像上的现行那一版（mkfs 的第 0 代根，
    /// 按 txg 0 在根环里该落的位置读）。
    pub(super) fn previous_version_text(&self, publish_index: usize) -> String {
        if let Some(previous) = publish_index.checked_sub(1) {
            return projection_text_of_root(&parse_root_write(
                &self.stream.writes[self.publishes[previous].root],
            ));
        }
        let nothing_persisted = vec![false; self.stream.writes.len()];
        let configuration = &self.chosen_system_configurations(&nothing_persisted)[0].parsed;
        let target = target_for_publish(
            CheckpointTxg(0),
            configuration.immutable.sizes.root_ring_slots_per_region,
        );
        let device = configuration.immutable.region_devices
            [usize::try_from(target.region).expect("区域号装得进 usize")];
        let offset = slot_offset(
            target,
            configuration.immutable.sizes.fixed_structure_slot_spacing,
        )
        .0;
        let width =
            usize::try_from(configuration.immutable.sizes.physical_block_size).expect("根槽宽");
        let bytes = self.read_base_image(&nothing_persisted, device, offset, width);
        projection_text_of_root(
            &RootRecord::parse_slot(&bytes, &LOCAL_FILESYSTEM_IDENTIFIER)
                .expect("基镜像上的第 0 代根自证得过"),
        )
    }
}

/// 一块盘上择到的系统配置槽：盘、槽在盘内的偏移、整槽字节、core 读者解出来的内容。
pub(super) struct ChosenSystemConfiguration {
    pub(super) device: DeviceIdentity,
    pub(super) offset: u64,
    pub(super) bytes: Vec<u8>,
    pub(super) parsed: SystemConfiguration,
}

/// 底态里最新的那一版 v_n（登记 5.5 记号）。
pub(super) struct NewestVersion {
    pub(super) publish_index: usize,
    pub(super) root_write: usize,
    pub(super) root: RootRecord,
    record_write: usize,
    pub(super) record: JournalRecord,
}

/// 一条伪记录的写法（登记 5.5「伪记录的头照记录 n 抄，改下面写明的字段」：点名项数 0、事务号 0、提交标记 1，反向链取逻辑前一条的头，
/// 载荷校验和与头校验和由 core 的写者重算，两块盘各写一份，落在计数器决定的环槽）。
pub(super) struct PseudoRecord {
    counter: u64,
    pub(super) checkpoint_txg: u64,
    pub(super) ordinal: u32,
    is_last_of_the_publish: bool,
    pub(super) watermark_step: u64,
}

impl BadStreamFacts<'_> {
    /// P1：两块盘上各自择到的那一槽，根环起点 64 → 65，照 core 的算法重封整槽校验和。另交回「补丁之后 core 读者读回 65、补丁之前读回 64」。
    pub(super) fn root_ring_base_patches(&self, persisted: &[bool]) -> (Vec<BytePatch>, bool) {
        let mut reads_back = true;
        let patches = self
            .chosen_system_configurations(persisted)
            .into_iter()
            .map(|chosen| {
                reads_back &= chosen.parsed.immutable.root_ring_base_slot.0
                    == BAD_ROOT_RING_BASE_SLOT_FIRST_VERSION;
                let mut bytes = chosen.bytes;
                bytes[BAD_SYSTEM_CONFIGURATION_ROOT_RING_BASE_OFFSET
                    ..BAD_SYSTEM_CONFIGURATION_ROOT_RING_BASE_OFFSET + 8]
                    .copy_from_slice(&BAD_ROOT_RING_BASE_SLOT_PATCHED.to_le_bytes());
                let slot_bytes = bytes.len();
                let digest = wide_checksum_with_field_zeroed(
                    &bytes,
                    slot_bytes,
                    SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
                );
                bytes[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET
                    ..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + digest.len()]
                    .copy_from_slice(&digest);
                reads_back &= SystemConfiguration::parse_slot(&bytes).is_ok_and(|patched| {
                    patched.immutable.root_ring_base_slot.0 == BAD_ROOT_RING_BASE_SLOT_PATCHED
                });
                BytePatch {
                    device: chosen.device,
                    offset: chosen.offset,
                    bytes,
                }
            })
            .collect();
        (patches, reads_back)
    }

    /// P2 型：根 n 的 466 字节、txg 改成给定值、重封，落在区域公式给这个 txg 的那一槽（区域 txg mod R、槽 (txg div R) mod S，
    /// 盘按逐区域设备身份；几何取底态择到的系统配置，位置由 core 的 `target_for_publish`、`slot_offset` 算）。根 n 不动。
    pub(super) fn pseudo_root_patch(
        &self,
        persisted: &[bool],
        newest: &NewestVersion,
        checkpoint_txg: u64,
    ) -> BytePatch {
        let configuration = &self.chosen_system_configurations(persisted)[0].parsed;
        let target = target_for_publish(
            CheckpointTxg(checkpoint_txg),
            configuration.immutable.sizes.root_ring_slots_per_region,
        );
        let slot_width = usize::try_from(self.stream.writes[newest.root_write].length_in_bytes())
            .expect("根槽宽装得进 usize");
        let mut root = newest.root;
        root.checkpoint_txg = CheckpointTxg(checkpoint_txg);
        BytePatch {
            device: configuration.immutable.region_devices
                [usize::try_from(target.region).expect("区域号装得进 usize")],
            offset: slot_offset(
                target,
                configuration.immutable.sizes.fixed_structure_slot_spacing,
            )
            .0,
            bytes: root.to_slot(slot_width),
        }
    }

    /// 根 n 原位改字段、重封（P5、N1）。
    pub(super) fn root_in_place_patch(
        &self,
        newest: &NewestVersion,
        change: impl FnOnce(&mut RootRecord),
    ) -> BytePatch {
        let write = &self.stream.writes[newest.root_write];
        let mut root = newest.root;
        change(&mut root);
        BytePatch {
            device: write.device,
            offset: write.offset.0,
            bytes: root
                .to_slot(usize::try_from(write.length_in_bytes()).expect("根槽宽装得进 usize")),
        }
    }

    /// 不重封：底态镜像上那一个字节翻一位（P8、N4、N5）。
    pub(super) fn flipped_byte_patch(
        &self,
        persisted: &[bool],
        device: DeviceIdentity,
        offset: u64,
    ) -> BytePatch {
        // 读者只收按扇区对齐的读（`check_request_against_the_device_contract`）：读整个扇区，取那一个字节。
        let sector_start = offset - offset % LOCAL_SECTOR_BYTES;
        let sector = self.read_base_image(
            persisted,
            device,
            sector_start,
            usize::try_from(LOCAL_SECTOR_BYTES).expect("512 装得进 usize"),
        );
        let original = sector[usize::try_from(offset - sector_start).expect("扇区内偏移")];
        BytePatch {
            device,
            offset,
            bytes: vec![original ^ BAD_FLIPPED_BIT],
        }
    }

    /// 伪记录（P3、P4、P9、N2、N3 与盲测 P3 型）：头照记录 n 抄，逐条改写法里的字段；第二条起反向链取前一条伪记录的头。
    pub(super) fn pseudo_record_patches(
        &self,
        persisted: &[bool],
        newest: &NewestVersion,
        pseudo_records: &[PseudoRecord],
    ) -> Vec<BytePatch> {
        let ring_bytes = self.chosen_system_configurations(persisted)[0]
            .parsed
            .immutable
            .sizes
            .journal_ring_bytes;
        let template = &newest.record;
        let mut previous_bytes = self.stream.writes[newest.record_write]
            .bytes()
            .expect("记录写带字节")
            .to_vec();
        let mut patches = Vec::new();
        for pseudo in pseudo_records {
            let record = JournalRecord {
                instance: template.instance,
                counter: pseudo.counter,
                checkpoint_txg: CheckpointTxg(pseudo.checkpoint_txg),
                transaction: 0,
                is_commit: true,
                ordinal_within_publish: JournalRecordOrdinalWithinPublish(pseudo.ordinal),
                place_in_publish: if pseudo.is_last_of_the_publish {
                    JournalRecordPlaceInPublish::LastRecordOfThePublish
                } else {
                    JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow
                },
                back_chain: back_chain_of(&previous_bytes),
                filesystem_identifier: template.filesystem_identifier,
                new_tree_table: template.new_tree_table,
                new_mapping_root: template.new_mapping_root,
                new_tree_identifier_watermark: template.new_tree_identifier_watermark
                    + pseudo.watermark_step,
                new_rollback_floor: template.new_rollback_floor,
                new_instance_table: template.new_instance_table,
                new_allocation_record_tree_root: template.new_allocation_record_tree_root,
                new_unmount_marker: template.new_unmount_marker,
                new_reclaim_threshold: template.new_reclaim_threshold,
                new_pinned_ring_slot: template.new_pinned_ring_slot,
                named: Vec::new(),
            };
            let bytes = record.to_bytes();
            let offset = record_offset(pseudo.counter, ring_bytes).0;
            for device in self.stream.base.devices.keys() {
                patches.push(BytePatch {
                    device: *device,
                    offset,
                    bytes: bytes.clone(),
                });
            }
            previous_bytes = bytes;
        }
        patches
    }
}

/// 一行坏状态的改法（登记 5.5 的 P 表、N 表与盲测族三型）。
#[derive(Clone, Copy, Debug)]
pub(super) enum BadShape {
    /// P1：两块盘上择到的系统配置槽，根环起点 64 → 65，重封。
    RootRingBaseMisdirected,
    /// P2（盲测 P2 型 txg_step = 3j）：伪根 txg n + txg_step。
    PseudoRootAhead { txg_step: u64 },
    /// P3（盲测 P3 型）、P9：伪续接一条，水位 + watermark_step。
    PseudoContinuation { watermark_step: u64 },
    /// P4：伪续接两条，末条标志 0、1。
    PseudoContinuationOfTwo,
    /// P5（盲测 P5 型）：根 n 的实例代号 i → i − 1，重封。
    RootInstanceLowered,
    /// P8、P10：根 n 偏移 122 翻一位、记录 n 两份各翻一位，都不重封。补丁与落点两处同一个写法；期望判定由调用处给：
    /// 第一次的表 P8 写「不在」，第 1 次重跑的表 P8 写「在」、P10（Mk⁺ 底态）写「不在」。
    NewestRootAndRecordUnreadable { expected_verdict: &'static str },
    /// N1：根 n 的 txg n → n − 2，重封，原位。
    RootTxgLowered,
    /// N2：同 P3，计数器 c_n + 2。
    PseudoRecordAfterAGap,
    /// N3：同 P4，末条标志 0、0。
    PseudoPairWithoutLastFlag,
    /// N4：根 n 偏移 122 翻一位，不重封。
    NewestRootUnverified,
    /// N5：p 的第一次单元写在盘身份较小那块盘上的那一份，单元内偏移 200 翻一位。
    NamedUnitCopyCorrupted { publish_index: usize },
}

impl BadStreamFacts<'_> {
    /// 版本表里出现过的 (实例代号, txg)：基镜像的第 0 代 (0, 0) 加每一次根槽写（P5 的前提断言用）。
    pub(super) fn version_identities(&self) -> BTreeSet<(u32, u64)> {
        std::iter::once((0, 0))
            .chain(
                self.publishes
                    .iter()
                    .map(|publish| (publish.instance, publish.checkpoint_txg)),
            )
            .collect()
    }

    /// 记录 n 在两块盘上的两份（写表下标）。
    pub(super) fn record_copies_of(&self, newest: &NewestVersion) -> Vec<usize> {
        self.publishes[newest.publish_index]
            .records
            .iter()
            .copied()
            .filter(|record_write| {
                parse_record_write(&self.stream.writes[*record_write]).counter
                    == newest.record.counter
            })
            .collect()
    }
}

pub(super) fn pseudo_continuation(
    newest: &NewestVersion,
    counter_step: u64,
    watermark_step: u64,
) -> PseudoRecord {
    PseudoRecord {
        counter: newest.record.counter + counter_step,
        checkpoint_txg: newest.root.checkpoint_txg.0 + 1,
        ordinal: 1,
        is_last_of_the_publish: true,
        watermark_step,
    }
}

pub(super) fn pseudo_pair(newest: &NewestVersion, second_is_last: bool) -> [PseudoRecord; 2] {
    [
        PseudoRecord {
            is_last_of_the_publish: false,
            ..pseudo_continuation(newest, 1, BAD_WATERMARK_STEP)
        },
        PseudoRecord {
            counter: newest.record.counter + 2,
            ordinal: 2,
            is_last_of_the_publish: second_is_last,
            ..pseudo_continuation(newest, 1, BAD_WATERMARK_STEP)
        },
    ]
}

#[cfg(test)]
mod tests;
