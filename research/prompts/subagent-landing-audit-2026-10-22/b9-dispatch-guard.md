# 第九批：派发闸报告行按草稿目录判、开关点名对齐失败行（tooling-writer，2026-10-23）

批次：records/2026-10-22-子agent结果落地审计.md#9；依据：拆分提案第九十节 ⑳、⑲ 两行，第三轮判决 research/prompts/landing-audit-r3-main-verification.md K2 表与「用户定案」（⑳ 定「按草稿目录判」），攻方 K2-1、K2-2、K2-3 与它的模型 research/prompts/landing-audit-r3-opus-model/。

## 结论

两条的出口都达成：

| 条 | 验收标准 | 读数 |
|---|---|---|
| ⑳ | 3 份转述别人报告的真提示（自己的报告行改成 result.md）放行 | 3/3：改前各拒「报告文件名」，改后放行 |
| ⑳ | 4 份写在逗号后的真报告行照拒 | 4/4：改前放行，改后拒「报告文件名」 |
| ⑳ | 27 份「交付：报告 …-output.md」写法的三方提示放行 | 27/27：改前拒「三方腿报告路径」，改后放行 |
| ⑳ | 14 种骨架填好放行；报告行填名字带 report 的文件照拒 | 14/14 放行；报告行换成草稿目录下的 final-report.md 13/14 拒，三方材料员那一种骨架没有草稿目录一行，见第 1 节 |
| ⑳ | 自证加格；新开关退回今天的锚法，自证红在新加的格 | 自证 460 → 471 种；`report-line-position-anchored` 下红的 8 格全是新加的格 |
| ⑲ | 19 条点名改成与打开开关后真失败的格名逐个相同（只改点名，不改判法） | 原样 19 条对不上；改后 18 条点名与失败格逐个相同，disable-check 点名 70 格都在它失败的 299 格里（第 2 节） |
| ⑲ | 照 break-switch-replay 格的读法逐条复跑，全对上 | 96 条（派发闸自己的 89 条，加 lib_long_run 5 条、lib_work_root 2 条）全记「对」；原样是「对」15、「没点名」80 |

推翻条件：主工作区 `bash .claude/hooks/runner-dispatch-guard.sh --selftest` 不绿；`RUNNER_DISPATCH_GUARD_BREAK=report-line-position-anchored` 下自证退 0；照 code-tooling break-switch-replay 格的读法复跑派发闸，有一条不是「对」；`--check-prompt` 对上面四组提示判得和表里不一样。

## 改过的文件

只改了 `.claude/hooks/runner-dispatch-guard.sh`。`scripts/agents/dispatch-prompt.py` 没改（它的自证与 `--check` 照原样跑绿，sha256 774dd0b9…，和开工时一样）。

`git diff --stat -- .claude/hooks/runner-dispatch-guard.sh scripts/agents/dispatch-prompt.py` 原样（比的是暂存区，暂存区里有上一批的改动）：

```
 .claude/hooks/runner-dispatch-guard.sh | 553 +++++++++++++++++----------------
 1 file changed, 291 insertions(+), 262 deletions(-)
```

只算这一份的改动（开工时的原样 `orig/runner-dispatch-guard.sh` 对现在，`git diff --no-index --stat` 末行）：

```
 1 file changed, 458 insertions(+), 301 deletions(-)
```

sha256：开工时 9b5d3d00f12e37594015e3f8f1f0f9ce9de3d80b5631e7d1c0063a3362d44e1c（和第七批交回报告记的一样），交回时 b3db189f74336fb4b626977c89a4970915f8fe110d4a6bb83fbb16bc2c6002ec。

换法：经 `scripts/gate/hook-swap.py` 换上四次，每次退 0。规格由 `tools/make-spec.py` 从草稿全文和换上之前那一版逐段对出（每段的旧串在原文里、新串在改后全文里各只出现一次；在内存里正着施加要得到草稿，倒着施加要回到原文）。每次都是草稿里先验（语法、内嵌 python 编译、喂 8 条），换上之后在主工作区再验，都对。四次依次是：判法加自证加新开关（16 处）；一格样本补草稿目录一行（1 处，见第 1 节末）；开关表点名改写（3 处）；两条开关点名补全（2 处）。草稿树里起不了改过的脚本的自证（重型闸拒「副本里的脚本与主仓那一份内容不同」），所以自证都在每次换上之后在主工作区跑；草稿里用的是 hook-swap 喂法、`--check-prompt`，再加上把内嵌 python 当模块载入、在真提示上比判定（不跑自证）。

