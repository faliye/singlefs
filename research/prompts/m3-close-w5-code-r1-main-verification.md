# m3-close-w5-code-r1 判决（代码轮第一轮，2026-10-19）

正文 `research/prompts/_m3-close-w5-code-r1-body.md`，背景材料 `research/prompts/_m3-close-w5-code-r1-background.md`（sha256 be6649a7d2b7a45a57910b32bbd10418ec1094f812d9593cad7995f51ea50628）。开工快照 `research/prompts/m3-close-w5-code-r1-snapshot/`：`tree-sha256.txt` 是被判 9 份文件，`product-code.diff` 是产品代码 diff，`kb-sha256.txt` 是 kb 快照。腿读的是树副本 `~/.local/share/singlefs/work/m3-close-w5-code-r1-tree`。

被判的 9 份文件是：
- `crates/singlefs-core/src/admission.rs`
- `crates/singlefs-core/src/deleted_inodes.rs`
- `crates/singlefs-core/src/mounted_session.rs`
- `crates/singlefs-core/src/mount.rs`
- `crates/singlefs-core/src/recovery.rs`
- `crates/singlefs-core/src/transaction.rs`
- `crates/singlefs-core/src/write_accounting.rs`
- `crates/singlefs-core/src/extent_tree.rs`
- `crates/singlefs-core/src/unlink.rs`

两条腿：
- 正推（Sonnet）：`research/prompts/m3-close-w5-code-r1-sonnet-output.md`，sha256 1cfffc068ad29902396bcf36a4058c0df81887b8cf2f4e286088e4a388a9e6a6。
- 攻方（Opus）：`research/prompts/m3-close-w5-code-r1-opus-output.md`，sha256 840e292edb78e54c1c1127b2299ebeb5cc7be07580d624c61a965a24a3ae395d。模型目录 `research/prompts/m3-close-w5-code-r1-opus-model/`。

## 一、核查

1. 引文：两条腿交回时各自跑了 cite-check。
   - 正推「✓ 核了 45 处引文、143 处裸行号，对上 45 处，没判 143 处」。它后半程有几处读的是主工作区 `crates/`；它附了 `diff -rq`，被判的 9 份与树副本相同。我把树副本的 9 份与 `tree-sha256.txt` 比过，逐份相同。
   - 攻方「✓ 核了 18 处引文、1 处裸行号，对上 18 处，没判 1 处」。
   - 下面判决用到的行，我在树副本里逐处读过：
     - `transaction.rs:3507`、`:3508`、`:3512`、`:3518`、`:3520`、`:8535`；
     - `mount.rs:8813`、`:8863`；
     - `crates/mutations.tsv:4852`。
2. 攻方量过的数：把模型目录拷到 `~/.local/share/singlefs/work/m3-close-w5-code-r1-main-checks/model`，照它报告「复跑」一节的命令起，6G 上限，新草稿目录。原样：
   - 「name=rerun_directed exit=101 test result: FAILED. 9 passed; 1 failed; 0 ignored; 0 measured; 1 filtered out」，红的是 `beta_after_two_rollbacks_lets_a_write_to_an_inode_born_after_the_deletion_through`，与它报告「要主 agent 跑的命令」一节写的期望相同；
   - 「name=rerun_fuzz seed=200 exit=101 种子 200：走了 40 步，β 对不上 1 次」。
   - W5 那两条变异活下来，我没复跑：攻方原样输出在 `out/mut-min-summary.txt`、`out/mut-guard-summary.txt`。下一批实现员要把这两条加成变异行并证红，那一步就是复核。

## 二、逐格判定

