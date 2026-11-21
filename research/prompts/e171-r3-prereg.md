# E171（三oracle合谋测试） 第 3 次重跑登记

写于 2026-11-09，执行器与装置改动之前。批次 `records/2026-11-08-重型门禁执行问题与缺陷总结.md#E171复跑红行`。
准入已判：用户 2026-11-09 弹窗定「修执行器并重出全部产物」；复跑表 29 行现跑对不上或跑不了，不是「输入没变、要不要跑」的问题，准入不拦。
上一次登记：`research/prompts/e171-r2-prereg.md`（第 2 次重跑，870 行）与 `research/prompts/e171-r1-prereg.md`（第 1 次重跑，2 988 行）；原登记（2026-10-03 那一批）已归档，`git log --all --diff-filter=D --name-only -- '*e171-preregistration.md*'` 取。
这一次不是差分重跑：臂与判据照上一次登记，但多了三样新东西（O3 执行器要跟上今天的盘上格式、判定方向变了要有判法、29 行逐行的比对对象表），所以第一至十四节自己写；各节没写的「同上一次登记」按每行所属的那一次登记读（原登记、第 1 次重跑、第 2 次重跑，对应关系在第十一节表里）。

只读了：调查报告 `/home/fy5090/.local/share/singlefs/work/e171-row-reds/result.md`（整份）、`replay.sh` 复跑表与 E171 各驱动函数、两份上一次登记的标题与第 2 次重跑登记第一节、E201 登记的第一、四节与标题、实验页第 1、13 至 20、31、1349 行（标题行与点名的行，读到的数照第四节列；页上其余的结果与结论小节没打开）。`research/results/` 下的产物没打开，只算了 sha256 与行数。

<!-- doc-lint:not-numbers R1 R2 R3 R4 R5 V1 V2 V3 V4 V5 V6 V7 V8 V9 V10 V11 C1 C2 C3 C4 C5 I1 I2 I3 I4 I5 X1 X2 S1 S2 S3 S4 S5 S6 S7 F1 F2 F3 F4 F5 F6 F7 F8 Z1 Z2 Z3 Z4 Z5 Z6 Z7 Z8 Q1 Q2 Q3 Q4 Q5 A1 A2 A3 A4 A5 A6 A11 O1 O2 O3 P1 P2 P3 D1 D2 D3 -->

## 一、问题

英文名：three_oracle_collusion

主 agent 给的问题（派发提示「关的是哪几条」原文逐字，五条）：

> 1. 第 5、6 行：执行器 scripts/experiment-apparatus/e170_second_run_o3_spec_executor/system_configuration.py:21 只认 incompat 首字节 0x02，产品 crates/singlefs-core/src/system_configuration.rs:33 与 .claude/kb/layout/01-first-txn.md:113 是 0x06；探针任务 10980 量到换 0x06 后两条流都恢复得出，但记录头 311 对 496（on_disk_layout.py:87 对 crates/singlefs-format/src/lib.rs:252），只改常量大概率不够。验收：登记写明这份 O3 执行器要跟上今天的盘上格式，要改到哪些常量与结构、怎么证它仍是不读 crates/ 的独立规范（独立性怎么保，判据写死）。先核同目录下 e170_second_run_o3_spec_executor_journal_header_487 这一份与它的差别和用处：用户要的是「修执行器」，改已有目录还是新建一份，你给依据定一个，写明它对 replay.sh 里 E170 那些行有没有影响。
> 2. 第 1 到 4 行的判定方向变化要先有判法，不能重出就算了。放行变红：E171.2 row 10、31，E171.4 row 10、30，E171.3 row 10 `P5-Mk`、row 30 `B13:P5:Mk@1`，都多判 I-8.14。红变不红：已知坏镜像 I-5.1 少 I-3.1、I-3.11，I-9.1 少 I-3.1。读成功变实现拒：E171.3 有 28 行 `expected=not_in`，报错在 recovery.rs:6811。验收：跑前判据写死这几类各自怎样算正常、怎样算产品或检查器有问题；结果反过来时怎么办也写。
> 3. 29 行全部重出。第 7 到 29 行这一趟没判定（任务在第 7 行开跑后取消），要预言的地方写明没量过。验收：比对对象表一行一个正式跑，29 行齐；每行写驱动、旧产物、重跑产物名；实验页第 31 行写二十五行与现在 29 行对不上，登记里写明。
> 4. 第 1349 行写「复跑一次出的是复跑那一刻那棵树的读数」，今天的树归 E201（三oracle合谋今树整表，装置已写、产物未跑）。验收：登记写明这次重出与 E201 的关系、谁都不替谁判，不改 E201 的登记。
> 5. 这 29 行耗时：E171.1 单行 3483 秒，每行还带一次 400 秒左右的前置。验收：登记写明能不能共用前置、哪些行要卡，给主 agent 决定次序用；不写线程、内存与机器。

### 读法写死

1. **「重出」**：复跑表里 E171 的每一行驱动，在开跑那一刻的主工作区（含已暂存与未暂存）上各现跑一次，产物写**新文件名**（第十一节表「重跑产物名」列），旧产物一个字节不动（E201 登记第一节读法 2 按 sha256 读其中几份，第七节 V9 核）。判定库里有同键记录的行可以「复用」，复用与现判分开报；复用的行产物取记录里留的那一份，同样算这一行重出了。
2. **「29 行」**：`grep -c '^E171|@driver' scripts/experiment-apparatus/replay.sh` 现数（写这一条时执行：29，第 2306 至 2334 行）。行号 `E171.<n>` 的 n 是这 29 行里的次序（调查报告里的 E171.1 到 E171.6 同此），第十一节表的 `#` 列同此。表里的次序不是依赖次序：第 9、10、11 行的驱动一条套一条（第十四节）。
3. **「O3 跟上今天的盘上格式」**：在新目录（读法 4）里，执行器对本实验两条流的基底镜像（`base-sectors.bin`）恢复得出，并且第五节 X1 的全部检查（V1 至 V5）过。「恢复得出」不等于「能判」：探针任务 10980 里常量换成 0x06 之后基底也恢复得出（`landed=(0,0,wm=65536,floor=0)`），但那是 mkfs 镜像、里面一条 journal 记录都没有，记录头宽度错了也看不出，所以不拿它当证据（第四节第 5 条）。
4. **目录定案**（岔路 R1）：新建，不改已有目录。派生自 `scripts/experiment-apparatus/e170_second_run_o3_spec_executor_journal_header_487/`（下称「487 目录」）的一份拷贝；名字执行员定，须说出内容（建议 `e170_second_run_o3_spec_executor_journal_header_496`，沿用 487 目录的命名；不用序数、不用「新」「旧」）。冻结目录 `e170_second_run_o3_spec_executor/` 与带见证谓词的 `e170_o3_spec_executor_with_witness_predicates_journal_header_487/` 一个字节不动。依据在第五节 5.1。
5. **判定方向变化**（岔路 R2）只按产物里的字段判，不按行号、不按第四节那几条点名的行判（那几条是预言，不是判据）：`expected=in|not_in`（E170 导出装置给每个坏状态行标的「该不该被恢复读出」）、`target_is_red`（已知坏镜像条目的目标不变量红没红）、`impl_consulted_outcome`、红集（逐个不变量编号）。五类 C1 至 C5 的定义与处置在第六节。
6. **「判定词」与「钉值」**：同第 2 次重跑登记读法 2。判定词变了是岔路 R3 的「判定变」，钉值变了另报、不进「判定变」。

### 岔路单（问题单，五行；派发提示五条逐条对应，没有倾向、没有数）

| 岔路 | 候选 | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|---|
| R1：O3 执行器跟上盘上格式，改哪里、怎么证独立 | X1 新建目录（487 目录的拷贝加改动）；X2 原目录（冻结目录）原地改加重冻结 | 新目录对基底与全持久态恢复得出且 V1 至 V5 全过；X2 的改动面（引用该目录的文件、变异表锚点）是否大于 X1 | V1 至 V5 在两条流上有读数；引用清单现数 | 开着 |
| R2：判定方向变了，怎样算正常、怎样算产品或检查器有问题 | 每类 C1 至 C5 各自「正常」或「有问题」或「归不到」 | 按第六节规则对重出产物逐行判后，落在「有问题」或「归不到」的行数 | 第 1 至 4 行重出、第 5 至 11 行重出（O3 列有读数之后）各按规则判完 | 开着 |
| R3：29 行重出之后，每一行是「逐字节同」「判定词不变、钉值变」「判定词变」还是「跑不了」 | 四种结局之一 | 每行重出产物与旧产物（第十一节表）并排 | 29 行每行有结局；「跑不了」的有缺的输入点名 | 开着 |
| R4：这次重出与 E201（三oracle合谋今树整表）的关系 | 互不替代、互不改；或这次的某行读数被 E201 引用 | E201 登记里读的旧产物这次动没动；E201 的 O3 目录是不是这次修的那一份 | V9 与引用清单有读数 | 开着 |
| R5：29 行的次序、共用前置、哪些行碰 GPU | 各行独立跑；套着的三行只跑最外一条取三份；先跑便宜的探针行 | 各行预计挂钟与前置是否可共用（第十四节） | 每行有预计挂钟与「卡」标注，没量过的标明 | 开着 |

未定项：
1. **第 12、13、15 行（r1 的比对器与跨机对照）的输入**：驱动读工作根 `replay-frozen-roots/e171-r1-dumps`，现在那里只有 `e203-segment-two-source`（`ls` 现查）；实验页标题行写「冻结层依赖的 r1 输入造不回来」。缺：要么这三行记「本次未跑·输入缺」（本登记的默认，第六节 V7），要么主 agent 定要重建 r1 修订 6 的冻结树与导出（驱动文件头写「推的挂钟几十小时」、两台）。谁来定：主 agent 或用户。
2. **第 9、10、11 行取一份还是三份**（第十四节）：缺一个只取产物的入口（要改 `replay.sh`，执行员的活）。谁来定：主 agent。
3. **第 7、8、9 至 11 行的挂钟**：没量过，只有驱动文件头的推数（第十四节）。承接的设计员或主 agent 在第 5 至 6 行跑过之后按读数补。
4. **分段怎么落到 `replay.sh`**：`replay.sh` 按实验号点名，E171 的 29 行没有行选择器（5.5「行级点名」）。缺：加选项、整条跑后取消、还是单驱动命令三选一。谁来定：主 agent。

## 二、被测条款与它引的定义

被测条款：`.claude/kb/experiments/171-三oracle合谋测试.md` 的「现状」与 Q1 到 Q4（第 13 至 20 行），连同点名这一次要对的两句（第 31 行、第 1349 行）。`quote-kb.py` 整段抄，回读逐字节一致。这一次重跑不问「现状」与 Q1 到 Q4 的判定对不对（不重判旧产物），问的是它们赖以立足的 29 行复跑今天还出不出、出的和旧产物差在哪里。

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:13-20`（整段抄，未转述）**

```markdown
**现状**：坏实现表 117 行用修过的逐行驱动在同一棵冻住的树上跑完一遍（s4，两条线道、每条 2 线程），连同 Q3-核三趟与比对器；s3 那一次整表因驱动把上一行的变异带进下一行而整表作废、留档（「s3 整表作废」一节）。O3（独立规约执行器） 用 E170（O3规约执行器第一版） 第 1 次重跑冻结的执行器，开头与收尾两次冻结核对都是 0 处不同。s4 的分母 D = 19 < 30，登记 F4 触发，扩表 164 行（序号 118–281）在同一份冻住的树上跑完，比对器把两半并成 281 行：分母 D = 37 ≥ 30。两趟里编不过的 6 行（第 33、59、127、163、206、243 行）当时的驱动没按登记 5.2 补位；驱动改成编不过之后补位，补位那一趟在同一份冻住的树上跑完（六行换上的都是未触达的行），比对器把 117 + 164 + 补位并成 `…-s4-f4-substitutes-comparator.out`：分母仍 D = 37，Q1、Q3 名单与补位之前逐字同（「编不过的 6 行补位（已跑）」一节）。读数到齐之后开了三轮结论判决（`research/prompts/e171-conclusion-r1-main-verification.md`、`…-r2-main-verification.md`、`…-r3-main-verification.md`，第三轮是最终轮），判这批读数撑不撑 D13（验证路线） 已定项 6：真盲区候选收成第 15、23、42、242、269、270、271 行，别的全放行行与未触达行里交回值不同的行归另几类（「三轮判决之后的三分」一节）；三分与读数一样只属于 s4 那一棵冻住的树。

| 行 | 判定 | 依据（产物） | 照字面之外要一起读的 |
|---|---|---|---|
| Q1 | 有：甲、乙各 12 行三条全放行（s4 的 7 行加扩表的第 242、269、270、271、277 行），丙 13 行（另加第 233 行）；分母 D = 37（补位之前与之后相同） | `e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator.out` 的 `name=q1_alpha`、`name=q1_beta`、`name=q1_gamma`（`verdict=yes_a_row_is_quiet_on_every_oracle`）、`name=q1_denominator`（`counted_D=37 … enough_to_judge_none_side=true`）；补位之前的 `…-s4-f4-comparator.out` 里 Q1 三种读法的名单行逐字同 | 照登记第六节 Q1-甲「≥ 1 判『有』」，与分母无关；D ≥ 30 只说「没有」那一边的门槛够了，这一批里「有」已成立。s4 那 7 行的读数在 F4 产物里没变（第 1–117 行的结果行逐字节相同）；扩表新出的 5 行与第 233 行（丙）、批 2 的第 265 行的读数在「F4 新出的全放行行」一节。三轮判决之后的三分在「三轮判决之后的三分」一节：真盲区候选 7 行（第 15、23、42、242、269、270、271 行），另几类 5 行（第 27、112、277 行靠记录重建的恢复，第 34 行分配政策，第 92 行条款在字节表与里程碑验收），没有一行还记「对这一批等价」；第 233 行在丙里全放行（甲乙里 O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官抓到），第 265 行批 2 是取样点不敏感。O3（独立规约执行器） 在 37 行上一行都没红（`name=q1_column_red_rows … O3（独立规约执行器）甲=0 O3（独立规约执行器）乙=0`）。补位之后分母仍 37，「有」没翻 |
| Q2 | 有（照登记字面）：甲、乙各 4 张两条全放行，全是 P8 | 同一份产物的 `name=q2_all_quiet`（`table_rows=107 all_quiet_rows=4 verdict=yes_a_bad_image_passes_both`）、`name=f3_p_rows_judged_in_by_o3`、`name=q2_negative_bases`；O2（独立解析器 + checker） 放行而 O3（独立规约执行器） 判不在的数在 `name=q2_family` 的 P 族与盲测 B 族两行（`o3_only_strict=12`、`o3_only_strict=17`，整行抄在「Q2 里 O2（独立解析器 + checker） 放行而 O3（独立规约执行器） 判不在的坏镜像」一节） | 4 张都是 E170（O3规约执行器第一版） 设计员推的「不在」而 O3（独立规约执行器） 判「在」的 P8 行，F3 触发；主 agent 照 F3 判成期望推错、这四张不是坏镜像（判决记在 D13（验证路线） 已定项 6 的射程）。Q2 的数与第二段 `…-s2-comparator.out` 同出一批输入，s4 比对器重算一遍；F4 产物里 Q2、F3 与对照的结果行与 s4 那一份逐字节相同。几何两点（`q2_geometry_unstable_kinds=2`）：翻面的是流一 P2、P5 两种补丁的 O2（独立解析器 + checker） 一列（底态 F 红、Mk 不红），O3（独立规约执行器） 两个底态都判「不在」，所以这两种补丁的「两条全放行」两点都不成立、不翻；全放行的 P8 两个底态都是 `same_on_both_points`。另有 O2（独立解析器 + checker） 放行而 O3（独立规约执行器） 判不在的坏镜像 29 张（P 族 12 张加盲测 B 族 17 张，12 + 17）：写「O2（独立解析器 + checker） 放行而 O3（独立规约执行器） 判不在」，不写「O3（独立规约执行器） 独有」，Q2 只比 O2（独立解析器 + checker） 与 O3（独立规约执行器） 一对，O1（内存模型对拍（HashMap 理想文件系统）） 与 RefFS 没在坏镜像上跑；这 29 张是结构自洽的伪造根、伪造续接镜像，补丁与期望同出 E170（O3规约执行器第一版） 的改法代码，细节在「它答不了的」 |
| Q3 | 有一行只靠一条：甲、乙各 23 行（只靠 O1（内存模型对拍（HashMap 理想文件系统）） 的 32、45、108、115、172、233、237、249、250、281；只靠 O2（独立解析器 + checker） 的 8、65、86、87、90、99、102、135、234、238、245、258、265） | 补位之后的 `…-s4-f4-substitutes-comparator.out` 的 `name=q3_alpha`、`name=q3_beta`（补位之前的 `…-s4-f4-comparator.out` 同名行逐字同）；Q3-核 9 行 `name=q3_core_row` 全是 `differences_outside_the_closed_column=0`（Q3-核取第一刀的行，扩表没有第一刀） | Q3-核 9 趟 0 处不同，读法 9 成立、F5 不触发。第 102、233、265 行批 1 与批 2 红的列不同，都是取样点不敏感（批 2 没取到红的那几段或那一类态），记「不稳定」。s4 那 11 行另外两条为什么看不到在「Q3 只靠一条的 11 行」一节，扩表新出的 12 行在「F4 新出的只靠一条的 12 行」一节 |
| Q4 | 只能进全量：117 行整表两条线道 5 569.6 秒，扩表 164 行另一趟 7 607.9 秒，门槛 60 秒 | `e171-three-oracle-collusion-2026-10-03-s4-f4-rows-timing.out` 的两行 `name=table source=table-driver.out`（`rows=117 table_wall_ns=5569579167726`、`rows=164 table_wall_ns=7607936234181`） | 281 行分两趟跑，两趟挂钟相加约 13 177.5 秒（两个数相加，不是一趟量的）；各行之和 11 114.4 + 15 191.9 秒是单线道的估计（没单独跑单线道）。编不过的 6 行补位另一趟 1 277.9 秒（`…-s4-f4-substitutes-rows-timing.out` 的 `name=table source=substitution-driver.out … rows=6 table_wall_ns=1277937171946`），三趟相加约 14 455.5 秒，也是三个数相加。Q3-核每趟三行约 114–130 秒 |
```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:31-31`（整段抄，未转述）**

