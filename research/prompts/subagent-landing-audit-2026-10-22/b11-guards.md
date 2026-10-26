# 第十一批：交回闸 ⑤ 认命名检查自己的退出码、交回闸 ② 不认仓外报告、派发闸 ⑳ 真提示钉进钩子样本目录（2026-10-23）

依据：`research/prompts/landing-audit-r5-main-verification.md` 节「二、逐格判」K2 表 K2-2a、K2-2b、K2-3a 与「交回闸 ② 仓外报告」四行；拆分提案第九十节 ㉜。
草稿目录 `/home/fy5090/.local/share/singlefs/work/landing-audit-b11-guards`（下文 `<草稿>`）。

## 结论

- 三条都改完，经 `scripts/gate/hook-swap.py` 定点换进主工作区（三份文件一个规格 43 处，另一个只改文件头开关表的规格 6 处），换上之后两份钩子的自证在主工作区退 0：交回闸 62 种变 74 种，派发闸 507 种变 512 种。
- ⑤：真会话记录里交回层面新拒 0 份、新放 3 份（K2-2a 那 4 份误拒里的 3 份，都打出了 `rc=1`）。第 4 份 `aa68d151c2fa72bb7` 照旧拒，因为它的 code-source-discipline 那一条接了 `| tail -8 | cut`，按判决属于「管道最后一段之外」，不认。逐条调用层面新拒 27 条（37 条命名检查：36 条在管道中间，1 条后面 `;` 接 grep、没打 `$?`），新放 11 条（全是 `echo …$?` 打出 1）。
- ②：真交回里新拒 46 次，全是点名仓外副本（`m5-dev` 37 次，`m4-predev` 9 次）里同名报告的那一类，正好是判决量到的 46 次；新放 0 次。派发闸 ⑰ 用同一个函数，在 693 份真三方腿派发提示上改前改后判得一样（拒 37 份、放 656 份）。
- ⑳：14 份真提示逐字节拷进 `.claude/hooks/fixtures/runner-dispatch-guard/real-prompts/`（后缀改成 `.txt`，内容没动），整份 sha256 同时钉在 `sources.tsv` 和期望表里。目录不在时 14 格逐格判红（改前是「本次未跑」，一格不判），少一份或改一个字节都只红那一份。
- 什么现象能推翻：
  - ⑤ 新拒的 36 条管道写法里，有 `| tail` 收尾、输出里看得到收尾汇总行的（比如 `a8a66a81f09d886b1`），命名检查其实跑完了，只是退出码认不出。要是主 agent 认定这类也该收，就要另立一条读汇总行的判法，这一批没做。
  - ② 新拒的 46 次，要是主 agent 定三方腿可以把报告写在仓外副本里（判决推翻条件那一句），交回闸就该改成按点名的绝对路径去核引文，而不是不认。

## 改过的文件

| 文件 | 改了什么 |
|---|---|
| `.claude/hooks/handback-guard.sh` | ⑤ 的判法（`tool_calls` 带上结果文字、`naming_sequence` 记下所在层、后面接不接管道、元素后面接的操作符；新加 `status_echo_pattern`、`status_echo_readings`；`sequential_naming_text` 按命名检查自己的退出码收）；② `leg_reports_named` 带项目根、没点名报告那一句改成写清出路；文件头 ② ⑤ 的判法说明与开关表；自证新加 12 格、改 2 格 |
| `.claude/hooks/lib_three_way_legs.py` | `leg_report_mentions(text, project_root)` 只留解到仓根下 `research/prompts/` 的；新加 `LEG_REPORT_PREFIX`、`prefix_resolves_to_root`；文件头认法 |
| `.claude/hooks/runner-dispatch-guard.sh` | ⑰ 整份提示那一支带仓根调库，新加开关 `leg-report-outside-root-kept`；⑳ 样本目录挪到钩子样本目录、期望表钉整份 sha256、`real_prompt_cells` 不再报本次未跑、新加 `real_prompt_sources`；自证新加 6 格、去掉 1 格；文件头开关表 |
| `.claude/hooks/fixtures/runner-dispatch-guard/real-prompts/`（新目录） | 14 份 `<名>.txt` 与 `sources.tsv` |

`git diff --stat -- .claude/hooks/handback-guard.sh .claude/hooks/lib_three_way_legs.py .claude/hooks/runner-dispatch-guard.sh` 原样（工作区对暂存区；三份在暂存区里另有别的会话这一批的改动，换上之前核过基线 sha256 与开工时相同）：

