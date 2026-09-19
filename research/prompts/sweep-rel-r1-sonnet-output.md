# sweep-rel-r1 正推腿（Sonnet）判决：A1、A4

判据文本、被判对象与开工快照均已现查（`research/prompts/_sweep-rel-r1-background.md`、`.claude/agents/sweep.md`、`.claude/main-agent.md`、`.claude/gate.d/73-research-gate-lint.sh`、`.claude/singlefs-ai-sop/scripts/gate-lint.sh`、`.claude/gate.d/89-stage-selftest.sh`）。所有引文均在本报告写入前对被引文件跑过一次 `grep -nF`，行号是该文件自己的行号（不是背景材料的行号）；`00c9d4f` 那一版与当前工作区在本报告引用的每一处均已用 `git diff 00c9d4f -- <文件>` 核过是否有改动，未改动之处两版行号相同。

工作区当前有别的会话未提交的改动（`git status --short` 可见 `.claude/kb/checks-owed.md` 等多份文件被改），本报告只判 `.claude/agents/sweep.md`、`.claude/main-agent.md` 那一行、`.claude/gate.d/73-research-gate-lint.sh` 与其样本这几个被判对象本身；引用别的 kb 文件时一律注明版本（`00c9d4f` 还是当前工作区），不把别的会话正在改的内容当成这一轮的判据。

## 一、A1：sweep.md 第 7–11 步、main-agent.md 那一行

### A1-1（打中）：main-agent.md 没有给逐行判段传「这一阶段做成的事」，而 sweep.md 第 10 步需要它

`sweep.md` 第 16–22 行「输入」一节，「阶段同步」是一个整体的活，只有一个项目符号列出它的全部输入项：

```
grep -n "这一阶段做成的事与第一次在真活上用上的东西" .claude/agents/sweep.md
```
命中：第 22 行——「你做哪一段（写事实表，或逐行判：候选表路径与分给你的组号，例 F1–F6）；这一阶段做成的事与第一次在真活上用上的东西，一行一件，没留改动的也写（例：第一次在真派发里用看门狗）；真活使用的证据由主 agent 在这里给，你不自己读会话记录（`~/.claude/` 照共用约束不碰）。」——这一整句是**同一个项目符号**里的分句，字面上不分「写事实表段才给」还是「两段都给」，就是「阶段同步」这个活的输入清单，两段共用。

而 sweep.md 第 10 步明写要用到这份清单：
```
grep -n "读完分到的组，再反向问一遍" .claude/agents/sweep.md
```
命中：第 38 行——「读完分到的组，再反向问一遍：每件做成的事，在该记它的载体里有没有一处记着，没有的在「## 逐行判定」里加一行（组写 M1、M2……，判定「要补」）。」——「每件做成的事」指的正是第 22 行那份清单；没有这份清单，逐行判的 sweep 无从知道有哪些「做成的事」要反向核。

但 `main-agent.md` 第 43 行给两段各写了各的输入，写给逐行判段的只有候选表与组号：
```
grep -n "候选表按组切段" .claude/main-agent.md
```
命中：第 43 行——「...→ 候选表按组切段，每段不超过约 200 行，一段派一个 `sweep` 逐行判（第二段）→ 交回齐了主 agent 跑 `stale-candidates.py --check-report`...」——这一句只交代候选表与组号，没有一个字提到把「做成的事」清单也发给逐行判段的 sweep。写事实表那一段在同一行前半明确写了「输入给改动范围与做成的事」，逐行判段没有对应的「输入给……」小句。

**会不会做错**：会。若主 agent 照 `main-agent.md` 第 43 行字面派发（只给候选表路径与组号），逐行判的 sweep 拿不到「做成的事」清单，第 10 步的反向核检就无从执行——不是漏判某一行，是整个第 10 步的输入缺失，只能要么跳过（报告里不出现任何 M 编号）、要么凭候选表内容自己瞎猜「做成的事」有哪些，两种都不是定义要的。这与背景材料 A1 判据「两段的输入……冲不冲突」直接对应：不是 sweep.md 与 agent-common.md 冲突，是 sweep.md 的输入要求与 main-agent.md 派发那一行的字面描述不一致。

