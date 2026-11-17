
/// 攻方腿的盘面（m3-close-b20-code-r2）：只进副本，不进主工作区。每一条写明它钉的是哪一个读数，读数在断言文字里。
mod attack_cases_of_the_second_round {
    use super::*;
    use singlefs_core::checksum::wide_checksum_with_field_zeroed;
    use singlefs_core::mount::{
        NewerPublishWitness, RollbackTarget, SelectedVersionAgainstTheWitness,
    };
    use singlefs_core::mounted_read::MountReadOnlyFailure;
    use singlefs_core::recovery::RecoveryFailure;
    use singlefs_core::system_configuration::SYSTEM_CONFIGURATION_CHECKSUM_OFFSET;
    use singlefs_core::system_configuration_repair::{
        rewrite_system_configuration_slots_for_the_asserted_version, AssertedVersion,
    };
    use singlefs_verification::mount_transparency::RefusalConditionNotMet;

    const FORMAT_VERSION_OFFSET: usize = 4;
    const FIRST_INCOMPAT_BYTE_OFFSET: usize = 6;
    const NODE_BYTES_OFFSET: usize = 301;

    fn slot_bytes_at(image: &MemoryPool, device: DeviceIdentity, offset: u64) -> Vec<u8> {
        let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
        PoolReader::read(image, device, DeviceOffsetInBytes(offset), slot_bytes)
            .expect("系统配置槽读得到")
    }

    fn generation_at(image: &MemoryPool, device: DeviceIdentity, offset: u64) -> u64 {
        SystemConfiguration::parse_slot(&slot_bytes_at(image, device, offset))
            .expect("这一槽本池自证过")
            .quantities
            .slot_generation
    }

    /// 同一盘一份别的池的系统配置：fsid 改一个字节，世代号给 `generation`，整槽校验和由 `to_slot` 封。
    fn slot_of_another_pool(image: &MemoryPool, device: DeviceIdentity, generation: u64) -> Vec<u8> {
        let offset = newest_system_configuration_slot(image, device);
        let mut another_pools =
            SystemConfiguration::parse_slot(&slot_bytes_at(image, device, offset))
                .expect("这一槽本池自证过");
        another_pools.immutable.filesystem_identifier[15] ^= 0xff;
        another_pools.quantities.slot_generation = generation;
        another_pools.to_slot()
    }

    fn write_slot(image: &mut MemoryPool, device: DeviceIdentity, offset: u64, bytes: &[u8]) {
        image
            .devices
            .get_mut(&device)
            .expect("有这块盘")
            .write(DeviceOffsetInBytes(offset), bytes);
    }

    fn resealed(mut slot: Vec<u8>, change: impl Fn(&mut [u8])) -> Vec<u8> {
        change(&mut slot);
        let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
        let digest = wide_checksum_with_field_zeroed(
            &slot,
            slot_bytes,
            SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
        );
        slot[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32]
            .copy_from_slice(&digest);
        slot
    }

    /// `devices` 那几块盘较旧那一槽的位置放一份世代号比本池最新那一槽大 100 的别的池的槽（择系统配置不筛 fsid 时这块盘择到它）。
    fn minimal_history_with_a_newest_generation_of_another_pool_on(
        parameters: &MakeFilesystemParameters,
        devices: &[DeviceIdentity],
    ) -> MemoryPool {
        let mut image = minimal_history_of_c971(parameters).image;
        for device in devices {
            let newest = newest_system_configuration_slot(&image, *device);
            let generation = generation_at(&image, *device, newest) + 100;
            let older = older_system_configuration_slot(&image, *device);
            let bytes = slot_of_another_pool(&image, *device, generation);
            write_slot(&mut image, *device, older, &bytes);
        }
        image
    }

