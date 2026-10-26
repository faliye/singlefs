# m4-gpugc-design-r1 正推腿（Sonnet）报告（2026-10-23）

立场：照正文「四、分工」正推一行，从 D26 已定项 13、D24 已定项 2 与 E185 四行读数正着推。不判别的腿的格，不替主 agent 采纳或出判决。

## 摘要（推荐一行一格；岔路在各格末尾的表里）

- G1：第一版 GPU 上只放扫描里的校验和（`parse_data_unit` 的头校验和与载荷 CRC）。甲乙丙加释放前核合计最多占一条意图主机 CPU 的 4.59%，GPU 一侧主机 CPU 一项没量，净收益的符号不知道；发布那一半不放（碰共享事务层）；Q9 不算翻面量。
- G2：乙-1（同一条意图内一次交），字节窗口开头验一次、状态每批重扫；固定开销压不下（φ ≥ 0.89），φ ≤ 10% 要 70–94 MiB，只有跨意图预取或宽 R 够得着，宽 R 要改盘上槽数字段且两种模式共用；wgpu 默认 128–256 MiB 内存块比小上限还大，产品要用 `Manual` 并按保留量比。
- G3：卡用 PCI 总线号写，不给时 = 已验厂商的独显、总线号升序，分活按窗口轮转、两份核同一次提交。
- G4：两份核算法不同（反射按位 × 正规按字节查表），输入与结果缓冲各一块，起卡前绝对值锚点；比对在 CPU；N = 2。
- G5：第一版会话方法（`&mut self`），批入口读一次；加打包搬迁只加一个枚举成员。
- G6：写带宽上限默认不限、兜底间隔默认 0（关）、显存上限不给时按用途上限取窗口；兜底与预算 `B_gc` 冲突（闲置时至多再发 20 次）。
- G7：新包，核心层不依赖它，CPU 参照用核心层现成的，与 checker 档不共用；第一版 WGSL 经 wgpu。

## 0　读法、复跑与数的来源