**推翻条件**：若能找到 main-agent.md 或 sweep.md 别处另有一句要求「派逐行判时也要带上做成的事清单」，或 sweep.md 第 10 步改成「向主 agent 要做成的事清单」这类主动索取的写法，这一条就不成立。我按上面两处引文逐字读过，没有找到这样的句子。

### A1-2（打中，「反过来」的形态）：第 9 步「带日期的分句」判据在复合句里可能把该重判的现状分句当成被日期锁定的事件句放过

第 9 步原文（已现查行号）：
```
awk 'NR==37' .claude/agents/sweep.md
```
命中第 37 行，关键两句：「带日期的分句，日期后面说的是那天发生了什么（改成、定案、跑了一次）才是事件句；说的是到那天为止的现状（「2026-09-14 已有 X；层 0 只有第一个事务」的后半句）就当现状句判。」

这条判据用**分号**把例句切成两半来演示「同一整句里，一半事件、一半现状」，隐含的操作方法是「按分句拆开，各自看日期管不管得到它」。但 `00c9d4f` 那一版真实存在的行里，日期与后续内容是用**冒号**连起来的（日期只出现在冒号前的短语里，后面跟一长串没有自己日期的具体数字/状态），这种结构下「日期后面说的是那天发生了什么」这句话按字面适用的范围并不清楚——冒号后的内容到底算「日期后面说的」（从而被日期定住、事件句不改），还是算另一个独立的、要单独判真假的现状分句。

真实例子（`.claude/kb/milestone/02-second-txn.md` 第 222 行，`00c9d4f` 与当前工作区逐字相同，已用 `git diff 00c9d4f -- .claude/kb/milestone/02-second-txn.md` 核过这一行未被改动）：
```
git show 00c9d4f:.claude/kb/milestone/02-second-txn.md | grep -n "现状（2026-09-17 部分落地）"
```
命中第 222 行开头：「**现状（2026-09-17 部分落地）**：层 0 的负载是两条流……门禁 54 号两条流全量都零违例：第一个事务 262165 个状态，固定脚本到 E 2104413 个状态（exhaustive=true，每个状态两遍恢复 + **26 条不变量的 checker** + 记录核对器）；……」

「现状（2026-09-17 部分落地）」是一个带日期的短语，「部分落地」是一个类似「改成」「定案」的状态变化动词（那天从「没有」变成「部分落地」），照第 9 步的判据，这一小段本身更像事件句。但冒号后面的整段内容——包括「26 条不变量的 checker」这个可以独立核实真假的数字——字面上并没有自己的日期，如果把它当成「日期后面说的内容」一并归入事件句不改，这个数字就永远不会被重判。


现查这个数字是否已经过时（不作为判据，只作为「这个歧义有没有实际后果」的旁证）：
```
git show 00c9d4f:.claude/kb/checks-owed.md | grep -n "池级 checker 判的 23 条"
```
命中第 24 行（C13 前置列）：「层 1 harness、每套布局的 checker ⚠️ 2026-09-14 第一套布局的形态有了：池级 checker 判的 23 条每条一份坏镜像……」——这是 `00c9d4f` 那一版的数（23 条），与第 222 行的「26 条」本来就不是同一次记的数、也可能不是同一个量（前者是「每条配一份已知坏镜像」的测试条数，后者是「层 0 崩溃点重放里 checker 实际评估的不变量条数」），我没有把两者判成同一个量，只是指出：这两条互相独立、各自可能过期的数字，若被第 9 步的日期规则一并锁进「事件句不改」，回扫员就再也不会去核它们了。当前工作区里 `checks-owed.md` 第 24 行已被另一个未提交的改动改成「29 条」（`sed -n '24p' .claude/kb/checks-owed.md`），而 `milestone/02-second-txn.md` 第 222 行的「26 条」在同一个 `git diff 00c9d4f` 里没有被这个未提交改动碰过——这恰好是「一个量的两处记录，一处已经在被回扫、另一处字面上可能被判据放过」这个风险的活样本，但我不确认这两个「条」是不是同一个量，因此不把它算作一次「打中的腐烂」，只算 A1-2 判据歧义的旁证。

