# merge-verify-disk-governance-r3：攻方腿（Opus）报告（2026-11-19）

被判对象：`scripts/gate/merge-and-verify.sh`（开工快照 judged-sha256 三份 `sha256sum -c` 都是 OK）。

**两段写成。** 前一段是前一条攻方腿写的：它在草稿目录的写者锁被这一次派发接手之后停手，模型没建。后一段是承接它的这条腿（前一段里叫 subagent(A)）接着写的：建了模型、补了目录形那一格、撤回了前一段「目录形没打中」那一句。

- 前一段原样留着，改过的地方都标「（承接改）」：这段说明、一览表、没打中里目录形那一行、11160 对 19480 那一条、末尾两节。新加的节标题带「（承接）」。
- 派发提示说前一条腿「报告一行没落盘」，与现场对不上：这份文件在承接开工时已有 135 行，修改时刻晚于草稿目录 `.writer-lock` 里这一次派发的时刻（`ls -la` 与锁文件的 `dispatched_date` 比的）。当场说明，照现场接着写。

### 复跑命令与文件 sha256（承接改）

- 模型目录 `research/prompts/merge-verify-disk-governance-r3-opus-model/`：
  - `0fa66b1880e7c2e072b667e09db029ae34fc8f49ad06f2c7aa063f38dbf4aad4  tools/rerun-from-a-clean-work-directory.sh`（从第二轮攻方模型原样拷来）
  - `76610e3b5d7cb10a4c9e790ff20d3d19281af0f8e0e751f90252f2d8b459dc76  tools/rerun.sh`（从 `scripts/agents/three-way-rerun-template.sh` 拷来，只换 rerun_steps 那一段）
- 复跑命令见末尾「要主 agent 跑的命令」第 1 条。第二个参数（冻结副本）给仓根：`rerun.sh` 先核 `scripts/gate/merge-and-verify.sh` 的 sha256 是开工快照那一份（5dbf5173 起头），对不上退 2、不跑；仓根那份换过之后，要给一棵还是那一份的树。
- 被判程序原样从被判脚本里取：挑目标那一段（`PY_ITEMS`）与算包那一段（`PY_MAIN_TREE_FINGERPRINT`）。cargo 一侧用 `cargo metadata --offline`；另有目录形一次 `cargo test --list` 与判决 5 玩具两臂的 cargo test、cargo build（都是玩具大小）。
- 每格一行 `name=<格> … verdict=matched|mismatched`，末行 `name=summary matched=<数> mismatched=<数>`。verdict 判的是读数与跑前写在 `rerun.sh` 每格注释里的预期一不一致，不是「被判程序对不对」。
- 这条腿没跑 `rerun.sh`（被判程序是本仓代码，要经任务服务交）。

下面的结论里：

- cargo 的做法是量的：两条腿都在草稿目录里直接跑过 `cargo metadata`（不编，cargo 1.98.0）；承接的这条腿跑的是 `probe2.sh`，输出 `probe2.txt`，都在草稿目录。
- 被判程序的做法是推的：照代码读出来，没跑；模型里每格写了跑前的预期，等主 agent 跑出来。

## 各格判定一览

| 格 | 判定 | cargo 一侧 | 被判程序一侧 |
|---|---|---|---|
| 按根文件比：没写 edition 的包（edition 2015）新加一段 [[test]] / [[bin]] | 打中：该退 3 而没退 | 量的 | 推的 |
| 按根文件比：path 写成 `./tests/a.rs`、名字相同 | 打中：不该退而退了 | 量的 | 推的 |
| 按根文件比：目录形（`tests/d/main.rs`、`src/bin/tool/main.rs`）的段不写 path（承接改） | 打中：不该退而退了；撤回前一段这一格的「没打中」 | 量的 | 推的 |
| 按根文件比：换段种、撞根文件、autobins = false | 没打中 | 量的 | 推的 |
| 认 workspace = true：registry 依赖经 `[patch.crates-io]` 换成工作区里的包 | 打中：该算进的依赖方没算进 | 量的 | 推的 |
| 认 workspace = true：package 改名、dev / build / target 条件依赖、点号键（承接改） | 没打中 | 量的（`probe2.txt` 的 `== wsforms`） | 推的 |
| 认 workspace = true：工作区根不在补丁树的根 | 没打中（本仓 0 处，只碰指纹那一路里的非成员包） | 没量 | 推的 |
| 能不能核：481、CARGO_BIN_EXE_、finish_attempt 行号、11160 对 19480、73%–94%（承接改） | 见第三节：前三样量的；11160 对 19480 模型里加了拆块那一格、待跑；73%–94% 只抽到 1 份 | | |

