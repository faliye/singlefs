# 第一轮全工程查复用键：crates 里的 Rust 侧（路子 crates）

日期 2026-10-15。被读的树：工作区 HEAD c1e9b324，`git status --porcelain -- crates` 输出为空（crates 下没有未提交改动）。
只读主工作区：没起 cargo、门禁、测试二进制，没动任何判定库与标记，没写源码摘要缓存。草稿目录里建过一份 `crates/`+`Cargo.toml`+`Cargo.lock` 的副本（`copy/`）只给 python 算摘要用，交回前删掉。
标注：「量的」= 读到算键 / 比键的那一行代码，或下面第 1 节那个已校验过的移植算法在副本上算出来的数；「推的」= 由代码读法推出、没观测过。

判据（派发提示原文）：剪枝的单元是函数；一条判定沿不沿用只许看它依赖的那些函数单元各自去注释后的指纹、工具链、解析出的依赖版本。①–⑧ 照派发提示。

## 1 核法与原样输出

### 1.1 校验路子：把 `judging_closure.rs` 的闭包与摘要算法照抄成 python，算出来的表与编译时 `build.rs` 写进 `OUT_DIR` 的表逐行相等

移植脚本 `tools/closure_port.py`（草稿目录里，只读树、不写树）。编译产物里最新的一份表是 `target/release/build/singlefs-checker-tier-d6a124019c061d7a/out/conclusion_code_digests.tsv`。

```
$ python3 -I tools/closure_port.py /home/fy5090/code/singlefs gpu | head -1
gpu_judge d17732ad426081b768ec15e68f294b83ef03a68486e6d0bd56e7145d997b992a
$ awk -F'\t' '$1=="gpu_judge"' target/release/build/singlefs-checker-tier-d6a124019c061d7a/out/conclusion_code_digests.tsv
gpu_judge	-	d17732ad426081b768ec15e68f294b83ef03a68486e6d0bd56e7145d997b992a
$ python3 -I tools/closure_port.py /home/fy5090/code/singlefs table | sort > table-port.sorted   # 编译表去掉表头同样排序
137
137
diff_rc=0
```

全表 137 行（GPU 一行，68 份候选文件各 CPU、观察者两行；`ls tests/*.rs tests/*/mod.rs src/bin/*.rs | wc -l` 是 67，加 `crash_injection.rs` 68）逐行相等。所以下面凡是「移植算法在副本上算」的数，与 Rust 那一份同一算法（量的：两份不共用代码，结果逐字节相同）。

### 1.2 GPU 判器闭包收了哪些文件（量的，移植算法）

```
$ grep -c '  file' gpu-closure.txt
53
      4 singlefs-checker
     10 singlefs-checker-tier
     32 singlefs-core
      1 singlefs-format
      5 singlefs-harness
      1 singlefs-replay-prefix
  file raw crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
  file raw crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl
  file raw crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
  file raw crates/singlefs-checker-tier/src/crash_judge_common.wgsl
  file raw crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl
  stopped crates/singlefs-checker-tier/src/lib.rs
  stopped crates/singlefs-core/src/lib.rs
```

core 那 32 份里有写路径与挂载：`transaction.rs`、`make_filesystem.rs`、`mount.rs`、`mounted_session.rs`、`unlink.rs`、`admission.rs`、`write_accounting.rs`、`write_request_split.rs`（`gpu-closure.txt` 全列）。

### 1.3 反推：在副本里造改动，看哪一把摘要动（量的，移植算法；脚本 `tools/scenarios.sh`，输出 `scenarios.out` 原样）

```
scenario=baseline gpu=d17732ad426081b7 cpu_of_tree_split=fe35be42de6dccdd observer_of_tree_split=e72cadd60a623a12
scenario=unit_test_added_inside_crash_rs_last_test_module gpu=92f2b67308442b45 cpu_of_tree_split=e36518a30b6beeab observer_of_tree_split=a085d372a22929fd
scenario=comment_appended_to_crash_judge_common_wgsl gpu=0e991cd4a56a3598 cpu_of_tree_split=fe35be42de6dccdd observer_of_tree_split=e72cadd60a623a12
scenario=comment_appended_to_core_transaction_rs gpu=d17732ad426081b7 cpu_of_tree_split=fe35be42de6dccdd observer_of_tree_split=e72cadd60a623a12
scenario=unused_function_appended_to_core_transaction_rs gpu=16245fa0da19979d cpu_of_tree_split=d0d4ec9e21da59a6 observer_of_tree_split=a8c4efd47418ed01
scenario=comment_appended_to_core_cargo_toml gpu=773cdc04f48ac655 cpu_of_tree_split=883146af906a8e74 observer_of_tree_split=0d35afc45b7f5a4d
scenario=comment_appended_to_cargo_lock gpu=4fd7578e7e54a1d3 cpu_of_tree_split=24024186bb66952f observer_of_tree_split=c9d5e48e98177414
scenario=crate_directory_singlefs_format_moved_same_content gpu=669fbdb7bd2813d5 cpu_of_tree_split=5fc57b6a027e8cc3 observer_of_tree_split=7913a3671337db29
scenario=unused_function_appended_to_the_setting_file gpu=d17732ad426081b7 cpu_of_tree_split=46b36a0a40955082 observer_of_tree_split=ad24dff1a9c5878d
scenario=comment_appended_to_the_setting_file gpu=d17732ad426081b7 cpu_of_tree_split=fe35be42de6dccdd observer_of_tree_split=e72cadd60a623a12
scenario=unused_function_appended_to_crash_amplification_stop_file gpu=d17732ad426081b7 cpu_of_tree_split=fe35be42de6dccdd observer_of_tree_split=e72cadd60a623a12
scenario=unused_function_appended_to_crash_judge_dispatch_excluded_file gpu=d17732ad426081b7 cpu_of_tree_split=fe35be42de6dccdd observer_of_tree_split=e72cadd60a623a12
```

（设点文件取 `tests/crash_enumeration_tree_split_streams.rs`。）读法：给 `crash.rs` 测试模块加一条空单测、给 wgsl 加一行注释、给 core 加一个没人调的函数、给 `Cargo.toml` / `Cargo.lock` 加注释、crate 目录换名不改内容，GPU 判器那一把都换；`.rs` 只加注释不换（阳性对照：证明这条路子分得出「注释不算」与「注释算」）。往停止文件、排除文件里加函数，三把都不动（见表 C17、C20 的漏判）。

一处改动让整张表多少行换掉（量的，移植算法，`comm -13` 数新表里不同的行）：

```
unit_test_in_crash_rs: entries_changed=121 of 137
unused_fn_in_core_transaction_rs: entries_changed=134 of 137
comment_in_cargo_lock: entries_changed=137 of 137
```

### 1.4 抽查给定状态判法在不在 CPU 结论闭包（量的，移植算法）

```
setting=tests/crash_injection_campaign.rs closure_files=65 has_crash_given_states=1 has_writable_mount_phases=1 has_history=1
setting=src/bin/e156_allocation_basis_counts.rs closure_files=49 has_crash_given_states=1 has_writable_mount_phases=1 has_history=0
setting=tests/crash_points_of_the_rollback_publish.rs closure_files=50 has_crash_given_states=1 has_writable_mount_phases=1 has_history=0
setting=tests/common_given_states_of_harness_scenario_images/mod.rs closure_files=50 has_crash_given_states=1 has_writable_mount_phases=1 has_history=0
setting=src/bin/e171_three_oracle_collusion.rs closure_files=58 has_crash_given_states=1 has_writable_mount_phases=1 has_history=1
```

## 2 表：每行一处

「管哪种判定」里的三种结论 = 崩溃判定库（崩溃枚举用例库、崩溃放量库、给定状态库）里按（块键，结论种类，版本）存的 GPU 判器结论、CPU 判器结论、观察者结论。

