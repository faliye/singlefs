# governance-defs-r3 核查员报告

我交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

## 判别力自证

挑 Sonnet 报告引的 `.claude/agents/mutation-triage.md:28`（该文件在开工快照清单里）。原样第 28 行核对：现文第 28 行确为 Sonnet 引的那句（「…59 号照它自己收尾的「计数：」行，那一行没有 `💥`、`⚠️` 两栏：测试进程没跑完的，输出里有一行 `error:` 开头（测试二进制被信号杀时 cargo 也打 `error: test failed`）或 `could not compile` 就记「无效」，否则记「没红」…」）。把引用行号加 1（改成 :29）后核对：`.claude/agents/mutation-triage.md` 第 29 行内容是「6. 没红与无效的逐条按 `mutation-sampling.md` 分类；判「取样点不敏感」的…」，与被引原文完全不同，判 **✗**。判别力自证生效，方法能分辨。

## 快照核验

`sha256sum -c research/prompts/governance-defs-r3-snapshot/sha256sums.txt`：41 行全部 `OK`，与主 agent 交回的一致，主树可直接用于核对。`git status --short` 在 `/home/user/singlefs` 输出为空，工作区干净。

## 报告文件 sha256 核对

- `governance-defs-r3-sonnet-output.md`：现算 `50c136c14759fc3927d9142253330feae1773f46ea22b16da13da16788b8e50f`，与交回给的一致。
- `governance-defs-r3-opus-output.md`：现算 `ba86f77755a78a60176b55e09422e1b74fc9735217861d2d1fbf016cd4f6f83b`，与交回给的一致。
两份报告均未被交回之后改过，正文核对有效（不记「分不清：报告在腿交回之后被改过」）。

## Sonnet（云端正推）报告核对表

全部引用文件均在开工快照清单里，`sha256sum -c` 全通过，逐条对主树该文件该行核对。

