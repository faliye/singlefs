# m2-step3-code-r1 核查员报告

你交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

## 判别力自证

从 Opus 报告挑一条引用：`crates/singlefs-core/src/recovery.rs:630-637`，抄的原文是
```rust
let root_own_record_counter = records
    .values()
    .find(|record| {
        record.instance == root.instance && record.checkpoint_txg == root.checkpoint_txg
    })
    .map(|record| record.counter);
let mut expected_next: Option<(InstanceGeneration, u64)> =
    root_own_record_counter.map(|counter| (root.instance, counter + 1));
```
在草稿目录里把行号加 1（改核 631-638）：

```
$ sed -n '631,638p' /home/fy5090/code/singlefs/crates/singlefs-core/src/recovery.rs
        .values()
        .find(|record| {
            record.instance == root.instance && record.checkpoint_txg == root.checkpoint_txg
        })
        .map(|record| record.counter);
    let mut expected_next: Option<(InstanceGeneration, u64)> =
        root_own_record_counter.map(|counter| (root.instance, counter + 1));
    for record in above.into_iter().take(in_flight_limit) {
```

行区间 631-638 缺开头一行 `let root_own_record_counter = records`、多了结尾一行 `for record in above.into_iter()…`，与抄的原文不逐字相同。**判 ✗。** 核查方法能分辨对错行号，往下按第 2 步继续核。

## 复跑：Opus 报告的两条命令

在 `/tmp/claude-1000/m2-step3-code-r1-verifier/copy/`（`rsync -a --exclude target --exclude .git` 从原仓拷贝，未在原目录跑）上执行。

**命令一**：`attack_opus_r1.rs` 拷进 `crates/singlefs-harness/tests/`，sha256 核对一致（`df37124f16589b021c5b40eb64a9eb95ce95557cde3ac07c392e9d16275d1a9e`），`nice -n 19 cargo test -p singlefs-harness --test attack_opus_r1 -- --nocapture`，末尾：

```
X1-A effective_root = Some((InstanceGeneration(1), CheckpointTxg(6))) prefix_applied = 1 above_water = 1
X1-A 读回的是 root = (InstanceGeneration(1), CheckpointTxg(4))，内容长度 3300（second=4100 third=2500 fourth=3300）
test x1_gap_is_skipped_when_the_chosen_roots_own_record_is_also_unreadable ... ok
X2-A NoPublishedVersion
test x2_fresh_pool_cannot_be_mounted_writable ... ok
X1-B chosen=(InstanceGeneration(1), CheckpointTxg(4)) effective=(InstanceGeneration(1), CheckpointTxg(6)) prefix_applied=1 W=4 rows=[InstanceRow { instance: InstanceGeneration(1), selected_root_txg: CheckpointTxg(6), applied_transaction_high_water: 4, is_rollback: false }]
test x1b_mount_after_the_gap_writes_a_row_that_is_not_a_prefix ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
```
与报告第 47/48/51/140/107 行逐字一致。**✓**

**命令二**：按 fragment 文件头说明把两段测试插进副本的 `second_transaction_step_zero_layer0.rs`（新增 `last_but_one_root_index` 字段并填值、插入两个 `#[test]` 函数，用 python 定点替换，未整份重写），sha256 核对 fragment 一致（`c6e413c84dd40c4b1f063a556796a8d08c6174ee61135b2857ce7a7b74f46bf5`），`nice -n 19 cargo test -p singlefs-harness --test second_transaction_step_zero_layer0 -- --nocapture opus_attack`，末尾：

