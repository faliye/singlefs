# 附录二：里程碑「第二个事务」增补 1 与增补 2 第一波代码改动（工作区无提交点，按「文件::项名」抽取；生成时刻 2026-09-17 18:15 UTC）

工作区没有提交点，给不出 `git diff`。按正文「一、被判的对象」那张「文件 / 项」表，逐项用
`python3 research/scripts/quote-rust-items.py 文件::项名 …` 整段抽（每段前自带文件名与行区间，
回读逐字节比对，四次调用全部退出码 0）；`write_accounting.rs` 是新文件，附全文；
`crates/mutations.tsv` 不是 Rust 源码，直接按行区间贴。三部分合起来覆盖表里全部十行。

## 一、按「文件::项名」抽取的代码段（`quote-rust-items.py`，回读逐字节比对，四次调用共 50 项、全部退出码 0）

### crates/singlefs-core/src/transaction.rs:64-88（CommitStep）

```rust
/// 提交步骤的封闭枚举：五种步骤种类（D17（实现分层与第三方管道） 已定项 2），`match` 不写通配臂——少一个臂编译不过。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommitStep<'publish> {
    WriteUnitToEveryDevice {
        slot: SlotNumber,
        unit: &'publish [u8],
        /// 这个单元的角色：只供按结构种类记账（增补 1），不影响写到哪、怎么写。
        identity: TransactionUnit,
    },
    WriteJournalRecordToEveryDevice {
        counter: u64,
        record: &'publish [u8],
    },
    /// 根槽 FUA 写：落区域 `txg mod 3` 的槽 `(txg div 3) mod 8`，写在那个区域归属的盘上。
    WriteRootRecordForceUnitAccess {
        checkpoint_txg: CheckpointTxg,
        root_slot: &'publish [u8],
    },
    /// 每盘一次超级块槽原地覆写：世代号 = 这块盘两槽里自证过的最大世代号 + 1，槽 = 世代号 mod 2（D22（单元原子性怎么合成） 已定项 16，逐盘计）。
    RotateSuperblockSlots {
        journal_tail: u64,
        journal_instance: InstanceGeneration,
    },
    Barrier,
}
```
### crates/singlefs-core/src/transaction.rs:117-182（perform）

```rust
    pub fn perform(&mut self, step: CommitStep<'_>) -> Result<(), BlockDeviceError> {
        if !matches!(step, CommitStep::Barrier) {
            self.has_writes_since_barrier = true;
        }
        match step {
            CommitStep::WriteUnitToEveryDevice {
                slot,
                unit,
                identity,
            } => {
                for (_, device) in self.devices.iter_mut() {
                    device.write_at(slot.to_device_offset(), unit, WriteDurability::Plain)?;
                    let kind = WrittenStructureKind::of_unit(identity);
                    self.writes_by_structure_kind.count_write_call(kind, unit);
                }
            }
            CommitStep::WriteJournalRecordToEveryDevice { counter, record } => {
                let offset = record_offset(counter, self.parameters.geometry.journal_ring_bytes);
                for (_, device) in self.devices.iter_mut() {
                    device.write_at(offset, record, WriteDurability::Plain)?;
                    self.writes_by_structure_kind
                        .count_write_call(WrittenStructureKind::JournalRecord, record);
                }
            }
            CommitStep::WriteRootRecordForceUnitAccess {
                checkpoint_txg,
                root_slot,
            } => {
                let target = target_for_publish(checkpoint_txg);
                let region_device =
                    self.parameters.region_devices[usize::try_from(target.region).expect("区域号")];
                let (_, device) = self
                    .devices
                    .iter_mut()
                    .find(|(identity, _)| *identity == region_device)
                    .expect("mkfs 的 check_geometry 核过区域归属");
                device.write_at(
                    slot_offset(
                        target,
                        self.parameters.geometry.fixed_structure_slot_spacing,
                    ),
                    root_slot,
                    WriteDurability::ForceUnitAccess,
                )?;
                self.writes_by_structure_kind
                    .count_write_call(WrittenStructureKind::RootSlot, root_slot);
            }
            CommitStep::RotateSuperblockSlots {
                journal_tail,
                journal_instance,
            } => {
                for index in 0..self.devices.len() {
                    self.write_superblock_slot(index, journal_tail, journal_instance)?;
                }
            }
            CommitStep::Barrier => {
                if self.has_writes_since_barrier {
                    for (_, device) in self.devices.iter_mut() {
                        device.barrier()?;
                    }
                    self.has_writes_since_barrier = false;
                }
            }
        }
        Ok(())
    }
```
### crates/singlefs-core/src/transaction.rs:184-224（write_superblock_slot）

```rust
    /// 一块盘写一次超级块槽：世代号 = 这块盘两槽里自证过的最大世代号 + 1（两槽都读不出时从 1 起），槽 = 世代号 mod 2。
    fn write_superblock_slot(
        &mut self,
        index: usize,
        journal_tail: u64,
        journal_instance: InstanceGeneration,
    ) -> Result<(), BlockDeviceError> {
        let spacing = u64::from(self.parameters.geometry.fixed_structure_slot_spacing);
        let identity = self.devices[index].0;
        let slot_generation = verified_superblock_slots(
            &*self.devices,
            identity,
            spacing,
            &self.parameters.filesystem_identifier,
        )
        .iter()
        .map(|superblock| superblock.slot_generation)
        .max()
        .unwrap_or(0)
            + 1;
        let superblock = Superblock {
            filesystem_identifier: self.parameters.filesystem_identifier,
            this_device: identity,
            device_count: u32::try_from(self.devices.len()).expect("设备数"),
            slot_generation,
            region_devices: self.parameters.region_devices,
            geometry: self.parameters.geometry,
            journal_tail,
            journal_instance,
        };
        self.has_writes_since_barrier = true;
        let slot_bytes = superblock.to_slot();
        self.devices[index].1.write_at(
            DeviceOffsetInBytes((slot_generation % SUPERBLOCK_SLOTS_PER_DEVICE) * spacing),
            &slot_bytes,
            WriteDurability::Plain,
        )?;
        self.writes_by_structure_kind
            .count_write_call(WrittenStructureKind::SuperblockSlot, &slot_bytes);
        Ok(())
    }
```

### crates/singlefs-core/src/transaction.rs:387-395（WarmUpOutput）

```rust
/// 暖机写出的东西：两代根、两条空记录、最后一条记录的字节（下一条的反向链要罩它）、每次空发布按结构种类的写。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WarmUpOutput {
    pub roots: Vec<RootRecord>,
    pub records: Vec<JournalRecord>,
    pub last_record_bytes: Vec<u8>,
    /// 与 `roots` 同序，一次空发布一份。
    pub writes: Vec<WritesByStructureKind>,
}
```
### crates/singlefs-core/src/transaction.rs:400-416（warm_up）

```rust
/// 暖机（D16（发布语义） 已定项 8）：mkfs 之后第一次可写挂载的两次空发布，checkpoint_txg 走 1、2；环在 mkfs 之后是空的，
/// jsn 从 1 起。其余见 `warm_up_after_journal_counter`。
///
/// # Errors
/// 块设备报的错原样交回。
pub fn warm_up<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    genesis: &RootRecord,
    instance: InstanceGeneration,
) -> Result<WarmUpOutput, BlockDeviceError> {
    warm_up_after_journal_counter(
        pool,
        genesis,
        instance,
        LAST_JOURNAL_COUNTER_AFTER_MAKE_FILESYSTEM,
    )
}
```
### crates/singlefs-core/src/transaction.rs:418-461（warm_up_after_journal_counter）

```rust
/// 暖机的两次空发布，接在环里 jsn 计数器为 `last_journal_counter` 的那条记录之后：checkpoint_txg 照格式常量走 1、2
/// （`WARM_UP_EMPTY_PUBLISHES`，D16（发布语义） 已定项 8），jsn 与超级块里的 tail 按记录号接着数、不取 txg
/// （D23（journal 的角色与格式） 已定项 18：tail 存 jsn 的 48 位计数器；已定项 14 第 3 条：计数器全池接着走；C366（暖机路径把 txg 写进计数器与 tail））。
/// 空记录不点名任何单元、事务号 0、提交标记 1、新根段照 mkfs 的根（D16（发布语义） 已定项 9 / D23（journal 的角色与格式） 已定项 19），
/// 根记录只改 checkpoint_txg 与实例代号。今天只有 `warm_up` 调它，环是空的、两个量按构造相等；给不相等的起点才分得出它们。
///
/// # Errors
/// 块设备报的错原样交回。
pub fn warm_up_after_journal_counter<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    genesis: &RootRecord,
    instance: InstanceGeneration,
    last_journal_counter: u64,
) -> Result<WarmUpOutput, BlockDeviceError> {
    let mut roots = Vec::new();
    let mut records = Vec::new();
    let mut writes = Vec::new();
    let mut previous_record_bytes: Option<Vec<u8>> = None;
    let mut previous_counter = last_journal_counter;
    for txg_number in 1..=WARM_UP_EMPTY_PUBLISHES {
        let output = publish_without_units(
            pool,
            genesis,
            ZeroUnitPublishPlan {
                txg: CheckpointTxg(txg_number),
                counter: previous_counter + 1,
                instance,
                back_chain: previous_record_bytes.as_deref().map_or(0, back_chain_of),
                rollback_floor: genesis.rollback_floor,
            },
        )?;
        previous_counter = output.record.counter;
        roots.push(output.root);
        records.push(output.record);
        writes.push(output.writes);
        previous_record_bytes = Some(output.record_bytes);
    }
    Ok(WarmUpOutput {
        roots,
        records,
        last_record_bytes: previous_record_bytes.expect("暖机至少一次"),
        writes,
    })
}
```
### crates/singlefs-core/src/transaction.rs:1061-1112（publish_first_file）

```rust
/// 第一个事务（字节表七 t1..t8 + 六 + 七）：分配八个落点、装八个单元、按 D16（发布语义） 已定项 7 的持久顺序落盘。
///
/// # Errors
/// txg 与 jsn 写死 3（`FIRST_TRANSACTION_TXG`），只接得上 txg 2、jsn 2 的那一版（两次暖机之后）：`previous_record_bytes` 解不出本池的记录，
/// 或它的 checkpoint_txg、jsn 不是 2 ⇒ `FirstFileVersionNotRightAfterTheSecondWarmUp`，一个字节都不写（m2-emptypool-nonempty-r1 云端攻方腿
/// Z3-A：接在 txg 4 的暖机根后面照写 txg 3，盖在暖机根上、冷恢复读不到）。检查的是记录不是 `genesis`：mkfs 同一个进程里的第一个事务
/// 传的是 mkfs 的第 0 代根，它只供照抄实例表指针。其余同 `publish_version`。
pub fn publish_first_file<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    genesis: &RootRecord,
    file: FirstFile<'_>,
    instance: InstanceGeneration,
    previous_record_bytes: &[u8],
) -> Result<TransactionOutput, PublishError> {
    let txg = CheckpointTxg(FIRST_TRANSACTION_TXG);
    let previous_record = JournalRecord::parse(
        previous_record_bytes,
        unit_filesystem_identifier(&pool.parameters.filesystem_identifier),
    )
    .map(|record| (record.checkpoint_txg, record.counter));
    let follows_directly = previous_record.is_some_and(|(previous_txg, previous_counter)| {
        previous_txg.0 + 1 == FIRST_TRANSACTION_TXG && previous_counter + 1 == FIRST_TRANSACTION_TXG
    });
    if !follows_directly {
        return Err(PublishError::FirstFileVersionNotRightAfterTheSecondWarmUp { previous_record });
    }
    let (_, previous_counter) =
        previous_record.expect("follows_directly 为真时上一条记录解得出（is_some_and）");
    publish_version(
        pool,
        allocator,
        PublishPlan {
            txg,
            counter: previous_counter + 1,
            transaction: FIRST_TRANSACTION_NUMBER,
            instance,
            back_chain: back_chain_of(previous_record_bytes),
            file: Some(FileVersionPlan {
                content: file.content,
                write_time_seconds: file.write_time_seconds,
                inode_object_birth: txg,
                change_count: 1,
            }),
            instance_table: InstanceTablePlan::Carry(genesis.instance_table),
            tree_birth_txg: txg,
            tree_identifier_watermark: TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH,
            rollback_floor: CheckpointTxg(0),
        },
        None,
    )
}
```

### crates/singlefs-core/src/transaction.rs:892-939（PublishError）

```rust
/// 发布能出的错：单元区放不下（哪一个单元没拿到落点），或底层块设备错。
#[derive(Debug)]
pub enum PublishError {
    NoSpaceFor {
        unit: TransactionUnit,
    },
    /// 分配记录树第一版只有一个节点，这次发布之后的记录数装不下（每次发布每盘加 8 条、释放只改写不删）。
    AllocationRecordsExceedOneNode {
        records: usize,
        capacity: usize,
    },
    /// 记账树第一版只有一个节点，这次发布要写的记账行装不下：行数 = 池级 3 行 + 每块盘 6 行，只随盘数变（477 条的节点在第 80 块盘时装不下）。
    AccountingEntriesExceedOneNode {
        entries: usize,
        capacity: usize,
    },
    /// 释放判定路径在上一版的映射里查不到这个单元（D19（块指针的结构与宽度预算） 已定项 5 第 1 条：不按提示释放）。
    ReleaseNotInMapping {
        unit: TransactionUnit,
    },
    /// 映射查出来的落点在分配记录里没有条目。
    ReleaseTargetNotAllocated {
        unit: TransactionUnit,
        slot: SlotNumber,
    },
    /// 映射查出来的落点已经是已释放的（同一个落点释放两次）。
    ReleaseTargetAlreadyReleased {
        unit: TransactionUnit,
        slot: SlotNumber,
    },
    /// 分配记录里的跨度与这个单元种类该有的跨度不符（映射给槽、记录给跨度，两边对不上）。
    ReleaseSpanMismatch {
        unit: TransactionUnit,
        slot: SlotNumber,
        recorded_span: u64,
        expected_span: u64,
    },
    /// 用户给的内容装不进一个数据单元（32768 − 头 − 预留）。
    ContentExceedsDataUnit {
        bytes: usize,
        capacity: usize,
    },
    /// `publish_first_file` 写死 txg 3、jsn 3，而上一版的记录不是 txg 2、jsn 2（`None` = 那几个字节解不出本池的记录）：接上去会盖在别的发布上。
    FirstFileVersionNotRightAfterTheSecondWarmUp {
        previous_record: Option<(CheckpointTxg, u64)>,
    },
    BlockDevice(BlockDeviceError),
}
```
### crates/singlefs-core/src/transaction.rs:1151-1217（publish_version）

```rust
/// 发布一版：先做准入（内容装得进一个数据单元、分配记录树装得下这次的记录），再释放上一版被换下的角色的落点、分配、装单元、
/// 按 D16（发布语义） 已定项 7 的持久顺序落盘。准入之后任何一步失败，分配器退回到进来时的样子——这次发布没有成立，
/// 释放与分配都不算数（第二轮攻方腿：`NoSpaceFor` 在释放之后、分配到一半返回，留下半新的池，拿同一个上一版重试撞断言）。
///
/// # Errors
/// `ContentExceedsDataUnit`、`AllocationRecordsExceedOneNode`、`AccountingEntriesExceedOneNode`、释放判定路径的四种错、`NoSpaceFor`、块设备错。
pub fn publish_version<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    plan: PublishPlan<'_>,
    previous: Option<&TransactionOutput>,
) -> Result<TransactionOutput, PublishError> {
    let rewritten = plan.rewritten_roles();
    // 释放判定路径先于准入：它只查不改（D19（块指针的结构与宽度预算） 已定项 5 第 1 条），查不到就整次发布不做。
    let release = match previous {
        Some(previous_version) => {
            placements_to_release_via_mapping(previous_version, allocator, &rewritten)?
        }
        // 第一个文件版本没有上一版的内存态：它重写树表时换下的是 mkfs 那片第 0 版树表单元，照样进 defer 队列
        // （D3（空间分配） 已定项 7；不释放它，txg 0 的根离开候选集之后这一槽就永远占着，I-3.1（已分配统计对得上） 在抬 F 之后红）。
        None => format_time_tree_table_to_release(allocator, &rewritten),
    };
    // 用户给的内容装不进一个数据单元是调用方能恢复的失败，不是不变量被破坏：报错，不走到 build_data_unit 的断言。
    if let Some(file) = &plan.file {
        let data_unit_capacity = data_unit_payload_capacity();
        if file.content.len() > data_unit_capacity {
            return Err(PublishError::ContentExceedsDataUnit {
                bytes: file.content.len(),
                capacity: data_unit_capacity,
            });
        }
    }
    // 分配记录树第一版只有一个节点：这次发布之后装不下就在动分配器之前报错，不许走到 build_index_node 的断言
    // （三方代码第一轮攻方腿：第 50 次覆盖写 panic）。释放只改写记录、不加；这次重写的每个角色每盘各加一条。
    let records_after_this_publish =
        allocator.records().len() + rewritten.len() * allocator.devices.len();
    let allocation_node_capacity = index_node_entry_capacity(
        usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect("10"),
        usize::try_from(ALLOCATION_RECORD_BYTES).expect("20"),
    );
    if records_after_this_publish > allocation_node_capacity {
        return Err(PublishError::AllocationRecordsExceedOneNode {
            records: records_after_this_publish,
            capacity: allocation_node_capacity,
        });
    }
    // 记账树第一版也只有一个节点：行数只随盘数变，装不下同样在动分配器之前报错，不走到 build_index_node 的断言
    // （代码三方第二轮攻方腿 Y3：约 80 块盘）。装行那一段按同一个 `allocator.devices` 装，并断言行数与这里算的相等。
    let accounting_entries_of_this_publish = accounting_entry_count(allocator.devices.len());
    let accounting_node_capacity = index_node_entry_capacity(
        usize::try_from(ACCOUNTING_KEY_BYTES).expect("22"),
        usize::try_from(ACCOUNTING_ENTRY_BYTES).expect("34"),
    );
    if accounting_entries_of_this_publish > accounting_node_capacity {
        return Err(PublishError::AccountingEntriesExceedOneNode {
            entries: accounting_entries_of_this_publish,
            capacity: accounting_node_capacity,
        });
    }
    // 整个分配器拷一份、失败就换回去：第一版每次发布约 450 KiB 的拷贝（两盘各一张 211968 位的位图），换来失败路径不留半新的池。
    let allocator_before_this_publish = allocator.clone();
    let outcome = publish_admitted(pool, allocator, &plan, previous, &rewritten, &release);
    if outcome.is_err() {
        *allocator = allocator_before_this_publish;
    }
    outcome
}
```
### crates/singlefs-core/src/transaction.rs:1224-1227（accounting_entry_count）

```rust
/// 一次发布写的记账行数：池级行加每块盘各一组。准入与装行共用这一处。
fn accounting_entry_count(device_count: usize) -> usize {
    POOL_WIDE_ACCOUNTING_ENTRIES + ACCOUNTING_ENTRIES_PER_DEVICE * device_count
}
```

### crates/singlefs-core/src/transaction.rs:1269-1398（build_file_version_units）

```rust
/// 装一个文件对象的四个单元（字节表二、四、四·二）。出生序号从调用方传进来的发号器取，发号器的作用域是一次 checkpoint、
/// 不是这一次调用（D19（块指针的结构与宽度预算） 已定项 9：同一棵树内每写出一个码 2 或码 3 单元加 1，作用域换到下一个 checkpoint 时清零）：
/// 同一个 checkpoint 里装第二个对象时序号接着数，不在同一个 (树, txg, 实例) 上从 0 重数、撞出重复的映射 key（里程碑「第二个事务」增补 2 第 19 行）。
fn build_file_version_units(
    checkpoint: &FileVersionCheckpoint<'_>,
    file: &FileVersionPlan<'_>,
    slots: FileVersionSlots,
    sequences: &mut BirthSequenceAllocator,
) -> FileVersionUnits {
    let txg = checkpoint.txg;
    let write_order = checkpoint.write_order;
    let instance = write_order.instance;
    let filesystem_identifier = checkpoint.filesystem_identifier;
    // t1 数据单元（字节表二）。
    let data_identity = DataUnitIdentity {
        tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT),
        object: FIRST_INODE_NUMBER,
        object_birth: file.inode_object_birth,
        anchor_offset: 0,
    };
    let data_unit = build_data_unit(
        data_identity,
        txg,
        filesystem_identifier,
        write_order,
        file.content,
    );
    let data_pointer = DataPointer {
        head: PointerHead {
            birth_tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT),
            birth_txg: txg,
        },
        locations: location_entries(checkpoint.device_identities, slots.data, &data_unit),
        write_order,
    };
    let extent_record = build_extent_record(FIRST_INODE_NUMBER, 0, data_pointer);
    let extent_key: [u8; 24] = extent_record[..usize::try_from(EXTENT_KEY_BYTES).expect("24")]
        .try_into()
        .expect("24");
    let key_order_mismatches = count_key_order_mismatches(&[extent_key]);

    // t3 inode 树叶容器（字节表四）：出生序号先于 t4 的根发号（树内先叶后根）。
    // 容器身份（容器号、容器出生代）在树建起来那次定下，之后每一版重写都不变（D8（核心索引结构） 已定项 6：一片叶活很多代、反复重写）。
    let inode_leaf_identity = PackedIdentity {
        birth_tree: TreeIdentifier(TREE_IDENTIFIER_INODE),
        record_type: PACKED_TYPE_INODE,
        container: FIRST_INODE_NUMBER,
        container_birth: checkpoint.tree_birth_txg,
    };
    let inode_leaf_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_INODE), txg, instance);
    let inode_record = InodeRecord {
        inode: FIRST_INODE_NUMBER,
        object_birth: file.inode_object_birth,
        size: u64::try_from(file.content.len()).expect("文件长度"),
        change_count: file.change_count,
        write_time_seconds: file.write_time_seconds,
    };
    let inode_leaf_unit = build_packed_unit(
        inode_leaf_identity,
        u16::try_from(INODE_RECORD_BYTES).expect("140"),
        &[inode_record.to_bytes()],
        txg,
        filesystem_identifier,
        write_order,
        inode_leaf_sequence,
    );
    let inode_leaf_pointer = NodePointer {
        head: PointerHead {
            birth_tree: TreeIdentifier(TREE_IDENTIFIER_INODE),
            birth_txg: txg,
        },
        locations: location_entries(
            checkpoint.device_identities,
            slots.inode_leaf,
            &inode_leaf_unit,
        ),
        instance,
        birth_sequence: inode_leaf_sequence,
    };

    // t2 extent 树根兼叶（字节表四·二）。
    let extent_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_EXTENT), txg, instance);
    let extent_unit = build_index_node(
        TreeIdentifier(TREE_IDENTIFIER_EXTENT),
        0,
        usize::try_from(EXTENT_KEY_BYTES).expect("24"),
        &extent_key,
        &extent_key,
        txg,
        filesystem_identifier,
        instance,
        extent_sequence,
        u16::try_from(EXTENT_LEAF_RECORD_BYTES).expect("112"),
        std::slice::from_ref(&extent_record),
    );

    // t4 inode 树根：层级 1，key 区间 [ino, ino]，一条 120 字节条目。
    let inode_root_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_INODE), txg, instance);
    let inode_key = FIRST_INODE_NUMBER.to_le_bytes();
    let inode_root_unit = build_index_node(
        TreeIdentifier(TREE_IDENTIFIER_INODE),
        1,
        8,
        &inode_key,
        &inode_key,
        txg,
        filesystem_identifier,
        instance,
        inode_root_sequence,
        u16::try_from(INODE_INTERNAL_ENTRY).expect("120"),
        &[build_inode_internal_entry(
            FIRST_INODE_NUMBER,
            inode_leaf_identity,
            inode_leaf_pointer,
        )],
    );
    FileVersionUnits {
        data_unit,
        data_pointer,
        extent_unit,
        extent_sequence,
        key_order_mismatches,
        inode_leaf_unit,
        inode_leaf_pointer,
        inode_leaf_sequence,
        inode_record,
        inode_root_unit,
        inode_root_sequence,
    }
}
```