```markdown
| 复跑 | `scripts/experiment-apparatus/replay.sh` 里 E171（三oracle合谋测试） 二十五行登记（`grep -c '^[E]171.@driver' scripts/experiment-apparatus/replay.sh`）：这一页原来的十一行，加第 1 次重跑九行（第一段·修订 6 那一趟三行、第二段两行、修订 8、10、11 的研究一侧变异三行、修订 12 一行），加第 2 次重跑五行 | 原来十一行见「复跑」一节；第一段那三行见「比对器、复跑与入库的跑法（修订 6 那一趟）」一节，第二段那两行见「第 1 次重跑第二段·修订 6 那一趟」一节，修订 12 那一行见「第 1 次重跑·修订 12：流三段长重钉（2026-10-13）」一节，修订 8、10、11 那三行页里没有节，第 2 次重跑五行见「第 2 次重跑·撤项核算段」一节 |
```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:1349-1349`（整段抄，未转述）**

```markdown
- 这一页的行级读数（全放行、只靠一条、未触达的名单与每行的态）只属于快照摘要那一棵冻住的树：它与现行的 `crates/` 不是同一份，页里的名单不能直接当现行实现的名单；`replay.sh` 的 E171（三oracle合谋测试） 复跑行读现行的 `crates/`，复跑一次出的是复跑那一刻那棵树的读数。s4 那一版入库装置（`scripts/experiment-apparatus/e171_three_oracle_collusion/`）上除 Q2 的 29 张之外，这几个数都不能原样重做（这一版装置没有挂载模式、没有 RefFS 列、panic 的流不导出；第 1 次重跑的装置 `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/` 有 RefFS 那一列，挂载模式与 panic 的流导不导出两项没核）。
```

## 三、实现今天的样子

主 agent 的观测（本登记写时现读，`grep -n` 与 `sed -n` 取行号）：

1. **产品的盘上格式常量**（`crates/` 里，执行器不读它们，只在停机条款里对一遍）：`crates/singlefs-core/src/system_configuration.rs` 第 33 行 `INCOMPAT_FIRST_BYTE_OF_THIS_LAYOUT`（位 1 与位 2，0x06）；`crates/singlefs-format/src/lib.rs` 第 234 行 `ROOT_RECORD_BYTES = 466`、第 249 行 `JOURNAL_NEW_ROOT_SEGMENT_BYTES = 373`、第 252 行 `JOURNAL_HEADER_BYTES = 496`。kb 同值：`.claude/kb/layout/01-first-txn.md` 第 113 行（incompat 第一个字节 0x06）、第 360 行（头合计 496、4096 记录余 3600、装 64 个点名项）、一节根记录表（回收门槛 t 8、钉槽 p 1 在分配记录树根指针之后、算法类型之前）。
2. **执行器今天的常量**（冻结目录 `scripts/experiment-apparatus/e170_second_run_o3_spec_executor/`）：`system_configuration.py` 第 21 行 `FIRST_VERSION_INCOMPATIBLE_BITS = 1 << 1`（第 106、204 行两处比较）；`on_disk_layout.py` 第 71 行 `ROOT_RECORD_BYTES = 457`、第 87 行 `JOURNAL_HEADER_BYTES = 311`。487 目录与带见证谓词的 487 目录：`on_disk_layout.py` 第 94 行 `JOURNAL_HEADER_BYTES = 487`、第 95 行新根段 364、第 71 行根记录 457；`system_configuration.py` 第 21 行同样是 `1 << 1`（两个 487 目录的 `on_disk_layout.py` 与 `system_configuration.py` 的 sha256 逐字相同：`b4b2cf97…`、`a07a280f…`，`sha256sum` 现算）。
3. **执行器与产品对不上的地方**（读两边常量得出，不是量到的行为）：incompat 首字节 0x02 对 0x06；根记录 457 对 466（多回收门槛 t 8 与钉槽 p 1）；journal 新根段 364 对 373（同一对字段）；记录头 311（冻结目录）或 487（两个 487 目录）对 496；点名项数 `(4096 − 头) ÷ 56` 在 496 时仍是 64（3600 ÷ 56 = 64.28，取整 64），记录头之后余 16 字节（执行器里写的是余 25）。
4. **487 目录与冻结目录的差**（`diff` 逐文件数 `<` 加 `>` 行，现算）：`executor_core.py`、`executor_main.py`、`root_ring.py`、`version_table.py`、`system_configuration.py`、`crash_dump.py`、`crash_image.py`、`crc32c_castagnoli.py` 为 0 行；`journal_ring.py` 8 行，`on_disk_layout.py` 38 行，`replay_prefix.py` 67 行（多一个 `RebuiltRoot` 数据类：重建的根带实例表指针、分配记录树根指针、卸载记号，投影仍只取六样），`spec_gaps.py` 3 行，`spec-gaps.tsv` 1 行，`synthetic_pool_for_tests.py` 47 行，`test_executor_on_synthetic_pool.py` 120 行。冻结目录独有 `comparator*` 五份与 `mutation_driver*` 两份（不在冻结范围里，`frozen_o3.py` 第 48 行起 `FILES_OUTSIDE_THE_FREEZE`）。所以判一个状态落不落在允许集合里的逻辑（`executor_core.py`）两边逐字相同，487 目录比冻结目录多的是格式补丁。
5. **谁引冻结目录**（`grep -rnE 'e170_second_run_o3_spec_executor([^_A-Za-z0-9]|$)' scripts crates .claude/gate.d`，排除 `__pycache__` 与样本目录，命中 14 份文件）：`replay.sh`（第 3072、3144、3213 行，E171 第二段起 O3 一侧）、`scripts/experiment-apparatus/e171_three_oracle_collusion/frozen_o3.py`（第 43 行目录、第 44 行冻结参照 `research/results/e170-r1-o3-spec-executor-2026-11-05-refreeze-executor.out`）、同目录 `o3_bad_image_judge.py`（第 159 行取比对源的摘要）、`e171_three_oracle_collusion_after_the_oracle_fixes/o3_bad_image_judge.py`、`e170_o3_spec_executor_with_witness_predicates/` 的 `run_segment_two.sh`、`run_segment_three.sh`、`mutation_driver_third_segment.py`、`comparator_bad_image_judge.py`，带见证谓词的 487 目录的 `run_q5_record_path_counts.py`、`comparator_q5_format_constants.py`、`run_segment_one_b_anchors_and_freeze.py`、`run_replay_result_lines.py`，冻结目录自己的 `mutation_driver_with_batch_and_bad_state_checks.py`，以及 `.claude/gate.d/comment-history-lag.tsv` 第 38 行；还有变异表 `scripts/mutation/tables/e170_second_run_o3_spec_executor.tsv`（锚点在冻结目录里）。
6. **E170（O3规约执行器第一版）在 `replay.sh` 里的九行**（第 2297 至 2305 行）只引两个 487 目录（第 2984、2998、3003、3013、3018、3033 行的 `e170_r2_bin …_journal_header_487`），不引冻结目录。
7. **E171 第二段起 O3 一侧怎么起执行器**：`replay.sh` 第 3144、3213 行 `cd scripts/experiment-apparatus/e170_second_run_o3_spec_executor && python3 executor_main.py --dumps … --workers 6`；`frozen_o3.py` 的 `run_executor` 与 `o3_bad_image_judge.py` 的 judge、control 经 `sys.path` 载入同一个目录的 `executor_core`、`crash_dump`、`replay_prefix`（`o3_bad_image_judge.py` 第 44 至 50 行）。第 5 行的开跑前核对读 `E171_FREEZE_REFERENCE`（`replay.sh` 第 3050 行，指 `e170-r1-…-2026-10-03-s1-executor.out`），Python 一侧读的是 11-05 重冻结那一份：两处冻结参照不是同一份文件。
8. **执行器对象的自校验**：`executor_main.py` 第 147 行起 `anchor_assertions`（A1 至 A6、A11：根环四个根的 (区域, 槽, 盘, 实例, txg)、环内三条记录 (0, 4096, 8192)、系统配置选中槽 (世代 5, 槽 1, tail 3, 实例 1)、版本表 `[(0,0),(1,1),(1,2),(1,3)]`；`executor_main.py` 第 148 行文档串说它们来自 E170（O3规约执行器第一版）登记第七节，kb 里对应的表我没逐值核），执行器每跑一条流就打 `name=anchor_assertion`。新目录不许改这些锚点（第九节变异、第十一节停机）。
9. **E201 的 O3 目录**：E201（三oracle合谋今树整表）登记第一节读法 3 点名 `e170_o3_spec_executor_with_witness_predicates_journal_header_487/` 为它的 O3；`e171_three_oracle_collusion_after_the_oracle_fixes/frozen_o3.py` 第 41、42 行同此。该目录的 `system_configuration.py` 第 21 行与冻结目录同为 `1 << 1`（第 2 条）。
10. **判定库复用**：`replay.sh` 文件头「复用」一节：行键含驱动函数的 shell 单元闭包与装置 main 起的单元闭包，键相同的行不现跑、取记录里的产物。改 `replay.sh` 里某个驱动或它引的 Python，只让引到它的行换键。

## 四、跑之前已经存在的数

读条款、判问法时已经知道或算出的数，照实列，不删。标「量」的是读文件或跑只读命令得到，标「报告」的是调查报告 `e171-row-reds/result.md` 里调查员量的、我没复核，标「推」的是读代码或读驱动文件头推的。

1. **前 6 行的现跑读数**（任务 10900，报告）：第 1 至 4 行现跑产物与留存产物的 `name=verdict` 四行逐字相同；逐行差分后能全部分类，行数合计等于 `diff` 行数：E171.1 193 行、E171.2 138 行、E171.3 246 行、E171.4 144 行。第 5、6 行（O3 一侧）败在 Python：执行器对两条流的基底都报 `no_choosable_system_configuration_on_any_device`（`system_configuration.py` 第 262、269 行与 `executor_core.py` 第 70 行抛出，报告读码得出；探针任务 10980 前两行 `arm=constant_as_frozen` 量到同一句）。
2. **判定方向变了的点名**（报告，量）：放行变红六行（E171.2 row 10、31，E171.4 row 10、30，E171.3 `P5-Mk` row 10 与 `B13:P5:Mk@1` row 30，都多判 I-8.14，E171.3 两行 `expected=not_in`）；原来就红而多红一项：E171.2 15 行多 I-8.14、1 行多 I-3.21，E171.4 18 行多 I-8.14、1 行多 I-3.21，E171.3 18 行多 I-8.14、流二 row 13 `N1-F` 多 I-3.21；读成功变实现拒：E171.3 28 行 `impl_consulted_outcome` 由 `file_read` 变 `failed:InvariantViolated`，报错句在 `crates/singlefs-core/src/recovery.rs` 第 6811 行附近（`记账条目数（inode 号水位的台阶行不算）不是 3 + 6 × 盘数`，量：`sed -n 6805,6815p`），28 行全是 `expected=not_in`、`expected=in` 的行没有一行结局变；红变不红：已知坏镜像 I-5.1 少 I-3.1、I-3.11 多 I-3.15、I-3.21，I-9.1 少 I-3.1，两条的目标不变量仍红（报告）。其余已知坏镜像条目九条红集只增不减（报告）。
3. **第四节第 2 条里「推」的部分**：28 行被拒的成因「伪造根的 txg 变大，当前那条水位行被算成台阶行」报告标「没量」；I-5.1、I-9.1 少红的归因报告标「没归因」；I-3.1 现行定义写成「对最新根账里的记录比」（`crates/singlefs-checker/invariants/05-空间记账.md` 第 8 行）。I-8.14 的定义（`03-journal.md` 第 21 行）是「新实例第一个 txg 高于更早实例的根与记录」。
4. **留存产物与任务 10900 现跑产物**（量，`sha256sum` 与 `wc -l`）：留存 E171.1 130 行 `9573c5ee…`、E171.2 131 行 `2f09f53c…`、E171.3 176 行 `d756ce47…`、E171.4 131 行 `e6d00269…`；任务 10900 现跑 E171.1 133 行 `8e4f9f57…`、E171.2 137 行 `32956b39…`、E171.3 176 行 `40171ae9…`、E171.4 137 行 `bc3985dd…`（调查员草稿目录 `e171-row-reds/E171.<n>.out`，不进仓）。这四份现跑产物是第 1 至 4 行重出时同输入同输出的参照（第六节 V6 的决定性核对）。
5. **探针任务 10980**（量，`task-server/task log 10980`）：冻结目录原样，两条流 `refusal=no_choosable_system_configuration_on_any_device`；常量换 0x06 后两条流 `refusal=None pool=True chosen_root=True ring_scan=True landed=(0,0,wm=65536,floor=0)`。`landed=(0,0,…)` 是 mkfs 基底（里面没有 journal 记录，实验页的锚点 A1 要的 `(1,3)` 在全持久态上）。影响：探针过了只说明系统配置槽读得通，不说明记录头、根记录的宽度对；所以第六节 V2 取全持久态上的锚点，不取基底。
6. **留存产物在工作区的情况**（量，`ls`）：29 个留存产物名里工作区在的 5 份（第 5、12、15、20、21 行；`ls research/results | grep e171` 现数 7 份文件，另 2 份不是复跑表的期望产物），其余 24 份已归档，取法 `git log --all --diff-filter=D --format=%h -n1 -- <路径>` 再 `git show <提交>^:<路径>`。第十一节表里的 sha256 与行数是对这些 git 版本现算的，`wc -l` 计换行。
7. **r1 输入**（量，`ls ~/.local/share/singlefs/work/replay-frozen-roots`）：只有 `e203-segment-two-source`，没有 `e171-r1-dumps`，也没有 `e171-r1-frozen-tree-revision-6`。实验页标题行写 r1 冻结层的输入造不回来（页上的话，我没核它的成因）。影响：第 12、13、15 行预计「跑不了」（推，读驱动：`e171_r1_inputs_present` 缺子路径返回 77）。
8. **实验页第 31 行写「二十五行登记」**（量）：「这一页原来的十一行，加第 1 次重跑九行……加第 2 次重跑五行」；现数 `replay.sh` 第 2306 至 2334 行是 29 行：十一加九加九。第 2 次重跑那一段是九行（第 2326 至 2334 行），页上写五。页上这个数要回改（kb-scribe 的活，第十三节最后一条登记）。
9. **耗时**：任务 10900（报告）E171.1 单行 3 483 秒，其余几行 396 至 556 秒；E171.1 区间日志有 10 行 `GPU_LIFECYCLE`，第 2 至 6 行区间 0 行（报告）。推的复用：后几行同样先跑基线，从判定库取（报告标「没核复用计数」）。驱动文件头的推数（旧树、旧执行器上）：第 9 行约两小时（整表 5 569.6 秒、Q3-核三趟各约两分钟）、第 10 行约四小时、第 11 行约四个半小时（补位 6 行 1 277.9 秒）。第 7、8、12 至 29 行没有这一趟的读数。
10. **套着的三行**（读驱动，量）：`driver_e171_s4_f4_comparator` 第一步 `driver_e171_s4_comparator >/dev/null`，`driver_e171_s4_f4_substitutes_comparator` 第一步 `driver_e171_s4_f4_comparator >/dev/null`，三条共用 `$OUT_DIR/e171-s4-comparator`；外层只留自己的 `comparator-f4.out`、`comparator-f4-substitutes.out`，s4 的 `comparator.out` 也留在同一个草稿目录里（驱动不删）。
11. **我写这份登记前读了调查报告**：第六节的类别规则只用产物里的字段（`expected`、`target_is_red`、`impl_consulted_outcome`、红集），不按报告点名的行号写；点名行号只放在「预言」里，不进判据。报告读到的结局（比如 28 行全是 `not_in`）我无法当作没看见，所以规则写成「`expected=in` 的行变了就停」这种与结局无关的形式，分辨力不靠报告的数。
12. **第 3a 步 ② 分辨率**：V2 的锚点是 `holds=true|false|not_run`，V3 的读数是 0 命中或点名行，V4 是 `recovered=true|false`，都是布尔或计数，没有比值，在对照臂（冻结目录）的基线上不退化（冻结目录上 `recovered=false`、锚点缺）。第六节 C1 至 C5 的差是整行数（分辨率一行）。
13. **第 3a 步 ③ 要触发的事件**：这一次的前提事件是「执行器对全持久态恢复得出」（V2）与「第 5、6 行的 O3 列真有读数」；不会自己发生，靠执行器修好。没有搬空、回收、淘汰、写满这类要触发的事件。
14. **判定库矛盾**（报告，量一次）：上一趟日志里 E171.1 的前一次现跑第二条流 `exported=false`，`not_exported_why` 是 `StoredBytesDoNotMatchTheLayout{…恢复字旁表有这一块这一种结论这一版本，结论块表没有}`，状态 `ordinal=33554509`（`verdict_store.rs` 第 4661 行）；任务 10900 这一趟没复现（`streams_not_exported=0`）。影响：失败条款 F5。

## 五、臂、阳性对照、真实基线

### 5.1 臂（岔路 R1；对面那条臂写成支持它的人认的样子）

**X1 新建目录**（本登记定案，读法 4）。怎么做：拷 487 目录里冻结范围内的文件（不含 `comparator*`、`mutation_driver*`，`executor_main.py` 第 228 行 `freeze_lines` 排除的就是它们）到新目录，按 5.4 改常量与结构。做完会怎样：本实验 O3 一侧（`frozen_o3.py` 的目录常量、`replay.sh` 两处 `cd`、冻结参照）指新目录；第三节第 5 条列的另外十几处引用一个字节不动，E170（O3规约执行器第一版）在 `replay.sh` 的九行（只引两个 487 目录）不受影响。「不受影响」从做法推得出：没有一个被别处引用的文件被改，第六节 V1 用 `sha256sum` 核旧目录文件集不变。

**X2 原目录（冻结目录）原地改加重冻结**（支持者认的样子）：「用户要的是修执行器，不是再养一份分叉；`frozen_o3.py` 文件头已写过先例——冻结目录改过一次（执行器默认全跑）之后重冻结，冻结参照换成 11-05 那一份；这一次照做：改冻结目录里那几个常量，重跑一遍取新的冻结行，两个 `frozen_o3.py` 的参照指新产物，只有一份执行器，以后谁用都是对的那一份，14 处引用不用改路径。」做完会怎样：冻结目录就是跟上今天格式的那一份。

**为什么定 X1**（依据，现查）：① 冻结目录被 14 份文件引（第三节第 5 条），其中 `e170_o3_spec_executor_with_witness_predicates/` 的 `run_segment_two.sh`、`run_segment_three.sh`、`mutation_driver_third_segment.py` 把冻结目录当「第一版」与带见证谓词的那一版并排比；原地改成 496 而带见证谓词的两个目录仍是 487，并排比的两边格式不同，那几处比的就不是同一件事（推，读文件头）。② 变异表 `scripts/mutation/tables/e170_second_run_o3_spec_executor.tsv` 的锚点在冻结目录里，改了要逐条重核锚点（`mutation-sampling.md` 第七类）。③ 实验页第 13 行把 O3 写成「E170 第 1 次重跑冻结的执行器」，改它是换身份，不是修常量。④ 487 目录已经带着 D23（journal 的角色与格式） 已定项 4 正文取 487 的格式补丁，与冻结目录的 `executor_core.py`、`executor_main.py`、`root_ring.py`、`version_table.py`、`system_configuration.py` 逐字相同（第三节第 4 条），从它起步改动面最小。X2 的代价（①②的清单）由 V11 现数，现数和上面推的不符时按 V11 的读数改判，改判写进第十二节修订、算新实验。

### 5.2 阳性对照与阴性对照（每条臂都跑）

- **对 X1**：阴性对照（冻结目录原样）：同一份 dump 上冻结目录的 `base_recovery` 必须是 `recovered=false`、`detail` 点名 `no_choosable_system_configuration_on_any_device`（复现任务 10980 前两行）；不是这样，说明「执行器跟不上格式」这个前提本身错了，整段停（F1）。阳性对照：新目录自己的合成池单测（`test_executor_on_synthetic_pool.py`，改成 496 / 0x06 / 466 之后）全绿，并且第九节 M1 至 M5 每条都让它或 V2 红。
- **对 X2**：不跑（没有第二条要跑的臂；X2 的改动面由 V11 现数，不是跑出来的结局）。X2 要是改判为定案，上面两条对照照样对它跑，不另设。
- **对判定方向的判法（岔路 R2）**：第六节 C1 至 C5 的判法脚本自己要被证红，第九节 M6 至 M8 是它的变异（往合成的产物对里塞已知的「该判有问题」和「该判正常」各一对，脚本必须分出来）。
- **对 29 行**：每行驱动的阳性对照是它原登记里的（O1、O2、O3 三列各自的阳性对照在第 7、9 至 11 行里面，r1 的在第 12 行），同上一次登记；这一次的重出照原登记原样跑，不增不减。

### 5.3 装置写在哪（写死）

默认走 `scripts/experiment-apparatus/`：新执行器目录、两处路径常量、冻结参照产物、判法脚本都是研究一侧的 Python 与 shell，**不改 `crates/`**；不是入库装置，登记里没有「执行员读 `crates/` 常量」的项（原因：这条岔路问的是独立规范跟没跟上格式，另写独立模型能答，不需要读 `crates/` 的这份代码）。`crates/` 的常量只在第十一节停机条款里用 `grep -n` 对一次，不 import、不抄进执行器。第 1 至 11 行、第 20 行原来就走入库装置（`e171_three_oracle_collusion`、`e170_specification_executor_export`、`e171_three_oracle_collusion_after_the_oracle_fixes` 三个 `crates/singlefs-verification/src/bin/` 下的二进制）带 `--features gpu`（`replay.sh` 第 356 行 `CHECKER_TIER_FEATURES=(--features gpu)`），这一次不改它们。

### 5.4 新执行器要改什么（岔路 R1；每一处只许取自 kb，注释里写 `文件@小节标题`，不写 `crates/` 路径；改前读 kb 原文，下面的数是我读 kb 与 `crates/` 常量对出来的）

1. **incompat 首字节**（`system_configuration.py` 第 21 行常量及第 106、204 行两处比较）：位 1 与位 2 都置，首字节 0x06，其余 31 字节全零（`.claude/kb/layout/01-first-txn.md@一、mkfs 已经种下的（新池新建文件之前就在盘上）` 自举头「feature bits」一行；位 n 住第 n div 8 个字节的第 n mod 8 低位，D22（单元原子性怎么合成） 已定项 13）。位 2 的语义是「inode 记录带截断字段」，执行器不解 inode，只当「认得的位」。
2. **根记录**（`on_disk_layout.py` 第 71 行附近的 `ROOT_RECORD_FIELD_ROWS`）：在 `allocation_records_root_pointer`（86）之后、`algorithm_type` 之前插 `reclamation_threshold`（8）与 `pinned_slot`（1），总宽 466（`.claude/kb/layout/01-first-txn.md@七、发布（根记录与根槽）` 字段表，D22 已定项 7）；`algorithm_type`、`nonce`、`message_authentication_code` 的偏移各后移 9。自证校验和覆盖整个 512 字节槽的写法不变。
3. **journal 记录头**（`on_disk_layout.py` 的 `JOURNAL_RECORD_FIELD_ROWS`、`JOURNAL_NEW_ROOT_SEGMENT_BYTES`、`JOURNAL_HEADER_BYTES` 与末尾的断言）：新根段在卸载记号（4）之后加 `new_root_reclamation_threshold`（8）与 `new_root_pinned_slot`（1），段宽 373；`file_system_identity_low_eight_bytes` 在 472、`message_authentication_code` 在 480、头末 496；断言改成 `JOURNAL_HEADER_BYTES + 16 == 512`、`(4096 − 496) // 56 == 64`、`99 + 373 == 472`（`.claude/kb/layout/01-first-txn.md@六、journal` 记录头字段表与「记录头字段表的偏移」一段；D23 已定项 4、已定项 15）。
4. **重建的根**（`replay_prefix.py` 的 `RebuiltRoot` 与 `rebuilt_root_after_applying`）：加回收门槛与钉槽两项，无记录可施加时取所选根的这两个字段，有记录时取记录新根段里的（D23 已定项 15 正文：新根段是「根记录里同一对字段」）。投影（`VersionProjection`）仍只取六样，不加。
5. **点名项上限与余量**（`on_disk_layout.py` 的 `NAMED_ITEMS_IN_A_4096_BYTE_RECORD_AT_MOST`、注释里「余 25 字节」）：上限仍 64，余量改 16。
6. **合成池与单测**（`synthetic_pool_for_tests.py`、`test_executor_on_synthetic_pool.py`）：造 466 字节根、496 字节头、0x06；单测里写死的 457、487、311、364、463、471、25 逐个换成上面的数（加法写法：`assert 头 == 99 + 373 + 8 + 16`，不写减法）；**加取样点**：根与记录里 `reclamation_threshold ≠ 0`、`pinned_slot ≠ 255` 的合成态（真基线上 t = 0、p = 255，只在真 dump 上量，这两个字段读错也看不出，见第九节 M4）；记录装满 64 个点名项的一条（496 + 64 × 56 = 4080 ≤ 4096）与 65 个的一条（必须拒）。
7. **空缺登记**（`spec-gaps.tsv`、`spec_gaps.py`）：回收门槛 t 与钉槽 p 的语义（I-7.18（根记录的回收门槛等于现算的门槛）、I-7.19（钉着的槽不住本实例写的根））执行器**不实现**，只读出、进重建的根；要是 D22 / D23 / 不变量文里有哪句让「择根」或「恢复结局」依赖 t 或 p，登记成新的一行空缺（编号接 G23 之后，列照现有 23 行的列），触发条件写执行器怎么记撞上，不暗中取一个读法（读法 11 那一类「格式补丁」不再沿用，kb 今天已写明的就按 kb 写）。
8. **不动**：`executor_core.py`、`executor_main.py`（含第 147 行起的锚点断言，锚点值取自 kb 表，不为通过而改）、`root_ring.py`、`version_table.py`、`crash_dump.py`、`crash_image.py`、`crc32c_castagnoli.py` 与 487 目录逐字相同（V1）。`root_ring.py` 第 107 行 `len(raw) < ROOT_RECORD_BYTES` 跟常量走，我推它不用改；要改，按 F2 停。
9. **注释只写现状**：拷来的注释里「格式补丁（…登记 12.5 格…）」「读成腐烂」「按读法 11」这类历史说法，在新目录里改成现状句（`agent-common.md`「代码与脚本的注释、说明只写现状」；`.claude/gate.d/comment-history-lag.tsv` 第 38、39 行已为两个旧目录的 `spec_gaps.py` 记了存量，新目录不往里加新行）。
10. **接上本实验**：`scripts/experiment-apparatus/e171_three_oracle_collusion/frozen_o3.py` 第 43、44 行（目录、冻结参照）与文件头说明、`replay.sh` 第 3050 行 `E171_FREEZE_REFERENCE`（开跑前核对的冻结参照）与第 3144、3213 行两处 `cd`，指新目录与新冻结参照。新冻结参照 = 新执行器在 V2 那份 dump 上现跑的产物（`name=freeze file=…` 逐文件加 `name=freeze_seed`），落 `research/results/`，文件名带 `-e171-r3`；盲测族种子 `85441ae491419411` 写死在各驱动与复跑里，不随重冻结变，不改。`after_the_oracle_fixes` 那一支的 `frozen_o3.py`（第 41、42 行）一个字节不动。
11. **变异表锚点**：`scripts/mutation/tables/e171_three_oracle_collusion.tsv` 若有锚点落在改到的文件里（`frozen_o3.py`、`o3_bad_image_judge.py`、`replay.sh`），改完逐条核仍恰命中一次（`mutation-sampling.md` 第七类）。

