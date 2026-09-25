# 里程碑二收尾：代码轮第三轮判决（2026-09-25）

<!-- doc-lint:not-numbers Z13 Z14 Z15 Z16 Z17 Z18 -->

## 一、这一轮的材料与证据

- **正文与材料**：
  - 正文 `research/prompts/_m2-final-code-r3-body.md`；
  - 材料 `_m2-final-code-r3-background.md`、`-checklist.md`、`-appendix.md`、`-diff.md`。
- **开工快照**：`research/prompts/m2-final-code-r3-snapshot/`，内有代码（src、tests）、定义、kb 四份 sha256。派核查员之前 `sha256sum -c` 全对得上，腿引的行都落在快照上。
- **腿与核对表**：
  - 云端攻方 `m2-final-code-r3-opus-output.md`，模型在 `m2-final-code-r3-opus-model/`；
  - 云端正推 `m2-final-code-r3-sonnet-output.md`；
  - 本地攻方 `m2-final-code-r3-local-attack-output-s1.md`、`-s2.md`；
  - 核查员 `m2-final-code-r3-verifier-output.md`：攻方 1 处 ✗、正推 1 处 ✗，都是出处位置写错，不改实质；本地攻方全 ✓。攻方 Z16-a 两遍定点扫描（盘 1、盘 0）独立重跑，产物与归档 sha256 逐字相同；越格线索两个种子也复现了。
  - 核查员交回时留了一遍「预演也读盘核」的现场重跑在后台，主 agent 用进程号等它跑完（10:2x JST），重跑出来的那份产物（草稿目录里，逐字节相同所以没另存）与攻方归档 `research/prompts/m2-final-code-r3-opus-model/logs/z16-targeted-fixreads-240-384.tsv` 逐字节相同（`cmp` 无差、sha256 同为 387da053fe37…）。

## 二、按路径点名

**这一轮判的 `crates/*/src/*.rs`**（第二轮冻结树到这一轮冻结树的 diff）：

- checker：
  - `crates/singlefs-checker/src/lib.rs`
  - `crates/singlefs-checker/src/position_addressed.rs`
  - `crates/singlefs-checker/src/walk.rs`
- core：
  - `crates/singlefs-core/src/allocation_record_tree.rs`
  - `crates/singlefs-core/src/allocator.rs`
  - `crates/singlefs-core/src/extent_tree.rs`
  - `crates/singlefs-core/src/lib.rs`
  - `crates/singlefs-core/src/mounted_read.rs`
  - `crates/singlefs-core/src/mount.rs`
  - `crates/singlefs-core/src/recovery.rs`
  - `crates/singlefs-core/src/transaction.rs`
  - `crates/singlefs-core/src/write_accounting.rs`
- format：
  - `crates/singlefs-format/src/lib.rs`
- harness：
  - `crates/singlefs-harness/src/bad_disk_input.rs`
  - `crates/singlefs-harness/src/history.rs`
  - `crates/singlefs-harness/src/model_comparison.rs`
  - `crates/singlefs-harness/src/model.rs`
- **不判**：`crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs`。它是 E156 的实验装置，由那个实验页判，这一轮只点名。

**这一轮判的定义**：
- `.claude/agent-common.md`
- `.claude/main-agent.md`
- `.claude/agents/crash-verifier.md`
- `.claude/rules/implementation-workflow.md`

冻结之后又改过的几处（检测器第五种拒绝、第三节 Z18 的改法），进第四轮点名。

## 三、逐格判

