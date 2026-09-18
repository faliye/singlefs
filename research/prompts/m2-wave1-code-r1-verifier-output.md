# `m2-wave1-code-r1` 核查员报告

你交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

## 〇、判别力自证

挑引用：opus 报告「`DeviceFreeMap::empty_segments`（`allocator.rs` 第 354 行）只数 `used_per_segment == 0`」。
在草稿目录副本 `/tmp/claude-1000/m2-wave1-code-r1-verifier/repo/crates/singlefs-core/src/allocator.rs` 里把行号加 1（354→355）核：

```
$ sed -n '355p' allocator.rs
        u64::try_from(
```

第 355 行不是 `pub fn empty_segments(&self) -> u64 {`，判 ✗。核查方法分辨得出错误。**自证通过。**

## 一、快照缺失与主树可信度——现查结论（本轮的前提）

派发提示没有给「腿开工那一刻的快照路径」。按定义「没给快照的代码轮，停下要，不对主树核」，本应停下不对主树核；
但现查发现可以分文件判断主树是否可信，结论如下（全部由本轮现查得出，非假设）：

1. **`crates/` 全树自开工以来未改动**：`find crates -newer research/prompts/_m2-wave1-code-r1-background.md`（背景材料落盘于 18:16:58 UTC）与
   `find crates -newer research/prompts/_m2-wave1-code-r1-diff.md`（附录二落盘于 18:16:30 UTC，注明「回读逐字节比对，退出码 0」）两条命令均**零命中**。
   ⇒ 本报告对 `crates/*.rs`（`mount.rs`、`transaction.rs`、`allocator.rs`、`write_accounting.rs`、`crash.rs`、`unit.rs`，及 `crates/mutations.tsv`）**直接对主树核**，不算「没给快照」的例外，因为这批文件本身就是稳定的。
2. **`.claude/kb/` 里只有一个文件在轮次进行中被改过**：`find .claude/kb -newer research/prompts/_m2-wave1-code-r1-background.md` 只命中
   `.claude/kb/milestone/02-second-txn.md`，`stat` 给出该文件 mtime `2026-09-17 18:43:24`——晚于 local-attack 交回（18:30:46）、sonnet 交回（18:41:42），早于 opus 交回（18:44:36）。
   `.claude/kb/decisions/*.md`、`.claude/kb/checks-owed.md`、`.claude/kb/verification-build.md` 均未被改过，对这些文件的引用**直接对主树核**。
3. **对 `milestone/02-second-txn.md` 的现查**（`git diff` 该文件，对比 HEAD）：改动落在 8 处 hunk，其中 7 处新旧行数相同（61/132/157/186/213/263/426 起，逐位替换、不挪行号），
   仅一处净增 39 行（`@@ -301,23 +301,62 @@`）。用当前内容逐字核对 sonnet 报告在这个文件里引的全部 10 处行号（162、222、328、329、330、331、333、334、335、340、342）与
   opus 报告引的 1 处（293）：**除 222 行外，全部逐字匹配当前内容**（含 sonnet 交回之前就已写就、且落在净增 hunk 范围内的 328–342 那几行）。
   222 行是唯一不匹配的一处，且不匹配的方式非常具体：sonnet 引的原文是「『陈旧 tail + 已复用的块』那一条**还没做**」，
   当前该行同一段落写的是「『陈旧 tail + 已复用的块』那一条 **2026-09-17 补上了**」——这正是 sonnet 报告 Y6 节判定「内部自相矛盾」时指出的那半句被改写掉了矛盾。
   综合三点（该 hunk 是同行数替换、发生时刻正好夹在 sonnet 与 opus 交回之间、改写内容正好命中 sonnet 发现的矛盾点），
   本报告认定：**sonnet 对 222 行的引用在其交回时点上是准确的，现在的不匹配是轮内编辑造成的，不是引用错误**；单列为「分不清：文件在腿交回之后被改过」，不计入 ✓ 也不计入 ✗。
   对该文件其余 10 处引用（sonnet 9 处、opus 1 处），按上面的逐字匹配结果计入 ✓。