### 5.5 分段（每段交回后主 agent 对着岔路单判续不续）

- **第一段（补岔路 R1，并让 R2 在第 1 至 4 行上有材料）**：5.4 的改动、V1 至 V5、V11、新冻结参照、两处路径；然后重出第 1 至 4 行（不碰 O3，便宜）与第 5、6 行（O3 的两条探针行）。补完 R1 够不够判：V1 至 V5 全有读数、第 5、6 行 O3 列有读数就够；V5 记「本次未跑」的，主 agent 定是否以 V2 至 V4 代之。R2 对第 1 至 4 行够判（C1 至 C4 的材料齐），对第 9 至 11 行还差。
- **第二段（补 R2 的第 9 至 11 行、R3 的第 7 至 11 行）**：第 7、8 行，然后第 9、10、11 行（套着的三条，第十四节）。判完 R2 在三大张整表上各类的结局。
- **第三段（补 R3 的第 12 至 29 行、R4）**：第 12、13、15 行（预计跑不了，先把缺的输入点名）、第 14、16 至 19 行（研究一侧变异）、第 20 至 29 行；V9。
- **行级点名**：`replay.sh` 的可点名项是实验号（文件头第 10 行起：「可点名的项是复跑表的实验号」；选项只认 `--admission-only`、`--selftest`、`--list-items`），`E171` 一次点到 29 行全部，没有点单行的选项（`grep -n -- '--row\b' scripts/experiment-apparatus/replay.sh` 的三处命中是 O3 对照的 `--row-name`，不是 `replay.sh` 的选项；写这一条时现查）。所以「第一段只出第 1 至 6 行」要么靠执行员给 `replay.sh` 加一个点名单行的选项（带自证，改 `replay.sh`），要么整条 `E171` 跑、第 7 行开跑后取消（任务 10900 就是这样），要么第 1 至 6 行之外的驱动不经 `replay.sh`、用第十四节写的单驱动命令。选哪个见未定项 4。
- 每段交回附「已够判 / 还差什么 / 剩下的量能不能让它翻面」一张表。一行开着的都不剩就停，后面的段不跑。

### 5.6 与 E201（三oracle合谋今树整表）的关系（岔路 R4；不改 E201 的登记）

- **问的不是同一件事**：这一次问「复跑表这 29 行今天还出不出、出的与旧产物差在哪」，读数属于每行开跑那一刻的树（实验页第 1349 行：「复跑一次出的是复跑那一刻那棵树的读数」）；E201 问「今天的树上两张名单 278 行整表逐行过三个 oracle，与 r1 逐格比」，用它自己的装置、自己的比法（`research/prompts/e201-preregistration.md` 第一节读法 6、7）。
- **谁都不替谁判**：这一次的任何一行读数不当 E201 的 Q1、Q2 的答案引；E201 将来的读数也不拿来判这 29 行是「同」还是「变」。两边引同一棵树的同一类读数（例：第 20 行的流段长、第 21 行的名单核对）时，各写各的产物名、各带自己的 `git rev-parse HEAD`（Z4），不合并。
- **这一次不动 E201 的任何输入**：不改 E201 登记；不碰它读的四份旧产物（P1、P2、P3 与名单核对产物，V9 核 sha256）；不碰它点名的带见证谓词的 487 目录。新执行器是另一个目录（读法 4），所以 E201 的 O3 一侧**没有被这次修好**——它的目录与冻结目录的 `system_configuration.py` 第 21 行同为 `1 << 1`（第三节第 9 条，`sed -n 21p` 现读），按这一次的 V2 的同一条推法，E201 起 O3 会遇到同一句 `no_choosable_system_configuration_on_any_device`（推，没跑）。修不修、怎么修（照这一次的 5.4 在它那一支再建一个目录，还是别的）要 E201 自己的修订，交主 agent。
- **E170 的九行**（`replay.sh` 第 2297 至 2305 行）也只引两个 487 目录，同一原因下它们对今天的树大概率同样跑不出 O3（推，没跑）；不在这一批，这一次也不改它们。

## 六、报哪些量与各自的判据

门槛都是跑前写死的；每个量一行，不合成真假；对应岔路 R1 至 R5（第一节）。

**V1（R1）新目录的改动面**：`diff -rq <487 目录> <新目录>`（排除 `__pycache__`）列出的文件集合；另对 `executor_core.py`、`executor_main.py`、`root_ring.py`、`version_table.py`、`crash_dump.py`、`crash_image.py`、`crc32c_castagnoli.py` 七份报两边 `sha256sum`；对旧三个目录（冻结目录、两个 487 目录）报改前改后的整目录文件 sha256 清单，改前改后逐字相同。门槛：改动文件 ⊆ {`on_disk_layout.py`、`system_configuration.py`、`replay_prefix.py`、`journal_ring.py`、`spec_gaps.py`、`spec-gaps.tsv`、`synthetic_pool_for_tests.py`、`test_executor_on_synthetic_pool.py`}；七份 sha256 相等；旧三个目录不变。判定：多出一个文件，或七份有一份不等，或旧目录变了 → X1 不只是格式常量，F2。翻面：R1 里「从 487 目录起步改动面最小」这句不成立。

