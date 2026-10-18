# C439（gate-ok 前移让同轮早先写的行不再被判）第二轮：云端辩方腿（Sonnet）报告（2026-10-15）

<!-- doc-lint:not-numbers K1 K4 K6 -->

立场：替第一轮判决里出局的丙、戊（集合键、比 HEAD）、门禁写的乙（退 merge-base）、按树挂的丁找站得住的理由，替庚二与 wu_cf 站得住的那一面答 K1、K4；K6 必答。
没读禁读清单里的任何东西（`research/prompts/c439-r2-opus-output.md`、`research/prompts/c439-r2-opus-model/`、`/home/fy5090/.local/share/singlefs/work/c439-r2-opus/`）。
冻结副本 `/home/fy5090/.local/share/singlefs/work/c439-r1/frozen/` 的 10 份与仓里此刻同一路径的 10 份逐份 `cmp` 过，全同；所以引脚本写仓内路径，行号就是冻结副本的行号。
模型在 `research/prompts/c439-r2-sonnet-model/`（引擎是 `c439-r1-opus-model/c439-model.sh` 逐字拷来再加 r2 标记的几处：新候选、新历史、`after_count` 字段）。「推的」都标了。

## 一、判定一览

| 格 | 判 | 一句话（数都在下面各节，原样输出在 `research/prompts/c439-r2-sonnet-model/out/`） |
|---|---|---|
| K6 丙（staged-green 的树对暂存树） | **判对**（照今天的外壳） | 交错三形与人给 `GATE_BASE` 在我的模型上全部复现（守规矩续作一次都没判到：丙 984/2274，今天 925/2274）。补两处就是庚二（里层判外壳 write-tree 的树；人给 `GATE_BASE` 不前移），缺一处都不行（只补前一处 636/2274，只补后一处 606/2274，两处 258/2274）。 |
| K6 戊（集合键、比 HEAD） | **判对** | `wu_set` 在「同页再抄一遍」「绕过提交同时涨表」两形上 135/135 漏；`wu_cf`（计数键 + 固定基线）两形 0/135。换键、换基线之后它就是 wu_cf。`wu_cf` 有一个同键搬家的残余（下面 K6 戊），这一形庚二不漏。 |
| K6 乙（门禁写 gate-ok、退 merge-base） | **判对** | 记号挂在 HEAD、HEAD 里有被判的树删掉的行：`yi_gate_mb` 三形 135/135 漏。记号改挂「被判的那棵树」就是庚二。只在「被判的树恰好等于开跑时 HEAD 的树」才写的弱修（`yi_clean`）也堵住这三形、带 N 或 S 的续作 0 漏，但每一批暂存着东西跑门禁的正常流程里它不前移，窗口只增不缩（留在暂存口径，长度 0..3 的扫描里 114 条续作各重判 1 次，庚二 0 次）。 |
| K6 丁（按树挂） | **判对，理由要换** | 第一轮写的出局理由是「继承丙的两形」，补上丙的两处之后这条理由就没了：`ding_tree_fixed`、`ding_lazy_fixed` 与庚二在 23 段历史上的覆盖逐段相同（23/23），重判更多（退出暂存口径 40 对 20），还要提交后钩子或补挂。出局成立，但理由是被庚二支配，不是漏。 |
| K6 额外：第一轮判决与 C836 里「带 `--force` 时外壳同样前移」 | **判错**（可达性） | 外壳 `gate-staged.sh` 第 20 行把 `--force` 摘掉，第 33 行不再传给里层，里层没有阶段会被强制跑；这一形经外壳造不出历史。庚二里「强制跑过不前移」那条守卫今天堵不了任何现有口子，只能当防御。 |
| K6 H3 两种代价口径（庚二下） | 都量了 | 退出暂存口径（别的会话先退出暂存再跑绿）：重判 1 次/续作（今天 46 次合计、最多 4 次）；留在暂存口径（那行留在暂存里被判过）：0 次（今天 40 次、最多 4 次）。两种都不漏判。 |
| K1 | 站得住 | 庚二的落法（`read-tree --reset -u <外壳 write-tree 的树>`）在 20 个案上 20/20 与 write-tree 同棵；今天的落法 14/20，其中 2 案被判的树缺东西而外壳照前移。子模块、符号链接、过滤器、eol、`add -N` 等静态案两种落法都同棵。守卫放外壳；`--force` 那条守卫经外壳不可达。 |
| K4 | 窄的站得住 | 只给「判新增行内容」的格改基准（doc-experiments 三格共一份新增行），不动 `gate_diff_base gate`；不带 `--staged` 的路必须跟着改；字面不碰 2026-09-24 定案，但改两条路窗口关系，照跑前条款交用户。 |

## 二、复跑

```
cd /home/fy5090/code/singlefs
bash scripts/hosts/run-with-memory-cap.sh 8G bash scripts/hosts/capped.sh 31 \
  bash research/prompts/c439-r2-sonnet-model/tools/rerun.sh --repo /home/fy5090/code/singlefs \
  --frozen /home/fy5090/.local/share/singlefs/work/c439-r1/frozen --work <新的草稿目录>
```

我跑了两次（第一次落进 `out/`，第二次与它逐字比），第二次的原样输出：

```
name=compare file=k6-examples.txt same=yes
name=compare file=k1-tree-identity.txt same=yes
name=compare file=h3-cost.txt same=yes
name=compare file=small-checks.txt same=yes
name=compare file=summary-L2.txt same=yes
```

长度 3 的扫描（16 个候选 × 23 段历史，约 20 分钟，不在 `rerun.sh` 里）的命令与产物：

```
bash scripts/hosts/capped.sh 31 bash research/prompts/c439-r2-sonnet-model/sweep-all.sh <冻结副本> <输出目录> 3 <历史…>
python3 research/prompts/c439-r2-sonnet-model/summarize.py <输出目录>/*/results.txt   # 逐字等于 out/summary-L3.txt
```

「守规矩续作」沿用第一轮攻方腿的口径：每一次提交（`c`、`k`）紧跟在一次门禁（`N`、`S`、`G`）之后；`G` 是人带 `GATE_BASE=HEAD` 跑外壳。「一次都没判到」是跑前条款 2 的观测：「一份可复跑的临时仓脚本：照流程跑门禁、提交、再跑门禁，那一行一次都没进 `gate_added_lines` 的输出」（`research/prompts/_c439-r1-body.md:47`）。

## 三、K6：第一轮出局的四个候选逐个复核

下面每个候选先给「打中它的形」在我的引擎上的结果，再给「改成什么之后它就是庚二或 wu_cf」以及改完后同一形的结果。例子名是 `out/k6-examples.txt` 里 `=== example=` 那一行的名字；`judged_ever=no` 就是跑前条款 2 的字面（那一行一次都没进判的行）。

### 丙：判对（照今天的外壳）；补两处就是庚二

判的树不是挂的树，出在这几行：外壳在「staged_tree="$(git write-tree)"」（`scripts/gate/gate-staged.sh:28`）算树，里层在「if ! git -C "$source_repository_root" diff --cached --binary > "$staged_scratch_directory/staged.patch"; then」（`.claude/singlefs-ai-sop/scripts/gate.sh:56`）另取一次暂存区，套到「git -C "$source_repository_root" worktree add --detach "$staged_tree" HEAD」（`.claude/singlefs-ai-sop/scripts/gate.sh:59`）建的树上，用的是「git -C "$staged_tree" apply --index "$staged_scratch_directory/staged.patch"」（`.claude/singlefs-ai-sop/scripts/gate.sh:87`）；外壳收尾只比「if [[ "$now_tree" != "$staged_tree" ]]; then」（`scripts/gate/gate-staged.sh:39`）。人给的 `GATE_BASE` 里层先认：「staged_diff_base="${GATE_BASE:-$(diff_base "$source_repository_root")}"」（`.claude/singlefs-ai-sop/scripts/gate.sh:102`）；上游自己的 gate-ok 前移有守卫「elif [[ -n "$START_HEAD" && -z "${GATE_BASE:-}" ]]; then」（`.claude/singlefs-ai-sop/scripts/gate.sh:774`），外壳没有同样一条，它的前移句是「git update-ref refs/sop/staged-green "$green_commit"」（`scripts/gate/gate-staged.sh:46`）。

