# m2-emptypool-nonempty-r1 云端攻方腿（Opus）报告：Z1 / Z2 / Z3 / Z4（2026-09-17）

<!-- doc-lint:not-numbers V1 V2 V3 Z1 Z2 Z3 Z4 Z5 Z6 Z7 -->

## 复跑与文件

副本：`/tmp/claude-1000/m2-emptypool-nonempty-r1-opus/copy/`（2026-09-17 10:16 UTC `rsync -a --exclude target --exclude .git` 拷自主树；`crates/singlefs-core/src/mount.rs` 与 `transaction.rs` 的 sha256 与主树逐字相同：`dd281a60…f158`、`8afadf28…6f04`；副本 `crates/` 与主 agent 快照 `emptypool-r1-legs-snapshot/crates` 用 `diff -rq` 比过、零差异）。**这份报告里所有数都是副本上量的，不是入库装置上的数。**

复跑（把模型放回副本的 harness 测试目录再跑；debug、`nice -n 19`，整份 9 条用例约 25 秒）：

```
cp research/prompts/m2-emptypool-nonempty-r1-opus-model/opus_attack_emptypool.rs <副本>/crates/singlefs-harness/tests/
cd <副本> && nice -n 19 cargo test --offline -p singlefs-harness --test opus_attack_emptypool -- --nocapture --test-threads 1
```

| 文件（`research/prompts/m2-emptypool-nonempty-r1-opus-model/`） | 行数 | sha256 |
|---|---|---|
| `opus_attack_emptypool.rs`（9 条用例：Z1 两条、Z2 一条、Z3 四条、Z4 一条、V2 射程一条） | 692 | `f4a12b7bfd07a8f4f36cdad3a1d66518b54114a04f16b0a7d0db285877d98146` |
| `run-all.log`（复跑那条 `cargo test` 命令的原样输出，`exit=0`，`test result: ok. 9 passed`） | 98 | `7b75ac23327e66fa4fe86ca2e42af7a9e2b645b996b491106e2612b945ea577f` |
| `copy-only-guard-for-z3.diff`（我给 Z3 两格提的改法，只在副本上打过、跑完已还原） | 21 | `ba67e8892fb13e3f1ba3f8c4292b8e6ccd8ee41450371823074845b6cfee1ba3` |

开跑前 `ps` 看到的：宿主上门禁 54 号的 `cargo test --release -p singlefs-harness --test second_transaction_step_zero_layer0 -- --include-ignored`（pid 1325368）在跑，没有 qemu / fio / e152；我只在副本自己的 `target/` 里跑 debug，没跑 release。

## 各格判定一览

| 格 | 判定 | 一句话 | 故障数 |
|---|---|---|---|
| Z1 | **打中 1 条（不分辨 V1 本身，病根在 F 回落）**；没打中 6 种形状 | F 回落（D16 已定项 1「生效」行，已打回重议）之后再抬 F：「前一条有效根」按回落后的 F 取，把 F 之下、树表单元已被 E 合法复用的第 0 代根算进有效根，V1 当成「读不出、要走修复」拒绝——修复修不了一个没坏的单元，能让池走出回落的抬 F 被挡住，直到根环把那几条 F 之下的根轮转盖掉 | 1（一个根槽坏一个字节） |
| Z2 | **打中 1 条（形状级，要瞬时读错）** | 拒绝判定与取号各读一遍超级块：两块盘第 2 次读槽 0 各报一次瞬时读错 ⇒ 拒绝判定看到号 1 放行、取号读到 2 并写进两盘超级块、然后 `assert!` panic——panic 时盘上已经有两次写 | 2 次瞬时读错（推的、没量：取号只落一盘的崩溃状态上 1 次就够） |
| Z3 | **打中 2 条** + 没打中 4 种 + 一条落在判据字面之外的射程观测 | A：两块盘超级块槽 0 各坏一字节 + 第一次挂载崩在 txg 1 的记录之后 ⇒ 空池挂载放行、新实例从 txg 2 起、零单元发布 3 次，`publish_first_file` 写死 txg 3 返回成功，冷恢复落在 txg 4 的暖机根上、文件没了，checker 零违例。B：mkfs 接受的区域归属 [0,1,1] / [1,0,0]（条款说第一版写死 0/1/0、不是 mkfs 参数，代码不核）⇒ 零故障就走到两条不同的 (1, 3) 根，快枚举 29 个状态里 checker I-7.8 判红 4、记录核对器红 4 | A：2 + 1 次崩溃；B：0 |
| Z4 | **问句的后一半坐实，触发句没中** | 第三条流与第一条流的崩溃镜像逐个相同（基线、33 个写、10 段三样 `==`）；枚举每个状态只跑两遍恢复 + checker + 记录核对器，不重开挂载——层 0 第三条流没有多罩一个状态。在同一批状态上补跑可写挂载：46 个取样状态 38 个被拒、8 个放行 | 0 |

V2 射程观测（不落在 Z1–Z4 的触发句上，照实交回）：只做过 mkfs 的池可写挂载成功、一个文件都不写、进程正常退出 ⇒ 之后每一次可写挂载都被 `InstanceRowsOnVersionWithoutFileUnsupported` 拒（零故障、零崩溃，第五节）。

## 一、Z1 非空判法的漏判与误判

### 1.1 打中：F 回落之后再抬 F，V1 把「F 之下、被合法复用的树表单元」当成读不出而拒绝

历史（模型 `opus_attack_emptypool.rs:450` `z1_raising_the_floor_after_it_fell_back_is_refused_on_a_legitimately_reused_tree_table`，对照组同一个函数里不改坏根槽）：

