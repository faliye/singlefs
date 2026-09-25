# c363b-r1 核查员报告

你交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

## 判别力自证

方法：从待核引用里抽一条——Opus 报告第 51 行「`singlefs-core/src/transaction.rs:3619` 整行 `    publish_admission(allocator, &rewritten)?;`」——在草稿目录的冻结副本里把行号 +1 改成 3620，核对第 3620 行的实际内容。

```
$ sed -n '3619p' crates/singlefs-core/src/transaction.rs   # 原行号
    publish_admission(allocator, &rewritten)?;
$ sed -n '3620p' crates/singlefs-core/src/transaction.rs   # +1 之后
    // 整个分配器拷一份、失败就换回去：第一版每次发布约 450 KiB 的拷贝（两盘各一张 211968 位的位图），换来失败路径不留半新的池。
```

第 3620 行是一段注释，与被核引文完全不同，与第 3619 行的原文逐字不符 ⇒ **判 ✗**。核查方法能分辨对错行号，往下用同一套方法核全部引用。

## 输入与快照核对

- 三条腿报告 sha256：Opus `research/prompts/c363b-r1-opus-output.md` = `fe44bccd64d999d5afa3e90a71c4d648ebda4a7442c3dfbc2c51bff75841e224`，与派发提示给的一致。Sonnet、本地攻方两份未给 sha256（未在派发提示里点名要核）。
- 快照 `research/prompts/c363b-r1-snapshot/crates-src-sha256.txt`（49 行）对 `/tmp/claude-1000/c363b-r1/tree/crates/` 逐条 `sha256sum -c`：49 项全部 `OK`。
- 快照 `research/prompts/c363b-r1-snapshot/kb-sha256-at-verify.txt`（6 行：checks-owed.md、decisions/03、08、16、28、`research/prompts/c355-c363-r3-main-verification.md`）对今天工作区里同名文件逐条 `sha256sum`：6 项全部与快照一致 ⇒ 本轮 kb 引用一律对**今天的工作区文件**核（按输入指示，快照与今天一致时按今天的核）。
- 冻结副本 `/tmp/claude-1000/c363b-r1/tree/` 没有 `.claude/kb/`，代码引用一律对这份快照核，kb 引用对工作区核（两者已确认一致）。

## 一、云端攻方腿（Opus）：`c363b-r1-opus-output.md`

### 1.1 kb 引文（文件:行号 + 整行抄）

| 引用（报告行） | kb 文件:行 | 核的结果 |
|---|---|---|
| L109「可再分配 …」 | `16-发布语义.md:33` | ✓ 逐字相同 |
| L53 D3 已定项7 合取表第1条 | `03-空间分配.md:134` | ✓ 逐字相同 |
| L55 D16 已定项1 准入那一行 | `16-发布语义.md:39` | ✓ 逐字相同 |
| L183 D3 已定项9第2条 | `03-空间分配.md:184` | ✓ 逐字相同 |
| L185 D28 已定项1第八项 | `28-挂载期承诺量.md:26` | ✓ 逐字相同 |

命令：对每一行用 `sed -n 'N p' <kb文件>` 取原文，与报告里 `整行：` 之后的反引号内容逐字比对，5 处全部一致，未截断未摘句。

### 1.2 代码引文（对冻结副本核，行号在快照里现查）

| 引用（报告行） | 快照文件:行 | 核的结果 |
|---|---|---|
| L51 `transaction.rs:3619` | 3619 | ✓（判别力自证同一处） |
| L52 `admission.rs:9` | 9 | ✓ 逐字相同 |
| L54 `transaction.rs:2586` | 2586 | ✓ 逐字相同 |
| L55 `mount.rs:887` | 887 | ✓ 逐字相同 |
| L153 `mount.rs:966` | 966 | ✓ 逐字相同 |
| L153 `mount.rs:999` 起 `.map_err(…)` | 999 | ✓ 999 行原文正是 `.map_err(\|cause\| {`，与「999 行起」一致 |
| L153 `mount.rs:1016` | 1016 | ✓ 逐字相同 |
| L153 `allocator.rs:270` | 270 | ✓ 逐字相同 |
| L144（第八项，回收扣住） `admission.rs:299` | 299 | ✓ 逐字相同（引用未标行号处，`admission.rs` 的 `reclaim_released_records_up_to` 只 insert 进 `reclaimed`、不删 `records`，函数体在 1141-1161 行核实，与 L116 的转述一致） |
| L83 `make_filesystem.rs:409`（函数名） | 409 | ✓ `pub fn allocator_after_make_filesystem` |

