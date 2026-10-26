# 第十一批：交回闸 ② 认本仓登记过的 worktree 下的三方腿报告（2026-10-23）

依据：`research/prompts/subagent-landing-audit-2026-10-22/b11-guards.md` 第二节与结论里「什么现象能推翻」那一条；拆分提案 `records/2026-09-16-subagent拆分提案.md` 第九十节 ㉜。
草稿目录 `/home/fy5090/.local/share/singlefs/work/landing-audit-b11-worktree-reports`（下文写 `<草稿>`）。

## 结论

- 改完了。经 `scripts/gate/hook-swap.py` 定点换进主工作区：一个规格 10 处，2 份文件，退 0。换上之后在主工作区跑交回闸自证，退 0，从 74 种变成 78 种。派发闸自证退 0，512 种（库换上之后跑的第一次红了 1 格，原因不在这一批，见「没做什么」）。
- 认法：工作树取项目根，再加上 `git -C <项目根> worktree list --porcelain` 列出的每一棵（bare 的、目录不在的不收）。报告路径前面那一截解到其中任何一棵，就认它 `research/prompts/` 那一层的报告。交回闸 ② 把报告按所在的那一棵分组，每组跑一次 `cite-check.py --root <那一棵>`。取不到 worktree 列表时只认项目根，没点名报告的拒绝说明里写明「这一次取不到本仓的 git worktree 列表（原因），只认了项目根下的」。
- 真会话记录（`~/.claude/projects/-home-fy5090-code-singlefs`，三方腿 746 份子 agent 记录）上量的结果：
  - ② 三方腿交回 670 次。改前按没点名报告拒、改后认出的有 37 次，全在 `m5-dev`。这 37 次按那一棵当根、开工时刻当 `--unchanged-since` 现跑 cite-check，37 次都退 0。仍按没点名报告拒的 15 次：`m4-predev` 9 次（那个目录不是登记过的 worktree，此刻也已经不在），另 6 次交回里根本没有 `research/prompts/…-output.md`。新拒 0 次。
  - ⑰ 三方腿派发提示 693 份。同一份派发闸代码只换库，改前改后判得一样：拒 37 份、放 656 份。拒的那 37 份里没有一份写了 `m5-dev`。真提示里写了 `/m5-dev/` 的有 11 份，改前改后都放。造的提示里只有 m5-dev 绝对路径的那一种，改前拒、改后放；m4-predev 的改前改后都拒。
- 什么现象能推翻：
  - 有三方腿在别的本仓 worktree 里把报告写成相对路径（`research/prompts/…`，没有树根前缀）。这种写法照旧按项目根解，会去核主工作区里同名的那一份；不同名就是 cite-check 退 2 放行。语料里 37 次全带绝对前缀，这种写法 0 次。
  - 有人在别的本仓 worktree 里写了同名报告，又让交回点名项目根那一份。那样核的是项目根那一份，这一批不判两份是不是同一个。
  - `git worktree list` 在钩子跑的环境里列不出 m5-dev（比如 CLAUDE_PROJECT_DIR 不在 git 仓里）。那样只认项目根，拒绝说明里会写原因。hook-swap 草稿树（不是 git 仓）里喂的那两条就走了这一支，见「探针与喂法」。

## 改过的文件

| 文件 | 改了什么 |
|---|---|
| `.claude/hooks/lib_three_way_legs.py` | 新加 `registered_trees(project_root)`（`:103`，同一个根一个进程里只取一次，起 git 时去掉 `GIT_DIR` 这类变量）、`prefix_tree`（`:130`，替掉 `prefix_resolves_to_root`）、`leg_report_mentions_in_trees(text, project_root, trees=None)`（`:146`，交回 `[(match, 那一棵)]`）。`leg_report_mentions(text, project_root)`（`:161`）签名不变，改成调同一处，只交回 match；文件头写了认法 |
| `.claude/hooks/handback-guard.sh` | `leg_reports_named`（`:298`）交回 `[(那一棵, 路径)]`；新加 `worktree_list_note`（`:319`）；`citation_problems`（`:327`）按树分组跑 cite-check，`--root` 用那一棵，不在项目根的报告失败行前面带「（那一棵）」，没点名报告那一句改了；文件头 ② 的判法说明、弄坏开关表（新开关 4 条，改了 2 条的点名）；自证新加 4 格，替身那一格改成换 `leg_report_mentions_in_trees` |