所有格（承接改）：被判程序一侧在模型里各有一格，`rerun.sh` 的格名写在各节里；全部是推的，等主 agent 跑出 `name=summary`。
按被判程序的版本分臂，四处打中都是「打中不分辨臂」：被判那一版与改前写法一样中（打中 1、2、4 的改前写法是弄坏开关 manifest-targets-compared-by-name，即按（段，名字）比；打中 3 的是直写 registry 版本那一样，改前改后都漏），病根在两版共用的「照 cargo 推目标的规矩手算」与「只认 path」。下面各节四句里的「分辨两臂」说的是按输入分的两臂（写不写 edition、path 怎么写），不是这个（`.claude/rules/evidence-discipline.md`@判据自己也会写错：打中之后先判是哪一种 那一张表第一行）。今天本仓能走到的是打中 2 与打中 4；打中 1、打中 3 要先有今天 0 处的写法（没写 edition 的包、仓根 `[patch]`）。

## 一、按根文件比

### 打中 1：没写 edition 的包，cargo 停掉别的自动认的目标，脚本不退 3

- 反例：包 alpha 没写 edition（cargo 按 2015 算），tests/ 下有 a.rs、b.rs、c.rs。补丁只往 Cargo.toml 加一段 `[[test]] name = "c" path = "tests/c.rs"`。
- cargo（量的，`cargo metadata --offline --no-deps`）：
  - 这一臂列出的测试只剩 `test c tests/c.rs`，a、b 不再是目标；
  - 只把这个包写成 `edition = "2021"` 的那一臂，a、b、c 都列得出；
  - 同样的写法放在 [[bin]] 上（加一段 `[[bin]] name = "one"`），src/bin/two.rs 也不再是目标。
- 被判程序（推的）：
  - 新侧只看 autotests / autobins，不看 edition：「if package_section.get(automatic_key, True) is False:」（`scripts/gate/merge-and-verify.sh:1879`）；
  - 所以新侧照样把 a、b 记成自动认的目标，`shadowed_automatic_targets` 不报，补丁落到 `--test c`、退 0。
- 文件头这一句不成立：「src/bin/<名>/main.rs 里名字与根文件都没被段点到的是自动认的，与 cargo 认的一样」（`scripts/gate/merge-and-verify.sh:107`）。
- 四句：
  - 分辨两臂：edition 2021 那一臂不中。
  - 被判程序做决定时看得到判别它的东西：Cargo.toml 的 `[package]` 里有没有 edition。
  - 满足的是文件头「cargo test 从此不跑那份文件、一句不报」那一分句。
  - 第二轮的改法 B 按根文件比，碰不到这一格：两侧根文件集合在被判程序眼里一样。
- 本仓今天的成员都写了 `edition = "2021"`，这种写法 0 处。分量与第二轮判决 3「本仓今天 0 处这种写法，照修」同一类。
- 走不到的理由（承接）：今天要走到这一格，得先有一个没写 edition 的已有包；补丁新建的包旧侧是空串、往已有包里删 edition 是 `[package]` 有改动，这两种都整份判不出。所以今天本仓走不到，是等写法出现才中的一格。
- 模型格（承接）：`edition_2015_with_path_root`、`edition_2015_without_path_root`、`edition_2015_bin_root`（被判程序，预期不退 3、只挑 c 或 one）；`edition_2015_*_cargo`（预期 a、b 或 two 不在）；对照 `edition_2021_*`（被判程序的输出与没写 edition 的那一份一样，cargo 列得出 a、b、c）；`*_by_name` 是改前写法那一臂。
- 用户动作放开扫过（承接）：新加的段写不写 path 两样、test 与 bin 两种段。