命令：`sed -n 'N,Mp' <快照路径下的文件>`，逐行与报告引文比对，10 处全部一致。

### 1.3 独立 grep 复核（任务点名的两条，独立于报告自己的command，在快照上重新跑）

**声明 A：「D28 式子在 `crates/*/src` 里的调用点为 0」**（式子实现函数 `admit_on_every_device`，见 L52）：

```
$ grep -rln "admit_on_every_device" crates/*/src/ | grep -v admission.rs
（空）
```
`admit_on_every_device` 只在 `admission.rs` 自身出现（定义 + 两处 `#[cfg(test)]` 用例调用），此外的 `crates/*/src/` 里零命中 ⇒ **✓ 确认**。

**声明 B：「发布路径只调 `publish_admission`」**（L51）：

```
$ awk 'NR>=3531 && NR<=3648 && /admi/' crates/singlefs-core/src/transaction.rs
    // 读的是同一张角色清单：清单里有几个数据单元，装 extent 叶时就装几条记录（`publish_admitted`）。
    publish_admission(allocator, &rewritten)?;
    let outcome = publish_admitted(
/// 可写挂载在**取号之前**按这次挂载要发的那几次（写行 + 暖机）算一遍（`publish_sequence_admission`，增补 2 第 20a 行：
```
`publish_version_of_trees`（3531-3648 行，`publish_version` 的内层实现，即发布路径）里唯一被**调用**的准入函数是 `publish_admission`（3619 行）；其余两处命中一处是注释提到别的函数名（`publish_admitted`，与准入无关，是装 extent 叶的函数），一处是注释提到 `publish_sequence_admission`（挂载路径用，不在这段范围内被调用）⇒ **✓ 确认**。

## 二、云端正推腿（Sonnet）：`c363b-r1-sonnet-output.md`

不含代码引用（该腿自陈不读 `crates/` 当依据，仅一次读快照核实「今天没有调用点」这句由主 agent 给出的观测，未作推导依据）；未建模型目录，无复跑命令。全部引文为 kb。

### 2.1 kb 引文逐条核（文件:行号 + 抄的原文）

| 引用（报告行） | kb 文件:行 | 核的结果 |
|---|---|---|
| L13 D3 已定项12 | `03-空间分配.md:243` | ✓ 逐字相同 |
| L21 D28 已定项4 射程句 | `28-挂载期承诺量.md:98` | ✓ 逐字相同 |
| L25-27 D8 已定项11（多层码2树） | `08-核心索引结构.md:321` | ✓ 逐字相同 |
| L33-35 D16 已定项9 | `16-发布语义.md:209` | ✓ 逐字相同 |
| L39-41 E148 依据句 | `28-挂载期承诺量.md:102` | ✓ 逐字相同 |
| L47-49 D3 已定项5 | `03-空间分配.md:93` | ✓ 逐字相同 |
| L51-53 D3 已定项5 射程句 | `03-空间分配.md:95` | ✓ 逐字相同 |
| L55 D3 已定项7 登记表标题指认 | `03-空间分配.md:115` | ✓ 该行是「#### 已定项 7：分配记录条目的盘上编码，与分配准入合取的登记表」标题，报告只指路未整段抄，与该处措辞（「那张登记表」）一致 |
| L67-69 D16 已定项1 | `16-发布语义.md:39` | ✓ 逐字相同 |
| L73 D28 已定项1「九项减项」 | `28-挂载期承诺量.md:24` | **✗ 行号指错**：24 行实为已定项1 内「逐设备算，每块盘各自满足」这一条分项，被描述的「九项减项」公式本身在 **19 行**（代码块内 `可用(d) = 容量(d) − 已分配(d) − …`）。该处不是整行抄（无 `>` 块引），是段落指路，但指向的行与被刻画的内容不是同一句。另见 2.2 节的口径问题 |
| L81 D28 已定项4「形态」句起 | `28-挂载期承诺量.md:90` 起 | ✓ 「起」字面上不是精确单行，已定项4 标题在 89 行，形态句本体在 94 行，90 行是空行到「定案」句之间的过渡；报告用「起」留了余量，不算误指 |
| L86-87 D8 已定项14 | `08-核心索引结构.md:380` | ✓ 逐字相同 |
| L89 D28 已定项4 形态句 | `28-挂载期承诺量.md:94` | ✓ 逐字相同 |
| L98-99 D8 已定项11 内部条目公式 | `08-核心索引结构.md:323` | ✓ 逐字相同 |
| L110-111 D16 已定项1「非空」判定 | `16-发布语义.md:43` | ✓ 逐字相同 |
| L113 D8 已定项11 树 ID 表 | `08-核心索引结构.md:317` | ✓ 引用片段（树 ID 范围那半句）与该行原文尾部逐字相同 |
| L117-119 D28 已定项1「挂载期承诺量」分项 | `28-挂载期承诺量.md:25` | ✓ 逐字相同 |
| L151-155 附录 A6 全文 C363 | `checks-owed.md:316` | ✓ `diff` 逐字节相同（见下） |

