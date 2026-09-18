# m2-step45-code-r3 核对表（三方核查员）

我交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

依据：`.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」；
`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「校验路径本身也要证明它会红」「引产物就整行抄」。

四条腿全部交齐（opus-output.md 239 行 + opus-model/x8_hold_and_isolate_bits.rs 218 行；
sonnet-output.md 228 行；local-attack-output-s1/s2.md + local-attack-translation-audit.md；
local-defense-output-s1/s2.md + local-defense-translation-audit.md），核对表写在这份文件，
草稿目录 `/tmp/claude-1000/m2-step45-code-r3-verifier/`。

## 判别力自证（必须判 ✗，判不出就停）

待核引用：Opus 报告 `m2-step45-code-r3-opus-output.md` 第 75 行称
「`.claude/kb/decisions/16-发布语义.md` 第 377 行」逐字是「准入」那一行
（可分配 = min(可再分配 + 活元数据 − 保留池, `df`)……）。核实：现查该文件第 377 行，
内容确为「准入」那一行，引用属实（✓，这是判别力自证之外顺带完成的一次真实核对）。

**判别力自证**：把行号故意加 1，核第 378 行（副本
`/tmp/claude-1000/m2-step45-code-r3-verifier/selftest/16-发布语义-copy.md`）：

```
$ awk 'NR==378' 16-发布语义-copy.md
| `df` | 没用过的 + 已释放的 + 活元数据 − 保留池 − 推空最坏残留 − 滞后量；……
```

第 378 行讲的是 `df` 的计算式，与待核引用文本（准入判据）不一致 ⇒ **判 ✗**。
方法能分辨真假引用，往下的核对按同一方法（取该行/该区间原文，逐字比对）进行。

## 一、云端攻方（Opus）

复跑（在我自己的草稿副本，不在 opus 原目录）：

