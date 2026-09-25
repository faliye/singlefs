# 核查员报告：m2-final-code-r2

你交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

## 干到一半收到的两条主 agent 消息

- 第一条：`.claude/agent-common.md`「长活可以等」检出写法从两种改成三种（加「起看门狗的错误写法」）。与我无关：我核的是冻结副本 `defs/.claude/agent-common.md`（仍是旧文本，两种写法），不核主工作区那一份；未改动任何步骤。
- 第二条：`.claude/agent-common.md` 第 57 行再加第四种检出（`run_in_background` 命令里裸 `&` 收尾、同条没有 `wait`）。同样与我无关，且我手上此刻没有正在跑、用裸 `&` 起的后台任务（早前一次误用 `&` 已在下方「判别力自证之外的一个操作纠正」里自查处理，此后全部长活改用 `run_in_background` 参数）。两条消息都不改变本报告的任何一处核对结论。

## 判别力自证

挑 `mount.rs:1703`（云端攻方 Z9 引文：`entry.rollback_target_instance <= root.instance`），在草稿目录副本里把行号人为加 1 核成 `mount.rs:1704`：

```
腿的抄文（应在 1703 行原样出现）：entry.rollback_target_instance <= root.instance
自证核的行号（1704）实际内容：                    && root.instance < entry.new_instance
结果：✗ —— 1704 行与腿抄的原文不符；原文实际在 1703 行。
```

判别力自证通过（判得出 ✗）。

## 操作记录：一次误用裸 `&` 并已纠正

核 Opus 复跑时第一次用 `bash model-copy/rerun.sh ... &` 起后台、未跟 `wait`，属违规写法。发现后立即列出整棵子进程（`ps -o pid,ppid,args --ppid`）、从最底层起用 `.claude/singlefs-ai-sop/scripts/proc.py stop <pid>` 逐层停净，确认三层 pid 均已不在，再改用 Bash 工具的 `run_in_background` 参数重跑。此事不影响后续任何一次核对（重跑是从干净状态开始的新一次）。

## 快照自证

三份开工快照哈希本次会话重新核过一遍，与主 agent 06:10 JST 的核验一致：

```
crates-src-sha256.txt：sha256sum -c --quiet 退出码 0
defs-sha256.txt：sha256sum -c --quiet 退出码 0
kb-sha256.txt（在 kb-snapshot 目录内跑）：sha256sum -c --quiet 退出码 0
```

腿的模型目录（`research/prompts/m2-final-code-r2-opus-model/SHA256SUMS`，122 个文件）`sha256sum -c --quiet` 退出码 0。

## 总计数（三条腿合计）

- 核了 95 处（Opus 31、Sonnet 34、本地攻方 30，按表格行/算术格逐项计，含算术格与常量引文）。
- ✓ 87 处；✗ 4 处；分不清（无快照 / 无法排除主工作区在腿交回之后被改过）4 处，单列、不计入 ✓ 也不计入 ✗。
- ✗ 与「分不清」的清单、每一处的实际位置，见下面各腿的表。

## 云端攻方（Opus）：`m2-final-code-r2-opus-output.md`

复跑方式：模型目录拷进 `/tmp/claude-1000/m2-final-code-r2-verifier/opus-rerun/model-copy/`（先核 SHA256SUMS 全绿），`rerun.sh` 里唯一的线程上限 `8` 改成派发给我的 `6`（草稿目录内的副本，只改这一处），`target` 目录另设在自己的草稿目录，全程 `nice -n 19`。先用 `bash rerun.sh` 整跑一遍（含 Z9-A/Z9-B/Z8/Z12 编译），随后为了避开 `--nocapture` 并行输出交错的问题，把 `SINGLEFS_R2_OPUS_WITNESS_RULE` 三个变体各自加 `--test-threads=1` 单独串行重跑一遍，专门核「改法表」的 12 格。

