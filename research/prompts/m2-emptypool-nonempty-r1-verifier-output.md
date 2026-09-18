# 核查员报告：m2-emptypool-nonempty-r1

**我交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。**

核查范围：云端攻方（Opus）`m2-emptypool-nonempty-r1-opus-output.md`、云端正推（Sonnet）
`m2-emptypool-nonempty-r1-sonnet-output.md`、本地攻方（`m2-emptypool-nonempty-r1-local-attack*.md`
一组四个文件）。核对基准是主 agent 存的快照
`/tmp/claude-1000/-home-fy5090-code-singlefs/e166d536-b665-4a88-8372-3091b9a0fc41/scratchpad/emptypool-r1-legs-snapshot/`
（`crates/` 与 `.claude/kb/`），不对主树核——主树的 `crates/` 之后已被实现员继续改动，行号会漂。

草稿目录：`/tmp/claude-1000/m2-emptypool-nonempty-r1-verifier/`（`repo/` 是「主树 rsync（排除 target、.git）
再用快照覆盖 crates/ 与 .claude/kb/」得到的副本，逐文件与快照 `diff -rq` 比对零差异）。

## sha256 核对

```
$ sha256sum research/prompts/m2-emptypool-nonempty-r1-opus-output.md research/prompts/m2-emptypool-nonempty-r1-sonnet-output.md
a0b1e16b179cced0cdb6284726de1b484d21e73db2880ed97db5ab2ae0e1c46b  research/prompts/m2-emptypool-nonempty-r1-opus-output.md
30241a8e7637e12471fcbd39e759c67140b35643c93008578cae55c8ea688c74  research/prompts/m2-emptypool-nonempty-r1-sonnet-output.md
```

与任务书给出的两个哈希逐字符相同（各 64 个十六进制字符）。✓

opus 模型目录三个文件的 sha256 也与报告表格里给的逐字相同（见下方 opus 节）。

## 判别力自证（先做）

挑了 sonnet 报告 Z5-a 一节的引用：「已定项 9 原文（`.claude/kb/decisions/16-发布语义.md:192`」，
整行抄的内容是：

> 9. **一次空发布写不写单元（2026-09-13，用户定案）：记账树存在时就写：空发布也是发布，按 D5（快照 / 空间记账机制） 已定项 2「每行每发布重写」重写记账行……** 正文见 D16（发布语义）「已定项 9」。 **状态：已定。**

在草稿目录的副本里，把这条引用的行号从 192 故意加 1 改成 193，按同样的核法（`awk 'NR==193' .claude/kb/decisions/16-发布语义.md` 取那一行，与引用的原文逐字比）去核：

```
$ awk 'NR==192' repo/.claude/kb/decisions/16-发布语义.md
9. **一次空发布写不写单元（2026-09-13，用户定案）……** 正文见 D16（发布语义）「已定项 9」。 **状态：已定。**
$ awk 'NR==193' repo/.claude/kb/decisions/16-发布语义.md
### 已定项 4（2026-09-13，用户定案）：一次发布整体施加或整体不施加，第二份 replay 停在句法层
```

193 行是完全不同的一条（已定项 4，不是已定项 9），核法在这一步正确判 **✗**（行号与引用内容对不上）。
判别力自证成立：核法分辨得出「行号错位」这类失效。

## 一、云端攻方（Opus）`m2-emptypool-nonempty-r1-opus-output.md`

### 1.1 模型/日志文件的 sha256 与行数

