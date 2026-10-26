# landing-audit-r7 判决：子 agent 落地审计第十二批改动第一轮（主 agent，2026-10-24）

正文：`research/prompts/_landing-audit-r7-body.md`。两条腿：正推（Sonnet，K1）`research/prompts/landing-audit-r7-sonnet-output.md`（sha256 `3045f40114afe7ea15bc3943edc7d693367a55364d92831642939147f38c9327`）、攻方（Opus，K2、K3）`research/prompts/landing-audit-r7-opus-output.md`（sha256 `417c40a23a5adfd4fe2120f8937753c4cce8416faffe43d5b95d7fa727b2c209`）。两条腿都没写模型目录，复跑工具与输入留在各自草稿目录。

## 这一轮被判过的文件

- `.claude/agent-common.md`（第 63、90、94、99 行那几条）：第 63、99 行与交回闸同一种落盘写法，正推核对上；第 90 行派发闸那一条的判据「本机与第二台成对的线程数、每台或两台各 N 线程或 NG」比钩子认的宽，攻方 K3-1 打中。
- `.claude/main-agent.md`：验证段第 1、4、5 步与 `--checker-quick-memory` 正推核对上；「一轮怎么开、怎么收」第 2 条说的登记处与门禁 deferred-items-registered 认的不是同一组，攻方 K2-4 打中；第 7 行「输入一节是派发时必须给的东西」与七份定义缺「复用：」，攻方 K2-6 打中。
- `.claude/rules/three-way-inference.md`（「核查与判决由主 agent 做」第 1 步）：只倒推 `judged-sha256.txt`、`kb-sha256.txt` 不倒推也不报，攻方 K2-1 打中。
- `.claude/agents/tooling-writer.md`（第 3 点）：没人打中；定义输入一节缺「复用：」，攻方 K2-6 打中（同 clerk、implementation-writer、investigator 与三份三方定义）。
- `.claude/hooks/handback-guard.sh`：正推 ① ㉕ ㉞ 做到；⑦ 的豁免按整份提示的子串认，「不是只补落产物」会误放（正推）。
- `.claude/hooks/runner-dispatch-guard.sh`：正推 ⑤ ㉗ ㉙ 做到；⑱ HOST_QUOTA 只认数字紧挨量词的字面，攻方 K3-1 打中。
- `.claude/hooks/lib_three_way_legs.py`、`.claude/hooks/lib_prompt_scratch_line.py`、`.claude/hooks/completion-cleanup-guard.sh`、`.claude/hooks/lib_writer_lock.py`、`.claude/hooks/heavy-test-guard.sh`、`.claude/hooks/selftest-registry.tsv`、`.claude/hooks/fixtures/runner-dispatch-guard/real-prompts/`：没人打中。
- `.claude/hooks/continuation-guard.sh`、`.claude/hooks/bash-command-detector.sh`、`scripts/agents/snapshot-tree.py`、`.claude/gate.d/lib/stage-cells.sh`：「平常跑设了就拒」被关守卫那一项借道，攻方 K2-2 打中（`snapshot-tree.py` 量了，另三处推的）；`snapshot-tree.py` 照搬迁登记倒推 r4–r6 全对上，没打中。
- `.claude/gate.d/doc-experiments.sh`：preregistration-parallelism 只读「预计挂钟：」那一行自己的态数，不对第六节，正推 ② 打中一半。
- `.claude/gate.d/doc-process-records.sh`（deferred-items-registered）：攻方 K2-4 打中。
- `.claude/gate.d/code-tooling.sh`（break-switch-replay 格）、`.claude/gate.d/break-switch-replay-files.tsv`：基准趟与开关趟共用一个限时，攻方 K2-3 打中（代码量了，超时的后果推的）。
- `.claude/gate.d/doc-registries.sh`、`.claude/gate.d/deferral-registries.tsv`、`.claude/gate.d/break-switch-outside-selftest-pending.tsv`、`.claude/gate.d/stage-owners.tsv`、`.claude/gate.d/lib-lag-baseline.py`、`.claude/gate.d/checker-tier-crates-mutation-replay.sh`、`.claude/long-run-live-hosts-exempt`、`.claude/heavy-thread-exempt`：没人打中。
- `scripts/agents/agent-watch.py`：内存过线共用 `registered_light_selftest`，工作根副本里入口同 sha256 而 source 的库改过的照认成轻的自证，攻方 K2-5 打中（认法量了，漏报推的）。
- `scripts/gate/merge-and-verify.sh`：`--selftest` 不经包装退 0、经 `run-with-memory-cap.sh 4G` 退 1（快档内存上限两格红），文件头第 121 行「没有按 36G 跑过」过时，正推 ㉟ 打中。
- `scripts/agents/dispatch-prompt.py`、`scripts/agents/agent-cost.py`、`scripts/agents/watch.sh`、`scripts/kb/` 下五份、`scripts/gate/hook-swap.py`、`scripts/kb/claim-experiment.sh`、`scripts/kb/migrate-changelog-format.py`、`scripts/hosts/detached-run.sh`、`scripts/mutation/mutate.sh`、`scripts/mutation/broad-mutation.py`、`scripts/mutation/mutation-shard-run.sh`：没人打中。
- E171 编排目录 `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/`（任务 53 的 11 份）：第二段修订 11 经修订 10 的让路与第一段逐行编排各按登记值分槽，同一台合计 60 线程（登记 31），攻方 K3-2 打中。
- `records/2026-10-23-subagent拆分提案.md` 第九十节去向列、`records/2026-10-22-子agent结果落地审计.md` 节「第十二批」：正推逐行核，⑤ 不做的依据有三条站不住、调度记录第 366 行「36G 够用」说过头、㉟ 行文「任务 53 到 56」多了 53。

