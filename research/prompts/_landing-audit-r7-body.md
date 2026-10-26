# landing-audit-r7：子 agent 落地审计第十二批改动第一轮正文（2026-10-24）

<!-- doc-lint:not-numbers K1 K2 K3 -->

## 一、题面

第十二批（调度记录 `records/2026-10-22-子agent结果落地审计.md` 节「第十二批：九十节剩下的开着的行全关」）照用户「继续做完 不要欠账」把拆分提案第九十节（现住 `records/2026-10-23-subagent拆分提案.md` 第 906 行起）剩下开着的 ①–⑤、⑩、㉕–㉗、㉙、㉚、㉝–㉟ 全派了：任务 38 到 56，交回报告归档在 `research/prompts/subagent-landing-audit-2026-10-22/b12-*.md`，去向列已逐行写上。这是这一批改动的第一轮（`.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」第一轮分工：过度外推与越位、内部矛盾与循环依赖、哪一条最脆弱）。

1. 每一处改法做没做到它那一行的形态列，有没有越出那一行、漏改、把推的写成量的；第九十节去向列写的「做了」与交回报告、与文件此刻的样子对不对得上。
2. 新判法、新句子与已有规则、定义、钩子、门禁之间有没有矛盾，有没有两处判同一件事而判法不同（交回闸与派发闸、看门狗两判、三份定点写入脚本、合入后验证与主 agent 定义）。
3. 这一批新判法里哪一处最脆：哪一格换一个真输入就误拦或误放。

被判的文件：

| 类 | 文件 |
|---|---|
| 定义与共用约束 | `.claude/agent-common.md`（第 63、90、94、99 行那几条）、`.claude/main-agent.md`（「写与验分两段」验证段第 1、4、5 步与合入后验证命令写法里的 `--checker-quick-memory`）、`.claude/rules/three-way-inference.md`（「核查与判决由主 agent 做」第 1 步）、`.claude/agents/tooling-writer.md`（第 3 点） |
| 钩子与登记表 | `.claude/hooks/handback-guard.sh`、`.claude/hooks/runner-dispatch-guard.sh`、`.claude/hooks/lib_three_way_legs.py`、`.claude/hooks/lib_prompt_scratch_line.py`、`.claude/hooks/completion-cleanup-guard.sh`、`.claude/hooks/bash-command-detector.sh`（⑫ 与进程安全扫描两处）、`.claude/hooks/continuation-guard.sh`、`.claude/hooks/lib_writer_lock.py`（文件头）、`.claude/hooks/heavy-test-guard.sh`（文件头）、`.claude/hooks/selftest-registry.tsv`、`.claude/hooks/fixtures/runner-dispatch-guard/real-prompts/` |
| 门禁与登记 | `.claude/gate.d/doc-experiments.sh`（preregistration-parallelism、replay-comparison-units）、`.claude/gate.d/doc-process-records.sh`（deferred-items-registered）、`.claude/gate.d/doc-registries.sh`、`.claude/gate.d/code-tooling.sh`（break-switch-replay 格与 NOT_RUN_HERE）、`.claude/gate.d/deferral-registries.tsv`、`.claude/gate.d/break-switch-replay-files.tsv`、`.claude/gate.d/break-switch-outside-selftest-pending.tsv`、`.claude/gate.d/stage-owners.tsv`、`.claude/gate.d/lib-lag-baseline.py`（文件头）、`.claude/gate.d/lib/stage-cells.sh`（认样本根那一判）、`.claude/gate.d/checker-tier-crates-mutation-replay.sh`（文件头 `MUTATION_SHARD_RUN_BREAK` 那几行）、`.claude/long-run-live-hosts-exempt`、`.claude/heavy-thread-exempt` |
| 脚本 | `scripts/agents/dispatch-prompt.py`、`scripts/agents/agent-cost.py`、`scripts/agents/agent-watch.py`、`scripts/agents/watch.sh`、`scripts/agents/snapshot-tree.py`（新建）、`scripts/kb/stale-candidates.py`、`scripts/kb/replace-once.py`、`scripts/kb/insert-row.py`、`scripts/kb/replace-batch.py`、`scripts/kb/lib_notation_check.py`、`scripts/kb/claim-experiment.sh`、`scripts/kb/migrate-changelog-format.py`、`scripts/gate/hook-swap.py`、`scripts/gate/merge-and-verify.sh`、`scripts/hosts/detached-run.sh`、`scripts/mutation/mutate.sh`、`scripts/mutation/broad-mutation.py`、`scripts/mutation/mutation-shard-run.sh`；E171 编排目录 `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/` 里任务 53 改的 11 份；任务 51、52、54 只改了文件头开关表与失败行写法的那些份不判 |
| 记录 | `records/2026-10-23-subagent拆分提案.md` 第九十节去向列、一百一十五节、一百一十九节到一百二十一节；调度记录节「第十二批」 |

## 二、实现今天的样子（主 agent 的观测，2026-10-24）

方案按 `crates/` 与 `scripts/` 今天的实现来谈（主工作区现状）。这一批没有改 `crates/` 下的任何文件；`scripts/gate/merge-and-verify.sh` 新参数 `--checker-quick-memory` 管的那一步判的是 `crates/singlefs-checker-tier` 的快档测试，键与复用由门禁 `checker-tier-crash-point-replay.sh` 算。

- 主 agent 在主工作区重跑过的自证读数，逐件写在调度记录节「第十二批」各条交回下面（交回闸 117 种、派发闸 609 种、骨架 110 项、`heavy-test-guard.sh` 1442 种、检出钩子 799 种、`merge-and-verify.sh` 52 种，任务 51 改过的 20 份、任务 53 的 E171 各层与单测、`stage-cells.sh` 25 条都退 0）；任务 51 到 56 的开关复跑两台各跑两趟，崩溃放量、崩溃点重放、变异分片、广谱变异与任务 51 的 20 份逐格相同、已登记，`agent-watch.py` 剩一百二十一节那处竞争、没登记。
- 交回报告给出、主 agent 没复算的真会话记录读数：交回闸 3195 份交回新放 0 新拒 165；派发闸 3394 份新放 0 新拒 4，3484 份新放 0 新拒 1545（10-14 起 192 份）；草稿目录库 4006 份 0 份不同；账表三方腿两桶 699 → 647、133 → 4。
- 没跑过的：门禁阶段本身（样本档要样本与暂存区相同，这一批没暂存，用户定这一批不提交）；门禁 code-tooling 的 break-switch-replay 格在真仓上整格（主 agent 与工具员用同一份判法的草稿复跑工具逐条跑过）。
- 主 agent 自己改的：共用约束第 94 行（交回正文闸加 Bash 写 `.rs` 与 ⑦）、`.claude/gate.d/lib-forbidden-notations.py` 的会话名排除、两张豁免表删 E171 行、`checker-tier-crates-mutation-replay.sh` 文件头三处、调度记录几处延后行改写、第九十节去向列。
- 别的会话这几天在同几份文件上的改动不在被判之列：`doc-process-records.sh` 的两份拆分提案都认、`changed-paths.sh` 新加的格与开关、`replay.sh`、`crash-amplification-lib.sh`、`multi-host-run.sh`、`run-with-memory-cap.sh`、`resource_release_broadcast.py`。腿判到它们时写明是别人的改动。
- 暂存区里有一截不是本会话暂存的改动，快照对 HEAD 取 diff。

开工快照：`research/prompts/landing-audit-r7-snapshot/judged-sha256.txt`、`research/prompts/landing-audit-r7-snapshot/judged-vs-head.diff`。

## 三、格

### K1　每处改法做没做到它那一行（正推）

- 对着第九十节 ①–⑤、⑩、㉕–㉗、㉙、㉚、㉝–㉟ 每一行的形态列与去向列：改法落在哪一份文件哪一句、哪一个判法，做到的写依据（文件:行），没做到、只做了一半、越出那一行的写哪一格；去向列写「做了」的，交回报告与文件此刻对不对得上。
- ⑤ 不做的六条（B8、B9、D3、E2、C4、A5）各自的依据站不站得住；⑩ 判不做的依据（上游查重脚本按会话记录认写法那一支只给没注册的收工钩子用）现查成立不成立。
- 交回报告与调度记录里写「推的、没量过」的（`--checker-quick-memory` 缺省、任务 49 的收窄代价「N 核」与单写一台的线程数不判），这一批有没有当成量的写进别处。

### K2　内部矛盾与越位（攻方）

- 交回闸 ⑤ ⑦、派发闸 ⑭ ⑮ ⑯ ⑰ 与新加的「复用：」「批次：」「报告：」几判之间，与 `.claude/agent-common.md`、`.claude/main-agent.md`、各份定义今天的句子之间，有没有一处要求、另一处拒；一份真提示照骨架写能不能同时过所有判。
- `scripts/agents/snapshot-tree.py` 与 `.claude/rules/three-way-inference.md` 第 1 步、`.claude/rules/path-moves.md` 第 6 步说的是不是同一件事；它倒推不出、sha256 对不上的那几份，判决的人照规则能不能知道。
- 门禁 break-switch-replay 格「点名与实际红的格逐格相同」与各份脚本自带的开关轮（`crash-amplification-lib.sh`、`crash-point-replay.sh` 读自己文件头的开关表）判法是不是一套；deferred-items-registered 格与 `.claude/main-agent.md`「一轮怎么开、怎么收」第 2 条说的登记处是不是同一组。
- 看门狗「进程常驻内存过线」排掉登记成轻的自证底下的进程，与「重型测试没带前缀」那一判共用认法之后，有没有把该报的真过线进程也排掉。
- `stage-cells.sh` 认样本根改成「`.gate-break-sample` 第一行逐字写那一句再接这个根的 `pwd -P`」之后，别的门禁在仓里、`--staged` 树里、别人造的小仓里设了开关，还拒不拒；有没有一种写法让平常跑的门禁也认开关。

### K3　哪一条最脆（攻方）

- 这一批新判法里换一个真输入就误拦或误放的那一格：先在主会话记录（`~/.claude/projects/-home-fy5090-code-singlefs/*/subagents/`）、真派发提示、真交回报告里找；报出来的每一处带输入原样与复跑命令。
- E171 编排改成每台取登记线程之后，修订 11 那几层的槽数与线程合计会不会越过登记值（修订 8 那两处已登记在拆分提案一百一十九节，不重复报）。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1 | 逐行核每处改法做没做到、去向列对不对得上 |
| 云端攻方（Opus） | K2、K3 | 找矛盾与越位，找最脆的那一格 |