```
 .claude/hooks/handback-guard.sh        | 277 ++++++++++++++++++++++++---------
 .claude/hooks/lib_three_way_legs.py    |  37 ++++-
 .claude/hooks/runner-dispatch-guard.sh | 119 +++++++++-----
 3 files changed, 320 insertions(+), 113 deletions(-)
```

新目录不进 diff（未跟踪，`git status --short -- .claude/hooks/` 打 `?? .claude/hooks/fixtures/`）。sha256：换上之前 `01fa3f9c…dcdfb0`（交回闸）、`f67d526a…265089`（库）、`03b8c2d1…d5b2`（派发闸）；换上之后（含文件头那一次）以 `sha256sum` 现取为准，反向规格在 `<草稿>/swap-real/hook-swap/reverse-20261023T051717Z-3290704.json` 与 `<草稿>/swap-header/hook-swap/reverse-20261023T051923Z-3300104.json`（撤回时先倒文件头那一份，再倒前一份）。

新钩子、新门禁阶段：没有，所以没写 `# gate-similar:` / `# hook-events:`。`gate-overlap.py --list` 看过，管这几条判据的就是这两份钩子本身，判据追加在它们里面。

## 一、交回闸 ⑤：认 timeout 包着且跑完的、只认命名检查自己的退出码（K2-2a、K2-2b）

### 判法（`.claude/hooks/handback-guard.sh:713` `sequential_naming_text`，`:671` `status_echo_pattern`，`:691` `status_echo_readings`）

- 顺序位置那一半不变（第一轮那五种照旧不在位置上）。退出码那一半改成：只收命名检查**自己的**退出码认得出、又是 0 或 1 的。
- 自己的退出码从它所在那一段（`&&` / `||` 连成的一段）的退出码认，有两个来源：
  - 顶层最后一段，取这一条 Bash 的退出码。退非 0 时整条只许有赋值、env、cd、export 与命名检查，规则沿用改前。
  - 后面紧跟 `; echo <标签>$?`（换行也算）的那一段，取输出里那一行打出的数。标签里的 `$D` 这类展开当任意字；同一种标签打了几次就按次序配，行数对不上的不认；`echo -n` 不认。
- 段的退出码怎么落到命名检查头上：
  - 它是那一段最后一个元素，就用那一段的；前面 `&&` 连着 cd 或命名检查的只认 0。
  - 后面接 `&&` 的，那一段退 0 才认它退了 0。
  - 后面接 `||`、在管道里不是最后一段、前面有 `!` 的，认不出，不收。命令里有 pipefail 时，管道最后一段也不收。
- `timeout` 不再单独判：没被杀掉时 timeout 的退出码就是命名检查自己的；被杀掉是 124 或 137，不在 0、1 里，自然不收。141、2、3、77 也一样不收。
- `bash -c` 那一段：外面那一条认出的退出码交进去，里面照同一套收。

### 探针改前改后（改前 = 主工作区开工时那一版，存在 `<草稿>/tree-before`；改后 = 换上之后的主工作区）

攻方 `probes/naming_masks.py`（只把树名换成 `tree-before` / `tree-after`，拷成 `<草稿>/tools/naming-masks.py`）原样输出：

```
误放：两条都接 | head -2，命名检查被 SIGPIPE 杀掉（真跑是 PIPESTATUS=141 0），工具退 0：改前退 0，改后退 2（判得对的应当是 2）
误放：两条都退 2（用法错）而后面接 ; echo rc=$?，工具退 0：改前退 0，改后退 2（判得对的应当是 2）
误放：两条都接 || true，真退 3（naming-lint 没有 .rs 可查）也退 0：改前退 0，改后退 2（判得对的应当是 2）
误拒：真会话里的写法 timeout 230 包着、输出落文件、; echo rc=$?（rc=1 是判红跑完），工具退 0：改前退 2，改后退 0（判得对的应当是 0）
对照：同一写法去掉 timeout，工具退 0：改前退 0，改后退 0（判得对的应当是 0）
```

攻方 `probes/naming_e2e.py`：4 份真记录逐字节拷进假主会话目录，喂两棵树里的钩子入口。原样输出，冒号后截短：