命令（附录整段核对）：
```
$ sed -n '316p' .claude/kb/checks-owed.md > actual.txt
$ sed -n '154p' research/prompts/c363b-r1-sonnet-output.md > report.txt
$ diff actual.txt report.txt && echo IDENTICAL
IDENTICAL
```

**计数**：核 18 处，✓ 17 处，✗ 1 处（L73）。

### 2.2 L73 的 ✗ 对论证的影响（观测，非判决）

行号错指之外，L73 把「除容量与已分配外全是预留或独占性质的量」列出 5 项（挂载期承诺量、被抛弃根独占量、待删占用、已承诺预留、checkpoint 保留池），但 19 行公式里减项共 8 个（已分配、不可回收、defer 待释放、挂载期承诺量、被抛弃根独占量、待删占用、已承诺预留、checkpoint 保留池），除容量与已分配外剩 7 项，报告列举漏了「不可回收」「defer 待释放」两项。这是转述遗漏，是否影响格1（从哪里借）的推论由主 agent 判——本行只报「遗漏」这一事实。

## 三、本地攻方腿（V3：树高与 ckpt_cost 算术）

### 3.1 运行记录复核（`c363b-r1-local-attack-runlog.md`）

命令：
```
$ wc -w research/prompts/c363b-r1-local-attack-output-s1.md research/prompts/c363b-r1-local-attack-output-s2.md
1081 …-s1.md
1239 …-s2.md
$ python3 research/scripts/oov-check.py <输出> <提示> 	# 两份样本
绿 …-s1.md  生词=0 拼接=0
绿 …-s2.md  生词=0 拼接=0
$ python3 research/scripts/corruption-check.py <输出>	# 两份样本
绿 …-s1.md  …（全部字段 0）
绿 …-s2.md  …（全部字段 0，Q3a.3 HEIGHT IF VALID 留空符合「无有效状态」预期）
```
词数、绿判定、退出码与运行记录逐字一致 ✓（3 处核对：词数 2、检测器判定 2 类）。

### 3.2 翻译核对表（`c363b-r1-local-attack-translation-audit.md`）表一：kb 引文逐句核对

逐行把「原文文件:行」列的引文与工作区 kb 现查一遍，并把「定稿」列所称的英文措辞去 `research/prompts/c363b-r1-local-attack.md` 里现找：

