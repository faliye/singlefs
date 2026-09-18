# bg-notify-r1 核查员报告

你交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

写于 2026-09-18 15:27–15:50 UTC 左右（东京约 00:27–00:50 JST）。草稿目录：`/tmp/claude-1000/bg-notify-r1-verifier/`。开工快照 `research/prompts/bg-notify-r1-start-snapshot.sha256` 现核：8 个文件 `sha256sum -c` 全部 `OK`，因此下面对这 8 个文件的行号核对直接对仓里工作树进行（内容与快照一致）；`.claude/rules/three-way-inference.md` 不在快照清单里，是设计/规则类文件，同样对主树核（本轮不改这类文件的历史记录）。

## 一、判别力自证

选定引用：Opus 报告引 `.claude/agent-common.md:40`（含四分句的完整引文，报告第 58 行原样抄出）。

做法：在草稿目录复制 `.claude/agent-common.md`，在复制件里于原第 40 行之前插入一个空行，使那句话整体挪到第 41 行；然后按下面「二」的同一套核对方法（`sed -n '40p'` 取副本第 40 行、比对是否等于 Opus 报告抄的原文）去核这条被人为改过行号的引用。

结果：副本第 40 行变成空行，与 Opus 报告抄的原文（「长活可以等，不给它设超时……」整句）不相等；被挪到第 41 行才对得上。按本报告下面统一的判据，这一格必须判 **✗（行号对不上原文实际所在行）**。

核查方法能产出 ✗，判别力自证通过，往下的 ✓ 判定不是摆设。

## 二、Opus 攻方腿（`research/prompts/bg-notify-r1-opus-output.md`，sha256 现核 `13f8b376136d4859d97890815ab7147a911425873d503c3d521643f2c96a1e8a`，与主 agent 给的一致）

### 2.1 文件:行号 + 抄的原文（逐条 `grep -n`/`sed -n` 现查，对工作树，因 8 文件快照吻合）

| 引用 | 核的结果 | 命令 |
|---|---|---|
| `.claude/agent-common.md:40`（长活可以等……整句） | ✓ 逐字相同 | `sed -n '40p' .claude/agent-common.md` |
| `.claude/agent-common.md:41`（缓存计时器那句） | ✓ 逐字相同 | `sed -n '41p' .claude/agent-common.md` |
| `.claude/main-agent.md:21`（子 agent 提示缓存 5 分钟档） | ✓ 逐字相同，行号精确（`grep -n ""` 现数） | `grep -n "" .claude/main-agent.md \| sed -n '21p'` |
| `.claude/main-agent.md:23`（叫醒之后主 agent 判断……同样放后台） | ✓ 逐字相同，行号精确 | 同上 `sed -n '23p'` |
| `.claude/main-agent.md:29`（交回按……SubagentHandback 消息判） | ✓ 逐字相同，行号精确 | 同上 `sed -n '29p'` |
| `.claude/hooks/bash-command-detector.sh:29,31,36`（三条正则/表达式） | ✓ 三行逐字相同 | `sed -n '29p;31p;36p' .claude/hooks/bash-command-detector.sh` |
| `.claude/agents/three-way-local-attack.md:27`（不许 setsid、&、disown） | ✓ 逐字相同 | `sed -n '25,29p'` |
| `.claude/rules/three-way-inference.md:302-304`（本地腿要前台跑……） | ✓ 逐字相同（文件不在快照内，但内容与本轮开头首次读取时一致） | `sed -n '300,305p'` |
| `records/2026-09-16-subagent拆分提案.md:540,576,577,579` | ✓ 四行全部逐字相同 | `sed -n '540p;576p;577p;579p'` |
| `.claude/agents/gate-triage.md:24-25` | ✓ 逐字相同 | `grep -n "" .claude/agents/gate-triage.md \| sed -n '24,25p'` |
| `.claude/agents/crash-verifier.md:23-24` | ✓ 逐字相同 | `grep -n "" .claude/agents/crash-verifier.md \| sed -n '23,24p'` |
| `.claude/singlefs-ai-sop/rules/command-safety.md:47-48` | ✓ 逐字相同 | `grep -n "" .../command-safety.md \| sed -n '47,48p'` |
| `research/scripts/cache-keepalive.sh:16` | ✓ 逐字相同 | `grep -n "" research/scripts/cache-keepalive.sh \| sed -n '16p'` |
| `research/scripts/agent-watch.py:26,404` | ✓ 两行逐字相同 | `grep -n "" research/scripts/agent-watch.py \| sed -n '26p;404p'` |

