# 书记员规格：里程碑二遗留收拢的 kb 一批（2026-09-19）

依据：`records/2026-09-19-里程碑二遗留收拢.md` 第二、三节；五份重核 `research/prompts/m2-closeout-recheck-a.md` 到 `-e.md`；主 agent 现查（下面每条写明）。不记决策变更史（这一批没有改决策正文）；不翻分项状态。

## 一、`.claude/kb/checks-owed.md` 原地改写（四处）

旧行与新行逐字在 `research/prompts/m2-closeout-kb-scribe/owed-spec.json`里，前四组：[说明, 旧整行, 新整行]；第五组是第四节的 C381。

1. C77：「怎么拦」那一格补「2026-09-18 起跨实例那一形已有……还欠同一实例内的那一形与撕裂注入的旗标」。依据：`research/prompts/m2-closeout-recheck-b.md` 第 22 行；变异第 97 行两次复跑都红（`research/prompts/m2-supp3-item1-r2-crash-verifier/59.log`）。
2. C42：前置「只有正例（实现员交回、待代码三方）」→「只有正例（代码三方 `m2-wave1-code-r1` 攻过，判决第 14 行 Y5 没算打中）」。依据：`research/prompts/m2-wave1-code-r1-main-verification.md` 第 14 行。
3. C143：前置「另一半交用户，见……第 10 行」→ 已答。依据：`.claude/kb/decisions-history/2026-09.md`「2026-09-06（其四十一）」，主 agent 现查。
4. C378：整行改写成剩下的题面（简称不动）。依据：`research/prompts/m2-closeout-recheck-d.md` 第 20a 行与「表外」第 10 条。

## 二、还清两行（从开着的表挪进「已还清」表）

在开着的表里删掉 C22 那一整行与 C369 那一整行；在「已还清」表里 C382 那一整行之后插入下面两行（四列：编号、简称、还清说明、日期）：

```
| C22 | 刚释放的块立即重分配 | 2026-09-19 核清：① 镜像侧 I-4.8（近 K 代根校验和自洽） 由池级 checker 判（`crates/singlefs-checker/src/walk.rs`）；② 故障注入：`crates/singlefs-harness/tests/second_transaction_step_five_reuse.rs` 那条用例把复用窗口压成 0（`reclaim_released_up_to(…, ReclaimedReuse::Immediately)`）并断言 I-4.8（近 K 代根校验和自洽） 红，`crates/mutations.tsv` 第 110 行（I-4.8 判定恒真）在门禁 59 号里红在它上面。依据：`research/prompts/m2-closeout-recheck-b.md` 第 22 行 | 2026-09-19 |
| C369 | 提交内生块段耗尽时没有回落 | 2026-09-17 实现、2026-09-18 过代码三方第一轮（`research/prompts/m2-wave1-code-r1-main-verification.md`；回落那一格另起里程碑「第二个事务」增补 2 收口表第 20c、20d 行）：`DeviceFreeMap::lowest_commit_generated_fallback_slot`，用例 `second_transaction_supplement_two_commit_generated_fallback.rs` 四条，`crates/mutations.tsv` 第 86–94 行在门禁 59 号里全红。C146（无空段时的回落政策全仓无定义） 第 ② 条随之还清一半，第 ① ③ 条（整理）仍开着。依据：`research/prompts/m2-closeout-recheck-b.md` 第 20 行 | 2026-09-19 |
```

## 三、新立三笔（开着的表，插在开着那张表的最后一行之后；2026-09-19 现查那一行是 C392（双轨翻转入口的穷举表没推），写之前再查一次，别的会话又加了就接在它加的最后一行之后，报告里写明接在哪一行之后）

取号：写之前现查欠账表的最大编号，取下三个；下面按 C385、C386 写，已被占就顺延（2026-09-19 现查最大号已到 C392，所以多半是 C393、C394），并在报告里写明取到的号（主 agent 据此改里程碑收口表）。