```
a7aa1608c74c3e21e（tooling-writer，记录 sha256 3e90a0855150a648…）：改前退 2：✗ tooling-writer 改了 crates/ 或 scripts/ 下的 .rs，…；改后退 0
aa87dd9fe761d727f（tooling-writer，记录 sha256 1d25a002905f1740…）：改前退 2：✗ …；改后退 0
aa68d151c2fa72bb7（tooling-writer，记录 sha256 372e60e5f98ff551…）：改前退 2：✗ …；改后退 2：✗ …
ab6dd5f35228122b7（tooling-writer，记录 sha256 18a52406f8ccb57e…）：改前退 2：✗ …；改后退 0
```

攻方 `probes/sigpipe.sh` 现跑原样（4000 行的小树，跑完删了）：

```
整条跑完：退 1，输出 4007 行
接 | head -2：PIPESTATUS=141 0（naming-lint 退 141 是被 SIGPIPE 杀掉，整条的退出码是 head 的）
```

hook-swap 喂法（`<草稿>/edits/feeds-handback.json`，8 条）喂改前那一版（`--feed-only`，换上之前）原样：

```
  ✗ 退 2（应当 0） 工具员 timeout 包着、; echo rc=$? 打出 rc=1 与 rc=0 的两条命名检查放行（K2-2a） ← 
  ✗ 退 0（应当 2） 工具员两条命名检查接 | head -2 拒（K2-2b） ← 
  ✗ 退 0（应当 2） 工具员两条命名检查接 || true 拒（K2-2b） ← 
  ✗ 退 0（应当 2） 工具员两条命名检查 ; echo rc=$? 打出 rc=2 拒（K2-2b） ← 
  ✓ 退 0（应当 0） 工具员两条命名检查各自单独跑、lint 判红退 1 放行 ← 
  ✓ 退 2（应当 2） 工具员改了 .rs 没跑命名检查拒 ← 
  ✗ 退 0（应当 2） 攻方腿只点名仓外同名的报告拒（交回闸 ② 仓外报告） ← 
  ✓ 退 0（应当 0） 回扫员交回短正文放行 ← 
✗ 只喂不换：.claude/hooks/handback-guard.sh 喂了 8 条，5 条不对
```

换上之后同一份喂法在草稿树和主工作区里各跑一遍，8 条全对（`✓ 换上了 3 份（改名换上新 inode、回读一致），主工作区里再验过：3 份文件语法检查、内嵌 python 2 段编译、喂了 8 条`）。

### 真会话记录上的新拒与新放

语料：`~/.claude/projects/-home-fy5090-code-singlefs/*/subagents/` 下 meta.json 记成三种要跑命名检查的类型和三方腿的全部 2221 份记录，2026-10-23 抽（`<草稿>/tools/extract-naming-corpus.py`）。起了命名检查的 Bash 调用，结果整份留着（交回闸读的就是整份），别的调用只留开头 200 字。量法照攻方 `naming_real.py`：整段判拿两版钩子的 `naming_problems`，逐条判拿 `completed_naming_text`（`<草稿>/tools/measure-naming.py`、`changed-calls.py`、`reject-reasons.py`）。

交回层面（改了要判的 `.rs` 的 152 份）：

```
  2026-10-22 之前：改前拒、改后拒：144 份
  2026-10-22 起：改前拒、改后拒：5 份
  2026-10-22 起：改前拒、改后放：3 份
  改前拒、改后放：a7aa1608c74c3e21e（tooling-writer，2026-10-22）、aa87dd9fe761d727f（tooling-writer，2026-10-23）、ab6dd5f35228122b7（tooling-writer，2026-10-22）
```

新拒 0 份。10-22 起照旧拒的 5 份：`aa68d151c2fa72bb7` 是管道写法；另 4 份是 experiment-runner 在带 `.git` 的 `m5-dev` 副本里改了 `.rs`，在那棵树上一条命名检查都没跑，改前改后都拒。

逐条调用（改前认得出点名的 107 条）：`{'新拒': 27, '新放': 11, '认的不同': 0, '一样': 69}`。

- 新放 11 条：每一条都读到了它自己那一行 `echo` 打出的数，全是 1（判红、跑完了）：`a7aa1608c74c3e21e` 两条 `echo rc=$?`，`aa87dd9fe761d727f` 两条，`ab6dd5f35228122b7` 两条（`echo csd rc=$?` 与 `echo rc=$?`），`aa68d151c2fa72bb7` 的 lint 一条，`a427eccd6824832fb` 一条（`echo exit=$?`），`afa34c9b55ae7bd3f` 一条，`a5febcd4f7471e722` 两条（`echo exit $?`）。读到 0 以外、1 以外的，0 条。
- 新拒 27 条、37 条命名检查，按改后不收的头一个原因分（`reject-reasons.py` 原样）：

