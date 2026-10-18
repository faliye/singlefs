# C439（gate-ok 前移让同轮早先写的行不再被判）第一轮：云端正推腿（Sonnet）报告（2026-10-14）

<!-- doc-lint:not-numbers H1 H2 H3 H4 H5 -->

立场：从冻结副本 `/home/fy5090/.local/share/singlefs/work/c439-r1/frozen/` 的脚本与 git 语义推每个候选的基准与判定，再用临时 git 仓照脚本规则模拟量一遍。冻结副本的 10 份与仓里此刻的 10 份逐字相同（`cmp` 逐份核过，全 same），所以引文用仓内路径，行号就是冻结副本的行号。
没读禁读清单里的任何东西。模型在 `research/prompts/c439-r1-sonnet-model/`（`lib.sh`、`histories.sh`、`measure-h4.sh`、`estimate-red.py`、`tools/rerun.sh`、`out/`）。

## 〇、结论先行（推的部分标明）

- 今天的取法在 14 段历史（13 段带错行、1 段对照）里让错行永久落在窗口外的有 9 段（H1 表）；C439 原形（`h1a_original`）在今天的脚本上复现：同一份历史去掉「整轮全绿让 gate-ok 前移」那一步（`h1a_nomove`）就被抓到。
- 甲（取 HEAD）13 段全漏，比今天还窄；乙（起点记 ref）被另一会话挪动即漏（`m1_roundmove`）、改写历史也漏；庚（给 `gate_diff_base gate` 补一档 gate-ok）7 段漏；丁（标记挂提交）只在改写历史时漏 1 段。这四个我判站不住（丁带条件，见 H2）。
- 丙（基准取 `refs/sop/staged-green` 那棵树）两条路都改时 14 段全进窗口；只改带 `--staged` 一条路时漏 1 段（`h1f_n_only`，只跑不带 `--staged` 的整轮）。戊（全量判加存量豁免表）14 段全进窗口。丙、戊都站得住，戊的代价高（H4），按跑前条款「两个以上候选站得住」要交用户。
- 丙的两种改法里，「两条路都改」改变 C528（两条门禁路判的改动窗口不同）记的窗口关系，按跑前条款要交用户；「只改带 `--staged` 一条路」不碰用户 2026-09-24 的定案，但 `scripts/gate/changed-paths.sh` 文件头第 12–13 行和 C528 那一行要改写（H4）。
- 丙此刻的头一道门槛：`refs/sop/staged-green` 停在 2026-09-22，从它到现在 quoted-result-lines 格会多判 1326 行（去重 1283 条），照格的口径粗估其中至多 351 条找不到对照（H4）；头一趟带 `--staged` 的整轮要先清掉或登记这批，否则 staged-green 永远不前移、窗口永远不缩。

## 一、实现今天的样子（正推腿现查，与背景材料相互独立）

- `crates/` 里没有这件事的代码。命令与原样输出：

```
$ grep -rn 'gate-ok\|diff_base\|staged-green' crates/ | wc -l
0
```

- 方案按 `scripts/`、`.claude/` 今天的实现来谈；下面「改哪个文件」都是对这些文件的改动。
- 此刻本仓的状态（只读现查，`GIT_OPTIONAL_LOCKS=0`）：

```
HEAD=c1e9b324 HEAD~1=ceac9b0f
N 带不带 --staged=不带：gate_diff_base gate=c1e9b324
S 带 --staged：diff_base=ceac9b0f
upstream=origin/master  origin/master=c1e9b324  gate-ok=不存在  staged-green=e2ae1837
```

  这与背景材料「本仓没有 `@{upstream}`」不同：我现查时 `branch.master.remote` 已设、`origin/master` 与 HEAD 同一个提交，即本仓此刻正处在 H1「全推出去之后」的形状（不带 `--staged` 取 HEAD，带 `--staged` 取 HEAD~1）。HEAD 在我开工后从 `f111c3f3` 挪到了 `c1e9b324`（别的会话提交了）；下面 H4 的量钉在 `f111c3f3` 上。

## 二、两条路的基准怎么算（引冻结副本）