### 打中 2：path 写成 ./tests/a.rs、名字相同，不该退而退了

- 反例：包 alpha（edition 2021）的 tests/a.rs 原是自动认的目标 a。补丁加一段 `[[test]] name = "a" path = "./tests/a.rs" harness = true`。
- cargo（量的）：metadata 列出 `test a tests/a.rs`，src_path 已经归一，同一份文件、同一个名字照跑。
- 被判程序（推的）：
  - 根文件原样取 path 的字符串：「root = entry_root_paths(directory, package, section, entry)[0]」（`scripts/gate/merge-and-verify.sh:1876`）；
  - 两侧按字符串比：「if not automatic or (after is not None and after[0] == root):」（`scripts/gate/merge-and-verify.sh:1900`）与「if broken != "manifest-targets-compared-by-name" and (section, root) in new_roots:」（`scripts/gate/merge-and-verify.sh:1902`）都不中；
  - 结果打一行「改后同名的 [[test]] 段指到 ./tests/a.rs」、判成顶掉、退 3。
- 四句：
  - 分辨两臂：path 写成 `tests/a.rs` 的那一臂不退。
  - 被判程序做决定时看得到判别它的东西：path 的字符串。
  - 满足的是文件头「根文件在新侧不再是…任何目标的根文件」那一分句（这里为假而被当成真）。
  - 第二轮的改法 B 不修这一格：改前按（段，名字）比，这一格同样退 3。
- 后果只是多弹一次窗问用户，不漏测。
- 模型格（承接）：`dot_dot_prefix_*`（`./tests/a.rs`）、`dot_dot_inner_*`（`tests/./a.rs`），段里另写 `harness = true` 与 `test = true` 两样都扫；`_root`、`_by_name` 两臂预期都退 3 一行、点名 test 目标 a；`_cargo` 预期 `test:a:tests/a.rs`。

顺带看到一处，病根不在这一轮改动里：`scripts/gate/rerun-failed-tests.py` 也不归一路径。

- 「relative = entry.get("path", default_root)」（`scripts/gate/rerun-failed-tests.py:251`）
- 「if relative not in declared and name not in declared:」（`scripts/gate/rerun-failed-tests.py:267`）

补丁加 `[[test]] name = "renamed" path = "./tests/a.rs"`、同一补丁又改 tests/a.rs 时：

- cargo 只认 renamed，a 不再是目标（量的，metadata 列出 `test renamed tests/a.rs`、没有 a）；
- 被判程序把 tests/a.rs 落到它以为还在的 `--test a`（推的）。
- `cargo test -p alpha --test a` 会报「no test target named」，这一行是推的、没跑。

### 打中 4（承接）：目录形的段不写 path，cargo 认得出，脚本说落不到、退 3