## 1. ⑳ 报告行按草稿目录判

### 改了什么（行号是交回时这一版）

- ④ 写范围不动：照旧只认行首、句首的报告标签（`report_line_mentions(prompt)`，`anywhere` 默认为假），派发里没让改它。攻方模型的计数里，不锚认法会让写范围多拒 28 份（`today_vs_unanchored:scope` 那一行），所以 ④ 不跟着放宽。
- ⑰ 的两判（文件名带 report、三方腿形态）改成 `report_line_mentions(prompt, anywhere=True)`：标签前面不连着字的，在哪都认（逗号后、「交付：」后也认）；「」『』“”与反引号里的不认（第 1312 行 `quoted_spans`，按行配对，「」可以套）。
- 「本提示的草稿目录」（第 1297 行 `REPORT_SCRATCH_MENTION`、第 1345 行 `report_scratch_directories`）：「草稿目录」标签前面不连着字、不在引号里，冒号可有可无，路径可以带反引号，不限行首，后面跟工作根或过渡期旧根底下的目录（根本身不算）。行首加冒号这一种不够用：K2-2 那 4 份里的工具员 a9717cf07fb6e7c21 写的是「…。草稿目录 `~/…/tooling-two-stage-followup/`（自己建，排他），报告 `…/report.md`」。
- ⑰ 文件名（第 1400 行）：只判 `own_report_token`（第 1359 行）认的词，也就是没有目录的裸文件名、落在本提示草稿目录下的路径（「<草稿目录>/…」写法也算在草稿目录下）；别的目录下的不判。
- 三方形态（第 1434 行）：报告行点名的路径合 `research/prompts/<名>-output.md` 就算有；不合的，落在本提示草稿目录下或是裸文件名的拒，别的目录下的当转述、不判。报告行里一处这一形态都没有时，再看整份提示里不在「」『』“”与反引号里的 `research/prompts/<名>-output.md`（第 1432 行 `THREE_WAY_REPORT_MENTION`，`<轮>` 这类形态写法不认）。一处都没有就拒，拒绝说明列出被当成转述的报告行。
- 新弄坏开关 `RUNNER_DISPATCH_GUARD_BREAK=report-line-position-anchored`（文件头第 200 行）：⑰ 两判退回第七批的认法，也就是只认行首句首、点名的路径都判、三方形态不看报告行之外。
- `report-line-unanchored` 开关的说明改成「④ ⑰ 都退回句中也认、引号里的也认」；文件头 ④、⑰ 两条，⑰ 的拒绝说明，自证的成功句同步改了。

### 自证格（460 → 471 种）

删掉一格：「报告名:句中逗号之后的报告标签不认」（应得 0）。新判法下这种写法要拒，它就是 K2-2。

新加 12 格：
- 报告名:句中逗号之后、落在本提示草稿目录下的报告标签照认（草稿目录写在句中、反引号里），应得 2（照 a9717cf07fb6e7c21 的写法）
- 报告名:句中逗号之后、落在本提示草稿目录下的「报告写到」照认，应得 2（照三个设计员的写法）
- 报告名:句首转述别的草稿目录下的 report.md 不判，应得 0（照 K2-1 那 3 处的写法）
- 报告名:没写草稿目录一行时，带目录的路径不判，应得 0
- 报告名:不带目录的裸文件名照判，应得 2
- 报告名:「<草稿目录>/report.md」写法照判，应得 2
- 报告名:三方腿报告行写 research/prompts/<名>-output.md 不在草稿目录下，名字带 report 也不判，应得 0
- 三方报告:草稿目录下的 result.md 写在逗号之后照拒，应得 2
- 三方报告:「交付：报告 …-output.md」写法放行，应得 0（K2-3）
- 三方报告:报告行之外不在引号里的 research/prompts/<名>-output.md 也认放行，应得 0
- 三方报告:只在引号或反引号里出现的 -output.md 不算，拒，应得 2
- 三方报告:转述别的草稿目录下的报告不当自己的报告行，应得 0

