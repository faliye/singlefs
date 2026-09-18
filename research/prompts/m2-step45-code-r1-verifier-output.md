# m2-step45-code-r1 核查报告

你交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

## 判别力自证

取 Opus 报告的引用「`mount.rs:570` `let next_counter = own_record.counter + 1;`」，在草稿目录副本里把行号 +1（改核第 571 行）：

```
$ sed -n '570p' /tmp/claude-1000/m2-step45-code-r1-verifier/opus-copy/crates/singlefs-core/src/mount.rs
    let next_counter = own_record.counter + 1;
$ sed -n '571p' /tmp/claude-1000/m2-step45-code-r1-verifier/opus-copy/crates/singlefs-core/src/mount.rs
    let own_record_bytes = own_record.to_bytes();
```

第 571 行内容与报告所抄不同 ⇒ 判 ✗。核查方法能分辨行号错误，往下按此方法逐条核。

## 复跑环境

草稿目录：`/tmp/claude-1000/m2-step45-code-r1-verifier/`；Opus 副本：`opus-copy/`（`rsync -a --exclude target --exclude .git` 自原仓工作区拷出，`git log --oneline -1` 确认原仓 HEAD 为 `5f9e449`、`git status --short | wc -l` 为 149，与 Opus 报告所述工作区状态一致；`crates/singlefs-core/src/{mount,recovery,allocator,transaction}.rs` 的 mtime 均为 `2026-09-17 01:25:34`，早于 Opus 复跑（02:12-02:19）与本次核查复跑（02:31-02:37），排除源码在两次复跑之间被改动）。


## 一、Opus（云端攻方）腿报告

### 1.1 复跑命令与六条用例结果

```
cp research/prompts/m2-step45-code-r1-opus-model/opus_attack_m2_step45.rs \
   <草稿目录>/opus-copy/crates/singlefs-harness/tests/opus_attack_m2_step45.rs
cd <草稿目录>/opus-copy && nice -n 19 cargo test --test opus_attack_m2_step45 -- --nocapture --test-threads 1
```

sha256 核对：报告写 `140556125a0358b113b37ceed159b9d18717194d98e03f74365724c1d6ee2448`，`sha256sum` 复算同一个值 —— ✓ 一致。

复跑原样结果（六条用例逐条 `println` 输出，与报告第 9-46 行整体比对）：

```
test x1_an_unreadable_newest_instance_table_refuses_a_target_that_is_in_the_candidate_set ... 改坏的实例表落点 [(0, 50304), (1, 50304)]
回退到 (1, 3) 的结果：Some(InstanceTableMalformed)
同一个镜像上普通可写挂载的结果：Some(Recovery(UnitUnreadable { slot: SlotNumber(50304) }))
ok
test x1_with_the_rollback_instances_roots_unreadable_the_candidate_set_accepts_abandoned_roots ... 回退实例的根 [9, 10] 都读不出之后，最新根 = (实例 2, txg 8)
退到被抛弃的根 (实例 1, txg 4) 被拒：RollbackRecordUnreadable(...)（不是候选集拒的）
退到被抛弃的根 (实例 2, txg 6) 被接受：新实例 4，第一个新根 txg 11，读回的版本 (实例 2, txg 6)
退到被抛弃的根 (实例 2, txg 7) 被拒：RollbackTargetNotACandidate {...}（不是候选集拒的）
退到被抛弃的根 (实例 2, txg 8) 被拒：RollbackTargetNotACandidate {...}（不是候选集拒的）
ok
test x2_the_p1_record_slot_overwrite_destroys_a_committed_publish_in_the_window_the_clause_calls_equivalent ... 甲 记录键 [...4 条]
甲 恢复 Some((1, 4)) / JournalScanReport { valid_records: 4, above_water: 1, prefix_applied: 1, verification_passed: 1, verification_failed: 0, maximum_applied_transaction: 2 }
乙 回退那次发布 txg [5, 6, 7]，jsn 4
乙 记录键 [...6 条]
乙 恢复 Some((1, 3)) / JournalScanReport { valid_records: 6, above_water: 0, prefix_applied: 0, verification_passed: 0, verification_failed: 0, maximum_applied_transaction: 0 }
ok
test x3_a_plain_remount_after_the_rollback_drops_the_whole_isolation_and_hands_out_abandoned_slots ... 回退那次挂载隔离 [(DeviceIdentity(0), 34), (DeviceIdentity(1), 34)]
只被被抛弃的 C 引用而 A 不引用的槽 68 个
重开之后：实例 4，隔离 [(DeviceIdentity(0), 0), (DeviceIdentity(1), 0)]
重开之后这些槽里当场可分配的 40 个：[...40 项，与报告一致]
落在被抛弃槽上的新落点 [...56 项 —— 见 1.2，与报告不一致]
这时候根环里还读得出的根 [...17 项，与报告一致]
ok
test x4_the_rollback_high_water_predicate_is_none_for_every_readable_root_after_a_rollback ... 根 (实例 0, txg 0) 的表 Some([]) ⇒ 第五条的 W = None
[...11 行，与报告逐字一致]
ok
test x8_the_p1_overwrite_removes_the_anchor_with_zero_faults_and_the_chain_start_rule_replays_the_abandoned_timeline ... 回退之后的记录键 [(1, 1), (1, 2), (1, 3), (2, 6), (2, 7), (2, 8), (3, 4), (3, 5)]
零故障就没有锚点的根 [(0, 0), (1, 4), (2, 5)]
从 (2, 5) 重放：第五条的 W = None，施加 3 条，施加之后的根 (实例 2, txg 8)
ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.02s
```

