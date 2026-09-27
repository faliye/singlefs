# verification-split-r1 云端正推（Sonnet）报告

判据格：正文第三节 Q3、Q5、Q7；分工见正文第四节「云端正推（Sonnet，`three-way-forward`）」一行。

## 引用清单核对说明（现查发现的偏差，先于三格判据）

`.claude/singlefs-ai-sop/scripts/gate.sh` 不在快照树里（被 git 忽略），派发提示写明「它这一轮没人改」。
现查：开工时该文件为 797 行；本报告写作过程中再次 `sha256sum` 发现它已变成 845 行——
`git status` 同时显示同一批 `.claude/gate.d/*.sh`、多份 kb 文件在工作区被改动，说明另一个会话正在这个仓里并行改动，
派发提示那句「它这一轮没人改」在我写报告的这段时间内**不成立**（现查，非转述）。
为了让本报告的引用可复核，我把 2026-09-27T11:33:00Z 那一刻的 `gate.sh` 拷成只读副本：
`/tmp/claude-1000/three-way-forward/verification-split-r1/gate.sh.pinned`（`sha256sum` `1a3140c0de61812336b103a1666ce93196f0f0742a7dd4e6fdde7517c2a42bde`），
下面凡引 `gate.sh` 的行号，都是这份 pinned 副本里现 `grep -n` 出来的行号，不是快照、也不是此刻工作区的行号。
`.claude/gate.d/54-layer0-replay.sh` 与其余 Q5 引用的 agent 定义、`.claude/rules/implementation-workflow.md`、
`.claude/kb/decisions/13-验证路线.md` 在写作这段时间内 `sha256sum` 与快照一致（逐个现查过，见下），行号按现在的工作区文件给。

**推翻条件**：若有人在这份 pinned 副本之后又改了 `gate.sh` 里 `drain_not_run_side_channel`、
`COVERED_BY` 或 `stage_counts_as_coverage` 那几段的行为，本节与 Q3 的判定要重新核一遍原文，不能沿用这份报告的行号。

## Q3：54 号快档「标记不作数报本次未跑、不判红」

### 3.1 `gate.sh` 收到 `GATE_NOT_RUN_FILE` 之后，`# gate-covers: 崩溃点重放` 这一轮算不算覆盖

**判绿的观测**（都在 `gate.sh.pinned` 里，现 grep 过）：

- `drain_not_run_side_channel() { # drain_not_run_side_channel <阶段名>：把这个阶段报的「本次未跑」收进汇总，清空侧信道`（`/tmp/claude-1000/three-way-forward/verification-split-r1/gate.sh.pinned:214`）：这个函数每跑完一个阶段就调一次（`record_stage` 里，第 265 行 `drain_not_run_side_channel "$name"`），把 `GATE_NOT_RUN_FILE` 里这一阶段写的每一行收进汇总数组 `NOT_RUN`。
- `if [[ -n "$not_run_line" ]]; then NOT_RUN+=("$stage_name        本次未跑一部分：$not_run_line"); STAGE_REPORTED_NOT_RUN=1; fi`（`gate.sh.pinned:219`）：只要 `GATE_NOT_RUN_FILE` 里有一行非空，`STAGE_REPORTED_NOT_RUN` 就被置 1。
- `declare -A COVERED_BY=()`（`gate.sh.pinned:618`）：覆盖表在「项目本地阶段」循环之前清空。
- `if [[ $stage_rc -ne 0 || $STAGE_REPORTED_NOT_RUN -eq 1 ]]; then stage_counts_as_coverage=0; fi`（`gate.sh.pinned:668`）：阶段退出码非 0，**或者**这一阶段报过「本次未跑」，`stage_counts_as_coverage` 都会被清成 0——54 号快档报过 `!` 那几行「本次未跑一部分」时，`STAGE_REPORTED_NOT_RUN` 一定是 1（因为 3.1 引的两行会先把它置 1），所以 `stage_counts_as_coverage=0` 这条分支必定命中。
- `elif [[ $stage_counts_as_coverage -eq 1 ]]; then`（`gate.sh.pinned:674`）与紧接着的 `COVERED_BY["$covered_key"]+="${COVERED_BY[$covered_key]:+、}$sname"`（`gate.sh.pinned:675`）：只有 `stage_counts_as_coverage` 还是 1 时，`COVERED_BY[崩溃点重放]` 才会被写入；上一条已经确认「本次未跑」会话让它变 0，所以这一行**不会**执行，`COVERED_BY[崩溃点重放]` 保持为空。