所以第一轮判决对今天的外壳判「出局」成立。但它自己在同一格写了「补上之后的形是庚二」（`research/prompts/c439-r1-main-verification.md:27`）：出局的是外壳的这两处，不是「拿 staged-green 的树当基准」这个想法。我把两处补丁拆开各跑一遍，看缺一处行不行：

| 候选（引擎里的名字） | 补了什么 | 14 段历史上守规矩续作一次都没判到 | 其中没有任何 N 或 S 的（人一直带 GATE_BASE） |
|---|---|---|---|
| 今天（`today`） | 无 | 925/2274 | 327 |
| 丙（`bing`） | 无 | 984/2274 | 294 |
| `geng` | 只补「里层判 write-tree 的树」 | 636/2274 | 258 |
| `bing_guard` | 只补「人给 GATE_BASE 不前移」 | 606/2274 | 294 |
| 庚二（`geng2`） | 两处都补 | 258/2274 | 258 |

表是 `out/summary-L3.txt` 的 `name=total14` 行的摘抄，14 段历史是：`h1_c439 h2_noverify h3_split h4_other h5_pushed h2u_unpushed h6_stamp_S h6_stamp_N h6_stamp_revert h8_amend h9_aba h9_stash h9_commit_undo h15_override`。庚二剩下的 258 条全是没有任何一次 N 或 S 的续作，即人每一趟都带 `GATE_BASE=HEAD` 跑：这是人自己每次都收窄窗口，两处补丁都拦不住，也不是外壳的口子。

同一批形在 `k6-examples.sh` 里各跑一例，原样输出：

```
id=bing-race-unstage cand=bing hist=h9_aba cont=S judged_ever=no
id=bing-race-stash cand=bing hist=h9_stash cont=S judged_ever=no
id=bing-race-commit-undo cand=bing hist=h9_commit_undo cont=S judged_ever=no
id=bing-override cand=bing hist=h15_override cont=kS judged_ever=no
id=geng2-race-unstage cand=geng2 hist=h9_aba cont=S judged_ever=yes
id=geng2-race-stash cand=geng2 hist=h9_stash cont=S judged_ever=yes
id=geng2-race-commit-undo cand=geng2 hist=h9_commit_undo cont=S judged_ever=yes
id=geng2-override cand=geng2 hist=h15_override cont=kS judged_ever=yes
id=geng-race-unstage cand=geng hist=h9_aba cont=S judged_ever=yes
id=geng-override cand=geng hist=h15_override cont=kS judged_ever=no
id=bing_guard-race-unstage cand=bing_guard hist=h9_aba cont=S judged_ever=no
id=bing_guard-override cand=bing_guard hist=h15_override cont=kS judged_ever=yes
```

补丁落在哪（推的，没有改任何脚本）：① 里层判外壳 write-tree 的树：把 `gate.sh` 第 56 行与第 87 行的「diff --cached + apply --index」换成对外壳已导出的 `SINGLEFS_STAGED_TREE`（「SINGLEFS_STAGED_TREE="$staged_tree"」（`scripts/gate/gate-staged.sh:31`））做 `read-tree --reset -u`，这一步要改上游 `gate.sh`；C836 的改法一列也这么写：「外壳把它 write-tree 的那棵树交给里层判（里层不再自己从暂存区重套）」（`.claude/kb/checks-owed.md:557`）。② 守卫放外壳（K1 一节细说）。

什么现象会推翻这一行：造得出一段历史，补了两处之后 staged-green 仍挂上含没判过的行的树——也就是 `geng2` 在 `out/summary-L3.txt` 里出现不带 G-only 的漏判（我扫了长度 0..3，23 段历史，没有）。

### 戊（集合键、比 HEAD）：判对；换键、换基线就是 wu_cf

第一轮的两形：存量里豁免着的错引文在同一页再抄一遍（`h11_dup`），绕过门禁的提交同时加错引文与它的豁免行（`h12_exempt_grow`）。四个变体各跑一遍，原样输出：

```
id=wu_set-duplicate cand=wu_set hist=h11_dup cont=S judged_ever=no
id=wu_set-grow cand=wu_set hist=h12_exempt_grow cont=S judged_ever=no
id=wu_count-duplicate cand=wu_count hist=h11_dup cont=S judged_ever=yes
id=wu_count-grow cand=wu_count hist=h12_exempt_grow cont=S judged_ever=no
id=wu_fixed-duplicate cand=wu_fixed hist=h11_dup cont=S judged_ever=no
id=wu_fixed-grow cand=wu_fixed hist=h12_exempt_grow cont=S judged_ever=yes
id=wu_cf-duplicate cand=wu_cf hist=h11_dup cont=S judged_ever=yes
id=wu_cf-grow cand=wu_cf hist=h12_exempt_grow cont=S judged_ever=yes
```

读法（引擎里 `wu_judge.py` 的四个模式）：`wu_set` 豁免按（路径、行文）集合认、只缩不涨比 HEAD；`wu_count` 改成计数；`wu_fixed` 集合键但比引入时的固定基线；`wu_cf` 计数键 + 固定基线。每个键、每个基线各修一形：

| 变体 | 同页再抄一遍（`h11_dup`） | 绕过提交同时涨表（`h12_exempt_grow`） |
|---|---|---|
| `wu_set` | 漏 135/135 | 漏 135/135 |
| `wu_count` | 0/135 | 漏 135/135 |
| `wu_fixed` | 漏 135/135 | 0/135 |
| `wu_cf` | 0/135 | 0/135 |

表是 `out/summary-L3.txt` 的 `name=total_wu` 行的摘抄（每格是守规矩续作里一次都没判到的条数 / 守规矩续作条数）。所以第一轮对「这两种豁免键」判出局判对了；改成什么：键从集合换成计数、基线从 HEAD 换成引入时的固定基线，它就是 wu_cf——这正是攻方腿自己提的 wu_cf，不是我新造的。

wu_cf 的一个残余，我查出来了，不替它藏：存量里豁免着的那一行在同一页里搬位置（绕过门禁的提交把它从页末搬到页首，行文一字不改），（路径、行文）键与计数都没变，那一行一直在豁免里，一次都没进判。原样输出：

```
id=wu_set-move cand=wu_set hist=h17_wu_move cont=S judged_ever=no
id=wu_cf-move cand=wu_cf hist=h17_wu_move cont=S judged_ever=no
id=geng2-move cand=geng2 hist=h17_wu_move cont=S judged_ever=yes
id=today-move cand=today hist=h17_wu_move cont=S judged_ever=yes
```