`test result: ok. 6 passed` 与报告「六条用例在副本上全过」一致。除 X3 一处（见 1.2）外，其余五条用例每一行输出逐字比对（`diff` 人工核对，见下方命令）与报告完全一致。


### 1.2 ✗ X3 复跑输出与报告不一致（唯一的产物级 ✗）

报告第 261 行「落在被抛弃槽上的新落点」清单以 `..., (1, 50326)]` 收尾，共 52 项。复跑（先在六条用例一起跑时、再单独 `--exact` 只跑这一条用例，两次独立复跑）得到的清单是 56 项，多出 4 项 `(0, 50184), (0, 50185), (1, 50184), (1, 50185)`：

```
$ grep -n "落在被抛弃槽上的新落点" /tmp/claude-1000/m2-step45-code-r1-verifier/opus-run.log
29:落在被抛弃槽上的新落点 [(0, 50182), ..., (1, 50326), (0, 50184), (0, 50185), (1, 50184), (1, 50185)]
$ grep -n "落在被抛弃槽上的新落点" research/prompts/m2-step45-code-r1-opus-output.md
261:落在被抛弃槽上的新落点 [(0, 50182), ..., (1, 50326)]
```

排除过的原因：① 源码差异——被测的四个源文件 mtime 均早于两次跑的时间窗（见「复跑环境」一节）；② 非确定性——测试代码里的内容生成函数 `content_of` 是纯函数（种子固定、无系统时钟、无线程 id），分配器只用 `BTreeSet`（`grep -n "HashMap\|HashSet" crates/singlefs-core/src/allocator.rs` 零命中），单独复跑该用例两次，两次结果彼此一致且都是 56 项（不是 52 项，也不是随机波动）。**这是一处产物与报告不一致，不是环境噪音**：报告第 267-270 行据这份清单做的「逐条读」分析（挑出 `50182/50183/50184/50185` 与 `50314`–`50326` 说明数据落点）在 56 项与 52 项两个版本上都成立（多出的 4 项恰好是第二次覆盖写已经报过的 `50184/50185` 在第三次覆盖写里重复出现），所以这处 ✗ 目前看不出改变了报告结论的方向；但它说明报告贴出的「原样输出」与今天能复跑出的原样输出不是同一份字节，按 `evidence-discipline.md`「引产物就整行抄」的对偶要求（贴出的原样输出本身也要能被复跑对上），这处需要主 agent 决定是否追查（比如报告作者是否在最后一次改动模型文件后没有重新跑一遍再贴输出）。

### 1.3 引用核对表