原有格里，路径写在 `~/.local/share/singlefs/work/w/` 下、本来应当拒的 10 格，样本前面补一行「草稿目录 ~/.local/share/singlefs/work/w/」：`report_name_head` 和七格直接写的样本。草稿目录一行不写冒号，这样 stdin 那几格不会在真工作根里建写者锁。另有「三方报告:攻方腿/正推腿报告写草稿目录下的 result.md 拒」两格和「三方报告:拒绝说明给出形态与骨架」的样本也补了草稿目录一行，「目录名带 report、文件名不带放行」补了 `reports/` 那一行。最后这一格是第二次换上补的：第一次换上之后，`report-name-whole-path` 开关打开也不红（复跑记「没红」），原因是那一格的样本没有草稿目录一行，路径不判。补上之后它红在这一格，跟改前一样。

### 该红的输入：新开关打开后自证的失败行（主工作区、交回时这一版，退 1，原样）

```
✗ 自检：报告名:句中逗号之后、落在本提示草稿目录下的报告标签照认（草稿目录写在句中、反引号里） 应当返回 2，实际 0
✗ 自检：报告名:句中逗号之后、落在本提示草稿目录下的「报告写到」照认 应当返回 2，实际 0
✗ 自检：报告名:句首转述别的草稿目录下的 report.md 不判 应当返回 0，实际 2
✗ 自检：报告名:没写草稿目录一行时，带目录的路径不判 应当返回 0，实际 2
✗ 自检：报告名:三方腿报告行写 research/prompts/<名>-output.md 不在草稿目录下，名字带 report 也不判 应当返回 0，实际 2
✗ 自检：三方报告:「交付：报告 …-output.md」写法放行 应当返回 0，实际 2
✗ 自检：三方报告:报告行之外不在引号里的 research/prompts/<名>-output.md 也认放行 应当返回 0，实际 2
✗ 自检：三方报告:转述别的草稿目录下的报告不当自己的报告行 应当返回 0，实际 2
```

### 喂法（hook-swap，stdin 入口，`feeds-swap1.json` 8 条；改前那一版是 `--feed-only` 喂开工时的钩子）

改前（`logs/feeds-before-swap1.txt` 原样）：

```
  ✓ 退 0（应当 0） clerk 报告行写草稿目录下的 result.md 放行 ← 
  ✓ 退 2（应当 2） clerk 报告行写草稿目录下的 report.md 拒 ← 
  ✓ 退 2（应当 2） clerk 报告行写裸文件名 report.md 拒 ← 
  ✓ 退 0（应当 0） 攻方腿报告写 research/prompts/x-r1-opus-output.md 放行 ← 
  ✓ 退 2（应当 2） 攻方腿报告写草稿目录下的 result.md 拒 ← 
  ✗ 退 0（应当 2） clerk 逗号之后、落在本提示草稿目录下的 report.md 拒（新判法） ← 
  ✗ 退 2（应当 0） clerk 句首转述别的草稿目录下的 report.md 不判（新判法） ← 
  ✗ 退 2（应当 0） 攻方腿「交付：报告 …-output.md」写法放行（新判法） ← 
✗ 只喂不换：.claude/hooks/runner-dispatch-guard.sh 喂了 8 条，3 条不对
```

改后：四次换上，草稿里和主工作区各喂一遍，8 条全对。末一次的输出在 `hook-swap/feeds-20261022T221959Z-41896.log`，16 行「退 X（应当 X）」，没有一行不对。

### 出口核对（`tools/accept.py`）

