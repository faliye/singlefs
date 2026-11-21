# no-task-numbers-governance-r2：攻方腿报告（2026-11-19）

轮名 no-task-numbers-governance-r2，治理文件轮第二轮。背景材料 `research/prompts/_no-task-numbers-governance-r2-body.md`，第一轮判决 `research/prompts/no-task-numbers-governance-r1-main-verification.md`。被判文件都在主工作区现读，sha256 与开工快照 `research/prompts/no-task-numbers-governance-r2-snapshot/judged-sha256.txt` 登记的逐个相同（派发闸 `.claude/hooks/runner-dispatch-guard.sh` 不在那份清单里，另记在模型的 `tools/judged-sha256.txt`）。

这份报告进仓，里面不写任务号：探针里的任务号一律写成 `<号>`；量的时候用的是真数字，原样在草稿目录 `/home/fy5090/.local/share/singlefs/work/no-task-numbers-governance-r2-opus/probes/` 与模型 `tools/cases.py` 里（`.py` 不判这一形态）。

## 复跑

模型目录 `research/prompts/no-task-numbers-governance-r2-opus-model/`。复跑命令（交主 agent，见末尾「要主 agent 跑的命令」）：

```
task-server/task submit --cwd /home/fy5090/code/singlefs -- bash /home/fy5090/code/singlefs/research/prompts/no-task-numbers-governance-r2-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/no-task-numbers-governance-r2-opus-run /home/fy5090/code/singlefs/research/prompts/no-task-numbers-governance-r2-opus-model
```

每个文件的 sha256：

```
79ac952124acd31feb5db4e1a02e6be1861633d8f15a3c3ea735be48282ca58e  live-base/clean.md
edc3b8ef60a73375695aa31644d204361bcf2d8cefe164644372e2b1cc37db43  live-base/count-two.md
7ffecdb9cdf728bc0d178d6d8f179b3c6449087b2b4965190f181b6a77477527  live-base/fenced-number.md
2f62a8c3d67bd6e99eb5612cb6ebed7d3e2dc988eb00b450820d397de9110c97  live-base/open-fence.md
62a4f0b25787bb9a24e0c604e660b8e3355aa34888672e03a8c7b22b923f3546  tools/cases.py
eb673fd13042990303e4162492fb9b4497a55cc7fb2a09853254ab895a9a4d05  tools/judged-sha256.txt
e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b  tools/rerun-from-a-clean-work-directory.sh
3235cc0ed06f418e067535af36bfacab216722cff3aecbea5a82f5398e251602  tools/rerun.sh
```

`tools/rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷，只换了 `rerun_steps`：先核 `tools/judged-sha256.txt` 里十份被判文件与 `--repo` 下的逐字节相同（不同退 3），再起 `tools/cases.py`。`cases.py` 每格打 `name=case label=… want=… got=… agree=…`，`want` 是这条腿量到的结局，末行 `name=case_summary cases=N disagreements=M`，`disagreements=0` 才算复跑与腿一致；之后打现存真行的扫描行 `name=scan_class` 与 `name=scan_total`。写闸喂的是 `<运行目录>/cases/fake-root` 下的假仓（`CLAUDE_PROJECT_DIR` 指过去），检出钩子与写入工具一律只判或干跑，模型复跑不往仓里写字。

这条腿没跑过 `cases.py`（载入判据库、喂钩子都归主 agent），下面「量的」都是我在主工作区直接起写入工具（共用约束「子 agent 不交任务」那一条点名能直接跑的 `replace-once.py`、`insert-row.py`、`replace-batch.py`、`append-record.py`、`ledger-append-section.py`）与 Bash 检出钩子 ⑰ 当场判我自己那几条命令得来的；写闸、骨架整份写、派发闸那几格是「推的，待主 agent 跑」。

## 各格判定一览

| 关 | 格 | 判定 | 依据（一句） |
|---|---|---|---|
| 1 新判据行对现存真行 | 1a「同」那一支新认的 `/` 连号误拒 kb 实验数据 | 成立（量的，现存真行 1 行） | 出路「放进反引号」照拒，只能改库 |
| 1 | 1b 日志、编译目录、现场目录名里的任务号放行 | 成立（量的，现存真行 45 行整行放过） | 出路叫人写「日志里的那一行」，日志文件名就带号 |
| 1 | 1c「同一分句」攒批放宽放过调度批号 | 成立（量的，现存记录真行 1 行，造的同形 4 条） | `追加第 N 批`、`写入第 N 批` 都放 |
| 1 | 1d 量词表把「处理、行为、项目」当量词 | 成立（量的，造的同形 3 条；现存真行没找到） | 第一轮只删了「只」，同一类还在 |
| 1 | 1e 任务号「个 / 条」放宽放过「条件」 | 成立（量的，造的同形 1 条；现存真行没找到） | 新放宽带进来的 |
| 2 新放行口子能不能绕 | 2a 围栏只认插入点前文，不认写完之后后面那几行 | 成立（量的，五种写法真写进仓或干跑放过） | 删一行开头围栏、插一行围栏、两次追加，批号就出了围栏 |
| 2 | 2b 判决里「A/B 读数：」一行能装任意几个别的任务号、在判决任何一节、引用块里也放 | 成立（量的；判别子只部分看得到） | 库只认行首与写法，不认是不是 A/B 那一条的号 |
| 2 | 2c 记录里「长跑拆分：」一行能装别的会话的任务号 | 成立（量的，同 2b 的形状） | 整行放过 |
| 2 | 2d 记录里以 `|` 起头、不是表格的一行带别的会话的批号 | 推不出（量的放过；第一轮判决交用户的表已定「整行放」） | 落在已定的放法里 |
| 3 第一轮没实测的接法 | 3a 写闸 Edit / MultiEdit 照 2a 的写法 | 推的，待主 agent 跑（成立的可能大） | 写闸只把插入点前文交给库，代码同 2a 的写入工具 |
| 3 | 3b `replace-batch.py` 的 `count` 只按头一处的前文判 | 成立（量的，一次调用） | 写闸 replace_all 逐处各取前文，这里只取头一处 |
| 3 | 3c `insert-row.py` 插一行围栏 | 成立（量的，一次调用） | 同 2a |
| 3 | 3d `replace-batch.py`、`ledger-append-section.py` 拒任务号与批号时收尾那句出路说成「会话号」 | 成立（量的，文字对不上） | 头几行出路对，收尾一行错 |
| 3 | 3e 骨架照填 | 没打中（填好的 A/B 读数一行量过放；整份 Write 推的，待主 agent 跑） | — |
| 3 | 3f 派发闸与判据库对同一行 A/B 读数判法不同 | 成立（判据库一侧量的；派发闸一侧按源码三行正则手抄算，推的） | 4 条探针派发闸收、判决里库拒 |

## 关 1：新判据行对现存真行

判据：现存真行或现存同形写法被新库误拒、或该拒的同形写法放行，就是打中，报次数。量法：把现存真行原样当新串（旧串等于新串，`count` 取它在文件里出现的次数）写成 `replace-batch.py --dry-run` 的规格，工具按真路径、真前文判（造规格的脚本 `probes/line_spec.py`、对出放行行的脚本 `probes/passed.py`，都在草稿目录）；造的探针用 `append-record.py --dry-run` 追加到这一轮的调度记录 `records/2026-11-19-进仓文字不写任务号与会话短号.md`（31 行，探针从第 32 行起）。候选行由草稿目录的 `probes/collect.py` 按我自己的正则收（不载入判据库），收法在它文件头。

### 1a「同」那一支新认的 `/` 连号：误拒 kb 实验数据，出路走不通（成立）

改法第 1 条叫「同」那一支另认「与」「和」「/」连的号（判决 `research/prompts/no-task-numbers-governance-r1-main-verification.md:36`）。库里连号的写法 `.claude/gate.d/lib-forbidden-notations.py:235`：「TASK_NUMBER_JOINER = r'\s*(?:[、，,/\-–~～]|与|和)\s*'」，「同」那一支 `.claude/gate.d/lib-forbidden-notations.py:240`：「同\s*{TASK_DIGITS}(?={TASK_NUMBER_JOINER}\d{{4,}})」。

现存真行 `.claude/kb/experiments/182-被抛弃根读不出的三条岔路.md:408`（判据表一行，「未确认档 G默认 同 <六位计数> / <六位计数>」，是两个状态计数，不是任务号）整行喂 `replace-batch.py --dry-run`，退 5（任务号写成 `<号>`，别的原样）：

```
✗ 新串写了任务号、会话短号或批号，共 1 处，.claude/kb/experiments/182-被抛弃根读不出的三条岔路.md 一个字节没写：
       第 408 行（任务服务的任务号）：「同 <号> / <号>」