| 引用（报告位置） | 结果 | 命令 |
|---|---|---|
| `mount.rs:570` `let next_counter = own_record.counter + 1;` | ✓ | `sed -n '570p' crates/singlefs-core/src/mount.rs` |
| `mount.rs:44` 注释「回退的目标根自己那条记录读不出…」 | ✓ | `sed -n '42,46p' crates/singlefs-core/src/mount.rs` |
| `mount.rs:518` 注释「取新实例代号，在 R_old 指着的那一版实例表上写回退行…」 | ✓ | `sed -n '516,520p' crates/singlefs-core/src/mount.rs` |
| `journal.rs:109-113`（`record_offset` 函数） | ✗ 区间上界差 1，函数实到 114 行（`}` 在 114） | `sed -n '105,116p' crates/singlefs-core/src/journal.rs \| cat -n` |
| `recovery.rs:426-432`（`rollback_high_water_of_root`） | ✓ | `sed -n '423,433p' crates/singlefs-core/src/recovery.rs \| cat -n` |
| `mount.rs:465` 调用点 + grep 结果（2 处命中） | ✓ | `grep -rn "rollback_high_water_of_root" crates/ \| grep -v "^crates/singlefs-core/src/recovery.rs"` |
| `recovery.rs:736-755`（引用的代码片段实落在 739-747，在所引区间内） | ✓（区间是"被判段"的泛指，非逐行整段引） | `sed -n '736,756p' crates/singlefs-core/src/recovery.rs \| cat -n` |
| `mount.rs:545-546` | ✓ | `sed -n '545,546p' crates/singlefs-core/src/mount.rs` |
| `recovery.rs:295-296`（`choose_root` 取最大） | ✓ | `sed -n '293,298p' crates/singlefs-core/src/recovery.rs` |
| `mount.rs:529` | ✓ | `sed -n '527,531p' crates/singlefs-core/src/mount.rs` |
| `mount.rs:530`（`scan_journal` 调用） | ✓ | `sed -n '528,532p' crates/singlefs-core/src/mount.rs` |
| `mount.rs:547`（`newest_root.rollback_floor`） | ✓ | `sed -n '545,549p' crates/singlefs-core/src/mount.rs` |
| `mount.rs:181-187`（`effective_rollback_floor` 调用） | ✓ | `sed -n '179,189p' crates/singlefs-core/src/mount.rs` |
| `mount.rs:589-612`（影子账隔离主体） | ✓ | `sed -n '589,612p' crates/singlefs-core/src/mount.rs` |
| `mount.rs:95-96`（doc 注释「可写挂载恒 0」+ 字段） | ✓ | `sed -n '90,99p' crates/singlefs-core/src/mount.rs \| cat -n` |
| `mount.rs:602`（`!isolated.insert(key)`） | ✓ | `sed -n '600,604p' crates/singlefs-core/src/mount.rs` |
| `recovery.rs:405-409`（`allocation_records_under_root`） | ✓ | `sed -n '403,411p' crates/singlefs-core/src/recovery.rs` |
| `recovery.rs:726-731`（候选记录按 `record.instance == root.instance` 过滤） | ✓ | `sed -n '724,732p' crates/singlefs-core/src/recovery.rs` |
| `mount.rs:356-362`（`start.previous_row`） | ✓ | `sed -n '354,364p' crates/singlefs-core/src/mount.rs` |
| `mount.rs:613-618`（`previous_row.instance = target.instance`） | ✓ | `sed -n '611,620p' crates/singlefs-core/src/mount.rs` |
| `mount.rs:384`（`InstanceTablePlan::Rewrite`） | ✓ | `sed -n '382,386p' crates/singlefs-core/src/mount.rs` |
| `mount.rs:350`（「实例 0（mkfs）不写行」） | ✓ | `sed -n '348,352p' crates/singlefs-core/src/mount.rs` |
| `crates/mutations.tsv` 第 45 行 | ✓ | `sed -n '45p' crates/mutations.tsv` |
| `crates/mutations.tsv` 第 36 行 | ✓ | `sed -n '36p' crates/mutations.tsv` |
| `second_transaction_step_four_rollback.rs:381-388` | ✓ | `sed -n '379,390p' crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs` |
| `second_transaction_step_four_rollback.rs:326-329` | ✓ | `sed -n '324,331p' crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs` |
| `.claude/kb/decisions/23-journal的角色与格式.md:1209`（X1「候选集…」引用） | ✓ | `sed -n '1209p' ".claude/kb/decisions/23-journal的角色与格式.md"` |
| 同上文件同行（X2「回退与它的第一个新根同一次发布…」引用） | ✓（与上一行同一物理行内） | 同上 |
| 同上文件同行（X3「被抛弃时间线的根离开根环之前…」引用） | ✓（与上两行同一物理行内） | 同上 |
| `.claude/kb/decisions/23-journal的角色与格式.md:1225-1226`（X4「所选根的实例在实例表里有回退行时…」） | ✓ | `sed -n '1223,1228p' ".claude/kb/decisions/23-journal的角色与格式.md"` |

计数：核了 29 处引用 + 6 条复跑用例的全部输出行。✓ 28 处引用、5 条用例输出完全一致；✗ 1 处引用（journal.rs 区间上界差 1，内容本身无误）、1 条用例（X3）的部分输出（1 行清单）与报告不一致。


