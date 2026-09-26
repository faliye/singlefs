# governance-defs-r3 云端正推（Sonnet）

立场：从第二轮审查报告（甲/乙/丙组，r2-sync）的现查结论，逐处推「改后的句子是不是那一条的直接后果」；不做的：不判 G1-G5 之外的格，不替主 agent 采信或出判决，不判本地腿（缺席）与云端攻方（Opus）那两条腿的格。

行号规则：定义与规则文件的行号一律用 `grep -n` 在当前仓文件里现取；审查报告的行号用 Read 工具 `cat -n` 现读并用 `grep -nF` 复核；背景材料 `_governance-defs-r3-*.md` 只用来确认「这一轮要判什么」的范围，不从里面数行号。

## G1 59 号计数（mutation-triage.md 第 4、5 步；mutation-sampling.md「改了一个格式常量之后」一节）

### 对应关系

改动对应乙组第 1 条（`research/prompts/governance-review-r2-report-B.md:9`）与甲组 A3（`research/prompts/governance-review-r2-report-A.md:11`，两条指向同一处）。

乙组第 1 条建议改法（原文，`report-B.md:9`）：
「两处改成：「59 号没有 💥、⚠️ 两栏：点名测试没打出结果行时，输出里有 `error:` 行（cargo 的 `error: test failed` 也算）或 `could not compile` 就记「无效」，否则记「没红」（没跑到），两种都让整道判红；分类前看那一条附的末 8 行，有 `process didn't exit successfully … (signal` 的按 💥 报、不按第八类」。」

改后现文（`.claude/agents/mutation-triage.md:28`，与 `.claude/rules/mutation-sampling.md:82` 同句，两处已用 `grep -n` 核对逐字相同）：
「…59 号照它自己收尾的「计数：」行，那一行没有 `💥`、`⚠️` 两栏：测试进程没跑完的，输出里有一行 `error:` 开头（测试二进制被信号杀时 cargo 也打 `error: test failed`）或 `could not compile` 就记「无效」，否则记「没红」，两种都让整道判红；「无效」里尾巴带 `process didn't exit successfully` 的是进程被杀，不是替换文编不过；另外三栏「被总上限挤掉」「排不上没跑」「scope 起不来没跑」不为 0 也让整道判红，照原样另列…」

### 判定：前半句是直接后果，后半句（分类去向）没有完全做到

前半句（「无效」的判定条件从「有编译错误」改成「有 `error:` 行或 `could not compile`」）与乙组第 1 条建议逐字对应，是直接后果。

后半句不是：乙组建议的是「分类前看那一条附的末 8 行…有 `process didn't exit successfully…(signal` 的**按 💥 报、不按第八类**」——这是给 mutation-triage.md 第 6 步「没红与无效的逐条按 `mutation-sampling.md` 分类」一个明确去向（改分到 💥 类）。改后现文只写「是进程被杀，不是替换文编不过」，**否定了它是第八类，但没有给出它该分到哪一类**。核 `.claude/rules/mutation-sampling.md` 的分类清单（`grep -n '^#' .claude/rules/mutation-sampling.md`）：第七类「变异压根没跑」、第八类「替换编不过」，**没有第九类**收「测试进程被信号杀」；而 `💥` 本身在改后现文里明写「59 号没有 💥、⚠️ 两栏」（`mutation-triage.md:28`），59 号的计数体系里根本不存在 💥 这一栏可归类。乙组建议里「按 💥 报」这半句放到 59 号语境下本身就自相矛盾（59 号没有 💥 栏），改后现文回避了这个矛盾，但代价是把「进程被杀该分进第几类」这个问题留白——分诊员第 6 步（`mutation-triage.md:29`）「没红与无效的逐条按 `mutation-sampling.md` 分类」对这一种「无效」条目答不出该归第几类。

什么现象会推翻它：`mutation-sampling.md` 或 `mutation-triage.md` 别处存在一个第九类（或对第八类的扩写），点名收「进程被信号杀」这种「无效」；现查 `grep -n '第九类\|信号杀\|process didn.t exit' .claude/rules/mutation-sampling.md` 未见新增类目，只有本处这一句，判定站得住。

### G1 另一处：三栏「不为 0 也判红」的去向同样只完成一半

