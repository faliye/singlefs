<!-- knowledge-sync -->
# m2-supp3-item2 阶段同步

阶段：里程碑「第二个事务」增补 3 第 2 件（理想模型与模型对拍）落地、代码三方两轮改法与新增门禁 74 号这一批（2026-09-19 至 2026-09-20）。基准 `cc3e8ba`，结束在工作区与暂存区，时间 UTC。
触发文件：.claude/gate.d/74-model-differential.sh、crates/singlefs-checker/src/walk.rs、crates/singlefs-core/src/allocator.rs、crates/singlefs-core/src/mount.rs、crates/singlefs-core/src/transaction.rs、crates/singlefs-harness/src/history.rs、crates/singlefs-harness/src/lib.rs、crates/singlefs-harness/src/model.rs、crates/singlefs-harness/src/model_comparison.rs、crates/mutations.tsv、crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs、crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs、crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs、crates/singlefs-harness/tests/second_transaction_supplement_two_commit_generated_fallback.rs、crates/singlefs-harness/tests/second_transaction_supplement_two_unequal_devices.rs
工作区里另有别的会话未提交的改动（`.claude/rules-rationale/`、`CLAUDE.md`、部分 `.claude/agents/` 与 `.claude/rules/`、`.claude/gate.d/55-qemu-first-transaction.sh`、`research/e7-index-bench/**` 等），不属于这一阶段、不进这次提交；回扫员为此在只叠这一阶段触发文件的仓副本上生成候选表。

## 搜索

事实表（10 行，F1–F10）与方法：`research/prompts/m2-supp3-item2-sync-facts.md`；`python3 research/scripts/stale-candidates.py --check-facts` 退出码 0（10 行事实的检索词都在基准那一版的现状句里命中，新立事项除外）。
候选表：`research/prompts/m2-supp3-item2-sync-candidates.tsv`，`--facts` 生成 184 行（180 个不同的行），分在 F1、F2、F5、F9、F10 五组。
逐行判定：`research/prompts/m2-supp3-item2-sync-judge.md`，`--check-report` 退出码 0（184 行都有判定），反向核对另加 M1–M4 四行；判定分布：要改 6、要补 4、事件句 16、不相干 162。
主 agent 按判定种类随机抽 20 行复判（种子 20260920；要改、要补各抽 2 行，其余按比例），逐行读载体现在的原文：20 行都判对。「取样点」那一组命中多是变异取样点与几何取样点（`.claude/rules/mutation-sampling.md` 那一族），与随机历史的取样点无关，判不相干成立。`RollbackTargetNotInRing` 在 `.claude/kb/experiments/154-两道闸串-重判与回收时点的代价.md` 与 `records/2026-09-17-已分配口径三方与两个实验.md` 的两处，说的是那次核查当天源码实际报的名字，判事件句不改。没有哪一段要重派。
候选表的行号按基准那一版算；别的会话在这期间提交了两次（决策瘦身与新立 C420），改口径的六处按内容定位、不按行号。

## 命中处置

| 载体 | 原句 | 处置 |
|---|---|---|
| .claude/kb/checks-owed.md:34 | （2026-09-14 起独立解析器 + checker 有代码：`crates/singlefs-checker`；内存模型对拍与独立规约执行器仍零代码） | 改了：C25（三个 oracle 零代码）那一格补上「2026-09-19 起内存模型对拍有代码：`crates/singlefs-harness/src/model.rs` 与 `model_comparison.rs`，由门禁 74 号跑」，独立规约执行器仍零代码不变 |
| .claude/kb/checks-owed.md:306 | write buffer 前端启用；事务层 + 模型对拍 | 改了：C344 的前置列写明 74 号那五段取样点罩不到 write buffer，要为它加取样点——模型对拍从「还没有」变成「有了但射程有限」 |
| .claude/kb/checks-owed.md:311 | D11（索引节点要不要留消息缓冲区） 已定项 1 / 2 的节点缓冲启用（第一版 ε = 0）；事务层 + 模型对拍 | 改了：C349 的前置列同上，写明 74 号罩不到节点缓冲 |
| .claude/kb/verification-build.md:5 | 门禁的未实现清单今天剩模型对拍、QEMU 崩溃注入与 shell 命名纪律三条；崩溃点重放由 54 号覆盖，射程是那两条流；QEMU 真实负载由 55 号覆盖，射程只到第一个事务。 | 改了：未实现清单改成剩两条，加上「模型对拍由 74 号覆盖，射程是随机历史那五段取样点」 |
| .claude/kb/verification-build.md:237 | 覆盖写事务、第 2′ 步的 crash refinement、第 5 步的 RefFS 与模型对拍、第 6 步的 O3（独立规约执行器） 都还没有代码 | 改了：把模型对拍从「还没有代码」那串里拿掉，RefFS 与 O3 仍在 |
| README.md:70 | \| 模型对拍 \| 功能正确性：随机操作序列与内存里的理想模型比对。**还没实现**，门禁把它列在未实现清单里 \| | 改了：改成「由门禁 74 号跑：随机历史五段取样点，每一步拿实现的结局与只住内存的理想模型比」 |
| .claude/kb/milestone/02-second-txn.md:388 | 门禁 74 号查四段；第二轮在出材料。还没做：第 3–7 件。 | 补了：代码三方第二轮打中两处与五条改法落地、第五段取样点「小盘上逼近单元区墙」、第四段改成每一步跑池级 checker、四道重验证的结果（回扫员反向核对的 M1–M4） |