### crates/singlefs-core/src/transaction.rs:1435-2009（publish_admitted）

```rust
fn publish_admitted<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    plan: &PublishPlan<'_>,
    previous: Option<&TransactionOutput>,
    rewritten: &[TransactionUnit],
    release: &[Placement],
) -> Result<TransactionOutput, PublishError> {
    let txg = plan.txg;
    let instance = plan.instance;
    let write_order = WriteOrder {
        instance,
        transaction: plan.transaction,
    };
    let filesystem_identifier = &pool.parameters.filesystem_identifier;
    let mut sequences = BirthSequenceAllocator::default();

    // 释放先于分配（D3（空间分配） 已定项 7）：被换下的落点进 defer 队列、槽仍占着，这次的落点不会落到它们上面。
    for placement in release {
        allocator.release(*placement, txg);
    }

    // 落点先于内容：分配记录树要装自己那条（D3（空间分配） 已定项 5），所以这次重写的落点在装任何单元之前全部取定。
    let mut slots: BTreeMap<TransactionUnit, SlotNumber> = BTreeMap::new();
    for identity in rewritten {
        let placement = match identity.placement() {
            PlacementRule::UserData => allocator.allocate_user_data(txg),
            PlacementRule::CommitGenerated(footprint) => {
                allocator.allocate_commit_generated(footprint, txg)
            }
        };
        let placement = placement.ok_or(PublishError::NoSpaceFor { unit: *identity })?;
        slots.insert(*identity, placement.slot);
    }
    let slot_of = |identity: TransactionUnit| slots[&identity];

    // 文件角色：有新版本就装四个单元，没有就照抄上一版的指针与字节。
    let carried_file = match &plan.file {
        Some(_) => None,
        None => Some(previous.expect("没有文件版本的发布要接在上一版之后：文件角色从它照抄")),
    };
    let FileVersionUnits {
        data_unit,
        data_pointer,
        extent_unit,
        extent_sequence,
        key_order_mismatches,
        inode_leaf_unit,
        inode_leaf_pointer,
        inode_leaf_sequence,
        inode_record,
        inode_root_unit,
        inode_root_sequence,
    } = match (&plan.file, carried_file) {
        (Some(file), _) => {
            let device_identities: Vec<DeviceIdentity> =
                pool.devices.iter().map(|(identity, _)| *identity).collect();
            build_file_version_units(
                &FileVersionCheckpoint {
                    txg,
                    write_order,
                    filesystem_identifier,
                    device_identities: &device_identities,
                    tree_birth_txg: plan.tree_birth_txg,
                },
                file,
                FileVersionSlots {
                    data: slot_of(TransactionUnit::Data),
                    inode_leaf: slot_of(TransactionUnit::InodeLeaf),
                },
                &mut sequences,
            )
        }
        (None, Some(carried)) => carried_file_version_units(carried),
        (None, None) => {
            unreachable!("上面按 plan.file 分过：没有文件版本时 carried_file 一定是 Some")
        }
    };
    // 实例表单元（码 3 打包记录类型 4）：重写时容器身份照 mkfs（容器 0、出生代 0），归树 0、在树表之前发号（mkfs 也是先实例表后树表）。
    let instance_table_rewrite = match &plan.instance_table {
        InstanceTablePlan::Rewrite(records) => {
            let sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_NONE), txg, instance);
            let unit = build_packed_unit(
                PackedIdentity {
                    birth_tree: TreeIdentifier(TREE_IDENTIFIER_NONE),
                    record_type: PACKED_TYPE_INSTANCE_TABLE,
                    container: 0,
                    container_birth: CheckpointTxg(0),
                },
                u16::try_from(INSTANCE_ROW_BYTES).expect("88"),
                records,
                txg,
                filesystem_identifier,
                write_order,
                sequence,
            );
            let pointer = NodePointer {
                head: PointerHead {
                    birth_tree: TreeIdentifier(TREE_IDENTIFIER_NONE),
                    birth_txg: txg,
                },
                locations: pool.location_entries(slot_of(TransactionUnit::InstanceTable), &unit),
                instance,
                birth_sequence: sequence,
            };
            Some((unit, pointer, sequence))
        }
        InstanceTablePlan::Carry(_) => None,
    };
    let instance_table_pointer = match (&plan.instance_table, &instance_table_rewrite) {
        (InstanceTablePlan::Carry(pointer), _) => *pointer,
        (InstanceTablePlan::Rewrite(_), Some((_, pointer, _))) => *pointer,
        (InstanceTablePlan::Rewrite(_), None) => unreachable!("重写时上面一定装了单元"),
    };

    // t5 分配记录树（字节表五）：mkfs 的两个单元分配代 0、其余是各自分配那次发布的 txg，每盘各一条，按 (设备, 槽号) 升序。
    let mut allocation_records: Vec<AllocationRecord> = allocator.records().to_vec();
    allocation_records.sort_by_key(AllocationRecord::sort_key);
    let allocation_sequence = sequences.next(
        TreeIdentifier(TREE_IDENTIFIER_ALLOCATION_RECORDS),
        txg,
        instance,
    );
    let allocation_unit = build_index_node(
        TreeIdentifier(TREE_IDENTIFIER_ALLOCATION_RECORDS),
        0,
        usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect("10"),
        &allocation_records[0].key_bytes(),
        &allocation_records[allocation_records.len() - 1].key_bytes(),
        txg,
        filesystem_identifier,
        instance,
        allocation_sequence,
        u16::try_from(ALLOCATION_RECORD_BYTES).expect("20"),
        &allocation_records
            .iter()
            .map(AllocationRecord::to_bytes)
            .collect::<Vec<_>>(),
    );

    // t6 记账树（D5（快照 / 空间记账机制） 已定项 8）：两盘 15 行——带设备维的六项每盘一行、池级三行；
    // 全部来自分配器在分配那一刻增量维护的数，不扫盘（`.claude/rules/fs-design.md` 第一格）。seq 一律 1（D8（核心索引结构） 已定项 10）。
    let pool_wide = |statistic: u16, tree: u64, value: u64| AccountingEntry {
        statistic,
        tree: TreeIdentifier(tree),
        device: DeviceIdentity(STATISTIC_NO_DEVICE_DIMENSION),
        generation: txg,
        value,
        sequence: ACCOUNTING_SEQUENCE_DIRECT_TO_LEAF,
    };
    let mut accounting_entries = vec![
        pool_wide(
            STATISTIC_INODE_WATERMARK,
            TREE_IDENTIFIER_INODE,
            FIRST_INODE_NUMBER + 1,
        ),
        pool_wide(STATISTIC_PENDING_DELETE_BYTES, TREE_IDENTIFIER_NONE, 0),
        pool_wide(
            STATISTIC_COMMITTED_RESERVATION_BYTES,
            TREE_IDENTIFIER_NONE,
            0,
        ),
    ];
    for device_map in &allocator.devices {
        let per_device = |statistic: u16, value: u64| AccountingEntry {
            statistic,
            tree: TreeIdentifier(TREE_IDENTIFIER_NONE),
            device: device_map.device,
            generation: txg,
            value,
            sequence: ACCOUNTING_SEQUENCE_DIRECT_TO_LEAF,
        };
        accounting_entries.push(per_device(
            STATISTIC_ALLOCATED_BYTES,
            device_map.allocated_slots() * SLOT_BYTES,
        ));
        // 第 2 项独立维护：分配器在分配那一刻减它，不由「容量 − 已分配」现算（D5（快照 / 空间记账机制） 已定项 4 的 ⚠️）。
        accounting_entries.push(per_device(
            STATISTIC_FREE_BYTES,
            device_map.free_slots() * SLOT_BYTES,
        ));
        accounting_entries.push(per_device(STATISTIC_UNRECLAIMABLE_BYTES, 0));
        // 第 5 项：已释放、还在 defer 窗口里的（它们仍算在已分配里：占着空间、被根环里的有效根引用）。
        accounting_entries.push(per_device(
            STATISTIC_DEFER_QUEUE_BYTES,
            device_map.deferred_slots() * SLOT_BYTES,
        ));
        accounting_entries.push(per_device(
            STATISTIC_FRAGMENTATION_RUNS,
            device_map.free_runs(),
        ));
        accounting_entries.push(per_device(
            STATISTIC_EMPTY_CLUSTER_SEGMENTS,
            device_map.empty_segments(),
        ));
    }
    assert_eq!(
        accounting_entries.len(),
        accounting_entry_count(allocator.devices.len()),
        "准入按 accounting_entry_count 判过装不装得下：这里装的行数要与它相等，改了行的构成要一起改那两个常量"
    );
    accounting_entries.sort_by_key(AccountingEntry::sort_key);
    let accounting_sequence =
        sequences.next(TreeIdentifier(TREE_IDENTIFIER_ACCOUNTING), txg, instance);
    let accounting_unit = build_index_node(
        TreeIdentifier(TREE_IDENTIFIER_ACCOUNTING),
        0,
        usize::try_from(ACCOUNTING_KEY_BYTES).expect("22"),
        &accounting_entries[0].key_bytes(),
        &accounting_entries[accounting_entries.len() - 1].key_bytes(),
        txg,
        filesystem_identifier,
        instance,
        accounting_sequence,
        u16::try_from(ACCOUNTING_ENTRY_BYTES).expect("34"),
        &accounting_entries
            .iter()
            .map(AccountingEntry::to_bytes)
            .collect::<Vec<_>>(),
    );

    let node_pointer =
        |tree: u64, identity: TransactionUnit, unit: &[u8], sequence: BirthSequence| NodePointer {
            head: PointerHead {
                birth_tree: TreeIdentifier(tree),
                birth_txg: txg,
            },
            locations: pool.location_entries(slot_of(identity), unit),
            instance,
            birth_sequence: sequence,
        };
    // 文件角色的指针：重写的按这次的落点算，照抄的取上一版。
    let (extent_pointer, inode_root_pointer) = match carried_file {
        None => (
            node_pointer(
                TREE_IDENTIFIER_EXTENT,
                TransactionUnit::ExtentRoot,
                &extent_unit,
                extent_sequence,
            ),
            node_pointer(
                TREE_IDENTIFIER_INODE,
                TransactionUnit::InodeRoot,
                &inode_root_unit,
                inode_root_sequence,
            ),
        ),
        Some(carried) => (
            carried.tree_root_pointer(TREE_IDENTIFIER_EXTENT),
            carried.tree_root_pointer(TREE_IDENTIFIER_INODE),
        ),
    };
    let allocation_pointer = node_pointer(
        TREE_IDENTIFIER_ALLOCATION_RECORDS,
        TransactionUnit::AllocationTree,
        &allocation_unit,
        allocation_sequence,
    );
    let accounting_pointer = node_pointer(
        TREE_IDENTIFIER_ACCOUNTING,
        TransactionUnit::AccountingTree,
        &accounting_unit,
        accounting_sequence,
    );

    // t7 中央映射树（字节表三·二）：码 1 一条 + 码 2 / 码 3 五条；映射树自己、树表、实例表豁免（D19（块指针的结构与宽度预算） 已定项 8 / 已定项 12）。
    // 照抄的文件角色 key 照旧、位置照旧；重写的按这次的指针算。
    let mapped_units_with_locations: Vec<(TransactionUnit, Vec<u8>, [LocationEntry; 2])> = vec![
        (
            TransactionUnit::Data,
            mapping_key_for_data(data_pointer.head, data_pointer.write_order),
            data_pointer.locations,
        ),
        (
            TransactionUnit::ExtentRoot,
            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, extent_pointer),
            extent_pointer.locations,
        ),
        (
            TransactionUnit::InodeLeaf,
            mapping_key_for_node(UNIT_CLASS_PACKED, inode_leaf_pointer),
            inode_leaf_pointer.locations,
        ),
        (
            TransactionUnit::InodeRoot,
            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, inode_root_pointer),
            inode_root_pointer.locations,
        ),
        (
            TransactionUnit::AllocationTree,
            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, allocation_pointer),
            allocation_pointer.locations,
        ),
        (
            TransactionUnit::AccountingTree,
            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, accounting_pointer),
            accounting_pointer.locations,
        ),
    ];
    let mapped_units: Vec<(TransactionUnit, Vec<u8>)> = mapped_units_with_locations
        .iter()
        .map(|(identity, key, _)| (*identity, key.clone()))
        .collect();
    let mut mapping_entries: Vec<(Vec<u8>, [LocationEntry; 2])> = mapped_units_with_locations
        .into_iter()
        .map(|(_, key, locations)| (key, locations))
        .collect();
    mapping_entries.sort_by_key(|(key, _)| mapping_key_sort_key(key));
    let mapping_sequence = sequences.next(
        TreeIdentifier(TREE_IDENTIFIER_CENTRAL_MAPPING),
        txg,
        instance,
    );
    let mapping_unit = build_index_node(
        TreeIdentifier(TREE_IDENTIFIER_CENTRAL_MAPPING),
        0,
        usize::try_from(MAPPING_KEY_BYTES).expect("27"),
        &mapping_entries[0].0,
        &mapping_entries[mapping_entries.len() - 1].0,
        txg,
        filesystem_identifier,
        instance,
        mapping_sequence,
        u16::try_from(MAPPING_ENTRY_BYTES).expect("55"),
        &mapping_entries
            .iter()
            .map(|(key, locations)| build_mapping_entry(key, *locations))
            .collect::<Vec<_>>(),
    );
    let mapping_pointer = node_pointer(
        TREE_IDENTIFIER_CENTRAL_MAPPING,
        TransactionUnit::MappingTree,
        &mapping_unit,
        mapping_sequence,
    );

    // t8 树表单元：七条按树 ID 升序（D8（核心索引结构） 已定项 8）；映射树的根住根记录、不进树表（D19（块指针的结构与宽度预算） 已定项 11）；
    // 头 ID（D5（快照 / 空间记账机制） 已定项 9）：inode 树写自己、extent 树写它服务的头，其余 0。
    let table_entry =
        |kind: u16, tree: u64, root: NodePointer, head_identifier: u64| TreeTableEntry {
            kind,
            tree: TreeIdentifier(tree),
            root,
            birth_txg: plan.tree_birth_txg,
            head_identifier,
        };
    let tree_table_entries = vec![
        table_entry(
            TREE_KIND_EXTENT,
            TREE_IDENTIFIER_EXTENT,
            extent_pointer,
            TREE_IDENTIFIER_INODE,
        ),
        table_entry(
            TREE_KIND_INODE,
            TREE_IDENTIFIER_INODE,
            inode_root_pointer,
            TREE_IDENTIFIER_INODE,
        ),
        table_entry(
            TREE_KIND_ALLOCATION,
            TREE_IDENTIFIER_ALLOCATION_RECORDS,
            allocation_pointer,
            0,
        ),
        table_entry(
            TREE_KIND_ACCOUNTING,
            TREE_IDENTIFIER_ACCOUNTING,
            accounting_pointer,
            0,
        ),
        table_entry(
            TREE_KIND_LIVELIST,
            TREE_IDENTIFIER_LIVELIST,
            NodePointer::empty_root(),
            0,
        ),
        table_entry(
            TREE_KIND_SPARSE_SIDE_TABLE,
            TREE_IDENTIFIER_SPARSE_SIDE_TABLE,
            NodePointer::empty_root(),
            0,
        ),
        table_entry(
            TREE_KIND_DEADLIST,
            TREE_IDENTIFIER_DEADLIST,
            NodePointer::empty_root(),
            0,
        ),
    ];
    let tree_table_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_NONE), txg, instance);
    let tree_table_unit = build_index_node(
        TreeIdentifier(TREE_IDENTIFIER_NONE),
        0,
        TREE_TABLE_KEY_WIDTH,
        &TREE_IDENTIFIER_EXTENT.to_le_bytes(),
        &TREE_IDENTIFIER_DEADLIST.to_le_bytes(),
        txg,
        filesystem_identifier,
        instance,
        tree_table_sequence,
        u16::try_from(TREE_TABLE_ENTRY_BYTES).expect("200"),
        &tree_table_entries
            .iter()
            .map(TreeTableEntry::to_bytes)
            .collect::<Vec<_>>(),
    );
    let tree_table_pointer = node_pointer(
        TREE_IDENTIFIER_NONE,
        TransactionUnit::TreeTable,
        &tree_table_unit,
        tree_table_sequence,
    );

    // 这一版全部角色的单元：重写的是这次装的，照抄的从上一版拷（八个文件 / 固定点角色按 bump 次序，实例表单元在末尾）。
    let rewritten_unit = |identity: TransactionUnit, bytes: Vec<u8>| PublishedUnit {
        slot: slot_of(identity),
        identity,
        bytes,
    };
    let mut units: Vec<PublishedUnit> = Vec::new();
    for identity in TransactionUnit::IN_BUMP_ORDER {
        let unit = match identity {
            TransactionUnit::Data => match carried_file {
                None => rewritten_unit(identity, data_unit.clone()),
                Some(carried) => carried_unit(carried, identity),
            },
            TransactionUnit::ExtentRoot => match carried_file {
                None => rewritten_unit(identity, extent_unit.clone()),
                Some(carried) => carried_unit(carried, identity),
            },
            TransactionUnit::InodeLeaf => match carried_file {
                None => rewritten_unit(identity, inode_leaf_unit.clone()),
                Some(carried) => carried_unit(carried, identity),
            },
            TransactionUnit::InodeRoot => match carried_file {
                None => rewritten_unit(identity, inode_root_unit.clone()),
                Some(carried) => carried_unit(carried, identity),
            },
            TransactionUnit::AllocationTree => rewritten_unit(identity, allocation_unit.clone()),
            TransactionUnit::AccountingTree => rewritten_unit(identity, accounting_unit.clone()),
            TransactionUnit::MappingTree => rewritten_unit(identity, mapping_unit.clone()),
            TransactionUnit::TreeTable => rewritten_unit(identity, tree_table_unit.clone()),
            TransactionUnit::InstanceTable => {
                unreachable!("IN_BUMP_ORDER 只有八个文件 / 固定点角色")
            }
        };
        units.push(unit);
    }
    match (&instance_table_rewrite, previous) {
        (Some((unit, _, _)), _) => {
            units.push(rewritten_unit(TransactionUnit::InstanceTable, unit.clone()))
        }
        (None, Some(previous_version)) => {
            if let Some(carried) = previous_version
                .units
                .iter()
                .find(|unit| unit.identity == TransactionUnit::InstanceTable)
            {
                units.push(carried.clone());
            }
        }
        (None, None) => {}
    }

    // 点名项（D23（journal 的角色与格式） 已定项 17）：这次重写的每个角色一项，key 尾段与映射 key 共用；照抄的不点名。
    let key_tail_of = |identity: TransactionUnit| -> [u8; 10] {
        match identity {
            TransactionUnit::Data => data_key_tail(data_pointer.write_order),
            TransactionUnit::ExtentRoot => node_key_tail(instance, extent_sequence),
            TransactionUnit::InodeLeaf => node_key_tail(instance, inode_leaf_sequence),
            TransactionUnit::InodeRoot => node_key_tail(instance, inode_root_sequence),
            TransactionUnit::AllocationTree => node_key_tail(instance, allocation_sequence),
            TransactionUnit::AccountingTree => node_key_tail(instance, accounting_sequence),
            TransactionUnit::MappingTree => node_key_tail(instance, mapping_sequence),
            TransactionUnit::TreeTable => node_key_tail(instance, tree_table_sequence),
            TransactionUnit::InstanceTable => node_key_tail(
                instance,
                instance_table_rewrite
                    .as_ref()
                    .map(|(_, _, sequence)| *sequence)
                    .expect("点名实例表单元的发布一定重写了它"),
            ),
        }
    };
    let written_units: Vec<&PublishedUnit> = rewritten
        .iter()
        .map(|identity| {
            units
                .iter()
                .find(|unit| unit.identity == *identity)
                .expect("重写的角色都装了单元")
        })
        .collect();
    let named: Vec<NamedUnit> = written_units
        .iter()
        .map(|unit| NamedUnit {
            locations: pool.location_entries(unit.slot, &unit.bytes),
            unit_class: unit.identity.unit_class(),
            birth_tree: unit.identity.tree(),
            birth_txg: txg,
            key_tail: key_tail_of(unit.identity),
        })
        .collect();
    let record = JournalRecord {
        instance,
        counter: plan.counter,
        checkpoint_txg: txg,
        transaction: plan.transaction,
        is_commit: true,
        back_chain: plan.back_chain,
        filesystem_identifier: unit_filesystem_identifier(filesystem_identifier),
        new_tree_table: tree_table_pointer,
        new_mapping_root: mapping_pointer,
        new_tree_identifier_watermark: plan.tree_identifier_watermark,
        new_rollback_floor: plan.rollback_floor,
        named,
    };
    let record_bytes = record.to_bytes();
    let root = RootRecord {
        filesystem_identifier: *filesystem_identifier,
        instance,
        checkpoint_txg: txg,
        tree_table: tree_table_pointer,
        tree_identifier_watermark: plan.tree_identifier_watermark,
        rollback_floor: plan.rollback_floor,
        instance_table: instance_table_pointer,
        mapping_root: mapping_pointer,
    };
    let root_slot = root.to_slot(pool.root_slot_bytes());

    // 持久顺序（D16（发布语义） 已定项 7）：这次重写的单元 → 屏障 → journal 记录 → 屏障 → 根槽 FUA → 超级块槽轮换。
    let writes_before_this_publish = pool.writes_by_structure_kind.clone();
    for unit in &written_units {
        pool.perform(CommitStep::WriteUnitToEveryDevice {
            slot: unit.slot,
            unit: &unit.bytes,
            identity: unit.identity,
        })?;
    }
    pool.perform(CommitStep::Barrier)?;
    pool.perform(CommitStep::WriteJournalRecordToEveryDevice {
        counter: plan.counter,
        record: &record_bytes,
    })?;
    pool.perform(CommitStep::Barrier)?;
    pool.perform(CommitStep::WriteRootRecordForceUnitAccess {
        checkpoint_txg: txg,
        root_slot: &root_slot,
    })?;
    pool.perform(CommitStep::RotateSuperblockSlots {
        journal_tail: plan.counter,
        journal_instance: instance,
    })?;

    Ok(TransactionOutput {
        root,
        record,
        record_bytes,
        units,
        rewritten: rewritten.to_vec(),
        data_pointer,
        mapping_keys: mapping_entries.into_iter().map(|(key, _)| key).collect(),
        allocation_records,
        accounting_entries,
        tree_table_entries,
        inode_record,
        mapped_units,
        released: release.to_vec(),
        key_order_mismatches,
        writes: pool
            .writes_by_structure_kind
            .since(&writes_before_this_publish),
    })
}
```

