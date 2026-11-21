# 钩子共用库折得出读法第三轮：正推腿报告（2026-11-21）

范围：背景材料 `research/prompts/_hook-lib-foldable-reads-governance-r3-body.md` 第 4、5 问；读了被判的四份（`.claude/hooks/lib_project_code_runs.py`、`_base.py`、`_launchers.py`、`_selftest.py`，`sha256sum -c research/prompts/hook-lib-foldable-reads-governance-r3-snapshot/judged-sha256.txt` 四份都 OK，且与草稿 `~/.local/share/singlefs/work/hook-lib-r2-fix-b/new/` 四份 `cmp -s` 逐字节相同）、`.claude/hooks/project_code_guard_selftest.py`、算键库 `scripts/gate/lib_function_units*.py` 与前两轮判决、工具员报告。禁读清单里的文件没读。

方案按 `crates/` 今天的实现来谈：这一轮被判的文件里没有 `crates/` 下的代码；`git diff HEAD --stat -- crates | tail -1` 此刻是「1446 files changed, 587159 insertions(+), 544876 deletions(-)」，工作区里别的改动很大，所以「crates/ 下一个字节没动」只能读成「这一题的换上没碰它」（换上规格只含四份钩子文件，见工具员报告 `result.md`），不能读成对提交基线没差异（见第 4 问 T0）。

我自己写的辅助脚本都在草稿目录 `/home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r3-sonnet/`，不 import 本仓代码、不写仓内；没有「模型」目录（没写行为模型）。sha256：

- `closure_check.py` 805e29111ca910d809b9fb80827e3d4178ae797904a10b61c0ed6f0e51449f0f
- `entry_probe.py` e2e2475f346c689be7c22f80ea010886fd2a637cfffbb310452eed3be889c918
- `audit_tables.py` d86791a68b8cd4ffd7934e8c91fa9873519ca45e724f0915b97f58921a5918c9
- `audit_sources.py` 5e85340d72b6eb7c489eb09dd3e1772d8572f24ed6d22286be29cf06f95ead62
- `audit_nonleading.py` 9ed2b088977a5c19a504b000f50b68cb23188ec0ad1e5d224430581d6aae8125
- `audit_other_forms.py` 351efe2714d0cff2c84cd22be0b3b554aff0002b4261f1e79ba3dc9fd7cea10a

我直接跑过的（读文本、不起项目代码）：`python3 -I audit_tables.py <仓根>`、`audit_sources.py`、`audit_nonleading.py`、`audit_other_forms.py`、`closure_check.py --logic-selftest`（六个合成样本全「ok」：干净、少一份、指纹不对、拒算行、data-directory 行、via 整目录各判对）。要起 `lib_function_units.py closure` 的（`closure_check.py <仓根>`、`entry_probe.py`）写在第三节交主 agent。

## 第 4 问：「实现今天的样子」逐条

每条回「成立」「不成立」「推不出」。「推不出」的都是主 agent 的运行读数（退出码、复跑条数、闭包个数）：我没有可复跑的产物，命令在第三节。

### 改法 1

