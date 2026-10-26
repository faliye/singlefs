# 第十二批任务 48：派发闸与几处钩子的跟进（tooling-writer，2026-10-23）

批次：`records/2026-10-22-子agent结果落地审计.md#12`。关的是 `research/prompts/subagent-landing-audit-2026-10-22/` 下 `b12-scratch-lib.md`、`b12-handback.md`、`b12-dispatch.md`、`b12-defs-method.md` 交回时点出的八条跟进。下文 `<草稿>` 指 `~/.local/share/singlefs/work/landing-audit-b12-dispatch-followup`。

## 结论

| 条 | 做到没有 | 读数 |
|---|---|---|
| 1 ⑰ 前缀名扩展名不分大小写 | 做了 | `SUMMARY.MD` 改前放、改后拒；`summary.markdown` 前后都放。真派发提示 4026 份里报告行写成大写扩展名的 0 份（唯一一处命中是本任务自己的派发提示正文，新判法照样放） |
| 2 ⑰ 认 `$SINGLEFS_WORK_ROOT/` 起头的三方腿报告 | 做了 | 喂 `报告：$SINGLEFS_WORK_ROOT/m5-dev/research/prompts/x-r1-opus-output.md`：改前退 2、改后退 0；展开后不在本仓任何一棵树里的照拒。真三方腿提示里这种写法 0 份 |
| 3 ⑯「草稿目录：」一行两种围栏认法不一致就拒 | 做了 | 两个方向都拒（照 ⑭ ⑮）。真派发提示 4026 份、有「草稿目录：」行的 2374 份里，两个方向都是 0 份，不新拒真提示 |
| 4 库文件头调用方表 | 做了 | 现数 6 份文件调它，原表只列 4 份，补了 `dispatch-prompt.py` 与 `agent-cost.py` |
| 5 完工清理闸相对路径按仓根解 | 做了 | 两格自证一红一绿，新开关下两格都红 |
| 6 三方腿缺「报告：」一行的有几份 | 只数，没改判法 | 骨架落地之后的三方腿派发 34 份，没有行首「报告：」一行的 **3 份**，不是 0，照派发逐份列出交主 agent（见「第 6 条」）。⑰ 对三方腿要求「报告：」一行这一改**没做** |
| 7 开关点名补到逐格相同 | 做了 | 派发闸 115 条开关：点名没照出 0、失败行没点名 0、1499 项对 1499 行。骨架 19 条：点名没照出 0、失败行没点名 0 |
| 8 `snapshot-tree.py` 照搬迁登记换回旧路径 | 做了 | landing-audit-r1 到 r6：对上份数 13→30、24→36、36→46、64→77、14→26、40→40；剩下没对上的逐份原因见「第 8 条」 |

什么现象能推翻：
- 主工作区派发闸、完工清理闸、骨架、`snapshot-tree.py`、库的 `--selftest` 不绿（交回时都绿，原样在 `<草稿>/final-*.txt`）。
- 照 break-switch-replay 格的读法复跑派发闸 115 条、骨架 19 条开关，出现点名没照出或失败行没点名的（读数在 `<草稿>/replay-guard-final-p*.jsonl`、`<草稿>/replay-dp-final.jsonl`）。
- 第 3 条的新拒在今天之后的真派发提示上出现误拦（照骨架写的提示都把「草稿目录：」放在围栏之外）。
- 第 8 条：倒推出的树里有一份照搬迁登记换回之后 sha256 对上、内容却不是快照那一刻的（sha256 撞了才会，推的）。

## 改过的文件

| 文件 | 开工 sha256 | 交回 sha256 | 换法 |
|---|---|---|---|
| `.claude/hooks/runner-dispatch-guard.sh` | bc9fe8ca3f2417366d7200a8e3b045d6755140ef915d4f8cbe8c198e9e892cf4 | 7e9623e255291344938ff20253903887c8afa72188ea007281cd6d2b6bd59911 | `hook-swap.py` 两次（swapA：判法、自证、文件头；swapC：开关点名） |
| `.claude/hooks/lib_prompt_scratch_line.py` | 5b4cbc18f647c28cad29a188a0a6a572867a418e253e6ad3023361968822a6f0 | bc098a5389e07d5ff239e97597c48c61a61964364109976913eae1a79f170903 | 随 swapA 一起换上（只改文件头调用方表） |
| `.claude/hooks/completion-cleanup-guard.sh` | 206cc5fc6b95ef8b061bf29e4c05eceafa4d9a8f5f0e3f84e9ce094b34276ea2 | 21f2078748b7ba82dcf339928db0c0b6bddfec029985aa8af3e2f9845fb700d0 | `hook-swap.py` 两次（swapB：解法、两格、开关；swapE：一条旧开关补点名） |
| `scripts/agents/dispatch-prompt.py` | ce669604eeba0686affb013e9f61979417f19b8410ef62e3344305e97f2f1584 | 11aef185754649bc94697fc630d20c6f1e49747dc565c7a521c2e2bdbb0b53b3 | `hook-swap.py` 脚本规格一次（swapD，`--verify` 给它的 `--selftest`），只改文件头开关表 |
| `scripts/agents/snapshot-tree.py` | dcc8dbfd9b83b02a1a341b1e930f78cef01495c0e46e7d1171c9ad10d69266fa | f3c6173ed01ff1e312e721af9087f2bffa71ffe5a0a34ea26ecc3c69b6f56267 | `hook-swap.py` 脚本规格三次（swapF、swapG、swapH，都带 `--verify`） |

`.claude/hooks/lib_three_way_legs.py` 没改（sha256 仍是 ae79f364370db34b8d3fb0e971a25948f51e768c444e9da93a7bd7431f0a268c）。`.claude/hooks/fixtures/runner-dispatch-guard/` 下的样本没动：新格都是自证里造的提示。每次换上之前都核过主工作区那一份与开工时（或上一次换上之后）逐字节相同，hook-swap 自己也核。

`git diff --stat -- <这五份>` 原样（对暂存区，派发闸与骨架里混着任务 39 没暂存的那一批，`snapshot-tree.py` 暂存的是任务 45 那一版）：

```
 .claude/hooks/completion-cleanup-guard.sh |  38 +++-
 .claude/hooks/lib_prompt_scratch_line.py  |  11 +-
 .claude/hooks/runner-dispatch-guard.sh    | 291 ++++++++++++++++++++++++++++--
 scripts/agents/dispatch-prompt.py         |  98 ++++++++++
 scripts/agents/snapshot-tree.py           | 218 +++++++++++++++++++---
 5 files changed, 610 insertions(+), 46 deletions(-)
```

只算这一件的改动（`git diff --no-index --stat <草稿>/originals/<文件> <文件>` 末行）：派发闸 278 insertions、13 deletions；骨架 98 insertions；库 7 insertions、4 deletions；完工清理闸 36 insertions、2 deletions；`snapshot-tree.py` 191 insertions、27 deletions。`python3 scripts/agents/copy-originals.py check <草稿>`：「清单 6 份：改过 5 份、不在了 0 份、没改 1 份」（没改的是 `lib_three_way_legs.py`）。

撤回：反向规格在 `<草稿>/hook-swap/`，按时刻倒着依次跑（swapH `reverse-20261023T094233Z-2227831.json`、swapG `…093801Z-1260539`、swapF `…093650Z-1131316`、swapE `…093350Z-4017070`、swapD `…093251Z-3936288`、swapC `…092618Z-2222639`、swapB `…090630Z-557743`、swapA `…090432Z-498198`）。钩子规格写 `--hook` 与 `--feeds <草稿>/feeds-guard-neutral.json`（完工清理闸用 `feeds-cleanup.json`），脚本规格写 `--verify`。另外三份 `…090340Z`、`…090410Z`、`…090623Z` 是 `--dry-run` 写的，不用。