### crates/singlefs-core/src/transaction.rs:947-952（BirthSequenceAllocator）

```rust
/// 出生序号：同一 (树, txg, 实例) 里每写出一个码 2 / 码 3 单元加 1，从 0 起（D19（块指针的结构与宽度预算） 已定项 9）。
/// 作用域是一次 checkpoint：`publish_admitted` 每次发布建一个，这次装的每个对象（`build_file_version_units`）与固定点结构都从它取号。
#[derive(Default)]
pub struct BirthSequenceAllocator {
    counters: BTreeMap<(TreeIdentifier, CheckpointTxg, InstanceGeneration), u32>,
}
```
### crates/singlefs-core/src/transaction.rs:954-966（impl BirthSequenceAllocator）

```rust
impl BirthSequenceAllocator {
    pub fn next(
        &mut self,
        tree: TreeIdentifier,
        txg: CheckpointTxg,
        instance: InstanceGeneration,
    ) -> BirthSequence {
        let counter = self.counters.entry((tree, txg, instance)).or_insert(0);
        let sequence = BirthSequence(*counter);
        *counter += 1;
        sequence
    }
}
```

### crates/singlefs-core/src/recovery.rs:486-681（rebuild_version）

```rust
/// 从盘上按所选根重建「上一版」：全部角色的单元字节、指针、树表、分配记录、记账行与 inode 记录，交给发布路径当上一版
/// （照抄没重写的角色、经映射释放被换下的角色都靠它；可写挂载在恢复之后调）。`record_standing_for_root` 是所选根自己那条记录
/// （读不出时由调用方顶一条；树表 0 条时用不到）。
///
/// # Errors
/// 树表不是 0 条而 `record_standing_for_root` 是 `None` ⇒ `NoRecordStandingForFileVersion`；单元读不到或解不开 ⇒ 走读同款的错。
pub fn rebuild_version(
    reader: &dyn PoolReader,
    root: &RootRecord,
    record_standing_for_root: Option<JournalRecord>,
) -> Result<RebuiltVersion, RebuildVersionFailure> {
    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
    let data_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
    let instance_table_bytes =
        read_unit_via_locations(reader, &root.instance_table.locations, data_bytes)?;
    let tree_table_bytes = read_unit_via_locations(reader, &root.tree_table.locations, node_bytes)?;
    let tree_table =
        parse_index_node(&tree_table_bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
            what: "树表单元",
        })?;
    if tree_table.entries.is_empty() {
        return Ok(RebuiltVersion::WithoutFile);
    }
    let record =
        record_standing_for_root.ok_or(RebuildVersionFailure::NoRecordStandingForFileVersion)?;
    let record_bytes = record.to_bytes();
    let mut tree_table_entries = Vec::with_capacity(tree_table.entries.len());
    for bytes in &tree_table.entries {
        tree_table_entries.push(TreeTableEntry::parse(bytes).ok_or(
            RecoveryFailure::UnitMalformed {
                what: "树表条目"
            },
        )?);
    }
    let pointer_of = |kind: u16| {
        tree_table_entries
            .iter()
            .find(|entry| entry.kind == kind)
            .map(|entry| entry.root)
            .ok_or(RecoveryFailure::UnitMalformed {
                what: "树表里没有那棵树",
            })
    };
    let extent_pointer = pointer_of(TREE_KIND_EXTENT)?;
    let inode_root_pointer = pointer_of(TREE_KIND_INODE)?;
    let allocation_pointer = pointer_of(TREE_KIND_ALLOCATION)?;
    let accounting_pointer = pointer_of(TREE_KIND_ACCOUNTING)?;
    let read_node = |pointer: &NodePointer, what: &'static str| {
        let bytes = read_unit_via_locations(reader, &pointer.locations, node_bytes)?;
        let node =
            parse_index_node(&bytes).map_err(|_error| RecoveryFailure::UnitMalformed { what })?;
        Ok::<(Vec<u8>, IndexNodeHeader), RecoveryFailure>((bytes, node))
    };
    let (extent_bytes, extent_node) = read_node(&extent_pointer, "extent 树根")?;
    let (_, data_pointer) = parse_extent_record(extent_node.entries.first().ok_or(
        RecoveryFailure::UnitMalformed {
            what: "extent 树根没有记录",
        },
    )?);
    let data_unit_bytes = read_unit_via_locations(reader, &data_pointer.locations, data_bytes)?;
    let (inode_root_bytes, inode_root_node) = read_node(&inode_root_pointer, "inode 树根")?;
    let (_, _, inode_leaf_pointer) =
        parse_inode_internal_entry(inode_root_node.entries.first().ok_or(
            RecoveryFailure::UnitMalformed {
                what: "inode 树根没有条目",
            },
        )?);
    let inode_leaf_bytes =
        read_unit_via_locations(reader, &inode_leaf_pointer.locations, data_bytes)?;
    let inode_leaf =
        parse_packed_unit(&inode_leaf_bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
            what: "inode 叶容器",
        })?;
    let inode_record = InodeRecord::parse(inode_leaf.records.first().ok_or(
        RecoveryFailure::UnitMalformed {
            what: "inode 叶容器没有记录",
        },
    )?)
    .ok_or(RecoveryFailure::UnitMalformed {
        what: "inode 记录"
    })?;
    let (allocation_bytes, allocation_node) = read_node(&allocation_pointer, "分配记录树根")?;
    let allocation_records: Vec<AllocationRecord> = allocation_node
        .entries
        .iter()
        .map(|bytes| AllocationRecord::parse(bytes))
        .collect();
    let (accounting_bytes, accounting_node) = read_node(&accounting_pointer, "记账树根")?;
    let accounting_entries: Vec<AccountingEntry> = accounting_node
        .entries
        .iter()
        .map(|bytes| AccountingEntry::parse(bytes))
        .collect();
    let (mapping_bytes, mapping_node) = read_node(&root.mapping_root, "映射树根")?;
    let mapping_keys: Vec<Vec<u8>> = mapping_node
        .entries
        .iter()
        .map(|entry| parse_mapping_entry(entry).0)
        .collect();
    let mapped_units = vec![
        (
            TransactionUnit::Data,
            mapping_key_for_data(data_pointer.head, data_pointer.write_order),
        ),
        (
            TransactionUnit::ExtentRoot,
            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, extent_pointer),
        ),
        (
            TransactionUnit::InodeLeaf,
            mapping_key_for_node(UNIT_CLASS_PACKED, inode_leaf_pointer),
        ),
        (
            TransactionUnit::InodeRoot,
            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, inode_root_pointer),
        ),
        (
            TransactionUnit::AllocationTree,
            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, allocation_pointer),
        ),
        (
            TransactionUnit::AccountingTree,
            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, accounting_pointer),
        ),
    ];
    let unit =
        |identity: TransactionUnit, locations: &[LocationEntry; 2], bytes: Vec<u8>| PublishedUnit {
            slot: locations[0].slot,
            identity,
            bytes,
        };
    let units = vec![
        unit(
            TransactionUnit::Data,
            &data_pointer.locations,
            data_unit_bytes,
        ),
        unit(
            TransactionUnit::ExtentRoot,
            &extent_pointer.locations,
            extent_bytes,
        ),
        unit(
            TransactionUnit::InodeLeaf,
            &inode_leaf_pointer.locations,
            inode_leaf_bytes,
        ),
        unit(
            TransactionUnit::InodeRoot,
            &inode_root_pointer.locations,
            inode_root_bytes,
        ),
        unit(
            TransactionUnit::AllocationTree,
            &allocation_pointer.locations,
            allocation_bytes,
        ),
        unit(
            TransactionUnit::AccountingTree,
            &accounting_pointer.locations,
            accounting_bytes,
        ),
        unit(
            TransactionUnit::MappingTree,
            &root.mapping_root.locations,
            mapping_bytes,
        ),
        unit(
            TransactionUnit::TreeTable,
            &root.tree_table.locations,
            tree_table_bytes,
        ),
        unit(
            TransactionUnit::InstanceTable,
            &root.instance_table.locations,
            instance_table_bytes,
        ),
    ];
    Ok(RebuiltVersion::WithFile(TransactionOutput {
        root: *root,
        record,
        record_bytes,
        units,
        rewritten: Vec::new(),
        data_pointer,
        mapping_keys,
        allocation_records,
        accounting_entries,
        tree_table_entries,
        inode_record,
        mapped_units,
        released: Vec::new(),
        key_order_mismatches: 0,
        writes: WritesByStructureKind::NOTHING_WRITTEN,
    }))
}
```

### crates/singlefs-core/src/lib.rs:26-26（write_accounting）

```rust
pub mod write_accounting;
```
### crates/singlefs-core/src/allocator.rs:118-136（PlacementRefusal）

```rust
/// 分配器拒绝一次分配的原因。拒绝发生在动任何状态之前：记录、位图、计数、开放段与 bump 游标都照旧。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlacementRefusal {
    /// 每块盘上都没有合政策的落点。
    NoFreeSlotOnAnyDevice,
    /// 有的盘上没有合政策的落点、别的盘上还有（盘不等大时小盘先满）：第一版的设备集合就是池里全部的盘，
    /// 小盘写满之后「当时可写的设备集合」怎么选是 D2（RAID 条带策略） 已定项 2 ⚠️ 那一半，没有条款；第一版不支持。
    SomeDevicesFullDeviceSetSelectionUndefined { full_devices: Vec<DeviceIdentity> },
    /// 各盘按自己的空闲图取出的用户数据落点不同：D3（空间分配） 已定项 8 要各盘各取、D2（RAID 条带策略） 已定项 10 每个副本一条位置条目，
    /// 而 `Placement` 两盘同槽、发布路径的位置条目只带一个槽号；第一版不支持。
    UserDataSlotsDifferAcrossDevicesPerDeviceSlotsUnsupported {
        slot_per_device: Vec<(DeviceIdentity, SlotNumber)>,
    },
    /// 开放段装不下之后各盘给的去处不同（一块开段一块回落，或开的段不同）：各盘上的聚簇段要不要对齐是 D3（空间分配） 已定项 8 待办 ①，
    /// 没有条款；`Placement` 两盘同槽；第一版不支持。
    CommitGeneratedPlacementsDifferAcrossDevicesSegmentAlignmentUndefined {
        answer_per_device: Vec<(DeviceIdentity, CommitGeneratedDeviceAnswer)>,
    },
}
```
### crates/singlefs-core/src/allocator.rs:138-144（DeviceAgreement）

```rust
/// 各盘各答一个之后合成的结论。
enum DeviceAgreement<Answer> {
    Agreed(Answer),
    NoAnswerOnAnyDevice,
    SomeDevicesWithoutAnswer(Vec<DeviceIdentity>),
    AnswersDiffer(Vec<(DeviceIdentity, Answer)>),
}
```
### crates/singlefs-core/src/allocator.rs:146-174（agreement_across_devices）

```rust
/// 每块盘的答案（`None` = 这块盘答不出）合成一个结论：全答不出、部分答不出、答得不同、一致。
fn agreement_across_devices<Answer: Copy + PartialEq>(
    answer_per_device: &[(DeviceIdentity, Option<Answer>)],
) -> DeviceAgreement<Answer> {
    let devices_without_answer: Vec<DeviceIdentity> = answer_per_device
        .iter()
        .filter(|(_, answer)| answer.is_none())
        .map(|(device, _)| *device)
        .collect();
    if devices_without_answer.len() == answer_per_device.len() {
        return DeviceAgreement::NoAnswerOnAnyDevice;
    }
    if !devices_without_answer.is_empty() {
        return DeviceAgreement::SomeDevicesWithoutAnswer(devices_without_answer);
    }
    let answers: Vec<(DeviceIdentity, Answer)> = answer_per_device
        .iter()
        .filter_map(|(device, answer)| answer.map(|present| (*device, present)))
        .collect();
    let first_answer = answers
        .first()
        .expect("上面两个判断之后每块盘都有答案、且至少一块盘")
        .1;
    if answers.iter().all(|(_, answer)| *answer == first_answer) {
        DeviceAgreement::Agreed(first_answer)
    } else {
        DeviceAgreement::AnswersDiffer(answers)
    }
}
```
### crates/singlefs-core/src/allocator.rs:432-451（lowest_commit_generated_fallback_slot）

```rust
    /// 提交内生块的回落落点（D3（空间分配） 已定项 8 ②「提交内生块的段耗尽时回落到该设备内槽号最小的空槽，同样排除 `R`」）：
    /// 这块盘单元区里起点槽号最小、整个跨度都不被挡的落点——挡的是 bump 游标绕开的同一套位（已分配、影子账隔离、抬 F 扣住），
    /// 回落不是绕开影子账或扣住的第二条路；两槽的码 3 容器照数据单元那一档起点 32768 对齐（D3（空间分配） 已定项 10 ⑤）。第一版 `R` 为空。
    #[must_use]
    pub fn lowest_commit_generated_fallback_slot(
        &self,
        footprint: UnitFootprint,
    ) -> Option<SlotNumber> {
        let unit_area_end = UNIT_AREA_START_SLOT + self.unit_area_slots;
        let mut candidate = UNIT_AREA_START_SLOT;
        while candidate + footprint.slots() <= unit_area_end {
            let is_whole_span_unblocked = (candidate..candidate + footprint.slots())
                .all(|slot| !self.is_blocked_for_commit_generated(SlotNumber(slot)));
            if is_whole_span_unblocked {
                return Some(SlotNumber(candidate));
            }
            candidate += footprint.alignment_in_slots();
        }
        None
    }
```
### crates/singlefs-core/src/allocator.rs:453-467（commit_generated_answer_without_open_segment）

```rust
    /// 开放段没有或装不下时这块盘给提交内生块的去处：最低的全空段；没有全空段就回落；连回落的空槽都没有就 `None`。
    #[must_use]
    pub fn commit_generated_answer_without_open_segment(
        &self,
        footprint: UnitFootprint,
    ) -> Option<CommitGeneratedDeviceAnswer> {
        match self.lowest_empty_segment() {
            Some(segment_start) => {
                Some(CommitGeneratedDeviceAnswer::OpenEmptySegment(segment_start))
            }
            None => self
                .lowest_commit_generated_fallback_slot(footprint)
                .map(CommitGeneratedDeviceAnswer::FallBackToLowestFreeSlot),
        }
    }
```
### crates/singlefs-core/src/allocator.rs:638-641（allocate_user_data）

```rust
    /// 用户数据：见 `try_allocate_user_data`；拒绝的原因在这里丢掉，发布路径把 `None` 报成装不下。
    pub fn allocate_user_data(&mut self, generation: CheckpointTxg) -> Option<Placement> {
        self.try_allocate_user_data(generation).ok()
    }
```

### crates/singlefs-core/src/allocator.rs:643-678（try_allocate_user_data）

```rust
    /// 用户数据：每块盘按政策函数各自取（D3（空间分配） 已定项 8 第 1 条「在每一块被选中的设备上各自取该设备内」），各盘一致才分配。
    ///
    /// # Errors
    /// 拒绝时分配器一样都没动：全部盘都没有落点、有的盘没有（盘不等大时小盘先满）、各盘的落点不同。
    pub fn try_allocate_user_data(
        &mut self,
        generation: CheckpointTxg,
    ) -> Result<Placement, PlacementRefusal> {
        let open = self.open_segment;
        let slot_per_device: Vec<(DeviceIdentity, Option<SlotNumber>)> = self
            .devices
            .iter()
            .map(|device| (device.device, device.lowest_user_data_slot(open)))
            .collect();
        let slot = match agreement_across_devices(&slot_per_device) {
            DeviceAgreement::Agreed(slot) => slot,
            DeviceAgreement::NoAnswerOnAnyDevice => {
                return Err(PlacementRefusal::NoFreeSlotOnAnyDevice);
            }
            DeviceAgreement::SomeDevicesWithoutAnswer(full_devices) => {
                return Err(
                    PlacementRefusal::SomeDevicesFullDeviceSetSelectionUndefined { full_devices },
                );
            }
            DeviceAgreement::AnswersDiffer(slot_per_device) => {
                return Err(
                    PlacementRefusal::UserDataSlotsDifferAcrossDevicesPerDeviceSlotsUnsupported {
                        slot_per_device,
                    },
                );
            }
        };
        let placement = Placement { slot, span: 2 };
        self.record(placement, generation);
        Ok(placement)
    }
```
### crates/singlefs-core/src/allocator.rs:680-688（allocate_commit_generated）

```rust
    /// 提交内生块：见 `try_allocate_commit_generated`；拒绝的原因在这里丢掉，发布路径把 `None` 报成装不下。
    pub fn allocate_commit_generated(
        &mut self,
        footprint: UnitFootprint,
        generation: CheckpointTxg,
    ) -> Option<Placement> {
        self.try_allocate_commit_generated(footprint, generation)
            .ok()
    }
```
### crates/singlefs-core/src/allocator.rs:690-752（try_allocate_commit_generated）

```rust
    /// 提交内生块：从开放段 bump；容器按 32768 对齐档，节点取游标处最低空槽。开放段没有或装不下时每块盘按自己的空闲图答一个去处——
    /// 最低的全空段（开新段），没有全空段就回落到槽号最小的空槽（D3（空间分配） 已定项 8 ②）——各盘答得一致才分配。
    ///
    /// # Errors
    /// 拒绝时分配器一样都没动（装不下的开放段也照旧开着）：全部盘都没有去处、有的盘没有、各盘的去处不同。
    pub fn try_allocate_commit_generated(
        &mut self,
        footprint: UnitFootprint,
        generation: CheckpointTxg,
    ) -> Result<Placement, PlacementRefusal> {
        if let Some(open) = self.open_segment {
            if let Some(slot) = self.bump_slot_in_open_segment(open, footprint) {
                return Ok(self.record_bumped(slot, footprint, generation));
            }
        }
        let answer_per_device: Vec<(DeviceIdentity, Option<CommitGeneratedDeviceAnswer>)> = self
            .devices
            .iter()
            .map(|device| {
                (
                    device.device,
                    device.commit_generated_answer_without_open_segment(footprint),
                )
            })
            .collect();
        let answer = match agreement_across_devices(&answer_per_device) {
            DeviceAgreement::Agreed(answer) => answer,
            DeviceAgreement::NoAnswerOnAnyDevice => {
                return Err(PlacementRefusal::NoFreeSlotOnAnyDevice);
            }
            DeviceAgreement::SomeDevicesWithoutAnswer(full_devices) => {
                return Err(
                    PlacementRefusal::SomeDevicesFullDeviceSetSelectionUndefined { full_devices },
                );
            }
            DeviceAgreement::AnswersDiffer(answer_per_device) => {
                return Err(
                    PlacementRefusal::CommitGeneratedPlacementsDifferAcrossDevicesSegmentAlignmentUndefined {
                        answer_per_device,
                    },
                );
            }
        };
        match answer {
            CommitGeneratedDeviceAnswer::OpenEmptySegment(segment_start) => {
                self.open_segment = Some(segment_start);
                self.bump_cursor = segment_start.0;
                let slot = self
                    .bump_slot_in_open_segment(segment_start, footprint)
                    .expect("每块盘都答了同一个全空段（已分配、隔离、扣住都是 0，整段在每块盘的单元区里）：64 槽装得下任何一个单元");
                Ok(self.record_bumped(slot, footprint, generation))
            }
            CommitGeneratedDeviceAnswer::FallBackToLowestFreeSlot(slot) => {
                self.open_segment = None;
                let placement = Placement {
                    slot,
                    span: footprint.slots(),
                };
                self.record(placement, generation);
                Ok(placement)
            }
        }
    }
```