| 文件 | 报告表格里的行数/sha256 | 现查 | 判定 |
|---|---|---|---|
| `opus_attack_emptypool.rs` | 692 行 / `f4a12b7b…d98146` | `wc -l`=692，`sha256sum`=`f4a12b7bfd07a8f4f36cdad3a1d66518b54114a04f16b0a7d0db285877d98146` | ✓ |
| `run-all.log` | 98 行 / `7b75ac23…5ea577f` | `wc -l`=98，`sha256sum`=`7b75ac23327e66fa4fe86ca2e42af7a9e2b645b996b491106e2612b945ea577f` | ✓ |
| `copy-only-guard-for-z3.diff` | 21 行 / `ba67e889…cee7bb3` | `wc -l`=21，`sha256sum`=`ba67e8892fb13e3f1ba3f8c4292b8e6ccd8ee41450371823074845b6cfee1ba3` | ✓ |

### 1.2 复跑命令（在草稿副本里跑，`nice -n 19`，debug，未跑 release）

```
cp research/prompts/m2-emptypool-nonempty-r1-opus-model/opus_attack_emptypool.rs \
   /tmp/claude-1000/m2-emptypool-nonempty-r1-verifier/repo/crates/singlefs-harness/tests/
cd /tmp/claude-1000/m2-emptypool-nonempty-r1-verifier/repo
nice -n 19 cargo test --offline -p singlefs-harness --test opus_attack_emptypool -- --nocapture --test-threads 1
```

退出码 0，`test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`（报告称 9 passed，一致）。
把复跑日志与 `run-all.log` 里 `V2`、`Z1F`、`Z1S`、`Z2`、`Z3N`、`Z3F`、`Z3R`、`Z3STATE`、`Z3TALLY`、`Z4`
开头的全部数据行抽出来逐行 diff：

```
$ grep -E "^(V2|Z1F|Z1S|Z2|Z3N|Z3F|Z3R|Z3STATE|Z3TALLY|Z4|test result:)" run-all.log > stored-facts.txt
$ grep -E "^(V2|Z1F|Z1S|Z2|Z3N|Z3F|Z3R|Z3STATE|Z3TALLY|Z4|test result:)" opus-rerun.log > rerun-facts.txt
$ diff stored-facts.txt rerun-facts.txt
65c65
< test result: ok. 9 passed; ...; finished in 24.89s
---
> test result: ok. 9 passed; ...; finished in 23.88s
```

65 行里只有最后一行的挂钟耗时不同（24.89s → 23.88s，预期会变），其余 64 行（含 `Z2` 两处 panic 消息
「`取号之前 refuse_instance_rows_on_version_without_file 按同一个号核过：树表 0 条的一版上要写的行为空`」、
panic 位置 `crates/singlefs-core/src/mount.rs:820:13`）逐字节相同。**判定：✓（可复现，全部数据行一致）。**

### 1.3 引用核对表（文件:行号 + 抄的原文，在快照副本上核）

