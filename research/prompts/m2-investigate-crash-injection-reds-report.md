# 调查报告：崩溃注入快档三条红（investigator）

## 〇、现场

- 副本取于 2026-09-27（`rsync -a --exclude /target --exclude .git` 主工作区，另删了副本里带过来的 `research/target`），HEAD `73ba4a4c019b9e3fc9c92f3122bfbbdaee93c321`。
- 那一刻 `git diff --stat -- crates` 的输出存为 `diffstat-at-copy.txt`，sha256 `7fc06c0df0c5edac2bfae89aab981f7075aaafcaca88b4e3b616f6ebd1011609`（末行 `85 files changed, 22097 insertions(+), 10583 deletions(-)`）；整份 `git diff -- crates` 存为 `diff-crates-at-copy.patch`，sha256 `ea8a6d7a9ddfbcab652c3afa96c00e513f7d9c5fe21eb5221346c973199aa32d`。都在草稿目录根。
- 下文引的行号都是**这份快照**的行号（`crash.rs`、`walk.rs` 的原件留在草稿目录 `crash.rs.orig`、`walk.rs.orig`）；主工作区此后有 A1b、B3a-2 等在改，行号可能已漂。
- 全部跑在副本上：release，`bash research/scripts/run-with-memory-cap.sh 16G bash research/scripts/capped.sh 4 cargo test --release --offline -p singlefs-harness --test …`，`TMPDIR` 指到草稿目录。开跑前 `ps` 没有 qemu / vm-bench / e152 / fio / cargo。

## 一、结论一览

| # | 判为 | 定位 | 从实八到今天为什么红 |
|---|---|---|---|
| 1 `claimed_state_missing_unit`（第一截，种子 …115 第 56 段） | **装置错**（记录核对器在崩溃注入这一路的入参取法；不是实现错、不是池级 checker） | `crash_injection.rs:883-886` 只把**到当前段为止的写表前缀**交给 `check_records_against`；`crash.rs:848-858` `instance_table_of_the_effective_root` 只在这份前缀里找「恢复落到的那一版」的根槽写，那一版由 journal 记录重建、根槽写在**下一段**（#357 在第 57 段），`.find(...)?` 落空 → 实例表 `None` → 实八加的「被抛弃时间线不要求单元在」豁免（`crash.rs:810`、`:824`）整条关掉 → 被抛弃的 (1, 4) 那个已被合法复用的数据单元判缺席 | **A1（审阅第 19 条）在 `transaction.rs` 加的三道屏障（快照 :441、:544、:687）改了段序列**（这段历史 73 段 → 77 段，快档 96 个崩溃状态 → 94 个），同一个种子抽到的 4 个崩溃状态换了一批，抽中了这一形。缺陷本身在 A1 之前就在：去掉三道屏障、手摆同一个持久集合（那时是第 52 段）照样红。B1 与这条无关（红在读任何实例表之前） |
| 2 I-3.1（第二截，种子 …136 第 91 段，可写挂载 + 一次发布之后） | **checker 口径错**（`walk.rs` 走「由记录施加出来的一版」时不走实例表指针，条款没写的那一格；实现的记账与回收谓词自洽） | `walk.rs:645-650` `walk_version_applied_only_by_records` 只走树表与映射根；`walk.rs:641-644` 的文档注释自己写着「实例表指针……那一版不在候选集里……这两样这里就漏数了：这一格条款没写」。差的 32768 字节就是 mkfs 写的第 0 片实例表（两盘各在槽 50176，跨 2 槽）：它由实例 1 的根 (0,0)…(1,4) 与由记录施加出来的 (1, 3) 引用，(2, 5) 写行时换下、释放代 5；环里只剩 (1, 2) 还指着它，而 (1, 2) 低于 F 3（`walk.rs:4937-4938` 滤掉），(1, 3) 走了但没走它的实例表 | 新截是 B3b 今天加的，以前没跑过这一格。同一个崩溃状态在去掉 A1 屏障的副本上手摆、挂载、发布，**同样的数照样红**；A1 只改了它被不被抽中。这段注释在 HEAD（2026-09-25 的提交）里就在（`git show HEAD:crates/singlefs-checker/src/walk.rs` 第 535–538 行），早于 B1 |
| 3 I-3.1（第三截，同一崩溃状态，二次崩溃崩在暖机） | 同第 2 条（checker 口径错） | 同第 2 条：同一个单元（槽 50176 释放代 5），这一版环里 (1, 4) 被实例表判抛弃、(1, 2) 低于 F，由记录施加出来的 (1, 3) 照样没走实例表 | 同第 2 条。它的签名只有 I-3.1，没有模型对不上，B3b 第二节第 7 条「允许的版本」那一取法不在这条路上 |