✗ 1 处里 1 处的新串写了会话号，--dry-run 没写
rc=5
```

照库的出路改（`.claude/gate.d/lib-forbidden-notations.py:739`：「命中的其实不是这几样（数量、计数、步号这类代码字面），放进反引号」），把那一段放进反引号再喂一次，照样退 5、点名同一处。改法第 5 条已让任务号在反引号里照判，所以这条出路对任务号那一条判据行的误拒不通，只剩改库或改说法（「同」换成「也是」就放）。

次数（量的）：我按 `同 <四位以上数> <「/」「与」「和」> <四位以上数>`、所在行不带「任务」「task」、不在 `records/` 下收候选，六个目录（`records`、`.claude/kb`、`research/prompts`、`.claude/rules`、`.claude/agents`、`.claude/skills`）只收到这一行；它在 kb 里，判法是误拒 1 行。`与 <数> 和 <数> 相同` 那一支的同形在六个目录里收到 0 行。

四句：不分辨臂（改法第 1 条前后都只这一份库，误拒是改法带进来的）；判别子看得到（行里没有「任务」「task」，「同」前面是「G默认」）；满足判据「现存真行被新库误拒」；改法第 5 条（出路文字）在这一格上不起作用，反而让出路走不通。

### 1b 日志、编译目录、现场目录名里的任务号：整行放过（成立）

任务服务的日志文件名、编译目录名与现场目录名带任务号：`/home/fy5090/.local/state/singlefs/task-server/logs/` 下 11091 个文件，名字的形状是 `task-<8 位十六进制>-<号>.attempt-<次>.log`（`ls … | head -5` 现查）；记录里另有 `builds/task-<8 位十六进制>-<号>-<片>`、`task-server-scenes/task-<8 位十六进制>-<号>-<片>`。库的任务号那一条要「任务 / 任务号」或 `task` 后面跟空白、`#`、`=`、冒号、左括号再跟号（`.claude/gate.d/lib-forbidden-notations.py:238`：「rf'|[Tt]asks?(?:\s*[=:#（(]\s*#?|\s+#?)\s*'」），`task-` 后面是短横，不中。

改法第 5 条把出路写成「任务号换成交给服务的那条命令（不带号）、结局行的 type= 与 exit= 那一段（不抄 task=）或日志里的那一行」（`.claude/gate.d/lib-forbidden-notations.py:734`），背景材料自己也写「读数（日志在 `/home/fy5090/.local/state/singlefs/task-server/logs/`）」。照出路引日志那一行，顺手写日志在哪一份，号就进仓；结局行 `log=` 那一段也是这个文件名。