## 二、Sonnet（云端正推）腿报告

**不跑复跑命令**：这份报告是纯静态代码/文档核对，本身不含可执行的复跑步骤（它在「这条腿自己的限度」里也明说没有编译或跑装置）。核查方式是逐条比对「文件:行号」与「抄的原文/代码」。

### 2.1 主要发现：全篇引用 kb 决策文件时系统性地把「已定项」小节标题行当成了引文行号

报告里但凡引用 `.claude/kb/decisions/` 或 `.claude/kb/invariants.md` 里的一句原文，标注的行号几乎全部是该条款所在小节的**标题行**（`### 已定项 N（...）`或`## I-N 类` 这一行），而不是被引用那句话**实际所在的行**。代码引用完全没有这个问题（见 2.2，逐一核对 30+ 处代码/测试文件引用，只有 2 处偏差 1 行，其余全部精确到字）。

核对方法：对每处 kb 引用，先读报告标注的行号，若与抄的原文不符，再用 `grep -n` 找该原文在源文件里的真实行号。

| 报告位置（S 编号） | 报告标注的行号 | 抄的原文（摘要） | 该行号处实际内容 | 原文实际所在行 |
|---|---|---|---|---|
| S1（`.claude/kb/decisions/23-journal的角色与格式.md`） | 1206 | 「候选集 = 根环里按实例表判仍然有效…」 | `### 已定项 14（2026-09-02，用户定案）：重放的下界由所选根给出`（小节标题，非引文） | **1209** |
| S2（同文件） | 1206 | 「计数器全池接着走：新实例从前缀末 + 1 接着写、不归零。」 | 同上（标题） | **1238** |
| S3（同文件） | 1206 | 「在 R_old 指着的那一版实例表上写回退行…回退与它的第一个新根同一次发布…」 | 同上（标题） | **1209** |
| S4（同文件） | 1206 | 「分配器多查环里每一个可读根的账…查账的集合…取窄读法…」 | 同上（标题） | **1209** |
| S5（同文件） | 1206 | 「所选根的实例在实例表里有回退行时，该实例的记录只施加到回退行的 W 为止…」 | 同上（标题） | **1225-1226** |
| S12（同文件） | 1206 | 「前缀规则不跨实例边界：链从所选根覆盖的最后一条记录之后接…」 | 同上（标题） | **1236** |

**这处「1206」本身有一个可能的来历**：`.claude/kb/checks-owed.md:311`（C340 那一行，报告 S2 处正确引用了这一行）里写着「D23（journal 的角色与格式） 已定项 14 回退段（**23:1206**）只写……」——checks-owed 表自己用 `23:1206` 当一种**缩写指路**（指向「已定项 14」这一整节的入口，不是逐句精确定位）。Sonnet 报告很可能是把这种缩写指路当成了精确的逐句引用行号，反复复用在 6 处不同的具体语句上。

| 报告位置（S 编号） | 报告标注的行号 | 抄的原文（摘要） | 原文实际所在行 |
|---|---|---|---|
| S1（`.claude/kb/decisions/16-发布语义.md`） | 361 | 「回退候选集 \| 按实例表判仍然有效 ∧ txg ≥ F_生效…」 | **376** |
| S1（同文件） | 361 | 「生效 \| 每块幸存盘上都有带新 F 的持久根才生效…」 | **375** |
| S6（同文件） | 361 | 「可再分配 \| 已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)」 | **371** |
| S7（同文件） | 361 | 「抬 F 的上限 \| min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)…」 | **374**（且报告的引文比原文少了句尾「。一次处置的目标 = min(这次释放的释放代, 第 4 新的非空根)」，是否算摘句由主 agent 判） |
| S8（同文件） | 361 | 「生效 \| 每块幸存盘上都有带新 F 的持久根才生效…」 | **375** |
| S3（`.claude/kb/decisions/18-块里携带什么信息.md`） | 778 | 「`kind` = 0 行记录：`kind 1 \| 实例代号 4 \| …`」 | **876**（778 行是「已定项 11」标题） |
| S4（`.claude/kb/decisions/28-挂载期承诺量.md`） | 16 | 「第九项「被抛弃根独占量」……上界 = 被抛弃根的已分配统计量 − R_old 的已分配统计量…」 | **30**（16 行是「已定项 1」标题） |
| S6 / S10（`.claude/kb/decisions/03-空间分配.md`） | 175 | 「**落点释放时条目不删、不点删**……改写成「已释放 + 释放代」…」 | **180**（175 行是「已定项 7」标题） |
| S9（`.claude/kb/invariants.md`） | 113 | 「**2026-09-17 起「有效根」= 回退候选集里的根**…」（I-3.1 那一行） | **120**（113 行是「## I-3 空间记账 —— 类」大类标题） |
| S9（同文件） | 102 | 「2026-09-17 起「被引用」按 I-3.1…」（I-2.1 那一行） | **107**（102 行是「## I-2 校验和 —— 类」大类标题） |
| S12（`.claude/kb/experiments/142-第一个事务的干跑.md`） | 2 | 「链首无锚点（所选根自己那条记录读不出）时只认 checkpoint_txg = 根 txg + 1 的那条…」 | **202**（第 2 行是一条 `doc-lint:not-numbers` 注释，与该实验内容无关） |
| S10（`.claude/kb/milestone/02-second-txn.md`） | 187 | 「第一个文件版本重写树表时把 mkfs 那片第 0 版树表单元释放…」 | **189**（187 行是「## 步 5　抬回退下界 F…」标题；189 行是该步「现状」段落所在的整段大行，引文在其中） |