**V2（R1）恢复与锚点**：新执行器对「E170 导出装置 `dump` 模式在今天的树上现导的目录」两条流（`new_pool_file_creation`、`fixed_script_through_e_then_unmount`）的全量输出（命令照 `replay.sh` 第 3144 行：`python3 executor_main.py --dumps <导出> --workers 6 --output <新文件>`，线程由任务服务定）。读：`name=base_recovery` 无 `recovered=false`；`name=anchor_assertion` 的 A1 至 A6（流一）与 A6、A11（流二）逐条 `holds=true`，**`holds=not_run` 算不过**；`name=dump_check` 的 `count_mismatches`、`checksum_mismatches`、`version_table_anomalies` 全 0；`name=torn_reading` 的 `differing_entries=0`。门槛：全部成立。判定：不成立 → 执行器没跟上（超出 5.4 的那一类结构差，F1）。同一条命令对冻结目录跑一次作阴性对照（5.2）。翻面：R1「O3 跟上了盘上格式」。

**V3（R1）独立性，五条，各占一行，各报各的**（读法：执行器是不读 `crates/` 的独立规范，规范来源只有 `.claude/kb/`）：
- I1 `grep -n 'crates' <新目录>/*.py` 的命中行与 487 目录同一命令的命中行逐行相同（487 目录今天两行：`test_executor_on_synthetic_pool.py` 第 92 行的类别串「只有读 crates 才写得下去」、`crc32c_castagnoli.py` 第 70 行注释「与 crates/ 无关」）；多一行 → 红。
- I2 对新目录每个 `.py` 做 `ast` 遍历：`import` 目标只许是标准库、新目录内的同级模块、`google_crc32c`（可选，`crc32c_castagnoli.py` 第 70 行）；`open(` 的路径只许来自命令行参数或 `os.path.join(<参数>, "<dump 里的固定文件名>")`（`manifest.tsv`、`base-sectors.bin`、`ops-payload.bin`、`torn-payload.bin`）与合成池的临时目录；出现别的路径常量 → 红。
- I3 **离仓运行**：把新目录整个拷到仓外的草稿目录（不带 `scripts/`、`crates/`、`.claude/` 任何东西），`cd` 进去、`PYTHONPATH` 清空，对同一份 dump 跑 V2 的命令，产物逐行与仓内跑相同（`name=config` 行里的路径与耗时字段除外）；读了仓里的东西就跑不起来或产物不同 → 红。
- I4 新目录每一处改动的注释都带 `.claude/kb/` 下的出处（`文件@小节标题`，标题逐字），出处只许是 kb；`grep -n 'crates/' ` 出现在新写的注释里 → 红。
- I5 阴性：往新目录的一份草稿拷贝里加一行 `open("crates/singlefs-format/src/lib.rs")`，I1 与 I2 必须都红（证检查分得出）；不红，I1、I2 作废重写。
判定：任一红 → 新执行器作废，这一次 O3 读数不采（F3）。翻面：R1 的「仍是独立规范」。

**V4（R1）种下的故障必须让 V2 红**（草稿拷贝里种，不进仓）：P-a incompat 首字节改回 0x02；P-b `JOURNAL_HEADER_BYTES` 改回 487（`fsid` 463、MAC 471）；P-c `ROOT_RECORD_BYTES` 改回 457（删 t、p 两行）。门槛：三条各自让 V2 变红，**红在点名的格**：P-a 两条流 `base_recovery recovered=false`；P-b 流一 A3 `holds=false` 或 `name=dump_check` 的 `checksum_mismatches` 不为 0；P-c 流一 A2 `holds=false`（根不自证）。哪条不红 → 该处检查分不出差，F1。

**V5（R1）逐态判定相同**：冻结目录（311 / 0x02 / 457）与新目录（496 / 0x06 / 466）各自用自己单测里的合成池（同一段抽象历史、同一组取样态）判，逐态比 `(verdict, landed 投影)`。门槛：全部相同。判定：有一态不同 → 改动不止于格式（F2）。没量过两边的合成池是不是同一段抽象历史、能不能直接对拍：做不到的，执行员在第十二节修订里写缺什么，这一格记「本次未跑」，不当通过。

**V6（R2）判定方向变化，逐行判**：对第 1 至 4 行、第 9 至 11 行（以及第 7 行里出现的同类）重出产物与旧产物按键（记录行种加 `row`、`stream`、`name`、条目号）配对，每一对按下面五类判，**每一行的类、读到的 `expected` / `target_is_red` / 红集前后、结论都进产物**，不只报计数。
- **C1 放行变红**（旧红集空或不含某项，新红集多出一项或以上，且旧时 O2 放行）：正常 = 该行 `expected=not_in`（坏状态行）或属已知坏镜像条目（本身就是坏的），且新增的每一项都是 `crates/singlefs-checker/invariants/` 下有定义行的编号；有问题 = `expected=in` 的行出现任何新红（假阳性）。
- **C2 原来就红、多红一项**：同 C1 的正常条件；`expected=in` 的行红集多出任何一项 = 有问题。（报告里 E171.3 流二 row 13 `N1-F` 多 I-3.21：它的 `expected` 我没读，若是 `in` 这条规则会停，预言，没量过。）
- **C3 读成功变实现拒**（`impl_consulted_outcome` 由 `file_read` 变 `failed:*`）：正常 = `expected=not_in` 且错误种类、`detail` 句在 `crates/singlefs-core/src/recovery.rs` 现行的拒绝句里（执行员列出不同的 `detail` 句及各自行数，每句 `grep -n` 到出处）；有问题 = `expected=in` 的行变 `failed:*`（产品拒了该读出的），或空变异、干净基线任何一行 `failed:*`，或反方向：`not_in` 行由 `failed:*` 变 `file_read`（产品放宽了）；归不到 = `detail` 句 `grep` 不到。
- **C4 红变不红**（旧红集有、新红集无的不变量 X；条目的目标不变量 T）：先要求 T 仍红（`target_is_red=true`）；T 不红 → 有问题（检查器真窄了）。T 仍红时，正常 = 同时满足 (i) X 自己作目标的已知坏镜像条目今天仍 `target_is_red=true`；(ii) 执行员给那条镜像打印 X 的判定细节（检查器给的原因）并列出 X 的定义文字今天与旧产物所在提交的差（`git log -p -- crates/singlefs-checker/invariants/<文件>.md`）；(iii) 写出「按今天的定义，该镜像的补丁没碰 X 要求的东西」的逐步推导，**主 agent 过目才算正常**，执行员自己不下「正常」。(i) 不成立 → 有问题；(i) 成立而 (ii)(iii) 做不出 → 归不到，交主 agent，不算正常也不算问题。预言（没量过）：I-3.1 有自己的条目，I-3.11 有没有自己的条目我没查，没有则 (i) 无从谈起、落「归不到」。
- **C5 其余漂移**：照调查报告同一份分类表（字段改名、流的段数与读数、撕裂扫描段号、文件摘要、基底读数、落点水位、只差摘要、已知坏镜像钉值 / 只差序号 / 新增条目 / 补丁数）：正常 = 落进这些类，且这些类的行里没有任何一个判定词、`target_is_red`、红集的变化；一行落不进任何类 → 归不到。分类后各类行数之和必须等于 `diff` 行数（差一行都停）。
- **结果反过来时**：有问题的行在重出产物里出现 → 停下，原样列出，交主 agent，不改规则、不换产物重跑、不把它改判成正常；归不到的同样列出交主 agent。规则判「正常」的行，若第二次跑（同输入）判法不同 → 判法不稳，停。
- **决定性核对**：第 1 至 4 行重出产物与任务 10900 现跑产物（第四节第 4 条）逐行 `diff`：这四行不经 O3，应逐字相同（预言，没量过；不同 → 现跑不确定，另报）。
- 门槛的写法与分辨率：每类是整行计数，分辨率一行；类别判定不用比值。翻面：R2 的「有问题」「归不到」行数 > 0。

**V7（R3）每行结局**：29 行逐行报四选一：`逐字节同`（与旧产物 sha256 相等）、`判定词不变·钉值变`、`判定词变`（同第 2 次重跑读法 2 的判定词与钉值分法）、`跑不了`（`replay.sh` 退 77 或驱动返回非 0，点名缺的输入）。门槛：每行有一个结局；`判定词变` 的行逐格列出（行种、键、字段、旧值、新值），一格不漏；`跑不了` 的行只许是第 12、13、15 行（预言：读驱动 `e171_r1_inputs_present`，没量过），别的行跑不了 → F7。`逐字节同` 的预言：第 22 至 24 行读 git 对象与名单核对产物，不读今天的 `crates/`（读驱动，没量过）。
**V8（R3）重出不覆盖**：29 个重跑产物名与旧产物名无一相同；`git status --short research/results` 里没有旧产物被改；失败 = F8。

**V9（R4）E201 不动**：`sha256sum research/prompts/e201-preregistration.md` 改前改后相同；E201 登记第一节读法 2 点名的 P1、P2、P3（`…-s2-comparator-r6.out`、`…-s2-rows-driver-r6.out`、`…-s1-comparator-resume2-r6-r3-2026-10-07.out`）、名单核对产物 `…-list-check-2026-11-01.out` 的 sha256 改前改后相同；`e171_three_oracle_collusion_after_the_oracle_fixes/frozen_o3.py` 与带见证谓词的 487 目录改前改后相同。不相同 → 越界，F9。判定：「互不替代」的读数。
**V10（R5，附带）每行挂钟与复用**：每行 `现判 / 复用` 与墙钟秒（在驱动进程里或任务日志里计，不在转发输出的循环里打时间戳，`test-discipline.md`「分段计时在被测进程里计」）。够判（给主 agent 定次序）后不再为它多跑。
**V11（R1）X2 的改动面现数**：`grep -rnE 'e170_second_run_o3_spec_executor([^_A-Za-z0-9]|$)'` 的文件数（第三节第 5 条：14 份）与 `scripts/mutation/tables/e170_second_run_o3_spec_executor.tsv` 的锚点数及落在 5.4 要改的行里的条数。附带，够判 R1 后不跑。

## 七、钉绝对值的断言

两类分开写。每条互比（新旧目录并排、重出与旧产物并排）旁边都有一条钉死的绝对值，「三条臂一起错」时靠这些发现。

**甲类：出自 kb 条款本身的**（不符时走第十节 F10「条款可能错」，不作废）：
- A-1 incompat 首字节 = 0x06（`.claude/kb/layout/01-first-txn.md` 第 113 行）；
- A-2 journal 记录头 = 496，其中 十个字段 78 + 四笔增量 21 + 新根段 373 + fsid 8 + MAC 16（`.claude/kb/decisions/23-journal的角色与格式.md@已定项 4：记录头约束在单个原子单元内，头 496 字节`）；
- A-3 根记录字段表 17 项，总宽 466（`.claude/kb/layout/01-first-txn.md@七、发布（根记录与根槽）` 表）；
- A-4 实验页第 31 行「二十五行」对现数 29（第四节第 8 条）：页上落后，kb-scribe 回改；
- A-5 锚点 A1 至 A6、A11 的值（`executor_main.py` 第 147 至 190 行；来源按第 148 行文档串是 E170 登记第七节，不是 kb 条款原文，所以这一条挪到下面乙类的「不符才作废」一栏读：不符 = 执行器没恢复对，走 F1）：根环四个根 `(区域, 槽, 盘, 实例, txg)` 为 `(0,0,0,0,0)、(0,1,0,1,3)、(1,0,1,1,1)、(2,0,0,1,2)`，环内三条 `(0,(1,1),1)、(4096,(1,2),2)、(8192,(1,3),3)`，选中槽 `(5,1,3,1)`。

**乙类：独立算出、用命令核过的**（不符才作废）。下面这条命令我写这份登记时现跑过（只读的 `python3 -c`，算术）：`python3 -c "print(78+21+373+8+16, 4+16+4+4+8+86+8+8+32+86+86+86+8+1+1+12+16, 86+86+8+8+86+86+4+8+1, 99+373, (4096-496)//56, 496+64*56, 496+65*56, 512-496, (1<<1)|(1<<2))"`，原样输出 `496 466 373 472 64 4080 4136 16 6`。逐项：
- B-1 头 496 = 78 + 21 + 373 + 8 + 16；B-2 根记录 466 = 457 + 9（t 8、p 1）；B-3 新根段 373 = 364 + 9；B-4 fsid 偏移 472 = 99 + 373，MAC 偏移 480，头末 496；B-5 点名项上限 `(4096 − 496) // 56 = 64`，装满 64 项的记录 4080 字节 ≤ 4096，65 项 4136 字节 > 4096（必须拒）；B-6 头之后余 16 字节；B-7 incompat 位 1 与位 2 = 6。
- B-8 这些数在新目录里**写成加法**（`assert 头 == 99 + 373 + 8 + 16`），不写减法（`test-discipline.md`「变异测试证明的是断言会红，不是覆盖」第 1 条）。
- B-9 重出产物行数、sha256：29 个重跑产物每个报 `sha256sum` 与 `wc -l`，由命令数，不手写。
- B-10 上面 A-5 的锚点值：不符 = 执行器恢复得不对（F1），不是条款错；锚点值不为通过而改（V1 的 `executor_main.py` sha256 钉住）。

## 八、轨迹与几何敏感性

**轨迹**：这一次没有轮次、没有吃光的初始供给、没有停机或起跑谓词吃一个会随轮变的量；谓词（第六节 C1 至 C4）吃的是逐行的标签（`expected`、`target_is_red`、`impl_consulted_outcome`、红集），所以要求报**逐行**读数而不只报计数（V6 末尾那一条），不适用「峰值、为正轮数、期末值」三样，也不在正文里用「从来」「恒」这类说轨迹的词。

**几何敏感性**（至少一个方向相反的取样点，翻面记「不稳定」、不翻记两点）：
- 格式一侧：真 dump 两条流，一条没有卸载（流一，根 `flags` 位 0 = 0）、一条以卸载收尾（流二，A11 要求位 0 = 1）；合成池的根环每区槽数 S 取 kb 区间的两条边 4 与 16（`ROOT_RING_SLOTS_PER_REGION_MINIMUM = 4`、`ROOT_RING_SLOTS_PER_REGION_MAXIMUM = 16`，`.claude/kb/layout/01-first-txn.md` 几何一行）；记录点名项数取 0 与 64 与 65（B-5）。V2、V5 在这几个点上各报一行，有一点翻面记「不稳定」。
- 判法一侧：V6 在三张不同的语料上各自独立判：坏状态行（`expected` 有 `in` 也有 `not_in`）、已知坏镜像条目、整表坏实现行（第 9 至 11 行）；一类规则在一张语料上「正常」、在另一张上「有问题」不记矛盾，记各自的结局。
- 判别力自证：把 C4 的 (i)、C1 的「`expected=in`」条件各挪到「放行」与「拦下」之间的一对合成产物上，判法脚本必须由「正常」转「有问题」（第九节 M6、M7）。

## 九、变异

给执行员列的变异，写在新目录的草稿拷贝或判法脚本上，不进仓；每条写在哪个取样点上改变输出，不改变输出的取样点另列，不当「抓到」：

| 变异 | 改什么 | 改变输出的取样点 | 在哪些取样点上不改变输出 |
|---|---|---|---|
| M1 | `system_configuration.py` 第 21 行 0x06 改回 0x02 | V2 两条流 `base_recovery`：`recovered=true` 变 `false` | 无（任何真 dump 都敏感） |
| M2 | `JOURNAL_HEADER_BYTES` 496 改 487，fsid / MAC 偏移改回 463 / 471 | V2 流一 A3、`dump_check` 的 `checksum_mismatches`（记录头校验和读不对）；合成池装满 64 项的一条 | 空 journal 的 mkfs 基底（里面一条记录也没有，探针任务 10980 就是这个点） |
| M3 | `ROOT_RECORD_BYTES` 466 改 457（删 t、p 两行） | V2 流一 A2（根不自证）；合成池根记录 | 同上，mkfs 基底的根 |
| M4 | `RebuiltRoot` 不取记录里的回收门槛与钉槽（写死 0 与 255） | 只在合成态 `t ≠ 0`、`p ≠ 255` 上（单测）；真 dump 的基线 t = 0、p = 255 不敏感，**不能用真 dump 抓它** | 真 dump 两条流全部取样态 |
| M5 | 点名项上限判断 `> 64` 改 `> 65`（或 `>=`） | 合成池 64 项的一条（边界）与 65 项的一条（必须拒） | 真 dump 里每条记录项数远小于 64（12 项一条） |
| M6 | 判法脚本：`expected=in` 的行新增红也判「正常」 | 合成产物对：一行 `expected=in` 红集多出 I-8.14 | `expected=not_in` 的行 |
| M7 | 判法脚本：C4 不要求目标不变量仍红 | 合成产物对：目标不变量由红变不红、另一不变量也少红 | 目标仍红的对 |
| M8 | 判法脚本：C3 只看 `file_read→failed`，不看 `failed→file_read` | 合成产物对：`not_in` 行 `failed:*` 变 `file_read` | 方向相同的对 |
| M9 | V3 的 I1：把 grep 的命中集合写成「不比较」 | I5 的草稿拷贝（加了 `open("crates/…")` 一行）必须由绿转红 | 干净的新目录（本来就绿） |
| M10 | 新目录 `executor_main.py` 第 147 至 190 行任一锚点值改动 | V1 七份 sha256 中 `executor_main.py` 一份不等 | 无（V1 在任何树上敏感） |