次数（量的）：六个目录里收到 93 行带 `task-<8 位十六进制>-<四位以上数>`；93 行逐行喂 `replace-batch.py --dry-run`（一次调用，1.69 秒、最大常驻 85848 KiB，`/usr/bin/time` 量的），拒 48 行（同一行里别处另有带「任务」的号），放 45 行：`records` 15 行、`.claude/kb` 1 行、`research/prompts` 29 行（放行清单 `probes/e1.passed`）。例 `records/2026-11-17-subagent账本自上而下清欠.md:450`、`records/2026-11-17-里程碑四收口续做.md:164`、`records/2026-11-10-门禁与实验欠账收尾调度.md:117`。造的三条（记录探针第 8、9、10 条：反引号里的日志路径、反引号里的编译目录、结局行带 `log=` 那一段）`append-record.py --dry-run` 都放。

四句：不分辨臂（改法前后都放）；判别子看得到（`task-<8 位十六进制>-<四位以上数>` 是任务服务独有的命名）；满足「该拒的同形写法放行」，且现存真行里有；改法第 5 条的出路文字在这一格上反而把人引过去。

### 1c「同一分句」攒批放宽：放过调度批号（成立）

库 `.claude/gate.d/lib-forbidden-notations.py:265`：「BATCH_SIZE_WORDS = re.compile(r'(?:seq|攒|追加|覆写|写入)\s*(?:批|$)')」，判法 `.claude/gate.d/lib-forbidden-notations.py:412`：「return BATCH_SIZE_WORDS.search(clause) is not None」，`clause` 是同一分句里这一处前面的文字。「追加」「写入」「覆写」是记录里描述调度的常用词，`$` 那一支让它们紧挨在 `第 N 批` 前面时也放，「批」那一支让同一分句前面任何「追加批注」这类词也放。

现存真行（量的）：`records/2026-11-17-subagent账本清欠调度记录.md:75` 是调度批号（`依赖表追加第 4 批`，派工具员那一批），不在表格行里，整行放过，放过它的只有这一条放宽。六个目录里按我的候选正则收到 141 行，喂 `replace-batch.py --dry-run` 放 129 行（`probes/e3.passed`）；`records` 与 kb 下放行的逐行看过，kb 的 23 行都是 fsync 攒批的大小（`seq 批 1`、`追加批 10`、`覆写批 1`、`攒批 64`，放得对），`records` 下除上面那一行外是表格行（另一条放宽）与 `攒批 12.04 秒`（流的名字）。

造的同形（量的，记录探针第 1–4 条，`append-record.py --dry-run` 都放）：`依赖表追加第 7 批`、`那一路会话写入第 7 批的改动`、`别的会话覆写批 105 的补丁`、`这一批追加批注后第 7 批交回`；对照 `它的第 105 批写入了三份文件`（「写入」在后面）拒。

四句：分辨臂——改法第 3 条之前是「前面紧挨」（背景材料「攒批那一条从「前面紧挨」放宽到「同一分句里前面有」」），紧挨那一版同样放过记录第 75 行（`追加第 4 批` 是紧挨，这一行不分辨两版），同一分句这一版另多放第 4 条那种；判别子部分看得到（`追加批 10` 与 `覆写批 105` 字面同形，机器分不开；`第 N 批` 那种分得开：我收到的 141 行候选里，fsync 攒批的大小都写成 `批 N`，没有一处写成 `第 N 批`）；满足「该拒的同形写法放行」；改法第 3 条正是带进这一格的那一条。

### 1d 量词表：「处理、行为、项目」当量词放过（成立，造的同形）

库 `.claude/gate.d/lib-forbidden-notations.py:253`：「BATCH_NUMBER_MEASURE_WORDS = '个条份项次件台组块页字道格趟片轮天周位行种处遍%'」，`批 N` 后面（可隔空白）跟这些字之一就当数量放。第一轮攻方打中「跟副词只的」，改法第 3 条只把「只」拿掉；同一类（量词字恰好是一个双字词的头一个字）还在。造的三条（记录探针第 5–7 条）`它的批 105 处理的是写闸`、`它的批 105 行为不对`、`它的批 105 项目里改了三份文件`，`append-record.py --dry-run` 都放；对照 `它的批 105 改了三份文件` 拒。现存真行：我按 20 个这样的双字词收候选，六个目录收到 0 行（推的：只查了这 20 个词）。

四句：不分辨臂（改法前后都放）；判别子部分看得到（「处」后面跟「理」与跟空白分得开，「批 3 处改动」这类真量词跟着一起受影响）；满足「该拒的同形写法放行」（同形是第一轮那一格）；改法第 3 条只修了「只」那一个字，对这几格不起作用。

### 1e 任务号「个 / 条」放宽放过「条件」（成立，造的同形）

改法第 1 条加的放宽 `.claude/gate.d/lib-forbidden-notations.py:242`：「(?:{TASK_NUMBER_JOINER}{TASK_DIGITS})*(?!\s*[个条])')」。造的 `任务 <号> 条件不满足就重交`（记录探针第 13 条）放；对照 `日志里任务 <号> 那一行` 拒。现存真行收「任务 <号> 条件 / 条目 / 条款 / 个别 / 个人」0 行。四句：分辨臂（改法第 1 条之前这一条拒）；判别子看得到；满足「该拒的同形写法放行」；改法第 1 条带进来的。

### 记录探针的原样输出（量的）

18 条探针，一行一条追加到 31 行的调度记录之后（第 k 条在第 31 + k 行），`append-record.py <记录> <探针文件> --dry-run` 退 7。任务号写成 `<号>`，别的原样（原样在 `probes/records-probes.out`）：

```
  ✗ 新串写了任务号、会话短号或批号，共 4 处，records/2026-11-19-进仓文字不写任务号与会话短号.md 一个字节没写：
       第 46 行（批号）：「批 105」
       第 47 行（任务服务的任务号）：「任务 <号>」
       第 48 行（任务服务的任务号）：「任务 <号>」
       第 49 行（批号）：「第 105 批」
rc=7
```

