# m3-defs-b9-r3 攻方腿（Opus）报告（2026-10-14）

<!-- doc-lint:not-numbers Q1 Q2 Q3 Q4 Q5 Q6 S2 S4 S6 P2 P4 P5 P6 U1 U2 U3 U4 Z1 Z4 Z5 Z7 F-Q4a F-Q4b F-Q1a -->

轮名 `m3-defs-b9-r3`（第二题第三轮，最终轮）。格：Q1、Q2、Q3、Q4。立场：造工作流历史攻前两轮站住的形态（S2、S4、S6 与第二轮辩方判「修到」的 Z1、Z4、Z5、Z7 条目）。

三份冻结副本下面简称：r3 = `~/.local/share/singlefs/work/m3-defs-b9-r3/repo/`（after-b11，被判的这一版）；r2 = `…/m3-defs-b9-r2/repo/`（after-b10-fixup）；r1 = `…/m3-defs-b9-r1/repo/`（after-b9-fixup，P 改法之前）。喂法全在我从三份各 rsync 出的拷贝上跑（不含 `crates/`、`research/results/` 与根下 `multi-host.env`），冻结副本本身没碰；样本里的私有值都是编的。

## 复跑

```
cd /home/fy5090/code/singlefs
bash scripts/hosts/run-with-memory-cap.sh 2G bash research/prompts/m3-defs-b9-r3-opus-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs --frozen ~/.local/share/singlefs/work/m3-defs-b9-r3/repo --work ~/.local/share/singlefs/work/<新的空目录>
```

- 前两轮冻结副本取 `--frozen` 的同级目录（`R1_FROZEN`、`R2_FROZEN` 可改）；一趟约 70 秒挂钟，前台跑得完，`RERUN_PARTS` 默认整趟（拷贝、q1–q4、清拷贝）。`--work` 必须是新的空目录。
- 输出里的草稿目录、模型目录、三份冻结副本路径换成 `<WORK>`、`<MODEL>`、`<FROZEN>`、`<R1>`、`<R2>`，样本里编的 ISO 时间戳的时分秒换成 `<时分秒>`，再与 `out/` 同名文件逐字节比。
- 我最后两趟复跑换了长短不同的草稿目录名，10 份输出都打 `same`。
- `tools/borrowed/` 是从前两轮攻方模型目录拷来的四个喂法（`z1_fence_feed.py`、`z4_source_drift.sh`、`z5_cells.py` 与 `z5-feed/`，第一轮；`s5_real_misread.sh`，第二轮），原样拷，只有 `z5-feed/setup.sh` 改了一处：样本记录里的 ISO 时间戳在跑的时候拼出来，文件里不留钟点字面。

模型目录每个文件的 sha256（`research/prompts/m3-defs-b9-r3-opus-model/SHA256SUMS` 同一份；`SHA256SUMS` 自己是 `ce76ad301d904a7ba1cafc6b373b98aeeb05522484061ee0ca074e8c7706b4ff`）：

