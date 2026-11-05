# d137-ledger-defs-r1 正推腿报告（格 F）

立场：从被判句子的原文推到「照这一句做会发生什么」，与对照文件此刻的文字逐处比。不看攻方腿的产出。没往统一任务服务交任务；需要起命令才核得了的写在「要主 agent 跑的命令」一节。

## 0. 被判文件此刻的 sha256（前 8 位）

快照 `research/prompts/d137-ledger-defs-r1-snapshot/files-sha256.txt` 的 11 份，此刻与快照逐份对得上（没有一份变过）：

| 文件 | sha256 前 8 位 |
|---|---|
| `.claude/agent-common.md` | b4ee6573 |
| `.claude/agents/clerk.md` | 44c8ae8b |
| `.claude/agents/experiment-designer.md` | 9f1edf3a |
| `.claude/agents/experiment-runner.md` | 467bf57a |
| `.claude/agents/implementation-writer.md` | 70ab774a |
| `.claude/agents/mutation-triage.md` | b60ebc1c |
| `.claude/agents/three-way-attack.md` | a4260bda |
| `.claude/agents/tooling-writer.md` | dcd6f8ef |
| `.claude/rules/verification.md` | 0072bcc9 |
| `.claude/rules/implementation-workflow.md` | 262d8682 |
| `.claude/skills/crash-test/commit-stages.md` | 21973562 |

对照与旁证文件（快照没登记，此刻值）：`scripts/gate/admission.py` 15b28dc4；`scripts/gate/merge-and-verify.sh` bdf206f6；`.claude/gate.d/doc-experiments.sh` 96c46f07；`.claude/gate.d/code-tooling.sh` 45de49a4；`scripts/crash/crash-case-check.py` 2ce57160；`scripts/mutation/mutate.sh` 1e064d0d；`.claude/gate.d/checker-tier-crash-point-replay.sh` d19991e5；`.claude/rules/mutation-sampling.md` 0ebfe2eb；`CLAUDE.md` 364f6f5a；`crates/singlefs-checker-tier/Cargo.toml` 6c3d6cb3；`.claude/main-agent.md` 439b97be；`.claude/hooks/runner-dispatch-guard.sh` acde6287；`scripts/hosts/compile-then-swap.py` e9d0288c；`scripts/mutation/crates-mutation-rows.py` 开工时 63f49976、写报告期间被别的会话改成 4a31ff70（见条 4.2）；`crates/singlefs-checker-tier/src/crash_amplification.rs` 672e599d；`crates/singlefs-checker-tier/src/crash_state_enumeration.rs` 80c2d27a；`crates/singlefs-checker-tier/src/judging_units_entries.rs` 00f4508f；`crates/singlefs-checker-tier/src/crash_point_replay_provenance.rs` 8e54716a；`.claude/gate.d/crates-mutation-light-rows.tsv` e4e0042d；`.claude/singlefs-ai-sop/rules/pushback-discipline.md` a182572c；`.claude/hooks/agent-write-scope.tsv` 8417ae19。

改动的原样来源：`~/.local/share/singlefs/work/d137-b19-definitions/this-batch.diff`（背景材料说 134 行 sha256 前 8 位 2a04697b；我只读，逐对 -/+ 行做了字符级比对），`commit-stages-main.diff`（20 行）。改动内容与此刻六份定义文件里的文字逐处对得上。

## 一句话总表（详见各条）

| 条 | 结论 |
|---|---|
| 1 一百二十六 ①② | 对 `cargo run` / `cargo test` 起法成立；对入库装置那一支不成立（定义教的做法里没有哪一步能带上 gpu），见 1.3 |
| 2 一百三十二 | 删“不存在小节”成立；「五处」写成对每个新用例都要五处，不成立（观察者表一处、轻型行一处是有条件的），见 2.2 |
| 3 一百三十 | 三份规则字句与门禁成立；`commit-stages.md` 第 19 行的重跑出路漏一格，不成立，见 3.2；`main-agent.md` 第 32 行还在教两格，见 3.3 |
| 4 一百六十六 ⑨ | 对 `mutate.sh` 与规则成立；对门禁：HEAD 成立，工作区在途改动把「清场没过没跑」行删了，定义三处还写着，见 4.2 |
| 5 清变量一带 | 删句成立；同一处「分两种出路」与派发闸此刻的三条出路对不上，见 5.2 |
| 6 read-once 四篇 | 路径、存在性、门禁判据成立；`pushback-discipline.md` 要求写 `.claude/warnings/`，攻方腿写不了，核不动该不该点名，见 6.3 |

## 条 1：一百二十六 ①②（experiment-designer.md、experiment-runner.md）

被判句子：
- 设计员：「登记里起它的 `cargo run` / `cargo test` 一律带 `--features gpu`」（`.claude/agents/experiment-designer.md:31`，sha 9f1edf3a），后接「整条写成 `task-server/task submit -- <命令>`，不写清场包装（清场由服务做）」与「判据里写一条「产物里 `grep -c NOT_RUN` 得 0」」。
- 执行员：「`grep -c NOT_RUN <产物>` 得 0」（`.claude/agents/experiment-runner.md:40`，sha 467bf57a），前面是「起它的 `cargo run` / `cargo test` 带 `--features gpu`」。