- 反例：包 alpha（edition 2021）里 `tests/d/main.rs` 原是自动认的目标 d。补丁只往 Cargo.toml 加一段不写 path 的 `[[test]] name = "d"`，段里另写一个键（`harness = true` 或 `test = true`；写段的通常是为了加 `harness = false` 这类键）。bin 那一样：`src/bin/tool/main.rs` 加一段 `[[bin]] name = "tool" test = true`。
- cargo（量的，`probe2.txt` 的 `== dirform`、`== dirbin`）：列出 `test d tests/d/main.rs`、`bin tool src/bin/tool/main.rs`，照是目标。
- 被判程序（推的，照代码走一遍）：
  - 两侧比那一步认得出：缺省根文件按「candidates = (["src/main.rs"] if section == "bin" and name == package else []) + [f"{folder}/{name}.rs", f"{folder}/{name}/main.rs"]」（`scripts/gate/merge-and-verify.sh:1810`）挑在的那一个，两侧都是 tests/d/main.rs，不算顶掉。前一段「目录形…两侧根文件相同、不退」说的就是这一步，这一步本身对。
  - 落到目标那一步落不到：目标清单取自另一份文件，「targets_of = {name: rerun.test_targets_of_package(directory, manifest, name) for name, (directory, manifest) in packages.items()}」（`scripts/gate/merge-and-verify.sh:1987`）。那一份给不写 path 的段只认一个缺省根文件：「default_root = "src/main.rs" if section == "bin" and name == package else f"{directory}/{name}.rs"」（`scripts/gate/rerun-failed-tests.py:250`）。tests/d.rs 不在，d 不进清单；tests/d/main.rs 因为名字 d 已被段点到，也不按自动认的进。
  - 于是这一段的根文件 tests/d/main.rs 不在目标清单里，子目录那一条也找不到 mod 它的目标，退到「return f"{label} 落不到测试目标（根文件」（`scripts/gate/merge-and-verify.sh:2153`），整份判不出、退 3。
  - 同一处还连着一格：段已在、补丁只改 tests/d/main.rs 的正文，也落不到、退 3（模型格 `dirform_body_root`）。
- 四句：
  - 分不分辨臂：不分。被判那一版与改前按名字比的那一臂一样退（模型格 `dirform_*_by_name`）；病根是同一个脚本里两份「缺省根文件」的算法不一样，一份在这一轮的被判文件里、一份在 `scripts/gate/rerun-failed-tests.py`。
  - 被判程序当时看得到判别它的东西：看得到，tests/d/main.rs 就在树里，两侧比那一步已经找到了它。
  - 满足的是文件头哪一分句：「落不到又不是这两种的整份判不出」（`scripts/gate/merge-and-verify.sh:101`）的字面成立，而文件头同一节「写了的段照写，没写 path 的照上面的缺省路径」（`scripts/gate/merge-and-verify.sh:106`）说的缺省路径含 `tests/<名>/main.rs`，cargo 也认，所以这一句退 3 是不该退而退了。
  - 第二轮改法 B、C 在这几格上还中：B 只动两侧比那一步，C 只动算包，都碰不到目标清单。
- 今天本仓走得到：成员里目录形的目标 0 份（`ls -d crates/*/tests/*/main.rs crates/*/src/bin/*/main.rs` 没有命中），但补丁新加一份 `tests/<名>/main.rs` 连同一段不写 path 的 [[test]] 就走到；后果是多弹一次窗，不漏测。
- 模型格：`dirform_harness_*`、`dirform_test_true_*`、`dirform_bin_root`（预期退 3 一行「落不到测试目标」、一项不挑）、`dirform_*_cargo`（预期 cargo 列得出）、`dirform_cargo_list`（预期 `cargo test --test d -- --list` 列出 `d_runs: test`）、`dirform_body_root`。

## 二、认 workspace = true

### 打中 3：registry 依赖经 [patch.crates-io] 换成工作区里的包，build 漏掉依赖方

- 反例：工作区成员 wcore、user。补丁只改 wcore，有两种写法：
  - 树根写 `[workspace.dependencies] wcore = "0.1"` 与 `[patch.crates-io] wcore = { path = "crates/wcore" }`，user 写 `wcore = { workspace = true }`；
  - user 直接写 `wcore = "0.1"`，树根同样加那条 patch。
- cargo（量的，`cargo metadata --offline`，带依赖、不编）：两种写法都解析成 `user -> ['wcore@path']`。
- 被判程序（推的）：
  - 认继承只查 `[workspace.dependencies]` 里写了 path 的项：「elif entry.get("workspace") is True and key in inherited:」（`scripts/gate/merge-and-verify.sh:3415`）；
  - `workspace_dependency_paths` 只收写了 path 的：「return {key: os.path.normpath(os.path.join(tree, entry["path"])) for key, entry in table.items() if isinstance(entry, dict) and isinstance(entry.get("path"), str)}」（`scripts/gate/merge-and-verify.sh:3393`）；
  - 两处都不读 `[patch]`，build-packages 只算出 wcore。
  - 依赖方 user 里的单态化错与链接错漏过去，与第二轮判决 3 同一种后果。
