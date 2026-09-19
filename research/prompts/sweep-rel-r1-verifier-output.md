# sweep-rel-r1 核查员报告

你交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

## 判别力自证

挑 Opus 腿引用的 `.claude/agents/sweep.md:35`（第 7 步整句），在草稿目录副本里把行号改成 36 后核对：

```
$ sed -n '36p' /tmp/claude-1000/sweep-rel-r1-verifier/snapshot-check/sweep.md
8. 写完跑 `stale-candidates.py --check-facts 事实表 --base 基准 [--target 结束]`，退出码不是 0 就补完再跑；过了再跑 `--facts 事实表 --base 基准 [--target 结束] --out 候选表`，把事实表、候选表与两条命令的输出末行交回。
```

第 36 行是第 8 步的文字，与 Opus 腿引用的第 7 步原句完全不同 ⇒ 判 ✗。核查方法有分辨力，继续往下做。

## 开工核验

- 7 个快照文件（`research/prompts/sweep-rel-r1-start-snapshot.sha256`）现场 `sha256sum -c` 全部 OK，与工作区当前内容逐字节一致 ⇒ 对这 7 个文件的核对，用工作区等价于用快照。
- 报告文件 sha256：`research/prompts/sweep-rel-r1-opus-output.md` 现算 `ef859e6bc76231e83e4714a03831b0438763684cd37a6923b06386dea3fa86bd`，与交回一致；`research/prompts/sweep-rel-r1-sonnet-output.md` 现算 `bb2053d07d7bf4365fe73dd58263c3bda1e7e825b8875b09b746d5f0bce24b1d`，与交回一致。两份都按交回给的哈希入册。
- 草稿目录：`/tmp/claude-1000/sweep-rel-r1-verifier/`（含 `opus-model-copy/` 模型副本、`tables/` 复跑产物、`snapshot-check/` 判别力自证副本、`check_41828.py` 独立核验脚本）。

## 一、Opus 攻方腿（research/prompts/sweep-rel-r1-opus-output.md）

复跑方式：把 `research/prompts/sweep-rel-r1-opus-model/` 整份拷进 `/tmp/claude-1000/sweep-rel-r1-verifier/opus-model-copy/`，脚本内部只读仓（`ROOT=os.getcwd()`），产物写进 `/tmp/claude-1000/sweep-rel-r1-verifier/tables/`，未写回仓里任何文件。拷贝前后 7 份 `.py`/`.sh` 的 sha256 与报告里登记的完全一致。