| 条 | 探针（任务号写成 `<号>`） | 结局 |
|---|---|---|
| 1 | `- 依赖表追加第 7 批（照现存记录行的写法）` | 放 |
| 2 | `- 那一路会话写入第 7 批的改动` | 放 |
| 3 | `- 别的会话覆写批 105 的补丁` | 放 |
| 4 | `- 这一批追加批注后第 7 批交回` | 放 |
| 5 | `- 它的批 105 处理的是写闸` | 放 |
| 6 | `- 它的批 105 行为不对` | 放 |
| 7 | `- 它的批 105 项目里改了三份文件` | 放 |
| 8 | `- 日志 ~/.local/state/singlefs/task-server/logs/task-<十六进制>-<号>.attempt-1.log 第 19–20 行` | 放 |
| 9 | `- 编译目录 builds/task-<十六进制>-<号>-1 是服务留的` | 放 |
| 10 | `- 结局 type=succeeded exit=0 log=<日志目录>/task-<十六进制>-<号>.attempt-1.log` | 放 |
| 11 | `- 长跑拆分：项 x 用时 9000 秒，任务 <号>；顺带记下别的会话的任务 <号>、<号> 也慢` | 放 |
| 12 | `| 别的会话的第 105 批还没交回`（一格、不是表） | 放 |
| 13 | `- 任务 <号> 条件不满足就重交` | 放 |
| 14 | `- TASK <号> 退 0`（大写，近似变体，第一轮攻过的角度，不计） | 放 |
| 15 | `- 它的批 105 改了三份文件`（对照） | 拒（第 46 行） |
| 16 | `- 日志里任务 <号> 那一行`（对照） | 拒（第 47 行） |
| 17 | `- **长跑拆分**：项 x，任务 <号>`（加粗的标签，规则写的是以「长跑拆分：」起头，拒得对） | 拒（第 48 行） |
| 18 | `- 它的第 105 批写入了三份文件`（对照） | 拒（第 49 行） |

## 关 2：新放行口子能不能绕

判据：造出一条能把任务号或别的会话的批号写进仓而不被拒的写法，就是打中。

### 2a 围栏只认插入点前文，不认这一次写让后面哪几行出了围栏（成立）

改法第 4 条让写闸 Edit、⑰ 追加与定点写入工具判围栏时先读目标文件，见判决 `research/prompts/no-task-numbers-governance-r1-main-verification.md:39`：「写闸 Edit、⑰ 追加与定点写入工具判围栏时先读目标文件，算出插入点前围栏开没开着」；共用约束 `.claude/agent-common.md:44`：「反引号与围栏代码块只放过批号与会话短号形状的代码字面（步号、参数这类），任务号写在里面照判、照样不写。」。批号与会话短号在围栏里不判（`.claude/gate.d/lib-forbidden-notations.py:727` 的 `skips_fenced_blocks`）。可是一行在不在围栏里不是那一行自己的事：插入点后面已有的行，会因为这一次写删了、加了或改坏了一行围栏而进出围栏，而各处只判这一次写进去的新串。先把批号当「代码字面」写进围栏（合规，放），再用一次不带号的写把开头那一行围栏去掉或在它前面补一行围栏，批号就落在围栏外的正文里。第一轮攻过的是「以收尾围栏起头的新内容」（新串自己），这里新串一个号都不带。

真写进仓的现场（量的，目标在我的模型目录 `research/prompts/no-task-numbers-governance-r2-opus-model/live/`，落盘后挪出仓到草稿目录 `live-moved-out-of-repo/`，sha256 在 `live-sha256.txt`）：

````
== control: 正文里直接写批号
$ python3 scripts/kb/replace-once.py <模型目录>/live/control-replace-once.md $'前文\n' $'前文\n那一路第 7 批交回\n'
  ✗ 新串写了任务号、会话短号或批号，共 1 处，…/control-replace-once.md 一个字节没写：
       第 2 行（批号）：「第 7 批」
rc=8
== step1: 写进围栏
$ python3 scripts/kb/replace-once.py <模型目录>/live/defence-replace-once.md $'前文\n' $'前文\n```\n那一路第 7 批交回\n```\n'
  ✓ …/defence-replace-once.md：命中 1 次，已替换（改名换上新 inode）并回读确认
rc=0
== step2: 删开头围栏行，新串不带号
$ python3 scripts/kb/replace-once.py <模型目录>/live/defence-replace-once.md $'前文\n```\n' $'前文\n'
  ✓ …/defence-replace-once.md：命中 1 次，已替换（改名换上新 inode）并回读确认
rc=0
== 结果（cat -n）
     1	前文
     2	那一路第 7 批交回
     3	```
````

另外四种写法（量的，同样落盘后挪出仓；`cat -n` 是挪出后在草稿目录里现读的）：

````
== insert-row.py 一次调用：在开头围栏前插一行围栏（目标先用 printf 排他新建为「前文 / ``` / 那一路第 7 批交回 / ```」，⑰ 放：批号在围栏里）
$ python3 scripts/kb/insert-row.py <模型目录>/live/defence-insert-row.md --after-literal '前文' --line '```'
  ✓ 插在 …/defence-insert-row.md:2（锚点第 1 行），写前复核 sha256 7ffecdb9cdf7 未变，改名换上新 inode
rc=0
     1	前文
     2	```
     3	```
     4	那一路第 7 批交回
     5	```
== Bash 检出钩子 ⑰，同一条命令两次追加（目标先排他新建为「前文 / ``` / 代码」，末尾围栏开着）
$ printf '%s\n' '```' >> <模型目录>/live/stale-append.md; printf '%s\n' '那一路第 7 批交回' >> <模型目录>/live/stale-append.md; echo rc=$?
rc=0
     1	前文
     2	```
     3	代码
     4	```
     5	那一路第 7 批交回
