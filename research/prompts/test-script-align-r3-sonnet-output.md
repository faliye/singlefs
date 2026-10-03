# test-script-align-r3 正推腿报告（V2、V3）（2026-10-01）

正文 `research/prompts/_test-script-align-r3-body.md`；背景材料 `research/prompts/_test-script-align-r3-background.md`。立场：正推。本报告只写观测、候选定义与翻面观测，不写倾向，不替主 agent 采纳。方案按 `crates/` 今天的实现谈；`crates/` 这一轮不判（第 2 批四份补丁仍在 `/tmp/claude-1000/impl-*/patch/`，没合入）。

探针与复跑脚本放 `research/prompts/test-script-align-r3-sonnet-model/`（`rerun.sh`、`SHA256SUMS`）；草稿与原始输出放 `/tmp/claude-1000/test-script-align-r3-sonnet/`（不入库，报告里贴的输出都是这一次真跑出来的，路径在各节写明）。

一律只读：没有编译、没有跑 checker 档、没有改仓里任何文件。跑过的自证全是 `.claude/hooks/selftest-registry.tsv` 登记为 light 的，经 `nice -n 19 bash research/scripts/capped.sh 8` 起；跑前 `ps` 看到别的会话的 `cargo build`、`cargo test`、`cargo run`（不是性能测量，没有 qemu-system、vm-bench、e152、fio），照常跑。

## 一、V2：未定项与欠账清单的完整性

### 1.1 照上游逐节推，对照欠账清单

对象是上游 `.claude/singlefs-ai-sop/rules/test-script-discipline.md`。「判定」一栏：记了 = 欠账里有对应的一笔；有检查 = 本仓已有门禁或脚本在判；漏记 = 既没有检查、欠账里也没有；review = 上游自己把它划给 review。

| 上游节与要求 | 本仓该有的检查 | 今天的现状 | 判定 |
|---|---|---|---|
| 用例原子化：一条用例一个场景 | 每个包的用例都判「按参数循环、每一轮重建被测状态」 | one-scenario 格只扫 harness 包（见 1.3 L1） | 漏记（checker 档） |
| 用例原子化：不读别的用例留下的状态 | 上游划给 review（第 96 行「靠 review 的」那一段） | `grep -rn 'env::set_var\|env::remove_var' crates --include=*.rs` 在 checker 档的 tests 与 src 里有命中（`crash_amplification_fixed_script_through_*.rs`、`common_crash_points/mod.rs`、`gpu_unit_checks.rs`） | review，不记 |
| 用例原子化：随机输入带种子 | 四个随机二进制判红打种子、凭种子复现 | C607 | 记了 |
| 用例原子化：用例名不带来历 | 文件名、文件头、用例函数名三处都判 | 文件名有门禁（test-file-names 格）；文件头 C610；用例函数名无（见 1.3 L2） | 漏记（函数名） |
| 用例原子化：红了只重跑红的 | 入口脚本取出红的项、条数与汇总对不上判红 | `research/scripts/rerun-failed-tests.py`；`research/scripts/crash-amplification.sh:848` 与 `.claude/gate.d/checker-tier-layer0-replay.sh:848` 的出路都带只重跑判红项的命令；`broad-mutation.py` 靠续接（有定论的不重跑） | 有检查 |
| 入口脚本：参数约定、`--list-items`、项名写错退 2 | 每个入口脚本都核 | 门禁阶段由 gate-structure 格核；研究脚本只靠各自自证；`research/scripts/replay.sh` 既无 `--list-items`、`--item`，也无 `--selftest`（见 1.3 L5） | 漏记（replay.sh） |
| 入口脚本：每项结束一行（项名、退出码、判了多少、红了多少、复用了多少、秒）与总计 | 每个入口脚本的行都按这一格式 | 只有 `crash-amplification.sh` 做过（G3）；`broad-mutation.py` 的行没有判了多少、红了多少、复用了多少（见 1.3 L5） | 漏记（别的入口） |
| 入口脚本：规模变量写进产物、全量标记不认缩过规模 | 每个读规模变量的入口 | C606（崩溃放量）；直接跑 cargo 的探索档（`crash_injection_large_tier_from_the_environment`）本来就不是全量，没有登记成 `crash-case:` | 记了 |
| 入口脚本：整个脚本离不开的写 run-condition，不满足退 78 | 同 | H4 已落；单机清场退 78 这一条只在调度记录里（见 1.3 L6⑤） | 有检查；尾巴没记 |
| 自证：每一种判定各喂一个已知输入 | 自证覆盖 | 上游划给 review | review，不记 |
| 自证：每处会判红的分支配弄坏开关，开关在文件头逐个列明，复跑方逐个打开 | 文件头三样 + 复跑方 | 研究脚本与钩子：C625、C622；门禁阶段：没有复跑方（见 1.3 L3，与 C625 一条的描述不符） | 记了一半，漏一半 |
| 自证：接进门禁或快档，自证用假驱动、不起编译与重活 | 每份自证在门禁里有人跑；起重活的写明为什么 | research-script-selftest-coverage 格管「有人跑」；起真重活的 4 份自证（1.3 L4）与「用假驱动」不合，只有两份在 `NOT_RUN_HERE` 里写了挂钟理由 | 漏记（偏离没登记成偏离） |
| 分快慢两档：档位标在用例上或登记表里；一律进快档的几类 | 逐类核 | C624 | 记了 |
| 分快慢两档：用编译开关分档，快档至少一趟编进来，编进 0 条报本次未跑 | 同 | C604（门禁一半已落、crates 一半欠） | 记了 |
| 判定可复用：键罩全输入、剪枝自证、按前缀共享 | 关掉复用与剪枝对照 | C605；复用数取不出的棘轮 C623 | 记了；C605 补的一个假定没落到行里（1.3 L6③） |
| 编译与算法都要快：release 开溢出检查与断言；进度间隔 | 构建配置；每过间隔打一行 | `debug-assertions` 用户定直接开；C611 | 记了 |
| 硬件加速：登记哪几类走加速、没卡报本次未跑、没有悄悄退回 | 登记表、没卡判法 | C626、C620、C608、C604 | 记了 |
| 全量的红钉回快档：钉表整行抄、产物里每个红都在钉表里 | 每道慢档一张钉表 | C609、C627 | 记了 |
| 门禁管哪一半：「按参数循环的用例拆没拆开」 | 项目可接的一格 | 同 L1 | 漏记（checker 档） |

### 1.2 三处写回员留成「未定」的：候选与翻面观测

只写问题、候选定义与「什么观测会让选择翻面」，不写倾向。「零轮」= 没被攻过。

#### 1.2.1 C623：棘轮靠常量还是现读

问题：驱动行补 `states_reused=` 的补丁合入之后，入口怎么知道「这一趟该带」，从而把取不出复用数的项判红。欠账原文写的两条路：`.claude/kb/checks-owed.md:492`「入口里一个常量在补丁合入那一次提交里翻开，或入口现读 `crates/` 里有没有 `states_reused` 这个串（今天 0 处），取哪一种未定」。今天 `grep -rn states_reused crates | wc -l` 输出 0（`research/prompts/test-script-align-r3-sonnet-model/v3-counts.sh` 里那一段，输出在 `/tmp/claude-1000/test-script-align-r3-sonnet/v3-counts.out`）。

候选定义：

- K（常量）：入口里登记一个常量，形态与全量展开深度那一个相同（`research/scripts/crash-amplification.sh:128` 是 `FULL_EXPAND_UP_TO=28`）；今天值为关；补丁合入那一次提交里翻开；开着时登记表里有复用层的项取不出复用数判红。登记表里每一项都有复用层：`research/scripts/crash-amplification.sh:59` 写「登记表里每一项都走 run_from_environment 的判定库，都有判定复用这一层」。
- R（现读）：入口开跑时读 `crates/` 里有没有 `states_reused` 这个串，有就按开着判。R 里还有一个没定的参数：读的范围，三种定义——整个 `crates/`、只读发射点所在的 `crates/singlefs-checker-tier/src/crash_amplification.rs`、只认发射行的格式串（补丁里是 `" states_reused={} exhaustive={} expand_up_to={} …"` 那一句）。
- 本腿新提的变体（零轮）：按项登记。`ITEMS` 每行加一列「这一项的驱动是否报 `states_reused=`」，按项翻开。只有出现一项没有复用层（上游「每项结束打一行」那一节写的「不复用」）时它才与 K 分开。

翻面观测（每条给得出的命令或输入）：

