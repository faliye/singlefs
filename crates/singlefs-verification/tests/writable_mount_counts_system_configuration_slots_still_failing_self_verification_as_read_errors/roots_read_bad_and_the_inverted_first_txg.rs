//! 根读坏与倒挂的第一个 txg：一块盘留着两槽自证过的旧槽、别的池的槽带不认识的 incompat 位时倒挂的第一个 txg 照放行（盘 0 不落后时同样的故障拒），根的记录读不出时第一个 txg 越过它，根只在读阶段之后读坏时新实例照样编号在选中版本之上。

use super::*;

/// 钉现状，条款不罩这一形（归 C6（块层语义假设写错） 那一类丢写，D22（单元原子性怎么合成） 已定项 8 射程写明不罩）：C971 的根与记录读坏（C、D），
/// 盘 1 最新那一槽读回全 0，盘 0 没收到那次轮换（两槽都自证过、见证值都不超过 B）。「没有一块盘两槽都自证过」不成立，乙入判不出不判，
/// c_见证 落回 B，可写挂载做成，新实例第一次发布落到被藏的 D 之下。
#[test]
fn a_device_holding_two_stale_self_verified_slots_lets_the_inverted_first_txg_through() {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let history = minimal_history_of_c971(&parameters);
    let mut image = history.image.clone();
    let device_zero = DeviceIdentity(0);
    let newest = newest_system_configuration_slot(&image, device_zero);
    let older = older_system_configuration_slot(&image, device_zero);
    let mut stale = SystemConfiguration::parse_slot(&slot_bytes_at(&image, device_zero, older))
        .expect("较旧那一槽本池自证过");
    stale.quantities.slot_generation -= 1;
    write_slot(&mut image, device_zero, newest, &stale.to_slot());
    assert_eq!(
        slot_counts_of(&image, device_zero),
        (0, 2, 0),
        "前提：盘 0 两槽都自证过"
    );
    let mut faults = read_faults_of_a_six_cell(
        &parameters,
        &history,
        ReadFaultKind::ReadsBackAllZeros,
        ReadFaultKind::ReadsBackAllZeros,
    );
    faults.retain(|fault| {
        fault.device != device_zero
            || fault.offset_in_bytes >= 2 * u64::from(FIXED_STRUCTURE_SLOT_SPACING)
    });
    let (outcome, _) = writable_mount_under_read_faults(&parameters, &image, &faults);
    let mounted = outcome.expect("可写挂载做成");
    assert!(
        mounted.output.row_publish.root().checkpoint_txg.0 <= TXG_OF_THE_NEWEST_VERSION,
        "新实例第一次发布落到被藏的 D 之下"
    );
}

/// 上一条的对照：同一组读坏，盘 0 没落后（最新那一槽见证到 C）。c_见证 比所选那一版 B 新，判据 N-配置 比出来为真，拒。
#[test]
fn the_same_faults_are_refused_when_device_zero_is_not_behind() {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let history = minimal_history_of_c971(&parameters);
    let device_zero = DeviceIdentity(0);
    let mut faults = read_faults_of_a_six_cell(
        &parameters,
        &history,
        ReadFaultKind::ReadsBackAllZeros,
        ReadFaultKind::ReadsBackAllZeros,
    );
    faults.retain(|fault| {
        fault.device != device_zero
            || fault.offset_in_bytes >= 2 * u64::from(FIXED_STRUCTURE_SLOT_SPACING)
    });
    let (outcome, _) = writable_mount_under_read_faults(&parameters, &history.image, &faults);
    assert!(
        matches!(
            outcome,
            Err(MountError::NewerStateStillUnreadableAfterOneReread(_))
        ),
        "盘 0 没落后时照判据 N-配置 拒；得到 {:?}",
        outcome.as_ref().map(|mounted| mounted.output.instance)
    );
}