`out/summary-L3.txt` 里 `name=total_wu` 的 `h17_wu_move` 一列：wu 一族四个变体都是 135/135，窗口类候选里庚二是 21/135（全是 G-only）、今天是 66/135。按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md@判据自己也会写错：打中之后先判是哪一种`：「打中之后先问四句」过一遍：① 分辨臂：分辨——打 wu 一族，不打窗口类；② 被判的系统看得到吗：看不到——「已知存量错话换了位置」与「新抄了一份同键的错话」在（路径、行文）键与计数上逐条相同；③ 满足条款的哪个分句：跑前条款 2 的字面是「那一行一次都没进 `gate_added_lines` 的输出」（`research/prompts/_c439-r1-body.md:47`），wu 一族根本没有这个输出，第一轮攻方腿把它读成「一次都没进判定」；④ 改法：把键改成含行号会让页首插一行就全部变红，我没有可行的修法。我的判法：**判不了它算不算打中 wu_cf**——按字面不满足，按攻方腿对戊的读法算打中；被搬的行行文与已登记的存量错话逐字相同，没有新的错信息进来，但它确实是一条 git diff 里的新增行、窗口类候选会判它（红，逼人回改）。这一形该记进 wu_cf 的代价（推的：能搬的只有豁免表里已登记的那约 100 行）。

什么现象会推翻「戊判对」：造得出一段历史，`wu_set` 或 `wu_count` 或 `wu_fixed` 在上面两形上不漏（上表是 `h11_dup`、`h12_exempt_grow` 的 135 条续作全扫，没有）。什么现象会推翻「wu_cf 补上两处就够」：除同键搬家之外，再造出一形让 wu_cf 的计数键或固定基线漏判。

### 乙（门禁写 gate-ok、退 merge-base）：判对；记号改挂被判的树就是庚二

第一轮的形：一行错引文绕过门禁提交（HEAD 里有了它），随后有人在暂存区（`h6_stamp_S`）或工作区（`h6_stamp_N`）把它删掉，整轮全绿，gate-ok 写成开跑时的 HEAD（含那一行），之后改动撤回或被 `git revert`（`h6_stamp_revert`），那一行回来，它已在基准里，永远不再是新增行。原样输出（`yi_gate_mb` 是被打中的；`yi_clean` 是我替乙找的弱修：只在被判的树恰好等于开跑时 HEAD 的树时才写 gate-ok；`geng2`、`ding_tree_fixed` 是记号挂被判的树）：

```
id=yi_gate_mb-stamp-S cand=yi_gate_mb hist=h6_stamp_S cont=S judged_ever=no
id=yi_gate_mb-stamp-N cand=yi_gate_mb hist=h6_stamp_N cont=S judged_ever=no
id=yi_gate_mb-stamp-revert cand=yi_gate_mb hist=h6_stamp_revert cont=S judged_ever=no
id=yi_clean-stamp-S cand=yi_clean hist=h6_stamp_S cont=S judged_ever=yes
id=yi_clean-stamp-N cand=yi_clean hist=h6_stamp_N cont=S judged_ever=yes
id=yi_clean-stamp-revert cand=yi_clean hist=h6_stamp_revert cont=S judged_ever=yes
id=geng2-stamp-S cand=geng2 hist=h6_stamp_S cont=S judged_ever=yes
id=geng2-stamp-N cand=geng2 hist=h6_stamp_N cont=S judged_ever=yes
id=geng2-stamp-revert cand=geng2 hist=h6_stamp_revert cont=S judged_ever=yes
id=ding_tree_fixed-stamp-S cand=ding_tree_fixed hist=h6_stamp_S cont=S judged_ever=yes
id=ding_tree_fixed-stamp-N cand=ding_tree_fixed hist=h6_stamp_N cont=S judged_ever=yes
id=ding_tree_fixed-stamp-revert cand=ding_tree_fixed hist=h6_stamp_revert cont=S judged_ever=yes
```

病根在「记号挂在 HEAD 这个提交上，被判的却是 HEAD 加暂存区（或工作区）」，不在退 merge-base。C439 自己留的那句「两种都要先想清多会话共用一个 ref 时谁定起点」（`.claude/kb/checks-owed.md:315`），换成「起点 = 最近一次整轮全绿的那棵被判的树」就有了答案：谁最后一个判绿，起点就是谁判的那棵树。这正是庚二。

弱修 `yi_clean` 值得单列，因为它是乙最像样的救法：它在 23 段历史上带 N 或 S 的续作里 0 漏（`name=never_with_NS cand=yi_clean never=0/3489`），代价是正常流程里 gate-ok 几乎不前移——每一批都是「暂存着东西跑门禁」，被判的树不等于 HEAD 的树，要有人在暂存区为空时再跑一趟才前移，窗口只增不缩。留在暂存口径（那行留在暂存里被判过）的重判：

```
name=h3cost hist=h14_rejudge_cost today=40/4/20/20 bing=0/0/0/20 geng2=0/0/0/20 geng2_nc=0/0/0/20 yi_gate_mb=20/1/20/20 ding_tree_fixed=0/0/0/20
```

每格是「合计 / 单条续作最大 / 至少重判一次的续作条数 / 守规矩续作条数」，长度 0..4、字母表 Sck 的全部守规矩续作；`yi_gate_mb` 每条续作都重判 1 次，庚二（`geng2`）0 次。`out/summary-L3.txt` 里 `name=cost hist=h14_rejudge_cost` 的 `yi_clean=114/1/114` 同形（字母表 NSckoG、长度 0..3）。

什么现象会推翻「乙判对」：`yi_gate_mb` 在三个 stamp 历史上被判到（上表三行 `judged_ever=no`，扫描 135/135 漏）。什么现象会推翻「记号改挂被判的树就是庚二」：`geng2` 在 stamp 历史上带 N 或 S 的续作里漏（我扫了，0/3489）。

### 丁（按树挂）：判对，理由要换

第一轮写的出局理由是「按树挂的继承丙的两形」。原样的 `ding_tree`（提交那一刻 HEAD 的树等于 staged-green 的树才挂记号）确实在这两形上被打中：

```
id=ding_tree-race-unstage cand=ding_tree hist=h9_aba cont=kS judged_ever=no
id=ding_tree-override cand=ding_tree hist=h15_override cont=kS judged_ever=no
```

但这两形是丙的两处缺口，补上（里层判 write-tree 的树；人给 `GATE_BASE` 不前移）之后丁就不再中。我给丁补了两个版本：`ding_tree_fixed`（补丁 + 提交那一刻挂记号，要提交后钩子；本仓的 `.git/hooks/pre-commit` 本地、不入库：「只是提交时间窗闸（本地、不入库），不跑门禁」（`research/prompts/_c439-r1-body.md:20`），所以这个版本要先解决钩子怎么随仓走）、`ding_lazy_fixed`（补丁 + 下一趟门禁开头发现 HEAD 的树等于 staged-green 的树才补挂，不要钩子）。两个版本与庚二比：

```
name=same_as_geng2 cand=ding_lazy_fixed same=23/23
name=same_as_geng2 cand=ding_tree same=19/23
name=same_as_geng2 cand=ding_tree_fixed same=23/23
name=same_as_geng2 cand=geng2 same=23/23
name=never_with_NS cand=ding_lazy_fixed never=0/3489
name=never_with_NS cand=ding_tree never=74/3489
name=never_with_NS cand=ding_tree_fixed never=0/3489
name=never_with_NS cand=geng2 never=0/3489
name=h3cost hist=h16_unstaged_other today=46/4/20/20 bing=20/1/20/20 geng2=20/1/20/20 geng2_nc=20/1/20/20 yi_gate_mb=44/4/20/20 ding_tree_fixed=40/4/20/20
```

两个修后版本与庚二在 23 段历史上的覆盖逐段相同（23/23），带 N 或 S 的续作 0 漏；重判更多（退出暂存口径每格 `ding_tree_fixed=40/4/20/20`，庚二 `20/1/20/20`），还多一族 `refs/notes/…` 与补挂逻辑。所以丁的出局成立，但成立的理由是「被庚二支配：覆盖一样、重判更多、机件更多」，不是「继承丙的漏」。主 agent 若把理由抄进岔路单，要用这一句。

什么现象会推翻「丁被庚二支配」：造得出一段历史，`ding_tree_fixed` 或 `ding_lazy_fixed` 判到而庚二判不到（我扫了 23 段历史、长度 0..3 的全部续作，没有；改写历史的形我没建模，见「没做什么」）。

### 额外一句判错：「带 `--force` 时外壳同样前移」经外壳不可达

C836 写「调用方给了 `GATE_BASE`（窗口被人收窄）或带 `--force`（强制跑过的阶段不让退出码变非 0）时外壳同样前移」（`.claude/kb/checks-owed.md:557`），第一轮判决也写「外壳同样照前移」（`research/prompts/c439-r1-main-verification.md:18`），庚二的定义里有一条「这一趟有阶段强制跑过（`--force`）时不前移」。读代码，外壳收到的 `--force` 到不了里层：外壳开头「preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}」（`scripts/gate/gate-staged.sh:20`）按 preflight 的写法把 `--force` 摘出去：「if [[ "$preflight_argument" == --force ]]; then PREFLIGHT_FORCE_GIVEN=1; else PREFLIGHT_ARGUMENTS+=("$preflight_argument"); fi」（`.claude/singlefs-ai-sop/scripts/preflight.sh:31`），起里层时只传剩下的「bash "$ROOT/.claude/scripts/gate.sh" --staged "$@"」（`scripts/gate/gate-staged.sh:33`）；里层 `gate.sh` 的强制选项只从自己收到的参数来：「if [[ $PREFLIGHT_FORCE_GIVEN -eq 1 ]]; then GATE_FORCE_OPTION=(--force); fi」（`.claude/singlefs-ai-sop/scripts/gate.sh:14`）。拿仓里真的 `preflight.sh` 对一个空脚本喂 `--force /some/root extra`（不起 `gate-staged.sh`、不起任何门禁阶段）：

```
== 一、preflight 摘 --force
after-strip: n=2 args=[/some/root extra] force_given=1
inner would run: gate.sh --staged /some/root extra
```

摘完剩 `/some/root extra`，`--force` 不在里面。所以经外壳不会有阶段被强制跑；直接 `gate.sh --staged --force` 不经外壳，也就不前移 staged-green。这一句判错的是「可达性」，不是守卫该不该写：外壳里加一句判断 `PREFLIGHT_FORCE_GIVEN`（它在第 20 行之后就有值）只花一行，防的是以后有人让外壳转发 `--force`；它今天堵不了任何现有口子，不能拿它当庚二比别的候选多出来的一条强项。（推的：我没有起 `gate.sh` 验强制阶段的退出码，只读了上面三处与 `small-checks.sh` 的输出。）

什么现象会推翻这一句：造得出一趟经 `gate-staged.sh` 的运行，`--force` 到了里层 `gate.sh`（例如外壳改成转发）；或者 `preflight.py` 里有不靠 `--force` 选项就返回 `forced` 的路（我没读 `preflight.py`）。

## 四、H3 那一格：两种代价口径在庚二下各量一遍

第一轮判决第二节 H3 一行：两腿的「丙会 / 不会把别的会话判绿过的行算进窗口」对不上，原因是两腿量的是两种情形（`research/prompts/c439-r1-main-verification.md:33`）。我把两种情形各写成一段共用一个暂存区的两会话历史，引擎里叫：

- 退出暂存口径 `h16_unstaged_other`（别的会话先把那行退出暂存再跑绿）：写行的会话暂存好行并跑绿（那行进窗口一次）；另一个会话把它退出暂存、在不含它的暂存区上跑绿，staged-green 于是挂上不含它的树；写行的会话重新暂存它。
- 留在暂存口径 `h14_rejudge_cost`（那行留在暂存里被判过）：另一个会话暂存好行、gate-staged 跑绿、提交。
- 附 `h19_index_moves`：写行的会话跑的当中，另一个会话往暂存区放了别的东西（庚二前移前还比不比暂存树）。

「重判」= 前缀之后，被追踪的那一行又进了一次 `gate_added_lines` 的输出。先看固定续作 `SS` 的逐趟原样输出（庚二 `geng2` 与今天 `today`；`staged-green -> tree …` 那几行是引擎对每个候选都打的外壳前移记录，只有以 staged-green 为基准的候选才用它）。

退出暂存口径，庚二：

```
=== hist=h16_unstaged_other cand=geng2 cont=SS
    run=1 path=S who=A head=1ad82d9 base=5c3364eb1b1a track_in_window=no verdict=green
      staged-green -> tree af16447ab07a
    commit bf379ed
    run=2 path=S who=A head=bf379ed base=cdd45ba2384a track_in_window=yes verdict=green
      staged-green -> tree 88957a8cdf9a
    另一个会话把 .claude/kb/b.md 退出暂存
    run=3 path=S who=B head=bf379ed base=2c9e7c0d9b6f track_in_window=no verdict=green
      staged-green -> tree af16447ab07a
    写行的会话重新暂存 .claude/kb/b.md
    run=4 path=S who=A head=bf379ed base=cdd45ba2384a track_in_window=yes verdict=green
      staged-green -> tree 88957a8cdf9a
    run=5 path=S who=A head=bf379ed base=2c9e7c0d9b6f track_in_window=no verdict=green
      staged-green -> tree 88957a8cdf9a