开工时的负载（`<草稿>/ps-start.txt`）：负载 63，主 agent 的 checker 档 cargo 与 crash-point-replay、几份 `verus-proof-check.py --selftest`、`code-tooling.sh --check …` 在跑；中途到过 103。只跑轻的自证与复跑，全部本机 `nice -n 19`，没有别的机器要用；复跑派发闸开关按 8 份分开跑（每份前台 25 到 45 秒），没等锁。

## 逐条：改了什么、样本红绿、自证格与弄坏开关

### 第 1 条：⑰ 前缀名的扩展名照工具层不分大小写

- 改法：`REPORT_REFUSED_PREFIX_NAME` 改成 `(?i:summary|findings|analysis)[^/]*\.(?i:md)`；两处调用（报告行那一判、三方腿本腿报告那一判）改调新函数 `report_name_refused_prefix(name)`。`.markdown` 照旧不拦（`fullmatch` 要以 `.md` 收尾）。文件头 ⑰ 条与常量上面的注释写明出处 `b12-scratch-lib.md`「6.1 工具层真调」。
- 自证格（新加 2 格）：
  - 红：「报告名:以 SUMMARY 开头、扩展名写成大写 .MD 的照拒（工具层同样拒）」应得 2。
  - 绿：「报告名:以 summary 开头的 .markdown 放行（工具层放）」应得 0。
- 弄坏开关 `RUNNER_DISPATCH_GUARD_BREAK=report-name-prefix-extension-lowercase`（扩展名退回只认小写）。打开后原样：

```
  ✗ 自检：报告名:以 SUMMARY 开头、扩展名写成大写 .MD 的照拒（工具层同样拒） 应当返回 2，实际 0
```

- 喂法（`<草稿>/feeds-guard.json`，stdin 入口，改前 `--feed-only` 原样，`<草稿>/feeds-before.txt`）：

```
  ✗ 退 0（应当 2） 工具员报告写到裸文件名 SUMMARY.MD 拒（扩展名大写，工具层同样拒） ← 
  ✓ 退 0（应当 0） 工具员报告写到裸文件名 summary.markdown 放行（工具层放） ← 
```

  换上之后草稿里与主工作区里各喂一遍，都对（`<草稿>/swapA.txt`）。

### 第 2 条：⑰ 认 `$SINGLEFS_WORK_ROOT/` 起头的三方腿报告路径

- 现查的病因：⑰ 的文件名那一判早就经 `lib_work_root.py` 的 `expand_written_path` 展开（`own_report_token`），认不出的是三方腿形态那一判：`lib_three_way_legs.py` 的 `prefix_tree` 只展开 `~` 与 `$HOME`，前缀里还带 `$` 的不认。
- 改法：派发闸里新加 `work_root_tokens_expanded(text)`，把 `$SINGLEFS_WORK_ROOT/`、`${SINGLEFS_WORK_ROOT}/` 逐处交 `lib_work_root.expand_written_path` 换成工作根（与交回闸 ② 同一种解法），`leg_report_reading` 开头先过它再认；库没改。文件头 ⑰ 条写明。
- 自证格（新加 3 格；工作根临时设成样本仓的上一层，`$SINGLEFS_WORK_ROOT/<样本仓目录名>/` 展开后就是样本仓本身这一棵树，不靠本机有哪些工作树）：
  - 绿：「三方报告:报告行写 $SINGLEFS_WORK_ROOT/ 起头、照 lib_work_root.py 展开后解到本仓一棵树的报告路径，放行」。
  - 绿：「三方报告:整份提示里写 ${SINGLEFS_WORK_ROOT}/ 起头、展开后解到本仓一棵树的报告路径，放行」。
  - 红：「三方报告:报告行写 $SINGLEFS_WORK_ROOT/ 起头、展开后不在本仓任何一棵树里的，拒」。
- 弄坏开关 `RUNNER_DISPATCH_GUARD_BREAK=leg-report-work-root-token-kept`（不展开）。打开后原样：

```
  ✗ 自检：三方报告:报告行写 $SINGLEFS_WORK_ROOT/ 起头、照 lib_work_root.py 展开后解到本仓一棵树的报告路径，放行 应当返回 True，实际 False
  ✗ 自检：三方报告:整份提示里写 ${SINGLEFS_WORK_ROOT}/ 起头、展开后解到本仓一棵树的报告路径，放行 应当返回 True，实际 False
```

- 真环境喂法：`报告：$SINGLEFS_WORK_ROOT/m5-dev/research/prompts/x-r1-opus-output.md`（`m5-dev` 是 `git worktree list` 里登记的那一棵）改前「✗ 退 2（应当 0）」，换上之后「✓ 退 0（应当 0）」；`…/hook-swap-feed-b12-not-a-tree/…` 前后都退 2。这一条不放进 hook-swap 的喂法：草稿树里 `git worktree list` 列不出 `m5-dev`，草稿里验会红（`<草稿>/swapA-dry.txt`），换上之后单独 `--feed-only` 喂了（`<草稿>/feeds-guard-m5dev.json`）。

### 第 3 条：⑯「草稿目录：」一行两种围栏认法遮不遮不一致就拒

- 改法：`writer_lock_verdict` 在建锁、判锁之前先调新函数 `scratch_line_fence_ambiguity(prompt)`：单看一行 `scratch_line_values` 认得出路径的「草稿目录：」行，比库的 `fenced_line_numbers`（照 CommonMark，没收尾的遮到文末）与派发闸已有的 `hidden_lines_by_wide_line_rule`（找不到关块的那一行不开块）遮不遮；不一致就拒，`--check-prompt` 同样拒。拒绝说明沿用 `code_block_ambiguity_refusal`（加了两个可选参数换掉「规矩」「怎么办」两行），逐行写两种认法各怎么认，出路是「把围栏写规整：每个围栏都补上收尾行…；或者把「草稿目录：」一行挪到所有围栏之前」。两个方向都拒，照 ⑭ ⑮。文件头 ⑯ 条写明。
- 真提示上先数过（`<草稿>/tools/scratch_fence_corpus.py`，读数 `<草稿>/scratch-fence-corpus.txt`）：

```
提示 4026 份；有「草稿目录：」行（单看一行认得出路径）的 2374 份
库遮、逐行认法不遮的行 0 处，涉及 0 份
逐行认法遮、库不遮的行 0 处，涉及 0 份
```

- 自证格（新加 6 格，经 stdin 入口真派，工作根指自检自己的临时目录）：
  - 红：「写者锁:「草稿目录：」一行照 CommonMark 被没收尾的围栏遮住、照逐行配围栏不遮，拒」应得 2。
  - 红：「写者锁:两种围栏认法不一致的拒绝说明逐行写两种认法、叫把围栏写规整」。
  - 红：「写者锁:「草稿目录：」一行照逐行配围栏被缩进 4 格的围栏遮住、照 CommonMark 不遮，拒」应得 2。
  - 红：「写者锁:--check-prompt 同样拒两种围栏认法不一致的「草稿目录：」一行」应得 2。
  - 「写者锁:两种围栏认法不一致时不建锁」应得 False。
  - 绿：「写者锁:示例行在闭合了的围栏里、真的那一行在围栏外，两种认法一致，放行并只在围栏外那一个目录建锁」应得 (0, True, False)。
