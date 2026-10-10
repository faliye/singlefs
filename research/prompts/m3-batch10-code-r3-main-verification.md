# 根环实现第 10 批：代码轮第三轮判决（2026-10-06）

<!-- doc-lint:not-numbers K1 K2 K3 K4 R1 Q3 rr2 S1 S2 S3 -->

正文 `research/prompts/_m3-batch10-code-r3-body.md`；改动全文 `research/prompts/_m3-batch10-code-r3-diff.md`（第 10h 批一份补丁）；冻结副本 `/tmp/claude-1000/m3-batch10-code-r3/tree/`；两份腿报告：辩方 `research/prompts/m3-batch10-code-r3-sonnet-output.md`（162 行，sha256 6e6d1018…fc21，模型 `-sonnet-model/`），攻方 `research/prompts/m3-batch10-code-r3-opus-output.md`（224 行，sha256 ed4d549c…4227，模型 `-opus-model/`）。这是第 10 批最后一轮（`.claude/rules/three-way-inference.md`「第三轮之后停」）：这一轮新冒出来的零轮形态不再为它开一轮，写进第四节交用户。

## 一、核查

| 核了什么 | 结果 |
|---|---|
| 开工快照 | 冻结副本对 `research/prompts/m3-batch10-code-r3-snapshot/crates-sha256.txt` 313 份、`kb-sha256.txt` 4 份逐份相同。腿跑着时主工作区被别的会话改了快照里的文件：形式化接入那条线 34-4（`crates/mutations.tsv` 8 行锚点，冻结副本取的已是改后那一版）与一批 kb 写回（`invariants.md` 五行、`checks-owed.md` 追加），删除与并发那条线第 24 批（10 份，合前原样 `/tmp/claude-1000/m3-batch24/before/`）与 #9 钉值重取（`crash_enumeration_sharded_across_processes.rs`，合前原样 `/tmp/claude-1000/m3-golden/before/`）。kb 四份快照时的原样主 agent 另存在 `/tmp/claude-1000/m3-batch10-code-r3/kb-at-snapshot/`（`sha256sum -c` 全对）。腿引的行一律在冻结副本上核 |
| 引文 | `cite-check.py`（`--root` 冻结副本、`--background`）两份合跑：「✓ 核了 51 处引文，对上 51 处，没判 0 处」 |
| 攻方模型复跑 | 主 agent 照它的 `rerun.sh` 在自己的草稿目录（`/tmp/claude-1000/m3-r1/verify-r3/`，副本已删）setup 之后跑 K1-3 那两段 `r7-prod-g`、`r7-reread_only-g`：两段的 `OPUS3-SUM` 行与腿的 `out/` 逐行相同（3 行、8 行）；`plan=r7-387` 原样：按槽并臂 `sp=narrow-384-20-2 … outcome=Err(FloorRaiseFailedAfterTheMountsPublishes)`，重读臂同一计划 `outcome=Err(NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread)`。`rerun.sh compare` 整套比 23 份，没跑的 21 段报不同，只认这两段 |

腿的数都在冻结副本的拷贝或攻方原型上，是副本上的数，不进 kb；判决只拿来分辨候选。

## 二、逐格判决

判据照正文第五节。这一轮两条腿立场不同（辩方复核、攻方造反例），攻方的「没打中」是一次抽样。

### K1　按槽并两遍