### crates/singlefs-harness/src/crash.rs:503-531（Layer0Tally）

```rust
/// 层 0 的计数：每个状态跑一遍看 journal 的恢复与一遍不看的，oracle 只判前者。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Layer0Tally {
    pub states: u64,
    pub violations: u64,
    pub root_persisted_states: u64,
    pub no_file_states: u64,
    pub file_read_states: u64,
    pub failed_states: u64,
    pub journal_differing_states: u64,
    pub verification_ran_states: u64,
    pub verification_failed_states: u64,
    pub first_violation: Option<String>,
    /// 不看 journal 那一遍恢复（`JournalPolicy::Ignore`）过同一个 oracle 判违例的状态数，另计、不混进 `violations`
    /// （靶向对照里「根槽已持久而单元缺席」那一格两遍都违例）。发布 B 之后两遍恢复会读出两个不同的版本，这一遍此前没人判。
    pub ignored_violations: u64,
    pub first_ignored_violation: Option<String>,
    /// 记录核对器判「根在案而记录缺席」的状态数。
    pub record_root_without_record: u64,
    /// 记录核对器判「恢复自称新态而单元缺席」的状态数。
    pub record_claimed_state_missing_unit: u64,
    /// checker：每条不变量在几个状态上评估过（成立或违例）、在几个状态上判违例、第一处违例。
    pub checker_evaluated_states: BTreeMap<&'static str, u64>,
    pub checker_violated_states: BTreeMap<&'static str, u64>,
    pub checker_first_violation: BTreeMap<&'static str, String>,
    /// checker 报「不适用」（这条不变量判的代码在这个状态上没跑到）的状态数：与评估过的状态数分开报，阴性结果不与「没跑到」混在一起
    /// （里程碑「第二个事务」步 6 验收第 3 条）；每条不变量的评估过 + 不适用 = `states`。
    pub checker_not_applicable_states: BTreeMap<&'static str, u64>,
}
```
### crates/singlefs-harness/src/crash.rs:534-552（checker_counts_by_invariant）

```rust
    /// checker 那一半按不变量报成一段：`I-x.y=评估过/判违例/不适用`，次序照 checker 的清单。
    #[must_use]
    pub fn checker_counts_by_invariant(&self) -> String {
        let count_of = |counts: &BTreeMap<&'static str, u64>, invariant: &str| {
            counts.get(invariant).copied().unwrap_or(0)
        };
        singlefs_checker::image::IMPLEMENTED_INVARIANTS
            .iter()
            .map(|invariant| {
                format!(
                    "{invariant}={}/{}/{}",
                    count_of(&self.checker_evaluated_states, invariant),
                    count_of(&self.checker_violated_states, invariant),
                    count_of(&self.checker_not_applicable_states, invariant)
                )
            })
            .collect::<Vec<String>>()
            .join(" ")
    }
```

### crates/singlefs-harness/src/crash.rs:695-793（evaluate_state_for_versions）

```rust
/// 评一个状态：跑两种 journal 政策的恢复，记进计数。`judged_root_index` 是被判的那次根槽 FUA 写（计「根槽已持久」的状态数用），
/// oracle 按 `versions` 判实际走的根该读出哪一版。
pub fn evaluate_state_for_versions(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    persisted: Vec<bool>,
    judged_root_index: usize,
    versions: &[PublishedVersion],
    tally: &mut Layer0Tally,
) -> RecoveryReport {
    let root_persisted = persisted[judged_root_index];
    let newest_persisted = newest_persisted_root(writes, &persisted);
    let image = CrashImage {
        base,
        writes,
        persisted,
    };
    let consulted = recover(&image, JournalPolicy::Consult);
    let ignored = recover(&image, JournalPolicy::Ignore);
    tally.states += 1;
    if root_persisted {
        tally.root_persisted_states += 1;
    }
    if consulted.outcome != ignored.outcome {
        tally.journal_differing_states += 1;
    }
    if consulted.journal.verification_passed + consulted.journal.verification_failed > 0 {
        tally.verification_ran_states += 1;
    }
    if consulted.journal.verification_failed > 0 {
        tally.verification_failed_states += 1;
    }
    match &consulted.outcome {
        RecoveryOutcome::NoFile { .. } => tally.no_file_states += 1,
        RecoveryOutcome::FileRead { .. } => tally.file_read_states += 1,
        RecoveryOutcome::Failed { .. } => tally.failed_states += 1,
    }
    if let Some(reason) = oracle_violation_for_versions(
        &consulted.outcome,
        consulted.effective_root,
        newest_persisted,
        versions,
    ) {
        tally.violations += 1;
        if tally.first_violation.is_none() {
            let persisted_kinds: Vec<&str> = image
                .persisted
                .iter()
                .zip(writes)
                .filter(|(is_persisted, _)| **is_persisted)
                .map(|(_, write)| write.kind.name())
                .collect();
            tally.first_violation = Some(format!(
                "{reason}（持久的写：{}）",
                persisted_kinds.join("|")
            ));
        }
    }
    if let Some(reason) = oracle_violation_for_versions(
        &ignored.outcome,
        ignored.effective_root,
        newest_persisted,
        versions,
    ) {
        tally.ignored_violations += 1;
        if tally.first_ignored_violation.is_none() {
            tally.first_ignored_violation = Some(reason);
        }
    }
    for (invariant, verdict) in check_pool_image(&image) {
        match verdict {
            InvariantVerdict::Holds => {
                *tally.checker_evaluated_states.entry(invariant).or_insert(0) += 1
            }
            InvariantVerdict::Violated(detail) => {
                *tally.checker_evaluated_states.entry(invariant).or_insert(0) += 1;
                *tally.checker_violated_states.entry(invariant).or_insert(0) += 1;
                tally
                    .checker_first_violation
                    .entry(invariant)
                    .or_insert(detail);
            }
            InvariantVerdict::NotApplicable(_) => {
                *tally
                    .checker_not_applicable_states
                    .entry(invariant)
                    .or_insert(0) += 1;
            }
        }
    }
    let records = check_records(&image, consulted.effective_root);
    if records.root_without_record {
        tally.record_root_without_record += 1;
    }
    if records.claimed_state_missing_unit {
        tally.record_claimed_state_missing_unit += 1;
    }
    consulted
}
```
### crates/singlefs-harness/src/scenario.rs:62-68（ScenarioPoint）

```rust
/// 整条路上调用方被叫到的两处。
pub enum ScenarioPoint {
    /// 取号写完、那道屏障做完，暖机还没开始（同一个写入口接着暖机，这里不另发屏障）。
    AfterInstanceAcquisition,
    /// 暖机之后、第一个事务之前（虚机档在这里给某块盘装上「漏一道屏障」）。
    BeforeFirstTransaction,
}
```

### crates/singlefs-harness/src/scenario.rs:70-129（run_first_transaction）

```rust
/// 整条路。`at_point` 在 `ScenarioPoint` 的两处各被叫一次：虚机档在两处给设备一层的计数拍快照，暖机与第一个事务各自的写就是快照之差。
pub fn run_first_transaction<
    Device: BlockDevice,
    AtPoint: FnMut(ScenarioPoint, &mut [(DeviceIdentity, Device)]),
>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    stream: &SharedStream,
    mut at_point: AtPoint,
) -> Result<FirstTransactionRun, String> {
    let genesis =
        make_filesystem(parameters, devices).map_err(|error| format!("mkfs：{error:?}"))?;
    let mkfs_operation_count = stream.operations().len();
    let mut allocator = PoolAllocator::new(
        devices
            .iter()
            .map(|(identity, device)| DeviceFreeMap::new(*identity, device.size_in_bytes()))
            .collect(),
    );
    allocator.mark_format_time_units(
        Placement {
            slot: INSTANCE_TABLE_SLOT,
            span: 2,
        },
        Placement {
            slot: TREE_TABLE_GENESIS_SLOT,
            span: 1,
        },
    );
    let content = first_file_content();
    let (instance, warm_up_output) = {
        let mut pool = PoolWriter::new(parameters, &mut *devices);
        let instance = acquire_instance(&mut pool).map_err(|error| format!("取号：{error:?}"))?;
        at_point(ScenarioPoint::AfterInstanceAcquisition, &mut *pool.devices);
        let warm = warm_up(&mut pool, &genesis.root, instance)
            .map_err(|error| format!("暖机：{error:?}"))?;
        (instance, warm)
    };
    assert_eq!(instance, InstanceGeneration(1));
    at_point(ScenarioPoint::BeforeFirstTransaction, devices);
    let mut pool = PoolWriter::new(parameters, devices);
    let output = publish_first_file(
        &mut pool,
        &mut allocator,
        &genesis.root,
        FirstFile {
            content: &content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS,
        },
        instance,
        &warm_up_output.last_record_bytes,
    )
    .map_err(|error| format!("第一个事务：{error:?}"))?;
    Ok(FirstTransactionRun {
        mkfs_operation_count,
        warm_up: warm_up_output,
        output,
        policy_mismatches: allocator.policy_mismatches,
    })
}
```
### crates/singlefs-harness/src/bin/first_transaction_on_device.rs:125-144（describe_publish_writes）

```rust
/// 一次发布的 `name=publish_writes` 行：合计取 `total()`（从记过的每一笔加），各种按 `IN_REPORT_ORDER` 逐个列、没写过的列 0。
fn describe_publish_writes(publish: &str, txg: u64, writes: &WritesByStructureKind) -> String {
    let total = writes.total();
    let per_kind: String = WrittenStructureKind::IN_REPORT_ORDER
        .iter()
        .map(|kind| {
            let kind_writes = writes.of(*kind);
            format!(
                " {name}_write_calls={} {name}_written_bytes={}",
                kind_writes.write_calls,
                kind_writes.written_bytes,
                name = kind.report_name()
            )
        })
        .collect();
    format!(
        "name=publish_writes publish={publish} txg={txg} write_calls={} written_bytes={}{per_kind}",
        total.write_calls, total.written_bytes
    )
}
```
### crates/singlefs-harness/src/bin/first_transaction_on_device.rs:146-169（publish_writes_against_device）

```rust
/// 一段窗口里几次发布按种类记的合计与设备一层数的比：返回结果行与相不相等。
fn publish_writes_against_device(
    window: &str,
    publishes: &[&WritesByStructureKind],
    device: WriteCallsAndBytes,
) -> (String, bool) {
    let by_kind = publishes
        .iter()
        .fold(WriteCallsAndBytes::NONE, |sum, writes| {
            sum.plus(writes.total())
        });
    let matches = by_kind == device;
    (
        format!(
            "name=publish_writes_against_device window={window} publishes={} by_kind_write_calls={} by_kind_written_bytes={} device_write_calls={} device_written_bytes={} matches={matches}",
            publishes.len(),
            by_kind.write_calls,
            by_kind.written_bytes,
            device.write_calls,
            device.written_bytes
        ),
        matches,
    )
}
```

### crates/singlefs-harness/src/bin/first_transaction_on_device.rs:345-508（switch_instance_and_publish_third_version）

```rust
/// 发布 B 之后：丢掉写的那一套句柄，同一对盘冷重开（`reopen` 拿到关掉的那几块盘、交回重新打开的），走可写挂载（恢复、取号、写行、暖机），
/// 再发布 C。挂载那一段窗口从每块盘收到第一道屏障（取号那一道）算起、到挂载返回为止，按种类的合计是写行与每次暖机之和：取号的超级块槽写不是发布、
/// 不进按种类的账，与第一个事务那条路上暖机窗口从取号之后算起同一个口径。
///
/// # Errors
/// 可写挂载失败、挂载之后现行那一版没有文件、取号那道屏障没数到、发布 C 失败：交回一句原因。
fn switch_instance_and_publish_third_version<Inner, Reopen>(
    parameters: &MakeFilesystemParameters,
    devices_after_second_transaction: Vec<(DeviceIdentity, CountedDevice<Inner>)>,
    stream: &SharedStream,
    geometry: &FixedGeometry,
    reopen: Reopen,
) -> Result<SecondInstanceRun<Inner>, String>
where
    Inner: BlockDevice,
    Reopen: FnOnce(Vec<(DeviceIdentity, Inner)>) -> Vec<(DeviceIdentity, Inner)>,
{
    let closed_devices: Vec<(DeviceIdentity, Inner)> = devices_after_second_transaction
        .into_iter()
        .map(|(identity, device)| (identity, device.into_inner_and_operations().0.inner))
        .collect();
    let mut devices: Vec<(DeviceIdentity, CountedDevice<Inner>)> = reopen(closed_devices)
        .into_iter()
        .map(|(identity, inner)| {
            (
                identity,
                RecordingBlockDevice::with_shared_stream(
                    identity,
                    FaultInjectingDevice::counting(inner),
                    stream.clone(),
                ),
            )
        })
        .collect();
    let mut lines = Vec::new();

    let operations_before_mount = stream.operations().len();
    let mount_started = Instant::now();
    let mut mounted = mount_writable(parameters, &mut devices)
        .map_err(|failure| format!("可写挂载：{failure:?}"))?;
    let mount_nanoseconds = mount_started.elapsed().as_nanos();
    let counts_after_mount = device_call_counts(&devices);
    let counts_after_acquisition: Vec<DeviceCallCounts> = devices
        .iter()
        .map(|(identity, device)| {
            device
                .inner()
                .counts_when_first_barrier_arrived
                .ok_or_else(|| {
                    format!(
                        "盘 {} 在可写挂载里一道屏障都没收到：取号那道屏障没发",
                        identity.0
                    )
                })
        })
        .collect::<Result<_, _>>()?;
    let mount_operations = stream.operations();
    let mount_segments =
        split_into_segments(&mount_operations[operations_before_mount..], geometry);
    let output = &mounted.output;
    let warm_up_txgs: Vec<String> = output
        .warm_up_publishes
        .iter()
        .map(|publish| publish.root().checkpoint_txg.0.to_string())
        .collect();
    let per_device_mount: Vec<String> = counts_after_mount
        .iter()
        .enumerate()
        .map(|(index, counts)| counts.describe(index))
        .collect();
    lines.push(format!(
        "name=writable_mount instance={} chosen_root={}:{} rows_written={} row_publish_root={} warm_up_txgs={} nanoseconds={mount_nanoseconds} operations={} segments={} closed_form={} {}",
        output.instance.0,
        output.chosen_root.instance.0,
        output.chosen_root.checkpoint_txg.0,
        output.rows_written.len(),
        root_text(&output.row_publish),
        warm_up_txgs.join(","),
        mount_operations.len() - operations_before_mount,
        segment_sizes_text(&mount_segments),
        closed_form_state_count(&mount_segments),
        per_device_mount.join(" ")
    ));
    lines.push(describe_publish_writes(
        "instance_row",
        output.row_publish.root().checkpoint_txg.0,
        writes_of(&output.row_publish),
    ));
    let mut mount_publishes: Vec<&WritesByStructureKind> = vec![writes_of(&output.row_publish)];
    for warm_up_publish in &output.warm_up_publishes {
        lines.push(describe_publish_writes(
            "warm_up",
            warm_up_publish.root().checkpoint_txg.0,
            writes_of(warm_up_publish),
        ));
        mount_publishes.push(writes_of(warm_up_publish));
    }
    let (mount_window_line, mount_window_matches) = publish_writes_against_device(
        "writable_mount",
        &mount_publishes,
        pool_writes_between(&counts_after_acquisition, &counts_after_mount),
    );
    lines.push(mount_window_line);

    let current = mounted
        .current
        .file_version()
        .ok_or_else(|| "可写挂载之后现行那一版没有文件：发布 B 之后重开，上一版带文件".to_string())?
        .clone();
    let instance = mounted.output.instance;
    let operations_before_third = stream.operations().len();
    let third_started = Instant::now();
    let third = {
        let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut pool,
            &mut mounted.allocator,
            &current,
            FirstFile {
                content: &third_file_content(),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 120,
            },
            instance,
        )
    }
    .map_err(|failure| format!("发布 C：{failure:?}"))?;
    let third_nanoseconds = third_started.elapsed().as_nanos();
    let counts_after_third = device_call_counts(&devices);
    let third_operations = stream.operations();
    let third_segments =
        split_into_segments(&third_operations[operations_before_third..], geometry);
    let per_device_third: Vec<String> = counts_after_third
        .iter()
        .enumerate()
        .map(|(index, later)| later.since(counts_after_mount[index]).describe(index))
        .collect();
    lines.push(format!(
        "name=third_transaction root_txg={} transaction={} released={} nanoseconds={third_nanoseconds} operations={} segments={} closed_form={} {}",
        third.root.checkpoint_txg.0,
        third.record.transaction,
        third.released.len(),
        third_operations.len() - operations_before_third,
        segment_sizes_text(&third_segments),
        closed_form_state_count(&third_segments),
        per_device_third.join(" ")
    ));
    lines.push(describe_publish_writes(
        "third_transaction",
        third.root.checkpoint_txg.0,
        &third.writes,
    ));
    let (third_window_line, third_window_matches) = publish_writes_against_device(
        "third_transaction",
        &[&third.writes],
        pool_writes_between(&counts_after_mount, &counts_after_third),
    );
    lines.push(third_window_line);

    Ok(SecondInstanceRun {
        devices,
        lines,
        every_window_matches_device: mount_window_matches && third_window_matches,
    })
}
```

### crates/singlefs-harness/src/bin/first_transaction_on_device.rs:514-861（main）