**结论**：54 号报过「本次未跑」的这一轮，`# gate-covers: 崩溃点重放` **不算覆盖**——`COVERED_BY[崩溃点重放]` 仍是空，
汇总里那一项继续列在未实现清单（`gate.sh.pinned` 第 747 行附近 `if [[ -z "${COVERED_BY[$key]:-}" ]]; then warn ...` 那一段，本报告未逐字核这一行，判定已经由 3.1 前四条推出，不再重复核对）。
这与正文判据表 Q3 的「判绿的观测」逐句对得上：`drain_not_run_side_channel` 把 `STAGE_REPORTED_NOT_RUN` 置 1、`gate-covers` 不算覆盖，两句都成立。
**分类：兑现了条款**（D13（验证路线） 已定项 15「checker 档默认只在提交时跑快档，全量由用户要求或夜间」的执行落地）。
**什么现象会推翻它**：如果 `gate.sh` 未来某次改动把 668 行那条 `stage_counts_as_coverage=0` 的判断条件从「`|| STAGE_REPORTED_NOT_RUN -eq 1`」删掉，只留 `stage_rc -ne 0`，那么报过「本次未跑」的 54 号只要自身退出码是 0，`崩溃点重放` 就会被错误地记成「已覆盖」——这个改动本身就是推翻条件。

### 3.2 单跑（`GATE_NOT_RUN_FILE` 不在）时，有没有别的东西告诉人全量没跑

**判绿的观测**（都在 `.claude/gate.d/54-layer0-replay.sh`，现查 `sha256sum` 与快照一致：`dc52e32e4ceae64a7514655b18be054617e5878dcaa22008eae6d7c5928f7433`）：

- `echo "  ! ${#missing_cases[@]} 条崩溃枚举用例没有作数的全绿标记，全量这一轮没跑，记「本次未跑」、不判红（全量默认不在提交时跑，D13 已定项 15）：${missing_cases[*]}"`（`.claude/gate.d/54-layer0-replay.sh:447`）
- `echo "     → 要跑全量（用户要求或夜间）：暂存之后，在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 --full <它的根>"`（`.claude/gate.d/54-layer0-replay.sh:453`）
- `if [[ -n "${GATE_NOT_RUN_FILE:-}" ]]; then`（`.claude/gate.d/54-layer0-replay.sh:456`）：这一段守卫只包住往 `GATE_NOT_RUN_FILE` 里写那几行（458 行 `echo "崩溃枚举用例 ${missing_case} 这批输入没有作数的全绿标记，全量没跑（要跑：54 号 --full）" >> "$GATE_NOT_RUN_FILE"`），**不包住** 447、453 两行——447、453 两行在 `if (( ${#missing_cases[@]} > 0 ))` 这个更外层的判断里（440 行左右开始），跟 `GATE_NOT_RUN_FILE` 变量在不在无关，单跑（`bash .claude/gate.d/54-layer0-replay.sh`，不经 `gate.sh`）时这个变量确实不会被设置，但只要有登记的用例缺当次有效标记，447 行的 `!` 与 453 行的 `→` 照样会打到标准输出。

**结论**：单跑时确实有 `!` 与 `→` 两行告诉人「全量这一轮没跑」，与判据表「判绿的观测」逐句对得上。**分类：兑现了条款**。
**什么现象会推翻它**：把 447、453 两行也挪进 456 行那个 `if [[ -n "${GATE_NOT_RUN_FILE:-}" ]]` 块里，单跑时这两行就不会再打印，届时判定要改判红。