两份派发闸都用 `--check-prompt <提示> --type <类型> --root <仓根>` 起，只看「报告文件名」「三方腿报告路径」两种拒。改前是开工时那一份，放在符号链接搭的草稿树里，库和主工作区共用；改后是主工作区交回时的这一份。提示原样拷在 `prompts/` 下：攻方模型那 14 份，加上我照攻方的做法从 a28fe10b58d88beec 改出来的 `-own-result` 一份（只把它自己的报告行 `…/report.md` 改成 `…/result.md`）；那 27 份三方提示从会话记录里取首条提示，放在 `prompts/k23/`。原样输出在 `logs/accept-final.txt`，下面是逐组读数：

```
== K2-1 转述别人报告的 3 份（自己的报告行改成 result.md）（3 份，改后应得 不报报告类的拒）
  ✓ aa9ec3423c1f54bba-own-result（implementation-writer） 改前 ['报告文件名']  改后 放行
  ✓ a4737c6f3660f4f7e-own-result（experiment-runner） 改前 ['报告文件名']  改后 放行
  ✓ a28fe10b58d88beec-own-result（kb-scribe） 改前 ['报告文件名']  改后 放行
  小计：3/3 对
== K2-2 写在逗号后的真报告行 4 份（4 份，改后应得 ['报告文件名']）
  ✓ a0a2dfee322e26786（experiment-designer） 改前 放行  改后 ['报告文件名']
  ✓ a789e7091d91430aa（experiment-designer） 改前 放行  改后 ['报告文件名']
  ✓ a8afb2185cfa840cc（experiment-designer） 改前 放行  改后 ['报告文件名']
  ✓ a9717cf07fb6e7c21（tooling-writer） 改前 放行  改后 ['报告文件名']
  小计：4/4 对
== K2-3「交付：报告 …-output.md」写法的三方提示（27 份，改后应得 不报报告类的拒）
  （27 行都是「改前 ['三方腿报告路径']  改后 放行」）
  小计：27/27 对
== 回归：第二轮两份真提示与攻方 K2-3 的两份最小复现（4 份，改后应得 不报报告类的拒）
  ✓ a3652270eb80e4fd0（three-way-attack） 改前 放行  改后 放行
  ✓ a303d33b0ccd5e68f（tooling-writer） 改前 放行  改后 放行
  ✓ a3652270eb80e4fd0-delivery-colon（three-way-attack） 改前 ['三方腿报告路径']  改后 放行
  ✓ a3652270eb80e4fd0-section-colon（three-way-attack） 改前 ['三方腿报告路径']  改后 放行
  小计：4/4 对
== 对照：转述那 3 份的原样（自己的报告行也写 report.md）（3 份，改后应得 ['报告文件名']）
  ✓ aa9ec3423c1f54bba（implementation-writer） 改前 ['报告文件名']  改后 ['报告文件名']
  ✓ a4737c6f3660f4f7e（experiment-runner） 改前 ['报告文件名']  改后 ['报告文件名']
  ✓ a28fe10b58d88beec（kb-scribe） 改前 ['报告文件名']  改后 ['报告文件名']
  小计：3/3 对
出口核对：五组都对
```

那 27 份是怎么取的：`tools/corpus.py` 拿改前改后两份在全部首条提示上比，三方形态改前拒、改后放行的 52 份里，用第五批的不锚认法（把模块里的 `REPORT_LINE` 换成 `REPORT_LINE_UNANCHORED`，也就是攻方 K2-3 计数 `today_vs_unanchored:form:today=True:other=False 27` 的那种认法）也放行的，正好 27 份（`logs/form-flips-final.txt` 末行「份数 52 其中第五批不锚放行 27」）。日期分布和攻方报告的一样：2026-09-25 一份、10-03 八份、10-05 十份、10-08 六份、10-09 一份、10-11 一份。

### 14 种骨架（`tools/skeletons.py`，原样在 `logs/skeletons-final.txt`）

每种用 `python3 scripts/agents/dispatch-prompt.py <类型> --set 草稿目录=<草稿目录>/skel/<类型> --set 认领用途=出口核对 --batch records/x.md#1` 打骨架，再填占位：预计挂钟填 2 小时内；删掉分片、续跑、多机三行和「上一段岔路表里还差」；三方腿的报告填 `research/prompts/x-r1-opus-output.md`；变异分诊员的表填 `crates/mutations.tsv`；别的占位填「已填」。末行原样：

