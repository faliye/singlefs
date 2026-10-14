# 代码欠账批甲簇（C572、B3）：代码轮第一轮正文（2026-10-14）

<!-- doc-lint:not-numbers B3 K1 K2 K3 K4 -->

## 一、题面

这一轮攻代码欠账批甲簇一份补丁（`~/.local/share/singlefs/work/code-b2-mount/patch/`），2026-10-14 已三路合进主工作区。同一批的乙丙丁簇（`code-b2-tests`）与线程缺省（`code-b1-thread-defaults`）只改测试代码、harness 档与 checker 档，或只改注释（`crates/singlefs-core/src/transaction.rs` 经 `comment-only-change.py . HEAD` 判只改注释），不开代码轮。射程里的产品代码只有 `crates/singlefs-core/src/mount.rs` 一份。

| 关的 | 落点 |
|---|---|
| B3：崩溃恢复抛弃的根那一段，影子账的豁免与判重按落点起点槽认，改成按落点罩着的每一槽认（五处） | `crates/singlefs-core/src/mount.rs` 的 `isolate_slots_referenced_only_by_abandoned_roots`（第 1557 行起），新私有辅助 `SPAN_OF_ONE_SLOT`（1723）、`slots_covered_by_a_placement`（1730）、`slots_of_a_placement_to_isolate_now`（1744）；用例六条在同文件末尾 `#[cfg(test)] mod shadow_ledger_exempts_and_isolates_slot_by_slot_tests`（9677） |
| C572：管理员回退的目标引用隔离单元时，隔离单元也逐盘验，有一份读不出或对不上就在任何写之前拒 | 同文件：`RollbackError::IsolatedUnitCopyUnreadableOrMismatched`（7521）、`resurrection_against_the_current_account`（7738）、`every_copy_of_the_isolated_units_the_target_references_reads_back`（7846）、`first_copy_that_does_not_read_back`（7884）、`roll_back_by_a_forward_publish`（8109）；harness 两份穷举 match 各补一臂（`crates/singlefs-harness/src/history.rs`、`model_comparison.rs`），新用例文件 `crates/singlefs-harness/tests/rollback_by_a_forward_publish_verifies_every_copy_of_an_isolated_unit_the_target_references.rs` |

条款与依据：`.claude/kb/decisions/23-journal的角色与格式.md` 已定项 14（第 355 行起，回退段「只隔离其中只被被抛弃根引用的槽」与「在任何写之前拒」那一格）；`.claude/kb/decisions/19-块指针的结构与宽度预算.md` 已定项 5（第 95 行起，硬规则 1）；`.claude/kb/checks-owed.md` C572（第 377 行）；用户 2026-10-12 定 C572「隔离单元也进逐盘验」（`records/2026-10-12-里程碑三非代码收尾.md` 第 65 行）；B3 的来处是 `research/prompts/m3-c554-c572-r1-opus-output.md` 第 154–189 行与那一轮判决 `research/prompts/m3-c554-c572-r1-main-verification.md`。

改动全文：`research/prompts/_code-backlog-b2-code-r1-diff.md`（`mount.rs` 相对 HEAD f111c3f3 的全部改动，含全仓改名带进来的注释改动；harness 三份；追加 6 行、整行替换 8 行变异）。实现员报告：`~/.local/share/singlefs/work/code-b2-mount/report.md`（改了哪几处对条款哪一句、用例改前改后原样输出、证红六行）。

## 二、实现今天的样子（主 agent 的观测，2026-10-14）