`runner-dispatch-guard.sh` 没改。⑰ 调的 `leg_report_mentions(judged, project_root)` 签名没变，自证里的替身也照旧换 `leg_report_mentions`。`is_leg_report_path` 没改：它判的是去掉树根之后的写法，去哪个根由调用方决定。⑰ 报告行那一支只去项目根前缀，m5-dev 的绝对路径在那一支落进「别的目录、当转述」，再由整份提示那一支经 `leg_report_mentions` 认出。所以这一支不改，⑰ 也照认 worktree 里的报告（上面造的提示就是这么放的）。

我自己的改动（对开工时留的原样，`git diff --no-index --stat`）：

```
 .../.claude => .claude}/hooks/handback-guard.sh    | 139 ++++++++++++++++-----
 1 file changed, 110 insertions(+), 29 deletions(-)
 .../hooks/lib_three_way_legs.py                    | 78 +++++++++++++++++-----
 1 file changed, 63 insertions(+), 15 deletions(-)
```

`git diff --stat -- .claude/hooks/handback-guard.sh .claude/hooks/lib_three_way_legs.py` 原样（工作区对暂存区，里面还有任务 35 没暂存的改动）：

```
 .claude/hooks/handback-guard.sh     | 398 +++++++++++++++++++++++++++---------
 .claude/hooks/lib_three_way_legs.py |  85 +++++++-
 2 files changed, 388 insertions(+), 95 deletions(-)
```

sha256：换上之前 `1ba1c96d…cbfc6d`（交回闸）、`e1617453…69defe`（库），和开工时留的原样相同；换上之后 `67893e18…405692`、`43b81c19…94c823`。反向规格是 `<草稿>/swap-real/hook-swap/reverse-20261023T053948Z-3348450.json`。

新钩子、新门禁阶段都没有，所以没写 `# gate-similar:` / `# hook-events:`。`gate-overlap.py --list` 看过（`<草稿>/gate-overlap-list.txt`），判这一条的就是交回闸自己，判据加在它里面。`git worktree list` 的解析在 `scripts/hosts/clean-session-scratch.py` 的 `worktrees_of` 里另有一份（3 行，不收主工作树）。钩子的库不 import `scripts/hosts/`，这一份另写在库里，也没写 `copy-kept`。gate-overlap 要是把它判成整段相同，交主 agent 定。

## 先红后改：造的输入

喂法 `<草稿>/edits/feeds-handback.json`，一共 4 条交回 JSON。会话记录指到不在的路径，所以认出报告之后引文那一条不判、放行，喂出来的退出码只反映认没认出。用 `hook-swap.py --feed-only` 喂主工作区那一份。

改前（换上之前，`<草稿>/feed-only-before.out` 原样）：

```
  ✗ 退 2（应当 0） 三方腿点名登记过的 worktree（m5-dev）里的报告：改前按没点名报告拒，改后认出（没有会话记录，引文这一条不判，放行） ← 
  ✗ 退 2（应当 0） 三方腿点名 ~ 起头写的 m5-dev 里的报告：同上 ← 
  ✓ 退 2（应当 2） 三方腿点名没登记的仓外副本（m4-predev）里的报告：改前改后都按没点名报告拒 ← 
  ✓ 退 0（应当 0） 三方腿点名项目根下的报告：改前改后都认（没有会话记录，放行） ← 
✗ 只喂不换：.claude/hooks/handback-guard.sh 喂了 4 条，2 条不对
```

改后（换上之后，`<草稿>/feed-only-after.out` 原样）：

```
  ✓ 退 0（应当 0） 三方腿点名登记过的 worktree（m5-dev）里的报告：改前按没点名报告拒，改后认出（没有会话记录，引文这一条不判，放行） ← 
  ✓ 退 0（应当 0） 三方腿点名 ~ 起头写的 m5-dev 里的报告：同上 ← 
  ✓ 退 2（应当 2） 三方腿点名没登记的仓外副本（m4-predev）里的报告：改前改后都按没点名报告拒 ← 
  ✓ 退 0（应当 0） 三方腿点名项目根下的报告：改前改后都认（没有会话记录，放行） ← 
✓ 只喂不换：.claude/hooks/handback-guard.sh 喂了 4 条，0 条不对
```

hook-swap 换上时用的喂法是这里面 neutral 的两条（`<草稿>/edits/feeds-swap.json`）。m5-dev 那两条没放进去，因为草稿树不是 git 仓，取不到 worktree 列表，只认草稿树自己，那两条在草稿里本来就该退 2。头一次 `--dry-run` 把 4 条全喂了，那两条在草稿里退 2，拒绝说明里写的是「这一次取不到本仓的 git worktree 列表（git worktree list 退 128：fatal: not a git repository (or any of the parent directories): .git），只认了项目根下的，登记过的 worktree 里的报告也没认出来」。这就是取不到列表那一支在真入口上走了一遍。