### 2.2「16 份定义 + 7 份规则」grep 计数表（报告第 156–179 行）

现跑同一条 `grep -cE 'run_in_background\|后台\|结束本轮\|nohup\|setsid\|disown\|前台\|单次上限\|keepalive'` 对 22 个文件逐一计数：**22 个文件的数字与报告表格逐一相同**（gate-triage.md=1、crash-verifier.md=1、three-way-local-attack.md=1、fs-design.md=1、three-way-inference.md=3，其余全部为 0）。✓

### 2.3 产物：模型文件 sha256（报告第 16–21 行）

现跑 `sha256sum research/prompts/bg-notify-r1-opus-model/{hook-variant.sh,l-regress.py,probe.py,run-output.txt,run.sh,shapes.py}`：**6 个哈希与报告逐字相同**。✓

### 2.4 复跑：`bash research/prompts/bg-notify-r1-opus-model/run.sh <草稿目录>`

**✗（不能稳定复现；但拆开两段单独跑，数据本身对得上）**。

- 开跑前 `ps -o pid,args -u "$(id -u)"` 看到的只是别的会话的 `cargo test`（4 个 PID），没有 `qemu-system`/`vm-bench.sh`/`e152-file-system-benchmark`/`fio`，按共用约束应当加 `nice -n 19` 照常跑，已照办；`uptime` 显示 load average 37–38（32 核）。
- 用报告给的原样命令、在草稿目录里连跑 **3 次**（`opus-rerun`、`opus-rerun2`、`opus-rerun3`），**3 次全部退出码 1**，且都在 `probe.py` 第 75 行 `done[name] = round(float(open(path).read()) - started, 1) ...` 处抛出 `ValueError: could not convert string to float: ''`（完整 traceback 见草稿目录 `opus-rerun{,2,3}-output.txt`）。这是一处竞态（TOCTOU）：`work.sh` 用 `date +%s.%N > done.$1` 先截断文件再写入，`exec_verdict` 检查 `os.path.exists` 之后立即读取，负载重时能读到文件已建、内容未写完的中间态。第一次运行到该崩溃点之前，前 36 行（「仓里的 hook」整张表）已经打印完，恰好在进入第二段 `probe.py`（变体表）时才崩；第二、三次运行更早崩（仅打印表头一行）。这是本机当前真实负载下的现象，不是我构造出来的，但也说明 Opus 给的这条「复跑」命令在有并发负载时不可靠。
- **数据本身核实**：把 `run.sh` 内部两段 `probe.py` 调用拆开单跑（分别对真实仓里 hook、和按 run.sh 同样手法拷贝改名后的 `hook-variant.sh`）：
  - 第一段（仓里 hook 的 33 行判定表）：`diff` 与 `run-output.txt` 第 1–36 行**逐字节相同**（退出码 0）。
  - 第二段（变体判定表）：`diff` 与 `run-output.txt` 第 39–73 行相比，**判定列（记/不记、脱钩/不脱钩、结论）全部相同**，仅 6 个格子的具体秒数有 0.1 秒抖动（6.0→6.1、4.0→4.1 这类，均为当前系统负载下的正常计时抖动，不改变任何判定）。
  - 变体自检（把 `hook-variant.sh` 按 run.sh 的手法拷到 `.claude/hooks/bash-command-detector.sh` 再跑 `--selftest`）：原样输出 `✓ 自检通过：……（查了 17 种）`，与报告逐字相同。`BASH_COMMAND_DETECTOR_DISABLE_SELF_BACKGROUND=1` 再跑一次，我这边打出 3 行 `✗`，报告只贴了「末两行」（`✗ 自检：后台起的 & 在 wait 之后……`、`→ 看 findings_for()……`）——这两行与我复跑输出的对应两行逐字相同，报告没有说只有两行 ✗，所以不算摘句。
  - `l-regress.py`（变体与原 hook 在 L1–L14 上的判定差）：原样输出 `L1–L14 共 14 条，变体与原 hook 第三种判定不同的 1 条：L13（原 记 → 变体 不记）`，与报告逐字相同。