| 引用 | 抄的内容/性质 | 核的结果 | 命令 |
|---|---|---|---|
| `.claude/agents/mutation-triage.md:28` 与 `.claude/rules/mutation-sampling.md:82` 同句 | 引「59 号照它自己收尾的「计数：」行…」整句 | ✓ 两处逐字相同，且与现文该行逐字相同 | `sed -n '28p' .claude/agents/mutation-triage.md`；`awk 'NR==82' .claude/rules/mutation-sampling.md` |
| `mutation-sampling.md` 无第九类/「信号杀」/「process didn't exit」新增条目 | 判定依据 | ✓ 全文件 grep 只命中本处一行 | `grep -n '第九类\|信号杀\|process didn.t exit' .claude/rules/mutation-sampling.md` |
| `research/prompts/governance-review-r2-report-B.md:9,10,11,12,13,14,15,16,17` | 乙组各条建议原文 | ✓ 逐行核对，全部与文件该行原文一致 | `sed -n '9,17p' research/prompts/governance-review-r2-report-B.md` |
| `research/prompts/governance-review-r2-report-A.md:11,15,17` | 甲组 A3/A7/A9 原文 | ✓ 与文件该行一致 | `sed -n '11p;15p;17p' research/prompts/governance-review-r2-report-A.md` |
| `research/prompts/governance-review-r2-report-C.md:9,19` | 丙组第 1、11 条原文 | ✓ 与文件该行一致 | `sed -n '9p;19p' research/prompts/governance-review-r2-report-C.md` |
| `.claude/agents/crash-verifier.md:27` | 「…git ls-files --others --exclude-standard -- <这些路径>」（旧引用比对） | ✓ 与现文一致 | `sed -n '27p'`（注：Sonnet 引 :27 的是旧文改动前一句，判定文字见下条 :24/:36-37） |
| `.claude/agents/crash-verifier.md:36-37` | 「一张表：阶段 / 退出码…/ 日志路径…」 | ✓ 与现文一致 | `sed -n '36,37p' .claude/agents/crash-verifier.md` |
| `.claude/agents/mutation-triage.md:19` | 「分 crates/mutations.tsv 里的条目时：提交时那一次门禁 59 号的输出路径（缺它不开工）。」 | ✓ 与现文一致 | `sed -n '19p' .claude/agents/mutation-triage.md` |
| `git diff 1c58cfa bfc447e -- .claude/agents/mutation-triage.md` 只改第 27、28 行 | 复跑命令 | ✓ 复跑 `git diff`，唯一 hunk 是 `@@ -24,8 +24,8 @@`（覆盖 24-31 行，实际改动内容只在 27-28 两行） | `git diff 1c58cfa bfc447e -- .claude/agents/mutation-triage.md` |
| `.claude/agents/three-way-verifier.md:21,22,27,28` | 引核查员定义自身多处 | ✓ 逐行与现文一致（本报告自身也是这份定义的执行者，行号与内容核对无误） | `sed -n '21p;22p;27p;28p' .claude/agents/three-way-verifier.md` |
| `.claude/rules/implementation-workflow.md:26`，标题 `:24` | 「派三方的腿之前…连同派腿的时刻（date -u）交核查员当输入」 | ✓ 与现文一致 | `sed -n '24,26p' .claude/rules/implementation-workflow.md` |
| `git diff 1c58cfa bfc447e -- .claude/rules/implementation-workflow.md` 只改「重型测试」表附近，不改这一节 | 复跑命令 | ✓ 复跑确认该节未在 diff 范围 | 同上（diff 未在此报告重复贴，已现查） |
| `.claude/agents/experiment-runner.md:27,30,32,39` | bin 名连字符化、编译位置、driver 登记、写范围一节 | ✓ 与现文逐字一致 | `sed -n '27p;30p;32p;39p' .claude/agents/experiment-runner.md` |
| `git diff 1c58cfa bfc447e -- .claude/agents/experiment-runner.md` 只改第 27、30、32 行 | 复跑命令 | ✓ 唯一 hunk `@@ -24,12 +24,12 @@`（覆盖 24-35 行，实改内容确认在 27/30/32） | `git diff 1c58cfa bfc447e -- .claude/agents/experiment-runner.md` |
| `.claude/hooks/agent-write-scope.tsv` 第 16 行 `experiment-runner\treplay.sh\t复跑登记` | 写范围闸表 | ✓ 与现文一致（现文措辞「复跑登记行」，Sonnet 引文写「复跑登记行」/正文摘述一致） | `grep -n '^experiment-runner' .claude/hooks/agent-write-scope.tsv` |
| `git diff 1c58cfa bfc447e -- .claude/hooks/agent-write-scope.tsv` 零改动 | 复跑命令 | ✓ 复跑无输出，确认零改动 | `git diff 1c58cfa bfc447e -- .claude/hooks/agent-write-scope.tsv` |
| `.claude/agents/kb-scribe.md:20,32` | N 项未定写法、写入后钩子清单 | ✓ 与现文逐字一致 | `sed -n '20p;32p' .claude/agents/kb-scribe.md` |
| `.claude/gate.d/75-decision-experiment-links.sh:226` | 正则含 `\s*` | ✓ 与现文一致 | `sed -n '226p' .claude/gate.d/75-decision-experiment-links.sh` |
| `.claude/gate.d/20-kb-shape.sh:214`（Sonnet 原文引 `:213`/`:214` 两种写法，均核对） | 正则不含 `+` | ✓ 与现文一致（现文第 214 行） | `sed -n '214p' .claude/gate.d/20-kb-shape.sh` |
| `git diff 1c58cfa bfc447e -- .claude/gate.d/20-kb-shape.sh` 只改注释、未碰该行 | 复跑命令 | ✓ diff 唯一改动是第 43-49 行附近的注释文字，第 214 行（正则）确未出现在 diff 里 | `git diff 1c58cfa bfc447e -- .claude/gate.d/20-kb-shape.sh` |
| `.claude/hooks/kb-scribe-followups.tsv` 相关行（21/30/49 号触发条件） | 转述 | 核不动（表格具体行号 Sonnet 未点名精确行号，只引用 grep 结果） | — |
| `.claude/agent-common.md:46`「253 是超过限时」等整段 | 与甲组 A7 对应 | ✓ 与现文一致，且末尾「退出码表以 run-with-memory-cap.sh 文件头为准」也与现文一致 | `sed -n '46p' .claude/agent-common.md` |
| `.claude/main-agent.md:59` | 「退出码是经内存包装的那条 --full 命令的，250–254 是包装自己的结局」 | ✓ 与现文一致 | `sed -n '59p' .claude/main-agent.md` |
| `.claude/gate.d/54-layer0-replay.sh:230` | 同句 | ✓ 与现文一致 | `grep -n '退出码是经内存包装的那条' .claude/gate.d/54-layer0-replay.sh` |
| `.gitignore:6` `/.claude/singlefs-ai-sop/`；`git ls-files .claude/singlefs-ai-sop/scripts/lib.sh` 零输出 | 判定依据 | ✓ 现查确认：`git check-ignore -v` 命中 `.gitignore:6`，`git ls-files` 零输出 | `git check-ignore -v .claude/singlefs-ai-sop/scripts/lib.sh`；`git ls-files .claude/singlefs-ai-sop/scripts/lib.sh` |

