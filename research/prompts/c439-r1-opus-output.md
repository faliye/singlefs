# C439（gate-ok 前移让同轮早先写的行不再被判） 第一轮攻方腿报告（2026-10-15）

攻击面：正文「三、格」（H1–H4，重点是 H2 每个候选连同今天的取法）。只在自己目录下的临时 git 仓里造历史；不起门禁阶段、不跑 gate.sh / gate-staged.sh、不跑 cargo；主仓只读（另做了一份 `git clone --shared` 在草稿目录里量 H4）。

## 复跑

模型目录 `research/prompts/c439-r1-opus-model/`。一条命令复跑报告点名的全部例子、H1 表、长度 0..2 的全格扫描与 H4 的本仓量数，并与 `out/` 逐字比：

```
cd /home/fy5090/code/singlefs
bash scripts/hosts/run-with-memory-cap.sh 8G bash scripts/hosts/capped.sh 31 \
  bash research/prompts/c439-r1-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs \
  --frozen /home/fy5090/.local/share/singlefs/work/c439-r1/frozen --work <新的草稿目录>
```

这一次的原样输出（第二次独立复跑，与第一次落进 `out/` 的逐字比）：

```
name=compare file=examples.txt same=yes
name=compare file=h1-today.txt same=yes
name=compare file=grid-L2.txt same=yes
name=compare file=real.txt same=yes
rc=0
```

长度 3、4 的扫描（`out/grid-L3.txt`、`out/grid-L4.txt` 的来处）太久，不在 rerun.sh 里，命令照下面逐段跑（每段一个历史；`<候选…>` 见各格），再 `python3 grid.py <输出>/*/results.txt`：

```
bash scripts/hosts/capped.sh 31 bash research/prompts/c439-r1-opus-model/sweep.sh <冻结副本> <输出>/<历史> <历史> 3 31 NSckoG today jia yi_shared yi_session yi_gate_h1 yi_gate_mb bing ding_a ding_rw ding_tree wu_set wu_count wu_fixed wu_cf geng geng2
bash scripts/hosts/capped.sh 31 bash research/prompts/c439-r1-opus-model/sweep.sh <冻结副本> <输出>/<历史> <历史> 4 31 NSck today yi_gate_mb bing ding_tree geng2
```

（有上游的两段历史 h5_pushed、h2u_unpushed 字母表另加 p。）长度 3：19 段历史 × 16 个候选，83248 次；长度 4：19 段 × 5 个候选，36795 次；两份 results 的退出码全为 0。

文件 sha256：

```
abb46efbda14bca93baadb7ff662d5b8a0d05bb1365e23e9d90f731b6edbcd33  c439-model.sh
8e08ecce4d97f215ae66f06affce77c6f596092a8a6b8ce70dbf05e4b212311b  examples.sh
74477e56846750e69644adc04e8f26ec3b1703cae8f3535cb702418567fd1581  grid.py
21cb48d7f3cf4fcd8b0412a3b0c7fbd227770aa2deaaa3898b661f75fe4b33ec  measure-real-cell.py
e3e4f9f1652b984941b6a7cc9d847cd5a01a711edffc6585c37106430a697ca1  measure-real-history.py
160b0a7a219d7087489872a07aba793ccd0519bed7a474504d7cd141781c3782  measure-real.sh
de9f32ec00bda43d3601af17b8a59c6ffea66beac33c8557bbdf21a94a3a1aa7  sweep.sh
eed188f57f1999317e3e14874fb3bcebc4770552e4803865d808b7d2ea2b7d25  tools/rerun.sh
d0697cdc29b9327fbadac37acef6ac93f09b6218fc5b3896a64e4ec9f7020b26  wu_judge.py
0045120c2d5a86391362db9d0f0d8d6b59897dfd6e6a3dca57f23b486da119d4  out/examples.txt
1ce73ddf544d090358765582a9a7924db67d71f9ac97c194d47b7f3b9cae8e33  out/grid-L2.txt
4f788bea607f2ddf5a79f00b8df8ca71b9af6628e1e0a669318fecc78f76d58d  out/grid-L3.txt
a8f8726925b945a5140b8dc6205ab8e1f593a7e8cd22bbc9f6f7e0a27a738804  out/grid-L4.txt
bfdbe8c418ecfb5c61b89f79d62a598873def503b837778a4fa5f73272b0adc9  out/h1-today.txt
fcad2d00a09b9aef33d580c577f67e19af9bb189d04b62f2c9dd50cac11de3d7  out/real.txt
```

### 模型怎么造、哪些是冻结代码

