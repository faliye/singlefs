# 实审 A3d 实现员报告：A3a 判全之后，两份测试的坏法与期望跟上

写于 2026-09-27（时刻都是 UTC；东京 JST = UTC + 9）。规格 `/tmp/claude-1000/impl-rev-a3d/spec.md`。在副本 `/tmp/claude-1000/impl-rev-a3d/work` 里改、交补丁 `/tmp/claude-1000/impl-rev-a3d/patch/`（`crates.patch`、`mutations-append.tsv`）。快照 04:19 UTC 取自主工作区（四个文件与 `crates/mutations.tsv` 的 sha256 记在 `snapshot-sha256.txt`；交回前核过四个文件此刻与快照逐字节相同）。

## 一、结论

- 规格表三行都做了。两个测试目标在副本里整二进制全绿：`second_transaction_supplement_three_bad_disk_input` 13 过 1 忽略（改前基线 8 过 1 红 1 忽略），`second_transaction_supplement_two_tree_split` 5 过（改前 4 过 1 红）。
- `bad_disk_input` 那四种走乙。记账树根、中央映射树根缩条目宽改成「每条只留 key、条目数不变」，水位那一行改标签时「节点头的最大 key 跟着改」。三份坏镜像都过得了判全那一道，可写挂载仍报原来那几个更深的成员，`every_fixed_panic_site…` 里可写挂载那一列一个字没改。R10 从盘上走不到了，这一格改成在同一条用例里直接调发布路径的释放判定（`instance_table_chain_to_release`），报的成员逐字与原期望相同。另加四条用例，钉这四种原坏法被判全那一道拒成什么成员（两个读者都钉）。
- **偏离 A3a 建议的一处**：A3a 乙写的是「条目宽那两种顺手把节点头的 key 区间改成剩下那一条的 key」。照那样做，记账、映射两棵树只剩 1 条，会先红在判全的条目数判定上（「记账条目数不是 3 + 6 × 盘数」「映射条目数不是 1 + …」，这两道在解条目之前，`recovery.rs` 的 `rebuild_version` 与 `walk_to_file` 都是这个次序），照样走不到 R4 / R2。所以改成保留全部条目、只把每条缩成它的 key：首末 key 不变，区间、条数、key 次序都对得上。
- `tree_split` 那一条走甲。期望改成 `Recovery(InvariantViolated { invariant: "I-1.1", detail: "分隔 key 大于孩子头里的最小 key" })`，盘上逐字节不变那一判原样留着。用例改名为 `a_central_mapping_root_whose_separator_hides_a_key_is_refused_by_the_rebuild_before_the_instance_generation_is_acquired`。
- `fault_injection.rs` 只改了模块头的文档注释：分两条写明 `HistoryDevice`（注入在录制器外）与 55 号 `CountedDevice`（录制器在外）各是什么次序、录制流各记什么。
- 变异：追加 9 行，副本里逐条 `prove-red.sh` 证红，9/9 抓到。第 252、256 行照旧抓到；586 行（checker 走读映射条目宽）也抓到。
- **报一处不是这一件带来的**：第 240 行「分配记录的读者不判条目宽…」（锚点 `allocator.rs` 的 `AllocationRecord::parse`）在副本里**没红**。这一件没碰能走到分配记录叶的那条坏法，理由在第六节。

**什么现象会推翻**：打进主工作区之后，这 9 行里有一行在门禁 59 号没红；或者 `bad_disk_inputs_never_read_back…` 快档撞出 panic、读回了没提交过的内容（坏法改了三种，快档的坏镜像份数没变：「三棵树都在」12 段 203 份、「树表 0 条」8 段 40 份，与基线同）。

## 二、这一轮写过的文件

补丁 `patch/crates.patch` 里的（相对快照；`git -C 主工作区 apply --check` 退 0）：

