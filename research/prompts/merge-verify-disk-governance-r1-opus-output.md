# merge-verify-disk-governance-r1 攻方腿报告（opus，2026-11-19）

被判对象：`scripts/gate/merge-and-verify.sh`（sha256 `c6badd7bdfb5505053ed6cb89a8f4f2db577539f46753a95aff0bca9f8b00d2a`，与 `research/prompts/merge-verify-disk-governance-r1-snapshot/judged-sha256.txt` 里那一行逐字相同，开工时 `sha256sum` 现算）、`.claude/main-agent.md`、`.claude/skills/merge-verify/SKILL.md`、根 `Cargo.toml` 的 `[profile.dev]`。改前基线 `~/.local/share/singlefs/work/merge-verify-drop-build-step/base/` 用 `diff -u` 比过（diff 落在草稿目录 `mv.diff`，1305 行）。

## 复跑命令与模型文件

模型目录 `research/prompts/merge-verify-disk-governance-r1-opus-model/`，两份文件：

| 文件 | sha256 | 行数 |
|---|---|---|
| `tools/rerun.sh`（从 `scripts/agents/three-way-rerun-template.sh` 拷，只换了 `rerun_steps` 那一段及它用的函数） | `1cfa75f915dae6a5213ab065d5a5b485300cf0d13983ad5a72e0c207bd89a169` | 399 |
| `tools/rerun-from-a-clean-work-directory.sh`（先删运行目录再调同一份 `rerun.sh`） | `0fa66b1880e7c2e072b667e09db029ae34fc8f49ad06f2c7aa063f38dbf4aad4` | 24 |

复跑命令（在新 shell 里起，`--cwd` 是本腿草稿目录）：

```
task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/merge-verify-disk-governance-r1-opus -- bash /home/fy5090/code/singlefs/research/prompts/merge-verify-disk-governance-r1-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/merge-verify-disk-governance-r1-opus/model-run /home/fy5090/code/singlefs/research/prompts/merge-verify-disk-governance-r1-opus-model
```

- 「冻结副本」那个参数填的是仓根：这一轮没有拷出的冻结树，`rerun.sh` 开头核 `scripts/gate/merge-and-verify.sh` 的 sha256 等于上面那一份，不等就退 2、什么都不跑（被判对象换过就不会拿新版本冒充）。
- 模型不涉崩溃状态，不建池、不调 `enumerate_crash_states`（第 3b 步那几条对它不适用）；起的是玩具 cargo 工作区与从被判脚本里原样取出的挑目标 python 段。
- 探针的内存与时间：没量（子 agent 不跑），推的：三棵两包玩具树、三棵一包玩具树，没有外部依赖，22 格里最重的是 `cargo clippy` 与 `cargo build`，合计推几分钟、内存推不到 2 GB；运行目录 `model-run` 里每棵树各有一个编译目录，看完由主 agent 删。
- 跑完看：`task wait` 末行 `exit=0`；`task-server/task log <号> --all` 里 `name=summary matched=22 mismatched=0` 是预期全中；每一格一行 `name=<格> … verdict=matched|mismatched`，跑前写死的预期在 `expected=` / `expected_items=` 里。

## 各格判定一览

| 攻击面 | 格 | 判定 | 依据 |
|---|---|---|---|
| 去掉 build | 补丁改 A 包的泛型代码，依赖 A 的 B 包实例化时单态化报错（常量求值失败） | 打中（按 cargo / rustc 语义推的，模型待主 agent 跑：`poly_after-postmono_*` 四格） | 下文「去掉 build」第 1 格 |
| 去掉 build | 补丁让 B 包按符号链接 A 包的函数找不到（链接错） | 打中（同上，`poly_after-link_*` 四格） | 下文第 2 格 |
| 去掉 build | 重编核对改读 clippy 的 Checking 行放过没重编的包 | 没打中 | 下文第 3 格 |
| 挑测试目标认 `[[test]]` / `[[bin]]` | 只改 `[[bin]]` 段的 path：挑了 `--bin`，起这个 bin 的集成测试（`CARGO_BIN_EXE_<名>`）没挑，旧版退 3 | 打中（推的，模型 `alpha_items_binpath`、`alpha_*_cli_test`） | 下文「挑测试目标」第 1 格；本仓 `singlefs-unit-graph` 就是这个形状 |
| 挑测试目标认 `[[test]]` / `[[bin]]` | 新加一段 `[[test]]` 的 name 与一个自动认的测试目标同名：那份测试从此不跑，脚本不报一句 | 打中（推的，模型 `alpha_items_shadow`、`alpha_*_list`） | 下文第 2 格 |
| 挑测试目标认 `[[test]]` / `[[bin]]` | 段外的键、name 缺或重名、旧侧倒不掉、新包、删包 | 没打中 | 下文「没打中的形状」 |
| 删上一次尝试的快照 | 别的任务、尝试号不小、KEEP_COPY、本机有进程在用 | 没打中 | 下文「删上一次尝试的快照」 |
| 删上一次尝试的快照 | 上一次尝试还在第二台上跑（peer-run-failed 重排） | 没打中（脚本文件头自认只核本机；重排与第二台那一趟谁先停核不动） | 同上 |
| 收尾守卫 | `BASHPID` 不等于 `$$` 而该收尾的时候 | 没打中 | 下文「收尾守卫」 |
| `[profile.dev]` | 改判定结局或复用键（admission、变异单元、unit-graph 工具链行、广谱变异之外） | 没打中 | 下文「[profile.dev]」与派发「关的是哪几条」第 2 条 |