M4、M5 是第三类（取样点不敏感）的预防：真 dump 上这两条抓不到，所以 5.4 第 6 条要求加合成取样点，缺一个这两条就「没红」，没红的先按 `mutation-sampling.md`「没红的三类」分，不记等价。

## 十、失败条款

每条紧跟「什么观测会让它触发」。作废的是这一段（阳性对照 / 独立性不过），合法结果是（判法落在「有问题」「归不到」）。

- **F1 执行器没修成，或检查分不出差别**（整段作废，不跑需要 O3 的第 5 行以后）：触发 = V2 在新目录上有任一 `holds=false`、`holds=not_run`、`recovered=false`；或 V4 三条种下的故障有一条没让 V2 红；或阴性对照（冻结目录）`recovered=true`。处置：不当「产品写错盘」（报告：真 dump 的系统配置槽自证全过）；把 dump 的字节对 kb 字段表再读一遍，列出对不上的字段，交主 agent。
- **F2 改动越出格式常量**：触发 = V1 多出一个文件、七份 sha256 有一份不等、V5 有一态判定不同、或 5.4 第 8 条的 `root_ring.py` 要改。处置：记一次、停，交主 agent；不自己扩任务。
- **F3 独立性破**（执行器作废，O3 读数不采）：触发 = V3 的 I1 至 I4 有一条红，或 I5 的阴性没让 I1、I2 红。
- **F4 判定方向「有问题」**（合法结果，不作废）：触发 = C1、C2、C3 判出「有问题」，或 C4 的 T 不红。处置：原样列出，停，交主 agent；不改规则、不换产物重跑。
- **F5 判定库矛盾再现**：触发 = 重出产物里某条流 `exported=false` 且 `not_exported_why` 含 `StoredBytesDoNotMatchTheLayout`，或 `streams_not_exported > 0`。处置：该行记「本次未判·判定库」，不算产品问题、不进 C 类、不重判；交主 agent 按调查报告「要主 agent 跑的命令」第 2 条读那一把键的状态。
- **F6 预言「逐字节同」的行判定词变**：触发 = 第 1 至 4 行与任务 10900 现跑产物不逐字相同，或第 22 至 24 行与旧产物不逐字节同。处置：记「判定变」，归因到驱动、比对器或输入哪一处变过（`git log` 取），不当作废。
- **F7 跑不了超出预言**：触发 = 第 12、13、15 行之外有行退 77 或非 0。处置：点名缺的输入或报错，交主 agent。
- **F8 覆盖了旧产物**（作废那一行）：触发 = V8 的 `git status --short research/results` 里有旧产物被改，或重跑产物名与旧产物名相同。
- **F9 越界 E201**（停）：触发 = V9 任一项不相同。
- **F10 条款可能错**：触发 = 甲类 A-1 至 A-5 与现读不符（kb 说 0x06 而 `crates/` 常量不是 0x06；实验页数对不上 29）。处置：两边都查（`implementation-first.md` 第 4 条），不默认哪边对；实验页数落后的回改交 kb-scribe，不停实验。

## 十一、作废条款与停机条款

**作废条款**（作废的是这一段或这一行，不是结论）：
- Z1 阳性 / 阴性对照不过（F1）或独立性不过（F3）：新执行器作废，第 5 至 11 行的 O3 读数不采；执行器重做是新一轮（改了就是新实验）。
- Z2 开跑之后改判据（第六节 C1 至 C5 的规则、V1 至 V11 的门槛）：改了就是新实验，重跑；合法的只有第十二节「只许收严或补臂」。
- Z3 规模变量（种子数、展开深度、`--rows` 这类）被设过：这一趟不当正式产物（`test-script-discipline.md`「入口脚本：不带参数就是不指定范围」）。
- Z4 每个重出产物头写 `git rev-parse HEAD` 与 `git status --short` 的 sha256；判据读数要跨两行比较（V6 的配对、V9）而两行的树不同时，该读数标「跨树」、不进 V6 的判定。
- Z5 旧产物被覆盖（F8）：作废那一行。

**停机条款**（装置与 `crates/` 实现对不上，`implementation-first.md` 第 4 条：两边都查，既不作废也不当结果）：
- S1 新执行器改完后，用 `grep -n` 对一遍：`system_configuration.py` 的 incompat 常量（应为 6）对 `crates/singlefs-core/src/system_configuration.rs` 第 33 行起的位 1 与位 2；`ROOT_RECORD_BYTES` 466 对 `crates/singlefs-format/src/lib.rs` 第 234 行；新根段 373 对第 249 行；头 496 对第 252 行。四处任一不等：先看 kb（第 113 行、第 360 行、根记录表），kb 与 `crates/` 相等而执行器不等 = 执行器没改对，回 5.4；kb 与 `crates/` 不等 = 条款可能错（F10），停，不改执行器去贴任何一边，交主 agent。
- S2 V2 在今天的真 dump 上不过、而 S1 四处全等：说明字段顺序或语义上还有我没列的差（5.4 之外），停，把对不上的字段列出（用真 dump 的字节对 kb 字段表），交主 agent；不自己扩改动面。
- S3 重出产物与 `crates/` 的现行拒绝句对不上（C3 的 `detail` 句 `grep` 不到）：停。

**够判停机**：岔路单每一行都够判就停，没跑的量逐条标「够判后未跑」。预计会出现的：V11、V10（附带）；第三段的第 14、16 至 19、25 至 29 行（研究一侧变异，不改变 R1 至 R4 任何一行）在主 agent 判 R3 的「跑不了」那一格只需要第 12、13、15 行时，可以标「够判后未跑」，但本登记默认 29 行全跑（读法 1），不跑要主 agent 在段间说。

**登记表归执行员**（`.claude/gate.d/replay-comparison-units.tsv` 文件头）：E171 没有单元表，下面表里 29 行在该文件里各要一行「# 不登记的比对对象：research/prompts/e171-r3-prereg.md #<n>：<理由>；E171 没有单元表，没有单元可登记；欠在 C951（E171 重跑的比对对象没有单元表）」，写法照 r2 的 #1 至 #7 七行（该文件第 38 至 44 行）；这一条设计员不做（写范围外），执行员在登记落地的同一次改动里做。产物落盘后，`replay.sh` 第 2306 至 2334 行第 4 列改指新产物名也是执行员的事（旧产物留在 git，E201 读其中几份，V9）。

比对对象表（一行一个正式跑；旧产物的 sha256 与行数是对工作区文件或 git 里删它那次提交的上一版现算的，`sha256sum`、`wc -l`，我没打开内容；驱动与行号对 `replay.sh` 第 2306 至 2334 行；备注列依次是：驱动类别（A 本机 gpu cargo 入库装置，B 研究一侧 Python，C 读 git 对象，D 读 r1 导出根）、与新执行器的关系、旧产物在哪、预言）：

| # | 比对对象 | sha256 | 行数 | `replay.sh` 行 / 跑法 | 重跑产物名 | 备注 |
|---|---|---|---|---|---|---|
| 1 | `e171-three-oracle-collusion-2026-10-03-s1-baseline-run1.out` | 9573c5ee19ecda3c60ae4ef62ffe2261acebff920eb3e8cb2d9456af9f424782 | 130 | `replay.sh` 第 2306 行 `driver_e171_s1_baseline` | `e171-three-oracle-collusion-2026-10-03-s1-baseline-run1-e171-r3.out` | A 本机 gpu cargo；不碰 O3；旧产物在git 提交 `59ce5a865^` 里；量过：任务 10900 判定词同、133 行 |
| 2 | `e171-three-oracle-collusion-2026-10-03-s1-known-bad-images.out` | 2f09f53cab926d97fdac699b86ebf8620e3a12ab3d39e7de9c3ba5450789bcd4 | 131 | `replay.sh` 第 2307 行 `driver_e171_s1_known_bad_images` | `e171-three-oracle-collusion-2026-10-03-s1-known-bad-images-e171-r3.out` | A 本机 gpu cargo；不碰 O3；旧产物在git 提交 `59ce5a865^` 里；量过：任务 10900 判定词同、137 行 |
| 3 | `e171-three-oracle-collusion-2026-10-03-s2-bad-states-export.out` | d756ce47c0ced1bfa8ad4a3c44acd22f0a3b94e1634e8cf278b65e4626e7d65d | 176 | `replay.sh` 第 2308 行 `driver_e171_s2_bad_states_export` | `e171-three-oracle-collusion-2026-10-03-s2-bad-states-export-e171-r3.out` | A 本机 gpu cargo；不碰 O3；旧产物在git 提交 `59ce5a865^` 里；量过：任务 10900 判定词同、176 行 |
| 4 | `e171-three-oracle-collusion-2026-10-03-s2-known-bad-images.out` | e6d0026957aca83888a6b2a2b069fdd6766d452bead1f9a9d99165c328ff9f29 | 131 | `replay.sh` 第 2309 行 `driver_e171_s2_known_bad_images` | `e171-three-oracle-collusion-2026-10-03-s2-known-bad-images-e171-r3.out` | A 本机 gpu cargo；不碰 O3；旧产物在git 提交 `59ce5a865^` 里；量过：任务 10900 判定词同、137 行 |
| 5 | `e171-three-oracle-collusion-2026-10-03-s2-images-o3.out` | a1b1eea0aea87a60422abd99c7664fb2ec64475ee1cdd4a83e0fa7aa572223e1 | 132 | `replay.sh` 第 2310 行 `driver_e171_s2_images_o3` | `e171-three-oracle-collusion-2026-10-03-s2-images-o3-e171-r3.out` | A 本机 gpu cargo；用新执行器；旧产物在工作区；没量过：任务 10900 败在执行器 |
| 6 | `e171-three-oracle-collusion-2026-10-03-s2-o3-control-baseline.out` | 2c9e324bd9e342a0763027f8db1aa1851de6c72ffb3882ce8c8104e687a8e8a7 | 4 | `replay.sh` 第 2311 行 `driver_e171_s2_o3_control_baseline` | `e171-three-oracle-collusion-2026-10-03-s2-o3-control-baseline-e171-r3.out` | A 本机 gpu cargo；用新执行器；旧产物在git 提交 `59ce5a865^` 里；没量过：任务 10900 败在执行器 |
| 7 | `e171-three-oracle-collusion-2026-10-03-s2-comparator.out` | c9108a4bb44bfc7b4fbfdf98b7b525e9312fed057c256462428f8e4880930b8a | 381 | `replay.sh` 第 2312 行 `driver_e171_s2_comparator` | `e171-three-oracle-collusion-2026-10-03-s2-comparator-e171-r3.out` | A 本机 gpu cargo；用新执行器；旧产物在git 提交 `59ce5a865^` 里；没量过 |
| 8 | `e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03-r7.out` | fe4652fffa987d99ea984fc4cb8c8d408d29bbe62f2ab8e549415ea2380a2837 | 138 | `replay.sh` 第 2313 行 `driver_e171_s3_comparator_mutations` | `e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03-r7-e171-r3.out` | B 研究一侧 Python 变异；读新冻结参照；旧产物在git 提交 `59ce5a865^` 里；没量过 |
| 9 | `e171-three-oracle-collusion-2026-10-03-s4-comparator.out` | a553865382b7895cfa22af6ca20844992fe65573b8bf456a352db9f57512824e | 2119 | `replay.sh` 第 2314 行 `driver_e171_s4_comparator` | `e171-three-oracle-collusion-2026-10-03-s4-comparator-e171-r3.out` | A 本机 gpu cargo；用新执行器；旧产物在git 提交 `59ce5a865^` 里；没量过；套在第 10、11 行里面 |
| 10 | `e171-three-oracle-collusion-2026-10-03-s4-f4-comparator.out` | e8660ccef12a13f76409a1b6f24ebd9a75444249ad65d5617551ac59c1e3c40e | 4698 | `replay.sh` 第 2315 行 `driver_e171_s4_f4_comparator` | `e171-three-oracle-collusion-2026-10-03-s4-f4-comparator-e171-r3.out` | A 本机 gpu cargo；用新执行器；旧产物在git 提交 `59ce5a865^` 里；没量过；先整条跑第 9 行，套在第 11 行里面 |
| 11 | `e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator.out` | 0f439cc2c86631a8d4c47a1f5da3205920b3f2bdb6e38684981a0f5b5cb5592a | 4795 | `replay.sh` 第 2316 行 `driver_e171_s4_f4_substitutes_comparator` | `e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator-e171-r3.out` | A 本机 gpu cargo；用新执行器；旧产物在git 提交 `59ce5a865^` 里；没量过；先整条跑第 10 行 |
| 12 | `e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-comparator-resume2-r6-r3-2026-10-07.out` | 40ee800a8681caf503c05131fbd05fcdba7a9514a38178660c55195330b1cabc | 811 | `replay.sh` 第 2317 行 `driver_e171_r1_s1_revision_6_comparator` | `e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-comparator-resume2-r6-r3-2026-10-07-e171-r3.out` | D 读 r1 导出根；不起执行器；旧产物在工作区；预计跑不了（推）：缺 e171-r1-dumps |
| 13 | `e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-cross-host-check-resume2-r6-r3-2026-10-07-r3.out` | a5a4895502ab5161f7e99f8813f77666edbb3d0bec881c32254077848cc81d90 | 8 | `replay.sh` 第 2318 行 `driver_e171_r1_s1_revision_6_cross_host_check` | `e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-cross-host-check-resume2-r6-r3-2026-10-07-r3-e171-r3.out` | D 读 r1 导出根；不起执行器；旧产物在git 提交 `a24379563^` 里；预计跑不了（推）：缺 e171-r1-dumps |
| 14 | `e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-research-mutations-2026-10-07-r3.out` | ac3e0b850682ad0f4a57d71d75a2bd5208bd06412d8d98aa880da4d2e07a259e | 78 | `replay.sh` 第 2319 行 `driver_e171_r1_s1_research_mutations` | `e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-research-mutations-2026-10-07-r3-e171-r3.out` | B 研究一侧 Python 变异；不碰 O3；旧产物在git 提交 `59ce5a865^` 里；没量过 |
| 15 | `e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-comparator-r6.out` | d0bc79ca3f580774cfd4bad2fd672006b98c8a5d02063c48fb9915cb246a8549 | 451 | `replay.sh` 第 2320 行 `driver_e171_r1_s2_revision_6_comparator` | `e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-comparator-r6-e171-r3.out` | D 读 r1 导出根与冻结树；不起执行器；旧产物在工作区；预计跑不了（推）：缺 e171-r1-dumps 与冻结树 |
| 16 | `e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-research-mutations-r6.out` | 3a241935b5f709f303dd38ae46532b4e576ece59906aaa76b1694cd6701fb441 | 60 | `replay.sh` 第 2321 行 `driver_e171_r1_s2_research_mutations` | `e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-research-mutations-r6-e171-r3.out` | B 研究一侧 Python 变异；不碰 O3；旧产物在git 提交 `59ce5a865^` 里；没量过 |
| 17 | `e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s1-revision-8-tools-research-mutations-2026-11-02-r3.out` | 289ce5d4e49b1adc0bc516433d700919ea9c3c39ca7261d7056b71eb299e62c2 | 54 | `replay.sh` 第 2322 行 `driver_e171_r1_s1_revision_8_tools_research_mutations` | `e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s1-revision-8-tools-research-mutations-2026-11-02-r3-e171-r3.out` | B 研究一侧 Python 变异；不碰 O3；旧产物在git 提交 `84b4ae23f^` 里；没量过 |
| 18 | `e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-08-s2-revision-10-tools-research-mutations.out` | 5ed77502626492501c1da47361d2a7b33d64088e85364b618212437888fdc212 | 62 | `replay.sh` 第 2323 行 `driver_e171_r1_s2_revision_10_tools_research_mutations` | `e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-08-s2-revision-10-tools-research-mutations-e171-r3.out` | B 研究一侧 Python 变异；不碰 O3；旧产物在git 提交 `59ce5a865^` 里；没量过 |
| 19 | `e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-08-revision-11-tools-research-mutations.out` | 72f271c97dc0f4d5c04128f1a128c36d0b06c0b4fa768023230a5a5dd022229b | 34 | `replay.sh` 第 2324 行 `driver_e171_r1_revision_11_tools_research_mutations` | `e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-08-revision-11-tools-research-mutations-e171-r3.out` | B 研究一侧 Python 变异，临时服务；不碰 O3；旧产物在git 提交 `59ce5a865^` 里；没量过 |
| 20 | `e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-01-stream-constants-r2-2026-11-08.out` | f1f4babc9639ee58d1c8a84fa0864c8b2c7fcf526d8338b80da0023c2698f77f | 83 | `replay.sh` 第 2325 行 `driver_e171_r1_revision_12_stream_constants` | `e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-01-stream-constants-r2-2026-11-08-e171-r3.out` | A 本机 gpu cargo（stream-constants）；不碰 O3；旧产物在工作区；没量过；读今天的 crates |
| 21 | `e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-01-list-check-2026-11-01.out` | e7d8caee76e35bbf4e955a2a7dad0aed1fa2a68e962f728e77a8147a6abdaa81 | 41 | `replay.sh` 第 2326 行 `driver_e171_second_rerun_list_check` | `e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-01-list-check-2026-11-01-e171-r3.out` | C 读 git 对象与今天的 mutations.tsv；不碰 O3；旧产物在工作区；没量过 |
| 22 | `e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-01-removal-effect-on-the-s4-f4-2026-11-01.out` | 4ed8153effcc0e67115a1957fae7e6b31721912beb0b44c6bc3a7d64163bce3a | 135 | `replay.sh` 第 2327 行 `driver_e171_second_rerun_removal_effect_on_the_whole_table_and_expansion_table` | `e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-01-removal-effect-on-the-s4-f4-2026-11-01-e171-r3.out` | C 读 git 对象；不碰 O3；旧产物在git 提交 `84b4ae23f^` 里；预言逐字节同（读驱动，没量过） |
| 23 | `e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-01-removal-effect-on-the-s4-f4-substitutes-2026-11-01.out` | 54de44bc117a8a01a8f66a686cc20f31ae3fadb6e12221432b125c491db7add1 | 135 | `replay.sh` 第 2328 行 `driver_e171_second_rerun_removal_effect_on_the_whole_table_and_expansion_table_substitutes` | `e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-01-removal-effect-on-the-s4-f4-substitutes-2026-11-01-e171-r3.out` | C 读 git 对象；不碰 O3；旧产物在git 提交 `84b4ae23f^` 里；预言逐字节同（读驱动，没量过） |
| 24 | `e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-01-removal-effect-on-the-revision-6-segment-two-2026-11-01.out` | c4e576112f14dc20286ce2a40a5f7285f6489a0bb6456663f6a9906ec6e857fc | 159 | `replay.sh` 第 2329 行 `driver_e171_second_rerun_removal_effect_on_the_revision_6_segment_two` | `e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-01-removal-effect-on-the-revision-6-segment-two-2026-11-01-e171-r3.out` | C 读 git 对象；不碰 O3；旧产物在git 提交 `84b4ae23f^` 里；预言逐字节同（读驱动，没量过） |
| 25 | `e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-01-list-removal-research-mutations.out` | 6436519a5f15a6d26d612408061358272f343998b411ec5e122be8714427a83b | 26 | `replay.sh` 第 2330 行 `driver_e171_second_rerun_list_removal_research_mutations` | `e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-01-list-removal-research-mutations-e171-r3.out` | B 研究一侧 Python 变异；不碰 O3；旧产物在git 提交 `84b4ae23f^` 里；没量过 |
| 26 | `e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-02-current-state-preconditions-research-mutations.out` | 8acb9f68dc1d81f269c2ffb69fc1f9a2c2dec9333bcada238e5e920609f04a88 | 26 | `replay.sh` 第 2331 行 `driver_e171_second_rerun_current_state_preconditions_research_mutations` | `e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-02-current-state-preconditions-research-mutations-e171-r3.out` | B 研究一侧 Python 变异；不碰 O3；旧产物在git 提交 `84b4ae23f^` 里；没量过 |
| 27 | `e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-02-current-state-library-root-research-mutations-2026-11-02.out` | 4b74ac0432a640f98fd954d18db7f17c7f2b4ae74ce97c49b90656e47618fefb | 12 | `replay.sh` 第 2332 行 `driver_e171_second_rerun_current_state_library_root_research_mutations` | `e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-02-current-state-library-root-research-mutations-2026-11-02-e171-r3.out` | B 研究一侧 Python 变异，临时服务；不碰 O3；旧产物在git 提交 `84b4ae23f^` 里；没量过 |
| 28 | `e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-02-current-state-announcement-research-mutations.out` | ac454a163af9890aa93dff1a5b20fa3df50786bba5ebdfb8df0b865533eeb76b | 8 | `replay.sh` 第 2333 行 `driver_e171_second_rerun_current_state_announcement_research_mutations` | `e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-02-current-state-announcement-research-mutations-e171-r3.out` | B 研究一侧 Python 变异；不碰 O3；旧产物在git 提交 `84b4ae23f^` 里；没量过 |
| 29 | `e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-02-current-state-threads-and-contrast-research-mutations-2026-11-02.out` | 17c7b3d32c1209708daaa75981af22a1422466132ea946c2c37b9bf612c440da | 50 | `replay.sh` 第 2334 行 `driver_e171_second_rerun_current_state_threads_and_contrast_research_mutations` | `e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-02-current-state-threads-and-contrast-research-mutations-2026-11-02-e171-r3.out` | B 研究一侧 Python 变异；不碰 O3；旧产物在git 提交 `84b4ae23f^` 里；没量过 |