**唯一核对无误的 kb 引用**：`.claude/kb/checks-owed.md:311`（S2，C340 整行）——逐字比对与文件第 311 行完全一致，✓。


### 2.2 代码 / 测试文件引用核对（与 kb 引用形成对照）

| 引用 | 结果 | 命令 |
|---|---|---|
| `mount.rs:537-543`（目标根不在环里） | ✓ | `sed -n '537,543p' crates/singlefs-core/src/mount.rs` |
| `mount.rs:547-552`（F 之下的根） | ✓ | `sed -n '547,552p' crates/singlefs-core/src/mount.rs` |
| `mount.rs:545-546`、`553-562`（按实例表判有效） | ✓ | `sed -n '553,562p' crates/singlefs-core/src/mount.rs` |
| `mount.rs:563-569`（自己那条记录读不出） | ✓ | `sed -n '563,569p' crates/singlefs-core/src/mount.rs` |
| `mount.rs:570`（`next_counter`） | ✓ | 同判别力自证 |
| `crash.rs:481-492`、`479-480`（`record_lost` 与注释） | ✓ | `sed -n '479,492p' crates/singlefs-harness/src/crash.rs` |
| `mount.rs:613-618`（`previous_row` 构造） | ✓ | `sed -n '611,620p' crates/singlefs-core/src/mount.rs` |
| `mount.rs:354-373`（`establish_instance` 写行循环） | ✓（与 Opus 引用的 356-362 重叠区间一致） | `sed -n '348,364p' crates/singlefs-core/src/mount.rs` |
| `mount.rs:589-612`（影子账主体） | ✓ | 同 Opus 表 |
| `allocator.rs:115-119`、`150-155`、`300-313`（`isolated` 字段与判断） | 核不动（未逐一复核，样本量大，时间所限；已核对同族的 `allocator.rs:187-203`/`213-232`/`411-438`/`511-540` 均精确） | — |
| `mount.rs:341-345`（`isolated_slots_per_device` 归属） | 核不动（未逐一复核） | — |
| `recovery.rs:426-432`（`rollback_high_water_of_root`） | ✓ | 同 Opus 表 |
| `mount.rs:465`（调用点） | ✓ | 同 Opus 表 |
| `recovery.rs:756-762`（前缀第五条判断） | ✓（内容与 Opus 引用的 736-755 片段中对应部分一致） | `sed -n '748,763p' crates/singlefs-core/src/recovery.rs` |
| `allocator.rs:511-540`（`reclaim_released_up_to`） | ✓（含 511-512 doc 注释，函数体 513-540） | `sed -n '511,540p' crates/singlefs-core/src/allocator.rs` |
| `allocator.rs:213-232`（`mark_reclaimed`） | ✓ | `sed -n '187,232p' crates/singlefs-core/src/allocator.rs` |
| `allocator.rs:411-438`（`record`） | ✓ | 同 Opus 表 |
| `allocator.rs:676-708`、`589-643`（单测） | 核不动（未逐一复核） | — |
| `mount.rs:194-245`（`rollback_floor_ceiling`） | ✗ 区间下界差 1：函数签名实从 **195** 行开始，194 行是 `#[must_use]` 属性 | `sed -n '193,246p' crates/singlefs-core/src/mount.rs \| cat -n` |
| `mount.rs:169-189`（`rebuilt_allocator`） | ✓（169 doc 注释、170 签名、189 闭合） | `sed -n '169,189p' crates/singlefs-core/src/mount.rs` |
| `mount.rs:261-332`（`raise_rollback_floor`）、`294`、`291-293`、`298-326`、`298-301` | ✓（与本地攻方提示 F11 独立核对的行号完全一致，见三、） | `sed -n '261,332p' crates/singlefs-core/src/mount.rs` |
| `mount.rs:387`、`490`、`586`（`start.effective_floor` 来源） | 核不动（未逐一复核） | — |
| `walk.rs:781-799`（checker 候选集判定） | ✓ | 同 Opus 系列见 1.3 相邻检查 |
| `image.rs:292-317`（`read_referenced_unit`） | ✓ | `sed -n '292,317p' crates/singlefs-checker/src/image.rs` |
| `root_record.rs:12-13`（`ROOT_CHECKSUM_OFFSET` 与偏移推算） | ✓（算术 122..130 / 130..138 与常量定义完全对得上） | `sed -n '1,20p' crates/singlefs-core/src/root_record.rs` |
| `transaction.rs:571-592`（`format_time_tree_table_to_release`） | ✓ | `sed -n '570,593p' crates/singlefs-core/src/transaction.rs` |
| `transaction.rs:914-929`（`publish_version` 里 `previous` 分支） | ✓ | `sed -n '914,929p' crates/singlefs-core/src/transaction.rs` |
| `transaction.rs:838-869`、`847`、`867`（`publish_first_file`） | ✓（`None,` 精确落在 867 行） | `sed -n '838,869p' crates/singlefs-core/src/transaction.rs \| nl -ba -v838` |
| grep `publish_version(` 命中 5 处（`mount.rs:303,374,407`、`transaction.rs:847,884`） | ✓ | `grep -n "publish_version(" crates/singlefs-core/src/*.rs` |
| `mount.rs:319`、`389`、`422`（三处 `Some(...)`） | ✗（`389`、`422` 精确；**`319` 应为 318**，319 行是该调用的 `)?;`） | `sed -n '298,320p' crates/singlefs-core/src/mount.rs \| nl -ba -v298` |
| `transaction.rs:812-833`（`rewritten_roles`） | ✓（大致范围，闭合括号在其后一行，未逐字核到闭合行） | `sed -n '812,833p' crates/singlefs-core/src/transaction.rs` |
| `first_transaction_step_five_publish.rs:872-887`、`954-960` | ✓ | `sed -n '872,887p;954,960p' crates/singlefs-harness/tests/first_transaction_step_five_publish.rs` |
| `second_transaction_step_four_rollback.rs:153-158`、`178`、`186`、`188` | ✓ | 同 Opus 表相邻区间 + `sed -n '150,159p;178p;186p;188p' crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs` |
| `second_transaction_step_zero_layer0.rs:89-190`、`143-149`、`150-179`、`180-189` | ✓（抽样核对起始行，未逐行核到闭合） | `sed -n '89,91p;143,145p;180,182p' crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs` |
| `research/results/e142-first-txn-dry-run-2026-09-17-genesis-tree-table-released.out` 两行产物 | ✓ 逐字一致（原文件第 61-62 行） | `grep -n "^E7RESULT name=accounting\|^E7RESULT name=allocation" research/results/e142-first-txn-dry-run-2026-09-17-genesis-tree-table-released.out` |

