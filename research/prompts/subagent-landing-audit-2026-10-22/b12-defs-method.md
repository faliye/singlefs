# 第十二批任务 45：验证段重跑命令写法与判决 cite-check 的根（tooling-writer，2026-10-23）

批次：`records/2026-10-22-子agent结果落地审计.md#12`。关的是 `records/2026-09-16-subagent拆分提案.md` 第九十节 ㉞ 里的两件（验证段第 1 步、判决 cite-check 的根）与兄弟任务 44 带出的 tooling-writer 第 3 点。三条都做到了验收。

## 改过的文件

| 文件 | 改了什么 | 怎么改 |
|---|---|---|
| `.claude/main-agent.md` | 验证段第 1 步命令带上两段前缀、写出脚本打出的只重跑快档那几格的命令怎么跑；第 4 步改成与第 1 步同一套；第 5 步括注跟着对齐 | `scripts/kb/replace-once.py` 四次定点 |
| `.claude/rules/three-way-inference.md` | 「核查与判决由主 agent 做」第 1 步：先倒推树、`--root` 给它、两条命令写进判决 | `replace-once.py` 一次 |
| `.claude/agents/tooling-writer.md` | 「做什么」第 3 点补一句：`scripts/` 下被当写工具用的脚本换上也经 `hook-swap.py` 的脚本规格 | `replace-once.py` 一次 |
| `scripts/agents/snapshot-tree.py` | 新建：开工快照倒推成一棵树 | Write（先 `ls` 确认不在） |
| `scripts/agents/README.md` | 表里加 `snapshot-tree.py` 一行 | `replace-once.py` 一次 |
| `.claude/gate.d/code-tooling.sh` | `SELFTEST_RUNNERS` 加一行 `"python3 scripts/agents/snapshot-tree.py --selftest"` | `replace-once.py` 一次（新 inode），`bash -n` 过 |
| `.claude/hooks/selftest-registry.tsv` | 新脚本登记 light | Edit 一次 |

`git diff --stat -- <这些文件>` 原样（`.claude/gate.d/code-tooling.sh`、`selftest-registry.tsv`、`main-agent.md`、`three-way-inference.md` 里含别的会话没提交的改动；新脚本未跟踪，不进 stat）：

```
 .claude/agents/tooling-writer.md     |  2 +-
 .claude/gate.d/code-tooling.sh       | 59 +++++++++++++++++++++++++-----------
 .claude/hooks/selftest-registry.tsv  |  3 +-
 .claude/main-agent.md                |  8 ++---
 .claude/rules/three-way-inference.md |  4 +--
 scripts/agents/README.md             |  1 +
 6 files changed, 52 insertions(+), 25 deletions(-)
?? scripts/agents/snapshot-tree.py
```

只算这一件自己的改动（与改前留底 `originals/` 比，`python3 scripts/agents/copy-originals.py check` 报「改过 5 份」）：main-agent.md 6 行进出、three-way-inference.md 2、tooling-writer.md 2、README.md 1、code-tooling.sh 1 行加（同一份里另有 9 行进出是别的会话在我改的那一段时间里落的 research-gate-lint 注释，没动）。别的会话原有的改动（main-agent.md 第 11 条与长活豁免表一句、three-way-inference.md 判决复跑第 3 步）都还在，没碰。

## 第一条：验证段第 1 步与第 4 步（`.claude/main-agent.md`）

改前改后 diff（`diff -u originals/.claude/main-agent.md .claude/main-agent.md`，长行原样）放在报告末尾一节「附：main-agent.md 改前改后」。改了的三处：