乙组第 4 条（`report-B.md:12`）建议：「第 5 步 59 号那半句补：「被总上限挤掉、排不上没跑、scope 起不来没跑三栏不并进三个数，不为 0 的那一次 59 号整道已判红、**这几条没有结论，逐条列出交主 agent**」。」

改后现文（`mutation-triage.md:28`）把「不为 0 也让整道判红」写进了 59 号自己那句括注里（「另外三栏…不为 0 也让整道判红，照原样另列」），但「逐条列出交主 agent」这半句留在同一行后半、原有的 `⚠️`、`⏱`、`🧱` 那一组里（「其余几栏不并进三个数：`⚠️`、`⏱`、`🧱` 不为 0 的整轮已判失败，`mutate.sh` 的 `💥` 不判失败，逐条列出交主 agent」），语法上「逐条列出交主 agent」只挂在 `⚠️⏱🧱`/`💥` 那一组名词短语后面，不直接挂在新加的「被总上限挤掉」等三栏上——按字面读，新三栏被要求「判红」，但没有被同一句里的「逐条列出交主 agent」覆盖到，是否也要交主 agent 不写得很显。

什么现象会推翻它：把「其余几栏不并进三个数」读成一个统括全部非三数栏目（含新三栏）的主句，「逐条列出交主 agent」自然覆盖新三栏——这是一种合理的中文读法，但不是唯一读法；这一条判「读法有歧义、审查建议的显式说明没有被逐字保留」，不判「一定会出错」。

## G2 崩溃验证员（crash-verifier.md 第 1 步与产出）

### 未跟踪文件检查（对应乙组第 9 条，`report-B.md:17`）

乙组第 9 条建议改法：「57 号补 `.claude/singlefs-ai-sop/scripts/lib.sh .lkmm-static-only`；未跟踪检查限定到 `crates/`（理由只罩那里），其余路径只做 `git diff --quiet`。」

改后现文（`.claude/agents/crash-verifier.md:27`，`grep -n` 现取）：
「…跑 `git diff --quiet -- <这些路径>`（退 0 才说明工作区与暂存区在这一道的输入上相同）与 `git ls-files --others --exclude-standard -- crates litmus .lkmm-static-only`（cargo 会把未跟踪的 `src/bin`、`tests` 文件认作目标，仓根有 `.lkmm-static-only` 时 57 号只跑静态那几层，要没有输出；别处的未跟踪文件不挡）。」

判定：**只做了建议的一半，另一半在这个改法下其实做不到**。

- 「宽的一边」（55 号未跟踪检查此前覆盖了整个 `research/scripts/ research/results/`，导致别的会话没提交的实验产物会挡住提交）被修正：未跟踪检查改成对 55/57/59 三道统一收窄成 `crates litmus .lkmm-static-only`，不再带 `research/scripts/ research/results/`。这与建议的方向一致（虽然字面不是「限定到 `crates/`」，而是「crates + litmus + 标记文件」，比建议略宽，但已达成「不挡别的会话的实验产物」这个目标）。
- 「窄的一边」（57 号还读 `.claude/singlefs-ai-sop/scripts/lib.sh` 与仓根 `.lkmm-static-only`，此前都不在检查范围里）没有被完整落实：`.lkmm-static-only` 确实被加进了新的 `git ls-files --others` 那一行，但 `lib.sh` 哪儿都没有出现——既没进 `git diff --quiet` 的输入集（57 号那句「litmus crates .claude/scripts/lkmm.sh」原样未改，`grep -n` 现查该行只有这一次出现），也没进未跟踪检查。现查 `.claude/singlefs-ai-sop/` 的 git 属性：`git check-ignore -v .claude/singlefs-ai-sop/scripts/lib.sh` 命中 `.gitignore:6:/.claude/singlefs-ai-sop/`，`git ls-files .claude/singlefs-ai-sop/scripts/lib.sh` 零输出——这个文件既不在版本库里、也被 `.gitignore` 挡着，`git diff --quiet` 与带 `--exclude-standard` 的 `git ls-files --others` 两种命令**原理上都看不到它的改动**。乙组建议里「57 号补…lib.sh」这半句，按现有的两条 git 命令机制并不可执行；改后现文没有写这半句，也没有写明为什么不写。

