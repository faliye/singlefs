# 第十批 landing-audit-b10-replay-decode：开关复跑格遇坏字节不崩、subagent_transcript.py 平常跑设开关就拒

日期：2026-10-23。依据：`records/2026-09-16-subagent拆分提案.md` 第九十节 ㉛ 那一行。

## 结论

两条都达成验收：

1. `.claude/gate.d/code-tooling.sh` 的 break-switch-replay 格改成按字节收自证输出，stdout、stderr 各自照 UTF-8 严格解。解不了的那一条记一条红，种类叫「坏字节」，红行点名路径、开关名与值、哪一路、第几字节和那几个字节，并给出路；别的条照判照报，不再整格抛 `UnicodeDecodeError`。加了红样本 `break-switch-replay-undecodable-red`，加了弄坏开关 `CODE_TOOLING_BREAK=replay-output-strict-decode`。
2. `scripts/agents/subagent_transcript.py` 加了 `refuse_break_outside_selftest()`，`main()` 解析参数之前第一句先调它：直接执行、参数不是单独一个 `--selftest`、又设了 `SUBAGENT_TRANSCRIPT_BREAK` 的，往 stderr 打一行点名变量「只在自证里认」并退 2。被 import 时照认这个变量（三处调用方的自证要用它）。自证加了一格「平常跑设开关要被拒」，加了弄坏开关 `SUBAGENT_TRANSCRIPT_BREAK=honor-break-outside-selftest`。

推翻条件：主 agent 在主工作区跑「要主 agent 跑的命令」里的样本档或几格时，下面任一种出现，结论就不成立：
- `break-switch-replay-undecodable-red`、`break-switch-replay-red`、`break-switch-replay-green` 有一份判错；
- break-switch-outside-selftest 格仍点名 `scripts/agents/subagent_transcript.py`；
- break-switch-replay 格里 `subagent_transcript.py` 三条开关有一条不是「对」。

预计会有的变化（不算推翻）：真仓上 break-switch-replay 格从整格崩，变成对 `scripts/hosts/run-build-directory.sh RUN_BUILD_DIRECTORY_BREAK=owner-left-alone` 记一条「坏字节」红，别的条照判。这一条要等管 `run-build-directory.sh` 的那个会话改到只吐 UTF-8（或从登记表删掉那一行）才转绿。`run-build-directory.sh` 我没碰。

## 改过的文件

- `.claude/gate.d/code-tooling.sh`：用 `scripts/kb/replace-once.py` 做了 6 处定点替换（先在草稿副本里验，再照同样的替换打到主工作区；换完和草稿副本 diff 为空）。
  - 文件头第 82 行：样本清单加 `break-switch-replay-undecodable-red`。
  - 文件头第 139–140 行：开关表加 `replay-output-strict-decode`。
  - 格上方注释第 4068–4069、4076–4077、4079–4080 行：判据加「坏字节」，写了样本与开关。
  - `attempt()` 第 4167–4182 行：起子进程与收输出。
  - 出路那句第 4231–4232 行。
  - 格之外的部分没动。这份文件里另有别的会话 1 行没暂存的改动，原样留着。
- `.claude/gate.d/fixtures/code-tooling.sh/break-switch-replay-undecodable-red/`：新建，共 7 个文件。
  - `.gate-cells`、`expect`；
  - `.claude/gate.d/break-switch-replay-files.tsv`、`.claude/hooks/selftest-registry.tsv`；
  - `scripts/gate/garbled.py`（新写）；
  - `scripts/gate/okay.py`、`scripts/gate/silent.py`（从 break-switch-replay-red 原样拷来）。
- `scripts/agents/subagent_transcript.py`：用 `replace-once.py` 做了 9 处定点替换。
  - 文档串：用法后面加守卫一段，开关表加第 37–38 行。
  - `KNOWN_BREAK_MODES` 第 50 行。
  - `import subprocess`。
  - `refuse_break_outside_selftest()` 第 62 行起。
  - 自证新格第 304–319 行。
  - 自证的出路句与成功句。
  - `main()` 第 333 行起。

`git diff --stat` 原样（code-tooling.sh 那 31 行里含别的会话那 1 行）：

```
 .claude/gate.d/code-tooling.sh        | 31 +++++++++++++++++++++-----
 scripts/agents/subagent_transcript.py | 41 ++++++++++++++++++++++++++++++++---
 2 files changed, 63 insertions(+), 9 deletions(-)
```
新样本目录还没进 git（`?? .claude/gate.d/fixtures/code-tooling.sh/break-switch-replay-undecodable-red/`）。

改完的 sha256：
- `cb830a506b7e36f408734b16cca759b72e41b78d343e17a864c9a9ac3bd4c5ec  .claude/gate.d/code-tooling.sh`
- `4d9678c3052777a6ed43f8c70df9ce651dd487263247f74eda077c7ba4748033  scripts/agents/subagent_transcript.py`

没新写 `# gate-similar:` / `# hook-events:` 行：没有新门禁阶段，也没有新钩子。

## 第一条：break-switch-replay 格遇坏字节

### 样本 `break-switch-replay-undecodable-red`

