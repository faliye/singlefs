# 写回员 d 报告（batch-2026-10-21 阶段同步，实验与验证落地）

## 规格与规格检查
- 规格：`/home/fy5090/.local/share/singlefs/work/batch-2026-10-21-writeback-d/spec.json`（sha256 `be6b5c995c15799189a62e8e3448d3d7abc88bebf587d6c36c7ab5638d9ebe40`）与同内容 `spec.md`（sha256 `96a37b353a22643a33392b01b28ea0744b2690934d079c1ce3c4a87d4c0f6b21`），124 条，生成脚本 `make_all2.py`。
- `kb-spec-check.py` 原样末行（spec.json、spec.md 各跑一次同）：`  ✓ 规格 124 条：旧串各恰好 1 次，编号引用、指代、依据配对与变更史形态都过（查了 124 条）`
- 写范围闸预检：规格里 18 个文件各喂一次 write-guard，18 个退出码都是 0（记录指到草稿目录 precheck.jsonl）。
- `replace-batch.py --dry-run`：`✓ 124 处都命中得对（18 个文件），--dry-run 没写`；实写：`✓ 124 处替换，写了 18 个文件（改名换上新 inode），回读一致`，退出码 0（原样输出在 replace-batch.out）。
- 没跑 `doc-decisions.sh --write`、`relabel-item.py`：这一份没有决策正文与分项状态改动（派发提示也写了并行写回各自不跑）。decisions.md、decisions-history.md 前后 sha256 相同。

## 改了什么（行号为写完之后的行号）
| 行 | 文件 | 改前一句 | 改后一句 |
|---|---|---|---|
| F5 | `experiments/113-*.md:126`、`.en.md:126`、`.ja.md:126` | 落点 C693 还当记账维待定 | 句尾补「C693 已还清，用户 2026-10-20 定「维持搬运，㊀ 不改」（D8 已定项 8 ㊀），记账那一维取 `Carry`」；英、日同句，译文里不写编号引用（见下「要主 agent 知道的点」第 1 条） |
| M1 | `experiments.md:95`、`.en.md:95`、`.ja.md:95` | 状态「已跑（2026-09-01）」 | 状态加「；2026-10-20 第 1 次重跑」，结论格补「一代口径下上界臂 45 格里 5 格过界（旧口径 9 格）、窄读 0 格」；71 页标题（zh/en/ja 第 1 行）同补 |
| F46 | `experiments.md:35`、`.en.md:35`、`.ja.md:35`；`experiments/10-*.md/.en.md/.ja.md` 第 1 行 | 未跑 / 文件系统已有，仍欠后台整理 | 部分已跑（第一段 21 片跑了 4 片，读数只当线索；剩 17 片停下，登记的三处要不要修订待定）；索引说明格换成装置、分片驱动、21 行变异表已入库与待定三处 |
| F48 | `experiments/175-*.md:11` | 装置源码原样留在产物目录的 `device-source.rs` | 装置源码原样放在 `scripts/experiment-apparatus/e175_product_path_inode_watermark_regression/device-source.rs`（不在产物目录里） |
| F58 | `experiments/142-*.md:22` | 两态闭式 16777240，补第三态后全量 16777260，全量由 crates 枚举 | 补「这是装置按取号写一轮量的」，加「取号写两轮之后 crates 里枚举的是 61 次操作、全量 16777268，本页装置没按两轮重跑」 |
| F58 | `experiments/142-*.md:310` | 今天的新池新建文件流是 12 单元、两态闭式 16777240、补第三态全量 16777260 | 拆成「本页装置量到的……（取号写一轮）」与「今天 crates 里的流取号写两轮，61 次操作、全量 16777268」 |
| F33 | `verification-build.md:71`、`.en.md:63`、`.ja.md:63` | 整理还没进代码 | 整理已进 core 代码但产品路径在批大小条款没定处停下（C146 ① 仍开着） |
| F59 | `verification-build.md:6、7`、`.en.md:6、7`、`.ja.md:6、7` | 14 条、16 777 260、16 777 240、1 678 377 204 | 35 条、16 777 268、16 777 243、1 678 377 220（三语同） |
| F60 | `verification-build.md:6`、`.en.md:6`、`.ja.md:6` | replay-prefix 一句只写恢复核与它的 Verus 证明 | 补「另有 `compaction_plan_proofs.rs` 证整理计划的选段健全、计划健全、进展与抽象恒等」 |

## 实验页回看与变更史（规则 3）
- 回看重新回看，日期 2026-10-21：113 页 4 行 ×3 语、71 页 6 行 ×3 语、10 页 2 行 ×3 语、175 页 2 行、142 页 42 行（原「页首只加了三语切换行」那句并进新句）。113、71、10、175 是在原回看前面加一段新句，加「此前」接原文；142 是合并改写。英、日表的回看格原来没有 2026-10-21 那段，这次只加我这一段，没替它们补别的。
- `experiments-history.md` 加 `### 2026-10-21` 块各一项：E10（:402 起）、E71（:2438 起）、E113（:3722 起）、E142（:4592 起）、E175（:5626 起）。改前写 `git show 841b3beab:<路径>`。experiments-history 只有中文一份，没有译文。
- 加块的位置都在各自节的第一个日期块之前；每节原来没有 2026-10-21 块。

