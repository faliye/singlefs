# defs-gatebatch-m2-r1 云端攻方（Opus）报告：J1–J4

- 腿：云端攻方（Opus），攻击面 J1、J2、J3、J4（正文第一节表）。写于 2026-09-26，UTC 13:0x–13:3x（JST 22:0x–22:3x）。
- 被判对象：开工时拿 `research/prompts/defs-gatebatch-m2-r1-snapshot/sha256sums.txt` 核过，11 个文件 `sha256sum -c` 全 OK（54 号、admission.py、stage-inputs.tsv、lib_heavy_tests.py、heavy-test-guard.sh 等与快照相同）。
- 执行了什么：三份自证各在临时拷贝上跑一次（原样：admission.py 140 格、lib_heavy_tests 55 种、heavy-test-guard 580 种，全过，日志在模型目录 `outputs/selftest-*-original.log`）；四个探针。没有执行 54 号本身、没有执行任何重型测试、没有枚举任何崩溃状态（本腿原型跑的崩溃状态数 = 0）；闸只喂 JSON、只看退出码。唯一编过的 Rust 是草稿目录里一个 5 行的 `rustc --test` 小文件（经内存包装），只为核 libtest 认不认缩写的参数。

## 复跑

```
bash research/prompts/defs-gatebatch-m2-r1-opus-model/rerun.sh
```

约 3 分钟（J2 那一份占 2 分 40 秒）。各探针退 1 表示有攻击格在今天的代码上不成立（即打中），退 2 表示对照格破了（探针自己有问题），退 0 表示全部成立。rerun.sh 另把改法 F1、F6 的补丁打在临时拷贝上，再跑 J1、J4。这一次的原样输出：

```
probe_j1_stage 今天的代码 exit=1
SUMMARY J1 54 号: cells=15 attack_cells_broken=6 control_cells_broken=0
probe_j2_manifest 今天的代码 exit=1
SUMMARY J2 排除法: cells=11 attack_cells_broken=6 control_cells_broken=0
probe_j3_environment 今天的代码 exit=1
SUMMARY J3 构建环境: cells=8 attack_cells_broken=3 control_cells_broken=0
probe_j4_hook 今天的代码 exit=1
SUMMARY J4 重型测试闸: cells=28 attack_cells_broken=17 control_cells_broken=0
probe_j1_stage 套 F1、F6 exit=0
SUMMARY J1 54 号: cells=15 attack_cells_broken=0 control_cells_broken=0
probe_j4_hook 套 F6 exit=1
SUMMARY J4 重型测试闸: cells=28 attack_cells_broken=16 control_cells_broken=0
```

逐格的原样行在 `outputs/j1.log`、`j2.log`、`j3.log`、`j4.log`、`j1-fix.log`、`j4-fix.log`（一格一行，制表符分隔：类别、成立与否、格名、明细），下文写成「j1.log:4」这样的行号。

模型目录 `research/prompts/defs-gatebatch-m2-r1-opus-model/` 各文件的 sha256（`SHA256SUMS` 原样）：

```
9d7a9b5a54efba4dbb10603f69b9c8443644b99ef315ae4fba96da0b85ca0c6d  ./fix-f1-54-layer0-replay.patch
51245f19fc4a79e86e414c04336b9be0dc215f6fd3108cf344f24022784fadf9  ./fix-f6-admission.patch
b4b5957b1015314be36193319bfee0e76c9d2d742922b78ef8eea2ec427b1a7b  ./outputs/j1-fix.log
746be04fa109acf4f4f50a5129207f06d2cca04770ef3099c0be05f73fda2f4c  ./outputs/j1.log
92f53825fa4a67ad00b59eb04749ecbbafc3aff187fb25e814b1511076786755  ./outputs/j2.log
5b8e6a7e58dd3400e803f6e8c99554641875778fc11d4a75ff9e3a6d89f979db  ./outputs/j3.log
3a50adfbb8beced07b4dbdee827ebd10c974453530112ae7a53d1703cbca648c  ./outputs/j4-fix.log
935c8c34be39eb5bb10e369cc9815d424b172089f903af9181a5fc28f8556713  ./outputs/j4.log
390a8eddbc1c76eefccc2cfe065a64fc4a31a29cd043e0f7c53df38e439761ef  ./outputs/selftest-admission-f1-f6.log
54dce6bd7826935a0f07eafccf53f16ca24aec33c9dfed69692455cbb4950f27  ./outputs/selftest-admission-original.log
20199c25ed5c3aa986c97cf004ee0369cd5604a5c628868f09dac4164ddbfc61  ./outputs/selftest-guard-original.log
9ab7de6661734b1023950e3da0ecfc9ddbdc6105f4ccd2dbec3b2a0dc2f27e3a  ./probe_common.py
0b0744b4fa68a1f983e642b660dbd175c39d870c8c041cc94e76eb167ad8f5ad  ./probe_j1_stage.py
8886410b1624692e7728cf0a33a96a55b52be4c5a69683df0267f248c27698a1  ./probe_j2_manifest.py
0669e97ce1e1621e235fc1dc08350f03a05835c0619e14479e400440c894ae79  ./probe_j3_environment.py
ff06b09785e982c77325bd44c14760afd71d652902b92a7aca0c6038b63fe2c7  ./probe_j4_hook.py
88be81e77dfc601a8b1d132ac515978ac573bb16a6f95d096f8c633cf1dee62b  ./rerun.sh
```

## 各格判定一览