    /// 判据 N-配置 的一拒（判官自己在镜像上复算，不看这里填的读数）。
    fn refusal_claimed_on_the_newest_version() -> MountError {
        let reading = SelectedVersionAgainstTheWitness {
            selected_version: RollbackTarget {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(TXG_OF_THE_NEWEST_VERSION),
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

    fn verdict_of_a_claimed_refusal_on(image: &MemoryPool) -> WritableMountRefusalVerdict {
        let fields = read_recovery_fields(image, ComparedVersion::TheRecoveredVersion);
        judge_writable_mount_refusal(
            image,
            fields.report.effective_root,
            &refusal_claimed_on_the_newest_version(),
            DiskAfterTheRefusal::ByteForByteUnchanged,
            ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
        )
    }

    fn says_the_criterion_is_false(verdict: &WritableMountRefusalVerdict) -> bool {
        matches!(
            verdict,
            WritableMountRefusalVerdict::Illegal { conditions_not_met, .. }
                if conditions_not_met.contains(
                    &RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage
                )
        )
    }

    /// 只读出路：每块盘世代号最大的那一槽是别的池的。可写挂载乙入判不出拒；只读挂载不筛 fsid，择到别的池那一份、一条根都读不到。离线出路走得通。
    #[test]
    fn newest_generation_of_another_pool_on_every_device_leaves_neither_writable_nor_read_only() {
        let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
        let image = minimal_history_with_a_newest_generation_of_another_pool_on(
            &parameters,
            &[DeviceIdentity(0), DeviceIdentity(1)],
        );
        for device in [DeviceIdentity(0), DeviceIdentity(1)] {
            assert_eq!(slot_counts_of(&image, device), (0, 1, 1), "前提：一槽本池、一槽别池");
        }
        let (outcome, image_after) = writable_mount_under_read_faults(&parameters, &image, &[]);
        assert!(
            refused_as_failing_self_verification_on_both_reads(&outcome),
            "读数：可写挂载乙入判不出拒"
        );
        assert!(image_after == image, "拒在取号之前");
        assert_eq!(
            mount_read_only(&image).err(),
            Some(MountReadOnlyFailure::Recovery(RecoveryFailure::NoValidRoot)),
            "读数：只读挂载择到别的池那一份、一条根都读不到"
        );
        let faults_lifted = Rc::new(Cell::new(false));
        let mut devices = devices_with_read_faults(&image, &[], &faults_lifted);
        let rewritten = rewrite_system_configuration_slots_for_the_asserted_version(
            &parameters,
            &mut devices,
            AssertedVersion {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(TXG_OF_THE_NEWEST_VERSION),
            },
        )
        .expect("读数：离线出路照断言 D 重写别的池的那两槽");
        assert_eq!(rewritten.slots_rewritten.len(), 2, "每块盘重写一槽");
        let repaired = memory_pool_of_sparse_devices(&devices, |device| &device.inner);
        let (after_repair, _) = writable_mount_under_read_faults(&parameters, &repaired, &[]);
        assert_eq!(
            after_repair.as_ref().map(|mounted| mounted.output.instance).ok(),
            Some(InstanceGeneration(2)),
            "读数：离线出路之后可写挂载做成"
        );
    }

    /// 对照：别的池的槽世代号小（实现员用例那一形），只读挂载照常。
    #[test]
    fn older_generation_of_another_pool_on_every_device_still_mounts_read_only() {
        let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
        let image = minimal_history_with_the_older_system_configuration_slot_from_another_pool_on(
            &parameters,
            &[DeviceIdentity(0), DeviceIdentity(1)],
        );
        assert!(mount_read_only(&image).is_ok(), "读数：择到本池那一份，只读挂载做成");
    }

    /// 判官把合法的拒判成不合法：盘 0 两槽都是别的池的，盘 1 较旧位置是世代号更大的别的池的槽。实现按本池 fsid 判乙入判不出、拒；
    /// 判官不筛 fsid，两块盘都择到别的池那一份，拿它当本池：盘 0 两槽「都自证过」，判据复算为假。
    #[test]
    fn the_judge_calls_a_legal_refusal_illegal_when_it_chooses_the_other_pool() {
        let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
        let mut image = minimal_history_with_a_newest_generation_of_another_pool_on(
            &parameters,
            &[DeviceIdentity(1)],
        );
        let device_zero = DeviceIdentity(0);
        let newest = newest_system_configuration_slot(&image, device_zero);
        let older = older_system_configuration_slot(&image, device_zero);
        let generation = generation_at(&image, device_zero, newest) + 200;
        let older_slot_of_another_pool = slot_of_another_pool(&image, device_zero, generation);
        let newest_slot_of_another_pool = slot_of_another_pool(&image, device_zero, generation + 1);
        write_slot(&mut image, device_zero, older, &older_slot_of_another_pool);
        write_slot(&mut image, device_zero, newest, &newest_slot_of_another_pool);
        assert_eq!(slot_counts_of(&image, device_zero), (0, 0, 2), "前提：盘 0 两槽都是别的池的");
        assert_eq!(slot_counts_of(&image, DeviceIdentity(1)), (0, 1, 1), "前提：盘 1 一槽本池、一槽别池");
        let (outcome, image_after) = writable_mount_under_read_faults(&parameters, &image, &[]);
        assert!(
            refused_as_failing_self_verification_on_both_reads(&outcome),
            "读数：可写挂载乙入判不出拒"
        );
        assert!(image_after == image, "拒在取号之前");
        let verdict = verdict_of_the_refusal_on(&image, &outcome);
        assert!(
            says_the_criterion_is_false(&verdict),
            "读数：判官判这一拒判据复算为假；得到 {verdict:?}"
        );
    }

    /// 判官放过不该拒的拒：一块盘世代号最大的那一槽是别的池的（单故障），实现照常做成；同一份盘面上喂一个声称的 N-配置 之拒，
    /// 判官择系统配置报各盘 fsid 对不上、判据按真，判合法。对照盘面（别的池的槽世代号小）判官照判不合法。
    #[test]
    fn the_judge_cannot_see_a_wrong_refusal_when_one_device_chooses_the_other_pool() {
        let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
        let image = minimal_history_with_a_newest_generation_of_another_pool_on(
            &parameters,
            &[DeviceIdentity(1)],
        );
        let (outcome, _) = writable_mount_under_read_faults(&parameters, &image, &[]);
        assert_eq!(
            outcome.as_ref().map(|mounted| mounted.output.instance).ok(),
            Some(InstanceGeneration(2)),
            "读数：单故障，可写挂载做成"
        );
        assert_eq!(
            verdict_of_a_claimed_refusal_on(&image),
            WritableMountRefusalVerdict::Legal(
                RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration
            ),
            "读数：判官判一次不该有的拒合法"
        );
        let control = minimal_history_with_the_older_system_configuration_slot_from_another_pool_on(
            &parameters,
            &[DeviceIdentity(1)],
        );
        let verdict = verdict_of_a_claimed_refusal_on(&control);
        assert!(
            says_the_criterion_is_false(&verdict),
            "读数：对照盘面上判官判不合法；得到 {verdict:?}"
        );
    }

    /// 判官与实现查一槽的次序不同：格式版本不认又节点大小不对、整槽校验和过的槽。实现（`parse_slot` 不看格式版本，节点大小不对当自证不过）记读坏、乙入判不出拒；
    /// 判官（`check_system_configuration_slot` 先报格式版本）不算坏，判据复算为假。
    #[test]
    fn the_judge_and_the_product_disagree_on_a_slot_with_an_unknown_format_version_and_wrong_node_bytes(
    ) {
        let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
        let mut image = minimal_history_of_c971(&parameters).image;
        for device in [DeviceIdentity(0), DeviceIdentity(1)] {
            let older = older_system_configuration_slot(&image, device);
            let bytes = resealed(slot_bytes_at(&image, device, older), |slot| {
                slot[FORMAT_VERSION_OFFSET] ^= 0x40;
                slot[NODE_BYTES_OFFSET] ^= 0x01;
            });
            write_slot(&mut image, device, older, &bytes);
        }
        let (outcome, _) = writable_mount_under_read_faults(&parameters, &image, &[]);
        assert!(
            refused_as_failing_self_verification_on_both_reads(&outcome),
            "读数：可写挂载乙入判不出拒"
        );
        let verdict = verdict_of_the_refusal_on(&image, &outcome);
        assert!(
            says_the_criterion_is_false(&verdict),
            "读数：判官判这一拒判据复算为假；得到 {verdict:?}"
        );
    }

    /// C971 的根与记录读坏（C、D），每块盘最新那一槽位置换成别的池的、布局位不认识的槽（整槽校验和过）：登记两句相撞，实现照「布局不认不算坏」不并进乙，
    /// 可写挂载做成，新实例第一次发布落到被藏的 D 之下。
    #[test]
    fn a_slot_of_another_pool_with_unknown_incompat_bits_lets_the_inverted_first_txg_through() {
        let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
        let history = minimal_history_of_c971(&parameters);
        let mut image = history.image.clone();
        for device in [DeviceIdentity(0), DeviceIdentity(1)] {
            let newest = newest_system_configuration_slot(&image, device);
            let generation = generation_at(&image, device, newest);
            let bytes = resealed(slot_of_another_pool(&image, device, generation), |slot| {
                slot[FIRST_INCOMPAT_BYTE_OFFSET] ^= 0x80;
            });
            write_slot(&mut image, device, newest, &bytes);
            assert_eq!(slot_counts_of(&image, device), (0, 1, 0), "前提：这一槽不进任何一个计数");
        }
        let mut faults = read_faults_of_a_six_cell(
            &parameters,
            &history,
            ReadFaultKind::ReadsBackAllZeros,
            ReadFaultKind::ReadsBackAllZeros,
        );
        faults.retain(|fault| fault.offset_in_bytes >= 2 * u64::from(FIXED_STRUCTURE_SLOT_SPACING));
        let (outcome, _) = writable_mount_under_read_faults(&parameters, &image, &faults);
        let mounted = outcome.expect("读数：可写挂载做成");
        assert!(
            mounted.output.row_publish.root().checkpoint_txg.0 <= TXG_OF_THE_NEWEST_VERSION,
            "读数：新实例第一次发布落到被藏的 D 之下"
        );
    }

    /// C971 的根与记录读坏（C、D），盘 1 最新那一槽读回全 0，盘 0 没收到那次轮换（两槽都自证过、见证值都不超过 B）。「没有一块盘两槽都自证过」不成立，
    /// 乙入判不出不判，c_见证 落回 B，可写挂载做成，新实例第一次发布落到被藏的 D 之下：比六格少坏一槽。
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
        assert_eq!(slot_counts_of(&image, device_zero), (0, 2, 0), "前提：盘 0 两槽都自证过");
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
        let mounted = outcome.expect("读数：可写挂载做成");
        assert!(
            mounted.output.row_publish.root().checkpoint_txg.0 <= TXG_OF_THE_NEWEST_VERSION,
            "读数：新实例第一次发布落到被藏的 D 之下"
        );
    }

    /// 对照：同一组读坏，盘 0 没落后（最新那一槽见证到 C）。c_见证 比所选那一版 B 新，判据 N-配置 比出来为真，拒。
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
            "读数：盘 0 没落后时照判据 N-配置 拒；得到 {:?}",
            outcome.as_ref().map(|mounted| mounted.output.instance)
        );
    }
}
