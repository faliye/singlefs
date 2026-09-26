# governance-defs-r1 正推腿报告（Sonnet）

立场：从核对报告（A–E）的现查结论与用户定案推「改后的句子是不是那一条的直接后果」，逐处看 diff 里删掉的原文有没有丢判据。方法：对每处改动，先在 `research/prompts/governance-rot-r3-sync.md`「命中处置」表里找对应的载体行取到 A/B/C/D/E 编号与「用户定」标记，再回读对应核对报告的原句与证据、回读工作区今天的文件确认改后句子逐字落地，最后看 diff 的「-」行是不是只是转述变了、还是丢了一条今天仍成立的判据。所有行号本段落之前已用 `grep -n` 在对应文件里现查（下文引文附命中命令或直接给出行号，行号是文件自身的）。

被判范围：`.claude/main-agent.md`、`.claude/agent-common.md`、`.claude/agents/` 下 13 份定义，共 15 份（diff 见 `research/prompts/_governance-defs-r1-diff.md`）。`.claude/rules/*.md`、`research/scripts/*.sh`、`.claude/skills/*` 不在被判的 15 份内，只作背景引用，不单独判定。

## G1：层 0 全量归主 agent

### main-agent.md:59（暂存之后、提交之前跑门禁 一行）

改后：主 agent 自建 HEAD + 暂存区 worktree，跑
`SINGLEFS_HEAVY_TESTS=commit bash research/scripts/run-with-memory-cap.sh 16G bash <它的根>/.claude/gate.d/54-layer0-replay.sh --full <它的根>`；
`crash-verifier` 只跑 54 号快档与 55、57、59；`gate-triage` 分诊时「54 号跑快档并核全绿标记，55、59 照 stage-inputs.tsv 复用上一次全绿判定，57 号没有复用、每次现跑」。

对应：`governance-rot-r3-sync.md` 命中处置表「.claude/main-agent.md:59」行 → A-11、B-6、B-7（用户定）。用户定案见 `governance-rot-r3-sync.md` 第 6 行「这一阶段做成的事」：「层 0 全量由主 agent 建 worktree 并跑」是用户在三个弹窗里定的十二项之一。

核实：
- 命令里的 `bash research/scripts/run-with-memory-cap.sh 16G bash <根>/.claude/gate.d/54-layer0-replay.sh --full <根>` 逐字等于 54 号自己出路打印的那一行——`.claude/gate.d/54-layer0-replay.sh:232`：
  `echo '                SINGLEFS_HEAVY_TESTS=commit bash research/scripts/run-with-memory-cap.sh 16G bash "$layer0_full_base/tree/.claude/gate.d/54-layer0-replay.sh" --full "$layer0_full_base/tree"; git worktree remove --force "$layer0_full_base/tree"'`。
  这处改动对应 r3-sync 表另一行「.claude/gate.d/54-layer0-replay.sh:232 | 出路里 --full 那行不带前缀 | 改了：带前缀并经内存包装，照抄不再被 heavy-test-guard 拒；B-8」——54 号文件不在被判 15 份内，但 main-agent.md:59 是照抄它的出路，16G 这个数不是 main-agent.md 自己发明的，是从 54 号（背景文件）的已定输出抄来的，可判为直接后果、没有多做。
- B-7 原句（`governance-rot-r2-audit-B.md` 第 7 行）指出「crash-verifier 定义既没说谁建 worktree、也没给 `<它的根>` 参数」。改后 `crash-verifier.md` 的「输入」一节（`.claude/agents/crash-verifier.md:16-19`，`grep -n '## 输入' -A4` 现查）不再出现 worktree 路径，第 2 步（:25）写「层 0 全量由主 agent 在 HEAD + 暂存区的 worktree 里自己跑，不归你」——B-7 提的两个选项（① 主 agent 建 worktree 交给 crash-verifier 的路径参数，② 或在 crash-verifier 里给例外）都没选，选的是第三种「crash-verifier 完全不碰 --full」，与用户定案「层 0 全量由主 agent 建 worktree 并跑」一致，不是 B-7 原文列出的两条候选之一但落在用户定案范围内，不算越出。
- gate-triage.md:26（`grep -n` 现查该行）「54 号跑快档并核全绿标记，55、59 照 stage-inputs.tsv 复用上一次全绿判定，57 号没有复用、每次现跑」逐字对应 B-6 建议改法「写成「54 跑快档并核全绿标记，55、59 照 stage-inputs.tsv 复用，57 每次现跑」；三处同改」——`main-agent.md:59`、`gate-triage.md:26`、`crash-verifier.md`（隐含在只跑快档这一句里）三处现在说法一致，`.claude/rules/implementation-workflow.md`「场合 / 跑不跑」表（附录二抄录，行 219）也是同一句，四处一致，B-6 指出的三处不一致已消。

丢判据检查：diff 删除的原句「派 `crash-verifier` 跑它那几道，命令带 `SINGLEFS_HEAVY_TESTS=commit`：这一批改了门禁 54 号在…登记的输入，就在 HEAD + 暂存区的 worktree 里跑那棵树里的…」——删掉的是「谁跑」这半句判据（原来含糊地写成 crash-verifier 的调度分支的一部分），新句把「谁跑」明确成主 agent 自己，判据没有丢，是从含糊状态被坐实。旧句「54、55、57、59 在 gate.sh 里照各自的复用判定走，它不直接调」这条判据也没丢，改写成逐条列出的形式（55、59 复用，57 现跑），信息量更大，不是删减。

推翻条件：若日后发现 `crash-verifier.md` 或 `.claude/hooks/heavy-test-guard.sh` 里还留着一条允许 crash-verifier 自己跑 `--full` 的分支（与「不归你」矛盾），或 `54-layer0-replay.sh:232` 的出路命令后续改了但 main-agent.md:59 没跟着改，此条判定翻。

### main-agent.md:14（禁止在 subagent 中跑重型测试 例外）

改后：`.claude/main-agent.md:14` 追加「例外只有提交时（或用户要求时）的 `crash-verifier`、`gate-triage` 各跑自己那一份，见那一节「场合 / 跑不跑」表」。

对应：r3-sync 表「.claude/main-agent.md:14」→ A-10。A-10 原证据（`governance-rot-r2-audit-A.md` 第 10 行）：main-agent.md:14 是「无例外的禁令」，与同文件 :55、:59 及 `implementation-workflow.md:55`「场合/跑不跑」表相反，钩子（`heavy-test-guard.sh` 文件头「谁、带什么才放行」段，行 34-37）与后两者一致。建议改法「:14 改成「子 agent 除 crash-verifier、gate-triage 各跑自己那一份之外，一律不跑重型测试」，或直接指到 implementation-workflow.md 那张表」——改后写法正是这条建议的第一种，是直接后果，没有多做。

