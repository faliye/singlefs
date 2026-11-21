# 拆分换上的脚本与服务治理文件轮第一轮：攻方腿报告（split-governance-scripts-services-r1，2026-11-20）

攻击面：正文「要腿判的」第 1 到 4 问。正文「实现今天的样子」最后一条列的两处（算键闭包跨不过拆出的模块、break-switch 三格把部分文件当独立脚本判）不重复报；下面每个打中都写明它与那两处是不是同一件。

## 各格判定一览

| 问 | 判定 | 打中 | 一句话 |
|---|---|---|---|
| 1 挪动之外行为变没变 | 不成立（打中） | 打中一、二、三 | 三处「拆前生效、拆后静默失效或断言炸」的行为，都不是闭包拒算那一件 |
| 2 拆分带出的同类缺陷有没有漏网 | 不成立（打中） | 打中一、二、三、四 | 两份「给入口模块换属性」、一份「按路径读入口原文找一行」漏网；task-server 五个混入类没打中 |
| 3 替身换没换在被测代码读的绑定上 | 不成立（打中） | 打中一（自证一格测空一半）、打中四（计数测空一部分，推的） | 同一形状：替身换在入口上，读者在拆出的模块里按自己的全局名读 |
| 4 子进程、`__file__`、`sys.path`、两个 Rust 服务 | python 与 shell 的起子进程、`__file__`、`sys.path`：抽查的族里成立；按路径读入口原文：不成立（打中三）；Rust 服务 `control` 与 `#[path]`：成立；服务没重编：推不出哪一格会不符（没找到比源码与在跑二进制的格） | 打中三 | 见第 4 问一节 |

四个打中的形状都在跨族那一层：被改的是 `scripts/gate/lib_function_units.py` 一族（拆成入口加库内模块），失效的是别的族里给它的入口模块换属性、按路径读它的入口原文的那几处。各族自己拆前拆后的自证对照照不到这一层（被改族的自证不起那几处；起那几处的族自证拆前就有别的红，或只核「大于 0」）。

## 要主 agent 跑的命令

