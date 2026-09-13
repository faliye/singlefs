# 背景材料：D14（双轨（大小文件 / 持久临时）） 未定项 2 第三轮——减数臂已定（D3（空间分配） 已定项 8）、被减数在同一装置上量过之后，「分配器提示的判据写成什么」怎么定

<!-- doc-lint:not-numbers H1 H3 J1 J2 J3 J4 J5 P1 P2 P3 P4 P5 P6 P7 P8 P9 -->

## 一、前提（写材料的人逐条现查过，条款整段抄在附录里，正文只指路；产物行整行抄在第四节）

| # | 前提 | 出处（附录里都有） |
|---|---|---|
| P1 | D14 给分配器提示这一层记的三个分句「不落盘、每次分配都可以重新判、零格式位」，判据要三句全过；`.claude/rules/fs-design.md` 硬要求 1（判据显式可计算写下来）、2（测试可强制进入）、5（分支变量不在操作中途改变） | D14「采纳的两层」；D14 未定项 2 的判据段 |
| P2 | 三条臂的下场：臂甲（后缀表）出局（C65、目标负载不指向它、「疑似短命」空白）；臂乙（上层显式给）的机制到不了分配层；臂丙（按目录）有机制、收益此前无据；臂庚（到达序）2026-09-09 之后提出、只记不判；「判据取 `locality_id`」2026-09-09 按三个分句判掉（破「不落盘」、掏空「每次分配可重判」，第一版恒 0，加密时新增语义类别泄漏）；将来真做只能取「创建时继承一次、此后不再改」 | D14 未定项 2 全文 |
| P3 | 减数臂 = D3 已定项 8（2026-09-13 用户定案）：第一版用户数据不进聚簇段，落已选设备内槽号最小、过准入合取、不在 `R` 保护期内的空槽；聚簇段只给提交内生块；≤ 4 KiB 打包前同此；只限第一版、不预判 D14 未定项 2；D3 已定项 5：提交内生块**必须**从聚簇段分配 | D3 已定项 8、已定项 5、已定项 7 |
| P4 | E151 第四次跑在减数臂的装置上加了两条「按目录」臂（目录 = 32 个连续 key，初始每目录一段连续）：`directory_hint`（候选槽 = 该目录成员所在段里的空槽，没有就回落）与 `directory_home`（家 = 初始所在段 + 一个尾部溢出段，每个溢出段归 8 个目录，家满才回落、回落不改家；它是跑产物之前按第一条臂的单测补的，修订记在跑前登记）。数在第四节 | E151 正文；`research/prompts/e151-r4-prereg.md`；产物 |
| P5 | E122 的两格收益（目录遍历段数、删目录腾出整段）都挂在交错度上，且 E122 一次性计数、不建模老化与再分配，模型只对 > 4 KiB 或搬走之前成立 | E122 全文 |
| P6 | D26 已定项 1：`R` 在意图期内不可作分配目的地，起跑水位含全空聚簇段数；已定项 3：准入第二合取式 `(可用 ≥ 需求) ∧ (全空聚簇段数 ≥ N₀)`；已定项 4：不存组身份、不动节点布局，政策那一半落回 D3 已定项 8 与 D26 已定项 1 | D26 那三节 |
| P7 | ≤ 4 KiB 对象写完之后按 D27 已定项 6 沿 key 序打包进容器（已定项 1 / 2：小对象落盘两次）⇒ 提示对那一档只管搬走之前 | D27 那三节 |
| P8 | D25 唯一的「主」是 seq 顺序大文件，负载表里没有目录交错这一维（有 multistream 一行，权重「次」）；D18 已定项 14 的五处已接受泄漏全是元数据的发布次序或轮换，「泄漏的是发布次序还是对象的语义类别」是那条区分；D8 已定项 3 / 6 的 `locality_id` 从父目录继承一次、第一版取 0 | D25、D18、D8 那几节 |
| P9 | 欠账：C65（分配器提示落不落盘自相矛盾：F2FS 例子住 superblock 与「零格式位」打架）、C239（基线臂：第 ① 笔 2026-09-13 已定、第 ② 笔 E151 已补，剩 E122 两臂对减数臂的映射）、C146（回落政策：第 ② 条已由 D3 已定项 8 答，第 ① 条归 D26 已定项 1）、C233（E122 结论点名的独有收益在自己产物里有反例，已改正） | 欠账表那四行（`--extra`） |

## 二、问法（五问）