```
种类 14；填好放行 14；报告行换成草稿目录下的 final-report.md 被「报告文件名」拒 13；三方腿换成 research/prompts/x-r1-opus-report-output.md 改后放行 3/3
```

- 填好：14 种改前改后都退 0、放行。
- 报告行换成 `<草稿目录>/skel/<类型>/final-report.md`：13 种改后拒「报告文件名」，三方三种另拒「三方腿报告路径」。三方材料员（three-way-materials）的骨架既没有「草稿目录：」也没有「报告：」一行，加上的那一行路径不在任何认得的草稿目录下，所以不判，改后放行。另外核了两种写法：报告行写裸文件名 `final-report.md`，改后拒「报告文件名」；补一行草稿目录、报告写它下面的 final-report.md，改后也拒「报告文件名」。
- 三方三种的报告行换成 `research/prompts/x-r1-opus-report-output.md`（第七批交回用的写法）：改前拒「报告文件名」，改后放行。这个路径不在草稿目录下，照用户定的「按草稿目录判」，⑰ 不判它；自证里有一格钉着这个结局（报告名:三方腿报告行写 research/prompts/<名>-output.md 不在草稿目录下，名字带 report 也不判）。三方腿的报告照共用约束是用 Bash 分段写的，工具层只拦 Write。要不要对三方腿报告形态的路径也判名字，归主 agent 定（第 4 节）。

### 全部真提示上两版比（`tools/corpus.py`，攻方模型 probes/report_line_corpus.py 的载入法：把内嵌 python 当模块载入，不跑 main 和自证）

本项目全部子 agent 会话记录的首条派发提示，`logs/corpus-final.txt` 原样：

```
prompts 3317
form:old=拒:new=放 52
name:old=拒:new=放 89
name:old=放:new=拒 50
```

④ 写范围一份都没变。逐份明细在 `logs/corpus-final-diff.jsonl`，分类在 `logs/form-flips-final.txt`、`logs/name-flips-final.txt`：

- 三方形态，改前拒、改后放的 52 份：27 份是 K2-3 的写法；14 份是报告写在工作树里（`~/.local/share/singlefs/work/m5-dev/research/prompts/…-output.md` 这种绝对路径），改前因为去不掉仓根前缀判「不是这一形态」，改后路径不在草稿目录下、不判，又在整份提示里认到 `research/prompts/…-output.md`；6 份（2026-10-21 m4-i315 两轮）报告行另点名了 `scripts/agents/cite-check.py`，改后当别的目录、不判；5 份报告行里一处都没有，是在整份提示里认到的（见下面「弱处」）。改前放、改后拒的 0 份。
- 文件名，改前拒、改后放的 89 份：79 份是提示里认不到草稿目录（没写「草稿目录」，或写成「在草稿目录 `/tmp/…`」这类前面连着字的），10 份是报告写在草稿目录的上一层或旁边（草稿目录 `…/kb-writeback-batch3/scribe`、报告 `…/kb-writeback-batch3/scribe-report.md` 这一类）。89 份的开工日期都在 2026-10-14 以前，那以后派的提示骨架都带「草稿目录：」一行。
- 文件名，改前放、改后拒的 50 份：41 份报告在本提示草稿目录下（逗号后、句中），9 份是裸文件名 `report.md`。按会话记录首条时间戳的日期算，2026-10-14 起的只有 3 份，就是 K2-2 那三个设计员；a9717cf07fb6e7c21 的首条时间戳日期落在 10-13，算进了更早的那 47 份。

弱处（推的，只在这份语料上数过，没被攻过）：
- 三方形态「整份提示里也认」照规格写的，不分是哪一行。三方腿的禁读清单常列另一条腿的 `-output.md`，所以只靠禁读清单那一行也能放行。上面那 5 份里，a2e0cc6ef56fad5c8、a5cd227cc10a22b61、aaa9106f04bbc1570 三份就只认到禁读清单里另一条腿的报告：第一份自己的报告写在反引号里，后两份的报告路径写在另一份 prompt.md 里，提示里没有。另两份 a020bdf3f372fe939、afc7bfa5621fc02fd 认到的有它们自己的「交付：报告写 research/prompts/…-opus-output.md」，这一句的标签是「报告写」，不是四种之一。没改：派发没这一条。要堵的话，「整份提示里认」那一步可以不认禁读清单那一行。
- 提示里认不到草稿目录时，⑰ 只判裸文件名。这一条是照规格（「别的目录下的路径不判」）写的；放过的那 79 份都在骨架普及之前。