| 引用 | 核的结果 | 命令 |
|---|---|---|
| `decisions/23-journal的角色与格式.md:373`「回退见证」整段 | ✓ 逐字相同 | `sed -n '373p' kb-snapshot/.claude/kb/decisions/23-*.md` |
| 同文件 `:387`①删除规则、`:388`②写满 | ✓ 逐字相同 | 同上，`sed -n '387,388p'` |
| `mount.rs:1703`（`entry.rollback_target_instance <= root.instance`） | ✓（判别力自证样本） | `sed -n '1703p'` |
| `mount.rs:1717`（「有槽持续读不出时条目删不掉、表就写得满」） | ✓ 逐字相同 | `sed -n '1715,1719p'` |
| `mount.rs:1733`、`:1862`（描述性转述，非逐字引） | ✓ 与代码行为一致 | `sed -n '1728,1736p'`、`sed -n '1858,1865p'` |
| `rollback_witness.rs:54`（容量函数签名） | ✓ | `sed -n '54p'` |
| `singlefs-format/src/lib.rs:227`（S=8，mkfs 常量） | ✓ | `grep -n 'AT_MAKE_FILESYSTEM: u64 = 8'` 命中行 227 |
| R×S−1=23 算术 | ✓ 3×8−1=23 | 算术核 |
| `mount.rs:1766`（挂载写入口用 `PoolWriter::new`） | ✓ | `sed -n '1762,1768p'` |
| `mount.rs:1968-1971`（取号预演断言，含「……」显式省略号） | ✓ 与实际断言文本一致 | `sed -n '1966,1972p'` |
| `mount.rs:921`（抬 F 先查冻结） | ✓ | `sed -n '917,923p'` |
| `recovery.rs:1176`（回退行查找） | ✓ | `sed -n '1172,1178p'` |
| `recovery.rs:1783`（末条标志之后同 txg 记录） | ✓ | `sed -n '1779,1785p'` |
| `transaction.rs:194`、`:806`、`:863` | ✓ 三处均对应 | `sed -n` 逐段核 |
| `logs/z9_rule_original_z9_a1.log`「末两行」引的 k=23/k=24 两行 | 内容 ✓ 逐字存在（第 26、27 行，全文 31 行）；位置 ✗ 不是该文件真正的最后两行（真末两行是 cargo 的 `test result: ok...` 与空行） | `wc -l`、`tail -2`、`grep -n '^k='` |
| `logs/z9_a_control.log` 三行 `mounts=40 max_witness_entries_before=Some(9) stop=None` | ✓ 逐字存在，且独立重跑逐字复现 | `grep '^==='`；复跑见下 |
| `logs/z9_b.log` 8 行 `m=0..3` | ✓ 逐字存在，独立重跑字节级复现 | `grep '^m='`；复跑见下 |
| `logs/z9_b_repair.log` 4 行 `m=0..3` | ✓ 逐字存在，独立重跑字节级复现 | 同上 |
| `logs/z8_z8_one_unit.log`、`z8_z8_two_units.log` 摘要 | ✓ 全部计数行独立重跑逐字复现（含 `mismatches=0 []`） | 见下 |

### 复跑结果（重点核 1、2）

- `bash model-copy/rerun.sh <草稿>/work` 后台跑完，退出码 0（任务通知 `completed`）。Z9-A 原始规则：24 次回退，第 24 次 `REFUSED Recovery(RollbackWitnessTableFullWhoseHandlingIsUndecided { entries: 24, capacity: 23 })`，A1、A2 两条历史都在——与报告逐字一致。
- `z9_a_control` 三种取法：`mounts=40 max_witness_entries_before=Some(9) stop=None`，三行都复现。
- Z9-B 与补见证两组 `m=0..3` 共 16 行，与 `logs/z9_b.log`、`logs/z9_b_repair.log` 逐字节相同（用 `diff` 级比对，非目测）。
- Z8 one_unit / two_units 两组全部计数行（`no-resend recover`、`resend second_failure=...`、`resend-prefix recover A/B(root 3)/B(root 4)`、`mismatches=0 []`）与两份日志逐字相同。
- Z12：`cargo test -q -p singlefs-harness --test r2_opus_z12 --no-run` 编译成功，产出测试二进制（未跑扫描脚本，按「重点核」范围不要求）。

### 改法表 12 格（重点核 2）：全部独立复核，用 `--test-threads=1` 逐条串行重放，避免并行 `--nocapture` 输出交错

