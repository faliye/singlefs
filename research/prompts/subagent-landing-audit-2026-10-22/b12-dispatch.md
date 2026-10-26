# 第十二批任务 39：派发闸、派发骨架与三方腿报告路径库（tooling-writer，2026-10-23）

批次：records/2026-10-22-子agent结果落地审计.md#12；关的行：拆分提案第九十节 ㉗ 全部、㉙(a)、㉚ 的 dispatch-prompt.py 那一处、㉟ 的跨行「」引文、dispatch-prompt.py 文件头第 38、39 行与开关点名。

## 结论

| 条 | 验收 | 读数 |
|---|---|---|
| 1 ㉗(i) 并 ㉙(a) | r4 攻方提示头一版的写法判红；r4、r5、r6 六份真腿提示放行 | 红样本 `a75e9e30847f57bcd-forbidden-own.txt` 改前放、改后拒（三方腿报告路径）；六份改前改后都放行；七份钉进钩子样本目录，自证逐份判 |
| 2 ㉗(ii) | 三方腿报告路径名字里带 report 照判 | 原来钉成「不判、应得 0」的那一格改成应得 2；另加整份提示那一支、SUMMARY 前缀、不带 report 放行、别的类型不判四格 |
| 3 ㉗(iii) | 没有草稿目录一行时只判裸文件名，写明判法加样本 | 文件头 ⑰ 写全（没写、或写了不在工作根下，都只判裸文件名与「<草稿目录>/…」，三方腿形态照判）；加三格 |
| 4 ㉗(iv) | 三种认法统一进库里一个函数 | 报告行那一支改调新加的 `leg_report_written_tree`，它与 `leg_report_mentions` 都只包在 `leg_report_mentions_in_trees` 外面，交回闸照旧调这个函数，签名没变；自证加一格，三处对 7 种写法判得一样 |
| 5 ㉚ | `DRAFT_LINE` 改调 `lib_prompt_scratch_line.py` | `claim_problems` 经 `draft_line_value` 调 `scratch_line_values`；原正则只在新开关 `draft-line-own-regex` 下用 |
| 6 ㉟ | 跨行「」整份配对；文件头第 38、39 行重写；6 条开关点名与红的格逐格相同 | `quoted_spans` 的「」『』“”整份配对（反引号仍按行）；第 38、39 行照今天的 ⑰ 重写；派发闸 disable-check 331/331、report-name-unchecked 38/38、report-line-position-anchored 25/25，dispatch-prompt.py long-run-lines-dropped 42/42、long-run-types-narrowed 36/36、live-words-narrowed 1/1，都逐格相同 |

真提示上的读数（3394 份子 agent 首条派发提示，加钉住的 21 份样本，共 3415 份；只比 ⑰ 的两判）：
- 新放 0。
- 新拒：按派发当时登记的工作树算是 4 份，按今天的工作树算是 7 份。多出来的 3 份（a3b0c2a40d1b94bf7、a823ae170ca3a7b37、aded7b9f532c1dae0）报告行写在 m4-predev 工作树里，那棵树今天已经不在了。在测量工具里把它当登记过的树算，这 3 份照样放行，所以不是判法新拒的。
- 4 份新拒逐份点名：
  - a2e0cc6ef56fad5c8（2026-10-16 攻方）：本腿的报告只写在反引号里，「报告写 `…`」用的也不是四种标签之一。
  - a5cd227cc10a22b61、aaa9106f04bbc1570（2026-10-13）：报告路径写在另一份 prompt.md 里，提示里没有。
  - 前面这 3 份改前只靠禁读清单里另一条腿的 -output.md 放行，第九批交回 b9-dispatch-guard.md「弱处」点名的就是这 3 份。照 ㉗(i) 的规矩它们该拒。
  - fx-a75e9e30847f57bcd-forbidden-own.txt：是我造的红样本，该拒。
- 结论：没有新拒是不该拒的，判法没为它们改。

推翻条件：
- 主工作区派发闸 `--selftest` 不绿。
- 上面六条开关下自证的红格与文件头点名的不是逐格相同。
- 新放不是 0，或者上面 4 份之外出现别的新拒。
- 交回闸自证因为库改动变红。

## 改过的文件