result cand=geng2 hist=h16_unstaged_other cont=SS judged_ever=yes runs=2,4 prefix_runs=3 after_prefix=4 after_count=1 total_runs=5
```

退出暂存口径，今天：

```
=== hist=h16_unstaged_other cand=today cont=SS
    run=1 path=S who=A head=1ad82d9 base=b14e019685ba track_in_window=no verdict=green
      staged-green -> tree af16447ab07a
    commit bf379ed
    run=2 path=S who=A head=bf379ed base=1ad82d9d8e13 track_in_window=yes verdict=green
      staged-green -> tree 88957a8cdf9a
    另一个会话把 .claude/kb/b.md 退出暂存
    run=3 path=S who=B head=bf379ed base=1ad82d9d8e13 track_in_window=no verdict=green
      staged-green -> tree af16447ab07a
    写行的会话重新暂存 .claude/kb/b.md
    run=4 path=S who=A head=bf379ed base=1ad82d9d8e13 track_in_window=yes verdict=green
      staged-green -> tree 88957a8cdf9a
    run=5 path=S who=A head=bf379ed base=1ad82d9d8e13 track_in_window=yes verdict=green
      staged-green -> tree 88957a8cdf9a
result cand=today hist=h16_unstaged_other cont=SS judged_ever=yes runs=2,4,5 prefix_runs=3 after_prefix=4,5 after_count=2 total_runs=5
```

留在暂存口径，庚二：

```
=== hist=h14_rejudge_cost cand=geng2 cont=SS
    run=1 path=S who=A head=1ad82d9 base=5c3364eb1b1a track_in_window=no verdict=green
      staged-green -> tree af16447ab07a
    commit bf379ed
    run=2 path=S who=B head=bf379ed base=cdd45ba2384a track_in_window=yes verdict=green
      staged-green -> tree 88957a8cdf9a
    commit 6e00db8
    run=3 path=S who=A head=6e00db8 base=2c9e7c0d9b6f track_in_window=no verdict=green
      staged-green -> tree 88957a8cdf9a
    run=4 path=S who=A head=6e00db8 base=2c9e7c0d9b6f track_in_window=no verdict=green
      staged-green -> tree 88957a8cdf9a
result cand=geng2 hist=h14_rejudge_cost cont=SS judged_ever=yes runs=2 prefix_runs=2 after_prefix=none after_count=0 total_runs=4
```

留在暂存口径，今天：

```
=== hist=h14_rejudge_cost cand=today cont=SS
    run=1 path=S who=A head=1ad82d9 base=b14e019685ba track_in_window=no verdict=green
      staged-green -> tree af16447ab07a
    commit bf379ed
    run=2 path=S who=B head=bf379ed base=1ad82d9d8e13 track_in_window=yes verdict=green
      staged-green -> tree 88957a8cdf9a
    commit 6e00db8
    run=3 path=S who=A head=6e00db8 base=bf379ed0fa00 track_in_window=yes verdict=green
      staged-green -> tree 88957a8cdf9a
    run=4 path=S who=A head=6e00db8 base=bf379ed0fa00 track_in_window=yes verdict=green
      staged-green -> tree 88957a8cdf9a
result cand=today hist=h14_rejudge_cost cont=SS judged_ever=yes runs=2,3,4 prefix_runs=2 after_prefix=3,4 after_count=2 total_runs=4
```

附，庚二（`geng2`，前移前比暂存树）与去掉那一比的庚二（`geng2_nc`）：

```
=== hist=h19_index_moves cand=geng2 cont=SS
    run=1 path=S who=A head=1ad82d9 base=5c3364eb1b1a track_in_window=no verdict=green
      staged-green -> tree af16447ab07a
    commit bf379ed
    run=2 path=S who=A head=bf379ed base=cdd45ba2384a track_in_window=yes verdict=green
      staged-green 不动：暂存树 88957a8cdf9a -> 2997ef8efe29
    run=3 path=S who=A head=bf379ed base=cdd45ba2384a track_in_window=yes verdict=green
      staged-green -> tree 2997ef8efe29
    run=4 path=S who=A head=bf379ed base=c3234ab67272 track_in_window=no verdict=green
      staged-green -> tree 2997ef8efe29