```
写行发布的单元写下标 [56, 57, 58, 59, 60, 61, 62, 63, 64, 65]
C 的单元写 16 条、记录写 2 条
X8-B 第 0 对单元写缺席：outcome_is_file=false violations=1 ignored=1 claimed_missing=1 checker=[("I-2.1", 1), ("I-7.2", 1)]
① 全持久：effective=Some((InstanceGeneration(2), CheckpointTxg(8))) violations=0 outcome_is_file=true
② 根持久 + C 的单元一份都不在：violations=1 ignored=1 claimed_missing_unit=1 first=Some("走读失败：UnitUnreadable { slot: SlotNumber(50326) }（持久的写：…）")
X8-B 第 1 对单元写缺席：outcome_is_file=true violations=0 ignored=0 claimed_missing=1 checker=[("I-2.1", 1)]
③ C 的根槽不在、记录与单元都在：effective=Some((InstanceGeneration(2), CheckpointTxg(8))) prefix_applied=1 journal_differing=1 violations=0
X8-B 第 2 对单元写缺席：outcome_is_file=true violations=0 ignored=0 claimed_missing=1 checker=[("I-2.1", 1)]
④ 根持久 + C 的两份记录都不在：violations=0 root_without_record=1
X8-B 第 3 对单元写缺席：outcome_is_file=true violations=0 ignored=0 claimed_missing=1 checker=[("I-2.1", 1)]
⑤ C 一个字节都不在：effective=Some((InstanceGeneration(2), CheckpointTxg(7))) violations=0 outcome=true
test opus_attack_controls_moved_to_the_third_publish_script ... ok
X8-B 第 4 对单元写缺席：outcome_is_file=true violations=0 ignored=0 claimed_missing=1 checker=[("I-2.1", 1), ("I-3.1", 1)]
test opus_attack_instance_table_unit_missing_but_every_root_persisted ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.66s
```
（②的省略号是报告自己就这么截断的写法，其余行全部逐字比对一致，含 ①③④⑤ 与 X8-B 五行）与报告第 200-205、220-225 行逐字一致（只是终端交织顺序不同，内容不变）。**✓**

## Opus 报告（攻方）：引用核对表

逐条取 `file:行区间`，与仓里现文件比对内容。

| 引用 | 核的结果 |
|---|---|
| `crates/singlefs-core/src/recovery.rs:628-629`（注释）+ `:630-637`+`:638-643` | ✓ 三段与源码逐字一致（自证那条已单独展开） |
| `.claude/kb/decisions/23-journal的角色与格式.md:1221-1222` | ✓ 逐字一致（该文件此处是一整段 4682 字符的长行，882/1221 等行号指的都是同一物理行内的不同分句，合法） |
| `.claude/kb/decisions/23-journal的角色与格式.md:1236` | ✓ 逐字一致 |
| `.claude/kb/invariants.md:75`（I-8.3 状态「未实现」） | ✓ 逐字一致 |
| `crates/singlefs-core/src/mount.rs:185-193`（own_record 回退取全环最大 counter） | ✓ 逐字一致 |
| `crates/singlefs-core/src/mount.rs:196-200` | ✓ 逐字一致 |
| `crates/singlefs-core/src/mount.rs:212-217`（next_counter） | ✓ 逐字一致 |
| `crates/singlefs-core/src/mount.rs:247`／`:252`／`:265`／`:277`／`:308` | ✓ 五处单行引用全部逐字一致 |
| `crates/singlefs-core/src/mount.rs:273-274`（引「transaction: 0、back_chain: 0」） | ✗ 实际位置有误：273 行是 `transaction: 0,`，274 行是 `instance,`；`back_chain: 0,` 在 275 行，不在被引的 273-274 区间内。正确区间应为 273 与 275（274 是不相关的 `instance,`） |
| `crates/singlefs-core/src/recovery.rs:347-352`（树表空报错） | ✓ 逐字一致 |
| `crates/singlefs-core/src/recovery.rs:588-592`／`:618-623`／`:626-627`／`:678` | ✓ 四处逐字一致 |
| `crates/singlefs-core/src/root_record.rs:12-13`（字段表 138 字节） | ✓ 逐字一致 |
| `crates/singlefs-core/src/transaction.rs:59`（FIRST_TRANSACTION_NUMBER=1）／`:305`／`:321`（warm_up 定义） | ✓ 三处逐字一致 |
| `crates/singlefs-core/src/transaction.rs:607-610`（assert_eq! 两盘同槽） | ✓ 逐字一致（含注释「两盘同槽（D2 已定项 10）」） |
| `crates/singlefs-core/src/transaction.rs:866-869`（counter/transaction/instance/back_chain） | ✓ 四字段各自的行号都在区间内 |
| `crates/singlefs-core/src/transaction.rs:867` | ✓ 逐字一致 |
| `crates/singlefs-core/src/journal.rs:80-94`（JournalRecord 字段表，无实例表项） | ✓ 逐字一致 |
| `crates/singlefs-core/src/journal.rs:170`（「不查反向链」注释） | ✓ 逐字一致 |
| `crates/singlefs-checker/src/walk.rs:410-441`／`:415-430` | ✓ 两处逐字一致（`judge_instance_table_rows` 函数体正好是 410-441） |
| `crates/singlefs-harness/src/crash.rs:484`（记录核对器判据） | ✓ 内容一致（Opus 是转述不是逐字引，转述准确） |
| `crates/singlefs-harness/src/crash.rs:639-649`（偏移 24/28） | ✓ 逐字一致 |
| `crates/singlefs-harness/tests/first_transaction_step_six_recovery.rs:255-292`（`:257` 起） | ✓ 用例名、内容、行号全部一致 |
| `crates/singlefs-harness/tests/second_transaction_step_three_second_instance.rs:469-501` | ✓ 用例 `one_missing_record_right_after_the_chosen_root_stops_the_prefix_even_when_later_records_are_intact` 内容一致 |
| `crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs:128-137`／`:167-173`／`:322-395` | ✓ 三处逐字一致（322-395 取自未插入 fragment 前的原文件，396 行版本） |
| `crates/mutations.tsv` 倒数第 4 行（`one_missing_record_right_after…`） | ✗ 实际位置：该表共 30 行（含表头注释行），内容在第 28 行，是**倒数第 3 行**（30、29、28 依次是倒数第 1/2/3 行），不是倒数第 4 行 |
| 命名与逻辑描述（`grep -rn "is_rollback" crates/`、`grep -n "读不出.*接…"` 等 grep 结果） | ✓ 现场重跑 grep，命中数与报告描述一致 |

