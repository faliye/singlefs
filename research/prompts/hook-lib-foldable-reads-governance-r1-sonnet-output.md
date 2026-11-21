# 钩子共用库折得出读法：治理文件轮第一轮正推腿报告（hook-lib-foldable-reads-governance-r1，2026-11-21）

腿：`three-way-forward`。负责正文第 4、5 问。没写模型（两问都是读文件与只读命令能答的），所以没有 `rerun.sh`。

读到的范围（两条腿都读「范围」里的文件，只有我读的另列）：

- 被判的 5 份：`.claude/gate.d/break-switch-replay-files.tsv`、`.claude/hooks/lib_project_code_runs.py`、`lib_project_code_runs_base.py`、`lib_project_code_runs_launchers.py`、`lib_project_code_runs_selftest.py`。读的是主工作区此刻的版本，`sha256sum` 与 `research/prompts/hook-lib-foldable-reads-governance-r1-snapshot/judged-sha256.txt` 五行逐行相同（量的）。
- 改前原样：工具员草稿 `~/.local/share/singlefs/work/hook-lib-foldable-reads/base/`（下称 `base/`）。我另核了它与 git 暂存区那一版逐字节相同（量的，见 4.0）。
- 只有我读的：工具员报告 `result.md`（下称 `result.md:行`）、`change.diff`、`hook-spec*.json`、`swap-*/` 里的日志，以及算键库 `scripts/gate/lib_function_units*.py` 里与这三处读法有关的段落。

行号约定：「主」是主工作区此刻的行号，「改前」是 `base/` 里的行号。引文都是在被引文件里 `grep -nF` 命中后取的行。

## 第 4 问：「实现今天的样子」逐条

### 4.0 先对一遍「今天」与「改前」各是哪一版

- 开工时主工作区四份与草稿 `new/` 逐字节相同：`cmp` 四次都无输出（量的）。做到一半，`lib_project_code_runs.py`、`_base.py`、`_selftest.py` 三份又被别的改动动过（`sha256sum` 与 `judged-sha256.txt` 不再相同，`_launchers.py` 与 `break-switch-replay-files.tsv` 仍相同），我拿 `diff <new/ 那份> <主工作区那份>` 看了：差只有三处，都是去掉 `gpu-verification-server/control serves` 这一项（`lib_project_code_runs.py` 开关表一句、`_base.py` 的名单表一条加两行注释、`_selftest.py` 的一格「名单：主 agent 的 gpu-verification-server/control serves」），与这一批的改法无关（量的）。这一批的五份改动没被碰；`_base.py` 里 `:852` 之后的行号比开工时小 1，`_selftest.py` 里 `:952` 之后的行号小 2，下面引的行号都是改后重取的。
- `base/` 四份与 git 暂存区里那一版的 sha256 相同（量的，`git show :<路径> | sha256sum` 对 `sha256sum <base 路径>`，前 16 位）：`lib_project_code_runs.py` `aa1a4672f6e96a73`、`_base.py` `43e70797259e209b`、`_launchers.py` `eb69b9cb2e88b96f`、`_selftest.py` `50c1adae4aad8168`。所以「改前」就是暂存区那一版。
- 主工作区相对暂存区的差，四份加起来 `git diff --stat` 是 80 增 14 删（16+28+8+42 行，量的），与 `result.md` 贴的一致；逐份把我自己 `diff -u` 出来的增删行去掉头部后与 `change.diff` 里同一份的增删行比，四份都相同（量的）。
- 这一批在 `.claude/gate.d/break-switch-replay-files.tsv` 里只动一行（`lib_project_code_runs.py` 那一行）：`git diff -U0` 列出该表 5 行改动，其余 4 行是别的文件的开关数，不是这一批的（量的）。

### 4.1 起因：「换上之前打出三行整目录读、data-file 7 个」

分三小条：

**4.1a 成立（改前的文件）；今天的文件里已经不是这样写了。**
正文说 `lib_project_code_runs_launchers.py` 取本仓那一份的 sha256 用运行时拼出的 `os.path.join(HOOK_REPOSITORY, location)`。

- 改前 `lib_project_code_runs_launchers.py:1366`：`    own = os.path.join(HOOK_REPOSITORY, location)`；改前 `:1368`：`    if digest is None or digest != file_sha256(own):`
- 主 `lib_project_code_runs_launchers.py:1368` 仍有 `    own = os.path.join(HOOK_REPOSITORY, location)`，但比内容那一行已是 `:1370`：`    if digest is None or digest != own_script_digest(location):`；`own` 只剩库那一半用它取目录（`:1374` 起）。

**4.1b 部分成立。**
正文说 `location` 由调用方给「折得出的目录 + 折不出的一截」。`same_content_as_own` 的调用方一共三处（`grep -rn same_content_as_own --include=*.py --include=*.sh .` 除登记、说明文字外只命中这三处）：

- `lib_project_code_runs_launchers.py:1329-1330`：`os.path.join(GATE_STAGE_DIRECTORY, stage)`，目录折得出、`stage` 折不出：与正文相符。
- `lib_project_code_runs_launchers.py:485`：`    location = os.path.join(DISPATCH_LAUNCHER_DIRECTORY, os.path.basename(words[0]))`，同上：与正文相符。
- `lib_project_code_runs_launchers.py:1354`：`    location = KNOWN_SCRIPT_LOCATIONS.get(os.path.basename(word))`，是字典 `get`、不是「目录 + 一截」。正文这一句只盖得住前两处；第三处折不出的原因不同（`result.md:7` 写的就是这一点，正文把三处并成一句了）。

**4.1c 成立（改前的文件）。**
正文说自证拷文件的源是 `os.path.join(part_probe_source, copied_name)`，`copied_name` 取遍一个折不出的元组相加。

