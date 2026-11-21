# 辩方腿报告（Sonnet）：四处协作工具改动第二轮，第 5、6 问（wrapped-writers-loop-paths-cite-titles-governance-r2，2026-11-21）

被复核的判决：`research/prompts/wrapped-writers-loop-paths-cite-titles-governance-r1-main-verification.md`「二、判决」四条与「三、改法」1–4。这条腿替被判出局的一方辩护：改动 A（检出钩子循环变量）、改动 B（任务服务包着的写工具）、改动 C（退 10 两种成因）、改动 D（引文核对小节标题）各自的作者与它们选的做法。只回第 5、6 问，没碰第 1–4 问。

## 读到的范围与三个状态

- 被判 17 份文件读主工作区此刻那一版：`sha256sum -c research/prompts/wrapped-writers-loop-paths-cite-titles-governance-r2-snapshot/judged-sha256.txt` 我跑了，不是 OK 的行数为 0（量的）。
- 第一轮判决引的行号是第一轮状态的行号，此刻的文件已换上修补、行号全移了。第一轮状态我从修补草稿的 `base/` 取：`~/.local/share/singlefs/work/task-server-writers-exit-ten-r1-fix/base/`（任务服务）、`~/.local/share/singlefs/work/detector-loop-target-paths-r1-fix/base/.claude/hooks/`（检出钩子）、`~/.local/share/singlefs/work/cite-check-title-parens-r1-fix/base/`（引文核对）。核过它们就是第一轮开工快照：`task_server_scheduler_lifecycle.py`、`task_server_lib_constants.py`、`task_server_scheduler_building.py`、`bash_command_detector_literals_and_signals.py`、`bash-command-detector.sh` 五份的 sha256 与第一轮快照 `judged-sha256.txt` 对应行逐份相同（量的，`sha256sum` 并排）；引文核对那份 `cite-check-title-lazy/new/` 与 `cite-check-title-parens-r1-fix/base/` 的 sha256 都是 `5f7bf420…`，相同。
- 第一轮之前（改动 A、B、D 各自的改前）取第一轮四个起始草稿的 `base/`：`detector-loop-target-paths/base/`、`task-server-wrapped-writer-reminders/base/`、`cite-check-title-lazy/base/`。第一轮主 agent 对照探针的两份钩子目录也核过：`~/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r1-main-rerun/probe-base/hooks/` 里 `bash_command_detector_literals_and_signals.py` 与 `detector-loop-target-paths/base/` 里那份 sha256 同为 `5c233be0…`（改动 A 之前），`probe-new/hooks/` 里那份与第一轮状态那份同为 `e297a24c…`。所以对照探针的「改前」「改后」就是它说的两个状态，不是别的版本。
- 引文写成「路径:行号」的，路径在仓里的是此刻的行；指第一轮状态的写成草稿目录的 `base/` 全路径，核对脚本对它们列进没判，不是漏核。
- 读数分两种：「量的」是我直接跑出来的（`python3 -I` 跑我自己只用 `re` 的探针、`python3 scripts/agents/cite-check.py` 对我自己的探针文件，两样子 agent 都许直接跑）；「推的」是读代码推的。凡是要引进判决的数，写进末尾「要主 agent 跑的命令」，由主 agent 复跑，跑出之前都当推的。
- 没读禁读清单里的文件（另一条腿的这一轮报告）。读了第一轮判决、第一轮正文与两条腿的报告、这一轮正文。

## 一览表（第 5 问）

| 第一轮判决条目 | 辩方判定 | 一句话 |
|---|---|---|
| 二.1 改动 A：12 种写法改前放改后拒，⑲、带条件两形、经名字引到循环变量一形没格 | 站得住，三处限定 | 对照探针分辨得出（探针两个状态的钩子目录核对过）；探针只调 ⑦ 与 ⑲，⑫ ⑯ 靠读代码、「没格」靠 grep，探针分辨不出这两样；带条件两形判成误拒对，但同一类在改前单名字一路上早就存在，不是这一改新开的一类 |
| 二.2 改动 B：变量或 `cd` 之后的补丁目录被杀不报，是回退 | 站得住，回退面比判决写的宽 | 回退只出在「算不出」的几形，改前对所有包着的被杀都报（含打全了的，是多报）；两个写工具只取头一个在被杀时也是回退，判决只记成漏报；判决引的「现读几行」只证明那一支存在，「改前照报」要看改前的文件才证得出 |
| 二.3 改动 C：「换一个 --scratch」只在占着那一趟是 --dry-run 时有用 | 站得住，但判决给的条件说窄了 | 真换对真换时它错；这一趟自己是 `--dry-run` 时同样有用，判决与修补后的出路都只写了占着的那一趟 |
| 二.4 改动 D：标题带括号整条认不出，比改前更松 | 站得住，限定在引文在前的写法；示例标题不是真标题 | 引文在前写法改前判红、改后不判不列；位置在前写法改前也是认不出；判决用的示例标题在 `.claude/rules/fs-design.md` 里不存在，那两次调用只证明解析结果，证明不了「合法的括号标题」 |

「站得住」指判决这一条按字面成立；「说窄」「回退面更宽」是我读到的、判决没写到或写偏的地方，不是推翻它。

## 第 5 问逐条

### 二.1 改动 A

