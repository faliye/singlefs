# m2-closeout-code-r1 核查员报告

**这是观测，不是判决**：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

## 0. 判别力自证

抽的引用：opus 报告第 42 行「两个系统配置槽一份自证过的都没有（校验和过、fsid 与本池相同），即这块盘不」（`.claude/kb/decisions/18-块里携带什么信息.md:314`）。
在草稿副本 `/tmp/claude-1000/m2-closeout-code-r1-verifier/selftest/opus-selftest.md` 里把行号 314 改成 315，用 `research/scripts/cite-check.py` 按第 2 步核：

```
✗ opus-selftest.md 第 42 行引「两个系统配置槽一份自证过的都没有（校验和过、fsid 与本池相同），即这块盘不」，位置写成 18-块里携带什么信息.md 第 315 行：那一行不含这句；原文在这份文件第 314 行
✗ 核了 6 处引文，1 处对不上，1 处没判（逐处列在上面）
```

判红，方法能分辨。（另一条「没判：19-块指针的结构与宽度预算.md」是因为自证用的根只挂了 kb-at-start 那 10 份文件，不含 D19；D19 不在这一轮 kb-at-start 快照名单里，按下文「核法说明」用主树核，不影响这次自证本身。）

## 核法说明

- 代码引文：核对 `refs/sop/m2-closeout-code-r1-snapshot` 的 `git archive` 副本（草稿目录 `/tmp/claude-1000/m2-closeout-code-r1-verifier/citecheck-tree/`），逐个用 `sha256sum -c` 与 `crates-src-sha256.txt` 核过一致（56 份全部 `OK`，命令与输出见下）。
- kb 引文：kb-at-start 10 份文件用的以 `kb-at-start/` 为准；报告引到的 kb 文件不在这 10 份里的（如 D19、D2、D28 部分小节、checks-owed.md 里的行——checks-owed.md 本身在 10 份内），先查它在主工作区的 `git log`/`git status`：mtime 与最近一次提交都早于对应腿的开工时刻（2026-09-27 起）且工作区无未提交改动的，按主树核，判定记 ✓/✗；本报告核到的这几份（D19、D2、D28、checks-owed.md 的引用行、`records/2026-09-27-代码审阅38条去向.md`）逐一查过，均无腿开工之后的改动。
- `research/scripts/cite-check.py` 只认「「quote」（`path:line`）」与「`path:line`：「quote」」两种同行写法；报告里大量用「kb 原文（`path:line`）：「quote」」（location 在括号里、quote 另起）这种脚本认不出的写法，以及跨物理行的多行引用（Rust `///` 文档注释换行、`- ` 列表符号），这些逐条人工核对，见下表。
- 复跑：把 opus 模型目录的 4 份用例与 `rerun.sh` 拷进快照 `git archive` 副本，在 `/tmp/claude-1000/m2-closeout-code-r1-verifier/rerun-opus/` 跑（含 `.git`？——实为纯 `tar` 副本，无 `.git`；`rerun.sh` 本身按快照 `git archive` 生成副本，未改写法），线程上限 5（`capped.sh 5`）、内存上限 12G（`run-with-memory-cap.sh 12G`），`nice -n 19`。

## 附：56 份代码快照哈希核对（`sha256sum -c`）

```
$ cd /tmp/claude-1000/m2-closeout-code-r1-verifier/citecheck-tree && sha256sum -c .../crates-src-sha256.txt
（56 行全部 OK，无 FAILED；见下方 opus 表内逐条代码引用核对）
```

## 攻方（Opus）报告核对表