| 文件 | 开工 sha256 | 交回 sha256 | 换法 |
|---|---|---|---|
| `.claude/hooks/runner-dispatch-guard.sh` | fc677522…2406e | bc9fe8ca3f2417366d7200a8e3b045d6755140ef915d4f8cbe8c198e9e892cf4 | hook-swap 四次（swap1 判法、自证、样本期望与库；swap2 报告行后面合形态的词；swap3、swap4 开关点名），每次退 0 |
| `.claude/hooks/lib_three_way_legs.py` | 43b81c19…4c823 | ae79f364370db34b8d3fb0e971a25948f51e768c444e9da93a7bd7431f0a268c | 随 swap1 一起换上 |
| `scripts/agents/dispatch-prompt.py` | e6f5e4f5…6d49e | ce669604eeba0686affb013e9f61979417f19b8410ef62e3344305e97f2f1584 | replace-batch 两次（每次先 --dry-run、换上之前核基线） |
| `.claude/hooks/fixtures/runner-dispatch-guard/real-prompts/sources.tsv` | — | f7af89d1c147b658cf4980d8277bcee26b46394651c860594328997a1f71b137 | Edit：文件头补来历，加 7 行 |
| 同目录新建 7 份：a61301bfaed0a3433.txt、a75e9e30847f57bcd.txt、a75e9e30847f57bcd-forbidden-own.txt、a77dffd2065a0ba7e.txt、ab7f880bc2d28e03a.txt、a8a3bd6f093e19673.txt、a42154ccf8829dc32.txt | — | 见 sources.tsv | `cp -n` 排他新建（先确认 7 份都不在） |

`git diff --stat -- <这几份>` 原样。比的是暂存区，派发闸里还混着别的会话没暂存的改动（⑥ 检出与 ⑯ 判不了的锁那一批，开工时就在，没回退）：

```
 .../runner-dispatch-guard/real-prompts/sources.tsv |  11 +-
 .claude/hooks/lib_three_way_legs.py                |  28 +-
 .claude/hooks/runner-dispatch-guard.sh             | 811 ++++++++++++++++-----
 scripts/agents/dispatch-prompt.py                  | 111 ++-
 4 files changed, 765 insertions(+), 196 deletions(-)
```

只算这一批的改动（开工原样对交回时的文件，`git diff --no-index --stat` 末行）：

- 派发闸：436 insertions、148 deletions
- 库：23 insertions、5 deletions
- 骨架：93 insertions、18 deletions

新加的 7 份样本是未跟踪的文件，上面的 stat 里没有。

## 各条改了什么（行号是交回时这一版）

1. ㉗(i) 并 ㉙(a)。派发闸 `leg_report_reading`（:1687）与 `three_way_report_verdict`（:1749），本腿的报告只认两处：
   - 报告行点名的，报告标签落在禁读清单里的不算。
   - 报告行里一处都没有时，整份提示里同时满足三条的那一处：不在引号里、不在禁读清单里、名字以本腿轮名加「-」起头。
   - 本腿轮名（`leg_round_names`，:1544）两处取：行首「背景材料：」一行里的 `_<轮>-background.md` / `_<轮>-body.md`，与前面不连着字、不在引号里的「轮名」标签。在语料上核过：认出的轮名与报告行点名的轮名一次都没冲突（634 处一致，61 处认不出轮名但有报告行）。
   - 禁读清单那几段（`forbidden_list_spans`，:1516）有三种写法：行首「禁读清单」/「禁读」接冒号的一行；冒号后面空着时往下连着的列表项；标题里有「禁读」的一节。
   - 禁读清单里点到本腿自己的报告（引号、反引号里的也算）就拒。拿来比的是禁读清单之外认到的那几份，所以开关 forbidden-counted 下不会自己撞自己。
2. ㉗(ii)。`report_file_name_verdict` 加了参数 `project_root=None`（:1611），两处调用都传了。类型是三方腿时，本腿的报告路径也判名字：带 report、以 SUMMARY / FINDINGS / ANALYSIS 开头的拒。
3. ㉗(iii)。文件头 ⑰ 的「提示里没有这一行的」一段写全了。加的三格：没有草稿目录一行时裸文件名照判；草稿目录一行不在工作根下时，带目录的不判、裸文件名照判。
4. ㉗(iv)。
   - 库里新加 `leg_report_written_tree(写法, 仓根[, 工作树])`（lib :171）：拿 `leg_report_mentions_in_trees` 认，要恰好一处、到词尾为止、前面那一截整个是路径。
   - 派发闸的 `leg_report_path_verdict`（:1671）改调它。所以 `./`、仓根与工作树的绝对路径、`~`、`$HOME`、`../<仓根目录名>/` 三处判得一样；占位、子目录、仓外副本三处都不认。
   - `is_leg_report_path` 留着，只给开关 leg-report-line-root-only 用。
   - 换上之后报告行有一格变红（「报告路径（形态 `…<轮>…`）：research/prompts/x-r1-sonnet-output.md」）。原来靠整份提示那一支放行，现在那一支要轮名。改法：第一个像路径的词不合形态、又不是本提示自己的时，这一行后面合形态的词照认（swap2）。这一条也写进了文件头。
   - 交回闸文件我没碰，它调的 `leg_report_mentions_in_trees` 签名没变；交回闸自证绿（117 种）。