- 文件头这一句的目的落空：「只编补丁改到的包与工作区里照 path 依赖传递依赖它们的包」（`scripts/gate/merge-and-verify.sh:130`）。
- 四句：
  - 分辨两臂：去掉 `[patch]`、user 直接写 path 的那一臂算得进。
  - 被判程序做决定时看得到判别它的东西：树根 `[patch.crates-io]` 那一张表。
  - 满足的是「该算进的包没算进」。
  - 第二轮的改法 C 只认 `[workspace.dependencies]` 的 path，碰不到这一格。
- 第二种写法（user 直接写 `wcore = "0.1"`）改前就漏，不是这一轮带进来的。本仓今天仓根没有 `[patch]` / `[replace]`，成员里也没有 workspace = true（`grep` 两样都 rc=1）。
- 判据分句的补充（承接）：文件头字面写的是「照 path 依赖」，user 对 wcore 的依赖字面上不是 path 依赖，按字面这一格不在它的射程里；按它自己的为什么那一句「编坏的只会是改到的包与依赖它们的包」（`scripts/gate/merge-and-verify.sh:135`），cargo 解析出的依赖方是 user，该编。所以这一格打中的是那一句的目的，不是字面（`.claude/rules/evidence-discipline.md`@判据自己也会写错：打中之后先判是哪一种 的「打中归错了判据」那一行）。
- 走不到的理由（承接）：今天要走到，得先有仓根 `[patch]`；补丁自己加 `[patch]` 就改到仓根 Cargo.toml，测试那一步整份判不出、退 3。
- 模型格（承接）：`patch_override_inherit_root`、`patch_override_explicit_root`（预期 build-packages 只算出 wcore）、`patch_override_*_cargo`（预期 `user@path`）。

### 没打中的那几种写法，模型里补了一格（承接）

- 树：core 与五个依赖它的包、一个不依赖谁的 bystander。依赖都写成继承：`[dependencies.core] workspace = true`、写在 `[dev-dependencies]`、`[build-dependencies] core.workspace = true`（点号键）、`[target.'cfg(unix)'.dependencies]`，以及树根 `core-renamed = { path = "crates/core", package = "core" }` 改名、成员写 `core-renamed = { workspace = true }`。
- cargo（量的，`probe2.txt` 的 `== wsforms`）：五个包都解析到 core。成员一侧写 `{ workspace = true, package = "core" }` 的，cargo 直接拒（`== wsmemberpackage` rc=101，「error inheriting `mine` from workspace root manifest's `workspace.dependencies.mine`」），所以改名只能写在树根那一侧，键名一致，`workspace_dependency_paths` 按键查对得上。
- 被判程序（推的）：`path_dependencies` 扫四张表、条目是表就认（点号键读成的也是表），五个都算得进。
- 模型格：`workspace_forms_root`（预期 alias builduser cfguser core devuser user，不含 bystander）、`workspace_forms_ignored`（弄坏开关 workspace-dependencies-ignored 那一臂只剩 core）、`workspace_forms_cargo`。

## 三、前两轮报出的数字与已定几条能不能核

- `picked_test_targets=481`：今天验证包 `crates/singlefs-verification/tests/*.rs` 是 483 份（量的，`ls … | wc -l`）。481 是第二轮那棵树上的数，判决 1 要的只是「挑上的个数等于 tests/ 下 .rs 的个数」，不靠 481 这个值。改法 A 已经不做。
- 「481 份里写了 `CARGO_BIN_EXE_` 的一份都没有」：只对验证包成立（量的，验证包 0 份）。全仓的集成测试里写它的有 4 份（量的，`grep -rlF CARGO_BIN_EXE_`）：
  - `crates/singlefs-unit-graph/tests/` 2 份，起 unit-graph；
  - `gpu-verification-server/tests/` 2 份，起 gpu-verification-server。
  这 4 份就是判决 4 那一格（只改 bin 根文件不挑起它的测试）在本仓今天会漏挑的量。用户已定按改动测，这里只给数、不再攻。