- `crates/singlefs-harness/src/bad_disk_input.rs`：
  - `EntriesKeptWhenNarrowing`（第 2107 行）：`OnlyTheFirst` 就是原来的缩法，`EveryOneAsItsKey` 是新的「每条只留 key、条数不变」。
  - `NarrowedEntryArea` 与 `narrow_the_entry_width_of_the_node(node, kept)`（第 2142 行），六处共用。extent、inode、分配记录树根、树表单元这四处仍走 `OnlyTheFirst`，行为不变。
  - `LargestKeyOfTheNodeHeader` 与 `relabel_the_inode_watermark_row(image, largest_key)`（第 2496、2506 行）。
  - `INDEX_NODE_SMALLEST_KEY_OFFSET`（第 88 行）；`NewestRootSlot` 多带一个 `checkpoint_txg`（第 1439 行）。
  - 公开的 `DamageRefusedAtTheHeaderJudgement` 与 `damage_image_the_way_the_header_judgement_refuses`（第 1886、1916 行），是三种原坏法，不进 `EVERY_DAMAGE_KIND`。
  - `release_check_of_the_instance_table_placement_released_on_the_second_device_only`（第 1955 行）。
  - 三种 `DamageKind` 的文档与 `name()` 跟着改（名字里多了「每条只留 key」「节点头的最大 key 跟着改」）；新增 core 的几条 `use`。
- `crates/singlefs-harness/src/fault_injection.rs`：只改模块头第 5–7 行的文档注释（改成 8 行两条）。
- `crates/singlefs-harness/tests/second_transaction_supplement_three_bad_disk_input.rs`：
  - `every_fixed_panic_site…`（第 330 行）：表由 13 格变成 12 格，R10 那一格挪到表后，直接调发布路径（第 487 行起）；记账、映射、水位三格的恢复那一列改了。
  - 文档注释改写：原来那段说两个读者走的不是同一条路，A3a 之后不对了。
  - 新 `observation_of`（第 506 行）与四条新用例（第 527、570、613、658 行）；导入加了四个名字。
- `crates/singlefs-harness/tests/second_transaction_supplement_two_tree_split.rs`：那条用例改名（第 390 行）、改期望、改文档；删掉不再用的两个导入 `CodeTwoTreeRefusal`、`PublishError`。

补丁之外：`patch/mutations-append.tsv` 追加 9 行，名字全以「实审 A3d：」起头，第四节逐行列。没有替换行、没有删行，第 252、256、592 行不动。

`git apply --stat patch/crates.patch`（对主工作区，原样）：

```
 crates/singlefs-harness/src/bad_disk_input.rs      |  338 +++++++++++++++++---
 crates/singlefs-harness/src/fault_injection.rs     |   10 -
 ..._transaction_supplement_three_bad_disk_input.rs |  267 ++++++++++++++--
 ...second_transaction_supplement_two_tree_split.rs |   34 +-
 4 files changed, 558 insertions(+), 91 deletions(-)
```

（补丁模式、主工作区没动，所以不附 `git diff --stat -- crates litmus`：那里显示的全是别的会话的改动。）

## 三、各处怎么做的，与恢复那一列为什么变了

两个读者今天调同一组判定，但读的东西不全一样（`recovery.rs` 快照行号）：

- 记账树：两边都先数条目（`accounting_entry_count_is_three_plus_six_per_device`，第 1895 / 3391 行），再逐条解（第 1902 / 3407 行）。所以缩宽不能缩条数。
- 映射树：重建先数条目（第 1916 行）再逐条解（第 1930 行）。冷走读也数（第 3395 行），但只在位置提示读不出时才解映射条目（第 406 行的 `locations_of_key`）。
- 水位：只有重建判「水位那一行在」，冷走读不取水位。

各坏法改后，两个读者实际报的（副本 `work-bad-disk.log` 原样）：

| 坏法 | 改法 | 恢复（冷走读） | 可写挂载 |
|---|---|---|---|
| 记账树根缩条目宽 | 15 条每条只留 22 字节 key，声明长度 330 | `EntryNarrowerThanItsFieldTable { what: "记账条目", entry_bytes: 22, field_table_bytes: 34 }`（原期望 I-1.1） | 同左（原期望，没改） |
| 中央映射树根缩条目宽 | 12 条每条只留 27 字节 key，声明长度 324 | `FileRead（实例 2 第 9 代根，32633 字节）`（原期望 I-1.1） | `EntryNarrowerThanItsFieldTable { what: "映射条目", entry_bytes: 27, field_table_bytes: 55 }`（原期望，没改） |
| 水位那一行改标签 | 第 14 条（末条）标签 12 → 60000，头里最大 key 跟着改 | `FileRead（实例 2 第 9 代根，32633 字节）`（原期望 I-1.1） | `InodeNumberWatermarkRowMissingFromTheAccountingTree`（原期望，没改） |
| R10 两盘账不对称 | 坏法没改；从盘上两个读者都报 E142「分配记录不是每个落点每盘各一条」 | —— | 表后直接调发布路径：`Err(ReleaseTargetAlreadyReleased { unit: InstanceTable, device: DeviceIdentity(1), slot: SlotNumber(50304) })`（与原期望逐字相同） |

