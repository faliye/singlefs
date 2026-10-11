# m3-formal-code-b34-r1 正推腿报告（Sonnet，2026-10-06）

立场：正推。对象是第 34 批的改动全文 `research/prompts/_m3-formal-code-b34-r1-diff.md`。冻结副本 `/tmp/claude-1000/m3-formal-code-b34-r1/tree/` 只读；
我编、跑、打临时补丁的是自己草稿目录里的副本 `/tmp/claude-1000/m3-formal-code-b34-r1-sonnet/work/`，每个补丁跑完都还原并核 sha256。
代码引文的行号是冻结副本里的行号；kb 引文的行号是 kb 文件自己的行号（现查）。模型与脚本在 `research/prompts/m3-formal-code-b34-r1-sonnet-model/`，复跑入口 `tools/rerun.sh`。

## 0 现场

- 冻结副本核验：副本 sha256 清单逐行对（命令与输出在第 11 节「收尾核验」）。每份副本用自己的编译目录（`CARGO_TARGET_DIR=<草稿>/target`）。
- 编译证据（第一趟 `build1.log`，原样）：`Compiling singlefs-checker`、`Compiling singlefs-core`、`Compiling singlefs-harness` 三行都在；变异 34 行每一趟都重编（`mut-results3.jsonl` 里 `compiled_pkg` 34 / 34 为真）。
- 起跑前看过 `ps`：只有别的会话的 `cargo test`（变异跑），没有 `qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`；每条命令 `nice -n 19`，经 `run-with-memory-cap.sh 8G` 与 `capped.sh 10`；等锁时长没量。
- 没跑：checker 档的一切（GPU 对拍、GPU 变异行、层 0、`crash_enumeration_*`、`mutations.tsv` 整表）；凡涉及它们的结论一律写「推的」。
- 我跑的都是 harness 档：`checker_known_bad_images` 121 条、两份改过的测试、新用例、mount_transparency 48 条、`--lib` 的 B4、抬 F 慢档那一条（`--include-ignored --exact`），
  以及十二个别的二进制作合法历史样本（原样 test result 行在 `results/tests-summary.txt`）。`row_publish_record_without_its_root` 在第一趟被我的 240 秒上限截断，第二趟单独跑过 `ok`。

## 1 速览（每条在后面各节有依据与推翻条件）

| 格 | 一句话 |
|---|---|
| P1 | 一致（CPU 一侧实测；GPU 一侧读码，推的）。新根段取自 `0..86` 与 `274..360`，取错字段的四种变异都被新用例抓；合法历史上我这趟 0 次多报，但 applied-only 那一类的合法样本很少 |
| P2 | 主走读、I-5.4、I-5.5、删除态、墓碑谱系、I-9.28 六处各自的「自证过的根」不是同一个集合（实测）；core 一侧新根全零时拒挂载且不退回上一条根，与 checker 同方向但不是同一个结局；没有一处在合法历史上误红 |
| P3 | core 根槽读者四个入口都经 `RootRecord::parse_slot`，没有留在只判 extent 偏移的；GPU 恢复内核的选根还只判 extent 偏移（已知，进第 35 批）；两份测试改得对，翻出生 txg 判的还是原来那一格 |
| P4 | Q1–Q4 各给候选、翻面观测、我在副本上量到的代价；四问都不挡这一批合入（理由逐问写）；Q1 有一个没被任何测试钉住的分支 |
| P5 | 36 行变异：整行唯一、锚点各命中一次、点名的测试都跑得到、34 行 harness 档全红、基线全绿；其中 2 行（34-4 重取的两行）红在 `allocation_roots[0]` 越界 panic 而不是断言上；B4 取样点 100 只罩住含 99 的窗口；#9 钉值在 34-1 合入后必然不对（旁证：逐状态 I-5.4 的判定变了） |

## 2 P1　RV

### P1-a　两类版本在合法历史上会不会多报

判定：**一致，我这一趟没打中**（一次抽样，不是证明；层 0 没跑）。

正推：
1. 条款要的是「按版本判」：`.claude/kb/invariants.md:198`：「记录与最新根同实例、txg 比最新根大 1、带提交标记，根环里没有它的根」。写侧保证每一版自己的分配记录树互不相交（同一行：「新记录罩住的槽上别的已回收记录要随这次分配一起删掉」），
   被多收的两类版本是同一次发布写出的树，与落了根槽的版本是同一种东西。
2. 多收的两类版本的单元本来就被走读读过：`crates/singlefs-checker/src/walk.rs:5559`：「tree_table_pointers.extend(tree_table_pointer_of_the_version_the_next_mount_applies_first);」（I-3.10 已读下一次挂载先施加的那一版），
   applied-only 那一类走主走读与 I-5.5。读不出时 I-5.4 报部分没判、不报红，复用过的单元位置条目校验和对不上，读不出。所以 RV 没有引入新类的单元。
3. 实测（harness 档，只读冻结副本的拷贝）：
   - 合法历史前缀探针：新池新建文件之后连发三次覆盖写，`mkfs` 之后每个写操作前缀一个状态。第一趟只数 I-5.4（`probe_prefix_states_of_legal_publishes_…`）：
     ```
     PROBE-PREFIX states=106 states_with_any_violation=0 i54_tally={"Holds": 106} first_op_count=82 total_ops=187
     ```
   - 同一探针加 `instrument` 补丁（把 RV 的版本数打到 stderr），这 106 个状态里 RV 真走到了下一次挂载先施加的那一版：
     ```
     12 个状态 next_mount_first=true（4 + 4 + 4），applied_only_by_records=0 在全部 106 个状态里
     ```
     所以这 106 个状态**不覆盖** applied-only 那一类；那一类的合法样本只有新用例里挂载之后的干净镜像（`two_allocation_records_covering_one_slot_in_a_version_applied_only_by_its_journal_record_…` 的前半，断言 0 违例且 I-5.4 `Holds`）与旧用例。
   - 别的 harness 二进制（回退、抬 F 复用、多实例覆盖写、根环转一圈、故障注入快档、随机历史等十二个）都过，原样行在 `results/tests-summary.txt`；其中多数在 `check_pool_image` 上断言合法镜像不红（`grep -lE check_pool_image` 在 harness 的 tests 与 src 下命中 65 个文件，我没逐个核断言）。抬 F 与回退的慢档我只跑了抬 F 那一条（`--include-ignored --exact`，`ok`）。