- 日期 2026-10-23。立场：正推；每格写候选（各自的定义）、推荐、依据；要交用户定的写岔路表（候选、翻面观测、够判条件，不写倾向）。
- 方案按 `crates/` 今天的实现来谈；引代码写主工作区路径与行号，与开工快照 `research/prompts/m4-gpugc-design-r1-snapshot/sha256sums.txt` 同一批文件（`mounted_session.rs`、`region_scan.rs`、`compaction_plan.rs`、`transaction.rs`、`mount.rs`、`system_configuration.rs` 的 sha256 我开工时现算，与快照逐行相同）。
- 引 kb 写 kb 文件名加那份文件自己的行号，行号是我在 kb 文件里 `grep -nF` 现查的；产物行号同理。
- 我量的数都在草稿目录 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-design-r1-sonnet/tools/`，只读 E185 入库产物，不编译、不起 GPU、不跑门禁。

复跑命令（仓根起；产物 sha256 与 E185 页第 23–25 行所写逐个相同，我现算过）：

```text
$ sha256sum research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out
f70eb21b8a7081489dd8fc3209080f3ab83f16ac40beee3042c03f4756f1e07a  research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out
18191ec3a8afbb27405987f99d3e36e5ae2538d3c9ab4f2717738495e767096b  research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out
$ python3 -I $D/tools/g1_upper_bound.py research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out
$ python3 -I $D/tools/callshares.py     research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out
$ python3 -I $D/tools/arms.py           research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out
```

（`$D` = `/home/fy5090/.local/share/singlefs/work/m4-gpugc-design-r1-sonnet`。后文还用到 `crc_bytes_per_intent.py`、`absolute_ms.py`、`g2_phi.py`、`g3_wall.py`、`g4_n_cost.py`、`intent_wall.py`、`batch_write_bytes.py`，每个的命令写在用到它的地方，全部脚本的 sha256 在文末「工具清单」。）

## G1　GPU 上算哪几段

### G1.1　尺与口径

D26 已定项 13 第 7 条的尺原文：「收益是后台任务少占的主机 CPU 时间（CPU 模式下它的 CPU 时间减 GPU 模式下它的 CPU 时间，后者含准备数据、提交、等结果、两份结果的比对，以及第 8 条交 CPU 判的那几条），挂钟与 CPU 模式持平也做」（`.claude/kb/decisions/26-后台整理与放置回收.md:242`）。写成式子：

```text
省下 = T_cpu(CPU模式，候选段) − [T_准备 + T_提交 + T_等结果 + T_两份核的主机侧 + T_比对 + T_交CPU判]    （全是主机 CPU 时间）
上界 = T_cpu(CPU模式，候选段)        （中括号里的项取 0；这是达不到的上界）
```

E185 只量到上界的左项；中括号里六项一项都没量：E185 第三段只记挂钟 `gpu_seconds`，不记主机线程 CPU（`research/results/e185-gc-batch-wall-clock-split-gpu-submission-r1-2026-10-22.out:83` 的字段是 `cpu_sixteen_threads_seconds=0.000025 gpu_seconds=0.004509543`，前者是 CPU 一侧重放、后者是 GPU 一侧挂钟）。所以这一格能给的是上界与「GPU 一侧每次提交的主机 CPU 至多多少才不亏」，给不出净收益的符号。

份额的分母统一用「一条意图的主机 CPU 总和」（签发那一次调用加全部批的 `call_cpu_nanoseconds`），因为 GC 的一次决策是一条意图，而 E185 页第 162–164 行 Q5–Q7 各自用「一批」「签发那一次」的分母，不能直接相加。两种分母的最大值都列在下面。

第二段（stage-split）的 `name=stage` 行字段：`crc_cpu_nanoseconds` 是甲（每个被读单元整单元 CRC-32C），`scan_cpu_nanoseconds` 包着甲（页第 281 行写「乙 整个 `scan_region`」；`scan_cpu − crc_cpu = scan_increment_cpu`，第 947 行 332311 − 257032 = 75279），`choose_cpu_nanoseconds` 只在签发那一次有值，`release_check_crc_cpu_nanoseconds` 是释放前读盘核那几份副本的整单元 CRC（页第 343 行：「甲段第二个子项不进 Q5、Q8、Q10 与第三段的 B_批，只报数」）。

### G1.2　每个候选的上界（命令与原样输出）

```text
$ python3 -I $D/tools/g1_upper_bound.py research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out
f    P    K  calls | intent_cpu_ms | jia%   jia+rel% yi_incr% bing%  | ceiling_jia_yi_bing(no rel)%  ceiling_all4% (max over 5 rounds) | ceiling_all4_ms(median) 
0.25 1024 4  3     |        34.982 |  0.59    1.06     0.19   1.06 |   1.84                         2.32 (max  2.36)             |   0.813
0.25 1024 8  2     |        23.081 |  0.72    1.44     0.25   1.61 |   2.58                         3.31 (max  3.38)             |   0.762
0.25 1024 16 2     |        22.875 |  0.73    1.46     0.24   1.63 |   2.61                         3.34 (max  3.36)             |   0.757
0.25 8192 4  3     |       284.963 |  0.07    0.13     0.17   0.14 |   0.37                         0.43 (max  0.46)             |   1.230
0.25 8192 8  2     |       185.196 |  0.09    0.18     0.21   0.21 |   0.51                         0.60 (max  0.60)             |   1.102
0.25 8192 16 2     |       185.505 |  0.09    0.18     0.21   0.21 |   0.51                         0.59 (max  0.61)             |   1.104
0.75 1024 4  7     |        84.599 |  1.31    1.90     0.41   0.44 |   2.16                         2.75 (max  2.81)             |   2.344
0.75 1024 8  4     |        48.779 |  1.53    2.54     0.49   0.76 |   2.78                         3.79 (max  3.89)             |   1.847
0.75 1024 16 3     |        36.258 |  1.60    2.96     0.51   1.01 |   3.12                         4.49 (max  4.59)             |   1.625
0.75 8192 4  7     |       694.101 |  0.16    0.23     0.35   0.06 |   0.57                         0.64 (max  0.67)             |   4.485
0.75 8192 8  4     |       393.457 |  0.19    0.31     0.41   0.10 |   0.70                         0.83 (max  0.86)             |   3.267
0.75 8192 16 3     |       296.311 |  0.19    0.36     0.40   0.13 |   0.73                         0.89 (max  0.92)             |   2.659
```

列的意思：`jia%` = 甲 / 意图 CPU；`jia+rel%` = 甲加释放前读盘核；`yi_incr%` = 乙的增量（头解析与映射反查，不含甲）；`bing%` = 丙；`ceiling_all4%` = 四项之和（甲、释放前核、乙增量、丙）。数是 5 轮的中位，括号里是 5 轮的最大。一条意图的 CPU 总和：P = 1024 为 22.9–84.6 ms，P = 8192 为 185.2–694.1 ms。

按调用（不是按意图）取的最大值，贴原样输出：

```text
$ python3 -I $D/tools/callshares.py research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out
batch  all_three_plus_release_check   max_share=0.049067 line=989 f=0.75 P=1024 K=16 round=1 kind=batch
batch  bing_choose                    max_share=0.000000 line=116 f=0.25 P=1024 K=4 round=1 kind=batch
batch  jia_crc                        max_share=0.020433 line=947 f=0.75 P=1024 K=4 round=1 kind=batch
batch  jia_crc_plus_release_check     max_share=0.043472 line=989 f=0.75 P=1024 K=16 round=1 kind=batch
batch  yi_plus_release_check          max_share=0.049067 line=989 f=0.75 P=1024 K=16 round=1 kind=batch
batch  yi_scan_whole(crc+incr)        max_share=0.026418 line=947 f=0.75 P=1024 K=4 round=1 kind=batch
issue  all_three_plus_release_check   max_share=0.072572 line=1090 f=0.75 P=1024 K=8 round=4 kind=issue
issue  bing_choose                    max_share=0.038940 line=132 f=0.25 P=1024 K=8 round=1 kind=issue
issue  jia_crc                        max_share=0.025603 line=1064 f=0.75 P=1024 K=16 round=3 kind=issue
issue  jia_crc_plus_release_check     max_share=0.025612 line=1064 f=0.75 P=1024 K=16 round=3 kind=issue
issue  yi_plus_release_check          max_share=0.034777 line=1064 f=0.75 P=1024 K=16 round=3 kind=issue
issue  yi_scan_whole(crc+incr)        max_share=0.034769 line=1064 f=0.75 P=1024 K=16 round=3 kind=issue
```

每个数取自哪一行，拿天花板最高的那个点（f = 0.75、P = 1024、K = 16）第 1 轮对一遍账（`g1_example.py`，原样输出）：

```text
$ python3 -I $D/tools/g1_example.py research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out 0.75 1024 16 1
行号 call_index kind call_cpu_ns crc_cpu_ns release_check_crc_cpu_ns scan_increment_cpu_ns choose_cpu_ns
984 0 issue 10104202 247971 80 82840 380892
989 1 batch 13894421 259441 344572 77751 0
996 2 final_batch 12780676 89940 176651 28370 0
合计 36779299 597352 521303 188961 380892
四项之和 = 1688508 ns = 1.689 ms；占意图 CPU 0.0459
```

行号是第二段产物的行号：第 984 行是签发（`kind=issue`，唯一有 `choose_cpu_nanoseconds` 的一行），第 989、996 行是批；四项之和 1.689 ms 占这条意图 36.779 ms 的 4.59%，就是上面表里 `max 4.59` 那个数。

### G1.3　候选与各自的上界

| 候选 | 定义（`crates/` 落点） | 上界取自哪一行 | 碰不碰 D24 已定项 2 第 2 条 / 一个事务层 |
|---|---|---|---|
| 甲 | 区域扫描里每个被读单元的整单元校验和：`parse_data_unit` 里的头校验和与载荷 CRC-32C（`crates/singlefs-core/src/unit.rs:391`、`:403`，`:581` 是入口），`scan_region`（`crates/singlefs-core/src/region_scan.rs:204`）逐条记录经 `read_unit_header` 调它 | 一批最大 0.020433（`research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out:947`）；意图中位 0.07%–1.60%（上表 `jia%`） | 不碰：整理自己的扫描，不是前台路径 |
| 甲加释放前核 | 再加发布路径里释放前对两份副本逐份读盘核的 CRC（`crates/singlefs-core/src/transaction.rs:5187` `read_one_copy_and_check_it_against_its_location_entry`，由 `:5084` `copies_failing_the_release_checksum_check` 逐个要释放的单元调） | 一批最大 0.043472（`:989`，释放前核 344572 ns / 13894421 ns 加甲 259441 ns）；意图中位 0.13%–2.96%（`jia+rel%`） | **碰**：这一段在共享发布路径里，前台的发布同样经它（`crates/singlefs-core/src/transaction.rs:11065` 的 `ReleaseChecksumCheck::ReadEveryReplacedCopyBeforeReleasingIt` 在 `publish_version_of_trees_relocating_units_after_a_check` 里，是 `publish_version_of_trees` 与整理的批共用的那一段，见 `crates/singlefs-core/src/transaction.rs:11098` 的文档「同 [`publish_version_of_trees`]，另带这次要搬的单元」）；要放 GPU 就要给共享事务层加一个 GPU 分支 |
| 乙 | `scan_region` 整个：甲加增量（头解析 `read_unit_header`、`classify_allocation` 里按槽反查映射条目与按头里 key 反查，`crates/singlefs-core/src/region_scan.rs:204` 起） | 一批最大 0.026418（`:947`）；乙增量单项最大 0.006521（E185 页第 163 行） | 不碰 |
| 丙 | 选段打分 `choose_region`（`crates/singlefs-core/src/compaction_plan.rs:88`），只在签发那一次调（`mounted_session.rs` 签发分支的 `loop { match choose_region(…` 在 `:3093`） | 签发那一次最大 0.03894（`research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out:132` 的 `choose_share_of_the_issue`）；意图中位 0.06%–1.63% | 不碰 |
| 甲乙丙全放 | 以上三段（不含释放前核） | 意图中位合计 0.37%–3.12%；释放前核也算时 0.43%–4.49%，5 轮最大 4.59%（`ceiling_all4%`）；调用口径：一批最大 4.9067%（`:989`），签发那一次最大 7.2572%（`:1090`） | 不碰（不含释放前核时） |
| 发布那一半 | 读这一版 `version_as_the_compaction_reads_it`（`crates/singlefs-core/src/mounted_session.rs:2835`，每推一步调一次，`:3026`）、整版克隆（`:3005`）、`publish_relocation_batch`（`crates/singlefs-core/src/transaction.rs:9675`）及它走的 `publish_compaction_step`（`:9748`） | 没有按函数拆的 CPU 数（见下） | 事务层部分**碰** |

「别的拆法」三种，都不是 GPU 候选，是同一把尺下的对照项：

1. 扫描里的校验和今天每批重验一遍：每条意图的甲所校验的字节数是该意图 R 里活数据字节数的 2.0–4.5 倍（f = 0.25 时 2.0–2.5 倍；f = 0.75：一条意图各调用的 `crc_bytes` 合计 1835008（K = 16）、2359296（K = 8）、3538944（K = 4），R 里活数据 786432；`python3 -I $D/tools/crc_bytes_per_intent.py <第二段产物>` 对第 1 轮各 `name=stage` 行的 `crc_bytes` 现加，原样输出贴在 G1.5）。这份重复是 `mounted_session.rs` 第 3191 行起「每批之前都重扫」带来的（对 R 重扫的判决在 `advance_compaction_by_one_publish` 文档里写成「对 R 重扫（每批之前都重扫，丁 1 交判第 3 点）」，`crates/singlefs-core/src/mounted_session.rs:2975`）。把 CPU 侧这份重复去掉（例如意图内只验一次）会同时降低 CPU 模式的甲，GPU 模式的上界随之降到同一个数。属于 G2「攒批中途要不要重扫」的岔路，见 G2。
2. CPU 侧校验和实现是 slicing-by-8 软件表（`crates/singlefs-core/src/checksum.rs:43` `crc32_castagnoli`），E185 第二段量到甲单线程 2.875–3.189 GB/s（E185 页第 165 行 Q8）。换成硬件 CRC 指令的实现会让甲的 CPU 时间降一截；这是对 CPU 基线的改动，我没量（硬件指令的速度是我对公开常识的推测，标「推的」），列出来是因为它直接压低 GPU 模式能省的数。
3. 乙增量的绝对 CPU 随池大小线性涨，原因是 CPU 侧的两处线性查找：`classify_allocation` 对每条记录各扫一遍全部映射条目（`crates/singlefs-core/src/region_scan.rs:286` 起的 `mapping_entries.iter().filter(… points_at …)` 与 `:309` 起的 `mapping_entries.iter().any(… entry.key == header.mapping_key)`）。`absolute_ms.py` 对第二段产物按意图合计各段的绝对毫秒（原样输出）：

```text
$ python3 -I $D/tools/absolute_ms.py research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out
f    K  P     | crc_ms  scan_incr_ms  choose_ms  release_check_ms | intent_cpu_ms
0.25 4  1024  |  0.206        0.068      0.372            0.166 |     34.982
0.25 4  8192  |  0.206        0.474      0.385            0.166 |    284.963
0.25 8  1024  |  0.165        0.057      0.372            0.165 |     23.081
0.25 8  8192  |  0.165        0.390      0.384            0.165 |    185.196
0.25 16 1024  |  0.165        0.055      0.372            0.166 |     22.875
0.25 16 8192  |  0.165        0.387      0.386            0.165 |    185.505
0.75 4  1024  |  1.114        0.349      0.370            0.496 |     84.599
0.75 4  8192  |  1.118        2.414      0.384            0.496 |    694.101
0.75 8  1024  |  0.744        0.236      0.371            0.495 |     48.779
0.75 8  8192  |  0.743        1.603      0.381            0.495 |    393.457
0.75 16 1024  |  0.578        0.184      0.367            0.495 |     36.258
0.75 16 8192  |  0.577        1.202      0.384            0.495 |    296.311
```

读法：P 从 1024 到 8192（8 倍），乙增量从 0.349 ms 到 2.414 ms（6.9 倍，f = 0.75、K = 4），而甲、释放前核、丙（盘大小没变）不变。**乙增量的份额不涨（0.2%–0.65%）是因为分母也线性涨**，绝对值在涨。这是 `.claude/kb/milestone/04-fourth-txn.md` 七·6 同一类的线性查找（按身份建索引可去），属 CPU 侧修法，GPU 不是它的对口办法。

### G1.4　三问

**问 1：每个候选在第 7 条这把尺下能省多少。** 上界见 G1.2 表与 G1.3 表；净值不知道，原因两条：（i）中括号里六项没量；（ii）E185 量到的 GPU 一侧只有挂钟：每次提交固定约 4.49 ms（卡 0）或 10.30 ms（卡 1）（E185 页第 200 行 Q11、Q12：`intercept_seconds=0.004492` 与 `intercept_seconds=0.010300`）。拿最好的一个点算：f = 0.75、P = 1024、K = 16，一条意图 3 次调用（签发 + 2 批）、四项上界合计中位 1.625 ms（上表最后一列）；若每次调用提交一次，主机 CPU 的预算是 1.625 ms ÷ 3 = 0.54 ms 每次提交，只占卡 0 挂钟截距 4.49 ms 的 12%、卡 1 的 5%。**只有等结果时线程不烧 CPU（睡眠）、加上准备、提交、读回、两份核各自的编码、比对合起来 ≤ 0.54 ms，才不亏；线程在驱动里自旋等，CPU 时间就约等于挂钟，必亏。** 驱动怎么等没量过；这是 G1 的第一条翻面观测。

**问 2：三段都不到 10% 时，省下的 CPU 时间从哪来。** 从这三段本身，且只有这三段：它们合计在每个测过的点上不超过一条意图 CPU 的 4.59%，这是在 GPU 一侧主机开销为 0 时的天花板。其余 95% 以上（一批里设备调用本身的 CPU 只占 0.3%–4.7%，`arms.py` 输出的 `device_cpu_share` 列，下面贴；剩下的是核心层在内存里读版本、克隆、重排账、发布）不在三段之内。

```text
$ python3 -I $D/tools/arms.py research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out | (sed -n 1p; grep ' batch |')
arm f P K kind | n | cpu_ms(med) device_cpu_ms(med) device_cpu_share wall_ms(med)
memory 0.25 1024 4 batch | 10 | 12.542 0.168 0.013 12.542
memory 0.25 1024 8 batch | 5 | 12.978 0.221 0.017 12.979
memory 0.25 1024 16 batch | 5 | 12.956 0.222 0.017 12.956
memory 0.25 8192 4 batch | 10 | 105.758 0.284 0.003 105.768
memory 0.25 8192 8 batch | 5 | 111.090 0.365 0.003 111.090
memory 0.25 8192 16 batch | 5 | 110.840 0.356 0.003 110.841
memory 0.75 1024 4 batch | 30 | 12.505 0.189 0.015 12.508
memory 0.75 1024 8 batch | 15 | 12.865 0.236 0.018 12.866
memory 0.75 1024 16 batch | 10 | 13.434 0.307 0.023 13.434
memory 0.75 8192 4 batch | 30 | 103.309 0.267 0.003 103.327
memory 0.75 8192 8 batch | 15 | 102.420 0.320 0.003 102.437
memory 0.75 8192 16 batch | 10 | 107.176 0.376 0.004 107.182
real_disk 0.25 1024 4 batch | 10 | 13.046 0.482 0.037 21.671
real_disk 0.25 1024 8 batch | 5 | 13.475 0.530 0.039 22.764
real_disk 0.25 1024 16 batch | 5 | 13.494 0.526 0.039 23.195
real_disk 0.25 8192 4 batch | 10 | 105.804 0.698 0.007 116.797
real_disk 0.25 8192 8 batch | 5 | 110.557 0.788 0.007 123.135
real_disk 0.25 8192 16 batch | 5 | 110.457 0.774 0.007 123.369
real_disk 0.75 1024 4 batch | 30 | 13.080 0.501 0.038 23.728
real_disk 0.75 1024 8 batch | 15 | 13.503 0.574 0.043 24.255
real_disk 0.75 1024 16 batch | 10 | 14.039 0.661 0.047 25.408
real_disk 0.75 8192 4 batch | 30 | 104.365 0.717 0.007 115.143
real_disk 0.75 8192 8 batch | 15 | 103.184 0.781 0.008 114.157
real_disk 0.75 8192 16 batch | 10 | 107.410 0.857 0.008 119.442
```

**问 3：发布那一半里有没有能放 GPU、又不碰 D24 已定项 2 第 2 条与「一个事务层」的部分。**

先说清「那一半」今天有多大：一批里设备调用的 CPU 只占 0.3%–4.7%（上表），甲乙丙合计最多 4.9%，其余 90% 以上是核心层内存里的计算。这一半的拆法**没有按函数拆的 CPU 数**：E185 页第 218 行 F7 写的是「CPU 的大头在其余（读这一版、发布）」，是减出来的，不是量的。能引的唯一一份拆法是七·1 调查员在改身份索引之前的读数（`records/2026-10-19-里程碑四GPU垃圾回收开工.md:68`：「A 约 44% `mounted_session.rs` `version_as_the_compaction_reads_it`」「发布路径约 39%」「B 约 15% `advance_compaction_by_one_publish` 两次整版克隆」），它量在七·6 的按身份索引改完之前；E185 第二次重跑的树已含那批改动（`grep -c FirstEntryOfEachIdentity crates/singlefs-core/src/transaction.rs` 我现数输出 29；`research/results/e185-r2-inputs-stage-split-2026-10-23.txt` 第 2 行记的 `transaction.rs` sha256 `a891136e…` 与今天主工作区那份相同），改后的拆法没有人量。所以这一半「各占几成」今天是**未量**。

逐项能不能放 GPU：

| 子项 | 落点 | 能放吗 | 依据 |
|---|---|---|---|
| 读这一版 | `crates/singlefs-core/src/mounted_session.rs:2835` `version_as_the_compaction_reads_it`、`:2807` `central_mapping_as_the_compaction_reads_it`（按 key 逐叶解码查映射） | 不放 | 输入是内存里的树节点结构，不是平的字节表；准备数据就是把它再展平一遍，展平的 CPU 与被省的同阶（推的，没量）。还是只读纯函数，所以是**CPU 侧优化**的对象（缓存、增量），不是 GPU 的对象 |
| 整版克隆 | `crates/singlefs-core/src/mounted_session.rs:3005` | 不放 | 内存拷贝，没有计算 |
| 发布本身（树重写、分配记录树重排、中央映射换位置、写盘装配） | `crates/singlefs-core/src/transaction.rs:9675` `publish_relocation_batch` → `:9748` `publish_compaction_step` | 不放 | 同一个发布入口族前台用户发布也走（`records/2026-10-19-里程碑四GPU垃圾回收开工.md:68`：「B–F 是事务提交本身，前台用户发布也付」）；`.claude/rules/fs-design.md:83`「不许为某个路径（如 fsync）开第二套事务机制。」，GPU 分支进共享事务层即是；D24 已定项 2 第 2 条「不许进前台路径。」（`.claude/kb/decisions/24-后台重活能不能卸给GPU.md:35`） |
| 发布里的释放前核 CRC | `crates/singlefs-core/src/transaction.rs:5187` | 不放 | 同上，且「释放之前读盘核」的意义就是释放前一刻读两份副本（D19 已定项 5 硬规则 1，`crates/singlefs-core/src/transaction.rs:5066` 的文档开头写「释放之前按映射条目的位置项读盘核校验和」）；GC 提前算好再交给发布会让这一核不再是释放前读的，是语义改动 |

结论：发布那一半里**没有**同时满足「不碰前台路径、不碰共享事务层、输入是平的字节表」的部分。F7 要补的那个「发布那一半」的臂，我的结论是不补 GPU 臂；应当补的是一条 CPU 侧的测量（按函数拆开改身份索引之后的一批 CPU），否则 GPU 模式的天花板（上界 4.59%）和被它压着的 90% 以上之间没有数可对。这条测量不是 GPU 候选，列进岔路表。

**问 4：Q9 要不要算翻面量。**

Q9 是 16 线程 CPU 的挂钟对 GPU 的挂钟（`research/results/e185-gc-batch-wall-clock-split-gpu-submission-r1-2026-10-22.out:118` 整行：`ratio=0.007746 judgement=gpu_loses … ratio_one_thread=0.112268 judgement_one_thread=gpu_loses`）。第 7 条的原文已经写明：「D24（后台重活能不能卸给 GPU） 已定项 3 那条按挂钟比的判据不管这一项罩着的后台任务」（`.claude/kb/decisions/26-后台整理与放置回收.md:242`）。所以 Q9 不是第 7 条那把尺，**不是翻面量**：它不回答「少占多少主机 CPU 时间」。另外它的比较对象 16 线程不对口径：整理今天在调用线程上同步跑（E185 页第 345 行：「今天的 GC 整条路在调用线程上同步跑」），CPU 模式下的基线是单线程。72 行里单线程版 `ratio_one_thread` 最大 0.126064（乙增量），`judgement_one_thread=gpu_loses` 72 行全是，无一行例外。

Q9 仍有用的两处：一是给出 GPU 一次提交的挂钟地板，`gpu_seconds` 4.499–11.296 ms（同一产物第 83–200 行的 72 行，两张卡），喂 G2；二是第 7 条写「挂钟与 CPU 模式持平也做」，没写比 CPU 模式慢多少还做：P = 1024 时一批 CPU 模式挂钟 12.5–13.4 ms（内存盘，上面 `arms.py` 输出），加一次提交的 4.5 ms 地板是 +34%–36%，两份核分两次提交就是 +67%–72%（同一次提交里放两个计算通道则约 +34%–36%）；P = 8192 时一批 102–111 ms，加 4.5 ms 是 +4%。「持平」在 P = 1024 这一档不成立，条款没说这时做不做。这是岔路，不是 Q9 的判定。

### G1.5　推荐

**推荐（依据全在上面的数）：第一版 GPU 上只放扫描里的那一份校验和（`parse_data_unit` 里的头校验和与载荷 CRC-32C，`crates/singlefs-core/src/unit.rs:391`、`:403`；E185 里叫甲）。** 理由三条：

1. 只有它的输入是平的字节表，而且扫描已经把这些字节读进来了（`region_scan.rs` 的 `read_unit_header` 经闭包 `read_unit_bytes` 取字节），「准备数据」是把已读的单元字节拼成一块缓冲，不是重新展平一棵树。
2. 它的 CPU 参照是现成的一个纯函数（`crates/singlefs-core/src/checksum.rs:43` `crc32_castagnoli`），D13 已定项 5 把 CRC-32C 的参数钉死（`.claude/kb/decisions/13-验证路线.md:160`：「CRC32C 各写一份的前提是参数钉死」），两份独立的 GPU 核与一份 CPU 参照的写法都有现成的判据（G4）。
3. 乙的增量（头解析、映射反查）最大 0.65%，输入是随每次发布变的映射条目，攒不起来（G2.4），而且它的绝对 CPU 随 P 涨是 CPU 侧线性查找造成的（上面第 3 条），先在 CPU 侧修；丙是签发时一次；释放前核与发布在共享事务层；各自要么份额小、要么输入不对口、要么碰事务层。

**这个推荐的天花板很低，要明说：** 甲在每个测过的点上限是一条意图 CPU 的 0.07%–1.60%（上表 `jia%`），一批最大 2.04%（页第 162 行 Q5 的 0.020433）。净收益符号不知道（问 1），不是推荐「它一定省」。

丙是否同放，是岔路（G1.6 表的 G1-a 行）。理由：甲和释放前核被 R 钉住（一条意图的 R 是一个 64 槽聚簇段，`crates/singlefs-core/src/logged_operations.rs:303` 读者拒收槽数不是 64 的条目，一块盘上至多 64 × 16384 = 1048576 字节），绝对 CPU 不随池变大（上面 `absolute_ms.py` 输出：crc_ms、release_check_ms 在 P = 1024 与 8192 相同）；乙增量随 P 涨但属 CPU 侧可修；丙每次调用的 CPU 随单元区整段数涨（两个盘大小外推）：第二段 `name=choose_disk_size` 的 `cpu_nanoseconds_per_full_segment` 最小 69.58 ns，推到 1 TiB 每次调用约 73.0–77.2 ms，`extrapolation=inferred`（`research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out:115` 整行：`cpu_nanoseconds_per_full_segment=70.046860 one_tebibyte_extrapolated_nanoseconds_per_call=73449456.170048 extrapolation=inferred`）。按这个 70 ns，丙的一次调用超过卡 0 一次提交的挂钟截距 4.49 ms，要单元区整段数 4.492 ms ÷ 70.05 ns ≈ 64125 段（≈ 62.6 GiB 的单元区）。但丙的输入是内存里的分配记录向量，GPU 要用得先展平（准备数据），这一步 CPU 与丙本身同阶（推的，没量）。所以丙是否值得放，取决于两个没量的数。

原样输出（`crc_bytes_per_intent.py`）：

```text
$ python3 -I $D/tools/crc_bytes_per_intent.py research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out
('0.25', '1024', '16') sum_crc_bytes 524288 sum_release_check_bytes 524288 calls 2
('0.25', '1024', '4') sum_crc_bytes 655360 sum_release_check_bytes 524288 calls 3
('0.25', '1024', '8') sum_crc_bytes 524288 sum_release_check_bytes 524288 calls 2
('0.25', '8192', '16') sum_crc_bytes 524288 sum_release_check_bytes 524288 calls 2
('0.25', '8192', '4') sum_crc_bytes 655360 sum_release_check_bytes 524288 calls 3
('0.25', '8192', '8') sum_crc_bytes 524288 sum_release_check_bytes 524288 calls 2
('0.75', '1024', '16') sum_crc_bytes 1835008 sum_release_check_bytes 1572864 calls 3
('0.75', '1024', '4') sum_crc_bytes 3538944 sum_release_check_bytes 1572864 calls 7
('0.75', '1024', '8') sum_crc_bytes 2359296 sum_release_check_bytes 1572864 calls 4
('0.75', '8192', '16') sum_crc_bytes 1835008 sum_release_check_bytes 1572864 calls 3
('0.75', '8192', '4') sum_crc_bytes 3538944 sum_release_check_bytes 1572864 calls 7
('0.75', '8192', '8') sum_crc_bytes 2359296 sum_release_check_bytes 1572864 calls 4
```

R 里活数据字节：f = 0.25 是 262144，f = 0.75 是 786432（`scan_read_bytes_in_the_region` 同值）。校验的字节 ÷ 活数据字节：f = 0.25 为 2.0–2.5 倍，f = 0.75 为 2.3–4.5 倍。

### G1.6　交用户定的岔路（G1）

| # | 候选（各自的定义） | 翻面观测 | 够判条件 |
|---|---|---|---|
| G1-a | 第一版 GPU 集合：A = 只放甲（扫描里的整单元校验和）；B = 甲加丙（选段打分，签发时一次）；C = 甲、乙、丙全放（乙的头解析与映射反查也放） | ① 一次 GPU 往返（准备缓冲、提交、等结果、读回、两份核、比对）的**主机线程 CPU 时间** c_rt（`CLOCK_THREAD_CPUTIME_ID`），按提交大小 128 KiB、262144 B、524288 B、1048576 B 各量；c_rt 大于「该候选一次调用的 CPU 时间」时该候选翻成亏。② 真池上丙一次调用的 CPU 与把分配记录向量展平成 GPU 输入的 CPU（单元区整段数 ≥ 64125 的池上），展平 CPU ≥ 丙本身的 CPU 时丙翻成亏 | ① 在两张卡上各量 ≥ 5 轮、四个提交大小各一，写明等结果是阻塞还是自旋；② 在 ≥ 3 个单元区大小（含 ≥ 64125 段）上量到 choose 与展平两个 CPU。E185 页第 3 段（`gpu-submission`）只记挂钟，这两样都要新量 |
| G1-b | Q9 的用法：是翻面量（按 D24 已定项 3 的挂钟比判 GPU 模式）／不是翻面量（只按第 7 条的 CPU 时间）／不是翻面量但设挂钟护栏（GPU 模式一批挂钟不超过 CPU 模式的 X 倍，X 用户定） | GPU 模式一批挂钟 ÷ CPU 模式一批挂钟：P = 1024 档 +34%–36%（一次提交）、+67%–72%（两次提交），P = 8192 档约 +4%（上文算术）；换成实测 c_rt 对应的挂钟后再比 | X 定了；GPU 模式的批挂钟在 P = 1024、8192 两档上各量到一次对照 |
| G1-c | 七·5 对照组里 GPU 模式对着哪个 CPU 基线：今天的 CPU 路径（扫描每批重验一遍、校验和软件表）／先在 CPU 侧去掉重复校验、换硬件 CRC 指令之后的 CPU 路径 | 去掉重复后甲的份额：校验字节从活数据字节的 2.0–4.5 倍降到 1 倍时，甲上界从 0.07%–1.60% 降到约 1/2–1/4.5 | 量到 CPU 侧去冗余后的一批 CPU（同装置同点）；用户定对照的基线 |
| G1-d | 要不要先补一条 CPU 侧测量：改身份索引之后一批 CPU 按函数拆开（读这一版、克隆、发布各占几成） | 读这一版加发布占一批 CPU 的几成；它决定甲乙丙四项的天花板（现 ≤ 4.59%）离「值得做」还差多远 | 在 P = 1024、8192、f = 0.25、0.75 四个点上按函数计 CPU（装置按函数计，口径写明），每点 5 轮 |

### G1.7　E185 的 a 是上界，不是产品能做到的下界

E185 第三段的固定开销 a（卡 0 的 4.49 ms、卡 1 的 10.30 ms）是用 `GpuCard::crc32c_of_ranges` 量的（E185 页第 282 行：「`GpuCard::crc32c_of_ranges` 先丢一次再计 5 次取中位」）。这个函数**每次调用都**在主机上分一块 `count × 32 KiB` 的缓冲并拷贝（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:778`–`:789`），再新建四个带初值的缓冲、一个回读缓冲、一个 bind group、一个命令编码器（`:798`、`:806`、`:814`、`:823`、`:831`、`:838`、`:861`），提交后 `map_async` 回读并 `device.poll(wgpu::PollType::Wait { … })` 阻塞等（`:879`、`:881`）。所以 4.49 ms 里含每次建缓冲、建 bind group 的开销；产品用常驻缓冲、常驻 bind group 时的固定开销会小多少，E185 没量，我也没有数。这一条只改一件事：G1.4 问 1 里「预算 0.54 ms 对 4.49 ms」是**对着装置写法的上界**，产品写法的固定开销可能低于它；净收益的符号仍要 c_rt（G1-a）才定。`device.poll(Wait)` 在驱动里是睡眠还是自旋，同样没量。

## G2　攒批的形态与显存上限

### G2.1　候选的定义

问题单第 3 行（`research/prompts/m4-gpugc-measure-forks.md`）的三个候选原文：甲 一批一交 ／ 乙 连续几批攒起来一次交，几批由显存上限定 ／ 丙 放宽一条意图的 R 到不止一个聚簇段。甲已经出局（E185 页第 249 行：56 行全判 `merge_batches`，φ 最小 0.911860，「一批一交（甲）出局」）。D26 已定项 13 第 3 条自己写的是乙：「GPU 模式把连续几批的计算攒起来一次交给 GPU，攒多少受它管」（`.claude/kb/decisions/26-后台整理与放置回收.md:238`）。

「连续几批」在 `crates/` 今天的实现里有两种读法，量级差两个数量级，要拆开：

- **乙-1　意图内**：同一条意图的各批一起交。字节来源只有这一条意图的 R：一块盘上一个聚簇段，`CLUSTER_SEGMENT_SLOTS` 64 槽（`crates/singlefs-format/src/lib.rs:325`）、槽 16384 字节（`:14`），至多 1048576 字节。同一时刻只有一条整理意图（`crates/singlefs-core/src/logged_operations.rs` 读者在多于一条整理意图时拒收，`LoggedOperationsRootRefusal::MoreThanOneCompactionIntent`，`:393`）。
- **乙-2　跨意图**：一次提交覆盖当前意图的 R 和随后几个候选段。下一条意图要等上一条完成、过全局差值判（`mounted_session.rs` 的 `NoYieldFromTheLastIntent` 分支，`:3049`）才签，选哪一段由签发时那一版的 `choose_region` 定，提前交的字节如果后来没被选中就白算。
- **丙　宽 R**：R 放宽到 W 个聚簇段。`crates/` 里 R 的槽数不存：`CompactionRegion` 只有设备与起始槽两个字段（`crates/singlefs-core/src/logged_operations.rs:71`），读者拒收槽数不是 64 的条目（`crates/singlefs-core/src/logged_operations.rs:303` 的 `if u64::from(slot_count) != CLUSTER_SEGMENT_SLOTS`），D26 已定项 2 的字段表同写：「`R` 的槽数 | 2 | 20 | 恒 64（一个聚簇段），别的值拒收」（`.claude/kb/decisions/26-后台整理与放置回收.md:64`）。宽 R 要改这一格的取值与读者的拒收，**CPU 模式读同一条意图也得认**（D24 已定项 2 第 1 条：文件系统必须在没有 GPU 时完整工作、语义完全相同，`.claude/kb/decisions/24-后台重活能不能卸给GPU.md:34`），所以它改的是两种模式共用的整理语义，不是 GPU 模式独有的参数。

### G2.2　算术：一次提交要多大才不被固定开销吃掉

E185 第三段给的 a、b（页第 200 行 Q11、Q12：卡 0 `intercept_seconds=0.004492 slope_seconds_per_byte=4.117289e-10`，卡 1 `intercept_seconds=0.010300 slope_seconds_per_byte=1.266004e-9`），φ = a ÷ (a + b·B)，φ ≤ 10% 判「一批一交够」、φ ≥ 50% 判「要攒批」，n* = ⌈9a ÷ (b·B_批)⌉（`research/prompts/e185-preregistration.md:332`）。我用同样的 a、b 对不同的一次提交字节数算 φ（`g2_phi.py`，原样输出）：

```text
$ python3 -I $D/tools/g2_phi.py
卡 0 a=0.004492 s b=4.117289e-10 s/B
  一条意图 f=0.25 的 R 全部活字节                                          B=   262144  φ=0.9765
  一条意图 f=0.75 的 R 全部活字节                                          B=   786432  φ=0.9328
  一整个 R（64 槽）                                                    B=  1048576  φ=0.9123
  K=1、R 全满、每批重扫的字节合计（32+31+…+1 个数据单元）                            B= 17301504  φ=0.3867
  f=0.75、K=4 每批重扫的字节合计（页 Q10 同一批的 6 次调用）                         B=  2752512  φ=0.7985
  φ≤10% 需要一次提交 ≥ 9a/b = 98190824 字节 = 93.6 MiB = 93.6 个 R(1 MiB)
卡 1 a=0.010300 s b=1.266004e-09 s/B
  一条意图 f=0.25 的 R 全部活字节                                          B=   262144  φ=0.9688
  一条意图 f=0.75 的 R 全部活字节                                          B=   786432  φ=0.9119
  一整个 R（64 槽）                                                    B=  1048576  φ=0.8858
  K=1、R 全满、每批重扫的字节合计（32+31+…+1 个数据单元）                            B= 17301504  φ=0.3198
  f=0.75、K=4 每批重扫的字节合计（页 Q10 同一批的 6 次调用）                         B=  2752512  φ=0.7472
  φ≤10% 需要一次提交 ≥ 9a/b = 73222517 字节 = 69.8 MiB = 69.8 个 R(1 MiB)
```

读法：

1. 乙-1 的上限是一个满 R（1048576 字节）：φ = 0.9123（卡 0）、0.8858（卡 1）。**乙-1 不管攒到多少批，φ 都不会离开「要攒批」那一档（≥ 50%）**。唯一越过 50% 的是「K = 1、R 全满、每批都重扫重验」这个极端：每批重扫的字节合计 17301504（0.3867 / 0.3198），而它靠的恰恰是每批把没搬的单元重验一遍的那份重复（G1.3 的「别的拆法」第 1 条）。
2. φ ≤ 10% 要一次提交 ≥ 69.8 MiB（卡 1）–93.6 MiB（卡 0），即 70–94 个满 R。只有乙-2（预取 70–94 个候选段）与丙（W = 70–94 个聚簇段）够得着。E185 第三段产物里 `merged_buffer_bytes=98364016`（`research/results/e185-gc-batch-wall-clock-split-gpu-submission-r1-2026-10-22.out:61`）与 `merged_buffer_bytes=73969744`（同文件 `:143`）是同一个量加上结果缓冲。
3. 乙-1 的 φ 与 E185 第三段实测的 Q13 值同值（f = 0.75 的 786432 字节：我算 0.9328 与 0.9119，页第 201 行写 0.932762 与 0.911860）。

### G2.3　两种形态各自怎么受 `background_gpu_memory_limit` 管

条款：「GPU 模式下全部后台任务在那张卡上合占的显存不超过它的那一个数……任何时刻都不许比它多，攒批、缓冲、暂存区一起算在里面」（`.claude/kb/decisions/26-后台整理与放置回收.md:238`）。要数的项：输入缓冲（合并字节）、两份核的结果缓冲、读回缓冲（主机可见的暂存区）、管线与着色器模块、wgpu 的内存块粒度。

- **块粒度比 limit 大**：`wgpu-hal-30.0.1/src/lib.rs:446` 默认的 `MemoryHints::Performance` 设备内存块 128–256 MiB（`min_device_memblock_size: 128 * MB`），`MemoryUsage` 是 8–64 MiB（`:452`），`Manual` 把下限钳在 4 MiB（`:469`）。checker 档的起卡写法是 `memory_hints: wgpu::MemoryHints::Performance`（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:523`）：照它起，第一次分配就占一整块 ≥ 128 MiB，任何小于 128 MiB 的 limit 都在第一次分配时被破。产品要用 `Manual`（块大小范围显式给）或 `MemoryUsage`，而且 limit 要拿**块的保留量**比，不是缓冲的逻辑字节。保留量 wgpu 暴露在 `Device::generate_allocator_report()`（`wgpu-30.0.1/src/api/device.rs:630`，返回 `Option<AllocatorReport>`，含 `total_reserved_bytes`，`wgpu-types-30.0.1/src/counters.rs:177`）；返回 `None` 的后端要自己记账。（这几处在 `~/.cargo/registry/src/*/` 下，不在仓里，我现读的。）
- **乙-1**：输入 ≤ 1 MiB，结果与读回 ≤ 几百字节（每单元一个 u32、两份核），实际占用被块粒度定下限，不被字节数定：用 `Manual` 钳到 4 MiB 块，设备内存最小占用是一两个块（8 MiB 量级，推的，没起过设备量）。limit 在这个量级以上都够，**limit 在乙-1 里几乎不起限制批数的作用**，起作用的是 limit 太小时起不了卡（报错、拒绝挂载，D26 已定项 13 第 4 条）。
- **乙-2 / 丙**：输入 = W MiB。limit 定 W：φ 要到 10% 得 W ≥ 70（卡 1）–94（卡 0）MiB。limit = 64 MiB 时，W 至多约 60 MiB，φ ≈ 0.15（卡 0）、0.11（卡 1）（`g2_phi.py` 的式子，B = 62914560）。两份核是否共用输入缓冲决定占用是 1× 还是 2×（G4）。
- **查得到总量与空闲量吗**：第 4 条要求「某一个数超过那张卡的显存总量」拒绝，第 3 条要求「卡上此刻空闲的不到这个数也照常跑，只用空闲的那些」。`wgpu-types-30.0.1/src/adapter.rs:145` 的 `AdapterInfo` 没有显存字段。checker 档的做法是起子进程 `nvidia-smi --query-gpu=pci.bus_id,index,memory.free`（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:323`）再按 PCI 总线号对上 wgpu 适配器，只认 NVIDIA 独显（`:453`–`:454`）。产品怎么拿总量与空闲量，`crates/` 里没有答案，列进岔路表 G2-c。E163 页第 33 行有一例：另一台一张 RTX 5080 空闲显存 141 MiB 时 `request_device` 交回 `Device(OutOfMemory)`。

### G2.4　攒批中途要不要重扫

今天每批重扫：`crates/singlefs-core/src/mounted_session.rs:2975`「对 R 重扫（每批之前都重扫，丁 1 交判第 3 点）」，代码在 `:3191` 起（`scan_of(region, &*devices)`），每批都重新经闭包读 R 里每条记录的单元字节并解头、校验（`crates/singlefs-core/src/region_scan.rs:204` 起，`read_unit_header`）。

要分开看的是两样输入：

- **字节**（甲的输入）：R 在意图期内不可作分配目的地（D26 已定项 1 第 3 条，`.claude/kb/decisions/26-后台整理与放置回收.md:30`「`R` 在意图期内不可作分配目的地」），R 里槽的字节在这条意图期内不被前台新写覆盖；discard 在整条 `run_compaction` 之后才发（`crates/singlefs-core/src/mounted_session.rs:2012` `run.discards = discard_the_segments_reusable_as_a_whole(…)`）。所以同一个槽的字节在两批之间不变。
- **状态**（分配记录、映射条目、钉住、隔离，乙增量与分类的输入）：每次发布都变（搬走的单元记录释放、映射条目换到目的地）。

所以「前一批发布改了这一版，后一批的输入怎么办」对甲不成问题，对乙增量与丙成问题：
- 甲（字节）：一次交、各批共用结果，字节不变，是对的；代价是**把「每批重验字节」改成「窗口开头验一次字节、每批只重扫状态」**。这改了上面 `:2975` 那句判决的一半（字节那一半）。安全性靠发布路径里的释放前读盘核（`crates/singlefs-core/src/transaction.rs:5084`）在释放前一刻读两份副本再核，它不被改。
- 乙增量（头解析、映射反查）：输入是映射条目，每批变，必须每批重上传，**攒不起来**，所以不放 GPU（与 G1 推荐一致）。
- 丙：只在签发时调一次，用当时那一版，没有陈旧问题。

### G2.5　「一批之内不换模式」在两种形态下怎么落

`.claude/rules/fs-design.md` 五条硬要求第 5 条：「分支变量不许在操作中途改变」；D26 已定项 13 第 1 条：「从下一批开始生效；一批之内不换」。落法：每批的入口（`advance_compaction_by_one_publish`，`crates/singlefs-core/src/mounted_session.rs:2989`）读一次当前模式，存进这一批的局部量，这一批的扫描、计划、发布全用这个局部量，不再读共享的那份。

- 乙-1：窗口 = 一条意图的 R。切换发生在窗口中间时，窗口里已算好的 GPU 校验结果**留在 GPU 模式的批里用**：切到 CPU 的第一批丢掉缓存、CPU 自己算（CPU 模式不吃 GPU 的结果，否则「CPU 模式完整工作、语义相同」与「运行时看得出走了哪条」都被搅混）；切回 GPU 时对剩下的单元重新交一次。
- 乙-2：窗口跨意图，切换时整个预取丢掉（它们是纯函数结果，丢了只是白算，没有语义后果）。
- 丙：宽 R 在签发那一刻定死，模式是每批读；与乙-1 同。
- 窗口中间 GPU 出错：第 4 条「正在算的那一批放弃（意图留在盘上，目标态单调）、那个后台任务停下并报错」，窗口里还没发布的各批同样放弃，什么都没写出去。

### G2.6　推荐与岔路

**推荐：乙-1，且把甲的字节验证改成窗口开头一次**（见 G2.4）。依据：它是 D26 已定项 13 第 3 条原话的形态；不改盘上格式（不碰 `crates/singlefs-core/src/logged_operations.rs:303`）；limit 对它只是个起卡门槛，不会出现「limit 太小攒不起来」。**要明说的代价**：它压不下固定开销（φ ≥ 0.89），所以它是否净赚完全看 G1-a 的 c_rt。

| # | 候选（各自的定义） | 翻面观测 | 够判条件 |
|---|---|---|---|
| G2-a | 乙-1 意图内一次交／乙-2 跨意图预取 W 个候选段／丙 宽 R（盘上槽数字段放开，两种模式共用） | 一次提交的主机 CPU 拟合成「固定部分 c0 + 每字节 c1·B」：c0 ≈ 0（等结果时线程睡眠，只有拷贝随字节涨）则乙-1 够，不用攒到 70 MiB；c0 大到压过乙-1 的上界（每条意图 0.76–4.6 ms）才轮到乙-2 或丙 | G1-a ① 的四个提交大小量齐，并多量一个 ≥ 16 MiB 的提交大小看 c1 |
| G2-b | 重扫：每批字节重验（今天）／窗口开头验一次字节、每批只重扫状态 | 在 R 内一个单元的字节于两批之间翻一位的注入下，两条路的结局表（前者在下一批的扫描里落「不一致」类、后者在释放前读盘核落隔离）是否都合 D19 已定项 5 的硬规则 | 注入跑过，两条路各自的结局逐项写出，用户看后定取哪个 |
| G2-c | 显存总量与空闲量从哪来：起 `nvidia-smi` 子进程（现有做法，NVIDIA 专用）／NVML 动态加载／Vulkan 扩展直接读预算（绕开 wgpu）／只用 `request_device` 的 OOM 报错当信号（不给总量） | 第 4 条的「超过显存总量拒绝」是否必须在挂载时、在没起设备的情况下判（需要总量）；还是允许起设备失败后再拒 | 用户定第 4 条的判法时机；候选里每一个在本机两张卡上读到的总量与空闲量与 `nvidia-smi` 报的逐项一致 |

## G3　多张卡怎么分活

### G3.1　今天的样子

- 产品里没有卡的概念：`grep -rn wgpu crates/singlefs-core/Cargo.toml` 零命中（核心层的依赖只有 `libc`、`singlefs-format`、`singlefs-replay-prefix`，`crates/singlefs-core/Cargo.toml` 的 `[dependencies]`）。
- 里程碑四七节写明：「产品里多卡的先后不读仓根 `multi-host.env`（那是测试装置的配置），按什么排要七·2 定」（`.claude/kb/milestone/04-fourth-txn.md:149`）。
- checker 档的做法是测试装置，不是产品：优先级列表 `GPU_CARDS_BY_PRIORITY`，每项 `<local|peer>:<nvidia-smi 序号>:<显存额度>`，「写在前面的优先级高，任务先装它、装满了才给后面的」（`crates/singlefs-checker-tier/src/gpu_card_configuration.rs:70`，解析在 `:74` `parse_cards_by_priority`）；卡的发现是 Vulkan 适配器加 `nvidia-smi` 按 PCI 总线号对上，只认 NVIDIA 独显（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:453` `if information.vendor != NVIDIA_VENDOR_IDENTIFIER`、`:454` `|| information.device_type != wgpu::DeviceType::DiscreteGpu`）。
- D26 已定项 13 第 2 条：「挂载参数 `background_gpu_devices` 是一张卡的列表（可以点几张）；不给就用本机全部显卡」（`.claude/kb/decisions/26-后台整理与放置回收.md:237`）；第 3 条的显存上限列表「与参与的卡逐个对应（给了 `background_gpu_devices` 就对它的列表，没给就对本机全部显卡的列表），不补齐」（`:238`）。所以**不给时的默认次序是用户可见的接口**：用户要照它写 `background_gpu_memory_limit` 的数。

### G3.2　三件要定的事与候选

**(i) 一张卡在列表里怎么写。**

| 候选 | 依据与问题 |
|---|---|
| PCI 总线号字符串（形如 `0000:01:00.0`） | wgpu 的 `AdapterInfo.device_pci_bus_id`：Vulkan 下取自 `VkPhysicalDevicePCIBusInfoPropertiesEXT`，「if available」（`wgpu-types-30.0.1/src/adapter.rs:173-179`，仓外）。插槽不变就不变；扩展缺失时没有 |
| `nvidia-smi` 序号 | checker 档现用（`crates/singlefs-checker-tier/src/gpu_card_configuration.rs:61`「`nvidia-smi` 的序号」），NVIDIA 专用、要起子进程 |
| wgpu 枚举序号 | `instance.enumerate_adapters` 的次序（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:425`）；次序由 Vulkan 加载器与 ICD 定，换驱动或加卡会不会变我没验（未核实） |

**(ii) 不给列表时「全部」指哪些卡。**

| 候选 | 依据与问题 |
|---|---|
| 全部 Vulkan 适配器 | 含集显与软件渲染（`DeviceType::Cpu`）：软件渲染跑在主机 CPU 上，与 GPU 模式「少占主机 CPU 时间」的目的相反 |
| 全部独显（`DiscreteGpu`），任何厂商 | WGSL 校验核只在 NVIDIA 卡上验过：E163 页第 31 行 M1 判「能」、E185 第三段 `gpu_card` 两行（`research/results/e185-gc-batch-wall-clock-split-gpu-submission-r1-2026-10-22.out:37` 的 `card_name=NVIDIA_GeForce_RTX_5090`、`:119` 的 `card_name=NVIDIA_GeForce_RTX_5060_Ti`）；别的厂商的编译器与整数行为没验 |
| 全部 NVIDIA 独显（checker 档现状） | 与已验范围同，要认厂商号（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:78` `NVIDIA_VENDOR_IDENTIFIER: u32 = 0x10DE`） |

另外一个条款没写的格：「全部」里有一张起不来设备（E163 页第 33 行：另一台一张 RTX 5080 空闲显存 141 MiB 时 `request_device` 交回 `Device(OutOfMemory)`），是拒绝挂载（第 4 条对点名的卡这样写）还是跳过它？拒绝则一台上另有进程占着一张卡就挂不上；跳过则「不降级」打了折扣，而且显存上限列表的个数对的是「全部显卡的列表」，跳过后个数对不上。列进岔路 G3-b。

**(iii) 怎么分活。** 一个窗口（乙-1，G2）至多 32 个数据单元、1048576 字节，没有可以切给多张卡的体量。候选：

- S1　全交第一张卡（其余卡验过能起、不派活）；
- S2　按窗口轮转：窗口 w 交第 (w mod n) 张卡，两份核同一次提交；
- S3　按核分卡：核 A 放第 (w mod n) 张、核 B 放第 ((w+1) mod n) 张，并行，n = 1 时同一张卡；
- S4　按单元切：一个窗口的单元对半分给两张卡，每张卡跑两份核。

用 E185 第三段拟合的 a、b（两张卡）算一个满 R 窗口的挂钟（`g3_wall.py`，原样输出）：

```text
$ python3 -I $D/tools/g3_wall.py
卡 a 比 2.293  b 比 3.075  链路峰值比 4.000（卡1 ÷ 卡0）
分法                                                  提交次数  挂钟(ms)
S1 全交卡0，两份核同一次提交                                        1      5.355
S1 全交卡1，两份核同一次提交                                        1     12.955
S2 按窗口轮转（两卡平均），两份核同一次提交                                 1      9.155
S3 核A卡0、核B卡1 并行（取慢的）                                    2     11.628
S4 按单元对半切两卡，每卡两份核同一次提交（取慢的）                             2     11.628
```

两张卡本身不同：a 差 2.29 倍、b 差 3.07 倍、链路峰值差 4.0 倍（`gpu_card` 行 `link_peak_bytes_per_second=63015384615` 对 `15753846154`）。读法：S4 把单元切给两张卡，每张卡还是付自己的固定开销，且两次提交的主机 CPU 翻倍，没有任何一个字节量下它赢（提交固定开销大于每字节开销：a 是 4.49 ms 对 b·B = 0.43 ms）；S3 的挂钟被慢卡定死（11.63 ms），主机 CPU 也是两次提交；S1、S2 只有一次提交。**按「省主机 CPU 时间」的尺，提交次数少的赢**，所以 S1、S2 优先于 S3、S4；S3 的价值只在核 A、核 B 分在两块物理显存里，见 G4 的独立性判据。

### G3.3　「一张卡与几张卡判定相同」怎么保证

判定函数是：每个单元的 (核 A 的 CRC，核 B 的 CRC，CPU 参照) → 确认／不确定／交 CPU 判，再加连续 N 轮的计数（G4）。保证分三层：

1. **构造**：两份核都是整数运算的纯函数（输入只有单元字节，WGSL 里没有浮点、没有原子累加到共享槽、没有读未初始化内存），每个单元的两份核结果只取决于这个单元的字节。每个 (单元, 核) 一定被算到一次，**与卡数无关**；把 (单元, 核) 派给哪张卡是纯函数（上面 S1–S4 之一），不进判定。N 轮的计数以单元的槽为键、住在会话里，**不以卡为键**。
2. **测试**：同一组输入在卡集合 {卡 0}、{卡 1}、{卡 0, 卡 1} 上各跑一遍，逐单元比三份判定向量，必须逐项相同（七·3 的验收已经这样写：「只点一张卡与点几张卡各跑一遍，判定逐项相同」，`.claude/kb/milestone/04-fourth-txn.md:145`）。
3. **注入**：往核 A 的某个单元上塞一个软件注入的错（在核里改一位，不是改卡），三个卡集合上该单元都必须得「不确定」，下一轮去掉注入后都得「确认」。

**限度要写明：** 「判定相同」成立的是无硬件故障、或故障是软件层注入在核里的情形。一块真卡自己出错（显存位翻转、驱动 bug）时，一张卡的配置里两份核同在这块卡上，可能两份一起错而「确认」；两张卡且核 A、核 B 分卡（S3）时只有一份错而「不确定」。这两种配置对同一次真故障的结局可以不同，**这是 D26 已定项 13 射程里已经认下的残余**（`.claude/kb/decisions/26-后台整理与放置回收.md:247`：「抓得到只有一份出的错……抓不到两份一起错的」），不是保证能补的。

### G3.4　推荐与岔路

**推荐**：卡的身份用 PCI 总线号字符串；不给列表时「全部」= 本机 Vulkan 独显中已验过的厂商（今天只有 NVIDIA），次序按 PCI 总线号升序；分活用 S2（按窗口轮转、两份核同一次提交）。依据：S2 满足「用全部」，一次提交，不让用户去猜次序（总线号升序是稳定的、用户从 `lspci` 就能对上）；S1 把别的卡闲置，与「不设置的话用全部」不合。

| # | 候选（各自的定义） | 翻面观测 | 够判条件 |
|---|---|---|---|
| G3-a | 分活：S1／S2／S3／S4（上文定义） | 一次提交的主机 CPU c_rt（G1-a）：S2 比 S3 每窗口少一次提交；若 G4-d 判定「两份核必须分在两块物理显存」，S3 就是唯一合条款的分法 | G1-a 的 c_rt 量到；G4-d 用户定 |
| G3-b | 「全部」里有卡起不来设备：拒绝挂载／跳过并报告（运行时可见）且显存上限列表按「实际参与的卡」对齐 | 同一台机器上另有进程占卡时，用户期望是挂不上还是照常挂载 | 用户定第 4 条射程；至少在一台有一张卡被占的机器上把两种行为各走一遍 |
| G3-c | 「全部」的厂商范围：NVIDIA 独显／任何独显／含集显 | 别的厂商的卡上 WGSL 校验核与 CPU 参照在 E163 同一批输入（2560 个单元）上逐项相等 | 在一张非 NVIDIA 的独显上跑过 E163 的同一批输入且逐项相等；本机没有这样的卡，需要用户给 |

## G4　两份 GPU 核比对的形态与 N

### G4.1　条款与今天的样子

- 条款原文：「两份独立写的 GPU 核（不共用核函数）对同一批输入各算一次，逐条比两份的结果。两次相同的条目算确认，交 CPU 改位图（CPU 只改位图，不再抽样复核）；两次不同的条目写明状态不确定，这一批不回收、不搬，留到下一轮重算。同一个条目连续 N 轮都不确定，交 CPU 那一份实现单独判一次，按 CPU 的判定处理」（`.claude/kb/decisions/26-后台整理与放置回收.md:243`）。
- 条款自己列的残余：「第 8 条的两份核抓得到只有一份出的错（一份核写错、没有 ECC 时的位翻转、不确定的竞争），抓不到两份一起错的：两份共用的那几样（同一个驱动与着色器编译器、同一份输入表的准备、对规则的同一种误读）出错时两份照样一致，那一类靠 GPU 与 CPU 两份实现在测试里的对拍和崩溃点重放抓」（同文件 `:247`）。
- D13 已定项 5：「校验和各用一份独立实现」（`.claude/kb/decisions/13-验证路线.md:156`），且「CRC32C 各写一份的前提是参数钉死（多项式、初值、输入输出反转、异或输出）：参数不钉死，两份独立实现会在健康镜像上给出不同结果，那是假红而不是独立性」（同文件 `:160`）。
- 仓里已有的 GPU 上算单元校验和的代码都是**检查装置**：checker 档的 `CRC32C_SHADER_SOURCE`（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:80`，一个调用按位算一段，反射多项式 0x82F63B78 的右移写法，`:87`、`:106`–`:109`）与 E163 装置（页第 9–10 行：臂 C 按位参照实现、臂 G「WGSL 计算着色器，逐单元一个调用，按位算同一套 CRC-32C」）。它们是 D13 已定项 5 说的 checker 一侧，产品的核不能照抄（见 G7）。
- 单元校验和在产品 CPU 路径里是**两道**：头校验和罩 `[0, header_end)` 且校验和字段按 0 参与（`crates/singlefs-core/src/unit.rs:391`），载荷 CRC 罩 `[header_end, 末尾)` 存在固定偏移（`:403`；数据单元 `parse_data_unit` 在 `:581` 起，头 105 字节、载荷 CRC 偏移 101）。E185 的甲量的是整单元一道 CRC-32C（E185 页第 281 行「甲 每个被读单元整单元 CRC-32C」）；产品的 GPU 核要算的是这两道。两道合起来覆盖单元的全部 32768 字节（头 105 加载荷 32663），算量与整单元一道相同，差只在切成两段、多一次头校验和字段清零的拷贝；G1 的上界没有把两者分开。

### G4.2　「独立」的判据：能共用什么、不许共用什么

| 项 | 判法 | 依据 |
|---|---|---|
| 着色器源文本里的函数与常量 | **不许共用**：两份源文本的函数名集合与常量名集合交集为空；多项式的写法不同（一份反射 0x82F63B78 右移，一份正规 0x1EDC6F41 左移加输入输出位反转） | D26 已定项 13 第 8 条「不共用核函数」；多项式写法不同是 E163 页第 74 行那条变异（臂 C 与着色器同改成同一个错误多项式，互比测试全绿）想堵的同形错 |
| 算法 | **不许同一种**：一份按位（每字节 8 步移位异或）、一份按字节查 256 项表；表由着色器自己在 GPU 上算出，不由主机拷入（改由主机生成，表生成代码就成了又一处要钉的代码） | 同上 |
| 与 checker 档、E163 装置、`singlefs-core` 的 `crc32_castagnoli`（`crates/singlefs-core/src/checksum.rs:43`，slicing-by-8 软件表） | **不许共用**：产品的两份核都新写，不拷 `crates/singlefs-checker-tier/src/gpu_unit_checks.rs:80` 那份 | D13 已定项 5（`.claude/kb/decisions/13-验证路线.md:156`）：checker 与实现之间只共享生成的常量 |
| 结果缓冲 | **不许共用**：每份核一块结果缓冲 | 共用则一份写坏另一份的结果两份一起错 |
| **输入的显存拷贝** | **建议不许共用**：每份核各自一块输入缓冲、各上传一次 | 条款的残余清单里写「没有 ECC 时的位翻转」是「抓得到只有一份出的错」之一；两份核读同一块缓冲时，这块缓冲里的翻转两份一起看到、一起算错，不算「只有一份」。这一项条款没明写，是我从那句话推的，列进岔路 G4-d |
| 输入表的准备（单元怎么切成头段与载荷段、偏移与长度） | **可以共用**（条款已认） | `:247`「同一份输入表的准备」 |
| 驱动与着色器编译器（naga → SPIR-V → NVIDIA 驱动） | **可以共用**（不可分） | 同上 |
| 每次起跑前的绝对值锚点 | **必须有**：每份核在每次起卡后先算一组固定输入的已知值（至少 "123456789" → 0xE3069283，空串 → 0，32768 个 0x00，32768 个 0xFF），不对就不起跑 | E163 页第 107 行：互比类测试全绿，「只有对 `EXPECTED_ANCHOR_VALUES` 的绝对值检查抓到——这条检查不可少」；`crates/singlefs-core/src/checksum.rs:94` 起的 CPU 单测同一个公开值 |

「对规则的同一种误读」里最容易共有的是**区段怎么切**（头罩到哪、校验和字段在哪、数据单元与索引节点的长度）：这一份准备代码两份核共用（条款认了），所以它的对错由 CPU 参照 `parse_data_unit`（`crates/singlefs-core/src/unit.rs:581`）与对拍测试管：测试里对同一批单元，GPU 路径给出的「头校验和是否对、载荷 CRC 是否对」必须与 `parse_data_unit` 逐项相同（七·3 的验收「两条路径在同一组输入上逐项相同」，`.claude/kb/milestone/04-fourth-txn.md:145`）。

### G4.3　一张卡与几张卡

一张卡：两份核在同一张卡上，各自的输入缓冲与结果缓冲，同一次提交里排两个计算通道、读回一次（挂钟 a + 2·b·B，`g3_wall.py` 第一行）。几张卡：G3 的 S2（两份核同卡）或 S3（核 A、核 B 分卡）。不要求多张卡：S2 在 n = 1 时就是一张卡的形态，S3 在 n = 1 时退成同卡。

### G4.4　比对放在 GPU 上还是 CPU 上

**CPU。** 两份核各交每个单元的 (头 CRC，载荷 CRC) 两个 u32；CPU 做三件比较：核 A 与核 B 逐项相等？各自与单元头里存着的值相等？不等则不确定。依据：

1. 量级：一个窗口至多 32 个单元，结果 32 × 2 × 4 × 2 = 512 字节，比较是几十次整数比较，远小于 CPU 现在为一个单元算一次 CRC 的时间（10.3–11.4 µs，E185 页第 165 行 Q8 的 2.875–3.189 GB/s 对 32768 字节）。这一项算进第 7 条的「两份结果的比对」，可以忽略。
2. 信任根：放 GPU 就要第三个核做比对。比对核写坏（恒判相等）时，所有条目都「确认」，而「确认的交 CPU 改位图，CPU 只改位图，不再抽样复核」（`:243`）——没有任何东西再抓它。放 CPU 则比对是普通 Rust 代码，进快档单测，进变异。
3. GPU 一侧只多一次提交内的小核，不省提交次数（读回是同一次）。

### G4.5　N 取几

「轮」的读法条款没写透：「两次不同的条目写明状态不确定，这一批不回收、不搬，留到下一轮重算」，「这一批」是整批不搬，还是只有那几个条目不搬、批里其余照搬？读法决定一轮重算的代价 C_w（见岔路 G4-b）。N 的算式不依赖读法，只依赖 C_w 与 CPU 判一个单元的代价 C_j：

```text
E(N) = C_w · Σ_{k=0}^{N-2} r^k  +  C_j · r^{N-1}      （N ≥ 2；条款要求至少重算一次，N = 1 与「留到下一轮」矛盾）
r = 重算后仍不确定的概率（0 = 纯瞬时错；1 = 确定性错）
C_j = 32768 B ÷ 2.875e9 B/s = 11.396 µs（慢的一端）
```

```text
$ python3 -I $D/tools/g4_n_cost.py
C_j = 11.396 µs
C_w=0.33 ms（下界）
  r=0.0  N=2:    0.3300 ms   N=3:    0.3300 ms   N=4:    0.3300 ms   N=8:    0.3300 ms
  r=0.01 N=2:    0.3301 ms   N=3:    0.3333 ms   N=4:    0.3333 ms   N=8:    0.3333 ms
  r=0.5  N=2:    0.3357 ms   N=3:    0.4978 ms   N=4:    0.5789 ms   N=8:    0.6549 ms
  r=1.0  N=2:    0.3414 ms   N=3:    0.6714 ms   N=4:    1.0014 ms   N=8:    2.3214 ms
C_w=13.434 ms（P=1024 一批）
  r=0.0  N=2:   13.4340 ms   N=3:   13.4340 ms   N=4:   13.4340 ms   N=8:   13.4340 ms
  r=0.01 N=2:   13.4341 ms   N=3:   13.5683 ms   N=4:   13.5697 ms   N=8:   13.5697 ms
  r=0.5  N=2:   13.4397 ms   N=3:   20.1538 ms   N=4:   23.5109 ms   N=8:   26.6582 ms
  r=1.0  N=2:   13.4454 ms   N=3:   26.8794 ms   N=4:   40.3134 ms   N=8:   94.0494 ms
C_w=111.090 ms（P=8192 一批）
  r=0.0  N=2:  111.0900 ms   N=3:  111.0900 ms   N=4:  111.0900 ms   N=8:  111.0900 ms
  r=0.01 N=2:  111.0901 ms   N=3:  112.2009 ms   N=4:  112.2120 ms   N=8:  112.2121 ms
  r=0.5  N=2:  111.0957 ms   N=3:  166.6378 ms   N=4:  194.4089 ms   N=8:  220.4443 ms
  r=1.0  N=2:  111.1014 ms   N=3:  222.1914 ms   N=4:  333.2814 ms   N=8:  777.6414 ms
```

读法：对 r ∈ [0, 1] 的每一个值，只要 C_w > C_j（下界 0.33 ms 是 C_j 的 29 倍），E(N) 随 N 不减（r = 0 时持平，r > 0 时严格增），**N = 2 最小**：多等一轮要多付一轮 C_w，换来的只是少付 C_j。N 变大的理由不在 CPU 时间，而在别处：让 GPU 一侧的瞬时错有更多机会自己消掉。E163 的 2560 个单元与 E185 第三段 32 行 `crcs_agree=true` 里没有一次不一致（E163 页第 26 行 `MERGE mismatched_count=0`；E185 页第 376 行「第三段逐段对拍 CRC 的 32 行 `crcs_agree=true`」），观测到的瞬时错率是 0 的量级，r 没有数据可估；所以 N 大于 2 缺依据，**推荐 N = 2**。

状态存放：每个单元的连续不确定轮数住会话内存里，以单元的槽（设备加槽号）为键，确认、搬走、挂载重来都清零，不写盘（D26 已定项 13 第 1 条「它只活在内存里」，`:236`）。

### G4.6　推荐与岔路

**推荐**：两份核算法不同（反射按位 × 正规按字节查表）、源文本无共用标识符、各自一块输入与结果缓冲、每次起卡前算绝对值锚点；比对在 CPU；N = 2。

| # | 候选（各自的定义） | 翻面观测 | 够判条件 |
|---|---|---|---|
| G4-a | 比对放哪：CPU／GPU 上第三个小核 | GPU 比对相对 CPU 比对节省的主机 CPU 时间（算式见上，CPU 比对是几十次比较）；以及往比对核里塞「恒判相等」的故障，现有检查能不能抓到 | 量到 CPU 比对的实际耗时；塞故障的测试写出来且红 |
| G4-b | 「这一批不回收、不搬」的读法：A 整批不搬／B 只那几个条目不搬，批里其余照搬／C 批内就地对那几个条目重算再判 | 一轮重算的代价 C_w：A 付一整步（至多一批 CPU，13.4 ms 或 111.1 ms 量级）；B 与 C 只付这几个单元的 GPU 往返；意图能不能在有条目一直不确定时完成 | 用户定读法；三种读法下 C_w 各量一次（G1-a 的 c_rt 量到之后） |
| G4-c | N：2／3／更大 | 真卡上单元级重算后仍不一致的概率 r（现有数据全是 0 次不一致，没有分母足够大的估计）；确定性不一致（某类字节内容让一份核总算错）是否出现 | 在两张卡上跑 ≥ 10⁶ 个单元次的重算，写明零次时的上界（三次法则 ≈ 3/10⁶）；用户定 N 是按 CPU 时间最小（N = 2）还是按别的目的 |
| G4-d | 输入显存拷贝：两份核共用一块（条款认下的残余）／各一块（我推的建议）／各一块且分在两张卡（S3，n ≥ 2 时） | 真卡显存位翻转的实际发生率（本机没有数据，两张卡都是消费级）；各一块的主机代价（多一次上传，≤ 1 MiB） | 用户定条款「没有 ECC 时的位翻转」是不是承诺要抓；若是，则取后两者之一 |

## G5　控制接口（挂着切换）的形态

### G5.1　今天的样子

- 整理没有后台线程：`MountedSession::run_compaction`（`crates/singlefs-core/src/mounted_session.rs:1956`）持 `&mut self` 同步推到停机原因或发布预算用完；`grep -n 'std::thread\|thread::spawn\|Mutex\|Arc<\|AtomicBool\|mpsc' crates/singlefs-core/src/mounted_session.rs crates/singlefs-core/src/mount.rs` 我现跑，零命中。`grep -rln fsync crates/singlefs-core/src` 只有 `block_device.rs` 与 `mount.rs`，`mount.rs` 里唯一一处是 `:9560` 的注释，没有「前台 fsync 在等」这个状态（G6 要用到）。
- 挂载入口是一串叠加一个只供测试开关的重载：`mount_writable`（`crates/singlefs-core/src/mount.rs:8043`）→ `mount_writable_with_space_admission`（`:8055`）→ `mount_writable_with_test_only_switches` → `mount_writable_with_compaction_switches`（`:8523`）→ …，产品路径恒用 `CompactionSwitches::PRODUCT`（`compaction_plan.rs`）。会话 `MountedSession`（`crates/singlefs-core/src/mounted_session.rs:83`）的整理状态在字段 `compaction: CompactionOfThisMount`（`:95`；`crates/singlefs-core/src/compaction_plan.rs:616`），只活在内存。
- 封闭枚举加成员、`match` 不写通配臂，仓里已有同一做法：「logged_ops 条目的种类。封闭集合：加一种意图就加一个成员，`match` 不写通配臂」（`crates/singlefs-core/src/logged_operations.rs:38`）。`compaction_plan.rs` 里的批大小、起跑强制等七个只供测试的开关也是封闭枚举。
- 里程碑七已写下留口范围：「留口只到三处：任务种类与后端两个枚举、控制接口、内存里的观测计数……不写占位代码」（`.claude/kb/milestone/07-seventh-txn.md:83`），且「控制接口只有一个开关，不按任务种类寻址」（`:78`）、「观测计数按「任务种类 × 后端」记」（`:81`）。

### G5.2　候选

**形态一：开关怎么进来。**

| 候选 | 定义 | 「下一批生效」的边界 |
|---|---|---|
| A　会话方法 | `MountedSession::switch_background_compute(&mut self, …)` 持 `&mut self`，只能在两次 `run_compaction` 之间调 | 下一次 `run_compaction` 的第一批；一次运行内部不切（比「下一批」严） |
| B　共享句柄 | 会话里放一个可克隆的控制句柄（原子单元或互斥量），别的线程能写；整理每批顶部读一次 | 每批顶部：`advance_compaction_by_one_publish`（`crates/singlefs-core/src/mounted_session.rs:2989`）进门读一次，存进这一批的局部量，这一批全用它 |
| C　每批前回调 | `run_compaction` 多收一个闭包，每批前问一次要不要换 | 同 B，但切换由调用方在回调里定 |

三者读到的位置相同——**批的入口**。代码里这个边界就是 `run_compaction_on_with_the_region_check_of_the_intent_deletion` 的 `loop {`（`crates/singlefs-core/src/mounted_session.rs:2705`）每转一圈调用的 `advance_compaction_by_one_publish`（`:2989`）的第一行：此前 `compaction.take_the_quarantines_noted_by_the_publishes(…)`（`:3001`）与整版克隆（`:3005`）还没做，选段、扫描（`:3026` 起）、计划、发布都在后面，全部读那一个局部量，所以「一批之内不换」靠把它当参数往下传，不靠锁。现状里批大小是**整趟运行开头读一次**（`:2654` 的 `let batch_units = match switches.batch_size`），计算后端不能照抄这个读法（那样一趟之内切不动）。

**形态二：GPU 变体里放什么。** 核心层不依赖 GPU 包（G7），所以 `ComputeBackend::Gpu` 里装的是核心层定义的 trait 对象，由 GPU 包实现。

| 候选 | 定义 |
|---|---|
| A　trait 对象 | 核心层定 `trait UnitChecksumOffload`（交一组单元字节，回每个单元的两份核结果与错误），`ComputeBackend::Gpu(Box<dyn UnitChecksumOffload>)` |
| B　泛型参数 | `MountedSession<Backend>`；类型沾满整个会话与所有调用方，不选 |
| C　函数指针表 | 核心层定一组 `fn` 指针，GPU 包填；不比 A 简单 |

**形态三：「挂载或切换那一刻」拒绝（D26 已定项 13 第 4 条）落在哪。** 点名的卡不存在、驱动不对、显存上限个数与卡数不等、超过显存总量，这些都要摸 GPU，核心层摸不到：落在 GPU 包的 `open(configuration) -> Result<handle, OpenRefused>`；调用方先 `open`，成功才把 `ComputeBackend::Gpu(handle)` 交给挂载或切换。核心层的「拒绝」只剩一种：交来的后端与任务对不上。所以「拒绝挂载」「拒绝切换」都是 `open` 失败后调用方没有调挂载或切换。**挂载时的整理续做**（`MountOutput::compaction_continuation`，`crates/singlefs-core/src/mount.rs:8523` 文档）会在挂载里跑批，所以后端必须在续做之前就位。

### G5.3　推荐

- 第一版取 **A（会话方法）**：整理今天在调用线程上同步跑，没有别的线程可写开关；`&mut self` 让「一批之内不换」由借用规则保证，不需要锁。边界照上面写，批入口读一次。若运行内部要切，B 是同一边界上的加法（把局部量的来源从字段换成句柄）。
- 类型（核心层，新模块，只活在内存）：`BackgroundTask`（今天一个成员：GC 搬迁）、`ComputeBackend { Cpu, Gpu(…) }`；`fn effective_backend(task: BackgroundTask, configured: &ComputeBackend) -> EffectiveBackend`，里面是 `match` 穷举 (任务, 后端) 的每一对，不写通配臂；没有 GPU 路径的任务那一臂交 `Cpu`（D26 已定项 13 第 1 条「没有 GPU 路径的照旧在 CPU 上算」）。
- 里程碑七接上打包搬迁：加 `BackgroundTask::PackedRelocation` 一个成员 → 编译器在 `effective_backend`、观测计数的下标、任何按任务种类 `match` 的地方逼着补；打包任务自己的 CPU 路径与 GPU 路径住它自己的模块，在它自己的批入口调 `effective_backend(PackedRelocation, …)` 读一次。开关、挂载参数、`open` 都不改（里程碑七 `:77`–`:78`）。
- 观测：`CompactionObservations`（`crates/singlefs-core/src/compaction_plan.rs:489`）加按「任务 × 后端」的批数，和 D26 已定项 13 第 8 条要的每批「确认、不确定、交 CPU 判」三个数；停机原因加一个成员（GPU 出错，`CompactionRunStop`，`crates/singlefs-core/src/compaction_plan.rs:414`）。
- 只供测试的开关：照 `CompactionSwitches` 的做法加「强制后端」「强制 GPU 在第 k 批出错」，让每条分支能被测试进入（`.claude/rules/fs-design.md` 五条硬要求第 2 条）。
- GPU 出错后「那个后台任务停下并报错，不换回 CPU」：任务的停下状态住会话内存，切换（到 `cpu` 或重新 `open` 的 `gpu`）清掉它；这一格条款没写停下状态在哪里清，是我补的读法，列进岔路。

| # | 候选（各自的定义） | 翻面观测 | 够判条件 |
|---|---|---|---|
| G5-a | 开关形态：A 会话方法（运行之间切）／B 共享句柄（运行中另一线程切）／C 每批前回调 | 整理是否要在独立线程上跑、调用方是否要在一次运行进行中切。今天 `grep` 线程相关零命中；里程碑十 FUSE 的线程模型还没有（`.claude/kb/milestone/04-fourth-txn.md:137` 写「命令行要到里程碑十的 FUSE」） | 里程碑十的线程模型有了，或用户明说要在运行内切 |
| G5-b | GPU 出错后的停下状态：A 会话里按任务记一个停下标志，切换清掉／B 不记，只靠 `CompactionRunStop` 的停机原因，下一次运行再试一次 | B 下 GPU 坏了每次起跑都重试一次、每次报错，是不是用户要的「停下并报错」 | 用户定第 4 条「那个后台任务停下」的读法；两种各走一遍 GPU 出错注入的结局 |
| G5-c | 挂载时的整理续做用哪个后端：用挂载参数给的／固定 CPU 直到挂载完成 | 续做会在挂载里跑批；挂载参数为 `gpu` 时续做是否也要走 GPU | 用户定；两种读法下续做的观测各写出来 |

## G6　三个默认值

条款：「什么时候起跑：每次发布之后按已定项 1 的三水位查一次；另有定时兜底，每隔一段时间再查一次，间隔是挂载参数。两种模式起跑规则相同」（`.claude/kb/decisions/26-后台整理与放置回收.md:240`）；「起跑之后的节奏：GC 占盘写带宽有一个上限（挂载参数）；前台有 fsync 在等时不发下一批」（`:241`）；显存上限「不给就不设上限，后台任务跑的时候卡上能用多少用多少」（`:238`）。射程里明写不管默认值：「定时兜底的间隔、写带宽上限的默认值（等 C412（整理的挂钟代价没量过） 与 E10（老化：结构抗老化 vs 事后整理））」（`:247`）。

### G6.1　先摆出来的三个现状

1. **没有线程，也没有「前台 fsync 在等」这个状态。** `crates/singlefs-core/src/mounted_session.rs` 与 `mount.rs` 里线程原语零命中（G5.1 的 grep），`fsync` 在核心层只出现在 `block_device.rs` 与 `crates/singlefs-core/src/mount.rs:9560` 的一条注释。里程碑六才处理并行提交，且把「后台事务会不会堵前台 fsync」留成里程碑六要答的问：「里程碑四……按串行提交设计，留下两问：……第 3 问（后台事务——删除批次、GC、删快照——在并行提交下怎么取号、会不会堵前台 fsync）」（`.claude/kb/milestone/06-sixth-txn.md:85`）。里程碑四的 GC 与前台同在一个 `&mut self` 里，前台 fsync 不可能在 GC 一批的中途「在等」：第 6 条的让路在串行设计下是空成立，真起作用要等里程碑六。
2. **水位恒 0，批大小未定：今天产品路径不会起跑。** `crates/singlefs-core/src/system_configuration.rs:415` 的 `compaction_watermarks` 三条恒取内置默认 0（0 = 关）；`CompactionBatchSize::NotDecidedByTheClauses`（`crates/singlefs-core/src/compaction_plan.rs:299`）让 `run_compaction` 在签任何意图之前停下（`crates/singlefs-core/src/mounted_session.rs:2654`）；`compaction_starts_after_a_user_change` 对 `FollowTheWatermarks` 恒返回 `false`（`crates/singlefs-core/src/compaction_plan.rs:820`）。定时兜底间隔设多少，今天都不会有任何一次真起跑。
3. **整理的发布预算 `B_gc` 是个硬刹车，兜底定时器碰得到它。** `publish_budget_of_a_compaction_run`（`crates/singlefs-core/src/compaction_plan.rs:808`）：`3S − 4 − 已经过去的发布数`，至少 1，S 是每区根槽数；mkfs 时 `ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM` = 8（`crates/singlefs-format/src/lib.rs:309`），所以一次用户改动之后整理至多发 20 次。「已经过去的发布数」只在会话发成一次**用户改动**时归零（`crates/singlefs-core/src/mounted_session.rs:1926` `compaction.publishes_since_the_last_user_change = 0`）。判决里写这条的理由是「GC 发布不改变用户可见状态，连发会把回退候选集挤成 1 个状态」（`research/prompts/m4-gc-r1-main-verification.md:39`）。后果：**文件系统闲置时，定时兜底每一次起跑都只能在剩下的预算里发，预算用完（`PublishBudgetSpent`）之后再怎么定时也不会再发布，直到下一次用户改动。** 兜底的目的（发布停了、债还在时接着清）在今天的预算规则下做不到。这是一格条款之间的冲突：D26 已定项 13 第 5 条与 G8 判决的 `B_gc`。

### G6.2　写带宽上限从什么量取

E185 第 4 行的读数（页第 250 行）：Q16 `maximum_ratio=0.015977 judgement=single_run_enough`（产物 `research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out:1958`：`E7RESULT name=q16 maximum_ratio=0.015977 judgement=single_run_enough fourth_segment=not_run_after_enough_to_judge`）。Q14 是 GC 单跑的写带宽，7.23–50.5 MB/s（页第 158 行：`write_bytes_per_second_median` 7229989.048161–50509463.337147）；Q15 是同一块盘的顺序写带宽 3161349388.1 B/s（页第 159 行，`direct_with_fdatasync_bytes_per_second_median`）。Q17（并发时前台 fsync 延迟）没跑：登记写的是「只在 Q16 > 10% 时跑」（`research/prompts/e185-preregistration.md:336`）。

我再从第一段产物按一条意图（签发加全部批）合起来算挂钟、写字节与写放大（`intent_wall.py`，原样输出）：

```text
$ python3 -I $D/tools/intent_wall.py research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out
arm f P K | intents | intent_wall_ms(med) intent_cpu_ms(med) write_bytes(med) relocation_copy_bytes(med) write_amp(write/copy) write_MB_per_s
memory 0.25 1024 4 | 5 | 35.0 35.0 1820160 524288 3.47 52.0
memory 0.25 1024 8 | 5 | 22.9 22.9 1344512 524288 2.56 58.8
memory 0.25 1024 16 | 5 | 22.9 22.9 1344512 524288 2.56 58.6
memory 0.25 8192 4 | 5 | 288.6 288.4 2147840 524288 4.10 7.4
memory 0.25 8192 8 | 5 | 188.2 188.2 1606656 524288 3.06 8.5
memory 0.25 8192 16 | 5 | 187.3 187.2 1606656 524288 3.06 8.6
memory 0.75 1024 4 | 5 | 85.4 85.3 4902400 1572864 3.12 57.4
memory 0.75 1024 8 | 5 | 49.0 49.0 3409920 1572864 2.17 69.5
memory 0.75 1024 16 | 5 | 37.0 37.0 2934272 1572864 1.87 79.4
memory 0.75 8192 4 | 5 | 708.7 708.6 5524992 1572864 3.51 7.8
memory 0.75 8192 8 | 5 | 400.5 400.4 3835904 1572864 2.44 9.6
memory 0.75 8192 16 | 5 | 301.9 301.8 3294720 1572864 2.09 10.9
real_disk 0.25 1024 4 | 5 | 82.1 38.4 1820160 524288 3.47 22.2
real_disk 0.25 1024 8 | 5 | 60.4 25.8 1344512 524288 2.56 22.3
real_disk 0.25 1024 16 | 5 | 61.1 25.7 1344512 524288 2.56 22.0
real_disk 0.25 8192 4 | 5 | 376.0 293.7 2147840 524288 4.10 5.7
real_disk 0.25 8192 8 | 5 | 266.1 193.1 1606656 524288 3.06 6.0
real_disk 0.25 8192 16 | 5 | 265.3 193.1 1606656 524288 3.06 6.1
real_disk 0.75 1024 4 | 5 | 174.0 90.7 4902400 1572864 3.12 28.2
real_disk 0.75 1024 8 | 5 | 103.6 52.6 3409920 1572864 2.17 32.9
real_disk 0.75 1024 16 | 5 | 82.1 40.1 2934272 1572864 1.87 35.7
real_disk 0.75 8192 4 | 5 | 839.2 720.6 5524992 1572864 3.51 6.6
real_disk 0.75 8192 8 | 5 | 497.0 409.3 3835904 1572864 2.44 7.7
real_disk 0.75 8192 16 | 5 | 391.1 309.9 3294720 1572864 2.09 8.4
```

读法：真盘上一条意图的挂钟 60.4–839.2 ms；写 1.34–5.52 MB，是搬走的字节的 1.87–4.10 倍（元数据那栏，`research/prompts/e185-preregistration.md:333` Q14 写「元数据那栏就是七·1 说的分配记录树写放大的字节」）；GC 单跑的写带宽按意图合计 5.7–35.7 MB/s（Q14 只数批的挂钟所以更高，7.2–50.5 MB/s）。P = 8192 时 5.7–8.4 MB/s，是 CPU 限的（Q1 随 P 翻面）；任何高于 8.4 MB/s 的上限都限不到它。

```text
$ python3 -I $D/tools/batch_write_bytes.py research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out
batch calls 300 write_bytes min/max/med 737792 1753600 967168.0 barrier_calls min/max 6 6 reloc min/max 262144 1048576
```

候选与问题：

| 候选 | 定义 | 对着已量的数 |
|---|---|---|
| 绝对数 | 默认写一个 B/s | 设备不同它的意义不同：Q15 的 3.16 GB/s 是一块 NVMe（`CT2000T500SSD8`，E185 页第 321 行）；换一块慢盘，同一个数可能是 100% |
| 本盘顺序写带宽的比例 | 默认 10%（即登记里 Q16 的「单跑够」门槛）| 要知道本盘带宽：挂载时测一次要写 1 GiB（Q15 的量法），不可取；只能由用户给或不设 |
| 不设（0 = 不限） | 默认不限，靠串行与让路 | 在量过的区间里 GC 单跑最大只占 Q15 的 1.6%（`maximum_ratio=0.015977`），限速在这里不会咬；咬不咬要看前台 fsync，那一项（Q17）没量 |
| 每批之间的最小间隔 | 不用 B/s，用「两批之间至少隔多少毫秒」 | 不需要知道设备带宽；换算：每批写 0.74–1.75 MB、中位 0.97 MB（`python3 -I $D/tools/batch_write_bytes.py <第一段产物>` 对 300 个非签发调用现算，输出见下） |

**结论：E185 回答的只是「GC 单跑够不够小」，回答不了「并发时前台 fsync 被拖多少」。** 第 4 行问题单的候选写的是「按一批在真盘上的写字节与挂钟」／「另跑前台负载量 GC 对前台 fsync 延迟的影响」（`research/prompts/m4-gpugc-measure-forks.md` 第 4 行），登记的停机条件让后一半没跑；而默认值要保护的恰是后一半。单跑够不等于并发不拖：一批写 0.74–1.75 MB、每个调用 6 次屏障（同一脚本输出的 `barrier_calls min/max 6 6`），它排在一次前台 fsync 前面时，前台要多等这一批的盘时间，这是我的推论，没量。

### G6.3　兜底间隔与 E10、C412 的关系

- C412 的行（`.claude/kb/checks-owed.md:291`）写的是：「整理跑一轮要多久、会不会跟前台抢 I/O，一个数都没有」。E185 补了前半：一条意图 60–839 ms（真盘），一批 12.5–25.4 ms（P = 1024）／102–123 ms（P = 8192）。后半（抢 I/O）是 Q17，没量。C412 只还了一半。
- E10 页首行写「部分已跑（第一段 21 片跑了 4 片，读数只当线索；剩 17 片停下，登记的三处要不要修订待定）」（`.claude/kb/experiments/10-老化结构抗老化vs事后整理.md:1`）。老化多快把空段耗光没有可引的时间常数，所以兜底间隔没有从老化速度来的依据。
- 兜底间隔在这里实际决定的是「一次起跑因为某个暂时的原因停下之后，隔多久再试」。暂时的原因在 `CompactionRunStop` 里有几种：被空间拒（`PausedShortOfSpace`）、`PublishBudgetSpent`、`NoYieldFromTheLastIntent`、将来的前台让路。其中 `PublishBudgetSpent` 靠定时器是好不了的（G6.1 第 3 条）。
- 水位查一次的代价：三个量「已由已定项 6 定为持久，挂载即有读数」（D26 已定项 1，`.claude/kb/decisions/26-后台整理与放置回收.md:28`），查是读三个数，我没量。

| 候选（间隔默认） | 依据 | 缺什么 |
|---|---|---|
| 0（关） | 与水位恒 0 同形：今天产品路径起不了跑（G6.1 第 2 条）；等 E10 定水位时一起定 | 条款射程写明「默认值等 C412 与 E10」，这个读法与它一致 |
| 1 s 量级 | 比一条意图最长挂钟 0.84 s 略大，不会在一条意图中途又起一次 | 串行设计下根本不会重叠（`&mut self`），所以这个下界在里程碑四不成立；没有别的依据 |
| 10 s／60 s 量级 | 没有依据，只是量级 | — |

### G6.4　显存上限不给时有没有要另设的保护

「不给就不设上限」不是「会无限占」，因为占用有**用途上限**：窗口再大，φ 降到 10% 以下之后再攒没有收益。E185 Q13 的 n*（页第 201 行）对应的缓冲：f = 0.75 时 73.3–98.6 MB（`merged_buffer_bytes=73313984` 在第三段产物 `:150`，`merged_buffer_bytes=98626320` 在 `:65`，路径 `research/results/e185-gc-batch-wall-clock-split-gpu-submission-r1-2026-10-22.out`），`g2_phi.py` 的式子给 69.8 MiB（卡 1）–93.6 MiB（卡 0）。所以窗口大小按 `min(能攒到的字节, 把 φ 压到 10% 所需的字节, 卡上空闲量)` 取，最大约 100 MiB 加结果缓冲；乙-1（G2 推荐）下能攒到的字节 ≤ 1 MiB，几乎不占。

要另设的保护，从条款与现状推出三处：

1. **起设备要有最小空闲量。** E163 页第 33 行：空闲显存 141 MiB 时 `request_device` 交回 `Device(OutOfMemory)`；checker 档为此另有一个门槛与按进程记账的账本（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:6` 文件头「空闲显存低于门槛的卡不用」）。产品要有同样的「最小空闲量」，低于它按第 4 条报错。
2. **分配失败要能缩窗口而不是直接停。** 第 3 条写「卡上此刻空闲的不到这个数也照常跑，只用空闲的那些」：别的进程中途占走显存时，分配得不到就缩窗口重试，缩到最小一个窗口还不行才按第 4 条的「GPU 出错」停下。这一读法条款没明写，我补的，列进岔路。
3. **wgpu 默认内存块 128–256 MiB**（`wgpu-hal-30.0.1/src/lib.rs:446`），不给上限时也要用 `Manual` 起设备，否则 1 MiB 的活占 128 MiB 的块（G2.3）。

### G6.5　推荐与岔路

**推荐**：（1）写带宽上限默认不限（0），理由是量过的区间里 GC 单跑只占顺序写带宽的 1.6% 以下，而串行设计下前台与 GC 不会并发；真要限，由用户给 B/s；让路在里程碑六实现。（2）兜底间隔默认 0（关），与水位恒 0 同一个读法；同时把 G6.1 第 3 条冲突交用户。（3）显存上限不给时按用途上限取窗口，并设最小空闲量与缩窗口两处保护。

| # | 候选（各自的定义） | 翻面观测 | 够判条件 |
|---|---|---|---|
| G6-a | `B_gc` 与定时兜底：A 兜底只在预算内起跑，预算用完等用户改动（今天的规则）／B 定时器起跑时清零预算（连发可能把回退候选集挤成一个状态，G8 判决 O11）／C 预算按时间补充（每隔一段时间补若干次） | 闲置文件系统上，GC 靠兜底清完一条意图所需发布数与 `B_gc` 的对比：一条意图 2–7 次调用（G1.2 的 `calls` 列），20 次预算够清 3–10 条意图；回退候选集被挤成一个状态是否可接受（`research/prompts/m4-gc-r1-main-verification.md:90`：B_gc 预算按 D16 已定项 1 的根环容量算） | 用户定回退候选集的底线（至少保留几个用户可见状态）；有了底线才能算 B、C 允许的最大连发数 |
| G6-b | 写带宽上限默认：不限／某个绝对数／某盘带宽的比例（用户给）／批间最小间隔 | Q17：并发时前台 `4 KiB O_DIRECT 写 + fdatasync` 的 p99 与不跑 GC 时的比，≥ 2 判「明显拖前台」（登记） | 把登记里的第四段跑一次（Q16 虽 ≤ 10% 但这一格要保护的是并发）；用户定要不要为这一格破登记的停机条件 |
| G6-c | 兜底间隔默认：0（关）／1 s／10 s／60 s | 水位与批大小定下（E10）之后，一条停在暂时原因上的意图隔多久被接着清的实际代价；查水位的 CPU | E10 登记的三处修订有定论、水位有非零默认；量一次水位查询的耗时 |
| G6-d | 显存上限不给时的保护：只靠用途上限／再加最小空闲量与缩窗口／再加一个硬上限（不论卡多大，如 256 MiB） | 同机并存的别的 GPU 进程在 GC 运行期间的显存波动（E163 的 141 MiB 空闲就是实例） | 有一台带别的 GPU 常驻进程的机器上，三种保护下 GC 与那个进程各自的失败次数各量一遍 |

## G7　GPU 代码住哪、用什么写

### G7.1　今天的样子

- 核心层的依赖只有三个：`libc`、`singlefs-format`、`singlefs-replay-prefix`（`crates/singlefs-core/Cargo.toml` 的 `[dependencies]`）；工作区成员八个，没有 `default-members`（`Cargo.toml` 的 `[workspace]`）。
- `wgpu` 只在 checker 档：`crates/singlefs-checker-tier/Cargo.toml:27` 的 `wgpu = { version = "=30.0.1", default-features = false, features = ["vulkan", "wgsl", "std"], optional = true }`，特性 `gpu = ["dep:wgpu", "dep:pollster"]`（`:40`）；那里的注释写明「GPU 核对默认不编」。
- 池级 checker 与实现之间只共享生成的常量（D13 已定项 5：「checker 与实现之间只共享一样东西：一份由 kb 的字段表生成的常量模块……其余一律不共享」，`.claude/kb/decisions/13-验证路线.md:156`）；`crates/singlefs-checker` 只依赖 `singlefs-format`。验证相关的包与档的分法在 `.claude/rules/verification.md:6`–`:19`，其中归类判据：「一条测试或一段代码要枚举崩溃状态、要真设备或外部工具、或跑一次以十分钟计，归 checker 档；否则归 harness 档」（`:18`）。
- 本机编 GPU 相关的东西缺这些：`nvcc`、`glslc`、`glslangValidator`、`spirv-as`、`dxc` 都不在 `PATH`（我现跑 `command -v`，全部无输出），`/usr/local/cuda*` 不存在。
- 注册表里有 `wgpu-30.0.1`，它的特性有 `spirv`（`wgpu-30.0.1/Cargo.toml:86`）、`glsl`（`:70`），以及 `ash-0.38.0+1.3.281`（wgpu 的 Vulkan 层本来就拉它）。（`~/.cargo/registry/src/*/`，仓外，我现查。）

### G7.2　包与依赖方向

D24 已定项 2 第 1 条：「文件系统必须在没有 GPU 时完整工作、且语义完全相同。⇒ GPU 是可选加速器，一个比特都不许进格式」（`.claude/kb/decisions/24-后台重活能不能卸给GPU.md:34`）。里程碑七·2 推的落点：「单开一个可选的包，core 不带它也能编」（`.claude/kb/milestone/04-fourth-txn.md:144`）。

| 候选 | 定义 | 利 | 弊 |
|---|---|---|---|
| D-1 | 新包（暂名 `singlefs-gpu-offload`）依赖 `singlefs-core`；核心层定 trait（G5），GPU 包实现它；核心层不依赖 GPU 包 | 一个新包，依赖单向（GPU 包 → core），GPU 包能用核心层的类型 | GPU 包能 `use` 核心层的 `crc32_castagnoli`，两份核「独立」要靠评审或一条检查守（见下） |
| D-2 | 再多一个只含 trait 与纯数据类型的接口小包（只依赖 `std`），核心层与 GPU 包都依赖它 | GPU 包不依赖核心层，物理上用不到核心层的 CRC，独立性由依赖图保证 | 多一个包；接口小包里的类型要改时两边同改 |
| D-3 | 不定 trait：核心层收一个闭包 `FnMut(&[&[u8]]) -> Result<Vec<[u32; 4]>, String>`，GPU 包只依赖 `singlefs-format` | 依赖最少 | 类型弱；错误信息是字符串，G5 的「停机原因」要从字符串里认；不推 |
| D-4 | 在核心层里加 Cargo 特性 `gpu` 依赖 `wgpu` | 不新增包 | 核心层的依赖闭包里有了 `wgpu`，「core 不带它也能编」靠特性不靠依赖图；特性在工作区里会被别的包的构建统一打开；不推 |

**依赖方向**：核心层不依赖 GPU 包（D-1、D-2、D-3 都满足）。GPU 包 → 核心层（D-1）或 GPU 包 → 接口小包 ← 核心层（D-2）。装配（把 `ComputeBackend::Gpu(handle)` 交给挂载）在里程碑十的前端：`grep -rn 'fuse' crates/*/Cargo.toml` 零命中，今天仓里没有这个包。

**工作区怎么放**：

| 候选 | 定义 | 对着已知的 |
|---|---|---|
| W-1 | 加进 `members`，同时写 `default-members` 把它排除在默认构建之外 | `cargo build --workspace` 仍会编它；`wgpu` 的锁定条目已在 `Cargo.lock`（D26 已定项 13 写 `wgpu 30.0.1`，`.claude/kb/decisions/26-后台整理与放置回收.md:245`），不新增锁定包 |
| W-2 | 加进 `members`，不设 `default-members` | `cargo test --workspace`、门禁与变异表都会带上它；`wgpu` 的编译量没有可引的数：七节调查员「编译计时没量成」（`records/2026-10-19-里程碑四GPU垃圾回收开工.md:316`） |
| W-3 | 像 `scripts/experiment-apparatus` 那样放在工作区外（`exclude`） | 门禁与变异表看不到它；产品代码不该这样 |

### G7.3　CPU 参照住哪、对拍放在哪

D24 已定项 2 第 3 条：「GPU 路径与 CPU 路径必须是两份实现，两份在测试里对拍」（`.claude/kb/decisions/24-后台重活能不能卸给GPU.md:36`）。产品里的 CPU 路径已经有：`crc32_castagnoli`（`crates/singlefs-core/src/checksum.rs:43`）与 `parse_data_unit`（`crates/singlefs-core/src/unit.rs:581`）。D26 已定项 13 第 8 条里「交 CPU 那一份实现单独判一次」的那一份就是它们。所以**CPU 参照不用新写，住在核心层**；新写的只有 GPU 一侧的两份核。

三层算法对同一个函数：CPU 参照是 slicing-by-8 软件表（`crates/singlefs-core/src/checksum.rs:43`）；核 A 反射按位；核 B 正规按字节查表（G4）。

对拍分两层，放哪一档按 `.claude/rules/verification.md:18` 的判据分：

1. **不要真卡的那一层**进 harness 档（快档）：「核 A、核 B、比对、N 轮、交 CPU 判」的整套裁决逻辑，对一个**核心层里写的、软件实现的假后端**（实现同一个 trait，可按单元注入一份核算错）跑。七·3 的验收里「塞一个让其中一份核在一个条目上算错的故障，那一条记状态不确定、这一批不回收不搬，下一轮去掉故障后回收」「同一条连续 N 轮都算错，第 N 轮交 CPU 判」「每批报的确认、不确定、交 CPU 判三个数钉绝对值」都能在这一层红（`.claude/kb/milestone/04-fourth-txn.md:145`）。
2. **要真卡的那一层**进 checker 档：真 WGSL 核对 CPU 参照在 E163 同一批输入上逐项相同，八个绝对值锚点，往 GPU 一侧塞已知故障对拍红。理由是「要真设备」。

这与七·3 验收写的「对拍进快档」有一处差：真卡对拍进不了快档（快档随时跑、不要求有 GPU）。是按上面分两层，还是把整个对拍记成 checker 档，列进岔路 G7-c。

### G7.4　与 checker 档已有的 GPU 代码共不共用

**不共用，两个方向都不。**

- 产品用 checker 档的核：把测试装置放进产品，且产品的校验和核与「判产品校验和对不对」的装置同一段，D13 已定项 5 要隔开的正是这个（`.claude/kb/decisions/13-验证路线.md:156`，「校验和各用一份独立实现」）。
- checker 档用产品的核去判产品：同理。checker 档可以把产品的 GPU 包**当被测对象**调用，对拍时的参照是 checker 档自己的 CRC（`crates/singlefs-checker/src/lib.rs:60` `crc32_castagnoli_table`）与它自己已有的 WGSL 核（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:80`），那是另一份。
- 所以产品 GPU 包里的两份核 + 核心层的 CPU 软件表 + checker 档的按位 CRC 与 WGSL 核，是**至少四份**互不引用的 CRC-32C 实现。它们各自的对错靠 D13 已定项 5 的参数钉死与 G4 的绝对值锚点钉。
- 共用的只有文字上的参数与布局常量：CRC-32C 的多项式、初值、输入输出反转、异或输出是 D18 已定项 17 的文字（`crates/singlefs-core/src/checksum.rs:1`–`:3` 注释复述），各实现各自声明一份（`grep -c -i 'polynomial\|0x82F63B78\|1EDC6F41' crates/singlefs-format/src/lib.rs` 我现跑输出 0，常量模块里没有它；E163 页第 74 行说 Rust 侧与 WGSL 侧是「两处独立声明的常量」）；单元布局常量 `DATA_UNIT_HEADER_BYTES` 等在 `crates/singlefs-format/src/lib.rs:35`，这个是真共用的常量模块。

### G7.5　WGSL 经 wgpu 之外有没有更好的做法

D26 已定项 13 原文：「GPU 路径默认写成 WGSL、经 `wgpu` 起」（`.claude/kb/decisions/26-后台整理与放置回收.md:245`），括注里写「不用另装 CUDA 工具链」（同一行），后一句「有更好的做法可以在设计轮比过之后换」（同一行）。比的维度和候选：

| 做法 | 要不要装工具链（本机现状） | 跨厂商 | 显存总量与空闲量 | 两份核的编译器能不能不同 | 备注 |
|---|---|---|---|---|---|
| WGSL 经 wgpu（现状） | 不要 | 是（Vulkan 后端） | `AdapterInfo` 没有显存字段，要另起 `nvidia-smi` 之类（G2.3） | 不能：WGSL → naga → SPIR-V → 驱动，两份核同一个 naga | checker 档与 E163 已验过 NVIDIA 两张卡 |
| WGSL 加 SPIR-V 直送：核 B 用另一种语言（GLSL）经另一个编译器（`glslc`）编成 SPIR-V，由 wgpu 的 `spirv` 特性装入 | 要装 `glslc`（本机无） | 是 | 同上 | **能**：两份核走不同的前端编译器，仍共用驱动 | 把「同一个着色器编译器」这项共用从两份核之间拿掉一半；`wgpu` 要多开 `spirv` 特性 |
| 经 `ash` 直接用 Vulkan（预编译 SPIR-V） | 要装 SPIR-V 编译器（本机无） | 是 | 能读 `VK_EXT_memory_budget` 与堆大小，补 G2-c 的缺口 | 能 | 要写 `unsafe` Vulkan 调用；`ash` 已在锁文件里 |
| CUDA（`nvcc` 或 `cudarc`） | 要装 CUDA 工具链（`nvcc` 本机无，D24 已定项 3 也写「`nvcc` 没有装」） | 否（NVIDIA 专用） | NVML 可读 | 能（PTX 与 CUDA C 两种）| D26 已定项 13 明说不用另装 CUDA 工具链 |
| OpenCL | 要装 ICD 与编译器（未查本机） | 是 | 能 | 能 | 我没有本机现状可引，标「未核实」 |

### G7.6　推荐与岔路

**推荐**：新包 + D-2 的依赖形（接口小包）或 D-1；核心层不依赖 GPU 包；W-1；CPU 参照用核心层现成的 `crc32_castagnoli` 与 `parse_data_unit`；与 checker 档不共用；第一版 WGSL 经 wgpu，两份核同走 naga。

我这里不替用户在 D-1 与 D-2 之间挑：D-1 省一个包，代价是「两份核独立」靠评审；D-2 多一个包，代价是接口小包的维护，换来依赖图保证。差别落在 G4-d 之外的另一处：GPU 包能不能物理上引到核心层的 CRC。

| # | 候选（各自的定义） | 翻面观测 | 够判条件 |
|---|---|---|---|
| G7-a | 包与依赖方向：D-1／D-2／D-3／D-4（上表） | 评审里「GPU 包不许 `use` 核心层的 CRC 与解析」这条能不能写成会红的检查（grep 级即可）；若写得出，D-1 足够 | 那条检查写出来，塞一个 `use singlefs_core::checksum` 进去会红 |
| G7-b | 第一版编译器：两份核同走 naga（现状）／核 B 走 `glslc` 编的 SPIR-V／经 `ash` 直接 Vulkan | 一个 naga 的缺陷会让两份核同错的概率有没有证据：拿核 A（WGSL）与同一算法的 GLSL/`glslc` 版在 E163 的 2560 个单元与八个锚点上逐项比，有一处不同就坐实 naga 是共用风险；全同则没有证据说它是 | 本机装得上 `glslc`（现无）；或用户给另一台有它的机器 |
| G7-c | 对拍放哪：分两层（假后端进快档、真卡进 checker 档）／全记 checker 档 | 假后端能不能覆盖七·3 验收里所有「注入一份核算错」的格；覆盖不全的那几格 | 七·3 验收逐格对一遍，哪些格要真卡写出来 |
| G7-d | 工作区：W-1／W-2／W-3 | 加进 `members` 后 `cargo build --workspace` 的编译挂钟与内存峰值（`scripts/hosts/run-with-memory-cap.sh`，不超过 16G 的上限）；门禁各格多出的时间 | 在一次性编译目录里量一次，写明命令与上限；我这轮没编（派发提示不跑重活） |

## 冲突与规则没说（并排整行抄）

**冲突 1（G2）：攒批要先验字节，每批重扫要后验字节。**

- D26 已定项 13 第 3 条：「GPU 模式把连续几批的计算攒起来一次交给 GPU，攒多少受它管。」（`.claude/kb/decisions/26-后台整理与放置回收.md:238`）
- `crates/singlefs-core/src/mounted_session.rs:2975`：「///    都发一次只删意图的发布丢掉它（攻方 K-c）；对 R 重扫（每批之前都重扫，丁 1 交判第 3 点）、计划一批（跳过这次发布本来就重写的节点、补上 R 里」

两句同时成立，要把「重扫」拆成字节与状态两半（G2.4）：字节窗口开头验一次，状态每批重扫。这改了第二句的一半。

**冲突 2（G6）：定时兜底要在没有用户改动时接着清，预算只在用户改动时归零。**

- D26 已定项 13 第 5 条：「什么时候起跑：每次发布之后按已定项 1 的三水位查一次；另有定时兜底，每隔一段时间再查一次，间隔是挂载参数。两种模式起跑规则相同。」（`.claude/kb/decisions/26-后台整理与放置回收.md:240`）
- `crates/singlefs-core/src/compaction_plan.rs:806`：「/// `3S − 4 − 已经过去的发布数`，至少 1。`S` 是每区根槽数。」与 `crates/singlefs-core/src/mounted_session.rs:1926`：「compaction.publishes_since_the_last_user_change = 0;」

**冲突 3（G7）：对拍放哪一档。**

- `.claude/kb/milestone/04-fourth-txn.md:145` 七·3 的验收里：「两条路径在同一组输入上逐项相同、对拍进快档」
- `.claude/rules/verification.md:18`：「归类判据：一条测试或一段代码要枚举崩溃状态、要真设备或外部工具、或跑一次以十分钟计，归 checker 档；否则归 harness 档。」

真卡上的 GPU 对拍要真设备，按第二句归 checker 档；七·3 的第一句要的「进快档」只有不要真卡的那一层（假后端）做得到（G7.3）。

**规则没说 4（G6）：串行设计下「前台有 fsync 在等」没有表示。**

- D26 已定项 13 第 6 条：「起跑之后的节奏：GC 占盘写带宽有一个上限（挂载参数）；前台有 fsync 在等时不发下一批。」（`.claude/kb/decisions/26-后台整理与放置回收.md:241`）
- `crates/singlefs-core/src/mount.rs:9560`：「/// 新根持久之后才交回 `Ok`（与 fsync 同一个返回条件）；那时 `current` 换成回退那次发布之后的一版。回退那次发布的事务号写 0」是核心层里 `fsync` 字样的全部（另一处在 `block_device.rs`）；`grep -n 'std::thread\|thread::spawn\|Mutex\|Arc<\|AtomicBool\|mpsc'` 在 `mounted_session.rs` 与 `mount.rs` 零命中。

**规则没说 5（G1）：GPU 模式比 CPU 模式慢多少还做。**

- D26 已定项 13 第 7 条：「GPU 模式的目的是少占主机 CPU 时间，数据量大时跑得更快只是副产物。」「挂钟与 CPU 模式持平也做」（`.claude/kb/decisions/26-后台整理与放置回收.md:242`）。条款只写到持平，没写慢于持平时做不做；P = 1024 档加一次提交的挂钟地板就是 +34%–36%（G1.4 问 4）。

**规则没说 6（G4）：「这一批不回收、不搬」是整批还是条目。**

- D26 已定项 13 第 8 条：「两次不同的条目写明状态不确定，这一批不回收、不搬，留到下一轮重算。」（`.claude/kb/decisions/26-后台整理与放置回收.md:243`）

**规则没说 7（G4）：「没有 ECC 时的位翻转」抓不抓得到，取决于「输入表的准备」包不包括显存里的输入拷贝。**

- D26 已定项 13 射程：「第 8 条的两份核抓得到只有一份出的错（一份核写错、没有 ECC 时的位翻转、不确定的竞争）」与「两份共用的那几样（同一个驱动与着色器编译器、同一份输入表的准备、对规则的同一种误读）」（同文件 `:247`）。

## 什么现象会推翻它

| 结论 | 推翻它的现象 |
|---|---|
| G1：甲乙丙合计天花板 ≤ 4.59%（一条意图）、发布那一半不放 GPU | 在另一批点（别的 f、P）上量到三段合计 ≥ 10%；或按函数拆开发布那一半，找到一段输入是平字节表、又不在共享事务层里 |
| G1：推荐只放扫描里的校验和 | c_rt 量出来大于一次调用的 CPU 上界（0.54 ms 量级）：则任何候选都亏，推荐不成立；或丙在真池上展平 CPU 远小于丙本身 CPU |
| G2：乙-1 φ ≥ 0.89 | 常驻缓冲写法下 a 量出来比 4.49 ms 低一个数量级（G1.7）；φ 随之变，乙-1 的「压不下固定开销」这句要重算 |
| G2：字节验一次、状态每批重扫是安全的 | 注入一个单元字节在两批之间翻位，发布路径的释放前读盘核没有把它隔离；或 R 内的槽字节在意图期内能被前台新写改动 |
| G3：S2 为默认分活 | G1-a 量出一次提交的主机 CPU 里「固定部分」可以忽略（c0 ≈ 0），此时 S3 的额外提交不亏，G4-d 的独立性就压过提交次数 |
| G4：比对放 CPU、N = 2 | 量出 C_w 小于 C_j（一轮重算比 CPU 判一个单元还便宜，如读法 C 批内就地重算）；此时 E(N) 不再随 N 单增 |
| G4：两份核算法不同 | 两份算法在锚点或 E163 的 2560 单元上给出不同的数（说明某一份写错，不是推翻独立性，是推翻「已写好」） |
| G5：第一版取会话方法 | 里程碑十的前端需要在一次运行进行中切换，或整理被放到独立线程上跑 |
| G6：写带宽上限默认不限 | Q17 量出来在单跑 ≤ 10% 的前提下前台 fsync 的 p99 仍 ≥ 2 倍 |
| G6：兜底间隔默认 0（关） | E10 登记的三处修订有定论、水位有非零默认之后，水位查询本身有可测的代价或意图经常停在暂时原因上 |
| G7：核心层不依赖 GPU 包、依赖单向 | 装配层需要核心层反过来知道 GPU 包的类型（则 trait 应放得更靠下） |

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| G1 | 规则没说（尺有，净值缺） | 第 7 条的尺有定义，E185 给出上界（一条意图 0.43%–4.59%，一批最大 4.9%，签发那一次最大 7.3%）；GPU 一侧主机 CPU（准备、提交、等、比对）一项没量，净收益符号不知道；推荐第一版只放扫描里的校验和，丙与去冗余留岔路 |
| G2 | 冲突（冲突 1） | 条款要攒批，代码每批重扫字节；乙-1 压不下固定开销（φ ≥ 0.89），达到 φ ≤ 10% 要 70–94 MiB，只有跨意图预取或宽 R 够得着，宽 R 要改盘上槽数字段；显存上限对乙-1 只是起卡门槛，wgpu 默认 128–256 MiB 块比任何小上限都大 |
| G3 | 规则没说 | 默认次序是用户可见的接口；推荐 PCI 总线号升序、已验厂商的独显、按窗口轮转；「全部」里有卡起不来是拒绝还是跳过条款没写 |
| G4 | 规则没说（规则没说 6、7） | 两份核算法不同、输入缓冲各一块、起卡前算绝对值锚点；比对放 CPU；N = 2（期望 CPU 代价在 N = 2 最小，对任何 r 成立）；「这一批」的读法交用户 |
| G5 | 一致 | 里程碑七已写下留口范围；第一版会话方法，边界在 `advance_compaction_by_one_publish` 进门，加打包只加一个枚举成员 |
| G6 | 冲突（冲突 2）加规则没说（规则没说 4） | 兜底受预算 `B_gc`（20 次）限，闲置时清不完；写带宽上限的并发那一半 Q17 没量；兜底间隔与水位同为 0；显存上限不给时按用途上限取窗口 |
| G7 | 一致（落点），冲突（冲突 3，对拍放哪一档） | 新包、核心层不依赖它、CPU 参照用核心层现成的、与 checker 档不共用；真卡对拍归 checker 档、假后端进快档 |

## 复核不了与没量

- 复核不了：GPU 一侧主机线程 CPU 时间（准备、提交、等结果、读回、两份核、比对），派发提示禁起 GPU；E185 第三段只有挂钟。常驻缓冲写法的固定开销同理。
- 复核不了：`wgpu`、`wgpu-hal`、`wgpu-types` 里的几行在 `~/.cargo/registry/src/*/` 下，不在仓里，cite-check 判不了，我用 `grep -n` 现读。
- 没量：改身份索引之后一批 CPU 按函数拆开的各成（G1 问 3）；`Vulkan` 适配器枚举次序是否随驱动或加卡变化；`device.poll(Wait)` 睡眠还是自旋；非 NVIDIA 卡上 WGSL 核的行为；`wgpu` 编译挂钟与内存峰值。
- 推的、已在文里标「推的」的：展平的 CPU 与被省的同阶；两份核同提交约 a + 2·b·B；硬件 CRC 指令会降低甲的 CPU。

## 没做什么

- 不判别的腿的格；不替主 agent 采纳或出判决；岔路表只写候选、翻面观测、够判条件，不写倾向（各格「推荐」一栏是正推腿从条款与读数推到的落点，不是对岔路的倾向）。
- 没有编译、没有起 GPU、没有跑门禁阶段与 checker 档测试；没有读禁读清单里任何文件的内容（一次 `ls` 与一次 `grep -rln` 的文件名列表里出现过 `m4-gpugc-design-r1-opus-*` 的名字，我没有打开）。
- 没有做模型，只有读 E185 入库产物的几个小脚本，在草稿目录；没有对 `crates/` 做任何改动。

## 工具清单

草稿目录 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-design-r1-sonnet/tools/` 下的脚本（只读 E185 入库产物，不写仓）：

```text
$ cd $D/tools && sha256sum *.py
6df265aad15fea58ac26f22cefc13114108f078f95ad00fd1b4086aeacc7d1d4  absolute_ms.py
00ad3e4a4c65723324167d9fed880e7ef1639c4b55c0c538b51e6a320c2877ae  arms.py
67d3fd6ff28383fcee9d932707841ecd1bdc28bc5bc32a12479100bfe4d27762  batch_write_bytes.py
e01740fb06467b9c4d2942752452509dbb2f24dca47be4a821e490d958d91ec4  callshares.py
6f4dd06aae6f96d2d387b9dc173c18ff4c31f6af19434f8f601711c00b84331b  crc_bytes_per_intent.py
a6967ce00b17da83e8591484a8ff125004b4e2eea6180484994f6e4e866e2c6f  g1_example.py
19e6cc9d8d10b755e2779c688a17393905c3d8868b4ff155c49e121eb16f1298  g1_upper_bound.py
ee7a3f6fafc6b6e513201c42bd196fee7464201faa12bae3d981c23f6f45b81d  g2_phi.py
569fd4b1b294cd275991ab34fbf0b16a8067c1bf9e2d45d7f427ff1105b57cb7  g3_wall.py
a6b58c7656b661d1449ea006b805d9653f4fb91162e2c5b18116256e4732295f  g4_n_cost.py
9521492a6fe74b8a5eb0c84f5c6c6bed834306d8f09db9eccc9f53ecfecdb9fa  intent_wall.py
```

报告里每一块「原样输出」我在交回前重跑对过：十一个脚本的输出都逐字出现在上文（`arms.py` 只取表头与批行）。