- 第 1 步命令代码段改成 `SINGLEFS_RUN_SCOPE='种类=whole; 跑什么=合入后验证; 范围=整轮; 为什么=<这一批补丁碰了哪些包>' SINGLEFS_HEAVY_TESTS=user-request bash scripts/gate/merge-and-verify.sh [--skip-apply] [--memory 上限] [--threads N] <补丁目录…>`，与第 10 条、调度表合入后验证那一行逐字一致（用 python 抽三处代码段放进一个集合，集合只有 1 个元素）。
- 第 1 步加一句：这一步红了、本次未跑（`--peer`、`--copy`、显存不够）或没带前缀时，脚本在「→ 怎么办：」那一行里打出只重跑快档那几格的整条命令（开头已带前缀与范围声明，线程与内存包装已拼好），主 agent 照原样带前缀跑、不弹窗，不改写、不另拼前缀。四种情形对到 `scripts/gate/merge-and-verify.sh` 打这条命令的 678（--peer）、684（--copy）、697（没带前缀）、710（显存不够）、735（测试红）行。
- 第 4 步：改完照第 1 步同一套再验，先只重跑上次红的与这一次改到的——编译、测试红的照 `rerun-failed-tests.py <上次的 test.log> -p <包>` 打的命令跑，快档红的照第 1 步那条脚本打的命令跑，这一次改到的照第 1 步那条命令只给这一批修补的补丁目录跑；整段的整轮重跑留到提交前。
- 第 5 步括注「整体再验一次绿」改成「照第 4 步重跑的都绿」：派发没点名这一步，不改它就与新第 4 步矛盾，算我加的对齐，主 agent 不认就反着替换回去（旧串新串都在附的 diff 里）。

验收：`grep -n 'merge-and-verify.sh' .claude/main-agent.md` 命中 6 行（7、22、29、44、52、96）；逐个抽反引号代码段判：

```
7 prefix _TESTS=user-request bash scripts/gate/merge-and-verify.sh [--skip-apply] <补丁目录…>
22 mention scripts/gate/merge-and-verify.sh
29 prefix ipts/gate/merge-and-verify.sh [--skip-apply] [--memory 上限] [--threads N] <补丁目录…>
44 mention scripts/gate/merge-and-verify.sh
52 prefix ipts/gate/merge-and-verify.sh [--skip-apply] [--memory 上限] [--threads N] <补丁目录…>
96 prefix ipts/gate/merge-and-verify.sh [--skip-apply] [--memory 上限] [--threads N] <补丁目录…>
```

四处命令都带两段前缀；22、44 两处只点脚本名，不是命令。第 7 行（「职责」一节）的命令带前缀，但参数只写了 `[--skip-apply] <补丁目录…>`，派发没让改它，没动。

五处例外（`.claude/agent-common.md:58`、`.claude/rules/implementation-workflow.md:45`、`:47`、`.claude/main-agent.md:22`、`.claude/hooks/heavy-test-guard.sh:871`）逐处读过：都说「带 `SINGLEFS_HEAVY_TESTS=user-request` 起的 checker 档快档那一步与它给出的只重跑那几格的命令（那一步红了、本次未跑或没带前缀时打出的那一条）」跑法照验证段第 1 步（heavy-test-guard.sh 那处写「跑法指到」）。第 1 步现在两样都写了：快档那一步就是第 1 步那条带前缀的命令；打出的那一条照原样带前缀跑、不弹窗。

推翻条件：主 agent 定了「修完一律整条重跑」，第 4 步就该退回，`merge-and-verify.sh:735`「改完只重跑那几格」与五处括注也跟着收窄。

## 第二条：判决里的 cite-check 对哪棵树跑

### 新脚本 `scripts/agents/snapshot-tree.py`

`snapshot-tree.py <轮名> --out <目录> [--repository <仓根>] [--list <清单文件名> …]`：读 `research/prompts/<轮>-snapshot/` 的 `judged-sha256.txt`（`--list` 可换成或加上同目录别的清单，例 `kb-sha256.txt`）与 `judged-vs-head.diff` / `judged-vs-index.diff`（两种都认，两份都在算用法错）；只铺清单点名的文件。diff 里有的：基底取 diff 头 `index <基底>..<结果>` 的基底 blob（全 0 是新文件；此刻 HEAD 正是这个 blob 的记「HEAD」，不是的照 blob 号在对象库里取，取不到报「倒推不出」；改名按旧路径取），在单独的临时目录里 `git apply` 打上这一段；diff 里没有的取此刻 HEAD，HEAD 里没有或 sha256 不是快照那一版的报「倒推不出」。逐份 sha256 与清单比，对不上的点名；倒推不出、对不上的都不铺进树。退出码 0 全对上、1 有倒推不出或对不上、2 用法错（`--out` 不空或落在仓里也是 2）。成功时多打一行判决里抄的 cite-check 命令。

