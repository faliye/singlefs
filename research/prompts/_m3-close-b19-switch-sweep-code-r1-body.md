# m3-close-b19-switch-sweep-code-r1：批 19 写段末两处产品代码改动的代码轮（2026-11-06）

## 一、题面

批 19 写段在 `research/prompts/m3-close-b19-r3fix-code-r1-main-verification.md` 那一轮判决之后又并进两处产品代码（射程照 `.claude/rules/implementation-workflow.md`「三步，缺一步就不算做完」：`crates/singlefs-core/src/` 走代码轮，含其中的 `#[cfg(test)]` 模块）：

1. E205（多处读坏倒挂三条改法的代价）要的新实例第一个 txg 只供测试开关，在 `crates/singlefs-core/src/mount.rs`。
2. C973（实例表清扫读根环不核落点）的改法，在 `crates/singlefs-core/src/instance_table_sweep.rs`。

同一批里并进 `mount.rs` 的测试模块（C650 留档与补测两件）也在这一轮的射程里。

这一轮只回答一件事：**这两处改完之后，代码做的是不是条款说的，哪一格会错**。

## 二、实现今天的样子（主 agent 的观测，2026-11-06）

方案按 `crates/` 今天的实现来谈。行号是此刻主工作区的（`mount.rs` sha256 7aeaa0980972ec71…，`instance_table_sweep.rs` sha256 bbff4180df8207ee…，本批补丁都已打；文件相对 HEAD 的差里还有别的会话的改动，不在这一轮射程）。

**开关（mount.rs）**
- `mount.rs:920` `pub enum FirstTxgOfTheNewInstance`，三档：
  - `PastTheHighestRootAndRecordInTheRing`：今天的实现；
  - `PastRingSlotsReadBadAtMount`：甲只并读坏，第 924 行；
  - `PastRingSlotsReadBadOrAllZeroAtMount`：甲并全零，第 926 行。
  - 档名经 `MountOutput::first_txg_of_the_new_instance_branch` 报出，第 1087 行，在第 7926 行填。
- `mount.rs:1502` `first_txg_of_new_instance`：今天的算法，取 `checkpoint_txg_after_the_version(max(highest_ring_txg, highest_record_txg))`。
- `mount.rs:1548` `first_txg_of_new_instance_under_the_test_only_switch`：按档分派，第 1562、1565 行是甲两档。
- `mount.rs:1630` `first_txg_past_the_ring_slots_read_bad_at_mount`：甲的算法。
  - 取 N = 读得出的根里最小的 txg。
  - 对挂载那一刻读不出、读得出而自证不过，以及（甲并全零那一档）读回全零的根环槽，在 [N + 1, N + 3S − 1] 里取落到该槽的那个 txg，当它「能住的最大 txg」。
  - 结果取 max(今天的值, 这些 txg 的最大者 + 1)。
  - 走出 u64 报序号到顶，不回卷。
- 只供测试的入口：`mount.rs:8974` `mount_writable_with_test_only_switches_and_the_first_txg_of_the_new_instance`。产品入口 `mount_writable` 那一串固定传今天那一档。
- 跑前登记：`research/prompts/e205-preregistration.md` 第一节「甲并全零」「甲只并读坏」两条，是开关两档的定义原文。
- 测试：`crates/singlefs-verification/tests/writable_mount_first_txg_switch_past_root_ring_slots_read_bad_or_all_zero.rs`（12 条）与 `mount.rs` 里 `first_txg_past_the_ring_slots_read_bad_at_mount_tests` 模块。
- 变异：`crates/mutations.tsv` 里「E205（多处读坏倒挂三条改法的代价） 新实例第一个 txg 的只供测试开关：…」18 行。主 agent 在副本上跑过证红与 `--item` 复跑，19 行（连锚点改写的 C378 一行）全抓到，任务 8528、8687、8776。