| 删除规则 | A1 | A2 | A3 |
|---|---|---|---|
| 冻结副本的 ① | ✓ 第24次写满(entries=24,capacity=23) | ✓ 同上 | ✓（已知，实二一） |
| dominance（① ∧ 去掉被罩住的） | ✓ 7条封顶(k=30时witness=7)，30次不满(stop:None) | ✓ 第24次写满 | ✓ 13条(WarmUp拒于witness=13) |
| abandons（① ∧ 只留还抛弃着的） | ✓ 第24次写满 | ✓ 1条封顶(stop:None) | ✓ 12条(WarmUp拒于witness=12) |
| both | ✓ 2条封顶(stop:None) | ✓ 1条封顶(stop:None) | ✓ 12条(WarmUp拒于witness=12) |

12 格全部核对：与报告表格逐格一致。（发现并记录一个陷阱：`cargo test --nocapture` 默认并行跑，前一测试的 `.`（通过标记）无换行直接接在下一测试的 `===` 标题前，粗心用 `grep '^==='` 会漏掉/错配标题行；本核查改用不带锚点的 `grep '==='` 并按 `target=` 字段模式识别，且额外加 `--test-threads=1` 消除交错，才把 12 格逐一分辨清楚。）

### Opus 腿计数

核了 31 处（引文表 19 行 + 改法表 12 格）；✓ 30 处；✗ 1 处（`logs/z9_rule_original_z9_a1.log`「末两行」的位置描述，内容本身逐字存在，但不在文件真正的最后两行，计为 ✗）；分不清 0 处。

---

## 云端正推（Sonnet）：`m2-final-code-r2-sonnet-output.md`

`Z7` 与部分 `Z11` 引用的文件在给定快照内（`tree/crates/`、`kb-snapshot/`、`defs/`），照快照核；`Z10` 的 gate 脚本与 `Z11` 的三个 hook（`heavy-test-guard.sh`、`continuation-guard.sh`、`bash-command-detector.sh`）不在任何一份给定快照里（`defs/` 目录只有 8 份文件，逐一 `find` 过，没有这四份），而 `git status` 证实 `.claude/gate.d/55-qemu-first-transaction.sh`、`bash-command-detector.sh`、`continuation-guard.sh` 均为已修改、`heavy-test-guard.sh` 为未跟踪新文件——主工作区这几份确认处于变动状态。按派发提示「没给快照的轮…腿引的行号与主树对不上时记『分不清』」的同一原则，这四份文件的核对结果一律记「分不清」，不论现读是否吻合，都不计入 ✓/✗。