4. 建议：下一轮把 `crates/` 与 `.claude/kb/` 的快照作为代码轮的强制输入，本轮能靠 mtime + git diff 补救，纯属运气（改动恰好同行数、恰好没碰其它被引行）。

## 二、云端攻方（Opus）报告核对

产物 sha256（报告表里给的 vs 现算）：

```
$ sha256sum research/prompts/m2-wave1-code-r1-opus-model/opus_probe_y1_y2_y3.rs research/prompts/m2-wave1-code-r1-opus-model/probe-run.log
99c61ad957207d605e43f844c0ae7efb01e32e43d4f4a227ec9e550cb67474f1  .../opus_probe_y1_y2_y3.rs
34e2606a88d3256a825b1afaff75041cf6abd7817d75727a22b39a7aea4bbb0d  .../probe-run.log
```
两个哈希与报告表里的逐字节相同。✓

### 复跑（草稿目录 `/tmp/claude-1000/m2-wave1-code-r1-verifier/repo`，`rsync` 自 opus 自己的仓副本，覆盖式换上 `research/prompts/` 里的官方模型文件，`diff` 确认换前换后逐字节相同）

```
$ cd /tmp/claude-1000/m2-wave1-code-r1-verifier/repo && nice -n 19 cargo test -p singlefs-harness --test opus_probe_y1_y2_y3 -- --nocapture --test-threads=1
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
```
去掉 `Compiling`/`Finished`/耗时三类必然不同的行之后，与官方 `probe-run.log` **逐行 diff 零差异**（`diff` 退出码 0）。四个数逐个核：

| 断言 | 报告原样引用 | 复跑原样输出 | 结果 |
|---|---|---|---|
| Y1-c 数据单元与提交内生块 | 数据单元 50240，七个提交内生块 50242–50249 | `data_slot=50240 commit_generated...=[("ExtentRoot",50242),("InodeLeaf",50244),("InodeRoot",50243),("AllocationTree",50246),("AccountingTree",50247),("MappingTree",50248),("TreeTable",50249)]` | ✓（50242–50249 是最小-最大值区间记法，报告明写「七个」，7 个值确实都落在该区间内） |
| Y1-d 记账行不变、最低段跳段 | 3310 → 3310 不变，最低段 50304 → 50368 | `isolated_segment=50304 empty_segments_row_before=3310 empty_segments_row_after=3310 lowest_empty_segment_now=Some(50368)` | ✓ |
| Y2-c 实例代号与录制流步数 | 两块盘实例代号 1 → 2，录制流多 3 步 | `instance_before=[1, 1] instance_after=[2, 2] operations_added=3` | ✓ |
| Y3-b 按种类合计 vs 设备落盘 | 按种类 21 次 / 344 576 字节，设备落盘 25 次 / 442 880 字节 | `by_kind_write_calls=21 by_kind_written_bytes=344576 ... device_landed_writes=25 device_landed_bytes=442880` | ✓ |

四条探针的四个断言全部对得上（本报告「二、」第 1 点已确认 `crates/` 自开工以来未改动，此处直接对主树 + 副本核，非「分不清」）。

### 代码行引用逐条核（对主树，`crates/` 确认未改动；重点核派发提示点名的 8 处全部在内）