**清扫落点核（instance_table_sweep.rs）**
- `instance_table_sweep.rs:464` `every_root_of_the_ring_self_verified`。改前逐槽 `parse_slot`、不核落点。改后在第 492 行起调读者同一个核 `root_ring::root_sits_in_the_ring_slot_the_formula_gives`（第 45 行 import）。
  - 落点不对，算这一槽自证不过，走原有的「重读一次，仍不过就这一轮不删」。
  - 交 `NoRowDeletedThisRound::ARootRingSlotIsNotSelfVerifiedAfterOneReread`（第 283 行、第 504 行）。
  - 与 `recovery::read_root_ring_slot` 的分类一致，没有开新的错误成员。
- 测试模块 `misplaced_ring_reads_tests`（第 955 行起）两条用例：
  - 两次读都错位：这一轮不删；
  - 只有第一次错位、重读读对：那一行不是候选行（阳性对照）。
- 变异两行：「C973 清扫读根环不核落点公式」「C973 清扫读根环自证不过不重读」。副本上证红两行都抓到（任务 8811）；「不核落点」那一行红在用例「错位读把实例 1 唯一的根换成了别槽的根，清扫判出要删的行」那一句。
- 读者一侧的落点核：
  - `crates/singlefs-core/src/root_ring.rs` 第 183–193 行一带，txg 0 只认第 0 代那几槽，别的 txg 只认 `target_for_publish` 那一槽；
  - `recovery.rs` 第 2009 行一带调它。

## 三、条款

- D16（发布语义） 已定项 6（`.claude/kb/decisions/16-发布语义.md` 第 195 行一带）：落点核「恢复与挂载读根环时都调」。
- C973（实例表清扫读根环不核落点）（`.claude/kb/checks-owed.md`）整行。
- C971（多处读坏时可写挂载写出倒挂的新实例根）（`.claude/kb/checks-owed.md`）整行。岔路单 `research/prompts/m3-close-b19-c971-forks.md`。
- `.claude/rules/fs-design.md`「五条硬要求」：
  - 每条分支能被测试强制进入；
  - 分支可观测；
  - 分支变量不在操作中途改变。

## 四、要回答的问题

**Q1（正推：代码做的是不是条款说的）**
1. 开关两档的算法与 E205 跑前登记第一节「甲并全零」「甲只并读坏」两条的定义逐句对。每一句都要判出对得上，或者给出对不上的那一句与代码行。
2. 产品入口在任何输入上走的都是今天那一档：读码给出证据链（调用点逐层），核「产品行为不变」。
3. 清扫改法与 D16 已定项 6 逐句对。另核：「错位读算自证不过、重读一次」与 `recovery::read_root_ring_slot` 的处置在同一形上是否一致。

**Q2（攻：哪一格会错）**
1. 甲的窗口 [N + 1, N + 3S − 1] 在边角上算得对不对：
   - S 不是 2 的幂；
   - N 是 0 或接近 u64 顶端；
   - 读得出的根只有第 0 代根；
   - 全部槽都读坏。
   - 每一种给出写行 txg，与登记定义比，算错的给反例盘面。
2. 开关有没有在挂载中途换档、或者漏到写行之外的路径（写行那次发布以外，还有没有别处取第一个 txg）。
3. 清扫改了之后，有没有一形合法历史（承诺内）让清扫比改前少删、多拒了本该删的行，比如真根恰在别槽的合法形。给出盘面或说明为什么没有。
4. 两边的用例与变异是不是测到了要测的东西：有没有一处改法没有变异罩着，有没有一条用例在改坏之后仍绿。

**Q3（只答一句）**：`instance_table_sweep.rs` 之外，`crates/singlefs-core/src/` 里还有没有读根环而不调落点核的地方（全文 grep `parse_slot` 与读根环的调用，逐处判它该不该核）。

不重复前几轮已攻过的角度：`research/prompts/m3-close-b19-e165-r1-main-verification.md`、`-r2-`、`-r3-` 里判过的形甲造不出、C971 在承诺外，以及 `m3-close-b19-r3fix-code-r1` 判过的按槽认盘三处。