| 引用 | 核的结果 | 命令/说明 |
|---|---|---|
| 第 3 行「代码位置不用「」引号写」的自我说明 | ✓ 属实：全篇代码位置引用确实极少用「」引号直接抄源码原文，多为函数名/描述，与逐字抄写的规范不冲突（不是转述纪律要拦的失真，是刻意选的写法） | 通读确认 |
| D18-已定项?（`18-块里携带什么信息.md:314`，第 42 行） | ✓ | 见判别力自证一节，未改行号时判绿 |
| `mount.rs:2947-2956` 文档字面「照可写挂载同一套核」（第 42 行，无「」引号） | ✓：`mount.rs:2947-2956` 在快照里确有该函数文档 | `sed -n '2947,2956p' crates/singlefs-core/src/mount.rs`（citecheck-tree） |
| D23 已定项 14（`23-journal的角色与格式.md:379`，第 90 行）「cur 那一版的账里仍分配、而 new 不引用的落点，释放代 = new 的 txg」 | ✓ | cite-check.py 判绿（第 6 处之一） |
| D19 已定项 5 硬规则 1（`19-块指针的结构与宽度预算.md:112`，第 91 行）「另一块盘上那一份对得上也一起留，各盘的账保持对称」 | ✓，但该文件不在 kb-at-start 10 份快照内；查过 `git log`/`git status`：最近一次改动 2026-09-25，早于腿开工，工作区无未提交改动 → 按主树核，行 112 逐字含引文 | `grep -n` 现查；见上文核法说明 |
| D23 已定项 14「在任何写之前拒」一格（`23-journal的角色与格式.md:383`，第 111 行）「cur 的账里仍分配的单元分配器不会发出去，不在逐盘验之内」 | ✓ | cite-check.py 判绿 |
| D13 已定项 4（`13-验证路线.md:71`，第 155-157 行）两条子引文（「不是单元写……这一类」「新旧不同的那一截……不按扇区」） | ✓✓：两条都逐字含在 kb-at-start D13 第 71 行内；cite-check.py 因引用位置与「」quote 不在同一行未捕获，人工核对 | `sed -n '71p'` 比对，见报告文本 |
| D13 已定项 4（`13-验证路线.md:73`，第 159 行）「两条层 0 流上取第三态的只有系统配置槽写」 | ✓ | cite-check.py 判绿（第 6 处之一） |
| D13 已定项 4（`13-验证路线.md:71`，第 175 行）「一次写只被它自己那块盘上之后的屏障（或它那块盘上的 FUA 写）排在之后的写前面」 | ✓ | cite-check.py 判绿 |

### Opus 报告：产物（log 文件）逐字核对

原始产物 `research/prompts/m2-closeout-code-r1-opus-model/logs/*.log` 与报告正文里贴的原样输出行，逐字比对（`grep -F` 命中即视为逐字一致）：

| 报告贴的行（节选） | 命中的日志文件 | 结果 |
|---|---|---|
| `Z3A unmount_blank accepted=true outcome_is_err=false writes_dev0=28 writes_blank_dev1=28` | `logs/run-z3a.log` | ✓ 命中 |
| `Z3A unmount_blank read_only_after chosen=(1,6) effective=(1,6) floor=5` | `logs/run-z3a.log` | ✓ 命中 |
| `Z3A unmount_blank checker_violations_after=4 first=Some("I-2.1: 树表单元 在盘 1 槽 50284 的那一份与位置条目里的校验和对不上")` | `logs/run-z3a.log` | ✓ 命中 |
| `Z3A unmount_blank writable_after err=WritableMountRefusedByDevicesWithoutTheSelectedVersion { ... }`（报告截断） | `logs/run-z3a.log` | ✓ 报告里省略号截断处之前的字面与日志逐字一致 |
| `Z1Q-B target_txg=4 rollback_txg=6 quarantined_slot=50180 record_before=Some((false, 3)) record_after=Some((false, 3)) released_user_visible_units=4` | `logs/run-z1q.log` | ✓ 命中 |
| `Z1Q-A checker_violations_before_rollback=3 first=Some("I-2.1: 数据单元 在盘 1 槽 50180 的那一份与位置条目里的校验和对不上")` | `logs/run-z1q.log` | ✓ 命中 |
| `Z1Q-A rollback_to_txg3 accepted=true err=None resurrected=Some(6)` | `logs/run-z1q.log` | ✓ 命中 |
| `Z5 split_points_tried=40312 self_verifying_other_than_old_and_new=0 midpoint_self_verifying=0` | `logs/run-z5.log` | ✓ 命中 |
| `Z5 fua_total=29 fua_with_unflushed_plain_writes_on_its_device=0` | `logs/run-z5.log` | ✓ 命中 |
| 随机走 4 GiB 表格行（步数 12000、回退做成 2075、checker 红 0、读回错 0、534s） | `logs/b2-1.log` | ✓：`rollback ok = 2075`、`checker_red_steps=0`、`read_back_wrong=0` 命中；耗时表格写 534s，`b2-1.log` 内 `elapsed_ms` 字段见下方复跑核对 |
| 随机走 384 槽表格行（12000、2065、0、0、419s） | `logs/b2-2.log` | ✓：字段命中，见下方复跑核对 |