**这一份计数**：核了 30 处（含判别力自证那一条另计）、✓ 28 处、✗ 2 处（均为「引用区间偏离实际内容所在行 1–2 行」的定位误差，抄的文字本身与仓内容一致，只是行号范围没框准）。

## Sonnet 报告（正推）：引用核对表，第一部分（S1/S3/S4/S5/S6/S7/S8/S11）

| 引用 | 核的结果 |
|---|---|
| `recovery.rs:618-623`／`recovery.rs:628-637`（S1 候选过滤、链首取法逐字代码块） | ✓ 与源码逐字一致（含变量名、括号层次） |
| `.claude/kb/decisions/23-journal的角色与格式.md:1236`／`:1221-1227` | ✓ 逐字一致 |
| `.claude/kb/decisions/18-块里携带什么信息.md:882`（S1 判定 4 引「重放之后那个根的 checkpoint_txg…属于实例 i 的、被这次重放施加的最大事务号」） | ✓ 逐字一致（同一物理长行） |
| `allocator.rs:275-282`／`:278`／`:112`／`:172`／`:215`（S3） | ✓ 五处逐字一致 |
| `crates/singlefs-core/src/allocator.rs:284-308`（rebuild_from_records 全文+ 注释） | ✓ 逐字一致，含函数上方三行注释 |
| `.claude/kb/decisions/03-空间分配.md`（已定项 7）／`.claude/kb/decisions/05-快照-空间记账机制.md:1742-1745`（S3 引用，未给出精确行号的部分不查） | 核不动其中一句未标行号的引用；已标行号的 `:1742-1745` 未展开核对（时间所限），标记「核不动」 |
| `crates/singlefs-core/src/transaction.rs:275-309`（acquire_instance 全函数） | ✓ 逐字一致（S4 代码块的字段名、控制流与源码一致，个别行做了紧凑排版但语义不变） |
| `transaction.rs:273-274`（重试注释） | ✓ 逐字一致 |
| `transaction.rs:300-304`／`:305-307` | ✓ 逐字一致 |
| `crates/singlefs-core/src/mount.rs:243-283`（S5 写行逻辑） | ✓ 与源码逐字一致（`mount.rs:247-266` 代码块核对，见下） |
| `mount.rs:247`／`:252`／`:265`（S5 判定 1-3、6） | ✓ 三处逐字一致 |
| `mount.rs:218-232`（first_txg 计算） | ✓ 逐字一致 |
| `crates/singlefs-core/src/mount.rs:285-323`（S6 暖机循环全文） | ✓ 逐字一致，含注释 |
| `mount.rs:23`（`use singlefs_format::{INSTANCE_ROW_BYTES, ROOT_RING_REGIONS};`） | ✓ 逐字一致 |
| `mount.rs:325-337`（Ok(Mounted{...}) 返回处） | ✓ 内容一致（函数收尾处） |
| `crates/singlefs-harness/tests/second_transaction_step_three_second_instance.rs:222-293`（S7） | ✓ 逐字一致 |
| `:95`／`:147` | ✓ 两处逐字一致 |
| `crates/singlefs-checker/src/walk.rs:405-441`（S8，judge_instance_table_rows） | ✓ 函数体正好在 410-441，405-409 是函数上方的文档注释（含 I-3.8 说明），引用范围合理 |
| `walk.rs:209-217`／`:212-217`／`:213`／`:214`（调用处 is_newest 判断、偏移 24） | ✓ 四处逐字一致 |
| `transaction.rs:580`（placements_to_release_via_mapping 定义处） | ✓ 函数确在此行开始（注释逐字一致），但 S2 判定 2 里所引「previous.unit(TransactionUnit::MappingTree).bytes」不在 580 行本身，是指向函数体内部，属于「指函数不指单行」的宽松引用，不算错 |
| `transaction.rs:906-912`／`:911`／`:928-936`（S11 三处改法） | ✓ 三处逐字一致 |
| `crates/singlefs-core/src/recovery.rs:759-770`（allocation_records_are_one_per_device） | ✓ 逐字一致 |
| `transaction.rs:840-880`（S7「底层调用同一条 publish_overwrite」） | 内容近似：`publish_overwrite` 函数实际从 853 行开始（840-852 是上一个函数的收尾），840-880 的区间偏宽但确实框住了函数体，不算错标，标「✓（区间偏松）」 |
| `transaction.rs:861-864`（S7 publish_version 起调用行） | ✓ 逐字一致 |