1. 固定脚本到 D、回退之后四次覆盖写 txg 11–14、抬 F 到 11（txg 15 落盘 0、16 落盘 1）、覆盖写 E（txg 17）——与 `crates/singlefs-harness/tests/second_transaction_step_five_reuse.rs` 的 `build_through_rollback` 与第 131 行那条验收用例同一段历史；E 的数据单元落回 50178（那条用例第 191 行断言的就是它，mkfs 第 0 版树表那一槽回收后被复用）。模型打印 `E_data_slot=50178`。
2. 进程退出，把 txg 16 的根槽（区域 1、盘 1，盘 1 上唯一带 F = 11 的根）改坏一个字节——与同文件第 250 行 `one_device_carrying_the_floor_alone_does_not_take_effect_on_remount` 的改法逐字相同。
3. 重开可写挂载：`crates/singlefs-core/src/recovery.rs:351` `effective_rollback_floor` 取各盘所带 F 最大值的最小值 = 0，新实例的根带 F = 0。许可它的条款 `.claude/kb/decisions/16-发布语义.md:375` 整行：`| 生效 | 每块幸存盘上都有带新 F 的持久根才生效；恢复后生效值 = 各幸存盘所带 F 最大值的最小值 |`。模型打印 `floor_after_remount=0`。
4. 再抬 F（到 12）：`crates/singlefs-core/src/mount.rs:569` 以 `current.root.rollback_floor`（= 0）调 `rollback_floor_ceiling`；第 464 行 `.filter(|root| root.checkpoint_txg >= current_floor)` 把第 0 代根留在有效根里；它的树表指针指 50178，那一槽现在是 E 的数据单元，整单元 CRC 对不上；第 485 行 `Err(failure) => Err(` 包成 `RollbackFloorCeilingNeedsUnreadableValidRootTreeTable`，第 569 行的 `?` 返回。许可「有效 = txg ≥ 当前的 F」「读不出就拒绝」的是 `.claude/kb/decisions/16-发布语义.md:383` 整行：⚠️ **「非空」从盘上怎么认（2026-09-17 用户定案）**：一条有效根算非空 ⟺ 它的树表里用户可见的树（inode 树、extent 树）的根指针，与它前一条有效根（按 txg 排；有效 = 按实例表判仍然有效 ∧ txg ≥ 当前的 F）树表里的不同。用户选的是「比较树表与上一条根」；比的是树表里这两棵树的条目、不是树表单元自己的落点——每一次发布（连空发布）都重写树表单元，按落点比会把空发布全算成非空（主 agent 核代码时收窄，`crates/singlefs-core/src/transaction.rs` 里空发布也重写树表）。——以及与 `mount.rs` 第 73–78 行 `MountError` 那个成员的文档注释。

`run-all.log` 原样两行：

```
Z1F damage_txg16=false E_data_slot=50178 floor_after_remount=11 genesis_root_tree_table_slot=50178 genesis_pointers=Err(UnitUnreadable { slot: SlotNumber(50178) }) raise_to_12 -> Ok((12, 3))
Z1F damage_txg16=true E_data_slot=50178 floor_after_remount=0 genesis_root_tree_table_slot=50178 genesis_pointers=Err(UnitUnreadable { slot: SlotNumber(50178) }) raise_to_12 -> Err("RollbackFloorCeilingNeedsUnreadableValidRootTreeTable { root: RollbackTarget { instance: InstanceGeneration(0), checkpoint_txg: CheckpointTxg(0) }, failure: UnitUnreadable { slot: SlotNumber(50178) } }")
```

对照组那一行里第 0 代根的树表同样读不出（`genesis_pointers=Err`），只是它在 F = 11 之下、不进有效根——被拒的原因不是单元坏了，是有效根的下界退回了 0。

**后果**：F 回落把 F 之下、单元已被复用的根重新放回有效根（这是 D16（发布语义） 已定项 1「生效」行 2026-09-17 打回重议时已记的那一段）；V1 在这之上加了一道：此后每一次抬 F 都在第 0 代根上被拒，直到根环把引用 50178 的第 0 代根、(1, 1)、(1, 2) 轮转盖掉（区域 0 / 1 / 2 的槽 0 分别等 txg 24、25、26 写进来；F 之下其余根的单元有没有被复用、会不会接着挡，我没逐条查）；准入不够时先推空发布抬 F（D16（发布语义） 已定项 1 表格「准入」那一行，`.claude/kb/decisions/16-发布语义.md` 第 377 行，行长只指路），这段时间里空间一紧，抬 F 这条出路就是断的——「读不出要走修复」在这里修不出东西，50178 上的字节是 E 的合法数据。

四句：

| 问 | 答 |
|---|---|
| 分不分辨臂 | 在「抬 F 被挡住」这一维上分辨：V1 的读法（有效根树表读不出 ⇒ 拒绝）挡住；实现员报告第八节 ④ 的另一读法（「可读」连树表也读得出才算有效）把第 0 代根排出有效根、上限照算、抬 F 放行、F 回到 11 之上；按记录事务号认（用户已否）不读树表、同样放行。病根（F 回落让被复用的根变回候选）三种读法共用、不分辨 |
| 系统当时看不看得到判别它的东西 | 看得到：抬 F 时手里的 `current.allocation_records` 里 50178 那条是分配代 17、未释放——自己的账就写着这一槽已被重新分配，第 0 代根对它的引用是旧的；盘 0 上 txg 15、17 两条根也带着 F = 11 |
| 满足判据字面的哪一个分句 | 问句「「前一条有效根」在……F 刚抬过的历史上取到的是不是该比的那一版」：不是（取进了 F 回落之前已不该在的根）。触发句「一段历史让上限多算或少算一格」**不中**：上限根本没算出来，结局是拒绝。按「打中归错了判据」自报：这一格的形状更接近 V1 与 D16 已定项 1「生效」重议的耦合，不是计数错 |
| 跑前条款的改法在这几格上还中不中 | 条款给的「落在没有条款的地方 ⇒ 代码按最保守的读法改（拒绝优先于猜）」：今天的代码就是拒绝，照它改**对这一格不起作用**。能让这一格不中的是「树表读不出的根不算有效根」（与用户「读不出不许跳过」直接冲突）或修 F 回落本身（重议中）——两样都没在模型里实现，没量 |

### 1.2 Z1 没打中的形状