三条之外顺带看到的（第五节）：种子 …129 第 17 段那一形（C554，「冷启动读回」MappingStillUnreadable 50240）在今天的代码上**还在**，只是因为同一个段序列变化没被抽中；它在快档里消失不是修好了。

## 二、第 1 条

### 复现（副本，快档原样）

命令：`TMPDIR=… CARGO_TARGET_DIR=… bash research/scripts/run-with-memory-cap.sh 16G bash research/scripts/capped.sh 4 cargo test --release --offline -p singlefs-harness --test second_transaction_supplement_three_crash_injection -- --exact crash_injection_fast_tier_recovers_only_into_versions_the_model_committed --nocapture`，日志 `logs/fast-tier-copy.log`，原样：

```
崩溃状态上 checker 跑了 94 次；记录核对器跑了 94 次：根在而记录一条都不在 0 次、恢复自称新态而单元缺席 5 次
崩溃状态上的新发现 RecordCheck { aspects: ["claimed_state_missing_unit"] }（crash_image）：种子 7463871032432355115，第 56 段（2 个写，持久 1 个，扣下段内第 [0] 个），落在 Operation(14)／Some(CrashRecoveryAbandoningTheNewestRoot)
崩溃状态上的新发现 CheckerViolations { invariants: ["I-3.1"] }（after_the_writable_mount_and_one_publish）：种子 7463871032432355136，第 91 段（2 个写，持久 1 个，扣下段内第 [1] 个），落在 Operation(21)／Some(PublishOverwrite)
崩溃状态上的新发现 CheckerViolations { invariants: ["I-3.1"] }（second_crash_inside_warm_up）：种子 7463871032432355136，第 91 段（2 个写，持久 1 个，扣下段内第 [1] 个），落在 Operation(21)／Some(PublishOverwrite)
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 5.85s
```

三条与 B3b 报告第四节逐字相同（I-3.1 两行的数也相同：4915200 / 4882432、4521984 / 4489216）。

### 最小复现（一条命令）

`SINGLEFS_CRASH_INJECTION_FIRST_SEED=7463871032432355115 SINGLEFS_CRASH_INJECTION_SEEDS=1 SINGLEFS_CRASH_INJECTION_OPERATIONS=24 SINGLEFS_CRASH_INJECTION_POINTS=4` 加上面那串包装，跑 `-- --exact crash_injection_large_tier_from_the_environment --ignored --nocapture`，日志 `logs/large-115-plain.log`，原样：

```
崩溃状态上 checker 跑了 4 次；记录核对器跑了 4 次：根在而记录一条都不在 0 次、恢复自称新态而单元缺席 1 次
崩溃状态上的新发现 RecordCheck { aspects: ["claimed_state_missing_unit"] }（crash_image）：种子 7463871032432355115，第 56 段（2 个写，持久 1 个，扣下段内第 [0] 个），落在 Operation(14)／Some(CrashRecoveryAbandoningTheNewestRoot)
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.75s
```

（副本里的 `crash.rs`、`walk.rs`、`crash_injection.rs` 这时已加了诊断，全部挂在 `DIAG_*` 环境变量后面，这条命令没设，走的是原路径；补丁在 `kept/`。）

