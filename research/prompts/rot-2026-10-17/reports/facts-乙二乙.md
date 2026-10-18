# 事实表（乙二乙片：E156–E164 实验页）

- 事实表：/home/fy5090/.local/share/singlefs/work/rot-2026-10-17/agents/乙二乙/facts-乙二乙.tsv，20 行事实（F300–F319，不含表头；`grep -c . 文件` 得 21 行含表头）
- 候选表：/home/fy5090/.local/share/singlefs/work/rot-2026-10-17/agents/乙二乙/candidates-乙二乙.tsv，151 行（150 个不同的行，不含表头；`wc -l` 152）
- 没建副本、没起编译，没跑 `--changes`。
- 认领：`work-directory-ownership.sh claim` 报「不在工作根的下一层」被拒（草稿目录是 rot-2026-10-17/agents/乙二乙，在工作根下第二层）；草稿目录里已有派发闸建的 `.writer-lock`，我只往里写了事实表、候选表和自己的生成脚本 `make_facts.py`、`check.out`。

## 两条命令（原样）

`python3 scripts/kb/stale-candidates.py --check-facts <facts-乙二乙.tsv> --base ce1457c3`，退出码 6，输出末两行：

```
  ✗ 事实表没罩全：67 条变更记录没罩、0 行正则坏了、0 行检索词在基准零命中、0 行在结束零命中、0 行检索词太宽、0 行多余的 &&、0 行冒充新立、0 行冒充只改措辞、0 行宽词没抽查
     → 怎么办：（提示句，略）
```

红只剩「67 条变更记录没罩」：我这一片的变更清单顶上那张 H 编号表是空的（这 9 页在 `## 历史版本` 里这个阶段没新加变更记录；`grep '^| H' changes.md | grep -E 'experiments/1(5[6-9]|6[0-4])-'` 无输出），67 条全是别片的。其余八项全是 0。

`python3 scripts/kb/stale-candidates.py --facts <facts-乙二乙.tsv> --base ce1457c3 --out <candidates-乙二乙.tsv>`，输出末行：

```
  ✓ 候选表 /home/fy5090/.local/share/singlefs/work/rot-2026-10-17/agents/乙二乙/candidates-乙二乙.tsv：151 行（150 个不同的行）
```

## 每个检索词在结束那一版（工作区）的命中行数

没有任何一个超过 100 行，所以没有宽词抽查；`&&` 一律没用（第一个词自己都不超过 500 行，门禁判多余），需要收窄的用单个正则里的就近匹配（`E159.{0,300}(311|点名项)|…`）。

| 行 | 检索词 | 基准命中 | 结束命中（候选行数） |
|---|---|---|---|
| F300 | `整行抄自产物` | 74 | 76 |
| F301 | `哪一?份承重` | 13 | 7 |
| F302 | `M44` | 5 | 5 |
| F303 | `18 passed\|19 passed` | 5 | 5 |
| F304（新立） | `C776` | 0 | 7（新立行不出候选） |
| F305 | `E159.{0,300}(311\|点名项)\|(311\|点名项).{0,300}E159` | 2 | 9 |
| F306 | `memory-peaks` | 11 | 15 |
| F307 | `probe_crash_states` | 1 | 1 |
| F308 | `E164.{0,300}(D13\|验证路线)\|(D13\|验证路线).{0,300}E164` | 5 | 6 |
| F309 | `W5.{0,40}省不下` | 2 | 3 |
| F310 | `E161.{0,300}(D13\|验证路线)\|(D13\|验证路线).{0,300}E161` | 2 | 3 |
| F311 | `Layer0StatePlan\|layer0_state_count\|Layer0Tally` | 7 | 6 |
| F312 | `e157-preregistration` | 4 | 3 |
| F313 | `\.\./mutate\.sh\|\.\./run-with-memory-cap\.sh\|\.\./capped\.sh` | 23 | 12 |

F314–F319 是「只改措辞」行，不出候选。

## 事实表里每件事的来由（差异片段来自 `git show ce1457c3:路径` 与工作区逐段比对）