```
$ mkdir -p /tmp/claude-1000/m2-step45-code-r3-verifier/opus-replay
$ nice -n 19 rsync -a --exclude target --exclude .git \
    /home/fy5090/code/singlefs/ /tmp/claude-1000/m2-step45-code-r3-verifier/opus-replay/copy/
$ cp .../opus-model/x8_hold_and_isolate_bits.rs \
    .../opus-replay/copy/crates/singlefs-core/tests/
$ sha256sum .../x8_hold_and_isolate_bits.rs
8a155f8eb45dfe649ae85791637c137318dfca359771d29f76515425349a6c7a  (与报告第 19 行一致)
$ wc -l x8_hold_and_isolate_bits.rs   → 218（与报告第 19 行一致）
$ cd copy && nice -n 19 cargo test -p singlefs-core --test x8_hold_and_isolate_bits
running 4 tests
test a_two_slot_record_of_an_abandoned_root_is_skipped_whole_when_its_start_slot_is_exempt_so_the_second_slot_stays_free ... ok
test the_bump_path_hands_out_a_slot_that_is_free_map_says_is_not_free_when_it_is_isolated_after_the_segment_was_opened ... ok
test once_the_holds_are_left_set_every_later_allocation_keeps_failing_until_release_reclaim_holds_is_called ... ok
test holding_the_reclaimed_segment_makes_the_next_commit_generated_allocation_fail_while_the_free_count_says_there_is_room ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

复跑输出与报告第 21-31 行原样输出逐字一致；sha256、行数均一致（判定：✓）。

引用核对表（取原文那一行/区间，逐字比对；行号均现查 `crates/`、`.claude/kb/` 现状，不从背景材料数）：

| 引用（文件:行，报告行号） | 报告怎么抄 | 核的结果 |
|---|---|---|
| mount.rs:501（报 37） | `release_reclaim_holds()` 调用点 | ✓ 该行确为 `allocator.release_reclaim_holds();` |
| mount.rs:465-468（报 48-55） | `reclaim_released_up_to(...)` 代码块 | ✓ 逐字一致 |
| mount.rs:493 + 488-501（报 57-67） | `)?;` 在 501 之前，代码块 | ✓ 493 行确为 `)?;`；488-501 代码块逐字一致 |
| allocator.rs:242-261（报 57） | `mark_reclaimed` 整个函数 | ✓ 逐字一致 |
| allocator.rs:221-232（报 57） | `hold_until_floor_takes_effect` | ✓ 逐字一致 |
| allocator.rs:607-611（报 57） | `release_reclaim_holds` | ✓ 逐字一致 |
| transaction.rs:955-958、954、942-952、1004（报 71-73） | `allocator_before_this_publish`、450 KiB 拷贝注释、`AllocationRecordsExceedOneNode`、`NoSpaceFor` | ✓ 四处逐字一致 |
| D16:377（报 75-78） | 「准入」那一行 | ✓（即判别力自证的目标行，已核） |
| mount.rs:385-397（报 81） | `raise_rollback_floor` 函数级文档、测试入口自陈 | ✓ 逐字一致（含「只供测试的强制入口」一句） |
| allocator.rs:309-325、160-167（报 123） | `lowest_user_data_slot`、`is_free` | ✓ 逐字一致 |
| allocator.rs:329-344（报 124） | `lowest_empty_segment` | ✓ 逐字一致 |
| allocator.rs:534-561、463-490、285-305（报 125） | `allocate_commit_generated`、`record`、`mark_allocated` | ✓ 三段逐字一致，含 `mark_allocated` 断言消息「跨度里有已分配的槽」 |
| mount.rs:449-464（报 131） | 抬 F 时重算影子账调用块 | ✓ 逐字一致 |
| allocator.rs:379-390（报 131） | `PoolAllocator::new` | ✓ 逐字一致 |
| mount.rs:178-183、300-304（报 135） | `reclaim_floor`、`oldest_valid_root` | ✓ 逐字一致 |
| mount.rs:221-225、231-237、246-255（报 147-160） | 豁免集与隔离循环三段代码块 | ✓ 三段逐字一致 |
| allocator.rs:200-212（报 147-160） | `DeviceFreeMap::isolate` | ✓ 逐字一致 |
| D23:1209（报 164-169） | 主句「不许重新分配也不许抹头……只隔离其中」 | ✓ 属实但为**显式声明的部分引用**（报告自陈「那一行整行是回退那一整段……下面抄的是其中加粗的主句开头」），截断处恰好在「只被被抛弃根引用的」限定词之前；报告正文其余处已充分讨论这个限定词，不构成摘句掩盖 |

| D28:30（报 207） | 「上界 = 被抛弃根的已分配统计量 − R_old 的已分配统计量」 | ✓ 逐字一致 |
| singlefs-format/lib.rs:185、192（报 213） | `ROOT_RING_REGIONS = 3`、`ROOT_RING_REGION_DEVICES = [0, 1, 0]` | ✓ 逐字一致 |
| root_ring.rs:32-38（报 213）| `target_for_publish` | ✓ 逐字一致 |
| mount.rs:472-476（报 213） | 抬 F 空发布 while 循环 | ✓ 逐字一致 |
| mount.rs:292-299、657-658、732-733（报 218） | `newest_table`、两处 `InstanceTableRecords::parse` 报错点 | ✓ 三段逐字一致 |
| recovery.rs:331-347、287-303、1228-1253（报 199/205/217） | `readable_roots`、`choose_root`、`recover` | ✓ 三段逐字一致 |
| walk.rs:786-799、831-843（报 184） | I-3.1 并集口径、判定代码 | ✓ 逐字一致 |
| recovery.rs:351-372（报 182） | `effective_rollback_floor` | ✓ 逐字一致 |
| mount.rs:278-279（报 182） | `rebuild_from_records` 调用点 | ✓ 逐字一致 |
| allocator.rs:572-577、372、580（报 182/186） | 回收候选过滤、`reclaimed` 字段、去重判断 | ✓ 三处逐字一致 |
| allocator.rs:614-621（`isolate_abandoned`，报未点行号，正文提及函数名） | 隔离调用是否传整跨度 | ✓ 现查确认调用 `device_map.isolate(slot, span)`，支持报告 X1 一节的机制描述 |

**Opus 小计**：核了 48 处（47 处文件:行引用 + 1 处模型文件三件套核对：行数/sha256/cargo test 输出），
✓ 48 处（其中 1 处为显式声明的部分引用，内容仍属实，计入 ✓），✗ 0 处，部分 0 处，核不动 0 处。

## 二、云端正推（Sonnet）

复跑（草稿副本 `/tmp/claude-1000/m2-step45-code-r3-verifier/sonnet-replay/copy/`，
脚本 `run_mutations_verify.py` 与 sonnet 自己 `/tmp/claude-1000/m2-step45-code-r3-sonnet/run_mutations.py`
逐字节比对，只有两处绝对路径不同，已贴 diff）：

```
$ diff run_mutations.py run_mutations_verify.py
3c3
<  root = ".../m2-step45-code-r3-sonnet/copy"
---
>  root = ".../m2-step45-code-r3-verifier/sonnet-replay/copy"
27c27
<  CARGO_TARGET_DIR = ".../m2-step45-code-r3-sonnet/target-mutate"
---
>  CARGO_TARGET_DIR = ".../m2-step45-code-r3-verifier/target-mutate"