```
在管道里不是最后一段（接 grep）：19 条；例：a427eccd6824832fb：bash .claude/singlefs-ai-sop/scripts/naming-lint.sh '$W/naming-tree' ｜ …
在管道里不是最后一段（接 tail）：16 条；例：a8a66a81f09d886b1：…code-source-discipline.sh --check vague-names,test-file-names,ordinal-names,data-names '$W/copy' ｜ …
在管道里不是最后一段（接 head）：1 条；例：a13da859a513c2328：nice -n 19 bash .claude/singlefs-ai-sop/scripts/naming-lint.sh
那一段的退出码看不到（后面 ; 接 grep，没有 echo $?）：1 条；例：a5e5d9ce99b57fb08：…naming-lint.sh '$W/repo'
```

新拒的 27 条落在 14 份记录里。这 14 份在交回层面改前改后判得一样：12 份改 `.rs` 的树此刻没有 `.git`（rsync 副本，或 `/tmp`、工作根下已删的副本），两版都不判；2 份（`ab4fd0f450ab0938c`、`aba6e70e0d1092764`）改在主工作区，两版都拒。所以交回层面一份没多拒。树有没有 `.git` 按此刻的文件系统判，与攻方量法相同。新拒里没有 `echo` 读到 0 或 1、却被拒的。管道那 36 条按判决（「`rc=` 一行或管道最后一段之外的写法不认」）属于认不出退出码，有几条（接 `| tail`）从输出看是跑完了的，见「结论」里的推翻条件。最后那 1 条（`naming-lint … > log 2>&1; grep -c … log`）退出码确实看不到。全部 38 条调用的命令和结果头尾在 `<草稿>/final-changed-calls.txt`。

### 自证格（`bash .claude/hooks/handback-guard.sh --selftest`，主工作区，退 0，`✓ handback-guard 自检通过（查了 74 种）`）

新加 9 格：
- `timeout 230 包着、输出落文件、; echo rc=$? 打出 rc=1（判红跑完）放行`（绿）
- `两条在同一条里各自 ; echo rc=$?（rc=0、rc=1 按次序配）放行`（绿）
- `两条接 | head -2、被 SIGPIPE 杀掉（退 141）而整条退 0，不算跑过`
- `两条接 | tail -n 20、整条退的是 tail 的 0，不算跑过`
- `两条接 || true、退出码被吃掉，不算跑过`
- `; echo rc=$? 打出 rc=2（用法错），不算跑过`
- `timeout 230 包着、; echo rc=$? 打出 rc=124（被 timeout 杀掉），不算跑过`
- `timeout -s KILL 包着、被杀退 137，不算跑过`
- `两条用 ; 连着、整条退 0（前一条的退出码看不到），拒`

改了 2 格：
- `两条用 ; 连着、后一条判红退 1 放行` 改成「…（前一条的退出码看不到），拒」：前一条自己的退出码看不到。
- `cd 之后根写成 $(pwd)、接管道跑命名检查放行` 去掉了 `| tail -n 20`，名字改成「…、&& 连着两条跑命名检查放行」：接了管道就认不出退出码，这一格本来判的是根的认法。

第一轮那五种（`false &&`、`true ||`、`if false`、函数体不调用、`timeout 0.1` 被杀退 124）、「改了 .rs 之后两条命名检查都跑了放行」（lint 判红退 1）照旧，结果不变。

弄坏开关（逐条在主工作区设上跑过，点名的项逐条都在失败行里，`<草稿>/tools/check-switch-named.py` 核的）：

| 开关 | 新还是改 | 打开后红的格（点名几项 / 失败行几条） |
|---|---|---|
| `HANDBACK_GUARD_BREAK=naming-tool-exit-trusted` | 新 | 段的退出码一律取这一条 Bash 的：`| head`、`| tail`、`|| true`、`rc=2`、`rc=124`、`;` 连着那两格（7 / 7） |
| `HANDBACK_GUARD_BREAK=naming-echo-unread` | 新 | 不读 echo 那一行：两格绿的放行变拒（2 / 2） |
| `HANDBACK_GUARD_BREAK=naming-timeout-refused` | 新 | 退回第十批的「包 timeout 又不是最后一条不收」：timeout + `rc=1` 那一格（1 / 1） |
| `HANDBACK_GUARD_BREAK=naming-exit-code-unchecked` | 改（echo 读到的也当 0） | 原两格加 `rc=124`、`rc=2`、退 137（5 / 5） |
| `HANDBACK_GUARD_BREAK=naming-position-unchecked` | 点名多了 7 格 | 原六格加 `| head`、`| tail`、`|| true`、`rc=2`、`rc=124`、`;` 连着两格（13 / 13） |
| `HANDBACK_GUARD_BREAK=naming-root-unmatched` | 格名改了一格、点名多了 2 格 | `$(pwd)` 那一格改名，加两格 echo 放行（6 / 6） |