- 改前 `lib_project_code_runs_selftest.py:1017`：`        part_probe_source = os.path.dirname(os.path.abspath(__file__))`
- 改前 `:1029`：`                for copied_name in ("lib_project_code_runs.py", "lib_shell_words.py", "lib_shell_words_core.py", "lib_shell_words_embedded.py", PROJECT_CODE_RUNS_BOOTSTRAP_FILE_NAME) + PROJECT_CODE_RUNS_PART_NAMES:`；改前 `:1030`：`                    shutil.copy(os.path.join(part_probe_source, copied_name), part_probe_directory)`
- 「折不出」的依据：算键库 `scripts/gate/lib_function_units_python_module.py:1219-1220`：`        if not isinstance(sequence, (ast.Tuple, ast.List)):` / `            return None`，两个元组相加是 `ast.BinOp`，走不到元组那一支（量的，读代码）。
- 今天 `lib_project_code_runs_selftest.py:1057-1058`：`                for copied_name in PART_PROBE_COPIED_FILES:` / `                    shutil.copy(os.path.join(part_probe_source, copied_name), part_probe_directory)`。

**4.1d 推不出（三行整目录读与「data-file 7 个」是任务服务里的输出，只读够不着）。**

- 记录里确有这句：`records/2026-11-17-subagent账本清欠调度记录.md:557` 写「闭包对照（`selftest-process-guard.py::watch_judgement`）：换前 via 整目录 `.claude/gate.d`、`.claude/hooks`、`scripts/hosts` 三行、data-file 7 个，换后 0 行、data-file 48 个。」与正文一致（量的，grep 命中）。
- 但算键库在这之后被改过：`find scripts/gate -maxdepth 1 -name 'lib_function_units*.py' -newer <result.md>` 点名了 `lib_function_units.py` 与 `lib_function_units_python_data_reads.py`（及两份自证夹具、自证入口）（量的）。`git diff -- scripts/gate/lib_function_units_python_data_reads.py` 里新增了 `LISTING_CALLS`、`listed_directories`、`directory_listing_derived`，并把「前缀是根里的目录、折不出的一截一律收成整个目录」改成：
  - 主 `scripts/gate/lib_function_units_python_data_reads.py:549-551`：`                whole = listed is None or any(self.directory_listing_derived(part, unit, listed & {relative}, depth + 1) for part in node.args[folded:])` / `                out.add(("directory", relative) if whole else ("refuse",))`
  - 推的（没跑）：用今天的算键库去算改前的代码，`file_sha256` 的那两处（门禁阶段、派发起法的 `location`）的折不出一截不是从目录清单里挑的，会落到 `("refuse",)`，记拒算（退 4）；自证拷文件那一处是拷文件的源，`lib_function_units_script_closure.py:548` 的 `        elif kind == "refuse" and not copied:` 免它拒算，既不收也不拒算。都不再是三行整目录读。「换前三行整目录」只在算键库改之前的版本上出现过；我没法从只读命令确认主 agent 取那一趟读数时算键库是哪一版。
  - 出路：「要主 agent 跑的命令」第 1 条在今天的算键库上重算一遍换上之后的闭包，看退出码与拒算行。

### 4.2 改法五条

**4.2a `lib_project_code_runs_base.py` 加 `GATE_STAGE_FILES`（17 道）、`DISPATCH_LAUNCHER_SCRIPTS`、`own_script_digest`：成立。**

- 主 `lib_project_code_runs_base.py:60`：`GATE_STAGE_FILES = (`，到 `:65` 收尾，里面 17 个文件名。`ls .claude/gate.d | grep -E '\.sh$'` 也是 17 个，两边排序后 `diff` 无输出（量的；两份清单落在我的草稿目录 `tuple-stages.txt`、`disk-stages.txt`）。所以正文说的新自证格「GATE_STAGE_FILES 与门禁目录顶层的阶段逐个相同」此刻是绿的。
- 主 `:199`：`DISPATCH_LAUNCHER_SCRIPTS = (MULTI_HOST_RUN_SCRIPT,)`。元素是名字 `MULTI_HOST_RUN_SCRIPT`（`:194`），不是字面串；算键库能折（模块级只赋一次的常量）。正文把它与 `GATE_STAGE_FILES` 并称「字面元组」，说的是形状，成立。
- 主 `:504` 起 `own_script_digest(location)`，三张表逐个比的三段：`:509` `    for name in KNOWN_SCRIPT_LOCATIONS:` / `:513` `        for stage in GATE_STAGE_FILES:` / `:516` `    for launcher in DISPATCH_LAUNCHER_SCRIPTS:`，表外 `:519` `    return None`；开关在 `:512` `    if not break_is_set("own-script-digest-stages-unlisted"):`，只包阶段那一段。「挑中就现读」成立：三处 `return file_sha256(os.path.join(HOOK_REPOSITORY, …))`（`:511`、`:515`、`:518`）都在调用时现读，没有缓存装饰器。
- 一个细节：`KNOWN_SCRIPT_LOCATIONS`（`:78-90`）的键里有名字 `STAGE_SELFTEST_SCRIPT`（`:89`），不是字面串；值 18 个全是字面串（我从 `:78-90` 取出带 `/` 的串去重数得 18，量的）。三张表的「字面」指值，正文没说错。

**4.2b `lib_project_code_runs_launchers.py` 比内容改成 `digest != own_script_digest(location)`，库那一半原样：成立。**

- 主 `:1370`：`    if digest is None or digest != own_script_digest(location):`；`:23` 的 import 行加了 `own_script_digest`。
- 库那一半 `:1372-1376` 原文：`    for relative in script_sourced_libraries(path):` … `            return False` / `    return True`，其中 `:1374` 的比较仍是 `file_sha256(copy_library) != file_sha256(os.path.normpath(os.path.join(os.path.dirname(own), relative)))`。我拿 `diff -u base/…launchers.py 主/…launchers.py` 看：只有导入行、文档串、`:1370` 三处不同（`+5 -3`，量的），库那一半逐字相同。

**4.2c `lib_project_code_runs_selftest.py` 拷文件循环元组与六格：成立。**