- 弄坏开关 `RUNNER_DISPATCH_GUARD_BREAK=writer-lock-fence-ambiguity-unchecked`（不比）。打开后原样：

```
  ✗ 自检：写者锁:「草稿目录：」一行照 CommonMark 被没收尾的围栏遮住、照逐行配围栏不遮，拒 应当返回 2，实际 0
  ✗ 自检：写者锁:两种围栏认法不一致的拒绝说明逐行写两种认法、叫把围栏写规整 应当返回 True，实际 False
  ✗ 自检：写者锁:「草稿目录：」一行照逐行配围栏被缩进 4 格的围栏遮住、照 CommonMark 不遮，拒 应当返回 2，实际 0
  ✗ 自检：写者锁:--check-prompt 同样拒两种围栏认法不一致的「草稿目录：」一行 应当返回 2，实际 0
```

- 喂法：「「草稿目录：」一行被没收尾的围栏遮住、照逐行配围栏不遮，拒（不建锁）」改前「✗ 退 0（应当 2）」，换上之后退 2；工作根下没有因喂法建出的目录或锁（`ls ~/.local/share/singlefs/work | grep -c hook-swap-feed` 为 0）。

### 第 4 条：`lib_prompt_scratch_line.py` 文件头调用方表

现数（`grep -rn lib_prompt_scratch_line` 在 `scripts`、`.claude/hooks`、`.claude/gate.d`、`.claude/scripts` 下，挑真 import / 载入它的）：`runner-dispatch-guard.sh:752`、`handback-guard.sh:401`、`completion-cleanup-guard.sh:223`、`scripts/agents/agent-watch.py:552`、`scripts/agents/agent-cost.py:112`、`scripts/agents/dispatch-prompt.py:266`，共 6 份；`.claude/gate.d/code-tooling.sh:484` 只是在 runner 表里起它的 `--selftest`，不是调用方。原表头一句写「四处都调它」、表题「五处用这一份」、表里列 4 份。改成「六份文件都调它」「六份文件用这一份」，补 `dispatch-prompt.py`（骨架 `--check` 的认领行那一判，`scratch_line_values`）与 `agent-cost.py`（账表每个子 agent 的草稿目录，`scratch_line_values`）两行；派发闸那一行补「⑯ 另用 `fenced_line_numbers` 比两种围栏认法」，完工清理闸那一行补「写相对路径的按仓根解，用 `repository_root`」。只改文件头，函数没动；库的自证 39 格照旧绿。

没改：「认法」一段里「没有那一行的提示，五处各按自己的规矩办」那一句（它列的是没有那一行时各处怎么办，不是调用方表）；要不要跟着改成六份、补骨架与账表各自怎么办，交主 agent 定。

### 第 5 条：完工清理闸按仓根解相对路径

- 改法：`subagent_roots` 取 `scratch_line_values` 之后，展开了还不是绝对路径的拼在 `lib_prompt_scratch_line.repository_root()` 后面，不再交 `os.path.abspath` 按钩子当前目录解。文件头「判哪几处」写明。
- 自证格（新加 2 格，场景 `subagent-assigned-relative`）：
  - 红：`subagent-assigned-relative-by-repository-root`：当前目录切到场景里一个比仓根深的仓外目录，提示写「草稿目录：<从仓根到工作根下那个目录的相对路径>」，里面的仓副本要拦下。
  - 绿：`subagent-assigned-relative-not-by-cwd`：当前目录切到工作根，提示写「草稿目录：assigned-by-cwd」，按仓根解不在工作根下，工作根下同名目录里的仓副本不算这个子 agent 的。
- 弄坏开关 `COMPLETION_CLEANUP_BREAK=prompt-scratch-relative-by-cwd`（退回按当前目录解；表 `BREAK_TABLE` 与文件头各加一行）。打开后原样（截在 120 字）：

```
✗ 自证 subagent-assigned-relative-by-repository-root：「草稿目录：../../../../tmp/completion-cleanup-selftest
✗ 自证 subagent-assigned-relative-not-by-cwd：「草稿目录：assigned-by-cwd」按仓根解不在工作根下，
```

- 自证：`bash .claude/hooks/completion-cleanup-guard.sh --selftest` 末行「✓ completion-cleanup-guard.sh 自证：各格都对（判了 45 格）；28 个弄坏开关各红在点名的格」（改前 43 格、27 个）。旧开关 `prompt-scratch-ignored` 因为新格多红一格，文件头补点名两格（新格与原来就没点名的 `subagent-assigned-shared-reading`），复跑这两条开关逐格相同。

### 第 6 条：三方腿派发提示里没有「报告：」一行的有几份

- 骨架落地的时刻取会话记录里头一次把 `THREE_WAY_REPORT_PLACEHOLDER = …` 写进 `scripts/agents/dispatch-prompt.py` 的那次 Edit（子 agent `a303d33b0ccd5e68f`，2026-10-22）。主会话记录里全部 Agent / Task 调用按 tool_use id 去重 4026 份，其中类型是三方腿（定义 frontmatter 登记 `leg: three-way` 的 three-way-attack、three-way-defense、three-way-forward）、派在那之后的 34 份。行首（可带 `- * > _`）「报告：」一行的认法照派发闸。读数 `<草稿>/leg-report-lines.txt`，工具 `<草稿>/tools/count_leg_report_lines.py`。
- **没有「报告：」一行的 3 份**，不是 0，所以 ⑰ 对三方腿要求「报告：」一行这一改没做，逐份交主 agent：

| tool_use id | 日期 | 类型 | 轮 | 报告怎么写的 |
|---|---|---|---|---|
| toolu_01Uj6Z3xVYScCRiXp1iCfWii | 2026-10-22 | three-way-attack | m3-close-c506-r3 | 句中「报告写进 `research/prompts/m3-close-c506-r3-opus-output.md`（第一段排他新建…）」 |
| toolu_013gxFURrRQssnrNkEUyBRnn | 2026-10-22 | three-way-attack | m3-close-reuse-closure-r2 | 句中「报告写进 `research/prompts/m3-close-reuse-closure-r2-opus-output.md`」 |
| toolu_014hpAFjxKMyU57dTYavw3Mo | 2026-10-22 | three-way-attack | m3-close-reuse-closure-r3 | 句中「报告写进 `research/prompts/m3-close-reuse-closure-r3-opus-output.md`」 |

  三份都在同一个主会话 `59734d6f-…`，禁读清单只列正推腿的报告与模型目录。拿今天的派发闸 `--check-prompt <提示> --type three-way-attack` 逐份核，三份都退 0：⑰ 把「报告写进」当报告行认（标签本身不在反引号里），点名的就是本腿的报告。剩下 31 份都有行首「报告：」一行。四种报告标签都没有的三方腿提示：0 份（推的：由上面 31 份加 3 份得出，没单独再数）。
- 要主 agent 定：要「报告：」一行就得先接受这 3 份写法将来被拒（或连同「报告写进」等四种标签都认）；推的弱处（辩方腿没写报告行时，正文点名同一轮别的腿报告当阅读材料、不在禁读清单里的会被当成本腿的报告）这一批没堵。

### 第 7 条：开关点名补到与实际红的格逐格相同