- 不带 `--staged`（以下记 N）：各阶段 source 共用库，`gate_diff_base gate`「GATE_BASE 是个提交就用它；否则 @{upstream} 的 merge-base；都没有（或 merge-base 取不到）就 HEAD。」（`scripts/gate/changed-paths.sh:10`）。没有上游就是 HEAD，窗口只含工作区、暂存区、未跟踪：「没有上游、也没设 GATE_BASE 时基准是 HEAD，窗口只含工作区与暂存区：已提交没推的那几次不在里面。」（`.claude/gate.d/doc-experiments.sh:110`）。
- 带 `--staged`（以下记 S）：外层在源仓上算 `diff_base`，设成里层的 `GATE_BASE`：「staged_diff_base="${GATE_BASE:-$(diff_base "$source_repository_root")}"」（`.claude/singlefs-ai-sop/scripts/gate.sh:102`），「if GATE_BASE="$staged_diff_base" GATE_STAGED_FROM="$source_repository_root" bash "$staged_gate"」（`.claude/singlefs-ai-sop/scripts/gate.sh:115`）；里层 `gate_diff_base gate` 第一档就认它。
- `diff_base` 在默认分支上、merge-base 等于 HEAD 时（「if [[ "$merge_base" == "$(git -C "$root" rev-parse HEAD 2>/dev/null)" ]]; then」（`.claude/singlefs-ai-sop/scripts/lib.sh:165`））：先试与上游的 merge-base（不等于 HEAD 才用），再试 gate-ok：「&& [[ -n "$gate_ok_commit" && "$gate_ok_commit" != "$(git -C "$root" rev-parse HEAD)" ]] \」（`.claude/singlefs-ai-sop/scripts/lib.sh:176`），最后「if parent_of_head="$(git -C "$root" rev-parse -q --verify HEAD~1 2>/dev/null)"; then」（`.claude/singlefs-ai-sop/scripts/lib.sh:180`）。**gate-ok 等于 HEAD 时被跳过、退到 HEAD~1**，所以 C439 原形里「前移之后马上再跑」今天已被这一条部分兜住（`h1a_immediate`：今天抓得到）；要中间再隔一个没验的提交才漏（`h1a_original`）。
- gate-ok 只在 N 一路写：「elif [[ -n "$START_HEAD" && -z "${GATE_BASE:-}" ]]; then」（`.claude/singlefs-ai-sop/scripts/gate.sh:774`）、「if ! git -C "$ROOT" update-ref refs/sop/gate-ok "$START_HEAD" 2>/dev/null; then」（`.claude/singlefs-ai-sop/scripts/gate.sh:781`）；记的是开跑时的 HEAD，而 N 的窗口只含未提交的改动。**这就是 C439 的根：gate-ok 盖章的是「HEAD 这个提交」，N 那一趟判的却只有 HEAD 之外的未提交改动。**
- staged-green 只在 S 一路写、整轮全绿且暂存树没变才前移：「if [[ $gate_rc -eq 0 ]]; then」（`scripts/gate/gate-staged.sh:36`）、「if [[ "$now_tree" != "$staged_tree" ]]; then」（`scripts/gate/gate-staged.sh:39`）、「git update-ref refs/sop/staged-green "$green_commit"」（`scripts/gate/gate-staged.sh:46`）；它记的是树，不是提交，且与六道重阶段复用共用：「判据只有一句：这一道读的那几条路径，在 refs/sop/staged-green 那棵树与这一次的暂存树之间，逐文件的源码摘要变没变。」（`scripts/gate/stage-must-run.sh:16`）。
- 取新增行：基准到工作区的 diff，加未跟踪文件整份：「diff_output="$(git "${quote_option[@]}" diff --no-renames --unified=0 --no-color --no-ext-diff --src-prefix=a/ --dst-prefix=b/ "$base" -- "$@")"」（`scripts/gate/changed-paths.sh:127`）；`git diff <提交>` 对任意提交都行，不要求它是 HEAD 的祖先。
- 用 `gate_added_lines` 的三处：quoted-result-lines 所在的六格共享一次取：「if gate_added_lines "$shared_base" .claude/kb ':(glob)research/**/*.md' > "$SHARED_ADDED_LINES"; then added_lines_ok=1; fi」（`.claude/gate.d/doc-experiments.sh:139`）；doc-process-records 的同步记录：「if added_record_lines="$(gate_added_lines "$gate_base" 'research/prompts/*-sync.md')"; then」（`.claude/gate.d/doc-process-records.sh:368`，行尾后半截省了）；doc-decisions：「added_out="$(gate_added_lines "$BASE" "$f")" || {」（`.claude/gate.d/doc-decisions.sh:1258`）。

## 三、H1　漏判的形（今天的取法）

模拟规则（照上面的引文逐条）：N 的窗口＝`gate_diff_base gate`（无上游取 HEAD，有上游取与上游的 merge-base）到工作区加未跟踪；S 的窗口＝`diff_base` 到暂存树；N 全绿写 gate-ok＝开跑时的 HEAD；S 全绿且暂存树没变写 staged-green＝包着暂存树的无父提交。窗口里出现 tag 含 `bad` 的 E7RESULT 行即这一趟红（等于 quoted-result-lines 报出错引文）。造仓、走步骤的脚本是 `research/prompts/c439-r1-sonnet-model/histories.sh`，原样逐趟输出在 `research/prompts/c439-r1-sonnet-model/out/h1h3-details.txt`（464 行）。下表只抄「今天」那一组的基准（提交用说明里的名字标，哈希每趟不同）。

| 历史（函数名） | git 历史的形状 | 今天 N 的基准 | 今天 S 的基准 | 错行 Lbad_A 在窗口里吗 |
|---|---|---|---|---|
| C439 原形（`h1a_original`） | c0 两路各绿一次（gate-ok＝c0）→ c1 写 Lbad_A，绕过门禁直接提交 → 工作区另有未提交的好行，N 整轮绿（gate-ok→c1）→ c2 绕过提交 → 提交前跑 S | HEAD＝c1（窗口只有那行未提交的好行） | c1（gate-ok；c1 里的 Lbad_A 在基准里，不算新增） | **不在，永久**（后面的 N 基准 HEAD＝c3，更窄） |
| 原形的紧接版（`h1a_immediate`） | 同上，但 N 绿完立刻带暂存再跑，中间没有 c2 | HEAD＝c1 | c0（gate-ok＝HEAD 被第二节引的那条 `!= HEAD` 判断跳过，退 HEAD~1） | 在（S 步 4 红） |
| 原形去掉 N 那一趟（`h1a_nomove`） | 同 `h1a_original`，没有「整轮绿让 gate-ok 前移」 | HEAD | c0（gate-ok 还在 c0） | 在（S 步 3 红）——两段历史只差那一趟 N，就是 C439 要说的 |
| 绕过门禁，两个提交（`h1b_bypass`） | 从没有 gate-ok；c1 写 Lbad_A、c2 写好行，都绕过；再跑 S | HEAD | c1（HEAD~1） | **不在，永久** |
| 绕过门禁，一个提交（`h1b_bypass_one`） | 只有 c1 | HEAD | c0（HEAD~1） | 在（S 步 2 红） |
| 一次暂存分几次提交（`h1c_split`） | 第一份暂存、S 绿、提交 c1；第二份（带 Lbad_A）c2、第三份 c3、第四份 c4 直接提交不跑；下一批跑 S | — | c3（HEAD~1） | **不在，永久** |
| 全部暂存好再分次提交（`h1c_allstaged`，对照） | 三份都暂存、S 跑一趟、再分两次提交；目标行是其中一行好行 | — | c0 | 在（S 步 2）——一趟里全判到，分几次提交本身不漏 |
| 另一个会话在两次门禁之间提交（`h1d_interleave`） | A 暂存好行、S 绿；B 暂存 Lbad_A 并提交（带走 A 已暂存的）；A 再暂存再提交一次不重跑；A 下一批跑 S | — | cB（HEAD~1） | **不在，永久** |
| 全推出去，一个绕过提交（`h1e_pushed_one`） | c1 写 Lbad_A，上游指到 HEAD；N、S 各跑一趟 | HEAD＝c1（上游 merge-base＝HEAD） | c0（HEAD~1） | N 不在、S 在（红）——这就是 C528 记的两条路窗口不同 |
| 全推出去，两个绕过提交（`h1e_pushed_two`） | c1 写 Lbad_A、c2 写好行，上游指到 HEAD | HEAD＝c2 | c1 | **两路都不在** |
| 只跑不带 `--staged` 的整轮（`h1f_n_only`） | c0 两路各绿；c1 写 Lbad_A、c2 写好行，都绕过；只跑 N | HEAD＝c2 | （没跑） | **不在，永久**（「跑全量工作区的跑 bash .claude/scripts/gate.sh 不带参数」（`.claude/skills/gate/staged-gate.md:16`）许这一路，是真有人走的） |

「永久」的判法：这一行之后每一趟的基准都不早于写下它的提交（窗口只会随新提交越来越窄），而 gate-ok 与 staged-green 的前移只在整轮全绿时发生，没有任何一趟再把它纳入窗口。账本里这一行一次都没进过窗口（`ever_in_window` 的 `no`）。

- **今天带与不带 `--staged` 两条路算出的基准不同的形**：`h1e_pushed_one` 一段（上面「N 不在、S 在」）；本仓此刻就在这个形状里（第一节的输出：N＝c1e9b324＝HEAD，S＝ceac9b0f＝HEAD~1）。
- 与背景材料对账：背景写「`e2ae1837` 的提交时刻之后 HEAD 上 131 个提交，带 `no-verify` 的 70 个，其中 27 个写着『提交时绕过门禁』」。我现查（HEAD 已到 c1e9b324）的原样输出：

```
$ git log --since="$(git log -1 --format=%cI e2ae1837)" --format='%H' | wc -l
143
$ git log --since="$(git log -1 --format=%cI e2ae1837)" --format='%B' | grep -c '提交时绕过门禁（--no-verify），没跑 gate-staged 与 checker 档'
39
$ git log --since="$(git log -1 --format=%cI e2ae1837)" --format='%B%x00' | tr '\n' ' ' | tr '\0' '\n' | grep -c 'no-verify'
82
```

  数不同（我的 143/39/82 对背景的 131/27/70），原因多半是 HEAD 往前走了与时刻的写法，我没追；量级相同：绕过门禁的提交在本仓是常态，所以「绕过门禁，两个提交」不是边角形。
- 什么现象会推翻 H1：造得出一段「一行错引文被提交后，今天的 N 或 S 在后续某趟窗口里含它、却不是因为 gate-ok 或 HEAD~1 的巧合」的历史，则上表「不在，永久」的那几段不成立；`h1a_nomove` 与 `h1a_original` 若在你的临时仓里判得一样，则 gate-ok 前移不是原因。

## 四、H2　候选逐个在 H1 每种历史上的判定

候选与我的实现方式（都写成对实现的改动，不改任何脚本，只在临时仓里模拟）：

| 候选 | 基准取什么 | 模拟时怎么写（`research/prompts/c439-r1-sonnet-model/lib.sh`） |
|---|---|---|
| 今天 | N：`gate_diff_base gate`；S：`diff_base` | `base_today_N`、`base_today_S` |
| 甲 | 带不带 `--staged` 都取 HEAD | `jia)` 一行 |
| 乙 | 开轮时记下 HEAD 到 `refs/sop/round-start`，两路都取它（不在 HEAD 的祖先链上就退回今天） | `yi)`；开轮由「一个会话」写（模拟里 `prelude` 写一次，`m1_roundmove` 里第二个会话再写一次覆盖）。谁写：门禁来写就是今天的 gate-ok；用户来写要多一步人工，三种写者都是同一条 ref，下面 H3 一并判 |
| 丙 | `refs/sop/staged-green` 的树与这次暂存树（S）或工作区（N）做树对树 diff。**分两版**：丙只改带暂存＝只有 S 取它，N 不动；丙两条路＝N、S 都取它。没有 staged-green 时退回今天 | `bing_s)`、`bing_sn)` |
| 丁 | 绿标记挂在提交上（模拟用 `git notes --ref=sop-green`）：S 绿之后，下一趟开头若 HEAD 的树恰好等于 staged-green 的树，就给 HEAD 挂标记；基准取 first-parent 链上最近一个带标记的提交，两路都取，没标记的提交一律在窗口里 | `ding)`、`ding_mark` |
| 戊 | 这几格不按窗口判，全量判，另扣一张存量豁免表（模拟里的 `EXEMPT_FILE`，一行一个 tag，空表） | `wu)`：基准取空树 |
| 庚（我加的） | 只改 N：给 `gate_diff_base gate` 的回退次序补一档 gate-ok（GATE_BASE→上游 merge-base→gate-ok→HEAD），S 不动。想法：让 N 判的窗口与它盖章的 gate-ok 一致 | `geng)` |

每段历史跑一遍每个候选，目标行是否进过任一趟的窗口（账本里 `ever_in_window`）。**原样输出**（`research/prompts/c439-r1-sonnet-model/out/h1h3-summary.txt`，14 段；`h1c_allstaged` 的目标行是好行，当对照；头 11 段是 H1，后 3 段 `m1`、`m3`、`m5` 是 H3 用的）：

```
== h1a_original（目标行 Lbad_A）
  今天     no
  甲        no
  乙        yes 步3 N
  丙只改带暂存 yes 步4 S
  丙两条路 yes 步3 N
  丁        yes 步3 N
  戊        yes 步3 N
  庚        yes 步3 N
== h1a_immediate（目标行 Lbad_A）
  今天     yes 步4 S
  甲        no
  乙        yes 步3 N
  丙只改带暂存 yes 步4 S
  丙两条路 yes 步3 N
  丁        yes 步3 N
  戊        yes 步3 N
  庚        yes 步3 N
== h1a_nomove（目标行 Lbad_A）
  今天     yes 步3 S
  甲        no
  乙        yes 步3 S
  丙只改带暂存 yes 步3 S
  丙两条路 yes 步3 S
  丁        yes 步3 S
  戊        yes 步3 S
  庚        yes 步3 S
== h1b_bypass（目标行 Lbad_A）
  今天     no
  甲        no
  乙        yes 步2 S
  丙只改带暂存 yes 步2 S
  丙两条路 yes 步2 S
  丁        yes 步2 S
  戊        yes 步2 S
  庚        no
== h1b_bypass_one（目标行 Lbad_A）
  今天     yes 步2 S
  甲        no
  乙        yes 步2 S
  丙只改带暂存 yes 步2 S
  丙两条路 yes 步2 S
  丁        yes 步2 S
  戊        yes 步2 S
  庚        yes 步2 S
== h1c_split（目标行 Lbad_A）
  今天     no
  甲        no
  乙        yes 步3 S
  丙只改带暂存 yes 步3 S
  丙两条路 yes 步3 S
  丁        yes 步3 S
  戊        yes 步3 S
  庚        no
== h1c_allstaged（目标行 Lok_p3）
  今天     yes 步2 S
  甲        yes 步2 S
  乙        yes 步2 S
  丙只改带暂存 yes 步2 S
  丙两条路 yes 步2 S
  丁        yes 步2 S
  戊        yes 步2 S
  庚        yes 步2 S
== h1d_interleave（目标行 Lbad_A）
  今天     no
  甲        no
  乙        yes 步3 S
  丙只改带暂存 yes 步3 S
  丙两条路 yes 步3 S
  丁        yes 步3 S
  戊        yes 步3 S
  庚        no
== h1e_pushed_one（目标行 Lbad_A）
  今天     yes 步3 S
  甲        no
  乙        yes 步2 N
  丙只改带暂存 yes 步3 S
  丙两条路 yes 步2 N
  丁        yes 步2 N
  戊        yes 步2 N
  庚        yes 步3 S
== h1e_pushed_two（目标行 Lbad_A）
  今天     no
  甲        no
  乙        yes 步2 N
  丙只改带暂存 yes 步3 S
  丙两条路 yes 步2 N
  丁        yes 步2 N
  戊        yes 步2 N
  庚        no
== h1f_n_only（目标行 Lbad_A）
  今天     no
  甲        no
  乙        yes 步3 N
  丙只改带暂存 no
  丙两条路 yes 步3 N
  丁        yes 步3 N
  戊        yes 步3 N
  庚        yes 步3 N
== m1_roundmove（目标行 Lbad_A）
  今天     no
  甲        no
  乙        no
  丙只改带暂存 yes 步2 S
  丙两条路 yes 步2 S
  丁        yes 步2 S
  戊        yes 步2 S
  庚        no
== m3_toctou（目标行 Lbad_A）
  今天     no
  甲        no
  乙        yes 步3 S
  丙只改带暂存 yes 步3 S
  丙两条路 yes 步3 S
  丁        yes 步3 S
  戊        yes 步3 S
  庚        no
== m5_rewrite（目标行 Lbad_A）
  今天     no
  甲        no
  乙        no
  丙只改带暂存 yes 步2 S
  丙两条路 yes 步2 S
  丁        no
  戊        yes 步2 S
  庚        no
```

按候选读上面那张表（漏＝目标行永久不进窗口；14 段里 `h1c_allstaged` 是好行对照，带错行的是 13 段，下面的「漏几段」都是 13 取分母）：

| 候选 | 漏几段 | 漏在哪几段 | 判 |
|---|---|---|---|
| 今天 | 9 | `h1a_original`、`h1b_bypass`、`h1c_split`、`h1d_interleave`、`h1e_pushed_two`、`h1f_n_only`、`m1_roundmove`、`m3_toctou`、`m5_rewrite` | 漏（参照） |
| 甲 | 13 | 全部；比今天还窄：`h1a_immediate`、`h1b_bypass_one`、`h1e_pushed_one` 今天抓得到、甲抓不到 | **站不住** |
| 乙 | 2 | `m1_roundmove`（第二个会话开轮覆盖了起点）、`m5_rewrite`（起点提交被改写，不在 HEAD 祖先链上，退回今天） | **站不住** |
| 丙只改带暂存 | 1 | `h1f_n_only`（N 一路没改，只跑 N 的人仍然漏） | 站得住（有射程） |
| 丙两条路 | 0 | — | **站得住** |
| 丁 | 1 | `m5_rewrite`（标记挂在被改写掉的旧提交上，first-parent 链上找不到带标记的祖先，退回今天） | 站不住（带条件，见下） |
| 戊 | 0 | — | 站得住（代价见 H4，交用户） |
| 庚 | 7 | `h1b_bypass`、`h1c_split`、`h1d_interleave`、`h1e_pushed_two`、`m1_roundmove`、`m3_toctou`、`m5_rewrite`——只要从没有人跑过不带 `--staged` 的整轮（本仓常态），gate-ok 不存在，S 一路退回 HEAD~1 | **站不住** |

逐个的理由：

- **甲**：基准＝HEAD，窗口只含工作区、暂存区与未跟踪，已提交没判的行一律在窗口外（取新增行的 diff 是基准到工作区，见第二节）。S 一路的基准从 HEAD~1 收到 HEAD，等于把「上一个提交」也移出窗口。推翻它的现象：本仓不再有绕过门禁的提交（背景与我现查的数都不是这样）。
- **乙**：起点要有人写，三种写者各有一处断：门禁来写＝今天的 gate-ok（`h1a_original` 漏）；开轮的会话来写、用户来写＝同一条 ref，被第二个会话覆盖（`m1_roundmove`）。要避免覆盖只有每个会话各存一条，而冻结副本的 `gate.sh`、`lib.sh`、`gate-staged.sh`、`stage-must-run.sh` 里没有读会话号的地方（`grep -n -i 'session_id\|CLAUDE_SESSION\|SESSION'` 只命中一行注释：「这里在临时 worktree 上把 git diff --cached 套到 HEAD 上，再对那棵树跑一遍整道门禁。」（`.claude/singlefs-ai-sop/scripts/gate.sh:46`）），门禁认不出「这是哪个会话的轮」。跑前条款第三行「某候选在两个会话交错时把起点挪到另一个会话还没判过的改动之后 ⇒ 出局」在 `m1_roundmove` 上触发，可复跑的临时仓脚本是 `research/prompts/c439-r1-sonnet-model/histories.sh m1_roundmove`。
- **丙**：基准是「最后一次整轮全绿的那棵树」。每一次窗口都是「不在最近一次全绿树里的行」，而全绿树里的行都是上一趟（或更早）判过的，所以一行错引文要永久漏，必须它进了某一棵被判绿的树却没被判到——模拟的 14 段里没有这样的形（推的：归纳依据见 H3 的最后写者赢一段）。树对树不依赖提交号，`m5_rewrite` 里 amend 之后提交号全换仍抓到。S 一路在里层用 `GATE_BASE`，`diff_base` 把它原样交出（「printf '%s' "$GATE_BASE"; return 0」（`.claude/singlefs-ai-sop/scripts/lib.sh:155`）），所以不动上游文件也能做到；N 一路要改共用库。
- **丁**：和丙同样「只信判过的树」，但信息挂在提交上，会因提交号变化丢失（`m5_rewrite` 漏）；且标记要在「S 绿→提交→下一趟开头」之间没有别的提交时才落得上，本仓绕过门禁的提交是常态（H1 末尾的数），落不上时窗口回弹到更早的标记，已判过的行重判（代价，没量频率）。如果主 agent 判定本仓不改写已提交的历史，丁与丙在漏判上等价、代价高于丙（多一族 ref `refs/notes/…`、merge 提交上 first-parent 歧义、需要有人写标记）。
- **戊**：基准是空树，什么历史都进窗口，所以 14 段全进。它换掉了「只判这次改动」这个设计本身，代价在 H4。
- **庚**：只补 N 一路的 gate-ok 档，能修 `h1a_original`、`h1f_n_only` 这类「N 盖章的 gate-ok 与 N 判的窗口不一致」，但 S 一路本来读 gate-ok、而 gate-ok 只有 N 全绿才会有，纯走 `gate-staged.sh` 的人没有这条 ref，窗口还是 HEAD~1。

什么现象会推翻上面这张表：任何一格的「漏」若在你的临时仓里被抓到，说明我的模拟规则（上面第二节）与 `gate.sh` 真跑的规则有出入；`out/h5.txt` 与 `out/h5-broken.txt` 是为这个造的（H5）。

## 五、H3　多会话：一个 HEAD、一个暂存区、一套 `refs/sop/`

| 候选 | 起点由谁定、写在哪 | 被别的会话的写挪动吗 | 交错时漏判吗 | 把别的会话的行算进这一次窗口吗 |
|---|---|---|---|---|
| 今天 | gate-ok：任一会话跑不带 `--staged` 的整轮并全绿，写开跑时的 HEAD（「if ! git -C "$ROOT" update-ref refs/sop/gate-ok "$START_HEAD" 2>/dev/null; then」（`.claude/singlefs-ai-sop/scripts/gate.sh:781`））；S 一路另取 HEAD~1 | 会：B 的 N 绿把起点推过 A 已提交没判的行（`h1a_original`），推的路径同 `m1_roundmove` | 漏（`h1d_interleave`、`m3_toctou`） | 窗口 gate-ok 到 HEAD 之间的提交不分谁写的都进 |
| 甲 | HEAD，每个提交都挪 | 会：任何会话提交即挪 | 漏（13 段全漏） | 不会（窗口只有未提交） |
| 乙 | 一条共用 ref，谁开轮谁写 | 会：`m1_roundmove`，B 开轮把起点推到 A 的未判提交之后 | 漏（`m1_roundmove`） | 轮内所有人的提交都进，不分谁写 |
| 丙 | `refs/sop/staged-green`：任一会话带 `--staged` 的整轮全绿且暂存树没变才写，最后写者赢 | 会，但只会挪到「刚被判绿的树」；挪动后的窗口仍含所有不在那棵树里的行 | 不漏（`m1_roundmove`、`m3_toctou`、`h1d_interleave` 全进） | **会**：`m2_lastwriter` 里 A 的 a1 在 A 的树上判绿过，B 把 a1 退出暂存后绿在不含 a1 的树上，A 再暂存 a1 时 a1 又进窗口（记代价，不出局） |
| 丁 | 标记写在提交上（notes），谁的树等于 staged-green 谁挂 | 不会挪已有标记；新标记只加不改 | 不漏（13 段里只 `m5_rewrite` 漏，与交错无关） | 会（标记落不上时窗口回弹） |
| 戊 | 没有起点；豁免表是仓内文件，改它走提交 | 不涉及 | 不漏 | 全量，不分 |
| 庚 | 同今天（gate-ok） | 同今天 | 漏 7 段 | 同今天 |

「提交时整个暂存区一起提交、不分是谁改的」对每个候选意味着什么（`m3_toctou`：A 跑绿之后 B 往同一个暂存区放了错行，A 的提交带走了它，之后又有一个没跑的提交）：

- 今天、甲、庚：B 的错行进了 A 的提交，之后的窗口都不含它，永远漏（`m3_toctou` 这三个都是 no）。
- 乙：起点在更早的 c0，没人动，所以抓到；但这靠「没有第二个会话开轮」（`m1_roundmove` 就是它被打破）。
- 丙、丁、戊：窗口是「不在最近一棵全绿树里的行」，B 在 A 跑完之后才放进暂存区的行不在 A 绿的那棵树里，下一趟必在窗口里：抓到。丙对这条是靠构造，不靠时序。
- 丙的残余（推的，没造出样本）：外壳在「staged_tree="$(git write-tree)"」（`scripts/gate/gate-staged.sh:28`）算暂存树，门禁里层在「if ! git -C "$source_repository_root" diff --cached --binary > "$staged_scratch_directory/staged.patch"; then」（`.claude/singlefs-ai-sop/scripts/gate.sh:56`）才取暂存区的 diff，两者之间有一小段时间；暂存区先加后退回会让「被判的树」与「写进 staged-green 的树」不同，而「if [[ "$now_tree" != "$staged_tree" ]]; then」（`scripts/gate/gate-staged.sh:39`）只比头尾。这条竞争窗口极窄，第二节引的 stage-must-run 复用判据已经背着同一个风险。另一条残余：`--force` 强制跑过的阶段不改退出码——「带 --force 强制跑过的阶段不让退出码变非 0」（`.claude/singlefs-ai-sop/scripts/gate.sh:8`），而外壳「if [[ $gate_rc -eq 0 ]]; then」（`scripts/gate/gate-staged.sh:36`）只看退出码，强制跑的整轮也会前移 staged-green；丙要求前移之前加一条「带 `--force` 不前移」的守卫，gate-ok 一路已经有这条：「if [[ -n "$STAGE_FORCED_SUMMARY" ]]; then GATE_OK_HELD_BY+=("$name（强制跑过）"); fi」（`.claude/singlefs-ai-sop/scripts/gate.sh:252`）。
- 什么现象会推翻 H3：`m1_roundmove`、`m3_toctou` 在你的临时仓里丙判成漏，或造得出一段交错使 staged-green 前移到含未判行的树上（那就要检查上面引的树比较那一行）。

## 六、H4　代价

**每个候选要改哪些文件**（都是对 `scripts/`、`.claude/` 的改动；`crates/` 里没有相关实现，见第一节的 grep）：

| 候选 | 要改的文件 | 动用户 2026-09-24 定案吗 | C528（两条门禁路判的改动窗口不同）现状说明要改哪句 |
|---|---|---|---|
| 甲 | `scripts/gate/changed-paths.sh` 的 `gate_diff_base` 的 `gate` 一档改成 HEAD（或让外壳给里层设 `GATE_BASE=HEAD`） | 不读 `GATE_DIFF_BASE`，但让两条路窗口相同＝改窗口关系 | 整条作废 |
| 乙 | 写起点的人或脚本（没定）、`changed-paths.sh` 回退次序、`lib.sh` 不动 | 不动 | 「全推出去了时一个是 HEAD~1、一个是 HEAD」仍成立 |
| 丙（改法 ①，外壳设 `GATE_BASE`） | 只改 `scripts/gate/gate-staged.sh`：开跑前 `export GATE_BASE=refs/sop/staged-green` 的提交（有这条 ref 时）；第二节引的上游 `gate.sh` 那一行本来就认 `${GATE_BASE:-…}`，上游不动 | 不动 | `changed-paths.sh` 文件头第 12–13 行与 C528 里「上游 `diff_base`」那半句要改成「staged-green」；**副作用**：所有读 `GATE_BASE` 的阶段一起变宽（knowledge-sync 判「这一轮碰了哪些」、`archive-past-rounds.py` 的 `base_of`、doc-decisions，以及上游阶段）。`scripts/gate/changed-paths.sh:14` 点名 `batch-scope` 格要的是「这一次提交要带哪些」、不该变宽，它读 `head` 取法所以不受影响 |
| 丙（改法 ②，加一档取法） | `scripts/gate/changed-paths.sh` 加一档只给「新增行」格用的取法（有 `SINGLEFS_STAGED_TREE` 且 `refs/sop/staged-green` 解析得到提交就取它，否则同 `gate`）；调用点改三处：`.claude/gate.d/doc-experiments.sh:139`、`.claude/gate.d/doc-process-records.sh:368`、`.claude/gate.d/doc-decisions.sh:1258`；`changed-paths.sh` 自证加格 | 丙只改带暂存：不读 `GATE_DIFF_BASE`，不动 `GATE_BASE` 的认法，路径窗口不变；丙两条路（N 也取它）：改变两条路新增行窗口的关系，按跑前条款第五行交用户 | 同上两处，且只说「新增行」的那几格；路径窗口（`gate_changed_paths`）仍是原说明 |
| 丁 | `gate-staged.sh`（或每趟开头）挂标记、`changed-paths.sh` 沿 first-parent 找标记、一族 `refs/notes/…`、`.git/hooks` 不入库所以不能靠提交钩子 | 同丙两条路 | 同丙 |
| 戊 | `doc-experiments.sh` 的 quoted-result-lines 逻辑（去掉 `SCOPE_BASE` 分支或默认全量）、一张只缩不涨的豁免表与它的自证、其余五格（`results-cited`、`evidence-in-repo`、`decision-links`、`archive-past-rounds`、`multipath-registry`）和 doc-decisions、knowledge-sync 同类处理；上游规则 `.claude/singlefs-ai-sop/rules/evidence-discipline.md:38` 要改 | 不读 `GATE_DIFF_BASE`；但「窗口」这个设计被取消，C528 整条失效 | 整条作废 |
| 庚 | `changed-paths.sh` 的 `gate` 一档加 gate-ok | 改 N 一路的回退次序＝改窗口关系 | 「全推出去了时」那句改 |

和单一取法的约束冲突与否：「算 diff 基准只许在 scripts/gate/changed-paths.sh 里：代码行出现 merge-base（--is-ancestor 核祖先除外）、@{upstream} 或 @{u}、gate-ok、」（`.claude/gate.d/code-tooling.sh:1031`）。丙改法 ② 把读 `staged-green` 的代码放在 `changed-paths.sh` 里，合这一条；改法 ① 在 `scripts/gate/gate-staged.sh`（不在该格射程 `.claude/gate.d/` 顶层之内）里设基准，格不会红，但等于第二处算基准，违背那一格的立意。`SINGLEFS_STAGED_TREE` 到得了阶段里：外壳「export SINGLEFS_STAGED_TREE」（`scripts/gate/gate-staged.sh:32`）；复用判据文件头写它被阶段读：「所以这一道只在 SINGLEFS_STAGED_TREE 给了树对象时才许复用——那个值由 scripts/gate/gate-staged.sh 算一次、」（`scripts/gate/stage-must-run.sh:39`），推的：同一个环境变量对 quoted-result-lines 所在阶段也到得了，没实跑验。

**窗口变宽之后存量红有多少**（`research/prompts/c439-r1-sonnet-model/measure-h4.sh`，对本仓 `f111c3f3` 的 HEAD 克隆里只读量，不动本仓的 ref、暂存区、工作区；原样输出在 `research/prompts/c439-r1-sonnet-model/out/h4-measure.txt`，24 行）：

```
HEAD=f111c3f3 HEAD~1=f0b94fef staged-green=e2ae1837（提交日期 2026-09-22，树 9b1d9d06，HEAD 的树 e843d87b）
staged-green 到 HEAD 的提交数（不是祖先，数的是 HEAD 的全部历史）：372；merge-base：
.out 里 E7RESULT 行（去重）：538831
  （gate_added_lines N 今天（HEAD） 用时 0 秒）
N 今天（HEAD）           新增行总数=0  格射程内的 E7RESULT 新增行=0  其中（去重后）.out 里逐字找不到=0
  照格的口径（登记着的产物 + 归档版）粗估：
    射程内 E7RESULT 新增行（路径加行文去重）=0  登记着的产物 241 份（树里 131 份，归档版取到 0 份）
    对照里逐字找不到=0（其中只在没登记的产物里有=0，哪儿都没有=0）；其中只在「## 历史版本」之后=0（门禁对历史节按行号只判新增的，这里不分）
  （gate_added_lines S 今天（HEAD~1） 用时 1 秒）
S 今天（HEAD~1）         新增行总数=1182  格射程内的 E7RESULT 新增行=0  其中（去重后）.out 里逐字找不到=0
  照格的口径（登记着的产物 + 归档版）粗估：
    射程内 E7RESULT 新增行（路径加行文去重）=0  登记着的产物 241 份（树里 131 份，归档版取到 0 份）
    对照里逐字找不到=0（其中只在没登记的产物里有=0，哪儿都没有=0）；其中只在「## 历史版本」之后=0（门禁对历史节按行号只判新增的，这里不分）
  （gate_added_lines 丙（staged-green） 用时 20 秒）
丙（staged-green）        新增行总数=786974  格射程内的 E7RESULT 新增行=1326  其中（去重后）.out 里逐字找不到=269
  照格的口径（登记着的产物 + 归档版）粗估：
    射程内 E7RESULT 新增行（路径加行文去重）=1283  登记着的产物 241 份（树里 131 份，归档版取到 110 份）
    对照里逐字找不到=351（其中只在没登记的产物里有=83，哪儿都没有=268）；其中只在「## 历史版本」之后=71（门禁对历史节按行号只判新增的，这里不分）
  （gate_added_lines 戊（空树，全量） 用时 60 秒）
戊（空树，全量）     新增行总数=821875  格射程内的 E7RESULT 新增行=1794  其中（去重后）.out 里逐字找不到=564
  照格的口径（登记着的产物 + 归档版）粗估：
      （扫了归档过的 .out 817 份，补找到 464 行）
    射程内 E7RESULT 新增行（路径加行文去重）=1749  登记着的产物 241 份（树里 131 份，归档版取到 0 份）
    对照里逐字找不到=100（其中只在没登记的产物里有=0，哪儿都没有=100）；其中只在「## 历史版本」之后=0（门禁对历史节按行号只判新增的，这里不分）
```

读这张表的口径（我的话）：
- 「格射程内的 E7RESULT 新增行」按 `doc-experiments.sh:1630-1633` 的路径筛（不含 `-history.md`、`decisions-history/`、`research/prompts/`、`research/results/`）；「去重后 .out 里逐字找不到」是拿全部 `research/results/*.out` 当对照的粗上界。
- 「照格的口径粗估」才接近门禁原判：窗口内的行只对照 `replay.sh` 登记着的产物，加登记着却已出了树的归档版（「登记的产物已归档出了树的，去版本库历史里取它最后那一版。退回全量判时照旧对照全部产物加全部归档。」`.claude/gate.d/doc-experiments.sh:1597`）；戊用全量口径＝对照全部 .out 加全部归档（我扫了 817 份归档过的 .out）。历史节的行门禁按行号只判新增的，这里按「路径加行文」去重数，没分。所以 351 是上界，不是红的确数。
- 此刻 N 今天（HEAD）窗口为 0 行是因为克隆里是干净的 HEAD；本仓工作区里别的会话没提交的改动 N 一路会算进去，我没量。
- 时间：`gate_added_lines` 对 staged-green 做基准用了 10 到 48 秒（几次跑出的值，随机器负载变），今天的 HEAD 与 HEAD~1 是 0 到 1 秒。

| 候选 | 此刻窗口里 quoted-result-lines 格要判的行 | 其中照格的口径粗估找不到对照 | 备注 |
|---|---|---|---|
| 今天 N（HEAD） | 0 | 0 | 克隆干净 |
| 今天 S（HEAD~1） | 0（总新增行 1182） | 0 | |
| 丙（staged-green，2026-09-22） | 1326 行（去重 1283 条） | 351（只在没登记的旧产物里有 83、哪儿都没有 268；其中 71 条只在历史节） | 总新增行 786974，格只取射程内的 |
| 戊（空树，全量口径） | 1794 行（去重 1749 条） | 100（扫完 817 份归档之后；只对照在树上的 .out 时是 564） | 豁免表约 100 行 |

- 丙：头一趟带 `--staged` 的整轮会在最多 351 条上红。红了 staged-green 不前移（`scripts/gate/gate-staged.sh:50` 的分支：「echo "  ! 整轮没全绿，refs/sop/staged-green 不动：红的这一趟什么也没证明"」），窗口不缩，下一趟还是这批——清掉或登记之前这一格永远红。staged-green 停在 2026-09-22 本身就是数据：从那以后（我数 143 个提交）没有一趟带 `--staged` 的整轮是全绿的（或没人前移），丙的窗口宽度＝距上次 S 全绿多久，门禁越少跑绿窗口越宽。
- 戊：每趟全量判，`gate_added_lines` 同量级（10 到 48 秒，另加扫归档的时间：817 份里有的上百 MB，我没单独计时，整套复跑（含这一扫）在 8G 上限下实测 1 到 5 分钟，随负载变；门禁里要预先缓存）；换掉了「只判新增」，也换掉了射程 2（新增行只对照登记着的产物，抓「重跑之后正文没跟」）：全量口径下抄了旧一轮产物里的行也算找得到。与规则冲突：「⇒ 正文引产物的检查按『这次改动』判，不按全仓判。」属 `.claude/singlefs-ai-sop/rules/evidence-discipline.md:38` 的条款，上游共享规则；`.claude/gate.d/doc-experiments.sh:1601` 注释第 4 条（「只判这次改动新增或改写的行：正文里早先抄下的行是历史，那时的产物按归档规则删掉了，再判就是判一个不存在的对照；」）同立场。
- 其余五格与 doc-decisions、knowledge-sync 同类的存量红我没量：它们不是整行抄产物，没有现成的「找得到」判据可代入，要主 agent 或实现员逐格看。判不了。

**与 `stage-must-run.sh` 复用判据是不是同一个起点**：丙是——都用 `refs/sop/staged-green` 那棵树（`scripts/gate/stage-must-run.sh:16`），前移也是同一次（`scripts/gate/gate-staged.sh:46`）；复用与窗口同进同退，只维护一个起点。甲、乙、丁、戊、庚都不是，要保持两个起点一致。

**C439 改法一列（若采丙，供主 agent 写回时参考，我不写）**：「基准改取 `refs/sop/staged-green` 那棵树与这次暂存树的树对树 diff（没有这条 ref 时退回现在的取法）；只给『新增行』格用；前移前补『带 `--force` 不前移』的守卫；头一趟先清掉或登记存量」。

## 七、H5　会红的样本与判别力自证

样本形状（`research/prompts/c439-r1-sonnet-model/histories.sh` 的 `hist_h1a_original`、`hist_h1b_bypass`，自证函数 `h5_selftest`）：

- 整轮全绿之后同一轮里早先写下的错引文（`h1a_original`）：c0 两路各绿一次；c1 写一行 `E7RESULT Lbad_A=1` 绕过门禁直接提交；工作区另有一行未提交的好行，不带 `--staged` 的整轮全绿（gate-ok→c1）；c2 绕过提交；暂存新行，提交前跑带 `--staged`。
- 绕过门禁提交的错引文（`h1b_bypass`）：同样的 c0；c1 写 Lbad_A、c2 写好行，都绕过；暂存新行，提交前跑带 `--staged`。

选定取法（丙两版）下必须进窗口、换回今天必须漏掉。原样输出（`out/h5.txt`）：

```
PASS h1a_original / 丙只改带暂存：Lbad_A 进过窗口=yes
PASS h1a_original / 丙两条路：Lbad_A 进过窗口=yes
PASS h1a_original / 今天：Lbad_A 进过窗口=no
PASS h1b_bypass / 丙只改带暂存：Lbad_A 进过窗口=yes
PASS h1b_bypass / 丙两条路：Lbad_A 进过窗口=yes
PASS h1b_bypass / 今天：Lbad_A 进过窗口=no
h5 自证查了 6 项，红 0 项
```

证红（把丙退回今天的取法，`MODEL_BREAK=bing-as-today`，`out/h5-broken.txt`）：

```
FAIL h1a_original / 丙只改带暂存：期望 yes，实测 no
FAIL h1a_original / 丙两条路：期望 yes，实测 no
PASS h1a_original / 今天：Lbad_A 进过窗口=no
FAIL h1b_bypass / 丙只改带暂存：期望 yes，实测 no
FAIL h1b_bypass / 丙两条路：期望 yes，实测 no
PASS h1b_bypass / 今天：Lbad_A 进过窗口=no
h5 自证查了 6 项，红 4 项
退出码 1
```

- 什么现象会推翻 H5：把丙换回今天，`out/h5-broken.txt` 若仍判绿，说明自证没有判别力（它实测红 4 项，退出码 1）。
- 这是窗口层面的自证（账本里 tag 进没进窗口），不是在真 quoted-result-lines 格上跑；真格的样本要主 agent 在 `.claude/gate.d/fixtures/` 的格式里造（本腿不跑门禁阶段）。

## 八、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| H1 今天 | 漏洞成立（射程见下） | 14 段里带错行的 13 段中 9 段错行永久不进窗口；C439 原形在今天的脚本上复现（`h1a_original` 对 `h1a_nomove`）；`h1a_immediate` 被 gate-ok＝HEAD 的跳过兜住，要隔一个没验的提交才漏 |
| H1 跑前条款第一行 | 不触发 | 我造出了今天的漏判，不降为「只在不带 `--staged` 时成立」 |
| H2/H3/H4 甲 | 站不住 | 13 段全漏，比今天还窄 |
| H2/H3/H4 乙 | 站不住 | 共用 ref 被第二个会话覆盖（`m1_roundmove`）、改写历史也漏；门禁里没有会话号可分 |
| H2/H3/H4 丙只改带暂存 | 站得住（有射程） | 14 段漏 1（只跑不带 `--staged` 的整轮）；改一个文件即可（外壳设 `GATE_BASE`，副作用大）或加一档取法；起点与复用判据同一个 |
| H2/H3/H4 丙两条路 | 站得住，但改窗口关系 | 14 段全进；要改 `changed-paths.sh` 的 N 一路，按跑前条款第五行交用户 |
| H2/H3/H4 丁 | 站不住（带条件） | 改写历史时漏 1 段；标记落不上时窗口回弹；条件是本仓不改写已提交历史时与丙等价、代价高于丙 |
| H2/H3/H4 戊 | 站得住，代价高，交用户 | 14 段全进；约 100 行存量豁免、换掉「只判新增」与射程 2、与 `.claude/singlefs-ai-sop/rules/evidence-discipline.md:38` 冲突、其余五格同类账没量 |
| H2/H3/H4 庚 | 站不住 | 7 段漏；只修 gate-ok 被 N 盖章的一类 |
| H3 交错 | 丙、丁、戊不漏；今天、甲、庚、乙各有漏法 | `m3_toctou` 对今天、甲、庚是漏，乙靠没人覆盖起点才不漏 |
| H4 存量红 | 判得出的数已给 | 丙头一趟最多 351 条红（上界）；戊约 100；其余五格判不了 |
| H5 判别力自证 | 站得住 | 丙两版进窗口、今天漏，6/6 对；退回今天自证红 4/6 |
| 跑前条款 | 触发两条 | 「某候选在两个会话交错时把起点挪到未判改动之后 ⇒ 出局」触发于乙；「两个以上候选站得住 ⇒ 岔路单」触发于丙与戊；「留下的候选要动用户 2026-09-24 的定案」只在丙两条路时触发 |

## 九、没做什么

- 没跑任何门禁阶段、`gate.sh`、`gate-staged.sh`、cargo；没在真 quoted-result-lines 格上跑样本；所有「窗口」都是照冻结副本规则在临时仓里的模拟（`gate_diff_base`、`gate_added_lines`、上游 `diff_base` 是 source 冻结副本的原函数，前移规则和红绿判定是我照引文写的）。
- 没量：丁标记落不上的频率；`SINGLEFS_STAGED_TREE` 在 quoted-result-lines 所在阶段里实读得到（推的）；丙改法 ① 对上游读 `GATE_BASE` 的阶段（`show-me-test` 等）变宽后的红数；除 quoted-result-lines 外其余用 `gate_added_lines` 的格的存量；staged-green 前移与暂存树比较之间的竞争窗口（只推）；真并发（只做了顺序交错）；`git worktree` 多个暂存区的形状。
- 351 与 100 是上界粗估，不是门禁原判。
- 没替主 agent采纳或出判决；「站得住/站不住」是我对每格的判定，岔路单与改法写回由主 agent 做。
- 没读禁读清单；没改冻结副本；没在本仓写任何 ref。

## 十、复跑

`bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/c439-r1-sonnet-model/tools/rerun.sh --repo <仓根> --frozen /home/fy5090/.local/share/singlefs/work/c439-r1/frozen --work <新的草稿目录> --model research/prompts/c439-r1-sonnet-model`。我复跑两次：`out/h1h3-summary.txt`、`out/h1h3-details.txt`、`out/h5.txt`、`out/h5-broken.txt` 四份逐字相同（`name=rerun_compare … same`）；`out/h4-measure.txt` 随仓与产物变，逐行比时除用时外相同。跑一遍约 1 到 5 分钟（随负载）。

## 附：`m2_lastwriter`（H3 里「把别的会话已判绿的行算进窗口」的原样逐趟输出，丙两条路与今天）

```
── m2_lastwriter / 今天
  步1 S 今天 基准=c0 窗口={ green 初始带暂存}
  步2 S 今天 基准=c0 窗口={Lok_a1 green A带暂存含a1}
  步3 S 今天 基准=c0 窗口={Lok_b1 green B带暂存不含a1}
  步4 S 今天 基准=c0 窗口={Lok_a1,Lok_b1 green A重新暂存a1}
── m2_lastwriter / 丙两条路
  步1 S 丙两条路 基准=c0 窗口={ green 初始带暂存}
  步2 S 丙两条路 基准=绿树@c0 窗口={Lok_a1 green A带暂存含a1}
  步3 S 丙两条路 基准=绿树@c0 窗口={Lok_b1 green B带暂存不含a1}
  步4 S 丙两条路 基准=绿树@c0 窗口={Lok_a1 green A重新暂存a1}
```

## 十一、交回前核对

- 草稿目录里的临时 git 仓、仓副本、克隆、冻结副本的拷贝、`upstream-lib-stub` 全部删了；草稿目录只剩派发时的 `prompt.md` 与锁文件。本仓没有写任何 ref、暂存区与工作区（只新增了本报告与模型目录 `research/prompts/c439-r1-sonnet-model/`）。
- `python3 scripts/agents/cite-check.py research/prompts/c439-r1-sonnet-output.md --root /home/fy5090/code/singlefs` 的末行：

```
  ✓ 核了 30 处引文、13 处裸行号，对上 30 处，没判 13 处（其中裸行号没有引文可比 13 处；逐处列在上面）
```