```
1a5b7dbacd6f2b818824a949c4293f13ca20d297e7f6fbd5ebd43ede95e4ba09  ./gate-samples/make_trees.sh
47862629937609ef1e4ca52bf820f23b110c193a98f9ad899d6d49b8599c18f4  ./gate-samples/要主agent跑.txt
2716ccd6ee5f2a96bb22070f39beeb96432963ecf9cd8a142657421462b3564e  ./out/q1-skeletons.txt
40aa67dccabf55122eb827b92f12b127d54de9ed1bc5730284e58f004370b3e7  ./out/q2-selftest-copy.txt
d3a06edb6a7d88fff98bb6f34dd0461e7517746cf7f548aa68408b6791ffa745  ./out/q3-mask-probe.txt
239c91da57d940e7056f8deb00362ba3506e406b16ec1b15b37e4b5457ee6547  ./out/q4-date-once.txt
7e3af6686076af394884729c3e86f6fd79d3cd8afcdf0be4a37be50c6d15e5c4  ./out/q4-s5-misread-under-fix.txt
b25fbb3a67dae2dab63a88028b6092b13ffeb73ccad9dc01de6b93c5d88c9f77  ./out/q4-term-cut.txt
11202cad25c296df23a052be119e1d79dd598761e801ad409324ada3b01e4a30  ./out/q4-z1-fence.txt
e032861791a71e44611dc35ce681a36e9131ffc3d869ad8905ad32be8c2d7a39  ./out/q4-z4-daily-changes.txt
aa0cdc86e015fcd26c822c99b061a353743819a0a86427faca99ddf78ade0a51  ./out/q4-z5-cells-r1.txt
dc7be17ed3ef4fef059a4f016e0684cbde2e43f41f17a1bc4d1fe8efbf6c5c2e  ./out/q4-z5-cells-r3.txt
b2d13494fa27542603642a4ad9c2bd0acf15b34c0487f6b3f94a5e9267534b46  ./tools/borrowed/s5_real_misread.sh
0cbcd0d3289aee5209c7d7f6d6ac02ed3214468344a9510bc3a3bf9443addfa4  ./tools/borrowed/z1_fence_feed.py
813b3f2399fc4e5473c6491b1f47484753ffcba4e3187cf0005df46bacd76e2c  ./tools/borrowed/z4_source_drift.sh
5e0ceeb0458f7214df073a7130cd8336798016f5f6689296e99d88eea177507a  ./tools/borrowed/z5_cells.py
91d29fa2bef9db640dce0f3d41480ecd4fd7a7d4618dc21e056288c831c51f21  ./tools/borrowed/z5-feed/multi-host.env
c4b6cd4ee3e52051012ef1bd426ab3130e82681ef1c9547c0a8cfba83773612e  ./tools/borrowed/z5-feed/.private-values-hostname
370c3a40bd5a9288fe32c9d090307375ec60fce5e70ff83b4eaf2ef5169baa73  ./tools/borrowed/z5-feed/setup.sh
e110d58bab2492fad9634e2f3c681c4ec893597da384fe575f532458ed332bd9  ./tools/make_fix_tree.sh
8cdfd660662424b802dd9872fb0b9e42e1b9da451882d519d886748794d48789  ./tools/q1_skeletons.sh
8af1d58e81f64cb3cdebc255386f10798ca0f025c2aa267de8fcf115c90d297a  ./tools/q2_selftest_copy.sh
12611c2994b0574ef2659144354d9b481d25999644629cbd1209ec95c9c2108d  ./tools/q3_mask_probe.py
1a3d8890e1ab841717f8ca38ba9c2e8bf263587e4526b0c019f32436d7bfb910  ./tools/q4_date_once.sh
88bf24873aa90463b595ac7c5672d381fae4713a0f2b1a13e5d38654877a80bf  ./tools/q4_term_cut.sh
722fda0ef0c89d890217504714f22f3acb249ab844c0b409489eb07f746929df  ./tools/rerun.sh
```

## 各格判定一览

「量过」= 在拷贝上真跑了钩子入口、库的命令行或抽出的判法，原样输出在模型目录 `out/`；「推的」= 按代码或文字推，没跑。「r1 上」列是同一段历史在 P 改法之前那一版上的结局。

| 格 | 判定 | 照规则做、结果错的那一步（一句） | r1 上 | r2 上 | 哪一版带进来 | 量过 / 推的 |
|---|---|---|---|---|---|---|
| Q4-甲（Z4-丙 改后又中） | 打中 | 当天调度记录里叫停那一行把日期写在全角括号里「（2026-10-14）」，登记照抄；提交前 date-once 格红，照它的出路 `doc-consolidate.py migrate` 改回记录，删日期连括号一起删；叫停表判法与 ⑥ 翻回「开着」，拒绝说明还写「只经过术语改名、删日期才对不上的照收」 | 中（不收、⑥ 拒） | 不中（收、⑥ 放） | U3（批 11）把 P5 修到的这一形带回来 | 量过 `out/q4-date-once.txt` |
| Q4-乙（Z4-乙 的变形） | 打中（弱） | 〔〕在一个旧名的中间起头或收尾（「块的校验…」「…超级」），照 path-moves 跑 `sweep-term.py --apply` 之后不收 | 中 | 不中 | U3 | 量过 `out/q4-term-cut.txt` |
| Q4 其余（Z1 围栏四形、Z6-甲、Z1-乙三形；Z4 三种日常改动原形；Z5 两形；Z7 文字） | 没打中（批 11 之后还修着） | — | 各形 r1 中 | 不中 | — | 量过 `out/q4-z1-fence.txt`、`out/q4-z4-daily-changes.txt`、`out/q4-z5-cells-r3.txt`；Z7 文字与判法 grep / diff 量过 |
| Q1（S2） | 没打中钩子与骨架；文字缺口一条（Q1-甲） | 共用约束「实验装置自己的线程变量（KS_THREADS 这类）同样设成不超过 N」：子 agent 看到机器忙、照这一句把装置线程设小，照字面合规，用户定案是「机器忙时也固定 31」 | 同（r1 同一句） | 同 | 改前也中 | 钩子与骨架量过 `out/q1-skeletons.txt`；子 agent 一侧推的 |
| Q2（S4） | 没打中 | — | 九例与 r3 逐例相同 | — | — | 量过 `out/q2-selftest-copy.txt` |
| Q3（S6） | 没打中 | — | — | 八形与 r3 逐形相同 | — | 量过 `out/q3-mask-probe.txt`（函数级，没起格） |