| 引用 | 核的结果 | 命令 |
|---|---|---|
| `code_two_tree.rs:433-437`（`split_in_the_middle`） | ✓ | `sed -n '430,440p'` |
| `code_two_tree.rs:412,419`（`route_for_insertion`） | ✓ | `sed -n '408,421p'` |
| `code_two_tree.rs:546,555,574`（删除/降高） | ✓ | 逐段 `sed -n` |
| `code_two_tree.rs:195`（`height()`） | ✓ | `sed -n '193,197p'` |
| `code_two_tree.rs:118`（内部条目宽公式） | ✓ | `sed -n '116,120p'` |
| `code_two_tree.rs:622-624`（先删后插注释） | ✓ | `sed -n '620,626p'` |
| `singlefs-format/src/lib.rs:131,134`（108/113） | ✓ | `sed -n '129,135p'` |
| `walk.rs:605-611`（checker 三样文档注释） | ✓ 逐字相同，确认只有三样、没有「树高」 | `sed -n '603,617p'` |
| `walk.rs:616`（`walk_code_two_subtree`） | ✓ | 同上 |
| `walk.rs:2509`（`judge_rollback_floor_raises_against_their_ceilings`） | ✓ | `sed -n '2505,2533p'` |
| `walk.rs:2531`（`if raising_root.rollback_floor <= floor_before_the_raise`） | ✓ 逐字相同 | 同上 |
| `mounted_read.rs:354-367`（`open_pool_for_read` 整棵读入映射） | ✓ 含函数名核对（`grep -n '^pub fn open_pool_for_read'` 命中 344，361 前最近一个 fn） | `sed -n '352,368p'` |
| `grep -rin 'height' crates/singlefs-checker/` 零命中 | ✓ 重跑退出码 1（零命中） | 现查 |
| `grep -n 'height\|层级' root_record.rs` 零命中 | ✓ 重跑退出码 1 | 现查 |
| `second_transaction_supplement_two_tree_split.rs:702`（测试函数名）、`:712-750`（7 个用例数组开头） | ✓ | `sed -n '700,714p'` |
| `mutations.tsv:577-581`（树分裂 checker 5 行，覆盖层级/两条分隔key不等式/区间/I-1.10，无树高） | ✓ 逐字相同 | `sed -n '577,581p'` |
| `08-核心索引结构.md:321-326`（已定项11整段） | ✓ 逐字相同 | `sed -n '321,328p'` |
| `08-核心索引结构.md:327`（内部条目=108/113） | ✓ 引的是该行前半句，逐字相同；后半句（分配记录树/extent 树的另一件事）未引，与 Z7 主张无关，不算摘句变义 | 同上 |
| `19-块指针的结构与宽度预算.md:103`（挂载态整棵读映射，含「……」显式省略） | ✓ | `sed -n '100,106p'` |
| `first_transaction_on_device.rs:2407-2413`（raise-rollback-floor 场景注释） | ✓ | `sed -n '2405,2414p'` |
| `first_transaction_on_device.rs:2451-2456`（`requested_floor`/`rollback_floor` 内存断言） | ✓ | `sed -n '2449,2457p'` |
| `first_transaction_on_device.rs:2593`（冷重开择 (2,11)） | ✓ | `sed -n '2591,2595p'` |
| `first_transaction_on_device.rs:1547`（`recover_cold` 字段表无 `rollback_floor`） | ✓ | `sed -n '1544,1550p'` |
| `first_transaction_on_device.rs:1153-1155`（`raise_rollback_floor` 行取内存值） | ✓ | `sed -n '1151,1157p'` |
| `first_transaction_device_log_check.rs:792-796`（比对范围注释与函数名） | ✓ | `sed -n '790,797p'` |
| 「推翻条件」里的 `grep -n 'rollback_floor' first_transaction_device_log_check.rs` 零命中 | **✗ 命中数为 6，不是零** | `grep -n 'rollback_floor' first_transaction_device_log_check.rs`；见下方原样输出 |
| `.claude/rules/implementation-workflow.md:14`（三方打表一行） | ✓（在 `defs/` 快照内） | `sed -n '12,16p' defs/.claude/rules/implementation-workflow.md` |

### ✗：`grep 'rollback_floor'` 推翻条件的原样输出（与腿声称的「零命中」矛盾）

```
33:    publish_the_third_version, raise_the_rollback_floor_to_its_ceiling, OnDeviceRunMode,
70:            ProgramWindow::RaiseRollbackFloor => "raise_rollback_floor",
178:            raise_the_rollback_floor_to_its_ceiling(
595:                    "mkfs,instance_acquisition,warm_up,first_transaction,second_transaction,reopen_and_writable_mount,third_transaction,fourth_transaction,raise_rollback_floor"
793:    /// 最后一个写的内容变了）两块盘都判红、红在被改的那一段（`fourth_transaction` / `raise_rollback_floor`）；`second-instance` 的盘
796:    fn raise_rollback_floor_mode_compares_the_raise_window_and_is_red_there_when_one_step_changed()
```

6 处命中，均是 `raise_rollback_floor`/`RaiseRollbackFloor` 这个模式名/函数名的字面出现，没有一处是在断言或解析 `root.rollback_floor` 这个字段的值。腿在 Z10 问题 1「推翻条件」一段引用的具体命令与它自称的输出（零命中）对不上——这是一处可复核的事实性错误，但没有推翻它紧邻的实质性论断（这 6 处都不是对字段值的独立解析/断言，「设备侧比对不独立核 F/根语义」这句本身仍待主 agent 现查判断，不由这条命令的对错决定）。

### 分不清（无快照，主工作区已确认在变动中）：4 处