| 形状 | 取样 / 依据 | 结论 |
|---|---|---|
| 同一个对象写回同样的内容 | 模型 `opus_attack_emptypool.rs:494`：`build_pool` 之后同内容覆盖写两次（txg 4、5），比两条根树表里的 86 字节 | 原样：`Z1S same content txg 4 vs 5: inode differing byte offsets [42, 54, 60, 61, 62, 63, 68, 74, 75, 76, 77]; extent differing byte offsets [42, 54, 60, 61, 62, 63, 68, 74, 75, 76, 77]`——偏移 42 是出生 txg（`crates/singlefs-core/src/transaction.rs:1430` `birth_txg: txg,`），54 / 68 槽号、60–63 / 74–77 校验和。两条有效根要同出生 txg 就得同 txg，而新实例的第一条 txg = max(环, 记录) + 1（`mount.rs:208` `first_txg_of_new_instance`）、同实例逐次 +1，同 txg 落同一个根槽、后写盖前写，两条并存不了 |
| 落点复用到同一个槽、位置条目与校验和都相同 | 模型 `opus_attack_emptypool.rs:494` 那一格：出生 txg 那 8 字节（偏移 42 起）先不同 | 没打中 |
| 空发布 / 写行 / 回退那次让两棵树的根指针字节变 | 读 `transaction.rs:1452–1455`（没有文件版本时两条指针取 `carried.tree_root_pointer`）；重开之后 `recovery.rs` 解析条目、发布时重新序列化，指针头里 MAC / nonce / 算法类型 / 压缩码 / 压后长度 / extent 偏移共 34 字节读时跳过、写时写 0（`crates/singlefs-core/src/pointer.rs` `PointerHead::write_to` / `read_from`），这个写者写出的原字节恒为 0 | 没打中；只有别的写者写过非 0 的 MAC 段、本实现重开后照抄才会把空发布算成非空（没造） |
| 最旧有效根与「两棵树都没有」比 | 读 `mount.rs:508` 与 `ceiling_from_newest_and_non_empty_roots`：最旧那条若是空发布会被算成非空，但它就是有效根里 txg 最小的那条，而非空不足 4 条时的回落值也是它 | 对上限无影响（论证，没量） |
| 被抛弃根夹在两条有效根之间、两次回退 | 已有用例 `the_rollback_publish_is_compared_with_the_previous_valid_root_and_not_with_the_abandoned_root_before_it`；两次回退（回退到 (3, 13)）只按代码推：写行区间恒为 [max(r_old, 1), 新实例)，R_old 之后的根全被抛弃，新实例第一条 txg 大于环里全部 txg，另一条分支上的有效根插不进来 | 没打中（第二种没建模） |
| 不经 F 回落，有效根的树表被合法复用 | 论证：txg ≥ F 的根 R 的树表在 R + 1 那次发布被放掉，释放代 R + 1 > max(F, 环里最旧有效根) | 没打中；变体「挂载那一刻第 0 代、(1, 1)、(1, 2) 三条根槽瞬时读不出，环里最旧有效根算高、50178 被回收」要三次瞬时读错，没建模 |

## 二、Z2 拒绝的时机

### 2.1 打中：拒绝判定与取号各读一遍超级块，瞬时读错让两次算出不同号，panic 时两盘超级块已写进新号

镜像：只做过 mkfs 的池上第一次可写挂载，录制流只持久取号那一段（两个超级块槽写，号 1、世代号 2：段 0 整段持久、段 1 一个都没持久，段号从 0 数）——4.2 节表里「段 1 掩码 0」那一行的状态。设备包一层 `TransientSuperblockReadErrorDevice`（模型 `opus_attack_emptypool.rs:294` 起，用例在第 331 行），按「偏移 0 那一槽第几次读」注入一次瞬时读错，其余读写原样。

每一步许可它的代码：

1. `mount.rs` `mount_writable` 里 `choose_superblock`（`recovery.rs:219`）读每盘槽 0 一次（第 1 次读）；读错时 `recovery.rs:78` `block_device.read_at(offset, &mut buffer).ok()?;` 把错变成 `None`，那一槽当作没有。
2. `mount.rs:777` `let instance_to_acquire = instance_generation_to_acquire(&pool);`：`transaction.rs:287` 起 → `highest_superblock_instance` → `recovery.rs:776` `.filter_map(|offset| reader.read(device, DeviceOffsetInBytes(offset), slot_bytes))`，每盘槽 0 读第 2 次；两盘第 2 次都读错 ⇒ 只剩 mkfs 的槽 1（号 0），根环里没有实例 1 的根 ⇒ 要取的号 = 1。
3. `mount.rs:778` `refuse_instance_rows_on_version_without_file(&start, instance_to_acquire)?;`：区间 [max(0, 1), 1) 为空，放行。
4. `mount.rs:779` `let instance = acquire_instance(&mut pool).map_err(MountError::Acquisition)?;`：`transaction.rs:310–311` 自己再算一遍（第 3、4 次读，这回读得到号 1）⇒ 取 2，两盘各写一次超级块槽（世代号 3、号 2），再发屏障。
5. `mount.rs:785` `for row_instance in first_row_instance..instance.0 {` 按取到的 2 算出写行区间 [1, 2)，`mount.rs:821` `rows_written.is_empty(),` 那条 `assert!` panic。

`run-all.log` 原样（`failing_reads_per_device` 是每盘第几次读槽 0 报错；最后一列是 panic 之后两盘四个超级块槽的（盘, 号, 世代号））：

```
Z2 failing_reads_per_device=[] failed=[[], []] -> InstanceRowsOnVersionWithoutFileUnsupported(chosen (0, 0), rows [1, 2)); disk_changed=false; superblock (device, instance, generation)=[(0, 1, 2), (0, 0, 1), (1, 1, 2), (1, 0, 1)]
Z2 failing_reads_per_device=[1] failed=[[1], [1]] -> InstanceRowsOnVersionWithoutFileUnsupported(chosen (0, 0), rows [1, 2)); disk_changed=false; superblock (device, instance, generation)=[(0, 1, 2), (0, 0, 1), (1, 1, 2), (1, 0, 1)]
Z2 failing_reads_per_device=[1, 2] failed=[[1, 2], [1, 2]] -> panic: 取号之前 refuse_instance_rows_on_version_without_file 按同一个号核过：树表 0 条的一版上要写的行为空; disk_changed=true; superblock (device, instance, generation)=[(0, 1, 2), (0, 2, 3), (1, 1, 2), (1, 2, 3)]
Z2 failing_reads_per_device=[2] failed=[[2], [2]] -> panic: 取号之前 refuse_instance_rows_on_version_without_file 按同一个号核过：树表 0 条的一版上要写的行为空; disk_changed=true; superblock (device, instance, generation)=[(0, 1, 2), (0, 2, 3), (1, 1, 2), (1, 2, 3)]
```