Sonnet 报告没有出现产物引用（没有交模型或复跑命令），本表覆盖它全部可核的文字引用。

**Sonnet 计数**：核了 24 处（含 3 处复跑 diff 命令），✓ 24 处，✗ 0 处，核不动 1 处（kb-scribe-followups.tsv 具体行号未点名），分不清 0 处。

## Opus（云端攻方）报告核对表

产物：模型目录 `research/prompts/governance-defs-r3-opus-model/`（22 个文件）、复跑命令 `bash research/prompts/governance-defs-r3-opus-model/run-all.sh <草稿目录>`、原样输出 `run-all.out`。

### 模型文件哈希核对

在仓里现算 22 个文件的 sha256，与报告第 16-37 行列出的逐条比对：**全部 22 行完全一致**（含 `run-all.out` 自身）。

命令：`cd research/prompts/governance-defs-r3-opus-model && find . -type f | sort | xargs sha256sum`

### 复跑

在草稿目录 `/tmp/claude-0/.../defs-r3-verifier/rerun` 下用 `nice -n 19 bash research/prompts/governance-defs-r3-opus-model/run-all.sh <该草稿目录>` 重跑，退出码 0，输出 200 行；把两份输出里的 `/tmp/…` 路径都替换成 `<TMP>` 后 `diff` 无差别，与报告「在另一个草稿目录再跑一次…diff 无差别」的说法一致。`git status --short` 复跑前后均为空，确认脚本未碰仓里文件。

### 引用与产物核对表