### Opus 报告：复跑核对（本次在 `/tmp/claude-1000/m2-closeout-code-r1-verifier/rerun-opus/` 用 `rerun.sh` 重跑）

三条目标测试（`opus_r1_z3_entries_with_a_substituted_device`、`opus_r1_z1_rollback_and_quarantined_copies`、`opus_r1_z5_torn_split_points_and_fua_release`）与随机走两个宽度（4 GiB、384 槽）全部跑完，逐字节输出核对：

- Z3-A、Z1Q-A、Z1Q-B 的全部原样输出行（`Z3A ...`、`Z1Q-B ...`、`Z1Q-A ...`）与报告贴的、与 `logs/run-z3a.log`、`logs/run-z1q.log` 逐字相同（`grep -n "Z3A\|Z1Q" 本次日志` 与原日志对比，见上表）。
- Z5 的 `split_points_tried=40312 self_verifying_other_than_old_and_new=0 midpoint_self_verifying=0`、`fua_total=29 fua_with_unflushed_plain_writes_on_its_device=0` 两行本次复跑与原日志逐字相同——40312 个撕裂镜像、0 个自证的结论可复现。
- 随机走 4 GiB 宽度（种子 201..401，60 步）：`steps_total=12000 rollbacks_done=2075 multi_unit_rollbacks=598 checker_red_steps=0 read_back_wrong=0`，`outcome rollback not-candidate BelowEffectiveFloor = 423`、`VersionWithoutFile = 190`，与报告表格「4 GiB：12000/2075/598/0/0」、「候选集拒绝 423/190」逐字一致；本次耗时 603.05s（报告写「534 s」，机器上同时有别的会话在跑 cargo，挂钟不同不改变步数与红/错计数）。
- 随机走 384 槽宽度：`steps_total=12000 rollbacks_done=2065 multi_unit_rollbacks=592 checker_red_steps=0 read_back_wrong=0`，`outcome rollback not-candidate BelowEffectiveFloor = 433`、`VersionWithoutFile = 190`，与报告表格「384 槽：12000/2065/592/0/0」、「候选集拒绝 433/190」逐字一致；本次耗时 388.06s（报告写「419 s」，同上）。

### Opus 报告：数出来的量核对

- 「随机走 24000 步...checker 红 0、冷恢复读回错 0」：24000 = 2 宽度 × 12000（12000 = 200 种子 × 60 步）。两路均已复跑确认：4 GiB 12000/2075/598/0/0，384 槽 12000/2065/592/0/0。
- 「Z5 的 40312 个撕裂镜像」：已复跑确认，`split_points_tried=40312`。
- Z1 表格里「拒绝的只有候选集那两种（`BelowEffectiveFloor` 423 / 433，`VersionWithoutFile` 190 / 190）」：4 GiB 一路复跑得 423 / 190，384 槽一路复跑得 433 / 190，与报告一致。

## 攻方（Opus）报告小计

- 核了引文 9 处（含判别力自证用的 1 处，按对上算）；✓ 9 处；✗ 0 处；核不动 0 处；分不清 0 处。
- 产物逐字核对 10 行，全部命中原始 log 文件（✓ 10）。
- 复跑：3 条目标测试 + 随机走 2 个宽度（4 GiB、384 槽）全部复现（输出逐字一致）。