1. **「按目录聚」在老化装置上买到什么、E122 的 `grouped` 对应哪条臂。** 跟着成员走的提示（`directory_hint`）在 2000 轮之后与减数臂几乎一样散；家固定的提示（`directory_home`）把每个目录钉在 2 段以内，目录遍历段数是减数臂的 54%–55%（跑前写死的 50% 没到、80% 推翻线也没到），runs8 上 key 序 runs 少 18%，uniform 打平。E122 的 `grouped`「同一父目录下的对象在盘上连续」在老化下对应哪一条？两条都不是的话，「按目录」这条臂在有再分配的系统里到底是什么。
2. **家固定的臂付了什么。** 它的溢出段就是尾部那 32 个初始全空段，第 2 轮就吃光（runs8 第 14–15 轮），M = 4 / 9 时提交内生块回落 7936 / 8000、17936 / 18000——与 D3 未定项 8 第二轮判掉「用户数据与提交内生块共用 bump 分配器」的是同一个量（D3 已定项 5 的「必须」）。能不能把溢出段改成「留 N₀ 个全空段给提交内生块」而不失去 2 段以内的局部性——这是推导题，推不出就说要第五次跑，不许补数。
3. **三个分句与硬要求。** 家固定的提示：家从 key 算（目录 = key 区间，真实系统里是「创建时父目录的家段」），每次分配现算，零盘上字段。它过不过「不落盘」（父目录关系本来就在盘上的 dirent / inode 里，与 `locality_id` 住 key 编码是不是同一种「落盘」）、过不过「每次分配都可以重新判」（家固定 = 创建时继承一次此后不改，正是 2026-09-09 用来判掉 `locality_id` 的那一句「掏空第二句」）、过不过硬要求 5（家在操作中途不变）。
4. **收益有没有消费者。** 段粒度的目录局部性（一个目录住 2 段以内）与删目录腾出的 32 槽块（runs8 上 11–12 对 0）今天有没有任何已定条款消费它：D25 的主负载 seq 顺序大文件不按目录读；「目录遍历」是 smallfile / multistream 那一档的事，权重「次」；删目录腾段喂的是 D26 已定项 1 的全空段供给。没有消费者的收益怎么进判决。
5. **要写成条款的形态。** 甲：第一版不做分配器提示，本项定「否」，判据栏写「无」，三个分句自动全过，「创建时继承一次此后不改」记为将来做时的机制约束；乙：第一版做家固定的目录提示（家 = 创建时父目录的家段 + 留 N₀ 的溢出段），这要改 D3 已定项 8 第 1 条（落点从「已选设备内槽号最小」改成「家内槽号最小、家满再全池」），而 D3 已定项 8 是用户当日定案；丙：维持未定，等真实负载的目录交错度。

## 三、判据（跑前写死）

| # | 判据 | 打红条件 | 打红要交的 | 打红后的处置 |
|---|---|---|---|---|
| J1 | 收益的消费者 | 有腿点名一条已定条款（不是「将来」「可能」）消费段粒度目录局部性或删目录腾段 | 条款出处整行 | 记「收益有消费者」，第 5 问的甲要重写理由 |
| J2 | 代价能不能免 | 有腿给出一个溢出段来源，家固定的臂不吃提交内生块的全空段、且每目录仍 ≤ 3 段（推导，写明溢出段从哪来、多大） | 设计一句 + 推导 | 第五次跑加臂，本轮不定乙 |
| J3 | 三个分句 | 有腿逐字证明家固定的提示破三个分句里的哪一句 | 分句原文 + 破在哪 | 乙出局，甲 / 丙二选一 |
| J4 | 硬要求 1 / 2 / 5 | 有腿证明家固定的提示写不成显式可计算的判据、或测试强制不进、或分支变量在操作中途变 | 逐条 | 同 J3 |
| J5 | 与 D3 已定项 8 的关系 | 乙要改 D3 已定项 8 的哪一句、改法是收严还是放宽 | 逐字 | 放宽 ⇒ 乙要用户重开 D3 已定项 8 |

**失败条款**：三腿对 J1 不一致 ⇒ 核实（全仓 grep「目录局部性 / 遍历 / readahead / 腾段」的消费方），不投票；J2 打中 ⇒ 不定乙，第五次跑；J3 或 J4 打中 ⇒ 乙出局。
**什么观测会触发**：J1——某腿贴出一条已定项原文里写着「按目录 / 目录遍历 / 删目录腾段」；J2——某腿给出与尾部全空段不重叠的溢出来源；J3——某腿把家固定与「每次分配都可以重新判」并排写出矛盾；J5——某腿指出乙的落点句与 D3 已定项 8 第 1 条不能同真。

