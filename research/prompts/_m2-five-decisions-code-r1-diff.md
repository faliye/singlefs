# 附录二：五条定案那一批代码改动（按「文件::项名」整段抽，加四个新测试文件全文与变异表新加的 6 行；2026-09-28）

基准：冻结副本 `/tmp/claude-1000/m2-five-decisions-code-r1/tree/crates/`，逐文件 sha256 在 `research/prompts/m2-five-decisions-code-r1-snapshot/crates-sha256.txt`（184 行；生成这份附录时在冻结副本根上跑 `sha256sum -c` 184 行全部 OK）。仓 HEAD 是 `e5253e8a`。生成日期 2026-09-28。

工作区里别的会话也有没提交的改动（crates 已被整个暂存），给不出只含这一轮的 `git diff`，所以按正文第二节列的项整段抽，不是 diff。行号是冻结副本里那份文件自己的行号。

抽法：代码项用 `research/scripts/quote-rust-items.py 文件::项名`（在冻结副本根上跑，每段前自带文件名与行区间）；新测试文件、变异表按行区间整段取。全部段落拼好之后逐段与冻结副本那几行逐字节比过。例外一处：`crates/singlefs-core/src/recovery.rs` 的 `effective_rollback_floor_of_the_roots_read`，`quote-rust-items.py` 在签名里 `[DeviceIdentity; 3]` 的分号处截断（只交出 1685-1689、退出码 0），改按行区间取，那一段的标题里写明。

项怎么对上正文第二节：

- 「新成员的匹配臂」（`history.rs`、`model_comparison.rs`）：正文没给函数名，取的是冻结副本里写着 `StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot` 的那三个函数（`history.rs` 一处、`model_comparison.rs` 两处；`grep` 这两份文件，另外两个新名字 `SomeSystemConfigurationSlotUnreadOrUnverified`、`DeviceWithAnUnreadOrUnverifiedSystemConfigurationSlotWhenReadingTheWitness` 一处都不出现）。
- 改钉的用例：正文只写了文件与「那一条」，函数名是拿冻结副本与 HEAD 的同名文件比、按正文的描述对上的。同一份文件里 HEAD 之后还改了别的用例（例：`rollback_by_a_forward_publish.rs` 的 `rolling_back_while_a_root_slot_this_process_wrote_reads_not_self_verified_is_refused_before_any_write`、`corrupt_on_disk_content_is_refused_instead_of_panicking.rs` 的 `a_tree_table_entry_at_or_above_the_tree_identifier_watermark_is_refused_by_the_writable_mount`），说的是别的轮次的事，没取。`entries_after_mount_refuse_swapped_or_behind_devices.rs` 在 HEAD 里没有（新暂存的文件），按正文「只剩一槽那一条」取那一个函数。
- 随列出的项一起取的辅助函数：`tests/common/mod.rs` 的 `system_configuration_slots_of_every_device`（`abandon_the_third_version_by_a_crash_before_its_rotation` 的文档注释点名它，HEAD 里没有）；`acquisition_writes_…` 里两条「落后一次轮换」用例与两条「丢最新槽」用例各自调的辅助函数；`fsync_drop_…` 里「见证拒因」那个断言函数 `assert_refused_by_the_witness_of_a_newer_publish`。`acquisition_writes_…` 的 `a_rolled_back_acquisition_writes_the_witnessed_tail_back_instead_of_zero` 只改了取号拒因成员的名字，也取了。
- 变异表：新加的 6 行按正文给的名字起头取。正文说的「改锚点 8 行」没有名字，冻结副本与 HEAD 的差里混着别的会话加的几十行，分不出是哪 8 行，没取；要看就读冻结副本的 `crates/mutations.tsv`。
- 没取：取号拒因改名在别的测试文件里的机械改名（`instance_acquisition.rs`、`acquisition_refuses_a_device_whose_witness_slots_turn_unreadable_and_the_remaining_writer_panics.rs`、`system_configuration_slot_is_overwritten_only_after_a_barrier.rs`），正文第二节没列；Z1 问到的 `SelectedVersionLackingOnDevice` 在 `crates/singlefs-core/src/mount.rs`，正文第二节没列，没取。

这份附录不并进背景材料，各条腿按正文里写的路径读冻结副本。

## 一、实现（正文第二节 `mount.rs`、`transaction.rs`、`recovery.rs`、`walk.rs`、`history.rs`、`model_comparison.rs` 那几条）

### crates/singlefs-core/src/mount.rs:417-435（WitnessedCounterComparison）

```rust
/// 判据 N-配置 拿 c_见证 跟什么比，按次序判（E158 第 3 次跑登记第 347 行）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WitnessedCounterComparison {
    /// c_见证 = 0（没有自证过的槽，或读得出的槽都是 mkfs 与 mkfs 之后第一次取号写的 0）：判据为假。
    NothingWitnessed,
    /// 所选那一版那次发布带「本次发布末条」标志的记录读得出（任一份），计数器 c_E：判据 = c_见证 > c_E。
    AgainstTheSelectedVersionsLastRecord {
        selected_version_last_record_counter: u64,
    },
    /// 所选那一版的末条读不出、计数器等于 c_见证 的记录读得出：判据 = 它的 (实例代号, checkpoint_txg) > 所选那一版的。
    /// 读得出几条（不同实例的记录落回过同一个计数器）时取键最大的那一条。
    AgainstTheRecordAtTheWitnessedCounter { record: RollbackTarget },
    /// 两条都读不出：判不出，按判据为真处置（登记里的标签 `undecidable`）。
    Undecidable,
    /// 见证读里有一个系统配置槽读不出或自证不过：c_见证 缺了那一槽，判不出，按判据为真处置
    /// （C331（择根倒挂压过已确认的写） 取甲，用户 2026-09-28 定；`research/prompts/unreadable-at-mount-r2-main-verification.md` L2）。
    /// 撕裂的系统配置轮换也落在这里：代价是那之后只剩只读挂载（同一判决 L1）。
    SomeSystemConfigurationSlotUnreadOrUnverified,
}
```

### crates/singlefs-core/src/mount.rs:387-405（witnesses_a_publish_newer_than_the_selected_version）

```rust
    /// 判据 N-配置 为真：系统配置见证过一次比所选那一版新的发布（判不出的也按真）。
    #[must_use]
    pub fn witnesses_a_publish_newer_than_the_selected_version(&self) -> bool {
        match self.witness.comparison {
            WitnessedCounterComparison::NothingWitnessed => false,
            WitnessedCounterComparison::AgainstTheSelectedVersionsLastRecord {
                selected_version_last_record_counter,
            } => self.witness.witnessed_journal_counter > selected_version_last_record_counter,
            WitnessedCounterComparison::AgainstTheRecordAtTheWitnessedCounter { record } => {
                (record.instance, record.checkpoint_txg)
                    > (
                        self.selected_version.instance,
                        self.selected_version.checkpoint_txg,
                    )
            }
            WitnessedCounterComparison::Undecidable
            | WitnessedCounterComparison::SomeSystemConfigurationSlotUnreadOrUnverified => true,
        }
    }
```

### crates/singlefs-core/src/mount.rs:4245-4320（newer_publish_witness）

```rust
/// 判据 N-配置（E158 第 3 次跑登记第 347 行）的读数：系统配置槽直接读盘（`verified_system_configuration_slots`，每块盘两槽），
/// 不经读缓存（第 4 次跑登记 5.1 第 1 条：重读那一遍也直接从盘上重读这几槽）；记录取读阶段那一遍扫出来的。
fn newer_publish_witness<Reader: PoolReader + ?Sized>(
    devices: &Reader,
    system_configuration: &crate::system_configuration::SystemConfiguration,
    selected_version: &RootRecord,
    records: &BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
) -> NewerPublishWitness {
    let slot_spacing_in_bytes = u64::from(
        system_configuration
            .immutable
            .sizes
            .fixed_structure_slot_spacing,
    );
    let verified_slots_of_each_device: Vec<Vec<crate::system_configuration::SystemConfiguration>> =
        devices
            .device_identities()
            .into_iter()
            .map(|device| {
                verified_system_configuration_slots(
                    devices,
                    device,
                    slot_spacing_in_bytes,
                    &system_configuration.immutable.filesystem_identifier,
                )
            })
            .collect();
    let witnessed_journal_counter = verified_slots_of_each_device
        .iter()
        .flatten()
        .map(|slot| slot.quantities.journal_tail)
        .max()
        .unwrap_or(0);
    let some_system_configuration_slot_unread = verified_slots_of_each_device.iter().any(
        |verified_slots_of_this_device| {
            u64::try_from(verified_slots_of_this_device.len())
                .expect("每块盘的系统配置槽数装得进 u64")
                < SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE
        },
    );
    let comparison = if some_system_configuration_slot_unread {
        WitnessedCounterComparison::SomeSystemConfigurationSlotUnreadOrUnverified
    } else if witnessed_journal_counter == 0 {
        WitnessedCounterComparison::NothingWitnessed
    } else if let Some(last_record_of_the_selected_version) = records
        .values()
        .filter(|record| {
            record.instance == selected_version.instance
                && record.checkpoint_txg == selected_version.checkpoint_txg
                && record.place_in_publish
                    == crate::journal::JournalRecordPlaceInPublish::LastRecordOfThePublish
        })
        .max_by_key(|record| record.counter)
    {
        WitnessedCounterComparison::AgainstTheSelectedVersionsLastRecord {
            selected_version_last_record_counter: last_record_of_the_selected_version.counter,
        }
    } else if let Some(record_at_the_witnessed_counter) = records
        .values()
        .filter(|record| record.counter == witnessed_journal_counter)
        .max_by_key(|record| (record.instance, record.checkpoint_txg))
    {
        WitnessedCounterComparison::AgainstTheRecordAtTheWitnessedCounter {
            record: RollbackTarget {
                instance: record_at_the_witnessed_counter.instance,
                checkpoint_txg: record_at_the_witnessed_counter.checkpoint_txg,
            },
        }
    } else {
        WitnessedCounterComparison::Undecidable
    };
    NewerPublishWitness {
        witnessed_journal_counter,
        comparison,
    }
}
```

### crates/singlefs-core/src/mount.rs:301-339（StillUnreadableAfterOneReread）

```rust
/// 可写挂载重读一次之后仍读不出的是哪一样（[`MountError::NewerStateStillUnreadableAfterOneReread`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StillUnreadableAfterOneReread {
    /// 判据 N-配置（定义取自 E158 第 3 次跑登记 `research/prompts/e158-r3-prereg.md` 第 347 行）：系统配置见证过一次比所选那一版新的发布——
    /// 系统配置在发布的根槽 FUA 之后才轮换（D16（发布语义） 已定项 7），见证到的发布它的根落过盘——而读阶段（择根、扫 journal、重放）
    /// 交出的所选那一版比它旧；在这次挂载的读缓存上重做读阶段一遍（[`ReadStageCache`]），仍判真。
    PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
        first_read: SelectedVersionAgainstTheWitness,
        reread: SelectedVersionAgainstTheWitness,
    },
    /// 代码审阅第 22 条：重建分配器时按最新那条根的实例表判哪几条根被抛弃（影子账、回收的门槛、根环表都按它），那张表读不出，
    /// 重读一次仍读不出。不再按「没有一条根被抛弃」往下走（那样一个槽都不隔离、`abandoned_roots_unreadable` 仍是 0）。
    InstanceTableOfTheNewestRootForTheShadowLedger {
        /// 重读那一遍择到的最新那条根；根环那一遍一条自证过的根都没读到时 `None`。
        newest_root_on_the_reread: Option<RollbackTarget>,
    },
    /// 挂着时抬 F 算 F 生效值（`recovery::effective_rollback_floor_rereading_the_newest_instance_table_once`）或管理员回退判候选算
    /// F_生效（`recovery::effective_rollback_floor_rereading_known_ring_slots_and_the_newest_instance_table_once`）：根环里最新那条根
    /// 指着的实例表（按它判哪几条根被抛弃）读不出、解不开，或一条自证过的根都择不到，连根环一起重读一次仍是这样（C554 乙报告 Q6）。
    /// 不按「不按表滤」往下走（那样被抛弃时间线上的根带的 F 也算进生效值）。实审 A3b 报告 Q4：与乙那一族同级，改之前是
    /// `RecoveryFailure::InstanceTableOfTheNewestRootStillUnreadableAfterOneReread`。
    InstanceTableOfTheNewestRootForTheEffectiveFloor {
        /// 重读那一遍择到的最新那条根；一条自证过的根都没读到时 `None`。
        newest_root_on_the_reread: Option<RollbackTarget>,
    },
    /// 挂着时抬 F 重算影子账与回收门槛读根环（`recovery::readable_roots_rereading_ring_slots_known_to_hold_a_root_once`）或管理员回退
    /// 判候选、算 F_生效 读根环：`ring_slot` 是这个进程知道住着一条根的槽（挂载那一刻读得出、或这个进程写过且 FUA 返回过），这一次读坏
    /// （`first_reading`：读不出或自证不过），重读一次仍坏（`reread`）——那一槽里的根在不在判不了，不按有根或没根猜（D16（发布语义） 已定项 1
    /// 「根槽这一次读坏」那一行全句，用户 2026-09-26 定）。挂载那一刻就读坏、之后没写过的槽不走这一条，当没有根。
    /// 实审 A3b 报告 Q4：与乙那一族同级，改之前是 `RecoveryFailure::RootRingSlotStillUnreadableAfterOneReread`（那时不分槽知不知道住着根）。
    RootRingSlotKnownToHoldARoot {
        ring_slot: RootRingSlot,
        first_reading: BadRootRingSlotReading,
        reread: BadRootRingSlotReading,
    },
    /// 影子账读一条被抛弃根的账（树表或分配记录树）读不出、从中央映射也现算不成，重读一次仍是这样：拒可写
    /// （C393（被抛弃根的账读不出时修复没有条款） 取 (b)-从映射现算那一条的最后一支，用户 2026-09-28 定）。
    AccountOfAnAbandonedRoot { abandoned_root: RollbackTarget },
}
```

### crates/singlefs-core/src/mount.rs:1232-1325（isolate_slots_referenced_only_by_abandoned_roots）

```rust
fn isolate_slots_referenced_only_by_abandoned_roots<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    unit_area_start: UnitAreaStart,
    allocator: &mut PoolAllocator,
    roots: &[RootRecord],
    is_abandoned: &dyn Fn(&RootRecord) -> bool,
    floor: CheckpointTxg,
    current_records: &[AllocationRecord],
    before_the_one_reread: &mut BeforeTheOneReread<'_>,
) -> Result<ShadowLedgerComputed, MountError> {
    let mut referenced_by_candidates: BTreeSet<(u32, u64)> = current_records
        .iter()
        .filter(|record| !record.is_released)
        .map(|record| (record.device.0, record.slot.0))
        .collect();
    for root in roots
        .iter()
        .filter(|root| !is_abandoned(root) && root.checkpoint_txg >= floor)
    {
        if let Some(placements) = placements_referenced_by_root(devices, unit_area_start, root) {
            referenced_by_candidates.extend(
                placements
                    .iter()
                    .map(|(device, slot, _)| (device.0, slot.0)),
            );
        }
    }
    let mut isolated: BTreeSet<(u32, u64)> = BTreeSet::new();
    let mut unreadable = 0;
    let mut placements_referenced_by_abandoned_root = BTreeMap::new();
    for root in roots.iter().filter(|root| is_abandoned(root)) {
        // C393（被抛弃根的账读不出时修复没有条款） 取 (b)-从映射现算（用户 2026-09-28 定，被攻过零轮；
        // `research/prompts/unreadable-at-mount-r2-main-verification.md` L3）：账（树表或分配记录树）读不出时不跳过，
        // 从中央映射现算这条根引用的落点，只在内存里隔离、不写盘，计进 `abandoned_roots_unreadable` 往上报（显式报告）；
        // 现算也不成（映射树读不出、解不开）才重读一次（重读之前调钩子），账与现算都仍不成就在取号之前拒可写、只读照常。
        let placements = match placements_referenced_by_root(devices, unit_area_start, root) {
            Some(placements) => placements,
            None => {
                unreadable += 1;
                match placements_referenced_by_root_computed_from_the_mapping(
                    devices,
                    unit_area_start,
                    root,
                ) {
                    Some(computed) => computed,
                    None => {
                        before_the_one_reread.before_rereading();
                        match placements_referenced_by_root(devices, unit_area_start, root).or_else(
                            || {
                                placements_referenced_by_root_computed_from_the_mapping(
                                    devices,
                                    unit_area_start,
                                    root,
                                )
                            },
                        ) {
                            Some(placements) => placements,
                            None => {
                                return Err(MountError::NewerStateStillUnreadableAfterOneReread(
                                    Box::new(
                                        StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot {
                                            abandoned_root: RollbackTarget {
                                                instance: root.instance,
                                                checkpoint_txg: root.checkpoint_txg,
                                            },
                                        },
                                    ),
                                ))
                            }
                        }
                    }
                }
            }
        };
        for (device, slot, span_slots) in placements.iter().copied() {
            let key = (device.0, slot.0);
            if referenced_by_candidates.contains(&key) || !isolated.insert(key) {
                continue;
            }
            allocator.isolate_abandoned(device, slot, span_slots);
        }
        placements_referenced_by_abandoned_root.insert(
            (root.instance, root.checkpoint_txg),
            placements
                .into_iter()
                .map(|(device, slot, span)| PlacementOnDevice { device, slot, span })
                .collect(),
        );
    }
    Ok(ShadowLedgerComputed {
        abandoned_roots_unreadable: unreadable,
        placements_referenced_by_abandoned_root,
    })
}
```

### crates/singlefs-core/src/mount.rs:1336-1381（placements_referenced_by_root_computed_from_the_mapping）

```rust
fn placements_referenced_by_root_computed_from_the_mapping<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    unit_area_start: UnitAreaStart,
    root: &RootRecord,
) -> Option<Vec<(DeviceIdentity, crate::address::SlotNumber, u64)>> {
    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
    let mapping_tree = crate::code_two_tree::read_code_two_tree(
        &root.mapping_root,
        &crate::transaction::MultiLevelCodeTwoTree::CentralMapping
            .read_expectation(root.mapping_root.head.birth_tree),
        crate::code_two_tree::CodeTwoTreeHeaderJudgement::OnlyWhatTheShapeNeeds,
        root,
        crate::unit::unit_filesystem_identifier(&root.filesystem_identifier),
        &mut |pointer: &crate::pointer::NodePointer| {
            read_unit_via_locations(devices, &pointer.locations, node_bytes)
        },
    )
    .ok()?;
    let mut located: Vec<([LocationEntry; 2], u64)> = Vec::new();
    for page in instance_table_page_pointers_as_far_as_readable(devices, root) {
        located.push((page.locations, TransactionUnit::InstanceTable.span_slots()));
    }
    located.push((root.tree_table.locations, TransactionUnit::TreeTable.span_slots()));
    for pointer in &mapping_tree.version.pointers {
        located.push((pointer.locations, TransactionUnit::MappingTree.span_slots()));
    }
    for entry in &mapping_tree.leaf_entries_in_key_order {
        let (key, locations) = crate::records::parse_mapping_entry(entry)?;
        let span = match key.first().copied()? {
            crate::unit::UNIT_CLASS_DATA | crate::unit::UNIT_CLASS_PACKED => 2,
            crate::unit::UNIT_CLASS_INDEX_NODE => 1,
            _ => return None,
        };
        located.push((locations, span));
    }
    let mut placements = Vec::new();
    for (locations, span) in located {
        let slot = slot_shared_by_both_location_entries(&locations).ok()?;
        for (identity, _) in devices {
            placement_lies_in_the_unit_area_of_its_device(devices, unit_area_start, *identity, slot, span)
                .ok()?;
            placements.push((*identity, slot, span));
        }
    }
    Some(placements)
}
```

### crates/singlefs-core/src/mount.rs:1484-1601（rebuilt_allocator）

```rust
fn rebuilt_allocator<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    system_configuration: &crate::system_configuration::SystemConfiguration,
    previous: &PreviousVersion,
    first_txg: CheckpointTxg,
    shadow_ledger: ShadowLedger,
    before_the_one_reread: &mut BeforeTheOneReread<'_>,
) -> Result<RebuiltAllocator, MountError> {
    let unit_area_start = unit_area_start_of_the_chosen_system_configuration(system_configuration);
    // 盘够不够长到单元区起点，可写挂载在读根环之前判过（`every_device_reaches_the_unit_area_start`，同一个起点）。
    let device_maps: Vec<DeviceFreeMap> = devices
        .iter()
        .map(|(identity, device)| {
            DeviceFreeMap::with_unit_area_start(*identity, device.size_in_bytes(), unit_area_start)
        })
        .collect();
    let mut allocator = match previous {
        PreviousVersion::WithFile { output, .. } => {
            PoolAllocator::rebuild_from_records(device_maps, output.allocation_records.clone())
        }
        PreviousVersion::WithoutFile { root, .. } => {
            allocator_of_version_without_file(devices, unit_area_start, device_maps, root)?
        }
    };
    let current_records = allocator.records().to_vec();
    let (newest_table, instance_table_of_the_newest_root) =
        instance_table_of_the_newest_root_read_at_most_twice(
            devices,
            system_configuration,
            before_the_one_reread,
        )?;
    let effective_floor = effective_rollback_floor_under_the_newest_roots_table(
        devices,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
        &newest_table,
    );
    // 影子账与分配器那张根环表读的是同一遍根环：两遍之间的瞬时读错会让两边看见的根不一样。
    let roots_with_ring_slots = readable_roots_with_ring_slots(
        devices,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    );
    let roots: Vec<RootRecord> = roots_with_ring_slots
        .iter()
        .map(|(_, root)| *root)
        .collect();
    let is_abandoned = |root: &RootRecord| abandoned_by_table(root, &newest_table);
    let oldest_valid_root = roots
        .iter()
        .filter(|root| !is_abandoned(root))
        .map(|root| root.checkpoint_txg)
        .min();
    allocator.reclaim_released_up_to(
        reclaim_floor(effective_floor, oldest_valid_root),
        ReclaimedReuse::Immediately,
    );
    let shadow_ledger_computed = match shadow_ledger {
        ShadowLedger::On => isolate_slots_referenced_only_by_abandoned_roots(
            devices,
            unit_area_start,
            &mut allocator,
            &roots,
            &is_abandoned,
            effective_floor,
            &current_records,
            before_the_one_reread,
        )?,
        ShadowLedger::Off => ShadowLedgerComputed {
            abandoned_roots_unreadable: 0,
            placements_referenced_by_abandoned_root: BTreeMap::new(),
        },
    };
    let ShadowLedgerComputed {
        abandoned_roots_unreadable,
        placements_referenced_by_abandoned_root,
    } = shadow_ledger_computed;
    let occupants: Vec<(RootRingSlot, RootRingOccupant)> = roots_with_ring_slots
        .iter()
        .map(|(ring_slot, root)| {
            let occupant = if is_abandoned(root) {
                RootRingOccupant::AbandonedRoot {
                    referenced_placements: placements_referenced_by_abandoned_root
                        .get(&(root.instance, root.checkpoint_txg))
                        .cloned()
                        .unwrap_or_default(),
                }
            } else {
                RootRingOccupant::ValidRoot {
                    checkpoint_txg: root.checkpoint_txg,
                }
            };
            (*ring_slot, occupant)
        })
        .collect();
    allocator.install_root_ring_occupancy(RootRingOccupancy::read_from_the_ring(
        system_configuration
            .immutable
            .sizes
            .root_ring_slots_per_region,
        occupants,
        effective_floor,
        CheckpointTxg(
            first_txg
                .0
                .checked_sub(1)
                .expect("新实例第一次发布的 txg = 环里最大 txg + 1 ≥ 1"),
        ),
    ));
    Ok(RebuiltAllocator {
        allocator,
        effective_floor,
        abandoned_roots_unreadable,
        instance_table_of_the_newest_root,
    })
}
```