## 正推（Sonnet）报告核对表

自动核对（`cite-check.py`，识别到 8 处「同行 quote+location」引文）：

```
✗ m2-closeout-code-r1-sonnet-output.md 第 126 行引「只在这份单测里用，核下面那几个字面量；core、checker、模型各自的那一份不调它」，位置写成 crates/singlefs-format/src/lib.rs 第 507 行（对着快照副本核）：那一行不含这句；原文在这份文件第 464 行
✗ 核了 8 处引文，1 处对不上，0 处没判
```

这一处：报告正文（第 126 行）自己写明「这一句在派发提示给的快照…里住在同一文件 464-465 行；行号不同是因为主工作区里 C11b…已经在这份文件顶部插入了整段新内容——本报告核对过快照 464-465 行与主工作区 507 行文字完全相同，只挪了位置，不影响这一格的判定」。现查：`citecheck-tree` 快照副本第 464-465 行与主工作区当前第 505-510 行区间内的对应两行，逐字相同（`sed -n` 比对，见下）。**判定**：cite-check.py 判 ✗ 成立（字面写的行号 507 在快照里不含这句），但腿已自报这一处替换、且替换后的内容核对无误——不是隐藏的引用错误，是disclosed 且verified 的行号口径切换。

```
$ sed -n '464,465p' citecheck-tree/crates/singlefs-format/src/lib.rs
    /// 码 2 头宽按字段表的三段加起来：只在这份单测里用，核下面那几个字面量；core、checker、模型各自的那一份不调它
    /// （三份的交叉断言在 `crates/singlefs-harness/tests/index_node_header_width_computed_three_ways_agrees_for_every_key_width.rs`）。
$ sed -n '506,507p' crates/singlefs-format/src/lib.rs
    /// 码 2 头宽按字段表的三段加起来：只在这份单测里用，核下面那几个字面量；core、checker、模型各自的那一份不调它
    /// （三份的交叉断言在 `crates/singlefs-harness/tests/index_node_header_width_computed_three_ways_agrees_for_every_key_width.rs`）。
```

余下 7 处 cite-check.py 判绿的（D28:19、D28:24、D28:82、D28:84、D28:88、walk.rs:4821、walk.rs:566）不再逐条列，均 ✓。

### Sonnet 报告：人工核对（cite-check.py 认不出「kb 原文（`path:line`）：「quote」」写法，逐条现查）

| 格 | kb 文件:行 | 结果 |
|---|---|---|
| Z2-3 | `28-挂载期承诺量.md:106` | ✓ 逐字含「**形态**：ckpt_cost = Σ（每棵记录树当前的高）+ 记账树每发布的节点数 + 1（树表），每次发布按当时的树高重算」 |
| Z2-4 | `02-RAID条带策略.md:224` | ✓ 逐字含引文（挂载准入例外那一句） |
| Z2-4 | `checks-owed.md:507` | ✓ 逐字含 C565 一行 |
| Z4-1 | `invariants.md:34`（I-1.11，两处引文：条款正文 + 实现状态列） | ✓✓ 两处都逐字含在该行内 |
| Z4-2 | `invariants.md:87`（I-8.6，两处） | ✓✓ |
| Z4-3 | `invariants.md:181`（I-5.1） | ✓ |
| Z4-4 | `invariants.md:55`（I-7.4） | ✓ |
| Z4-5 | `invariants.md:58`（I-7.7） | ✓ |
| Z6-1 | `13-验证路线.md:134` 与 `:140`（两行） | ✓✓ |
| Z2-3 | `records/2026-09-27-代码审阅38条去向.md:50` | ✓ 逐字含「20 | 「改条款按盘分路计并改实现」…kb 写回 D28 已定项 4」；该文件 mtime 早于两条云端腿开工时刻，按主树核 |
| Z2-3、Z4-3 | 代码：`admission.rs:494`、`walk.rs:588-589`、`walk.rs:2414-2421`、`walk.rs:2432` | ✓✓✓✓ 均逐字命中快照副本对应行 |