### 3.3 有没有一条路让 54 号退 0 而汇总里既没有「本次未跑」也没有「未实现」

**判红的观测**（同一份 `54-layer0-replay.sh`，逻辑复核，未实际起跑 `cargo test`——起跑属于重型测试，子 agent 不跑，正推腿按代码推理即可，属于「正推：从代码推到结论」）：

`missing_cases` 数组只在循环体里两处 `continue` 之前被追加（426、438 行附近，对应「算不出指纹」与「没有作数的全绿标记」两种情形）。如果 `.claude/gate.d/stage-inputs.tsv` 里登记的每一条 `crash-case:` 行，这一批输入的指纹都能在 `git common-dir` 里找到一份**此前某次 `--full` 跑**留下的、当次仍然作数的全绿标记（`crash-case-marker-check` 判绿），`missing_cases` 就会是空数组：`(( ${#missing_cases[@]} > 0 ))`（`.claude/gate.d/54-layer0-replay.sh:446`）为假，447–460 行那一整段（`!` 行、`GATE_NOT_RUN_FILE` 写入、`→` 出路）**一行都不会执行**；脚本继续跑到 462–464 行打印 `✓` 并 `exit 0`。回到 `gate.sh.pinned`：`STAGE_REPORTED_NOT_RUN` 全程是 0，`stage_rc` 是 0，`stage_counts_as_coverage` 保持 1，`COVERED_BY[崩溃点重放]` 被写入（`gate.sh.pinned:674-675`）——汇总里「未实现」清单不会列出「崩溃点重放」，`NOT_RUN` 数组里也不会出现它。**这条路确实存在**，与判据表「判红的观测」字面成立。

**但这条路不是漏洞，是设计内的复用**：它触发的前提是「全部登记用例已经有一份对得上当前输入指纹的全量绿标记」——即某一次 `--full` 真的跑过、而且从那之后代码与输入都没变。这正是 `.claude/rules/verification.md`「全量 | 54 号 `--full`：在 HEAD + 暂存区的 worktree 里逐条跑登记的崩溃枚举用例（`--include-ignored --exact`），那一格全绿标记在就复用」（`.claude/rules/verification.md:33`，行号取自快照副本 `/tmp/claude-1000/three-way-forward/verification-split-r1/verification.md`，因工作区这份已被另一会话改动，`sha256sum` 与快照不一致，见附录清单说明）与 D13（验证路线） 已定项 15「checker 档自己分两档：**快档**是 `cargo test --release -p singlefs-checker` 不带 `--ignored`……**全量**是登记在 `.claude/gate.d/stage-inputs.tsv` 键为 `crash-case:` 的用例逐条 `--include-ignored --exact`……提交时不默认跑，由用户要求或夜间跑」（`.claude/kb/decisions/13-验证路线.md:318`）明写的复用机制——「未实现清单里不出现」对应的是「这批输入真的已经被验过」，不是「什么都没做就声称覆盖」。

**分类：兑现了条款**（属于 D13 已定项 15 与 verification.md 明写的复用行为，不是矛盾），但判据表字面问的是「有没有这条路」——**答案是有**，需要把「有路径」与「这条路径是否构成隐患」分开写：本报告的立场是它不构成隐患，因为复用要求指纹相等（代码、输入、工具链版本都没变），一旦任何一样变了，`crash-case-marker-check` 判不过，`missing_cases` 就非空，退回 3.1／3.2 的路径。

**另一条没量到的边角**：若 `.claude/gate.d/stage-inputs.tsv` 里 `crash-case:` 行本身被整批删空（`crash_case_rows` 数组为空），同一个 `for` 循环一次都不执行，`missing_cases` 同样是空数组，会走到同一条「退 0、未实现清单不列出」的路——这种情形在真仓里现在不成立（背景材料第二节：「8 行 `crash-case:`」加新登记的第 9 行，`crash_case_rows` 非空），**这一格没有实测复现，只是代码结构上看得出的另一条同型路径，标「没量到」，留给 D13 已定项 15 之外的另一条门禁（若要接，应加一条「`crash_case_rows` 为空时不许算覆盖」）**。