什么现象会推翻它：`.claude/singlefs-ai-sop/` 改成不被 `.gitignore` 挡、或改后定义别处（例如另加一条比较文件 mtime/hash 的检查）已经在管 `lib.sh` 的漂移——现查这一轮改动范围（`git diff 1c58cfa bfc447e -- .claude/singlefs-ai-sop`）零命中，没有这样的检查，判定站得住：**乙组第 9 条关于 lib.sh 的那一半建议，在写死用 git 命令做未跟踪/暂存对比这个前提下不可执行，改后定义悄悄丢弃了它，但没有留痕**。

### 与它衔接的一侧：mutation-triage.md 的「输入」没有跟着改

乙组第 7 条（`report-B.md:15`）建议是两处一起改：「验证员产出补一列「日志路径」…；**分诊员那一句写明「崩溃验证员报告里 59 号那一行的日志路径」**。」

`crash-verifier.md` 的产出确实加了日志路径列（见下一节）；但 `mutation-triage.md` 的「输入」一节（`grep -n '门禁 59 号的输出路径' .claude/agents/mutation-triage.md` → `19:- 分 \`crates/mutations.tsv\` 里的条目时：提交时那一次门禁 59 号的输出路径（缺它不开工）。`）这一轮的 diff 没有触碰这一行（`git diff 1c58cfa bfc447e -- .claude/agents/mutation-triage.md` 只改了第 27、28 行），仍然只写「门禁 59 号的输出路径」，没有点名「崩溃验证员报告里那一列日志路径」。这不是错误（分诊员本来就要去崩溃验证员的报告或草稿目录找这份输出），但审查建议明确要求两处一起改、其中一处没跟上，是一次不完整的双边修复。

什么现象会推翻它：崩溃验证员的报告模板此前就已经把「59 号输出路径」这个提法与「日志路径」列绑定（例如报告本身直接写「59 号输出路径 = 日志路径」），使得 mutation-triage.md 不改这句话也不会产生歧义——但这需要另外核崩溃验证员实际写出来的报告格式，本报告没有那种报告可核，不下这个结论。

### 产出加日志路径列（对应乙组第 7 条）

改后现文（`.claude/agents/crash-verifier.md:36-37`，`grep -n` 现取）：
「一张表：阶段 / 退出码（0、非 0、77）/ 耗时 / 原样的 ✓，或 ✗ 与 → / 计数行 / 日志路径；末尾「没做什么」。每个阶段的整份输出都写进 `<草稿目录>/<阶段>.log`（前台跑也写），59 号那一份是变异分诊员要的输入。」

与乙组第 7 条建议「验证员产出补一列「日志路径」（59 号必给，前台跑的也把输出落进 `<草稿目录>/<阶段>.log`）」逐字对应，是直接后果，判定：站得住。

什么现象会推翻它：崩溃验证员实际交回的报告没有这一列，或前台跑的阶段没有落 `.log` 文件——这需要一次真实运行的报告去核，这一轮没有这样的产物，「核不动」。

## G3 核查员（three-way-verifier.md 输入与第 1 步；implementation-workflow.md「代码轮派腿之前记一份开工快照」一节）

### 对应关系

对应乙组第 2 条（`research/prompts/governance-review-r2-report-B.md:10`）。

乙组第 2 条建议改法：「设计轮不再一律「分不清」：照第 28 行清单外文件的办法，用 `git log --since=<腿开工时刻> -- <文件>` 与 `git status --short -- <文件>` 现查，**没被改过的对不上记 ✗、被改过或查不了的才记「分不清」**；第 1 步写明自证挑一条「文件在快照里且 `sha256sum -c` 通过、或腿开工之后没被改过」的引用。」

改后现文（`.claude/agents/three-way-verifier.md:21`，`grep -n` 现取）：
「…没给快照的轮（设计轮）对主树核，腿引的行号与主树对不上时，拿腿开工时刻照第 2 步现查那个文件在腿开工之后有没有被改过：**没改过记 ✗，改过或没给开工时刻记「分不清：文件可能在腿开工之后被改过」**（第 6 步的「分不清」一栏）。」

