# gate-shrink-r1 云端正推（Sonnet）报告

判据格：第 1、2、5、6 问（正文「四、分工」）。

## 判定一览（先给结论，依据在各节）

| 格 | 判定 | 一句话 |
|---|---|---|
| Q1 A 的合并丢不丢覆盖 | 一致（未丢格） | 84 格全部找到去处，37 个 doc-kb 格名无重名，`--check <格名>` 机制在全部 14 道带格名数组的门禁里已支持，合并只是把数组拼接 |
| Q2 doc-kb 37 格的出路与该不该拆 | 一致（不必拆） | 合并前每个源文件本就是「多格聚合、一格红拖累整道退出码」的形态；`gate.sh` 的 `run_stage` 不吞子脚本自己的 stdout，每格自己的 ✗/→ 原样穿透；`--check` 保住单格调用点 |
| Q5 C 节定义改动是否只做了 Q8/Q9/Q15/门禁改名 | 冲突（不是） | 找到三处不在这四类里的改动，见下 |
| Q6 该在写入时拦的检查 | 冲突（有，且已被这一轮删掉一条） | `kb-scribe-followup.sh` 这一轮被整段删掉，是这一轮唯一被移出「写入即拦」的机制；另有几类仍适合写时拦但目前没有钩子 |

## 一、Q1 + Q2：完整合并与改名方案

### 1.1 十五道合并后门禁

沿用背景材料「二、方案 A」给的类与名字（`research/prompts/_gate-shrink-r1-background.md:20-35`），逐一核对格数：

| 类 | 文件名 | 合并自（旧文件） | 格数 | 是否真合并（多源→一源） |
|---|---|---|---|---|
| 文档类 | `doc-kb.sh` | 10、20、30、32、43 | 37 | 是（5→1） |
| 文档类 | `doc-experiments.sh` | 34 | 10 | 否（改名） |
| 文档类 | `doc-process-records.sh` | 11 | 7 | 否（改名） |
| 文档类 | `doc-text.sh` | 12 | 3 | 否（改名） |
| 代码类 | `code-source-discipline.sh` | 13、27、33 | 8 | 是（3→1） |
| 代码类 | `code-tooling.sh` | 47 | 8 | 否（改名） |
| 代码类 | `code-research-build.sh` | 15 | 1 | 否（改名） |
| harness 类 | `harness-tests.sh` | 14、74 | 2 | 是（2→1） |
| checker 类 | `checker-independence-and-sync.sh` | 92、94 | 2 | 是（2→1） |
| checker-tier 类 | `54-layer0-replay.sh`（不改名） | 54 | 1 | 否 |
| checker-tier 类 | `checker-tier-qemu-device-streams.sh` | 55 | 1 | 否（改名） |
| checker-tier 类 | `checker-tier-lkmm.sh` | 57 | 1 | 否（改名） |
| checker-tier 类 | `checker-tier-mutation-replay.sh` | 59 | 1 | 否（改名） |
| checker-tier 类 | `checker-tier-replay.sh` | 87 | 1 | 否（改名） |
| checker-tier 类 | `checker-tier-test-environment.sh` | 77 | 1 | 否（改名） |

合计 84 格（`awk -F'\t' 'NR>1{print $1}' research/prompts/gate-shrink-r1-inventory/cells.tsv | wc -l` → 84），与今天 `.claude/gate.d/*.sh` 的格数总和一致（`.claude/gate.d/10-references-and-invariants.sh:4` 等各文件头「gate-stage」括号内列出的格名数，逐份加总同为 84，方法见 `research/prompts/gate-shrink-r1-inventory/report.md:12-24`）。

**54 号例外的依据**：`.claude/rules/implementation-workflow.md:75`「54 号 `--full` 在本地配置（仓根 `multi-host.env`...）」一段与 `research/scripts/admission.py`、`research/scripts/layer0-shard-run.sh` 按文件名读取的事实（引自附录二`records/2026-09-28-门禁59号提速与双机分片.md:79-90` 一节的转述：「54 号例外：它的文件名写在 `research/scripts/admission.py`（8 处）与 `layer0-shard-run.sh`（1 处）里...所以与 KV 接入同一批改」）——本轮不动 54 号文件名，其余 checker-tier 五道（55/57/59/77/87）今天已判定「改名只让各自按文件名存的全绿标记作废一次」，可以随本轮改。