- T1 新识别法 `shell_source_lines` 在 `_launchers.py`：成立。「def shell_source_lines(text, file_name, inherited=None):」（`.claude/hooks/lib_project_code_runs_launchers.py:1100`）。
- T2 认的几种写法：成立（读码）。写法清单在文档串里：「`L="$X/lib"`」与「`R="$(cd "$X/../.." && pwd)"`」（`.claude/hooks/lib_project_code_runs_launchers.py:1103`、`:1104` 两行，前者在第 1104 行、后者在第 1103 行）；顶层 cd：「SHELL_TOP_LEVEL_CD = re.compile(r'^cd\s+')」（`.claude/hooks/lib_project_code_runs_launchers.py:967`）；库看得到入口设的变量：「def shell_places_inside(places, relative):」（`.claude/hooks/lib_project_code_runs_launchers.py:1091`）在第 1204 行被用：「shell_places_inside(places, inner))」（`.claude/hooks/lib_project_code_runs_launchers.py:1204`）。「认」是不是真认出仓里那 36 份脚本，我没跑识别函数：推不出，要主 agent 复跑共用库自证（第三节命令 1），我另用不 import 本仓代码的文本扫描独立数过：三张表 36 份脚本加库表 shell 库里行首 source / `.` 行 131 行（`audit_sources.py`，输出「line-initial source lines (scripts+shell libs, textual): 131」），库表里每份 shell 库的文件名都被某条 source 行点到（「shell libs whose basename is not in any line-initial source line: 0」），source 行里写到的 `*.sh` 都在库表或三张表里（「source lines naming a *.sh outside table+scripts: 0」）。
- T3 认不出的 source 行先查登记表、没登记的照不同判：成立。查登记：「library = None if own_file is None else OWN_SCRIPT_UNRECOGNIZED_SOURCES.get((own_file, written))」（`.claude/hooks/lib_project_code_runs_launchers.py:1186`）；没登记进 `unrecognized`，拷贝判不同：「if reading.unrecognized:」（`.claude/hooks/lib_project_code_runs_launchers.py:1587`）接着第 1588 行「return False」。登记表条数：`audit_tables.py` 输出「OWN_SCRIPT_UNRECOGNIZED_SOURCES type dict len 10 distinct 10」，表头在「OWN_SCRIPT_UNRECOGNIZED_SOURCES = {」（`.claude/hooks/lib_project_code_runs_base.py:237`）。
- T4 库表由 21 份补到 63 份：成立。表头「OWN_SCRIPT_LIBRARY_FILES = (」（`.claude/hooks/lib_project_code_runs_base.py:204`）；`audit_tables.py` 输出「OWN_SCRIPT_LIBRARY_FILES type tuple len 63 distinct 63」「table sorted: True」「table-missing-on-disk 0 []」；第二轮修补前那一版：`python3 -I` 读草稿 `~/.local/share/singlefs/work/hook-lib-r2-fix-b/base/.claude/hooks/lib_project_code_runs_base.py` 得「base table len 21」。另：`.claude/gate.d/lib/` 下 31 份 `.sh`、`scripts/hosts/peer-host-lib.sh`、登记表点名的三份库都已在这 63 份里（`closure_check.py` 的 `expected()` 把它们并进来后「libraries」仍是 63）。
- T5 表那一格另加一条不经识别函数的核法（逐行数行首 source / `.`）：成立。「results.append(("同族：三张表里的脚本与 OWN_SCRIPT_LIBRARY_FILES 里的 shell 库，行首是 source / . 的每一行都认出了或登记过（不经识别函数逐行数）", [],」（`.claude/hooks/lib_project_code_runs_selftest.py:1106`），数行用的正则说明在「# 不经识别函数数 source 行那一格认的行：缩进之后以 source 或 . 起头、后面不是 =（注释行另外跳过）」（`.claude/hooks/lib_project_code_runs_selftest.py:187`）。限定：这一条只数行首的；不在行首的命令位置 source（`$(source …)`、`(source "$library" && …)`、`bash -c '… source "$2" …'`）它数不到，只靠识别函数交回的 `unrecognized`。我的文本扫描数到 22 处不在行首的命令位置 source（`audit_nonleading.py` 末行「non-line-initial command-position source occurrences: 22」），点名的库都已在库表里，登记表里 `"$library"`、`"$2"`、`./args`、`in` 等写在其中；这不推翻 T5，写在这里给攻方腿的第 1 问当线索。

### 改法 2

- T6 本仓那一边挑不到库一律判不同：成立。「if own_digest is None or (os.path.lexists(copy_library) and file_sha256(copy_library) != own_digest):」（`.claude/hooks/lib_project_code_runs_launchers.py:1592`）：`own_digest is None`（表外的、表里点名而盘上没有的，`own_library_digest` 只在表里按相等挑，见 `.claude/hooks/lib_project_code_runs_base.py:553` 的函数）在 `or` 的左边，不再被「别处那边没有」挡在前面。`lexists` 还在这一行，但它只管别处那一份拷贝（别处没有某份库时那份不比），不是「本仓那一边」的筛；正文说的「表那一格不再用 `lexists` 筛」也成立：对 `.claude/hooks/lib_project_code_runs_selftest.py` 搜 `lexists` 零命中，表那一格读法是「(sorted(compared_library_table), [listed for listed in OWN_SCRIPT_LIBRARY_FILES if file_sha256(os.path.join(HOOK_REPOSITORY, listed)) is None])))」（`.claude/hooks/lib_project_code_runs_selftest.py:1089`）。
- T7 「先加 source 行、后建库」两步各一格：成立。第一步：「results.append(("同族：先加 source 行、库还没建：本仓那一份引的库盘上没有，逐字节相同的拷贝不按名字判（读正文）", False,」（`.claude/hooks/lib_project_code_runs_selftest.py:1028`）；第二步：「results.append(("同族：先加 source 行、后建库：两边都建好、逐字节相同的拷贝照按名字判", True, judged_by_name(TWO_STEP_STAGE, two_step_copy)))」（`.claude/hooks/lib_project_code_runs_selftest.py:1033`）。这两格测的是「表里点名而盘上还没建」那一形（样本把库表换成多一项的 `OWN_SCRIPT_LIBRARY_FILES + (TWO_STEP_LIBRARY,)`），与判决说的两步一致；「source 行引了一份表外的库」那一形（`own_library_digest` 因不在表里交 None）这两格没点到，由 `own-library-digest-unlisted` 下红的格管。
- T8 fixtures 那一格改成把仓里的阶段源码拷进自证自造的门禁目录：成立。「real_stage_directory = os.path.join(work, "gd-repository-stages", ".claude", "gate.d")」（`.claude/hooks/lib_project_code_runs_selftest.py:412`），拷源「shutil.copyfile(os.path.join(HOOK_REPOSITORY, GATE_STAGE_DIRECTORY, "crates-mutation-replay.sh"), os.path.join(real_stage_directory, "crates-mutation-replay.sh"))」（`.claude/hooks/lib_project_code_runs_selftest.py:415`）。共用库自证里这一格不再读仓里 `fixtures/` 目录在不在；`project_code_guard_selftest.py` 那边的两格还读（第 5 问 U3）。

### 改法 3

- T9 自证读自己的语法树、按循环体里的 `shutil.copy*` 认那一圈：成立。「copy_loops = [node for node in ast.walk(own_source_tree) if isinstance(node, ast.For) and isinstance(node.target, ast.Name)」（`.claude/hooks/lib_project_code_runs_selftest.py:1287`），后几行（第 1288 到 1292 行）要求循环体里有 `shutil.copy*` 调用、参数里同时有循环变量与 `part_probe_directory`；被认的那一圈是「for copied_name in PART_PROBE_COPIED_FILES:」（`.claude/hooks/lib_project_code_runs_selftest.py:1308`）。期望值是 `(["PART_PROBE_COPIED_FILES"], 1, True)`：认到零圈时迭代对象集合是空表、与期望不等，格红。常量只赋一次、字面串元组：「PART_PROBE_COPIED_FILES = (」（`.claude/hooks/lib_project_code_runs_selftest.py:131`），`grep -c PART_PROBE_COPIED_FILES` 得 10 处，赋值只此一处。
- T10 两条开关与 `own-library-table-short`：成立。「if break_is_set("part-probe-copy-loop-tuple-added"):」（`.claude/hooks/lib_project_code_runs_selftest.py:1293`）、「if break_is_set("part-probe-copied-files-tuple-added"):」（`.claude/hooks/lib_project_code_runs_selftest.py:1298`）；文件头各有一条：「LIB_PROJECT_CODE_RUNS_BREAK=part-probe-copy-loop-tuple-added」（`.claude/hooks/lib_project_code_runs.py:250`）、「LIB_PROJECT_CODE_RUNS_BREAK=part-probe-copied-files-tuple-added」（`.claude/hooks/lib_project_code_runs.py:248`）；表少一项：「if listed == OWN_SCRIPT_LIBRARY_DROPPED_BY_BREAK and break_is_set("own-library-table-short"):」（`.claude/hooks/lib_project_code_runs_base.py:560`），文件头「LIB_PROJECT_CODE_RUNS_BREAK=own-library-table-short」（`.claude/hooks/lib_project_code_runs.py:232`）。开关打开后红在点名的格：推不出（要复跑开关，第三节命令 4）。

### 改法 4

- T11 共用库文件的载入判法收窄：成立。按模块名：「TASK_SERVER_LIBRARY_IMPORT = re.compile(」（`.claude/hooks/lib_project_code_runs_base.py:944`），那一整行只认 import 加模块名、from 模块名 import、`import_module` 与 `__import__` 后接带引号模块名三种；按文件要同时有载入调用与文件名：「if (not break_is_set("task-server-library-file-loads-unjudged") and TASK_SERVER_LIBRARY_FILE_NAME in text and TASK_SERVER_LIBRARY_FILE_LOAD.search(text)」（`.claude/hooks/lib_project_code_runs_base.py:1247`）；按模块名的那一支照旧在第 1245 行：「if TASK_SERVER_LIBRARY_IMPORT.search(text):」（`.claude/hooks/lib_project_code_runs_base.py:1245`）。库文件名不在 `task_server_guarded()` 的路径与名字里（那里只有库文件名 `DATABASE_NAME`、密钥文件名与 socket，读 `.claude/hooks/lib_project_code_runs_base.py:1147` 起的几行），所以只读命令里写到那个文件名不被那两支拦。只读放行的样本：「("绕开：只读的 grep 里写到 task_server_lib.py 不算", "grep -n 'task_server_lib.py' task-server/task_server.py", False),」（`.claude/hooks/lib_project_code_runs_selftest.py:1230`）。单独的开关：「LIB_PROJECT_CODE_RUNS_BREAK=task-server-library-file-loads-unjudged」（`.claude/hooks/lib_project_code_runs.py:333`）。
- 一次实测（不是推的）：我第一次往报告里追加这一节时，heredoc 的正文里同时写了那个文件名、载入调用名加开括号和「import 加模块名」的字面，项目代码闸整条拒了，原样输出头一行：「✗ 绕开 task-server 的写法被拒：在代码里载入 task-server/task_server_lib.py（它管库与签名密钥）」。也就是只读地「写文字」的命令，正文同时带这两样也被拒：判法按整段命令文本里有载入调用与文件名、不看这条命令本身是不是在载入。这与 T11 的判法（`TASK_SERVER_LIBRARY_FILE_NAME in text and TASK_SERVER_LIBRARY_FILE_LOAD.search(text)`）一致；`grep` 本身带着载入调用名加开括号的写法同理会被拒（给攻方腿的第 3 问；我没单独喂）。

### 开关表、换上、闭包

- T12 文件头开关表 91 条：成立。对 `.claude/hooks/lib_project_code_runs.py` 数行首「  LIB_PROJECT_CODE_RUNS_BREAK=」的行，`grep -cE` 输出 91；按名字取出后 `sort | uniq -d | wc -l` 输出 0（91 条各不相同）。本轮六条新名字：「LIB_PROJECT_CODE_RUNS_BREAK=script-directory-variables-skipped」（`.claude/hooks/lib_project_code_runs.py:287`）、「LIB_PROJECT_CODE_RUNS_BREAK=unrecognized-sources-skipped」（`.claude/hooks/lib_project_code_runs.py:388`）、own-library-table-short、两条 `part-probe-*`、task-server-library-file-loads-unjudged 在上面各条里点过；它们在代码里各被读：`script-directory-variables-skipped` 在「skipped = break_is_set("script-directory-variables-skipped")」（`.claude/hooks/lib_project_code_runs_launchers.py:1108`），`unrecognized-sources-skipped` 在「if not break_is_set("unrecognized-sources-skipped"):」（`.claude/hooks/lib_project_code_runs_launchers.py:1188`）。
- T13 开关复跑登记行写九十一条：成立。「九十一条开关」（`.claude/gate.d/break-switch-replay-files.tsv:70`）。两份改前基线草稿的文件头条数（同一个 `grep -cE` 模式）：`~/.local/share/singlefs/work/hook-lib-r2-fix-b/base` 是 85，`~/.local/share/singlefs/work/hook-lib-keys-cover-libraries/base` 是 83，与前两轮判决与工具员报告的「八十三」「85 变 91」叙述相合。
- T14 换上后四份与草稿 `new/` 逐字节相同：成立（`cmp -s` 四份都输出同）。「干跑、真换各单独交、都退 0」：推不出（退出码是主 agent 任务的结局，我没有可读的；草稿 `~/.local/share/singlefs/work/hook-lib-r2-fix-b/swap-real/hook-swap/` 与 `swap-dry/hook-swap/` 各留了喂法日志与反向规格，只说明两趟起过，不含退出码）；复核命令：第三节命令 2（`hook-swap.py --read-outcome`，只读，退 0 是「换完了」）。
- T15 共用库自证、项目代码闸自证退 0，开关复跑「复跑了 91 条开关（1 份），自证红且点名的格与实际红的格逐格相同 91 条」：推不出（同上，没有我能读的结局）。命令 1、3、4。
- T16 两条闭包 refused 0 行、via 里没有目录、data-file 109 个、27 份 `*-cells.sh` 与 `peer-host-lib.sh` 都在：推不出（要起 `lib_function_units.py closure`）。我能静态核的前提：应有集合 107 份（三张表 36、库表并上盘上 `lib/*.sh` 与登记库 63、`PART_PROBE_COPIED_FILES` 8、检出钩子那一份 1、盘上 `.claude/gate.d/*.sh` 17，互相有重叠）每份盘上都在（`closure_check.py` 的 `expected()` 现算后逐份 `isfile`，输出「missing-on-disk: []」）；`.claude/gate.d/*.sh` 盘上 17 份都在三张表推出的脚本集合里（输出「stages not in GATE_STAGE_FILES-derived scripts: []」）；盘上 `.claude/gate.d/lib/*.sh` 31 份全在 63 份库表里（并进应有集合后「libraries」仍是 63）。主 agent 的那一行读数在记录里：「两条闭包都是 refused 0 行、via 里没有目录、data-file 109 个」（`records/2026-11-17-subagent账本清欠调度记录.md:595`）；我的应有集合是 107，所以 109 里至少有 2 份不在我的集合里（EXTRA 或我漏算），是什么等第三节命令 5 打出来。

### 其余

- T0 「`crates/` 下一个字节没动」（正文第三段）：对本题的换上成立、对提交基线不成立。换上只改四份钩子文件（工具员报告 `result.md` 的「改过的文件」一节：「4 files changed, 563 insertions(+), 75 deletions(-)」）；`git diff HEAD --stat -- crates | tail -1` 此刻输出「1446 files changed, 587159 insertions(+), 544876 deletions(-)」，是工作区里别的改动，与这一题无关。判决写「crates/ 没动」时要带上「这一题没碰」的限定。
- T17 工具员留给主 agent 的两处存在、描述准确：成立。闭包正向断言：「- 正向断言（判决改法 3）：「两条闭包应有的文件集合都在」要起算键库，放不进共用库自证。」（`/home/fy5090/.local/share/singlefs/work/hook-lib-r2-fix-b/result.md:111`）；`project_code_guard_selftest.py` 那一格：「stage_sample_command = "bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh .claude/gate.d --item crates-mutation-replay.sh"」（`.claude/hooks/project_code_guard_selftest.py:1081`）。补充：同一依赖不止「第 1081 行起」那一格，第 1317 行起的另一格（经真实入口喂）也用同一条命令：「"command": "task-server/task submit -- bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh .claude/gate.d --item crates-mutation-replay.sh"}}))」（`.claude/hooks/project_code_guard_selftest.py:1319`），两格都在第 5 问 U3 判。
- T18 「改前原样」两份草稿在：成立（上面 T13 读过两份 `base/` 里的文件头）；`~/.local/share/singlefs/work/hook-lib-r2-fix-b/base/` 里四份的 sha256 与 `research/prompts/hook-lib-foldable-reads-governance-r2-snapshot/judged-sha256.txt` 四行相同（我逐份 `sha256sum` 对过）。
- T19 「第二轮核不动：键那一半…在攻方腿的副本上算不出键（`key` 退 4）」与判决原话相合：成立。第二轮判决：「- keys 段：type=command-failed exit=1，两个基线都「key=None … exit=4」，副本里算不出键」（`research/prompts/hook-lib-foldable-reads-governance-r2-main-verification.md:18`）。

## 第 5 问：前两轮判决里「核不动」「推的」逐条现查

前两轮判决里带「核不动」「推的」「推不出」的位置，逐条看今天能不能核、怎么核（`grep -nE '核不动|推的|推不出'` 在两份判决里命中第一轮第 10、14、18、24、26 行与第二轮第 9、18、22、26、27、49 行）。

- U1a 第一轮「键漏」段四个基线算不出键、核不动（「副本里那一版算键库对这两个入口拒算」，`research/prompts/hook-lib-foldable-reads-governance-r1-main-verification.md:18`）：副本上今天仍不能核，主工作区上能。要核的是「库表 63 份是不是都进了闭包的 data-file、指纹与盘上字节相同」，命令 5 打 `missing` 与 `fingerprint-differs`；两样都是 0 才算 63 份都在、每份的字节进了键（数据行第四列是字节 sha256，`scripts/gate/lib_function_units.py:16`）。静态前提我已核：63 份每份盘上都在（`audit_tables.py`：「table-missing-on-disk 0 []」）。
- U1b 第一轮「`preflight.sh` 改前也不在键里（攻方腿读码推的）」「改前罩没罩住是攻方腿读码推的」：改前那一半复核不了（改前的树只剩四份文件的草稿，没有整棵仓，闭包算不出；第二轮判决已把「连带罩住」改正为「改前是拒算、没有键」，不再靠这一句）。今天那一半能核：`.claude/scripts/preflight.sh` 在库表里（`.claude/hooks/lib_project_code_runs_base.py:219` 一行末尾），命令 5 的 `missing` 里它不该出现。
- U1c 第一轮正推腿「换前三行整目录读、data-file 7、473 种、1492 种、83 条」推不出（`research/prompts/hook-lib-foldable-reads-governance-r1-main-verification.md:10` 一行里转述）：「83 条」我现查到了（改前基线草稿文件头 83 条，见 T13）；其余四个是换前读数，主工作区已换上，复核不了，也不再影响任何一条改法，不追。
- U1d 第二轮「fixtures 目录在不在不进键」与「存在性不进键」的键那一半核不动（`research/prompts/hook-lib-foldable-reads-governance-r2-main-verification.md:22`），另一处写「存在性不进键那一半核不动，改法里一并处理」（`research/prompts/hook-lib-foldable-reads-governance-r2-main-verification.md:26`）：分三层。库的存在性：本仓那一边按表里字面路径读，「根外的、盘上没有的跳过」（`scripts/gate/lib_function_units.py:133`），所以表里点名的库盘上有则进键、没有则不在键里，盘上从无到有键变；这一层按文档读是成立，实测靠命令 5。共用库自证里依赖 `fixtures/` 的那一格：已改成不依赖（T8）。`project_code_guard_selftest.py` 里依赖它的两格：没动，见 U3。
- U1e 第二轮「27 份里 24 份与 `scripts/hosts/peer-host-lib.sh` 不在」（`research/prompts/hook-lib-foldable-reads-governance-r2-main-verification.md:18`）：今天库表 63 份里含 `lib/` 下 31 份 `.sh`（其中 `*-cells.sh` 27 份）与 `scripts/hosts/peer-host-lib.sh`，命令 5 的 `libraries` 一栏现算并核它们在不在；`ls .claude/gate.d/lib/*-cells.sh | wc -l` 输出 27，与正文数相同。
- U1f 登记表 10 条 source 行引的库在不在：成立（静态，`audit_tables.py` 逐条打）。三条点名了库的：`.claude/gate.d/lib/crash-point-replay-crash-case-cells.sh` 第 36 行的「(source "$library" && heavy_thread_default) 2>/dev/null」（变量取 `$crash_point_replay_stage_repository/scripts/hosts/heavy-threads.sh`，登记为 `scripts/hosts/heavy-threads.sh`）、`.claude/scripts/lkmm.sh` 第 93 行的 `"${PREFLIGHT_PROGRAM%/*}/lib.sh"`（登记为 `.claude/singlefs-ai-sop/scripts/lib.sh`）、`.claude/scripts/preflight.sh` 第 20 行的 `"$project_preflight_library"`（登记为 `.claude/singlefs-ai-sop/scripts/preflight.sh`），三份库盘上都在、都在 63 份库表里（输出「lib-on-disk: True | lib-in-table: True」各一行）。另七条登记空串的行我逐条在原文件里找到：feature-card、doc-experiments-preregistration、doc-text 三份里的三条是嵌在 shell 里的 awk / python 的 `source in …`、`source is …`（`for source in …`、`if source is None`），`"$2"` 两条是 `bash -c` 串里的位置参数，`/args`、`./args` 是 vm-bench 在虚机与临时目录里读的 args 文件；这七条不是本仓的库，登记空串合理。登记之后「那一条引的库还比不比、进不进键」：登记了库的三条，库在库表里、`own_library_digest` 比、命令 5 里 `libraries` 栏进键；登记空串的不比、不进键，这是设计。

### U2 闭包正向断言的落点（工具员没定）

判断：落在门禁 code-tooling 里、紧挨 `research-script-selftests` 的一格（在真仓上算 `judge:` 闭包的那一道），配两条弄坏开关（应有集合少一份、闭包里多一处整目录读）；不落共用库自证，也不落算键库的真仓拷贝格。

依据：

1. 共用库自证起不了算键库：登记表写它「不起子进程」（「.claude/hooks/lib_project_code_runs.py	selftest	# 在临时工作区里判造出来的命令行，不起子进程」，`.claude/hooks/selftest-registry.tsv:211`），自证源码里对 `lib_function_units` 只在两处注释里提到（`grep -nE 'lib_function_units' .claude/hooks/lib_project_code_runs_selftest.py` 命中第 129、1266 两行，都是注释），没有 import。
2. 被盯的键在被判的仓里算：「与判法的闭包（judge:，在这一道所在的仓里算）」（`.claude/gate.d/lib/code-tooling-selftest-cells.sh:31`），名单在「SELFTEST_JUDGE_ENTRIES = (」（`scripts/gate/selftest-process-guard.py:164`），第一条就是 `watch_judgement`，正是两条闭包之一。成员断言放在这一道旁边，核的就是实际要算键的那棵树。
3. 拒算与缺成员方向不同：「拒算的（闭包收不全）照跑、不记」（`.claude/gate.d/lib/code-tooling-selftest-cells.sh:35`）——拒算是安全方向（每次现判），危险的是不拒算而静默缺成员，改了库键不变、旧绿被复用。所以断言要三样一起：不拒算、成员齐、指纹等于盘上字节；`closure_check.py` 做的就是这三样。
4. 不选算键库的真仓拷贝格：那一类格拷的只有 `scripts` 与 `.claude`（「for top in ("scripts", ".claude"):」，`scripts/gate/lib_function_units_selftest_fixture_trees.py:426`），没有 `crates/`；第一轮判决量到副本上这两条闭包会拒算（`research/prompts/hook-lib-foldable-reads-governance-r1-main-verification.md:18`），在拷贝上断言成员，红可能是拒算造成的而不是成员缺。现有先例（`rust_lex_namespace_closure_cell`，「def rust_lex_namespace_closure_cell(work):」在 `scripts/gate/lib_function_units_selftest_rust_cells.py:967`）取的是算键库自己的入口、用「closure_key(closure, allow_refused=True)」（`scripts/gate/lib_function_units_selftest_rust_cells.py:982`），不碰 herd7 与 cargo；这两条闭包不是这种。这一条里「拷贝上会拒算」是推的：第一轮是攻方腿的副本，不是 `real_tree`；要核可以在 `real_tree` 形态的拷贝上交命令 6 的 `entry_probe.py`（本题没有交这一条，列在「没做什么」）。
5. 应有集合怎么算有先例：`runtime_data_files_cell` 用 ast 读登记表再与现扫的读法逐项比（「def runtime_data_files_cell(work):」，`scripts/gate/lib_function_units_selftest_program_registries.py:1054`）；这一格同样从 `lib_project_code_runs_base.py` 与 `_selftest.py` 的字面表用 ast 现算，不另抄一份清单。

代价：门禁格只在门禁那一趟（提交时的整轮）跑，只改共用库、没改算键库的中间几趟没有当场信号；补法是让这一格的键含这几份库（它读的就是它们）。另一选择是写进算键库的真仓自证，好处是改算键库当场红，坏处见依据 4。

### U3 `project_code_guard_selftest.py` 那一格依赖 fixtures 目录在不在（工具员没定）

判断：改，与共用库自证里 T8 同一做法；改的是两格，不是一格。

依据：

1. 依赖是真的：喂进去的命令是「stage_sample_command = "bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh .claude/gate.d --item crates-mutation-replay.sh"」（`.claude/hooks/project_code_guard_selftest.py:1081`），`cwd` 是仓根（「repo, real_sample_index = project_code_runs.HOOK_REPOSITORY, project_code_runs.lib_project_code_runs_base.SAMPLE_INDEX_REPOSITORY」，`.claude/hooks/project_code_guard_selftest.py:147`），判法按阶段目录里 `fixtures/<阶段>` 是不是目录挑被喂的阶段：「if stage.endswith(".sh") and not stage.startswith(".") and os.path.isdir(os.path.join(gate_directory, "fixtures", stage))」（`.claude/hooks/lib_project_code_runs_base.py:773`）；挑到之后读阶段源码找临时服务核对（`.claude/hooks/lib_project_code_runs_launchers.py:463` 起）。仓里 `.claude/gate.d/fixtures/crates-mutation-replay.sh/` 此刻是目录（`ls -d` 成功），挪走或改名这一格就变，而路径来自命令文本，算键库按字面折不出它。
2. 第二格同一依赖：第 1317 行起的真实入口喂法用同一条命令（「"command": "task-server/task submit -- bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh .claude/gate.d --item crates-mutation-replay.sh"}}))」，`.claude/hooks/project_code_guard_selftest.py:1319`），工具员报告只点了第 1081 行那一格。
3. 换法有先例、不丢东西：共用库自证里已经这样换了（T8，`.claude/hooks/lib_project_code_runs_selftest.py:412` 到第 421 行）：脚本取「own_stage_selftest = os.path.join(HOOK_REPOSITORY, KNOWN_SCRIPT_LOCATIONS[STAGE_SELFTEST_SCRIPT])」（`.claude/hooks/lib_project_code_runs_selftest.py:398`）这一份本仓的绝对路径，阶段目录换成自证自造的、里面拷仓里阶段源码并造 `fixtures/<阶段>/red`，期望的整条命令用 `shlex.join` 现拼。守卫自证那两格测的是「入口接线：退 2、说明点名那一道、给整条包装写法」，与读哪一棵门禁目录无关；「仓里那一道阶段源码真带临时服务核对」这件事已由共用库自证里拷仓里源码的那一格（`.claude/hooks/lib_project_code_runs_selftest.py:419`）兜住。
4. 不改的反方：这两格是端到端喂真入口，换成样本目录之后「真仓这一道」只剩共用库那一格看着；以及依赖若已在键里（命令 6 打出 `fixtures=[…]` 非空）就只是「会红」而非「静默复用」，改不改都行。我的判断是仍改：键里有没有是推的，改了之后两格不再依赖盘上目录状态。

## 第三节：要主 agent 跑的命令

一条一项；都没起过（子 agent 不交任务）。每条交任务只写一条，不串别的。

1. 共用库自证。在仓根 `/home/fy5090/code/singlefs` 起：`task-server/task submit -- python3 .claude/hooks/lib_project_code_runs.py --selftest`。看：`TASK_SERVER_RESULT` 的 `exit=0`，日志末行「✓ lib_project_code_runs 自检通过（查了 N 种…）」，没有「✗ lib_project_code_runs 自检：」行。核 T15。
2. 上一趟换上的结局。在仓根起：`task-server/task submit -- python3 scripts/gate/hook-swap.py --read-outcome`。看：`exit=0`（换完了，每份文件现在等于回读）；退 1 是已退回，退 2 是读不了或没有结局文件。核 T14 的「真换退 0」。
3. 项目代码闸自证。在仓根起：`task-server/task submit -- bash .claude/hooks/project-code-guard.sh --selftest`。看：`exit=0`，没有「✗」行。核 T15。
4. 开关复跑。在仓根起：`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 break-switch-replay 格，点名 .claude/hooks/lib_project_code_runs.py; 范围=2 项; 为什么=共用库换上之后核 91 条开关各自红在文件头点名的格上' task-server/task submit -- bash .claude/gate.d/code-tooling.sh --check break-switch-replay --item .claude/hooks/lib_project_code_runs.py`（范围声明照工具员报告那一条原样抄，没改）。看：`exit=0`，汇总「复跑了 91 条开关…逐格相同 91 条」，没有 `red-but-unnamed` 与 `named-but-not-red`。核 T10、T15。
5. 两条闭包的正向核（成员、指纹、拒算、整目录读）。在草稿目录 `/home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r3-sonnet` 起：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r3-sonnet -- python3 -I /home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r3-sonnet/closure_check.py /home/fy5090/code/singlefs`。看：首行「EXPECTED total=107 …」（应有集合现算，我这边算出 107）；两条「ENTRY」行各自 `exit=0`、`refused=0`、`whole-directory-reads=0`、`missing=0`、`fingerprint-differs=0`；`data-files` 个数与记录里的 109 对照；`EXTRA` 行列出闭包里有而不在应有集合里的（我预计 109 减 107 份，是什么由它打出来）；`MISSING` 或 `FINGERPRINT-DIFFERS` 有行就是库表 63 份里有一份没进键或指纹不是盘上字节。脚本退 0 是全满足，退 1 是有一样不满足，退 2 是超时（每条入口内部 1500 秒上限）。内存与时间：没跑过，没量；闭包是只读算键，内存上限由服务定。核 T16、U1a、U1b、U1d、U1e、U1f、U2 依据 3。
6. 守卫自证入口的闭包里有没有那一个 fixtures 目录（U3 的前提）。同一草稿目录起：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r3-sonnet -- python3 -I /home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r3-sonnet/entry_probe.py /home/fy5090/code/singlefs .claude/hooks/project-code-guard.sh --program-argument --selftest`。看：「ENTRY」行的 `exit`（0 判完、4 有拒算）、「REFUSED」行（有就是这个入口的键拒算，是安全方向，依赖不会静默复用）、「DATA」行里 `fixtures=[…]` 与 `crates-mutation-replay=[…]` 是不是空：两个都空且没有 REFUSED，就是守卫自证那两格依赖的盘上状态不在键里（U3 判断成立）；有 `.claude/gate.d/fixtures/crates-mutation-replay.sh/…` 或 `.claude/gate.d/crates-mutation-replay.sh` 在列，就是已进键，U3 降为可选。

我自己直接跑过、不经服务的（读文本、`py_compile`、合成样本，不 import 本仓代码、不写仓内）：`audit_tables.py`、`audit_sources.py`、`audit_nonleading.py`、`audit_other_forms.py`、`closure_check.py --logic-selftest`、`entry_probe.py` 在草稿里造的假仓根（一个只会打几行假闭包行的假 `lib_function_units.py`）上核了它的解析，跑完已删假仓根。

## 什么现象会推翻

- T1 到 T13、T17 到 T19（读码成立的）：换上之后的四份文件与 `judged-sha256.txt` 不再一致，或被引的行号处的原文变了，就要重读；我读的是四份 sha256 对得上的版本。
- T3、T4 与 U1f（表与登记表）：命令 5 的 `MISSING` 里出现库表里的某一份，或命令 1 里「同族：OWN_SCRIPT_LIBRARY_FILES 与三张表里每份脚本引的库逐个相同，表里每一份都在本仓盘上」「同族：三张表里的脚本与它们引的库里，认不出写法的 source 行都登记在 OWN_SCRIPT_UNRECOGNIZED_SOURCES 里、登记的每一条都还在」红，T3、T4 就不成立。
- T5 的限定（行首之外数不到）：若命令 1 绿而我的 `audit_nonleading.py` 列的 22 处里有一处点名的库不在库表、也不在登记表里，那一处是识别函数漏认的实例；我对这 22 处逐处看到点名的库都在表里，没有这样的实例。
- T14、T15、T16 的「推不出」：命令 1 到 5 全绿即改成「成立」；任何一条红或 `EXTRA`/`MISSING` 里有库表的成员，就改成「不成立」并按红的格定位。
- U2 的判断：命令 6 在 `real_tree` 形态的拷贝上要是算得出键（不拒算），依据 4 就不成立，落点可改回算键库的真仓自证；`research-script-selftests` 这一道里的 `judge:` 闭包要是不在被判的仓里算（与依据 2 引的第 31 行相反），则依据 2 不成立。
- U3 的判断：命令 6 的「DATA」行里 `fixtures` 与 `crates-mutation-replay` 两个列表非空，就说明依赖已在键里，「改」降为可选；若守卫自证里这两格换成样本目录之后「说明点名那一道、给整条包装写法」的断言红，说明换法丢了接线核对，不能照 T8 这样换。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| T0 `crates/` 没动 | 成立（限定：这一题没碰）；对提交基线不成立 | 换上只改四份钩子文件；`git diff HEAD --stat -- crates` 此刻 1446 files changed，是别的改动 |
| T1 `shell_source_lines` | 成立 | `.claude/hooks/lib_project_code_runs_launchers.py:1100` |
| T2 认的写法 | 成立（读码）；真认出 36 份脚本推不出 | 写法在文档串与正则里；文本扫描独立数到 131 行行首 source，全点到库表 |
| T3 登记表 10 条与不同判 | 成立 | 登记表 10 条；没登记进 `unrecognized`、拷贝返回 False |
| T4 库表 21 到 63 | 成立 | 现 63 份、有序、不重复、盘上都在；改前基线 21 份 |
| T5 不经识别函数的核法 | 成立（限行首） | 只数行首；不在行首的 22 处命令位置 source 数不到 |
| T6 本仓一边挑不到判不同 | 成立 | `own_digest is None` 在 `or` 左边；`lexists` 只留在别处那份拷贝上 |
| T7 先加 source 行、后建库两格 | 成立 | 测的是「表里点名、盘上没建」那一形；表外那一形不在这两格里 |
| T8 fixtures 那一格改法 | 成立 | 仓里阶段源码拷进自造门禁目录判；共用库自证不再读仓里 `fixtures/` |
| T9 语法树格 | 成立 | 按循环体里的 `shutil.copy*` 认那一圈，零圈时红 |
| T10 三条开关 | 成立；开关下红在点名格推不出 | 代码与文件头都在；要命令 4 |
| T11 载入判法收窄与开关 | 成立 | 模块名照拒；文件载入要载入调用加文件名；只读 grep、sed -n、wc 样本放行；一次实测：写文字的命令正文带这两样也被拒 |
| T12 开关表 91 条 | 成立 | 91 条各不相同；六条新名字各在代码里被读 |
| T13 登记行九十一条 | 成立 | `.claude/gate.d/break-switch-replay-files.tsv:70`；两份基线 85、83 条 |
| T14 换上四份相同、干跑与真换退 0 | 前半成立；退 0 推不出 | `cmp -s` 四份同；退出码要命令 2 |
| T15 自证退 0 与开关复跑 91 条 | 推不出 | 要命令 1、3、4 |
| T16 两条闭包的读数 | 推不出 | 前提静态成立（107 份应有文件盘上都在）；命令 5 |
| T17 工具员留的两处 | 成立；补一格 | 同一依赖还有第 1317 行起那一格 |
| T18 改前原样 | 成立 | 两份基线在，四份 sha256 与第二轮快照相同 |
| T19 第二轮核不动的描述 | 成立 | 与第二轮判决原话相合 |
| U1a 到 U1f 核不动与推的 | 现查 | 主工作区上能核的写成命令 5、6；改前读数复核不了；登记库静态成立 |
| U2 闭包正向断言落点 | 判断：门禁 code-tooling 紧挨 `research-script-selftests` 一格 | 共用库自证起不了算键库；键在被判的仓里算；拷贝格有拒算风险（推的） |
| U3 守卫自证依赖 fixtures | 判断：改，两格，照共用库那一格的做法 | 路径来自命令文本、算键库折不出；换成自造目录不丢接线核对 |

## 没做什么

- 没替主 agent 采纳或出判决；没往统一任务服务交任务、没起 `run_in_background`；判决要引的数没复跑，命令在第三节。
- 没跑识别函数、共用库自证、项目代码闸自证、开关复跑、两条闭包与换上结局的读取；T14 到 T16 因此是「推不出」。
- 没有建仓副本、没有编译目录；没有 `real_tree` 形态拷贝上的闭包（U2 依据 4 那一半，推的）。
- 攻方腿的格（第 1 到 3 问）没做；上面写到的行外 source（22 处）、载入调用名加开括号被只读命令带上等，是顺手看到的线索，没当结论。
- 前两轮判决里换前读数（整目录读行数、data-file 7、473 种、1492 种）复核不了：换前的树没有整棵。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r3-sonnet/` 里是我写的六份只读辅助脚本（命令 5、6 用其中两份，留着）与分段草稿、引文核对输出（交回前已删），没有仓副本、没有编译目录。