## 一、核查

- 开工快照：`python3 scripts/agents/snapshot-tree.py landing-audit-r7 --out <草稿目录>/r7-tree` 末行「✓ 快照 landing-audit-r7：清单点名 80 份，倒推出 80 份，sha256 对上 80 份」；`sha256sum -c research/prompts/landing-audit-r7-snapshot/judged-sha256.txt` 80 份、`kb-sha256.txt` 4 份全部 OK，腿跑着时没有人改被判文件。
- 引文：`python3 scripts/agents/cite-check.py research/prompts/landing-audit-r7-sonnet-output.md research/prompts/landing-audit-r7-opus-output.md --root <草稿目录>/r7-tree --background research/prompts/_landing-audit-r7-background.md` 末行「✓ 核了 43 处引文、92 处裸行号，对上 34 处，没判 101 处（其中裸行号没有引文可比 78 处；逐处列在上面）」，0 处对不上。没判的 101 处里，判决要用的那几处主 agent 逐处在主工作区打开核了（被判文件与快照逐份相同）：`snapshot-tree.py` 第 12、77、146 行，`cite-check.py` 第 108 行，`code-tooling.sh` 第 4631、4635 行，`doc-process-records.sh` 第 269、274 行与 `pointers` 那一段，`selftest-registry.tsv` 第 12 行，`agent-watch.py` 的 `memory_selftest_owner`，`doc-experiments.sh` 第 2543 行，`merge-and-verify.sh` 第 121 行，调度记录第 366 行，`rows_orchestrator_segment_two_revision_10.py` 第 15–18 行，都与腿引的一致。
- 复跑（主工作区，被判文件与快照相同）：
  - K2-2：`SNAPSHOT_TREE_BREAK=guard-off,skip-sha-check python3 scripts/agents/snapshot-tree.py landing-audit-r6 --out <草稿> --list judged-sha256.txt --list kb-sha256.txt` 平常跑照样执行，原样「✓ .claude/kb/checks-owed.md  来源：HEAD（diff 里没有它）  sha256 a109e9f0ea79」，退 1；单写 `skip-sha-check` 照拒、退 2。
  - K2-6：十份带预计挂钟的定义里 `grep -c '复用：'` 为 0 的七份：clerk、implementation-writer、investigator、three-way-attack、three-way-defense、three-way-forward、tooling-writer。
  - K3-1：攻方造的句子喂主工作区派发闸 `--check-prompt … --type tooling-writer --root .`：「两台各 31 线程跑。」退 2；「两台各跑 31 线程。」「本机与第二台各 31 线程跑。」「本机 31、第二台 31 线程跑。」「本机与第二台各给 36G。」「本机跑 31 路并行，第二台跑 31 路并行。」都退 0；观测句「上一趟主 agent 复跑两趟（本机并行 31，第二台…）」退 0、「本机 31 线程、第二台 31 线程的那一趟里 3 条…」退 2。
  - K3-2：`python3 -I <攻方草稿>/probe_e171.py` 原样「台 1：两段各按登记值分槽，线程合计 60，登记值 31」。
  - ㉟（正推）：`bash scripts/hosts/run-with-memory-cap.sh 4G bash scripts/gate/merge-and-verify.sh --selftest` 退 1，红「✗ 自检：checker 档快档那一步：内存上限 峰值表里这一步近几次最大 10 GiB（其中一次撞了顶），没给上限时应当取 15G（乘 1.5）…实际退出码 0，快档那一趟收到「36G …」」与「…峰值表算出来的比 --memory 小时应当取 --memory 8G…」；不经包装退 0。