读法照门禁 code-tooling 的 break-switch-replay 格：开关表与「」项的读法从 `.claude/gate.d/code-tooling.sh` 的 `BREAK_SWITCH_TABLE_LIBRARY` 现取，其余 `_BREAK` 变量与 `SINGLEFS_HEAVY_TESTS` 摘掉，设上一条起 `--selftest`，点名的项按 `…` 切段、要按次序落在同一条 ✗ 行里。另比一样：每一条 ✗ 行都要被某一项照到。工具 `<草稿>/tools/replay.py`，补项工具 `<草稿>/tools/add_items.py`（派发闸的项写到「，实际」为止；骨架的项照原来的样子写到「实际」为止，登记值 31 与认领行里填的目录换成 `…`，免得跟着登记值与样本目录变）。

派发闸改前（`<草稿>/replay-guard-base-*.jsonl`，112 条）没点名的正是 `b12-dispatch.md` 列的 19 条；这一批的新格又让 6 条原来逐格相同的多红了几格。25 条补了 153 项，补完复跑（`<草稿>/replay-guard-final-p1…p8.jsonl`）：

```
total 115 exact 115 missing>0 0 unnamed>0 0 exit0 0 items 1499 rows 1499
```

派发闸逐条（项数 / 失败行数；「改前」是开工时、「中间」是第 1 到 3 条换上之后、「补完」是交回时）：

| 开关（`RUNNER_DISPATCH_GUARD_BREAK=` 省去） | 改前 | 中间 | 补完 | 哪一类 |
|---|---|---|---|---|
| disable-heavy-test | 4/6 | 4/6 | 6/6 | 19 条之一 |
| disable-snapshot | 16/57 | 16/57 | 57/57 | 19 条之一 |
| inputs-ignored | 3/5 | 3/5 | 5/5 | 19 条之一 |
| report-name-whole-path | 1/5 | 1/6 | 6/6 | 19 条之一 |
| placeholders-ignored | 1/13 | 1/13 | 13/13 | 19 条之一 |
| runner-backfill-refused | 6/9 | 6/9 | 9/9 | 19 条之一 |
| scope-token-unexpanded | 3/4 | 3/4 | 4/4 | 19 条之一 |
| snapshot-loose-lines | 15/16 | 15/16 | 16/16 | 19 条之一 |
| snapshot-unreadable-aborts | 2/4 | 2/4 | 4/4 | 19 条之一 |
| snapshot-unreadable-denies | 2/6 | 2/6 | 6/6 | 19 条之一 |
| valve-reason-ignored | 34/39 | 34/39 | 39/39 | 19 条之一 |
| long-run-duplicate-label-first-wins | 6/8 | 6/8 | 8/8 | 19 条之一 |
| check-mode-registers | 1/2 | 1/2 | 2/2 | 19 条之一 |
| snapshot-abandoned-unchecked | 31/34 | 31/34 | 34/34 | 19 条之一 |
| snapshot-abandoned-line-unread | 15/16 | 15/16 | 16/16 | 19 条之一 |
| writer-lock-unchecked | 8/13 | 8/18 | 18/18 | 19 条之一（第 3 条的新格让它再多红 5 格） |
| LIB_LONG_RUN_BREAK=hosts-unchecked | 21/22 | 21/22 | 22/22 | 19 条之一 |
| LIB_LONG_RUN_BREAK=hosts-canonical-unchecked | 20/21 | 20/21 | 21/21 | 19 条之一 |
| SINGLEFS_WORK_ROOT_BREAK=new-root-unknown | 4/44 | 4/48 | 48/48 | 19 条之一 |
| disable-check | 331/331 | 331/337 | 337/337 | 这一批让它多红 |
| three-way-report-unchecked | 24/24 | 24/25 | 25/25 | 这一批让它多红 |
| report-name-prefixes-unchecked | 6/6 | 6/7 | 7/7 | 这一批让它多红 |
| report-name-unchecked | 38/38 | 38/39 | 39/39 | 这一批让它多红 |
| report-line-position-anchored | 25/25 | 25/26 | 26/26 | 这一批让它多红 |
| thread-limit-any-refused | 137/137 | 137/139 | 139/139 | 这一批让它多红 |

骨架 `scripts/agents/dispatch-prompt.py`（7 条补了 98 项；`<草稿>/replay-dp-base.jsonl` → `<草稿>/replay-dp-final.jsonl`）：

| 开关 | 改前 项/行（没点名） | 补完 项/行（没点名） |
|---|---|---|
| DISPATCH_PROMPT_BREAK=guard-skipped | 1/15（14） | 15/15（0） |
| DISPATCH_PROMPT_BREAK=claim-line-dropped | 2/44（42） | 24/44（0） |
| DISPATCH_PROMPT_BREAK=types-before-nine | 1/9（8） | 9/9（0） |
| DISPATCH_PROMPT_BREAK=thread-formula-restored | 2/31（29） | 31/31（0） |
| DISPATCH_PROMPT_BREAK=three-way-report-default-dropped | 4/14（6） | 10/14（0） |
| RUNNER_DISPATCH_GUARD_BREAK=placeholders-ignored | 1/6（3） | 4/6（0） |
| RUNNER_DISPATCH_GUARD_BREAK=thread-limit-any-refused | 1/19（16） | 17/19（0） |

骨架补完的复跑原样：「switches 19; exact 12; missing>0 0; unnamed>0 0; count-differs 7; exit0 0」。7 条项数与行数不等的（上表 4 条，加原来就这样的 claim-check-skipped 2/6、selftest-location-dropped 3/6、three-way-report-unchecked 1/3），都是失败行都被点名、点名的都照出来了，只是一项照到几行：骨架一格失败时把派发闸输出带进失败行，同一句派发闸的话（「✗ 认领行：有「草稿目录：…」却没有「认领：…」一行」17 处同文）在几格里各出现一次，文件头原有的「三种各一句」写法也是一项照三行。逐格相同按「点名没照出 0、失败行没点名 0」判。

### 第 8 条：`snapshot-tree.py` 照搬迁登记换回旧路径再倒推

- 改法：读 `<仓根>/.claude/gate.d/path-moves.tsv`，读法与换法都调 `.claude/gate.d/lib-lag-baseline.py` 的 `path_moves`、`rewrite_moved_paths`、`rewrite_moved_bytes`（登记每一行新旧对调、次序倒排），不另写一份换法。照原样倒推不出或 sha256 对不上时再试一次：
  - diff 里有它：这一份的那一段逐行去掉行首 `+ - 空格` 记号后换回旧路径（`rewrite_moved_paths` 认前缀边界时 `-` 紧挨着路径不算路径开头，所以要先去掉记号），再取基底、再打；
  - diff 里没有它：此刻 HEAD 那一份换回旧路径再比；
  - 清单点名的路径在 diff 与 HEAD 里都找不到：照登记往前、往后各换一次路径再找。
  换回之后 sha256 对上的才铺进树，铺在清单点名的路径上，说明里写「照搬迁登记换回旧路径（.claude/gate.d/path-moves.tsv 第 N 行…）」，汇总行写「对上的里有 N 份照搬迁登记换回旧路径」；照原样对上的不换。登记表写坏了、库载不进的打一行提示、照原样倒推。文件头写明。