```rust
fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    if arguments.len() != 4 {
        eprintln!("用法：first_transaction_on_device <盘 0> <盘 1> <direct | page-cache | skip-first-transaction-barrier | second-transaction | second-instance>");
        std::process::exit(2);
    }
    let device_paths = [arguments[1].clone(), arguments[2].clone()];
    let mode = match arguments[3].as_str() {
        "direct" => RunMode::Direct,
        "page-cache" => RunMode::PageCache,
        "skip-first-transaction-barrier" => RunMode::SkipFirstTransactionBarrier,
        "second-transaction" => RunMode::SecondTransaction,
        "second-instance" => RunMode::SecondInstance,
        other => {
            eprintln!("不认识的模式 {other}");
            std::process::exit(2)
        }
    };
    let policy = match mode {
        RunMode::Direct
        | RunMode::SkipFirstTransactionBarrier
        | RunMode::SecondTransaction
        | RunMode::SecondInstance => PageCachePolicy::BypassWithDirectInputOutput,
        RunMode::PageCache => PageCachePolicy::GoThroughPageCache,
    };
    let publishes_second_version = match mode {
        RunMode::Direct | RunMode::PageCache | RunMode::SkipFirstTransactionBarrier => false,
        RunMode::SecondTransaction | RunMode::SecondInstance => true,
    };
    let mut emitter = Emitter { emitted: 0 };
    let probe = |attribute: &str| -> u32 {
        let values: Vec<Option<u32>> = device_paths
            .iter()
            .map(|path| {
                probe_queue_number_under(Path::new(SYSFS_CLASS_BLOCK), Path::new(path), attribute)
            })
            .collect();
        match (values[0], values[1]) {
            (Some(first), Some(second)) if first == second => first,
            (first, second) => {
                eprintln!("两块盘的 queue/{attribute} 读不到或不相等：{first:?} / {second:?}");
                std::process::exit(4)
            }
        }
    };
    let physical_block_size = probe("physical_block_size");
    let minimum_input_output_bytes = probe("minimum_io_size");
    let write_cache = probe_queue_text_under(
        Path::new(SYSFS_CLASS_BLOCK),
        Path::new(&device_paths[0]),
        "write_cache",
    )
    .unwrap_or_else(|| "NA".to_string());
    let parameters = e142_parameters(physical_block_size, minimum_input_output_bytes);
    let stream = SharedStream::new();
    let mut devices: Vec<(
        DeviceIdentity,
        RecordingBlockDevice<FaultInjectingDevice<DirectInputOutputBlockDevice>>,
    )> = device_paths
        .iter()
        .enumerate()
        .map(|(index, path)| {
            let identity = DeviceIdentity(u32::try_from(index).expect("设备号"));
            let device = FaultInjectingDevice::counting(open(path, policy));
            (
                identity,
                RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
            )
        })
        .collect();
    let device_bytes = devices[0].1.size_in_bytes();
    emitter.emit(&format!(
        "name=geometry mode={} physical_block_size={physical_block_size} minimum_io={minimum_input_output_bytes} spacing={} write_cache={} device_bytes={device_bytes}",
        arguments[3],
        parameters.geometry.fixed_structure_slot_spacing,
        write_cache.replace(' ', "_")
    ));
    let counters_before: Vec<_> = device_paths
        .iter()
        .map(|path| block_layer_counters(path))
        .collect();
    let mut counts_after_acquisition: Option<Vec<DeviceCallCounts>> = None;
    let mut counts_before_first_transaction: Option<Vec<DeviceCallCounts>> = None;
    let run = run_first_transaction(&parameters, &mut devices, &stream, |point, devices| {
        let counts: Vec<DeviceCallCounts> = devices
            .iter()
            .map(|(_, device)| DeviceCallCounts::of(device.inner()))
            .collect();
        match point {
            ScenarioPoint::AfterInstanceAcquisition => counts_after_acquisition = Some(counts),
            ScenarioPoint::BeforeFirstTransaction => {
                counts_before_first_transaction = Some(counts);
                if mode == RunMode::SkipFirstTransactionBarrier {
                    devices[0].1.inner_mut().skip_next_barrier = true;
                }
            }
        }
    })
    .unwrap_or_else(|reason| {
        eprintln!("写路径失败：{reason}");
        std::process::exit(5)
    });
    let counters_after: Vec<_> = device_paths
        .iter()
        .map(|path| block_layer_counters(path))
        .collect();
    let counts_after_first_transaction: Vec<DeviceCallCounts> = devices
        .iter()
        .map(|(_, device)| DeviceCallCounts::of(device.inner()))
        .collect();
    let counts_after_acquisition =
        counts_after_acquisition.expect("run_first_transaction 取号之后叫过一次");
    let counts_before_first_transaction =
        counts_before_first_transaction.expect("run_first_transaction 第一个事务之前叫过一次");

    let operations = stream.operations();
    let geometry = FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
    };
    let warm_up_end = operations.len() - 23;
    let paths: [(&str, &[RecordedOperation]); 5] = [
        ("mkfs", &operations[..run.mkfs_operation_count]),
        (
            "instance_acquisition",
            &operations[run.mkfs_operation_count..run.mkfs_operation_count + 2],
        ),
        (
            "warm_up",
            &operations[run.mkfs_operation_count + 2..warm_up_end],
        ),
        ("transaction", &operations[warm_up_end..]),
        ("post_mkfs_stream", &operations[run.mkfs_operation_count..]),
    ];
    for (name, slice) in paths {
        let segments = split_into_segments(slice, &geometry);
        emitter.emit(&format!(
            "name=segments path={name} operations={} segments={} closed_form={} kinds={}",
            slice.len(),
            segment_sizes_text(&segments),
            closed_form_state_count(&segments),
            segment_kinds_text(&segments)
        ));
    }
    for (index, (identity, device)) in devices.iter().enumerate() {
        let counted = device.inner();
        emitter.emit(&format!(
            "name=device_calls device={} path={} writes={} written_bytes={} force_unit_access_writes={} barriers={} skipped_barriers={} {}",
            identity.0,
            device_paths[index],
            counted.write_calls,
            counted.written_bytes,
            counted.force_unit_access_writes,
            counted.barrier_calls,
            counted.skipped_barriers,
            describe_counters(counters_before[index], counters_after[index])
        ));
    }
    for (warm_up_writes, warm_up_root) in run.warm_up.writes.iter().zip(&run.warm_up.roots) {
        emitter.emit(&describe_publish_writes(
            "warm_up",
            warm_up_root.checkpoint_txg.0,
            warm_up_writes,
        ));
    }
    emitter.emit(&describe_publish_writes(
        "first_transaction",
        run.output.root.checkpoint_txg.0,
        &run.output.writes,
    ));
    let warm_up_publishes: Vec<&WritesByStructureKind> = run.warm_up.writes.iter().collect();
    let (warm_up_line, warm_up_matches) = publish_writes_against_device(
        "warm_up",
        &warm_up_publishes,
        pool_writes_between(&counts_after_acquisition, &counts_before_first_transaction),
    );
    emitter.emit(&warm_up_line);
    let (first_transaction_line, first_transaction_matches) = publish_writes_against_device(
        "first_transaction",
        &[&run.output.writes],
        pool_writes_between(
            &counts_before_first_transaction,
            &counts_after_first_transaction,
        ),
    );
    emitter.emit(&first_transaction_line);
    let mut every_window_matches_device = warm_up_matches && first_transaction_matches;
    emitter.emit(&format!(
        "name=transaction policy_mismatches={} key_order_mismatches={} root_txg={} back_chain={}",
        run.policy_mismatches,
        run.output.key_order_mismatches,
        run.output.root.checkpoint_txg.0,
        run.output.record.back_chain
    ));

    // 第二个事务（发布 B）：同一个进程里再覆盖写一次，分配器从第一个事务的分配记录重建（与可写挂载同一条路）；
    // 两次快照之差就是这一次发布交给设备的调用数，挂钟单独计。
    let expected_content = if publishes_second_version {
        let before: Vec<DeviceCallCounts> = devices
            .iter()
            .map(|(_, device)| DeviceCallCounts::of(device.inner()))
            .collect();
        let operations_before = stream.operations().len();
        let device_maps: Vec<DeviceFreeMap> = devices
            .iter()
            .map(|(identity, device)| DeviceFreeMap::new(*identity, device.size_in_bytes()))
            .collect();
        let mut allocator =
            PoolAllocator::rebuild_from_records(device_maps, run.output.allocation_records.clone());
        let started = Instant::now();
        let second = {
            let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
            publish_overwrite(
                &mut pool,
                &mut allocator,
                &run.output,
                FirstFile {
                    content: &second_file_content(),
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
                },
                run.output.root.instance,
            )
        }
        .unwrap_or_else(|failure| {
            eprintln!("第二个事务失败：{failure:?}");
            std::process::exit(5)
        });
        let nanoseconds = started.elapsed().as_nanos();
        let after_operations = stream.operations();
        let second_segments =
            split_into_segments(&after_operations[operations_before..], &geometry);
        let after: Vec<DeviceCallCounts> = devices
            .iter()
            .map(|(_, device)| DeviceCallCounts::of(device.inner()))
            .collect();
        let per_device: Vec<String> = after
            .iter()
            .enumerate()
            .map(|(index, later)| later.since(before[index]).describe(index))
            .collect();
        emitter.emit(&format!(
            "name=second_transaction root_txg={} transaction={} released={} nanoseconds={nanoseconds} operations={} segments={} closed_form={} {}",
            second.root.checkpoint_txg.0,
            second.record.transaction,
            second.released.len(),
            after_operations.len() - operations_before,
            segment_sizes_text(&second_segments),
            closed_form_state_count(&second_segments),
            per_device.join(" ")
        ));
        emitter.emit(&describe_publish_writes(
            "second_transaction",
            second.root.checkpoint_txg.0,
            &second.writes,
        ));
        let (second_transaction_line, second_transaction_matches) = publish_writes_against_device(
            "second_transaction",
            &[&second.writes],
            pool_writes_between(&before, &after),
        );
        emitter.emit(&second_transaction_line);
        every_window_matches_device &= second_transaction_matches;
        second_file_content()
    } else {
        first_file_content()
    };

    // `second-instance`：发布 B 之后同一对盘冷重开、可写挂载，再发布 C。
    let (devices, expected_content) = match mode {
        RunMode::SecondInstance => {
            let switched = switch_instance_and_publish_third_version(
                &parameters,
                devices,
                &stream,
                &geometry,
                |closed_devices| {
                    drop(closed_devices);
                    device_paths
                        .iter()
                        .enumerate()
                        .map(|(index, path)| {
                            (
                                DeviceIdentity(u32::try_from(index).expect("设备号")),
                                open(path, policy),
                            )
                        })
                        .collect()
                },
            )
            .unwrap_or_else(|reason| {
                eprintln!("第二个实例失败：{reason}");
                std::process::exit(5)
            });
            for line in &switched.lines {
                emitter.emit(line);
            }
            every_window_matches_device &= switched.every_window_matches_device;
            (switched.devices, third_file_content())
        }
        RunMode::Direct
        | RunMode::PageCache
        | RunMode::SkipFirstTransactionBarrier
        | RunMode::SecondTransaction => (devices, expected_content),
    };

    // 冷重开：丢掉写的那一套句柄，按路径重新打开，恢复只通过块设备读盘。
    drop(devices);
    let reopened: Vec<(DeviceIdentity, DirectInputOutputBlockDevice)> = device_paths
        .iter()
        .enumerate()
        .map(|(index, path)| {
            (
                DeviceIdentity(u32::try_from(index).expect("设备号")),
                open(path, policy),
            )
        })
        .collect();
    let report = recover(&reopened, JournalPolicy::Consult);
    let (outcome, root, content_matches) = match &report.outcome {
        RecoveryOutcome::FileRead { root, content } => (
            "file_read",
            format!("{}:{}", root.0 .0, root.1 .0),
            *content == expected_content,
        ),
        RecoveryOutcome::NoFile { root } => {
            ("no_file", format!("{}:{}", root.0 .0, root.1 .0), false)
        }
        RecoveryOutcome::Failed { root, failure } => {
            eprintln!("恢复失败：{failure:?}");
            (
                "failed",
                root.map_or_else(
                    || "none".to_string(),
                    |root| format!("{}:{}", root.0 .0, root.1 .0),
                ),
                false,
            )
        }
    };
    emitter.emit(&format!(
        "name=recover_cold outcome={outcome} root={root} content_matches={content_matches} valid_records={} above_water={} applied={} verification_passed={} mapping_fallbacks={}",
        report.journal.valid_records, report.journal.above_water, report.journal.prefix_applied, report.journal.verification_passed, report.mapping_fallbacks
    ));
    emitter.finish();
    if !(outcome == "file_read" && content_matches && every_window_matches_device) {
        std::process::exit(1);
    }
}
```

### crates/singlefs-harness/src/bin/first_transaction_on_device.rs:894-1052（second_instance_mode_mounts_writes_the_row_warms_up_publishes_the_third_version_and_every_window_matches）

```rust
    /// `second-instance` 模式在宿主上照同一条路跑一遍：稀疏内存盘代替 virtio 盘，「冷重开」把镜像交给新句柄。
    /// 发布 B 之后可写挂载取实例 2、写行 txg 5、暖机 txg 6 / 7，发布 C 是 txg 8；两段窗口按种类的合计都与设备一层相等；
    /// 冷恢复择实例 2 的根读回第三版。
    #[test]
    fn second_instance_mode_mounts_writes_the_row_warms_up_publishes_the_third_version_and_every_window_matches(
    ) {
        let parameters = e142_parameters(512, 512);
        let stream = SharedStream::new();
        let mut devices: Vec<(DeviceIdentity, CountedDevice<SparseBlockDevice>)> = (0..2u32)
            .map(|device_number| {
                let identity = DeviceIdentity(device_number);
                (
                    identity,
                    RecordingBlockDevice::with_shared_stream(
                        identity,
                        FaultInjectingDevice::counting(SparseBlockDevice::new(
                            SPARSE_DEVICE_BYTES,
                            PhysicalBlockSizeInBytes(512),
                        )),
                        stream.clone(),
                    ),
                )
            })
            .collect();
        let run = run_first_transaction(&parameters, &mut devices, &stream, |_point, _devices| {})
            .expect("第一个事务");
        let mut allocator = PoolAllocator::rebuild_from_records(
            devices
                .iter()
                .map(|(identity, device)| DeviceFreeMap::new(*identity, device.size_in_bytes()))
                .collect(),
            run.output.allocation_records.clone(),
        );
        let second = {
            let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
            publish_overwrite(
                &mut pool,
                &mut allocator,
                &run.output,
                FirstFile {
                    content: &second_file_content(),
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
                },
                run.output.root.instance,
            )
            .expect("发布 B")
        };
        assert_eq!(second.root.checkpoint_txg, CheckpointTxg(4));
        let geometry = FixedGeometry {
            fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
            journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        };

        let switched = switch_instance_and_publish_third_version(
            &parameters,
            devices,
            &stream,
            &geometry,
            |closed_devices| {
                closed_devices
                    .into_iter()
                    .map(|(identity, closed)| {
                        let mut reopened = SparseBlockDevice::new(
                            SPARSE_DEVICE_BYTES,
                            PhysicalBlockSizeInBytes(512),
                        );
                        reopened.image = closed.image;
                        (identity, reopened)
                    })
                    .collect()
            },
        )
        .expect("第二个实例");

        let line_named = |name: &str| -> Vec<&str> {
            switched
                .lines
                .iter()
                .map(String::as_str)
                .filter(|line| field(line, "name") == Some(name))
                .collect()
        };
        let mount_lines = line_named("writable_mount");
        assert_eq!(mount_lines.len(), 1, "{:#?}", switched.lines);
        let mount = mount_lines[0];
        assert_eq!(field(mount, "instance"), Some("2"), "{mount}");
        assert_eq!(field(mount, "chosen_root"), Some("1:4"), "B 的根：{mount}");
        assert_eq!(
            field(mount, "rows_written"),
            Some("1"),
            "实例 1 那一行：{mount}"
        );
        assert_eq!(field(mount, "row_publish_root"), Some("2:5"), "{mount}");
        assert_eq!(field(mount, "warm_up_txgs"), Some("6,7"), "{mount}");

        let publish_lines: Vec<(Option<&str>, Option<&str>)> = line_named("publish_writes")
            .into_iter()
            .map(|line| (field(line, "publish"), field(line, "txg")))
            .collect();
        assert_eq!(
            publish_lines,
            vec![
                (Some("instance_row"), Some("5")),
                (Some("warm_up"), Some("6")),
                (Some("warm_up"), Some("7")),
                (Some("third_transaction"), Some("8")),
            ]
        );
        let windows = line_named("publish_writes_against_device");
        let window_summary: Vec<(Option<&str>, Option<&str>, Option<&str>)> = windows
            .iter()
            .map(|line| {
                (
                    field(line, "window"),
                    field(line, "publishes"),
                    field(line, "matches"),
                )
            })
            .collect();
        assert_eq!(
            window_summary,
            vec![
                (Some("writable_mount"), Some("3"), Some("true")),
                (Some("third_transaction"), Some("1"), Some("true")),
            ],
            "两段窗口按种类的合计都等于设备一层数的：{windows:#?}"
        );
        assert!(switched.every_window_matches_device);
        let third_lines = line_named("third_transaction");
        assert_eq!(third_lines.len(), 1, "{:#?}", switched.lines);
        assert_eq!(field(third_lines[0], "root_txg"), Some("8"));
        assert_eq!(
            field(third_lines[0], "transaction"),
            Some("1"),
            "事务号按实例从 1 起"
        );
        assert_eq!(
            field(third_lines[0], "segments"),
            Some("16+2+1+2"),
            "发布 C 与发布 B 同型"
        );

        let cold: Vec<(DeviceIdentity, SparseBlockDevice)> = switched
            .devices
            .into_iter()
            .map(|(identity, device)| (identity, device.into_inner_and_operations().0.inner))
            .collect();
        let report = recover(&cold, JournalPolicy::Consult);
        match report.outcome {
            RecoveryOutcome::FileRead { root, content } => {
                assert_eq!(root, (InstanceGeneration(2), CheckpointTxg(8)));
                assert!(content == third_file_content(), "冷恢复读回第三版");
            }
            RecoveryOutcome::NoFile { root } => panic!("冷恢复择到 {root:?} 却没有文件"),
            RecoveryOutcome::Failed { root, failure } => {
                panic!("冷恢复失败：{root:?} {failure:?}")
            }
        }
    }
```

### crates/singlefs-harness/src/bin/first_transaction_device_log_check.rs:95-181（main）

```rust
fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    if arguments.len() != 6 && arguments.len() != 8 {
        eprintln!("用法：first_transaction_device_log_check <log0.img> <log1.img> <设备字节数> <physical_block_size> <minimum_io> [<disk0.img> <disk1.img>]");
        std::process::exit(2);
    }
    let device_bytes = parse_number(&arguments[3], "设备字节数");
    let physical_block_size = u32::try_from(parse_number(&arguments[4], "physical_block_size"))
        .expect("物理块大小装得进 u32");
    let minimum_input_output_bytes =
        u32::try_from(parse_number(&arguments[5], "minimum_io")).expect("io_min 装得进 u32");
    let parameters = e142_parameters(physical_block_size, minimum_input_output_bytes);
    let stream = SharedStream::new();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = (0..2u32)
        .map(|index| {
            let identity = DeviceIdentity(index);
            let device =
                SparseBlockDevice::new(device_bytes, PhysicalBlockSizeInBytes(physical_block_size));
            (
                identity,
                RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
            )
        })
        .collect();
    run_first_transaction(&parameters, &mut devices, &stream, |_point, _devices| {})
        .unwrap_or_else(|reason| {
            eprintln!("宿主重跑写路失败：{reason}");
            std::process::exit(2)
        });
    let operations = stream.retained_operations();
    let mut every_device_matches = true;
    let mut emitted = 0u64;
    for index in 0..2usize {
        let log_bytes = std::fs::read(&arguments[1 + index]).unwrap_or_else(|error| {
            eprintln!("读不了 {}：{error}", arguments[1 + index]);
            std::process::exit(2)
        });
        let log = parse_device_log(&log_bytes).unwrap_or_else(|error| {
            eprintln!("{} 解析不了：{error:?}", arguments[1 + index]);
            std::process::exit(2)
        });
        let identity = DeviceIdentity(u32::try_from(index).expect("设备号"));
        let expected = expected_device_events(&operations, identity);
        let (expected_writes, expected_flushes) = count_kinds(&expected);
        let (observed_writes, observed_flushes) = count_kinds(&log.events);
        let comparison = compare_allowing_trailing_flushes(&expected, &log.events);
        let divergence = if comparison.divergence.is_none()
            && comparison.trailing_flushes > ACCEPTED_TRAILING_FLUSHES
        {
            Some((
                expected.len() + ACCEPTED_TRAILING_FLUSHES,
                None,
                Some(DeviceEvent::Flush),
            ))
        } else {
            comparison.divergence.clone()
        };
        let divergence_text = match &divergence {
            None => "none".to_string(),
            Some((position, program, device)) => {
                format!("at={position} program={program:?} device={device:?}").replace(' ', "")
            }
        };
        every_device_matches &= divergence.is_none();
        println!(
            "E7RESULT name=device_log device={} declared_entries={} expected_writes={expected_writes} expected_flushes={expected_flushes} observed_writes={observed_writes} observed_flushes={observed_flushes} trailing_flushes={} divergence={divergence_text}",
            identity.0,
            log.declared_entries.map_or_else(|| "NA".to_string(), |declared| declared.to_string()),
            comparison.trailing_flushes,
        );
        emitted += 1;
    }
    println!("E7RESULT name=device_log_verdict matches={every_device_matches}");
    emitted += 1;
    let mut recovered = true;
    if arguments.len() == 8 {
        let (line, read_back) =
            recover_from_disk_images([&arguments[6], &arguments[7]], physical_block_size);
        println!("E7RESULT {line}");
        emitted += 1;
        recovered = read_back;
    }
    println!("E7RESULT name=done emitted={}", emitted + 1);
    if !(every_device_matches && recovered) {
        std::process::exit(1);
    }
}
```

### crates/singlefs-harness/tests/second_transaction_supplement_one_write_accounting.rs:263-326（first_transaction_and_overwrite_writes_by_kind_add_up_to_the_recorded_writes_and_the_byte_table）

```rust
/// 验收第 1 条：暖机两次、第一个事务、发布 B 各自按种类的合计与录制器逐项相等；发布 B 与第一个事务都是 21 次写调用、344 576 字节，
/// 每次暖机 5 次、16 896 字节；每一种钉成字节表的数。
#[test]
fn first_transaction_and_overwrite_writes_by_kind_add_up_to_the_recorded_writes_and_the_byte_table()
{
    let published = publish_through_overwrite();
    let operations = published.stream.operations();
    assert_eq!(published.warm_up.writes.len(), 2, "暖机两次空发布各一份账");

    assert_eq!(
        published.warm_up.writes[0]
            .total()
            .plus(published.warm_up.writes[1].total()),
        recorded_writes(&operations[published.warm_up_start..published.first_transaction_start]),
        "暖机两次按种类的合计 == 录制器在这段里记下的写"
    );
    assert_eq!(
        published.first_transaction.writes.total(),
        recorded_writes(&operations[published.first_transaction_start..published.overwrite_start]),
        "第一个事务按种类的合计 == 录制器在这段里记下的写"
    );
    assert_eq!(
        published.overwrite.writes.total(),
        recorded_writes(&operations[published.overwrite_start..published.overwrite_end]),
        "发布 B 按种类的合计 == 录制器在这段里记下的写"
    );

    for (warm_up_index, warm_up_writes) in published.warm_up.writes.iter().enumerate() {
        assert_eq!(
            warm_up_writes.total(),
            calls_and_bytes(5, 16_896),
            "第 {} 次暖机：记录 2 + 根槽 1 + 超级块槽 2",
            warm_up_index + 1
        );
    }
    assert_eq!(
        published.first_transaction.writes.total(),
        calls_and_bytes(21, 344_576),
        "第一个事务：单元 8 × 2 盘 = 16 次 327 680 字节 + 记录 2 次 8192 + 根槽 1 次 512 + 超级块槽 2 次 8192"
    );
    assert_eq!(
        published.overwrite.writes.total(),
        calls_and_bytes(21, 344_576),
        "发布 B 与第一个事务写同样八个角色：E152 量到的 344 576 字节、21 次写调用"
    );

    for (warm_up_index, warm_up_writes) in published.warm_up.writes.iter().enumerate() {
        every_kind_matches(
            &format!("第 {} 次暖机", warm_up_index + 1),
            warm_up_writes,
            &zero_unit_publish_by_kind(),
        );
    }
    every_kind_matches(
        "第一个事务",
        &published.first_transaction.writes,
        &file_version_publish_by_kind(),
    );
    every_kind_matches(
        "发布 B",
        &published.overwrite.writes,
        &file_version_publish_by_kind(),
    );
}
```
### crates/singlefs-harness/tests/second_transaction_supplement_one_write_accounting.rs:328-390（writable_remount_row_publish_and_each_warm_up_publish_add_up_to_the_recorded_writes）

```rust
/// 验收第 1 条里「写行与暖机的空发布各自一份」：发布 B 之后可写挂载——取号 2 次超级块槽写不属于任何一次发布；写行那次发布 15 次、
/// 213 504 字节；之后两次暖机空发布各 13 次、147 968 字节；整段录制器记下的写 == 取号 + 三次发布的合计。
#[test]
fn writable_remount_row_publish_and_each_warm_up_publish_add_up_to_the_recorded_writes() {
    let mut published = publish_through_overwrite();
    let parameters = e142_parameters(512, 512);
    let mount_start = published.stream.operations().len();
    let mounted = mount_writable(&parameters, &mut published.devices).expect("可写挂载");
    let operations = published.stream.operations();
    let row_publish = mounted
        .output
        .row_publish
        .file_version()
        .expect("B 带文件：写行发布是带文件的一版");
    let warm_up_publishes: Vec<&TransactionOutput> = mounted
        .output
        .warm_up_publishes
        .iter()
        .map(|version| {
            version
                .file_version()
                .expect("带文件的一版之后的暖机仍是带文件的一版")
        })
        .collect();
    let acquisition = calls_and_bytes(2, 8192);

    assert_eq!(
        warm_up_publishes.iter().fold(
            acquisition.plus(row_publish.writes.total()),
            |sum, warm_up_publish| sum.plus(warm_up_publish.writes.total())
        ),
        recorded_writes(&operations[mount_start..]),
        "取号两盘各一次超级块槽 + 写行发布 + 各次暖机按种类的合计 == 录制器在挂载这段里记下的写"
    );

    assert_eq!(
        warm_up_publishes.len(),
        2,
        "txg 6 落盘 0、txg 7 落盘 1：暖机两次"
    );
    assert_eq!(
        row_publish.writes.total(),
        calls_and_bytes(15, 213_504),
        "写行：实例表单元 + 四个固定点单元 × 2 盘 = 10 次 196 608 字节 + 记录、根槽、超级块槽 5 次 16 896"
    );
    for (warm_up_index, warm_up_publish) in warm_up_publishes.iter().enumerate() {
        assert_eq!(
            warm_up_publish.writes.total(),
            calls_and_bytes(13, 147_968),
            "第 {} 次暖机：四个固定点单元 × 2 盘 = 8 次 131 072 字节 + 5 次 16 896",
            warm_up_index + 1
        );
    }

    every_kind_matches("写行发布", &row_publish.writes, &row_publish_by_kind());
    for (warm_up_index, warm_up_publish) in warm_up_publishes.iter().enumerate() {
        every_kind_matches(
            &format!("挂载之后第 {} 次暖机", warm_up_index + 1),
            &warm_up_publish.writes,
            &later_warm_up_publish_by_kind(),
        );
    }
}
```