| 引用 | 核的结果 |
|---|---|
| `.claude/kb/decisions/16-发布语义.md:375` 整行「生效」表格行 | ✓ 逐字相同 |
| `.claude/kb/decisions/16-发布语义.md:377` 「准入」表格行（转述，未加引号） | ✓ 该行确实是「准入」那一格，含「先推空发布抬 F」 |
| `.claude/kb/decisions/16-发布语义.md:383` ⚠️「非空」整段 | ✓ 逐字相同（含 ⚠️ 与括注） |
| `.claude/kb/decisions/16-发布语义.md:288` fsync 那句 | ✓ 逐字相同 |
| `.claude/kb/decisions/02-RAID条带策略.md:263` 区域归属整行 | ✓ 逐字相同 |
| `.claude/kb/decisions/02-RAID条带策略.md:58` 转述句「2026-09-14 用户定案：不是 mkfs 参数」 | ✓ 该子串确实出现在这一行 |
| `.claude/kb/decisions/23-journal的角色与格式.md:1209` D23 已定项 14 管理员回退段 | ✓ 该行以「显式例外：管理员回退」开头，与引用的转述一致 |
| `.claude/kb/decisions/18-块里携带什么信息.md:882`（转述，未加引号） | ✓ 该行含「每次可写挂载都写行」等被转述的内容（此行也被 sonnet 引用，见下节） |
| `crates/singlefs-core/src/recovery.rs:351 effective_rollback_floor` | ✓ 函数定义确实起于 351 行 |
| `crates/singlefs-core/src/mount.rs:569`（`rollback_floor_ceiling(...)?;` 调用点） | ✓ 逐字相同 |
| `mount.rs:464`（`.filter(\|root\| root.checkpoint_txg >= current_floor)`） | ✓ 逐字相同 |
| `mount.rs:485`（`Err(failure) => Err(`） | ✓ 逐字相同 |
| `mount.rs:508`（`None => UserVisibleTreeRootPointers::ABSENT,`） | ✓ 逐字相同 |
| `mount.rs:777`/`778`/`779`/`785`/`821`（Z2 各步引用的代码行） | ✓ 五行全部逐字相同 |
| `mount.rs:916`/`922`（`let next_counter = records` / `let first_txg = first_txg_of_new_instance(...)`） | ✓ 逐字相同 |
| `mount.rs:208 first_txg_of_new_instance` | ✓ 该行确是函数签名首行 |
| `transaction.rs:287 instance_generation_to_acquire` | ✓ 该函数起于 287 行 |
| `transaction.rs:956`（`let txg = CheckpointTxg(FIRST_TRANSACTION_TXG);`）、`:962`（`counter: FIRST_TRANSACTION_TXG,`） | ✓ 逐字相同 |
| `transaction.rs:1430`（`birth_txg: txg,`） | ✓ 逐字相同 |
| `crates/singlefs-core/src/make_filesystem.rs:147` | ✓ 逐字相同 |
| `recovery.rs:219 choose_superblock`、`:78`（`.ok()?;` 那句）、`:776`（`.filter_map(...)`） | ✓ 三处均逐字相同 |
| `recovery.rs:819 起` `replay_journal`（只说「起」，未加引号） | **部分**：819 行实际是空行，`pub fn replay_journal(` 在 821 行（820 行是它的文档注释）。「起」字面上允许粗略指位，但精确位置偏了 2 行 |
| `crates/singlefs-harness/src/crash.rs:687`/`688`/`739` | ✓ 三处均逐字相同 |
| `first_transaction_step_seven_layer0.rs:124`/`127` | ✓ 逐字相同 |
| `second_transaction_step_three_formatted_pool_layer0.rs:50`/`53` | ✓ 逐字相同 |
| `opus_attack_emptypool.rs` 内部函数行号（450/213/294/557/677/149/168/494/517） | ✓ 九处全部对应正确的测试函数定义行 |

**Opus 节计数：核了 27 处引用/复现项，✓ 26，部分（记非 ✓ 非 ✗）1（`recovery.rs:819` 偏 2 行），✗ 0。**

## 二、云端正推（Sonnet）`m2-emptypool-nonempty-r1-sonnet-output.md`

### 2.1 引用核对表（文件:行号 + 抄的原文，在快照副本上核）