- 没复跑的：K2-3 的超时后果（要起门禁 break-switch-replay 整格，提交时才跑的检查；用户定这一批不提交）；K2-4 门禁真跑（同上，主 agent 对着 `pointers` 与 ①③ 的读法核了代码）；K2-5 真有过线进程被漏报（攻方只量了认法）。

## 二、逐格判

### K1（正推：每处改法做没做到它那一行）

| 行 | 腿报 | 主 agent 判 |
|---|---|---|
| ① | 做到；⑦ 的豁免按子串认会误放；`b12-handback.md` 第 64 行把 a9dbfee4c29c6da98 说成只写实验页 | 采：做到；豁免子串那一处进下一批（零轮）；交回报告那一句是交回报告的说法，不改冻结证据 |
| ② | 一半：挂钟算式做到，第六节的态数不对 | 采：现查 `doc-experiments.sh` 第 2543 行「态数与单态耗时只读「预计挂钟：」那一行，写在第六节或别的行、这一行里没重写的不读」，第九十节 ② 要对的是第六节 |
| ③ | 做到；登记处表是定义第 2 条没列的第四类 | 采；与攻方 K2-4 同一处，并进 K2-4 |
| ④ | 做到 | 采 |
| ⑤ | 一半：六条骨架加闸做到；不做的六条 B9、E2 成立，B8、A5 一半，D3 依据不成立，C4 弱 | 采：D3 现查 `scripts/gate/apply-writer-patch.py` 里没有撞文件的核对（`grep -n 'numstat\|撞文件\|互不撞'` 零命中），常设要求清单 `b12-standing-requirements.md` 第 11 行本把 D3 列为「骨架按类型打那一行、派发闸核那一行在」做得了；B8 的依据核的是「岔路表还差」不是指到前一份报告；A5「节在不在」判得了。第九十节 ⑤ 的去向列里这三条的依据改写，三条进下一批 |
| ⑩ | 判不做的依据成立 | 采 |
| ㉕ ㉖ ㉗ ㉙ ㉚ ㉝ ㉞ | 做到 | 采；㉕ 行外「目标本身写成 `$(…)/crates/…rs` 的不计」记进下一批（零轮）；㉚ ⑯ 收严影响 63 与交回报告 84 是语料范围不同，不改 |
| ㉟ | 做到，三处偏差 | 采：调度记录第 366 行「36G 够用」改成「那一趟没被杀（峰值正好等于上限，含页缓存）」；`merge-and-verify.sh` 第 121 行过时、自证经内存包装跑就红，进下一批；行文「任务 53 到 56」改成「任务 54 到 56」 |