| 引用/产物摘录 | 核的结果 | 命令 |
|---|---|---|
| `.claude/gate.d/59-crates-mutation-replay.sh:326,338,342,344,348` | ✓ 5 处逐字与现文一致 | `awk -v n=<行号> 'NR==n' .claude/gate.d/59-crates-mutation-replay.sh` |
| `.claude/gate.d/59-crates-mutation-replay.sh:176` `COPIED_INTO_EACH_SHARD = [...]` | ✓ 与现文一致 | `awk 'NR==176' .claude/gate.d/59-crates-mutation-replay.sh` |
| `crates/mutations.tsv:7`、`:688`（cargo 参数、第 6 列测试名） | ✓ 与现文一致；`:688` 参数 `-p singlefs-harness --bin e156_allocation_basis_counts`、第 6 列 `accounti…$` 前缀确认 | `awk -F'\t' 'NR==688{print}' crates/mutations.tsv`；`NR==7` 同 |
| `crates/mutations.tsv` 表里成形 724 行 | ✓ 与报告一致 | `awk -F'\t' 'NF>1' crates/mutations.tsv \| wc -l` → 724 |
| `crates/mutations.tsv` 第 202、297、298、684–689、693、694 行第 6 列带 `$` 共 11 行 | ✓ 与 `run-all.out` 里 g1-judge.py 段列出的 11 行完全一致（首尾行核对，行号、函数名、判档说明逐字相同） | `sed -n '13,25p' run-all.out` 对 `crates/mutations.tsv` 对应行核对 |
| `a1f4691`（2026-09-24）、`346f5e6`（2026-09-25）两次提交把 11 行之一带入表 | ✓ 两个提交日期核实相符 | `git show -s --format='%h %ad' --date=format:'%Y-%m-%d' a1f4691`；`git log --format='%h %ad' -1 -S '…anchor$' -- crates/mutations.tsv` |
| `.claude/hooks/agent-write-scope.tsv:7` `implementation-writer\tlitmus/**` | ✓ 与现文一致 | `awk 'NR==7' .claude/hooks/agent-write-scope.tsv` |
| `.claude/hooks/agent-write-scope.tsv:16`（experiment-runner 那 13 行、`replay.sh` 一行） | ✓ 与现文一致；experiment-runner 段确为 11-23 共 13 行 | `grep -n '^experiment-runner' .claude/hooks/agent-write-scope.tsv` |
| `.claude/gate.d/57-lkmm.sh:18,22`；`.claude/scripts/lkmm.sh:296`（读仓根标记、`source lib.sh`、`exit 3`） | ✓ 与现文一致 | `awk 'NR==18\|\|NR==22' .claude/gate.d/57-lkmm.sh`；`grep -n 'singlefs-ai-sop/scripts/lib.sh' .claude/scripts/lkmm.sh`；`awk 'NR==296' .claude/scripts/lkmm.sh` |
| `.claude/gate.d/55-qemu-first-transaction.sh:122,303`（`.qemu-prerecorded`、`exit 3`） | ✓ 与现文一致 | `awk 'NR==122\|\|NR==303' .claude/gate.d/55-qemu-first-transaction.sh` |
| `research/scripts/replay.sh:637,456,602,616,619` | ✓ 5 处逐字与现文一致 | `awk 'NR==<行号>' research/scripts/replay.sh` |
| `replay.sh` 里 `@driver_e<号>` 登记：14 个 E158、1 个 E9、1 个 E156、1 个 E142 | ✓ 现数与报告一致 | `grep -oE '@driver_e[0-9]+' research/scripts/replay.sh \| sed -E 's/@driver_e([0-9]+).*/\1/' \| sort \| uniq -c` |
| `research/scripts/run-with-memory-cap.sh:18,27,132`（退出码表、`exit "$command_exit"`） | ✓ 与现文一致 | `awk 'NR==18\|\|NR==27\|\|NR==132' research/scripts/run-with-memory-cap.sh` |
| `.claude/agents/crash-verifier.md:24` 整句（未跟踪检查、git diff --quiet） | ✓ 与现文一致 | `sed -n '24p' .claude/agents/crash-verifier.md` |
| `.claude/rules/implementation-workflow.md:24,26` | ✓ 与现文一致（同 Sonnet 表） | 同上 |
| `.gitignore:6`、`.gitignore:14` | ✓ 与现文一致 | `sed -n '1,15p' .gitignore` |
| `.claude/agents/three-way-verifier.md:21,22,27,28` | ✓ 与现文一致（同 Sonnet 表） | 同上 |
| `run-all.out` 里 g1-judge.py / g2-untracked.sh / g3-git-blind.sh / g3-who-records-time.sh / g4-driver-name.sh / g5-exit-two.sh 各段，报告正文引用的表格与历史行 | ✓ 逐段与 `run-all.out` 原文逐字比对（D1–D7 各节的引用块），全部命中，无摘句或改写 | `sed -n '1,102p' research/prompts/governance-defs-r3-opus-model/run-all.out` |
| `run-hooks.sh` 段：60 次 `rc=0`（10 条命令 × 前台/后台 × 3 钩子）+ 对照组 8 行 `rc=0` + 4 行 `rc=2` | ✓ 现数：`grep -c 'rc=0'` 总 75（减 7 个脚本段尾各 1 行 `rc=0`＝68，符合「60+8」）；`grep -c 'rc=2'` 为 4，与「检出记录条数：4」一致 | `grep -c 'rc=0' run-all.out`；`grep -c 'rc=2' run-all.out`；`grep '检出记录条数' run-all.out` |
| `git diff --stat bfc447e HEAD -- .claude research/scripts crates` 无输出（报告开头「开工核过」一行） | 核不动（该现查是 Opus 腿开工时刻的状态，我核查时刻在其后，无法反推腿开工那一刻的仓库状态是否真是那样；但可核对**现在**同一命令的输出） | 现查同一命令，现在同样无输出，与「这一轮没有额外改动」的说法相容，不构成推翻，但也不能倒推腿开工那一刻 |
| D1、D2 关于 cargo/libtest 输出形状「按代码推、没有真跑」 | 报告自己标注为推测（「这一步没真跑」「合成输出的形状是推的」），不是当作已核实的观测 | 不核（Opus 自己已标「核不动」/「推的」，未冒充观测） |