- 自证格（新加 1 个场景、2 项：「搬迁之后的旧轮」退 0 并点名每份的来源与登记行号，「树里的 tools/new-name/run.sh」是快照那一刻的内容）：临时仓里提交基底，造快照（改一份上下文行带旧路径的、改一份本身要搬家的、删一行「-tools/…」紧挨着记号的、留一份没改而正文带旧路径的），再 `git mv` 加全仓把旧路径换成新路径（快照目录里的清单与 diff 也换）、登记一行、提交。`--selftest` 23 项（改前 21 项）全绿。
- 弄坏开关（`SNAPSHOT_TREE_BREAK=`）：
  - `path-moves-ignored`（不读登记表）：「✗ 自检：搬迁之后的旧轮：应当退 0 …」「✗ 自检：搬迁之后的旧轮：树里的 tools/new-name/run.sh 应当是快照那一刻的内容（里面还是旧路径）」两格红，复跑 7 项对 7 行。
  - `diff-markers-kept`（diff 整段照换、不先去掉记号）：「✗ 自检：搬迁之后的旧轮：应当退 0 …」红，失败行里带「✗ kb/remove.md  diff 打不上」，3 项对 3 行。
  - 旧开关 `skip-sha-check`、`head-only` 因新格多红的那几行文件头补了点名；`head-only`、`no-unquote`、`index-diff-ignored` 另有原来就没点名的嵌套输出行（r1 到 r4 那几趟的逐份 ✗ 行，19、8、4 行），不在这一批的开关名单里，没补。`snapshot-tree.py` 在 `.claude/gate.d/break-switch-replay-files.tsv` 里，复跑 7 条开关点名没照出 0。

#### 拿各轮跑的读数（`python3 scripts/agents/snapshot-tree.py <轮> --out <草稿>/snap-after/<轮>-tree`，原样输出在 `<草稿>/snap-after/<轮>.txt`，改前在 `<草稿>/snap-base/<轮>.txt`）

| 轮 | diff | 清单点名 | 改前 倒推出 / 对上 | 改后 倒推出 / 对上 | 其中照搬迁登记换回 | 改后倒推不出 / 对不上 |
|---|---|---|---|---|---|---|
| landing-audit-r1 | 对暂存区 | 51 | 16 / 13 | 30 / 30 | 17 | 21 / 0 |
| landing-audit-r2 | 对暂存区 | 63 | 25 / 24 | 36 / 36 | 12 | 27 / 0 |
| landing-audit-r3 | 对暂存区 | 74 | 38 / 36 | 46 / 46 | 10 | 28 / 0 |
| landing-audit-r4 | 对 HEAD | 80 | 68 / 64 | 78 / 77 | 13 | 2 / 1 |
| landing-audit-r5 | 对 HEAD | 27 | 17 / 14 | 27 / 26 | 12 | 0 / 1 |
| landing-audit-r6 | 对 HEAD | 40 | 40 / 40 | 40 / 40 | 0 | 0 / 0（退 0） |
| apparatus-rename-r1 | 对 HEAD | 14 | 12 / 12 | 12 / 12 | 0 | 2 / 0 |
| apparatus-rename-r2 | 不是 git 格式 | 15 | 0 / 0 | 0 / 0 | 0 | 15 / 0 |

r6 汇总行原样：「✓ 快照 landing-audit-r6：清单点名 40 份，倒推出 40 份，sha256 对上 40 份（基底取自 HEAD 19 份、diff 头记的 blob 0 份、新文件 21 份；diff 用的是 judged-vs-head.diff）」。

#### 剩下对不上的，逐份为什么（逐份清单在文末附录）

1. **快照那一刻是暂存了、还没提交的新文件**（r1 21 份、r2 27 份、r3 27 份、apparatus-rename-r1 的 `.claude/gate.d/path-moves.tsv`）：对暂存区的 diff 只记工作区对暂存区，暂存区里那一版不在 diff 里，HEAD 里也没有（哪次提交里都没有）。不是搬迁造成的。其中此刻工作区那一份 sha256 与快照相同的：r1 11 份、r2 17 份、r3 19 份、apparatus-rename-r1 1 份；r1 有 3 份（`replay-comparison-units-lag.tsv` 三处）工作区里也不在了。
2. **搬迁那一趟换了登记前缀认不到的写法**（这一种照登记反着换换不回来）：
   - `.claude/hooks/handback-guard.sh`（r4 对不上、r5 对不上）：diff 的 `+` 行里 `os.path.join(work, "scripts", "experiments", "e1_probe.rs")` 被换成了 `"scripts", "experiment-apparatus"`，拆成几段的写法不是「scripts/experiments」这个前缀（`.claude/rules/path-moves.md` 第 1 步点名的「拆成几段的写法」）。
   - `.claude/gate.d/doc-experiments.sh`、`scripts/kb/claim-experiment.sh`（r4 倒推不出）：`+` 行里 `$'…\nscripts/experiments\t…'` 这种字符串里，路径前面紧挨着 `\n` 的 `n`，按前缀边界不算路径开头，搬迁那一趟换了、反着换换不到。
   核过：这三份 diff 头记的结果 blob（`5e5825882`、`fdde9ae05`、`e01063938`、`79dc2f1d3`）都还在对象库里，sha256 与快照清单相同（核法：`git cat-file blob <结果 blob>` 算 sha256 与清单比；handback-guard.sh 那一份另用 `git apply` 打在基底上与结果 blob 逐行比过，只差上面那一行，`<草稿>/diag/hb-r4-*`）。
3. **对暂存区的旧轮里暂存区另有改动**（r3 `.claude/kb/checks-owed.md`，推的）：diff 里没有它（工作区与暂存区相同），HEAD 那一份是快照之后的提交，暂存区那一版没留下。
4. **apparatus-rename-r1 的 `scripts/experiment-apparatus/e171_three_oracle_collusion/frozen_o3.py`**：新路径那一份哪次提交里都没有，HEAD 里只有搬迁之前旧路径那一份；换回旧路径之后内容也不同，快照那一刻它在搬家之外另有改动，diff（12 段）里又没有它那一段（推的：快照时 diff 没取到这一份）。
5. **apparatus-rename-r2 整轮**：`judged-vs-head.diff` 不是 `git diff` 格式（每段是 `--- /dev/fd/63 … [<路径>]`、`+++ <路径> … [<路径>]`，没有 `diff --git` 头与 `index` 行），一段都认不出，15 份都按「diff 里没有它」取 HEAD：7 份快照之后被提交改过，8 份是暂存了没提交的。不是搬迁造成的，这一批没加这种格式的读法。

要主 agent 定的（不在这一批）：第 2 种里结果 blob 都在对象库、sha256 对得上，「diff 头记的结果 blob 在对象库里、sha256 与清单相同就取它」能把 r4、r5 剩下的这几份补上；第 1 种工作区里 sha256 还对得上的可以取工作区那一份。两样都是新的取法，派发没要，没做。

## 自证（交回时在主工作区跑，都登记为 light，`nice -n 19`）

| 命令 | 退出码 | 末行（截在头一段） |
|---|---|---|
| `bash .claude/hooks/runner-dispatch-guard.sh --selftest` | 0 | ✓ 自检通过（查了 565 种）（改前 554 种） |
| `python3 .claude/hooks/lib_prompt_scratch_line.py --selftest` | 0 | ✓ lib_prompt_scratch_line 自检通过（查了 39 格） |
| `bash .claude/hooks/completion-cleanup-guard.sh --selftest` | 0 | ✓ completion-cleanup-guard.sh 自证：各格都对（判了 45 格）；28 个弄坏开关各红在点名的格 |
| `python3 scripts/agents/dispatch-prompt.py --selftest` | 0 | ✓ 自检通过（查了 95 项） |
| `python3 scripts/agents/snapshot-tree.py --selftest` | 0 | ✓ snapshot-tree 自检通过（查了 23 项）（改前 21 项） |
| `bash .claude/hooks/handback-guard.sh --selftest`（调同一份库，只复跑） | 0 | ✓ handback-guard 自检通过（查了 117 种） |
| `python3 scripts/agents/agent-cost.py --selftest`（同上） | 0 | ✓ 自检通过（查了 29 项，其中 outcome 的 21 个子 agent） |
| `python3 scripts/agents/agent-watch.py --selftest`（同上，只读） | 0 | ✓ 自检：临时目录在通过、判红、抛异常三条路上都删了 |

