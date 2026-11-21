# 钩子共用库按名单读本仓文件、改成算键库折得出的写法：治理文件轮第一轮（hook-lib-foldable-reads-governance-r1，2026-11-21）

方案按 `crates/` 今天的实现来谈：这一轮被判的文件里没有 `crates/` 下的代码，改的是项目代码闸的共用库（`.claude/hooks/lib_project_code_runs*.py`）与开关复跑登记表一行；`crates/` 只作为这几份库认的会编命令的对象出现，`crates/` 下一个字节没动。

## 问题

1. 改动有没有改变项目代码闸与共用库的判定（拒放、按名字判、经内存包装认法），改变的那几处是不是只收严、不放宽（攻方腿）。
2. 改动有没有让算键库给依赖这几份库的判定算出漏了输入的键：该进键而没进的、该拒算而没拒算的（攻方腿）。
3. 「实现今天的样子」每一条今天的文件是不是这样写的（正推腿）。

## 实现今天的样子

主 agent 的观测（读的是换上之后主工作区的文件；读数取自主 agent 经任务服务交的那几趟，记录在 `records/2026-11-17-subagent账本清欠调度记录.md` 第 557 行）：

- 起因：算键库 `scripts/gate/lib_function_units.py closure . scripts/gate/selftest-process-guard.py::watch_judgement --judging --explain` 换上之前打出三行整目录读：`via .claude/gate.d:.claude/gate.d … lib_project_code_runs_base.py:file_sha256 data-read`、`via .claude/hooks:.claude/hooks … lib_project_code_runs_selftest.py:selftest data-read`、`via scripts/hosts:scripts/hosts … lib_project_code_runs_base.py:file_sha256 data-read`；data-file 7 个。来处：`lib_project_code_runs_launchers.py` 取本仓那一份的 sha256 用运行时拼出的 `os.path.join(HOOK_REPOSITORY, location)`，`location` 由调用方给「折得出的目录 + 折不出的一截」；自证拷文件的源 `os.path.join(part_probe_source, copied_name)`，`copied_name` 取遍一个折不出的元组相加。
- 改法（工具员草稿 `~/.local/share/singlefs/work/hook-lib-foldable-reads/`，`base/` 是改前原样、`new/` 是新版、`change.diff` 236 行、`result.md` 是它的报告）：
  - `.claude/hooks/lib_project_code_runs_base.py` 加模块级字面元组 `GATE_STAGE_FILES`（17 道阶段）、`DISPATCH_LAUNCHER_SCRIPTS = (MULTI_HOST_RUN_SCRIPT,)`，加 `own_script_digest(location)`：在 `KNOWN_SCRIPT_LOCATIONS` 的值、`GATE_STAGE_FILES`、`DISPATCH_LAUNCHER_SCRIPTS` 三张字面表里逐个比，挑中就现读那一份的 sha256，表外交 None；开关 `own-script-digest-stages-unlisted` 下不比阶段表。
  - `.claude/hooks/lib_project_code_runs_launchers.py` 比内容那一处改成 `digest != own_script_digest(location)`；库那一半（`same_content_as_own` 逐份比 source 进来的库）原样留着。
  - `.claude/hooks/lib_project_code_runs_selftest.py` 拷文件改成循环模块级字面元组 `PART_PROBE_COPIED_FILES`（8 份），加六格。
  - `.claude/hooks/lib_project_code_runs.py` 文件头函数表、开关表加新项，再导出新名字。
  - 工具员报告写的判定变化只有两处：派发起法收到 `DISPATCH_LAUNCHER_OPTIONS` 之外的 basename 时改交 False（它说现有调用方走不到这一支）；`.claude/gate.d` 新加阶段而没加进 `GATE_STAGE_FILES` 时，别处那一份逐字节相同的拷贝改成读正文（它说更严），自证「GATE_STAGE_FILES 与门禁目录顶层的阶段逐个相同」一格会红。库那一半它写「推不出、不收也不拒算，推的，没跑过」。
- 换上：`scripts/gate/hook-swap.py` 单独交，退 0，四份与草稿 `new/` 逐字节相同。共用库自证「✓ lib_project_code_runs 自检通过（查了 473 种…）」、项目代码闸自证「✓ 自检通过（查了 1492 种，其中该拒 332 种）」退 0；开关复跑「复跑了 83 条开关（1 份），自证红且点名的格与实际红的格逐格相同 83 条…拒算 0」；`.claude/gate.d/break-switch-replay-files.tsv` 第 70 行条数改八十三条。
- 换上之后同一条 closure：三行整目录读 0 行，data-file 48 个，都是三张表与 `PART_PROBE_COPIED_FILES` 点名的单个文件。

## 范围

这一轮被判的文件（5 份，逐个列在 `research/prompts/hook-lib-foldable-reads-governance-r1-snapshot/judged-sha256.txt`，都读主工作区此刻那一版）；改前原样在草稿 `base/` 下同路径。

## 要腿判的

攻方腿（每问回「成立」「不成立」「推不出」，推不出的算打中；打中的给能照喂的输入或 文件:行）：

1. 判定变化：拿改前（草稿 `base/`）与改后（主工作区）逐个调用点比 `own_script_digest` 取代的那一处，除工具员报告写的两处外还有没有判定不同的输入；那两处是不是只收严；「现有调用方走不到」那一支是不是真走不到（`project_code_guard_decisions.py`、`lib_project_code_runs_launchers.py` 的调用处）。
2. 键漏输入：三张字面表与盘上实际要比的那几份对不对得上（`KNOWN_SCRIPT_LOCATIONS` 的值、`.claude/gate.d` 顶层阶段、`scripts/hosts` 下的启动器）；库那一半「不收也不拒算」照 `.claude/rules/fs-design.md`「分支的三条约束」第 3 条（「单元切不开、调用图收不全而拒算键的照跑，不静默当成没变」）该不该拒算，给出现在会漏进键的那一份文件。
3. 新六格与新开关是不是红在名字说的那件事；有没有一格在改前的代码上也绿（分不出有无这一改）。

正推腿（逐条回「成立」「不成立」「推不出」）：

4. 「实现今天的样子」每一条：今天的文件是不是这样写的（写出 文件:行 与那一句原文）。
5. 工具员报告「没做什么」与「推的」几处，在只读能查到的范围内逐条现查。

## 分工

攻方腿做第 1 到 3 问，正推腿做第 4、5 问。两条腿都读「范围」里的文件；只有一条腿读到的范围在报告里写明。
