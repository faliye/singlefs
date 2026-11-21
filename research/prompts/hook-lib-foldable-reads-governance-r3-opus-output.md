# hook-lib-foldable-reads-governance-r3：攻方腿（Opus）报告（2026-11-21）

被判的是主工作区此刻的四份共用库（`research/prompts/hook-lib-foldable-reads-governance-r3-snapshot/judged-sha256.txt`，开工时 `sha256sum -c` 四份都 OK）。攻击面是正文「要腿判的」第 1 到 3 问；前两轮已攻过而没打中的五样不重复。没读禁读清单里的那份报告。

没建模型目录：这一轮的装置与第二轮一样，是草稿目录里的一条探针（不碰崩溃状态、不起 cargo、不读写判定库），六段命令在文末「要主 agent 跑的命令」。草稿目录 `/home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r3-opus/`：

| 文件 | sha256 | 做什么 |
|---|---|---|
| `baselibs/lib_project_code_runs_launchers.py` | `f5c166d7192d8e8b3bc2e94eec142bf71c01593f6c093b138d603c251ad992fb` | 第二轮修补之前那一版（拷自 `~/.local/share/singlefs/work/hook-lib-r2-fix-b/base/.claude/hooks/`，sha256 相同），探针只从它的语法树里取 `SHELL_SOURCED_LIBRARY` 的正则原文，打「改前那一臂认不认」 |
| `probes/probe_r3.py` | `17a0e918f399cb1ac4115b67635e01dc6814bbe26e4c9a66888e11a683f33b43` | 六段：forms（识别函数对各写法交什么）、family（样本树上 judged_by_name 的判定）、depth（深度上限截掉的库）、selfsame（本仓那一份当别处那一份比）、loads（改法 4 的文本判法）、astcell（改法 3 那一格的原码对变体） |
| `silent_shapes_scan.py` | `e9cee02c3cd1d4c1382f7ff02f497019a4fdd4b4cbfe2bb46888c3ac81fbabed` | 我写的只读扫描（自己的正则，不 import 本仓代码，直接跑过）：三张表 36 份与库表里的 shell 文件里几种「命令位置之外」的 source 写法 |
| `silent-shapes.txt` | `8c10b2c4140542829ba1f9b30321d0a7a46604ed2558befc5ab9376b670ccb83` | 上一份的输出，`FILES 86` |
| `python_depth_model.py` | `3165db7671ab3b8a104a42a2db2d37cd9031c232dfa2a99c28a5ecee892c6fe4` | 我的模型（不 import 本仓代码）：照 `PYTHON_SIBLING_REFERENCE` 与 `script_sources_into` 的先深后广、seen、深度上限只走 python 那一支 |
| `python-depth-broad-mutation.txt` | `268d5e054fe3c2e197c6507347d2f8f591c1778785b4ce30f76753c76ea53b7f` | 模型对 `scripts/mutation/broad-mutation.py` 的输出 |
| `python-depth-without-base.txt` | `75690e45eeeb8fa8d3601708fd4b8c0d00f11bd7ce8275667a7dfd758af3fb11` | 同一模型，样本树里拿掉 `broad_mutation_base.py` |
| `table-files.txt` | `da57842e6dfe575a4595fd68b5f7a36ed0d624128f769a241e4be1c8403271f8` | 按文本从 `lib_project_code_runs_base.py` 抽出的三张表 36 份与库表 63 份 |

读数的出处分三类，每格标明：「量过·钩子」是我这一次 Bash 调用被现役项目代码闸当场判出的原样输出；「量过·模型」是上面两份不 import 本仓代码的脚本的输出；「推的」是读码推的，待主 agent 跑探针（预言逐条写在探针里，读数与预言不同的行带 `MISMATCH`）。

## 各格判定一览