| 编号 | 位置（文件:行号） | 这把键或标记管哪种判定 | 键由什么组成（逐项） | 违反 | 重做方向（单元怎么切、键改成什么、落点） |
|---|---|---|---|---|---|
| C1 | `crates/singlefs-checker-tier/src/judging_closure.rs:18-139`（词法）、`:856-875`（每份文件一个摘要）、`:1073-1085`（拼进摘要） | 三种结论的版本（`crash_identity.rs:647-655` 拼进版本消息） | 闭包里每份 `.rs` 整份词法单元串（去空白、注释、文档注释）的 SHA-256；`#[cfg(test)]` 模块照收（`judging_closure.rs` 里认 `cfg(test)` 的只有它自己的测试模块 `:1331`，量的） | ③（量的：core `transaction.rs` 末尾加一个没人调的函数，137 行变 134 行；`crash.rs` 测试模块加一条空单测，137 行变 121 行，§1.3） | 单元切成 fn 项（自由函数、`impl` 里的方法、`const`/`static` 初值、类型定义各算一项），每项去注释后取指纹；`#[cfg(test)]` 项整个不收。落点 `judging_closure.rs`（`build.rs` 经 `#[path]` 同一份） |
| C2 | `judging_closure.rs:747-759`（`RawBytes`）、`:866-868`、`:903-915`（`include_str!` 读的文件按原文） | GPU 判器结论版本（五份 wgsl 内核都经 `crash_judge_gpu.rs:73-81` 的 `include_str!` 进闭包） | 五份 `.wgsl` 各取原文字节 SHA-256 | ②（量的：`crash_judge_common.wgsl` 末尾加一行注释，GPU 那一把换，§1.3）；③（整份内核文件一把） | wgsl 按 `fn` 切单元、去 `//` 注释取指纹；内核按入口 `@compute fn` 的调用图收单元；Rust 侧生成的常量串（`kernel_prelude_wgsl_constants` 这类）当它的生成函数单元算。落点 `judging_closure.rs`，加一份 wgsl 切分 |
| C3 | `judging_closure.rs:1067-1072`（`entry <路径>`、`stop <路径>` 行）、`:1080-1084`（`file <路径> <raw\|lexical> <摘要>`） | 三种结论版本 | 每份入口、停止文件、闭包文件的仓内路径逐字进摘要 | ⑤（量的：`crates/singlefs-format` 整个目录换名、内容不变，三把都换，§1.3） | 键里只放单元指纹的多重集（按指纹排序），路径、文件名不进键；要给人看「哪几份代码」另写说明表。落点 `judging_closure.rs::conclusion_code` |
| C4 | `judging_closure.rs:993-1005`（`manifests_of`）、`:1086-1092` | 三种结论版本 | 仓根 `Cargo.lock` 整份原文 SHA-256；闭包碰到的每个 crate 的 `Cargo.toml` 整份原文 SHA-256 | ②③（量的：`Cargo.lock` 末尾加一行注释，137/137 行全换；core `Cargo.toml` 加注释三把都换，§1.3） | 依赖版本从 `Cargo.lock` 解析：只取闭包里单元真引到的外部包（名、版本、来源、checksum）及其传递依赖；`Cargo.toml` 只取解析后影响编译的字段（启用的 features、edition、`[profile]`），不取原文。落点 `judging_closure.rs::manifests_of` |
| C5 | `judging_closure.rs:877-990`（`closure_members`：顺 `use` 与路径按文件收、`:936-978` 同 crate 里 `impl` 头点名已收类型的文件整份收）、`:417-418`、`:434`、`:439`（解析不到落回 crate 根 `lib.rs`） | 三种结论版本 | 闭包单位是文件：GPU 判器闭包 53 份，core 32 份，含 `transaction.rs`、`make_filesystem.rs`、`mount.rs`、`unlink.rs`、`admission.rs`（量的，§1.2） | ③ | 闭包按调用图收函数单元：从判器入口 fn 出发，经名字解析到被调的 fn、方法、const、类型定义，不按文件、不按 `impl` 头整份带。落点 `judging_closure.rs` |
| C6 | `judging_closure.rs:1137-1145`（`entry_files`）；`crash_state_enumeration.rs:1000`、`:1009`（观察者代码文件 = 用例文件）；`crash_amplification.rs:1630-1636`（设点文件从身份串切出）、`:12962-12976`（CPU 版本按设点文件各一把） | CPU 结论与观察者结论版本 | CPU = `crash.rs` 闭包 ∪ 整份设点文件出发的闭包；观察者 = 整份观察者代码文件出发的闭包 | ③⑤（量的：设点文件加一个没人调的函数，它的 CPU 与观察者两把都换，§1.3；版本按设点文件路径选） | 入口改成这一项真执行的那个测试函数（崩溃点闭包、观察者闭包）这个 fn 单元，而不是整份文件；选版本不按路径。落点 `judging_closure.rs::ConclusionCodeEntry`、`crash_amplification.rs::ConclusionVersionsOfThePlan`、`crash_state_enumeration.rs` 观察者那两行 |
| C7 | `judging_closure.rs:1181-1188`（候选目录 `tests/`、`src/bin/` 与 `crash_injection.rs`）、`:1206-1237`、`:1244-1262`（`of_the_tree` 每份候选文件各算两行） | 编译时摘要表（三种结论版本的来源） | 候选按目录列：`tests/` 顶层每份 `.rs`、每个子目录的 `mod.rs`、`src/bin/` 每份 | ③（目录级：表的行由目录里有哪些文件定，不由这一趟要判的项定） | 表的行改成「登记过要判的项 → 它的入口 fn 单元」；编译时只算这一趟二进制里会用的那几项。落点 `judging_closure.rs`、`build.rs` |
| C8 | `crash_identity.rs:771-783`（`conclusion_code_entries_differing_from_the_build`）；调用 `crash_amplification.rs:12951-12958`、`:11861-11875`（给定状态引擎） | 整趟写不写库（三种结论） | 整张编译时表与盘上现算整张表逐行比，任何一行不等整趟不写库、退 78 | ③⑥（整表级：`tests/` 或 `src/bin/` 下任何一份与这一项无关的文件编完又改了，所有项整趟不写，推的；比较的整表性是量的） | 删整表比，只比这一趟要用的那几把（`conclusion_version_checked_against_the_build` 已按把比）；版本换成单元指纹后，比的是这一项用到的单元指纹。落点 `crash_identity.rs`、`crash_amplification.rs` 两处调用 |
| C9 | `crash_identity.rs:604`（`"singlefs conclusion version 1"`）、`judging_closure.rs:783`（`"singlefs conclusion code digest 1"`）、`:1176`（表头 `… table 1`）、`crash_amplification.rs:12095`（`given-states-input-v3`）、`:12136`（`given-states-plan-v1`）、`:14387`（`observed image deduplication key 1`） | 三种结论版本、给定状态块键、观察者去重键的写法 | 手写的写法号进键 | ④（写法号；改算键代码靠人记得抬） | 键的写法由算键那几个函数单元的指纹代替手写号。落点同各行 |
| C10 | `crash_identity.rs:572-578`（`version_text`：`observer:<名字>`）、`:647-655`（版本消息 `kind=`） | 观察者结论版本 | 种类串里带观察者名字 | ⑤（观察者改名换版本，量的读码） | 观察者按它的代码单元指纹区分，不按名字。落点 `crash_identity.rs::ConclusionKind`、`verdict_store.rs:629-643` |
| C11 | `crash_identity.rs:71-78`（`identity_text`）、`crash_amplification.rs:613-615`（拼身份）、`:399-406`（`block_key_under` 把身份当节点名放进块键） | 崩溃放量与崩溃枚举用例库的块键（三种结论都挂在块键下） | 节点名 = `<设点文件仓内路径>::<崩溃点名>::<复用键十六进制>` | ⑤（测试文件改名、崩溃点或项改名、流程一字不变，这些块全部换键：量的读码） | 块键只由复用键（写表、起步镜像）、计划哈希、块起点定；路径与名字只进路径索引表（`verdict_store.rs` 0x06）与覆盖报告。落点 `crash_amplification.rs::block_key_under`、`verdict_store.rs::VerdictBlockKey` |
| C12 | `crash_amplification.rs:822-824`（末节点身份） | 同 C11 | 末节点身份用最后一个点的设点文件路径加 `end_of_flow` | ⑤（两条流全部持久的那个状态一样，最后一个点的设点文件不同就不共享，推的） | 同 C11 |
| C13 | `crash_amplification.rs:353-366`（块起点 = 点内序号）、`:643-722`（计划文本：每段段序、态数、展开名，往下游找的发布，点的 `states=`，`judge_inputs=`）、`:733` | 同 C11 | 一把计划哈希罩整点全部段；块起点按点内序号 | ③（点里一段的展开方式或态数变，整点的块全换键；前一段态数变后面的块起点全挪，量的读码，挪位是推的） | 块按段切：块键 = 这一段的写表与起步镜像摘要、段展开、这一段判器另读的输入、段内序号；点级 `states=` 去掉。落点 `crash_amplification.rs::plan_crash_points_with_slices`、`PlannedCrashPoint::block_key` |
| C14 | `crash_state_enumeration.rs:745-754`（一条崩溃枚举用例整条流当一个崩溃点：`name: item.name`、`code_file_in_repository: item.setting_file`、`writes: 0..writes.len()`） | 崩溃枚举用例库的块键 | 复用键 = 整条流的写表 + 基镜像；节点名带用例文件路径与项名 | ③⑤（流里任何一次写变，整条流所有块换键，前缀状态也不复用，推的；名字进键量的） | 按步或按段切成前缀树节点（与放量同一套），块键不带项名与文件路径。落点 `crash_state_enumeration.rs::FlowToRecord` |
| C15 | `crash_amplification.rs:12081-12145`（`block_key_of_the_table`）；串的来源 `tests/crash_injection_campaign.rs:81-91`（`{:?}` 打 draw、weights、execution）、`tests/crash_points_of_the_rollback_publish.rs:150`（`prefixes=<个数>`）、`tests/common_given_states_of_harness_scenario_images/mod.rs:357`（`scenario=<名>`） | 给定状态库的块键 | 输入串 = `item=` 项名、`seeds_and_scale=` 种子与规模串、`table=` 表名、基镜像摘要、旋钮值、整张表的每次写、接缝、每个状态的持久集合与自称版本；节点名 = `<设点文件>::<项名>::<表名>`；计划 = 态数、写数、接缝数；块起点恒 0（一张表一块） | ③⑤（加一个种子、改一个枚举变体名、多一个前缀，内容没变的表全换键；一张表加一个状态整表换键，量的读码） | 块键只由表内容定（基镜像、写、接缝、持久集合、自称版本、旋钮值），按状态或按段切块；项名、种子串、表名、文件路径进说明表。落点 `crash_amplification.rs::GivenStatesEngine::block_key_of_the_table` |
| C16 | `crash_amplification.rs:12942-12985`（`of_the_plan`：GPU 一把对全部点 `:12979`）；比较 `verdict_store.rs:3119-3137`（整把键逐字节全等） | 三种结论能不能沿用 | （块键，种类，版本）全等；版本见 C1–C6 | ③（GPU 闭包 53 份里任何一份的词法变，全部点的 GPU 结论作废；对账表第 1 条） | 每块记下判它时用到的判器单元指纹集合；沿用 = 那个集合里的每个单元此刻指纹不变（或按单元执行集合）。落点 `crash_identity.rs`、`verdict_store.rs`、`crash_amplification.rs` |
| C17 | `crash_amplification.rs:14060`（`OBSERVER_CONCLUSION_COMPOSITION_VERSION = 2`）、`:14099-14106`；拼块代码在 `crash_amplification.rs`，它是停止文件（`judging_closure.rs:691-694`） | 观察者结论版本 | 观察者版本 ‖ 池级 checker 版本 ‖ 手写号 | ④⑦（量的：往 `crash_amplification.rs` 加函数三把都不动，§1.3；拼法改了忘抬号，旧结论照沿用，推的） | 拼块的那几个函数当单元并进观察者结论版本；删手写号。落点 `crash_amplification.rs::observer_versions_to_write_under` 与 `judging_closure.rs` 停止表 |
| C18 | `crash_amplification.rs:14085-14097` | 观察者结论里池级 checker 那一把 | GPU：整个 GPU 判器版本；CPU：`crash.rs` ∪ 观察者代码文件出发的闭包（`setting_file: &observer.observer_code_file`，`:14094`） | ③（观察者文件里改一条别的用例，池级 checker 那一把换，推的） | 只取池级 checker 入口函数单元的闭包。落点同上 |
| C19 | `crash_amplification.rs:14382-14394`（`observed_image_deduplication_key`） | 观察者看过的镜像（表 0x16）与镜像上池级 checker 结论（表 0x18）能否复用 | 观察者名 ‖ 观察者代码版本 ‖ 另读输入摘要 ‖ 镜像摘要 | ⑤（名字）；版本那一格带 C6 的 ③ | 去名字，版本换观察者 fn 单元指纹。落点同上 |
| C20 | `crash_judge_gpu.rs:502-504`（`GPU_JUDGE_REVISION = 1`）；`judging_closure.rs:678-687`（`crash_judge_dispatch.rs`、`gpu_unit_checks.rs` 不许进判器闭包）；`crash_judge_dispatch.rs:5`（文件头：「这一份修了会改结论的毛病时，抬 `GPU_JUDGE_REVISION`」） | GPU 判器结论版本 | 派活一侧（上卡、一次派几个状态、读回结论字并按序号归到状态）不进键，靠人抬手写号；打包本身 `pack_tables` 在 `crash_judge_gpu.rs:700`，在闭包里 | ④⑦（量的：往 `crash_judge_dispatch.rs` 加函数三把都不动，§1.3；派活里读回、归位的代码改了会改结论而版本不动，推的） | 派活里决定「哪个状态拿到哪个结论字」的函数（读回、按序号归位、草稿区切分）当单元并进 GPU 版本；只改快慢的不进；删 `GPU_JUDGE_REVISION`。落点 `judging_closure.rs`、`crash_judge_gpu.rs`、`crash_judge_dispatch.rs` |
| C21 | `crash_judge_gpu.rs:531-548`（`gpu_judge_digest`：`"gpu-crash-judge-v7"` 前缀 + 生成常量 + 五份内核原文） | 文件头 `crash_judge_gpu.rs:21` 说它「进判定表的键」；产品路径不用（`grep -rn gpu_judge_digest` 只在 `tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:2714` 断言非零与文档里，量的） | 手写号 + 内核原文 | ②④（死键，文件头与代码不一致） | 删这个函数或改成单元指纹；文件头改成现状。落点 `crash_judge_gpu.rs` |
| C22 | `crash_judge_tables.rs:54`（`JUDGE_TABLES_VERSION = 9`）、`:1703-1708`（`digest`） | 文件头 `crash_judge_tables.rs:11` 说表进 KV 时键带它；库里判表实际按打包字节的 SHA-256 存（`crash_library.rs:846-850`），这把 `digest` 只在 `tests/crash_judge_tables_versions_reproduce_every_crash_image.rs:247` 断言非零（量的） | 手写号 + 表字 | ④（死键，文件头不一致） | 删手写号与这把摘要；判表按内容寻址已够。落点 `crash_judge_tables.rs` |
| C23 | `crash_facts.rs:35`（`FACTS_EXTRACTOR_VERSION = 2`）、`:636-642`（`FlowFacts::digest`） | 事实表在库里的键（表 0x07），清单按它指 | 手写抽取器号 ‖ 事实字节 | ④（事实表已按内容寻址，手写号只多换键；抽取代码改了靠人抬） | 去手写号；若要表达「哪一版抽取规则」，用抽取入口函数单元指纹。落点 `crash_facts.rs` |
| C24 | `crash_amplification.rs:3934-3941`（`FactsVerifierOn::verifier_digest` 的 CPU 那一支） | 第 ② 段核对结论（核对表 0x08）能否复用 | `"cpu-facts-verifier-v2"` ‖ `include_str!("crash_facts.rs")` 整份原文 | ②③④⑦（改 `crash_facts.rs` 一行注释就换键；核对调的 `singlefs_checker::check_unit`、`check_root_slot`、`crc32_castagnoli_table`（`crash_facts.rs:20`）不进键，改了照复用，推的） | 键 = `verify_state_from_facts` 起的调用图里每个函数单元的指纹（跨 crate）+ 依赖版本。落点 `crash_amplification.rs::verifier_digest`、`crash_facts.rs` |
| C25 | `crash_verify_gpu.rs:361-367`（`gpu_verifier_digest`） | GPU 核对结论能否复用 | `"gpu-facts-verifier-v4"` ‖ 整份着色器原文（含 Rust 生成的常量串） | ②④⑦（着色器注释也算；宿主侧打包事实表、上卡、读回的 Rust 代码（`GpuFactsVerifier`）不进键，推的漏判） | 着色器按 fn 去注释切单元；宿主侧决定输入布局与读回的函数单元并进。落点 `crash_verify_gpu.rs` |
| C26 | `crash_amplification.rs:4043-4050`（`read_verifier_block(&key, &verifier_digest)` 只看 `is_some()`） | 核对表复用 | （块键，核对代码摘要）在不在；行里存的事实表摘要不比 | ⑦（事实抽取代码变了、事实表换了一份，GPU 核对行照复用：比较缺这一格是量的，后果推的） | 复用条件加「存着的事实表摘要 = 这一趟读回的」或把抽取函数单元指纹并进核对键。落点 `crash_amplification.rs::verify_blocks_from_these_facts` |
| C27 | `verdict_store.rs:152-171`（`VerdictBlockKey` 与 `to_block_key_bytes`）、`:181-190`（接种类与版本）、`:629-643`（观察者种类带名字） | 库里所有按块键的表（判定块、结论块、违例、恢复字、计数向量、块属性、观察者录入块、核对表） | 输入指纹 ‖ 名字长度 ‖ 流名或节点名 ‖ 计划哈希 ‖ 块起点；结论键另接种类（观察者带名字）‖ 版本 | ⑤ | 键结构去掉名字字段；种类只区分判器，观察者按代码单元指纹。落点 `verdict_store.rs` |
| C28 | `verdict_store.rs:16-24`（判定块、违例按「块键 ‖ 判法版本」存，表 0x01、0x04、0x09）；`crash_amplification.rs:1134-1188`（`PipelineJudgingCode`）、`:1644-1647`（`TheJudgeVersion`：「直接调流水线各段的对拍用例与单测用它；`run_from_environment` 不用」）；版本算法 `crash_identity.rs:482-496` → `judging_closure.rs:1012-1051` | 流水线单测与对拍用例的判定（`tests/crash_amplification_pipeline_records_checks_and_reuses.rs:159-160` 用它） | 闭包每份文件「仓内路径一行 + 整份词法串」（不是摘要，整串）、`include_str!` 文件原文摘要、`Cargo.lock` 与各 `Cargo.toml` 原文摘要、工具链；两个判器合一把 | ②③⑤⑧（同一个库里两套键并存，同一块按两套各判一次） | 删「判法版本」那一套与它的表，单测与对拍改走单元版本。落点 `verdict_store.rs`、`crash_amplification.rs`、`crash_identity.rs`、`judging_closure.rs::judging_code_stopping_at` |
| C29 | `verdict_store.rs:917-920`（录入清单表 0x11 键是流名）、`crash_library.rs:703-722`（`record_the_item` 按项名登清单，同名下一趟盖掉）、`verdict_store.rs:240-264`（分片进度表 0x14 键 = 流名 ‖ 段号 ‖ 判器） | 录入段到判段的交接、分片续跑进度（不是判定本身） | 名字当键 | ⑤（改名只丢交接与进度，判定块不丢，推的） | 清单按录入内容摘要寻址，项名只作索引行；分片进度键按段内容摘要（段写表与起步镜像）。落点 `verdict_store.rs`、`crash_library.rs` |
| C30 | `crash_library.rs:767-786`（`two_stages`：`stage.records()` 就跑录入）；`crash_amplification.rs:10988-11080`（`run_from_environment_with_observers` 录入分支） | 录入段（跑流、排计划、抽事实、建 GPU 判表、写库） | 没有键：每趟每项照跑 | ⑥（对账表第 11 条；量的读码，代价推的） | 录入段输出按「录入函数单元指纹 + 流输入摘要」入库，库里已有就跳过录入。落点 `crash_library.rs::two_stages`、`crash_amplification.rs` |
| C31 | `crash_library.rs:110-138`（六类各一份库、目录名写死）、`:156-161`（三类认 `SINGLEFS_CRASH_AMPLIFICATION_LIBRARY`，入口脚本按趟交）、`:229-245` | 判定库放哪、几份 | 类 → 子目录名常量；入口脚本可按趟换目录 | ⑧（崩溃枚举用例库按整流一点、节点名带项名与用例文件，放量库按每步一点、节点名带 `file!()`，同一批状态两库各判、互不共享：键不同是量的读码，重复判的规模推的；目录名改一次整库丢） | 合成一份按单元键的库，类只当索引；库目录固定一处，不随趟换、不靠改目录名表达格式变化。落点 `crash_library.rs`（入口脚本那一半归脚本路子） |
| C32 | `crash_library.rs:186-197`（`amplification_unit_tests_library_directory_emptied_at_the_start`） | 流水线单测（钉「空库起跑第一趟全判、第二趟全复用」的那几条） | 起跑清空这一份库 | ⑥（这几条单测每趟从零判） | 要设计判断（它测的就是空库起跑）：方向是空库只建在一次性副本里，单测结论本身按单测 fn 单元指纹记。落点 `crash_library.rs` 与调用它的测试 |
| C33 | `crash_library.rs:866-895`（事实表对拍）、`:908-938`（判表对拍） | GPU 与 CPU 对拍用例的结论 | 库里只存输入（事实表、判表），对拍结论不存 | ⑥（每趟两边都重判） | 对拍结论按（输入摘要、两边判器单元指纹）入库，命中就沿用。落点 `crash_library.rs` 与对拍测试 |
| C34 | `src/bin/e170_o3_spec_executor_export.rs:35-44`、`src/bin/e171_three_oracle_collusion.rs:33-42`、`src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:38-46`（`sampling_item_name_of_this_call`：项名 = 装置名 + `-` + 进程内 `fetch_add` 计数） | 这三份装置取样进崩溃枚举用例库的块 | 项名就是崩溃点名（`crash_state_enumeration.rs:749`），进节点名（C11） | ⑤（取样调用的先后一变、多一次或少一次调用，后面全部换键；文件头 `e170…:34`、`e171…:32`、`e171…after…:37` 说「块键按内容算，下一趟照样复用判过的块」，与代码不符：量的读码） | 同 C11、C14（块键不带项名）；项名只作清单索引。落点三份装置与 `crash_state_enumeration.rs` |
| C35 | `src/bin/e156_allocation_basis_counts.rs:703-706`、`src/bin/e168_delete_ring_window_borrow.rs:1266-1269`、`src/bin/e171_three_oracle_collusion.rs:1866-1868`、`src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:7038-7040`、`:10001-10005`、`src/bin/e158_root_choice_repair.rs:22531-22533`、`src/crash_injection.rs:5576-5579` | 这些装置与单测进给定状态库的块 | `GivenStatesItem` 的项名字面量、设点文件路径字面量、`seeds_and_scale`（多为 `"fixed"`）进块键与节点名（C15） | ⑤ | 同 C15。落点各装置与 `crash_amplification.rs` |
| C36 | `tests/common_crash_points/mod.rs:78`、`:226`、`:231`、`:245`、`:266`、`:281`、`:302`、`:326`（`recorder.step(file!(), "<名>", …)`）；设点文件路径常量（例 `tests/crash_enumeration_tree_split_streams.rs:45-46`，全仓 50 份文件带这一类写法：`grep -ln 'setting_file\|SETTING_FILE\|repository_relative_file(file!())\|file!()'` 计数） | 放量与崩溃枚举用例的节点身份、CPU 版本选择 | `file!()` 与名字字面量 | ⑤（经 C11、C6） | 同 C11、C6；测试侧不再给路径常量。落点各测试文件随引擎一起改 |
| C37 | `crash_identity.rs:22`、`crash_point_replay_progress.rs:22-70`（工具链身份：`rustc -Vv` 前三行、`cargo -V`、目标三元组） | 三种结论版本里的工具链 | 工具链三样 | 无（合规）；发现日志按签名去重不是复用键 | 不改 |
| C38 | `crash_given_states.rs`（整份） | 没有键 | 它是给定状态的 CPU 判法，却不是 CPU 判器入口（`judging_closure.rs:654-657` 入口只有 `crash.rs`、`crash_judge_gpu.rs`），进 CPU 版本靠设点文件引到；§1.4 抽查 5 份都引到 | ⑦（结构上的漏判口子，推的；抽查没找到实例） | 判器入口按「判给定状态表的那个 fn」登记成单元入口。落点 `judging_closure.rs` |
| C39 | `crash_judge_conclusions.rs`、`crash.rs`、五份 `.wgsl` | 没有自己的键 | 它们随 C1/C2 按整文件进闭包 | 见 C1、C2 | 见 C1、C2 |
| C40 | `gpu_unit_checks.rs:634-644`（驱动着色器盘缓存目录：`GATE_CROSS_RUN_TMPDIR` → `~/.cache/singlefs` → `TMPDIR` → `/tmp`）、`:700-716`（进程内按着色器源码摘要缓存编好的管线）；`crash_judge_dispatch.rs:243-254`（`kernel_sources_digest`） | 编译缓存，不是判定 | 着色器原文 | 不在 ①–⑧ 射程（不判对错）；缓存可能落在 `/tmp`（推的，看 `GATE_CROSS_RUN_TMPDIR` 怎么设） | 不改；见「射程外」 |