4. 没覆盖：层 0 全量与崩溃放量（checker 档）。「推的」：RV 在层 0 的合法状态上不红，理由是上面第 1、2 点；一次都没验。

推翻它的现象：层 0 或 harness 里任一合法状态上 I-5.4 红，或在合法状态上报部分没判。

### P1-b　两类版本的新根段取自哪里，取错一版会不会让判法与实现一起错

判定：**一致**。
- 取法：`crates/singlefs-checker/src/walk.rs:7583`：「const NEW_ROOT_SEGMENT_TREE_TABLE_POINTER: std::ops::Range<usize> = 0..86;」，
  `crates/singlefs-checker/src/walk.rs:7588`：「const NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER: std::ops::Range<usize> = 274..360;」。
  条款侧：`.claude/kb/decisions/23-journal的角色与格式.md:154` 写「新根段 99」，同文件 452 行写「一次发布切成几条记录时，每条记录都带同一份新根段」。
  树表指针在段内 0、分配记录树根在 274（根记录里 36 与 342 之间隔 306，段里没有 32 字节自证校验和：306 − 32 = 274），GPU 一侧 `crates/singlefs-checker-tier/src/crash_judge_common.wgsl:95`：「const JOURNAL_NEW_ROOT_SEGMENT_OFFSET: u32 = 99u;」、
  `crates/singlefs-checker-tier/src/crash_judge_common.wgsl:100`：「const JOURNAL_NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER: u32 = 274u;」，三处一致。
- 两条独立的路：core 的施加法 `crates/singlefs-core/src/recovery.rs:3666`：「tree_table: record.new_tree_table,」用的是 `journal.rs` 解出来的字段，不与 checker 共用偏移表；checker 另有 I-8.10 逐字节比落了根槽的版本的新根段与根记录
  （`crates/singlefs-checker/src/walk.rs:7601`：「const NEW_ROOT_SEGMENT_FIELDS_COMPARED_WITH_THE_ROOT」），所以新根段取错字段这件事在已落根的版本上有第三条路在看。
- 我造了四个「取错一版里的字段」的变异（自拟，不在表里，`tools/custom_mut.py`、`data/custom1.json`），施加在 I-5.4 调用点上、跑整份 `checker_known_bad_images`（121 条）：

  | 变异 | 红的条数 | 新用例是否在其中 |
  |---|---|---|
  | M1 applied-only 版的「分配记录树根」取成映射根指针（86..172） | 6 | 是（applied-only 的三条） |
  | M2 下一次挂载先施加的那一版同样取错 | 9 | 是（`…next_mount_applies_first…`） |
  | M3 applied-only 版的树表指针取成实例表指针（188..274） | 3 | 是（三条） |
  | M4 下一次挂载先施加的那一版的树表指针取成实例表指针 | 4 | 是 |

  都不是编译失败、都还原并核了 sha256（`results/custom1.out`、`results/custom1b.out`）。
- 「取成别的记录」：一版由 (实例代号, txg) 认，一版切成几条记录时每条带同一份新根段（上面 452 行），`.find` 取先读到的那一条，同一版内取哪条内容相同。我没造「取别的记录」的变异（等价，推的）。
  「同一记录的旧段」：记录里只有一份新根段，没有旧段这个来源；落了根槽的版本由 I-8.10 比。

推翻它的现象：两条记录的新根段对不上而 I-8.10 与 I-5.4 同时没报；或上面任一变异在新用例下存活。

### P1-c　GPU 扫描内核与 CPU 是不是同一组版本、同一判法

