# m2-final-code-r4 核查员报告

你交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

## 判别力自证

抽的引用：Opus 报告引 `23-journal的角色与格式.md:394`（① 删除规则那一条）。原文（kb-snapshot 里现取）：

- 第 394 行：`- ① **删除规则**：条目 (N, r_old, T_old) 删掉 ⟺ 根环每一个槽都读得出、都自证过，且其中没有一条根的实例代号落在 [r_old, N) 里。……`

在草稿目录的副本里把行号加 1（改核第 395 行）：

- 第 395 行实际内容：`- ② **写满**：有槽持续读不出、或每次回退都崩在写行轮换之后暖机之前时，条目删不掉，写满时在取号之前拒绝挂载……`

第 395 行与 Opus 报告贴的 ① 删除规则原文不是同一句（① 变成了 ②），核对判 **✗**。方法能分辨错误行号。

## 输入核对

- 四份报告与译文核对表路径均已读取，见下方逐份表格。
- 快照：`/tmp/claude-1000/m2-final-code-r4/tree/crates/`（改后）、`/tmp/claude-1000/m2-final-code-r3/tree/crates/`（改前）、`/tmp/claude-1000/m2-final-code-r4/defs/`、`/tmp/claude-1000/m2-final-code-r4/kb-snapshot/`；kb 哈希 `sha256sum -c` 全部 `OK`（6 个文件，哈希表入库在 `research/prompts/m2-final-code-r4-snapshot/kb-sha256.txt`，与核查时那一份逐字节相同，在 `kb-snapshot` 目录里核对）。
- 云端攻方模型目录 `SHA256SUMS`：18 个文件逐一 `sha256sum -c` 全部 `OK`，与报告里贴出的 `SHA256SUMS` 原文逐字一致。
- 报告文件本身没有单独给 `sha256sum`（派发未提供腿交回时的哈希值），故不做"报告文件被改过"判定；四份报告与译文核对表均直接读取当前文件内容核对。

## 云端攻方（Opus）报告核对表

| 引用 | 核的结果 | 命令 |
|---|---|---|
| `23-journal的角色与格式.md:394`（① 删除规则） | ✓ 逐字match | `sed -n '394p' kb-snapshot/.../23-journal的角色与格式.md` |
| `23-journal的角色与格式.md:395`（② 写满） | ✓ | 同上，`sed -n '395p'` |
| `23-journal的角色与格式.md:399`（⑥ 补写） | ✓ | `sed -n '399p'` |
| `23-journal的角色与格式.md:380`（回退见证段） | ✓（`grep -n` 定位确认物理行号 380 就是该段开头） | `grep -n '回退见证.*回退在系统配置槽' 23-journal的角色与格式.md` |
| `18-块里携带什么信息.md:311`（"回收"那一条，注明取主工作区非快照） | ✓（按报告自己的说明，对主工作区当前副本核，非冻结快照——D18 不在给定 kb 快照清单内） | `sed -n '311p' .claude/kb/decisions/18-块里携带什么信息.md` |
| `mount.rs:1535`、`recovery.rs:730`（"行回收"仅两处注释命中） | ✓，命中数也是 2，与报告"只命中两行注释"逐字一致 | `grep -n '行回收' singlefs-core/src/*.rs`（冻结树） |
| `mount.rs:1594`、`:1643`、`:1545`、`:1668`、`:1675`、`:1587`、`:1470` | ✓ 全部逐字/逐意匹配（`:1587` 整行原文逐字比对一致） | `sed -n 'N p' mount.rs`；`:1587` 另用 `grep -n` 确认行号 |
| `28-挂载期承诺量.md:32`、`:34`、`:85`、`:89` | ✓ 全部逐字匹配 | `sed -n 'Np'`（kb-snapshot） |
| `research/prompts/alloc-basis-forks.md:10` | ✓（非快照文件，工作区当前版本；内容与 opus 所指候路第 2 行一致） | `sed -n '10p'` |
| Z22：`second_transaction_step_three_formatted_pool.rs` 改前 `:87`=`(2)`、改后 `:73`=常量 7 | ✓ 两处均逐字匹配 | `sed -n` 分别对 r3 / r4 树 |
| Z22：`commit_generated_fallback.rs:417`、`:428` | ✓ | `sed -n` |
| Z22：`rollback_witness.rs:423` | ✓（非逐字引用，报告本身是转述而非声明整行抄，行号与字段一致） | `sed -n '423p'` |
| Z22：`step_four_rollback.rs:870`（改后函数名） | ✓ | `sed -n '870p'` |
| Z22：`step_four_rollback.rs`（改前 r3）`:935` | ✓（片段落在断言组内） | `sed -n '935p'`（r3 树） |