- 推翻条件（写进报告的）「harness 的后台任务改成等整个进程组……」与本条无关，这里另给一条：**这条复跑命令在空闲机器（load average < 5）上连续跑 3 次全部不崩，才算这条命令本身稳定**；当前 3/3 崩溃，暂只能记「数据经拆分复核为真，但给出的复跑命令本身不是可无条件重复执行的」。

### 2.5 真实 harness 行为的独立复核（报告 A1① 一节，X07/X08 与 PPID=1 的claim）

在草稿目录另行构造，不依赖任何历史 PID：`pid=$(nice -n 19 bash work.sh x07 8 > x07.log 2>&1 & echo $!); wait "$pid"`。原样输出 `/bin/bash: line 1: wait: pid 712560 is not a child of this shell`，退出码 127；事后 `ps -o pid,ppid,args -p 712560` 显示 `PPID=1`。**✓ 与报告「wait: pid … is not a child of this shell」「PPID 变成 1」的描述一致**（数字不同是因为 PID 是这次现跑分配的，报告原文也说这是「本腿在真 harness 里三次看到」的活 PID，不是写死的断言值）。

### 2.6 关于 nohup / setsid 的说法（主 agent 指定要核的一项）

报告第 41、98 行明写：「字面禁掉而其实不脱钩的只有 `nohup` 不带 `&`（F01）与 `setsid -w`（F07），两种都没有非用不可的场合」——这句把「不脱钩」的 setsid 变体限定为只有带 `-w` 那一种，隐含着「不带 `-w` 的裸 `setsid CMD`（不带 `&`）会脱钩」。

**核查（用主 agent给的三条实测事实之一 + 本报告独立复核）**：主 agent 给的事实「`bash -c 'setsid sleep 2 > /dev/null 2>&1 < /dev/null'` 外层等了 2.0 秒」。我在草稿目录独立重跑同一条命令三次同类命令（`setsid sleep 2 ...`、`nohup sleep 2 ...`、嵌套 `bash -c "bash -c 'sleep 2 &'"`），原样 `elapsed=2.0026` / `elapsed=2.0019` / `elapsed=0.0017`，与主 agent 给的 2.0 / 2.0 / 0.0016 三个数一致（现跑吻合）。

`research/prompts/bg-notify-r1-opus-model/shapes.py` 里通篇现查（`grep -n setsid`），**只有 F07 一条用了 `setsid -w`，没有任何一条形态测试过裸 `setsid CMD`（不带 `-w`、不带 `&`）**。而按实测，裸 `setsid sleep 2`（无 `&`）外层等了整整 2.0 秒——与 `nohup CMD`（无 `&`）的行为（同样 2.0 秒、同样不脱钩）是同一类，不是报告暗示的「脱钩」那一类。

**判定：✗（报告对裸 `setsid` 的行为描述与实测不符；该情形本轮 33 条形态里未被测试过）**。这处✗只针对「裸 setsid 会脱钩」这一句事实性描述，不涉及报告后面给的改写建议是否该采纳——那是主 agent 的判决职责。

## 三、Sonnet 正推腿（`research/prompts/bg-notify-r1-sonnet-output.md`，sha256 现核 `b8ccd51ed12745a68b1ec5f1cb69c4ddadbe226d226b20a4831a06a3f75737a3`，与主 agent 给的一致）

### 3.1 文件:行号 + 抄的原文