## 十二、修订

写于 2026-11-09，新执行器目录与检查器写完之后、任何产物之前。依据都是读代码与读 kb（这一段没有跑过单测，单测与检查归主 agent 交的任务）；跑出来与下面不符的，以读数为准、再修订。原判据一条不改，下面只补臂与把读法落成可执行的写法。

1. **目录**：新目录名定为 `scripts/experiment-apparatus/e170_second_run_o3_spec_executor_journal_header_496/`（读法 4）；改动的八个文件、七份不动的文件与 V1 门槛一致，检查器 `scripts/experiment-apparatus/e171_three_oracle_collusion/new_o3_executor_checks.py`（单测 `test_new_o3_executor_checks.py`）的 `footprint` 子命令现数。新目录的单测里删掉了「按 311 字节旧头写一条记录」那一个用例（里程碑 17 收尾之前不兼容旧格式，不造旧形态输入）；用来分出记录头宽度错的是 U-F1（偏移钉字面数）、U-F4（64 项装得下、声称 65 项的记录挡在字节边界上）、U-F7（字面偏移上读 t、p、fsid）与下面的 V4 P-b。
2. **V2 的产物就是新冻结参照**：V2 的全量输出落 `research/results/`，文件名 `e170-o3-spec-executor-journal-header-496-2026-11-09-freeze-executor-e171-r3.out`，第五节 5.4 第 10 条的冻结参照指它；V2 跑出来之后新目录不再改一个字节（`name=freeze file=` 行钉住）。读法由检查器 `read-v2` 子命令落成：base_recovery 每条流要有 `name=load` 且无 `recovered=false`；锚点流一 A1 至 A6、流二 A6 与 A11 逐条 `holds=true`，缺行与 `holds=not_run` 算不过；dump_check 三个计数全 0；torn_reading 的 differing_entries 为 0；末行 `name=complete`（收严：登记 V2 没写完成标记，进程中途崩溃的产物不当读数）。A7、A12、A13 只报不判。
3. **V3 的落地写法**：I1 逐行比的是 `文件:行号:正文`；I2 的 open 规则 = 第一个参数是变量或属性，或 `os.path.join` / `dirname` / `abspath` 拼出、字符串常量只许是 `manifest.tsv`、`base-sectors.bin`、`ops-payload.bin`、`torn-payload.bin`、`ops.tsv`、`calls.tsv`、`torn.tsv`、`states.tsv`、`spec-gaps.tsv`，另报非文档串的仓内路径形字符串；I3 逐行比时去掉 `name=timing`、`name=state_time` 两种行整行，其余行里去掉 `_ns` 结尾的词、`worker=`、`dumps=`，各态判定行排序后比；I4（收严）= 新增或改写的区间只要带注释（`#` 或文档串），连同前 6 行后 3 行里要有一处 `.claude/kb/<路径>.md@` 形的出处，被引的 kb 文件要在仓里（小节标题逐字由人读 `git diff` 核，检查器不核标题），改动行里写 `crates/` 判红；I5 = 往 `system_configuration.py` 末尾加 `open("crates/singlefs-format/src/lib.rs")` 的拷贝，I1 与 I2 都必须红。
4. **V4 的落地写法与一条补臂**：三条故障 P-a、P-b、P-c 各在草稿拷贝里种，起 V2 的同一条命令，读 V2 各格。P-b 与 P-c 的种法另删 `on_disk_layout.py` 末尾两个载入时断言（否则执行器载入就崩，读不到点名的格）；各加一条保留断言的信息臂（`P-b-import-checks-kept`、`P-c-import-checks-kept`，不判，报崩溃信息）。「红在点名的格」要求执行器跑完（退出码 0 且末行 `name=complete`）且点名的格红；崩溃（退出码非 0）另记 `crashed=true`，不记红在点名的格。读代码得到的预言（没跑过）：P-c 在真 dump 上预计不在 A2 红——`root_ring.py` 的 `root_is_self_verified` 的校验和罩整个 512 字节槽，只比 `len(raw) < ROOT_RECORD_BYTES`，删 t、p 两行不改它读的任何偏移，而 `replay_prefix.py` 读 `reclamation_threshold` 会在载入后抛 KeyError；读数与此不符照读数。P-c 红不在点名的格时怎么读（F1 触不触发）交主 agent 定，这一段不改 V4 的门槛。
5. **第九节 M5 的实现形态（补一条臂）**：代码里没有字面的 64 与 65，点名项上限由 `journal_ring.py` 里 `头 + 点名项数 × 56 > 记录长度` 这一道字节边界给出；M5 改成 M5-one-more-item（边界多放一项：`> record_bytes` 改 `> record_bytes + 56`），单测 U-F4 的声称 65 项的记录把头校验和与载荷校验和都重封，使这一道边界是唯一挡它的。`>=` 那一种在 4096 与 56 下 `496 + n × 56` 取不到 4096，是等价变异，不记「没红」。M4 分记录分支与根分支两种（`replay_prefix.py` 里 t、p 写死 0 与 255），单测 U-F2 与 U-F8 各抓一支；由检查器 `unit-mutations` 子命令在草稿拷贝里跑，必须红的单测名写在子命令里。
6. **V5 的落地写法**：冻结目录与新目录各起一个子进程（同名模块不能放进一个解释器），用各自的 `synthetic_pool_for_tests.py` 造流，取样态 = 前缀持久 0 至 N 写、全持久缺一写、撕裂写在前缀末与全持久上各一；逐态比 `(strict, provisional, landed_text, landed 投影的摘要)`，「同一段抽象历史」= 两边写流骨架（设备、种类、偏移、长度、调用分段）的摘要相同且取样态集合相同，不同则整格红；有一态判出 error 也红。
7. **没有写进这次的**：第六节 V6（判定方向的判法脚本）与第九节 M6 至 M8，这一段不写。

### 修订 8（2026-11-09，V2 第一次跑出之后、产物作废重出之前；记一次输）：A4 钉值过时

**输**：V2 第一次跑（任务 11106）末行 `name=v2_verdict ok=false failing_cells=anchor_A4:new_pool_file_creation`，产物第 12 行 `name=anchor_assertion id=A4 stream=new_pool_file_creation holds=false chosen={0: (6, 0, 3, 1), 1: (6, 0, 3, 1)}`；其余 17 格 `ok=true`。登记 5.4 的清单没有核 `executor_main.py` 第 185 行的 A4 钉值 `(世代号, 槽号, journal tail, 实例代号) = (5, 1, 3, 1)`，该值抄自 E170 第 1 次重跑的产物，那时取号只写一轮；5.4 第 8 条「不动 `executor_main.py`、锚点值取自 kb 表」写的前提（锚点值与 kb 表一致）没核过。

**依据（只用 kb，不读 `crates/`）**：`.claude/kb/layout/01-first-txn.md` 第 62 行（a1：取号两轮，第一轮槽 0 世代号 2，第二轮槽 1 世代号 3，实例代号都写 1）、第 65 行（w3：暖机系统配置槽 0 世代号 4 tail = 1）、第 68 行（w6：槽 1 世代号 5 tail = 2）、第 83 行（t15：事务系统配置槽 0 世代号 6 tail = 3）、第 97 行（槽世代号从 1 起每写 +1、下一次写的槽 = 世代号 mod 2、择槽取校验和过且世代号最大）、第 119 行（槽世代号 mkfs 1，取号 2、3，暖机 4、5，新池新建文件发布后 6）、第 399 行（同上）。推：mkfs 1 → 2（槽 0）→ 3（槽 1）→ 4（槽 0）→ 5（槽 1）→ 6（槽 0），选中最大世代号 6、槽 0 = 世代号 mod 2、tail 3、实例代号 1，即 `(6, 0, 3, 1)`。第 38 行已写明：旧值 `(5, 1, 3, 1)` 是「取号写一轮时的历史产物」。

**改了什么**：① `executor_main.py` 第 185 行 `(5, 1, 3, 1)` 换成 `(6, 0, 3, 1)`，只此一个字节串；其余锚点（A1 至 A3、A5 至 A7、A11 至 A13）逐个对过 kb 都不动（A2 的四个根与偏移 `(1 << 20) + 4096`、A3 的环内三条 (0, 4096, 8192)、A5 版本表、A12 的三个 magic、A13 的槽距 4096 与 w1 至 w6、t13、t14 行相符；A7 的 12 项是 t13 行「点名 t1..t12 共 12 项」）。② V1 的白名单：`executor_main.py` 进改动文件集合（八个变九个），不动的文件由七份变六份（去掉 `executor_main.py`）；收严：`executor_main.py` 相对 487 目录必须恰好一行不同、且这一行是 A4 锚点行里 `(5, 1, 3, 1)` 换成 `(6, 0, 3, 1)`，别的字节一个不动（检查器 `footprint` 多出 `name=v1_executor_main_diff only_the_a4_pin=…` 一行，`v1_verdict` 带 `executor_main_only_the_a4_pin=`）。第九节 M10 的读法随之改成：锚点行里除 A4 钉值之外的任何改动，让这一行 `only_the_a4_pin=false`、V1 红。③ 合成池 `synthetic_pool_for_tests.py` 的 `new_pool_file_creation_stream` 加参数 `acquisition_rounds`（缺省 1，老形态不变，既有用例全不动）；`acquisition_rounds=2` 写取号两轮、暖机写槽 0 世代 4 与槽 1 世代 5、事务写槽 0 世代 6。④ 单测 `test_executor_on_synthetic_pool.py`：既有老形态小池上的 A4 读数由 `holds=true` 改为 `holds=false`（小池取号一轮，选中 `(5, 1, 3, 1)`；既有第 170 行的断言钉的是这个小池自己的事实，不改）；新增 `test_a4_with_two_acquisition_rounds_…`，在取号两轮的小池上跑驱动，断言 A4 `holds=true` 且 `chosen={0: (6, 0, 3, 1), 1: (6, 0, 3, 1)}`。⑤ 变异 M11：把 `executor_main.py` 的钉值改回 `(5, 1, 3, 1)`，`test_a4_` 那条用例必须红（检查器 `unit-mutations` 的第五行）。

**收严、不放宽**：V2 的判据（A1 至 A6、A11 全 `holds=true`）一字不动，只是 A4 的期望值跟 kb；原来 `(5, 1, 3, 1)` 的读数不当通过。第一次 V2 产物作废重出（主 agent 先删 `research/results/e170-o3-spec-executor-journal-header-496-2026-11-09-freeze-executor-e171-r3.out` 再重跑，该文件在 `git status` 里是本趟新建的未跟踪文件，不是旧产物，F8 不触发）。

### 修订 9（2026-11-09，产物之前）：V4 P-c 的读法与补一条真 dump 上看得见的臂

主 agent 在派发提示里定：崩溃也算查出来了，但修订 4 里「P-c 在流一 A2 红」这条预言记一次输，不放宽判据；再补一条真 dump 上看得见的臂。
- **P-c 的预言记输**：修订 4 读代码预言 P-c 不在 A2 红、预计崩溃（`replay_prefix.py` 读 `reclamation_threshold` 抛 KeyError），登记 V4 与第九节 M3 写的「流一 A2 `holds=false`」这条没有得到支持的读法记为输；实际结果以读数为准，写进实验页。V4 的 P-c 判「抓到」改为：执行器崩溃（退出码非 0）或流一 A2 `holds=false`，二者任一（检查器 `seeded-faults` 分开记 `crashed=` 与 `red_in_named_cell=`，另报 `caught=`；`crash_counts=true` 只对 P-c 生效，P-a、P-b 的判据不动，两条 `-import-checks-kept` 信息臂仍不判）。
- **补一条臂 `P-c-misplaced-rows`（变异号 M3b）**：t、p 两行读成放在 `self_checksum` 之前（紧跟 `rollback_floor`），根记录总宽不变，`self_checksum` 的偏移后移 9，同样删掉 `on_disk_layout.py` 末尾两个载入时断言；预言（读代码，没跑）：真 dump 上每个根的自证校验和读错位置，不崩溃，流一 A2 `holds=false` 或 `base_recovery recovered=false`（两格任一红，`expectation=any`）。抓到 = 执行器跑完（退出码 0 且末行 `name=complete`）且那两格任一红。门槛：P-a、P-b、P-c、P-c-misplaced-rows 四条各自抓到；任一没抓到 → F1。

### 修订 10（2026-11-09，29 行产物之前）：V3 的 I4 第一次红在一句测试注释

- **记一次输**：任务 11118 判 `name=v3_verdict ok=false red=I4`，点名 `test_executor_on_synthetic_pool.py:786` 的一句新注释没写 kb 出处，这是修订 8 加的。I1、I2、I3、I5 都绿。按 F3 的字面，I4 红时执行器作废。
- **只许收严**：判据不动，I4 仍要每处改动的注释带 kb 出处。
- **写明改了什么**：只给这句注释补 `.claude/kb/layout/01-first-txn.md` 的出处，执行器代码与 V2 产物一字不变；补完重跑 V3，I4 绿才算过，不绿仍按 F3 作废。
- **又记一次输**：补注释改了测试文件，`name=freeze` 那一行的 sha256 跟着变，任务 11125 因此红在 I3（`same_as_in_repository=false`）。这是改了新目录而没重出 V2，登记写明 V2 产物出来之后新目录不再改。处置：删掉 V2 产物，用这份文件重出 V2，再跑读 V2 与 V3；之后新目录一个字节都不再改。

