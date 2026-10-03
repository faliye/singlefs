# test-script-align-r2 辩方腿（Sonnet）报告：U3、U4

日期：2026-10-01。批次：`records/2026-10-01-对接上游测试脚本规范.md#2`。立场：辩方——复核第一轮判决 `research/prompts/test-script-align-r1-main-verification.md`，替被判出局或被留下的一方找站得住的理由，再照写回后的字面走一遍。
方案按 `crates/` 今天的实现谈：`crates/` 这一轮不判，凡是「补丁里才有」的东西（`NOT_RUN_NO_GPU_ADAPTER` 的打印、`states_reused=`、`decide_gpu_test_start`）都写「今天 crates 零命中」。
探针与原样输出在 `research/prompts/test-script-align-r2-sonnet-model/`（`rerun.sh` 复跑，`SHA256SUMS` 在同目录）。重型测试没跑；只跑了登记为 light 的 `--selftest` 三份（本仓路径）与只读的 grep / 探针。

## 一、结论（先看这一段）

1. **判决表里采的改法，逐条现查都打中了被攻的那个形态**（T2-1、T2-2、T2-3、T2-4、T2-6、T3-1、T3-2、T3-4、T3-6、T1-1、T1-2、T1-5：改法在今天的文件里；钩子两份与崩溃放量入口的自证我现跑退 0，`mutate.sh` 与 54 号样本档的自证没复跑）。没有一条「采」是采错的；T3-3 采的是两半，其中「列出哪几类」今天做不到（见下一条④）。
2. **但有四处同一形态在别的臂上没被罩住，判决没点名**：① 弄坏开关平常跑也认（T3-5、T3-7 的形态）今天还在 research/scripts 与钩子的几十份文件里，C622 的「怎么拦」够不着它们；② feature-gated-tests 只扫 `tests/` 顶层，`src/lib.rs` 里挂特性的 6 个模块（共 45 条 `#[test]`）不在它的射程；③ stage-selftest.sh 不带 `--item` 时仍按名字认（T2-3 的形态），没有任何 C 号；④ T3-3 要的「列出没判的是哪几类」今天做不到，C605 写「这一半另算」却没有行接。
3. **T1-3 写回的字面照着干会做错三处**（第三节）：「全量」有两个所指、「动了崩溃点重放或判器」没有判法、「谁记上一次全量」没有人记；D13 同一段里「准入条件」与「异步准入判据、红了不回退」互相打架；C20 的尾部两句是主 agent 自己改的、不是用户选的。
4. **欠账改写的「怎么拦」**（第四节逐笔）：C609 落不成（栏里自己写着要先定两件）；C604、C605、C608、C610、C611、C613、C622、C623、C625 要补一句才能落成；C606、C607、C612、C620、C624、C626、C627 能落成。C612 的行号九处里八处已经对不上；新记的几笔没有和旧行重。
5. **改钩子那条流程会拦死整个会话**：钩子 .sh 里 bash 层一个语法错，任何 agent 的任何 Bash 都退 2（连「换回原样」那条 Bash 也被拦）；挂 Agent|Task 的派发闸同样。其余三种坏法是静默放行（第五节实测表）。更稳的做法在第五节末尾。

## 二、U3 表一：第一轮判决逐条复核

判定三档：站得住（改法打中了被攻的形态）/ 够不着（改法没打到那个形态，或去向判错）/ 同样打中别的臂（同一形态在没改的写法上还在）。「推翻」栏写什么现象会推翻这一格。