## 正推（Sonnet）报告小计

- 核了引文 19 处（8 处机检 + 11 处人工，含判别力自证之外新核的全部 kb/代码/记录引用）；✓ 18 处；✗ 1 处（disclosed 且已核对内容一致，见上）；核不动 0 处；分不清 0 处。
- 报告自述「不跑测试」，本核查没有为 Z2/Z4/Z6/Z7 单独复跑 `cargo test`（这几格不在主 agent 给的必核复跑清单里；主 agent 要求复跑的四组均属 Opus 腿）。

## 本地攻方腿核对表

### 干净样本与损坏闸复核（重跑 `corruption-check.py` / `oov-check.py`）

| 项 | 运行记录声称 | 本次复核 | 结果 |
|---|---|---|---|
| s1 词数 | 1895 | `wc -w` = 1895 | ✓ |
| s2 词数 | 890 | `wc -w` = 890 | ✓ |
| s1 sha256 | `ec33326d...c91c` | 复算一致 | ✓ |
| s2 sha256 | `d4922f6f...631b00` | 复算一致 | ✓ |
| s1 corruption-check | 绿 | 绿，`星号落单=0 粘连=0 实词自复读=0` 等全 0 | ✓ |
| s1 oov-check | 绿，生词=1（`plausibly`） | 绿，`生词=1 拼接=0`，生词 `plausibly` | ✓ |
| s2 corruption-check | 绿 | 绿，全 0 | ✓ |
| s2 oov-check | 绿，生词=2（`segments'`） | 绿，`生词=2 拼接=0`，生词 `segments'` | ✓ |
| void1 corruption-check | 红，「星号落单=1 粘连=1」 | 红，`星号落单=1 粘连=1` | ✓ |
| void2 corruption-check | 红，「星号落单=1」 | 红，`星号落单=1 粘连=0` | ✓（粘连未提及，现查为 0，与「未提及即 0」一致） |
| 提示无 markdown 强调 | 未声明，规则要求 | `grep -c '\*\*'` = 0 | ✓ |

### 提示里给的事实表数字核对（对仓里钉的数，不核样本答复本身）

| 事实 | 出处 | 核对结果 |
|---|---|---|
| 第一条流全量 150994980 | `first_transaction_step_seven_layer0.rs` `FULL_STATES` | ✓ 快照第 439 行 `const FULL_STATES: u64 = 150_994_980;` |
| 第一条流快档 54 | 同文件 `QUICK_TIER_STATES` | ✓ 第 442 行 `= 54;` |
| 第二条流全量 14960689284 | `second_transaction_step_zero_layer0.rs` `FULL_STATES_THROUGH_THE_UNMOUNT` | ✓ 第 91 行 |
| 第二条流快档 390 | 同文件 `QUICK_TIER_STATES_THROUGH_THE_UNMOUNT` | ✓ 第 100 行 |
| 头宽 k=8 → 131 | `singlefs-format/src/lib.rs` `INODE_TREE_ROOT_INDEX_NODE_HEADER_BYTES` | ✓ 第 59 行 `= 131;` |
| 头宽 k=24 → 163 | 同文件 `EXTENT_TREE_INDEX_NODE_HEADER_BYTES` | ✓ 第 65 行 `= 163;` |
| ⌊(4096−311)÷56⌋=67 | 同文件 `JOURNAL_RECORD_BYTES=4096`、`JOURNAL_HEADER_BYTES=311`、`JOURNAL_NAMED_ENTRY_BYTES=56`（式子算出）、自测断言 `assert_eq!(JOURNAL_NAMED_ENTRIES_PER_RECORD, 67, ...)` | ✓ 第 191、200、404-405 行；算术 (4096-311)/56=3785/56=67.59…→67 |
| A4b 三格 ckpt_cost（`two-4GiB`、`two-1GiB`、`two-4GiB-mapping-4-8`，每格 overwrite@txg4 那一行） | `m2-rev-a4b-implementer-report.md` 第三节原样行 | ✓✓✓ 三行逐字命中该报告第三节代码块（`grep -F` 命中，见下） |

