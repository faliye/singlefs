# landing-audit-r4：子 agent 落地审计第九批改动第一轮正文（2026-10-23）

<!-- doc-lint:not-numbers K1 K2 K3 -->

## 一、题面

前三轮判决（`research/prompts/landing-audit-r1-main-verification.md`、`research/prompts/landing-audit-r2-main-verification.md`、`research/prompts/landing-audit-r3-main-verification.md`）连同用户定案，把落地审计里还拦不住的事记成拆分提案 `records/2026-09-16-subagent拆分提案.md` 第九十节 ⑥ 到 ㉔ 这几行。第九批八件（任务 24 到 31，依赖表与逐件结果在调度记录 `records/2026-10-22-子agent结果落地审计.md` 节「第九批」）照这几行改了定义、共用约束、钩子、门禁与脚本，交回报告归档在 `research/prompts/subagent-landing-audit-2026-10-22/b9-*.md`。这一轮是这一批改动的第一轮（`.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」第一轮分工：过度外推与越位、内部矛盾与循环依赖、哪一条最脆弱）：

1. 每一处改法做没做到它那一行的验收标准（九十节「要拦什么」列），有没有越出那一行、漏改、把推的写成量的。
2. 新句子、新判法与已有规则、定义、钩子、门禁之间有没有矛盾，有没有两处判同一件事而判法不同。
3. 这一批新判法里哪一处最脆：哪一格换一个输入就误拦或误放。

被判的文件：

| 类 | 文件 |
|---|---|
| 定义与共用约束 | `.claude/agent-common.md`、`.claude/main-agent.md`、`.claude/agents/implementation-writer.md`、`.claude/agents/kb-scribe.md`、`.claude/agents/experiment-designer.md`、`.claude/agents/three-way-forward.md`、`.claude/agents/three-way-attack.md`、`.claude/agents/three-way-defense.md`（三份只动 frontmatter） |
| 钩子与登记表 | `.claude/hooks/runner-dispatch-guard.sh`、`.claude/hooks/handback-guard.sh`、`.claude/hooks/bash-command-detector.sh`（只判 ⑰ 那一段：mktemp 模板落在自己认领的草稿目录里）、`.claude/hooks/lib_three_way_legs.py`、`.claude/hooks/write-guard.sh`（只判自证里 kb-scribe 不变量清单那几格）、`.claude/hooks/agent-write-scope.tsv`、`.claude/hooks/lib_heavy_tests.py`（只判顶层 `lib-` 库那一支）、`.claude/hooks/selftest-registry.tsv` |
| 门禁与登记 | `.claude/gate.d/code-tooling.sh`（新格 kb-directories-write-scope）、`.claude/gate.d/doc-experiments.sh`、`.claude/gate.d/doc-decisions.sh`、`.claude/gate.d/doc-registries.sh`、`.claude/gate.d/checker-independence-and-sync.sh`、`.claude/gate.d/experiment-number-directories.tsv`、`.claude/gate.d/replay-comparison-units.tsv`、`.claude/gate.d/break-switch-replay-files.tsv`、`.claude/gate.d/stage-owners.tsv`；新样本 `.claude/gate.d/fixtures/` 下 `checker-independence-and-sync.sh/layout-checker-sync-base-unreadable-red`、`code-tooling.sh/kb-directories-write-scope-green`、`code-tooling.sh/kb-directories-write-scope-red`、`doc-decisions.sh/history-loss-base-unreadable-red`、`doc-experiments.sh/experiment-orphans-registry-red`、`doc-experiments.sh/replay-comparison-units-table-green`、`doc-experiments.sh/replay-comparison-units-table-red`、`doc-registries.sh/kb-directories-unreadable-red` |
| 脚本 | `scripts/agents/agent-watch.py`、`scripts/agents/agent-cost.py`、`scripts/agents/watch.conf`、`scripts/agents/watch.sh`（只判键表一处）、`scripts/agents/dispatch-prompt.py`、`scripts/gate/merge-and-verify.sh`（只判新加的 checker 档快档那一步）、`scripts/kb/claim-experiment.sh`、`scripts/kb/lib_experiment_number_directories.py` |
| 记录 | `records/2026-09-16-subagent拆分提案.md` 第九十节 ⑥ 到 ㉗ 的去向列；调度记录节「第九批」 |

## 二、实现今天的样子（主 agent 的观测，2026-10-23）

方案按 `crates/` 与 `scripts/` 今天的实现来谈（主工作区现状）。这一批没有改 `crates/` 下的任何文件；`scripts/gate/merge-and-verify.sh` 新加的那一步起门禁 `checker-tier-crash-point-replay.sh --check crash-case-replay,feature-gated-tests`，判的是 `crates/singlefs-checker-tier` 的快档测试，键与复用由那一道算。

- 主 agent 在主工作区重跑过的自证读数（调度记录节「第九批」逐件写了）：派发闸 483 种、交回闸 49 种、检出钩子 764 种、写闸 229 种、`lib_heavy_tests.py` 783 种、`heavy-test-guard.sh` 1395 种、`scripts/gate/light-selftest-guard.py` 14 格、`lib-lag-baseline.py` 9 项、`dispatch-prompt.py` 92 项、`agent-cost.py` 23 项、`merge-and-verify.sh` 45 种、`claim-experiment.sh`、`watch.sh`、`agent-watch.py`，都退 0；几条新开关逐个打开红在点名的格。
- 没跑过的：门禁阶段本身（样本档 `stage-selftest.sh` 要样本与暂存区相同，这一批没暂存）、code-tooling 的 break-switch-replay、break-switch-headers、kb-directories-write-scope 几格；样本只在工具员草稿里逐份核过。
- 主 agent 自己改的、没被工具员做过的：`write-guard.sh` 自证里「kb-scribe 不变量清单的子目录越界」一格换成「子目录里的 .md 放行、.rs 越界」两格（规格由任务 31 的工具员起草，主 agent 经 `hook-swap.py` 换上）。
- 别的会话这几天在同几份文件上的改动，不在被判之列：`bash-command-detector.sh` 的 ⑫ 收严（算不出落点的一律执行前拒）、`agent-watch.py` 的根分区余量告警、`merge-and-verify.sh` 的每趟自己的编译目录、`heavy-test-guard.sh` 拒在主工作区直接起会编的 cargo、`.claude/main-agent.md`「一轮怎么开、怎么收」第 11 条。腿判到它们时写明是别人的改动，不算这一批的。
- 暂存区里已有这一批多份文件的一截改动（不是本会话暂存的），快照因此对 HEAD 取 diff，不对暂存区取。

开工快照：`research/prompts/landing-audit-r4-snapshot/judged-sha256.txt`（被判文件）、`research/prompts/landing-audit-r4-snapshot/judged-vs-head.diff`（被判文件对 HEAD 的 diff，加新建文件全文）。

## 三、格

### K1　每处改法做没做到它那一行（正推）

- 对着九十节 ⑥ 到 ㉔ 每一行的「要拦什么」与去向列：改法落在哪一份文件哪一句、哪一个判法，做到的写依据（文件:行），没做到、只做了一半、越出那一行的写哪一格。
- 每条改法照 `.claude/rules/implementation-workflow.md`「改 agent 定义与共用约束，走同一条三步」第 3 步：该红的输入现在会不会红；交回报告里写「推的、没量过」的（⑬ 的 512 MiB 下限、⑯ 加 frontmatter 键之后定义照常派出、任务 30 的桶变化），这一批有没有当成量的写进别处。

### K2　新句子与已有的有没有矛盾（攻方）

- 定义之间：`.claude/main-agent.md` 第 3 条「重型测试提交之外确实要跑，先弹窗问用户」与第 10 条、「写与验分两段」验证段第 1 步新加的 checker 档快档那一步（带 `SINGLEFS_HEAVY_TESTS=user-request`）；第 8 条新加的结局账 `--out records/<轮>-outcome-<这一天>.txt` 与 `.claude/rules/directory-layout.md`、记录类门禁对 `records/` 的要求；`.claude/agent-common.md` 交回正文闸那一条补的两样与 `.claude/hooks/handback-guard.sh` 文件头；`.claude/agents/kb-scribe.md` 写范围与 `CLAUDE.md`「什么时候派哪个 agent」那张表里「只限这些 `.md`」。
- 钩子之间：派发闸 ⑰ 新认法（只判本提示草稿目录下的路径与裸文件名、三方腿认整份提示里的 `research/prompts/<名>-output.md`、三个前缀）与写闸、交回闸里管报告文件的判法；交回闸新 ⑤（命名检查）与 `.claude/agent-common.md` 里「命名检查交回前跑」那一条、与 rsync 副本交主 agent 那一支。
- 门禁之间：`lib_heavy_tests.py` 认顶层 `lib-` 库之后，带 `--selftest` 而没登记的 `lib-forbidden-notations.py` 从放行变成被拒；`experiment-number-directories.tsv` 放在 `.claude/gate.d/` 而 `.claude/rules/directory-layout.md`「只被门禁自己读的配置留在门禁旁边」，`claim-experiment.sh` 也读它；`.claude/gate.d/replay-comparison-units.tsv` 文件头新改的五行机读「# 不登记的比对对象」与 `.claude/kb/checks-owed.md` 的 C879（E165 根选择与杀伤界正式跑没有单元）那一行说的是不是同一件事，C879 的现状句这一批之后还对不对。

### K3　哪一条最脆（攻方）

- 第九十节 ㉕、㉖、㉗ 三行已记的口子之外：这一批新判法里，换一个输入就误拦或误放的地方。重点看：⑭ 看门狗怎么认自证临时目录（照登记表认 light 自证、按 TMPDIR 与名字形态认）；⑬ 阈值取下限之后，峰值表里真大的命令还报不报；⑳ 草稿目录一行的认法（冒号可有可无、可带反引号、不限行首）会不会认到别处引的草稿目录；⑥ 交回闸按会话记录认「最后一次改 `.rs` 之后跑过两条命名检查」，被钩子拒的、跑在别的根上的算不算；`merge-and-verify.sh` 判「补丁没碰 checker 档闭包」靠什么，闭包外被测试读到的文件会不会让这一步误报本次未跑。
- 给出能复跑的输入：在自己目录里拷那份钩子或脚本，喂一条造出来的输入，引输出整行。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1 | 从九十节每一行推到改法：做到没有、依据在哪 |
| 云端攻方（Opus） | K2、K3 | 找互相矛盾的、两处判同一件事而判法不同的，和换个输入就误判的 |
