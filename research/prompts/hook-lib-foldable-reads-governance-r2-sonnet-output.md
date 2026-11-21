# 辩方腿报告：钩子共用库折得出读法，治理文件轮第二轮（hook-lib-foldable-reads-governance-r2，Sonnet，三方辩方腿）

日期：2026-11-21。作答正文第 4、5 问。读的范围：背景材料 `research/prompts/_hook-lib-foldable-reads-governance-r2-body.md`；被复核的判决 `research/prompts/hook-lib-foldable-reads-governance-r1-main-verification.md`；它依据的两份腿报告与工具员报告；被判四份共用库（主工作区此刻那一版，`sha256sum` 与 `judged-sha256.txt` 四行逐行相同，量的）；算键库 `scripts/gate/lib_function_units*.py` 的文件头；`scripts/gate/admission.py`、`scripts/gate/selftest-process-guard.py`、`.claude/gate.d/lib/code-tooling-selftest-cells.sh`、`.claude/hooks/project_code_guard_decisions.py`、`.claude/hooks/selftest-registry.tsv`；主 agent 第一轮复跑留在草稿 `~/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r1-main-rerun/probes/` 的 `progress.md`、两份 `differential-copy-*.json` 与两条探针（只读）。两条腿都读的范围，我读的就是全部范围；没读禁读清单里的文件（另一条腿的第二轮报告）。

## 本腿的文件（都在草稿 `~/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r2-sonnet/`，没写模型、没有 `rerun.sh`）

| 文件 | sha256 | 做什么 |
|---|---|---|
| `tables_arithmetic.py`（输出 `tables_arithmetic.txt`） | `3c63ffa570911934022be962ddd1de902449f67a9cf86bbc67d34a8607a7fba7` | 只读：用 ast 读三张字面表与库表，数个数、相交、哪几份库落在改前那三个整目录之外；已跑（直接跑的只读脚本，不起子进程） |
| `union_count.py`（输出 `union_count.txt`） | `f137ab44ba22e7190b3fa6e361a2fc023beb1e5c33ee36adedba149ecb9d7f52` | 只读：三张表、`PART_PROBE_COPIED_FILES`、库表、检出钩子那一份的并集大小；已跑 |
| `closure_membership_check.py` | `46eaaac63f72d401e03ef4191c779ab1e90c82130375066dacdac0cac22ab9bd` | 对两个入口各起一次 `closure --judging --explain`，正向核「应有的文件都在 data 行里」；没跑，命令在「要主 agent 跑的命令」 |

`python3 -m py_compile` 三份都过。读数里「量的」是我直接跑只读命令或读文件得到的，「推的」是读码推出来的、没有跑，「待主 agent 跑」是等下面的命令。

## 判定一览

| 问 | 判决条目 | 辩方判定 | 一句话 |
|---|---|---|---|
| 4 | 攻方第 1 问：不成立，判对照探针坐实 | 站得住（对第一轮修补那一版）；够不着（对主工作区此刻的代码） | 探针的 28 处不同里 27 处在派发起法、1 处是探针自己拿掉表项造出来的；探针载的是还没有库表的那一版；两处收严是读码推的 |
| 4 | 攻方第 2 问：坐实而采 | 站得住；其中「改前由整目录数据单元连带罩住」一句够不着 | 翻面行是函数层的量，不分辨键里有没有；键里没有靠主工作区闭包；改前一臂在算键库此刻的写法下是拒算，不是整目录罩住 |
| 4 | 攻方第 3 问：坐实而采 | 站得住（变体下自证全绿）；「改前同样绿」与「键不变」两句核不动 | 变体与新写法对普通字符串输入逐条相同，六格全绿是必然；喂「比较相等而字面指到别处」的位置的格分得开（后来加的，推的）；另两句没有量 |
| 4 | 正推第 4、5 问：改正为 44 个点名的加 4 个换前就在的 | 站得住（数对得上）；「换前就在 7 个里」核不动 | 36 加 8 是 44（量的）；四组并集 65 加 4 等于记录里的 69（算术相符，推的） |
| 5 | 改法 1 的做法 | 不被「让算键库拒算」「收整个目录」打中；与「加一格断言」同样留一条缺口 | 见第 5 问逐个替代方案 |
| 5 | 改法 2 的做法 | 同上 | 见第 5 问逐个替代方案 |