### crates/singlefs-harness/tests/second_transaction_supplement_two_commit_generated_fallback.rs:78-152（a_publish_on_a_pool_without_an_empty_cluster_segment_falls_back_to_the_lowest_free_slot_on_every_device）

```rust
/// C369 的验收：第一个事务 A 之后开放段 [50240, 50304) 用满、别的段都不全空，覆盖写 B 要成功。
/// 回落政策函数在这张空闲图上给的落点（手算，不调分配器）：B 先释放 A 的八个落点（进 defer、槽仍占着），数据单元不受段约束、
/// 取 A 的 50180 之后最低的偶数空槽对 50182；extent 树根是第一个提交内生块，开放段装不下、没有全空段 ⇒ 回落到槽号最小的空槽 50179
/// （mkfs 的 50176–50178 与 A 的数据单元之间那个从没分配过的洞）；inode 树叶容器两槽、起点 32768 对齐，50182–50183 刚给了数据单元 ⇒ 50184；
/// 之后的一槽节点按 bump 次序接着取最低空槽 50186–50190。两块盘上的分配记录同槽、分配代 4；冷启动读回 B。
#[test]
fn a_publish_on_a_pool_without_an_empty_cluster_segment_falls_back_to_the_lowest_free_slot_on_every_device(
) {
    let mut pool = build_pool("supplement-two-fallback-publish");
    leave_no_empty_cluster_segment(&mut pool.allocator);
    for device_map in &pool.allocator.devices {
        assert_eq!(
            device_map.lowest_empty_segment(),
            None,
            "盘 {:?} 上还有全空段：造的形态不对",
            device_map.device
        );
        assert!(
            device_map.free_slots() > 200_000,
            "盘 {:?} 上仍有大片空槽：{}",
            device_map.device,
            device_map.free_slots()
        );
    }
    let second_content = content_of(4100, 3);
    let second = try_overwrite_in_process(&mut pool, &second_content, InstanceGeneration(1))
        .expect("没有全空段、每块盘仍有空槽：发布必须成功（C369）");
    let expected_slots = [
        (TransactionUnit::Data, 50182),
        (TransactionUnit::ExtentRoot, 50179),
        (TransactionUnit::InodeLeaf, 50184),
        (TransactionUnit::InodeRoot, 50186),
        (TransactionUnit::AllocationTree, 50187),
        (TransactionUnit::AccountingTree, 50188),
        (TransactionUnit::MappingTree, 50189),
        (TransactionUnit::TreeTable, 50190),
    ];
    for (identity, slot) in expected_slots {
        assert_eq!(
            second.unit(identity).slot,
            SlotNumber(slot),
            "{}：落点要等于回落政策函数",
            identity.tag()
        );
        for device in [DeviceIdentity(0), DeviceIdentity(1)] {
            let record = pool
                .allocator
                .record_for(device, SlotNumber(slot))
                .unwrap_or_else(|| panic!("{} 在盘 {device:?} 上没有分配记录", identity.tag()));
            assert!(!record.is_released, "{} 的记录仍分配", identity.tag());
            assert_eq!(
                record.generation,
                CheckpointTxg(4),
                "{} 的分配代",
                identity.tag()
            );
        }
    }
    assert_eq!(
        pool.allocator.open_segment(),
        None,
        "回落不开段：没有全空段可开"
    );
    assert_eq!(pool.allocator.policy_mismatches, 0);

    let reopened = pool.reopen_cold();
    assert_eq!(
        recover(&reopened, JournalPolicy::Consult).outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(1), CheckpointTxg(4)),
            content: second_content,
        },
        "冷启动择 B 的根、从回落的落点读回第二次的内容"
    );
}
```
### crates/singlefs-harness/tests/second_transaction_supplement_two_commit_generated_fallback.rs:154-173（fallback_skips_a_slot_isolated_by_the_shadow_ledger）

```rust
/// 回落绕开影子账隔离的槽（与 bump 游标同一套位）：没有全空段时把 50179 在两块盘上都隔离，下一个一槽的提交内生块越过它、落到
/// 50182（50180–50181 是 A 的数据单元）。
#[test]
fn fallback_skips_a_slot_isolated_by_the_shadow_ledger() {
    let mut pool = build_pool("supplement-two-fallback-isolated");
    leave_no_empty_cluster_segment(&mut pool.allocator);
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        pool.allocator
            .isolate_abandoned(device, SlotNumber(50179), 1);
    }
    let placement = pool
        .allocator
        .try_allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(4))
        .expect("隔离之外还有空槽");
    assert_eq!(
        placement.slot,
        SlotNumber(50182),
        "被隔离的 50179 不许发出去，回落取下一个不被挡的空槽"
    );
}
```

### crates/singlefs-harness/tests/second_transaction_supplement_two_commit_generated_fallback.rs:215-293（raising_the_floor_when_the_only_empty_segment_is_held_falls_back_to_slots_outside_the_hold）

```rust
/// 增补 2 第 ② 行那一格，扣住的段外面还有不被挡的空槽：抬 F 之前开放段占满、每个全空段占掉段首一槽；抬 F 到 8 回收 A、B 与
/// 重开之后前几版释放的落点，[50240, 50304) 里只剩回收的槽 ⇒ 它是唯一的全空段、又整段扣住。补回落之前这里报 `NoSpaceFor`；
/// 今天抬 F 成功，空发布的固定点全部回落，一个都不在这次回收（扣住）的槽上；生效放开之后那一段是全池唯一的全空段。
#[test]
fn raising_the_floor_when_the_only_empty_segment_is_held_falls_back_to_slots_outside_the_hold() {
    let mut pool = build_six_overwrites_after_a_writable_remount("supplement-two-raise-falls-back");
    leave_no_empty_cluster_segment(&mut pool.allocator);
    let raised = raise_floor(&mut pool, CheckpointTxg(8))
        .expect("扣住的段外面还有不被挡的空槽：回落之后抬 F 的空发布拿得到固定点");
    let reclaimed_slots: BTreeSet<u64> = raised
        .reclaimed
        .iter()
        .flat_map(|placement| placement.slot.0..placement.slot.0 + placement.span)
        .collect();
    assert!(
        reclaimed_slots.contains(&50240),
        "A 的 extent 树根 50240 在这次回收里：{reclaimed_slots:?}"
    );
    let handed_out: Vec<(u64, u64)> = raised
        .publishes
        .iter()
        .flat_map(|publish| {
            publish
                .placements()
                .into_iter()
                .map(|placement| (publish.root.checkpoint_txg.0, placement.slot.0))
        })
        .collect();
    for (txg, slot) in &handed_out {
        assert!(
            !reclaimed_slots.contains(slot),
            "txg {txg} 的固定点落在这次回收的槽 {slot} 上：F 还没在两块盘上生效"
        );
    }
    // 回落落点手算：扣住的是 50176–50178（mkfs 两个单元）、50180–50183（A、B 的数据单元）与 [50240, 50304) 里 A、B 的提交内生块；
    // 50184–50193 是 txg 8–12 的数据单元（释放代 9–13 > 8，仍在 defer）、50194 是现行数据单元 ⇒ 不被挡的最低空槽依次是 50179、50196 起。
    let rewritten_fixed_points = [
        TransactionUnit::AllocationTree,
        TransactionUnit::AccountingTree,
        TransactionUnit::MappingTree,
        TransactionUnit::TreeTable,
    ];
    let fixed_point_slots: Vec<(u64, Vec<u64>)> = raised
        .publishes
        .iter()
        .map(|publish| {
            (
                publish.root.checkpoint_txg.0,
                rewritten_fixed_points
                    .iter()
                    .map(|identity| publish.unit(*identity).slot.0)
                    .collect(),
            )
        })
        .collect();
    assert_eq!(
        fixed_point_slots,
        vec![
            (14, vec![50179, 50196, 50197, 50198]),
            (15, vec![50199, 50200, 50201, 50202]),
            (16, vec![50203, 50204, 50205, 50206]),
        ],
        "三次空发布的固定点都落在回落政策函数给的槽上"
    );
    assert_eq!(
        pool.allocator.open_segment(),
        None,
        "空发布全走回落，没开段"
    );
    for device_map in &pool.allocator.devices {
        assert_eq!(
            device_map.lowest_empty_segment(),
            Some(SlotNumber(50240)),
            "盘 {:?}：生效放开之后，回收空的那一段是唯一的全空段",
            device_map.device
        );
        assert_eq!(device_map.empty_segments(), 1);
    }
}
```
### crates/singlefs-harness/tests/second_transaction_supplement_two_commit_generated_fallback.rs:295-338（raising_the_floor_with_no_free_slot_outside_the_hold_still_fails_and_the_hold_stays_in_the_process）

```rust
/// 同一格，扣住的槽之外一个空槽都没有：抬 F 之前把每块盘上的空槽全占掉，回收出来的全是扣住的槽 ⇒ 第一次空发布照旧报 `NoSpaceFor`、
/// 一个写都没发；扣住位留在这个进程里：记账算它们空闲，分配器却一个都发不出去。
#[test]
fn raising_the_floor_with_no_free_slot_outside_the_hold_still_fails_and_the_hold_stays_in_the_process(
) {
    let mut pool = build_six_overwrites_after_a_writable_remount("supplement-two-raise-no-slot");
    for device_map in &mut pool.allocator.devices {
        for slot in UNIT_AREA_START_SLOT..UNIT_AREA_START_SLOT + device_map.unit_area_slots() {
            if device_map.is_free(SlotNumber(slot)) {
                device_map.mark_allocated(SlotNumber(slot), 1);
            }
        }
    }
    let operations_before = pool.stream.operations().len();
    let refused = raise_floor(&mut pool, CheckpointTxg(8));
    assert!(
        matches!(
            refused,
            Err(MountError::Publish(PublishError::NoSpaceFor {
                unit: TransactionUnit::AllocationTree
            }))
        ),
        "回收出来的全是扣住的槽，第一个固定点就拿不到：{:?}",
        refused.as_ref().err()
    );
    assert_eq!(
        pool.stream.operations().len(),
        operations_before,
        "报错在任何写之前"
    );
    for device_map in &pool.allocator.devices {
        assert!(
            device_map.free_slots() > 0,
            "盘 {:?}：回收的槽记账上已算空闲",
            device_map.device
        );
    }
    assert_eq!(
        pool.allocator
            .try_allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(14)),
        Err(PlacementRefusal::NoFreeSlotOnAnyDevice),
        "扣住位没放开：之后的发布照样分配不到固定点"
    );
}
```

### crates/singlefs-harness/tests/second_transaction_supplement_two_unequal_devices.rs:230-315（filling_the_smaller_device_to_the_end_of_its_unit_area_refuses_further_placements_instead_of_panicking）

```rust
/// C368 的验收：小盘单元区末尾那一对偶数槽在两块盘上都空时，用户数据按设备取、两块盘都落在那里；之后小盘一个空槽都没有、
/// 大盘在小盘末尾之后还有 1 万多个槽——用户数据、一槽节点、两槽容器都拒成「小盘满了、设备集合怎么选没有条款」，分配器一样没动；
/// 走发布路径报 `NoSpaceFor`（数据单元先分配），录制流一步都不多、分配器退回。只看盘 0 的写法在这里 panic（小盘越界）。
#[test]
fn filling_the_smaller_device_to_the_end_of_its_unit_area_refuses_further_placements_instead_of_panicking(
) {
    let mut pool = build_unequal_pool("supplement-two-unequal-fill");
    let generation = CheckpointTxg(pool.output.root.checkpoint_txg.0 + 1);
    occupy_free_slots_below_the_smaller_unit_area_end(&mut pool.allocator, &[196_638, 196_639]);
    let last_pair = pool
        .allocator
        .try_allocate_user_data(generation)
        .expect("小盘单元区末尾那一对在两块盘上都空");
    assert_eq!(
        last_pair,
        Placement {
            slot: SlotNumber(196_638),
            span: 2
        }
    );
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        assert!(
            pool.allocator
                .record_for(device, SlotNumber(196_638))
                .is_some(),
            "盘 {device:?} 上有这一对的分配记录"
        );
    }
    for device_map in &mut pool.allocator.devices {
        if device_map.is_free(SlotNumber(196_640)) {
            device_map.mark_allocated(SlotNumber(196_640), 1);
        }
    }
    assert_eq!(pool.allocator.devices[1].free_slots(), 0, "小盘满了");
    assert!(
        pool.allocator.devices[0].free_slots() >= 211_968 - 146_465,
        "大盘在小盘末尾之后还有空槽：{}",
        pool.allocator.devices[0].free_slots()
    );

    let before = fingerprint(&pool.allocator);
    let smaller_device_full = PlacementRefusal::SomeDevicesFullDeviceSetSelectionUndefined {
        full_devices: vec![DeviceIdentity(1)],
    };
    assert_eq!(
        pool.allocator.try_allocate_user_data(generation),
        Err(smaller_device_full.clone()),
        "用户数据"
    );
    assert_eq!(
        pool.allocator
            .try_allocate_commit_generated(UnitFootprint::OneSlot, generation),
        Err(smaller_device_full.clone()),
        "一槽节点"
    );
    assert_eq!(
        pool.allocator
            .try_allocate_commit_generated(UnitFootprint::TwoSlotsAligned, generation),
        Err(smaller_device_full),
        "两槽容器"
    );
    assert_eq!(
        fingerprint(&pool.allocator),
        before,
        "拒绝时分配器一样都没动"
    );

    let operations_before = pool.stream.operations().len();
    let refused = try_overwrite(&mut pool);
    assert!(
        matches!(
            refused,
            Err(PublishError::NoSpaceFor {
                unit: TransactionUnit::Data
            })
        ),
        "{:?}",
        refused.as_ref().err()
    );
    assert_eq!(
        pool.stream.operations().len(),
        operations_before,
        "拒在任何写之前"
    );
    assert_eq!(fingerprint(&pool.allocator), before, "失败的发布退回分配器");
}
```

### crates/singlefs-harness/tests/second_transaction_supplement_two_unequal_devices.rs:317-377（devices_answering_different_places_for_a_commit_generated_block_are_refused_before_anything_is_written）

```rust
/// 各盘给提交内生块的去处不同：小盘单元区里只剩末尾三个槽、一个全空段都没有 ⇒ 小盘回落到 196638；大盘在小盘末尾之后还有全空段 ⇒
/// 开段 196672。各盘上的聚簇段要不要对齐没有条款（D3（空间分配） 已定项 8 待办 ①），拒成那个成员、分配器不动；发布里数据单元两块盘
/// 同落 196638，extent 树根在这里被拒 ⇒ `NoSpaceFor`，录制流一步都不多。
#[test]
fn devices_answering_different_places_for_a_commit_generated_block_are_refused_before_anything_is_written(
) {
    let mut pool = build_unequal_pool("supplement-two-unequal-disagree");
    let generation = CheckpointTxg(pool.output.root.checkpoint_txg.0 + 1);
    occupy_free_slots_below_the_smaller_unit_area_end(
        &mut pool.allocator,
        &[196_638, 196_639, 196_640],
    );
    let before = fingerprint(&pool.allocator);
    assert_eq!(
        before.open_segment,
        Some(SlotNumber(50_304)),
        "挂载时写行与暖机开的段还开着（已被占满）：拒绝不许把它关掉"
    );
    assert_eq!(
        pool.allocator
            .try_allocate_commit_generated(UnitFootprint::OneSlot, generation),
        Err(
            PlacementRefusal::CommitGeneratedPlacementsDifferAcrossDevicesSegmentAlignmentUndefined {
                answer_per_device: vec![
                    (
                        DeviceIdentity(0),
                        CommitGeneratedDeviceAnswer::OpenEmptySegment(SlotNumber(196_672))
                    ),
                    (
                        DeviceIdentity(1),
                        CommitGeneratedDeviceAnswer::FallBackToLowestFreeSlot(SlotNumber(196_638))
                    ),
                ],
            }
        )
    );
    assert_eq!(
        fingerprint(&pool.allocator),
        before,
        "拒绝时分配器一样都没动"
    );

    let operations_before = pool.stream.operations().len();
    let refused = try_overwrite(&mut pool);
    assert!(
        matches!(
            refused,
            Err(PublishError::NoSpaceFor {
                unit: TransactionUnit::ExtentRoot
            })
        ),
        "{:?}",
        refused.as_ref().err()
    );
    assert_eq!(
        pool.stream.operations().len(),
        operations_before,
        "拒在任何写之前"
    );
    assert_eq!(fingerprint(&pool.allocator), before, "失败的发布退回分配器");
}
```
### crates/singlefs-harness/tests/second_transaction_supplement_two_unequal_devices.rs:379-436（user_data_slots_that_differ_across_devices_are_refused_before_anything_is_written）

```rust
/// 各盘的用户数据落点不同（等大的池，只在盘 0 上隔离 50182–50183：只有拼出来的、两盘分配记录不对称的镜像走得到）：
/// 盘 0 答 50184、盘 1 答 50182，`Placement` 两盘同槽装不下，拒成那个成员、分配器不动；覆盖写报 `NoSpaceFor`，盘上逐项不变。
#[test]
fn user_data_slots_that_differ_across_devices_are_refused_before_anything_is_written() {
    let mut pool = common::build_pool("supplement-two-user-data-disagree");
    pool.allocator
        .isolate_abandoned(DeviceIdentity(0), SlotNumber(50_182), 2);
    let before = fingerprint(&pool.allocator);
    let snapshot_before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    assert_eq!(
        pool.allocator.try_allocate_user_data(CheckpointTxg(4)),
        Err(
            PlacementRefusal::UserDataSlotsDifferAcrossDevicesPerDeviceSlotsUnsupported {
                slot_per_device: vec![
                    (DeviceIdentity(0), SlotNumber(50_184)),
                    (DeviceIdentity(1), SlotNumber(50_182)),
                ],
            }
        )
    );
    assert_eq!(
        fingerprint(&pool.allocator),
        before,
        "拒绝时分配器一样都没动"
    );

    let publish_parameters = parameters();
    let content = vec![9u8; 4100];
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let refused = publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FirstFile {
            content: &content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        InstanceGeneration(1),
    );
    assert!(
        matches!(
            refused,
            Err(PublishError::NoSpaceFor {
                unit: TransactionUnit::Data
            })
        ),
        "{:?}",
        refused.as_ref().err()
    );
    assert_eq!(fingerprint(&pool.allocator), before, "失败的发布退回分配器");
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        snapshot_before,
        "盘上逐项不变：超级块槽、根环里的根、录制流步数"
    );
}
```

### crates/singlefs-harness/tests/second_transaction_supplement_two_warm_up_counter.rs:24-108（warm_up_after_a_journal_counter_other_than_zero_counts_records_on_from_it_while_txg_stays_one_and_two）

```rust
#[test]
fn warm_up_after_a_journal_counter_other_than_zero_counts_records_on_from_it_while_txg_stays_one_and_two(
) {
    let parameters = parameters();
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = (0..2u32)
        .map(|device_number| {
            (
                DeviceIdentity(device_number),
                SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
            )
        })
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let warmed = {
        let mut pool = PoolWriter::new(&parameters, &mut devices);
        let instance = acquire_instance(&mut pool).expect("取号");
        assert_eq!(instance, InstanceGeneration(1));
        warm_up_after_journal_counter(
            &mut pool,
            &genesis.root,
            instance,
            LAST_JOURNAL_COUNTER_BEFORE_WARM_UP,
        )
        .expect("暖机")
    };

    let txg_and_counter: Vec<(CheckpointTxg, u64)> = warmed
        .records
        .iter()
        .map(|record| (record.checkpoint_txg, record.counter))
        .collect();
    assert_eq!(
        txg_and_counter,
        vec![(CheckpointTxg(1), 41), (CheckpointTxg(2), 42)],
        "jsn 接着前缀末 40 数（41、42），txg 照格式常量走 1、2"
    );
    let root_txgs: Vec<CheckpointTxg> = warmed
        .roots
        .iter()
        .map(|root| root.checkpoint_txg)
        .collect();
    assert_eq!(root_txgs, vec![CheckpointTxg(1), CheckpointTxg(2)]);

    let superblock = choose_superblock(&devices).expect("暖机之后超级块自证得过");
    assert_eq!(
        superblock.journal_tail, 42,
        "超级块的 tail 存 jsn 计数器（最后一条空记录的 42），不是 txg 2"
    );

    let filesystem_identifier = unit_filesystem_identifier(&parameters.filesystem_identifier);
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        for (expected_txg, counter) in [(CheckpointTxg(1), 41), (CheckpointTxg(2), 42)] {
            let bytes = PoolReader::read(
                &devices,
                device,
                record_offset(counter, JOURNAL_RING_DEFAULT_BYTES),
                record_bytes,
            )
            .expect("环里那一格读得到");
            let record = JournalRecord::parse(&bytes, filesystem_identifier)
                .expect("jsn 对应的那一格里是一条自证过的记录");
            assert_eq!(
                (record.checkpoint_txg, record.counter),
                (expected_txg, counter),
                "盘 {} 上 jsn {counter} 那一格",
                device.0
            );
        }
        for counter_equal_to_txg in [1, 2] {
            let bytes = PoolReader::read(
                &devices,
                device,
                record_offset(counter_equal_to_txg, JOURNAL_RING_DEFAULT_BYTES),
                record_bytes,
            )
            .expect("环里那一格读得到");
            assert!(
                bytes.iter().all(|byte| *byte == 0),
                "盘 {} 上 jsn {counter_equal_to_txg} 那一格没写过：记录不按 txg 落格",
                device.0
            );
        }
    }
}
```
### crates/singlefs-harness/tests/second_transaction_supplement_two_accounting_node_full.rs:100-107（seventy_nine_devices_fill_the_accounting_node_exactly_and_still_publish）