| 判决引的读数或判断 | 分辨得出什么 | 分辨不出什么 | 依据 |
|---|---|---|---|
| 对照探针 26 行：12 行改前 0 改后拒、2 行两边都 1、12 行两边都 0 | 分辨得出「改后 ⑦ 与 ⑲ 的判定函数对这 12 种写法多拒」。两份钩子目录的 sha256 已核（见上），改前改后不是别的版本；两份输出我并排数过：不同的行恰好 12 行，全是 0 变 1（量的：`paste probe-base.out probe-new.out` 按列比） | 分辨不出 ⑫ ⑯：探针只调 `script_in_place_refusal`（`wrapped-writers-loop-paths-cite-titles-governance-r1-main-rerun/tools/loop_probe.py:72`）与 `record_append_refusal`（同文件:74），⑫ ⑯ ⑨ ⑭ 不在里面。判决没把 ⑫ ⑯ 写成量的，只写「⑫ ⑯ ⑲ 跟着收严」，措辞没越界 | `wrapped-writers-loop-paths-cite-titles-governance-r1-main-rerun/probe-new.out:12`、`:13`、`:24`（三行改后各拒 1 条）；`wrapped-writers-loop-paths-cite-titles-governance-r1-main-rerun/probe-base.out:12`、`:13`、`:24`（同三行改前 0） |
| 「⑲、带条件两形、经名字引到循环变量一形在自证里没有格钉着」 | 探针分辨不出「自证有没有格」，这一句来自第一轮攻方腿的 grep。我在第一轮状态的自证文件上重做：`bash_command_detector_selftest_entries_and_main.py`（⑯ ⑲ 的实现所在）里没有任何喂给钩子的 python `for` 循环；`bash_command_detector_selftest_waits_and_scope.py` 里 python 循环格只有第 520 行（⑨）、第 594 行（⑭）与 ⑦ 的第 1880–1921 行，⑦ 的那几格目标要么是循环变量本身、要么是 `os.path.join('scripts', name)` 直接套循环变量，没有「循环体里 `target = os.path.join(…, name)` 再 `open(target, 'w')`」那一形（量的，`grep -nE` 在 `~/.local/share/singlefs/work/detector-loop-target-paths-r1-fix/base/.claude/hooks/` 两份上） | 「⑫ 没格」说宽了：⑫ 与 ⑭ 共用 `for write in python_in_place_writes(step.code, every_write=not overwrite_only, service_writes=service_judged):` 这一行（`.claude/hooks/bash_command_detector_literals_and_signals.py:1729`），⑭ 的第 594 行那格走的就是这一行，开关 python-loop-variable-unexpanded 下它也在文件头点名的红格里（`.claude/hooks/bash-command-detector.sh:741` 起那一条，格名「工具员 python 的 for 逐个取元组」在第 744 行）。⑫ 独有的是「每个候选落在写范围外就拒」那一半，那一半没有 python 循环格；⑯ ⑲ 是各自独立的调用点，确实没格 | 草稿 `detector-loop-target-paths-r1-fix/base/.claude/hooks/bash_command_detector_selftest_waits_and_scope.py:520`、`:594`、`:1880`、`:1883` |
| 带条件两形「判它为误拒」 | 分辨得出：两行改后拒的内容都是 `scripts/running.sh`，而探针仓里这个脚本存在、`if os.path.exists(f): continue` 会跳过它，`if f.endswith('.md')` 也会排除它。拒的正是守卫在运行时排掉的那个候选，误拒坐实 | 分辨不出「误拒是不是这一改新开的一类」：探针没放单名字对照 | `wrapped-writers-loop-paths-cite-titles-governance-r1-main-rerun/probe-new.out:12`、`:13` |
| 「要么认条件、要么文件头写明条件不看」 | 不用探针：条件不看是这条钩子一贯的读法。此刻 `bash_command_detector_literals_and_signals.py` 里读 `ast.If` 的地方 0 处（`grep -c 'ast\.If'` 为 0，量的）；改前 `python_in_place_writes` 对整棵语法树里每个调用都判（草稿 `detector-loop-target-paths/base/.claude/hooks/bash_command_detector_literals_and_signals.py:412` 起，`for node in ast.walk(tree)`），单名字 `p = '…/running.sh'` 下面 `if False: open(p, 'w')` 改前就拒（推的，补充探针有这一格）。⑫ 的 shell 循环同样是逐个取值、不读循环体里的条件 | 补充探针没跑，单名字那一格改前读 1 是推的 | 「循环体里的条件（if、continue、break、try）不看，体里每个调用都按每个候选判」（`.claude/hooks/bash_command_detector_literals_and_signals.py:356`）、「`for 名字 in 写死的词` 逐个取值」（`.claude/hooks/bash_command_detector_literals_and_signals.py:1640`） |
| 绑定计数说宽「方向是多不认（偏严）」 | 成立，而且比判决说的更强：改前循环变量算不出、不交写；改后只在认得出时才按候选各交一份写，没有循环变量的路径与改前同一算法。所以任何绑定漏数的写法，改后最多比改前多拒，没有一种会比改前更放 | 没有 | 草稿 `detector-loop-target-paths-r1-fix/base/.claude/hooks/bash_command_detector_literals_and_signals.py` 的 `python_path_choices`（第 403 行文档串与第 407 行 `if not names:` 那一支：没有循环变量的交回只有一组、照 `python_literal_path` 算）；探针里 `fixed-target-inside-big-loops` 与 `records-append-single-name` 两行两边都是 1 |