文件头：`# admission: always 每次都拿此刻的 git 对象库与快照目录现铺一棵树，上一次铺出的树不替这一次作保`、`# run-condition: command git`；`__main__` 先判弄坏开关守卫、再调 `preflight(__file__)`（照 `cite-check.py`、`copy-originals.py` 的写法）。文件头写了与 `scripts/devices/snapshot-check.sh` 的分工（它只核不铺树）。不是门禁阶段、不是钩子，不写 `# gate-similar:` / `# hook-events:`；`gate-overlap.py --list` 里没有判同一件事的（搜 snapshot、快照、倒推，只有 `snapshot-check.sh` 与 doc-process-records 的说明里提到快照核对）。

### 自证与弄坏开关

`python3 scripts/agents/snapshot-tree.py --selftest`（登记 light），原样：

```
  ✓ snapshot-tree 自检通过（查了 21 项）：快照之后有提交的按 diff 头记的基底 blob 倒推、新文件、改名、非 ASCII 引号路径、diff 里没有的取 HEAD，铺出的树逐份 sha256 与清单相同；改坏的 diff 判对不上、基底 blob 取不到与 diff 里没有又被提交改过的判倒推不出，都退 1、不铺进树；对暂存区的旧轮照认；用法错 6 种退 2，--help 退 0；平常跑设了弄坏开关退 2 并点名只在自证里认
```
退出码 0。造法：临时目录里 `git init` 一个小仓，提交基底，再造开工那一刻的未提交改动（改两份、一份非 ASCII 文件名 `records/记录.md`、一份带执行位、一份新文件、一次 `git mv` 加改动），记清单与 `git -c core.quotepath=true diff HEAD -M`；另造一轮对暂存区（暂存 x2、工作区 x3，`git diff`）；之后再提交一轮改动，让 HEAD 不再是快照时那一版。

样本红绿（都是自证里的格，命令行同一份 `run()`）：

| 样本 | 期望 | 实测 |
|---|---|---|
| 快照之后有提交（7 份） | 退 0，倒推出 7 份、对上 7 份；树里逐份重算 sha256 与清单相同 | 绿 |
| 对暂存区的旧轮 | 退 0，基底取 diff 头的 blob | 绿 |
| 改坏的 diff（`+A2` 改成 `+A9`） | 退 1，点名 kb/one.md「sha256 对不上」，不铺进树 | 红（判对） |
| 基底 blob 号改成不在对象库里的 | 退 1，点名「倒推不出：diff 头记的基底 blob … 取不到」 | 红（判对） |
| diff 里没有、快照之后被提交改过 | 退 1，点名 kb/later.md「倒推不出」 | 红（判对） |
| 用法错 6 种、`--help` | 退 2 / 0 | 判对 |
| 平常跑设了开关（子进程起） | 退 2 并点名只在自证里认 | 判对 |

弄坏开关 `SNAPSHOT_TREE_BREAK=<项>`（文件头开关表照 mutate.sh 写法：一行开关名与弄坏哪一处，下一行起「」里写失败行原文），逐个打开跑自证：

| 开关 | 弄坏哪一处 | 自证退出码 | 点名的项 |
|---|---|---|---|
| `skip-sha-check` | `lay_file()` 不比 sha256 | 1 | 3 项全在失败行里 |
| `head-only` | `base_for()` 基底一律取此刻 HEAD | 1 | 8 项全在 |
| `no-unquote` | `unquoted()` 不解 git 引号路径 | 1 | 4 项全在 |
| `index-diff-ignored` | `diff_path()` 不认 judged-vs-index.diff | 1 | 2 项全在 |
| `guard-off` | `refuse_break_outside_selftest()` 不拒 | 1 | 1 项在 |