5. ㉚。dispatch-prompt.py 的 `draft_line_value`（:285）调库，`claim_problems`（:300）改用它。读不到库时 stderr 写一句、认领行不核。
6. ㉟。
   - `quoted_spans`（:1472）的「」『』“”整份配对、配最近的那个开头；反引号仍按行配对。
   - dispatch-prompt.py 文件头第 36–44 行照今天的 ⑰ 重写：两处认法、四种拒、禁读清单只写别的腿；另记草稿目录一行的认法在共用库。
   - 开关点名：派发闸 19 条、骨架 4 条重写成逐格列全，读法与写法见下一节。

## 弄坏开关与自证格

新开关（派发闸，`RUNNER_DISPATCH_GUARD_BREAK=`，文件头第 312–325 行）：

| 开关 | 弄坏哪一处 | 管的条 |
|---|---|---|
| leg-report-line-root-only | 报告行那一支退回只去仓根前缀再调 is_leg_report_path | ㉗(iv) |
| leg-report-forbidden-counted | 禁读清单当普通正文 | ㉗(i) |
| leg-report-any-round | 整份提示那一支不看本腿轮名 | ㉗(i) |
| leg-report-forbidden-own-unchecked | 不判禁读清单点到本腿自己的报告 | ㉙(a) |
| leg-report-name-unchecked | 文件名那一判不看三方腿的报告路径 | ㉗(ii) |
| quote-pairs-per-line | 「」退回按行配对 | ㉟ |

骨架新开关：`DISPATCH_PROMPT_BREAK=draft-line-own-regex`，退回原来的 DRAFT_LINE（㉚）。

读法照 code-tooling 的 break-switch-replay 格：开关表与「」项的读法从 `.claude/gate.d/code-tooling.sh` 现取（`tools/replay.py`），其余 `_BREAK` 变量摘掉，起 `--selftest`。另比一样：每一格失败行都被点名。主工作区交回时这一版的读数（`replay-guard-final.jsonl`、`replay-dp-after2.jsonl`）：

```
派发闸：switches 112; exact 93; missing>0 0; unnamed>0 19; exit0 0
骨架：  switches 19; exact 12; missing>0 0; unnamed>0 7; exit0 0
```

逐格相同的（点名数 = 失败行数，点名没照出 0，失败行没点名 0）：
- 派发闸：disable-check 331、report-name-unchecked 38、report-line-position-anchored 25，6 条新开关（2、3、2、4、3、3）。另有 10 条点名被这一批改的格名带坏了、或这一批新格让它们多红，一并列全：leg-report-placeholder-kept 3、leg-report-outside-root-kept 3、real-prompts-absent-green 2、real-prompts-digest-unchecked 1、thread-limit-any-refused 137、report-line-labels-split 20、report-line-unanchored 8、LIB_PROMPT_SCRATCH_LINE_BREAK=label-anywhere 6、three-way-report-unchecked 24、report-name-prefixes-unchecked 6。
- 骨架：long-run-lines-dropped 42、long-run-types-narrowed 36、live-words-narrowed 1、draft-line-own-regex 3。骨架这几条的项写到「实际」为止：失败行后半截是派发闸输出截下来的最后 600 字，会跟着派发闸的措辞变。

新开关打开后红的格（原样，`logs/new-switch-rows.txt`，长的截在 260 字）：