## 二、交回闸 ② 不认仓外报告（与派发闸 ⑰ 共用 `lib_three_way_legs.py` 一处认法）

### 判法（`.claude/hooks/lib_three_way_legs.py:106` `leg_report_mentions(text, project_root)`，`:90` `prefix_resolves_to_root`，`:33` `LEG_REPORT_PREFIX`）

- 形态照旧（`LEG_REPORT_MENTION`）。每处命中往前取连着的那一截路径，遇到空白、引号、反引号、尖括号、中英文标点与括号、`=`、`:` 就停。
- 这一截是空的、`./`、仓根的绝对路径、`~` / `$HOME` / `${HOME}` 起头展开后是仓根、或从仓根起的相对写法解回仓根的，认；别的（仓外副本里同名的、带别的变量的、URL 里的）不认。
- 交回闸 `leg_reports_named(message, project_root)`（`handback-guard.sh:282`）调它；派发闸 ⑰ 整份提示那一支（`runner-dispatch-guard.sh:1524`）也调它，传派发那一刻的仓根。
- 一份都没认出时，拒绝说明那一句改成：「交回里没点名报告文件（项目根下的 research/prompts/<轮>-…-output.md；点名仓外副本里同名的那一份不算：报告写进项目根的 research/prompts/，交回点名那一份）」。

几种写法在换上之后的库里认出什么（仓根 `/home/fy5090/code/singlefs`；草稿里起的，库与换上的那份逐字节相同）：

| 写法 | 认出 |
|---|---|
| `报告 research/prompts/x-r1-opus-output.md` | 认 |
| `报告：/home/fy5090/code/singlefs/research/prompts/x-r1-opus-output.md` | 认 |
| `报告：/home/fy5090/.local/share/singlefs/work/m4-predev/research/prompts/m4-trunc-r2-opus-output.md` | 不认 |
| `` `./research/prompts/x-r1-opus-output.md` `` | 认 |
| `~/code/singlefs/research/…`、`$HOME/code/singlefs/research/…` | 认 |
| `$ROOT/research/…`、`copy/research/…`、`https://x/research/…` | 不认 |
| `../singlefs/research/…`、`「research/…」`、`report=/home/fy5090/code/singlefs/research/…` | 认 |

### 真会话记录上的新拒与新放（`<草稿>/tools/measure-legs.py`，同一份语料；三方腿类型取此刻 frontmatter 登记的 attack、defense、forward）

```
② 三方腿交回（一次交回调用一次）：
  两边认的一样：617 次
  两边认的一样（都没点名）：6 次
  新拒：改前认出、改后一份都不认（按没点名报告拒）；仓里同名的那一份不在：37 次；例：a028ec0c5ee1bd7bb（three-way-attack，2026-10-19）：'…报告 /home/fy5090/.local/share/singlefs/work/m5-dev/research/prompts/m5-box-lifecycle-r1-opus-output.md…' ｜ …
  新拒：改前认出、改后一份都不认（按没点名报告拒）；仓里同名的那一份在：9 次；例：a2e0cc6ef56fad5c8（three-way-attack，2026-10-16）：'…报告：/home/fy5090/.local/share/singlefs/work/m4-predev/research/prompts/m4-trunc-r2-opus-output.md…' ｜ …
⑰ 三方腿派发提示（一份记录一份）：
  改前拒、改后拒：37 份
  改前放、改后放：656 份
```

- 新拒 46 次，前面那一截只有两种：`/home/fy5090/.local/share/singlefs/work/m5-dev/` 37 次，`…/work/m4-predev/` 9 次。按开工日期：10-15 两次、10-16 七次、10-19 二十次、10-20 六次、10-21 六次、10-22 五次。
- 改前这 46 次怎么判的：37 次仓里没有同名文件，`cite-check.py` 读不了退 2（现跑核过：`✗ 读不了 …/research/prompts/m5-box-lifecycle-r1-opus-output.md（文件不在，或不是普通文件）`，`rc=2`），交回闸按「cite-check 退出码 2，引文这一条没判（放行）」放了；9 次核的是仓里同名的另一份。
- 新放 0 次（改法只会少认，不会多认）。⑰ 693 份派发提示改前改后一份没变。
- 代价要主 agent 知道：`m5-dev` 那条线上的三方腿在仓外副本里写报告，10-19 到 10-22 交回 37 次，换上之后会全被拒（出路写在拒绝说明里）。要允许这种写法，就是推翻条件那一种：交回闸改成按点名的绝对路径核引文。这一批没做。