- 主 `:131-134`：`PART_PROBE_COPIED_FILES = (` 起，8 个 `.py` 名（从这四行里 `grep -o '"[^"]*\.py"'` 数得 8，量的）；循环在 `:1057`。次序与改前 `:1029` 那条表达式相同（入口、`lib_shell_words` 三份、引导那一份、三份部分文件），所以拷的东西没变，变的只是算键库折得出。
- 六格：主 `:870`（门禁阶段拷贝格）、`:873`（派发拷贝格）、`:882`（阶段名单相同格）、`:885`（按位置挑得到格）、`:887`（表外交 None 格）、`:1053`（引导拷贝格）。`grep -c 'results.append'` 主 84、改前 78（量的），增 6。

**4.2d `lib_project_code_runs.py` 文件头函数表、开关表加新项并再导出：成立。**

- 函数表 `:30` `  own_script_digest(location) -> str | None`；开关表 `:211` `  LIB_PROJECT_CODE_RUNS_BREAK=own-script-digest-stages-unlisted  own_script_digest() 不从 GATE_STAGE_FILES 里挑…`；`family-copy-content-unchecked` 那条补点名的两格在 `:180-182`。
- 再导出：`:386`（`DISPATCH_LAUNCHER_SCRIPTS`）、`:388`（`GATE_STAGE_FILES`）、`:407`（`own_script_digest`）、`:448`（`PART_PROBE_COPIED_FILES`）。

**4.2e `.claude/gate.d/break-switch-replay-files.tsv` 第 70 行改八十三条：成立。**
`sed -n 70p` 是 `.claude/hooks/lib_project_code_runs.py	60	# 八十三条开关（…`，暂存区那一版同一行是 `八十二条开关`（量的）。

### 4.3 「工具员报告写的判定变化只有两处」

成立（静态读码；我没能起样本去喂，这一条的推翻条件见各条末尾）。

我自己逐个调用点看了 `own_script_digest` 取代的那一处（`file_sha256(own)`，`own = join(HOOK_REPOSITORY, location)`）：

- `same_content_as_own` 只有三个调用点（4.1b 列的 `:485`→`:489`、`:1330`、`:1354`→`:1359`），除去文字说明外，全仓 `.py`、`.sh` 没有别的调用（`grep -rn 'same_content_as_own\|own_script_digest' --include=*.py --include=*.sh .`，量的）。
- 三个调用点的 `location` 要么出自 `KNOWN_SCRIPT_LOCATIONS` 的值（`:1354` 的 `.get(…)` 返回值，必在第一段挑得中），要么是 `.claude/gate.d/<阶段>`、`scripts/hosts/<basename>`。第一段挑中时读的路径与改前相同（`os.path.join(HOOK_REPOSITORY, <那个值>)`）；阶段挑中时 `os.path.join(HOOK_REPOSITORY, GATE_STAGE_DIRECTORY, stage)` 与改前 `os.path.join(HOOK_REPOSITORY, location)` 在 `location == os.path.join(GATE_STAGE_DIRECTORY, stage)` 时是同一路径；派发那一段同理。`HOOK_REPOSITORY` 全仓只在 `lib_project_code_runs_base.py:159` 赋一次（`grep -rn 'HOOK_REPOSITORY *=' .claude/hooks/ scripts/ task-server/*.py`，量的），没有哪处在自证里换掉它，所以 `base.py` 里的 `own_script_digest` 与 `launchers.py` 里改前读的是同一个根。
- 所以判得不同的只有「`location` 不在三张表里、而改前的 `own` 在盘上有那份文件」这一类。盘上有的文件里，属于这一类的有：门禁目录顶层新加的、没进 `GATE_STAGE_FILES` 的阶段；`scripts/hosts/` 下名字不是 `multi-host-run.sh`、也不在 `KNOWN_SCRIPT_LOCATIONS` 值里的脚本。前一类是工具员写的第二处。后一类是第一处，但工具员的写法偏宽：「`DISPATCH_LAUNCHER_OPTIONS` 之外的 basename」里的 `capped.sh`、`run-with-memory-cap.sh` 在 `KNOWN_SCRIPT_LOCATIONS` 的值里（`lib_project_code_runs_base.py:82`），第一段仍挑得中，判法不变；真正改交 False 的是既不在 `DISPATCH_LAUNCHER_OPTIONS` 也不在那张表里的名字。
- 「现有调用方走不到」：派发那一支只有一个调用方 `project_code_guard_decisions.py:1051`：`                inner_capped = inner_capped or project_code_runs.dispatched_under_memory_cap(command.words, command.directory)`，它在 `:1050` `            if launched is not None:` 之内，`launched` 来自 `:1049` 的 `command_under_dispatch_launcher`，那个函数在 `lib_project_code_runs_launchers.py:234` `    if name not in DISPATCH_LAUNCHER_OPTIONS or not located_in(words[0], directory, DISPATCH_LAUNCHER_DIRECTORY):` 后面才往下走。`DISPATCH_LAUNCHER_OPTIONS` 只有 `multi-host-run.sh` 一个键（`lib_project_code_runs_base.py:195-197`），所以进来的 `basename` 必是它，`location` 必是 `scripts/hosts/multi-host-run.sh`，第三段挑得中。自证里另有三处直接调（`lib_project_code_runs_selftest.py:799`、`:801`、`:873`），文件名都是 `multi-host-run.sh`。`result.md:13` 引的 `lib_project_code_runs_launchers.py:233` 是改前行号的 `if name not in DISPATCH_LAUNCHER_OPTIONS …` 那一行，不是调用点，意思没引错。
- 「更严，不放宽」的方向：见第 5 问 5.6。
- 推翻条件：找到一个不在三张表里的 `location`，被 `same_content_as_own` 收到，且该处在改前的 `own` 盘上有文件、`path` 与它逐字节相同——静态看只有上面两类；动态要看自证新格与今后的样本。

### 4.4 「换上」那一条

