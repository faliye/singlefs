# 第十二批任务 38：交回闸与共用约束（2026-10-23）

依据：`records/2026-09-16-subagent拆分提案.md` 第九十节 ①、㉕（交回闸那一半）、㉞（交回闸四件）、㉟（三件）；`.claude/gate.d/break-switch-outside-selftest-pending.tsv` 里交回闸那一行。草稿目录 `/home/fy5090/.local/share/singlefs/work/landing-audit-b12-handback`（下文写 `<草稿>`）。

## 结论

- 五条都做到了。交回闸经 `scripts/gate/hook-swap.py` 换上两次（第一次 40 处、第二次 11 处，都是草稿里验过、换上之后在主工作区再验过，退 0）。主工作区自证退 0，查了 117 种（改前 78 种）。
- 文件头开关表 45 条开关（交回闸自己 42 条，另有同目录库的 3 条）逐条打开跑 `--selftest`：每一条点名的格与实际红的格逐格相同（`<草稿>/exactness-2.txt`，「开关 45 条，不同的 0 条」）。改前这样比有 6 条对不上（`<草稿>/exactness-baseline.txt`），这次一起修了。
- 平常跑设了 `HANDBACK_GUARD_BREAK` 退 2，并点名它只在自证里认；`break-switch-outside-selftest-pending.tsv` 里那一行删了。
- 共用约束第 63、99 行改成了同一种落盘写法 `<命令> > <草稿目录>/<名>.txt 2>&1; echo rc=$?; tail -n 20 <草稿目录>/<名>.txt`。交回闸拒绝说明照抄这一句（常量 `NAMING_FILED_OUTPUT_FORM`，`.claude/hooks/handback-guard.sh:441`）。用的是 `scripts/kb/replace-once.py`，每处命中 1 次；别的句子没动。
- 真会话记录：3195 份交回。新放 0 份，新拒 165 份，逐份分开看，都不是「这一刻会误拒」：
  - 105 份是 ⑦。全在执行员第 3c 步落地之前（执行员会话记录里头一次出现 `landing-checklist` 的是 2026-10-22 的 `a427eccd6824832fb`；以它开工那一刻为界，时刻是 summarize.py 从记录里现取的，报告里不写钟点），那时还没有这条规矩。落地之后的执行员 8 份，⑦ 新拒 0 份。
  - 59 份是 ② 只写了相对路径的报告，此刻找不到。58 份在 git 历史里，是交回之后才被归档删掉的；剩下 1 份是那条腿自己 `cat >` 写的，后来没进仓。交回那一刻这些文件都在，新判法会照旧按项目根去核。
  - 1 份是 ⑤，Bash 里 `cp` 进主工作区的 `.rs`，之后确实没跑命名检查。
- 什么现象能推翻：
  - 第 3c 步落地之后，有执行员写了装置、交回里确实带了清单，却被 ⑦ 拒了。
  - 有执行员只把装置挪进主工作区（cp / mv），或经 python / sed 改装置，没交清单，⑦ 却放行了。这是已知的口子，见「没做什么」。
  - 有三方腿在别的 worktree 里干活，交回写的是相对路径，项目根下有同名旧报告而那棵树里没有，于是核成了项目根那一份。

## 改过的文件

| 文件 | 改了什么 |
|---|---|
| `.claude/hooks/handback-guard.sh` | ② 认 `$SINGLEFS_WORK_ROOT/` 起头的写法；相对路径按干活的那棵 worktree 解，都找不到就拒。⑤ Bash 写 `.rs` 也算改动；根写成 `$(git rev-parse --show-toplevel)` 照认；认 `${PIPESTATUS[k]}`；认先存 `rc=$?`、之后再 echo 的写法；读 persisted-output 存下的那一份；拒绝说明分开写两种情况。新加 ⑦ 执行员登记落地清单。平常跑设了开关就退 2。新加 39 格自证、14 条开关，文件头开关表逐条重写成实际红的格 |
| `.claude/agent-common.md` | 只改第 63、99 行，各一处定点替换（`replace-once.py`） |
| `.claude/gate.d/break-switch-outside-selftest-pending.tsv` | 删 `.claude/hooks/handback-guard.sh` 那一行 |

`git diff --stat -- <这三份>` 原样（对的是暂存区，里面有别的会话没暂存的改动）：

```
 .claude/agent-common.md                            |    8 +-
 .../break-switch-outside-selftest-pending.tsv      |    3 -
 .claude/hooks/handback-guard.sh                    | 1047 ++++++++++++++++++--
 3 files changed, 955 insertions(+), 103 deletions(-)
```

我自己的改动（对开工时存的原样 `<草稿>/baseline/`，`git diff --no-index --stat`，逐份）：

```
 1 file changed, 951 insertions(+), 96 deletions(-)     handback-guard.sh
 1 file changed, 2 insertions(+), 2 deletions(-)        agent-common.md
 1 file changed, 1 deletion(-)                          break-switch-outside-selftest-pending.tsv
```

换上之后的 sha256：`9aa1ae9e6bb72a98f47bf5d8ec985b59fa267c1f17e32946bb3b79ff03877786  .claude/hooks/handback-guard.sh`、`a0c614dcd93977b8e6a73ff3e1224bb7d66c766052ce108a0f835ca8084187be  .claude/agent-common.md`、`dad4ced7b35c4f1d9f2655e816d778cc209458723058ca9c45df4fd0a24e992d  .claude/gate.d/break-switch-outside-selftest-pending.tsv`。

共用约束改后的两句原样：