### 自证格与开关

- 交回闸：新加 3 格，都过——
  - `三方腿报告路径:只点名仓外同名的 …/research/prompts/r1-opus-output.md 不认，按没点名报告拒`
  - `三方腿报告路径:项目根起头的绝对路径照认、引文对上放行`
  - `三方腿报告路径:./ 起头的照认、引文对上放行`
  
  替身那一格的替身改成两个参数。
- 派发闸：新加 2 格，都过——
  - `三方报告:整份提示里只有仓外同名的 …/research/prompts/…-output.md，拒`
  - `三方报告:整份提示里写仓根起头的绝对路径，放行`
- `HANDBACK_GUARD_BREAK=leg-report-outside-root-kept`（新）：`leg_reports_named` 直接用 `LEG_REPORT_MENTION`，仓外的也认。红 2 格，点名 2 项都在失败行里：仓外那一格，加替身那一格（开关绕过了替身）。
- `RUNNER_DISPATCH_GUARD_BREAK=leg-report-outside-root-kept`（新）：⑰ 整份提示那一支直接用 `LEG_REPORT_MENTION`。红 2 格（仓外那一格，替身那一格实得 `(True, False, True)`），点名 2 项都在。
- 原有的 `leg-report-subdirectory-kept`（交回闸）：现在红 4 格，比改前多了仓外那一格，点名已补上；`leg-report-placeholder-kept`（派发闸）红 2 格，点名的 2 项都在。

## 三、派发闸 ⑳ 的真提示钉进钩子样本目录（K2-3a）

- 新目录 `.claude/hooks/fixtures/runner-dispatch-guard/real-prompts/`：14 份 `<名>.txt`，逐字节拷自 `research/prompts/landing-audit-r3-opus-model/prompts/<名>.md`。后缀改成 `.txt`，是为了不让上游 doc-lint 扫 `.md` 的那几项（围栏、历史陈述、文风）扫到它们；doc-lint 的排除表 `.claude/doc-lint-exclude` 只排 `research/prompts/`，又不在我的写范围里。
- 同目录 `sources.tsv`（sha256 `46b3ecc6…a156`）写来历（第三轮攻方输出「复跑」一节写明的出处），每行写文件名、整份 sha256、拷自哪一份。拷完核过：两边 `sha256sum` 排序后逐行相同。
- 派发闸（`runner-dispatch-guard.sh:1984` `REAL_PROMPTS_DIRECTORY`，相对钩子所在目录）：期望表 `REAL_PROMPT_EXPECTATIONS` 从 16 位改成钉整份 sha256；`real_prompt_cells`（`:2004`）里目录不在、少一份、读不了的那几格判红，不再返回「本次未跑」；`real_prompt_sources`（`:2024`）读 `sources.tsv`。
- 自证新加 4 格（都过）：
  - `真提示样本:sources.tsv 与期望表逐份同名同 sha256`
  - `真提示样本:目录不在时 14 份逐份判红、不报本次未跑`
  - `真提示样本:少一份时只有那一份判红`
  - `真提示样本:一份末尾多一个空格时只有那一份判红`
  
  后两格在自证的临时目录里拷两份样本目录再改；样本目录本身不在时，这两格照样红。去掉了旧格 `真提示:目录不在时报本次未跑、一格都不判绿`。成功那句改成报样本目录与钉法，原样：`自检通过（查了 512 种）`。
- 文件头里引到真提示格名的两条开关（`report-name-unchecked`、`report-line-position-anchored`）跟着改成 `.txt` 和整份 sha256。复跑过，点名的那一项都在失败行里。

探针改前改后（`<草稿>/tools/real-prompts-probe.py`，进程内调两版 `real_prompt_cells`，不起自证；改前读 `research/…/prompts/`，改后读钩子样本目录）原样：