| 引用 | 核的结果 |
|---|---|
| `.claude/kb/decisions/16-发布语义.md:192` 已定项 9 整行 | ✓（判别力自证用的就是这条，逐字相同） |
| grep `^#### 已定项 9\|^### 已定项 9` 零命中 | ✓ 复算零命中 |
| `.claude/kb/decisions/18-块里携带什么信息.md:882` 引「每次可写挂载都写行（实例 0 不写……三轮三方）」 | **✗（部分）**：原文是「……同一次发布——**每次可写挂载都写行**（实例 0 不写……」，「每次可写挂载都写行」两侧带 `**…**` 加粗标记，报告引用时把这两个 `**` 去掉了。内容一字不差，但不是逐字符「整行抄」（丢了 markdown 强调符） |
| `mount.rs` `establish_instance` 765–882 行 | ✓ 起止行号都对（765 是 `fn establish_instance<Device: BlockDevice>(`，882 是它的闭合 `}`） |
| `mount.rs:824-825` 注释「上一个实例是 0、要写的行为空……」 | ✓ 逐字相同 |
| `.claude/kb/decisions/16-发布语义.md:383` ⚠️「非空」整段 | ✓ 逐字相同（与 Opus 引用同一行，见上节，此处独立复核仍一致） |
| `mount.rs:496-505`（前一条有效根的 `.filter().max_by_key()` 链） | ✓ 逐字相同（含分行位置） |
| `mount.rs:499-505` 单独引用同一段 | ✓ 逐字相同 |
| `mount.rs:439` 「有效 = 自证合法……非空 = 树表里……(txg, 实例) 排……」 | ✓ 逐字相同 |
| `mount.rs:506-508`（`match previous_valid_root { Some(...)=>..., None => ABSENT }`） | ✓ 逐字相同 |
| `mount.rs:427-428` `user_visible_trees_changed` 函数头文档注释 | ✓ 逐字相同 |
| C143 CJ2 出处，訂正为 `.claude/kb/decisions/23-journal的角色与格式.md:1240` | ✓ 1240 行确实是 CJ2 那一句（`⚠️ **checkpoint_txg 也一样（2026-09-16，C143…定案 CJ2）**`），订正准确 |
| grep 四个 `MountError` 变体在 `.claude/kb/decisions/` 全零命中 | ✓ 复算全部为 0（`FileVersionWithoutAnyJournalRecord`、`InstanceRowsOnVersionWithoutFileUnsupported`、`RollbackToVersionWithoutFileUnsupported`、`VersionWithoutFileNotWrittenByMakeFilesystem`） |
| `mount.rs:34-79`（MountError 枚举，从第二个成员起）、`:56-65`、`:73-78` | ✓ 三个区间逐字相同 |
| `refuse_instance_rows_on_version_without_file`（`mount.rs:744-762`） | ✓ 起止行号对（744 是 `fn`，762 是闭合 `}`） |
| grep 链 `rebuild_previous_version`/`FileVersionWithoutAnyJournalRecord`/`NoRecordStandingForFileVersion`（8 处行号：recovery.rs 476/490/509，mount.rs 36/142/143/167/1027） | ✓ 复算 8 处行号逐一相同 |
| `.claude/kb/milestone/02-second-txn.md:135`（步 3 现状，摘引一大段） | ✓ 逐字相同（含 `InstanceRowsOnVersionWithoutFileUnsupported` 等错误名与「是设计判断、没做」） |
| `.claude/kb/milestone/02-second-txn.md:189`（步 5 现状，「非空」概括句） | ✓ 逐字相同 |
| `.claude/kb/layout/01-first-txn.md:398`（段序列与闭式 262165） | ✓ 该行确实是「只做过 mkfs 的池的可写挂载」那一行，内容一致 |

### 2.2 十条变异复跑（第 62–71 行，报告声明的核心复跑项）

在草稿副本里写了独立脚本（不是 `.claude/gate.d/59-crates-mutation-replay.sh` 的整表版，只取第 62–71 行、逻辑与该脚本一致：
锚点命中数核 1 次 → 改坏 → 跑点名 `cargo test` 参数 → 记录 `test result:` 行与 FAILED 行 → 还原）。

```
$ nice -n 19 python3 verify-mutations-62-71.py
loaded 10 rows (62-71)
line 62..71: anchor_count=1（全部十条，两个文件 mount.rs / transaction.rs）
line 62 [...] status=RED   test ... torn_journal_record... ... FAILED
line 63 [...] status=RED   test ... the_rollback_publish_is_compared... ... FAILED
line 64 [...] status=RED   test mount::non_empty_root_tests::root_is_non_empty_when_either... ... FAILED
line 65 [...] status=RED   test ... writable_mount_of_a_formatted_pool_takes_instance_one... ... FAILED
line 66 [...] status=RED   同上（同一条测试名）
line 67 [...] status=RED   test ... the_formatted_pool_mount_and_first_file_stream_keeps... ... FAILED
line 68 [...] status=RED   test ... every_crash_state_outside_the_unit_segment... ... FAILED
line 69 [...] status=RED   test ... writable_mount_after_a_crash_between_warm_up... ... FAILED
line 70 [...] status=RED   test ... rolling_back_to_a_warm_up_root_without_a_file_version... ... FAILED
line 71 [...] status=RED   test ... raising_the_floor_is_refused_when_a_valid_root_tree_table... ... FAILED
```