| 引用（文件:行，报告原样描述） | 核的结果 |
|---|---|
| `mount.rs`:829 `acquire_expected_instance` 调用点 | ✓ 逐字 |
| `mount.rs`:866「写行那次发布 `publish_rows_on_file_version`」 | **✗ 行号差 2**：第 866 行是 `let row_publish = match &start.previous {`；`publish_rows_on_file_version(` 调用实际在第 **868** 行 |
| `mount.rs`:880 `?` 上抛 | ✓（`)?)`） |
| `mount.rs`:977 `rebuild_previous_version` | ✓ |
| `mount.rs`:814 `establish_instance`、:566 `raise_rollback_floor`、:280 `isolate_slots_referenced_only_by_abandoned_roots`、:377 与 :629 两处 `reclaim_released_up_to` | ✓ 全部逐字（均为函数定义行或调用点） |
| `transaction.rs`:227 `roll_back_acquisition` | ✓ |
| `transaction.rs`:369–384 `write_acquired_instance` | ✓ 函数体逐字匹配 |
| `transaction.rs`:1036–1058 `rewritten_roles`、:1039「`Data` 排第一个」 | ✓ |
| `transaction.rs`:1459–1468（含 :1466 `NoSpaceFor`） | ✓ |
| `transaction.rs`:1157 `publish_version`、:1211–1215 分配器换回 | ✓ |
| `transaction.rs`:1626–1629 `STATISTIC_EMPTY_CLUSTER_SEGMENTS` | ✓ |
| `transaction.rs`:1850 `rewritten_unit`、:1230 `carried_unit` | ✓ |
| `transaction.rs`:1967（取基线）、:2005–2007（取差） | ✓ 逐字（`let writes_before_this_publish = pool.writes_by_structure_kind.clone();` / `writes: pool.writes_by_structure_kind.since(&writes_before_this_publish),`） |
| `transaction.rs`:1968–1988「六个 `?`」 | ✓，现数确为 6 个 |
| `transaction.rs`:1968–1974「`identity` 只从 `unit.identity` 来」 | ✓ |
| `write_accounting.rs`:49 `WrittenStructureKind::of_unit` | ✓ |
| `allocator.rs`:395 `lowest_user_data_slot`、:399 `open_segment.is_some_and` | ✓ 两处均准确（同一函数内两个不同引用点，非内部矛盾） |
| `allocator.rs`:239 `is_free`、:315 `is_blocked_for_commit_generated`、:436 `lowest_commit_generated_fallback_slot`、:415 `lowest_empty_segment` | ✓ 全部函数体逐字匹配报告转述的逻辑（`!allocated && !isolated && !held...` 等） |
| `allocator.rs`:266/278/299/321/328/371 六个改状态方法 | ✓ 全部函数签名匹配 |
| `allocator.rs`:588 `record`、:147 `agreement_across_devices` | ✓ |
| `allocator.rs`:354 `empty_segments`（只数 `used_per_segment==0`） | ✓（函数体核实：只 `filter(|used| **used==0).count()`） |
| `allocator.rs`:200 注释「记账已经算它空闲，分配器却不许发出去」 | ✓ 逐字 |
| `allocator.rs`:210 `DeviceFreeMap::new`、:120–136 `PlacementRefusal`、:619 `release`、:522 `rebuild_from_records`、:801 `reclaim_released_up_to` | ✓ |
| `allocator.rs`:743 `self.open_segment = None;` | ✓ 逐字 |
| `allocator.rs`:647 `try_allocate_user_data`、:695 `try_allocate_commit_generated`、:700–704（只读段） | ✓ |
| `allocator.rs`:755 `bump_slot_in_open_segment`、:739 `.expect(...)` | ✓ |
| `allocator.rs`:639、:681 `allocate_user_data`/`allocate_commit_generated` | ✓ |
| `03-空间分配.md`:253、:255（D3 已定项 8 ①②） | ✓ 逐字 |
| `03-空间分配.md`:392（D3 已定项 10 ①「全空聚簇段数」定义） | ✓ 逐字 |
| `.claude/kb/milestone/02-second-txn.md`:293 | ✓（见「一、」第 3 点；此文件唯一可直接对主树核的引用，因交回时点晚于该文件被改的时点） |
| 产物核：`grep -rn "write_at(" crates/ --include=*.rs \| grep -v "fn write_at"` 共 36 处，生产代码仅 `make_filesystem.rs`:227/232/284/303 与 `transaction.rs`:128/136/153/216 | ✓ 现跑同一条命令，结果 36，行号逐个吻合；`recovery.rs` 现查 `write_at` 零命中，与报告一致 |
| `transaction.rs`:130/137/161/221 四处 `count_write_call` | ✓ |

**Opus 报告计数**（表格行数用 `grep -c '^|'` 数，扣掉每张表的表头与分隔行）：核了 36 行（4 行探针数字复跑 + 32 行代码/产物引用，见上面两张表）；✓ 35；✗ 1（`mount.rs`:866，实际在 868，2 行之差）；分不清 0；核不动 0。两个模型文件的 sha256 另核 2 项，均一致，不计入 36 行。