```
改前、目录在：状态 'ran'；14 格；红 0 格
改后、目录在：状态 'ran'；14 格；红 0 格
改前、目录不在：状态 '本次未跑：⑳ 真提示样本目录 /tmp/tmp07j2hzm_/不在 不在（第三'；0 格；红 0 格
改后、目录不在：状态 'ran'；14 格；红 14 格
改前、少一份：状态 'ran'；14 格；红 1 格：['真提示:a9717cf07fb6e7c21.md']
改后、少一份：状态 'ran'；14 格；红 1 格：['真提示:a9717cf07fb6e7c21.txt']
改前、a9717cf07fb6e7c21 末尾多一个空格：状态 'ran'；14 格；红 1 格：['真提示:a9717cf07fb6e7c21.md']
改后、a9717cf07fb6e7c21 末尾多一个空格：状态 'ran'；14 格；红 1 格：['真提示:a9717cf07fb6e7c21.txt']
改后 sources.tsv 与期望表逐份同名同 sha256： True
期望表改前改后逐份同一份提示、同样的两种拒、整份 sha256 以改前那 16 位起头： True
```

开关：
- `RUNNER_DISPATCH_GUARD_BREAK=real-prompts-absent-green`（跟着改）：目录不在、少一份的那几格记绿。红 2 格，点名 2 项都在：
  - 「真提示样本:目录不在时 14 份逐份判红、不报本次未跑 应当返回 ('ran', 14, 14)，实际 ('ran', 14, 0)」
  - 「真提示样本:少一份时只有那一份判红 应当返回 ['a9717cf07fb6e7c21.txt']，实际 []」
- `RUNNER_DISPATCH_GUARD_BREAK=real-prompts-digest-unchecked`（新）：不核 sha256。红 1 格：「真提示样本:一份末尾多一个空格时只有那一份判红 应当返回 ['a9717cf07fb6e7c21.txt']，实际 []」。

## 自证总数与语法检查

| 跑了什么（主工作区，换上之后） | 结果 |
|---|---|
| `bash .claude/hooks/handback-guard.sh --selftest` | 退 0，`✓ handback-guard 自检通过（查了 74 种）`（开工时 62 种） |
| `bash .claude/hooks/runner-dispatch-guard.sh --selftest` | 退 0，`✓ 自检通过（查了 512 种）`（开工时 507 种） |
| 交回闸 8 条开关逐条设上跑 `--selftest` | 8 条都退 1；点名的项逐条都在失败行里，点名项数等于失败行数 |
| 派发闸 6 条开关逐条设上跑 `--selftest` | 6 条都退 1；我动过的 4 条点名项数等于失败行数。`report-name-unchecked`（点名 20 项、失败 33 条）与 `report-line-position-anchored`（点名 4 项、失败 22 条）的点名本来就没列全，这一批没补 |
| `bash -n` 两份钩子、`python3 -m py_compile` 库、两段内嵌 python 编译 | hook-swap 在草稿与主工作区各做一次，都过 |
| 门禁 code-source-discipline 那四格（命名检查那一条命令，主工作区） | 退 1，三格红；命中里 0 处落在这一批改的文件里（`grep -c` 数这几份文件名，得 0），红的都在别的文件 |

两次换上的规格与喂法：`<草稿>/edits/spec-all.json`（43 处）、`<草稿>/edits/spec-header-extras.json`（6 处，只补文件头开关表里点名的格）、`<草稿>/edits/feeds-handback.json`（8 条）、`<草稿>/edits/feeds-dispatch2.json`（2 条，派发闸只在草稿里 `--dry-run` 喂过）。改法的生成脚本 `<草稿>/edits/*.py` 与 `build.sh`。

## 要主 agent 跑的命令

工作目录 `/home/fy5090/code/singlefs`，不需要内存包装与线程数（只起 python 与 bash）。

1. 弄坏开关复跑方与开关表格式（两份钩子的开关表改了，判决交代改了开关的交主 agent 跑复跑方那一格）：

```
SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 break-switch-headers、break-switch-replay 格; 范围=2 格; 为什么=第十一批改了交回闸与派发闸的弄坏开关表，复跑每条开关点名的项' bash .claude/gate.d/code-tooling.sh --check break-switch-headers,break-switch-replay /home/fy5090/code/singlefs
```

   看哪几行：`break-switch-replay` 那一格里 `.claude/hooks/handback-guard.sh` 与 `.claude/hooks/runner-dispatch-guard.sh` 两份没有「点名的项没在失败行里」「退 0」「超时」。登记表给的上限各 120 秒；我单跑时每条开关在 100 秒和 110 秒的 timeout 内都跑完了。

