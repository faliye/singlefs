# merge-verify-disk-governance-r2 攻方腿报告（opus，2026-11-19）

被判对象：`scripts/gate/merge-and-verify.sh`（sha256 `75c1df5d5d8dc859a68241c9d4fae9adb3b23aa92530e75dfc7b81602fdf8d5c`，开工时 `sha256sum -c research/prompts/merge-verify-disk-governance-r2-snapshot/judged-sha256.txt` 七份全 OK）。改前基线 `~/.local/share/singlefs/work/merge-verify-drop-build-step/batch7/base/scripts/gate/merge-and-verify.sh`，用 `diff -u` 比过（草稿目录 `base-vs-now.diff`，830 行）。只攻这一轮的修补与第一轮判决写「核不动」的那一格；第一轮攻过的角度不重复。

## 复跑命令与模型文件

模型目录 `research/prompts/merge-verify-disk-governance-r2-opus-model/`：

| 文件 | sha256 | 行数 |
|---|---|---|
| `tools/rerun.sh`（从 `scripts/agents/three-way-rerun-template.sh` 拷，第 1–79 行与第 86 行起的自证一段与模板逐字相同，只换了 `rerun_steps` 那一段及它用的函数） | `dfb21fbf8fd6510da0b6f826f16dca5177256f2e5fa27d663b1a0abb609d4e76` | 397 |
| `tools/rerun-from-a-clean-work-directory.sh`（拷第一轮攻方腿模型里的同名文件，逐字相同：先删运行目录、再调同一份 `rerun.sh`） | `0fa66b1880e7c2e072b667e09db029ae34fc8f49ad06f2c7aa063f38dbf4aad4` | 24 |

复跑命令（新 shell 里起，`--cwd` 是本腿草稿目录）：

```
task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/merge-verify-disk-governance-r2-opus -- bash /home/fy5090/code/singlefs/research/prompts/merge-verify-disk-governance-r2-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/merge-verify-disk-governance-r2-opus/model-run /home/fy5090/code/singlefs/research/prompts/merge-verify-disk-governance-r2-opus-model
```

- 「冻结副本」参数填仓根：这一轮没有拷出的冻结树，`rerun.sh` 开头核 `scripts/gate/merge-and-verify.sh` 的 sha256 等于上面那一份，不等退 2、什么都不跑。
- 不涉崩溃状态，不建池、不调 `enumerate_crash_states`（第 3b 步那几条不适用）。起的是：从被判脚本里原样切出的挑目标 python 段（`MERGE_AND_VERIFY_ITEMS_PROGRAM` 的 here 文档正文），喂四棵玩具树与一棵「照本仓验证包的清单与文件名铺的空文件树」；真 cargo 只编三棵玩具工作区（没有外部依赖，`--offline`）。
- 探针的内存与时间：没量（子 agent 不跑），推的：挑目标程序在 481 个空根文件上跑两趟，几秒；cargo 共起 9 条，每条编一到三个几行的包，合计推一到三分钟，内存推不到 1 GB。运行目录 `model-run` 里有每组一个编译目录（在服务给的 `CARGO_TARGET_DIR` 底下，没给就在 `model-run/target`）。
- 跑完看：`task wait` 末行 `exit=0`；`task-server/task log <号> --all` 里 `name=summary matched=12 mismatched=0` 是预期全中。每格一行 `name=<格> … verdict=matched|mismatched`，跑前写死的预期在 `expected=` 与 `rerun.sh` 里每一格上面那行注释。

## 各格判定一览

| 攻击面 | 格 | 判定 | 依据 |
|---|---|---|---|
| `CARGO_PROFILE_DEV_DEBUG=0` 只设在 build 那一条上 | 测试与 build 两趟的产物、指纹打架 | 没打中（不打架，但 cargo 另编一份依赖：代价，不是错；模型 `debuginfo_off` / `debuginfo_kept` 待主 agent 跑） | 下文「一」 |
| 同上 | 重编核对读 build 的 Compiling 行认错包 | 没打中（与第一轮 clippy Checking 行那一格同形、前提同样够不着） | 下文「一」 |
| 同上 | 构建脚本、复用键读到这个变量 | 没打中 | 下文「一」 |
| `[[bin]]` 段有差别时挑同包集成测试 | 挑多了：验证包加删一段装置 `[[bin]]`，挑上 481 个集成测试目标（等于整包），没有一个起这个 bin | **打中**（推的，按代码读；模型 `verification_bin_*` 三格待跑） | 下文「二」第 1 格 |
| 同上 | 挑少了：只改 bin 根文件的正文，起它的集成测试照旧不挑（修补只认清单段） | **打中**（推的；模型 `bin_body_*` 四格待跑） | 下文「二」第 2 格 |
| 同上 | 挑少了：别的包的集成测试起这个 bin | 没打中（`crates/` 今天没有这个形状） | 下文「二」第 3 格 |
| 同上 | 要特性的那几个怎么记 | 顺带看到、不计数：改了的装置 bin 自己记「要特性、本次未跑」，而它的特性是缺省开着的 | 下文「二」第 4 格 |
| 新旧两侧目标集合比较 | 误退 3：新加一段名字不同、根文件就是那份自动认的文件的段（本仓 `singlefs-unit-graph` 的写法），判成「被顶掉」 | **打中**（推的；模型 `renamed_*` 三格待跑） | 下文「三」第 1 格 |
| 同上 | 漏掉被顶掉的：目录形、`autotests = false`、`[lib]` 改名、2015 版次 | 没打中 | 下文「三」第 2 格 |
| 跨台删上一次尝试的快照 | 上一次尝试还在第二台上跑时删了在用的那一棵 | 没打中（第一轮「核不动」的那一格这一轮读到底了：服务等第二台清理确认才记尝试结束、才重排） | 下文「四」 |