## 3 文件头、文档与代码不一致（随对应行一起改）

| 位置 | 文字怎么说（原文摘到够认） | 代码怎么做（量的） | 随哪一行改 |
|---|---|---|---|
| `crash_identity.rs:6-8` | 「复用键罩三样属性：……写表；……父输出；判法代码……连同 `Cargo.lock` 与工具链」 | `StepAttributes::reuse_key` 只拼写表与父输出（`crash_identity.rs:103-115`） | C1、C11 |
| `crash_identity.rs:4-5` | 「前两段是标签，只给覆盖报告；哈希串是复用键」 | 整个身份串进块键节点名（`crash_amplification.rs:399-406`） | C11 |
| `crash_identity.rs:12`、`:588` | 「停止表（判器的 ∪ 流水线五份）」 | 流水线停止表七份（`judging_closure.rs:691-720`） | C5 |
| `crash_amplification.rs:272` | 「块键里的节点名只有它，路径不进块键（改名不使判定失效）」 | 身份带设点文件路径与崩溃点名，改它们就换块键 | C11 |
| `crash_amplification.rs:1132` | 「判法改了每个点的复用键都变、树不动」 | 判法不进复用键，进的是版本；这一套产品路径不用（`:1644-1647`） | C28 |
| `crash_amplification.rs:11617` | 「种子与规模进块键，闭包进版本」 | 与代码一致，但与新判据矛盾 | C15 |
| `verdict_store.rs:88` | 输入指纹是「门禁 checker-tier-crash-point-replay 给每条崩溃枚举用例算的输入指纹（`scripts/gate/admission.py`）」 | 是这一步的复用键或给定状态输入串摘要（`crash_amplification.rs:399-406`、`:12131`） | C27 |
| `verdict_store.rs:16-24`、`:99-102` | 判定块按「块键 ‖ 判法版本」存、「复用只认同一判法版本」 | 产品路径按（块键，种类，版本）存（0x0A 起那几张表） | C28 |
| `crash_state_enumeration.rs:8` | 「改了用例文件只换它那把 CPU 版本」 | 观察者代码文件也是用例文件（`:1000`、`:1009`），观察者版本一起换；用例文件改名还换块键 | C6、C14 |
| `crash_judge_gpu.rs:21` | `gpu_judge_digest` 「进判定表的键」 | 产品路径不用 | C21 |
| `crash_judge_tables.rs:11` | 表进 KV 时「键带 `JUDGE_TABLES_VERSION` 与表的摘要」 | 按打包字节 SHA-256 存 | C22 |
| `crash_facts.rs:13-16` | 事实表键带「产出它的代码摘要（`FlowFacts::digest` 连同 `FACTS_EXTRACTOR_VERSION`）」 | 带的是手写号，不是代码摘要 | C23 |
| `judging_closure.rs:681`、`crash_judge_dispatch.rs:5` | 「改结论的毛病修在这里时抬 `GPU_JUDGE_REVISION`」 | 手写号（与新判据矛盾） | C20 |
| `e170…:34`、`e171…:32`、`e171…after…:37` | 「块键按内容算，下一趟照样复用判过的块」 | 项名带进程内计数，进节点名 | C34 |