## 探针：样本仓上改前、改后与每条开关（`<草稿>/tools/probe-trees.py`，读数 `<草稿>/probe-trees.out`）

样本：主树 `trees-sample`（git 仓，有一次空提交）、经 `git worktree add` 登记的 `trees-registered`、单独 `git init` 的副本 `trees-unregistered`、`.git` 指到不在的目录的 `trees-list-unreadable`。主树里 `kb/rule.md` 第 1 行是标题、第 2 行是那句话；另三处只有那一句，占第 1 行。表里每格是 decide 的退出码：

| 样本 | 改前 | 改后 | subdirectory-kept | outside-root-kept | worktrees-unread | git-directory-trusted | citation-root-main | worktree-error-unnamed |
|---|---|---|---|---|---|---|---|---|
| 登记过的 worktree 里引文对上 | 2（没点名） | 0 | 0 | 0 | **2** | 0 | **2** | 0 |
| 登记过的 worktree 里引文对不上 | 2（没点名） | 2（引文） | **0** | **0** | 2（没点名） | 2 | **0** | 2 |
| 没登记的副本（带 .git） | 2（没点名） | 2（没点名） | **0** | **0** | 2 | **0** | 2 | 2 |
| 列表读不了的根（另核拒绝说明里有没有「取不到」） | 2，没有 | 2，有 | **0**，没有 | **0**，没有 | 2，有 | 2，有 | 2，有 | 2，**没有** |

粗体的格就是开关打开后自证里会红的那一格，和文件头开关表点名的逐条对得上（见下一节）。

## 自证格与开关（主工作区，换上之后）

新加 4 格，都在 `handback-guard.sh` 的 `selftest()` 里（`:1168`–`:1172`）。样本照上一节那样搭在自证的临时目录里，git 只在那里面起，`finally` 里整个删掉：

- `三方腿报告路径:登记过的 worktree 里引文对上放行`：样本里的绿格
- `三方腿报告路径:登记过的 worktree 里引文对不上拒`：worktree 里的报告照样核引文，而且按那一棵当根
- `三方腿报告路径:没登记成 worktree 的仓外副本（带 .git）里的不认，按没点名报告拒`：样本里的红格
- `三方腿报告路径:取不到 worktree 列表时只认项目根、拒绝说明写明`

改了 1 格：替身那一格（`:1181`）的标签改成 `三方腿报告路径:交回闸调的是 lib_three_way_legs.py 的 leg_report_mentions_in_trees`，替身换的是 `leg_report_mentions_in_trees`。

| 跑了什么 | 结果 |
|---|---|
| `bash .claude/hooks/handback-guard.sh --selftest` | 退 0，`✓ handback-guard 自检通过（查了 78 种）`（开工时 74 种），3.32 秒 |
| `bash .claude/hooks/runner-dispatch-guard.sh --selftest` | 头一次退 1，红 1 格，原因不在这一批（见「没做什么」）；重跑退 0，`✓ 自检通过（查了 512 种）` |
| 交回闸新加的 4 条开关、改了点名的 2 条，逐条设上跑 `--selftest` | 6 条都退 1，点名的项数等于失败行数，点名的都在失败行里：subdirectory-kept 7/7、outside-root-kept 5/5、worktrees-unread 1/1、git-directory-trusted 1/1、citation-root-main 2/2、worktree-error-unnamed 1/1（`<草稿>/tools/check-switch-named.py`，读数 `<草稿>/switches/hb-*.out`） |
| 交回闸另外 22 条开关逐条设上跑 | 22 条都退 1。点名的项都在失败行里，有两条例外，都和这一批无关：`limit-hint-unnamed` 开关表里写的标签是截短的；`cite-any-agent` 点名里有一项是概括写法（「⑤ ⑥ 那几格里应当放行的项目子 agent（…）」），我的比对工具认不出这两种写法。我新加的格作为没点名的失败行只出现在 `no-cite` 下（引文那一条整个关掉，worktree 里对不上的、副本、取不到那 3 格跟着红），和任务 35 那 3 格一样没补进 `no-cite` 的点名 |
| 派发闸 `leg-report-outside-root-kept`、`leg-report-placeholder-kept` 两条 | 都退 1，点名 2/2、2/2 |
| `bash -n`、内嵌 python 编译、`py_compile` 库 | hook-swap 在草稿与主工作区各做一次，都过 |