$ nice -n 19 python3 run_mutations_verify.py
loaded 6 rows (lines >= 51)
=== RESULTS ===
51  步 4：被抛弃根的树表读不出时不计数              RED (as expected)
52  步 5：抬 F 之后不按新候选集重算影子账            RED (as expected)
53  步 5：影子账的豁免不看候选根的 txg 是否 ≥ F      RED (as expected)
54  步 5：抬 F 回收的槽不扣住、生效之前就能发出去    RED (as expected)
55  步 5：抬 F 生效之后不放开扣住的槽                RED (as expected)
56  步 5：没做过可写挂载的进程抬 F 时 panic 而不是报错 RED (as expected)
FAILURES: 0
```

与报告第 184-201 行原样输出逐字一致（判定：✓）。

```
$ cargo test --offline -p singlefs-harness --test second_transaction_step_four_rollback
test result: ok. 6 passed; 0 failed...
$ cargo test --offline -p singlefs-harness --test second_transaction_step_five_reuse
test result: ok. 7 passed; 0 failed...
```

与报告第 203-208 行一致（6 个 + 7 个 = 13 个用例，全绿，判定：✓）。

引用核对表：

| 引用（文件:行） | 报告怎么抄 | 核的结果 |
|---|---|---|
| mount.rs:201-208（报 20-26） | `isolate_slots_referenced_only_by_abandoned_roots` 文档整段 | ✓ 但**只抄了 201-204**；报告随后在 T2 节分别引 205（报 69）、207（报 69）、207-208（报 71），三处也都核实逐字一致——不是漏抄，是把同一个 8 行文档拆成两处引用，无内容缺失 |
| mount.rs:262（报 30） | 「影子账只住内存……」 | ✓ 逐字一致 |
| mount.rs:433-436（报 12/94/247-248 提及） | 「回收在写第一条带新 F 的根之前……」 | ✓ 逐字一致 |
| **D23:1206-1244 → 「有效只按实例表判」（报 29）** | 「用户窄读法措辞（附录 D23:1206-1244 段）只说『有效只按实例表判』」 | **✗ 未找到**：`grep -n "有效只按实例表判" .claude/kb/decisions/23-journal的角色与格式.md` 命中 0 次，附录里逐字抄录的 D23:1206-1244 原文同样不含这句。该短语实际出现在 `crates/singlefs-core/src/mount.rs:202`（代码注释转述用户定案）与 `.claude/kb/milestone/02-second-txn.md:160`（现状段），以及背景材料 `_m2-step45-code-r3-background.md:34`。**误写成 D23 的内容，原文实为 mount.rs:202 / 02-second-txn.md:160**（背景材料 34 行是同一句话的又一处出现，不是 D23 摘录）。这条支撑了 T1a「差别只有一条」的关键论据，值得主 agent 复核该结论是否仍站得住 |
| D16:358-380（报 76） | 已定项 1 表格「生效」行等 | ✓ 逐字一致（375「生效」、376「回退候选集」、377「准入」三行均核对） |
| mount.rs:397-507（报 78） | `raise_rollback_floor` 整个函数范围 | ✓ 397 行确为函数签名首行，507 行确为闭合 `}` |
| mount.rs:472-476（报 82-88） | while 循环条件 | ✓ 逐字一致 |
| allocator.rs:122-126（报 12） | `held_until_floor_takes_effect` 字段文档 | ✓ 逐字一致 |
| allocator.rs:220-232（报 12） | 同一字段的置位函数 | ✓ 与 Opus 一节核过的 221-232 一致（sonnet 多算了空行到 220） |
| allocator.rs:397-430（报 164） | `rebuild_from_records` | ✓ 逐字一致，含「m1 实例表跨度 2」相关的第 0 版树表单元识别逻辑 |
| mount.rs:407-411（报 164） | `RaiseNeedsWritableMountInThisProcess` | ✓ 逐字一致 |
| invariants.md:120（报 165） | I-3.1 状态列整行 | ✓ 逐字一致 |
| checks-owed.md:311（报 166） | C340 第三列 | ✓ 该行整行与报告转述一致。**旁注（不算 sonnet 的错）**：C340 第三列自己写的子引用「23:1206」「23:1233」在 D23 文件里均偏差 3 行（实际是 1209、1236）；sonnet 的判定（两句短语确实逐字见于 D23 全文）依然成立，只是 checks-owed.md 自身的行号引用有既存漂移，供主 agent 参考是否要另立一条 kb 修订 |
| 02-second-txn.md:160、189（报 163-164） | 步骤 4/5 现状段 | ✓ 两段均逐字比对一致（超长段落逐字核完） |
| mount.rs:702-705、709-716、769-774（报 167） | `mount_rollback` 文档、签名、`next_counter` | ✓ 三段逐字一致 |
| second_transaction_step_four_rollback.rs:2（报 168-176） | 模块级文档「jsn 接在 A 那条记录之后 = 4」（P1 措辞，未跟着改） | ✓ 逐字一致，确认这一行仍是 P1 措辞；同文件第 159 行函数级注释确认已经是 P2/jsn9 措辞（「jsn 接在环里最大的 8 之后」），与报告说法一致 |
| mount.rs:562（报 122，"附近"） | `establish_instance` 写新根 F 字段那一行 | ✓ 精确定位为第 562 行 `rollback_floor: start.effective_floor,`，报告自己标注"附近"，不算误差 |
| mount.rs:280-285（报 147） | `rebuilt_allocator` 内 `effective_floor` 现算值 | ✓ 逐字一致 |

**Sonnet 小计**：核了 29 处（27 处文件:行引用 + 1 处变异复跑 + 1 处基线用例复跑），
✓ 27 处，✗ 1 处（D23:1206-1244 误引），部分 0 处，核不动 0 处。
"方向"一节里 (a) naive/深化版补丁的具体违例数字（如 1474560、1146880 字节）**未复跑**——
派发未点名要求复这一段，且它按报告自陈本就"副本上的数不进 kb"，留给主 agent 决定是否要另外核实。

## 三、本地攻方（转述核对表 + 样本）

`m2-step45-code-r3-local-attack-translation-audit.md` 10 行核对，逐行取「原文文件:行」现查、
比对「定稿」文本是否确实出现在 `m2-step45-code-r3-local-attack.md`（实际发给本地模型的提示）里：

| # | 原文文件:行 | 核的结果 |
|---|---|---|
| 1 | D23:1209「不许重新分配也不许抹头」 | ✓ 提示第 17 行确有 "must not be reallocated and must not have their headers erased"，两个并列动作都在 |
| 2 | mount.rs:202 + D16:376「候选=可读∧有效∧txg≥F」 | ✓ 提示第 13 行三个合取项都在，"it is readable" 打头 |
| 3 | D28:30「由回退路径维护」 | ✓ 提示第 30 行末句 "maintained by the rollback code path" 在 |
| 4 | D28:30「按设备算」 | ✓ 提示第 30 行 "computed separately per device" 在 |
| 5 | D28:30「不遍历树」 | ✓ 提示第 30 行 "without walking the whole tree and without adding any new on disk field" 两条限定都在 |
| 6 | checks-owed.md:328（C356）「两处对同一个量给了两个口径」 | ✓ 提示第 32 行 "explicitly flagged...as two different specifications" 在 |
| 7 | checks-owed.md:296（C318）「按统计量之差算，不遍历树」 | ✓ 提示第 32 行 "adds in parentheses that this is computed the same way...without walking the tree" 在 |
| 8 | recovery.rs:**406**（报「407-409」）「不把内部节点的指针当分配记录解」 | **✗ 行号偏差**：`grep -n` 确认该注释在第 406 行，407-409 是紧随其后判断 `level != 0` 的代码本身，不是这句注释。定稿内容本身确实在提示第 48 行（"does not attempt to interpret that node's contents as allocation records"），只是核对表登记的源文件行号错了 |
| 9 | recovery.rs:405「里程碑步 6 的欠账」 | ✓ 逐字一致，提示第 50 行 "tracked as unfinished work for a later milestone step" 在 |
| 10 | mount.rs:433,447「回收在生效之前」 | ✓ 提示第 42 行 "Before the corresponding reclaim of freed space happens, G5 recomputes..." 把时序放在句首，与核对表描述一致 |

运行记录复核（`m2-step45-code-r3-local-attack-runlog.md`）：

```
$ wc -w m2-step45-code-r3-local-attack-output-s1.md   → 1310（与记录一致）
$ wc -w m2-step45-code-r3-local-attack-output-s2.md   → 1022（与记录一致）
$ nice -n 19 python3 research/scripts/corruption-check.py <s1>   → 绿，exit=0
$ nice -n 19 python3 research/scripts/oov-check.py <s1> <prompt> → 生词=0 拼接=0，exit=0（与记录一致）
$ nice -n 19 python3 research/scripts/oov-check.py <s2> <prompt> → 生词=0 拼接=0，exit=0（与记录一致）
```

判定：✓（记录里的调用方式明确带了提示文件名作为第二参数，即 oov-check 用提示文件建生词豁免表，
复跑时同样带上提示文件才复现「生词=0」；不带则会得到非零生词数，见下节说明，不算记录出错）。

**Attack 小计**：核了 16 处（10 处转述表 + 2 处 wc -w + 2 处 corruption-check + 2 处 oov-check 复跑），
✓ 15 处，✗ 1 处（recovery.rs 行号偏差），部分 0 处，核不动 0 处。

## 四、本地辩方（转述核对表 + 样本）

`m2-step45-code-r3-local-defense-translation-audit.md` 17 行核对，逐行现查：

| # | 原文文件:行 | 核的结果 |
|---|---|---|
| 1 | D23:1209 实例表判据 | ✓ |
| 2 | D16:375「生效」行 | ✓ |
| 3 | D16:376 回退候选集 | ✓ |
| 4 | D16:371 可再分配谓词 | ✓ |
| 5 | D23:1209 主句逐字（含「只被…引用」限定词） | ✓ |
| 6 | D23:1209 窄读法定案逐字，首稿删除误加的中文括注 | ✓ 现查提示文件 `m2-step45-code-r3-local-defense.md` 全文只在 4 处出现中文（第 61/81/133/195 行，均为 kb 文件名本身，符合「保留 kb 中文文件名」的例外），无游离中文括注残留 |
| 7 | mount.rs:213-258（G5 机制，自陈从「213-253」现查改成「213-258」） | ✓ 现查函数体确实止于第 258 行 `}`（`grep -n "^fn isolate_slots_referenced_only_by_abandoned_roots\|^}"` 逻辑复核一致） |
| 8 | mount.rs:202-205（G5 对窄读法措辞的收严说明） | ✓ 逐字一致；顺带确认「有效只按实例表判」这句的**真实出处正是这里**（mount.rs:202），与第二节 Sonnet 报告里的误引形成对照 |
| 9 | D28:30 第九项定义 | ✓ |
| 10 | 背景材料:34（T1 攻方问题整句） | ✓ 与实际背景材料第 34 行逐字一致 |
| 11 | mount.rs:241-245（跳过逻辑，自陈从「238-244」现查改成「241-245」）+ mount.rs:99-101（字段文档，自陈从「215-217」改成「99-101」） | ✓ 两处都现查确认：241-245 恰好是 `for root...else { unreadable += 1; continue; };` 五行；99-101 恰好是 `abandoned_roots_unreadable` 字段的两行文档 + 字段声明行 |
| 12 | 背景材料:26（今天没有消费者） | ✓ |
| 13 | 背景材料:35（T2 攻方问题整句） | ✓ |
| 14 | mount.rs:178（签名）、175-176（doc）reclaim_floor | ✓ |
| 15 | mount.rs:306、466（两处调用点） | ✓ 逐字一致，参数名分别为 `effective_floor`、`new_floor` |
| 16 | mount.rs:433-436（记账先动、口径归 alloc-basis） | ✓ |
| 17 | 背景材料:36（T3 攻方问题整句） | ✓ |

运行记录复核（`m2-step45-code-r3-local-defense-runlog.md`）：

```
$ wc -w <defense-s1> → 1084（与记录一致）; wc -w <defense-s2> → 1060（与记录一致）
$ corruption-check.py 两份 → 绿，exit=0
$ oov-check.py <s1>（不带提示文件） → 生词=10：exemption generalizes exemptions generalization
  strongest account's effective's unallocatable misrepresents counters'（与记录逐词一致）