辩方判定：站得住。要替改动 A 作者说的三句：一，12 行里 10 行是它要的新能力（嵌套、`try`、`with`、集合、函数里、`Path / 名字`、`shutil.copy` 进目录、`python3 -c`、经只赋过一次的名字引到循环变量、⑲ 的循环），不是缺陷；剩下两行是带条件的误拒；二，带条件两形是误拒，但出路在拒绝说明里，且与单名字一路同类；三，它没有把任何一种改前拒的写法放过去。

什么现象会推翻：补充探针里 `single-name-under-false-guard` 在改前钩子目录读 0（那样「同一类先例」不成立，带条件两形就是改动 A 新开的误拒）；或 ⑭ 的第 594 行那格在 `BASH_COMMAND_DETECTOR_BREAK=python-loop-variable-unexpanded` 下不红（那样「⑫ ⑭ 共用点被钉住」不成立）。

### 二.2 改动 B

| 判决引的读数或判断 | 分辨得出什么 | 分辨不出什么 | 依据 |
|---|---|---|---|
| 主 agent 现读第一轮状态 `task_server_scheduler_lifecycle.py` 第 1757–1759 行：结局文件按 `os.path.join(cwd, named[0], …)` 拼，不在就交 None | 分辨得出「这一支存在、`named[0]` 是切出的原词、`cwd` 是任务工作目录」 | 分辨不出「改前照报」：判决的回退结论要两半，这几行只给「改后不报」那一半。改前那一半要看改前文件 | 草稿 `~/.local/share/singlefs/work/task-server-writers-exit-ten-r1-fix/base/task-server/task_server_scheduler_lifecycle.py:1757`、`:1759` |
| 「改前 writer 落成 bash、不进结局文件那一支、照报」 | 我在改前文件上读到：命令那一层的 `-c` 串是一个整词，折成相对仓根的路径不在写工具表里，`positions` 为空，`writer` 取 `command[0]` 即 `bash`，补丁那一支（`== WRITER_PATCH_ENTRY`）进不去，直接出 `workspace-write-interrupted`（推的，读码；没起服务） | 同上：没有一格自证在改前文件上喂过这个形状 | 草稿 `~/.local/share/singlefs/work/task-server-wrapped-writer-reminders/base/task-server/task_server_scheduler_lifecycle.py:1677`、`:1685`、`:1709` |
| 回退的范围 | 改前对所有包着的被杀都报，含补丁已经打全的（多报）；改后读结局文件，打全了的不报。所以回退只出在补丁目录算不出的几形：`$p`、`$1`、`cd` 之后的相对路径，不是所有被杀的包着的写工具 | 没有 | 第一轮攻方腿报告 2b 三形；此刻修补写明「补丁目录 … 从这条命令里算不出」（`task-server/task_server_scheduler_lifecycle.py:1827` 一行的原文，下一格引） |
| 乙「一个串里两个写工具只取头一个」 | 判决记成漏报（换钩子提醒改前同样不出）。被杀这一支另有一半：`patch && swap` 被杀时头一个是补丁、结局文件写着完成，改后交 None；改前是 `bash`、照报。所以这一形被杀也是回退，判决写轻了 | 推的（没有格喂过两个写工具） | 草稿 `task-server-writers-exit-ten-r1-fix/base/task-server/task_server_scheduler_building.py:1291`、`:1299`（认到头一条就交） |
| 包着的 `--dry-run` 不出退 10 提醒，「并进改法」 | 同意并进改法；它不是回退：改前包着的整个不识别，改后识别了五个只读词之外的写法、`--dry-run` 留在外面，是新能力没做全 | 没有 | 第一轮正推腿报告 B-5 |

「算不出照报」此刻的原文：「从这条命令里算不出（变量、位置参数、命令替换或 cd 到算不出的目录），没读结局文件」（`task-server/task_server_scheduler_lifecycle.py:1827`）。

辩方判定：站得住。替改动 B 作者说的：它把「所有被杀的包着的都多报」收成「读结局文件、打全了的不报」，回退只留在算不出的几形，修补又把这几形补回照报。

什么现象会推翻：在改前文件上喂一条 `bash -c 'python3 scripts/gate/apply-writer-patch.py "$p"'` 被杀的任务，`workspace-write-interrupted` 不出（那样「改前照报」不成立，甲就不是回退）。第一轮没有这样的格，要量得新写一格。

### 二.3 改动 C

两趟 `hook-swap.py` 同用一个 `--scratch` 时，后一趟在占草稿目录处被拒、退 10，日志里是「✗ 草稿目录」那一种。把「换一个 `--scratch` 之后还退不退 10」按两趟各是真换还是 `--dry-run` 排开：

| 占着的一趟 | 这一趟 | 换一个 --scratch 之后 | 依据 |
|---|---|---|---|
| 真换 | 真换 | 仍退 10（这个仓的换上被占） | 真换先占草稿目录、再占仓：`held, busy_lines = hold_scratch(scratch, mode)`（`scripts/gate/hook_swap_swap.py:928`）、`held, busy_lines = hold_repository(outcome_file, scratch, mode)`（`scripts/gate/hook_swap_swap.py:943`）；自证里另一个草稿目录的后一趟真换退 10（`scripts/gate/hook_swap_selftest.py:1645`） |
| 真换 | `--dry-run` | 能跑 | `if dry_run:` 在占仓之前就返回（`scripts/gate/hook_swap_swap.py:941`）；自证里另一个草稿目录的 `--dry-run` 要退 `EXIT_SWAPPED`（`scripts/gate/hook_swap_selftest.py:1652`） |
| `--dry-run` | 真换 | 能跑（占着的一趟没占仓） | 同上，`--dry-run` 只占草稿目录 |
| `--dry-run` | `--dry-run` | 能跑 | 「几条 --dry-run 要并行就各给一个 --scratch」（`scripts/gate/hook-swap.py:44`） |

