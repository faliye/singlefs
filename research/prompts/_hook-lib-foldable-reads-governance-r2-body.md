# 钩子共用库按名单读本仓文件：治理文件轮第二轮，攻第一轮判决的修补（hook-lib-foldable-reads-governance-r2，2026-11-21）

这一轮攻的是第一轮判决（`research/prompts/hook-lib-foldable-reads-governance-r1-main-verification.md`）「三、改法」三条已换上主工作区的修补，并派辩方腿复核这份判决。方案按 `crates/` 今天的实现来谈：这一轮被判的文件里没有 `crates/` 下的代码，改的是项目代码闸的共用库，`crates/` 下一个字节没动。第一轮攻过而没打中的角度不重复：判定变化只在已知两处（判定对照探针 447 个输入、unexpected=0）、派发起法那一支走不到、三张字面表与盘上对得上。

## 问题

1. 每一处修补做的是不是判决那一条说的事，修没修到位，有没有带进新的缺口（攻方腿）。
2. 第一轮判决够不够得着：判成坐实而采的第 2、3 问依据撑不撑得住，判成不成立的第 1 问判得对不对；改法 1 选「字面库表加一格从正文现算」、改法 2 选「glob 列阶段逐份读」，是不是同样打中所有替代方案（让算键库拒算、收整个目录、在门禁加格断言闭包），替被判出局的一方辩护（辩方腿）。

## 实现今天的样子

主 agent 的观测（读的是换上之后主工作区的文件；读数取自主 agent 经任务服务交的那几趟，记录在 `records/2026-11-17-subagent账本清欠调度记录.md` 第 577 行）：

- 改法 1：`.claude/hooks/lib_project_code_runs_base.py` 加字面表 `OWN_SCRIPT_LIBRARY_FILES`（21 份）与 `own_library_digest`：从表里按相等挑一项、拿表里的字面路径读本仓那一份；`same_content_as_own` 比库时本仓那一边改调它，表外的库照不同认（读正文，比此前严）。共用库自证加一格「同族：OWN_SCRIPT_LIBRARY_FILES 与三张表里每份脚本按自己所在目录引的库（盘上有的）逐个相同」，另加 `crash-point-replay.sh` 连同库原样拷 / 改过一份库两样本。
- 改法 2：自证读检出钩子那一处改成只读 `bash_command_detector_scope_and_index.py` 字面路径；门禁目录列举改成 glob 列 `.claude/gate.d/*.sh`、逐道读本仓那一份的 sha256、与从表里挑出的那一份比。工具员推的：新加一道阶段（只建文件）键会变（多一份数据文件），没量。
- 改法 3：自证加一格「同族：本仓那一份照表里的字面路径读，不拿传进来的位置拼（…）」与开关 `own-script-digest-location-joined`（自证里把 `own_script_digest` 换成攻方腿那种「查表命中后拿传进来的位置拼路径读」）；另一条新开关 `own-library-digest-unlisted`；三条已有开关补点名的格。开关文件头现数 85 条。攻方变体里「拷文件改回元组相加」那一半没有自证格分得出（两种写法运行时相同），工具员报告写只能靠闭包看，常设检查要在门禁 code-tooling 加格（那份文件此刻别的会话在改）。
- 换上：干跑、真换各单独交、都退 0，四份与草稿 `~/.local/share/singlefs/work/hook-lib-keys-cover-libraries/new/` 逐字节相同；共用库自证、项目代码闸自证退 0。闭包：`selftest-process-guard.py::watch_judgement` 那一条 refused 0 行、data-file 69 个、via 里没有目录，`stage-cells.sh`、`heavy-threads.sh`、`preflight.sh`、`crash-point-replay-crash-case-cells.sh` 在 data-file 里；`lib_project_code_runs_selftest.py::selftest` 那一条 refused 0 行、data-file 69 个、门禁顶层阶段 17 道都在（盘上 17 道）、检出钩子那一份在。
- 开关复跑（期望 85 条）读写判定库，别的会话此刻在删判定库重建，还没交、没有读数。
- 改前原样：草稿 `~/.local/share/singlefs/work/hook-lib-keys-cover-libraries/base/`（第一轮修补之前）与 `~/.local/share/singlefs/work/hook-lib-foldable-reads/base/`（这一题开题之前）；修补的工具员报告在第一个草稿的 `result.md`。

## 范围

这一轮被判的文件（4 份，逐个列在 `research/prompts/hook-lib-foldable-reads-governance-r2-snapshot/judged-sha256.txt`，都读主工作区此刻那一版）。第一轮判决与它引的主 agent 复跑读数在辩方腿的射程里。

## 要腿判的

攻方腿（每问回「成立」「不成立」「推不出」，推不出的算打中；打中的给能照喂的输入或 文件:行）：

1. 改法 1：字面库表与「从正文现算」那一格，有没有一份被比的库仍不进键（多层 source、`../` 链、变量拼的 source 路径、Python 脚本 import 的同目录模块）；那一格本身读 36 份脚本正文不收也不拒算，脚本改了加一行 source 时，那一格的判定会不会照旧复用；表外的库照不同认，有没有把一处此前该判同族的判成不同、因而拒了该放的命令。
2. 改法 2：glob 列阶段逐份读，新加一道阶段时键是不是真会变（给出核法）；`.claude/gate.d/` 下非顶层、非 `.sh` 的文件有没有影响判定却不进键。
3. 改法 3：新格与两条新开关是不是红在名字说的那件事；「拷文件改回元组相加」那一半只靠闭包看，有没有一种办法在共用库自证里分得出（不碰门禁 code-tooling）。

辩方腿（逐条回「成立」「不成立」「推不出」）：

4. 第一轮判决「二、判决」四条：判成坐实而采的，主 agent 引的复跑读数分不分辨得出（FLIP 段量的是判定翻面、不是键；KEY 段核不动）；判成不成立的第 1 问判得对不对。
5. 第一轮改法 1、2 选的做法与替代方案（让算键库拒算、收整个目录、在门禁加一格断言闭包里没有整目录读与拒算）各在哪一种情形下判错，被选的是不是同样打中所有替代方案。

## 分工

攻方腿做第 1 到 3 问，辩方腿做第 4、5 问。两条腿都读「范围」里的文件；只有一条腿读到的范围在报告里写明。