- 「四份与草稿 `new/` 逐字节相同」：成立（4.0，`cmp` 四次无输出，量的）。
- 「`hook-swap.py` 单独交，退 0」：推不出。退出码在任务服务的结局行里；`.git/singlefs-hook-swap-outcome.tsv` 按仓只留最近一趟，现在里面是别的文件的换上，读不到这一趟。我能核的只有末态：四份已是新版、`swap-real/hook-swap/` 下留着这一趟的喂法日志（`feeds-*.log`、`detections-*.jsonl`、`reverse-*.json`）。
- 「共用库自证查了 473 种」「项目代码闸自证查了 1492 种，其中该拒 332 种」「开关复跑 83 条、逐格相同 83 条、拒算 0」：推不出（任务输出，只读够不着）。算术上对得上：`records/2026-11-17-subagent账本清欠调度记录.md:536` 记着上一趟「共用库「查了 467 种」」「项目代码闸「查了 1492 种，其中该拒 332 种」」「共用库 82 条对上」（量的，grep 命中），473 = 467 + 6（六格），83 = 82 + 1（一条新开关），项目代码闸 1492 / 332 不变（这一批没动它的格）。这是对得上，不是复核。
- 「`.claude/gate.d/break-switch-replay-files.tsv` 第 70 行条数改八十三条」：成立（4.2e）。
- 「`GATE_STAGE_FILES` 与门禁目录顶层逐个相同」一格此刻为绿：成立（4.2a，量的）。

### 4.5 「换上之后同一条 closure：三行整目录读 0 行，data-file 48 个，都是三张表与 `PART_PROBE_COPIED_FILES` 点名的单个文件」

**推不出；其中「都是」按我的数对不上。**

- 输出只读够不着（同 4.1d），而且算键库在这之后改过（4.1d）；读数是在算键库哪一版上取的，我确认不了。
- 「都是」：我数的三张表与元组点名的文件是 44 个：`KNOWN_SCRIPT_LOCATIONS` 值 18、`.claude/gate.d` 阶段 17、`scripts/hosts/multi-host-run.sh` 1、`PART_PROBE_COPIED_FILES` 8。去重后仍是 44，且 44 个路径在盘上都存在（逐个 `[ -f ]`，缺 0；清单在草稿目录 `expected-44.txt`，量的）。`result.md:11` 里的「36 份 data-file」也是 18 + 17 + 1。正文说 data-file 48，差 4 个：不管那 7 个换前 data-file 里有没有那三个目录，都至少有 4 个不是这两组点名的文件（它们是改前就读的别的数据文件，比如登记表一类，我猜的，没看到清单）。所以「48 个都是…点名的单个文件」按字面不成立，成立的说法是「新增的 44 个都是…」。
- 出路：「要主 agent 跑的命令」第 1 条，把 `data` 行的第二列取出来与 `expected-44.txt` 比，多出的 4 个与 `via` 三类整目录行、`refused` 行一并看。

## 第 5 问：工具员报告「没做什么」与「推的」逐处现查

引 `result.md:行`。「推的」指报告自己标了推、没跑的几处；「没做什么」是 `result.md:108-116`。

### 5.1 `result.md:108`「没跑门禁与测试…主工作区里已存在的脚本与钩子一个字节没动…只跑了 `make-swap-spec.py`（正反两份）与 `python3 -m py_compile`」

- 「一个字节没动」：交回那一刻的话，我现在读不到那一刻。能核的：`base/` 四份与暂存区同（4.0），所以工具员读到的主工作区确是暂存区那一版；主工作区现在是新版，是主 agent 换上的（4.4）。没有矛盾。
- `make-swap-spec.py` 正反两份：成立。`hook-spec.json` sha256 `2751dd531a243b7b20cd095477cff900c623721f91c64f3c3cea8003de192799`、`hook-spec-revert.json` `953f595fe85812370d01766fcc6280037c785b97fb5ef267aed25aeb8ed71ca7`，与 `result.md:44-45` 贴的相同（`sha256sum`，量的）；`make-swap-spec.txt` 与 `make-swap-spec-revert.txt` 各是一句「✓ 写出 …：4 份文件 22 处，正着依次施加等于新版、倒着依次施加回到基线，每处恰好命中 1 次」；`jq` 数规格 22 条，按 `file` 分是 7（`lib_project_code_runs.py`）、3（`_base`）、3（`_launchers`）、9（`_selftest`）（量的）。
- `py_compile`：成立。我把主工作区这四份拷进草稿目录 `compile-check/` 里各 `python3 -m py_compile` 一遍，四份都过（量的；主工作区与 `new/` 相同，所以等于核了 `new/`）。`compile-check/` 用完整个删了（删前 `du -sh` 是 388K）。
- 「没往统一任务服务交任务」：核不了。这一句是工具员对自己动作的陈述，只读命令看不到它交没交过；`swap-dry-run/hook-swap/`、`swap-real/hook-swap/` 里的日志时间是后来主 agent 的两趟（`result.md:47` 写「已建好、是空的」在当时成立，现在两个目录里都有 `hook-swap/` 子目录和日志，`swap-revert/` 仍是空的，量的）。

### 5.2 `result.md:109`「库那一半没改」

成立。依据 4.2b：`launchers.py` 相对改前只有导入行、文档串、`:1370` 三处不同，`:1372-1376` 逐字相同（`diff -u`，量的）。理由那几句（要给 18 份脚本、17 道阶段与 `multi-host-run.sh` 各写一张字面库表）是推的：我没量每份脚本 source 几份库，「几十项」没核；这是设计取舍，归主 agent 判。

### 5.3 `result.md:15` 库那一半「推不出、不收也不拒算」，报告自己标「推的，没跑过」

按算键库现行代码静态走一遍，结论与报告相同，但中间两处细节报告没写、我补上；跑不过，仍记「推的」。