打中 3 处，都在挑测试目标那两块。读数都是「推的」：模型写好、没跑，判决要引的数由主 agent 复跑。另撤回第一轮攻方腿提的一条改法、部分撤回一条（下文「五」）。

## 一、`CARGO_PROFILE_DEV_DEBUG=0` 只设在 build 那一条上（没打中）

被判的：「这一条 cargo 的环境里设 CARGO_PROFILE_DEV_DEBUG=0（只设在 build 这一条上，test、fmt、clippy 不设）」（`scripts/gate/merge-and-verify.sh:137`），写法「build_command=(env CARGO_PROFILE_DEV_DEBUG=0 "${CARGO_BUILT_COMMAND[@]}")」（`scripts/gate/merge-and-verify.sh:2849`）；根清单「debug = "line-tables-only"」（`./Cargo.toml:32`）。次序是测试、fmt、clippy、build，之后验证包那一步交门禁（`--release`，另一档）。

1. 产物与指纹不打架，但依赖另编一份。cargo 把档的设置（调试信息在内）算进每个单元的元数据摘要（推的：按 cargo 的做法记的，没在这一版 cargo 1.98.0 上跑过），测试那一趟（`line-tables-only`）与 build 那一趟（`0`）是两套单元、两套产物文件名，互不覆盖、互不判旧，所以不会出现「build 把测试的产物换掉、测试读到另一档的库」。代价：测试那一趟编过的依赖，build 那一趟再编一份不带调试信息的。模型 `debuginfo_off` 跑前写死 `build_compiling_leaf=1 leaf_rlibs=2`，对照臂 `debuginfo_kept`（build 不设变量）写死 `0` 与 `1`；读出来 off 那一臂是 `0` 与 `1`，「另编一份」撤回。
   - 这一条进「交用户的」那一类，不是错：文件头按「调试信息只占盘」（`scripts/gate/merge-and-verify.sh:137`）、「占每个测试二进制的 73%–94%」（`scripts/gate/merge-and-verify.sh:138`）算省，没算依赖那一层不再与测试那一趟共用，验证包这类依赖多的包要多编一整套依赖、多跑一次各包的构建脚本（验证包的构建脚本要装整个工作区算键）。净省多少要真 cargo 在本仓量，模型玩具的 `target_kib=` 不外推。
2. 重编核对读 build 的 Compiling 行：那几行是另一套单元（关掉调试信息的那一套）打的。在重编核对要防的那段历史上（测试那一趟的产物被 cargo 判新鲜、没重编），build 照编、打出 Compiling 行，核对按「任一份日志里有就算」放过——但这与第一轮攻过的 clippy Checking 行同形：「靠 build 与 clippy 那两趟的行证明这一趟重编了」（`scripts/gate/merge-and-verify.sh:166`），clippy 那一趟同样是另一套单元、对工作区每个成员都打一行，并集里多一份同类证据不改结局；那段历史怎么在合入后验证里造出来，第一轮已判够不着（快照每趟新拷、路径每趟不同）。不重复攻，记没打中。
3. 构建脚本与复用键读不读这个变量：验证包的构建脚本算工具链行，「debug（调试信息）不改编出来的代码跑出的结果，不进键」（`crates/singlefs-unit-graph/src/build_toolchain.rs:17`），登记 `rerun-if-env-changed` 的只是它读的那几项（`crates/singlefs-verification/build.rs:289` 那一行按 `PROFILE_SETTINGS_READ` 逐个打）；准入键按键名去掉 `_DEBUG`（第一轮攻方腿已核，`scripts/gate/admission.py` 的 `profile_variable_left_out`）。build 那一趟设的变量不进测试那一趟的键，也不让测试那一趟的构建脚本重跑。

