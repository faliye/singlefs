# m3-close-b19-r3fix-code-r1：批 19 第三轮判决零轮改法合入之后的代码轮（2026-11-05）

## 一、题面

第三轮判决 `research/prompts/m3-close-b19-reason-r3-main-verification.md` 第二节 Q4 第 3 问一行定了一条零轮改法：写者 `crates/singlefs-core/src/mount.rs` 里还有三处按 txg 认盘或认槽（判决写的是快照里的第 5044、5053、5275 行），改成按根所在那一槽的区域认盘，与准入那一处同一个口径（C953（写者与 checker 对第 0 代根记在哪块盘不一致） 的改法：写者读环时按每一份根所在那一槽的区域认盘，不按 txg 推）。实现员照它改完，补丁 `~/.local/share/singlefs/work/m3-close-b19-r3-impl-core/patch/crates.patch` 已打进主工作区。

这一批合入里只有这一处是产品代码（射程照 `.claude/rules/implementation-workflow.md`「三步，缺一步就不算做完」：`crates/singlefs-core/src/` 等四个包的 `src/` 走代码轮；池级 checker、harness 档、checker 档都是测试代码，不进这一轮）。同一批里 checker 一侧的改法（第五判违例文字、全零树表指针改报部分没判、C961、C962、C963 候选一）与 harness 一侧的胶水、故障注入、理想模型改法都是第三轮判决之后的零轮形态或测试代码，不在这一轮攻。

这一轮只回答一件事：**mount.rs 这三处改完之后，代码做的是不是条款说的，哪一格会错**。

## 二、实现今天的样子（主 agent 的观测，2026-11-05）

方案按 `crates/` 今天的实现来谈。行号是此刻主工作区的（`crates/singlefs-core/src/mount.rs` sha256 cb00d920…，两份补丁都已打；文件相对 HEAD 的差里还有别的会话与批 19 早先几份补丁的改动，这一轮只攻下面这几处）。

- `mount.rs:5013` `struct RootsReadForRaisingTheFloor`：抬 F 那一遍（准入与卸载共用）读出的根，现在带着读出它的那一槽（`roots_with_ring_slots`，第 4962–5007 行组装：先 `readable_roots_with_ring_slots_rereading_ring_slots_known_to_hold_a_root_once` 读，再按挂载时根环表补读坏的那几槽）。
- `mount.rs:5069` `ring_slots_pinned_since_the_mount_without_a_root_read_on_this_pass`（调用在第 5041 行）：算环里最旧有效根时，「挂载时钉着、这一遍没读出根」的槽，认法从「这一遍读出的根的 txg 推出的槽」改成「这一遍读出根的那一槽」。
- `mount.rs:5088` `roots_on_the_devices_of_their_ring_slots`（调用在第 5311 行）：正常卸载那一格当场放开的那一截（「释放代不高于每块盘上最新有效根的 txg 最小者」），每块盘上最新有效根改按根所在那一槽的区域认盘。
- 用例：`mount.rs:11162` 模块 `ring_slots_of_the_roots_read_for_raising_the_floor_tests` 三条（第 0 代根上按槽与按 txg 结局不同的三形，只在函数这一级造）。实现员报告（`~/.local/share/singlefs/work/m3-close-b19-r3-impl-core/result.md`「第 1 条的说明」）：真写者上两种认法算出的数相同（推的：暖机在准入与卸载之前已在每块盘落了本实例的根），没做整池用例。
- 变异行：`crates/mutations.tsv` 里「抬 F 那一遍按 txg 认盘」「抬 F 那一遍按 txg 认读出根的槽」「抬 F 那一遍读出根的槽不从钉里滤掉」三行（副本上 `prove-red.sh` 三行都抓到，任务 6526）；「RGall …」与 `第10b批 RR4 …` 两行换了锚点（`--item` 复判在主工作区排着，任务 6688）。
- 写者的落点公式：`crates/singlefs-core/src/root_ring.rs:172` `region: checkpoint_txg.0 % ROOT_RING_REGIONS`（txg 推区域）；mkfs 在三个区域的槽 0 都种第 0 代根（txg 0）。

## 三、条款

- D16（发布语义） 已定项 1（`.claude/kb/decisions/16-发布语义.md`）：「环里最旧有效根」一行、「抬 F 的上限」一行、「根槽这一次读坏」一行、「正常卸载」一行（每块盘上最新有效根、当场可用的那一截、挂载时读不出的槽并进最小值）。
- C953（写者与 checker 对第 0 代根记在哪块盘不一致）（`.claude/kb/checks-owed.md` 已还清表）：写者读环时按每一份根所在那一槽的区域认盘，不按 txg 推。
- 第三轮判决第二节 Q4 第 3 问一行与第四节。

## 四、要回答的问题

**Q1（正推：代码做的是不是条款说的）**：三处改完之后，「环里最旧有效根」「每块盘上最新有效根」在抬 F 那一遍与正常卸载那一格，按 D16 已定项 1 的字面逐句对：哪一句与代码对得上、哪一句对不上，对不上的给出最小的一段合法历史与代码走到的行。

**Q2（攻：哪一格会错）**：
1. 按槽认盘之后，有没有一形合法历史让写者在准入或卸载上与 checker（`crates/singlefs-checker/src/walk.rs` 按槽认盘那几处）结局不同？今天两边口径一致是推的。
2. `RootsReadForRaisingTheFloor` 里补读坏的那几槽（第 4994–4996 行一带）带的槽与它真正住的槽可能不同吗（重读、读坏不是全零、读回全零三种）？带错时三处哪一处会算错，结局是多扣还是少扣、会不会放开被引用的单元？
3. 「真写者上两种认法算出的数相同」这一句（实现员报告，推的）：给出它成立要靠的前提，逐个查今天的写者满不满足；有一个不满足就给出反例历史。
4. 三条函数级单测与三行变异是不是测到了要测的东西：变异改回按 txg，三条单测各红在哪；有没有一处改法没有变异罩着。

**Q3（只答一句）**：这三处之外，mount.rs 里还有没有按 txg 推区域来认盘或认槽的地方（全文 grep `% ROOT_RING_REGIONS` 与按 txg 求槽的调用，逐处判它该不该按槽）。

不重复前三轮已攻过的角度（第三轮判决第一节、第二节列的全零并、推空、准入那一处按槽认盘的结论）。