打中 4 处，都落在两块：去掉 build（2 处）、挑测试目标认清单段（2 处）。四处的读数都还是「推的」：模型写好了、没跑，判决要引的数由主 agent 复跑。

## 去掉 build

被判的那一句：「合入后验证做 fmt、clippy、测试、验证包四步，不跑 cargo build：clippy 带 --all-targets 已对全部目标做类型检查，调试构建把测试二进制全链一遍只占盘。」（`scripts/gate/merge-and-verify.sh:7`）。测试那一步只编挑出的目标：「不带上依赖它们的包，也不跑包里没点到的目标」（`scripts/gate/merge-and-verify.sh:70`）。流程文字同一处改了：「另跑 fmt、clippy，只跑补丁改到的测试目标与变异行点名的测试目标」（`.claude/main-agent.md:57`）。

「只占盘」不成立：`cargo build` 除了链接，还做单态化与代码生成；`cargo check`（clippy 是它加 lint）不单态化、不链接。有两类错只在这一步报，而补丁改到的包的依赖方、包里没挑上的 bin / 集成测试 / example，现在合入后验证里谁都不编它们的代码生成。

### 第 1 格：单态化时才报的编译错（打中）

走法（模型 `poly` 那一组，`tools/rerun.sh` 的 `write_poly_tree`、`postmono_model`）：

1. 工作区两个包：`poly-core` 有泛型函数 `store<T>`，函数体里 `const { assert!(core::mem::size_of::<T>() <= 64, …) }`；`poly-user` 依赖它，`src/main.rs` 调 `poly_core::store([0_u64; 4])`（32 字节）。
2. 补丁只改 `crates/poly-core/src/lib.rs`，把上限从 64 收到 8。
3. 合入后验证照被判脚本：挑测试目标落到 `poly-core:src/lib.rs`（`--lib`），`poly-user` 是依赖方、不挑（`scripts/gate/merge-and-verify.sh:70` 那一句许可）；fmt、clippy 照跑；没有 build（`scripts/gate/merge-and-verify.sh:7` 许可）。
4. 该红而没红的那一句：`poly-user` 的 `main` 把 `store::<[u64; 4]>` 单态化时常量求值失败，这一句只有 `cargo build --all-targets -p poly-core -p poly-user`（改前那一步）报；clippy 只做类型检查，`poly-core` 的 `--lib` 测试里只实例化了 `u32`。合入后验证退 0，进第 2、3 步合进主工作区。

模型跑前写死的预期：`poly_items_postmono` 只挑 `poly-core:src/lib.rs|--lib`；`poly_after-postmono_clippy`、`poly_after-postmono_picked_test` 退 0；`poly_after-postmono_build` 退非 0（`first_error=` 是 E0080 那一类）；`poly_before_*` 三格都退 0（补丁之前是绿的）。推翻条件：`poly_after-postmono_clippy` 退非 0（这一版 clippy 在检查阶段就求了这个常量），则这一格撤回。

本仓里这个形状有没有：`crates/` 下泛型里的常量断言今天我没找到（grep `const {`、`const _: () =` 的命中都是非泛型的常量或 thread_local 初值，非泛型的 clippy 照样求值），所以这是「补丁可以造出来、门不再挡」，不是「已经漏过一次」。

### 第 2 格：链接错（打中）