| 引用 | 现读（供参考，不计入✓/✗） | 为什么分不清 |
|---|---|---|
| `.claude/gate.d/55-qemu-first-transaction.sh:3-4,84,212` | 现读逐字与腿引一致（`sed -n` 核对） | 文件被 git 标记为已修改（`git diff --stat` 显示 64 插入/37 删除），且该文件本身就是这一轮 raise-rollback-floor 特性新增的载体，无法证明现读版本与腿核的是同一次快照 |
| `.claude/hooks/heavy-test-guard.sh:26-33`（八类）、`:34-38`（放行表） | 现读范围有偏差：「整轮门禁」「E152 装置」两类实际在 34、35 行，不在引用的 26-33 区间；「放行表」标题「谁、带什么才放行」实际在 37 行，不在引用的 34-38 起点 | 该文件 `git status` 显示为全新未跟踪文件（`??`），完全没有版本基线可比对 |
| `.claude/hooks/continuation-guard.sh:10-16` | 现读逐字与腿转述一致 | 文件被 git 标记为已修改 |
| `.claude/hooks/bash-command-detector.sh:2,30` | 现读逐字与腿引一致（含「三种拒绝」「起看门狗的错误写法」） | 文件被 git 标记为已修改；且主 agent 期间的第一条消息证实 `agent-common.md`（不是这份 hook 本身）刚发生了与这一处相关的同步编辑，说明这一片正处在别的会话正在改动的状态 |

### ✗：`crash-verifier.md:19`、`gate-triage.md:19`（在 `defs/` 快照内，非「无快照」情形）

- `crash-verifier.md:19` 现读是「- 这一次是提交时跑还是用户要求时跑：决定命令带 `SINGLEFS_HEAVY_TESTS=commit` 还是 `=user-request`。」，腿引的那句「只在提交时（或主 agent 转达用户要求时）跑你那几道，不跑 `gate.sh` 整轮与全量 `cargo test`」**实际在第 25 行**（已用 `cat -n` 核对全文 42 行）。
- `gate-triage.md:19` 现读同样是「这一次是提交时跑还是用户要求时跑」那一句，腿引的「54、55、57、59 这几道重阶段……你不直接调它们，也不跑全量 `cargo test`」**实际在第 26 行**。
- 两处都不是「误写成背景材料行号」——`_m2-final-code-r2-background.md` 里全文搜这两句引文均零命中，说明腿没有引用背景材料的编号，是单纯的行号数错（且两份文件模板结构相近，第 19 行恰好都是同一句「这一次是提交时跑还是用户要求时跑」，像是把这句误当成了引用目标）。

### Sonnet 腿计数

核了 34 处；✓ 27 处；✗ 3 处（`grep rollback_floor` 零命中声称、`crash-verifier.md:19`、`gate-triage.md:19`）；分不清 4 处（gate.d/55、heavy-test-guard.sh、continuation-guard.sh、bash-command-detector.sh，均因无快照且主工作区确认在变动，单列不计入 ✓/✗）。

## 本地攻方：`m2-final-code-r2-local-attack-output-s1.md`、`-s2.md` 与译文核对表

本地腿的提示明令答复不写文件名与行号（只准引 `FACT 1`–`FACT 9` 与任务标签），核对行号的活落在译文核对表 `m2-final-code-r2-local-attack-translation-audit.md` 的「原文文件:行」列，逐条现查如下（对快照 `tree/crates/`、`kb-snapshot/`）。

| FACT / 常量 | 原文文件:行 | 核的结果 |
|---|---|---|
| FACT 1（`NODE_BYTES=16384`） | `singlefs-format/src/lib.rs:14-15` | ✓ 逐字相同 |
| FACT 2（头宽公式、159/169） | `lib.rs:49-50,68-72` | ✓ 逐字相同 |
| FACT 3（叶条目 34/55） | `lib.rs:92-93,127-128` | ✓ 逐字相同 |
| FACT 4（内部条目 108/113） | `lib.rs:130-134`；`decisions/08-核心索引结构.md:327` | ✓ 逐字相同（与 Sonnet 腿核的是同一处） |
| FACT 5（容量公式） | `unit.rs:120-131`；`code_two_tree.rs:94-95,103` | ✓ 逐字相同 |
| FACT 6（15 行=3+6D） | `recovery.rs:2369,2371`；`transaction.rs:5070-5071`；`lib.rs:136-137` | ✓ 逐字相同，含核对表登记的「首稿略去后半句」属实（`transaction.rs:5071` 确有「全部来自分配器…不扫盘」半句未引，登记为有意省略成立） |
| FACT 8（见证表 481/4096/R=3/S∈[4,16]） | `lib.rs:195-247,208-223` | ✓ 逐字相同 |
| FACT 9（journal 4096/311/56） | `lib.rs:161-176`；`assert_eq!(JOURNAL_NAMED_ENTRY_BYTES, 56, …)` 于 `lib.rs:351` | ✓ 逐字相同，56 = `LOC_ENTRY(14)×2+1+8+8+10+1` 算术核对成立 |