| 行 | kb 文件:行核对 | 「定稿」列所称英文措辞是否真在提示文件里 |
|---|---|---|
| FACT G（`08-核心索引结构.md:321`） | ✓ 逐字相同（同判别力自证节引用） | **✗ 不在**：该行声称「定稿保留『(the key range in D18, decision item 18)』…括注挪到第 4 条分隔 key 的判定句之后」，但 `grep -n "D18\|decision item 18\|key range" c363b-r1-local-attack.md` 零命中，FACT G 全文（提示文件第 65 行）里没有这个括注，五条编号译文本身完整但这处括注确实缺失 |
| FACT E1（`08-核心索引结构.md:377`） | ✓ 逐字相同 | ✓ 「(user decision K1)」确实不在提示文件里，与「非遗漏、按精神不抄」的说明一致 |
| FACT E3（`08-核心索引结构.md:382`） | ✓ 逐字相同 | ✓ 三件事、两步处置全部逐句可在提示文件第 54 行找到 |
| FACT F1（`08-核心索引结构.md:382`同行另一分句） | ✓ 逐字相同 | ✓ 「(the splitting and shrinking rule)」在，「decision item 11」不在，与所称一致 |
| FACT H（`28-挂载期承诺量.md:94`） | ✓ 逐字相同 | ✓ 两处括注均不在、`m2-keyspace-r1` 已改写成「a separate round of review」，与所称一致 |

**计数**：5 行 kb 引文核对 5/5 ✓；5 行「定稿真在提示文件里」核对 4/5 ✓、1/5 ✗（FACT G 行）。

⚠️ FACT G 这一处的性质：翻译核对表的推理是对的（D18 括注是「条款自己写了要连这句一起引」的显式警告，理由成立），但**执行掉了**——审核表记录的是「已经补回」，而实际发给本地模型的提示文件里从未出现这句。核对表本身就是「英文提示里的每一句转述，写完都要对着原文核一遍」这条纪律要求的产物，这一条自己没有核到自己声称完成的动作。是否影响 Q1/Q2 的作答（该括注管的是分隔 key 判定，不直接进树高算术）由主 agent 判。

### 3.3 翻译核对表二：FACT 编号对照来源（数值、函数、结构）

对冻结副本逐条现查（命令与前两节同一套 `sed -n`，此处只列结果，命令详见判别力自证与 1.2 节所示范式，48 处均用同一命令模式跑过）：

| FACT | 核对结果 |
|---|---|
| A1–A7（`lib.rs`/`unit.rs`/`code_two_tree.rs` 常量与函数体） | ✓ 7/7 逐字相同 |
| B1–B7（记账树常量、角色枚举、`height_read_from_the_root_node_header`） | ✓ 7/7 逐字相同（含 `MultiLevelCodeTwoTree` 只有 `Accounting`/`CentralMapping` 两个变体，B6 属实） |
| C1–C7（中央映射树，同构） | ✓ 7/7 |
| D1–D4（分配记录树单节点、`refuse_when_the_allocation_records_do_not_fit_one_node`） | ✓ 4/4，且独立算出容量 812（见 3.4 节）与 D3 引用一致 |
| E1–E5（kb 引文 + 由 D1/D2 算出的 812） | ✓（E1/E3 引文见表一；E4 claim「实现员还没开工」在 `transaction.rs:1275-1278` 只有两个变体处交叉核实，属实） |
| F1–F4（树表单节点、`build_index_node` 的 level 实参、断言文本） | ✓ 4/4，`transaction.rs:4873` 恰是 `0,`（level 实参） |
| G/G2/G3（kb 引文 + 代码注释 + 自推理） | ✓ G 已核；G2 `code_two_tree.rs:10` 逐字相同；G3 是本地腿自己的推理，标注「不适用」，不构成外部引用，未核 |
| H/H2（kb 引文 + 「材料里没给出这个量的公式」的核实） | ✓ H 已核；H2 通读附录确认 `28-挂载期承诺量.md` 已定项1、4 与 `16-发布语义.md` 已定项1、9 均未给出「记账树每发布节点数」的公式，与 H2 一致 |

**计数**：表二 48 行涉及的引用点核 45 处（G2/G3/H2 三处为自证或核实性描述，非「文件:行」引用），45/45 ✓。

### 3.4 两份样本的算术逐格重算（不依赖模型输出，独立用 FACT 表推导）

用 `python3` 按 FACT A5 / B / C / D 现算：

