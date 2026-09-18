# 附录二：里程碑「第二个事务」第二波代码改动（工作区无提交点，按「文件::项名」整段抽；2026-09-18 02:28 UTC 生成）

**基准**：工作区没有提交点，给不出 `git diff`。本附录按正文第一节表里给的「文件::项名」清单，用 `python3 research/scripts/quote-rust-items.py 文件::项名 …` 逐项整段抽取（每段前自带文件名与行区间，抽取时已回读逐字节比对，退出码 0）；四个全新模块与一个全新测试文件按定义「附新文件全文」整份抄入；`crates/mutations.tsv` 按主 agent 指定的第 101–120 行整段抄入。

**取法与正文条目的对应，两处需要说明**：
- 正文写「两处 `persist` 闭包」——闭包本身不是可命名项，`quote-rust-items.py` 抽不到匿名闭包；改抽这两处闭包各自所在的最小可命名外层函数：`publish_without_units`（第一处，第 504-571 行）与 `publish_admitted`（第二处，第 1562-2144 行），两处闭包都在抽出的范围内。
- 正文写「`candidate_indexes`」——现查它在 `crates/singlefs-checker/src/walk.rs` 里不是独立项，一处是 `judge_release_generation_and_tree_table_birth` 的参数（已被该项的抽取覆盖），另一处是 `check_pool_image` 里的局部变量；改抽 `check_pool_image` 整个函数（第 1157-1389 行）以覆盖后一处。

**正文写「`first_transaction_step_five_publish.rs` 与 `second_transaction_step_five_reuse.rs` 改过的断言」，未给测试函数全名**：现查 `crates/mutations.tsv` 第 101-120 行（本附录第三节）确认了具体是哪两条——`inode_and_extent_lookups_from_the_root_read_the_first_file_back`（第 11 行「改动计数写回 1」那条变异指定的用例）与 `reclaiming_without_raising_the_floor_reuses_a_slot_a_candidate_root_still_references_and_the_checker_goes_red`（收口表第 22 行「候选根那一格不判 I-4.8」那条变异指定的用例），已按这两个全名抽取；不排除这两个文件里还有别的断言也因这一轮改动而改过但没被这段 `mutations.tsv` 摘录点到，未逐一核对全部 8 / 11 个用例。

## 一、已有文件里被改动的 Rust 项（quote-rust-items.py 整段抽，回读逐字节一致，退出码 0）

### crates/singlefs-core/src/mount.rs:31-112（MountError）

```rust
/// 可写挂载没做成。
#[derive(Debug)]
pub enum MountError {
    Recovery(RecoveryFailure),
    /// 所选根下面有文件版本，而环里一条自证过的记录都没有：从盘上重建上一版要一条记录顶着，拿不出来
    /// （树表 0 条的根用不到记录，刚 mkfs 的池不走这里）。
    FileVersionWithoutAnyJournalRecord,
    /// 所选根指着的实例表单元解不出行与链指针。
    InstanceTableMalformed,
    /// 抬 F 要读现行版本的实例表判候选集，而现行版本里没有重写过的实例表单元（`TransactionOutput::units` 里没有实例表那一角色）：
    /// mkfs 加第一个事务的进程、只做过 mkfs 的池可写挂载之后发了第一个文件版本的进程，现行版本都是 `publish_first_file` 的输出，都走到这里
    /// （alloc-basis 第二轮云端攻方腿第七节第 1 条：`TransactionOutput::unit` 在那里会 panic）。
    RaiseNeedsRewrittenInstanceTableUnitInCurrentVersion,
    Acquisition(AcquisitionFailed),
    Publish(PublishError),
    /// 回退的目标根不在根环里（没有那个 (实例, txg) 的可读根槽）。
    RollbackTargetNotInRing(RollbackTarget),
    /// 回退的目标根在根环里，却不在回退候选集里：被抛弃时间线的根，或 txg 低于回退下界 F。
    RollbackTargetNotACandidate {
        target: RollbackTarget,
        reason: &'static str,
    },
    /// 要抬的 F 超过上限 min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)（D16（发布语义） 已定项 1）。
    RollbackFloorAboveCeiling {
        requested: CheckpointTxg,
        ceiling: CheckpointTxg,
    },
    /// 所选根的树表 0 条（还没发布过文件版本），而这次挂载要给 [max(所选根的实例, 1), 新实例) 写实例表行：写行要重写实例表，
    /// 没有文件版本的一版上它的落点记在哪没有条款，第一版不支持。在取号之前拒绝，盘上一个字节都不动。
    InstanceRowsOnVersionWithoutFileUnsupported {
        chosen_root: RollbackTarget,
        first_row_instance: InstanceGeneration,
        instance_to_acquire: InstanceGeneration,
    },
    /// 回退的目标根的树表 0 条（还没发布过文件版本）：回退行要重写实例表，没有文件版本的一版上它的落点记在哪没有条款，
    /// 第一版不支持。在任何写之前拒绝。
    RollbackToVersionWithoutFileUnsupported(RollbackTarget),
    /// 树表 0 条、而根记录指着的实例表或树表不是 mkfs 写的那一版（指针的诞生 txg 不是 0）：这样一版的分配记录在哪没有条款，
    /// 这个实现自己写不出这样的根（坏盘或别的写者才有），拒绝挂载。
    VersionWithoutFileNotWrittenByMakeFilesystem {
        root: RollbackTarget,
        instance_table_birth_txg: CheckpointTxg,
        tree_table_birth_txg: CheckpointTxg,
    },
    /// 算抬 F 的上限时一条有效根（不是被抛弃的、txg ≥ F）的树表读不出或解不开：它算不算非空判不了，读不出要走修复、不跳过也不猜，
    /// 这次抬 F 拒绝（不回收、不发布）。
    RollbackFloorCeilingNeedsUnreadableValidRootTreeTable {
        root: RollbackTarget,
        failure: RecoveryFailure,
    },
    /// 取号之前判定时算出的号与取号写之前重算的号不同（两次读超级块之间有瞬时读错）：判定作废，一个字节都没写。
    InstanceGenerationChangedBeforeAcquisition {
        expected: InstanceGeneration,
        recomputed: InstanceGeneration,
    },
    /// 写行那次发布的准入（这次之后的分配记录条数、这次要写的记账行数）算不过：在**取号之前**拒绝，盘上一个字节都不动、
    /// 两块盘超级块里的实例代号不动（增补 2 第 20a 行，代码三方第一轮打中）。改之前这两条只在发布路径里算，
    /// 取号（两次超级块槽写 + 一道屏障）已经写完才报出来，而取号的回卷只管取号自己那几次写报错、管不到之后的发布失败 ⇒
    /// 分配记录树满了的池此后每试一次可写挂载就再烧一个实例代号。
    RowPublishAdmissionRefusedBeforeAcquisition {
        instance_to_acquire: InstanceGeneration,
        cause: PublishError,
    },
    /// 暖机那几次空发布（写行之后推到本实例的根覆盖每块盘，D16（发布语义） 已定项 8 甲′）里第几次的准入算不过：连写行那次一起
    /// 在取号之前算，算不过在任何写之前返回（增补 2 第 20a 行，2026-09-18 用户定案）。改之前取号之前只算写行那一次，
    /// 「写行装得下、暖机第 N 次装不下」的池是取号写完、写行也发完，暖机才报错——实例代号照样烧掉。
    WarmUpAdmissionRefusedBeforeAcquisition {
        instance_to_acquire: InstanceGeneration,
        /// 第几次暖机空发布算不过，从 1 数。
        warm_up_publish_index: usize,
        /// 这次挂载要推几次暖机空发布（按根环落点公式现算）。
        warm_up_publishes_planned: usize,
        cause: PublishError,
    },
    /// 空池挂载（所选根的树表 0 条、要写的行为空）的形状不是 mkfs 同一个进程里第一个事务那一种：新实例的第一次发布不是 txg 1、jsn 1，
    /// 或零单元写行（txg 1）与暖机（txg 2）落在同一块盘上。第一个文件版本写死 txg 3 / jsn 3 接不上这样的形状，在取号之前拒绝。
    FormattedPoolMountNotShapedLikeTheFirstTransaction {
        chosen_root: RollbackTarget,
        first_txg: CheckpointTxg,
        first_counter: u64,
    },
}
```

### crates/singlefs-core/src/mount.rs:836-873（refuse_publishes_before_acquisition_that_do_not_pass_admission）

```rust
/// 这次挂载在取号之后要发的那几次——写行一次、暖机 `warm_up_publishes_planned` 次——的准入，在取号之前一串算完
/// （增补 2 第 20a 行：写行那一半是代码三方第一轮打中的，暖机那一半是 2026-09-18 用户定案）：分配记录树与记账树在每一次之后
/// 都要装得下，前面几次要新增的分配记录算进后面几次的基数；算不过就在任何写之前返回——取号写出去的实例代号一去不回，
/// 回卷只管取号自己那几次写报错，管不到取号之后的发布失败。
/// 读的是内存里这个分配器，与发布路径那一遍同一份输入（取号不碰它），发布路径那一遍仍在、是动分配器之前的最后一道。
/// 树表 0 条的一版上写行与暖机都走零单元发布（不写单元、不加分配记录、不写记账行），那一格没有这两条准入、不判。
fn refuse_publishes_before_acquisition_that_do_not_pass_admission(
    allocator: &PoolAllocator,
    start: &InstanceStart,
    instance_to_acquire: InstanceGeneration,
    warm_up_publishes_planned: usize,
) -> Result<(), MountError> {
    match &start.previous {
        PreviousVersion::WithFile { .. } => {
            let shapes: Vec<PublishShape> = std::iter::once(PublishShape::ROW_PUBLISH)
                .chain(std::iter::repeat_n(
                    PublishShape::EMPTY_PUBLISH,
                    warm_up_publishes_planned,
                ))
                .collect();
            publish_sequence_admission(allocator, &shapes).map_err(|refusal| {
                match refusal.publish_index {
                    0 => MountError::RowPublishAdmissionRefusedBeforeAcquisition {
                        instance_to_acquire,
                        cause: refusal.cause,
                    },
                    warm_up_publish_index => MountError::WarmUpAdmissionRefusedBeforeAcquisition {
                        instance_to_acquire,
                        warm_up_publish_index,
                        warm_up_publishes_planned,
                        cause: refusal.cause,
                    },
                }
            })
        }
        PreviousVersion::WithoutFile(_) => Ok(()),
    }
}
```

### crates/singlefs-core/src/mount.rs:875-905（warm_up_publish_txgs）

```rust
/// 暖机要推的那几次空发布，按它们的 checkpoint_txg 列出来（D16（发布语义） 已定项 8 甲′）：从写行那次发布的 txg 起逐个加一，
/// 按根环落点公式看落哪块盘，直到本实例的根覆盖每块盘，至多根环的区域数那么多次。
/// 纯算——只读区域归属表与池里的盘，不碰盘、不碰分配器、不看分配记录，所以取号之前算得出来。
/// 取号之前算准入与取号之后推发布共用这一份计划（`establish_instance` 按它推），两处的次数因此必定相同：
/// 各算各的时只要有一处多算一次，准入就罩不住实际推的那几次。
fn warm_up_publish_txgs(
    parameters: &MakeFilesystemParameters,
    all_devices: &[DeviceIdentity],
    row_publish_txg: CheckpointTxg,
) -> Vec<CheckpointTxg> {
    let device_of_txg = |txg: CheckpointTxg| {
        parameters.region_devices[usize::try_from(target_for_publish(txg).region).expect("区域号")]
    };
    let mut covered: Vec<DeviceIdentity> = vec![device_of_txg(row_publish_txg)];
    let mut warm_up_publishes: Vec<CheckpointTxg> = Vec::new();
    let mut latest_txg = row_publish_txg;
    while all_devices
        .iter()
        .any(|identity| !covered.contains(identity))
        && u64::try_from(warm_up_publishes.len()).expect("次数") < ROOT_RING_REGIONS
    {
        let next_txg = CheckpointTxg(latest_txg.0 + 1);
        let device = device_of_txg(next_txg);
        if !covered.contains(&device) {
            covered.push(device);
        }
        warm_up_publishes.push(next_txg);
        latest_txg = next_txg;
    }
    warm_up_publishes
}
```

### crates/singlefs-core/src/mount.rs:907-1045（establish_instance）