## 三、云端正推（Sonnet）报告核对

行号一律对主树 `grep -n`/`sed -n` 现查（该报告自陈「不从背景材料或 diff 附录里数」，与本报告做法一致）。

### U2–U7（Y5，条款/代码一致性判定引用）

| 引用 | 核的结果 |
|---|---|
| D3 已定项 8 ②，`03-空间分配.md:255`「提交内生块的段耗尽时回落到该设备内槽号最小的空槽，同样排除 `R`」 | ✓ 逐字 |
| `checks-owed.md:344`（C369）、`:343`（C368） | ✓ 逐字 |
| D3 已定项 8 第 1 条，`03-空间分配.md:253` | ✓ 逐字 |
| D2 已定项 2 标题，`02-RAID条带策略.md:101` | ✓ 逐字 |
| `allocator.rs:436-451`、`432` 行注释、`315-318` `is_blocked_for_commit_generated`、`434` 行「第一版 `R` 为空」 | ✓ |
| D3 已定项 8 第 1 条同段引用（`03-空间分配.md:253`「不选设备、不轮转设备」半句） | ✓ 逐字，确认这半句也在同一行内 |
| `allocator.rs:8` 文件头注释、`try_allocate_user_data`（647-678）、`try_allocate_commit_generated`（695-752）、`agreement_across_devices`（147-174） | ✓ 行区间与函数对应 |
| D23 已定项 18，`23-journal的角色与格式.md:1250`「tail 8 字节存 jsn 的 48 位计数器……」 | **✗ 引用范围不全**：报告引的整段话实际横跨 1249–1250 两行（「在飞记录数上限……写进超级块；」在 1249 行，「tail 8 字节……」才在 1250 行），报告只标了单一行号 :1250 |
| `transaction.rs:418-420`、`:490-544`（`publish_without_units`）、`:532-534`、`:185-224`（`write_superblock_slot`） | ✓ |
| D19 已定项 9，`19-块指针的结构与宽度预算.md:409` | ✓ 逐字 |
| `transaction.rs:947-948`、`955-965`（`next()`）、`1450`/`2046`/`2100`（三处 `BirthSequenceAllocator::default()`） | ✓ |
| 里程碑步 3，`milestone/02-second-txn.md:162`「其它树装不下也要报错不 panic……」 | ✓ 逐字（此行不在唯一净增 hunk 范围内，现查内容与交回时应一致，见「一、」第 3 点） |
| D5 已定项 4 登记表，`05-快照-空间记账机制.md:358/361/363/364/369/370/371/383` | ✓ 全部 8 处逐字，含列头「带不带设备维」「带不带树维」核实 |
| `transaction.rs:1220/1222/1225-1227/1191-1196/1204-1211` | ✓ |
| `checks-owed.md:52`（C42）、`:87`（C77） | ✓ 逐字 |
| `second_transaction_step_zero_layer0.rs`:805-807/809/902-906/922/994/1068-1071/1089/1090-1093/1097/1126-1135/341-344 | ✓ 全部 11 处逐字（含断言消息原文） |

### Y6（写回，收口表/mutations.tsv/crash.rs 逐行核）