### crates/singlefs-core/src/mount.rs:2341-2600（raise_the_floor_through）

```rust
/// 抬 F 那一串（准入与卸载共用，D16（发布语义） 已定项 1「抬 F 那一串」）。两个入口只差 [`RaiseFloorEntry`] 那两样：
/// 判不判上限、根带不带卸载记号。
fn raise_the_floor_through<Device: BlockDevice>(
    entry: RaiseFloorEntry,
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    allocator: &mut PoolAllocator,
    current: &mut TransactionOutput,
    new_floor: CheckpointTxg,
    shadow_ledger: ShadowLedger,
) -> Result<FloorRaisedThroughTheSequence, MountError> {
    // 分配器上冻结着一次没重发的发布（D23（journal 的角色与格式） 已定项 14「这一版的失败处置」）：这一串的第一次空发布本来就会被拒，
    // 而下面的影子账与回收在发布之前就动分配器——重发成功时分配器整个换成那次发布之后的一份，这些改动会被一起丢掉。第一道就拒，一样都不动。
    if let Some(frozen) = allocator.frozen_publish() {
        return Err(MountError::RaiseFloorSequencePublishFailed(Box::new(
            PublishSequenceFailed {
                cause: PublishError::PublishFrozenAfterAWriteFailureIsNotResentYet {
                    checkpoint_txg: frozen.checkpoint_txg(),
                    instance: frozen.instance(),
                },
                writes_before_the_first_publish: WritesByStructureKind::NOTHING_WRITTEN,
                writes_of_persisted_publishes: Vec::new(),
                writes_of_failed_publishes: Vec::new(),
            },
        )));
    }
    // 下面的影子账重算与回收在第一次空发布之前就动分配器：第一次空发布在任何写之前被拒（落点、准入、释放判定），这一串一次都没落盘、
    // F 没有一条根带出去，分配器整个换回这一份——扣住的槽放开、回收的回到 defer、补的隔离撤掉（C546（抬 F 被拒时扣住的槽不退回））。
    let allocator_before_the_raise = allocator.clone();
    // 调用方交进来的参数与盘表照可写挂载同一套核，写入口只按盘上那一份建（实审 A1b Q2、Q3）：对不上在任何写之前拒，分配器还没动。
    let CallerInputsAgreeingWithTheDisk {
        system_configuration,
        parameters_of_the_pool,
    } = caller_inputs_agreeing_with_the_disk(parameters, devices, current)?;
    // 候选集按现行那一版的实例表判：从它的根指着的第 0 片沿链真读出来、解出来（C502（抬 F 时现行版本里没有实例表单元）；
    // D18（块里携带什么信息） 已定项 11：一张表可以不止一片）。不看 `TransactionOutput::units`——那是这个进程内存里的角色列表，
    // 第一个文件版本那一版起就不带实例表单元，mkfs 同一个进程里后面发多少次都一样，而表就在根指针后面、读得出也解得开。
    let table = instance_table_chain_of_root(&*devices, &current.root)
        .map_err(|_unreadable_or_malformed| MountError::InstanceTableMalformed)?
        .records;
    // F 的生效值只增不减（「抬 F 那一串」那一行）：新 F 低于盘上现算的生效值，往下「抬」条款没写，在任何写之前拒。
    // 先写系统配置那一步写之前按盘上现状再算一次、取两者的大者（`transaction::RollbackFloorOfASystemConfigurationWrite::RaisedFloor`）。
    // 判「有效根」的那张表（根环里最新那条根的实例表）读不出就重读一次，仍读不出就拒这一次抬，不按「不按表滤」算（C554 乙报告 Q6）：
    // 在动分配器与任何写之前返回，分配器还没动。
    let effective_floor = effective_rollback_floor_rereading_the_newest_instance_table_once(
        &**devices,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    )
    .map_err(|still_unreadable| {
        MountError::NewerStateStillUnreadableAfterOneReread(Box::new(still_unreadable.into()))
    })?;
    if new_floor < effective_floor {
        return Err(
            MountError::RequestedFloorBelowTheEffectiveFloorWhoseRaiseIsUndecided {
                requested: new_floor,
                effective: effective_floor,
            },
        );
    }
    let ceiling_judged_by_the_admission = match entry {
        RaiseFloorEntry::Admission => {
            let ceiling = rollback_floor_ceiling(
                devices,
                &system_configuration,
                current.root.rollback_floor,
                &table,
                &ring_slots_known_to_hold_a_root_by(allocator),
            )?;
            if new_floor > ceiling {
                return Err(MountError::RollbackFloorAboveCeiling {
                    requested: new_floor,
                    ceiling,
                });
            }
            Some(ceiling)
        }
        // 卸载抬到现行那一版的 txg，不另判上限（「抬 F 的上限」那一行：等价于上限取最新的持久有效根、不按盘取小）。
        RaiseFloorEntry::Unmount => None,
    };
    let all_devices: Vec<DeviceIdentity> = devices.iter().map(|(identity, _)| *identity).collect();
    let device_of_txg = |txg: CheckpointTxg| {
        let target = target_for_publish(
            txg,
            parameters_of_the_pool.geometry.root_ring_slots_per_region,
        );
        parameters_of_the_pool.region_devices[usize::try_from(target.region).expect("区域号")]
    };
    // 这一串的 txg 在动分配器（影子账重算、回收）之前算：越过 u64::MAX 就拒（代码审阅第 36 条），分配器与盘都不动。
    let planned_txgs = txgs_of_the_publishes_carrying_the_floor_to_every_device(
        current.root.checkpoint_txg,
        &all_devices,
        &device_of_txg,
    )?;
    // 回收在写第一条带新 F 的根之前：一条根带的 F 与它的记账行要说同一件事——checker 的 I-3.1（已分配统计对得上） 按那条根自己的 F
    // 取候选集的并集，释放代 ≤ F 的落点不在并集里，记账的「已分配」也不能再算它们。但回收的槽在生效（每块盘都有带新 F 的根）之前
    // 不许发出去：带新 F 的空发布自己就在分配固定点，开放段满了会开到刚回收空的那一段（alloc-basis 第二轮云端攻方腿打中），所以先扣住、
    // 落满每块盘之后再放开。记账按写那条根的那一刻的 F 算还是按它持久之后的 F_生效 算，口径交 alloc-basis 那一轮（预想）。
    // 回收门槛与影子账读的这一遍根环，照 D16（发布语义） 已定项 1「根槽这一次读坏」那一行分两类（与算上限那一遍同一个分法）：
    // 这个进程知道住着根的槽读坏就重读那一槽一次，仍坏就拒这一次抬（C554 乙报告 Q6：按「没有根」往下走，那一槽里的被抛弃根引用的槽
    // 不隔离、也不计数）；挂载那一刻就读坏、之后没写过的槽当没有根。在动分配器（影子账重算、回收）之前返回，分配器与盘都不动。
    let roots = readable_roots_rereading_ring_slots_known_to_hold_a_root_once(
        &**devices,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
        &ring_slots_known_to_hold_a_root_by(allocator),
    )
    .map_err(|still_unreadable| {
        MountError::NewerStateStillUnreadableAfterOneReread(Box::new(still_unreadable.into()))
    })?;
    let oldest_valid_root = roots
        .iter()
        .filter(|root| !abandoned_by_table(root, &table))
        .map(|root| root.checkpoint_txg)
        .min();
    // 候选集按新 F 缩小，只被被抛弃根引用的槽会变多（一个槽此前靠一条低于新 F 的根豁免），回收之前先按新 F 重算影子账，
    // 不然那个槽回收之后就发得出去（步 4 / 步 5 代码三方第二轮辩方腿：窄读法要按每次挂载与每次抬 F 的候选集现算）。
    let abandoned_roots_unreadable = if shadow_ledger == ShadowLedger::On {
        match isolate_slots_referenced_only_by_abandoned_roots(
            devices,
            unit_area_start_of_the_chosen_system_configuration(&system_configuration),
            allocator,
            &roots,
            &|root| abandoned_by_table(root, &table),
            new_floor,
            &current.allocation_records,
            &mut BeforeTheOneReread::RereadImmediately,
        ) {
            Ok(computed) => computed.abandoned_roots_unreadable,
            Err(error) => {
                *allocator = allocator_before_the_raise;
                return Err(error);
            }
        }
    } else {
        0
    };
    let reclaimed = allocator.reclaim_released_up_to(
        reclaim_floor(new_floor, oldest_valid_root),
        ReclaimedReuse::HeldUntilFloorTakesEffect,
    );
    let publishes_in_the_sequence = planned_txgs.len();
    let mut pool = PoolWriter::new(&parameters_of_the_pool, devices.as_mut_slice());
    // 这一串在任何写之前在分配器的一份拷贝上整串预演（与可写挂载取号之前那一串同一个做法，`dry_run_of_the_publishes_after_acquisition`）：
    // 第 n 次在拷贝上报错（取不到落点、别的落盘之前的错），这一串一次都不发——F 不带出去，分配器整个换回抬 F 之前那一份
    // （扣住的槽放开、回收的回到 defer、补的隔离撤掉，C546（抬 F 被拒时扣住的槽不退回））。不预演时前 n − 1 次已经落盘：
    // 盘上带着新 F 而 F 没在每块盘上生效，扣住的槽留在这个进程里、计数上是空闲而发不出去（实二五报告第六节 Q3），
    // 而回退留下空档时那条落了盘的新 F 让 checker 在合法状态上判 I-3.1 红（增补 2 收口表第 43 行那一形，
    // 代码三方 `research/prompts/m2-final-code-r3-main-verification.md` 第三节「越格线索」的两个种子）。
    let placements_rehearsed = match rehearse_the_publishes_raising_the_floor(
        &pool,
        allocator,
        current,
        new_floor,
        &planned_txgs,
        entry.unmount_marker(),
    ) {
        Ok(placements_rehearsed) => placements_rehearsed,
        Err(refusal) => {
            *allocator = allocator_before_the_raise;
            return Err(
                MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite {
                    refused_publish_in_the_sequence: refusal.refused_publish_in_the_sequence,
                    publishes_in_the_sequence,
                    cause: refusal.cause,
                },
            );
        }
    };
    // 预演过了、第一次发布之前：先把新 F 写进每块盘的系统配置、过一道屏障，之后才发第一条带新 F 的根（SysPre，D16（发布语义） 已定项 1
    // 「抬 F 那一串」、已定项 7）。于是任何一条带新 F 的根落盘时，它那块盘的系统配置里已经有新 F（I-7.12（系统配置 F 不低于同盘根上的 F））；
    // 带新 F 的根全坏了，生效值照样读得出新 F。这一步报错：这一串的根一条都不发，已写进的新 F 留着、不回卷，分配器换回抬 F 之前那一份。
    let system_configuration_writes_before_the_first_publish =
        match write_the_raised_floor_into_every_system_configuration(
            &mut pool,
            new_floor,
            current.record.counter,
            current.record.instance,
        ) {
            Ok(writes) => writes,
            Err(failed) => {
                *allocator = allocator_before_the_raise;
                return Err(
                    MountError::RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot(Box::new(
                        failed,
                    )),
                );
            }
        };
    let mut publishes = Vec::new();
    // 迭代上界是预演过的次数（至多根环区域数）；跨轮携带的是现行那一版（下一次的计划接着它）。
    for planned_txg in &planned_txgs {
        let published = publish_version_with_unmount_marker(
            &mut pool,
            allocator,
            empty_publish_plan_raising_the_floor(current, new_floor),
            Some(&*current),
            entry.unmount_marker(),
        );
        let next = match published {
            Ok(next) => next,
            Err(cause) => {
                // 预演过了还在这里报错的只剩两种：落盘途中失败（那一次冻结在分配器上等原样重发，它的字节按回收之后的账装的，不换），
                // 与真发时读盘核出对不上、那一份留在已分配，而那一槽在拷贝上被这一串自己回收又发了出去（两边分叉，
                // 同 `establish_instance` 那一格）。第一次就在任何写之前被拒的，这一串什么都没落盘，分配器换回抬 F 之前的那一份；
                // 第二次起被拒的，前面已落盘的根带着新 F 与回收之后的账，扣住位照旧留在这个进程里，下一次抬 F 做成才放开
                // （C516（抬 F 那一串发布被拒时前面几次已落盘）；这一格扣住位怎么放，条款没写）。
                if publishes.is_empty() && allocator.frozen_publish().is_none() {
                    *allocator = allocator_before_the_raise.clone();
                }
                // 抬 F 的写入口是这里开的、随错一起丢掉：已经落盘的那几次各自的写与失败那一次已记的写都随错交出（增补 2 收口表第 58 行）。
                return Err(MountError::RaiseFloorSequencePublishFailed(
                    publish_sequence_failed(
                        &pool,
                        system_configuration_writes_before_the_first_publish.clone(),
                        publishes
                            .iter()
                            .map(|persisted: &TransactionOutput| persisted.writes.clone())
                            .collect(),
                        cause,
                    ),
                ));
            }
        };
        assert_eq!(
            next.root.checkpoint_txg, *planned_txg,
            "真发的 txg 与这一串计划里的那一个相同：计划从现行那一版的 txg 逐个加一，每一次都接着现行那一版加一"
        );
        publishes.push(next.clone());
        *current = next;
    }
    // 预演取的落点就是真发取的：同一个分配器、同一串分配动作。有一份换下的单元读盘核对不上时不比（那一份真发时留在已分配，
    // 拷贝上照常释放，那一槽若在这一串里被回收两边可以不同，同 `establish_instance` 那一格）。
    let some_copy_was_quarantined = publishes.iter().any(|version: &TransactionOutput| {
        !version
            .quarantined_after_release_checksum_mismatch
            .is_empty()
    });
    if !some_copy_was_quarantined {
        let placements_taken: Vec<PlacementsTakenByOnePublish> = publishes
            .iter()
            .map(placements_taken_by_a_file_version)
            .collect();
        assert_eq!(
            placements_taken, placements_rehearsed,
            "抬 F 之前在分配器的拷贝上预演取的落点与真发起来取的逐次相同：同一个分配器、同一串分配动作，这一串里没有一份被隔离；\
             不同说明两处的计划、次序或记根分叉了，预演判的不是真发的那一串"
        );
    }
    allocator.release_reclaim_holds();
    Ok(FloorRaisedThroughTheSequence {
        ceiling_judged_by_the_admission,
        system_configuration_writes_before_the_first_publish,
        publishes,
        reclaimed,
        abandoned_roots_unreadable,
    })
}
```

### crates/singlefs-core/src/transaction.rs:630-667（self_verified_system_configurations_of_every_device）

```rust
/// 取号那一刻逐盘读到的两槽：盘表里每块盘两槽中本池自证过（整槽校验和过、fsid 与本池相同）的系统配置，按盘表次序、一块盘一项。
/// 只读盘、不写。
///
/// 某块盘一份都读不出就拒（C554 乙-配置续 Q1，主 agent 2026-09-27 定走「拒」）：这块盘不「可见」（D18（块里携带什么信息） 已定项 11
/// 「可写挂载的顺序」：「可见」= 独占打开成功且系统配置读得通），与可写挂载取号之前的逐盘核（`mount.rs` 的
/// `devices_without_the_selected_version` 第一支，Z3-A 乙）同一判。那一道核与这一次读之间一次瞬时读错（或盘在两次读之间坏掉）就走得到这里；
/// 不拒的话见证值取别的盘的，这块盘上自证过的槽里见证的那次发布就漏在见证值之外。
///
/// # Errors
/// 按盘表次序第一块有一槽读不出或自证不过的盘 ⇒ [`InstanceAcquisitionFailed::DeviceWithAnUnreadOrUnverifiedSystemConfigurationSlotWhenReadingTheWitness`]。
fn self_verified_system_configurations_of_every_device<Device: BlockDevice>(
    pool: &PoolWriter<'_, Device>,
) -> Result<Vec<Vec<SystemConfiguration>>, InstanceAcquisitionFailed> {
    let spacing = u64::from(pool.parameters.geometry.fixed_structure_slot_spacing);
    let filesystem_identifier = pool.parameters.filesystem_identifier;
    pool.devices
        .iter()
        .map(|(identity, _)| {
            let slots_of_this_device = verified_system_configuration_slots(
                &*pool.devices,
                *identity,
                spacing,
                &filesystem_identifier,
            );
            // C331（择根倒挂压过已确认的写） 取甲（用户 2026-09-28 定）：有一槽读不出或自证不过，见证值就缺了那一槽，取不了值。
            if u64::try_from(slots_of_this_device.len()).expect("每块盘的系统配置槽数装得进 u64")
                < SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE
            {
                return Err(
                    InstanceAcquisitionFailed::DeviceWithAnUnreadOrUnverifiedSystemConfigurationSlotWhenReadingTheWitness {
                        device: *identity,
                    },
                );
            }
            Ok(slots_of_this_device)
        })
        .collect()
}
```

### crates/singlefs-core/src/transaction.rs:754-762（InstanceAcquisitionFailed）

```rust
/// 取号没做成（[`acquire_instance`]、取号的写那一半）。
#[derive(Debug)]
pub enum InstanceAcquisitionFailed {
    /// 取号那一刻读见证值时，`device` 两槽里有一槽读不出或本池自证不过（C554 乙-配置续 Q1；D18（块里携带什么信息） 已定项 11
    /// 「可见」；C331（择根倒挂压过已确认的写） 取甲，用户 2026-09-28 定，从「一份都没有」收严成「缺一槽」）：一个字节都没写。第一道屏障之前就读不出时连屏障都没发；屏障之后重读才读不出时发了那一道屏障、没有写。
    DeviceWithAnUnreadOrUnverifiedSystemConfigurationSlotWhenReadingTheWitness { device: DeviceIdentity },
    /// 屏障或取号写报错（回卷的结局在里面）。
    Acquisition(AcquisitionFailed),
}
```

### crates/singlefs-core/src/transaction.rs:770-786（ExpectedInstanceAcquisitionFailed）

```rust
/// 按调用方判定时算出的号取号没做成。
#[derive(Debug)]
pub enum ExpectedInstanceAcquisitionFailed {
    /// 写之前重算出的号与调用方判定时算出的号不同（两次读系统配置之间一次瞬时读错就够：读错的槽当作没有）：一个字节都没写。
    InstanceGenerationChangedBeforeWrite {
        expected: InstanceGeneration,
        recomputed: InstanceGeneration,
    },
    /// 写之前重算时盘上最大的实例代号已是 `u32::MAX`（两次读之间瞬时读错才会与判定时不同）：一个字节都没写。
    InstanceGenerationPastTheTopOfItsRange(InstanceGenerationPastTheTopOfItsRange),
    /// 同 [`InstanceAcquisitionFailed::DeviceWithAnUnreadOrUnverifiedSystemConfigurationSlotWhenReadingTheWitness`]：可写挂载取号之前的逐盘核读得出、
    /// 取号那一刻读见证值时这块盘有一槽读不出或自证不过（两次读之间一次瞬时读错就够）。一个字节都没写。
    DeviceWithAnUnreadOrUnverifiedSystemConfigurationSlotWhenReadingTheWitness {
        device: DeviceIdentity,
    },
    Acquisition(AcquisitionFailed),
}
```

### crates/singlefs-core/src/recovery.rs:1685-1749（effective_rollback_floor_of_the_roots_read；按行区间取：quote-rust-items.py 在签名里 `[DeviceIdentity; 3]` 的分号处截断成 1685-1689，行区间从同一文档注释起、到第 1687 行之后第一个顶格 `}` 为止）

```rust
/// 生效的回退下界 F 的算法本身（[`effective_rollback_floor`] 的文档），根环已经读过、判「有效根」的那张表已经读过（读不出时 `None`：
/// 不按表滤，只有 [`effective_rollback_floor`] 那一份这样传）。
fn effective_rollback_floor_of_the_roots_read<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    region_devices: &[DeviceIdentity; 3],
    immutable_sizes: &SystemImmutableSizes,
    filesystem_identifier: &[u8; 16],
    roots_with_ring_slots: &[(RootRingSlot, RootRecord)],
    newest_roots_table: Option<&InstanceTableRecords>,
) -> CheckpointTxg {
    let mut newest_valid_root_on_each_device: BTreeMap<DeviceIdentity, RootRecord> =
        BTreeMap::new();
    for (ring_slot, root) in roots_with_ring_slots {
        if newest_roots_table
            .is_some_and(|table| root_is_abandoned_by_the_instance_table(root, table))
        {
            continue;
        }
        let device = region_devices[usize::try_from(ring_slot.region).expect("区域号")];
        let newest_on_this_device = newest_valid_root_on_each_device
            .entry(device)
            .or_insert(*root);
        if (root.checkpoint_txg, root.instance)
            > (
                newest_on_this_device.checkpoint_txg,
                newest_on_this_device.instance,
            )
        {
            *newest_on_this_device = *root;
        }
    }
    // 被抛弃根带的 F 也算进 F_生效（收口表第 ② 行，用户 2026-09-28 定「算进」，被攻过一轮；
    // `research/prompts/abandoned-floor-r1-main-verification.md` M2）：不算时被抛弃根带着比别处高的 F 那一形上 I-7.12 红、
    // 回退能退到那个 F 之下，而被抛弃时间线抬 F 时已按它回收过槽。
    let highest_on_the_abandoned_roots = roots_with_ring_slots
        .iter()
        .filter(|(_, root)| {
            newest_roots_table
                .is_some_and(|table| root_is_abandoned_by_the_instance_table(root, table))
        })
        .map(|(_, root)| root.rollback_floor)
        .max();
    let highest_on_the_roots = newest_valid_root_on_each_device
        .values()
        .map(|root| root.rollback_floor)
        .chain(highest_on_the_abandoned_roots)
        .max();
    let slot_spacing_in_bytes = u64::from(immutable_sizes.fixed_structure_slot_spacing);
    let highest_in_the_system_configurations = reader
        .device_identities()
        .into_iter()
        .flat_map(|device| {
            verified_system_configuration_slots(
                reader,
                device,
                slot_spacing_in_bytes,
                filesystem_identifier,
            )
        })
        .map(|system_configuration| system_configuration.quantities.rollback_floor)
        .max();
    highest_on_the_roots
        .max(highest_in_the_system_configurations)
        .unwrap_or(CheckpointTxg(0))
}
```