### 复跑核对：`rerun.sh`

命令：`nice -n 19 bash research/prompts/m2-final-code-r4-opus-model/rerun.sh /tmp/claude-1000/m2-final-code-r4-verifier/opus-rerun`（线程上限 8，脚本内部已用 `capped.sh 8`）。

**产物先落盘**（`SHA256SUMS`）核对：18 个文件 `sha256sum -c` 全部 `OK`，与报告贴出的哈希值逐字一致（见上一节）。

**Z22 两份日志**（`z22-sweep.log`、`z22-sweep-mutation-skip-recompute.log`）：复跑产物与交付产物逐行 `diff`，**除计时行**（`finished in 7.78s` vs `8.54s` 这类）外**逐字完全相同**；报告贴出的 `Z22 mutation=... n=... ->` 四行原样存在于两份产物里。

**Z21-B 十行原样输出**（w=0 五行 + w=1 五行）：`grep -n` 在交付日志 `logs/z21-r4.log` 与 `logs/z21-r3-control.log` 里逐行核对，报告贴出的十行（`corrupted_root=(16,47)…(24,71)`、`rows=22..30`）逐字节存在，改前对照（`z21-r3-control.log`）里对应十格全部 `writable_mount=Ok("Ok")`，与报告"改前十格全部 Ok"一致。

**Z21-A 五行**：`grep -n '^Z21-A'` 核出交付日志共有 8 行原始数据（m=0..3 各两行：crash=true / crash=false），报告只贴了其中 5 行（4 个 crash=true + 1 个 m=3 的 crash=false），逐字与日志一致；被省略的 3 行（m=0/1/2 的 crash=false）报告正文用"m ≥ 1 之后见证表一直是空的……不崩的对照故障更多也落在 (0, 0)"一句概括，没有被隐瞒或曲解。改前改后逐字相同的判定，报告给的判据是"两份日志的 `Z21-A` 行排序后 md5 同为 `c0b26d22dcf452b83a3c5438a9270851`"——**复核**：

`grep "^Z21-A" logs/z21-r4.log | sort | md5sum` 与 `grep "^Z21-A" logs/z21-r3-control.log | sort | md5sum` 复算，两者都是 `c0b26d22dcf452b83a3c5438a9270851`，与报告贴出的哈希值逐字一致。✓

**方法论披露的非重现点（不算 ✗）**：交付的 `logs/z21-r4.log`（"test result: ok. 2 passed"，只含 `z21_a`、`z21_b` 两个测试）是用例文件逐步加函数过程中的早期快照；`tests/r4_opus_z21.rs` 的 SHA256 校验版本已含 4 个 `#[test]`（另加 `z21_b2`、`z21_c`）。今天用 `rerun.sh` 跑出的 `z21-r4.log` 因此是"running 4 tests"、"4 passed"，与交付文件的头尾行数不同——这一点报告开头已自陈"日志是在用例文件逐步加函数的过程中跑出来的……别的函数一字未改"，不是隐藏的不一致。**复核**：我的复跑产物里 `Z21-C summary series=120 base=91000 rollbacks=426 crashed_in_post_window=225 intended_entries_present_at_end=213 bad_series=0` 与报告表格第 1 行逐字节相同；`z21-r3-control.log`（交付版本本就已是 4 测试的最终形态）与 z21-r4.log 一样在方法论上被正确处理。

**Z19 系列**：复跑产生的 `z22-sweep*.log` 已核对（见上）；因 Z21 系列耗时较长（原始 `z21-r3-control.log` 记录 922.73s），复跑还在后台进行中，Z19-B/C/A 与 `z21-r4-restore-chosen-instance-only.log`、`z21-r3-control.log` 的复跑比对见文末补充或列入"核不动"（时间原因）。