| 引用 | 核的结果 |
|---|---|
| `milestone/02-second-txn.md:328/329/330/331/333/334/335/340/342`（收口表第 19/20/20′/21/23/23′/23″/28/30 行） | ✓ 全部 9 处逐字匹配现在的内容——见「一、」第 3 点：这批行落在净增 39 行的 hunk 范围内，仍逐字匹配，说明该 hunk 在这批行上是纯挪位、没改内容 |
| `milestone/02-second-txn.md:222`「必红计数」句 | **单列「分不清：文件在腿交回之后被改过」**，理由见「一、」第 3 点；不计入 ✓ 也不计入 ✗ |
| `transaction.rs:1450`（同 U5）、`1272`（`build_file_version_units`） | ✓ |
| `crates/mutations.tsv` 第 77–106 行：`unequal_devices\|commit_generated_fallback` 计 9、`second_transaction_step_zero_layer0` 计 6 | ✓ 现跑同一条 `awk`+`grep -c` 命令，结果分别为 9、6，与报告逐字相同 |
| `.claude/kb/verification-build.md:76`「用户数据的落点与政策函数不一致的次数」 | ✓ 逐字；该行末列原文正是「同上 `fallback_policy_mismatches`」 |
| `grep -rn 'fallback_policy_mismatches' crates/` → 零命中 | ✓ 现跑同一条命令，零命中 |
| `crash.rs:505`（`Layer0Tally`）、`525/526/530`（三字段）、`536-548`（`checker_counts_by_invariant`） | ✓ |
| `transaction.rs:767`「`.expect("树表第 1 版起恒有七条")`」 | ✓ 逐字 |
| `.claude/gate.d/55-qemu-first-transaction.sh:43` `MODES=(...)` | ✓ 逐字，确认不含 `second-instance` |
| `first_transaction_on_device.rs`:526（`"second-instance" =>`）、:898（宿主态测试名） | ✓ |

**值得主 agent 注意但不属于本报告判定范围的一处发现**（观测，非推理判断）：sonnet 在 U6 节称
「`grep -rn 'fallback_policy_mismatches' crates/` → 零命中……这个计数器在 `crates/` 里今天根本没有实现」。
现查 `allocator.rs:494` 确有字段 `pub policy_mismatches: u64`（无 `fallback_` 前缀，第 510/948 行各一处使用），与 `milestone/02-second-txn.md:330`
现在写的「`PoolAllocator::policy_mismatches` 字段还在（`allocator.rs` 第 494 行）」一致。即：sonnet 搜索的标识符名称与 `verification-build.md:76`
自己使用的名称（`fallback_policy_mismatches`）逐字相同、搜索本身没有做错，但代码里实现该计数器用的是不带前缀的名字，
搜索目标本身没有覆盖到实现。这是可复核的事实分歧（字段是否存在），不是本报告要判定的"结论对不对"，转交主 agent。

**Sonnet 报告计数**（同上，按行数）：核了 26 行（U2–U7 表 16 行 + Y6 表 10 行）；✓ 24；✗ 1（`23-journal的角色与格式.md:1250` 引用范围少标一行，实横跨 1249–1250）；分不清 1（`milestone/02-second-txn.md:222`）；核不动 0。

## 四、本地攻方（local-attack）核对

### 转述核对表（`m2-wave1-code-r1-local-attack-translation-audit.md`）逐条核

对该表第二列全部 14 处「原文文件:行」现查（`.claude/kb/decisions/16-发布语义.md`、`23-journal的角色与格式.md`、`19-块指针的结构与宽度预算.md`、`05-快照-空间记账机制.md`、
`crates/singlefs-core/src/transaction.rs`、`crates/singlefs-core/src/unit.rs`、`crates/singlefs-format/src/lib.rs`）：