### crates/singlefs-checker/src/walk.rs:5724-6367（check_pool_image）

```rust
/// 池级 checker 的入口：每条第一版不变量都报，没评估到的报「不适用」并带理由。
#[must_use]
pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, InvariantVerdict)> {
    let system_configurations = chosen_system_configurations(reader);
    let mut root_ring_judgements = Judgements::default();
    let chosen: Vec<_> = system_configurations
        .iter()
        .filter_map(|(device, chosen)| {
            chosen
                .clone()
                .map(|(view, geometry)| (*device, view, geometry))
        })
        .collect();
    // 一块盘的两个系统配置槽都无效时**不早退**：恢复会改用别的盘上那一份、照常挂上
    // （`crates/singlefs-core/src/recovery.rs` 的 `choose_system_configuration`，D22（单元原子性怎么合成） 已定项 8
    // 「系统配置每盘放一份」买的就是这份冗余）。早退会让这种镜像上每一条不变量都报「不适用」，
    // 而它是一个挂得上的合法镜像——故障注入让一块盘的两个槽先后写失败就造得出它，判定会被静默放过
    // （.claude/kb/checks-owed.md 的 C461）。⚠️ 「哪块盘的系统配置全废了」今天没有编号报得出来，仍欠在 C461。
    // 任一盘任一槽自证过而带这一版读者不收的池级值（格式版本、加密类型、槽距、physical_block_size、环长、journal 环起点、
    // 根环起点、单元区起始槽号）：实现整池拒绝挂载
    // （`RecoveryFailure::SystemConfigurationValueRefused`），checker 报违例、别的不变量同「挂不上的镜像」一样不作保
    // （用户 2026-09-27 定系统配置越界整池拒，实审 A3-checker-2）。只有一部分槽带越界值时不拿别的槽照判下去。
    let any_system_configuration_slot_carries_a_refused_value =
        judge_system_configuration_values_the_reader_accepts(
            &system_configuration_slot_readings(reader),
            &mut root_ring_judgements,
        );
    if any_system_configuration_slot_carries_a_refused_value {
        for invariant in crate::image::IMPLEMENTED_INVARIANTS {
            root_ring_judgements.not_applicable(
                invariant,
                "池里有一槽自证过的系统配置带这一版读者不收的池级值：实现整池拒绝挂载，这不是一个挂得上的镜像",
            );
        }
        return root_ring_judgements.into_report();
    }
    if chosen.is_empty() {
        for invariant in crate::image::IMPLEMENTED_INVARIANTS {
            root_ring_judgements.not_applicable(
                invariant,
                "池里没有一块盘交得出有效的系统配置：这不是一个挂得上的镜像",
            );
        }
        return root_ring_judgements.into_report();
    }
    let geometry = chosen[0].2;
    // 各盘择到的系统配置要属于同一个池：fsid 逐盘相同（一块别的池的旧盘插进来，实例代号可以恰好也是 1，I-7.7 看不出来）。
    for (device, view, _) in &chosen {
        root_ring_judgements.judge(
            "I-1.4",
            view.filesystem_identifier == geometry.filesystem_identifier,
            || {
                format!(
                    "盘 {device} 择到的系统配置 fsid 与盘 {} 的不同：这块盘不属于这个池",
                    chosen[0].0
                )
            },
        );
    }
    let devices = reader.devices();
    let regions = usize::try_from(geometry.regions).expect("R");
    let region_devices = &geometry.region_devices[..regions.min(3)];
    let distinct: BTreeSet<u32> = region_devices.iter().copied().collect();
    let heaviest = devices
        .iter()
        .map(|device| {
            region_devices
                .iter()
                .filter(|candidate| *candidate == device)
                .count()
        })
        .max()
        .unwrap_or(0);
    let pigeonhole = regions.div_ceil(devices.len().max(1));
    root_ring_judgements.judge(
        "I-7.6",
        distinct.len() == regions.min(devices.len()) && heaviest <= pigeonhole && region_devices.iter().all(|device| devices.contains(device)),
        || format!("根环区域的设备 {region_devices:?}：不同值 {}、最重的盘背 {heaviest} 个（上界 {pigeonhole}）", distinct.len()),
    );
    let filesystem_identifier_low = u64::from_le_bytes(
        geometry.filesystem_identifier[..8]
            .try_into()
            .expect("8 字节"),
    );
    let roots = valid_roots(reader, &geometry);
    // 每块盘两槽里自证过、fsid 与本池相同的系统配置槽（与实现取号同一读法，D18（块里携带什么信息） 已定项 11）：
    // I-7.12 逐盘比，回退候选集的 F 生效值取它们带的 F 的最大值（C556（checker 与层 0 不读系统配置里的 F））。
    let verified_system_configurations_of_this_pool: Vec<(
        u32,
        Vec<crate::SystemConfigurationView>,
    )> = verified_system_configuration_slots(reader)
        .into_iter()
        .map(|(device, slots)| {
            (
                device,
                slots
                    .into_iter()
                    .map(|(view, _)| view)
                    .filter(|view| view.filesystem_identifier == geometry.filesystem_identifier)
                    .collect(),
            )
        })
        .collect();
    judge_own_device_number_is_the_device_identity(
        &verified_system_configurations_of_this_pool,
        &mut root_ring_judgements,
    );
    judge_system_configuration_floor_against_the_roots_on_each_device(
        &verified_system_configurations_of_this_pool,
        &roots,
        &geometry,
        &mut root_ring_judgements,
    );
    judge_instance_carriers(reader, &geometry, &roots, &mut root_ring_judgements);
    // I-8.6、I-8.7、I-8.8 与 I-8.9 只读 journal 环，不读根：根环全灭的镜像上它们照样判得了（那一格归 I-7.1）。
    let journal_records_by_device =
        scanned_journal_records_by_device(reader, &geometry, filesystem_identifier_low);
    judge_journal_back_chain(&journal_records_by_device, &mut root_ring_judgements);
    judge_location_order_of_journal_named_entries(
        &journal_records_by_device,
        &mut root_ring_judgements,
    );
    judge_transaction_numbers_per_instance(&journal_records_by_device, &mut root_ring_judgements);
    judge_commit_markers_per_transaction(&journal_records_by_device, &mut root_ring_judgements);
    judge_publish_ordinals_and_last_record_flags(
        &journal_records_by_device,
        &mut root_ring_judgements,
    );
    root_ring_judgements.judge("I-7.1", !roots.is_empty(), || {
        "根环里一条自证过的根都没有".to_string()
    });
    if roots.is_empty() {
        root_ring_judgements.not_applicable(
            "I-7.3",
            "根环里一条自证过的根都没有：S 空，没有「代号最大者」可谈",
        );
        return root_ring_judgements.into_report();
    }
    judge_root_ring_health(&roots, &mut root_ring_judgements);
    // 择根：按 (txg, 实例) 取最大（D22（单元原子性怎么合成） 已定项 7，与实现 `recovery::choose_root` 同一个读法）。
    let newest_index = roots
        .iter()
        .enumerate()
        .max_by_key(|(_, (_, _, root))| (root.checkpoint_txg, root.instance))
        .map(|(index, _)| index)
        .expect("上面判过根环里至少一条自证过的根");
    let mut walk = Walk::starting_with(
        reader,
        root_ring_judgements,
        filesystem_identifier_low,
        roots[newest_index].2.instance,
        roots[newest_index].2.checkpoint_txg,
    );
    // 先走最新的根：它的走读断没断就是 I-7.2；记账行只取最新根下面的。
    walk.walk_root(&roots[newest_index].2.record_bytes, true);
    let newest_failures = walk.walk_failures.clone();
    // I-3.11（已分配减 defer 等于最新根走读）要的「从最新有效根走读到的、这块盘上被引用的槽数」：走法与 I-3.1 同一个
    // （`note_reference` 记下的 (设备, 起点槽, 跨度)），只取最新根——这一刻 `references` 里还只有最新根这一遍记下的。
    let slots_referenced_by_the_newest_root = slots_referenced_per_device(&walk.references);
    let start_slots_referenced_by_the_newest_root: BTreeSet<(u32, u64)> = walk
        .references
        .keys()
        .map(|(device, start_slot, _)| (*device, *start_slot))
        .collect();
    // I-4.8（近 K 代根校验和自洽）与 I-7.4（近 K 代块未被复用）：候选集里任一根（最新根也在候选集里）出发遍历，所有块的校验和
    // 都与父指针一致、走读不断——最新根那一次各算一格；候选集只剩最新根时两条都还判得到（本地攻方腿：全称量词在单元素集合上照样成立）。
    let newest_txg = roots[newest_index].2.checkpoint_txg;
    let newest_walked_into_reused_or_erased_unit =
        !newest_failures.is_empty() || walk.judgements.violation_count("I-2.1") > 0;
    walk.judgements
        .judge("I-4.8", !newest_walked_into_reused_or_erased_unit, || {
            format!("最新根（txg {newest_txg}）出发的遍历有单元对不上或读不出")
        });
    walk.judgements
        .judge("I-7.4", !newest_walked_into_reused_or_erased_unit, || {
            format!("最新根（txg {newest_txg}）引用的单元已被复用或抹头（校验和对不上或头用不了）")
        });
    let accounting_seen = walk.accounting_seen;
    let accounting = walk.accounting.clone();
    // 别的根只取按最新根指着的实例表仍然有效的：(i, T) 有效 ⟺ 无 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti
    // （D23（journal 的角色与格式） 已定项 14 回退段的候选集规则）；被抛弃时间线的根引用的单元由影子账隔离、不在当前账里。
    // 再加一条：txg ≥ 回退下界 F 的生效值（D16（发布语义） 已定项 1 的回退候选集，`effective_rollback_floor`）；F 之下的根引用的单元可以已被回收复用，
    // 它们不在当前账里、也不再是「近 K 代」——I-2.1 只在候选集里的根上判。
    let instance_table_rows = walk.instance_table_rows.clone();
    let abandoned_by_the_newest_roots_table = |root: &crate::RootView| {
        instance_table_rows.iter().any(|row| {
            row.instance == root.instance && root.checkpoint_txg > row.published_checkpoint_txg
        })
    };
    // 回退候选集的下界取 F 生效值（D16（发布语义） 已定项 1「生效」，SysPre）：max(各幸存盘最新持久有效根所带 F 的最大值,
    // 池里自证过的系统配置槽带的 F)。有效 = 按最新根指着的实例表判不是被抛弃的；每块盘取落在它上面的根环区域里 (txg, 实例) 最大的那一条
    // （最新根的实例表读不出时一行都没有、不滤）；被抛弃时间线上的根带的 F 也算进来（D16（发布语义） 已定项 1，用户 2026-09-28 定「算进」）。
    // 只读根上的 F 时，带新 F 的根全坏、系统配置里的新 F 还在的合法镜像上，F 之下、单元已被合法复用的根会被当成候选，
    // I-7.4（近 K 代块未被复用） 等按候选集判的几条误红（C556（checker 与层 0 不读系统配置里的 F））。
    let mut newest_valid_root_on_each_device: BTreeMap<u32, &crate::RootView> = BTreeMap::new();
    for (region, _, root) in &roots {
        if abandoned_by_the_newest_roots_table(root) {
            continue;
        }
        let device = geometry.region_devices[usize::try_from(*region).expect("区域号")];
        let newest_on_this_device = newest_valid_root_on_each_device
            .entry(device)
            .or_insert(root);
        if (root.checkpoint_txg, root.instance)
            > (
                newest_on_this_device.checkpoint_txg,
                newest_on_this_device.instance,
            )
        {
            *newest_on_this_device = root;
        }
    }
    let highest_floor_on_the_abandoned_roots = roots
        .iter()
        .filter(|(_, _, root)| abandoned_by_the_newest_roots_table(root))
        .map(|(_, _, root)| root.rollback_floor)
        .max()
        .unwrap_or(0);
    let highest_floor_on_the_newest_valid_and_the_abandoned_roots =
        newest_valid_root_on_each_device
            .values()
            .map(|root| root.rollback_floor)
            .max()
            .unwrap_or(0)
            .max(highest_floor_on_the_abandoned_roots);
    let effective_rollback_floor = verified_system_configurations_of_this_pool
        .iter()
        .flat_map(|(_, slots)| slots.iter().map(|view| view.rollback_floor))
        .fold(
            highest_floor_on_the_newest_valid_and_the_abandoned_roots,
            u64::max,
        );
    // 掉出遍历的根槽按理由各记一次（两样都占的两边都记）：I-3.1 判红时这几个数就是「遍历为什么少算」的机理标识。
    let mut root_slots_dropped_as_abandoned = 0u64;
    let mut root_slots_dropped_below_floor = 0u64;
    let candidate_indexes: Vec<usize> = roots
        .iter()
        .enumerate()
        .filter(|(index, (_, _, root))| {
            // 被抛弃：按最新根指着的实例表判（D23（journal 的角色与格式） 已定项 14 候选集）。
            let abandoned = abandoned_by_the_newest_roots_table(root);
            let below_floor = root.checkpoint_txg < effective_rollback_floor;
            let walked = *index == newest_index || (!abandoned && !below_floor);
            if !walked {
                if abandoned {
                    root_slots_dropped_as_abandoned += 1;
                }
                if below_floor {
                    root_slots_dropped_below_floor += 1;
                }
            }
            walked
        })
        .map(|(index, _)| index)
        .collect();
    for index in candidate_indexes.iter().copied() {
        let (_, _, root) = &roots[index];
        if index != newest_index {
            let mismatches_before = walk.judgements.violation_count("I-2.1");
            let failures_before = walk.walk_failures.len();
            walk.walk_root(&root.record_bytes, false);
            // 这条候选根引用的单元有一个校验和对不上或头用不了，就是它指着的块被复用或抹头了（I-7.4（近 K 代块未被复用）），
            // 从它出发的遍历也就不自洽（I-4.8（近 K 代根校验和自洽））；两条按每条候选根各判一格。
            let walked_into_reused_or_erased_unit = walk.judgements.violation_count("I-2.1")
                > mismatches_before
                || walk.walk_failures.len() > failures_before;
            let root_txg = root.checkpoint_txg;
            walk.judgements
                .judge("I-7.4", !walked_into_reused_or_erased_unit, || {
                    format!(
                        "候选根 txg {root_txg} 引用的单元已被复用或抹头（校验和对不上或头用不了）"
                    )
                });
            walk.judgements
                .judge("I-4.8", !walked_into_reused_or_erased_unit, || {
                    format!("候选根 txg {root_txg} 出发的遍历有单元对不上或读不出")
                });
        }
    }
    // 回退候选集补上「由记录施加出来、根槽从没落盘的那一版」（2026-09-23 用户定候选 b）：它们没有根槽，
    // 上面按根环走的那一遍走不到，而它们换下的单元还在 defer 里、仍算在已分配里（增补 2 收口表第 54 行那 12 个状态差的 65 536 字节）。
    // 走法与判定照候选根：引用进同一个并集（I-3.1 / I-5.1），单元照判 I-2.1 等，I-7.4 / I-4.8 每一版各判一格。
    let versions_applied_only_by_records = versions_applied_only_by_records(
        &journal_records_by_device,
        &roots,
        &instance_table_rows,
        effective_rollback_floor,
    );
    for version in &versions_applied_only_by_records {
        let mismatches_before = walk.judgements.violation_count("I-2.1");
        let failures_before = walk.walk_failures.len();
        walk.walk_version_applied_only_by_records(version);
        let walked_into_reused_or_erased_unit = walk.judgements.violation_count("I-2.1")
            > mismatches_before
            || walk.walk_failures.len() > failures_before;
        let (instance, version_txg) = (version.instance, version.checkpoint_txg);
        walk.judgements
            .judge("I-7.4", !walked_into_reused_or_erased_unit, || {
                format!(
                    "由记录施加出来的那一版（实例 {instance}、txg {version_txg}）引用的单元已被复用或抹头（校验和对不上或头用不了）"
                )
            });
        walk.judgements
            .judge("I-4.8", !walked_into_reused_or_erased_unit, || {
                format!(
                    "由记录施加出来的那一版（实例 {instance}、txg {version_txg}）出发的遍历有单元对不上或读不出"
                )
            });
    }
    judge_blocks_referenced_by_abandoned_roots(
        reader,
        &geometry,
        &roots,
        newest_index,
        &instance_table_rows,
        filesystem_identifier_low,
        &mut walk.judgements,
    );
    let referenced_units_judged_against_their_pointer =
        walk.referenced_units_judged_against_their_pointer;
    let mut location_entry_checksums = LocationEntryChecksumsForQuarantine {
        known: std::mem::take(&mut walk.location_entry_checksums),
        roots_not_walked_yet: Some(RootsNotWalked {
            reader,
            filesystem_identifier_low,
            mount_root_instance: roots[newest_index].2.instance,
            mount_root_checkpoint_txg: roots[newest_index].2.checkpoint_txg,
            root_records: roots
                .iter()
                .enumerate()
                .filter(|(index, _)| !candidate_indexes.contains(index))
                .map(|(_, (_, _, root))| root.record_bytes.clone())
                .collect(),
        }),
    };
    let mut judgements = walk.judgements;
    judgements.judge("I-7.2", newest_failures.is_empty(), || {
        format!("最新的根走不完：{}", newest_failures.join("；"))
    });
    // I-1.2 与 I-4.2 在遍历方向上判的是同一批单元：走读一个被引用单元都没走到时两条都没有对象
    // （根槽读得出而树表指针全零、或每个单元的头都用不了），报「不适用」并带理由，不报成立。
    if referenced_units_judged_against_their_pointer == 0 {
        for invariant in ["I-1.2", "I-4.2"] {
            judgements.not_applicable(
                invariant,
                "走读一个被引用的单元都没读到（实例表单元按 I-1.2 那一行的例外不进这两条）：出生身份与已发布谓词都没有对象",
            );
        }
    }
    // I-1.8：扫描方向按已发布谓词过滤之后归并成组。谓词要挂载根的 (实例代号, txg) 与它指着的那一版实例表，
    // 所以这一遍排在走读之后（`instance_table_rows` 是走最新根时取的）。
    let published = PublishedPredicate {
        mount_root_instance: roots[newest_index].2.instance,
        mount_root_checkpoint_txg: roots[newest_index].2.checkpoint_txg,
        instance_table_rows: &instance_table_rows,
    };
    let content_units =
        scanned_content_units(reader, &geometry, filesystem_identifier_low, &published);
    if judge_merged_version_total_order(reader, &content_units, &mut judgements) == 0 {
        judgements.not_applicable(
            "I-1.8",
            "扫描方向没有两份归并到一起、或两组归并到同一个 key 的码 1 / 码 3 已发布单元：归并与定序都比不出来",
        );
    }
    // 这两遍各自按候选集里每条根读树表与树节点：共用一份按位置条目记的缓存，候选根常指着同一个单元，层 0 每个崩溃状态都跑。
    let mut index_node_cache = IndexNodeCache::new();
    judge_release_generation_and_tree_table_birth(
        reader,
        &roots,
        &candidate_indexes,
        newest_index,
        &mut index_node_cache,
        &mut judgements,
    );
    judge_allocation_records_disjoint(
        reader,
        &roots,
        &candidate_indexes,
        &mut index_node_cache,
        &mut judgements,
    );
    let allocation_node_pointers = allocation_record_node_pointers_of_the_candidate_versions(
        reader,
        &roots,
        &candidate_indexes,
        &versions_applied_only_by_records,
        tree_table_pointer_of_the_version_the_next_mount_applies_first(
            &journal_records_by_device,
            &roots,
            newest_index,
        ),
        &mut index_node_cache,
    );
    judge_allocation_generations_against_unit_births(
        reader,
        &allocation_node_pointers,
        &mut index_node_cache,
        &mut judgements,
    );
    judge_rollback_floor_raises_against_their_ceilings(
        reader,
        &geometry,
        &roots,
        &mut index_node_cache,
        &mut judgements,
    );
    // I-9.6（水位大于两处最大号）：记账里那条「inode 号水位」要大于遍历侧算出的 inode 树内最大 key。
    // 两条独立路径——水位是发布路径在记账树里写下的一个数，最大 key 是 checker 逐片叶容器逐条记录数出来的。
    // 另一半（> 全部已发布的墓碑记录的对象 ID）今天没有对象：墓碑是打包记录类型 1，这一版一片都不写
    // （没有删除，C118（`deleted_inodes` 树的形态无落点） 未定形态）。
    let inode_number_watermark = accounting
        .get(&(STATISTIC_INODE_WATERMARK, STATISTIC_NO_DEVICE_DIMENSION))
        .copied();
    match (
        inode_number_watermark,
        walk.largest_inode_number_in_the_inode_tree,
    ) {
        (Some(watermark), Some(largest_inode)) => {
            judgements.judge("I-9.6", watermark > largest_inode, || {
                format!(
                    "记账里的 inode 号水位 {watermark} 不大于 inode 树内最大 key {largest_inode}"
                )
            });
        }
        (None, _) => judgements.not_applicable(
            "I-9.6",
            "最新的根下面没有记账树、或记账里没有 inode 号水位那一行",
        ),
        (Some(_), None) => {
            judgements.not_applicable("I-9.6", "最新的根下面走不到 inode 树里的任何一条记录")
        }
    }
    if !walk.inode_tree_walked {
        judgements.not_applicable(
            "I-9.12",
            "最新的根下面走不到 inode 树的内部节点（树表 0 条或根读不出）",
        );
    }
    for (inode, unit_object_birth, what) in &walk.data_unit_objects {
        if let Some(record_birth) = walk.inode_object_birth.get(inode) {
            judgements.judge("I-9.10", record_birth == unit_object_birth, || {
                format!(
                    "{what} 的对象出生代 {unit_object_birth} 与 inode 记录的 {record_birth} 不符"
                )
            });
        }
    }
    // I-5.1：同一块盘上，不同的引用不许占重叠的槽（两条位置条目落在不同盘上是显式的多副本）。
    let mut per_device: BTreeMap<u32, Vec<(u64, u64, &String)>> = BTreeMap::new();
    for ((device, slot, span), what) in &walk.references {
        per_device
            .entry(*device)
            .or_default()
            .push((*slot, *span, what));
    }
    for (device, mut ranges) in per_device.clone() {
        ranges.sort_unstable_by_key(|(slot, span, _)| (*slot, *span));
        for pair in ranges.windows(2) {
            let (first_slot, first_span, first_what) = pair[0];
            let (second_slot, _, second_what) = pair[1];
            judgements.judge("I-5.1", first_slot + first_span <= second_slot, || {
                format!("盘 {device}：{first_what}（槽 {first_slot} 跨 {first_span}）与 {second_what}（槽 {second_slot}）重叠")
            });
        }
    }
    // I-3.1 判红时，说明文字里带一段机理标识：遍历覆盖了哪些根、剩下的按什么理由没覆盖。
    // 判读的一方（`singlefs-harness` 的 `history::allocation_statistic_mechanism`）按它分辨「记账多算」是哪一种机理造成的：
    // 环转过一圈把 F 之上的根挤出了环，还是回退下界把环里读得到的根挡在了外面——两者的签名（只有 I-3.1 红、记账多于遍历）
    // 一模一样，不带这一段就只能按签名认，机理不同的新问题会被「已知红」清单接走（代码三方 m2-supp3-item3-code-r1 判决 K6 的假阴那一半）。
    let root_ring_slot_count = geometry.regions * geometry.slots_per_region;
    let oldest_readable_root_txg = roots
        .iter()
        .map(|(_, _, root)| root.checkpoint_txg)
        .min()
        .unwrap_or(0);
    let readable_root_slot_count = u64::try_from(roots.len()).expect("根槽数");
    let walked_root_slot_count = u64::try_from(candidate_indexes.len()).expect("候选根槽数");
    let walked_versions_applied_only_by_records =
        u64::try_from(versions_applied_only_by_records.len()).expect("版本数");
    // 「并进遍历的由记录施加出来的版本」那一段是 2026-09-23 候选 b 加的；判读的一方按字段名取数（`leading_number_after`），
    // 不看各段的次序，所以插在「遍历的候选根槽」之后、与它挨着读。
    let mechanism = move || {
        format!(
            "；机理：根环槽数 {root_ring_slot_count}、最新根 txg {newest_txg}、环里自证过的根槽 {readable_root_slot_count} 个、最老的自证过的根 txg {oldest_readable_root_txg}、遍历的候选根槽 {walked_root_slot_count} 个、并进遍历的由记录施加出来的版本 {walked_versions_applied_only_by_records} 个、被实例表判抛弃的根槽 {root_slots_dropped_as_abandoned} 个、回退下界 F {effective_rollback_floor}、低于 F 的根槽 {root_slots_dropped_below_floor} 个"
        )
    };
    // I-3.1 / I-5.2：最新根下面记账树的「已分配」「空闲」逐盘对遍历得到的和与容量。
    // I-3.11：同一块盘上「已分配」减「defer 待释放」对只走最新根那一遍得到的和。
    let slots_referenced_by_every_walked_version = slots_referenced_per_device(&walk.references);
    let start_slots_referenced_by_every_walked_version: BTreeSet<(u32, u64)> = walk
        .references
        .keys()
        .map(|(device, start_slot, _)| (*device, *start_slot))
        .collect();
    // 隔离的记录（已分配而没有根引用、checker 自己读那一份也读不出或与位置项里的校验和对不上）在这两条上豁免：I-3.1 对全部走过的版本的引用、
    // I-3.11 对最新根这一遍的引用各算一份（`quarantined_slots_exempted_per_device`）。
    let newest_root_view = &roots[newest_index].2;
    let quarantined_exempted_against_every_walked_version = quarantined_slots_exempted_per_device(
        reader,
        newest_root_view,
        &start_slots_referenced_by_every_walked_version,
        &mut location_entry_checksums,
        &mut index_node_cache,
    );
    let walked_slots_of_every_walked_version: BTreeSet<(u32, u64)> = walk
        .references
        .keys()
        .flat_map(|(device, start_slot, span_slots)| {
            (*start_slot..*start_slot + *span_slots).map(move |slot| (*device, slot))
        })
        .collect();
    let roots_below_the_floor: Vec<&crate::RootView> = roots
        .iter()
        .map(|(_, _, root)| root)
        .filter(|root| {
            !abandoned_by_the_newest_roots_table(root)
                && root.checkpoint_txg < effective_rollback_floor
        })
        .collect();
    let deferred_below_the_floor_against_every_walked_version =
        deferred_slots_referenced_only_below_the_floor_per_device(
            reader,
            filesystem_identifier_low,
            newest_root_view,
            &roots_below_the_floor,
            &walked_slots_of_every_walked_version,
            effective_rollback_floor,
            &mut index_node_cache,
        );
    let quarantined_exempted_against_the_newest_root = quarantined_slots_exempted_per_device(
        reader,
        newest_root_view,
        &start_slots_referenced_by_the_newest_root,
        &mut location_entry_checksums,
        &mut index_node_cache,
    );
    // 最新根的记账是按它自己带的 F 记的（抬 F 的回收在写第一条带新 F 的根之前做，`mount::raise_rollback_floor`）：它带的 F 低于
    // F 生效值时——先把新 F 写进系统配置之后、带新 F 的根落盘之前（D16（发布语义） 已定项 1「抬 F 那一串」），或带新 F 的根全读不出——
    // 它的「已分配」还算着 F 生效值之下那些根引用的槽，与按 F 生效值取的候选集并集不可比，I-3.1 这一格报不适用（实二的读法，交主 agent）。
    let accounting_was_written_under_the_effective_floor =
        roots[newest_index].2.rollback_floor == effective_rollback_floor;
    if accounting_seen {
        for device in &devices {
            let quarantined_exempted = quarantined_exempted_against_every_walked_version
                .get(device)
                .copied()
                .unwrap_or(0)
                * SLOT_BYTES;
            let deferred_below_the_floor = deferred_below_the_floor_against_every_walked_version
                .get(device)
                .copied()
                .unwrap_or(0)
                * SLOT_BYTES;
            let walked = slots_referenced_by_every_walked_version
                .get(device)
                .copied()
                .unwrap_or(0)
                * SLOT_BYTES
                + quarantined_exempted
                + deferred_below_the_floor;
            let allocated = accounting
                .get(&(STATISTIC_ALLOCATED_BYTES, *device))
                .copied();
            if accounting_was_written_under_the_effective_floor {
                judgements.judge("I-3.1", allocated == Some(walked), || {
                    format!(
                        "盘 {device}：记账的已分配 {allocated:?}，遍历全部有效根得到 {walked}（其中隔离豁免 {quarantined_exempted}、F 之下仍在 defer 的 {deferred_below_the_floor}）{}",
                        mechanism()
                    )
                });
            }
            // 单元区起点是系统配置里读来的 8 字节、空闲与已分配是记账行里读来的 8 字节：减法在起点越过盘末时下溢、加法在两个大值上溢出，
            // checker 当场 panic（代码审阅 6c 第 6 条那一族）。起点越过盘末时单元区没有容量可比（`None`），这一格与盘容量读不出同样判红；
            // 两个值加起来溢出也判红、不回绕（与 I-3.11 那一判同一个写法）。
            let capacity = reader.device_bytes(*device).and_then(|bytes| {
                (bytes / SLOT_BYTES)
                    .checked_sub(geometry.unit_area_start_slot)
                    .map(|unit_area_slots| unit_area_slots * SLOT_BYTES)
            });
            let free = accounting.get(&(STATISTIC_FREE_BYTES, *device)).copied();
            judgements.judge("I-5.2", matches!((free, allocated, capacity), (Some(free), Some(allocated), Some(capacity)) if free.checked_add(allocated) == Some(capacity)), || {
                format!("盘 {device}：空闲 {free:?} + 已分配 {allocated:?} ≠ 单元区 {capacity:?}")
            });
            // I-3.11（已分配减 defer 等于最新根走读）：写者把活单元记成已释放、放进 defer 时，已分配与空闲两个和照样对得上
            // （I-3.1 / I-5.2 都绿），只有这一条的差对不上。判据写成加法（defer + 最新根走读 == 已分配），盘上读来的值大到溢出也判红、不回绕。
            let deferred = accounting
                .get(&(STATISTIC_DEFER_QUEUE_BYTES, *device))
                .copied();
            let quarantined_exempted_newest = quarantined_exempted_against_the_newest_root
                .get(device)
                .copied()
                .unwrap_or(0)
                * SLOT_BYTES;
            let referenced_by_the_newest_root = slots_referenced_by_the_newest_root
                .get(device)
                .copied()
                .unwrap_or(0)
                * SLOT_BYTES
                + quarantined_exempted_newest;
            judgements.judge(
                "I-3.11",
                matches!((allocated, deferred), (Some(allocated), Some(deferred)) if deferred.checked_add(referenced_by_the_newest_root) == Some(allocated)),
                || {
                    format!("盘 {device}：记账的已分配 {allocated:?} 减 defer 待释放 {deferred:?}，不等于从最新根（txg {newest_txg}）走读到的 {referenced_by_the_newest_root}（其中隔离豁免 {quarantined_exempted_newest}）")
                },
            );
        }
        if !accounting_was_written_under_the_effective_floor {
            judgements.not_applicable(
                "I-3.1",
                "最新根带的 F 低于 F 生效值（系统配置里的新 F 已落、带它的根还没落或全读不出）：最新根的记账按它自己的 F 记，与按 F 生效值取的候选集并集不可比",
            );
        }
    } else {
        judgements.not_applicable("I-3.1", "最新的根下面还没有记账树（第 0 代树表）");
        judgements.not_applicable("I-5.2", "最新的根下面还没有记账树（第 0 代树表）");
        judgements.not_applicable("I-3.11", "最新的根下面还没有记账树（第 0 代树表）");
    }
    // I-7.8：根环全部有效根的水位取 max，要大于盘上出现过的最大树 ID（码 2 单元头 ∪ 走过的树表条目；
    // 单元头那一半不数没发布过的孤儿，读法见 `scanned_tree_identifiers`）。
    let watermark = roots
        .iter()
        .map(|(_, _, root)| root.tree_identifier_watermark)
        .max()
        .unwrap_or(0);
    let newest_published_txg = roots
        .iter()
        .map(|(_, _, root)| root.checkpoint_txg)
        .max()
        .unwrap_or(0);
    let mut seen = scanned_tree_identifiers(
        reader,
        &geometry,
        newest_published_txg,
        &instance_table_rows,
        filesystem_identifier_low,
    );
    seen.extend(walk.tree_identifiers_in_tables.iter().copied());
    let highest_seen = seen.iter().max().copied().unwrap_or(0);
    judgements.judge("I-7.8", watermark > highest_seen, || {
        format!("根环水位最大 {watermark}，盘上出现过的最大树 ID {highest_seen}")
    });
    judgements.into_report()
}
```