- 判决 6 引的「第 5995 行起的 `finish_attempt`」：今天在 `task-server/task_server.py:5984`（量的，`grep -n 'def finish_attempt'`）。行号漂了，不影响结论。
- 判决 5 的 `target_kib=11160` 对 `19480`：前一条腿没复跑（模型没建）。承接改：模型里加了 `debuginfo_breakdown` 一组，照第二轮攻方模型同一棵玩具树、同样的先后重跑两臂，把编译目录拆成可执行、rlib 与 rmeta、incremental 三块，另读 dbg-user 可执行里 `.debug_*` 段的字节数；格 `debuginfo_gap_source` 判的就是下面推翻条件那一句（预期：可执行那一块的差额大于 rlib 与 rmeta 那一块）。等主 agent 跑。
  - 推的：差额多半来自 debug = 0 时 cargo 默认剥掉可执行文件里的调试信息，不来自 rlib。玩具树里依赖小，「关掉调试信息时依赖整份重编一份」那一项几乎为 0；本仓的依赖里有 wgpu、rocksdb 这类，这一项量不到。
  - 所以这组数说明不了本仓的方向。判决 5 自己已写「玩具树，只当方向，不外推」，这里补一句：连方向也是由玩具里另一项撑的。
  - 推翻条件：把两臂编译目录拆成可执行、rlib/rmeta、incremental 三块，差额若主要在 rlib，这一句就不成立。
- 文件头引的 73%–94%（承接）：出处在这一轮的禁读记录里，原量法核不动。这条腿另抽了 1 份：仓根 `target/debug/deps/unit_graph-3aec26e79f64e211`（文件 52291768 字节，2026-11-16 编的，编它时的档位没查），`size -A` 读出 `.debug_*` 段合计 40205330 字节，占文件 77%（量的，只 1 份）。落在 73%–94% 之内，只说明不矛盾。
- 文件头引的「595 个可执行、合计 233.4 GiB」（承接）：`records/2026-11-13-全量提交门禁前收尾调度.md` 第 1503 行确有这一句（量的，`sed -n 1503p`）。
- 背景材料说的「门禁开关复跑 123 条」（承接）：文件头 `#   MERGE_AND_VERIFY_BREAK=` 起头的开关行 122 条（量的，`grep -c`），加 SINGLEFS_WORK_ROOT_BREAK 一条是 123，与 `.claude/gate.d/break-switch-replay-files.tsv` 第 72 行写的「MERGE_AND_VERIFY_BREAK 一百二十二条、SINGLEFS_WORK_ROOT_BREAK 一条」对得上。开关复跑本身没复跑。
- 已定几条撑不撑得住（承接）：判决 2、3 由这一轮的改法 B、C 修了它们点名的那一格（自证格在被判脚本里），这一轮打中的四格都在那两格旁边、不推翻那两条判决本身；判决 5 见上；判决 4、6 这一轮没新的反证。

## 没打中的形状（试过的走法与范围）

都在 edition 2021 的单包玩具树上，cargo 一侧用 `cargo metadata` 量。