## Q4　第二轮辩方判「修到」的条目，批 11 之后还修着没有

### Q4-甲　出处那一行的日期写在全角括号里，照 date-once 的出路删掉之后叫停又失效（量过，Z4-丙 改后又中）

被判的字面（r3）：
- 规则：`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r3/repo/.claude/rules/three-way-inference.md:121`：「（那一行只经过术语改名、删日期），取提示那一行」
- 库：`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r3/repo/.claude/hooks/lib_abandoned_rounds.py:241`：「只经过术语改名、删日期的那一行照收，转述与改了别的字的不收」；比的样子只去掉新旧名、日期、空白与逗号，`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r3/repo/.claude/hooks/lib_abandoned_rounds.py:237`：「return UNCOMPARED_CHARACTERS.sub("", removed.sub("", text))」；它自己的注释说照的是删日期那份脚本，`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r3/repo/.claude/hooks/lib_abandoned_rounds.py:200`：「删日期时它两边的空白与逗号跟着删」
- 删日期的那份脚本把括号连日期一起删：`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r3/repo/scripts/kb/history-consolidate.py:252`：「line = re.sub(r'（' + standalone + r'）', '', line)」
- date-once 格的出路：`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r3/repo/.claude/gate.d/doc-text.sh:533`：「看它删了哪些日期、并了哪些小节，照着改回仓里」
- 门禁格拒绝说明：`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r3/repo/.claude/gate.d/doc-process-records.sh:908`：「只经过术语改名、删日期才找不到的，行号写到那一行照收」

历史（喂法 `tools/q4_date_once.sh`，每一步对着许可它的那一句）：
1. 用户当天叫停一轮。主 agent 在当天的调度记录 `records/2026-10-14-样本调度.md` 记一行「- hx-paren-r1 用户叫停（2026-10-14）：……，两条腿当场停掉。」。日期写进正文是 date-once 格要抓的写法，与第一轮 Z4-丙 的历史同一个结构（那一次是「用户 2026-10-12 说」）；括号这种写法在 r3 的 `records/` 96 份里有 360 行（命令与输出见下）。
2. 照规则第 121 行在 `research/prompts/abandoned-rounds.tsv` 登记一行，出处 `records/2026-10-14-样本调度.md 第 6 行〔hx-paren-r1 用户叫停（2026-10-14）〕`，〔〕逐字照抄。叫停表判收，派实现员改那一轮快照里的文件，⑥ 放。
3. 提交前门禁 doc-text 的 date-once 格红，照它的出路跑 `doc-consolidate.py migrate` 并照着改回记录：那一行成了「- hx-paren-r1 用户叫停：……」（括号与日期一起没了；登记表在 `research/prompts/`，date-once 不管它，不动）。
4. 叫停表判法与 ⑥ 再判一次。

结果（r1、r2、r3 与我的试改各跑同一段历史）：
- r3 删之前：`research/prompts/m3-defs-b9-r3-opus-model/out/q4-date-once.txt:13`：「叫停表：hx-paren-r1 收 ⑥ 派实现员退出=0」
- r3 删之后：`research/prompts/m3-defs-b9-r3-opus-model/out/q4-date-once.txt:14`：「叫停表：hx-paren-r1 不收：第 9 行的出处 records/2026-10-14-样本调度.md 里找不到〔hx-paren-r1 用户叫停（2026-10-14）〕这段原话」；同一行「⑥ 派实现员退出=2」，说明末尾还写着「那一行只经过术语改名、删日期才对不上的照收」
- r2 删之后：`research/prompts/m3-defs-b9-r3-opus-model/out/q4-date-once.txt:8`：「叫停表：hx-paren-r1 收 ⑥ 派实现员退出=0」
- r1 删之后：`research/prompts/m3-defs-b9-r3-opus-model/out/q4-date-once.txt:2`：「不收：第 9 行的出处 records/2026-10-14-样本调度.md 里找不到〔hx-paren-r1 用户叫停（2026-10-14）〕这段原话」；同一行「⑥ 派实现员退出=2」
- 对照（日期不在括号里、或括号里还有别的字）r3 照收：`research/prompts/m3-defs-b9-r3-opus-model/out/q4-date-once.txt:16`：「叫停表：hx-plain-r1 收 ⑥ 派实现员退出=0」；`research/prompts/m3-defs-b9-r3-opus-model/out/q4-date-once.txt:18`：「叫停表：hx-comma-r1 收」
- r2 收的原因是 P5 只看提示那一行有没有这一轮的名字：`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r2/repo/.claude/hooks/lib_abandoned_rounds.py:232`：「if not hits and hint_usable and name_in(cited_lines[line_number - 1], round_name):」；U3 加了逐字比，比的时候没去掉删日期留下的那对空括号「（）」。