「全在」是拿草稿目录里的 `check-named.py` 照 break-switch-replay 格的读法（下一行起「」原文、按 … 切段、要落在同一条 ✗ 行里）逐项核的，末行「missing 0」。平常跑设了开关：`SNAPSHOT_TREE_BREAK=head-only python3 scripts/agents/snapshot-tree.py landing-audit-r6 --out <草稿>/x` 退 2，原样：

```
  ✗ SNAPSHOT_TREE_BREAK=head-only 是弄坏开关，只在自证（--selftest）里认；平常跑设了就拒，这一次什么都没做
     → 怎么办：unset SNAPSHOT_TREE_BREAK 再来；要看开关红在哪一格，跑 SNAPSHOT_TREE_BREAK=<项> python3 scripts/agents/snapshot-tree.py --selftest
```
`<草稿>/x` 没建。`python3 -m py_compile` 过（编出的 pyc 当场删了）。

挂进去的两处：`.claude/gate.d/code-tooling.sh` 的 `SELFTEST_RUNNERS` 新一行 `  "python3 scripts/agents/snapshot-tree.py --selftest"`（在 cite-check 那一行后面）；`.claude/hooks/selftest-registry.tsv` 新一行 `scripts/agents/snapshot-tree.py	light	# 临时目录里 git init 一个小仓、造快照清单与 diff，进程内调自己铺树，只起 git（init、add、commit、mv、diff、rev-parse、cat-file、apply），不起 cargo、编译、虚机`。没往 `.claude/gate.d/break-switch-replay-files.tsv` 加行（派发没点名这张表），break-switch-replay 格会把它列成「没接复跑的」一份并报本次未跑。

### 验收：landing-audit-r6

`python3 scripts/agents/snapshot-tree.py landing-audit-r6 --out <草稿目录>/r6-tree` 退 0，末两行原样：

```
  ✓ 快照 landing-audit-r6：清单点名 40 份，倒推出 40 份，sha256 对上 40 份（基底取自 HEAD 19 份、diff 头记的 blob 0 份、新文件 21 份；diff 用的是 judged-vs-head.diff）；树在 /home/fy5090/.local/share/singlefs/work/landing-audit-b12-defs-method/r6-tree
  判决里的 cite-check 写成：python3 scripts/agents/cite-check.py <全部腿报告> --root /home/fy5090/.local/share/singlefs/work/landing-audit-b12-defs-method/r6-tree --background <背景材料>
```

倒推出 40 份、sha256 对上 40 份。这一轮的基底都还是此刻 HEAD（19 份）或新文件（21 份），「HEAD 之后被提交改过、按 diff 头 blob 取」那一路在 r6 上没走到，只在自证里走过。

在这棵树上跑 `python3 scripts/agents/cite-check.py research/prompts/landing-audit-r6-sonnet-output.md research/prompts/landing-audit-r6-opus-output.md --root <草稿目录>/r6-tree --background research/prompts/_landing-audit-r6-background.md`，退 0，末行原样：

```
  ✓ 核了 15 处引文、144 处裸行号，对上 11 处，没判 148 处（其中裸行号没有引文可比 20 处；逐处列在上面）
```

对不上 0 处。两份腿报告此刻的 sha256 与判决记的相同（`0f8c181e…`、`faf19c87…`）。和判决比：

| 跑法 | 结果 |
|---|---|
| 判决记的（`--root .`，写判决那一刻） | 核了 15 处引文、144 处裸行号，对上 15 处，没判 144 处 |
| 此刻 `--root .` | 7 处对不上（handback-guard.sh、agent-common.md、lib_three_way_legs.py、lib_prompt_scratch_line.py 后来改过、行号挪了），144 处没判，退 1 |
| 倒推出的树 | 对不上 0 处、对上 11 处；15 处引文里 4 处列进没判：opus 报告第 263 行引的 `landing-audit-r4-main-verification.md`、`landing-audit-r5-main-verification.md`、`.claude/rules/three-way-inference.md`，第 274 行引的 `_landing-audit-r6-body.md`，都不在快照里（「在给的根下都找不到」）；裸行号 124 处路径认不出（只写文件名的在树里没有 git 可查，或被引文件不在快照里），只剩 20 处可比 |