```rust
/// 取号 → 写行发布 → 暖机到本实例的根覆盖每块盘：可写挂载与回退共用的后半段。
fn establish_instance<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    mut allocator: PoolAllocator,
    start: InstanceStart,
) -> Result<Mounted, MountError> {
    let isolated_slots_per_device: Vec<(DeviceIdentity, u64)> = allocator
        .devices
        .iter()
        .map(|device_map| (device_map.device, device_map.isolated_slots()))
        .collect();
    let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
    let all_devices: Vec<DeviceIdentity> =
        pool.devices.iter().map(|(identity, _)| *identity).collect();
    // 暖机推哪几次，取号之前先算出来：准入按这一份算，取号之后的循环也按这一份推。
    let warm_up_publishes_planned = warm_up_publish_txgs(parameters, &all_devices, start.first_txg);
    let instance_to_acquire = instance_generation_to_acquire(&pool);
    refuse_instance_rows_on_version_without_file(&start, instance_to_acquire)?;
    refuse_formatted_pool_mount_not_shaped_like_the_first_transaction(parameters, &start)?;
    refuse_publishes_before_acquisition_that_do_not_pass_admission(
        &allocator,
        &start,
        instance_to_acquire,
        warm_up_publishes_planned.len(),
    )?;
    let instance = acquire_expected_instance(&mut pool, instance_to_acquire).map_err(
        |failure| match failure {
            ExpectedInstanceAcquisitionFailed::InstanceGenerationChangedBeforeWrite {
                expected,
                recomputed,
            } => MountError::InstanceGenerationChangedBeforeAcquisition {
                expected,
                recomputed,
            },
            ExpectedInstanceAcquisitionFailed::Acquisition(acquisition) => {
                MountError::Acquisition(acquisition)
            }
        },
    )?;

    // 写行：给 [max(上一个实例, 1), 新实例) 里每个实例各一行——上一个实例 (i, T, W)（回退时是回退行，flags bit0 = 1），
    // 中间实例 (i, 0, 0)；实例 0（mkfs）不写行（D18（块里携带什么信息） 已定项 11）。
    let mut rows_written = Vec::new();
    let first_row_instance = start.previous_row.instance.0.max(1);
    for row_instance in first_row_instance..instance.0 {
        let row = if row_instance == start.previous_row.instance.0 {
            InstanceRow {
                instance: InstanceGeneration(row_instance),
                selected_root_txg: start.previous_row.selected_root_txg,
                applied_transaction_high_water: start.previous_row.applied_transaction_high_water,
                is_rollback: start.previous_row.is_rollback,
            }
        } else {
            InstanceRow {
                instance: InstanceGeneration(row_instance),
                selected_root_txg: CheckpointTxg(0),
                applied_transaction_high_water: 0,
                is_rollback: false,
            }
        };
        rows_written.push(row);
    }
    let row_publish = match &start.previous {
        PreviousVersion::WithFile { output, table } => {
            PoolVersion::WithFile(publish_rows_on_file_version(
                &mut pool,
                &mut allocator,
                output,
                table,
                &rows_written,
                RowPublishIdentity {
                    txg: start.first_txg,
                    counter: start.next_counter,
                    instance,
                    rollback_floor: start.effective_floor,
                },
            )?)
        }
        PreviousVersion::WithoutFile(previous_root) => {
            assert!(
                rows_written.is_empty(),
                "取号之前 refuse_instance_rows_on_version_without_file 按同一个号核过：树表 0 条的一版上要写的行为空"
            );
            // 上一个实例是 0、要写的行为空：第一次可写挂载的形态，零单元（D16（发布语义） 已定项 9「树表 0 条 ⇒ 零单元」；
            // 第一次可写挂载要写的区间是空的，第一个事务的字节不变，layout/01-first-txn.md 八「第一次之后的可写挂载（写行）」那一行 2026-09-14 的改名注）。
            PoolVersion::WithoutFile(
                publish_without_units(
                    &mut pool,
                    previous_root,
                    ZeroUnitPublishPlan {
                        txg: start.first_txg,
                        counter: start.next_counter,
                        instance,
                        back_chain: 0,
                        rollback_floor: start.effective_floor,
                    },
                )
                .map_err(PublishError::from)?,
            )
        }
    };

    // 暖机（D16（发布语义） 已定项 8 甲′）：本实例的根覆盖两块盘之前连推空发布，推哪几次按取号之前算好的那一份计划
    // （`warm_up_publishes_planned`），不在这里另算一遍——两处各算各的，只要有一处多算一次，取号之前那道准入就罩不住实际推的那几次。
    assert_eq!(
        row_publish.root().checkpoint_txg,
        start.first_txg,
        "写行那次发布的 txg 就是算暖机计划用的那个：两条路（带文件的写行、树表 0 条的零单元发布）交给发布路径的 txg 都取 start.first_txg"
    );
    let mut current = row_publish.clone();
    let mut warm_up_publishes = Vec::new();
    for planned_txg in &warm_up_publishes_planned {
        assert_eq!(
            CheckpointTxg(current.root().checkpoint_txg.0 + 1),
            *planned_txg,
            "这次空发布要用的 txg 与计划里的那一个相同：`publish_empty_after` 取的是现行那一版的 txg 加一，计划也是逐个加一"
        );
        let next = publish_empty_after(&mut pool, &mut allocator, &current, instance)?;
        warm_up_publishes.push(next.clone());
        current = next;
    }

    Ok(Mounted {
        output: MountOutput {
            instance,
            chosen_root: start.chosen_root,
            effective_root: start.effective_root,
            journal: start.journal,
            rows_written,
            row_publish,
            warm_up_publishes,
            isolated_slots_per_device,
            abandoned_roots_unreadable: start.abandoned_roots_unreadable,
        },
        allocator,
        current,
    })
}
```

### crates/singlefs-core/src/transaction.rs:1059-1065（PublishShape）

```rust
/// 一次发布重写哪些角色，只由两件事定：这次写不写文件版本、实例表是重写还是照抄。计划的身份字段（新实例代号、表的字节、txg）
/// 都不进来——可写挂载要在取号之前算写行那次发布的准入，而那时新实例代号还没取（增补 2 第 20a 行）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PublishShape {
    pub rewrites_file_version: bool,
    pub rewrites_instance_table: bool,
}
```

### crates/singlefs-core/src/transaction.rs:1067-1107（impl PublishShape）

```rust
impl PublishShape {
    /// 写行那次发布的形状：不写文件版本、重写实例表（`publish_rows_on_file_version` 交给 `publish_version` 的那张计划同形）。
    pub const ROW_PUBLISH: PublishShape = PublishShape {
        rewrites_file_version: false,
        rewrites_instance_table: true,
    };

    /// 暖机那几次空发布的形状：不写文件版本、实例表照抄（`publish_empty_after` 交给 `publish_version` 的那张计划同形，
    /// 那里有一条断言钉住两者相等）。记账树已经存在 ⇒ 照样重写四个固定点单元（D16（发布语义） 已定项 9）。
    pub const EMPTY_PUBLISH: PublishShape = PublishShape {
        rewrites_file_version: false,
        rewrites_instance_table: false,
    };

    /// 这次发布重写的角色，按 bump 次序：用户数据先取（自己的政策）；提交内生块按树 ID 升序、树内先叶后根、映射树倒数第二、树表最末
    /// （D3（空间分配） 已定项 10 ⑤）。实例表单元归树 0，排在全部提交内生块之前（mkfs 也是先实例表后树表；里程碑步 3 的决策点）。
    #[must_use]
    pub fn rewritten_roles(self) -> Vec<TransactionUnit> {
        let mut roles = Vec::new();
        if self.rewrites_file_version {
            roles.push(TransactionUnit::Data);
        }
        if self.rewrites_instance_table {
            roles.push(TransactionUnit::InstanceTable);
        }
        if self.rewrites_file_version {
            roles.extend([
                TransactionUnit::ExtentRoot,
                TransactionUnit::InodeLeaf,
                TransactionUnit::InodeRoot,
            ]);
        }
        roles.extend([
            TransactionUnit::AllocationTree,
            TransactionUnit::AccountingTree,
            TransactionUnit::MappingTree,
            TransactionUnit::TreeTable,
        ]);
        roles
    }
}
```

### crates/singlefs-core/src/transaction.rs:1263-1276（publish_admission）

```rust
/// 一次发布的准入里与这次写什么内容无关的那两条：分配记录树与记账树第一版各只有一个节点（分裂不做），这次发布之后都要装得下。
/// 只读——不动分配器、不发一个写，算不过时盘上逐字节不变。两处调它：发布路径在动分配器之前（`publish_version`）；
/// 可写挂载在**取号之前**按这次挂载要发的那几次（写行 + 暖机）算一遍（`publish_sequence_admission`，增补 2 第 20a 行：
/// 算不过就不许先把实例代号烧掉——取号是两次超级块槽写加一道屏障，之后再拒绝，池此后每试一次可写挂载就多烧一个代号）。
/// 两处读的是同一个内存里的分配器，取号不碰它，所以两次必定同答案；发布路径那一遍仍留着，它是动分配器之前的最后一道。
///
/// # Errors
/// `AllocationRecordsExceedOneNode`（这次之后的分配记录条数越过一个节点）、`AccountingEntriesExceedOneNode`（记账行数越过一个节点）。
pub fn publish_admission(
    allocator: &PoolAllocator,
    rewritten: &[TransactionUnit],
) -> Result<(), PublishError> {
    admission_of_one_publish(allocator, allocator.records().len(), rewritten)
}
```

### crates/singlefs-core/src/transaction.rs:1309-1344（admission_of_one_publish）

```rust
/// 一次发布的那两条准入，分配记录的基数由调用方给：发布路径给的是分配器此刻的记录数，取号之前那一串给的是把前面几次
/// 要新增的算进去之后的数。
fn admission_of_one_publish(
    allocator: &PoolAllocator,
    records_before_this_publish: usize,
    rewritten: &[TransactionUnit],
) -> Result<(), PublishError> {
    // 分配记录树第一版只有一个节点：这次发布之后装不下就报错，不许走到 build_index_node 的断言
    // （三方代码第一轮攻方腿：第 50 次覆盖写 panic）。释放只改写记录、不加；这次重写的每个角色每盘各加一条。
    let records_after_this_publish =
        records_before_this_publish + rewritten.len() * allocator.devices.len();
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
    // 记账树第一版也只有一个节点：行数只随盘数变（代码三方第二轮攻方腿 Y3：约 80 块盘）。
    // 装行那一段按同一个 `allocator.devices` 装，并断言行数与这里算的相等。
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
    Ok(())
}
```

### crates/singlefs-core/src/transaction.rs:1278-1300（publish_sequence_admission）

```rust
/// 接连几次发布的准入，在第一次动分配器之前一次算完：按次序逐次走 `publish_admission` 那两条，前面几次要新增的分配记录
/// 算进后面几次的基数。基数只加不减，理由与单次那条同一句——释放只改写记录、不加，回收要 F 抬到释放代之上（`reclaim_released_up_to`），
/// 而这一串（写行 + 暖机）里不抬 F。
///
/// # Errors
/// `PublishSequenceRefusal`：第几次算不过（从 0 数），连它的 `PublishError` 一起交回。
pub fn publish_sequence_admission(
    allocator: &PoolAllocator,
    shapes: &[PublishShape],
) -> Result<(), PublishSequenceRefusal> {
    let mut records_before_this_publish = allocator.records().len();
    for (publish_index, shape) in shapes.iter().enumerate() {
        let rewritten = shape.rewritten_roles();
        admission_of_one_publish(allocator, records_before_this_publish, &rewritten).map_err(
            |cause| PublishSequenceRefusal {
                publish_index,
                cause,
            },
        )?;
        records_before_this_publish += rewritten.len() * allocator.devices.len();
    }
    Ok(())
}
```

### crates/singlefs-core/src/transaction.rs:1302-1307（PublishSequenceRefusal）

```rust
/// 一串发布的准入里第几次算不过（从 0 数）、为什么。
#[derive(Debug)]
pub struct PublishSequenceRefusal {
    pub publish_index: usize,
    pub cause: PublishError,
}
```

### crates/singlefs-core/src/transaction.rs:240-244（writes_of_failed_publishes）

```rust
    /// 这个写入口上中途失败的发布各自已记的写，按失败的先后排。合计对账时它与每一次成功发布的账相加，才等于设备一层记下的写。
    #[must_use]
    pub fn writes_of_failed_publishes(&self) -> &[WritesByStructureKind] {
        &self.writes_of_failed_publishes
    }
```

### crates/singlefs-core/src/transaction.rs:230-238（count_failed_publish）

```rust
    /// 中途失败的那次发布已记的写：这次开始时的快照到此刻的差，记成失败账里的一份（增补 2 第 20b 行）。
    /// 一次失败记一份，哪怕一个写都没发出去就失败（那一份是空的）——份数就是这个写入口上失败过几次发布。
    fn count_failed_publish(&mut self, writes_before_this_publish: &WritesByStructureKind) {
        let written_before_the_failure = self
            .writes_by_structure_kind
            .since(writes_before_this_publish);
        self.writes_of_failed_publishes
            .push(written_before_the_failure);
    }
```

### crates/singlefs-core/src/transaction.rs:504-571（publish_without_units）

```rust
/// 零单元发布（D16（发布语义） 已定项 9「树表 0 条 ⇒ 零单元」）：屏障 → 空记录 → 屏障 → 根槽 FUA → 超级块槽轮换；
/// 空记录不点名任何单元、事务号 0、提交标记 1，新根段照上一版的根，根记录照上一版的根、只换 checkpoint_txg、实例代号与回退下界。
/// 第一次可写挂载的暖机与「只做过 mkfs 的池」上的可写挂载都走它（第一个事务的字节不变）。
///
/// # Errors
/// 块设备报的错原样交回。
pub fn publish_without_units<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    previous_root: &RootRecord,
    plan: ZeroUnitPublishPlan,
) -> Result<ZeroUnitPublishOutput, BlockDeviceError> {
    let record = JournalRecord {
        instance: plan.instance,
        counter: plan.counter,
        checkpoint_txg: plan.txg,
        transaction: 0,
        is_commit: true,
        back_chain: plan.back_chain,
        filesystem_identifier: unit_filesystem_identifier(&pool.parameters.filesystem_identifier),
        new_tree_table: previous_root.tree_table,
        new_mapping_root: previous_root.mapping_root,
        new_tree_identifier_watermark: previous_root.tree_identifier_watermark,
        new_rollback_floor: plan.rollback_floor,
        named: Vec::new(),
    };
    let record_bytes = record.to_bytes();
    let root = RootRecord {
        filesystem_identifier: previous_root.filesystem_identifier,
        instance: plan.instance,
        checkpoint_txg: plan.txg,
        tree_table: previous_root.tree_table,
        tree_identifier_watermark: previous_root.tree_identifier_watermark,
        rollback_floor: plan.rollback_floor,
        instance_table: previous_root.instance_table,
        mapping_root: previous_root.mapping_root,
    };
    let root_slot = root.to_slot(pool.root_slot_bytes());
    let writes_before_this_publish = pool.writes_by_structure_kind.clone();
    // 中途失败时这次已记的写要交出去（增补 2 第 20b 行）：四步收在一个闭包里，失败在这里记账再把错原样交回。
    let persist = |writer: &mut PoolWriter<'_, Device>| -> Result<(), BlockDeviceError> {
        writer.perform(CommitStep::Barrier)?;
        writer.perform(CommitStep::WriteJournalRecordToEveryDevice {
            counter: plan.counter,
            record: &record_bytes,
        })?;
        writer.perform(CommitStep::Barrier)?;
        writer.perform(CommitStep::WriteRootRecordForceUnitAccess {
            checkpoint_txg: plan.txg,
            root_slot: &root_slot,
        })?;
        writer.perform(CommitStep::RotateSuperblockSlots {
            journal_tail: plan.counter,
            journal_instance: plan.instance,
        })
    };
    if let Err(cause) = persist(pool) {
        pool.count_failed_publish(&writes_before_this_publish);
        return Err(cause);
    }
    Ok(ZeroUnitPublishOutput {
        root,
        record,
        record_bytes,
        writes: pool
            .writes_by_structure_kind
            .since(&writes_before_this_publish),
    })
}
```