- F300 五页（E156、E158、E159、E161、E164）把整行贴的 `E7RESULT name=…` 产物行换成「产物行见 git show <提交>:<产物路径> 第 N 行」指针；各页「影响的决策」回看相应写 2026-10-15「整行抄的产物行换成指向产物的指针，数、结论与产物不变」。真变了的是页内不再带产物字段，所以不算只改措辞；候选里是别处仍说「页内整行抄」的现状句。
- F301 E156 第 167 行：哪份产物承重由「主 agent 定（登记 S14）」改成「主 agent 已认定为第 5 次产物」。
- F302 E156 第 177 行：M35 重写 / M44 追加由「要换、要追加」改成「已打进 crates/mutations.tsv」（该表带 e156 的 27 行基准与现在相同，`grep -c 'e156_allocation_basis_counts' crates/mutations.tsv` 两版都得 27）。
- F303 E156 第 177 行：`test result:` 核对口径 18 passed → 19 passed（第 6 次含新 U20）。
- F304 新立 C776（E155、E159 装置停在头 311 与点名项 67）：基准零命中（`git grep C776 ce1457c3` 无输出）；E155、E159 页首状态句与 experiments.md 索引行点名它。
- F305 E159 装置：原来页内回看写「记录头 307 → 311 重跑，产物与 307 下逐字节相同」，现在写装置仍是 311 / 67、没跟着 journal 头 487 / 点名项 64 重跑。
- F306 内存峰值表引用从 `scripts/hosts/memory-peaks.tsv 第 8034 行` 改成 `~/.local/share/singlefs/memory-peaks.tsv`（`git ls-tree ce1457c3 scripts/hosts/memory-peaks.tsv` 空，`ls ~/.local/share/singlefs/memory-peaks.tsv` 有；表本身从未入库）。
- F307 E164 探针 `probe_crash_states.rs`：基准的 `git ls-tree` 里有，工作区 `ls scripts/experiments/e164_crash_amplification_design_function_test/` 没有，页内改写为「已从装置目录删掉，要接再从 git 历史取」。
- F308 E164 影响的决策：基准三行备料（已定项 4、9、15），现在四行（加已定项 17 支撑，写「改了：F4 量到……函数单元连同引用的常量、类型、静态项与宏一起算」）；已定项 9 原写「要写回的设计还没写进 kb」不再出现。
- F309 E164 W5：已定项 17 那一行把「崩溃枚举节点省不下」限定成「只在按 crate 闭包下量过，按函数单元键没量」。
- F310 E161 影响的决策：基准三行（已定项 4、7 备料，D24 不影响），现在四行（加已定项 16 支撑）。
- F311 E161 页里 crates 符号 `Layer0StatePlan`、`layer0_state_count_with_torn_in_place_overwrites`、`Layer0Tally` 改成 `CrashStatePlan`、`crash_state_count_with_torn_in_place_overwrites`、`CrashPointReplayTally`，落点行号随之换。
- F312 E157 页里 `research/prompts/e157-preregistration.md:688–700 / :656–686` 改成「跑前登记已在 31128273 归档」（`ls research/prompts/e157-preregistration.md` 无此文件）。
- F313 `bash ../mutate.sh`、`../run-with-memory-cap.sh`、`../capped.sh` 改成 `../mutation/mutate.sh`、`../hosts/run-with-memory-cap.sh`、`../hosts/capped.sh`（E157、E159、E160、E162、E163 页）。

## 判为「只改措辞」的载体清单

- `.claude/kb/experiments/161-崩溃放量的去重与分段耗时.md`：「第一条流」「第二条流」→「新池新建文件流」「覆盖写固定脚本流」（F314）
- `.claude/kb/experiments/158-择根与修复四岔路.md`、`161-…md`：「层 0」→「崩溃点重放」（F315）
- 156、157、158、159、160、162、163、164 号页：「今天」「现在」「目前」「此前」改日期或删，「本实验」「本文件」「上」「下」「前者」「后者」改成名字（F316）
- 156、157、158、161、162 号页：行号引用换成锚串，随代码后移的行号跟着改（F317）
- 156 至 164 号页「影响的决策」回看列：叠着的历次回看收成一条（F318）；回看列规则本身没改（`git diff ce1457c3 -- .claude/rules/format-evolution.md` 对回看一节无差异）
- 156 号页：doc-lint:not-numbers 追加 M36 M42、括注删掉、页内局部编号大小写对齐（F319）

## 要主 agent 现查（不在事实表形态里）

1. E164 页第 158 行 W5 一行仍写「在观测到的历史上坐实：20 个提交全部 8/8 重跑」，没带第 197 行新加的限定（按 crate 闭包下量过、函数单元键没量）；`experiments.md` 第 186 行 E164 索引行同样没带限定，也没提已定项 17 支撑。F309 的候选会列到，但这两处值得整体看。
2. E156 页第 169 行「新单测 U19 与整份装置的 18 条单测」是第 5 次的事件句，不用改；但E156 页首与 `experiments.md` 第 178 行「17 单测」「22 条变异」、E156 页第 187 行 `17 passed` 与 E156 页第 177 行 19 passed 并存，页内口径是否各自对应一次重跑，我没逐个核。
3. E159 第 42 行 / E156 多处的指针 `git show 5d0b7e7c1667:research/results/…out 第 N 行`：我没有逐条核对提交 5d0b7e7c1667 里该产物的第 N 行就是被替换掉的那行。
4. E164 页第 183 行指向 `git show dbcc0d2f:research/e7-index-bench/src/bin/…/probe_crash_states.rs`：只核了 `scripts/experiments/e164_*/probe_crash_states.rs` 在基准有、工作区没有，没核 dbcc0d2f 里这个路径存在。
5. 认领命令的路径写法：派发提示给的草稿目录在工作根下第二层，`work-directory-ownership.sh claim` 只认工作根下一层，提示里的命令照写会被拒。

## 没做什么

- 没逐行判候选（本任务只写事实表）；没核别片的 H 编号；没读 `research/prompts/`；没改任何仓内文件。
- 变更清单切片是逐字符差异，读不全的地方我另用词级比对工作区与基准（脚本在会话暂存目录）重读了 9 页的全部差异；E7RESULT 指针行（几百行）按同类看，没逐行核。