```
header_bytes(k) = 86 + 2k + 29
capacity(k, w) = (16384 - header_bytes(k)) // w
capacity(22, 34)  = 477   （B4，两份样本均答 477 ✓）
capacity(22, 108) = 150   （B5，两份样本均答 150 ✓）
capacity(27, 55)  = 294   （C4，两份样本均答 294 ✓）
capacity(27, 113) = 143   （C5，两份样本均答 143 ✓）
capacity(10, 20)  = 812   （D3/E2，两份样本 Q3a.2 均答 812 有效、Q3a.3 813 均答无效 ✓）
```

Q1/Q2 各行「SHORTEST POSSIBLE HEIGHT」：两份样本在 Q1.1–Q1.4、Q2.1–Q2.4 上的高度（1/1/2/2）与按 A5 算出的容量阶梯（477、150×477=71550）一致，Q1.5/Q2.5 的最短高度 3（71551、42043 超过两层容量）也一致，两份样本这一列全部 ✓。

Q4-HEIGHTS 与 Q4-SUMS 两张表：按题面给定的取数规则（regime1/2 用 Q3a.1/Q3a.2 的高度 1，regime3/4/5 分配树高度记 undefined，树表高度恒 1）逐格重算，两份样本的 10+10 格全部与重算结果一致，Two-tree/Three-tree/Four-tree 求和与 undefined 传播规则两份样本也全部一致——**这两张表 ✓，20+20 格全部核对通过**。

**「TALLER HEIGHT ALSO POSSIBLE」列（两份样本不一致，逐格重算判哪一份对）**：

FACT G3 的一般原则（该树曾经装过更多条目、部分清空但没有任何节点降到 0 条目时，同样条目数下可以维持更高的高度）对 n ≥ 2 的任何条目数都适用；n = 1 是唯一的下界特例（1 个条目不可能分布在 ≥2 个非空叶子上，2 层结构必然导致其中一叶为 0 条目而触发收缩）。逐格重算：

| 格 | 样本1 | 样本2 | 重算结果 |
|---|---|---|---|
| Q1.1（n=1） | no | No | ✓ 两份一致，n=1 特例成立 |
| Q1.2（n=477） | yes | Yes | ✓ 两份一致 |
| Q1.3（n=478） | yes | Yes | ✓ 两份一致 |
| Q1.4（n=71550） | yes | Yes | ✓ 两份一致 |
| **Q1.5（n=71551）** | **no** | **Yes** | **样本1 与自己在 Q1.3/Q1.4 的推理矛盾**：n=71551 不是下界特例（远大于 1），高度 4 及以上完全可以用「大量叶子各存 1 条目、无叶清零」的方式构造，构造原则与 Q1.3/Q1.4 判「yes」时同一条。样本1 判「no」缺自洽的理由，应为 ✗；样本2「Yes」与 FACT G3 一致 |
| Q2.1–Q2.4 | 与 Q1 同构 | 与 Q1 同构 | ✓ 两份一致 |
| **Q2.5（n=42043）** | **no** | **Yes** | 同 Q1.5，样本1 ✗，样本2 ✓ |

**计数**：算术格重算共 20（Q1 5 + Q2 5 + Q3a 3 + Q3b 1 + Q4-HEIGHTS 5 高度值一致但每格 4 项算 5 + Q4-SUMS 5）+「taller possible」判断 10 格。样本1：taller-possible 列 10 格中 8 ✓ 2 ✗（Q1.5、Q2.5）；样本2：10 格全 ✓。其余数值格两份均 ✓。

这是运行记录判定「干净」（`corruption-check.py`/`oov-check.py` 判绿）与「算术正确」是两件不同的事的实例：样本1 没有字词损坏，但 Q1.5/Q2.5 两格与自己另外两格（Q1.3/Q1.4）的推理原则相矛盾。是否影响 V3 的结论（该腿未被要求下结论，只填表）由主 agent 判。

## 四、Opus 承重数字：先对已入库产物的独立复算

在写完整轮复跑（第五节）之前，先用**独立命令**（不搬报告自己给的 awk，重新写等价查询）在已入库的 `results/v1_reach.out`、`summary.txt`、`summary.z-restore.txt`、三份 trace 文件上复核，确认这些数字与产物自洽；第五节再判产物本身是否可由源码独立重新产出。