R10 那一格搭分配器的方式：拿基线镜像最新那条根里的实例表指针，手搭一个分配器（两条记录与坏法在盘上造的那一对相同：盘 0 还着、代取根的 txg；盘 1 已释放、释放代 `u64::MAX`），调 `transaction::instance_table_chain_to_release`（写行那次发布换下实例表链时调的就是它，`pub`）。`placement_registered_unreleased_on_every_device` 本身是私有的，只能经它进。放在 `every_fixed_panic_site…` 里面，不另开用例：第 256 行点名的就是这条用例，这样第 256 行不用改就还抓得到。

## 四、新测试与证红（`research/scripts/prove-red.sh --copy …/work singlefs-harness <名…>`，日志 `prove-red-logs/`、`prove-red-logs-2/`）

先跑了一份不改的基线副本（`baseline/`，同一快照）：两个目标的整二进制里基线红集就是规格说的那两条（`every_fixed_panic_site…` 红在记账那一格的可写挂载列，`rebuilt_central_mapping_root…` 红在 I-1.1），第二节原样抄。新测试与改过的测试都不在基线红集里（新名字，或者改后不红）。

| 追加的变异（第一段） | 改坏哪一行 → 哪条断言红 | 结局 |
|---|---|---|
| 实审 A3d：坏法改回原样——记账树根缩条目宽只留第一条（…） | `bad_disk_input.rs` 记账那一格 `EveryOneAsItsKey` → `OnlyTheFirst` → `every_fixed_panic_site…` 第 472 行：该让恢复交回 `EntryNarrowerThanItsFieldTable { what: "记账条目"`，实际 `InvariantViolated I-1.1 根 key 区间与条目不符` | 抓到（001） |
| 实审 A3d：坏法改回原样——中央映射树根缩条目宽只留第一条（…） | 映射那一格 `EveryOneAsItsKey` → `OnlyTheFirst` → 同一行：该让恢复交回 `FileRead（实例 2 第 9 代根`，实际 I-1.1 | 抓到（002） |
| 实审 A3d：坏法改回原样——水位那一行改标签时节点头的最大 key 照旧（…） | `FollowsTheRelabelledRow` → `LeftAsItWas` → 同一行：该让恢复交回 `FileRead…`，实际 I-1.1 | 抓到（003） |
| 实审 A3d：普查 R10 直接调发布路径那一格手搭的分配器两盘都还着（…） | `release_check_…` 里盘 1 那条 `is_released: true` → `false` → 第 496 行：实际 `Ok([Placement { slot: SlotNumber(50304), span: 2 }])` | 抓到（004） |
| 实审 A3d：重建读记账树不判节点头（记账根只剩一条：…） | `recovery.rs` `rebuild_version` 记账那一处 `EveryHeaderAgainstItsReference` → `OnlyWhatTheShapeNeeds` → `an_accounting_root_narrowed…` 第 555 行：可写挂载实际 `E142 记账条目数不是 3 + 6 × 盘数` | 抓到（005） |
| 实审 A3d：重建读中央映射树不判节点头（映射根只剩一条：…） | 同上，映射那一处 → `a_central_mapping_root_narrowed…` 第 598 行：实际 `E142 映射条目数不是 1 + …` | 抓到（006） |
| 实审 A3d：重建读记账树不判节点头（水位那一行改了标签、头里最大 key 照旧：…） | 记账那一处同上 → `an_inode_watermark_row…` 第 641 行：实际 `InodeNumberWatermarkRowMissingFromTheAccountingTree` | 抓到（007） |
| 实审 A3d：重建不判分配记录每个落点每盘各一条（两盘的账不对称：可写挂载改红在下一道分配记录代的判定上） | `rebuild_version` 那次 `allocation_records_are_one_placement_per_device_on_every_device(…)?` 改成 `let _ = …;` → `allocation_records_released…` 第 687 行：实际 `E142 分配记录跨度为 0，或分配代 / 释放代晚于根`（释放代 `u64::MAX` 被下一道接走） | 抓到（`prove-red-logs-2/001.log`；改名之前那一遍 008 也抓到） |
| 实审 A3d：重建读中央映射树不判节点头（分隔 key 藏住一把 key：…） | 映射那一处同上 → `tree_split` 的 `a_central_mapping_root_whose_separator_hides_a_key…` 第 478 行：实际 `RowPublishAdmissionRefusedBeforeAcquisition { … PreviousShapeRoutesAKeyAwayFromTheLeafHoldingIt }` | 抓到（009） |