前两行是对照：不注入、或只在择超级块那一次读错，拒绝照常在写之前返回、盘上不变。只让第 2 次读错（最后一行）就够：两盘各一次瞬时读错。**推的、没量**：取号只落了一块盘的那个崩溃状态（另一盘槽 0 仍是号 0）上，只要落了号 1 的那块盘第 2 次读错一次就同样走到 panic。

**后果**：panic 之前盘上多了两次超级块写（号 2）；下一次可写挂载要取 3、写 [1, 3)，照样被拒——比不注入时多跳一个号、多一次 panic，没有丢数据。

四句：

| 问 | 答 |
|---|---|
| 分不分辨臂 | 分辨「判定与取号各自读盘」（今天）和「取号只认判定时读到的那个号：写之前再算一遍、对不上就不写、报错返回」两种写法；后者在这一格上写之前返回。不分辨取号规则本身的旧缺口：瞬时读错把号算低会让取号重用旧号（`acquire_instance` 在 V3 之前就这样，D18（块里携带什么信息） 已定项 11 的「重试到 T_retry」没做，`transaction.rs:305–306` 的 ⚠️ 自己写着） |
| 系统当时看不看得到判别它的东西 | 设备报了错（`read_at` 返回 `Err`），`recovery.rs:78` 的 `.ok()?` 把它和「这一槽没有合法超级块」合成同一个 `None`——判别它的信号在读路径上被丢掉 |
| 满足判据字面的哪一个分句 | 问句「两盘超级块不一致、一盘槽坏时两次会不会算出不同号」：会（瞬时读错，不是持久坏槽；持久坏槽两次读到的一样，第一行对照就是）。触发句「一段历史让拒绝返回时盘上已经有一次写」：这里不是 `Err` 返回，是代替拒绝的那条 `assert!`（实现员报告续·一：原 `todo!` 那一处改成断言）在写之后 panic——按字面算半中，我照实报成「断言形态的拒绝在写之后触发」 |
| 跑前条款的改法在这几格上还中不中 | 「拒绝优先于猜」若落成把第 821 行的 `assert!` 换成 `Err` 返回：**还中**（写在第 779 行已经发生）。要让它不中得把比较挪到写之前（取号函数收一个期望号，重算不等就不写），或判定与取号共用同一次读——都没实现，没量 |

### 2.2 Z2 没打中的形状

| 形状 | 取样 / 依据 | 结论 |
|---|---|---|
| `mount_writable` 在 `establish_instance` 之前有写 | 逐个读 `choose_superblock`、`choose_root`、`scan_journal`、`replay_journal`（`recovery.rs:819` 起，只读）、`rebuild_previous_version`、`first_txg_of_new_instance`、`rebuilt_allocator`（回收与隔离只动内存里的分配器） | 没有写 |
| 两盘超级块持久不一致（取号只落一块盘）、一盘槽 0 持久坏 | 4.2 节表里「段 0 掩码 1、2」两个状态：两次算号读到同一份字节 | 在写之前拒绝，没打中 |
| 回退到树表 0 条的根：拒绝之前的副作用 | 读 `mount.rs` `mount_rollback` 到第 1016 行 `if tree_table_has_no_entries(&*devices, &target_root)? {` 之前：`choose_superblock`、`choose_root`、`scan_journal`、`readable_roots`、`instance_table_of_root`、`effective_rollback_floor`，全是读；树表单元第一次读错走 `?` 也在写之前 | 没打中 |
| 抬 F 的两种拒绝之前的副作用 | 读 `raise_rollback_floor`：第 569 行 `?` 之前只有 `choose_superblock` 与解析内存里的实例表单元；重算影子账、回收、扣住、发布都在其后。一次抬 F 失败（第三轮 X8-A 那种）留下的扣住位再遇到这里的拒绝，是那次失败的遗留、不是这次拒绝的副作用 | 没打中 |

## 三、Z3 空池挂载之后的第一个文件版本

### 3.1 打中 A：两块盘超级块槽 0 各坏一字节 + 第一次挂载崩在 txg 1 的记录之后 ⇒ 空池挂载放行、`publish_first_file` 写死的 txg 3 盖在暖机根上，返回成功的文件版本冷恢复读不到

历史（模型 `opus_attack_emptypool.rs:213` `z3_first_file_after_two_superblock_faults_lands_under_a_newer_empty_root`）：

1. mkfs 之后第一次可写挂载，崩在「取号两写 + txg 1 的记录两写」持久、txg 1 的根槽没持久（段 0、段 1 整段持久、段 2（txg 1 的根槽）没持久，段号从 0 数、与 4.2 节同；D13（验证路线） 已定项 4 的切段允许的崩溃状态）。
2. 两块盘超级块槽 0（取号写进号 1、世代号 2 的那一槽）各坏一个字节——两个介质故障。择超级块只剩 mkfs 的槽 1（世代号 1、号 0），根环只有三份第 0 代根。模型原样：`Z3F after faults: chosen superblock slot_generation=1 journal_instance=0 readable_roots=[(0, 0), (0, 0), (0, 0)]`。
3. 重开可写挂载：`transaction.rs:287` 起算出要取的号 1；`mount.rs:778` 区间 [1, 1) 为空、放行；`mount.rs:916` `let next_counter = records` 与 `mount.rs:922` `let first_txg = first_txg_of_new_instance(devices, &superblock, &records);` 因环里那条 txg 1 的记录得 jsn 2、txg 2；写行零单元发布 txg 2（区域 2、盘 0），`mount.rs:852`–`858` 的暖机循环推到覆盖两块盘：txg 3（区域 0、盘 0）、txg 4（区域 1、盘 1）。原样：`Z3F remount -> Ok(instance 1 row_txg 2 warm_ups 2)`、`Z3F row_publish (inst 1, txg 2, jsn 2); warm-ups [(3, 3), (4, 4)]; current (txg 4, jsn 4)`。
4. 照用例 `second_transaction_step_three_formatted_pool.rs` 的调法接 `publish_first_file(…, mounted.current.root(), …, mounted.current.record_bytes())`：`transaction.rs:956` `let txg = CheckpointTxg(FIRST_TRANSACTION_TXG);`、`:962` `counter: FIRST_TRANSACTION_TXG,` ⇒ 根 (1, 3) 写进暖机 txg 3 那个根槽、记录写进 jsn 3 那个记录槽，返回 `Ok`。原样：`Z3F publish_first_file returned Ok: root (inst 1, txg 3), jsn 3`。根槽 FUA 已返回，按 `.claude/kb/decisions/16-发布语义.md:288` 整行 `fsync 等根槽持久之后才返回。恢复重放在施加任何记录之前，必须逐项验证点名单元的校验和。**`，这一版可以对用户确认了。
5. 冷恢复两种政策都落在 txg 4 的暖机根上、没有文件；checker 没有一条违例（非「成立」的只有两条不适用）。原样：