## 四、E151 第四次跑的数（整行抄自 `research/results/e151-arrival-and-container-arms-2026-09-13-directory.out`，目录局部性只报 M = 0 / 9 两档）

```text
E7RESULT name=directory arm=first_fit load=uniform metadata_blocks=0 directory_runs_total=8025 segments_touched_mean_percent=1986 segments_touched_max=27 single_directory_chunks_32=0 single_directory_chunks_64=0 runs_median=8186 hint_fallback_seed0=0 fallback_pct=0.0
E7RESULT name=directory arm=bump_seg load=uniform metadata_blocks=0 directory_runs_total=8044 segments_touched_mean_percent=2207 segments_touched_max=30 single_directory_chunks_32=17 single_directory_chunks_64=7 runs_median=8186 hint_fallback_seed0=0 fallback_pct=93.5
E7RESULT name=directory arm=arrival_by_request load=uniform metadata_blocks=0 directory_runs_total=8168 segments_touched_mean_percent=2868 segments_touched_max=32 single_directory_chunks_32=12 single_directory_chunks_64=8 runs_median=8189 hint_fallback_seed0=0 fallback_pct=93.3
E7RESULT name=directory arm=directory_hint load=uniform metadata_blocks=0 directory_runs_total=7847 segments_touched_mean_percent=1221 segments_touched_max=24 single_directory_chunks_32=0 single_directory_chunks_64=0 runs_median=8183 hint_fallback_seed0=17285 fallback_pct=0.0
E7RESULT name=directory arm=directory_home load=uniform metadata_blocks=0 directory_runs_total=4420 segments_touched_mean_percent=167 segments_touched_max=2 single_directory_chunks_32=0 single_directory_chunks_64=0 runs_median=8063 hint_fallback_seed0=0 fallback_pct=0.0
E7RESULT name=verdict_directory arm=directory_hint load=uniform metadata_blocks=0 first_fit_directory_runs=8025 hint_directory_runs=7847 hint_at_most_half=false first_fit_segments_touched_max=27 hint_segments_touched_max=24 hint_segments_touched_mean_percent=1221 hint_stays_in_few_segments=false first_fit_single_chunks_32=0 hint_single_chunks_32=0 hint_more_single_chunks=false first_fit_runs=8186 hint_runs=8183 key_runs_tied_within_5pct=true
E7RESULT name=verdict_directory arm=directory_home load=uniform metadata_blocks=0 first_fit_directory_runs=8025 hint_directory_runs=4420 hint_at_most_half=false first_fit_segments_touched_max=27 hint_segments_touched_max=2 hint_segments_touched_mean_percent=167 hint_stays_in_few_segments=true first_fit_single_chunks_32=0 hint_single_chunks_32=0 hint_more_single_chunks=false first_fit_runs=8186 hint_runs=8063 key_runs_tied_within_5pct=true
E7RESULT name=directory arm=first_fit load=uniform metadata_blocks=9 directory_runs_total=8028 segments_touched_mean_percent=1986 segments_touched_max=27 single_directory_chunks_32=0 single_directory_chunks_64=0 runs_median=8186 hint_fallback_seed0=0 fallback_pct=0.0
E7RESULT name=directory arm=bump_seg load=uniform metadata_blocks=9 directory_runs_total=8044 segments_touched_mean_percent=2208 segments_touched_max=30 single_directory_chunks_32=17 single_directory_chunks_64=9 runs_median=8186 hint_fallback_seed0=0 fallback_pct=94.2
E7RESULT name=directory arm=arrival_by_request load=uniform metadata_blocks=9 directory_runs_total=8161 segments_touched_mean_percent=2862 segments_touched_max=32 single_directory_chunks_32=13 single_directory_chunks_64=6 runs_median=8188 hint_fallback_seed0=0 fallback_pct=94.2
E7RESULT name=directory arm=directory_hint load=uniform metadata_blocks=9 directory_runs_total=7776 segments_touched_mean_percent=1096 segments_touched_max=18 single_directory_chunks_32=0 single_directory_chunks_64=0 runs_median=8177 hint_fallback_seed0=13627 fallback_pct=0.0
E7RESULT name=directory arm=directory_home load=uniform metadata_blocks=9 directory_runs_total=4406 segments_touched_mean_percent=168 segments_touched_max=2 single_directory_chunks_32=0 single_directory_chunks_64=0 runs_median=8066 hint_fallback_seed0=9 fallback_pct=12.3
E7RESULT name=verdict_directory arm=directory_hint load=uniform metadata_blocks=9 first_fit_directory_runs=8028 hint_directory_runs=7776 hint_at_most_half=false first_fit_segments_touched_max=27 hint_segments_touched_max=18 hint_segments_touched_mean_percent=1096 hint_stays_in_few_segments=false first_fit_single_chunks_32=0 hint_single_chunks_32=0 hint_more_single_chunks=false first_fit_runs=8186 hint_runs=8177 key_runs_tied_within_5pct=true
E7RESULT name=verdict_directory arm=directory_home load=uniform metadata_blocks=9 first_fit_directory_runs=8028 hint_directory_runs=4406 hint_at_most_half=false first_fit_segments_touched_max=27 hint_segments_touched_max=2 hint_segments_touched_mean_percent=168 hint_stays_in_few_segments=true first_fit_single_chunks_32=0 hint_single_chunks_32=0 hint_more_single_chunks=false first_fit_runs=8186 hint_runs=8066 key_runs_tied_within_5pct=true
E7RESULT name=directory arm=first_fit load=runs8 metadata_blocks=0 directory_runs_total=7956 segments_touched_mean_percent=1972 segments_touched_max=26 single_directory_chunks_32=0 single_directory_chunks_64=0 runs_median=7997 hint_fallback_seed0=0 fallback_pct=0.0
E7RESULT name=directory arm=bump_seg load=runs8 metadata_blocks=0 directory_runs_total=7304 segments_touched_mean_percent=1767 segments_touched_max=26 single_directory_chunks_32=25 single_directory_chunks_64=11 runs_median=7312 hint_fallback_seed0=0 fallback_pct=92.0
E7RESULT name=directory arm=arrival_by_request load=runs8 metadata_blocks=0 directory_runs_total=7275 segments_touched_mean_percent=1852 segments_touched_max=26 single_directory_chunks_32=19 single_directory_chunks_64=11 runs_median=7270 hint_fallback_seed0=0 fallback_pct=91.8
E7RESULT name=directory arm=directory_hint load=runs8 metadata_blocks=0 directory_runs_total=7887 segments_touched_mean_percent=1662 segments_touched_max=24 single_directory_chunks_32=0 single_directory_chunks_64=0 runs_median=7993 hint_fallback_seed0=40505 fallback_pct=0.0
E7RESULT name=directory arm=directory_home load=runs8 metadata_blocks=0 directory_runs_total=4310 segments_touched_mean_percent=192 segments_touched_max=2 single_directory_chunks_32=12 single_directory_chunks_64=0 runs_median=6555 hint_fallback_seed0=0 fallback_pct=0.0
E7RESULT name=verdict_directory arm=directory_hint load=runs8 metadata_blocks=0 first_fit_directory_runs=7956 hint_directory_runs=7887 hint_at_most_half=false first_fit_segments_touched_max=26 hint_segments_touched_max=24 hint_segments_touched_mean_percent=1662 hint_stays_in_few_segments=false first_fit_single_chunks_32=0 hint_single_chunks_32=0 hint_more_single_chunks=false first_fit_runs=7997 hint_runs=7993 key_runs_tied_within_5pct=true
E7RESULT name=verdict_directory arm=directory_home load=runs8 metadata_blocks=0 first_fit_directory_runs=7956 hint_directory_runs=4310 hint_at_most_half=false first_fit_segments_touched_max=26 hint_segments_touched_max=2 hint_segments_touched_mean_percent=192 hint_stays_in_few_segments=true first_fit_single_chunks_32=0 hint_single_chunks_32=12 hint_more_single_chunks=true first_fit_runs=7997 hint_runs=6555 key_runs_tied_within_5pct=false
E7RESULT name=directory arm=first_fit load=runs8 metadata_blocks=9 directory_runs_total=7960 segments_touched_mean_percent=1972 segments_touched_max=26 single_directory_chunks_32=0 single_directory_chunks_64=0 runs_median=7995 hint_fallback_seed0=0 fallback_pct=0.0
E7RESULT name=directory arm=bump_seg load=runs8 metadata_blocks=9 directory_runs_total=7357 segments_touched_mean_percent=1779 segments_touched_max=25 single_directory_chunks_32=25 single_directory_chunks_64=11 runs_median=7371 hint_fallback_seed0=0 fallback_pct=93.1
E7RESULT name=directory arm=arrival_by_request load=runs8 metadata_blocks=9 directory_runs_total=7360 segments_touched_mean_percent=1863 segments_touched_max=25 single_directory_chunks_32=22 single_directory_chunks_64=11 runs_median=7347 hint_fallback_seed0=0 fallback_pct=92.9
E7RESULT name=directory arm=directory_hint load=runs8 metadata_blocks=9 directory_runs_total=7804 segments_touched_mean_percent=1498 segments_touched_max=23 single_directory_chunks_32=0 single_directory_chunks_64=0 runs_median=7962 hint_fallback_seed0=35635 fallback_pct=0.0
E7RESULT name=directory arm=directory_home load=runs8 metadata_blocks=9 directory_runs_total=4315 segments_touched_mean_percent=192 segments_touched_max=2 single_directory_chunks_32=11 single_directory_chunks_64=0 runs_median=6559 hint_fallback_seed0=76 fallback_pct=12.2
E7RESULT name=verdict_directory arm=directory_hint load=runs8 metadata_blocks=9 first_fit_directory_runs=7960 hint_directory_runs=7804 hint_at_most_half=false first_fit_segments_touched_max=26 hint_segments_touched_max=23 hint_segments_touched_mean_percent=1498 hint_stays_in_few_segments=false first_fit_single_chunks_32=0 hint_single_chunks_32=0 hint_more_single_chunks=false first_fit_runs=7995 hint_runs=7962 key_runs_tied_within_5pct=true
E7RESULT name=verdict_directory arm=directory_home load=runs8 metadata_blocks=9 first_fit_directory_runs=7960 hint_directory_runs=4315 hint_at_most_half=false first_fit_segments_touched_max=26 hint_segments_touched_max=2 hint_segments_touched_mean_percent=192 hint_stays_in_few_segments=true first_fit_single_chunks_32=0 hint_single_chunks_32=11 hint_more_single_chunks=true first_fit_runs=7995 hint_runs=6559 key_runs_tied_within_5pct=false
E7RESULT name=directory_answer hint_halves_directory_runs_in_every_cell=false hint_more_single_chunks_in_every_cell=false home_halves_directory_runs_in_every_cell=false home_more_single_chunks_in_every_cell=false home_stays_in_few_segments_in_every_cell=true criterion=E151_H1_H3
E7RESULT name=grid arm=directory_hint load=uniform metadata_blocks=0 runs_median=8183 empty_segments_median=31 empty_peak_after_drain_median=0 drained_at_seed0=never rounds_with_empty_after_drain_seed0=0 write_amp=1.000 fallback_pct=0.0 metadata_fallback_seed0=0 container_writes_seed0=0
E7RESULT name=grid arm=directory_hint load=uniform metadata_blocks=4 runs_median=8183 empty_segments_median=29 empty_peak_after_drain_median=0 drained_at_seed0=never rounds_with_empty_after_drain_seed0=0 write_amp=1.000 fallback_pct=0.0 metadata_fallback_seed0=0 container_writes_seed0=0
E7RESULT name=grid arm=directory_hint load=uniform metadata_blocks=9 runs_median=8177 empty_segments_median=29 empty_peak_after_drain_median=0 drained_at_seed0=never rounds_with_empty_after_drain_seed0=0 write_amp=1.000 fallback_pct=0.0 metadata_fallback_seed0=0 container_writes_seed0=0
E7RESULT name=grid arm=directory_hint load=runs8 metadata_blocks=0 runs_median=7993 empty_segments_median=31 empty_peak_after_drain_median=0 drained_at_seed0=never rounds_with_empty_after_drain_seed0=0 write_amp=1.000 fallback_pct=0.0 metadata_fallback_seed0=0 container_writes_seed0=0
E7RESULT name=grid arm=directory_hint load=runs8 metadata_blocks=4 runs_median=7946 empty_segments_median=29 empty_peak_after_drain_median=0 drained_at_seed0=never rounds_with_empty_after_drain_seed0=0 write_amp=1.000 fallback_pct=0.0 metadata_fallback_seed0=0 container_writes_seed0=0
E7RESULT name=grid arm=directory_hint load=runs8 metadata_blocks=9 runs_median=7962 empty_segments_median=29 empty_peak_after_drain_median=0 drained_at_seed0=never rounds_with_empty_after_drain_seed0=0 write_amp=1.000 fallback_pct=0.0 metadata_fallback_seed0=0 container_writes_seed0=0
E7RESULT name=grid arm=directory_home load=uniform metadata_blocks=0 runs_median=8063 empty_segments_median=0 empty_peak_after_drain_median=0 drained_at_seed0=2 rounds_with_empty_after_drain_seed0=0 write_amp=1.000 fallback_pct=0.0 metadata_fallback_seed0=0 container_writes_seed0=0
E7RESULT name=grid arm=directory_home load=uniform metadata_blocks=4 runs_median=8066 empty_segments_median=0 empty_peak_after_drain_median=0 drained_at_seed0=2 rounds_with_empty_after_drain_seed0=0 write_amp=1.000 fallback_pct=5.8 metadata_fallback_seed0=7936 container_writes_seed0=0
E7RESULT name=grid arm=directory_home load=uniform metadata_blocks=9 runs_median=8066 empty_segments_median=0 empty_peak_after_drain_median=0 drained_at_seed0=2 rounds_with_empty_after_drain_seed0=0 write_amp=1.000 fallback_pct=12.3 metadata_fallback_seed0=17936 container_writes_seed0=0
E7RESULT name=grid arm=directory_home load=runs8 metadata_blocks=0 runs_median=6555 empty_segments_median=0 empty_peak_after_drain_median=0 drained_at_seed0=14 rounds_with_empty_after_drain_seed0=0 write_amp=1.000 fallback_pct=0.0 metadata_fallback_seed0=0 container_writes_seed0=0
E7RESULT name=grid arm=directory_home load=runs8 metadata_blocks=4 runs_median=6552 empty_segments_median=0 empty_peak_after_drain_median=1 drained_at_seed0=14 rounds_with_empty_after_drain_seed0=0 write_amp=1.000 fallback_pct=5.8 metadata_fallback_seed0=7936 container_writes_seed0=0
E7RESULT name=grid arm=directory_home load=runs8 metadata_blocks=9 runs_median=6559 empty_segments_median=0 empty_peak_after_drain_median=1 drained_at_seed0=15 rounds_with_empty_after_drain_seed0=0 write_amp=1.000 fallback_pct=12.2 metadata_fallback_seed0=17808 container_writes_seed0=0
```