1. R 的命中面。补丁 `/tmp/claude-1000/impl-crash-amp-lines/patch/crates.patch`（`sha256sum` 是 f8170b37ad74f8147377a2cac7f1a2899a5adb8c95d36fd01bcfe4d111efee79；`/tmp` 路径只用来指补丁在哪，补丁没合入、数是读补丁数的）里 `states_reused` 出现在三处：`src/crash_amplification.rs` 10 行（字段、文档注释、格式串、取法）、`tests/crash_amplification_pipeline_records_checks_and_reuses.rs` 5 行、`mutations-append.tsv` 1 行（`awk '/^\+\+\+ b\//{f=$2} /states_reused/{c[f]++} END{for(k in c) print c[k], k}' crates.patch` 与 `grep -c states_reused mutations-append.tsv`）。合入后整个 `crates/` 命中 3 个文件、只有 1 个是发射点。读范围取整个 `crates/` 时，「有这个串」与「驱动会发」之间有 2 个文件的差；取发射行格式串时差为 0，但棘轮与那一个文件名、那一句格式串绑在一起（改名时棘轮静默关）。这条观测量的是：合入之后真能造出「串在、驱动不发」或「驱动发、串不在」的改动吗（改格式串留测试与变异行、改字段名不改格式串各试一次，看棘轮开没开、红没红）。
2. 入口读不读源码。`grep -n 'crates/' research/scripts/crash-amplification.sh` 今天只命中注释（第 15、64 行），代码里 0 处读 `crates/` 的源码。R 是入口第一次读源码。入口自证用假驱动与假清场（`CRASH_AMPLIFICATION_ITEM_DRIVER`、`CRASH_AMPLIFICATION_QUIESCE_STUB`），那几种跑法里 R 读哪一棵树没定；双机时第二台的树靠 `research/scripts/peer-host-lib.sh:67`（`peer_host_copy() { # <rsync 的参数…>：rsync -a（不带 --delete）`）同步。观测：第二台上的 `git rev-parse HEAD` 与本机是否相同；不同时 R 读的是本机的串，对端跑的驱动不一定是同一份。
3. K 的翻法。补丁的 `+++` 行 7 个文件全在 `crates/` 下，入口在 `research/scripts/` 下：K 是第二处手改。观测：合入那一次提交的暂存区里两处是否同在；不同在的窗口里棘轮关着、行里全 `reused=unknown`、退 0——就是今天的状态。要让门禁核「常量与 crates 同步」，那一格读 `crates/` 的办法就是 R 的某一种定义。

#### 1.2.2 C625：文件头写法取哪种

问题：「文件头弄坏开关表三样写全（开关名、弄坏哪一处、打开后自证哪一项该红）」要有一种机器能解析的写法。欠账原文：`.claude/kb/checks-owed.md:494`「先定一种能解析的文件头写法，…才有输入：今天至少三种」。

候选定义（都按今天仓里已有的写法取）：

- A（`research/scripts/crash-amplification.sh` 写法）：每开关一行，三段——开关名、弄坏哪一处、第 N 格；格号是 `run_selftest` 里的序号，判错的那一格打「✗ 自证第 <号> 格」。表头一句：`research/scripts/crash-amplification.sh:87`「一行一个：开关名、弄坏哪一处、打开后 --selftest 哪几格该红」。已经被机器读：自证末尾「开关轮」从这张表读开关与它该红的格，逐个打开核红的格与表相等，并核「代码里认的开关与文件头表里列的不一样」（第 611 行）两个方向。
- B（`research/scripts/mutate.sh` 写法）：`MUTATE_BREAK=<名>  <弄坏哪一处>` 一行，下一行起写点名的项（`run_case` 的标签，括号里写「✗ 自检：」的句首）。表头一句：`research/scripts/mutate.sh:77`「逐项写开关名、弄坏哪一处、打开后自证哪几项红」。文件头没被机器读（`grep -c '开关轮' research/scripts/mutate.sh` 输出 0），逐个打开靠人跑（本报告 2.3 节跑了一遍）。
- C（`.claude/hooks/lib_heavy_tests.py` 写法）：开关名后括号里一句话，点名的格用自然语言（格标题、「共八格红」这类条数）。表头一句：`.claude/hooks/lib_heavy_tests.py:156`「弄坏开关（只给自证用，证明那几格会红）：LIB_HEAVY_TESTS_BREAK 设成下面一个或几个」。
- D（门禁阶段写法，本腿补的第四种，欠账原文没列）：`.claude/gate.d/code-tooling.sh:61`「#   runner-exit-swallowed    research-script-selftests 回读时把每条的退出码都当 0：runner-red 判绿」——开关名、弄坏哪一处、冒号后点名哪几份样本判错。

翻面观测：

1. 今天各写法已有多少文件。`research/prompts/test-script-align-r3-sonnet-model/header-survey.py`（前 140 行里数；A = 文件头里有「第 N 格」，B = 有 `# <变量>_BREAK=<名>` 起头的行，是启发式，正则写在脚本里）：含 `_BREAK` 的文件 78 份，前 140 行里提到的 72 份，其中 A 2 份、B 9 份、两样都不是 61 份，前 140 行没提到的 6 份（`/tmp/claude-1000/test-script-align-r3-sonnet/header-survey.out`）。选哪种，等于选「多少份要改文件头」。
2. 失败行里能不能点名那一项。`grep -l '自证第' research/scripts/* .claude/hooks/*` 命中 3 份（`crash-amplification-item.sh`、`crash-amplification.sh`、`verify-citations.sh`）；`grep -l '✗ 自检：' research/scripts/* .claude/hooks/*` 命中 33 份。A 要求失败行带序号，B 要求带句首；各脚本里序号与句首哪个更稳（同一脚本里句首是否重复）是这一条要量的。
3. 加一格时表要不要重写。`crash-amplification.sh --selftest` 的格数在本批里 11（调度记录 `records/2026-10-01-对接上游测试脚本规范.md:140`）→ 21（第 244 行）→ 27（第 503 行），今天复跑还是 27（2.1 节）；`lib_heavy_tests.py --selftest` 的种数 313（第 242 行）→ 376（第 505 行）→ 393（第 508 行）→ 411（第 515 行）→ 464（第 520 行），今天复跑 464。每次增格，A 的序号与 C 的条数都得跟着改；B 与 D 的项名、样本名不用。观测：下一次增格时表里要改几行。
4. 门禁阶段有没有复跑方。D 的「点名哪几份样本」没有任何一格逐个打开去核（1.3 的 L3）；A 的复跑方已经存在（开关轮）。选 D 同时要新建复跑方。

#### 1.2.3 C622：只给样本自证设的标记由谁设

问题：门禁阶段的 `*_BREAK` 平常跑也认，要加一个「只有样本自证才设」的标记，平常跑里设了开关就退 2；这个标记谁设、在哪一步设。欠账原文：`.claude/kb/checks-owed.md:491`「标记谁来设没定：门禁阶段的样本由上游 `.claude/singlefs-ai-sop/scripts/stage-selftest.sh` 喂（本仓不改上游，它里面零处 `_BREAK`），落成前先定由谁、在哪一步设这个标记」。

今天的事实（命令与输出在 `research/prompts/test-script-align-r3-sonnet-model/rerun.sh` 的「C622」一段）：

- 上游起样本的那一行 `.claude/singlefs-ai-sop/scripts/stage-selftest.sh:124`「out="$(cd "$work" && env -u GATE_BASE -u GATE_STAGED_FROM -u GATE_DIFF_BASE bash "$stage" "$work"」只摘三个变量，别的环境原样传进阶段；样本自己没有设环境的位置（`expect` 与 `setup.sh`，`setup.sh` 在另一个子进程里跑，设的环境传不到阶段）。
- 平常跑的整轮门禁也调它：`.claude/singlefs-ai-sop/scripts/gate.sh:407`「bash "$SCRIPTS/stage-selftest.sh" "$ROOT/.claude/gate.d"」。
- 16 份阶段文件里 15 份 source 共用库 `.claude/gate.d/lib/stage-cells.sh`，54 号不用（`.claude/gate.d/checker-tier-layer0-replay.sh:16`「不改用共用库 lib/stage-cells.sh：它的汇总把 0、77 之外的退出码并成 1，而样本档判绿退 3」）；16 份里 14 份含 `_BREAK`（只有 `checker-tier-lkmm.sh`、`checker-tier-qemu-device-streams.sh` 没有）；含 `_BREAK` 的不同变量名 25 个（除 `STAGE_CELLS_BREAK` 外 24 个，各读各的）；非注释行里读 `_BREAK` 的共 54 行，分在 14 份文件里（含 `lib-forbidden-notations.py` 10 行、`lib-governance-refs.py` 2 行、`lib/rejection-blocks.py` 2 行）。例：`.claude/gate.d/checker-tier-layer0-replay.sh:237`「layer0_break="${GATE_LAYER0_BREAK:-}"」无条件读。
- 今天没有任何一格逐个打开这些开关去跑样本：在 `.claude/gate.d/*.sh` 与 `.claude/gate.d/lib/*` 里找「给 `_BREAK` 赋值再起命令」，只有 `.claude/gate.d/lib/stage-cells.sh:383`「if STAGE_CELLS_BREAK="$break_name" bash "$library_under_test" --selftest-nested」一处真在起（共用库自己的嵌套自证）；其余命中都是注释、`echo` 与文档字符串（1.3 的 L3）。
- 研究脚本一侧已有写法：标记是一个环境变量指向一份文件，文件内容要等于固定的一行，只有 `--selftest` 自己起的子进程设它，读完就清（`research/scripts/crash-amplification.sh:145`「SELFTEST_MARK_CONTENT="crash-amplification.sh --selftest 起的子进程"」，`research/scripts/mutate.sh:137`「SELFTEST_MARK_CONTENT="mutate.sh --selftest 起的子进程"」）。
- 54 号门禁样本一侧已有写法：样本根里放一个标记目录（`.layer0-sample-tools`），阶段看它在不在、里面的假工具要可执行（登记表 `.claude/hooks/selftest-registry.tsv` 的 `samples-light-marker:.layer0-sample-tools` 一行）。

候选定义：

- M1（开的人设）：谁开某个阶段的某个开关去喂样本，谁在那一次 `stage-selftest.sh` 的环境里同时设标记与开关；标记的形态可以照研究脚本那一种（文件加固定一行）。设标记的是调用 `stage-selftest.sh` 的那一方（人、或以后 C625 门禁那一格里的复跑方）。
- M2（样本根里放）：标记是样本目录里的一个文件或目录（形态同 `.layer0-sample-tools`），阶段只在 `<根>/<标记>` 在时读 `*_BREAK`；开关本身仍从环境来（`setup.sh`、`expect` 设不了环境）。
- M3（上游设）：向上游提：`stage-selftest.sh` 起样本时设一个标记变量。本仓不改上游（C622 原文），只能提需求；上游收过同形的需求（`records/2026-10-01-对接上游测试脚本规范.md:500` 记着上游收下「stage-selftest.sh 只喂点名的几道」，加了 `--item`、`--list-items`，主 agent 现查已同步进副本）。