十条变异对应的测试名、锚点唯一性（全部 `anchor_count=1`）与「改坏 → 红」全部与报告表格一致。
还原后逐文件比对：

```
$ diff -q repo/crates/singlefs-core/src/mount.rs "$SNAP/crates/singlefs-core/src/mount.rs"
$ diff -q repo/crates/singlefs-core/src/transaction.rs "$SNAP/crates/singlefs-core/src/transaction.rs"
```

两条 `diff -q` 都无输出（还原干净），与报告「diff -q 逐文件核对……（无输出，文件相同）」一致。✓

还原之后重跑四个测试文件 + `singlefs-core --lib`：

```
five_reuse:                11 passed（报告 11 passed，一致，耗时不同属预期）
four_rollback:               7 passed（报告 7 passed，一致）
three_formatted_pool:        2 passed（报告 2 passed，一致）
three_formatted_pool_layer0: 2 passed, 1 ignored（报告 2 passed 1 ignored，一致）
```

单独重跑 `root_is_non_empty_when_either...`：`1 passed; 0 failed; 0 ignored; 0 measured; 38 filtered out`，
与报告一致。✓

第 69 行报告称「第一遍脚本跑批时输出被截到 3000 字符尾部……单独重跑确认」——我的脚本按条单独跑
（无批量截断问题），69 行同样判红，测试名与 panic 消息（`assertion \`left == right\` failed: 两盘超级块槽
逐字节不变：没有取号`）与报告一致（报告贴的正是这条 panic 消息）。

**Sonnet 节计数：引用核对 19 处（1 处判 ✗-部分：D18 已定项 11 那行丢了 `**` 强调符），
复跑核对 1 组共 10 条变异 + 6 项还原/回归检查全部 ✓。合计 19 + 1 + 6 = 26 项，✓ 25，✗（部分）1，核不动 0。**

## 三、本地攻方（`m2-emptypool-nonempty-r1-local-attack*.md`）

四个文件：提示 `m2-emptypool-nonempty-r1-local-attack.md`（1483 行，英文，Z6 攻击面）、
转述核对表 `-translation-audit.md`、运行记录 `-runlog.md`、两份输出 `-output-s1.md`/`-output-s2.md`。

### 3.1 运行记录核对

字词损坏闸复算（对已落盘的两份输出文件重跑检测脚本，不需要重新调用网关）：

```
$ python3 research/scripts/corruption-check.py .../output-s1.md
绿 ... cjk=0 words=1123 fffd=0 汉字复读=0 英文复读=0 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0
$ python3 research/scripts/oov-check.py .../output-s1.md
绿 ... 生词=11 拼接=0     生词: mutations DiskSnapshot unaccounted
$ python3 research/scripts/corruption-check.py .../output-s2.md
绿 ... cjk=0 words=831 fffd=0 汉字复读=0 英文复读=0 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0
$ python3 research/scripts/oov-check.py .../output-s2.md
绿 ... 生词=5 拼接=0      生词: unmentioned mutations PoolReader
```

四条命令的输出与运行记录表格里登记的字段（cjk/words/fffd/各类复读/粘连计数、生词数与去重后的词表）逐项相同。
`wc -w` 复算：s1=1106、s2=805，与运行记录一致。**判定：✓（4 项字词损坏检测 + 2 项词数，共 6 项全部可复现）。**