### 1.1 特性名、判据范围、点名的门禁格　成立

- 特性只剩 `gpu`：「gpu = ["dep:wgpu", "dep:pollster"]」（`crates/singlefs-checker-tier/Cargo.toml:39`，sha 6c3d6cb3）。`grep -n '^default' crates/singlefs-checker-tier/Cargo.toml` 零命中。账本原写的 `verdict-store,gpu` 没照抄是对的。
- 判段的认法两边一字不差：「JUDGE_ENTRY_MARKERS="crash_amplification:: crash_state_enumeration::"」（`.claude/gate.d/doc-experiments.sh:2910`，sha 96c46f07）；定义写的「源文件里有 `crash_amplification::` 或 `crash_state_enumeration::`」与它是同一个集合。
- 格名 preregistration-judge-features 在门禁里存在：`.claude/gate.d/doc-experiments.sh:28` 的 gate-cell 行。
- 特性写法：门禁认 `--features`、`--features=`、`-F`，且 `feature == "gpu" or feature.endswith("/gpu")` 或 `--all-features`（`.claude/gate.d/doc-experiments.sh:3068`）。定义的 `--features gpu` 在其中。
- 定义比门禁严：门禁只判点名 `--bin` 的命令（没点名的只报数）；定义要求走判段的装置所有 `cargo run` / `cargo test` 都带 gpu。照定义做不会撞拒，多出来的只是没人判。
- 清场：「凡用 GPU 先清场，跑完不复原」「调用方经 `task-server/task submit -- <命令>` 交，不在外面套清场」（`.claude/rules/verification.md:63`，sha 0072bcc9）。「清场由服务做」对得上。门禁自己的出路句也是这个意思：「经 task-server/task submit 交，清场由服务做」（`.claude/gate.d/doc-experiments.sh:3088`）。

推翻它的现象：门禁 `--check preregistration-judge-features` 对一份按定义写的登记报红；或 `Cargo.toml` 的 `[features]` 里 `gpu` 被改名。

### 1.2 `grep -c NOT_RUN` 数到的行　成立（两种行都数得到），一处措辞与门禁出路句不同