== 对照：⑰ 往干净文件追加同一行，当场拒（拒绝说明「printf >> …/stale-append-control.md 第 2 行任务号、会话短号或批号「第 7 批」」）
== replace-batch.py 一份规格两处依次施加（先写进围栏、再删开头围栏行）
$ python3 scripts/kb/replace-batch.py probes/batch-two-edits.json
✓ 2 处替换，写了 1 个文件（改名换上新 inode），回读一致
two_rc=0
     1	前文
     2	那一路第 7 批交回
     3	```
== replace-batch.py 一处 count 2（目标「``` / 占位甲 / ``` / 占位甲」，头一处在围栏里、第二处在外）
$ python3 scripts/kb/replace-batch.py probes/batch-count.json      # [{"old": "占位甲", "new": "那一路第 7 批交回", "count": 2}]
✓ 1 处替换，写了 1 个文件（改名换上新 inode），回读一致
count_rc=0
     1	```
     2	那一路第 7 批交回
     3	```
     4	那一路第 7 批交回
== 对照：同一文件只换围栏外那一处，--dry-run 退 5，「第 4 行（批号）：「第 7 批」」
````

各处只把插入点前面的文字交给库：`scripts/kb/replace-once.py:148`：「preceding = content[:content.index(old_text)]」；`scripts/kb/insert-row.py:205` 取插入行之前的各行；检出钩子 `.claude/hooks/bash-command-detector.sh:5645`：「preceding = literal_append_preceding_text(write, landing)」，那份前文是起这条命令之前读的文件（`.claude/hooks/bash-command-detector.sh:5620` 起那个函数整份读目标），同一条命令里前一次追加写进去的收尾围栏它看不见（工具员自己写明的「同一条命令里先 `>` 再 `>>` 写同一份的读的是起跑前那一份」，这一格就是它）；`scripts/kb/replace-batch.py:131`：「preceding = texts[path][:texts[path].find(edit['old'])]」只取头一处，`scripts/kb/replace-batch.py:135` 却把每一处都换掉（见 3b）。

写闸是同一种接法（推的，待主 agent 跑模型里「写闸:」那几格）：`.claude/hooks/write-guard.sh:624` 与 `:634` 取每一处 old_string 前面的文字，判的只是 new_string。按代码推，Edit 删开头围栏行、Edit 在开头围栏前补一行围栏、Edit 把开头围栏的信息串改成带反引号（库认法里带反引号的信息串不开围栏，`.claude/gate.d/lib-forbidden-notations.py:434`），与一次 MultiEdit「先写进围栏、再删开头围栏行」，都放；Edit replace_all 一处在围栏里一处在外会拒（写闸逐处各取前文，这一点与 `replace-batch.py` 不同）。

四句：
- 分辨臂：不分辨。改法第 4 条之前（只按新串认围栏）与之后（接上插入点前文）都放；病根是两版共用的「只判这一次写进去的新串」，不是改法第 4 条写错了哪一处。
- 判别子看得到：看得到。写闸已经在别的道里算「Edit / MultiEdit 施加到现文件之后的内容」（`.claude/hooks/write-guard.sh:953`：「"""Write 给的整份内容，或 Edit / MultiEdit 施加到现文件之后的内容」），写入工具手里也有写之前、写之后两份整文。
- 判据分句：「造出一条能把……别的会话的批号写进仓而不被拒的写法」，`那一路第 7 批` 是别的会话那一路的批号，五种写法都真写进了仓（模型目录在仓里）。会话短号同一条路（同样在 `skips_fenced_blocks` 里）。
- 跑前条款给的改法：判决里改法第 4 条在这几格上照中。

我提的改法（只在我的模型上量过、被攻过零轮；模型 `tools/cases.py` 的 `flipped_hits`，没跑，推的，待主 agent 跑）：写之前、写之后整份各交库判一遍（围栏按整份认），写之后多出来的违例行（按行文计数，旧违例相抵）就拒。它不看插入点在哪，五种写法与写闸、⑰ 一次命令里几次写（在内存里依次施加）一个口径；文件里已有的旧违例不拦这一次改别处。代价：每次写判两遍整份文件（推的：`replace-batch.py` 判 93 行用 1.69 秒，整份 kb 文件判一遍是同一量级）。

### 2b 判决里「A/B 读数：」一行当容器（成立；判别子只部分看得到）

规则 `.claude/rules/three-way-inference.md@判决由主 agent 做，不由投票做`：「A/B 读数由主 agent 经任务服务跑出，判决「改法」一节那一条下面单起一行」，并写「判决里别处不写任务号」。库只认行首的 A/B 读数加冒号、值里那一处本身是任务 <号> 或 task <号> 的写法，见 `.claude/gate.d/lib-forbidden-notations.py:398`：「and match.start() >= reading.start('value') and ZERO_ROUND_TASK_NUMBER.match(match.group(0)) is not None)」；它不认在哪一节、不认几个号、不认是不是 A/B 那一条的号；行首认法 `.claude/gate.d/lib-forbidden-notations.py:259`：「LINE_START = r"^[ \t>*_\-]*"」，引用块的 `>` 也算行首。

量法：11 条探针一次插在第一轮判决 `## 回看决策` 标题下面（不是「三、改法」一节），`replace-batch.py --dry-run`，探针第 k 条在第 67 + k 行。任务号写成 `<号>`，别的原样（原样 `probes/verdict-probes.out`）：

```
✗ 新串写了任务号、会话短号或批号，共 5 处，research/prompts/no-task-numbers-governance-r1-main-verification.md 一个字节没写：
       第 70 行（任务服务的任务号）：「task=<号>」
       第 71 行（任务服务的任务号）：「同 <号>、<号>」
       第 73 行（任务服务的任务号）：「任务 <号>」
       第 77 行（任务服务的任务号）：「task=<号>」
       第 78 行（任务服务的任务号）：「task log <号>」
```

