# 第 24 批代码轮第一轮：云端正推（Sonnet）报告（2026-10-04）

<!-- doc-lint:not-numbers K1 K2 F3 B1 H2 -->

接续前一条撞用量限额的同立场腿（它报告一个字没写）。正文 `research/prompts/_m3-formal-code-b24-r1-body.md`，改动全文 `research/prompts/_m3-formal-code-b24-r1-diff.md`。
代码引文读冻结副本 `/tmp/claude-1000/m3-formal-code-b24-r1/tree/`（引文里的 `crates/…` 路径就是那棵树里的路径）；kb 与判决文件读主仓。
标「推的」的是 checker 档没跑、只读码推出来的；标「量的」的是我在自己拷出的副本上跑 harness 档探针或无依赖小 crate 量出来的。

## 0　现场核对（接着做之前）

| 项 | 做法 | 结果 |
|---|---|---|
| 冻结副本对快照清单 | `cd tree; sha256sum -c research/prompts/m3-formal-code-b24-r1-snapshot/crates-sha256.txt \| awk '{print $NF}' \| sort \| uniq -c`，再 `wc -l` 清单 | 输出 `288 OK`，清单 288 行，没有一行不是 OK |
| 前腿落盘的东西 | `ls -la` 模型目录与 `out/` | `k1_model.py`、`k1_run.sh`、`probe/` 三份探针、`out/` 7 份日志都在；报告没有 |
| 前腿 K1 小模型 | 给 `k1_model.py` 加三条改坏（`take13`、`last12-window`、`last16-window`，用 `research/scripts/replace-once.py` 改），全部 20 个变异目录重跑 `k1_run.sh`（正常取样与加强取样各一遍） | 输出落 `out/r2-k1-rows-and-extras.log`、`out/r2-k1-stronger-sampling.log`；前腿已有的变异结局逐个同 |
| 前腿 B1 探针 | `B24_GRID=base` 与 `drift` 各重跑一遍 | 汇总行与前腿日志逐字同（见 K2-a），落 `out/r2-b1-*-grid.summary.txt` |
| 前腿撕裂探针 | 重跑 | 4 passed，落 `out/r2-torn-device-zero.log` |
| 前腿真历史探针 | 前腿跑 48 操作那一遍中途 `error: test failed`、没留下数 | 重跑 64 史 × 16 操作、16 史 × 48 操作，两遍都 `test result: ok`，落 `out/r2-real-history-segments-*.log` |
| 负载与排队 | 开跑前 `ps` | 只有别的会话的 cargo / mutate；没有 qemu-system、vm-bench、e152、fio。第一次起 release 编译取 8G 上限被内存包装拒（slice 已占 34.2 GiB），改 4G 并设 `RUN_WITH_MEMORY_CAP_WAIT_SECONDS=120` 后都排上 |

没碰 checker 档：没有编、没有跑 `singlefs-checker-tier` 的任何测试二进制。副本里的探针都是 `singlefs-harness` 档；K1 的小 crate 把冻结副本里的真函数与真测试原文抽出来、无依赖，经内存包装跑。

## K1　F3-d 三条用例与六行变异

### K1-a　三条用例与六行变异，照判决与攻方表做了没有：一致（量的、对代码是读的）

判决要的形态（`research/prompts/m3-formal-code-b23-r1-main-verification.md:25`）：「改法 F3-d 进下一批：段号 0..=63 各 4095 个子集、长 28 的段末 12 个写 4095 种、历史种子 0 与 1，种子各自两两不同；照攻方表各配变异」。

| 判决要的 | 代码里 | 对上 |
|---|---|---|
| 段号 0..=63 各 4095 个子集 | `crates/singlefs-checker-tier/src/crash_injection.rs:4484`：「const SEGMENT_INDEXES: std::ops::RangeInclusive<usize> = 0..=63;」，共用搭建 `…:4443` 取 `(0..4095)` 个掩码（`…:4438`：「(1 << WRITES_VARIED_AT_THE_END_OF_THE_SEGMENT) - 1;」），段末变的个数 `…:4434`：「const WRITES_VARIED_AT_THE_END_OF_THE_SEGMENT: usize = 12;」 | 是 |
| 长 28 的段、末 12 个写 4095 种 | `…:4515`：「const WRITES_LANDED_BEFORE_THE_VARIED_END: usize = 16;」前缀 16 个全落了，加末 12 个写 = 28；段号 16 在 `…:4518` | 是 |
| 历史种子 0 与 1 | `…:4535`：「const HISTORY_SEEDS: [HistorySeed; 2] = [HistorySeed(0), HistorySeed(1)];」，段号 16、前缀 0 在 `…:4540`：「seed, 16, 0,」 | 是 |

攻方表的口径：攻方 `f3_fix` 的 A、B、C 三格定义（`research/prompts/m3-formal-code-b23-r1-opus-model/probe/opus_probe.rs:491` 起）与上面三条逐项同（A：`(0u64..64)`、长 12、历史种子 0；B：段号 16、`vec![true; 16]` 加 12 位；C：种子 0 与 1、段号 16、长 12）。

六行变异（`crates/mutations.tsv` 第 2436–2441 行；我的 `rowcheck.py` 按 `\n` 还原后在冻结副本里各数一次，见 `out/r2-tsv-rows-hit-once.txt`：七行（含 B1-c 那行 2442）都是 `orig count 1`）。每行改写后我在小 crate 里跑 F3-c 原用例加 F3-d 三条用例，量到的剩余种子数与攻方表逐个同：