| 引用 / 复跑对象 | 核的结果 | 命令 |
|---|---|---|
| `research/scripts/stale-candidates.py` 行号 10/23/25/26/82/83/255/281/292/296/301/303/322/332/338/339/366/379/392/404（共 20 处） | ✓ 全部逐行核对，内容与报告引文逐字一致 | `awk -v n=<N> 'NR==n' research/scripts/stale-candidates.py` |
| `.claude/agents/sweep.md:35`（第 7 步整句） | ✓（即判别力自证用的那一条） | 见上 |
| `.claude/main-agent.md:43`（阶段同步那一行） | ✓ | `awk 'NR==43' .claude/main-agent.md` |
| `.claude/gate.d/47-research-script-selftests.sh:27` | ✓，且报告如实标注这是工作区未提交改动，未误当已提交内容 | `awk 'NR==27' .claude/gate.d/47-research-script-selftests.sh`；`git diff` 确认改动存在 |
| `attack_facts.py` → T0/T1/T1b/T3/T4/T5 六张表（罩住数、候选行数、退出码） | ✓ 逐字段完全重现，含 T3/T4/T5 的换行数（17/17/17） | `python3 opus-model-copy/attack_facts.py $T research/scripts/stale-candidates.py`，与 `attack_facts.out` 逐字节比对一致 |
| `probe_gates.py` → T6（41 行、退出码 0、罩住 0、候选 7 行）+ 候补闸 a/b/c 六表 | ✓ 逐字段完全重现 | `python3 opus-model-copy/probe_gates.py $T` |
| `probe_gates2.py` → 候补闸 c′/d 两表 | ✓ 逐字段完全重现 | `python3 opus-model-copy/probe_gates2.py $T` |
| `probe_cap.py` → T7（9 行 `&&` 收窄、退出码 6、罩住 25、候选 4190）+ 14 个自然概念名词命中数 | ✓ 逐字段完全重现，含 F7b（37→368 行）等全部 9 行明细 | `python3 opus-model-copy/probe_cap.py $T` |
| `attack_report.sh` → R1–R4（退出码 0）、R5（退出码 3）、`--groups` 七组（含 `F1-G3→['F1','F2','F3']`、`F6-F1→set()`、F1-F12 与 F7b 候选数 218/37） | ✓ 与 `attack_report.out` diff 后（仅替换草稿路径字段）完全相同，`diff` 退出码 0 | `bash opus-model-copy/attack_report.sh $T` |
| `probe_fixes.py` → 上限 150/500 对 T0/T7 的退出码、`--groups` 四种改法判定 | ✓ 逐字段完全重现 | `python3 opus-model-copy/probe_fixes.py $T` |
| `mutate_tool.py` → 10 个变异（M1–M10）的 `--selftest`/`--benchmark` 退出码与末行 | ✓ 逐字段完全重现，M8 selftest=5、M9 benchmark=4、其余 8 个（含空变异 M10）两条都 0 | `python3 opus-model-copy/mutate_tool.py $T/mutants` |
| `research/prompts/knowledge-rot-2026-09-18-sync.md` 里 `.md:数字` 引用计数「42 处」 | ✓ | `grep -cE '\.md:[0-9]+' research/prompts/knowledge-rot-2026-09-18-sync.md` |
| 共用约束 awk：`three-way-attack` 在 `stage-owners.tsv` 里挂 0 个阶段 | ✓ | 按报告给的 awk 原样跑，输出 0 |
| `research/scripts/relay-timing-lint.py:481`（R5 表格切格 `\|` 转义误红机制）与 `table_rows_under` 按裸 `\|` 切格 | ✓ | `awk 'NR==351' research/scripts/stale-candidates.py`、`grep -n 'def table_rows_under'` |
| `grep -rn 'stale-candidates' .claude/gate.d/*.sh` 命中只有 47 号 | ✓ | 同一条命令重跑，只命中 47 号 |
| 哈希候选空间「41828」与「合成三处、反查得到 3 处」 | 核不动（报告的「复跑」清单里没有对应脚本，7 个 `.py`/`.sh` 都不产出这一段）；用我自己写的等价脚本（同样调用 `sc.current_state_corpus`/`sc.location_hash`）独立核出 41828 与反查一致 True，数值吻合，但不是该腿脚本本身的复跑 | `/tmp/claude-1000/sweep-rel-r1-verifier/check_41828.py`（自建，非该腿产物） |

Opus 腿这一段：核了 17 类引用/复跑对象（含 20 处行号并作 1 类计），✓ 16 类，核不动 1 类（41828 demo 无原始脚本，已用等价脚本独立核实数值本身为真），✗ 0 类。

## 没做什么（Opus 腿这一段）

- 没有去核「候补闸与改法只在这条腿自己的模型上量过、被攻过零轮」这句自陈本身的真假（这是它自己标注的限度，不是需要复核的事实）。
- 没有验证「没打中的形状」表格里六条的正确性（均为按代码读出、未跑，报告已如实标「没跑」）。
- 不判 T7、T5 等格「打中」是否真的落在 A2 触发列字面内——报告自己也写明这是要主 agent 判的分歧点。

## 二、Sonnet 正推腿（research/prompts/sweep-rel-r1-sonnet-output.md）