翻面观测：

1. 复跑方在不在。M1 里设标记的就是复跑方。今天没有复跑方（上面第四条）；C625 的门禁一半落成时才会有。观测：C625 要不要给门禁阶段建复跑方——要，M1 的设标记的位置自然出现；不要，M1 只剩「人手开」。
2. 读取点在哪里。54 行读取分在 14 份文件、24 个私有变量名，只有 `STAGE_CELLS_BREAK` 经共用库，54 号一份自己读自己的。标记判一处还是判 54 处：共用库给一个「读开关」的函数（只在标记在时回值），14 份文件的 54 行都得改成调它；观测是这 54 行里经共用库的有几行（`stage-cells.sh` 12 行、其余不经；54 号那一行本来就不经）。
3. 整轮门禁里漏进来的开关。整轮门禁每次提交都经 `gate.sh:407` 起 `stage-selftest.sh`：M3 下标记在整轮的样本步里一直在，平常环境里漏一个 `*_BREAK` 会被样本步认，而真阶段那一步不认（标记不在）；M1 下样本步不认；M2 下带标记的样本（拷进临时目录）样本步会认。观测：在一个干净临时仓里把 `GATE_LAYER0_BREAK` 漏进环境，分别在三种定义下起整轮门禁样本步，看红在哪一步（本腿没跑：整轮门禁与 54 号样本档按定义不归本腿跑，只列要量什么）。
4. 样本能不能带标记。M2 要每份红样本都带标记文件：本仓 `.claude/gate.d/fixtures/` 下有 16 份阶段的样本目录（`stage-selftest.sh --list-items` 列 16 道，本报告 2.1 节复跑），经 `git ls-files .claude/gate.d/fixtures | wc -l` 数得出要放标记的目录个数；放不放得进去取决于样本被 `lib_heavy_tests.py` 判「与 HEAD 加暂存区逐字节相同」那一格（`research/prompts/test-script-align-r1-main-verification.md:27`「只放行和已提交内容逐字节相同的样本」）。

#### 1.2.4 表里别的「未定」（不是这三处，但同样没有定义）

本腿顺着数到的，供主 agent 判要不要进这一轮的交用户表：`.claude/kb/checks-owed.md:478`（C609）「今天落不成：落成之前要先定的两件都还待定」；`.claude/kb/checks-owed.md:479`（C610）「这一类算不算来历没人定过（零轮），要用户或三方定」；`.claude/kb/checks-owed.md:474`（C605）「没核过它们差的是不是键里的那一项」。

### 1.3 这一批该记而没记的缺口（漏记候选）

每条给：现象、命令与原样输出、什么现象会推翻它。「漏记」的判据：`grep -n` 欠账清单 `.claude/kb/checks-owed.md` 找不到对应的一笔。

#### L1 一条用例一个场景只判了 harness 档，checker 档没有

- 上游要求：`.claude/singlefs-ai-sop/rules/test-script-discipline.md:11` 与「门禁管哪一半」（第 94 行）都把「按参数循环的用例拆没拆开」列为项目可接的一格；范围写的是项目自己的验证代码，含穷举与长时间随机的重型验证（第 4 行）。
- 本仓：判法 `research/scripts/crash-case-check.py:195` 的 `harness_test_paths` 只取 `crates/singlefs-harness/tests/*.rs`（第 196 行`return sorted(glob.glob(os.path.join(root, "crates", HARNESS_PACKAGE, "tests", "*.rs")))`）；文件头第 9 行写明是「harness 档一条用例一个场景」。
- 观测：把同一个判法函数原样用在 checker 档 `tests/*.rs` 上（`research/prompts/test-script-align-r3-sonnet-model/one-scenario-on-tier.py`，只调函数、不起测试）：

```
checker-tier tests/*.rs 里查了 150 条用例，判红 2 条
   crates/singlefs-checker-tier/tests/crash_points_of_the_rollback_publish.rs:127 every_crash_point_of_the_rollback_publish_recovers_to_the_third_version_or_to_the_rollback_publish_and_every_image_is_gre
   crates/singlefs-checker-tier/tests/rollback_floor_written_into_the_system_configuration_first_and_normal_unmount.rs:464 normal_unmount_raises_the_floor_to_the_current_txg_with_marked_empty_publishes_o
```

（输出在 `/tmp/claude-1000/test-script-align-r3-sonnet/one-scenario-on-tier.out`。）`crates/*/src/` 里的 `#[cfg(test)]` 单测一概不判（全仓 `crates/**/*.rs` 里 `#[test]` 函数共 1321 条，见 L2 的输出第一行；checker 档 `tests/*.rs` 是其中 150 条，`src/` 里的单测与装置二进制里的单测不在判法的射程）。
- 欠账里没有：`grep -n 'one-scenario' .claude/kb/checks-owed.md` 零命中。
- 推翻它的现象：那 2 条按 harness 档标记（`harness-test-granularity:one-scenario`）本来就该放行，或者上游把 checker 档用例的拆分判据另写（本腿没找到）。

#### L2 用例函数名带欠账号，文件名有门禁、函数名没有

- 上游要求：`.claude/singlefs-ai-sop/rules/test-script-discipline.md:14`「不带里程碑、步号、批次这类来历」（整句是「用例名写场景和预期……不带里程碑、步号、批次这类来历；来历进 commit message 或项目 kb」）。
- 本仓判法：`research/scripts/crash-case-check.py:192` 的 `MILESTONE_FILE_NAME` 里有 `(^|_)c[0-9]{3}(_|$)`（欠账号），但只拿文件名去比（第 230 行 `if MILESTONE_FILE_NAME.search(stem)`）；C610 管的是文件头文档注释里的日期（`.claude/kb/checks-owed.md:479`）。
- 观测（`research/prompts/test-script-align-r3-sonnet-model/test-fn-names.py`，输出 `/tmp/claude-1000/test-script-align-r3-sonnet/test-fn-names.out`）：

```
test fns: 1321
欠账号开头 c<三位数>_ 的: 21 分在 7 份文件
实验号或 m<数> 的（待判是不是来历）: 5
```

21 条里 6 份在 `crates/singlefs-harness/tests/`（例 `c393_tree_table_unreadable_on_every_read`、`c519_known_loss_after_the_warm_up_…`、`c533_row_publish_record_persisted_…`），1 份是 `crates/singlefs-core/src/admission.rs`（`c355_…`、`c375_…`）。另 5 条带实验号或 `m1_`、`m3_`、`m5_`（`lexical_digest_matches_the_e164_anchors`、`eight_byte_flip_probes_recover_as_the_e142_product_records`、`e168_delete_ring_window_borrow.rs` 里三条），算不算来历没人定过。
- 欠账里没有：`grep -n '函数名' .claude/kb/checks-owed.md` 只命中 C48 一行（与用例名无关），没有一笔管用例函数名带来历。
- 推翻它的现象：项目另有规则允许用例名带欠账号（本腿在 `.claude/rules/` 里没找到）；或这 21 条的名字里 `c393` 之类是被测对象的名字而不是来历（例如 `c393` 是某个常量或格式字段）——读那 21 条的函数体可核，本腿没逐条读。

#### L3 门禁阶段的弄坏开关没有复跑方，C625 对它的描述与现状不符

- 上游要求：`.claude/singlefs-ai-sop/rules/test-script-discipline.md:40`「复跑的一方（自证自己，或门禁的自证）逐个打开、看它红在点名的那一项。开关只给自证用，平常跑时不认。」
- 欠账原文：`.claude/kb/checks-owed.md:494`「本仓 `.claude/gate.d/code-tooling.sh` 管的是门禁阶段的…，研究脚本与钩子的弄坏开关没有同类格」——读起来是门禁阶段的一半已经有格在复跑。
- 观测：16 份阶段里 14 份含 `_BREAK`（1.2.3 节）。在 `.claude/gate.d/*.sh` 与 `.claude/gate.d/lib/*` 里找把 `_BREAK` 赋值再起命令的非注释行，只有 `.claude/gate.d/lib/stage-cells.sh:383` 一处（共用库自己的嵌套自证）；`.claude/gate.d/code-tooling.sh` 的「开关 → 样本」是文件头注释（第 60 行起，例第 61 行 `runner-exit-swallowed`），没有一格去打开它们；`.claude/gate.d/fixtures/` 下样本里出现 `BREAK` 的 8 份全是 `setup.sh` 里的注释。调度记录也写着这件事归提交前人手核：`records/2026-10-01-对接上游测试脚本规范.md:108`「第 1 批只改了注释，对应关系要在真仓跑样本核」。
- 与 C622 的关系：C622 要「只有样本自证才设」的标记，设标记的复跑方今天不存在（1.2.3 节观测 1）。
- 推翻它的现象：有一格我没扫到的门禁或脚本在循环里打开阶段的 `*_BREAK`（本腿 grep 的是 `.claude/gate.d/`、`research/scripts/`、`.claude/hooks/`；扫 `research/scripts/*.py`、`*.sh` 里 `GATE_.*_BREAK=` 可再核一遍）。

#### L4 四份自证起真重活，与上游「自证用假驱动」不合，没记成偏离