### A4b ckpt_cost 三行来源核对（`grep -Fn` 命中 `m2-rev-a4b-implementer-report.md`）

```
70:  1x two-4GiB-mapping-4-8 overwrite h_alloc=3 h_map=2->2 acc_before=1 ckpt_by_height=7 ckpt_a4=9 ckpt=24 alloc=d0:2,d1:2+root1 map=4 acc=1 tt=1 fixed=11 diff=-13 diff_a4=2 | overwrite@txg4
87:  1x two-1GiB overwrite h_alloc=2 h_map=1->1 acc_before=1 ckpt_by_height=5 ckpt_a4=6 ckpt=8 alloc=d0:1,d1:1+root1 map=1 acc=1 tt=1 fixed=6 diff=-2 diff_a4=0 | overwrite@txg4
90:  1x two-4GiB overwrite h_alloc=3 h_map=1->1 acc_before=1 ckpt_by_height=6 ckpt_a4=8 ckpt=12 alloc=d0:2,d1:2+root1 map=1 acc=1 tt=1 fixed=8 diff=-4 diff_a4=0 | overwrite@txg4
```
均与 `m2-closeout-code-r1-local-attack.md` C3 里三行原样一致（第三节，逐字抄，非摘句）。

### 翻译核对表逐行核（`research/scripts/cite-check.py` 先跑，机检 5 处 ✗、5 处「没判」，逐条人工复核）

自动核对结果：

```
✗ ...translation-audit.md:47 引 first_transaction_step_seven_layer0.rs:437-438「这条流上取三态的只有 8 次系统配置槽写，...」：那一行不含这句
✗ ...translation-audit.md:52 引 crash.rs:1725-1728（快照副本，非主工作区）「段内原地写（单元写之外的写）各取它的几态、...」：那一行不含这句
✗ ...translation-audit.md:57 引 first_transaction_step_seven_layer0.rs:440-441「A 那一段 2 原地写...」：那一行不含这句
✗ ...translation-audit.md:62 引 second_transaction_step_zero_layer0.rs:89-90「这条流上取三态的只有 46 次系统配置槽写...」：那一行不含这句
✗ ...translation-audit.md:72 引 admission.rs:476-481「checkpoint 保留池的 ckpt_cost...」：那一行不含这句
- 没判：...:10/15/20/25/30 引 _m2-closeout-code-r1-body.md:96：路径认不出（仓里没有这个文件名）
```

**「没判」5 处**：路径解析失败是我核查时 `--root` 顺序把 kb-at-start 的迷你 git 仓当 `roots[0]` 取 basename 索引的副作用（不含 `research/prompts/`），不是报告本身的问题。人工核：`sed -n '96p' research/prompts/_m2-closeout-code-r1-body.md` 一行同时逐字含五条引文的全部片段（「按事实表逐格填数、与仓里钉的数比」「闭式怎么来（按段序列与每段写数，第三态记一格）」「A4b 量表里 ... 三格的 ckpt_cost 各项」「头宽 `86 + 2 × k + 29` 在 k = 8、24 上」「一条记录装几个点名项」）——**5 处均 ✓**。

**机检 5 处 ✗，人工逐一复核，定位到两种不同的根因**：