- 判段没跑时同一个函数打两行：`CRASH_AMPLIFICATION_NOT_RUN step=judge …` 与 `gpu_not_run_line` 的 `NOT_RUN_NO_GPU_ADAPTER …`（`crates/singlefs-checker-tier/src/crash_amplification.rs:1499-1510`，sha 672e599d；头常量「pub const CRASH_AMPLIFICATION_NOT_RUN_LINE_PREFIX: &str = "CRASH_AMPLIFICATION_NOT_RUN";」在 `:1523`）。打印用 `println!`，进标准输出（`:17512`「println!("{line}");」）。崩溃枚举那一侧只打 `NOT_RUN_<原因>` 一行（`crates/singlefs-checker-tier/src/crash_state_enumeration.rs:187`）。所以 `grep -c NOT_RUN`（不带下划线）两族都数得到，定义括号里写的「`NOT_RUN_<原因>` 行与 `CRASH_AMPLIFICATION_NOT_RUN` 行都算」属实。
- 触发条件：「(gpu_is_switched_on && !gpu_feature_is_compiled_in).then(」（`crates/singlefs-checker-tier/src/crash_amplification.rs:1406`）。此刻主机配置「ENABLE_GPU=1」（`multi-host.env:28`），所以此刻这台机上不带 gpu 编的判段二进制一定打这两行。
- 措辞差：门禁出路句写「登记里写核 NOT_RUN_ 0 行」（`.claude/gate.d/doc-experiments.sh:3088`），带下划线；`CRASH_AMPLIFICATION_NOT_RUN step=judge` 里 NOT_RUN 后面是空格，`grep NOT_RUN_` 数不到它。设计员定义不带下划线是对的；照门禁句写会少数一族，但同一函数同时打的 `NOT_RUN_NO_GPU_ADAPTER` 还在，计数仍非 0，不漏判。门禁那句建议主 agent 顺手改成不带下划线（不在本轮判的文件里，不替你改）。
- 数到无关行的风险：我在 `crates/singlefs-checker-tier/src/bin/` 与 `scripts/experiment-apparatus/` 里 grep `NOT_RUN`，打到产物里的只有上面这两族；其余是常量名、退出码名与注释（例 `crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs:403` 的 `EXIT_CODE_NOT_RUN`，不进产物）。小写的 `not_run=` 判决字段不被大写 `NOT_RUN` 匹配。核到的范围：这两个目录的 `.rs`；产物文件里手写的文字没逐份看。
- 小边角（不构成不成立）：`grep -c` 数到 0 时打 `0` 但退出码是 1，放进 `&&` 链会让整条命令退 1；空产物也数得 0（空转成绿）。执行员 4b 先按「臂 × 历史 × 几何」数结果行，空产物过不了 4b，所以不漏。

### 1.3 入库装置那一支：定义教的「带 gpu」没有落点　不成立（对这一支）

- 走判段的装置全在入库装置里：`grep -rln 'crash_amplification::\|crash_state_enumeration::' scripts/experiment-apparatus` 零命中；`crates/singlefs-checker-tier/src/bin/` 下命中 9 份（crash_library_reader、e156、e158、e161、e168、e170、e171 两份、e184）。所以这两句话实际管的只有入库装置。
- 执行员对入库装置的做法：「入库装置一律不由执行员起」，续派后「执行员交回的产物命令只执行这一份（写成 `task-server/task submit -- <二进制…>`）」，那一份是 `<草稿目录>/compile-then-swap/target/release/e<号>_<英文名>`（`.claude/agents/experiment-runner.md:30`，sha 467bf57a）。
- 那一份二进制怎么编的：「"cargo", "build", "--offline", "-p", package, "--bin", bin_name, *profile」（`scripts/hosts/compile-then-swap.py:300`，sha e9d0288c），没有特性参数；`grep -c -e features -e gpu scripts/hosts/compile-then-swap.py` 输出 0。`Cargo.toml` 没有默认特性。
- 推出的结果：照 4c 之前的做法走完，这份二进制不带 gpu；此刻 `ENABLE_GPU=1`，它跑判段必打 NOT_RUN 两行；执行员 4c 新句「不满足的那一份按本次未跑报，不当绿」就会让这一支永远报未跑。设计员那一半（登记里写 `cargo run … --features gpu`）能过门禁，但执行员按 1b/30 的做法并不执行那条 `cargo run`。两句各自成立，合起来这一支走不通，而且定义里没有一句说「编这份二进制时怎样带 gpu」。
- 没做的：没实际跑 `compile-then-swap.py` 看它编出什么；读码推。实验里已有的先例（E184 等）怎么跑出的我没查产物。
- 推翻它的现象：主 agent 找到一次入库装置判段实验、产物里 `grep -c NOT_RUN` 为 0 且二进制来自 `compile-then-swap.py`；或 `compile-then-swap.py` 此刻另有带特性的路径（我 grep 到的是 0 处）。
- 同类的窄点：定义里「（源文件里有 …）」指装置自己那份源文件；门禁明说「只读 bin 的那一份源文件」（`.claude/gate.d/doc-experiments.sh:2901`），判段入口写在 bin 引的别的模块里的，门禁与定义同样看不见。两边一致，记作共同的盲区，不是冲突。

## 条 2：一百三十二（implementation-writer.md 第 16 行与第 7a 步）

被判句子：「它要登记五处，报告里逐处给出行原文」（`.claude/agents/implementation-writer.md:47`，sha 70ab774a），逐处是 stage-inputs.tsv 的 `crash-case:` 行、崩溃点重放合表、轻型变异登记表、`OBSERVER_ENTRIES`、`PROVENANCE_OF_THE_FLOWS`。

### 2.1 删掉指向不存在小节的那半句　成立

- `.claude/rules/verification.md` 的二级标题列表（`grep -n '^#'`）里没有「崩溃枚举用例住哪、怎么登记」。
- 在 `.claude/agents/`、`.claude/rules/`、`.claude/skills/`、`CLAUDE.md`、`.claude/singlefs-ai-sop/rules/` 里 grep 这一串零命中；仓里剩下的命中是 `.claude/gate.d/harness-model-differential-and-scenarios.sh:9`、`.claude/gate.d/checker-tier-crash-point-replay.sh:921` 的普通行文（「崩溃枚举用例住哪、登记没有」，不是当小节名引），和 `records/` 下的历史记录。别处没有还在教旧指向的。
- 工具员 `result.md` 说 `scripts/crash/crash-case-check.py` 文件头第 25 行也指向不存在的小节：我对不上。该行引的是「门禁管哪一半」崩溃枚举用例住在 checker 档那一行（`scripts/crash/crash-case-check.py:25`，sha 2ce57160），而 `.claude/rules/verification.md:139` 的表里确有这一行（「崩溃枚举用例住在 checker 档、标了 `#[ignore]` 的登记进崩溃点重放合表」）。这条不影响被判句子，只是 `result.md` 的那句说多了。
- 删完上下文：第 16 行括号「（测试文件按测什么起名、一条用例一个场景、checker 档测试文件第一行声明模块）」留着，但它说的三件事里，剩下的三节只有前两件有出处：起名在 `.claude/rules/verification.md:20`，一条用例一个场景在 `:29`；「第一行声明模块」在 verification.md 里 `grep -n '第一行'` 零命中，「声明模块」只出现在没被点名的「门禁管哪一半」表里（`:139`），格式只在 `scripts/crash/crash-case-check.py` 文件头 modules 一段。读通顺，括号里的第三件现在没有指到的出处。严重度低：7a 要求交 `--only placement,modules,provenance`，modules 一样会替它判。

### 2.2 「五处」的数　不成立（对每个新用例一律五处这一读法）

逐处对门禁：
1. `.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行 + 2. 崩溃点重放合表：placement 判，且只判标了 `#[ignore]` 的；7a 本句已要求标 `#[ignore]`。成立。判法原文：「不标 #[ignore] 的是 checker 档的快档」不要求登记（`scripts/crash/crash-case-check.py:32`）。
3. 轻型变异登记表：表头写明「改坏对象不在核心代码下的行不登记，它们的测试目标也不要求登记」（`.claude/gate.d/crates-mutation-light-rows.tsv:5`，sha e4e0042d）。定义里「要补的那一行（挑法在那份表的文件头）」已带条件，成立。
4. `OBSERVER_ENTRIES`：登记的是观察者，不是用例；「没登记的观察者取键拒算」（`crates/singlefs-checker-tier/src/judging_units_entries.rs:135`，sha 00f4508f）。读 `.claude/gate.d/stage-inputs.tsv` 的 47 个 checker 档崩溃用例测试目标，`OBSERVER_ENTRIES` 里有 `target: "test:<目标>"` 的是 30 个，另 17 个一行都没有（命令见第四节第 3 条）。一个不自带观察者的新用例照 7a 逐处交「行原文」，这一处没有行可交，实现员要么编一行要么写“没有”，句子没给这条出路。
5. `PROVENANCE_OF_THE_FLOWS`：provenance 判法写明只有「写着 enumerate_crash_states 的」测试目标才要 `enumeration_of_the_item` 行（`scripts/crash/crash-case-check.py:55`）；放量分项那一行本句已写「每个放量分项」带条件。7a 句首括号把「自己逐个造崩溃状态（名字带 `every_crash`……）」也算崩溃枚举用例，这类不调 `enumerate_crash_states` 的用例，provenance 不要求 `enumeration_of_the_item`。

归结：第 4 处无条件、第 5 处的 `enumeration_of_the_item` 半句无条件，而门禁对这两处是有条件的。照字面做不会撞拒（多交了没人拒），但会逼实现员为不需要的位置造行，也使「五处」成了错的计数。建议主 agent 改成「最多五处」并给 4、5 各一句条件。

其余核对：
- 命令 `python3 scripts/crash/crash-case-check.py --only placement,modules,provenance`：三项都是该脚本认的项名（`scripts/crash/crash-case-check.py:6-13`），provenance 一项「还没有门禁调它；主 agent 照实现员报告交 --only provenance」（同文件 `:13`），与 7a「由主 agent 交」一致。
- 写范围：实现员写 `crates/**`（`.claude/hooks/agent-write-scope.tsv:8`，sha 8417ae19），`.claude/gate.d/**` 只有 tooling-writer（同文件 `:30`）。定义说「`crates/` 下那两处自己写，`.claude/gate.d/` 下那三处由主 agent 或 tooling-writer 登记」对得上。
- 行号：`OBSERVER_ENTRIES` 在 `judging_units_entries.rs:146`，`PROVENANCE_OF_THE_FLOWS` 在 `crash_point_replay_provenance.rs:139`（sha 8e54716a），`enumeration_of_the_item` 在 `:97`、`amplification_of_the_item` 在 `:111`，与定义里的名字、路径一致。
- 推翻现象：门禁或测试对一个不自带观察者的新崩溃用例报「OBSERVER_ENTRIES 缺行」；或 `crash-case-check.py provenance` 对不调 `enumerate_crash_states` 的用例报缺 `enumeration_of_the_item`。我读 `crash-case-check.py:55` 的判法是不报，没起脚本。

## 条 3：一百三十（verification.md、implementation-workflow.md、commit-stages.md）

被判句子（同一句意思三处写）：
- 「另有 product-gpu-real-card-tests 一格」（`.claude/rules/verification.md:35`，sha 0072bcc9）
- 「另有 product-gpu-real-card-tests 一格」（`.claude/rules/implementation-workflow.md:44`，sha 262d8682）
- 「另有 product-gpu-real-card-tests 一格」（`.claude/skills/crash-test/commit-stages.md:10`，sha 21973562）
- `implementation-workflow.md:68` 同义，写成「另有 product-gpu-real-card-tests 一格：产品 GPU 包带 `tests-on-a-real-gpu` 的真卡对拍」。

### 3.1 与门禁此刻做的对照　成立

- 门禁自己的格声明：「product-gpu-real-card-tests 快档起产品 GPU 包（singlefs-background-gpu）带 tests-on-a-real-gpu 特性才编的真显卡对拍测试」（`.claude/gate.d/checker-tier-crash-point-replay.sh:11`，sha d19991e5）。包与特性一致；特性在包里：「tests-on-a-real-gpu = []」（`crates/singlefs-background-gpu/Cargo.toml:22`）。
- 起法含 `--test-threads 1`：「#   起法：cargo test --release --no-fail-fast -p singlefs-background-gpu --features tests-on-a-real-gpu」（`.claude/gate.d/checker-tier-crash-point-replay.sh:155`）。「没卡报本次未跑」：格声明里「没卡打的 NOT_RUN_NO_GPU_ADAPTER 逐条记本次未跑」（同文件 `:11`）。「对拍判红整格红」「每条测试函数按单元键复用」同在 `:11`。
- 三份文字互相是同一个意思，只是详略不同：verification.md 最全（包名、特性、`--test-threads 1`、按单元键复用、红则整格红、没卡未跑），另两份缩成「真卡对拍、`--test-threads 1`、没卡报本次未跑」，没有矛盾。
- 提交时 `merge-and-verify.sh` 起三格：「CHECKER_QUICK_CELLS="crash-case-replay,feature-gated-tests,product-gpu-real-card-tests"」（`scripts/gate/merge-and-verify.sh:275`，sha bdf206f6），弄坏开关 `:277` 退回两格；与上面三份规则字句「快档三格」对得上。
- “两趟”与第三格：三份文字仍说「快档两趟」再「另有一格」，没有把第三格算进“趟”里；verification.md 的「两趟里每条测试函数各是一条判定」(`:35`，后半) 不含第三格，但同一句里已写「每条测试函数按单元键复用」。读起来不矛盾。

### 3.2 commit-stages 第 19 行的重跑出路漏一格　不成立

- 本批新加的半句：「feature-gated-tests 格里带 `gpu` 特性那一趟与 product-gpu-real-card-tests 格的真卡对拍要卡」，同一句后面照旧是「清场回读没过时等卡上那个不归清场白名单的进程跑完再 `--check feature-gated-tests` 只重跑这一格」（均在 `.claude/skills/crash-test/commit-stages.md:19`）。
- 门禁一侧：「要上卡的三处（查库那一批里有带 gpu 特性的命令、feature-gated-tests 带 gpu 特性那一趟、product-gpu-real-card-tests 的对拍那一趟）开跑前都经 require_service_card 核环境里有这张卡」（`.claude/gate.d/checker-tier-crash-point-replay.sh:189`），没卡就“那一格判红”。`--check` 可以点几格：「--check <格名>[,<格名>…]」（同文件 `:87`）。
- 句内不一致：同一句先说两格都要卡，再说卡的问题只重跑 `feature-gated-tests` 一格。清场回读没过若同时拦下两格，按字面只重跑一格，产品 GPU 包那一格仍是红或未跑，要等下一整趟才补。应改成 `--check feature-gated-tests,product-gpu-real-card-tests`（merge-and-verify 也是这样点格的，`scripts/gate/merge-and-verify.sh:99`）。
- 核不动的部分：清场回读没过时到底是哪几格被拦、还是整趟都不起，要看 `task-server` 对这道门禁的套法（`task-server/task_server.py` 的 recognize_gpu）；我没读，写“复核不了”。所以结论限于「句子自己前后不一致」，不断言门禁在清场失败时一定拦两格。
- 推翻现象：主 agent 读 `task-server` 后确认清场失败只会拦 feature-gated-tests 那一格。

### 3.3 别处还在教旧说法　有一处

- `.claude/main-agent.md:32`（sha 439b97be，不在本轮被判文件里）：「crash-case-replay、feature-gated-tests 两格只现判单元键对不上的测试函数」，描述补丁合入后的 checker 档快档步；而 `merge-and-verify.sh` 此刻起三格（`scripts/gate/merge-and-verify.sh:99` 头注释「起门禁 .claude/gate.d/checker-tier-crash-point-replay.sh 快档里按条复用测试函数的三格」）。一句话两个数。该改成三格。不是本批改动引起，是本批没扫到。

## 条 4：一百六十六 ⑨ 计数栏（mutation-triage.md 第 32、41 行，与 mutation-sampling.md 第 79 行同一套）

被判句子：「`mutate.sh` 收尾那行「计数：」报抓到、没红、无效、没跑完、抓名字有盲区、锚点失效六栏」（`.claude/agents/mutation-triage.md:32`，sha b60ebc1c），与第 41 行「连同另起一行报的「基线：」行，有「清场没过没跑：」「本次未跑：」行的一并贴原样」。

### 4.1 `mutate.sh` 一侧　成立

- 此刻收尾那行六栏、次序：`printf '抓到 %s 条、没红 %s 条、无效 %s 条、没跑完 %s 条、抓名字有盲区 %s 条、锚点失效 %s 条'`（`scripts/mutation/mutate.sh:2046`，sha 1e064d0d），与定义的次序、字面一致。符号对应：「抓到 ✅、没红 ❌、无效 ⏭、没跑完 💥、抓名字有盲区 ⚠️、锚点失效 ⚓，几栏加起来等于表的条数」（`scripts/mutation/mutate.sh:2033`）。后面还接「、现判 N 条、复用 M 条、拒算 R 条」（`:2030`），“照原样另列”覆盖它。
- 删掉的五个符号（⏱ 🧱 ⏳ 🚧 🎴）：`scripts/mutation/mutate.sh` 里各 0 处；`.claude/agents`、`.claude/rules`、`.claude/skills`、`CLAUDE.md`、`.claude/main-agent.md`、`.claude/agent-common.md` 里 `⏳ 🚧 🎴 显存不够： 被总上限挤掉 排不上没跑 scope 起不来没跑` 零命中。别处没有还在教旧栏的。
- 第 32 行“`⚠️`、`⚓` 不为 0 的整轮已判失败，`💥` 不判失败”本批没动；`mutate.sh` 文件头「退出码」写「1 有没红或抓名字的规则有盲区」（`:138`），锚点失效退 3 写在 `mutation-sampling.md:57`，没矛盾。

### 4.2 门禁一侧　在途改动让「清场没过没跑」一句对不上（本腿开工之后别的会话改了 `crates-mutation-rows.py`）

- 本腿开工时 `scripts/mutation/crates-mutation-rows.py` 的 sha 是 63f49976，读到的收尾输出有 计数 / 清场没过没跑 / 基线 / 本次未跑 四行；写报告期间这份文件被改（此刻 sha 4a31ff70，工作区有未暂存改动，`git status --short` 报 `MM`），`git diff -- scripts/mutation/crates-mutation-rows.py` 里 `清场没过没跑` 出现在 11 行删除里，此刻文件里 `grep -c '清场没过没跑'` 输出 0。HEAD 版本里是 11 处。这份文件不在第一节的被判与对照名单里，是我为核第 32 行多读的；它改动的原因我没查。
- 此刻（sha 4a31ff70）的收尾行：「计数：抓到 {len(caught)} 条、没红 {len(failures)} 条、无效 {len(invalid)} 条、锚点失效 {len(anchor_broken)} 行」后接一段“共 N 条”（`scripts/mutation/crates-mutation-rows.py:4744`、`:4745`）；基线一行「基线不绿 {len(baseline_bad)} 行」（`:4749`）；本次未跑一行（`:4762`，只在有未跑行时打，`本次未跑：1 条` 的自证退 0，`:6119` 附近）；sections 里是 anchor_broken、baseline_bad、invalid、failures 四类（`:4773` 起）。没有清场没过没跑那一行与那一类。
- 对定义的影响：`.claude/agents/mutation-triage.md:32` 与 `:41`、`.claude/rules/mutation-sampling.md:79` 仍写「清场没过没跑：N 条」。按 HEAD 版本的门禁，这句成立（HEAD 里 `:4646` 一带有那一行，我开工时读到过）；按工作区此刻的门禁，这句描述的行不再打。这是别的会话的在途改动，不是本批句子的错；落地时要同一批改这三处，否则就是一份定义教着门禁已不打的行。
- 其余部分对 HEAD 与此刻的门禁都成立：「锚点失效 N 行」一栏、无 `💥` `⚠️` 两栏、基线另起一行报「基线不绿 N 行」、`本次未跑` 另起一行、「共 N 条」；无效、没红、锚点失效、基线不绿都让整道判红，本次未跑不判红。
- 与规则：我用逐字符比较拿 `.claude/rules/mutation-sampling.md:79`（sha 0ebfe2eb）与 `mutation-triage.md:32` 从「`mutate.sh` 没有三数汇总行」起的两段比，差别只有：规则把“`⚠️`、`⚓` 不为 0 的整轮已判失败……”放在前面、分类三数之后接“`crates/mutations.tsv` 整表复跑是重型……”，定义把同一句放在后面接“与改动前的数比”。栏名、次序、六栏、另起三行的字全同，是同一套。

### 4.3 小边角（不构成不成立）

- 定义第 32 行“几栏加起来要等于表的条数”只对 `mutate.sh` 的符号栏成立。门禁那一行的“抓到+没红+无效+锚点失效”在有本次未跑、清场没过没跑、基线不绿行时小于“共 N 条”。这句夹在两段之间，读者可能套到门禁；第 41 行没让加总，影响小。
- 推翻现象：`mutate.sh` 或 `crates-mutation-rows.py` 收尾栏名/次序变动；本轮两份都没变。

## 条 5：agent-common.md 第 98 行去掉清变量一句

被判句子：「派 `experiment-designer` 写重跑登记而准入（`scripts/gate/admission.py experiment`）判输入没变（退 77）」（`.claude/agent-common.md:98`，sha b4ee6573）。

### 5.1 删句与此刻的做法　成立

- `admission.py` 此刻：环境里设了这个变量，任一子命令都用法错退 2、什么都不做：「RETIRED_RERUN_REASON_VARIABLE in os.environ and not」（`scripts/gate/admission.py:6881`，sha 15b28dc4），文件头「没有强制重跑：实验键与上一份产物头相同就退 77」（`:262`）。
- 派发闸此刻不清它：「派发闸不清它，照退 2 拒，出路另写」（`.claude/hooks/runner-dispatch-guard.sh:1354`，sha acde6287），清变量只在弄坏开关下发生（`:358`、`:1368`）。所以共用约束不再叫人“起它之前清掉”是对的：留着那句反而教人去清一个派发闸自己不会清的变量。
- 旁证：本会话环境里该变量未设：`env | grep -c SINGLEFS_EXPERIMENT_RERUN_REASON` 输出 0。
- 工具员 `result.md` 的处置说明（派发闸“仍然在起 admission.py 之前清掉这个变量”）已与钩子此刻不符：钩子在那之后又改过，此刻的文字是上面两行。读 `result.md` 这一段的人别照它。

### 5.2 同一句里「分两种出路」与派发闸的“三条出路”对不上　不成立（小）

- 定义：「退 2 的拒绝说明分两种出路」（`.claude/agent-common.md:98`），两种是“接准入”与“让 admission.py 能跑”。
- 钩子文件头：「退 2 一律照拒，拒绝说明按输出分三条出路」（`.claude/hooks/runner-dispatch-guard.sh:68`），第一条就是“环境里设着已删的强制重跑变量……出路是从起会话的环境里去掉它”（同文件 `:69`）。
- 本批删掉“清变量”之后，定义里不再有任何一句提这个变量（`grep -rn RERUN_REASON .claude/agents .claude/rules .claude/skills .claude/agent-common.md .claude/main-agent.md CLAUDE.md .claude/singlefs-ai-sop/rules` 零命中），而派发闸三条出路里仍有它。主 agent 在环境里设着该变量时，会收到钩子自己的“怎么办”文字，所以不会卡死；只是定义说两种、钩子说三种。建议把“两种”改成与钩子相同的三条，或删“分两种出路”这半句只留“出路见拒绝说明”。
- 推翻现象：钩子自证 `runner-dispatch-guard.sh --selftest` 在设了该变量时拒绝说明里没有第三条出路。钩子此刻被改过，自证我没起，写进第四节。

## 条 6：四份 read-once 规则由定义按路径点名

### 6.1 路径存在、写法、门禁判据　成立

- 四个文件都在：`.claude/singlefs-ai-sop/rules/engineering-philosophy.md`、`machine-first.md`、`pushback-discipline.md`（sha a182572c）、`design-doc-discipline.md`；`.claude/rules/` 下没有同名文件（`ls .claude/rules/` 十二个文件里没有这四个），门禁取 `.claude/rules/<名>`、没有再取上游副本，落点是上面四个。
- 点名位置（逐份 grep）：engineering-philosophy.md、machine-first.md 在 `.claude/agents/implementation-writer.md:16`（sha 70ab774a）与 `.claude/agents/tooling-writer.md:16`（sha dcd6f8ef）的「开工先读：」行末；pushback-discipline.md 在 `.claude/agents/three-way-attack.md:17`（sha a4260bda）；design-doc-discipline.md 在 `.claude/agents/clerk.md:16`（sha 44c8ae8b）。
- 门禁判据：「每一条的路径在 .claude/agents/*.md 或 .claude/agent-common.md 里至少出现一次（按整段路径认，前后不接路径里的字）」（`.claude/gate.d/code-tooling.sh:1713`，sha 45de49a4）。我用 `grep -lF` 对 CLAUDE.md 第 38、61 行 read-once 登记的 11 个文件各查一遍：每个都至少有一份定义按完整路径点名，没有落空（命令与输出见第四节第 2 条的复现）。主 agent 报 4436 那一趟 rules-named-by-definitions 绿的结论与我这一读法不冲突。
- 读法：这些都是规则类文件，按共用约束整份读；`.claude/agents/clerk.md:16` 里写「改 `records/` 下的文件时另读」是条件句，与同一行前半「规则只读共用约束「规则怎么读」一节写的那三处」不矛盾（后面接的是“派发提示另点名的再读；改 records/ 时另读”）。
- 写范围对：clerk 能写 `records/**`：「clerk	records/**」（`.claude/hooks/agent-write-scope.tsv:45`，sha 8417ae19）；全表里写 `records/` 的只有 clerk（tooling-writer 两行里的 `records/` 只出现在说明列的文件名里），所以只点名 clerk 够用。

### 6.2 tooling-writer 同时挂 engineering-philosophy 与 machine-first　成立
两篇各自读完不会互相打架；`.claude/agents/tooling-writer.md:16` 把它们插在 `directory-layout.md` 与“第二种活另读 rules-discipline”之间，不破坏后面的条件句。

### 6.3 pushback-discipline 点给攻方腿　核不动

- 该规则是为“用户提议与实测对不上”写的，里面要求把警告写进 `.claude/warnings/<日期>.md`（`.claude/singlefs-ai-sop/rules/pushback-discipline.md:44` 节标题「警告写在哪：`.claude/warnings/<日期>.md`」），并规定“反对过一次、对方明确重申，那就是对方的决定：照做，不再劝”。
- 三方攻方腿的写范围只有报告、模型目录、草稿目录（`.claude/agents/three-way-attack.md:51`），写不了 `.claude/warnings/`。整份读时有两处用不上：写警告文件、“对方重申就照做”。攻方腿的立场（找漏洞、反推）与“对不上就当场说”相合，与“照做”不相合。
- 门禁明说它管不到“一条规则该被哪几份定义点名”（`.claude/gate.d/code-tooling.sh:1715`），所以不会红。该不该给攻方腿读，是判断，不是对错；我核不动，记作主 agent 判。若要点名，可在句末加“只用「对不上就当场说」「分清我核过和我印象里」两节”（规则类文件按共用约束整份读，加限定要主 agent 认可这一例外）。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| F-1a 设计员/执行员 gpu 特性名、判段认法、点名的格 | 成立 | `gpu` 是 checker 档包唯一特性；`crash_amplification::`、`crash_state_enumeration::` 与 `JUDGE_ENTRY_MARKERS` 同一集合；清场由服务做与 `.claude/rules/verification.md:63` 一致 |
| F-1b `grep -c NOT_RUN` | 成立 | 两族行都打到标准输出、都被数到；门禁出路句写 `NOT_RUN_`（带下划线）与定义不同，少数一族但不漏判 |
| F-1c 入库装置那一支 | 不成立 | 判段装置全是入库装置，执行员只跑 `compile-then-swap.py` 编的二进制，它不带 gpu；此刻 `ENABLE_GPU=1`，这一支照定义走必然报未跑；定义没说怎样编带 gpu 的 |
| F-2a 删不存在的小节 | 成立 | 全仓定义、规则、skill 零命中；`result.md` 说 `crash-case-check.py:25` 也指它，我对不上 |
| F-2b 「五处」 | 不成立 | 观察者表与 `enumeration_of_the_item` 两处在门禁里是有条件的，47 个用例目标里 17 个没有观察者行；改成「最多五处」并写条件 |
| F-2c 括号里「第一行声明模块」 | 成立（小缺） | 被点名的三节里没有「第一行」，出处在 `crash-case-check.py` 文件头 |
| F-3a 三份规则/skill 的新一格 | 成立 | 与门禁格声明、起法、`merge-and-verify.sh` 的三格一致 |
| F-3b `commit-stages.md:19` 重跑出路 | 不成立 | 句内先说两格要卡，再只让重跑 `feature-gated-tests` 一格；门禁 `--check` 可点几格 |
| F-3c 别处还在教两格 | 有一处 | `.claude/main-agent.md:32` 写「crash-case-replay、feature-gated-tests 两格」，`merge-and-verify.sh` 起三格 |
| F-4 计数栏 | 成立（`mutate.sh`、规则）；门禁一侧有在途改动 | 六栏次序、符号、另起行都对；`crates-mutation-rows.py` 被别的会话改过，「清场没过没跑」行已删，定义三处还写着 |
| F-5a 删清变量一句 | 成立 | `admission.py` 见变量退 2；派发闸此刻不清它，与删句一致；`result.md` 的处置说明已过期 |
| F-5b 「分两种出路」 | 不成立（小） | 派发闸文件头写三条出路，第一条是该变量；定义里再没提它 |
| F-6a 四份规则点名 | 成立 | 路径存在、写法对、门禁判据对得上；CLAUDE.md read-once 的 11 个文件各至少被一份定义按路径点名 |
| F-6b pushback-discipline 给攻方腿 | 核不动 | 规则要求写 `.claude/warnings/`，攻方腿写不了；该不该点名是判断 |

## 要主 agent 跑的命令

（不重型；提交时都是几秒到几十秒。命令原样，不改。）

1. 判段特性格真仓复核（条 1）：`task-server/task submit -- bash .claude/gate.d/doc-experiments.sh --check preregistration-judge-features`
2. 规则被定义点名格真仓复核（条 6）：`task-server/task submit -- bash .claude/gate.d/code-tooling.sh --check rules-named-by-definitions`
3. 7a 的命令本身跑得动，且看现状有几项报缺（条 2）：`task-server/task submit -- python3 scripts/crash/crash-case-check.py --only placement,modules,provenance`
4. 派发闸此刻对设了已删变量的拒绝说明（条 5.2；钩子比 `result.md` 新）：`task-server/task submit -- bash .claude/hooks/runner-dispatch-guard.sh --selftest`
5. 门禁收尾行的现状（条 4.2；先问清别的会话的改动是否要落地）：`task-server/task submit -- python3 scripts/mutation/crates-mutation-rows.py --selftest`（按它文件头弄坏开关不用开；看收尾文字是否还带「清场没过没跑」）。这一条的用处是确认在途改动自证绿，不是为了核本批句子。

只读复现（直接在仓根跑，不经服务；输出我已在文中引用）：

- 条 1.3：`grep -c -e features -e gpu scripts/hosts/compile-then-swap.py`（我的输出 0）；`grep -rln 'crash_amplification::\|crash_state_enumeration::' scripts/experiment-apparatus`（零命中）。
- 条 2.2：
  `python3 - <<'PY'` 内容：读 `.claude/gate.d/stage-inputs.tsv`，`re.findall(r'test=singlefs-checker-tier:([a-z0-9_]+):')` 去重得 47 个目标；再数 `crates/singlefs-checker-tier/src/judging_units_entries.rs` 里含 `test:<目标>"` 的，得 30。这一段用的是 `python3` 内联脚本，按共用约束属“起脚本”，我自己跑过一次得上面两个数，主 agent 复跑才算数。
- 条 6.1：`for n in engineering-philosophy machine-first design-doc-discipline pushback-discipline code-discipline test-discipline preflight-discipline mutation-sampling test-script-discipline path-moves directory-layout; do p=.claude/rules/$n.md; [ -f $p ] || p=.claude/singlefs-ai-sop/rules/$n.md; echo "$n: $(grep -lF "$p" .claude/agents/*.md .claude/agent-common.md | tr '\n' ' ')"; done`，我的输出里 11 行都不空。
- 条 4.2：`git diff -- scripts/mutation/crates-mutation-rows.py | grep -c '清场没过没跑'`（我的输出 11）与 `grep -c '清场没过没跑' scripts/mutation/crates-mutation-rows.py`（输出 0）。

## 没做什么

- 没有起 `compile-then-swap.py`、`crash-case-check.py`、钩子自证、门禁单格；条 1.3、2.2、5.2 的推论是读码推出的，推翻条件写在各条里。
- 没读 `task-server/task_server.py` 的 GPU 清场套法，所以 3.2 只断言“句内不一致”，不断言清场失败时门禁一定拦两格。
- 没查入库装置判段实验的历史产物是怎么跑出来的（E184 等）；1.3 的“走不通”是按定义文字与 `compile-then-swap.py` 此刻的参数推的。
- 没改任何被判文件，不替主 agent 采纳或出判决；没往统一任务服务交任务；判决要引的数没复跑。
- 没核 `result.md` 里的“逐条”之外的改动（本轮只判第一节六条）；`this-batch.diff` 的 134 行我读的是它的内容，没另算 sha（背景材料说 2a04697b）。
- 攻方腿的产出没读。