### crates/singlefs-core/src/transaction.rs:1562-2144（publish_admitted）

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
    // 中途失败时这次已记的写要交出去（增补 2 第 20b 行）：六步收在一个闭包里，失败在这里记账再把错原样交回——
    // 账是两次快照之差、只在成功路径上取，失败那次落盘的写不交出去就不属于任何一次发布，与设备一层的合计对不上。
    let writes_before_this_publish = pool.writes_by_structure_kind.clone();
    let persist = |writer: &mut PoolWriter<'_, Device>| -> Result<(), BlockDeviceError> {
        for unit in &written_units {
            writer.perform(CommitStep::WriteUnitToEveryDevice {
                slot: unit.slot,
                unit: &unit.bytes,
                identity: unit.identity,
            })?;
        }
        writer.perform(CommitStep::Barrier)?;
        writer.perform(CommitStep::WriteJournalRecordToEveryDevice {
            counter: plan.counter,
            record: &record_bytes,
        })?;
        writer.perform(CommitStep::Barrier)?;
        writer.perform(CommitStep::WriteRootRecordForceUnitAccess {
            checkpoint_txg: txg,
            root_slot: &root_slot,
        })?;
        writer.perform(CommitStep::RotateSuperblockSlots {
            journal_tail: plan.counter,
            journal_instance: instance,
        })
    };
    if let Err(cause) = persist(pool) {
        pool.count_failed_publish(&writes_before_this_publish);
        return Err(PublishError::BlockDevice(cause));
    }

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

### crates/singlefs-core/src/transaction.rs:1126-1181（publish_first_file）

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
                // 改动计数 = 最后一次改动所在发布的 checkpoint_txg（D8（核心索引结构） 已定项 6 偏移 88 的字段表定义）：
                // 第一个事务这次发布的 txg，暖机之后是 3（`FIRST_TRANSACTION_TXG`，上面那句 `let txg` 取的也是它）。
                // 写 1 是 2026-09-18 之前的老样子（暖机把第一个事务从 txg 1 推到 3 时这一格没跟着改，增补 2 第 11 行）。
                // 这里不写 `txg.0`：`crates/mutations.tsv` 第 22 行按那串字面锚在 `publish_overwrite` 上，同一份文件里出现两次它就腐化。
                change_count: FIRST_TRANSACTION_TXG,
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

### crates/singlefs-core/src/allocator.rs:881-885（cluster_segments）

```rust
    /// 这次挂载开过的聚簇段的起点（含已经回落掉、这会儿不 bump 的那些）：用户数据一个都不许落进去。
    #[must_use]
    pub fn cluster_segments(&self) -> &BTreeSet<SlotNumber> {
        &self.cluster_segments
    }
```