| 格 | 判定 | 打中了什么（一句） | 这一批新引入，还是之前就有 |
|---|---|---|---|
| J1-a 范围那一问把改动摘掉了（A1–A3） | **打中（量过）** | 只改登记表（新登记一条用例、改某条用例的第三列）或只改 54 号本身时，快档退 77（本次未跑），不去核任何一格标记；带 `SINGLEFS_GATE_FULL=1` 时同样的改动判红 | A1、A2 是这一批带进来的（登记行现在决定跑哪几条、怎么判）；A3 以前就有 |
| J1-b 判法不在指纹里（A4） | **打中（量过）** | 判日志、判标记的代码从 54 号挪进了 admission.py，而 admission.py 不进每条用例的指纹：修好判法之后，旧判法写的那一格照样被复用，快档照样判绿 | 这一批带进来的（以前判法写在 54 号里，54 号进指纹） |
| J1-c 并发两趟 `--full`、改到一半又改回来 | 低危，推的，没量 | 同一条用例、同一个指纹的两趟并发：后一趟判红，并删掉前一趟刚写的那一格（假红方向）；主工作区里跑 `--full` 时源码被改了又改回来，指纹前后相同，标记照写 | 并发那一种是这一批带进来的；改了又改回来那一种以前就有 |
| J1 续跑目录串用、`--start-over`、被打断留下标记 | 没打中 | 见「没打中的形状」 | — |
| J2-a 减多了（U1、U2） | **打中（量过，形状构造出来的，今天的仓里没有这种写法）** | 用例在编译期用 `include!(concat!(…))` 拼出别的测试目标的文件名；另一个包的 build.rs 按目录读本包的 `tests/`。两种情况下改那个目标，指纹都不变。文件头「认不出的」那一段没写这两种 | 这一批带进来的 |
| J2-b 减少了（O1–O4） | **打中（量过，在真仓拷贝上）** | 只改 `crates/mutations.tsv`、某个只被别的测试用到的 `tests/<目录>/mod.rs`、harness 的 `src/bin/*.rs`，四条用例的指纹全变，第二条流（约 2.3 天，数字取自第三轮判决，推的）要重跑 | 这一批带进来的（排除法本身的粒度问题） |
| J3-a 按路径登记的可执行文件不按内容进指纹（E1–E3） | **打中（量过），今天能不能走到：低** | runner 与 `RUSTC_WRAPPER` 指的脚本换了内容，指纹不变；`.cargo/config.toml` 里 `build.rustc` 指的编译器换了版本，指纹也不变（同样的编译器用 `RUSTC` 给时会变） | 这一批带进来的（第三轮 U4 只落了「值」这一半） |
| J3 `CARGO_BUILD_JOBS` 不进指纹 | 没打中 | 并行度不改变编出来的东西（这一句是推的）；它不进指纹是对的 | — |
| J3 主工作区与 worktree 的指纹是不是一定相同 | 不一定，但只会往假红的方向偏（量过 3 格） | 两边干净、环境相同时相同；主工作区多出一个 src 下的未跟踪文件，或只有一边的上层目录里有 `.cargo/config.toml`，两边就不同。两种都只会造成假红，造不成假复用 | — |
| J4-a 闸能绕过去（B1–B13） | **打中（量过，只喂 JSON）** | 子 agent 用 `cargo nextest`、`cargo mutants`（本机装了）、`--config` 定别名、runner、把测试二进制拷到别处、`find -exec`、`/usr/bin/time`、`flock`、`rustup run`、`chrt`、`systemd-run --scope`、`prlimit`，都能不被拒就跑到登记的全量用例 | 包装与子命令这一类以前就有，层 0 同样受影响（j4.log:23–24）；专门冲着崩溃枚举用例的是 runner、别名、`--include-ignored` 这几种 |
| J4-b 用例函数的 `#[ignore]` 被去掉（I1、I2） | **打中（量过）** | 闸认这几条用例的前提是它们标了 `#[ignore]`，可没有一处检查这一点：去掉之后，不带 `--ignored` 的命令就跑 2^18 个状态，闸放行，`crash-cases` 自查也退 0 | 这一批带进来的 |
| J4-c 误拒（R1、R2） | **打中（量过），危害小** | `--ignored --list`（只列出用例、一条都不跑）、`--include-ignored --exact <一条快用例>`，这两种都被拒了 | 这一批带进来的 |

## J1：54 号逐条跑、逐条标记、续跑

### J1-a 只改登记表或 54 号本身，快档退 77，一格标记都不核（A1–A3，量过）

**历史**（`probe_j1_stage.py` 拿临时小仓复现：54 号的拷贝放在 `.claude/stage-under-test/` 下，用假 cargo 跑）：
1. HEAD 里已经有一个标了 `#[ignore]` 的测试目标 `case_d.rs`，还没登记。三条已登记的用例 `--full` 判绿，各写一格标记（j1.log:1）。
2. 这一次暂存的改动只动 `.claude/gate.d/stage-inputs.tsv`，加上一行 `crash-case:case-d`（A1）。另两种造法：只把 stream-b 那一行第三列的 `exhaustive=LAYER0B` 删掉（A2），或只给 54 号加一行注释（A3）。
3. 整轮门禁跑快档，结果退 77（j1.log:4、6、8）。同样的改动加 `SINGLEFS_GATE_FULL=1` 再跑，快档退 1，点名的正是那一条用例（j1.log:5、7、9）。对照：改 `crates/` 下的 src/lib.rs，快档不退 77、判红（j1.log:3）。