走法（`after-link` 那一棵）：`poly-core` 原有 `#[no_mangle] pub extern "C" fn singlefs_model_answer()`，`poly-user/src/bin/links.rs` 在 `extern "C"` 块里按这个名字声明并调用；补丁只把 `#[no_mangle]` 去掉（符号名被改编）。clippy 不链接、照过；挑出的 `poly-core --lib` 测试照过；只有链接 `links` 这个 bin 时报找不到符号。预期：`poly_after-link_clippy`、`poly_after-link_picked_test` 退 0，`poly_after-link_build` 退非 0（`first_error=` 是链接失败那一行）。

本仓里按符号跨包链接的写法今天没有（`crates/` 下 grep `#[no_mangle]` 0 命中），同样是「造得出、不再挡」。

### 这两格的四句

1. 分不分辨臂：分辨。同一段历史上，改前那一臂（build `-p <改到的包与依赖它们的包>`）红、改后那一臂绿；模型的 `poly_*_build` 与 `poly_*_clippy` 就是两臂。
2. 系统当时看不看得到判别它的东西：看得到。补丁、依赖方的包名与源码都在快照里，改前就是用它们算 build 的包集合；改后是不去编，不是看不到。
3. 满足的是判据字面的哪一个分句：背景材料「问题」第 2 条第一个分项「链接错、只有调试构建才暴露的编译错」的前两种。
4. 跑前条款给的改法：背景材料没给改法（它只问会不会错）。这几格后来由提交时的整轮门禁兜住：上游 `check.sh` 的「cargo build --all-targets || die」（`.claude/singlefs-ai-sop/scripts/check.sh:86`）由门禁阶段「run_stage "构建与单测" bash "$SCRIPTS_DIRECTORY/check.sh" "$ROOT"」（`.claude/singlefs-ai-sop/scripts/gate.sh:556`）起。所以后果是发现从合入之前挪到提交时：这期间主工作区里是编不过的代码，第 3 步照样合进去。

我提的改法（只在我的模型上量过、被攻过零轮）：

| 改法 | 单态化那一格 | 链接那一格 | 盘 |
|---|---|---|---|
| 恢复改前的 build（只编改到的包与依赖它们的包），连同已加的 `[profile.dev] debug = "line-tables-only"` | 推的，待主 agent 跑（改前那一臂就是 `poly_after-postmono_build`） | 推的，待主 agent 跑（`poly_after-link_build`） | 推的：背景材料按节表推 138.2 GB→约 26.8 GB，没量 |
| 恢复 build、这一步另设 `CARGO_PROFILE_DEV_DEBUG=0`（调试信息整个关，只为看编不编得过、链不链得上） | 推的，待主 agent 跑（`poly_after-postmono_build_without_debuginfo`） | 推的，待主 agent 跑（`poly_after-link_build_without_debuginfo`） | 推的：比只留行号表再少 `.debug_line`；`CARGO_PROFILE_DEV_DEBUG` 不进复用键（`scripts/gate/admission.py` 的 `profile_variable_left_out` 按整个键名去掉 `_DEBUG`），只设在 build 那一条上也不碍测试那一步的键 |
| 不恢复 build，靠提交时的 `check.sh` | 不修（时点后移） | 不修（时点后移） | — |

### 第 3 格：重编核对改读 clippy 的 Checking 行（没打中）

改后重编核对把 clippy 日志的 Checking 行换写成 Compiling、与测试日志并在一起交 `lib_rebuilt_packages.py`：「clippy_checking_as_compiling "$logs/clippy-checking-as-compiling.log" "${clippy_logs[@]}"」（`scripts/gate/merge-and-verify.sh:2853`）。核对是按包取并集：任一份日志里有这个包的那一行就算重编了。

能放过的形状（推的）：同一个编译目录里，测试那一趟的产物（代码生成那一侧）被 cargo 判新鲜、没重编，而 clippy 那一趟的检查产物（另一套指纹）是旧的、重查了，于是 Checking 行替测试那一趟作了保。改前同样的事也会发生在 build 那一侧（并集里 build.log 有 Compiling 就过），只是 build 的产物与测试共用依赖库那一层，比 clippy 那一套更难与测试错开。

够不着的原因：产物要比源码「新」才会被判新鲜，而快照每趟新拷、修改时刻是此刻（文件头「快照」），且快照路径 `<日志目录>/repo` 每趟不同，路径包的产物名随路径变，旧产物对不上；要错开只能是另一趟在这一趟拷完之后往同一个编译目录里编同一棵快照，这棵快照只归这一趟。调用方给 `CARGO_TARGET_DIR` 时也是这样（`--skip-apply` 同样另拷一棵新快照）。试过：调用方固定 `MERGE_AND_VERIFY_LOG_DIRECTORY` 与 `CARGO_TARGET_DIR` 连跑两趟（第二趟快照路径与第一趟相同，但第一趟收尾删了快照、第二趟新拷的源码比产物新，照重编）；验证段第 2 步在留下的快照上交的命令（编的是那棵快照，不再起合入后验证核这一趟）。都够不着。