### 4.1 36136 段 / 15114 段命中 / 267648 步

对 `summary.txt` 第 1 节（每类段数、命中段数）逐类求和：
```
209+209+742+742+2968+2968+1696+530+212+3180+22680 = 36136        （总段数）
75+75+331+323+1095+1135+528+97+72+1433+9950       = 15114        （命中段数）
```
对第 2 节（每类固定点被拒步数）求和：
```
4359+4431+11396+11678+20604+20943+10010+2805+846+28839+151737 = 267648
```
三项均与 Opus 报告合计数逐字一致 ✓。

### 4.2 混杂因素一列全是 0

`summary.txt` 末节 11 行 `with_missed_reclaim=0`，逐类核对，11/11 全为 0，与报告「267648 次命中全是 0」一致 ✓。

### 4.3 最短的一条（trace-A-66.txt）

```
$ cat results/trace-A-66.txt | head -6
S A-mkfs-process-overwrite 66 K=72 0 Ow Applied … allocated=23 …
S A-mkfs-process-overwrite 66 K=72 1 Ow Applied … allocated=33 …
S A-mkfs-process-overwrite 66 K=72 2 Ow Applied … allocated=43 …
S A-mkfs-process-overwrite 66 K=72 3 Ow Applied … allocated=53 …
S A-mkfs-process-overwrite 66 K=72 4 Ow Applied … allocated=63 deferred=51 … free_slots=3 allocatable=3
S A-mkfs-process-overwrite 66 K=72 5 Ow Refused(PlacementRefused(NoFreeSlotOnAnyDevice)@t3) … allocated=63 deferred=51 … free_slots=3 allocatable=3
```
步 0–4（第 1–5 次覆盖写）Applied，步 5（第 6 次覆盖写）在 `t3` 被拒，此后步 6–71 全部同样被拒、`allocated`/`deferred`/`free_slots` 逐步不变（已用 `sort -u` 核对 free_slots 列在步 5 之后恒为 3）——与报告「第 6 次覆盖写被拒在 t3」「63/51/3」「此后每次都被拒，盘上逐字节不变」逐字一致 ✓。

### 4.4 「扣住的槽」T2/T3 共 2492 次

在已入库的 `v1_reach.out` 上重新写等价 awk（列号相同，条件独立写）：
```
$ awk -F'\t' '$1=="R"&&($28=="T2"||$28=="T3"){t++; if($16>$17)h++} END{print t,h}' v1_reach.out
2492 2492
```
即 2492 次 T2/T3 命中，其中 2492 次（全部）`计数空闲($16) > 可发($17)` ⇒ 与报告 L44「T2 + T3 共 2492 次，2492 次命中时计数空闲都大于可发」逐字一致 ✓；分级细分（D 208+216、D-挂载 144+136、H 700+620、I 257+211，求和 424+280+1320+468=2492）与 `summary.txt` 分级一节逐类核对同样一致 ✓。

### 4.5 Z-restore 从 2492 降到 870

```
$ awk -F'\t' '$1=="R"&&$2~/^(D-raise|D-raise-mounted|H-live-data-raise|I-after-refused-raise)$/&&$9=="fixed-point"{n++; if($16>$17)h++; if($28=="T2"||$28=="T3")t++} END{print n,h,t}' v1_reach.out
222123 13890 2492
$ awk -F'\t' '$1=="R"&&$2~/^(D-raise|D-raise-mounted|H-live-data-raise|I-after-refused-raise)$/&&$9=="fixed-point"{n++; if($16>$17)h++; if($28=="T2"||$28=="T3")t++} END{print n,h,t}' v1_reach.z-restore.out
222123 5811 870
```
`held_at_hit`（计数空闲>可发）13890 → 5811，`T2T3` 2492 → 870，`fp_refusals` 两份都是 222123（不变，符合「有固定点命中的段数逐格不变」的报告说法）——与报告 L172-174 原样输出逐字一致 ✓。