新开关（文件头开关表里都写了弄坏哪一处、点名哪几格）：

- `HANDBACK_GUARD_BREAK=leg-report-worktrees-unread`：只认项目根，就是任务 35 那一版
- `HANDBACK_GUARD_BREAK=leg-report-git-directory-trusted`：前面那一截只要是带 `.git` 的目录就认，不看登没登记成本仓的 worktree
- `HANDBACK_GUARD_BREAK=leg-report-citation-root-main`：worktree 里的报告照样用项目根当 `--root` 核
- `HANDBACK_GUARD_BREAK=leg-report-worktree-error-unnamed`：拒绝说明里不写取不到 worktree 列表

原有的 `leg-report-subdirectory-kept` 和 `leg-report-outside-root-kept` 各补点名了 3 格：副本、worktree 里对不上、取不到。

## 真会话记录上的改前改后（`<草稿>/tools/measure-legs.py`，读数 `<草稿>/measure-legs.out`）

语料：`python3 -I <草稿>/tools/extract-legs.py ~/.claude/projects/-home-fy5090-code-singlefs <输出> 12`，抽了 746 份三方腿子 agent 记录（用完已删，要重抽照这条跑）。改前用开工时那一版钩子与库，改后用换上的那一版。⑰ 这边用同一份派发闸代码，只换库。读数原样（例子只留每类第一条）：

```
三方腿类型（此刻 frontmatter 登记的）：['three-way-attack', 'three-way-defense', 'three-way-forward']
改后认的工作树：(['/home/fy5090/code/singlefs', '/home/fy5090/.local/share/singlefs/work/m5-dev'], None)
② 三方腿交回（一次交回调用一次）：
  两边认的一样（都在项目根）：618 次
  改前一份不认（按没点名报告拒）、改后认出：/home/fy5090/.local/share/singlefs/work/m5-dev：37 次；例：a028ec0c5ee1bd7bb（three-way-attack，2026-10-19）：'…报告 /home/fy5090/.local/share/singlefs/work/m5-dev/research/prompts/m5-box-lifecycle-r1-opus-output.md\nsha256 5' ｜ …
  改前改后都一份不认（都按没点名报告拒）：15 次；例：a2e0cc6ef56fad5c8（three-way-attack，2026-10-16）：'…报告：/home/fy5090/.local/share/singlefs/work/m4-predev/research/prompts/m4-trunc-r2-opus-output.md（284 行，sha256 0fa' ｜ …
  改后认出的那几次，报告路径前面那一截：{'/home/fy5090/.local/share/singlefs/work/m5-dev/': 3, '报告：/home/fy5090/.local/share/singlefs/work/m5-dev/': 21, '报告写完：/home/fy5090/.local/share/singlefs/work/m5-dev/': 2, '**报告**：`/home/fy5090/.local/share/singlefs/work/m5-dev/': 1, '报告：`/home/fy5090/.local/share/singlefs/work/m5-dev/': 7, '`/home/fy5090/.local/share/singlefs/work/m5-dev/': 2, '辩方腿报告已写完：/home/fy5090/.local/share/singlefs/work/m5-dev/': 1}
  那几次按所在那一棵现跑 cite-check：{'退 0': 37}
⑰ 三方腿派发提示（一份记录一份；同一份派发闸，只换库）：
  改前拒、改后拒：37 份
  改前放、改后放：656 份