### 修订 11（2026-11-09，29 行产物之前，第 5 至 11 行与整表产物之前）：C2 加钉死放行表；E171.2 的标签；产物形状变了

**依据**：主 agent 2026-11-09 的定案（派发提示），读数是调查报告（工作根 `e171-n1f-i321/result.md`）：E171.3 与 E171.4 的流二 `N1-F`（表行 13）多红 I-3.21，`expected=in`，F4 触发。

**记一次输**：第六节 C2 与 C1 的预言"`expected=in` 的行红集多出任何一项 = 有问题"在这一行打中，而这一行不是产品或检查器的问题：补丁只改最新根的 txg 字段并重封，最新根退成上一版，带最新一版内容的假根仍是候选根，遍历走到的单元不在上一版的分配记录里，I-3.21 按定义句字面该红；`expected=in` 说的是恢复读得出，不是池级检查器放行。I-3.21 在 2026-10-03 的旧产物之后才实现，所以旧产物里没有这一红。登记写 C2 时把 `expected=in` 当成"池级检查器不该多红"，是判别子观测不到。

**怎么改（收严，不是放宽）**：C2 的字面不动。加一张钉死的放行表 `scripts/experiment-apparatus/e171_three_oracle_collusion/pinned-allowances.tsv`，一行一条：实验行、流、表行号、名字、不变量、理由、依据。现在两行：E171.3 与 E171.4，流二，行 13，`N1-F`，I-3.21。判法脚本 `direction_change_judge.py` 加 `--allowances`、`--experiment-row`：
- 只有 `expected=in`、流、行号、名字都相符、且该不变量确是新增的红，才算正常；
- 同一行多出表外的任何一项，照判有问题；
- 表里有一条在产物里没用上（行没有、名字不符、该不变量没新增），判有问题（防表腐烂）；
- 表多一列、少一列、不变量没有定义行、同一格重复，脚本退 2；
- 不给 `--allowances` 就没有任何放行；表里别的实验行不套用；
- 这行之外的任何有问题照停。

单测（`test_direction_change_judge.py`）：放行表多放一列、放错一行、多红别的、条目没用上、名字不符、真表恰好两行；变异 M12（放行不看表外新红）、M13（放行不看名字）、M14（审计关掉）各让点名的用例红。

**E171.2 的标签**：E171.2 的 `e170_bad_row` 行没有 `expected`。`--labels-mode seed-independent-prefix` 从 E171.3 的 `bad_row` 产物按行号取 P 行与 N 行（盲测行之前的前缀），前缀之后的盲测行一律记 `not_in`。依据：驱动 `replay.sh` 的注释写"P、N 行与种子无关"；导出装置 `e170_specification_executor_export.rs` 里种子的随机数只在盲测行的循环里用，盲测行的形状只有 P2、P3、P5，期望都是 `not_in`；E171.2 与 E171.4 的 `e170_bad_row` 行 0 至 17，新旧产物都逐行相同。前提读不出（行号不连续、盲测行不在末尾或有非 `not_in`、各流前缀长度不同）就不给标签。预言（没跑）：E171.2 补上标签后，流二行 13 `N1-F` 多红 I-3.21 是有问题 1 条（放行表按派发只放 E171.3、E171.4，不含 E171.2，要不要加由主 agent 定）。

**产物形状变了**（协调者消息，别的会话把已知坏镜像表搬进 `crates/singlefs-verification/src/known_bad_images.rs`，记录在 `records/2026-11-06-门禁判定全部入库.md` 批 13）：
- `@driver_e171_s1_known_bad_images`、`@driver_e171_s2_known_bad_images` 的产物去掉四格（`known_bad_test_file_sha256_is_the_copied_one`、`known_bad_entry_count_here`、`known_bad_entry_count_in_the_original_file`、`known_bad_entry_names_equal_the_original_file_in_order`），换成一行 `name=known_bad_entry_count count=<条数>`，条数现在 47；`bad_images_report.py` 的 `name=a9_a10_a8` 行去掉 `known_bad_rows_expected`、`q2_table_rows_expected` 两个字段。
- 第六节"决定性核对"（第 1 至 4 行重出产物与任务 10900 现跑产物逐字相同）对第 2、4 行不再成立，任务 10900 的现场仍是旧形状；只对第 1、3 行仍按原文核。
- C5 认这种形状变化：旧产物里那四格的 `s4` 行只在旧产物、新产物多一行条数且条数等于 `known_bad_entry` 行数，记 C5 正常（字段删了不算判定变）；判法脚本已加，单测 `TableMovedShape`。条数与条目行数不符、或别的 `s4` 行消失，仍归不到。
- 第三节 A8 的钉值作废（该钉值依赖被删的字段），不再作判据。

**没动**：C1 至 C5 的其余规则、V1 至 V11 的门槛、F4 的处置原文。

### 修订 12（2026-11-09，29 行产物之前）：C4 三条主 agent 过目判正常。量的（探针任务 11192，副本只加打印）：I-5.1 镜像上 I-3.1、I-3.11 都是 `Holds`（不是没判全），记账 allocated=278528、deferred=16384、walked=262144 算术相符，这张镜像改由 I-3.15、I-3.21 与目标 I-5.1 判红；I-9.1 镜像上 I-3.1 是 `Holds`、I-3.11 仍 `Violated`；I-3.1 作目标的镜像 I-3.1 与 I-3.11 都仍 `Violated`；干净镜像全 `Holds`。所以检查器没有变窄，判法仍把这三条报 unplaced（C4 不自动判正常），过目结论在这一条。另定跑法：不一次点全表，按 `--item` 分行交，不要卡的行不占卡；第 9、10、11 行放一条任务共用前置。第 5、6 行先跑，它们不过不开其余行。
### 修订 13（2026-11-09，第 5 至 9 行产物之前；只补装置，原判据不动）：逐行驱动的线道树漏了 `prune-server/`；变异驱动基线红没留原因
依据：任务 11511（日志 `task-b0f6b921-11511.attempt-1.log`，现场 `e171-r3-run/card-rows`）。① 第 5、7、9 行逐行驱动 116 行编译全败（45 秒），`compile-failed-1.log`：「failed to read …/lane0/tree/prune-server/Cargo.toml」，因为 `scripts/experiment-apparatus/e171_three_oracle_collusion/row_driver.py` 的 `SYNCED_ENTRIES` 不带验证包经 path 依赖的仓根 `prune-server/`。改法：`sync_tree` 在 `SYNCED_ENTRIES` 之外，现读 `crates/*/Cargo.toml` 的 `path = "…"`，把指到仓根 crates/ 之外的第一层目录（今天只有 prune-server，仍按 `--exclude target` 不拷 target）一并同步（`path_dependency_entries`；不引 `scripts/hosts` 的共用库，因为这份驱动在第二台上单独拷在套件目录里）；`SYNCED_ENTRIES` 那一行与 `repository-copy-closure-exempt.tsv` 的登记锚串一字不动。冻住的树（`replay.sh` `driver_e171_s4_comparator` 的 `frozen="$scratch/frozen-tree"`）本来经 `replay_copy_closure_into`（minimal-copy.py 的闭包）拷，闭包里已有 prune-server，不用改。判据、读法、产物名、行号都不动。证红：`test_synced_entries.py` 新增 `test_a_path_dependency_outside_crates_reaches_the_snapshot_and_the_lane_tree` 与 `test_the_real_repository_syncs_the_path_dependency_of_the_verification_package`；弄坏开关 `E171_ROW_DRIVER_BREAK=path-dependencies-not-synced`（证红命令在执行员报告「要主 agent 跑的命令」，结果待填）。冻结的 `e171_three_oracle_collusion_after_the_oracle_fixes/row_driver.py`（第 2 次重跑与 E201 的出处证据，exempt 表写明「不改」，`SYNCED_ENTRIES` 同样不带 prune-server）一个字不动：今天 replay.sh 的 E171 各行用的是 `e171_three_oracle_collusion/row_driver.py`。
② 第 6 行（`driver_e171_s3_comparator_mutations`）产物 `E7RESULT name=baseline unit_tests=red`：单测基线红只留一个「red」，查不出是哪几条。只补诊断：`mutation_driver.py` 的 `run_unit_tests` 在基线那一次把单测输出写进 `<scratch>/baseline-unit-tests.log`（不进产物行，产物格式不变）。原因未定：读过的候选（线程变量、冻结参照缺失、副本位置推仓根）在这份目录的四个单测模块里读不出红；基线 15 秒就红，不是 300 秒超时。待主 agent 跑报告里的诊断命令（在任务现场留下的 `baseline-before` 副本里直接跑那四个模块）读出红在哪几条，再另补修订。

### 修订 14（2026-11-11，第 7 至 11 行产物之前；只补装置与跑法，原判据不动）：整表按行区间分片跑、阳性对照只跑空变异、装置的环境补 CARGO 并带 gpu 特性编、第 8 行旧产物取得回

用户定每条普通测试不超过 4 小时、整表分片。任务 12566（`replay.sh --item E171.7 … --item E171.11`，冻树）结局 `over-four-hours`。以下读数都是量的，现场 `~/.local/share/singlefs/work/c999-frozen-tree-final-products/e171-card-rows/`：
1. **14104 秒花在哪**：`E171.1.err` 第 139850 行 `row_scope scope=full_table rows=115 of=115` 出在阳性对照 O1-对拍 那一趟（输出根 `rows-controls-O1-对拍`，做完 109 个行目录），`rows-s2-trial` 没开。成因：`row_driver.py` 现行「不给 `--rows` 就是整表」，对照命令只给 `--empty-mutation`（「另跑空变异」），每条对照于是跑了空变异加整表；旧驱动 `--rows` 缺省空串（`git show 59ce5a865^:scripts/experiment-apparatus/e171_three_oracle_collusion/row_driver.py` 第 593 行），对照只跑 row-0。比对器只读对照输出根的 `row-0/dump`（`replay.sh` 比对器的 `--control …:empty_mutation:…/row-0/dump`）。s2 趟（第 7 行）与 s4 趟（第 9 至 11 行）各有四条对照（O1-对拍、O1-崩溃判官、O2、O3）。那 109 个行目录的 `row-result.tsv`：编译合计 12 532 秒（中位 65.2 秒，最大 1 892 秒，空变异冷编 220.6 秒），装置合计 201 秒（中位 1.5 秒）。
2. **这 109 行的装置读数没判成**：`row-1/device.err` 有 43 处 panic「要经 cargo test 起（环境里要有 CARGO）：判法单元指纹要记编这个测试二进制的工具链」（`crates/singlefs-verification/src/crash_point_replay_progress.rs` 第 41 行），`device.out` 判决行 `streams_not_exported=2`，装置 1.5 秒就退。成因：装置 `row` 模式的 O2 经给定状态引擎判（`e171_three_oracle_collusion.rs` 的 `given_states_engine_of_this_experiment`），引擎起时要环境里有 `CARGO`，驱动直接起装置、环境里没有。驱动还用不带 `--features gpu` 的 `cargo build` 编装置，而引擎在配了 GPU 的环境里遇到不带 gpu 特性的二进制判段报本次未跑（`crash_amplification.rs` 的 `start_from_environment` 文档注释），基线、已知坏镜像各趟用 `cargo run … --features gpu`（`CHECKER_TIER_FEATURES`）。按这样的行读数跑完整表也是废的。
3. **第 7 行「要卡却不用卡」**（读码）：要卡是因为驱动闭包点到 `CHECKER_TIER_FEATURES`——基线、dump、坏状态、已知坏镜像各趟经 `cargo run --features gpu` 起装置，三个模式的 O2 都经给定状态引擎，配了 GPU 就在卡上判（文件头 `task-server-gpu-arguments` 列了 E171.7）。任务 12566 里卡利用率一直 0，是因为那 14104 秒里跑的是第 1 条的对照整表，装置在引擎起时就 panic、没碰过卡；真用卡的是开头的前置（基线两趟、dump、坏状态、已知坏镜像、关一条两趟、O3 判镜像、O3 基线），`E171.1.err` 里第一条到第 139841 行最后一条 `GPU_LIFECYCLE` 的时间差 1 344 秒。能不能拆成录入不占卡、判定才占卡：引擎本来分两段（`crash_library.rs` 的 `RunStage`，环境变量 `record` 只跑录入段、`judge` 只跑判段，录入段 CPU 跑数据写进库），但这份装置在 `pool_checker_red_names_through_the_given_states_engine` 里遇到只录、只判的占位就 panic（「实验装置不分只录、只判两趟跑」），而行的导出文件要在同一趟里写进 O2 名单，所以现在拆不开。改法：装置三个模式在录入段只录表、不写 O2 列，判段再补写；行驱动对每行先起录入段（不占卡）、判段集中起。这是入库装置（`crates/singlefs-implementation-experiments/src/bin/e171_three_oracle_collusion/main.rs`）的改动，这一次不做，只靠分片缩短每条任务占卡的时长。

**这一次改了什么**（都在 `scripts/experiment-apparatus/` 下，没有 `.rs`，不碰 `crates/`）：
- `e171_three_oracle_collusion/row_driver.py`：① `--empty-mutation-only`（只跑 row-0，对照每一趟用它）；② `--row-range <起>-<止>` 取序号闭区间里的行，同一个输出根里各片的行目录互不相交，区间不许切开文件组（补位只在组内换行），带 `--keep-compile-failures-as-they-are`（不补位）的不受限；③ `--print-shard-ranges <片数>`（默认按文件组对齐，`--shards-may-cut-groups` 只按行数均分）；④ `--reuse-from <输出根>`（可给几次）：整表那一趟从各片的共用输出根取键相同、可复用的行目录（硬链接，不行就拷），编不过、测量管子缺陷、超时的行不取；`table …` 行多一个 `rows_taken_from_other_roots=`；⑤ 装置的环境补 `CARGO`（没有就取 `PATH` 上的 cargo），编装置带 `--features gpu`；⑥ 装置的 `device.out`、`device.err` 里有 `NOT_RUN` 或工具链缺失的句子，这一行记测量管子的缺陷（阶段 `device_output`），驱动退 5、不再领新行。复用键、行目录格式、比对器读的东西都没动。
- `replay.sh`：E171 的 s2、s4 两趟的阳性对照改用 `--empty-mutation-only`；环境里设了 `E171_ROW_SHARDS_ROOT` 时，s2、s4、补位三趟整表命令带 `--reuse-from $E171_ROW_SHARDS_ROOT/step8`，扩表一趟带 `$E171_ROW_SHARDS_ROOT/f4`；没设就整表现跑。
- 新增 `e171-row-shard.sh <step8|f4> <第几片>/<共几片>`：一片一条任务，表 step8（115 行）切 4 片、f4（163 行）切 6 片，各片 2 条线道、每条 2 线程，输出根 `$E171_ROW_SHARDS_ROOT/<表>`，工作根每片一个。
- 单测 `test_row_shards.py`（分片区间、`--row-range` 剪行、对照只跑 row-0、分片加合并与整表同一份行目录和 `row …` 行、快照不同不取、编不过的行不取、环境有 CARGO、编装置带 gpu 特性、装置输出的缺陷）。

**Z3 与产物**：各片里设了 `--row-range`，各片是中间物，不进 `research/results/`；比对器出的产物来自整表那一趟（不带 `--row-range`、`--rows`），所以第十一节 Z3 不触发。阳性对照改只跑空变异不改比对器读的 row-0，对照读数不变（预言，没量过）；第 7 行 s2 趟的 `rows-s2-trial` 仍是现行驱动的整表 115 行加空变异，旧产物的 `name=row` 只有 9 行（空变异加 `--rows 1,6,8,18,28,64,83,84` 八行，`git show 59ce5a865^:research/results/e171-three-oracle-collusion-2026-10-03-s2-comparator.out | grep -c 'E7RESULT name=row '`），行数对不上不是这一条修订造成的，记入第 7 行的 R3 结局。

**第 8 行（`driver_e171_s3_comparator_mutations`）旧产物取得回**：任务 12566 里判「取不回」是因为快照树是 `git init` 出来的空库（`git log --all` 在里面找不到）；在主工作区 `git log --all --diff-filter=D --format=%h -n1 -- research/results/e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03-r7.out` 得 `59ce5a865`，`git show 59ce5a865^:<路径> | sha256sum` 得 `fe4652fffa987d99ea984fc4cb8c8d408d29bbe62f2ab8e549415ea2380a2837`、`wc -l` 得 138，与第十一节表第 8 行相同。处置：比对对象不换；新产物在快照树里出（Python 变异驱动，不要卡，上一趟 14 分 56 秒跑完、基线 `unit_tests=green`），旧一边由承接的执行员在主工作区取、与新产物并排，不在快照树里比。第 7、9、10、11 行同理（旧产物都在 `59ce5a865^`）。