## 4 没查到 / 判不了

- GPU 与 CPU 结论互不借（种类进键 `verdict_store.rs:629-633`；`crash_amplification.rs:11861` 起只认这一趟 `judged_by` 那一种）：两种判器的函数单元本来不同，按新判据键不同是正当的；要不要互借得先证两种结论逐位等价，判不了，交设计（对账表第 9 条后半）。
- 驱动版本、卡型只是块属性、不进版本（`verdict_store.rs:562-570`，J8、M2-5 定的）：判据只列了函数单元、工具链、依赖版本，驱动不在里面；不进算不算 ⑦，判不了。
- 判器另读的流程输入 `judge_inputs`（`crash_amplification.rs:965-1126`：这个点之前全部写逐行链起来、根槽写 txg 不严格递增时之后全部写也进）：这是判器真读的数据，不是代码，不在「函数单元」判据里；它让前面任何一次写变都换掉后面所有点的计划哈希，算不算过宽，判不了。
- `include_str!(concat!(…))` 不进闭包（`judging_closure.rs:498-505` 只认字面量串）：现有两处（`crash_identity.rs:661`、`crates/singlefs-harness/src/heavy_threads.rs:22`）都不判对错，没有实例；判器以后这样读文件会漏，推的 ⑦。
- 「按调用图收函数单元」做到多细（trait 动态分派、宏展开、泛型、`impl` 块里的关联常量）：这是重做方向的设计题，不在这一轮。
- `crash_amplification.rs` 17182 行没逐行读完：读了算块键、计划、版本、观察者、录入、核对、给定状态几段；全文件另有没有键，靠 `grep -n 'sha256_digest\|fn .*digest\|fn .*key\|fn .*version\|_HEADER\|_VERSION: \|MAGIC'` 那一次的全部命中逐个看过（字节格式的 `*_MAGIC` 是序列化头，不是键），推的全集。
- C32 那几条单测要不要剪枝（它测的就是空库起跑），判不了。