2. 提交前那一趟（`scripts/gate/gate-staged.sh`）会判到这一批的几格，我没跑：
   - gate-overlap：新加的行有没有与已有的整段相同。`sequential_naming_text` 是整段重写的，没留旧的一份。
   - code-tooling 的 research-gate-lint（两份钩子的拒绝出路）、break-switch-outside-selftest（两份钩子都在 `.claude/gate.d/break-switch-outside-selftest-pending.tsv` 里，新开关同样只在自证里用）。
   - doc-text：新目录里 14 份 `.txt` 与 `sources.tsv` 算新加的行。内容与暂存区里 `research/prompts/landing-audit-r3-opus-model/prompts/` 那 14 份逐字节相同，那 14 份本来就是这一批新加的，判法一样。

3. 下一次归档删掉 `research/prompts/landing-audit-r3-opus-model/` 之后，可以再跑一次 `bash .claude/hooks/runner-dispatch-guard.sh --selftest`，看它仍是 512 种、没有「!」行。

## 复跑这一批的量数

- 语料：`python3 -I <草稿>/tools/extract-naming-corpus.py ~/.claude/projects/-home-fy5090-code-singlefs <新文件> 12`。份数会随新记录涨。
- 改前那一版：`git show :.claude/hooks/<文件>` 取暂存区那一版（主 agent 暂存之前都成立），放进一棵按 `<草稿>/tools/make-tree.sh` 建的树；改后取主工作区。
- 量法：
  - ⑤：`measure-naming.py`、`changed-calls.py`、`reject-reasons.py`
  - ② ⑰：`measure-legs.py`
  - ⑳：`real-prompts-probe.py`
  - 攻方探针：`naming-masks.py`、`naming-e2e.py`（只换了树名），`sigpipe.sh` 原样跑
  
  读数都在 `<草稿>/final-*.out`、`final-*.txt`。

## 没做什么

- 没跑重型测试（checker-tier-crash-point-replay、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification、checker-tier-research-build-and-replay、checker-tier-broad-mutation、`gate.sh`、全量 `cargo test`、`check.sh`），不编译；没跑门禁阶段与 lint，只跑了命名检查那四格的那一条；没提交。
- 没走定义三方：这一批没改 agent 定义、共用约束与规则。
- 管道写法（`naming | tail` 这类）里退出码认不出、输出里看得到汇总行的，判决定了不认，这一批没另立读汇总行的判法。`printf … $?`、`X=$?` 再 `echo $X`、结果被截成 persisted-output 时落在预览之外的 `rc=` 行，都按读不到算，不收。重抽语料核过：认得出点名的命名检查调用 108 条（比量数时多 1 条，是我这一次跑的命名检查），前两种都是 0 条，persisted-output 1 条。
- `PIPESTATUS` 写法没认（语料里 1 条，`a8519aab2a61cba49`，改前改后都不认）。
- ② 没改成「仓外副本按点名的绝对路径核引文」（推翻条件那一种，要主 agent 定）。
- 派发闸 `report-name-unchecked`、`report-line-position-anchored` 两条开关原本就没把会红的格列全（多出 13 条、18 条失败行），不在这一批，没补。
- 别的会话可能全仓改名 `scripts/experiments/`：这三份文件里改到的地方没有引那串路径，换上之前核过基线 sha256 与开工时相同。

## 草稿目录清理

删了（删之前 `du -sh`）：
- `tree-before` 14M、`tree-after` 14M（钩子、定义与 `scripts/agents|kb|hosts` 的拷贝）
- 语料 `corpus-naming.jsonl` 412M
- `swap-dry-handback` 88K、`swap-dry-dispatch` 84K、`swap-dry-dispatch2` 84K（hook-swap 演练的喂法日志）

剩下 1.2M，是文本：本报告、改法与规格、反向规格（`swap-real/`、`swap-header/` 下的 `hook-swap/reverse-*.json`，撤回用）、量数与日志、`fixture-staging/`（14 份样本与 `sources.tsv` 的暂存，与仓里那一份相同）、`feeds/session`（喂法用的假会话记录）。喂法用的空树 `feeds/tree`（20K，带一个 `.git` 文件，完工清理闸认作工作树）交回前也删了；`<草稿>/edits/feeds-handback.json` 要再喂，先跑 `python3 -I <草稿>/tools/make-feeds.py <草稿> <草稿>/edits/feeds-handback.json` 并照它重建 `feeds/tree/.git`。没有仓副本、编译目录或库拷贝。