| tsv 行 | 变异 | 红的用例（量的） | 剩余种子数（量的） | 攻方表同一格 |
|---|---|---|---|---|
| 2436 | 只折前 12 个写 | B | B = 1 | `f3-fix.txt:4` take(12) B=1 |
| 2437 | 只折前 16 个写 | B | B = 1 | `f3-fix.txt:5` take(16) B=1 |
| 2438 | 按位拼接 | A 与 C | A = 4096，C = 4096 | `f3-fix.txt:8` bitmask-shl A=4096 C=4096 |
| 2439 | 去段号 | A | A = 4095 | `f3-fix.txt:15` drop-segment A=4095 |
| 2440 | 去历史种子 | C | C = 4095 | `f3-fix.txt:13` drop-history-seed C=4095 |
| 2441 | 四样异或改成或 | A 与 C | A = 13254，C = 5998 | `f3-fix.txt:17` or-not-xor A=13254 C=5998 |

F3-c 原用例在六种下都没红（`out/r2-k1-rows-and-extras.log` 里每个 `row1`…`row6` 段里 `all_first_crash_states_of_a_segment_of_twelve_writes…` 都是 `ok`），与判决说的「F3-c 4095 断言抓不住这六种」同。没改坏时 4 条都绿（`== variant=none exit=0`）。

引文：攻方表几行逐字：
- `research/prompts/m3-formal-code-b23-r1-opus-model/out/f3-fix.txt:8`：「f3-fix variant=bitmask-shl inline=green A=4096/262080:red B=4095/4095:green C=4096/8190:red」
- `research/prompts/m3-formal-code-b23-r1-opus-model/out/f3-fix.txt:17`：「f3-fix variant=or-not-xor inline=green A=13254/262080:red B=4095/4095:green C=5998/8190:red」

推翻 K1-a 的现象：冻结副本里 `crash_injection.rs` 的取样常数与上表对不上；或 tsv 某行原文在冻结副本里命中不是恰好一次；或把那六个改写套进小 crate 之后某条剩余种子数与攻方表不同。三样我都核了，没出现。小 crate 与真测试的差别只在三个类型桩（`HistorySeed`、`StepPosition`、`CrashPoint`，形状抄冻结副本：`crates/singlefs-harness/src/history.rs:251`、`crates/singlefs-checker-tier/src/crash_injection.rs:254`），函数与测试体是原文。

### K1-b　六行变异是不是各自只让点名的那条红：部分（量的）

tsv 第 6 列点名的那条，六行改写下都红（上表），所以每行都被它点名的用例杀死、杀伤一条不缺。但「只让点名那条红」不成立：行 2438（按位拼接）与行 2441（或代异或）的点名条是 A，C 也同时红（4096 / 8190、5998 / 8190）。这与攻方表同一格一致（`f3-fix.txt:8`、`:17` 的 C 列都是 red），也与 C 条文档自己写的一致（`crates/singlefs-checker-tier/src/crash_injection.rs:4532`：「按位拼接剩 4096 个、四样改成或剩 5998 个」）。后果：这两行的杀伤依赖 A 一条，C 的多红不增加覆盖，但也没有害处；只在「点名的那条」意义上要读成「至少点名的那条红」。

推翻条件：改写后点名的那条没红（我量到的都红）；或点名的条之外红了 F3-c 原用例（没有）。

### K1-c　取样是不是覆盖了真历史会走到的段号与段长：冲突（量的；对实害的判断是推的）

并排两句。判决与代码这一边：

- `crates/singlefs-checker-tier/src/crash_injection.rs:4484`：「const SEGMENT_INDEXES: std::ops::RangeInclusive<usize> = 0..=63;」
- `crates/singlefs-checker-tier/src/crash_injection.rs:4515`：「const WRITES_LANDED_BEFORE_THE_VARIED_END: usize = 16;」（加 `…:4434` 的 12 个，段长 28）
- `crates/singlefs-checker-tier/src/crash_injection.rs:4535`：「const HISTORY_SEEDS: [HistorySeed; 2] = [HistorySeed(0), HistorySeed(1)];」

我在副本上量真历史这一边（探针 `probe/b24_segment_probe.rs` 在 harness 档照 `draw_crash_points` 的候选段口径数：`crates/singlefs-checker-tier/src/crash_injection.rs:3404`：「(marks.after_make_filesystem..marks.after_the_last_finished_step)」，探针里同一口径在 `research/prompts/m3-formal-code-b24-r1-sonnet-model/probe/b24_segment_probe.rs:48`：「(run.operations_written_by_make_filesystem..last).contains(&stream_indexes[*w])」）：

- `research/prompts/m3-formal-code-b24-r1-sonnet-model/out/r2-real-history-segments-64h16ops.log:5`：「max_segment_index=100 candidates_with_index_over_63=532 histories_with_such=37」（384 槽，64 史 × 16 操作，候选段 3932 个）
- `research/prompts/m3-formal-code-b24-r1-sonnet-model/out/r2-real-history-segments-64h16ops.log:7`：「longest_candidate=30 lengths_over_28=4」（4 GiB 那一档，同 64 史 × 16 操作）
- `research/prompts/m3-formal-code-b24-r1-sonnet-model/out/r2-real-history-segments-16h48ops.log:6`：「max_segment_index=222 candidates_with_index_over_63=1576 histories_with_such=16」（384 槽，16 史 × 48 操作，候选段 2520 个）