括号写法有多常见（r3 冻结副本的 `records/`，与我拷贝里的 `diff -rq` 无差）：

```
$ grep -cE '（20[0-9]{2}-[0-9]{2}-[0-9]{2}）' records/*.md | awk -F: '{s+=$2} END {print s}'
360
$ grep -hE '（20[0-9]{2}-[0-9]{2}-[0-9]{2}）' records/*.md | grep -E '[a-z0-9]-r[0-9]' | grep -c '「'
57
```

今天 `records/` 里括号日期正好等于文件名日期、又带轮名与「」的行我逐份查是 0 行（存量都已被 date-once 删过），所以这段历史要一行新记的叫停才走得到；写法本身是常见的那一种。

四句：
- 分不分辨臂：分。同一段历史 r2 收、r1 与 r3 不收，来源是 U3（批 11）。
- 看得到：看得到。判的那一刻〔〕与提示那一行都在手边，去掉新旧名、日期、空白与逗号之后只差一对空括号；删日期用的是仓里同一份 `history-consolidate.py`。
- 满足的分句：规则第 121 行「（那一行只经过术语改名、删日期），取提示那一行」——这一行只经过删日期，规则说收、库不收；不是转述也没改别的字。结局：用户已叫停的轮重新算开着，⑥ 挡写回员与实现员，门禁 abandoned-rounds 红，拒绝说明还告诉主 agent 这种情形照收。
- 改法：跑前条款没有点名改法；U3 是来源（这一格上 U3 让 P5 修到的又中）；我的 F-Q4a 在这一格上修到（量过，`research/prompts/m3-defs-b9-r3-opus-model/out/q4-date-once.txt:20`：「叫停表：hx-paren-r1 收 ⑥ 派实现员退出=0」），第一轮三种日常改动照收（`out/q4-z4-daily-changes.txt` 第 10–12 行），第二轮 S5-甲 的转述照旧不收（`research/prompts/m3-defs-b9-r3-opus-model/out/q4-s5-misread-under-fix.txt:2`：「fix 叫停表：m3-batch63-code-r1 不收」）。

跑前条款两条都对得上：r3 上中（「站不住」那一条的观测），r1 上同一段历史也中（「也打中 P 改法之前那一版」那一条）；r2 不中。按哪一条记由主 agent 定。

### Q4-乙　〔〕在一个旧名的中间起头或收尾，照 path-moves 改术语之后不收（量过，弱）

历史（喂法 `tools/q4_term_cut.sh`，小树里只有调度记录、登记表、r3 的改名表与豁免表）：调度记录那一行「- 超级块的校验没跑完：去向 hx-cutstart-r1 两条腿停掉，用户「……」。」，登记表〔〕从「块的校验没跑完」起抄（规则第 121 行只要「一段原话」，没要求从词的边上起）；之后照 `.claude/rules/path-moves.md` 跑 `sweep-term.py --apply`，记录那一行成了「系统配置的校验没跑完：……」，登记表在豁免目录里不动。另一形是〔〕收在「……超级」。

- r3：`research/prompts/m3-defs-b9-r3-opus-model/out/q4-term-cut.txt:9`：「改动后：hx-cutstart-r1 不收：第 1 行的出处 records/log.md 里找不到〔块的校验没跑完：去向 hx-cutstart-r1 两条腿停掉〕这段原话」；收尾那一形同样不收（第 8 行）。
- r2：`research/prompts/m3-defs-b9-r3-opus-model/out/q4-term-cut.txt:6`：「改动后：hx-cutstart-r1 收」（第 5 行收尾那一形同）。
- r1：两形都不收（第 2、3 行）。
- 对照：〔〕把旧名整个抄进去，r3 照收：`research/prompts/m3-defs-b9-r3-opus-model/out/q4-term-cut.txt:7`：「改动后：hx-whole-r1 收」。