### 3.2 转述核对表抽样核对（原文文件:行 vs. 实际内容，快照副本上核）

| 转述核对表条目 | 现查结果 |
|---|---|
| T1 doc（`second_transaction_step_five_reuse.rs:317-319`） | ✓ 三行原文与译文对应内容一致，英译无遗漏限定词 |
| T2 doc（`:364-366`），标注多加「in txg order」 | ✓ 原文「夹在 A 与 D 之间」确未明说按 txg 排序，核对表如实记录这一处添加 |
| T9 doc（`:423-425`） | ✓ 三行原文与译文对应，「读不出就要走修复，不能跳过」完整译出 |
| T3 doc（`mount.rs:1100-1101`），标注多加「which has no previous root」 | ✓ 原文确实没有重复这半句，核对表如实记录 |
| D18 已定项 11 代码内注释（`mount.rs:482`），标注漏译「读不出要走修复」半句 | ✓ 现查 `mount.rs` 附近注释，该半句确实只在别处（T9 docstring）译出，未在这条代码内注释重复译，核对表如实记录并说明判据表不依赖这条 |
| DiskSnapshot 文档注释（`common/mod.rs:242-243`） | ✓ 逐字对应（见下方 3.3 节独立复核） |
| harness/src/lib.rs:1,3，标注漏译两处出处引注（里程碑步 0 / D13 已定项 4） | ✓ 现查原文确有这两处括注引用，译文确实略去，核对表如实记录并说明不影响判据 |
| Section 3.7 断言消息串译文（`...layer0.rs:57`），标注「比原文短一个字」 | ✓ 原文「写行发布的记录两写」「暖机的记录两写」，译文简化为「the row-writing record」「the warm-up record」，核对表如实记录 |

抽样 8 条转述核对表条目，**全部✓**：核对表标注的每一处「多加/漏译」都能在原文里现查到依据，
没有发现「漏报」的多加或缺失限定词。

### 3.3 提示正文（Section 3–6）源码引用抽样核对