**每一步的出处**：
- 快档的第二问只按 54 号那一行登记的路径判范围：`.claude/gate.d/54-layer0-replay.sh:114` 原文 `scope_reason="$(bash "$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/change-touches-crates.sh" "$ROOT" "${layer0_registered_input_paths[@]}")"`。54 号那一行的路径是 `crates/ Cargo.toml Cargo.lock`（`.claude/gate.d/stage-inputs.tsv:24`）。`change-touches-crates.sh:105` `  for prefix in "${prefixes[@]}"; do` 只拿这几个前缀去比。
- 答「没碰」就退 77：`.claude/gate.d/54-layer0-replay.sh:117` `    echo "  ! 本阶段跳过（这次改动没碰它判的东西）：$scope_reason"`，下一行 `exit 77`。
- 77 不拦提交：`.claude/singlefs-ai-sop/scripts/gate.sh:250` `# 退出码 77 = 这一轮无对象可判，记「本次未跑」，既不算通过也不算失败`。
- 可登记表对这种改动的承诺是：`.claude/gate.d/stage-inputs.tsv:19` `# 不会让这道阶段重跑——而清单本身变了必定重跑，改清单的代价因此是「下一趟全跑一次」，不是零；加一个实验行也一样。`第一问（stage-must-run）确实把登记表与 54 号本身算进去了（`research/scripts/admission.py:921` 那一行 `input_paths += [REGISTRATION_TABLE, f".claude/gate.d/{stage}"]`），但第二问不算，于是第一问答完「要跑」，第二问又把它摘掉了。
- 自证为什么没抓到：54 号那几格全带着 `SINGLEFS_GATE_FULL=1` 跑（`research/scripts/admission.py:2633`），范围这一问从来没被走到。

**四句**：
- 分不分辨臂：这一轮判的是一份实现，没有几条候选要选。这一格分辨的是「今天的 54 号」与「改法 F1」：F1 在 A1–A3 上成立（j1-fix.log:4、6、8），今天的代码不成立。
- 被判的系统当时看不看得到判别它的东西：看得到。做这个决定时，54 号手里已经有登记表（`$layer0_input_table`）、自己的路径（`$layer0_stage_script_path`）和准入模块的路径（`$layer0_admission_module`，`.claude/gate.d/54-layer0-replay.sh:69`）。
- 满足判据字面的哪一个分句：正文 J1「某条用例的输入变了而它的旧标记照样作数」旁边那一问，加上共用问句里的「放过」。这里的输入（登记行、判它的 54 号）确实进了指纹，指纹也确实变了；问题是快档根本没走到核指纹那一步。归「放过」，不归「假复用」。
- 跑前条款给的改法在这几格上还中不中：规格里给的改法（清单用排除法写、指纹补第三轮 U4 那几样；`records/2026-09-24-里程碑二收尾调度.md:190`）都不碰范围这一问。它们在这三格上照样中。

**这一格在真流程里怎么走到**：新写一条崩溃用例时，常见的是先提交测试（带 `#[ignore]`），再在另一次提交里登记它。这一批自己的做法就是这样：floor-raise 那条用例是实五加的，门禁批才登记。登记的那一次提交如果不碰 `crates/`，新用例就一次都没被核过，整轮门禁照样过。`.claude/main-agent.md:61` 派崩溃验证员的条件是「改了门禁 54 号在 stage-inputs.tsv 登记的输入」，也只看 `crates/`、`Cargo.*`。这句文字是否也要改归 J5，这里只指出它和代码的缺口是同一个。

### J1-b 判法搬进了 admission.py，而 admission.py 不在指纹里（A4，量过）

**历史**：
1. 提交一份「旧判法」的 admission.py（造法：把 `research/scripts/admission.py:1291` `        problem, note = judge_worker_threads(prefix, line, log_lines, machine_cores, threads_explicitly_one)` 换成不判工作线程）。stream-b 的日志是 64 片却只起了 1 个线程，旧判法判绿，写下那一格（j1.log:11）。
2. 提交修好的判法，也就是今天这一份。stream-b 的指纹前后逐字相同（j1.log:12：`旧 639097891a56ec19…，新 639097891a56ec19…`）。
3. `--full` 复用那一格，一条都不跑（j1.log:13）。快档带 `SINGLEFS_GATE_FULL=1`，退 0（j1.log:14）。修好的判法拿同一份日志判，是红的：「只起了 1 个工作线程」（j1.log:15）。

**出处**：每条用例的指纹只把 54 号当「判它的」：`.claude/gate.d/54-layer0-replay.sh:168` `      --extra-file "<判它的 54 号：$layer0_stage_file_name>" "$layer0_stage_script_path" --toolchain --build-environment)"; then`。改动之前，判法写在 54 号正文里，例如附录二 `research/prompts/_defs-gatebatch-m2-r1-diff.md:445` `-if [[ "$line" != *"exhaustive=true"* ]]; then`、`:476` `-worker_threads_are_acceptable "两次发布那条流" "$worker_threads_b" || exit 1`，54 号进指纹，判法也就跟着进了。现在判法在 `research/scripts/admission.py:1265` `def judge_crash_case_log(…)`，快档复核标记用的是 `:1321` `def crash_case_marker_problems(case, fingerprint, marker):`，后者只复核输入指纹、用例名、test result、计数行、exhaustive 这几样，不复核工作线程，也不复核「读回的片 + 这一趟跑的片 = 总片数」。

**四句**：
- 分不分辨臂：分辨「今天的 54 号」与「F1」（F1 把 admission.py 当 `--extra-file` 放进指纹）：j1-fix.log:12–14 三格都成立。
- 看不看得到：看得到，54 号知道准入模块的路径（`:69`）。
- 满足的分句：J1「某条用例的输入变了而它的旧标记照样作数（假复用）」，字面就是这一句。这里的「输入」是判它的代码。
- 跑前改法还中不中：U4 与排除法都不碰这一格，还中。

### J1-c 低危，推的，没量