```

- ② 37 次全部改回放行，这正是任务 35 量到的 m5-dev 那 37 次（份数、例子都一样）。我现跑 cite-check 用的是那一棵此刻的文件，开工之后改过的被引文件只记「没判」。所以「37 次都退 0」说的是此刻核得过，不是当时那一刻的结果。
- 那 9 次 `m4-predev` 照拒。`<草稿>/dispatch-rejects.out` 里，仍按没点名报告拒的 15 次分成两类：前缀是 `报告：…/work/m4-predev/` 的 6 次、`报告：` 后面加反引号再接 `…/work/m4-predev/` 的 3 次，另 6 次交回里根本没有 `research/prompts/…-output.md`。总数 670 比任务 35 的 669 多 1 次，是之后新落的记录；新拒 0 次。
- ⑰：693 份，改前改后一份没变。改后仍拒的 37 份里没有一份写了 `m5-dev`：18 份 attack、10 份 forward、4 份 defense 是整份提示里没有这一形态，另 5 份是报告行点名的文件不是这一形态。真提示里写了 `/m5-dev/` 的 11 份，改前改后都放。造的提示：「分段落盘到 …/m5-dev/research/prompts/x-r1-sonnet-output.md」与「报告：…/m5-dev/research/prompts/x-r1-opus-output.md」都是改前拒、改后放；m4-predev 的改前改后都拒。

## 要主 agent 跑的命令

工作目录 `/home/fy5090/code/singlefs`。只起 python 与 bash，不需要内存包装，也不用设线程数。

1. 弄坏开关的复跑方与开关表格式（交回闸开关表改了）：

```
SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 break-switch-headers、break-switch-replay 格; 范围=2 格; 为什么=第十一批交回闸认登记过的 worktree，新加四条弄坏开关、改了两条的点名，复跑每条开关点名的项' bash .claude/gate.d/code-tooling.sh --check break-switch-headers,break-switch-replay /home/fy5090/code/singlefs
```

   看什么：`break-switch-replay` 那一格里 `.claude/hooks/handback-guard.sh` 没有「点名的项没在失败行里」「退 0」「超时」。登记表给交回闸每条开关的上限是 120 秒，我单跑一次自证要 3.32 秒。

2. 提交前那一趟（`scripts/gate/gate-staged.sh`）会判到的几格我没跑：gate-overlap（库里新加的 `registered_trees` 和 `scripts/hosts/clean-session-scratch.py` 的 `worktrees_of` 都解析同一种输出，但写法不同）；code-tooling 的 research-gate-lint（交回闸的拒绝出路）和 break-switch-outside-selftest（交回闸在 `.claude/gate.d/break-switch-outside-selftest-pending.tsv` 里，新开关也只在自证里设）。

## 没做什么

- 没跑重型测试（checker-tier-crash-point-replay、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification、checker-tier-research-build-and-replay 本身、checker-tier-broad-mutation 与 `scripts/mutation/broad-mutation.py` 起 cargo 的用法、`gate.sh`、全量 `cargo test`、`check.sh`），也不编译。门禁阶段与 lint 都没跑。没改 `.rs`，所以命名检查不用跑。没走定义三方：这一批没改 agent 定义、共用约束与规则。没提交。
- 派发闸自证头一次的那 1 格红，原因不在这一批：`✗ 自检：真提示:a9717cf07fb6e7c21.txt（tooling-writer，…） 应当返回 ('6ffe7b19…', False, False)，实际 ('a192f6e0…', False, False)`。只是钉的 sha256 对不上，判定那两样没变。别的会话做 `scripts/experiments → scripts/experiment-apparatus` 目录改名时，把 `research/prompts/landing-audit-r3-opus-model/prompts/a9717cf07fb6e7c21.md` 和钩子样本目录里那一份一起改了一行，`.claude/hooks/fixtures/runner-dispatch-guard/real-prompts/sources.tsv` 与 `runner-dispatch-guard.sh:2000` 的期望又是那个会话随后重钉的（sources.tsv 的修改日期落在我开工之后，派发闸 `:2000` 对我开工时留的原样改了这一行）。重钉之后重跑退 0，512 种。这一批没碰派发闸，也没碰那几份样本。冻结的真提示被目录改名一并改掉，这件事要不要管，交主 agent 定。
- 别的本仓 worktree 里写成相对路径（`research/prompts/…`，没有树根前缀）的报告，照旧按项目根解，没改成按交回那个子 agent 的 cwd 去解：语料里这种写法 0 次。
- `no-cite` 开关的点名没补新格，和任务 35 那 3 格的做法一样。`limit-hint-unnamed` 点名的标签是截短的，没动。
- `is_leg_report_path` 与派发闸 ⑰ 报告行那一支没改，理由见「改过的文件」。
- m4-predev 那个目录有自己的会话记录目录（`~/.claude/projects/-home-fy5090--local-share-singlefs-work-m4-predev`），那边的会话以它自己为项目根，没放进语料。

## 草稿目录清理

删之前 `du -sh` 量过，删了这些：`tree-before` 14M、`tree-after` 14M（钩子、定义与 `scripts/agents|kb|hosts` 的拷贝，探针与量数用）、`swap-dry` 44K（演练）、语料 `corpus-legs.jsonl` 4.7M。剩下的都是文本：本报告、`originals/`（换上之前的三份原样，536K）、`edits/`（规格与喂法）、`swap-real/hook-swap/`（喂法日志与反向规格）、`tools/`（探针、量数、生成规格的脚本）、读数（`*.out`）、`switches/`（开关复跑的输出）。没有仓副本、编译目录或库拷贝。`.deliverables` 里声明了报告、`originals` 与反向规格三项。