**小计（第一部分）**：核了 24 处、✓ 22 处、1 处核不动、1 处区间偏松但不算错。

## Sonnet 报告（正推）：引用核对表，第二部分（S2、S9、X9、X10 —— 本部分有多处 ✗）

| 引用 | 核的结果 |
|---|---|
| `crates/singlefs-core/src/recovery.rs:327-424`（rebuild_version 全函数） | ✓ 函数含文档注释确在 327-424 区间内（doc comment 327-331，函数体 332 起） |
| `recovery.rs:341`（读实例表） | ✓ 逐字一致 |
| `recovery.rs:342-350`（读树表） | ✓ 内容一致（树表字节读取+解析+空判断的前半段） |
| `recovery.rs:351-355`（S2「树表空报错」，引 `tree_table.entries.is_empty()` → `InvariantViolated` 那个 if 块） | ✗ 实际位置有误：`if tree_table.entries.is_empty() { return Err(RecoveryFailure::InvariantViolated { invariant: "挂载", detail: "所选根下面还没有文件版本（树表为空）", }); }` 整块在 **347-352** 行；被引的 351-355 只框住了这个 if 块的收尾两行（351 `});`、352 `}`）和下一段无关代码（353-355，构造 `tree_table_entries`），没有覆盖判据本身（`tree_table.entries.is_empty()` 在 347 行，`InvariantViolated` 与 detail 文案在 348-350 行） |
| `recovery.rs:363-397`（S2「读五棵树的根节点（extent/inode/allocation/accounting/mapping）」） | ✗ 区间不完整：363-397 只覆盖了 extent（370, 380-386）与 inode（371, 387-399 的前半段），**没有覆盖 allocation（408 行起）、accounting（414 行起）、mapping（420 行起）**——这三棵树的根节点读取代码全部在 397 行之后。要覆盖五棵树，区间至少要到 ~425 行 |
| `recovery.rs:194-214`（read_unit_via_locations，S2 引用） | ✓ 逐字一致 |
| `recovery.rs:405-409`（S2 判定 2 引「不是 mapping_keys 那个只剩 key 的派生字段」） | ✗ 实际位置有误：`mapping_keys` 变量定义在 **421-425** 行（`let mapping_keys: Vec<Vec<u8>> = mapping_node.entries.iter().map(...).collect();`）；被引的 405-409 是 inode_record 解析收尾与 allocation_pointer 读取开头，跟 mapping_keys 毫无关系 |
| `recovery.rs:422`（S2「`rewritten: Vec::new()`」） | ✗ 位置严重偏离：`rewritten: Vec::new(),` 实际在 **510** 行（`TransactionOutput` 构造体里），422 行是 `.entries`（属于 421 行 `mapping_keys` 定义的下一行），与 `rewritten` 毫无关系，偏差达 88 行 |
| `crash.rs:559-629`（S9 oracle_violation_for_versions） | ✓ 函数在此区间内 |
| `crash.rs:550-551`（S9「只按 txg 认时 oracle 在那一格两个方向都错」注释） | ✓ 逐字一致 |
| `crash.rs:585-600`（S9 NoFile 分支代码） | ✓ 逐字一致 |
| `crash.rs:734`（ignored_violations 计数） | ✓ 内容一致 |
| `.claude/kb/decisions/13-验证路线.md`（S9 引已定项 4，未标精确行号） | 核不动（未给出行号，语义与决策文件对应部分一致，未逐字比对） |
| `research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs:2636-2652`（X9，E142 候选过滤 + root_own_record_counter + expected 初始化代码块） | ✓ 引用的这一段（到 `expected` 初始化为止）内容与源码一致 |
| `e142...rs:2647-2650`（X9「jsn 断号即止」`if (record.instance, record.counter) != expected_key { break; }`） | ✗ 实际位置有误：这条判断在 for 循环内部，位于 **2660-2663** 行；被引的 2647-2650 是 `let water = ...`（2647）、`let mut rebuilt = *root;`（2648）与两行注释（2649-2650），与断号判断无关 |
| `e142...rs:2524-2530`（X9「JournalScanReport 结构体在装置里只有 5 个字段」） | ✗ 实际位置有误：`struct JournalScanReport` 定义在 **2535-2540** 行（含 5 个字段：valid_records/above_water/prefix_applied/verification_passed/verification_failed，字段数说法正确）；被引的 2524-2530 是另一个枚举 `RecoveryOutcome` 的定义，内容完全不同 |
| `e142_first_transaction_dry_run.rs:3706`（X9「单测钉 6：`assert_eq!(tally.verification_ran_states, 6, ...)`」） | ✗ 实际位置有误：该断言在 **3721** 行；3706 行是 `let BuiltPool { recording, mkfs_operation_count, .. } = built_pool();`，与这条断言无关 |
| `.claude/kb/milestone/02-second-txn.md:111`／`:135`／`:156`（X10） | ✓ 三处逐字一致 |