## 挑测试目标认 `[[test]]` / `[[bin]]` 段

被判的那几句：「包的 Cargo.toml（包目录下的，不是工作区根的）：改动只落在 [[test]] / [[bin]] 段里的，新加、改了、删掉的每个段落到它点名的目标」（`scripts/gate/merge-and-verify.sh:98`）；「写了 required-features 的段记「要特性」同上；段删掉而根文件不再是测试目标（或段写了 test = false）的报一句、不跑」（`scripts/gate/merge-and-verify.sh:100`）。落法只取段自己点名的根文件：「roots += [found for found in entry_root_paths(directory, package, section, entry) if found not in roots]」（`scripts/gate/merge-and-verify.sh:2051`）。改前包的 Cargo.toml 有任何改动都退 3（基线文件头那一句「包的 Cargo.toml、build.rs、tests/ 顶层的数据文件这类」，见草稿目录 `mv.diff` 第 80 行的删除行）。

### 第 1 格：只改 `[[bin]]` 段，起这个 bin 的集成测试没挑（打中）

cargo 编集成测试时把同包每个 bin 的路径交给它（`env!("CARGO_BIN_EXE_<名>")`），集成测试起的就是改了的那个 bin。段只落到 `--bin <名>`（bin 自己的单元测试），读这个 bin 的集成测试不在落点里。

走法（模型 `alpha` 那一组，`write_alpha_tree`、`manifest_model`）：

1. 包 `alpha`：`[[bin]] name = "tool" path = "src/bin/tool.rs"`（打 `hello`）；`tests/cli.rs` 起 `env!("CARGO_BIN_EXE_tool")`、断言输出是 `hello`；另有一份已在树里的 `bins/tool_next.rs`（打 `changed`）。
2. 补丁只改 `crates/alpha/Cargo.toml`：`path` 换成 `bins/tool_next.rs`。
3. 被判程序（从被判脚本里原样取出的那一段 python，`targets` 那一样）：段 `tool` 改了，新旧两侧根文件都落；新侧落到 `--bin tool`，旧侧 `src/bin/tool.rs` 已不是目标、不落；`landed` 为真，不报 REFUSE、不报 NOTE。预期 `alpha_items_binpath` 的 ITEM 恰好是 `alpha:bins/tool_next.rs|--bin tool`。
4. 该红而没红的那一句：`cargo test -p alpha --test cli` 改前退 0（`alpha_before_cli_test`）、改后退非 0（`alpha_binpath_cli_test`，`tool_prints_hello` 断言 `changed` 不等于 `hello`）；合入后验证只跑 `--bin tool`（`alpha_binpath_picked_test` 退 0），整条退 0。改前同一个补丁退 3。

本仓的同一个形状：`crates/singlefs-unit-graph/Cargo.toml` 有一段 `[[bin]]`，「path = "src/bin/unit_graph.rs"」（`crates/singlefs-unit-graph/Cargo.toml:19`）；两个集成测试起它：「Command::new(env!("CARGO_BIN_EXE_unit-graph"))」（`crates/singlefs-unit-graph/tests/unit_graph_command_line_prints_one_item_per_line.rs:11`）、「let output = Command::new(env!("CARGO_BIN_EXE_unit-graph"))」（`crates/singlefs-unit-graph/tests/strip_verus_specifications_from_judging_unit_keys.rs:486`）。只改这一段 `path`（或 `edition`、`harness` 这类改 bin 行为的键）的补丁，合入后验证只跑 `--bin unit-graph`、退 0。

同一个病根早就有：补丁直接改 `src/bin/*.rs` 的正文，照「测试目标根文件」那一条也只落到那个 bin，读它的集成测试同样不挑（我按 `roots_by_location` 读的，没跑）。这一轮把同一个缺口从「源码」扩到了「清单段」：改前清单段一改就退 3，改后放成挑了目标。

四句：分辨臂（改前退 3、改后退 0，同一补丁）；系统看得到判别它的东西（包里的集成测试源码在快照里，`CARGO_BIN_EXE_<名>` 是字面量，`singlefs-unit-graph` 自己的 `rust_items.rs` 就按这个前缀认）；满足的是背景材料「问题」第 2 条第二个分项「把该退 3 的放成挑了目标」；背景材料没给跑前改法。