### K2（攻方：内部矛盾与越位）

| 条 | 主 agent 判 |
|---|---|
| K2-1 kb 清单不倒推、非 git 根上按文件名的引文全认不出 | 打中，采（量了）。这一轮判决因此只核了被判文件那 80 份的引文；kb 引文主 agent 按 `kb-sha256.txt` 4 份 OK 在主工作区核 |
| K2-2 关守卫项借道 | 打中，采（`snapshot-tree.py` 量了，续做闸、检出钩子、`stage-cells.sh` 推的） |
| K2-3 基准趟与开关趟共用限时、两套读表 | 代码那一半打中，采；超时的后果推的，没核 |
| K2-4 deferred-items-registered 与定义第 2 条认的登记处不同 | 打中，采（代码读的）；与正推 ③ 并成一件 |
| K2-5 副本里库改过照认成轻的自证 | 认法那一半打中，采；与拆分提案一百一十八节（重型闸同一处）同根，并进那一节 |
| K2-6 七份定义缺「复用：」 | 打中，采（量了） |

### K3（攻方：哪一条最脆）

| 条 | 主 agent 判 |
|---|---|
| K3-1 HOST_QUOTA 只认紧挨的字面 | 打中，采（复跑逐条一致）；真提示 18 份 0 份撞上，误拦那一半钩子注释已承认分不出派法与观测，改宽要回到真语料重量 |
| K3-2 E171 第二段与第一段各按登记值分槽 | 算术打中，采；同时跑的时间段推的。并进拆分提案一百一十九节（同一个改法：按「这一台已分出去的线程」扣） |

## 三、判决

1. 第一轮。两条腿打中的 13 处（正推 ② ⑤ ㉟ 与 ① ㉕ 两处小毛病，攻方 K2 六条、K3 两条）主 agent 逐条现查都坐实，没有一条被推翻；照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」，打中的先挂起，改法并进下一批，改完开 landing-audit-r8 攻那一批的修补。
2. 腿提的改法全部被攻过零轮。
3. 这一批的出口（第九十节剩下开着的行全关）除 ② ⑤ 两行只做了一半、㉟ 里 `merge-and-verify.sh` 那一件留了自证缺陷之外都做到；那三处与这一轮打中的照 `.claude/main-agent.md`「一轮怎么开、怎么收」第 1 条「批内冒出的发现只进下一轮的清单」登记在 `records/2026-10-23-subagent拆分提案.md` 一百二十二节，记录里的两处说过头的话（调度记录第 366 行、第九十节 ㉟ 行文）判决后当场改。

## 四、交用户

| 件 | 内容 | 被攻过 |
|---|---|---|
| 下一批要不要现在开 | 一百二十二节十一件（攻方 K2-1、K2-2、K2-3、K2-4、K2-6、K3-1，正推 ② ⑤ ㉟ 与 ① ㉕ 两处）连同并进一百一十八节的 K2-5、并进一百一十九节的 K3-2，派第十三批，改完开 landing-audit-r8；不开就留在拆分提案 | — |
| 改法 | 各件改法都是腿提的或主 agent 推的 | 零轮 |

## 五、没核动的

- K2-3 超时的后果、K2-4 门禁真跑：门禁阶段是提交时才跑的检查，用户定这一批不提交。
- K2-5 漏报一个真过线进程：要在副本里改库起一个吃内存的进程，没造。
- K3-2 两段真在同一台同时跑多久：要起 E171 装置，不起。

## 回看决策

不涉及决策：这一轮判的是子 agent 定义、钩子、门禁与研究脚本的协作形态，`.claude/kb/decisions/` 下没有一条分项的依据引这些文件或被这一轮的结论改动。