所以「换一个 `--scratch`」有用，当且仅当两趟里至少有一趟是 `--dry-run`。判决写的是「只在占着那一趟是 `--dry-run` 时有用」，漏了「这一趟自己是 `--dry-run`」那一行；修补后的出路同样只写占着的那一趟：「占着的那一趟是 --dry-run 的，可以换一个 --scratch」（`task-server/task_server_scheduler_lifecycle.py:1729`），README 中文同一句也是（`task-server/README.md:174`，「占着的那一趟是 `--dry-run` 的可以换一个 `--scratch`」）。服务看得到这一条自己带不带 `--dry-run`（命令在手），看不到占着的那一趟是不是（拒绝说明里没写模式）。

| 判决引的读数或判断 | 分辨得出什么 | 分辨不出什么 |
|---|---|---|
| 主 agent 现读第一轮状态 `task_server_scheduler_lifecycle.py` 第 1697–1698 行（出路「或换一个 --scratch」） | 分辨得出这句话在服务里是无条件的 | 分辨不出它在哪种组合下错：要对着 hook-swap 的占用顺序读，判决用的是 hook-swap.py 文件头，不是这两行 |
| 异常表那一行出路开头一句退 9、退 10 不分（`task-server/task_server_lib_constants.py:129`） | 分辨得出：那一行同时列退 9 与退 10，开头的句子「等上一趟写完成再把同一条命令原样交一遍」后半写着「读得到它写了完成再交」，占着的是 `--dry-run` 时它不写结局文件，这个信号等不来 | 没有。它与退 10 那一句并排，判决判成坐实、改成分开写，成立 |
| 「447 格」读数写宽（记录第 574 行只写退 0） | 成立，是正文措辞的更正，不是改动 C 的缺陷 | 没有 |

辩方判定：站得住（真换对真换时原句确实错），但判决给的条件说窄了。原句「换一个 `--scratch`」在四种组合里三种对、一种错；修补把它改成只在「占着的是 `--dry-run`」时给，这一趟自己是 `--dry-run` 而占着的是真换时，修补后的提醒只叫「等占着的那一趟结束再交」，比实际能做的保守（多等，不会错报）。

什么现象会推翻：真换占着一个草稿目录时，另一趟 `--dry-run` 给同一个 `--scratch` 之外的草稿目录仍退 10（那样第二行不成立）。`hook_swap_selftest.py:1652` 那一格就是这个观测，它绿即第二行成立。

### 二.4 改动 D

我用三个状态的正则对 7 种写法各取一次标题（`title_states.py`，只用 `re`，正则行是从三个状态的 `cite-check.py` 里整行抄到 `frag-base.py`、`frag-r1.py` 的），再对此刻的整份脚本跑一遍（量的）；下面七行依次是 `probe1.md` 里的 P1 到 P7：

| 写法（探针第几行：`probe1.md`） | 改前取到的标题 | 第一轮状态取到的 | 此刻整份跑 |
|---|---|---|---|
| 引文在前、真括号标题、真原文、带反引号 | `放宽` | 认不出（`[]`） | 对上 |
| 引文在前、真括号标题、伪造原文、带反引号 | `放宽` | `[]` | 判红，点名「不含这句」 |
| 引文在前、真括号标题、伪造原文、不带反引号 | `放宽` | `[]` | 判红 |
| 引文在前、真长标题无括号、真原文 | `不为` | `不为省空间牺牲自包含` | 对上 |
| 引文在前、真长标题无括号、伪造原文 | `不为` | `不为省空间牺牲自包含` | 判红 |
| 引文在前、主 agent 读数里那个标题（后面带「（附）」，文件里没有这个标题） | `不为` | `[]` | 判红，「命中 0 次」 |
| 位置在前、真括号标题、伪造原文 | `[]` | `[]` | 判红 |

读法：