判定：**一致（读码，推的）**，GPU 没跑。
- 版本：候选根、`R_DATA` 里由记录施加出来的版本、下一次挂载先施加的那一版，各调一次 `judge_allocation_records_disjoint_of_a_version`（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:900`：「fn judge_allocation_records_disjoint_of_a_version(base: u32, tree_table_offset: u32, held_offset: u32) {」），
  次序与 CPU 的 `.chain` 相同。`R_DATA` 的四个收录条件在 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3765`：「if (root_slot_is_readable || lt64(r.txg, effective_floor) || !lt64(oldest_valid_txg, r.txg)) {」，
  与 CPU 的 `crates/singlefs-checker/src/walk.rs:7740`：「let at_or_above_the_floor = record.checkpoint_txg >= rollback_floor;」、7741 行「let its_units_are_not_reclaimable_yet = oldest_valid_root_txg < record.checkpoint_txg;」同判；同键取先到的一条。
- 下一次挂载先施加的那一版：GPU 把 I-3.10 里内联的那段抽成 `record_index_of_the_version_the_next_mount_applies_first`（同文件 1071 行），逻辑与改前逐句相同；I-3.10 与 I-5.4 共用它。
- 判法：违例位同判；部分没判 GPU 不置位（`.claude/kb/invariants.md:198` 写明「只有违例位、部分没判不置位」）。树表指针全零时 GPU 的 I-5.4 `return`，CPU 记部分没判：违例位上一致。
- 变异证红：GPU 只有下一次挂载先施加的那一版有一行（`crates/mutations.tsv` 第 2855 行），applied-only 那一次调用没有对拍流也没有变异行，这是 `.claude/kb/checks-owed.md:584` 的 C735 已记的欠。我没跑。

推翻它的现象：对拍里任一状态 CPU 与 GPU 的 I-5.4 违例位不同。

### P1-d　条款与实现的差（规则没说）

`.claude/kb/invariants.md:198` 写 applied-only 那类是「已被一次恢复施加过、实例表里有行」，实现还多三个条件：不低于回退下界（`walk.rs:7740`）、它换下的单元还没到回收的时候（`walk.rs:7741`）、反向链对得上能进重放前缀。
低于下界或单元已可回收的由记录施加的版本 I-5.4 不判，条款没写这三条。不挡合入，归写回批补一句。推翻它：条款别处已写这三条（我没在 I-5.4 那一行里找到）。

## 3 P2　全零树表指针（用户定甲）

用户原话：`records/2026-10-01-里程碑三形式化接入.md:1177`：「甲 checker 判它损坏」。判决 b33 第五节第 1 条的实现范围是「池级 checker 对自证过的根，树表指针全零不再当没有树表，照 core 跟过去判损坏；删除态扫描等同一读法的几处一并改」。

### P2-a　有没有一处在合法历史上误红

判定：**没打中**（读码加实测，一次抽样）。
- 合法写者写不出全零树表指针：在 core 里把整条全零的 `empty_root()` 赋给树表指针的只有测试模块里的四处，其余都是真树表指针（命令与输出）：
  ```
  $ grep -n 'tree_table: NodePointer::empty_root()\|new_tree_table: NodePointer::empty_root()' crates/singlefs-core/src/*.rs
  journal.rs:404   (journal.rs:388 起是 #[cfg(test)] mod tests)
  root_record.rs:201   (root_record.rs:191 起是 #[cfg(test)] mod tests)
  mount.rs:7067、mount.rs:7143   (mount.rs:7053、7123 各是 #[cfg(test)])
  ```
  mkfs 写真树表：`crates/singlefs-core/src/make_filesystem.rs:584`：「tree_table: NodePointer {」。
- 34-1 实现员在全部合法历史上探过：`records/2026-10-01-里程碑三形式化接入.md:1198` 的交回说「探针两趟（改前非慢档、改后 release 带慢档）0 次」（他们的观测，我没复跑）。
- 我这趟：`checker_known_bad_images` 121 条与十二个别的二进制全过；干净镜像探针 `PROBE clean checker_violated=[]`、`checker_partially_judged=[]`（`results/probes.out`）。
推翻它：任一合法状态上出现全零树表指针，或合法镜像上 I-5.4 / I-5.5 / I-9.28 / 删除态 / 墓碑谱系因它报部分没判。

### P2-b　有没有一处把本该判的东西整版放过

判定：**有三处放过，都不是这一批引入的，但都落在 Q1 的缺口里**。实测都在 `results/probes.out`。
1. 由记录施加出来的那一版（applied-only）树表指针全零：主走读照旧当没有树表（`crates/singlefs-checker/src/walk.rs:1446`：「AllZeroTreeTablePointerReading::NoTreeTable,」、
   1472 行「AllZeroTreeTablePointerReading::NoTreeTable => {}」），I-5.4 / I-5.5 / 删除态 / 墓碑却把它记成部分没判（下表）。实测把 (1, 5) 那条记录的新根段树表指针 86 字节写零、重封：
   ```
   PROBE q1b.zeroed(applied-only version (1,5)) violated=["I-3.1", "I-3.5"]
   PROBE q1b.zeroed(applied-only version (1,5)) partially_judged=["I-5.4", "I-5.5"]   （I-9.5 / I-9.18 / I-9.24 报「射程里判不了」）
   ```
   红的是 I-3.1 与 I-3.5（账对不上），不是「这一版损坏」（I-2.1 / I-4.8 / I-7.4 不红）。
2. 下一次挂载先施加的那一版全零：它根本不进主走读（只有 I-3.10 与新加的 I-5.4 读它）。实测同样把新根段树表指针写零：
   ```
   PROBE q1.zeroed_new_root_segment_tree_table_pointer(next-mount version) violated=[]
   PROBE q1.zeroed_new_root_segment_tree_table_pointer(next-mount version) partially_judged=["I-5.4"]
   PROBE q1 mount_writable=Err Recovery(UnitUnreadable { slot: SlotNumber(0) })
   ```
   checker 一条不红，core 可写挂载拒成「盘 0 槽 0 读不出」。
3. 不是候选的自证过的根（低于回退下界、被实例表判抛弃）：主走读不走它，全零时只有 I-9.28 报部分没判。实测把最新根的 F 改成 3 让 txg 1、2 低于下界，再把 txg 2 的树表指针写零：
   violated 与只改 F 时逐项相同（`["I-3.1", "I-7.9", "I-7.12", "I-8.10"]`，都是我改 F 带来的），I-9.28 报部分没判并点名「盘 0 上的根（实例 1、txg 2）：树表指针全零」。
还留着静默 skip 的几处（行号清点，没逐处造盘面）：`walk.rs:4195`（`references_of_root`，记 `is_complete = false`）、5561（I-3.10 读集）、5673、6227、6375、6448、9168（I-9.6 墓碑那一半，全零时 `Ok(None)`）。
它们或只读不判、或只处理最新根（最新根全零时主走读已红），我没造出一处让它们放过一个本该红的合法性之外的状态。
推翻它：造出一张镜像，其中 applied-only 版全零而 I-3.1 / I-3.5 也不红。

### P2-c　「自证过的根」在每一处是不是同一个集合

判定：**不是**（读码，三处加实测）。

| 处 | 版本集合 | 树表指针全零时 | 位置（冻结副本） |
|---|---|---|---|
| 主走读 | 候选根 = 最新根 ∪（没被抛弃且不低于 F 的根） | 记走读失败，红 I-2.1 / I-4.8 / I-7.4，最新根再红 I-7.2 | `crates/singlefs-checker/src/walk.rs:1412`、`crates/singlefs-checker/src/walk.rs:10630`：「(!abandoned && !below_floor);」 |
| 主走读 | 由记录施加出来的版本 | 静默（NoTreeTable） | `walk.rs:1446` |
| I-5.4 | 候选根 + applied-only + 下一次挂载先施加的那一版 | 部分没判 | `walk.rs:5319` |
| I-5.5 | 候选根 + applied-only | 部分没判 | `walk.rs:4672` |
| 删除态、墓碑谱系 | 候选根 + applied-only | 报「射程里判不了」 | `walk.rs:8727`、`walk.rs:9611` |
| I-9.28 | 根环里全部读得出且自证过的根（不按候选滤） | 部分没判 | `crates/singlefs-checker/src/walk.rs:5107`：「TreeTableReading::PointerAllZero => {」 |

条款对不上实现：`.claude/kb/invariants.md:315` 写「只判根环里自证过的根」并写明主走读对由记录施加出来的版本「不记走读失败」；`.claude/kb/invariants.md:292`（I-9.5）写「自证过的根树表指针全零照 I-9.28」。
这两处都只说「根」，实现里 I-5.4 / I-5.5 / 删除态 / 墓碑谱系把由记录施加出来的版本也算进了「部分没判」，下一次挂载先施加的那一版只进了 I-5.4。这一格条款没说，交 Q1。
推翻它：再造一张镜像，在上面某一处对某一类版本的处置与表里写的不同。

### P2-d　改法与 core 读路径是不是同一个结局

判定：**同方向，不是同一个结局**（实测，`probe_newest_root_…`、`probe_older_root_…`）。
```
newest(txg3) checker_violated=["I-2.1", "I-4.8", "I-7.2", "I-7.4", "I-8.10"]    （I-8.10 是我只改根槽没改记录带来的，不属这一格）
newest(txg3) recover.outcome=Failed { root: Some((InstanceGeneration(1), CheckpointTxg(3))), failure: UnitUnreadable { slot: SlotNumber(0) } }
newest(txg3) mount_writable=Err "Recovery(UnitUnreadable { slot: SlotNumber(0) })"
older(txg2)  checker_violated=["I-2.1", "I-4.8", "I-7.4", "I-8.10"]    mount_writable=Ok chosen_txg=CheckpointTxg(3)
```
- 同方向：最新根全零，core 拒、checker 红；旧候选根全零，core 挂载成功、checker 红，与 `.claude/kb/invariants.md:133` 对根记录四条指针那一支写的「checker 红这一条而 core 挂载成功，两边各判各的、不矛盾」同形。
- 不同：core 对 extent 偏移 / MAC / 算法类型等判损坏时是「这条根不可择、退回上一条根」，挂载成功；对全零树表指针 core 择中这条根、读树表时失败，**不退回**上一条完好的根（txg 2 的根与记录能把状态施加回同一版）。
  用户定甲没动 core（判决 b33 第五节第 1 条那一行的乙，core 把全零当自证不过，没选），这一格记下来。
- I-2.5 照旧报成立（`PROBE … I-2.5=Holds`）：`crates/singlefs-checker/src/image.rs:1466`：「/// I-2.5：位置条目按设备身份严格升序；整条全零的指针豁免。」，这是 Q2。
推翻它：core 对全零树表指针的根在任一入口退回上一条根。

## 4 P3　34-2 根记录四条指针

### P3-a　新判法在 core 每个读根槽的入口是不是都走到

判定：**一致**；checker 档装置里有一处（GPU 恢复内核选根）还只判 extent 偏移，已知、进第 35 批。
- 四条指针都换了：`crates/singlefs-core/src/root_record.rs:150`、158、163、168 各一处「NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(」，入口是 `crates/singlefs-core/src/root_record.rs:128`：「pub fn parse_slot(bytes: &[u8], expected_filesystem_identifier: &[u8; 16]) -> Option<Self> {」。
- 只判 extent 偏移那一支删净：`grep -rn 'read_judging_the_extent_offset_from' crates` 命中 0 行（我在冻结副本上跑的）；core 里非测试路径的 `Pointer::read_from` 一处不剩：
  ```
  $ grep -rn 'Pointer::read_from' crates/singlefs-core/src/
  singlefs-core/src/pointer.rs:490:            NodePointer::read_from(&mut ByteReader::at(&bytes, 0)),     （pointer.rs 自己的单测）
  ```
- 读根槽字节的入口全经 `parse_slot`：`crates/singlefs-core/src/recovery.rs:1257`：「match RootRecord::parse_slot(&bytes, filesystem_identifier) {」（`read_root_ring_slot`，只读恢复、可写挂载读根环、回退判候选、抬 F 的上限与预演读根环都走它：
  调用点 `recovery.rs` 1197 / 1277 / 1406 / 1482、`mount.rs` 2396 / 4252）；`crates/singlefs-core/src/mount.rs:1764`：「match RootRecord::parse_slot(&bytes, filesystem_identifier) {」；
  `crates/singlefs-core/src/instance_table_sweep.rs:484`：「root_read = RootRecord::parse_slot(&bytes, filesystem_identifier);」；
  `crates/singlefs-core/src/recovery.rs:3265`：「(RootRecord::parse_slot(&bytes, &root.filesystem_identifier).as_ref() == Some(root))」。没有第二条读根字节的路。
- checker 档装置：`crates/singlefs-checker-tier/src/bin/` 下 e156 / e158 / e165 / e170 调的是 core 的 `parse_slot`，自动带上新判法；GPU 恢复内核选根不带：
  `crates/singlefs-checker-tier/src/crash_judge_common.wgsl:928`：「fn root_record_pointers_carry_no_extent_offset(r: RootFields) -> bool {」，
  `crates/singlefs-checker-tier/src/crash_judge_common.wgsl:957`：「if (!candidate.valid || !root_record_pointers_carry_no_extent_offset(candidate)) {」。
  后果（推的，GPU 没跑）：对 core 现在拒、GPU 仍择中的根，GPU 恢复与 core 恢复会选出不同的根；层 0 的流里没有这种根。
- 实测（harness 档）：新用例 4 条两种挂载都退到暖机最后那条根；`pointer_extent_offset_…_both_mounts` 2 条、`…_every_tree_reader` 两份各 7 与 8 条全过（`results/tests-summary.txt`）。
推翻它：在 core 里找到一条不经 `parse_slot` 读根槽字节的路径；或 harness 档任一入口对四类字段之一的非第一版取值照择。

### P3-b　改了的两份测试改得对不对

判定：**对**，判别力有一处移了位。
1. `mount_transparency_judged_field_by_field_after_a_writable_mount.rs`：新辅助 `flip_one_byte_of_a_root_record_range`（`crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs:1773`），
   只对整段 86 字节全零的指针段改写出生 txg 那一字节（偏移 42：`BIRTH_TXG_IN_THE_POINTER_HEAD`，同文件 1768 行「const BIRTH_TXG_IN_THE_POINTER_HEAD: usize = 16 + 12 + 1 + 1 + 2 + 2 + 8;」，
   与 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:22` 的头部字段表「头部 50：MAC 16、nonce 12、算法类型 1、压缩算法码 1、压后长度 2、extent 偏移 2、出生树 8、出生 txg 8」逐项相加一致）。
   原来写段首（MAC 第一字节，同文件 1762 行「None => bytes[range.start] = 1,」），根记录读者现在判它损坏、重封后自证不过、到不了判官。非全零的指针段仍走 `flip_one_byte_of`（第一个非 0 字节改 0，头部前 34 字节恒 0，碰不到）。
   判的还是原来那一格：这两行判的是「这一段是不是全零」（中央映射树根空性、「R1 带树 ⇒ R2 全零」），翻哪个字节都会让它变非全零。我跑了 48 条（`ok`），并施加了 `mutations.tsv` 第 2860、2861 行两条新变异（把这两格判官改成恒成立），
   两条都红在点名的用例上，断言消息是「「中央映射树根指针」翻了一字节：判它的 MappingRootEmptiness 要红：[]」与「「分配记录树根指针」翻了一字节：判它的 AllocationRecordTreeRoot 要红：[]」。
   我的推断（没有现成变异覆盖）：这个翻法碰不到「出生 txg 那一段」本身的判官行；这一格今天没有这样的行。
2. `pointer_extent_offset_that_is_not_zero_is_refused_by_both_mounts.rs`：被删的 `read_judging_the_extent_offset_from` 换成 `read_with_head_fields_judgement_from(…, judged)`。两个读法最后都进 `PointerHeadAsRead::judged`（`crates/singlefs-core/src/pointer.rs:209`：「fn judged(self) -> Result<PointerHead, PointerHeadFieldOutsideTheFirstVersion> {」），
   所以这条单测现在比的是同一实现的两个入口，判别力比改前小；根记录读者那一路的判别力移到了同文件的 `every_root_carrying_a_non_zero_instance_table_extent_offset_is_refused_by_recovery_and_the_writable_mount` 与新文件的四条。
   变异 `mutations.tsv` 第 1787 行（根记录实例表指针不判 extent 偏移）施加后红在「只读恢复：每条根的实例表指针 extent 偏移都非 …」那条断言上（`results/mut-results3.jsonl` 里 k=19）。
3. 新文件四条用例各改一条指针一类字段（树表：算法类型；实例表：压缩算法码；映射根：压后长度；分配记录树根：MAC），四条变异（表里第 2856–2859 行）各红在点名的用例上。没有 nonce 一格：nonce 在 `…_every_tree_reader` 里由 inode 内部条目那条钉住，四条指针共用 `judged`，不另造。
推翻它：把 `judged` 里任一段的判摘掉而 harness 档全绿（我没做这种变异）。

## 5 P4　条款没写的四问

下面每一问给候选、翻面观测、我在副本上量到的代价和「挡不挡这一批合入」，不替用户定。补丁都在 `tools/patch_walk.py`，跑完还原并核了 sha256（`results/patches.out` 里每个补丁末行 `restore ok`）。

### Q1　由记录施加出来的那一版里树表指针全零，要不要也判损坏

今天（候选 A，照旧）：主走读对这类版本静默，I-5.4 / I-5.5 / 删除态 / 墓碑谱系却记部分没判，下一次挂载先施加的那一版只进 I-5.4（见 P2-b、P2-c）。
- 候选 B：applied-only 那一类也照 core 跟过去：`walk.rs:1446` 的 `NoTreeTable` 改成 `FollowedLikeTheRootRecordReaderOfTheCore`，一行。
- 候选 C：B 加上给下一次挂载先施加的那一版补一遍主走读（今天它不进主走读，任何内容都不走）：要新增一种 `VersionBeingWalked`、I-7.4 / I-4.8 每版一格、外观规则，GPU 同改；没量。
- 候选 D：反方向，I-5.4 / I-5.5 / 删除态 / 墓碑谱系对记录版也改回「没有树表」、不报部分没判，与主走读和用户定的「自证过的根」范围对齐。
- 翻面观测：applied-only 版全零的镜像上 checker 红的是「损坏」那几条（I-2.1 / I-4.8 / I-7.4）还是账对不上那两条（I-3.1 / I-3.5）；next-mount 版全零的镜像上 checker 红不红。
- 量到：
  | 形 | 今天 A | 候选 B（补丁 q1a） |
  |---|---|---|
  | applied-only 版全零 | violated=`["I-3.1", "I-3.5"]`，partial=`["I-5.4", "I-5.5"]` | violated=`["I-2.1", "I-3.1", "I-4.8", "I-7.4"]` |
  | next-mount 版全零 | violated=`[]`，partial=`["I-5.4"]`；core 可写挂载拒 | 不变（它不进主走读） |

  候选 B 下 `checker_known_bad_images` 121 条全过，探针二进制（含它自带的同一份 121 条加 4 条探针，共 125）也全过；next-mount 那一格「不变」是读码结论（补丁只动 applied-only 那一处调用），我没在 q1a 下重跑那一形：**没有任何测试钉住 `NoTreeTable` 这一支**（`mutations.tsv` 里也没有对应行），所以 A 与 B 之间换哪个都不会有现成用例变红，也就是说今天选 A 没有证据在看着。
- 挡不挡：**不挡**。合法历史上没有这样的版本（P2-a）；applied-only 形今天已经红（I-3.1 / I-3.5），只是红的名字不是「损坏」；next-mount 形 checker 绿而 core 拒是该类版本不进主走读的老现状，不是 34 批引入。
- 推翻「不挡」：发现合法写者能写出全零树表指针的版本。

### Q2　全零树表指针要不要同时红 I-2.5

今天（A）：I-2.5 对整条全零的指针整条豁免（`crates/singlefs-checker/src/image.rs:1466`），实测 `I-2.5=Holds`。条款：`.claude/kb/invariants.md:134` 把今天写出全零指针的处所列成四处（树表里 livelist、稀疏旁表、deadlist 三条条目，加 mkfs 与暖机根记录里的中央映射树根指针），根记录里的树表指针不在其中。
- 候选 B：跟全零树表指针时另调 `judge_location_entries_order`（不带豁免），改 3 行（补丁 q2）。
- 翻面观测：全零树表指针的镜像上 I-2.5 红不红；合法镜像上是否多出 I-2.5 红。
- 量到：补丁 q2 下 `checker_known_bad_images` 119 过 2 败，败的是两条全零用例的 `violated` 列表多了 `"I-2.5"`（`left: ["I-2.1", "I-2.5", "I-4.8", "I-7.2", "I-7.4"]` 对 `right: ["I-2.1", "I-4.8", "I-7.2", "I-7.4"]`，另一条同形），
  要改的是这两条用例的期望列表，合法镜像上不会多判（只在全零指针上多调一次）。
- 挡不挡：**不挡**。推翻：补丁 q2 下任一合法镜像出现 I-2.5 红。

### Q3　GPU 判器要不要对全零树表指针同改

今天（A）：CPU 红 GPU 不红，`.claude/kb/invariants.md:315` 已写；GPU 的 `w_read_index_node` 见全零指针直接返回（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1481`：「if (p.all_zero) {」）。
- 候选 B：在 `w_walk_tree_table_and_mapping`（同文件 3096 行起）里，候选根（`w_context == 0`）全零时先 `w_read_referenced_unit(p, NODE_BYTES)` 判 I-2.1、再 `w_failures += 1u`；加一条造全零树表指针根的录制流、一行变异。
- 翻面观测：对拍里那条流 CPU 红 GPU 不红则对拍红（今天没有这样的流，所以对拍不红）。
- 代价：读码估约 6 行 WGSL、一条对拍用例、一行变异；GPU 对拍在 checker 档、要 NVIDIA 独显，我没量，「推的」。另一缺口：GPU 恢复内核选根只判 extent 偏移（P3-a），core 现在拒的根比它多一圈，同样要对拍才看得见。
- 挡不挡：**不挡**（层 0 的流里没有全零树表指针的根，P2-a）。推翻：层 0 某条流里出现这样的根。

### Q4　I-5.5「认出了那一版但没有数据单元」要不要另写理由

今天（A）：落缺省理由（`crates/singlefs-checker/src/image.rs:358`：「.unwrap_or("这个镜像上没有它判的对象"),」），`.claude/kb/invariants.md:199` 已写明「认出来而没有数据单元那一形仍落缺省理由」。
- 候选 B：`walk.rs:4815` 那个 `if` 加 `else`，再写一句理由（补丁 q4，3 行）。
- 翻面观测：要把「没有对象」与「一次都没判」在这一形上也分开时才需要；今天这一形与「I-5.5 的判法根本没跑」报同一句话。
- 量到：补丁 q4 下 `checker_known_bad_images` 121、`pool_checker_judges_tree_table_head_…` 9、`reuse_after_raising_the_floor` 3（慢档那一条另在 base1 里过）、`writable_mount_of_a_formatted_pool` 15 全过，没有测试依赖这一形的缺省理由。
  这一形在合法历史里恐怕不出现：探针里零长度首文件仍有一个数据单元（`PROBE-I55 zero-length-first-file I-5.5=Holds`），要「认出来而没数据单元」得是第一次建树那一版的 extent 树根全零，合法写者不写（推的，没造出这样的镜像）。
- 挡不挡：**不挡**。推翻：造出合法历史上认出来而没有数据单元的一版。

## 6 P5　I-5.5 专门理由、B4、#9、变异行

### P5-a　I-5.5 专门理由与它的看门用例

判定：**分得出「没有对象」与「一次都没判」；分不出「认出来而没有数据单元」与「一次都没判」**（读码加实测）。
- 实现：认出一版就计数，`crates/singlefs-checker/src/walk.rs:4728`：「versions_built_from_the_mkfs_watermark += 1;」；一版都没认出才写专门理由，`crates/singlefs-checker/src/walk.rs:4815`：「judgements.not_applicable(invariant, NO_VERSION_BUILDS_TREES_FROM_THE_MKFS_WATERMARK);」。
  `into_report` 的优先级是违例 > 部分没判 > 射程里判不了 > 评估过则成立 > 不适用（理由或缺省），缺省句在 `crates/singlefs-checker/src/image.rs:358`。所以专门理由只在「一版都没认出、也没有读不出的版本」时才出现；有读不出的版本时报部分没判。
- 看门用例：`crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs:326`：「if *reason == singlefs_checker::walk::NO_VERSION_BUILDS_TREES_FROM_THE_MKFS_WATERMARK」。变异 `crates/mutations.tsv:2854`（判法提前 `return`，一版都不判）施加后红在这条用例的 I-5.5 断言上，
  断言消息是「I-5.5 在 E 之后的镜像上报「候选集里没有从 mkfs 水位第一次建树的那一版」的专门不适用理由：NotApplicable(…)」（`mut-results3.jsonl` k=11）。基线：`--include-ignored --exact` 那一条 `ok`（`results/tests-summary.txt` 的 t-base1）。
- 分不出的一形：「认出来而没有数据单元」不写理由（条款也这么写，`.claude/kb/invariants.md:199`），与提前返回报同一句缺省话（Q4）。
推翻它：造出一张「认出来而没有数据单元」的镜像，它的 I-5.5 报专门理由；或提前返回的变异在这条用例下存活。

### P5-b　B4 新取样点

判定：**补的点按判决做对了，抓到了原变体；取样点集仍留两段盲带**。
- 实现：`crates/singlefs-harness/src/model.rs:4333`：「const CONTAINER_COUNTS: [u64; 7] = [1, 2, 3, 4, 5, 100, 135];」（36 → 49 组合）。判决 b33 第二节 P1「B4」那一行要的是「补一个落在 60–118 之间的取样点（例 100）」。
- `mutations.tsv:2862` 施加后红在「现行 1 片、目标 100 片：回退写出的那一版最左…」那条断言上（k=27）。
- 窗口敏感性（我把变异里的窗口换成别的，`tools/b4_windows.py`；窗口按「最左之外的片数」，即片数减 1，含两端）：

  | 窗口（最左之外的片数） | 对应片数 | 结果 |
  |---|---|---|
  | 6..=58 | 7..59 | 绿，变异存活 |
  | 5..=98 | 6..99 | 绿，变异存活 |
  | 100..=133 | 101..134 | 绿，变异存活 |
  | 5..=99、6..=99 | 6..100、7..100 | 红 |
  | 101..=134、119..=134 | 102..135、120..135 | 红 |
  | 60..=118（34-3 那一行） | 61..119 | 红 |
  新点 100 只罩住含 99（= 100 片）的窗口；片数落在 6..99 或 101..134 内部的窗口变体仍存活，与 `.claude/rules/mutation-sampling.md` 第三类同形，补一个点没有关上这一类。
  这不违背判决（判决只要一个落在 60–118 之间的点），记下来供主 agent 决定要不要加点；每多一个点，组合数从 49 变 64，这条单测 0.00 秒。
推翻它：把窗口换成 6..=98 之后这条单测红（我量到绿）。

### P5-c　#9 三个钉值的取法与注释

判定：**取法与注释有两处对不上；钉值在 34-1 合入后必然不对（推的，旁证是实测）**。按派发不判钉值本身。
- 三个值：`crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs:845` 的打印行 sha `50615e47…`、同文件 850 行的进度文件名（计划哈希 `4987883029…`）、852 行的进度文件内容 sha `f6f10d5d…`。
- 注释对不上：同文件 829 行：「那棵树上重取（见下）」写的来历是「第 28 批 + 根环线 10e」那棵树；同文件 847 行：「打的行与内容的 SHA-256 取自 2026-10-05 主工作区」写的是形式化线第 34 批派出时那一版。两段并存、说法不同，829 行那句没跟着改。
- 取自合入 34-1 之前：847 行自己写「形式化线第 34 批派出时那一版，含删除线第 22 批」。34-1 改了 I-5.4 的逐状态判定，而 `GOLDEN_TALLY` 打每条不变量的「评估 / 违例 / 不适用」状态数
  （`crates/singlefs-checker-tier/src/crash.rs:841`：「"{invariant}={}/{}/{}",」）。旁证（harness 档，新池新建文件 mkfs 之后每个写操作前缀一个状态，共 58 个）：
  ```
  RV 开（冻结副本）  i54_tally={"Holds": 9, "NotApplicable": 49}  first_k={"Holds": 74, "NotApplicable": 25}
  RV 关（补丁 rvoff）i54_tally={"Holds": 5, "NotApplicable": 53}  first_k={"Holds": 78, "NotApplicable": 25}
  ```
  即 RV 让 4 个「记录已落、根槽没落」的状态从不适用变成成立。层 0 同一条流（`golden_new_pool_file_creation_stream`）上 I-5.4 那三个数会跟着变，所以 `50615e47…` 与 `f6f10d5d…` 在含 34-1 的树上对不上，
  与主 agent 已量到的「合入后打印行 sha 变成 4fc9cde7…」方向相合；那个变化里有 34-1 的份，不只是删除线第 24 批的。
- 建议（不替主 agent 定）：重取要在 34、删除线 24、根环线 10h 全部合入之后的树上做，且 847 行那句「已有各条计数不变」只对 34-1 之前成立，重取后要改。
推翻它：checker 档 golden 那条用例在含 34-1 的树上用 `50615e47…` 通过（我没跑）。

### P5-d　变异行（34-1 的 13 行、34-2 的 6 + 8 行、34-3 的 1 行、34-4 的 8 行，共 36 行）

判定：**锚点、点名、基线都站住；34 行 harness 档全红；有 2 行红在越界 panic 上**。
- 静态：`tools/select_rows.py`：每行整行在 `crates/mutations.tsv` 里恰好一次、与改动全文逐字相同、`find` 在源文件里恰好命中一次（36 / 36）。
- 动态（`tools/mutate_rows2.py`，每行施加、跑点名的测试、还原并核 sha256）：
  ```
  rows 34  exit101 34  named_test_failed 34  restore_ok 34  compile_err 0  compiled_pkg 34
  Counter({'断言（或自带消息 panic）': 32, '越界/算术/unwrap panic': 2})
  ```
  checker-tier 档两行（`mutations.tsv` 2855 行的 GPU 判器 RV、2644 行的 26-5 GPU 判器）没跑，GPU 对拍归提交时。
- 基线：这些行点名的测试在冻结副本上全绿（`results/tests-summary.txt`）。
- 同一个测试挂了四行（`an_all_zero_tree_table_pointer_in_an_older_candidate_root_…`，k=5、7–10）：每行红在不同的断言上（`first_panic` 逐行不同），不是同一条断言被反复打。
- **红在越界 panic 上的两行**：`mutations.tsv:955`（实审 B2 第 9 条）与 `mutations.tsv:965`（实审 B2b），都是 `index out of bounds: the len is 0 but the index is 0`。机理：替换文把 `allocation_roots` 清成空 `Vec`，
  后面按下标取，`crates/singlefs-checker/src/walk.rs:5329`：「tree_table.entries.is_empty() && !allocation_roots[0].all_zero;」。变异后文件里报的位置是 `walk.rs:5327:71`（替换把三行并成一行）。
  这两行的替换文在 git HEAD 的 `crates/mutations.tsv` 第 963 行（`git show HEAD:crates/mutations.tsv`）一字相同，34-4 只重取了 `find` 锚点；`allocation_roots[0]` 这个下标在改动全文里一处都没有（对 `_m3-formal-code-b34-r1-diff.md` 的 `grep -n` 命中 0 行），是 34 批之前就在的代码。
  所以不是 34-4 引入，但 34-4 重取后这两行仍然是「红了而没有证明那条互不相交的判在起作用」。
  不越界的替代变异我试了（`data/custom2.json` 的 ALT1：`vec![parse_node_pointer(&[0u8; 86])]`）：两条点名的测试都红在断言上（`results/custom2.out`：「两条记录罩住同一个槽，只该红 I-5.4：[]」与「写行那次发布之后：与登记的判定不符」）。
推翻它：任一行在副本上跑绿；或 29、30 两行换成 ALT1 后仍红在越界上。

## 7 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| P1-a 合法历史多报 | 一致（没打中，一次抽样） | harness 档 106 个前缀状态里 12 个走到下一次挂载先施加的那一版，I-5.4 全 `Holds`；applied-only 一类合法样本少；层 0 没跑，推的 |
| P1-b 新根段取处 | 一致 | `0..86` 与 `274..360` 对 D23 的新根段 99；M1–M4 四种取错字段都被新用例抓 |
| P1-c GPU 同组同判法 | 一致（读码，推的） | 版本收录条件、偏移、次序同 CPU；部分没判 GPU 不置位；GPU 没跑 |
| P1-d 条款与实现 | 规则没说 | I-5.4 行没写 applied-only 的下界、未到回收、反向链三条件，归写回批 |
| P2-a 合法历史误红 | 没打中 | 合法写者不写全零树表指针；实现员探针 0 次（他们的观测）；harness 全绿 |
| P2-b 放过整版 | 有三处放过（非本批引入） | applied-only 版只红 I-3.1 / I-3.5；next-mount 版 checker 绿而 core 拒；非候选根只有 I-9.28 部分没判 |
| P2-c 同一集合 | 不是 | 主走读、I-5.4、I-5.5、删除态与墓碑、I-9.28 各是不同的版本集合；条款只说「根」 |
| P2-d 与 core 同结局 | 同方向、不同结局 | 最新根全零 core 拒而不退回上一条完好的根；其余入口 checker 红、core 挂载成功 |
| P3-a core 入口 | 一致；GPU 恢复内核例外 | 四个读根槽入口都经 `parse_slot`；`crash_judge_common.wgsl:928` 选根还只判 extent 偏移（已知） |
| P3-b 两份测试 | 对 | 翻出生 txg 判的仍是「是否全零」那两行，两行新变异红；`both_mounts` 里两个读法现在进同一个 `judged`，判别力移到别处 |
| P4-Q1 | 不挡 | 候选 A / B / C / D；B 一行、121 + 4 条探针全绿；没有测试钉住 `NoTreeTable` |
| P4-Q2 | 不挡 | 候选 B 3 行，两条全零用例的期望列表要多一个 I-2.5，合法镜像 0 影响 |
| P4-Q3 | 不挡 | 候选 B 估约 6 行 WGSL 加一条对拍流加一行变异；GPU 没跑，推的 |
| P4-Q4 | 不挡 | 候选 B 3 行，没有测试依赖缺省理由；这一形在合法历史里可能不出现 |
| P5-a I-5.5 专门理由 | 分得出「没有对象」，分不出「认出来而没数据单元」 | 提前返回变异红在看门用例上 |
| P5-b B4 | 做对了，留两段盲带 | 100 只罩含 99 的窗口；6..99 与 101..134 内的窗口变体存活 |
| P5-c #9 钉值 | 注释对不上；钉值必然不对（推的） | 829 与 847 两处来历说法并存；取自合入 34-1 之前，RV 让 58 个前缀状态里 4 个 I-5.4 变了 |
| P5-d 变异行 | 站住；2 行红在越界 | 36 行唯一、锚点各一次；harness 34 行全红、点名的测试都跑到；`mutations.tsv:955`、`:965` 红在 `allocation_roots[0]` 越界 |

## 8 没做什么

- 不判别的腿的格，不替主 agent 采纳或出判决，P4 四问不替用户定。
- 没跑任何 checker 档的东西：GPU 对拍（含新加的那条）、GPU 变异行（`mutations.tsv` 2855、2644）、层 0、`crash_enumeration_*` 钉值、`mutations.tsv` 整表。GPU 一侧与层 0 一侧的结论都是「推的」。
- 没复跑 34-1 实现员「合法历史上探针 0 次」那一趟（他们的观测，只转述）。
- 没逐处造盘面验证 P2-b 里清点出的七处静默 skip（只清点了行号）。
- 没动 `crates/`（副本里的补丁与探针都在草稿目录，不进仓）；没动 kb。
- 没做「取成别的记录」那种变异（判为等价，推的）。
- 没跑 harness 档的全量（170 个二进制）；主 agent 的合入验证已有，我只跑了本报告点名的那一批与十二个合法历史样本，抬 F 与回退的慢档只跑了点名的一条。

## 9 复核不了

- GPU 对拍与 GPU 变异：要一张 NVIDIA 独显、归提交时（`.claude/kb/checks-owed.md:584` C735 同写）。
- 层 0 上的 I-5.4 / #9 钉值：checker 档，归提交时。
- 「合入后打印行 sha 变成 4fc9cde7…」是主 agent 的观测，我没量。

## 10 要主 agent 跑的命令

没有长活要交。复跑我这一趟：
```
S=<自己的草稿目录> bash research/prompts/m3-formal-code-b34-r1-sonnet-model/tools/rerun.sh setup build tests rows custom b4 probes patches
```
`rows` 与 `patches` 各超过 4 分钟，后台跑；跑完看 `$S/mut-results3.jsonl` 的 34 行（`exit` 全 101、`named_test_failed` 全真）与 `$S` 里各 `t-p-*.log`。

## 11 收尾核验

冻结副本与我的两份副本（编完、跑完、补丁全还原之后，删之前）逐文件核 sha256 清单，原样输出：
```
$ cd /tmp/claude-1000/m3-formal-code-b34-r1/tree && sha256sum -c ../crates-sha256.txt | grep -vc ': OK$'      → 0（315 行都 OK）
$ cd /tmp/claude-1000/m3-formal-code-b34-r1-sonnet/work && sha256sum -c …/crates-sha256.txt | grep -vc ': OK$'   → 0（315 行都 OK）
$ cd /tmp/claude-1000/m3-formal-code-b34-r1-sonnet-rr/work && sha256sum -c …/crates-sha256.txt | grep -vc ': OK$' → 0（315 行都 OK）
```
副本里额外的探针文件 `zz_probe_*.rs` 不在清单里，不算改动；仓里 `crates/` 我没动。中途我的一趟自拟变异被 240 秒上限截断过一次（M4），副本当时留着变异，我在下一趟之前发现 sha256 不对并从冻结副本还原；
被截断的那趟没有输出，M4 之后重跑一次、出了结果（`results/custom1b.out`），其余各趟的 `restore ok` 都是脚本自己核过的。

已删的自建目录（编译目录与仓副本，共约 9 GB，结果与日志已落进模型目录 `results/`，复跑靠 `tools/rerun.sh`）：
- `/tmp/claude-1000/m3-formal-code-b34-r1-sonnet/work`、`/tmp/claude-1000/m3-formal-code-b34-r1-sonnet/target`
- `/tmp/claude-1000/m3-formal-code-b34-r1-sonnet-rr/work`、`/tmp/claude-1000/m3-formal-code-b34-r1-sonnet-rr/target`（验证 `tools/rerun.sh` 能复跑出同样的输出，用的第二份副本与编译目录）

留着的是两个草稿目录里的日志与 `rows.json`（不是编译目录，也不是仓副本）。

cite-check 末行（`python3 scripts/cite-check.py research/prompts/m3-formal-code-b34-r1-sonnet-output.md --root /tmp/claude-1000/m3-formal-code-b34-r1/tree --root . --background research/prompts/_m3-formal-code-b34-r1-background.md`）：
```
  ✓ 核了 33 处引文，对上 33 处，没判 0 处（逐处列在上面）
```