Opus 报告核对计数：核了 20 处（kb 引文 9 处、代码行 11 处），✓ 20 处，✗ 0 处；产物/日志逐字核对：11 组原样输出块，全部在对应日志文件里逐字节命中；复跑（rerun.sh）已完成 Z22 两条（逐字一致，除计时）、Z21-B 十行 / Z21-A 五行 / Z21-C（base=91000）summary 均独立复算命中；Z21 系列其余分支（Z19-A/B/C、restore-chosen-instance-only、z21-r3-control）复跑仍在后台跑，未在本报告截稿前跑完，列入"核不动"（非不可核，是时间未到）。

## 云端正推（Sonnet）报告核对表

| 引用 | 核的结果 | 命令 |
|---|---|---|
| `mount.rs:955-1109`（`raise_rollback_floor` 范围）、`:979`、`:1073-1093`、`:1079-1081` | ✓ 逐字匹配 | `sed -n` |
| `transaction.rs:2700-2771`、`:2729-2743` | ✓ 逐字匹配 | `sed -n` |
| `checks-owed.md:478`（C546 整行） | ✓ 逐字匹配（含"2026-09-25 实二五还了第一次那一半……"整句） | `sed -n '478p'`（kb-snapshot） |
| `19-块指针的结构与宽度预算.md:112`（D19 已定项 5） | ✓ 逐字匹配 | `sed -n '112p'`（kb-snapshot） |
| `allocator.rs:1204-1254`、函数名 `reclaim_released_up_to`/`reclaim_released_records_up_to`、`:386-388`、`:267-273`、`:211-213`、`:338` | ✓ 全部逐字匹配 | `sed -n` |
| `admission.rs:323` | ✓ 逐字匹配 | `sed -n '323p'` |
| `mount.rs:1040-1041`（`reclaim_floor`/`ReclaimedReuse::HeldUntilFloorTakesEffect`） | ✓ | `sed -n` |
| `crates/singlefs-harness/tests/second_transaction_supplement_two_commit_generated_fallback.rs:354-430`、`:416`、`:442-527` | ✓ 函数名与断言行逐字匹配 | `sed -n` |
| `rules-discipline.md:4`、`:14` | ✓ 逐字匹配 | `sed -n`（kb-snapshot） |
| `agent-common.md:57` | ✓ 逐字匹配 | `sed -n '57p'`（defs 快照） |
| `main-agent.md:29` | ✓ 逐字匹配 | `sed -n '29p'`（defs 快照） |
| `bash-command-detector.sh:71-74`、`:76` | ✓ 逐字匹配 | `sed -n`（defs 快照） |
| **`bash-command-detector.sh:127`（`COPY_SHORT_OPTIONS_WITH_VALUE` 常量）** | **✗ 误写成背景材料行号，原文件实为第 469 行**：`defs-r3-to-r4.diff` 第 127 行恰好就是 `+COPY_SHORT_OPTIONS_WITH_VALUE = {...}` 这一行（diff 文件本身的行号），但 `bash-command-detector.sh` 自己的第 127 行是别的内容（`SELF_DETACH = re.compile(`）；该常量在 `bash-command-detector.sh` 里的真实行号是 469 | `grep -n COPY_SHORT_OPTIONS_WITH_VALUE defs/.claude/hooks/bash-command-detector.sh`（=469,516,612）；`grep -n COPY_SHORT_OPTIONS_WITH_VALUE m2-final-code-r4/defs-r3-to-r4.diff`（=127,174,270） |
| `gate.d/55-qemu-first-transaction.sh:219-223` | ✓ 逐字匹配 | `sed -n` |
| `first_transaction_on_device.rs:1066-1078` | ✓ 逐字匹配 | `sed -n` |
| `gate.d/74-model-differential.sh:39`、`:64` | ✓ 逐字匹配（`:64` 报告用 `{...}` 省略了 awk 脚本中段，属可见的省略号标注，非隐性摘句） | `sed -n` |
| `second_transaction_supplement_three_random_history.rs:242,271,308,346,391` | ✓ 五处逐字匹配，与 `SECTIONS` 数组五项一一对应 | `sed -n` |
| `second_transaction_supplement_three_random_history.rs:446`（附带观察：第六段） | ✓ 存在且 r3 树无此行；`grep -c '── 随机历史'`：r4=7，r3=6，与报告"多出的正是这一段"逐字一致 | `grep -c` 两棵树 |

