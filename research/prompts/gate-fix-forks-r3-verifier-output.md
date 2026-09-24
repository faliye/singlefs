# gate-fix-forks-r3 核查员报告

核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

**接续说明**：上一个核查员撞会话限额，报告一行未落盘；交接摘要
`/tmp/claude-1000/gate-fix-forks-r3-verifier/handover.md`。本报告第一步核了它留下的现场
（`opus-rerun/`、`selftest/`、`opus-model-sha.txt`），已跑完的复跑（t1-scenes、t5-states、
t5-config-amend、t5-64、t6-mutants、t6-fixture-v2、t7-invocations 七道，均 `DIFF: IDENTICAL`）
不重做，本报告接着把剩下的复跑（t1-titles.py、t1-mention-forms.py、t5-moved-pages.py、
t-fix.out 的 diff 核对）做完，再核三条腿的引用与产物。

## 一、判别力自证（沿用上一个核查员已做的结果，原样抄）

方法：把 opus 报告引用的 `.claude/gate.d/75-decision-experiment-links.sh:17` 的行号改成 18（副本，
`/tmp/claude-1000/gate-fix-forks-r3-verifier/selftest/75-decision-experiment-links.sh`），
用同样的比对方法核这一条引用是否与副本第 18 行逐字相同。

```
引用原文（opus 报告第 79 行给出）：
#      它指的决策文件要在这次改动里。这次改动把实验的标题状态改成「已跑」（状态段恰好是这两个字）时，
--- 副本第 17+1=18 行原样 ---
#      决策正文（历史版本之前）用 `E<号>（` 引了它的每条决策都要回看：决策文件在这次改动里，或者表里那条决策的一行改过。
--- 逐字比对 ---
判定：不一致（✗）—— 方法能分辨
```

方法能分辨错的行号，判别力自证通过。

## 二、开工核对

- 三份腿报告 sha256（本会话现算，与派发提示给的对比）：

```
$ sha256sum research/prompts/gate-fix-forks-r3-opus-output.md research/prompts/gate-fix-forks-r3-sonnet-output.md
1612af0ba9c39cff13119360479fd30383526c42895322b4f302133f8066f887  research/prompts/gate-fix-forks-r3-opus-output.md
a7240a19e7da04e14c2e4e519df9adbc91fc49c231d61529aa43f6c6a37f92b3  research/prompts/gate-fix-forks-r3-sonnet-output.md
```

两份都与派发提示给的 sha256 逐字相符，判「原样」，不按「改过」记。

- 快照清单 105 份，对快照原样树 `/tmp/claude-1000/gate-fix-forks-r3-snapshot-tree/` 之前已核对全 OK
  （沿用上一个核查员的记录，未重跑——`sha256sum -c` 是纯读操作，清单与树都没变过，不重复跑）。
- 快照清单里没有 `research/prompts/*.md`（实验页、决策文件的正文语料）、`.claude/kb/experiments.md`、
  `.claude/kb/experiments/` 单页、`.claude/singlefs-ai-sop/scripts/doc-lint.sh`：这几类文件是本报告下文
  多处「核不动」的共同原因，先在此处登记，下文不重复解释。

## 三、云端攻方（Opus）腿

### 3.1 模型 sha256（22 份文件）

`sha256sum -c` 对 `research/prompts/gate-fix-forks-r3-opus-model/` 下 22 份文件全部 `OK`（上一个核查员已核，本报告复核一次，结果相同，命令与输出见 `research/prompts/gate-fix-forks-r3-verifier-rerun/opus-model-sha.txt` 与本会话重跑记录）。

### 3.2 复跑 12 条命令