```
RUNNER_DISPATCH_GUARD_BREAK=leg-report-placeholder-kept
    ✗ 自检：三方报告:报告行写没填的 <轮> 占位、整份提示没有别的 -output.md，拒 应当返回 2，实际 0
    ✗ 自检：三方报告:报告行与整份提示两处形态都经 lib_three_way_legs.py 的 leg_report_mentions_in_trees 应当返回 (True, True, False)，实际 (False, True, True)
    ✗ 自检：三方报告:报告行、整份提示与交回闸 ② 三处对同一组写法判得一样 应当返回 ([True, True, True, True, False, False, False], [True, True, True, True, False, False, False], [True, True, True, True, False, False, False])，实际 ([True, True, True, False, True, False, False], [True, True, True, True,
RUNNER_DISPATCH_GUARD_BREAK=leg-report-outside-root-kept
    ✗ 自检：三方报告:整份提示里只有仓外同名的 …/research/prompts/…-output.md，拒 应当返回 2，实际 0
    ✗ 自检：三方报告:报告行与整份提示两处形态都经 lib_three_way_legs.py 的 leg_report_mentions_in_trees 应当返回 (True, True, False)，实际 (True, False, False)
    ✗ 自检：三方报告:报告行、整份提示与交回闸 ② 三处对同一组写法判得一样 应当返回 ([True, True, True, True, False, False, False], [True, True, True, True, False, False, False], [True, True, True, True, False, False, False])，实际 ([True, True, True, True, False, False, False], [True, True, True, True,
RUNNER_DISPATCH_GUARD_BREAK=leg-report-line-root-only
    ✗ 自检：三方报告:报告行与整份提示两处形态都经 lib_three_way_legs.py 的 leg_report_mentions_in_trees 应当返回 (True, True, False)，实际 (False, True, True)
    ✗ 自检：三方报告:报告行、整份提示与交回闸 ② 三处对同一组写法判得一样 应当返回 ([True, True, True, True, False, False, False], [True, True, True, True, False, False, False], [True, True, True, True, False, False, False])，实际 ([True, True, True, False, False, False, False], [True, True, True, True
RUNNER_DISPATCH_GUARD_BREAK=leg-report-forbidden-counted
    ✗ 自检：三方报告:禁读清单里另一条腿的 -output.md 不当本腿的报告，拒 应当返回 2，实际 0
    ✗ 自检：三方报告:禁读清单冒号后空着、往下列表项里另一条腿的 -output.md 也不当本腿的报告，拒 应当返回 2，实际 0
    ✗ 自检：三方报告:标题「禁读」一节里的 -output.md 不当本腿的报告，拒 应当返回 2，实际 0
RUNNER_DISPATCH_GUARD_BREAK=leg-report-any-round
    ✗ 自检：三方报告:整份提示里只有别的轮的 -output.md（背景里的上一轮腿报告），拒 应当返回 2，实际 0
    ✗ 自检：三方报告:轮名 x-r1 不认 x-r10 下的 -output.md，拒 应当返回 2，实际 0
RUNNER_DISPATCH_GUARD_BREAK=leg-report-forbidden-own-unchecked
    ✗ 自检：三方报告:禁读清单里点到本腿自己的报告路径拒（landing-audit-r4 攻方提示头一版的写法） 应当返回 2，实际 0
    ✗ 自检：三方报告:禁读清单里点到本腿自己的报告，写在反引号里也拒 应当返回 2，实际 0
    ✗ 自检：三方报告:禁读清单点到本腿报告的拒绝说明点名那一份与改法 应当返回 True，实际 False
    ✗ 自检：真提示:a75e9e30847f57bcd-forbidden-own.txt（three-way-attack，landing-audit-r4 攻方腿改一处：禁读清单写成本腿自己的报告（攻方提示头一版的写法，拆分提案第九十节 ㉙），拒） 应当返回 ('89a944c57dddb9e109e73d2368048f71957cdd48099f45db60042391ded6dd96', False, True)，实际 ('89a944c57dddb9e109e73d2368048f71957cdd4809
RUNNER_DISPATCH_GUARD_BREAK=leg-report-name-unchecked
    ✗ 自检：报告名:三方腿报告行写的本腿报告路径 research/prompts/<名>-output.md 名字里带 report 照判 应当返回 2，实际 0
    ✗ 自检：报告名:三方腿整份提示里本腿轮名下的报告路径名字里带 report 照判 应当返回 2，实际 0
    ✗ 自检：报告名:三方腿的报告路径以 SUMMARY 开头照判 应当返回 2，实际 0
RUNNER_DISPATCH_GUARD_BREAK=quote-pairs-per-line
    ✗ 自检：报告名:跨行的「」引文里的报告标签不认 应当返回 0，实际 2
    ✗ 自检：引号:「」照整份配对、可以跨行、配最近的那个开头，反引号按行配对 应当返回 [(5, 8), (10, 13), (14, 22)]，实际 [(5, 8), (10, 13)]
    ✗ 自检：三方报告:跨行的「」引文里的 -output.md 不当本腿的报告，拒 应当返回 2，实际 0
DISPATCH_PROMPT_BREAK=live-words-narrowed
    ✗ 自检：现算词表应当认出每一个现算词：available_parallelism nproc cpu_count sched_getaffinity 核数 忙闲 空闲 负载 除以 ÷
DISPATCH_PROMPT_BREAK=draft-line-own-regex
    ✗ 自检：认领行 绿：闭合了的围栏代码块里的草稿目录一行不算，不要求认领：应当 0 个问题，实际 ["有「草稿目录：~/x/」却没有「认领：bash scripts/hosts/work-directory-ownership.sh claim <草稿目录> --purpose '<用途一句>'」一行"]
    ✗ 自检：认领行 红：半角冒号的草稿目录一行同样要认领：应当 1 个问题，实际 []
    ✗ 自检：认领行调共用库：「草稿目录：」一行应当经 lib_prompt_scratch_line.py 的 scratch_line_values 认，实际 []
```