### 1.2 逐格映射（Q1：丢不丢覆盖）

**不改判据的部分（9 道，含 6 道 checker-tier）**：`doc-experiments.sh`（原 34 号 10 格）、`doc-process-records.sh`（原 11 号 7 格）、`doc-text.sh`（原 12 号 3 格）、`code-tooling.sh`（原 47 号 8 格）、`code-research-build.sh`（原 15 号 1 格）与 6 道 checker-tier，判据函数体一个字不改，只改文件名与补 `# gate-category:`。这一类天然不丢格：源与目标一一对应，`--check <格名>` 的格名字面不变。

**真合并的 4 道，逐格核对无重名、无消失**（`awk -F'\t' 'NR>1{print $1"\t"$2}' research/prompts/gate-shrink-r1-inventory/cells.tsv` 现查得出下表）：

`doc-kb.sh`（37 格，10+20+30+32+43）：
- 原 10 号（7）：experiment-refs、decision-refs、declared-counts、governance-refs、invariant-count-elsewhere、citations、invariant-anchors
- 原 20 号（13）：kb-shape、item-ref-status、status-redundancy、cross-decision-status、settled-item-self-open、settled-ref-says-open、stale-open-items、settled-same-file、freeze-layer-membership、decision-items-sync、blocking-verdict、user-verdict-owed、decision-summary-width
- 原 30 号（3）：entry-added、shape、status-sync
- 原 32 号（9）：field-refs、field-projection、field-table-sums、first-txn-hooks、admission-terms、segment-registry、format-const-placeholders、second-txn-hooks、tree-table-reserve
- 原 43 号（5）：table-shape、closeout-collects-open、paid-cited-tests、audit-contradictions、row27-preconditions

37 个格名两两不同（无重名，`sort | uniq -d` 现查零命中），`--check <格名>` 合并后仍然唯一可达。

`code-source-discipline.sh`（8 格，13+27+33）：vague-names、test-file-names（原 13）；format-constants、clause-enums、feature-bits（原 27）；mutation-tables、absolute-assertions、reading-discipline（原 33）。无重名。

`harness-tests.sh`（2 格，14+74）：one-scenario（原 14，已是显式格名）；74 号今天是「整道」（无 `CELL_NAMES` 数组，见 `research/prompts/gate-shrink-r1-inventory/cells.tsv` 该行第 2 列写「整道」），合并时要给它起一个格名——建议 `model-differential`（与它的 `# gate-stage:` 描述一致，`.claude/gate.d/74-model-differential.sh:4`），不与 one-scenario 冲突。

`checker-independence-and-sync.sh`（2 格，92+94）：92、94 号今天都是「整道」，合并时各起格名 `layout-checker-sync`、`checker-implementation-disjoint`（同样取自各自的 `# gate-stage:` 首句），不冲突。

**结论**：84 格里没有一格在合并方案里找不到去处；没有一处因为合并产生「两格同名、后面盖掉前面」的风险（两处「整道」升格为显式格名时才第一次需要人工取名，取名不撞已用名）。**唯一实质性变化**是把 74、92、94 三道从「无格名、单格即整道」升格为「有格名的多格聚合体」，这本身不改变判据，只是让 `--check` 从「无意义（只有一格）」变成「有意义（合并后同伴变多）」。

### 1.3 每道合成后门禁的出路、`--check`、`# gate-category:`、跑的次序（Q2）

**出路（howto/→）不受合并影响**：`gate.sh` 的 `run_stage()` 只做 `head1`（打阶段名）→ 调子脚本 → `record_stage`（登记退出码），子脚本自己写到 stdout 的每一行 `✗` 与紧跟的 `→` 原样穿透，`gate.sh` 不截断、不吞（`.claude/singlefs-ai-sop/scripts/gate.sh:283-291`）。这条在合并前就是这样：`20-doc-decision-documents.sh` 今天已经是 13 格聚合、一格红就让整道退出码非 0，`gate.sh` 汇总照样打出这 13 格各自的 ✗/→（cells.tsv 里 20 号各格的判定本身就是从这份聚合脚本原样跑出来的）。**合并 5→1 只是把这种「聚合、退出码取或」的形态从 5 份文件收成 1 份，形态本身在第一轮合并时就已存在，这一轮不新增这个问题。**