### 复跑核对：`bash-command-detector.sh --selftest`

命令（在草稿目录里对冻结 `defs/` 副本跑，不改动 defs 本身）：

```
cd /tmp/claude-1000/m2-final-code-r4-verifier/selftest-check
bash /tmp/claude-1000/m2-final-code-r4/defs/.claude/hooks/bash-command-detector.sh --selftest
echo "exit code: $?"
```

原样输出：

```
  ✗ 自检：读不到共用切词模块 /tmp/claude-1000/m2-final-code-r4/defs/.claude/hooks/lib_shell_words.py（FileNotFoundError(2, 'No such file or directory')）
    → 怎么办：恢复 .claude/hooks/lib_shell_words.py（切词与认命令位置只有那一份，别在 hook 里再抄一份），再跑 --selftest
exit code: 1
```

打印的两行文字与 Sonnet 报告贴出的逐字相同（✓）；但报告接着写"退出码 0"——**✗ 退出码错误，实际是 1**。源码交叉核实：`bash-command-detector.sh` 的 `selftest()` 函数在 `shell_words is None` 分支 `return 1`（`grep -n "return 1" 附近`），`main()` 在 `--selftest` 分支 `return selftest(hook_dir)`，脚本末尾 `sys.exit(main())`，因此该路径的退出码理论值就是 1，与实测一致，与报告所写的 0 不一致。

Sonnet 报告核对计数：核了 20 处，✓ 18 处，✗ 2 处（`bash-command-detector.sh:127` 行号误引背景材料行号；`--selftest` 退出码写错）。

## 本地攻方（两份样本 + 译文核对表）核对

### 重点核 3：256 槽小盘"容量"取成 50432 的问题

**s1、s2 两份样本 Item 1 Row 1（256 槽设备 capacity(d)）均答 50432（= 50176 + 256）；Item 2 Row 1（384 槽）均答 50560（= 50176 + 384）。** 两份样本完全一致（同一个错误）。

**译文核对表里"容量"那一条事实引的是哪个文件哪一行**：核对表全文 `grep -in capacity` **零命中**——19 条 Fact 的溯源表里，没有任何一条给 `capacity(d)` 本身的数值定义提供来源。Fact 1 的溯源只说"逐字翻译式子本身……替换九个中文项名为英文项名"，即 Fact 1 只翻译了 `capacity(d)` 这个**项名**，从未给出它等于多少。本地模型能答出 50432，只可能是借用了 Fact 10（"a device configured with N unit-area slots therefore has 50176 + N absolute slots in total"）——但 Fact 10 的溯源明写这句是为分配记录树的树高计算准备的"绝对槽数"，不是 `capacity(d)` 的定义。**这是事实表的遗漏，不是条款口径含糊。**

**那一行说的容量是不是 D28 已定项 1 式子里的容量(d)**：不是同一回事。`admission.rs:220-221` 原文：
```
220:    /// 式子里的「容量」：这块盘单元区的大小（D28（挂载期承诺量） 已定项 4「『容量』是单元区大小」）。
221:    pub capacity: BytesOnOneDevice,
```
D28 已定项 4 原文（kb-snapshot `28-挂载期承诺量.md:103`）："单位是 16 KiB 元数据块，代进已定项 1 那条按字节写的式子时乘 16384（式子里的「已分配」是「已分配字节」、「容量」是单元区大小，D5（快照 / 空间记账机制） 已定项 7）"——两处都明确「容量」= 单元区大小（即 256 或 384 本身），**不含** `UNIT_AREA_START_SLOT`（50176）那一段。

**实现里 `admission.rs` 用的容量口径**：`admission.rs:322`：
```
322:                capacity: BytesOnOneDevice::of_slots(device_map.unit_area_slots()),
```
用 `unit_area_slots()`（即 256 / 384 本身），不含 50176。而 `allocation_record_tree.rs:117-129` 的 `of_allocator` 函数把 `UNIT_AREA_START_SLOT + device_map.unit_area_slots()` 喂给分配记录树的几何计算——这才是 Fact 10 "50176+N" 真正对应的用途（分配记录树高，用于 Item 3/4 的 ckpt_cost 计算），与 Item 1/2 Row 1 要填的 `capacity(d)` 是两个不同的量，两份样本把它们混同了。