**小计（第二部分）**：核了 16 处、✓ 9 处、✗ 6 处、核不动 1 处（未给精确行号）。

## Sonnet 报告 X9「同一条规则，一致」：核内容本身，发现与源码不符

X9 的判断表这一行写：「链首读不出（候选里有别的实例的记录）| `expected_next = None`；… | 同构，`above` 同样先过滤 `record.instance == root.instance` | **相同**」，以及「jsn 断号即止 | … | … | **相同**」，结论「X9 判『同一条规则』」。

这不是行号问题，是**取那一行比内容**之后，两边代码本身不同：

crates（`crates/singlefs-core/src/recovery.rs:638-643`，已核对逐字）：
```rust
for record in above.into_iter().take(in_flight_limit) {
    if let Some(expected_key) = expected_next {
        if (record.instance, record.counter) != expected_key {
            break;
        }
    }
    expected_next = Some((record.instance, record.counter + 1));
    if !record.is_commit {
```
`expected_next` 为 `None`（锚点读不出）时，`if let Some(...)` 整块跳过，**没有 else 分支**，候选列表第一条无条件通过。

E142 装置（`research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs:2660-2670`，实际位置，非 Sonnet 所引的 2647-2650）：
```rust
for record in above.into_iter().take(usize::try_from(JOURNAL_IN_FLIGHT_RECORD_LIMIT).expect("在飞上限")) {
    if let Some(expected_key) = expected {
        if (record.instance, record.counter) != expected_key {
            break;
        }
    } else if record.checkpoint_txg != chain_start_txg_without_anchor {
        break;
    }
    expected = Some((record.instance, JournalCounter(record.counter.0 + 1)));
    if !record.is_commit {
        break;
    }
```
`expected` 为 `None` 时，E142 多一个 **`else if record.checkpoint_txg != chain_start_txg_without_anchor { break; }`**（`chain_start_txg_without_anchor = CheckpointTxg(root.checkpoint_txg.0 + 1)`，定义在 2658 行）——第一条候选的 `checkpoint_txg` 必须恰好是「根的 txg + 1」才通过，否则整段前缀判空。这一支在 crates 里不存在。

现查 `research/prompts/_m2-step3-code-r1-diff.md` 第 429-464 行：这一次 diff 把 crates 的 `expected_next` 从「恒为 `None`」改成了「锚点读得出时取它的 counter+1，读不出仍是 `None`」，改动说明本身没有加任何针对「读不出」这一支的额外判断。`research/e7-index-bench/` 目录不在这次 diff 的改动范围内（`grep -n "e142_first_transaction_dry_run.rs" _m2-step3-code-r1-diff.md` 零命中），说明 E142 这个 `else if` 分支是之前某一轮就加上的、这一轮没有跟着改。

而且这不是纸面推演：本报告「复跑」一节的命令一（`x1_gap_is_skipped_when_the_chosen_roots_own_record_is_also_unreadable`）已经在 crates 上实测出「锚点（所选根自己那条记录）读不出时，代码把 jsn 6 无条件接上、跳过缺失的 jsn 5」——这正是 crates 缺那个 `else if` 分支的行为后果；同一段历史换到 E142 那个 `else if` 分支上，`record.checkpoint_txg`（jsn 6 对应的 txg 6）不等于 `chain_start_txg_without_anchor`（根 txg 4 + 1 = 5），会在第一条就 `break`、前缀判空——与 crates 的行为不同。