### 机理（诊断用例 `kept/zz_diag_investigation.rs`，按种子与段号重建同一个崩溃状态；日志 `logs/diag115b.log`）

这段历史（种子 …115，24 步）第 14 步是 `CrashRecoveryAbandoningTheNewestRoot`，挂出实例 5（写行 (5, 14)、之后 (5, 15)、(5, 16)）。段序列（原样）：

```
SEG 56: ["#355 JournalRecord dev0 off=16834560 len=4096", "#356 JournalRecord dev1 off=16834560 len=4096"]
SEG 57: ["#357 RootRecordFua dev0 off=1069056 len=512 root=(InstanceGeneration(5), CheckpointTxg(15))"]
```

崩溃状态是第 56 段扣下 #355、留 #356。诊断输出（原样）：

```
RECOVERY effective_root=Some((InstanceGeneration(5), CheckpointTxg(15)))
DIAG_RECORDS missing: publish=(1, 4) effective_root=Some((InstanceGeneration(5), CheckpointTxg(15))) offset=822181888 copies=[60, 61] lens=[32768, 32768] devices=[0, 1] root_write_index=86 table=None
RECORD_CHECK RecordCheck { root_without_record: false, claimed_state_missing_unit: true }
ROOT_WRITE_OF_EFFECTIVE in_prefix=false index_in_full_stream=Some(357)
TABLE_OF_EFFECTIVE_FROM_FULL_STREAM Some(InstanceTableOfRootRecord { rows: [InstanceTableRow { instance: 1, published_checkpoint_txg: 3, applied_transaction_high_water_mark: 0 }, InstanceTableRow { instance: 2, published_checkpoint_txg: 9, applied_transaction_high_water_mark: 0 }, InstanceTableRow { instance: 3, published_checkpoint_txg: 11, applied_transaction_high_water_mark: 0 }, InstanceTableRow { instance: 4, published_checkpoint_txg: 12, applied_transaction_high_water_mark: 0 }] }) abandons(1,4)=Some(true)
RECORD_CHECK_WITH_THE_FULL_STREAM RecordCheck { root_without_record: false, claimed_state_missing_unit: false }
```

逐步：
1. 恢复施加了 #356 那份记录（`prefix_applied=1`），落到 (5, 15)；(5, 15) 的根槽写 #357 在第 57 段，不在第 56 段的崩溃状态里持久，也**不在交给记录核对器的写表前缀里**（`crash_injection.rs:813-814` `writes_up_to_this_segment = &writes[..first_write_of_the_segment + writes_in_the_segment]`，`:883-886` 把它交给 `check_records_against`）。
2. `crash.rs:848-858` `instance_table_of_the_effective_root` 在交进来的 `writes` 里倒着找根身份等于 (5, 15) 的根槽写，`.find(...)?`（:858）落空，交回 `None`。它文档注释 :845 写的前提「根槽没落盘、由记录重建的那一版也在记录流里」在层 0 那一路成立（`crash.rs:2055` 每个状态的持久集合按整条写表长 `writes.len()` 建，写表是整条流），在崩溃注入这一路不成立。
3. 表是 `None` ⇒ `crash.rs:810` 的 `abandoned_by_the_landed_version` 对每次发布都是假 ⇒ :823-824 要求 (1, 4)（txg 4 ≤ 15）的单元在；(1, 4) 的数据单元（偏移 822181888）已被后来的发布复用盖掉 ⇒ :837 判红。
4. 把同一个状态的写表换成整条流（后面的写都记成没持久）：找得到 #357，读出的实例表有行 (1, 3)，`abandons(1,4)=Some(true)`，记录核对器一条不红。

### 排除掉的解释