## 第 4 问：判决「二、判决」四条逐条

### 条 1：攻方第 1 问不成立，判对照探针坐实（替被判出局的攻方辩护）

分辨得出吗：对第一轮修补那一版分辨得出，对主工作区此刻的代码够不着。

- 28 处不同拆开（量的：`python3 -I` 直读 `differential-copy-base.json` 与 `differential-copy-new.json`，447 个标签、28 个不同）：27 个是 `dispatched_under_memory_cap scripts/hosts/<文件> identical`，1 个是 `drift`。`drift` 那一行是探针自己造的：`probe_function_differential.py` 文件头写「drift：改后那一臂把 doc-text.sh 从 GATE_STAGE_FILES 里拿掉」，`--patch-drift` 下 `arm` 把它从元组里去掉再比。所以它量的是「表漏一道时会不会翻面」，不是盘上有过翻面。盘上此刻没有漏：`GATE_STAGE_FILES` 17 项与 `.claude/gate.d` 顶层非 `lib-` 起头的 `*.sh` 17 份排序后逐个相同（量的，`python3 -I` 读元组与 `os.listdir`，两边差集都空）。27 个派发那几行的 `reach` 行里只有 `scripts/hosts/multi-host-run.sh` 三棵树为 True（量的，同两份 json）。
- 探针载的是第一轮修补那一版，不是今天的：`grep -c own_library_digest` 在探针用的 `copy-new` 里 `lib_project_code_runs_base.py` 与 `lib_project_code_runs_launchers.py` 都是 0（量的），主工作区此刻有「def own_library_digest(library):」（`.claude/hooks/lib_project_code_runs_base.py:517`），比库那一边改调它：「own_library_digest(os.path.normpath(os.path.join(os.path.dirname(location), relative)))」（`.claude/hooks/lib_project_code_runs_launchers.py:1374`）。今天的代码多一类判定不同的输入：库在两边都在、逐字节相同、而不在 `OWN_SCRIPT_LIBRARY_FILES` 里，从前判同族，现在 `own_library_digest` 交 None（「if listed == library:」（`.claude/hooks/lib_project_code_runs_base.py:524`）挑不中）、`file_sha256(copy_library) != None` 成立、`same_content_as_own` 交 False。这一类在背景材料「实现今天的样子」里自己写了（表外的库照不同认），不在 447 个输入里（探针的库树 `libs-identical`、`libs-stale` 只有 `crash-point-replay.sh` 与它的五份库，都在表里）。盘上此刻这一类是空的靠共用库自证那一格（表与现算的逐个相同）；记录写「共用库自证、项目代码闸自证都退 0」（`records/2026-11-17-subagent账本清欠调度记录.md:577`），这一条是主 agent 的读数，我没复跑。所以「判定变化只在已知两处」对第一轮那一版成立，对今天的代码要写成「已知两处，加表外的库这一类（收严）」。
- 两处只收严是读码推的、没有量：判对照探针是函数层的，没有量到钩子的拒放。读码：「if project_code_runs.judged_by_name(command.words[0], command.directory):」（`.claude/hooks/project_code_guard_decisions.py:1083`）为真时 `continue`、不读正文，为假时往下做认不出写法的判与读正文，只会多记 use；「if project_code_runs.existing_copy_elsewhere(wrapper_word, wrapper_directory):」（`.claude/hooks/project_code_guard_decisions.py:938`）为真时多扫一遍包装脚本正文；「inner_capped = inner_capped or project_code_runs.dispatched_under_memory_cap(command.words, command.directory)」（`.claude/hooks/project_code_guard_decisions.py:1051`）的返回值进 `inner_capped`，正推腿读到文档串写着判不判都不看它（转述，正推腿报告第 5.4 节），这一处改交 False 对拒放没有影响。
- 判得对不对：对（对第一轮修补那一版）。
- 什么现象会推翻它：探针在今天的主工作区上重跑出 `unexpected` 大于 0；或盘上出现一份不在三张表里、又被 `same_content_as_own` 收到的位置；或读正文这一支让一条原来拒的命令放行（只靠读码，没有量过）。