- 第 63 行那一句：长命令的输出落进草稿目录的文件，只读汇总行，写成 `<命令> > <草稿目录>/<名>.txt 2>&1; echo rc=$?; tail -n 20 <草稿目录>/<名>.txt`（`echo rc=$?` 紧跟在命令后面，汇总行也可以用 `grep -c`、`grep ✗` 读；接管道（`| tail`、`| head`）、`|| true` 或中间夹别的命令的，交回闸认不出命令自己的退出码），不把整份日志、整份 diff 读进上下文。
- 第 99 行那一句：输出不重定向到 `.log`（`.claude/hooks/two-logs-guard.sh` 拒）；要落盘照「不做」一节读文件那一条的写法 `<命令> > <草稿目录>/<名>.txt 2>&1; echo rc=$?; tail -n 20 <草稿目录>/<名>.txt`，或经 `bash scripts/hosts/two-logs.sh <名> -- <命令>`；接管道（`| tail`、`| head`、`| grep`）、`|| true`、后面用 `;` 接别的命令而没紧跟 `echo rc=$?` 的，交回闸认不出它自己的退出码，不算跑过。跑了的，两条的命令与末尾原样输出贴进报告。

新函数（行号是换上之后 `grep -n` 现取的）：`work_root_tokens_expanded` :518、`agent_working_trees` :531、`leg_reports_named` :551、`citation_problems` :594、`persisted_output` :741、`bash_rs_writes` :753、`git_toplevel` :809、`status_echo_pattern` :1072、`saved_status_assignment` :1105、`status_target` :1112、`status_echo_readings` :1123、`naming_unrecognized_reason` :1258、`naming_problems` :1269、`naming_refusal` :1316、`sections_titled` :1449、`table_data_rows` :1465、`landing_file_index` :1482、`landing_applies` :1494、`landing_problems` :1524、`landing_refusal` :1619、`selftest_child` :2240、`main` :2249。

## 逐条：样本（自证格）红绿、弄坏开关

样本都是自证里的格，跑在主工作区：`bash .claude/hooks/handback-guard.sh --selftest`。它在 `.claude/hooks/selftest-registry.tsv` 里登记为 light。换上之后退 0，输出末行「✓ handback-guard 自检通过（查了 117 种）…」，全文在 `<草稿>/selftest-2.txt`。每条开关逐个打开之后红在哪几格，见下面「弄坏开关逐条复跑」那一节。格名照抄自证 cases 里的标签。

### 1. ① 执行员的登记落地清单（新加的交回闸 ⑦）

- 判法：`landing_problems`（:1524）。哪些执行员要判：这一段写了装置，或往 `research/results/` 落了产物（`landing_applies` :1494）；派发提示写着「只修锚点」或「只补落产物」的不判。要判的，交回正文、交回点名的报告与派发提示「报告：」那一份里，要有一节标题含「登记落地清单」，节里的表至少一行。表里每个「文件:行」要找得到那份文件、它有这一行；每行最后一格的用例名要 grep 得到。找的地方：项目根、草稿目录、报告所在目录、会话记录里改过的文件，再加主工作区 `crates/`、`scripts/`。
- 红样本：「执行员报告里没有登记落地清单一节拒」「执行员登记落地清单的文件:行超出那份文件的行数拒」「执行员登记落地清单的文件在副本与主工作区里都找不到拒」「执行员登记落地清单的用例名 grep 不到拒」「执行员登记落地清单一节里一行都没有拒」「执行员落了产物、没有登记落地清单一节拒」，都是应得 2，实得 2。
- 绿样本：「执行员登记落地清单的文件:行与用例名都找得到放行」「执行员登记落地清单写在交回正文里也认」「执行员只补落产物的派发不判登记落地清单」「执行员这一段没写装置、没落产物（只写实验页）不判登记落地清单」「回扫员没有登记落地清单不判」，都是应得 0，实得 0。拒绝说明那一格是「执行员没有登记落地清单的拒绝说明写出那一节与出路」。
- 弄坏开关：`no-landing-checklist`、`landing-exemption-ignored`、`landing-applies-always`、`landing-location-unchecked`、`landing-case-unchecked`。
- 判法改过一次（「新拒不是真没跑的就改判法」）：头一版对每个执行员都判。拿真记录量，第 3c 步落地前后加起来新拒 210 份；落地当天（10-22）与之后有 4 份，是只写实验页、只把装置挪进主工作区、只比读数、主 agent 跑完产物后写页的执行员（`afb958c0ba47124cb`、`a95c614a0e12e103f`、`a4356d8e8c64934fe`、`a9dbfee4c29c6da98`），它们按第 3c 步本来就不用交清单。所以改成只判「这一段写了装置或落了产物」的。改后，落地之后的 8 份执行员里判了 4 份（`a427eccd6824832fb`、`ac532acba01448647`、`ad81f57ff2589c987`、`a4dcb307d1deb60a2`），一共核了 8 处文件:行、100 个用例名，全找得到（`<草稿>/landing-probe-final.txt`）。

### 2. ㉕ 交回闸那一半

- Bash 里写 `.rs`（`bash_rs_writes` :753）认这几种：输出重定向 `>`、`>>`、`>|`、`&>` 的目标；`tee` 的文件参数；`cp` / `mv` 的目标，目标是 `.rs`，或者目标是目录而来源是 `.rs`（`-t` 照认）。目标里的变量照这一段命令里赋过的值展开。
  - 红样本：「Bash 里 mv 写进 crates/ 的 .rs、没跑命名检查拒」「Bash 里 cp 到 crates/ 下的目录、来源是 .rs，没跑命名检查拒」「Bash 里 > 重定向写 .rs、没跑命名检查拒」「Bash 里 >> 追加写 .rs（目标在变量里）、没跑命名检查拒」「Bash 里 tee 写 .rs、没跑命名检查拒」，应得 2、实得 2。
  - 绿样本：「Bash 里 mv 写 .rs 之后两条命名检查都跑了放行」「Bash 里把 .rs 拷到草稿目录不算改动放行」，应得 0、实得 0。
  - 弄坏开关 `naming-bash-writes-ignored`：上面五条红样本都红。