| 命令 | 结果 | 说明 |
|---|---|---|
| `t1-scenes.sh $S . $R/t1` | ✓ DIFF: IDENTICAL | |
| `t5-states.sh $S . $R/t5s` | ✓ DIFF: IDENTICAL | |
| `t5-config-amend.sh $S . $R/t5c` | ✓ DIFF: IDENTICAL | |
| `t5-64.sh $S $R/t5-64` | ✓ DIFF: IDENTICAL | |
| `t6-mutants.sh $S . $R/t6` | ✓ DIFF: IDENTICAL | |
| `t6-fixture-v2.sh $S . $R/t6v2` | ✓ DIFF: IDENTICAL | |
| `t7-invocations.sh $S . $R/t7` | ✓ DIFF: IDENTICAL | |
| `t-fix.sh $S . $R/fix` | ✓ 123 行，仅 1 行（草稿目录路径本身）不同，其余逐字相同（含「2 个样本判错（判对 14 个，8 个阶段没有样本）」这句关键结论） | 路径按派发指示换成本报告草稿目录，属预期差异，不判 ✗ |
| `t1-titles.py .`（按指示把仓根换成快照树） | 核不动：快照树里没有 `research/prompts/` 下的实验页 | 重跑输出「实验页 0 份」，与快照树本就不含该语料一致 |
| `t1-mention-forms.py .`（同上换仓） | 核不动：快照树里没有 `.claude/kb/decisions*` | 重跑输出「决策 0 份」（原产物「决策 28 份」），二者「非标准写法 0 处」的结论一致，只是分母来源不同 |
| `t5-moved-pages.py .`（同上换仓） | 核不动：快照树里没有 `.claude/kb/experiments.md` | 重跑直接 `FileNotFoundError` 退出 1（见下方 3.3 的另一项发现） |

命令与原样输出见 `research/prompts/gate-fix-forks-r3-verifier-rerun/`（`t1-scenes.out` 等 8 个 `.out` 与对应 `diff` 结果、`t1-mention-forms.out`、`t5-moved-pages.out` 的新跑结果）。

### 3.3 产物与正文数字核对——发现一处不符（✗）

`t5-moved-pages.out` 是 opus 腿在**真仓**（非快照树）上跑出的原始产物，随模型一起交回、sha256 已核对（`OK`）。opus 报告正文三处（表格 T5-a 行、`## T5-a` 小节、`## 没打中的形状`）引用这份产物的数字，说「0 行对不上」、「站住」。逐字核对产物文件本身：

```
$ cat research/prompts/gate-fix-forks-r3-opus-model/t5-moved-pages.out
88：204 份正文里整行抄的 E7RESULT 行 612 行；树里、归档、HEAD 删掉的产物里都找不到的 1 行，分在 1 份：
  .claude/kb/experiments/153-账本形态与环上有洞的代价.md：1 行，例 第 85 行 E7RESULT name=q_family family=H4d2 s=8 placement=Impl period=1 arm=interval_narrow publishes=146 q1_
40：实验正文点名的产物 255 份；树里、HEAD、git 历史里都没有的 0 份，分在 0 页：
```

正文三处的原话（`grep -n`）：

```
67:| T5-a | `--no-renames` 让搬家整页算新增 | **站住** | 真仓 204 份正文 616 行 E7RESULT、256 份被点名产物，搬家后多判的行里对不上的 0 行 | — | — |
161:…`t5-moved-pages.out`：照 88 号的取法，204 份正文里整行抄的 E7RESULT 616 行，在树里…都找不到的 0 行；照 40 号的取法，实验正文点名的产物 256 份，树里…都没有的 0 份。…
294:- T5：搬家后 88、40 号多判的旧行——真仓 616 行、256 份产物里 0 行对不上（`t5-moved-pages.out`）。…
```

三处一致地写「616 行」「256 份」「0 行对不上」，产物文件本身写的是「**612** 行」「**255** 份」、且 88 号那一支明确列出**1 行**对不上（`.claude/kb/experiments/153-账本形态与环上有洞的代价.md` 第 85 行），带着具体例子。**判 ✗**：正文引产物的数字与产物本身不符，而且不是四舍五入级别的漂移——「0 行对不上」与产物里明写的「1 行对不上」方向相反，产物自己给出了一个反例（T5-a 判「站住」这一格所依赖的关键事实）。

什么会推翻这一条：产物文件 `t5-moved-pages.out` 被证明不是当时那次命令跑出来的原始输出（例如 sha256 对不上），或者正文这三处数字来自另一次未留存的重跑。当前 sha256 核对显示产物文件与模型交回时的哈希一致，不支持这个推翻条件。

### 3.4 文件:行号 引文核对（对快照树逐字比对）

46 处「文件:行号」引用逐条用 `awk 'NR==N'` 取快照树对应行、与报告里贴的原文比对，43 处逐字相符，
3 处「核不动」（2 处 `doc-lint.sh:900/901`、1 处 `30-decision-history.sh:52`——后者 opus 报告自己已注明
「（主树，不在快照清单里）」，不算它没交代清楚）。抽样贴几条代表性的：