## 现查（HEAD 加已暂存）
- `git show HEAD:crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs`：`FULL_STATES_WITH_TWO_STATES_PER_WRITE: u64 = 16_777_243`（:757）、`FULL_STATES: u64 = 16_777_268`（:761）；`crash_enumeration_fixed_script_stream.rs`：`FULL_STATES_THROUGH_THE_UNMOUNT: u64 = 1_678_377_220`（:107）。
- `grep -c '^crash-case:' .claude/gate.d/stage-inputs.tsv`：工作区 35，暂存区 35，HEAD 34。
- `git show HEAD:crates/singlefs-core/src/transaction.rs` 有 `pub fn publish_relocation_batch`（:9125）；`mounted_session.rs` 有 `pub fn run_compaction`（:1938）与 `CompactionRunStop::BatchSizeNotDecidedByTheClauses`（:1934、:2642）。
- `git ls-tree -r HEAD --name-only crates/singlefs-replay-prefix`：有 `src/compaction_plan_proofs.rs`；HEAD 的 `.claude/gate.d/verus-pins.tsv` 第 28、29 行登记 lib.rs 与它。
- E10：HEAD 有 `research/prompts/e10-r1-prereg.md`、`scripts/experiment-apparatus/e10_aging_compaction.rs`、`scripts/experiment-apparatus/e10-aging-compaction-run.sh`、`scripts/mutation/tables/e10_aging_compaction.tsv`（`wc -l` 21 行）、`research/results/e10-aging-compaction-2026-10-13/` 下 4 份（disk_1gib-s4/s8/s16、main-s4）。
- E175：`scripts/experiment-apparatus/e175_product_path_inode_watermark_regression/device-source.rs` 在 HEAD；`research/results/e175-product-path-inode-watermark-regression-2026-10-08/` 下只有 `d-P.out`。
- E71 的数：71 页「第 1 次重跑」一节的 `over_budget_count` 行（上界臂 45 格 old_over=9 new_over=5，窄读 new_over=0）；产物在 HEAD。
- C693 已在 `.claude/kb/checks-owed.md` 已还清表（HEAD :863），③ 写「D8 已定项 8 ㊀ 维持搬运（用户 2026-10-20 定「维持搬运，㊀ 不改」）」。

## 剔出的行
无。16 行的依据在 HEAD 与已暂存的门禁、脚本改动里都找得到，没有只在工作区的。

## 没改的行 / 与行清单不同的处理
- F46 行里「21 行变异表（抓到 20、无效 1）」：records 只记了「变异表 21 行」与「M8 无效」，「抓到 20」没有出处，没写；改写成「21 行变异表已入库」。
- F46 行里「登记的模型与判据要修订三处」：records（2026-10-13-里程碑四预开发.md 的 E10 调查员交回）写的是调查员「交判」要不要修订三处（第 3 行判据、Y1 定义、P1 期望），接手会话定；写成「要不要修订待定」，不写成已定要修订。
- E10 标题与索引状态用「部分已跑」而不是「在跑」：`doc-experiments.sh` 的 index-sync 只认「未跑」「部分已跑」「已跑（够判）」三个状态词，正文与索引互相对得上。

## 要主 agent 知道的点
1. kb-spec-check 的编号引用检查对译文（.en.md、.ja.md）也要求写中文简称，而 trilingual 规则要求译文写半角括号译名，两者对不上。所以 113 页英、日句尾与 113 回看英、日前缀里没写 C693、D8 的编号，写成「that owed check is cleared」「その未済の検査は完済」；中文那句带编号与简称。译文没有丢中文新增的 D8 已定项 8 ㊀ 指针，主 agent 觉得要补，另行补。
2. 同类未判行：`verification-build.md:130`（及 `.en.md`、`.ja.md` 的对应行）写「新池新建文件……状态数是 16 777 260（`FULL_STATES`；每次写只取两态是 16 777 240……）」，与 F59 同一件事、同一批依据，已过时；它不在候选表和行清单里，我没改。
3. 142 页第 1 行标题、:202、:238、:542 也写「三态 16777260 与 `crates/` 钉的相同」，是那几次跑当时的叙述（事件句），没动。
4. E10 页两行回看写「不受影响」：对 D3 已定项 14、D10 已定项 1 的备料关系与结论没变；D10 决策文件第 19 行「而它未跑」另有写回员改，若那里改成「改了」，E10 页回看的理由要配合。
5. 按 `.claude/rules/format-evolution.md` 「回看」那段，142、175 两页没有 en、ja 译文，只改了中文。
6. 其余写回员并行：`experiments.md`、`experiments-history.md`、`verification-build.*` 里有别的会话没提交的改动（experiments.md 的 E187 行、experiments-history.md 与 verification-build.* 的暂存改动），我的改动按内容定点，没碰它们。

## 前后 sha256
开工时（`sha-start.txt`）与收尾时（`sha-end.txt`）都在草稿目录；改过的 18 个文件：experiments/10-老化结构抗老化vs事后整理.{md,en.md,ja.md}、experiments/113-不认识的树-补齐臂集.{md,en.md,ja.md}、experiments/71-记账key的条目数与宽度.{md,en.md,ja.md}、experiments/142-新池新建文件的干跑.md、experiments/175-产品路径上inode号水位退回与重发号.md、experiments.{md,en.md,ja.md}、experiments-history.md、verification-build.{md,en.md,ja.md}（都在 .claude/kb/ 下）。decisions.md、decisions-history.md 前后相同（`9abb9da05230…`、`f3c573715b90…`）。

## 没做什么
- 没判实验撑不撑决策分项，回看一律写「不受影响」加理由，没有写「改了」。
- 没跑任何门禁阶段、`--write`、`relabel-item.py`，没提交。
- 没改 `verification-build` 第 130 行等不在行清单里的行；没改 `.claude/kb/decisions/` 下的任何文件。