| 解释 | 凭哪条观测排除 |
|---|---|
| 实现错：恢复不该落到 (5, 15)，或 (1, 4) 的单元不该被复用 | 同一状态上模型没有对不上、池级 checker 一条不红（诊断用例对崩溃镜像跑 `check_pool_image`，`logs/diag115b.log` 里没有 `CHECKER` 行；快档这一行的新发现签名只有 RecordCheck）；落到的那一版的实例表按条款判 (1, 4) 被抛弃（上面第 4 步） |
| 池级 checker 口径（B1 判严） | 红在读任何实例表之前：`in_prefix=false`，`None` 来自 `.find(...)?`，不是 `InstanceTableOfRootRecord::read` 读不出；池级 checker 在这条路上没参与判定 |
| 实八的豁免判据本身写错 | 整条流喂进去就不红，且判别力还在（见「推翻条件」一节的造红） |
| 这个状态是 A1 之后才有的新盘面 | 去掉 A1 三道屏障的副本上，写表逐条相同（`writes=546`），同一对写在第 52 段，手摆同一个持久集合照样红（`logs/diag115-noa1.log`：`SEGMENT_OF_WRITE #355 = 52 ([355, 356])`、`RECORD_CHECK RecordCheck { root_without_record: false, claimed_state_missing_unit: true }`、`RECORD_CHECK_WITH_THE_FULL_STREAM … claimed_state_missing_unit: false }`） |

### 推翻条件与造它的那一次

- **什么会推翻「前缀取法让豁免失效」**：把同一次崩溃注入的记录核对器改喂整条写表（持久集合照这个崩溃状态、后面全记没持久），这一条还红；或者整条流喂进去之后记录核对器在「没被抛弃的发布缺单元」上不再红（那就是整条流把判据弄瞎了，不是修好了）。
- **造的两次**：
  1. 在副本的 `crash_injection.rs` 第一截上加一个开关 `DIAG_EXPERIMENT_FULL_STREAM`，打开时交整条写表（补丁 `kept/experiment-full-stream-crash_injection.rs.patch`），只开它跑快档（`logs/fast-tier-only-DIAG_EXPERIMENT_FULL_STREAM.log` 第 41 行原样）：`崩溃状态上 checker 跑了 94 次；记录核对器跑了 94 次：根在而记录一条都不在 0 次、恢复自称新态而单元缺席 0 次`；…115 那一行没了，第 2、3 条两行还在。
  2. 判别力：同一个崩溃状态、整条写表，再把 (5, 14) 的一个节点两份（#318、#319，偏移 827359232）都记成没持久，记录核对器照红，且这次读得出实例表、(1, 4) 被豁免、红的是 (5, 14)（`logs/diag115-discriminate-318,319.log` 原样）：`DIAG_RECORDS missing: publish=(5, 14) effective_root=Some((InstanceGeneration(5), CheckpointTxg(15))) offset=827359232 copies=[318, 319] …` 与 `DISCRIMINATE withheld=[318, 319] effective_root=Some((InstanceGeneration(5), CheckpointTxg(15))) record_check_with_full_stream=RecordCheck { root_without_record: false, claimed_state_missing_unit: true }`。（先试过扣 #316/#317，那是 (5, 14) 写的实例表片，扣了表读不出、豁免又关，不算干净的判别；扣 #339/#340 时恢复退到 (5, 14)，不要求 (5, 15)，也不算。两次都留在 `logs/`。）
- 同一个形状还有一处没验：`crash_injection.rs:1427-1428`、`:1461-1466` 第三截（二次崩溃）也是把挂载那一段的写表前缀交给 `check_records_against`；这一轮的读数里二次崩溃上没有记录核对器判红，没有造状态去验它。

## 三、第 2、3 条

### 复现

快档原样见第二节。单种子（`SINGLEFS_CRASH_INJECTION_FIRST_SEED=7463871032432355136`，其余同第 1 条的最小复现），日志 `logs/large-136-plain.log`，原样（去重之后）：