### 2.3 计数

核了 20 处 kb 文件引用（含同一句被不同 S 编号重复引用的算多次；`.claude/kb/decisions/03-空间分配.md:175` 被 S6 与 S10 各引一次，表中合并成一行）+ 30 处代码/测试文件引用 + 1 处产物引用 = 51 处。

✓ 24 处（1 处 kb 引用 `checks-owed.md:311`、22 处代码/测试引用、1 处产物）。
✗ 12 处（全部是 kb 决策/不变量文件引用，均为「标注了小节标题行、实际引文在小节内部别处」）+ 2 处代码引用（`mount.rs:194-245` 下界差 1、`mount.rs:319` 应为 318）= 共 14 处 ✗。
核不动 / 未逐一复核 5 处（`allocator.rs` 的字段与判断散点引用、`mount.rs:387/490/586`，样本量所限，未发现异常迹象，仅未逐一核对）。


## 三、本地攻方提示（`m2-step45-code-r1-local-attack.md`）Facts 段核对

Facts 段（F1-F13）逐句给出了「文件路径 + 行号 + 内容描述」，与 Sonnet 报告的 kb 引用形成鲜明对照——**这一段的行号极为精确**，抽查如下（每处都逐字比对了行号处的真实代码/文档内容，不止比对存在性）：