| 引用 | 核的结果 |
|---|---|
| `.claude/main-agent.md:29` 两句英文（`This agent has not reported yet…`、`the result below may be interim`） | ✓ `grep -nF` 两句都在 `.claude/main-agent.md`、`research/prompts/_bg-notify-r1-body.md` 命中，行号与报告一致 |
| `research/scripts/agent-watch.py:214`（`self.state = "本轮结束、没交回"`） | ✓ 逐字相同，且上下文（212–216 行 `elif` 链）与报告描述的分支条件一致 |
| `agent-watch.py:229-230`（`is_done()` 定义） | ✓ 逐字相同 |
| `agent-watch.py:312`（`agent_alerts` 定义行） | ✓ 逐字相同 |
| `agent-watch.py:329`（「无动静」告警行） | ✓ 逐字相同 |
| `agent-watch.py:192`（`is_handed_back = True`） | ✓ 逐字相同 |
| `agent-watch.py:208`（`self.state = "已交回"`） | ✓ 逐字相同 |
| `agent-watch.py:45,52,57,59-60,70,74,120-123`（TaskStop 判法整段） | ✓ 全部逐字相同 |
| `research/scripts/check-staged.sh:75,77,78,80,82,85,86,88,89` | **部分✗，见 3.2** |

### 3.2 check-staged.sh「判据代码」引用：加了源码里没有的注释（✗，整行抄纪律）

报告把这段标成「判据代码（行号现查）」，即声称是逐行抄的源码。现查 `research/scripts/check-staged.sh` 第 75、82、86、89 行原文分别是：

```
75:    return 1
82:    return 1
86:    return 77
89:  return 0
```

**都没有行尾注释**。而报告写成：

```
75:    return 1        # 一个阶段都没跑到
82:    return 1         # 红（不论混不混 77）
86:    return 77        # 全 77
89:  return 0            # 绿（不论混不混 77）
```

四行都多出了源码里没有的 `#` 注释。**判定：✗（引产物没有整行抄，混进了转述性注释，且没有注明这是自己加的旁注）**。行号本身、`return` 语句本体都对；只是这四处的「整行」被悄悄接上了不存在的尾巴。不影响 3.1 里其余引用与 3.3 里复跑结果的正确性判断。

### 3.3 复跑（对四个自写脚本 + 仓里两个脚本的 `--selftest`/判红开关 + 现补的 fixture）

把 `research/prompts/bg-notify-r1-sonnet-model/` 整份拷进草稿目录 `sonnet-rerun/`，四个自写 Python 脚本（内部用绝对路径 `/home/fy5090/code/singlefs` 只读 import 仓里的 `agent-watch.py`，未做任何写改）原样跑：

| 脚本 | 复跑原样输出 | 与报告对比 |
|---|---|---|
| `mid-tool-taskstop-test.py` | `state = 被停, is_done() = True, pending_tool = (…, 'Bash', 'cargo test --release')` | ✓ 一致（时间戳因现跑而不同，属预期） |
| `missing-parent-session-test.py` | `没崩：state = 本轮结束、没交回，is_done() = False` | ✓ 逐字相同 |
| `other-id-stopped-test.py` | `task_stop_time(target) = None` / `state = 本轮结束、没交回, is_done() = False` / `PASS` | ✓ 逐字相同 |
| `resumed-then-stopped-again-test.py` | `529.9 秒前` / `state = 被停` / `is_done() = True` / `PASS` | ✓ 逐字相同，含报告强调的「529.9 秒」这个精确数字 |

仓里脚本 `--selftest` 系列（拷到草稿目录跑）：