试过、够不着的走法：调用方环境里本来就设了 `CARGO_PROFILE_DEV_DEBUG`（测试与 clippy 照它编，build 那一条 `env` 盖成 0；与上面第 1 点同形，只是测试那一档换了）；`.cargo/config.toml` 的 `[profile]` 或 `rustflags` 里写调试信息（仓里那一份只有 `[env]` 的 `BINDGEN_EXTRA_CLANG_ARGS`，现读过）；`[profile.dev.package."*"]` 一类按包盖掉的写法（根清单里没有，`grep -n 'profile' Cargo.toml` 只命中 `[profile.release]` 与 `[profile.dev]` 两段）；`--all-targets` 里的 bench 走 bench 档（dev 下 cargo 给测试类目标用 test 档，继承 dev，变量照样管到；`crates/` 下没有 bench）。

## 二、`[[bin]]` 段有差别时挑同包集成测试

被判的：「[[bin]] 段有差别（新加、改了、删掉）的，另把同包的集成测试目标（tests/ 下的，要特性的记「要特性」同上）一并挑上」（`scripts/gate/merge-and-verify.sh:110`），理由「集成测试经 CARGO_BIN_EXE_<名> 起包里的 bin，bin 的根文件或名字改了，起它的集成测试跟着变」（`scripts/gate/merge-and-verify.sh:111`）。挑法是同包每个 `--test` 目标、不看它起不起这个 bin：「test_roots = [root for root, found in targets_of[package].items() if found.selector[0] == "--test"]」（`scripts/gate/merge-and-verify.sh:2148`）。

### 第 1 格：验证包加删一段装置 `[[bin]]`，挑上整包 481 个集成测试目标（打中）

本仓的形状：验证包的装置二进制逐个写段，「# 装置二进制逐个声明，只为挂 required-features（路径与名字同自动发现的那一份）」（`crates/singlefs-verification/Cargo.toml:57`）；`git log --since=2026-10-01 -p -- crates/singlefs-verification/Cargo.toml | grep -c '^+\[\[bin\]\]'` 现跑得 26。加一个装置、收掉一个旧装置，都是「`[[bin]]` 段有差别」。验证包 `tests/` 下 481 份 `.rs`（`ls crates/singlefs-verification/tests/*.rs | wc -l` 现跑），写 `CARGO_BIN_EXE_` 的 `.rs` 0 份（`grep -rlF --include='*.rs' 'CARGO_BIN_EXE_' crates/singlefs-verification | wc -l` 现跑得 0）。

走法（模型 `verification_bin_section`：树里是验证包此刻的清单与各根文件的名字，文件都是空的）：

1. 补丁只往 `crates/singlefs-verification/Cargo.toml` 末尾加一段 `[[bin]] name = "e999_model_apparatus"`、`path = "src/bin/e999_model_apparatus.rs"`、`required-features = ["apparatus-binaries"]`（根文件补丁之前就在树里，补丁不碰它）。
2. 被判程序：段外的键没差别；新段与旧侧自动认的那一个同名同根文件，不算顶掉；这一段的根文件落到「要特性」；`bin_names` 非空，照第 2148 行挑上同包每个 `--test` 目标。预期 481 个 `ITEM`、一句 `NOTE`「集成测试目标 481 个」、`REQUIRES` 一行点名 `e999_model_apparatus`，不退 3。
3. 主 agent 照「第一列逐个写成 `--item <项>`」（`.claude/skills/merge-verify/SKILL.md:19`）点 481 项交合入后验证：测试那一步对验证包起 481 个目标（`--release`），验证包那一步把 `test=` 落在这些目标里的登记用例逐项交门禁（`.claude/gate.d/stage-inputs.tsv` 里 `crash-case:` 行 111 条，`grep -nF 'crash-case:' … | wc -l` 现数），全域。
4. 该拦而没拦的那一句：「不跑整个包，也不起门禁的整格」（`scripts/gate/merge-and-verify.sh:9`）与「日常开发的测试不跑整个包：提交阶段的门禁之外，测试命令只点名这一次改到的测试函数与项」（`.claude/agent-common.md:71`）。这一趟就是整包跑，却以 items 种类、不带 `--whole-package-confirmed` 走，不经用户确认。改前（第一轮被判的那一版）同一补丁只挑 `--bin`；再早一版退 3、交用户定整包。

跑前写死：`verification_bin_intact` 的 `picked_test_targets` 等于 `tests_rs` 且不少于 400、`note_with_count=1`、`changed_bin_requires=1`；`verification_bin_unpicked`（同一补丁、弄坏开关 `manifest-bin-tests-unpicked` 那一臂）`picked_test_targets=0`；`verification_bin_executable_users` `files=0`。推翻条件：intact 那一臂挑上的 `--test` 目标不到 400 个，这一格撤回。

四句：