- 读发生在 `file_sha256` 的 `open(path, …)`；`path` 是参数，实参来自各调用点，库那一半的实参是 `:1374` 的 `os.path.normpath(os.path.join(os.path.dirname(own), relative))`。
- 先问折不折得出：`own` 靠 `location`，三个调用方里 `:1330`、`:485` 的实参折不出（`stage`、`basename(words[0])`），整串折不出。
- 再走 `scripts/gate/lib_function_units_python_data_reads.py:453` `        directory = self.fixed_directory_read(node, unit, depth)`：这个函数 `:512-514` 要求最外层是 `os.path.join`，库那一半最外层是 `os.path.normpath`，交 None（与报告「normpath 包着，不走 fixed_directory_read」一致，量的，读代码）。
- 再走 `:456` `        origins = self.head_origin(node, unit, depth + 1)`：`os.path.normpath`、`os.path.dirname` 在 `scripts/gate/lib_function_units_python_facts.py:16-17` 的 `PATH_TRANSFORMS` 里，`os.path.join` 取头一个实参，一路追到 `own = os.path.join(HOOK_REPOSITORY, location)` 的头 `HOOK_REPOSITORY`。它在 `launchers.py` 里是 import 进来的名字，`head_origin_once` 的名字那一支 `:893-896` `            found = self.single_assignment(node.id, None)` 对 import 的名字取不到赋值，交 None（量的，读代码）。
- 头出处是 None，`:457-458`：`        # 出处推不出、前缀也推不出的：照调用方交进来的输入认（文件头「运行期读的数据文件」二的末一种），不收也不拒算` / `        return frozenset(origins) if origins is not None else frozenset({("skip", "unknown")})`，落到 `("skip", "unknown")`。条文在 `scripts/gate/lib_function_units.py:157`：`    出处推不出、前缀也推不出的（对象属性、读进来的值、循环变量、调用的返回值）：照调用方交进来的认，不收也不拒算（头一类的几种都试过之后才落到这里）；`。
- 推翻条件：第 1 条命令的输出里出现退出码 4（拒算）或 `refused` 行落在 `lib_project_code_runs_launchers.py:1374`，或出现 `via` 起头的 `.claude/gate.d`、`scripts/hosts` 整目录行。

### 5.4 `result.md:13`「派发起法：…改交 False，现有调用方走不到这一支」

成立（静态）。调用点与 `command_under_dispatch_launcher` 的门在 4.3 逐行列了；`result.md:13` 引的两处里 `lib_project_code_runs_launchers.py:233` 是改前行号，今天是 `:234` 的 `if name not in DISPATCH_LAUNCHER_OPTIONS or not located_in(…)`（那是门，不是调用点）。

另补一个报告没写的事实（量的，读代码）：`dispatched_under_memory_cap` 的返回值进的是 `project_code_guard_decisions.py:1051` 的 `inner_capped`，再传给 `:1059` 的 `scan_uses(…, inner_capped, …)` 的 `memory_capped` 参数。`project_code_guard_decisions.py:857` 的文档串写着：`    memory_capped：text 整个跑在 run-with-memory-cap.sh 里（起它所在脚本的那一条经它包着）：只交给切词与复用的键，判不判都不看它（经没经内存包装不再判，文件头「经 task-server 起」）。` `grep -rln memory_capped .claude/hooks task-server scripts`（去 fixtures）只命中 `lib_shell_words.py`、`lib_shell_words_core.py`、`project_code_guard_decisions.py` 三份，没有一处拿它做拒放判断。所以第一处判定变化（改交 False）在今天的钩子里连「更严」都谈不上，对拒放没有影响；它只出现在自证里直接调它的几格。这一句限在 `.claude/hooks`、`task-server`、`scripts` 三个目录的 `memory_capped` 命中里。

### 5.5 `result.md:14` 门禁阶段：「新加阶段而没加进 `GATE_STAGE_FILES` 时，别处逐字节相同的拷贝改成读正文」

成立（静态）。`own_script_digest` 的阶段那一段只挑 `GATE_STAGE_FILES` 里的（`lib_project_code_runs_base.py:513-515`），表外落到 `:519` `    return None`，`same_content_as_own` 在 `:1370` 因 `digest != None` 返回 False，`judged_by_name`（`:1329-1330`）的 `or` 两边都假，返回 False。已有阶段不受影响：17 个全在表里（4.2a）。

### 5.6 `result.md:14`「更严，不放宽」

成立（静态），限在门禁阶段这一处：

- `judged_by_name` 在钩子里只有一个非自证的使用点：`project_code_guard_decisions.py:1083`：`        if project_code_runs.judged_by_name(command.words[0], command.directory):`，后一行 `:1084` 是 `            continue`，上一行 `:1082` 注释写「里面起什么由它们自己的自证与门禁管，正文不读」（`grep -n judged_by_name .claude/hooks/*.py .claude/hooks/*.sh` 去掉自证与导出，量的）。为真时只是不读正文；为假时读正文，读出来只会多记 use、多记拒，不会少。
- 「起这份脚本算跑本仓的代码」不经 `judged_by_name`：`lib_project_code_runs_base.py:969` 的 `project_code_use` 看的是起的文件真实路径落在本仓里或是本仓某份文件的拷贝（它的文档串写着「或是本仓某份文件的拷贝（project_relative_path）」），与 `judged_by_name` 无关。所以改成读正文不会让一次直接起变成放行。
- 派发起法那一处见 5.4（对拒放没影响）。

### 5.7 `result.md:110` 阶段名单的漂移检查漏口

前半成立，后半推不出。

- 成立：新格「阶段名单相同格」的输入是 `lib_project_code_runs_selftest.py:880` `        listed_stages = sorted(name for name in os.listdir(gate_stage_root)`，是读目录。算键库不收读目录：`scripts/gate/lib_function_units.py:174`：`  shell 闭包读的文件（重定向 <、cat 的参数这类）这一份还不收、也不拒算；读目录（os.listdir / os.walk / glob）同样还不收。`（量的）。所以新加的、没进表的阶段文件不在这条判定的键里。
- 推不出：「要等别的改动让键变了才会重跑、才会红」这一层依赖自证复用判定怎么用键，我没读那份代码。补一点：新阶段没进表的后果是它的拷贝读正文（5.5、5.6），不是放宽，这一点与报告相同。