- 判决的方向成立：引文在前的括号标题，第一轮状态整条认不出，伪造的原文照过；改前是取短、判红。
- 改前的「判红」对真原文也红（第一行改前取到 `放宽`，标题命中 0 次，`judge_section` 要求标题逐字命中恰好一行，见「标题逐字命中恰好一行」（`scripts/agents/cite-check.py:246`））：改前从没能验过这一类标题，是真假一起误红。所以「比改前更松」是闸口严格程度上的更松，不是「验证能力的回退」。判成回退要加这一句，否则读者会以为改前能抓住伪造。
- 位置在前的写法改前就是认不出（最后一行改前 `[]`），不是第一轮带进来的。判决没分，写成「标题带括号的小节写法改后整条认不出」，应当限定为引文在前。
- 判决读数里的示例标题在仓里不存在：`.claude/rules/fs-design.md:33` 是「## 不为省空间牺牲自包含」，没有「（附）」；它来自核对脚本自证用的小文件（`scripts/agents/cite-check.py:363` 那行写出的文件）。所以那两次 `citations_in` 调用证明的是解析结果（改前「不为」、改后 `[]`），证明不了「真有这个括号标题的合法引文被丢」；后者要靠攻方腿用真标题 `.claude/rules/machine-first.md:76`（「## 放宽的（人脑带宽的产物）」）整份跑的那一次。判决的结论没因此错，读数的出处该改写成这个。
- 暴露面（粗数，量的）：`.claude/rules/*.md` 与 `.claude/singlefs-ai-sop/rules/*.md` 的 333 个标题里带括号的 8 个；`research/prompts`、`records`、`.claude/kb`、`.claude/rules` 里「引文紧挨 `@…（`」的行粗数 18 行（`grep -rnE` 两步过滤，不分写法，只作量级）。
- 「，」「。」那一形「同一个根子」：根子相同（`TITLE_END` 认不出结尾），后果方向相反。括号是「不配、整条不判」（松），逗号与句号是「配过头、取一整串、判标题不在」（严，误红）。并进改法没问题，但同一个后果的说法不成立。此刻修补把误红改成单列没判（`scripts/agents/cite-check.py:250` 那一支，`judge_section` 里「小节标题认不出结尾」）。

辩方判定：站得住，限定在引文在前写法；读数出处要改写。

什么现象会推翻：改前文件对第一行那种真引文判绿（那样改前不是误红，「更松」就是验证能力的回退）。我是按 `judge_section` 读码推的，没有在改前脚本上跑：改前脚本子 agent 直接起会被拒（本仓的脚本），要主 agent 在改前副本里跑（命令见末节）。

## 第 6 问：改法 1–4 的做法与替代方案各在哪种情形下判错

每个改法写两格：选的做法判错的一种情形、替代方案判错的一种情形，各带文件:行；再写同类先例与推翻它的现象。「推的」指读代码推的、没起程序。

### 改法 1：选「条件不看、文件头写明」，替代「认条件」

- 选的做法判错的情形：一个循环给 `scripts/` 下建几份新脚本、其中一份已存在，用 `os.path.exists` 守卫跳过它。第一轮探针里这一形改后拒的是被守卫跳过的那一份（`wrapped-writers-loop-paths-cite-titles-governance-r1-main-rerun/probe-new.out:12`，拒的内容 `scripts/running.sh`）。方向是多拒，不是漏放：没有任何一种写法改前拒、改后放（见第 5 问二.1 最后一行）。出路在拒绝说明里：「要改已有脚本的用 Edit 工具或写临时文件再 `mv`，只写新文件的把已有脚本从候选里拿掉」（`.claude/hooks/bash_command_detector_selftest_entries_and_main.py:1525`）。代价是每遇一次多一轮改写。
- 替代方案「认条件」判错的情形：守卫判的路径和写的路径不是同一个名字，或条件在运行期才定。例如 `for f in ('notes.md', 'scripts/running.sh'): if os.path.exists('notes.md'): open(f, 'w')`，认「先判 exists」这一形而不绑守卫与被写的是同一个名字，就会把它放过去；`if sys.argv[1] == 'force'`、`if os.path.getsize(f) == 0` 这类静态读不出（推的）。钩子此刻读 `ast.If` 的地方 0 处（`grep -c 'ast\.If' .claude/hooks/bash_command_detector_literals_and_signals.py` 为 0，量的），认条件是新开一整个判断面，每认一种守卫写法就多一种绕过的写法。这一格的方向是漏放，比第一格的过拒重。
- 同类先例：找到。改前单名字一路与 shell 循环都不读循环体里的条件：「`for 名字 in 写死的词` 逐个取值」（`.claude/hooks/bash_command_detector_literals_and_signals.py:1640`）；单名字 `p = '…/running.sh'` 下 `if False:` 的写改前就拒（推的，补充探针有这一格）。所以改法 1 是把 python 循环拉到同一读法，不是新立一种。
- 什么现象会推翻：补充探针里 `single-name-under-false-guard` 在改前钩子目录读 0（先例不成立）；或仓里（`.claude/skills`、`records`）有人经常写「带 exists 守卫的字面循环」往 `scripts/` 下建多份脚本，过拒成了常态代价。我在 `.claude/skills`、`.claude/agents`、`.claude/rules` 下 `grep -rn "os\.path\.exists"` 0 处命中（量的），这三处没有这种写法的示范；子 agent 实际写的 heredoc 不在这些文件里，没有系统数过，所以只能写「没找到示范」，不能写「没有人这样写」。

### 改法 2：选「算不出照报」，替代「拒收这类命令」