放过的有：第 1 条 `A/B 读数：任务 <号>；顺带记下别的会话那条复跑是任务 <号>、门禁是任务 <号>`（三个号，后两个与 A/B 无关）、第 2 条 `> A/B 读数：任务 <号>`（引用块）、第 8 条 `A/B 读数：任务 <号> 与任务 <号>`，三条都在「回看决策」一节里。逐条见 3f 的表。

四句：不分辨臂（改法第 4 条之前整行放、之后只放任务 <号> 写法的那几处，两版都放第 1、8 条）；判别子只部分看得到——「在不在三、改法一节」「号是第几个」机器看得到，「这个号是不是那条 A/B 任务」看不到（`.claude/rules/evidence-discipline.md@判据自己也会写错：打中之后先判是哪一种` 的「判别子观测不到」那一形，那一部分要改判据）；满足「造出一条能把任务号写进仓而不被拒的写法」（第 1 条的门禁那个号）；改法第 4 条只收了行首与文件，没收节与个数。

我提的收法（零轮，推的）：判决里只放过 `## 三、改法` 一节里的那一行，且值里只放头一处任务号写法（连号算一处），后面的照判。它修第 1 条与「回看决策」那一处，修不了「头一处就写成门禁那个号」。

### 2c 记录里「长跑拆分：」一行当容器（成立，同 2b 的形状）

库 `.claude/gate.d/lib-forbidden-notations.py:392`：「if LONG_RUN_SPLIT_LINE.match(line) and」，记录里以它起头的一行整行放过任务号。记录探针第 11 条 `长跑拆分：项 x 用时 9000 秒，任务 <号>；顺带记下别的会话的任务 <号>、<号> 也慢` 放（量的）。规则 `.claude/rules/long-run-split.md` 第 6 步只要这一行写「项名、任务号、用时、怎么拆、拆后的项名与它们重跑的任务号」，别的会话的号不在里面。四句同 2b：不分辨臂；判别子部分看不到（拆后重跑的号与别的号都是任务号）；满足判据；改法第 4 条只收了文件。

### 2d 记录里以 `|` 起头、不是表格的一行（推不出）

记录探针第 12 条 `| 别的会话的第 105 批还没交回`（一格、没有表头分隔行，Markdown 里是一段正文）放（量的）；`ledger-append-section.py --dry-run --judgement '别的会话第 105 批'`（拆分提案账本的索引表，不是依赖表）也放，退 0：「（干跑）会取第 二百七十六 节：索引行插在 …/records/2026-11-17-subagent账本自上而下清欠.md:81」。这两样落在第一轮判决交用户的表已定的放法里（`research/prompts/no-task-numbers-governance-r1-main-verification.md:50` 那一行：表格行只放过 `records/` 下的、不细到哪一列），按字面不算新打中，交主 agent 看：那一行的依据写「`records/` 是依赖表唯一住处」，而拆分提案账本的索引表也住在 `records/` 下、经写入工具写进判定格，这一句的前提不全对（这是辩方腿那一格的事，我只报现象）。

## 关 3：第一轮没实测的接法

判据：两边对同一行判法不同，或照骨架写的判决被闸拒，就是打中。

### 3a 写闸 Write / Edit / MultiEdit（推的，待主 agent 跑）

我没有 Write / Edit 工具，照写闸自证的喂法（`.claude/hooks/write-guard.sh:1668` 那一段：`CLAUDE_PROJECT_DIR` 指到临时目录、`AGENT_HOOK_DETECTIONS` 另给、hook 输入 JSON 从标准输入喂）写进模型 `tools/cases.py` 的 `write_guard_cases`，12 格：Edit 把批号挪出围栏（对照，拒）、Edit 删开头围栏行（放）、Edit 在开头围栏前插一行围栏（放）、Edit 把开头围栏的信息串改成带反引号（放）、MultiEdit 一次调用先写进围栏再删开头围栏行（放）、Edit replace_all 一处在围栏里一处在外（对照，拒）、判决里五条 A/B 探针（2b 与 3f 那几条）、记录里 A/B 读数一行（对照，拒）。want 是按代码推的，跑出来不一致的格照实判。

### 3b `replace-batch.py` 的 `count` 只按头一处的前文判（成立，量的）

见 2a 末一段的现场：一份规格一处 `count: 2`，头一处在围栏里、第二处在外，写进仓、退 0；只换第二处的对照退 5。`scripts/kb/replace-batch.py:131` 只取头一处的前文，`:135` 换掉每一处。写闸的 Edit replace_all 逐处各取一段前文（`.claude/hooks/write-guard.sh:624`），两边对同一种写法判法不同。改法（推的）：逐处各取前文判，同写闸；或并进 2a 那条「整份前后差」。

### 3c `insert-row.py`（成立，量的）

插的新行本身（`--line '```'`）没有号、按插入点前文判得对；它让后面已有的行出了围栏，见 2a。

### 3d 拒绝说明收尾一行把任务号与批号说成会话号（成立，量的，文字对不上）

`replace-batch.py --dry-run` 拒 kb 那一行任务号时，头一行「✗ 新串写了任务号、会话短号或批号」与库的出路对，收尾两行是 `scripts/kb/replace-batch.py:254` 的「处的新串写了会话号」与「新串里的会话号换成「别的会话」这类不带号的说法」（1a 的原样输出）。`ledger-append-section.py --dry-run` 拒正文里的 `第 105 批` 时收尾一行同样是 `scripts/kb/ledger-append-section.py:287`：「新文本里的会话号换成「别的会话」这类不带号的说法，事情与日期照留，再追加」。照收尾那一行改，批号与任务号改不掉。改法（推的）：收尾一行按命中的形态说，或只指回上面库给的出路。

### 3e 骨架照填（没打中）