## Q5：定义有没有自相矛盾（每次提交跑层 0 全量 / 按名字含 layer0 判）

逐句核对下列六份文件，`sha256sum` 现查全部与快照一致（`research/prompts/verification-split-r1-snapshot/kb-sha256.txt` 或直接 `git show refs/sop/verification-split-r1-snapshot:<路径> | sha256sum` 现比对过，六份都是 `SAME`）：`.claude/main-agent.md`、`.claude/agents/crash-verifier.md`、`.claude/agents/gate-triage.md`、`.claude/agents/implementation-writer.md`、`.claude/agent-common.md`、`.claude/rules/implementation-workflow.md`。

**main-agent.md 禁止第 1 条（挂钟长不是收窄它的理由）与 verification.md「全量默认不在提交时跑」**：

- `.claude/main-agent.md:10`：「崩溃点测试准确率和正确率优先，其次再衡量时间成本」：要加崩溃点、要多枚举一档，就加。挂钟长不是收窄它的理由，优先保证正确率和准确率。这一条管枚举域，不管跑的时机：什么时候跑按 `.claude/rules/verification.md`，checker 档默认只在提交时跑快档，全量由用户要求或夜间。
- 这句话**自己就写明了射程**：它管的是「枚举域」（层 0 要不要多枚举一档、要不要更细），不管「什么时候跑」，并且显式把后者交给 `verification.md`。所以它与 `.claude/rules/verification.md:18`「默认只在提交时跑；想单独跑就带 `SINGLEFS_HEAVY_TESTS=user-request`」（行号取自 verification.md 快照副本，工作区这份已被另一会话改动、`sha256sum` 与快照不一致）**不矛盾，是同一句话里的两个分句**，没有一句话要求「每次提交跑层 0 全量」。

**crash-verifier.md 第 1、1b、1c、2、4 步与 54 号新语义**：

- `.claude/agents/crash-verifier.md:22`：54 号跑哪一档：默认快档（`cargo test --release -p singlefs-checker` 加逐条核全绿标记，`.claude/rules/verification.md`），派发提示什么都不写就是它；要跑全量的写明「54 号带 --full」……
- 第 1、1b、1c 步（27–29 行）只处理「要不要起 54 号」「起哪一档」「双机分片怎么配」这几件事，没有一处写「无论如何都要 `--full`」；第 2 步（30 行）重复了「54 号默认跑快档……派发提示写明「54 号带 --full」时才在 HEAD + 暂存区的 worktree 里……跑全量」；第 4 步（32 行）只是「原样抄它的判定行」，不含跑法的判断。**五步逐句读下来没有一句要求每次提交跑全量、也没有任何一处按名字里含不含 `layer0` 来判**（54 号的判定完全按「带不带 `--full` 参数」与「crash-case 标记指纹对不对」，不看测试函数名字）。

**gate-triage.md 第 2、3 步**：

- `.claude/agents/gate-triage.md:28`：……54、55、57、59 这几道重阶段在 `gate.sh` 里照各自的复用判定走（**54 号跑快档并核全绿标记，标记不作数报「本次未跑」不判红**，`.claude/rules/verification.md`），你不直接调它们……
- 第 3 步（29 行）讲的是「红了怎么归属这一轮 / 不是这一轮 / 环境 / 并发」，不涉及要不要跑全量。**两步都直接引用了 54 号的新语义（快档 + 复用判定 + 本次未跑不判红），没有一句要求全量**。

**implementation-writer.md 第 4、7a 步与交回那一节**：