| 问 | 判定 | 一句话 | 证据 |
|---|---|---|---|
| 1 识别法：表外写法 | 成立（打中） | 识别函数只在命令位置上找 source，命令位置不含 `)`、`elif`、赋值前缀、`builtin` / `command`、引号里的 shell 代码；参数是 `<(…)` 的整行跳过。这几种不进 `lines`、不进 `unrecognized`，照「按名字判」走，不落到「照不同判」；其中 `)` 之后、赋值前缀、`builtin` / `command`、`elif` 四种接 `$(dirname "${BASH_SOURCE[0]}")/…` 的，改前那条正则在任何空白之后都认（`(?:^|[;&|\s])`），这一改之后跳过，是改法 1 带进来的退回；不经识别函数的那一格只数行首的 source，几种都数不到。仓里现有一处：`doc-text.sh` 的 dated-status 那一格在单独的 bash 里 source 规范副本的 `lib.sh`，这份库对 `doc-text.sh` 的拷贝从来不比 | 写法与命令位置：读码；仓里那一处：量过·模型（`silent-shapes.txt`）；判定：推的，待 forms、family 段 |
| 1 识别法：深度上限 | 成立（打中） | `SOURCED_LIBRARY_DEPTH = 4` 截掉的库静默不比；`broad-mutation.py` 的 import 链因为入口被自己的库再引一次、白占一层，`crates-mutation-rows.py` 落在第四层、它运行期 import 的 15 份 `crates_mutation_rows_*.py` 与 `lib_mutation_units.py`、`lib_release_only_tests.py` 共 17 份不比、不在库表里，表那一格与数行那一格（跳过 `.py`）都看不见 | 量过·模型（`python-depth-broad-mutation.txt`：`CAP 4 libraries=12` 与库表里 12 份 `.py` 逐个相同，`CAP 99` 多出 17 行 `CUT`）；判定推的，待 depth、family 段 |
| 1 登记表 10 条 | 成立（打中一条形） | 10 条各自确是认不出（逐条见下）；但登记按「那一份文件、照写的参数」认、不认行，同一份里再写一行同样参数引别的库，照登记的那一份库比、新库不比，自证两格都绿。crash-case-cells.sh 的 `"$library"` 认不出正是因为这个名字在那份里赋了两个不同的值，第三处照样被吞 | 读码；判定推的，待 family 段 `registered-line-absorbs-second-library` |
| 1 误认拒了该放的 | 不成立（推的） | 自证绿时，三张表里认出的每个位置都在库表里、都在盘上；忠实的拷贝逐份相同，误认不让它判成不同。只在「误认到一份别处与本仓合法不同的文件」时多读一次正文（`X+=` 这类，形状写在下面），没找到现役脚本这样写 | 推的，待 selfsame 段（36 份本仓那一份当别处那一份比都该交 True） |
| 2 表外的库判不同而拒现役命令 | 不成立（推的） | 忠实拷贝读出的库是本仓读出的子集，都在表里；没找到现役命令因此掉成读正文。另有一处次序依赖：python 那一支先深后广加深度上限，别处那边少一份中间文件时会多读出表外的库、照不同判（只在坏了的拷贝上出现，不是现役） | 推的；次序依赖量过·模型（`python-depth-without-base.txt`），判定待 selfsame 段 |
| 2 两步样本 | 部分成立（打中一半） | 两格测的是判决说的两步的判定（库没建判不同、建好之后判同族），开关红在点名的格；判决同一句要的「表那一格红」那一半（表里点名而盘上没有的）没有样本、没有开关，把那一半换成空表自证照绿 | 读码推的（文件头开关表里没有一条点名它会因盘上缺一份而红） |
| 2 fixtures 那一格 | 成立（测得到） | 换做法之后测的是「仓里那道阶段的源码里有临时服务核对、给了样本档目录就被点名」，判法只看源码与目录在不在，样本档里有什么本来就不进判法；丢掉的只是「仓里那个目录此刻在」，那不是被判代码的性质 | 读码（`stage_selftest_fed_stages` 只看 `fixtures/<阶段>` 是不是目录） |
| 3 分辨格躲不躲得开 | 成立（打中） | 正文点名的三种（`for` 换 `while`、换 `map`、先赋给别的名字）都让那一格红，躲不开；躲得开的是另外三种：模块顶层 `PART_PROBE_COPIED_FILES += …`、`PART_PROBE_COPIED_FILES: tuple = …`（只数 `ast.Assign`）、函数里 `global` 再赋（只看模块顶层），与在那一圈旁边另用推导式拷（只认 `ast.For`） | 读码推的，待 astcell 段（八个变体，原码从仓里那份自证的语法树原样取出跑） |
| 3 改法 4 漏过的真载入 | 成立（打中） | 文本判法要求载入调用与 `open(` 写在同一段（不跨 `;`、不跨行）：先读进变量再 `exec`、`compile` 之后 `exec` 都放行，这两种收窄之前的正则照拒（退回）；`import json, <库名>`、`runpy.run_module`、拼出来的路径交给按文件载入，收窄前后都放行（原来就漏） | 量过·钩子（echo 同一段文本：两条放行、对照那一条被拒，原样输出在第 3 问）；判法函数待 loads 段 |
| 3 改法 4 仍拒的只读命令 | 成立（打中） | 只读的 grep 搜载入调用的名字（带括号）、`grep -nF` 抄自证里那几行样本，照拒 | 量过·钩子（两条只读 grep 当场被拒，原样输出在第 3 问） |

## 第 1 问：改法 1 的识别法与登记表

### 打中一：命令位置之外的 source 静默跳过

识别函数找 source 的位置只有这一张表：「SHELL_COMMAND_POSITION = r'(?:^|[;&|({!]|\bthen\b|\bdo\b|\belse\b|\bif\b|\bwhile\b|\buntil\b)\s*'」（`.claude/hooks/lib_project_code_runs_launchers.py:961`）；参数取不出词的整行跳过：「if not written or written.startswith("="):」（同文件第 1134 行）。不在 `lines` 里的行既不进 `unrecognized`（同文件第 1186 行起那一段只看 `lines`），也不让 `same_content_as_own` 在「if reading.unrecognized:」（同文件第 1587 行）那里照不同判，就是静默跳过。

逐种（每种都能照喂：前面一行 `X="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"`，探针 forms 段的样本就是这几行）：

| 写法 | bash 真 source 吗 | 识别函数 | 不经识别函数的那一格（`LINE_INITIAL_SOURCE`，只数行首） |
|---|---|---|---|
| `readonly R=…` / `local F=…` 再 `source "$R/lib/a.sh"`、`. "${X}/lib/a.sh"`、`[[ … ]] && source …`、`then source …`、`(source … && f)` | 是 | 认出 `lib/a.sh`（推的） | — |
| `source <(cat "$X/lib/a.sh")`、`… && . <("$X/lib/a.sh" --print-environment)` | 是（读的是那条命令的输出，后一种还先跑了库） | 词取不出、整行跳过 | 行首那一种数得到，`&&` 之后那一种数不到 |
| `case "$MODE" in` / `  gpu) source "$X/lib/a.sh" ;;` | 是 | `)` 不在表里，跳过 | 数不到 |
| `QUIET=1 source "$X/lib/a.sh"` | 是 | 赋值前缀不在表里，跳过 | 数不到 |
| `builtin source …`、`command . …` | 是 | 跳过 | 数不到 |
| `elif source "$X/lib/a.sh"; then` | 是 | `\bif\b` 在 `elif` 里没有词界，跳过 | 数不到 |
| `eval "source \"$X/lib/a.sh\""` | 是 | 引号不在表里，跳过 | 数不到 |
| `bash -c 'source "$1" && f' child "$X/lib/a.sh"` | 是（子 shell 里） | 单引号不在表里，跳过 | 数不到 |
| `bash -c 'r=1; source "$1"' child …` | 是 | 看见，交 None（照不同判，或登记） | — |

最后两行是同一件事的两种写法：引号里的 shell 代码，source 前面是 `;` 就看见、是引号就看不见。登记表里 `("scripts/crash/crash-amplification-two-hosts.sh", '"$2"'): ""` 那一条正是 `bash -c 'repository_root="$1"; source "$2" …'`（`scripts/crash/crash-amplification-two-hosts.sh:910`）被看见之后登记的，同一形态换个次序就不登记也不比。

仓里现有的这一形（量过·模型，`silent-shapes.txt` 里 `quoted-shell-code` 17 行）：17 行里 16 行引的是自己、样本树里的拷贝，或同一入口另有直接 source 的那份库（`heavy-threads.sh`、`crash-amplification-lib.sh`），不漏比；剩下一行漏比：