骨架的占位行 `scripts/agents/verdict-skeleton.py:69` 是 `A/B 读数：任务 <号>`，填上号之后在判决路径上放（量的：2b 那次的第 5 条，第 72 行没被拒）；骨架的待填提示 `scripts/agents/verdict-skeleton.py:66` 叫人写命令与 `type=`、`exit=`，照写不带号。整份骨架照待填提示填好再 Write 喂写闸，写在模型 `skeleton_cases` 里（在假仓里起 `verdict-skeleton.py --repository <假仓> --out <运行目录>/skeleton.md`），推的，待主 agent 跑。

### 3f 派发闸与判据库对同一行 A/B 读数判法不同（成立）

派发闸 `.claude/hooks/runner-dispatch-guard.sh:2583`：「if break_switch_is("zero-round-reading-number-unchecked") or ZERO_ROUND_TASK_NUMBER.search(reading.group("value")):」——值里 search 到一处任务号就收这一行；判据库（2b 引的第 398 行）要值里每一处任务号写法本身都是「任务 <号> / task <号>」。规则要实现员的派发提示带「同样一行」，派发闸自己的出路叫人写「A/B 读数：任务 <号>，<量到什么>」（`.claude/hooks/runner-dispatch-guard.sh:2588`），`<量到什么>` 里写两臂的号或抄结局行，派发闸收、同一行进判决被库拒。判据库那一列是 2b 那次量的；派发闸那一列是我把 `.claude/hooks/runner-dispatch-guard.sh` 第 1023、2569、2570 行三条正则手抄成字面（草稿目录 `probes/dispatch_side.py`）在本地算的，推的；模型 `library_cases` 照 `verdict-skeleton.py` 的抽法从源码原样抽出来再判一遍，待主 agent 跑。

| 条 | 探针（任务号写成 `<号>`） | 判据库（判决里，量的） | 派发闸（推的） |
|---|---|---|---|
| 1 | `A/B 读数：任务 <号>；顺带记下别的会话那条复跑是任务 <号>、门禁是任务 <号>` | 放 | 收 |
| 2 | `> A/B 读数：任务 <号>` | 放 | 收 |
| 3 | `A/B 读数：任务 <号>，结局 task=<号> type=succeeded exit=0` | 拒 | 收 |
| 4 | `A/B 读数：任务 <号>（同 <号>、<号> 两臂）` | 拒 | 收 |
| 5 | `A/B 读数：任务 <号>` | 放 | 收 |
| 6 | `1. A/B 读数：任务 <号>` | 拒 | 不收 |
| 7 | `A/B 读数：TASK <号>` | 放（大写不中任务号那一条） | 收 |
| 8 | `A/B 读数：任务 <号> 与任务 <号>` | 放 | 收 |
| 9 | `A/B 读数：见日志 task-<十六进制>-<号>.attempt-1.log` | 放（1b 那一格） | 不收 |
| 10 | ``A/B 读数：任务 <号>，结局 `TASK_SERVER_RESULT task=<号> type=succeeded exit=0` `` | 拒 | 收 |
| 11 | `A/B 读数：任务 <号>；task-server/task log <号> 末行` | 拒 | 收 |

第 3、4、10、11 条派发闸收、判据库拒；第 9 条反过来（库放、闸不收，号从日志名进仓，归 1b）。四句：不分辨臂（改法第 4 条之前库整行放，那时两边对这四条一致地放）；判别子看得到；满足「两边对同一行判法不同」；改法第 4 条只把行首认法对齐了，值的判法没对齐——它正是带进这一格的那一条。改法（推的）：派发闸收 A/B 读数那一行时也调判据库，把会被判决拒的写法当场拒，出路写成判决能照抄的样子。

## 几个改法各修哪一格

| 改法 | 1a | 1b | 1c | 1d | 1e | 2a | 2b | 2c | 3b | 3d | 3f |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 判决改法第 1–6 条（已换上） | 不修（带进来的） | 不修 | 不修（第 3 条带进来的） | 不修 | 不修（第 1 条带进来的） | 不修（量的：五种写法都放） | 部分（行首、文件） | 部分（文件） | 不修 | 不修 | 不修（第 4 条带进来的） |
| 我提：写前写后整份各判、多出的违例行就拒（模型 `flipped_hits`） | — | — | — | — | — | 修（推的，待主 agent 跑） | — | — | 修（推的，待主 agent 跑） | — | — |
| 我提：`replace-batch.py` 逐处各取前文 | — | — | — | — | — | 只修 count 那一种（推的） | — | — | 修（推的） | — | — |
| 我提：⑰ 同一条命令里几次写在内存里依次施加再判 | — | — | — | — | — | 只修 ⑰ 那一种（推的） | — | — | — | — | — |
| 我提：任务号那一条另认 `task-<8 位十六进制>-<号>`，出路写「引日志写它打出的那一行，不写日志文件名」 | — | 修（推的） | — | — | — | — | — | — | — | — | — |
| 我提：`追加`、`覆写`、`写入` 只放紧挨着的 `批 <数>`，不放 `第 <数> 批`；分句放宽只留给 `seq`、`攒` | — | — | 部分（推的：`覆写批 105` 那种照放） | — | — | — | — | — | — | — | — |
| 我提：量词字后面紧跟汉字的不当量词 | — | — | — | 修（推的；`批 3 处改动` 这类真量词会跟着被拒） | — | — | — | — | — | — | — |
| 我提：「个 / 条」后面紧跟汉字的不放 | — | — | — | — | 修（推的） | — | — | — | — | — | — |
| 我提：判决里只放 `## 三、改法` 一节、值里头一处 | — | — | — | — | — | — | 部分（推的） | — | — | — | — |
| 我提：「同」那一支的 `/` 连号只在行里另有「任务」「task」时算，或出路改说「换说法」 | 修（推的） | — | — | — | — | — | — | — | — | — | — |
| 我提：收尾一行按命中形态说 | — | — | — | — | — | — | — | — | — | 修（推的） | — |
| 我提：派发闸收 A/B 那一行时也调判据库 | — | — | — | — | — | — | — | — | — | — | 修（推的） |