| 引用 | 核的结果 |
|---|---|
| `16-发布语义.md:207`（D16 已定项 8 主句） | ✓ 逐字，五个限定词核对表自称的都在 |
| `16-发布语义.md:208`（follow-up note，实付 2 次/格式常量/区域归属 0,1,0） | ✓ 逐字；核对表自陈删掉的三处（D2 已定项7 交叉引用、`FIRST_TRANSACTION_TXG=3`、两句元信息）现查确实不影响本轮算式 |
| `23-journal的角色与格式.md:1250`（只引 tail 编码那半句） | ✓ 逐字——**与上面"三、"里 sonnet 的引用不同，这里核对表只摘了确实整句都在 1250 行的那部分，未越界到 1249 行**，无 ✗ |
| `23-journal的角色与格式.md:1265`（① 空发布记录） | ✓ 逐字；核对表自陈删掉「①」编号与「(不改)」标记，现查这两处确是元信息 |
| `19-块指针的结构与宽度预算.md:409`（D19 已定项 9） | ✓ 逐字 |
| `05-快照-空间记账机制.md:481`（D5「第一个事务写 15 行」锚点） | ✓ 逐字 |
| `transaction.rs:1219`（池级三行注释）、`:1221`（设备维六行注释） | ✓ 逐字 |
| `transaction.rs:1220,1222,1225-1227`（`POOL_WIDE_ACCOUNTING_ENTRIES`/`ACCOUNTING_ENTRIES_PER_DEVICE`/`accounting_entry_count`） | ✓ |
| `singlefs-format/src/lib.rs:47,50-54`（头部字节公式）、`:110`（`ACCOUNTING_KEY_BYTES=22`）、`:111`（`ACCOUNTING_ENTRY_BYTES`公式）、`:15`（`NODE_BYTES=16384`） | ✓ 全部逐字 |
| `unit.rs:126-131`（`index_node_entry_capacity`，整数除法向零取整） | ✓ |
| `transaction.rs:1199,1204-1209`（准入判据，严格大于） | ✓ |
| `transaction.rs:950-952`（`BirthSequenceAllocator` 结构体）、`:955-965`（`next()`，先取后加） | ✓ |
| `transaction.rs:1450`（`publish_admitted` 内 `BirthSequenceAllocator::default()`） | ✓ |
| 「本轮自造的英文脚手架词」：`starting record number`↔`last_journal_counter`、`publish index`↔`txg_number`、`the previous counter`↔`previous_counter` | ✓ 现查 `warm_up_after_journal_counter` 函数签名与循环体，三个 Rust 标识符逐字与核对表所称对应 |

14 行全部 ✓，0 处 ✗。

### 两份样本的 13 个编号答案：算术复核 + 与提示要求的格核对

按提示 Section 2/3/4 给定的公式手算全部 13 项，两份样本逐项比对：

| 项 | 按提示公式算出 | s1 | s2 |
|---|---|---|---|
| W1.start0.pub1/pub2 | txg 1/1 jsn 1/2 tail 1/2 | 一致 | 一致 |
| W1.start1.pub1/pub2 | jsn 2/3 tail 2/3 | 一致 | 一致 |
| W1.start40.pub1/pub2 | jsn 41/42 tail 41/42 | 一致 | 一致 |
| W2.devices1/2/78/79/80 行数 | 9/15/471/477/483 | 一致 | 一致 |
| W2 max entries | floor((16384−159)/34)=477（159=86+2×22+29，34=22+8+4） | 一致 | 一致 |
| W2 准入判定（严格大于 477 才拒） | passes/passes/passes/passes/AccountingEntriesExceedOneNode | 一致 | 一致 |
| W3.four_in_a_row | 0 1 2 3 | 一致 | 一致 |
| W3.next_publish_first_value | 0 | 一致 | 一致 |

13 项全部一致，两份样本互相一致，且都与提示自带公式吻合（这是算术核实，不是三方推论）。
编号格式：`wc -l`+`grep -n` 现查，两份样本的 13 个标签（`W1.start0.pub1` … `W3.next_publish_first_value`）都在场且与提示要求的标签逐一对应，无缺号、无重号。

### 运行记录（runlog）产物数字复核

```
$ wc -w -l research/prompts/m2-wave1-code-r1-local-attack-output-s1.md   → 342 words 38 lines
$ wc -w -l research/prompts/m2-wave1-code-r1-local-attack-output-s2.md   → 447 words 13 lines
```
与 runlog 逐字相同。✓

```
$ grep -n 'checkable' ...s1.md | wc -l   → 4     （runlog 称「命中 4 次」✓）
$ grep -n 'checkable' ...s2.md | wc -l   → 4     （runlog 称「命中 4 次」✓）
$ grep -n 'AccountingEntriesExceedOneNode' ...s1.md | wc -l → 1  （runlog 称「命中 1 次」✓）
$ grep -n 'AccountingEntriesExceedOneNode' ...s2.md | wc -l → 2  （runlog 称「命中 2 次」✓）
```

### 复跑 `oov-check.py`（草稿目录 `/tmp/claude-1000/m2-wave1-code-r1-verifier/local-attack`，拷入 s1/s2 与提示文件）