自证：
- 派发闸 525 种增到 554 种。
- dispatch-prompt.py 92 项增到 95 项。
- 交回闸 117 种，没改它，因为库改了才复跑。
- agent-cost.py 29 项，同上，只读地复跑。

四份都在主工作区跑、都退 0，原样在 `logs/selftest-final.out`、`logs/dp-selftest-dp2.out`、`logs/handback-selftest.out`、`logs/agent-cost-selftest.out`。四份都登记为 light。

hook-swap 喂法（`feeds-swap1.json`，stdin 入口，6 条）：
- 改前（`--feed-only` 喂开工时的钩子，`logs/feeds-before-swap1.txt`）：

```
  ✗ 退 0（应当 2） 攻方腿禁读清单点到本腿自己的报告（landing-audit-r4 攻方提示头一版）拒
  ✓ 退 0（应当 0） 攻方腿禁读清单列另一条腿、报告行写本腿的放行 ← 
  ✗ 退 0（应当 2） 正推腿报告路径名字里带 report 拒 ← 
  ✓ 退 0（应当 0） 辩方腿没有报告行、整份提示里本腿轮名下的 -output.md 放行 ← 
  ✗ 退 0（应当 2） 正推腿只在禁读清单里有 -output.md、没有报告行拒 ← 
  ✗ 退 0（应当 2） 辩方腿跨行「」引文里的 -output.md 不当本腿的报告、拒 ← 
✗ 只喂不换：.claude/hooks/runner-dispatch-guard.sh 喂了 6 条，4 条不对
→ 怎么办：不对的几条是钩子此刻的判法与喂法对不上；先看是钩子退回了、还是喂法写错
```

- 改后：四次换上，草稿里和主工作区里各喂一遍，6 条全对。
- 喂法里的草稿目录一行不写冒号，所以没在真工作根里建锁。

语法检查：
- hook-swap 每次在草稿里和主工作区都跑了 `bash -n` 并编译内嵌 python。
- dispatch-prompt.py 用 `py_compile` 与 `ast.parse` 查过。py_compile 写出的 `scripts/agents/__pycache__/dispatch-prompt.cpython-312.pyc` 已删，那个目录被 git 忽略。

## 新写的 # gate-similar: / # hook-events:

无。没有新建门禁阶段或钩子，判据都追加进了派发闸。`gate-overlap.py --list` 里管派发提示的只有派发闸一份（`logs/gate-overlap-list.txt` 第 53 行）。

## 交主 agent 的

1. 撤回：hook-swap 写的反向规格在 `hook-swap/` 下，按时刻倒着依次跑。
   - `reverse-20261023T082436Z-1769143.json` 是 --dry-run 那一次写的，不用。
   - 其余四份依次撤 swap4、swap3、swap2、swap1：`python3 scripts/gate/hook-swap.py <反向规格> --hook .claude/hooks/runner-dispatch-guard.sh --scratch <草稿目录> --feeds <草稿目录>/feeds-neutral.json`。
   - dispatch-prompt.py 从 `originals/` 对出规格，再走 replace-batch。
   - 7 份样本与 sources.tsv 的 7 行跟着派发闸 swap1 一起撤。
