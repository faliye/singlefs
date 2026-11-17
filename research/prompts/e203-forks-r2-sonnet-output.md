# e203-forks-r2 辩方腿（Sonnet）报告：复核第一轮判决第二节七格与第三节交用户表（2026-11-07）

腿：`three-way-defense`，辩的方：被第一轮判出局或被压成下界的那几方（B1 的下界用法、B8 的 F2 一格、B4 的读法）。草稿目录 `/home/fy5090/.local/share/singlefs/work/e203-forks-r2-sonnet/`。禁读清单里的文件没读。

## 零、读数怎么来的、状态怎么标

- 输入：`~/.local/share/singlefs/work/e203-runner-revision-7/report-inputs/` 下 36 份产物（下面逐份 sha256），任务 10180 日志（`/home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-10180.attempt-1.log`），冻结源树 `~/.local/share/singlefs/work/e203-runner-segment-2-r2/pristine-source/`。
- 我写了一批只读产物的 python 脚本（`tools/*.py`，不写仓、不编译、不起项目代码）直接跑过；它们的读数在主 agent 复跑之前一律是「推的，待主 agent 跑」。能和任务日志里已有的行逐位对上的，我在读数旁写了日志行号，当作交叉印证（印证用的是日志，不是我的脚本）。
- 我没往服务交任何任务；`tools/drop_vs_clone.rs`、`tools/drop-vs-clone.sh` 写好没编没跑，命令在「要主 agent 跑的命令」一节。
- 复现算法（B8 步数）：`tools/b8port.py` 把模型 `scripts/experiment-apparatus/e203_admission_scan_cost.rs` 里 `by_delta_input_from_the_product`（第 2353–2517 行）与 `by_delta_rows`（第 185–313 行）逐字段搬成 python，读计数产物的 `name=row`、`name=state`、`name=publish`、`name=delta` 四种行，对 caller、before_landing、between 三段各算一遍再逐行相加。校验：它算出的同挂载覆盖写 F 轴两端 2102 与 152344、α=0.620064，与日志第 2216 行逐位相同；U 轴 0.049137、u 轴 0.261401 与日志第 2207、2210 行逐位相同。
- 复现算法（Q10 残差）：`tools/q10.py` 读计时产物的 `name=segment_nanos` 与各行 `exclusive_nanos`，残差 = 段挂钟 − 该段各行独占之和，点轴值从同名计数产物的 `name=point` 行取；校验：日志第 2863、2864、2872 行的五个 α 与它算的逐位相同到 6 位小数。

### 产物 sha256（`sha256sum *.out`，36 份）

```
a5fb144f775ac30639aa5ba2a62f56bfb83b14ca432594a8545619c5c7219926  e203-admission-scan-cost-counts-base-2026-11-07.out
5859f7fdc94686d36c0d75bf1ff4aefb34fa7200b54723cd1bd631d49e7850c8  e203-admission-scan-cost-counts-data_units_10000-2026-11-07.out
f18c50762963333da50f9fe6c042ecc3c8205394b01a697baf5eb0c9aad56f43  e203-admission-scan-cost-counts-data_units_1000-2026-11-07.out
63bc9aa100886ae45a1ee8dd1c84c955f5156d66087188a0c032ce8c274d5666  e203-admission-scan-cost-counts-data_units_100-2026-11-07.out
e0df6c989f7b1a7d0a8123913b331066f3faacd17ec002f12be01b9ef867b51d  e203-admission-scan-cost-counts-data_units_1-2026-11-07.out
a72158eb16d6a589bd1eafad2eaefd1df8ccaca2eb08c1f9c53a7b5f27186b08  e203-admission-scan-cost-counts-diagonal-2026-11-07.out
7df7e41e87efc0fca732787c02b4b7bfb5a3c3b5acc3626e7fc176643dde5cec  e203-admission-scan-cost-counts-files_10000-2026-11-07.out
fe225b94497d9345732c1f8ba0a93bbc1d3f6c72624388a7043564fe52a82199  e203-admission-scan-cost-counts-files_1000-2026-11-07.out
7995bcf1e426acd6bfaa16f00573b6f8e458c86961f9dae509e8c1933f4df009  e203-admission-scan-cost-counts-files_100-2026-11-07.out
60a4f08b352d5683837d876e2aeb09ddf55d938e743c15fd6a8c2f7cf37eae4c  e203-admission-scan-cost-counts-files_1-2026-11-07.out
ae4b6ada91f7afe0916a27ae71054207990b38b9a0246853afca816dfed2cee7  e203-admission-scan-cost-counts-unit_area_2_pow_14-2026-11-07.out
bcef4e8dd70206e21b894b67cbeea2ca1398cf0874d03f8a86987472de4fa53b  e203-admission-scan-cost-counts-unit_area_2_pow_20-2026-11-07.out
7f1bccda497db46e430e2f1950b0e9161a1c6ef26f82bfce015dc9a11cce69fe  e203-admission-scan-cost-counts-unit_area_2_pow_24-2026-11-07.out
02570477573a3497e43152fac1eb8797f313672cf80bc348063861b9a0bf1b23  e203-admission-scan-cost-counts-unlinks_10000-2026-11-07.out
7f87980a77e2c2acc442f5e69a40b33fd3d88e68cab425a0e92d406321237085  e203-admission-scan-cost-counts-unlinks_1000-2026-11-07.out
e09a4048c74c26eab8beba87f53cae2cc02acf70ebcbcc3dca4044699e7fc287  e203-admission-scan-cost-counts-unlinks_100-2026-11-07.out
68924d59b85f2ddc303b3c6cbaa025d2ffce0c19abbebf4407f9b38b2126dcfe  e203-admission-scan-cost-counts-unlinks_1-2026-11-07.out
337c9cf93ed50fec4e518697a7b54669228d903bf027d1576a7c694adf515626  e203-admission-scan-cost-counts-user_scale_unit_area-2026-11-07-r2.out
ae76c632fbe0d5e4fa03eb118ef2e1a96ddad58f5ff332aed3d455b847db7ca7  e203-admission-scan-cost-timing-base-2026-11-07.out
11b161116ced0ceea2fd6c2a10fa1de14e0eeefa71e9c864112584aba075d154  e203-admission-scan-cost-timing-data_units_10000-2026-11-07.out
ea553c65f38d3a45af5266bbb6790a0b046a49830c54cb90c68f358fdd67e570  e203-admission-scan-cost-timing-data_units_1000-2026-11-07.out
2d328fe6c063b8ca26fc3cab495349bddfbb3662bfd7c6464bc366bfd5beb582  e203-admission-scan-cost-timing-data_units_100-2026-11-07.out
c7e90370c771b9c0ba19cc373b276089281bb8b8ddcc16552fcf6d3bfada76ca  e203-admission-scan-cost-timing-data_units_1-2026-11-07.out
3cb4e57c98e8e9ba612e95a1366c64a6bc6df047b5be4f5a203923b79e11ca68  e203-admission-scan-cost-timing-diagonal-2026-11-07.out
ff1527895eae8b94cee1b1d49d2aa0643b548733310c131b2a9d2260759a2a4a  e203-admission-scan-cost-timing-files_10000-2026-11-07.out
5f729d2b6cad2f870935be2148a5a36db5e58179e0ae2514924bd2154d4ac03c  e203-admission-scan-cost-timing-files_1000-2026-11-07.out
7be38a7ea04a1ad2f3ea3a17faf0cc7ac594a36157e1972a9f49a3946adc50b7  e203-admission-scan-cost-timing-files_100-2026-11-07.out
a018639aee413725a7260015f168dffe518c6d3c60a32f9d9abe5c15b60165d5  e203-admission-scan-cost-timing-files_1-2026-11-07.out
e250e55f301882052bac48fee35ecaa8bc28860642ba545a43e3602e4e049e43  e203-admission-scan-cost-timing-unit_area_2_pow_14-2026-11-07.out
eed47313f9aaf17a627f0a28182c35cfd23e21943e10166d44019540504b7a72  e203-admission-scan-cost-timing-unit_area_2_pow_20-2026-11-07.out
1b347fe777fa58359a8b0abf420e25b84c407eedbe6e27758ed40683ef8dde5e  e203-admission-scan-cost-timing-unit_area_2_pow_24-2026-11-07.out
70c910cd4159eac1277027f90b62acb5c89188114670fcb55da504b421bd69e6  e203-admission-scan-cost-timing-unlinks_10000-2026-11-07.out
4bcf3d60740963896ecac8d798a2565d9f6a13c1a1d157f4dd2c6701fa75b766  e203-admission-scan-cost-timing-unlinks_1000-2026-11-07.out
a40340b4fb5871a777a3f979378408e429c45cbe8ebf2380f401d64a0ca95719  e203-admission-scan-cost-timing-unlinks_100-2026-11-07.out
3404fb15b2f3bc36e7b33828a3673d9733abde62241d311c26249acb6b026dbc  e203-admission-scan-cost-timing-unlinks_1-2026-11-07.out
025f498e2ee0518724a32bd6d0c41b3e5d2946ec30e30681558be0ffc3c53216  e203-admission-scan-cost-timing-user_scale_unit_area-2026-11-07-r2.out
```