四句：分辨臂——分（r2 收，r1、r3 不收，U3 带回来）；看得到——一半：〔〕里的「块」与提示那一行里的「系统配置」字面上连不起来，要知道「块」原属「超级块」得有改名之前那一版（git 里有，判的那一刻库不看）；分句——规则第 121 行「那一行只经过术语改名」，这一行只经过术语改名，规则说收、库不收；改法——F-Q4a 碰不到这一格（`out/q4-term-cut.txt` 第 11、12 行仍不收），F-Q4b（推的）碰得到。

弱在哪：多数人抄〔〕会从词的边上起止；可规则与拒绝说明都没要求，日常改动之后才露出来，主 agent 看到的说明还是「只经过术语改名……照收」。

### Q4 其余：批 11 之后还修着（没打中）

第二轮辩方表里判「修到」、Q4 点名的几组，我在 r3 上重喂了前两轮的原喂法（`tools/borrowed/`）：
- Z1 围栏四形（波浪线块里露出三个反引号、括注那一形、列表项里没闭合的围栏、HTML 注释）与 Z6-甲：r3 仍拒。`research/prompts/m3-defs-b9-r3-opus-model/out/q4-z1-fence.txt:2`：「T1-tilde-lone-fence-16 investigator r3=2 r2=2 r1=0」，第 3–5 行与第 13 行同；其中 T2、T3 在 r3 上改由 U1 的「两种认法不一致」拒（同一份输出里的首行）。Z1-乙 三形（O1–O3）r3 仍放（第 7–9 行）。
- Z4 三种日常改动原形（追加补记、术语改名、date-once 删「用户 2026-10-12 说」里的日期）：r3 仍收。`research/prompts/m3-defs-b9-r3-opus-model/out/q4-z4-daily-changes.txt:7`：「r3 excerpt self-quote 改动前：hx-quote-r1 收 改动后：hx-quote-r1 收」，第 8、9 行同；r1 三形不收（第 1–3 行）。
- Z5 两形（date-once 红行路径、sweep-term 输出路径；clock-times 片段半截）：r3 打码。`research/prompts/m3-defs-b9-r3-opus-model/out/q4-z5-cells-r3.txt:12`：「✗ records/〔文件名含私有值〕：正文里重复所属日期 1 处」；第 3 行片段「〔私有值〕 上 2026-10-13T<时分秒> 收尾」；r1 同一份在第 3 行打出半截（`out/q4-z5-cells-r1.txt`）。`doc-text.sh` 在 r2 与 r3 逐字节相同（sha256 前 8 位都是 b47e98f8，三份冻结副本各算一次）。
- Z7 文字：「基准那一版没有这张表时不比」在 r3 的 `.claude/gate.d/stage-owners.tsv` 第 48 行、`.claude/agent-common.md` 第 67 行、`.claude/main-agent.md` 第 55 行各命中 1 次（`grep -nF`）；`lib_long_run.py` r2 与 r3 无差，`code-tooling.sh` 的 diff 里没有一行碰长活豁免表（`diff … | grep -ci 'long-run\|exempt-check\|长活'` 输出 0）。

试过而没打中的 Q4 形状另列在「没打中的形状」。

## Q1　⑮ 缺行出路两种写法、`dispatch-prompt.py` 的 clerk 改名（S2）

### 钩子与骨架：没打中（量过）

喂法 `tools/q1_skeletons.sh`：`dispatch-prompt.py <类型>` 给 ⑮ 那九种类型与设计员各打一份骨架，照骨架写的填法填好（预计挂钟写 2 小时内、后三行删掉、占位填上），交 `dispatch-prompt.py --check`（起同一份派发闸）。九种骨架都带「线程上限：31」，除变异分诊员缺变异表那一条之外都放；clerk 那一份：`research/prompts/m3-defs-b9-r3-opus-model/out/q1-skeletons.txt:16`：「骨架填好-clerk clerk --check 退出=0」。两种出路各一份、加「要别的数只写已判行」：r3 三份都放（第 21–23 行），r1 同（第 44–46 行）。最后那一形钩子放、子 agent 照共用约束不起重活，是第二轮已立账的「单独已判行而理由写数」，不再交。

两行「线程上限：」（31 与 16）加已判行钩子放、以哪一行为准没写，第二轮辩方的 S2 矩阵已列（`research/prompts/m3-defs-b9-r2-sonnet-model/out/s2-exit-matrix.txt` 第 15 行），不再交。