`three-way-verifier.md:27`：「判别力自证，先做：从待核的引用里挑一条，在草稿目录的副本里把它的行号加 1，按第 2 步核它（挑一条文件在快照里、或腿开工之后没被改过的），必须判 ✗；判不出就停下报告「核查方法不分辨」。」

### 判定：主体是直接后果；自证那句丢了一个限定词

「没改过记 ✗，改过…记「分不清」」与建议的「没被改过的对不上记 ✗、被改过或查不了的才记「分不清」」逐字对应（多出的「没给开工时刻」并入「分不清」分支，是把「查不了」具体化为「没给开工时刻」这一种，属于同一个判据的展开，不是新判据）。这一半是直接后果，能解开乙组指出的死结：判别力自证不再永远落进「分不清」。

第 1 步的自证挑选条件，建议原文是「文件在快照里**且 `sha256sum -c` 通过**、或腿开工之后没被改过」，改后现文写的是「文件在快照里、或腿开工之后没被改过」——**丢了「且 `sha256sum -c` 通过」这个限定词**。差别在于：一个文件「在快照清单里」不等于它现在的哈希还对得上快照（`three-way-verifier.md:28` 第 2 步本身写明「先拿快照清单 `sha256sum -c` 核那个文件；对得上就到主树那一行比内容，**对不上**就只对主 agent 给的倒推副本核，两样都没有的记「分不清」」）。如果自证时挑中一个「在快照清单里、但 `sha256sum -c` 核不过」的文件（被别的会话动过、又没有倒推副本），按第 2 步的逻辑它会落进「分不清」而不是 ✗，自证仍然会失败——这正是乙组第 2 条最初想堵住的那类死结的一个变体，只是触发条件从「设计轮」缩小成了「代码轮里挑错了自证对象」。

什么现象会推翻它：核查员实际执行自证时，「文件在快照里」这个条件在操作上就等同于「快照校验通过」（例如快照清单本身只登记当时被核实过哈希的文件，形式上不会有「在清单里但核不过」的情况）——现查 `implementation-workflow.md`「代码轮派腿之前记一份开工快照」一节（`grep -n '腿开工那一刻的快照\|sha256sum'`），快照生成的时点与自证执行的时点之间如果没有别的会话改动，两者确实等价；但这依赖「派腿之后没有别的会话动过快照里的文件」这个运行时条件，不是文字本身能保证的，所以判定是「丢了一个限定词，多数情况下无害，但字面不再等价」，不是「一定出错」。

### implementation-workflow.md「代码轮派腿之前记一份开工快照」一节：这一轮未改动，且已经满足 G3 的问题

`grep -n "代码轮派腿之前记一份开工快照" research/prompts/_governance-defs-r3-diff.md` 零命中，确认这一节这一轮没有被改动（`git diff 1c58cfa bfc447e -- .claude/rules/implementation-workflow.md` 只有「重型测试」表那一处改动，行号在 `implementation-workflow.md:52` 附近，不在这一节）。现查该节原文（`.claude/rules/implementation-workflow.md:26`，`grep -n` 现取）：「派三方的腿之前，把这一轮被判的文件与材料点名的 kb 文件记一份 `sha256sum` 快照（放这一轮的材料目录），**连同派腿的时刻（`date -u`）交核查员当输入**；腿跑着的时候主 agent 不改这些文件，要改的等判决时一起改。」——这一句本来就要求「派腿时刻」交核查员，与 `three-way-verifier.md:22`（未改动）「腿开工时刻（UTC）：第 2 步现查…要用」互相对得上，G3 提出的「「派腿时刻」由谁、在哪一步记，main-agent 有没有要求」这个问题在改动之前就已经有答案（main-agent 经这条规则要求派腿时给出），这一轮的改动没有影响这一点，也没有必要改它。

什么现象会推翻它：`implementation-workflow.md` 这一节在此前某一轮被删除或弱化过「连同派腿的时刻…交核查员当输入」这句——现查 `git log -p -- .claude/rules/implementation-workflow.md` 未去做（不在这一轮判定范围内，只需确认这一轮没有改它），判定站得住。

## G4 执行员入库装置（experiment-runner.md 第 2、4、5 步）