```
Z3F cold recovery (Consult) -> NoFile { root: (InstanceGeneration(1), CheckpointTxg(4)) }
Z3F cold recovery (Ignore) -> NoFile { root: (InstanceGeneration(1), CheckpointTxg(4)) }
Z3F checker I-3.1 NotApplicable("最新的根下面还没有记账树（第 0 代树表）")
Z3F checker I-5.2 NotApplicable("最新的根下面还没有记账树（第 0 代树表）")
```

四句：

| 问 | 答 |
|---|---|
| 分不分辨臂 | 分辨：`publish_first_file` 写死 txg / jsn 3（今天）与「第一个文件版本取现行那一版的 txg + 1、jsn + 1」两条臂——后者写 (1, 5)，是最新根、冷恢复读得到；也分辨「空池放行」与「只放行与第一个事务同形的那一格、其余拒绝」（3.3 节） |
| 系统当时看不看得到判别它的东西 | 看得到：挂载时 `start.first_txg` = 2、`start.next_counter` = 2 在取号之前就算出来了；调 `publish_first_file` 时 `mounted.current` 的 txg 4、jsn 4 在手里，函数不看 |
| 满足判据字面的哪一个分句 | 触发句「txg 不是 3 时走进这条路」：中（现行那一版是 txg 4，还是走进了写死 txg 3 的路）；问句分句「零单元暖机次数不是 2」：中（零单元发布 3 次：txg 2、3、4）。可达性要两个介质故障加一次崩溃 |
| 跑前条款的改法在这几格上还中不中 | 「代码按最保守的读法改（拒绝优先于猜）」：3.3 节的守卫在副本上让第 3 步返回 `Err`、不再往下走——这一格不中（只在我的模型上量过、被攻过零轮） |

### 3.2 打中 B：mkfs 接受区域归属 [0, 1, 1] / [1, 0, 0]，零故障走到两条不同的 (1, 3) 根

条款 `.claude/kb/decisions/02-RAID条带策略.md:263` 整行：`**第一版写死 0 / 1 / 0（2026-09-14，用户定案）**：区域 0 → 盘 0、区域 1 → 盘 1、区域 2 → 盘 0，mkfs 逐区域写进超级块那 12 字节（每区域 4 字节设备身份）。`；同文件第 58 行索引表那一格写「2026-09-14 用户定案：不是 mkfs 参数」。代码：`crates/singlefs-core/src/make_filesystem.rs:147` `for (region, device) in parameters.region_devices.iter().enumerate() {` 只核区域指的盘在不在池里，归属照 `MakeFilesystemParameters.region_devices` 收。**所以 B 是「代码可达、条款排除」**，比 A 弱一档，照实分开报。

模型 `opus_attack_emptypool.rs:557`：四种归属各做 mkfs → 可写挂载 → `publish_first_file`，数根槽写、冷恢复、18 写那一段不展开的层 0 快枚举（与 `second_transaction_step_three_formatted_pool_layer0.rs` 的快用例同一个 `enumerate_layer0_selecting_versions`、版本表只登 (1, 3)）。⚠️ 模型里第一个文件版本用的分配器与上一版是从盘上最新根与最新记录重拼的（`mounted` 被打印函数拿走了），起点与 `format_time_allocator` 相同，没与 `mounted.allocator` 逐字比。`run-all.log` 原样两行：

```
Z3R regions=[0, 1, 0] mount=Ok(instance 1 row_txg 1 warm_ups 1) newest_before_file=(1, 2) jsn 2 first_file=(1, 3) jsn 3 root_writes(index, instance, txg)=[(4, 1, 1), (9, 1, 2), (30, 1, 3)] segments=[2, 2, 1, 2, 2, 1, 18, 2, 1, 2] recover="FileRead(InstanceGeneration(1), CheckpointTxg(3)) content_ok=true" fast_states=22 oracle_violations=0 first=None ignored_violations=0 record_checker=(0, 0) checker_violations=[]
Z3R regions=[0, 1, 1] mount=Ok(instance 1 row_txg 1 warm_ups 2) newest_before_file=(1, 3) jsn 3 first_file=(1, 3) jsn 3 root_writes(index, instance, txg)=[(4, 1, 1), (9, 1, 2), (14, 1, 3), (35, 1, 3)] segments=[2, 2, 1, 2, 2, 1, 2, 2, 1, 18, 2, 1, 2] recover="FileRead(InstanceGeneration(1), CheckpointTxg(3)) content_ok=true" fast_states=29 oracle_violations=7 first=Some("第 3 代根下面有文件却报没有（持久的写：superblock_slot|superblock_slot|journal_record|journal_record|root_record_fua|superblock_slot|superblock_slot|journal_record|journal_record|root_record_fua|superblock_slot|superblock_slot|journal_record）") ignored_violations=4 record_checker=(1, 3) checker_violations=["I-7.8=4:根环水位最大 11，盘上出现过的最大树 ID 15"]
```

（`[1, 0, 0]` 那一行除 `regions=` 外与 `[0, 1, 1]` 逐字段相同，`[0, 0, 1]` 与 `[0, 1, 0]` 逐字段相同，见日志。）

