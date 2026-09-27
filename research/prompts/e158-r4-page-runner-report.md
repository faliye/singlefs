# E158 第 4 次跑写实验页：执行员报告（接续撞限额的前一个执行员，2026-09-27 JST 14:3x–15:2x）

## 一、结论

- 接手时核现场：前一个执行员起的五段合并 `r4-compare`（后台任务 `be4wknxee`）已跑完，rc=0；产物 `research/results/e158-root-choice-repair-2026-09-27-r4-compare.out` 1832 行、末行 `E7RESULT name=done emitted=1831`、首行指纹 `ad9f5442…`。它没写过任何仓内文件（交接摘要第三节 0 个 Write / Edit）；实验页、`replay.sh`、`experiments-history.md`、索引里 E158 那几处的未提交改动都是主 agent 与别的会话的，没有写了一半的。没有重起 compare。
- 写完的：
  - 实验页 `.claude/kb/experiments/158-择根与修复四岔路.md`：页首新加「2026-09-27：第 4 次跑」一节（岔路表三行、装置与口径、86 份产物逐个点名、丢写、H-随 两栏、seg2 与 seg2b、多拒、多读、Q6、阳性对照与 V1 作废、F / S 条款、判决行、它答不了的、单测与变异、复跑）与「2026-09-27：第 3 次跑第一段」一节（只当参考，点名 r3 的 10 份与 `…-2026-09-27-r2-all-today.out`）；标题改成「第 4 次跑已交（岔路 1、2 用户已定，3 记欠）」；「2026-09-26：第 2 次跑第一段」一节开头那句改成「第 2 次跑第一段的记录」；影响的决策表五行都写了 2026-09-27 的回看（D23（journal 的角色与格式） 已定项 14 那一行补产物指针，另四行 `grep -c` 得 0、仍是备料）；页末历史加 2026-09-27 一条。
  - 索引行（`.claude/kb/experiments.md` 的 E158 那一行）重写成新状态与一句结论；`.claude/kb/experiments-history.md` 加一条。
  - `research/scripts/replay.sh` 加 `E158|@driver_e158_r4_compare||e158-root-choice-repair-2026-09-27-r4-compare.out|exact` 与 `driver_e158_r4_compare` 函数（登记表第 209 行，函数在 `driver_e158_r3_seg1_compare` 之后）。
- `replay.sh E158` 里我那一行：**pass，字节一致**（`/tmp/singlefs-replay-2773721/E158.18.line`：`E158  @driver_e158_r4_compare  字节一致 e158-root-choice-repair-2026-09-27-r4-compare.out`）。复跑用的是工作树的 bin（sha256 `5720ed09…`，不是生成产物的 seg2b 今天那一臂 `e71224fc…` 那一版），compare 的输出照样逐字节相同。
- 门禁：75、84、85、88、99、33、80、96 绿；40 号只剩两份 `gate69-refresh-replay-…-2026-09-25.log`（不是 E158），E158 的产物全被点名了；27、34、52、69、86 红在 E142、E157、E161–E163 与 `research/prompts/` 下别人的文件上，一处都不是这一次写的。doc-lint 红在 `162-崩溃放量判定块存储选型.md`，我写的三份不红。
- 推翻条件：五段合并的 compare 在复跑时与入库那份不逐字节相同；或页上抄的 `E7RESULT` 行在它里面找不到（88 号判的就是这个，现在绿）。

## 二、岔路表（岔路单 `research/prompts/c554-fix-forks.md` 三行；页上同名表在实验页第 23 行那一节）

行号都指 `research/results/e158-root-choice-repair-2026-09-27-r4-compare.out`（下称合并 compare）。