真用例里喂的历史长度（读码）：`crates/singlefs-checker-tier/src/crash_injection.rs:5699`：「let history = generate_history(HistorySeed(seed), 16);」；`crates/singlefs-checker-tier/tests/crash_injection_campaign.rs:1182`：「let history = generate_history(HistorySeed(5), 18);」。16 操作的历史里已有 37/64 个历史带段号 > 63 的候选段，候选段共 532/3932 个。
段长这边我量的直方图与攻方 b23 的段长表逐格同（`research/prompts/m3-formal-code-b23-r1-opus-model/out/f3-segments.txt:3`：「longest=30 histogram={1: 968, 2: 2147, 12: 9, 16: 216, 18: 55, 20: 82, 22: 48, 24: 279, 26: 16, 28: 108, 30: 4}」，我的 `…r2-real-history-segments-64h16ops.log:7` 同一个 histogram），4 GiB 档有 4 个 30 个写的候选段，B 的段长 28 看不见最后两个写。

判定：判决要的三条（段号 0..=63、段长 28、历史种子 0 与 1）代码照做了（K1-a），但三条的取样范围没有罩住真历史会走到的：段号到 100（16 操作）/ 222（48 操作）、段长到 30、历史种子远不止 0 与 1。这不是 24-1 改错——判决 `research/prompts/m3-formal-code-b23-r1-main-verification.md:25` 自己定的就是 0..=63 与 28，那一行里攻方写的是「用例固定种子 0、段号 16、段长 12，真历史候选段长到 26 / 30」，而 0..=63 没有量过真历史段号上界。

推翻条件：真用例喂的历史里候选段号从不超过 63（我量到 100 与 222）；或 4 GiB 档候选段长从不超过 28（我量到 30，4 个）。

### K1-d　还有没有第七种有害改坏三条都放过：有，四类（量的剩余种子数在小 crate 上；是否「有害」的实害是推的）

在小 crate 上，除了 tsv 那六行，再改写别的（原文同样在冻结副本里恰好命中一次），跑 F3-c 原用例与 F3-d 三条用例。正常取样下 `exit=0` 的是三条加原用例全绿，即「放过」：

| 改坏 | 正常取样（日志行） | 加强取样（A 段号 0..=127、B 前缀 20 个写即段长 32、C 历史种子 0..128；日志行） | 为什么算有害（推的） |
|---|---|---|---|
| 段号只取低 6 位 | 全绿（`research/prompts/m3-formal-code-b24-r1-sonnet-model/out/r2-k1-rows-and-extras.log:129`：「== variant=x_segment-and-63 exit=0」） | A 红（`research/prompts/m3-formal-code-b24-r1-sonnet-model/out/r2-k1-stronger-sampling.log:135`：「== variant=x_segment-and-63 exit=101」） | 真历史段号到 100：段 70 与段 6 同一子集同一历史种子时二次崩溃种子相同；候选段里 1 个写与 2 个写的占 3115/3932，它们的子集只有几种，撞的概率高 |
| 只折前 28 / 29 个写 | 全绿（`…:163`：「== variant=x_take28 exit=0」、`…:170`：「== variant=x_take29 exit=0」） | B 红（`out/r2-k1-stronger-sampling.log:172`、`:182` 两行 `exit=101`） | 4 GiB 档真有长 30 的候选段，最后两个写被丢掉 |
| 历史种子只取低 1 位 / 低 6 位 | 全绿（`…:81`：「== variant=x_history-seed-and-1 exit=0」、`…:88`） | C 红（`out/r2-k1-stronger-sampling.log:81`、`:91` 两行 `exit=101`） | 真跑的历史种子远不止 0 与 1，种子差 2 的倍数的两段历史会摆出同一串二次崩溃 |
| 只折段末 12 / 16 个写（`skip(len − 12)` / `skip(len − 16)`） | 全绿（`…:95`：「== variant=x_last12-window exit=0」、`…:102`：「== variant=x_last16-window exit=0」） | 仍全绿（`research/prompts/m3-formal-code-b24-r1-sonnet-model/out/r2-k1-stronger-sampling.log:101`：「== variant=x_last12-window exit=0」、`:108`） | 是 `.take(12)` 的镜像：B 只在段末 12 个写上变、前缀钉死，只折末尾窗口的指纹看不见前缀；这正是 b23 判决 K1 行里正推腿点名过的「末 12 个写窗口」（`research/prompts/m3-formal-code-b23-r1-main-verification.md:25`：「抓得住折得太少的，抓不住 `.take(12)`、`.take(13)`、末 12 个写窗口、低 32 位、掩码当整数」），F3-d 的变异表没有配它 |