读法：[0, 1, 1] 下 txg 1、2 都落盘 1，暖机推到 txg 3 才覆盖盘 0，现行那一版已是 (1, 3)、jsn 3；`publish_first_file` 又写 (1, 3)、jsn 3——根槽写里 (1, 3) 出现两次（写表下标 14 与 35），同一个 (实例, txg)、同一个 jsn 两份内容不同。层 0 快枚举里 checker 在 4 个状态上判 I-7.8（树 ID 水位）红、记录核对器「根在案而记录缺席」1 次、「恢复自称新态而单元缺席」3 次；oracle 那 7 条里至少一部分是版本表按 (实例, txg) 认、分不清两条 (1, 3) 造成的（oracle 自己的口径，不算实现的红），checker 与记录核对器那 8 次不依赖版本表。

四句：

| 问 | 答 |
|---|---|
| 分不分辨臂 | 分辨写死 txg 3 与「取现行 + 1」（后者写 (1, 4)，没有身份碰撞）；也分辨「mkfs 拒收非 0/1/0 的归属」（只修 B、修不了 A） |
| 系统当时看不看得到判别它的东西 | 看得到：`mounted.current.root().checkpoint_txg` = 3；挂载前 `parameters.region_devices` 就能算出零单元发布要几次 |
| 满足判据字面的哪一个分句 | 触发句「txg 不是 3 时走进这条路」：中（该写 txg 4）；问句「零单元暖机次数不是 2」：中（3 次）。前提是条款排除的归属 |
| 跑前条款的改法在这几格上还中不中 | 3.3 节守卫：副本上挂载返回 `Err`，不中（只在我的模型上量过、被攻过零轮） |

### 3.3 我给 A、B 两格提的改法（只在我的模型上量过、被攻过零轮）

`copy-only-guard-for-z3.diff`（21 行，打在副本的 `crates/singlefs-core/src/mount.rs` 上，量完已还原、还原后 sha256 与主树相同）：在 `establish_instance` 里 `refuse_instance_rows_on_version_without_file` 之后、`acquire_instance` 之前，树表 0 条的上一版只放行「新实例从 txg 1、jsn 1 起，且 txg 1 与 txg 2 落在不同的盘上」那一格，其余返回 `Err`（借用了 `VersionWithoutFileNotWrittenByMakeFilesystem` 这个成员，字段里塞的是 first_txg 与 next_counter——只为量，不是提议的错误形态）。

副本上量到的（打着守卫跑，之后还原）：

| 用例 | 结果 |
|---|---|
| 模型 A 那条 | 原样 `Z3F remount -> Err(VersionWithoutFileNotWrittenByMakeFilesystem { root: RollbackTarget { instance: InstanceGeneration(0), checkpoint_txg: CheckpointTxg(0) }, instance_table_birth_txg: CheckpointTxg(2), tree_table_birth_txg: CheckpointTxg(2) })`，用例在 `expect("没拒绝")` 处 panic（预期内：挂载被拒了） |
| 模型 B 那条 | `[0, 1, 0]` 照常放行、快枚举 22 个状态零违例；`[0, 1, 1]` 在 `expect("可写挂载")` 处 panic，即挂载返回 `Err` |
| `second_transaction_step_three_formatted_pool`、`…_layer0`（快的两条）、`second_transaction_step_four_rollback` | 依次原样：`test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.39s`、`test result: ok. 2 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.46s`、`test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.07s` |

没量的：守卫之后「在写之前」由构造保证（位置在第 779 行取号之前），没拿 `DiskSnapshot` 比；步 5 的用例、层 0 全量都没跑。守卫只挡住 A、B 这两格，挡不住第二节的瞬时读错（那一格的号在守卫之后才被第二次算错）。另一条路「`publish_first_file` 改成取现行那一版 + 1」会改第一个事务的 txg 3（格式常量 `FIRST_TRANSACTION_TXG = 3`，`.claude/kb/layout/01-first-txn.md` 八那两行标记），没做。

### 3.4 Z3 没打中的形状

| 形状 | 取样 | 结论 |
|---|---|---|
| 第一个文件版本换下 mkfs 第 0 版树表时，两条路的释放、defer、记账、录制流 | 已有用例断言录制流与 `TransactionOutput` 相等；模型 `opus_attack_emptypool.rs:517` 再往后比：第一个文件版本之后两条路的分配器（`Debug` 全文，含只住内存的开放段、游标、回收集、`format_time_tree_table`）、各再覆盖写两次之后的录制流、分配器、现行那一版 | 原样 `Z3N allocators_equal_after_first_file=true streams_after_mkfs_equal=true allocators_equal_after_two_overwrites=true outputs_equal=true`，没打中 |
| 根环里有 txg > 0 的根、两盘超级块代号不一的池走进零单元写行那一支 | 按代码推：要写的行为空 ⟺ 要取的号 = 1 ⟺ 超级块与根环里都没有号 ≥ 1；实例 ≥ 1 的根一条可读就取 2 并拒绝；第 4.2 节表里两盘号不一的两个状态都被拒 | 零故障下走不进；A 那格要两个超级块槽坏 |
| 根环里有 txg > 0、实例 0 的根 | 读写者：mkfs 只写 txg 0 的第 0 代根，别的发布都带取到的号 ≥ 1 | 走不到 |
| 在同一进程里抬 F | 读 `raise_rollback_floor`：`current` 是 `publish_first_file` 的输出，`units` 里没有实例表单元 ⇒ `RaiseNeedsWritableMountInThisProcess`（这个进程其实做过可写挂载，名字与事实不符；与 mkfs 同进程跑第一个事务那条路同一个形态） | 在写之前拒绝，不算打中；名字不符交回 |

## 四、Z4 层 0 第三条流

### 4.1 问句的后一半坐实：第三条流枚举的崩溃镜像与第一条流逐个相同

`second_transaction_step_three_formatted_pool_layer0.rs:50` `let base = formatted.memory_pool_after_mkfs();` 与第 53 行 `writes_and_segments(&operations[formatted.mkfs_operation_count..], &geometry());`，对照 `first_transaction_step_seven_layer0.rs:124` 与第 127 行同形。模型 `opus_attack_emptypool.rs:149` 把两边的基线镜像、写表（带内容）、段三样直接比，原样：