**每条命令的挂钟估计**（推；依据：编译中位 65 秒、冷编 221 秒、前置 1 344 秒是上面的读数，装置加 O3 每行 150 至 200 秒没量过——行读数不是废的才有数，按基线各趟约 150 秒推）：每行约 250 秒。step8 每片约 29 行、2 条线道：29 × 250 ÷ 2 + 冷编 ≈ 4 000 秒（约 1.1 小时），编译慢行（上面的 1 892 秒那样的）每多一个加至多半小时，最坏约 2 小时；f4 每片 27 或 28 行同量级。整表那一趟（取各片的行）：第 7 行 前置 1 344 + 四条对照约 1 800 + 编不过的 2 行补位 500 + Q3-核 3 趟 750 + 比对器 ≈ 4 500 秒（约 1.3 小时）；第 9、10、11 行同一条命令点三个 `--item`（共用草稿目录，后一行见到前一段的记号不重做）约 6 000 秒（约 1.7 小时）；第 8 行约 15 分钟。没有一条超过 2 小时。

**快照树**：冻树已删，这一次在主工作区此刻的树上建（`scripts/hosts/minimal-copy.py`，写法同上一趟 C999 的做法），建在这一批全部装置改动、单测都过之后，建完逐文件清单留草稿目录；产物的 HEAD 与清单摘要进实验页的树一段。

**不动**：第六节 C1 至 C5、V1 至 V11 的门槛，第七节钉值，第九节变异，第十节失败条款，第十一节作废与停机条款，修订 1 至 13，第十一节表第 7 至 11 行的比对对象与重跑产物名。

### 修订 15（2026-11-13，第 7、9、10、11 行整表产物跑过之后补记；只记跑法，原判据一字不动）：整表四行实际的跑法与登记的三处不同

**时点与依据**：这一条写在产物之后，不是跑前修订；依据是派发提示（`records/2026-11-10-门禁与实验欠账收尾调度.md#批次 7` 那一次写回）原句「这一趟与登记写的跑法有三处不同，登记修订里要写明（修订写在登记文件末尾「十二、修订」之下，新编号；原判据一字不动）」。不收严也不放宽任何判据，只把跑法记下来，读数照第六节原判据读。

1. **装置分录入、判两段起**：`crates/singlefs-implementation-experiments/src/bin/e171_three_oracle_collusion/main.rs` 认 `--stage record|judge`（文件头第 24 行：「`record` 只跑录入段，`judge` 只跑判段，不给就两段先后跑」），录入段不持卡，判段前后领卡、还卡各一回。用户定「现在就改，重跑全部」，所以 278 行与共用前置全部重出，没有取修订 14 那一趟（任务 12566、13253 至 14048 那几片）的任何一行。
2. **逐行驱动与共用前置按项入判定库、整表从库取行**：键是函数单元键；整表那四行（第 7、9、10、11 行，任务 15469、15470、15471、15479）的逐行驱动从库里取各片判过的行。步长 8 那张表按 16 片、扩表按 24 片跑（`scripts/experiment-apparatus/e171-row-shard.sh` 的 `--step8-slices 16 --f4-slices 24`；第 1 片是先按 8 片切的第 1 片，14 行），修订 14 写的 4 片、6 片不再是实际片数。各片带 `--row-range`，是中间物；进 `research/results/` 的四份产物来自不带 `--row-range`、`--rows` 的整表那一趟，第十一节 Z3 不触发（同修订 14 的读法）。
3. **比对器一处改动**：`scripts/experiment-apparatus/e171_three_oracle_collusion/comparator.py` 的 `rows_with_substitutes`（第 259 行起）对补位输出根里从库取回、这一趟没补位的原行（原行没编不过、`substituted_from` 是 `-`、表名与结局都与原行相同）不换也不拒。第 11 行第一次跑（任务 15472）因此判「跑不了」，改后重出（任务 15479）。单测一条、变异表 `scripts/mutation/tables/e171_three_oracle_collusion.tsv` 加 X74（第 66 行）；证红任务 15477、同目录单测任务 15476。第 7、9、10 行三份产物是改之前的比对器出的，它们不走补位输出根，这一处改动碰不到它们（读码推的，没重出核）。

**旧一边**：四份旧产物在快照树里取不回（快照树没有 git 历史），照修订 14 第 8 行那一段的处置，由执行员在主工作区 `git show 59ce5a865^:research/results/<名>` 取出、与新产物并排；sha256 与行数与第十一节表第 7、9、10、11 行相同。旧产物是 281 行名单下跑的，这一趟是 278 行名单（第 2 次重跑撤掉第 33、107、236 行），这一层差异单列，不并进判定变。

读过的文件（行区间；grep 命中行也算读过）：
- 规矩：`.claude/agent-common.md` 第 1 至 127 行；`.claude/singlefs-ai-sop/rules/test-discipline.md`、`evidence-discipline.md`，`.claude/rules/three-way-inference.md`、`mutation-sampling.md`、`implementation-first.md`、`test-script-discipline.md` 整份。
- 调查报告：`/home/fy5090/.local/share/singlefs/work/e171-row-reds/result.md` 第 1 至 245 行；该目录里 `E171.1` 至 `E171.4` 的 `.out`、`.stored` 只算了 `wc -l` 与 `sha256sum`。
- `scripts/experiment-apparatus/replay.sh`：第 1 至 60 行、第 2297 至 2340 行、第 3040 至 3570 行；`grep -n` 命中：`driver_e171`、`^e171_`、`CHECKER_TIER_FEATURES=`、`E171_FREEZE_REFERENCE=`、`--row\b`、`REPLAY_JOBS`、`list-items)`、`e170_second_run_o3_spec_executor`、`journal_header_487`。
- 执行器：三个目录（`e170_second_run_o3_spec_executor/`、`…_journal_header_487/`、`e170_o3_spec_executor_with_witness_predicates_journal_header_487/`）的文件清单；487 目录 `on_disk_layout.py` 第 1 至 120 行、`system_configuration.py` 第 90 至 135 行、`executor_main.py` 第 147 至 190 行与第 225 至 299 行；`grep -n` 命中：`crates`、`inode`、`new_root_`、`JOURNAL_HEADER_BYTES|ROOT_RECORD_BYTES|487|311|457|364|463`、`^import|^from`、`open(`、`FIRST_VERSION_INCOMPATIBLE_BITS`；冻结目录与 487 目录逐文件 `diff` 数行。
- 本实验 Python 一侧：`e171_three_oracle_collusion/frozen_o3.py` 第 1 至 60 行；`o3_bad_image_judge.py` 的 `grep -n` 命中；`e171_three_oracle_collusion_after_the_oracle_fixes/` 目录清单与 `frozen_o3.py` 的 `grep -n FROZEN_DIRECTORY_RELATIVE|FREEZE_REFERENCE_RELATIVE` 命中。
- `crates/`：`crates/singlefs-core/src/system_configuration.rs` 第 25 至 40 行；`crates/singlefs-format/src/lib.rs` 第 245 至 256 行与三处 `grep -n` 命中（第 234、249、252 行）；`crates/singlefs-core/src/recovery.rs` 第 6805 至 6815 行；`crates/singlefs-checker/invariants/03-journal.md` 第 21 行（I-8.14）、`02-根环.md` 第 25、26 行、`05-空间记账.md` 第 8、18、28 行的 `grep -n` 命中。
- kb：`.claude/kb/layout/01-first-txn.md` 第 108 至 118 行、第 130 至 170 行、第 355 至 402 行与 `grep -n` 命中；`.claude/kb/decisions/23-journal的角色与格式.md` 的 `grep -n` 命中（第 150 至 176、473 行）；`.claude/kb/experiments/171-三oracle合谋测试.md` 的标题行 `grep -n '^#'`，第 2 至 21 行各取前 50 个字符，第 31、1349 行，第 13 至 20 行（`quote-kb.py` 抄入第二节，未逐字读）。
- 登记：`research/prompts/e171-r2-prereg.md` 第 1 至 50 行与标题；`research/prompts/e171-r1-prereg.md` 只数了行数；`research/prompts/e201-preregistration.md` 第 1 至 60 行、第 337 至 352 行与标题行；`.claude/gate.d/replay-comparison-units.tsv` 第 1 至 44 行；`.claude/kb/checks-owed.md` 的 C951 行。
- 任务日志：`task-server/task log 10980`（主 agent 给了号）。

命令（原样，在仓根）：
- `bash scripts/hosts/work-directory-ownership.sh claim /home/fy5090/.local/share/singlefs/work/e171-r3-prereg --purpose 'E171 第 3 次重跑登记草稿'`
- `ls research/prompts | grep -i e171`（r1、r2 已占，r3 未占）
- `grep -c '^E171|@driver' scripts/experiment-apparatus/replay.sh`（29）；`sed -n 2306,2334p scripts/experiment-apparatus/replay.sh | awk -F'|' …` 对每个旧产物名查工作区，不在的 `git log --all --diff-filter=D --format=%h -n1 -- <路径>` 再 `git show <提交>^:<路径> | sha256sum` 与 `wc -l`（草稿目录 `old-products.tsv`）。
- `grep -rnE 'e170_second_run_o3_spec_executor([^_A-Za-z0-9]|$)' scripts crates .claude/gate.d`（14 份文件）；`sha256sum` 两个 487 目录的 `on_disk_layout.py`、`system_configuration.py`、`executor_core.py`。
- `python3 -c "print(78+21+373+8+16, 4+16+4+4+8+86+8+8+32+86+86+86+8+1+1+12+16, …)"`（第七节乙类）。
- `python3 scripts/kb/quote-kb.py <草稿目录>/quote-out.md '.claude/kb/experiments/171-三oracle合谋测试.md:13-20' '…:31-31' '…:1349-1349'`（回读逐字节一致）。
- 追加与定点替换：`cat >> research/prompts/e171-r3-prereg.md`、`python3 scripts/kb/replace-once.py`。

### 修订 16（2026-11-13，第 7、9、10、11 行整表产物之后；用户定扩表）：C2 的放行表扩到这四行

**依据**：用户 2026-11-13 定案「扩表」（原话在 `records/2026-11-10-门禁与实验欠账收尾调度.md` 2026-11-13 那一条）。量的：四份新产物的 Q2 表里，流二 `fixed_script_through_e_then_unmount` 第 13 行 `N1-F`（`expected=in`）都多红 I-3.21（`research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator-e171-r3.out` 第 4717 行），F4 照第六节字面在这四行上已记触发一次（实验页「第 3 次重跑·整表四行」一节第 5 条）。

**怎么改**：C2 的字面不动。钉死的放行表 `scripts/experiment-apparatus/e171_three_oracle_collusion/pinned-allowances.tsv` 加四行：E171.7、E171.9、E171.10、E171.11，流二，行 13，`N1-F`，I-3.21，理由与修订 11 那三行相同（同一张 Q2 表的同一行、同一个成因）。钉着表的单测 `test_the_pinned_table_holds_exactly_the_seven_decided_rows` 改成七行。

**不动**：别的 `expected=in` 行多红任何一项照旧是有问题；这四行上别的不变量多红照旧是有问题。这一处扩表被攻过零轮；判法脚本 `direction_change_judge.py` 还读不动比对器产物（修订 15 记的那一处），这四行此刻没有脚本在读。

## 十四、并行与挂钟

独立单位：29 个重出行里，第 9、10、11 行互相不独立（套着：第 10 行的驱动第一步整条跑第 9 行，第 11 行的第一步整条跑第 10 行，三条共用 `$OUT_DIR/e171-s4-comparator` 这一个草稿目录与冻住的树，不能同时起）；其余 26 行各用自己的草稿目录，互相独立；V1 至 V5 的检查（V3 的五条、V4 的三条种下的故障）各自独立。第 22 至 24 行读 git 对象与已有的名单核对产物，不读本趟别的行的产物，独立。
线程：取登记值 31（`.claude/heavy-thread-default`），不覆盖；一条任务里同时起的几行在进程内分 `SINGLEFS_THREAD_CAP`（`replay.sh` 文件头「派行」一节：线程之和不超过 `REPLAY_JOBS`，要卡的行彼此串行、可以与不要卡的行一起跑）。
串行：第 9、10、11 行因共享草稿目录与冻住的树必须串行（9 先，10 在 9 之后，11 在 10 之后，或只跑最外层一条取三份）；所有碰 GPU 的行（第 1 至 11 行、第 20 行，`replay.sh` 带 `--features gpu`，第三节第 5 条之外的行类别见第十一节表备注 A）彼此串行，因为要卡的行互相串行；第 5、6 行必须在 V2 至 V4 过了之后才起（O3 读数没有执行器前没有意义，F1）。计时类的量（V10）在被测进程里计（驱动 / 任务日志里的墙钟），不在转发输出的循环里打时间戳。
预计挂钟：各段如下（推，写依据；没量过的写「没量过」）：
  - 第一段（执行器改动、V1 至 V5、第 1 至 6 行）：第 1 至 4 行用任务 10900 的读数，E171.1 3 483 秒（冷）、其余 396 至 556 秒（报告，量；复用判定库），合计不超过 3 483 + 3 × 556 = 5 151 秒；第 5、6 行是第一次真跑 O3，没量过（推：与第 2 至 4 行同量级加 O3 在 117 + 164 行坏镜像上判的时间，数不出）。V2 一次执行器全量输出没量过（`executor_main.py --workers` 默认全批）；V3、V4 是几十个小进程，推分钟级；V5 没量过。总体按「执行员 / 主 agent 的工作」不进挂钟，机器挂钟推约 2 至 3 小时。
  - 第二段：第 7 行 = 前置约 400 秒加阳性对照四条与 Q3-核三趟（`replay.sh` 第 3127 行起驱动，没量过，推 0.5 至 1 小时）；第 8 行 Python 变异表 138 行的上一版产物，没量过；第 9 行驱动文件头推约 2 小时（整表 5 569.6 秒，旧树旧执行器）、第 10 行约 4 小时、第 11 行约 4.5 小时（补位 6 行 1 277.9 秒），**三条各自独立跑 = 约 10.5 小时，只跑第 11 行取三份 = 约 4.5 小时**（依据第四节第 10 条：外层草稿目录里留着内层的 `comparator.out`、`comparator-f4.out`；取法要加只取产物的入口，改 `replay.sh`，未定项 2）。
  - 第三段：第 12、13、15 行预计立刻退 77（秒级）；第 14、16 至 19、25 至 29 行是研究一侧变异表，没量过（推每行分钟级到一小时级，合计约 1 至 3 小时，无读数）；第 20 行一次 `cargo run` 加比对；第 21 至 24 行读 git，推分钟级。
预计在用核数：各段如下（推）：第一段第 1 至 6 行与 V2 约 31 个线程（登记值），V3、V4 个位数；第二段第 9 至 11 行 `row_driver.py --lanes 2 --threads 2` 加编译，高峰约 31；第三段研究一侧变异多为单进程，个位数核，可与第 20 行（要卡）同时。双机的分配不写（服务定）。
- 超过 2 小时的段（第二段的第 9 至 11 行；第一、三段合计也可能超）：**分片单元** = 行（29 个互相独立，除套着的三行）；行内 `row_driver.py` 的坏实现行（第 9 行 117 行、第 10 行扩表 164 行）有 `--selected-rows` / `--rows` 选行参数（`replay.sh` 第 3262 至 3300 行用法），可按行区间切；套着的三行共享草稿目录与冻住的树，**不能跨台分**，只能整串在一台。**续跑入口** = 同一条命令重起只补没做完的：`replay.sh` 的行键复用（判定库有同键的行不现跑，文件头「复用」）与 `row_driver.py` 逐行按键复用（驱动文件头「按键复用」，我没验过续跑只补没做完的）；做不到的是套着的三行中途被打断后，草稿目录被下一次开头的 `rm -rf` 清掉（第 3194 行起 `driver_e171_s4_comparator` 开头），中途的导出不保留，只有已入判定库的逐行判定能复用。
- **共用前置与卡（给主 agent 定次序，R5）**：① 编译：同一条 `replay.sh E171` 任务里只编一次（文件头「编译目录」：一趟一份）；分开多次交就各编一次。② 前置 400 秒左右（基线两趟、E170 导出装置 dump 与 bad-states、已知坏镜像）在第 2、3、4、5、6、7、9 行里各重做一遍（驱动里各自 `rm -rf` 自己的草稿目录再起，第 3079 至 3085 行 `e171_s2_prepare_bad_states`），不能共用草稿目录；能共用的只有判定库里同键的逐状态判定（复用），所以第 2 行起比第 1 行快（报告，量）。③ 要卡：第 1 至 11 行与第 20 行（`cargo run … --features gpu`），第 20 行是否真走崩溃判段入口我没核（`stream-constants` 模式「不写导出」，读文件头），执行员 `grep` `JUDGE_ENTRIES`（`.claude/gate.d/doc-experiments.sh`）核；第 12 至 19、21 至 29 行是 Python / git / 研究一侧，不碰 GPU，可与要卡的行并行。④ 次序建议（主 agent 定）：先 V2 至 V4 → 第 5、6 行（短，决定 O3 是否能用）→ 第 1 至 4 行 → 第 7、8 行 → 第 9 至 11 行（最长，单独）；Python 行（第 14、16 至 19、21 至 29）与它们并行。