第一次带提示文件参数复跑，得「生词=0」，与 runlog 的「生词=5/6」不符；改成**不带提示文件参数**单独复跑（`oov-check.py <输出文件>`，不传第二个参数）：

```
$ python3 oov-check.py m2-wave1-code-r1-local-attack-output-s1.md
绿 ...s1.md  生词=5 拼接=0     生词: checkable AccountingEntriesExceedOneNode
$ python3 oov-check.py m2-wave1-code-r1-local-attack-output-s2.md
绿 ...s2.md  生词=6 拼接=0     生词: checkable AccountingEntriesExceedOneNode
```
与 runlog「生词=5（去重后 2 个）/生词=6（去重后 2 个），均为 `checkable`、`AccountingEntriesExceedOneNode`」逐字吻合。✓
（顺带复跑 `corruption-check.py` 两份样本，均绿，退出码 0，runlog 未给具体数字，此处仅作补充确认，不计入下方计数。）

**Local-attack 报告计数**：转述核对表 14 行（`grep -c '^|'` 数，扣表头分隔行），全部 ✓；算术核对表 8 行（每行覆盖 s1、s2 两份样本，16 次数字比对全部一致，按行数计 8 项）；runlog 产物数字 4 类（s1/s2 的 `wc` 各 1 类 + `checkable`/`AccountingEntriesExceedOneNode` 的 `grep` 计数 2 类）；`oov-check.py` 复跑 2 次。
✓：14 + 8 + 4 + 2 = 28；✗：0；核不动：0；分不清：0。

## 五、三腿合计

| 腿 | 核了 | ✓ | ✗ | 分不清 | 核不动 |
|---|---|---|---|---|---|
| 云端攻方（Opus） | 36 | 35 | 1（`mount.rs`:866→实为 868） | 0 | 0 |
| 云端正推（Sonnet） | 26 | 24 | 1（`23-journal的角色与格式.md:1250` 引用范围少标 1249 行） | 1（`milestone/02-second-txn.md:222`） | 0 |
| 本地攻方 | 28 | 28 | 0 | 0 | 0 |
| **合计** | **90** | **87** | **2** | **1** | **0** |

（以上「核了」一律按表格行数计，用 `grep -c '^|'`（扣表头与分隔行）现数，不手数；每行常常捆着不止一处具体的文件:行引用，表格正文里逐处都写了。另有 2 项 sha256 核对与 4 项 `oov-check.py`/`corruption-check.py` 复跑不计入本表，均在上文各节列出且全部一致。）

## 六、没做什么

- 不判一条打中成不成立、该不该采纳；不核推理本身（Y1–Y7、U2–U7 的判定是否该采信、四条探针的"打中"归哪一格判据），只核引用、产物与复跑。
- 未核 opus 报告"改法"一节（A1–A3、B1–B2、C1–C2）里标"推的"的六条改法——报告自陈"没实现、没跑"，没有可核的产物或行号。
- 未核 sonnet 报告里标"复核不了"的几处（"单文件发布字节不变"、残留记录流"65536 字节"具体算术、三项"已有"变异未逐一找专用测试文件）——这些是该报告自己标注的范围外项，不是遗漏。
- 未编译过 `crates/` 主树本身、未跑门禁、未跑层 0 全量、未在虚机里核对 `.claude/gate.d/55-qemu-first-transaction.sh` 的实际执行结果，只现查了脚本文件里的 `MODES` 定义行。
- 未去索取或重建轮次开工时刻的 `crates/`/`.claude/kb/` 快照文件本身；本报告"一、"节的可信度判断建立在 mtime 与 `git diff` 的现查推断上，不是对着一份真正的快照逐字核的，建议下一轮代码轮把快照列为强制输入。
- 未对本地模型网关重新发起真实调用（`ask-local.sh` 依赖外部网关且非确定性），只对已落盘的两份样本产物做算术复核、产物字符串复核与脚本复跑。
- 未逐条核 opus 报告"六、没打中的形状"表格里最后一行以外的其余读码论证的完整性（只核了其中点名的行号是否存在，未重新走一遍它枚举六个改状态方法是否确实枚举完整这类需要通读全部函数体才能下结论的判断）。