- 选的做法判错的情形：补丁目录算不出（`$p`、`$1`、`cd` 之后的相对路径），而那一趟其实已经打全或还没开始，服务照报「可能写了一半」，多报一回。代价小：这一项「随结局推一回；结局没人取照 unclaimed-outcome 再推」（`task-server/task_server_lib_constants.py:123`），依据里点名了要读结局文件的命令、没读结局文件（`task-server/task_server_scheduler_lifecycle.py:1827` 一行写明「从这条命令里算不出（变量、位置参数、命令替换或 cd 到算不出的目录），没读结局文件」）。对认得出写工具的不漏报；漏报只出在服务认不出写工具那一层（下一条）。
- 替代方案「拒收」判错的情形：仓里批量打补丁的脚本本来就是变量驱动的：`python3 "$APPLY" "$dir" --root "$tree"`（`scripts/gate/merge-and-verify-items-lib.sh:482`），工具名也经变量：`APPLY="${MERGE_AND_VERIFY_APPLY:-$here/apply-writer-patch.py}"`（`scripts/gate/merge-and-verify.sh:1151`）。拒收若按「认得出写工具、认不出补丁目录」收窄，窄不住，因为服务连写工具名经变量的都认不出：「路径要字面写着，经变量拼的」认不出（`task-server/task_server_scheduler_building.py:1472`），同一类「算不出」里一半拒、一半放；若按「一切算不出都拒」，会拒掉这类批处理脚本本身。（合入后验证自己不算写主工作区，见 `task-server/task_server_scheduler_building.py:1372` 的注释，所以这里引它只证明仓里批量打补丁的写法是变量驱动的，不证明它会被拒收。）
- 同类先例：提交时拒的先例有：「提交时拒，退 3」（`task-server/README.md:65`，`unnamed-run`），但它判的是这条命令会不会白烧资源（`cargo test` 点没点名），不是「被杀之后认不认得出现场」；按「现场认不出」拒收的先例我没找到。检出钩子「认不出的一律拒，偏严」（`.claude/hooks/project-code-guard.sh:88`）也是拒收取向，但它拦的是还没发生的写，位置在执行之前，与任务服务事后提醒不同。
- 同样打中两个方案的：服务认不出的写工具名（经变量拼的）。照报到不了，拒收也判不到。
- 什么现象会推翻：一条补丁目录经变量给的真任务被杀、留下换到一半的主工作区，而那一回「可能写了一半」被淹没、没人读它的依据。读不出这个观测，只有第一轮留下的 `unclaimed-outcome` 再推这一条兜着。

### 改法 3：选「改提醒文字」，替代「改 hook-swap 的退出码」

- 选的做法判错的情形：日志读不到或认不出，提醒落到「两种都可能」。读法在 `task_server_constants.py`：日志读不了交空串，读得了只取末 65536 字节（`task-server/task_server_constants.py:401` 的 `def log_tail(path):` 与 `LOG_TAIL_BYTES = 65536`，`task-server/task_server_constants.py:157`）；拒绝说明之后同一条命令里还有大量输出（hook-swap 后面接别的步骤）就读不到。另一格：服务认的两句起头与 hook-swap 现行原文（`scripts/gate/hook_swap_swap.py:512`「✗ 草稿目录 {scratch} 正被另一趟 hook-swap.py 占着（{who}）」）之间没有一格自证绑着：自证样本是手抄的（`task-server/task_server_selftest_notifying.py:1307` 的 `def swap_busy_log_lines(world):`，只在文档串里提 `hook_swap_swap.py`，不调它的函数）。hook-swap 改了起头那句，服务静默退化成「两种都可能」，自证照绿。退化的结果是出路写「照日志里的拒绝说明办」，不错报。
- 替代方案「改退出码」判错的情形：退出码是接口，改它要同改的地方多：任务服务里提退 10 的有 10 份文件（`task_server_lib_constants.py`、`task_server_constants.py`、`task_server.py`、`task_server_scheduler_lifecycle.py`、`task_server_lib.py`、两份自证、三份 README，量的：`grep -lE '退 10|exit 10|10 で|SWAP_SCRATCH_BUSY_EXIT_CODE'`），`scripts/gate` 里引 `EXIT_SCRATCH_BUSY` 的 4 份（`hook-swap.py`、`hook_swap_swap.py`、`hook_swap_base.py`、`hook_swap_selftest.py`）。更要紧的是，新码也分不出「占着的那一趟是不是 `--dry-run`」，那是「换 --scratch」有没有用的另一半条件（第 5 问二.3）。
- 同样打中两个方案的：占着的那一趟是不是 `--dry-run`。拒绝说明里没写，退出码也没有，服务只看得到这一趟自己。
- 同类先例：找到，而且支持替代方案：hook-swap 自己已经按成因分码，`EXIT_PREVIOUS_SWAP_RUNNING = 9` 与 `EXIT_SCRATCH_BUSY = 10`（`scripts/gate/hook_swap_base.py:58`、`:59`）。所以「改退出码」不是不自然，选文字的理由只是改动半径小、与 hook-swap 的自证解耦。
- 什么现象会推翻：真任务里退 10 的日志常读不到或被冲掉，`swap_busy_cause` 常交 None（改退出码划算）；或 hook-swap 的拒绝说明起头改过字而服务没跟上。前者要从服务的提醒记录里数，我数不到，待主 agent 量。

### 改法 4：选「认不出结尾列进没判」，替代「判红」

