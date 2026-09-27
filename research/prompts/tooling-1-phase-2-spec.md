# 第二段规格：门禁批三方第二轮判完之后才动的文件（tooling-1 起草，2026-09-27 JST）

这些文件在 `research/prompts/defs-gatebatch-m2-r2-snapshot/sha256sums.txt` 的开工快照里，第一段一个字没改。每条写：文件、旧串（原文整行或能 `grep -cF` 恰好命中 1 次的片段）、新串、为什么（指到分析报告的节号）。旧串以动手那一刻的文件为准，先 `grep -cF` 核一次。

## 一、`.claude/gate.d/stage-owners.tsv`（序 1 改归属、序 9 新阶段登记）

改四行（第二列整格替换，第三列理由整格替换），加一行：

| 行（第一列） | 旧第二列 | 新第二列 | 新第三列 |
|---|---|---|---|
| `47-research-script-selftests.sh` | `three-way-materials` | `three-way-materials,tooling-writer` | 材料员用的抽取脚本与本地腿脚本的自证；研究脚本由 tooling-writer 改，改完先跑 |
| `62-stage-owners.sh` | `gate-triage` | `gate-triage,tooling-writer` | 本表与门禁目录、agent 定义；tooling-writer 加阶段、改定义之后先跑 |
| `63-agent-write-scope.sh` | `gate-triage` | `gate-triage,tooling-writer` | 看项目 settings 里的钩子注册、钩子自证、写范围表与定义；tooling-writer 改钩子与定义之后先跑 |
| `73-research-gate-lint.sh` | `gate-triage` | `gate-triage,tooling-writer` | 研究脚本、hook 与 .claude/scripts 的拒绝带出路与 shell 纪律；tooling-writer 改完先跑 |
| 新行 `41-crash-case-registered.sh` | — | `implementation-writer,gate-triage` | 崩溃枚举用例要标 #[ignore] 并登记成 crash-case：实现员新写用例之后先跑 |

新行与 `41-crash-case-registered.sh` 文件同一次落（62 号①要目录与表一一对应，只落一样就红）。

## 二、新门禁阶段 `.claude/gate.d/41-crash-case-registered.sh`（序 9 的门禁那一半）

- 判据全在 `research/scripts/crash-case-check.py`（第一段已立，自证挂 47 号）：阶段只调 `python3 "$ROOT/research/scripts/crash-case-check.py" "$ROOT"`，原样转它的输出与退出码（2 → 1）。
- 文件头：`# gate-stage: 直接调全量崩溃枚举的测试函数标了 #[ignore] 并登记成 crash-case`；`# gate-similar: 54-layer0-replay.sh 跑登记了的 crash-case 用例、核全绿标记；这里只判源码里的用例有没有标 ignore、有没有登记，不跑任何测试`；`# gate-similar: 80-absolute-assertions.sh 也扫 crates/*/tests/ 的源码，但判的是断言形态`（先跑 `python3 .claude/singlefs-ai-sop/scripts/gate-overlap.py --list` 复核还有没有更像的）。
- 样本 `.claude/gate.d/fixtures/41-crash-case-registered.sh/{red,green}/`：green 放一个标了 `#[ignore]`、登记了的用例与一个 `quick_tier` 快档；red 放一个没标没登记的，`expect` 要 `没标 #[ignore]` 与 `没有登记成 crash-case` 两句。
- ⚠️ 仓里现查红一处（2026-09-27 JST 00:5x 跑 `python3 research/scripts/crash-case-check.py` 的原样行）：
  `✗ crates/singlefs-harness/tests/crash_enumeration_resumes_from_its_progress_file.rs:383 a_run_that_goes_red_leaves_no_progress_file_behind：直接调全量崩溃枚举，却没标 #[ignore]` 与同一处「没有登记成 crash-case」。
  接阶段之前先派实现员处理那一条：几步的合成流就在测试函数上面加 `// crash-case-check:not-a-crash-case <理由>`；真是全量就标 `#[ignore]` 并在 `stage-inputs.tsv` 登记 `crash-case:` 行（后者动 `stage-inputs.tsv`，同属第二段）。

## 三、`.claude/hooks/heavy-test-guard.sh` 与 `.claude/hooks/lib_heavy_tests.py`（报告 4.5 两行）

1. `heavy-test-guard.sh`：`cargo test` 点名的测试目标里有没标 `#[ignore]` 的崩溃枚举用例（判法复用 `research/scripts/crash-case-check.py` 的 `crash_enumeration_tests()`，按路径导入，不抄第二份），而命令不带 `--ignored` / `--include-ignored`：写一条检出（只记不拦，交看门狗报）。自证加一对样本。
2. 重型放行表只留 `lib_heavy_tests.py` 一处（`records/2026-09-16-subagent拆分提案.md` 第四十节第 19 行）：`heavy-test-guard.sh` 里另写的放行名单删掉，改成调模块。
3. 55 / 57 / 59 号的静态分支（`--selftest`、只核标记的那一支）与 `herd7 -version` 放行（第四十节第 40 行）：判法先走一轮定义三方定，定了再改 `lib_heavy_tests.py` 的 `classify`。
4. `heavy-test-guard.sh` 文件头第 6 行 `# gate-similar: runner-dispatch-guard.sh …（只在「跑 / 复跑 / 执行 / 运行 / bash / 起」的宾语是重型阶段时才拒，否定、转述、引号里的都不判）…` 已经不对：派发闸第一段改成只查一行「重型测试：不跑」。改成：`# gate-similar: runner-dispatch-guard.sh 管同一条规矩（子 agent 不跑重型测试），但挂 PreToolUse[Agent|Task]、在派发那一刻只查提示里有没有一行「重型测试：不跑」；这里挂 PreToolUse[Bash]、在执行那一刻判真要跑的命令与它执行的脚本`。