- 根写成 `$(git rev-parse --show-toplevel)` 的，从命令当时的目录往上找，第一层带 `.git` 的目录当根（`git_toplevel` :809）。
  - 绿样本：「根写成 $(git rev-parse --show-toplevel)、cd 进 worktree 副本里跑放行」。
  - 红样本：「根写成 $(git rev-parse --show-toplevel)、cd 到别的树里跑的不算在改动所在的那一棵上拒」。
  - 弄坏开关 `naming-git-toplevel-unknown`：绿样本那一格红。

### 3. ㉞ 交回闸四件

- (a) 落盘写法。`<命令> > 文件 2>&1; echo rc=$?; tail …` 改前就认，有格「timeout 230 包着、输出落文件、; echo rc=$? 打出 rc=1（判红跑完）放行」。这次新认两种：一种是先存 `rc=$?`、读完汇总行再 echo 出来；另一种是 `echo rc=$?` 追加进落盘的文件、同一条里用 tail 读出来（`status_echo_readings` :1123）。共用约束第 63、99 行与拒绝说明写的是同一句 `NAMING_FILED_OUTPUT_FORM`。
  - 绿样本：「输出落文件、rc=$? 存下、读完汇总行再 echo $rc 放行」「echo rc=$? 追加进落盘的那份文件、同一条里 tail 读出来放行」。
  - 红样本：「rc=$? 存下之后又被改写再 echo，不算跑过」。
  - 弄坏开关 `naming-saved-status-unread`：存下再 echo 的那一格红。
- (b) 拒绝说明分两种写：改完之后一次都没起过的写「会话记录里改完之后没跑过」；起过而认不出跑完的，写「跑了，但认不出它自己的退出码：<为什么>」（`naming_refusal` :1316、`naming_unrecognized_reason` :1258）。
  - 拒绝说明的格：「跑了而接 | tail 的拒绝说明写跑了但认不出退出码、为什么、照抄的写法」「没跑的拒绝说明写没跑过、不写认不出退出码」「echo rc=$? 打出 124 的拒绝说明写被 timeout 杀掉」「存成 persisted-output 读不到的拒绝说明写 persisted-output」。
  - 弄坏开关 `naming-refusal-reasons-merged`：会红其中三格。「没跑的…不写认不出」那一格在这条开关下照样绿，它钉的是另一半。
- (c) persisted-output。工具结果只留了预览的，照预览里写的路径读存下来的整份输出，再找 `rc=` 那一行（`tool_calls`、`persisted_output` :741）。
  - 样本里造了两份带 persisted-output 的会话记录：绿样本「退 0、输出存成 persisted-output、rc= 那一行在存下来的整份里放行」；红样本「存成 persisted-output 而存下来的那一份不在，认不出退出码拒」。
  - 弄坏开关 `naming-persisted-unread`：绿样本那一格红。
- (d) 报告路径写成 `$SINGLEFS_WORK_ROOT/` 开头的：② 先照 `lib_work_root.py` 的 `work_root()` 展开，再交给 `lib_three_way_legs.py` 去认（`work_root_tokens_expanded` :518）。那份库我没动。③、⑥ 和新的 ⑦ 改前就经 `expand_written_path` 展开，这次补了一格把它钉住。
  - 绿样本：「三方腿报告路径:$SINGLEFS_WORK_ROOT/ 起头的 worktree 报告照 lib_work_root.py 展开认、引文对上放行」。
  - 红样本：「…引文对不上按引文拒（不按没点名报告拒）」，以及「执行员工作根下的报告里绿行没数（$SINGLEFS_WORK_ROOT 写法）」。
  - 弄坏开关 `leg-report-work-root-token-kept`：② 那两格红。

### 4. ㉟ 三件

- (a) 命名检查用 `PIPESTATUS` 取退出码的：`; echo <标签>${PIPESTATUS[k]}` 打出的数，算作紧挨着那条管道第 k 段自己的退出码（`status_target` :1112）。
  - 绿样本：「两条接 | tail、后面 ; echo rc=${PIPESTATUS[0]} 打出它自己的退出码放行」。
  - 红样本：「两条接 | tail、echo ${PIPESTATUS[1]} 打的是 tail 的，不算跑过」。
  - 弄坏开关 `naming-pipestatus-unread`：绿样本那一格红。
  - 真记录 `a8519aab2a61cba49` 改前改后都没翻：那一份里 `${PIPESTATUS[0]}` 跟在门禁 94 那一条后面，命名检查 naming-lint 接的是 `| tail -5; echo exit=$?`，打出的是 tail 的退出码；改的那棵树又是 `/tmp/claude-1000/impl-kv-stage1/work`，没有 `.git`，按 rsync 副本不判。
- (b) 别的 worktree 里只写相对路径的报告：这个子 agent 在别的登记过的工作树里干过活（会话记录里工具调用的参数与 cwd 点名到那一棵，`agent_working_trees` :531），那一棵里有这份报告的，按那一棵当根核引文；没有、项目根下有的，按项目根核；哪一棵都没有，拒，拒绝说明头一行是「✗ 三方腿交回点名的报告找不到」。
  - 绿样本：「三方腿报告路径:别的 worktree 里干活、相对路径写的报告按那一棵核、引文对上放行」。
  - 红样本：「…引文对不上拒」与「三方腿报告路径:相对路径写的报告在干活的那一棵与项目根下都找不到拒、拒绝说明写明找不到」。
  - 弄坏开关 `leg-report-relative-root-only`：两条红样本都红。退回改前那样按项目根解，文件不在时 cite-check 退 2，交回闸就放行了。
