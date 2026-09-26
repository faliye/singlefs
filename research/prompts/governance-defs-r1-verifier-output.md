# governance-defs-r1 核查员报告

这是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

## 判别力自证

选定引用：opus 腿 A10 引 `.claude/agent-common.md:55`（原句「长活可以等，不给它设超时…」）。在草稿目录副本 `agent-common.md.selftest` 里把行号从 55 改成 56 再核：56 行实际内容是「等长活时不起缓存计时器，结束本轮直接等完成通知。」，与报告引文不同字。
判定：**✗（行号与内容对不上，方法能分辨）**。方法有效，往下按它做。

## 前置核实

- `sha256sum -c research/prompts/governance-defs-r1-snapshot/sha256sums.txt`：31 行全部 `OK`——被判的 15 份定义、材料点名的规则/钩子/脚本，腿跑着的时候一个字节都没被别的会话改过。因此下文「文件:行号」一律直接对**当前主树**核，不需要倒推副本。
- 报告文件当前 `sha256sum` 与主 agent 给的交回值逐字相同（sonnet：`df556f7e…26fb`；opus：`81bf7dde…3302`），核的是这两份文件现在的内容。
- opus 腿模型目录 `research/prompts/governance-defs-r1-opus-model/` 下 10 个文件的 `sha256sum` 与报告正文表格里给的逐字相同（`probe.sh`、`probes.sh`、`run-all.sh`、`g1-hash-divergence.sh`、`g3-sample-numbering.sh`、`g5-mutate-markers.sh`、`g6-relabel-crates-anchor.sh`、`cite.sh`、`citations.tsv`、`run-all.out`）。

## 复跑（opus 腿模型，草稿目录副本）

命令：
```
cp -r research/prompts/governance-defs-r1-opus-model <草稿目录>/rerun/model
nice -n 19 bash <草稿目录>/rerun/model/run-all.sh <草稿目录>/rerun/scratch
```
容器前提与报告一致：无 `rsync`（`run-all.sh` 用 tar 拷副本）、`run-with-memory-cap.sh --check` 无用户级 systemd scope 退 251、以 root 跑。

复跑输出与 `research/prompts/governance-defs-r1-opus-model/run-all.out` 做逐字比对（先把两份输出里的 `/tmp/…` 路径统一替换成 `<TMP>` 再 diff，因为路径本身是每次新建的临时目录、不是承重内容）：

```
diff <(sed 's#/tmp/[^ ]*#<TMP>#g' <草稿目录>/rerun/run-all.out.rerun) \
     <(sed 's#/tmp/[^ ]*#<TMP>#g' research/prompts/governance-defs-r1-opus-model/run-all.out)
```
输出为空——**逐字一致**，包括 A1（G1 的 C0–C7 八行键与红绿判定）、A4（`--check 4G` 退 251 与错误文案）、A6（noclobber 默认关闭、H1–H4 四行退出码与 stderr）、A7（G5 的五条标记行、字面数 0/0/0、行首符号数 2/1/1/1）、A9（G6 的锚点命中从 1 变 0、33 号退出码 1、逐文件改写行数）、A10（`&` 三种写法的退出码与拒绝文案）。

## 攻方腿（Opus）核对表

重点核 A1、A4、A6、A9、A10、A7；其余（A2、A3、A5、A8，与「构造不出」「没打中」两节）按引文逐条抽查。