- **两趟 `--full` 并发跑同一条用例、同一个指纹**：两趟共用 `<common-dir>/singlefs-layer0-progress/<指纹>/`。后开的那一趟在 `Layer0ProgressFile::open` 里把进度文件换名重写（`crates/singlefs-harness/src/layer0_progress.rs:700` `        let rewriting_path = path.with_extension("rewriting");`），先跑完的那一趟在 `finish` 里把这个路径删掉（`:767` `                std::fs::remove_file(&self.path).unwrap_or_else(|error| {`）。后跑完的那一趟再删时找不到文件，panic，判红。接着 54 号在 `:344` 执行 `delete_crash_case_marker "$case_key" "$fingerprint_at_start"`，把先跑完的那一趟刚写下的绿标记删掉。结局是假红，不会假绿。54 号自己不加锁，只靠崩溃验证员定义里「另有 `--full` 在跑时不起」那一句挡着。
- **主工作区里跑 `--full`，源码被改了又改回来**（例如别的会话在做「先证红」）：开跑与跑完两次算出的指纹相同，而 cargo 编的是中间那一版。54 号文件头要求在 worktree 里跑，但脚本本身不强制。以前那一版 54 号也有这个问题。

## J2：按排除法算每条用例的输入

### J2-a 减多了：被减掉的文件其实编进了用例（U1、U2，量过，形状是构造的）

用的是 admission.py 自证里那个小仓（包 pkg，用例 own_case，别的测试目标 other_target）：
- **U1**：own_case.rs 里写 `include!(concat!("other_", "target.rs"));`，在编译期把 other_target.rs 编进自己。改 other_target.rs，指纹不变（j2.log:1：`改前 690d64c93af0962c…，改后 690d64c93af0962c…`）。对照：同一处写成字面路径 `include_str!("other_target.rs")`，指纹就变（j2.log:2）。
- **U2**：另一个包 gen 的 build.rs 按目录读 `../pkg/tests`，pkg 自己没有 build.rs。改 other_target.rs，指纹不变（j2.log:3）。

**出处**：只看本包有没有 build.rs：`research/scripts/admission.py:1154` `    """包里有 build.rs、写了 [[test]]、package.autotests 或 package.build：测试目标怎么编、读什么由它们另定，这一个包的测试文件一份都不减。"""`。文件头承认认不出的只有两种：`:90` `崩溃枚举用例减去的「别的测试目标独占的文件」认不出的：用拼出来的名字在运行期读别的测试文件（名字不以整词出现在任何代码里）、`；`:91` `build.rs 之外的构建期代码按目录读 tests/；这两种会让那份文件被减掉而它其实被读了。`。U1 是编译期拼名字，U2 是别的包的 build.rs，两种都不在这两句里。

**今天的仓里走得到吗**：走不到。四条登记用例的测试文件里没有 `concat!`，`crates/` 下也没有 build.rs。这是写法上的缺口，不是今天真仓上的错。

**四句**：分不分辨臂不适用。看不看得到：U2 看得到（别的包的 build.rs 就在清单里）；U1 只看得到 `include!(concat!(` 这个形状，拼出来的名字要展开宏才知道，做不到逐字判，只能保守地一律不减。满足的分句：J2「造一处改动：它改变了某条登记用例的行为，而被算成「与它无关」减掉了（…拼接出来的名字…）」。跑前改法：记录第三节的规格只说「清单用排除法写」，没有碰这两种。

### J2-b 减少了：改用例读不到的文件，四条照样重跑（O1–O4，量过，真仓拷贝）

探针把真仓 `git ls-files -co --exclude-standard -- crates/ Cargo.toml Cargo.lock` 列出的文件、登记表、`.gitignore` 拷进临时仓，用假工具链算四条登记用例的指纹。那一刻的拷贝里，每条都留 69 个文件、减去 68 个（j2.log:4–7；实七正在改 `crates/`，所以文件数会随时间变）。每次只改一个文件：

| 格 | 只改了什么 | 为什么四条（或其中几条）读不到它 | 指纹变了的（原样） |
|---|---|---|---|
| O1 | `crates/mutations.tsv` 加一行 | 没有哪个 .rs 在编译期或运行期读它（`grep -rn 'mutations.tsv' crates --include=*.rs` 只有文档注释里提到） | j2.log:8：四条全变 |
| O2 | `tests/common_tree_split/mod.rs` 加一个没人调的函数 | 用到它的只有 tree_split 那几份测试与 position_addressed_trees_layer0，四条用例的测试目标都不 `mod` 它 | j2.log:9：四条全变 |
| O3 | harness 的 `src/bin/e142_first_transaction_write_dump.rs` 加一个函数 | 集成测试不链接 bin，`crates/` 里没有任何 `CARGO_BIN_EXE_` | j2.log:10：四条全变 |
| O4 | `tests/common_admission/mod.rs` 加一个函数 | 只有 floor-raise 那条 `mod common_admission;` | j2.log:11：另外三条全变 |

**代价**：每一种改动都会让第二条流重跑。按第三轮判决，冻结形状下这一趟 32 线程约 2.3 天，推的，线性外推；新形状没量过。O1 在现行流程里很常见：`.claude/rules/implementation-workflow.md:12`（第 3 步那一行）要求「每一处改法在 `crates/mutations.tsv` 留一条」，所以补一条变异、或只给已有测试补判别力，就要搭上一趟全量。O3 的 E142 装置归实验，改得也勤。

**出处**：排除法只减 `tests/` 下别的测试目标独占的文件（`research/scripts/admission.py:955` 起那一节的说明），`tests/<目录>/mod.rs` 这类共用模块、非 .rs 文件、`src/bin/` 一律留下。这是规格有意为之的「宁宽勿窄」（`.claude/gate.d/stage-inputs.tsv:22`）。所以这一格打中的是粒度，不是正确性。