同时红了哪些：每条都只跑点名的那一条用例（`--` 后面的过滤串只命中它），日志里 `test result: FAILED. 0 passed; 1 failed`，没有别的用例同时跑。

已有行复核（不追加，只证）：

| 行 | 结局 |
|---|---|
| 252「普查 R2：核心层映射条目读者的字段表宽度判去掉…」 | 抓到（011）：可写挂载 panic 在 `crates/singlefs-core/src/bytes.rs:72`「range end index 31 out of range for slice of length 27」，第 459 行的断言红 |
| 256「普查 R10：释放前的校验退回只核第一条位置条目那块盘…」 | 抓到（012）：第 496 行，释放判定交回 `Ok([Placement { slot: SlotNumber(50304), span: 2 }])` |
| 586「普查 R2：checker 走读中央映射条目之前不判条目宽…」 | 抓到（013）：池级 checker panic 在 `crates/singlefs-checker/src/walk.rs:862`「range end index 55 out of range for slice of length 27」 |
| 240「分配记录的读者不判条目宽：可写挂载那一侧的落点…」 | **没红**（010），见第六节 |

第 3 步：变异表里点名这两个目标（含 `_layer0`）的 37 行，逐行在副本里核过，锚点都恰好命中一次，点名的用例也都还在。核的办法：在工作副本的快照 `work-crates-snapshot/` 上跑一段内联 python，逐行数锚点命中次数，再从测试文件里抽 `#[test]` 的函数名比对。输出末行原样：`rows 37 bad 0`。其中 9 行是我追加的。主工作区的表 04:19 UTC 之后涨到 1250 行（别的会话追加的），我那 9 个名字都不在里面；5 行 `recovery.rs` 锚点在主工作区的 `recovery.rs` 上各命中 1 次（它与快照逐字节相同）。

## 六、第 240 行没红：不是这一件带来的（照报，没修）

- 第 240 行的锚点是 `crates/singlefs-core/src/allocator.rs:79` `if bytes.len() < usize::try_from(ALLOCATION_RECORD_BYTES).expect("20") {`（`AllocationRecord::parse`）。它只在读分配记录**叶**的时候走到（`allocation_record_tree.rs:726`）。
- `every_fixed_panic_site…` 里只有「分配记录树根缩条目宽」这一种坏法碰条目宽，可它坏的是根。分配记录树按位置寻址，根在第 2 层（日志 010 原样：`树种类 3 的根（盘 [0, 1] 槽 50345，层级 2）条目宽 96 → 10`），两个读者都先红在内部条目那一道（`allocation_record_tree.rs:754`，`EntryNarrowerThanItsFieldTable { what: "分配记录树内部条目" }`），走不到叶。
- 这一格的期望在 HEAD 上就已经是「分配记录树内部条目」（`git show HEAD:…/second_transaction_supplement_three_bad_disk_input.rs` 第 368–370 行）。所以第 240 行在 HEAD 上就抓不到，与 A3a、与这一件都无关。这一件没改那条坏法（仍是 `OnlyTheFirst`），也没改那一格的期望。
- 要它再红，得有一条坏法缩某片分配记录**叶**的条目宽（叶之上的内部条目照旧），或者把第 240 行改指别的用例。这超出了这一件的清单，交主 agent 定。

## 五、第 4 步那几样的末尾原样输出（都在副本 `work/` 上跑，线程上限 4、内存上限 8G）