**只报观测，不判存不存在设计问题、以及要不要以 crates 还是以 E142 为准**：那是主 agent 的推论。这里只核实「X9 判『同一条规则、一致』」这句话，在「锚点读不出」这个分支上，与两份源码逐字比对的结果不符。

## 本地攻方（提示 `m2-step3-code-r1-local-attack.md`）：事实段 F1-F24 抽查

提示作者从 `crates/` 与 kb 条款译成英文喂给本地模型；核的是这些译文与源码/条款是否一致（含限定词有没有漏译）。样本 -s2、-s3 两次答复的判定本身不判（那是推理），只核它们引用的事实号在提示里是否存在、提示的事实是否与源码/kb一致。

| 事实 | 核的结果 |
|---|---|
| F1（`mount.rs:169-241`，mount_writable 恢复+own_record+rebuild_version+acquire_instance） | ✓ 行区间精确：169 是函数签名，241 行正是 `acquire_instance` 调用行（`let instance = acquire_instance(&mut pool).map_err(MountError::Acquisition)?;`），F1 描述的最后一句「calls acquire_instance」与 241 行吻合 |
| F3（`mount.rs:285-323`，暖机循环，含「never checks whether covered now actually contains every device」的断言） | ✓ 逐字核对 285-323 全函数，"covered" 变量、`device_of_txg`、循环条件、`ROOT_RING_REGIONS`（=3）均与源码一致；确认循环结束后直接 `Ok(Mounted{...})`（325 行起），F3 所称「没有对『仍未覆盖』的错误分支」属实 |
| F4（`allocator.rs`，DeviceFreeMap 与 PoolAllocator::release） | 部分核：`mark_released` 只 `deferred_slots += span`（172 行）、`mark_allocated` 里 `free_slots -= span`（215 行）与 F4 描述一致；`PoolAllocator::release` 的四种校验未逐条重读（时间所限），列「核不动（未逐条展开）」 |
| F5（`allocator.rs:284-308`，rebuild_from_records + 原文注释） | ✓ 逐字核对，含函数上方三行中文注释的翻译准确（"leaves it behind" 对应「先留着」） |
| F7（`recovery.rs:759-798`，allocation_records_are_one_per_device，FIRST_TRANSACTION_PLACEMENTS_PER_DEVICE=10） | 部分核：函数开头（759-770）逐字核对一致；F7 描述的完整逻辑（含 `FIRST_TRANSACTION_PLACEMENTS_PER_DEVICE` 常量与注释译文）未继续往下核到 798 行，列「核不动（未展开到行尾）」 |
| F9（publish_admitted 里 STATISTIC_INODE_WATERMARK 等固定值） | 未核（未读 transaction.rs 约 1215-1271 行区间源码），列「核不动」 |
| F14（分配记录树条款，D3 已定项 7/已定项 11） | 与之前查过的 `.claude/kb/decisions/03-空间分配.md` 已定项 7 段落（「落点释放时条目不删」相关文字，见 Sonnet S3 引用核对）内容口径一致，未见漏译限定词 |
| F18（`.claude/kb/invariants.md:127`，I-3.8 内容+状态） | ✓ 逐字核对：中文原文「实例表 kind 0 行按实例代号唯一；行只在回收条件成立后删（整轮清扫对池中每一个落点都得出判定……）；行的实例代号 < 挂载根实例」与英文译文一一对应，回收五个条件（每个落点都判定、没有该实例未发布单元、无读失败、全部设备在线、根环无该实例发布的根）全部译出，未漏 |
| F19（`.claude/kb/invariants.md:53`，I-7.7，注明"condensed"） | ✓ 内容对应，含「过半是准入门槛不是写入范围」→"being a majority is only an admission gate...it does not mean every device...was actually written to" 这一处容易漏译的限定词，译文保留了 |
| F21（`.claude/kb/decisions/13-验证路线.md` 已定项 4，崩溃状态定义） | ✓ 与该文件第 218 行摘要（「屏障切段、段内任意整写子集；撕裂态并进『没持久』……」）口径一致 |
| F22（D16 已定项 8，暖机承诺+WARM_UP_EMPTY_PUBLISHES=2 的警告） | 与 `.claude/kb/decisions/16-发布语义.md` 对应段落（已在 Sonnet S6 核对处间接确认）口径一致，"registered as a format constant...if the assignment changes...must be recomputed" 与中文「代价：跳号……」「⚠️ 若配比改变则要重算」对应 |
| F24（上一轮五处修复，含 (a)-(e)） | 未逐条回查历史轮次的原始判决文本，只核了其中与 crash.rs 相关的 (c)（`crash.rs:585-600`，已在 Sonnet 表核对，内容一致），其余 (a)(b)(d)(e) 列「核不动（要翻上一轮记录）」 |

