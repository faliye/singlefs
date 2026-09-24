# m2-wave3-code-r1 三方核查员报告

你交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

写于 2026-09-24（UTC 01:50 开工；JST 10:50）。草稿目录 `/tmp/claude-1000/m2-wave3-verifier/`。

输入：轮名 `m2-wave3-code-r1`；三条腿报告——云端攻方 `research/prompts/m2-wave3-code-r1-opus-output.md`（给定 sha256 `dc92dc659f4d74641ec3c393545a3692a593a33cd4de8d38a8662661d463bb55`，模型目录 `research/prompts/m2-wave3-code-r1-opus-model/`）、云端正推 `research/prompts/m2-wave3-code-r1-sonnet-output.md`（给定 sha256 `023fbb235bbe3f071f001491120dcf3ad6f32faaf1dbf16f940954a8cfa5c91d`）、本地攻方 `research/prompts/m2-wave3-code-r1-local-attack.md` + `-translation-audit.md` + `-runlog.md` + `-output-s1.md` + `-output-s2.md`（`-output-void1.md` 作废）；背景材料 `research/prompts/_m2-wave3-code-r1-background.md`；开工快照 `research/prompts/m2-wave3-code-r1-snapshot/sha256sums.txt`（36 份，00:50:44 UTC）。

## sha256 核对（腿报告文件）

```
$ sha256sum research/prompts/m2-wave3-code-r1-opus-output.md research/prompts/m2-wave3-code-r1-sonnet-output.md
dc92dc659f4d74641ec3c393545a3692a593a33cd4de8d38a8662661d463bb55  research/prompts/m2-wave3-code-r1-opus-output.md
023fbb235bbe3f071f001491120dcf3ad6f32faaf1dbf16f940954a8cfa5c91d  research/prompts/m2-wave3-code-r1-sonnet-output.md
```

两份都与主 agent 给定的 sha256 逐字符一致。

## 开工快照核对（现跑，不采信主 agent 转述）

```
$ sha256sum -c research/prompts/m2-wave3-code-r1-snapshot/sha256sums.txt
```
结果：36 份里 34 份 OK，2 份 FAILED——`.claude/kb/checks-owed.md`、`.claude/kb/milestone/02-second-txn.md`，与主 agent 转述一致（现跑坐实，不是抄的）。三条腿报告对这两份文件按 `文件名:行号` 的引用：

```
$ grep -noE '(checks-owed\.md|02-second-txn\.md):[0-9]+' research/prompts/m2-wave3-code-r1-opus-output.md research/prompts/m2-wave3-code-r1-sonnet-output.md research/prompts/m2-wave3-code-r1-local-attack.md research/prompts/m2-wave3-code-r1-local-attack-translation-audit.md
```
零命中（现跑确认）。因此下文里凡是引到这两份文件的地方都不因为「文件已变」记 ✗——没有这类引用要记。其余 34 份（含全部 `crates/` 与其余 4 份 `.claude/kb/decisions/*`、`invariants.md`、`fs-design.md`）与快照逐字节一致，下文对这些文件的行号核对按**主工作区当前内容**核（与快照等价，不用切副本）。

## 判别力自证

抽取 Sonnet 报告里的引用 `.claude/kb/invariants.md:26`（I-1.3，整行抄），在草稿副本 `/tmp/claude-1000/m2-wave3-verifier/self-test/sonnet-output-shifted.md` 里把这一处行号改成 27（`python3` 定点替换，只改第 17 行那一处出处标注，不动别处），按下面「每处引用」的核法核它：

```
$ awk 'NR==27' .claude/kb/invariants.md | cut -c1-80
| I-1.4 | 块头 fsid 一致 | 任一单元，其头记录的 fsid == 系统配…
```

而报告里紧跟在出处标注之后引用的整行原文开头是 `> I-1.3 | 块头树 ID 一致 | 任一单元，其头记录的所属树 ID 与…`。第 27 行是 I-1.4，与引用的 I-1.3 原文完全不是同一行。

**结果：✗**（行号 27 处的内容与引用处抄的原文不符，判定为「行号错」）。核查方法能分辨错误引用，往下按此方法核全部引用。

## 腿一：云端攻方（Opus）`research/prompts/m2-wave3-code-r1-opus-output.md`

### kb 引用（整行核对，`awk 'NR==行号'` 现取，逐字符比对）

| 引用 | 核的结果 |
|---|---|
| `.claude/kb/decisions/08-核心索引结构.md:240`（附录 D，D8 已定项 ②「累计」⚠️ 段） | ✓ 逐字符一致 |
| `.claude/kb/invariants.md:58`（附录 D，I-7.8 整行） | ✓ 逐字符一致；正文里摘引的那一句「扫描方向只数诞生代号…孤儿节点不算「出现过」」是该整行的逐字子串 |
| `.claude/kb/decisions/23-journal的角色与格式.md:362`（附录 D，已定项 3「计数器全池接着走」整段） | ✓ 逐字符一致 |
| `.claude/kb/decisions/08-核心索引结构.md:203`（「孤儿与活版的实例代号必不相等」一句，报告自称「同一句在这份文件里命中 2 次，取第 203 行那一次」） | ✓：`grep -n` 现查确认该句在该文件里恰好出现两次，第 203 行与第 264 行；报告点名取的是 203 行，与自称一致 |
| `.claude/kb/invariants.md:136`（附录 D，I-3.10 整行） | ✓ 逐字符一致 |
| `.claude/kb/invariants.md:127`（附录 D，I-3.1 整行） | ✓ 逐字符一致 |

