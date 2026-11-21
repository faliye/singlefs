# hook-lib-foldable-reads-governance-r3：判决（2026-11-21）

正文 `research/prompts/_hook-lib-foldable-reads-governance-r3-body.md`，开工快照 `research/prompts/hook-lib-foldable-reads-governance-r3-snapshot/`。这是这一题的第三轮（最后一轮，治理文件轮），攻第二轮判决 `research/prompts/hook-lib-foldable-reads-governance-r2-main-verification.md` 的修补；第一轮判决 `research/prompts/hook-lib-foldable-reads-governance-r1-main-verification.md`。这一轮之后不开第四轮，这一轮的改法都标零轮。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 攻方（Opus） | `research/prompts/hook-lib-foldable-reads-governance-r3-opus-output.md` | 第 1 问打中：识别函数只在命令位置上找 source，case 分支、赋值前缀、`builtin source` / `command .`、`elif`、`eval` 串、`bash -c` 串、`source <(…)` 这几形既不进认不出、也不照不同判，静默跳过（前四形改前的正则认得出，是退回）；仓里现有一处漏比（`.claude/gate.d/lib/doc-text-content-and-history-cells.sh:663` 在子 bash 里 source 规范副本的 `lib.sh`）；`SOURCED_LIBRARY_DEPTH = 4` 静默截断，`crates-mutation-rows.py` 运行期载入的 17 份 `.py` 不比；登记表的键没有行号，同一份文件里再写一行同样参数的 source 引别的库被当成已登记。第 2 问：表外的库让现役命令被拒不成立；「表那一格红」那一半没有样本与开关。第 3 问打中：分辨格躲得开四种（模块顶层 `+=`、带注解的再赋值、函数里 `global` 再赋、旁边另用推导式拷）；改法 4 只读 grep 仍被拒（量过）、先 `open().read()` 存变量再 `exec`、先 `compile` 再 `exec` 放行（量过，收窄带进来的） |
| 正推（Sonnet） | `research/prompts/hook-lib-foldable-reads-governance-r3-sonnet-output.md` | 第 4 问文件现状各条成立，主 agent 的任务读数推不出；`project_code_guard_selftest.py` 依赖 fixtures 目录的格有两处（第 1081 行起与第 1317 行起）；不在行首的命令位置 source 22 处、核法数不到，点名的库都在表里；写报告的命令被项目代码闸整条拒过一次。第 5 问：闭包正向断言落点建议门禁 code-tooling 紧挨 `research-script-selftests` 的一格；守卫自证那两格建议照共用库自证的做法改 |

核查：

- 开工快照：`sha256sum -c research/prompts/hook-lib-foldable-reads-governance-r3-snapshot/judged-sha256.txt` 4 份都 OK，主工作区此刻就是开工那一刻的树，没另起 snapshot-tree。
- 引文：`python3 scripts/agents/cite-check.py research/prompts/hook-lib-foldable-reads-governance-r3-opus-output.md research/prompts/hook-lib-foldable-reads-governance-r3-sonnet-output.md --background research/prompts/_hook-lib-foldable-reads-governance-r3-body.md`，末行「核了 60 处引文、28 处裸行号，对上 60 处，没判 28 处」，没有对不上的。
- 主 agent 复跑（两条腿的探针与脚本拷进 `~/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r3-main-rerun/`，攻方探针 sha256 与腿报告相同）：
  - 攻方探针六段，命令都是 `task-server/task submit --cwd <复跑目录> -- python3 <复跑目录>/probes/probe_r3.py /home/fy5090/code/singlefs <复跑目录> <段>`，六段都 type=succeeded exit=0（预言逐条都中）。读数里实测与契约不同的行：
    - forms：「FORM case-arm spec='lib/a.sh' … got='SILENT'」，同样 SILENT 的还有 environment-prefix、builtin-source、command-dot、elif-condition、process-substitution、process-substitution-after-and、eval-string、child-shell-string-head。
    - family：追加这几形的 source 行之后库过期的拷贝照判同族（appended-environment-prefix、appended-builtin-source、appended-case-arm、appended-elif-condition、appended-process-substitution、appended-child-shell-string-head 都是 spec=False got=True）；「registered-line-absorbs-second-library spec=False got=True」「real-doc-text-sop-lib spec=False got=True」「real-broad-mutation-deep-part spec=False got=True」。
    - depth：「DEPTH broad-mutation.py-libraries-cut-by-depth spec=0 … got=17」。
    - selfsame：38 行里只有「broad-mutation-copy-without-broad_mutation_base.py spec=True got=False」一行不同，其余现役脚本的原样拷贝都判同族。
    - loads：放行了该拒的 open-read-then-exec、compile-then-exec-heredoc、exec-open-split-line、comma-import、run-module、path-joined-spec；拒了该放的 grep-loader-call、grep-fixed-selftest-sample、sed-loader-lines。
    - astcell：module-augmented-assignment、helper-global-rebinding、module-annotated-reassignment、comprehension-extra-copy 四种下分辨格是 green（该 red）。
  - 正推腿的闭包核对（`closure_check.py`）：type=succeeded exit=0，「EXPECTED total=107」，两个入口都是 refused=0、whole-directory-reads=0、missing=0、fingerprint-differs=0。
  - 正推腿的入口探针（`entry_probe.py`，入口 `.claude/hooks/project-code-guard.sh --program-argument --selftest`）：type=succeeded exit=0，DATA 行里没有 `.claude/gate.d/fixtures/crates-mutation-replay.sh/` 那个目录。
  - 换上之后的开关复跑、两份自证与两条闭包的读数在调度记录 `records/2026-11-17-subagent账本清欠调度记录.md` 第 595 行。