| # | 状态 | 已够判 / 还差什么 | 登记里剩下的量能不能让它翻面 | 算它的命令 |
|---|---|---|---|---|
| 1（乙，用户已定） | 已够判（用户已定） | 丢写：今天、甲-槽、乙-槽、乙-窄读-槽 全族 > 0；-配置 四臂 H1g 各 16 / 32；-配置续 四臂 H1f、H1g、实七-甲、实七-乙、实八 都是 0，H-随 照字面 6 / 37（全是 ③c，不算 ③c 2、Q1 只算环内且不算 ③c 1）。多拒 L 族：甲 / 乙 / 乙-窄读 各 22（其中 L5 里 3 格为罩第 22 条），丙 16，-槽 19；第二段五族每臂 0。同成多读（L 族）乙-配置 +3 / +4、乙-配置续 +7 / +8。Q6：今天 3 / 72 没罩住，候选各臂 0（甲 三臂作废） | 不能：五段都跑完了，没有登记里剩下的量；能改它的只有主 agent 对 H-随 ③c 那 6 格的读法 | 丢写：页上「丢写」一段那条 `grep 'name=r4_loss ' … | awk …` 命令（合并 compare 第 1546–1554、1567–1575、1588–1596 行）；多拒 `grep -n 'name=r4_q4 arm=yi-cfg family=l[0-9a-z]*@GEOMETRY_PRIMARY'`（第 634、638 行）；多读页上「多读」一段的 awk；Q6 第 1564、1585、1606 行 |
| 2（乙-配置续，用户已定 12:08） | 已够判（用户已定） | 两臂只在 H1g 分开（乙-配置 16 / 32、乙-配置续 0 / 32）；其余各族（H1f、第二段四族 seg2b）逐族相同；多拒同 22；同成多读乙-配置续每格多 4（4988 对 2472，629 格） | 不能：同上 | 第 1575、1596 行（H1g）；1569、1590 行（H-随）；1571、1592 行（实七-乙）；634、754 行（L3） |
| 3（乙 / 乙-窄读，记欠） | 记欠 | 两臂判定不逐格相同：乙-窄读 在读回全零、b = records、T(1) 与 O(1 只) 格上拒（H1d 拦下多 `O=4,T=4`、H1f-cuts 多 `O=8,T=8`；PC-N-盖 [臂] 句打不中，Q1–Q3 作废）；丢写、L 族多拒两臂逐族同；同成多读 L 族同（2472 / 4988），H1d 上乙-配置 +50136（32 格）、乙-窄读-配置 +942（24 格）。**读缓存峰值没量**（登记里没有这个量） | 不能：登记里没有峰值这个量，第 4 次跑再跑也量不到；要量得另立 | 第 617、856 行（H1d 拦下）；624、863 行（H1f-cuts 拦下）；657、896 行（H1d 同成多读）；1775、1782 行（作废） |

「上一段岔路表里还差」三件逐件交代：① 第二段实七-乙、实八两族的数页上以 seg2b 为准，写明撞键与重跑、seg2 与 seg2b 在 H-随 / H-随全 / 实七-甲三族逐字节相同（页上贴了 `cmp` 那条命令与 12 行原样输出）；② H-随 那 6 格照登记字面算丢写，页上并列「不算 ③c」「Q1 只算环内且不算 ③c」两栏（命令与 12 行原样输出、两行汇总表）；③ 第 3 行读缓存峰值没量、记欠（页上「它答不了的」第一条与岔路表第 3 行）。

## 三、判决行（第 4c 步，这一次唯一新产的东西是写页；产物是前一个执行员跑出、我核过的合并 compare）

`grep -n 'name=verdict' research/results/e158-root-choice-repair-2026-09-27-r4-compare.out | wc -l` 输出 `0`：这个装置不打 `name=verdict` 行，判定写在各行 `verdict=` 字段。按字段数（`grep -o 'verdict=[A-Za-z0-9_]*' … | sort | uniq -c`）：`33 covered`、`43 fail`、`1 inference_holds`、`12 not_constructible`、`3 not_covered`、`381 pass`、`6 vacuous`。逐个点名「没过」的：