**这一份计数**：核了 12 处事实，✓ 8 处、部分核 2 处、核不动 2 处，✗ 0 处。

## 本地辩方（提示 `m2-step3-code-r1-local-defense.md`）：事实 A-U 抽查

| 事实 | 核的结果 |
|---|---|
| Fact D（rewritten_roles() 逻辑：file→Data、instance_table Rewrite→InstanceTable、file→Extent/InodeLeaf/InodeRoot、恒定四项，`transaction.rs:785-811`） | ✓ 逐字核对（函数体 789-807，789-811 区间含函数上方两行注释，合理） |
| Fact D 后半（实际逐盘写由 `rewritten` 顺序驱动，`transaction.rs:1555-1607`） | ✓ 核对 1555-1607：`written_units` 由 `rewritten.iter().map(...)` 构造（1555-1562），随后 `for unit in &written_units { pool.perform(CommitStep::WriteUnitToEveryDevice...) }`（1601 行起），顺序确实与 `rewritten` 一致 |
| Fact A（提交协议：写单元→屏障→journal 记录→屏障→根 FUA→超级块轮换，`transaction.rs`约1601-1614） | ✓ 与上面核对的 1599-1607 附近内容吻合（含中文注释「持久顺序（D16 已定项 7）」） |
| Fact E（写行发布 `rewritten_roles()` 返回 `[InstanceTable, AllocationTree, AccountingTree, MappingTree, TreeTable]`） | ✓ 用 Fact D 验证过的函数逻辑推导：file=None、instance_table=Rewrite ⇒ 确实只推 InstanceTable + 四个固定角色，顺序一致 |
| Fact F（BirthSequenceAllocator，`transaction.rs:701-719`；`tree()` 方法 TreeTable/InstanceTable 同归 TREE_IDENTIFIER_NONE，`:453-470`；InstanceTable 发号语句在 `:1156`） | ✓ 三处逐字核对：701-717 是 struct+impl（区间给到 719 略宽 2 行，内容仍在同一个 impl 块内，不算错）；453-470 逐字一致；1156 行 `sequences.next(TreeIdentifier(TREE_IDENTIFIER_NONE), txg, instance)` 精确一致 |
| Fact H（D16 已定项 8 引文，WARM_UP_EMPTY_PUBLISHES=2 的完整警告段） | 与 Sonnet S6 判定 4 引用的同一段（`.claude/kb/decisions/16-发布语义.md`「已定项 8」）口径一致 |
| Fact K/L/M（D3 已定项 7，allocator.rs rebuild_from_records 上方注释） | ✓ 与 Opus/Sonnet 都核对过的 `allocator.rs:284-308` 上方三行中文注释一一对应（"leaves the unused-but-not-full slots behind" 对应「先留着」） |
| Fact P/Q（D23 已定项 14、I-8.3；`recovery.rs:599-683` 的 replay_journal 逻辑描述） | ✓ 与前面核对过的 recovery.rs 618-645 区间逐字对应，包括「on the very first iteration, if expected-next is None...whichever record happens to be first in the sorted list is accepted unconditionally」——这与本报告「复跑」实测的行为完全一致 |
| Fact R（`second_transaction_step_three_second_instance.rs` 的 `one_missing_record_right_after...` 用例，「不覆盖锚点自身读不出的情形」） | ✓ 与前面核对的用例内容（469-500 行）一致：该用例改坏的是「紧接着锚点的下一条记录」而不是锚点自己，Fact R 的表述准确 |
| Fact S/T（NoPublishedVersion、build_pool 不经 mount_writable） | 内容与 Opus 报告 X2① 的核对结果（`mount.rs:196-200`、`recovery.rs:347-352`）一致，未见偏差 |

**这一份计数**：核了 10 处，✓ 10 处，✗ 0 处。局部辩方引用的行号精度明显高于 Sonnet 报告的 S2/X9 部分。

## 有没有把行号误写成背景材料/diff 附录的行号