- 上游要求：`.claude/singlefs-ai-sop/rules/test-script-discipline.md:41`「自证用假的驱动，不起编译与重活，改脚本的人随时能跑；项目拦重型测试的闸不拦自证。」
- 观测：`.claude/hooks/selftest-registry.tsv` 里登记为 heavy 的 3 份（`research/scripts/compile-then-swap.py`：真跑 cargo build 与 cargo test --no-run；`research/scripts/crash-amplification-two-hosts.sh`：本机起两份真的 checker 档用例；`research/scripts/vm-bench.sh`：起虚机）加 heavy-if-set 的 1 份（`research/scripts/layer0-shard-run.sh`）；`v3-counts.sh` 输出「3 heavy、1 heavy-if-set」。门禁里只有两份写了理由，且理由是挂钟：`.claude/gate.d/code-tooling.sh:382`「"research/scripts/vm-bench.sh	自证要连起三次虚机，挂钟太重，不进每轮门禁"」与它下一行 two-hosts；`compile-then-swap.py` 在每轮门禁的 `SELFTEST_RUNNERS` 里跑（`.claude/gate.d/code-tooling.sh:252`）。
- 欠账里没有：`grep -c '假驱动' .claude/kb/checks-owed.md` 输出 0。
- 推翻它的现象：上游对「重活」有项目豁免的写法，或这四份已在别处（豁免登记、给上游的反馈 F1–F12）写明——F1–F12 里没有（`records/2026-10-01-对接上游测试脚本规范.md` 第 113–125 行读过）。

#### L5 入口脚本一层：`replay.sh` 与 `broad-mutation.py` 没按入口脚本的格式核过

上游把入口脚本定义为「把一批登记的项组织起来跑的脚本」，必须带位置参数的一张表、一份输入的工具不算（`.claude/singlefs-ai-sop/rules/test-script-discipline.md:4`「入口脚本指把一批登记的项组织起来跑的脚本」）。本仓这样的脚本：`research/scripts/crash-amplification.sh`、`research/scripts/broad-mutation.py`、`research/scripts/e165-replay-mutation-kill-bounds.sh`、`research/scripts/replay.sh`（`mutate.sh` 必须带变异表，不算）。调度记录的逐节核对表只拿 `crash-amplification.sh` 比过每项行的格式（`records/2026-10-01-对接上游测试脚本规范.md:36`），G3 也只改了它。

- L5a `research/scripts/replay.sh`：用法是 `research/scripts/replay.sh:6`「bash research/scripts/replay.sh [实验号...]      # 不给参数就全跑」（位置参数是实验号）。`grep -c -- '--list-items' research/scripts/replay.sh`、`grep -c -- '--item'`、`grep -c -- '--selftest'` 三条各输出 0（本节命令的合并输出见 `rerun.sh` 的「L5」一段）。阶段层有：`.claude/gate.d/checker-tier-research-build-and-replay.sh` 文件头第 30、31 行给出 `--list-items` 与 `--check experiment-replay --item E142`，它的样本放的是「带假 replay.sh 的小仓」（第 37 行），测的是阶段，不是 replay.sh 自己的三条完整性闸（文件头第 11–15 行写的跑前删本轮输出、收尾行 emitted 与实际行数相等、退出码非 0 判红）有没有会红的证据。欠账清单里点名 `research/scripts/replay.sh` 的是 C39、C125、C429、C487、C532 五条（`grep -n 'research/scripts/replay.sh' .claude/kb/checks-owed.md`），C39（复跑时退出码）与 C532（只剩区间断言）管的是它复跑时怎么判，没有一笔管它自己的判法有没有会红的自证。
- L5b `research/scripts/broad-mutation.py`：每条一行的字段是 `research/scripts/broad-mutation.py:82`「输出：每条一行 BROAD_MUTATION_ITEM sequence=」后面接 `outcome=… stage=… build_seconds=… test_seconds=… name=…`，没有判了多少、红了多少、复用了多少（上游「每项结束打一行：项名、退出码、判了多少、红了多少、复用了多少」在第 31 行）。它有复用这一层（有定论的条跳过，第 69 行一带），所以也不能写「不复用」。
- 推翻它的现象：`replay.sh`、`broad-mutation.py` 被项目另定为不算入口脚本（本腿没找到这样的登记）；或 F7（有复用层而取不出数写「取不出」）已被主 agent 定为只管 `crash-amplification.sh`。

#### L6 调度记录里留了「下一批」尾巴、欠账表里没有的几笔

下面每条都是调度记录写明「进下一批 / 进第二波 / 合入后补」，在 `.claude/kb/checks-owed.md` 里用 `grep -c` 查零命中。命令输出：`grep -c '基线就是红' …` 0、`'BUILTIN_FEEDS'` 0、`'版本表'` 0、`'变异表证过红\|按变异表'` 0、`'selftest-pattern-words-judged'` 0。

1. 单机清场：`records/2026-10-01-对接上游测试脚本规范.md:244`「`ENABLE_ACROSS_MACHINES=0` 时入口读不到清场命令、全量退 78，要单机清场得改 `peer-host-lib.sh`，进第二波」。入口里对应的分支今天还在：`research/scripts/crash-amplification.sh:722`「✗ 多机配置没开双机（ENABLE_ACROSS_MACHINES 不是 1），读不到配置里的清场命令，清不了场」。上游「硬件加速」一节第 80 行有「要退回就改配置或显式传参」：单机（`ENABLE_ACROSS_MACHINES=0`）今天跑不了全量，记录写要改 `peer-host-lib.sh`，没有一笔写成缺口或设计。
2. `mutate.sh` 基线撞 254：`records/2026-10-01-对接上游测试脚本规范.md:509`「下一批：基线撞 254 仍报…红的」，文件头自己也写着：`research/scripts/mutate.sh:67`「基线退 254 照旧按基线不绿退 2」。
3. 钩子一侧两笔：`records/2026-10-01-对接上游测试脚本规范.md:520`「下一批：hook-swap.py 的内置喂法 `BUILTIN_FEEDS` 没加本轮几条；lib 文件头 `selftest-pattern-words-judged`…实测两格」。对照：`research/scripts/hook-swap.py` 第 82 行有 `BUILTIN_FEEDS`，`.claude/hooks/lib_heavy_tests.py` 第 202 行是那一条开关的说明。
4. C605 用例的一个假定：`records/2026-10-01-对接上游测试脚本规范.md:485`「已知风险：C605 用例假定…，推的没跑过」。C605 一行 `.claude/kb/checks-owed.md:474`「是现成的候选，没核过它们差的是不是键里的那一项」只写了候选没核过，没有这一个假定。
5. 种子复现 8 条新测试：`records/2026-10-01-对接上游测试脚本规范.md:248`「缺口：harness 档 8 条新测试只有 2 条按变异表证过红」，要合入后补变异表；C607 一行只写了「先给四个随机二进制各加一个注入失败的入口」。
6. 用户已定的术语改名没有落地也没有账：`records/2026-10-01-对接上游测试脚本规范.md:88` 记用户定「改成快档 / 慢档」，第 221 行排在第二波；今天 `.claude/rules/verification.md:21`「## harness 档里再分轻用例与耗时用例」仍是旧名。它不是检查，不进欠账表，但交用户表里没有这一行。
7. 旁注，不是欠账：`git status --short research/scripts | grep memory-peaks` 输出 `A  research/scripts/memory-peaks.tsv.1569600.partial`，`ls -la` 看是 0 字节、日期 Oct 1 08:07——包装的峰值表写到一半的临时文件被暂存了，随这一批提交会进库。

推翻 L6 任何一条的现象：对应的一笔在欠账表里换了说法（本腿只按关键词零命中判，没逐行读 C604–C635 之外的条）。

#### L7 欠账行里已经过期的句子

- C613 的 `.claude/kb/checks-owed.md:482`「表头那一半 `tooling-writer` 在改」；登记表头今天已经有这条例外：`.claude/hooks/selftest-registry.tsv:10`「只问版本的（参数只有 `-V` 或 `--version`，例如 admission.py --selftest 调真 cargo 的 `-V`）不算起重活，不碍写 light」。
- C612 的 `.claude/kb/checks-owed.md:481`「五样都已在脚本里，主 agent 带前缀跑 `mutate.sh --selftest` 核过就移进已还清」。本腿复跑 `mutate.sh --selftest` 退 0、47 项，16 个开关各退 1（2.1、2.3 节）；移不移进已还清归主 agent。

## 1.4 两笔重了或互相矛盾

1. C20 与 D13 已定项 9：C20 的 `.claude/kb/checks-owed.md:23`「没跑过的那套在下一次里程碑收尾的提交之前判红」；D13 已定项 9 的 `.claude/kb/decisions/13-验证路线.md:184`「跑红或用户不跑，提交照样可以进，记一笔未结的红或…，直到修掉或补跑」。「判红」是不是让提交过不去，两处没说到一起；C20 同一行还要求记每套布局的上次完整跑过时间戳并纳入门禁读数。候选定义：①「判红」= 门禁退非 0、提交过不去（与 D13 冲突）；②「判红」= 记一笔未结的红（与 D13 同）。观测：这一批之后 `gate.sh` 里有没有读这个时间戳的一格——今天没有（C635 的前置栏写「路径清单今天只写在 D13 已定项 9 正文里，门禁要读得先登记成一份文件」）。
2. C409 引了 D13 里没有的句子：`.claude/kb/checks-owed.md:333`「这条准入条件今天没有执行路径，而门禁汇总里看不出它缺席」前面那句，把「里程碑收尾、动了崩溃点重放或判器的那一批提交之前 = 全量档全绿」放在引号里，像是 D13 的引文；`grep -cF '里程碑收尾、动了崩溃点重放或判器的那一批提交之前 = 全量档全绿' .claude/kb/decisions/13-验证路线.md` 输出 0，在 `.claude/kb/checks-owed.md` 里输出 1；D13 今天的原句是上一条那一句（跑红或用户不跑照样可以进）。C409 同一行的前半句引的就是这一句，「= 全量档全绿」与它矛盾。
3. C20 与 C635：C20 要「每套布局」的上次完整跑过时间戳；C635 只写「两样全量判绿时各写一份带日期与输入指纹的标记」（`.claude/kb/checks-owed.md:504`），粒度是两样全量，不是每套布局。C20 一行把机器可读的那一半推给 C635，但两笔的粒度不同，没有一笔管「每套布局」。
4. C604 一行里混着两半，前置只写了其中一半：门禁那一半已落、仍欠 crates 补丁合入后的 GPU 对拍打 `NOT_RUN_NO_GPU_ADAPTER`（前置栏写补丁合入）；另一半「feature-gated-tests 格把 `src/lib.rs` 里挂特性的模块一并数进编进来几条」是门禁本地改动，不依赖补丁（`grep -rn NOT_RUN_NO_GPU_ADAPTER crates` 零命中与它无关），却与补丁合入挂在同一格前置下，看起来也得等。
5. C622、C625 同一块（弄坏开关）：不重复，但耦合——C625 的复跑方是 C622 里「谁设标记」的现成答案之一（1.2.3 节观测 1），而 C625 对门禁阶段的描述（L3）与现状不符。