- (c) 文件头开关表：45 条开关逐条打开跑自证，点名的格与实际红的格逐格相同，见下一节。`no-cite` 补上了新格，`limit-hint-unnamed` 写的是全名，`no-limit`、`cite-any-agent`、`no-naming-check`、`leg-types-hardcoded` 这几条改前就点名不全，一起补齐了。

### 5. 弄坏开关只在自证里认（`break-switch-outside-selftest-pending.tsv` 那一行）

- `main`（:2249）最先判，在读 hook 的 JSON 之前：设了 `HANDBACK_GUARD_BREAK`，这一次又不是 `--selftest`、也不是自证起的子进程，就退 2，stderr 点名变量、写明它只在自证里认。自证起的子进程带 `HANDBACK_GUARD_SELFTEST_MARK`，指向一份内容是 `SELFTEST_MARK_CONTENT` 的标记文件（`selftest_child` :2240，读完就从环境里摘掉）。样子照 `scripts/mutation/mutate.sh` 的 `MUTATE_SELFTEST_MARK`。
- 门禁那一格的探针怎么起，我就照那样在主工作区起了一次，原样：

```
$ env HANDBACK_GUARD_BREAK=no-limit bash .claude/hooks/handback-guard.sh --force --break-guard-probe-unknown-option < /dev/null; echo "rc=$?"
  ✗ handback-guard：HANDBACK_GUARD_BREAK=no-limit 设着，这个弄坏开关只在自证里认（--selftest 与它起的子进程），平常跑不认：这一次没判，退 2
    → 怎么办：unset HANDBACK_GUARD_BREAK 再来；要看一条开关弄坏了什么，在 bash .claude/hooks/handback-guard.sh --selftest 前面设它
rc=2
$ echo '{"tool_name":"SubagentHandback","tool_input":{"message":"x"},"agent_type":"sweep"}' | bash .claude/hooks/handback-guard.sh; echo "rc=$?"
rc=0
```

- 自证格「平常跑设了 HANDBACK_GUARD_BREAK 退 2、点名它只在自证里认」应得 (2, True)。弄坏开关 `honor-break-outside-selftest` 打开时这一格红，实得 (0, False)。原来那格「走真实 stdin 入口：超长交回」的子进程带着标记，照旧认调用方设的开关，所以 `no-limit` 下那一格照旧红。
- 文件头写的「代码里读的 *_BREAK」只有 `HANDBACK_GUARD_BREAK` 一个，门禁那一格按文件头开关表第一条的值 `no-limit` 起探针。
- 待改表那一行删了：`diff <草稿>/baseline/break-switch-outside-selftest-pending.tsv .claude/gate.d/break-switch-outside-selftest-pending.tsv` 原样是

```
7d6
< .claude/hooks/handback-guard.sh	# 还没改成平常跑设了开关就拒、点名只在自证里认
```

## 弄坏开关逐条复跑（`<草稿>/tools/switch_exactness.py`，读数 `<草稿>/exactness-2.txt`）

做法：对文件头开关表里的每一条，先把环境里所有名字以 `_BREAK` 结尾的变量摘掉，只设这一条，跑主工作区那一份的 `--selftest`。再把失败行「✗ 自检：<标签> 应当返回 …」里的标签，与文件头那一条下面点名的标签逐个比，多的、少的都算不同。换上之后原样：