| 引用 / 复跑对象 | 核的结果 | 命令 |
|---|---|---|
| `.claude/agents/sweep.md:22`（阶段同步输入清单那一条项目符号） | ✓ | `awk 'NR==22' .claude/agents/sweep.md` |
| `.claude/agents/sweep.md:38`（第 10 步「读完分到的组」） | ✓ | `awk 'NR==38' .claude/agents/sweep.md` |
| `.claude/agents/sweep.md:37`（第 9 步整句，带日期分句判据） | ✓ | `awk 'NR==37' .claude/agents/sweep.md` |
| `.claude/agents/sweep.md:41–43`（写范围一节） | ✓ | `sed -n '41,43p' .claude/agents/sweep.md` |
| `.claude/main-agent.md:43`（逐行判段派发只给候选表与组号） | ✓ | `awk 'NR==43' .claude/main-agent.md` |
| `.claude/gate.d/73-research-gate-lint.sh` 引文「命中第 3、4、86 行」（`research/prompts/ 下的脚本是冻结证据，不扫` 与 `for directory in "$ROOT/research/scripts" ...`） | ✗ 实际位置：`grep -n 'ROOT/research/scripts\|ROOT/.claude/hooks\|不扫'` 重跑命中的是第 6 行与第 13 行，不是 3、4、86 行；该文件是这一轮 7 个快照文件之一、内容与开工时逐字节一致，不存在「腿交回之后被改过」的可能，是纯粹的行号引用错误。引文内容本身（两句话）与第 6、13 行逐字相符 | `grep -n 'ROOT/research/scripts\|ROOT/.claude/hooks\|不扫' .claude/gate.d/73-research-gate-lint.sh` |
| `.claude/doc-lint-exclude:4`（research/prompts/ 排除理由） | ✓ | `awk 'NR==4' .claude/doc-lint-exclude` |
| `.claude/singlefs-ai-sop/scripts/gate-lint.sh:39`（GATE_LINT_DIR 注释）、`:48`（`SCAN=` 赋值） | ✓ | `awk -v n=<N> 'NR==n' .claude/singlefs-ai-sop/scripts/gate-lint.sh` |
| 同文件 `:51`（`SCANS=("$SCAN")`） | 分不清：文件可能在腿交回之后被改过——`.claude/singlefs-ai-sop/` 由 `.gitignore` 排除（`git check-ignore -v` 确认），不在这一轮给的 7 个快照文件里，git 里也查不到它的历史；现查该行内容实际落在第 52 行，与报告差 1。这一处不计入 ✗，单列 | `git check-ignore -v .claude/singlefs-ai-sop/scripts/gate-lint.sh`；`grep -n 'SCANS=("\$SCAN")' .claude/singlefs-ai-sop/scripts/gate-lint.sh` → 52 |
| `.claude/singlefs-ai-sop/scripts/selftest.sh:150/181/334`（`GATE_LINT_DIR=` 三处显式钉死扫描目录） | ✓（三处行号与内容都对，间接佐证上一条的「51」更像单纯引用误差而非文件漂移，但仍按无快照记「分不清」，不升级为 ✗） | `grep -n 'GATE_LINT_DIR=' .claude/singlefs-ai-sop/scripts/selftest.sh` |
| `.claude/kb/milestone/02-second-txn.md:222`（`00c9d4f` 那一版，「现状（2026-09-17 部分落地）……26 条不变量的 checker」） | ✓，且 `git diff 00c9d4f` 的两处改动区间（`@@ -110,7 +110,7@@`、`@@ -210,12 +210,12@@`）都不覆盖第 222 行，与报告「这一行未被改动」的核实一致 | `git show 00c9d4f:.claude/kb/milestone/02-second-txn.md \| sed -n '222p'`；`git diff 00c9d4f -- .claude/kb/milestone/02-second-txn.md` |
| `.claude/kb/checks-owed.md:24`（`00c9d4f` 那一版「23 条」；当前工作区已改「29 条」） | ✓ 两个版本内容都核对一致 | `git show 00c9d4f:.claude/kb/checks-owed.md \| sed -n '24p'`；`sed -n '24p' .claude/kb/checks-owed.md` |
| `.claude/agent-common.md:27`（Read/Bash 排他新建）、`:23–30`（写一节）、`:32–43`（不做一节） | ✓ | `grep -n '^#' .claude/agent-common.md`；`sed -n '23,30p'`/`sed -n '32,43p'` |
| A4-2 实测复现：red 样本「共 17 个脚本、148 条拒绝」 | ✓ 逐字重现 | `env -u GATE_BASE -u GATE_STAGED_FROM bash .claude/gate.d/73-research-gate-lint.sh "$work"`（red 拷贝） |
| A4-2 实测复现：green 样本「17 个脚本、147 条拒绝都带了出路」 | ✓ 逐字重现 | 同上（green 拷贝） |
| A4-3：门禁 73 号对真仓直接跑，`relay-timing-lint.py:481`、「共 67 个脚本、253 条拒绝」、`exit=1` | ✓ 逐字重现 | `env -u GATE_BASE -u GATE_STAGED_FROM bash .claude/gate.d/73-research-gate-lint.sh` |
| `research/scripts/relay-timing-lint.py:481` 引文内容 | ✓ | `awk 'NR==481' research/scripts/relay-timing-lint.py` |

Sonnet 腿这一段：核了 20 处引用/复跑对象，✓ 18 处，✗ 1 处（`73-research-gate-lint.sh` 引文行号 3/4/86，实际 6/13——该文件在快照内，判定确凿），分不清 1 处（`gate-lint.sh:51` vs 实际 52，文件不在快照内，不能排除腿交回后被改）。