### 5.8 `result.md:111-114`「附带看到、没改的看不见的读」

三处都成立，逐处：

- `lib_project_code_runs_selftest.py:817-818` 按 glob 列再读：改前 `:816-817` 是注释行与 `for detector_source_path in [os.path.join(hook_directory_of_library, "bash-command-detector.sh")] + sorted(glob.glob(os.path.join(hook_directory_of_library, "bash_command_detector_*.py"))):`（改前行号比 `result.md` 写的 817-818 靠前一行，是注释行算进去的差别，内容对）；今天在 `:824-826`。读目录与读列出来的文件，算键库一个也不收（`lib_function_units.py:174`）。
- `lib_project_code_runs_selftest.py:864` `            shutil.copyfile(os.path.join(HOOK_REPOSITORY, KNOWN_SCRIPT_LOCATIONS["mutate.sh"]), os.path.join(identical, "scripts", "mutation", "mutate.sh"))`：`KNOWN_SCRIPT_LOCATIONS` 在这份里是 import 进来的名字。算键库折字典下标要字典在本模块里只赋一次（`scripts/gate/lib_function_units_python_module.py:999-1008` `module_dictionary`：`candidates` 要是本模块的 `assign`），这里不是，折不出，落成「根本身 + 折不出的一截」，拷文件的源不拒算。报告的话成立。
- `filecmp.cmp` 两处：`lib_project_code_runs_base.py:933` `    return real_path == os.path.realpath(own) or (os.path.isfile(own) and filecmp.cmp(real_path, own, shallow=False))`、`lib_project_code_runs_launchers.py:890` `                    return filecmp.cmp(own, absolute, shallow=False)`；算键库认的读法在 `scripts/gate/lib_function_units_python_data_reads.py:351-372` 的 `read_target`（`open`、`read_text`、`read_bytes`、`Path(…).open`、`shutil.copy*`），`grep -n filecmp scripts/gate/lib_function_units*.py` 去自证没有命中（量的）。「永远不进键」比我核的宽：核到的是「今天的算键库不认 `filecmp`」。
- 引的行号：报告写的 `lib_function_units_python_data_reads.py:348-369` 在算键库改过之后是 `:351-372`（`result.md:114`），`lib_project_code_runs_base.py:903-906` 是改前行号，今天 `:930-933`。

### 5.9 `result.md:7-10` 引的算键库与 `base.py` 行号，现查

| 报告引的 | 今天 | 判定 |
|---|---|---|
| `lib_function_units_script_closure.py:582-597` resolve_parameter_reads | `:582` `    def resolve_parameter_reads(self):` | 成立 |
| `lib_function_units_script_closure.py:546-549` 拷文件只免拒算 | `:544-549`，`:548` `        elif kind == "refuse" and not copied:` | 成立 |
| `lib_function_units_python_module.py:1178-1227` fold_elements | `:1178` 起，`:1219-1220` 是不认 BinOp 的那两行 | 成立 |
| `lib_function_units_python_module.py:1317-1333` 取遍全部值 | `:1317` 起，`:1333` `                return values or None` | 成立 |
| `lib_function_units_python_data_reads.py:503-545` fixed_directory_read | `:506-554`，函数体已改 | 行号过期，语义也变了（4.1d） |
| `lib_project_code_runs_base.py:70` `KNOWN_SCRIPT_LOCATIONS` | 改前 `:70` 对；主 `:78`（上面插了 8 行） | 成立（改前行号） |
| 「全仓 grep 没有下标赋值、update 或别的赋值」 | 我的 grep（`.py`、`.sh`，去 fixtures）只命中 `lib_project_code_runs_base.py:78` 一处赋值 | 成立，范围限在 `.py`、`.sh` |

### 5.10 `result.md:44-48` 产物与喂法

- 规格 sha256、22 处：成立（5.1）。
- 「喂法用…`project-code-guard.json`。那几条都是 neutral 判法，这一版不改钩子的拒放」：喂法文件 11 条，`jq '[.[]|.neutral]|group_by(.)'` 得 `[{"v":true,"n":11}]`，11 条全标 neutral（量的）；11 条标签里没有一条碰同族拷贝或派发起法（`jq -r '.[]|.label'`）。「这一版不改钩子的拒放」这一句比我核的宽：我核到的是「喂法覆盖的 11 条不碰这两处」，两处的判定变化见 5.4、5.5、5.6。
- 「`swap-*/` 已建好、是空的」：当时成立；现在 `swap-dry-run/hook-swap/`、`swap-real/hook-swap/` 里有主 agent 那两趟的日志，`swap-revert/` 仍空（5.1）。

### 5.11 `result.md:60-80` 新六格与开关的「期望」

静态走了一遍，期望成立；真红不红要主 agent 跑（第 2、3 条命令）。

- 开关 `family-copy-content-unchecked`：`same_content_as_own` 起手就返回 False（`lib_project_code_runs_launchers.py:1366-1367` 的 `break_is_set`），门禁阶段拷贝格与派发拷贝格在 `identical` 树里不是 `family_own_copy`（`FAMILY_OWN_REPOSITORY` 在那一段是 `None`，`lib_project_code_runs_selftest.py:851`，只认 `HOOK_REPOSITORY`，`identical` 树不是它），期望的 True 变 False，红；加上已有的 `mutate.sh` 格共三格，与文件头 `.claude/hooks/lib_project_code_runs.py:180-182` 点名的三格一致。
- 开关 `own-script-digest-stages-unlisted`：阶段那一段整段跳过（`:512`），门禁阶段拷贝格红；「按位置挑得到格」的实际值是 `.claude/gate.d/` 下 17 个阶段位置（其余 18 + 1 个位置仍挑得中），不是 `[]`，红。别的格不读阶段表，不受影响。
- 一个报告没写的观察（推的，没跑）：门禁阶段拷贝格与派发拷贝格在改前的代码上也是绿的（改前同样 `file_sha256(own)` 与拷贝逐字节相同、比内容通过），所以它们钉的是「开关能打红它们」，不是「改前红、改后绿」。第 3 问（攻方腿）问的「有没有一格在改前也绿」可以往这里看，我不下判。