表里「我提」的每一条都只在我的模型上量过或只推过、被攻过零轮；「修」都没在副本上实现、没跑。

## 没打中的形状

- 写闸 Edit replace_all：逐处各取前文（`.claude/hooks/write-guard.sh:624`），一处在围栏里一处在外按代码推会拒，没打中（推的，模型里有这一格）。MultiEdit 依次施加、各取施加那一刻的前文（`:634`），新串自己那一段判得对；打中它的只有 2a 那种「改后面已有的行的围栏」。
- 括号里短号后面跟会话词（改法第 2 条新加的那一支）：六个目录按同一个正则收候选，`records` 外只有 1 份文件命中，`records` 里命中的都是真短号（`（46 量的`、`（d5 转述` 这类），没找到误拒。
- kb 里 `第 N、M 批`、`第 N–M 批`（改法第 3 条新认的区间与并列）：kb 收到的都是调度批号（例 `.claude/kb/checks-owed.md:21`、`:447`），拒得对，没找到误拒。
- kb 里 fsync 攒批大小（`seq 批 1`、`追加批 10`、`覆写批 1`、`攒批 64`）：23 行候选全放（量的，1c），没有误拒。
- 判决里 A/B 读数一行写成编号列表项（`1. A/B 读数：…`）、值里带 `task-server/task log <号>`：库与派发闸都不收，两边一致。
- 记录里加粗的 `**长跑拆分**：`：库拒，规则写的是以「长跑拆分：」起头，拒得对。
- 骨架：填好的 A/B 读数一行放；整份照填 Write 待主 agent 跑。
- 取样范围：现存真行只收了我自己正则的几类候选（`probes/collect.py` 文件头），六个目录全扫的那一遍写在模型 `scan_real_lines` 里、待主 agent 跑；造的探针 18 条记录、11 条判决、5 种围栏写法、2 份 kb 行、2 条锚点。

攻击面之外的线索（交主 agent，不计入各格）：批号那一条放过 `#第 N 批`（`.claude/gate.d/lib-forbidden-notations.py:256` 的 `(?<![#{ASCII_WORD}])第`，原意是调度记录的锚点），正文里写 `别的会话 #第 105 批 还没交回` 与派发提示那一行 `批次：records/<调度记录>.md#第 5 批` 照抄进记录，`append-record.py --dry-run` 都放（量的，记录第 32、33 行没被拒、第 34 行的对照被拒）。这一条放法第一轮之前就有，不是这一轮的修补。

## 这条腿自己的限度

- 写闸、骨架整份 Write、派发闸从源码抽正则那几格我没跑（载入钩子与判据库归主 agent），一览表里标了「推的」。
- 「量的」都是主工作区上起写入工具与 ⑰ 当场判我的命令量到的，不在副本上；判决要引的数照模型复跑。
- 2a 那五个现场真写进了仓里我的模型目录，量完挪出仓到草稿目录 `live-moved-out-of-repo/`，仓里只留干净的 `live-base/` 四份（其中 `fenced-number.md` 的批号在围栏里，是库放过的写法）。
- 现存真行的候选是我自己的正则收的，「没找到」只对那几个正则成立；1d、1e 只有造的同形、没有现存真行。
- 这条腿不涉及崩溃状态枚举，没有原型跑全量崩溃状态，状态数合计 0。
- 我提的改法都被攻过零轮，「修」都是推的。

## 没做什么

- 不判辩方那几格（第一轮判决够不够得着、交用户的表三处判法站不站得住）；2d 只报现象。
- 不替主 agent 采纳；副本上没有量任何数。
- 没往统一任务服务交任务；模型复跑、喂写闸与检出钩子、骨架整份写的命令在下一节，判决要引的数没复跑。
- 没跑 `cases.py`（载入仓里的判据库与钩子，归主 agent），它的格与扫描行都没出过结局。

## 要主 agent 跑的命令

1. 命令：`task-server/task submit --cwd /home/fy5090/code/singlefs -- bash /home/fy5090/code/singlefs/research/prompts/no-task-numbers-governance-r2-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/no-task-numbers-governance-r2-opus-run /home/fy5090/code/singlefs/research/prompts/no-task-numbers-governance-r2-opus-model`
   - 在哪起：仓根 `/home/fy5090/code/singlefs`（新 shell）。
   - 看哪几行：`task wait` 末行 `TASK_SERVER_RESULT` 的 `type=succeeded exit=0` 算过；退 3 是被判文件与 `tools/judged-sha256.txt` 对不上（主工作区换过，先看是谁改的）。`task-server/task log <号> --all` 里：`name=rerun_no_disagreement` 与 `name=case_summary cases=N disagreements=0`（N 是格数：71，数 `cases.py` 里各段的格得来）算与腿一致；有 `agree=no` 的行逐格照实判。扫描行推的预期：`name=scan_class class=log-path-task-number-passed` 三行合计 45（`records` 15、`.claude/kb` 1、`research/prompts` 29），`class=clause-batch-size-exempt-in-records dir=records lines=1`，`class=same-joined-refused-outside-records dir=.claude/kb lines=1`。
   - 估时（推的）：喂钩子与干跑几十次、每次几秒；扫六个目录 2758 份、211M 文本，第一轮攻方同一类扫描交过服务跑完；合计估不超过 10 分钟，没量过。探针的内存：`replace-batch.py --dry-run` 判 93 行最大常驻 85848 KiB（量的），模型整趟没量过。

## 清理

- 没建仓副本、编译目录与库拷贝。草稿目录 `/home/fy5090/.local/share/singlefs/work/no-task-numbers-governance-r2-opus` 共 5.6M（`du -sh`），是探针、规格、输出与挪出仓的现场，留给主 agent 核。