登记表里有三份，都登记成 light：
- `garbled.py` 有两条开关：
  - `GARBLED_BREAK=bytes`：失败行原文（点名的那句）后面跟 `ff fe e2 9c` 坏字节，退 1；
  - `GARBLED_BREAK=plain`：照常红在点名的行上。
- `okay.py` 全对。
- `silent.py` 退 0。

期望：
- 退 1；
- 「坏字节，stdout 里有不合法的 UTF-8」与 silent 那条「没红」各一条红；
- 汇总「2 处不对（复跑了 4 条、3 份）」，出路里有「输出带不合法 UTF-8 的」。

坏字节那条的点名句本身就在失败行里。所以只做容错解码、不记红的写法，同样会让这份样本判错。

### 改前的读数

喂法照 stage-selftest 旧形态：拷进临时目录、cd 进去、`bash <阶段> <目录>`，比 exit= 与每条 want=。草稿脚本是 `feed-one.sh`。主工作区改前的阶段：

```
break-switch-replay-undecodable-red：错（退 1）
UnicodeDecodeError: 'utf-8' codec can't decode byte 0xff in position 45: invalid start byte
```

### 改后的读数（主工作区的阶段）

```
break-switch-replay-undecodable-red：对（退 1）
break-switch-replay-red：对（退 1）
break-switch-replay-green：对（退 0）
```

新样本的红行原样：

```
  ✗ scripts/gate/garbled.py GARBLED_BREAK=bytes：坏字节，stdout 里有不合法的 UTF-8（第 45 字节起：ff fe e2 9c 0a），点名的项比不了；改那份脚本让它在这条开关下只吐 UTF-8
  ✗ scripts/gate/silent.py SILENT_BREAK=noop：没红，设上之后自证照样退 0（开关没弄坏什么）
  ✗ 脚本与钩子的弄坏开关复跑：2 处不对（复跑了 4 条、3 份）
```

### 弄坏开关

`CODE_TOOLING_BREAK=replay-output-strict-decode` 退回 `text=True` 的收法。主工作区阶段的读数：

```
break-switch-replay-undecodable-red：错（退 1）    ← 整格又抛 UnicodeDecodeError，四条 want 找不到
break-switch-replay-red：对（退 1）
break-switch-replay-green：对（退 0）
```

原有的 `CODE_TOOLING_BREAK=replay-items-unchecked` 照旧只翻 `break-switch-replay-red`（草稿副本上量的）：

```
break-switch-replay-undecodable-red：对（退 1）
break-switch-replay-red：错（退 1）
break-switch-replay-green：对（退 0）
```

code-tooling.sh 整道登记在 `.claude/gate.d/stage-break-switch-replay-exempt.tsv`，stage-break-switch-replay 格不复跑它的头注。新开关的头注照这一道原有的 `CODE_TOOLING_BREAK=<项>` 写法写。

## 第二条：subagent_transcript.py 平常跑设开关就拒

### 改前的读数（break-switch-outside-selftest 格的探针起法）

```
$ env SUBAGENT_TRANSCRIPT_BREAK=refused-counts-delivered python3 -B scripts/agents/subagent_transcript.py --force --break-guard-probe-unknown-option </dev/null
  ✗ 认不出的用法：--break-guard-probe-unknown-option
     → 怎么办：import 它再调 handback_delivered()、HandbackTracker、first_record_moment()、started_between()；直接执行只认 --selftest
exit=2
```

### 先造红

先只加自证新格、不加守卫，跑主工作区的 `--selftest`：

```
  ✗ subagent_transcript 自证：平常跑设开关要被拒：设上 SUBAGENT_TRANSCRIPT_BREAK=refused-counts-delivered 应当退 2 并点名它只在自证里认、不设不点名，实际设上退 2、点名 0 行，不设点名 0 行
exit=1
```

### 改后的读数

```
$ env SUBAGENT_TRANSCRIPT_BREAK=refused-counts-delivered python3 -B scripts/agents/subagent_transcript.py --force --break-guard-probe-unknown-option </dev/null
  ✗ SUBAGENT_TRANSCRIPT_BREAK=refused-counts-delivered 只在自证里认（只给 --selftest 证红用），这一趟不是 --selftest，不往下走
     → 怎么办：去掉 SUBAGENT_TRANSCRIPT_BREAK 再跑；要证红就跑 SUBAGENT_TRANSCRIPT_BREAK=<项> python3 scripts/agents/subagent_transcript.py --selftest
exit=2
```

不设变量时照旧报「认不出的用法」，退 2，没有点名变量的那一行。

`python3 scripts/agents/subagent_transcript.py --selftest`：

```
  ✓ subagent_transcript 自证通过（查了 13 项）：…；直接执行不是 --selftest 而设了开关的被拒、退 2
exit=0
```

### 自证新格与弄坏开关

自证新格起自己一次，参数是 `--break-guard-probe-unknown-option`，分设上、不设两次：
- 设上时要退 2，输出里要有点名变量且带「自证」的行；
- 不设时不许有这一行。