### `crates/` 代码引用

| 引用 | 抄的内容 | 核的结果 |
|---|---|---|
| `crates/singlefs-core/src/mount.rs:317` | `CheckpointTxg(highest_ring_txg.max(highest_record_txg).0 + 1)` | ✓ 逐字符一致 |
| `crates/singlefs-checker/src/walk.rs:877` | `&& read_u64(&bytes, 52 + 2 * usize::from(bytes[51])) <= newest_published_txg` | ✓ 逐字符一致 |
| `crates/singlefs-checker/src/walk.rs:2614` | `fn versions_applied_only_by_records(` | ✓ 逐字符一致；四条判据 ①②③④ 逐条比对函数体（2614-2661 行）与报告描述完全对应，含 ③ 用的 `rollback_floor` 参数确认由调用点（2858-2862 行）传入 `newest_rollback_floor` |
| `crates/singlefs-checker/src/walk.rs:1581` | `fn allocation_record_node_pointers_of_the_candidate_versions(` | ✓ 逐字符一致 |
| `crates/singlefs-format/src/lib.rs:127`（附带事实，不在正文判定里但支撑推理） | `pub const TREE_IDENTIFIER_CENTRAL_MAPPING: u64 = 15;` | ✓ |

### 附录 A.2 命令复跑（在主工作区现跑，非破坏性只读命令）

```
$ grep -rn "publish_version(" --include=*.rs crates | grep -v "fn publish_version"
```
10 行结果与报告贴出的 10 行**内容完全相同**（grep 的目录遍历顺序不保证，两次跑行序不同，但集合相同，逐条核对无一处缺漏或多出）。报告称 mount.rs 852/913/981、transaction.rs 2156/2271 五处调用「全部带 `Some(...)`」——五处逐一现查参数列表末项，`mount.rs:871`→`Some(&*current)`、`mount.rs:930`→`Some(previous)`、`mount.rs:981`→`Some(current_file_version)`（同一行）、`transaction.rs:2181`→`Some(previous)`、`transaction.rs:2289`→`Some(previous)`，全部核实为真。传 `None` 的那一处（`second_transaction_supplement_two_accounting_node_full.rs:93`）与配的 `TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH`（同文件 90 行）也核实为真。**✓**

### 附录 A.1、B、C、E：既有产物里的命令复算（在模型目录 `logs/` 上现跑同一条命令）

逐条重跑报告给出的命令，比对原样输出：

| 命令（对象） | 结果 |
|---|---|
| Appendix A.1 两段 perl/grep（`y1-run1.log`、`fixD-y1.log` 计数与 k=44/54/84/98 摘录） | ✓ 逐字符一致（130 格全部只红 I-7.8，四行摘录内容一致） |
| Appendix B 两条 grep（`fixD-tests.log`、`fixD-m356.log`） | **命令与输出的对应关系有一处对不上**：报告给的命令是 `grep -E '^test result\|^exit\|FAILED' .../fixD-tests.log .../fixD-m356.log`（`fixD-tests.log` 在前），本核用相同参数顺序重跑，`grep` 按文件参数顺序输出，结果是 **`fixD-tests.log` 的三行在前、`fixD-m356.log` 的三行在后**；而报告贴出的输出顺序是 **`fixD-m356.log` 三行在前、`fixD-tests.log` 三行在后**。六行本身的文字内容分别核对均与两份日志逐字符相符，不是伪造数据，但**贴出的「命令 + 输出」这一组现在复跑对不上顺序**——按这份报告给的命令实际执行，得不到报告里那个顺序。记为 **✗（顺序对不上，非内容错）** |
| Appendix C 三条命令（`y3-run1.log`/`y3-formatted.log` 的 I-3.1/I-3.10 计数、k=60/78 摘录、`prefix_applied` 摘录） | ✓ 逐字符一致 |
| Appendix E 四条命令（`mutY4-run2.log`/`fixE-mutY4.log` 的 k=77..81 摘录、两处 `diff` 一致性断言） | ✓ 逐字符一致，`IDENTICAL_LINES`／347 与 `SAME_OUTSIDE_78_80` 均复现 |

### Opus 报告小计

核了 22 处（kb 引用 6 处 + 代码引用 5 处 + Appendix A.2 grep 1 处 + Appendix A.1/B/C/E 命令复算 10 处，B 记两条各算 1 处，共 4+2+3+... 具体计数：kb 6、代码行号 5、A.2 grep 1、A.1 命令 1、B 命令 1、C 命令 3、E 命令 4，合计 21）；✓ 20 处，✗ 1 处（Appendix B 的命令/输出顺序对不上，内容本身不错）。核不动：无（这一段全部可现查或现跑）。

## 复跑：Opus 模型目录 `run.sh`（在草稿目录 `/tmp/claude-1000/m2-wave3-verifier/rerun-opus/` 独立重建，非在腿的原目录跑）

```
$ nice -n 19 bash research/prompts/m2-wave3-code-r1-opus-model/run.sh "$(pwd)" /tmp/claude-1000/m2-wave3-verifier/rerun-opus
```