### crates/singlefs-checker/src/walk.rs:4231-4299（deferred_slots_referenced_only_below_the_floor_per_device）

```rust
/// 收口表第 43 行取丁-defer（用户 2026-09-28 定，被攻过一轮；`research/prompts/abandoned-floor-r1-main-verification.md` M1）：
/// I-3.1（已分配统计对得上） 的「实际遍历」并上这样的槽——F_生效 之下、没被抛弃的根引用着，候选集那一遍没走到，而最新根的分配记录树里
/// 罩住它的记录是已释放、释放代高于 F_生效 的（还在 defer 里，抬 F 还不许回收，记账照算已分配）。逐盘交回槽数。
/// 最新根的树表或分配记录树读不出时一槽都不并（照旧判）。
fn deferred_slots_referenced_only_below_the_floor_per_device(
    reader: &dyn ImageReader,
    filesystem_identifier_low: u64,
    newest_root: &crate::RootView,
    roots_below_the_floor: &[&crate::RootView],
    walked_slots: &BTreeSet<(u32, u64)>,
    effective_rollback_floor: u64,
    cache: &mut IndexNodeCache,
) -> BTreeMap<u32, u64> {
    let mut deferred_per_device: BTreeMap<u32, u64> = BTreeMap::new();
    let tree_table_pointer = parse_node_pointer(&newest_root.record_bytes[36..122]);
    if tree_table_pointer.all_zero {
        return deferred_per_device;
    }
    let Some(tree_table) = read_index_node_without_judging(reader, &tree_table_pointer, cache)
    else {
        return deferred_per_device;
    };
    let mut released_above_the_floor: BTreeSet<(u32, u64)> = BTreeSet::new();
    for entry in &tree_table.entries {
        if entry.len() < tree_table_entry_bytes() || read_u16(entry, 10) != TREE_KIND_ALLOCATION {
            continue;
        }
        let allocation_root = parse_node_pointer(&entry[14..100]);
        if allocation_root.all_zero {
            continue;
        }
        let Some((_, records)) =
            allocation_record_tree_without_judging(reader, &allocation_root, cache)
        else {
            continue;
        };
        for record in records {
            if record.is_released && record.generation > effective_rollback_floor {
                released_above_the_floor.extend(
                    (record.slot..record.slot + record.span_slots)
                        .map(|slot| (record.device, slot)),
                );
            }
        }
    }
    let mut deferred: BTreeSet<(u32, u64)> = BTreeSet::new();
    for root in roots_below_the_floor {
        let mut walk_of_the_root_below_the_floor = Walk::starting_with(
            reader,
            Judgements::default(),
            filesystem_identifier_low,
            newest_root.instance,
            newest_root.checkpoint_txg,
        );
        walk_of_the_root_below_the_floor.walk_root(&root.record_bytes, false);
        for (device, start_slot, span_slots) in walk_of_the_root_below_the_floor.references.keys() {
            for slot in *start_slot..*start_slot + *span_slots {
                let key = (*device, slot);
                if !walked_slots.contains(&key) && released_above_the_floor.contains(&key) {
                    deferred.insert(key);
                }
            }
        }
    }
    for (device, _) in deferred {
        *deferred_per_device.entry(device).or_insert(0) += 1;
    }
    deferred_per_device
}
```

### crates/singlefs-harness/src/history.rs:2411-2429（still_unreadable_after_one_reread_member）

```rust
fn still_unreadable_after_one_reread_member(
    still_unreadable: &StillUnreadableAfterOneReread,
) -> &'static str {
    match still_unreadable {
        StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion { .. } => {
            "PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion"
        }
        StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheShadowLedger { .. } => {
            "InstanceTableOfTheNewestRootForTheShadowLedger"
        }
        StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheEffectiveFloor { .. } => {
            "InstanceTableOfTheNewestRootForTheEffectiveFloor"
        }
        StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot { .. } => {
            "RootRingSlotKnownToHoldARoot"
        }
        StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot { .. } => "AccountOfAnAbandonedRoot",
    }
}
```

### crates/singlefs-harness/src/model_comparison.rs:712-788（refusal_reason_of_mount_error）

```rust
/// 挂载、抬 F 的错误成员说的是哪条理由。
#[must_use]
pub fn refusal_reason_of_mount_error(error: &MountError) -> ObservedRefusalReason {
    match error {
        MountError::Publish(failed) | MountError::RaiseFloorSequencePublishFailed(failed) => {
            refusal_reason_of_publish_error(&failed.cause)
        }
        MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite { cause, .. }
        | MountError::RowPublishAdmissionRefusedBeforeAcquisition { cause, .. }
        | MountError::WarmUpAdmissionRefusedBeforeAcquisition { cause, .. } => {
            refusal_reason_of_publish_error(cause)
        }
        // 可写挂载写行与暖机之后推抬 F、抬 F 自己报错（实审 A1b Q5）：理由就是抬 F 那个错的理由（改之前挂载原样交回它）。
        MountError::FloorRaiseFailedAfterTheMountsPublishes(failed) => {
            refusal_reason_of_mount_error(&failed.cause)
        }
        MountError::RollbackFloorAboveCeiling { .. } => {
            explained(ModelRefusalReason::FloorAboveCeiling)
        }
        // 取号之后那一串自己的落点在取号之前就取不到：说的是分配器那一条原因（每块盘上都没有 = 单元区墙）。
        MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided {
            refusal, ..
        } => refusal_reason_of_placement_refusal(refusal),
        // 取号之前空间准入不够（实例切换的预留拿不到）：同发布那一条，是模型的单元区墙。
        MountError::SpaceAdmissionRefusedBeforeAcquisition { .. } => {
            explained(ModelRefusalReason::UnitAreaWall)
        }
        // 树表 0 条、而实例表已经不是 mkfs 那一片：零故障走得到（写过行的那一版上再挂载一次），模型照代码今天的读法划进必须拒。
        MountError::VersionWithoutFileNotWrittenByMakeFilesystem { .. } => {
            explained(ModelRefusalReason::VersionWithoutFileNotWrittenByMakeFilesystem)
        }
        // 读阶段判出系统配置见证过比所选那一版新的发布、重读一次仍判真（C554 乙）：崩溃恢复抛弃根那一步把最新那条根读成全 0 就走到，
        // 模型那一步答必须拒（`IdealModel::answer_mount_writable_with_the_newest_root_unreadable`）。
        MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable) => {
            match **still_unreadable {
                StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion { .. } => {
                    explained(
                        ModelRefusalReason::NewerPublishWitnessedBySystemConfigurationStillUnreadableAfterOneReread,
                    )
                }
                // 重建分配器时最新那条根的实例表重读仍读不出（代码审阅第 22 条）：读阶段判完、判据为假之后才读它，崩溃恢复抛弃根
                // 那一步在读阶段就拒了走不到；只在读路径上注入故障、正好落在那一片上才有，模型没有这一条。
                StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheShadowLedger { .. }
                // 挂着时抬 F 算 F 生效值读最新那条根的实例表、重算影子账读根环里这个进程知道住着根的槽，重读一次仍读坏（实审 A3b Q4，
                // 改之前经 `MountError::Recovery` 交出、同样记成没理由）：只在读路径上注入故障才有，模型没有这一条。
                | StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheEffectiveFloor { .. }
                | StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot { .. }
                | StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot { .. } => {
                    ObservedRefusalReason::Unexplained
                }
            }
        }
        // 恢复失败、记录读不出、表解不开、取号失败、坏盘上才有的根、判定与取号之间号变了、有盘不带所选那一版（空盘、停在旧状态）、
        // 交进来的盘少于 w 的下限（随机历史每次都交整池两块盘）、抬 F 先写系统配置那一步的块设备错、要抬到的 F 低于盘上的生效值
        // （同一个进程里上一次先写系统配置只写进一部分盘才有）、算抬 F 上限时一个知道住着根的根环槽读坏又重读仍坏（读路径上注入故障才有）：
        // 健康的内存盘上都不该出现。
        MountError::Recovery(_)
        | MountError::FileVersionWithoutAnyJournalRecord
        | MountError::InstanceTableMalformed
        | MountError::Acquisition(_)
        | MountError::FormatTimeUnitLocationsOnDifferentSlots { .. }
        | MountError::RollbackFloorCeilingNeedsUnreadableValidRootTreeTable { .. }
        | MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread { .. }
        | MountError::InstanceGenerationChangedBeforeAcquisition { .. }
        | MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion { .. }
        | MountError::WritableDeviceCountBelowTheStripeWidthLowerBound { .. }
        | MountError::RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot(_)
        | MountError::RequestedFloorBelowTheEffectiveFloorWhoseRaiseIsUndecided { .. }
        // 盘表里同一个身份交了几次、调用方参数与盘上系统配置不一致、有盘落后于现行那一版又缺它的单元：随机历史每次都交整池两块盘
        // （没换过盘）、用建池的那一份参数，走不到。
        | MountError::DeviceIdentitiesHandedInMoreThanOnce { .. }
        | MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration { .. }
        | MountError::DevicesBehindTheCurrentVersionAndMissingItsUnits { .. }
        // 盘上的实例代号或 txg 已到顶：坏镜像才有。
        | MountError::SequenceNumberPastTheTopOfItsRange(_) => ObservedRefusalReason::Unexplained,
    }
}
```

### crates/singlefs-harness/src/model_comparison.rs:841-876（root_ring_slot_still_bad_after_one_reread_of_rollback_error）

```rust
/// 挂着时回退判候选集读根环，一个知道住着根的根环槽读坏、重读仍坏
/// （`RollbackError::CandidateJudgementStillUnreadableAfterOneReread` 装着 `RootRingSlotKnownToHoldARoot`）时实现点名的那个槽；别的成员 None。
#[must_use]
pub fn root_ring_slot_still_bad_after_one_reread_of_rollback_error(
    error: &RollbackError,
) -> Option<ModelRingPosition> {
    match error {
        RollbackError::CandidateJudgementStillUnreadableAfterOneReread(still_unreadable) => {
            match **still_unreadable {
                StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot {
                    ring_slot, ..
                } => Some(model_ring_position(ring_slot)),
                StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion { .. }
                | StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheShadowLedger { .. }
                | StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheEffectiveFloor { .. }
                | StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot { .. } => None,
            }
        }
        RollbackError::PublishFrozenAfterAWriteFailureIsNotResentYet { .. }
        | RollbackError::Recovery(_)
        | RollbackError::CallerInputsDisagreeWithTheDisk(_)
        | RollbackError::NextCheckpointTxgPastTheTopOfItsRange { .. }
        | RollbackError::CurrentInstanceTableMalformed
        | RollbackError::TargetNotACandidate { .. }
        | RollbackError::TargetVersionUnreadable { .. }
        | RollbackError::CurrentAccountUnreadable { .. }
        | RollbackError::TargetTreeIdentifiersDifferFromTheCurrentVersionWhoseHandlingIsUndecided {
            ..
        }
        | RollbackError::UserVisibleUnitWithoutItsRecordInTheCurrentAccount { .. }
        | RollbackError::UserVisibleUnitStillAllocatedUnderAnotherGeneration { .. }
        | RollbackError::ResurrectedUnitCopyUnreadableOrMismatched { .. }
        | RollbackError::CurrentVersionUnitNotReleasable { .. }
        | RollbackError::Publish(_) => None,
    }
}
```
## 二、改钉的用例与造盘面的辅助函数（正文第二节 `tests/common/mod.rs`、孤记录那一条与「改钉的用例」那几份）

### crates/singlefs-harness/tests/common/mod.rs:427-548（abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before）