| 行 | 字段 = 取值 | 为什么 | 登记预期内？ |
|---|---|---|---|
| 602–609、1195–1202、1316–1323 | PC-多读 (ii) `point_value[…] tag=arm verdict=fail measured_extra_read_calls=Some(0)expected=Some(24)`（甲-槽、乙-窄读-槽、乙-槽 各 8） | -槽 臂读阶段替换择根那一遍，同一串读，+0 | 是（装置之前主 agent 认定第 1 项） |
| 718、721、837、840、957、960、1076、1079、1203、1324 | PC-多读 (iii) `first_pass_reads_equal_jia_total[…] verdict=fail`（差 1） | 成因没量证 | 否（第一段新发现，主 agent 认定第 3 项已处置） |
| 1382、1384、1386 | PC-22 `refused_only tag=arm verdict=fail`（甲三臂） | O(2 只) 构造里实例表读得出，甲照定义挂上 | 否（第四段新发现，主 agent 认定第 1 项已处置） |
| 1352、1355、1364、1367、1373、1379 | PC-多拒 `arm_refuses tag=arm verdict=fail n1=1`（六条 R = 3 臂） | R = 3 臂没有 L 族的格，compare 把「没格」记成没拒（源码 `arm_cell` 为空时 `refused=false`） | **否，而且只在五段合并之后才出现**，交主 agent（见第五节第 1 条） |
| 1353、1356、1365、1368、1374、1380 | `counted_as_extra_refusal tag=measure verdict=vacuous` | 同上，前件不成立 | 同上 |
| 1402–1413 | PC-22 两句 `tag=arm verdict=not_constructible`（六条 R = 3 臂） | R = 3 臂没有 L5 | 同上 |
| 1564–1566 | `r4_q6 arm=today … not_covered_cells=3 verdict=not_covered`（G0、S16、S4） | 今天没罩住第 22 条那一格 | 是（PC-22 [今] 句要的就是它） |
| 1724–1829 里 26 行 | `r4_void … voided=true` | 上面几类句子的机械后果 | 前 14 行是（与各段 compare 同），R = 3 臂那 12 行否 |
| 1831 | `r4_missed_arm_and_today_sentences count=61` | 43 fail + 12 not_constructible + 乙-窄读 两臂 PC-N-盖 6 句 | 拿不准算不算失败类计数，点名 |
| 1431、1432、1435、1436、1438、1440 | `r4_base_identity arm=*-three-rounds prefixes=0 differing=0 verdict=pass` | 空判：没有可比的前缀 | 拿不准，点名 |

名字表示不匹配 / 差异的整数计数：`mismatches` 2 个、`differences` 6 个、`differing` 18 个、`cells_differing_outside_o1_and_cell_22` 1 个，全是 0。

## 四、复跑与门禁

`bash research/scripts/capped.sh 4 bash research/scripts/replay.sh E158`（`nice -n 19`，后台）汇总行原样：`字节一致 2 ／ 仅计时不同 0 ／ 对不上 13 ／ 跑不了 3 ／ 结论断言不中 0 ／ 产物已归档 0 ／ 输入没变没跑 0`，退出码 1。
- 我登记的那一行：`E158  @driver_e158_r4_compare  字节一致 e158-root-choice-repair-2026-09-27-r4-compare.out`。
- 另一行字节一致的是第 3 次跑的 `driver_e158_r3_seg1_compare`。
- 对不上的 13 行都是早先登记的 E158 行（第一次跑各模式、`r2-all`、`r3-seg1` 今天那一臂），读工作树的 `crates/`，工作树在各次快照之后已改过（含合进 C554 乙），第 4 次跑登记文件头第 5 条预先写了会漂；没查差在哪、没改指。
- 跑不了的 3 行（`driver_e158_q3_1_s4`、`driver_e158_q2_1_g0`、`driver_e158_q2_1_g0_session_s5`，退出码 143）是我停的：我那一行 05:43 UTC（14:43 JST）已出结果，这三行是第一次跑的穷举，跑了 34 分钟还在算，与这一次无关；照共用约束从叶子起逐个 `proc.py stop`（2774796、2774820、2774916），上层随之退出，`ps` 核过一个不剩。全量复跑归 87 号。

门禁（逐个 `nice -n 19 bash .claude/gate.d/<文件>`，日志在草稿目录 `gate-<阶段>.log`，写完页之后一起跑）：