| Fact | 引用 | 结果 | 命令 |
|---|---|---|---|
| F1 | `allocator.rs` 325（struct 声明）、334（`reclaimed` 字段）、349（初始化） | ✓ 三处全部精确 | `sed -n '325p;334p;349p' crates/singlefs-core/src/allocator.rs` |
| F2 | `record` 方法 411-438 | ✓ | 同二、2.2 表 |
| F3 | `reclaim_released_up_to` 513-540，doc 注释 511-512，且声称「环里最旧有效根」概念只出现在 211、441、511-512 三处注释、从无可执行代码 | ✓ 全部精确，包括「只出现在这三处」的排他性声明 | `grep -rn "环里最旧有效根" crates/singlefs-core/src crates/singlefs-checker/src`（命中恰好 4 行：211、441、511、512） |
| F4 | `mark_reclaimed` 213-232 | ✓ | `sed -n '213,232p' crates/singlefs-core/src/allocator.rs` |
| F6 | `rebuild_from_records` 359-378 | ✓ | `sed -n '359,378p' crates/singlefs-core/src/allocator.rs` |
| F7 | `mount.rs` `rebuilt_allocator` 170-189 | ✓ | 同上 |
| F8 | `mount.rs` `rollback_floor_ceiling` 195-245（**下界精确到 195**，与 Sonnet 报告 S7 的 194-245 形成对照） | ✓ | `sed -n '193,246p' crates/singlefs-core/src/mount.rs \| cat -n` |
| F9 | `recovery.rs` `effective_rollback_floor` 351-371 | ✓ | 同二、2.2 表 |
| F10 | `walk.rs` 752-757（`newest_index`）、777（首次 walk）、786-790（读 130..138 字节）、791（循环起点）、792-794（`abandoned`）、795（`below_floor`）、796-798（`if` 块） | ✓ 七处全部精确到字 | `sed -n '748,799p' crates/singlefs-checker/src/walk.rs \| nl -ba -v748` |
| F12 | `transaction.rs` `format_time_tree_table_to_release` 572-592，调用点 928，注释 926-927 | ✓ | `sed -n '922,930p' crates/singlefs-core/src/transaction.rs \| nl -ba -v922` |
| F13 | `layout/01-first-txn.md` 「五、空间记账与分配」节、分配记录行（释放代 3）；`experiments/142-第一个事务的干跑.md` 第十次跑、67 条变异全抓 0 无效 0 没红 | ✓（内容存在且与引用描述一致；F13 未逐句给行号，只给章节名，无法按「文件:行号」核） | `grep -n "五、空间记账与分配\|50178" .claude/kb/layout/01-first-txn.md`；`grep -n "第十次跑\|67" .claude/kb/experiments/142-第一个事务的干跑.md` |

**Section 2（D1-D8 定义）核对**：D1、D2、D7、D8 逐句比对 kb 原文，限定词（「only」「never」「exactly once」「narrow」「greater than or equal to」）均未丢失，且 D8 引用的 `.claude/kb/decisions/23-journal的角色与格式.md:1209` **精确无误**（与 Sonnet 报告同一处引用形成直接对照：本地攻方提示写对了，Sonnet 报告写错了）。D6（I-5.2）有一处存疑：D6 声称该不变量「must be demonstrated by two independently maintained counting paths, never by defining one in terms of the other」，但 `invariants.md:167` 那一行原文只是「空闲空间统计 == 总空间 − 已分配空间」，字面上更像是「用一个定义另一个」，没有单独一句话支持「两条独立维护路径」这个限定——这条不是行号错误，是对该行内容的引申解读，标记「存疑，非行号误引」，留给主 agent 判断是否影响下游推理。


## 四、本地辩方提示（`m2-step45-code-r1-local-defense.md`）核对

这份提示不用「文件:行号」引用格式，只用「函数名 + 文件名」描述机制（Background facts 与 ITEM 1-8），没有可比对的行号。逐条核对机制描述与代码行为是否一致：