**推翻条件**：若 sweep.md 第 9 步补一句「冒号或分号之后没有自己日期的每个子分句，各自单独判」（或等价的显式拆句规则），这条歧义就消失。现在的行文只用一个分号例句演示拆分，没有说清「日期后面」这个短语在遇到冒号引导的长列举时算不算把整段都锁住。

### A1-3（部分打中）：main-agent.md「抽 20 行复判」写了「重派」，没写清重派之后要不要再抽验

```
grep -n "随机抽 20 行自己复判" .claude/main-agent.md
```
命中第 43 行：「……再从判成不相干与事件句不改的行里随机抽 20 行自己复判，抽到判错的那一段重派 → 主 agent 逐处判……」

背景材料 A1 判据明写要核「主 agent 那一行的抽 20 行复判有没有写清抽中判错之后怎么办」。这句话写了「抽中判错 ⇒ 重派那一段」，这一半是写清楚了的（不是空话）。但没写清的是：重派回来的那一段，要不要再抽样验一次，还是主 agent 从此直接信它。`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「一条腿只抽一次样不算一次观测」一节讲的是「模型的答复是有变化的观测」，这条规则原文针对的是三方论证里的推论腿，不直接约束这里的「回扫员逐行判 QA 抽样」，我不把这一条算成与该规则的直接冲突，只记为定义本身留白：重派之后如果不再抽验，一次误判被发现、改一次，之后同一段里其余的误判（没被那 20 行抽到的）不会再被查到，等于只堵了这一次抽样运气好抓到的那个坑。

**推翻条件**：若 main-agent.md 那一行后面另有一句「重派回来的那一段再抽 N 行复判」或等价规定，这一条不成立；我在同一行、以及 sweep.md 全文里都没找到这样的句子。

### A1-4：没有找到的冲突（如实报告）

- 逐字核过 sweep.md 第 41–43 行「写范围」（「报告文件、草稿目录。不改任何被搜到的文件，不改 `stale=`。」）与 `.claude/agent-common.md` 第 23–30 行「写」一节：sweep.md 只有 `Read, Bash`（第 4 行 `tools: Read, Bash`），第 7、8 步里 `--out <草稿目录>/changes.md`、`--facts ... --out 候选表` 都是新建文件，`stale-candidates.py` 的 `--out` 参数用 `open(arguments.out, 'x', ...)`（背景材料附录二第 718 行）排他新建——这与 agent-common.md 第 27 行「tools 只有 Read / Bash 的：新建文件一律排他……」一致，没有冲突。
- 逐字核过第 7、8、9、11 步与共用约束「不做」一节（agent-common.md 第 32–43 行）：第 7–11 步全程只跑 `stale-candidates.py` 与 `git show`/`git log`，没有 `git add`/`commit`/`stash` 之类的写操作，没有编译 Rust，没有跑 `gate.sh` 全量，符合「不做」一节。
- 第 9、10 步排除 `research/prompts/` 与 `briefs/` 的写法（第 37、38 行）与 `.claude/rules/three-way-inference.md`「原样保存的证据不许事后改」一节（本文件我未获授权读取正文，仅按背景材料判据表引用的口径核对：`research/prompts/` 是冻结证据）方向一致，没有发现字面冲突。

## 二、A4：门禁 73 号


### A4-1：射程本身对——没打中

```
grep -n 'ROOT/research/scripts\|ROOT/.claude/hooks\|不扫' .claude/gate.d/73-research-gate-lint.sh
```
命中第 3、4、86 行：脚本头部注释第 3–4 行明写「research/prompts/ 下的脚本是冻结证据，不扫（同 .claude/doc-lint-exclude 那一行的理由）」，第 86 行 `for directory in "$ROOT/research/scripts" "$ROOT/.claude/hooks"` 只把这两个目录收进 `TARGETS`。现查 `.claude/doc-lint-exclude`：
```
grep -n "research/prompts" .claude/doc-lint-exclude
```
命中第 4 行，理由一致（「当时原样发给模型的提示，与 research/results/ 的产物一一对应；改提示只能连同重跑一起改」）。射程与注释所写一致，`research/prompts/` 确实不在扫描范围内。

### A4-2（打中）：共享 gate-lint 在样本临时目录里跑的时候，同时把真仓的 `.claude/singlefs-ai-sop` 包也扫了进去，样本没有被单独判

`73-research-gate-lint.sh` 调用共享脚本时没有设 `GATE_LINT_DIR`：
```
grep -n 'LINT="' .claude/gate.d/73-research-gate-lint.sh
grep -n 'bash "\$LINT"' .claude/gate.d/73-research-gate-lint.sh
```
命中第 9、17 行：`LINT="$(cd "$(dirname "$0")/../.." && pwd)/.claude/singlefs-ai-sop/scripts/gate-lint.sh"`；`if bash "$LINT" "${TARGETS[@]}"; then`。

而共享脚本 `gate-lint.sh` 的扫描范围分两部分：
```
grep -n 'GATE_LINT_DIR\|^SCAN=\|^SCANS=' .claude/singlefs-ai-sop/scripts/gate-lint.sh
```
命中第 39、48、51 行：第 39 行注释「GATE_LINT_DIR 可指定要扫的目录（selftest 拿样本喂它用），默认扫本脚本所在目录」；第 48 行 `SCAN="${GATE_LINT_DIR:-$(cd "$SCRIPTS/.." && pwd)}"`；第 51 行 `SCANS=("$SCAN")`，后面才 `for d in "$@"; do ... SCANS+=(...); done` 把位置参数（也就是 73 号传进来的 `TARGETS`）追加进去。`73-research-gate-lint.sh` 没有设 `GATE_LINT_DIR`，所以 `SCAN` 落到默认值——真仓的 `.claude/singlefs-ai-sop`（`$SCRIPTS` 是共享脚本自己所在目录，`$SCRIPTS/..` 是真仓这份规范副本的根，不是样本、也不是 `89-stage-selftest.sh` 拷出来的临时目录）。

实测复现（把 `.claude/gate.d/fixtures/73-research-gate-lint.sh/red` 拷进临时目录、按 `89-stage-selftest.sh` 同样的方式调用）：
```
work=$(mktemp -d); cp -a .claude/gate.d/fixtures/73-research-gate-lint.sh/red/. "$work/"
env -u GATE_BASE -u GATE_STAGED_FROM bash .claude/gate.d/73-research-gate-lint.sh "$work" 2>&1 | tail -4
rm -rf "$work"
```
原样输出：
```
  ✗ 门禁自检失败：1 处（共 17 个脚本、148 条拒绝）
  ✗ research/scripts/ 或 .claude/hooks/ 里有不带出路的拒绝（上面逐处列出）
     → 怎么办：照 .claude/singlefs-ai-sop/rules/sop-first.md「每一条拒绝都必须给出下一步」补出路：die 加第二个参数，bad 后五行内写 howto，打印的 ✗ 之后跟一行 →