另量了几条，不算第七种：
- 去盐（`seed.0` 不异或 `SECOND_CRASH_POINT_SEED_SALT`）：四处取样（含加强）都全绿（`…:74`：「== variant=x_drop-salt exit=0」）。异或一个常数不改变种子两两是否不同，是等价改动，不算有害；攻方表同一格也全绿（`research/prompts/m3-formal-code-b23-r1-opus-model/out/f3-fix.txt:14`：「f3-fix variant=drop-salt inline=green A=262080/262080:green B=4095/4095:green C=8190/8190:green」）。
- 指纹截成低 32 位：全绿（`research/prompts/m3-formal-code-b23-r1-opus-model/out/f3-fix.txt:9`：「f3-fix variant=fnv-as-u32 inline=green A=262080/262080:green B=4095/4095:green C=8190/8190:green」），4095 个子集在 32 位里仍两两不同，实害只在长段上按生日界极小，不算。
- 段号只取低 8 位（`& 255`、`as u8`）：正常与加强取样都全绿（`…:122`、`…:136`；加强下 `out/r2-k1-stronger-sampling.log:128`、`:145`）。我量到的段号上界 222 < 256，今天用例喂得到的历史里不撞；历史再长就会撞，属条件式，不算今天的有害。
- 只把段号那一样改成或：A、C 都红（`…:109`：「== variant=x_or-segment-only exit=101」），已被抓。
- 只折前 13、前 24 个写：B 红（`…:143`、`…:153` 两行 `exit=101`），已被抓。

这四类是零轮形态：按正文第五节，这是最后一轮，不再为它们开轮，写进判决的交用户表或记账。每一类若要入表，原文改写在 `k1_model.py` 的 `EXTRA` 字典里，原文各命中一次。
有害与否按 `crates/singlefs-checker-tier/src/crash_injection.rs:2837` 的口径读：「只混段号时同一段里的各个第一次崩溃态用同一串随机数，挂载写表相同就摆到同一组二次崩溃）。」——种子相同的后果是二次崩溃摆点失去多样性，不是误报；checker 档没跑，实害的大小是推的。

推翻条件：加强取样下这四类里任何一类仍红之外的结局（我量到：前三类转红，末窗口仍绿）；或能指出真用例里不会出现段号 > 63、段长 > 28、历史种子多于 2 个（我量到前两样出现，后一样是读码：真跑的种子是 `HistorySeed(seed)`，`seed` 取不同值）。

## K2　B1-c 新前提与四处注释

### K2-a　B1-c 新前提照判决做了没有、有没有钉住「建 0 个」变异的判别力：一致（量的），前提与变异同真同假，但两边是同一个调用

判决要的（`research/prompts/m3-formal-code-b23-r1-main-verification.md:29`）：「改法 B1-c 进下一批：前提块加」。代码里（冻结副本）：