### 条 2：攻方第 2 问坐实而采（被判出局的是修补一方「库文件不在键里，改前改后都一样」）

分辨得出吗：分一半。翻面行分辨不出键，键里有没有靠主工作区闭包；改前一臂的描述够不着。

- 翻面行量的是判定翻面：`progress.md` 的 `FLIP` 三行（`before=True after=False` 之类）是改本仓那一份库之后同一个函数的输出变没变。它证明判定依赖这几份库，不分辨这几份库在不在键里：键在在或不在，翻面都一样。键探针的 KEY 段两臂都算不出键：`KEY-BASELINE` 四行都是 `exit=4 key=None`，`copy-base` 的 `refused=6`、`copy-new` 的 `refused=5`，`copy-new` 打出来的前三条都是 `external-program-unkeyed`（herd7、cargo、qemu-system-aarch64），与这一改无关，是仓副本里算键时的拒算（推的：主工作区同一条闭包 refused 0 行）。所以「在不在键里」只有主工作区那一趟闭包能答，判决引的就是它（`records/2026-11-17-subagent账本清欠调度记录.md:557` 起的那几条）。同一份 `progress.md` 里 `CLOSURE copy-new unmutated` 一行写 `exit=0 … data-files=0`：探针在副本上数出 0 份 data 行，而主工作区读到 48 个，两个读数不是同一种读法（探针只数第一列恰好是 `data` 的行，算键库的单元行第一列可能是 `data-file`，推的，没看输出），所以探针的闭包段不能拿来数「有」，只能数「整目录 via」与「refused」。
- 「改前由整目录数据单元连带罩住」够不着。算键库文件头此刻写：「三、拒算 data-path-unfolded（落在读它的那一行）」（`scripts/gate/lib_function_units.py:170`），其中「不按整目录收（弄坏开关 python-unlisted-directory-reads-whole 下照旧整棵收成 data-directory）」（`scripts/gate/lib_function_units.py:171`）。整目录收成数据单元现在只在弄坏开关下出现，默认是拒算。判决复跑的改前一臂印证：`KEY-BASELINE` 里 `copy-base` 打出一条 `refused	data-path-unfolded	.claude/hooks/lib_project_code_runs_base.py:486	with open(path, "rb") as handle:`（这是 `copy-base` 那份文件里的行号，主工作区此刻 `file_sha256` 的 `open` 在 `lib_project_code_runs_base.py` 第 509 行），`copy-new` 的前三条里没有它，两臂差的一条拒算（6 对 5）多半就是它（推的）。所以改前在算键库此刻的写法下是「没有键、照跑」，不是「整目录罩着」；`records/2026-11-17-subagent账本清欠调度记录.md:557` 的「换前 via 整目录 `.claude/gate.d`、`.claude/hooks`、`scripts/hosts` 三行、data-file 7 个」是算键库改写之前的读数，今天重算改前那一臂量不出来。结论的方向不变而说法要改：这一改把这几样从「拒算（不复用）」换成「有键、键里缺这几份（会复用）」，比判决写的「掉出键」更糟，因为拒算是安全的一边（「单元切不开、调用图收不全而拒算键的照跑，不静默当成没变」（`.claude/rules/fs-design.md:73`））。
- 「依赖这几份库的判定会照旧复用」要限定是哪些判定。`scripts/gate/selftest-process-guard.py` 里没有一处引 `judged_by_name`、`existing_copy_elsewhere`、`same_content_as_own`、`dispatched_under_memory_cap`、`own_script_digest`、`own_library_digest`（量的，`grep -cE` 六个名字的合并式，输出 0）；`watch_judgement`（「def watch_judgement(command, directory, registry_root, environment, broken):」（`scripts/gate/selftest-process-guard.py:746`））登记表里每一条自证的键都带它的闭包，那 233 行的判定结果动态上不会随这几份库变（推的：闭包是静态的，经模块整份载入才收到这几处读）。真依赖的是跑共用库自证与项目代码闸自证的两条：`.claude/hooks/selftest-registry.tsv` 的第 210、211 行登记了「.claude/hooks/project-code-guard.sh」与「.claude/hooks/lib_project_code_runs.py」；`FLIP-SELFTEST` 两行（改检出钩子字面集合、加一道阶段，自证各红一格，`exit=1`）量到的是这两条自证的判定真依赖检出钩子那一份与门禁目录的阶段（自证真红），同样不分辨键里有没有，键里有没有仍靠主工作区闭包。所以条 2 的坑是真的、范围是这两条自证（与它们的闭包相同的别的入口），不是 233 行都会照旧复用。
- 判得对不对：对（修补一方没有站得住的辩护）；判决里「改前罩住」一句与「会照旧复用」的范围要改。
- 什么现象会推翻它：主工作区在第一轮修补之前那四份上（算键库此刻的版本）重算闭包，出现整目录 via 而没有拒算行（改前罩住成立）；或那两条自证的键在改本仓 `stage-cells.sh` 一行之后仍变（另有路进键，坑不成立）。