### Q1-甲　共用约束让装置自己的线程变量「不超过 N」：照字面合规地按忙闲降（推的，改前也中）

- 共用约束：`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r3/repo/.claude/agent-common.md:61`：「实验装置自己的线程变量（KS_THREADS 这类）同样设成不超过 N。」
- 用户定案的登记：`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r3/repo/.claude/heavy-thread-default:6`：「机器忙时也固定 31、不看忙闲」；主 agent 定义：`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r3/repo/.claude/main-agent.md:53`：「机器忙时也固定这个数，不看忙闲」。

历史：派执行员跑一个只读 `KS_THREADS` 的装置，提示写「线程上限：31」（⑮ 放）。执行员开跑前照共用约束「不做」一节看负载（第 59 行那一条，重活开跑时 `host-resources-reminder.sh` 还会打各台此刻的空闲），看到别的会话的 `cargo` 占着一半核，照第 61 行「设成不超过 N」把 `KS_THREADS` 设成 16、`capped.sh 31` 照写。每一步都有字面许可；结局是这一趟按忙闲降了，违反「机器忙时也固定 31」。钩子目录里没有一处读 `KS_THREADS` 或「不超过 N」（`grep -rn 'KS_THREADS\|不超过 N' .claude/hooks/*.sh .claude/hooks/*.py` 输出 0 行），这一半只靠子 agent 自守。

四句：分辨臂——不分：r1 的共用约束第 61 行同一句（`grep -nF '同样设成不超过 N'` 命中第 61 行），r2、r3 同；看得到——派发闸看不到装置变量，子 agent 看得到负载；分句——「照定义做却违反用户定案」；改法——前两轮的改法都碰不到这一格，F-Q1a（推的）碰得到。按跑前条款「也打中 P 改法之前那一版」记，另立一笔账。

## Q2　仓副本里正文改过的自证按提交时的检查办（S4）：没打中（量过）

喂法 `tools/q2_selftest_copy.sh`：stdin JSON 喂各版的 `heavy-test-guard.sh`，工作根设成草稿目录里的一个目录，被起的是登记为 light 的 `scripts/gate/apply-writer-patch.py`，「改过正文」是副本那一份末尾加一行注释。九例 r3 与 r1 逐例相同（`out/q2-selftest-copy.txt` 第 1–9 行对第 10–18 行）：
- 子 agent：没改过的副本放；改过正文拒，自加 `=commit` 也拒（`research/prompts/m3-defs-b9-r3-opus-model/out/q2-selftest-copy.txt:3`：「r3 子agent-改过正文-自加commit前缀 退出=2」）。
- 主 agent：不带前缀拒（第 4 行）；带 `=commit` 放（`research/prompts/m3-defs-b9-r3-opus-model/out/q2-selftest-copy.txt:5`：「r3 主agent-改过正文-带commit 退出=0」）；带 `=user-request`、副本在工作根外面、副本里的新脚本没登记，带前缀都放（第 6、7、9 行）。

规则第 54 行那一格、重型那一条「提交之外先弹窗」与「收回放宽」三句在这九例上没有一例撞：子 agent 一律不跑，主 agent 不带前缀不跑，带前缀跑的是提交时的检查那一类、不是重型。还剩的两处都是前两轮与上一题已交的：钩子不读副本正文，改成起重活的「light」自证主 agent 带 `=commit` 照放（第二轮辩方 S4 专问 B1）；入口不变、改了它 import 的库，子 agent 在副本里照跑（上一题 V5-甲）。不再交。

## Q3　doc-text 打码（S6）：没打中（量过，函数级）

喂法 `tools/q3_mask_probe.py`：从 `doc-text.sh` 抽出打码那一份共用函数原样 exec，配置与主机名都是编的，按三条通道的调法喂八形。r3 与 r2 八形都没有整值、没有 6 字以上的半截（`out/q3-mask-probe.txt` 第 3–10 行与第 13–20 行），例：`research/prompts/m3-defs-b9-r3-opus-model/out/q3-mask-probe.txt:3`：「路径里值带空格（引号括起的仓路径）」，同一行「整值 []；半截（≥6 字）[]」。八形是：值带空格（引号括起的仓路径）在 sweep-term 输出与片段里各一；行内锚里带斜杠与值；全角冒号粘着值；同一条路径里值出现两次；本机主机名的短名在文件名里；两个值紧挨着；值比片段长、两头都切在值中间。第二轮攻方的两形（两个值首尾重叠、换大小写）与 doc-text 之外不打码是已立账，没重交。