**四句**：看不看得到：O1、O3 看得到（文件类型与位置就足以判）；O2、O4 要顺着模块图走，这正是规格有意不做的（E142 那一行的理由：「漏一个模块就是该跑的不跑」）。满足的分句：J2「另看会不会减得太少，让无关改动把 2.8 天的第二条流拖去重跑」。跑前改法：排除法本身就是改法，它不碰这几格。

## J3：构建环境进指纹

### J3-a 按路径登记的可执行文件，只进路径，不进内容（E1–E3，量过；今天能不能走到：低）

| 格 | 造法 | 原样 |
|---|---|---|
| E1 | `CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER` 指的脚本从 `exec "$@"` 改成 `exec "$@" --skip every_crash_state` | j3.log:1：`e9d6860335f8a238… → e9d6860335f8a238…` |
| E2 | `RUSTC_WRAPPER` 指的脚本加一个 `-C overflow-checks=off` | j3.log:2：前后相同 |
| E3 | 仓根 `.cargo/config.toml` 写 `[build] rustc = "<路径>"`，那个编译器的 `-V` 从 1.80.0 换成 1.99.0，配置文本不变 | j3.log:3：前后相同 |
| E4 对照 | 同一个编译器改用 `RUSTC` 环境变量给，`-V` 变了 | j3.log:4：指纹变 |

**出处**：环境变量这一类只进它的值，`research/scripts/admission.py:444` `            lines.append((f"<构建环境：环境变量 {variable}>", environment[variable].encode("utf-8", "surrogateescape")))`；只有 `RUSTC` 多取一次 `-V`，`:453` `        lines.append(("<构建环境：RUSTC 的值与它的 -V>", rustc.encode("utf-8", "surrogateescape") + b"\n" + completed.stdout))`。配置文件按文本进，文本里指向的 `build.rustc`、`build.rustc-wrapper`、`target.<三元组>.runner` 与 linker 这些可执行文件，本身不进。第三轮判决 U4 那一格（`research/prompts/m2-layer0-scale-r3-main-verification.md:31`）对 `RUSTC` 明写了「取 `$RUSTC -V`」，对 RUNNER、WRAPPER 与配置文件里的这几个键没写。现在的实现照字面做了，缺口也就照字面留着。

**今天走不走得到**：本机仓根往上各层、`~/.cargo` 下都没有 cargo 配置文件，也没设这几个环境变量（开工时核过：`for d in / /home … ; [ -e "$d/$f" ]` 一个都没有，`env | grep -E '^(CARGO|RUST)'` 为空）；仓里的包装也不设它们。所以只有人或 agent 手设 runner、wrapper 时才会碰上。E1 危害最大：runner 能改测试二进制的参数与环境。但 runner 改成让用例少跑，日志多半会过不了「1 passed」与计数行那几关；改成更隐蔽的，例如只动 `SINGLEFS_LAYER0_*`，这一格就会放它过去。这是推的。

**四句**：看不看得到：看得到，路径就在环境里，照 `RUSTC` 那样取内容的哈希或 `-V` 就能分辨。满足的分句：J3「还有什么改了会改测试二进制的行为而不进指纹」。跑前改法 U4：只取值，在 E1–E3 上还中。

### J3-b `CARGO_BUILD_JOBS` 不进指纹：没打中

E5（j3.log:5）照设计是相同的。并行度不改变编出来的东西（codegen-units 由 profile 定，与 jobs 无关，推的，没编两份比对）。它不进指纹是对的。不对称的地方在于：`.cargo/config.toml` 里写 `[build] jobs = N` 按文本进指纹，改它会白白重跑一趟，是假红方向。

### J3-c 主工作区与 HEAD + 暂存区 worktree 的指纹：不一定相同，但只会往假红的方向偏

E6（j3.log:6）：两边干净、环境相同，指纹相同。E7（j3.log:7）：主工作区 `src/` 下多一个没暂存的未跟踪文件，两边不同；未跟踪的测试文件会被当成「别的目标」减掉，不造成差异，所以探针改用 src 下的文件。E8（j3.log:8）：只有主工作区的上一层目录里有 `.cargo/config.toml`，两边不同，cargo 在两边读到的配置也确实不同。另外两处看代码推得出、没量的差异：设了 `CARGO_HOME` 而 `~/.cargo/config.toml` 也在的时候，主工作区一侧会按「目录层级」把 `~/.cargo/config.toml` 算进来，`/tmp` 下的 worktree 不会；`gate.sh` 自己导出 `TMPDIR`（`.claude/singlefs-ai-sop/scripts/gate.sh:180` `export TMPDIR`），它建的 worktree 与崩溃验证员照出路句 `mktemp -d` 建的 worktree，上层目录不一定相同。这些差异都只会让快档对不上标记、判红，造不出假复用。前提是 `gate.sh --staged` 与 `--full` 都在 worktree 里跑，这一点是流程文字的事，归 J5。

## J4：重型测试闸认崩溃枚举用例

**怎么喂的**：`probe_j4_hook.py` 用子 agent 身份（`agent_type=implementation-writer`，不带 `SINGLEFS_HEAVY_TESTS`），把 JSON 喂给仓里的 `.claude/hooks/heavy-test-guard.sh`，只看退出码，检出记录重定向到临时文件。除了 B2、B12，每条命令都套一层 `bash research/scripts/run-with-memory-cap.sh 8G`，这样被拒的原因只可能是「重型」，不会是「没经内存包装」。

### J4-a 绕过去的写法（B1–B13，全部放行；原样见 j4.log:8–20）