### 第 2 格：新加的段按名字顶掉自动认的目标，那份测试从此不跑、脚本不报一句（打中）

cargo 的规矩：清单里写了 `[[test]] name = "beta"`，`tests/beta.rs` 就不再按自动发现认成目标（名字被占了）；仓里照抄的那一份判法就是「if relative not in declared and name not in declared:」（`scripts/gate/rerun-failed-tests.py:267`）。

走法：包 `alpha` 有自动认的 `tests/beta.rs`（`beta_guards_greeting`）与一份不是目标的 `checks/other.rs`；补丁只往 `crates/alpha/Cargo.toml` 加一段 `[[test]] name = "beta" path = "checks/other.rs"`。被判程序：段 `beta` 新加，落到 `checks/other.rs`（`--test beta`），退 0；`tests/beta.rs` 从此不是任何目标，它不在新加、改了、删掉的哪一段里，所以「段删掉…报一句」那一支（`scripts/gate/merge-and-verify.sh:2058` 的 NOTE）走不到，输出里一个字都不提它。预期：`alpha_items_shadow` 的 ITEM 恰好是 `alpha:checks/other.rs|--test beta`，`alpha_items_shadow_mentions_tests_beta count=0`；真 cargo 下 `beta_guards_greeting` 改前在 `cargo test -p alpha -- --list` 里（`alpha_before_list`），改后不在（`alpha_shadow_list`）。`[[bin]] name = "<包名>"` 加一段写了别的 path 的，同样顶掉自动认的 `src/main.rs`。

四句：分辨臂（改前退 3、改后退 0 且不报一句）；看得到（旧侧清单已经读成表，`test_targets_of_package` 对新旧两侧各算一遍、比目标集合就知道少了谁）；满足的分句同上一格；背景材料没给改法。这一格比上一格重：上一格是挑漏，测试还在；这一格是那份测试从工作区的测试集合里没了，提交时 `cargo test --all` 也不再跑它。

我提的改法（只在我的模型上量过、被攻过零轮；下表各格都是推的，模型里没有改法臂）：

| 改法 | 第 1 格 | 第 2 格 |
|---|---|---|
| `[[bin]]` 段有差别时，同包的集成测试与 bench（cargo 交 `CARGO_BIN_EXE_` 的那几类目标）一并挑上 | 推的，待主 agent 跑 | 不修 |
| 新旧两侧各算一遍 `test_targets_of_package`，旧侧是目标、新侧不是（且不是被删掉的那一段自己的根文件）的，退 3 或至少打一句 NOTE | 不修 | 推的，待主 agent 跑 |
| 退回改前：包的 Cargo.toml 一改就退 3 | 修（退 3） | 修（退 3） |

## 删上一次尝试的快照（没打中）