| 引用 | 核的结果 | 命令 |
|---|---|---|
| A1：`.claude/agents/crash-verifier.md:25` 全文 | ✓ 逐字相符 | `sed -n '25p' .claude/agents/crash-verifier.md` |
| A1：`.claude/main-agent.md:59` 全文 | ✓ 逐字相符（与 sonnet 腿引的同一行一致） | `sed -n '59p' .claude/main-agent.md` |
| A1：`.claude/rules/implementation-workflow.md:55` 全文 | ✓ 逐字相符 | `sed -n '55p' .claude/rules/implementation-workflow.md` |
| A1：`.claude/gate.d/stage-owners.tsv:37` | ✓ 逐字相符 | `sed -n '37p' .claude/gate.d/stage-owners.tsv` |
| A1：`.claude/gate.d/54-layer0-replay.sh:61,182,190,191,279,284,29` | ✓ 全部逐字相符 | `sed -n '61p;182p;190p;191p;279p;284p;29p' .claude/gate.d/54-layer0-replay.sh` |
| A1：`.claude/gate.d/stage-inputs.tsv:11` | ✓ 逐字相符 | `sed -n '11p' .claude/gate.d/stage-inputs.tsv` |
| A1：模型复跑（`g1-hash-divergence.sh`） | ✓ 复跑输出与报告贴的 8 行（C0–C7）逐字相同 | 见上「复跑」一节 |
| A2：`.claude/gate.d/stage-inputs.tsv:11`（同上）、54 号 190/191 行（同上） | ✓ | 同上 |
| A2：`git rev-parse --git-common-dir` 下无层 0 标记 | ✓ 本容器现查同样为空 | `ls -la "$(git rev-parse --git-common-dir)" \| grep -i layer0`，0 行 |
| A3：`.claude/agents/experiment-runner.md:32` | ✓ 逐字相符 | `sed -n '32p' .claude/agents/experiment-runner.md` |
| A3：`.claude/hooks/heavy-test-guard.sh:74` | ✓ 逐字相符 | `sed -n '74p' .claude/hooks/heavy-test-guard.sh` |
| A3：`research/scripts/replay.sh:383,414` | ✓ 逐字相符 | `sed -n '383p;414p' research/scripts/replay.sh` |
| A3：钩子真跑（replay.sh 被拒 4 处、经包装后放行） | ✓ 复跑（见上）exit=2/exit=0 与贴出的行号、文案逐字相同 | 见上「复跑」一节 |
| A4：`.claude/agent-common.md:46` 全文 | ✓ 逐字相符（含「15、74 号…按轻阶段对待」一句） | `sed -n '46p' .claude/agent-common.md` |
| A4：`research/scripts/run-with-memory-cap.sh:21,25` | ✓ 逐字相符 | `sed -n '21p;25p' research/scripts/run-with-memory-cap.sh` |
| A4：`--check 4G` 退 251 与错误文案 | ✓ 复跑退出码与文案逐字相同 | 见上「复跑」一节 |
| A4：`.claude/agents/three-way-attack.md:37` | ✓ 逐字相符 | `sed -n '37p' .claude/agents/three-way-attack.md` |
| A5/A6：`.claude/agents/three-way-local-attack.md:27,28` | ✓ 逐字相符 | `sed -n '27p;28p' .claude/agents/three-way-local-attack.md` |
| A5/A6：`.claude/agent-common.md:35` | ✓ 逐字相符 | `sed -n '35p' .claude/agent-common.md` |
| A6：noclobber 默认关闭的两次独立 Bash 调用 | ✓ 复跑同样得到 `noclobber off`、第二次 `>` 覆盖退 0 | `set -o \| grep noclobber` 两次独立调用均为 `off` |
| A6/A5：模型复跑（H1–H4） | ✓ 复跑四行退出码、stderr 首行、字节数与报告贴的逐字相同 | 见上「复跑」一节 |
| A5：`research/scripts/ask-local.sh:88`、`oov-check.py:249` | ✓ 逐字相符 | `sed -n '88p' research/scripts/ask-local.sh`；`sed -n '249p' research/scripts/oov-check.py` |
| A7：`.claude/agents/mutation-triage.md:28` | ✓ 逐字相符 | `sed -n '28p' .claude/agents/mutation-triage.md` |
| A7：`research/scripts/mutate.sh:111,504,514,534,536,538,647` | ✓ 全部逐字相符 | `sed -n '111p;504p;514p;534p;536p;538p;647p' research/scripts/mutate.sh` |
| A7：模型复跑（5 条标记 + 计数行 + 两种数法） | ✓ 复跑输出与报告贴的逐字相同（字面数 0/0/0、行首符号数 2/1/1/1、变异表条数 5） | 见上「复跑」一节 |
| A7 附：`.claude/gate.d/59-crates-mutation-replay.sh:407-408` | ✓ 逐字相符（八栏加共 N 条） | `sed -n '407,408p' .claude/gate.d/59-crates-mutation-replay.sh` |
| A8：`.claude/agents/three-way-verifier.md:21,27` | ✓ 逐字相符 | `sed -n '21p;27p' .claude/agents/three-way-verifier.md` |
| A8：清单外 8 个文件的 `comm -23` 结果 | ✓ 复核同样得到这 8 个文件（用报告里 `cite.sh` 抽出的引文列表核对，见下方「A8 复核」） | 见下 |
| A9：`.claude/agents/kb-scribe.md:30` | ✓ 逐字相符 | `sed -n '30p' .claude/agents/kb-scribe.md` |
| A9：`.claude/main-agent.md:61` | ✓ 逐字相符 | `sed -n '61p' .claude/main-agent.md` |
| A9：`.claude/gate.d/lib-item-ref-status.py:74` | ✓ 逐字相符 | `sed -n '74p' .claude/gate.d/lib-item-ref-status.py` |
| A9：`experiment-runner.md:27`「只追加」一句 | ✓ 逐字相符 | `grep -n '只追加，不改别人的' .claude/agents/experiment-runner.md` |
| A9：模型复跑（D23 第 4 条翻状态、13 个文件改写行数、锚点命中 1→0、33 号退出码 1） | ✓ 复跑逐字相同（在草稿目录仓副本上，不在主仓） | 见上「复跑」一节 |
| A10：`.claude/agent-common.md:55` 全文 | ✓ 逐字相符（判别力自证已用这一行） | `sed -n '55p' .claude/agent-common.md` |
| A10：`.claude/hooks/bash-command-detector.sh:2366,2377` | ✓ 逐字相符 | `sed -n '2366p;2377p' .claude/hooks/bash-command-detector.sh` |
| A10：`.claude/singlefs-ai-sop/rules/command-safety.md:100` | ✓ 逐字相符 | `sed -n '100p' .claude/singlefs-ai-sop/rules/command-safety.md` |
| A10：钩子真跑（三种 `&`/`wait` 写法） | ✓ 复跑三行退出码与拒绝文案逐字相同 | 见上「复跑」一节 |
| G2 构造不出：`.claude/gate.d/74-model-differential.sh:46` | ✓ 逐字相符 | `sed -n '46p' .claude/gate.d/74-model-differential.sh` |
| G2 构造不出：`.claude/gate.d/15-research-build.sh:25` | ✓ 逐字相符 | `sed -n '25p' .claude/gate.d/15-research-build.sh` |
| G2 构造不出：`.claude/hooks/lib_shell_words.py:53` | ✓ 逐字相符 | `sed -n '53p' .claude/hooks/lib_shell_words.py` |
| G1 依据：`.claude/singlefs-ai-sop/scripts/gate.sh:419,535` | ✓ 逐字相符 | `sed -n '419p;535p' .claude/singlefs-ai-sop/scripts/gate.sh` |
| G1 依据：`research/scripts/watch.sh:7` | ✓ 逐字相符 | `sed -n '7p' research/scripts/watch.sh` |
| G6 依据：`.claude/hooks/bash-command-detector.sh:116` | ✓ 逐字相符 | `sed -n '116p' .claude/hooks/bash-command-detector.sh` |