```
Z4 base_equal=true writes_equal=true segments_equal=true writes=33 write_bytes=386560 segments=[2, 2, 1, 2, 2, 1, 18, 2, 1, 2]
```

每个状态做什么：`crates/singlefs-harness/src/crash.rs:687` `let consulted = recover(&image, JournalPolicy::Consult);`、第 688 行 `Ignore` 那一遍、第 739 行 `for (invariant, verdict) in check_pool_image(&image) {`、之后记录核对器——没有一处重开挂载。重开与挂载只在准备那一步（`prepare`）里对**整条**流做一次。于是第三条流的 262165 个状态就是第一条流那 262165 个镜像，判法少了第一条流的逐条「评估过的状态数」钉值（第三条流只要求 5 条 > 0）；它证明的是「挂载路径写出的字节与第一个事务相同」（这一点已由 `second_transaction_step_three_formatted_pool.rs` 的流相等断言单独钉住），不是「挂载路径在崩溃之后怎样」。

触发句「一个崩溃状态按恢复语义不合法而 26 条全绿或不适用」：**没中**——状态与第一条流相同，恢复与 checker 的判定也相同；第一条流全量零违例是 `first_transaction_step_seven_layer0.rs` 的断言（门禁 54 号在 release 下跑它），我这一轮没跑全量。

### 4.2 在同一批崩溃状态上补跑可写挂载（层 0 不跑的那一步）

模型 `opus_attack_emptypool.rs:168`：按层 0 的枚举形状取状态（前面的段整段持久 + 当前段任意真子集，最后全持久），< 10 写的九段全部展开（22 个），18 写那一段取 24 个确定掩码（`(index × 10007 + 3) mod (2^18 − 1)`），每个状态建一对内存盘重开走 `mount_writable`。`run-all.log` 里 46 行 `Z3STATE`，归并如下（段号从 0 数，段序列 `2+2+1+2+2+1+18+2+1+2`）：

| 状态 | 挂载结果（原样归并） |
|---|---|
| 段 0 掩码 0（只做过 mkfs） | `Ok(instance 1 row_txg 1 warm_ups 1)` |
| 段 0 掩码 1、2（取号只落了一块盘，两盘超级块号不一） | `InstanceRowsOnVersionWithoutFileUnsupported(chosen (0, 0), rows [1, 2))` |
| 段 1 掩码 0（取号两写都持久，没有记录、没有根） | `InstanceRowsOnVersionWithoutFileUnsupported(chosen (0, 0), rows [1, 2))` |
| 段 1 掩码 1、2；段 2 掩码 0 | `InstanceRowsOnVersionWithoutFileUnsupported(chosen (0, 0), rows [1, 2))` |
| 段 3 三个；段 4 掩码 0 | `InstanceRowsOnVersionWithoutFileUnsupported(chosen (1, 1), rows [1, 2))` |
| 段 4 掩码 1、2；段 5 掩码 0；段 6 取样 24 个；段 7 掩码 0 | `InstanceRowsOnVersionWithoutFileUnsupported(chosen (1, 2), rows [1, 2))` |
| 段 7 掩码 1、2；段 8；段 9 三个；全持久 | `Ok(instance 2 row_txg 4 warm_ups 1)` |

原样合计：`Z3TALLY {"InstanceRowsOnVersionWithoutFileUnsupported": 38, "Ok": 8}`。按代码推全量（段 6 其余 2^18 − 1 − 24 个掩码没跑）：段 6 的子集里 txg 3 的记录与根都没持久，所选根恒为 (1, 2)、树表 0 条、要取 2，恒被拒 ⇒ 262165 个状态里 262157 个被拒、8 个放行。这 8 个放行的全在「txg 3 的记录至少一份持久」之后。

### 4.3 「8 条不适用」会不会掩住违例

- 第三条流的状态与第一条流相同，这一问两条流答案相同。第一条流用例钉死那 8 条只在 4 个「txg 3 根槽已持久」的状态上评估；段 7 掩码 1、2 与段 8 掩码 0 这 3 个「记录已持久、根没持久」的状态上，恢复靠 journal 给出带文件的 (1, 3)，checker 却按最新根槽 (1, 2) 判、8 条不适用。按代码推：那 3 个状态恢复出来的版本指的就是根槽持久那 4 个状态里 checker 判过的同一个树表单元与同一批单元，结构违例在那 4 个状态上会红；只有根记录自己的字段（F、树 ID 水位、实例表指针）在那 3 个状态上由记录给出、checker 不看——没造出违例，不算打中。
- 3.1 节 A 那份镜像不是崩溃状态，但它给了一个「checker 零违例而返回过成功的版本读不到」的镜像：没有一条不变量说「最后确认的那条根要是择根序里最新的」，checker 只看一份镜像、看不到确认过什么；这不是 8 条不适用造成的，是那份镜像上 I-3.1、I-5.2 以外的 24 条都判成立。

## 五、V2 射程观测（不落在 Z1–Z4 任何一条的触发句上，交回主 agent）

模型 `opus_attack_emptypool.rs:677`：只做过 mkfs 的池可写挂载成功，不发第一个文件版本，进程正常退出；再重开两次。`run-all.log` 原样：

```
V2 first mount -> Ok(instance 1 row_txg 1 warm_ups 1)
V2 remount attempt 0 -> InstanceRowsOnVersionWithoutFileUnsupported(chosen (1, 2), rows [1, 2))
V2 remount attempt 1 -> InstanceRowsOnVersionWithoutFileUnsupported(chosen (1, 2), rows [1, 2))
V2 cold recovery -> NoFile { root: (InstanceGeneration(1), CheckpointTxg(2)) }
```

这与 `second_transaction_step_three_formatted_pool.rs` 里 `writable_mount_after_a_crash_between_warm_up_and_the_first_file_is_refused_before_acquiring_an_instance` 是同一个盘面，那条用例与正文都把它叫作「崩溃」；实际上不需要崩溃——挂上、什么都不写、进程正常退出就进这一格，之后每一次可写挂载都被拒。加上 4.2 节：第一次可写挂载在取号那两写里崩一次（盘上没有根、没有记录，只多了一个号）也进这一格。