```
崩溃状态上的新发现 RecordCheck { aspects: ["claimed_state_missing_unit"] }（crash_image）：种子 7463871032432355136，第 91 段（2 个写，持久 1 个，扣下段内第 [1] 个），落在 Operation(21)／Some(PublishOverwrite)
崩溃状态上的新发现 CheckerViolations { invariants: ["I-3.1"] }（after_the_writable_mount_and_one_publish）：种子 7463871032432355136，第 91 段（2 个写，持久 1 个，扣下段内第 [1] 个），落在 Operation(21)／Some(PublishOverwrite)
崩溃状态上的新发现 CheckerViolations { invariants: ["I-3.1"] }（second_crash_inside_warm_up）：种子 7463871032432355136，第 91 段（2 个写，持久 1 个，扣下段内第 [1] 个），落在 Operation(21)／Some(PublishOverwrite)
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 1.39s
```

单跑时多出第一行：这个崩溃状态**第一截也是第 1 条那一形**（落到由记录重建的 (5, 26)，根槽写 #666 在下一段；诊断 `logs/diag136.log`：`ROOT_WRITE_OF_EFFECTIVE in_prefix=false index_in_full_stream=Some(666)`、`RECORD_CHECK_WITH_THE_FULL_STREAM … claimed_state_missing_unit: false }`）。快档里它被 …115 那一条按 (截, 签名) 去重吃掉了，所以快档报告看不到。

### 机理

在副本 checker 的 I-3.1 判定前加了一段诊断（`DIAG_I31`，补丁 `kept/instrumentation-walk.rs.patch`）：记账多于遍历时列出环里每条根、系统配置的 F、实例表行、由记录施加出来的版本，以及最新根的分配记录树里没被任何走过的版本引用的记录。第二截那一份池（诊断用例在同一崩溃状态上调快照里的 `mount_and_publish_once_on_the_crash_image`，只在副本里加了一个 `pub` 包装；日志 `logs/diag136-mount-a1-tables.log`），原样摘：

```
DIAG_I31 BEGIN allocated0=Some(4915200) walked0=4882432 newest_index=17 effective_floor=3
DIAG_I31 root[16] region=2 slot=0 txg=2 instance=1 F=0 candidate=false abandoned=false instance_table_page0=(dev0,slot50176)
DIAG_I31 record_applied_version instance=1 txg=3
DIAG_I31 record_applied_version instance=2 txg=5
DIAG_I31 record_applied_version instance=5 txg=26
DIAG_I31 unreferenced_record device=1 slot=50176 span=2 generation=5 released=true
DIAG_I31 unreferenced_record device=0 slot=50176 span=2 generation=5 released=true
CHECKER_AFTER_MOUNT I-3.1 Violated("盘 0：记账的已分配 Some(4915200)，遍历全部有效根得到 4882432（其中隔离豁免 0）；机理：根环槽数 24、最新根 txg 29、环里自证过的根槽 24 个、最老的自证过的根 txg 2、遍历的候选根槽 23 个、并进遍历的由记录施加出来的版本 3 个、被实例表判抛弃的根槽 0 个、回退下界 F 3、低于 F 的根槽 1 个")
```

（环里其余 23 条根的实例表第 0 片都不是槽 50176：实例 2 → 50304、3 → 50368、4 → 50432、5 → 50496、6 → 50688。）

那一个单元的来历（诊断用例每一步之后在活盘面上读最新根下槽 50176 那条分配记录，`logs/diag136-trace.log` 原样摘）：

```
STEP StartingPoint … TRACE newest=(1,3) F=0 … record_of_slot_50176=[(1, 50176, 2, 0, false), (0, 50176, 2, 0, false)]
STEP Operation(1) … TRACE newest=(1,4) F=0 … record_of_slot_50176=[(1, 50176, 2, 0, false), (0, 50176, 2, 0, false)]
STEP Operation(2) outcome=Some("Applied(Mounted { instance: InstanceGeneration(2), publishes: 3, … TRACE newest=(2,7) F=0 … record_of_slot_50176=[(1, 50176, 2, 5, true), (0, 50176, 2, 5, true)]
```

（五元组是 (盘, 槽, 跨度, 代, 已释放)。之后一直是释放代 5、已释放，直到历史末尾。）