## 引文核对表（上面各处引的那些行，按 cite-check 认的写法再列一遍，行号是做完之后重取的）

被判文件与改前：

- `.claude/hooks/lib_project_code_runs_launchers.py:1370`：「if digest is None or digest != own_script_digest(location):」
- `.claude/hooks/lib_project_code_runs_launchers.py:1368`：「own = os.path.join(HOOK_REPOSITORY, location)」
- `.claude/hooks/lib_project_code_runs_launchers.py:485`：「location = os.path.join(DISPATCH_LAUNCHER_DIRECTORY, os.path.basename(words[0]))」
- `.claude/hooks/lib_project_code_runs_launchers.py:1354`：「location = KNOWN_SCRIPT_LOCATIONS.get(os.path.basename(word))」
- `.claude/hooks/lib_project_code_runs_launchers.py:1330`：「or same_content_as_own(resolved, os.path.join(GATE_STAGE_DIRECTORY, stage)))」
- `.claude/hooks/lib_project_code_runs_launchers.py:234`：「if name not in DISPATCH_LAUNCHER_OPTIONS or not located_in(words[0], directory, DISPATCH_LAUNCHER_DIRECTORY):」
- `.claude/hooks/lib_project_code_runs_launchers.py:1374`：「file_sha256(os.path.normpath(os.path.join(os.path.dirname(own), relative)))」
- `.claude/hooks/lib_project_code_runs_launchers.py:1366`：「if break_is_set("family-copy-content-unchecked"):」
- `.claude/hooks/lib_project_code_runs_base.py:60`：「GATE_STAGE_FILES = (」
- `.claude/hooks/lib_project_code_runs_base.py:199`：「DISPATCH_LAUNCHER_SCRIPTS = (MULTI_HOST_RUN_SCRIPT,)」
- `.claude/hooks/lib_project_code_runs_base.py:504`：「def own_script_digest(location):」
- `.claude/hooks/lib_project_code_runs_base.py:512`：「if not break_is_set("own-script-digest-stages-unlisted"):」
- `.claude/hooks/lib_project_code_runs_base.py:519`：「return None」
- `.claude/hooks/lib_project_code_runs_base.py:78`：「KNOWN_SCRIPT_LOCATIONS = {」
- `.claude/hooks/lib_project_code_runs_base.py:159`：「HOOK_REPOSITORY = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))」
- `.claude/hooks/lib_project_code_runs_base.py:933`：「filecmp.cmp(real_path, own, shallow=False)」
- `.claude/hooks/lib_project_code_runs_selftest.py:131`：「PART_PROBE_COPIED_FILES = (」
- `.claude/hooks/lib_project_code_runs_selftest.py:1057`：「for copied_name in PART_PROBE_COPIED_FILES:」
- `.claude/hooks/lib_project_code_runs_selftest.py:880`：「listed_stages = sorted(name for name in os.listdir(gate_stage_root)」
- `.claude/hooks/lib_project_code_runs_selftest.py:851`：「lib_project_code_runs_launchers.FAMILY_OWN_REPOSITORY = None」
- `.claude/hooks/lib_project_code_runs_selftest.py:864`：「shutil.copyfile(os.path.join(HOOK_REPOSITORY, KNOWN_SCRIPT_LOCATIONS["mutate.sh"])」
- `.claude/hooks/lib_project_code_runs.py:30`：「own_script_digest(location) -> str | None」
- `.claude/hooks/lib_project_code_runs.py:211`：「LIB_PROJECT_CODE_RUNS_BREAK=own-script-digest-stages-unlisted」
- `.claude/gate.d/break-switch-replay-files.tsv:70`：「八十三条开关」
- `.claude/hooks/project_code_guard_decisions.py:1051`：「inner_capped = inner_capped or project_code_runs.dispatched_under_memory_cap(command.words, command.directory)」
- `.claude/hooks/project_code_guard_decisions.py:1083`：「if project_code_runs.judged_by_name(command.words[0], command.directory):」
- `.claude/hooks/project_code_guard_decisions.py:857`：「只交给切词与复用的键，判不判都不看它」

算键库与记录：

- `scripts/gate/lib_function_units_script_closure.py:548`：「elif kind == "refuse" and not copied:」
- `scripts/gate/lib_function_units_python_module.py:1219`：「if not isinstance(sequence, (ast.Tuple, ast.List)):」
- `scripts/gate/lib_function_units_python_data_reads.py:551`：「out.add(("directory", relative) if whole else ("refuse",))」
- `scripts/gate/lib_function_units_python_data_reads.py:458`：「return frozenset(origins) if origins is not None else frozenset({("skip", "unknown")})」
- `scripts/gate/lib_function_units.py:174`：「读目录（os.listdir / os.walk / glob）同样还不收」
- `scripts/gate/lib_function_units.py:157`：「照调用方交进来的认，不收也不拒算」
- `records/2026-11-17-subagent账本清欠调度记录.md:557`：「换前 via 整目录 `.claude/gate.d`、`.claude/hooks`、`scripts/hosts` 三行、data-file 7 个，换后 0 行、data-file 48 个」

## 要主 agent 跑的命令

只有一条，其余格我能用只读命令核完。

1. 在今天的算键库上重算换上之后的闭包，核第 4 问 4.5 与第 5 问 5.3
   `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r1-sonnet -- python3 /home/fy5090/code/singlefs/scripts/gate/lib_function_units.py closure /home/fy5090/code/singlefs scripts/gate/selftest-process-guard.py::watch_judgement --judging --explain`
   - 在哪个目录起：`--cwd` 是我的草稿目录 `/home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r1-sonnet`（命令里的脚本与根都写了绝对路径，入口按根相对）。
   - 看哪几行：末行 `TASK_SERVER_RESULT` 的 `exit=`（0 判完；4 是闭包里有拒算；3 是算不出）；`task-server/task log <号> --all` 里：
     - `via` 起头、第二列是 `.claude/gate.d:.claude/gate.d`、`.claude/hooks:.claude/hooks`、`scripts/hosts:scripts/hosts` 的行，应为 0 行；
     - 以 `refused` 起头的行，应为 0 行（落在 `lib_project_code_runs_launchers.py:1374` 的任何一行就推翻 5.3）；
     - 以 `data` 起头（不含 `data-directory`）的行数，记录写 48；把这些行的第二列排序后与草稿目录里 `expected-44.txt`（排序后）比，多出来的应正好是 4 个、且是这几份判法改动之前就读的别的数据文件；`data-directory` 起头的行逐条看，只要不是上面三类整目录就不算问题。
   - 读不准的地方：算键库在 `result.md` 之后被改过（4.1d），读数会随它的版本变；这一条给的是今天那一版的数，与记录里的 48 不一致不等于换上有问题，要对着差的那几份文件看。
   - 内存与耗时：我没跑，说不出；闭包只读仓、不编，量级不会大于记录里同一条命令那一趟（推的）。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 4.0 改前即暂存区、主工作区与 `new/` | 成立（开工时）；之后三份被无关改动动过 | `base/` 四份与暂存区 sha256 相同；三份后来只差去掉 `serves` 那一项，这一批的改法没被碰 |
| 4.1a launchers 用运行时拼的 `os.path.join(HOOK_REPOSITORY, location)` 取 sha256 | 成立（改前）；今天已改走 `own_script_digest` | 改前 `:1366`、`:1368`；今天 `:1370` |
| 4.1b `location` 是「折得出的目录 + 折不出的一截」 | 部分成立 | 三个调用方里两处是，`:1354` 是字典 `get` |
| 4.1c 自证拷文件源与折不出的元组相加 | 成立（改前） | 改前 `:1029-1030`；`fold_elements` 不认 BinOp（`python_module.py:1219`） |
| 4.1d 换前三行整目录读、data-file 7 | 推不出 | 输出在任务服务；算键库之后改过，今天的版本上改前代码是拒算，不再是整目录 |
| 4.2a base 三张表与 `own_script_digest` | 成立 | 阶段元组与盘上 17 个 `.sh` 逐个相同；现读、无缓存 |
| 4.2b launchers 改一处、库那一半原样 | 成立 | `diff -u` 只三处不同，`:1372-1376` 逐字相同 |
| 4.2c selftest 元组 8 份、六格 | 成立 | `results.append` 78 变 84；拷的名字与次序同改前 |
| 4.2d 入口文件头与再导出 | 成立 | 函数表 `:30`、开关表 `:211`、导出 `:386/:388/:407/:448` |
| 4.2e tsv 第 70 行八十三条 | 成立 | 暂存区同一行是八十二条 |
| 4.3 判定变化只有两处 | 成立（静态） | 第一处工具员写宽了：`capped.sh`、`run-with-memory-cap.sh` 在表里，不变；第一处对今天的拒放没有影响（5.4） |
| 4.4 换上：四份相同、退 0、473、1492/332、83 | 四份相同成立；其余推不出 | 数字与上一趟的 467、1492/332、82 加这一批的增量对得上，不是复核 |
| 4.5 换上之后 0 行、data-file 48「都是点名的单个文件」 | 推不出；「都是」对不上 | 我数的点名文件 44 个，差 4 个；见命令 1 |
| 5.1 没做什么：没跑项目代码、只跑了 make-swap-spec 与 py_compile | 成立（可核的部分） | 规格 sha256、22 处、py_compile 都对；「没交任务」核不了 |
| 5.2 库那一半没改 | 成立 | 同 4.2b；「几十项字面库表」没量，归主 agent 判 |
| 5.3 库那一半不收也不拒算（推的） | 静态走通，仍是推的 | 最外层是 normpath、头是 import 的名字 → `("skip", "unknown")`；命令 1 能推翻 |
| 5.4 派发起法那一支走不到 | 成立（静态） | 唯一调用方在 `launched is not None` 之内；其返回值不进任何拒放判断 |
| 5.5 门禁阶段新增未入表则读正文 | 成立（静态） | `own_script_digest` 表外交 None，`judged_by_name` 为假 |
| 5.6 「更严，不放宽」 | 成立（静态），限门禁阶段 | 为假只多读正文；「起它算跑本仓代码」不经 `judged_by_name` |
| 5.7 阶段名单漂移漏口 | 前半成立，后半推不出 | 读目录不收（`lib_function_units.py:174`）；复用判定怎么用键我没读 |
| 5.8 附带三处看不见的读 | 成立 | 行号：报告引的算键库行号在算键库改后过期，`base.py` 的是改前行号 |
| 5.9 报告引的算键库行号 | 一半成立、一半过期 | `fixed_directory_read` 已是 `:506-554` 且语义变了 |
| 5.10 产物、喂法与「不改钩子的拒放」 | 成立，最后一句偏宽 | 喂法 11 条全 neutral；覆盖的 11 条不碰这两处 |
| 5.11 六格与开关的期望 | 静态成立，真红不红待跑 | 门禁阶段格与派发格在改前代码上也绿，钉的是「开关能打红」 |

## 没做什么

- 没起样本进程、没跑模型、没往任务服务交任务；判决要引的数（闭包行数、自证格数、开关复跑数）都没复跑，命令在上一节（只一条）。
- 没读自证复用判定怎么用算键库的键（5.7 的后半），没读 `admission.py`；没量每份脚本 source 几份库（5.2）。
- 没核任务输出里的退出码、查了多少种、开关复跑条数（4.4）；只能对 `records/2026-11-17-subagent账本清欠调度记录.md:536` 与 `:557` 做算术对照。
- 没替主 agent 采纳或出判决；没改这一批的任何被判文件、没改 `crates/`，没建仓副本、编译目录；草稿目录只留四个文本（`expected-44.txt`、`tuple-stages.txt`、`disk-stages.txt`、`cite2.txt`）。
- 没读禁读清单里的文件（另一条腿的报告）。