```
样本 `red/research/scripts/sample.sh` 只有 1 个脚本、1 条拒绝，而输出报「共 17 个脚本、148 条拒绝」——多出的 17 个脚本、147 条拒绝全部来自真仓的 `.claude/singlefs-ai-sop/scripts/`（用 green 样本重跑一遍能看到同一批 17 个脚本、147 条拒绝，且全部判定「都带了出路」，与样本无关）：
```
work=$(mktemp -d); cp -a .claude/gate.d/fixtures/73-research-gate-lint.sh/green/. "$work/"
env -u GATE_BASE -u GATE_STAGED_FROM bash .claude/gate.d/73-research-gate-lint.sh "$work" 2>&1 | tail -3
rm -rf "$work"
```
原样输出：
```
  ✓ 门禁自检通过：17 个脚本（.sh 与 .py）、147 条拒绝都带了出路
```

`89-stage-selftest.sh`（第 39 行 `env -u GATE_BASE -u GATE_STAGED_FROM bash "$stage" "$work"`）同样没有 unset 或设置 `GATE_LINT_DIR`，所以正式跑判别力自检时会撞上同一个耦合。

**这不是「射程扫错了目录」，是「样本判定的红绿结论今天恰好没被这份耦合带偏」**：因为真仓的 `.claude/singlefs-ai-sop` 包眼下自己一条违规都没有（147/148 条都带出路），所以红样本的那 1 条注入违规、绿样本的 0 条注入违规，仍然能正确地让退出码分别落在 1 和 0。但样本本身并没有被单独抽出来判——回答背景材料 A4 判据「共享 gate-lint 在样本临时目录里扫的是样本还是真仓」：**两者都扫，而且真仓那一部分（147/148 条）远远压过样本自己的 1 条**。这与 `.claude/singlefs-ai-sop/scripts/selftest.sh` 里同一个共享脚本的自检写法不一致：
```
grep -n 'GATE_LINT_DIR=' .claude/singlefs-ai-sop/scripts/selftest.sh
```
命中第 150、181、334 行，三处都显式 `env GATE_LINT_DIR="$d"`／`GATE_LINT_DIR="$r"` 把扫描范围钉死在样本目录，不依赖默认值。73 号没有照这个已有写法做。

**会不会「样本在改坏的阶段下照样判对」**：目前不会——因为 `TARGETS`（第 85–88 行）正确地由 `$1`（`89` 传进来的临时目录）算出，样本本身没有被吃掉；一旦真仓 `.claude/singlefs-ai-sop` 未来出现哪怕一条真实的「拒绝没出路」（即使与 73 号无关），绿样本会被这条无关的真仓违规拖到判红，89 号会报「green 期望退出 0，实测 1」，而这条红与 73 号脚本、与两个样本本身是否写对完全无关——判别力样本从此不再纯粹测的是 73 号自己的逻辑。

**推翻条件**：若 73 号脚本在调用 `$LINT` 前加一行 `export GATE_LINT_DIR=` 指向 `TARGETS` 之外一个不含脚本的空目录（或直接指向 `${TARGETS[0]}` 并把其余目标继续作为位置参数追加），使 `SCAN` 不再默认落到真仓的 `.claude/singlefs-ai-sop`，这一条即可撤销；我按当前文件内容核过，没有这一行。

### A4-3（如实报告，不算入 A4 打中）：门禁 73 号对当前真仓直接跑，退出码是 1

```
env -u GATE_BASE -u GATE_STAGED_FROM bash .claude/gate.d/73-research-gate-lint.sh; echo "exit=$?"
```
原样输出（节选）：
```
  ✗ relay-timing-lint.py:481  拒绝但没有出路（直接打印的 ✗，到下一处拒绝之前没有「→」）
                print(f"  ✗ 有 {len(unexempted)} 处读子进程输出的循环一边给行打时间戳一边转打（{statistics}）")
     → 怎么办： 在这条拒绝后面补一行出路，例： echo '     → 怎么办：<下一步做什么>'
                循环里逐条列明细的那种行，在行尾标 # gate-lint:detail（出路写在它的汇总上）；
                汇总行标 # gate-lint:summary。两种豁免都要显式写出来。

  ✗ 门禁自检失败：1 处（共 67 个脚本、253 条拒绝）