## 5 射程外（撞见了，不归这一路改）

- `crates/singlefs-checker-tier/build.rs:45-79`：`rerun-if-changed` 盯每个 crate 的 `src/` 与 `Cargo.toml`、`tests/`、`src/bin/`，管的是重编，不是判定。
- `gpu_unit_checks.rs:634-644` 的驱动着色器盘缓存落点可能是 `/tmp`（`GATE_CROSS_RUN_TMPDIR` 由门禁设），编译缓存，不判对错。
- `crates/mutations.tsv`：crates 变异的行键与判法版本在 `scripts/mutation/crates-mutation-rows.py`（脚本路子，对账表第 15、16 条）。
- `src/bin/e165_replay_bounded_enumeration.rs:2821-2823` 的输入指纹只给钉表行自核；钉表重放的复用键在 `scripts/experiments/e165-pin-red-examples.py`（脚本路子，对账表第 18 条）。
- `crates/singlefs-replay-prefix/src/lib.rs`（Verus 核 crate）没有键；形式化的键在 `scripts/gate/verus-proof-check.py`。
- 崩溃枚举用例的外层全绿标记、快档、整道键在 `scripts/gate/admission.py`（对账表第 12–14 条），不在 crates 里。

## 6 推翻条件（什么现象会推翻）

- C1：在仓副本里只往 `crash.rs` 末尾那个 `#[cfg(test)]` 模块加一条空单测，`cargo build -p singlefs-checker-tier` 之后 `OUT_DIR/conclusion_code_digests.tsv` 的 `gpu_judge` 行不变，就推翻（移植算法说会变：§1.3 第 2 行；Rust 实编没做，cargo 不归这一路）。
- C2、C3、C4：同法只给一份 wgsl 加注释 / 只把 `crates/singlefs-format` 目录换名并改 workspace 成员 / 只给 `Cargo.lock` 加一行注释，表的 `gpu_judge` 行不变，就推翻。
- C5：Rust 那条列闭包的用例（`crash_identity.rs` 测试模块里列判器闭包的那几条）打出的 GPU 闭包不含 `crates/singlefs-core/src/transaction.rs`，就推翻。
- C8：编完只改一份无关的 `tests/*.rs` 再跑一项，不打 `CRASH_POINT_REPLAY_NOT_WRITTEN_CONCLUSION_VERSION_DIFFERS_FROM_THE_BUILD reason=table_differs`，就推翻。
- C11、C14、C34：只给一个崩溃点或项改名、流程一字不变，同一条命令的行里旧块报复用而不是现判，就推翻。
- C26：在已有核对行的库上改事实抽取让事实字节变，GPU 核对那一趟 `blocks_already_verified` 是 0（全部重核），就推翻。
- C17、C20 漏判那一半：派活里读回归位的代码、观察者拼块代码改了而结论逐位不变的例子只能证「这一次没漏」，推不翻；推翻要找到「这类改动从结构上不可能改结论」的论证。
- 移植算法本身：任何一行与编译表不等就推翻 §1.3、§1.4 全部；实测 137/137 相等（§1.1）。

## 7 排除掉的解释与凭的观测

- 「`.rs` 只改注释也换版本」：排除。§1.3 `comment_appended_to_core_transaction_rs`、`comment_appended_to_the_setting_file` 两行三把都不动。② 只记 wgsl 与清单原文。
- 「打包判表 `pack_tables` 在派活文件里、不进 GPU 版本」：排除。`grep -rn 'fn pack_tables'` 落在 `crash_judge_gpu.rs:700`，在 GPU 闭包里；C20 只留派活读回归位那一半（推的）。
- 「给定状态判法不在 CPU 结论闭包」：抽查 5 份设点文件都在（§1.4），排除实例，只留 C38 结构风险。
- 「GPU 判器闭包只有判器」：排除。§1.2 53 份，core 32 份含写路径。
- 「判表摘要 `JUDGE_TABLES_VERSION`、`gpu_judge_digest` 是产品键」：排除。全仓 grep 只在测试断言与文档里用（C21、C22）。

## 8 附：grep 列出的每个文件的去向（292 份）

命令：`grep -rln "sha256\|digest\|fingerprint\|version\|manifest\|closure" crates/ --include=*.rs --include=*.wgsl --include=build.rs`，输出 292 行（`wc -l`）。分桶：表里有行的按行号；`library-users.txt`（52 份，用库或设点）；只命中领域词 version 的（`version-only.txt`，172 份，其中落在表里有行的另按行号）；其余按读到的用法。`??` 计数 0。