```rust
/// 崩溃恢复造出一条被抛弃的根（第一轮判决 H6 那一形；D23（journal 的角色与格式） 已定项 14 射程：影子账与按实例表判抛弃留着，
/// 理由是崩溃恢复）：进程退出之后，`newest`（根环里最新的那条根那一版）的根槽与它的数据单元（两份）暂时读不出（暂存、清零），
/// 它那次发布之后轮换写的系统配置槽（每块盘一槽）坏掉、系统配置没见证到它（清零、不写回；见证在时 C554 乙拒可写，造不出被抛弃的根），
/// 重开可写挂载——择根落到它前一条根，它那条记录施加前验点名单元失败、不施加；新实例写行与暖机。再把暂存的字节原样写回：
/// `newest` 的根又读得出，按新实例的实例表判是被抛弃的。池换成那次挂载交回的分配器与现行版本（要带文件），交回那次挂载——
/// 它看不见 `newest`，影子账隔离 0；之后的挂载才看得见。
pub fn abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before(
    pool: &mut BuiltPool,
    newest: &TransactionOutput,
) -> singlefs_core::mount::Mounted {
    use singlefs_core::block_device::{BlockDevice, WriteDurability};
    let publish_parameters = parameters();
    let target = singlefs_core::root_ring::target_for_publish(
        newest.root.checkpoint_txg,
        publish_parameters.geometry.root_ring_slots_per_region,
    );
    let root_slot_device =
        publish_parameters.region_devices[usize::try_from(target.region).expect("区域号")];
    let root_slot_offset = singlefs_core::root_ring::slot_offset(
        target,
        publish_parameters.geometry.fixed_structure_slot_spacing,
    );
    let root_slot_bytes =
        usize::try_from(publish_parameters.geometry.physical_block_size).expect("根槽宽");
    let data_unit_bytes = usize::try_from(singlefs_format::DATA_UNIT_BYTES).expect("32768");
    fn index_of_device(devices: &[(DeviceIdentity, Recorded)], identity: DeviceIdentity) -> usize {
        devices
            .iter()
            .position(|(candidate, _)| *candidate == identity)
            .expect("池里有这块盘")
    }
    let mut devices = pool.reopen_recorded();
    let places_to_hide = std::iter::once((root_slot_device, root_slot_offset, root_slot_bytes))
        .chain(
            newest
                .data_pointers
                .iter()
                .flat_map(|pointer| pointer.locations)
                .map(|location| {
                    (
                        location.device,
                        location.slot.to_device_offset(),
                        data_unit_bytes,
                    )
                }),
        );
    let mut saved: Vec<(
        DeviceIdentity,
        singlefs_core::address::DeviceOffsetInBytes,
        Vec<u8>,
    )> = Vec::new();
    for (identity, offset, length) in places_to_hide {
        let index = index_of_device(&devices, identity);
        let mut bytes = vec![0u8; length];
        devices[index].1.read_at(offset, &mut bytes).expect("暂存");
        devices[index]
            .1
            .write_at(offset, &vec![0u8; length], WriteDurability::Plain)
            .expect("清零");
        saved.push((identity, offset, bytes));
    }
    // 系统配置没见证到 `newest`：C554 乙之后崩溃恢复还抛弃得了最新那条根的只剩这一形（系统配置在根槽 FUA 之后才轮换，D16（发布语义） 已定项 7；
    // 见证在时可写挂载重读一次仍读不出它就拒可写，`MountError::NewerStateStillUnreadableAfterOneReread`）：崩在 `newest` 的根槽 FUA 之后、
    // 它那次系统配置轮换写之前。轮换写那一槽（每块盘世代号最大的那一槽，槽号 = 世代号 mod 2）因此还是再前一次轮换留下的内容：
    // 世代号 g − 2、自证照过。录制流只记内容哈希、拿不回那份字节，这里拿另一槽（世代号 g − 1，见证的是 `newest` 前一次发布）的内容改世代号为 g − 2
    // 重新编码写回——见证值与那份真内容一样停在 `newest` 之前，两槽都自证得过。不清零：C331（择根倒挂压过已确认的写） 取甲之后
    // 见证读有一槽读不出或自证不过就拒可写（`WitnessedCounterComparison::SomeSystemConfigurationSlotUnreadOrUnverified`），清零造的是撕裂轮换那一形。
    let slot_spacing_in_bytes = u64::from(publish_parameters.geometry.fixed_structure_slot_spacing);
    let rotation_not_written_of_each_device: Vec<(
        singlefs_core::address::DeviceOffsetInBytes,
        Vec<u8>,
    )> = devices
        .iter()
        .map(|(identity, _)| {
            let mut verified_slots = singlefs_core::recovery::verified_system_configuration_slots(
                devices.as_slice(),
                *identity,
                slot_spacing_in_bytes,
                &publish_parameters.filesystem_identifier,
            );
            verified_slots.sort_by_key(|slot| slot.quantities.slot_generation);
            let [older, newest_slot] =
                <[_; 2]>::try_from(verified_slots).expect("造被抛弃的根之前每块盘两槽都自证得过");
            let newest_generation = newest_slot.quantities.slot_generation;
            let mut rotation_not_written = older;
            rotation_not_written.quantities.slot_generation = newest_generation
                .checked_sub(2)
                .expect("`newest` 之前至少还有两次轮换");
            (
                singlefs_core::address::DeviceOffsetInBytes(
                    (newest_generation % singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE)
                        * slot_spacing_in_bytes,
                ),
                rotation_not_written.to_slot(),
            )
        })
        .collect();
    for ((_, device), (newest_slot, rotation_not_written)) in
        devices.iter_mut().zip(rotation_not_written_of_each_device)
    {
        device
            .write_at(newest_slot, &rotation_not_written, WriteDurability::Plain)
            .expect("见证 newest 的系统配置槽写回成轮换之前的那一代");
    }
    let mounted = singlefs_core::mount::mount_writable(&publish_parameters, &mut devices)
        .expect("最新那条根与它的数据单元读不出：择根落到前一条根，照常可写挂载");
    for (identity, offset, bytes) in &saved {
        let index = index_of_device(&devices, *identity);
        devices[index]
            .1
            .write_at(*offset, bytes, WriteDurability::Plain)
            .expect("原样写回");
    }
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator.clone();
    pool.output = mounted
        .current
        .file_version()
        .expect("落到的那一版带文件")
        .clone();
    mounted
}
```

### crates/singlefs-harness/tests/common/mod.rs:812-831（system_configuration_slots_of_every_device）

```rust
/// 每块盘两个系统配置槽的全部字节，按 (盘在盘表里的位置, 偏移) 存。
pub fn system_configuration_slots_of_every_device(
    pool: &BuiltPool,
) -> Vec<(usize, singlefs_core::address::DeviceOffsetInBytes, Vec<u8>)> {
    use singlefs_core::block_device::BlockDevice;
    let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    let slot_bytes =
        usize::try_from(singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let devices = pool.devices.as_ref().expect("镜像开着");
    let mut saved = Vec::new();
    for (index, (_, device)) in devices.iter().enumerate() {
        for slot in 0..singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE {
            let offset = singlefs_core::address::DeviceOffsetInBytes(slot * spacing);
            let mut bytes = vec![0u8; slot_bytes];
            device.read_at(offset, &mut bytes).expect("读系统配置槽");
            saved.push((index, offset, bytes));
        }
    }
    saved
}
```

### crates/singlefs-harness/tests/common/mod.rs:833-928（abandon_the_third_version_by_a_crash_before_its_rotation）

```rust
/// 崩溃恢复造出一条被抛弃的根的另一形（收口表第 43 行调查员的最小复现，`research/prompts/closeout-recheck-2026-09-28/row43-investigator-report.md`）：
/// `third` 的根槽 FUA 持久、系统配置轮换没持久就崩（`system_configuration_before_the_third` 是它那次发布之前每块盘两槽的字节，由
/// [`system_configuration_slots_of_every_device`] 取）；下一次可写挂载时 C 的根槽与数据单元一时读不出（挂载交回之后原样写回）：择根落到 B，
/// C 那条记录施加前验点名单元失败、不施加，新实例 2 写行、暖机。池换成那次挂载交回的分配器与现行版本。
pub fn abandon_the_third_version_by_a_crash_before_its_rotation(
    pool: &mut BuiltPool,
    third: &TransactionOutput,
    system_configuration_before_the_third: &[(
        usize,
        singlefs_core::address::DeviceOffsetInBytes,
        Vec<u8>,
    )],
) {
    use singlefs_core::block_device::BlockDevice;
    let publish_parameters = parameters();
    let target = singlefs_core::root_ring::target_for_publish(
        third.root.checkpoint_txg,
        publish_parameters.geometry.root_ring_slots_per_region,
    );
    let root_slot_device =
        publish_parameters.region_devices[usize::try_from(target.region).expect("区域号")];
    let root_slot_offset = singlefs_core::root_ring::slot_offset(
        target,
        publish_parameters.geometry.fixed_structure_slot_spacing,
    );
    let root_slot_bytes =
        usize::try_from(publish_parameters.geometry.physical_block_size).expect("根槽宽");
    let data_unit_bytes = usize::try_from(singlefs_format::DATA_UNIT_BYTES).expect("32768");
    let mut devices = pool.reopen_recorded();
    for (index, offset, bytes) in system_configuration_before_the_third {
        devices[*index]
            .1
            .write_at(
                *offset,
                bytes,
                singlefs_core::block_device::WriteDurability::Plain,
            )
            .expect("C 的轮换没落盘：退回轮换之前的字节");
    }
    let places_to_hide = std::iter::once((root_slot_device, root_slot_offset, root_slot_bytes))
        .chain(
            third
                .data_pointers
                .iter()
                .flat_map(|pointer| pointer.locations)
                .map(|location| {
                    (
                        location.device,
                        location.slot.to_device_offset(),
                        data_unit_bytes,
                    )
                }),
        );
    let mut hidden = Vec::new();
    for (identity, offset, length) in places_to_hide {
        let index = devices
            .iter()
            .position(|(candidate, _)| *candidate == identity)
            .expect("池里有这块盘");
        let mut bytes = vec![0u8; length];
        devices[index].1.read_at(offset, &mut bytes).expect("暂存");
        devices[index]
            .1
            .write_at(
                offset,
                &vec![0u8; length],
                singlefs_core::block_device::WriteDurability::Plain,
            )
            .expect("一时读不出：清零");
        hidden.push((index, offset, bytes));
    }
    let mounted = singlefs_core::mount::mount_writable(&publish_parameters, &mut devices);
    for (index, offset, bytes) in &hidden {
        devices[*index]
            .1
            .write_at(
                *offset,
                bytes,
                singlefs_core::block_device::WriteDurability::Plain,
            )
            .expect("原样写回");
    }
    let mounted = mounted.expect("择根落到 B，照常可写挂载");
    assert_eq!(
        mounted.output.effective_root.checkpoint_txg,
        singlefs_core::address::CheckpointTxg(4),
        "恢复落到 B"
    );
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator.clone();
    pool.output = mounted
        .current
        .file_version()
        .expect("落到的那一版带文件")
        .clone();
}
```

### crates/singlefs-harness/tests/acquisition_writes_the_witnessed_journal_tail_so_two_crashed_acquisitions_still_witness_the_newest_publish.rs:343-435（a_later_acquisition_writes_the_largest_tail_of_the_pool_while_one_device_lags_one_rotation）

```rust
/// 新池新建文件之后每块盘两槽：世代 4（暖机第二次，tail 2）、世代 5（新池新建文件，tail 3）。盘 `damaged_device` 世代 5 那一槽退回轮换之前
/// （崩在新池新建文件那次轮换写完另一块盘、没写到这一块：这一槽还是世代 3 的内容，这里拿世代 4 那一槽改世代号为 3 重新编码写回，两槽都自证得过），
/// 池里最大的 tail 3 只在另一块盘上读得出：第二次取号写进每块盘的 tail 都是 3——取整池的最大值，不是各盘自己的，也不是最小的。
/// 不清零：C331（择根倒挂压过已确认的写） 取甲之后取号那一刻有一槽读不出就拒（见 `an_acquisition_refuses_a_device_that_lost_only_its_newest_slot`）。
fn a_later_acquisition_writes_the_largest_tail_of_the_pool_while_one_device_lags_one_rotation(
    tag: &str,
    damaged_device: DeviceIdentity,
) {
    let mut pool = build_pool(tag);
    let new_pool_file_creation_counter = pool.output.record.counter;
    assert_eq!(
        new_pool_file_creation_counter, 3,
        "暖机 jsn 1、2，新池新建文件 jsn 3"
    );
    let slot_spacing_in_bytes = u64::from(geometry().fixed_structure_slot_spacing);
    {
        let devices: &mut Vec<(DeviceIdentity, Recorded)> =
            pool.devices.as_mut().expect("新池新建文件写完，盘还开着");
        let mut generation_three = verified_system_configuration_slots(
            devices.as_slice(),
            damaged_device,
            slot_spacing_in_bytes,
            &parameters().filesystem_identifier,
        )
        .into_iter()
        .find(|slot| slot.quantities.slot_generation == 4)
        .expect("世代 4 那一槽自证得过");
        generation_three.quantities.slot_generation = 3;
        let (_, device) = devices
            .iter_mut()
            .find(|(identity, _)| *identity == damaged_device)
            .expect("池里有这块盘");
        device
            .write_at(
                DeviceOffsetInBytes(
                    (5 % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE) * slot_spacing_in_bytes,
                ),
                &generation_three.to_slot(),
                WriteDurability::Plain,
            )
            .expect("世代 5 那一槽退回世代 3");
    }
    let before_the_acquisition = pool.memory_pool();
    let tails_on_disk: Vec<(DeviceIdentity, Vec<u64>)> = DISKS
        .into_iter()
        .map(|disk| {
            (
                disk,
                slot_readings_of(&before_the_acquisition, disk)
                    .into_iter()
                    .map(|reading| reading.journal_tail)
                    .collect(),
            )
        })
        .collect();
    assert_eq!(
        tails_on_disk,
        DISKS
            .into_iter()
            .map(|disk| {
                if disk == damaged_device {
                    (disk, vec![2, 2])
                } else {
                    (disk, vec![2, new_pool_file_creation_counter])
                }
            })
            .collect::<Vec<_>>(),
        "落后一次轮换的那块盘两槽 tail 都是 2（世代 3、4），另一块两槽 tail 2、3"
    );
    let largest_tail_of_the_pool = tails_on_disk
        .iter()
        .flat_map(|(_, tails)| tails.iter().copied())
        .max()
        .expect("池里有自证过的系统配置");
    assert_eq!(largest_tail_of_the_pool, new_pool_file_creation_counter);

    assert_eq!(acquire_once_then_crash(&mut pool), InstanceGeneration(2));
    let after_the_acquisition = pool.memory_pool();
    for disk in DISKS {
        let acquisition_writes: Vec<SlotReading> = slot_readings_of(&after_the_acquisition, disk)
            .into_iter()
            .filter(|reading| reading.journal_instance == InstanceGeneration(2))
            .collect();
        assert_eq!(
            acquisition_writes
                .iter()
                .map(|reading| reading.journal_tail)
                .collect::<Vec<_>>(),
            vec![largest_tail_of_the_pool],
            "{disk:?}：一份取号写，tail 是取号前池里两槽最大的 tail {largest_tail_of_the_pool}：{acquisition_writes:?}"
        );
    }
}
```

### crates/singlefs-harness/tests/acquisition_writes_the_witnessed_journal_tail_so_two_crashed_acquisitions_still_witness_the_newest_publish.rs:437-444（a_later_acquisition_writes_the_largest_tail_of_the_pool_into_every_device_when_the_first_device_lags_one_rotation）

```rust
#[test]
fn a_later_acquisition_writes_the_largest_tail_of_the_pool_into_every_device_when_the_first_device_lags_one_rotation(
) {
    a_later_acquisition_writes_the_largest_tail_of_the_pool_while_one_device_lags_one_rotation(
        "c554-yi-carry-device-zero-damaged",
        DeviceIdentity(0),
    );
}
```

### crates/singlefs-harness/tests/acquisition_writes_the_witnessed_journal_tail_so_two_crashed_acquisitions_still_witness_the_newest_publish.rs:446-453（a_later_acquisition_writes_the_largest_tail_of_the_pool_into_every_device_when_the_second_device_lags_one_rotation）

```rust
#[test]
fn a_later_acquisition_writes_the_largest_tail_of_the_pool_into_every_device_when_the_second_device_lags_one_rotation(
) {
    a_later_acquisition_writes_the_largest_tail_of_the_pool_while_one_device_lags_one_rotation(
        "c554-yi-carry-device-one-damaged",
        DeviceIdentity(1),
    );
}
```

### crates/singlefs-harness/tests/acquisition_writes_the_witnessed_journal_tail_so_two_crashed_acquisitions_still_witness_the_newest_publish.rs:455-529（a_rolled_back_acquisition_writes_the_witnessed_tail_back_instead_of_zero）

```rust
/// 取号写在盘 1 上报错：盘 0 那份已写出、回卷成旧代号 1。回卷是「像没取过号」，回卷写的 tail 与取号写同是取号那一刻读到的见证值 3，
/// 不退回 0；盘 1 一个字节都没写成。
#[test]
fn a_rolled_back_acquisition_writes_the_witnessed_tail_back_instead_of_zero() {
    /// 系统配置两槽住在偏移 0 与 4096，都在这个界之下。
    const SYSTEM_CONFIGURATION_SLOTS_END_OFFSET: u64 = 8192;
    let mut built = build_pool("c554-yi-carry-rolled-back");
    let new_pool_file_creation_counter = built.output.record.counter;
    let plan = SharedFaultPlan::unarmed(geometry());
    let mut devices: Vec<(DeviceIdentity, FaultInjectingBlockDevice<Recorded>)> = built
        .devices
        .take()
        .expect("新池新建文件写完，盘还开着")
        .into_iter()
        .map(|(identity, inner)| {
            (
                identity,
                FaultInjectingBlockDevice::new(identity, inner, plan.clone()),
            )
        })
        .collect();
    plan.arm(FaultSchedule {
        fault: InjectedFault::WriteFails,
        device: FaultDeviceSelector::OnlyDevice(DeviceIdentity(1)),
        placement: FaultPlacement::OffsetBelow(SYSTEM_CONFIGURATION_SLOTS_END_OFFSET),
        counting: FaultCounting::AcrossThePool,
        occurrence: FaultOccurrence::EVERY_MATCHING_CALL,
    });
    let failure = {
        let pool_parameters = parameters();
        let mut writer = PoolWriter::new(&pool_parameters, devices.as_mut_slice());
        match acquire_instance(&mut writer).expect_err("盘 1 的取号写报错，取号必须失败") {
            InstanceAcquisitionFailed::Acquisition(acquisition) => acquisition,
            InstanceAcquisitionFailed::DeviceWithAnUnreadOrUnverifiedSystemConfigurationSlotWhenReadingTheWitness {
                device,
            } => panic!("这条用例里每块盘两槽都读得出自证过的系统配置，取号不该拒在见证值那一核：{device:?}"),
        }
    };
    assert!(
        matches!(failure.rollback, AcquisitionRollback::RolledBack),
        "盘 0 那份已写出，要回卷：{failure:?}"
    );
    assert_eq!(
        slot_readings_of(devices.as_slice(), DeviceIdentity(0)),
        vec![
            SlotReading {
                slot_generation: 6,
                journal_instance: InstanceGeneration(2),
                journal_tail: new_pool_file_creation_counter,
            },
            SlotReading {
                slot_generation: 7,
                journal_instance: InstanceGeneration(1),
                journal_tail: new_pool_file_creation_counter,
            },
        ],
        "盘 0：取号写世代 6 带新号 2，回卷写世代 7 带旧号 1；两写的 tail 都是取号前的见证值 {new_pool_file_creation_counter}"
    );
    assert_eq!(
        slot_readings_of(devices.as_slice(), DeviceIdentity(1)),
        vec![
            SlotReading {
                slot_generation: 4,
                journal_instance: InstanceGeneration(1),
                journal_tail: 2,
            },
            SlotReading {
                slot_generation: 5,
                journal_instance: InstanceGeneration(1),
                journal_tail: new_pool_file_creation_counter,
            },
        ],
        "盘 1 一个字节都没写成"
    );
}
```

### crates/singlefs-harness/tests/acquisition_writes_the_witnessed_journal_tail_so_two_crashed_acquisitions_still_witness_the_newest_publish.rs:531-580（an_acquisition_refuses_a_device_that_lost_only_its_newest_slot）

```rust
/// C331（择根倒挂压过已确认的写） 取甲（用户 2026-09-28 定；`research/prompts/unreadable-at-mount-r2-main-verification.md` L2 V4 那一支）：
/// 取号那一刻读见证值，某块盘两槽里有一槽读不出或自证不过（这里清零世代 5 那一槽，另一槽照样自证得过），就在第一个取号写之前拒，
/// 两块盘的系统配置槽逐字节不变。改之前（只在一份都没有时拒）这块盘照样取号、见证值缺了那一槽。
fn an_acquisition_refuses_a_device_that_lost_only_its_newest_slot(tag: &str, damaged_device: DeviceIdentity) {
    let mut pool = build_pool(tag);
    let slot_spacing_in_bytes = u64::from(geometry().fixed_structure_slot_spacing);
    {
        let devices: &mut Vec<(DeviceIdentity, Recorded)> =
            pool.devices.as_mut().expect("新池新建文件写完，盘还开着");
        let (_, device) = devices
            .iter_mut()
            .find(|(identity, _)| *identity == damaged_device)
            .expect("池里有这块盘");
        device
            .write_at(
                DeviceOffsetInBytes(
                    (5 % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE) * slot_spacing_in_bytes,
                ),
                &vec![0u8; usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096")],
                WriteDurability::Plain,
            )
            .expect("世代 5 那一槽清零");
    }
    let before_the_acquisition = pool.memory_pool();
    let slots_before: Vec<Vec<SlotReading>> = DISKS
        .into_iter()
        .map(|disk| slot_readings_of(&before_the_acquisition, disk))
        .collect();
    let pool_parameters = parameters();
    let mut devices = pool.reopen_recorded();
    let refusal = {
        let mut writer = PoolWriter::new(&pool_parameters, devices.as_mut_slice());
        acquire_instance(&mut writer).expect_err("有一槽读不出：取号在任何写之前拒")
    };
    pool.devices = Some(devices);
    assert!(
        matches!(
            refusal,
            InstanceAcquisitionFailed::DeviceWithAnUnreadOrUnverifiedSystemConfigurationSlotWhenReadingTheWitness { device }
                if device == damaged_device
        ),
        "拒因点名坏了一槽的那块盘：{refusal:?}"
    );
    let after_the_refusal = pool.memory_pool();
    let slots_after: Vec<Vec<SlotReading>> = DISKS
        .into_iter()
        .map(|disk| slot_readings_of(&after_the_refusal, disk))
        .collect();
    assert_eq!(slots_after, slots_before, "拒的时候一个取号写都没写");
}
```

### crates/singlefs-harness/tests/acquisition_writes_the_witnessed_journal_tail_so_two_crashed_acquisitions_still_witness_the_newest_publish.rs:582-588（an_acquisition_refuses_the_first_device_that_lost_only_its_newest_slot_before_writing_anything）

```rust
#[test]
fn an_acquisition_refuses_the_first_device_that_lost_only_its_newest_slot_before_writing_anything() {
    an_acquisition_refuses_a_device_that_lost_only_its_newest_slot(
        "c331-jia-acquisition-device-zero",
        DeviceIdentity(0),
    );
}
```

### crates/singlefs-harness/tests/acquisition_writes_the_witnessed_journal_tail_so_two_crashed_acquisitions_still_witness_the_newest_publish.rs:590-596（an_acquisition_refuses_the_second_device_that_lost_only_its_newest_slot_before_writing_anything）

```rust
#[test]
fn an_acquisition_refuses_the_second_device_that_lost_only_its_newest_slot_before_writing_anything() {
    an_acquisition_refuses_a_device_that_lost_only_its_newest_slot(
        "c331-jia-acquisition-device-one",
        DeviceIdentity(1),
    );
}
```

### crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs:1088-1112（torn_tree_table_of_an_abandoned_root_is_counted_and_does_not_fail_the_mount）