## 没做什么（Sonnet 腿这一段）

- 未验证 A1-2、A1-3 的推论本身是否成立（是否真构成歧义/漏洞），只核了引文行号与内容，判决交主 agent。
- 未验证「重派之后要不要再抽验」这类流程缺口是否真的会导致漏检，只核了 sweep.md/main-agent.md 全文里确实找不到相反的句子（Sonnet 自陈的检索方式合理，未另行扩大搜索范围复核穷尽性）。
- `.claude/singlefs-ai-sop/scripts/gate-lint.sh` 与 `selftest.sh` 未获这一轮快照覆盖，只能对工作区现查，核不到腿开工时刻的状态。

## 三、本地攻方腿（sweep-rel-r1-local-attack*.md）

被核对象：`sweep-rel-r1-local-attack.md`（提示）、`-translation-audit.md`（译文核对表）、`-output-s1.md`/`-output-s2.md`（两次抽样）、`-runlog.md`（运行记录）。本地腿无 Bash，全部引文行号由派发这份材料的人在 `git show 00c9d4f:文件` 下现查，核查员逐条重跑同一条命令核对。

| 引用 | 核的结果 | 命令 |
|---|---|---|
| 三条新事实 = `_sweep-rel-r1-background.md:59` 一整句拆三条 | ✓ 三条内容都在该行内 | `awk 'NR==59' research/prompts/_sweep-rel-r1-background.md` |
| Step 9 全文 = `.claude/agents/sweep.md:37`（工作区版） | ✓ | `awk 'NR==37' .claude/agents/sweep.md` |
| L1：`checks-owed.md:33`（C22 前置列，00c9d4f） | ✓ | `git show 00c9d4f:.claude/kb/checks-owed.md \| sed -n '33p'` |
| L2：`checks-owed.md:24`（C13 前置列，00c9d4f，「23 条」） | ✓ | 同上 `sed -n '24p'` |
| L3：`decisions/16-发布语义.md:209` | ✓ | `git show 00c9d4f:.claude/kb/decisions/16-发布语义.md \| sed -n '209p'` |
| L4：`checks-owed.md:304`（C333 前置列） | ✓ | `sed -n '304p'` |
| L5：`decisions/08-核心索引结构.md:483` | ✓ | `git show 00c9d4f:.claude/kb/decisions/08-核心索引结构.md \| sed -n '483p'` |
| L6：`verification-build.md:155` | ✓ | `git show 00c9d4f:.claude/kb/verification-build.md \| sed -n '155p'` |
| L7：`milestone/02-second-txn.md:222` | ✓ | `git show 00c9d4f:.claude/kb/milestone/02-second-txn.md \| sed -n '222p'` |
| L9 表格行：`decisions/26-后台整理与放置回收.md:234`（「5 记账交互」行） | ✓ | `git show 00c9d4f:.claude/kb/decisions/26-后台整理与放置回收.md \| sed -n '234p'` |
| L9 警示句：同文件「第 219 行（标题正下方）」 | ✗ 实际位置：第 220 行是「### 六个未定项的论证材料」标题，第 219 行是空行，警示句本身在第 222 行（标题下方隔一空行）。引文内容（整句）与第 222 行逐字相符，只是行号数错——数成了标题前一行，应为标题后两行 | `git show 00c9d4f:.claude/kb/decisions/26-后台整理与放置回收.md \| awk 'NR==219||NR==220||NR==221||NR==222'` |
| L10：`checks-owed.md:159`（C157 检查列） | ✓ | `sed -n '159p'` |
| `-runlog.md`：s1 词数 1527、oov 生词=3（reevaluation）、退出码 0 | ✓ 逐字重现 | `wc -w`；`python3 research/scripts/oov-check.py …-output-s1.md` |
| `-runlog.md`：s2 词数 1169、oov 生词=5（passage's、contradicted）、退出码 0 | ✓ 逐字重现 | 同上，对 s2 |
| `-runlog.md`：`corruption-check.py` 两次都判绿 | ✓ | `python3 research/scripts/corruption-check.py …-output-s{1,2}.md`，两次 exit=0 |

本地腿这一段：核了 15 处引用/复跑对象，✓ 14 处，✗ 1 处（L9 警示句行号 219，实际 222）。

### 五处自陈偏离直译（9b、9c/9e、9f、9g、L9 带警示句）逐条核