## 2. ⑲ 派发闸开关表的点名对齐失败行

### 读法（`tools/replay-emulate.py`）

读法从 `.claude/gate.d/code-tooling.sh` 现取，不另写：开关表的读法 `BREAK_SWITCH_TABLE_LIBRARY`（`switch_entries`、`incomplete`），格函数 `cell_break_switch_replay` 里的 `quoted_items`、`appears`，`attempt` 的判法照抄。每条开关设上之后（其余名字以 `_BREAK` 结尾的变量摘掉，`SINGLEFS_HEAVY_TESTS` 也摘掉），在仓根起 `bash .claude/hooks/runner-dispatch-guard.sh --selftest`。失败行是输出里带 ✗ 的行，点名的项是开关行下面那几行里「」括着的原文，每一项要在某一行失败行里出现。结局分对、守卫、没红、没点名、不符、超时几种；不是「对」的先重跑一次再定；并行 8，单条上限 200 秒（门禁那一格按登记表里的秒数）。开关表里一共 96 条：派发闸自己的 89 条（第七批 88 条，加这一批新加的 1 条），另有 `LIB_LONG_RUN_BREAK` 5 条、`SINGLEFS_WORK_ROOT_BREAK` 2 条，这 7 条的红格也在派发闸的自证里。格读整份开关表，所以 96 条都要对上。

### 原样的读数

开工时那一版（`logs/replay-orig.jsonl`）：

```
条数 95，头注没写全 0，官方读法结局 {'对': 15, '没点名': 80}
```

原来的写法是「<项名>（应当返回 2，实际 0）；…」，没有「」，所以照格的读法 80 条无从比。记「对」的 15 条，是项名里刚好带了「」，比的只是「」里那一小段。

点名对不上失败行的条（`tools/header-audit.py`：在点名的字里逐字找真失败的格名，剩下的字去掉「（应当返回…，实际…）」和标点之后还有字，或者点名了没红的格，就算对不上）。原样 19 条（`logs/header-audit-orig.txt` 末行「对不上的条数 19 / 95」）：

disable-check、heavy-permission-unchecked、report-name-unchecked、valve-reason-ignored、long-run-unchecked、wall-clock-optional、long-run-types-narrowed、snapshot-abandoned-ignored、snapshot-abandoned-quote-short-ok、code-fence-loose、code-block-line-rule、thread-required-six-types、thread-limit-unchecked、thread-limit-any-refused、thread-valve-ignored、thread-valve-loose、thread-formula-accepted、thread-line-optional、thread-required-only-writer。

对不上的写法有四种：「<九种类型各一>」「<同八种各一>」这类模板；「新加的拒绝格同样红」「开关 heavy-permission-unchecked 下红的那十格同样红」这类概括；点名了不红的格（heavy-permission-unchecked 的「stdin:正文许可跑 checker 档拒绝（退出码 2）」、valve-reason-ignored 的「stdin:快照清单读不了时 stderr 写了是哪一份」、snapshot-abandoned-quote-short-ok 的「快照:不合格的拒绝说明写了没收的原因（…）」）；括注里夹着说明句（report-name-unchecked 的「两条 stdin 的 stderr 格应当返回 1，实际 0」、snapshot-abandoned-ignored 那一条）。第七批交回列名单的时候用的是另一种粗读法，名字和这里不完全一样（那边写「snapshot-abandoned 三条」，没列 disable-check、report-name-unchecked），条数一样是 19。

### 改了什么（只改点名，不改判法）