| 行 | 根因 | 结论 |
|---|---|---|
| 47（first_transaction:437-438） | Rust `///` 文档注释跨物理行；`cite-check.py` 把换行折成一个空格，而原文两行紧接处本无空格（中文连排），导致机械比对里多出一个空格 | 剥掉 `///` 前缀、不在行拼接处插空格后逐字节命中——**✓，机检假红** |
| 52（crash.rs:1725-1728） | 同上（`///` 折行）+ 引用范围多带 1 行（1728，紧邻内容之外），不影响命中 | 剥掉后逐字命中——**✓，机检假红** |
| 57（first_transaction:440-441） | 同上（`///` 折行） | 剥掉后逐字命中——**✓，机检假红** |
| 62（second_transaction:89-90） | 同上（`///` 折行） | 剥掉后逐字命中——**✓，机检假红** |
| 72（admission.rs:476-481） | **真实错误**：引文末段「树表 1（每次发布重写一个单元，D16（发布语义） 已定项 9）；实例表链不进（它的开销归已定项 3 的切换预留）」实际住在第 482 行，在给的行区间 476-481 之外；同时该行是 rustdoc `- ` 列表项，机检还额外撞上了折行空格 | **✗，真实的行区间误差**：应写 `476-482`，不是 `476-481` |

**另一处人工发现（cite-check.py 抓不到，因为它不是「文件:行」格式的引文，而是核对表自己对「定稿」发给模型的英文措辞的断言）**：

第 2 条（Fact set A 任务句）「定稿：「with the third state recorded as its own column」」——现查 `research/prompts/m2-closeout-code-r1-local-attack.md` 全文（`grep -n "column"`、`grep -n "third.*state"`），**这个短语一次都没出现**。实际发给模型的 Fact-set-A 任务句是：「TASK for fact set A: build a table with one row per segment in A1 (10 rows...). For each row, state: the segment's write count n; whether you classify it as one of the "four system-configuration-slot segments"...; the full-enumeration state count for that row using the A3 formula; the quick-tier state count for that row using the A5 formula...」——确实要求逐行分类 + 两套状态数分列成表格的不同列，但没有用核对表里断言的那句话。**✗：核对表第 2 条「定稿」一栏写的最终英文与实际发出去的提示文本不符**（不是漏译或摘句，是核对表本身对「发出去的是什么」这句自证性断言与实际提示文件不一致）。

其余「多出来的」「首稿缺的」判定逐条与实际提示文件核对（详见下方复核清单），除第 2 条外全部属实：

| 条 | 断言 | 核对 |
|---|---|---|
| 1 | 任务开场句英文含「show the arithmetic」「state whether it matches」 | ✓ `research/prompts/m2-closeout-code-r1-local-attack.md:1` 逐字含 |
| 8（A6，「不译补第三态之前 29」） | 定稿只保留 what 不译历史对照数 | ✓ 提示 A6 段落逐字与「英文项」一致，无该数字 |
| 11（B3，补「n is the segment's total write count」） | 定稿补了这句 | ✓ 提示第 45 行逐字含 |
| 13（C1，只保留 what 不译 why） | 定稿不含「用户 2026-09-27 定」「它的开销归已定项 3」两处历史/理由括注 | ✓ 提示 C1 段落逐字核对，两处均未出现 |
| 14（C2，同上） | 定稿只保留公式与验证点，不译两个实写对照数 | ✓ 提示 C2 段落无「实写 5」「实写 7」 |
| 15（新加「ambiguous row」提示纪律） | 提示第 100 行含该句 | ✓ 逐字命中 |
| 16（新加「what observation would show this wrong」） | 提示第 5 行含该句 | ✓ 逐字命中 |

## 本地攻方腿核对小计

- 干净样本核对（sha256、词数、损坏闸复核）：核了 12 项，✓ 12 项。
- 事实表数字核对：核了 7 项，✓ 7 项。
- A4b 三行来源：核了 3 项，✓ 3 项。
- 翻译核对表逐行核对：核了 16 条（14 条引文/断言 + 2 条「多出来的」自陈）；✓ 14 条（含 4 条机检假红、5 条机检「没判」经人工核实为 ✓）；✗ 2 条（admission.rs 行区间应为 476-482、第 2 条「定稿」文本与实际提示不符）。

## 全轮汇总