| 阶段 | 退出码 | 首个判定行（原样，截断） |
|---|---|---|
| 27 | 1 | `✗ 格式常量在 kb 与实验源码之间对不上：` |
| 33 | 0 | `✓ 150 个实验二进制都有成形的变异表，1710 条变异的原文各命中源码一次；crates/mutations.tsv 1265 条的原文各命中源码一次；没有同名实验二进制、锚点不在本阶段射程的表 7 张：research/mutatio…` |
| 52 | 1 | `✗ 段序列登记表与 E142 产物的 name=segments 行（或第二条流与钉它的用例）对不上，共 2 处：` |
| 80 | 0 | `✓ 155 个实验二进制各自至少有一条绝对值断言（research/e7-index-bench/src/bin 下 150 份，别处 src/bin 下以 e<数字>_ 开头的 5 份）` |
| 96 | 0 | `✓ 实验源码纪律：扫了 155 个文件（读不动的 0 个）；C59 查了 745 处名字带 seed 的标识符，折叠写法 17 处（豁免表 17 行，放行 17 处）；C60 查了 1471 个整型结构体字段，恒为字面量 0 的 14 个（…` |
| 34 | 1 | `✗ 实验索引与正文对不上 1 处：` |
| 40 | 1 | `✗ 有实验产物没被 experiments.md 点名——跑过但没写回，或写回了却无法复核：` |
| 69 | 1 | `✗ 这些实验装置与变异表这一轮改了，research/results/ 里却没有一份不比它旧的产物——这一跑多半只留在 /tmp 的草稿里：` |
| 75 | 0 | `✓ 决策与实验双向登记对得上、回看不过期（查了实验页 161 个、决策 28 条；实验页 161 个（待回填 0）、决策 28 条（待回填 0）、已瘦身决策的已定项 345 个（其中写「无实验」的 147 个）、表行 798 行、这次改动触…` |
| 84 | 0 | `✓ 登记产物里判决字段是 false 的都在对应实验页里点了名（点名 15 个）` |
| 85 | 0 | `✓ 点了产物的实验都写了复跑命令（点了产物的 17 页里判了 17 页）` |
| 86 | 1 | `✗ research 里有这些实验号的东西，kb/experiments/ 里却没有正文：` |
| 88 | 0 | `✓ 这次改动新增或改写的 kb 正文里整行抄的产物行都在产物里逐字找得到（判了 68 行，对照 156 份登记着的产物）` |
| 99 | 0 | `✓ 登记了路径与共用项的实验页判过了（4 份、19 条路径）；还没写登记节的 9 份在 multipath-registry-lag.tsv 里、每一份都现存且确实还没写，与基准 faf255e235300d129ede6d6f85af31…` |

- 40 号现在点名的只剩 `research/results/gate69-refresh-replay-e155-e157-e159-2026-09-25.log` 与 `…-postupdate-2026-09-25.log` 两份（`grep -c 'e158-root-choice-repair-2026-09-27' gate-40-results-cited.log` 输出 `0`）；写页之前它点名 E158 的 r2 / r3 / r4 共 96 份（前一个执行员留的 `gate-40-before.log`，`grep 'research/results/' gate-40-before.log | grep -c 'e158-root-choice-repair'` 输出 `96`）。
- 27 号红在 `e142_…rs:106`、`e157_…rs:74`；34 号红在 E162 没进索引；52 号红在 E142 与 layout kb；69 号红在 E163 装置没有产物、`research/prompts/` 下几份别人的报告引 `/tmp`；86 号红在 E163。日志里 E158 命中都是 0，都不是这一次写的文件，没修；86 号出路里「把 research 下那些文件删掉」没照做。
- doc-lint（`bash .claude/singlefs-ai-sop/scripts/doc-lint.sh .`）末行：`✗ 文档铁律检查失败：1 个文件违规、0 处编号引用无定义、1 处编号定义/引用不合规、0 条排除项不合规、0 处规则清单/警告记录不合规（检查 523，跳过 0）`——违规全在 `.claude/kb/experiments/162-崩溃放量判定块存储选型.md`（自指、「本轮」、E161 / A6 裸引用）；我写的实验页、`experiments-history.md`、`experiments.md` 没有命中。第一次跑时我自己的页上红过两类（`T1` 在 D13（验证路线） 里有登记位、我把它写进了 not-numbers；`key=E158` 与 `grep -c 'E158 …'` 两处裸引用），已改成 `T(1)` 与不带编号的写法。

## 五、要主 agent 看的