再加 `--list judged-sha256.txt --list kb-sha256.txt` 铺 kb 那 4 份：退 1，4 份 kb 都「倒推不出：diff 里没有它，此刻 HEAD 那一份 sha256 … 不是快照的 …」（快照之后 kb 被提交改过、diff 里又没有 kb 的基底）；cite-check 读数不变。

推翻条件：有一份判决当时的读数在这棵树上复出对不上（说明树铺错了），或腿引到的、在快照里的文件在树上被判「找不到」。

### 旧轮顺手跑的（不是验收，给主 agent 判要不要管）

| 轮 | 结果 |
|---|---|
| landing-audit-r1（对暂存区） | 点名 51，倒推出 16、对上 13；倒推不出 35、对不上 3 |
| landing-audit-r2（对暂存区） | 点名 63，倒推出 25、对上 24；倒推不出 38、对不上 1 |
| landing-audit-r3（对暂存区） | 点名 74，倒推出 38、对上 36；倒推不出 36、对不上 2 |
| landing-audit-r4 | 点名 80，倒推出 68、对上 64；倒推不出 12、对不上 4 |
| landing-audit-r5 | 点名 27，倒推出 17、对上 14；倒推不出 10、对不上 3 |
| apparatus-rename-r1 | 点名 14，倒推出 12、对上 12；倒推不出 2 |
| apparatus-rename-r2 | 点名 15，diff 0 段，15 份都倒推不出（快照之后都被提交改过） |

r4 倒推不出的 12 份全是「diff 打不上」。查了 `.claude/main-agent.md` 那一份：基底 blob `9d333a94a` 第 83 行是 `` `scripts/experiments/replay.sh` ``，diff 里同一行上下文写的是 `` `scripts/experiment-apparatus/replay.sh` ``——搬迁那一趟把快照目录里的 diff 也换了路径，上下文对不上基底。这类轮要倒推得先把 `.claude/gate.d/path-moves.tsv` 登记的搬迁在 diff 上反着换一遍，脚本没做（派发没要）；对暂存区的旧轮另有清单点名而 diff 里没有、此刻 HEAD 里也没有的（r1 有 21 份），按派发的取法倒推不出。

### `.claude/rules/three-way-inference.md`「核查与判决由主 agent 做」第 1 步

改后整句：

> 1. 先跑 `python3 scripts/agents/snapshot-tree.py <轮> --out <草稿目录>/<轮>-tree`，照 `research/prompts/<轮>-snapshot/` 的 `judged-sha256.txt` 与对 HEAD（旧轮对暂存区）的 diff 倒推出开工快照那一刻的树、逐份核 sha256；再跑 `python3 scripts/agents/cite-check.py <全部腿报告> --root <草稿目录>/<轮>-tree --background <背景材料>`，`--root` 给倒推出的这棵树，两条命令原样写进判决：它判了的（对不上、指到标题行、写成背景材料的行号）照它的判定写进判决；它列成「没判」的与认不出写法的引文，逐处人工核；`snapshot-tree.py` 点名倒推不出、sha256 对不上的那几份，判决里写明是哪几份，腿引到它们的行逐处人工核。

改前是 `1. 先跑 \`python3 scripts/agents/cite-check.py <全部腿报告> --root <快照根> --background <背景材料>\`：它判了的（…）照它的判定写进判决；它列成「没判」的与认不出写法的引文，逐处人工核。`。最后半句（倒推不出的那几份写明、逐处人工核）是我补的出路：脚本退 1 时第 1 步要有下一步；主 agent 不要可以删掉那半句。

## 第三条：`.claude/agents/tooling-writer.md`「做什么」第 3 点

在「改 `.claude/hooks/` 下的钩子只经 `scripts/gate/hook-swap.py` 换上（草稿里先验、换上后再验、不对按反向定点替换回滚），不直接定点改」之后补一句：