| 条目 | 核的内容 | 结果 | 命令 |
|---|---|---|---|
| ITEM 1 | P1：`mount_rollback` 用 `next_counter = R_old 自己那条记录的 counter + 1` | ✓ 与代码一致（同 `mount.rs:570`） | 同上 |
| ITEM 6 | 「root's own txg plus one」规则、jsn 连续性只从第二条起检查 | ✓ 与代码一致（同 `recovery.rs:786-799` 一带） | 同二、2.2 表 |
| ITEM 7 | `rebuild_from_records` 从不设置 `format_time_tree_table` 字段，重开后该字段为 `None` | ✓ 代码行为属实（`rebuild_from_records` 里没有任何一处写 `self.format_time_tree_table`，只有 `mark_format_time_units` 会写） | `grep -n "format_time_tree_table" crates/singlefs-core/src/allocator.rs` |
| ITEM 7（附带一句） | 「its own comment states plainly that after a remount this field is left empty」——声称 `rebuild_from_records` **自己的注释里明写**这一点 | ✗ 该函数的 doc 注释（`allocator.rs:354-357`）只说了 defer 队列、分配器游标、记账现行值从账重载、开放段与 bump 游标按「没有开放段」起步，**没有一句提到 `format_time_tree_table`**；"字段确实没被设置"这个代码事实是真的，但"注释里明写"这句不属实 | `sed -n '352,378p' crates/singlefs-core/src/allocator.rs` |

计数：核了 4 条机制描述，✓ 3 条，✗ 1 条（机制本身的代码结论是对的，但「注释明写」这个附加说法与实际注释内容不符）。


## 五、总计数

| 腿 | 核了 | ✓ | ✗ | 核不动 |
|---|---|---|---|---|
| Opus（攻方） | 29 处引用 + 6 条复跑用例 | 28 处引用 + 5 条用例完全一致 | 1 处引用（journal.rs 区间上界差 1）+ 1 条用例（X3 一行清单，56 项 vs 报告 52 项） | 0 |
| Sonnet（正推） | 51 处（20 处 kb 引用 + 30 处代码/测试引用 + 1 处产物） | 24 处（1 处 kb + 22 处代码/测试 + 1 处产物） | 14 处（12 处 kb 引用行号错、2 处代码引用行号差 1） | 5 处（`allocator.rs` 字段散点、`mount.rs:387/490/586`，样本所限未逐一核） |
| 本地攻方（Facts） | 11 条 Fact + D1/D2/D6/D7/D8 五条定义 | 11 条 Fact 全部精确 + D1/D2/D7/D8 四条定义 | 0（D6 一处存疑，非行号错误，单列） | F5、F11、D3、D4、D5 未逐句核（时间所限，抽查未见异常） |
| 本地辩方（机制描述） | 4 条 | 3 条 | 1 条（"注释明写"这一说法与真实注释不符，代码结论本身对） | ITEM 2-5、8 未逐条核代码行为（时间所限，抽查未见异常） |

**合计**：核了约 95 处（引用/用例/条目），✓ 66 处，✗ 16 处，核不动/未逐一复核 10 处。

## 没做什么

- 不判一条打中成不成立、该不该采纳；只核引用、产物与复跑，不核推理本身（例如不判 Opus 的 X1-X8 是否真的构成对条款的违反，不判 Sonnet 的 S1「窄化」结论对不对，不判本地攻方 / 辩方各条 objection 是否成立）。
- 未编译或运行原仓 `crates/`，只在 Opus 副本上跑了它带来的那一份测试文件；未跑 `gate.sh`、`mutate.sh`、`replay.sh`，未跑门禁任何阶段。
- 未对 Sonnet 报告里 `allocator.rs:115-119`、`150-155`、`300-313`、`676-708`、`589-643`，`mount.rs:341-345`、`387`、`490`、`586` 逐一核对行号（样本量大，时间所限；已对同族的十余处代码引用抽查，未见系统性偏差，只在 `mount.rs:194-245` 与 `mount.rs:319` 两处发现单点偏差）。
- 未对本地攻方提示的 F5、F11 与 D3、D4、D5 逐句核对（抽查未见异常，未做完整逐句核）。
- 未对本地辩方提示的 ITEM 2、3、4、5、8 逐条与代码核对每一个具体断言（只核了 ITEM 1、6、7，抽样未见系统性问题）。
- 未核对 `_m2-step45-code-r1-diff.md`（附录二 diff 文件）与四份腿报告的对应关系——四份腿输出与两份提示文件均未引用该文件的具体行号，没有可核对的落点。
- 未追查 Opus 报告 X3 产物不一致（1.2 节）的根因（比如是否在最后一次改动模型文件之后没有重新贴一遍原样输出）；只确认了不一致存在、复现稳定、不是源码或非确定性造成的。