### bin 名连字符化（对应乙组第 6 条，`report-B.md:14`）

乙组第 6 条建议改法：「执行员那句改成「`name` 写 `e<号>-` 加英文名、下划线全换成连字符」。」

改后现文（`.claude/agents/experiment-runner.md:27`，`grep -n` 现取，节选）：「`research/e7-index-bench` 的 `[[bin]]` 的 `name` **全写连字符** `e<号>-<英文名里的下划线换成连字符>`」——与建议逐字对应，直接后果，判定站得住。什么现象会推翻它：仓里已有 bin 的 `name` 保留下划线的写法被当作有意的约定（乙组自己给出的推翻条件），这一轮没有找到这种反例（`grep -n 'name = "e[0-9]*-[a-z0-9-]*_' research/e7-index-bench/Cargo.toml` 未重跑，判定沿用乙组已给的现查）。

### 入库装置编译位置与登记（对应乙组第 5 条，`report-B.md:13`）

乙组第 5 条建议改法：「第 4 步补「入库装置那一支在仓根编（`cargo run -p singlefs-harness --bin e<号>_<英文名>`，经内存包装），二进制在仓根 `target/`」；第 5 步与写范围补「入库装置在 `replay.sh` 里登记成 `E<号>|@driver_e<号>|…` 并写同名驱动函数，形态照 E156」。」

改后现文：
- 第 4 步（`experiment-runner.md:30`）：「生成产物用的二进制编在 `research/target/` 下（**入库装置那一支编在仓根的 `target/` 下**），用最后一次源码改动编出来…」——落实了「二进制在仓根 `target/`」这半句；没有重复「`cargo run -p singlefs-harness --bin e<号>_<英文名>`，经内存包装」这个具体命令，但这个命令在第 5 步驱动函数（`driver_e<号>`，照 E156 先例）里已经落实——`replay.sh` 现存的 `driver_e156()`（`grep -n 'driver_e156' research/scripts/replay.sh`）本身就是 `(cd .. && cargo run -q -p singlefs-harness --bin e156_allocation_basis_counts)`，第 5 步要求「照 E156、E158 的先例另写一个 `driver_e<号>` 驱动函数」等于把这条命令的责任转交给了驱动函数本身，不是漏写，是换了个交代的位置。
- 第 5 步（`experiment-runner.md:32`）：「在 `research/scripts/replay.sh` 里登记复跑（`research/` 那一支写一行登记；**入库装置照 E156、E158 的先例另写一个 `driver_e<号>` 驱动函数，登记行写 `E<号>|@driver_e<号>||<产物>|exact`**），跑一次 `bash research/scripts/run-with-memory-cap.sh 4G bash research/scripts/replay.sh E<号>`…」——与建议逐字对应，直接后果。

判定：第 4、5 步是直接后果（其中「经内存包装」这半句通过第 5 步统一跑 `run-with-memory-cap.sh` 落实，不是漏项）。什么现象会推翻它：`driver_e<号>` 驱动函数被要求自己再套一层内存包装（即 `run-with-memory-cap.sh` 只包了 `replay.sh` 整体调用、驱动函数内部裸跑 cargo 会绕开包装）——现查 `research/scripts/replay.sh` 里 `driver_e156`、`driver_e158` 两个先例（`grep -n -A2 'driver_e156()\|driver_e158()' research/scripts/replay.sh`）均是裸 `cargo run`，包装只在外层 `run-with-memory-cap.sh` 那一层，与第 5 步「跑一次 `bash research/scripts/run-with-memory-cap.sh 4G bash research/scripts/replay.sh E<号>`」一致，未见矛盾。

### 写范围一节：文字没跟着改，但闸门本身不受影响

乙组第 5 条建议「第 5 步**与写范围**补…」——第 5 步已改，「写范围」一节（`experiment-runner.md:39`）这一轮没有改动（`git diff 1c58cfa bfc447e -- .claude/agents/experiment-runner.md` 只改了第 27、30、32 行），仍然写「`research/scripts/replay.sh` 里这个实验的登记行」，字面上只提「登记行」，没有提「驱动函数」。