### crates/singlefs-core/src/allocator.rs:705-770（try_allocate_commit_generated）

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
                self.cluster_segments.insert(segment_start);
                let slot = self
                    .bump_slot_in_open_segment(segment_start, footprint)
                    .expect("每块盘都答了同一个全空段（已分配、隔离、扣住都是 0，整段在每块盘的单元区里）：64 槽装得下任何一个单元");
                Ok(self.record_bumped(slot, footprint, generation))
            }
            CommitGeneratedDeviceAnswer::FallBackToLowestFreeSlot(slot) => {
                // 回落之后没有段可 bump 了，但刚才那一段仍是聚簇段、仍装着这次提交的内生块：它留在 `cluster_segments` 里，
                // 用户数据照旧不许落进去（D3（空间分配） 已定项 8 第 2 条；增补 2 第 20c 行）。
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

### crates/singlefs-core/src/allocator.rs:393-415（lowest_user_data_slot）

```rust
    /// 用户数据落点：起点槽号最小、偶数槽（32768 对齐）、两槽都空、不在任何一个聚簇段里（D3（空间分配） 已定项 10 ②
    /// 逐字「候选落点不在任何开放的聚簇段里」，量词是「任何」不是「当前那一个」）。
    #[must_use]
    pub fn lowest_user_data_slot(
        &self,
        cluster_segments: &BTreeSet<SlotNumber>,
    ) -> Option<SlotNumber> {
        let mut candidate = UNIT_AREA_START_SLOT;
        let end = UNIT_AREA_START_SLOT + self.unit_area_slots;
        while candidate + 1 < end {
            let inside_cluster_segment = cluster_segments.iter().any(|segment| {
                candidate >= segment.0 && candidate < segment.0 + CLUSTER_SEGMENT_SLOTS
            });
            if !inside_cluster_segment
                && self.is_free(SlotNumber(candidate))
                && self.is_free(SlotNumber(candidate + 1))
            {
                return Some(SlotNumber(candidate));
            }
            candidate += 2;
        }
        None
    }
```

### crates/singlefs-checker/src/walk.rs:1099-1155（judge_release_generation_and_tree_table_birth）

```rust
/// I-3.9 与 I-9.14 共用的这一遍：候选集里每条根各走一遍取引用集合与树表。候选集只剩一条根时两条都报「不适用」——
/// 「最早不再引用它的那条有效根」与「跨根相同」都要第二条根才有内容（2026-09-18 用户定案随 C374 立条时定的口径）。
fn judge_release_generation_and_tree_table_birth(
    reader: &dyn ImageReader,
    roots: &[(u64, u64, crate::RootView)],
    candidate_indexes: &[usize],
    newest_index: usize,
    judgements: &mut Judgements,
) {
    if candidate_indexes.len() < 2 {
        judgements.not_applicable(
            "I-3.9",
            "回退候选集里只有一条根：没有第二条根能见证「不再引用这个落点」",
        );
        judgements.not_applicable(
            "I-9.14",
            "回退候选集里只有一条根：树表条目没有第二条根的那一份可比",
        );
        return;
    }
    let mut cache = IndexNodeCache::new();
    // 最新根那棵账定这一遍走多深：它一条已释放记录都没有时 I-3.9 判不了，别的根只要树表（I-9.14 那一半）。
    let newest = references_of_root(
        reader,
        &roots[newest_index].2.record_bytes,
        ReferenceScanDepth::EveryReferencedPlacement,
        &mut cache,
    );
    let depth = if newest
        .allocation_records
        .iter()
        .any(|record| record.is_released)
    {
        ReferenceScanDepth::EveryReferencedPlacement
    } else {
        ReferenceScanDepth::TreeTableOnly
    };
    let mut scanned: Vec<ScannedCandidateRoot> = Vec::with_capacity(candidate_indexes.len());
    for index in candidate_indexes.iter().copied() {
        scanned.push(ScannedCandidateRoot {
            root_index: index,
            checkpoint_txg: roots[index].2.checkpoint_txg,
            references: if index == newest_index {
                newest.clone()
            } else {
                references_of_root(reader, &roots[index].2.record_bytes, depth, &mut cache)
            },
        });
    }
    scanned.sort_unstable_by_key(|root| root.checkpoint_txg);
    let newest_position = scanned
        .iter()
        .position(|root| root.root_index == newest_index)
        .expect("最新根在回退候选集里（`check_pool_image` 无条件把它放进去）");
    judge_release_generations(&scanned, newest_position, judgements);
    judge_tree_table_birth_txg(&scanned, judgements);
}
```

### crates/singlefs-checker/src/walk.rs:833-896（references_of_root）

```rust
/// 从一条根走一遍，取它的树表条目；`depth` 是「走到叶」时再取它引用的全部落点与分配记录。只读不判。
fn references_of_root(
    reader: &dyn ImageReader,
    record: &[u8],
    depth: ReferenceScanDepth,
    cache: &mut IndexNodeCache,
) -> RootReferences {
    let mut references = RootReferences {
        placements: BTreeSet::new(),
        tree_table_birth_txg: BTreeMap::new(),
        tree_table_placement: None,
        allocation_records: Vec::new(),
        depth,
        is_complete: true,
    };
    let instance_table = parse_node_pointer(&record[170..256]);
    references.note(&instance_table);
    // 中央映射树的根住根记录（D19（块指针的结构与宽度预算） 已定项 11）。映射条目指的单元与树里引用的是同一批，不重复数——
    // 主走读的 `note_reference` 也不数它们（数了 I-3.1（已分配统计对得上） 与 I-5.1（引用不重叠） 会把同一个落点算两遍）。
    let mapping_root = parse_node_pointer(&record[256..342]);
    references.note(&mapping_root);
    let tree_table_pointer = parse_node_pointer(&record[36..122]);
    references.note(&tree_table_pointer);
    if tree_table_pointer.all_zero {
        references.is_complete = false;
        return references;
    }
    references.tree_table_placement = Some((
        tree_table_pointer.locations[0].device,
        tree_table_pointer.locations[0].slot,
    ));
    let Some(tree_table) = read_index_node_without_judging(reader, &tree_table_pointer, cache)
    else {
        references.is_complete = false;
        return references;
    };
    for entry in &tree_table.entries {
        // 条目宽是节点头里的一个字段：头校验和过、载荷 CRC 过而条目宽被改窄的镜像也要能判（那由 I-1.7 / I-1.1 说话），
        // 这一遍只如实记「数不全」，不按短条目去解字段。
        if entry.len() < tree_table_entry_bytes() {
            references.is_complete = false;
            continue;
        }
        let tree = read_u64(entry, 0);
        references
            .tree_table_birth_txg
            .insert(tree, read_u64(entry, TREE_TABLE_ENTRY_BIRTH_TXG_OFFSET));
        let tree_root = parse_node_pointer(&entry[14..100]);
        if tree_root.all_zero {
            // day-1 注册、还没有根节点的树（livelist 6、稀疏旁表 7、deadlist 8）：不占落点。
            continue;
        }
        references.note(&tree_root);
        if depth == ReferenceScanDepth::TreeTableOnly {
            continue;
        }
        let Some(node) = read_index_node_without_judging(reader, &tree_root, cache) else {
            references.is_complete = false;
            continue;
        };
        collect_tree_references(&mut references, &node, read_u16(entry, 10));
    }
    references
}
```

### crates/singlefs-checker/src/walk.rs:1157-1389（check_pool_image）

```rust
/// 池级 checker 的入口：每条第一版不变量都报，没评估到的报「不适用」并带理由。
#[must_use]
pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, InvariantVerdict)> {
    let superblocks = chosen_superblocks(reader);
    let mut root_ring_judgements = Judgements::default();
    let chosen: Vec<_> = superblocks
        .iter()
        .filter_map(|(device, chosen)| {
            chosen
                .clone()
                .map(|(view, geometry)| (*device, view, geometry))
        })
        .collect();
    if chosen.is_empty() || chosen.len() != superblocks.len() {
        for invariant in crate::image::IMPLEMENTED_INVARIANTS {
            root_ring_judgements.not_applicable(
                invariant,
                "有一块盘两个超级块槽都无效：这不是一个挂得上的镜像",
            );
        }
        return root_ring_judgements.into_report();
    }
    let geometry = chosen[0].2;
    // 各盘择到的超级块要属于同一个池：fsid 逐盘相同（一块别的池的旧盘插进来，实例代号可以恰好也是 1，I-7.7 看不出来）。
    for (device, view, _) in &chosen {
        root_ring_judgements.judge(
            "I-1.4",
            view.filesystem_identifier == geometry.filesystem_identifier,
            || {
                format!(
                    "盘 {device} 择到的超级块 fsid 与盘 {} 的不同：这块盘不属于这个池",
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
    let roots = valid_roots(reader, &geometry);
    judge_instance_carriers(reader, &geometry, &roots, &mut root_ring_judgements);
    root_ring_judgements.judge("I-7.1", !roots.is_empty(), || {
        "根环里一条自证过的根都没有".to_string()
    });
    if roots.is_empty() {
        return root_ring_judgements.into_report();
    }
    let newest_index = roots
        .iter()
        .enumerate()
        .max_by_key(|(_, (_, _, root))| (root.checkpoint_txg, root.instance))
        .map(|(index, _)| index)
        .expect("非空");
    let mut walk = Walk {
        reader,
        judgements: root_ring_judgements,
        filesystem_identifier_low: u64::from_le_bytes(
            geometry.filesystem_identifier[..8]
                .try_into()
                .expect("8 字节"),
        ),
        references: BTreeMap::new(),
        visited_units: BTreeSet::new(),
        walk_failures: Vec::new(),
        tree_identifiers_in_tables: BTreeSet::new(),
        inode_object_birth: BTreeMap::new(),
        data_unit_objects: Vec::new(),
        accounting: BTreeMap::new(),
        accounting_seen: false,
        instance_table_rows: Vec::new(),
    };
    // 先走最新的根：它的走读断没断就是 I-7.2；记账行只取最新根下面的。
    walk.walk_root(&roots[newest_index].2.record_bytes, true);
    let newest_failures = walk.walk_failures.clone();
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
    // 再加一条：txg ≥ 最新根带的回退下界 F（D16（发布语义） 已定项 1 的回退候选集）；F 之下的根引用的单元可以已被回收复用，
    // 它们不在当前账里、也不再是「近 K 代」——I-2.1 只在候选集里的根上判。
    let instance_table_rows = walk.instance_table_rows.clone();
    let newest_rollback_floor = u64::from_le_bytes(
        roots[newest_index].2.record_bytes[130..138]
            .try_into()
            .expect("8 字节"),
    );
    let candidate_indexes: Vec<usize> = roots
        .iter()
        .enumerate()
        .filter(|(index, (_, _, root))| {
            let abandoned = instance_table_rows.iter().any(|(row_instance, row_txg)| {
                *row_instance == root.instance && root.checkpoint_txg > *row_txg
            });
            let below_floor = root.checkpoint_txg < newest_rollback_floor;
            *index == newest_index || (!abandoned && !below_floor)
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
    let mut judgements = walk.judgements;
    judgements.judge("I-7.2", newest_failures.is_empty(), || {
        format!("最新的根走不完：{}", newest_failures.join("；"))
    });
    judge_release_generation_and_tree_table_birth(
        reader,
        &roots,
        &candidate_indexes,
        newest_index,
        &mut judgements,
    );
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
    // I-3.1 / I-5.2：最新根下面记账树的「已分配」「空闲」逐盘对遍历得到的和与容量。
    if accounting_seen {
        for device in &devices {
            let walked: u64 = per_device.get(device).map_or(0, |ranges| {
                ranges.iter().map(|(_, span, _)| span * SLOT_BYTES).sum()
            });
            let allocated = accounting
                .get(&(STATISTIC_ALLOCATED_BYTES, *device))
                .copied();
            judgements.judge("I-3.1", allocated == Some(walked), || {
                format!("盘 {device}：记账的已分配 {allocated:?}，遍历全部有效根得到 {walked}")
            });
            let capacity = reader
                .device_bytes(*device)
                .map(|bytes| (bytes / SLOT_BYTES - geometry.unit_area_start_slot) * SLOT_BYTES);
            let free = accounting.get(&(STATISTIC_FREE_BYTES, *device)).copied();
            judgements.judge("I-5.2", matches!((free, allocated, capacity), (Some(free), Some(allocated), Some(capacity)) if free + allocated == capacity), || {
                format!("盘 {device}：空闲 {free:?} + 已分配 {allocated:?} ≠ 单元区 {capacity:?}")
            });
        }
    } else {
        judgements.not_applicable("I-3.1", "最新的根下面还没有记账树（第 0 代树表）");
        judgements.not_applicable("I-5.2", "最新的根下面还没有记账树（第 0 代树表）");
    }
    // I-7.8：根环全部有效根的水位取 max，要大于盘上出现过的最大树 ID（全部码 2 单元头 ∪ 走过的树表条目）。
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
    let mut seen = scanned_tree_identifiers(reader, &geometry, newest_published_txg);
    seen.extend(walk.tree_identifiers_in_tables.iter().copied());
    let highest_seen = seen.iter().max().copied().unwrap_or(0);
    judgements.judge("I-7.8", watermark > highest_seen, || {
        format!("根环水位最大 {watermark}，盘上出现过的最大树 ID {highest_seen}")
    });
    judgements.into_report()
}
```

### crates/singlefs-checker/src/image.rs:35-36（IMPLEMENTED_INVARIANTS）

```rust
/// 第一版 checker 判的不变量，按这个次序报；每次都全部报出来，没评估到的报「不适用」。
pub const IMPLEMENTED_INVARIANTS: [&str; 28] = [
```

### crates/singlefs-harness/src/lib.rs:18-18（first_transaction_regions）

```rust
pub mod first_transaction_regions;
```

### crates/singlefs-harness/src/lib.rs:19-19（hexadecimal）

```rust
pub mod hexadecimal;
```

### crates/singlefs-harness/src/lib.rs:22-22（sha256）

```rust
pub mod sha256;
```

### crates/singlefs-harness/tests/second_transaction_supplement_two_row_publish_admission.rs:331-342（a_writable_mount_whose_first_warm_up_publish_does_not_fit_is_refused_before_the_instance_is_acquired）

```rust
/// 验收（增补 2 第 20a 行，2026-09-18 用户定案的那一半）：写行那次装得下（796 + 10 = 806 ≤ 812）、暖机第 1 次装不下
/// （806 + 8 = 814 > 812）的池——在取号之前拒绝。48 次覆盖写 + 1 次空发布 ⇒ 20 + 768 + 8 = 796 条。
#[test]
fn a_writable_mount_whose_first_warm_up_publish_does_not_fit_is_refused_before_the_instance_is_acquired(
) {
    a_writable_mount_is_refused_before_acquisition_at_a_warm_up_publish(
        "supplement-two-warm-up-admission-first",
        48,
        1,
        1,
    );
}
```

### crates/singlefs-harness/tests/second_transaction_supplement_two_row_publish_admission.rs:344-356（a_writable_mount_whose_second_warm_up_publish_does_not_fit_is_refused_before_the_instance_is_acquired）

```rust
/// 验收（同上）：写行那次与暖机第 1 次都装得下（788 + 10 = 798、798 + 8 = 806 ≤ 812）、暖机第 2 次装不下（806 + 8 = 814 > 812）
/// 的池——在取号之前拒绝。47 次覆盖写 + 2 次空发布 ⇒ 20 + 752 + 16 = 788 条。
/// 这一条才分辨得出「只把第一次暖机算进去」的半截改法：那种改法在这个池上仍然放行，取号与写行都发出去。
#[test]
fn a_writable_mount_whose_second_warm_up_publish_does_not_fit_is_refused_before_the_instance_is_acquired(
) {
    a_writable_mount_is_refused_before_acquisition_at_a_warm_up_publish(
        "supplement-two-warm-up-admission-second",
        47,
        2,
        2,
    );
}
```

### crates/singlefs-harness/tests/second_transaction_supplement_one_write_accounting.rs:512-594（a_publish_that_fails_midway_hands_out_what_it_already_wrote_so_the_retry_still_adds_up）

```rust
/// 增补 1 验收加的那一条（增补 2 第 20b 行，代码三方第一轮打中，判决第二节第 2 行）：一次发布中途设备写报错、调用方重试成功之后，
/// 按种类的合计与设备一层记的仍对得上。
///
/// 一次发布的账是发布前后两次快照之差、两次都在成功路径上取 ⇒ 中途任一步返回时已落盘的写不属于任何一次发布的账；
/// 不把它交出去，这一段窗口里设备一层数到的写就比按种类的合计多（攻方探针：按种类 21 次 / 344 576，设备 25 次 / 442 880）。
/// 这里让盘 1 的第 6 次写报错（发布 C 写第六个单元时）：失败那次已记 11 次写、245 760 字节（数据单元、extent 树根、inode 树叶容器、
/// inode 树根、分配记录树节点两盘各一份 + 记账树节点盘 0 那一份），重试整整 21 次 / 344 576 字节，两份相加正好是录制器在这段里记下的。
#[test]
fn a_publish_that_fails_midway_hands_out_what_it_already_wrote_so_the_retry_still_adds_up() {
    let mut published = publish_through_overwrite();
    let parameters = e142_parameters(512, 512);
    let window_start = published.stream.operations().len();
    let content = third_file_content();
    // 盘 1 再放行 5 次写：发布 C 的第六个单元写到盘 1 时报错（单元按 bump 次序写，每个单元两盘各一次）。
    published.write_faults[1].set(Some(5));
    let (failed_publishes, retry) = {
        let mut writer = PoolWriter::new(&parameters, published.devices.as_mut_slice());
        let failure = publish_overwrite(
            &mut writer,
            &mut published.allocator,
            &published.overwrite,
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 120,
            },
            InstanceGeneration(1),
        )
        .expect_err("盘 1 的第 6 次写报错，发布必须失败");
        assert!(
            matches!(
                failure,
                singlefs_core::transaction::PublishError::BlockDevice(_)
            ),
            "中途失败的是设备写：{failure:?}"
        );
        published.write_faults[1].set(None);
        let retry = publish_overwrite(
            &mut writer,
            &mut published.allocator,
            &published.overwrite,
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 120,
            },
            InstanceGeneration(1),
        )
        .expect("盘好了，拿同一个上一版重试");
        (writer.writes_of_failed_publishes().to_vec(), retry)
    };
    let operations = published.stream.operations();

    assert_eq!(failed_publishes.len(), 1, "中途失败过一次，交出一份账");
    assert_eq!(
        failed_publishes[0].total(),
        calls_and_bytes(11, 245_760),
        "失败那次已记的写：五个单元两盘各一份 + 第六个单元盘 0 那一份"
    );
    every_kind_matches(
        "中途失败的发布 C",
        &failed_publishes[0],
        &failed_midway_publish_by_kind(),
    );
    assert_eq!(
        retry.writes.total(),
        calls_and_bytes(21, 344_576),
        "重试那次是完整的一次发布"
    );
    every_kind_matches(
        "重试的发布 C",
        &retry.writes,
        &file_version_publish_by_kind(),
    );
    assert_eq!(
        failed_publishes[0].total().plus(retry.writes.total()),
        recorded_writes(&operations[window_start..]),
        "失败那次已记的写 + 重试那次的账 == 录制器在这段窗口里记下的写"
    );
    assert_eq!(
        recorded_writes(&operations[window_start..]),
        calls_and_bytes(32, 590_336),
        "设备一层：11 + 21 次、245 760 + 344 576 字节"
    );
}
```

### crates/singlefs-harness/tests/checker_known_bad_images.rs:684-711（each_c374_bad_image_reddens_only_its_own_invariant_after_the_overwrite）

```rust
/// C374 那两条：发布 B 之后的干净镜像上每条不变量都成立，两份坏镜像各自**只**红在自己那一条上
/// （别的不变量跟着红就说明镜像改宽了，判别力算不到这一条头上）。
#[test]
fn each_c374_bad_image_reddens_only_its_own_invariant_after_the_overwrite() {
    let clean = image_after_the_overwrite("known-bad-after-overwrite");
    for (invariant, found) in check_pool_image(&clean) {
        assert_eq!(
            found,
            InvariantVerdict::Holds,
            "发布 B 之后的干净镜像上 {invariant} 要真被评估过且成立"
        );
    }
    for (invariant, mutation) in known_bad_images_after_the_overwrite() {
        let mut image = clean.clone();
        mutation(&mut image);
        let verdicts = check_pool_image(&image);
        let violated: Vec<&str> = verdicts
            .iter()
            .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
            .map(|(name, _)| *name)
            .collect();
        assert_eq!(
            violated,
            [invariant],
            "判红的该只有 {invariant}：{verdicts:?}"
        );
    }
}
```

### crates/singlefs-harness/tests/first_transaction_step_five_publish.rs:626-748（inode_and_extent_lookups_from_the_root_read_the_first_file_back）

```rust
#[test]
fn inode_and_extent_lookups_from_the_root_read_the_first_file_back() {
    let pool = build_pool("lookup");
    let tree_table_entries = tree_table_entries_from_disk(&pool);
    // inode 树：根（层级 1）→ 内部条目（分隔 key = ino）→ 叶容器 → 140 字节记录。
    let inode_entry = tree_table_entries
        .iter()
        .find(|entry| entry.tree.0 == TREE_IDENTIFIER_INODE)
        .expect("inode 树在树表里");
    let inode_root_location = inode_entry.root.locations[1];
    let inode_root_unit = read_unit_from(
        &pool,
        inode_root_location.device,
        inode_root_location.slot,
        16384,
    );
    assert_eq!(
        crc32_castagnoli_bitwise(&inode_root_unit),
        inode_root_location.unit_checksum
    );
    let inode_root = index_node_view(&inode_root_unit).expect("inode 根");
    assert_eq!(inode_root.entries.len(), 1);
    let mut reader = ByteReader::at(&inode_root.entries[0], 0);
    assert_eq!(reader.get_u64(), FIRST_INODE_NUMBER, "分隔 key");
    assert_eq!(
        (
            reader.get_u64(),
            reader.get_u16(),
            reader.get_u64(),
            reader.get_u64()
        ),
        (12, 2, 1, 3),
        "身份引用四元组"
    );
    let leaf_pointer = NodePointer::read_from(&mut reader);
    let leaf_unit = read_unit_from(
        &pool,
        leaf_pointer.locations[0].device,
        leaf_pointer.locations[0].slot,
        32768,
    );
    assert_eq!(
        crc32_castagnoli_bitwise(&leaf_unit),
        leaf_pointer.locations[0].unit_checksum
    );
    let leaf = packed_unit_view(&leaf_unit).expect("叶容器");
    assert_eq!(
        (
            leaf.birth_tree,
            leaf.record_type,
            leaf.container,
            leaf.container_birth,
            leaf.record_width
        ),
        (12, 2, 1, 3, 140)
    );
    assert_eq!(leaf.records.len(), 1);
    let inode_record = InodeRecord::parse(&leaf.records[0]).expect("inode 记录");
    assert_eq!(
        inode_record,
        InodeRecord {
            inode: FIRST_INODE_NUMBER,
            object_birth: CheckpointTxg(3),
            size: 3000,
            // 改动计数 = 最后一次改动所在发布的 checkpoint_txg（D8（核心索引结构） 已定项 6 偏移 88）：
            // 第一个事务发布在 txg 3（增补 2 第 11 行，2026-09-18 用户定案；此前钉的 1 是暖机把 txg 推到 3 之前的值）。
            change_count: 3,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS
        },
        "逐字段回读等于写入"
    );
    // extent 树：根兼叶 → key (0, ino, 0) → 指针 88 → 数据单元 → 载荷前 size 字节。
    let extent_entry = tree_table_entries
        .iter()
        .find(|entry| entry.tree.0 == TREE_IDENTIFIER_EXTENT)
        .expect("extent 树在树表里");
    let extent_unit = read_unit_from(
        &pool,
        extent_entry.root.locations[0].device,
        extent_entry.root.locations[0].slot,
        16384,
    );
    let extent_root = index_node_view(&extent_unit).expect("extent 根兼叶");
    assert_eq!(
        (
            extent_root.level,
            extent_root.entries.len(),
            extent_root.entry_width
        ),
        (0, 1, 112)
    );
    let mut extent_reader = ByteReader::at(&extent_root.entries[0], 0);
    assert_eq!(
        (
            extent_reader.get_u64(),
            extent_reader.get_u64(),
            extent_reader.get_u64()
        ),
        (0, FIRST_INODE_NUMBER, 0),
        "extent key"
    );
    let pointer = DataPointer::read_from(&mut extent_reader);
    assert_eq!(pointer, pool.output.data_pointer);
    assert_eq!(
        (
            pointer.head.birth_tree.0,
            pointer.head.birth_txg.0,
            pointer.write_order.instance.0,
            pointer.write_order.transaction
        ),
        (11, 3, 1, 1)
    );
    for location in pointer.locations {
        let data_unit = read_unit_from(&pool, location.device, location.slot, 32768);
        assert_eq!(crc32_castagnoli_bitwise(&data_unit), location.unit_checksum);
        assert_eq!(check_unit(&data_unit).expect("数据单元"), 1);
        assert_eq!(
            &data_unit[134..134 + usize::try_from(inode_record.size).expect("size")],
            &file_content()[..],
            "读回内容逐字节相同"
        );
    }
}
```

### crates/singlefs-harness/tests/second_transaction_step_five_reuse.rs:483-518（reclaiming_without_raising_the_floor_reuses_a_slot_a_candidate_root_still_references_and_the_checker_goes_red）

```rust
/// 必红（C22（刚释放的块立即重分配）、复用窗口置 0）：不抬 F、直接把释放代 ≤ 11 的落点回收，E 落回 A 的数据落点 50176——
/// F = 0 时 A（txg 3）还是候选，影子账按窄读法豁免它引用的槽、没隔离 50176；checker 走 A 时那片数据的校验和对不上 ⇒ I-2.1 红。
#[test]
fn reclaiming_without_raising_the_floor_reuses_a_slot_a_candidate_root_still_references_and_the_checker_goes_red(
) {
    let mut pool = build_through_rollback("step-five-window-zero");
    four_overwrites_after_the_rollback(&mut pool);
    let reclaimed = pool
        .allocator
        .reclaim_released_up_to(CheckpointTxg(11), ReclaimedReuse::Immediately);
    assert_eq!(reclaimed.len(), 18);
    let reuse = overwrite_in_process(&mut pool, &content_of(2000, 31), InstanceGeneration(3));
    assert_eq!(
        reuse.data_pointer.locations[0].slot,
        SlotNumber(50176),
        "A 的数据落点被拿走：A（F = 0 时仍是候选）还引用它，窄读法没隔离它"
    );
    assert_eq!(newest_root_floor(&pool), CheckpointTxg(0), "F 没抬");
    let verdicts = check_pool_image(&pool.memory_pool());
    let violated: Vec<&str> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(name, _)| *name)
        .collect();
    assert!(
        violated.contains(&"I-2.1"),
        "A 的根还是候选，它的数据单元被盖了：{verdicts:?}"
    );
    // C22（刚释放的块立即重分配） 的「怎么拦」第 ① 条点的是 I-4.8（近 K 代根校验和自洽）：从最近 K 代任一根遍历，校验和必须全对。
    // 只断言 I-2.1 红，拦的是「有一个单元的校验和与父指针对不上」，没说那是从一条**候选根**出发的遍历走进去的
    // （收口表第 22 行 2026-09-17 逐句核出来的那一句）。
    assert!(
        violated.contains(&"I-4.8"),
        "复用窗口置 0 之后 I-4.8 必须变红：不红说明这条约束根本没被检查（C22（刚释放的块立即重分配） 第 ② 条）：{verdicts:?}"
    );
}
```

## 二、全新文件全文（本轮新增模块，无旧版本可比，整份抄入；均回读 sha256 与开工快照一致）

### crates/singlefs-harness/src/first_transaction_regions.rs（308 行，sha256 457e3fed…8ee7）

```rust
//! 第一个事务写到的那 21 个区域，以及把它们的字节打成结果行的只读导出口。
//!
//! 这张表是 E142（第一个事务的干跑） 第十一次跑的跑前登记（`research/prompts/e142-r11-prereg.md` 第一节第 3 条）
//! 写死的那份区域清单，逐行抄自 `.claude/kb/layout/01-first-txn.md` 零那一节的写清单：
//! 第一个事务写到的 8 个单元的落点（t1..t8，两盘各一份 = 16 行）、jsn (1, 3) 那条 journal 记录的两个落点（t9）、
//! 这次发布的根槽（t10，区域 0 槽 1，只落在根环区域 0 那块盘）、这次发布写的超级块槽（t11，两盘各一份）
//! ⇒ 16 + 2 + 1 + 2 = **21**，与那一节「⇒ 写请求数 … 21 条」逐项对得上。
//!
//! 这张表只对 E142 那套几何成立（两盘、`physical_block_size` = 512、io_min = 512 ⇒ 固定结构槽距 4096、根槽宽 512），
//! 换几何要连表一起改：`region_table_against_writes` 就是把它钉在实装身上的那道检查。
//!
//! **只读**：本模块与 [`crate::scenario`] 之外的写路径一个字节都不碰（量 5 要的是「实装写出来的字节」，不是「另写一遍」）。
//! 装置（`research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs`）与这里是两份代码，不共用，
//! 对不上时两边都查（`.claude/rules/implementation-first.md` 第 4 条）。

use std::collections::BTreeMap;

use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, SlotNumber};
use singlefs_format::{
    JOURNAL_RECORD_BYTES, JOURNAL_RING_START_SLOT, ROOT_RING_BASE_SLOT, ROOT_RING_CHUNK_BYTES,
    ROOT_RING_PRIME_STEP, SLOT_BYTES, SUPERBLOCK_SLOT_BYTES,
};

use crate::crash::MemoryPool;
use crate::hexadecimal::hexadecimal_text;
use crate::sha256::sha256_hexadecimal;
use crate::{RecordedOperation, RecordedOperationKind};

/// 区域清单的行数：登记第一节第 3 条写死的 21 行。
pub const FIRST_TRANSACTION_REGION_COUNT: usize = 21;

/// 长度大的区域只打前后各这么多字节的十六进制（整段太长，逐字节比对靠 sha256）。
pub const HEAD_AND_TAIL_BYTES: usize = 32;

/// E142 几何的固定结构槽距：io_min = 512 ⇒ max(4096, 512) = 4096（D2（RAID 条带策略） 已定项 19）。
const E142_FIXED_STRUCTURE_SLOT_SPACING_BYTES: u64 = 4096;
/// E142 几何的根槽宽 = 探到的 `physical_block_size` = 512（字节表零那一节的 m3「371 / 512 槽」）。
const E142_ROOT_SLOT_BYTES: u64 = 512;
/// 这次发布写的超级块槽是槽 1（t11：世代号 5、tail = 3，根槽之后再更新）。
const FIRST_TRANSACTION_SUPERBLOCK_SLOT_INDEX: u64 = 1;
/// 这次发布的根槽：区域 `3 mod 3` = 0 的槽 `(3 div 3) mod 8` = 1（t10）。
const FIRST_TRANSACTION_ROOT_RING_REGION: u64 = 0;
const FIRST_TRANSACTION_ROOT_RING_SLOT_INDEX: u64 = 1;
/// 根环区域 0 落在哪块盘：mkfs 的 `region_devices` 第一项（E142 取 `[0, 1, 0]`）。
const FIRST_TRANSACTION_ROOT_RING_REGION_DEVICE: u32 = 0;
/// jsn (1, 3) 那条记录在环内的偏移：两次暖机各占一条 4096（w1 在 0、w4 在 4096）⇒ 第一个事务这条在 8192。
const FIRST_TRANSACTION_JOURNAL_RECORD_RING_OFFSET: u64 = 2 * JOURNAL_RECORD_BYTES;

/// t1..t8 的落点槽号（字节表零那一节的写清单）。
const DATA_UNIT_SLOT: u64 = 50180;
const EXTENT_ROOT_SLOT: u64 = 50240;
const INODE_LEAF_SLOT: u64 = 50242;
const INODE_ROOT_SLOT: u64 = 50244;
const ALLOCATION_ROOT_SLOT: u64 = 50245;
const ACCOUNTING_ROOT_SLOT: u64 = 50246;
const MAPPING_ROOT_SLOT: u64 = 50247;
const TREE_TABLE_SLOT: u64 = 50248;

/// 十六进制打多少：整段照打，还是只打前后各 [`HEAD_AND_TAIL_BYTES`] 字节。封闭集合，`match` 不写通配臂。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HexadecimalExtent {
    /// 整段十六进制照打（固定结构那 5 行：根槽 512、journal 记录 4096 两份、超级块槽 4096 两份）。
    WholeRegion,
    /// 只打 sha256 与前后各 32 字节（16 KiB / 32 KiB 的单元那 16 行）。
    HeadAndTail,
}

impl HexadecimalExtent {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            HexadecimalExtent::WholeRegion => "whole_region",
            HexadecimalExtent::HeadAndTail => "head_and_tail",
        }
    }
}

/// 区域清单的一行。`name` 与 E142 装置里同一个结构的标签同名，量 5 按 `region=` 加 `device=` 两边配对。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FirstTransactionRegion {
    pub name: &'static str,
    pub device: DeviceIdentity,
    pub offset: DeviceOffsetInBytes,
    pub length_in_bytes: u64,
    pub hexadecimal_extent: HexadecimalExtent,
}

const fn unit_region(
    name: &'static str,
    device_number: u32,
    slot: u64,
    span_in_slots: u64,
) -> FirstTransactionRegion {
    FirstTransactionRegion {
        name,
        device: DeviceIdentity(device_number),
        offset: SlotNumber(slot).to_device_offset(),
        length_in_bytes: span_in_slots * SLOT_BYTES,
        hexadecimal_extent: HexadecimalExtent::HeadAndTail,
    }
}

const fn fixed_structure_region(
    name: &'static str,
    device_number: u32,
    offset_in_bytes: u64,
    length_in_bytes: u64,
) -> FirstTransactionRegion {
    FirstTransactionRegion {
        name,
        device: DeviceIdentity(device_number),
        offset: DeviceOffsetInBytes(offset_in_bytes),
        length_in_bytes,
        hexadecimal_extent: HexadecimalExtent::WholeRegion,
    }
}

const FIRST_TRANSACTION_ROOT_SLOT_OFFSET: u64 =
    SlotNumber(ROOT_RING_BASE_SLOT).to_device_offset().0
        + FIRST_TRANSACTION_ROOT_RING_REGION * ROOT_RING_PRIME_STEP * ROOT_RING_CHUNK_BYTES
        + FIRST_TRANSACTION_ROOT_RING_SLOT_INDEX * E142_FIXED_STRUCTURE_SLOT_SPACING_BYTES;

const FIRST_TRANSACTION_JOURNAL_RECORD_OFFSET: u64 =
    SlotNumber(JOURNAL_RING_START_SLOT).to_device_offset().0
        + FIRST_TRANSACTION_JOURNAL_RECORD_RING_OFFSET;

const FIRST_TRANSACTION_SUPERBLOCK_SLOT_OFFSET: u64 =
    FIRST_TRANSACTION_SUPERBLOCK_SLOT_INDEX * E142_FIXED_STRUCTURE_SLOT_SPACING_BYTES;

/// 登记里那 21 行，顺序就是结果行的顺序：8 个单元各两盘（t1..t8）、根记录（t10）、journal 记录两盘（t9）、超级块槽两盘（t11）。
/// 前 16 行是 16 KiB / 32 KiB 的单元，只打 sha256 与前后 32 字节；后 5 行（根槽、journal 记录两份、超级块槽两份）整段十六进制照打。
pub const FIRST_TRANSACTION_REGIONS: [FirstTransactionRegion; FIRST_TRANSACTION_REGION_COUNT] = [
    unit_region("data_unit", 0, DATA_UNIT_SLOT, 2),
    unit_region("data_unit", 1, DATA_UNIT_SLOT, 2),
    unit_region("extent_root", 0, EXTENT_ROOT_SLOT, 1),
    unit_region("extent_root", 1, EXTENT_ROOT_SLOT, 1),
    unit_region("inode_leaf", 0, INODE_LEAF_SLOT, 2),
    unit_region("inode_leaf", 1, INODE_LEAF_SLOT, 2),
    unit_region("inode_root", 0, INODE_ROOT_SLOT, 1),
    unit_region("inode_root", 1, INODE_ROOT_SLOT, 1),
    unit_region("allocation_root", 0, ALLOCATION_ROOT_SLOT, 1),
    unit_region("allocation_root", 1, ALLOCATION_ROOT_SLOT, 1),
    unit_region("accounting_root", 0, ACCOUNTING_ROOT_SLOT, 1),
    unit_region("accounting_root", 1, ACCOUNTING_ROOT_SLOT, 1),
    unit_region("mapping_root", 0, MAPPING_ROOT_SLOT, 1),
    unit_region("mapping_root", 1, MAPPING_ROOT_SLOT, 1),
    unit_region("tree_table", 0, TREE_TABLE_SLOT, 1),
    unit_region("tree_table", 1, TREE_TABLE_SLOT, 1),
    fixed_structure_region(
        "root_record",
        FIRST_TRANSACTION_ROOT_RING_REGION_DEVICE,
        FIRST_TRANSACTION_ROOT_SLOT_OFFSET,
        E142_ROOT_SLOT_BYTES,
    ),
    fixed_structure_region(
        "journal_record",
        0,
        FIRST_TRANSACTION_JOURNAL_RECORD_OFFSET,
        JOURNAL_RECORD_BYTES,
    ),
    fixed_structure_region(
        "journal_record",
        1,
        FIRST_TRANSACTION_JOURNAL_RECORD_OFFSET,
        JOURNAL_RECORD_BYTES,
    ),
    fixed_structure_region(
        "superblock",
        0,
        FIRST_TRANSACTION_SUPERBLOCK_SLOT_OFFSET,
        SUPERBLOCK_SLOT_BYTES,
    ),
    fixed_structure_region(
        "superblock",
        1,
        FIRST_TRANSACTION_SUPERBLOCK_SLOT_OFFSET,
        SUPERBLOCK_SLOT_BYTES,
    ),
];

/// 一个区域此刻在镜像上的字节。区域落在池外（盘不够大、表写错）是 bug，不带着坏坐标往下算。
#[must_use]
pub fn region_bytes(image: &MemoryPool, region: &FirstTransactionRegion) -> Vec<u8> {
    let end = region
        .offset
        .0
        .checked_add(region.length_in_bytes)
        .expect("区域末端放得进 u64");
    assert!(
        end <= image.device_size_in_bytes,
        "区域 {} 落在盘外：末端 {end} > 盘 {} 字节",
        region.name,
        image.device_size_in_bytes
    );
    let device_image = image
        .devices
        .get(&region.device)
        .expect("区域清单里的设备身份都是这个池里的盘（E142 两盘 0 / 1）");
    device_image.read(
        region.offset,
        usize::try_from(region.length_in_bytes).expect("区域长度放得进 usize"),
    )
}

/// 一个区域一行结果行：设备、绝对偏移、长度、内容的 sha256，再按 `hexadecimal_extent` 打十六进制。
#[must_use]
pub fn region_result_line(image: &MemoryPool, region: &FirstTransactionRegion) -> String {
    let bytes = region_bytes(image, region);
    let head = format!(
        "name=impl_region_bytes region={} device={} offset={} length={} sha256={} hexadecimal_extent={}",
        region.name,
        region.device.0,
        region.offset.0,
        region.length_in_bytes,
        sha256_hexadecimal(&bytes),
        region.hexadecimal_extent.name()
    );
    match region.hexadecimal_extent {
        HexadecimalExtent::WholeRegion => {
            format!("{head} hexadecimal={}", hexadecimal_text(&bytes))
        }
        HexadecimalExtent::HeadAndTail => {
            assert!(
                bytes.len() >= HEAD_AND_TAIL_BYTES * 2,
                "只打前后各 {HEAD_AND_TAIL_BYTES} 字节的区域，长度至少是它的两倍：{} 只有 {} 字节",
                region.name,
                bytes.len()
            );
            format!(
                "{head} head_and_tail_bytes={HEAD_AND_TAIL_BYTES} head_hexadecimal={} tail_hexadecimal={}",
                hexadecimal_text(&bytes[..HEAD_AND_TAIL_BYTES]),
                hexadecimal_text(&bytes[bytes.len() - HEAD_AND_TAIL_BYTES..])
            )
        }
    }
}

/// 整张表的结果行，顺序照表。
#[must_use]
pub fn region_result_lines(image: &MemoryPool) -> Vec<String> {
    FIRST_TRANSACTION_REGIONS
        .iter()
        .map(|region| region_result_line(image, region))
        .collect()
}

/// 表与「第一个事务真正发出的写」对得上吗：两边都是 (设备, 偏移, 长度) 的多重集，一一配对，剩下的两边各自列出来。
/// 表是从登记抄来的常量，实装是另一份代码——这道检查不让它们悄悄分叉（分叉时量 5 比的就不是同一批字节了）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegionTableAgainstWrites {
    /// 第一个事务发出的写调用数（屏障不算）。
    pub write_calls: usize,
    /// 表里有、这一趟没写到的行。
    pub regions_without_a_write: Vec<&'static str>,
    /// 写到了、表里没有的落点。
    pub writes_outside_the_table: Vec<(DeviceIdentity, DeviceOffsetInBytes, u64)>,
}

impl RegionTableAgainstWrites {
    #[must_use]
    pub fn matches(&self) -> bool {
        self.write_calls == FIRST_TRANSACTION_REGION_COUNT
            && self.regions_without_a_write.is_empty()
            && self.writes_outside_the_table.is_empty()
    }
}

/// `first_transaction_operations` 是录制流里第一个事务那一段（暖机之后的全部步骤，屏障在内）。
#[must_use]
pub fn region_table_against_writes(
    first_transaction_operations: &[RecordedOperation],
) -> RegionTableAgainstWrites {
    let mut unmatched_regions: BTreeMap<
        (DeviceIdentity, DeviceOffsetInBytes, u64),
        Vec<&'static str>,
    > = BTreeMap::new();
    for region in &FIRST_TRANSACTION_REGIONS {
        unmatched_regions
            .entry((region.device, region.offset, region.length_in_bytes))
            .or_default()
            .push(region.name);
    }
    let mut write_calls = 0;
    let mut writes_outside_the_table = Vec::new();
    for operation in first_transaction_operations {
        match operation.kind {
            RecordedOperationKind::Barrier => continue,
            RecordedOperationKind::Write | RecordedOperationKind::WriteForceUnitAccess => {
                write_calls += 1;
            }
        }
        let key = (operation.device, operation.offset, operation.length);
        match unmatched_regions.get_mut(&key) {
            Some(names) => {
                names.pop();
                if names.is_empty() {
                    unmatched_regions.remove(&key);
                }
            }
            None => writes_outside_the_table.push(key),
        }
    }
    RegionTableAgainstWrites {
        write_calls,
        regions_without_a_write: unmatched_regions.into_values().flatten().collect(),
        writes_outside_the_table,
    }
}
```

### crates/singlefs-harness/src/sha256.rs（234 行，sha256 d6ebfa14…d65b）

```rust
//! SHA-256（FIPS 180-4）本地实现：E142（第一个事务的干跑） 量 5 的区域摘要要能在仓外用 `sha256sum` 独立复算，
//! 所以摘要必须是标准 SHA-256——本模块只做摘要，不参与任何写路径。
//! 仓里另有一个 [`crate::fnv1a_64`]，那个只认「同一份内容」、不是标准摘要，仓外核不了。
//! 本地实现的理由与录制器同一条（`lib.rs` 的模块注释）：这个工作区不引第三方 crate。
//!
//! 名字里的 `sha256` 是 FIPS 180-4 给这个算法起的名字，不是我们的缩写。

use crate::hexadecimal::hexadecimal_text;

/// FIPS 180-4 第 4.2.2 节的 64 个轮常量（前 64 个素数立方根小数部分的前 32 位）。
const ROUND_CONSTANTS: [u32; 64] = [
    0x428a_2f98,
    0x7137_4491,
    0xb5c0_fbcf,
    0xe9b5_dba5,
    0x3956_c25b,
    0x59f1_11f1,
    0x923f_82a4,
    0xab1c_5ed5,
    0xd807_aa98,
    0x1283_5b01,
    0x2431_85be,
    0x550c_7dc3,
    0x72be_5d74,
    0x80de_b1fe,
    0x9bdc_06a7,
    0xc19b_f174,
    0xe49b_69c1,
    0xefbe_4786,
    0x0fc1_9dc6,
    0x240c_a1cc,
    0x2de9_2c6f,
    0x4a74_84aa,
    0x5cb0_a9dc,
    0x76f9_88da,
    0x983e_5152,
    0xa831_c66d,
    0xb003_27c8,
    0xbf59_7fc7,
    0xc6e0_0bf3,
    0xd5a7_9147,
    0x06ca_6351,
    0x1429_2967,
    0x27b7_0a85,
    0x2e1b_2138,
    0x4d2c_6dfc,
    0x5338_0d13,
    0x650a_7354,
    0x766a_0abb,
    0x81c2_c92e,
    0x9272_2c85,
    0xa2bf_e8a1,
    0xa81a_664b,
    0xc24b_8b70,
    0xc76c_51a3,
    0xd192_e819,
    0xd699_0624,
    0xf40e_3585,
    0x106a_a070,
    0x19a4_c116,
    0x1e37_6c08,
    0x2748_774c,
    0x34b0_bcb5,
    0x391c_0cb3,
    0x4ed8_aa4a,
    0x5b9c_ca4f,
    0x682e_6ff3,
    0x748f_82ee,
    0x78a5_636f,
    0x84c8_7814,
    0x8cc7_0208,
    0x90be_fffa,
    0xa450_6ceb,
    0xbef9_a3f7,
    0xc671_78f2,
];

/// FIPS 180-4 第 5.3.3 节的初始摘要（前 8 个素数平方根小数部分的前 32 位）。
const INITIAL_HASH_VALUES: [u32; 8] = [
    0x6a09_e667,
    0xbb67_ae85,
    0x3c6e_f372,
    0xa54f_f53a,
    0x510e_527f,
    0x9b05_688c,
    0x1f83_d9ab,
    0x5be0_cd19,
];

/// 压缩函数一次吃 512 位。
const BLOCK_BYTES: usize = 64;
/// 尾部那个「消息位数」字段 64 位。
const MESSAGE_LENGTH_FIELD_BYTES: usize = 8;
/// 摘要 256 位。
pub const DIGEST_BYTES: usize = 32;

/// 一段字节的 SHA-256 摘要。
#[must_use]
pub fn sha256_digest(message: &[u8]) -> [u8; DIGEST_BYTES] {
    let mut hash_values = INITIAL_HASH_VALUES;
    let padded = padded_message(message);
    let (blocks, leftover) = padded.as_chunks::<BLOCK_BYTES>();
    assert!(
        leftover.is_empty(),
        "补白之后一定是整块：补白只会把长度补到 512 位的整数倍"
    );
    for block in blocks {
        compress_one_block(&mut hash_values, block);
    }
    let mut digest = [0u8; DIGEST_BYTES];
    for (word_index, word) in hash_values.iter().enumerate() {
        digest[word_index * 4..word_index * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    digest
}

/// 一段字节的 SHA-256 摘要，小写十六进制（与 `sha256sum` 打出来的那 64 个字符逐字相同）。
#[must_use]
pub fn sha256_hexadecimal(message: &[u8]) -> String {
    hexadecimal_text(&sha256_digest(message))
}

/// FIPS 180-4 第 5.1.1 节的补白：接一个 0x80，补 0 到离块尾还差 8 字节，末尾写消息的位数（大端）。
fn padded_message(message: &[u8]) -> Vec<u8> {
    let message_length_in_bits = u64::try_from(message.len())
        .expect("消息字节数放得进 u64")
        .checked_mul(8)
        .expect("消息位数放得进 u64：FIPS 180-4 的上界是 2^64 位");
    let mut padded = Vec::with_capacity(message.len() + BLOCK_BYTES * 2);
    padded.extend_from_slice(message);
    padded.push(0x80);
    while !(padded.len() + MESSAGE_LENGTH_FIELD_BYTES).is_multiple_of(BLOCK_BYTES) {
        padded.push(0);
    }
    padded.extend_from_slice(&message_length_in_bits.to_be_bytes());
    padded
}

/// FIPS 180-4 第 6.2.2 节的一轮压缩。`working_variables` 的下标 0..=7 就是规范里的 a..h——
/// 那八个名字是规范定的单字母，本项目不另起名字，靠下标与这条注释对上。
fn compress_one_block(hash_values: &mut [u32; 8], block: &[u8; BLOCK_BYTES]) {
    let mut message_schedule = [0u32; 64];
    let (four_byte_words, leftover) = block.as_chunks::<4>();
    assert!(leftover.is_empty(), "512 位块正好切成 16 个 32 位字");
    for (word_index, four_bytes) in four_byte_words.iter().enumerate() {
        message_schedule[word_index] = u32::from_be_bytes(*four_bytes);
    }
    for word_index in 16..64 {
        let fifteen_words_back = message_schedule[word_index - 15];
        let two_words_back = message_schedule[word_index - 2];
        let small_sigma_zero = fifteen_words_back.rotate_right(7)
            ^ fifteen_words_back.rotate_right(18)
            ^ (fifteen_words_back >> 3);
        let small_sigma_one = two_words_back.rotate_right(17)
            ^ two_words_back.rotate_right(19)
            ^ (two_words_back >> 10);
        message_schedule[word_index] = message_schedule[word_index - 16]
            .wrapping_add(small_sigma_zero)
            .wrapping_add(message_schedule[word_index - 7])
            .wrapping_add(small_sigma_one);
    }
    let mut working_variables = *hash_values;
    for round in 0..64 {
        let big_sigma_one = working_variables[4].rotate_right(6)
            ^ working_variables[4].rotate_right(11)
            ^ working_variables[4].rotate_right(25);
        let choose = (working_variables[4] & working_variables[5])
            ^ (!working_variables[4] & working_variables[6]);
        let first_temporary = working_variables[7]
            .wrapping_add(big_sigma_one)
            .wrapping_add(choose)
            .wrapping_add(ROUND_CONSTANTS[round])
            .wrapping_add(message_schedule[round]);
        let big_sigma_zero = working_variables[0].rotate_right(2)
            ^ working_variables[0].rotate_right(13)
            ^ working_variables[0].rotate_right(22);
        let majority = (working_variables[0] & working_variables[1])
            ^ (working_variables[0] & working_variables[2])
            ^ (working_variables[1] & working_variables[2]);
        let second_temporary = big_sigma_zero.wrapping_add(majority);
        working_variables = [
            first_temporary.wrapping_add(second_temporary),
            working_variables[0],
            working_variables[1],
            working_variables[2],
            working_variables[3].wrapping_add(first_temporary),
            working_variables[4],
            working_variables[5],
            working_variables[6],
        ];
    }
    for (hash_value, working_variable) in hash_values.iter_mut().zip(working_variables) {
        *hash_value = hash_value.wrapping_add(working_variable);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 公开的测试向量（FIPS 180-2 附录 B 的三条 + 长消息那条），值与 `sha256sum` 打出来的逐字相同——
    /// 摘要算错时这里先红，量 5 的区域摘要就不会拿一个自洽却非标准的哈希去比。
    #[test]
    fn sha256_matches_the_published_test_vectors() {
        assert_eq!(
            sha256_hexadecimal(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_hexadecimal(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            sha256_hexadecimal(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
            "56 字节：补白之后正好跨两块，长度字段单独占一块"
        );
        assert_eq!(
            sha256_hexadecimal(&vec![b'a'; 1_000_000]),
            "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0",
            "一百万个 a：长消息那条向量，位数字段超过 32 位"
        );
    }

    /// 区域摘要要认得出「差一位」：32 KiB 的单元里翻一位，摘要必须变。
    #[test]
    fn flipping_one_bit_of_a_data_unit_sized_message_changes_the_digest() {
        let message = vec![0x5au8; 32768];
        let mut flipped = message.clone();
        let flipped_index = 823 % flipped.len();
        flipped[flipped_index] ^= 0x01;
        assert_ne!(sha256_hexadecimal(&message), sha256_hexadecimal(&flipped));
    }
}
```

### crates/singlefs-harness/src/hexadecimal.rs（26 行，sha256 6e7f7394…5253）

```rust
//! 字节串写成小写十六进制文本。E142（第一个事务的干跑） 第十一次跑的跑前登记第一节第 4 条：
//! 值一律报成小端十六进制字节串，不报十进制——免得「8 字节字段」与「变了几个字节」混成一句。

use std::fmt::Write as _;

/// 一段字节按盘上的先后写成小写十六进制，两个字符一个字节，中间不插分隔符。
#[must_use]
pub fn hexadecimal_text(bytes: &[u8]) -> String {
    let mut text = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(text, "{byte:02x}").expect("往 String 里写 fmt 不会失败");
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hexadecimal_text_keeps_the_on_disk_byte_order_and_pads_each_byte_to_two_characters() {
        assert_eq!(hexadecimal_text(&[]), "");
        assert_eq!(hexadecimal_text(&[0x03, 0x00, 0x00, 0x0f]), "0300000f");
        assert_eq!(hexadecimal_text(&[0xff, 0x00]), "ff00");
    }
}
```

### crates/singlefs-harness/src/bin/first_transaction_region_bytes.rs（152 行）

```rust
//! E142（第一个事务的干跑） 量 5 的实装一侧：在两块内存盘上跑 `scenario::run_first_transaction`
//! （与虚机档、与 E142 装置同参数：同 fsid、同写入时刻、同 3000 字节内容、两盘 4 GiB），
//! 把第一个事务写到的那 21 个区域的字节打成一行一个区域的结果行。
//!
//!   first_transaction_region_bytes
//!
//! 不收参数、不读环境、不碰真设备、不写任何文件：**只读导出**，写路径一个字节都不改。
//! 结果行形态照 `first_transaction_on_device`：`E7RESULT name=… `，末行报条数。
//! 区域清单与它的来历见 `singlefs_harness::first_transaction_regions` 的模块注释。
//!
//! 退出码：0 = 区域表与这一趟真正发出的 21 条写逐条对得上；1 = 对不上（表与实装分叉了，两边都要查）
//! 或者命令行给了参数。

use singlefs_core::address::DeviceIdentity;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_format::{FIRST_TRANSACTION_TXG, TEST_IMAGE_DEFAULT_BYTES};
use singlefs_harness::crash::{MemoryPool, SparseBlockDevice};
use singlefs_harness::first_transaction_regions::{
    region_result_lines, region_table_against_writes, HexadecimalExtent, FIRST_TRANSACTION_REGIONS,
    FIRST_TRANSACTION_REGION_COUNT,
};
use singlefs_harness::hexadecimal::hexadecimal_text;
use singlefs_harness::scenario::{
    e142_parameters, run_first_transaction, ScenarioPoint, E142_FILESYSTEM_IDENTIFIER,
    FIRST_FILE_BYTES, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

/// E142 那套几何：两块同构盘，`physical_block_size` = io_min = 512。
const DEVICE_COUNT: u32 = 2;
const PHYSICAL_BLOCK_SIZE_IN_BYTES: u32 = 512;
const MINIMUM_INPUT_OUTPUT_BYTES: u32 = 512;

/// 空清单在结果行里写 `none`，不留一个空值——空值与「这一格没打出来」在文本里分不开。
fn list_or_none(list: &str) -> &str {
    if list.is_empty() {
        "none"
    } else {
        list
    }
}

struct Emitter {
    emitted: u64,
}

impl Emitter {
    fn emit(&mut self, body: &str) {
        self.emitted += 1;
        println!("E7RESULT {body}");
    }
    fn finish(&mut self) {
        self.emitted += 1;
        println!("E7RESULT name=done emitted={}", self.emitted);
    }
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if !arguments.is_empty() {
        eprintln!("first_transaction_region_bytes 不收参数，收到 {arguments:?}");
        std::process::exit(1);
    }

    let parameters = e142_parameters(PHYSICAL_BLOCK_SIZE_IN_BYTES, MINIMUM_INPUT_OUTPUT_BYTES);
    let stream = SharedStream::new();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = (0
        ..DEVICE_COUNT)
        .map(|device_number| {
            let identity = DeviceIdentity(device_number);
            (
                identity,
                RecordingBlockDevice::with_shared_stream(
                    identity,
                    SparseBlockDevice::new(
                        TEST_IMAGE_DEFAULT_BYTES,
                        PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_SIZE_IN_BYTES),
                    ),
                    stream.clone(),
                ),
            )
        })
        .collect();

    // 第一个事务那一段在流里从哪一步起：暖机之后的那个口子上拍一次长度，之后的每一步都是这个事务发出的。
    let mut steps_before_first_transaction: Option<usize> = None;
    let run =
        run_first_transaction(
            &parameters,
            &mut devices,
            &stream,
            |point, _devices| match point {
                ScenarioPoint::AfterInstanceAcquisition => {}
                ScenarioPoint::BeforeFirstTransaction => {
                    steps_before_first_transaction = Some(stream.operations().len());
                }
            },
        )
        .expect("整条路（mkfs → 取号 → 暖机 → 第一个事务）在内存盘上跑得通");
    let steps_before_first_transaction = steps_before_first_transaction
        .expect("run_first_transaction 一定走过 BeforeFirstTransaction");

    let image = MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.inner().image.clone()))
            .collect(),
        device_size_in_bytes: TEST_IMAGE_DEFAULT_BYTES,
    };
    let operations = stream.operations();
    let against_writes = region_table_against_writes(&operations[steps_before_first_transaction..]);

    let mut emitter = Emitter { emitted: 0 };
    emitter.emit(&format!(
        "name=impl_config devices={DEVICE_COUNT} image_bytes={TEST_IMAGE_DEFAULT_BYTES} physical_block_size={PHYSICAL_BLOCK_SIZE_IN_BYTES} minimum_input_output_bytes={MINIMUM_INPUT_OUTPUT_BYTES} file_bytes={FIRST_FILE_BYTES} write_time_seconds={FIXED_WRITE_TIME_SECONDS} filesystem_identifier={} checkpoint_txg={FIRST_TRANSACTION_TXG} mkfs_operations={} policy_mismatches={}",
        hexadecimal_text(&E142_FILESYSTEM_IDENTIFIER),
        run.mkfs_operation_count,
        run.policy_mismatches
    ));
    for line in region_result_lines(&image) {
        emitter.emit(&line);
    }
    let whole_region_rows = FIRST_TRANSACTION_REGIONS
        .iter()
        .filter(|region| region.hexadecimal_extent == HexadecimalExtent::WholeRegion)
        .count();
    emitter.emit(&format!(
        "name=impl_region_bytes_summary regions={FIRST_TRANSACTION_REGION_COUNT} whole_region={whole_region_rows} head_and_tail={}",
        FIRST_TRANSACTION_REGION_COUNT - whole_region_rows
    ));
    emitter.emit(&format!(
        "name=impl_region_table_against_writes regions={FIRST_TRANSACTION_REGION_COUNT} write_calls={} regions_without_a_write={} writes_outside_the_table={} matches={}",
        against_writes.write_calls,
        list_or_none(&against_writes.regions_without_a_write.join(",")),
        list_or_none(
            &against_writes
                .writes_outside_the_table
                .iter()
                .map(|(device, offset, length)| format!("device{}@{}+{length}", device.0, offset.0))
                .collect::<Vec<String>>()
                .join(",")
        ),
        against_writes.matches()
    ));
    emitter.finish();
    if !against_writes.matches() {
        eprintln!(
            "区域表与第一个事务真正发出的写对不上：见 name=impl_region_table_against_writes 那一行"
        );
        std::process::exit(1);
    }
}
```

### crates/singlefs-harness/tests/first_transaction_region_bytes.rs（275 行，新测试文件，正文未点具体用例名，整份抄入）

```rust
//! E142（第一个事务的干跑） 量 5 的实装一侧（`first_transaction_regions` 与 `first_transaction_region_bytes`）的验收：
//! ① 区域清单就是登记那 21 行，数目与顺序都对，而且与第一个事务真正发出的 21 条写逐条配得上；
//! ② 同一条路跑两次、结果行逐字节相同；
//! ③ 改动计数那 8 字节改一位，对应区域的 sha256 变、别的 20 行一个字符都不变。
//!
//! 期望值一律从 `.claude/kb/layout/01-first-txn.md` 零那一节的写清单抄成字面量（双份记账：
//! 代码里的常量表与这里的期望表分别抄一遍登记，抄错一处就对不上），不从被测的常量表反推。

use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_format::{FIRST_TRANSACTION_TXG, ROOT_RING_REGION_DEVICES, TEST_IMAGE_DEFAULT_BYTES};
use singlefs_harness::crash::{MemoryPool, SparseBlockDevice, SECTOR_BYTES};
use singlefs_harness::first_transaction_regions::{
    region_result_lines, region_table_against_writes, FirstTransactionRegion, HexadecimalExtent,
    FIRST_TRANSACTION_REGIONS, FIRST_TRANSACTION_REGION_COUNT,
};
use singlefs_harness::scenario::{e142_parameters, run_first_transaction, ScenarioPoint};
use singlefs_harness::{RecordedOperation, RecordingBlockDevice, SharedStream};

/// 改动计数那 8 字节的盘上绝对偏移：inode 叶落在槽 50242，记录区从单元内 136 起，记录内偏移 88
/// ⇒ 50242 × 16384 + 136 + 88（跑前登记第七节乙类那一行的 823165152）。
const CHANGE_COUNT_FIELD_OFFSET: u64 = 50242 * 16384 + 136 + 88;
const CHANGE_COUNT_FIELD_BYTES: usize = 8;

struct FirstTransactionImage {
    image: MemoryPool,
    /// 录制流里第一个事务那一段（暖机之后的每一步，屏障在内）。
    first_transaction_operations: Vec<RecordedOperation>,
}

/// 与 `first_transaction_region_bytes` 那个 bin 同一条路、同参数：两块 4 GiB 内存盘上 mkfs → 取号 → 暖机 → 第一个事务。
fn run_on_memory_devices() -> FirstTransactionImage {
    let parameters = e142_parameters(512, 512);
    let stream = SharedStream::new();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = (0..2u32)
        .map(|device_number| {
            let identity = DeviceIdentity(device_number);
            (
                identity,
                RecordingBlockDevice::with_shared_stream(
                    identity,
                    SparseBlockDevice::new(TEST_IMAGE_DEFAULT_BYTES, PhysicalBlockSizeInBytes(512)),
                    stream.clone(),
                ),
            )
        })
        .collect();
    let mut steps_before_first_transaction: Option<usize> = None;
    run_first_transaction(
        &parameters,
        &mut devices,
        &stream,
        |point, _devices| match point {
            ScenarioPoint::AfterInstanceAcquisition => {}
            ScenarioPoint::BeforeFirstTransaction => {
                steps_before_first_transaction = Some(stream.operations().len());
            }
        },
    )
    .expect("整条路在内存盘上跑得通");
    let steps_before_first_transaction =
        steps_before_first_transaction.expect("暖机之后那个口子一定被叫到");
    let operations = stream.operations();
    FirstTransactionImage {
        image: MemoryPool {
            devices: devices
                .iter()
                .map(|(identity, device)| (*identity, device.inner().image.clone()))
                .collect(),
            device_size_in_bytes: TEST_IMAGE_DEFAULT_BYTES,
        },
        first_transaction_operations: operations[steps_before_first_transaction..].to_vec(),
    }
}

/// 翻一位（不是整字节）：读出那个扇区、异或最低位、写回去。
fn flip_lowest_bit(image: &mut MemoryPool, device: DeviceIdentity, offset: u64) {
    let sector_offset = DeviceOffsetInBytes(offset - offset % SECTOR_BYTES);
    let device_image = image.devices.get_mut(&device).expect("池里有这块盘");
    let mut sector = device_image.read(
        sector_offset,
        usize::try_from(SECTOR_BYTES).expect("扇区 512 字节"),
    );
    let index_in_sector = usize::try_from(offset % SECTOR_BYTES).expect("扇区内偏移");
    sector[index_in_sector] ^= 0x01;
    device_image.write(sector_offset, &sector);
}

fn read_bytes(image: &MemoryPool, device: DeviceIdentity, offset: u64, length: usize) -> Vec<u8> {
    let sector_offset = DeviceOffsetInBytes(offset - offset % SECTOR_BYTES);
    let index_in_sector = usize::try_from(offset % SECTOR_BYTES).expect("扇区内偏移");
    let sector = image.devices.get(&device).expect("池里有这块盘").read(
        sector_offset,
        usize::try_from(SECTOR_BYTES).expect("扇区 512 字节"),
    );
    sector[index_in_sector..index_in_sector + length].to_vec()
}

/// 登记那 21 行抄一遍：(区域名, 设备号, 盘上绝对偏移, 长度, 十六进制打法)。
/// 偏移列由「16 KiB 槽号 × 16384」算出（字节表零那一节的写清单），固定结构三样各自的来历写在行尾注释里。
/// 一行一个登记行，所以这个函数不让 rustfmt 拆行（拆开之后「21 行」这件事在源码里就看不出来了）。
#[rustfmt::skip]
fn registry_rows() -> Vec<(&'static str, u32, u64, u64, HexadecimalExtent)> {
    use HexadecimalExtent::{HeadAndTail, WholeRegion};
    vec![
        ("data_unit",       0, 50180 * 16384, 32768, HeadAndTail), // t1
        ("data_unit",       1, 50180 * 16384, 32768, HeadAndTail),
        ("extent_root",     0, 50240 * 16384, 16384, HeadAndTail), // t2
        ("extent_root",     1, 50240 * 16384, 16384, HeadAndTail),
        ("inode_leaf",      0, 50242 * 16384, 32768, HeadAndTail), // t3
        ("inode_leaf",      1, 50242 * 16384, 32768, HeadAndTail),
        ("inode_root",      0, 50244 * 16384, 16384, HeadAndTail), // t4
        ("inode_root",      1, 50244 * 16384, 16384, HeadAndTail),
        ("allocation_root", 0, 50245 * 16384, 16384, HeadAndTail), // t5
        ("allocation_root", 1, 50245 * 16384, 16384, HeadAndTail),
        ("accounting_root", 0, 50246 * 16384, 16384, HeadAndTail), // t6
        ("accounting_root", 1, 50246 * 16384, 16384, HeadAndTail),
        ("mapping_root",    0, 50247 * 16384, 16384, HeadAndTail), // t7
        ("mapping_root",    1, 50247 * 16384, 16384, HeadAndTail),
        ("tree_table",      0, 50248 * 16384, 16384, HeadAndTail), // t8
        ("tree_table",      1, 50248 * 16384, 16384, HeadAndTail),
        // t10：第 3 代根记录，根环区域 0（起点 1 MiB）的槽 1（槽距 4096），槽宽 = physical_block_size 512
        ("root_record",     0, 1024 * 1024 + 4096, 512, WholeRegion),
        // t9：jsn (1, 3) 那条记录，journal 环从槽 1024（16 MiB）起，环内偏移 8192（两次暖机各占一条 4096）
        ("journal_record",  0, 16 * 1024 * 1024 + 8192, 4096, WholeRegion),
        ("journal_record",  1, 16 * 1024 * 1024 + 8192, 4096, WholeRegion),
        // t11：超级块槽 1（世代号 5、tail = 3），槽距 4096、槽宽 4096
        ("superblock",      0, 4096, 4096, WholeRegion),
        ("superblock",      1, 4096, 4096, WholeRegion),
    ]
}

fn describe(region: &FirstTransactionRegion) -> (&'static str, u32, u64, u64, HexadecimalExtent) {
    (
        region.name,
        region.device.0,
        region.offset.0,
        region.length_in_bytes,
        region.hexadecimal_extent,
    )
}

#[test]
fn the_region_table_lists_the_twenty_one_registered_regions_in_order() {
    let expected = registry_rows();
    assert_eq!(
        expected.len(),
        FIRST_TRANSACTION_REGION_COUNT,
        "登记第一节第 3 条写死 21 行：8 个单元 × 2 盘 + 根槽 1 + journal 记录 × 2 盘 + 超级块槽 × 2 盘"
    );
    assert_eq!(
        FIRST_TRANSACTION_REGIONS.len(),
        FIRST_TRANSACTION_REGION_COUNT
    );
    let actual: Vec<(&'static str, u32, u64, u64, HexadecimalExtent)> =
        FIRST_TRANSACTION_REGIONS.iter().map(describe).collect();
    assert_eq!(
        actual, expected,
        "区域清单要与登记那 21 行逐行相同，顺序也相同"
    );
    let whole_region_rows = FIRST_TRANSACTION_REGIONS
        .iter()
        .filter(|region| region.hexadecimal_extent == HexadecimalExtent::WholeRegion)
        .count();
    assert_eq!(
        whole_region_rows, 5,
        "整段十六进制照打的只有固定结构那 5 行：根槽 512、journal 记录 4096 两份、超级块槽 4096 两份"
    );
}

#[test]
fn the_region_table_matches_the_writes_the_first_transaction_really_issues() {
    let run = run_on_memory_devices();
    let against_writes = region_table_against_writes(&run.first_transaction_operations);
    assert_eq!(
        against_writes.regions_without_a_write,
        Vec::<&str>::new(),
        "表里每一行都要有一条写落在它上面"
    );
    assert_eq!(
        against_writes.writes_outside_the_table,
        Vec::<(DeviceIdentity, DeviceOffsetInBytes, u64)>::new(),
        "第一个事务发出的每一条写都要在表里"
    );
    assert_eq!(
        against_writes.write_calls, FIRST_TRANSACTION_REGION_COUNT,
        "字节表零那一节：事务本身 21 条写请求"
    );
    assert!(against_writes.matches());

    // 根槽那一行不靠表自己说了算：区域与槽号由 root_ring 按 txg 算，落哪块盘由 mkfs 的 region_devices 定。
    let root_target = target_for_publish(CheckpointTxg(FIRST_TRANSACTION_TXG));
    let root_row = FIRST_TRANSACTION_REGIONS
        .iter()
        .find(|region| region.name == "root_record")
        .expect("表里有根槽那一行");
    assert_eq!(root_row.offset, slot_offset(root_target, 4096));
    assert_eq!(
        root_row.device,
        DeviceIdentity(
            ROOT_RING_REGION_DEVICES[usize::try_from(root_target.region).expect("区域号")]
        )
    );
}

#[test]
fn running_the_same_pipeline_twice_gives_byte_identical_result_lines() {
    let first_run = run_on_memory_devices();
    let second_run = run_on_memory_devices();
    let first_lines = region_result_lines(&first_run.image);
    assert_eq!(first_lines.len(), FIRST_TRANSACTION_REGION_COUNT);
    assert_eq!(
        first_lines,
        region_result_lines(&first_run.image),
        "同一个镜像打两次，结果行逐字节相同"
    );
    assert_eq!(
        first_lines,
        region_result_lines(&second_run.image),
        "整条路跑两次（不取系统时钟、不取随机数），21 行结果行逐字节相同"
    );
}

#[test]
fn flipping_one_bit_of_the_change_count_moves_only_that_regions_digest() {
    let run = run_on_memory_devices();
    let before = region_result_lines(&run.image);
    assert_eq!(
        read_bytes(
            &run.image,
            DeviceIdentity(0),
            CHANGE_COUNT_FIELD_OFFSET,
            CHANGE_COUNT_FIELD_BYTES
        ),
        FIRST_TRANSACTION_TXG.to_le_bytes(),
        "改动计数那 8 字节在 inode 叶记录区偏移 88（盘上绝对偏移 {CHANGE_COUNT_FIELD_OFFSET}），第一个事务写 checkpoint_txg"
    );

    let mut flipped_image = run.image.clone();
    flip_lowest_bit(
        &mut flipped_image,
        DeviceIdentity(0),
        CHANGE_COUNT_FIELD_OFFSET,
    );
    let after = region_result_lines(&flipped_image);

    let changed: Vec<usize> = before
        .iter()
        .zip(&after)
        .enumerate()
        .filter(|(_index, (old_line, new_line))| old_line != new_line)
        .map(|(index, _lines)| index)
        .collect();
    assert_eq!(
        changed,
        vec![4],
        "翻的是盘 0 的 inode 叶那一位：只有第 5 行（inode_leaf device=0）该变，别的 20 行一个字符都不许动"
    );
    let changed_region = &FIRST_TRANSACTION_REGIONS[4];
    assert_eq!(changed_region.name, "inode_leaf");
    assert_eq!(changed_region.device, DeviceIdentity(0));
    assert_ne!(
        sha256_field(&before[4]),
        sha256_field(&after[4]),
        "改动计数改一位，这个区域的 sha256 必须跟着变"
    );
}

/// 结果行里 `sha256=` 那一格。
fn sha256_field(line: &str) -> &str {
    line.split_whitespace()
        .find_map(|field| field.strip_prefix("sha256="))
        .expect("每一行结果行都有 sha256= 那一格")
}
```

## 三、crates/mutations.tsv 第 101–120 行（主 agent 指定的范围，按 --extra 文件:1-末行 同类整段带入，制表符分隔，原样抄入）

```tsv
增补 2 第 21 行：暖机把 jsn 计数器写成 txg（C366；接在 jsn 40 之后时记录落进 jsn 1、2 那两格、tail 写 2）	crates/singlefs-core/src/transaction.rs	txg: CheckpointTxg(txg_number),\n                counter: previous_counter + 1,	txg: CheckpointTxg(txg_number),\n                counter: txg_number,	-p singlefs-harness --test second_transaction_supplement_two_warm_up_counter -- warm_up_after_a_journal_counter	warm_up_after_a_journal_counter_other_than_zero_counts_records_on_from_it_while_txg_stays_one_and_two
增补 2 第 19 行：出生序号发号器挪回每装一个文件对象重建一次（同一个 checkpoint 的第二个对象从 0 重数）	crates/singlefs-core/src/transaction.rs	) -> FileVersionUnits {\n    let txg = checkpoint.txg;	) -> FileVersionUnits {\n    let mut sequences_rebuilt_on_every_call = BirthSequenceAllocator::default();\n    let sequences = &mut sequences_rebuilt_on_every_call;\n    let txg = checkpoint.txg;	-p singlefs-core --lib -- transaction::tests::second_file_object	second_file_object_in_the_same_checkpoint_continues_birth_sequences_instead_of_restarting_at_zero
增补 2 第 28 行：记账树装不下不判（80 块盘走到装节点的断言 panic）	crates/singlefs-core/src/transaction.rs	    if accounting_entries_of_this_publish > accounting_node_capacity {	    if false && accounting_entries_of_this_publish > accounting_node_capacity {	-p singlefs-harness --test second_transaction_supplement_two_accounting_node_full -- eightieth_device	eightieth_device_overflows_the_accounting_node_and_the_publish_is_refused_before_anything_is_written
增补 2 第 28 行：记账树正好装满 477 行也判装不下（79 块盘发布被拒）	crates/singlefs-core/src/transaction.rs	    if accounting_entries_of_this_publish > accounting_node_capacity {	    if accounting_entries_of_this_publish >= accounting_node_capacity {	-p singlefs-harness --test second_transaction_supplement_two_accounting_node_full -- seventy_nine_devices	seventy_nine_devices_fill_the_accounting_node_exactly_and_still_publish
增补 2 第 30 行：真设备二进制的挂载窗口从重开那一刻算起（取号的两次超级块槽写进了设备一层的数、不在按种类的账里）	crates/singlefs-harness/src/bin/first_transaction_on_device.rs	        pool_writes_between(&counts_after_acquisition, &counts_after_mount),	        pool_writes_between(&vec![DeviceCallCounts { write_calls: 0, written_bytes: 0, force_unit_access_writes: 0, barrier_calls: 0 }; counts_after_mount.len()], &counts_after_mount),	-p singlefs-harness --bin first_transaction_on_device -- second_instance_mode	second_instance_mode_mounts_writes_the_row_warms_up_publishes_the_third_version_and_every_window_matches
增补 2 第 30 行：真设备二进制每道屏障都覆盖「第一道屏障」那份计数（挂载窗口从最后一道屏障算起）	crates/singlefs-harness/src/bin/first_transaction_on_device.rs	        if self.counts_when_first_barrier_arrived.is_none() {	        if true {	-p singlefs-harness --bin first_transaction_on_device -- second_instance_mode	second_instance_mode_mounts_writes_the_row_warms_up_publishes_the_third_version_and_every_window_matches
增补 2 第 20a 行：写行那次发布的准入不在取号之前算（取号之后发布才被拒，实例代号已经烧掉）	crates/singlefs-core/src/mount.rs	    refuse_publishes_before_acquisition_that_do_not_pass_admission(\n        &allocator,\n        &start,\n        instance_to_acquire,\n        warm_up_publishes_planned.len(),\n    )?;\n	    // 变异：写行那次发布的准入不在取号之前算\n	-p singlefs-harness --test second_transaction_supplement_two_row_publish_admission -- a_writable_mount_that_cannot_publish	a_writable_mount_that_cannot_publish_the_rows_is_refused_before_the_instance_is_acquired
增补 2 第 20b 行：中途失败的发布不把这次已记的写交出去（与设备一层记的合计对不上）	crates/singlefs-core/src/transaction.rs	    if let Err(cause) = persist(pool) {\n        pool.count_failed_publish(&writes_before_this_publish);\n        return Err(PublishError::BlockDevice(cause));\n    }	    if let Err(cause) = persist(pool) {\n        return Err(PublishError::BlockDevice(cause));\n    }	-p singlefs-harness --test second_transaction_supplement_one_write_accounting -- a_publish_that_fails_midway	a_publish_that_fails_midway_hands_out_what_it_already_wrote_so_the_retry_still_adds_up
增补 2 第 20c 行：用户数据只排除当前那一个开放段（回落把开放段置空之后，那一段对用户数据开放）	crates/singlefs-core/src/allocator.rs	        let cluster_segments = &self.cluster_segments;	        let cluster_segments = &self.open_segment.into_iter().collect::<BTreeSet<SlotNumber>>();	-p singlefs-core --lib -- allocator::tests::user_data_does_not_land	user_data_does_not_land_in_a_cluster_segment_that_the_fallback_closed
收口表第 22 行 C22（刚释放的块立即重分配）：候选根那一格不判 I-4.8（复用窗口置 0 之后 I-4.8 不红）	crates/singlefs-checker/src/walk.rs	                .judge("I-4.8", !walked_into_reused_or_erased_unit, || {	                .judge("I-4.8", true, || {	-p singlefs-harness --test second_transaction_step_five_reuse -- reclaiming_without_raising_the_floor	reclaiming_without_raising_the_floor_reuses_a_slot_a_candidate_root_still_references_and_the_checker_goes_red
增补 2 第 20a 行：取号之前只算写行那一次，暖机那几次空发布不算（写行发完、暖机才被拒，实例代号已经烧掉）	crates/singlefs-core/src/mount.rs	            let shapes: Vec<PublishShape> = std::iter::once(PublishShape::ROW_PUBLISH)\n                .chain(std::iter::repeat_n(\n                    PublishShape::EMPTY_PUBLISH,\n                    warm_up_publishes_planned,\n                ))\n                .collect();	            let shapes: Vec<PublishShape> = vec![PublishShape::ROW_PUBLISH];	-p singlefs-harness --test second_transaction_supplement_two_row_publish_admission -- a_writable_mount_whose_first_warm_up	a_writable_mount_whose_first_warm_up_publish_does_not_fit_is_refused_before_the_instance_is_acquired
增补 2 第 20a 行：取号之前只把第一次暖机算进去（写行装得下、暖机第 2 次装不下的池照样放行）	crates/singlefs-core/src/mount.rs	                    PublishShape::EMPTY_PUBLISH,\n                    warm_up_publishes_planned,	                    PublishShape::EMPTY_PUBLISH,\n                    warm_up_publishes_planned.min(1),	-p singlefs-harness --test second_transaction_supplement_two_row_publish_admission -- a_writable_mount_whose_second_warm_up	a_writable_mount_whose_second_warm_up_publish_does_not_fit_is_refused_before_the_instance_is_acquired
增补 2 第 11 行：第一个事务的改动计数写回 1（字段表定义是最后一次改动所在发布的 checkpoint_txg = 3）	crates/singlefs-core/src/transaction.rs	                change_count: FIRST_TRANSACTION_TXG,	                change_count: 1,	-p singlefs-harness --test first_transaction_step_five_publish -- inode_and_extent_lookups	inode_and_extent_lookups_from_the_root_read_the_first_file_back
C374 I-3.9：释放代的区间判据整条拿掉（判定恒真）	crates/singlefs-checker/src/walk.rs	        let in_the_witnessed_interval = record.generation > last_referencing_txg\n            && first_root_without_it.is_some_and(|txg| record.generation <= txg);	        let in_the_witnessed_interval = true;	-p singlefs-harness --test checker_known_bad_images -- each_c374_bad_image	each_c374_bad_image_reddens_only_its_own_invariant_after_the_overwrite
C374 I-9.14：树表条目诞生 txg 跨根相等的判定恒真	crates/singlefs-checker/src/walk.rs	        let first_birth_txg = sightings[0].birth_txg;\n        let birth_txg_is_the_same_across_roots = sightings\n            .iter()\n            .all(|sighting| sighting.birth_txg == first_birth_txg);	        let birth_txg_is_the_same_across_roots = true;	-p singlefs-harness --test checker_known_bad_images -- each_c374_bad_image	each_c374_bad_image_reddens_only_its_own_invariant_after_the_overwrite
E142 量 5：sha256 轮常量改一位	crates/singlefs-harness/src/sha256.rs	    0x428a_2f98,	    0x428a_2f99,	-p singlefs-harness --lib -- sha256::tests	sha256_matches_the_published_test_vectors
E142 量 5：区域清单换行序	crates/singlefs-harness/src/first_transaction_regions.rs	    unit_region("data_unit", 1, DATA_UNIT_SLOT, 2),\n    unit_region("extent_root", 0, EXTENT_ROOT_SLOT, 1),	    unit_region("extent_root", 0, EXTENT_ROOT_SLOT, 1),\n    unit_region("data_unit", 1, DATA_UNIT_SLOT, 2),	-p singlefs-harness --test first_transaction_region_bytes -- the_region_table_lists	the_region_table_lists_the_twenty_one_registered_regions_in_order
E142 量 5：数据单元那一行少算一槽	crates/singlefs-harness/src/first_transaction_regions.rs	    unit_region("data_unit", 0, DATA_UNIT_SLOT, 2),	    unit_region("data_unit", 0, DATA_UNIT_SLOT, 1),	-p singlefs-harness --test first_transaction_region_bytes -- the_region_table_matches	the_region_table_matches_the_writes_the_first_transaction_really_issues
E142 量 5：写入时刻改取系统时钟	crates/singlefs-harness/src/scenario.rs	            write_time_seconds: FIXED_WRITE_TIME_SECONDS,	            write_time_seconds: u64::try_from(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).expect("单调").as_nanos()).expect("纳秒放得进 u64"),	-p singlefs-harness --test first_transaction_region_bytes -- running_the_same_pipeline_twice	running_the_same_pipeline_twice_gives_byte_identical_result_lines
E142 量 5：区域摘要不罩内容	crates/singlefs-harness/src/first_transaction_regions.rs	        sha256_hexadecimal(&bytes),	        sha256_hexadecimal(&bytes[..0]),	-p singlefs-harness --test first_transaction_region_bytes -- flipping_one_bit	flipping_one_bit_of_the_change_count_moves_only_that_regions_digest
```