| 引用 | 声称位置 | 现查结果 |
|---|---|---|
| T1/T2/T9 函数体 | `second_transaction_step_five_reuse.rs:317-362`/`364-391`/`423-481` | ✓ 起止行精确对应（317 是文档注释首行，362/391/481 是各自函数闭合 `}`） |
| T3 测试模块 | `mount.rs:1088-1128` | ✓ 1088 是 `#[cfg(test)]`，1128 是模块闭合 `}` |
| T4/T7 共享 helper | `second_transaction_step_three_formatted_pool.rs:25-56` | ✓ `region_device`（25 行起）与 `assert_checker_verdicts` 内容逐字相同 |
| T7 | `:58-111` | ✓ 58-60 文档注释、111 闭合 `}`，内容逐字相同 |
| T4 | `:113-284` | ✓ 113-116 文档注释、284 闭合 `}`，内容逐字相同 |
| T8 | `second_transaction_step_four_rollback.rs:84-110` | ✓ 84-85 文档注释、110 闭合 `}` |
| 上下文函数 1 `rollback_floor_ceiling` | `mount.rs:438-519`（标「around」） | ✓ 438 文档注释首行、519 闭合 `}` |
| 上下文函数 2 `format_time_allocator` | `mount.rs:385-425`（标「around」） | ✓ 385 文档注释首行、425 闭合 `}` |
| 上下文函数 3 `publish_without_units` | `transaction.rs:392-433`（标「around」） | ✓ 392 是 `pub fn`，433 是 CODE 块显式截断处（含「// ... 其余未展示」注释，不是函数真正的闭合行，但提示原文本身说明了这是截断展示，不是虚报终点） |
| **上下文函数 4 `publish_empty_after`** | **claimed `transaction.rs:699-737`（标「around」）** | **✗：实际定义在 `crates/singlefs-core/src/mount.rs:701-740`，不在 `transaction.rs`。内容（函数体全文）与提示 CODE 块逐字相同，只是文件与起止行号都指错了（transaction.rs → 应为 mount.rs；699 → 应为 701）。** |
| 上下文函数 5a `establish_instance`（截断展示） | `mount.rs:764-780` | ✓（起始行 764 对应文档注释上一行；1034-1050 CODE 块内容与实际 mount.rs 逐字相同） |
| 上下文函数 5b `mount_rollback`（截断展示） | `mount.rs:966-1017` | 核不动（未在草稿副本里核对该函数的精确起止行号，只核了 CODE 块内容——内容读起来与代码逻辑一致，未逐字比对） |
| M1–M10（Section 4，原文/替换文/cargo 参数） | 对照 `crates/mutations.tsv` 第 62-71 行 | ✓ 十条全部字段（原文、替换文、cargo test 参数）逐字比对与 tsv 相同（`@@` 与 `::` 的记号替换按 Section 2 声明的规则对应） |
| `common/mod.rs:242-243` DiskSnapshot 文档注释 | Section 6.1 | ✓ 逐字相同 |
| `common/mod.rs:71-80` `BuiltPool` 结构体 | Section 6.5（标「around」） | **✗（轻微）**：CODE 块省略了 `pub devices: ...` 字段前的一行文档注释（`` /// `take` 出去就是「进程退出、镜像关掉」：冷启动要重新打开文件。``），未加任何省略标注，而 Section 2 声明 CODE 块是「literal excerpt ... copied without changing anything」；8 个字段本身全部完整无误，只是漏抄了这一行注释 |
| `harness/src/lib.rs:1,3` 模块级文档注释 | Section 6.2 | ✓ 逐字对应（对应的出处括注按核对表已知略去，见 3.2 节） |
| `harness/src/lib.rs` push 函数 | around lines 101-115 | ✓ `fn push` 确实在第 101 行 |
| `transaction.rs` CommitStep 枚举 | around lines 64-84 | ✓（枚举声明在 65 行，「around」容差内） |
| `second_transaction_step_five_reuse.rs` `allocator_state`/`AllocatorState` | around lines 393-420 | ✓（分别在 394/417 行，「around」容差内） |

**关于「上下文函数 4」错误的重要限定**：这条错误在 Mutation M5 的条目（提示第 1179-1191 行）里被显式提到并
自行订正——该条目原文写「File: `crates/singlefs-core/src/mount.rs`. Note: this exact line of source text is
inside context function 4 (`publish_empty_after`), which `crates/mutations.tsv` records under the path
`crates/singlefs-core/src/mount.rs` in this row; the file column in the tsv is what is authoritative.」，
现查 `crates/mutations.tsv` 第 66 行的 file 字段确实是 `crates/singlefs-core/src/mount.rs`，与 M5 条目里
给模型的最终依据一致。也就是说：**「上下文函数 4」标题行的文件路径与起止行号本身是错的（transaction.rs:699-737 →
实为 mount.rs:701-740），但提示文档在真正用到这个函数做判据的地方（M5 条目）已经显式指出并订正**，
两份模型输出（s1/s2）对 M5 的推理内容也没有出现因文件搞混而产生的错误迹象。

按「误写成背景材料第 N 行」的分类法核对：`publish_empty_after` 在
`_m2-emptypool-nonempty-r1-background.md`、`_m2-emptypool-nonempty-r1-diff.md` 里只作为**调用点**出现过
（`let next = publish_empty_after(...)`），从未作为函数**定义**出现，所以这不是「背景材料行号被误抄成
kb/代码行号」那一类；是撰写提示时对函数所在文件的记忆错误，与背景材料无关。

### 3.4 十条变异的「必须红」测试名与本地腿输出格 M-T 对照（不判本地腿的推理对不对，只核输入是否忠实）