1. 探针（两臂：HEAD 里拆前的 `lib_function_units.py` 一份、主工作区此刻一份；同一棵三行的小 Rust 工作区）：
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/split-governance-scripts-services-r1-opus -- python3 /home/fy5090/.local/share/singlefs/work/split-governance-scripts-services-r1-opus/probe/probe_entry_patch.py --repo /home/fy5090/code/singlefs --work /home/fy5090/.local/share/singlefs/work/split-governance-scripts-services-r1-opus/probe-work`
   - 在哪起：`/home/fy5090/.local/share/singlefs/work/split-governance-scripts-services-r1-opus`
   - 看哪几行：末行 `TASK_SERVER_RESULT` 的 `type=succeeded exit=0`；日志里 `PROBE_DONE` 一行在、没有 `PROBE_ERROR`；再读下面「探针的判读」那张表的六行。
   - 范围声明：这条不是崩溃、门禁、cargo test、合入后验证或实验复跑，我照不要范围声明写；服务要的话照拒绝说明补。
2. 打中二的自证格（可选，判它在主工作区上是不是已经红着；拆前闭包拒算那一件可能先把它挡在前头）：
   - 命令：`task-server/task submit -- python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/mutation/crates-mutation-rows.py --selftest --cells selftest_static_check_parallel_cell`
   - 在哪起：仓根 `/home/fy5090/code/singlefs`
   - 看哪几行：末行 `TASK_SERVER_RESULT` 的 `exit=`；日志里含「单线程那一趟」「并行那一趟」的那一格判没判错（打中二推的是并行那一趟 `{'product-behavior': 0}` 之类，取了 0 份）。

探针文件与草稿工具的 sha256（`sha256sum` 现取）：

```
fa9a85f6953f210ea7e9363a242c4e42a61bbe339b9fc36e750511e49e2ed7da  probe/probe_entry_patch.py
a09b29c3b388eb6ce43ee69ae74e6d6199efe2bf01875500da44a37ae268ea8b  tools/attr_patch_audit.py
3e57263512ed34722d29e6dc0ba8972d85f20bf568aecd169ea8e3fe44d1de89  tools/cross_patch.py
82584b4f41a2870eb2fb8289694877a34313c47009a5d7d9f5c93ef57b263806  tools/def_diff.py
c4b3e100a9a40d026374b8db6b99ea3d22d0d7d285dad41041a6cac22c610524  tools/free_names.py
df1eaf82b35bf3f7d44c307230c4e2cd44ecacd233da8ab701449f51415ad3d6  tools/globals_audit.py
0cd195e4dc898969fb6b7b62b6b26b947c47dccf2caa79d6fcac6f51f10827ed  tools/multi_defined.py
494b0e798bfef996c1b8100b9acedf206f3529eedc7871e2836e59cd2f2e8b38  tools/sh_func_diff.py
51ae0d64fc7878c233421120ecb49c15af2b8210609f5416aeefaa7d7cf49670  tools/run_dd.sh
7d500b6a2222cae6b6afb1e48e287ac90d9f5cc7c6d8e03616453c3d93c12966  tools/run_dd_selftest.sh
```

（路径相对草稿目录 `/home/fy5090/.local/share/singlefs/work/split-governance-scripts-services-r1-opus/`。`tools/` 下几份是只读仓里文件文本的静态分析（`ast`、`symtable`、`difflib`），我直接跑了，不起项目代码；探针要 import 项目代码，交主 agent。这一轮没写崩溃状态模型，没有 `rerun.sh`：这一题的打中靠静态读与一份探针，不靠崩溃枚举。）

### 探针的判读（跑前写死）

| 行 | 拆前（head 臂）推的 | 拆后（split 臂）推的 | 量出相反怎么改结论 |
|---|---|---|---|
| `PROBE_RUSTFILE arm=head` | `patched_calls` 等于 `files_loaded`（1） | — | head 臂也是 0：打中二的「拆前生效」不成立，打中二降成「拆前拆后一样没用上」，撤回「拆分带来的」那半句 |
| `PROBE_RUSTFILE arm=split` | — | `patched_calls=0`、`files_loaded=1` | split 臂不是 0：打中二撤回 |
| `PROBE_RUSTWORKSPACE arm=head` | `counted_loads=1` | — | head 臂是 0：打中四撤回 |
| `PROBE_RUSTWORKSPACE arm=split` | — | `counted_loads=0` | split 臂是 1：打中四撤回 |
| `PROBE_ROW_LINE` 两行 | head 臂 `occurrences=2`（RustFile.row 与 WgslFile.row 同一行字） | split 臂 `occurrences=0` | split 臂不是 0：打中三撤回 |
| `PROBE_MEMO arm=split` | — | `via_entry_hits=1`、`via_rust_closure_module_hits=0`，三个 `*_sees_memo=False` | 经模块调也涨了、或 `*_sees_memo=True`：打中一撤回 |

`PROBE_ROW_LINE` 两个数我已用 `grep -cF` 现核过一次（主工作区入口 0 次，`git show HEAD:scripts/gate/lib_function_units.py` 2 次，命中的是 `scripts/gate/lib_function_units_rust_file.py` 第 474 行与 `scripts/gate/lib_function_units_wgsl.py` 第 58 行），探针里再跑一次是给判决复跑用；其余五行是推的，待主 agent 跑。

## 第 1、2、3 问：打中

### 打中一：test-plan 的「数据文件单元按路径只读一次」换在入口上，拆后够不着读者

被判对象里许可它的那几句：

- 换的地方：`scripts/gate/admission_unit_key.py:1562`：「library.data_file_unit = remembered(library.data_file_unit, lambda root, path: (root, path))」，下一行同样换 `data_directory_unit`（`scripts/gate/admission_unit_key.py:1563`：「library.data_directory_unit = remembered(library.data_directory_unit, lambda root, path: (root, path))」）。`library` 是 `function_units()` 交的 `lib_function_units.py` 入口模块（`scripts/gate/admission_base.py:286` 起的那一个函数，按路径载入入口）。
- 拆后读者不在入口里：`scripts/gate/lib_function_units_rust_closure.py:6`：「from lib_function_units_base import Refusal, UsageError, data_directory_unit, data_file_unit, snippet_of」，调用处 `scripts/gate/lib_function_units_rust_closure.py:356`：「data_units[file] = data_file_unit(workspace.root, file)」；`scripts/gate/lib_function_units_script_closure.py:472`：「units[path] = data_file_unit(self.root, path)」；`scripts/gate/lib_function_units_closure_keys.py:463`：「data_file_unit(root, value) if kind == "file" else data_directory_unit(root, value)」。三份各按自己模块的全局名读，入口上的换法碰不到它们。入口 `scripts/gate/lib_function_units.py` 里没有一处调 `data_file_unit`（`grep -n` 现查，只在第 1602 行的再导出名单里出现）。
- 拆前：`git show HEAD:scripts/gate/lib_function_units.py` 里 `def data_file_unit` 在第 1626 行、`def data_directory_unit` 在第 1649 行，调用在第 5195、5207、5228、5234、14716、14733、15522 行，同一份模块，换模块属性就换到了全部读者。
- 同一个函数里另换的 `manifest_sections`、`registry_dependency_filter` 是 `global` 换本模块的名字（`scripts/gate/admission_unit_key.py:1543`：「global manifest_sections, registry_dependency_filter」），读者都在 `admission_unit_key.py` 里（`grep -n` 现查：第 454、551、552、578、617、635、1383、1392、1513、1520 行），拆后照旧生效。打中的只是数据文件那一半。

能照喂的输入：探针 `PROBE_MEMO` 那一行（命令在上面）。不起 test-plan，只把 `test_plan_memoized_readers()` 换上之后，经入口调两次与经 `lib_function_units_rust_closure` 模块调两次，各数 `TEST_PLAN_SHARED["reader_memo_hits"]`（计数在 `scripts/gate/admission_unit_key.py:1554`：「TEST_PLAN_SHARED["reader_memo_hits"] = TEST_PLAN_SHARED.get("reader_memo_hits", 0) + 1」）。推的：经入口涨 1、经模块涨 0。

拆前拆后判得不一样的地方：

- test-plan 里每条测试函数的闭包各把数据文件单元现读一遍（拆前按路径只读一次）。答案相同、多花内存与时间；`reader-memo-hits=` 那一段（`scripts/gate/admission_commands.py:941`）的数变小。量的只有「拆后读者不在入口里」，涨多少没量。
- 自证测空一半（第 3 问）：`scripts/gate/admission_selftest_support.py:614`：「selftest.expect("test-plan：一趟里清单与数据文件按路径只读一次", recorded(single, "reader-memo-hits") > 0,」只核大于 0，`manifest_sections` 那一半照旧命中就过，数据文件那一半失效它看不出。文件头开关说明 `scripts/gate/admission.py:475`：「ADMISSION_BREAK=test-plan-file-readers-unmemoized  test-plan 里 Cargo.toml、cargo 缓存里的清单与数据文件单元每条测试函数各读一遍」——不开开关，数据文件单元此刻已经每条各读一遍。

四句：

1. 分不分辨臂：这一题没有臂，按「拆前 / 拆后」两态判。分辨：拆前生效（同一模块）、拆后不生效，探针的两种调法各给一个数。
2. 被判的系统当时看不看得到判别它的东西：看得到。`test_plan_memoized_readers` 换之前能查 `sys.modules` 里哪几份模块持有 `data_file_unit` 的绑定；它只换了入口那一份。
3. 满足的是哪个分句：正文第 2 问「入口再导出的是拷贝而不是同一个对象、`from x import NAME` 之后……只改了本模块的绑定」的同形——这里是外面换入口模块的属性，读者是 `from lib_function_units_base import data_file_unit` 那几份，`lib_function_units_base` 里那一份也没换。第 3 问「换在被测代码真正读的那个绑定上」：没换在。
4. 跑前条款给的改法：正文没给改法。它与「算键闭包跨不过拆出的模块」不是同一件：那一件修好（闭包能跨模块收单元），这几份模块照旧各读各的绑定，打中一照中。

### 打中二：crates 变异 static-check 的「建工作区按文件切片并行」换在入口上，拆后一份都取不到

被判对象里许可它的那几句：

- 换的地方：`scripts/mutation/crates_mutation_rows_plan.py:552`：「lfu.RustFile = rust_file_from_slices」，紧接着 `scripts/mutation/crates_mutation_rows_plan.py:554`：「workspace = lfu.RustWorkspace(inner.root, view)」；取了几份记在 `scripts/mutation/crates_mutation_rows_plan.py:562`：「prebuilt_workspace.files[view] = len(taken)」。`lfu` 是 `mutation_units.function_units()`，即按路径载入的 `lib_function_units.py` 入口。
- 拆后 `RustWorkspace` 建文件用的是它自己模块里的 `RustFile`：`scripts/gate/lib_function_units_rust_workspace.py:555`：「self.files[path] = self.product_view_file(RustFile(self.root, path, text, directory), text, directory)」，名字由同一份文件第 16 行起的 `from lib_function_units_rust_file import (` 引进。入口上换的 `RustFile` 它读不到。
- 拆前：`git show HEAD:scripts/gate/lib_function_units.py` 里 `class RustFile` 在第 2664 行、`class RustWorkspace` 在第 4148 行、建文件那一行在第 4314 行，同一份模块；`git show HEAD:scripts/mutation/crates-mutation-rows.py` 第 6982 行就是同一句 `lfu.RustFile = rust_file_from_slices`。

拆前拆后判得不一样的地方：

- 生产路径（这是 crates 变异门禁 static-check 的真跑法，不只自证）：fork 出来的子进程照旧按文件切好每份 `.rs`，本进程建工作区时一份都不取、全部现切一遍。答案相同，切片那一趟的活全白做，static-check 的挂钟多出一整趟切文件（推的，没量多少）。`prebuilt_workspace.files` 记 0。
- 自证那一格：`scripts/mutation/crates_mutation_rows_commands.py:672`：「not single_prebuilt.get(lfu.PRODUCT_BEHAVIOR) and (parallel_prebuilt.get(lfu.PRODUCT_BEHAVIOR) or 0) >= mutation_units.PARALLEL_MINIMUM,」要并行那一趟取到至少 `PARALLEL_MINIMUM`（8，`scripts/mutation/lib_mutation_units.py:109`）份；拆后取到 0，这一格该红（推的，待主 agent 跑上面第 2 条；它可能被闭包拒算那一件先挡住，挡住时这一格的红看不出来）。弄坏开关 `workspace-files-unsliced` 要它红的那一格，不开开关也红，开关测空。

能照喂的输入：探针 `PROBE_RUSTFILE` 两行。推的：head 臂替身被调 1 次，split 臂 0 次。

四句：

1. 分辨：拆前替身被调、拆后不被调，探针两臂各给一个数。
2. 看不看得到：`prebuilt_workspace` 换之前看得到 `lib_function_units_rust_workspace` 在 `sys.modules` 里；它只换了入口。
3. 满足的分句：第 2 问「`from x import NAME` 之后……只改了本模块的绑定」的同形（外面换入口属性，读者在 `from lib_function_units_rust_file import` 那一份里）；第 1 问「判法的分支……」里 static-check 走的分支没变、取没取到变了。
4. 与闭包拒算那一件不同：那一件修好，这里照中。

### 打中三：crates 变异判法单元那一格按路径改入口原文里的一行，那一行拆后不在入口里

被判对象里许可它的那几句：

- `scripts/mutation/crates_mutation_rows_units.py:1514`：「units_file = os.path.join(judging_root, "scripts", "gate", "lib_function_units.py")」，读整份原文后 `scripts/mutation/crates_mutation_rows_units.py:1517`：「assert units_text.count(row_line) >= 1, units_file」，要找的是 `RustFile.row` 那一行函数体。
- 那一行拆后在 `scripts/gate/lib_function_units_rust_file.py:474`：「return snippet_of(self.source_rows[line - 1]) if 0 < line <= len(self.source_rows) else ""」，入口里 0 次（`grep -cF` 现核）。拆前 HEAD 入口里 2 次。
- 同一格数闭包里有几个单元落在入口那份文件上：`scripts/mutation/crates_mutation_rows_units.py:1530`：「function_units_count = sum(reference.file == "scripts/gate/lib_function_units.py" for reference in function_units_references)」。就算断言过了，改的也不是被数的那份文件。

拆前拆后判得不一样的地方：这一格（`selftest_judging_unit_function_units_cell`，开关 `judging-function-units-excluded` 点名的那一格，`scripts/mutation/crates_mutation_rows_driver.py:322`）拆前改入口、指纹变、数得到入口的单元；拆后在断言那一行抛 `AssertionError`。此刻它被 `selftest_judging_unit_start` 在闭包拒算时先交 None 挡在前头（`scripts/mutation/crates_mutation_rows_units.py:1348` 起那一段），所以今天看到的是闭包拒算那一格的红；那一件修好之后，这一格照样炸。两件不是同一件。

能照喂的输入：探针 `PROBE_ROW_LINE` 两行（数已现核：split 0、head 2）。

四句：分辨（拆前找得到、拆后找不到）；系统看得到（它可以按函数单元找 `RustFile.row` 落在哪份文件，没这样做）；满足第 2 问「判法只读入口原文」与第 4 问「入口按 `__file__` 找同目录文件」同类的分句（这里是按写死的入口路径读原文）；正文没给改法。

### 打中四（推的，测空一部分）：verus 证明项自证数「Rust 工作区载入了几次」，计数子类换在入口上

- 换的地方：`scripts/gate/verus_proof_check_selftest_fixtures.py:751`：「module.RustWorkspace = CountingWorkspace」，`module` 是 `scripts/crash/lib_verdict_library.py` 的 `function_units()` 交的入口模块。
- 计得到的：`scripts/crash/lib_verdict_library.py:211`：「self._workspace = function_units().RustWorkspace(self.root)」，按入口属性取，照旧计得到。
- 计不到的：库内模块里按自己全局名建的，如 `scripts/gate/lib_function_units_closure_keys.py:120`：「workspace = RustWorkspace(root)」、`scripts/gate/lib_function_units_script_closure.py:977`：「workspace = RustWorkspace(root, options.judgment_kind, options.product_dump)」。拆前这些也在入口模块里、也被计。
- 那一格要恰好 1 次：`scripts/gate/verus_proof_check_selftest.py:1176`：「multi_listed_exit == 0 and all(multi_states.get(item) == "键相同" for item in multi_proof_items) and multi_listed_loads == 1」。拆后若有一条路径在库内模块里另建了工作区，这一格看不见、照绿。今天这一格走的路径是不是只经 `FunctionUnitIndex.workspace()` 我没量，所以只写「测空一部分」、推的。探针 `PROBE_RUSTWORKSPACE` 两行量的是同一形状（经 `rust_units` 建的那一次，推的：head 1、split 0）。
- 四句：分辨（拆前计得到库内建的、拆后计不到）；系统看得到（可以在 `lib_function_units_rust_workspace` 上换）；满足第 3 问分句；正文没给改法。

### 同一形状没打中的几处（现查过，替身换对了）

- `scripts/gate/hook_swap_selftest.py:1492`：「hook_swap_swap.scratch_lock_content = replacement」：读者 `claim_record_removal` 在 `hook_swap_swap.py` 里（正推腿那一题，这里只记它不在打中里）。
- `scripts/gate/lib_function_units_selftest_closure_rules.py:265`：「lib_function_units_script_closure.CLOSURE_ROUND_LIMIT = 1」：读者只在 `scripts/gate/lib_function_units_script_closure.py` 第 230、232 行。
- `scripts/mutation/crates_mutation_rows_units.py:466`：「lfu.closure_of = collected_closure_of」：调用方 `scripts/mutation/lib_mutation_units.py` 第 951 行按 `lfu.closure_of(` 取入口属性，换得到。
- `scripts/mutation/tests/test_audit_regressions.py:121`：「mock.patch.object(rows_units, 'key_toolchain_lines', return_value=[]):」：`KeyContext` 在 `crates_mutation_rows_units.py` 第 769 行按本模块名读，换得到。
- `scripts/agents/tests/test_agents.py:295`：「with mock.patch.object(alerts, 'progress_file_of', return_value=str(self.root / 'missing')):」：换在 `agent_watch_alerts` 上，读者同模块。
- `scripts/gate/lib_function_units_selftest_real_repo.py:228` 换 `module.judge_inputs_unchanged`：`admit_experiment` 留在 `admission.py` 里按本模块名读（`scripts/gate/admission.py:924`：「unchanged, description = judge_inputs_unchanged(root, key, unit_key.key)」）。
- task-server：`task-server/tests/test_cleanup_protocol.py:65`：「stack.enter_context(patch.object(module, name, replacement))」与 `task-server/task_server_selftest_memory.py:45` 同形，每份持有那个名字的服务模块都换；`patch.object(control.lib, "read_configuration", …)` 的读者是 `control` 自己按 `lib.read_configuration()` 取（`task-server/control:187` 等），换得到。

## 第 2 问另查：模块级可变状态存两份、`global` 改写

- `global` 改写共 9 处（`grep -n '^\s*global '` 在被判的 python 里现数）：`agent_watch_alerts.py` 的 `ACK_UPDATE_PAUSE_SECONDS`、`admission_unit_key.py` 那一处、`broad_mutation_judging.py` 的 `TEST_MODE` 与 `PRUNE_MODULE`、`apply-writer-patch.py` 的 `CRASH_VERIFY_HINT_PATCHES`、`prune_server_client_selftest_service.py` 的 `build_server_binary`、`broad_mutation_library.py` 的 `REUSE_CHECKS_MODULE`、两份自证夹具。逐个查了读者：除 `admission_unit_key.py` 那一处（读者同模块、生效）外，读者都与改写在同一份模块里，没有别的模块经 `from … import` 持一份旧绑定（静态工具 `tools/globals_audit.py`，三种 star import 另按 `tools/free_names.py` 的认法核过）。
- 同一个可变对象在两份模块各定义一份：task-server 服务加库对拆前备份逐个比顶层名字（`tools/def_diff.py`），两份模块都定义的只有 `STALL_AT_CAP_FRACTION`、`SCRIPT_DIRECTORY`、`main` 等，拆前两份原文件就各有一份（`tools/multi_defined.py` 对拆前备份现跑）；`task_server_selftest_world.py` 里 `read_text`、`attention_lines` 各定义两次、后一份盖前一份，拆前那一份自证文件里也是两次、次序相同。没打中。
- task-server 五个混入类：`SchedulerLifecycle` 等五个类都没有基类、没有类属性（`ast` 现查），方法名在五个类与 `Scheduler` 之间没有重名（`tools/def_diff.py` 的 DUP 一项为空），五份里没有 `super()`；`Scheduler` 的方法集合与拆前备份逐个同名、同文（拆前备份里的方法在拆后都找得到、正文相同，CHANGED 只有 `Scheduler` 的基类那一行）。没打中。
- 函数里引到、而所在模块没有绑定的全局名（拆后在那条路径上就是 `NameError`）：`tools/free_names.py`（`symtable` 判作用域）对被判的 python 逐份跑，命中都在 `lib_function_units_selftest_*`、`crates_mutation_rows_*`、`broad_mutation_selftest_a.py` 三族，都是自证夹具在开跑时往这几份模块的命名空间里放的名字（`scripts/mutation/crates_mutation_rows_driver.py` 的 `load_selftest_fixtures`、`scripts/gate/lib_function_units_selftest_run.py` 第 574 至 597 行、`scripts/mutation/broad_mutation_selftest_b.py:51`），静态判不了，没算打中。task-server、agent-watch、hook-swap、admission、verus、apply-writer-patch、prune 自证、crash、hosts、e165 各份 0 处。

## 第 4 问：子进程与路径

python 入口拆出模块之后起自己、找同目录文件（抽查的都对，没打中）：

- 在拆出的模块里起入口的，都换成了指向入口的常量，不用模块自己的 `__file__`：`scripts/gate/hook_swap_base.py:568`：「ENTRY_SCRIPT = os.path.join(SCRIPT_DIRECTORY, "hook-swap.py")」；`scripts/gate/verus_proof_check_base.py:860` 同形；`scripts/mutation/crates_mutation_rows_base.py:62`：「MODULE_PATH = os.path.realpath(os.path.join(SCRIPT_DIRECTORY, "crates-mutation-rows.py"))」；`scripts/mutation/broad_mutation_base.py:37`：「SELF_PATH = os.path.join(SCRIPT_DIRECTORY, "broad-mutation.py")」；`scripts/prune-server/prune_server_client_selftest_service.py:22`、`scripts/agents/agent_watch_transcripts.py:22`、`scripts/gate/lib_crates_split_selftest.py:26` 同形；admission 复用哨兵的脚本名改读 `ENTRY_SCRIPT_NAME`（`scripts/gate/admission_base.py:896`）。被判 python 里所有 `[sys.executable, <路径>, …]` 起法（不含 `-c` 的 156 行，`grep` 现数）逐行看了路径常量的来处，没有落到拆出模块自己的 `__file__` 上的。
- e165 内联 python 经 `python3 -` 起，`sys.path[0]` 是空串，入口自己插 `SCRIPT_DIRECTORY`（`scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh:330`：「sys.path.insert(0, SCRIPT_DIRECTORY)」），弄坏开关集合 `BREAK` 定义在 `e165_replay_mutation_kill_bounds_variant_judgement.py`、入口照环境 `update`，三份模块的模块级语句里没有读 `BREAK` 的（`ast` 现查），不会在 import 时就按空集合算定。prove-red 的内联 python 同样插目录再 import。
- `inspect.getsource` 取函数正文送到第二台跑的两处（`task-server/task_server_lib_readings.py` 第 277、397 行）：两个函数自带 `os_module` 参数、不引模块级名字，挪模块不影响。

shell 拆成被 source 的库（抽查的对，没打中）：

- `crash-point-replay.sh`、`multi-host-run.sh`、`run-with-memory-cap.sh` 三份：拆进 `*-selftest.sh` 的函数，入口里只有 `--selftest` 那一支调（`grep -nwF` 逐个函数名现查：`run_selftest`、`run_selftest_parallel`、`selftest_wheel_segment_choose`、`multi_host_selftest`），都在 `source` 那一行之后；EXIT trap `selftest_cleanup` 只在拆出去的 `run_selftest` 里挂（`scripts/hosts/run-with-memory-cap-selftest.sh:42`：「trap selftest_cleanup EXIT」），平常路径不会碰到没定义的函数。三份与拆前备份逐个函数比（`tools/sh_func_diff.py`），函数正文只在自证读源码的地方改成读两份（`replay_selftest_library`），别的一字不差。
- `merge-and-verify.sh` 四份库在入口里按 `here` 绝对路径 source；库里起入口用 `bash "$0"`，被 source 时 `$0` 是入口。和拆前备份比，多出的是变异行带前像那一件功能改动，不是拆分。
- `replay.sh`：自证按字面读源码的几处改成 `REPLAY_SOURCE_FILES`；按 `REPLAY_ENTRY_SCRIPT` 单读入口的那几处要找的函数（`driver_e162_anchors`、`driver_e182`、`e168_patched_tree`、`driver_e171_s4_comparator`、e171 那一组等十五个）逐个现查都还在 `replay.sh` 里；`.claude/gate.d/qemu-device-streams.sh:476` 用 awk 读的 `E142|@driver_e142` 表行还在 `replay.sh` 第 493 行；`scripts/gate/lib_function_units_selftest_closure_rules.py:303` 按原文找的八段字都还在 `replay.sh` 里（`grep -lF` 逐段现查）。

按路径读入口原文：不成立，见打中三。我只对「按入口文件名读原文」做了全仓 `grep`（61 行，`grep -E` 加读文件的动词现筛），逐行看过；按别的方式拿到入口路径再读的（变量拼出来的）没扫全，写在限度里。

两个 Rust 服务：

- `gpu-verification-server/control` 读的源码：`gpu-verification-server/control:332`：「LIBRARY_PENDING_POOL_SOURCE = os.path.join(SCRIPT_DIRECTORY, "src", "library_pending_pool.rs")」，它要的 `VERSION_MISMATCH_LINE_PREFIX` 与三个格式串都还在根里：`gpu-verification-server/src/library_pending_pool.rs:135`：「pub const VERSION_MISMATCH_LINE_PREFIX: &str = "GPU_VERIFICATION_SERVER_VERSION_MISMATCH";」，格式串在同一份文件第 772、787、807 行。成立。
- 集成测试根：`gpu-verification-server/tests/library_pending_pool_on_a_temporary_library/main.rs:12`：「#[path = "../common/mod.rs"]」，下一行 `mod common;`；五份子模块各在 `main.rs` 第 74 至 78 行声明，各自 `use super::*;`。成立（没编，编不编得过以主 agent 的 clippy 或点名测试为准）。
- 测试全名变了（裸名变成 `<子模块>::<名>`）：仓里点这个目标的登记表只有 `.claude/gate.d/path-moves.tsv` 一行（`grep -rln` 现查，不含 `research/`）；`gpu-verification-server/src/break_switches.rs` 的 `red_test` 表仍写裸名，但全仓没有代码读这张表去起测试（`grep -rn 'red_test'` 现查只有定义与文档注释），没打中。
- 服务没重编重起：两个 `control` 里没有一格比「源码」与「在跑的二进制」（`grep` `getmtime`、`st_mtime`、`sha256` 现查：sha256 只比 `--binary` 给的与装好的那一份）；没有 build.rs。拆分是纯挪动，在跑的旧二进制与源码的行为差只来自同一批里别的功能改动（两个服务另有十几份 `M` 文件，`git status --short` 现查），不归拆分。推不出哪一格的结局会因为「没重编」而与源码不符；能推翻这一句的观测：哪一格自证或门禁读在跑服务的版本键与源码现算的比。

## 自己提的改法（只在我的探针上推过、被攻过零轮）

| 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| 甲：换替身时换在读者所在的模块上——打中一换 `lib_function_units_base`、`_rust_closure`、`_script_closure`、`_closure_keys` 四份的 `data_file_unit` 与 `data_directory_unit`；打中二换 `sys.modules["lib_function_units_rust_workspace"].RustFile`；打中四换 `lib_function_units_rust_workspace.RustWorkspace` 之外，另在建工作区的那几份模块上各换一次 | 打中一、二、四 | 推的，待主 agent 跑 |
| 乙：照 task-server 自证的写法（`test_cleanup_protocol.py` 的 `patch_server`）写一个共用的「把每份持有这个名字的 `lib_function_units*` 模块都换上」，三处都改调它 | 打中一、二、四；以后再拆出模块也罩得住 | 推的，待主 agent 跑 |
| 丙：打中三改成按单元找 `RustFile.row` 落在哪份文件（经 `rust_units` 或闭包的 `reference.file`），改那一份、数那一份 | 打中三 | 推的 |

甲在打中一上只罩今天这四份读者，以后再拆一次又漏；乙罩得住新拆的，但「每份持有这个名字」里会包括只再导出、不读的入口，换了不碍事。两种改法对打中三都不起作用。

## 没打中的形状与取样范围

- `from x import NAME` 之后对 NAME 重新赋值：被判 python 全部跑了 `tools/globals_audit.py`，只命中打中一那一处（且读者同模块、那一半生效）。
- 给入口或别的模块换属性（`X.N = …`、`setattr`、`patch.object`、`patch("m.n")`）：被判的 python 跑 `tools/attr_patch_audit.py`；全仓 `scripts/`、`.claude/`、`task-server/`、两个服务目录下的 python 跑 `tools/cross_patch.py`（64 行命中，逐行看过，除打中一、二、四外都是换在读者模块上、换在对象实例上或名字撞车的假命中）。`.sh` 里内联 python 换属性的只按 `grep` 查了 `library.`、`lfu.`、`module.` 等几种持有者名字。
- 拆前拆后逐个顶层定义比正文（`tools/def_diff.py`）：task-server 三份（对拆前备份）、admission、hook-swap、verus、apply-writer-patch（对拆分会话留的原文）、agent-watch、prune 自证（对拆分会话留的拆前树）。CHANGED 的逐个看了：拆分该做的适配（入口常量、按模块取、延后 import 自证）与同期别的功能改动（变异行前像、复用读法），没有判法分支、默认值、退出码被顺手改的。
- shell 函数逐个比正文（`tools/sh_func_diff.py`）：crash-point-replay、run-with-memory-cap、multi-host-run、merge-and-verify（对拆前留的原文）、replay（对 HEAD；HEAD 与拆前不是同一份，同期改动多，只看了非驱动的几处）。

## 抽了哪几族、哪几族没看

被判文件 202 份，读法分两层：

- 静态工具全量跑过的：被判的全部 `.py`（`tools/globals_audit.py`、`tools/attr_patch_audit.py`、`tools/free_names.py`、`tools/cross_patch.py`），被判的全部 `.sh` 的 `BASH_SOURCE`、`$0`、`source` 行（`grep`）。
- 逐个定义比正文、逐处看过的族：task-server 三份入口与 24 份模块（对 `crates-split-task-server/backup-before-apply/` 里拆前备份）；`scripts/gate` 的 admission、hook-swap、verus-proof-check、apply-writer-patch 四族（对 `crates-split-gate-scripts/originals/`）；agent-watch、prune 自证（对 `crates-split-scripts-misc/base/`）；crash-point-replay、multi-host-run、run-with-memory-cap、merge-and-verify 四份 shell；replay.sh 一族（对 HEAD）；e165 入口与三份模块的 `BREAK` 与 `sys.path`；两个 Rust 服务只看了 `gpu-verification-server/control` 读源码的那几处、集成测试根与 `break_switches.rs` 的 `red_test`。
- 没逐个比正文的（只过了静态工具）：`lib_function_units` 一族 23 份（正文那一族拆前拆后自证已比过、且归闭包那一件在修）；变异三族（`crates_mutation_rows_*`、`broad_mutation_*`、`prove_red_*`：拆分会话没留拆前原文的文件，只留了 sha256，HEAD 又不是拆前那一份）；`lib_crates_split_*` 与 `crates-split-pure-move.py`（新文件，没有拆前）；`scripts/crash` 的 `crash-case-check.py`、`crash-flow-coverage.py`、`lib_crash_point_replay_items.py`、两份 crash-amplification（`M` 的改动是同期功能，不是拆分）；两个服务 33 份 Rust 子模块正文；三份 task-server README 与两份 README.md。
- 正推腿那一题（hook-swap 自证换替身、看门狗前缀表补登、task-server 再导出覆盖）我没判，只在第 3 问里顺带记了 hook-swap 那一处读者在哪。

被判清单的 sha256 与主工作区此刻比（`sha256sum -c` 现跑）有 5 份对不上：`scripts/gate/README.md`、`scripts/mutation/broad_mutation_selftest_a.py`、`scripts/mutation/broad_mutation_selftest_b.py`、`scripts/mutation/crates_mutation_rows_commands.py`、`scripts/mutation/tests/test_audit_regressions.py`。其中 `crates_mutation_rows_commands.py` 暂存区那一份与清单相同（打中二引的第 672 行在暂存区与工作区都在）；另四份暂存区与工作区都不是清单那一版，开工之后有人在改。我读的是工作区此刻那一版，打中一到四引的行都不在这四份里。

## 这条腿自己的限度

- 四个打中的「拆前生效」都是读 `git show HEAD:…` 推的：`lib_function_units.py` 拆前是单文件、换模块属性罩得住全部读者，这一句我按 HEAD 原文与行号核了；HEAD 是不是拆前那一刻的原文，我没另核（拆分会话留在工作根的那几份原文里没有 `lib_function_units.py`）。探针 head 臂就是拿这一份跑。
- 打中一、二的代价（多读几遍、多切一趟）没量；打中二那一格自证此刻红不红没跑；打中四今天走的路径有没有库内另建工作区没量。
- `tools/free_names.py` 判不了运行时往模块命名空间里塞名字的写法（自证夹具那三族），也判不了 `getattr(module, 变量)`；`tools/cross_patch.py` 只认持有者是名字或属性链、属性名是字面的写法。
- 按入口路径读原文那一类，我只按入口文件名全仓 `grep`；路径由变量拼出再读的没扫全，可能还有打中三的同类。
- shell 那几份只比了函数正文与顶层语句；`declare` 在函数里 source 变局部、`set -e` 与 trap 在被 source 时的差别，我只在那三份 `*-selftest.sh` 上看了顶层赋值都是普通赋值（不是 `declare`），别的库没逐个看。
- 第一轮、一次抽样：没打中的那几格（task-server 混入类、子进程路径、Rust 服务）按 `.claude/rules/three-way-inference.md@一条腿只抽一次样不算一次观测——否定结论尤其不算` 不算「没打中」的定论。

## 没做什么

- 不判正推腿那几格（第 5 到 7 问）；不替主 agent 采纳；没往统一任务服务交任务，探针与自证格的读数都没跑出来，判决要引的数由主 agent 照「要主 agent 跑的命令」跑；副本上没量任何数（这一轮没建仓副本）。
- 没写崩溃状态模型、没有 `rerun.sh` 与外包一层的复跑脚本：这一题不涉崩溃枚举。
- 没删的：草稿目录 `/home/fy5090/.local/share/singlefs/work/split-governance-scripts-services-r1-opus/` 里的 `probe/`、`tools/` 与各份 `*.txt` 中间输出（报告的复核材料，`.deliverables` 已写；636K，`du -sh` 现量）。这一轮没建编译目录、仓副本与工作树。探针跑时会在同目录建 `probe-work/`（一份 HEAD 的 `lib_function_units.py`、一个指回仓里 `.claude` 的符号链接与一棵三行的小工作区），主 agent 读完判读行后删它。