> `scripts/` 下被各会话当写工具用的脚本（`scripts/kb/` 那几份定点写入脚本这类）换上也经它，走它的脚本规格（文件都在 `scripts/` 下，不带 `--hook` 与 `--feeds`，`--verify` 给被换脚本的 `--selftest`）。

脚本规格的写法照 `scripts/gate/hook-swap.py` 文件头用法第二行与 `--verify` 那一段，和兄弟任务 44 的报告 `research/prompts/subagent-landing-audit-2026-10-22/b12-kb-scripts.md` 节「条目三（㉟）：hook-swap.py 收 scripts/ 下的文件」一致。

## 改过的定义、共用约束与规则（交主 agent 开定义三方）

1. `.claude/main-agent.md`：「写与验分两段」验证段第 1、4、5 步。
2. `.claude/rules/three-way-inference.md`：「核查与判决由主 agent 做」第 1 步。
3. `.claude/agents/tooling-writer.md`：「做什么」第 3 点。

共用约束 `.claude/agent-common.md` 没改。

## 没做什么

- 没跑重型测试（`checker-tier-crash-point-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification、checker-tier-research-build-and-replay 本身、checker-tier-broad-mutation 与 `scripts/mutation/broad-mutation.py` 起 cargo 的用法、`gate.sh`、全量 `cargo test`、`check.sh`）；没走定义三方；没提交。
- 没跑门禁阶段与各类 lint（code-tooling 的 research-script-selftests、break-switch-headers、break-switch-outside-selftest 等格，rules-lint、doc-lint 归提交前）；只跑了 `gate-overlap.py --list`、新脚本的 `--selftest` 与五个开关、`bash -n .claude/gate.d/code-tooling.sh`、`python3 -m py_compile`。没改 `.rs`，命名检查不用跑。
- 没往 `.claude/gate.d/break-switch-replay-files.tsv` 登记新脚本（派发没点名）；要接复跑的，加一行就行，五个开关点名的项已核过都在失败行里。
- 没给旧轮做「反着换搬迁」的倒推；r1–r5、apparatus-rename 两轮按派发的取法倒推不全（见「旧轮顺手跑的」）。
- `main-agent.md` 第 7 行「职责」一节的命令参数写法比第 10 条短，没动；第 5 步括注是我为对齐第 4 步加的改动，派发没点名。
- 倒推出的 r6 树（3.1M）交回前删了；要用时照第二条的命令重铺。

## 清理

草稿目录 `~/.local/share/singlefs/work/landing-audit-b12-defs-method/` 里没有仓副本与编译目录；留着的是报告、留底 `originals/`（5 份改前原文与清单）、自证与开关的输出、`check-named.py`、cite-check 三份输出与 `main-agent.diff`、`mine-*.diff`。selftest 的临时仓在 `TMPDIR` 下，跑完自己删了。

## 附：main-agent.md 改前改后

`diff -u originals/.claude/main-agent.md .claude/main-agent.md` 原样：

```diff
--- /home/fy5090/.local/share/singlefs/work/landing-audit-b12-defs-method/originals/.claude/main-agent.md	2026-10-23 08:09:45.343729751 +0000
+++ .claude/main-agent.md	2026-10-23 08:37:37.194916800 +0000
@@ -49,11 +49,11 @@
 
 **验证段**
 