- 冻结副本 `~/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/`（主工作区合入之后的 `Cargo.toml`、`Cargo.lock`、`.cargo`、`crates/`、`litmus/`、`.claude/heavy-thread-default`）。被判文件、harness 三份、`crates/mutations.tsv` 与引到的三份 kb 文件的 sha256 在 `research/prompts/code-backlog-b2-code-r1-snapshot/crates-sha256.txt`。代码行号按冻结副本。
- 主工作区里 `mount.rs` 相对 HEAD 改了 704 行（增删行数），其中 692 行是甲簇补丁，其余是全仓改名换掉的注释词（旧名登记在 `.claude/kb/term-renames.md`）。
- 别的会话合入之后跑过：`cargo check --workspace --all-targets` 过、`cargo clippy --workspace --all-targets -D warnings` 零警告、除 checker 档外六个包测试全绿（core 255 条，harness 1735 条）（`records/2026-10-13-复用键与全仓改名.md`「合入」一节）。实现员副本里：B3 六条改前全红、改后全绿；C572 两条，去掉新加那一步时 `expect_err` 红；六行变异 `prove-red` 都抓到（报告「变异与证红」一节）。
- 实现员自定、主 agent 接受、被攻过零轮的两处（报告「设计上要主 agent 看的两处」）：B3 用例放在 `mount.rs` 的 `#[cfg(test)]` 里直调私有函数，没有经公开入口造的 harness 用例；「cur 那一版不引用」用 `current.units` 的槽判，实现员推的是「cur 还引用而不在 `units` 里的单元会被多验一次，方向是多拒」。
- 已知、记作欠账、报出来不算打中的：`.claude/kb/checks-owed.md` C554（第 365 行）；`m3-c554-c572-r1` 判决里留给用户的岔路。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　B3 五处按槽展开，撑不撑得住「只隔离其中只被被抛弃根引用的槽」
①② 两份豁免集合与 ③④⑤ 三圈逐槽判重：同一起点、跨度不同的两份落点，跨度大的那份多出来的槽有没有一槽在任何一圈里漏隔离或误隔离；豁免先于判重的次序换了会不会改结局；`slots_covered_by_a_placement` 末槽的 `checked_add` 与 `expect` 依赖「进影子账的落点都判过落在单元区里」，这一条在五处各自的来路上成不成立；分配器一侧逐槽隔离对同一槽幂等，五处交给它的槽序列在两块盘上是否一致。六条用例直调私有函数：它们造的三种形与 `m3-c554-c572-r1` 攻方报告里的三形是不是同一件事，经公开入口（可写挂载、抬 F）能不能走到一条用例没罩的形。

### K2　C572 隔离单元怎么认
「cur 的账里两块盘上都仍分配、同分配代，而 cur 那一版的单元里没有一个落在它的槽上」是不是恰好等于条款说的「被隔离的那些」：有没有隔离单元因为 cur 那一版恰好引用同一槽而漏验（漏拒），有没有不是隔离来的单元被多验（多拒，例如 `deleted_inodes` 队列里还没走完的 inode 的单元在不在 `current.units` 里）；两条位置项不同槽时怎么认。

### K3　C572 拒在什么时候
新的逐盘验排在复活集逐盘验之后、动分配器之前：从 `roll_back_by_a_forward_publish` 进来到新那一步之间有没有任何盘上写或分配器状态变化；两种拒因同时成立时报哪一个，与条款「在任何写之前拒」与已有的拒因次序合不合；`first_copy_that_does_not_read_back` 抽出来之后，复活集那一判的读法、次序、拒因是不是逐字不变。

### K4　harness 补的两臂与用例、变异
`model_comparison.rs` 两处把新成员判成 `Unexplained` / `None`、`history.rs` 一处给成员名字串：随机历史与故障注入里新拒因会不会被当成不该有的分歧，模型那一侧有没有对应的预期结局；新用例文件两条（拒与阳性对照）造的隔离是不是条款说的那种隔离、盘上不动是怎么核的；追加 6 行变异的替换文改的是不是点名的那一处行为、换一个取样点还红不红（`.claude/rules/mutation-sampling.md`「没红的三类，判据不同」），整行替换 8 行改的锚在冻结副本里是不是恰好命中一次。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1–K4 | 核「代码与测试做的是不是条款说的那件事」，逐条对条款原文与冻结副本的行 |
| 云端攻方（Opus） | K1–K4 | 在冻结副本的拷贝上造反例：漏隔离或误隔离的一槽、漏验或多验的单元、拒之前已经写了的一步、用例在错的理由上绿、变异改坏照绿；读码的结论尽量换成造出来的用例 |

## 五、交付要求

- 报告先写进 `research/prompts/code-backlog-b2-code-r1-<腿名>-output.md`（正推 `-sonnet-output.md`、攻方 `-opus-output.md`），每一次工具调用写进文件的内容不超过 150 行，分段追加，第一段排他新建，回复只写短句；早点开写。
- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。引代码写冻结副本里的路径与行号。
- 草稿放腿自己的目录 `~/.local/share/singlefs/work/code-backlog-b2-code-r1-<腿名>/`；量法拷冻结副本，不在冻结副本里改；建副本与编译目录之前先查自己草稿目录里有没有不用的，先删。模型与驱动随报告拷进 `research/prompts/code-backlog-b2-code-r1-<腿名>-model/`（含 `tools/rerun.sh`，从 `scripts/agents/three-way-rerun-template.sh` 拷来、只换 `rerun_steps` 那一段；复跑经 `scripts/hosts/run-with-memory-cap.sh`）。
- 每条编译与测试命令经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 16G 的上限，同一时刻只起一条 cargo；编出来的测试二进制也经它跑；用自己的编译目录，几份变异副本各用各的。前台命令预计超过 4 分钟的拆成小段。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起），不跑 checker 档测试，不起 GPU。
- 新写的文字不用「层 0」「第一段 / 第二段」「第一条流 / 第二条流」这几种旧名（`.claude/kb/term-renames.md`）。