原样在 `<草稿>/final-*.txt`。语法：`bash -n` 两份钩子过；三份 `.py` 用 `py_compile` 编进草稿目录（编完删掉），过；hook-swap 每次在草稿里和主工作区里都另做了 `bash -n` 与内嵌 python 编译。

新开关逐条打开各红在点名的格（上面各条贴了原样）；复跑读数：派发闸 115 条全部逐格相同，骨架 19 条点名没照出 0、失败行没点名 0，完工清理闸 `prompt-scratch-ignored`、`prompt-scratch-relative-by-cwd` 两条逐格相同，`snapshot-tree.py` 7 条点名没照出 0（新开关两条逐格相同）。

## 新写的 `# gate-similar:` / `# hook-events:` 行

无。没有新建门禁阶段或钩子：判据都追加进了已有的派发闸、完工清理闸与 `snapshot-tree.py`。`python3 .claude/singlefs-ai-sop/scripts/gate-overlap.py --list`（`<草稿>/gate-overlap-list.txt`）里管派发提示的只有派发闸一份（第 53 行），管完工清理的只有完工清理闸（第 46 行）。

## 第二种活改过的定义、共用约束与规则

无。没改 `.claude/agents/`、`.claude/agent-common.md`、`.claude/main-agent.md`、`.claude/rules/`，不用开定义三方。注意共用约束「执行前拒绝的写法」派发闸那一条（`.claude/agent-common.md` 第 90 行）写了 ⑯ 只在「持有者还在跑」时拒，第 3 条新加的「两种围栏认法不一致」那一种拒没写进共用约束；要不要补，交主 agent 定（改共用约束走定义三方）。

## 要主 agent 跑的命令

提交前归门禁、这一批没跑的几格（子 agent 不跑门禁阶段与 lint）：

- `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 break-switch-headers、break-switch-replay、break-switch-outside-selftest 三格; 范围=3 格; 为什么=派发闸加了 3 条开关、完工清理闸 1 条、snapshot-tree.py 2 条，开关表补了点名' bash .claude/gate.d/code-tooling.sh --check break-switch-headers,break-switch-replay,break-switch-outside-selftest <仓根>`：看 break-switch-replay 一行里派发闸、完工清理闸、`snapshot-tree.py`、`lib_prompt_scratch_line.py` 四份没有「点名的项没在失败行里」。派发闸单跑一次自证在负载 85 时约 25 秒，登记表给每条 120 秒。
- 门禁 code-tooling 的 research-script-selftests 格（`snapshot-tree.py` 与 `dispatch-prompt.py` 在 runner 表里），与 gate-overlap、hooks-registered（钩子注册没动）。
- 命名检查不用跑：没改 `.rs`。

## 没做什么