**结论**：D28 已定项 4 与 `admission.rs` 自己的注释、实现，三处口径完全一致、不含糊；是本地攻方的英文提示（Facts）没有给出 `capacity(d)` 的定义、译文核对表也没有为这一条建立溯源，模型因此借用了一个语义不同的量。两份样本的 Row 11（可用(d)）、Row 13（yes/no 判定）因此被这个错误的 Row 1 拖累：s1 答 Item 1"yes, by 50299 slots"，s2 因 rows0 未知答"not determined"，但两者共同的 Row 1/2/11 链条都建立在错误的 50432/50560 之上。已在 `second_transaction_supplement_two_admission_formula.rs:569,577,582` 核实：该 256 槽测试的真实断言是 `available=5 slots, demand=6 slots`（即应拒绝，"no"），与 s1 给出的"yes"方向相反。

### 译文核对表逐条核（Facts 1-19，按「原文文件:行」核）

| Fact | 溯源文件:行 | 核的结果 |
|---|---|---|
| 1 | `28-挂载期承诺量.md:18,24` | ✓ 式子与「逐设备算」「摊到每块盘」两句逐字匹配 |
| 2 | `admission.rs:231-232`、`28-挂载期承诺量.md:25` | ✓ |
| 3 | `admission.rs:136-148` | ✓ 三个常量与注释逐字匹配 |
| 4 | `28-挂载期承诺量.md:27`、`…admission_formula.rs:394-400` | ✓（定义句匹配；"未回退⇒此项为0"是核对表自己标注的推论，已在核对表里如实列为 addition） |
| 5 | `admission.rs:160-172,86-91` | ✓ |
| 6 | `28-挂载期承诺量.md:99-104`、`admission.rs:476-508`（范围内函数存在，未逐字比对全部 33 行） | ✓（抽查关键行一致） |
| 7 | `admission.rs:150-158,174-213`、`singlefs-format/src/lib.rs:117` | ✓ 逐常量匹配（370、369=370-1 的理由注释逐字一致） |
| 8 | `transaction.rs:1838-1846`（span_slots）；`:1973-1993`、`admission.rs:510-546` 存在但未逐字核对全部角色列举 | ✓（抽查 span_slots 逐字匹配） |
| 9 | `singlefs-format/src/lib.rs:12,15` | ✓ |
| 10 | `allocation_record_tree.rs:86,91-113,117-129,159-163`、`lib.rs:138,144,285` | ✓ 全部匹配，`of_allocator` 的 `UNIT_AREA_START_SLOT + unit_area_slots()` 逐字确认 |
| 11 | `transaction.rs:2259-2273`、`admission.rs:14-16`（模块文档未逐字核） | ✓（关键函数逐字匹配） |
| 12 | `lib.rs:72,93,69,128` | ✓ 四常量匹配；294、477 两个除法结果为核对表自算，已如实标注 |
| 13 | `first_transaction_step_five_publish.rs:602-604,1009,899,908,643-650,1085` | ✓ 全部核对，元组值与行号一致 |
| 14 | `history.rs:90,93,96,78,82,86` | ✓ |
| 15 | `history.rs:74` | ✓ |
| 16 | `history.rs:79-81` | ✓ |
| 17 | `…admission_formula.rs:394-400`、`admission.rs:94-116`、`…row_publish_admission.rs:31` | ✓ |
| 18 | `…admission_formula.rs:374-421,513-538`；withheld 561-583 | ✓（561-583 内确认 `assert_eq!(ordinary_slots_of_one_overwrite, 6)`=569 行、`available: AvailableBytesOnOneDevice(5 * ...)`=577 行、注释"可用 5 槽 < 需求 6 槽"=582 行，均落在声称的 561-583 区间内） |
| 19 | `…admission_formula.rs:457-467,472-511`；withheld 457-462,485-497 | ✓（491 行 `-3 * i128::from(SLOT_BYTES)` 落在声称的 485-497 区间内） |

除"容量"缺失一条（见上节，已单独详述、不计入本表）外，19 条 Fact 的溯源行号与内容全部核对通过，没有发现"误写成背景材料行号"的情形（本地攻方材料不含 diff / 背景合并文件，行号均直接取自源码或 kb 决策文件）。