$ oov-check.py <s2>（不带提示文件） → 生词=8：exemption redefining unaddressable（与记录一致，记录只列了 3 个样例词但计数一致）
```

**重要说明**：本地辩方记录未写出「调用方式」这一行命令；`ask-local.sh` 把提示文件路径原样传给
`oov-check.py` 当第二参数，若当轮以 `cat prompt.md | ask-local.sh`（走 stdin）方式调用，
`PROMPT_PATH` 为空串，`oov-check.py` 收到的第二参数是空字符串（Python 里空串为假，等同不传），
生词豁免表退化为空——这正好复现「不带提示文件」得到的 10 / 8 这两个数字。
本地攻方记录明写了调用命令带文件名（走豁免表，生词=0）；两条腿用的调用方式不同，
但**各自的数字都能在对应的调用方式下精确复现**，两条记录本身不矛盾（判定：✓，附加说明）。

**Defense 小计**：核了 21 处（17 处转述表 + 2 处 wc -w + 2 处 oov-check 复跑），✓ 21 处，✗ 0 处。

## 五、样本里模型自己给的行号（未经核实，仅作提示）

两份腿的 runlog 都已自陈「模型自己给的代码位置引用……未经核实」。抽 2 处现查，确认这类引用不可信：

| 样本 | 模型自称 | 现查结果 |
|---|---|---|
| local-defense-output-s1.md ITEM 3 DEFENSE | "mount.rs line 182" 是 `is_free` 检查 | `is_free` 定义在 `allocator.rs`（不在 `mount.rs`），且在 `allocator.rs` 里也不在第 182 行（实为第 161 行起） |
| local-defense-output-s1.md ITEM 3 DEFENSE | "release_reclaim_holds…in release_reclaim_holds (mount.rs line 485)" | `mount.rs:485` 实际内容是 `back_chain: back_chain_of(&current.record_bytes),`，与 `release_reclaim_holds` 无关；该调用实际在 `mount.rs:501` |

两处均确认不准确，与两条腿 runlog 里「模型自己给出的代码位置引用……来源不明，未经核实」的自我提示一致；
不计入 ✓/✗ 统计，单列。

## 六、总计

| 腿 | 核了多少处 | ✓ | ✗ | 部分 | 核不动 |
|---|---|---|---|---|---|
| 判别力自证 | 1 | — | 1（按要求必须判 ✗，达成） | 0 | 0 |
| 云端攻方 Opus | 48 | 48 | 0 | 0 | 0 |
| 云端正推 Sonnet | 29 | 27 | 1 | 0 | 0 |
| 本地攻方 | 16 | 15 | 1 | 0 | 0 |
| 本地辩方 | 21 | 21 | 0 | 0 | 0 |
| 模型自给行号（单列，不计入判定） | 2 | 0 | 2（预期内，不算腿的错） | 0 | 0 |
| **合计（计入统计的四腿部分）** | **114** | **111** | **2** | **0** | **0** |

两处 ✗：

1. **Sonnet 报告 T1a**（`m2-step45-code-r3-sonnet-output.md` 第 29 行）：把「有效只按实例表判」
   误引为 D23（`.claude/kb/decisions/23-journal的角色与格式.md:1206-1244`）的内容，
   该文件全文零命中；真实出处是 `crates/singlefs-core/src/mount.rs:202`、
   `.claude/kb/milestone/02-second-txn.md:160`、`research/prompts/_m2-step45-code-r3-background.md:34`。
   这条支撑了 sonnet「T1 差别只有一条（txg ≥ F）」的判定依据，值得主 agent 复核。
2. **本地攻方转述核对表第 8 行**：把 `recovery.rs` 里一句注释的行号写成「407-409」，
   现查实为第 406 行（407-409 是紧随其后的代码，不是这句注释）。定稿内容本身确实
   完整出现在发给本地模型的提示里，不影响提示本身的准确性，只是核对表自身的源行号标错。

## 没做什么

- 不判一条打中成不成立、该不该采纳；不核推理本身（比如 X1/X2/X3/X8 的可达性论证是否站得住、
  T1/T2/T3 判「一致」是否正确、"方向"一节哪个改法该选），这些交主 agent。
- 未复跑 Sonnet 报告"方向"一节 (a) naive/深化版补丁在副本上跑出的具体违例字节数
  （1474560 / 1081344 / 1146880 等）——派发未点名要求复这一段，报告自己也说明这些数
  "副本上的数不进 kb"。
- 未复跑 Opus 报告里"没打中的形状"表格中各条推理链的可达性论证（如 `ROOT_RING_REGIONS`
  上限那一格），只核对了其中引用的格式常量数值本身。
- 未对本地攻方/辩方样本里除已抽查的 2 处之外的全部"模型自己给的具体行号"逐一核实——
  按任务要求，这类行号本就未经核实，只抽样确认了两条腿 runlog 的自我提示准确。
- 未跑 `check.sh`、门禁 54/59 号、命名纪律、`doc-lint.sh`——不在这一轮核查员的职责范围，
  四条腿自己的报告里也都注明没有跑这些。
- 未修改 `/home/fy5090/code/singlefs` 下任何已有文件，未做任何 git 写操作；
  全部复跑与探针都在 `/tmp/claude-1000/m2-step45-code-r3-verifier/` 草稿目录完成。

## 历史版本

（暂无历史）