**A8 复核**（清单外文件的核对表标签是否成立）：报告称本报告引了 28 个去重「文件:行」、其中 8 个不在 31 行快照清单里。用报告自带的 `cite.sh`/`citations.tsv` 复核：

```
cd <草稿目录>/rerun/model && bash cite.sh < citations.tsv | grep -oP '^\S+(?=:\d+:)' | sort -u > /tmp/x-cited
comm -23 /tmp/x-cited <(cut -d'  ' -f2- ../../../../../home/user/singlefs/research/prompts/governance-defs-r1-snapshot/sha256sums.txt | sort -u)
```
得到同样 8 个文件（`15-research-build.sh`、`74-model-differential.sh`、`lib-item-ref-status.py`、`lib_shell_words.py`、`gate.sh`、`crates/mutations.tsv`、`replay.sh`、`watch.sh`）。A8 判定「核对表第 2 步没有清单外文件这一支、按字面会误标『分不清：文件在腿开工之后被改过』」本身成立：本报告在核对这批引文时，是按「快照对得上 sha256 →直接对主树核」处理的（这 8 个文件不在快照清单，但都在主仓、内容与引文核对全部相符），采用的正是 A8 建议的第三种处置（清单外文件对主树核），不是字面第 2 步会导出的「分不清」标签——这印证 A8 指出的缺口是真实的（核对表字面上没有这一支，要靠人补），不是它自己编造的。