| 命令 | 复跑原样输出 | 与报告对比 |
|---|---|---|
| `python3 agent-watch.py --selftest` | `✓ agent-watch 自检通过：……（查了 15 个子 agent、1 个进程、8 次看门狗）` | ✓ 逐字相同 |
| `AGENT_WATCH_BREAK=taskstop python3 agent-watch.py --selftest` | 3 行 `✗` + 1 行 `→`，`EXIT=1` | ✓ 逐字相同 |
| `bash check-staged.sh --selftest` | `selftest: 通过（……）` `EXIT=0` | ✓ 逐字相同 |
| `CHECK_STAGED_77_IS_RED=1 bash check-staged.sh --selftest` | `selftest: 无对象可判（退出码 77）的阶段被当成红、或没在成功行里列名` `EXIT=1` | ✓ 逐字相同 |
| `CHECK_STAGED_USE_WORKTREE=1 bash check-staged.sh --selftest` | `selftest: 拿工作区原样去跑确认判红（……）` `EXIT=0` | ✓ 逐字相同 |
| `CHECK_STAGED_NO_TRAP=1 bash check-staged.sh --selftest` | `selftest: 关掉 trap 确认判红（……）` `EXIT=0` | ✓ 逐字相同 |

「红 + 77 混合」fixture（报告称是「另起一个临时仓」现补的，产物只给了 `.claude/gate.d/{20-red.sh,40-none.sh}` 与 `kb/a.md` 内容，没留 `.git`）：在草稿目录按报告描述的操作顺序重建（`git init` → 提交「干净」版 `kb/a.md` → 改成「BAD（这次要判红）」→ `git add`），跑 `bash check-staged.sh 20 40`：

```
  RED 20-red.sh
  ✗ kb 里有 BAD
    → 删掉
  77  40-none.sh（无对象可判，没跑）
  ✗ HEAD + 暂存区上跑了 2 个阶段，红 1 个；无对象没跑 1 个：40-none.sh
    → 红的是这一次提交带进来的（别人的未提交改动不在这棵树里）；先查它在不在 HEAD 上就红：暂存区清空再跑一次
EXIT=1
```

✓ 与报告贴的原样输出逐字相同（含 `git status --short` 的 `M  kb/a.md`）。

### 3.4 其余观察（不算 ✗，供主 agent 参考）

- A3 情形三「TaskStop」grep 0 命中：`grep -n "TaskStop" .claude/main-agent.md .claude/agent-common.md` 现跑同样 0 命中。✓ 与报告一致。
- Sonnet 报告没有可供复跑的「引产物」以外的命令性结论性数字需要额外核（`--selftest` 的用量数字「查了 15 个子 agent、1 个进程、8 次看门狗」已随 `--selftest` 原样核过，见 3.3 表格第一行）。

## 四、本地攻方腿（`research/prompts/bg-notify-r1-local-attack-output-s1.md`、`-s2.md`，核对表 `bg-notify-r1-local-attack-translation-audit.md`，运行记录 `bg-notify-r1-local-attack-runlog.md`）

### 4.1 转述核对表逐条核（原文文件:行是否存在、抄的是不是原文、有没有丢/多限定词）

| 核对表行 | 原文文件:行 | 核的结果 |
|---|---|---|
| Rule 1 | `.claude/agent-common.md:40` | ✓ 行号存在，核对表转述的四个分句与原文逐句对得上（「原样交给它」「不再加三词」「不在末尾加 &」「并行再 wait 的例外」） |
| Rule 2 | `.claude/hooks/bash-command-detector.sh:29,31,33-36,47-48` | ✓ 六处行号全部现查存在，内容与转述对得上；「多个单独 & 满足其一即触发」对应第 36 行 `any(...)`，核对 |
| lone `&` 定义 | `research/prompts/_bg-notify-r1-body.md:63` | ✓ 行号存在，五个排除序列（`&&`、`>&`、`&>`、`\|&`、`<&`）逐一核对无误 |
| 第三列问题框架 | `_bg-notify-r1-body.md:55` | ✓ 行号存在；核对表自称「多出一层限定」（is the real long-running work…no longer connected…），现查原文第 55 行确实只有一句朴素问法，没有这层解释——核对表如实标出了这处「多出来的」，没有隐瞒 |
| 事实表 L1–L14 | `_bg-notify-r1-body.md:65,67-80` | ✓ 表头行号、逐行行号都存在；核对表claim「L4 原文带『只有』、L5/L8/L10/L11 不带」，现查 `_bg-notify-r1-body.md` 对应行逐一核实：L4=「无（只有 `2>&1`）」、L5=「无（`2>&1`、`&&`）」、L8=「无（`\|&`）」、L10=「无（`2>&1`）」、L11=「无（`&>`）」——**✓ 与核对表描述一致** |