删法：「for record in "$(dirname -- "$current")"/*/"$ATTEMPT_RECORD_NAME"; do」（`scripts/gate/merge-and-verify.sh:921`）逐个读同一层日志目录里的 `task-attempt`，任务号逐字相等、尝试号严格更小、`repo` 是目录不是链接，核过本机没人开着才 `rm -rf`。开跑时序：trap 先装（`scripts/gate/merge-and-verify.sh:5021`），再记、再删（`scripts/gate/merge-and-verify.sh:5026`），都在拷快照之前。

逐个试过的形状与为什么够不着：

- 别的任务：任务号逐字比（不比前缀、不转数）；任务号由服务按任务给，每次尝试只交这一条自己的；分片的各片、只重跑红的命令、验证段第 2 步那几条都是另交的任务、另有任务号。没找到同一任务号落到两条不同任务上的走法（只在服务换库重新编号时会撞，那是服务层的事，没核它会不会发生）。
- 尝试号不小：比较是 `attempt < TASK_SERVER_ATTEMPT`，尝试号的写法只认 `^[1-9][0-9]*$`；同一尝试号的两棵（自证「上一次尝试：别的任务号…」那一格演过）不删。
- 带 KEEP_COPY 留给验证段第 2 步的：文件头写明一并删（「带 MERGE_AND_VERIFY_KEEP_COPY=1 留下的也删（同一任务的重试会另留一棵新的）」那一句，`scripts/gate/merge-and-verify.sh:52` 起）。主 agent 只在最后一次尝试结束后读到快照路径（`task wait` 末行），上一次尝试的快照没有到过第 2 步手里。上一次尝试也可能是整趟跑完才被判重排的（`killed-externally`：earlyoom 只杀了一个测试进程，脚本照跑到底、留着快照），那一棵是完整的，但下一次尝试会另留一棵，主 agent 用的是下一棵。
- 本机还有进程在用：`holders` 核工作目录、根目录、可执行文件、文件描述符与内存映射（「for link, what in (("cwd", "工作目录"), ("root", "根目录"), ("exe", "可执行文件")):」，`scripts/hosts/run_build_directory.py:57`）；上一次尝试的合入后验证主 shell 一直 `cd` 在快照里，本机还活着就一定被认出。
- 上一次尝试还在第二台上跑：脚本文件头自认「核的只是本机的进程」（`scripts/gate/merge-and-verify.sh:53`）。够得着的唯一来路是 `peer-run-failed` 重排（第二台连不上、ssh 被重置）时第二台那一趟还没停、这一次派在本机。第二台的命令是脱离 ssh 起的，停它靠服务事后 `--recover-cleanup`；服务是等清理确认了才重排、还是先重排，我没读到底（核不动），所以记没打中、单列。就算撞上，被删的是那一趟自己的快照，它之后编不过、测不过，往判定库写的是红（下一趟照现判），推不出假绿。

## 收尾守卫（没打中）

守卫：「if [[ "$BASHPID" != "$$" && "${MERGE_AND_VERIFY_BREAK:-}" != "finish-in-forked-shell" ]]; then trap - EXIT; exit "$rc"; fi」（`scripts/gate/merge-and-verify.sh:858`）。要让该删的不删，得有一次本该收尾的 `finish` 跑在 `BASHPID` 不等于 `$$` 的 shell 里，而主 shell 自己那一次不跑。

- 子 shell 平常不跑继承来的 EXIT trap：本机 bash 5.2.21 现跑 `( exit 3 )`、`$(exit 4)`、管道里的 `{ exit 5; }`、`{ …; exit 6; } &` 四种，EXIT trap 都只在主 shell 退出时跑一次（草稿里那一条 `bash -c` 的原样输出：`sub=3`、`cs=4`、`pipe=5`、`bg=6`，末行 `EXIT-trap-in 889121 main=889121`）。所以守卫只在 fork 之后、trap 换回缺省之前那一刻起作用，那一刻跑 `finish` 的子 shell 本来就不该收尾。
- 主 shell 自己收尾：trap 装在主 shell 的顶层（不在函数、管道或子 shell 里），主 shell 里 `BASHPID` 等于 `$$`，守卫不拦。
- 被 INT / TERM 停：服务停尝试单元是 `KillMode=mixed`（`task-server/task_server.py:9411`），TERM 只到单元的主进程，`attempt_runner.py` 再「只发给自己起的那一个子进程」（`task-server/attempt_runner.py:390`）即内存包装；包装「先 systemctl --user stop 自己开的那个 scope 单元（$unit.scope），scope 里的命令随之收到 TERM、停下」（`scripts/hosts/run-with-memory-cap.sh:59`），scope 里每个进程各收一次 TERM。主 shell 停在前台的 `two-logs.sh` 上（`scripts/gate/merge-and-verify.sh:1309`），等它退了跑 `trap 'exit 143' TERM`，再进 `finish`，`BASHPID` 等于 `$$`。

顺带看到、不是这一轮改的（推的，没跑）：`finish` 头一步只清 EXIT trap，INT / TERM 两个 trap 还是 `exit`；`finish` 删编译目录与快照的那几十秒里再来一个 TERM，或 scope 停止时限到了发 KILL，收尾停在半截。服务给单元的收尾时限是「UNIT_STOP_SECONDS = 150」（`task-server/task_server.py:792`）。这条与守卫无关，改前的 `finish` 一样。

## `[profile.dev] debug = "line-tables-only"`（没打中，派发「关的是哪几条」第 2 条也在这里答）

改动只有根 `Cargo.toml` 新加的一段，`debug = "line-tables-only"` 在第 32 行（第 25 行是 `[profile.release]` 原有的同一句）。逐个查了谁读根清单的 `[profile]`、读进去做什么：

| 读的地方 | 做什么 | 这一改的影响 |
|---|---|---|
| 准入与复用键 `scripts/gate/admission.py` | 「PROFILE_SETTINGS_LEFT_OUT = frozenset(("debug", "incremental"))」（`scripts/gate/admission.py:868`）；清单里的档只在 `profile_lines` 摊开那一档：「flattened_setting_lines(f"profile.{profile}", keyed_profile_settings((sections.get("profile") or {}).get(profile) or {}), out)」（`scripts/gate/admission.py:1627`） | `[profile.dev]` 去掉 `debug` 后是空表，摊开一行都没有，与改前没有这一段相同：键不变（推的，按代码读的，没跑） |
| 变异单元 `scripts/mutation/lib_mutation_units.py` | 「PROFILE_SETTINGS_NOT_AFFECTING_OUTPUT = frozenset(("debug", "incremental"))」（`scripts/mutation/lib_mutation_units.py:103`）；`flattened` 对空表交空列表 | 同上，键不变（推的） |
| 验证包构建脚本与 `unit-graph --toolchain-of-the-build` | 「`debug`（调试信息）不改编出来的代码跑出的结果，不进键」（`crates/singlefs-unit-graph/src/build_toolchain.rs:17`）；只读 `PROFILE_SETTINGS_READ` 那七个键 | 工具链行不变（推的）；根清单在验证包构建脚本的被监视文件里，改了会重跑构建脚本、重编验证包，是时间不是结局 |
| 广谱变异 `scripts/mutation/mutation-prune.py` | 「lines.append(f"profile {canonical(parsed.get('profile', {}))}")」（`scripts/mutation/mutation-prune.py:1049`），整张 `[profile]` 原样进全局键 | 键全变、广谱变异判定全部作废（背景材料已列，不另算） |
| `crates/` 下的测试与构建脚本 | grep `Backtrace|backtrace|addr2line|gimli|RUST_BACKTRACE|debuginfo|\.debug_` 在 `crates/` 的 `.rs`、`.toml` 里 10 处命中，全是 `Formatter::debug_struct`；构建脚本读 `PROFILE`（`dev` / `release`），不读 `DEBUG` | 没有一处读回溯或调试信息，结局不变 |

搜过的范围与命令（都是这一轮现跑的）：`grep -rlnE '"profile"|\[profile|profile\.dev|line-tables-only|debuginfo' scripts .claude/gate.d task-server prune-server/src`（逐份看了命中的十一份脚本里相关的行，`scripts/experiment-apparatus/` 下是装置自己的清单与编译目录，不读仓根清单）、`grep -rnE 'Backtrace|backtrace|addr2line|gimli|RUST_BACKTRACE|debuginfo|\.debug_' crates --include=*.rs --include=*.toml`、`grep -n '"DEBUG"\|"PROFILE"\|rerun-if-changed' crates/*/build.rs`。`.claude/gate.d/*.sh` 里 `profile` 那一组词 0 命中。

结论：在上面这几处范围里，没有一道判定的结局或复用键因为这一段改变，只有广谱变异的全局键整张变（已知）。推翻条件：哪份测试或门禁读 `.debug_info` / 变量与类型的调试信息、或拿整份根 `Cargo.toml` 的字节摘要进键（不是去掉 `debug` 后的生效值）；`.claude/gate.d/` 与 `scripts/` 之外（`.claude/singlefs-ai-sop/`、`gpu-verification-server/`）我没搜。

## 没打中的形状

挑测试目标（清单段）这一块，按代码读过、都照文件头退 3 或照规矩落了的：

- 段外的键：`manifest_target_changes` 按顶层键逐个比，`[package]`（含 `autotests`、`autobins`、`edition`、`metadata`）、`[lib]`、`[features]`、各类依赖、`[target.…]`、`[lints]`、`[[example]]`、`[[bench]]`、`cargo-features` 都是 `test`、`bin` 之外的顶层键，有差别就退 3。
- name 缺、不是字符串或重名：两侧各判一次，退 3。
- 旧侧倒不掉：在临时 git 仓里照给的次序倒着逐份 `git apply --reverse --include=<路径>`，倒不掉、读不了退 3；两份补丁先加后删同一个依赖，净差为零，按净差判（这是对的）。
- 新包（补丁新建包的 Cargo.toml）：旧侧是空串、读成空表，`[package]` 有差别，退 3；删包：文件不在，`package_of` 落不进任何包，退 3。
- 缺省路径：`[[bin]]` 与包同名时 `src/main.rs`、`src/bin/<名>.rs`、`src/bin/<名>/main.rs` 树里在的全交，多挑不漏挑。反方向有一处误拒（推的，没跑，不在本攻击面）：`[[test]] name = "x"` 不写 path、根文件是 `tests/x/main.rs` 时，cargo 认它，`rerun-failed-tests.py` 列目标时只认 `tests/<名>.rs`，于是被判程序落不到、退 3。
- `test = false` 写进改了的段：报一句不跑（文件头许可的那一支）。

去掉 build 这一块另试过、够不着的：只在 `debug_assertions` 下才编的代码（clippy 与 build 都是 dev 档，同一套 cfg）；构建脚本与过程宏（检查时也编、也链、也跑）；只挂特性的目标（改前的 build 不带特性，同样不编它们）；产品 GPU 包（改前靠 build `-p` 依赖方编到，改后靠 clippy 的 Checking 行过重编核对，两边都没证明验证包那一步里门禁自己那一趟编的是新的，持平）。

取样范围：两个模型各三棵玩具树、四份补丁；没有扫补丁的别的写法（例如同时改 `[[bin]]` 段与 bin 源码、`[[test]]` 改名后登记表里的 `test=` 跟不跟），没有扫别的 cargo 版本。

## 这条腿自己的限度

- 四处打中的读数都是推的：模型写好、没跑（子 agent 不交任务）。cargo 的两条规矩（检查不单态化、不链接；`[[test]]` 的 name 顶掉同名的自动目标）是我按 cargo / rustc 的行为记的，没在这一版工具链（rustc 1.98.0、cargo 1.98.0，`rustc --version` 现查）上跑过，模型就是用来坐实或推翻它们的。
- 被判的挑目标程序是从冻结脚本里按 here 文档的起止行原样切出来、用文件跑的，与脚本里 `python3 - … <<<"$MERGE_AND_VERIFY_ITEMS_PROGRAM"` 的起法差在 `sys.argv[0]` 一个词（程序只读 `sys.argv[1:]`）；补丁头是照自证 `manifest_patch` 的写法拼的。
- 跨台那一格（上一次尝试还在第二台跑）没读到服务重排与第二台清理的先后，记没打中，是核不动，不是核过。
- `[profile.dev]` 的搜索没进 `.claude/singlefs-ai-sop/` 与 `gpu-verification-server/`。
- 背景材料「问题」第 2 条列的五块都答了；这条腿只有一次抽样，没打中的三块按 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」只算一次。

## 要主 agent 跑的命令

1. 模型整份复跑（判决要引的数都从这一趟取）：

   `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/merge-verify-disk-governance-r1-opus -- bash /home/fy5090/code/singlefs/research/prompts/merge-verify-disk-governance-r1-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/merge-verify-disk-governance-r1-opus/model-run /home/fy5090/code/singlefs/research/prompts/merge-verify-disk-governance-r1-opus-model`

   - 在哪个目录起：`/home/fy5090/.local/share/singlefs/work/merge-verify-disk-governance-r1-opus`（submit 的 `--cwd`）。
   - 跑完看：末行 `TASK_SERVER_RESULT` 的 `type=succeeded`、`exit=0`；`task-server/task log <号> --all` 里 `name=summary matched=22 mismatched=0`。`mismatched` 不是 0 的，看 `verdict=mismatched` 那几行：`poly_after-*_clippy` 退非 0 推翻「去掉 build」那两格；`poly_after-*_build` 退 0 同样推翻；`alpha_items_binpath` / `alpha_items_shadow` 的 `items=` 不是预期那一项，或 `refusals=` 不是 0，推翻「挑测试目标」对应那一格；`alpha_binpath_cli_test` 退 0 或 `alpha_shadow_list` 的 `hits=` 不是 0，推翻 cargo 那半句。`exit=2` 且日志点名 sha256 的，是被判脚本换过了，不是模型的结论。
   - 跑完删运行目录 `/home/fy5090/.local/share/singlefs/work/merge-verify-disk-governance-r1-opus/model-run`（里面每棵玩具树各一个编译目录，在 `target` 底下，或在服务给的 `CARGO_TARGET_DIR` 底下）。

## 没做什么

- 不判正推那几格；不替主 agent 采纳；没往统一任务服务交任务，判决要引的数没复跑（命令在上一节）。
- 没在副本上改被判脚本、没实现我提的改法，改法表里每一格都是推的。
- 草稿目录里只有 diff 与切出的文本（`mv.diff`、`main-agent.diff`、`skill.diff`、`runscope.diff`、`mvtail.txt`），没有仓副本、编译目录或库拷贝，交回前不用删；`model-run` 由主 agent 那一趟建、主 agent 删。

## 引文核对

`python3 scripts/agents/cite-check.py research/prompts/merge-verify-disk-governance-r1-opus-output.md` 退 0，末行原样：

```
  ✓ 核了 23 处引文、10 处裸行号，对上 23 处，没判 10 处（其中裸行号没有引文可比 10 处；逐处列在上面）
```

没判的 10 处多数是只写了文件与行号、没接引文的位置；另有几处我写了引文而脚本没认成引文（`scripts/gate/merge-and-verify.sh` 第 7、70、100 行那几句），原文已用 `grep -nF` 逐字核过、各命中 1 次。