- 96 条的点名一律改成「<项名> 应当返回 <应得>，实际 <实得>」，也就是打开开关后自证那一行失败行「✗ 自检：」后面的原文，项之间用「；」隔开。开关表前面那句写法说明（第 133 行起两行）同步改了。
- 19 条里有 18 条，点名改成打开开关后失败的全部格，次序照失败行，和失败格逐个相同。disable-check 一进来就放行，失败 299 格，没有全列：原来逐字点名的 60 格照留，「那十格同样红」那一句换成 heavy-permission-unchecked 下红的 10 格的名字，一共 70 格，都在它失败的格里。
- 别的 77 条：原来逐字点名的格照留，只改写法。其中 report-line-labels-split 删了 3 格，「三方报告:报告路径写成仓根下的绝对路径放行」等三格，它们在这一批之后不红了（三方形态改成也认报告行之外，「报告路径：…-output.md」那几份提示在整份提示里认到了）。这是第 1 节改判法带出来的，不是原来点名写错。
- 生成与核对：`tools/rewrite-header.py`、`tools/full-entries.py` 按复跑出来的失败行写；每条写完先拿格的 `quoted_items` 读回来，要和写进去的项逐个相同，才换上。

### 改后的读数（主工作区、交回时这一版，`logs/replay-final2.jsonl`、`logs/final-check2.txt`）

```
条数 96，头注没写全 0，官方读法结局 {'对': 96}
条数 96；「」里每项与一行失败行整句相同 96 条；点名的格都在失败的格里 96 条；原样对不上的 19 条里点名与失败逐个相同 18 条
```

没对上的那一条就是 disable-check：「点名 70 格、失败 299 格」，原因见上一节。

每条要跑多久（`tools/timing.py`，并行 8，同一时间别的会话在跑 cargo test，机器负载 50 到 90，`logs/timing.txt` 原样）：

```
条数 96，并行 8；挂钟秒数 最短 8.9 中位 23.2 最长 53.5；退非 0 的 96 条
```

登记表规定并行下最慢的一条要在上限的一半以内。照这个负载，派发闸那一行的秒数写 120 才够（53.5 不到 60）。单跑一次自证不到 4 秒。复跑表里加行归兄弟任务，`.claude/gate.d/` 我没碰。

## 3. 自证读数（主工作区、交回时这一版）

两份都登记为 light（`.claude/hooks/selftest-registry.tsv`），我自己跑的：

| 命令 | 结果 |
|---|---|
| `bash .claude/hooks/runner-dispatch-guard.sh --selftest` | 退 0，「✓ 自检通过（查了 471 种）」（开工时 460 种：删 1 格，加 12 格） |
| `python3 scripts/agents/dispatch-prompt.py --selftest` | 退 0，「✓ 自检通过（查了 90 项）」（没改，派发闸换上之后复跑；它的 --check 那几格经派发闸 --check-prompt 起） |
| 96 条开关逐条打开（照 break-switch-replay 格的读法） | 96 条都退 1，都记「对」（第 2 节） |
| `RUNNER_DISPATCH_GUARD_BREAK=report-line-position-anchored` | 退 1，红 8 格，都是新加的格（第 1 节） |

语法：hook-swap 每次在草稿里和主工作区都跑了 `bash -n`，内嵌 python 也编译了，都过。没改 `.rs`，所以没跑命名检查。

## 4. 交主 agent 定的、要主 agent 跑的

1. 复跑表加派发闸一行，归兄弟任务（`.claude/gate.d/break-switch-replay-files.tsv`）：`.claude/hooks/runner-dispatch-guard.sh<TAB>120<TAB># …`。秒数照第 2 节的读数取。加上之后，门禁 code-tooling 的 break-switch-replay 格会复跑这 96 条。
2. 提交前（归门禁，我没跑）：code-tooling 的 break-switch-headers、break-switch-replay 两格，gate-overlap、hooks-registered。这一次没新建钩子，也没改 `# hook-events:`、`# gate-similar:` 两行。
3. 规格没覆盖、推的三处，要不要堵由主 agent 定，都没改：
   - 三方形态「整份提示里也认」，会把禁读清单里另一条腿的 `-output.md` 也认进来（3 份真提示只靠这一处放行，见第 1 节「弱处」）。可以在那一步不认「禁读清单」那一行。
   - 三方腿报告行写 `research/prompts/<名含 report>-output.md`，⑰ 不判了（不在草稿目录下）。第七批的骨架核对里，这一种 3/3 是拒的。
   - 提示里没有草稿目录的，⑰ 只判裸文件名。10-14 以前的 79 份真提示因此不拒了；10-14 以后的骨架都带「草稿目录：」一行。