读法：槽 50176 是 mkfs 写的第 0 片实例表（写表下标 #8（盘 0）、#10（盘 1），归在根 (0, 0) 那次发布里，`logs/diag136.log` 的 `OFFSET_WRITE` 两行），实例 1 的每条根都指着它；崩溃恢复抛弃 (1, 4) 之后实例 2 写行 (2, 5) 重写了实例表，把它换下、释放代 5。回收谓词（`walk.rs:4658-4660` 引的 D3（空间分配） 已定项 7：释放代 ≤ max(F 生效值, 环里最旧有效根)）下 5 > max(3, 2)，写者把它留在 defer、算在已分配里，与谓词自洽。checker 这边：
- (1, 2) 在环里、指着它，但 txg 2 < F 3，被 `walk.rs:4937-4938` 滤掉；
- (1, 3) 按 `versions_applied_only_by_records` 的四条件进了遍历（环里 (1, 3) 的根槽已被转圈盖掉、实例 1 行 T = 3、3 ≥ F、环里最旧有效根 2 < 3），但 `walk.rs:645-650` 只走记录新根段里的树表与映射根，不走实例表；`walk.rs:641-644` 的注释写明「实例表指针——覆盖写不重写实例表，这一版的实例表就是它施加在其上那一版的，那一版在候选集里时已经走过」「那一版不在候选集里……这两样这里就漏数了：这一格条款没写，没有定案之前照这样走」。这里「施加在其上那一版」正是 (1, 2)，而它不在候选集里。
- (1, 4)（第三截里还在环里）被实例表判抛弃，不走。

第三截（二次崩溃崩在暖机）同一个差：`logs/large-136-diag.log` 第二段 `DIAG_I31 BEGIN allocated0=Some(4521984) walked0=4489216 newest_index=1 effective_floor=3`，`root[9] … txg=4 instance=1 F=0 candidate=false abandoned=true`、`root[16] … txg=2 instance=1 F=0 candidate=false abandoned=false`、`record_applied_version instance=1 txg=3`，没引用的仍是 `slot=50176 span=2 generation=5 released=true` 两盘各一条。

`.claude/kb/invariants.md` 第 131 行 I-3.1 的 checker 读法只说「有效根 = 回退候选集里的根」「由记录施加出来的版本」没有单独一句；它们的实例表单元怎么算，条款里没有（与代码注释「这一格条款没写」一致）。

### 排除掉的解释

| 解释 | 凭哪条观测排除 |
|---|---|
| 实现多记了一个单元（该回收没回收） | 多出来的那一条是真被 (1, 3) 这一版用着的实例表片：(1, 3) 在回退候选集里（txg 3 ≥ F 3，实例 1 行 T = 3），它的实例表按 `walk.rs:642` 就是 (1, 2) 那一片；按 `walk.rs:4658-4660` 引的回收谓词，释放代 5 > max(3, 2) 回收不了。**没读 core 的回收代码本身**，「写者按这个谓词留着它」是从分配记录的读数（释放代 5、已释放、一直留到历史末尾）推的 |
| checker 的 F 生效值取错（系统配置 F 与根上 F 对不上） | 系统配置四个槽 F 都是 3，最新根 F 也是 3（`DIAG_I31 sysconf … rollback_floor: 3`、`root[17] … txg=29 instance=6 F=3`），I-3.1 走的是判定那一路、没有报「不适用」 |
| 装置错：新截的模型问法或 B3b 第二节第 7 条「允许的版本」 | 两条的签名都只有 CheckerViolations I-3.1；诊断用例不经崩溃注入的模型与判定，只调挂载加发布、再跑 `check_pool_image`，得到同样的数（`logs/diag136-mount-a1.log` 原样：`CHECKER_AFTER_MOUNT I-3.1 Violated("盘 0：记账的已分配 Some(4915200)，遍历全部有效根得到 4882432…`） |
| 是 A1 或 B1 带进来的 | 去掉 A1 三道屏障的副本上同一个崩溃状态（那里是第 86 段）挂载发布之后同样的数照样红（`logs/diag136-mount-noa1.log`）；漏数的那段代码与注释在 HEAD（B1 之前）就在 |