1. **五段合并之后多出 12 行作废，是 compare 的写法，不是数**：六条 R = 3 臂（只跑第五段 `h1d-uncut-r3`、`h1e-r3` 两族，没有 L 族）在合并之后第一次遇到今天那一臂的 L3 格，`r4-compare` 在这一臂找不到同一格就按 `refused=false` 判 `arm_refuses` `fail`（读代码核过：seg2b 今天那一臂副本的 bin 第 16239 行取 `arm_cell`、第 16272 行 `let refused = arm_cell.is_some_and(|cell| cell.text("refused") == "true");`、第 16276 行打 `"arm_refuses"`，`arm_cell` 为 `None` 时 `refused` 取 false），PC-22 两句记 `not_constructible`，于是 V1 ④ 把这六臂的 Q4、Q6 标作废。这六臂在合并 compare 里没有一行 `r4_q4` / `r4_q6`，作废不碰任何数；各段自己的 compare 里没有这 12 行。页上写明了；要不要改 compare 把「没有格」记成 `not_constructible`，是新一次跑的事。
2. **D23（journal 的角色与格式） 第 432 行**（`.claude/kb/decisions/23-journal的角色与格式.md`）那条依据还写着「E158（择根与修复四岔路） 实验页还没写这一段，依据先指……两份报告」。页现在写了，这一句该改指实验页「2026-09-27：第 4 次跑」一节；决策文件不在我的写范围，没动。实验页那一行仍是「支撑」（依据段引了 E158（择根与修复四岔路））。
3. **`replay.sh` 只登记了五段合并的 compare**：各段臂产物（含今天那一臂）要在快照副本上跑，工作树的 `crates/` 已合进 C554 乙（`mount.rs` 里 `PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion`），今天那一臂在工作树上已不存在，登记它们只会立刻漂；第 3 次跑登记了今天那一臂（`driver_e158_r3_seg1_today`，现在对不上 2239 行），第 4 次跑没照这个先例。要照先例补今天那一臂的几行，交主 agent 定。
4. **标题的旧括注没有原样搬进历史**：约 4.7 KB，逐段对应正文里第一次跑各节与第 2 次跑一节；页末历史与 `experiments-history.md` 都写了「改前那串括注的原文在这一次改动之前的版本里（git）」。要原样搬进 `experiments-history.md`，交主 agent。
5. **「路径与结论登记」一节没补**：第 4 次跑 7.3 是装置自己的账与 `crates/` 交回的逐项对拍（`device_n_cfg` 对甲-配置、`device_n_slot` 对甲-槽、装置重放的 E 对 `crates/` 的 E），按 `.claude/rules/format-evolution.md` 那一节该有这张表；E158 页从来没有、也不在 `multipath-registry-lag.tsv` 里（99 号查不到）。这一次没写，交主 agent 定要不要补、谁补。
6. seg2b 产物与合并 compare 的首行指纹都是 `ad9f5442…`，没把 seg2b 修过的装置 `e71224fc…` 算进去（seg2b 执行员已交），页上写明了。

## 六、单测、变异、登记修订

- 单测：`grep -c '#\[test\]' crates/singlefs-harness/src/bin/e158_root_choice_repair.rs` 输出 `95`。这一段没改装置、没跑单测。
- 变异：这一段没跑。第 4 次跑的装置变异在 `crates/mutations.tsv` 里 24 行（`grep -c 'root_choice_repair 第 4 次跑' crates/mutations.tsv` 输出 `24`，第 1118–1128、1243–1255 行），入库装置那一支三个数写「待提交时 59 号」；各趟执行员在草稿副本上的数（装置 11 / 0 / 0、第二段 11 / 0 / 0、seg2b 12 / 0 / 0、臂表 8 / 0 / 0、M3 臂那一半 1 / 0 / 0）抄自各段报告，没复核。`mutate.sh` 没跑，内存撞顶与超时没有数。
- 登记修订：没有（产物早已跑过，按定义不改登记）。

## 七、没做什么

- 没跑任何新产物（合并 compare 是前一个执行员 05:17 UTC 起、跑完的，我只核了它齐不齐、与各段逐行对上、复跑逐字节一致）。
- 没判任何候选出局或胜出，没判结论能不能推翻或确立决策；没改决策文件。
- 没跑门禁全量、没跑 87 号与任何重型测试，没提交。
- 没修 27、34、52、69、86 号与 doc-lint 在别人文件上的红。
- `replay.sh E158` 里第一次跑的三行穷举被我停了（第四节），它们的结果没有。

## 八、草稿目录 `/tmp/claude-1000/e158-r4-page/`

- 只有文本：交接摘要、`progress.md`、各段草稿 `section-*.md`、`insert.md`、`inserted-actual.md`、门禁日志、doc-lint 日志、`replay-E158.log`、`run-gates.sh`、本报告。没有编译目录、仓副本、工作树。
- `replay.sh` 自己留下的输出目录 `/tmp/singlefs-replay-2773721/`（各行的 `.out` / `.err` / diff 用，外加它建的稀疏镜像 `e9.img`、`e45.img`）没删：汇总行点名它给 diff 用；不是编译目录，也不是仓副本。
- 臂副本 `/tmp/claude-1000/e158-r4-seg2b/arms/`、`/tmp/claude-1000/e158-r4-device/arms/` 是别的执行员建的，我没动。