这是否会让写范围闸拒绝写驱动函数：现查 `.claude/hooks/agent-write-scope.tsv`（`grep -n 'experiment-runner' .claude/hooks/agent-write-scope.tsv`）第 16 行是 `experiment-runner	research/scripts/replay.sh	复跑登记行`，第二列是**整份文件的 glob**（`research/scripts/replay.sh`，没有按行或按函数名收窄），`.claude/hooks/write-guard.sh` 的匹配逻辑（`grep -n 'glob_to_regex(pattern).match(target)' .claude/hooks/write-guard.sh`）按**文件路径**匹配，不按文件内容或改动的行匹配——这意味着闸门本身放行整份 `replay.sh` 的任何改动，写驱动函数不会被拒。乙组自己的「核过没问题的」一节（`report-B.md:31`）也现跑验证过「经包装的 `replay.sh E160`；经包装的 `replay.sh E156`」等对照，闸门放行。所以 G4 提出的「写范围闸放不放行驱动函数」这一问，答案是「放行」，但定义正文（`experiment-runner.md:39`）的措辞没有同步说明这一点——是一处文字与实际闸门行为不一致（文字偏窄），不是功能性缺口。

什么现象会推翻它：`agent-write-scope.tsv` 第 16 行被改成按行范围或函数名收窄（例如只放行「登记行」那一行、不放行函数体）——现查该表这一轮的 diff（`git diff 1c58cfa bfc447e -- .claude/hooks/agent-write-scope.tsv`）零改动，判定站得住。

## G5 其余（kb-scribe.md 输入与写入后钩子；agent-common.md「不做」一节；main-agent.md「暂存之后、提交之前跑门禁」一行）

### 决策标题状态 N 的写法（对应乙组第 3 条，`report-B.md:11`）

乙组第 3 条给了两条并列出路：「写成「（N项未定），N 与「项」之间不留空格；N 写阿拉伯数字，或一到十的单个汉字」；**或**把两道门禁的正则对齐（交丙组）。」丙组第 11 条（`research/prompts/governance-review-r2-report-C.md:19`）另给了门禁侧的具体补丁：「75 在数字与「项」之间加 `\s*`；20 的汉字支改成 `[一二两三四五六七八九十]+`（十一项以上是 73ba4a4 就有的，顺带）」。

改后现文（`.claude/agents/kb-scribe.md:20`，`grep -n` 现取，节选）：「…决策文件标题行的状态改成什么（已定 / 半定（N 项未定）/ 待定（N 项未定），**N 写阿拉伯数字或一到十的汉字数字，20、75 号都认**）；…」

现查两道门禁实际状态：`.claude/gate.d/75-decision-experiment-links.sh:226`（`grep -n` 现取）已改成 `r'^## D\d+ \S.*? —— (已定|半定|待定)(（[0-9一二两三四五六七八九十]+\s*[项条]未定）)?\s*$'`（补了 `\s*`，与丙组第 11 条建议一致）；`.claude/gate.d/20-kb-shape.sh:214`（`grep -n` 现取）仍是 `r'([0-9]+|[一二两三四五六七八九十])\s*[项条]未定'`——**20 号的汉字支没有补 `+`，`git diff 1c58cfa bfc447e -- .claude/gate.d/20-kb-shape.sh` 只改了「历史类文件跳过」那一处注释，没有碰这一行**。

判定：这是一次自洽的组合修复，不是简单的「按丙组第 11 条把两道门禁都补齐」。因为 20 号仍然只认单个汉字数字，`kb-scribe.md` 把书写规范收窄到「一到十的汉字数字」（一到十恰好都是单字符，落在 20 号现在能正确解析的范围内），绕开了 20 号需要打补丁的必要性；75 号则确实按丙组建议打了 `\s*` 补丁。两条门禁一个改代码、一个靠书写规范避开，合起来能让 `report-B.md:11` 举的两个反例（「（3 项未定）」带空格、「（十一项未定）」两字数字）都不再产生：前者被 75 号的 `\s*` 接住，后者被 kb-scribe.md 禁止书写超过十的多字汉字数字接住。这是乙组第 3 条「或把两道门禁的正则对齐」那条出路的一个变体（只对齐了一道、用书写约束堵住另一道的盲区），效果上达成了同一个目标，但不是逐字执行任何一条建议原文。