| 文件:行 | 结果 |
|---|---|
| `.claude/gate.d/75-decision-experiment-links.sh:17,18,26,30,68,287,368,389,394,444,450` | ✓ 11 处全部逐字相符 |
| `.claude/gate.d/40-results-cited.sh:53,119` | ✓ |
| `.claude/gate.d/88-quoted-result-lines.sh:118` | ✓ |
| `.claude/gate.d/11-batch-scope.sh:50` | ✓ |
| `.claude/gate.d/61-settled-same-file.sh:86` | ✓（含行尾续行反斜杠，逐字符核对相符） |
| `.claude/gate.d/31-blocking-verdict.sh:59`、`52-segment-registry.sh:20,27` | ✓ |
| `.claude/rules/path-moves.md:6` | ✓ |
| `research/scripts/changed-paths.sh:99` | ✓ |
| `.claude/singlefs-ai-sop/scripts/lib.sh:220`、`gate.sh:433` | ✓ |
| `.claude/singlefs-ai-sop/scripts/doc-lint.sh:900,901` | 核不动：快照树里没有此文件 |
| `.claude/gate.d/30-decision-history.sh:52` | 核不动：快照树里没有此文件（报告自己已注明「主树，不在快照清单里」） |
| `research/prompts/_gate-fix-forks-r3-body.md:13,79`（背景材料，非 kb） | ✓ 与背景材料本身逐字相符 |
| `.claude/gate.d/64-change-range-single-source.sh:15,38,40,43,52` | ✓ 5 处全部逐字相符 |
| `research/scripts/check-segment-registry.py:218,235,264,269,342,358,360,478,484,518,583,585,587` | ✓ 13 处全部逐字相符 |

命令与逐条输出见本会话记录（Bash 历史，均对 `/tmp/claude-1000/gate-fix-forks-r3-snapshot-tree/` 现取）。

## 四、云端正推（Sonnet）腿

### 4.1 文件:行号 引文核对

| 文件:行 | 结果 |
|---|---|
| `.claude/gate.d/75-decision-experiment-links.sh:16-18,25,52,54-58` | ✓ 逐字相符（对快照树） |
| `.claude/gate.d/10-kb-rot.sh:15-17` | ✓ |
| `.claude/rules/implementation-workflow.md:22,8-14,20` | ✓ 对快照树相符（此文件是「腿跑着的时候改过」三份之一，但快照树保留了腿开工时的原样，报告本身就是对着这份原样引用，核对无误） |
| `.claude/rules/format-evolution.md:41-43` | **部分 ✗**：报告第 50 行引用第 41 行「支撑、推翻要写到分项……都要有这一行」，实际第 41 行原文结尾还有「、关系是支撑或推翻。」一句，报告的引文在此处截断，没有抄全（也没抄开头的「关系是 支撑 / 推翻 / 备料 / 不影响 之一。」）。判定：引文摘句，未整行抄，按「引 kb 里的条目要整行抄」核对表单列，不因此认定第 41/42 行「双向限定在依据段」的判断本身错——那是推理，不归本报告判 |
| `.claude/kb/experiments/105-…md:5`、`152-…md:7`、`154-…md:6`、`156-…md:7`、`69-…md:5,7,153-155` | ✓ 对主树现查相符（不在快照清单，行号与内容今天与报告一致） |
| `research/prompts/_gate-fix-forks-r3-background.md:845` | ✓ 现查确认背景材料第 845 行确实写「两条判据」，与脚本头部「三条」（①②③）不符——sonnet 指出的材料自相矛盾属实 |

### 4.2 可复跑的数字断言

| 断言 | 复跑结果 |
|---|---|
| 「另外 23 道阶段」cd 之后用 `dirname "$0"` | ✓：对当前主树重跑同一段 python 脚本，输出 23，文件名单逐一相同 |
| 143 个 red/green 带 expect 的样本目录 | ✓：`find` 命令重跑得 143 |
| 其中 54/55/57/59/87 号样本 6 份 | ✓：`grep` 重跑命中同 6 个目录（59×2、57×2、55×2） |
| 137 个样本全部判对、0 处 ✗ | ✓：sonnet 草稿目录 `stage-selftest-filtered.out` 里 `grep -c '✓.*判得对'`=137、`grep -c '✗'`=0，与报告一致 |
| `changed-paths.sh --selftest` 14 项 | ✓（对快照树）：14 项，逐字相符；**对当前主树重跑得 15 项**（多出「HEAD 没出生取空树」一条自证）——这是主树在核查期间被进一步改动（`research/scripts/changed-paths.sh` 今天已比腿工作时新增了一条自证），核对快照树消掉这处表面不符，不判 ✗，供主 agent 知悉主树漂移方向 |
| 64 号「三条」判据、红样本各缺 5/7/2 条 | 核不动（需要拷贝改脚本重跑三次变体，本报告未重做这一步；sonnet 自己的命令与原样输出已在报告正文完整贴出，逻辑自洽，未独立复算） |
| 十九格 `clip` 用例 | 核不动（未独立重算 19 格，抽样核对了 gen-decision-items.py 引用的 10 处行号，见下） |
| `.claude/scripts/gen-decision-items.py:34-35,44-45,46-47,48-54,49` 等 | ✓ 对快照树逐字相符 |