- 选的做法判错的情形：编造的标题带「，」，伪造的原文也列进没判、不红。此刻读数（量的，`probe2.md` 第 3 行）：「没判：probe2.md:3 引 .claude/rules/machine-first.md@根本没有这一节，随便写的：小节标题认不出结尾」。核对脚本只在有对不上时退 1（`scripts/agents/cite-check.py:350` 的 `return 1` 在 `if bad:` 下，没判的走 `:352` 的 `return 0`），交回闸也只拦退 1（`.claude/hooks/handback-guard.sh:682`「退 1（有对不上的引文）交那一趟的结果；退 0 交 None」），所以这一处单独出现时放行。没判行淹在别的没判里：第一轮两份报告加背景材料此刻整份跑，末行「85 处没判（其中裸行号没有引文可比 63 处」（量的，`out-r1reports.txt` 末二行），一条标题认不出结尾的行会是这 85 条里的 1 条（那两份报告里实际一条也没有，`grep -c '认不出结尾'` 为 0，量的）。
- 替代方案「判红」判错的情形：真标题后面把说明写在同一个位置里（`probe2.md` 第 4 行：真标题后接逗号与「补充说明」；那个真标题在 `.claude/rules/machine-first.md` 的第 76 行，存在）。此刻这一行列进没判；判红就是误红，作者只能改写成行号写法，这正是改前的状态。标题超过八十个字的真标题同理。
- 同类先例：找到。这个脚本的原则就是「判不了的不判、单列」（`scripts/agents/cite-check.py:17`）：路径认不出、行号超出文件之外被引文件在开工之后改过、标题认不出结尾都走同一条，改法 4 是把标题认不出结尾也并进来，没有另立原则。
- 旁见（不是缺陷）：带反引号的标题有确定的结尾（反引号），括号没配上时取到反引号，标题命中 0 次，此刻判红而不是没判（`probe2.md` 第 5 行，量的：「标题「放宽的（人脑带宽的产物」在这份文件里命中 0 次」）；没判只给不带反引号、结尾真认不出的。
- 什么现象会推翻：在仓里全部 `*-output.md` 上跑一遍，「小节标题认不出结尾」的没判行里逐条读，有伪造的原文（那样判红更好），或一条都没有真正认不出（那样这一支是死代码）。我试着在子 agent 里整批跑被拦了，命令写在下面，由主 agent 交。

## 旁见（不在第 5、6 问里，交主 agent 判要不要进别的腿的射程）

- `.claude/hooks/bash-command-detector.sh:224` 文件头那一句「同名的这几种在也照按候选判」里有一个多出来的「在」字，意思读得通但像手滑；改动 A 修补里新写的，不是我动的。

## 复跑用的文件与读数性质

没写模型目录与 `rerun.sh`：要跑的都是几个小探针，不是状态模型。文件都在 `/home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r2-sonnet-b`：

| 文件 | sha256 | 是什么 |
|---|---|---|
| `tools/loop_probe_extra.py` | `ef512fa8f141b0a9097b91cd284342f1e4f3cf09b4cd851409433f6a4b5df899` | 补充对照探针，装置抄自第一轮复跑目录的 `tools/loop_probe.py`（前 39 行原样），只换了写法表 |
| `title_states.py` | `65b5d968a0d658bcda3eafe62b8a3caa0db555533879943339c88d65e4aa6652` | 三个状态的小节标题正则对同一份输入各取一次标题，只用 `re` |
| `frag-base.py`、`frag-r1.py` | `7e9506962a81842b028c402974c9d770cf822d68aed4fa682d5c41153cd17b8d`、`425869c50ddee58a293c87928fcba250b8fd06cdd17102b05fe33f6fe2d99e21` | 从改前与第一轮状态的 `cite-check.py` 整行抄出的正则行 |
| `probe1.md`、`probe2.md` | `6381ef95fb5eb309e4520c8fffbfa6efba136e478271f2fb1570a3438572138f`、`49f5cee628da72b19f117b2cfe28ac9695a3c83e9d46d76a5252fac47806876e` | 引文探针输入 |
| `cc-base-run/scripts/agents/cite-check.py`、`cc-r1-run/scripts/agents/cite-check.py` | `effe2352c5bb1dd809b7f1029d4787ee25984ecd0724b580008cb90b2034066b`、`5f7bf420ec1a072b0b2b7abc09d013791e75ebbd51965f260a2ad1fe15db0abd` | 改前与第一轮状态的 `cite-check.py` 原样拷贝（各带一个指向仓里 `.claude` 的符号链接，给它的 `preflight` 导入用） |

探针的内存与时间我没单独量：`title_states.py` 与两趟 `cite-check.py` 我直接跑过，都是秒级、只读几份文件；`loop_probe_extra.py` 我没跑（它导入钩子模块，要经任务服务），装置与第一轮对照探针同一份，那一份第一轮主 agent 跑过、退 0。
读数性质：第 5 问里标「量的」的是我直接跑出来的（`title_states.py` 的表、`cite-check.py` 对 `probe1.md` 与 `probe2.md` 的此刻读数、`paste` 数第一轮两份输出的行差、`grep` 数）；标「推的」的是读代码推的，补充探针与两份改前、第一轮状态的 `cite-check.py` 整份跑都没跑，写在下面「要主 agent 跑的命令」。

## 要主 agent 跑的命令

每条在哪个目录起（即 `--cwd`）、看哪几行写在条后。都不碰判定库，不起服务以外的东西；`--cwd` 与命令里的路径都在工作根下。

1. 补充探针，改前钩子目录（第一轮复跑目录里的 `probe-base/hooks`，改动 A 之前）：
   - `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r2-sonnet-b -- python3 /home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r2-sonnet-b/tools/loop_probe_extra.py /home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r1-main-rerun/probe-base/hooks base /home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r2-sonnet-b/probe-work`
   - 在 `/home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r2-sonnet-b` 起；看 `TASK_SERVER_RESULT` 的 `exit=0`，日志里 6 行 `case` 与末行 `done`、`base`、`6`。推的读数（待跑）：`single-name-under-false-guard` 1、`single-name-exists-guard-existing-file` 1、`loop-exists-guard-existing-and-new` 0、`loop-rebound-by-except-as` 0、`loop-rebound-by-def` 0、`loop-all-new-files-no-guard` 0。头两行是 1，「条件不看」的先例就成立；头两行是 0，先例不成立。