### 我的脚本 sha256（目录 `/home/fy5090/.local/share/singlefs/work/e203-forks-r2-sonnet/`，路径相对它）

```
cca52b289c67e015833f94904818352f0d14fe4e5702378f726ff843cd339750  tools/b8port.py
7536389d775b70ef6dfe7a559cd302ebe399852922fd2b6aa318fc2973cb12e4  tools/copies.py
76f8398e7fb064d012542afe13de8673c9405e92861b139ddebf05f2d5eddee1  tools/drop_vs_clone.rs
69b781bfab02d15205968058ff613806503125249b3e6f1490c9aa0b6f9b4b98  tools/drop-vs-clone.sh
6e048a03491ff94ec72676e5b2a74c8fb730f3db0dbcd0511d750241503e75f9  tools/hist.py
8214ce1a31e57189bb9fe942c27910e2dfe41eeb3be4dd9eff6062fad9c6f1ae  tools/mono2.py
e4f0583b09aa10ab8826454895b396172829f3cac1b519f4d54af257be49e888  tools/mono.py
1a2caa3753d56f05083375da2f8a0ed4baf03d5ae208d387afb7cfb94fe43c0d  tools/noncopy.py
cf5fc41ad36da3d68ac38d1d030ba089fd48d8a89dbd1ee3f373f0ccad0464b4  tools/q10.py
e9ea12de1d5002d62b73b2378aace2cd0297f3eeddd5e31661b86e70a6b8c9af  tools/q6all.py
261d2ec9622a618c918260c2ef07c6b4475cdc57096e05268b518d3408a1128d  tools/rerun.sh
3be7d2a6e618870de4dc3b8f66ccaa993863d1c7d837b014b088be11796190cf  tools/residual_vs_copy.py
a810cca746a57e140f899c961f17ce4a7ba4933bfc11addd438482f6d0a91c74  tools/rounds_detail.py
3eab39a47abe6d56984841cf748eeffef382cb924ea5f7cc11dc702463710421  tools/rounds.py
6f9be9fd8d8d965f614d6d821d28c3b4af1d1789fbcb1ee4c2dc4c102f9f5d37  tools/rows_diff.py
3df73bafb82965181d09dafd380793ef1eb46ddb877d605909d741a4c69b4005  tools/s1_residual.py
7d27232ce0c176a34c8bdcc081ffd414cce37ef08bd33fb2321d12fc261ab8da  tools/s1_rows.py
a662442014b3bd0659ca9d23deccd1cc7f2841afae910adb719a2d8bcce056c2  tools/s36_half.py
c2ad01cb73bb61e3158d9d3e1bcdf34a8e2f72c20c72692b7919a2da2826687c  tools/state_axis.py
ccbe31876ac34690e050a7918df4c548ada2ba00e45b2fcfeb96630e87cbdc2f  tools/x2.py
eba73ab42bb8dc9c85994b3dec329a96bf18189df9d34f20bec6da53baa9d3cc  tools/x2range.py
```

## 一、总表：第一轮判决每一条打中，辩方的判定

判定三档：站得住 / 够不着（它引的东西不足以推出那句结论，或数已变）/ 同样打中别的臂。判决文件简称「第一轮判决」= `research/prompts/e203-forks-r1-main-verification.md`。