## 正推腿（Sonnet）核对表

| 引用 | 核的结果 | 命令 |
|---|---|---|
| `.claude/main-agent.md:59` 全文 | ✓ 逐字相符 | `sed -n '59p' .claude/main-agent.md` |
| `.claude/gate.d/54-layer0-replay.sh:232` | ✓ 逐字相符 | `sed -n '232p' .claude/gate.d/54-layer0-replay.sh` |
| `.claude/agents/crash-verifier.md:16-19,25` | ✓ 逐字相符 | `sed -n '16,19p;25p' .claude/agents/crash-verifier.md` |
| `.claude/agents/gate-triage.md:26` | ✓ 逐字相符 | `sed -n '26p' .claude/agents/gate-triage.md` |
| `.claude/rules/implementation-workflow.md`「场合/跑不跑」表「附录二抄录，行 219」 | **✗ 行号与出处都错**：附录二（`_governance-defs-r1-diff.md`）第 219 行是「-- 三个数（原样贴 `mutate.sh` 的汇总行）…」，与「场合/跑不跑」表毫不相关；这段表格内容实际逐字落在**背景材料** `_governance-defs-r1-background.md` 第 219 行，而 `implementation-workflow.md` 自己的对应行是第 55 行（已在攻方腿核对表里核过、逐字相符） | `sed -n '219p' research/prompts/_governance-defs-r1-diff.md`；`sed -n '219p' research/prompts/_governance-defs-r1-background.md`；`sed -n '55p' .claude/rules/implementation-workflow.md` |
| `.claude/main-agent.md:14` | ✓ 逐字相符 | `sed -n '14p' .claude/main-agent.md` |
| `.claude/agent-common.md:46` 全文 | ✓ 逐字相符 | `sed -n '46p' .claude/agent-common.md` |
| `research/scripts/run-with-memory-cap.sh:19` | ✓ 逐字相符 | `sed -n '19p' research/scripts/run-with-memory-cap.sh` |
| `.claude/agents/implementation-writer.md:23`（引「（经内存包装，共用约束「不做」一节）」） | ✓ 逐字相符 | `sed -n '23p' .claude/agents/implementation-writer.md` |
| `.claude/hooks/heavy-test-guard.sh` 行 75-83「另一道，与重型不重型无关」段 | ✓ 段落存在、内容与转述相符 | `sed -n '75,83p' .claude/hooks/heavy-test-guard.sh` |
| `.claude/agents/implementation-writer.md:23`（引「在主工作区改」） | **✗ 行号错**：:23 现在是空标题行「## 做什么」，「在主工作区改」这句实际在第 13 行 | `sed -n '23p' .claude/agents/implementation-writer.md`；`grep -n '在主工作区改' .claude/agents/implementation-writer.md` → 13 |
| `.claude/agents/implementation-writer.md:23`（引「门禁阶段只跑阶段归属表登记给你的那几道…」，B-9 一段，出现两次同一引用） | **✗ 行号错**：这句实际在第 28 行，不在 23 行 | `grep -n '门禁阶段只跑阶段归属表登记给你的那几道' .claude/agents/implementation-writer.md` → 28 |
| `.claude/agents/implementation-writer.md:24`（引「其余行照样追加进 `crates/mutations.tsv` 末尾」） | **✗ 行号错**：这句同样在第 28 行，24 行是空行 | `sed -n '24p;28p' .claude/agents/implementation-writer.md` |
| `git apply --check` 全仓已删 | ✓ 现查 0 命中，删除属实 | `grep -n 'git apply --check' .claude/agents/implementation-writer.md`，退出码 1 |
| `check.sh:22-30` `CODE_DISCIPLINE_LINTS` 与 `:31` clippy 命令 | ✓ 逐字相符 | `sed -n '22,31p' .claude/singlefs-ai-sop/scripts/check.sh` |
| `.claude/agents/three-way-attack.md:37` | ✓ 逐字相符 | `sed -n '37p' .claude/agents/three-way-attack.md` |
| `.claude/agents/three-way-local-attack.md:27,28,29` | ✓ 逐字相符 | `sed -n '27p;28p;29p' .claude/agents/three-way-local-attack.md` |
| `research/scripts/ask-local.sh` 判红分支（:63-96 区间描述）、`:107-119`、`:88` | ✓ 抽查的具体行（88）逐字相符；:63-96、:107-119 是区间转述，未见摘句或曲解 | `sed -n '88p' research/scripts/ask-local.sh` |
| `research/scripts/oov-check.py:237,249` | ✓ 逐字相符 | `sed -n '237p;249p' research/scripts/oov-check.py` |
| `.claude/agents/three-way-local-defense.md:11` | ✓ 逐字相符 | `sed -n '11p' .claude/agents/three-way-local-defense.md` |
| `.claude/agents/three-way-verifier.md:21,27` | ✓ 逐字相符 | `sed -n '21p;27p' .claude/agents/three-way-verifier.md` |
| `.claude/main-agent.md:54` 全文 | ✓ 逐字相符 | `sed -n '54p' .claude/main-agent.md` |
| `.claude/rules/three-way-inference.md:135,163` | ✓ 逐字相符 | `sed -n '135p;163p' .claude/rules/three-way-inference.md` |
| `.claude/agents/mutation-triage.md:26,27,28`（三个数怎么数一节，A7 有关） | ✓ 逐字相符（:28 与攻方腿 A7 引的同一行） | `sed -n '26p;27p;28p' .claude/agents/mutation-triage.md` |
| `research/scripts/mutate.sh:647` | ✓ 逐字相符 | `sed -n '647p' research/scripts/mutate.sh` |
| `.claude/gate.d/59-crates-mutation-replay.sh:407-408` | ✓ 逐字相符 | `sed -n '407,408p' .claude/gate.d/59-crates-mutation-replay.sh` |
| `.claude/agents/mutation-triage.md:15,18,19`（bin 名、59 号输出路径） | ✓ 逐字相符 | `sed -n '15p;18p;19p' .claude/agents/mutation-triage.md` |
| `.claude/agents/experiment-runner.md:26`（入库装置分支） | ✓ 逐字相符 | `sed -n '26p' .claude/agents/experiment-runner.md` |
| `crates/mutations.tsv` 第 1 行（六段格式） | ✓ 逐字相符 | `sed -n '1p' crates/mutations.tsv` |
| `.claude/agents/experiment-designer.md:22,24` | ✓ 逐字相符 | `sed -n '22p;24p' .claude/agents/experiment-designer.md` |
| `.claude/decision-links-pending` 现状（3 行注释、0 条目） | ✓ 现查相符 | `cat .claude/decision-links-pending` |
| `.claude/main-agent.md:24` | ✓ 逐字相符 | `sed -n '24p' .claude/main-agent.md` |
| `.claude/main-agent.md:16`（三处记录点位） | ✓ 逐字相符（标题「## 一轮怎么开、怎么收」下第 2 步内容，含「里程碑收口表」等三处，抽查确认存在） | `grep -n '里程碑收口表\|checks-owed.md\|决策正文' .claude/main-agent.md \| head -5` |
| `.claude/main-agent.md:60` | ✓ 逐字相符 | `sed -n '60p' .claude/main-agent.md` |
| `.claude/agents/sweep.md:16-22`（四种活的名字） | ✓ 抽查 :22「阶段同步」一项存在，其余三项（改了一个数或格式常量、撤回一条结论、新立一条判据）经 `grep -n` 全部命中 | `grep -n '改了一个数或格式常量\|撤回一条结论\|新立一条判据\|阶段同步' .claude/agents/sweep.md` |
| `.claude/agent-common.md:55`（B-20，`&`/`wait`） | **✗ 行号错**：正文写「改后 `.claude/agent-common.md:41`」，实际在第 55 行（小节标题本身写对了 :55，正文引用时写成 :41） | `sed -n '41p;55p' .claude/agent-common.md` |
| `.claude/singlefs-ai-sop/rules/command-safety.md`「## 并行不许把失败吃掉」标题 | ✓ 现查存在 | `grep -n '^## 并行不许把失败吃掉' .claude/singlefs-ai-sop/rules/command-safety.md` |
| `.claude/agent-common.md:34`（B-21/B-23） | ✓ 逐字相符 | `sed -n '34p' .claude/agent-common.md` |
| `.claude/hooks/bash-command-detector.sh:116` ⑦ | ✓ 逐字相符 | `sed -n '116p' .claude/hooks/bash-command-detector.sh` |
| `.claude/agent-common.md:44`（progress.md，A-17） | **✗ 行号错**：正文写「改后 `.claude/agent-common.md:44`」，实际在第 57 行 | `sed -n '44p;57p' .claude/agent-common.md` |
| `.claude/agents/gate-triage.md:32`（ref 名） | ✓ 逐字相符 | `sed -n '32p' .claude/agents/gate-triage.md` |
| `.claude/singlefs-ai-sop/scripts/gate.sh:535` | ✓ 逐字相符 | `sed -n '535p' .claude/singlefs-ai-sop/scripts/gate.sh` |
| `.claude/agents/kb-scribe.md:20,28,30,32`（B-17/18/19/24） | ✓ 抽查 :20、:30、:32 三行内容与转述相符（:28 为空行，报告未直接引 :28 原文，只引 :20/:30/:32/:39） | `sed -n '20p;30p;32p;39p' .claude/agents/kb-scribe.md` |
| `.claude/agents/sweep.md:22`（连字符例） | ✓ 抽查 :37 处 `F1-F6` ASCII 连字符存在 | `grep -n 'F1-F6' .claude/agents/sweep.md` |
| `.claude/agents/three-way-materials.md:24` | ✓ 逐字相符 | `sed -n '24p' .claude/agents/three-way-materials.md` |