**`--check <格名>` 够用**：现有全部 14 道带 `CELLS=`/`CELL_NAMES=`/`CELL_ORDER=`/`GRID_NAMES=` 数组的门禁文件头都写了「`--check <格名>` 只跑那一格」（`grep -n -- '--check' .claude/gate.d/*.sh` 逐份核过，见共用约束「陈述外部状态之前现查」）；合并只需把几个数组 `CELLS=(...)` 顺次拼接，`case "$argument" in --check) ...; esac` 那段外壳照抄一次即可，不用新写判据。子 agent 已经在用单格调用（`three-way-materials.md` 的 `--check implementation-premise`、`prior-art.md` 的 `--check citations`），合并后这两处调用点不变（implementation-premise 住在 doc-process-records.sh，citations 住在 doc-kb.sh），只是文件名要跟着改。

**`# gate-category:` 与去编号文件名**：13 道今天没写 `# gate-category:`（10、11、15、30、43、55、57、59、74、77、87、92、94，见 `research/prompts/gate-shrink-r1-inventory/report.md:50`「没写 `# gate-category:` 的 13 道」），合并收尾时逐道补一行，值取上表「类」列的五选一。

**该不该拆回来**：不该。37 格合一件事只改变「一屏汇总里看到几行 FAIL」，不改变「红了怎么修」——子 agent 与 gate-triage 修红时本来就先看点名的文件与行在不在这一轮改动里（`.claude/agent-common.md:89`「红了先看它点名的文件与行在不在这一轮的改动里」），从来不是靠汇总行数分辨归属。真正的成本是「一屏看不出红的是决策文档那一半还是欠账表那一半」，这个信息本来就住在 `✗` 行自己的文件名与格名里，不住在汇总行里。

## 二、Q5：逐份判附录二里这一轮改的定义与规则

十五个「全部改动都属于这一轮」的文件里，十二个只做了 Q8/Q9/Q15/门禁编号改名/记时区规矩，逐处核对与背景材料附录二给出的分类一致（`.claude/agent-common.md`、`.claude/agents/experiment-runner.md`、`tooling-writer.md`、`three-way-materials.md`、`prior-art.md`、`crash-verifier.md`、`implementation-writer.md`、`investigator.md`、`kb-spec-drafter.md` 里的 changelog-format-r1 部分之外的那一行、`.claude/rules/implementation-workflow.md`（除下述一段）、`implementation-first.md`、`mutation-sampling.md`、`path-moves.md`）。**但另外三处不在这四类里**，附录二的归属表没有列出：

### 5.1 `.claude/rules/three-way-inference.md` + `.claude/main-agent.md`：撤销「核查员按轮派」

`three-way-inference.md` 把「核查员按轮派。这一轮有腿交了模型、产物或复跑命令，就派 `three-way-verifier`」整句删掉，换成「核查与判决由主 agent 做，不派子 agent 去核子 agent」并新增五步核对流程；`main-agent.md` 的分工表同步把「→ 派 `three-way-verifier`（有腿交了模型...）→ 主 agent 写判决」改成「→ 主 agent 自己核、自己写判决...；不派核查员」。

这是撤销一个 agent 角色（`three-way-verifier`），不是门禁编号改名，不是 Q8（doc-lint 挪到提交时），不是 Q9（扫仓门禁只在提交前跑），也不是 Q15（看门狗放行样本自检）或记时规矩。附录二自己写明「派发提示未点名 `three-way-verifier.md` 的删除...本材料不审这几处」（`research/prompts/_gate-shrink-r1-diff.md:7`），但只排除了对 `three-way-verifier.md` 这份文件本身的审查，没有把 `three-way-inference.md` 与 `main-agent.md` 里同一件事的另一半（撤销引用它的流程步骤）一并排除——而这两处恰恰在「全部改动都属于这一轮」的十五个文件名单里。**这是一个第五类改动，附录二的归属表没有覆盖它**，应当在这一轮的判决里单独列一句：撤销核查员角色是不是这一轮定案的一部分，需要主 agent 确认；如果不是，这两处改动要单独走三方或退回上一次 commit 的版本。