/// 钉现状，条款不罩这一形（这一槽要有别的版本布局的池，第一版没有；「布局位不认识的槽不算坏」与「别的池的槽算坏」两句的优先次序条款没定）：
/// C971 的根与记录读坏（C、D），每块盘最新那一槽位置换成别的池的、布局位不认识的槽（整槽校验和过）。实现照「布局位不认识不算坏」不并进乙入判不出，
/// 可写挂载做成，新实例第一次发布落到被藏的 D 之下。
#[test]
fn a_slot_of_another_pool_with_unknown_incompat_bits_lets_the_inverted_first_txg_through() {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let history = minimal_history_of_c971(&parameters);
    let mut image = history.image.clone();
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let newest = newest_system_configuration_slot(&image, device);
        let generation = generation_at(&image, device, newest);
        let mut slot = slot_of_another_pool(&image, device, generation);
        slot[INCOMPAT_BITMAP_OFFSET_IN_THE_SYSTEM_CONFIGURATION_SLOT] ^= 0x80;
        let digest = wide_checksum_with_field_zeroed(
            &slot,
            slot_bytes,
            SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
        );
        slot[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32]
            .copy_from_slice(&digest);
        write_slot(&mut image, device, newest, &slot);
        assert_eq!(
            slot_counts_of(&image, device),
            (0, 1, 0),
            "前提：这一槽不进任何一个计数"
        );
    }
    let mut faults = read_faults_of_a_six_cell(
        &parameters,
        &history,
        ReadFaultKind::ReadsBackAllZeros,
        ReadFaultKind::ReadsBackAllZeros,
    );
    faults.retain(|fault| fault.offset_in_bytes >= 2 * u64::from(FIXED_STRUCTURE_SLOT_SPACING));
    let (outcome, _) = writable_mount_under_read_faults(&parameters, &image, &faults);
    let mounted = outcome.expect("可写挂载做成");
    assert!(
        mounted.output.row_publish.root().checkpoint_txg.0 <= TXG_OF_THE_NEWEST_VERSION,
        "新实例第一次发布落到被藏的 D 之下"
    );
}

/// 一段字节只在第 `earliest_bad_read_number` 到 `latest_bad_read_number` 次读（从 1 数）读回全零的盘：外面套在带常驻读故障的盘上。写到这一段就撤。
struct DeviceWithOneRangeBadOnChosenReads {
    inner: DeviceWithReadFaults,
    range: Option<(u64, u64)>,
    earliest_bad_read_number: u64,
    latest_bad_read_number: u64,
    reads_of_the_range: Rc<Cell<u64>>,
}

impl DeviceWithOneRangeBadOnChosenReads {
    fn lift_when_written_over(&mut self, offset_in_bytes: u64, length_in_bytes: u64) {
        if let Some((start, length)) = self.range {
            if offset_in_bytes < start + length && start < offset_in_bytes + length_in_bytes {
                self.range = None;
            }
        }
    }
}

impl BlockDevice for DeviceWithOneRangeBadOnChosenReads {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.inner.read_at(offset, buffer)?;
        let buffer_length = u64::try_from(buffer.len()).expect("读长装得进 u64");
        if let Some((start, length)) = self.range {
            if offset.0 < start + length && start < offset.0 + buffer_length {
                let read_number = self.reads_of_the_range.get() + 1;
                self.reads_of_the_range.set(read_number);
                if self.earliest_bad_read_number <= read_number
                    && read_number <= self.latest_bad_read_number
                {
                    let from = usize::try_from(start.saturating_sub(offset.0)).expect("缓冲内偏移");
                    let to =
                        usize::try_from((start + length).min(offset.0 + buffer_length) - offset.0)
                            .expect("缓冲内偏移");
                    buffer[from..to].fill(0);
                }
            }
        }
        Ok(())
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.lift_when_written_over(
            offset.0,
            u64::try_from(bytes.len()).expect("写长装得进 u64"),
        );
        self.inner.write_at(offset, bytes, durability)
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.lift_when_written_over(offset.0, length);
        self.inner.write_zeroes_at(offset, length)
    }

    fn discard_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<DiscardOutcome, BlockDeviceError> {
        self.lift_when_written_over(offset.0, length);
        self.inner.discard_at(offset, length)
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.inner.barrier()
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }

    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }

    fn journal_record_offsets_hint(
        &self,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        self.inner
            .journal_record_offsets_hint(ring_start, ring_bytes)
    }
}

/// D 的根自证过、D 的记录两份都读回全零（不带瞬时读错）：新实例第一次发布越过 D（txg 7），环里 D 这条根把 max 抬上去。
#[test]
fn root_read_with_its_records_unreadable_takes_the_first_txg_past_it() {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let history = minimal_history_of_c971(&parameters);
    let faults = faults_on_the_records_of_the_newest_version(&history);
    let (outcome, _) = writable_mount_under_read_faults(&parameters, &history.image, &faults);
    let mounted = outcome.expect("可写挂载做成");
    assert_eq!(
        (
            mounted.output.effective_root.checkpoint_txg.0,
            mounted.output.row_publish.root().checkpoint_txg.0
        ),
        (TXG_OF_THE_NEWEST_VERSION, TXG_OF_THE_NEWEST_VERSION + 1),
        "落到 D，写行那次发布越过 D"
    );
}