- 换段种：`[[bin]] name = "a" path = "tests/a.rs"`。cargo 同时认 bin a 与 test a（量的）。被判程序新侧照样有自动认的 test a、不退 3，落到 `--bin a`（推的）。两边一致。
- 撞根文件：自动认的 a、b 并存，加 `[[test]] name = "b" path = "tests/a.rs"`（或 name = "a" path = "tests/b.rs"）。tests/b.rs（或 tests/a.rs）没人认了，被判程序照退 3，判得对（推的）。承接：cargo 列出的只有 `test b tests/a.rs`（量的，`probe2.txt` 的 `== collide`）；模型格 `collide_root`、`collide_cargo`。换段种那一条的模型格是 `switch_root`、`switch_cargo`。
- 换段种的一处旁注（承接，不算打中）：目标清单按根文件做键，cargo 里并存的 bin a 与 test a 根文件同是 tests/a.rs，清单里只留先加进去的 `--bin a`：「if relative not in found and exists(relative):」（`scripts/gate/rerun-failed-tests.py:233`）。这一份补丁只该跑 bin a，结果对；之后只改 tests/a.rs 的补丁也只跑 `--bin a`、不跑 test a。test a 与 bin a 编的是同一份源码、跑的是同一批 #[test]，漏不漏要看两者的 cfg 与特性，没往下量。
- 目录形（承接改，撤回）：前一段这里写「自动认的 a 是 `tests/a/main.rs`，段写同一路径或不写 path，两侧根文件相同、不退（推的）」。两侧比那一步确实不退；段不写 path 时落到目标那一步落不到、退 3，见打中 4。撤回的依据是 `scripts/gate/rerun-failed-tests.py:250` 那一行只认 `tests/<名>.rs`，前一段只走了两侧比、没走落目标。段写同一路径（`path = "tests/a/main.rs"`）的那一样两步都过（推的）。
- autobins = false：cargo 连 src/main.rs 也不认（量的）。被判程序同样（推的）。改 autotests / autobins 本身属于 `[package]` 有改动，整份判不出。
- 名字不同、加了 test = false 或 required-features 的段指到自动认的文件：被判程序不报顶掉，但分别打「不跑」的一句或记要特性，不是一句不报（推的）。
- workspace = true 带 package 改名（`alias = { path = "crates/wcore", package = "wcore" }`）、写在 dev / build 依赖、`target.'cfg(unix)'.dependencies`：`path_dependencies` 按 path 认、四张表都扫，都算得进（推的，没量）。承接改：cargo 一侧已量、被判程序一侧进了模型，见第二节「没打中的那几种写法，模型里补了一格」。
- 工作区根不在补丁树的根：本仓 prune-server、scripts/experiment-apparatus 是另外的工作区。成员里没有 workspace = true，build 只留树根工作区的成员，只碰指纹那一路里的非成员包，0 处（推的）。

## 改法与各格（承接）

都是零轮：只在这条腿的读法上推过，没实现、没跑，没被攻过。「推的，待主 agent 跑」指模型里有对应的格，改法本身没有臂。

| 改法 | 修哪几格 | 第二轮改法 B、C 在这几格上 | 状态 |
|---|---|---|---|
| P1：根文件路径先归一（`entry_root_paths` 与 `scripts/gate/rerun-failed-tests.py:251` 两处都 `os.path.normpath`），两侧比与目标清单用同一种写法做键 | 打中 2（`dot_*`）；前一段顺带看到的 `./tests/a.rs` 改名那一处 | 还中 | 推的，待主 agent 跑 |
| P2：目标清单给不写 path 的段找缺省根文件时，与 `default_root_paths` 同一份候选（`tests/<名>.rs`、`tests/<名>/main.rs`；bin 另有 `src/main.rs`、`src/bin/<名>/main.rs`），改在 `scripts/gate/rerun-failed-tests.py:250` | 打中 4（`dirform_*`） | 还中 | 推的，待主 agent 跑 |
| P3：推自动认的目标时认 edition：`[package]` 没写 edition 或写 2015（`edition.workspace = true` 的照树根 `[workspace.package]` 取），而同种段写了至少一段、autotests / autobins 没写成 true 的，不推自动认的目标；两侧比与目标清单两处同改 | 打中 1（`edition_2015_*`） | 还中 | 推的，待主 agent 跑 |
| P4：算包时读树根 `[patch.<来源>]` 里写了 path 的项：依赖（键名或 package = 改的名字）与被 patch 的名字相同的，照 patch 的 path 认 | 打中 3（`patch_override_*`） | 还中 | 推的，待主 agent 跑 |
| P5（前一段提的）：两侧目标集合、目标清单与依赖图都改用 `cargo metadata` 取 | 打中 1 到 4 一起 | 还中 | 推的；旧侧要另放一份倒出来的 Cargo.toml 跑一次，依赖图那一半要离线索引在 |