```
HANDBACK_GUARD_BREAK=no-limit：退 1，点名 3 格、红 3 格，逐格相同
HANDBACK_GUARD_BREAK=limit-hint-unnamed：退 1，点名 1 格、红 1 格，逐格相同
HANDBACK_GUARD_BREAK=no-cite：退 1，点名 12 格、红 12 格，逐格相同
HANDBACK_GUARD_BREAK=cite-any-agent：退 1，点名 39 格、红 39 格，逐格相同
HANDBACK_GUARD_BREAK=no-green-count：退 1，点名 5 格、红 5 格，逐格相同
HANDBACK_GUARD_BREAK=zero-count-ok：退 1，点名 1 格、红 1 格，逐格相同
HANDBACK_GUARD_BREAK=judge-general-purpose：退 1，点名 1 格、红 1 格，逐格相同
HANDBACK_GUARD_BREAK=no-background：退 1，点名 3 格、红 3 格，逐格相同
HANDBACK_GUARD_BREAK=no-naming-check：退 1，点名 35 格、红 35 格，逐格相同
HANDBACK_GUARD_BREAK=naming-root-unmatched：退 1，点名 15 格、红 15 格，逐格相同
HANDBACK_GUARD_BREAK=naming-any-file：退 1，点名 1 格、红 1 格，逐格相同
HANDBACK_GUARD_BREAK=naming-copy-judged：退 1，点名 5 格、红 5 格，逐格相同
HANDBACK_GUARD_BREAK=naming-refused-counts：退 1，点名 1 格、红 1 格，逐格相同
HANDBACK_GUARD_BREAK=naming-position-unchecked：退 1，点名 19 格、红 19 格，逐格相同
HANDBACK_GUARD_BREAK=naming-exit-code-unchecked：退 1，点名 6 格、红 6 格，逐格相同
HANDBACK_GUARD_BREAK=naming-tool-exit-trusted：退 1，点名 13 格、红 13 格，逐格相同
HANDBACK_GUARD_BREAK=naming-echo-unread：退 1，点名 7 格、红 7 格，逐格相同
HANDBACK_GUARD_BREAK=naming-timeout-refused：退 1，点名 3 格、红 3 格，逐格相同
HANDBACK_GUARD_BREAK=naming-bash-writes-ignored：退 1，点名 5 格、红 5 格，逐格相同
HANDBACK_GUARD_BREAK=naming-git-toplevel-unknown：退 1，点名 1 格、红 1 格，逐格相同
HANDBACK_GUARD_BREAK=naming-pipestatus-unread：退 1，点名 1 格、红 1 格，逐格相同
HANDBACK_GUARD_BREAK=naming-saved-status-unread：退 1，点名 1 格、红 1 格，逐格相同
HANDBACK_GUARD_BREAK=naming-persisted-unread：退 1，点名 1 格、红 1 格，逐格相同
HANDBACK_GUARD_BREAK=naming-refusal-reasons-merged：退 1，点名 3 格、红 3 格，逐格相同
HANDBACK_GUARD_BREAK=leg-report-relative-root-only：退 1，点名 2 格、红 2 格，逐格相同
HANDBACK_GUARD_BREAK=leg-report-work-root-token-kept：退 1，点名 2 格、红 2 格，逐格相同
HANDBACK_GUARD_BREAK=no-landing-checklist：退 1，点名 7 格、红 7 格，逐格相同
HANDBACK_GUARD_BREAK=landing-exemption-ignored：退 1，点名 1 格、红 1 格，逐格相同
HANDBACK_GUARD_BREAK=landing-applies-always：退 1，点名 1 格、红 1 格，逐格相同
HANDBACK_GUARD_BREAK=landing-location-unchecked：退 1，点名 2 格、红 2 格，逐格相同
HANDBACK_GUARD_BREAK=landing-case-unchecked：退 1，点名 1 格、红 1 格，逐格相同
HANDBACK_GUARD_BREAK=honor-break-outside-selftest：退 1，点名 1 格、红 1 格，逐格相同
HANDBACK_GUARD_BREAK=leg-report-subdirectory-kept：退 1，点名 10 格、红 10 格，逐格相同
HANDBACK_GUARD_BREAK=leg-report-outside-root-kept：退 1，点名 8 格、红 8 格，逐格相同
HANDBACK_GUARD_BREAK=leg-report-worktrees-unread：退 1，点名 4 格、红 4 格，逐格相同
HANDBACK_GUARD_BREAK=leg-report-git-directory-trusted：退 1，点名 1 格、红 1 格，逐格相同
HANDBACK_GUARD_BREAK=leg-report-citation-root-main：退 1，点名 6 格、红 6 格，逐格相同
HANDBACK_GUARD_BREAK=leg-report-worktree-error-unnamed：退 1，点名 1 格、红 1 格，逐格相同
HANDBACK_GUARD_BREAK=no-scribe-targets：退 1，点名 5 格、红 5 格，逐格相同
HANDBACK_GUARD_BREAK=scribe-section-anywhere：退 1，点名 1 格、红 1 格，逐格相同
HANDBACK_GUARD_BREAK=scribe-section-unread：退 1，点名 1 格、红 1 格，逐格相同
HANDBACK_GUARD_BREAK=leg-types-hardcoded：退 1，点名 4 格、红 4 格，逐格相同
SINGLEFS_WORK_ROOT_BREAK=new-root-unknown：退 1，点名 8 格、红 8 格，逐格相同
LIB_PROMPT_SCRATCH_LINE_BREAK=label-anywhere：退 1，点名 1 格、红 1 格，逐格相同
SINGLEFS_WORK_ROOT_BREAK=legacy-root-unknown：退 1，点名 1 格、红 1 格，逐格相同
开关 45 条，不同的 0 条
```

改前这样比的读数（`<草稿>/exactness-baseline.txt`）末行是「开关 31 条，不同的 6 条」，这 6 条是 `no-limit`、`limit-hint-unnamed`、`no-cite`、`cite-any-agent`、`no-naming-check`、`leg-types-hardcoded`。

这一批新加的开关，与 `no-cite`、`limit-hint-unnamed` 在文件头点名的格，原样如下（文件头里每行前面还有 `#`）：