- `.claude/agents/implementation-writer.md:34`：……checker 档不跑（`.claude/rules/verification.md`）：自己新加的崩溃枚举流写进 `crates/singlefs-checker/tests/`，不在交回前跑，随提交时的 54 号快档与之后的全量验……
- `.claude/agents/implementation-writer.md:42`：7a. 新写的崩溃枚举用例（测试函数直接调 `enumerate_layer0` 一族做全量枚举、不是 `quick_tier` 那几个的）一律写在 `crates/singlefs-checker/tests/` 里、标 `#[ignore]`……
- 第 4 步明写「checker 档不跑」「交回前不跑」，把验证挪到「提交时的 54 号快档与之后的全量」——即实现员本身**从不**在交回前跑层 0；7a 步按「是不是直接调 `enumerate_layer0` 一族」分流，**不是按函数名字含不含字面 `layer0`**（`enumerate_layer0` 与 `layer0` 不是同一个判据：`crates/singlefs-harness/tests/` 里名字含 `layer0` 但不直接调 `enumerate_layer0` 的用例，按 verification.md 的定义仍属于 harness，这与「按名字含 `layer0` 判」是两种不同的判据，implementation-writer.md 用的是后者更精确的那种）；「产出」一节「全文写进报告文件，交回只写结论、报告路径与 `sha256sum`」（`.claude/agents/implementation-writer.md:53`）不含任何验证动作。**没有一句要求每次交回跑层 0 全量**。

**agent-common.md「不做」一节**：

- `.claude/agent-common.md:47`：重型测试就是 checker 档那一批（`.claude/rules/verification.md`）：……与 54 号、QEMU、herd7、crates 变异整表、全量 `cargo test`……只在提交代码时跑一次、或用户要求时跑；子 agent 一律不跑……
- 这句把「54 号」整体（不分快档全量）划进「重型测试」，规定的是**子 agent 一律不跑**、只在提交或用户要求时才跑，**没有说提交时跑的是全量**——具体跑哪一档由 `crash-verifier.md` 与 `verification.md` 定（默认快档）。这与「每次提交跑全量」不矛盾：它管的是「谁能跑、什么时候能跑」，不管「跑哪一档」。

**implementation-workflow.md「checker 档只在提交时跑，harness 随时跑」与「复用上一次全量门禁的判定」两节**：

- `.claude/rules/implementation-workflow.md:53`：| 每次提交代码 | checker 档快档，命令带 `SINGLEFS_HEAVY_TESTS=commit`：`crash-verifier` 跑 54 号（快档：`cargo test --release -p singlefs-checker`，再逐条核崩溃枚举用例的全绿标记，不作数的报「本次未跑」）与 55、57、59 号……
- `.claude/rules/implementation-workflow.md:54`：| 用户要求，或夜间 | checker 档全量：54 号 `--full`（HEAD + 暂存区的 worktree 里，逐条按输入复用）……
- 表格本身把「每次提交」与「用户要求或夜间」分成两行、各自写清跑哪一档，**字面上没有交叉**；「复用上一次全量门禁的判定」一节（30–44 行）讲的是**别的阶段**（未登记复用判定的阶段每次照跑、登记了的按输入指纹复用），不特指 54 号也不要求全量，与「checker 档只在提交时跑」一节不矛盾。

**结论（Q5 整体）**：逐句核对下来，六份文件里**没有一句**要求「每次提交都跑层 0 全量」，也**没有一句**按测试函数名字是否包含字面 `layer0` 来判定档位（`crash-case-check.py` 与 verification.md 用的判据是「是否直接调 `enumerate_layer0` 一族做全量枚举」与「住在哪个包」，是比名字更严的判据，`crates/singlefs-harness/tests/` 下仍有名字含 `layer0` 但不属于 checker 档的用例，按这条更严的判据它们照样是 harness）。**分类：兑现了条款**——D13（验证路线） 已定项 15 定的「checker 档默认只在提交时跑快档，全量由用户要求或夜间」这句话，在这六份文件里被逐一落地成一致的执行细则，没有找到反着说的句子。

**什么现象会推翻它**：日后如果这六份文件里任何一处改成「每次提交」加「层 0 `--full`」（或去掉「默认快档」四个字变成裸的「跑 54 号」）、或者 `crash-case-check.py` 的判据被换成字面 `layer0` 子串匹配（这样会把 harness 里名字含 `layer0` 但不调 `enumerate_layer0` 的用例也划进 checker 档，判据变粗），Q5 的判定要改判「和条款说反话」。