```rust
#[test]
fn seventy_nine_devices_fill_the_accounting_node_exactly_and_still_publish() {
    assert_eq!(accounting_node_capacity(), 477);
    let mut pool = many_device_pool(79);
    let output =
        publish_first_file_version(&mut pool).expect("79 块盘：3 + 6 × 79 = 477 行，正好装满");
    assert_eq!(output.accounting_entries.len(), 477);
}
```

### crates/singlefs-harness/tests/second_transaction_supplement_two_accounting_node_full.rs:109-151（eightieth_device_overflows_the_accounting_node_and_the_publish_is_refused_before_anything_is_written）

```rust
#[test]
fn eightieth_device_overflows_the_accounting_node_and_the_publish_is_refused_before_anything_is_written(
) {
    let mut pool = many_device_pool(80);
    let images_before: Vec<SparseDevice> = pool
        .devices
        .iter()
        .map(|(_, device)| device.inner().image.clone())
        .collect();
    let result = publish_first_file_version(&mut pool);
    assert!(
        matches!(
            result,
            Err(PublishError::AccountingEntriesExceedOneNode {
                entries: 483,
                capacity: 477
            })
        ),
        "80 块盘要 3 + 6 × 80 = 483 行：{result:?}"
    );
    assert_eq!(
        pool.stream.operations().len(),
        0,
        "录制流一步都没有：一个写、一道屏障都没发"
    );
    let images_after: Vec<SparseDevice> = pool
        .devices
        .iter()
        .map(|(_, device)| device.inner().image.clone())
        .collect();
    assert!(images_after == images_before, "80 块盘上逐字节不变");
    assert!(
        pool.allocator.records().is_empty(),
        "报错在动分配器之前：一条分配记录都没有"
    );
    assert!(
        pool.allocator
            .devices
            .iter()
            .all(|device_map| device_map.allocated_slots() == 0),
        "每块盘一个槽都没分出去"
    );
}
```

### crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs:805-923（residual_record_seeded_into_the_base_image_is_applied_in_every_crash_state_whose_chain_reaches_it）

```rust
/// 步 0「预想的细节」第三条的正例（C42（残留记录冒充合法前缀）、E32（上一条时间线的残留）；验收第 2 条的正例那一半）：基镜像里预置一条
/// jsn 连续、校验和过、点名单元也在、属于所选根实例、所选根的实例表里没有回退行的记录。只展开 B 的根槽之后的小段，每个状态按独立的谓词判它该不该被施加：
/// 实例 2 的根一条都没持久（所选根还是 B 的 (1, 4)，链从 jsn 4 接得到 jsn 5）⇔ 恢复落在 (1, 5)、读出它那一版的内容。报出跑到的状态数。
#[test]
fn residual_record_seeded_into_the_base_image_is_applied_in_every_crash_state_whose_chain_reaches_it(
) {
    let prepared = prepare_with_residual_record_seeded("layer0-residual-record");
    let root_indexes: Vec<usize> = prepared
        .writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    let second_publish_root_index = root_indexes[3];
    let second_publish_record_indexes: Vec<usize> = prepared
        .writes
        .iter()
        .enumerate()
        .filter(|(index, write)| {
            write.kind == StepKind::JournalRecord
                && *index > root_indexes[2]
                && *index < second_publish_root_index
        })
        .map(|(index, _)| index)
        .collect();
    assert_eq!(second_publish_record_indexes.len(), 2, "B 的记录两份");
    let second_instance_root_indexes = &root_indexes[4..];
    // 只展开 B 的根槽段之后的段：种子是 B 之后才写下的，B 的根槽没持久而种子已在环里的状态走不到——
    // 取号的超级块槽都没持久时环里就有实例 1 的记录，checker 的 I-7.7（超级块实例代号不低于根环）① 正确地判红（先前全部展开时 3 个状态）。
    let second_publish_root_segment_index = prepared
        .segments
        .iter()
        .position(|segment| segment.contains(&second_publish_root_index))
        .expect("B 的根槽写在某一段里");
    let expand = |segment_index: usize, segment: &[usize]| {
        segment_index > second_publish_root_segment_index && segment.len() < 10
    };
    let mut states_whose_chain_reaches_the_residual_record = 0u64;
    let mut states_contradicting_the_predicate: Vec<String> = Vec::new();
    let tally = enumerate_layer0_selecting_versions_observing_each_state(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        &prepared.versions,
        &expand,
        &mut |crash_image, consulted_report| {
            let persisted = &crash_image.persisted;
            assert!(
                persisted[second_publish_root_index]
                    && second_publish_record_indexes
                        .iter()
                        .all(|record_index| persisted[*record_index]),
                "展开的状态里 B 的记录与根槽都已持久"
            );
            // B 的根槽已持久、所选根的实例表（mkfs 那一版）里没有回退行：水位还在 (1, 4)，链从 jsn 4 接到 jsn 5，直到实例 2 的某条根持久。
            let chain_reaches_the_residual_record = !second_instance_root_indexes
                .iter()
                .any(|root_index| persisted[*root_index]);
            if chain_reaches_the_residual_record {
                states_whose_chain_reaches_the_residual_record += 1;
            }
            let landed_on_the_residual_record = consulted_report.effective_root
                == Some(RESIDUAL_RECORD_ROOT)
                && consulted_report.outcome
                    == (RecoveryOutcome::FileRead {
                        root: (InstanceGeneration(1), CheckpointTxg(4)),
                        content: residual_content(),
                    });
            if chain_reaches_the_residual_record != landed_on_the_residual_record {
                states_contradicting_the_predicate.push(format!(
                    "谓词 {chain_reaches_the_residual_record}、实际走的根 {:?}、施加 {}",
                    consulted_report.effective_root, consulted_report.journal.prefix_applied
                ));
            }
        },
    );
    let expanded: Vec<Vec<usize>> = prepared
        .segments
        .iter()
        .enumerate()
        .filter(|(segment_index, segment)| expand(*segment_index, segment))
        .map(|(_, segment)| segment.clone())
        .collect();
    println!(
        "RESIDUAL_RECORD states={} states_whose_chain_reaches_the_residual_record={} violations={} checker_by_invariant(evaluated/violated/not_applicable) {}",
        tally.states,
        states_whose_chain_reaches_the_residual_record,
        tally.violations,
        tally.checker_counts_by_invariant()
    );
    assert!(
        states_contradicting_the_predicate.is_empty(),
        "该施加残留记录与恢复实际落在 (1, 5) 对不上的状态：{states_contradicting_the_predicate:?}"
    );
    assert_eq!(tally.states, closed_form_state_count(&expanded));
    assert_eq!(
        (tally.states, states_whose_chain_reaches_the_residual_record),
        (31, 19),
        "跑到的：B 与取号的超级块槽段 15 + 写行发布的记录段 3 + 写行发布的根槽段 1；跑不到的 12 个是实例 2 的某条根已持久"
    );
    assert_eq!(
        tally.violations, 0,
        "第一处违例：{:?}",
        tally.first_violation
    );
    assert_eq!(
        tally.ignored_violations, 0,
        "不看 journal 那一遍恢复同样过 oracle：{:?}",
        tally.first_ignored_violation
    );
    assert_eq!(tally.failed_states, 0);
    // I-3.1（已分配统计对得上）在实例 2 的根为最新的 12 个状态上判红（口径未定，2026-09-17 写这条用例时发现）：记账的已分配逐盘比遍历候选根多 65536 字节，
    // 正是残留记录那一版（(1, 5)，根槽从没落盘、只由记录施加出来）自己的四个固定点单元 50261–50264——写行发布 txg 6 把它们释放进 defer，
    // 环里没有一条根引用它们。checker 的「已分配 = 候选根引用的并集」与分配器「defer 里的仍算已分配」在「由记录施加出来的那一版」上分歧，
    // 改哪一边是 I-3.1 口径的设计问题；这里钉的是现状，不是认下来的行为。
    assert_checker_and_record_checker_counts(&tally, &[("I-3.1", 12)], 0);
}
```

### crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs:989-1141（stale_tail_with_a_reused_named_unit_in_its_window_recovers_every_crash_state_to_the_same_end_state）

```rust
/// 步 6 必红「陈旧 tail + 已复用的块」（verification-build.md 崩溃点重放第一版必红用例表；C77（重放起点未定义）；D23（journal 的角色与格式） 已定项 3
/// 那条 ⚠️ 与已定项 14）：固定脚本到 E 之后再覆盖写一次（txg 18 的数据单元落回 50180，A 那条记录点名的单元被合法复用），录制流里每次超级块槽写的
/// tail 都改成 2。txg 18 的 16 个单元写全持久之后的每个崩溃状态：恢复必须完成、终态与 tail 没改坏的同一个状态逐项相等、施加前验证一次都不失败；
/// 从陈旧 tail 起逐条验证、失配即中止的恢复在这些状态上中止，红在逐项相等那条。再注入一次真撕裂：施加前验证恰判失败一次（陈旧失配不进这个计数器）。
#[test]
fn stale_tail_with_a_reused_named_unit_in_its_window_recovers_every_crash_state_to_the_same_end_state(
) {
    let prepared = prepare(
        "layer0-stale-tail",
        Script::ReuseOfTheFirstDataUnitSlotAfterE,
    );
    let stale_writes = writes_with_stale_journal_tail(&prepared.writes, STALE_JOURNAL_TAIL);
    let root_indexes: Vec<usize> = prepared
        .writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    let publish_e_root_index = root_indexes[16];
    let reuse_units_segment_index = prepared
        .segments
        .iter()
        .position(|segment| segment.contains(&(publish_e_root_index + 1)))
        .expect("E 的超级块槽写与 txg 18 的单元写同段");
    assert_eq!(prepared.segments[reuse_units_segment_index].len(), 18);
    let expand = |segment_index: usize, segment: &[usize]| {
        segment_index > reuse_units_segment_index && segment.len() < 10
    };
    let mut states_with_a_reused_named_unit_after_the_stale_tail = 0u64;
    let mut records_with_a_reused_named_unit: Vec<u64> = Vec::new();
    let mut states_differing_from_the_true_tail: Vec<String> = Vec::new();
    let tally = enumerate_layer0_selecting_versions_observing_each_state(
        &prepared.base,
        &stale_writes,
        &prepared.segments,
        prepared.judged_root_index,
        &prepared.versions,
        &expand,
        &mut |stale_tail_image, stale_tail_report| {
            assert_eq!(
                choose_superblock(stale_tail_image)
                    .expect("超级块")
                    .journal_tail,
                STALE_JOURNAL_TAIL,
                "展开的状态里择到的超级块都带陈旧的 tail"
            );
            let mismatched =
                records_after_tail_naming_a_mismatched_unit(stale_tail_image, STALE_JOURNAL_TAIL);
            if !mismatched.is_empty() {
                states_with_a_reused_named_unit_after_the_stale_tail += 1;
                records_with_a_reused_named_unit = mismatched;
            }
            let true_tail_image = CrashImage {
                base: &prepared.base,
                writes: &prepared.writes,
                persisted: stale_tail_image.persisted.clone(),
            };
            let true_tail_report = recover(&true_tail_image, JournalPolicy::Consult);
            if *stale_tail_report != true_tail_report {
                states_differing_from_the_true_tail.push(format!(
                    "tail 陈旧：{:?} 走 {:?}；tail 没改坏：{:?} 走 {:?}",
                    stale_tail_report.outcome,
                    stale_tail_report.effective_root,
                    true_tail_report.outcome,
                    true_tail_report.effective_root
                ));
            }
        },
    );
    println!(
        "STALE_TAIL states={} states_with_a_reused_named_unit_after_the_stale_tail={} records_with_a_reused_named_unit={:?} violations={} failed={} verification_failed={}",
        tally.states,
        states_with_a_reused_named_unit_after_the_stale_tail,
        records_with_a_reused_named_unit,
        tally.violations,
        tally.failed_states,
        tally.verification_failed_states
    );
    assert!(
        states_differing_from_the_true_tail.is_empty(),
        "改坏 tail 之后恢复的终态与 tail 没改坏的同一个状态不同：{states_differing_from_the_true_tail:?}"
    );
    assert_eq!(
        tally.states, 8,
        "txg 18 的记录段 3 + 根槽段 1 + 超级块槽段 3 + 全部持久 1"
    );
    assert_eq!(
        (
            states_with_a_reused_named_unit_after_the_stale_tail,
            records_with_a_reused_named_unit
        ),
        (8, vec![3]),
        "8 个状态里 txg 18 的单元写都已持久：陈旧 tail 之后 A 那条记录（jsn 3）点名的 50180 两份都已被复用"
    );
    assert_eq!(
        tally.violations, 0,
        "第一处违例：{:?}",
        tally.first_violation
    );
    assert_eq!(tally.failed_states, 0, "恢复在每个状态上都完成");
    assert_eq!(
        tally.verification_failed_states, 0,
        "陈旧失配不进施加前验证的计数器：水位之下的记录不验"
    );
    assert_eq!(tally.ignored_violations, 0);
    // 记录核对器第二条判据（恢复自称的 txg ≥ 某次发布、那次发布的某个单元两份都不在）在 8 个状态上都判：A（txg 3）的数据单元两份被 txg 18 合法复用了。
    // 这条判据写成时流里没有复用，它不认「被流里更晚、已持久的写盖掉」——口径未定（2026-09-17 写这条用例时发现），这里钉的是现状，不是认下来的行为。
    assert_checker_and_record_checker_counts(&tally, &[], 8);

    // 撕裂注入：txg 18 的记录已持久、根槽与超级块槽没持久，再把它点名的数据单元两份都改坏——施加前验证判失败、恢复停在 E (3, 17)、
    // 验证失败恰为 1 次：陈旧 tail 之后那条点名块已被复用的记录（jsn 3）不进这个计数器，真撕裂与陈旧失配分得开。
    let mut torn_writes = stale_writes.clone();
    let reused_data_unit_offset = SlotNumber(50180).to_device_offset();
    let mut torn_copies = 0;
    for (write_index, write) in torn_writes.iter_mut().enumerate() {
        if write_index > publish_e_root_index
            && write.kind == StepKind::UnitWrite
            && write.offset == reused_data_unit_offset
        {
            write.bytes[4000] ^= 0xff;
            torn_copies += 1;
        }
    }
    assert_eq!(torn_copies, 2, "txg 18 的数据单元两盘各一份");
    let persisted_before_the_reuse_root: Vec<bool> = (0..torn_writes.len())
        .map(|write_index| write_index < prepared.judged_root_index)
        .collect();
    let mut torn_tally = Layer0Tally::default();
    let torn_report = evaluate_state_for_versions(
        &prepared.base,
        &torn_writes,
        persisted_before_the_reuse_root,
        prepared.judged_root_index,
        &prepared.versions,
        &mut torn_tally,
    );
    assert_eq!(
        (
            torn_report.effective_root,
            torn_report.journal.verification_failed,
            torn_report.journal.prefix_applied
        ),
        (Some((InstanceGeneration(3), CheckpointTxg(17))), 1, 0),
        "撕裂的那条被旗标、不施加：{:?}",
        torn_report.journal
    );
    assert_eq!(
        torn_tally.violations, 0,
        "停在 E、读出 E 的内容：{:?}",
        torn_tally.first_violation
    );
}
```


## 二、新文件 `crates/singlefs-core/src/write_accounting.rs` 全文

```rust
//! 每次发布按结构种类计的写（里程碑「第二个事务」增补 1 第 1 件）：写调用数与写字节。
//!
//! 种类由发布路径在构造提交步骤时给出：单元写带着它的角色，journal 记录、根槽、超级块槽由步骤成员本身定。写入口把一次写交给设备、
//! 设备报成功之后记一笔。记账不发写、不改写的次序、屏障与段序列，也不加提交步骤成员或块设备动作（D17（实现分层与第三方管道）
//! 已定项 2 / 已定项 5）。一次写调用 = 交给一块盘的一次 `write_at`：镜像的两份各算一次，与设备一层数的口径相同，
//! 所以一次发布按种类的合计要与设备一层数的逐次相等。

use std::collections::BTreeMap;

use crate::transaction::TransactionUnit;

/// 一次写调用写的是哪一种盘上结构：单元按角色分到每棵树各自的节点、树表单元、实例表单元，固定结构分 journal 记录、根槽、超级块槽。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum WrittenStructureKind {
    DataUnit,
    ExtentTreeNode,
    InodeTreeLeafContainer,
    InodeTreeRoot,
    AllocationRecordTreeNode,
    AccountingTreeNode,
    CentralMappingTreeNode,
    TreeTableUnit,
    InstanceTableUnit,
    JournalRecord,
    RootSlot,
    SuperblockSlot,
}

impl WrittenStructureKind {
    /// 报数的次序：单元按 bump 次序（实例表单元在末尾，同 `TransactionOutput::units`），固定结构按持久顺序（D16（发布语义） 已定项 7）。
    /// 合计不经过这张清单（`WritesByStructureKind::total` 只从记过的账里加）：清单漏一种，报出的各种之和就对不上合计。
    pub const IN_REPORT_ORDER: [WrittenStructureKind; 12] = [
        WrittenStructureKind::DataUnit,
        WrittenStructureKind::ExtentTreeNode,
        WrittenStructureKind::InodeTreeLeafContainer,
        WrittenStructureKind::InodeTreeRoot,
        WrittenStructureKind::AllocationRecordTreeNode,
        WrittenStructureKind::AccountingTreeNode,
        WrittenStructureKind::CentralMappingTreeNode,
        WrittenStructureKind::TreeTableUnit,
        WrittenStructureKind::InstanceTableUnit,
        WrittenStructureKind::JournalRecord,
        WrittenStructureKind::RootSlot,
        WrittenStructureKind::SuperblockSlot,
    ];

    /// 一个单元角色的写归哪一种。
    #[must_use]
    pub const fn of_unit(identity: TransactionUnit) -> WrittenStructureKind {
        match identity {
            TransactionUnit::Data => WrittenStructureKind::DataUnit,
            TransactionUnit::ExtentRoot => WrittenStructureKind::ExtentTreeNode,
            TransactionUnit::InodeLeaf => WrittenStructureKind::InodeTreeLeafContainer,
            TransactionUnit::InodeRoot => WrittenStructureKind::InodeTreeRoot,
            TransactionUnit::AllocationTree => WrittenStructureKind::AllocationRecordTreeNode,
            TransactionUnit::AccountingTree => WrittenStructureKind::AccountingTreeNode,
            TransactionUnit::MappingTree => WrittenStructureKind::CentralMappingTreeNode,
            TransactionUnit::TreeTable => WrittenStructureKind::TreeTableUnit,
            TransactionUnit::InstanceTable => WrittenStructureKind::InstanceTableUnit,
        }
    }

    /// 结果行里这一种的字段名前缀。
    #[must_use]
    pub const fn report_name(self) -> &'static str {
        match self {
            WrittenStructureKind::DataUnit => "data_unit",
            WrittenStructureKind::ExtentTreeNode => "extent_tree_node",
            WrittenStructureKind::InodeTreeLeafContainer => "inode_tree_leaf_container",
            WrittenStructureKind::InodeTreeRoot => "inode_tree_root",
            WrittenStructureKind::AllocationRecordTreeNode => "allocation_record_tree_node",
            WrittenStructureKind::AccountingTreeNode => "accounting_tree_node",
            WrittenStructureKind::CentralMappingTreeNode => "central_mapping_tree_node",
            WrittenStructureKind::TreeTableUnit => "tree_table_unit",
            WrittenStructureKind::InstanceTableUnit => "instance_table_unit",
            WrittenStructureKind::JournalRecord => "journal_record",
            WrittenStructureKind::RootSlot => "root_slot",
            WrittenStructureKind::SuperblockSlot => "superblock_slot",
        }
    }
}

/// 写调用数与写字节。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WriteCallsAndBytes {
    pub write_calls: u64,
    pub written_bytes: u64,
}

impl WriteCallsAndBytes {
    pub const NONE: WriteCallsAndBytes = WriteCallsAndBytes {
        write_calls: 0,
        written_bytes: 0,
    };

    #[must_use]
    pub const fn plus(self, other: WriteCallsAndBytes) -> WriteCallsAndBytes {
        WriteCallsAndBytes {
            write_calls: self.write_calls + other.write_calls,
            written_bytes: self.written_bytes + other.written_bytes,
        }
    }
}

/// 按结构种类记的写：只有写过的种类有条目。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WritesByStructureKind {
    counted: BTreeMap<WrittenStructureKind, WriteCallsAndBytes>,
}

impl WritesByStructureKind {
    /// 一笔都没记：新开的写入口，以及从盘上重建、不是这个进程写出的版本。
    pub const NOTHING_WRITTEN: WritesByStructureKind = WritesByStructureKind {
        counted: BTreeMap::new(),
    };

    /// 记一次写调用：这一种的写调用加 1、写字节加这次写的长度。
    pub fn count_write_call(&mut self, kind: WrittenStructureKind, bytes: &[u8]) {
        let entry = self.counted.entry(kind).or_insert(WriteCallsAndBytes::NONE);
        *entry = entry.plus(WriteCallsAndBytes {
            write_calls: 1,
            written_bytes: u64::try_from(bytes.len()).expect("一次写的长度装得进 u64"),
        });
    }

    /// 某一种的写；没写过是 `NONE`。
    #[must_use]
    pub fn of(&self, kind: WrittenStructureKind) -> WriteCallsAndBytes {
        self.counted
            .get(&kind)
            .copied()
            .unwrap_or(WriteCallsAndBytes::NONE)
    }

    /// 全部种类的合计：从记过的每一笔加起来，不经过 `IN_REPORT_ORDER`。
    #[must_use]
    pub fn total(&self) -> WriteCallsAndBytes {
        self.counted
            .values()
            .fold(WriteCallsAndBytes::NONE, |sum, kind_writes| {
                sum.plus(*kind_writes)
            })
    }

    /// `earlier` 之后多记的：一次发布的账 = 发布结束时的累计减去发布开始时的累计；这一段里没多写的种类不留条目。
    ///
    /// # Panics
    /// `earlier` 里某一种比这里多：它不是同一个写入口更早的快照（累计只增不减）。
    #[must_use]
    pub fn since(&self, earlier: &WritesByStructureKind) -> WritesByStructureKind {
        for (kind, earlier_writes) in &earlier.counted {
            let later_writes = self.of(*kind);
            assert!(
                later_writes.write_calls >= earlier_writes.write_calls
                    && later_writes.written_bytes >= earlier_writes.written_bytes,
                "累计只增不减：{kind:?} 的更早快照 {earlier_writes:?} 大于现在的 {later_writes:?}，传进来的不是同一个写入口更早的快照"
            );
        }
        let counted = self
            .counted
            .iter()
            .map(|(kind, later_writes)| {
                let earlier_writes = earlier.of(*kind);
                (
                    *kind,
                    WriteCallsAndBytes {
                        write_calls: later_writes.write_calls - earlier_writes.write_calls,
                        written_bytes: later_writes.written_bytes - earlier_writes.written_bytes,
                    },
                )
            })
            .filter(|(_, difference)| *difference != WriteCallsAndBytes::NONE)
            .collect();
        WritesByStructureKind { counted }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn since_keeps_only_the_later_writes_and_total_adds_every_counted_kind() {
        let mut cumulative = WritesByStructureKind::NOTHING_WRITTEN;
        cumulative.count_write_call(WrittenStructureKind::SuperblockSlot, &[0u8; 4096]);
        cumulative.count_write_call(WrittenStructureKind::SuperblockSlot, &[0u8; 4096]);
        let before_publish = cumulative.clone();
        cumulative.count_write_call(WrittenStructureKind::DataUnit, &[0u8; 32768]);
        cumulative.count_write_call(WrittenStructureKind::RootSlot, &[0u8; 512]);
        cumulative.count_write_call(WrittenStructureKind::SuperblockSlot, &[0u8; 4096]);
        let publish = cumulative.since(&before_publish);
        assert_eq!(
            publish.of(WrittenStructureKind::SuperblockSlot),
            WriteCallsAndBytes {
                write_calls: 1,
                written_bytes: 4096
            },
            "发布之前的两次超级块槽写不算进这次发布"
        );
        assert_eq!(
            publish.of(WrittenStructureKind::DataUnit),
            WriteCallsAndBytes {
                write_calls: 1,
                written_bytes: 32768
            }
        );
        assert_eq!(
            publish.of(WrittenStructureKind::JournalRecord),
            WriteCallsAndBytes::NONE,
            "没写过的种类是零"
        );
        assert_eq!(
            publish.total(),
            WriteCallsAndBytes {
                write_calls: 3,
                written_bytes: 37376
            }
        );
        assert_eq!(
            cumulative.total(),
            WriteCallsAndBytes {
                write_calls: 5,
                written_bytes: 45568
            }
        );
        assert_eq!(
            before_publish.since(&before_publish),
            WritesByStructureKind::NOTHING_WRITTEN,
            "两次快照之间没写：一个条目都不留"
        );
    }

    #[test]
    fn report_order_lists_every_kind_once_so_the_reported_kinds_add_up_to_the_total() {
        let mut every_kind_once = WritesByStructureKind::NOTHING_WRITTEN;
        let mut next_length_in_bytes = 1usize;
        for identity in [
            TransactionUnit::Data,
            TransactionUnit::ExtentRoot,
            TransactionUnit::InodeLeaf,
            TransactionUnit::InodeRoot,
            TransactionUnit::AllocationTree,
            TransactionUnit::AccountingTree,
            TransactionUnit::MappingTree,
            TransactionUnit::TreeTable,
            TransactionUnit::InstanceTable,
        ] {
            every_kind_once.count_write_call(
                WrittenStructureKind::of_unit(identity),
                &vec![0u8; next_length_in_bytes],
            );
            next_length_in_bytes *= 2;
        }
        for fixed_structure in [
            WrittenStructureKind::JournalRecord,
            WrittenStructureKind::RootSlot,
            WrittenStructureKind::SuperblockSlot,
        ] {
            every_kind_once.count_write_call(fixed_structure, &vec![0u8; next_length_in_bytes]);
            next_length_in_bytes *= 2;
        }
        for kind in WrittenStructureKind::IN_REPORT_ORDER {
            assert_eq!(
                every_kind_once.of(kind).write_calls,
                1,
                "{kind:?}：九个单元角色各归一种、三种固定结构各一种，每种恰好一笔"
            );
        }
        let reported = WrittenStructureKind::IN_REPORT_ORDER
            .iter()
            .fold(WriteCallsAndBytes::NONE, |sum, kind| {
                sum.plus(every_kind_once.of(*kind))
            });
        assert_eq!(
            reported,
            every_kind_once.total(),
            "报数清单漏了一种或重复了一种：各种长度互不相同（2 的幂），和对不上"
        );
        assert_eq!(
            every_kind_once.total(),
            WriteCallsAndBytes {
                write_calls: 12,
                written_bytes: 4095
            },
            "十二笔，长度 1 到 2048 各一次"
        );
        let mut names: Vec<&str> = WrittenStructureKind::IN_REPORT_ORDER
            .iter()
            .map(|kind| kind.report_name())
            .collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), 12, "结果行的字段名前缀互不相同");
    }
}
```