- 「  if ! kb_listing="$(bash -c 'source "$1" && kb_directories "$2"' dated-status "$REPO_ROOT/.claude/singlefs-ai-sop/scripts/lib.sh" "$(pwd)" 2>"$registry_errors")"; then」（`.claude/gate.d/lib/doc-text-content-and-history-cells.sh:663`，门禁 doc-text 的 dated-status 格，平常的那一趟就跑它）。`doc-text.sh` 引的库里没有规范副本的 `lib.sh`（`lib.sh` 进库表是因为 `.claude/scripts/lkmm.sh` 那一条登记）：别处一份 `doc-text.sh` 连同它认出的库都与本仓相同、只有它那份 `.claude/singlefs-ai-sop/scripts/lib.sh` 不同，照按名字判。推的，待 family 段 `real-doc-text-sop-lib`（预言 got=True、契约要 False）与 `FAMILY-READING real-doc-text` 那一行（预言库表里没有 `../singlefs-ai-sop/scripts/lib.sh`）。

后果有多重：按名字判的门禁阶段照样只能经服务起，只读的几种用法（`--list-items` 这类）不跑 dated-status 那一格，所以这一处眼下没有一条直接跑的命令因此执行了没比过的库（推的）。打中的是 `same_content_as_own` 文档串那一句「    """别处那一份与本仓 location 那一份逐字节相同、它按自己所在目录 source / import 的库（script_source_reading）在别处那边有的也都相同：」（`.claude/hooks/lib_project_code_runs_launchers.py:1576`），与判决改法 1 的「不静默跳过」。

### 打中二：深度上限静默截掉库（python 那一支最明显）

「SOURCED_LIBRARY_DEPTH = 4」（`.claude/hooks/lib_project_code_runs_launchers.py:978`）与「if depth + 1 < SOURCED_LIBRARY_DEPTH:」（同文件第 1203 行）：第四层的库记进 `libraries`、正文不再读，它再引的库静默不进。`seen` 不含入口自己，`broad_mutation_base.py` 写着 `"broad-mutation.py"`，入口在第二层被当成库再读一遍、白占一层（量过·模型）：

```
CAP 4 libraries=12
  LIB depth=1 broad_mutation_base.py
  LIB depth=2 broad-mutation.py
  LIB depth=3 broad_mutation_judging.py
  LIB depth=4 mutation-prune.py
  LIB depth=3 broad_mutation_library.py
  LIB depth=4 crates-mutation-rows.py
```

（`python-depth-broad-mutation.txt` 前 7 行；模型的 12 份与库表里 12 份 `.py` 逐个相同，这是模型对得上真函数的一处校验。）`CAP 99` 那一半多出 17 行 `CUT`，头两行是「  CUT depth=5 lib_mutation_units.py」「  CUT depth=5 crates_mutation_rows_base.py」。这 17 份运行期确会载入：`scripts/mutation/broad_mutation_library.py:175` 起「    specification = importlib.util.spec_from_file_location("crates_mutation_rows_for_broad_mutation",」按文件载入 `crates-mutation-rows.py` 并 `exec_module`，那一份模块顶层就 `from crates_mutation_rows_base import (…` 这一串（`scripts/mutation/crates-mutation-rows.py:1243` 起）。别处一份 `broad-mutation.py` 连同 12 份都与本仓相同、只改了 `crates_mutation_rows_judging.py`，照按名字判。推的，待 family 段 `real-broad-mutation-deep-part`（预言 True）与对照 `real-broad-mutation-control-library`（预言 False）；depth 段对 36 份逐份列截掉的库（shell 那一支有没有被截的，读码没算全，交 depth 段）。

自证看不见：表那一格比的是表与识别函数读出的集合（两边都截了），数行那一格「            if counted_file.endswith(".py"):」跳过 python（`.claude/hooks/lib_project_code_runs_selftest.py:1097`）。

### 登记表 10 条：各是不是真认不出、登记了之后引的库比不比

| 登记（`.claude/hooks/lib_project_code_runs_base.py` 第 238 到 247 行） | 那一行在哪、为什么认不出 | 登记的去向对不对 |
|---|---|---|
| crash-case-cells.sh `"$library"` → `scripts/hosts/heavy-threads.sh` | 第 36 行 `(source "$library" && heavy_thread_default) 2>/dev/null`；`library` 在这一份里赋了两个值（第 34 行指 `heavy-threads.sh`，第 303 行指 `scripts/crash/lib_crash_point_replay_items.py`），「每一处赋值都解到同一个位置才认」不满足，真认不出 | 对这一行对；结构上有缺口，见下「打中三」 |
| feature-card-and-audit-cells.sh `in` → 空 | 第 428、669 行 awk 的 `for (source in count)`，`(` 之后的 `source` 被当成命令 | 对（不是库） |
| doc-experiments-preregistration-and-replay-cells.sh `is` → 空 | 第 1538 行嵌在 shell 里的 python `if source is None:` | 对 |
| doc-text-content-and-history-cells.sh `in` → 空 | 第 400 行嵌在里面的 python `… if source in by_context[context]` | 对 |
| `.claude/scripts/lkmm.sh` `"${PREFLIGHT_PROGRAM%/*}/lib.sh"` → 规范副本 `lib.sh` | 第 93 行；`${名%/*}` 只认 `BASH_SOURCE` 与 `0` 两个名字，真认不出 | 对：运行期 `PREFLIGHT_PROGRAM` 指规范副本 `preflight.py`，副本不在本树时取主仓那一份（git common dir），那一份就是本仓那一份，比不比结果一样（推的） |
| `.claude/scripts/preflight.sh` `"$project_preflight_library"` → 规范副本 `preflight.sh` | 第 20 行；`project_preflight_library` 赋了两次（第 11 行本树的规范副本、第 15 行 git common dir 那一层的），真认不出 | 同上：第二个值只在本树没有规范副本时取，取到的是主仓那一份（推的） |
| crash-amplification-lib.sh `"$2"` → 空 | 第 1077 行，自证里 heredoc 写出的外壳，`$2` 是这份库自己 | 对 |
| crash-amplification-two-hosts.sh `"$2"` → 空 | 第 910 行 `bash -c '…; source "$2" …'`，`$2` 是 `crash-amplification-lib.sh`；它是库，同一入口第 188 行另有直接 source，照比 | 结局对（另有一处比到），写法上把一行真 source 库的登记成「不是库」 |
| vm-bench.sh `/args` → 空 | 第 110 行，虚机里起的那段（`/bin/busybox poweroff -f` 下面）读虚机根下的 `/args` | 对 |
| vm-bench.sh `./args` → 空 | 第 288 行 `( cd "$scratch" && … && . ./args && … )`，读运行时写进临时目录的参数文件 | 对 |

### 打中三：登记按「文件、照写的参数」认，同一份里另一行同样写法的 source 被吞

查登记那一句是「library = None if own_file is None else OWN_SCRIPT_UNRECOGNIZED_SOURCES.get((own_file, written))」（`.claude/hooks/lib_project_code_runs_launchers.py:1186`），键里没有行号、也不数这样的行有几条；自证核登记那一格比的是集合：「(sorted(unregistered_sources), sorted(set(OWN_SCRIPT_UNRECOGNIZED_SOURCES) - registered_sources))))」（`.claude/hooks/lib_project_code_runs_selftest.py:1092`），数行那一格碰到登记过的 `(文件, 参数)` 就记成认出（同文件第 1080 行起那一圈）。能照喂的输入（探针 family 段 `registered-line-absorbs-second-library` 就是这一段，加在 `.claude/gate.d/lib/crash-point-replay-crash-case-cells.sh` 末尾）：

```
crash_point_replay_probe_peer_value() {
  local library="$crash_point_replay_stage_repository/scripts/hosts/peer-host-lib.sh"
  (source "$library" && peer_host_configured_value PROBE) 2>/dev/null
}
```

这一行照第 238 行登记认成 `heavy-threads.sh`，`peer-host-lib.sh` 对 `crash-point-replay.sh` 的拷贝不比；登记那一格、数行那一格、表那一格（`peer-host-lib.sh` 已因 `multi-host-run.sh` 在表里）都绿（推的）。同样的吞法对两条 `"$2"` → 空 更直接：`crash-amplification-lib.sh` 里以后任何一个函数写 `source "$2"` 都照「不是库」跳过。预言：family 段那一行 got=True、契约要 False，`FAMILY-READING` 那一行 `peer-library-read=False`。

### 误认：会不会把该判同族的判成不同、拒了该放的命令

不成立（推的）。理由：表那一格要求识别函数读出的每个位置都在库表里、「表里每一份都在本仓盘上」；此刻那一格绿，三张表里没有一处误认到盘上没有的文件。忠实的拷贝（仓副本、worktree）里那几份与本仓逐字节相同，误认到的文件也相同，不让它判成不同。会误认的形状是有的，只是没找到现役脚本这样写：

- `X+=/lib`（`SHELL_ASSIGNMENT` 只认 `名=`）之后 `source "$X/a.sh"`：认成 `a.sh`，bash 读的是 `lib/a.sh`；
- 先 `source "$L/a.sh"`、后面才赋 `L=…/lib`：认成 `lib/a.sh`，那一刻 `L` 其实来自环境；
- 库继承的是入口读完之后的当前目录，入口在 source 之后才顶层 `cd` 的，库里相对路径解到 cd 之后的目录。

这几种误认到盘上有的别的文件时，静默比错文件（漏比真库）；误认到盘上没有的，表那一格红。核法：selfsame 段把 36 份本仓那一份当别处那一份比，预言都交 True；forms 段 `plus-assignment`、`used-before-assignment` 两行预言 `a.sh`、`lib/a.sh`。

## 第 2 问：改法 2

### 表外的库一律判不同：有没有现役命令因此被拒

不成立（推的）。`same_content_as_own` 里「if own_digest is None or (os.path.lexists(copy_library) and file_sha256(copy_library) != own_digest):」（`.claude/hooks/lib_project_code_runs_launchers.py:1592`）只在别处那边读出表外的库时多判不同。别处那边读出的库取决于两样：正文（与本仓逐字节相同，shell 那一支的位置推算不看盘）与盘上有哪些文件（python 那一支照 `os.path.isfile` 认同目录的 `.py`、库读不开就不往下读）。忠实拷贝的文件是本仓的子集，读出的库也是子集、都在表里。核法：selfsame 段把三张表 36 份本仓那一份当别处那一份比（拷贝完整照本仓时的上界），预言 36 行都 `got=True`；有一行 False，就是现役脚本的忠实拷贝也被判成不同、掉成读正文，那一份的命令（`bash <副本>/<那一份> --list-items` 这类只读用法）就是要找的被拒命令。

另有一处次序依赖（量过·模型，不是现役）：python 那一支先深后广、`seen` 先到先占、深度到 4 截断，别处那边少一份中间文件时，后面的库会从更浅的路径被读到，于是读出表外的库、照不同判。`python-depth-without-base.txt` 里拿掉 `broad_mutation_base.py` 之后「  LIB depth=3 crates_mutation_rows_base.py」进了截断之内，后面 15 份都是表外的库。拿掉的是入口必载的那一份，那棵树本来就跑不起来，所以不算现役命令被拒；它说明的是「判定取决于别处那边的文件集合」这一点，与打中二同一个病根。selfsame 段 `broad-mutation-copy-without-broad_mutation_base.py` 预言 got=False、`SELFSAME-OUTSIDE-TABLE` 列出这十几份。

### 「先加 source 行、后建库」两步样本测的是不是判决说的那两步

部分成立。判决改法 2 原话「2. 库在不在盘上：表里点名而盘上没有的照「不同」判并让表那一格红，不拿 `lexists` 筛掉；先加 source 行后建库那一形两步各自的判定写成样本格。fixtures 目录在不在决定的那一格，改成不依赖那一处的存在性，或把它读成算键库收得进的形态。零轮。」（`research/prompts/hook-lib-foldable-reads-governance-r2-main-verification.md:35`）。

- 判定那一半测到了：样本仓（`own_repository_swapped` 把本仓那一边换成样本仓、库表加上 `TWO_STEP_LIBRARY`）里，库没建时逐字节相同的拷贝判不同、两边都建好之后判同族，两格各有开关点名（文件头开关表里 `same-family-by-path-tail` 点名前一格，`family-copy-content-unchecked`、`own-library-digest-unlisted`、`own-library-table-short`、`own-script-digest-stages-unlisted` 点名后一格）。
- 「让表那一格红」那一半没测到：表那一格比的第二项是「(sorted(compared_library_table), [listed for listed in OWN_SCRIPT_LIBRARY_FILES if file_sha256(os.path.join(HOOK_REPOSITORY, listed)) is None])))」（`.claude/hooks/lib_project_code_runs_selftest.py:1089`），读的是真仓，真仓里 63 份都在；两步样本只换了 `judged_by_name` 那两次调用的本仓那一边，没有喂到这一格。文件头开关表里点名这一格的两条（`own-library-table-short`、`script-directory-variables-skipped`）都只让第一项变，没有一条让「表里点名而盘上没有」出现。能照喂的变体：把那一项改成 `[]`，自证照绿（推的，读码；没在副本上跑）。

### fixtures 那一格换了做法之后还测不测得到原来那件事

成立（测得到）。原来那件事是第二轮判决记的「`.claude/gate.d/fixtures/crates-mutation-replay.sh/` 在不在决定自证一格、不进键」。被判的代码 `stage_selftest_fed_stages` 只看「            if stage.endswith(".sh") and not stage.startswith(".") and os.path.isdir(os.path.join(gate_directory, "fixtures", stage))」（`.claude/hooks/lib_project_code_runs_base.py:773`），再看阶段源码里有没有临时服务核对；新做法把两道阶段的源码按字面路径拷进自证自己的门禁目录、建 `fixtures/<阶段>/red`，喂的还是同一个函数、同一份源码，判定只随源码变。丢掉的只是「仓里那个目录此刻在」这一个盘面事实，那不是被判代码的性质（仓里样本档挪走了，裸写的那条命令本来就不该再拒）。同一个依赖在 `.claude/hooks/project_code_guard_selftest.py` 第 1081 行起那一格还在，归正推腿，这里不判。

## 第 3 问：改法 3 的分辨格与改法 4 的载入那一判

### 分辨格换等价写法躲不躲得开

那一格认「拷部分文件那一圈」只认 `for`：「        copy_loops = [node for node in ast.walk(own_source_tree) if isinstance(node, ast.For) and isinstance(node.target, ast.Name)」（`.claude/hooks/lib_project_code_runs_selftest.py:1287`）；数赋值只数模块顶层的 `ast.Assign`：「        copied_files_assignments = [node for node in own_source_tree.body if isinstance(node, ast.Assign)」（同文件第 1296 行）。期望值是 `(["PART_PROBE_COPIED_FILES"], 1, True)`。

| 变体（拷的文件多一份 `lib_writer_lock.py`） | 那一格 | 为什么 |
|---|---|---|
| `for` 换 `while`（先 `list(…)` 再 `pop`） | 红，躲不开 | 一个 `for` 都认不到，迭代名那一项成空表 |
| 换 `list(map(lambda …: shutil.copy(…), …))` | 红 | 同上 |
| 先赋给别的名字 `copied_names = PART_PROBE_COPIED_FILES + …` 再 `for … in copied_names` | 红 | 迭代名是 `copied_names` |
| 模块顶层另起一行 `PART_PROBE_COPIED_FILES += ("lib_writer_lock.py",)` | 绿，躲开 | `ast.AugAssign` 不是 `ast.Assign`，数出来还是 1、值还是字面元组 |
| 模块顶层 `PART_PROBE_COPIED_FILES: tuple = PART_PROBE_COPIED_FILES + …` | 绿 | `ast.AnnAssign` 同上 |
| 模块里一个函数 `global PART_PROBE_COPIED_FILES` 再赋、模块顶层调它 | 绿 | 只看 `own_source_tree.body` 那一层的赋值 |
| 原样留着那一圈，旁边另写 `[shutil.copy(…, part_probe_directory) for extra_name in …]` | 绿 | 推导式不是 `ast.For` |

都是推的（读码），探针 astcell 段从仓里那份自证的语法树里原样取出这一格的几条语句、对八份改过的正文各跑一次（预言照上表）。

躲开的四种在算键库那一边是什么后果，分两样（推的，没量键）：算键库登记模块级赋值时把 `ast.Assign`、`ast.AnnAssign`、`ast.AugAssign` 一起记（`scripts/gate/lib_function_units_python_module.py:201`），「只赋过一次」的取法见到两处就交 None（同文件第 1010 行起 `single_assignment`），所以 `+=` 与带注解的再赋多半让那一处折不出、拒算或连整个目录，不是静默漏键；`global` 再赋与旁边另拷那两种，算键库那一边怎么认我没读到（模块级只按顶层语句记、推导式另有一支 `loop_binding`），可能折成只有字面元组那 8 份、漏掉多拷的那一份。那一格的名字说的是「算键库折得出拷的是哪几份」，这两种若真漏键就是格与它保的东西一起漏。核法：主工作区上不能改文件，要在副本上对这两种变体各起一次 `closure`，看数据行里有没有 `lib_writer_lock.py`，交主 agent 定要不要做。

### 改法 4：漏过的真载入、仍被拒的只读命令

被判的判法：「TASK_SERVER_LIBRARY_FILE_LOAD = re.compile(r"\b(?:spec_from_file_location|SourceFileLoader|SourcelessFileLoader|run_path|load_source|execfile)\s*\(|\bexec\s*\([^;\n]*\bopen\s*\(")」（`.claude/hooks/lib_project_code_runs_base.py:946`）与「    if (not break_is_set("task-server-library-file-loads-unjudged") and TASK_SERVER_LIBRARY_FILE_NAME in text and TASK_SERVER_LIBRARY_FILE_LOAD.search(text)」（同文件第 1247 行）。判的是整段命令文本：调用名与文件名在同一段文本里出现就拒，不分它们在 python 代码里还是在 grep 的参数里；`exec(` 与 `open(` 要在同一段里（`[^;\n]*` 不跨分号、不跨行）。

量过·钩子：下面几条是我这一轮在自己的 Bash 里起的（都只读或只 echo，什么也不载入），现役项目代码闸当场判的原样输出。echo 那几条的命令文本里就是那段 python 代码，判的是同一个整段文本判法，所以 echo 放行等于这段文本过了这一判（python3 -c 真起时别的判法拒不拒，我没起，推的；loads 段直接调 `task_server_text_bypass` 核同一件事）。另一处同样被拒、也算一次量：我写这一节时，把含调用名的正文与含文件名的替换写在同一条 Bash 里，被同一句拒绝说明拒了，这一节是拆成两条写进去的。

仍被拒的只读命令（打中）：

- `grep -n 'spec_from_file_location(' task-server/task_server_lib.py; echo rc=$?` → 被拒，stderr 首行「PreToolUse:Bash hook error: [bash "$CLAUDE_PROJECT_DIR"/.claude/hooks/project-code-guard.sh]: ✗ 绕开 task-server 的写法被拒：在代码里按文件载入 task-server/task_server_lib.py（它管库与签名密钥）」，末行「→ 怎么办：改用 task-server/task 的那几个子命令或 task-server/control；…命令里只是搜这几个名字的（grep 库文件名这类），把名字拆开写（`grep -n 'task-server\.sqlite'`）。」（中间省略的两句与下一条逐字相同）。
- `grep -nF "runpy.run_path('task-server/task_server_lib.py')" .claude/hooks/lib_project_code_runs_selftest.py; echo rc=$?` → 被拒，stderr 与上一条逐字相同。这一条是照三方腿的引文规矩（引文先 `grep -nF` 原文一次）核自证第 1228 行那条样本时起的：引文核对本身就被拒，只能拆开字符串再 grep。

漏过的真载入（打中）：

- `echo "python3 -c \"src = open('task-server/task_server_lib.py').read(); exec(src)\""; echo rc=$?` → 放行，输出「python3 -c "src = open('task-server/task_server_lib.py').read(); exec(src)"」「rc=0」。对照 `echo "python3 -c \"exec(open('task-server/task_server_lib.py').read())\""` → 被拒，首行同上。
- 喂给 `python3 -` 的 heredoc，两行 `code = compile(open('task-server/task_server_lib.py').read(), 'lib', 'exec')`、`exec(code)`，echo 那一段 → 放行，`rc=0`。
- `echo "python3 -c \"import sys; sys.path.insert(0, 'task-server'); import json, task_server_lib\""` → 放行，`rc=0`（`\bimport\s+task_server_lib\b` 认不出逗号后面的那个名字）。
- `echo "python3 -c \"import runpy, sys; sys.path.insert(0, 'task-server'); runpy.run_module('task_server_lib')\""` → 放行，`rc=0`（`run_module` 不在调用名单里，文本里也没有文件名）。

收窄前后对比（推的，loads 段 `LOADS-PREVIOUS` 行核）：收窄之前那一支是 `task_server_lib\.py['\"]`（任何带引号的文件名），前两种（先读进变量、compile 之后）照拒，这一改之后放行，是收窄带进来的；逗号 import、`run_module`、路径拼出来再交给按文件载入的，收窄前后都漏，原来就有。正文点名的几种里 `importlib.machinery` 的 `SourceFileLoader(`、`imp.load_source(`、`__import__('task_server_lib')` 照拒（推的，loads 段核）；`python3 task-server/task_server_lib.py` 直接起由「跑本仓的代码」那一判拒（`project_code_use` 认文件真实路径在本仓里，推的）；`PYTHONPATH=task-server python3 -m task_server_lib --selftest` 在仓根起时，词层交出的是 `task_server_lib.py`（`.claude/hooks/lib_project_code_runs_base.py:921` 那句注释「# python3 -m 起单测的模块（lib_shell_words 交出来的是 <模块>.py）」），`launched_file` 从当前目录解、仓根下没有这一份，交 None，文本里又没有调用名，推的是放行，loads 段 `module-run-from-repository-root` 核。

## 打中之后的四句

| 打中 | 分不分辨「改前 / 改后」 | 被判代码当时看不看得到判别它的东西 | 满足的是判决哪一句的哪个分句 | 判决给的改法在这一格上还中不中 |
|---|---|---|---|---|
| 一 命令位置之外的 source | 一半分辨：改前那条正则在 `(?:^|[;&|\s])` 之后认，`)` 之后、赋值前缀、`builtin` / `command`、`elif` 这四种接按自己所在目录的路径时改前认得、改后跳过（退回，forms、family 两段的 `PREVIOUS` 行核）；`<(…)`、`eval`、引号里的 shell 代码改前改后都跳过（残留） | 看得到：那一行就在它读的正文里 | 改法 1「认不出的 source 行让那一份脚本照「读正文」判（不按名字判同族），不静默跳过」的「不静默跳过」；表那一格的「另加一条不经识别函数的核法」只数行首，够不着 | 改法 1 照现在的写法还中（它修的是变量那一形） |
| 二 深度上限 | 不分辨：改前的递归也有 `SOURCED_LIBRARY_DEPTH`；残留 | 看得到：截断的那一层它手里有路径 | 同上「不静默跳过」；改法 1 验收「库表补全」（这 17 份不在表里） | 还中 |
| 三 登记吞行 | 分辨：登记表是改法 1 新加的 | 看得到：行号就在 `lines` 里 | 改法 1「认不出的 source 行…不静默跳过」：登记过的写法以后的新行被当成登记过 | 是改法 1 自己带进来的 |
| 第 2 问 表那一格的盘面一半 | 分辨：那一半是改法 2 新加的 | 看得到 | 改法 2「并让表那一格红」：有判法、没有一条样本或开关证明它会红 | 改法 2 的样本格只盖住判定那一半 |
| 第 3 问 分辨格躲得开 | 分辨：格是改法 3 新加的 | 看得到：同一棵语法树里 `AugAssign` 与推导式都在 | 改法 3「核拷文件那一圈只照 `PART_PROBE_COPIED_FILES` 迭代、它只赋一次」的「只赋一次」（只数 `Assign`）与「那一圈」（只认 `for`） | 正文点名的三种它挡住了，另外四种挡不住 |
| 第 3 问 改法 4 | 分辨：先读进变量再 exec 那两种是收窄带进来的；只读 grep 仍被拒是收窄没收到 | 看得到：判法手里是整段命令文本，切词层本来就分得出 grep 的参数与喂给 python 的代码 | 改法 4「`grep`、`sed -n`、`wc` 这类只读命令里出现文件名不拒」与「收窄到真载入的写法」两句各中一处 | 改法 4 的两条新样本（只读 grep 不带调用名、真载入写在同一段）都避开了打中的那几格 |

## 我提的改法（只在我的模型上量过、被攻过零轮；下表每格都是推的，没实现、没跑）

| 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| 甲：识别函数在命令位置之外见到 `source` / `.` 后接一个词（注释外、不跟 `=`）也交回一行、位置交 None；`<(…)` 不再 `continue`，照认不出交回。不经识别函数的那一格改数全部这样的出现、不只数行首 | 打中一（六种写法与 `doc-text` 那一行都落到「认不出」，要么登记、要么照不同判） | 推的，待主 agent 在副本上实现再跑 forms、family 段 |
| 乙：`seen` 先放进入口自己；深度到顶时把剩下没读的库照「认不出」交回（照不同判），或去掉深度上限只靠 `seen` 防环；python 那一支另加一条不经识别函数的核法（逐份 `.py` 数同目录的 import 与按文件载入的 `spec_from_file_location(…, <同目录文件>)`） | 打中二（`broad-mutation.py` 那 17 份进表、开始比） | 推的，待 depth 段 |
| 丙：登记表的键改成「那一份文件、行号、照写的参数」，或保留现在的键而在自证里数每个键在正文里命中几行、与登记的行数相等 | 打中三 | 推的 |
| 丁：加一条弄坏开关（或样本库表），让表那一格比的库表里多一项盘上没有的，证它的第二项会红 | 第 2 问表那一格 | 推的 |
| 戊：那一格数模块里对 `PART_PROBE_COPIED_FILES` 的每一处绑定（`Assign`、`AnnAssign`、`AugAssign`、函数里 `global` 之后的赋值），并核整棵树里目标含 `part_probe_directory` 的拷贝调用都在那一圈里 | 第 3 问分辨格 | 推的，待 astcell 段在实现之后重跑 |
| 己：按文件载入那一判只看喂给 python 的代码（`-c` 的代码、喂给 `python3 -` 的正文），在那段代码里文件名与 `open` / `compile` / `exec` / 载入调用任一同在就拒（不要求同一段）；另认 `import a, <库名>` 与 `run_module`；grep / sed / wc / cat 的参数不进这一判 | 第 3 问两格 | 推的，待 loads 段在实现之后重跑 |

## 要主 agent 跑的命令

六条互不依赖、次序不限，都在草稿目录起；探针只调主工作区共用库再导出的函数（`lib_project_code_runs.<名>`，`own_repository_swapped` 换本仓那一边时只在 `with` 里），样本树建在 `<草稿目录>/probe-work/<段>/`、段末删掉；不改主工作区的文件、不起 cargo、不起门禁与自证、不读写判定库、不算键。退出码：0 预言逐条都中，1 有 `MISMATCH` 行，2 探针自己坏了（看 `PROBE-BROKEN` 行）。带 ` HIT` 的行是读数与被判对象的契约不同（打中）。估时推的：每段几秒到一分钟以内（最重的是 family 段建十来棵小样本树）。

1. `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r3-opus -- python3 /home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r3-opus/probes/probe_r3.py /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r3-opus forms`
   - 在哪起：`/home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r3-opus`。
   - 看：末行 `TASK_SERVER_RESULT` 的 `exit=`；`task-server/task log <号> --all` 里 19 行 `FORM`。第 1 问打中一看 `process-substitution`、`case-arm`、`environment-prefix`、`builtin-source`、`command-dot`、`elif-condition`、`eval-string`、`child-shell-string-head` 八行预言 `got='SILENT'`；`FORM-PREVIOUS` 行里 `case-arm`、`environment-prefix`、`builtin-source`、`command-dot`、`elif-condition` 的改前正则读数（这几行的样本走的是变量 `$X`，改前那条正则认变量形还要过它的变量过滤，这一列只当线索）；误认看 `plus-assignment` 预言 `'a.sh'`、`used-before-assignment` 预言 `'lib/a.sh'`。
2. `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r3-opus -- python3 /home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r3-opus/probes/probe_r3.py /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r3-opus family`
   - 在哪起：同上。
   - 看：`appended-control-recognized` 预言 `got=False`（对照，样本法本身判得出不同）；另六行 `appended-*` 预言 `got=True HIT`，对应的 `FAMILY-PREVIOUS` 行里 `environment-prefix`、`builtin-source`、`case-arm`、`elif-condition` 预言 `['lib/two-step-library.sh']`（改前认得，退回的证据），`process-substitution`、`child-shell-string-head` 预言 `[]`；`registered-line-absorbs-second-library` 预言 `got=True HIT`、`peer-library-read=False`；`real-doc-text-sop-lib` 预言 `got=True HIT`、对照 `real-doc-text-control-stage-cells` 预言 `got=False`，`FAMILY-READING real-doc-text` 一行里没有 `../singlefs-ai-sop/scripts/lib.sh`；`real-broad-mutation-deep-part` 预言 `got=True HIT`、对照 `real-broad-mutation-control-library` 预言 `got=False`。
3. `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r3-opus -- python3 /home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r3-opus/probes/probe_r3.py /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r3-opus depth`
   - 在哪起：同上。
   - 看：36 行 `DEPTH-ENTRY`；`DEPTH-CUT scripts/mutation/broad-mutation.py …` 预言 17 行、都 `in-table=False`；`DEPTH broad-mutation.py-libraries-cut-by-depth` 预言 `got=17 HIT`；别的入口有没有 `DEPTH-CUT` 我没预言（shell 那一支没算全），有的就是打中二在 shell 那一支的实例；末行 `DEPTH-SUMMARY`。
4. `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r3-opus -- python3 /home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r3-opus/probes/probe_r3.py /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r3-opus selfsame`
   - 在哪起：同上。
   - 看：36 行 `SELFSAME <位置>` 预言都 `got=True`（有一行 False 就是第 2 问要找的现役被拒，我的「不成立」撤回）；`broad-mutation-copy-without-broad_mutation_base.py` 预言 `got=False HIT`、`SELFSAME-OUTSIDE-TABLE` 列出十几份 `crates_mutation_rows_*.py`；`…-without-broad_mutation_selftest_a.py` 预言 `got=True`。
5. `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r3-opus -- python3 /home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r3-opus/probes/probe_r3.py /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r3-opus loads`
   - 在哪起：同上。
   - 看：13 行 `LOADS`：真载入的前六行预言 `got=False HIT`，`machinery-loader`、`imp-load-source`、`dunder-import` 预言 `got=True`；只读的 `grep-loader-call`、`grep-fixed-selftest-sample`、`sed-loader-lines` 预言 `got=True HIT`，`grep-plain-name` 预言 `got=False`；`LOADS-PREVIOUS` 行里 `open-read-then-exec`、`compile-then-exec-heredoc`、`exec-open-split-line` 预言 `previous-regex-judged=True`（收窄前照拒，退回的证据），`comma-import`、`run-module`、`path-joined-spec` 预言 False；三行 `LOADS-PROJECT`：`direct-run` 与 `module-run-inside-task-server` 预言 True，`module-run-from-repository-root` 预言 `got=False HIT`。
6. `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r3-opus -- python3 /home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r3-opus/probes/probe_r3.py /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r3-opus astcell`
   - 在哪起：同上。
   - 看：8 行 `ASTCELL`：`baseline` 预言 `got='green'`（原码取对了的对照）；`while-replacement`、`renamed-iteration`、`map-replacement` 预言 `got='red'`；`module-augmented-assignment`、`helper-global-rebinding`、`module-annotated-reassignment`、`comprehension-extra-copy` 预言 `got='green' HIT`。

结果反过来我照它改结论：forms 段那八行有一行不是 `SILENT`，那一种撤出打中一；family 段 `appended-*` 六行有 `got=False` 的同样撤；`real-doc-text-sop-lib`、`real-broad-mutation-deep-part`、`registered-line-absorbs-second-library` 交 False 的，对应那一处撤回；astcell 段四个预言绿的变体红了，第 3 问分辨格那一半撤回；loads 段真载入的几行 `got=True` 的，那一种撤出漏过的名单。

## 没打中的形状

- 正文点名的新脚本写法里，`readonly X=…`、`local X=…`、`. "${X}/…"`、函数里 source、`[[ … ]] && source`、`then source`、`(source …)` 都认得出（读码推的，forms 段前六行核）；`source <(…)` 是正文点名里唯一静默跳过的。
- 变量赋值认不出的写法（`${X:=…}`、`${X:-…}`、`$(readlink -f …)`、`cd -- …`、行尾续行）都落到「认不出」，照不同判或要登记，不静默（读码）。
- 登记表 10 条逐条都是真认不出、去向都对（上面那张表）；打中的是键的写法，不是哪一条登记错。
- 误认让忠实拷贝判成不同、拒了该放的命令：没找到（推的，selfsame 段核）。
- 表外的库让现役脚本掉成读正文：没找到（推的，selfsame 段核）；次序依赖只在坏了的拷贝上出现。
- fixtures 那一格：换做法之后判的还是同一个函数与同一份阶段源码，没找到它测丢了被判代码的哪一样。
- 分辨格对正文点名的 `while`、`map`、先赋给别的名字三种都红。
- 改法 4 对 `importlib.machinery` 的 `SourceFileLoader`、`imp.load_source`、`__import__` 照拒（推的，loads 段核）；直接起 task-server 那份库由「跑本仓的代码」那一判拒（推的）。
- 取样范围：shell 写法 19 种样本（forms 段）、7 种接在门禁阶段末尾的写法与 3 处仓里现有的入口（family 段）、三张表 36 份逐份（depth、selfsame 段）、13 条命令文本（loads 段）、8 个变体（astcell 段）；仓里现有写法的扫描是三张表与库表里 86 份 shell 文件（`silent-shapes.txt` 末行 `FILES 86`）。

## 这条腿自己的限度

- 探针一段都没跑：除了「量过·钩子」那几条（我自己 Bash 里被现役钩子当场判的）与「量过·模型」那两份不 import 本仓代码的脚本，判定都是推的、预言写在探针里，等主 agent 跑出。
- 「量过·钩子」的 echo 那几条只证整段文本判法那一支放行，真起 `python3 -c` 时别的判法拒不拒我没起。
- python 深度那一条靠的是我自己照抄判法写的模型；模型读出的 12 份与库表逐个相同是一处校验，不是独立路径；depth 段用真函数核。
- shell 那一支的深度截断没算全，只读码看了几条最深的链（门禁阶段 → `lib/*-cells.sh` → `heavy-threads.sh` → `.claude/scripts/preflight.sh` → 规范副本 `preflight.sh`，第五层那一份没有 source 行，此刻不漏）。
- 键那一半一样没量：躲开分辨格的四种变体在算键库那边漏不漏键，是读算键库代码推的；要在副本上起 `closure` 才核得动。
- 打中一、二的「后果」只推到判定层：按名字判的那几份在只读用法里跑不跑到没比过的库，我只对 `doc-text.sh` 的 dated-status 那一格推了「眼下不跑到」，`broad-mutation.py` 的只读用法载不载 `crates-mutation-rows.py` 没查。
- 这是这一题最后一轮；这里提的改法甲到己都是零轮。

## 没做什么

- 没建仓副本、没建编译目录、没往统一任务服务交任务；判决要引的数没复跑（命令在「要主 agent 跑的命令」）。
- 没判正推腿的第 4、5 问，也没判 `.claude/hooks/project_code_guard_selftest.py` 第 1081 行起那一格（归正推腿）。
- 没喂钩子（整条命令经 `project-code-guard.sh` 判），只调了共用库里的判法函数；没起共用库自证与开关复跑。
- 草稿目录里我建的 `scratch/order/`（31 份 `scripts/mutation/*.py` 的拷贝，给模型拿掉一份文件用）已删（删前 `du -sh` 2.4M）；探针、扫描脚本、模型与它们的输出、`baselibs/` 留着给主 agent 复跑，写进 `.deliverables`。