```
   HANDBACK_GUARD_BREAK=limit-hint-unnamed  ① 超长的拒绝说明退回改前那一句（只写「没给就写进草稿目录」，不写 result.md 与三方腿的文件名）
       超长交回的拒绝说明写明报告文件名（没给报告路径写草稿目录的 result.md，三方腿写 research/prompts/<轮>-*-output.md）（「应当返回 True，实际 False」）
   HANDBACK_GUARD_BREAK=no-cite  decide() 不判 ② 三方腿与核查员的引文
       攻方腿引文行号错（「应当返回 2，实际 0」）
       核查员没点名报告（「应当返回 2，实际 0」）
       三方腿报告路径:只点名子目录下的 -output.md 不认，按没点名报告拒（「应当返回 True，实际 False」）
       三方腿报告路径:只点名没填的 <轮> 占位不认，按没点名报告拒（「应当返回 True，实际 False」）
       三方腿报告路径:只点名仓外同名的 …/research/prompts/r1-opus-output.md 不认，按没点名报告拒（「应当返回 True，实际 False」）
       三方腿报告路径:登记过的 worktree 里引文对不上拒（「应当返回 2，实际 0」）
       三方腿报告路径:没登记成 worktree 的仓外副本（带 .git）里的不认，按没点名报告拒（「应当返回 True，实际 False」）
       三方腿报告路径:别的 worktree 里干活、相对路径写的报告按那一棵核、引文对不上拒（「应当返回 2，实际 0」）
       三方腿报告路径:相对路径写的报告在干活的那一棵与项目根下都找不到拒、拒绝说明写明找不到（「应当返回 (2, True)，实际 (0, False)」）
       三方腿报告路径:$SINGLEFS_WORK_ROOT/ 起头的 worktree 报告引文对不上按引文拒（不按没点名报告拒）（「应当返回 (2, False)，实际 (0, False)」）
       三方腿报告路径:取不到 worktree 列表时只认项目根、拒绝说明写明（「应当返回 True，实际 False」）
       三方腿类型表:样本仓登记的 sample-leg 没点名报告拒（「应当返回 2，实际 0」）
   HANDBACK_GUARD_BREAK=naming-bash-writes-ignored  bash_rs_writes() 一个都不认（改前：Bash 里 mv、cp、重定向、tee 写的 .rs 不算改动）
       Bash 里 mv 写进 crates/ 的 .rs、没跑命名检查拒（「应当返回 2，实际 0」）
       Bash 里 cp 到 crates/ 下的目录、来源是 .rs，没跑命名检查拒（「应当返回 2，实际 0」）
       Bash 里 > 重定向写 .rs、没跑命名检查拒（「应当返回 2，实际 0」）
       Bash 里 >> 追加写 .rs（目标在变量里）、没跑命名检查拒（「应当返回 2，实际 0」）
       Bash 里 tee 写 .rs、没跑命名检查拒（「应当返回 2，实际 0」）
   HANDBACK_GUARD_BREAK=naming-git-toplevel-unknown  naming_check_occurrences() 不认根写成 $(git rev-parse --show-toplevel) 的（改前：根认不出，不算跑过）
       根写成 $(git rev-parse --show-toplevel)、cd 进 worktree 副本里跑放行（「应当返回 0，实际 2」）
   HANDBACK_GUARD_BREAK=naming-pipestatus-unread  status_echo_pattern() 不认 ${PIPESTATUS[k]}（当任意字）
       两条接 | tail、后面 ; echo rc=${PIPESTATUS[0]} 打出它自己的退出码放行（「应当返回 0，实际 2」）
   HANDBACK_GUARD_BREAK=naming-saved-status-unread  status_echo_readings() 不认「名字=$?」存下、之后 echo $名字 打出的数
       输出落文件、rc=$? 存下、读完汇总行再 echo $rc 放行（「应当返回 0，实际 2」）
   HANDBACK_GUARD_BREAK=naming-persisted-unread  tool_calls() 不读 persisted-output 存下来的那一份（只按预览认，改前的读法）
       退 0、输出存成 persisted-output、rc= 那一行在存下来的整份里放行（「应当返回 0，实际 2」）
   HANDBACK_GUARD_BREAK=naming-refusal-reasons-merged  naming_problems() 不记跑了却没认出的原因（拒绝说明退回一律写没跑过）
       跑了而接 | tail 的拒绝说明写跑了但认不出退出码、为什么、照抄的写法（「应当返回 True，实际 False」）
       echo rc=$? 打出 124 的拒绝说明写被 timeout 杀掉（「应当返回 True，实际 False」）
       存成 persisted-output 读不到的拒绝说明写 persisted-output（「应当返回 True，实际 False」）
   HANDBACK_GUARD_BREAK=leg-report-relative-root-only  leg_reports_named() 只写相对路径的报告一律按项目根（改前：干活的那棵 worktree 不看，找不到也不拒）
       三方腿报告路径:别的 worktree 里干活、相对路径写的报告按那一棵核、引文对不上拒（「应当返回 2，实际 0」）
       三方腿报告路径:相对路径写的报告在干活的那一棵与项目根下都找不到拒、拒绝说明写明找不到（「应当返回 (2, True)，实际 (0, False)」）
   HANDBACK_GUARD_BREAK=leg-report-work-root-token-kept  work_root_tokens_expanded() 不展开 $SINGLEFS_WORK_ROOT/（改前：② 认不出这种写法）
       三方腿报告路径:$SINGLEFS_WORK_ROOT/ 起头的 worktree 报告照 lib_work_root.py 展开认、引文对上放行（「应当返回 (0, False)，实际 (2, True)」）
       三方腿报告路径:$SINGLEFS_WORK_ROOT/ 起头的 worktree 报告引文对不上按引文拒（不按没点名报告拒）（「应当返回 (2, False)，实际 (2, True)」）
   HANDBACK_GUARD_BREAK=no-landing-checklist  decide() 不判 ⑦ 执行员的登记落地清单
       执行员报告里没有登记落地清单一节拒（「应当返回 2，实际 0」）
       执行员登记落地清单的文件:行超出那份文件的行数拒（「应当返回 2，实际 0」）
       执行员登记落地清单的文件在副本与主工作区里都找不到拒（「应当返回 2，实际 0」）
       执行员登记落地清单的用例名 grep 不到拒（「应当返回 2，实际 0」）
       执行员登记落地清单一节里一行都没有拒（「应当返回 2，实际 0」）
       执行员落了产物、没有登记落地清单一节拒（「应当返回 2，实际 0」）
       执行员没有登记落地清单的拒绝说明写出那一节与出路（「应当返回 True，实际 False」）
   HANDBACK_GUARD_BREAK=landing-exemption-ignored  landing_problems() 只修锚点、只补落产物的派发也判
       执行员只补落产物的派发不判登记落地清单（「应当返回 0，实际 2」）
   HANDBACK_GUARD_BREAK=landing-applies-always  landing_problems() 没写装置、没落产物的执行员也判
       执行员这一段没写装置、没落产物（只写实验页）不判登记落地清单（「应当返回 0，实际 2」）
   HANDBACK_GUARD_BREAK=landing-location-unchecked  landing_problems() 不核清单里的文件:行
       执行员登记落地清单的文件:行超出那份文件的行数拒（「应当返回 2，实际 0」）
       执行员登记落地清单的文件在副本与主工作区里都找不到拒（「应当返回 2，实际 0」）
   HANDBACK_GUARD_BREAK=landing-case-unchecked  landing_problems() 不 grep 清单里的用例名
       执行员登记落地清单的用例名 grep 不到拒（「应当返回 2，实际 0」）
   HANDBACK_GUARD_BREAK=honor-break-outside-selftest  main() 平常跑设了弄坏开关也照认、照常判（改前的样子）
       平常跑设了 HANDBACK_GUARD_BREAK 退 2、点名它只在自证里认（「应当返回 (2, True)，实际 (0, False)」）
```