## 三、`crates/mutations.tsv` 第 77-106 行（名字以「增补 1：」「增补 2：」「增补 2 第」「步 0：」「步 6：」开头的 30 行，制表符分隔：变异名 / 文件 / 原串 / 改后串 / cargo test 过滤串 / 期望变红的测试名）

```tsv
增补 1：发布路径漏计数据单元的写（第一个事务按种类的合计对不上录制器在设备一层记下的写）	crates/singlefs-core/src/transaction.rs	                    self.writes_by_structure_kind.count_write_call(kind, unit);	                    if identity != TransactionUnit::Data { self.writes_by_structure_kind.count_write_call(kind, unit); }	-p singlefs-harness --test second_transaction_supplement_one_write_accounting -- first_transaction_and_overwrite_writes_by_kind	first_transaction_and_overwrite_writes_by_kind_add_up_to_the_recorded_writes_and_the_byte_table
增补 1：发布路径漏计实例表单元的写（写行发布按种类的合计对不上录制器）	crates/singlefs-core/src/transaction.rs	                    self.writes_by_structure_kind.count_write_call(kind, unit);	                    if identity != TransactionUnit::InstanceTable { self.writes_by_structure_kind.count_write_call(kind, unit); }	-p singlefs-harness --test second_transaction_supplement_one_write_accounting -- writable_remount_row_publish	writable_remount_row_publish_and_each_warm_up_publish_add_up_to_the_recorded_writes
增补 1：写入口漏计超级块槽的写（暖机按种类的合计对不上录制器）	crates/singlefs-core/src/transaction.rs	        self.writes_by_structure_kind\n            .count_write_call(WrittenStructureKind::SuperblockSlot, &slot_bytes);	        drop(slot_bytes);	-p singlefs-harness --test second_transaction_supplement_one_write_accounting -- first_transaction_and_overwrite_writes_by_kind	first_transaction_and_overwrite_writes_by_kind_add_up_to_the_recorded_writes_and_the_byte_table
增补 1：树表单元的写记到中央映射树名下（合计不变，按种类钉的字节表红）	crates/singlefs-core/src/write_accounting.rs	            TransactionUnit::TreeTable => WrittenStructureKind::TreeTableUnit,	            TransactionUnit::TreeTable => WrittenStructureKind::CentralMappingTreeNode,	-p singlefs-harness --test second_transaction_supplement_one_write_accounting -- first_transaction_and_overwrite_writes_by_kind	first_transaction_and_overwrite_writes_by_kind_add_up_to_the_recorded_writes_and_the_byte_table
增补 1：一次发布的账不减发布之前的写调用（发布前的取号写算进这次发布）	crates/singlefs-core/src/write_accounting.rs	                        write_calls: later_writes.write_calls - earlier_writes.write_calls,	                        write_calls: later_writes.write_calls,	-p singlefs-core --lib -- write_accounting::tests::since_keeps	since_keeps_only_the_later_writes_and_total_adds_every_counted_kind
增补 1：真设备二进制的设备一层比对恒判相等	crates/singlefs-harness/src/bin/first_transaction_on_device.rs	    let matches = by_kind == device;	    let matches = true;	-p singlefs-harness --bin first_transaction_on_device -- device_window_check	device_window_check_matches_only_when_both_write_calls_and_bytes_are_equal
增补 1：报数清单把根槽换成重复的 journal 记录（报出的各种之和对不上合计）	crates/singlefs-core/src/write_accounting.rs	        WrittenStructureKind::RootSlot,\n        WrittenStructureKind::SuperblockSlot,\n    ];	        WrittenStructureKind::JournalRecord,\n        WrittenStructureKind::SuperblockSlot,\n    ];	-p singlefs-core --lib -- write_accounting::tests::report_order	report_order_lists_every_kind_once_so_the_reported_kinds_add_up_to_the_total
增补 1：报数清单把根槽换成重复的 journal 记录（真设备二进制的结果行跟着错）	crates/singlefs-core/src/write_accounting.rs	        WrittenStructureKind::RootSlot,\n        WrittenStructureKind::SuperblockSlot,\n    ];	        WrittenStructureKind::JournalRecord,\n        WrittenStructureKind::SuperblockSlot,\n    ];	-p singlefs-harness --bin first_transaction_on_device -- publish_writes_line	publish_writes_line_gives_the_total_then_every_kind_in_report_order_with_zeros
增补 1：真设备二进制的窗口计数取后一次快照的字节、不减前一次	crates/singlefs-harness/src/bin/first_transaction_on_device.rs	                written_bytes: difference.written_bytes,	                written_bytes: later_device.written_bytes,	-p singlefs-harness --bin first_transaction_on_device -- pool_writes_between	pool_writes_between_adds_the_differences_of_every_device
增补 2：用户数据落点只取盘 0 的答案（C368：小盘写满之后越界 panic）	crates/singlefs-core/src/allocator.rs	            .map(|device| (device.device, device.lowest_user_data_slot(open)))	            .map(|device| (device.device, self.devices[0].lowest_user_data_slot(open)))	-p singlefs-harness --test second_transaction_supplement_two_unequal_devices -- filling_the_smaller_device	filling_the_smaller_device_to_the_end_of_its_unit_area_refuses_further_placements_instead_of_panicking
增补 2：提交内生块开段与回落只取盘 0 的答案（C368：小盘越界 panic）	crates/singlefs-core/src/allocator.rs	                    device.commit_generated_answer_without_open_segment(footprint),	                    self.devices[0].commit_generated_answer_without_open_segment(footprint),	-p singlefs-harness --test second_transaction_supplement_two_unequal_devices -- filling_the_smaller_device	filling_the_smaller_device_to_the_end_of_its_unit_area_refuses_further_placements_instead_of_panicking
增补 2：各盘给提交内生块的去处不同也不拒（D3 已定项 8 待办 ① 没条款的分支被定成取盘 0）	crates/singlefs-core/src/allocator.rs	    if answers.iter().all(|(_, answer)| *answer == first_answer) {	    if true || answers.iter().all(|(_, answer)| *answer == first_answer) {	-p singlefs-harness --test second_transaction_supplement_two_unequal_devices -- devices_answering_different_places	devices_answering_different_places_for_a_commit_generated_block_are_refused_before_anything_is_written
增补 2：各盘的用户数据落点不同也不拒（Placement 两盘同槽装不下）	crates/singlefs-core/src/allocator.rs	    if answers.iter().all(|(_, answer)| *answer == first_answer) {	    if true || answers.iter().all(|(_, answer)| *answer == first_answer) {	-p singlefs-harness --test second_transaction_supplement_two_unequal_devices -- user_data_slots_that_differ	user_data_slots_that_differ_across_devices_are_refused_before_anything_is_written
增补 2：提交内生块拒绝之前先关开放段（拒绝动了分配器）	crates/singlefs-core/src/allocator.rs	        let answer_per_device: Vec<(DeviceIdentity, Option<CommitGeneratedDeviceAnswer>)> = self	        self.open_segment = None;\n        let answer_per_device: Vec<(DeviceIdentity, Option<CommitGeneratedDeviceAnswer>)> = self	-p singlefs-harness --test second_transaction_supplement_two_unequal_devices -- devices_answering_different_places	devices_answering_different_places_for_a_commit_generated_block_are_refused_before_anything_is_written
增补 2：提交内生块段耗尽不回落（C369）	crates/singlefs-core/src/allocator.rs	            None => self\n                .lowest_commit_generated_fallback_slot(footprint)\n                .map(CommitGeneratedDeviceAnswer::FallBackToLowestFreeSlot),	            None => None,	-p singlefs-harness --test second_transaction_supplement_two_commit_generated_fallback -- a_publish_on_a_pool_without	a_publish_on_a_pool_without_an_empty_cluster_segment_falls_back_to_the_lowest_free_slot_on_every_device
增补 2：提交内生块段耗尽不回落（抬 F 的空发布在唯一全空段被扣住时又报 NoSpaceFor）	crates/singlefs-core/src/allocator.rs	            None => self\n                .lowest_commit_generated_fallback_slot(footprint)\n                .map(CommitGeneratedDeviceAnswer::FallBackToLowestFreeSlot),	            None => None,	-p singlefs-harness --test second_transaction_supplement_two_commit_generated_fallback -- raising_the_floor_when_the_only	raising_the_floor_when_the_only_empty_segment_is_held_falls_back_to_slots_outside_the_hold
增补 2：回落只看已分配位（发出影子账隔离的槽）	crates/singlefs-core/src/allocator.rs	                .all(|slot| !self.is_blocked_for_commit_generated(SlotNumber(slot)));	                .all(|slot| !self.allocated[Self::index(SlotNumber(slot))]);	-p singlefs-harness --test second_transaction_supplement_two_commit_generated_fallback -- fallback_skips_a_slot_isolated	fallback_skips_a_slot_isolated_by_the_shadow_ledger
增补 2：回落只看已分配位（抬 F 的空发布发出扣住的槽）	crates/singlefs-core/src/allocator.rs	                .all(|slot| !self.is_blocked_for_commit_generated(SlotNumber(slot)));	                .all(|slot| !self.allocated[Self::index(SlotNumber(slot))]);	-p singlefs-harness --test second_transaction_supplement_two_commit_generated_fallback -- raising_the_floor_with_no_free_slot	raising_the_floor_with_no_free_slot_outside_the_hold_still_fails_and_the_hold_stays_in_the_process
步 0：崩溃镜像的记录提示漏掉基镜像里的记录（种进基镜像的残留记录恢复扫不到）	crates/singlefs-harness/src/crash.rs	        let mut offsets = self\n            .base\n            .journal_record_offsets_hint(device, ring_start, ring_bytes)?;	        let mut offsets: Vec<DeviceOffsetInBytes> = Vec::new();	-p singlefs-harness --test second_transaction_step_zero_layer0 -- residual_record_seeded	residual_record_seeded_into_the_base_image_is_applied_in_every_crash_state_whose_chain_reaches_it
步 0：恢复把没有回退行的所选根也按 W = 0 封顶（前缀第五条误判，种进基镜像的残留记录不施加）	crates/singlefs-core/src/recovery.rs	                rollback_high_water_of_root(reader, &root),	                Some(0),	-p singlefs-harness --test second_transaction_step_zero_layer0 -- residual_record_seeded	residual_record_seeded_into_the_base_image_is_applied_in_every_crash_state_whose_chain_reaches_it
步 6：恢复从超级块 tail 起逐条验证记录的点名单元、失配即中止（E78 的自我中止形态）	crates/singlefs-core/src/recovery.rs	            let records = scan_journal(reader, &superblock);	            let records = scan_journal(reader, &superblock);\n            let trusted_tail_mismatch = records\n                .values()\n                .filter(|record| record.counter > superblock.journal_tail)\n                .any(|record| {\n                    replay_journal(\n                        reader,\n                        &RootRecord {\n                            instance: record.instance,\n                            checkpoint_txg: CheckpointTxg(record.checkpoint_txg.0 - 1),\n                            ..root\n                        },\n                        superblock.geometry.journal_ring_bytes,\n                        &BTreeMap::from([((record.instance, record.counter), record.clone())]),\n                        true,\n                        None,\n                    )\n                    .0\n                    .verification_failed\n                        > 0\n                });\n            if trusted_tail_mismatch {\n                return RecoveryReport {\n                    outcome: RecoveryOutcome::Failed {\n                        root: Some(root_key),\n                        failure: RecoveryFailure::NoValidRoot,\n                    },\n                    effective_root: None,\n                    journal: JournalScanReport::default(),\n                    mapping_fallbacks,\n                };\n            }	-p singlefs-harness --test second_transaction_step_zero_layer0 -- stale_tail	stale_tail_with_a_reused_named_unit_in_its_window_recovers_every_crash_state_to_the_same_end_state
步 6：I-3.8 的判定没跑到（层 0 报「不适用」，阴性结果与没跑到混在一起）	crates/singlefs-checker/src/walk.rs	        self.judgements.judge(\n            "I-3.8",	        let _ = (\n            "I-3.8",	-p singlefs-harness --test second_transaction_step_zero_layer0 -- every_crash_state_outside_the_two_unit_segments	every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims
步 6：I-2.1 的判定没跑到（层 0 报「不适用」，阴性结果与没跑到混在一起）	crates/singlefs-checker/src/image.rs	        judgements.judge("I-2.1", matches, || {	        let _ = ("I-2.1", matches, || {	-p singlefs-harness --test second_transaction_step_zero_layer0 -- every_crash_state_outside_the_two_unit_segments	every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims
步 6：层 0 计数不记「不适用」（评估过 + 不适用 ≠ 状态数）	crates/singlefs-harness/src/crash.rs	            InvariantVerdict::NotApplicable(_) => {\n                *tally\n                    .checker_not_applicable_states\n                    .entry(invariant)\n                    .or_insert(0) += 1;\n            }	            InvariantVerdict::NotApplicable(_) => {}	-p singlefs-harness --test second_transaction_step_zero_layer0 -- every_crash_state_outside_the_two_unit_segments	every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims
增补 2 第 21 行：暖机把 jsn 计数器写成 txg（C366；接在 jsn 40 之后时记录落进 jsn 1、2 那两格、tail 写 2）	crates/singlefs-core/src/transaction.rs	txg: CheckpointTxg(txg_number),\n                counter: previous_counter + 1,	txg: CheckpointTxg(txg_number),\n                counter: txg_number,	-p singlefs-harness --test second_transaction_supplement_two_warm_up_counter -- warm_up_after_a_journal_counter	warm_up_after_a_journal_counter_other_than_zero_counts_records_on_from_it_while_txg_stays_one_and_two
增补 2 第 19 行：出生序号发号器挪回每装一个文件对象重建一次（同一个 checkpoint 的第二个对象从 0 重数）	crates/singlefs-core/src/transaction.rs	) -> FileVersionUnits {\n    let txg = checkpoint.txg;	) -> FileVersionUnits {\n    let mut sequences_rebuilt_on_every_call = BirthSequenceAllocator::default();\n    let sequences = &mut sequences_rebuilt_on_every_call;\n    let txg = checkpoint.txg;	-p singlefs-core --lib -- transaction::tests::second_file_object	second_file_object_in_the_same_checkpoint_continues_birth_sequences_instead_of_restarting_at_zero
增补 2 第 28 行：记账树装不下不判（80 块盘走到装节点的断言 panic）	crates/singlefs-core/src/transaction.rs	    if accounting_entries_of_this_publish > accounting_node_capacity {	    if false && accounting_entries_of_this_publish > accounting_node_capacity {	-p singlefs-harness --test second_transaction_supplement_two_accounting_node_full -- eightieth_device	eightieth_device_overflows_the_accounting_node_and_the_publish_is_refused_before_anything_is_written
增补 2 第 28 行：记账树正好装满 477 行也判装不下（79 块盘发布被拒）	crates/singlefs-core/src/transaction.rs	    if accounting_entries_of_this_publish > accounting_node_capacity {	    if accounting_entries_of_this_publish >= accounting_node_capacity {	-p singlefs-harness --test second_transaction_supplement_two_accounting_node_full -- seventy_nine_devices	seventy_nine_devices_fill_the_accounting_node_exactly_and_still_publish
增补 2 第 30 行：真设备二进制的挂载窗口从重开那一刻算起（取号的两次超级块槽写进了设备一层的数、不在按种类的账里）	crates/singlefs-harness/src/bin/first_transaction_on_device.rs	        pool_writes_between(&counts_after_acquisition, &counts_after_mount),	        pool_writes_between(&vec![DeviceCallCounts { write_calls: 0, written_bytes: 0, force_unit_access_writes: 0, barrier_calls: 0 }; counts_after_mount.len()], &counts_after_mount),	-p singlefs-harness --bin first_transaction_on_device -- second_instance_mode	second_instance_mode_mounts_writes_the_row_warms_up_publishes_the_third_version_and_every_window_matches
增补 2 第 30 行：真设备二进制每道屏障都覆盖「第一道屏障」那份计数（挂载窗口从最后一道屏障算起）	crates/singlefs-harness/src/bin/first_transaction_on_device.rs	        if self.counts_when_first_barrier_arrived.is_none() {	        if true {	-p singlefs-harness --bin first_transaction_on_device -- second_instance_mode	second_instance_mode_mounts_writes_the_row_warms_up_publishes_the_third_version_and_every_window_matches
```