许可它的（两处都只指路、括号里是我的转述，整行见原文；D23 那一段整段也在背景材料附录里）：`.claude/kb/decisions/23-journal的角色与格式.md:1209`（D23（journal 的角色与格式） 已定项 14 管理员回退那一段，讲取号留在盘上的新号让下一次取号跳过一个号、被跳过的号按 D18（块里携带什么信息） 已定项 11 的写行规则得一行，句末带 2026-09-14 定案出处）；`.claude/kb/decisions/18-块里携带什么信息.md:882`（D18 已定项 11 实例表那一行，含加粗的「每次可写挂载都写行」及其括注「实例 0 不写」等，该行 11469 字节）——号 1 要一行，而树表 0 条的一版上写行没有条款、V3 按「拒绝优先于猜」拒。所以 V2 的「只做过 mkfs 的池（上一个实例是 0）可写挂载」今天只在「超级块从没被取号写过」那一个盘面上成立。这是 V2 / V3 的射程问题，不是代码与判据不符；要不要给「号被取过、而这个号一条根一条记录都没写出」的池一条不写行的路，是条款层的问题。

## 六、没打中的形状（汇总）

| 格 | 试过的形状 | 取样范围 |
|---|---|---|
| Z1 | 同内容写回、落点复用到同槽、空发布 / 写行 / 回退改指针字节、最旧有效根与「两棵树都没有」比、被抛弃根夹在中间、两次回退、不经 F 回落的合法复用 | 模型量 1 格（同内容两次覆盖写，txg 4 / 5）；其余按代码推（1.2 节逐条写了依据），两次回退没建模 |
| Z2 | `establish_instance` 之前的写、两盘超级块持久不一致、一盘槽 0 持久坏、回退与抬 F 拒绝之前的副作用 | 两盘号不一的 2 个崩溃状态实跑（4.2 节）；持久坏槽下两次算号相同由第 2.1 节对照行（不注入 / 只在第 1 次读注入）实跑；其余读代码 |
| Z3 | 两条路的第一个文件版本之后再覆盖写两次的字节、分配器内存态；零故障下根环有 txg > 0 的根或号不一的池走进零单元那一支；同进程抬 F | 模型量 1 条脚本（覆盖写两次）；零故障可达性按代码推加 4.2 节 46 个状态实跑 |
| Z4 | 崩溃状态上 checker 8 条不适用掩住违例；零单元发布崩在记录与根之间恢复出什么 | 第三条流 22 个快状态（与第一条流相同）由已有用例覆盖，我只读了判定代码与第一条流的计数钉值；层 0 不模拟撕裂写（`crash.rs` 第 1–2 行：撕裂态并进「没持久」），「根写到一半」在这套装置里等于没写 |

## 七、这条腿自己的限度

- **全部数都是副本上的**：副本与主树 `crates/` 零差异（开工时比过），模型是我写的、被攻过零轮；四处打中都没在入库装置上重做。
- 打中的故障模型各不相同，强度要分开看：Z3-A 要两个超级块槽的介质故障加一次崩溃；Z2 要设备对同一个扇区先报瞬时读错、再读成功（`FileBackedBlockDevice` 与本仓的内存盘都不会自己这样，我包了一层注入）；Z3-B 要一个条款排除、代码不核的 mkfs 参数；Z1 要一个根槽坏一字节（与已有用例 `one_device_carrying_the_floor_alone_does_not_take_effect_on_remount` 同一个故障）。只有第五节的 V2 射程观测是零故障零崩溃，而它不落在判据的触发句上。
- 4.2 节「262157 / 262165」是 46 个实跑状态加代码推理外推的，18 写那一段只取了 24 个掩码。
- Z3-B 的 oracle 7 条违例里有多少是版本表按 (实例, txg) 认造成的，我没逐状态拆；我只采信 checker 与记录核对器那 8 次。
- 3.3 节守卫只在副本上打过、跑了 5 个用例文件里的快用例（我的模型 2 条 + 已有 3 个文件），没跑步 5 的用例、没跑层 0 全量、没做变异；它借用了一个语义不对的 `MountError` 成员。
- 我没有读 kb 里 F 回落重议那一轮（`alloc-basis-r3-main-verification.md`）的判决，Z1 那格与它的关系只按 D16 已定项 1 表格后「生效」那段 ⚠️ 的正文判。

什么现象会推翻各格的结论：

| 格 | 推翻它的观测 |
|---|---|
| Z1 打中 | 入库装置上同一段历史（改坏 txg 16 根槽后重开、再抬 F）抬 F 成功，或拒绝点名的不是 F 回落之前已在 F 之下的根 |
| Z2 打中 | `acquire_instance` 改成不重读盘（或与判定共用同一次读）之后，同样的注入不再 panic；或证明块设备层保证读错不会瞬时恢复（那样这一格不可达） |
| Z3-A 打中 | 入库装置上两盘槽 0 各翻一字节、崩在 txg 1 记录之后重开，挂载被拒，或 `publish_first_file` 之后冷恢复读到文件 |
| Z3-B 打中 | mkfs 拒收非 0 / 1 / 0 的归属（那样代码也不可达），或 [0, 1, 1] 下快枚举 checker 与记录核对器都是 0 |
| Z4 问句后一半 | 两条流的 `prepare` 给出不同的基线或写表（例如固定写入时间或 fsid 不同） |

## 八、没做什么

- 没碰 Z6（本地攻方的格），没判正推腿的 Z5 / Z7。
- 没改原仓任何已有文件、没做 git 写操作；只新建了 `research/prompts/m2-emptypool-nonempty-r1-opus-output.md` 与 `research/prompts/m2-emptypool-nonempty-r1-opus-model/` 下三个文件（其中两个在本轮里删掉旧版重拷过一次，都是本轮自己建的）。
- 没跑 release、没跑层 0 全量枚举、没跑门禁任何阶段（定义没把阶段登记给这条腿）；没读别的腿的输出与主 agent 的核实。
- 没在入库装置上重做任何一格；没实现、没量「`publish_first_file` 取现行 + 1」「取号收期望号」「树表读不出的根不算有效」这三个改法。