## 四、`.claude/main-agent.md`（报告 4.6 八条，序 6、15、20 的那一句；动完走定义三方，门禁 72 号）

逐条：旧串（`grep -cF` 恰好 1），新串。

1. 「一轮怎么开、怎么收」第 4 条——旧：`4. **派实现员之前先列出它要动的 `crates/` 文件**，与在跑的实现员要动的文件有交集，就排在那一个后面，或并给同一个实现员做，不并行改同一批文件。`
   新：`4. **派实现员之前先列出它要动的 `crates/` 文件**，写进派发提示单独一行「要动的 crates 文件：…」；一件活不超过 3 件、8 个文件（推的，没量过）。与在跑的实现员要动的文件有交集，就排在那一个后面，或并给同一个实现员做，不并行改同一批文件；派发闸按这一行登记、求交集，撞了就拒。`
2. 「派出去之后」看门狗那一句——旧：`每次派发（包括续做）之后，立刻用 Bash 的 `run_in_background` 起看门狗 `bash research/scripts/watch.sh <这次派出的 agent id，逗号分隔>`（只填 agent 号：会话目录它自己找，阈值在 `research/scripts/watch.conf` 里配，不在命令行上手敲；调用里不再加 `&`、`nohup`、`disown`）`
   新：`一个会话一个看门狗：用 Bash 的 `run_in_background` 起 `bash research/scripts/watch.sh`（不带 agent 号：它从 CLAUDE_CODE_SESSION_ID 找会话目录，每次检查自己找还没结束的子 agent，新派的自动接上；阈值在 `research/scripts/watch.conf` 里配；调用里不加 `&`、`nohup`、`disown`）。已有一个在盯时再起会当场退 5，不用停旧的起新的；它退出报警之后再起一次。确认只是慢的告警写 `bash research/scripts/watch.sh --ack <agent 号>:<告警名>`，确认记进会话的状态目录、下一次起读回（条件类的条件消失就作废，hook 检出这类事件到那个子 agent 结束才作废）`
   同一段里「上下文过大」相关的说法（若有）改成：看门狗只对 implementation-writer / experiment-runner 在 70 万报「该交接」，别的类型不报。
3. 同一节——旧：`临时派不读共用约束的 agent（general-purpose 这类）干带长等待的活时，派发提示里写上共用约束「长活可以等」那一条里后台命令怎么写。`
   新：`general-purpose 这类不读共用约束的 agent，只在「什么时候派哪个 agent」那张表里没有对应行时派；派的时候提示里写一行「开工先读 `.claude/agent-common.md`」（派发闸查），干带长等待的活再写上那份「长活可以等」那一条里后台命令怎么写。`
4. 「派发提示怎么写」一节末尾加一段：`每份派发提示写一行「重型测试：不跑」（crash-verifier、gate-triage 除外；派发闸查）。定义 frontmatter 的 `required-inputs:` 列着它要的输入，每一样照写出名字；派发闸逐组查，缺一组就拒。规格起草一次不超过 8 条（推的，没量过）。在跑的 opus 子 agent 满 8 个（用户 2026-09-27 定）、或某一族撞了限额还没到 resets 时刻，派发闸拒派同一族：等、换模型，或交用户定之后在提示里写放行行（「opus 并发已判：…」「限额窗口已判：…」）。`
5. 「代码轮派腿之前记一份开工快照」旁（`.claude/rules/implementation-workflow.md` 那一节被引用处，或本文件「派出去之后」一节末）加：`冻结副本（腿读的开工快照副本）建好就 `chmod -R a-w`；腿要改先拷进自己的草稿目录。`
6. 「交回怎么读」——旧：`交回按子 agent 的 SubagentHandback 消息判。**交回之后不再给它发消息**（续做闸 `.claude/hooks/continuation-guard.sh` 拒绝）`
   新：`交回按子 agent 的 SubagentHandback 消息判。**交回之后不再给它发消息**（续做闸 `.claude/hooks/continuation-guard.sh` 拒绝：它自己的会话记录，或主会话记录里的交回消息，哪一边先到都算）`