1. 分不分辨臂：分辨。同一补丁，不挑集成测试那一臂 0 个 `--test` 目标、修补那一臂 481 个；只挑字面写了 `CARGO_BIN_EXE_<名>` 的那一臂（下文「六」改法 A）0 个（推的，没实现、没跑）。
2. 系统当时看不看得到判别它的东西：看得到。集成测试的源码在快照里，被判程序本来就按 mod / `#[path]` 链读每个目标的文件（`files_reached_from`），`CARGO_BIN_EXE_<名>` 是 `env!` 里的字面量；本仓 `singlefs-unit-graph` 就按这个前缀认边：「} else if let Some(binary_name) = variable.strip_prefix("CARGO_BIN_EXE_") {」（`crates/singlefs-unit-graph/src/rust_items.rs:2008`）。
3. 满足的是判据字面的哪一分句：背景材料「问题」第 2 条第二分项「挑多了（不用这个 bin 的集成测试）」。
4. 改法在这一格上中不中：第一轮判决改法 2 就是现在的做法，这一格就是它打的；第一轮攻方腿那一条（同包集成测试与 bench 一并挑上）同样中，撤回（下文「五」）。

### 第 2 格：只改 bin 根文件的正文，起它的集成测试照旧不挑（打中）

修补的理由句写「bin 的根文件或名字改了，起它的集成测试跟着变」，做法只接在清单段那一支（`land_manifest_changes`）；改 bin 根文件正文的补丁走的是「正是一个测试目标的根文件（tests/<目标>.rs、tests/<目标>/main.rs、src/bin/<名>.rs、src/main.rs、[[test]] / [[bin]] 写的 path）→ 那个目标」（`scripts/gate/merge-and-verify.sh:87`），落法「if relative in targets and relative != rerun.DOC_TARGET:」（`scripts/gate/merge-and-verify.sh:2056`）只交这个 bin 自己。

本仓形状：`crates/singlefs-unit-graph/src/bin/unit_graph.rs` 是 `[[bin]] unit-graph` 的根文件，同包两份集成测试起它，其一「Command::new(env!("CARGO_BIN_EXE_unit-graph"))」（`crates/singlefs-unit-graph/tests/unit_graph_command_line_prints_one_item_per_line.rs:11`）。补丁改 `unit_graph.rs` 的参数解析或输出格式，合入后验证只跑 `--bin unit-graph`、退 0，两份集成测试要到提交时 `cargo test --all` 才红。

走法（模型 `bin_body`）：包 `alpha` 的 `[[bin]] tool`（`src/bin/tool.rs` 打 `hello`），`tests/cli.rs` 经 `CARGO_BIN_EXE_tool` 起它、要 `hello`；补丁只把 `tool.rs` 改成打 `changed`。跑前写死：`bin_body_before_cli` 退 0；`bin_body_items` 恰好 `alpha:src/bin/tool.rs|--bin tool`、不退 3；`bin_body_after_picked` 退 0；`bin_body_after_cli` 退非 0。推翻条件：`bin_body_items` 挑上了 `alpha:tests/cli.rs`。

第一轮攻方腿报告顺带写过这个病根（「同一个病根早就有：」那一段，`research/prompts/merge-verify-disk-governance-r1-opus-output.md:107`），判决没收。四句：这一格不分辨改前改后两臂（两臂都只挑 `--bin`），按 `.claude/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」是「打中不分辨臂」，病根在两臂共用的「按根文件位置落」；分辨的是改法 A 那一臂（按字面找 `CARGO_BIN_EXE_<名>`，正文改了与清单段改了走同一条）；系统看得到（同上一格）；满足「挑少了」那一分句；第一轮判决改法 2 在这一格上照中。

### 第 3 格：别的包的集成测试起这个 bin（没打中）

`grep -rnE 'CARGO_BIN_EXE|current_exe|"cargo"|CARGO_TARGET_DIR|target/debug' crates --include=*.rs` 现跑：`CARGO_BIN_EXE_` 只出现在 `singlefs-unit-graph` 自己的两份集成测试与它解析这个写法的源码里；起别的包的程序的只有两个装置 bin 与 `unit_graph.rs` 读 `CARGO` 去起 cargo，都不是集成测试。`crates/` 今天没有这个形状，记没打中（它不是 `cargo test` 能交路径的写法：cargo 只给同包的集成测试与 bench 交 `CARGO_BIN_EXE_<名>`）。

### 第 4 格：要特性的那几个怎么记（顺带看到，不计数）

「写了 required-features 的 [[test]] / [[bin]]（cargo test 不带特性不编）记「要特性」」（`scripts/gate/merge-and-verify.sh:94`），目标清单照「wanted = entry.get("test", section in ("bin", "test")) and not entry.get("required-features")」（`scripts/gate/rerun-failed-tests.py:253`）把它们排掉；而验证包的那个特性缺省开着：「default = ["apparatus-binaries"]」（`crates/singlefs-verification/Cargo.toml:50`），`cargo test -p singlefs-verification` 不带特性照编、照跑这些 bin 的单元测试。装置 bin 里 25 个写了 `#[test]`（`grep -c '#\[test\]' crates/singlefs-verification/src/bin/*.rs | grep -vc ':0$'` 现跑），`e171_three_oracle_collusion.rs` 一份 56 个。改它的正文，合入后验证报「要特性、本次未跑」、退 0。这一条是改前就有的判法（第 94 行不在这一轮的 diff 里），这一轮的 `[[bin]]` 那一段照它记：第 1 格那一趟里改了的那个 bin 自己本次未跑，同包 481 个不相干的目标照跑。不计数，写在这里交主 agent 判要不要另立一题。

## 三、新旧两侧目标集合比较

被判的：「旧侧自动认的目标在新侧没有、或同名的目标根文件换了（新加一段 [[test]] 与它同名、或一段点了它的根文件），cargo test 从此不跑那份文件、一句不报」（`scripts/gate/merge-and-verify.sh:108`）。两侧集合以（段，目标名）为键：自动认的只在「if os.path.isfile(os.path.join(directory, root)) and (section, name) not in found and root not in declared_roots:」（`scripts/gate/merge-and-verify.sh:1884`）时进集合；新侧没有这个键就判顶掉，「change = f"改后同名的 [[{section}]] 段指到 {after[0]}" if after is not None」（`scripts/gate/merge-and-verify.sh:1894`）那一句的另一支写「改后没有这个目标（它的根文件被别的 [[{section}]] 段点了名）」，整份判不出：「return f"自动认的目标被顶掉，cargo test 从此不跑它的根文件、一句不报」（`scripts/gate/merge-and-verify.sh:2126`）。

### 第 1 格：段换了个名字、根文件就是那份自动认的文件，判成被顶掉、退 3（打中）

cargo 的规矩（推的，按 cargo 推目标的做法记的；模型用真 cargo 核）：写了的段点了某份文件，自动发现就不再按文件名另认它，但这份文件照样是目标，只是名字换成段里写的。被判程序按名字比：旧侧的（bin，`ug_tool`）在新侧没有了，就说「cargo test 从此不跑它的根文件」，而 cargo 照新名字 `ug-tool` 编它、跑它里面的测试。

本仓就有这个写法：`crates/singlefs-unit-graph/Cargo.toml` 的「name = "unit-graph"」（`crates/singlefs-unit-graph/Cargo.toml:18`）、「path = "src/bin/unit_graph.rs"」（`crates/singlefs-unit-graph/Cargo.toml:19`），自动发现给这份文件的名字是 `unit_graph`。补丁给哪个包照这个样子加一段（给 bin 起带连字符的名字、给一份测试改个名字或挂特性），合入后验证退 3。

走法（模型 `renamed_targets`）：包 `ug` 里 `src/bin/ug_tool.rs`、`tests/foo.rs` 原是自动认的（名字 `ug_tool`、`foo`），补丁只往 `Cargo.toml` 加 `[[bin]] name = "ug-tool" path = "src/bin/ug_tool.rs"` 与 `[[test]] name = "foo-renamed" path = "tests/foo.rs"`。跑前写死：`renamed_items` 一行 `REFUSE`，同一行点名「bin 目标 ug_tool」与「test 目标 foo（」；`renamed_bin_list`（`cargo test -p ug --bin ug-tool -- --list`）列出 `tests::bin_unit_test_still_runs: test` 一行；`renamed_test_list`（`--test foo-renamed`）列出 `foo_still_runs: test` 一行。推翻条件：两份 `--list` 退非 0 或列不出（cargo 真的不跑这两份文件了），这一格撤回、被判的那一句成立。

同一类、没进模型的写法（推的）：段里 `path = "./tests/foo.rs"`。「return [entry["path"]] if isinstance(entry.get("path"), str) else default_root_paths」（`scripts/gate/merge-and-verify.sh:1809`）原样取路径、不规整，新侧同名目标的根文件 `./tests/foo.rs` 不等于旧侧的 `tests/foo.rs`，判「改后同名的段指到 ./tests/foo.rs」、退 3；cargo 按路径分量比，是同一份文件。

四句：

1. 分不分辨臂：分辨。按根文件比的那一臂（第一轮攻方腿那一条写的是「新旧两侧各算一遍 `test_targets_of_package`，旧侧是目标、新侧不是」，那张表以根文件为键）在这一格上不退 3（推的，没实现）；现在按（段，名字）比的那一臂退 3。
2. 系统看不看得到：看得到。新侧集合里这份根文件就在（以别的名字），比根文件就知道它还是目标。
3. 满足的分句：背景材料「问题」第 2 条第三分项「会不会把该挑的判成被顶掉（误退 3）」。
4. 改法：第一轮判决改法 3 写「改后少了的（被同名段顶掉的自动目标）退 3」，实现成按名字比，这一格就落在实现与「改后少了的」之间。退 3 是保守的一边，不放过错；后果是一趟白跑、要用户定整包，原因句写的那半句不成立。

### 第 2 格：漏掉被顶掉的（没打中）

逐个试过、都照文件头退 3 或照规矩认了的：

- 目录形 `tests/<名>/main.rs`：「elif os.path.isfile(os.path.join(absolute, name, "main.rs")):」（`scripts/gate/merge-and-verify.sh:1881`）照收进候选，同名段指到别处照判顶掉。
- `autotests = false` / `autobins = false`、`[lib]` 改名、改版次：都是 `[package]` 或 `[lib]` 的差别，先在段外的键那一关退 3（`scripts/gate/merge-and-verify.sh:1844` 那一句「段有改动」）。2015 版次下写一段就关掉自动发现那一条，`crates/` 下十一个包都是 `edition = "2021"`（`grep -n '^edition' crates/*/Cargo.toml` 现跑），补丁要改版次又先退 3，够不着。
- 同名同根文件、只加 `test = false` 或 `required-features`：旧目标从 `cargo test` 的缺省集合里出去，脚本照「required-features 与 test = false 不影响这个集合）」（`scripts/gate/merge-and-verify.sh:107`）不当顶掉，但 changes 那一支报一句「现在不是 cargo test 会跑的目标（段删掉了或写了 test = false），不跑」（`scripts/gate/merge-and-verify.sh:2142`），或记要特性、报本次未跑；报了，不是「一句不报」。
- 删掉一段、让原来被它按名字压住的另一份自动文件冒出来：冒出来的那一份根文件不在补丁里、不被挑（推的）。要树里先有一份被压住的文件；`crates/` 下写了段的两个包（`singlefs-background-gpu-tests` 的五段 `[[test]]` 不写 path、根文件就是同名的那一份，`singlefs-unit-graph` 的 `[[bin]]` 点的就是自动那一份）都没有被压住的另一份文件，今天够不着。

## 四、跨台删上一次尝试的快照（没打中；第一轮「核不动」的那一格这一轮读到底）

被判的删法这一轮没改：删之前「核的只是本机的进程」（`scripts/gate/merge-and-verify.sh:54`）。日志目录在两台共享的工作根下，上一次尝试在第二台、这一次在本机时，本机看不见第二台的进程。要打中，得服务在上一次尝试的第二台那一趟还没停的时候就起下一次尝试。读服务的收尾与重排（都是推的：按代码读到底，没跑）：

1. 第二台的每一次尝试都有清理回执：「peer_cleanup_file = os.path.join(directory, "peer-cleanup.json") if placement.host == PEER else ""」（`task-server/task_server.py:9207`）；起远端命令之前先记 pending：「multi_host_record_cleanup pending starting」（`scripts/hosts/multi-host-run.sh:1448`），文件头写「起远端命令之前先落 pending，确认进程停下并核目录没有持有者后才写 confirmed」（`task-server/task_server.py:133`）。
2. 收尾先核回执：「peer_clean, peer_note = self.confirm_peer_cleanup(attempt)」（`task-server/task_server.py:5995`），「if not peer_clean or not local_clean:」（`task-server/task_server.py:6001`）那一支记 cleanup_pending 就返回，「状态机不变：清理没确认的照旧保持 starting/running、记 cleanup_pending」（`task-server/task_server.py:140`）。只认「if record and record.get("status") in ("not-started", "confirmed"):」（`task-server/task_server.py:6121`）。
3. 定去向（重排就在这里，`retry_decision`）只在那之后：「self.settle_attempt(task, attempt, result)」（`task-server/task_server.py:6077`）；补收尾的 `settle_ended_attempts` 只补已经记成 ended 的。取消那一侧停单元之后同样先 `confirm_peer_cleanup`（`task-server/task_server.py:10174`）。

所以下一次尝试起来之前，上一次尝试在第二台的进程已确认停了、目录没有持有者，删它的快照删不到在用的那一棵。`peer-run-failed`（ssh 被重置、第二台那一层没跑成）走的是同一个收尾，回执 pending 就一直不重排。没有回执就放行的那一支「if not path:   # 升级前的尝试没有此协议」（`task-server/task_server.py:6111`）只给协议之前建的尝试，新尝试在第 9207 行都建了回执。

顺带看到的一种撞名（推的，概率极小，不计数）：缺省日志目录是 `<日期>-<进程号>`（`scripts/gate/merge-and-verify.sh:5222`），两台的进程号各管各的。同一天两台撞上同一个进程号时，后起的那一趟见快照位置已有东西就退 2、不删（`scripts/gate/merge-and-verify.sh:5225` 那一句「已经有东西，不往里铺、也不删它」）；只有后起的那一趟恰在先起的那一趟建好日志目录、还没拷快照的那几秒里起，两趟才共用一个日志目录、`task-attempt` 被改写，之后同一任务的重试才可能按改写的那一行删到别人的快照。

## 五、撤回第一轮攻方腿给的方案

1. **撤回**：「`[[bin]]` 段有差别时，同包的集成测试与 bench（cargo 交 `CARGO_BIN_EXE_` 的那几类目标）一并挑上」（`research/prompts/merge-verify-disk-governance-r1-opus-output.md:123`）。第一轮判决改法 2 照它实现（「挑测试目标：`[[bin]]` 段有差别时，把同包的集成测试一并挑上。」，`research/prompts/merge-verify-disk-governance-r1-main-verification.md:31`）。依据：上文「二」第 1 格，本仓 `[[bin]]` 段最多的包恰是验证包，加删一个装置就把整包 481 个集成测试目标连同登记的崩溃枚举用例挑上，而一个起这个 bin 的都没有；第 2 格，这个按包整挑的做法连「改 bin 正文」那条更常见的来路都没接上。换成下文「六」改法 A（零轮）。
2. **部分撤回**：「恢复 build、这一步另设 `CARGO_PROFILE_DEV_DEBUG=0`」那一行的盘那一格「推的：比只留行号表再少 `.debug_line`」（`research/prompts/merge-verify-disk-governance-r1-opus-output.md:79`）。那一格只比了可执行里的调试段，漏了关掉调试信息的 build 不再与测试那一趟共用依赖、要另编一整套（上文「一」第 1 点，模型 `debuginfo_*` 两格待跑）；省不省、省多少，那一句不再作数。同一格后半句「只设在 build 那一条上也不碍测试那一步的键」照旧成立（上文「一」第 3 点）。
3. 不撤回、但实现偏了：「新旧两侧各算一遍 `test_targets_of_package`，旧侧是目标、新侧不是（且不是被删掉的那一段自己的根文件）的，退 3 或至少打一句 NOTE」（`research/prompts/merge-verify-disk-governance-r1-opus-output.md:124`）那张表以根文件为键；实现成以（段，名字）为键，多出了上文「三」第 1 格的误退 3。

## 六、我提的改法（只在我的模型上设计过、没实现、被攻过零轮）

- 改法 A：`[[bin]]` 段有差别、或补丁改到 bin 根文件（连同它 mod / `#[path]` 链上的文件）时，只挑同包集成测试里 mod / `#[path]` 链走得到的文件中字面写了 `CARGO_BIN_EXE_<旧名>` 或 `CARGO_BIN_EXE_<新名>` 的那几个（被判程序已有 `files_reached_from`）；包里有集成测试用非字面的写法拼这个名字（`concat!` 拼进 `env!`、`option_env!`），整份退 3。
- 改法 B：两侧集合以根文件为键：旧侧自动认的根文件在新侧不是任何目标的根文件、文件又还在，才算顶掉；同一根文件换了名字的，照新名字落到目标；比之前路径先规整（`os.path.normpath`）。
- 改法 C（不是修错，是省代价的取舍）：build 与测试用同一档，让依赖只编一份：测试那一步也设 `CARGO_PROFILE_DEV_DEBUG=0`（代价：测试崩溃时回溯没有行号；panic 消息里的文件与行号照有），或 build 不设（可执行带行号表）。哪个省盘、省时，要真 cargo 在本仓量。

| 改法 | 「二」第 1 格（挑多） | 「二」第 2 格（改正文挑少） | 「三」第 1 格（误退 3） | 「一」依赖另编一份 |
|---|---|---|---|---|
| A | 推的，待主 agent 跑（验证包 0 份写 `CARGO_BIN_EXE_`，推出挑 0 个） | 推的，待主 agent 跑（`tests/cli.rs` 字面写了 `CARGO_BIN_EXE_tool`，推出挑上） | 不修 | 不修 |
| B | 不修 | 不修 | 推的，待主 agent 跑（`tests/foo.rs` 新侧仍是 `foo-renamed` 的根文件，推出不退 3、挑上 `--test foo-renamed`） | 不修 |
| C | 不修 | 不修 | 不修 | 推的，待量（模型 `debuginfo_kept` 那一臂就是「build 不设」的玩具版） |
| 退回改前（清单一改就退 3） | 修（退 3） | 不修 | 修成另一种退 3 | 不修 |

## 没打中的形状

- 「一」那一块：调用方环境里本来就设了 `CARGO_PROFILE_DEV_DEBUG`、`.cargo/config.toml` 里写调试信息、按包盖档、bench 档、构建脚本读 `DEBUG`、准入键读这个变量、build 之后的验证包那一步（`--release`，另一档）受影响——都够不着或不改结局；重编核对那一格与第一轮 clippy Checking 行同形，不重复。
- 「二」那一块：别的包的集成测试起这个 bin（`crates/` 下没有）；要特性的那几个记法在验证包上是错的，但那是改前就有的判法，不计数。
- 「三」那一块：目录形、`autotests` / `autobins`、`[lib]` 改名、版次、只加 `test = false` 或 `required-features`、删段让被压住的文件冒出来（今天没有被压住的文件）。
- 「四」那一块：`peer-run-failed`、取消、没有回执的旧尝试、两台进程号撞名，都推不出删到在用的那一棵（撞名那一种要几件小概率的事同时发生，不计数）。

取样范围：模型四组、五棵树、四份补丁（验证包加一段 `[[bin]]`、玩具包加两段改名的段、玩具 bin 改正文、三包工作区两臂）；没扫补丁的别的写法（删一段装置 `[[bin]]`、改 `[[bin]]` 的 `required-features`、在 `singlefs-background-gpu-tests` 加 `[[bin]]` 连带挑上五个要真卡的测试），没扫别的 cargo 版本。

## 这条腿自己的限度

- 三处打中与「一」那一块的代价，读数都是推的：模型写好、没跑（子 agent 不交任务）。cargo 的两条规矩（档的调试信息进单元的元数据摘要；写了的段点了某份文件，这份文件照新名字仍是目标）是按 cargo 的做法记的，没在 cargo 1.98.0 上跑过，模型就是用来坐实或推翻它们的。
- 「二」第 1 格的模型树只有验证包的清单与空的根文件，挑上的个数从被判程序读出；真跑一趟合入后验证要多久、主 agent 会不会在点 481 项之前停下来问，没量也没核（SKILL 第 19 行只写逐个照写）。
- 「四」只读代码，没造第二台失联再恢复的历史跑；读的是此刻的 `task-server/task_server.py` 与 `scripts/hosts/multi-host-run.sh`，它们不在被判之列，别的会话可能改。
- 背景材料「问题」第 2 条列的四块都答了；只这一次抽样，没打中的几块按 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」只算一次。

## 要主 agent 跑的命令

1. 模型整份复跑（`rerun-from-a-clean-work-directory.sh` 先删运行目录，再调同一份 `rerun.sh`；判决要引的数都从这一趟取）：

   `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/merge-verify-disk-governance-r2-opus -- bash /home/fy5090/code/singlefs/research/prompts/merge-verify-disk-governance-r2-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/merge-verify-disk-governance-r2-opus/model-run /home/fy5090/code/singlefs/research/prompts/merge-verify-disk-governance-r2-opus-model`

   - 在哪个目录起：`/home/fy5090/.local/share/singlefs/work/merge-verify-disk-governance-r2-opus`（submit 的 `--cwd`）。
   - 跑完看：末行 `TASK_SERVER_RESULT` 的 `type=succeeded`、`exit=0`；`task-server/task log <号> --all` 里 `name=summary matched=12 mismatched=0`。`exit=2` 且日志点名 sha256 的，是被判脚本换过了，不是模型的结论。
   - 哪一行 `verdict=mismatched` 推翻什么：`verification_bin_intact` 的 `picked_test_targets` 不到 400 → 撤回「二」第 1 格；`bin_body_items` 挑上了 `alpha:tests/cli.rs`，或 `bin_body_after_cli` 退 0 → 撤回「二」第 2 格；`renamed_bin_list` / `renamed_test_list` 退非 0 或 `hits=0` → 撤回「三」第 1 格（被判那一句成立）；`renamed_items` 的 `refusals=0` → 被判程序不退 3，同样撤回；`debuginfo_off` 读出 `build_compiling_leaf=0 leaf_rlibs=1` → 撤回「一」的「另编一份依赖」与「五」第 2 条。
   - 跑完删运行目录 `/home/fy5090/.local/share/singlefs/work/merge-verify-disk-governance-r2-opus/model-run`（五棵树、日志；编译目录在服务给的 `CARGO_TARGET_DIR` 底下的照 `task-server/task clean <号>` 删）。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；没往统一任务服务交任务，判决要引的数没复跑（命令在上一节）。
- 没在副本上改被判脚本、没实现改法 A、B、C，改法表里每一格都是推的。
- 「二」第 4 格（要特性的记法在验证包上是错的）是改前就有的判法，不在这一轮的修补里，只写出来交主 agent 判要不要另立一题。
- 草稿目录里只有 `base-vs-now.diff`（`diff -u` 改前基线与此刻被判脚本），没有仓副本、编译目录或库拷贝，交回前不用删；`model-run` 由主 agent 那一趟建、主 agent 删。

## 引文核对

`python3 scripts/agents/cite-check.py research/prompts/merge-verify-disk-governance-r2-opus-output.md` 退 0，末行原样：

```
  ✓ 核了 42 处引文、9 处裸行号，对上 42 处，没判 9 处（其中裸行号没有引文可比 9 处；逐处列在上面）
```

没判的 9 处是只写了文件与行号、没接引文的位置，或引文里套着「」、脚本没认成引文的（`scripts/gate/merge-and-verify.sh` 第 94、110 行那两句）；那两句原文已用 `grep -nF` 逐字核过、各命中 1 次。