## 二、V3：前两轮报出的数能不能核

复现命令都在 `research/prompts/test-script-align-r3-sonnet-model/` 里：`rerun.sh <草稿目录>`（只读部分）、`selftests.sh`、`switch-round.sh`、`gate54-switches.sh`、`stage-samples.sh`；原始输出在 `/tmp/claude-1000/test-script-align-r3-sonnet/`（`final-selftests/`、`final-switches/`、`rerun-final/` 等，不入库）。「出处」栏写调度记录 `records/2026-10-01-对接上游测试脚本规范.md` 的行号，或判决、交回报告。

### 2.1 自证格数、种数、开关数、样本判对数

| 数 | 出处 | 原数 | 复现数 | 命令 | 判定 |
|---|---|---|---|---|---|
| `lib_heavy_tests.py --selftest` 种数 | records/2026-10-01-对接上游测试脚本规范.md:520「`lib_heavy_tests.py --selftest` 464 种、`heavy-test-guard.sh --selftest` 980 种退 0」 | 464 | 464，退 0 | `bash research/scripts/capped.sh 8 python3 .claude/hooks/lib_heavy_tests.py --selftest` | 对上 |
| `heavy-test-guard.sh --selftest` 种数与该拒数 | 同上；`/tmp/claude-1000/tooling-writer-hook-brace/report.md` 写「980 种，其中该拒 186 种」 | 980 / 186 | 980 / 186，退 0 | `bash research/scripts/capped.sh 8 bash .claude/hooks/heavy-test-guard.sh --selftest` | 对上 |
| `hook-swap.py --selftest` 种数 | records/2026-10-01-对接上游测试脚本规范.md:515「三份自证退 0（411 种、962 种、9 种）」的第三个 | 9 | 9，退 0 | `bash research/scripts/capped.sh 8 python3 research/scripts/hook-swap.py --selftest` | 对上 |
| `crash-amplification.sh --selftest` 格数与开关轮 | records/2026-10-01-对接上游测试脚本规范.md:503「入口自证 27 格、开关轮 22 个、分项驱动自证 12 格」 | 27 / 22 | 27 格（末行「27 格」）、开关轮 22 行（`grep -c '开关轮：'`），退 0 | `bash research/scripts/capped.sh 8 bash research/scripts/crash-amplification.sh --selftest` | 对上 |
| `crash-amplification-item.sh --selftest` 格数 | 同上第三个数 | 12 | 末行「12 格都对」，退 0 | `bash research/scripts/capped.sh 8 bash research/scripts/crash-amplification-item.sh --selftest` | 对上 |
| `mutate.sh --selftest` 项数 | records/2026-10-01-对接上游测试脚本规范.md:509「自证 47 项，16 个开关各红在点名格」 | 47 | 「查了 47 项」，退 0 | `bash research/scripts/capped.sh 8 bash research/scripts/mutate.sh --selftest` | 对上 |
| `admission.py --selftest` 格数与判错数 | records/2026-10-01-对接上游测试脚本规范.md:243「`admission.py --selftest` 283 格 1 格判错」 | 283 / 1 | 「1 格判错（共 283 格）」，退 1，红的是 `environment-outside-stage-marker` 那一格 | `bash research/scripts/capped.sh 8 python3 research/scripts/admission.py --selftest` | 对上（与第三轮正文「已知、不归本批」同一格） |
| `stage-selftest.sh --list-items` 道数 | records/2026-10-01-对接上游测试脚本规范.md:500「`--list-items` 列出 16 道」 | 16 | 输出 16 行 | `bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh "$PWD/.claude/gate.d" --list-items \| wc -l` | 对上 |
| 54 号样本档判对数 | records/2026-10-01-对接上游测试脚本规范.md:501「点名的 1 道阶段判得都对（8 个样本）」 | 8 | 「点名的 1 道阶段判得都对（8 个样本）：checker-tier-layer0-replay.sh」，退 0 | `bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh "$PWD/.claude/gate.d" --item checker-tier-layer0-replay.sh` | 对上 |
| 门禁道数与格数 | records/2026-10-01-对接上游测试脚本规范.md:243「`gate-structure-check.py` 16 道 103 格全对」 | 16 / 103 | `ls .claude/gate.d/*.sh \| wc -l` 输出 16；各道 `--list` 的格数相加 103 | `rerun.sh` 的「V3 欠账计数」段末（`--list` 逐道数） | 数对上；`gate-structure-check.py` 自己被钩子拒，核不了（2.7） |
| 登记表份数 | records/2026-10-01-对接上游测试脚本规范.md:242「登记 76 份：light 72、heavy 3、heavy-if-set 1」 | 76（72 / 3 / 1） | 脚本行 77（73 / 3 / 1） | `awk` 数登记表（`v3-counts.sh` 的「登记表」段） | 差 1：`research/scripts/hook-swap.py` 在 records:242 之后登记为 light（`.claude/hooks/selftest-registry.tsv` 里该行），解释得通 |
| 样本档登记份数 | records/2026-10-01-对接上游测试脚本规范.md:494「登记 16 道：3 道带假工具标记、3 道 samples-heavy、10 道 samples-light」 | 16（3 / 3 / 10） | 16（3 / 3 / 10） | 同上 | 对上 |
| 历史数：被后来改动覆盖的中间数 | records:140（11 格、24 项）、:242（313 种）、:244（21 格、11 格）、:494（35 项）、:505（376、915、该拒 173）、:508（393、932）、:515（411、962） | 各自 | 不可复现：后来的改动把这些自证又加了格，今天只有最终数 | — | 只能与最终数相比，上面各行已核 |



### 2.2 探针退出码

| 探针 | 出处 | 原数 | 复现 | 命令 | 判定 |
|---|---|---|---|---|---|
| 攻方第二轮通配六条以 implementation-writer 身份 | `research/prompts/test-script-align-r2-main-verification.md` 与 records:515 | 改后全退 2、两条轻通配退 0 | 六条退 2；`admission*.py --selftest`、`crates-mutation-rows.p? --selftest` 退 0；三条非通配对照退 2 | `bash research/prompts/test-script-align-r2-opus-model/rerun.sh <草稿目录>`（只喂钩子 JSON；模型目录 `sha256sum -c SHA256SUMS` 全 OK） | 对上 |
| 攻方 U1-a：不带 `--item` 的改过拷贝 | r2 判决第一节 | 改后退 2 | 退 2；对照 1（带 `--item`）、对照 2（样本不同）也退 2 | 同上 | 对上 |
| 攻方 U2-a：54 号样本假 rustc 不核内容 | `research/prompts/test-script-align-r2-main-verification.md` 的 U2 一行与 C613 | 放行（已知缺口，记进 C613） | 退 0（agent=three-way-attack） | 同上 | 对上；C613 里这条要等「跟踪起进程」那一格落成才红 |
| 第三轮正文第二节点名的五条 | `research/prompts/_test-script-align-r3-body.md` 第 27 行 | `mutate.{sh,x}`、`checker-tier-layer0-replay.{sh,}`、`mutation?.tsv` 退 2；`ls …{a,b}.sh`、`cat crates/*.toml` 退 0 | 退 2、2、2、0、0 | `rerun.sh` 的「探针」段（`hook-probe.py` 以 implementation-writer 身份） | 对上 |
| 辩方第二轮九行 | r2 判决第一节「`rerun.sh` 它跑过九行全 0」 | 全 0 | `kb-facts`、`hook-failures`、`dispatch-guard`、`guard-spot-checks`、`flat-draft`、两份钩子自证、`crash-amp-selftest` 各 `rc=0` | `bash research/prompts/test-script-align-r2-sonnet-model/rerun.sh <新目录>`（输出 `/tmp/claude-1000/test-script-align-r3-sonnet/r2sonnet-rerun.txt`） | 对上 |

### 2.3 开关逐个打开

#### 2.3.1 `mutate.sh` 自证的 16 个开关（交回报「16 个开关各红在点名格」）

模型目录 `switch-round.sh` 起的一轮（`/tmp/claude-1000/test-script-align-r3-sonnet/final-switches/summary.txt`），与原交回 `/tmp/claude-1000/tooling-writer-mutate-parallel-reason/report.md` 的 switch-summary 逐个比；「判错条数」= 输出里 `^  ✗ 自检：` 的行数：