7. 「什么时候派哪个 agent」表加四行（插在「要别家文件系统的事实」那一行之前）：
   `| 改门禁、钩子、研究脚本、看门狗，或照判决改 agent 定义、共用约束与规则 | `tooling-writer`（派发提示给关的是哪一条、要改的文件、出口、判红样本）→ 改了定义的走定义三方（门禁 72 号） |`
   `| 定案之后写回 kb 之前 | `kb-spec-drafter` 起草规格（一次不超过 8 条），`research/scripts/kb-spec-check.py` 绿了再派 `kb-scribe`（派发闸对书记员的规格跑同一个脚本） |`
   `| 测试或门禁结果与预期不符，要查真假与机理 | `investigator`（给现象原文与要回答的问题；只查不修） |`
   `| 盘点一张表的现状（收口表、欠账表这类，逐行判一个问题） | `sweep` 第五种活 |`
   「跑变异表」那一行加一句：`广谱变异（没有表的几千条扫描）是实验，走 experiment-designer 登记，不派 mutation-triage。`
8. 同一张表「改 `crates/`：条款已定…」那一行加：`实现员交补丁的，用 `python3 research/scripts/apply-writer-patch.py <补丁目录>` 打（按名字合并变异表、跑 33 号）；它打出「自上次崩溃验证全绿以来已打 N 个补丁」的提示时，弹窗问用户要不要用 `SINGLEFS_HEAVY_TESTS=user-request` 派 crash-verifier 跑一次（只跑输入变了的崩溃枚举用例）。主 agent 自己改 `crates/` 限于「探索性、要用户边看边拍板」那一行。`
9. 「一轮怎么开、怎么收」加一条（接在第 9 条后）：`10. 用户问进度或还差多少：贴 `python3 research/scripts/closeout-status.py` 的输出（按收尾调度表的批次与状态现算），不现翻调度表。`
10. 同一节加：`11. 自己写的判决里的引文同样跑 `python3 research/scripts/cite-check.py <判决>`；每条「采纳 / 定案」写明被哪几轮打中、几轮站住，少于三轮多数的标「零轮」或「一轮」（做成 56 / 72 号的一项形式检查是推的，还没写）。`

## 五、`.claude/agents/gate-triage.md`（报告 3.6 那一行、序 4 的 required-inputs）

1. frontmatter——旧：`omitClaudeMd: true\n---`（在 gate-triage.md 里恰好 1 次）；新：`omitClaudeMd: true\nrequired-inputs: 暂存, 草稿目录, 报告\n---`。
2. 「产出」——旧：`- 一张表：阶段 / 绿·红·77 / 归属（这一轮 · 不是这一轮 · 环境 · 并发 · 分不清）/ 门禁原样的 ✗ 与 → ；然后未实现清单原样。`
   新：`- 一张表：阶段 / 绿·红·77 / 归属（这一轮 · 不是这一轮 · 环境 · 并发 · 分不清）/ 谁来修（tooling-writer · kb-scribe · implementation-writer · experiment-runner（只修锚点）· 主 agent，按红阶段点名的文件路径归：.claude/gate.d/、.claude/hooks/、research/scripts/ 归 tooling-writer，.claude/kb/ 归 kb-scribe，crates/ 归 implementation-writer，变异表锚点归 experiment-runner，其余归主 agent）/ 门禁原样的 ✗ 与 → ；然后未实现清单原样。全文写进报告文件，交回只写结论、报告路径与 sha256sum。`

## 六、`.claude/agents/crash-verifier.md`（序 4 的 required-inputs）

- frontmatter——旧：`omitClaudeMd: true\n---`；新：`omitClaudeMd: true\nrequired-inputs: SINGLEFS_HEAVY_TESTS, 内存上限|上限, 草稿目录, 报告\n---`。（派发闸对 crash-verifier 不查「重型测试：不跑」一行，只查这几组。）

## 七、`.claude/rules/implementation-workflow.md`（派发闸判法变了，第 70 行那一句跟着改）

- 旧：`派发提示要子 agent 跑重型测试的，由 `.claude/hooks/runner-dispatch-guard.sh` 拒绝。`
  新：`派 crash-verifier、gate-triage 之外的类型，派发提示里没有一行「重型测试：不跑」的，由 `.claude/hooks/runner-dispatch-guard.sh` 拒绝（提示正文里提到重型阶段的句子不再逐句判，执行时由上面这道闸与看门狗的进程一层兜底）。`

## 八、`.claude/gate.d/stage-inputs.tsv`、`research/scripts/admission.py`、`.claude/gate.d/54-layer0-replay.sh`、`research/scripts/check-segment-registry.py`

- 第一段的 22 条里没有要改这四份的；只有第二节那条红（`crash_enumeration_resumes_from_its_progress_file.rs:383`）若判成真的全量用例，要在 `stage-inputs.tsv` 加它的 `crash-case:` 行（与实现员给 `#[ignore]` 同一次落）。

## 九、第二段做完之后跑什么

`bash .claude/gate.d/62-stage-owners.sh`、`63`、`72`（改了定义与规则要判决点名）、`bash .claude/gate.d/41-crash-case-registered.sh` 与它的 red / green 样本、doc-lint、规则纪律项目本地那一道、`python3 .claude/singlefs-ai-sop/scripts/gate-overlap.py`，贴末行与退出码。