- 没跑重型测试（`checker-tier-crash-point-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification、checker-tier-research-build-and-replay 本身、checker-tier-broad-mutation 与 `scripts/mutation/broad-mutation.py` 起 cargo 的用法、`gate.sh`、全量 `cargo test`、`check.sh`）；没走定义三方；没提交。
- 没跑门禁阶段与各类 lint（见上一节）。
- 第 6 条只数了、没改判法（3 份不是 0）。
- 没改 `lib_three_way_legs.py`：第 2 条照交回闸那样包在调库的外面展开。
- 没碰 `.claude/hooks/handback-guard.sh`、`bash-command-detector.sh`、`continuation-guard.sh`、`heavy-test-guard.sh`、`scripts/agents/agent-watch.py`、`scripts/agents/watch.sh`、`.claude/gate.d/` 下的文件（只读、只起它们的自证）。
- 第 8 条没加「取 diff 头记的结果 blob」「取工作区那一份」两种新取法，也没加 apparatus-rename-r2 那种非 git 格式 diff 的读法（理由与读数见第 8 条）；`snapshot-tree.py` 的 `head-only`、`no-unquote`、`index-diff-ignored` 三条开关原来就有的嵌套输出行没补点名。
- 完工清理闸另有 12 条开关点名比实际红的格少（`build-directory-unrecognized` 等，`<草稿>/replay-cleanup.txt`），不在这一批的名单里，没补。
- 库文件头「认法」一段「五处各按自己的规矩办」那一句没改（第 4 条）。
- 中途没有收到主 agent 的消息。

## 清理

- 删了：铺出的快照树 `snap-base/*-tree`（合计约 15M）、`snap-after/*-tree`（约 29M）、`diag/apply-hb`（打补丁核对用的小树），以及抽出来的派发提示语料 `dispatches.jsonl`（约 48M，重抽用 `python3 <草稿>/tools/extract_dispatches.py <输出>`）。hook-swap 的草稿树它自己删了。
- 留着的都是文本：`originals/`（6 份改前原文与清单）、`draft/`（与主工作区逐字相同的最终全文，外加 `*.after-swapA.sh`、`*.before-items.py` 两份中间版）、`hook-swap/`（喂法与验证输出、反向规格）、`spec-*.json`、`feeds-*.json`、`replay-*.jsonl`、`final-*.txt`、`switch-*.txt`、`snap-base/*.txt`、`snap-after/*.txt`、`leg-missing/`（第 6 条那 3 份提示原文与 `--check-prompt` 输出）、`diag/hb-r4-*`、`tools/`。没有仓副本、编译目录或库拷贝；草稿目录交回时 3.7M（`du -sh`）。

## 附：第 8 条剩下没对上的逐份清单（括号里是此刻的样子：哪次提交里有没有、工作区里在不在、sha256 与快照比）

landing-audit-r1：
- `.claude/gate.d/replay-comparison-units-lag.tsv`（哪次提交里都没有、工作区里不在）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `.claude/gate.d/replay-comparison-units.tsv`（哪次提交里都没有、工作区里在、工作区那一份已改）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `.claude/gate.d/fixtures/doc-experiments.sh/replay-comparison-units-green/.claude/gate.d/replay-comparison-units-lag.tsv`（哪次提交里都没有、工作区里不在）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `.claude/gate.d/fixtures/doc-experiments.sh/replay-comparison-units-green/.claude/gate.d/replay-comparison-units.tsv`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `.claude/gate.d/fixtures/doc-experiments.sh/replay-comparison-units-green/crates/sample/units.rs`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `.claude/gate.d/fixtures/doc-experiments.sh/replay-comparison-units-green/expect`（哪次提交里都没有、工作区里在、工作区那一份已改）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `.claude/gate.d/fixtures/doc-experiments.sh/replay-comparison-units-green/.gate-cells`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `.claude/gate.d/fixtures/doc-experiments.sh/replay-comparison-units-green/research/prompts/e991-registration.md`（哪次提交里都没有、工作区里在、工作区那一份已改）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `.claude/gate.d/fixtures/doc-experiments.sh/replay-comparison-units-green/scripts/experiment-apparatus/replay.sh`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有（照搬迁登记换过的路径也没有）
- `.claude/gate.d/fixtures/doc-experiments.sh/replay-comparison-units-green/setup.sh`（哪次提交里都没有、工作区里在、工作区那一份已改）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `.claude/gate.d/fixtures/doc-experiments.sh/replay-comparison-units-red/.claude/gate.d/replay-comparison-units-lag.tsv`（哪次提交里都没有、工作区里不在）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `.claude/gate.d/fixtures/doc-experiments.sh/replay-comparison-units-red/.claude/gate.d/replay-comparison-units.tsv`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `.claude/gate.d/fixtures/doc-experiments.sh/replay-comparison-units-red/crates/sample/units.rs`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `.claude/gate.d/fixtures/doc-experiments.sh/replay-comparison-units-red/expect`（哪次提交里都没有、工作区里在、工作区那一份已改）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `.claude/gate.d/fixtures/doc-experiments.sh/replay-comparison-units-red/.gate-cells`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `.claude/gate.d/fixtures/doc-experiments.sh/replay-comparison-units-red/research/prompts/e991-registration.md`（哪次提交里都没有、工作区里在、工作区那一份已改）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `.claude/gate.d/fixtures/doc-experiments.sh/replay-comparison-units-red/scripts/experiment-apparatus/replay.sh`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有（照搬迁登记换过的路径也没有）
- `.claude/gate.d/fixtures/doc-experiments.sh/replay-comparison-units-red/setup.sh`（哪次提交里都没有、工作区里在、工作区那一份已改）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `.claude/gate.d/fixtures/doc-experiments.sh/replay-comparison-units-zero/.claude/gate.d/replay-comparison-units.tsv`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `.claude/gate.d/fixtures/doc-experiments.sh/replay-comparison-units-zero/expect`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `.claude/gate.d/fixtures/doc-experiments.sh/replay-comparison-units-zero/.gate-cells`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有

landing-audit-r2：
- `.claude/gate.d/fixtures/doc-experiments.sh/replay-comparison-units-green/.claude/gate.d/replay-comparison-units.tsv`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `.claude/gate.d/fixtures/doc-experiments.sh/replay-comparison-units-green/crates/sample/units.rs`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `.claude/gate.d/fixtures/doc-experiments.sh/replay-comparison-units-green/.gate-cells`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `.claude/gate.d/fixtures/doc-experiments.sh/replay-comparison-units-green/research/prompts/e991-registration.md`（哪次提交里都没有、工作区里在、工作区那一份已改）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `.claude/gate.d/fixtures/doc-experiments.sh/replay-comparison-units-green/scripts/experiment-apparatus/replay.sh`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有（照搬迁登记换过的路径也没有）
- `.claude/gate.d/fixtures/doc-experiments.sh/replay-comparison-units-red/.claude/gate.d/replay-comparison-units.tsv`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `.claude/gate.d/fixtures/doc-experiments.sh/replay-comparison-units-red/crates/sample/units.rs`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `.claude/gate.d/fixtures/doc-experiments.sh/replay-comparison-units-red/.gate-cells`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `.claude/gate.d/fixtures/doc-experiments.sh/replay-comparison-units-red/research/prompts/e991-registration.md`（哪次提交里都没有、工作区里在、工作区那一份已改）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `.claude/gate.d/fixtures/doc-experiments.sh/replay-comparison-units-red/scripts/experiment-apparatus/replay.sh`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有（照搬迁登记换过的路径也没有）
- `research/prompts/subagent-landing-audit-2026-10-22/blind-rework.md`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `research/prompts/subagent-landing-audit-2026-10-22/doc-experiments-cell-stop.md`（哪次提交里都没有、工作区里在、工作区那一份已改）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `research/prompts/subagent-landing-audit-2026-10-22/handoff-gaps.md`（哪次提交里都没有、工作区里在、工作区那一份已改）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `research/prompts/subagent-landing-audit-2026-10-22/investigator-sweep.md`（哪次提交里都没有、工作区里在、工作区那一份已改）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `research/prompts/subagent-landing-audit-2026-10-22/kb-scribe.md`（哪次提交里都没有、工作区里在、工作区那一份已改）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `research/prompts/subagent-landing-audit-2026-10-22/no-effect.md`（哪次提交里都没有、工作区里在、工作区那一份已改）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `research/prompts/subagent-landing-audit-2026-10-22/outcome-summary-2026-10-01-to-2026-10-21.txt`（哪次提交里都没有、工作区里在、工作区那一份已改）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `research/prompts/subagent-landing-audit-2026-10-22/outcome-summary-since-2026-10-01.txt`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `research/prompts/subagent-landing-audit-2026-10-22/r1fix-definitions.md`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `research/prompts/subagent-landing-audit-2026-10-22/r1fix-gates.md`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `research/prompts/subagent-landing-audit-2026-10-22/r1fix-hooks.md`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `research/prompts/subagent-landing-audit-2026-10-22/rework.md`（哪次提交里都没有、工作区里在、工作区那一份已改）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `research/prompts/subagent-landing-audit-2026-10-22/tw-definitions.md`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `research/prompts/subagent-landing-audit-2026-10-22/tw-gate-scripts.md`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `research/prompts/subagent-landing-audit-2026-10-22/tw-hooks.md`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `research/prompts/subagent-landing-audit-2026-10-22/tw-outcome-ledger.md`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `research/prompts/subagent-landing-audit-2026-10-22/writers.md`（哪次提交里都没有、工作区里在、工作区那一份已改）：倒推不出：diff 里没有它，此刻 HEAD 里也没有

landing-audit-r3：
- `.claude/kb/checks-owed.md`（提交过、工作区里在、工作区那一份已改）：倒推不出：diff 里没有它，此刻 HEAD 那一份 sha256 b34a284931af 不是快照的 1994b83d6228（快照之后有提交改过它，diff 里没有它的基底；照搬迁登记换回旧路径也对不上）
- `.claude/gate.d/fixtures/doc-experiments.sh/replay-comparison-units-red/.claude/gate.d/replay-comparison-units.tsv`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `.claude/gate.d/fixtures/doc-experiments.sh/replay-comparison-units-red/crates/sample/units.rs`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `.claude/gate.d/fixtures/doc-experiments.sh/replay-comparison-units-red/expect`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `.claude/gate.d/fixtures/doc-experiments.sh/replay-comparison-units-red/.gate-cells`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `.claude/gate.d/fixtures/doc-experiments.sh/replay-comparison-units-red/research/prompts/e991-registration.md`（哪次提交里都没有、工作区里在、工作区那一份已改）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `.claude/gate.d/fixtures/doc-experiments.sh/replay-comparison-units-red/scripts/experiment-apparatus/replay.sh`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有（照搬迁登记换过的路径也没有）
- `.claude/gate.d/fixtures/doc-experiments.sh/replay-comparison-units-red/setup.sh`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `research/prompts/subagent-landing-audit-2026-10-22/blind-rework.md`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `research/prompts/subagent-landing-audit-2026-10-22/doc-experiments-cell-stop.md`（哪次提交里都没有、工作区里在、工作区那一份已改）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `research/prompts/subagent-landing-audit-2026-10-22/handoff-gaps.md`（哪次提交里都没有、工作区里在、工作区那一份已改）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `research/prompts/subagent-landing-audit-2026-10-22/investigator-sweep.md`（哪次提交里都没有、工作区里在、工作区那一份已改）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `research/prompts/subagent-landing-audit-2026-10-22/kb-scribe.md`（哪次提交里都没有、工作区里在、工作区那一份已改）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `research/prompts/subagent-landing-audit-2026-10-22/no-effect.md`（哪次提交里都没有、工作区里在、工作区那一份已改）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `research/prompts/subagent-landing-audit-2026-10-22/outcome-summary-2026-10-01-to-2026-10-21.txt`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `research/prompts/subagent-landing-audit-2026-10-22/outcome-summary-since-2026-10-01.txt`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `research/prompts/subagent-landing-audit-2026-10-22/r1fix-definitions.md`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `research/prompts/subagent-landing-audit-2026-10-22/r1fix-gates.md`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `research/prompts/subagent-landing-audit-2026-10-22/r1fix-hooks.md`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `research/prompts/subagent-landing-audit-2026-10-22/r2fix-definitions.md`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `research/prompts/subagent-landing-audit-2026-10-22/r2fix-gates.md`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `research/prompts/subagent-landing-audit-2026-10-22/r2fix-hooks.md`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `research/prompts/subagent-landing-audit-2026-10-22/rework.md`（哪次提交里都没有、工作区里在、工作区那一份已改）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `research/prompts/subagent-landing-audit-2026-10-22/tw-definitions.md`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `research/prompts/subagent-landing-audit-2026-10-22/tw-gate-scripts.md`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `research/prompts/subagent-landing-audit-2026-10-22/tw-hooks.md`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `research/prompts/subagent-landing-audit-2026-10-22/tw-outcome-ledger.md`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `research/prompts/subagent-landing-audit-2026-10-22/writers.md`（哪次提交里都没有、工作区里在、工作区那一份已改）：倒推不出：diff 里没有它，此刻 HEAD 里也没有

landing-audit-r4：
- `.claude/hooks/handback-guard.sh`（提交过、工作区里在、工作区那一份已改）：sha256 对不上：倒推出 6403fe990e68、快照 bab11748f599（倒推出的也不是 diff 头记的结果 blob 5e5825882：diff 改坏了，或基底取错了；基底取自HEAD）
- `.claude/gate.d/doc-experiments.sh`（提交过、工作区里在、工作区那一份已改）：diff 打不上：git apply 退 1（error: patch failed: .claude/gate.d/doc-experiments.sh:282）；基底取自HEAD；照搬迁登记换回旧路径之后也不行：sha256 对不上：倒推出 d5fe0405251a、快照 7
- `scripts/kb/claim-experiment.sh`（提交过、工作区里在、工作区那一份已改）：diff 打不上：git apply 退 1（error: patch failed: scripts/kb/claim-experiment.sh:12）；基底取自HEAD；照搬迁登记换回旧路径之后也不行：sha256 对不上：倒推出 c8dbdbfd3ee5、快照 b4837

landing-audit-r5：
- `.claude/hooks/handback-guard.sh`（提交过、工作区里在、工作区那一份已改）：sha256 对不上：倒推出 b1677cd84560、快照 01fa3f9c87de（倒推出的也不是 diff 头记的结果 blob fdde9ae05：diff 改坏了，或基底取错了；基底取自HEAD）

apparatus-rename-r1：
- `.claude/gate.d/path-moves.tsv`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `scripts/experiment-apparatus/e171_three_oracle_collusion/frozen_o3.py`（哪次提交里都没有、工作区里在、工作区那一份已改）：倒推不出：diff 里没有它，此刻 HEAD 那一份 sha256 5add53ca56a2 不是快照的 a788555cacd6（快照之后有提交改过它，diff 里没有它的基底；照搬迁登记换回旧路径也对不上）

apparatus-rename-r2：
- `.claude/gate.d/lib-lag-baseline.py`（哪次提交里都没有、工作区里在、工作区那一份已改）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `.claude/gate.d/path-moves.tsv`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 里也没有
- `.claude/hooks/lib_long_run.py`（提交过、工作区里在、工作区那一份已改）：倒推不出：diff 里没有它，此刻 HEAD 那一份 sha256 9b5ab1f73cfd 不是快照的 66e8b4c30b03（快照之后有提交改过它，diff 里没有它的基底；照搬迁登记换回旧路径也对不上）
- `scripts/gate/changed-paths.sh`（提交过、工作区里在、工作区那一份已改）：倒推不出：diff 里没有它，此刻 HEAD 那一份 sha256 12971e7ec4f4 不是快照的 6f5a15521162（快照之后有提交改过它，diff 里没有它的基底；照搬迁登记换回旧路径也对不上）
- `.claude/gate.d/doc-decisions.sh`（提交过、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 那一份 sha256 aa2b18230f3f 不是快照的 74ac64b9865c（快照之后有提交改过它，diff 里没有它的基底；照搬迁登记换回旧路径也对不上）
- `.claude/gate.d/doc-experiments.sh`（提交过、工作区里在、工作区那一份已改）：倒推不出：diff 里没有它，此刻 HEAD 那一份 sha256 dcdc1b805430 不是快照的 19221b5b5e6d（快照之后有提交改过它，diff 里没有它的基底；照搬迁登记换回旧路径也对不上）
- `.claude/gate.d/code-tooling.sh`（提交过、工作区里在、工作区那一份已改）：倒推不出：diff 里没有它，此刻 HEAD 那一份 sha256 0150a83ddae5 不是快照的 7d8030f82d83（快照之后有提交改过它，diff 里没有它的基底；照搬迁登记换回旧路径也对不上）
- `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/reuse_library.py`（哪次提交里都没有、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 那一份 sha256 f19d0d44dc0a 不是快照的 d56517cb7179（快照之后有提交改过它，diff 里没有它的基底；照搬迁登记换回旧路径也对不上）
- `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/frozen_o3.py`（哪次提交里都没有、工作区里在、工作区那一份已改）：倒推不出：diff 里没有它，此刻 HEAD 那一份 sha256 65f2f6c9f104 不是快照的 a50ca5911640（快照之后有提交改过它，diff 里没有它的基底；照搬迁登记换回旧路径也对不上）
- `scripts/experiment-apparatus/e171_three_oracle_collusion/frozen_o3.py`（哪次提交里都没有、工作区里在、工作区那一份已改）：倒推不出：diff 里没有它，此刻 HEAD 那一份 sha256 5add53ca56a2 不是快照的 d34ff5ef068a（快照之后有提交改过它，diff 里没有它的基底；照搬迁登记换回旧路径也对不上）
- `.claude/agent-common.md`（提交过、工作区里在、工作区那一份已改）：倒推不出：diff 里没有它，此刻 HEAD 那一份 sha256 d0bdffc5f4db 不是快照的 76af797bb45e（快照之后有提交改过它，diff 里没有它的基底；照搬迁登记换回旧路径也对不上）
- `.claude/main-agent.md`（提交过、工作区里在、工作区那一份已改）：倒推不出：diff 里没有它，此刻 HEAD 那一份 sha256 f8f299026927 不是快照的 58999fa11c79（快照之后有提交改过它，diff 里没有它的基底；照搬迁登记换回旧路径也对不上）
- `.claude/rules/path-moves.md`（提交过、工作区里在、工作区那一份已改）：倒推不出：diff 里没有它，此刻 HEAD 那一份 sha256 8bbfe0f49b93 不是快照的 6352efbb1fdf（快照之后有提交改过它，diff 里没有它的基底；照搬迁登记换回旧路径也对不上）
- `.claude/long-run-live-hosts-exempt`（提交过、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 那一份 sha256 603dbfdf8b5d 不是快照的 5779c66e68a8（快照之后有提交改过它，diff 里没有它的基底；照搬迁登记换回旧路径也对不上）
- `.claude/heavy-thread-exempt`（提交过、工作区里在、工作区那一份 sha256 与快照相同）：倒推不出：diff 里没有它，此刻 HEAD 那一份 sha256 95c2492c3954 不是快照的 624ae3858873（快照之后有提交改过它，diff 里没有它的基底；照搬迁登记换回旧路径也对不上）