| 格 | 写法（登记目标 `record_checker_judges_absence_by_the_persisted_set`，下称 c561） | 为什么放行（出处） |
|---|---|---|
| B1 | `cargo nextest run -p singlefs-harness --run-ignored all -E 'test(=…)'` | 子命令不是 test / t 就不判：`.claude/hooks/lib_heavy_tests.py:252` `    if subcommand not in ("test", "t"):`。本机没装 nextest（`cargo nextest --version` 报 `no such command`），装上就能用 |
| B2 | `cargo mutants -p singlefs-harness -- --test c561 -- --include-ignored` | 同上。本机装了 cargo-mutants 27.1.0（`cargo mutants --version` 原样），它的用法行是 `Usage: cargo mutants [OPTIONS] [-- <CARGO_TEST_ARGS>...]`。尾参能不能原样到 libtest 是推的；不带尾参时它照样对整个包反复跑 `cargo test`，L2（j4.log:24）同样放行 |
| B3 | `cargo --config 'alias.xt="test"' xt -p singlefs-harness --test c561 -- --ignored` | 别名展开之后才是 test，闸只认命令行上写出来的子命令 |
| B4 | 前缀 `CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER=<脚本>`，命令里不带 `--ignored` | runner 替测试二进制补上 `--include-ignored`；闸只看 `--` 之后的词：`:259` `    libtest_arguments = rest[rest.index("--") + 1:] if "--" in rest else []` |
| B5 | `cargo --config 'target.x86_64-unknown-linux-gnu.runner=[…]' test …` | 同上 |
| B6 | `cp <deps 下的 c561-哈希> <草稿>/rc && … <草稿>/rc --ignored` | 按路径形状认测试二进制：`:305` `TEST_BINARY_PATH = re.compile(r"(?:^|/)deps/([A-Za-z0-9_]+)-[0-9a-f]{16}$")`；拷走、改名之后名字就丢了（这一种是「判别子观测不到」） |
| B7 | `find target/release/deps -name 'c561-*' -type f -executable -exec {} --ignored ';'` | `find -exec` 起的命令不在命令位置上 |
| B8–B13 | `/usr/bin/time -v`、`flock <锁>`、`rustup run stable`、`chrt -i 0`、`systemd-run --user --scope -p MemoryMax=8G`、`prlimit --as=…` 包在 `cargo test … -- --ignored` 外面 | 闸只剥这几种前缀：`.claude/hooks/lib_shell_words.py:37` 那一行及其上下（nohup、setsid、command、exec、nice、ionice、timeout、env、stdbuf、sudo、taskset），外加关键字 `time`；`/usr/bin/time` 是可执行文件，不是那个关键字 |