- **9b**：删「只判候选表里分给你的组（……）」半句、加「视十行已从正确的结束版本读出」一句。经与原文 `.claude/agents/sweep.md:37` 比对，删的半句在这次任务里确实无对应机制（没有候选表分组）；加的那句是让本地腿不必自己跑 `git show` 的显式声明。两处都在核对表里如实标注，未发现被隐藏的丢失。
- **9c/9e**：把「在结束那一版之后还是不是真话」「以结束那一版的里程碑文件、`git log` 与代码为准」收窄成「只认三条给定新事实」。经比对原文，这是真实的语义收窄（原文允许查任何权威源，此处只许查三条事实），核对表本身承认这一点并给出理由（自足提示做不到真的查 git log）。这处收窄本身没有让某一格的答案被证明或证伪：L1、L2、L6 三行模型给出「irrelevant」的理由是「日期不同 = 不同的事实」，这条推理本身在给定的英文规则（含 9c/9e 收窄）下也站不住——9c 明写「判的是分句，不是……」「即使欠账仍开着，前置或理由过时也要判要改」，日期不同不构成默认不相关的理由。但这是模型对给定规则的误用，不是规则翻译本身缺了限定词或多了限定词，不算「翻译致其作废」，按核查员职责不评判这一层推理对错，只记录为观测供主 agent 逐条现查（尤其 L2：Fact 2「29 条」与 L2 引文「23 条」明显是同一个量的两个时点，模型两次抽样都判「irrelevant」而未判「needs-change」，这一点值得主 agent重点复核，但不计入本报告的 ✗/✓）。
- **9f**：五个判定标签各加英文释义、「检索词」概念替换成「the passage's words overlap with a new fact's topic」。核对表如实说明这是为让标签自洽做的必要展开；核查员未找到某一格的判定因为这条释义而被误导成不该有的标签（模型误判 L2 等的原因见上，与这条释义无关，是模型对「不相干」标签本身的适用条件理解有误，独立于英文措辞）。
- **9g**：原文只要求解释「为什么不受影响」，英文加「or is」使其也要求解释「为什么受影响」。这是加宽要求（多问一句），不删限定词，不影响判定标准本身，未发现致某格作废的情形。
- **L9 带警示句**：警示句整句带入且内容忠实（已核对，见上表 ✗ 那一行——内容对、行号错）。模型 s1/s2 两次都据此警示句判 L9「irrelevant」（因为「这是旧记录，不是现状」），这个判断路径与警示句的翻译是否忠实无关（忠实），不因行号引用错误而受影响——行号错误只出现在核对表自己的文档里，从未进入发给本地模型的提示原文（提示原文第 60 行直接给出警示句全文，不带行号）。

⇒ 五处自陈偏离直译中，**没有一处因翻译本身的缺失或增补而让某一格的答复作废**；L9 警示句的行号引用错误是核对表自己的文档瑕疵，不传导到模型输入，不影响 s1/s2 的任何一格。

### 附带观测（不计入判定，供主 agent 参考）

s1、s2 两次抽样对同一批行给出不同判定：L3（needs-change vs event-clause-no-change）、L6（irrelevant vs needs-change）、L8（needs-change vs irrelevant）。按「一条腿只抽一次样不算一次观测」，这属于「不稳定」，不能单独采信任何一次的判定；且 L2 两次都判「irrelevant」而未判「needs-change」，按两次都不打中的规则记「没打中」（这条论证不构成对 Step 9 判据本身的攻击证据）。

## 没做什么（本地腿这一段）

- 未评判 s1/s2 各格判定对错本身（是否该是 needs-change 还是 irrelevant），只核引文位置与内容、复跑脚本判定、译文有没有丢字或多字。
- 未重新调用本地模型（未消耗额外抽样），只对已有的两份输出做字词损坏闸复核与文本比对。
- 未核 `-translation-audit.md` 第四节「首稿改动记录」列出的五处 `replace-once.py` 定点改动的中间过程，只核了定稿内容与原文是否一致。

## 三腿汇总计数

| 腿 | 核了 | ✓ | ✗ | 核不动 / 分不清 |
|---|---|---|---|---|
| Opus 攻方 | 17 | 16 | 0 | 1（核不动） |
| Sonnet 正推 | 20 | 18 | 1 | 1（分不清） |
| 本地攻方 | 15 | 14 | 1 | 0 |
| 合计 | 52 | 48 | 2 | 2 |