- `crates/singlefs-checker-tier/tests/crash_injection_publish_after_a_mount_whose_deletion_continuation_stays_short.rs:160`：「let publish_creating_no_inode = publish_new_inodes(」，个数参数 `…:164`：「0,」，断言 `…:169`：「publish_creating_no_inode.is_ok(),」。前提块在原有两条前提（`…:131` 起的续做没推完、`…:142` 起的 nlink = 0）之后、真发布 `…:174` 之前，同一份 `crash_image` 上另挂一次（`…:149`：「let mut devices = plain_devices_on(&crash_image);」）。
- 被它守着的那行变异改的是 `crates/singlefs-checker-tier/src/crash_injection.rs:1882`：「PoolVersion::WithFile(current) => publish_new_inodes(」那一支的个数 `…:1886`（`1,`，今天值）；tsv 第 2413 行（B1）把它改成 `0,`。

**我怎么量的**：副本里的 harness 档探针 `probe/b24_b1_probe.rs` 在盘面参数的网格上每个格子做三件事——造盘面（抄 B1 用例的 `crash_image_between_the_deletion_batches`，覆盖写次数、单元数、批数与单元区槽数当参数）、原用例的前两条前提与终判（「旧用例绿」）、(i) B1-c 加的前提块（抄原文）(ii) 发布臂照 `mount_and_publish_once_on_the_crash_image` 把「建几个」当参数跑建 0 个（即变异结局）。每个旧用例绿的格子比：前提成立 ⇔ 「建 0 个」变异红（结局是 `Published`，不是单元区墙）。两张网格，都是重跑的原样输出（`out/r2-b1-base-grid.summary.txt`、`out/r2-b1-drift-grid.summary.txt` 第 1 行）：

- 基线网格（覆盖写 3、14 个单元、1 批，单元区槽数 110..=420 步 1）：`research/prompts/m3-formal-code-b24-r1-sonnet-model/out/r2-b1-base-grid.summary.txt:1`：「b24-b1 mode=base cells_built=293 build_failed=18 old_test_green=11 new_test_green_(premise_ok)=5 green_cells_where_zero_mutation_red_and_only_some_red=5 green_cells_where_only_some_red_zero_not_red=6 disagreements_between_premise_and_zero_mutation=0」
- 漂移网格（覆盖写 1..=6、单元数 {10,12,14,16,18}、批 1..=2、槽 100..=330 步 2，6960 格里 1093 个建不出盘面）：`research/prompts/m3-formal-code-b24-r1-sonnet-model/out/r2-b1-drift-grid.summary.txt:1`：「b24-b1 mode=drift cells_built=5867 build_failed=1093 old_test_green=244 new_test_green_(premise_ok)=117 green_cells_where_zero_mutation_red_and_only_some_red=117 green_cells_where_only_some_red_zero_not_red=127 disagreements_between_premise_and_zero_mutation=0」

读法：旧用例绿的 244 格里，前提成立的 117 格正是「建 0 个」变异红的 117 格，前提为假的 127 格正是变异不红、只有「只看 Some」那行红的 127 格，逐格零分歧；127 格的假因全是 `PlacementRefused`（`premise_failure_kinds={"PlacementRefused": 127}`）。也就是旧用例在这 127 格上是瞎的，新前提把它们全拦下。判决的「漂 3–8 槽」那一段在基线网格里逐槽可见：
- `research/prompts/m3-formal-code-b24-r1-sonnet-model/out/r2-b1-base-cells-126-140.txt:6`：「cell slots=133 ow=3 units=14 batches=1 old_test=green new_premise_ok=false mutation_zero_red=false」
- `research/prompts/m3-formal-code-b24-r1-sonnet-model/out/r2-b1-base-cells-126-140.txt:7`：「cell slots=134 ow=3 units=14 batches=1 old_test=green new_premise_ok=true mutation_zero_red=true」
128..=133 为假、134..=138 为真，与判决说的「盘面往下漂 3–8 槽」（136 − 8 = 128、136 − 3 = 133）对得上；tsv 第 2442 行把 `UNIT_AREA_SLOTS` 换成 133（冻结副本 `crates/mutations.tsv` 第 2442 行，该行起头是「崩溃注入第二截的发布臂（24-1 改法 B1-c）：盘面挪到单元区 133 槽」；主仓今天那一行在别的行号，所以不写成行号引文），在 133 槽上旧用例绿、前提为假，所以红在这条前提上，与行文「红在这条前提上」一致。

攻方 b23 的数据也与它对得上：`cat b1-slots.txt b1-neighbourhood.txt | grep class=GREEN | …` 把「旧用例绿」的行按（类、create0 结局）数，原样输出：

```
     36 class=GREEN-KILLS-BOTH create0=Published
     40 class=GREEN-KILLS-SOME-ONLY create0=Explained(UnitAreaWall)
```

36 + 40 = 76，与注释「用例绿的 76 份盘面」同；`create0=Published` ⇔ 「建 0 个」变异红逐行同。

**不独立的地方（照实写）**：前提块与变异臂都是对挂载交回的现行一版调 `publish_new_inodes(…, 0, …)`，所以「同真同假」主要是确认两边的盘面、设备包装与挂载起法没有分叉，不是两条不共用调用的独立路径：前提用 `plain_devices_on`（`crates/singlefs-harness/tests/common_admission_pool/mod.rs:396`：「SparseBlockDevice::new(image.device_size_in_bytes, PhysicalBlockSizeInBytes(512));」），变异臂用 `RecordingBlockDevice` 加建池参数的物理块大小；建池参数里那个数也是 512（`crates/singlefs-harness/src/history.rs:198`：「let mut parameters = e142_parameters(512, 512);」），所以两条分不出差别。网格只覆盖 `UnitAreaOf384Slots` 这一档，参数是上面写的范围；4 GiB 档与更大的单元数没有量。

推翻条件：任何一格旧用例绿而前提与变异不同真假（我量到 0 格 / 255 个旧用例绿的格子，两张网格共建成 6160 格）；或 `plain_devices_on` 与变异臂的挂载在某个盘面上走出不同的现行版本；或实现把「建 0 个」改成不经 `publish_new_inodes`。

### K2-b　四处注释逐句对代码：四处都对得上（读码、量的、推的分开标）

**① `writable_mount_refusal_on_a_crash_state_without_read_faults.rs` 模块文档（冻结副本第 4–6 行）：一致**

| 注释说的 | 代码 | 对上 |
|---|---|---|
| 撕裂两槽那一形上 C670 那一条「照常比（跳过撕裂的槽取 c_见证）」（`crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs:5`：「在这一形上照常比（跳过撕裂的槽取 c_见证）」） | `crates/singlefs-harness/src/mount_transparency.rs:1489`：「自证不过的槽（撕裂的、没写过的）跳过」；`…:1491`：「那一判按真、这里照常比，」（`…:1517` 取 journal_tail 最大值，`…:1518` 的 `.max()`） | 是（读码） |
| 「用例那一份盘面上复算为假、不报」 | 用例盘 0 两槽撕裂、盘 1 没轮换，c_见证 等于第一个文件那一版末条的计数器 ⇒ 判据假；测试断言结果是合法（`…:313`：「WritableMountRefusalVerdict::Legal(」）而不是 `Illegal` | 是（测试自己的断言，harness 档，副本里 4 条都绿，见下） |
| 「换一份盘面照常比可以为真」（`…:5` 同一行） | 我在副本里写探针：只落轮换的盘面上再把盘 0 两个系统配置槽都撕裂，盘 1 的轮换保留；原样输出 `research/prompts/m3-formal-code-b24-r1-sonnet-model/out/r2-torn-device-zero.log:8`：「b24-torn verdict = Illegal { reason: NewerPublishWitnessedBySystemConfiguration, conditions_not_met: [WitnessCriterionTrueOnACrashImageWithoutReadFaults(WitnessedCounterAboveTheLastRecordOfTheSelectedVersion)] }」 | 是（量的）：同一形（盘 0 一份自证过的都没有）换盘面，C670 那一条报真 |
| 「拒照 U6 判——系统配置见证照 R2 判得出（撕裂不算判不出）」（`…:5`） | `crates/singlefs-harness/src/mount_transparency.rs:1360`：「撕裂的轮换（读得出、自证不过，设备没报错）不算判不出；一块盘报读错而另一块盘两槽都自证过也不算。」 | 是（读码） |
| 「判据按『某块盘一份自证过的都没有』那一支为真」（`…:6`） | `crates/singlefs-harness/src/mount_transparency.rs:1383`：「.any(|slots| slots.self_verified.is_empty())」，`…:1413` 起把它与 R2 判不出并成「按真」 | 是（读码） |

副本里原文件的三条测试加我的探针一起跑：`research/prompts/m3-formal-code-b24-r1-sonnet-model/out/r2-torn-device-zero.log:13`：「test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s」（harness 档，副本）。推翻条件：换一份盘面撕裂盘 0 两槽、盘 1 轮换保留，判官仍判合法（我量到 Illegal）。

**② H2 常数注释（`pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs` 第 59–66 行）：一致**

| 注释说的 | 依据 | 对上 |
|---|---|---|
| 第 k 次挂载上第一个文件版本 txg = 3k − 1，从 k ≥ 2 起成立，第 1 次是 3、第 2 次 5、第 3 次 8 | 前一轮正推腿的探针原样输出 `research/prompts/m3-formal-code-b23-r1-sonnet-model/out/h2-txg.log:18`：「k=1:txg=3(3k-1=2)!!」、`:19`：「k=2:txg=5(3k-1=5)」、`:20`：「k=3:txg=8(3k-1=8)」 | 是（旧日志，我没重跑那个探针） |
| k = S + 2 时旧根是 5 ≥ 释放代 3，k = S + 1 时是 2 < 3 | 算术：`3(S+2) − 1 − 3S = 5`、`3(S+1) − 1 − 3S = 2`；释放代常数 `…:58`：「const RELEASE_TXG_OF_THE_MKFS_INSTANCE_TABLE: u64 = 3;」 | 是（算的） |
| S ≥ 5 时两个 k 都 ≥ 6 | k = S + 1 ≥ 6、S + 2 ≥ 7 | 是（算的） |
| 今天 S = 8，第 10 次挂载、txg 29 | `crates/singlefs-format/src/lib.rs:279`：「pub const ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM: u64 = 8;」，`3·10 − 1 = 29`，h2-txg.log 同（`research/prompts/m3-formal-code-b23-r1-sonnet-model/out/h2-txg.log:25` 一行，k=10） | 是 |
| S 只能取 5–15，4、16 编不过 | `crates/singlefs-core/src/root_ring.rs:50`：「slots_per_region > ROOT_RING_SLOTS_PER_REGION_MINIMUM」与 `…:51` 的 `< ROOT_RING_SLOTS_PER_REGION_MAXIMUM`；`crates/singlefs-format/src/lib.rs:270`：「pub const ROOT_RING_SLOTS_PER_REGION_MINIMUM: u64 = 4;」、`…:275`：「pub const ROOT_RING_SLOTS_PER_REGION_MAXIMUM: u64 = 16;」；前一轮日志 `research/prompts/m3-formal-code-b23-r1-sonnet-model/out/h2-sweep.log:97`：「=== S=16 variant=orig ===」后面是 `E0080` 编不过 | 是 |
| 正推在 S = 5、6、7、8、12、15 上量过两条用例绿、放宽一个 txg 的变异红 | 前一轮 `h2-sweep.log` 里这六个 S 的 `orig` 段都 `rc(pipeline)=0`、`slack1` 段都 `rc(pipeline)=101`（`grep -nE '=== S=(5|6|7|8|12|15) variant='` 命中 12 行） | 是（旧日志，没重跑） |

判决 K3 行说「常数注释『16 上量过』照现值改」（`research/prompts/m3-formal-code-b23-r1-main-verification.md` K3 H2 前提断言那一行）：现注释写的是 15 与 5–15，16 已不出现，与 `root_ring.rs:51` 的严格小于对得上。推翻条件：S 取 5–15 里某一个时两条用例红、或放宽一个 txg 的变异不红（前一轮六个点上都没有；我没重量 9、10、11、13、14）。

**③ `crates/singlefs-core/src/root_record.rs` 卸载记号那句（冻结副本第 22–24 行与第 29 行）：一致**

| 注释说的 | 代码 | 对上 |
|---|---|---|
| 施加出来的那一版取那条记录新根段里的记号（`crates/singlefs-core/src/root_record.rs:22`：「由记录施加出来的那一版不归下面哪一个成员固定：它取那条记录新根段里的记号」） | `crates/singlefs-core/src/recovery.rs:3601`：「unmount_marker: record.new_unmount_marker,」，上一行注释 `…:3596`：「每条记录都带整次发布的新根段（已定项 15），施加末条那一份」；kb 条款 `.claude/kb/decisions/23-journal的角色与格式.md:451`：「实例表指针、卸载记号、分配记录树根从新根段来，不再照抄所选根」 | 是 |
| 记录的新根段照抄那次发布写的根上的记号（`crates/singlefs-core/src/root_record.rs:23`：「记录的新根段照抄那次发布写的根上的记号」） | `crates/singlefs-core/src/transaction.rs:1399`：「new_unmount_marker: root.unmount_marker,」（空发布那一条）与 `…:1936` 同一句（多记录发布的每一条）；`…:1392`：「新根段七项取这次发布写的根上的」 | 是 |
| 卸载那一串发布的记录带 1、别的发布的记录带 0（`crates/singlefs-core/src/root_record.rs:24`：「别的发布的记录带 0。」） | 非测试代码里只有一处造 `WrittenByTheUnmountSequence`：`crates/singlefs-core/src/mount.rs:2436`：「RaiseFloorEntry::Unmount => UnmountMarker::WrittenByTheUnmountSequence,」，卸载入口的串 `…:2428`：「这一串的根全带卸载记号」；`publish_version_with_unmount_marker(` 只有 `mount.rs:3147` 一个非定义的调用（`transaction.rs:6156` 是它的包装，`transaction.rs:6166` 注释：「只有正常卸载那一串」）。`grep -rn 'UnmountMarker::WrittenByTheUnmountSequence' crates/singlefs-core/src` 原样输出 8 行：`root_record.rs` 的 23、38、59、238、261（238、261 是它的单元测试），`journal.rs:498`（测试），`mount.rs:2436`，`transaction.rs:6166`（注释） | 是（读码） |
| 记号 0 那一支「别的每一条根——mkfs 的第 0 代、暖机、写行、发布、准入抬 F、回退」（`crates/singlefs-core/src/root_record.rs:29`：「flags 位 0 = 0：别的每一条根——mkfs 的第 0 代、暖机、写行、发布、准入抬 F、回退。」） | 这几处都造 `NotWrittenByTheUnmountSequence`：`make_filesystem.rs:581`、`transaction.rs:1368`、`transaction.rs:1886`、`mount.rs:3526`、`mount.rs:3586`；准入抬 F 是 `crates/singlefs-core/src/mount.rs:2435`：「RaiseFloorEntry::Admission => UnmountMarker::NotWrittenByTheUnmountSequence,」 | 是（读码；我没逐个核「回退」那一入口，该入口在同一批 grep 命中的 `NotWritten` 里） |

判决 K3 行指的毛病（「挂在『= 0』变体下易读成记号恒为 0」）已解：施加那句现在挂在枚举 `UnmountMarker` 的文档上（`…root_record.rs:22`），不在 `NotWrittenByTheUnmountSequence` 变体下。推翻条件：`recovery.rs:3601` 以外还有一处施加记录的地方不取 `new_unmount_marker`（我 grep 到的只有这一处）；或有第二处非测试代码造 `WrittenByTheUnmountSequence`（我 grep 到 1 处）。

**④ B1 文件那句注释（冻结副本第 40–41 行；正文说的「第 40 行」是这句的起头，改动的几个字在第 41 行）：一致**

`crates/singlefs-checker-tier/tests/crash_injection_publish_after_a_mount_whose_deletion_continuation_stays_short.rs:41`：「上建 0 个也被落点拒（`PlacementRefused`）、照样记 `UnitAreaWall`，分不开那一支建几个。」

| 句里说的 | 代码或量的 | 对上 |
|---|---|---|
| 128 槽上建 0 个被落点拒（`PlacementRefused`） | 量的：`research/prompts/m3-formal-code-b24-r1-sonnet-model/out/r2-b1-base-cells-126-140.txt:1`：「cell slots=128 ow=3 units=14 batches=1 old_test=green new_premise_ok=false mutation_zero_red=false (zero-arm: Refused(Explained(UnitAreaWall), "PlacementRefused"))」；攻方 b23 同格 `research/prompts/m3-formal-code-b23-r1-opus-model/out/b1-slots.txt:22`（133 槽）：「create0=Explained(UnitAreaWall)/PlacementRefused[]」 | 是 |
| 照样记 `UnitAreaWall` | `crates/singlefs-harness/src/model_comparison.rs:669`：「PublishError::PlacementRefused { refusal, .. } => {」进 `refusal_reason_of_placement_refusal`，那里 `…:728`：「PlacementRefusal::NoFreeSlotOnAnyDevice => explained(ModelRefusalReason::UnitAreaWall),」 | 是（读码） |
| 不是「被空间准入拒」（原句是这样写的，diff 里改掉了） | 空间准入的拒是另一条：`crates/singlefs-harness/src/model_comparison.rs:673`：「PublishError::SpaceAdmissionRefused(_) => explained(ModelRefusalReason::UnitAreaWall),」；在这几档上被空间准入拒的是建 1 个，不是建 0 个：`research/prompts/m3-formal-code-b23-r1-opus-model/out/b1-slots.txt:22`：「create1=Explained(UnitAreaWall)/SpaceAdmissionRefused[」 | 是：改后的措辞对，原句把「建 0 个」与「建 1 个」的拒法混了 |
| 128 槽（第 40 行说的是攻方探针按 100..=320 步 2 扫出的第一个短续做盘面） | 这一句没改；那次扫描是 b21 攻方的探针，我没复核 | 没核（不在这一批的改动里） |

推翻条件：128 槽盘面上建 0 个的结局不是 `PlacementRefused`（我量到是）；或 `refusal_reason_of_placement_refusal` 把 `NoFreeSlotOnAnyDevice` 映射到别的拒因（读码是 `UnitAreaWall`）。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1-a 三条用例与六行变异照判决与攻方表 | 一致（量的） | A/B/C 取样与 `f3_fix` 逐项同；六行原文各命中一次；改后的剩余种子数 1、1、4096、4095、4095、13254 与攻方表逐格同 |
| K1-b 六行是否各自只让点名的那条红 | 部分（量的） | 点名的那条都红；行 2438、2441 另红 C（攻方表同一格也是），F3-c 原用例六种下全绿 |
| K1-c 取样是否罩住真历史的段号与段长 | 冲突（量的） | 代码 `0..=63`、段长 28、种子 0 与 1；真历史段号到 100（16 操作）/ 222（48 操作）、段长到 30；判决自己就是这样定的 |
| K1-d 第七种有害改坏 | 有（量的剩余种子数；实害是推的） | 段号取低 6 位、只折前 28/29 个写、历史种子取低位、只折段末 12/16 个写，三条加原用例全放过；加强取样下前三类转红，末窗口仍放过 |
| K2-a B1-c 新前提钉住「建 0 个」 | 一致（量的），但同调用 | 旧用例绿的 255 格里前提成立 ⇔ 「建 0 个」变异红，0 分歧；133 槽前提为假；前提与变异臂是同一个调用，不是独立两路 |
| K2-b① writable_mount 模块文档 | 一致 | 跳过撕裂槽照常比、R2、某块盘无自证槽按真三句读码对上；「换一份盘面可以为真」在副本探针上量出 Illegal |
| K2-b② H2 常数注释 | 一致 | 3k−1 从 k ≥ 2、S + 1 / S + 2 的算术、S 取 5–15、今天 S = 8，都对上；txg 与 S 扫描引的是前一轮旧日志，没重跑 |
| K2-b③ root_record 卸载记号 | 一致（读码） | 恢复施加取 `record.new_unmount_marker`，记录照抄发布写的根，非测试代码里只有卸载那一串造 1 |
| K2-b④ B1 文件第 40–41 行 | 一致 | 128 槽建 0 个被落点拒、记 `UnitAreaWall`；原句「被空间准入拒」确实错，改后对 |

## 没做什么

- checker 档没编也没跑：F3-d 三条用例、B1-c 前提块、tsv 七行变异（2436–2442）在真 `singlefs-checker-tier` 里的编译与运行都没做。K1 的数是把冻结副本里的真函数与真测试原文抽进无依赖小 crate 量的；K2-a 的数是 harness 档探针量的。所以关于这些改动「在 checker 档会不会红」只能说「推的」。
- 副本上的数不进 kb：照 `.claude/rules/three-way-inference.md` 那一条，要引得由主 agent 在入库装置上重做。
- 没重跑前一轮正推腿的 H2 探针（`h2-txg.log`、`h2-sweep.log` 是旧日志，原样引用，没重跑）；没量 S = 9、10、11、13、14。
- 没核 B1 文件第 38–40 行里 b21 的旧引文（`k2b.txt` 第 2、4、6 行、100..=320 步 2 的扫描），不在这一批改动里。
- 没量 4 GiB 档的 B1 网格，也没量操作数大于 48 的历史上的段号。
- 判别腿的格（攻方的）与判决：不采纳、不判；上面 K1-c、K1-d 的「零轮形态」交主 agent 写进判决的交用户表或记账。
- 前腿留下的 `out/b1-drift-grid.log`（2.9 MB，满是建不出盘面的 panic 回溯）没删，引用的是我重跑后的汇总 `out/r2-b1-*-grid.summary.txt`。

## 模型、脚本与复跑

模型目录 `research/prompts/m3-formal-code-b24-r1-sonnet-model/`：`k1_model.py`（小 crate 生成器，含 `EXTRA` 改坏表与 `--stronger`）、`k1_run.sh`（逐个变异目录经内存包装跑）、`rowcheck.py`（tsv 七行原文命中次数）、`setup_copy.sh`（冻结副本拷成编译副本并放三份探针）、`probe/`（`b24_segment_probe.rs`、`b24_b1_probe.rs`、`b24_torn_probe_tail.rs`）、`out/`（前腿 7 份旧日志，加我的 `r2-*` 重跑输出）。
复跑 K1：`python3 k1_model.py <工作目录> all [--stronger]`，再 `bash k1_run.sh <工作目录>`（每个变异一个目录，约 7 秒）；复跑探针：`bash setup_copy.sh`，在副本里 `cargo test --offline --release -p singlefs-harness --test b24_b1_probe`（`B24_GRID=base|drift`）、`--test b24_segment_probe`（`B24_HISTORIES`、`B24_OPERATIONS`）、`--test b24_torn_probe`，都经 `run-with-memory-cap.sh` 与 `capped.sh 10`。没有要主 agent 起的长活。

输出文件的 sha256（我写进 `out/` 与模型目录的）见草稿 `/tmp/claude-1000/m3-formal-code-b24-r1-sonnet/sha.txt`；交回前我在末尾另给报告自己的 sha256。

## 清理

我建的编译目录与副本都已删：`/tmp/claude-1000/m3-formal-code-b24-r1-sonnet/copy`（冻结副本拷成的编译副本，865M）、`…/target-rel`（release 编译目录，660M）、`…/k1work`、`…/k1strong`、`…/k1work2`、`…/k1strong2`（小 crate 变异目录）。要复跑照上一节：`setup_copy.sh` 重建副本，`k1_model.py` 重生成小 crate。草稿目录里留的只有日志与小脚本，没有编译目录与工作树。