跑前 `ps` 看到机器上另有 9 条以上 `cargo test --all`/`cargo build`/`cargo clippy` 在跑（不同实现员的 `/tmp/claude-1000/impl-*` 目录），没有 `qemu-system`/`vm-bench.sh`/`e152`/`fio`；照共用约束加 `nice -n 19`、`CARGO_BUILD_JOBS=4` 跑。比对时把两边路径（`/tmp/claude-1000/m2-wave3-opus/repo*` vs `/tmp/claude-1000/m2-wave3-verifier/rerun-opus/repo*`）与编译计时行（`Compiling`/`Finished ... in Xs`/`finished in Xs`）一并剥离，只比 `k=`/`step=`/`action=`/结果行等实质内容（比对脚本 `/tmp/claude-1000/m2-wave3-verifier/compare-log.sh`）。

### Y1-a（130 格只红 I-7.8）

```
$ diff <(strip logs/y1-run1.log) <(strip rerun-opus/y1-run1.log)
```
排序后逐行比对**完全一致（0 差异）**；用报告 Appendix A.1 同一条 perl 聚合命令跑重跑产物，得到与报告一致的计数（mkfs/rollback 各前缀 clean/red 计数逐行相同）与「130 violations=1: ["I-7.8」。**Y1-a 独立复跑坐实**，与报告逐字段相符（两条线程交错顺序不同属预期的非确定性，不算差异）。

### Y3 两条扫描（候选 b 与 C533，Y4-a 的对照证据）

```
$ diff <(strip logs/y3-formatted.log) <(strip rerun-opus/y3-formatted.log)
```
排序后完全一致（C533 的 119 格全绿复现）。

`y3-run1.log`（候选 b 主扫描）排序后比对：**只有一行差异**——留存的原始日志末尾多一行 `exit=0`，且原始日志的测试摘要行是「…0 measured; **2** filtered out…」，本次独立重跑同一条命令得到的却是「…0 measured; **4** filtered out…」（现查 `cargo test -p singlefs-harness --test opus_attack_y3 -- --list` 确认该测试二进制里实际有 5 个测试：`y1::` 两个（`opus_attack_y3.rs` 顶部 `#[path="opus_attack_y1.rs"] mod y1;` 带进来的）加自身 3 个，按名字过滤剩 1 个匹配、4 个被过滤，这与我方重跑的数字对得上，而不是留存日志的「2 个被过滤」）。**这处「filtered out」计数与末尾 `exit=0` 行，用今天 `run.sh` 里的命令（不含 `echo exit=$?`）重跑复不出**——说明留存的 `logs/y3-run1.log` 很可能不是用现在存档的这份 `run.sh`/`opus_attack_y3.rs` 组合直接产生的（或者产生时该驱动文件还没有 `mod y1` 这一段）。**这一条记 ✗（产物与「跑它的命令」现在对不上）**，但两边逐行按 `k=`/`step=`/`red=` 排序比对的**实质内容（i.e. 每个前缀每一步判出的违例）逐字符相同**，候选 b 与 I-3.10 的读数本身不受影响。

### 改法 D（`fixD-walk.diff`，副本 `repo-fixD`）

```
$ diff <(strip logs/fixD-tests.log) <(strip rerun-opus/fixD-tests.log)
```
两条测试套件在独立重跑里同样全绿：`checker_known_bad_images` 23 passed、`second_transaction_step_four_rollback` 11 passed（本次重跑分别用时 64.06s / 45.03s，机器负载比留存产物记的 1039.53s / 668.32s 轻很多，属正常挂钟波动，不影响判定）。**✓**

`fixD-y1.log`（改法 D 之下 Y1-a 130 格是不是真的清零）：见文末「复跑最终结果」一节——独立复跑坐实，288 格全绿。

### 改法 E（`fixE-walk.diff`）

`fixE-mutY4.log`、`fixE-y3.log` 与 `fixD-m356.log` 三段结果见文末「复跑最终结果」一节——三段全部独立复跑坐实。

## 腿二：云端正推（Sonnet）`research/prompts/m2-wave3-code-r1-sonnet-output.md`

### kb 引用（整段/整行核对）

| 引用 | 核的结果 |
|---|---|
| `.claude/kb/invariants.md:26`（I-1.3 整行） | ✓ 逐字符一致（判别力自证用的就是这一处） |
| `.claude/rules/fs-design.md:17-41`（整段抄） | ✓ 逐字符一致，含末尾空行 |
| `.claude/kb/decisions/16-发布语义.md:207-221`（已定项 9 整段） | ✓ 逐字符一致 |
| `.claude/kb/decisions/22-单元原子性怎么合成.md:145`（已定项 7 字段表「分配记录树根指针」行） | ✓ 逐字符一致 |
| `.claude/kb/invariants.md:281`（I-9.14 整行） | ✓ 逐字符一致 |
| `.claude/kb/decisions/22-单元原子性怎么合成.md:725`（偏移累加表「…分配记录树根指针 342…」，共引用 2 次：第 118 行「偏移（342，见…:725 的偏移累加表）」与「字段表（…:707-725）」） | **✗ 行号错**：该文件今天全文只有 534 行，第 725 行根本不存在。现查偏移累加表实际住在该文件**第 150 行**（`行序即盘上顺序，偏移按行累加：…分配记录树根指针 342…`），字段表整节在**第 128-153 行**（`#### 已定项 7：根记录的字段表` 标题在 128 行，「合计 457 字节」收尾在 153 行）。**误写成背景材料第 725 行，原文件实为第 150 行**：`grep -n` 现查 `research/prompts/_m2-wave3-code-r1-background.md` 与 `_m2-wave3-code-r1-appendix.md`，同一句「行序即盘上顺序…」在背景材料里正好住在第 725 行（附录里住第 236 行），坐实是把背景材料的行号当成了 kb 文件自己的行号——而报告开头明文承诺「kb 行号同样现查取自 `.claude/kb/`、`.claude/rules/` 当前文件，不从背景材料的附录里数」，这一处没有兑现 |
| `.claude/kb/decisions/22-单元原子性怎么合成.md:725`（第三次，`newest_rollback_floor` 偏移 130 的出处，第 153 行原文） | 同上，**✗，同一种误写**，实际应为**第 150 行** |

### `crates/` 代码引用

| 引用 | 核的结果 |
|---|---|
| `walk.rs:449-471`（`walk_tree_table_and_central_mapping_root`，463-471 行注释+调用） | ✓ 内容与行号一致（467 行起的调用横跨 467-471 行，报告写「467 行起」不是「467 行」，不算错） |
| `mounted_read.rs:328` | ✓ 逐字符一致 |
| `recovery.rs:808-824` | ✓ 一致；808-809 行注释的摘引子串一致 |
| `recovery.rs:1527-1534`（报告细分「1531-1533 行」+「1530 行注释」） | **部分 ✗**：1530 行确实是注释，但注释原文「中央映射树不进树表：它是哪棵树由根记录里它那条根指针的出生树说」与 recovery.rs:808-809 的注释「中央映射树不进树表、取根记录里它那条根指针的出生树」用词不完全相同，报告称「1530 行注释**同一句话**」不准确（应为「同一个意思，用词不同」）；另外报告贴出的代码片段（`mapping: read_tree_root(reader, root.mapping_root.head.birth_tree, 27, &root.mapping_root, root, expected_filesystem_identifier)?`）实际横跨源码 **1531-1538** 行（8 行），报告标的范围「1531-1533」只覆盖前 3 行，闭合括号 `)?,` 等在 1538 行——**行区间少标了 5 行** |
| `walk.rs:2803-2807`（`newest_rollback_floor` 赋值） | **行区间偏差**：`u64::from_le_bytes(` 这半句实际在**第 2802 行**，报告标的起点 2803 只包含到 `.expect("8 字节"),`；闭合的 `);` 在**第 2806 行**，2807 行是下一条注释，不属于这段赋值。报告给的范围与实际赋值语句所在行（2802-2806）整体偏移 1 行 |
| `mount.rs:1419` | 该行内容为「// I-9.14（树表条目的诞生 txg 跨根不变） 只比同一条时间线上的根，被这次回退切掉的那几条不与新线比。」，与 invariants.md:281 里「不与现行线上的根比」措辞不同（「新线」≠「现行线」），报告称「同样逐字重复这一句」**不准确**，应为「同一个意思，转述用词不同」 |
| `walk.rs:1387-1435`／`1441-1493`（`judge_tree_table_birth_txg`／`judge_release_generation_and_tree_table_birth`，含 1410-1417、1418、1477 行细节） | ✓ 逐字符一致，函数名、行为、`for index in candidate_indexes.iter().copied()` 所在行（1477）核对无误 |
| `walk.rs:2810-2828`（`candidate_indexes` 过滤逻辑，代码块引用） | ✓ 逐字符一致（含用 `...` 显式标出省略的 `if !walked {...}` 那几行，未冒充完整） |
| `transaction.rs:1220`／`singlefs-format/src/lib.rs:127,357` | ✓ 均逐字符一致 |
| `make_filesystem.rs:318`／`transaction.rs:3372,611,729-737`（分配记录树根指针四种写入/释放判定） | ✓ 五处（含 729-737 的 if/else 结构）逐一现查内容一致 |
| `transaction.rs:901-952`（`publish_instance_table_after_the_release_check`） | ✓ 901 行、952 行内容与函数名核对一致 |

### 其余可现查的陈述句

- 「Y5 子问 1 提到的 `root_record_of_a_file_version_leaves_the_allocation_record_tree_pointer_zero`（`transaction.rs:3371` 注释里点名的用例）今天在 `crates/` 里搜不到同名测试函数」：`grep -rn` 全仓核实——该标识符只在 `transaction.rs:3371` 的注释里出现一次，任何 `.rs` 文件里都没有同名函数。**✓**

### Sonnet 报告小计

核了 26 处（kb 引用 8 处，见上表 7 行——第 6 行捆着 2 处引用；代码引用 17 处，见上表 11 行——第 7/9/10 行分别捆着 2/3/4 处引用；陈述句核实 1 处；判别力自证占用的 1 处不重复计入）。✓ 20 处，✗ 6 处：kb 表第 6、7 行共 3 处（decisions/22:725 三次误引，均为背景材料行号误当 kb 文件行号）；代码表第 4 行 1 处（recovery.rs 行区间少标 5 行 + 「同一句话」表述不准）；第 5 行 1 处（walk.rs:2803-2807 行区间整体偏 1 行）；第 6 行 1 处（mount.rs:1419「逐字重复」表述不准，实为转述）。核不动：无。

## 腿三：本地攻方 `research/prompts/m2-wave3-code-r1-local-attack.md` 等

本地腿按提示第 3 条明令「不引源码行号、不引 diff 行号」（只按函数名与本提示自己的行号），因此这条腿没有「文件:行号 + 抄的原文」这一类需要核的引用；核的重点是（a）提示文件自身嵌入的结构性事实是否属实，（b）转述核对表逐行核对，（c）字词损坏闸的复算，（d）抽样是否满足「一条腿只抽一次样不算一次观测」。

### （a）提示文件 `m2-wave3-code-r1-local-attack.md` 里可现查的结构性事实

| 陈述 | 核的结果 |
|---|---|
| 第 17 节问题 1 七行的 `crates/mutations.tsv` 行号定位（Row 1~7 分别对应 331~337 行） | ✓：`sed -n '331,337p' crates/mutations.tsv` 逐行内容与七行描述（extent 指针未换、inode 写入时间、反向链、屏障缺失、层 0 归属错位、C481、C378）一一对应 |
| 「四个 patch 共碰了 13 个 `crates/*/src/*.rs` 文件；`make_filesystem.rs`、`allocator.rs` 被引用但这一批没有自己的 diff hunk」 | ✓：`grep -oE '^--- a/crates/[^ ]+/src/[^ ]+\.rs$' research/prompts/_m2-wave3-code-r1-diff.md \| sort -u` 现跑得到恰好 13 个文件，与提示第 39 行列出的 13 个文件名（image.rs、walk.rs、mounted_read.rs、mount.rs、recovery.rs、root_record.rs、transaction.rs、bad_disk_input.rs、crash.rs、fault_injection.rs、history.rs、model_comparison.rs、model.rs）逐一对应；`grep -n "make_filesystem.rs\|allocator.rs" .../_m2-wave3-code-r1-diff.md \| grep -E '^---\|^+++\|^diff --git'` 零命中，确认这两个文件确实没有 diff hunk |
| 第 2~14 题「`crates/mutations.tsv` 里文件列是 X 的总行数」十三个数字（4/65/6/22/17/6/8/3/50/7/23/0/56） | ✓：`awk -F'\t' '{print $2}' crates/mutations.tsv \| sort \| uniq -c` 现跑逐一核对十三个数字全部相符（含一开始因 `grep` 子串误伤 `second_transaction_supplement_three_random_history.rs` 把 `history.rs` 数成 7 的自我纠错，`uniq -c` 精确匹配路径后确认是 6） |
| 「`versions_applied_only_by_records` 命中的现有变异行恰好 2 条，before-snippet 都是 `for version in &versions_applied_only_by_records {`」 | ✓：`grep -c` 与逐行核对一致 |
| 「`BaseImageTier`/`newest_root_tree_table_has_no_entries`、`Layer0PublishOfState`、`tree_identifier_watermark_of_the_ring`、`FileVersionTreeIdentifiers`/`issued_from_watermark` 各恰好命中现有变异行 1 条」 | ✓：五个 `grep -c` 均为 1，且 `tree_identifier_watermark_of_the_ring` 命中的正是第 337 行（C378，与 Row 7 同一条变异），与提示第 67 行「这条变异我不能确认是否真碰到这个字段本身」的谨慎表述吻合 |
| 「39 个 harness 集成测试文件」（第 49、77、79 行三处出现「39 files」） | **数字与清单不符**：提示第 77 行给出的清单实际列了 **40** 个文件名（`python3` 现算逗号分隔项数），与 `find crates/singlefs-harness/tests/ -maxdepth 1 -type f -name '*.rs' \| wc -l` 现查今天目录下 **40** 个 `.rs` 文件逐一比对（`diff` 空）完全一致——**清单本身是对的、齐全的，只是三处文字都把「40」写成了「39」**，记 ✗（数字性陈述，不是清单缺漏） |
| Question 8/10/14/11 里对 model.rs、mount.rs、mounted_read.rs、transaction.rs doc 注释的英译（转述核对表覆盖的那几处，见下） | 见「（b）转述核对表」 |

### （b）转述核对表 `m2-wave3-code-r1-local-attack-translation-audit.md` 逐行核

| 原文文件:行 | 核的结果 |
|---|---|
| `crates/mutations.tsv:331~337`（Row 1~7 标签，「步 X 验收第 Y 条」「C481」「C378（认了）」） | ✓ 七行标签逐一与 `sed -n` 现取的 tsv 原文对应（已在（a）核过一次，这里核的是「首稿缺什么标签」的转述表本身没有说错） |
| Question 8 model.rs 被删枚举成员注释：「只存在于 `_m2-wave3-code-r1-diff.md:4220-4222`」 | ✓：`awk 'NR==4220,NR==4222'` 现取，三行 `-` 前缀删除行内容与译文对应（回退目标树表 0 条…设计空白，交主 agent） |
| Question 8 `answer_mount_rollback` 替换注释：「对应 diff `:4257-4259`」 | ✓：三行 `+` 前缀新增行内容（候选集只有上面那三条…C511 第 3 步…I-9.14 只比同一条时间线上的根）与译文逐句对应 |
| Question 10 mount.rs 被删长注释：「`:2920-2928`」 | ✓：九行删除内容（含「树 11」「D23 已定项 14」「C493 由此还清」）与译文逐句对应，且转述核对表自己记录的「首稿漏译、定稿补回」两处引用（树 11、D23/C493）确认现在补上了 |
| Question 10 mount.rs 新字段注释：「`@@ -295,6 +282,10 @@` 之后新增」 | ✓：`grep -n '^@@ -295,6 +282,10 @@'` 现查该 hunk 头在文件里唯一命中（第 2936 行），其后新增字段注释内容（D8 已定项 8 ②、`recovery::highest_tree_identifier_watermark_in_the_ring`）与译文对应 |
| Question 11 mounted_read.rs 新注释：「`:2880-2881`」 | ✓：现取内容与译文（it does not go into the tree table…birth-tree of its root pointer）对应，核对表自己也记「未发现遗漏限定词」 |
| Question 14 transaction.rs 改写后注释：「`:3559-3564`」 | ✓：六行内容（含「2026-09-23 崩溃注入快档打中」「mkfs 的 11」）与译文补回的限定词对应 |

「提示文件里比原文多出来的限定词/括注」两条（Row 1 的「instead of the new one」、Question 8 model.rs 译文末尾说明句）：逐条核对，均如实标注为「补充说明，不改变原意」，没有藏着未标注的多加限定词。**本节 7 处全部 ✓。**

### （c）字词损坏闸复算

```
$ python3 research/scripts/corruption-check.py research/prompts/m2-wave3-code-r1-local-attack.md
绿 …  cjk=0 words=5247 fffd=0 汉字复读=0 英文复读=0 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0
$ python3 research/scripts/corruption-check.py research/prompts/m2-wave3-code-r1-local-attack-output-s1.md
绿 …  words=1071 （其余计数全 0）
$ python3 research/scripts/corruption-check.py research/prompts/m2-wave3-code-r1-local-attack-output-s2.md
绿 …  words=517 （其余计数全 0）
$ python3 research/scripts/oov-check.py .../output-s1.md .../local-attack.md
绿 …  生词=2（inequality、BaseImageTier's）拼接=0
$ python3 research/scripts/oov-check.py .../output-s2.md .../local-attack.md
绿 …  生词=0 拼接=0
```
四条命令现跑结果与 `runlog.md` 记的 words 数、生词清单、拼接数**逐字段一致**。**✓**

第 1 次调用判红作废（粘连=8，8 处 Rust `::` 被启发式规则误判）：`corruption-check.py` 的 `splice_marks` 用正则 `(?<![A-Za-z0-9])[:;,]\w` 判「标点粘连」——对 `A::B` 这种路径，第一个 `:` 前面是字母（不命中），第二个 `:` 前面是标点（非字母数字）、后面跟字母，**恰好命中一次**；机制上确认「一处 `A::B` 记一次粘连」成立，与「8 处 `::` → 粘连=8」的说法在机制上自洽。但**原始（改前）提示文件未被留存**（未提交进 git、已被就地改写成 `..`，`git log` 对该文件无历史），**无法重新对那一份改前文本跑 corruption-check.py 坐实「恰好 8 处」这个具体数字**——记「核不动：改前的提示文件没有留副本，机制成立但具体计数复不了核」。

### （d）抽样次数

`runlog.md` 记 3 次调用（1 次判红作废不计入样本、2 次退出码 0 且两道字词损坏闸判绿），达到「至少两份干净样本」。按 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」，两份干净样本、结论方向一致（对 14 题的作答内容），满足最低门槛。**✓**（这一条只核「记录与门槛」，不核抽样结论本身对不对——那是主 agent 的判断）。

### 核心发现：Row 6（C481）的「没有分档断言」前提是假的，local-attack.md 第 31 行

提示第 31 行称：「I could not find, inside that test function itself, any assertion that reads a tier count or a tier name」（在 `bad_disk_inputs_never_read_back_an_uncommitted_version_and_panic_only_at_known_sites` 里找不到读分档数或分档名的断言）。**现查这句话是假的**：

```
$ grep -n "^fn \|^#\[test\]" crates/singlefs-harness/tests/second_transaction_supplement_three_bad_disk_input.rs | head -3
100:fn print_uncaptured(text: &str) {   （非该函数，仅示范边界搜法）
101:fn bad_disk_inputs_never_read_back_an_uncommitted_version_and_panic_only_at_known_sites() {
264:#[test]
```
该函数体（101~252 行，中间无嵌套 `fn`）里第 145-150 行原样是：
```rust
    assert_eq!(
        report.tally.base_image_tiers_sampled(),
        EVERY_BASE_IMAGE_TIER.len(),
        "有基线档一段历史都没抽到镜像：{}",
        report.tally.render()
    );
```
紧邻它上方第 143-144 行的注释原样是：「C481（坏盘输入的基线镜像取不到树表 0 条的盘面）：基线按档抽，每一档都要真抽到过镜像、真造出过坏镜像。报告里那一句「基线档 N / M 档抽到过镜像」的 N 由这里钉住——把树表 0 条那一档从抽样里拿掉，N 由 2 变 1，这里红。」——**代码自己的注释点名这条 `assert_eq!` 就是为了抓 Row 6 这个 C481 变异而写的**；现查 `EVERY_BASE_IMAGE_TIER`（`crates/singlefs-harness/src/bad_disk_input.rs:875`）今天长度恰为 2，与断言比较的字面量吻合。

**后果**：本地腿两份样本（s1、s2）在这个假前提下，Question 1 Row 6 都答「no」（same-target: no），落回条件句也都写反了方向（s1：「if the test asserted on the number of baseline tiers…, the mutation would fail it」——这句话描述的恰恰是已经存在的事实，只是模型被提示误导成了反事实）；Question 4 Column A（该文件里的测试是否点名了这一round的改动）两份样本都答「unknown」，而 `base_image_tiers_sampled()` 正是这一round新增的方法（提示第 51 行自己也这么写），被这条断言直接调用——按提示自己给的判据（第 41 行「a test naming a segment means the test's own source calls that function…」），Column A 应为 "yes"，不是 "unknown"。**这不是抽样的问题，两份样本答案一致只是因为它们看到的是同一个假前提**——这一条记为背景材料本身未先核实（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「喂给多方论证的背景材料，本身要先核」），不是「本地腿没打中」。

### 本地攻方小计

核了 23 处（结构性事实 8 处 + 核心发现 1 处（Row 6 的假前提，单独计） + 转述核对表 7 处 + 字词损坏闸命令 4 处 + 抽样次数记录 1 处 + 39/40 计数陈述已计入结构性事实的 8 处之内，不重复计）。✓ 20 处，✗ 2 处：39 files 应为 40 files（清单本身完整无缺，只是文字数错）；**Row 6（C481）「测试里没有分档断言」这句是假的——代码里现查确有一条 `assert_eq!` 断言分档数，且注释自称正是为抓这条变异而写，这一处已经不是行号或措辞的精度问题，是背景材料自己的事实性错误，直接改变了本地腿对 Row 6 与 Question 4 Column A 两格的作答方向，见上「核心发现」**。核不动 1 处（第 1 次判红时的原始提示文本，未留副本，机制自洽但具体计数「8」复不了核）。

## 复跑最终结果

跑前 `ps` 现查：`uptime` 显示 load average 175/167/142（32 核机器），`ps aux | grep cargo` 数出另有 12 条以上 `cargo test/build/clippy` 在跑，没有 `qemu-system`/`vm-bench.sh`/`e152`/`fio`；照共用约束继续用 `nice -n 19`、独立 `CARGO_TARGET_DIR` 跑，不因负载重就不跑，但挂钟因此显著拉长（部分阶段单条命令原本几十秒，这一次跑到几分钟到十几分钟）。

| 阶段 | 目标（原始产物字节数） | 复跑结果 |
|---|---|---|
| `y1-run1.log`（Y1-a，130 格只红 I-7.8） | 46124 | **✓ 独立复跑坐实**：排序后逐行 0 差异，`perl` 聚合计数与 I-7.8 计数（130）逐字符复现 |
| `y3-formatted.log`（C533，119 格全绿） | 8983 | **✓** 排序后 0 差异 |
| `y3-run1.log`（候选 b 主扫描，180 处 I-3.1／48 处 none／347 行判定） | 79745 | **内容 ✓，但产物与「跑它的命令」对不上**：排序后逐行比对本体完全一致；只有末尾一行 `exit=0` 与「filtered out」计数（原始 2、复跑 4）对不上——现查 `--list` 发现 `opus_attack_y3.rs` 顶部 `#[path] mod y1;` 会把 `opus_attack_y1.rs` 的两个测试一并编进这个二进制，今天用现存的 `run.sh`（不含 `echo exit=$?`）复跑只能得到「4 filtered out」，得不到留存日志的「2 filtered out」+ 末尾 `exit=0`。记 **✗（可疑：留存日志疑似不是用今天这份 `run.sh`/驱动组合直接生成的）**，但不影响候选 b 与 I-3.10 的判定内容本身 |
| `mutY4-run2.log`（Y4-a，k=78..80 崩溃镜像不判、挂载后红） | 11348 | **✓ 独立复跑坐实**：排序后 0 差异 |
| `fixD-tests.log`（改法 D 回归：`checker_known_bad_images` 23、`second_transaction_step_four_rollback` 11） | — | **✓**：本次复跑两条测试套件仍分别 23 passed / 11 passed，用时 64.06s / 45.03s（比留存的 1039.53s / 668.32s 快得多，机器负载忽高忽低，属正常波动） |
| `fixD-y1.log`（改法 D 之下 Y1-a 130 格清零） | 36351 | **✓ 独立复跑坐实**：排序后 0 差异，288 行 `violations=0`，全绿 |
| `fixE-mutY4.log`（改法 E 之下 Y4 变异 k=78..80 由漏变红） | 12324 | **✓ 独立复跑坐实**：排序后 0 差异 |
| `fixE-y3.log`（改法 E 不引入误报：候选 b + C533 两条扫描判定不变） | 88925 | **✓ 独立复跑坐实**（交回 SubagentHandback 之后后台任务继续跑完，02:30 完成，89092 字节）：剥掉两边各自的时序噪音（cargo「running for over 60 seconds」慢测试提示、随机线程 PID）后排序比对 0 差异 |
| `fixD-m356.log`（改法 D 之下变异 356 仍红） | 2404 | **✓ 独立复跑坐实**（同一批后台任务跑完，2322 字节）：同一条用例仍然 `FAILED`（`test after_rolling_back_to_a_warm_up_root_the_ring_watermark_outlives_the_file_version_roots_leaving_the_ring ... FAILED`，`0 passed; 1 failed; ...`），剥掉线程 PID 差异后与留存产物一致——改法 D 没有把变异 356 的判别力拿掉 |

**已经坐实的核心结论**：主 agent 点名的两个「重点复跑」目标——Y1-a 的 130 格与 Y4-a 的 k=78..80 三格——**都已经在与腿完全独立的重新编译、重新跑一遍里逐字符复现**，不依赖信任腿留存的日志。改法 D、改法 E 各自涉及的全部 6 段产物（`fixD-tests.log`、`fixD-y1.log`、`fixD-m356.log`、`fixE-mutY4.log`、`fixE-y3.log`，加上前面的 `y1-run1.log`、`y3-formatted.log`、`mutY4-run2.log`）**8 段全部独立复跑坐实**，run.sh 里没有一段是靠信任留存日志过关的。唯一记 ✗ 的是 `y3-run1.log` 的日志/命令对不上，这是本报告发现的一处新问题，与改法 D/E 本身的正确性无关，也不影响候选 b 与 I-3.10 的判定内容。

## 三条腿汇总计数

| 腿 | 核了 | ✓ | ✗ | 核不动 |
|---|---|---|---|---|
| 云端攻方（Opus） | 21 | 20 | 1（Appendix B 命令/输出顺序对不上，内容不错） | 0 |
| 云端正推（Sonnet） | 26 | 20 | 6（decisions/22:725 三处误引 + 两处行区间偏差 + 一处措辞不准） | 0 |
| 本地攻方 | 23 | 20 | 2（39/40 计数错 + **Row 6 假前提，见「核心发现」**） | 1（第 1 次判红的原始提示文本未留副本） |
| Opus 独立复跑（`run.sh`） | 8 个阶段 | 8 个阶段内容逐字符坐实（含 `y3-run1.log` 本体） | 另记 1 处（`y3-run1.log` 的「命令 + 完整输出」这一组核对，与留存产物对不上顺序/计数，见上文） | 0 |

## 没做什么

- 不判三条腿里任何一条打中的东西成不成立、该不该采纳、Row 6 那格该改判成「yes」还是别的——那是主 agent 的职责，本报告只核出「喂给本地腿的这句前提是假的」这一个事实。
- 不判 Y1/Y3/Y4/Y2/Y5 各条推论本身的设计对不对（是不是该收窄、改法 D/E 好不好），只核引用、产物与复跑。
- 没有重新审阅背景材料 `_m2-wave3-code-r1-background.md`、`_m2-wave3-code-r1-appendix.md`、`_m2-wave3-code-r1-checklist.md`、`_m2-wave3-code-r1-body.md` 本身的完整性（三条腿都没有直接引用它们的行号或内容当证据，只有 Sonnet 那处误引的行号巧合落在背景材料里，见「decisions/22:725」那一条）。
- 没有核 `crates/mutations.tsv` 除本地攻方提示点名的那七行、以及本报告顺手核对的那几个标识符匹配之外的其余全部 379 行内容。
- （2026-09-24 02:30 UTC 补记）`fixE-y3.log`、`fixD-m356.log` 两段复跑在本报告初次落盘时仍在后台进行；后台任务已在交回之后跑完，两段结果已补进「复跑最终结果」，均为 ✓。比对脚本 `/tmp/claude-1000/m2-wave3-verifier/compare-log.sh` 与全部产物留在草稿目录。
- 没有核 Opus/Sonnet 报告里标为「推测，没跑」「没验」「只推」的那些格（如 Y1 的「瞬时读错拉低水位」、Y3 的「④ 成立而多并一版把真泄漏藏住」），这些格两条腿自己已声明没做，不在核查范围。
- 没有对本地腿另外两个建议表述之外的判断词（yes/no/unknown）逐格与源码重新对答一遍——除 Row 6 与 Question 4 Column A 那两格因为前提被证伪而重点核之外，其余 12 题 × 4 列的具体作答内容是否与（假设前提为真时）代码事实相符，本报告没有逐格重算，只核了提示本身给的「事实」对不对。

## 产出文件

- 报告：`research/prompts/m2-wave3-code-r1-verifier-output.md`（本文件）
- 草稿目录：`/tmp/claude-1000/m2-wave3-verifier/`——含判别力自证副本 `self-test/sonnet-output-shifted.md`、独立重跑产物 `rerun-opus/`（`repo`、`repo-mutY4`、`repo-fixD`、`repo-fixE-mut`、`repo-fixE`、`repo-fixD-m356` 及各自 `target*` 与日志）、比对脚本 `compare-log.sh`、辅助校验用的临时文件（`actual-tests.txt`、`claimed-tests.txt`、`actual-insert.txt`、`intended-insert.txt` 等）。草稿目录未清理，供主 agent之后核对或续跑。