| 格 | 正推 | 攻方 | 判定 |
|---|---|---|---|
| W1 写入口与条款 | 一致：两个新入口走同一个 `publish_version`，换下进 defer、准入、会话入口与第一个文件同一套；覆盖缺口三处：别的号 nlink = 0 的写入口拒绝没有测试与变异，空间拒推抬 F 对别的号没有用例，新增的穷举分支没有变异行 | 没打中：交错写、空内容写、快照下写，checker 每版全绿、读回逐字节对；准入没攻 | 代码对条款；正推列的覆盖缺口照收，进修补批 |
| W2 上段规范形 | 一致：写、删都只经 `plan_the_extent_upper_segment`；kb 文字没跟上：D8 已定项 14、16 没有规范形那一句，I-9.40 说明里「GPU 不进内核」与代码相反 | 没打中：长到层级 2 再删到一个、再删空，降两层照抄成根，I-9.40 全绿；截断那条路走 `truncate.rs` 自己的上段计划、不经统一计划，只因截断不改有条目的号集合才等价（推的） | 代码对判决 M2；kb 文字进阶段同步写回；截断那条结构偏离记一笔账，不在这一批改 |
| W3 删除各步 | 一致；kb 第 5、7 条与射程没补句 | 没打中：别的号批大小 1 的每一批、最后一批、段边界状态都续做得完，三种删序终态上段同为 (0, 0) | 一致；kb 文字进阶段同步 |
| W4 恢复、续做、墓碑与 β | 一致；覆盖缺口：β 的 `.max()` 与 `inode_record.is_some()` 提前返回没有变异，也没有「删、回退、再删」两条墓碑的用例；I-1.13 说明里 GPU 那句过期 | **打中，量过**：删 1（txg 6）→ 建 3（出生 8）→ 回退到删 1 之前 → 再删 1（txg 10）→ 回退到建 3 那一版 → 写 3 做成，池级 checker 0 红。原因是墓碑随现行那一版（`mount.rs:8813`），β 取最大（`transaction.rs:3520`），读到被第二次回退抛下的 10。判决 U4「墓碑每对象一条」在回退历史上不成立（量过，两条） | **β 读法错，要改**。这条时间线上 3 生于 1 删完之后，D8 已定项 16 射程「删完之后不再建带数据的文件」被越过，用户 2026-10-19 定的 β「对象出生代晚于 inode 1 墓碑的死亡 txg」没说几条墓碑时取哪条，实现员取最大是读法空白里取错了 |
| W5 测试与变异 | 期望多为绝对值；变异锚点 95 行全命中、点名测试 74 个全在；缺口 A 到 I | **打中，量过**：`.max()` 改 `.min()`、删掉 `if self.inode_record.is_some()`，与 β 有关的三个现有目标照样全绿；理想模型照抄取最大，随机历史在第一个文件删完之后不删别的号 | 照收 |

## 三、改法（被攻过零轮；写回代码之后再攻一轮）

1. **β 只认这一版父链上的墓碑**（攻方改法甲）：取 inode 1 的墓碑死亡 txg 时，按谱系记录（D18 已定项 24 第 5 条 (T_rb, T_old)）把落在任何一段 (T_old, T_rb) 开区间里的死亡 txg 排除，再取最大。
   - 理想模型（`crates/singlefs-harness/src/model.rs`）照同一读法改。
   - 「inode 1 记录在就不判」那一句保留：1 活着时这一版上没有删完第一个文件这件事。它要配一条会红的用例，就是攻方的 `a_write_after_rolling_back_past_the_deletion_of_the_first_file_is_not_judged_by_beta`。
2. **测试与变异**：
   - 攻方 W4 的两段历史（定向用例与种子 200、1051 的交错历史）钉进 harness 档用例；
   - `.max()` 与提前返回各一条变异行，并证红；
   - 正推缺口 A（别的号 nlink = 0 写入口拒）、E 与 F（空间拒推抬 F × 别的号）、G（`extent_tree.rs` 换下判据两半各一行）、新增的穷举分支（`write_accounting.rs`、`admission.rs` 新加的臂）各补用例或变异行；
   - 快照 × 别的文件、整理搬迁 × 别的文件各一条用例（正推 W5 节）。
3. 随机历史在第一个文件删完之后也删别的号：模型实现员当时只在第一个文件记录在时按号删。这一条改 `crates/singlefs-harness/src/history.rs` 的抽法，并进同一批。

## 四、交给实现员

一个实现员，一批改完：第三节第 1 到 3 条，加上调查员查清的 `scan_rebuild_reads_lineage_and_retirement_records.rs` 那条转红（报告 `~/.local/share/singlefs/work/m3-close-w5-retire-red-investigation/report.md`）。改完回到验证段第 1 步整体再验。

改了产品代码 `transaction.rs`（β）与可能的 `mount.rs`，所以再开代码轮第二轮。

## 五、阶段同步与记账

- kb 文字写回，交写回员：
  - D8 已定项 14、16 补规范形与第 5、7 条，射程句改「按 inode 号删任一 inode」；
  - I-9.40 与 I-1.13 说明里过期的 GPU 那一句；
  - 判决 m3-multi-file-unlink-r1 U4 那句「墓碑每对象一条」的射程：回退之后再删会有两条，β 读法照第三节第 1 条。
- 记账：
  - 截断那条路不经统一上段计划（推的等价，靠截断不改号集合）；
  - 每状态对拍派活超时「整套里红、单跑绿」（修 GPU 红那一趟量的）。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D8（核心索引结构） 已定项 16 | 支撑 | 2026-10-19 改了：β 读法收严成「只认这一版父链上的墓碑」，写回时射程句与 K-U5 那一句一起改 |
| D8（核心索引结构） 已定项 14 | 支撑 | 2026-10-19 不受影响：代码对判决 M2；规范形那一句本来就要随阶段同步写回 |
| D18（块里携带什么信息） 已定项 24 | 支撑 | 2026-10-19 不受影响：墓碑与谱系记录的条款原文没说每个对象一条，β 改法用的是它第 5 条的 (T_rb, T_old) |