result cand=geng2 hist=h19_index_moves cont=SS judged_ever=yes runs=2,3 prefix_runs=2 after_prefix=3 after_count=1 total_runs=4
=== hist=h19_index_moves cand=geng2_nc cont=SS
    run=1 path=S who=A head=1ad82d9 base=5c3364eb1b1a track_in_window=no verdict=green
      staged-green -> tree af16447ab07a
    commit bf379ed
    run=2 path=S who=A head=bf379ed base=cdd45ba2384a track_in_window=yes verdict=green
      staged-green -> tree 88957a8cdf9a
    run=3 path=S who=A head=bf379ed base=2c9e7c0d9b6f track_in_window=no verdict=green
      staged-green -> tree 2997ef8efe29
    run=4 path=S who=A head=bf379ed base=c3234ab67272 track_in_window=no verdict=green
      staged-green -> tree 2997ef8efe29
result cand=geng2_nc hist=h19_index_moves cont=SS judged_ever=yes runs=2 prefix_runs=2 after_prefix=none after_count=0 total_runs=4
```

再放开用户动作：字母表 `Sck`（只含 `--staged` 一路），长度 0..4 的全部守规矩续作（每次提交紧跟在一次 `S` 之后），每格是「合计 / 单条续作最大 / 至少重判一次的续作条数 / 守规矩续作条数」，原样输出：

```
name=h3cost hist=h14_rejudge_cost today=40/4/20/20 bing=0/0/0/20 geng2=0/0/0/20 geng2_nc=0/0/0/20 yi_gate_mb=20/1/20/20 ding_tree_fixed=0/0/0/20
name=h3cost hist=h16_unstaged_other today=46/4/20/20 bing=20/1/20/20 geng2=20/1/20/20 geng2_nc=20/1/20/20 yi_gate_mb=44/4/20/20 ding_tree_fixed=40/4/20/20
name=h3cost hist=h19_index_moves today=46/4/20/20 bing=20/1/20/20 geng2=20/1/20/20 geng2_nc=0/0/0/20 yi_gate_mb=44/4/20/20 ding_tree_fixed=40/4/20/20
```

读法：

| 口径 | 庚二 | 今天 | 说明 |
|---|---|---|---|
| 退出暂存口径 | 每条续作恰好重判 1 次（20/1/20/20） | 合计 46、单条最多 4 次 | 庚二重判一次后 staged-green 回到含它的树，之后不再进窗口；今天的窗口从 HEAD~1 起，那行没提交前每一趟都进 |
| 留在暂存口径 | 0 次（0/0/0/20） | 合计 40、单条最多 4 次 | 庚二下第一轮判决说的「别的会话判绿过的行再进窗口」不发生；今天每一趟都重判到它被提交且 HEAD~1 越过它为止 |

两种口径下庚二都不漏判（第一轮判决已判这一点，上面的 `h14`、`h16` 在 `out/summary-L3.txt` 里 `name=cover` 两行都是 0/135），只在退出暂存口径多付一次重判。单位是「那一段被退出暂存的行」：按第一轮攻方腿在 `f111c3f3` 上量的 1326 行（25 份页）算，一页平均约 53 行（`python3 -c "print(round(1326/25, 2))"` 输出 53.04，1326 与 25 取自 `research/prompts/c439-r1-opus-model/out/real.txt` 第 4 行的 `quoted_in_scope=1326 files=25`，我没有复跑那一份），一次退出暂存口径的代价是多判约一页的行（推的）。

附的 `h19`：庚二收尾比暂存树（`scripts/gate/gate-staged.sh:39`）时，另一个会话往暂存区放东西会让这一趟不前移，下一趟多重判 1 次；被判的树就是开跑时的 write-tree 的树时，这一比不再是正确性所需（被判的就是 tree28），去掉它（`geng2_nc`）重判 0 次，覆盖逐段与 `geng2` 相同（`name=same_as_geng2 cand=geng2_nc same=23/23`）。是否去掉是外壳的取舍，不是庚二能不能成立的条件。

什么现象会推翻这一节：引擎里 `h16_unstaged_other` 的庚二逐趟日志里，staged-green 在另一个会话那一趟没有回到不含该行的树（那就不会有第二次进窗口），或 `h14_rejudge_cost` 的庚二出现第二次进窗口。

## 五、K1：庚二的落法会不会再开口子

外壳的 write-tree 与里层建出的临时树逐字节是不是同一棵，我在 `k1-tree-identity.sh` 里拿真的 git 量：今天的落法照 `gate.sh` 第 56、59、87 行做（`diff --cached --binary`、`worktree add --detach HEAD`、`apply --index`）；庚二的落法是 `worktree add --detach HEAD` 之后 `read-tree --reset -u <外壳 write-tree 的树>`。钩子位置：`m1` 在外壳 write-tree 之后、里层 diff 之前（第 56 行之前）；`m2` 在 diff 之后、worktree add 之前（第 59 行之前）；`m3` 在整趟判完之后、外壳收尾比树之前（`scripts/gate/gate-staged.sh:39` 之前）。`today_leak=yes` = 被判的树缺了 tree28 里有的东西，外壳却照前移。原样输出（12 个静态案、9 个并发案）：

```
== 静态案
name=k1 case=static-plain tree28=9b046a255426 today_tree=9b046a255426 today_tree_eq=yes geng2_tree=9b046a255426 geng2_tree_eq=yes files_eq=yes today_judged_lacks=none today_shell_advances=yes today_leak=no geng2_judged_lacks=none geng2_shell_advances=yes geng2_leak=no
name=k1 case=static-mode_link tree28=911ba0201d71 today_tree=911ba0201d71 today_tree_eq=yes geng2_tree=911ba0201d71 geng2_tree_eq=yes files_eq=yes today_judged_lacks=none today_shell_advances=yes today_leak=no geng2_judged_lacks=none geng2_shell_advances=yes geng2_leak=no
name=k1 case=static-binary tree28=47fc894fba0d today_tree=47fc894fba0d today_tree_eq=yes geng2_tree=47fc894fba0d geng2_tree_eq=yes files_eq=yes today_judged_lacks=none today_shell_advances=yes today_leak=no geng2_judged_lacks=none geng2_shell_advances=yes geng2_leak=no
name=k1 case=static-gitlink tree28=620a03a3859e today_tree=620a03a3859e today_tree_eq=yes geng2_tree=620a03a3859e geng2_tree_eq=yes files_eq=yes today_judged_lacks=none today_shell_advances=yes today_leak=no geng2_judged_lacks=none geng2_shell_advances=yes geng2_leak=no
name=k1 case=static-filter tree28=bca512c17990 today_tree=bca512c17990 today_tree_eq=yes geng2_tree=bca512c17990 geng2_tree_eq=yes files_eq=yes today_judged_lacks=none today_shell_advances=yes today_leak=no geng2_judged_lacks=none geng2_shell_advances=yes geng2_leak=no
name=k1 case=static-eol tree28=5acd93828774 today_tree=5acd93828774 today_tree_eq=yes geng2_tree=5acd93828774 geng2_tree_eq=yes files_eq=yes today_judged_lacks=none today_shell_advances=yes today_leak=no geng2_judged_lacks=none geng2_shell_advances=yes geng2_leak=no
name=k1 case=static-ita tree28=b518ee31bd69 today_tree=b518ee31bd69 today_tree_eq=yes geng2_tree=b518ee31bd69 geng2_tree_eq=yes files_eq=yes today_judged_lacks=none today_shell_advances=yes today_leak=no geng2_judged_lacks=none geng2_shell_advances=yes geng2_leak=no
name=k1 case=static-typechange tree28=d94c9c18bca1 today_tree=d94c9c18bca1 today_tree_eq=yes geng2_tree=d94c9c18bca1 geng2_tree_eq=yes files_eq=yes today_judged_lacks=none today_shell_advances=yes today_leak=no geng2_judged_lacks=none geng2_shell_advances=yes geng2_leak=no
name=k1 case=static-rename tree28=7bfe8b090bb3 today_tree=7bfe8b090bb3 today_tree_eq=yes geng2_tree=7bfe8b090bb3 geng2_tree_eq=yes files_eq=yes today_judged_lacks=none today_shell_advances=yes today_leak=no geng2_judged_lacks=none geng2_shell_advances=yes geng2_leak=no
name=k1 case=static-empty tree28=4ad275939061 today_tree=4ad275939061 today_tree_eq=yes geng2_tree=4ad275939061 geng2_tree_eq=yes files_eq=yes today_judged_lacks=none today_shell_advances=yes today_leak=no geng2_judged_lacks=none geng2_shell_advances=yes geng2_leak=no
name=k1 case=static-space tree28=d632050b1c18 today_tree=d632050b1c18 today_tree_eq=yes geng2_tree=d632050b1c18 geng2_tree_eq=yes files_eq=yes today_judged_lacks=none today_shell_advances=yes today_leak=no geng2_judged_lacks=none geng2_shell_advances=yes geng2_leak=no
name=k1 case=static-conflict write-tree=fail(外壳第 28 行就退 2，不进里层)
== 并发案（钩子位置见文件头）
name=k1 case=race-extra-stage-before-diff tree28=1813b062942f today_tree=2799ae01c7a8 today_tree_eq=no geng2_tree=1813b062942f geng2_tree_eq=yes files_eq=no today_judged_lacks=none today_shell_advances=no today_leak=no geng2_judged_lacks=none geng2_shell_advances=no geng2_leak=no
name=k1 case=race-unstage-before-diff tree28=1813b062942f today_tree=b518ee31bd69 today_tree_eq=no geng2_tree=1813b062942f geng2_tree_eq=yes files_eq=no today_judged_lacks=two.txt today_shell_advances=no today_leak=no geng2_judged_lacks=none geng2_shell_advances=no geng2_leak=no
name=k1 case=race-unstage-readd-before-diff tree28=1813b062942f today_tree=1813b062942f today_tree_eq=yes geng2_tree=1813b062942f geng2_tree_eq=yes files_eq=yes today_judged_lacks=none today_shell_advances=yes today_leak=no geng2_judged_lacks=none geng2_shell_advances=yes geng2_leak=no
name=k1 case=race-commit-all-before-diff tree28=1813b062942f today_tree=1813b062942f today_tree_eq=yes geng2_tree=1813b062942f geng2_tree_eq=yes files_eq=yes today_judged_lacks=none today_shell_advances=yes today_leak=no geng2_judged_lacks=none geng2_shell_advances=yes geng2_leak=no
name=k1 case=race-commit-all-before-worktree tree28=1813b062942f today_tree= today_tree_eq=fail(rc=4;error: patch failed: two.txt:1 error: two.txt: patch does not apply ) geng2_tree=1813b062942f geng2_tree_eq=yes files_eq=na today_judged_lacks=none today_shell_advances=yes today_leak=no geng2_judged_lacks=none geng2_shell_advances=yes geng2_leak=no
name=k1 case=race-commit-unrelated-before-worktree tree28=1813b062942f today_tree=38f80676fdb0 today_tree_eq=no geng2_tree=1813b062942f geng2_tree_eq=yes files_eq=no today_judged_lacks=none today_shell_advances=no today_leak=no geng2_judged_lacks=none geng2_shell_advances=no geng2_leak=no
name=k1 case=race-soft-reset-before-worktree tree28=1813b062942f today_tree=1f4d04b17ef6 today_tree_eq=no geng2_tree=1813b062942f geng2_tree_eq=yes files_eq=no today_judged_lacks=base.txt today_shell_advances=yes today_leak=yes geng2_judged_lacks=none geng2_shell_advances=yes geng2_leak=no
name=k1 case=race-extra-stage-before-worktree tree28=1813b062942f today_tree=1813b062942f today_tree_eq=yes geng2_tree=1813b062942f geng2_tree_eq=yes files_eq=yes today_judged_lacks=none today_shell_advances=no today_leak=no geng2_judged_lacks=none geng2_shell_advances=no geng2_leak=no
name=k1 case=race-unstage-then-readd-after-judge tree28=1813b062942f today_tree=b518ee31bd69 today_tree_eq=no geng2_tree=1813b062942f geng2_tree_eq=yes files_eq=no today_judged_lacks=two.txt today_shell_advances=yes today_leak=yes geng2_judged_lacks=none geng2_shell_advances=yes geng2_leak=no
```

计数（`grep -c` 数出来的）：20 个能落地的案里，庚二的落法 20/20 与 tree28 同棵；今天的落法 14 同、5 不同、1 套不上补丁（`race-commit-all-before-worktree`：「error: patch failed: two.txt:1」，里层 `die`，是封死的失败）；`today_leak=yes` 两案（`race-soft-reset-before-worktree`、`race-unstage-then-readd-after-judge`），庚二 0 案。两种落法落到磁盘的文件逐字相同的有 14 案（11 个静态案 + 3 个并发案，正是今天的落法也与 tree28 同棵的那 14 案）。另有 1 案（`static-conflict`）索引里有冲突，write-tree 本身失败，外壳第 28 行就退出，不进里层。

| 子问 | 判 | 证据 |
|---|---|---|
| 上游 `gate.sh --staged` 收一棵树 | 站得住 | 把第 56、87 行的 diff + apply 换成对外壳已导出的树做 read-tree；树的来源「SINGLEFS_STAGED_TREE="$staged_tree"」（`scripts/gate/gate-staged.sh:31`）今天就在环境里。没给这个变量或对象不在（不是外壳起的）就退回老做法，那时没有外壳，staged-green 不动。要改上游，C836 的改法一列同样写「里层收一棵树要改上游」，等发版。顺带：`stage-must-run.sh` 复用判据的依据是「--staged 恰好完整验了这一句」（`scripts/gate/stage-must-run.sh:33`），落法换了之后这一句才字面成立（K2 归攻方，我只指这一句） |
| 外壳自己建临时树，不带 `--staged` 直接起里层 | 站得住，但机件重复（推的） | 不用等上游发版，外壳要自己重做上游这几步：规范副本不在时拷（「if [[ -d "$source_repository_root/.claude/$staged_family" && ! -d "$staged_tree/.claude/$staged_family" ]]; then」（`.claude/singlefs-ai-sop/scripts/gate.sh:91`））、cargo 前缀登记缺了就拒（`.claude/singlefs-ai-sop/scripts/gate.sh:96`）、给里层导出 `GATE_STAGED_FROM`（「GATE_BASE="$staged_diff_base" GATE_STAGED_FROM="$source_repository_root" bash "$staged_gate"」（`.claude/singlefs-ai-sop/scripts/gate.sh:115`），里层读它的地方是「STAGED_FROM="${GATE_STAGED_FROM:-}"; unset GATE_STAGED_FROM」（`.claude/singlefs-ai-sop/scripts/gate.sh:131`））。漏什么：上游以后改这几步，外壳那一份不跟；我没有造出具体的漏判历史，因为没起过 `gate.sh` |
| 外壳用私有 `GIT_INDEX_FILE` 把暂存区冻住再起里层，不改上游 | 站不住 | 里层开头清掉 git 的本地环境变量：「unset $(git rev-parse --local-env-vars 2>/dev/null) 2>/dev/null || true」（`.claude/singlefs-ai-sop/scripts/gate.sh:20`）；`small-checks.sh` 实测 `git rev-parse --local-env-vars` 的第 10 项就是 `GIT_INDEX_FILE`（下面原样输出），私有索引在里层第一行就被清掉 |
| 前移也搬进上游 `gate.sh --staged` | 站得住（推的） | 三个守卫（调用方原来的 `GATE_BASE`、有没有阶段被强制跑、实际判的是哪棵树）的事实都在里层知道；外壳只看退出码与树，知道得少。代价：共享脚本里要写本仓的 ref 名，且上游发版 |
| 规范副本的拷贝 | 不引入差异 | 它被 `.gitignore` 挡着（`git check-ignore -v` 输出 `.gitignore:8:/.claude/singlefs-ai-sop/`），不进任何一棵树；里层判的是源仓工作区那份副本而不是 tree28 里的，今天的落法与庚二的落法一样，不是庚二引入的口子 |
| cargo 前缀登记 | 不引入差异 | 缺了就 `die`（「if [[ -f "$source_repository_root/$CARGO_COMMAND_PREFIX_FILE" && ! -f "$staged_tree/$CARGO_COMMAND_PREFIX_FILE" ]]; then」（`.claude/singlefs-ai-sop/scripts/gate.sh:96`）），是封死的失败，两种落法同 |
| HEAD 在两步之间被挪动、暂存区在两步之间被改走又改回 | 庚二的落法不受影响 | 并发案 9/9：庚二的树都等于 tree28，今天的 5/9 不同、1/9 套不上、2/9 缺东西且外壳照前移 |
| 子模块（gitlink）、符号链接、可执行位、二进制、过滤器、eol 属性、`add -N`、文件变目录、改名带改动、空文件、中文与空格路径 | 都同棵 | 静态案 11/11：两种落法都等于 tree28，文件逐字同；这个仓本身没有 filter 配置（`git config --list --show-origin` 里没有 filter 与 autocrlf 行），过滤器与 eol 两案是假设性的 |
| 守卫：人给 `GATE_BASE` | 放外壳 | 外壳读的是它自己的环境，与里层第 102 行读的是同一个变量（「staged_diff_base="${GATE_BASE:-$(diff_base "$source_repository_root")}"」（`.claude/singlefs-ai-sop/scripts/gate.sh:102`））。放里层不行：里层起下一层时把 `GATE_BASE` 设成它算好的值（第 115 行），下游看不出是不是人给的；前移的动作又在外壳，要另立一个传话的约定。外壳守卫漏什么：人每一趟都带 `GATE_BASE=HEAD`（G-only 的 258 条续作）——那是人自己每次收窄窗口，守卫不拦也不该拦 |
| 守卫：`--force` | 经外壳不可达 | 见第三节末「额外一句判错」；在外壳写一句判断 `PREFLIGHT_FORCE_GIVEN` 只是防御 |
| 外壳收尾比暂存树（`scripts/gate/gate-staged.sh:39`） | 庚二的落法下可以去掉 | 被判的就是 tree28，前移 tree28 本身不错；去掉后重判少一次（第四节附），覆盖逐段不变（`geng2_nc`） |

```
== 二、worktree 与主仓共用 refs/sop
worktree_sees_equal_main=yes
== 三、git rev-parse --local-env-vars 里的索引变量
found_at_position=10:GIT_INDEX_FILE
```

什么现象会推翻这一节：庚二的落法在某一案上建出的树不等于 tree28（我量了 20 案，没有）；`git config` 里真有 filter 或 autocrlf 的话，过滤器与 eol 两案的结论要在有配置的仓上重量；`gate.sh --staged` 里有一步不经索引、直接改临时树的文件（我只读了第 55 到 127 行，没有读到）。

## 六、K4：庚二的射程

三种射程，引擎里只建得出前两种的覆盖差异（引擎只有一个格，别的格没建模）：

- 窄射程：在 `gate_diff_base` 里加第三种取法（暂叫 `judged`）：读 `refs/sop/staged-green` 的提交，读不到就取空树（全量判，不取 HEAD——取 HEAD 就把 H1 的形原样请回来）；只有「判新增行内容」的格用它。
- 宽射程：把 `gate_diff_base gate` 整个改成先看 staged-green。
- 外壳导出：外壳起里层前把 `GATE_BASE` 设成 staged-green 的提交，零上游改动，只管带 `--staged` 的一路，但等于宽射程的这一路。

| 子问 | 判 | 证据 |
|---|---|---|
| 只给 quoted-result-lines（或「新增行」类的格），还是 `gate_diff_base gate` 整个改 | 窄的站得住；整个改会把「这一轮」的问题一起拉宽 | `gate_diff_base gate` 的使用者问的不都是「自上次判过以来新增的行有没有错」。archive-past-rounds 删的是「在 HEAD 里有、且这一轮没碰过」的文件（`scripts/kb/archive-past-rounds.py:28`），而这一轮的起点必须与 knowledge-sync 格同一个基准：「的起点与门禁 doc-process-records 的 knowledge-sync 格同一个基准」（`scripts/kb/archive-past-rounds.py:29`），两道基准不一致的后果是「每次提交之后，上一轮的同步记录必然同时满足」（`scripts/kb/archive-past-rounds.py:87`）那场打架；batch-scope 格有意取 HEAD：「基准是 HEAD（工作区、暂存区与未跟踪），不是 GATE_BASE」（`.claude/gate.d/doc-process-records.sh:37`），knowledge-sync 取的是「一轮可以跨几个提交，所以取 GATE_BASE」（`.claude/gate.d/doc-process-records.sh:38`）。宽射程把这些「这一轮」的窗口拉到 staged-green（此刻停在 2026-09-22）。推的：knowledge-sync 要回扫三周的触发文件，archive-past-rounds 把三周的记录都当本轮保留；这些都不是 C439 要的 |
| 窄的具体射程 | doc-experiments 的三个格，不含另两处 `gate_added_lines` 调用点 | doc-experiments 只取一份新增行：「if gate_added_lines "$shared_base" .claude/kb ':(glob)research/**/*.md' > "$SHARED_ADDED_LINES"; then added_lines_ok=1; fi」（`.claude/gate.d/doc-experiments.sh:139`），喂给 quoted-result-lines（「ADDED_LINES="$SHARED_ADDED_LINES"」（`.claude/gate.d/doc-experiments.sh:1621`））、results-cited（「只判这次改动新增或改写的点名行（射程与 quoted-result-lines 格同一条）」（`.claude/gate.d/doc-experiments.sh:346`））、evidence-in-repo（「按要求才跑的实验要读实验页这一轮新加的行」（`.claude/gate.d/doc-experiments.sh:471`））；这三格要改，得给它们单独取一份新增行（第二个基准），路径窗口 `shared_base` 不动。另两处与同格的 `gate_changed_paths` 配对用同一个基准：knowledge-sync 的「if added_record_lines="$(gate_added_lines "$gate_base" 'research/prompts/*-sync.md')"; then added_record_lines_status=0; else added_record_lines_status=1; fi」（`.claude/gate.d/doc-process-records.sh:368`）与同格的 `gate_base`，doc-decisions 的「added_out="$(gate_added_lines "$BASE" "$f")" || {」（`.claude/gate.d/doc-decisions.sh:1258`）与「all_changed="$(gate_changed_paths "$BASE" untracked)" || {」（`.claude/gate.d/doc-decisions.sh:1246`）；同一格里两半各用一个基准会自相矛盾，所以不改。（推的：我没有造这两格的历史） |
| 不带 `--staged` 那条路改不改 | 要改；新取法与走哪条路无关，两条路读同一条 ref | 只改带 `--staged` 的一路（引擎里 `geng2_s`）：14 段历史上守规矩续作一次都没判到 789/2274，庚二 258/2274（`out/summary-L3.txt` 的 `name=total14`）；多出来的是只跑不带 `--staged` 整轮的续作，原样输出见下。代价：不带 `--staged` 的整轮不前移 staged-green（「③ 不带 `--staged` 跑不许前移，所以这个外壳只做 `--staged` 这一条路。」（`scripts/gate/gate-staged.sh:15`）），只跑它的人窗口只增不缩（推的：窗口宽度 = 距上次 `--staged` 整轮全绿多久） |
| 各自要动 C528 现状说明的哪一句 | 窄：加一句、限定一句；宽与外壳导出：第一句不再成立 | C528 的第一句「上游 `gate.sh --staged` 在里层把 `GATE_BASE` 设成上游 `diff_base` 的基准」（`.claude/kb/checks-owed.md:354`）：外壳导出之后里层第一档认到的是 staged-green，宽射程之后不带 `--staged` 的一路也读 staged-green，这句要重写；窄射程对路径窗口（`gate_changed_paths`）这句仍对，对新增行的格不再对，加限定即可。「同一批改动两条路判的窗口不同」（`.claude/kb/checks-owed.md:354`）：窄射程对新增行的格抹平了这个差。「两条路的窗口差异照实写进 `scripts/gate/changed-paths.sh` 文件头（已写）」（`.claude/kb/checks-owed.md:354`）：文件头第 10 到 13 行要加第三种取法。C449 并入的豁免表那一句「取相对 `refs/sop/staged-green` 的变没变，是第三套算法」（`.claude/kb/checks-owed.md:354`）要再加一行「新增行的格读 staged-green」。三种射程都不碰的：`base_of` 说明里那句待改（同一行状态列写着还剩它） |
| 碰不碰用户 2026-09-24 的定案 | 字面不碰，但要照跑前条款交用户 | 定案是共用脚本不认 `GATE_DIFF_BASE`：「不认上游 gate.sh 导出的 GATE_DIFF_BASE（用户 2026-09-24 定）」（`scripts/gate/changed-paths.sh:12`）；三种射程都不读 `GATE_DIFF_BASE`。但跑前条款是「留下的候选要动用户 2026-09-24 的定案 ⇒ 交用户，不由判决定」（`research/prompts/_c439-r1-body.md:50`），触发观测里有「改两条路窗口关系的」：窄射程让新增行的格两条路取同一个基准，关系变了。所以三种都要交用户，窄射程动的格最少 |
| 漏改的那条路留下什么形 | 见下 | ① 只改带 `--staged` 的一路：只跑不带 `--staged` 整轮的续作漏，例 `geng2_s-n-only-bypass`、`geng2_s-n-only-c439`（下面原样输出）。② 窄射程之外的格：knowledge-sync 的新增记录行、doc-decisions 的已定标题，以及全部 `gate_changed_paths` 的使用者，原样留着 H1 的形（绕过门禁的提交里新增的行与路径之后永远不再进窗口；推的，没建模）。③ 外壳导出：上游阶段里读 `GATE_BASE` 的一起变宽，gate.sh 自己写着：「show-me-test 与版本纪律要读它」（`.claude/singlefs-ai-sop/scripts/gate.sh:130`）。④ 取不到 staged-green 时必须取空树：取 HEAD 就复现 H1 |

只跑不带 `--staged` 整轮的续作（`geng2_s` 是庚二只改带 `--staged` 的一路，`geng2` 两条路都改），原样输出：

```
id=geng2_s-n-only-bypass cand=geng2_s hist=h2_noverify cont=N judged_ever=no
id=geng2-n-only-bypass cand=geng2 hist=h2_noverify cont=N judged_ever=yes
id=geng2_s-n-only-c439 cand=geng2_s hist=h1_c439 cont=NN judged_ever=no
id=geng2-n-only-c439 cand=geng2 hist=h1_c439 cont=NN judged_ever=yes
```

什么现象会推翻这一节：窄射程之外的某个 `gate_diff_base gate` 使用者，换成 staged-green 之后并不拉宽它问的问题（我只读了 archive-past-rounds、doc-process-records 两处的说明与 doc-experiments、doc-decisions 的调用点，没有逐格读完十几个使用者）；或者 `geng2_s` 的 N-only 漏判在真格上被别的机制接住（引擎只建了一个格）。

## 七、站得住的那一面（替庚二与 wu_cf 说的话）

**庚二**

- 覆盖：23 段历史、3489 条守规矩续作里，带 N 或 S 而一次都没判到的 0 条；剩下的漏全是人每一趟都带 `GATE_BASE=HEAD` 的续作。对照（`out/summary-L3.txt`，原样）：

```
name=never_with_NS cand=bing never=786/3489
name=never_with_NS cand=bing_guard never=312/3489
name=never_with_NS cand=ding_tree never=74/3489
name=never_with_NS cand=ding_tree_fixed never=0/3489
name=never_with_NS cand=geng never=474/3489
name=never_with_NS cand=geng2 never=0/3489
name=never_with_NS cand=geng2_nc never=0/3489
name=never_with_NS cand=geng2_s never=670/3489
name=never_with_NS cand=today never=778/3489
name=never_with_NS cand=wu_cf never=114/3489
name=never_with_NS cand=wu_set never=342/3489
name=never_with_NS cand=yi_gate_mb never=342/3489
```

- 机件小：里层一处落法（上游）加外壳一句守卫；不新增 ref、不新增 notes、不要提交后钩子。
- 窗口在稳态下是「距上次 `--staged` 整轮全绿之后的改动」，不随历史变长；一次性的代价是第一次前移之前要先清掉此刻窗口里的红：第一轮量到 1326 行、355 行红、其中 101 行在任何产物里都找不到（`research/prompts/c439-r1-main-verification.md:34` 采的数，我没有复跑）。
- 多机：staged-green 是各 clone 本地的 ref，`git config --get-all remote.origin.fetch` 只有 `+refs/heads/*:refs/remotes/origin/*`，没有 push 配置；第二台自己判自己的，从本机拉来的行在第二台的窗口里会被判一次（推的）。
- 退出暂存口径多付一次重判；留在暂存口径 0 次（第四节）。

**wu_cf**

- 覆盖：14 段错行历史上守规矩续作一次都没判到 0/2274；`h11_dup`、`h12_exempt_grow` 各 0/135；人给 `GATE_BASE` 的那段（`h15_override`）0/183。不依赖任何 ref，交错、`GATE_BASE`、多机、改写历史都碰不到它。原样输出：

```
name=cover hist=h15_override bing=183/183 bing_guard=33/183 ding_lazy_fixed=33/183 ding_tree=53/183 ding_tree_fixed=33/183 geng=183/183 geng2=33/183 geng2_nc=33/183 geng2_s=92/183 today=116/183 wu_cf=0/183 wu_count=0/183 wu_fixed=0/183 wu_set=0/183 yi_clean=33/183 yi_gate_mb=33/183
```

- 残余：同页搬家（第三节戊）。
- 我替它辩不了的：与 `.claude/singlefs-ai-sop/rules/evidence-discipline.md@原样保存的证据不许事后改`：「不按全仓判」那一条（正文引产物的检查按这次改动判）的冲突（第一轮已记，不随豁免键消失），以及全量判在此刻 1794 行、455 行红的一次性代价（第一轮 H4）。
- 固定基线放在仓里的文件里时，绕过门禁的提交可以改基线本身；同类的改法（改门禁脚本、改外壳）对所有候选一样中，不分臂（推的，没建模）。

## 八、没做什么

- 没起任何门禁阶段、`gate.sh`、`gate-staged.sh`，没跑 cargo，没跑重型测试。引擎里的外壳与里层是照冻结行号重写的流程（沿用攻方腿的重写，加了三个候选与六段历史），与真脚本不一致的地方会让结论跟着错。推翻条件：主 agent 在临时仓里真跑 `gate-staged.sh` 复现 `bing-race-unstage`，再在庚二的落法下验 staged-green 不挂含错行的树。
- 没验：外壳自建临时树的机件重复只是读代码推的；`--force` 经外壳不可达读了 `gate-staged.sh`、`preflight.sh` 与 `small-checks.sh` 的输出，没有起 `gate.sh` 验强制阶段的退出码，也没读 `preflight.py`。
- 没建模：改写历史（amend、rebase）下庚二与丁的差异（只有 `h8_amend` 一段，覆盖 21/135 同为 G-only）；doc-experiments 之外的格；真并发（交错是在固定的几处插进去的）；`stage-must-run.sh` 的复用一侧（K2 归攻方）；产物重跑之后全量判要回改多少行（K5 归攻方）。
- 1326 行、25 页、355 行红、101 行真错是第一轮的数，我没有复跑；一页约 53 行是拿它们除出来的。
- 模型目录里 `tools/rerun.sh --selftest` 没起（脚本自证归主 agent 按 `.claude/hooks/selftest-registry.tsv` 登记起）。
- 没替主 agent 撤销或维持判决；「判对 / 判错 / 判不了」只是对第一轮每一行的复核。
- 没有必须由主 agent 跑的命令；要复核就照第二节两条命令。
- 草稿目录里的临时仓、`git worktree` 与仓副本交回前已删。

## 九、引文核对

`python3 scripts/agents/cite-check.py research/prompts/c439-r2-sonnet-output.md --root /home/fy5090/code/singlefs` 的末行（行号重算后跑的）：

```
  ✓ 核了 50 处引文、8 处裸行号，对上 50 处，没判 8 处（其中裸行号没有引文可比 7 处；逐处列在上面）
```

没判的 8 处：7 处是裸行号（没有引文可比，行号都在文件范围内），1 处是 k1 输出里 git 的报错原文，路径认不出。