2. 补充探针，第一轮状态钩子目录（`probe-new/hooks`）：
   - `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r2-sonnet-b -- python3 /home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r2-sonnet-b/tools/loop_probe_extra.py /home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r1-main-rerun/probe-new/hooks new /home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r2-sonnet-b/probe-work`
   - 在 `/home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r2-sonnet-b` 起；同上，末行 `done`、`new`、`6`。推的读数（待跑）：前五行都是 1，`loop-all-new-files-no-guard` 是 0。`loop-rebound-by-except-as` 与 `loop-rebound-by-def` 是 1，就是「绑定计数说宽、偏严」的实证；是 0，说宽的方向要重判。
3. 三个状态的标题取法：
   - `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r2-sonnet-b -- python3 -I /home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r2-sonnet-b/title_states.py /home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r2-sonnet-b/probe1.md`
   - 在 `/home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r2-sonnet-b` 起；看 14 行输出，我量到的是：P1、P2、P3 改前 `['放宽']`、第一轮状态 `[]`；P4、P5 改前 `['不为']`、第一轮状态 `['不为省空间牺牲自包含']`；P6 改前 `['不为']`、第一轮状态 `[]`；P7 两边都 `[]`。
4. 改前的整份核对（判「改前对真原文也判红」）：
   - `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r2-sonnet-b -- python3 /home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r2-sonnet-b/cc-base-run/scripts/agents/cite-check.py /home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r2-sonnet-b/probe1.md --root /home/fy5090/code/singlefs`
   - 在 `/home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r2-sonnet-b` 起；看末行「处对不上」。推的读数（待跑）：核了 6 处引文、6 处对不上（P1 真原文、P4 真原文都在里面）；P1 或 P4 不在红的行里，「改前对真原文也红」不成立。
5. 第一轮状态的整份核对（判「引文在前的括号标题伪造原文照过」）：
   - `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r2-sonnet-b -- python3 /home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r2-sonnet-b/cc-r1-run/scripts/agents/cite-check.py /home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r2-sonnet-b/probe1.md --root /home/fy5090/code/singlefs`
   - 在 `/home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r2-sonnet-b` 起；推的读数（待跑）：核了 2 处引文、1 处对不上（只有 P4、P5 被取到，P5 红），P1、P2、P3、P6、P7 都不在红的行里也不在没判里。P2 在红的行里，判决的「整条认不出」不成立。
6. 此刻状态的整份核对（我直接跑过，复跑对数）：
   - `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r2-sonnet-b -- python3 /home/fy5090/code/singlefs/scripts/agents/cite-check.py /home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r2-sonnet-b/probe1.md`，在 `/home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r2-sonnet-b` 起；我量到：核了 7 处引文、0 处裸行号，5 处对不上（P2、P3、P5、P6、P7）、0 处没判。
   - `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r2-sonnet-b -- python3 /home/fy5090/code/singlefs/scripts/agents/cite-check.py /home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r2-sonnet-b/probe2.md`，在 `/home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r2-sonnet-b` 起；我量到：核了 3 处引文，1 处对不上（第 5 行），2 处没判（第 3、4 行，说「小节标题认不出结尾」）。
7. 可选：改法 4「什么现象会推翻」那一项要在整批报告上数「小节标题认不出结尾」的没判行，逐条读有没有伪造的原文：
   - `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r2-sonnet-b -- python3 /home/fy5090/code/singlefs/scripts/agents/cite-check.py /home/fy5090/code/singlefs/research/prompts/*-output.md`，在 `/home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r2-sonnet-b` 起（通配由主 agent 的 shell 展开成每份报告；我在子 agent 里试过仓根相对路径的这一条，被分类器拦了，没有读数）。

跑完判决引完数，`/home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r2-sonnet-b` 整目录（现 `188K`，没有仓副本与编译目录）由主 agent 删；`cc-base-run`、`cc-r1-run` 里的 `.claude` 是符号链接，删之前先 `rm` 掉这两个链接再删目录。

## 没做什么

- 不替主 agent 撤销或维持第一轮判决，只交复核结果；上面「站得住」「说窄」「回退面更宽」都是我读到的，不是对判决的裁决。
- 没往统一任务服务交任务，判决要引的数（补充探针、改前与第一轮状态的整份核对、三个状态的标题取法）都没经服务复跑，命令在上一节。
- 没跑补充探针（它导入钩子模块，子 agent 不起）；第 5 问二.1 里「单名字 `if False` 改前就拒」与二.2 里「改前照报」是读码推的，不当量的。
- 没读禁读清单里的文件；没碰第 1–4 问（改动 E 与第一轮修补的攻击面）。
- 没改任何仓内文件，只写了本报告与草稿目录。

## 交回前的引文核对

`python3 scripts/agents/cite-check.py research/prompts/wrapped-writers-loop-paths-cite-titles-governance-r2-sonnet-output.md --background research/prompts/_wrapped-writers-loop-paths-cite-titles-governance-r2-body.md` 退 0，末行：「核了 15 处引文、35 处裸行号，对上 15 处，没判 35 处」；没判的是指向仓外草稿目录的引用（路径认不出）与行号在范围内、没有引文可比的裸行号，没有对不上的。