`SUBAGENT_TRANSCRIPT_BREAK=honor-break-outside-selftest` 让守卫不拒。自证给子进程设的值，是父进程里的 honor 值（父进程设了它时）或 `refused-counts-delivered`。读数：

```
  ✗ subagent_transcript 自证：平常跑设开关要被拒：设上 SUBAGENT_TRANSCRIPT_BREAK=honor-break-outside-selftest 应当退 2 并点名它只在自证里认、不设不点名，实际设上退 2、点名 0 行，不设点名 0 行
exit=1
```

照 break-switch-replay 格的判法（读法取 code-tooling.sh 里 `BREAK_SWITCH_TABLE_LIBRARY` 的原文，草稿脚本是 `replay-one.py`），复跑了这份文件头的三条开关：

```
文件头开关 3 条；没写全的 0 条
SUBAGENT_TRANSCRIPT_BREAK=refused-counts-delivered：退 1，点名 3 项，没在失败行里 0 项 → 对
SUBAGENT_TRANSCRIPT_BREAK=start-by-mtime：退 1，点名 3 项，没在失败行里 0 项 → 对
SUBAGENT_TRANSCRIPT_BREAK=honor-break-outside-selftest：退 1，点名 1 项，没在失败行里 0 项 → 对
```

import 这份库的三处调用方，自证都是 light，在主工作区照跑，都退 0：
- `python3 scripts/agents/agent-cost.py --selftest`：「✓ 自检通过（查了 23 项…」
- `bash .claude/hooks/continuation-guard.sh --selftest`：「✓ 自检通过（查了 20 种）…」
- `python3 scripts/agents/agent-watch.py --selftest`：「✓ agent-watch 自检通过…」

语法检查：`bash -n .claude/gate.d/code-tooling.sh` 退 0；`python3 -m py_compile scripts/agents/subagent_transcript.py` 通过。

## 要主 agent 跑的命令

工作目录都是主工作区 `/home/fy5090/code/singlefs`，不起重活，不用内存包装与线程上限。

1. code-tooling.sh 的整份样本档（登记为 samples-light；我只照喂法逐份核了 break-switch-replay 的三份，没喂整道）：
   `bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh $PWD/.claude/gate.d --item code-tooling.sh`
   看末尾「判得都对」；判错的每份各有一行 `✗ code-tooling <样本> …`。
2. 真仓上的复跑方和相邻两格：
   `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 break-switch-replay、break-switch-outside-selftest、break-switch-headers 三格; 范围=3 格; 为什么=第十批改了复跑格的收输出与 subagent_transcript.py 的守卫，核真仓上这三格对它们的判法' bash .claude/gate.d/code-tooling.sh --check break-switch-replay,break-switch-outside-selftest,break-switch-headers /home/fy5090/code/singlefs`
   看三处：
   - break-switch-outside-selftest 不再点名 `scripts/agents/subagent_transcript.py`；
   - break-switch-replay 不再抛 `UnicodeDecodeError`。预计只记一条 `scripts/hosts/run-build-directory.sh RUN_BUILD_DIRECTORY_BREAK=owner-left-alone：坏字节，…` 红（归管 run-build-directory.sh 的那个会话），`subagent_transcript.py` 三条都不出现在红行里；
   - break-switch-headers 不点名 `subagent_transcript.py`。
3. 研究脚本那一路的拒绝出路与语法（守卫与自证新格各有一处 `✗` 带 `→`）：
   `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 research-gate-lint 格; 范围=1 格; 为什么=subagent_transcript.py 加了拒绝行与自证格，核出路与语法' bash .claude/gate.d/code-tooling.sh --check research-gate-lint /home/fy5090/code/singlefs`
4. 提交前照常经 `scripts/gate/gate-staged.sh` 跑整轮。新样本目录 `.claude/gate.d/fixtures/code-tooling.sh/break-switch-replay-undecodable-red/` 要一起暂存。

## 没做什么

- 没跑重型测试：checker-tier-crash-point-replay、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification、checker-tier-research-build-and-replay 本身、checker-tier-broad-mutation 与 `scripts/mutation/broad-mutation.py` 起 cargo 的用法、`gate.sh`、全量 `cargo test`、`check.sh`。
- 没跑门禁阶段本身（上面三条命令交主 agent），也没跑 stage-selftest 整道和各类 lint。
- 命名检查没跑：这一批没改 `.rs`。
- 没走定义三方：这一批是第一种活，不改定义、共用约束与规则。没提交。
- 没改 `scripts/hosts/run-build-directory.sh`，也没碰 break-switch-replay 格之外的部分、`.claude/hooks/`、别的 `scripts/agents/` 文件、`records/`、`research/`。
- 草稿：
  - 副本 `tree/`（364K，code-tooling.sh 拷贝加几个指回主工作区的符号链接）用完删了。
  - 留在草稿目录的有 `feed-one.sh`、`replay-one.py`、`edits/`（各处替换的旧串与新串）和各次读数的 `.out`，都不是仓副本或编译目录。
- 开跑前看过负载（`ps -o pid,args -u "$(id -u)"`）：只有别的会话的 cargo 与 crates 变异门禁，没有性能测量；这一批不编译，都在本机跑。