### 条 3：攻方第 3 问坐实而采（替被判出局的新六格与新开关辩护）

分辨得出吗：「变体下自证全绿」分辨得出；「改前同样绿」「阶段漂移那一刻键不变」两句分辨不出，没有量。

- 量到的：`progress.md` 的 `MUTANT-SELFTEST	exit=0	failures=0 … 查了 473 种`，与 `FLIP-SELFTEST	baseline` 的 473 种相同；变体（先查表、查中照旧拿传进来的位置拼路径读）下六格与新开关一格都不红，这一句成立。但全绿不是「六格设计得不好」的证据：六格喂的位置都是普通字符串，变体与新写法对普通字符串逐条给同一个结果，任何格喂它们都分不开。能分开的是喂一个与表项比较相等而字面指到别处的位置：新写法照表里的字面路径读、读到本仓那一份，变体拿传进来的位置拼路径、读到别处。工具员后来加的就是这一格（背景材料写「同族：本仓那一份照表里的字面路径读，不拿传进来的位置拼」），它红不红是推的，等开关复跑。所以「六格分不出」对第一轮修补那一版成立，不能推成「自证里分不出变体」。
- 「第 1、2 格是绿样本，改前代码上同样绿」没有量：`FLIP-SELFTEST` 与 `MUTANT-SELFTEST` 的行都是在 `copy-new` 上跑的，新六格的名字在改前那一份里不存在，没有在 `copy-base` 上跑过自证；这一句出自读码（攻方腿与正推腿都是读码，正推腿报告第 5.11 节）。
- 「第 3 格读目录列举，阶段漂移那一刻键不变」只量到一半：`FLIP-SELFTEST	new-stage-file	exit=1	failures=2	named-cell-lines=1` 量的是格真红，不是键变没变，每次都是现跑；KEY 段两臂算不出键，「键不变」没有读数。能引的是算键库文件头：「读目录（os.listdir / os.walk / glob）同样还不收」（`scripts/gate/lib_function_units.py:184`）。那一句写在 shell 闭包那一句里、对 python 的列举算不算在内不明确（推的）；修补之后的写法改成 glob 列、逐份读，算键库此刻写：「→ 目录里名字合模式的那几份普通文件逐份收成 data-file，不收整个目录」（`scripts/gate/lib_function_units.py:150`），按它加一道阶段会多一份 data-file、键会变；背景材料写这一点工具员「没量」，我也没量。
- 判得对不对：对（量到的那一句）；另两句不作依据。
- 什么现象会推翻它：开关复跑里新开关点名的格与实际红的格对不上；或变体套上之后「本仓那一份照表里的字面路径读」那一格仍绿。

