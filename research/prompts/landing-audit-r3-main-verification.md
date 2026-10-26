# landing-audit-r3 判决：子 agent 落地审计这一批改动第三轮、最后一轮（主 agent，2026-10-23）

<!-- doc-lint:not-numbers K1 K2 K3 K4 -->

正文 `research/prompts/_landing-audit-r3-body.md`；辩方腿 `research/prompts/landing-audit-r3-sonnet-output.md`（sha256 a7dbf524…），模型 `research/prompts/landing-audit-r3-sonnet-model/`；攻方腿 `research/prompts/landing-audit-r3-opus-output.md`（sha256 9605a92c…），模型 `research/prompts/landing-audit-r3-opus-model/`。前两轮判决 `research/prompts/landing-audit-r1-main-verification.md`、`research/prompts/landing-audit-r2-main-verification.md`。第三轮之后不再开轮（`.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」），这一轮新冒出来的改法都写进节「四、交用户」、标零轮。

## 这一轮被判过的文件

- `.claude/agent-common.md`：第五批 K3-2 到 K3-5 的几句、第七批第 108 行删「名字里带 report 的文件」半句；辩方点出同一句里 SUMMARY、FINDINGS、ANALYSIS 三个前缀没人实测过。
- `.claude/main-agent.md`：第五批 K3-2、K3-5、K3-9 的几句（前两轮站住，这一轮没人打中）。
- `.claude/agents/experiment-designer.md`：第七批第 3a 步 ②（预期差连同基线列进「跑之前已经存在的数」、分不出的交回）；没人打中。
- `.claude/agents/implementation-writer.md`：第五批第 4 步「命名检查除外」；攻方 K2-13 只打中措辞（列举里没写命名检查），与共用约束、钩子不矛盾。
- `.claude/agents/experiment-runner.md`、`.claude/agents/sweep.md`、`.claude/agents/tooling-writer.md`、`.claude/agents/three-way-attack.md`、`.claude/agents/three-way-defense.md`：第五批改的几处，前两轮站住，这一轮没人打中。
- `.claude/hooks/runner-dispatch-guard.sh`：第七批报告行锚法、否定词；攻方 K2-1、K2-2、K2-3 打中锚法本身分不出自己的报告行与转述。
- `.claude/hooks/handback-guard.sh`：第七批超长拒绝说明；没人打中。
- `.claude/hooks/continuation-guard.sh`、`.claude/hooks/lib_long_run.py`、`.claude/hooks/agent-write-scope.tsv`、`.claude/hooks/selftest-registry.tsv`、`scripts/agents/subagent_transcript.py`、`scripts/agents/agent-watch.py`、`scripts/agents/dispatch-prompt.py`：第五批站住的形态，攻方核过续做闸与交回定义没被第七批改坏。
- `scripts/agents/agent-cost.py`：第七批东京日期切天、三方腿两桶、词表尾词界；攻方核出两版结局账的差全由切天解释。
- `.claude/gate.d/lib-lag-baseline.py`、`.claude/gate.d/doc-experiments.sh`、`.claude/gate.d/doc-decisions.sh`、`.claude/gate.d/doc-registries.sh`、`scripts/kb/quote-kb.py`、`scripts/kb/stale-candidates.py`、`.claude/gate.d/doc-process-records.sh`、`.claude/gate.d/code-source-discipline.sh`、`.claude/gate.d/code-tooling.sh`、`.claude/gate.d/replay-comparison-units.tsv`、`.claude/gate.d/stage-owners.tsv` 与新样本 `.claude/gate.d/fixtures/doc-experiments.sh/replay-comparison-units-rekey-red/`、`.claude/gate.d/fixtures/doc-experiments.sh/decision-links-pending-base-unreadable-red/`、`.claude/gate.d/fixtures/doc-decisions.sh/evidence-ledger-base-unreadable-red/`：攻方 K2-8（登记表文件头还写旧键）、K2-9（doc-registries 起不来退 77）打中，都轻。
- `.claude/kb/checks-owed.md` C879、C880；`records/2026-10-22-子agent结果落地审计.md`；`records/2026-09-16-subagent拆分提案.md` 第九十节；归档目录 `research/prompts/subagent-landing-audit-2026-10-22/`。

## 一、核查

- 快照：`sha256sum -c research/prompts/landing-audit-r3-snapshot/judged-sha256.txt` 74 份全对上，腿跑着的时候被判文件没动。
- `python3 scripts/agents/cite-check.py research/prompts/landing-audit-r3-sonnet-output.md research/prompts/landing-audit-r3-opus-output.md --root . --background research/prompts/_landing-audit-r3-background.md` 末行：「✓ 核了 16 处引文、164 处裸行号，对上 16 处，没判 164 处（其中裸行号没有引文可比 164 处；逐处列在上面）」。下面采的每一条，主 agent 打开那一行或跑那条命令现看过（列在各条）。
- 攻方模型拷进草稿、经 `scripts/hosts/run-with-memory-cap.sh 4G` 跑 `tools/rerun.sh --repo <仓根> --frozen <仓根> --work <新目录>`，退 0：18 份输出与腿的 `out/` 逐字节相同（含 `dispatch-check.txt`、`name-verdict-variants.txt`、`sentence-start-extra.txt`、`decompose.txt`、`threshold-sweep-today.txt`、`r2-rebuild.txt`、`r2-cite-check-rebuilt.txt`、`archive-sha256.txt`）；3 份不同的是逐个子 agent 的明细与交回形状清单（`today-tokyo-agents.jsonl`、`before-r2fix-utc-agents.jsonl`、`handback-shapes-today.txt`），腿交回之后又有子 agent 开工与交回，汇总那几份相同。
- 辩方的几条主 agent 现查：N01，`grep -c subagent_transcript .claude/gate.d/break-switch-replay-files.tsv` 输出 0；N10，`agent-cost.py outcome` 在 `.claude/main-agent.md`、`.claude/agent-common.md` 里 0 处；N15，调度记录第 56 行「修红」那一条在九十节里 0 处；N09，第二轮判决 K3-4 那行写的「定义写的就是主 agent 建的工作树」，在 `.claude/agent-common.md` 与 `.claude/agents/*.md` 里 0 处（第 97 行写的是 git worktree 建的副本）；⑯ 的判据，`.claude/singlefs-ai-sop/scripts/gate-overlap.py:75`「CLONE_MINIMUM_LINES = 8」。

## 二、逐格判

### K1（辩方：未定项清单的完整性）

辩方把六份工具员交回、两份判决、四份腿报告里的「没做」「没查」「留给下一批」「交主 agent」归成 66 件：有去向 40、一半 5、没有 21（重 6、轻 15）。

| 条 | 辩方 | 判 |
|---|---|---|
| N01 | `subagent_transcript.py` 的弄坏开关不在复跑表 | 采：并进九十节 ⑲ |
| N06 | ⑧ 没定「子目录算不算覆盖」 | 采：⑧ 补这一句 |
| N08 | 共用约束写 SUMMARY、FINDINGS、ANALYSIS 开头也被拒，没人实测 | 采：九十节 ㉑，零轮 |
| N09 | 第二轮判决 K3-4「不改」引的前提在定义里没有 | 采：第二轮判决那一条的理由不成立；结论（不改）照旧，因为 `.claude/agent-common.md` 节「不做」禁子 agent 建 worktree，第 97 行「git worktree 建的副本」只可能是主 agent 建的，两处合读不矛盾；第二轮判决是冻结证据，不回改，这里记更正 |
| N10 | 结局账没有常设的调用点 | 采：九十节 ㉒，零轮 |
| N15、N16 | 「修红」与「中途停下」两条缺口没去向 | 采：九十节 ㉓，交用户 |
| 九十节形态 | ⑥ 缺 rsync 一支的适用条件；⑪ 第七批已做没标；⑯ 判据靠查重永远是绿、漏第四处；⑰ 出处在调度记录里找不到 | 采：四行都改了；⑰ 的出处补进调度记录节「第二轮判决与第七批」末尾 |
| 交主 agent 的 10 处 | 都改了；K3-1 的限定没落到调度记录第 40 行的 1.1% | 采：第 40 行补限定 |
| 其余 | 有去向或站得住 | — |

### K2（攻方：已定项撑不撑得住）

| 条 | 攻方 | 判 |
|---|---|---|
| K2-1 | 派发外加的「句首也认」把转述别人报告的句子当报告行，2026-10-14 以后 3 份真提示今天照拒 | 采：九十节 ⑳，零轮 |
| K2-2 | 判决定的「逗号之后不认」放过 4 份写在逗号后的真报告行（真数据里没出事） | 采：并进 ⑳。第二轮判决的规格自己带出来的 |
| K2-3（轻） | 「交付：报告 …-output.md」写法 27 份被判没有报告行，全在 2026-10-11 及以前 | 采：并进 ⑳ |
| K2-8、K2-9、K2-13（轻） | 登记表文件头旧键；doc-registries 起不来退 77；实现员第 4 步列举措辞 | 采：九十节 ㉔ |
| 没打中 | 否定词判法（3292 份真提示 0 份判得不同）、`--since` 调用、库的 `./`、`.` 拒、三方两桶（真数据 0 误认）、`\bfix` 尾词界、续做闸与交回定义 | — |

⑳ 的结论：行首与句首两种锚法在真提示上各有误拦，按位置分不出「自己的报告行」与「引别人的报告」；能分辨的观测是路径落不落在本提示「草稿目录：」下（攻方模型上量过，被攻过零轮）。

### K3（攻方：报出的数能不能核）

| 条 | 攻方 | 判 |
|---|---|---|
| K3-1 | 结局账 2144 与 2125 的差全由切天解释：只在旧窗口 45 个、只在新窗口 26 个都在切天差出的那九个小时里，两版都有的 2099 个桶一个没变；返工字样 327→313 | 站得住（攻方复出、主 agent 复跑模型逐字节相同） |
| K3-2 | 门槛换值表在新版上复得出，(0.9, 0.7) 那一格 6.2% 变 6.1%；调度记录第 138 行把闭窗口量的换值读数写进了 2268 那一次的句子 | 采：第 138 行补出处 |
| K3-3 | 自证种数（460、23、18、90、20、12、9）与两份真提示的 `--check` 读数复得出；第二轮没复算的 45 个、1.66% 复得出 | 站得住 |
| K3-4（轻） | 第二轮判决「`--root .` 与快照根同义」不成立：只对快照里的那 63 份成立，腿引到而不在快照里的文件判的是活仓；攻方用快照 diff 头里的改前 blob 重建第二轮快照，63 份全对上，以它为根 85 处引文判得了 39 处，全对上 | 采：更正第二轮判决那一句的射程（冻结证据不回改，这里记）；这一轮 `--root .` 同样只对快照里的 74 份成立 |

## 三、判决

一、主 agent 已改（这一轮内）：调度记录第 40 行 1.1% 补出处、第 138 行换值读数补出处、两处「用量撞顶」改成「中途停下」、节「第二轮判决与第七批」末尾补第七批看门狗检出那一条；九十节 ⑥ 补 rsync 一支、⑧ 补子目录、⑪ 标第七批已做、⑯ 改判据并补第四处、⑰ 改出处、⑲ 并进 N01；加 ⑳ 到 ㉔。

二、不再开轮。前两轮采的改法（第五批、第七批）这一轮没被打穿，照三轮多数判站住；这一轮打中的都登进九十节，改不改、什么时候改交用户（节「四、交用户」）。

三、提交前要跑、这一批还没跑过的：五道门禁的样本档 `stage-selftest.sh`（doc-experiments、doc-decisions、doc-process-records、code-source-discipline、code-tooling）、库与派发闸新开关的复跑格、真仓单跑第七批改过的格；`.claude/batch-scope` 登记这一批的文件（辩方第二轮已点）。这几样要样本与暂存区相同，归提交前整轮。

## 四、交用户

都标零轮（腿或主 agent 推的改法，没被攻过）：

| 九十节 | 是什么 | 选项 |
|---|---|---|
| ⑳ | 派发闸报告行的认法：按位置锚分不出自己的报告行与转述 | 照攻方改成按路径落不落在本提示草稿目录下判；或维持今天的锚法、接受 3 份转述误拦与 4 份逗号后放过 |
| ㉑ | 共用约束里 SUMMARY、FINDINGS、ANALYSIS 三个前缀没人实测 | 先实测再定那一句；或先删掉没测过的三个前缀 |
| ㉒ | 结局账没有常设的时机 | 每批收尾的收拢表那一步跑闭窗口 `outcome --summary` 并落盘；或维持按需 |
| ㉓ | 「修红」与「中途停下没交回」两条缺口没去向 | 逐条立拦它的形态；或在调度记录写不改的依据 |
| ⑥ 到 ⑲、㉔ | 前两轮与这一轮登的工具欠账 | 下一批工具改动一起派；或挑几条先做 |

## 五、没核动的

- 攻方 K2-1 的 3 份转述、K2-2 的 4 份逗号后报告行：主 agent 只复跑了模型、比了输出，没逐份回原会话记录读上下文核「是转述」；采的是「按位置分不出」这一机制，攻方给的推翻条件（那几句其实是派它写进那个路径）没核。
- 辩方 66 件的归并与 21 件没去向的逐条判定，主 agent 只现查了 N01、N06（九十节「子目录」0 处）、N09、N10、N15 与 ⑯、⑰ 的出处，其余照辩方报告的出处列采。
- 攻方 K2-9 是读代码推的，门禁阶段实跑被重型测试闸拒，主 agent 也没跑（归提交前）。

## 回看决策

不涉及决策：被判的都是 agent 定义、共用约束、钩子、门禁与协作脚本的工作流写法，加上欠账表两行、里程碑三「待优化」表一句现状与两份调度记录；不碰 `.claude/kb/decisions/` 的任何分项。

## 用户定案（2026-10-23，弹窗）

- ⑳：「按草稿目录判（推荐）」。
- ㉑：「先实测再定（推荐）」。
- ㉒：「每批收尾跑一次（推荐）」。
- ⑥ 到 ㉔：「下一批一起派（推荐）」。
- ㉓ 修红里的 checker 档红（`research/prompts/subagent-landing-audit-2026-10-22/rework.md` 第 178 行 K11，7/15、折算 26.49 百万）：「合入后验证跑受影响的快档（推荐）」——主 agent 合入后验证时带 `SINGLEFS_HEAVY_TESTS=user-request` 只跑单元键对不上的 checker 档快档测试，红了进写段下一批。㉓ 中途停下没交回那一条：主 agent 判不改，依据是交接摘要（`scripts/agents/agent-handover.py`）接手后大半半成品被用上，白做约 2 到 4 百万，占闭窗口结局账 7004.91 百万约 0.05%（调度记录节「流程缺口」那一行）。