| 格 | 辩方 | 攻方 | 判 |
|---|---|---|---|
| 同一槽两遍读到两个不同的自证根，取重读那一条 | 没打中：取重读的前提是故障集里没有「读到旧内容」，harness 的三种读故障都不是 | 没打中：故障模型里造不出；故障模型之外的错位读上，取重读与取第一遍各在对称的一格丢 C，分不出臂 | **站住**（一次抽样）；「读到旧内容」不在故障模型里，记线索 |
| 一遍读坏不是全零、另一遍全零，并成读坏 | 没打中：读坏的约束包含全零的 | 没打中（按代码推的）：三处消费者都是；822 段全绿、分不出「全零压过读坏」那条臂 | **站住** |
| 并出来的分类与「挂载那一刻读坏」是不是同一类 | 没打中：并出来的读坏集与 `BadAtMount` / `AllZeroAtMount` 都只从 `ring_slot_readings` 派生（`mount.rs:2194`、`2200`） | **不是同一类，线索（活性代价，checker 不红）**：第一遍自证、重读起一直读坏的槽并成自证根，根环表记成「这个进程知道住着根的槽」，进程内抬 F 重读它仍坏就拒这一次抬；窄盘上 31 段从「挂得上」、31 段从「取号之前 Q3 拒」变成取号、写行、暖机之后报 `FloorRaiseFailedAfterTheMountsPublishes`（盘面 checker 0 违例、丢盘整读不变）；只在「从重读起一直坏」的计划上出现 | **新的零轮形态，交用户**（第四节 S1）。两腿答的不是同一件事：辩方答的是读坏集的来源一致，攻方答的是并成自证根之后进了另一类 |
| Q3 收窄之后该拒却挂上、挂上之后 checker 红 | 没打中 | 打中、不分辨臂：重读臂拒成 Q3、按槽并挂上的 678 段里 139 段红 I-7.4，全是被抛弃根 C 那一槽两遍都读坏；同一形在重读臂里挂上照样红（203 段） | **归 C554**（`.claude/kb/checks-owed.md` C554 那一行写着的进入法）：不算这批的打中；给 C554 记一笔新入口与数（R1 之后 139 / 4146 段从「Q3 拒」漏到「挂上」） |
| 网格与层 0 | — | 没打中：按槽并臂 4146 段，红的 488 段全是 C554 那一形；重读臂另有 310 段红、不是 C554 那一形，同一批计划在按槽并臂上全绿；层 0 四段 82044 态 checker 与 oracle 都 0（只挂载、不接着写，分不出臂） | **站住**（一次抽样）；带一次覆盖写的层 0 每段 16797718–268828700 态，交用户（第四节 S2） |

### K2　按槽归盘

辩方：第二轮 K2-1 站住，变异行 `crates/mutations.tsv` 第 2864 行只红 `the_first_writable_mount_of_a_formatted_pool_has_nothing_witnessed_and_does_not_reread`。攻方：两种归法只在第 0 代根三份都在时不同（mkfs 之后第一次可写挂载 66 态里 24 态，都不重读，checker 0）；82110 态里非第 0 代根落在公式算不到的槽一次都没有；卸载那一格推的（根槽写被吞时两种归法都只放开到 ≤ 0）。**站住**。

### K3　改过期望的用例

| 格 | 判 |
|---|---|
| rollback 三条慢档 | 两腿都没打中方向：每一步只红 I-3.1、都在记账多的一边（辩方副本探针 60 个状态；攻方逐步打印）。**断言强度缺口（两腿都报）**：用例只在挂载那一刻钉差值（819200、917504），「写到 txg 51 之后」只比不变量名、中间 19 次覆盖写没查方向；多记的字节随覆盖写单调涨到 5799936 / 5898240。用户认的口径是方向加时长，不含大小：下一批把每一步改成核方向（只许记账多），不钉大小 |
| Q3 判官「按表判被抛弃」判等价 | 两腿都没打中：写行范围不含要取的号（辩方推的） |
| 丢盘整读新起点 | 两腿都没打中；辩方报新起点只跑了读错一种读坏计划，读回全零与乱字节没跑：下一批补 |

### K4　数能不能核与清单全不全（辩方）

- 14 行变异、18 个原文各恰好一次、九种组合有单测逐对钉、三行变异（2870、2871、2872）复跑各红：对上。`[9, 7, 10, 8]` 只在变异下的失败信息里出现，不是入库断言钉的值。
- 缺口：① R1 定案写「补 K2-2 那 12 段与反方向 4 段」，第 10h 批只做了 3 + 1（只 384 槽、前置覆盖写 1 次），调度记录没写缩减：下一批补齐盘宽与前置覆盖写两维；② E158 重读计数「归提交时的实验复跑」没有载体：记进提交前清单；③ 第 10 批收拢表没建：在调度记录里建（第五节）；④ `DeletedRowsStillPresent` 仍无用例无变异：下一批；⑤ C724 行里 `mount.rs` 行号过期（2463、2506 → 2543、2586）：下一份写回改。