| 判决条目（原句与行号） | 辩方判定 | 证据（细节在后面各节） |
|---|---|---|
| 「E203 的 F1 照登记记 unknown，不改」（`research/prompts/e203-forks-r1-main-verification.md:17`），即 B1 不能拿 X2 直接读数翻面 | 站得住 | 登记 F6 行与修订 3 白纸黑字（`research/prompts/e203-preregistration.md:618`、`:680`）；X2 是 (2³⁰−1408, 10, 10, 10)，与 Q4 的格 (2³⁰−1408, 10⁶, D_user, 1000) 只有 U 一根轴对得上。修订 7 之后的日志里 F1 仍是 unknown（日志 3077 行一带，见第二节）。这一判不改变 B1 的命：X2 整份拷一行下界仍在（第二节） |
| 「X2 上整份拷分配器一行 1.32–1.42 秒（量的）」（`research/prompts/e203-forks-r1-main-verification.md:17`） | 够不着（数已过时） | 修订 7 的产物里这一行在 X2 上是 1.390–1.719 秒（30 个读数：两种挂载 × 三种发布 × 5 轮），覆盖写的 10 个读数 1.390–1.688 秒；旧范围出自已删的上一批产物。下界结论（远大于 1 毫秒）不变，写给用户的数要换 |
| 「下界只能用整份拷一行，不能用扫类合计」（`research/prompts/e203-forks-r1-main-verification.md:18`） | 站得住，且在新产物上复现 | 新产物上 B1 扫类合计沿轴仍有下降的格（`tools/mono2.py`：u、D、F 三轴共 11 处相邻两点下降，6 处是后一点固定点轮数少一轮、整份拷少一份，另 5 处在同轮数下降 0.1%–1.1%），第二节 |
| 同一判对 B4：「两道之间那一段 X2 上每次覆盖写约 2.78 秒（量的），其中 99.999% 是整份拷两次（正推腿，推的）」（`research/prompts/e203-forks-r1-main-verification.md:33`） | 同样打中 B4 | 这个数同样是 X2 点、同样不是用户规模的格；要当下界得多一条前提（固定点至少 2 轮）；新产物 2.960–4.476 秒，「99.999%」是步数占比，时间占比是整份拷独占 86% 加未归行的残差 14%（第二节） |
| 同一判对 B8 | 够不着 | B8 的数全是按式子从计数推的，不吃 X2 的挂钟；X2 只在 `q6_top_rows` 里当 U 最大的点出现（第二节） |
| 「F2 打中的主要是登记式子的错」（`research/prompts/e203-forks-r1-main-verification.md:19`），即主要是 s36 | 站得住（对上一批产物）；修订 7 之后这一句不再描述剩下的那一格 | 上一批产物已删，我复核不到那 97.5–99.3%；修订 7 之后 s36 在剩下那一格的涨里只占 23.8%（第三节表） |
| 「扣掉后 F 轴同一次挂载覆盖写仍 α=0.583（固定点 22 轮的乘数），这一格 B8 仍输」（`research/prompts/e203-forks-r1-main-verification.md:19`） | 方向站得住，说法过粗 | 新产物同一格 α=0.620064（日志 2216 行）；涨的不是单乘数：固定点轮数 2→22，每轮重写角色数还从 11 涨到 84，轮上累计角色 21→1201（57 倍）；这是 1 个发布（540 个里的 1 个）；第三节 |
| 「B4 只能当改条款交」（`research/prompts/e203-forks-r1-main-verification.md:22`） | 站得住（覆盖写类）；对删除类发布，B4 有一条字面站得住的读法 | 第四节：D28 的接线与删除类发布不判式子（冻结源树 12891 行）合起来，删除类发布里唯一的准入就是落点那一判 |
| 「删除类发布不判式子判，登记读法 1 把那一段算进「准入之前」是多算」（`research/prompts/e203-forks-r1-main-verification.md:22`） | 站得住，且应当更用力 | 同上；这句是 B4 唯一不靠改条款的立足点 |
| 「修订 s36 之后只剩 F 轴同一次挂载覆盖写一格涨（固定点轮数乘数，推的）」（`research/prompts/e203-forks-r1-main-verification.md:32`） | 站得住 | 新日志 D、F 两轴 12 格里只有这一格 GrowsWithTheAxis；但 u 轴同挂载覆盖写 α=0.261401 落在「介于」带（日志 2210 行），同一个机制在更小的幅度上也有；第三节 |
| 同一行「收严形态（固定点轮数有上界）是新候选，零轮」 | 够不着 | 源码里已经有上界 64（冻结源树 13436 行），产品注释写「产品路径上三四轮就定」（13433 行），实测一个发布走了 22 轮；所以问题不是「有没有上界」，是 22 轮这件事本身；第五节 |

## 二、B1 只作下界：替 B1 辩，这一判是不是同样打中 B4、B8 的下界用法

### 2.1 判决原句与登记行

- 判决：「E203 的 F1 照登记记 unknown，不改」（`research/prompts/e203-forks-r1-main-verification.md:17`）；「下界只能用整份拷一行，不能用扫类合计」（`research/prompts/e203-forks-r1-main-verification.md:18`）。
- 登记：「X2 量过之后 Q4 在 U 那一部分改用 X2 的量，记录数那一部分仍外推」（`research/prompts/e203-preregistration.md:618`）；「X2 量过之后照严的读法 Q4 不判」（`research/prompts/e203-preregistration.md:680`）。
- 新日志：`name=verdict clause=F1 fired=unknown`（`task-b0f6b921-10180.attempt-1.log:3077`）。

### 2.2 替 B1 辩：能辩出什么

B1 最强的立论只有两条，都过不了：