| 开关 | 原退出码 / 判错条数 | 复现 | 开关 | 原 | 复现 |
|---|---|---|---|---|---|
| memoryascaught | 1 / 6 | 1 / 6 | queueasinvalid | 1 / 1 | 1 / 1 |
| memorynotfail | 1 / 1 | 1 / 1 | squeezedascrashed | 1 / 1 | 1 / 1 |
| droprow | 1 / 17 | 1 / 17 | honorbreakoutsideselftest | 1 / 5 | 1 / 5 |
| countblind | 1 / 1 | 1 / 1 | noparallelreason | 1 / 16 | 1 / 16 |
| sharedcopy | 1 / 13 | 1 / 13（另四次抽样 11、9、11、11） | overridecallerreason | 1 / 2 | 1 / 2 |
| nofeatures | 1 / 1 | 1 / 1 | baselinequeueasred | 1 / 1 | 1 / 1 |
| outertimeout | 1 / 6 | 1 / 6 | exportparallelreason | 1 / 1 | 1 / 1 |
| swallowwrapper | 1 / 1 | 1 / 1 | nocap（`RUN_WITH_MEMORY_CAP_BREAK`） | 1 / 10 | 1 / 10 |

16 个开关数对上：文件头 `MUTATE_BREAK=` 的不同取值 15 个（`grep -oE 'MUTATE_BREAK=[a-z-]+' research/scripts/mutate.sh | sort -u | wc -l` 输出 15），加包装的 `nocap` 一个。15 个开关判错条数一致，`sharedcopy` 五次抽样 11、9、11、11、13，文件头自己写了这条「红不红不定」（2.6 节）。只比了退出码与条数，没有逐个比红在哪一项（输出里只有「✗ 自检：」句首，不带 `run_case` 的标签）。`mutate.sh --selftest` 本身退 0、47 项（2.1 节）。

#### 2.3.2 54 号门禁文件头的弄坏开关（交回报「五个开关逐个打开，各自让点名的样本判错」，另一个 `sample-tools-unchecked`）

模型目录 `gate54-switches.sh`：逐个设开关，经 `stage-selftest.sh … --item checker-tier-layer0-replay.sh` 喂样本（登记为 samples-light-marker，假 cargo）。这是人手版的复跑方，今天没有一格门禁在做它（L3）。不开开关：退 0、8 个样本全对。输出 `/tmp/claude-1000/test-script-align-r3-sonnet/g54/summary.txt`：

| 开关 | 文件头 `.claude/gate.d/checker-tier-layer0-replay.sh` 写的判错样本 | 复现判错的样本 | 判定 |
|---|---|---|---|
| `GATE_LAYER0_BREAK=no-findings-file` | 「red 或 green」（没点名） | green、red | 对得上，但文件头没点名 |
| `…=ignore-unfinished` | 同上 | red | 同上 |
| `…=ignore-red-states` | 同上 | red | 同上 |
| `…=no-truncation` | 同上 | red | 同上 |
| `…=static-check-skipped` | static-check-red | static-check-red | 对上 |
| `CRASH_CASE_CHECK_BREAK=rows-ignored` | static-check-red | static-check-red | 对上 |
| `…=no-feature-pass` | quick-features-green | quick-features-gpu-not-run、-green、-red、-zero 四份 | 文件头只点了一份，实际四份 |
| `…=features-not-from-source` | quick-features-green | 同上四份 | 同上 |
| `…=ignore-feature-pass-exit` | quick-features-red | quick-features-red | 对上 |
| `…=ignore-zero-feature-files` | quick-features-zero | quick-features-zero | 对上 |
| `…=ignore-gpu-not-run` | quick-features-gpu-not-run | quick-features-gpu-not-run | 对上 |
| `…=sample-tools-unchecked` | sample-cargo-not-executable | sample-cargo-not-executable | 对上 |

12 个开关都让整道退 1。文件头点名的样本与实际判错的样本不相等的有 6 个（4 个没点名、2 个点少了），原交回的「各自让点名的样本判错」对「点名的样本一定判错」成立，对「只有点名的判错」不成立；C625 要的「红在点名的那一项」若按相等判，这 6 个会红。

#### 2.3.3 第三轮正文点名的 `code-tooling.sh` 两份样本

`bash research/scripts/capped.sh 8 bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh "$PWD/.claude/gate.d" --item code-tooling.sh`（模型目录 `stage-samples.sh <目录> code-tooling.sh`）退 1，末行「✗ 2 个样本判错（判对 24 个，0 个阶段没有样本）」，判错的是 `green`、`coverage-claims-green`，各缺「research/scripts/ 里声称有 --selftest 的 2 份中 1 份有门禁阶段在跑」与「research/scripts/vm-bench.sh：自证要连起三次虚机」两句。与正文第 28 行「已知、不归本批」同一件，对上。

### 2.4 欠账里写的计数

命令都在 `v3-counts.sh`（输出 `/tmp/claude-1000/test-script-align-r3-sonnet/rerun-final/rerun.out` 的「V3 欠账计数」段）。

| 欠账与数 | 原数 | 复现数 | 命令 | 判定 |
|---|---|---|---|---|
| C604：`tests/` 顶层整份挂特性的文件 | 7 份 | 7 份（行 7、文件 7） | `grep -n '^#!\[cfg(' crates/singlefs-checker-tier/tests/*.rs \| wc -l`、`grep -ln … \| wc -l` | 对上 |
| C604：`src/lib.rs` 挂特性的 6 个模块的 `#[test]` | 45 条 | 45（13 + 9 + 5 + 0 + 0 + 18） | `grep -c '#\[test\]' crates/singlefs-checker-tier/src/<模块>.rs` 逐个 | 对上 |
| C604：`cfg_attr` 命中 | 1 处，`record_checker_judges_absence_by_the_persisted_set.rs:180` | 1 处，同一行 | `grep -rn 'cfg_attr' crates --include=*.rs` | 对上 |
| C604、C608、C620：`NOT_RUN_NO_GPU_ADAPTER`、`decide_gpu_test_start` 在 `crates/` 里 | 零命中 | 0、0 | `grep -rn NOT_RUN_NO_GPU_ADAPTER crates`、`grep -rn decide_gpu_test_start crates` | 对上 |
| C605：`each_attribute_changes_the_reuse_key` 在 `crates/mutations.tsv` | 0 | 0 | `grep -c each_attribute_changes_the_reuse_key crates/mutations.tsv` | 对上 |
| C605：四种关复用的名字在 checker 档包里 | 零命中 | 0 | `grep -rn 'no_reuse\|without_reuse\|reuse_off\|disable_reuse' crates/singlefs-checker-tier \| wc -l` | 对上 |
| C606：用例不经入口时的默认展开 | `DEFAULT_EXPAND_UP_TO` = 6，`…through_rollback.rs:31`、`…through_unmount.rs:38` | 两处都是 `const DEFAULT_EXPAND_UP_TO: usize = 6;`，行号 31、38 | `grep -n 'const DEFAULT_EXPAND_UP_TO: usize' …` | 对上 |
| C608：`crates/mutations.tsv` 里开 `gpu` 的行 | 12 | 12 | `awk -F'\t' 'NF==6 && $5 ~ /gpu/' crates/mutations.tsv \| wc -l` | 对上 |
| C608：改 `.wgsl`、`crash_verify_gpu.rs`、`gpu_unit_checks.rs` 的行 | 0 | 0 | `awk -F'\t' 'NF==6 && $2 ~ /\.wgsl\|crash_verify_gpu\|gpu_unit_checks/' crates/mutations.tsv \| wc -l` | 对上 |
| C609、C627：E165 装置定死的钉表段数 | 8（第 1681 行） | `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs` 第 1681 行是 `const PINNED_TABLE_COLUMN_COUNT: usize = 8;` | `awk 'NR==1681' …` | 对上 |
| C610：`crates/*/tests/*.rs` 里带日期的 `//!` 行 / 文件 | 89 行 / 66 份 | 89 / 66 | `grep -E '^//!.*[0-9]{4}-[0-9]{2}-[0-9]{2}' crates/*/tests/*.rs \| wc -l`；`grep -lE … \| wc -l` | 对上 |
| C610：其中同一行没有 kb 指路 | 70 行 / 52 份 | 70 / 52 | 上一条接 `\| grep -vE '(D\|E\|C\|I)[0-9]+（\|\.claude/kb/\|已定项\|未定项'` | 对上 |
| C610：日期后跟「用户」或「定」的行 / 其中同一行带 kb 指路 | 70 / 18 | 70 / 18 | 同上，`.*(用户\|定)` 与再接 `grep -E '(D\|E\|C\|I)[0-9]+（\|\.claude/kb/\|已定项\|未定项'` | 对上；按另一份更松的正则（`已定项\|D[0-9]+（\|C[0-9]{3}\|E[0-9]{2,3}`，第一轮正推腿 `rerun.sh` 第 45 行）数是 23，见 2.6 |
| C610：既没有「用户」也没有「定」的带日期行 | 14 | 14 | `grep -E '^//!.*日期' … \| grep -vE '用户\|定' \| wc -l` | 对上 |
| C610：四个举例的行（`…two_flows_share_their_prefix.rs` 第 2 行、`crash_enumeration_position_addressed_trees.rs:2`、`crash_enumeration_record_spill_over_stream.rs:3`、`acquisition_refuses_…panics.rs:1`） | 内容如欠账所写 | 逐行 `awk 'NR==N'` 读到：第 2 行「用户 2026-09-28 定」；`:2`「D8（核心索引结构） 已定项 14，用户 2026-09-24 定 K1–K4」；`:3`「已定项 17（用户 2026-09-24 定）」；第 1 行「实审 A3c（规格 …，写于 2026-09-27）」（后一份在 `crates/singlefs-harness/tests/` 下，不在 checker 档目录） | `awk 'NR==2' crates/singlefs-checker-tier/tests/…` | 对上（欠账没写后一份在 harness 目录，不影响计数） |
| C611：进度间隔 | `PROGRESS_INTERVAL_SECONDS = 300`，`broad-mutation.py` 与 `e165-replay-mutation-kill-bounds.sh` | `research/scripts/broad-mutation.py:154` 与 `research/scripts/e165-replay-mutation-kill-bounds.sh:175` 各有 `PROGRESS_INTERVAL_SECONDS = 300` | `grep -n 'PROGRESS_INTERVAL_SECONDS = 300' …` | 对上 |
| C620：PCI 厂商号 0x10de 的设备 | 4 个，2 个 class 0x030000、2 个 class 0x040300 | 4 个：`0000:01:00.0`、`0000:10:00.0` 是 0x030000，`0000:01:00.1`、`0000:10:00.1` 是 0x040300 | `for d in /sys/bus/pci/devices/*; do …` 读 vendor 与 class | 对上 |
| C622：含 `_BREAK` 的研究脚本 / 钩子 | 48 份 / 9 份 | 研究脚本：工作区 51、暂存区 49；钩子 9 | `grep -l '_BREAK' research/scripts/* \| wc -l`；`git grep --cached -l '_BREAK' -- research/scripts \| wc -l`；钩子同法 | 对不上，2.6 列：F2 原清单（`/tmp/claude-1000/tooling-writer-test-script-align/break-files.txt`）里研究脚本是 46 份，今天比它多 5 份：`crash-amplification-lib.sh`、`verify-citations.sh`、`hook-swap.py`（暂存区里都有）与 `build-patched-qemu.sh`、`qemu-blklogwrites-check.sh`（别的会话的未跟踪文件）；48 = 46 + 前两份，`hook-swap.py` 在 C622 写下之后才新建，解释得通 |
| C623：`states_reused` 在 `crates/` 里 | 零命中 | 0 | `grep -rn states_reused crates \| wc -l` | 对上 |
| F2 盘点（records:147）：含 `*_BREAK` 的文件 / 自己读那个变量的行数 | 73 份 / 54 份 | 用它留下的清单与脚本（`/tmp/claude-1000/tooling-writer-test-script-align/break-files.txt` 73 行、`break-inventory2.py`）重跑：清单 73 份（研究脚本 46、钩子 9、阶段 18）；脚本打出 54 行（`wc -l`），与原 `break-inventory2.tsv` 逐行比有几行的「缺第三样」一栏变了（文件头后来改过，例 `checker-tier-layer0-replay.sh` 的开关名从 1 个认成 3 个） | `python3 /tmp/claude-1000/tooling-writer-test-script-align/break-inventory2.py /tmp/claude-1000/tooling-writer-test-script-align/break-files.txt \| wc -l`；今天的含 `_BREAK` 文件 `grep -rl '_BREAK' research/scripts .claude/hooks .claude/gate.d .claude/scripts --exclude-dir=fixtures \| wc -l` 输出 78 | 73 与 54 都对上（只读旧清单）；今天的 78 比 73 多的 5 份就是上一行那 5 份 |