```rust
/// 被抛弃根 C 的树表单元在两块盘上都改坏：挂载照样成功（步 4 / 步 5 代码三方第二轮云端攻方腿打中：一条被抛弃根的树表撕裂不能让每次挂载都失败），
/// 计数一条读不出的被抛弃根往上报，C 引用的落点从中央映射现算，只被 C 引用的 14 个槽照样隔离（与 C 读得出时同，见上一条用例；
/// C393（被抛弃根的账读不出时修复没有条款） 取 (b)-从映射现算，用户 2026-09-28 定）。
#[test]
#[ignore = "harness 耗时用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn torn_tree_table_of_an_abandoned_root_is_counted_and_does_not_fail_the_mount() {
    let ThroughTheThirdPublish {
        mut pool, third, ..
    } = build_through_third_publish("step-four-torn-abandoned");
    abandon_the_third_version_by_a_recovery_that_lands_on_the_second_warm_up(&mut pool, &third);
    let mut devices = pool.reopen_recorded();
    corrupt_unit_copies(
        &mut devices,
        &third.root.tree_table.locations,
        usize::try_from(singlefs_format::NODE_BYTES).expect("16384"),
    );
    let remounted =
        mount_writable(&parameters(), &mut devices).expect("被抛弃根的树表撕裂不拒绝挂载");
    assert_eq!(remounted.output.abandoned_roots_unreadable, 1, "C 读不出");
    assert_eq!(
        remounted.output.isolated_slots_per_device,
        vec![(DeviceIdentity(0), 14), (DeviceIdentity(1), 14)],
        "从中央映射现算，只被 C 引用的 14 个槽照样隔离（与 C 读得出时同，见上一条用例）"
    );
}
```

### crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs:1114-1222（an_abandoned_roots_allocation_record_whose_span_runs_past_the_unit_area_is_counted_and_does_not_panic）

```rust
/// 被抛弃根 C 那棵账最左那片叶的第一条分配记录改成「起点贴着单元区末尾、跨度 32767 槽」，链上校验和逐道重算（panic 面普查 R7）：
/// 挂载不 panic，C 计成一条读不出的被抛弃根，引用的落点从中央映射现算，只被 C 引用的槽照样隔离。
#[test]
#[ignore = "harness 耗时用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn an_abandoned_roots_allocation_record_whose_span_runs_past_the_unit_area_is_counted_and_does_not_panic(
) {
    let ThroughTheThirdPublish {
        mut pool, third, ..
    } = build_through_third_publish("step-four-abandoned-span");
    abandon_the_third_version_by_a_recovery_that_lands_on_the_second_warm_up(&mut pool, &third);
    let mut devices = pool.reopen_recorded();
    let node_bytes = usize::try_from(singlefs_format::NODE_BYTES).expect("16384");
    let allocation_tree_pointer = third
        .tree_table_entries
        .iter()
        .find(|entry| entry.kind == TREE_KIND_ALLOCATION)
        .expect("C 的树表里有分配记录树")
        .root;
    let lowest_leaf_of_device_zero = third
        .allocation_record_tree
        .nodes
        .iter()
        .filter_map(|(node, _)| match node {
            AllocationRecordTreeNode::BelowTheRoot(position)
                if position.level == 0 && position.device == DeviceIdentity(0) =>
            {
                Some(*node)
            }
            AllocationRecordTreeNode::BelowTheRoot(_) | AllocationRecordTreeNode::Root => None,
        })
        .min()
        .expect("盘 0 上至少一片装着记录的叶");
    let path_pointers: Vec<_> = [
        AllocationRecordTreeNode::Root,
        AllocationRecordTreeNode::BelowTheRoot(AllocationRecordTreeNodePosition {
            level: 1,
            device: DeviceIdentity(0),
            index_in_device: 0,
        }),
        lowest_leaf_of_device_zero,
    ]
    .into_iter()
    .map(|node| {
        third
            .allocation_record_tree
            .pointer_of(node)
            .expect("C 那棵树里有这个节点")
    })
    .collect();
    let root_slot_position = root_slot_position_of(third.root.checkpoint_txg.0);
    let mut root_slot = read_root_slot(&mut devices, root_slot_position);
    let mut tree_table_node =
        read_unit_at(&mut devices, third.root.tree_table.locations[0], node_bytes);
    let mut allocation_tree_nodes_from_the_root: Vec<Vec<u8>> = path_pointers
        .iter()
        .map(|pointer| read_unit_at(&mut devices, pointer.locations[0], node_bytes))
        .collect();
    let unit_area_end_slot = UNIT_AREA_START_SLOT + unit_area_slots_of_device(IMAGE_BYTES);
    let what = move_the_first_allocation_record_past_the_end_of_the_unit_area_and_reseal_the_chain(
        &mut root_slot,
        &mut tree_table_node,
        &mut allocation_tree_nodes_from_the_root,
        unit_area_end_slot,
    )
    .expect("C 那棵账最左那片叶装着记录");
    for (pointer, bytes) in path_pointers
        .iter()
        .zip(&allocation_tree_nodes_from_the_root)
    {
        write_unit_to_every_location(&mut devices, pointer.locations, bytes);
    }
    write_unit_to_every_location(
        &mut devices,
        third.root.tree_table.locations,
        &tree_table_node,
    );
    write_root_slot(&mut devices, root_slot_position, &root_slot);
    let system_configuration = choose_system_configuration(&devices).expect("系统配置");
    let abandoned_root_on_disk = readable_roots(
        &devices,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    )
    .into_iter()
    .find(|root| {
        root.instance == third.root.instance && root.checkpoint_txg == third.root.checkpoint_txg
    })
    .expect("C 的根槽自证校验和重算过，仍然读得出");
    let ledger = allocation_records_under_root(&devices, &abandoned_root_on_disk);
    assert!(
        matches!(
            ledger,
            Err(RecoveryFailure::AllocationRecordOutsideThePoolGeometry {
                what: "分配记录不在它所在叶按位置罩的那一段里，或末槽越过叶的末槽"
            })
        ),
        "C 那棵账该报「不在它所在叶按位置罩的那一段里」，实际交回的是 {ledger:?}（{what}；分配记录树根在槽 {:?}）",
        allocation_tree_pointer.locations.map(|location| location.slot)
    );
    let remounted = mount_writable(&parameters(), &mut devices)
        .unwrap_or_else(|error| panic!("被抛弃根那棵账里的坏记录不拒绝挂载：{error:?}（{what}）"));
    assert_eq!(remounted.output.abandoned_roots_unreadable, 1, "{what}");
    assert_eq!(
        remounted.output.isolated_slots_per_device,
        vec![(DeviceIdentity(0), 14), (DeviceIdentity(1), 14)],
        "账解不开也从中央映射现算，只被 C 引用的 14 个槽照样隔离，与树表撕裂那一条同一个数（C393（被抛弃根的账读不出时修复没有条款） 取 (b)-从映射现算，用户 2026-09-28 定）"
    );
}
```

### crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs:555-603（an_abandoned_root_whose_instance_table_pointer_sits_below_the_unit_area_refuses_the_writable_mount_instead_of_panicking）

```rust
/// 场景：第一个文件那一版之后重开一次（实例 2 写行 (1, 3, W)、暖机），再往根环一个空槽里放一条实例 1、txg 4 的根——照抄实例 1 暖机那条
/// 树表 0 条的根、只改 txg 与实例表指针：实例表指针两条位置条目都指槽 100（单元区起点之下）。按最新那条根的实例表，它是被抛弃的根。
/// 预期：这条根的账算「解不开」，它是树表 0 条的一版、从中央映射也现算不成，第二次重开在取号之前拒可写、报出这条根（C393（被抛弃根的账读不出时修复没有条款） 取 (b)-从映射现算，用户 2026-09-28 定），不 panic。
/// 改之前影子账只判了两条位置条目同槽，槽 100 进 `isolate_abandoned`，在 `DeviceFreeMap::index` 的 expect 上 panic。
#[test]
fn an_abandoned_root_whose_instance_table_pointer_sits_below_the_unit_area_refuses_the_writable_mount_instead_of_panicking(
) {
    let (mut devices, first) = pool_after_the_first_file();
    let first_remount =
        mount_writable(&parameters(), &mut devices).expect("第一次重开：实例 2 写行、暖机");
    assert_eq!(first_remount.output.instance, InstanceGeneration(2));
    let roots = roots_in_the_ring(&devices);
    let (_, warm_up_root_of_instance_one) = roots
        .iter()
        .find(|(_, root)| {
            root.instance == first.root.instance && root.checkpoint_txg == CheckpointTxg(1)
        })
        .copied()
        .expect("实例 1 暖机那条 txg 1 的根还在环里");
    let occupied: Vec<RootRingSlot> = roots.iter().map(|(ring_slot, _)| *ring_slot).collect();
    let system_configuration = system_configuration_of(&devices);
    let empty_ring_slot = every_root_ring_slot(&system_configuration.immutable.sizes)
        .into_iter()
        .rev()
        .find(|ring_slot| !occupied.contains(ring_slot))
        .expect("根环里还有空槽");
    let mut abandoned = warm_up_root_of_instance_one;
    abandoned.checkpoint_txg = CheckpointTxg(first.root.checkpoint_txg.0 + 1);
    for location in &mut abandoned.instance_table.locations {
        location.slot = SLOT_BELOW_THE_UNIT_AREA_NOBODY_USES;
    }
    write_root_into_the_ring_slot(&mut devices, empty_ring_slot, &abandoned);
    // 指针槽号不在单元区里的那条被抛弃根算「账读不出」；它是暖机那一版（树表 0 条、映射根是空指针），从中央映射也现算不成，
    // 重读一次仍不成就在取号之前拒可写（C393（被抛弃根的账读不出时修复没有条款） 取 (b)-从映射现算那一条的最后一支，用户 2026-09-28 定）——不 panic。
    let refusal = mount_writable(&parameters(), &mut devices)
        .expect_err("被抛弃根的指针不在单元区里、映射也现算不成：拒可写，不 panic");
    let MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable) = &refusal else {
        panic!("拒因是「重读一次仍读不出」：{refusal:?}");
    };
    assert!(
        matches!(
            still_unreadable.as_ref(),
            StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot { abandoned_root }
                if abandoned_root.instance == first.root.instance
                    && abandoned_root.checkpoint_txg == CheckpointTxg(first.root.checkpoint_txg.0 + 1)
        ),
        "拒因点名那条被抛弃根：{still_unreadable:?}"
    );
}
```

### crates/singlefs-harness/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs:442-525（a_device_left_with_one_self_verified_system_configuration_slot_is_not_refused_by_any_entry）

```rust
/// Y2-c：盘 1 两个系统配置槽里较新（世代号大）那一槽写坏一个字节，只剩上一次轮换那一槽自证得过——它落后于现行那一版，而现行那一版的单元
/// 都在盘 1 上。四个入口都照常做成（不误拒），做完之后池级 checker 不红；入口写过系统配置轮换的，之后照常可写挂载，
/// 一次轮换都没写的，那一槽还坏着，之后的可写挂载按 C331（择根倒挂压过已确认的写） 取甲拒。
#[test]
fn a_device_left_with_one_self_verified_system_configuration_slot_is_not_refused_by_any_entry() {
    for entry in EVERY_ENTRY {
        let (mut pool, _, now) = pool_with_a_snapshot_after_the_first_file();
        let mut devices = devices_from([
            (DeviceIdentity(0), &now, DeviceIdentity(0)),
            (DeviceIdentity(1), &now, DeviceIdentity(1)),
        ]);
        let spacing = u64::from(pool.parameters.geometry.fixed_structure_slot_spacing);
        let newer_generation = verified_system_configuration_slots(
            &devices,
            DeviceIdentity(1),
            spacing,
            &pool.parameters.filesystem_identifier,
        )
        .into_iter()
        .map(|slot| slot.quantities.slot_generation)
        .max()
        .expect("盘 1 两槽自证得过");
        // 下一次写的槽 = 世代号 mod 2（D22（单元原子性怎么合成） 已定项 16）：世代号 g 那一份住槽 g mod 2。
        let newer_slot_offset = DeviceOffsetInBytes((newer_generation % 2) * spacing);
        let device_one = &mut devices[1].1;
        let mut newer_slot = vec![0u8; 512];
        device_one
            .read_at(newer_slot_offset, &mut newer_slot)
            .expect("读较新那一槽");
        newer_slot[100] ^= 0xff;
        device_one
            .write_at(newer_slot_offset, &newer_slot, WriteDurability::Plain)
            .expect("写坏较新那一槽");
        assert_eq!(
            verified_system_configuration_slots(
                &devices,
                DeviceIdentity(1),
                spacing,
                &pool.parameters.filesystem_identifier,
            )
            .len(),
            1,
            "{entry:?}：盘 1 只剩一槽自证"
        );
        let refusal = go_through(entry, &mut pool, &mut devices);
        assert!(
            refusal.is_none(),
            "{entry:?}：只剩一槽自证的盘不误拒：{}",
            describe(&refusal)
        );
        let after = MemoryPool {
            devices: devices
                .iter()
                .map(|(identity, device)| (*identity, device.image.clone()))
                .collect(),
            device_size_in_bytes: now.device_size_in_bytes,
        };
        assert_eq!(
            common_admission::checker_violations_on(&after),
            Vec::<String>::new(),
            "{entry:?}：做完之后池级 checker 不红"
        );
        // 入口的写里有一次系统配置轮换就把写坏的那一槽重写了，之后照常可写挂载；入口一次轮换都没写（抬 F 到准入上限而没有可抬的），
        // 那一槽还自证不过，之后的可写挂载按 C331（择根倒挂压过已确认的写） 取甲拒（见证读缺一槽判不出，用户 2026-09-28 定）。
        let slots_of_device_one_verified_afterwards = verified_system_configuration_slots(
            &devices,
            DeviceIdentity(1),
            spacing,
            &pool.parameters.filesystem_identifier,
        )
        .len();
        let outcome = writable_mount_outcome_on_a_copy_of(&pool.parameters, &devices);
        if slots_of_device_one_verified_afterwards == 2 {
            assert!(outcome.is_ok(), "{entry:?}：写坏的那一槽已被轮换重写，之后照常可写挂载：{outcome:?}");
        } else {
            assert!(
                outcome
                    .as_ref()
                    .is_err_and(|refusal| refusal.contains("SomeSystemConfigurationSlotUnreadOrUnverified")),
                "{entry:?}：写坏的那一槽还自证不过，之后的可写挂载按甲拒：{outcome:?}"
            );
        }
    }
}
```

### crates/singlefs-harness/tests/fsync_drop_and_devices_without_the_selected_version.rs:1569-1601（assert_refused_by_the_witness_of_a_newer_publish）

```rust
/// C554 乙先拒的那一格（`MountError::NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion)`）：
/// 读阶段交出的所选那一版是 `selected_version`、它那次发布末条记录读得出、计数器 `selected_version_last_record_counter`，另一块盘的系统配置
/// 见证过计数器 `witnessed_journal_counter`（更大）；重读一遍读数相同。乙在读阶段之后、取号之前逐盘核之前判，所以这几格走不到逐盘核
/// （调查 `research/prompts/m2-investigate-three-reds-report.md` 第二节）。这几格里有一块盘的系统配置槽读不出（每次读都报错、或空盘），
/// C331（择根倒挂压过已确认的写） 取甲之后判据先按「见证读缺一槽，判不出」为真（用户 2026-09-28 定），读数里的比较记成那一支；
/// 见证值照旧是另一块盘读得出的最大 tail，所选那一版末条的计数器 `selected_version_last_record_counter` 只进说明文字。
fn assert_refused_by_the_witness_of_a_newer_publish(
    error: &MountError,
    what: &str,
    selected_version: RollbackTarget,
    witnessed_journal_counter: u64,
    selected_version_last_record_counter: u64,
) {
    let reading = SelectedVersionAgainstTheWitness {
        selected_version,
        witness: NewerPublishWitness {
            witnessed_journal_counter,
            comparison: WitnessedCounterComparison::SomeSystemConfigurationSlotUnreadOrUnverified,
        },
    };
    assert!(
        matches!(
            error,
            MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable)
                if **still_unreadable
                    == StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
                        first_read: reading,
                        reread: reading,
                    }
        ),
        "{what}：该被 C554 乙拒（见证 {witnessed_journal_counter} > 所选那一版末条 {selected_version_last_record_counter}），实际 {error:?}"
    );
}
```

### crates/singlefs-harness/tests/fsync_drop_and_devices_without_the_selected_version.rs:1667-1726（blank_device_refuses_the_writable_mount_by_name_before_any_write）

```rust
/// 空盘在盘 1（盘 0 带着 txg 5 那一版）：见证读缺盘 1 两槽，C331（择根倒挂压过已确认的写） 取甲之后判据按真，在逐盘核之前拒，
/// 两块盘逐字节不变；逐盘核点名空盘「没有自证过的系统配置」那一支经可写挂载走不到了（甲 先拒）。
/// 空盘在盘 0（盘 1 带着的最新一版是 txg 4：txg 5 的根在盘 0 上，它那条记录的点名单元盘 0 那一份验不过、施加不了）：
/// 盘 1 的系统配置见证过 txg 5 那次发布（jsn 5 > 所选那一版末条 4），C554 乙在逐盘核之前拒，逐盘核这一格不可达（C554 乙之后）；
/// 同样拒在取号之前、两块盘逐字节不变。逐盘核点名落后盘那一判由
/// `stale_device_one_refuses_the_writable_mount_naming_every_unit_it_misses_before_any_write` 钉着。
#[test]
fn blank_device_refuses_the_writable_mount_by_name_before_any_write() {
    let pool = pool_at_txg_five_with_device_one_stale_at_txg_three();
    let [_, second, third] = &pool.versions;
    assert_eq!(
        (second.record.counter, third.record.counter),
        (4, 5),
        "txg 4、txg 5 各一条记录"
    );
    let mut devices_with_device_zero_blank = vec![
        (DEVICE_ZERO, blank_device()),
        (DEVICE_ONE, sparse_from(&pool.full, DEVICE_ONE)),
    ];
    let image_before_the_mount_with_device_zero_blank =
        memory_pool_of(&devices_with_device_zero_blank);
    let refusal_with_device_zero_blank =
        mount_writable(&pool.parameters, &mut devices_with_device_zero_blank)
            .expect_err("盘 0 是空盘，可写挂载要拒");
    assert_refused_by_the_witness_of_a_newer_publish(
        &refusal_with_device_zero_blank,
        "盘 0 是空盘",
        version_key(1, 4),
        5,
        4,
    );
    assert_eq!(
        memory_pool_of(&devices_with_device_zero_blank),
        image_before_the_mount_with_device_zero_blank,
        "盘 0 是空盘：拒在取号之前，两块盘逐字节不变"
    );
    // 盘 1 是空盘：见证读缺盘 1 两槽，C331（择根倒挂压过已确认的写） 取甲之后判不出、按真，重读仍缺，同样在逐盘核之前拒（用户 2026-09-28 定）；
    // 见证值取盘 0 读得出的最大 tail 5，所选那一版 (1, 5) 末条 5。
    let mut devices_with_device_one_blank = vec![
        (DEVICE_ZERO, sparse_from(&pool.full, DEVICE_ZERO)),
        (DEVICE_ONE, blank_device()),
    ];
    let image_before_the_mount_with_device_one_blank =
        memory_pool_of(&devices_with_device_one_blank);
    let refusal_with_device_one_blank =
        mount_writable(&pool.parameters, &mut devices_with_device_one_blank)
            .expect_err("盘 1 是空盘，可写挂载要拒");
    assert_refused_by_the_witness_of_a_newer_publish(
        &refusal_with_device_one_blank,
        "盘 1 是空盘",
        version_key(1, 5),
        5,
        5,
    );
    assert_eq!(
        memory_pool_of(&devices_with_device_one_blank),
        image_before_the_mount_with_device_one_blank,
        "盘 1 是空盘：拒在取号之前，两块盘逐字节不变"
    );
}
```

### crates/singlefs-harness/tests/fsync_drop_and_devices_without_the_selected_version.rs:2111-2152（mount_writable_while_every_read_of_device_one_fails_is_refused_by_the_witness_before_any_write）

```rust
/// 读路径那一格（不是 fsync 失败；调查文件那条 `mount_writable_while_every_read_of_device_one_fails_is_recorded` 只记不判）：
/// 第一个文件（txg 3）→ 覆盖写 txg 4 之后，盘 1 的每一次读都报错、写照常。交进来的是两块盘，w 的下限那一判不拦；
/// 所选那一版是 (1, 3)、不是 (1, 4)：txg 4 的根只落在区域 1（盘 1，读不出），txg 4 那条记录点名的单元在盘 1 上那一份验不了、
/// 那次发布不施加（同「盘 0 是空盘」那一格所选的是 (1, 4) 而不是 (1, 5) 的道理）。盘 0 的系统配置见证过 txg 4 那次发布
/// （jsn 4 > 所选那一版末条 3），可写挂载在取号之前被 C554 乙拒——逐盘核（点名盘 1「没有自证过的系统配置」）在乙之后不可达，
/// 那一判由 `blank_device_refuses_the_writable_mount_by_name_before_any_write` 的「盘 1 是空盘」那一格钉着。一个写、一道屏障都没落下。
#[test]
fn mount_writable_while_every_read_of_device_one_fails_is_refused_by_the_witness_before_any_write()
{
    let (mut rig, mut allocator, first, instance) = build_rig();
    let second =
        overwrite(&mut rig, &mut allocator, &first, &content_of(2), instance).expect("txg 4");
    assert_eq!(
        (instance, second.root.checkpoint_txg, second.record.counter),
        (InstanceGeneration(1), CheckpointTxg(4), 4),
        "实例 1 的 txg 4 那一版，末条记录计数器 4"
    );
    drop(allocator);
    rig.plan.arm(FaultSchedule {
        fault: InjectedFault::ReadFails,
        device: FaultDeviceSelector::OnlyDevice(DEVICE_ONE),
        placement: FaultPlacement::AnyOffset,
        counting: FaultCounting::PerDevice,
        occurrence: FaultOccurrence::EVERY_MATCHING_CALL,
    });
    let operations_before_the_mount = rig.stream.operation_count();
    let error = mount_writable(&rig.parameters, &mut rig.devices)
        .expect_err("盘 1 每次读都报错，可写挂载要拒");
    rig.plan.disarm();
    assert_refused_by_the_witness_of_a_newer_publish(
        &error,
        "盘 1 每次读都报错",
        version_key(1, 3),
        4,
        3,
    );
    assert_eq!(
        rig.stream.operation_count(),
        operations_before_the_mount,
        "拒在取号之前：录制流里一个写、一道屏障都没多"
    );
}
```

### crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs:219-334（formatted_pool_mount_starting_after_a_leftover_record_publishes_from_the_next_txg）