exit=1
```
`research/scripts/relay-timing-lint.py` 不在这一轮「一、被判的对象」表列出的改动范围内，也不在开工快照的 7 个文件里，按 `.claude/agent-common.md`「红了先看它点名的文件与行在不在这一轮的改动里；不在的不修，照写」——这条红与这一轮 sweep.md/main-agent.md/gate 73 的改动无关，不修，如实交回：门禁 73 号一旦挂进 `stage-owners.tsv` 并被人跑一次，会先撞上这个既有、与本轮无关的违规。


## 三、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| A1（第 7–11 步与调度那一行 vs agent-common、别的定义） | 打中（A1-1） | main-agent.md 第 43 行给逐行判段的派发只提了候选表与组号，没提 sweep.md 第 22 行要求的「这一阶段做成的事」清单，而 sweep.md 第 38 行第 10 步需要这份清单才能反向核 |
| A1（第 9 步「判的是分句」「带日期的分句」会不会把现状句判成事件句或反过来） | 打中（A1-2） | 冒号引导、日期只出现在冒号前短语里的复合句（`milestone/02-second-txn.md:222`「现状（2026-09-17 部分落地）：……26 条不变量的 checker……」），第 9 步只用分号例句演示拆句，没写清冒号后没有自己日期的内容要不要单独判 |
| A1（第 10 步与第 9 步重复或漏） | 部分打中（并入 A1-1） | 不是重复；「漏」体现在 A1-1：main-agent.md 没有把第 10 步要用的输入接上 |
| A1（抽 20 行复判有没有写清判错之后怎么办） | 部分打中（A1-3） | 写了「重派」，没写重派之后要不要再抽验 |
| A1（写范围与 agent-common.md、「不做」一节冲不冲突） | 没打中 | 逐字核过写范围、tools、Bash 排他新建、`research/prompts`/`briefs` 排除，没发现字面冲突 |
| A4（射程对不对） | 没打中 | `research/scripts/`、`.claude/hooks/` 收进 `TARGETS`，`research/prompts/` 按头部注释与 `.claude/doc-lint-exclude:4` 一致地不扫 |
| A4（红绿样本分不分得出） | 打中（并入 A4-2） | 今天分得出，但分的依据混进了真仓 `.claude/singlefs-ai-sop` 包（147/148 条），不是样本本身；真仓一旦出现无关违规，绿样本会被拖红 |
| A4（共享 gate-lint 扫的是样本还是真仓） | 打中（A4-2） | 两者都扫：`TARGETS`（第 1 个位置参数算出）是样本，`SCAN`（`gate-lint.sh` 默认值，因为没设 `GATE_LINT_DIR`）是真仓的 `.claude/singlefs-ai-sop`，且真仓那部分的规模（147/148 条）远超样本（1 条） |

## 四、没做什么

- 不判 A2、A3、L1–L10 逐格判定（分给攻方腿与本地攻方腿，禁读清单里也不许看它们的产出）。
- 不替攻方找绕法：A1-2、A4-2 只指出定义/脚本本身的歧义与耦合，没有去构造「如何利用它蒙混过关」的具体案例。
- A1-2 里「26 条」是否与 checks-owed.md 里「23/29 条」是同一个量，我没有判定，只作为歧义有没有实际后果的旁证，不算判据本身的证据。
- 没有跑 `stale-candidates.py --selftest` 或 `--benchmark`（那是 A2/A3 的射程，我没有改动过这个文件，也没有必要为 A1/A4 去跑它）。
- 没有去核 `.claude/rules/three-way-inference.md` 与 `format-evolution.md` 的正文（不在我的开工先读清单内，只按背景材料判据表引用的口径处理，没有整份读取）。
- 89-stage-selftest.sh 真正跑起来时是否也复现同样的耦合，我用等价的手工复现（`mktemp -d` + `cp -a` + 同样的 `env -u` 前缀）验证过，没有直接跑 `bash .claude/gate.d/89-stage-selftest.sh` 全量（那会覆盖别的阶段的样本，超出这一格的判据范围，也可能与同仓其他会话的未提交改动产生交互）。
- A4-3 只如实报告 `research/scripts/relay-timing-lint.py:481` 这条与本轮无关的既有红，没有去修它。

（报告完）