## 三、改法与去向

| 改法 | 出处 | 被攻轮数 | 去向 |
|---|---|---|---|
| rollback 三条慢档每一步核方向（只许 I-3.1 红在记账多的一边），不钉大小 | 两腿 K3 | — | 下一批 |
| R1 用例补齐盘宽（4 GiB 与 384 槽）与前置覆盖写（1 次与 3 次）两维 | 辩方 K4 | — | 下一批 |
| 丢盘整读新起点补读回全零与乱字节两种读坏 | 辩方 K3 | — | 下一批 |
| `DeletedRowsStillPresent` 补用例与变异 | 辩方 K4（第一轮 K7 就列过） | — | 下一批 |
| C554 记新入口与数；C724 行号改 | 攻方 K1-4、辩方 K4 | — | 下一份 kb 写回 |
| 两遍里有一遍读坏的自证根在根环表里另记一类、抬 F 重读它仍坏时照 `BadAtMount` 钉住不拒 | 攻方 K1-3 | 零轮，没实现、推的 | 交用户（S1） |
| 重读过、仍有读坏的槽而最新根的实例表里有被抛弃实例的行时拒可写 | 攻方 K1-4 | 零轮，推的，会把 S1 那批再拒掉一部分，没量 | 记进 C554，不交（碰不到 R1 的两遍取法，属 C554 那一族） |
| 读根槽时核 txg 与槽对不对得上 | 攻方 K1-1 | 零轮 | 记线索（「读到旧内容」不在故障模型里） |

## 四、交用户的岔路

岔路单 `research/prompts/m3-batch10-code-r3-forks.md`（S1、S2 两行）。

## 五、旁及与去向

| 线索 | 依据 | 去向 |
|---|---|---|
| 「读到旧内容」不在读故障模型里 | 两腿 K1-1 | 记欠账 |
| E158 重读计数没有载体 | 辩方 K4 | 提交前清单 |
| 第 10 批收拢表 | 第一轮判决 K7、本轮辩方 K4 | 调度记录里建 |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D16（发布语义） 已定项 1 | 备料 | 2026-10-06 不受影响：按槽并两遍、按槽归盘照第 37 行实现，站住；S1 定了再看「根槽这一次读坏」那一行要不要补一类 |
| D23（journal 的角色与格式） 已定项 14 | 不影响 | 2026-10-06 不受影响：Q3 两遍并起来都没有才拒，与第 392 行那段单子一致 |

## 被判的 `crates/` 文件

第 10h 批补丁改到的每一份：`crates/singlefs-core/src/mount.rs`、`crates/singlefs-harness/src/mount_transparency.rs`、`crates/singlefs-harness/tests/a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs`、`crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs`、`crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read.rs`、`crates/singlefs-harness/tests/writable_mount_refusal_for_no_valid_root_in_the_rebuilt_allocators_root_ring.rs`、`crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs`。

## 六、用户定案

用户 2026-10-06 弹窗定（岔路单 `research/prompts/m3-batch10-code-r3-forks.md`）：

| 岔路 | 定的 | 去向 |
|---|---|---|
| S1 按槽并两遍之后，第一遍自证、重读起一直读坏的槽被记成「这个进程知道住着根的槽」，抬 F 重读仍坏就拒、挂载在写行暖机之后失败 | 另立一题改 | 下一批实现：两遍里有一遍读坏的自证根在根环表里另记一类，抬 F 重读它仍坏时照「挂载那一刻读坏」钉住、不拒；D16 已定项 1 第 37 行补这一类；单开一轮代码三方（攻方提的改法，被攻过零轮） |
| S2 带一次覆盖写的层 0（每段 16797718–268828700 态，推的每臂 20–35 小时） | 不跑，记欠账 | 欠账表记一笔：态数、推的挂钟、分得出两臂的理由；里程碑收尾时与崩溃放量全量一起排 |