### 条 4：正推第 4、5 问，正文「48 个都是点名的」改正为 44 个点名的加 4 个换前就在的

分辨得出吗：数分辨得出（44、65、69 三个数对得上）；「4 个换前就在 7 个里」分辨不出，清单没留。

- 44（量的）：`union_count.py` 读主工作区得 `scripts 36 copied 8 libraries 21 detector 1`、`scripts+copied 44`、相交为空、八份拷文件都在盘上；36 是 18 加 17 加 1（`tables_arithmetic.txt`：`KNOWN values 18 stages 17 launcher 1 three-tables-union 36`）。所以「点名的单个文件是 44 个」成立，正文写的「都是」（48 个）按字面不成立。
- 69（算术相符，推的）：四组并集 `all-four 65`（44 加 21 份库，库表与三张表相交一份 `scripts/mutation/broad-mutation.py`，再加检出钩子那一份），加那 4 个等于 69，与主 agent 读到的「refused 0 行、data-file 69 个、via 里没有目录」（`records/2026-11-17-subagent账本清欠调度记录.md:577`）一致。这是个数对上，不是集合比过：69 里有没有恰好这 65 份没有比。`closure_membership_check.py` 就是比这个的（命令在后面）。
- 「4 个（两张自证登记表、`scripts/kb/replace-once.py`、`task-server/task`）换前就在 7 个里」核不动：换前的 7 个没有留清单，记录里只有那个数；算键库在那之后改过（上面条 2 引的两行），今天重算改前量不出同一个 7。判决没把它当核过，写的就是「核不动」。
- 「派发起法那一支走不到，坐实」：`reach` 三行为 True 且都是 `multi-host-run.sh`（量的，同两份 json，90 行 `reach` 里 3 行）；量的是 `command_under_dispatch_launcher(["<树>/<文件>", "--peer", "--", "cargo", "test"], <树>)` 一种命令形状，门槛「if name not in DISPATCH_LAUNCHER_OPTIONS or not located_in(words[0], directory, DISPATCH_LAUNCHER_DIRECTORY):」先看名字（`.claude/hooks/lib_project_code_runs_launchers.py:234`），换一种参数写法门槛不变，所以三行足够。
- 判得对不对：对。
- 什么现象会推翻它：`closure_membership_check.py` 的 `MISSING` 非空（库表的某几份没折出）、`EXTRA` 不是 4 个；或 `command_under_dispatch_launcher` 对 `scripts/hosts` 下别的文件交非 None。

## 第 5 问：改法 1、2 选的做法与三个替代方案，各在哪一种情形下判错