什么现象会推翻它：决策文件标题里出现「十一项未定」这种需要两个汉字字符的写法（`grep -n '（[一二两三四五六七八九十][一二两三四五六七八九十]*项未定）' .claude/kb/decisions/*.md` 现查零命中，判定站得住）；或者「一到十」范围之外还有别的场景需要多字数字（例如超过十项未定的决策），届时 20 号的盲区会重新暴露，而 `kb-scribe.md` 现在的写法会挡不住书记员被要求写「十一项未定」这种情况（这属于「往后如果出现就会打中」的假设性推翻条件，这一轮实测未出现）。

### 写入后钩子先红的清单（对应乙组第 8 条，`report-B.md:16`）

已在前文按 `.claude/hooks/kb-scribe-followups.tsv` 核实：改后现文（`.claude/agents/kb-scribe.md:32`，`grep -n` 现取）「…红的是这个流程下一步会消掉的（**21 号在第 3 步的 `21-decision-items-sync.sh --write` 之前、30 号在变更史条目写之前**、49 号在 `--write` 之前、75 号在实验页那几行写之前），照流程往下走…」——与乙组第 8 条建议的字面分组（「30 号与 49 号在变更史写完之前」）不同：改后现文把 30 号单独归入「变更史条目写之前」，49 号仍留在原有的「`--write` 之前」，没有按建议把 49 号也挪过去。

核 `.claude/hooks/kb-scribe-followups.tsv`（`grep -n` 现取）：第 6 行对 `.claude/kb/decisions/` 的写入同时触发 21、30 等阶段；第 8 行对 `.claude/kb/decisions-history` 的写入触发 48、**49**、30。49 号（`49-history-brief.sh`）只在写变更史（`.claude/kb/decisions-history/`）之后才会被触发，但 `kb-scribe.md` 第 2 步明写「先跑不带 `--write` 的 `bash .claude/gate.d/49-history-brief.sh`…都是这一轮的才跑 `--write`」——49 号真正转绿要等到带 `--write` 那一次，光「写完变更史」这一步本身并不足以让 49 号绿。**改后现文把 49 号继续挂在「`--write` 之前」是对的，按乙组建议字面把 49 号并入「变更史写完之前」反而会引入一个新的不准确**。

判定：改后现文没有照抄乙组第 8 条的字面分组，但按 `kb-scribe-followups.tsv` 与 `kb-scribe.md` 第 2 步自己的描述现查，改后现文的分组更准确；乙组建议原文在这一处本身有一个小疏漏（把 49 号错并进了「变更史写完之前」）。这不是「改动越出了审查结论」，是改动没有逐字采纳建议、而是采纳了建议的方向（把 21、30 都补进红栏说明）并做了更准确的归类。

什么现象会推翻它：`kb-scribe.md` 第 2 步的描述与实际不符，即 49 号在只写完变更史（不带 `--write`）时就已经转绿——这需要现跑 `49-history-brief.sh` 两次（写变更史后、`--write` 前 vs `--write` 后）比较退出码，这一轮没有可写的决策变更史样本，「核不动」。

### agent-common.md 内存包装退出码（对应甲组 A7，`report-A.md:15`）与 main-agent.md 退出码措辞（对应甲组 A9，`report-A.md:17`，丙组第 1 条，`report-C.md:9`）

已在前文分段核实，均是直接后果：`agent-common.md`（`grep -n '253 是超过限时'` 命中一处）逐字对应甲组 A7 建议；`main-agent.md:59` 与 `54-layer0-replay.sh:230`（`grep -n '退出码是经内存包装的那条'`，两处都命中）把措辞改成「退出码是经内存包装的那条 `--full` 命令的，250–254 是包装自己的结局」，比甲组 A9 建议的「退出码是内存包装的：0–249 就是 `--full` 的，250–254 照 `.claude/agent-common.md` 那张表读」少了「照哪张表读」这个指向，但没有引入新的错误说法；配合丙组第 1 条对 `54-layer0-replay.sh:232-234` 脚本本身的修复（`layer0_tree_ready` 守卫，apply 失败时不再无条件跑 `--full` 而是直接把 `layer0_full_rc` 设成 1），改后措辞与脚本今天的真实行为一致（已现查脚本源码确认守卫逻辑存在）。