核对表本身的「文件:行是否存在、是否整行照抄」这一层：**全部 ✓**，没有发现摘句或行号错位。

### 4.2 逐格核对：本地腿样本的答案是否与实测事实一致（★ 关键发现，✗）

主 agent 给了两组独立事实可用来核本地腿的填表结果：① 用真实 hook 对 L1–L14 各判一次的 `bg-notify-r1-hook-truth.tsv`；② 三条 setsid/nohup 计时实测。

**① hook-truth.tsv 先自证**：我在草稿目录独立把 L1–L14 的命令文本逐条喂给仓里真实的 `.claude/hooks/bash-command-detector.sh`（`run_in_background=True`，只统计「run_in_background 里又自己放后台」这一种检出，与 hook-truth.tsv 同一个判据），得到：

```
L1 flag  L2 flag  L3 no   L4 no   L5 no   L6 no   L7 flag
L8 no    L9 flag  L10 flag L11 no  L12 no  L13 flag L14 flag
```

与主 agent 给的 `bg-notify-r1-hook-truth.tsv`（`/tmp/claude-1000/.../scratchpad/bg-notify-r1-hook-truth.tsv`）**14 行逐条相同**。（旁注：我第一次没有区分 hook 的第②种检出「没有 timeout 的等待循环」与第③种「自己放后台」，把 L4 误判成 flag——L4 是 `until … do sleep …; done`，触发的是第②种检出，不是第③种；剔除第②种后 L4=no，与 hook-truth.tsv 一致。这提醒我自己核对时也要看清检出的是哪一种，不只看「有没有记」。）hook-truth.tsv 本身可信。

**逐格比对本地腿两份样本的第②列（hook 判据字面记不记）**：

| L | hook-truth | s1 第②列 | s2 第②列 | 判定 |
|---|---|---|---|---|
| L1–L8, L10–L13 | 与样本答案对照 | 全部一致 | 全部一致 | ✓（12 行） |
| **L9** | **flag** | no | no | **✗（两份样本都错）** |
| **L14** | **flag** | no | no | **✗（两份样本都错）** |

L9 = `curl -s "https://example.invalid/?a=1&b=2" > page.html`，L14 = `bash -c 'sleep 10 &'`。两份样本都写「& 在引号里所以不算单独的 &」——但提示原文给出的 lone `&` 定义（见 4.1「lone `&` 定义」一行）**逐字只排除 `&&`、`>&`、`&>`、`\|&`、`<&` 五种两字符组合，完全没有提到「引号里的不算」**；真实 hook 的正则同样不解析引号，是纯字符匹配。L13 里 `nohup` 在引号里，两份样本都正确地判定「hook 仍然记」（不因为在引号里就豁免），**同一份答案对 `nohup` 关键词不认引号豁免、对 `&` 却认引号豁免，自相矛盾**，而且与 hook 的真实字面判据、与提示自己给的定义都对不上。

**② setsid/nohup 计时实测比对本地腿第③列**：主 agent 给的三条实测（我已独立复现，见二 2.6）：`setsid CMD`（无 `&`）外层等 2.0 秒（不脱钩）、`nohup CMD`（无 `&`）外层等 2.0 秒（不脱钩）、嵌套 `bash -c "bash -c 'sleep 2 &'"` 外层 0.0016 秒（脱钩）。