被选的做法（现状）：字面库表「OWN_SCRIPT_LIBRARY_FILES = (」（`.claude/hooks/lib_project_code_runs_base.py:204`）列 21 份、「def own_library_digest(library):」（`.claude/hooks/lib_project_code_runs_base.py:517`）只读表里的字面路径、表外交 None（比库时照不同认，读正文）、表全不全由自证一格核；阶段列举改成 glob 列加逐份读，检出钩子那一处只读字面路径。

### 替代方案：让算键库拒算（读本仓里折不出的路径一律拒算）

- 判错的情形：拒算落在许多判定共用的那个判法函数里。「SELFTEST_JUDGE_ENTRIES = (」（`scripts/gate/selftest-process-guard.py:164`）头一项就是 `watch_judgement`，登记表每一行自证的键都带它的闭包；`.claude/hooks/selftest-registry.tsv` 此刻 233 行（量的，`grep -vcE '^#|^[[:space:]]*$'`）。拒算的那一条没有键：「这一趟照跑、不复用、不入库」（`scripts/gate/admission.py:56`），自证格里也写「拒算的（闭包收不全）照跑、不记」（`.claude/gate.d/lib/code-tooling-selftest-cells.sh:35`）。所以闭包里有一处拒算，233 行每次提交都现判，不管改了什么；里面有一条一条十几 GiB 的大自证（「SELFTEST_RUNNERS_MEMORY_BOUND 里的大自证」（`.claude/gate.d/lib/code-tooling-selftest-cells.sh:22`））。这违反的是 fs-design 同一条的前半句：只重跑键对不上的那几条（「门禁的范围必须可判定」，`.claude/rules/fs-design.md:73`）。这不是推演：门禁自己有一条弄坏开关造的就是它：「=selftest-judge-whole-guard：判法取整份 selftest-process-guard.py，每一条都拒算」（`.claude/gate.d/lib/code-tooling-selftest-cells.sh:342`）；第一轮修补之前那四份在算键库此刻的写法下也是这个状态（`progress.md` 的 `KEY-BASELINE` 对 `copy-base` 是 `exit=4 key=None`、拒算里有 `data-path-unfolded` 落在 `file_sha256` 的 `open`）。
- 它不判错的地方：安全，不会把过期的判定当成没变。fs-design 同一条后半句「单元切不开、调用图收不全而拒算键的照跑，不静默当成没变」（`.claude/rules/fs-design.md:73`）就是认它的。
- 被选的是不是同样中：不中。被选的做法把读折成字面表之后，主 agent 读到「refused 0 行、data-file 69 个、via 里没有目录」（`records/2026-11-17-subagent账本清欠调度记录.md:577`），这条判法的键算得出（我没复跑，命令 1 同时给出两个入口的 `refused` 数）。反过来被选的做法不如它的一处：拒算不用维护表，被选的要靠表与自证格；表漏了，这个失效方向是安全的一边（表外交 None、判不同、读正文，判定与表里的库内容无关，所以键里不缺它）。
- 什么现象会推翻它：命令 1 里任一入口 `refused` 大于 0 且落在 `lib_project_code_runs_*.py`；或表外的库交 None 之后读正文放行了一条原来拒的命令（只靠读码，没量过）。

### 替代方案：收整个目录（`.claude/gate.d`、`scripts/hosts`、`.claude/hooks` 各一个数据单元）

- 判错的情形一，漏：库不在这三个目录里。`tables_arithmetic.txt`：`library-inside-three-whole-dirs 6 outside 15`，落在外面的包括 `.claude/scripts/preflight.sh`（17 道阶段与 `multi-host-run.sh` 都 source 它，攻方腿读码，第 2 问）、`.claude/singlefs-ai-sop/scripts/lib.sh`、`scripts/mutation/` 下 12 份（其中 `broad-mutation.py` 同时在按名字判的表里）。判决条 2 也认：`preflight.sh` 改前就不在键里。整目录收成数据单元挡不住这一类，因为目录不对。
- 判错的情形二，抖：目录太大、什么都进键。`find .claude/gate.d -type f` 此刻 9809 份，其中 `.claude/gate.d/fixtures` 下 9670 份（量的）；整目录的指纹是「数据目录的指纹：目录下（递归，什么目录都不跳过）」（`scripts/gate/lib_function_units_base.py:352`），改任何一份样本档，凡带这个目录单元的闭包的键都变，233 行的判法键跟着变。算键库文件头此刻已不这样收：「不按整目录收（弄坏开关 python-unlisted-directory-reads-whole 下照旧整棵收成 data-directory）」（`scripts/gate/lib_function_units.py:171`），按名字模式列的清单只收合模式的几份：「→ 目录里名字合模式的那几份普通文件逐份收成 data-file，不收整个目录」（`scripts/gate/lib_function_units.py:150`）。
- 被选的是不是同样中：不中。库表里 15 份在三个目录之外，都显式列着（`.claude/hooks/lib_project_code_runs_base.py:204` 起）；阶段列举只收 `.claude/gate.d` 顶层合 `*.sh` 的那 17 份，样本档不进键。被选的不如它的一处：新加阶段或库，整目录什么都不用写，被选的要补表，补漏了自证格红（红在提交那一刻，不是改的当下）。
- 什么现象会推翻它：命令 1 的 `data-files` 里有 `.claude/gate.d/fixtures/` 下的文件；或库表之外有一份被 `script_sourced_libraries` 认出的库而自证那一格没红。

### 替代方案：在门禁加一格，断言闭包里没有整目录读、没有拒算

- 判错的情形一，负向断言放过静默漏：算键库有四种读既不收进键也不拒算：「调用方交进来的输入：不进这一份键、不拒算」（`scripts/gate/lib_function_units.py:162`）、「出处推不出、前缀也推不出的」那一类「不收也不拒算」（`scripts/gate/lib_function_units.py:167`）、拷文件源折不出的一截「一律落在这里」（`scripts/gate/lib_function_units.py:169`）、「读目录（os.listdir / os.walk / glob）同样还不收」（`scripts/gate/lib_function_units.py:184`）。落在这四种里的读，闭包没有整目录 via、没有 `refused` 行，断言为绿，键里却缺那份文件。第一轮修补之前库那一半就是这个形状（正推腿报告第 5.3 节，推的）。判决条 3 里这一格要补的「拷文件改回元组相加」那一半也是这个形状：拷文件的源免于拒算：「elif kind == "refuse" and not copied:」（`scripts/gate/lib_function_units_script_closure.py:548`）；改回去以后拷文件源的那一截折不出、落进「一律落在这里」，没有整目录 via、没有拒算，只是 `PART_PROBE_COPIED_FILES` 那 8 份从 data 行里消失（推的，待主 agent 在副本上改回元组相加后跑命令 1 看 `MISSING` 非空而 `refused` 与 `whole-directory-via` 为 0）。所以断言要有正向的一半：data 行包含应有的集合（`closure_membership_check.py` 就是这一半），只断言没有整目录读与没有拒算，抓得住「先查表、查中拼路径读」那一种变体（`MUTANT-CLOSURE` 量到 `exit=4 refused=1`）、抓不住这一种。
- 判错的情形二，射程：这一格只管列在格里的入口。键里带共用库闭包的入口不止两个：判法那五项（`scripts/gate/selftest-process-guard.py:164`）、登记表第 210、211 行那两条自证；攻方腿自己写「只算了两个入口的键」（攻方腿报告，限度一节）。入口一多，每个入口一趟 `closure` 在副本上量到的算键约 46 至 51 秒（`progress.md` 的 `KEY-BASELINE` 四行，量的，是 `key` 子命令不是 `closure`）。
- 被选的是不是同样中：被选的做法单独用，在情形一的第二半（拷文件改回元组相加、或把库表挪到别的模块使循环折不出）同样中，判决条 3 要加那一格正是为此；所以被选的与「负向断言」不是互相替代，被选的要配「正向断言」才不留这条缺口，配「负向断言」仍留。
- 什么现象会推翻它：命令 1 的 `MISSING` 为空、而改回元组相加的副本上命令 1 照样 `MISSING` 为空（说明那一种读改回去并不丢 data 行，条 3 那一半不用靠正向断言）。

### 被选的做法自己判错的情形（辩方不替它瞒）

- 库表挪出本模块：循环变量绕同模块字面元组才折得出，跨模块的字面元组循环折不出（工具员报告第 121 行自己写了这一条：「跨模块的字面元组循环此刻折不出」，`research` 之外的草稿 `result.md`，没判）；有人把 `OWN_SCRIPT_LIBRARY_FILES` 挪到别的模块，自证与开关复跑都不红，只有闭包看得见。这与上面的负向断言放过的是同一个洞，补法同一个（正向断言）。
- 表里的库与盘上不符：表里多一份盘上没有的，`own_library_digest` 读不到交 None，判不同，安全；表里漏一份盘上有的，同上，安全，自证格红。两个方向都不静默。

## 要主 agent 跑的命令

只有一条。两个入口各起一次 `closure`，只读本仓，不碰判定库，不编译、不起 cargo、不设崩溃域。

1. 正向核闭包成员（条 2 的键里有没有、条 4 的 65 加 4、第 5 问负向断言放过的那一类）
   `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r2-sonnet -- python3 /home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r2-sonnet/closure_membership_check.py /home/fy5090/code/singlefs`
   - 在哪起：`--cwd` 就是上面那个草稿目录（命令里的脚本与仓根都写了绝对路径）。
   - 看：末行 `TASK_SERVER_RESULT` 的 `exit=`（0 是两个入口都没有 `MISSING`，1 是有，2 是 `closure` 超时）；`task-server/task log <号> --all` 里：`EXPECTED total=65`；两行 `ENTRY`，各自 `refused=0`、`whole-directory-via=0`、`missing=0`、`extra=4`，`data-files` 应是 69（主 agent 在判决时读到的）；`row-kinds` 那一段看第一列是 `data` 还是 `data-file`（上面条 2 说探针在副本上数出 0 份，可能是第一列写法没认）；`MISSING` 行逐条看是库表里哪一份没折出，`EXTRA` 行应正好是判决写的那 4 个（两张自证登记表、`scripts/kb/replace-once.py`、`task-server/task`）。
   - 估时与内存：没跑过，说不出；每个入口一趟 `closure` 与第一轮 `KEY-BASELINE` 的算键同量级（约 46 至 51 秒，量的，是 `key` 子命令），两个入口顺序跑，估两分钟以内（推的）。
   - 脚本里 `closure` 子进程用 `start_new_session=True` 起，超时只对它自己的组发 `SIGTERM`；没有写任何文件。

没写的命令：把 `selftest` 里 `for copied_name in PART_PROBE_COPIED_FILES:` 改回元组相加之后再跑上面这条（要在仓副本里改，副本我没建；第一轮探针在副本上 `CLOSURE` 一行数出 `data-files=0`，副本上的读数先要弄清是装置的写法还是副本的毛病）。这一条是第 5 问「负向断言放过」那句的实测，结局没出之前那句写「推的，待主 agent 跑」。

## 什么现象会推翻本报告的结论（汇总）

- 命令 1 出 `MISSING` 非空：第 5 问里「被选的做法不留静默漏」要改，条 4 的 65 加 4 等于 69 只是个数对上。
- 命令 1 的 `refused` 大于 0：第 5 问里「被选的不被拒算那一条打中」不成立。
- 主工作区在第一轮修补之前那四份上重算闭包出现整目录 via 而没有拒算：条 2 里「改前不是罩着、是拒算」不成立。
- 今天的代码上差分探针 `unexpected` 大于 0：条 1「已知两处加表外的库」要改。

## 没做什么

- 没往统一任务服务交任务、没复跑判决引的任何数（命令在上一节）；没读也没写判定库。
- 没写模型、没有 `rerun.sh`、没建仓副本（草稿目录只有三份只读小脚本与两份它们的输出，`du -sh` 约 40K）。
- 没替主 agent 撤销或维持判决；条 2、3 里「判决里哪一句要改」只是辩方复核的结果。
- 没对着今天的主工作区重跑差分探针与键探针（探针与副本是第一轮的，第二轮的副本在第一轮草稿目录里，不归我删）。
- 没核那 4 个「换前就在」的文件各是什么：清单没留。
- 没动被判四份文件、算键库与任何别的仓内文件；除本报告外没写仓内文件。
