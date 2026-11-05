# m3-close-b19-reason-r1：C717 丁的重建前提与 I-3.1 取宽条件（2026-10-30）

## 一、题面

批 19 两位实现员交回时各留了一格零轮的设计判断，登记成 C934（I-7.9 按 journal 重建抬 F 那一刻的根环靠一条推的前提）与 C937（I-3.1 照 I-3.18 改记槽数取宽的条件被攻过零轮），欠账表两行都写「开一轮推论攻」。前两轮（`research/prompts/m3-close-b19-checker-r1-main-verification.md`、`research/prompts/m3-close-b19-design-r2-main-verification.md`）判的是条文方向：C717（回退下界上限的单窗口盲带）走丁由用户定（丁：借 journal 新根段重建抬 F 那一刻的根环再判上限），I-3.1（已分配统计对得上）取宽由第二轮 Q9 攻方的条件定；这一轮攻的是实现员照判决写出来的**判法本身**：前提成不成立、放过的方向漏不漏。

## 二、实现今天的样子（主 agent 的观测，2026-10-30）

方案按 `crates/` 今天的实现来谈。行号是此刻主工作区的（批 19 全部补丁已打，第六趟合入后验证编译与测试全绿）。

**C717 丁，池级 checker（`crates/singlefs-checker/src/walk.rs`）**

- `judge_rollback_floor_raises_against_their_ceilings`（9334 起）：不带卸载记号的抬 F 根先判 `window_lowest_ceiling_is_the_lowest_ceiling_of_the_ring_at_the_raise`（8856 起）：每个区域所在的盘上窗口里都有有效根，且去掉窗口里最旧那条之后判得了空不空的非空有效根去重后不少于 `NON_EMPTY_VALID_ROOT_RANK_THE_CEILING_TAKES`（8754，值 4）；两样都成立照旧按窗口判，不成立走丁。
- 丁 `rollback_floor_against_the_journal_reconstruction_of_the_ring_at_the_raise`（9246 起）逐槽调 `root_in_the_ring_slot_at_the_raise`（8986 起）认抬 F 那一刻那一槽住的是谁：镜像上坐着 txg 比抬 F 那条根 r 小的根就是它；全零算空着；读不出认不出；坐着 txg ≥ r 的根（被盖过）时取落点式在 r 之下的最大正 txg u（`largest_positive_txg_the_ring_slot_names_below`，8963 起），在 journal 认出的版本里找 u 带提交标记的记录，恰好一个实例的才往下认：u 与 r 同一实例，或 u 那个实例在 r 的实例表里的行 T > u，认作那一刻住着 u；u ≥ T、那个实例没有行、journal 里没有 u、几个实例都有 u，都认不出；只认一层，不往 u − 3S 推；被盖掉的第 0 代根在抬之前的 F 为 0 时认不出。任一槽认不出，整条报部分没判。认出的根照「txg ≥ 抬之前的 F、按 r 的实例表不被抛弃」滤出有效根，再过 `rollback_floor_ceiling_of_the_valid_roots`（8770 起）同一个上限式子。
- 函数文档（8976 起）自己写明「实例中间的一版：之后那个实例还发布过，它的根落了盘」是推的。