- 每段历史一个新的临时仓（`git init`、提交时刻写死，所以提交号逐次一样）。根提交里放一行存量错引文（戊的豁免表里有它）；各候选的记号先挂在第二个提交上，再照流程一次 gate-staged 全绿、提交，算起点。
- 基准与窗口全用冻结代码：`scripts/gate/changed-paths.sh` 整份 source（`gate_diff_base`、`gate_added_lines`），`.claude/singlefs-ai-sop/scripts/lib.sh` 第 144–189 行的 `diff_base` 抽出来 source（整份要 preflight.sh，冻结副本里没有）。
- gate.sh / gate-staged.sh 本身不跑，照冻结行为重写：`--staged` 先 write-tree，再 `git diff --cached --binary`、`worktree add HEAD`、`apply --index`，基准在源仓上算后经 `GATE_BASE` 交进里层（「staged_diff_base="${GATE_BASE:-$(diff_base "$source_repository_root")}"」（`.claude/singlefs-ai-sop/scripts/gate.sh:102`））；gate-ok 只在不带 `--staged`、GATE_BASE 为空的全绿时前移（「elif [[ -n "$START_HEAD" && -z "${GATE_BASE:-}" ]]; then」（`.claude/singlefs-ai-sop/scripts/gate.sh:774`））；staged-green 照 `scripts/gate/gate-staged.sh` 第 28、38–46 行。
- 判红判绿只判 quoted-result-lines 的简化版：判的行里以 E7RESULT 开头、`research/results/*.out` 里逐字找不到就红。被追踪的那一行「在不在窗口」直接看 `gate_added_lines` 的输出（条款 2 的字面）。
- 用户动作放开扫（`.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」那一条）：固定前缀与故障之后，续作取字母表 N（不带 `--staged`）、S（gate-staged）、c（改无关文件、暂存、提交）、k（原样提交暂存区）、o（会话甲开一轮）、G（带 `GATE_BASE=HEAD` 的 gate-staged）、p（推上游）上长度 0..3（全部候选）或 0..4（五个候选）的全部序列。
- 「守规矩」的续作：每一次提交前紧挨着一次门禁（S、G 或 N；`.claude/skills/gate/staged-gate.md` 第 16 行两条路都许）。「永久」= 守规矩、跑过门禁的续作一条都没判到（扫描上界之内）。
- 对照：错引文只暂存、没提交时，16 个候选下一次 gate-staged 全都判到它（`out/examples.txt` 末尾 16 条 control，judged_ever=yes）。

## 各格判定一览

「永久」的读法见上；例子名是 `out/examples.txt` 里 `=== example=` 那一行的名字。候选的代号：today 今天的取法；jia 甲；yi_shared 乙（开轮的会话写，全仓一条 ref）；yi_session 乙（开轮的会话写，每会话一条 ref）；yi_gate_h1 乙（门禁写 gate-ok，两条路都读，不是祖先时照今天退 HEAD~1）；yi_gate_mb 乙（同上，不是祖先时退 merge-base，没有记号退空树）；bing 丙；ding_a 丁（门禁全绿时在开跑的 HEAD 上挂 notes）；ding_rw 丁（同上，另开 `notes.rewriteRef`，改写提交时记号跟着走）；ding_tree 丁（提交那一刻树等于 staged-green 的树才挂）；wu_set / wu_count / wu_fixed 戊（豁免按（路径、行文）集合 / 计数认，只缩不涨比 HEAD / 比引入时的固定基线）；wu_cf、geng、geng2 是这条腿自己加的（见「自己提的」一节，被攻过零轮）。

| 格 | 对象 | 判定 | 量过 / 推的 | 打中的条款 | 例子（历史 + 续作） |
|---|---|---|---|---|---|
| H1 | 今天的取法 | 打中：绕过门禁提交一行之后，只要再有一次提交，此后任何续作两条路都判不到它（长度 4 内 85/85，有上游且已推时 156/156）；不带 `--staged` 那条路在「没上游或已全推」时从来判不到已提交的行 | 量过 | 条款 1 不触发（造出了今天的漏判） | today-c439-original、today-bypass、today-split、today-other-session、today-pushed-pathN、today-pushed-commit |
| H1 | 本仓现场 | 打中：f111c3f3 上有 101 行整行抄的产物行在任何一版产物里都找不到（100 行在 E164 页正文），都在今天两条路的窗口外 | 量过（用重写的比对法） | 同上 | `out/real.txt` |
| H2 | 甲 jia | 打中，永久：绕过门禁提交，下一次门禁起就判不到（长度 3 内 135/135） | 量过 | 条款 2 | jia-bypass |
| H2 | 乙 yi_shared | 打中，永久：甲会话绕过门禁提交之后，乙会话开轮把共用的起点挪到它后面 | 量过 | 条款 3 | yi-shared-overwrite |
| H2 | 乙 yi_session | 打中（要「重开一轮」这一步）：别的会话绕过门禁的提交之后本会话开新一轮，起点越过它 | 量过 | 条款 3 | yi-session-reopen |
| H2 | 乙 yi_gate_h1 | 打中：门禁全绿过的提交被 amend 之后不再是祖先，退 HEAD~1，再一次提交就判不到；revert 形同样打中 | 量过 | 条款 2 | yi-gate-h1-amend、yi-gate-h1-revert |
| H2 | 乙 yi_gate_mb | 打中，永久：一行已提交而没判过，判的那棵树里删了它（暂存或工作区里改掉），门禁全绿把 gate-ok 挂到含它的 HEAD 上；之后改动被撤回或 revert，它回来，长度 4 内 114/114 判不到 | 量过 | 条款 2；工作区那一形是别的会话的未提交改动，也落条款 3 | yi-gate-mb-stamp-S、-stamp-N、-stamp-revert |
| H2 | 丙 bing | 打中，永久：gate-staged 跑的当中另一个会话把暂存区改走又改回原样（unstage 再 add、`stash` 再 `stash pop --index`、提交再 `reset --soft`），被判的树缺那一行，staged-green 却挂上含它的树（长度 4 内 182/182）；另有人带 `GATE_BASE=HEAD` 跑 gate-staged 时 staged-green 照样前移（182/182） | 量过 | 交错那三形条款 3；GATE_BASE 那一形条款 2 | bing-race-unstage、-stash、-commit-undo、bing-override |
| H2 | 丁 ding_a | 打中，永久：与 yi_gate_mb 同一形（记号挂在 HEAD 上，HEAD 里有判的树里删掉的行） | 量过 | 条款 2 | ding-a-stamp-S |
| H2 | 丁 ding_rw | 打中，永久：记号随 amend 带到新提交上，amend 进去的那一行判不到 | 量过 | 条款 2 | ding-rw-amend |
| H2 | 丁 ding_tree | 打中：丙的三种交错与 GATE_BASE 那一形之后，紧跟一次原样提交（k），之后永久（85/85） | 量过 | 条款 3 / 条款 2 | ding-tree-race、ding-tree-override |
| H2 | 戊 wu_set、wu_fixed | 打中，永久：存量里豁免的一行错引文，在同一页再抄一遍，新的那行也被豁免 | 量过 | 条款 2（无窗口，按「一次都没进判定」读） | wu-set-duplicate、wu-fixed-duplicate |
| H2 | 戊 wu_set、wu_count | 打中，永久：绕过门禁的提交同时加了错引文和它的豁免行，只缩不涨比 HEAD 看不到表涨了 | 量过 | 条款 2 | wu-set-grow、wu-count-grow |
| H2 | 庚二 geng2（零轮） | 没打中：19 段历史，长度 3（含 G）与长度 4（NSck），续作里只要有一次不带 GATE_BASE 的门禁就判到 | 量过 | — | geng2-override-caught、geng2-stamp-caught |
| H3 | 多会话 | 打中：见上面条款 3 的几格；另记代价：会话乙判绿过的行被算进甲的下一次窗口，yi_gate_mb 在 310/310 条跑过门禁的续作里都重判，bing / ding_tree / geng2 是 0/310 | 量过 | 条款 4（记代价） | `out/grid-L4.txt` 的 h14_rejudge_cost |
| H4 | 代价 | 丙此刻（f111c3f3、staged-green e2ae1837）的窗口里 1326 行（25 份页），照窗口判法找不到 355 行（14 份）；戊全量 1794 行（46 份），找不到 455 行（21 份） | 量过（用重写的比对法） | — | `out/real.txt` |
| 背景 | 「本仓没有 @{upstream}」 | 不成立：本仓有上游 origin/master，2026-09-22 以来推过 23 次；f111c3f3 那时已全推，所以两条路的基准与正文报的一样，原因不同 | 量过 | — | 「本仓的上游」一节 |

## H1：今天的取法在哪几种历史上永远判不到一行

### 两条路各取什么

- 不带 `--staged`：项目阶段走「gate：GATE_BASE 是个提交就用它；否则 @{upstream} 的 merge-base；都没有（或 merge-base 取不到）就 HEAD。」（`scripts/gate/changed-paths.sh:10`）。没上游或已全推时就是 HEAD，窗口只有工作区、暂存区与未跟踪文件：**已提交的行这条路一次都判不到**。
- 带 `--staged`：里层的 GATE_BASE 是上游 `diff_base`，在默认分支上而 merge-base 等于 HEAD 时取「# 优先级：上游 > 门禁上次通过的位置 > HEAD~1。」（`.claude/singlefs-ai-sop/scripts/lib.sh:167`）。gate-ok 只由不带 `--staged` 的全绿写，写的是开跑时的 HEAD（「if ! git -C "$ROOT" update-ref refs/sop/gate-ok "$START_HEAD" 2>/dev/null; then」（`.claude/singlefs-ai-sop/scripts/gate.sh:781`））。

### 每种历史末尾、再接一步时两条路的基准（`out/h1-today.txt`，量过）

被追踪的那一行 L 是一行错引文。「前缀」是起点之后的固定几步；表里四列是前缀之后再接的续作；「判到」= L 出现在那一次 `gate_added_lines` 的输出里。

| 历史（模型里的键） | 前缀 | N | S | c 再 N | c 再 S |
|---|---|---|---|---|---|
| C439 原形（h1_c439） | L 绕过门禁提交；不带 `--staged` 全绿（窗口 = HEAD，L 不在，gate-ok 挂到 L 那次提交） | 基准 HEAD，判不到 | gate-ok 等于 HEAD，退 HEAD~1，判到 | 判不到 | 基准 = gate-ok = L 那次提交，判不到 |
| 绕过门禁的提交（h2_noverify） | L 绕过门禁提交 | 判不到 | HEAD~1，判到 | 判不到 | HEAD~1 = L 那次提交，判不到 |
| 一次暂存分几次提交（h3_split） | 暂存两份、gate-staged 全绿、只提交一份；再把 L 加进暂存区，和剩下那份一起提交（「已经判过了」） | 判不到 | 判到 | 判不到 | 判不到 |
| 另一个会话在两次门禁之间提交（h4_other） | 甲 gate-staged 全绿、提交；乙把 L 写进另一页、提交 | 判不到 | 判到 | 判不到 | 判不到 |
| 全推出去之后（h5_pushed，有上游） | L 绕过门禁提交、推 | 基准 = 上游 merge-base = L 那次提交，判不到 | 上游 merge-base 等于 HEAD，退 HEAD~1，判到 | 判不到 | 上游 merge-base = L 那次提交，判不到 |
| 有上游、还没推（h2u_unpushed） | L 绕过门禁提交、不推 | 上游 merge-base 在 L 之前，判到 | 判到 | 判到 | 判到；推了再提交（cpS、pcS）才判不到 |

放开扫的结果（`out/grid-L4.txt` 之外另核，命令与输出见「复跑」的长度 4 结果）：前缀之后只要再有一次提交（续作以 c 开头），长度 4 之内所有续作都判不到 L —— h1_c439、h2_noverify、h3_split、h4_other、h8_amend、h15_override 各 85/85，h5_pushed 156/156；只跑 N 的续作 4/4 判不到。没推的 h2u_unpushed 以 c 开头的只有 62/156 判不到（都是中间推过的）。

结论：今天的漏判不要求 C439 原形里的 gate-ok 前移；**一次绕过门禁的提交（或一次 gate-staged 之后又往暂存区加东西再提交），后面再跟一次任意提交，那一行就永远在两条路的窗口外**（有上游时还要求中间推过一次）。C439 原形只是其中一种：前移把 gate-ok 挂到含 L 的提交上，之后 gate-staged 的窗口从那里起算。

推翻条件：在临时仓里照 `out/examples.txt` 的 today-bypass 那一段跑，若 `c 再 S` 那一次的 `gate_added_lines` 输出里有 L，这一格就不成立。

### 本仓的上游

正文「现场：本仓没有 `@{upstream}`」不成立。主仓只读现查（2026-10-15）：

```
$ git rev-parse --abbrev-ref '@{upstream}'
origin/master
$ git reflog show --date=short origin/master | head -2
c1e9b324 refs/remotes/origin/master@{2026-10-14}: update by push
f111c3f3 refs/remotes/origin/master@{2026-10-13}: update by push
```

2026-09-22 以来 origin/master 的 reflog 里推了 23 次（`git reflog show --date=short origin/master` 按日期数）。正文取样那一刻 HEAD = f111c3f3 = 上游，所以「`diff_base` 交出 HEAD~1、`gate_diff_base gate` 交出 HEAD」两个数都对，但走的是「已全推」那一支，不是「没上游」那一支。影响：本仓在「提交了、还没推」的那段时间里两条路都看得到已提交的行（上表最后一行），漏判要求中间推过一次；推得很勤（上面 23 次），所以实际上落在 h5_pushed 那一行。

### 本仓现场的漏判实例（量过，重写的比对法）

在 f111c3f3 的 `--shared` 克隆上，照 quoted-result-lines 格的射程取全部整行抄的产物行，按它的「全量判」比法（树里全部产物，加全部归档过的产物；比法照 `.claude/gate.d/doc-experiments.sh` 第 1710–1773 行重写，不是跑那一格）：

```
name=real_cell_window mode=full label=full quoted_in_scope=1794 not_found=101 files_with_not_found=2
name=real_cell_file mode=full label=full file=.claude/kb/experiments/164-崩溃放量新设计的小范围功能测试.md not_found=100
name=real_cell_file mode=full label=full file=.claude/kb/experiments/168-删除借保留量的环窗口.md not_found=1
```

E164 页那 100 行是产物行抄短了（不是整行）。例：页里「E7RESULT name=w3_scan commits=318 renames=433 test_file_renames=169」（`.claude/kb/experiments/164-崩溃放量新设计的小范围功能测试.md:35`），在正文里不在历史节；归档产物里的那一行后面还有 `head=…`、`crates_raw_digest=…` 两段（`git show '31128273^:research/results/e164-crash-amplification-design-function-test-2026-09-28-repository.out' | grep w3_scan`）。这一行 2026-09-28 随 dbcc0d2f 进仓，那一版的格已经按整行比（dbcc0d2f 里那一格写着 `stripped not in compared_lines`）。它此刻在今天两条路的窗口外，按上表永远判不到；丙的窗口（staged-green 起）与戊的全量都判得到它。

## H2：逐个候选的反例历史

每一形只写前缀与故障；续作放开扫过，数在一览表里。每一步许可它的那一句列在步后。

### 甲（jia，带不带 `--staged` 都取 HEAD）

历史：L 写进页、暂存、提交（临时仓不挂钩子；本仓 `.git/hooks/pre-commit` 只是时间窗闸、不跑门禁，正文第二节写明），之后任何门禁的窗口都只有未提交的东西。甲把今天不带 `--staged` 那条路的毛病搬到两条路上：今天至少 gate-staged 还看得到 HEAD~1 那一次提交（上表 S 列判到），甲连这个都没有。C439 改法一列的第一个改法「要么把基准改成「上一次提交」」就是甲，它在这一格上照样中。

### 乙（开轮的会话写起点）

- 全仓一条 ref（yi_shared）：甲会话开轮（起点 C0）；L 绕过门禁提交；乙会话开轮把那一条 ref 写成 HEAD。甲之后的每次门禁基准都越过了 L（`out/examples.txt` 的 yi-shared-overwrite，长度 3 内 135/135）。条款 3 字面：「同一份临时仓里两组操作交错，选定取法下另一组写的错引文漏掉」。
- 每会话一条 ref（yi_session）：乙会话把 L 写进另一页、提交、会话结束；甲会话开新一轮（起点 = HEAD），之后甲的窗口越过 L（yi-session-reopen）。不开新一轮时甲的窗口一直包着 L（扫描里只有带 o 的续作漏，41/135），所以它漏不漏取决于「谁什么时候开轮」——这一步是用户定的，放开扫之后照样有一条让它漏的路。会话结束而记号留着时，没人再拿那一条 ref 跑门禁；清掉它，它点着的那段就没人判。

### 乙（门禁写起点 = gate-ok 两条路都认）

- 回退 HEAD~1（yi_gate_h1，等于两条路都用今天上游的 `diff_base`）：gate-staged 全绿把 gate-ok 挂到 C1；amend C1 把 L 加进去；gate-ok 不再是 HEAD 的祖先（「&& git -C "$root" merge-base --is-ancestor "$gate_ok_commit" HEAD 2>/dev/null; then」（`.claude/singlefs-ai-sop/scripts/lib.sh:177`）不过），退到 HEAD~1（`.claude/singlefs-ai-sop/scripts/lib.sh:180`）；再一次提交之后 HEAD~1 就是 amend 出来的那个提交，L 永远在窗口外（yi-gate-h1-amend）。
- 回退 merge-base（yi_gate_mb）：amend 那一形被 merge-base 接住（yi-gate-mb-amend-caught，judged_ever=yes），但下面这一形打穿：
  1. L 绕过门禁提交（C2）。
  2. 有人把含 L 的那一段改掉，改动只在暂存区（或只在工作区——别的会话还没收尾的编辑）。
  3. gate-staged（或不带 `--staged`）全绿：被判的树里没有 L，窗口 gate-ok..被判的树里自然也没有；全绿后 gate-ok 写成开跑时的 HEAD = C2，而 C2 里有 L。
  4. 那份改动被撤回（`git restore`），或者提交之后又被 `git revert`：L 回到 HEAD。
  5. 之后 gate-ok..HEAD 的 diff 里 L 两头都在，永远不是新增（yi-gate-mb-stamp-S、-stamp-N、-stamp-revert；长度 4 内 114/114）。
  病根是「判的是 HEAD + 暂存（或工作区），记号却挂在 HEAD 这个提交上」：HEAD 里有、被判的树里删掉的行，被一起盖章。今天的 gate-ok 前移（`.claude/singlefs-ai-sop/scripts/gate.sh:781`）是同一个病根，C439 原形是它的特例。C439 改法一列的第二个改法「在前移时记下当轮起点」在这一格上照样中。

### 丙（staged-green 的树对这次的暂存树）

被判的树与前移到的树不是同一棵：gate-staged 在开跑时 write-tree 记下要前移的那棵（「staged_tree="$(git write-tree)"」（`scripts/gate/gate-staged.sh:28`）），gate.sh 却自己再取一次暂存区的 diff 套到当时的 HEAD 上（`.claude/singlefs-ai-sop/scripts/gate.sh:56`、`.claude/singlefs-ai-sop/scripts/gate.sh:59`），收尾只比「开跑时那棵」与「此刻这棵」（「now_tree="$(git write-tree)"」（`scripts/gate/gate-staged.sh:38`））。中间被改走又改回原样，这一比看不出来：

| 交错（模型键） | 乙会话在 write-tree 之后做的 | 在收尾比树之前做的 | 被判的树 | 挂上的树 |
|---|---|---|---|---|
| h9_aba | `git rm --cached` 含 L 的新页 | `git add` 回来 | 没有 L | 有 L |
| h9_stash | `git stash push` | `git stash pop --index` | 没有 L | 有 L |
| h9_commit_undo | `git commit`（暂存区清空，取 diff 时是空的），取完 diff、建 worktree 之前 `git reset --soft HEAD~1` | — | 旧 HEAD，没有 L | 有 L |

之后 staged-green 的树里有 L，丙的窗口（那棵树到这次的树）永远不含它（长度 4 内 182/182）。

另一形不要交错：L 绕过门禁提交之后，有人带 `GATE_BASE=HEAD` 跑 gate-staged（gate.sh 第 102 行先认人给的 GATE_BASE；「本次无对象可判」那句出路就教人这么写，见 `.claude/singlefs-ai-sop/scripts/gate.sh:468`），窗口收窄、全绿；gate.sh 自己不前移 gate-ok（「# GATE_BASE 被显式指定时不记（--staged 的里层也走这一支）：窗口是人为收窄的，拿它当起点会把中间的提交漏掉；」（`.claude/singlefs-ai-sop/scripts/gate.sh:769`）），gate-staged.sh 却没有同一条守卫（「git update-ref refs/sop/staged-green "$green_commit"」（`scripts/gate/gate-staged.sh:46`）只看退出码与树），staged-green 挂上含 L 的树（bing-override）。

### 丁（门禁绿的标记挂在提交上）

- ding_a（门禁在开跑的 HEAD 上挂记号）：与 yi_gate_mb 同一形打穿（ding-a-stamp-S）。
- ding_rw（另开 `notes.rewriteRef`，免得 rebase / amend 之后记号丢光）：amend 把 L 加进挂了记号的提交，记号跟到新提交上（ding-rw-amend）。不开它则 amend、rebase 之后记号丢，窗口只会变宽。
- ding_tree（提交那一刻树等于 staged-green 的树才挂）：继承丙的两种病，丙那几形之后紧跟一次原样提交（k），记号挂上含 L 的提交，此后 85/85 判不到（ding-tree-race、ding-tree-override）。没有 k 时被下一次门禁接住（一览里的「some」）。

### 戊（全量判 + 只缩不涨的存量豁免表）

- 键取（路径、行文）集合（wu_set、wu_fixed）：存量里一行豁免着的错引文，在同一页再抄一遍（从旧段落复制进新一节），新那行与旧那行同键，被豁免，一次都不判（wu-set-duplicate、wu-fixed-duplicate）。
- 只缩不涨比 HEAD（wu_set、wu_count）：「只缩不涨」本身要一个比对基准，比 HEAD 就把窗口问题搬回来——绕过门禁的提交同时加 L 与它的豁免行，此后表与 HEAD 一样，L 永远豁免（wu-set-grow、wu-count-grow）。比引入时的固定基线（wu_fixed）接得住这一形。
- 计数键 + 固定基线（wu_cf，这条腿加的，零轮）两形都接住（wu-cf-duplicate-caught、wu-cf-grow-caught），19 段历史长度 3 内没漏（`out/grid-L3.txt`）。它的代价见 H4：开工时要把 455 行（照窗口判法）或 101 行（照全量判法）逐行登进表；且「只缩不涨」与产物重跑冲突——产物换一轮，正文早先抄的行在全量判下变红，表又不许涨，只能当场回改正文（射程 4 原本把这些当历史不判：「#   4. 只判这次改动新增或改写的行：正文里早先抄下的行是历史，那时的产物按归档规则删掉了，再判就是判一个不存在的对照；」（`.claude/gate.d/doc-experiments.sh:1601`））。这一条是推的，没量重跑一轮会让多少行变红。

### 打中之后的四句（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」）

| 打中的形 | 分不分辨臂 | 被判的系统当时看不看得到判别它的东西 | 满足的分句 | C439 改法一列的两个改法在这几格上 |
|---|---|---|---|---|
| 绕过门禁 + 再一次提交 | 分：打今天、甲、乙的会话写法（要重开一轮）；不打 yi_gate_mb、丙、丁、戊 | 看得到：已提交的行就在基准与 HEAD 之间 | 条款 2「那一行一次都没进 `gate_added_lines` 的输出」 | 改成上一次提交（甲）：照中；前移时记起点：门禁写的接住，会话写的照中 |
| 记号挂在 HEAD 上、HEAD 里有被判的树删掉的行 | 分：打 yi_gate_mb、ding_a、ding_rw（与今天的 gate-ok）；丙、ding_tree、庚二不中 | 看得到：开跑时 HEAD 与被判的树都在手上，`git diff HEAD <被判的树>` 就列出被删的行 | 条款 2；改掉 L 的是别的会话未提交的编辑时同时是条款 3 | 两个改法都照中（甲本来就漏，门禁写的就是这一格） |
| gate-staged 当中另一会话改走又改回暂存区 | 分：打丙、ding_tree；gate-ok 那一族不中（记号挂的是开跑时的 HEAD，L 没提交） | 看得到：gate-staged 手里有开跑时那棵树，gate.sh 里有它实际判的那棵，两棵一比就知道 | 条款 3「把起点挪到另一个会话还没判过的改动之后」 | 甲：L 还在暂存区，下一次照样判到，不中；门禁写的不中 |
| 人给 GATE_BASE 跑 gate-staged | 分：打丙、ding_tree、庚；gate-ok 那一族有守卫不中 | 看得到：GATE_BASE 就在环境里 | 条款 2 | 甲、门禁写的都不中 |
| 豁免键撞上 / 表在绕过门禁的提交里涨了 | 只分戊的几个变体 | 看得到：同键行数、表与固定基线的差 | 条款 2（戊没有窗口，按「一次都没进判定」读） | 不适用 |

## H3：多会话（一个 HEAD、一个暂存区、一套 `refs/sop/`）

| 候选 | 起点由谁定 | 会被别的会话的写挪动吗 | 交错下漏判（量过的形） | 别的会话判绿过的行再进窗口（h14，长度 4，跑过门禁的 310 条续作） |
|---|---|---|---|---|
| 今天 | 不带 `--staged` 全绿的任何会话（gate-ok）；HEAD | 会：任何会话的提交挪 HEAD~1，任何会话的整轮全绿挪 gate-ok | h4_other：乙的 L 再隔一次提交就出窗口 | 155/310 |
| 甲 | 任何会话的提交 | 会 | h4_other | 0/310（窗口里只有未提交的） |
| 乙 会话写、一条 ref | 最后开轮的会话 | 会 | h13_shared_open | 长度 4 没扫；长度 3（字母表含 G）146/219 |
| 乙 会话写、每会话一条 | 本会话 | 不会被写，但别的会话的提交照进本会话的窗口；本会话重开一轮就越过别人没判的提交 | h4_other + o | 长度 4 没扫；长度 3 146/219 |
| 乙 门禁写（yi_gate_mb） | 任何会话的全绿 | 会；不带 `--staged` 那条路判整个工作区，别的会话没收尾的编辑也算进「被判的树」，于是 stamp 那一形可以由别的会话的编辑触发 | h6_stamp_N | 310/310 |
| 丙 | 任何会话的 gate-staged 全绿 | 会；交错改暂存区能让挂上的树含没判过的行 | h9_aba、h9_stash、h9_commit_undo | 0/310 |
| 丁 ding_tree | 同丙，加提交那一刻 | 同丙 | 同丙 + k | 0/310 |
| 庚二（零轮） | 同丙 | 同丙；交错被「判的就是挂的那棵」挡住 | 没漏 | 0/310 |

「提交时整个暂存区一起提交、不分是谁改的」对各候选：今天、甲、乙、丁的窗口按提交算，别的会话暂存着、被一起提交的行，归谁都一样只看它在不在基准与 HEAD 之间；丙、ding_tree、庚只看树的内容，谁提交、分几次提交都不影响窗口。代价一面：丙与庚下，暂存区里别的会话的一行红会让每个会话的 gate-staged 都红、staged-green 谁都挪不动——这正是促使人去写 `GATE_BASE=HEAD` 收窄的那一步（h15_override），所以庚二那条守卫不是可选的（推的：没量多会话下红挡住前移的频率）。

## H4：代价

### 本仓此刻的窗口宽度与存量红（量过，`out/real.txt`）

在 f111c3f3 的 `--shared` 克隆上（工作区干净，`dirty=0`），冻结的 `gate_added_lines <基准> .claude/kb ':(glob)research/**/*.md'` 取窗口，再照 quoted-result-lines 的射程过滤（`*-history.md`、`decisions-history/`、`research/prompts/`、`research/results/` 不算，行去空白后以 `E7RESULT ` 开头）。「窗口判法」= 只比 replay.sh 登记的产物（树里的，或已出树的最后一版），照「compared_lines = registered_lines」（`.claude/gate.d/doc-experiments.sh:1765`）重写；「全量判法」= 树里全部产物加全部归档。产物行去首尾空白（「text = line.strip().replace('\r', '')」（`.claude/gate.d/doc-experiments.sh:1718`））。历史节「只判新增的行」没照抄（窗口里的行一律算判），所以这里的找不到行数可能比格多几行。

```
name=real_window label=today_N base=HEAD added_lines_total=0 quoted_in_scope=0 files=0 not_found_in_any_out=0
name=real_window label=today_S base=HEAD~1 added_lines_total=1182 quoted_in_scope=0 files=0 not_found_in_any_out=0
name=real_window label=bing base=e2ae1837615a added_lines_total=786974 quoted_in_scope=1326 files=25 not_found_in_any_out=279
name=real_window label=full base=4b825dc642cb added_lines_total=821875 quoted_in_scope=1794 files=46 not_found_in_any_out=576
name=real_cell_window mode=window label=bing quoted_in_scope=1326 not_found=355 files_with_not_found=14
name=real_cell_window mode=window label=full quoted_in_scope=1794 not_found=455 files_with_not_found=21
name=real_cell_window mode=full label=bing quoted_in_scope=1326 not_found=101 files_with_not_found=2
name=real_cell_window mode=full label=full quoted_in_scope=1794 not_found=101 files_with_not_found=2
```

读法：
- 丙此刻（staged-green 的树 9b1d9d06，2026-09-22）让这一格从 0 行变成 1326 行要判；照窗口判法 355 行红，其中 101 行在任何产物里都找不到（真错），其余 254 行在归档或没登记的产物里找得到（产物换了登记或重跑过，正文没跟）。丙一上来就是这 355 行红，过不了就挪不动 staged-green；要么先修，要么在引入时把 staged-green 重挂到此刻的暂存树上（那就等于放过这 355 行，其中 101 行是真错）。
- 戊全量 1794 行，照窗口判法 455 行红（21 份页）、照全量判法 101 行；戊的豁免表开工就是这几百行。
- 乙（门禁写）与丁在记号不存在时退空树，等于戊不带豁免表；在引入时把记号挂到 HEAD 则 0 行，那 101 行永远不判。
- today_S（HEAD~1）此刻窗口里 0 行：上一次提交（f0b94fef → f111c3f3）没碰射程内的 E7RESULT 行。

### 要改哪些文件（推的，没实现）

| 候选 | 本仓 | 上游 | 动不动用户 2026-09-24 定案（共用脚本不认 `GATE_DIFF_BASE`） | C528 现状说明 |
|---|---|---|---|---|
| 甲 | `scripts/gate/changed-paths.sh` 的 gate 取法改 HEAD | `lib.sh` 的 `diff_base` 或 `gate.sh` 第 102 行不再设上游基准 | 不动 | 两条路窗口一样了，`scripts/gate/changed-paths.sh` 第 12–13 行那句要改 |
| 乙 门禁写 | `changed-paths.sh` 读 `refs/sop/gate-ok` | `gate.sh` 在 `--staged` 全绿时也前移（今天第 119 行起写明不前移） | 不认 GATE_DIFF_BASE 也做得到（直接读 ref），字面不动 | 同上 |
| 丙 / 庚二 | `gate-staged.sh` 开跑前把 `GATE_BASE` 设成 staged-green 那个提交（gate.sh 第 102 行先认 GATE_BASE），人已给了 GATE_BASE 时不前移；`changed-paths.sh` 没设 GATE_BASE 时读 staged-green | 庚：要让里层判的正是开跑时 write-tree 的那棵，`gate.sh --staged` 得能收一棵树（今天第 56、59、87 行自己从暂存区重套），这是上游改动 | 不动 | 两条路窗口一样了，同上 |
| 丁 | 提交后挂 notes 的钩子或包装 | 无 | 不动 | 同上 |
| 戊 | quoted-result-lines 格改全量、加豁免表与只缩不涨的固定基线 | 无 | 不动 | 不涉及（这一格不再按窗口判；同一基准的别的格照旧） |

与 `scripts/gate/stage-must-run.sh` 的复用判据：丙、庚二用的就是它那条 `refs/sop/staged-green`，是同一个起点。反过来，丙那三种交错今天就会让 staged-green 挂上一棵没被判过的树，而 stage-must-run 拿这棵树判「重阶段输入没变、可跳过」——这是同一个病根落在复用判据上，C439 之外的一格（推的，没在模型里造复用那一侧）。

## 自己提的（被攻过零轮，只在我的模型上量过）

- 庚（geng）：丙，再让被判的树就是开跑时 write-tree 的那棵（模型里 `read-tree --reset -u <那棵树>` 代替重套 diff）。
- 庚二（geng2）：庚，再加一条与 `.claude/singlefs-ai-sop/scripts/gate.sh:774` 同义的守卫——GATE_BASE 是人给的就不前移 staged-green。
- wu_cf：戊，豁免按（路径、行文）计数，只缩不涨比引入时的固定基线。

| 改法 | 修哪一格 | 在打中的格上 |
|---|---|---|
| 庚 | 丙的三种交错 | 量过：h9_aba、h9_stash、h9_commit_undo 长度 4 内 0 漏（geng-race-caught） |
| 庚 | 丙的 GATE_BASE 那一形 | 量过：照中（geng-override，judged_ever=no）——庚单独不够 |
| 庚二 | 交错 + GATE_BASE | 量过：19 段历史，长度 3、字母表含 G（`out/grid-L3.txt` 的结果里，含 N 或 S 的 3446 条续作 0 条漏）与长度 4、字母表 NSck（`out/grid-L4.txt`，0 漏）；漏的只剩「之后每一次门禁都带 GATE_BASE」的续作 |
| 庚二 | stamp 那一形（yi_gate_mb 打中的） | 量过：没漏（geng2-stamp-caught）；树对树天生不中这一形 |
| 庚二 | 存量 | 推的：引入时的 355 行红照旧，见 H4 |
| wu_cf | 戊的键撞上、表涨了 | 量过：两形都接住；19 段历史长度 3 内没漏 |
| wu_cf | 产物重跑后正文变红而表不许涨 | 推的：没量 |

## 没打中的形状

| 候选 | 试过的形 | 取样 | 结果 |
|---|---|---|---|
| 丙 bing | 绕过门禁、分几次提交、别的会话提交、推过 / 没推、amend、revert、stamp 三形、豁免表两形、会话写起点的两形、重判代价 | 长度 4、字母表 NSck(+p)，每段 114～268 条守规矩的续作 | 0 漏；只在交错与 GATE_BASE 两族上漏 |
| yi_gate_mb | amend（merge-base 接住）、三种交错（L 没提交，记号挂的 HEAD 里没有 L）、GATE_BASE（gate.sh 有守卫）及上面各形 | 同上 | 只在 stamp 三形上漏 |
| ding_tree | stamp 三形、amend | 同上 | 0 漏（树对树） |
| wu_cf | 全部 19 段 | 长度 3、字母表 NSckoG(+p) | 0 漏 |
| 庚二 | 全部 19 段 | 长度 3（含 G）、长度 4（NSck） | 见上 |
| 全部窗口候选 | 页从排除的位置（`*-history.md`、`research/prompts/`）搬进射程 | 没在模型里造 | 推的：`gate_added_lines` 写死 `--no-renames`（`scripts/gate/changed-paths.sh` 第 125–127 行），搬家后整份算新增，判得到 |
| 丙 | 跑的当中另一会话 `git commit -- <路径>`（只提交那几条路径） | 没在模型里造 | 推的：提交会改那几条路径的索引项，收尾比树时不同，不前移 |
| 丙 | 两个 gate-staged 同时跑、互相覆盖 staged-green | 没在模型里造 | 推的：各自挂的都是自己判过的树（没交错时），覆盖只让窗口变宽 |

不在这一轮射程里、没攻的：产物重跑或换登记让正文早先抄的行变错（射程 4 有意不判）；历史节「只判新增的行」在丙的树对树窗口下的行号对法。

## 这条腿自己的限度

- 判红判绿只有 quoted-result-lines 一格的简化版，产物取 `research/results/*.out` 全部；同一基准的别的格（doc-process-records 五格、doc-text 等）没逐格建模。它们共用 `gate_diff_base gate`，窗口一样，判法不同。
- gate.sh、gate-staged.sh 没跑，是按冻结行号重写的流程；重写与原脚本不一致的地方会让结论跟着错（推翻条件：主 agent 在临时仓里真跑 gate-staged.sh 复现 bing-race-unstage 那一段，staged-green 不挂上含 L 的树）。
- 「永久」是扫描上界之内的永久（长度 3 全部候选，长度 4 五个候选），之外是推的：基准只往后挪、L 已在基准里，就不再出现在 diff 里。
- 会话只是模型里的标签，没有真的两个进程；交错是在固定的两处插进去的，不是随机调度。
- 本仓的数用的是重写的比对法（`measure-real-cell.py`），历史节「只判新增」没照抄；101 行那一组用的是最宽的比法（全部产物加全部归档），不受登记表影响。
- 早先在模型的旧版本上跑过一批探路扫描（提交时刻没写死、候选与历史较少），不当证据引；报告里的数都来自 `out/` 下这一版的产物。
- 每批的段数与挂钟：长度 3 每段历史 4144 次（有上游的 6400 次），长度 4 每段 1705 次（有上游的 3905 次），31 并行，每段 40～110 秒（别的会话的 cargo 让负载在 60 上下）；正式跑之前先跑了 273 次的一小段（2 秒）估时。

## 没做什么

- 没起任何门禁阶段、gate.sh、gate-staged.sh，没跑 cargo、重型测试。
- 没在本仓克隆上跑 quoted-result-lines 那一格本身；没量戊在产物重跑一轮后会多出多少红。
- 没建模 stage-must-run.sh 的复用那一侧（交错让它复用一棵没判过的树，是推的）。
- 不判正推腿那几格，不替主 agent 采纳；副本与模型上的数不算入库装置上的数。
- 要主 agent 跑的命令：没有必须的；要复核就照「复跑」一节那一条。

## 引文核对

`python3 scripts/agents/cite-check.py research/prompts/c439-r1-opus-output.md --root /home/fy5090/code/singlefs` 末行：

```
  ✓ 核了 14 处引文、6 处裸行号，对上 14 处，没判 6 处（其中裸行号没有引文可比 6 处；逐处列在上面）
```

草稿目录 `/home/fy5090/.local/share/singlefs/work/c439-r1-opus/` 里的临时 git 仓与本仓克隆已删；留着的是各批扫描的 `results.txt`、合表与日志（约 38M）。