```rust
/// 环里留着一条孤记录时，只做过 mkfs 的池照常可写挂载（2026-09-23 用户定案收窄 R4：`NEW_POOL_FILE_CREATION_TXG` 只管 mkfs
/// 同一个进程里那条流，不管任何池的第一个文件版本，所以「新实例的第一次发布不是 txg 1、jsn 1」不再是拒绝的理由）：
/// mkfs 之后第一次可写挂载崩在取号两写与 txg 1 的记录两写都持久、txg 1 的根槽没持久；两块盘系统配置槽 0（取号写进号 1 的那一槽）
/// 退回 mkfs 写的那一份（造出来的盘面：取号写之后才有记录，真崩溃留不下这一形）——择系统配置只剩 mkfs 的那一份、根环只有第 0 代根，要取的号 1、要写的行为空；环里那条 txg 1 的记录让新实例从
/// txg 2、jsn 2 起 ⇒ 零单元发布 txg 2（落盘 0），暖机推到 txg 4（落盘 1）才覆盖两块盘，之后第一个文件版本接着写 txg 5。
#[test]
fn formatted_pool_mount_starting_after_a_leftover_record_publishes_from_the_next_txg() {
    let mut formatted = format_pool("step-three-formatted-leftover-record");
    let mut first_mount_devices = formatted.reopen_recorded();
    mount_writable(&parameters(), &mut first_mount_devices).expect("第一次可写挂载");
    formatted.devices = Some(first_mount_devices);
    let operations = formatted.retained_operations();
    let (writes, segments) =
        writes_and_segments(&operations[formatted.mkfs_operation_count..], &geometry());
    let mut persisted = vec![false; writes.len()];
    for write_index in segments[0].iter().chain(segments[1].iter()) {
        persisted[*write_index] = true;
    }
    let crash_stream = SharedStream::retaining_contents();
    let mut devices = crash_state_devices(
        &formatted.memory_pool_after_mkfs(),
        &writes,
        &persisted,
        &crash_stream,
    );
    // 系统配置槽 0 退回 mkfs 写的那一份（mkfs 两槽写同一份字节，拿槽 1 的原样写回），两槽都自证得过：取号写进号 1 的那一槽看不见了。
    // 不写坏一个字节：C331（择根倒挂压过已确认的写） 取甲之后见证读缺一槽就拒可写（用户 2026-09-28 定），那样造的是撕裂轮换，走不到孤记录这一支。
    let slot_spacing_in_bytes = u64::from(geometry().fixed_structure_slot_spacing);
    let system_configuration_slot_bytes =
        usize::try_from(singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    for (_, device) in &mut devices {
        let mut mkfs_slot = vec![0u8; system_configuration_slot_bytes];
        device
            .wrapped_device()
            .read_at(DeviceOffsetInBytes(slot_spacing_in_bytes), &mut mkfs_slot)
            .expect("读系统配置槽 1（mkfs 写的那一份）");
        device
            .wrapped_device_mut()
            .image
            .write(DeviceOffsetInBytes(0), &mkfs_slot);
    }
    let image_before = memory_pool_of_sparse_devices(&devices);
    let before = disk_snapshot(&image_before, &crash_stream);
    assert_eq!(
        before
            .readable_roots
            .iter()
            .map(|root| (root.instance, root.checkpoint_txg))
            .collect::<Vec<_>>(),
        vec![(InstanceGeneration(0), CheckpointTxg(0)); 3],
        "根环只有三份第 0 代根"
    );
    let mounted = mount_writable(&parameters(), &mut devices).expect("孤记录之后照常可写挂载");
    assert_eq!(mounted.output.instance, InstanceGeneration(1), "要取的号 1");
    assert!(
        mounted.output.rows_written.is_empty(),
        "上一个实例是 0：不写行"
    );
    assert_eq!(
        (
            mounted.output.row_publish.root().checkpoint_txg,
            mounted.output.row_publish.record().counter
        ),
        (CheckpointTxg(2), 2),
        "环里那条 txg 1 的记录让新实例从 txg 2、jsn 2 起"
    );
    assert_eq!(
        mounted
            .output
            .warm_up_publishes
            .iter()
            .map(|publish| publish.root().checkpoint_txg)
            .collect::<Vec<_>>(),
        vec![CheckpointTxg(3), CheckpointTxg(4)],
        "txg 2 与 3 都落盘 0，推到 txg 4（落盘 1）才覆盖两块盘"
    );
    assert_eq!(region_device(2), DeviceIdentity(0));
    assert_eq!(region_device(3), DeviceIdentity(0));
    assert_eq!(region_device(4), DeviceIdentity(1));
    // 第一个文件版本接着写 txg 5：`publish_first_file` 从现行那一版接着算，不写死 3。
    let mut allocator = mounted.allocator;
    let current = mounted.current;
    let content = file_content();
    let first = {
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            current.root(),
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            InstanceGeneration(1),
            current.record_bytes(),
        )
        .expect("第一个文件版本")
    };
    assert_eq!(
        (first.root.checkpoint_txg, first.record.counter),
        (CheckpointTxg(5), 5)
    );
    assert_eq!(
        recover(
            &memory_pool_of_sparse_devices(&devices),
            JournalPolicy::Consult
        )
        .outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(1), CheckpointTxg(5)),
            content
        },
        "冷启动读回那个文件"
    );
}
```
## 三、新测试文件全文（四份）

### crates/singlefs-harness/tests/a_writable_mount_refuses_when_a_system_configuration_slot_does_not_verify.rs:1-98（新文件全文，98 行）

```rust
//! C331（择根倒挂压过已确认的写） 取甲（用户 2026-09-28 定，被攻过两轮；`research/prompts/unreadable-at-mount-r2-main-verification.md` L2）：
//! 可写挂载判 N-配置 的见证读里，只要有一个系统配置槽读不出或自证不过，就判不出、按判据为真：重读一次，仍缺那一槽就在取号之前拒可写，
//! 盘上逐字节不变；只读挂载照常（`mounted_read::mount_read_only`）。改之前读不出的槽不进 max、全读不出取 0 判「没见证」，
//! 新实例会从较旧的根可写挂载，确认过的写之后被较旧实例的根压过（第一轮判决 V1、V2）。
//! 这一形的代价（撕裂的系统配置轮换之后只剩只读挂载）是定案的一部分，见同一判决 L1。
mod common;

use common::{build_pool, parameters, BuiltPool};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::mount::{
    mount_writable, MountError, StillUnreadableAfterOneReread, WitnessedCounterComparison,
};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_format::{SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE, SYSTEM_CONFIGURATION_SLOT_BYTES};

/// 新池新建文件之后每块盘两槽：世代 4、世代 5（新池新建文件那次轮换）。把 `damaged_device` 世代 5 那一槽清零（撕裂那一形），另一槽照样自证得过。
fn pool_with_one_system_configuration_slot_zeroed(
    tag: &str,
    damaged_device: DeviceIdentity,
) -> BuiltPool {
    let mut pool = build_pool(tag);
    let slot_spacing_in_bytes = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    let devices = pool.devices.as_mut().expect("新池新建文件写完，盘还开着");
    let (_, device) = devices
        .iter_mut()
        .find(|(identity, _)| *identity == damaged_device)
        .expect("池里有这块盘");
    device
        .write_at(
            DeviceOffsetInBytes((5 % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE) * slot_spacing_in_bytes),
            &vec![0u8; usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096")],
            WriteDurability::Plain,
        )
        .expect("世代 5 那一槽清零");
    pool
}

fn a_writable_mount_refuses_before_acquisition_when_one_system_configuration_slot_does_not_verify(
    tag: &str,
    damaged_device: DeviceIdentity,
) {
    let mut pool = pool_with_one_system_configuration_slot_zeroed(tag, damaged_device);
    let image_before = pool.memory_pool();
    let mut devices = pool.reopen_recorded();
    let refusal = mount_writable(&parameters(), &mut devices)
        .expect_err("有一槽系统配置自证不过：判不出，重读一次仍缺，拒可写");
    pool.devices = Some(devices);
    let MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable) = &refusal else {
        panic!("拒因是「重读一次仍读不出更新的状态」：{refusal:?}");
    };
    let StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
        first_read,
        reread,
    } = still_unreadable.as_ref()
    else {
        panic!("拒在判据 N-配置 那一格：{still_unreadable:?}");
    };
    for reading in [first_read, reread] {
        assert_eq!(
            reading.witness.comparison,
            WitnessedCounterComparison::SomeSystemConfigurationSlotUnreadOrUnverified,
            "两遍读都判「有一槽系统配置读不出或自证不过」：{reading:?}"
        );
    }
    assert!(
        pool.memory_pool() == image_before,
        "拒可写在取号之前：盘上逐字节不变"
    );
    mount_read_only(&pool.memory_pool()).expect("只读挂载照常");
}

#[test]
fn a_writable_mount_refuses_before_acquisition_when_the_first_devices_newest_system_configuration_slot_does_not_verify(
) {
    a_writable_mount_refuses_before_acquisition_when_one_system_configuration_slot_does_not_verify(
        "c331-jia-mount-device-zero",
        DeviceIdentity(0),
    );
}

#[test]
fn a_writable_mount_refuses_before_acquisition_when_the_second_devices_newest_system_configuration_slot_does_not_verify(
) {
    a_writable_mount_refuses_before_acquisition_when_one_system_configuration_slot_does_not_verify(
        "c331-jia-mount-device-one",
        DeviceIdentity(1),
    );
}

/// 对照：两槽都自证得过的同一个池照常可写挂载（判据 N-配置 为假，不重读）。
#[test]
fn a_writable_mount_with_every_system_configuration_slot_verified_goes_ahead() {
    let mut pool = build_pool("c331-jia-mount-control");
    let mut devices = pool.reopen_recorded();
    mount_writable(&parameters(), &mut devices).expect("两槽都自证得过：照常可写挂载");
    pool.devices = Some(devices);
}
```

### crates/singlefs-harness/tests/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs:1-465（新文件全文，465 行）

```rust
//! C393（被抛弃根的账读不出时修复没有条款） 取 (b)-从映射现算（用户 2026-09-28 定，被攻过零轮；
//! `research/prompts/unreadable-at-mount-r2-main-verification.md` L3）：崩溃恢复抛弃根 C (2, 8) 之后再重开可写挂载（实例 4），
//! C 的分配记录树根节点或树表两份都读不出时，影子账不再跳过 C，从中央映射现算 C 引用的落点照样隔离（与读得出那一臂同是每盘 14 槽），
//! 计进 `abandoned_roots_unreadable` 往上报；之后覆盖写到 txg 31（txg 32 的根盖掉 C 的根槽）C 的单元一个都不复用、I-7.4（近 K 代块未被复用） 成立。
//! 改之前读不出只计数、跳过，隔离 0，重开之后第一次覆盖写（txg 14）就把 C 的数据单元（盘 0 槽 50184）改写、池级 checker 红 I-7.4
//! （调查员报告 `research/prompts/closeout-recheck-2026-09-28/c393-investigator-report.md`；这份用例从那份调查的副本用例改来）。
mod common;

use common::{
    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before, build_pool, parameters,
    publish_overwrite_in_process, with_unreadable_ranges, without_unreadable_ranges, BuiltPool,
    FailingReadsOfARange, SharedUnreadableRanges, UnreadableRange, UnreadableRangeReadBack,
    FIXED_WRITE_TIME_SECONDS,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread, BeforeTheOneReread,
    ShadowLedger,
};
use singlefs_core::transaction::{TransactionOutput, TransactionUnit};
use singlefs_format::SLOT_BYTES;
use singlefs_harness::memory_pool::MemoryPool;
use std::cell::Cell;

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Account {
    AllocationRecordTreeRoot,
    TreeTable,
}

impl Account {
    fn unit(self) -> TransactionUnit {
        match self {
            Account::AllocationRecordTreeRoot => TransactionUnit::AllocationTree,
            Account::TreeTable => TransactionUnit::TreeTable,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Unreadability {
    /// 阳性对照：包装在，一次都不坏。
    Readable,
    /// 两份每次读都坏，钩子不撤（产品路径 RereadImmediately）。
    EveryRead,
    /// 两份每次读都坏，直到产品的「重读一次」钩子被调用时撤掉（若产品重读，重读读得出）。
    EveryReadUntilTheRereadHook,
    /// 两份各只坏第一次读，之后照读（设备一层的瞬时错）。
    OnlyTheFirstReadOfEachCopy,
}

/// A、B（实例 1）、重开取号 2、C (2, 8)；崩溃恢复抛弃 C（落到 (2, 7)，实例 3 写行 9、暖机 10）、C 写回。
fn pool_after_a_recovery_abandoned_c(tag: &str) -> (BuiltPool, TransactionOutput) {
    let mut pool = build_pool(tag);
    let first = pool.output.clone();
    pool.output = publish_overwrite_in_process(
        &mut pool,
        &first,
        &content_of(4100, 3),
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(1),
    )
    .expect("B");
    let mut devices = pool.reopen_recorded();
    let mounted = singlefs_core::mount::mount_writable(&parameters(), &mut devices).expect("取号 2");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted.current.into_file_version().expect("带文件");
    let before = pool.output.clone();
    let abandoned = publish_overwrite_in_process(
        &mut pool,
        &before,
        &content_of(2500, 11),
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(2),
    )
    .expect("C");
    let abandoning =
        abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before(&mut pool, &abandoned);
    let newest = *abandoning.current.root();
    assert_eq!(
        (newest.instance, newest.checkpoint_txg),
        (InstanceGeneration(3), CheckpointTxg(10))
    );
    (pool, abandoned)
}

struct Remount {
    pool: BuiltPool,
    abandoned: TransactionOutput,
    account_slot: u64,
    result: Result<(), String>,
    isolated: Vec<(DeviceIdentity, u64)>,
    abandoned_roots_unreadable: u64,
    rereads: String,
    space_admission: String,
    reads_of_each_copy: (u64, u64),
    failed_reads: u64,
    hook_calls: u32,
    current_txg: Option<CheckpointTxg>,
    /// C 那次发布写出的单元里，重开之后在盘 0 的分配器里是空闲的那几个（起点槽）。
    free_units_of_c_after_remount: Vec<u64>,
}

fn remount(tag: &str, account: Account, unreadability: Unreadability) -> Remount {
    let (mut pool, abandoned) = pool_after_a_recovery_abandoned_c(tag);
    let unit = abandoned.unit(account.unit());
    let failing = match unreadability {
        Unreadability::Readable => FailingReadsOfARange::Never,
        Unreadability::EveryRead | Unreadability::EveryReadUntilTheRereadHook => {
            FailingReadsOfARange::Every
        }
        Unreadability::OnlyTheFirstReadOfEachCopy => FailingReadsOfARange::OnlyTheFirst,
    };
    let ranges: Vec<UnreadableRange> = [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|device| UnreadableRange {
            device,
            offset_in_bytes: unit.slot.0 * SLOT_BYTES,
            length_in_bytes: u64::try_from(unit.bytes.len()).expect("长度"),
            failing_reads: failing,
        })
        .collect();
    let unreadable = SharedUnreadableRanges::new(ranges, UnreadableRangeReadBack::DeviceError);
    let mut devices = with_unreadable_ranges(pool.reopen_recorded(), &unreadable);
    let hook_calls = Cell::new(0u32);
    let lifting = unreadable.clone();
    let mut hook = || {
        hook_calls.set(hook_calls.get() + 1);
        if unreadability == Unreadability::EveryReadUntilTheRereadHook {
            lifting.lift();
        }
    };
    let mounted = mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread(
        &parameters(),
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        BeforeTheOneReread::CallTheTestOnlyHookFirst(&mut hook),
    );
    let reads_of_each_copy = (unreadable.reads_of_range(0), unreadable.reads_of_range(1));
    let failed_reads = unreadable.failed_reads();
    pool.devices = Some(without_unreadable_ranges(devices));
    let mut remount = Remount {
        pool,
        abandoned: abandoned.clone(),
        account_slot: unit.slot.0,
        result: Ok(()),
        isolated: Vec::new(),
        abandoned_roots_unreadable: 0,
        rereads: String::new(),
        space_admission: String::new(),
        reads_of_each_copy,
        failed_reads,
        hook_calls: hook_calls.get(),
        current_txg: None,
        free_units_of_c_after_remount: Vec::new(),
    };
    match mounted {
        Ok(mounted) => {
            assert_eq!(mounted.output.instance, InstanceGeneration(4));
            remount.isolated = mounted.output.isolated_slots_per_device.clone();
            remount.abandoned_roots_unreadable = mounted.output.abandoned_roots_unreadable;
            remount.rereads = format!("{:?}", mounted.output.rereads);
            remount.space_admission = format!("{:?}", mounted.output.space_admission);
            remount.free_units_of_c_after_remount = abandoned
                .units
                .iter()
                .filter(|unit| mounted.allocator.devices[0].is_free(unit.slot))
                .map(|unit| unit.slot.0)
                .collect();
            remount.pool.allocator = mounted.allocator;
            remount.pool.output = mounted.current.into_file_version().expect("带文件");
            remount.current_txg = Some(remount.pool.output.root.checkpoint_txg);
        }
        Err(error) => remount.result = Err(format!("{error:?}")),
    }
    remount
}

/// C 那次发布写出的单元里，此刻盘 0 上逐字节已不是 C 写的那几个。
fn overwritten_units_of(image: &MemoryPool, abandoned: &TransactionOutput) -> Vec<(u64, String)> {
    let device = image.devices.get(&DeviceIdentity(0)).expect("盘 0");
    abandoned
        .units
        .iter()
        .filter(|unit| {
            device.read(DeviceOffsetInBytes(unit.slot.0 * SLOT_BYTES), unit.bytes.len()) != unit.bytes
        })
        .map(|unit| (unit.slot.0, unit.identity.tag()))
        .collect()
}

fn verdict_of(image: &MemoryPool, invariant: &str) -> InvariantVerdict {
    check_pool_image(image)
        .into_iter()
        .find(|(name, _)| *name == invariant)
        .expect("清单里有")
        .1
}

fn violated(image: &MemoryPool) -> Vec<&'static str> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(name, verdict)| matches!(verdict, InvariantVerdict::Violated(_)).then_some(name))
        .collect()
}

/// 实例 4 里接着覆盖写，直到 C 的某个单元在盘上被改写或 txg 到 31（txg 32 的根盖掉 C 的根槽）。
/// 交回 (第一次改写时的 txg, 被改写的单元, 那一刻 I-7.4 的判定, 那一刻红的不变量, C 的账两份抹零之后 I-7.4 的判定)。
fn continue_until_a_unit_of_c_is_reused(remount: &mut Remount) -> String {
    let mut report = String::new();
    while remount.pool.output.root.checkpoint_txg < CheckpointTxg(31) {
        let steps = usize::try_from(remount.pool.output.root.checkpoint_txg.0).expect("txg");
        let previous = remount.pool.output.clone();
        remount.pool.output = publish_overwrite_in_process(
            &mut remount.pool,
            &previous,
            &content_of(2000 + steps, steps + 41),
            FIXED_WRITE_TIME_SECONDS + 600,
            InstanceGeneration(4),
        )
        .expect("实例 4 覆盖写");
        let image = remount.pool.memory_pool();
        let overwritten = overwritten_units_of(&image, &remount.abandoned);
        if !overwritten.is_empty() {
            let txg = remount.pool.output.root.checkpoint_txg;
            let verdict = verdict_of(&image, "I-7.4");
            let red = violated(&image);
            let mut damaged = image.clone();
            for device in [0u32, 1] {
                damaged
                    .devices
                    .get_mut(&DeviceIdentity(device))
                    .expect("盘")
                    .write(
                        DeviceOffsetInBytes(remount.account_slot * SLOT_BYTES),
                        &vec![0u8; usize::try_from(SLOT_BYTES).expect("槽宽")],
                    );
            }
            let damaged_verdict = verdict_of(&damaged, "I-7.4");
            report.push_str(&format!(
                "reuse_txg={} overwritten={overwritten:?} I-7.4={verdict:?} red={red:?} I-7.4_after_zeroing_the_account={damaged_verdict:?}",
                txg.0
            ));
            return report;
        }
    }
    let image = remount.pool.memory_pool();
    report.push_str(&format!(
        "no_reuse_through_txg={} I-7.4={:?} red={:?}",
        remount.pool.output.root.checkpoint_txg.0,
        verdict_of(&image, "I-7.4"),
        violated(&image)
    ));
    report
}

fn run_arm(account: Account, unreadability: Unreadability) -> (Remount, String) {
    let tag = format!("c393-{account:?}-{unreadability:?}");
    let mut remount = remount(&tag, account, unreadability);
    let line = format!(
        "name=c393 account={account:?} unreadability={unreadability:?} account_slot={} result={:?} current_txg={:?} free_units_of_c_after_remount={:?} isolated={:?} abandoned_roots_unreadable={} reads_of_each_copy={:?} failed_reads={} hook_calls={} rereads={} space_admission={}",
        remount.account_slot,
        remount.result,
        remount.current_txg.map(|txg| txg.0),
        remount.free_units_of_c_after_remount,
        remount.isolated,
        remount.abandoned_roots_unreadable,
        remount.reads_of_each_copy,
        remount.failed_reads,
        remount.hook_calls,
        remount.rereads,
        remount.space_admission,
    );
    let reuse = if remount.result.is_ok() {
        continue_until_a_unit_of_c_is_reused(&mut remount)
    } else {
        "mount_refused".to_string()
    };
    (remount, format!("{line} {reuse}"))
}

const ISOLATED_14: [(DeviceIdentity, u64); 2] = [(DeviceIdentity(0), 14), (DeviceIdentity(1), 14)];

/// 读不出的那几臂共同的结局：照常可写挂载（实例 4，txg 到 13），不重读（现算就成了，钩子 0 次），计数 1（显式报告），
/// 从映射现算照样隔离每盘 14 槽，之后覆盖写到 txg 31 C 的单元一个都不复用、I-7.4 成立。
fn assert_recomputed_from_the_mapping_isolated_and_not_reused(account: Account, unreadability: Unreadability) {
    let (remount, line) = run_arm(account, unreadability);
    eprintln!("{line}");
    assert_eq!(remount.result, Ok(()), "照常可写挂载");
    assert_eq!(remount.current_txg, Some(CheckpointTxg(13)));
    assert_eq!(remount.hook_calls, 0, "从映射现算就成了，不走「重读一次」");
    assert_eq!(remount.abandoned_roots_unreadable, 1, "读不出的账照样计数往上报");
    assert_eq!(remount.isolated, ISOLATED_14.to_vec(), "现算出的落点与读得出那一臂隔离的一样多");
    assert_eq!(remount.free_units_of_c_after_remount, Vec::<u64>::new(), "C 的单元一个都没回到空闲");
    assert!(line.contains("no_reuse_through_txg=31 I-7.4=Holds red=[]"), "{line}");
}

fn readable_control_isolates_14_and_nothing_of_c_is_reused_through_txg_31(account: Account) {
    let (remount, line) = run_arm(account, Unreadability::Readable);
    eprintln!("{line}");
    assert_eq!(remount.result, Ok(()));
    assert_eq!(remount.hook_calls, 0);
    assert_eq!(remount.abandoned_roots_unreadable, 0);
    assert_eq!(remount.isolated, ISOLATED_14.to_vec());
    assert!(remount.reads_of_each_copy.0 >= 1, "影子账那一读真的读到了这一段");
    assert_eq!(remount.free_units_of_c_after_remount, Vec::<u64>::new());
    assert!(line.contains("no_reuse_through_txg=31 I-7.4=Holds red=[]"), "{line}");
}

#[test]
fn c393_readable_allocation_record_tree_control_isolates_14_and_nothing_of_c_is_reused_through_txg_31() {
    readable_control_isolates_14_and_nothing_of_c_is_reused_through_txg_31(Account::AllocationRecordTreeRoot);
}

#[test]
fn c393_readable_tree_table_control_isolates_14_and_nothing_of_c_is_reused_through_txg_31() {
    readable_control_isolates_14_and_nothing_of_c_is_reused_through_txg_31(Account::TreeTable);
}

#[test]
fn c393_allocation_record_tree_unreadable_on_every_read() {
    assert_recomputed_from_the_mapping_isolated_and_not_reused(Account::AllocationRecordTreeRoot, Unreadability::EveryRead);
}

#[test]
fn c393_allocation_record_tree_unreadable_until_the_reread_hook() {
    assert_recomputed_from_the_mapping_isolated_and_not_reused(
        Account::AllocationRecordTreeRoot,
        Unreadability::EveryReadUntilTheRereadHook,
    );
}

/// 分配记录树根节点是进映射的节点：提示两份读不出时经中央映射回退再读一次（`recovery::read_mapped_tree_node_via_hint_then_central_mapping`），
/// 映射落点与提示同槽，第二次读读得出——设备一层只坏第一次的瞬时错在这里被吸收。
#[test]
fn c393_allocation_record_tree_unreadable_only_on_the_first_read_of_each_copy() {
    let (remount, line) = run_arm(
        Account::AllocationRecordTreeRoot,
        Unreadability::OnlyTheFirstReadOfEachCopy,
    );
    eprintln!("{line}");
    assert_eq!(remount.result, Ok(()));
    assert_eq!(remount.hook_calls, 0);
    assert_eq!(remount.reads_of_each_copy, (2, 1), "盘 0 提示一读、盘 1 提示一读、映射回退盘 0 再一读");
    assert_eq!(remount.failed_reads, 2);
    assert_eq!(remount.abandoned_roots_unreadable, 0);
    assert_eq!(remount.isolated, ISOLATED_14.to_vec());
    assert!(line.contains("no_reuse_through_txg=31 I-7.4=Holds red=[]"), "{line}");
}

#[test]
fn c393_tree_table_unreadable_on_every_read() {
    assert_recomputed_from_the_mapping_isolated_and_not_reused(Account::TreeTable, Unreadability::EveryRead);
}

#[test]
fn c393_tree_table_unreadable_until_the_reread_hook() {
    assert_recomputed_from_the_mapping_isolated_and_not_reused(Account::TreeTable, Unreadability::EveryReadUntilTheRereadHook);
}

/// 树表豁免映射，只按位置条目读一遍（两份各一次），没有第二次读：只坏第一次与一直坏结局相同。
#[test]
fn c393_tree_table_unreadable_only_on_the_first_read_of_each_copy() {
    assert_recomputed_from_the_mapping_isolated_and_not_reused(Account::TreeTable, Unreadability::OnlyTheFirstReadOfEachCopy);
}

/// 钩子的阳性对照：同一段历史，最新那条根 (3, 10) 的实例表那一片在影子账那一读起坏、钩子里撤——产品的重读走到钩子（1 次），
/// 重读读得出，照隔离 14。证明上面各臂的「钩子 0 次」不是钩子接错了。
#[test]
fn c393_positive_control_the_hook_is_called_when_the_newest_instance_table_is_unreadable() {
    let (mut pool, _) = pool_after_a_recovery_abandoned_c("c393-hook-positive-control");
    let mut devices_for_the_page = pool.reopen_recorded();
    let newest = singlefs_core::recovery::choose_root(
        &devices_for_the_page,
        &singlefs_core::recovery::choose_system_configuration(&devices_for_the_page).expect("择"),
    )
    .expect("最新根");
    let _ = &mut devices_for_the_page;
    pool.devices = Some(devices_for_the_page);
    let page = newest.instance_table;
    let ranges: Vec<UnreadableRange> = page
        .locations
        .iter()
        .map(|location| UnreadableRange {
            device: location.device,
            offset_in_bytes: location.slot.to_device_offset().0,
            length_in_bytes: singlefs_format::DATA_UNIT_BYTES,
            failing_reads: if location.device == DeviceIdentity(0) {
                FailingReadsOfARange::FromTheNthOnward(3)
            } else {
                FailingReadsOfARange::Every
            },
        })
        .collect();
    let unreadable = SharedUnreadableRanges::new(ranges, UnreadableRangeReadBack::DeviceError);
    let mut devices = with_unreadable_ranges(pool.reopen_recorded(), &unreadable);
    let hook_calls = Cell::new(0u32);
    let lifting = unreadable.clone();
    let mut hook = || {
        hook_calls.set(hook_calls.get() + 1);
        lifting.lift();
    };
    let mounted = mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread(
        &parameters(),
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        BeforeTheOneReread::CallTheTestOnlyHookFirst(&mut hook),
    )
    .expect("重读读得出");
    pool.devices = Some(without_unreadable_ranges(devices));
    eprintln!(
        "name=c393-hook-positive-control hook_calls={} isolated={:?} abandoned_roots_unreadable={} rereads={:?}",
        hook_calls.get(),
        mounted.output.isolated_slots_per_device,
        mounted.output.abandoned_roots_unreadable,
        mounted.output.rereads
    );
    assert_eq!(hook_calls.get(), 1);
    assert_eq!(mounted.output.isolated_slots_per_device, ISOLATED_14.to_vec());
}

/// 账在盘上真坏（两份抹零）而没有复用时，池级 checker 对 I-7.4 判什么：拿读得出那一臂重开之后、还没接着写的镜像抹 C 的账。
fn checker_on_a_persistently_damaged_account_without_reuse(account: Account) {
    let tag = format!("c393-damaged-{account:?}");
    let remount = remount(&tag, account, Unreadability::Readable);
    let image = remount.pool.memory_pool();
    let before = verdict_of(&image, "I-7.4");
    let mut damaged = image.clone();
    for device in [0u32, 1] {
        damaged
            .devices
            .get_mut(&DeviceIdentity(device))
            .expect("盘")
            .write(
                DeviceOffsetInBytes(remount.account_slot * SLOT_BYTES),
                &vec![0u8; usize::try_from(SLOT_BYTES).expect("槽宽")],
            );
    }
    let after = verdict_of(&damaged, "I-7.4");
    let red = violated(&damaged);
    eprintln!("name=c393-damaged account={account:?} I-7.4_before={before:?} I-7.4_after={after:?} red_after={red:?}");
    assert_eq!(before, InvariantVerdict::Holds);
}

#[test]
fn c393_checker_on_a_persistently_damaged_allocation_record_tree_without_reuse() {
    checker_on_a_persistently_damaged_account_without_reuse(Account::AllocationRecordTreeRoot);
}

#[test]
fn c393_checker_on_a_persistently_damaged_tree_table_without_reuse() {
    checker_on_a_persistently_damaged_account_without_reuse(Account::TreeTable);
}
```