### 4.6 `summarize.sh` 独立复算（衔接源数据 → 统计表这一段的可复现性）

```
$ bash summarize.sh results/v1_reach.out > /tmp/.../summary-recompute.txt
$ diff results/summary.txt /tmp/.../summary-recompute.txt && echo IDENTICAL
IDENTICAL
$ bash summarize.sh results/v1_reach.z-restore.out > /tmp/.../summary-zrestore-recompute.txt
$ diff results/summary.z-restore.txt /tmp/.../summary-zrestore-recompute.txt && echo IDENTICAL
IDENTICAL
```
`summarize.sh` 从已入库的两份 `v1_reach*.out` 能确定性、逐字节地重新产出两份 `summary*.txt` ⇒ 「原始数据 → 统计表」这一段可复现。**这一节确认的是产物内部自洽与统计脚本的确定性，不确认 `v1_reach.out` 本身是否真由 `cargo test` 跑冻结副本 + `v1_reach.rs` 产生**——那一半见第五节（整轮从源码重新编译执行）。

## 五、独立整轮复跑（不在腿的原目录、不用腿自己的草稿，从冻结副本重新编译执行）

**做法**：把 `c363b-r1-opus-model/` 整份拷进草稿目录（保持 `research/prompts/<名>/` 的相对深度，使 `run.sh` 自己算出的 `REPO` 指向能读到 `research/scripts/capped.sh` 与快照清单的副本，其余不改一行），用同样的 `run.sh` 三个参数分别指向我自己的冻结副本读入口、草稿编译树、草稿输出目录，全程 `nice -n 19`，不写进腿的原目录。开跑前 `ps` 核实无 qemu/vm-bench/fio 在跑（只有别的 cargo 编译，未受影响）。

```
$ nice -n 19 bash run.sh /tmp/claude-1000/c363b-r1/tree \
    /tmp/claude-1000/c363b-r1-verifier/opus-rerun/draft-tree \
    /tmp/claude-1000/c363b-r1-verifier/opus-rerun/results
```
退出码 0；`test result: ok. 1 passed …`；主编译 finished in 623.00s（报告原数 620.61s）、Z-restore 编译分支 …finished in 541.96s（报告原数 529.90s），挂钟量级一致（报告自称的「整轮约 22 分钟」属实，本次约 23 分钟，含两次 `cargo build --release` 各约 24s）。

### 5.1 产物逐字节比对

| 产物 | 原文件 sha256 | 复跑 sha256 | 结果 |
|---|---|---|---|
| `summary.txt` | `e02f45f2…` | `e02f45f2…` | **逐字节相同** |
| `summary.z-restore.txt` | `76bee7f3…` | `76bee7f3…` | **逐字节相同** |
| `trace-A-66.txt` | `746a22c3…` | `746a22c3…` | **逐字节相同** |
| `trace-C-64-n24.txt` | `b57576e7…` | `b57576e7…` | **逐字节相同** |
| `trace-C-120-n48.txt` | `a8e8d528…` | `a8e8d528…` | **逐字节相同** |
| `trace-D-raise-136-k20-s1000.txt` | `ccdc1c3d…` | `ccdc1c3d…` | **逐字节相同** |
| `v1_reach.out`（378894 行） | `08239dcd…` | `390231e9…` | sha256 不同；`diff` 逐行核：**唯一差异是第 1 行编译耗时（0.48s→0.00s）与末行总耗时（620.61s→623.00s）两行，其余 378892 行逐字节相同** |
| `v1_reach.z-restore.out`（311663 行） | `c946c4e1…` | `335bde0a…` | sha256 不同；`diff` 核：**唯一差异是末行总耗时（529.90s→541.96s），其余 311662 行逐字节相同** |

两份 `.out` 文件的 sha256 不一致完全落在「复跑命令输出里嵌着的时间戳字段」这一条已知例外（挂钟耗时不是决定性数据），按字段比对通过；六份统计/追踪产物逐字节相同，说明整套判定（36136/15114/267648、11 类混杂因素全 0、最短一条的 t3、T2/T3 2492、Z-restore 降到 870）**在一次完全独立的编译与执行下逐字节复现**，不依赖任何缓存或未提交状态。