## Q7：D13（验证路线） 已定项 15 与已定项 4、5、7、9 之间有没有互相矛盾的句子

`.claude/kb/decisions/13-验证路线.md` 现查 `sha256sum` 与快照一致（`493d33351f2c7279cc786da2e44897e5308356c83f76c8fbd61d49a6b8f956b8`），下面行号是这份文件里现 `grep -n` 出来的。

### 7.1 已定项 4、5、7 与已定项 15：没有互相矛盾的句子

- 已定项 4（`.claude/kb/decisions/13-验证路线.md:70`）定的是「崩溃点重放的崩溃状态集合怎么定义」——按设备切段、整写取几态；它管的是**模型层的枚举域**，已定项 15 管的是**代码住哪个包、什么时候跑**，两者是正交的轴，正文互相没有引用也没有互相否定的句子。
- 已定项 5（`.claude/kb/decisions/13-验证路线.md:85`）的射程句：「集成测试不算共享：`crates/singlefs-checker/tests/` 下的崩溃枚举用例经 `[dev-dependencies]` 依赖 `singlefs-core` 与 `singlefs-harness` 造镜像再交给 checker 判，门禁 94 号只读 `[dependencies]` 与 `[build-dependencies]`（已定项 15）」（`.claude/kb/decisions/13-验证路线.md:93`）；已定项 15 的射程句同样写：「不管 checker 库与实现共享什么（已定项 5：checker 包的 `[dev-dependencies]` 依赖 core 与 harness 造镜像再判，不算共享）」（`.claude/kb/decisions/13-验证路线.md:320`）。**两句互相点名、互相复述同一个结论（dev-dependencies 造镜像不算共享），字面一致，不矛盾**。
- 已定项 7（`.claude/kb/decisions/13-验证路线.md:126`）定的是「O2（独立解析器 + checker） 判的是一个镜像」，管的是判定域（单镜像谓词 vs 需要第二个输入的记录核对器），与已定项 15 的射程句（转述：「checker 档」与「池级 checker」是两个名字，前者是这一档的测试，后者是 `check_pool_image` 那个一元谓词）——原文「是两个名字：前者是这一档的测试，后者是 `check_pool_image` 那个一元谓词（已定项 7）」（`.claude/kb/decisions/13-验证路线.md:320`）只是把两个名字分清楚，**没有互相否定的句子**。

**分类：兑现了条款**——三个已定项与已定项 15 之间没有找到互相矛盾的句子，已定项 5、7 反而在射程句里互相点名、互相印证。

### 7.2 已定项 9 定案表「层 0 冒烟」那一行改了触发之后，「范围」列与射程句自相矛盾（已定项 9 内部）

**判红的观测**——这不是已定项 9 与已定项 15 之间的矛盾，是**已定项 9 自己的定案表与自己的射程段之间**的矛盾：

- 定案表：「层 0 冒烟 | 固定种子的最小复现负载（写请求数两位数） | **不抽样，全量** | checker 档的全量（已定项 15）：用户要求或夜间跑；提交时默认只跑快档 |」（`.claude/kb/decisions/13-验证路线.md:175`）——「范围」列仍写「写请求数两位数」。
- 射程段：「登记在 `.claude/gate.d/stage-inputs.tsv` 的崩溃枚举用例已经长到并行线一那条流 12230590578 个状态，**不再是「写请求数两位数」的冒烟体量**，所以提交时默认只跑快档，全量按已定项 15 由用户要求或夜间跑」（`.claude/kb/decisions/13-验证路线.md:191`）。

这两句同属已定项 9，字面直接冲突：定案表说「范围」是「写请求数两位数」，射程段紧接着说「不再是『写请求数两位数』的冒烟体量」——**射程段用引号原样引用了定案表那句话，再原地否定它**。这不是我读出来的隐含矛盾，是文档自己写出来的显式矛盾。

**现查这一轮的 diff**（`research/prompts/_verification-split-r1-diff.md`，现 `grep -nF` 过）：