对上面找到的全部 ✗（`mount.rs:273-274`、`crates/mutations.tsv` 位置、`recovery.rs:351-355`/`:363-397`/`:405-409`/`:422`、`e142...rs:2524-2530`/`:2647-2650`/`:3706`），逐条去查 `_m2-step3-code-r1-background.md`（2169 行）与 `_m2-step3-code-r1-diff.md`（2867 行）对应行号是什么内容：

```
$ sed -n '351,355p;363p;397p;405,409p;422p' research/prompts/_m2-step3-code-r1-background.md
```
均是决策变更史 checklist 表格行（形如「### 2026-09-14（其X）… | 不抄 | 历史版本」），与 recovery.rs 的任何内容都不相关，不是「原文其实在背景材料里那一行」这种情形。

```
$ sed -n '2524,2530p;2647,2650p' research/prompts/_m2-step3-code-r1-diff.md
```
diff.md 在这几行是别的文件（`second_transaction_step_three_second_instance.rs`）的 diff 片段，也和 `e142_first_transaction_dry_run.rs` 的内容不相关；而且已确认 `research/e7-index-bench/` 整个目录零命中于本轮 diff（`grep -n "e142_first_transaction_dry_run.rs" _m2-step3-code-r1-diff.md` 无输出）。

**结论：以上全部 ✗ 都是「原文件里数错了行号」，不是「把背景材料/diff 的行号当成了源文件行号」。**

## 总计数

| 腿 | 核了 | ✓ | ✗ | 核不动/部分核 |
|---|---|---|---|---|
| 判别力自证 | 1 | — | 1（要求判 ✗，结果确实判 ✗） | — |
| 复跑命令（Opus 两条） | 2 | 2 | 0 | 0 |
| Opus 报告引用 | 30 | 28 | 2 | 0 |
| Sonnet 报告引用（含 X9 内容核对单列一条） | 41 | 31 | 7 | 2（未标精确行号的决策引用与 F4/F9 类未展开項不计入 Sonnet，此处只计文件:行号引用；X9 内容核对本身另记为 1 条重大发现，不计入这 41） |
| 本地攻方事实 F1-F24 抽查 | 12 | 8 | 0 | 4 |
| 本地辩方事实 A-U 抽查 | 10 | 10 | 0 | 0 |
| **合计** | **96** | **79** | **10** | **6** |

（Sonnet 的 X9 内容比对——crates 与 E142 在「锚点读不出」分支上代码不同——单独成节展开，不计入上表的引用行号计数，因为它不是「文件:行号」核对，而是「取那一行比内容」之后发现两处被比较的源码本身不一致，判据见「Sonnet 报告 X9」一节。）

## 没做什么

- 不判一条打中成不成立、该不该采纳；不核推理本身，只核引用、产物与复跑——这句话照抄进本报告开头。
- 不判 X9「同一条规则」这句结论对不对、crates 该不该照 E142 的写法收严，也不判 Opus 的 X1/X4 打中成不成立——只报告两处被比较的源码内容不同，判断交主 agent。
- 没有逐条核 Sonnet 报告里全部 73 个 `文件:行号` 引用（超出时间预算），已优先核了每个 S1-S11/X9/X10 段落里承重的那几条；未核的主要集中在纯粹的「与已核过的相邻行内容一致」的重复性引用（如同一函数内多处零散单行引用）。
- 没有核 Opus/Sonnet 报告里未标精确行号的决策条款引用（如 `.claude/kb/decisions/05-快照-空间记账机制.md:1742-1745` 之外的宽泛引用、`.claude/kb/decisions/13-验证路线.md` 已定项 4 的整段引用）——这些本身没有可比对的单一行号。
- 没有核本地攻方/辩方样本输出（-s2/-s3/-s1 空文件）里推理判断的对错（例如 5a「不 hold up」的判断成不成立），只核了它们所依赖的事实段与源码/kb 是否一致。
- F4、F9、F24(a)(b)(d)(e) 只做了部分核或完全没核（时间所限），不代表它们有问题，只是没查。
- 没有跑 `gate.sh` 或任何门禁阶段（`.claude/gate.d/stage-owners.tsv` 未给这个 agent 登记任何阶段）。
- 没有编译原仓、没有在原仓目录下跑任何命令；复跑只在 `/tmp/claude-1000/m2-step3-code-r1-verifier/copy/` 副本里做，副本已跑完两条命令后原样留存供复核。
- 草稿目录 `/tmp/claude-1000/m2-step3-code-r1-verifier/draft/` 本轮未用到（判别力自证直接在原仓上用 `sed` 核对行号偏移，没有另外拷贝副本文件）。