### 2.5 两份判决里的引文与现查行号

| 数 | 原数 | 复现 | 命令 | 判定 |
|---|---|---|---|---|
| 第一轮判决 cite-check 总处数 | `research/prompts/test-script-align-r1-main-verification.md` 第一节「核了 69 处引文，对上 69 处，没判 0 处」 | 总处数 69 对上；对着今天的树 35 处对不上、0 处没判 | `judgement-lines.sh`（内部是 `python3 research/scripts/cite-check.py research/prompts/test-script-align-r1-sonnet-output.md research/prompts/test-script-align-r1-opus-output.md --root . --background research/prompts/_test-script-align-r1-background.md`） | 总数对上；「全对」今天复现不了，原因见下 |
| 第二轮判决 cite-check 总处数 | `research/prompts/test-script-align-r2-main-verification.md` 第一节「核了 100 处引文，对上 100 处，没判 0 处」 | 总处数 100 对上；今天 43 处对不上、0 处没判 | 同上，换 r2 的两份报告与背景材料 | 同上 |
| 为什么今天对不上 | 判决当时对着开工快照核 | 第一轮快照 `judged-sha256.txt` 30 份里 17 份今天 `sha256sum -c` 为 FAILED、`kb-sha256.txt` 4 份全 FAILED；第二轮 16 份里 8 份、kb 3 份全 FAILED；对不上的引文落在哪几份：第一轮 `checks-owed.md` 11、`13-验证路线.md` 5、`decisions-history.md` 4、`agent-common.md` 2、`tooling-writer.md` 2、`selftest-registry.tsv` 2、`CLAUDE.md` 2、`crash-amplification.sh` 2 等，第二轮 `checks-owed.md` 22、`lib_heavy_tests.py` 8、`13-验证路线.md` 6、`heavy-test-guard.sh` 4 等 | `sha256sum -c research/prompts/test-script-align-r{1,2}-snapshot/{judged,kb}-sha256.txt`；`grep '✗ test-script' <cite-check 输出>` 按被引文件数 | 每一处对不上的被引文件都在 FAILED 那一批里（第一轮另有 `.claude/gate.d/doc-decisions.sh` 不在快照清单里，但 `git status --short` 显示它有改动）；这是「冻结证据」的正常漂移，不是引文写错。第三轮快照（`test-script-align-r3-snapshot`）11 份、kb 2 份今天 `sha256sum -c` 全 OK |
| 第一轮判决「主 agent 现查坐实」里的 find 与行号 | T2-2：`find .claude/gate.d/fixtures/checker-tier-layer0-replay.sh/quick-features-*/ -path '*layer0-sample-tools*' -type f -printf '%m %p\n'` 出 `664 …/cargo`、`664 …/rustc`、`664 …/quick-tier` | 今天同一条命令：12 个工具文件 755（四份样本各 `cargo`、`rustc`、`quick-tier`），另有 9 个数据文件（`cases/*.log`、`*.exit`）664 | `find …`（`v3-counts.sh` 的 T2-2 段，输出 `12 755`、`9 664`） | 是判决采纳后的改动，原数不再成立；判决当时的数没法复现，只能说今天的状态与「采」的去向一致 |
| 同一份判决的行号：`lib_heavy_tests.py` 第 141、1595、2142 行；`crash-amplification.sh` 第 658、715–719、721 行；`mutate.sh` 第 84–86、90 行；`crash-case-check.py` 第 190–192 行 | 各行的内容见判决 | 今天这些行的内容变了（例：`lib_heavy_tests.py:141` 是花括号与参数通配的说明，`crash-amplification.sh:658` 是 `if (( ${#test_only_given[@]} > 0 )); then`，`mutate.sh:90` 是 `every-outcome、…` 的说明，`crash-case-check.py:190` 是注释、191–192 是 `MILESTONE_FILE_NAME`） | `judgement-lines.sh` 的「第一轮判决里点名的行号」段（`awk 'NR==N'`），输出 `/tmp/claude-1000/test-script-align-r3-sonnet/judgement-lines.out` | 行号漂移，没有一处被引的东西不存在；这些行号是「当时的现查」，不是欠账或规则里的引用 |
| 第一轮判决 T3-3「`grep -n 'ignored_points\|states_skipped_by_ignore' research/scripts/crash-amplification.sh` 零命中」 | 0 | 今天 21 行命中（入口已读这两个字段） | `grep -c 'ignored_points\|states_skipped_by_ignore' research/scripts/crash-amplification.sh` | 原数是改前的状态；T3-3 采纳后今天不再是 0，与判决的去向一致 |
| 第一轮正推腿 `rerun.sh` 第 2 段：指向上游节名逐字存在 | 28 处、坏 0 | 今天 28 处、坏 0 | `bash research/prompts/test-script-align-r1-sonnet-model/rerun.sh`（输出 `/tmp/claude-1000/test-script-align-r3-sonnet/r1sonnet-rerun.txt`，末行「total 28 bad 0」） | 对上 |

### 2.6 对不上的，列出原数、复现数与命令