- `research/prompts/_verification-split-r1-diff.md:609`：`-| 层 0 冒烟 | 固定种子的最小复现负载（写请求数两位数） | **不抽样，全量** | 任何触碰写路径的提交 |`
- `research/prompts/_verification-split-r1-diff.md:610`：`+| 层 0 冒烟 | 固定种子的最小复现负载（写请求数两位数） | **不抽样，全量** | checker 档的全量（已定项 15）：用户要求或夜间跑；提交时默认只跑快档 |`

对比这两行：**这一轮只改了「触发」列（最后一列），「范围」列（`固定种子的最小复现负载（写请求数两位数）`）一个字没动**。同时 diff 里射程段那一行也在这一轮被改写、新插入了「不再是『写请求数两位数』的冒烟体量」这句话（`research/prompts/_verification-split-r1-diff.md` 里对应的 `+` 行与本报告 191 行引用的正文相同，插入前的旧射程段没有这句）。**这意味着这一轮的作者已经在射程段里显式承认了「范围」列过期，却没有同时改「范围」列本身**——这是一处这一轮引入、且自己在同一次改动里点破又没收口的矛盾。

**结论**：「层 0 冒烟 = 固定种子的最小复现负载（写请求数两位数）」这一格**不再成立**，射程段自己已经这么说了；这一格**要改**，改成与射程段一致的说法（例如把「范围」列拆成「冒烟档：固定种子的最小复现负载（写请求数两位数）」与「全量档：登记的崩溃枚举用例，今天已长到十亿级状态」两行，或者干脆把这句"两位数"的定义限定成只描述**层 0 快档甲二**的负载规模，不再用来描述整个「层 0」）。**分类：这一轮的改动和条款说反话**——不是与已定项 15 说反话，是已定项 9 定案表与它自己刚被这一轮改写过的射程段说反话。

**什么现象会推翻它**：如果日后把「范围」列改成不再用「写请求数两位数」这个具体数字描述（比如改成「甲二快档的最小复现负载（数量级见射程）」），或者射程段那句「不再是……」被删掉、不再自相矛盾，这一条判定要重新核。

### 7.3 已定项 15「checker 档自己分两档」与已定项 9 的「三层」怎么对得上

- 已定项 15 定案句：「checker 档自己分两档：**快档**是 `cargo test --release -p singlefs-checker` 不带 `--ignored`……**全量**是登记在 `.claude/gate.d/stage-inputs.tsv` 键为 `crash-case:` 的用例逐条 `--include-ignored --exact`……提交时不默认跑，由用户要求或夜间跑，**跑法按已定项 9 分层**」（`.claude/kb/decisions/13-验证路线.md:318`）。
- 已定项 9 的「三层」（层 0 冒烟 / 层 1 常规 / 层 2 全量）是**崩溃点重放负载规模**的分类轴；已定项 9 自己的射程句写明：「今天只有层 0（门禁 54 号）；层 0 分两档：平时快档……全量……层 1、层 2 在门禁里没有阶段」（`.claude/kb/decisions/13-验证路线.md:191`）。

**这两句合起来读是一致的，不是并列的两套分类**：已定项 15 的「快档／全量」两档，说的就是已定项 9 里「层 0」内部的那两档（「层 0 分两档：平时快档……全量……」），已定项 15 自己也写了「跑法按已定项 9 分层」，把「全量」具体怎么跑的细节显式交还给已定项 9。已定项 9 的「层 1」「层 2」目前都「在门禁里没有阶段」（`.claude/kb/decisions/13-验证路线.md:191`），不存在的东西不会跟已定项 15 的「两档」打架。

**唯一需要指出的措辞歧义（不是矛盾）**：已定项 15 定案句开头把 QEMU（55 号）、herd7（57 号）、变异整表（59 号）、实验复跑（87 号）与 checker 包的测试一起归进「checker 档」，然后说「checker 档自己分两档」——字面上容易读成「55/57/59/87 号也各自分快档／全量」，但已定项 15 紧接着给出的快档／全量定义（`cargo test --release -p singlefs-checker` 与 `crash-case:` 逐条 `--include-ignored`）**只描述 54 号**；55/57/59/87 号按 `.claude/rules/implementation-workflow.md`「复用上一次全量门禁的判定」一节各自走自己的输入指纹复用（不是「快档/全量」这种二分）。这是措辞上「checker 档」这个名字覆盖面（五样东西）与「自己分两档」这句紧跟着的定义覆盖面（只有 54 号）不完全对齐，**但没有一句话直接互相否定**，判定为「没量到互相矛盾的字面句子，但这一处指代范围值得下一轮改窄措辞」。