4. `scripts/agents/dispatch-prompt.py` 文件头第 36 到 37 行还写着「派发闸 ⑰ 对这三种另判形态：报告行点名的文件不是 research/prompts/<名>-output.md（例如草稿目录下的 result.md）或没有报告行就拒」。现在派发闸认的是「一处这一形态都没有就拒」，报告行之外的也认。这只是文字，自证不读它，派发没让改，我没碰。

撤回：反向规格在 `hook-swap/` 下。`reverse-20261022T214834Z-273069.json` 是 `--dry-run` 那一次写的，没换上，不用它。其余四份按时间倒着跑，依次撤第四、三、二、一次换上：`python3 scripts/gate/hook-swap.py <反向规格> --hook .claude/hooks/runner-dispatch-guard.sh --scratch <草稿目录> --feeds <草稿目录>/feeds-neutral.json`。也可以一次撤回：拿 `orig/runner-dispatch-guard.sh` 当草稿，用 `tools/make-spec.py` 对着此刻那一版出规格，再照 hook-swap 换上。

## 5. 草稿目录里留着的

`/home/fy5090/.local/share/singlefs/work/landing-audit-b9-dispatch-guard/` 下：

- `orig/`：开工时两份的拷贝，加 `orig.sha256`。
- `draft/`：每次换上前后的全文（`swap1.sh`、`swap2.sh`、`swap3.sh` 与交回时这一版 `runner-dispatch-guard.sh`、`header-new.sh`）。
- `spec-swap1.json` 到 `spec-swap4.json`、`feeds-swap1.json`、`feeds-neutral.json`、`hook-swap/`（喂法输出与反向规格）。
- `prompts/`：攻方模型那 14 份真提示，加我改出的 a28fe10b58d88beec-own-result.md；`prompts/k23/` 是那 27 份三方提示。
- `skel/`：14 种骨架，填好的和换了报告名的。
- `tools/`：这一份写的探针和生成脚本。
- `logs/`。

用来比改前的那棵符号链接草稿树 `tree-orig/` 和草稿用的 `tree/` 都已删掉。没有仓副本，也没有编译目录。

## 没做什么

- 没跑重型测试（checker-tier-crash-point-replay、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification、checker-tier-research-build-and-replay 本身、checker-tier-broad-mutation 与 `scripts/mutation/broad-mutation.py` 起 cargo 的用法、`gate.sh`、全量 `cargo test`、`check.sh`）；没走定义三方；没提交。
- 没跑门禁阶段与各类 lint；没改 `.rs`，没跑命名检查。
- 没碰 `.claude/gate.d/`（复跑表那一行归兄弟任务）、`.claude/hooks/handback-guard.sh`、`.claude/hooks/bash-command-detector.sh`、`scripts/kb/`、`scripts/agents/agent-cost.py`、`scripts/agents/agent-watch.py`、`.claude/agents/`、`.claude/agent-common.md`、`.claude/main-agent.md`、`records/`、`research/`；`scripts/agents/dispatch-prompt.py` 没改。
- 第 4 节第 3 条那三处推的改法没做，派发里没有。
- 跨行的「」引文没处理：引号按行配对，一行里只有开头的「、没有配对的」，这一行后面的字不当引号里的。上一批交回记的这一条照旧。
- 改后的 `--check-prompt` 只核了第 1 节列的几组提示与 14 种骨架；全部 3317 份真提示只比了 ⑰ 两判与 ④ 三样判定，没有逐份起整条判法。
- disable-check 那一条点名没有全列它失败的 299 格（第 2 节）。
- 没回原会话记录逐份核 K2-1 那 3 处是不是转述（照攻方的判读和判决节「五、没核动的」）。