## 我提的改法（只在我的模型上量过、被攻过零轮）

| 改法 | 改什么 | 修的格 | 在打中的格上 | 在别的格上 |
|---|---|---|---|---|
| F-Q4a | `lib_abandoned_rounds.py` 的 `compared_form` 去掉新旧名与日期之后，再去掉剩下的空全角括号「（）」（试改树 `tools/make_fix_tree.sh`，一行）；更稳的写法是比之前对〔〕照记录的文件名日期与小节日期跑一遍 `history-consolidate.py` 的 `strip_same_date`，删日期的写法以后改了也跟着走 | Q4-甲 | 量过：括号那一形收、⑥ 放（`out/q4-date-once.txt` 第 20 行）；更稳的写法推的 | 量过：第一轮三种日常改动照收（`out/q4-z4-daily-changes.txt` 第 10–12 行），第二轮 S5-甲 的转述照旧不收（`out/q4-s5-misread-under-fix.txt` 第 2 行）；Q4-乙 两形照旧不收（`out/q4-term-cut.txt` 第 11、12 行）——碰不到 Q4-乙 |
| F-Q4b | 原话在现在的记录里找不到时，到登记那一行进 git 的那次提交（`git log -S` 找登记表加这一行的提交）里那一版记录去找〔〕，找到了再按 `git diff` 把那一行映到今天的行号，取映过来的那一行（第二轮攻方 F-S5a 的后一半） | Q4-乙；也罩得住 Q4-甲 | 推的 | 推的：碰不到登记时还没提交的记录；git 不在时退回现判法 |
| F-Q1a | 共用约束第 61 行「同样设成不超过 N」改成「同样设成 N，不按忙闲降；要别的数照「线程上限已判」那一条由主 agent 在提示里写」 | Q1-甲 | 推的（文字） | 推的：只改文字，钩子不判，同前两轮的文字改法一样靠子 agent 自守 |

## 顺手看到（只记一行，不判）

- U3（这一轮不开攻）：Q4-甲、Q4-乙 就是 U3 的逐字比带回来的，已在上面按 Q4 交。
- 不在 Q1–Q4 被判文件里：`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r3/repo/scripts/crash/crash-amplification-two-hosts.sh:40`：「里第二台此刻的可用线程（至少 1）」——文件头说第二台线程现算，库已改成取登记值（`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r3/repo/scripts/crash/crash-amplification-lib.sh:208`：「heavy_thread_default」），同一份的自证格 `peer-threads-from-host-resources`（第 733 行）还按现算期望 5；推的，没跑那份自证。r1 与 r3 这两份逐字节相同。

## 要主 agent 跑的命令

门禁格子 agent 起不了。造树脚本与逐条命令在 `research/prompts/m3-defs-b9-r3-opus-model/gate-samples/`（`make_trees.sh <冻结副本根> <新的空目录>`，`要主agent跑.txt` 写了三份冻结副本各一遍）：
- abandoned-rounds 格（Q4-甲）：`bash <G>/.claude/gate.d/doc-process-records.sh --check abandoned-rounds <T>/abandoned-paren-after-migrate`。预计 r3 退 1、点名 hx-paren-r1「找不到〔hx-paren-r1 用户叫停（2026-10-14）〕这段原话」；r2 退 0、成功句另列照行号提示取的 1 行；r1 退 1。对照树 `abandoned-paren-before-migrate` 三版都预计退 0。
- 预计来自格与 ⑥ 共用的 `lib_abandoned_rounds.py` 命令行在本腿树上的输出（`out/q4-date-once.txt`），没起过格本身。
- 这一腿没跑崩溃枚举与原型扫描，状态数 0；没有轻型三条要交。

## 没打中的形状