1. 「Q4 是上界不是下界，X2 量的是 U 一根轴，记录轴没量，所以不能说 B1 慢」。反过来才是事实：F1 要的是「四格的 T 都 ≥ 2 毫秒」，T 是各行 步数 × 每步纳秒 之和，各项非负，所以 T ≥ 任一行单独的值。整份拷一行在 X2 上是 1.390–1.719 秒（30 个读数，`tools/x2range.py`；样例行：「segment=caller row=whole_allocator_copy_before_the_judgement calls=0 steps=0 scale=0 steps_to_the_hit=0 scale_to_the_hit=0 exclusive_nanos=1417588240」，`report-inputs/e203-admission-scan-cost-timing-user_scale_unit_area-2026-11-07-r2.out:115`），比 2 毫秒高三个数量级。要把这个读数搬到用户规模的格，前提只有一条：这一行的大小只由 U 定。冻结源树里每张表建图时一次定长（「allocated: vec![false; usize::try_from(unit_area_slots)」，`pristine-source/crates/singlefs-core/src/allocator.rs:469`），整份拷是 `#[derive(Clone, Debug)]` 的 `DeviceFreeMap`（「pub struct DeviceFreeMap {」，`pristine-source/crates/singlefs-core/src/allocator.rs:304`）；调用方那一份是「let allocator_before_this_publish = allocator.clone();」（`pristine-source/crates/singlefs-core/src/transaction.rs:12609`），每次发布恰好一份（540 个发布读数里 `whole_allocator_copy_before_the_judgement` 的步数÷`whole_copy_elements` 恒为 1，`tools/copies.py`）。所以第一轮判决的「只作下界」在 s1 这一行上成立，且不需要「步数随记录数不减」。
2. 「F6 打中，Q4 不判，所以 B1 不能被翻面」。这是登记字面（618、680 两行），站得住；但它只管 E203 的 F1 记什么，不改 B1 在岔路表上的下界读数。B1 要留下，只剩一条路：改条款把整份拷分配器列为例外（第一轮判决第三节「留 B1 并在条款里明示整份拷分配器是例外」，`research/prompts/e203-forks-r1-main-verification.md:34`）。这条路有先例形状：岔路单自己把照抄字节的代价划给 E199（「照抄字节的代价归 E199（发布照抄上一版的代价）」，`research/prompts/e203-preregistration.md:37`）；但同一行把「整份拷分配器（调用方 `:12314` 与固定点每轮）」列在扫类里（`research/prompts/e203-preregistration.md:37`），所以例外是改条款，不是读法。

判：第一轮判决对 B1 的这一判站得住；数 1.32–1.42 秒要换成新产物的 1.390–1.719 秒（覆盖写 1.390–1.688 秒）。

### 2.3 同样打中 B4

第一轮判决交用户表里 B4 一行的数：「两道之间那一段 X2 上每次覆盖写约 2.78 秒（量的），其中 99.999% 是整份拷两次（正推腿，推的）」（`research/prompts/e203-forks-r1-main-verification.md:33`）。同一个毛病：

- 这个点是 X2，u、D、F 全是 10，不是 Q4 的格；它是 B4 保留下来的那一段在 U=2³⁰−1408 上的值。当用户规模的下界可以，当「B4 每次覆盖写约 2.78 秒」的点值不行。
- 要当下界，比 s1 多一条前提：固定点轮数 ≥ 2。查到的：540 个发布读数里 `fixed_point_rounds` 最小 2（2 轮 487 个，3 轮 38，4 轮 10，5 轮 3，6 轮 1，22 轮 1；`tools/hist.py`），整份拷份数等于轮数（比值恒 1.0，`tools/copies.py`）。源码给了一半的结构理由：循环「for _round in 0..ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT {」（`pristine-source/crates/singlefs-core/src/transaction.rs:13348`），首轮重写集是空的（「第一轮重写集是空的」，`pristine-source/crates/singlefs-core/src/transaction.rs:13433`），发布改了记录就必须再来一轮收敛；每轮一份拷贝「let mut rehearsal = allocator.clone();」（`pristine-source/crates/singlefs-core/src/transaction.rs:13393`）。所以覆盖写类的下界是 2 份，这一条没有被一个反例推翻，但也没有被证出来（推的）。
- 新产物上这个数是 2.960–4.476 秒（覆盖写 10 个读数，`tools/x2.py`；日志 q9 行样例：「name=q9 point=user_scale_unit_area state=same_mount kind=overwrite steps_between=13220563557」，`task-b0f6b921-10180.attempt-1.log:2346`），不是 2.78 秒。「99.999%」是步数占比（整份拷元素 6610215990 × 2 轮 = 13220431980，占 steps_between 13220563557 的 99.999%）；挂钟占比不是：整份拷独占 2.546 秒、段挂钟 2.960 秒（同一次读数，`report-inputs/e203-admission-scan-cost-timing-user_scale_unit_area-2026-11-07-r2.out:148`、`:165`），86% 对 14%，余下 14% 是不归任何一行的残差（第四节讲它是什么）。

判：同样打中 B4。B4 的 X2 读数同样只是下界；写给用户时「约 2.78 秒」要换成「X2 上 2.960–4.476 秒，用户规模的格不低于两份整份拷」，并标 2 轮下界是推的。

### 2.4 够不着 B8

B8 的数是模型按式子从计数推的，不吃 X2 的挂钟。X2 在模型里只出现在 `q6_top_rows`（挑 U 最大的点，第一轮判决第二节第 2 格已指出）。B8 的短板是另一个方向：它没有任何用户规模的上界读数——这不是第一轮这一判打中的，是整个方案的状态，不拿来当这一判的后果。

### 2.5 方向对称这一点，写给交用户表

X2 的读数只能证明「不低于」。它能拒 B1（整份拷一行就 ≥ 1.39 秒）、能说 B4「不低于两份整份拷」，不能放行任何一条臂。第一轮判决没有把它用在放行上，这一判没有偏袒谁。

什么现象会推翻这一节：(1) 在 U=2³⁰−1408 上量到整份拷一行 < 1 毫秒；(2) 冻结源树里 `DeviceFreeMap` 的表在挂载后改了长度；(3) 一个覆盖写发布在固定点上只走 1 轮（540 个里 0 个）。

## 三、B8 剩下的一格：F 轴、同一次挂载、覆盖写、α=0.620064

### 3.1 逐行：涨来自哪几行（读产物算）

算法见第零节：`tools/b8port.py` 读 `counts-base`（F=10）与 `counts-files_10000`（F=10⁴）两份计数产物，同一次挂载、覆盖写、第 1 轮，caller、before_landing、between 三段逐行相加；`tools/rows_diff.py` 打下表。两端 2102 与 152344，α=ln(152344/2102)/ln(1000)=0.620064，与日志逐位相同（「name=q6 axis=F state=same_mount kind=overwrite low_end=2102 high_end=152344 alpha=0.620064 band=GrowsWithTheAxis」，`task-b0f6b921-10180.attempt-1.log:2216`）。三段里只有 between 动：0→0、36→36、2066→152308。

| B8 行（模型行名） | F=10 | F=10⁴ | 增量 | 占总增量 150242 | 这一行吃的 B1 量的输入 |
|---|---|---|---|---|---|
| allocator_release_linear_find | 840 | 76864 | 76024 | 50.6% | 调用次数 84→4804 × lg(分配记录数)（10→16） |
| mapped_units_whole_table | 294 | 36030 | 35736 | 23.8% | 轮上进出角色 42→2402 × lg(M)（6→14） |
| release_judgement_version_indexes | 114 | 15890 | 15776 | 10.5% | 释放判定那一圈的查询次数 × 索引层数 |
| records_of_each_leaf | 170 | 5460 | 5290 | 3.5% | 每次调用前后不同的记录数之和 |
| multi_level_tree_planning | 94 | 5024 | 4930 | 3.3% | 三棵码 2 树每轮进出 key 数 × 树高 |
| allocation_record_tree_plan | 24 | 2898 | 2874 | 1.9% | 每轮内容变了的叶数之和 8→966 × 层数 3 |
| allocation_record_tree_node_set | 24 | 2898 | 2874 | 1.9% | 同上 |
| record_intersection_filter | 282 | 2840 | 2558 | 1.7% | 调用次数 × lg(记录数) + 相交记录数 |
| whole_allocator_copy_in_a_settling_round | 42 | 2402 | 2360 | 1.6% | 每轮释放数 + 重写角色数之和 42→2402 |
| deadlist_changes_indexes | 48 | 1540 | 1492 | 1.0% | 同 s19 的查询 |
| accounting_entries_of_this_publish | 30 | 330 | 300 | 0.2% | 调用次数 2→22 × 记账行 15 |
| 其余各行合计（含 user_data_slot_linear_scan 68、checkpoint_cost_accounting_tree_plan 60 两个没动的行） | 140 | 168 | 28 | 0.02% | inode_number_watermark_steps 与 empty_segment_linear_scan 微增 |

规模与轮数（同一对点，`tools/rounds_detail.py`、`tools/state_axis.py`）：固定点轮数 2→22；每轮重写的角色数 F=10 时 7、14，F=10⁴ 时 11、24、28、32、…、82、84（逐轮约加 2–4），累计 21→1201（57 倍）；每轮内容变了的叶 4、4 → 10、14、18、…、70；分配记录数 922→47624；`mapped_units` 35→10221；聚簇段数 9→230。

### 3.2 是固定点轮数的乘数吗

不止。轮数只涨 11 倍（2→22），B8 的步数涨 72.5 倍。多出来的是「每轮重写的角色数也在逐轮上涨」：累计角色 57 倍，再乘 lg 项（lg(分配记录数) 10→16，lg(M) 6→14）。第一轮判决写的「固定点 22 轮的乘数」（`research/prompts/e203-forks-r1-main-verification.md:19`）方向对，量级说成线性乘数偏小。

另有两处算法印证它不是 s36 造的：把模型里 s36 整行拿掉，α 仍 0.602811（总数 1808→116314）；把 s36 的查询次数从「条目数 2×」改成「角色数 1×」，α 仍 0.612083（`tools/s36_half.py`）。第一轮判决说主要归 s36（`research/prompts/e203-forks-r1-main-verification.md:19`），那是对上一批产物说的，修订 7 之后 s36 只占 23.8%，最大的是 s25 一行的 50.6%（它的调用次数是 B1 量的，修订 7 之后两处查找都记）。

### 3.3 局部斜率：0.620 是一条割线，不是幂律

同一格 F=10、100、1000、10⁴ 四点的 B8 步数是 2102、2331、6961、152344，相邻两点的局部 α 是 0.045、0.475、1.340（`tools/s36_half.py`）。固定点轮数与聚簇段数同向：轮数 2、2、3、22，聚簇段数 9、9、28、230（`tools/state_axis.py`；例：「point=files_10000 state=same_mount kind=overwrite round=1 units=10291 mapped_units=10221 allocation_records=47624 cluster_segments=230」，`report-inputs/e203-admission-scan-cost-counts-files_10000-2026-11-07.out:1040`）。相关，不是已证的因果（推的）。含义：割线 0.620 低估最后一个数量级的斜率（1.34）；数据不支持「B8 在 F 上慢涨」，也不支持「0.620 就是它的指数」。

### 3.4 这一格是不是装置造的：三个分开的问题

1. **B8 的定义含不含这一格——含。** 登记写死 B8 不改调用结构（「B8 不改调用结构，调用次数取 B1 量的」，`research/prompts/e203-preregistration.md:215`；「调用结构（几轮、调几次）不变」，`research/prompts/e203-preregistration.md:247`）。22 轮和 4804 次释放查找是 B1 在这个发布上量到的，B8 按定义继承。模型式子没造这一格：s36 换成两种别的读法，这一格都还在。
2. **装置造了什么——三样，都不是式子。** (a) 只读第 1 轮：模型写「同上，取第 1 轮（计数是确定的）。」（`scripts/experiment-apparatus/e203_admission_scan_cost.rs:1007`），B8 输入函数也固定取第 1 轮（`scripts/experiment-apparatus/e203_admission_scan_cost.rs:2360`），前提是登记写的「计数类是确定性模型，只跑一轮」（`research/prompts/e203-preregistration.md:322`）。产物把这个前提推翻了：同一格五轮的固定点轮数是 22、2、2、2、2（第 1 轮「point=files_10000 state=same_mount kind=overwrite round=1 had_caller_segment=true fixed_point_rounds=22」，`report-inputs/e203-admission-scan-cost-counts-files_10000-2026-11-07.out:1039`；第 2 轮「point=files_10000 state=same_mount kind=overwrite round=2 had_caller_segment=true fixed_point_rounds=2」，同文件 `:1254`），第 2–5 轮 B8 步数 4226、3882、3874、3826，对 F=10 的 2098、2098、2098、2254，α 是 0.1014、0.0891、0.0888、0.0766（`tools/s36_half.py`）。(b) 一个端点：F=10⁴ 是这根轴的最后一点，没有 10⁵（第 3.3 节）。(c) 历史：这一个发布是「同一次挂载里攒了一整段历史」那一种的第一次量（登记历史第 6 条写了它与重挂载「方向相反」，`research/prompts/e203-preregistration.md:238`）；同一历史重挂载之后同一个发布只走 2 轮、α=0.101423（「name=q6 axis=F state=remounted kind=overwrite low_end=2270 high_end=4574 alpha=0.101423 band=DoesNotGrow」，`task-b0f6b921-10180.attempt-1.log:2255`）。
3. **B8 真输吗——在这一个发布上是。** 540 个发布读数里只有这 1 个走 22 轮（`tools/hist.py`），第 1 轮读数共 108 个里也只有它；但 F2 的字面是任一轴任一状态任一发布（「Q6 在任一根轴上（任一种挂载状态、任一种被量的发布）记「B8 仍随总数涨」，α ≥ 0.5」，`research/prompts/e203-preregistration.md:614`）。把读法改成「五轮都要 ≥ 0.5 才算」是放宽，按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「臂的定义也在「跑前写死」之列」不能悄悄做；改成「五轮里任一轮」与现在同结果。所以辩方拿不掉这一格的命中，只能说：它是一个发布的命中，不是 B8 式子的性质。

### 3.5 同机制在别处，和 B1 对照

- u 轴同挂载覆盖写也有同一个机制：第 1 轮 6 轮固定点（`report-inputs/e203-admission-scan-cost-counts-unlinks_10000-2026-11-07.out:505`），α=0.261401，落在「介于」带（「name=q6 axis=u state=same_mount kind=overwrite low_end=2102 high_end=12789 alpha=0.261401 band=Between」，`task-b0f6b921-10180.attempt-1.log:2210`）。登记 Q6 的「全部轴 ≤ 0.25 记」才是「B8 不随总数涨」（`research/prompts/e203-preregistration.md:331`），所以即使这一格不算，Q6 也只能是「介于」，不是「不随」。
- 同一格里 B1 扫类步数（不含两个整份拷）是 142792997（`tools/noncopy.py`），B8 是 152344，B8 低三个数量级；它输在绝对条款上，赢在相对上。

什么现象会推翻第 3.4 节第 2 点：把第 2–5 轮的 B8 步数也算进去的 α 带仍 ≥ 0.5（我算的是 0.077–0.101）；或者第 1 轮的 22 轮在同一历史重放一遍不再出现（那就是偶发，不是 B1 在这一格的行为）。

## 四、B4：「两道之间那一段」读出运行时决策路径，有没有站得住的读法

### 4.1 条款原文与冻结源树行号

- 条款（D8（核心索引结构） 已定项 16 射程）：「准入之前的步代价不随盘上容器总数、记录总数涨；准入之后装单元、拷字节不算运行时决策路径」（`.claude/kb/decisions/08-核心索引结构.md:516`）。它读的是 `.claude/rules/fs-design.md` 那一格：「代价不许随盘容量增长」（`.claude/rules/fs-design.md:23`），该格的线「线划在「谁消费这个数」，不划在「用什么机制算出来」」（`.claude/rules/fs-design.md:19`）。
- 用户当时的原话只有两句，没有定义「准入」是哪一道：「管，只管准入之前」「五处加准入现在改，根源先量」（`.claude/kb/decisions-history.md:3545`）；条款读法是用户在同一条款上定过一次的（同一条目标题：「已定项 16 射程补代价条款的读法」，`.claude/kb/decisions-history.md:3543`）。所以「改射程」是这条款有过的改法，B4 是同一种形状的第二次改。
- D28（挂载期承诺量） 已定项 1 的接线：「算定这次发布的形状之后、读盘核与动分配器之前」判（`.claude/kb/decisions/28-挂载期承诺量.md:35`）；删除类发布「删除类发布的落点那一判照判」（`.claude/kb/decisions/28-挂载期承诺量.md:38`）；条款自己把两种拒分开叫：「准入拒或落点被拒由会话照」（`.claude/kb/decisions/28-挂载期承诺量.md:40`）。
- 冻结源树 `pristine-source/crates/singlefs-core/src/transaction.rs`，`judge_the_publish_before_any_read_or_write` 里的次序：先是落点那一判（注释「准入先判这次的单元落不落得下」，`:12801`；注释里的需求口径「普通分配那一半照计划数，固定点按 ckpt_cost 的最坏数」，`:12803`；调用「admit_the_units_landing_on_every_device(」，`:12818`）；然后是固定点「let settled = settle_the_allocation_record_tree(」（`:12831`）；然后式子那一判的需求取自固定点的结果「let demand = demand_of_the_roles_on_each_device(」（`:12887`），删除类发布不判它（「删除类发布（意图、各批、最后一批）不判式子那一判、落点那一判照判」，`:12891`；判不判的条件「!step.is_a_deletion_class_publish()」，`:12900`），式子那一判的调用「admit_on_every_device(」（`:12916`）。今天的 `crates/singlefs-core/src/transaction.rs` 里同一串语句晚 30 行（落点判 12848、固定点 12861、需求 12917、删除类条件 12930）。

### 4.2 三种读法，各自有没有条款字面

| 读法 | 「准入」指 | 「两道之间」在不在射程内 | 条款字面 |
|---|---|---|---|
| 登记读法 1（`research/prompts/e203-preregistration.md:21`「两道准入都算」） | 一次发布里到式子那一判为止的两道 | 在 | 与 D28 已定项 1 接线一致：式子那一判是「算定形状之后」判的，形状算定本身在它之前（`.claude/kb/decisions/28-挂载期承诺量.md:35`）；与 fs-design 的线一致：固定点的结果的消费者是式子那一判的需求（`:12887`），落在「ENOSPC 准入」那一行 |
| B4 的定义（`research/prompts/e203-preregistration.md:43`「把两道准入之间那一段（连同准入之后的真取落点）读成不在射程内」） | 第一道（落点判） | 不在 | 没有字面：kb 里「准入」与「落点」是分开叫的（`.claude/kb/decisions/28-挂载期承诺量.md:40`）；只有源码注释把落点判叫「准入先判」（`:12801`），注释不是条款。fs-design 的线（`.claude/rules/fs-design.md:19`）反着指：固定点的结果喂的是准入的需求。所以这一读法是改条款，第一轮判决「B4 只能当改条款交」（`research/prompts/e203-forks-r1-main-verification.md:22`）站得住，且与岔路单把它叫「改射程」同形，不是打败它 |
| 按发布种类分的读法（本腿提出，零轮） | 这一次发布里实际判的那一道 | 写入类（覆盖写）：在；删除类（unlink 的意图与最后一批）：不在 | 有字面：删除类发布不判式子（`:12891`、`:12900`），唯一的准入就是落点判（`.claude/kb/decisions/28-挂载期承诺量.md:38`），固定点在它之后（`:12831`）；「准入之后装单元、拷字节不算」（`.claude/kb/decisions/08-核心索引结构.md:516`）又盖住固定点里的整份拷贝（步数占比 99.999%，第二节）。第一轮判决那句「登记读法 1 把那一段算进「准入之前」是多算」（`research/prompts/e203-forks-r1-main-verification.md:22`）说的就是这一格 |

按发布种类分的读法只覆盖删除类，覆盖写那一半仍要改条款。它也不是干净的：固定点里不是拷字节的那些扫类行在删除类发布里同样在落点判之后，条款第二半只豁免了「装单元、拷字节」，对别的步沉默。

### 4.3 B4 在不改条款下成立的另一条路：改代码次序，不改条款

落点判已经在用「固定点按 ckpt_cost 的最坏数」（`pristine-source/crates/singlefs-core/src/transaction.rs:12803`），也就是说不等固定点算完就能判一道准入。式子那一判如果也吃最坏数（第一轮判决新候选之一「式子判吃最坏数」，`research/prompts/e203-forks-r1-main-verification.md:34`），固定点就整段挪到所有准入之后，落在条款第二半「准入之后装单元、拷字节不算」之内，B4 就不用改条款。代价是多拒：最坏数比固定点算出的数大多少，这一轮产物里没有（第五节）。

### 4.4 F3 打中，它分不分辨 B4 与别的臂

F3 的字面是「Q10：某一段的残差在某根轴上 5 轮都 α ≥ 0.5」（`research/prompts/e203-preregistration.md:615`）；日志 `name=verdict clause=F3 fired=true`（`task-b0f6b921-10180.attempt-1.log:3079`）。我按段拆（`tools/q10.py`，与日志 q10 行逐位相同，样例「name=q10 axis=u state=same_mount kind=overwrite segment=before_landing alphas=0.787311,0.757060,0.771941,0.790239,0.789375 band=GrowsWithTheAxis」，`task-b0f6b921-10180.attempt-1.log:2863`）：

| 段 | 5 轮都 ≥ 0.5 的轴 | 是哪些臂留着这块残差 |
|---|---|---|
| caller（P0，只有整份拷） | 没有（24 个格里没有一格 5 轮都 ≥ 0.5，单轮最大 0.341） | — |
| before_landing（S1） | u 轴（六格里五格 5 轮都 ≥ 0.5，范围 0.59–0.88）；F 轴 unlink 最后一批（两种挂载，0.52–0.66） | B1、B8、B4 都留着：这一块没挂在任何一个被替换的行上，三臂一样 |
| between（S2） | U 轴重挂载（三种发布 0.67–0.77）；D、F、u 轴多格 | 分两半：挂在整份拷上的（下面）B8 去掉、B4 与 B1 留着；没挂的三臂一样 |

- S1 的残差是 B4 最疼的一块，因为 B4 保留的只有 P0 加 S1。同挂载覆盖写：u=10⁴ 时 S1 段挂钟 6513.6 微秒，各行独占之和 5001.2 微秒，残差 1560.2 微秒（24%）；F=10⁴ 时 1477.2、1307.5、169.7 微秒（`tools/s1_residual.py`）。独占里最大的是 `checkpoint_cost_accounting_tree_plan`（u=10⁴：4909.9 微秒；`tools/s1_rows.py`），那是 B4 要靠式子替换的行，残差不在它里面。若 α≈0.77 延到 u=10⁶，残差 ≈ 1.56 毫秒 × 100^0.77 ≈ 54 毫秒（推的，外推两个数量级，登记自己的 Q5 对这类外推核不过）。结论：F3 在 S1 上打中三臂一样，不分辨 B4 与 B1、B8，但它打的正是 B4 唯一的卖点（落点判之前干净）。
- S2 的 U 轴残差跟着整份拷走：覆盖写、between 段，残差 ÷ 整份拷独占在 X2 同挂载是 0.162、0.164、0.178、0.181、0.169，重挂载 0.165、0.164、0.166、0.190、0.184；U=2²⁰ 时 0.156–0.167；U=2²⁴ 同挂载 0.069–0.072、重挂载 0.131–0.185（`tools/residual_vs_copy.py`）；U≤2¹⁷ 时残差 39–79 微秒，在 2¹⁴ 到 2¹⁷ 之间几乎不动，而整份拷独占从 7–13 微秒涨到 31–71 微秒（重挂载第 1 轮一个 337 微秒的读数除外），两者不成比例。最像的解释是每轮那份拷贝出来的分配器在轮末放手（drop）时的释放代价没算进「整份拷」那一行的守卫（推的，没有直接量过）。若是，F3 在这一半上就不是「清单不全」，是拷贝那一行的守卫罩得不够：B8 把拷贝换成撤回日志，这块残差跟着没了，B4 的 S2 与 B1 留着。登记 F3 的后果写「B8、B4 两行的结论标「只对清单内的步」」（`research/prompts/e203-preregistration.md:615`）对这一半不对，它分辨 B8 与 B4。
- 四句：它分辨臂吗（S1 上不分辨，S2 的 U 轴上分辨 B8 与 B4/B1）；系统当时看得到吗（不涉及判别子）；满足的是哪个分句（字面「某一段的残差在某根轴上 5 轮都 α ≥ 0.5」，满足）；条款给的改法碰得到吗（登记 F3 的改法是补清单、重登记；对 S2 这一半补清单碰不到，要改的是守卫范围）。

另一个相关的读数，F5：日志 `name=q8 arm=b4 axis=F state=same_mount kind=overwrite low_end=36 high_end=36 alpha=0.000000 band=DoesNotGrow`（`task-b0f6b921-10180.attempt-1.log:2217`）。B4 臂的 P0 加 S1 在 F 轴两端都是 36 步，因为 B8 的式子在这些行上不吃 F，而把 F 轴上的涨（全在 between 段，152308 步）按定义排除在 B4 的射程外。所以 F5 没打中是定义的后果：它能区分 B4 与 B1（B1 的 P0 加 S1 在 u 轴 808523→892325，`task-b0f6b921-10180.attempt-1.log:2212`，被整份拷占满），区分不了 B4 与 B8 在 S1 上的数。

什么现象会推翻第 4.2 节按发布种类分的读法：D28 已定项 1 或实现里出现删除类发布也判式子那一判；或删除类发布的落点判被取消（那就没有「唯一的准入」）。什么现象会推翻第 4.4 节「S2 残差跟着拷贝」：整份拷守卫改罩到轮末放手之后，残差仍不随之消失（第五节的命令量放手那一半）。

## 五、交用户表里的零轮候选：这一轮的数够不够判

候选名单照第一轮判决第三节新候选一行（「只把整份拷分配器换成撤回日志」「分块共享拷贝」「B8 收严形态」「留 B1 并在条款里明示整份拷分配器是例外」「式子判吃最坏数」，`research/prompts/e203-forks-r1-main-verification.md:34`）。「够判」指：用户要的代价数这一轮产物里已有；「仍缺」指要一次新登记的量。以下读数除注明的外都是我算的，待主 agent 复跑（第六节）。

| 候选 | 这一轮已有的数 | 仍缺的数 | 判 |
|---|---|---|---|
| 留 B1 并在条款里明示整份拷分配器是例外 | X2 上一次覆盖写 P0 加 S1 加 S2 的挂钟 4.355–5.878 秒（`tools/x2range.py`）；整份拷一行 1.390–1.719 秒（第二节）；覆盖写至少 3 份（调用方 1 份加固定点每轮 1 份，`tools/copies.py`），X2 重挂载第 1 轮走了 3 轮（4 份）；一个发布走 22 轮的话份数按轮数涨（U=2¹⁷ 上量到，X2 尺寸上没量，推的：约 23 份 × 1.3 秒） | 例外只写整份拷时，记录那一半还在：B1 扫类步数（不含两个整份拷）在 u、D、F 轴的 α 是 0.445–1.086（`tools/noncopy.py`）；S1 段挂钟 u=10⁴ 时 6.5 毫秒（`tools/s1_residual.py`）。要不要把这些也列例外，是条款问题，数已够 | 够判（代价数已有；用户要定的是例外写多宽） |
| 只把整份拷分配器换成撤回日志，条款不动 | 省下的：X2 上 P0 的 1.390–1.719 秒、S2 的整份拷 2.54–3.84 秒；X2 上扣掉两个整份拷之后，各行独占加起来约 0.45 毫秒（between 段 0.40、S1 段 0.06），另有不归行的残差 S1 段 0.04 毫秒、between 段 0.41 秒（`tools/x2.py`、`tools/residual_vs_copy.py`）；条款的容量那一半（`.claude/rules/fs-design.md:23`）因此能过：U 轴 B8 步数 α 0.02–0.11（`tools/q6all.py`） | 条款的记录那一半不动就过不了：同上 B1 扫类（不含两个整份拷）α 0.445–1.086。撤回日志自己的写入与回倒代价这一轮没有登记量（B8 的 Q7 式子给了推的数，日志里有，我没复核）。另有未归行的 0.41–0.63 秒残差是否随拷贝走，决定 X2 上换完之后是 < 1 毫秒还是 0.4 秒量级（第四节，命令见第六节） | 「条款不动」这一半够判：不过。收益那一半仍缺一个量（残差随不随拷贝） |
| 分块共享拷贝 | 每个发布改的槽位数与记录数（`round_placements_released` 加 `round_roles_rewritten` 之和：F=10 时 42，22 轮点 2402）；整份拷元素数 | 被改的槽位落在几个块里：产物只有个数，没有位置，换任何块大小都算不出共享后要拷几块；块索引的常驻内存；同样过不了记录那一半 | 仍缺 |
| B8 收严（固定点轮数有上界） | 源码已经有上界 64（`pristine-source/crates/singlefs-core/src/transaction.rs:13436`），注释写三四轮就定（`:13433`）；540 个发布里 2 轮 487、3 轮 38、4 轮 10、5 轮 3、6 轮 1、22 轮 1；那一格第 2–5 轮 B8 步数 3826–4226，α 0.077–0.101 | 22 轮是怎么来的（产物里只有逐轮重写角色数 11、24、28、…、84，没有原因）；上界取多少会把真实发布拒掉：上界 4 拒掉 ≥ 5 轮的 5 个读数（5 轮 3 个、6 轮 1 个、22 轮 1 个，占 540 的 0.9%），上界 22 拒 0 个；上界不是限轮就完事，被拒的发布要有出路，等于要把收敛做快的算法，代价没有数 | 仍缺 |
| 式子判吃最坏数 | 落点判已经这么判（`pristine-source/crates/singlefs-core/src/transaction.rs:12803`）；固定点算出的需求在产物里是逐轮重写角色数 | 最坏数（`ckpt_cost` 口径）与固定点算出的数各是多少、差几倍，决定多拒多少：产物没有记最坏数；这一判挪到固定点前之后，固定点整段落进条款第二半「准入之后装单元、拷字节不算运行时决策路径」（`.claude/kb/decisions/08-核心索引结构.md:516`），B4 就不用改条款 | 仍缺 |

小结：这一轮的数已经够判的是「留 B1 加例外」与「只换整份拷」的条款那一半；仍缺代价数的是分块共享、B8 收严、式子判吃最坏数，以及「只换整份拷」的收益那一半（残差）。四个仍缺的量都要改插计数脚本或加一个小测量，属于新一次登记，我没有做。什么现象会推翻这一节的判：「留 B1 加例外」那行，若例外要同时豁免记录那一半，用户要定的就不止例外写多宽；「只换整份拷」那行，若放手占比（量放手占比那一项）< 0.02，收益那一半就从「仍缺」变成「X2 上换完仍剩 0.4 秒量级」；「B8 收严」那行，若 22 轮的成因是历史用例的取点而不是实现，这一行不用单立候选。


## 六、要主 agent 跑的命令

两项：一项复跑本报告里我算的全部读数，一项量第四节那个「残差是不是跟着整份拷走」的假设。都不带内存、线程与机器参数。

**复跑读数**

- 命令：`task-server/task submit -- bash /home/fy5090/.local/share/singlefs/work/e203-forks-r2-sonnet/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/e203-runner-revision-7/report-inputs --work /home/fy5090/.local/share/singlefs/work/e203-forks-r2-sonnet-rerun --model /home/fy5090/.local/share/singlefs/work/e203-forks-r2-sonnet`
- 起的目录：仓根 `/home/fy5090/code/singlefs`（`--work` 那个目录要不存在或是空的）。
- 判结果：`TASK_SERVER_RESULT` 那一行 `type=` 为完成、`exit=0`；`task-server/task log <号>` 末行 `name=rerun_done files=17`；再逐条对 `/home/fy5090/.local/share/singlefs/work/e203-forks-r2-sonnet-rerun/out/` 里的行（与本报告的数一一对应）：
  - `rows_diff.txt` 第 1 行：`total 2102 152344 alpha 0.6200642142367406`（第三节）；
  - `q6all.txt`：`F same_mount overwrite 2102 152344 0.62 G` 与 `u same_mount overwrite 2102 12789 0.261 B`（第三节）；
  - `s36_half.txt`：`不含 s36 1808 116314 0.6028111883626578`、`{10: 2102, 100: 2331, 1000: 6961, 10000: 152344} [0.045, 0.475, 1.34]`、`round 2 2098 4226 0.1014` 至 `round 5 2254 3826 0.0766`（第三节）；
  - `hist.txt`：`rounds histogram over 540 publishes [(2, 487), (3, 38), (4, 10), (5, 3), (6, 1), (22, 1)]`（第二、三、五节）；
  - `copies.txt`：`min rounds 2 max 22` 与 `copies s24 / rounds ratio set [1.0]`（第二节）；
  - `x2range.txt`：`X2 s1 exclusive s: min 1.390 max 1.719 n=30`（第二节）；
  - `mono2.txt` 末行：`下降对数 11 其中后一点的固定点轮数更少的 6`（总表里「下界只能用整份拷一行」那一行）；
  - `q10.txt`：`('U', 'same_mount', 'overwrite', 'between') [0.462, 0.41, 0.481, 0.482, 0.449] BBBBB`，与日志 2864 行之外的 q10 格逐个对（第四节）；
  - `residual_vs_copy.txt`、`s1_residual.txt`、`s1_rows.txt`、`noncopy.txt`：第四、五节引的数。
  不一致的那一行，以主 agent 复跑为准，我的报告对应处作废。

**量放手占比：整份拷贝的放手（drop）占拷贝（clone）的几成**

- 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/e203-forks-r2-sonnet -- bash tools/drop-vs-clone.sh /home/fy5090/.local/share/singlefs/work/e203-forks-r2-sonnet/drop-vs-clone.out`
- 起的目录：`/home/fy5090/.local/share/singlefs/work/e203-forks-r2-sonnet`（`rustc -O` 编 `tools/drop_vs_clone.rs`，没编过，编不过就是我的错，退出码非 0 时把编译输出交回来）。
- 判结果：`exit=0`，日志末行 `15`（三个槽数 × 5 轮的 `DROPVSCLONE` 行数）；读 `drop-vs-clone.out` 里 `unit_area_slots=1073740416` 那五行的 `drop_over_clone`：产物里 X2 的「between 段残差 ÷ 整份拷独占」是 0.162–0.190。落在 0.1–0.25 之间，第四节「残差跟着整份拷走」的解释得到支持（仍不是证明）；< 0.02，该解释被推翻，S2 的 U 轴残差另有来源。峰值内存约 15.6 GB（两块盘各一份原件加一份拷贝），撞上限由服务报，不影响复跑读数那一项。

我没往 `.deliverables` 写交接行：两项命令都只用草稿目录里的 `tools/`，这个目录在主 agent 复跑完之前请留着（没有仓副本、没有编译目录；`du -sh` 220K）。

## 七、没做什么

- 没往统一任务服务交任务，判决要引的数没复跑：两项命令在上一节；我报告里的读数在主 agent 跑出之前是「推的，待主 agent 跑」，只有写了日志行号的（q6、q9、q10、F1/F3 那几行）是日志里的原样。
- 没撤销也没维持任何一条判决，只交复核结果。第二节对 B1 下界、第三节对 F2 那一格、第四节对 B4 读法、第五节对候选的判定，是辩方立场的材料。
- 没复核上一批产物（上一轮判决引的 97.5–99.3% 与 0.583、1.32–1.42 秒、2.78 秒）：它们已被删，只能对新产物说「不再成立」或「数已变」。
- 没查 22 轮的成因：产物里没有，要新登记一条逐轮原因的差量读数。
- 没量最坏数与固定点算出数的差、没量分块共享要拷几块、没量撤回日志的开销：产物里没有，要改插计数脚本。
- 没动 `crates/`、登记、模型与插计数脚本；没写 kb。只写了本报告与草稿目录里的脚本。
- 没读禁读清单里的文件。第一轮两条腿的报告只通过 `git show 84b4ae23f:` 读到过，用来对第一轮的措辞，没引到本报告里。