| 文件 | 去向 |
|---|---|
| `crates/singlefs-checker-tier/build.rs` | 射程外（编译触发 rerun-if-changed；算法同 C1–C7） |
| `crates/singlefs-checker-tier/tests/crash_enumeration_multi_record_publish_stream.rs` | C36（设点文件或用库的用例：贡献路径、项名、种子串，键由引擎算，见 C11、C14、C15） |
| `crates/singlefs-checker-tier/tests/gpu_judge_tree_identifier_watermark_of_the_newest_root_like_the_cpu_checker.rs` | C36（设点文件或用库的用例：贡献路径、项名、种子串，键由引擎算，见 C11、C14、C15） |
| `crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs` | C36（设点文件或用库的用例：贡献路径、项名、种子串，键由引擎算，见 C11、C14、C15） |
| `crates/singlefs-checker-tier/tests/crash_enumeration_record_spill_over_stream.rs` | C36（设点文件或用库的用例：贡献路径、项名、种子串，键由引擎算，见 C11、C14、C15） |
| `crates/singlefs-checker-tier/tests/crash_injection_publish_after_a_mount_whose_deletion_continuation_stays_short.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-checker-tier/tests/record_checker_reuse_exemption_legality.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-checker-tier/tests/bad_disk_input_campaign.rs` | 没有复用键（测试里的摘要、指纹只作断言比较或观察者另读输入） |
| `crates/singlefs-checker-tier/tests/crash_amplification_two_flows_share_their_prefix.rs` | C36（设点文件或用库的用例：贡献路径、项名、种子串，键由引擎算，见 C11、C14、C15） |
| `crates/singlefs-checker-tier/tests/crash_enumeration_writable_mount_of_a_formatted_pool.rs` | C36（设点文件或用库的用例：贡献路径、项名、种子串，键由引擎算，见 C11、C14、C15） |
| `crates/singlefs-checker-tier/tests/crash_segments_per_device_and_torn_in_place_overwrites.rs` | C36（设点文件或用库的用例：贡献路径、项名、种子串，键由引擎算，见 C11、C14、C15） |
| `crates/singlefs-checker-tier/tests/crash_amplification_fixed_script_through_rollback.rs` | C36（设点文件或用库的用例：贡献路径、项名、种子串，键由引擎算，见 C11、C14、C15） |
| `crates/singlefs-checker-tier/tests/crash_injection_writable_mount_after_the_crash.rs` | C36（设点文件或用库的用例：贡献路径、项名、种子串，键由引擎算，见 C11、C14、C15） |
| `crates/singlefs-checker-tier/tests/crash_points_of_the_rollback_publish.rs` | C36（设点文件或用库的用例：贡献路径、项名、种子串，键由引擎算，见 C11、C14、C15） |
| `crates/singlefs-checker-tier/tests/crash_enumeration_tree_split_streams.rs` | C36（设点文件或用库的用例：贡献路径、项名、种子串，键由引擎算，见 C11、C14、C15） |
| `crates/singlefs-checker-tier/tests/crash_amplification_planned_import_names_the_gpu_conclusions.rs` | C36（设点文件或用库的用例：贡献路径、项名、种子串，键由引擎算，见 C11、C14、C15） |
| `crates/singlefs-checker-tier/tests/crash_enumeration_writable_mount_of_an_io_min_8192_pool.rs` | C36（设点文件或用库的用例：贡献路径、项名、种子串，键由引擎算，见 C11、C14、C15） |
| `crates/singlefs-checker-tier/tests/gpu_judge_root_choice_tie_takes_the_earlier_root_like_the_cpu_checker.rs` | C36（设点文件或用库的用例：贡献路径、项名、种子串，键由引擎算，见 C11、C14、C15） |
| `crates/singlefs-replay-prefix/src/lib.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-checker-tier/tests/gpu_recovery_root_choice_refuses_root_record_pointer_head_fields_like_the_core.rs` | C36（设点文件或用库的用例：贡献路径、项名、种子串，键由引擎算，见 C11、C14、C15） |
| `crates/singlefs-checker-tier/tests/rollback_floor_written_into_the_system_configuration_first_and_normal_unmount.rs` | C36（设点文件或用库的用例：贡献路径、项名、种子串，键由引擎算，见 C11、C14、C15） |
| `crates/singlefs-checker-tier/tests/crash_enumeration_position_addressed_trees.rs` | C36（设点文件或用库的用例：贡献路径、项名、种子串，键由引擎算，见 C11、C14、C15） |
| `crates/singlefs-checker-tier/tests/crash_injection_record_checker_sees_every_publish_across_a_second_crash.rs` | C36（设点文件或用库的用例：贡献路径、项名、种子串，键由引擎算，见 C11、C14、C15） |
| `crates/singlefs-checker-tier/tests/crash_enumeration_findings_log_dedupes_by_signature_and_survives_resume.rs` | C36（设点文件或用库的用例：贡献路径、项名、种子串，键由引擎算，见 C11、C14、C15） |
| `crates/singlefs-checker-tier/tests/crash_points_of_the_instance_table_sweep.rs` | C36（设点文件或用库的用例：贡献路径、项名、种子串，键由引擎算，见 C11、C14、C15） |
| `crates/singlefs-checker-tier/tests/crash_amplification_product_path_books_its_scratch_on_the_card_pool.rs` | 没有复用键（测试里的摘要、指纹只作断言比较或观察者另读输入） |
| `crates/singlefs-checker-tier/tests/a_publish_returns_only_after_the_system_configuration_rotation_is_durable.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-checker-tier/tests/gpu_unit_checks_agree_with_the_cpu_and_catch_a_flipped_byte.rs` | C36（设点文件或用库的用例：贡献路径、项名、种子串，键由引擎算，见 C11、C14、C15） |
| `crates/singlefs-checker-tier/tests/crash_points_tree_identifier_watermark_orphans.rs` | C36（设点文件或用库的用例：贡献路径、项名、种子串，键由引擎算，见 C11、C14、C15） |
| `crates/singlefs-checker-tier/tests/crash_injection_read_faulted_states_never_hold_the_witness_criterion.rs` | C36（设点文件或用库的用例：贡献路径、项名、种子串，键由引擎算，见 C11、C14、C15） |
| `crates/singlefs-checker-tier/tests/crash_given_states_of_a_rollback_to_the_root_at_the_effective_floor.rs` | C36（设点文件或用库的用例：贡献路径、项名、种子串，键由引擎算，见 C11、C14、C15） |
| `crates/singlefs-checker-tier/tests/crash_injection_campaign.rs` | C36（设点文件或用库的用例：贡献路径、项名、种子串，键由引擎算，见 C11、C14、C15） |
| `crates/singlefs-checker-tier/tests/crash_amplification_fixed_script_through_unmount.rs` | C36（设点文件或用库的用例：贡献路径、项名、种子串，键由引擎算，见 C11、C14、C15） |
| `crates/singlefs-checker-tier/tests/crash_judge_gpu_tables_at_their_capacity.rs` | C36（设点文件或用库的用例：贡献路径、项名、种子串，键由引擎算，见 C11、C14、C15） |
| `crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs` | C36（设点文件或用库的用例：贡献路径、项名、种子串，键由引擎算，见 C11、C14、C15） |
| `crates/singlefs-checker-tier/tests/common_crash_points/mod.rs` | C36（设点文件或用库的用例：贡献路径、项名、种子串，键由引擎算，见 C11、C14、C15） |
| `crates/singlefs-checker-tier/tests/crash_facts_gpu_verifier_agrees_with_the_cpu_reference.rs` | C36（设点文件或用库的用例：贡献路径、项名、种子串，键由引擎算，见 C11、C14、C15） |
| `crates/singlefs-checker-tier/tests/crash_judge_tables_versions_reproduce_every_crash_image.rs` | C36（设点文件或用库的用例：贡献路径、项名、种子串，键由引擎算，见 C11、C14、C15） |
| `crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs` | C36（设点文件或用库的用例：贡献路径、项名、种子串，键由引擎算，见 C11、C14、C15） |
| `crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs` | C36（设点文件或用库的用例：贡献路径、项名、种子串，键由引擎算，见 C11、C14、C15） |
| `crates/singlefs-checker-tier/tests/common_given_states_of_harness_scenario_images/mod.rs` | C36（设点文件或用库的用例：贡献路径、项名、种子串，键由引擎算，见 C11、C14、C15） |
| `crates/singlefs-checker-tier/src/crash_library.rs` | C29–C33 |
| `crates/singlefs-checker-tier/tests/crash_enumeration_acquisition_barrier.rs` | C36（设点文件或用库的用例：贡献路径、项名、种子串，键由引擎算，见 C11、C14、C15） |
| `crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl` | C2、C39 |
| `crates/singlefs-checker-tier/src/crash_verify_gpu.rs` | C25 |
| `crates/singlefs-checker-tier/src/crash_state_enumeration.rs` | C6、C14；文件头见第 3 节 |
| `crates/singlefs-checker-tier/src/crash_point_replay_progress.rs` | C37（合规，没有违例） |
| `crates/singlefs-checker-tier/src/device_log.rs` | 没有复用键（写日志格式的 version 字段） |
| `crates/singlefs-checker-tier/src/crash_judge_tables.rs` | C22 |
| `crates/singlefs-checker-tier/src/crash_facts.rs` | C23、C24 |
| `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` | C36（设点文件或用库的用例：贡献路径、项名、种子串，键由引擎算，见 C11、C14、C15） |
| `crates/singlefs-checker-tier/src/crash_identity.rs` | C8–C11、C28、C37；文件头见第 3 节 |
| `crates/singlefs-checker-tier/src/bad_disk_input.rs` | 没有复用键（摘要行与 CRC 是被测盘内容） |
| `crates/singlefs-checker-tier/src/on_device_modes.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-checker-tier/src/gpu_unit_checks.rs` | C40（没有判定键，只有编译缓存） |
| `crates/singlefs-checker-tier/src/crash_amplification.rs` | C6、C8、C9、C11–C13、C15–C19、C24、C26、C28、C30；文件头见第 3 节 |
| `crates/singlefs-checker-tier/src/bin/deletion_write_dump.rs` | 没有复用键（产物行里打每次写的 sha256） |
| `crates/singlefs-checker-tier/tests/crash_enumeration_floor_raise_pushed_by_admission.rs` | C36（设点文件或用库的用例：贡献路径、项名、种子串，键由引擎算，见 C11、C14、C15） |
| `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` | C2、C39 |
| `crates/singlefs-checker-tier/src/crash_judge_gpu.rs` | C20、C21 |
| `crates/singlefs-checker-tier/src/crash_judge_common.wgsl` | C2、C39 |
| `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` | C36（设点文件或用库的用例：贡献路径、项名、种子串，键由引擎算，见 C11、C14、C15） |
| `crates/singlefs-core/src/inode_tree.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-checker-tier/src/crash_given_states.rs` | C38（没有键） |
| `crates/singlefs-checker-tier/src/crash.rs` | C39（没有键；CPU 判器入口，整份进 CPU 与 GPU 闭包，见 C1） |
| `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs` | C34、C35 |
| `crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl` | C2、C39 |
| `crates/singlefs-checker-tier/src/crash_injection.rs` | C35（单测进给定状态库）；也是候选设点文件（C7） |
| `crates/singlefs-checker-tier/src/bin/new_pool_file_creation_device_log_check.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump_one_device.rs` | 没有复用键（产物行里打 sha256） |
| `crates/singlefs-checker-tier/src/judging_closure.rs` | C1–C7、C9、C28 |
| `crates/singlefs-reference-filesystem/src/lib.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-checker-tier/src/crash_judge_dispatch.rs` | C20、C40 |
| `crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs` | 没有复用键（产物行里打 sha256） |
| `crates/singlefs-checker-tier/src/verdict_store.rs` | C16、C27–C29；文件头见第 3 节 |
| `crates/singlefs-core/src/lineage_tree.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs` | C14、C36（项名 e161_crates_comparison 进节点名）；装置自己的 Digest128 是实验量，不是复用键 |
| `crates/singlefs-core/src/system_configuration.rs` | 没有复用键（盘上格式的校验和、格式版本号） |
| `crates/singlefs-checker-tier/src/bin/e168_delete_ring_window_borrow.rs` | C35 |
| `crates/singlefs-core/src/instance_table_sweep.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-core/src/mounted_read.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-core/src/mount.rs` | 没有复用键（盘上格式的校验和、格式版本号） |
| `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs` | C34、C35 |
| `crates/singlefs-core/src/pointer.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-core/src/unit.rs` | 没有复用键（盘上格式的校验和、格式版本号） |
| `crates/singlefs-harness/tests/fsync_drop_and_devices_without_the_selected_version.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/unlink_in_histories_judged_by_the_model.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs` | C34 |
| `crates/singlefs-core/src/recovery.rs` | 没有复用键（盘上格式的校验和、格式版本号） |
| `crates/singlefs-harness/tests/write_entries_refuse_an_inode_being_deleted.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-checker/src/walk.rs` | 没有复用键（盘上格式的校验和、格式版本号） |
| `crates/singlefs-core/src/admission.rs` | 没有复用键（盘上格式的校验和、格式版本号） |
| `crates/singlefs-checker/src/image.rs` | 没有复用键（盘上格式的校验和、格式版本号） |
| `crates/singlefs-core/src/root_record.rs` | 没有复用键（盘上格式的校验和、格式版本号） |
| `crates/singlefs-core/src/mounted_session.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-core/src/code_two_tree.rs` | 没有复用键（盘上格式的校验和、格式版本号） |
| `crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs` | C35（另有 write_fingerprint 是产物里的观测量，不是复用键） |
| `crates/singlefs-core/src/transaction.rs` | 没有复用键（盘上格式的校验和、格式版本号） |
| `crates/singlefs-harness/tests/journal_last_record_flag_of_a_multi_record_publish.rs` | 没有复用键（测试里的摘要、指纹只作断言比较或观察者另读输入） |
| `crates/singlefs-harness/tests/write_accounting_per_structure.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-core/src/records.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-core/src/allocation_record_tree.rs` | 没有复用键（盘上格式的校验和、格式版本号） |
| `crates/singlefs-harness/tests/an_abandoned_roots_rollback_floor_counts_toward_the_effective_floor.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/replayed_version_takes_the_instance_table_and_allocation_record_tree_from_the_new_root_segment.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/rollback_target_left_in_a_hole_outside_the_one_revolution_window_is_a_candidate.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/instance_table_second_page_write.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-core/src/journal.rs` | 没有复用键（盘上格式的校验和、格式版本号） |
| `crates/singlefs-core/src/extent_tree.rs` | 没有复用键（盘上格式的校验和、格式版本号） |
| `crates/singlefs-harness/tests/admission_checkpoint_cost_per_device_paths.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/release_checksum_quarantine.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/abandoned_root_whose_account_is_unreadable_passes_when_the_current_account_records_later_units_over_its_slots.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/checker_judges_reserved_bytes_and_widths_it_used_to_skip.rs` | 没有复用键（测试里的摘要、指纹只作断言比较或观察者另读输入） |
| `crates/singlefs-harness/tests/shadow_ledger_candidates_and_the_deferred_reclaim_threshold_take_the_whole_ring.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/writable_mount_refusal_judged_against_the_d23_list.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/pool_checker_gap_row_conditions_and_incomplete_reference_pairs.rs` | 没有复用键（测试里的摘要、指纹只作断言比较或观察者另读输入） |
| `crates/singlefs-harness/tests/writable_mount_refusal_for_an_unreadable_account_with_incomplete_records_judged_on_the_d23_list.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/pool_checker_floor_raise_ceiling_takes_only_roots_inside_the_one_revolution_window.rs` | 没有复用键（测试里的摘要、指纹只作断言比较或观察者另读输入） |
| `crates/singlefs-harness/tests/admission_formula_on_the_mounted_state.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/random_histories.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/mounted_recovery_reads_each_candidate_once_in_order.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/rebuild_with_an_unreadable_data_unit.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/commit_generated_units_fall_back_when_the_segment_runs_out.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-core/src/allocator.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/writable_mount_rejudges_f1_after_falling_back_and_accepts_when_the_reread_clears_the_fault.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/accounting_and_mapping_tree_split.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/writable_mount_refusal_for_a_publish_missing_records_recomputed_on_the_crash_image.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/fault_injection_fast_tier.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/unlink_single_file_publishes.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/acquisition_writes_the_witnessed_journal_tail_so_two_crashed_acquisitions_still_witness_the_newest_publish.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/named_unit_verification_stands_in_for_the_barrier_before_the_record.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/deletion_scenario_writes.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/rollback_by_a_forward_publish_verifies_every_copy_of_an_isolated_unit_the_target_references.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/model_comparison_judges_the_instance_table_and_allocation_generations_of_a_version_without_a_file.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/unequal_devices.rs` | 没有复用键（测试里的摘要、指纹只作断言比较或观察者另读输入） |
| `crates/singlefs-harness/tests/root_ring_turn_in_one_mount.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/model_compares_the_third_and_fourth_inode_leaf_containers.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/writable_mount_takes_a_second_instance.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/new_pool_file_creation_write_regions.rs` | 没有复用键（测试里的摘要、指纹只作断言比较或观察者另读输入） |
| `crates/singlefs-harness/tests/core_review_tree_table_duplicates_and_slot_one_search.rs` | 没有复用键（测试里的摘要、指纹只作断言比较或观察者另读输入） |
| `crates/singlefs-harness/tests/journal_ring_geometry_fields_are_recomputed_when_choosing_the_system_configuration.rs` | 没有复用键（测试里的摘要、指纹只作断言比较或观察者另读输入） |
| `crates/singlefs-harness/tests/overwrite_in_one_instance.rs` | 没有复用键（测试里的摘要、指纹只作断言比较或观察者另读输入） |
| `crates/singlefs-harness/tests/journal_scan_keeps_the_first_copy_of_a_duplicated_record_key.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/root_record_pointer_encryption_and_compression_fields_outside_the_first_version_are_refused_by_both_mounts.rs` | 没有复用键（测试里的摘要、指纹只作断言比较或观察者另读输入） |
| `crates/singlefs-harness/tests/acquisition_refuses_a_device_whose_witness_slots_turn_unreadable_and_the_remaining_writer_panics.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/writable_mount_reads_each_candidate_in_order_through_the_read_stage_cache.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/checker_known_bad_images.rs` | 没有复用键（测试里的摘要、指纹只作断言比较或观察者另读输入） |
| `crates/singlefs-harness/tests/system_configuration_slots_per_region.rs` | 没有复用键（测试里的摘要、指纹只作断言比较或观察者另读输入） |
| `crates/singlefs-harness/tests/mapping_tree_capacity_admission.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs` | 没有复用键（测试里的摘要、指纹只作断言比较或观察者另读输入） |
| `crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs` | 没有复用键（测试里的摘要、指纹只作断言比较或观察者另读输入） |
| `crates/singlefs-harness/tests/recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/journal_replay_applies_no_more_records_than_the_in_flight_limit.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/presumed_clause_checks.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/row_publish_admission.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/system_configuration_mutability_classes.rs` | 没有复用键（测试里的摘要、指纹只作断言比较或观察者另读输入） |
| `crates/singlefs-harness/tests/checker_cross_links_malformed_nodes_and_mapping_entries.rs` | 没有复用键（测试里的摘要、指纹只作断言比较或观察者另读输入） |
| `crates/singlefs-harness/tests/zero_unit_publishes_are_compared_with_what_the_row_publish_of_the_same_mount_handed_in.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/pointer_extent_offset_that_is_not_zero_is_refused_by_both_mounts.rs` | 没有复用键（测试里的摘要、指纹只作断言比较或观察者另读输入） |
| `crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/admission_ring_window_and_tombstone_prepay_on_the_publish_path.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/multi_record_publishes_of_transaction_zero.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/row_publish_checks_before_acquisition.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/common_admission/mod.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/random_history_generation_keeps_the_existing_weights_step_for_step.rs` | 没有复用键（测试里的摘要、指纹只作断言比较或观察者另读输入） |
| `crates/singlefs-harness/tests/system_configuration_fields_core_and_checker_refuse_the_same_table.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/checker_judges_reference_intervals_and_every_new_root_segment_field.rs` | 没有复用键（测试里的摘要、指纹只作断言比较或观察者另读输入） |
| `crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs` | 没有复用键（测试里的摘要、指纹只作断言比较或观察者另读输入） |
| `crates/singlefs-harness/tests/common_corrupted_allocation_record/mod.rs` | 没有复用键（测试里的摘要、指纹只作断言比较或观察者另读输入） |
| `crates/singlefs-harness/tests/warm_up_journal_counter.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/mount_transparency_judges_the_system_configuration_witness_by_r2.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/system_configuration_rollback_floor_and_layout_identity.rs` | 没有复用键（测试里的摘要、指纹只作断言比较或观察者另读输入） |
| `crates/singlefs-harness/tests/unlink_crash_image_remounted_twice_continues_to_identical_disks.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/common_tree_split/mod.rs` | 没有复用键（测试里的摘要、指纹只作断言比较或观察者另读输入） |
| `crates/singlefs-harness/tests/a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/admission_unlink_after_writing_up_to_the_edge_does_not_oversell.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/accounting_tree_grows_a_level_when_inode_number_watermark_steps_cross_one_leaf.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/writable_mount_refusal_for_no_valid_root_in_the_rebuilt_allocators_root_ring.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/multi_unit_file_write_and_read.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/common_admission_pool/mod.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/reference_filesystem_translation_of_histories.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/writable_mount_isolates_the_units_named_by_unapplied_journal_records_until_their_root_ring_slot_is_overwritten.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/root_ring_slot_unreadable_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/whole_device_loss_after_warm_up.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/an_abandoned_roots_rollback_floor_counts_toward_the_effective_floor.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/allocation_record_tree_root_level_of_writer_and_reader.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/first_file_version_checks_the_previous_record.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/instance_table_second_page_write.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/accounting_tree_node_full.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/common_pointer_head_corruption/mod.rs` | 没有复用键（测试里的摘要、指纹只作断言比较或观察者另读输入） |
| `crates/singlefs-harness/tests/instance_table_sweep_at_the_end_of_a_writable_mount.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/a_lone_surviving_device_reads_back_what_it_read_before_the_unmount.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/replayed_version_takes_the_instance_table_and_allocation_record_tree_from_the_new_root_segment.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/many_inodes_write_and_read.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs` | 没有复用键（测试里的摘要、指纹只作断言比较或观察者另读输入） |
| `crates/singlefs-harness/tests/inode_leaf_merge_and_borrow_against_an_independent_reference.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/writable_mount_of_a_formatted_pool.rs` | 没有复用键（测试里的摘要、指纹只作断言比较或观察者另读输入） |
| `crates/singlefs-harness/tests/writable_mount_rejudges_missing_records_after_falling_back_to_the_witnessed_version.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/reused_allocation_record_overlap.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/common_root_ring_holes/mod.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/rebuilt_root_takes_every_new_root_segment_field_from_the_record.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/journal_last_record_flag_of_a_multi_record_publish.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/release_checksum_quarantine.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/overwrite_in_one_instance.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/fault_injection_fast_tier.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/common_unwitnessed_publish/mod.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/root_ring_turn_in_one_mount.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/raising_the_floor_after_holes_at_the_oldest_end_of_the_one_revolution_window.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/admission_refuses_before_units_cannot_land_outside_the_mounts_clustered_segments.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/named_unit_verification_stands_in_for_the_barrier_before_the_record.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/system_configuration_fields_core_and_checker_refuse_the_same_table.rs` | 没有复用键（测试里的摘要、指纹只作断言比较或观察者另读输入） |
| `crates/singlefs-harness/tests/common/mod.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/root_ring_slot_unreadable_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/admission_formula_on_the_mounted_state.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/fsync_drop_and_devices_without_the_selected_version.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/rollback_target_left_in_a_hole_outside_the_one_revolution_window_is_a_candidate.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/container_trees_grow_a_level_when_the_root_is_full.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/writable_mount_refusal_after_the_read_stage_on_a_read_faulted_crash_state_is_judged_apart_from_the_witness_criterion.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/random_histories.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/reuse_after_raising_the_floor.rs` | 没有复用键（测试里的摘要、指纹只作断言比较或观察者另读输入） |
| `crates/singlefs-harness/tests/rollback_lineage_record_and_its_checkpoint_cost_term.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/row_publish_admission.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/row_publish_checks_before_acquisition.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/unlink_mount_continuation_and_the_emptied_version.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/accounting_and_mapping_tree_split.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/instance_table_chain_read.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery.rs` | 没有复用键（测试里的摘要、指纹只作断言比较或观察者另读输入） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/rollback_lineage_record_and_its_checkpoint_cost_term.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/inode_leaf_absorb_and_borrow_on_the_unlink_path.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/src/new_pool_file_creation_regions.rs` | 没有复用键（产物行打区域 sha256） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/unlink_of_a_file_spanning_two_extent_lower_leaves_in_product_batches.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/src/lib.rs` | 没有复用键（模块声明） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/shadow_ledger_candidates_and_the_deferred_reclaim_threshold_take_the_whole_ring.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/src/reference_filesystem_translation.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` | 没有复用键（测试里的摘要、指纹只作断言比较或观察者另读输入） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/multi_unit_file_write_and_read.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/admission_raises_the_floor_before_refusing.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/src/sha256.rs` | 没有复用键（SHA-256 实现本身；在 GPU 闭包里，见 C1） |
| `crates/singlefs-harness/src/model.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-checker-tier/tests/record_checker_judges_absence_by_the_persisted_set.rs` | C36（设点文件或用库的用例：贡献路径、项名、种子串，键由引擎算，见 C11、C14、C15） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/rollback_by_a_forward_publish.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/src/history.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/src/mount_transparency.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/src/memory_pool.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-checker-tier/tests/common_crash_point_observers/mod.rs` | 没有复用键（测试里的摘要、指纹只作断言比较或观察者另读输入） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/writable_mount_takes_a_second_instance.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-checker-tier/src/crash_judge_conclusions.rs` | C39（没有键） |
| `crates/singlefs-core/src/unlink.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/src/model_comparison.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/entries_after_mount_refuse_swapped_or_behind_devices.rs` | 没有复用键（测试里的摘要、指纹只作断言比较或观察者另读输入） |
| `crates/singlefs-harness/tests/model_answers_unmount_with_the_floor_at_the_current_version.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs` | 射程外（输入指纹只是钉表行自核，复用键在 scripts/experiments/e165-pin-red-examples.py） |
| `crates/singlefs-harness/tests/core_review_geometry_back_chain_and_empty_inode.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs` | C35 |
| `crates/singlefs-harness/tests/entries_after_a_writable_mount_refuse_a_device_without_a_self_verified_system_configuration.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/instance_table_sweep_at_the_end_of_a_writable_mount.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/admission_refuses_before_units_cannot_land_outside_the_mounts_clustered_segments.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/unreadable_abandoned_root_slot.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/pointer_extent_offset_that_is_not_zero_is_refused_by_every_tree_reader.rs` | 没有复用键（测试里的摘要、指纹只作断言比较或观察者另读输入） |
| `crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs` | 没有复用键（测试里的摘要、指纹只作断言比较或观察者另读输入） |
| `crates/singlefs-harness/tests/tree_nodes_and_the_central_mapping_on_mounted_read.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/core_review_unit_area_start_and_publish_limits.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/read_recovery_fields_follow_the_fallback_to_the_witnessed_version.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/crash_recovery_judge_refuses_a_root_older_than_the_newest_persisted_one.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/pool_checker_tree_identifier_birth_txg_agrees_across_every_readable_root.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/multi_record_publishes_of_transaction_zero.rs` | 没有复用键（测试里的摘要、指纹只作断言比较或观察者另读输入） |
| `crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/src/scenario.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs` | 没有复用键（测试里的摘要、指纹只作断言比较或观察者另读输入） |
| `crates/singlefs-harness/src/fault_injection.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |
| `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` | C2、C39 |
| `crates/singlefs-harness/tests/common_images_for_the_checker_tier/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs` | 没有复用键（只有领域词 version：根版本、文件版本、格式版本） |

## 9 要主 agent 跑的命令（可选，用来拿 Rust 实编坐实 §1.3 的移植结论）

这一路按派发不起 cargo。要坐实 C1–C4，在一份仓副本里（不在主工作区）逐个造 §1.3 的改动后跑，工作目录是副本根，内存上限与线程数照主 agent 的口径：

```
bash scripts/hosts/capped.sh <线程数> cargo build -p singlefs-checker-tier
awk -F'\t' '$1=="gpu_judge"' target/debug/build/singlefs-checker-tier-*/out/conclusion_code_digests.tsv
```

看 `gpu_judge` 那一行的摘要是否随改动变；与 §1.3 每一行的前 16 位比（基线 `d17732ad426081b7`）。`cargo build` 不跑编出来的代码，不是重型。

## 10 没做什么

- 没起 cargo、门禁、测试二进制与任何自证，没动判定库与标记，没写源码摘要缓存；§1.3、§1.4 的数是 python 移植算法在草稿副本上算的，移植算法与编译表 137/137 逐行相等（§1.1），Rust 实编没做。
- 块键（C11–C15）、核对复用（C26）、观察者（C17–C19）、录入段（C30）那几条只读了代码，没在副本里造现象（要跑 checker 档测试，是重型）。
- 没量任何一处的代价（多少块、多少挂钟因此重判）；两库重复判的规模、录入段每趟的耗时都是推的，以日志路子为准。
- `crash_amplification.rs` 没有逐行读完（见第 4 节）；`crash.rs`、wgsl 五份没逐行读（只核它们有没有自己的键）。
- 没给重做方向写代码，也没判「按调用图收单元」的具体做法；C32 要不要剪枝、GPU 与 CPU 互借、驱动版本进不进键交设计。
- 草稿副本 `copy/` 交回前删掉；留下的是 `tools/closure_port.py`、`tools/scenarios.sh`、`tools/destinations.py` 与几份输出（`gpu-closure.txt`、`scenarios.out`、`table-*.sorted`、`grep-list.txt`、`triage.tsv`、`version-only.txt`、`library-users.txt`、`other-hits.txt`、`destinations.md`）。