| 数 | 原数 | 复现数 | 命令 | 说明 |
|---|---|---|---|---|
| `mutate.sh --selftest` 开关 `sharedcopy` 的判错条数 | 13（`/tmp/claude-1000/tooling-writer-mutate-parallel-reason/report.md` 的 switch-summary） | 五次抽样：11、9、11、11、13（第一次开关轮 11；再抽三次 9、11、11，`/tmp/claude-1000/test-script-align-r3-sonnet/sharedcopy-{1,2,3}.out`；模型目录 `switch-round.sh` 那一轮 13，`final-switches/sharedcopy.out`） | `MUTATE_BREAK=sharedcopy bash research/scripts/capped.sh 8 bash research/scripts/mutate.sh --selftest`，数 `grep -c '^  ✗ 自检：'` | 文件头自己写了「红不红不定」（`research/scripts/mutate.sh:90`：这一条开关让几个工作进程并发改同一份源码，几条例「可能跟着红」）；五次的数在 9 到 13 之间，退出码都是 1，各次输出都含文件头点名的那一句（MUTATE_JOBS=4 时变异应当落在至少 2 份副本上）。这不是自证坏了，是原交回把一次抽样的 13 当成了定数 |
| 两份判决 cite-check「全对」 | 69 / 100 全对 | 总处数 69、100 对上；今天 35、43 处对不上 | 2.5 节 | 被引文件在判决之后被改（快照 sha256 FAILED），不是引文错 |
| C622「研究脚本 48 份」 | 48 | 工作区 51、暂存区 49 | `grep -l '_BREAK' research/scripts/* \| wc -l`、`git grep --cached -l '_BREAK' -- research/scripts \| wc -l` | 2.4 节：48 + `hook-swap.py` = 49，另 2 份是别的会话的未跟踪文件 |
| 登记表「76 份（72 / 3 / 1）」 | 76 | 77（73 / 3 / 1） | 2.1 节 | 差的 1 份是 `hook-swap.py`（light），在 records:242 之后登记 |
| C610「日期后跟用户或定的 70 行里 18 行带 kb 指路」 | 18 | 18（按欠账写明的写法）；第一轮正推腿自己的 `rerun.sh` 第 45 行打 23 | 欠账写法：`grep -E '^//!.*日期.*(用户\|定)' crates/*/tests/*.rs \| grep -E '(D\|E\|C\|I)[0-9]+（\|\.claude/kb/\|已定项\|未定项' \| wc -l`；那份 `rerun.sh`：`grep -cE '已定项\|D[0-9]+（\|C[0-9]{3}\|E[0-9]{2,3}'` | 差 5 行是正则不同：后者认 `C393`、`E142` 这类不带括号简称的编号，前者要求「编号 + 全角括号」。C610 要的是「同一行有 kb 指路」，哪一种算指路没写；23 与 18 都「复现了」，但不是同一个数 |
| H10「自证 19875 处 / 1098 文件、自检 2498 处 / 477 文件」 | records:79 | 全仓（除 `.git`、`target`、上游副本）自证 33321 处 / 1595 文件、自检 3235 处 / 560 文件；再排 `prompts`、`records`、`results`：自证 2411 / 324、自检 630 / 166；HEAD（排副本）自证 12017 / 1252、自检 1967 / 456 | `v3-counts.sh` 的「术语计数」段；`git grep -I -o 自证 HEAD -- . ':!.claude/singlefs-ai-sop' \| wc -l` | 对不上，原命令没写；本腿试了三种范围都不是原数。记录里没有取法，这一对数不能复核 |
| 「耗时用例 100 个文件 267 处、轻用例 22 个文件 48 处」 | records:88 | 全仓：耗时用例 163 / 853、轻用例 31 / 66；排 `prompts`、`records`、`results`：35 / 132、5 / 8；HEAD（排副本）：66 / 205、18 / 32 | 同上 | 对不上，取法没写；术语改名还没做（L6 第 6 条），这两个数要等改名那件开工时按它的取法重数 |
| 判决与记录里的行号：`records/2026-10-01-对接上游测试脚本规范.md:36`「`research/scripts/crash-amplification.sh` 第 240–268 行」、C607 的 `history.rs:4481` | 240–268、4481 | 入口里每项行早已移位；`crates/singlefs-harness/src/history.rs` 里「新发现 …第一个种子」那一句在第 4482 行，`writeln!` 从第 4479 行起 | `grep -n '种子' crates/singlefs-harness/src/history.rs \| awk -F: '$1>4440 && $1<4520'` | 行号漂移（工作区对 `history.rs` 有 6 加 5 减的改动：`git diff --stat HEAD -- crates/singlefs-harness/src/history.rs`）；第 4481 行与句子差一行，不影响欠账的意思 |

### 2.7 复核不了的

- `research/scripts/gate-structure-check.py`（records:243「16 道 103 格全对」）：被 `.claude/hooks/heavy-test-guard.sh` 拒（提交时才跑的检查，three-way-forward 不跑）；本腿只数了各道 `--list` 的格数相加（103），没有判它的「全对」。
- 54 号 `--full`、崩溃放量全量、crates 变异整表（records:494 的「crates 变异门禁四份样本照预期 4 份」，登记为 samples-heavy）、QEMU、herd7、形式化验证：都是重型测试，按定义不跑。这几道的样本档、`--selftest` 里登记为 heavy 的，数据只能照交回里写的。
- C607 调查员草稿探针「48/48、48/48、32/32、32/32 全不同，快档对照 96 个种子 0 个不同」：要编 `crates/singlefs-harness` 跑随机历史，且 `crates/` 这一轮不判；探针草稿 `/tmp/claude-1000/investigator-test-script-gaps/` 在，本腿没复跑。
- 第 2 批四份 `crates/` 补丁里写的数（例如种子复现「8 条新测试只有 2 条按变异表证过红」、GPU 对拍「9 行变异」「`apply-writer-patch.py --dry-run` 过」）：补丁没合入、要在新底座上重跑 `--dry-run`；本腿只数了 `states_reused` 在补丁里的分布（1.2.1 节）。
- 各件交回里「主 agent 带前缀跑」的几份自证（`crash-amplification.sh --selftest`、`mutate.sh --selftest` 等）带 `SINGLEFS_HEAVY_TESTS=user-request` 前缀的那一趟：本腿跑的是不带前缀、登记为 light 的同一条命令，同一份代码；前缀只影响钩子放不放行，两趟输出没有逐字比（推的）。

## 三、什么现象会推翻这些结论

- V3 里每一行「对上」：同一条命令在草稿或模型目录里重跑得到别的数；被引文件在复现之后又被别的会话改过（本腿读的是 2026-10-01 的工作区，`git status` 里有别的会话未提交的改动）。
- V3「对不上」：2.6 节每一行都给了差额的来源；推翻它的现象是原数的取法被找到且复现出原数（H10 与改名那两对数，取法没留下）。
- V2 的 L1–L7：各条末尾写了；总的说，欠账清单或规则里能 `grep` 到对应的一笔，就不是漏记。
- 三处「未定」：不选，只给候选；翻面观测里每条都是命令或输入，量出来的数能改变「哪个候选更窄、更宽」的判断。

## 四、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| V2 欠账完整性：对照上游逐节 | 有漏记 | 7 条候选：L1 checker 档没判「一条用例一个场景」（2/150）、L2 用例函数名带欠账号 21 条、L3 门禁阶段弄坏开关无复跑方（C625 描述不符）、L4 四份自证起真重活未记偏离、L5 `replay.sh`、`broad-mutation.py` 没按入口格式核、L6 调度记录里 6 笔「下一批」尾巴不在欠账表、L7 C612、C613 两处过期句 |
| V2 三处「未定」：C623 棘轮 | 只给候选与翻面观测 | K 常量 / R 现读（读范围三种）/ 按项登记（零轮）；观测：补丁合入后 `states_reused` 命中 3 个文件、入口今天 0 处读源码、第二台 checkout 是否同提交 |
| V2 三处「未定」：C625 文件头写法 | 只给候选与翻面观测 | A 格号、B 项名、C 自然语言、D 样本名（本腿补）；观测：现有份数 2 / 9 / 61、失败行带序号 3 份对句首 33 份、增格时表要改几行、门禁阶段无复跑方 |
| V2 三处「未定」：C622 标记 | 只给候选与翻面观测 | M1 开的人设 / M2 样本根里放 / M3 上游设；观测：复跑方今天不存在、54 行读取点分在 14 份文件、整轮门禁每次经 `gate.sh:407` 起样本步 |
| V2 两笔重了或互相矛盾 | 有 | C20 的「判红」与 D13 已定项 9 的「提交照样可以进」；C409 把 D13 里没有的句子放在引号里；C20 与 C635 粒度不同；C604 前置只写了一半；C622 与 C625 耦合 |
| V3 自证格数、种数、开关数、样本判对数 | 一致（一处抽样差） | 464/980/9/27/22/12/47/283/16/8/103 全复现；`sharedcopy` 13 对五次抽样 9–13（文件头写了红不红不定）；12 个 54 号开关里 6 个点名与实测的样本集不相等 |
| V3 探针退出码 | 一致 | 通配六条退 2、两条轻通配退 0、点名五条 2/2/2/0/0、辩方九行 rc=0 |
| V3 欠账里的计数 | 一致（有解释的漂移） | C604/C605/C606/C608/C609/C610/C611/C620/C623 全复现；C622 48→49/51、F2 73/54 用原清单复现、登记 76→77 都有来源；C610 的 18 与第一轮腿的 23 是正则不同 |
| V3 判决里的引文与行号 | 总数一致，逐处今天对不上 | 69 与 100 总处数对上；今天 35、43 处对不上，全在快照 sha256 已 FAILED 的被引文件里；第三轮快照 0 FAILED |
| V3 术语计数（H10、改名） | 对不上，不可核 | 原取法没留下，试了三种范围都不是原数 |

## 五、没做什么

- 没判别：V1（攻方的格）；没替主 agent 采纳或出判决，没写倾向。
- 没编译、没跑 checker 档、没跑 54 号 `--full`、QEMU、herd7、crates 变异整表、形式化验证、`gate.sh` 整轮、`gate-structure-check.py`（被钩子拒，2.7 节）。
- 没改 `crates/`、没碰第 2 批四份补丁（只读了 `crates.patch` 数 `states_reused`，没合入）。
- 没逐条读 L2 里 21 条用例的函数体（`c393` 等是不是被测对象的名字，未核）；L1 的 2 条没读源码核是否该放行。
- 没在三种「标记谁来设」的定义下真起整轮门禁样本步（按定义不归本腿跑，只列了要量什么）。
- C607 的草稿探针数没复跑；H10 与改名两对术语数取法没留下，没能复核。
- 不入库的草稿：`/tmp/claude-1000/test-script-align-r3-sonnet/`（各份原始输出与 `final-selftests/`、`final-switches/`、`g54/`、`ss/`、`rerun-final/`）。入库的只有本报告与 `research/prompts/test-script-align-r3-sonnet-model/`。草稿里没有编译目录、仓副本或工作树。