### 推翻条件与造它的那一次

- **什么会推翻「漏的是由记录施加出来那一版的实例表」**：让 checker 在走由记录施加出来的一版时，把它施加在其上那一版（环里同实例、txg 更小的最新一条根）的实例表链也走一遍，I-3.1 还红，或者遍历多出来的不正好是 32768 字节（那就是别的单元也漏了，或者多算了）。
- **造的那一次**：副本 `walk.rs` 里加开关 `DIAG_EXPERIMENT_TABLE`（补丁在 `kept/instrumentation-walk.rs.patch`，只做确认，不是修法建议：它在基底根不在环里时仍会漏）。
  - 第二截那份池上：`DIAG_I31 BEGIN allocated0=Some(4915200) walked0=4915200 newest_index=17 effective_floor=3`，没有 `unreferenced_record` 行，I-3.1 不再出现在 `CHECKER_AFTER_MOUNT`（`logs/diag136-mount-exp1-always.log`）：遍历正好多了 32768，与记账逐字节相等。
  - 单种子 …136（`logs/large-136-experiment.log`）：I-3.1 两行都没了，只剩第一截那条 RecordCheck。
  - 快档只开它（`logs/fast-tier-only-DIAG_EXPERIMENT_TABLE.log`）：第 2、3 条两行没了，第 1 条那行与「单元缺席 5 次」还在。
  - 两个开关一起开跑快档（`logs/fast-tier-both-experiments.log` 第 41、98 行原样）：`崩溃状态上 checker 跑了 94 次；记录核对器跑了 94 次：根在而记录一条都不在 0 次、恢复自称新态而单元缺席 0 次`、`test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 5.97s`（报告里 `崩溃状态：以已知红收尾 {}、新发现 0`）。

## 四、第 1 条从实八到今天：二分

候选三样里只有 A1 改了段序列（B1 只动 checker，第 1 条的红在 checker 之前）。做法：副本复制一份，只把 A1 在 `transaction.rs` 加的三道池屏障删掉（快照 :441 回卷写前、:544 抬 F 先写系统配置前、:687 取号写前；补丁 `kept/a1-barriers-removed-transaction.rs.patch`），别的都是今天的代码。

| 跑的是什么 | 今天的代码 | 去掉 A1 三道屏障 |
|---|---|---|
| 快档崩溃状态数 | 94（`logs/fast-tier-copy.log` 第 27 行） | 96（`logs/fast-tier-noa1.log` 第 31 行），与 A1 自己开工时基线日志 `/tmp/claude-1000/impl-rev-a1/baseline-crash_injection.log` 第 145 行的 96 相同 |
| 快档「单元缺席」 | 5 次 | 1 次（第 45 行；与 A1 基线日志第 159 行的 1 次相同） |
| 快档新发现 | …115 RecordCheck、…136 两行 I-3.1 | …129 `冷启动读回` 与 …129 `模型说该成、实现拒了`（after_the_writable_mount_and_one_publish），没有 …115、…136 |
| 种子 …115 单跑（24 步 4 点） | 红，单元缺席 1 次（`logs/large-115-plain.log`） | 绿，`恢复自称新态而单元缺席 0 次`、`test result: ok. 1 passed`（`logs/large-7463871032432355115-noa1.log`） |
| 种子 …115 这段历史的段数 | 77 | 73（写表都是 546 条） |
| 手摆同一个持久集合（#355 扣下、#356 留） | 红（第 56 段） | 红（第 52 段，`logs/diag115-noa1.log`） |

结论：A1 的屏障只拆段、不改写的内容，让 `draw_crash_points`（按种子与段表抽样，`crash_injection.rs:784-786` 调它时传 `history.seed` 与 `&segments`）对种子 …115 抽出的 4 个状态换了一批，抽中了记录核对器这个早就存在的缺口。实八报告里「种子基 + 2 单元缺席 3 → 0」那 3 个状态落到的版本根槽写都在前缀里，实八的豁免罩得住；今天抽中的这个落到由记录重建的版本，豁免罩不住。