### 5.2 `.claude/rules/verification.md`：耗时用例的量法改写，牵连 `research/scripts/harness-test-timing.py` 的删除

原文「耗时用例：**单线程** debug 下单条跑到 60 秒及以上的用例...轻重由量出来的数定，不由整份全量日志里 libtest 的『has been running for over 60 seconds』定：`python3 research/scripts/harness-test-timing.py measure`...」改成「耗时用例：debug 下单条跑到 60 秒及以上的用例...单条用时看平常跑的时候量到的，不另外单独量」，同时删掉「轻重由量出来的数定」一整句与 `harness-test-timing.py` 的调用法。现查 `git status --porcelain -- research/scripts/harness-test-timing.py` 得 `D`（整份 275 行删除，暂存区），确认这是同一件事的两半：**改判据的量法本身，不是重命名或搬门禁号**。这处改动与 Q8/Q9/Q15/门禁改名都对不上；它更像是 HEAD 那次提交「harness 耗时用例改名与只重跑失败的用例」的后续收尾，但既然没有被附录二列进四类理由里，同样该单独确认。

### 5.3 `.claude/rules/implementation-workflow.md`：59 号新增双机变异分片的整段机制

「双机分片」一节除了把 `layer0-shard.env` 改名 `multi-host.env`（改名部分符合门禁改名类），还整段新增了 59 号的双机分片流程：新脚本 `research/scripts/mutation-shard-run.sh`（`git status --porcelain` 显示 `A`，新文件）、新环境变量 `GATE_MUTATION_ACROSS_MACHINES`、`GATE_MUTATION_START_OVER`、跨机拷回与 `crates-mutation-rows.py import` 核指纹的整套逻辑，以及「门禁管哪一半」段落新增的判红判据（工作线程数逐片核对）。这与背景材料自己「一、现状」一节的陈述直接对不上：背景材料「一、现状」一节（`research/prompts/_gate-shrink-r1-background.md:16`）写道「54、59 号与双机分片...这一轮对它们只改名、改开关，不加功能」，而 `implementation-workflow.md` 这一段实打实新增了 59 号的双机分片功能（新脚本、新变量、新判据），不是改名或改开关。**

这三处的共同点：都落在「全部改动都属于这一轮」的文件名单里，但都不在附录二给出的四类理由（Q8/Q9/Q15/门禁改名/记时规矩）之内，其中第三处还与背景材料本身的另一句陈述矛盾。**推翻这条判定的现象**：若主 agent 能在这一轮的派发历史或另一份记录里找到这三处改动各自的授权（例如「这一轮同时定案撤销核查员角色」的用户原话，或「59 号双机分片就是这一轮做的」的实现员报告），则这三处应改记为「这一轮的第五类改动」而不是「混入的越权改动」；本报告没有读那份授权记录（不在给定背景材料与我的读取范围内），所以只能报「对不上」，不能报「一定是错的」。

### 5.4 「有没有改出互相矛盾的两句」

逐份核对未发现硬矛盾：`three-way-materials.md` 步骤 0 的「开工先跑 `--check implementation-premise`」发生在工作**开始之前**（前提检查），不是「交回前跑门禁」，与 Q9 不冲突；`kb-scribe.md` 第 4 步明写「第 2、3 步里的 `--check shape` 与 `--write` 是写回流程的一步，照跑」，把这两次调用显式排除在「门禁阶段交回前不跑」之外，是一句自洽的例外声明，不是矛盾。**唯一的语义漂移（不算硬矛盾）**：`agent-common.md`「门禁」一节把 `stage-owners.tsv` 第二列的角色从「哪个门禁阶段该由谁在干完自己的活之后先跑」改写成「提交前整轮门禁判这一道红时先派给谁修」，但同一节里紧邻的上一句已明写「`crash-verifier` 在提交时跑登记给它的那几道，照它的定义」——这个例外写在前面，所以 `crash-verifier.md` 仍把该表当「我该主动跑哪几道」的清单来读，两句连起来读不矛盾；只是分开摘录容易读成矛盾，建议在门禁一节里把 crash-verifier 的例外紧跟着第二句一起重复一次（写作代价小，可读性收益大，不是这一轮必须做的事）。