## 真会话记录上的改前改后（`<草稿>/tools/measure.py`、`summarize.py`，读数 `<草稿>/measure-final.tsv`、`<草稿>/summary-final.txt`）

- 语料：`~/.claude/projects/-home-fy5090-code-singlefs/*/subagents/` 与 m4-predev 那棵树自己的 `~/.claude/projects/-home-fy5090--local-share-singlefs-work-m4-predev/*/subagents/`，共 3436 份会话记录，其中带 SubagentHandback 的 3195 份。
- 做法：每份取最后一次 SubagentHandback 的正文，进程里分别载入改前（`<草稿>/baseline/handback-guard.sh`）和改后（主工作区此刻那一份）的 `decide()`，各喂一遍，比退出码。项目根取主工作区；m4-predev 那个目录已经不在了，它那 29 份也按主工作区判。④ 判的是「此刻有没有进程开着输出文件」，与这一批无关，两版都换成交空表。
- 汇总原样：

```
交回 3195 份；改前/改后： {('2', '2'): 251, ('0', '0'): 2779, ('0', '2'): 165}
执行员会话记录里头一次出现 landing-checklist： 2026-10-22〔原样是带钟点的时间戳，报告里只写日期，换掉了钟点〕

== 新拒 165 份： {'⑦登记落地清单': 105, '②相对路径报告找不到': 59, '⑤命名检查': 1}
  2026-09-26 a5ba4e5f874825019 experiment-runner ⑤命名检查  ｜ ✗ experiment-runner 改了 crates/ 或 scripts/ 下的 .rs（Edit / Write，或 Bash 里 mv、cp、重定向写的），改完之后两条命名检查没都认出跑完（只认会话记录里真跑完的命令、认得出它自己的退出码是 0 或 1，报告里写跑了不算）： ⏎   /home/fy5090/code/singlefs：最后改的是 /home/fy5090/code/singlefs/crates/singlefs-harness/src/bin/
  2026-09-25 a9e5998113e9ba068 three-way-defense ②相对路径报告找不到 git 历史里没有 ｜ ✗ 三方腿交回点名的报告找不到： ⏎   交回里只写相对路径的报告 research/prompts/defs-memory-migration-r2-sonnet-output.md 在你干过活的工作树与项目根（/home/fy5090/code/singlefs）下都找不到：报告还没写进去，或写在了别处——写进去再交回，或交回里写那一份的绝对路径 ⏎ → 怎么办：报告写进你干活的那棵工作树（或项目根）的 research/prompts/，交回写那一份的路径（别的 wor
  （没逐份列的：⑦ 第 3c 步落地之前的 105 份；② 相对路径报告此刻不在、git 历史里有的见上面的计数）

== 新放 0 份： {}
```

- 按会话记录目录分：`-home-fy5090-code-singlefs` 3166 份，0/0 2757、2/2 245、0/2 164；m4-predev 29 份，0/0 22、2/2 6、0/2 1（那一份是 2026-10-13 的执行员 `a38415f4e06793849`，在第 3c 步落地之前）。
- 新拒逐份怎么判：
  - ⑦ 105 份：开工时刻都早于 `a427eccd6824832fb` 头一次写 `landing-checklist` 那一条记录（2026-10-22），那时执行员会话记录里一次都还没出现过 `landing-checklist`，是规矩还没有，不是误拒。落地之后的执行员 8 份，新拒 0 份（`<草稿>/landing-probe-final.txt`）。
  - ② 59 份：交回里只写了相对路径的报告，此刻在项目根下不在。其中 58 份 `git log --all -- <路径>` 查得到，是交回之后被归档删掉的。剩下 1 份 `a9e5998113e9ba068`，报告 `research/prompts/defs-memory-migration-r2-sonnet-output.md` 是它自己在会话里用 `set -o noclobber && cat > …` 写的，之后没进仓。交回那一刻 59 份都在项目根下，新判法会照旧按项目根去核，那一刻判得与改前一样。另用正则在各自的会话记录里找写这份报告的命令（Write 或 `cat >` / `tee`），找到 37 份；另 22 份是我的正则没认出写法，没逐份看。
  - ⑤ 1 份 `a5ba4e5f874825019`（2026-09-26，执行员）：它用 `cp $R/repo/crates/singlefs-harness/src/bin/e158_root_choice_repair.rs crates/...` 把 `.rs` 拷进主工作区，之后没跑门禁那四格；naming-lint 跑的是旧路径 `.claude/scripts/naming-lint.sh`，而且跑在拷进去之前。是真没跑。
- 新放 0 份。PIPESTATUS、先存 `rc=$?` 再 echo、persisted-output 这三种写法，语料里都没碰上「改前拒、改后放」的交回。
- 改前改后都拒 251 份（`<草稿>/both-refused.txt`）：① 123、⑤ 97、⑥ 16、② 引文 15。⑤ 那 97 份改后的拒绝说明里，96 份写「会话记录里改完之后没跑过」。1 份 `aa68d151c2fa72bb7`（工具员）写的是「跑了，但认不出它自己的退出码：后面没有紧跟 ; echo rc=$?…」：它把门禁那四格的输出落进文件、后面接的是别的命令，就是 K2-1 说的那一种。

## 新写的 `# gate-similar:` / `# hook-events:` 行

没有。新判据都追加进了已有的 `.claude/hooks/handback-guard.sh`；开工时跑 `gate-overlap.py --list`，挂 `PreToolUse[SubagentHandback]` 的已有三份：它、`handback-scratch-check.sh`、`completion-cleanup-guard.sh`。没有新钩子，`.claude/settings.json` 没动。