### 十格算术复核（s1、s2 两份样本各自独立核）

| 项 | 应得值（据上表常量现算） | s1 | s2 |
|---|---|---|---|
| W.max_s | 3×16−1=47 | ✓ 47 | ✓ 47 |
| W.table_bytes | 1+47×16=753 | ✓ 753 | ✓ 753 |
| W.fits_in_slot | 481+753=1234≤4096，是 | ✓ | ✓ |
| AC.leaf_cap | ⌊16225/34⌋=477 | ✓ 477 | ✓ 477 |
| AC.internal_cap | ⌊16225/108⌋=150 | ✓ 150 | ✓ 150 |
| MP.leaf_cap | ⌊16215/55⌋=294 | ✓ 294 | ✓ 294 |
| MP.internal_cap | ⌊16215/113⌋=143 | ✓ 143 | ✓ 143 |
| ROWS.height_at_15 | h=1，477≥15 | ✓ 1 | ✓ 1 |
| ROWS.min_devices_to_split | D=80（3+6×80=483>477，D=79 时 477 未超） | ✓ 80 | ✓ 80 |
| REC.named_cap | ⌊3785/56⌋=67 | ✓ 67 | ✓ 67 |

10 格 × 2 份样本 = 20 处全部 ✓，与常量表现算结果逐格相等。

### 运行记录旁证

`wc -w` 现查：s1 = 350、s2 = 680，与 `m2-final-code-r2-local-attack-runlog.md` 样本表里登记的词数逐字相符（✓）。

### 本地攻方腿计数

核了 30 处（8 组 FACT 常量 + 10 格算术 × 2 份样本 20 处 + 词数旁证 2 处）；✓ 30 处；✗ 0 处；分不清 0 处。

## 没做什么

- 不判一条打中成不成立、该不该采纳；不核推理本身，只核引用、产物与复跑。「grep 'rollback_floor' 零命中」这一处 ✗ 只说明腿写的那句话与命令的实际输出对不上，不判 Z10 问题 1 的实质论断（设备侧比对不独立核 F/根语义）对不对——那句论断要不要站得住，由主 agent 再核。
- Z9 的 A1–A3、B、对照三组历史之外，Opus 自己在报告里承认「没做专门扫描」（删除规则是否会误删仍护着根的条目、各盘一新一旧撕裂）；这一层不属于我的复核范围（我核的是它已声称的产物与复跑，不替它补做新扫描）。
- Z12 两遍完整扫描（`run_z12_sweep.sh`、`run_z12_sweep2.sh`，96 份日志、共 19062 次挂载）没有重跑——「重点核」只点名 Z9-A/Z9-B/改法表，Z12 只核了它编译成功（`--no-run`）这一步；96 份摘要日志的内容逐字核对没有做。
- Sonnet 腿指名读「主工作区现行版」的 gate 脚本与三个 hook，因为这一轮没有给它们的快照，按「没给快照…记『分不清』」的同一原则处理，没有判它们 ✓ 或 ✗；这四处的现读内容与腿的转述实际都吻合（详见上文分不清表的「现读」列），但吻合不构成「同一份文件」的证据，仍按分不清记。
- 本地攻方腿本身在译文核对表「没做什么」一节点名的三处（FACT 7/TASK/FORMAT RULES 不是转述、未判本地模型答复方向、未核云端两条腿）不重复核——那是它自己声明的射程外，不属于三方核对表要管的范围。
- 未跑门禁、未编译整个工作区、未跑 `.claude/gate.d/` 任何阶段（不归核查员）。
- 未核对报告文件当前 sha256 与云端腿交回时自报的 sha256 是否一致——派发提示未给出交回时报的具体哈希值，无从比对；已核的是模型目录内部 `SHA256SUMS` 的自洽性（Opus 122 个文件全绿）。
- 未对 Sonnet 腿「没做什么」一节点名的几处未走读内容（`read_code_two_tree`/`flatten` 细节、`on_device_modes.rs` 的 second-instance 写序、`runner-dispatch-guard.sh`）做补充核对——那些是它自己声明的射程外。