**C717 丁，GPU 扫描内核（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl`）**：`c_root_in_the_ring_slot_at_the_raise`（1796 起）与 `c_rollback_floor_ceiling`（1697 起）照 CPU 同判；对拍用例 `crates/singlefs-checker-tier/tests/crash_judge_gpu_rollback_floor_ceiling_rebuilt_from_the_journal.rs`（单实例三张镜像与跨实例镜像）。

**写者在根槽写失败时怎么办（`crates/singlefs-core/src/transaction.rs`）**

- `persist_the_publish_or_freeze_it`（1563 起）：落盘那几步（写单元、屏障、journal 记录、根槽、系统配置槽）里任何一步失败，这次已记的写进失败账（`count_failed_publish`，694 起），分配器换回发布之前的样子，这次发布冻结（`FrozenPublish`）；下一次发布之前先 `resend_the_frozen_publish`（1594 起）逐字节原样重发（checkpoint_txg、计数器、序号、位置都不变），再失败照旧冻结着。D23（journal 的角色与格式） 已定项 14「这一版的失败处置：发布不接受失败」。
- 零单元发布（树表 0 条的一版上写行，`publish_without_units`，1657 起）失败不冻结，错原样交回调用方。
- 写者算准入抬 F 的上限时，写失败的槽按旧内容算（D16（发布语义） 已定项 1「环里最旧有效根」一行）。

**恢复怎么定新实例的 T 与坏槽的钉值（`crates/singlefs-core/src/mount.rs`）**

- `first_txg_of_new_instance`（1403 起）：新实例第一次发布的 txg = max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1（D23（journal 的角色与格式） 已定项 14 第 3 条）。所以「u 的记录落了、u 的根没落」的崩溃态之后新实例的 T = u + 1 > u。
- 挂载时读坏的根槽按它可能住过的最旧 txg 钉住（2924 起的文档与 2950–2975 的算法；`crates/singlefs-core/src/allocator.rs` 1758 起 `oldest_bad_slot`），D16（发布语义） 已定项 1「根槽这一次读坏」一行。

**I-3.1 取宽（`crates/singlefs-checker/src/walk.rs`）**

- `records_with_the_spans_the_span_invariant_reddens_set_to_their_unit_classes`（10360 起）：最新根里每条**未释放**记录，起点槽上的单元头用得了而那一类的槽数 `!=` 记录自述的跨度（10372，两个方向都改），改记成那一类的槽数；一条都没改交回 `None`。
- `allocated_statistic_lies_within_the_bounds_of_either_span_reading`（10391 起）：记录自述与改记两份界各取宽的那一边——(L) 取两份 |E(θ_high)| 的小者、(P) 遍历走到的每个槽至少在一份 E(θ_high) 里、(U) 取两份 |E(t)| 的大者再与 |Ω ∪ U| 取小；取宽后成立报部分没判、点名 I-3.18（分配记录的跨度等于它罩住的单元那一类的槽数），仍不成立判红。条文在 `crates/singlefs-checker/invariants/05-空间记账.md` I-3.1 一行；I-3.18 在同文件第 25 行；I-5.4（分配记录罩住的槽互不相交） 只看记录之间的重叠。单测 `record_wider_than_its_two_slot_unit_is_left_to_the_span_invariant`（25687 起）钉「记录比单元宽」那个方向也放过。

## 三、格

### Q1　C934：「实例中间的一版根槽一定落了盘」这条前提

判法认 u 落了盘只靠「u 与 r 同一实例，或 u 那个实例的行 T > u」。要攻的是：**在写者今天的实现上，有没有一段合法历史，u 带提交标记的记录在 journal 里、u 那个实例之后还有版本或恢复给它的行 T > u，而 u 的根从没在它那一槽落过盘**。分三问：

1. 写者在根槽写失败时实例停不停。今天是冻结后原样重发（`persist_the_publish_or_freeze_it`），同一实例下一版发出之前 u 必须重发成功。要答：重发「成功」而字节没落到盘（写返回成功、崩溃前没持久；坏槽、被钉的槽）走得到吗；零单元发布不冻结那一支，失败之后同一实例接着发下一版走得到吗；D16（发布语义） 已定项 1 提到的「一次处置里根槽写失败 ≤ k_tol 次」在写者里对应哪一段代码、那一段跳不跳过失败的那一版。
2. 崩溃态里「u 的记录落了、u 的根没落」，下一次挂载是新实例，`first_txg_of_new_instance` 给它 T = u + 1 > u。这一形重建会把 u 当成那一刻的住户，而真住户是上一圈的根或空槽。要答：这一形在今天的写者与恢复上造得出造不出（第一个崩溃点重放用例的形态写出来）；造得出时重建出的上限与真上限谁高——主 agent 推的（没量）：重建多认了一条更新的根，上限只会不低于真上限，方向是漏红而不是误红；要腿按 `rollback_floor_ceiling_of_the_valid_roots` 的式子逐项核这个方向，并给出一张「坏写者把 F 抬进这一条缝里、丁判成立」的盘面，或证明这条缝为空。
3. 钉槽 p（D16（发布语义） 已定项 1「根槽这一次读坏」一行与 `mount.rs` 2924 起的钉值）能不能把第 1、2 问的形认出来：读坏的槽重建时报认不出（`RingSlotOfTheImageSeenFromTheRaise` 读不出那一支），但第 2 问的槽读得出、坐着更新的根。

要交的：每一问一个结论（造得出 / 造不出 / 推的），造得出的给盘面与历史（写者动作序列、崩溃点），造不出的给写者代码里挡住它的那一行。

### Q2　C934：三处保守简化各让「部分没判」多出多少

只认一层不往 u − 3S 推；u 是行 T、被抛弃、实例没有行都认不出；被盖掉的第 0 代根在抬之前 F = 0 时认不出。要在第二轮两腿模型（`research/prompts/m3-close-b19-design-r2-opus-model/`、`research/prompts/m3-close-b19-design-r2-sonnet-model/`，推的模型、各 20 000 段合法历史）那一套口径上各量一次：三处分别放开（照模型里「两种都算、结论一致才判」那一支）与不放开，部分没判的比例各差多少。第二轮丁那一格报的 5.1% / 2.2%（攻方模型）与 J ≥ 24 时判红 93.0% / 94.2%（辩方模型）是没有这三处简化时的数。腿建的模型放 `research/prompts/m3-close-b19-reason-r1-<模型>-model/`，数的口径与第二轮同一份。

### Q3　C937：两个方向都放过，漏不漏一块没有别的不变量红的畸形

取宽把「记录比单元窄」（跨 1 槽罩 2 槽的单元：后一槽盘上没有归属）与「记录比单元宽」（跨 3 槽罩 2 槽的单元：多罩了一槽）两个方向的记录都改记成单元那一类的槽数再算界。I-3.18 在两个方向上都红，所以第二轮 Q9 的条件（「配不上的那块畸形若没有别的不变量判红，那一处判红、不报部分没判」）字面上都满足。要攻的是：**有没有一张镜像，I-3.1 按记录自述三判不成立、取宽后成立、而把 I-3.1 退成部分没判之后镜像上的真病（统计漏了或多了的那几槽）不被 I-3.18 的红说明覆盖**——例如宽的那一条记录多罩的槽恰好被另一条记录合法罩住（I-5.4 红），或多罩的槽上有别的单元被遍历走到（(P) 两份里至少一份含它就放过）；或者窄的那一条少罩的槽被别的记录罩住、统计多算。只放「记录比单元窄」一个方向（10372 的 `!=` 改 `>`）够不够：造一张只在「宽」方向上漏判的镜像，或证明「宽」方向上取宽之后成立的每一种盘面都已被 I-3.18 或 I-5.4 的红完整说明（真病一个字不少）。两腿都给出：两个方向各一张取宽后成立的最小镜像，逐条列那张镜像上红的不变量与它们的违例说明，判说明有没有把真病说全。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 正推腿（Sonnet） | Q1–Q3 | 从写者、恢复与判法的代码推到结论：逐问给造得出的历史或挡住它的那一行；Q2 建模型量三处简化的代价；Q3 给两张最小镜像与逐条不变量的说明 |
| 攻方腿（Opus） | Q1–Q3 | 找反例：Q1 造「记录在、根没落、T > u」的历史与坏写者钻缝的盘面；Q2 攻第二轮模型的口径对这三处简化适不适用；Q3 造取宽后成立而真病没被说全的镜像；不重复第一、二轮攻过的角度（根环逐条相同的坏写者、journal 回绕、W 的判法） |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数；引代码写开工快照里的路径与行号。
- 方案按 `crates/` 今天的实现来谈；要建小模型核的，模型放 `research/prompts/m3-close-b19-reason-r1-<模型>-model/`，草稿放腿自己的目录（`~/.local/share/singlefs/work/<腿名>/`），不在主工作区改。
- 不跑重型测试、不起 checker 档的 cargo、不碰 GPU；小模型经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 4G 的上限。
- 报告写进 `research/prompts/m3-close-b19-reason-r1-<模型>-output.md`，分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句。