/// D23（journal 的角色与格式） 已定项 14 第 3 条：盘面同上一条（D 的记录两份都读回全零，崩在 D 的根槽之后、轮换之前），D 的根槽在这次挂载的第几次读读回全零都扫一遍
/// （只这一次 / 从这一次起一直；读阶段读到 D、取第一个 txg 那一遍直接读盘读坏的那几格都在里面，第几次读落在那一遍里不写死）。
/// 每一格可写挂载做成、写行 txg 大于所选那一版、池级 checker 无违例、第二次可写挂载做成；每格打一行 `FIRST_TXG_CELL`。
#[test]
fn root_read_bad_only_after_the_read_stage_still_numbers_the_new_instance_above_the_selected_version(
) {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let history = minimal_history_of_c971(&parameters);
    let faults = faults_on_the_records_of_the_newest_version(&history);
    let target = target_for_publish(
        CheckpointTxg(TXG_OF_THE_NEWEST_VERSION),
        parameters.geometry.root_ring_slots_per_region,
    );
    let root_device = parameters.region_devices[usize::try_from(target.region).expect("区域号")];
    let root_offset = slot_offset(target, parameters.geometry.fixed_structure_slot_spacing).0;
    let mut cells_selecting_the_newest_version = 0u64;
    for only_this_read in [true, false] {
        for earliest_bad_read_number in 1..=16u64 {
            let latest_bad_read_number = if only_this_read {
                earliest_bad_read_number
            } else {
                u64::MAX
            };
            let faults_lifted = Rc::new(Cell::new(false));
            let reads_of_the_range = Rc::new(Cell::new(0u64));
            let mut devices: Vec<(DeviceIdentity, DeviceWithOneRangeBadOnChosenReads)> =
                devices_with_read_faults(&history.image, &faults, &faults_lifted)
                    .into_iter()
                    .map(|(identity, inner)| {
                        (
                            identity,
                            DeviceWithOneRangeBadOnChosenReads {
                                inner,
                                range: (identity == root_device)
                                    .then_some((root_offset, ROOT_SLOT_BYTES)),
                                earliest_bad_read_number,
                                latest_bad_read_number,
                                reads_of_the_range: Rc::clone(&reads_of_the_range),
                            },
                        )
                    })
                    .collect();
            let mounted = mount_writable(&parameters, &mut devices).unwrap_or_else(|refusal| {
                panic!("只这一次 {only_this_read} 第 {earliest_bad_read_number} 次读：可写挂载做成；得到 {refusal:?}")
            });
            let image_after = memory_pool_of_sparse_devices(&devices, |device| &device.inner.inner);
            let effective = mounted.output.effective_root.checkpoint_txg.0;
            let row = mounted.output.row_publish.root().checkpoint_txg.0;
            let violated = violated_invariants(&image_after);
            let (second_mount, _) =
                writable_mount_under_read_faults(&parameters, &image_after, &[]);
            println!(
                "FIRST_TXG_CELL only_this_read={only_this_read} earliest_bad_read_number={earliest_bad_read_number} reads={} effective={effective} row={row} violated={violated:?} second_mount_ok={}",
                reads_of_the_range.get(),
                second_mount.is_ok()
            );
            assert!(
                row > effective,
                "只这一次 {only_this_read} 第 {earliest_bad_read_number} 次读：写行 txg {row} 大于所选那一版 {effective}"
            );
            assert_eq!(
                violated,
                Vec::<&'static str>::new(),
                "只这一次 {only_this_read} 第 {earliest_bad_read_number} 次读：池级 checker 无违例"
            );
            assert!(
                second_mount.is_ok(),
                "只这一次 {only_this_read} 第 {earliest_bad_read_number} 次读：第二次可写挂载做成"
            );
            if effective == TXG_OF_THE_NEWEST_VERSION {
                cells_selecting_the_newest_version += 1;
            }
        }
    }
    assert!(
        cells_selecting_the_newest_version > 0,
        "前提：扫到了读阶段读到 D 的那几格"
    );
}