## 二、判决

1. 攻方第 1 问：坐实而采。九种 source 写法静默跳过（forms、family 两段量到），其中接脚本自己所在目录的几形是第二轮修补带进的退回；仓里现有的漏比两处坐实（`doc-text` 的子 bash 里那一份 `lib.sh`、`broad-mutation.py` 第四层之后的部分文件）；库深度上限静默截断 17 份坐实；登记行不带行号、同一份文件里第二条同参数的 source 被当成已登记坐实。误认导致误拒不成立（selfsame 段 38 行里 37 行相同，不同的那一行是深度截断的同一个根子）。
2. 攻方第 2 问：表外的库让现役命令被拒，不成立（selfsame 段量到）。「表里点名而盘上没有时表那一格红」没有样本与开关，照腿判、采进改法。fixtures 那一格换法之后仍测得到原来那件事，成立。
3. 攻方第 3 问：坐实而采。分辨格四种躲法量到（astcell 段）；改法 4 两头都有：只读的 grep、sed 带载入调用名与文件名的仍拒，`open().read()` 再 `exec`、`compile` 再 `exec`、拆行的 `exec(open(`、逗号 import、`run_module`、路径拼出来的 spec 放行（loads 段）。这一判是整段命令文本上的字面认法，认不全是这一类认法的性质；收窄之后漏的那几种里前三种是这一轮修补带进的。
4. 正推第 4、5 问：文件现状成立；正文「`crates/` 下一个字节没动」限定为这一题没碰 `crates/`。守卫自证依赖 fixtures 目录的两格坐实（入口探针量到那个目录不在键里），采进改法。闭包正向断言的落点采正推腿的判断：门禁 code-tooling 里一格；那份门禁此刻别的会话在改，这一条不在这一题里做，记进调度记录的未做登记，等那份文件空出来单开一件。

## 三、改法

这一轮的改法都标零轮（第三轮之后不再开轮），改的是治理文件，不进产品代码，不做 A/B。

1. 识别 source 行：case 分支的 `)` 之后、赋值前缀、`builtin` / `command` 前缀、`elif` / `while` / `until` 条件里的 source 照认；`eval` 串、`bash -c` 串、`source <(…)` 与别的认不出的写法进「认不出」（查登记表，没登记的照不同判），不静默跳过。自证每一形一格。零轮。
2. 库深度：到上限仍有没展开的库时那一份照不同判（读正文）并让表那一格红，不静默截断；`broad-mutation.py` 被自己的库再引的那一层不占深度（已见过的不重复展开）；`crates-mutation-rows.py` 运行期载入的那 17 份进库表或进登记表。零轮。
3. 登记表 `OWN_SCRIPT_UNRECOGNIZED_SOURCES` 的键带上这一行在那份文件里是第几条同参数的 source（或那一份文件里这种写法的条数），同一份文件里多出一条同参数的 source 时表那一格红。「表里点名而盘上没有时表那一格红」补样本与开关。零轮。
4. 分辨格：`PART_PROBE_COPIED_FILES` 另核模块里没有对它的增量赋值、带注解的再赋值与 `global` 再赋，拷部分文件的函数里只有那一圈调 `shutil.copy*`（推导式里的也算）。零轮。
5. 载入那一判：只读命令（命令词是 grep、sed -n、wc、cat、head、tail、rg、awk 只打印的，与写进文件的 heredoc 正文）里出现载入调用名与文件名不拒；python 代码里（`python3 -c`、喂给 python 的 heredoc、`.py` 脚本正文）文件名与 `exec`、`compile`、`run_module`、`import` 同现的照拒，拆行、存变量再用的照同一段代码里同现判。文件头写明认法与认不到的。零轮。
6. `.claude/hooks/project_code_guard_selftest.py` 第 1081 行起与第 1317 行起两格改成照共用库自证的做法（把阶段源码拷进自证自己造的门禁目录里判），不依赖仓里 fixtures 目录在不在。零轮。
7. 不在这一题里做、另记去向的：闭包正向断言放门禁 code-tooling 一格（等那份文件空出来）；不在行首的命令位置 source 22 处核法数不到（改法 1 做完后由那一格的另一条核法一并数）。

## 交用户的

无。

## 这一轮被判过的文件

- .claude/hooks/lib_project_code_runs.py：判了文件头开关表 91 条与复跑读数一致，成立；新格与开关随改法 1–5 补
- .claude/hooks/lib_project_code_runs_base.py：判了库表 63 份与登记表 10 条（成立，107 份都在键里）；登记键不带行号、载入那一判两头认错（坐实，改法 3、5）
- .claude/hooks/lib_project_code_runs_launchers.py：判了 `shell_source_lines` 与库深度；九种写法静默跳过、深度静默截断（坐实，改法 1、2）
- .claude/hooks/lib_project_code_runs_selftest.py：判了两步样本、fixtures 那一格的换法（成立）；分辨格四种躲法、表那一格红没有样本（坐实，改法 3、4）

## 回看决策

不涉及决策：这一轮判的是项目代码闸共用库的修补，没有拿依据去推翻或确立 `.claude/kb/decisions.md` 里任何一条。