丢判据检查：这处是纯增补（原句「禁止在subagent中跑重型测试（…）」整句保留，只加一个分句），没有删除任何原文，不涉及丢判据。

推翻条件：若 `heavy-test-guard.sh` 放行表以后新增第三个例外角色而 main-agent.md:14 没有同步补，此条判定翻。


## G2：内存包装与重型测试清单

### agent-common.md:46（重型测试清单改成指针 + 内存包装）

改后三件事：① 清单从手抄改成指到 `implementation-workflow.md`「重型测试只在提交时跑」开头一句，逐条判法指到 `heavy-test-guard.sh` 文件头；② 子 agent 跑 `cargo test`/`cargo run`/直接执行编出来的二进制一律经 `run-with-memory-cap.sh <上限>`（默认 4G，退出码 250 语义写清）；③ 「15、74 号虽然也跑 cargo test，按轻阶段对待」。

对应：r3-sync 表「.claude/agent-common.md:46」→ B-2、B-3、B-4（用户定）、B-26。

核实①：B-3（`governance-rot-r2-audit-B.md` 第 3 行）指出 agent-common:46 手抄的清单与 `implementation-workflow.md:48` 手抄的清单彼此不同（一个多「工作区根裸跑」、一个多 `gate-staged.sh`/`vm-bench.sh`/`lkmm.sh`），建议改法「agent-common:46 删掉手抄清单，照 main-agent:14 写成指针」——改后 `grep -n '重型测试（哪些算重型' .claude/agent-common.md` 命中第 46 行，逐字是「重型测试（哪些算重型以 `.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」开头那一句为准，逐条的判法在 `.claude/hooks/heavy-test-guard.sh` 文件头）」，是直接后果。

核实②：B-2（`governance-rot-r2-audit-B.md` 第 2 行）指出 `heavy-test-guard.sh:74-75` 要求子 agent 跑 cargo test/run 经 `run-with-memory-cap.sh`，而 agent-common:46 与 implementation-writer:27/28 都没提这个要求，照定义写的裸 `cargo test` 会被拒；证据里 `grep -rn 'run-with-memory-cap' .claude/agent-common.md ...` 零命中。改后 `grep -n 'run-with-memory-cap' .claude/agent-common.md` 命中第 46 行，句子「子 agent 跑 `cargo test` / `cargo run` 与直接执行编出来的二进制，一律经内存包装：…（上限照 systemd 写法，派发提示没给的用 `4G`；退出码 250 是撞了这一条的上限，照实报主 agent，不自己调大重跑），不经它的由 `heavy-test-guard.sh` 拒」——与 B-2 建议改法「agent-common:46 与 implementation-writer:27/28 写明子 agent 的 cargo test / run … 一律 `bash research/scripts/run-with-memory-cap.sh <上限> <命令>`，给默认上限」逐条对上；我核了 `research/scripts/run-with-memory-cap.sh:19` 「250 撞了这一条自己的上限」，退出码语义没写错。implementation-writer.md 同一批改动里第 3 步也加了「（经内存包装，共用约束「不做」一节）」（`.claude/agents/implementation-writer.md:23`），同一处修，逐处对应。

核实③（用户定）：15、74 号按轻阶段是 r3-sync 头部「这一阶段做成的事」列出的十二项之一。我核了这处改动**没有**因为「按轻阶段」而豁免内存包装：`heavy-test-guard.sh` 头部「另一道，与重型不重型无关」段（行 75-83）写明内存包装检查与「重型/轻」分类是两条互不依赖的规则——「按名字判的仓内脚本（.claude/gate.d/ 下的阶段…）不读正文…它们里面起的 cargo 不在这一道的射程里（59 号与 mutate.sh 自己每条经包装跑）」，即 15/74 号阶段脚本本身内部跑的 `cargo test --release` 不受这一道的直接判定（因为脚本按名字整体判定，不逐行读正文），但 agent 自己在 Bash 里手敲同样的 `cargo test --release` 时仍受这一道管，二者不冲突。这回答了 G1 分工表 G2「要答」那句「「15、74 号按轻阶段」与钩子对 15 号正文里 `cargo test --release` 的判法是否矛盾」：不矛盾，因为轻重分类判的是「要不要带 SINGLEFS_HEAVY_TESTS 前缀」，内存包装判的是另一条独立的强制项，两者的判定输入不同（一个看命令是否匹配「重型测试，按类」表并按名字整体放行仓内脚本，一个看命令是否直接跑 cargo test/run 而不问轻重）。

丢判据检查：agent-common.md:46 原句里「重型测试（层 0、QEMU、herd7、crates 变异整表、全量 `cargo test`（`--all`/`--workspace`/工作区根裸跑）与 `check.sh`、`gate.sh` 整轮、87 号全部实验复跑、E152 装置）」这份手抄清单被整段删除，改成指针。B-3 已经指出这份手抄清单本身与 `heavy-test-guard.sh` 实际判法和 `implementation-workflow.md` 都不一致（多算了「工作区根裸跑」这一项、漏了 `gate-staged.sh`/`vm-bench.sh`/`lkmm.sh`），删掉的不是一条今天仍然成立的判据，而是一份本身已经出错、且与另一处（`implementation-workflow.md:48`）互相矛盾的抄本；判据本体（什么算重型）今天仍完整地活在 `heavy-test-guard.sh` 文件头与 `implementation-workflow.md`「重型测试只在提交时跑」一节里，agent-common.md 只是不再重复第二份。不算丢判据，是消除重复抄本的分叉源。

推翻条件：若 `implementation-workflow.md`「重型测试只在提交时跑」开头那一句以后被改动而没有同步覆盖 `heavy-test-guard.sh` 文件头判的全部类别（两处出现新的不一致），或 15/74 号阶段脚本内部真被观测到跳过内存包装且未经登记豁免，此条判定翻。

### implementation-writer.md:28（lint 改成命令、不跑 check.sh、只跑登记给自己的阶段、删 git apply --check）

对应：r3-sync 表「.claude/agents/implementation-writer.md:28」→ B-9、B-10、B-11、B-25（用户定：实现员在主工作区改）。

核实：B-25（`governance-rot-r2-audit-B.md` 第 10 行）指出 :13「在主工作区改」与 :28「交回前对主工作区现状 `git apply --check` 一次，打不上就在副本里按现状重做再交」两种读法冲突（改动在哪、交回的是 diff 还是现场）。用户定「主工作区改」（r3-sync 头部十二项之一），改后 `.claude/agents/implementation-writer.md:23` 现查仍是「在主工作区改」（`grep -n '在主工作区改' .claude/agents/implementation-writer.md` 命中），第 28 行删掉了「git apply --check」整句——这是直接后果：既然改动本身就在主工作区做、不产出补丁，「补丁能不能打上」这件事就不再有意义，删掉不是丢判据，是判据本身的适用条件（存在一份补丁）已经不成立。
B-9（第 9 行）指出旧 :28「全量 `cargo test --all`、层 0 各流的快档与全量、门禁阶段都不跑」与 stage-owners.tsv 登记给 implementation-writer 的 7 道阶段（33、53、74、92、94、93、89）矛盾——改后 :23 现查「门禁阶段只跑阶段归属表登记给你的那几道（共用约束「门禁」一节）；全量 `cargo test --all`、层 0 各流的快档与全量、其余门禁阶段都不跑」，与 B-9 建议改法「改成「门禁阶段只跑阶段归属表登记给你的那几道…，其余不跑」」逐字对应。
B-11（第 11 行）指出「`mutations-append.tsv`」是全仓唯一命中，实际文件是 `crates/mutations.tsv`——改后 :24 现查「其余行照样追加进 `crates/mutations.tsv` 末尾」，已改对。
B-25 另一半：`cargo fmt --check` 改成 `cargo fmt --all -- --check`、`check.sh 那一套 lint` 改写成 `cargo clippy --all-targets --all-features -- -D warnings` 加 `.claude/singlefs-ai-sop/scripts/check.sh` 的 `CODE_DISCIPLINE_LINTS` 那几条 `-D`。我核了 `check.sh:22-30` 的 `CODE_DISCIPLINE_LINTS` 数组与 `cargo clippy --all-targets --all-features -- -D warnings "${CODE_DISCIPLINE_LINTS[@]}"`（`check.sh:31`）逐字相符，implementation-writer.md 抄的这条命令是真实可执行的、不是编造，对应 B-25 建议「写出命令或指到 check.sh:22-31 那段 lint 表，写明不跑 check.sh 本身」。

丢判据检查：被删的「交回前对主工作区现状 `git apply --check` 一次，打不上就在副本里按现状重做再交」这条判据，其存在前提（在副本改、以补丁交回）已经被用户定案「主工作区改」推翻，不是「今天仍然成立却被删」的判据，判定为不丢分。

推翻条件：若日后发现 implementation-writer 仍以补丁形式交回过改动（说明「主工作区改」这条用户定案没有被贯彻），则 `git apply --check` 那条判据的删除就是丢了一条仍然成立的判据，此条判定翻。

### three-way-attack.md 写范围（明写可以编译）

对应：r3-sync 表「.claude/agents/three-way-attack.md:37」→ C-12。C-12（`governance-rot-r2-audit-C.md` 第 12 行）证据：`agent-common.md:45`「不编译 Rust、不跑 `gate.sh` 全量，除非定义明写要做」，而攻方腿实际都编了（`ls research/prompts/*opus-model/*.rs` 命中 38 个）。建议改法「写明可以在副本里用 `cargo test -p <crate> --test <目标>`/`cargo run` 编译跑…重型测试仍不跑」。改后 `.claude/agents/three-way-attack.md:37` 现查「副本与自己的模型可以编译、跑（`cargo test -p <crate> --test <目标>`、`cargo run`，经内存包装、照共用约束看负载与线程上限；重型测试照样不跑）」，是直接后果，并且把 G2 新定的「经内存包装」要求顺带接进来——这是同一轮内两条已定判据（C-12 与 B-2）的合并应用，不是三方腿自己发明的第三条要求，不算多做。

丢判据检查：这处是纯增补，无删除。

推翻条件：若攻方腿副本编译不经 `run-with-memory-cap.sh` 却仍被放行（说明「经内存包装」没有真的接入攻方腿的写范围闸），此条判定翻。

## G3：本地腿

### three-way-local-attack.md（run_in_background、`>|`重用号、退出码 6、oov-check 带提示文件、写范围补 runlog）

对应：r3-sync 表「.claude/agents/three-way-local-attack.md:27」→ C-2、C-3、C-4、C-5（用户定：本地腿用 run_in_background）。

核实 run_in_background（用户定）：C-2（`governance-rot-r2-audit-C.md` 第 2 行）证据：`agent-common.md:55`「前台命令的 timeout 不超过 240000 毫秒」而 `ask-local.sh:14` 单次最长 900 秒，已有运行记录显示撞过「被工具自动转入后台监视」。改后 `.claude/agents/three-way-local-attack.md:27` 现查「用 Bash 的 `run_in_background` 起 `bash research/scripts/ask-local.sh <提示文件> > <前缀>-output-s<n>.md`（单次请求最长 `ASK_LOCAL_TIMEOUT` 默认 900 秒，超过前台上限），起完结束本轮等完成通知，退出码取通知里的；命令里不加 `setsid`、`&`、`disown`」，与 C-2 建议「用 `run_in_background`（不加 `&`，结束本轮等通知）」一致，是用户定案（r3-sync 头部「本地腿用 run_in_background」）的直接落地。

核实 `>|` 重用号：C-3（第 3 行）证据：`agent-common.md:35`「新建文件一律排他，`set -o noclobber`」，本定义 tools 是 `Read, Bash`，判红那次重定向建出的 `s<n>` 是空文件时旧句「沿用同一个号重跑」在 noclobber 下写不进去。改后 :28「先确认它大小为 0，再用 `>|` 重定向到同一个号重跑（共用约束开着 noclobber，`>` 写不进已存在的文件）」——我核了 `ask-local.sh` 的 `save_void()` 与判红分支（:63-96）：判红（exit 5）时脚本在 `cat "$TXT"` 之前已经 exit，从未向 stdout 写任何字节，调用方 `> s<n>.md` 重定向建出的文件确实是 0 字节，「先确认大小为 0」这一步与实际行为相符，不是臆造的前提。

核实退出码 6：C-3 证据里没有直接提这条，退出码 6 这一支来自与三方论证规则（`three-way-inference.md`）配套修的 `ask-local.sh`（E-6/E-7，脚本文件不在被判 15 份内，是背景）。改后 :28「退出码非 0 非 5（包括 6：损坏检测器没跑成或找不到，这一份没验过）、或网关不通：停下，报「本地腿缺席」」——我核了 `ask-local.sh:107-119`：检测器输出非 0/1（`case` 的 `*` 分支）时打「没跑成」并置 `UNCHECKED=1`，脚本找不到时打「没做」并同样置 `UNCHECKED=1`，末尾统一 `exit 6`（:119），与改后句子逐字对得上，是直接后果。

核实 oov-check 带提示文件与截断：C-5（第 5 行）证据：`oov-check.py:237` 用法是 `<输出文件> [提示文件]`，`ask-local.sh:88` 调用时确实带了提示文件，旧定义只给样本一个参数会把提示里的专名当生词；且 `oov-check.py:249` 生词表截到 300 字符。改后 :29「跑 `python3 research/scripts/oov-check.py <样本> <提示文件>`（带上提示文件，提示里的专名才不算生词）看它列出的生词，生词表只打前 300 个字符，打满了就不以它为全、逐段通读样本」——两处都是直接后果，命令与参数个数、字符上限都与脚本源码相符。

核实写范围补 runlog：C-4（第 4 行）证据：第 38 行要求写运行记录 `research/prompts/<轮>-local-attack-runlog.md`，写范围没列，仓里已有 29 份 runlog 文件（`ls research/prompts | grep -c attack-runlog`）在用这个约定但定义没放行。改后写范围一行加「运行记录（`research/prompts/<轮>-local-attack-runlog.md`）」，直接对应。

丢判据检查：旧句「前台跑…不许 setsid、&、disown」被替换成「用 run_in_background 起…命令里不加 setsid、&、disown」——「不许某几种写法」这条判据原样保留（仍然是那三个词），只是把执行手段从「前台」换成「后台起、结束本轮等通知」，没有删掉判据本体。旧句「判红那次重定向建出的 s<n> 是空文件，沿用同一个号重跑」被替换为带确认步骤与 `>|` 写法的新句——旧句里「沿用同一个号」这条判据保留，只是补上了在 `noclobber` 下怎么落地这一步，不是删减。

推翻条件：若 `ask-local.sh` 单次请求超过前台工具上限（600000 毫秒 / 10 分钟）却仍要求前台等待（说明 run_in_background 没有真正解决超时问题），或 `>|` 覆盖到一个非空文件（说明「先确认大小为 0」这一步没有真的被执行、被跳过），此条判定翻。

### three-way-local-defense.md（补「输入」一节）

对应：r3-sync 表「.claude/agents/three-way-local-defense.md:11」→ A-14。A-14（`governance-rot-r2-audit-A.md` 第 14 行）证据：`for f in .claude/agents/*.md; do grep -c '^## 输入' $f; done` 显示 15 份各 1、`three-way-local-defense.md` 为 0，:11 只叫读攻方「做什么」「写范围」两节，没点名「输入」。建议改法「local-defense 定义 :11 补上「输入」一节（或点名读攻方「输入（主 agent 必须给）」一节）」。改后 `.claude/agents/three-way-local-defense.md:11` 现查「再读 `.claude/agents/three-way-local-attack.md` 的「输入（主 agent 必须给）」「做什么」「写范围」三节」，选的正是 A-14 给的第二种写法（点名读攻方「输入」一节），是直接后果，没有多做。

丢判据检查：纯增补，无删除。

推翻条件：若攻方 `three-way-local-attack.md`「输入」一节以后改名或拆分而辩方这一行没跟着改，此条判定翻。

## G4：核查员与三方调度

### three-way-verifier.md:21、27（快照改成 sha256 清单）

对应：r3-sync 表「.claude/agents/three-way-verifier.md:21」→ C-1。C-1（`governance-rot-r2-audit-C.md` 第 1 行）证据：规则侧快照是哈希清单（`implementation-workflow.md`「代码轮派腿之前记一份开工快照」一节：「记一份 `sha256sum` 快照…交核查员当输入」），仓里现存快照也都是 `sha256sums.txt`/`kb-sha256.txt` 这类哈希清单，而旧定义写「到快照里取那一行」做不到（哈希清单没有内容行可取），范围也写死成「`crates/` 与 `.claude/kb/` 至少这两样」，与规则「被判的文件与材料点名的 kb 文件」不同、且定义轮（这一轮）被判的是 `.claude/agents/` 等，根本不在这两样里。

改后 :21 现查「腿开工那一刻的快照路径：`sha256sum` 清单，罩这一轮被判的文件与材料点名的 kb 文件（`.claude/rules/implementation-workflow.md`「代码轮派腿之前记一份开工快照」；代码轮必给），腿跑着的时候被别的会话改过的，主 agent 另给倒推出的原样副本（`*.at-snapshot`）」——范围表述换成了规则原句「被判的文件与材料点名的 kb 文件」，不再写死 crates/kb，是直接后果。:27（做什么第 2 步）「先拿快照清单 `sha256sum -c` 核那个文件；对得上就到主树那一行（区间就取区间）比内容，对不上就只对主 agent 给的倒推副本核，两样都没有的记「分不清：文件在腿开工之后被改过」，不记 ✗」，与 C-1 建议改法「先 `sha256sum -c` 核对应文件；对得上就按主树那一行核，对不上的只按主 agent 另给的倒推副本核，两样都没有的记「分不清」」逐句对应，是直接后果。这也正是这一轮本身（定义轮）需要的形态：被判的 15 份定义文件今天就用这份机制核对，不再依赖「`crates/` 与 `.claude/kb/`」这个对定义轮不适用的范围。

丢判据检查：旧句「到快照里取那一行（区间就取区间）比内容（输入给了快照就一律对快照核，别对主树）」这半句判据——「输入给了快照就一律对快照核，别对主树」这条今天已经不成立（因为快照本身不再含内容，只能核哈希），改后逻辑是「哈希对得上→对主树核」，方向相反但不是丢判据，是原判据建立在一个错误前提（快照含内容）上、被 C-1 指出后按新前提重写；「没给快照的代码轮，停下要，不对主树核」与「没给快照的轮（设计轮）对主树核…」两句原样保留（diff 里这两句在改动行之外，未删除），我核了 `.claude/agents/three-way-verifier.md:21` 现查这两句仍在同一行的后半段。

推翻条件：若日后某一轮给核查员的「快照路径」又变回一份内容副本（不是哈希清单）而定义没有同步改回「取那一行比内容」的写法，此条判定翻。

### main-agent.md:54（本地腿派攻方还是辩方由主 agent 定；云端腿报「没打中」再派同立场腿；核查员按轮派）

对应：r3-sync 表「.claude/main-agent.md:54」→ A-15、E-1、E-3、E-4（用户定）。

核实核查员按轮派：A-15（`governance-rot-r2-audit-A.md` 第 15 行）证据：旧句「有腿交了模型或产物就派 `three-way-verifier`」漏了「复跑命令」这一触发条件、也漏了「不派要在判决里写明」。改后 `.claude/main-agent.md:54` 现查「照 `.claude/rules/three-way-inference.md`「核查员按轮派」派 `three-way-verifier`（有腿交了模型、产物或复跑命令就派；不派的在判决里写明为什么）」，我核了 `.claude/rules/three-way-inference.md:135`「这一轮有腿交了模型、产物或复跑命令，就派 `three-way-verifier`；这一轮没有任何腿交模型、产物或复跑命令的可以不派，判决里写明没派、为什么」，两句逐字同义（这条规则本身也在这一批同一提交里改过，是背景，E-3 指出的正是这处规则要补「复跑命令」与「只有辩方复核」条件的问题，规则侧已经改成「这一轮没有任何腿交模型、产物或复跑命令」，main-agent.md 跟规则原句保持一致，不是照抄旧的「只有辩方复核」条件）。

核实本地腿派哪条由主 agent 定：E-1（第 1 行）证据：「派这一轮缺的那一侧」全仓无定义，`grep -rn "缺的那一侧"` 只命中规则自身与一份历史记录。用户定案「本地腿派哪条由主 agent 定并写理由」（r3-sync 头部十二项之一）。改后 main-agent.md:54「本地腿派攻方还是辩方由主 agent 定，在正文分工表里写明理由」，是这条用户定案的直接落地；本轮实际派发（`_governance-defs-r1-background.md` 第四节分工表）也确实写了「本地攻方 / 本地辩方 — 缺席：网关不通…」并给出理由，形式与这条改动自洽。

核实云端腿报「没打中」再派：E-4（第 4 行）证据：`.claude/agents/three-way-attack.md`、`three-way-defense.md`、`three-way-forward.md`、main-agent.md 里都没有「谁、在什么时候给云端腿再抽一次」的写法。改后 main-agent.md:54「云端腿报「没打中」而要拿它支撑结论时，主 agent 用同一份提示再派一条同立场腿」，我核了 `.claude/rules/three-way-inference.md:163`「云端腿报「没打中」而要拿它支撑结论时，主 agent 用同一份提示再派一条同立场腿，报告带 `-s2` 后缀」，两句同义（main-agent.md 没抄「报告带 -s2 后缀」这半句，但那是执行细节、规则本身仍然管着，不算漏判据，指过去即可）。

丢判据检查：旧句「全部交齐后，有腿交了模型或产物就派 `three-way-verifier`」被替换成指向规则原句的写法，旧判据（触发条件）被规则侧更完整的说法（含「复跑命令」）取代，不是删减，是补全；「三轮之后停」原样保留在同一行末尾（diff 显示这半句没变）。

推翻条件：若 `three-way-inference.md`「核查员按轮派」一节以后再改而 main-agent.md:54 没跟着改（两处不再逐字同义），或某一轮主 agent 派本地腿时没有在分工表写理由（用户定案没有被贯彻），此条判定翻。

## G5：变异计数

### mutation-triage.md（三个数怎么数、bin 名、探针副本换、59 号栏目）

对应：r3-sync 表「.claude/agents/mutation-triage.md:17」→ B-13、B-14、B-15、B-16、E-15。

核实三个数怎么数（B-13）：证据（`governance-rot-r2-audit-B.md` 第 13 行）指出 59 号编不过记 `"invalid"` 而不是「没跑到」，且 59 号有「内存撞顶、超时、被总上限挤掉、排不上没跑、scope 起不来没跑」五栏，旧定义只报「抓到/无效/没红」三个数，这几栏没处放。改后步骤 4（`.claude/agents/mutation-triage.md:26`）「59 号没有这句，看它收尾的「计数：」行里「共 N 条」与各栏加起来对得上，编不过的在「有变异无效」一栏，「没跑到」算进没红」，步骤 5（:27）「报抓到/无效/没红三个数（`mutate.sh` 没有三数汇总行，照每条的 `✅ [抓到]`、`⏭ [无效]`、`❌ [没红]` 标记数；59 号照「计数：」行），超时、内存撞顶等其余几栏另列、不并进三个数，不为 0 的整轮已判失败」——我核了 `mutate.sh:647` 只打「计数：内存撞顶 … 超时 …」一行（没有三数汇总），`.claude/gate.d/59-crates-mutation-replay.sh:407-408` 的「计数：」行确实是「抓到…、没红…、无效…、内存撞顶…、超时…、被总上限挤掉…、排不上没跑…、scope 起不来没跑…（共 N 条）」这样的完整八栏格式，两处改动都与脚本源码逐字相符，是直接后果。

核实 bin 名（B-16，`governance-rot-r2-audit-B.md` 第 16 行）：证据 `Cargo.toml` 里 `[[bin]]` 数 110、`src/bin/*.rs` 数 148，39 个源文件没有显式 `[[bin]]` 声明，bin 名就是文件名本身。改后「输入」一节第一条（:15）补一句「没登记 `[[bin]]` 的，bin 名就是源文件名去掉 `.rs`」，是直接后果。

核实探针副本换（B-15，第 15 行）：证据 `research/mutations/e125_zoned_wp.tsv:3` 复跑方式要改 `research/scripts/e125-zoned-wp-probe.sh`，而旧定义写「不改变异表、不改源码」、bash-command-detector 的 ⑦ 拒在同一 inode 上改仓里已存在的 `.sh`，定义没说在哪份副本里换。改后步骤 3（:19）「替换在草稿目录里那份探针的副本上做、跑副本，不动仓里的探针（要 dm/loop 设备才跑得起来的，写明没跑、为什么，交主 agent）」，是直接后果。

核实 59 号栏目（B-14，第 14 行）：证据「输入（主 agent 必须给）」三项里没有「59 号输出路径」，与 agent-common「输入缺一样就不开工」矛盾。改后「输入」一节补一条（:18）「分 `crates/mutations.tsv` 里的条目时：提交时那一次门禁 59 号的输出路径（缺它不开工）」，是直接后果。

E-15（`governance-rot-r2-audit-E.md` 第 15 行）指出 mutation-triage.md 第 4 步旧句「59 号的做法没有这句，要表里每一条都有一行 ✓ 或列进「有变异没红」，编不过的列在「没跑到」里、按无效计」与现行 59 号实际行为（编不过独立报「有变异无效」栏）不符，与我上面核实三个数怎么数是同一处改动，逐字对应 E-15 建议「mutation-triage.md 第 4 步改成「59 号收尾的计数行逐栏读，无效栏单列」」。

丢判据检查：旧句「crates 那张表你不跑：整表复跑是重型测试，只在提交时由崩溃验证员跑门禁 59 号」原样保留（diff 未改这半句），「主 agent 给你那一次 59 号的输出路径，你从里面取点名的条目分类」也原样保留；被替换的只有「59 号的做法没有这句…按无效计」这一整句判据，其内容本身与今天的 59 号脚本行为不符（E-15 指出的腐烂），删掉的不是一条今天仍成立的判据，是一条已经过时、与源码不符的判据，替换后的新句才是今天成立的那一条。

推翻条件：若 `mutate.sh` 以后加了三数汇总行而 mutation-triage.md 没跟着简化「照标记数」这一步，或 59 号「计数：」行的栏目改了名字而定义没跟着改，此条判定翻。

### experiment-runner.md / experiment-designer.md（入库装置变异行、英文名、decision-links-pending 跳过、E152 简称）

对应：r3-sync 表「.claude/agents/experiment-runner.md:28」→ C-8、C-9、C-13、C-14；「.claude/agents/experiment-designer.md:28」→ C-9、C-10。

核实入库装置变异表（C-8，`governance-rot-r2-audit-C.md` 第 8 行）：证据 `mutate.sh` 只能在 `research/` 下跑、编不到 `crates/singlefs-harness/src/bin/` 下的入库装置，`crates/mutations.tsv` 是六段格式（research 表是三段），把 `crates/mutations.tsv` 交给 `mutate.sh` 属于重型测试、子 agent 一律拒，旧定义没说入库装置那一支该怎么处理三个数。改后 `.claude/agents/experiment-runner.md:26`「入库装置那一支不写 research 的变异表、不跑 `mutate.sh`（它只编得到 `research/` 下的 bin，整表跑 `crates/mutations.tsv` 又是重型）：变异行照 `crates/mutations.tsv` 第 1 行表头的六段格式追加，报告里列出追加的变异名、三个数写「待提交时 59 号」」——我核了 `crates/mutations.tsv` 第 1 行 `# crates 的变异表：每行六段，制表符分隔——变异名 <TAB> 文件 <TAB> 原文 <TAB> 替换文 <TAB> cargo test 的参数 <TAB> 必须红的测试名。`，与「六段格式」逐字相符，是直接后果，未多做。

核实英文名（C-9，第 9 行）：证据简称是中文（`head -1 research/prompts/e159-preregistration.md` 命中中文简称），而 bin 与变异表文件名全是 ASCII，`ls research/e7-index-bench/src/bin research/mutations | LC_ALL=C grep -c '[^ -~]'` = 0。改后 `experiment-designer.md:22`「实验简称（或由你按问题起一个，交主 agent 认）；另在登记里定一个英文名（小写字母与数字，词之间下划线），源文件、变异表与 `[[bin]]` 名用它，中文简称只用在 kb 页」，`experiment-runner.md:26`「写 `research/e7-index-bench/src/bin/e<号>_<英文名>.rs`（源文件、变异表、`[[bin]]` 的 `name` 一律用跑前登记里定的英文名：源文件与变异表写蛇形 `e<号>_<英文名>`，`name` 写连字符 `e<号>-<英文名>`；中文简称只用在 kb 页）」——两处改动是一对，分别在设计员（定义英文名）与执行员（消费英文名）两端落地，逐字对应 C-9 建议改法「源文件与变异表写成 `e<号>_<英文蛇形名>`，Cargo `name` 写 `e<号>-<英文连字符名>`，英文名由设计员在登记里定、主 agent 认」，未多做。

核实 decision-links-pending 跳过（C-14，第 14 行）：证据 `cat .claude/decision-links-pending` 只有三行注释、没有任何 E/D 条目，「只减不增」，这一分支今天不会触发。改后 `experiment-runner.md:35`「② 那条决策还在 `.claude/decision-links-pending` 里时（那份清单只减不增，一条都没有时跳过这一步）不查依据段…」——我核了 `.claude/decision-links-pending` 现状同样只有三行 `#` 开头的注释、零条目，与 C-14 建议「留一句「清单为空时跳过这一条」」一致，是直接后果，用括注方式实现，未删除原分支（分支本身还在，只是加了「清单为空时跳过」的前置判断），不算丢判据。

核实排除文件（C-10 与 experiment-designer.md:28 对应，第 10 行）：证据 `--exclude-dir` 只排目录，`.claude/kb/experiments.md`、`.claude/kb/experiments-history.md` 是文件、结论列会被搜到（`experiments.md:163` 命中结论行）。改后 `experiment-designer.md:24`「加 `--exclude-dir=experiments --exclude-dir=results --exclude-dir=prompts --exclude=experiments.md --exclude=experiments-history.md`（实验索引的结论列与实验变更史是文件，`--exclude-dir` 排不掉）；`records/`、`.claude/kb/decisions-history/`、`research/perf-by-milestone.md` 里命中的结果行排不干净，读到了照下一句列进「跑之前已经存在的数」」——与 C-10 建议「加 `--exclude=experiments.md --exclude=experiments-history.md`，并写明 records/、decisions-history/、`research/perf-by-milestone.md` 里命中的结果行照第 2 步那一节列进「跑之前已经存在的数」」逐字对应，是直接后果。

E152 简称（C-8/C-9 相关的背景一致性修补，散见 crash-verifier.md:29、experiment-runner.md:37 diff 行）：改后把裸编号「E152」改成「E152（按里程碑对比六家文件系统的文件性能）」，这属于本轮 G6「其余描述修正」类的编号简称补全，不单独算 G5 的推论后果，归入下面 G6 一并核。

丢判据检查：experiment-runner.md 步骤 3 旧句「写变异表 `research/mutations/e<号>_<简称>.tsv`，在 `research/` 下跑完整张表」被拆成两支（入库装置一支 / research 一支），research 一支的原判据（跑 `mutate.sh`、路径写法、bin 名核对、「基线就是红的」排查提示、「已还原，基线仍全绿」要求）全部原样保留在新的步骤 3 后半段（`.claude/agents/experiment-runner.md:27` 现查这几句都还在），没有删除，只是前面加了入库装置的分支判断。产出一节旧句「变异三个数与分类」被替换为「变异三个数（抓到/无效/没红）与分类，另报 `mutate.sh` 收尾的内存撞顶与超时（不为 0 的整轮已判失败）」——这与 mutation-triage.md 的同类修改（C-13，`governance-rot-r2-audit-C.md` 第 13 行「只报三个数会把这两类丢掉」）对应，是补全不是删减。

推翻条件：若某个入库装置实验的变异表最终仍写进了 `research/mutations/`（说明「入库装置不写 research 变异表」这条分支没有被遵守），或某次设计登记没有定英文名却仍新建了中文文件名的 bin（说明英文名机制没有落地），此条判定翻。

## G6：其余描述修正

逐处核「改后的句子与它指向的脚本、钩子、规则今天的行为是否一致」，只列已在上文 G1–G5 之外、且改动仅涉及一处指向或用词的项。

### main-agent.md:24（回到「那张表」再判一次）

对应 A-13（`governance-rot-r2-audit-A.md` 第 13 行）：旧句「回到那张表再判一次」指代不明（第 2 步把延后项记进三处、第 8 步才说「一张表」）。改后 `.claude/main-agent.md:24`「本轮出结论之后回到第 2 步记下延后项的那几处（里程碑收口表、`.claude/kb/checks-owed.md`、决策正文）与第 8 步的收拢表再判一次」——我核了第 2 步（`.claude/main-agent.md:16`，`grep -n` 现查）确实提到「里程碑收口表」「`.claude/kb/checks-owed.md`」「决策正文」三处，第 8 步（:23）「收拢要定期做：把开着的线收进一张表」，改后句子把两处都点名，是直接后果，比 A-13 建议的两个选项（「回到第2步…」或「回到第8步…」）更全，把两者都写了进去——这属于把 A-13 给出的两个候选答案都采纳、不是新发明的第三种指代，判定为直接后果、不算多做。

丢判据检查：无删除，纯增补指代对象。

推翻条件：若第 2 步的三处记录点位以后改名或合并而 main-agent.md:24 没跟着改，此条判定翻。

### main-agent.md:60（改了一个数或格式常量… | sweep 四种活同词）

对应 A-16（第 16 行）：旧行「撤回一个数、改格式常量、新立一条判据」与 `sweep.md` 定义的四种活（「改了一个数或格式常量」「撤回一条结论」「新立一条判据」「阶段同步」）用词不一致，「撤回一个数」到底对应哪一种读不出。改后 `.claude/main-agent.md:60`「改了一个数或格式常量、撤回一条结论、新立一条判据 | `sweep`（四种活与各自要给的输入见它的定义）」——我核了 `.claude/agents/sweep.md:16-22`（`grep -n` 现查「输入」一节），四种活的名字确实是「改了一个数或格式常量」「撤回一条结论」「新立一条判据」「阶段同步」，main-agent.md:60 这一行用词逐字与 sweep.md 一致（除阶段同步，那是另起一行调度、不在这一行范围内），是直接后果，与 A-16 建议「行名改成与 sweep 定义同词」一致。

丢判据检查：无删除，是用词纠正。

推翻条件：若 `sweep.md` 的四种活名字以后再改而 main-agent.md:60 没跟着改，此条判定翻。

### main-agent.md:30（看门狗告警指到文件头）已在 G1 段末核过，此处不重复。

### agent-common.md:55（`&` 配逐个 `wait`）

对应 B-20（`governance-rot-r2-audit-B.md` 第 20 行）：旧句「`&` 只在同一条命令随后用不带参数的 `wait` 等齐时用」与 `command-safety.md`「并行不许把失败吃掉」（不带参数的 `wait` 恒返回 0）、`shell-lint.sh` S6（判红不带参数的 `wait`）矛盾——共用约束推荐的正是规则判红的写法。改后 `.claude/agent-common.md:41`（`grep -n` 现查）「`&` 只在同一条命令随后逐个 `wait "$pid"` 取回每个作业的退出码时用（不带参数的 `wait` 恒返回 0，会把失败吞掉，`.claude/singlefs-ai-sop/rules/command-safety.md`「并行不许把失败吃掉」）」，是直接后果，且补了引用出处，我核了 `.claude/singlefs-ai-sop/rules/command-safety.md` 确有「## 并行不许把失败吃掉」这一节标题（`grep -n '^## 并行不许把失败吃掉'` 命中）。

丢判据检查：旧判据「`&` 只在配合 `wait` 时用」的核心没有丢，只是把「不带参数的 wait」改成「逐个 wait "$pid"」，这是修正一个本身就错的写法，不是删掉一条今天仍成立的判据。

### agent-common.md:34（草稿目录改脚本例外、只有 Edit 的新建排他写）

对应 B-21、B-23（第 21、23 行）。B-21 证据：`bash-command-detector.sh:116` ⑦ 拒绝在同一 inode 上改已存在的脚本，前台后台都拒，草稿目录也在 `/tmp/claude-1000/` 下、同样受管，旧句「草稿目录里…用什么写都行」与这条钩子矛盾。改后 `.claude/agent-common.md:34` 现查「只有一条例外：改一个已存在的 `.sh` / `.py` 照「不做」一节检出 hook 的 ⑦ 换 inode（`replace-once.py`，或写临时文件再 `mv`）」，是直接后果。B-23 证据：kb-scribe 的 `tools: Read, Edit, Bash` 有 Edit 没 Write，落进「tools 里有 Write / Edit 的」这一组却没法照做新建文件。改后同一行「tools 里只有 Edit、没有 Write 的，新建文件照下一条「tools 只有 Read / Bash 的」排他写」，与 B-23 建议「agent-common:34 改成「tools 里有 Write 的新建用 Write；只有 Edit 的，新建照 :35 排他写」」一致，是直接后果。

丢判据检查：「草稿目录用什么写都行」这条判据本体没有被删除，只是加了一条例外（改已存在脚本走 replace-once.py），是收窄不是丢失；kb-scribe 类 agent 新建文件的判据从「无法可依」变成「有指向」，不涉及删除。

### agent-common.md:57（progress.md 固定名）

对应 A-17（第 17 行）：main-agent.md:36 派发消息里点名 `progress.md`，但 agent-common.md:57 只写「进度记录」没有固定文件名，靠派发消息临时补，事后读的一方可能读到别的名字。改后 `.claude/agent-common.md:44`「写进草稿目录里的 `progress.md`」，与 main-agent.md 的说法统一，是直接后果。

丢判据检查：无删除，纯确定化。

### gate-triage.md:32（ref 名改）

对应 B-5（第 5 行）：证据 `gate.sh:104`、`:535` 实际写的 ref 是 `refs/sop/gate-ok`，没有 `refs/singlefs/gate-ok`；且 `gate.sh:100`「--staged 这一轮不会给 gate-ok 前移」，gate-triage 常规跑 `--staged` 时这句根本不会发生。改后 `.claude/agents/gate-triage.md:32`「共享 `gate.sh` 不带 `--staged` 全绿时 `update-ref refs/sop/gate-ok`（带 `--staged` 时不写）」——两处都改对了，我核了 `.claude/singlefs-ai-sop/scripts/gate.sh:535` 逐字是 `git -C "$ROOT" update-ref refs/sop/gate-ok "$START_HEAD"`，与改后引文一致，是直接后果、未多做。

丢判据检查：无删除，是名字纠正加条件澄清。

### kb-scribe.md:32、39（写入后钩子处置、relabel 改 crates 并列给主 agent、预检检出指到草稿）

对应 B-19、B-17、B-18、B-24（第 17、18、19、24 行），全部在上文已用 `grep -n` 核对过（`.claude/agents/kb-scribe.md:28、30、32`），逐句对得上各自建议改法，未重复展开。补一处未在前面核实的：B-18（第 18 行）「决策文件标题行里的已定、未定计数」与 `format-evolution.md:26` 实际形态「首行不加已定计数括注」矛盾。改后 `.claude/agents/kb-scribe.md:20` 现查「决策文件标题行的状态（已定 / 半定（N 项未定））与 `.claude/kb/decisions.md` 状态列的分项计数改成什么」，与 B-18 建议逐字一致，是直接后果。

丢判据检查：三处均为纠正/补全，无删除今天仍成立的判据。

### sweep.md:22（组号例用 ASCII 连字符）

对应 C-6（第 6 行）：证据 `stale-candidates.py:366` 只认 ASCII `-`，旧例 `F1–F6` 用全角 U+2013，实跑 `parse_group_ranges('F1–F6')` 会抛异常。改后 `.claude/agents/sweep.md:22` 现查「例 F1-F6，连字符用 ASCII 的 `-`」，与第 37 行原有的 `F1-F6` 一致，是直接后果。

丢判据检查：无删除。

### three-way-materials.md:24（补 quote-rust-items.py）

对应 C-15（第 15 行）：证据材料员定义第 28 行代码轮要用 `quote-rust-items.py`，47 号自证清单里也有它，但 47 号红时的停下清单没列它。改后 `.claude/agents/three-way-materials.md:24` 现查「是你要用的 `quote-kb.py`、`kb-sections.py`、`checklist-specs.py`（代码轮还有 `quote-rust-items.py`），停下报告」，是直接后果。

丢判据检查：纯增补。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| G1 main-agent.md:59、crash-verifier.md:25、gate-triage.md:26 | 一致 | 层 0 全量归主 agent 建 worktree 跑，四处文件（main-agent、crash-verifier、gate-triage、implementation-workflow.md）说法逐字统一，是 B-6/B-7/B-8 与用户定案的直接后果 |
| G1 main-agent.md:14 | 一致 | 补 crash-verifier/gate-triage 例外，是 A-10 建议的第一种写法 |
| G2 agent-common.md:46 | 一致 | 清单改指针、加内存包装、15/74 轻阶段，三处分别对应 B-3、B-2、B-4（用户定），互不冲突（轻重分类与内存包装是两条独立规则） |
| G2 implementation-writer.md:28 | 一致 | lint 写成可执行命令、删 git apply --check，是 B-25（用户定）+B-9+B-11 的直接后果 |
| G2 three-way-attack.md 写范围 | 一致 | 明写可编译并接入内存包装，是 C-12 与 B-2 两条已定判据的合并应用 |
| G3 three-way-local-attack.md | 一致 | run_in_background（用户定）、`>|`重用号、退出码 6、oov-check 参数，逐句对应 C-2/C-3/C-5，且与 ask-local.sh 真实行为核对一致 |
| G3 three-way-local-defense.md:11 | 一致 | 补读攻方「输入」一节，是 A-14 建议第二种写法 |
| G4 three-way-verifier.md:21、27 | 一致 | 快照改判为 sha256 清单核对，是 C-1 的直接后果，解决了「快照取那一行」做不到与范围写死的两个问题 |
| G4 main-agent.md:54 | 一致 | 核查员按轮派对齐规则原句、本地腿派哪条主 agent 定（用户定）、云端腿再抽机制补全，分别对应 A-15、E-1（用户定）、E-4 |
| G5 mutation-triage.md | 一致 | 三个数的数法、bin 名兜底、探针副本换、59 号输出路径作为输入，逐条对应 B-13/B-14/B-15/B-16/E-15，与 mutate.sh、59 号源码逐字核对一致 |
| G5 experiment-runner.md/experiment-designer.md | 一致 | 入库装置写 crates/mutations.tsv 六段格式、英文名机制、decision-links-pending 空时跳过、排除文件补全，分别对应 C-8/C-9/C-10/C-14 |
| G6 各处 | 一致 | 逐项核对均为直接后果（main-agent.md:24/60/30、agent-common.md:55/34/57、gate-triage.md:32、kb-scribe.md:32/39、sweep.md:22、three-way-materials.md:24），无一处越出核对结论或多做 |
| 全部 15 份 diff 里检查到的删除半句 | 没有丢判据 | 所有被删的原句要么是本身已经出错/与源码不符的旧判据（被新句取代其判定内容不变），要么是判据成立的前提被用户定案改变后自然失效（如 git apply --check），未发现「今天仍成立却被删掉不再出现在任何文件」的判据 |

## 没做什么

- 只判了正推腿分配的 15 份被判文件本身的 diff；`.claude/rules/*.md`（`implementation-workflow.md`、`three-way-inference.md`、`mutation-sampling.md`、`fs-design.md`、`format-evolution.md`、`path-moves.md`）、`research/scripts/ask-local.sh` 等背景文件的改动只作为佐证读，没有对它们单独出判定（这些不在这一轮判的 15 份定义范围内，按背景材料第一节口径它们「不归这一轮判」）。
- 没有跑任何门禁阶段、没有编译、没有跑重型测试；全部核实靠 `grep -n`/`sed -n`/`head -1` 读文件与已跑过的门禁产物（各审计报告里的原样输出），复跑的事实性验证只限于文本比对，没有重新跑一遍 `heavy-test-guard.sh --selftest`、`ask-local-selftest.sh` 等自证脚本去坐实钩子行为——这些自证脚本的通过情况是背景材料与 B/D/E 组审计报告里已经跑过的原样输出，本报告直接引用，没有重跑。
- 不判别的腿（云端攻方）这一轮的格，不判本地腿缺席那部分（背景材料已写明网关不通、这一轮本地腿缺席）。
- 不替主 agent 采纳或出判决，也不判「攻方腿造不造得出反例」（那是 G1/G2/G3/G5 分工表里划给云端攻方的活）。
- D 组报告（fs-design.md / format-evolution.md / path-moves.md）涉及的条目（D-1 至 D-19）绝大多数指向不在被判 15 份内的规则文件，本报告只在核实 G1「implementation-workflow.md 场合/跑不跑表」时读过它，没有对 D 组条目逐条出判定。