2. `.claude/hooks/lib_prompt_scratch_line.py` 文件头的「五处用这一份」要加 `scripts/agents/dispatch-prompt.py`（认领行那一判）。那份库归任务 40，我没碰，请转给它。
3. 提交前归门禁、我没跑的几格：
   - code-tooling 的 break-switch-headers、break-switch-replay：派发闸新加 6 条开关；dispatch-prompt.py 不在复跑登记表里。
   - code-tooling 的 break-switch-outside-selftest：派发闸平常跑也读开关，原来就这样。
   - gate-overlap、hooks-registered。
   - 单跑一次派发闸自证约 10 秒（负载 49 时），登记表给每条 120 秒。
4. 派发闸另有 19 条、骨架另有 7 条开关，点名比实际红的格少（只核了点名的在不在，这是那一格的合同），开工前就这样。不在这一批，留给任务 42 的全仓补齐：
   - 派发闸：disable-heavy-test、disable-snapshot、inputs-ignored、report-name-whole-path、placeholders-ignored、runner-backfill-refused、scope-token-unexpanded、snapshot-loose-lines、snapshot-unreadable-aborts、snapshot-unreadable-denies、valve-reason-ignored、long-run-duplicate-label-first-wins、check-mode-registers、snapshot-abandoned-unchecked、snapshot-abandoned-line-unread、writer-lock-unchecked、LIB_LONG_RUN_BREAK=hosts-unchecked、hosts-canonical-unchecked、SINGLEFS_WORK_ROOT_BREAK=new-root-unknown。
   - 骨架：guard-skipped、claim-line-dropped、types-before-nine、thread-formula-restored、three-way-report-default-dropped，以及经骨架起的 RUNNER_DISPATCH_GUARD_BREAK=placeholders-ignored、thread-limit-any-refused。
5. 判法自己还有一处推的弱处，没量过误放：
   - 辩方腿没写报告行时，正文里点名同一轮别的腿的报告当阅读材料、又不在禁读清单里，会被当成本腿的报告放行。
   - 语料里报告行之外认的那一支放行的，都是本腿的报告（新放 0、新拒只有上面那几份），没撞上这种情形。

## 草稿目录

`/home/fy5090/.local/share/singlefs/work/landing-audit-b12-dispatch/`，12M：
- 删了：交回前删掉了比改前改后用的两棵符号链接树 `before-hooks/`（504K，含开工时派发闸与库的拷贝）、`after-hooks/`（76K），以及抽出来的真提示语料 `corpus/`（18M）。
- 留着的，在 `.deliverables` 里声明：`originals/`（备份）、`draft/`（与主工作区逐字相同的最终全文）、`logs/`（每次换上前那一版、自证与喂法输出）、`hook-swap/`（反向规格）、本报告。
- 另留文本：工具 `tools/`、读数 `*.jsonl`、规格 `spec-*.json`、喂法、`fixture-staging/`、`samples/`。
- 没有仓副本，没有编译目录。
- 精确量 precheck 时在 /tmp 留了 8 个临时样本仓，已逐个删掉。

## 没做什么

- 没跑重型测试（checker-tier-crash-point-replay、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification、checker-tier-research-build-and-replay 本身、checker-tier-broad-mutation 与 `scripts/mutation/broad-mutation.py` 起 cargo 的用法、`gate.sh`、全量 `cargo test`、`check.sh`）；没走定义三方；没提交。
- 没跑门禁阶段与各类 lint。没改 `.rs`，所以没跑命名检查；主 agent 中途通知的新写法用不上。
- 没碰 `.claude/hooks/handback-guard.sh`、`.claude/agent-common.md`、`.claude/hooks/lib_prompt_scratch_line.py`、`scripts/agents/agent-cost.py`、`scripts/kb/`、`.claude/gate.d/`。
- 派发闸 `heavy_permission_sentences` 的引号（`HEAVY_PERMISSION_QUOTED`）仍按行配对，没跟着改成跨行。它本来逐行判，出处 b9:264 说的是 ⑰ 的 `quoted_spans`。
- 语料是开工时抽的 3394 份。之后新派的几份（这一批的兄弟任务）没进语料。
- 中途收到主 agent 的消息：在派发闸 swap4 之后、写报告之前，通知共用约束改了三处。这一批不跑命名检查，没改任何步骤。