字段口径：`directory_runs_total` = 每个目录成员按槽排序后的连续段数之和（初始 256，全散约 8192）；`segments_touched_mean_percent` = 每目录散在几个 64 槽段的均值 ×100；`single_directory_chunks_32 / 64` = 只住一个目录成员的 32 / 64 槽块数（初始 256 / 0）；`hint_fallback_seed0` = 提示找不到位置回落到减数臂的次数（种子 0，2000 轮 × 64 次分配）；`metadata_fallback_seed0` = 提交内生块落不进聚簇段的次数（分母 M × 2000）。

## 五、候选（跑前写死）

| 候选 | 条款 |
|---|---|
| **甲（倾向）** | D14 未定项 2 定「否」：第一版不做分配器提示，用户数据的落点全按 D3 已定项 8；判据栏写「无」；将来若做，机制只许「创建时继承一次、此后不再改」，且提示不得占用提交内生块的全空段供给（D3 已定项 5）——这两句作为将来重开的前置写进正文 |
| **乙** | 第一版做家固定的目录提示：对象创建时继承父目录的家段，落家内槽号最小的空槽，家满落溢出段（留 N₀ 个全空段给提交内生块），再满落全池；D3 已定项 8 第 1 条改成「家内槽号最小」 |
| **丙** | 维持未定，等真实负载的目录交错度（D25 负载表没有这一维） |

## 六、倾向与「结果反过来我接不接受」

**倾向甲**：① 唯一把目录钉住的臂（家固定）付的代价正是 D3 未定项 8 第二轮判掉甲′的那个量——提交内生块 99% 落不进聚簇段；② 段粒度局部性与删目录腾段今天没有条款消费；③ 家固定 = 创建时继承一次此后不改，与 2026-09-09 判掉 `locality_id` 时用的「掏空每次分配都可以重新判」是同一形态；④ 乙要改一条当日用户定案。
**反过来**：J1 打中（有消费者）且 J2 打中（溢出段有不吃提交内生块空段的来源）⇒ 乙回到候选表、第五次跑；J3 / J4 打中 ⇒ 乙出局，甲 / 丙由「有没有消费者」定。

## 七、要你们交什么

逐判据交判定，每格带附录出处或产物行；第 2 问要么给出溢出段的来源与大小，要么明说推不出；第 5 问判甲 / 乙 / 丙哪一个成立，判甲的话把「否」那条写成条款（含将来重开的两句前置）。