**这一条本身可推翻的现象**：若 `crash-verifier.md` 从 `stage-owners.tsv` 取到的文件名与 `.claude/gate.d/` 今天的实际文件名对不上（即该表把 54/55/57/59/74/77/94 的行也改成了旧编号或旧文件名），则「还说得通」这一判定要改判——现查 `stage-owners.tsv` 里这几行的第一列（`54-layer0-replay.sh`、`55-qemu-device-streams.sh`、`57-lkmm.sh`、`59-crates-mutation-replay.sh`、`74-model-differential.sh`、`77-test-environment.sh`、`94-checker-implementation-disjoint.sh`）与 `ls .claude/gate.d/*.sh` 逐字比对，七行全部与仓里现存文件名相同（该表其余大多数行——`10-kb-rot.sh`、`11-batch-scope.sh`、`16-freeze-layer-membership.sh` 等——指向的是round 1 合并之前的旧文件名，在仓里已不存在，但这些行不归 `crash-verifier` 使用，归 `kb-scribe`、`experiment-runner` 等其他 agent；这批陈旧登记已经被 `47-code-tooling-selftests-and-registries.sh` 的 stage-owners 格判红，见该格今天的判定「表还按旧文件名，归『登记收尾』一件」，是已知欠账，不是这一轮新introduced 的问题）。

## 三、Q6：「只在提交时跑」这条规矩的代价——该在写入那一刻拦的清单

### 3.1 这一轮已经删掉的一条写时拦截机制

`kb-scribe.md` 原文（改动前）：「每次写入之后，写入后钩子 `.claude/hooks/kb-scribe-followup.sh` 按 `.claude/hooks/kb-scribe-followups.tsv` 跑几道阶段、把红的 ✗ 与 → 交回给你...」（`research/prompts/_gate-shrink-r1-diff.md:213`），这一轮整段删掉。`git status --porcelain -- .claude/hooks/kb-scribe-followup.sh .claude/hooks/kb-scribe-followups.tsv` 现查得两个 `D`（均为暂存区删除），确认这两份文件本身也在这一轮被删。**这条机制原本做的正是「书记员写完 kb 格式错误立刻拦，不等提交」**：它按流程分别在 21 号（分项状态）`--write` 之前、30 号（变更史）条目写之前、49 号 `--write` 之前、75 号（实验页那几行）写之前插入检查。删掉之后，这类格式错误（分项索引与正文不符、变更史条目形状不对、实验页判决行没被点名）从「书记员写完当场看到 ✗」退化为「提交前 `gate-triage` 跑整轮门禁才看到 ✗，再派回书记员修」。

这条代价是真实的、可核的：`kb-scribe.md` 第 4 步改动前后的对照已在 `research/prompts/_gate-shrink-r1-diff.md:213-214`（原文）与现今文件第 4 步（`第 2、3 步里的 --check shape 与 --write 是写回流程的一步，照跑`）里逐字可比——这一轮**保留**了 30 号 `--write`（变更史现状行）与 20 号 `decision-items-sync --write`（分项状态）两个仍在写入当口调用的检查，**删掉**的是包住它们、覆盖面更广的那层通用钩子（原来还管 49 号、75 号那两类）。

### 3.2 建议留在写入那一刻拦的，按类型列出