- Q1：九种类型与设计员的骨架（`out/q1-skeletons.txt`）；两种出路与「要别的数只写已判行」；前两轮的围栏各形（见 Q4）。⑮ 那九种之外的类型（设计员、写回员、sweep、prior-art、材料腿）逐份看了定义，设计员只写登记、别的不编译不跑测试，缺线程上限一行不落空 31（读定义，推的）。`capped.sh`、`heavy_threads.py`、`heavy-threads.sh`、`run-with-memory-cap.sh` 里没有按忙闲改线程数的写法（对这四份 `grep -n 'nproc\|loadavg\|available_threads'` 出 5 行：`capped.sh`、`heavy-threads.sh` 的三行在弄坏开关那一支，`run-with-memory-cap.sh` 的两行只是打「未设（整机 N 核）」这句说明）。别的会话明说让出 CPU（上一题之前的 b3 第一轮已交）、两行线程上限（第二轮辩方已交）没重交。
- Q2：九例（`out/q2-selftest-copy.txt`）。没试：门禁样本档（`stage-selftest.sh --item`）在副本里改过阶段正文的那一支，第 54 行那一格只写脚本的自证，样本档走的是另一支判法。
- Q3：八形（`out/q3-mask-probe.txt`）。集成副本根下有没有 `multi-host.env` 我只用 `ls` 看了在不在（在，没读值），所以第二轮攻方推的「仓副本里没有配置就原样打」在集成副本上不成立。
- Q4：第一轮的围栏、三种日常改动、doc-text 两形、Z7 文字都还修着（见 Q4 其余）。没交的两形：照 prime-marks 的出路给变体改名（登记表也在 prime-marks 扫的范围里，两边会一起改，推的、没造树）；改名表里两行接连替换出第三个词（一行的新名后面紧跟着另一行正则认的旧名，要刻意凑；按 `sweep-term.py` 逐行替换与库里一次去掉的差别推 r3 不收，没跑）；轮名自己含旧名、改名之后记录里的轮名变了而登记表没变（r1、r2、r3 按代码推都不收，改前也中，没造树）。

## 这条腿自己的限度

- Q1-甲 子 agent 那一侧（看到负载就照「不超过 N」降）是推的，没派真 agent；钩子那一侧量过（不判装置变量）。
- 门禁格本身没起；abandoned-rounds 格与 ⑥ 共用的 `lib_abandoned_rounds.py` 命令行、派发闸的 `--check-prompt` 是同一份判法，格的预计写在「要主 agent 跑的命令」。
- Q4-甲 那段历史要一行新记的、当天日期写进括号的叫停行；`records/` 里今天没有现成的这种行（存量已被 date-once 删过）。
- Q4-乙 的〔〕在词中间起止是人挑的抄法，规则不禁也不常见，所以标「弱」。
- F-Q4a 只在本腿树上跑了上面列的几组喂法（日期、术语、三种日常改动、第二轮 S5-甲），没跑库自己的自证、两份门禁样本档与上一题的样本。
- Q3 是函数级（抽出共用函数喂编的配置），没起 doc-text 本身。
- 开工看负载：别的会话有 checker 档的覆盖率编译与 `cargo clippy` 在跑，没有 `qemu-system`、`vm-bench`、`e152`、`fio`；本腿只起 python 与 bash 判法，都加 `nice -n 19`，没编译，线程上限 31 没用上。复跑经内存包装一趟约 70 秒挂钟。
- 读了什么：定义点名的 `.claude/agent-common.md`、`.claude/rules/three-way-inference.md`、`.claude/singlefs-ai-sop/rules/evidence-discipline.md` 读的是主工作区那一份（共用约束与三方规则也是被判文件，与禁读清单「主工作区里被判的…字面」相抵，照定义读了；`three-way-attack.md` 作为本腿系统提示进来）；报告里被判的字面一律取自冻结副本。没读禁读清单里的 sonnet 第三轮报告、模型目录与草稿目录；冻结副本根下 `multi-host.env` 拷贝时排除了，值没读；集成副本那一份只 `ls` 看了在不在。

## 没做什么

- 没起门禁格（`--check`）、没跑任何登记的 `--selftest`（`dispatch-prompt.py` 只打骨架与 `--check`）、没跑 `gate.sh`、没编译、没跑重型测试与轻型三条。
- 没判 Q5、Q6（正推腿的格），没判 U1–U4 本身，没替主 agent 采纳；副本上量出的数不算入库装置上的数。
- 草稿目录 `~/.local/share/singlefs/work/m3-defs-b9-r3-opus/` 里的三份冻结副本拷贝（`t-r1`、`t-r2`、`t-r3`）、试改树 `fix-r3`、各趟复跑目录与喂法的小树交回前删掉；留下 `prompt.md` 与认领标记。

## 交回前核

`python3 scripts/agents/cite-check.py research/prompts/m3-defs-b9-r3-opus-output.md --root /home/fy5090/code/singlefs` 的末行：

```
  ✓ 核了 30 处引文、0 处裸行号，对上 30 处，没判 0 处（其中裸行号没有引文可比 0 处；逐处列在上面）
```