| 判决条目 | 辩方判定 | 证据 | 推翻它的现象 |
|---|---|---|---|
| T2-1 样本档只看标记目录在不在 | 站得住 | `.claude/hooks/lib_heavy_tests.py:1027`：「def stage_sample_marker_matches(stage, root, marker):」，函数体核假 cargo 可执行且与仓里某份样本逐字节相同；54 号自己也退 2：`.claude/gate.d/checker-tier-layer0-replay.sh:20`：「样本档的标记目录里假 cargo 或假 rustc 不在或不可执行」。同形的两个标记（herd7、QEMU）是文件，只核字节相同，头注释已写 | 空的 `.layer0-sample-tools/` 加一份非样本内容的可执行 cargo 仍被放行 |
| T2-1 同形：假 rustc | 同样打中别的臂（头注释已披露，今天不可达） | `.claude/hooks/lib_heavy_tests.py:127`：「54 号的假 rustc 不核」；54 号阶段侧核 rustc 在且可执行（上一行 20），内容不核 | 有人在样本根放内容不同的可执行 rustc 而阶段照样放行 |
| T2-2 新样本假工具没有执行位 | 站得住 | `git ls-files -s` 对 `.claude/gate.d/fixtures/checker-tier-layer0-replay.sh/*/.layer0-sample-tools/{cargo,rustc,quick-tier}`：除故意坏的 `sample-cargo-not-executable`（100644）外全是 100755（`outputs/guard-spot-checks.out` 第 C 段原样） | 暂存区里出现 100644 的假工具而样本不是故意红的 |
| T2-3 自证登记按路径尾部认 | 站得住（登记表那一半） | `.claude/hooks/lib_heavy_tests.py:932`：「return os.path.realpath(resolved) == os.path.realpath(os.path.join(SELFTEST_REGISTRY_ROOT, key))」；`outputs/guard-spot-checks.out` 第 A 段：路径尾部写成 `.claude/hooks/` 的草稿拷贝，子 agent 与不带前缀的主 agent 喂 `--selftest` 都退 2，主 agent 带 `SINGLEFS_HEAVY_TESTS=commit` 退 0；本仓那一份谁都退 0 | 别处同名拷贝的 `--selftest` 子 agent 喂进去退 0 |
| T2-3 同形：stage-selftest.sh 不带 `--item` | **同样打中别的臂** | `.claude/hooks/lib_heavy_tests.py:128`：「不带 --item / --list-items 的 ① 仍按名字与位置认 stage-selftest.sh」；`grep -n 'stage-selftest' .claude/kb/checks-owed.md` 只命中 C46、C66、C153、C613（`outputs/kb-facts.out` 第 15 段），没有一行记这口子 | 欠账表里出现一行点名它；或钩子不带 `--item` 时也核真实路径 |
| T2-4 样本里的 setup.sh 不核 | 站得住（样本目录那一半） | `.claude/hooks/lib_heavy_tests.py:1104`：「def sample_directory_changes(sample_directory, stage):」按暂存区逐文件比（多出的 setup.sh、模式、内容）；`heavy-test-guard.sh:72`：「样本改完要暂存之后才放行」 | 样本目录多一份 setup.sh 仍被放行 |
| T2-4 同形：被喂阶段目录里的共用库与登记表 | 同样打中别的臂（头注释已披露） | `.claude/hooks/lib_heavy_tests.py:127`：「阶段读的登记表（stage-inputs.tsv 这类）不与仓里比」；`.claude/kb/checks-owed.md:484`（C613）只写样本的 `setup.sh`、不写阶段 source 的 `lib/` | 钩子核阶段 source 的 lib/ 与登记表，或 C613 运行时检查把整棵树罩进去且在提交前跑 |
| T2-4 的代价：tooling-writer 改了样本之后自己跑不了 | 站得住的新事实（定义没写） | `heavy-test-guard.sh:72` 要求样本与暂存区相同；`.claude/agent-common.md:48`：「不做 git 写操作」；所以它刚加或刚改的样本在暂存之前一律被拒，`.claude/agents/tooling-writer.md:34`：「以及照拒绝说明也起不了的，写明命令交主 agent」兜住，但每次改样本都落这一句 | tooling-writer 的定义允许它 `git add` 样本 |
| T2-5 登记为 light 的没起重活（只抽一次，不作「没打中」用） | 站得住（去向对）；同样打中别的臂：登记表自己的写法与 C613 对不上 | 判决没拿它当「没打中」用，只拿「六件起了 `-V`」这一条正面观测改 C613：`research/prompts/test-script-align-r1-opus-output.md:157`：「cargo 版本 ×80；rustc 版本 ×80」（admission.py）。但登记表头仍写 `.claude/hooks/selftest-registry.tsv:9`：「才写 light」（它前面一句要求每个子进程里的 cargo 都「换成了假的、或根本不起」），admission.py 那一行的理由写 `.claude/hooks/selftest-registry.tsv:19`：「PATH 前面放假 cargo、假 rustc、假 herd7」，C613 现在却写 `.claude/kb/checks-owed.md:484`：「只问版本的（参数只有 `-V` 或 `--version`）不算起重活」。三处互相对不上，照 C613 落成之后登记表头会判 admission.py 违规 | 登记表头补「只问版本的除外」，六行理由改成实际起了真 `-V` |
| T2-6 定义第 7 步照字面跑不了；`case` 模式行误拒 | 站得住 | `--item` 已落：`.claude/hooks/heavy-test-guard.sh:74`：「带 `--item <阶段文件名>`（可给几次）只判点名的那几道」；`case` 误拒有自证格：`.claude/hooks/heavy-test-guard.sh:1045`：「工具实现员起带 case 模式行 .claude/gate.d/lib/*.py) 的脚本：放行」；定义改成 `.claude/agents/tooling-writer.md:34`：「以及照拒绝说明也起不了的，写明命令交主 agent」。我现跑 `lib_heavy_tests.py --selftest`（393 种）、`heavy-test-guard.sh --selftest`（932 种，该拒 175 种）退 0（`outputs/lib-heavy-tests-selftest.out`、`outputs/heavy-test-guard-selftest.out`） | 自证退非 0；或带 case 模式行的脚本仍被拒 |
| T3-1、T3-2 前一项红后一项退 78；驱动退 78 而行里报红 | 站得住 | 入口头注释 `research/scripts/crash-amplification.sh:57`：「整趟有红先判：别的项或这一项红就退 1，没有红才退 78」与 `research/scripts/crash-amplification.sh:56`：「行里报了红记 exit=78 verdict=red」；入口 `--selftest` 现跑退 0，开关轮 22 行「红在第 N 格，与文件头相同」（`outputs/crash-amp-selftest.out`）。同形的兄弟：`research/scripts/crash-amplification-lib.sh:196`：「两类都有按 device（设备用不上时那一趟判的不算数）」说设备用不上时判的不算数，入口现在反着取（红不让 78 盖掉）——偏向假红，不是假绿，可接受 | 一份驱动行 `red>0` 而驱动退 78 的假驱动喂进去，入口退 78 或记 not-run |
| T3-3 入口不读 ignore 两个字段 | 一半站得住，另一半**够不着** | 站得住的：入口头注释 `research/scripts/crash-amplification.sh:73`：「成功句末尾接「按登记整批不判：N 个点、M 态」」。够不着的：上游要求「列出没判的是哪几类」（`.claude/singlefs-ai-sop/rules/test-script-discipline.md:62`：「按登记整批不判的一类不算剪枝：登记表一行一类、写明为什么不判，成功那句报跳过了多少」），入口只能写 `research/scripts/crash-amplification.sh:73`：「驱动行只报点数与态数、不报类名」；`grep -rn ignored_points crates` 今天 3 处命中（`outputs/kb-facts.out` 第 15 段）全是 `crash_amplification.rs` 里的局部变量与变异表一行，没有一处是驱动行字段，所以入口今天读到的永远是「取不出」。C605 前置写 `.claude/kb/checks-owed.md:476`：「这一半另算，不在这一条的对照里」，`grep -n '类名' .claude/kb/checks-owed.md` 只命中 C430（无关）：「另算」没有一行接 | 欠账表里出现一行点名「驱动行补类名与登记一类一行」 |
| T3-4 缺 `red=` 判绿；全量现判 0 复用 0 判绿 | 站得住 | `research/scripts/crash-amplification.sh:48`：「有一行缺 red= 或 red 的值不是非负整数」与 `research/scripts/crash-amplification.sh:50`：「scope=full（不带参数的全量）里一项现判 0 态、复用 0 态」判红；开关轮覆盖第 26、27 格（同一份输出尾部） | 缺 `red=` 的行喂进去仍判绿 |
| T3-5、T3-7 弄坏开关与换驱动平常跑也认 | 入口与 `mutate.sh` 站得住；**同样打中别的臂**（范围比 C622 大得多） | 入口：`research/scripts/crash-amplification.sh:82`：「平常跑时带了只给自证用的变量」；`mutate.sh` 同样（头注释 `research/scripts/mutate.sh:72` 起「只供自证」一段）。别的臂没改：`research/scripts/run-with-memory-cap.sh:126`：「BROKEN_JUDGEMENT="${RUN_WITH_MEMORY_CAP_BREAK:-}"」（每次跑编译出来的代码都经它）、`research/scripts/layer0-shard-run.sh:132`：「layer0_shard_break="${LAYER0_SHARD_RUN_BREAK:-}"」、`research/scripts/verus-proof-check.py:164`：「BREAK = {item for item in os.environ.get("VERUS_PROOF_CHECK_BREAK", "").split(",") if item}」、`research/scripts/crash-amplification-lib.sh:140`：「crash_amplification_item_break_is() {」。`outputs/kb-facts.out` 第 2 段数出带 `_BREAK` 变量名的文件：门禁 16、门禁共用库 2、钩子 9、research/scripts 47。C622 的「怎么拦」写在 `.claude/kb/checks-owed.md:493`：「一格门禁扫 `.claude/gate.d/` 里读 `*_BREAK` 而不经这一判的」，只扫 gate.d、标记放在 gate.d 的共用库里，够不着 research/scripts 与钩子 | 那几份在非自证调用下设了开关就退 2；或 C622 的扫描射程写到 research/scripts 与钩子 |
| T3-6 `mutate.sh` 252/254 不判失败 | 站得住（读头注释与格表；没复跑 `mutate.sh --selftest`） | `research/scripts/mutate.sh:62`：「内存不够排不上——等满 RUN_WITH_MEMORY_CAP_WAIT_SECONDS 秒还放不下」、`research/scripts/mutate.sh:65`：「整个 slice 满了，内核在 slice 里挑了这一条杀，不是它自己撞了上限」各自归类；头注释的开关表里 `queue-full`（`research/scripts/mutate.sh:99` 一带）与 `squeezed` 两格各管一种。C612 把这一条写成「已落地、销账待核」，没有别的欠账 | 假包装退 252 / 254，整轮仍退 0 |
| T3-8 54 号不认 `cfg_attr`（潜在，记进 C604 不改） | 站得住；**同样打中别的臂**（形态相同、今天有实例） | 去向对：`.claude/kb/checks-owed.md:475`：「feature-gated-tests 格只认 `#[cfg(…)]` 与 `#![cfg(…)]`，不认 `cfg_attr`」。但这个格只扫 `tests/` 顶层（`.claude/gate.d/checker-tier-layer0-replay.sh:670`：「attribute_start = re.compile(r"#(!?)\[\s*cfg\s*\(")」），`crates/singlefs-checker-tier/src/lib.rs:11`：「#[cfg(feature = "verdict-store")]」与另外五处挂特性的模块里共有 45 条 `#[test]`（`outputs/kb-facts.out` 第 3 段数出），带特性那一趟 `--lib` 照跑，但「编进来几条」只数 `tests/` 文件（`.claude/gate.d/checker-tier-layer0-replay.sh:692`：「count_tests_run_by_file() {」）：库里挂特性的单测编进来 0 条，没有格会报 | 加一格数库单测，或 C604 射程句补「库里挂特性的模块不在这一格」 |
| T3-9 分项驱动缺值退 2 | 站得住（读代码，没跑） | `research/scripts/crash-amplification-item.sh:94`：「--expand-up-to) require_value "$1" $# "${2:-}"」 | `--expand-up-to` 缺值退 1 |
| T3-10、T3-11 只抽一次，不作「没打中」用 | 判得对；T3-11 我补一次读代码当第二次样本，同样没打穿，但看到一处「逐条挂」的空转 | `.claude/gate.d/checker-tier-layer0-replay.sh:692`：「count_tests_run_by_file() {」按文件数通过加失败的条数；整份挂特性的文件 0 条会报，逐条挂特性（`items`）而文件里另有不挂特性的用例时，数字恒大于 0，挂特性的那几条有没有编进来看不出。今天 `tests/` 里有这种文件（`record_checker_judges_absence_by_the_persisted_set.rs` 里 `#[cfg(any(feature = "verdict-store", feature = "gpu"))]`）；因为特性是整趟统一传的，编不进来的路径只剩「特性名没传」，今天不可达，记「线索」不记「打穿」 | 逐条挂特性的文件在传了特性的那一趟里仍编不进来 |
| T1-1 去重丢了并行线号的例子 | 站得住 | `.claude/rules/verification.md:18`：「`*_parallel_line_one_*`」；判它的脚本 `research/scripts/crash-case-check.py:192` 的正则还认 two、three，规则只举了 one（举例，不要求穷举） | 规则正文不再提并行线号 |
| T1-2 C607 节名截短 | 站得住 | `.claude/kb/checks-owed.md:478`：「用例原子化：每一条都能单独跑」；checks-owed.md 里点名 `test-script-discipline.md` 小节的写法共 17 处，对不上 0 处（`outputs/kb-facts.out` 第 9 段） | 副本改了节名 |
| T1-5 三处定义写法不一 | 站得住 | 同一句在三处各命中 1 次：`.claude/agent-common.md:87`、`CLAUDE.md:22`、`.claude/agents/tooling-writer.md:34`：「登记为 heavy 的、登记为 heavy-if-set 而命令里设了那个变量的、没登记的」（`outputs/kb-facts.out` 第 10 段）。残留一点：「登记为 light」不点名样本档的三种判定词 `samples-light` / `samples-light-marker:` / `samples-heavy` | 三处写法再分叉 |
| T1-3 交用户（C20「定期全量扫」与 D13「合并前」没有触发者） | 去向站得住（交用户是对的）；写回的字面**够不着**——见第三节 | 用户选了「里程碑收尾、动了崩溃点重放或判器的那一批提交之前，由主 agent 弹窗请你跑一趟全量」（`.claude/kb/decisions-history.md:6014`：「里程碑收尾、动了崩溃点重放或判器的那一批提交之前，由主 agent 弹窗请你跑一趟全量」）。写回时主 agent 另改了 C20 的尾部两句（`records/2026-10-01-对接上游测试脚本规范.md:502`：「主 agent 另改 C20 两句」），变更史列 C20 改了什么只写开头一句（`.claude/kb/decisions-history.md:6013`：「欠账表 C20（罕见布局无人跑） 怎么拦一栏的」），尾部两句没进变更史，也没写「被攻过零轮」 | 变更史补上尾部两句并标零轮 |
| D13「夜间」改「全量」无多余改动（正推判不改） | 判决时站得住；T1-3 写回之后**这一段自己打架** | 同一段里并排三句：`.claude/kb/decisions/13-验证路线.md:183`：「只能是异步准入判据」、`.claude/kb/decisions/13-验证路线.md:184`：「里程碑收尾、动了崩溃点重放或判器的那一批提交之前 = 全量档全绿（准入条件），由主 agent 弹窗请用户跑一趟全量」、`.claude/kb/decisions/13-验证路线.md:185`：「全量档红了，处置不是回退」。提交之前要全绿就是同步闸；红了不回退、带着未结的红接着提交就不是闸 | D13 这一段写清「弹窗之后用户不跑 / 跑红了，提交放不放」 |
| T4 各笔欠账的改写 | 见第四节 | 第四节逐笔 | — |

## 三、T1-3 写回后的字面：主 agent 在里程碑收尾时照着做，哪一步会做错

被走的字面：`.claude/main-agent.md:10`：「里程碑收尾、动了崩溃点重放或判器的那一批提交之前，先弹窗请用户跑一趟全量」，指向 D13 已定项 9；C20（`.claude/kb/checks-owed.md:23`）与 C409（`.claude/kb/checks-owed.md:333`）同句。

| 步 | 字面要做什么 | 照字面做会怎样（具体到命令或情形） | 证据 | 推翻它的现象 |
|---|---|---|---|---|
| 1 认出「里程碑收尾」 | 主 agent 自己认 | 没有机器可读的收尾标记：里程碑状态是正文里的一句，`.claude/kb/milestone/02-second-txn.md:5`：「状态：2026-09-29 判定结束」；CLAUDE.md 当前里程碑那一段也是自由文字。没有任何门禁或钩子读它，认不认全靠主 agent 记得 | `outputs/kb-facts.out` 第 11 段：`grep -rn '里程碑收尾' .claude/gate.d .claude/hooks research/scripts` 命中 0 行 | 有门禁或钩子读里程碑状态行 |
| 2 认出「动了崩溃点重放或判器的那一批提交」 | 主 agent 自己认 | 三种读法互相不等。a) 指意图：「判器」没有定义，`.claude/rules/verification.md:13` 里叫判器的是 GPU 判器（`crash_judge_tables`、`crash_judge_gpu`），D13 里叫判决器的是池级 checker（`.claude/kb/decisions/13-验证路线.md:320`：「**池级 checker** 是 `crates/singlefs-checker`，判决器库」），记录核对器又是 `crash`。b) 指门禁的指纹：54 号每条用例的输入是 `.claude/gate.d/stage-inputs.tsv:37`：「crash-case:layer0-first-stream crates/ Cargo.toml Cargo.lock」，整个 `crates/`，动 harness 或 core 也算。c) 指路径清单：没有登记。只动 harness 或 core 而没碰判器的一批，按 a) 读不算、按 b) 读算（指纹算的是整个 `crates/` 减去用例读不到的文件） | `outputs/kb-facts.out` 第 4 段：`stage-inputs.tsv` 里 `crash-case:` 8 行；git common-dir 里只有 1 个全绿标记（2026-09-27）；自那之后动过 `crates/` 的提交 3 个。所以 54 号快档每次提交都打 `.claude/gate.d/checker-tier-layer0-replay.sh:640`：「全量这一轮没跑」，这条提示恒亮，分不出「这批动了判器」 | 登记一张「崩溃点重放与判器」的路径清单，或把读法钉成 b) |
| 3 「弹窗请用户跑一趟全量」——哪一个全量 | 弹窗里要写命令 | 「全量」有两个所指。D13 已定项 15 定义：`.claude/kb/decisions/13-验证路线.md:320`：「**全量**是登记在 `.claude/gate.d/stage-inputs.tsv` 键为 `crash-case:` 的用例逐条 `--include-ignored --exact`」，即 54 号 `--full`；verification.md 另有一行 `.claude/rules/verification.md:35`：「`bash research/scripts/crash-amplification.sh`：不带参数跑登记表里全部分项」，CLAUDE.md:8 说的「一趟 7 小时 37 分」是这一个。两行的时机列都写「用户要求」（`.claude/rules/verification.md:34`、`:35`）。D13 已定项 9 全文不定义崩溃放量全量（`grep -n '崩溃放量' D13` 只命中 195、262 两条对用户原话的引用）。弹窗选项写哪一个、两个都跑还是跑一个，字面没给；用户选「跑」之后主 agent 照 `.claude/main-agent.md:66` 起哪一条也没给 | 同左 | D13 已定项 9 点名「全量 = 哪一个或两个」 |
| 4 弹窗顺序 | 「那一批提交之前」 | `.claude/main-agent.md:66`：「`--full` 不默认跑，用户要求时才在 HEAD + 暂存区的 worktree 里跑那棵树里的」——全量要在暂存之后、在暂存区的树上跑，而提交前那张流程行（`.claude/main-agent.md:66` 整行）没有「判是不是这两个时点、要不要弹窗」这一步，只有 `main-agent.md:10` 的「禁止」条里一句。按流程行干，没有任何一步会触发弹窗；同一条还写 `.claude/main-agent.md:10`：「这一条管枚举域，不管跑的时机」，随后接着写跑的时机 | 同左 | 流程行补一步，或 `main-agent.md:10` 把时机那句拆出去 |
| 5 用户的回答不是「跑」 | 字面没有这一支 | D13 写「准入条件」（`.claude/kb/decisions/13-验证路线.md:184`），又写「只能是异步准入判据」（`:183`）、「处置不是回退」（`:185`）。用户答「这次不跑」或全量跑红了，提交放行还是挡住，三句各给一个答案 | 同左 | 补一句 |
| 6 「上一次全量」谁记 | C20：「没跑过的那套在下一次里程碑收尾的提交之前判红」 | 要判「上次完整跑过」就要两个时间戳：每套布局上次完整跑过、上一次里程碑收尾。前者 C20 自己写 `.claude/kb/checks-owed.md:23`：「仍欠「上次完整跑过」的时间戳」；后者没有记录。机器上现成的两样都没接：54 号的逐条全绿标记（git common-dir，1 个，且记的是输入指纹不是时间）；崩溃放量入口新写的 `runs/entry-<UTC>-<进程号>/` 目录（`outputs/kb-facts.out` 第 11 段：今天一个都没有，旧写法的 51 个目录是 9 月 29 日起的，加手起名的 `full-run-*.log`）。所以「判红」那一句今天没有读数可判 | `outputs/kb-facts.out` 第 4 段；`.claude/kb/checks-owed.md:23` | 门禁读到「上次完整跑过」与「上次收尾」 |
| 7 C20 尾句 | 「这条不需要人守着」 | `.claude/kb/checks-owed.md:23`：「这条不需要人守着，阶段收尾时由主 agent 弹窗请用户跑即可」——同一句既说不需要人守着，又把动作交给主 agent 记得弹窗。C20 原来的「定时跑即可」是机器能做的形态；这一改把它换成要靠记性的形态，欠账表写进来的门槛是说得出怎么拦会红（`.claude/kb/checks-owed.md:10`：「三样缺一样就是还没想清楚，先别写」） | 同左 | 尾句改成「由门禁读时间戳判红」 |

可落成会红检查的候选（零轮，只是给主 agent 判的形态，没被攻过）：里程碑收尾的那次提交，门禁把 54 号快档里 `missing_cases` 非空从「本次未跑」改判红，并要求崩溃放量有一份与被判树、判法版本相同的全量完成记录；「这次提交是里程碑收尾」要一个机器可读的标记（例如里程碑文件里一行固定格式的状态）。没有这个标记，这一条检查仍然靠记性。

## 四、U4 表二：欠账逐笔，「怎么拦」落不落得成会红的检查

判定：能落成（给得出一个会红的输入，前置也在栏里）/ 要补一句才能落成 / 落不成（今天没有会红的输入）。行号是 `.claude/kb/checks-owed.md` 里那一行所在的行（C20=23、C409=333、C604=475 … C613=484、C620=491、C622=493 … C627=498）。补丁没合入的东西（`NOT_RUN_NO_GPU_ADAPTER`、`decide_gpu_test_start`、`states_reused`）今天 `crates/` 零命中（`outputs/kb-facts.out` 第 12 段），下面都按「补丁合入后」谈。

| 笔 | 判定 | 为什么，会红的输入或缺的那一句 | 推翻它的现象 |
|---|---|---|---|
| C604 快档不开特性 | 门禁一半已落；crates 一半**要补一句** | 门禁一半有样本 `quick-features-gpu-not-run`。crates 一半的「怎么拦」是一条要求（`.claude/kb/checks-owed.md:475`：「GPU 对拍在找不到 GPU 适配器的机器上打 `NOT_RUN_NO_GPU_ADAPTER` 并返回成功」），没有会红的输入：本机两张卡，没卡的路径跑不到，要有一个能把「没卡」强行喂进去的入口。这个入口就是 C620 的纯函数（`decide_gpu_test_start`）；而且判据冲突——C604 与 C608（`.claude/kb/checks-owed.md:479`：「没卡的机器上这几行变异记」本次未跑）用的是 Vulkan 看到的「找不到适配器」，C620（`.claude/kb/checks-owed.md:491`：「PCI 上有卡而 Vulkan 枚举不出判红」）说那种情形必须判红。照 C604 字面落成，插着卡而驱动坏了的机器报「本次未跑」，正是 C620 要拦的假绿。54 号的出路句也是同一个说法（`.claude/gate.d/checker-tier-layer0-replay.sh:804`：「GPU 对拍没跑（」…「这台机器找不到 GPU 适配器」） | C604、C608 改成「PCI 上也没卡」并指到 C620 |
| C605 复用与剪枝没有关掉再判的对照 | **要补一句**（跨趟那半）；键层变异能落成 | 跨趟那半写 `.claude/kb/checks-owed.md:476`：「再拿与 A 只差一项被键漏掉的输入 B 在同一个库上跑，与 B 关掉复用的结果逐态比，不同判红」，没写「B 的真判定要与 A 的不同」：B 与 A 判定相同时，读到 A 的判定与 B 自己判的逐态相同，键漏一项的变异存活。键层：`crates/singlefs-checker-tier/src/crash_identity.rs:93`：「pub write_table_with_barriers: [u8; DIGEST_BYTES],」与下一行共两个属性，`each_attribute_changes_the_reuse_key` 各改一次，`crates/mutations.tsv` 里点它的行 0 条（`outputs/kb-facts.out` 第 7 段），补一行变异就红。判法版本那一类已有变异（`outputs/kb-facts.out` 第 13 段：`crates/mutations.tsv` 1453、1454 两行），不用再补 | 变异表里键漏一项而 B 判定与 A 相同的输入全部判红 |
| C606 入口不写展开深度 | 能落成（crates 一半等补丁） | 会红输入：`SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=abc` 要判红（`.claude/kb/checks-owed.md:477`：「规模变量的值认不出判红、不退回默认」）；入口一半已有自证 | 变量写错仍退回默认 6 |
| C607 随机取样点用例凭种子复现不出 | 能落成 | 前置已写成有：`.claude/kb/checks-owed.md:478`：「先给四个随机二进制各加一个注入失败的入口」 | 前置栏改回「无」 |
| C608 GPU 变异不罩内核 | **要补一句** | 同 C604：没卡判据与 C620 对齐；其余（12 行、0 行 `.wgsl`）可落成，本机有卡能真红 | 同 C604 |
| C609 全量出的红没有钉表 | **落不成**（崩溃枚举一类除外） | 「怎么拦」一栏已写成一格门禁（`.claude/kb/checks-owed.md:480`：「一格门禁：全量产物里的红逐个按整行文本在钉表里找得到，找不到判红」），前置栏却写（同一行）：「落成之前先定两件」——判哪一份产物、变异整表与 QEMU 的红出红的那一行是什么。这两件没定，门禁没有输入可读。把「先定两件」挪成「怎么拦」的第一步，E165 一类先落 | 两件定了、各一份会红的产物样本 |
| C610 测试文件头还写着来历 | 能落成；**要补两句** | 结构判据能落：给任一测试文件头加一行带日期而无 kb 指路的 `//!` 就红（`.claude/kb/checks-owed.md:481`：「同一行没有 kb 指路就判红」）。要补：① 行里的数没给命令，我拿近似正则数出带日期的 `//!` 行 89 行 / 66 份、无 kb 指路 70 行 / 52 份，行里写 87 / 65、68 / 51（`.claude/kb/checks-owed.md:481`：「带日期的 `//!` 行 87 行、在 65 份文件里」），对不上，别的会话在加测试文件；② 结构判据放过「有 kb 指路、又写用户何时定」的混合行，我数出 18 行，例 `crates/singlefs-checker-tier/tests/crash_enumeration_record_spill_over_stream.rs:3`：「已定项 17（用户 2026-09-24 定）」。第一轮判决把这类算作指路不是来历（T4），上游规则是「来历进 commit message 或项目 kb」，混合行里「用户何时定」仍是来历，这一条没人定过，零轮 | 用户或三方定混合行算不算来历 |
| C611 长跑的项不按挂钟打进度 | **要补一句** | 一句话的拦法（`.claude/kb/checks-owed.md:482`：「按挂钟每 5 分钟至少一行」）只有 Python 有现成形态：`research/scripts/e165-replay-mutation-kill-bounds.sh:1083`：「Progress(3, PROGRESS_INTERVAL_SECONDS, clock=lambda: clock[0], sink=sink)」。栏里点名的 Rust 发射点要先改成能被时钟驱动，bash 阶段（QEMU 一格、crates 变异整表一格）没有对应形态：要每处各写一条 | 每个点名的处各有一条假时钟用例 |
| C612 变异自证把包装排队算进限时 | 能落成（已落，检查就是 `mutate.sh --selftest`）；**要换行号** | 栏里九处行号里八处对不上：`.claude/kb/checks-owed.md:483`：「第 135 行 `selftest_time_limit=2`」，今天在 `research/scripts/mutate.sh:154`：「local selftest_time_limit=2」；其余七处同样漂（`outputs/kb-facts.out` 第 1 段逐条列「行里写的 → 实际」），只有 `research/scripts/run-with-memory-cap.sh` 第 87 行还对。同一份脚本在这一轮里一直在改，行号写进 kb 几小时就烂 | 行号换成能 grep 的名字 |
| C613 自证登记表靠读代码判轻重 | 能落成；**要补三句** | 形态对（跟踪起进程的系统调用，不靠采样；`.claude/kb/checks-owed.md:484`：「只问版本的（参数只有 `-V` 或 `--version`）不算起重活」）。要补：① 红例写成「`--selftest` 里起 `~/.cargo/bin/cargo build`」（同一行），真跑它就真编；红例要用立刻失败的参数（例如指到不存在的清单），否则造红例本身起重活；② 检查在 execve 之后才红，起了的那一步照跑：自证若被改成真跑 checker 档用例，红是在它跑完之后才报；我试了 strace 6.8 的 `-e inject=execve:error=EPERM:when=2` 让第二次 execve 失败，没成（`bash -c '/bin/true'` 里的子进程照样 `= 0`），要拦在起之前得换办法，这一条是推的；③ 与登记表头矛盾，见表一 T2-5 | 同 T2-5 一行，加：一份被改成真编的自证在检查里被拦在编之前 |
| C620 GPU 对拍认没卡只看 Vulkan | 能落成（补丁合入后） | 纯函数单测喂「PCI 有卡、Vulkan 零个」判红，是现成的会红输入；本机核得对：厂商号 0x10de 的 4 个设备里 2 个 class 0x030000、2 个 class 0x040300（`outputs/kb-facts.out` 第 12 段），栏里写的 class 前缀 0x03 过滤是必要的 | 单测喂这一组输入不红 |
| C622 门禁的弄坏开关平常跑也认 | **要补一句**；射程也要补 | 「怎么拦」两半：静态扫描（`.claude/kb/checks-owed.md:493`：「一格门禁扫 `.claude/gate.d/` 里读 `*_BREAK` 而不经这一判的」）能落成，会红输入是一份阶段文件里直接读 `$X_BREAK`；共用库标记那一半没写谁来设它：门禁阶段的样本由上游 `stage-selftest.sh` 喂（我们不改上游），「开关 → 样本」的 runner 在 `.claude/gate.d/code-tooling.sh`（C625 自己写的：`.claude/kb/checks-owed.md:496`：「本仓 `.claude/gate.d/code-tooling.sh` 管的是门禁阶段的「开关 → 样本」」），多半由它设，要写明。射程：只扫 gate.d，research/scripts 47 份与钩子 9 份没有人扫（表一 T3-5、T3-7 一行） | 扫描射程含 research/scripts 与钩子，或写明它们不是门禁、不判 |
| C623 取不出复用数没有棘轮 | **要补一句** | 栏里「驱动行合入 `states_reused=` 之后…取不出复用数判红」（`.claude/kb/checks-owed.md:494`：「棘轮：驱动行合入 `states_reused=` 之后」），「合入之后」是一个时间点，入口怎么知道已经合入没有写：没有开关，驱动行少打字段与「补丁还没合」分不开。两种写法都能落：入口里一个常量在补丁合入时翻开，或入口读 `crates/` 里有没有 `states_reused` 这个串（今天 0 处）决定。会红输入：翻开之后造一项驱动行少 `states_reused=` 的假驱动 | 栏里写明谁、什么时候翻 |
| C624 一律进快档的几类没逐类核 | 能落成（前置是登记表，今天没有） | 前置栏已写。与 C604、C605、C608、C620 重一部分对象（剪枝自证、加速路径对拍）：它们各自有用例级的拦法，C624 只管登记与档位，建议栏里写这一句，免得登记表与用例级各记一份 | 登记表里的类与那几行的拦法对不上 |
| C625 弄坏开关文件头三样没有门禁 | **要补一句**（统一写法） | 「缺一样判红」要先有一种能解析的写法；今天至少三种：`research/scripts/crash-amplification.sh:89`：「swallow-exit」那张表（开关名、弄坏哪一处、第 N 格）、`research/scripts/mutate.sh:78`：「MUTATE_BREAK=memoryascaught  撞顶记成抓到」（开关加一句话，红的格另写在后面的段里）、`.claude/hooks/lib_heavy_tests.py:153`：「selftest-heavy-rows-as-light（登记成 heavy 的行当 light 判）」（名字、一句话、点名的格标题与条数）。第二半（`.claude/kb/checks-owed.md:496`：「登记为 light 的自证逐个打开开关，红的项与文件头点名的不等判红」）入口自己的自证已经在做：`research/scripts/crash-amplification.sh:88`：「--selftest 末尾那一轮从这张表现读，逐个打开、核对红的格与这里写的相等」，门禁那一格应复用这个写法，不另立 | 栏里先定一种写法，会红输入是一份缺第三样的文件头 |
| C626 走 GPU 的计算类没有登记 | 能落成 | 前置栏已写（先改读配置与选路的代码） | 登记与代码对不上而门禁绿 |
| C627 恢复重放钉表整行抄只落了装置那一半 | 能落成 | 前置栏已写，并写了补丁打上之后钉表补全之前那一格会红的窗口 | 补丁与钉表之间没有这个窗口 |
| C20、C409 | 见第三节 | C20 的会红形态依赖尚欠的时间戳与收尾标记；C409 的「怎么拦」只管层 1、层 2 两个阶段，新的两个时点没有行 | — |

### 新记的几笔有没有和已有的重

| 新记 | 和谁重叠 | 判定 |
|---|---|---|
| C622 | C612（`MUTATE_BREAK` 平常跑退 2 也写在那里）、C625 | 一半重 C612 已落的部分，一半是新的（gate.d 与别的臂），不重 |
| C623 | C606（入口一半）、C605 | 不重：C606 管展开深度，C623 管复用数字段 |
| C624 | C605、C604、C608、C620 | 对象有重叠，层级不同（登记表 vs 用例）；建议栏里写分工 |
| C625 | C622、C46（门禁样本） | 不重；与入口自证里已有的逐个打开开关重 |
| C626 | C620 | 不重：C620 是没卡判据，C626 是计算类登记 |
| C627 | C609 | C609 前置已指向 C627，不重 |

我用 `awk` 按「要拦什么」「怎么拦」两列扫了「弄坏开关」「钉表」「复用数」「快档」「加速硬件」「平常跑」六个词（`outputs/kb-facts.out` 第 14 段）：除了新记几笔与 C609、C612，命中的旧行是 C476、C504、C511、C599（「快档」在别的题里）与 C633（形式化一道的另一件事），没有第二份同题的旧行。

## 五、U4 表三：改钩子那条流程会不会拦死整个会话

流程的字面（`records/2026-10-01-对接上游测试脚本规范.md:508`）：「主 agent 定不开草稿口子，流程为草稿里语法检查加只调判定函数的探针、换上后立即跑自证、红了马上换回原样，进第二轮审」。定义里只有一句语法检查：`.claude/agents/tooling-writer.md:34`：「语法检查（`bash -n`、`python3 -m py_compile`）照做」，没写次序，也没写这一流程。换上用 `research/scripts/replace-once.py`（`.claude/agents/tooling-writer.md:30` 一带），每次一处、每处原子改名，整批不是原子的。

我在草稿里对重型测试闸造了四种坏法，另对派发闸造了一种，拷本仓现在的 `heavy-test-guard.sh`、`lib_heavy_tests.py`、`lib_shell_words.py`、`selftest-registry.tsv`，只把 JSON 从标准输入喂钩子、看退出码，被判的命令一条都不起（`probe-hook-failures.sh`，原样输出 `outputs/hook-failures.out`；下表里 stderr 首行的草稿路径已换成 `<草稿>`）：

| 坏法 | 交回前的三种静态检查看得出吗（`bash -n` / 编译 lib / 抽出 .sh 里内嵌 python 再编） | 实测结局（喂 `cargo test --all` 与 `ls -la`） | 会话后果 |
|---|---|---|---|
| lib 语法错 | 只有「编译 lib」看得出 | 退 0，stderr 写「读不到共用模块」，记检出 | 钩子整个失效（连 `cargo test --all` 也放行），主 agent 看得到检出 |
| 调用点与被调函数改得不同步（缺名字） | 三种都看不出 | 只在走到那条路径的命令上退 0、stderr 写「没判成」；别的命令照判 | 那一条路径静默失效，**不记检出**（`.claude/hooks/heavy-test-guard.sh:1470`：「没判成」那支只打 stderr 就 `return 0`） |
| 钩子 .sh 里内嵌的 python 语法错 | 只有「抽出内嵌 python 再编」看得出；`bash -n` 与 lib 编译都过 | 退 1，什么都没记 | 钩子整个静默失效：按 Claude Code 钩子的退出码约定只有 2 拦、别的非 0 只报错不拦（没在真会话里喂过，这一句是约定不是实测）（`.claude/hooks/heavy-test-guard.sh:163`：「python3 /dev/fd/3」把整段 python 当程序喂，语法错就整个起不来） |
| 钩子 .sh 的 bash 层语法错（尾部多一行 `if true; then`） | 只有 `bash -n` 看得出 | **退 2，连 `ls -la` 也退 2**，主 agent 与子 agent 一样 | **整个会话所有 Bash 调用被拦死**，包括「换回原样」那一条 Bash 命令本身（bash 解析语法错的退出码就是 2，`outputs/hook-failures.out` 里 bash-tail-syntax 一段） |
| 挂 Agent 与 Task 的派发闸 `runner-dispatch-guard.sh` 同样的 bash 层语法错 | 只有 `bash -n` | 一条合规派发退 2（`outputs/dispatch-guard.out`） | **所有派发被拦死**，连派去修它的 tooling-writer 也派不出 |

这张表读出来的是：钩子被设计成坏了就放行（`.claude/hooks/heavy-test-guard.sh:181`：「文件不在、语法错、导入时抛的都算读不到」，`.claude/hooks/runner-dispatch-guard.sh:546`：「挂在每一次派发上的闸，坏了不能把所有派发都挡住」），所以大多数坏法的后果是**静默失效**而不是拦死；拦死只出在 bash 层解析不过这一类，而那恰好是 `bash -n` 看得出的一类。流程的几处缺口：

1. **「红了马上换回原样」这条 Bash 命令本身会被拦。** bash 层语法错时每条 Bash 调用都退 2（`heavy-test-guard.sh` 与 `bash-command-detector.sh` 都挂在 Bash 上），`replace-once.py`、`mv` 都出不去。恢复只剩 Edit / Write：写闸只拒「整份覆盖仓里已存在又没进 git 的文件」（`.claude/agent-common.md:78`：「Write 整份覆盖仓里已存在又没进 git 的文件」），钩子文件进了 git，不在被拒之列，tooling-writer 的写范围含它们（`.claude/agents/tooling-writer.md:39`：「`.claude/hooks/**`、`.claude/gate.d/**`」）。这条路定义里没写。
2. **「原样」是开工快照，别的会话也在改这几份文件。** 第一轮判决写过：`research/prompts/test-script-align-r1-main-verification.md:8`：「三份对不上——`.claude/agents/crash-verifier.md`、`.claude/hooks/lib_heavy_tests.py`、`research/scripts/verus-proof-check.py`，都是别的会话在腿跑着时改的」；「别碰」只是对另一个会话发的一句话（`records/2026-10-01-对接上游测试脚本规范.md:508`），没有机制。整份换回开工快照会盖掉别人在这段时间里的改动。
3. **换上不是原子的。** 一批多处替换逐处换上，每一处是原子改名，整批不是；半成品状态正是上表第二行（调用点与被调函数不同步）：静默失效、不记检出。
4. **探针只罩得住想到的路径。** 上表第二行三种静态检查全过；要靠探针喂到那条路径，或靠自证（932 种）。自证本身只要几秒（`rerun.sh` 打出两份钩子自证合计 1.28 秒；我手工量 `heavy-test-guard.sh --selftest` 一次是 4.07 秒），但「换上到跑绿」的窗口是多处换上的工具调用次数乘每次的间隔，不是这几秒。
5. **定义里只有一句语法检查，没有次序，也没写内嵌 python。** `.claude/agents/tooling-writer.md:34`：「语法检查（`bash -n`、`python3 -m py_compile`）照做」放在「交回前」那一步里，换上之后再做也算做了；`py_compile` 是对 `.py` 的，`.sh` 里内嵌的那一大段 python 它不管，上表第三行因此漏过去。

**草稿里直接自证这条路：不用开口子，但也走不通**（`outputs/guard-spot-checks.out` 第 A 段、`outputs/flat-draft.out`）：路径尾部写成 `.claude/hooks/` 的草稿根，子 agent 与不带前缀的主 agent 喂 `lib_heavy_tests.py --selftest` 都退 2，主 agent 带 `SINGLEFS_HEAVY_TESTS=commit` 退 0（我的类型喂不了真自证，所以草稿根里自证能不能跑到绿**没验**）；路径尾部不是 `.claude/hooks/` 的平目录钩子不拦，但自证要从上两层找仓根与 `.claude/singlefs-ai-sop/`，平目录里 `lib_heavy_tests.py --selftest` 退 1、`heavy-test-guard.sh --selftest` 退 1（找不到 preflight.sh）。所以 tooling-writer 报告说「草稿副本里没能跑自检」属实，但拦它的是路径尾部，不是自证本身做不成。

**更稳的做法（推的，被攻过零轮）**：
- **先验后换，做成一个脚本，不靠流程记性**。仓里有同形先例：`research/scripts/compile-then-swap.py`（`research/scripts/compile-then-swap.py:4`：「先编后换：草稿目录里改好的入库装置」，`research/scripts/compile-then-swap.py:24`：「编不过、目标被改过、参数不对，都在换上之前停下」），副本上先判、核主工作区那份在这段时间里没被别人改、原子改名换上、回读。钩子版同样三步：① 草稿根上跑 `bash -n`、抽出内嵌 python 再编、编 lib；② 喂一组固定 JSON（良性命令退 0；重型命令退 2；stderr 不含「没判成」「读不到共用模块」；退出码不是 1），这一组正是上表五种坏法各自的会红输入（`probe-hook-failures.sh`）；③ 核现在的文件 sha 与开工快照一致才换，不一致停下报告，一次换一个文件、每换一个重喂一遍。这个脚本只喂 JSON、不起重活，可以登记成 light，任何 agent 能跑，不开任何口子。
- **回滚只做反向替换，不整份覆盖**：换之前存下每一处的新旧串，回滚前核当前文件 sha 等于自己换上之后的 sha，不等就停下报告，不盖别人的改动。
- **定义里写三句**：钩子 .sh 换上之前 `bash -n` 并抽出内嵌 python 再编；bash 层坏了的恢复走 Edit / Write；回滚用反向替换。这是改定义，要走定义三方。
- 草稿根里自证（主 agent 带前缀）作为备选，代价是借用「commit」这个前缀去做不是提交的事，且我没验它能跑绿，不推荐先选它。

## 六、复跑

```
bash research/prompts/test-script-align-r2-sonnet-model/rerun.sh /tmp/claude-1000/<新草稿目录>
```

`rerun.sh` 依次起 `probe-kb-facts.sh`、`probe-hook-failures.sh`、`probe-dispatch-guard.sh`、`probe-guard-spot-checks.sh`、`probe-flat-draft.sh`，再各跑一次两份钩子自证与入口自证（登记为 light、本仓路径、经 `capped.sh 8`），各写一份输出到草稿目录，退出码都是 0 才算跑完；喂钩子的探针只读本仓、只把 JSON 喂钩子，被判的命令一条都不起。我跑过一遍：九行退出码全 0，输出与入库的 `outputs/` 逐项相同（只有草稿路径的写法不同）。

## 七、没做什么

- 没判 U1、U2（攻方的格）：钩子收紧后的放行面逐条攻、入口与 `mutate.sh` 的假绿假红，我只在 T3-1、T3-2、T3-4、T3-6、T3-9 上读过头注释与代码，T3-6 没复跑 `mutate.sh --selftest`，也没复跑 54 号样本档的自证。
- 没跑任何重型测试；`crash-amplification.sh --selftest`、`lib_heavy_tests.py --selftest`、`heavy-test-guard.sh --selftest` 是登记为 light 的，在本仓路径上各跑了一次。
- 没读 `crates/` 补丁（`/tmp/claude-1000/impl-*/patch/`）；凡是补丁里才有的东西都标了「今天零命中」。
- 草稿根里自证能不能跑到绿没验（我的类型被拒）；strace 注入 `execve` 失败试了一次没成，那一句是推的。
- 没判 T3-9 的运行时退出码（只读了代码）；没复核 C610 的「用户…定」混合行算不算来历（只数了行、给了例，政策没人定过）。
- 每一格只抽样一次：探针的结局可以复跑（`rerun.sh`），判读是我一次读的；没有派第二条同立场腿。
- 没替主 agent 撤销或维持判决：上面只是复核结果。
- 没写 kb、规则、定义、钩子或脚本；写过的只有本报告与模型目录（这一轮我手里没有 Write 与 Edit，报告与模型文件都是 Bash 排他新建，改自己的报告用 `replace-once.py`）。草稿目录 `/tmp/claude-1000/test-script-align-r2-sonnet/` 里是几份钩子文件的拷贝与符号链接（没有编译目录、没有仓副本、没有工作树），交回前删。