| L | 命令形态 | 实测应得的第③列 | s1 第③列 | s2 第③列 | 判定 |
|---|---|---|---|---|---|
| **L10** | `setsid bash long.sh … < /dev/null`（无 `&`，无 `-w`） | no（不脱钩，与 `setsid CMD` 实测同构） | yes | yes | **✗（两份样本都错，理由都写成「setsid detaches」）** |
| L14 | `bash -c 'sleep 10 &'`（外层是嵌套 bash -c，内层带 &） | yes（脱钩，与嵌套实测同构） | yes | yes | ✓ |

L10 的两份样本理由分别是「the shell exits immediately after starting setsid」「setsid detaches the process so it continues after shell exit」——这与「裸 `setsid`（不带 `-w`、不带 `&`）不会让外层提前退出」的实测**直接矛盾**。这与 2.6 节 Opus 报告里同一处误解（裸 setsid 会脱钩）同源：两条独立的腿（云端攻方与本地攻方）在同一件事上犯了同一个方向的错，而这件事本轮有现成的实测可以推翻它。

**判定汇总**：L9、L14 的第②列，L10 的第③列，在两份样本里都是**稳定重复的错误**（不是「一次没打中」那种需要二次抽样才能定论的情形——这里我拿到的是与实测事实直接矛盾的、两次抽样一致的错误答案，属于「核对表里的具体条目对不上外部事实」，不是「本地腿这一轮没找到反例」那类需要按抽样纪律再抽一次的情形）。

### 4.3 运行记录产物核（`bg-notify-r1-local-attack-runlog.md`）

| 声明 | 核的结果 | 命令 |
|---|---|---|
| s1 `wc -w` = 925 | ✓ | `wc -w bg-notify-r1-local-attack-output-s1.md` → 925 |
| s2 `wc -w` = 884 | ✓ | 同上 s2 → 884 |
| s1 `oov-check.py` 绿，生词=16 | ✓ 原样 `绿 …s1.md  生词=16 拼接=0`，生词列 `overturned disagrees` | `python3 research/scripts/oov-check.py …-output-s1.md` |
| s2 `oov-check.py` 绿，生词=14 | ✓ 原样 `绿 …s2.md  生词=14 拼接=0`，生词列 `overturned` | 同上 s2 |

## 五、主 agent 自己的两处观测

### 5.1 用 hook 本身对 L1–L14 判定（`bg-notify-r1-hook-truth.tsv`）

见 4.2 节「① hook-truth.tsv 先自证」：**独立复跑与该 tsv 逐行相同，14/14**。可以放心拿它去核别的腿。

### 5.2 三条 setsid/nohup/disown 计时实测

| 命令 | 主 agent 给的数 | 我独立复跑的原样输出 | 判定 |
|---|---|---|---|
| `bash -c 'setsid sleep 2 > /dev/null 2>&1 < /dev/null'` | 2.0 秒 | `elapsed=2.0026` | ✓ |
| `bash -c "bash -c 'sleep 2 &'"` | 0.0016 秒 | `elapsed=0.0017` | ✓ |
| `bash -c 'nohup sleep 2 > /dev/null 2>&1'` | 2.0 秒 | `elapsed=2.0019` | ✓ |

三条都能重复，可以放心拿它们去核 Opus 与本地腿（见二 2.6、四 4.2）。

## 六、计数

下表按每一小节的核查粒度逐项计数（一行代表报告里核过的一组，不再往细拆到每个字符）：