```
| C385 | 被抛弃根的账读不出时修复没有条款 | 回退之后被抛弃的根的树表或分配记录树读不出时，今天只计数、不拒绝挂载（`crates/singlefs-core/src/mount.rs` 的 `abandoned_roots_unreadable`），用户否决了跳过、要走修复，而「修复」指什么动作全仓没有条款；`crates/` 里没有 scrub 或自愈（`grep -rn -i "scrub\|repair\|self_heal\|自愈\|修复" crates/*/src/` 只命中注释） | 条款定下「修复」是什么动作之后：造一段回退之后被抛弃根的分配记录树两份都读不出的历史，按条款断言挂载的结局；判别力自证：把修复那一步拿掉必须红 | 条款要先定「修复」指什么动作（三方）；实现要按 (区域, 槽) 让读返回失败的只供测试的开关（里程碑「第二个事务」增补 2 收口表第 24 行） | 里程碑「第二个事务」步 4 现状与增补 2 收口表第 ③ 行；2026-09-19 重核（`research/prompts/m2-closeout-recheck-a.md` 第 ③ 行）立账 |
| C386 | 释放判定不核映射条目位置项里的单元校验和 | 从盘上重建上一版时，释放判定路径（`crates/singlefs-core/src/transaction.rs` 的 `placements_to_release_via_mapping`）只核「映射里查得到 key、分配记录在册、没释放过、跨度对得上」，不核映射条目位置项里带的单元校验和；`rebuild_version`（`crates/singlefs-core/src/recovery.rs`）读映射条目只取 key，位置项与它带的校验和丢掉 | 条款定了「释放之前要不要按位置项里的校验和核盘上单元、核不过怎么办」之后：造一份映射条目位置项与盘上单元不符的镜像，释放按条款的结局；判别力自证：把那一核拿掉必须红 | 条款（三方，里程碑「第二个事务」增补 2 收口表第 13 行） | 步 1 / 步 2 代码三方第二轮攻方腿 Y1 的射程（`research/prompts/m2-code-r2-main-verification.md` 第 42 行）；2026-09-19 重核（`research/prompts/m2-closeout-recheck-a.md` 第 13 行）立账 |
| C387 | 代字段只有验收断言盯着 | 按 C374（释放代与树表诞生 txg 只有验收断言盯着） 的判据回扫盘上代字段（`research/prompts/m2-generation-fields-sweep-report.md` 第二节），不过 9 项：根记录与 journal 记录带的回退下界 F（只被别的不变量当输入）、单元头与点名项的诞生代号（I-1.2（块头写序已发布）、I-1.8（归并后版本全序）、I-3.5（引用区间的精确性）、I-3.6（deadlist 紧界） 在册而没实现）、树表条目 `previous_snapshot_txg`（`crates/singlefs-core/src/records.rs` 恒写 0、读出即丢）、已分配记录的分配代（里程碑「第二个事务」增补 2 收口表第 44 行）、记账代、inode 改动计数（类型是裸 `u64`）、实例表行 T、父指针缓存的出生 txg 与写序与子单元头是否一致；另 I-3.9（释放代落在停止引用它的那一格区间里） 在「某条候选根的引用集合走不完」时整条报不适用，没有坏镜像罩着「那时另有一处释放代真错了」（同一报告第三节） | 每项先有一条不变量说它该取什么值，再由池级 checker 判：一份只红在它身上的坏镜像，`crates/mutations.tsv` 一条「判定恒真」变异在门禁 59 号里红；I-3.9 那一处补一份组合坏镜像（一条非最新候选根读不全、最新根上释放代写错），必须红而不是报不适用 | 不变量措辞（主 agent 逐条给，书记员写进 `.claude/kb/invariants.md`）；报告里「分不清」的四项（超级块槽世代号、根记录与 journal 记录自身的 txg、实例表行 W）先判算不算这类代字段 | 2026-09-19 回扫（`research/prompts/m2-generation-fields-sweep-report.md`，里程碑「第二个事务」增补 2 收口表第 12 行的回扫那一步）立账 |
```

## 四、C381 前置那一格（第五组，同在 `owed-spec.json`）

依据：`research/prompts/c381-r1-main-verification.md` 第四、五节。原地替换，照第一节的做法。