`cargo test --offline -p singlefs-harness --test second_transaction_supplement_two_tree_split`（`work-tree-split.log`）：
```
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.91s
```
`cargo test --offline -p singlefs-harness --test second_transaction_supplement_three_bad_disk_input`（`work-bad-disk.log`）：
```
增补 3 的验收「三个读者都不许 panic」在这一档（写死的种子基、12 段 × 16 步）上成立：KNOWN_PANIC_SITES 空着（0 条），三个读者 panic 0 次。
test result: ok. 13 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 499.55s
```
改前基线（`baseline/` 副本，同两条命令）：
```
  baseline-tree-split.log:test result: FAILED. 4 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.77s
  baseline-bad-disk.log:test result: FAILED. 8 passed; 1 failed; 1 ignored; 0 measured; 0 filtered out; finished in 1215.16s
```
`cargo fmt --all -- --check`：退 1，唯一的 Diff 在别人的文件（`crates/singlefs-harness/src/bin/first_transaction_on_device.rs:3059`，与快照逐字节相同、基线副本上单跑 rustfmt 同样退 1）；我动的四个文件 `rustfmt --edition 2021 --check` 退 0。`fmt.log` 首行原样：
```
Diff in /tmp/claude-1000/impl-rev-a3d/work/crates/singlefs-harness/src/bin/first_transaction_on_device.rs:3059:
```
`cargo clippy --offline --all-targets --all-features -- -D warnings` 加 `CODE_DISCIPLINE_LINTS` 七条：退 0，末行原样：
```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 9.05s
```
`cargo build --offline --all-targets`：退 0，末行原样（副本里 E161 编得过，没删它）：
```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 33.29s
```

登记给我的门禁阶段（`stage-owners.tsv` 里 implementation-writer 那几道：33、53、74、92、94、93、89）。补丁模式下我的改动只在副本里，所以都在 `work/` 副本里跑（cwd = 副本，`bash .claude/gate.d/<阶段>`），33 号另在主工作区现状上跑了一次：

| 阶段 | 退出码 | 末行 / 判定（原样摘自 `gate-<阶段>.log`） |
|---|---|---|
| 33（副本） | 1 | `crates/mutations.tsv:659 E158 root_choice_repair session s10：…原文在 crates/singlefs-harness/src/bin/e158_root_choice_repair.rs 里命中 2 次`。这是快照里 E158 那一行，不是我的；我那 9 行不在红行里 |
| 33（主工作区现状，14:1x 打上 E158 补丁之后） | 1 | 只剩 `research/mutations/e163_gpu_multicard_crc32c.tsv 不存在（被测的是 research/e7-index-bench/src/bin/e163_gpu_multicard_crc32c.rs）`，crates 表那一段没有红行。E163 不是我的 |
| 53 | 0 | `✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：…）` |
| 92 | 77（本次未跑） | `! /tmp/claude-1000/impl-rev-a3d/work 不是 git 仓，本阶段跳过`，按没判写 |
| 94 | 0 | `✓ checker 与实现只共享常量模块 singlefs-format（…checker 闭包 1 个、实现闭包 2 个，内部依赖图 4 个 crate）…` |
| 93 | 0 | `✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致（…扫了 57 个 .rs，认出 4 处 feature bit 常量…）` |
| 89 | 77（本次未跑） | `⊘ 本次未跑：收口表第 27 行那几笔的前置一个都没进来，今天无对象可判（5 条逐字探针、覆盖 4 笔…）`，按没判写 |
| 74 | 1 | `test result: FAILED. 21 passed; 3 failed; 2 ignored`：`second_transaction_supplement_three_random_history` 那三条，判法见下 |

`apply-writer-patch.py …/patch --dry-run`（对主工作区现状）原样：`✓ 核过了（--dry-run，没改）：补丁 有，变异表合并之后 1264 行`，退 0。`git -C 主工作区 apply --check patch/crates.patch` 退 0（14:1x 主工作区又打上 E158 补丁之后重做过）。

74 号红的判法：同一个二进制在改前的 `baseline/` 副本（同一快照、不带这一件的改动）复跑，命令 `cargo test --offline --release -p singlefs-harness --test second_transaction_supplement_three_random_history`（经 `run-with-memory-cap.sh 8G` 与 `capped.sh 4`，日志 `baseline-random-history.log`，05:29 UTC 跑完），红的是同样那三条，末尾原样：