| 部分 | 核了几处 | ✓ | ✗ | 核不动/分不清 |
|---|---|---|---|---|
| Opus：文件:行引用（2.1） | 14 | 14 | 0 | 0 |
| Opus：grep 计数表（2.2） | 1（22 个文件一次性核） | 1 | 0 | 0 |
| Opus：模型文件 sha256（2.3） | 1（6 个哈希一次性核） | 1 | 0 | 0 |
| Opus：run.sh 复跑（2.4） | 1 | 0 | 1（命令本身不可靠重复；数据经拆分复核为真） | 0 |
| Opus：真 harness 独立复核（2.5） | 1 | 1 | 0 | 0 |
| Opus：setsid/nohup 说法（2.6） | 1 | 0 | 1 | 0 |
| Sonnet：文件:行引用（3.1） | 9 | 9 | 0 | 0 |
| Sonnet：check-staged.sh 代码引用（3.2） | 1（4 行一起判） | 0 | 1（混入源码没有的注释） | 0 |
| Sonnet：四个自写脚本复跑（3.3） | 4 | 4 | 0 | 0 |
| Sonnet：仓里脚本 selftest/判红开关复跑（3.3） | 6 | 6 | 0 | 0 |
| Sonnet：红+77 fixture 复跑（3.3） | 1 | 1 | 0 | 0 |
| 本地腿：转述核对表（4.1） | 5 | 5 | 0 | 0 |
| 本地腿：L1–L14 第②列（4.2） | 14×2 份=28 格 | 24 | 4（L9、L14 各两份样本） | 0 |
| 本地腿：L1–L14 第③列（4.2，只核 L10/L14） | 2×2 份=4 格 | 2（L14 两份） | 2（L10 两份） | 0 |
| 本地腿：运行记录产物（4.3） | 4 | 4 | 0 | 0 |
| 主 agent 观测：hook-truth.tsv（5.1） | 1（14 行一起核） | 1 | 0 | 0 |
| 主 agent 观测：三条计时（5.2） | 3 | 3 | 0 | 0 |

**合计：核了 24 组/项（不逐格拆分时的粗粒度计数），细到格的话 L1–L14 两列两份样本共 56 格另计。✓ 数：18 组全对（含 hook-truth 与计时两组主 agent 自证）；✗ 数：4 组（Opus 复跑命令稳定性、Opus setsid 说法、Sonnet 代码引用混入注释、本地腿 L9/L10/L14 三格 × 两份样本）；核不动/分不清：0。**


## 七、没做什么

- 不判一条打中成不成立、该不该采纳；不核推理本身，只核引用、产物与复跑（例：不判「A1① 该不该改写」「A3 情形三算不算真正打中」，那是主 agent 的判决职责）。
- Opus 报告里的大量「模型量过」的具体秒数（X03–X13、F02–F07 等在报告正文表格里的各自秒数）没有逐格重新计时复核，只复核了会决定「判定」列的结构性结论（记/不记、脱钩/不脱钩），以及 2.4/2.5 两处专门挑的样本；理由是这些秒数本身受本机当前负载影响（已在 2.4 节说明 0.1 秒级抖动是正常的），逐格重新计时对判定没有增量信息。
- Opus 报告里标注为「没真跑（推的）」的 X11（`systemd-run`）没有验证，报告自己也说没真跑。
- Opus 报告「五、改法各修哪一格」表格里每一条变体规则改动（`coproc`/`tmux`/`wait` 位置这些正则改法）没有逐条重新推导，只核了变体产出的判定表与自检结果整体对不对（2.4 节已核）。
- Sonnet 报告里 A3「情形三」判定为「入口文本本身规则没说」这一结论本身（是不是该算 A1③ 的同一格、算不算真正打中）不归我判，只核了支撑它的 grep 结果（0 命中，见 3.4）。
- 本地攻方腿的运行记录里「两份样本均干净……未触发第 6 步的五次上限」这句没有另外验证「第 6 步」具体流程是不是在这次调用里被正确执行——只核了 `oov-check.py` 现跑结果与运行记录数字一致（4.3 节）。
- 没有编译 Rust，没有跑门禁阶段（`stage-owners.tsv` 未登记 `three-way-verifier` 名下任何阶段，本轮不改 `crates/`）。
- 没有替主 agent 判断 L9/L10/L14 这三处发现该如何处置（是否要求本地腿重派、是否影响 A2/A3 的最终结论）——只如实报告与实测事实的矛盾，处置由主 agent 定。
- 除报告文件与草稿目录 `/tmp/claude-1000/bg-notify-r1-verifier/` 外未写仓里任何文件；报告的一次定点替换（清理第六节开头一段有算术错误的文字）用 `research/scripts/replace-once.py`，命中 1 次、已写回并回读确认。