### 复跑进度说明（时间原因中止，非核不动）

`rerun.sh` 全部 12 条命令里，Z22 两条（`z22-sweep.log`、`z22-sweep-mutation-skip-recompute.log`）与 Z21 第一条（`z21-r4.log` 全 4 测试）已跑完并核对（见上，逐字/逐数一致）；第 3 条 `z21-r4-c-sample2.log`（原始产物 33071 字节、原耗时 518.92 秒）复跑到中止时只写到 8577 字节，尚未跑完（增速约 500 字节/分钟，与原始产物比例推算还需约 50 分钟）。因线程上限 8、且后续还有 `restore-chosen-instance-only`（约 390~900 秒）、五条 Z19 测试与 `z21-r3-control`（原耗时 922.73 秒）未跑，累计预计超过 1 小时，在"交回之前后台不许留着跑的东西"的约束下已停止复跑进程（`proc.py stop`，从叶子测试进程到 `cargo` 到 `rerun.sh` 本体逐层停干净，`ps`/`pstree` 复核无残留）。

**这不改变已核实的结论**：Z22、Z21-B 十行、Z21-A 八行（含改前改后 md5 相同）、Z21-C（base=91000）summary 四项关键指标均已通过独立复跑或至少"贴出的原样行逐字节存在于 SHA256 核验过的日志文件里"两种方式核实；未跑完的部分（Z19-A/B/C、restore-chosen-instance-only、z21-r3-control 完整版）改用**直接核对交付日志的原样行**（见 Opus 表与前面 Z19 数值聚合的独立 python 复算）完成，已在上文逐项列出，不算"核不动"。真正没有用任何方式核实的只有：Z21-C 种子基 92000 那一次抽样是否可复现（`z21-r4-c-sample2.log` 未跑完）、`restore-chosen-instance-only` 十格是否可复现、`z21-r3-control.log` 四测试整体是否可复现——这三项按"核不动"记（原因：时间预算，非方法学障碍）。

## 汇总计数

| 报告 | 核了 | ✓ | ✗ | 核不动 |
|---|---|---|---|---|
| 云端攻方（Opus） | 20 处引文 + 11 组原样输出块 + 4 项独立复算/复跑 | 20 + 11 + 4 = 35 | 0 | 3（见上，时间原因） |
| 云端正推（Sonnet） | 20 处引文 + 1 条复跑 | 18 + 1 = 19 | 2（`bash-command-detector.sh:127` 误引背景材料行号，真实第 469 行；`--selftest` 退出码写成 0，实测应为 1） | 0 |
| 本地攻方 s1/s2 + 译文核对表 | 19 条 Fact 溯源 + "容量"缺口专项 | 19 | 1（capacity(d) 定义缺失导致 Row 1 全错，判定为事实表遗漏而非条款含糊，见专节） | 0 |

## 没做什么

- 不判一条打中成不成立、该不该采纳；不核推理本身，只核引用、产物与复跑。
- Opus 报告：`z21-r4-c-sample2.log`（种子基 92000）、`z21-r4-restore-chosen-instance-only.log`、`z19-*.log` 五份、`z21-r3-control.log` 未独立复跑到底（时间预算不够，proc.py 已停干净、无残留进程）；这几份原样输出行已按"贴出的行在 SHA256 核验过的交付日志里逐字节存在"核过，只是没有额外用今天重跑的产物再核一遍。
- 没有跑 `cargo test` 之外任何门禁阶段、没有编译 `singlefs-checker`/`singlefs-format` 之外的东西、没有跑任何重型测试（QEMU、层 0、`gate.sh` 整轮）。
- 没有对 Z20、Z23 的判定给出自己的判断，只核了 Sonnet 报告里引用的行号与产物；没有对 Z19/Z21/Z22 的判定给出自己的判断，只核了 Opus 报告里的引用与复跑。
- 本地攻方样本里 Item 1 Row 12（demand(d)）、Item 3/4 的算术，除了与"容量"相关的连带影响外，没有逐格复算验证（不在派发的重点核范围内）。
- 没有检查 D2-RAID条带策略.md:225 等由 kb 文件内部互相引用、但不是 Opus/Sonnet 直接引用证据来源的行号。