P1 到 P4 各修一格、互不替代；P5 一样换掉四处，但把「照规矩手推」整个换成跑 cargo，牵动的是另一个文件与合入后验证的准备步骤，零轮里最重。

## 撤回（承接）

- 撤回前一段「目录形 没打中」那一格，依据见打中 4 与「没打中的形状」里目录形那一行。推翻这一撤回的观测：模型格 `dirform_harness_root` 读出 `refusals=0` 且挑上了 `--test d`。
- 撤回第二轮攻方腿提的「build 与测试用同一档」（第二轮判决没采）。依据：本仓压盘的量在可执行文件上（`records/2026-11-13-全量提交门禁前收尾调度.md` 第 1503 行的 595 个、233.4 GiB），可执行里调试段的占比这条腿抽的 1 份是 77%；build 与测试同一档等于把这一块留着，省下的只是依赖另编的那一份 rlib。这条依据一半是推的。推翻它的观测：`debuginfo_gap_source` 读出 mismatched（玩具里 rlib 那一块的差额更大），且在本仓规模上量出依赖另编那一份比可执行省下的多。

## 这条腿自己的限度

- 前一段写：写者锁被接手，模型目录、`rerun.sh` 与复跑包装都没建，第 8、9 步没做。承接改：模型目录、`rerun.sh` 与复跑包装已建，`bash -n` 过；第 9 步「整份跑一次」照规矩交主 agent，见下一节。被判程序一侧的读数全是推的。
- 这一题不是崩溃点题，没有崩溃状态要枚举，第 3b 步不适用。
- 取样的范围：都是单包或几个包的玩具树；改动只取「补丁只改一份 Cargo.toml」或「只改一份 .rs」两种形状。由写补丁的人定的几样（段里另写什么键、写不写 path、test 与 bin 哪种段、路径写法两种）在每格里放开扫了；没扫的：一份补丁同时改几个包、段里写 required-features 的那一支（它在两侧比之前就走 required_entry 记要特性）。
- 探针的内存与时间：没跑，推的估计是几分钟以内（约 30 次被判程序、20 次 cargo metadata、玩具大小的 cargo test 三次与 cargo build 两次），内存在几百 MiB 以内。
- 草稿目录里前一条腿建的玩具树 probe/、probe-patch/：交回前再看已经不在（`du -sh` 报不存在），不是这条腿删的。这条腿自己建的 probe2/（10 棵玩具树，560K，没有编译产物）交回前已删；cargo 那一侧的原样输出留在 `probe2.txt`。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；没往任务服务交任务；模型没跑，判决要引的数要主 agent 复跑。
- 没在仓副本上实现 P1 到 P5，没有改法那一臂的读数。
- 「工作区根不在补丁树的根」那一格没进模型，前一段的推的照旧。

## 要主 agent 跑的命令

前一段这里的两条（`probe-metadata.sh`、`probe-patch.sh`）由下面第 1 条的模型盖住了（承接改）：模型里 cargo 那一侧的格就是那两份探针量的东西，另加了被判程序一侧。

1. 模型整份跑一次。
   - 命令：`task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/merge-verify-disk-governance-r3-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/merge-verify-disk-governance-r3-opus-model-run /home/fy5090/code/singlefs/research/prompts/merge-verify-disk-governance-r3-opus-model`
   - 在哪起：仓根 `/home/fy5090/code/singlefs`。
   - 看哪几行：末行 `TASK_SERVER_RESULT` 的 `exit=0`；日志里 `name=judged_script sha256=5dbf51739d699464addf4a105aa12d9fdb1952c348ea5e4bd69eeaa0baf7210e`；末行 `name=summary matched=48 mismatched=0` 是四处打中与几格对照都照这条腿推的出了；有 `verdict=mismatched` 的那一行，它那一格这条腿推错了，照那一行的读数判。`exit=2` 是被判脚本换过或运行目录不合，没跑。
   - 跑完删运行目录 `/home/fy5090/.local/share/singlefs/work/merge-verify-disk-governance-r3-opus-model-run`（玩具树与编译目录都在里面）。