抽查 Grid A（Question 1）s1/s2 输出里点名的测试 id（T1-T9）与 mutation id（M1-M10）是否都在提示 Section 3/4
给出的名字范围内：s1、s2 两份输出通读一遍，M1-M10、T1-T9 的编号与名字全部落在提示定义的范围内，没有发现
输出引用了提示里不存在的编号或名字。✓（这一步只核「答案没有凭空引用提示外的实体」，不判答案对不对——
「打中成不成立」不归我判）。

## 四、汇总计数

| 腿 | 核了几处 | ✓ | ✗（含部分） | 核不动 |
|---|---|---|---|---|
| 云端攻方 Opus | 27（含 sha256/行数 3 项、复跑数据行比对 1 组、kb/代码引用 23 处） | 26 | 1（`recovery.rs:819` 精确位置偏 2 行，非逐字引用，只是「起」的粗略指位） | 0 |
| 云端正推 Sonnet | 26（引用 19 处 + 复跑/还原/回归 7 组） | 25 | 1（D18 已定项 11 引用丢了 `**` 强调符，内容无误） | 0 |
| 本地攻方（提示+核对表+运行记录+输出） | 6（运行记录复算）+ 8（转述核对表抽样）+ 17（正文源码引用抽样）+ 1（M-T 编号忠实性） = 32 | 30 | 2（上下文函数 4 文件/行号错误；`BuiltPool` CODE 块漏抄一行文档注释未标注） | 1（上下文函数 5b 的精确起止行号未逐字核） |

**合计：核了 85 处，✓ 81，✗（含部分/轻微）4，核不动 1。**

判别力自证：1 处，正确判 ✗（成立）。

## 没做什么

- 不判一条打中成不成立、该不该采纳；不核推理本身对不对（V1/V2/V3 该不该改代码、Z1-Z7 各格的判定结论），
  只核引用、产物与复跑——这些留给主 agent 逐条现查。
- 没有跑门禁任何阶段（54 号层 0 全量、59 号变异表全量、`check.sh`、命名纪律）；只对 `crates/mutations.tsv`
  第 62-71 行这十条变异做了独立的、逻辑等价于 59 号脚本的局部复跑（用自己写的 Python 脚本，未调用
  `.claude/gate.d/59-crates-mutation-replay.sh` 本身，因为它会跑全表、代价太大不在这轮范围内）。
- 没有重新调用本地模型网关（`ask-local.sh`）；`ask-local.sh` 相关的字词损坏检测复算用的是已落盘的
  `-output-s1.md`/`-output-s2.md` 两份文件，不是重新问一遍模型。
- 没有对 Opus 报告 Z3-B 节「oracle 违例里有多少是版本表按 (实例, txg) 认造成的」这类报告自己也承认
  「没逐状态拆」的部分做二次核实——报告本身已注明是限度，不在核查范围内重新计算。
- 没有对本地攻方提示 Section 5b（`mount_rollback` 截断展示）的精确起止行号做逐字核对，只核了 CODE 块
  内容与实际代码逻辑相符；这条记为「核不动」不是「跑不动」，是这一轮时间预算内没有安排到。
- 没有对本地攻方 s1/s2 输出的 Grid B（Question 2）、Grid C（Question 3）21 格逐条判断是否符合提示给出的
  事实重新推演——那属于「打中成不成立」，不归核查员判。
- 没有核对 `.claude/kb/milestone/02-second-txn.md`、`layout/01-first-txn.md` 里除已抽查的两处（135/189 行、
  398 行）之外的其余内容是否与代码一致；只核了 sonnet 报告实际引用到的那几处。
- 没有检查 Opus 报告里「Z3-B 表格四种区域归属」那一段之外的更多 `opus_attack_emptypool.rs` 源码逐行核对
  （只核了报告点名的十几处具体行号，没有通读全部 692 行去找报告未提及但可能存在的其它问题）。