```
failures:
    crash_recovery_abandoning_the_newest_root_then_raising_the_floor_into_its_txg_ends_in_the_known_red_form_of_closeout_row_43
    random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation
    rolling_back_to_the_root_abandoned_by_the_crash_recovery_step_is_refused_on_the_abandoned_timeline

test result: FAILED. 21 passed; 3 failed; 2 ignored; 0 measured; 0 filtered out; finished in 158.42s
```

`work/` 副本上 74 号那一次（`gate-74-model-differential.sh.log`）红的也是这三个名字，`21 passed; 3 failed; 2 ignored`。签名是 `ModelDisagreement { aspect: "模型说该成、实现拒了" }`：模型说 `MountWritable` 该成，实现报 `NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion)`。这是 C554 乙之后本来就有的基线红，模型跟上乙那一件正在修，不是这一件带来的。照派发提示的要求没修。

## 五之二、接手之后对主工作区现状重核（2026-09-27 约 05:3x UTC，JST 14:3x）

前一个实现员在 05:29 UTC 撞了会话限额，我从这里接手。已经做完、证过红的都没重做，只核了下面几样：

- 主工作区的 `crates/mutations.tsv` 现在 1255 行（打上 E158 bin 补丁之后）。`git -C /home/fy5090/code/singlefs apply --check /tmp/claude-1000/impl-rev-a3d/patch/crates.patch` 退 0。`python3 research/scripts/apply-writer-patch.py /tmp/claude-1000/impl-rev-a3d/patch --dry-run` 末行原样 `✓ 核过了（--dry-run，没改）：补丁 有，变异表合并之后 1264 行`，退 0（1255 + 9）。
- 主工作区里这四个文件的 sha256 与 `snapshot-sha256.txt` 逐个相同，所以副本上的测试与证红结果对今天的主工作区照样成立。
- 追加的 9 行照 33 号的解法（只把 `\n` 还原成换行）数锚点。4 行 `bad_disk_input.rs` 在打过补丁的 `work/` 副本上各命中 1 次；5 行 `recovery.rs` 在主工作区现状上各命中 1 次（主工作区的 `recovery.rs` 与副本逐字节相同，`cmp` 退 0）。9 个名字都不在主工作区的表里。
- 第 240、252、256、586、592 行的名字与目标仍在原来的行号上，没有挪。

## 七、停下交主 agent 的设计问题

1. **第 240 行抓不到**（第六节）。这一行在 HEAD 上就抓不到，和这一件无关。要让它再抓到，有两条路：加一条坏法，缩某片分配记录叶的条目宽；或者把这一行改指一条能走到 `AllocationRecord::parse` 的用例。两条都在这一件的清单之外，我没做。
2. **偏离 A3a 乙原话的一处**（第一节）。A3a 的原话是「只剩那一条」；我改成了「条数不变、每条只留 key」，因为照原话做走不到 R4 / R2，会先红在条目数判定上。守卫要罩的对象（R4 / R2 那两道字段表宽度判定）没变，第 252 行照样抓得到。要是主 agent 要的是「条数也对不上」这一类坏法，那得另开一格。
3. 规格之外，我没新定别的设计。

## 八、没做什么

- 没走三方对抗，也没提交。层 0 全量、QEMU、herd7、crates 变异表整表（59 号）都归 `crash-verifier`。追加的 9 行只在副本里用 `prove-red.sh` 逐条证过，整表复跑留给提交时的 59 号。
- 名字带 layer0 的测试二进制没跑，标了 ignore 的崩溃注入快档（`bad_disk_input` 目标里那 1 条忽略）也没跑。第 3 步核锚点时点名 `_layer0` 目标的那几行只核了锚点和用例名，没跑。
- 登记给我的阶段里有两道本次没判：92 号在副本上退 77（副本不是 git 仓）；89 号退 77（今天没有要判的对象）。这两道按没判算。33 号在主工作区现状上红在 E163 的变异表缺了，那不是我的。
- `cargo fmt --all -- --check` 退 1，唯一的 Diff 在别人的 `first_transaction_on_device.rs`，我没修。
- 74 号那三条基线红没修（见上）。
- 补丁没打进主工作区，由主 agent 用 `apply-writer-patch.py` 来打。

## 九、草稿目录里的副本

交回之前删掉了 `baseline/`（16G）、`work/`（17G，带各自的 target）和 `work-crates-snapshot/`（8.8M）。补丁、日志、进度、规格都留在 `/tmp/claude-1000/impl-rev-a3d/`。