**推翻这条二分的现象**：去掉三道屏障之后快档仍报 …115，或者手摆的同一个持久集合在去掉屏障的副本上不红。两样都查过，都没出现（上表）。

## 五、顺带看到的：…129 那一形（C554）在今天的代码上还在

快档里 …129「冷启动读回」今天不见了，是同一个段序列变化让它没被抽中，不是修好了。手摆同一个状态（(2, 4) 的根槽写 #76 扣下）：

- 去掉 A1 屏障的副本（第 17 段，`logs/diag129-noa1.log`）：`RECOVERY outcome="Failed { root: Some((InstanceGeneration(1), CheckpointTxg(3))), failure: MappingStillUnreadable { slot: SlotNumber(50240) } }"`、`RECORD_CHECK RecordCheck { root_without_record: false, claimed_state_missing_unit: true }`
- 今天的代码（第 18 段，`logs/diag129-a1.log`）：`SEGMENT_OF_WRITE #76 = 18 ([76])`，恢复与记录核对器两行与上面逐字相同。

这一条不在派发的三条里，只查到「还在」为止，机理没往下查。

## 六、没做什么

- 没修，没判该怎么改；上面两个开关（`DIAG_EXPERIMENT_FULL_STREAM`、`DIAG_EXPERIMENT_TABLE`）只用来确认定位，不是修法建议——后者在「施加在其上那一版」的根槽也已被转圈盖掉时照样会漏，前者没核它对层 0 以外别的调用方有没有影响。
- 没跑重型测试：没跑层 0、54/55/57/59/87 号、全量 `cargo test`、`gate.sh`；只在副本上跑了崩溃注入那一个测试二进制里点名的快档与大档用例、以及自己加的诊断用例。
- 没在 debug 下复现（只跑了 release）；三条在 release 下与 B3b 报告（debug）逐字相同。
- 第三截（二次崩溃）没单独手摆，靠大档单种子与 `DIAG_I31` 的第二段读数；第三截记录核对器的前缀取法（`crash_injection.rs:1427-1428`、`:1461-1466`）会不会撞上第 1 条同一形，没造状态验。
- 没读 core 的回收代码（`allocator.rs` 等）核「写者按 `walk.rs:4658-4660` 那条谓词留着槽 50176」，那一句是从分配记录读数推的。
- 没二分 B1、A2a、B3a 对第 2、3 条的影响：新两截今天才有；两条的根因代码在 HEAD 就在，去掉 A1 也照红，没有别的候选要排。
- 没查 …129 那一形今天的机理（第五节）。

## 七、草稿目录里删了什么、留了什么

删了（删之前 `du -sh`）：副本 `/tmp/claude-1000/investigate-crash-injection-reds/work` 172M、去掉 A1 屏障的副本 `…/work-noa1` 172M、编译目录 `…/target` 1.1G、`…/target-noa1` 1.1G、崩溃镜像目录 `…/tmp` 71M（大档留下的判红镜像）；另删了从 HEAD 加快照补丁重建出来核行号的 `…/recon`（一份 `crash_injection.rs`）。

留着（都小，都不是仓副本）：
- `report.md`（这份）；`copy-time.txt`、`head-at-copy.txt`、`diffstat-at-copy.txt`、`diff-crates-at-copy.patch`（2.6M）、`status-crates-at-copy.txt`：取副本那一刻的现场。
- `crash.rs.orig`、`walk.rs.orig`：快照里这两份的原件，行号以它们为准。
- `kept/`：诊断用例 `zz_diag_investigation.rs`、四份诊断与实验补丁、追加到 `crash_injection.rs` 末尾的 `pub` 包装 `appended-diag-fn-crash_injection.rs.txt`。要复跑：把补丁打到一份新副本上、用例放进 `crates/singlefs-harness/tests/`，按上文的环境变量跑 `--ignored`。
- `logs/`（2.2M）：上文引的每一份日志。
- `build1.log`：第一次编译的日志。