| 腿 | 核了 | ✓ | ✗ | 核不动 | 分不清 |
|---|---|---|---|---|---|
| 攻方 Opus | 9（引文）+ 10（产物行）+ 4（复跑目标：核对全部四条已完成复跑，逐字/逐字段一致） | 23 | 0 | 0 | 0 |
| 正推 Sonnet | 19（引文） | 18 | 1（disclosed，内容核对一致） | 0 | 0 |
| 本地攻方 | 12（样本/闸）+ 7（数字）+ 3（A4b 行）+ 16（翻译表） | 36 | 2 | 0 | 0 |

**复跑最终结果**：opus 模型目录 `rerun.sh` 全部四条命令跑完（3 条目标测试 + 随机走 2 个宽度），逐行/逐字段核对：

- Z3-A、Z1Q-A、Z1Q-B 全部原样输出行逐字与报告、与原始 `logs/run-z3a.log`、`logs/run-z1q.log` 一致。
- Z5：`split_points_tried=40312 self_verifying_other_than_old_and_new=0 midpoint_self_verifying=0`、`fua_total=29 fua_with_unflushed_plain_writes_on_its_device=0` 与原始 `logs/run-z5.log` 一致。
- 随机走 4 GiB 宽度：`steps_total=12000 rollbacks_done=2075 multi_unit_rollbacks=598 checker_red_steps=0 read_back_wrong=0`，候选集拒绝 `BelowEffectiveFloor=423`、`VersionWithoutFile=190`，与报告表格一致（复跑耗时 603.05s，报告写 534s，机器负载不同导致挂钟差异，不影响步数/红/错计数）。
- 随机走 384 槽宽度：`steps_total=12000 rollbacks_done=2065 multi_unit_rollbacks=592 checker_red_steps=0 read_back_wrong=0`，候选集拒绝 `BelowEffectiveFloor=433`、`VersionWithoutFile=190`，与报告表格一致（复跑耗时 388.06s，报告写 419s）。

四条复跑命令全部逐字段可复现，没有一条打不中或数字漂移的。

## 没做什么

- 不判一条打中成不成立、该不该采纳；不核推理本身，只核引用、产物与复跑。
- 不判 Sonnet 报告 Z2/Z4/Z6/Z7 各格的代码语义是否真的兑现条款——只核引文字面与快照/主树是否逐字对应，未单独复跑 `cargo test`（这几格不在主 agent 给的复跑清单里；Sonnet 自己也声明「不跑测试」）。
- 不判本地攻方两份样本（s1、s2）里模型填的具体算式和数字对不对——运行记录本身也写明「不解读、不总结、不采纳两份样本里的具体答复，判它打没打中是主 agent 的事」，本报告只核样本本身干净（无损坏、词数/哈希对得上）与提示里给的事实表数字对不对，不核模型答案。
- 不判两份样本方向是否一致（运行记录里也未做）；没有主 agent 给的答案表，5b 步逐格比数没有做（运行记录已说明，本报告同样未补做，因为主 agent 派发提示里没有给答案表路径）。
- 不重跑 Sonnet 报告点名的 `index_node_header_width_computed_three_ways_agrees_for_every_key_width.rs`、`checker_narrow_invariants_and_abandoned_roots.rs` 等用例——不在主 agent 给的必核清单内，且 Sonnet 自己已声明未跑、若跑不过会推翻对应格判定。
- 未核 D19、D2、D28、checks-owed.md、`records/2026-09-27-代码审阅38条去向.md` 这几份不在 kb-at-start 快照内的文件在“两条云端腿开工时刻之间”是否被改过——只核了“最近一次改动早于两条腿的开工时刻、且现在工作区无未提交改动”，足以支持按主树核，但没有对每一分钟做穷举式时间戳核验。
- 未跑任何名字带 `layer0` 的目标、崩溃注入快档、`gate.sh`；复跑严格限定在主 agent 点名的四组目标，均带 `--test <目标>`，线程上限 5（`capped.sh 5`），内存上限 12G。
- 草稿目录 `/tmp/claude-1000/m2-closeout-code-r1-verifier/`（含 `citecheck-tree/`、`rerun-opus/`、`selftest/`）未删——交回前会按流程清理并在交回正文里逐一写明。