## 五、本地攻方（Qwen）腿

### 5.1 转述核对表——逐条核「原文文件:行」

21 处「原文文件:行」逐条用 `awk 'NR==N'` 取快照树（`.claude/gate.d/80-absolute-assertions.sh`、
`crates/singlefs-harness/src/bin/*.rs`、`.claude/scripts/gen-decision-items.py`）或主树
（`research/prompts/_gate-fix-forks-r3-body.md`，不在快照清单）比对：

| 文件:行 | 结果 |
|---|---|
| `.claude/gate.d/80-absolute-assertions.sh:2,18,20-21` | ✓ 逐字相符 |
| `.claude/gate.d/80-absolute-assertions.sh:19` | ✓ 内容相符；核对表里这一格的英文引文实际跨到了第 20 行开头一小句（「so an experiment living under crates/ follows the same rule as one under research/」出自第 20 行「住在 crates/ 下的实验与 research/ 下的同规矩」），只标了 `:19`，行号标注略窄，内容本身没有失真 |
| `crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs:1-2` | ✓ |
| `research/prompts/_gate-fix-forks-r3-body.md:19` | ✓（对主树现查，这是背景材料本身，非 kb） |
| `crates/singlefs-harness/src/bin/e158_root_choice_repair.rs:1` | ✓ |
| `crates/singlefs-harness/src/bin/first_transaction_device_log_check.rs:1` | ✓ |
| `crates/singlefs-harness/src/bin/first_transaction_on_device.rs:1,5-7` | ✓（含核对表指出的「虚机档误译成 real-machine tier」这处首稿错误，定稿改译 virtual-machine tier，与源文件「虚机档（QEMU/KVM）」一致） |
| `crates/singlefs-harness/src/bin/first_transaction_region_bytes.rs:1-3` | ✓ |
| `research/scripts/replay.sh`（`driver_e142` 函数体，核对表自己声明「不写行号」） | 核不动：核对表没给行号，按它自己交代的理由（函数名定位）不核 |
| `.claude/scripts/gen-decision-items.py:32-35,36-37,42,44-45,46-47,49,50,51,62,66-69` | ✓ 全部逐字相符（10 处行区间） |

### 5.2 发现一处遗漏（✗）：连接符字符集「六个」与源码正则「七个」字符不符

核对表第 35 行与提示原文（`gate-fix-forks-r3-local-attack.md:200-203`）都写「six specific Chinese
punctuation or connective characters」，并列出「顿号、逗号（ASCII）、全角逗号、中点、`的`、
`与`」共六项（提示原文把最后一项写成「two single Chinese characters … meaning "of" and "and"」，
只对应 `的` 与一个表「and」的字）。现查源码正则（快照树 `.claude/scripts/gen-decision-items.py:51`）：

```
        t2 = re.sub(r'[\s/、,，·的与和]+$', '', t2)
```

字符类里除空白与斜杠外，实际有 **7** 个有意义字符：`、`、`,`、`，`、`·`、`的`、`与`、`和`——
`与`和`和`是两个不同的字、都表「and」，提示与核对表都只覆盖了其中一个，另一个（`与` 或 `和`，
提示原文没有点名具体是哪一个，只说「一个表 and 的字」）没有出现在给模型的事实描述里。
核对表自己写的「具体六个字符（顿号、逗号、全角逗号、中点、的、与、和）」这一行本身就自相矛盾——
带资付括号里列了 7 项，前面却说「六个」。**判 ✗**：提示给模型的字符集事实表比源码正则少了一个字符，
核对表没有把这个缺口记下来（它把「六」与列出的「七项」的落差当成了无需解释的事实）。