### crates/singlefs-harness/tests/raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic.rs:1-114（新文件全文，114 行）

```rust
//! 里程碑「覆盖写、释放、回退与复用」增补 2 收口表第 43 行取丁-defer（用户 2026-09-28 定，被攻过一轮；
//! `research/prompts/abandoned-floor-r1-main-verification.md` M1）：I-3.1（已分配统计对得上） 的「实际遍历」并上 F_生效 之下、没被抛弃的根
//! 引用着、候选集那一遍没走到、最新根的账里是已释放且释放代高于 F_生效 的槽（还在 defer 里）。
//!
//! 历史只靠崩溃恢复造被抛弃实例（调查员报告 `research/prompts/closeout-recheck-2026-09-28/row43-investigator-report.md` 的最小复现）：
//! A（txg 3）→ 覆盖写 B（4）、C（5）→ C 的系统配置轮换没落盘就崩（轮换写全部退回崩溃前的字节，两槽照样自证得过）→ 可写挂载时 C 的根槽与
//! 数据单元一时读不出，落到 B、实例 2 → 再可写挂载一次（影子账看见 C）→ 覆盖写四次 → 抬 F 到 5。每块盘差的 10 槽释放代 6、
//! 只被 B（txg 4，F 之下）与被抛弃的 C 引用、还在 defer 里：改之前只有 F = 5 那一格 I-3.1 红（记账多于遍历），并上之后全绿。
mod common;

use common::{
    abandon_the_third_version_by_a_crash_before_its_rotation, build_pool, parameters,
    publish_overwrite_in_process, system_configuration_slots_of_every_device, BuiltPool,
    FIXED_WRITE_TIME_SECONDS,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::mount::{mount_writable, raise_rollback_floor, ShadowLedger};
use singlefs_core::transaction::TransactionOutput;
use singlefs_harness::memory_pool::MemoryPool;

fn content_seeded_by(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

fn overwrite(pool: &mut BuiltPool, content: &[u8], instance: InstanceGeneration) -> TransactionOutput {
    let previous = pool.output.clone();
    let output = publish_overwrite_in_process(
        pool,
        &previous,
        content,
        FIXED_WRITE_TIME_SECONDS + 60,
        instance,
    )
    .expect("覆盖写");
    pool.output = output.clone();
    output
}

fn violations_on(image: &MemoryPool) -> Vec<(&'static str, String)> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some((invariant, detail)),
            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
        })
        .collect()
}

/// A、B、C（实例 1）→ 崩溃恢复抛弃 C → 再可写挂载一次 → 覆盖写四次：抬 F 之前全绿的池，与再挂载那一次的实例代号。
fn pool_before_the_raise(tag: &str) -> BuiltPool {
    let mut pool = build_pool(tag);
    overwrite(&mut pool, &content_seeded_by(4100, 3), InstanceGeneration(1));
    let system_configuration_before_the_third = system_configuration_slots_of_every_device(&pool);
    let third = overwrite(&mut pool, &content_seeded_by(2500, 11), InstanceGeneration(1));
    assert_eq!(third.root.checkpoint_txg, CheckpointTxg(5), "C 是 txg 5");
    abandon_the_third_version_by_a_crash_before_its_rotation(
        &mut pool,
        &third,
        &system_configuration_before_the_third,
    );
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("再可写挂载：影子账看见 C");
    let instance = mounted.output.instance;
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator.clone();
    pool.output = mounted.current.file_version().expect("带文件").clone();
    for seed in [17usize, 19, 23, 29] {
        overwrite(&mut pool, &content_seeded_by(3000 + seed, seed), instance);
    }
    assert_eq!(
        violations_on(&pool.memory_pool()),
        Vec::<(&'static str, String)>::new(),
        "抬 F 之前池级 checker 全绿"
    );
    pool
}

fn raise_the_floor_and_judge(tag: &str, floor: u64) -> Vec<(&'static str, String)> {
    let mut pool = pool_before_the_raise(tag);
    raise_rollback_floor(
        &parameters(),
        pool.devices.as_mut().expect("镜像开着"),
        &mut pool.allocator,
        &mut pool.output,
        CheckpointTxg(floor),
        ShadowLedger::On,
    )
    .expect("抬 F");
    violations_on(&pool.memory_pool())
}

/// F 抬到 C 那个 txg（5）：F 落进被抛弃实例留下的空档，B（txg 4）掉到 F 之下、C 被抛弃，只被它俩引用、释放代 6 的 10 槽还在 defer 里。
#[test]
fn raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_every_invariant_green() {
    assert_eq!(
        raise_the_floor_and_judge("row43-gap-floor-5", 5),
        Vec::<(&'static str, String)>::new(),
        "F 之下仍在 defer 的槽并进遍历：I-3.1 不红"
    );
}

/// 对照：同一段历史只换 F（6：释放代 6 的那 10 槽已可回收），改之前改之后都全绿。
#[test]
fn raising_the_floor_past_the_gap_keeps_every_invariant_green() {
    assert_eq!(
        raise_the_floor_and_judge("row43-gap-floor-6", 6),
        Vec::<(&'static str, String)>::new(),
        "F 抬过空档：全绿"
    );
}
```

### crates/singlefs-harness/tests/an_abandoned_roots_rollback_floor_counts_toward_the_effective_floor.rs:1-123（新文件全文，123 行）

```rust
//! 里程碑「覆盖写、释放、回退与复用」增补 2 收口表第 ② 行：被抛弃根带的 F 算进 F_生效（D16（发布语义） 已定项 1，用户 2026-09-28 定「算进」，
//! 被攻过一轮；`research/prompts/abandoned-floor-r1-main-verification.md` M2）。实现一侧 `recovery::effective_rollback_floor` 与池级 checker
//! 各算一份（两份不共用代码），两份都要把被抛弃根带的 F 算进去。
//!
//! 被抛弃根带着比各盘有效根与系统配置都高的 F，只在「系统配置里那一槽一时读不出」时的可写挂载上造得出（攻方腿 E4）；C331 取甲之后那样的
//! 可写挂载被拒，所以这里按坏镜像的造法直接造盘面：崩溃恢复抛弃 C（收口表第 43 行调查员那一形，
//! `common::abandon_the_third_version_by_a_crash_before_its_rotation`）之后，把 C 的根记录改成带 F = 4 重新编码写回——各盘有效根与系统配置里的 F 都是 0。
mod common;

use common::{
    abandon_the_third_version_by_a_crash_before_its_rotation, build_pool, parameters,
    publish_overwrite_in_process, system_configuration_slots_of_every_device, BuiltPool,
    FIXED_WRITE_TIME_SECONDS,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::recovery::{choose_system_configuration, effective_rollback_floor};
use singlefs_core::root_record::RootRecord;
use singlefs_core::transaction::TransactionOutput;

/// 改写之后 C 带的 F：高于 0（各盘有效根与系统配置里的 F），不高于 C 自己的 txg 5。
const FLOOR_CARRIED_BY_THE_ABANDONED_ROOT: CheckpointTxg = CheckpointTxg(4);

fn content_seeded_by(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

fn overwrite(pool: &mut BuiltPool, content: &[u8]) -> TransactionOutput {
    let previous = pool.output.clone();
    let output = publish_overwrite_in_process(
        pool,
        &previous,
        content,
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(1),
    )
    .expect("覆盖写");
    pool.output = output.clone();
    output
}

/// A、B、C（实例 1）→ 崩溃恢复抛弃 C（落到 B、实例 2 写行与暖机）→ C 的根记录改带 F = 4。
fn pool_whose_abandoned_root_carries_a_higher_floor(tag: &str) -> BuiltPool {
    let mut pool = build_pool(tag);
    overwrite(&mut pool, &content_seeded_by(4100, 3));
    let system_configuration_before_the_third = system_configuration_slots_of_every_device(&pool);
    let third = overwrite(&mut pool, &content_seeded_by(2500, 11));
    abandon_the_third_version_by_a_crash_before_its_rotation(
        &mut pool,
        &third,
        &system_configuration_before_the_third,
    );
    let publish_parameters = parameters();
    let target = singlefs_core::root_ring::target_for_publish(
        third.root.checkpoint_txg,
        publish_parameters.geometry.root_ring_slots_per_region,
    );
    let root_slot_device =
        publish_parameters.region_devices[usize::try_from(target.region).expect("区域号")];
    let root_slot_offset = singlefs_core::root_ring::slot_offset(
        target,
        publish_parameters.geometry.fixed_structure_slot_spacing,
    );
    let root_slot_bytes =
        usize::try_from(publish_parameters.geometry.physical_block_size).expect("根槽宽");
    let devices = pool.devices.as_mut().expect("镜像开着");
    let (_, device) = devices
        .iter_mut()
        .find(|(identity, _)| *identity == root_slot_device)
        .expect("池里有这块盘");
    let mut slot = vec![0u8; root_slot_bytes];
    device.read_at(root_slot_offset, &mut slot).expect("读 C 的根槽");
    let mut abandoned_root = RootRecord::parse_slot(&slot, &publish_parameters.filesystem_identifier)
        .expect("C 的根自证得过");
    assert_eq!(abandoned_root.checkpoint_txg, CheckpointTxg(5), "根槽里住的是 C");
    assert_eq!(abandoned_root.rollback_floor, CheckpointTxg(0));
    abandoned_root.rollback_floor = FLOOR_CARRIED_BY_THE_ABANDONED_ROOT;
    device
        .write_at(
            root_slot_offset,
            &abandoned_root.to_slot(root_slot_bytes),
            WriteDurability::Plain,
        )
        .expect("C 的根改带 F = 4 写回");
    pool
}

/// 实现一侧：F_生效 取到被抛弃的 C 带的 F。改之前只看各盘最新的有效根（带 0）与系统配置（0），F_生效 = 0。
#[test]
fn the_implementations_effective_floor_counts_the_floor_an_abandoned_root_carries() {
    let pool = pool_whose_abandoned_root_carries_a_higher_floor("abandoned-floor-counts-core");
    let image = pool.memory_pool();
    let system_configuration = choose_system_configuration(&image).expect("择系统配置");
    assert_eq!(
        effective_rollback_floor(
            &image,
            &parameters().region_devices,
            &system_configuration.immutable.sizes,
            &parameters().filesystem_identifier,
        ),
        FLOOR_CARRIED_BY_THE_ABANDONED_ROOT,
        "被抛弃的 C 带的 F 算进 F_生效"
    );
}

/// 池级 checker 一侧：F_生效 取到 4，高于最新根带的 F（0）——I-3.1（已分配统计对得上） 那一格按「最新根带的 F 低于 F 生效值」报不适用。
/// 改之前 checker 的 F_生效 = 0 = 最新根带的 F，I-3.1 照常判。
#[test]
fn the_checkers_effective_floor_counts_the_floor_an_abandoned_root_carries() {
    let pool = pool_whose_abandoned_root_carries_a_higher_floor("abandoned-floor-counts-checker");
    let verdict = check_pool_image(&pool.memory_pool())
        .into_iter()
        .find_map(|(invariant, verdict)| (invariant == "I-3.1").then_some(verdict))
        .expect("checker 报 I-3.1");
    assert!(
        matches!(&verdict, InvariantVerdict::NotApplicable(reason) if reason.contains("最新根带的 F 低于 F 生效值")),
        "checker 的 F_生效 取到被抛弃根带的 4、高于最新根带的 0：{verdict:?}"
    );
}
```

## 四、`crates/mutations.tsv` 新加的 6 行（前面带文件头的列说明）

### crates/mutations.tsv:1-5（文件头：列的说明）

```text
# crates 的变异表：每行六段，制表符分隔——变异名 <TAB> 文件 <TAB> 原文 <TAB> 替换文 <TAB> cargo test 的参数 <TAB> 必须红的测试名。
# 原文 / 替换文里的 \n 表示换行；原文在文件里必须恰好命中一次（锚点腐化就红，门禁 59 号）。
# 门禁 59 号把仓拷到临时目录、逐条改坏、跑点名的测试、要求那条测试判红、还原；一条没红就整道红。
# 这是 show-me-test.md「存进仓的变异清单，交给门禁反复复跑」那一条在 crates 上的形态：
# 三方对抗每一轮打中之后的改法，各留一条「改回去它就红」的变异，改法被悄悄撤回时这里先响。
```

### crates/mutations.tsv:1424-1429（新加的 6 行：名以「C331 甲」「C393 (b)-从映射现算」「收口表第 43 行丁-defer」「收口表第 ② 行算进」起头）

```text
C331 甲（用户 2026-09-28 定）：可写挂载见证读缺一槽不判不出，回到「读不出的槽不进 max」	crates/singlefs-core/src/mount.rs	    let comparison = if some_system_configuration_slot_unread {	    let comparison = if false && some_system_configuration_slot_unread {	-p singlefs-harness --test a_writable_mount_refuses_when_a_system_configuration_slot_does_not_verify -- a_writable_mount_refuses_before_acquisition_when_the_first_devices	a_writable_mount_refuses_before_acquisition_when_the_first_devices_newest_system_configuration_slot_does_not_verify
C331 甲 V4（用户 2026-09-28 定）：取号那一刻缺一槽照样取号，回到「一份都没有才拒」	crates/singlefs-core/src/transaction.rs	                < SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE\n            {\n                return Err(	                < 1\n            {\n                return Err(	-p singlefs-harness --test acquisition_writes_the_witnessed_journal_tail_so_two_crashed_acquisitions_still_witness_the_newest_publish -- an_acquisition_refuses_the_first_device	an_acquisition_refuses_the_first_device_that_lost_only_its_newest_slot_before_writing_anything
C393 (b)-从映射现算（用户 2026-09-28 定）：从映射一律现算不成，回到读不出就重读、再拒可写	crates/singlefs-core/src/mount.rs	    root: &RootRecord,\n) -> Option<Vec<(DeviceIdentity, crate::address::SlotNumber, u64)>> {\n    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");\n    let mapping_tree	    root: &RootRecord,\n) -> Option<Vec<(DeviceIdentity, crate::address::SlotNumber, u64)>> {\n    if root.checkpoint_txg.0 != u64::MAX {\n        return None;\n    }\n    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");\n    let mapping_tree	-p singlefs-harness --test an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping -- c393_tree_table_unreadable_on_every_read	c393_tree_table_unreadable_on_every_read
收口表第 43 行丁-defer（用户 2026-09-28 定）：I-3.1 的遍历不并 F 之下仍在 defer 的槽	crates/singlefs-checker/src/walk.rs	                + quarantined_exempted\n                + deferred_below_the_floor;	                + quarantined_exempted\n                + deferred_below_the_floor * 0;	-p singlefs-harness --test raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic -- raising_the_floor_into_the_gap	raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_every_invariant_green
收口表第 ② 行算进（用户 2026-09-28 定）：实现的 F_生效 不算被抛弃根带的 F	crates/singlefs-core/src/recovery.rs	        .chain(highest_on_the_abandoned_roots)	        .chain(highest_on_the_abandoned_roots.filter(|_| false))	-p singlefs-harness --test an_abandoned_roots_rollback_floor_counts_toward_the_effective_floor -- the_implementations_effective_floor	the_implementations_effective_floor_counts_the_floor_an_abandoned_root_carries
收口表第 ② 行算进（用户 2026-09-28 定）：checker 的 F_生效 不算被抛弃根带的 F	crates/singlefs-checker/src/walk.rs	        .max(highest_floor_on_the_abandoned_roots);	        .max(highest_floor_on_the_abandoned_roots * 0);	-p singlefs-harness --test an_abandoned_roots_rollback_floor_counts_toward_the_effective_floor -- the_checkers_effective_floor	the_checkers_effective_floor_counts_the_floor_an_abandoned_root_carries
```