## 第二种活改过的定义与共用约束（交主 agent 开定义三方）

- `.claude/agent-common.md`：第 63 行（「不做」一节，读文件那一条）、第 99 行（「门禁」一节，「命名检查交回前跑」）。走门禁 doc-process-records 的 agent-def-adversarial-review 格。
- 没改的：第 94 行「交回正文闸」那一条还是改前的说法，没写 ⑤ 认 Bash 写的 `.rs`，也没写新的 ⑦ 登记落地清单。派发提示只许改第 63、99 行，所以没动它；要不要改、怎么改，交主 agent 定。

## 要主 agent 跑的命令

工作目录 `/home/fy5090/code/singlefs`。只起 python 与 bash，不用内存包装，也不用设线程数。

1. 门禁 code-tooling 里管弄坏开关与待改表的三格：

```
SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 break-switch-headers、break-switch-replay、break-switch-outside-selftest 格; 范围=3 格; 为什么=第十二批交回闸新加 14 条开关、重写开关表点名的格、平常跑设开关就拒，待改表删了交回闸那一行' bash .claude/gate.d/code-tooling.sh --check break-switch-headers,break-switch-replay,break-switch-outside-selftest /home/fy5090/code/singlefs
```

   看什么：`break-switch-replay` 那一格里 `.claude/hooks/handback-guard.sh` 没有「点名的项没在失败行里」「退 0」「超时」。登记表给交回闸每条开关的上限是 120 秒；单跑一次自证约 1 秒。`break-switch-outside-selftest` 那一格对 `.claude/hooks/handback-guard.sh` 探针是退 2、有点名一行（上面第 5 条贴了原样），待改表少了 1 份。
2. 提交前那一趟（`scripts/gate/gate-staged.sh`）会判到、我没跑的：gate-overlap（交回闸这一次改动是在原文件里追加，没新建文件）、code-tooling 的 research-gate-lint（新的拒绝都带 `→ 怎么办`）。

## 没做什么

- 没跑重型测试（`checker-tier-crash-point-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification、checker-tier-research-build-and-replay 本身、checker-tier-broad-mutation 与 `scripts/mutation/broad-mutation.py` 起 cargo 的用法、`gate.sh`、全量 `cargo test`、`check.sh`），也没编译。门禁阶段与各类 lint 没跑。没改 `.rs`，所以命名检查不用跑。没走定义三方。没提交。
- 草稿里那一份改过的交回闸，没在副本里起过它的 `--selftest`。草稿只经 `hook-swap.py` 喂法验（5 条：回扫员短交回放行、超长照拒、通用 agent 不判、攻方没点名报告照拒、执行员读不到会话记录不判），自证是换上之后在主工作区跑的。真记录那一趟是在我自己的脚本里载入两版 `decide()` 喂的，没有起自证。
- 没动 `.claude/hooks/lib_three_way_legs.py`，兄弟任务 39 在改它。② 认 `$SINGLEFS_WORK_ROOT/` 与认相对路径，都做在交回闸里、包在调那份库的外面：先展开再交给库；库认出的相对路径换一棵树。任务 39 要是在库里也展开了 `$SINGLEFS_WORK_ROOT/`，交回闸这边那一步就是多余的，结果不变。派发闸 ⑰ 照旧认不出 `$SINGLEFS_WORK_ROOT/` 起头的报告路径，这一批没碰派发闸。
- ⑦ 认不出的：
  - 清单行里只写「:行号」不写文件的不核。真报告里多数是这种写法，行号接在函数名后面，文件写在节首；落地之后被判的 4 份里，只有 `ac532acba01448647` 写了 8 处带文件名的「文件:行」。
  - 装置只在 Bash 里经 python / sed 改的、只 cp / mv 挪的，都不算「写了装置」，那一段不判。落地之后的 `a6cf77dfdf516044f`、`af8f9949a760f2c29` 就是这种，它们交了清单，可这一条没判它们。
  - 用例名只认小写、至少三段下划线连着的词，被 `…` 截短的不算。
- ⑤ 认不出的：Bash 里 `cp -r` 整个目录、`install`、`sed -i`、python 写的 `.rs`；同一条命令里，先写 `.rs`、后跑命名检查，与先跑、后写，判法一样，都按「先算写、再算命名检查」。
- 另一种「之后读到退出码」的写法没做：上一条命令把 `echo rc=$?` 写进文件，下一条 Bash 再 `tail` 那份文件读出来。认的只有同一条命令里读出来的，与先存 `rc=$?`、之后再 echo 的。
- ② 用绝对路径点名的报告此刻不在，照旧是 cite-check 退 2、放行。派发提示只要求相对路径那一种「找不到才拒」。
- 共用约束第 94 行没改，原因见上一节。
- 命名检查之外的交回判法，都没在真记录上逐条调用地量，只量了交回这一层。

## 草稿目录清理

`<草稿>`（4.3M）里没有仓副本、编译目录或库拷贝。`hook-swap.py` 每次跑完都删了它的草稿树，只留下 `<草稿>/hook-swap/` 下的喂法日志与反向规格。留着的东西：`baseline/` 是开工时三份文件的原样，`draft/` 是换上的那一版，`tools/` 是量真记录、复跑开关、生成规格的脚本，另有各次读数。要撤回换上的改动，按 `<草稿>/hook-swap/reverse-*.json` 倒着跑 `hook-swap.py`：先撤第二次换上写的那一份，再撤第一次的（文件名里带写下时的时刻，`ls -t <草稿>/hook-swap/reverse-*.json` 排在最前的两份；dry-run 那一次也留了一份，不用撤）。