什么会推翻这一条：确认 `与` 与 `和` 在这条正则的语境下本来就是同一个语义槽位、提示原文「meaning
"and"」那半句原本就打算同时代表两个字（如果提示原文明确写了「与或和」两个字都译，则不成立）——
现查提示原文第 202 行只写单数「a … character」，不支持这个反驳。

### 5.3 运行记录（`corruption-check.py`、`oov-check.py`、词数）复跑

| 断言 | 复跑结果 |
|---|---|
| 提示本身：cjk=531 words=4229 fffd=0……；oov 生词=17 拼接=0 | ✓ 两条命令重跑，输出逐字相符 |
| s1：退出码 0，`wc -w`=389，oov 生词=0 拼接=0，corruption 绿（words=384） | ✓ 全部相符 |
| s2：退出码 0，`wc -w`=726，oov 生词=3（apparatus、blklogwrites、contradicted）拼接=0，corruption 绿（words=718） | ✓ 全部相符，含具体的三个生词逐字相符 |

## 六、计数汇总

| 腿 | 核了几处 | ✓ | ✗ | 核不动 |
|---|---|---|---|---|
| 云端攻方（Opus） | 引文 46 处 + 模型复跑 12 条 + 产物数字核对 1 组 = 59 | 55 | 1（3.3 节：正文数字与自己产物 t5-moved-pages.out 不符） | 4（doc-lint.sh×2、30-decision-history.sh×1、t1-titles/t1-mention-forms/t5-moved-pages 三条重跑因快照缺语料而记 3 处，其中 t5-moved-pages 与✗那一条是同一处但角度不同，此处合并计入核不动一次、✗一次，不重复计） |
| 云端正推（Sonnet） | 引文/数字 15 处 | 13 | 1（format-evolution.md:41 摘句未整行抄） | 1（64 号三变体、19 格 clip 未独立复算，计为 1 处核不动，报告正文已注明） |
| 本地攻方（Qwen） | 引文 21 处 + 运行记录 3 组 = 24 | 22 | 1（5.2 节：连接符字符集六对七不符） | 1（replay.sh driver_e142，核对表自己交代不写行号） |

三条腿合计核了约 97 处，✓ 90 处，✗ 3 处，核不动 6 处。


## 七、没做什么

- 不判一条打中成不成立、该不该采纳；不核推理本身，只核引用、产物与复跑——本报告的三处 ✗ 都只是
  「引文/产物与其自己声称的数字不符」，不代表 T5-a「站住」、format-evolution.md 那一格的判断、或
  「六字符/七字符」那一处的英文提示设计本身是错的，这些判断留给主 agent 逐条现查。
- 没跑 54、55、57、59、87 号（派发提示明令）。
- 没有独立复算 sonnet 报告「64 号三条判据各关一条红样本判错」的三次屏蔽实验（拷贝改脚本重跑）——
  sonnet 报告正文自己贴了完整命令与原样输出，本报告未重做，标「核不动」。
- 没有独立复算 sonnet 报告「十九格 clip 用例」的完整重跑（只抽样核对了它引用的 10 处源码行号）。
- 没有对 opus 报告 T1-f 里「10 号并入 ⑨ 之后」那条正则的真仓 7 页现跑复核（opus 报告自己已经贴了
  命令与原样输出，本报告未重跑第二遍）。
- 没有核对 sonnet 报告「读法乙」E105/E152/E154/E156 四页「### 影响的决策」表格里具体哪几行写着
  「备料」（只核了 E69 那一页「三行全是不影响」这一具体断言，因为这是报告里唯一被单独拎出来
  与其余四页对照的关键行）。
- 没有核对 opus 报告 T5-c/T5-d/T5-e 三张表格里每一格的退出码（① ② ③ 各种写法、七种仓状态 ×
  七道阶段、G/F 两组配置）——这些需要在快照树上搭建多个临时仓现跑，工作量很大，本报告只核了
  这些表格引用的源码行号（64 号五处、75/61 号相关行），没有重新搭建现场逐格复算退出码矩阵。
- 没有编译 Rust，没有跑 `gate.sh` 全量或任何门禁阶段。
- 没有改动仓里除报告文件之外的任何路径；草稿产物在
  `/tmp/claude-1000/gate-fix-forks-r3-verifier/`（`opus-rerun/`、`selftest/`、`opus-model-sha.txt`）。