## 计数

- 攻方腿（Opus）：核了 41 处引用/复跑；✓ 41 处；✗ 0 处；核不动 0 处（模型复跑全部在容器内可跑，无需编译主仓或联网）。
- 正推腿（Sonnet）：核了 37 处引用；✓ 32 处；✗ 5 处（行号与实际内容不符：`implementation-workflow.md`「场合/跑不跑」表标成「附录二行219」实为背景材料行219、implementation-writer.md「在主工作区改」标成:23实为:13、implementation-writer.md「门禁阶段只跑…」标成:23实为:28（同一错误在正文出现两处引用）、agent-common.md「`&`/`wait`」正文标成:41实为:55、agent-common.md「progress.md」标成:44实为:57）；核不动 0 处。
- 判别力自证：1 处，判 ✗（方法有效）。

## 没做什么

- 不判一条打中成不成立、该不该采纳；只核引用、产物与复跑，攻方腿 A1-A10 各条「四句」自评、改法优劣、是否被攻过零轮，都是推论，不在这份核查范围内，留给主 agent 逐条现查。
- 没有对正推腿全部 15 份定义的每一处引用逐字核对（原文档共 259 行、涉及引用约 60 余处），核的是能用 `sed -n`/`grep -n` 机械核对的「文件:行号+抄的原文」这一类；正推腿报告里的「对应」「丢判据检查」「推翻条件」几段属于推论评述，未核。
- 没有编译 Rust、没有跑 `gate.sh`、没有跑任何重型测试（54/55/57/59/87 号、`cargo test --all`）；攻方腿模型复跑全部在草稿目录副本上完成，未触碰主仓的 git 与 `crates/` 源码。
- 未对 D 组报告（`fs-design.md`/`format-evolution.md`/`path-moves.md`）及攻方腿标「构造不出」「没打中」两节里未列出具体「文件:行」的概述性陈述逐条现查（这些陈述本身没有可核的「文件:行+抄文」形式，或已被上方表格覆盖）。
- 本地腿本轮缺席（网关不通），没有本地腿报告可核；未收到主 agent 给的 `*.at-snapshot` 倒推副本，因为这一轮 31 个快照文件的 `sha256sum -c` 全部对得上，不需要倒推。