什么现象会推翻它：`.claude/agent-common.md` 那张退出码表被进一步改写而 `run-with-memory-cap.sh` 文件头的表没有跟着改（两处出现分歧）——现查 `agent-common.md:46`（`grep -n` 现取）末尾写「退出码表以 `research/scripts/run-with-memory-cap.sh` 文件头为准」，把权威定义指向脚本本身而不是自己重复一份表，不存在两处分歧的可能，判定站得住。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| G1（59 号计数） | 半直接后果 | 「无效」判定条件改法逐字对应乙组第 1 条前半句；但「进程被杀该分进第几类」与「新三栏要不要交主 agent」两处，建议的后半句没有被逐字落实，留白或依赖有歧义的读法 |
| G2（崩溃验证员） | 半直接后果 | 未跟踪检查的「宽的一边」（55 号误挡别的会话产物）修正到位；「窄的一边」里 `.lkmm-static-only` 落实、`lib.sh` 因被 `.gitignore` 挡着而无法用 git 命令落实，改后定义未交代这一点；与之配套的 mutation-triage.md 输入措辞未同步改 |
| G3（核查员） | 基本是直接后果，一处限定词丢失 | 设计轮不再一律「分不清」的死结按建议解开；但第 1 步自证挑选条件比建议原文少了「`sha256sum -c` 通过」这个限定词，多数情形无害、字面不等价 |
| G4（执行员入库装置） | 直接后果 | bin 名连字符化、入库装置编译位置与 `driver_e<号>` 登记均逐字或等效对应乙组第 5、6 条；「写范围」一节文字未同步更新，但闸门本身（按文件路径匹配）已经放行，不构成功能性缺口 |
| G5（其余） | 直接后果，两处为更准确的改写而非逐字照搬 | agent-common.md 退出码、main-agent.md 措辞、54 号脚本三处逐字或等效对应甲组 A7/A9 与丙组第 1 条；kb-scribe.md 的「N 项未定」写法与写入后钩子清单两处，没有逐字照抄乙组第 3、8 条的建议原文，但现查门禁与 followups 表之后，改后的写法比建议原文更准确（分别绕开了 20 号未打补丁的盲区、避免把 49 号错并入变更史步骤） |

## 没做什么

- 不判 G1-G5 之外的格：三份判决问题清单里明确划给云端攻方（Opus）或本地腿的部分不判；本地攻方 / 本地辩方缺席，不替它们补判。
- 不出判决、不替主 agent 采信：本报告只给对应关系与「什么现象会推翻它」，站不站得住由主 agent 逐条现查。
- 没有跑任何重型测试（门禁 54、55、57、59、87、整轮门禁、`gate-staged.sh`），G1 里「测试进程被信号杀时 cargo 是否一定打 `error: test failed`」这一步沿用乙组第 1 条已有的现查证据（`research/scripts/mutate.sh:496`、`research/prompts/governance-defs-r2-opus-model/g3-outputs/crash-before-named-test.txt` 的判定结果），没有重新构造一条 abort 变异去复核；乙组自己也标注这一步是「推的，没量过」的延伸。
- 没有核 G1-G5 未点名、但同一批 diff 里出现的文件（`three-way-inference.md`、`format-evolution.md`、`10-kb-rot.sh`、`lib-governance-refs.py`、`heavy-test-guard.sh`、`runner-dispatch-guard.sh`、`ask-local.sh`、`ask-local-selftest.sh`、`90-term-renames.sh`、fixtures）——这些属于「一起改的规则、skill、钩子、门禁与脚本」，按正文第一节只当背景读，不在「被判的 7 份」之列。
- 没有对每一处引用逐一跑 `quote-kb.py` 之类的抽取工具核对，全部引文改用 `grep -nF`/`grep -n` 现查一次命中并附命令在正文里；没有找到命中的没有写成引文（本报告没有出现这种情况，所有引文均已现查确认命中）。
- kb-scribe.md 第 2 步「49 号是否在只写完变更史时就转绿」与 crash-verifier.md「日志路径」列在真实报告里存不存在，都需要一次真实运行才能核，本报告标「核不动」，不下结论。