| 候选 | 现在归哪一格 | 理由：为什么适合写时拦，不适合只等提交 |
|---|---|---|
| 决策分项索引与正文状态不符（原 21 号，今属 `doc-kb.sh` 的 `decision-items-sync`） | 已经保留（kb-scribe 第 3 步显式跑 `--write`） | 不需要新增；这一格本来就没被这一轮移出写时检查 |
| 变更史条目形状（原 30 号 `shape`） | 已经保留（kb-scribe 第 2 步显式跑 `--check shape`） | 同上 |
| **实验页判决行的 `false` 字段没被点名**（`verdict-false-named`，今属 `doc-experiments.sh`） | 只在提交时整轮门禁判 | 这一格判的是「执行员写实验页时漏抄了一处判决行」，错误只在执行员自己的这次写入里产生，第三方（其它会话）不会中途改动它；等到提交才发现，要么退回给已经交回的执行员重开一轮，要么主 agent 现改，两种都比写完当场核一次贵。**这类「错误只可能来自这次写入本身、且核对成本是几秒钟量级」的格，是最适合恢复写时拦截的一类** |
| **实验索引行与正文标题不同步**（`index-sync`，今属 `doc-experiments.sh`） | 只在提交时整轮门禁判 | 同上理由：书记员/执行员自己写歪的，写完立刻能自证 |
| **决策与实验双向登记**（`decision-links`，今属 `doc-experiments.sh`，原 75 号） | 已从 `kb-scribe.md` 第 3b 步的写时检查移出（这一轮删掉「写完跑 75 号：它报『不对称』…」那句，`research/prompts/_gate-shrink-r1-diff.md:21`） | 这一格判的是「决策分项与实验页互相点名对不上」，两侧可能分属不同的执行员/书记员，写时检查曾经能在同一次写入里立刻发现「规格只给了一边」；删掉之后要等提交才发现，且修复往往要回头问主 agent「规格是不是漏了一行」——这条更值得恢复，因为它原本就是这一轮唯一一处点名会报错的写时检查（原文写「写完跑 75 号：它报『不对称』时看点名的那一对在不在这一份规格里」），删除的代价比其余几格更具体 |
| kb 用词/自称/位置指代等文本纪律（`kb-shape` 等） | 提交时整轮门禁判；`write-guard.sh` 已经覆盖了角标（12 号 `prime-marks`）与钟点（`clock-times`）两种写法 | 不建议扩大 `write-guard.sh` 的覆盖面：这几类判据要读一整份 kb 文件的上下文（例如「同一决策的另一条已定分项是否也写着『未定』」），不是单次 Edit 的 `new_string` 局部模式匹配能判的，写时钩子只擅长判「这一次要写进去的字符串本身」，判不了「写完之后与全文件的关系」——这也是 `write-guard.sh` 今天只覆盖角标与钟点两种（都是纯字符串模式）而不覆盖 kb-shape 类判据的原因，不建议改 |

**这条结论可推翻的现象**：若主 agent 核实「合并之后 `verdict-false-named`、`index-sync`、`decision-links` 三格提交时才发现的实际频率极低」（例如查 `records/` 与 `checks-owed.md` 里这三格过去半年从未在真实语料上抓到过一次真缺陷），则这条代价可以判定为「理论存在、实测不值得再建一层写时钩子」，我的建议就该改成「维持现状，不恢复」。cells.tsv 里能查到的历史记录（`research/prompts/gate-shrink-r1-inventory/cells.tsv` 对应行的「抓到过没有」列）没有覆盖这三格，本报告没有另外核实，交主 agent 现查。

## 没做什么

- 没有判 Q3（B1–B9 逐条）、Q4（B6 次序）——按分工表这两问归云端攻方（Opus），本报告不重复判。
- 没有核实 `.claude/rules/three-way-inference.md` 撤销核查员一事是否另有授权记录（5.1 节写明推翻条件，未去找那份授权）。
- 没有对 `harness-test-timing.py` 删除本身（5.2 节）做正确性判断，只核实了它是这一轮的一部分、且不属于 Q8/Q9/Q15/门禁改名四类。
- 没有编译、没有跑门禁阶段（`--check` 单格核对全部靠现读文件头与 `grep`，未实跑，因资源限制「跑门禁只跑非重型的格」而这几处判定不需要实跑就能核实）；没有跑 `gate.sh` 整轮或 `gate-staged.sh`。
- 没有判「登记收尾」（去编号、改全仓引用）具体怎么做——那是实施阶段（工具实现员/书记员）的活，本报告只给出 15 个最终文件名与格名，不写重命名的执行脚本。
- 本地攻方判第 1 问的逐格映射，与本报告 1.2 节各自独立完成，未互相核对（分工要求互不重复）。