### 5.2 判定

第四节列出的每一个数字（36136、15114、267648、11 类 with_missed_reclaim=0、trace-A-66 的 63/51/3 与「被拒在 t3」、T2+T3=2492、Z-restore 870）全部由这次独立复跑的产物**逐字节验证**，不仅仅是「用报告自己的命令在报告自己的产物上再跑一遍」。**✓ 全部确认**。

## 六、按腿汇总计数

| 腿 | 核了几处 | ✓ | ✗ | 核不动 / 分不清 |
|---|---|---|---|---|
| Opus（云端攻方） | kb 引文 5 + 代码引文 10 + 独立 grep 2 + 承重数字 5 大项（36136/15114/267648、混杂因素、最短一条、T2/T3、Z-restore，逐项用产物字段核并配整轮独立复跑） | 22 | 0 | 0 |
| Sonnet（云端正推） | kb 引文 18 | 17 | 1（L73 `28-挂载期承诺量.md:24` 行号误指，且列举有遗漏） | 0 |
| 本地攻方（V3） | 运行记录 3（词数×2、检测器判定×2 算 1 组）+ 翻译核对表一 kb 引文 5 + 表一「定稿」英文措辞核对 5 + 表二引用点 45 + 算术格（Q1/Q2/Q3a/Q3b/Q4-HEIGHTS/Q4-SUMS 共 40 格数值 + 10 格 taller-possible 判断，两份样本各算一次） | 数值格：样本1 40/40、样本2 40/40；taller-possible：样本1 8/10、样本2 10/10；kb 引文 5/5；「定稿」措辞核对 4/5；表二 45/45 | 表一 FACT G「定稿」1 处（D18 括注声称已补回，实际提示文件里没有）；样本1 Q1.5、Q2.5 两格（taller-possible 判「no」与自己 Q1.3/Q1.4 的推理矛盾） | 0 |

**总计**：核对项约 190 处（含逐格算术），✓ 约 186 处，✗ 4 处（Sonnet 1 处、本地攻方翻译核对表 1 处、本地攻方样本1 算术 2 处），核不动 0 处，分不清 0 处。

## 没做什么

- 不判三条腿任何一条推论打中成不成立、该不该采纳——四处 ✗ 只是观测，是否影响各腿的结论、是否需要重派，由主 agent 逐条现查后定。
- 没有核 Sonnet 报告 L55 D3 已定项7 登记表标题那一处指路引用之外、正文散落提到但未显式给出「文件:行」的其余 D3/D8/D16/D28 分项措辞（例如「E148」「C516」等仅引编号未附行号的提法）——这些不落在「文件:行号 + 抄的原文」的射程内。
- 没有核 Opus 报告第七节「没打中的形状」表里各行的取样过程本身（只核了它们引用的既有产物字段，未重新扫描第八节之外的取样范围）。
- 没有核 Opus 报告草稿目录里提到但已被 `run.sh` 的 `rsync --delete` 删除的中间产物（`tree-z`、`work/`、`full1.out`、`z.out` 等）——按报告自陈这些不作依据，未去索取。
- 没有核 Sonnet 报告未点名具体行号的「E148」模型范围之外的旁证（如 D5 已定项 2、D22 已定项 7、D19 已定项 8/10 等报告提到但未整行抄的分项）——这些在报告里是指路而非逐字引用，不在核对射程。
- 没有判 Sonnet 报告 L73 的遗漏（漏列「不可回收」「defer 待释放」两项减项）是否影响格 1「从哪里借」的结论——这需要判断推论本身，不归核查员。
- 未跑 `.claude/gate.d/` 任何阶段，未编译除 `v1_reach` 测试二进制之外的任何目标，未跑任何名字含 `layer0` 的测试。
- 复跑仅覆盖 Opus 腿点名的这一个测试文件与两个编译分支（主副本 + Z-restore 补丁副本），未复核 Opus 报告里提到但这次未重新触发的其余对照（如「反推」的第七节表格所列各形态的取样脚本本身）。

报告文件：`research/prompts/c363b-r1-verifier-output.md`。