闸自己的文件头写明了看不见哪些（`.claude/hooks/heavy-test-guard.sh:71` `# 看不见的（照常放行、不记检出）：变量里拼出来的与通配（`*`、`?`、`[`）写出来的命令与脚本路径、eval、`<<<` 喂给 shell 的字符串；`；`:73` `#   make（Makefile 里起的命令）与 xargs 起的命令；…`）。B1–B13 都不在这份清单里。B8–B13 这一类早就有，层 0 同样放行（j4.log:23，L1：`/usr/bin/time -v` 包层 0 目标，退 0）。B8 与 B12 最值得提防：一个是量峰值内存的常用写法，一个是自己设内存上限的常用写法，子 agent 不带恶意也会这么写，而且这样写同时躲开了「没经内存包装」那一道（`runs_compiled_code` 只认命令词是 cargo 或测试二进制的情形）。

### J4-b 闸的前提是用例函数标了 `#[ignore]`，没有一处检查（I1、I2，量过）

今天 c561 的全量用例上一行是 `crates/singlefs-harness/tests/record_checker_judges_absence_by_the_persisted_set.rs:544` `#[ignore = "崩溃枚举（262144 个状态、每个一遍恢复）：提交时由崩溃验证员按输入哈希跑（release），平时不跑"]`（j4.log:26）。在副本里删掉这一行：
- I1：`cargo test -p singlefs-harness --test c561`（不带 `--ignored`）闸放行（j4.log:27）。这时 2^18 个状态整份会跑，这一句是推的。
- I2：同一个副本上 `admission.py crash-cases` 退 0（j4.log:28），它只核 `fn <名>(` 在不在（`research/scripts/admission.py:1060` `    function_form = re.compile(r"\bfn\s+" + re.escape(case.function) + r"\s*\(")`）。

规则那一句把前提写死了：`.claude/rules/implementation-workflow.md:58` 「libtest 参数带 `--ignored` 或 `--include-ignored` 的」。实现员调试时去掉 `#[ignore]`，或者新写一条用例时忘了标，都会让这一类不再被当成重型。名字带 layer0 的两条不受影响，因为它们按名字判。

### J4-c 误拒（R1、R2，量过，危害小）

`… --test c561 -- --ignored --list` 只列出用例，一条都不跑（j4.log:21）；`… -- --include-ignored --exact the_false_red_and_the_misaligned_hole_have_identical_crash_images_and_only_the_hole_is_missing` 只跑一条快用例（j4.log:22）。两种都被拒。闸只要看到 `--ignored` / `--include-ignored` 就算（`.claude/hooks/lib_heavy_tests.py:78` `IGNORED_TEST_ARGUMENTS = {"--ignored", "--include-ignored"}`），不管过滤之后还剩什么。今天 c561 与 floor-raise 两个目标里各只有一条 `#[ignore]`，所以「只跑别的 ignored 快用例」这种合法命令还不存在。另外，`--no-run` 只编层 0 目标也被拒（j4.log:25，L3，退 2），这是早就有的，不归这一批。

### J4 的四句

- 分不分辨臂：不适用。F6（`crash-cases` 要求登记的函数标 `#[ignore]`）在 I2 上成立（j4-fix.log:28），在 I1 上不成立（j4-fix.log:27），闸那一侧另要改。
- 看不看得到：B1–B5、B7–B13 看得到，别名、runner、包装、子命令都写在命令文本里。B4 的 runner 放在环境变量里，B5 的放在 `--config` 里，也都看得到；只有写进 cwd 下 `.cargo/config.toml` 的才要读文件。B6 看不到（拷走之后名字没了），属于「判别子观测不到」：要么改成按参数认（带 `--ignored` 加上登记的用例函数名），要么接受它，交给看门狗在进程那一层兜。看门狗能不能兜住没核，列在限度里。I1 看得到（闸可以读源码看登记函数标没标 `#[ignore]`）。
- 满足的分句：J4「子 agent 有没有一种写法绕得过去（…别名、`cargo nextest`、直接起测试二进制加 `--ignored` 的别的拼法）」与「有没有把只跑同一目标里快用例的合法命令误拒」，两个分句各有打中的格。
- 跑前改法：规格要的是「认崩溃枚举用例」，没有给具体改法可核。

## 改法（只在我的模型上量过、被攻过零轮）

| 改法 | 修哪几格 | 量过 / 推的 |
|---|---|---|
| **F1**（54 号，补丁 `fix-f1-54-layer0-replay.patch`）：范围那一问的前缀在登记路径之外，再加 `.claude/gate.d/stage-inputs.tsv`、跑的这一份 54 号、准入模块（按 `realpath --relative-to=$ROOT` 取相对路径）；`write_crash_case_manifest` 再加一个 `--extra-file "<判它的准入模块：admission.py>" "$layer0_admission_module"` | A1、A2、A3、A4 | **量过**：副本上 `probe_j1_stage.py` 由 6 格不成立变成 0 格（原样 `SUMMARY J1 54 号: cells=15 attack_cells_broken=0 control_cells_broken=0`，j1-fix.log:16）。副本上的 `admission.py --selftest` 139/140：唯一红的那一格是「54 号 --full 设续跑的环境变量」，那一格的 `expected_fingerprint` 照旧只带 54 号那一个 `--extra-file`，要跟着 F1 改（`outputs/selftest-admission-f1-f6.log`）。另外，stage-inputs.tsv 里 54 号那一行最好也加上 `research/scripts/admission.py`，让 stage-must-run 的复用判定同样看得到它（推的，没量） |
| **F6**（admission.py，补丁 `fix-f6-admission.patch`）：`crash_case_test_files` 找到用例函数之后，再核它前面那串属性里有没有 `#[ignore`，没有就报登记错（`crash-cases` 退 2，54 号快档与 `--full` 开头都红） | I2 | **量过**：j4-fix.log:28 成立。副本自证除上面那一格外都过；四条真用例都标了 `#[ignore]`，副本上自查通过（同一份日志里「真仓的登记表自查」那一格是绿的） |
| F7（闸）：`cargo_use` 除 test / t 之外，把 nextest（`--run-ignored`）、mutants、miri、llvm-cov、hack 也当成跑测试的子命令；`--config` 里有 `alias.` 或 `.runner`、环境里有 `CARGO_TARGET_*_RUNNER` 的，一律当成可能带 `--ignored`；前缀表补 `/usr/bin/time`、`flock`、`rustup run`、`chrt`、`prlimit`、`systemd-run`、`strace`、`perf` | B1–B5、B8–B13 | 推的，没实现 |
| F8（闸）：命令词是任意可执行文件、参数里有 `--ignored` / `--include-ignored`，且（`--exact` 之后的名字，或任一参数）等于登记的用例函数名时，按崩溃枚举用例算 | B6、B7 的一部分 | 推的，没实现；只点名测试目标、不点名函数的写法照样漏 |
| F9（闸）：登记的测试目标被点名、却不带 `--ignored` 时，读源码核登记的函数标没标 `#[ignore]`，没标就按重型算 | I1 | 推的，没实现（F6 已经让门禁在提交时红，F9 是在执行前拦住） |
| F10（闸）：带 `--list` 的不算 | R1 | 推的；R2 要知道过滤之后剩下哪几条，在闸里做不到，只能接受误拒 |
| F11（admission.py）：包里有 `include!(concat!(`、或者任何一个包有 build.rs 读 `tests`，就不减；或者把这两种补进文件头「认不出的」那一段 | U1、U2 | 推的，没实现 |
| F12（admission.py）：`crates/mutations.tsv` 与没有任何 .rs 读 `CARGO_BIN_EXE_` 时的 `src/bin/*.rs` 不进用例的输入 | O1、O3 | 推的，没实现；O2、O4 要顺着模块图走，规格有意不做，照「宁宽勿窄」留着，代价写明即可 |
| F13（admission.py）：环境变量里的 RUNNER、WRAPPER，与配置文件里 `build.rustc` / `build.rustc-wrapper` / `target.*.runner` 指的可执行文件，按内容哈希进指纹（`rustc` 一类取 `-V`） | E1–E3 | 推的，没实现 |

## 推翻条件

- J1-a 被推翻，如果：gate.sh 或 gate-staged 在 54 号之外另有一道，会在只改登记表或 54 号的提交上核崩溃用例的标记；或者 77 在提交流程里被当成红。
- J1-b 被推翻，如果：`crash-case-marker-check` 复核标记时，把判日志的全部条件（工作线程、片数守恒）都重判一遍；或者 admission.py 已经以别的途径进了每条用例的指纹。
- J2-b 的代价被推翻，如果：第二条流全量实测远短于 2.3 天。
- J4-a 的 B1–B13 被推翻，如果：项目 settings 里另有一道 PreToolUse 钩子拒这些写法（我只喂了这一道闸）。

## 没打中的形状（试过什么、范围多大）

- **libtest 认不认缩写的参数**：草稿里用 `rustc --test` 编一个 5 行的小测试，逐个试 `--include-ignored`、`--include-ig`、`--include`、`--ignore`、`--ign`。只有第一个跑到了 ignored 用例，其余全报 `error: Unrecognized option`（原样，本机 stable 工具链）。这条路走不通。
- **闸认得出的写法**（j4.log:1–7，全部判对）：`--test=<名>` 的等号写法、`cargo +stable t` 加通配、直接执行 deps 下的二进制、在包目录里不带 `-p`、别的测试目标带 `--ignored`、点名目标但不带 `--ignored`。
- **续跑目录跨输入串用**：进度目录按这条用例的指纹分（`.claude/gate.d/54-layer0-replay.sh:200`），指纹里有登记行，两条用例不会共用一个目录。进度文件头另核流名与枚举计划（第三轮 U1），读 `layer0_progress.rs` 的 `open` / `restored_slices_of` 没找到串用的路径。这是推的，没造并发以外的场景去量。
- **`--start-over` 没传到、或别处漏清 `SINGLEFS_LAYER0_START_OVER`**：`--full` 用 `env -u` 或 `=1` 设它，自证有一格核这个。快档不清它，但只有两份层 0 全量用例读续跑的环境变量（`first_transaction_step_seven_layer0.rs:456`、`second_transaction_step_zero_layer0.rs:736` 各有一处 `Layer0Resume::from_environment`），而这两份在快档里被 ignore。没有漏。
- **被打断或判红之后留下作数的标记**：标记只在判日志、复算指纹都过了之后写，而且先写临时文件再换名，读的时候跳过 `.partial.`。判红的那一支删的是开跑那一批的格。没造出来。
- **54 号判红了，靠「整份进度文件读回」在下一趟洗成绿**：harness 在枚举跑完时就删了进度文件（`crates/singlefs-harness/src/crash.rs:2374` `    let progress_file_after_completion = progress_file.map(Layer0ProgressFile::finish);`，`finish` 里删），54 号那边再判线程红时，文件已经不在，下一趟从头跑。只有在最后一片写完、`finish` 之前被杀这一个窄窗口里，才剩一份完整的进度文件，而那一趟本来就没被判红。
- **运行期环境变量改测试行为**：`grep -rn 'env::var' crates` 在四条用例的路径上只读到线程数（`SINGLEFS_LAYER0_THREADS`）与续跑的三个变量，其余读环境变量的都是别的测试。线程数不改判定（多线程合并要确定，这一条是推的）。
- **注释剥离写错，把真点了名的地方剥掉**：读过 `rust_code_without_comments`，逐个对了字符字面量、生命周期、原始字符串、字符串里的 `//` 与 `/*`，没找到错的。没做模糊测试。
- **别的形状**：指向别的测试文件的符号链接（按链接指到的内容算哈希，推的）；`tests/common.rs` 这种名叫 common 的目标（`mod common;` 整词命中，自证有一格）；Cargo.toml 注释里提到目标名（Cargo.toml 不剥注释，结果是多留，不会漏）。都没打中。
- **取样范围**：J1 在一个 3 条用例的小仓上造了 4 类改动；J2 在小仓上造了 2 种写法，在真仓拷贝上对 4 条用例各改了 4 个文件；J3 在小仓上造了 8 格；J4 喂了 28 条命令。都是构造的，不是随机取样。

## 这条腿自己的限度

- 54 号是拷进小仓、拿假 cargo 跑的，假 cargo 照控制目录打日志。真 harness 在这些场景下怎么打日志（例如 J1-c 并发时 panic 的原话）没核，J1-c 整格是推的。
- J2-b 的代价用的是第三轮判决里的外推数，没重新量。O1–O4 拷的是真仓当时的 `crates/`，实七正在改它，文件数会变；四个文件的相对位置不会因此变。
- 闸只喂了 JSON，没看 `research/scripts/agent-watch.py` 在进程那一层能不能事后认出 B3、B4、B5、B8–B13 起的子进程。这几种起到最后，都是 cargo 以 `--ignored` 或 `--include-ignored` 起测试二进制，看门狗按 cmdline 多半认得出，这是推的；B6（拷走改名）它认不出。
- F1、F6 只在我的探针与副本自证上量过，被攻过零轮；F7–F13 都是推的。
- 对 J5 的文字（`.claude/main-agent.md:61`、`crash-verifier.md`）不下判断，只在 J1-a 里指出它与代码漏的是同一处。

## 没做什么

- 没执行 54 号本身，没执行任何重型测试，没枚举任何崩溃状态（本腿原型跑的崩溃状态数 = 0，不占用「约 10⁷」那个额度）。
- 没改仓里任何被判的文件。改法只打在草稿目录的拷贝上；补丁放在模型目录，供主 agent 与核查员复跑。
- 没读禁读清单里的文件（`defs-gatebatch-m2-r1-sonnet-*`、`defs-gatebatch-m2-r1-local-attack*`）。
- 没判 J5；没替主 agent 采纳任何一格。副本上量出的数不算入库装置上的数。
- 草稿目录 `/tmp/claude-1000/defs-gatebatch-r1-opus/`（两份仓副本、改法的拷贝、小编译产物、日志）交回之前整个删掉；要用的日志已经拷进模型目录的 `outputs/`。