**分类：兑现了条款**（已定项 15 与已定项 9 之间没有互相矛盾的句子，且互相点名核对）；上面提到的措辞覆盖面问题单独记，不算矛盾。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| Q3.1 覆盖算不算 | 兑现了条款 | `gate.sh.pinned:214/219/668/674-675` 逐句证实：报过「本次未跑」的 54 号不算覆盖 `# gate-covers: 崩溃点重放` |
| Q3.2 单跑有没有告知 | 兑现了条款 | `54-layer0-replay.sh:447/453` 的 `!`/`→` 不受 `GATE_NOT_RUN_FILE` 守卫限制，单跑照样打印 |
| Q3.3 有没有退 0 且两样都不报的路 | 有这条路，但是设计内的复用 | 全部登记用例已有当次有效全绿标记时，54 号退 0 且计入覆盖；这是 D13 已定项 15／verification.md 明写的「逐条按输入复用」，不是漏洞；`crash_case_rows` 为空这一支路径「没量到」 |
| Q5 六份定义有没有自相矛盾 | 兑现了条款 | 逐句核对 main-agent.md/crash-verifier.md/gate-triage.md/implementation-writer.md/agent-common.md/implementation-workflow.md，没有一句要求每次提交跑层 0 全量，也没有一句按名字含 `layer0` 判 |
| Q7.1 已定项 4/5/7 与 15 | 兑现了条款 | 三项正交或互相点名印证，没有互相否定的句子 |
| Q7.2 已定项 9「范围」列 vs 射程句 | 这一轮改动和条款说反话（已定项 9 内部自相矛盾） | 定案表「写请求数两位数」与射程句「不再是『写请求数两位数』的冒烟体量」原地互相否定，diff 证实这一轮只改了「触发」列、没改「范围」列，也是这一轮新插入的射程句自己点破了这处矛盾却没收口 |
| Q7.3 已定项 15「两档」与已定项 9「三层」 | 兑现了条款（附一处措辞覆盖面歧义） | 「两档」具体指层 0 内部的快档/全量，与已定项 9 互相点名一致；已定项15开头把55/57/59/87号也归进「checker档」再说「自己分两档」，字面覆盖面比紧跟的定义（只讲54号）宽，未发现互相矛盾语句 |

## 没做什么

- 不判 Q1、Q2、Q4、Q6、Q8、Q9：按分工表这几格归云端攻方（Opus）与本地攻方，不属于这条腿。
- 没有实际起跑 `cargo test -p singlefs-checker`、`gate.sh` 或任何 checker 档测试：Q3、Q5、Q7 都是文本对文本、代码对代码的逐句核对（正推：从代码与 kb 原文推到结论），不需要跑重型测试；子 agent 本就不许跑 checker 档（`.claude/agent-common.md`「不做」一节）。
- 没有替主 agent 采纳或出判决：本报告只给判定与观测，三方一致与否、要不要改 D13 已定项 9「范围」列，由主 agent 按 `.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」定。
- 没有核对 `.claude/gate.d/94-checker-implementation-disjoint.sh`、`research/scripts/crash-case-check.py`、`lib_heavy_tests.py` 的具体判据实现细节（这些是 Q1、Q4、Q6 的范围，归 Opus）；Q5 提到 `crash-case-check.py` 的判据（「是否直接调 `enumerate_layer0` 一族」）只在核对 implementation-writer.md 措辞时顺带引用了它的口径，没有去读它的源码逐行核。
- `research/scripts/cite-check.py` 的自跑结果见交回消息里贴的末行。