| 格 | 腿说什么 | 主 agent 现查与判 |
|---|---|---|
| Z13 分配记录树按位置寻址 | 攻方两次抽样：叶缺席再长 1859 段、树表 0 条那一路 576 格、随机历史 11 档，都没打中；checker 那份几何与实现只共享格式常量 | **没打中**（两次抽样，按三方规则记「没打中」） |
| Z14 extent 树两段 | 正推：内联与下段来回切换时旧节点全释放、两层缩回一层、32634 与写路径同一个口径，三问都兑现条款 | **兑现了条款** |
| Z15 挂载怎么读 | 正推：整棵读与按需读都兑现；两个测试缺口——① 打开文件读几个节点的自报数只在零多跳的取样点上与块层数对过；② 提示过期、经映射回退那一路只在单单元镜像上测过，没在下段多层时测 | **兑现了条款，测试有缺口**。两个缺口补测试，交实二六 |
| Z16 预演与真发 | 攻方：C542（取号前预演不读盘核，坏盘加回退到最老根）那一格 16 格打中，两次抽样相同；后果比欠账写的重一步——写行已落盘、它的根盖掉回退目标的环槽，再试同一条回退报 `NotInRing`。其余形态（666 段带文件小容量、回退到最旧根、冻结重挂 160 段）没打中 | **打中已知的 C542**，后果重一步，已由书记十四记进 C542。改不改仍是用户 2026-09-24「记欠账不改」那条定案，主 agent 不推翻；攻方试的「预演也读盘核」量过的数一并交用户（第五节） |
| Z17 15 个测试二进制重钉的数 | 攻方：只读 D3（空间分配） 已定项 10 ⑤、D8（核心索引结构） 已定项 14、D3 已定项 7 的独立模型，逐项算出测试钉的每个数（12 个单元、50180 / 50240..=50252、树表 50252、3472375808 等），全对上 | **没打中**。另报：D3 已定项 10 第 212 行与 D19（块指针的结构与宽度预算） 第 201 行的例子还是旧的 8 单元布局——已记在 E142 重跑之后那一批写回里 |
| Z18 定义改动 | 正推：`agent-common.md` 写全了检测器的拒绝，`main-agent.md` 那张清单少「起看门狗的错误写法」 | **和条款说反话**一处，已改（2026-09-25 JST 10:2x，`main-agent.md`「派出去之后」第一段） |
| 算术 | 本地攻方两次抽样十格全对：W 812、扇出 169、根层 1 / 2 / 3、extent 143 / 144 / 147、145 个单元的下段两层；「整块盘槽数」与「单元区槽数」两种读法在三种盘面上根层相同 | **没打中**（两次）。主 agent 现查：实现写的一侧用「单元区起点 + 单元区槽数」、读的一侧用「盘字节数 ÷ 16384」算同一个绝对槽数（`allocation_record_tree.rs` 的 `of_allocator` 与 `of_reader`），两条路径算同一个量，没有断言把它们连起来——立欠账 C552 |
| 越格线索 | 攻方：240 槽小盘上抬 F 那一串第二次发布被落点拒之后，I-3.1（已分配统计对得上） 多 8 槽（种子 4000000045、4000000204），核查员复现 | 主 agent 判：这就是 C546（抬 F 被拒时扣住的槽不退回） 第二次起那一半（实二五报告第六节第 3 问），在合法历史上让 checker 判红，不能留着。交实二六：抬 F 那一串任何一次被拒都把分配器换回、扣住的槽放开，用这两个种子钉住 |

## 四、改法

| # | 改法 | 交谁 | 被攻过几轮 |
|---|---|---|---|
| 1 | 抬 F 那一串第 n 次（n ≥ 2）被拒时同样换回分配器、放开扣住的槽；种子 4000000045、4000000204 做成用例，今天的代码上红 | 实二六 | 零轮 |
| 2 | Z15 两个测试缺口：多跳取样点上自报节点数与块层数相等；下段多层时提示过期经映射回退读得回 | 实二六 | — |
| 3 | 分配记录树根层的绝对槽数，写侧与读侧两条路径加一条断言连起来，或者收成一个函数；立 C552 | 实二六，书记员记欠账 | — |
| 4 | `main-agent.md` 拒绝清单补上起看门狗的错误写法 | 主 agent，已改 | — |

## 五、交用户

- **C542 改不改**：用户 2026-09-24 定「记欠账不改」。这一轮量出后果重一步：写行已落盘、回退目标离环、管理员没法重试同一条回退。攻方在它的模型上试的「预演也读盘核」，16 格都改成取号之前拒、号不涨、不发写，其余 2603 段没多拒一段；代价是预演多读每个换下单元两份、故障注入用例的读序号全变，瞬时故障那一半修不了。被攻过零轮。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D8（核心索引结构） 已定项 14 | 不影响 | 2026-09-25 不受影响：Z13、Z14、Z15 兑现条款，实现取值没被打中 |
| D18（块里携带什么信息） 已定项 11 | 不影响 | 2026-09-25 不受影响：C542 那一格是条款自认的缺口，改不改交用户 |
| D28（挂载期承诺量） 已定项 1 | 不影响 | 2026-09-25 不受影响：越格线索落在 C546 已记的那一半 |