-1. 主 agent 跑 `bash scripts/gate/merge-and-verify.sh [--skip-apply] [--memory 上限] [--threads N] <补丁目录…>`，打过的与没打过的补丁分两条命令、依次跑：先跑写段打过补丁的那一批，带 `--skip-apply`（只编、只验，脚本先逐份核补丁已打在树上，核不过退 1）；再跑没打过的那一批，不带（脚本来打）；每条命令只给自己那一批的补丁目录（测试挑包与重编核对都按给的目录算），某一批为空就不跑那一条，两条都绿才算这一步完成；两条命令开头都带 `SINGLEFS_HEAVY_TESTS=user-request`：脚本在编译与测试之后另跑 checker 档快档里单元键对不上的测试函数（键与复用照门禁 `checker-tier-crash-point-replay` 快档那一套现读，补丁没碰 checker 档包与它依赖的包时报本次未跑），没带前缀这一步拒、整条退非 0；这一步红了，红的测试函数并进写段的下一批；起验证的命令（这一条、轻型三条、`cargo test` 点名的包与目标、`scripts/gate/gate-staged.sh`）开头都带范围声明 `SINGLEFS_RUN_SCOPE`，写法照 `.claude/hooks/run-scope-guard.sh` 文件头。
+1. 主 agent 跑 `SINGLEFS_RUN_SCOPE='种类=whole; 跑什么=合入后验证; 范围=整轮; 为什么=<这一批补丁碰了哪些包>' SINGLEFS_HEAVY_TESTS=user-request bash scripts/gate/merge-and-verify.sh [--skip-apply] [--memory 上限] [--threads N] <补丁目录…>`，打过的与没打过的补丁分两条命令、依次跑：先跑写段打过补丁的那一批，带 `--skip-apply`（只编、只验，脚本先逐份核补丁已打在树上，核不过退 1）；再跑没打过的那一批，不带（脚本来打）；每条命令只给自己那一批的补丁目录（测试挑包与重编核对都按给的目录算），某一批为空就不跑那一条，两条都绿才算这一步完成；两条命令开头都带 `SINGLEFS_HEAVY_TESTS=user-request`：脚本在编译与测试之后另跑 checker 档快档里单元键对不上的测试函数（键与复用照门禁 `checker-tier-crash-point-replay` 快档那一套现读，补丁没碰 checker 档包与它依赖的包时报本次未跑），没带前缀这一步拒、整条退非 0；这一步红了，红的测试函数并进写段的下一批；这一步红了、本次未跑（`--peer`、`--copy`、显存不够）或没带前缀时，脚本在「→ 怎么办：」那一行里打出只重跑快档那几格的整条命令（开头已带 `SINGLEFS_HEAVY_TESTS=user-request` 与范围声明，线程与内存包装已拼好），主 agent 照原样带前缀跑、不弹窗，不改写、不另拼前缀；起验证的命令（这一条、轻型三条、`cargo test` 点名的包与目标、`scripts/gate/gate-staged.sh`）开头都带范围声明 `SINGLEFS_RUN_SCOPE`，写法照 `.claude/hooks/run-scope-guard.sh` 文件头。
 2. 接着跑轻型三条的全部（核心部分加这一段全部补丁改到的文件点名的项），以及各份实现员报告「要主 agent 跑的命令」里的轻型行。
 3. 整段的合入里有产品代码改动的，代码轮三方开一轮，快照取在全部合入之后，攻整段的合入；整段只改测试代码的不开代码轮（只改注释的产品代码文件开不开、射程与例外都照 `.claude/rules/implementation-workflow.md`「三步，缺一步就不算做完」）。
-4. 这一段红的、判决打中的，合成一批派 `implementation-writer` 一次改全；改完回到验证段第 1 步整体再验一次。不逐条改、逐条验。
-5. 验证段绿了（红的都改完、整体再验一次绿），实验装置与实验跑片接着自动开始，不等用户点头。
+4. 这一段红的、判决打中的，合成一批派 `implementation-writer` 一次改全；改完照验证段第 1 步同一套再验，先只重跑上次红的与这一次改到的：编译、测试红的照 `python3 scripts/gate/rerun-failed-tests.py <上次的 test.log> -p <包>` 打出的命令跑（声明在前，内存包装插在声明与 `cargo` 之间），checker 档快档红的照第 1 步那条脚本打出的只重跑那几格的命令跑，这一次改到的照第 1 步那条命令只给这一批修补的补丁目录跑；整段的整轮重跑留到提交前。不逐条改、逐条验。
+5. 验证段绿了（红的都改完、照第 4 步重跑的都绿），实验装置与实验跑片接着自动开始，不等用户点头。
 
 ## 派出去之后
 
```