### 复跑一致性专项

`run-all.sh` 只调用 `bash`、`python3`、`git`（`grep -oE '\b(curl|wget|ssh|scp|nc)\b'` 在全部脚本与 `.cmd` 文件里零命中）；复跑前后 `git status --short` 均为空，证实脚本没有写仓库文件。

**Opus 计数**：模型文件哈希核对 22 个文件全部一致（单独一项，不计入下面 21 行）；复跑一致性 1 次，一致（单独一项，不计入下面 21 行）；「引用与产物核对表」共 21 行：✓ 19 行、核不动 1 行（腿开工那一刻仓库状态，现在核不出）、不适用 1 行（D1/D2 的合成输出，Opus 自己已标「推的」「没真跑」，不是它当观测报的东西，不计入 ✓/✗/核不动）；✗ 0 处，分不清 0 处。

## 总计数

- Sonnet 报告：核了 24 处（21 处文字引用/复跑 diff + 1 项 grep 判定依据 + 2 项现查确认），✓ 24 处，✗ 0 处，核不动 1 处，分不清 0 处。
- Opus 报告：模型文件哈希核对 22 个文件全部一致；复跑一次一致；「引用与产物核对表」21 行中 ✓ 19、核不动 1、不适用 1（Opus 自称推的部分）；✗ 0 处，分不清 0 处。
- 两份报告合计：核了约 46 处可核事实，✓ 43 处，✗ 0 处，核不动 2 处，分不清 0 处，不适用 1 处（腿自称推测、非观测的部分）。

## 没做什么

- 不判 G1–G5 各条推论打中成不成立、该不该采纳，也不判两条腿之间谁的判定更对；这些交主 agent 逐条现查裁定。
- 未复核 cargo/libtest 在真实编译下对信号杀死测试进程时是否确实打印 `error: test failed`（Opus D1、D2 与 Sonnet 判定都依赖这一条，两条腿都自称是「推的，没真跑」；仓里唯一现存的真实证据是 `governance-defs-r2-opus-model/g3-outputs/crash-before-named-test.txt` 与 `research/scripts/mutate.sh:496` 的旧注释，Sonnet 已引用，Opus 未重新核实——本报告同样未再跑 59 号或造一次真实的 abort 变异，因为那是重型测试，子 agent/核查员不跑）。
- 未核 `.claude/hooks/kb-scribe-followups.tsv` 里 21/30/49 号触发条件的精确行号（Sonnet 只给了 grep 命中方式，未点名行号，本报告因此记「核不动」，不是 ✗）。
- 未反推 Opus 报告开头「开工核过快照…`git diff --stat bfc447e HEAD -- .claude research/scripts crates` 无输出」这一句在腿实际开工那一刻（2026-09-26 约 22:20 UTC）是否为真：这是一次性状态断言，现在（核查时刻，晚于开工）重跑同一命令依然无输出，但不能据此倒推开工那一刻的仓库状态，记「核不动」。
- 未对 Sonnet、Opus 两份报告之间的判定分歧（例如 D1/D2 与 Sonnet G1 判定对同一处「无效」判档问题的描述角度不同）做仲裁，两条腿的角度本身就不完全重叠（Sonnet 判「后半句留白」，Opus 判「次序相反」/「第三类来源」），这些差异是否互斥、是否都成立，留给主 agent。
- 未跑任何重型测试（54、55、57、59、87、整轮门禁、`gate-staged.sh`），未编译，未执行 cargo。
- 只写了本报告文件与草稿目录 `/tmp/claude-0/-home-user-singlefs/77b4b1be-00af-5f85-8659-f6e6d51fb840/scratchpad/defs-r3-verifier/`（含一次模型复跑的输出 `rerun.out`），未改仓里任何文件；`git status --short` 复跑前后均为空，已现查确认。
