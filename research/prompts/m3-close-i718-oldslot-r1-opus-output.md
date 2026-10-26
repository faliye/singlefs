# m3-close-i718-oldslot-r1 攻方腿（Opus）报告（2026-10-23）

立场：假设三个候选与第四节各格的倾向是错的，造合法历史去打。全部数在我自己的仓副本上量（副本上的数，不是入库装置上的数）。我自己提的改法（甲-任一圈、甲-任一读不出槽、丁甲、丁乙）只在我的模型上量过，被攻过零轮。

## 复跑命令与文件 sha256

```
bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m3-close-i718-oldslot-r1-opus-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs --work <新的草稿目录>
```

- `--frozen` 给仓根即可：rerun 只拷 `crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo`、`.claude/heavy-thread-default`，并打出副本里 walk.rs 的 sha256 与快照 `judged-sha256.txt` 里那一行比（`e30ca36b…`）。
- 挂钟：六格一个进程顺序跑约 6 分钟，一圈扫描约 8 分钟，回归 19 次约 2 分钟，调查员补丁那一份另编一次；合计约 20 分钟，超过子 agent 前台上限，我分 7 件并行跑过（下文「这条腿自己的限度」）。rerun 末尾打 `name=cells_condensed same_as=…`、`name=scan same_as=…`、`name=investigator_patch same_as=…`、`name=regressions same_as=…` 四行，任一行是 `DIFFERS` 就看它点名的 diff。
- 不碰 checker 档：只编跑 `singlefs-harness` 的测试目标与 `singlefs-checker --lib`。

```
7d3f5564d07c8b49c6b10a912f910c82f604ccb94be47616edeb2c303f290982  research/prompts/m3-close-i718-oldslot-r1-opus-model/out/cells-condensed.txt
e83536438795f64fed878e792cdc5b5eb3ddaf05c3fc876feaa4e25ba7204126  research/prompts/m3-close-i718-oldslot-r1-opus-model/out/investigator-patch-cells-a-b-condensed.txt
ea33df492bf007b703511d16ea1603f847bb17d46bb5b707c3551b8f4a8a3cef  research/prompts/m3-close-i718-oldslot-r1-opus-model/out/regressions.txt
ee2d9af41bee21becfd0bb7253775b54067e9d1097fb6fe1d97a4b55565dfd1c  research/prompts/m3-close-i718-oldslot-r1-opus-model/out/scan-one-revolution.txt
88074447563fdee83ba1a5b2df468a7a15e1c70b61cf95d21bfbe8ad19d6b549  research/prompts/m3-close-i718-oldslot-r1-opus-model/out/scan-tally.tsv
d234380682632c3e46cddf9f115837ff668776b28f77c7ae52eb91c684c67366  research/prompts/m3-close-i718-oldslot-r1-opus-model/tests/i718_oldslot_attack_probe.rs
81484476953929baf01e79d243f621a4add3e914cb9b96c47d2ba83a68a53ad4  research/prompts/m3-close-i718-oldslot-r1-opus-model/tools/condense.py
d01d5d28648998ed364d7ad27b99c9fb84da1658c3908fb0e11dab9f4e8077f7  research/prompts/m3-close-i718-oldslot-r1-opus-model/tools/investigator-prototype-i718-next-slot-by-its-old-content.patch
cfc30d32e0c778ab74547cea0042ff089a88f5eb1a37501c33858e8572a07678  research/prompts/m3-close-i718-oldslot-r1-opus-model/tools/rerun.sh
f9e579e5c901b6c0a2c82eae9b3ddc42b98f9304b6be050d09bef5a2d39950e3  research/prompts/m3-close-i718-oldslot-r1-opus-model/tools/tally.py
2751684b2bbbeb40d4b4be4f5553fe25e04aa128698380c2742dbd1b1a04a408  research/prompts/m3-close-i718-oldslot-r1-opus-model/tools/walk-candidate-switch-ding.py
e3cb973e32d46786f7b1eabad4ff2d42106ed8c3e98a7326bffce76c3001e480  research/prompts/m3-close-i718-oldslot-r1-opus-model/tools/walk-candidate-switch.py
b25a2727513b3864fca79cdba71b204649888d5f7aa538af3107e8a403501917  research/prompts/m3-close-i718-oldslot-r1-opus-model/tools/walk-helpers-only-for-the-investigator-patch.py
```

调查员的甲补丁原样拷进模型目录（sha256 `d01d5d28…`，与 `~/.local/share/singlefs/work/m3-close-b16-i718-baseline/` 里那一份同值），rerun 在另一份副本里原样打上它再判，不经我的开关。

## 各格判定一览

候选在副本 walk.rs 里装成开关（`tools/walk-candidate-switch.py`、`tools/walk-candidate-switch-ding.py`）：「今天/丙」= 判法不动；「甲」= 调查员补丁同义（另有原样打补丁那一份对照）；「乙」= 下一槽读不出自证过的根就报不适用（照正文字面，不设第一圈的闸）；「乙-过第一圈」= 乙加 N+1 > 3S 的闸。下面四个是我提的、零轮：「甲-任一圈」= 下一槽读不出、N+1 > 3S 时，旧内容取 N+1−k·3S（k ≥ 1）任一圈；「甲-任一读不出槽」= 任一读不出的槽按它那一族 txg 任取；「丁甲」= 甲-任一圈，另要 journal 里有最新根同一实例、txg N+1 的自证记录；「丁乙」= 乙-过第一圈，另要同一条记录。

| 格 | 判定 | 量的还是推的 | 一句话 |
|---|---|---|---|
| V1-甲放过的错 t | 打中（弱） | 量的 | 下一槽被写坏的镜像上甲收比真值高的 t（环洞形 t 写 15、真值 3，不安全方向）；同一条根在「根槽写之前崩」那张镜像上六个候选全红，靠兄弟态兜住 |
| V1-乙的判据只看镜像判不判得出 | 打中 | 量的 | 判得出「下一槽读不出」，判不出「下一槽是因为这次写失败才读不出」：第一圈从没写过的槽、挂载时钉住的槽都落进去；钉槽那一形上 I-7.18 条文点名「判得出」的两类坏实现全报不适用（11/11、12/12） |
| V1-乙第一圈 | 打中 | 量的 | 照正文字面不设闸：S = 4 的第一圈 9 张镜像里 8 张报不适用；入库坏镜像 `the_root_ring_health_invariant_reddens_when_the_previous_generation_is_gone_but_not_right_after_mkfs` 从 `["I-7.3", "I-7.18"]` 变成 `["I-7.3"]`、用例红 |
| V2-环洞落在下一槽（第二节没列） | 打中甲 | 量的 | 可写挂载崩在写行根槽之前留下的洞，一圈之后正好轮到下一次发布：旧内容是 N+1−2·3S 那条根，不是 N+1−3S。甲在合法镜像上红 36/36（一圈扫描 12 段 × 写 0 / 对半撕 / 枚举撕法），调查员补丁原样跑同红 |
| V2-挂载时钉住的槽轮到下一槽 | 打中乙 | 量的 | 合法镜像今天与甲都绿；乙报不适用，连带漏掉两类坏实现 |
| V2-第一圈从没写过的槽 | 打中乙（字面） | 量的 | 见 V1-乙第一圈 |
| V2-纯掉电撕裂 | 没打中新东西 | 量的 | 崩溃点重放枚举本来就产出撕裂态（原地覆写接一条撕裂镜像），照它的取法另写一份撕裂，结论与写 0、对半撕一样 |
| V2-撕裂域里别的槽 | 没打中 | 推的 | 槽距 ≥ 4096、逐槽按判定宽度对齐，一次槽写撕不到邻槽；入库用例 4 KiB 扇区边界撕裂那一格照绿 |
| V2-整盘丢失 | 不算 | 量的 | 去掉一块盘的镜像上 checker 另红 I-7.6 等七条，本身不当合法镜像判，不拿它判臂 |
| V3-甲对「写失败的槽按旧内容算」 | 打中 | 量的 | 甲把旧内容写死成 N+1−3S，那一句说的是旧内容；环洞形上两者不同，甲照字面就是与 D16 已定项 1 那一句打架 |
| V3-乙对 I-7.18 条文 | 打中 | 量的 | 条文写两类坏实现「判得出」，乙让它们在钉槽那一形上判不出 |
| V3-丙对 C560 / 崩溃点重放 | 代价 | 撕裂态量的，哪几条流满环是推的 | 丙就是 C560 写的「认成已知红」；撕裂态在崩溃点重放的枚举域里，凡转满一圈、建账发布之后那一槽是最旧根的流都要豁免 I-7.18 |
| V3-我提的甲-任一圈对 C863 | 打中自己 | 量的 | 钉槽那一形上「p 写 255 而还钉着」被收下（12/12），多出一类判不出的坏实现，与用户 C863「不认已认代价」打架；丁甲把它补回来（12/12 红） |
| V4-GPU | 推的 | 推的 | 甲、乙要一次 divmod64 加一遍根表；甲-任一圈可写成 O(1)；丁要 GPU 侧查 journal 记录，代价没量 |

## 装置

- 池：环洞池几何（`crates/singlefs-harness/tests/common_root_ring_holes/mod.rs:30`「pub const SLOTS_PER_REGION_OF_THE_RING_HOLE_POOLS: u64 = 4;」），环长 3S = 12，区域归盘 0 / 1 / 0，内存稀疏盘、每段历史从内存拷。只取这一种几何（限度一节）。
- 判器：副本 walk.rs 在 I-7.18 那一判（主工作区 `crates/singlefs-checker/src/walk.rs:8525`「newest_root.reclaim_threshold == recomputed_threshold,」）前面加开关；I-7.19 照原样判。开关由 `I718_CANDIDATE` 环境变量或探针里的 `set_i718_candidate_for_the_attack_probe` 选。
- 探针 `tests/i718_oldslot_attack_probe.rs` 七个测试：`cell_a`…`cell_f` 与一圈扫描 `scan_the_base_length_over_one_revolution`。每张镜像先打环、最新根、t、p、下一槽里的根，再逐个候选跑整份 `check_pool_image`、打 I-7.18 的判定与今天判法下判红的全部不变量。`tools/condense.py` 把它压成一张镜像一行，存在 `out/cells-condensed.txt`（78 行）。
- 改 t、p 的坏镜像只改根槽、不改那次发布的 journal 记录，所以坏镜像上 I-8.10 跟着红；有几张 I-3.1 也跟着红（账是按真 t 回收的）。这两样是我改字节的副产物，判臂只看 I-7.18。

## V1、V2：三种合法历史与三类坏写者

### 格 A（题面那一形，对照）

实例 1 写到 txg 14（环里 3–14），覆盖写 txg 15，它的根槽正是最旧那条 txg 3 住的槽；根槽写写 0、对半撕、照崩溃点重放的撕法撕（新旧不同那一截前一半新、后一半旧），恢复之前的镜像。`out/cells-condensed.txt` 原样一行：

```
cell=A image=root_slot_written_zero devices=[0,1] newest=(14, 1) t=3 p=255 F=0 next_slot_root=None today_or_bing=violated jia=holds yi_literal=not_applicable yi_after_first_lap=not_applicable jia_any_lap_of_next_slot=holds jia_any_unreadable_slot=holds ding_jia_with_next_record=holds ding_yi_with_next_record=not_applicable today_red=["I-7.18"]
```

今天红（现算 4）、甲绿、乙不适用，与调查员报的一致。崩溃点重放的枚举本来就产出这张撕裂态：`crates/singlefs-checker-tier/src/crash.rs:2329`「其后每个原地覆写的写接一条撕裂镜像」，取法在 `crates/singlefs-checker-tier/src/crash.rs:2309`「pub fn torn_image_of_in_place_overwrite(old: &[u8], new: &WrittenContents) -> Vec<u8> {」；我照它的文档另写了一份撕法（不拷代码），结论与写 0 相同。

### 格 B（第二节没列的历史：环洞落在下一槽）——打中甲

步骤（每一步许可它的那一句）：

1. 实例 1 覆盖写到 txg L（L 取 12–23 一圈）。
2. 实例 2 可写挂载，崩在写行那次根槽写之前：写行的单元与记录已落、根槽没落，那一槽留着上一圈的根（`crates/singlefs-harness/tests/common_root_ring_holes/mod.rs:207`「pub fn crash_a_mount_before_its_row_root(image: &MemoryPool) -> (MemoryPool, u64) {」，入库的环洞用例就是这么造洞；D16 已定项 1 那一行本来就点名「崩在记录之后、根槽之前那一形留在槽里的旧根」也在候选集里、也压着门槛，`.claude/kb/decisions/16-发布语义.md:36`「窗口之外的有效根（崩在记录之后、根槽之前那一形留在槽里的旧根）也在候选集里、也压着门槛」）。
3. 实例 3 可写挂载（写行、暖机），一直覆盖写到 N = 写行 txg + 11：洞那一槽一直没被写，里面是 L+1−3S 那条根（实例 1 的，按实例表有效）。
4. 覆盖写 N+1，它正落在洞那一槽；根槽写写 0 或撕开，进程在重发之前崩（与题面同一种故障）。

L = 14 那一段：洞在区域 0 槽 1，住着 txg 3；实例 3 从 txg 16 写到 26；N = 26 时写者算 t = 3（环里 3、16–26），N+1 = 27 正落在洞上。原样：

```
cell=B image=root_slot_written_zero devices=[0,1] newest=(26, 3) t=3 p=255 F=0 next_slot_root=None today_or_bing=violated jia=violated yi_literal=not_applicable yi_after_first_lap=not_applicable jia_any_lap_of_next_slot=holds jia_any_unreadable_slot=holds ding_jia_with_next_record=holds ding_yi_with_next_record=not_applicable today_red=["I-7.18"]
```

甲照 N+1−3S = 15 现算第二种，得 15；镜像上现算 16；写者写的 t = 3：两种都不等，判红。合法写者、单故障、在合法镜像上红。调查员补丁原样打上（不经我的开关）同红，`out/investigator-patch-cells-a-b-condensed.txt` 里 B 的四张：写 0、对半撕、枚举撕法三张红，根槽写之前那张绿。

一圈扫描（`out/scan-tally.tsv`，L = 12–23 共 12 段，每段写 0 / 对半撕 / 枚举撕法三张），B 那三行原样：

```
B	root_slot_torn_after_the_first_half	12	violated:12	violated:12	not_applicable:12	not_applicable:12	holds:12	holds:12	holds:12	not_applicable:12
B	root_slot_torn_like_the_crash_point_replay_enumeration	12	violated:12	violated:12	not_applicable:12	not_applicable:12	holds:12	holds:12	holds:12	not_applicable:12
B	root_slot_written_zero	12	violated:12	violated:12	not_applicable:12	not_applicable:12	holds:12	holds:12	holds:12	not_applicable:12
```

列序：今天/丙、甲、乙、乙-过第一圈、甲-任一圈、甲-任一读不出槽、丁甲、丁乙。由用户决定的那一步（实例 1 写到几）放开扫了一圈 12 个值，12 / 12 同向；洞之后实例 3 写几版不是自由的：要等到 N+1 落在洞上，这一形才出现，它之前的每张镜像（`cell_d` 的 9 张 whole 镜像）六个候选全绿。

四句：

- 分不分辨臂：分。甲红、乙不适用、丙红（已知红）、甲-任一圈与丁甲绿。
- 被判的系统当时看不看得到：看不到旧内容是哪一圈的根（那一槽已被写坏）。看得到 t 本身与 t ≡ N+1 (mod 3S)：甲-任一圈就是靠这一点（t 落在下一槽那一族 txg 里且低于现算）收下它。
- 满足的是判据哪一个分句：正文第四节 V2「还有哪些合法历史会让镜像上最旧那一槽不是建账时的那条根……三个候选各自罩不罩得住」——甲罩不住。不是 V1 的「放过错 t」，是反方向的误红。
- 跑前条款给的改法在这一格上还中不中：正文没有跑前改法。我提的甲-任一圈、丁甲在这一格上不中（量的，36 / 36 绿）。

### 格 F（第二节没列的历史：挂载时钉住的槽轮到下一槽）——打中乙，也打中我提的甲-任一圈

步骤：

1. 题面那一形的写 0 镜像（格 A，txg 15 的根槽写成 0）上，实例 2 可写挂载。那一槽读回全零，回收门槛钉在它可能住过的最旧 txg（`.claude/kb/decisions/16-发布语义.md:39`「挂载那一刻读坏（读不出，或读得出不是全零而自证不过）与读回全零的槽，回收门槛三处」），根记录带 t 与 p（同一行「钉也落盘：根记录带这次发布用的回收门槛 t 与钉槽 p」）。
2. 实例 2 覆盖写到 N = 26：之前没有一次发布落在那一槽，N+1 = 27 才轮到它。写者 t = 3、p = 3。
3. 镜像停在这里（没有在飞的发布），或者 N+1 在飞（记录已落、根槽写之前崩，或根槽写成 0）。

这一形上 N 那条根的每一张镜像里下一槽都读不出：根槽写之前那张、没有在飞的那张都一样，甲靠的「兄弟态」在这里不存在。

原样（`out/cells-condensed.txt`）：

```
cell=F image=newest26 devices=[0,1] newest=(26, 2) t=3 p=3 F=0 next_slot_root=None today_or_bing=holds jia=holds yi_literal=not_applicable yi_after_first_lap=not_applicable jia_any_lap_of_next_slot=holds jia_any_unreadable_slot=holds ding_jia_with_next_record=holds ding_yi_with_next_record=holds today_red=[]
cell=F-bad-t-as-floor image=newest26 devices=[0,1] newest=(26, 2) t=0 p=3 F=0 next_slot_root=None today_or_bing=violated jia=violated yi_literal=not_applicable yi_after_first_lap=not_applicable jia_any_lap_of_next_slot=violated jia_any_unreadable_slot=violated ding_jia_with_next_record=violated ding_yi_with_next_record=violated today_red=["I-8.10","I-7.18"]
cell=F-bad-p-255 image=newest26 devices=[0,1] newest=(26, 2) t=3 p=255 F=0 next_slot_root=None today_or_bing=violated jia=violated yi_literal=not_applicable yi_after_first_lap=not_applicable jia_any_lap_of_next_slot=holds jia_any_unreadable_slot=holds ding_jia_with_next_record=violated ding_yi_with_next_record=violated today_red=["I-8.10","I-7.18"]
```

两份坏写者正是 I-7.18 条文点名「判得出」的那两类：`crates/singlefs-checker/invariants/02-根环.md:25`「这一条判得出（后者现算的门槛不带钉、比 t 高，求等红」。一圈扫描（L = 12–23）F 的几行原样：

```
F	bad_p_255	12	violated:12	violated:12	not_applicable:12	not_applicable:12	holds:12	holds:12	violated:12	violated:12
F	bad_t_as_floor	12	holds:1 violated:11	holds:1 violated:11	not_applicable:12	not_applicable:12	holds:1 violated:11	holds:1 violated:11	holds:1 violated:11	holds:1 violated:11
F	legit	12	holds:12	holds:12	not_applicable:12	not_applicable:12	holds:12	holds:12	holds:12	holds:12
F-next-in-flight-bad-p-255	crash_before_the_root_slot_write	12	violated:12	violated:12	not_applicable:12	not_applicable:12	holds:12	holds:12	holds:12	not_applicable:12
```

`bad_t_as_floor` 那 1 段 holds 是 L = 23：钉值 0 = F，把 t 改成 F 字节不变，不是坏镜像（`out/scan-one-revolution.txt` 里 L = 23 那一行 t=0 p=0）。

- 乙（两种写法）：合法镜像 12 / 12 不适用，两类坏写者 11 / 11、12 / 12 不适用，一个都没判出来。乙只看「下一槽读不出」，钉住的槽与写失败的槽在这一谓词上相同。
- 甲：这一形上 N+1−3S 正好低于钉值以外的全部根，又不低于钉值，第二种现算与第一种相同，甲照判、两类都红（量的）。
- 我提的甲-任一圈、甲-任一读不出槽：`p 写 255 而还钉着` 那份 12 / 12 收下——t = 3 与下一槽那一族 txg 对得上，镜像上与「旧内容是 txg 3、这次写失败」逐字节相同（判别子观测不到）。多出一类判不出的坏实现，与 `.claude/kb/checks-owed.md:548`「两类不记成已认代价、另立一笔、要做成判得出的形态」那一条用户立场打架。
- 丁甲：要求 journal 里有 N+1 的记录才放宽。没有在飞的那张（F-bad-p-255）12 / 12 红；N+1 在飞那张（F-next-in-flight-bad-p-255）仍 12 / 12 收下——记录与写坏的槽两样在镜像上都在，同样判不出。它靠同一条根在「N+1 的记录落盘之前」那张兄弟镜像上判红（F-bad-p-255 那一行）兜住，这一兜是推的：同一个坏写者在两张镜像上写同一个 t、p。

四句（乙那一格）：分辨臂（乙不适用、今天与甲红）；系统看不看得到：看得到 p 指着下一槽（p = 3 = 下一槽的钉值），乙没用它；满足的分句：V1「乙的「不适用」判据是不是池级 checker 只看镜像就判得出来」——判得出谓词，判不出它想罩的那一态；改法：在乙上加「p 不指着下一槽」能救合法与 bad_t 两份，救不了 bad_p（p = 255，推的，没实现）。

### 格 E（第一圈从没写过的槽）——打中乙的字面

正文写的乙：`research/prompts/_m3-close-i718-oldslot-r1-body.md:29`「这一态（最新根之后那一槽读不出自证过的根）上 I-7.18 报不适用」，没有「已转过一圈」的闸。第一圈里 N+1 那一槽从没写过（mkfs 只在每区槽 0 种第 0 代），读回全零。S = 4 时新池写到 txg 3–11 的 9 张镜像，乙 8 张不适用，t 改成 t+1 的 9 张坏镜像乙 8 张不适用（今天与甲 9 / 9 红）。原样两行：

```
cell=E image=newest3 devices=[0,1] newest=(3, 1) t=0 p=255 F=0 next_slot_root=None today_or_bing=holds jia=holds yi_literal=not_applicable yi_after_first_lap=holds jia_any_lap_of_next_slot=holds jia_any_unreadable_slot=holds ding_jia_with_next_record=holds ding_yi_with_next_record=holds today_red=[]
cell=E-wrong-t-plus-one image=newest3 devices=[0,1] newest=(3, 1) t=1 p=255 F=0 next_slot_root=None today_or_bing=violated jia=violated yi_literal=not_applicable yi_after_first_lap=violated jia_any_lap_of_next_slot=violated jia_any_unreadable_slot=violated ding_jia_with_next_record=violated ding_yi_with_next_record=violated today_red=["I-8.10","I-7.18"]
```

入库用例照字面的乙会红（`out/regressions.txt` 原样）：

```
known_bad_ring_health candidate=yi test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 277 filtered out; finished in 0.21s  left: ["I-7.3"]
```

那条断言在 `crates/singlefs-harness/tests/checker_known_bad_images.rs:3030`「["I-7.3", "I-7.18"],」。推的、没跑（checker 档）：新池新建文件那条流在第一圈里，`crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs:245`「`I-7.18=9/0`」那 9 次评估会全变成不适用；GPU 对拍 `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:6584`「新池新建文件那次发布的根与记录里 t 写成 1」那份坏镜像也不再红。照字面的乙在第一版几何（S 更大）上丢得更多：第一圈多长就丢多少张。加闸（乙-过第一圈）之后这一格不中（量的：9 / 9 照判）。甲的补丁自带这道闸（`next_txg > ring_length_in_txgs`），正文对甲的定义没写；N+1 ≤ 3S 时「N+1−3S 那一条根」不存在。

### V1：甲的「两种现算之一」放过的错 t

两份坏写者，各取「下一槽写坏」与「根槽写之前崩」两张镜像（原样）：

```
cell=A-wrong-t-4 image=crash_before_the_root_slot_write devices=[0,1] newest=(14, 1) t=4 p=255 F=0 next_slot_root=Some((3, 1)) today_or_bing=violated jia=violated yi_literal=violated yi_after_first_lap=violated jia_any_lap_of_next_slot=violated jia_any_unreadable_slot=violated ding_jia_with_next_record=violated ding_yi_with_next_record=violated today_red=["I-8.10","I-7.18"]
cell=A-wrong-t-4 image=root_slot_written_zero devices=[0,1] newest=(14, 1) t=4 p=255 F=0 next_slot_root=None today_or_bing=holds jia=holds yi_literal=not_applicable yi_after_first_lap=not_applicable jia_any_lap_of_next_slot=holds jia_any_unreadable_slot=holds ding_jia_with_next_record=holds ding_yi_with_next_record=not_applicable today_red=["I-3.1","I-8.10"]
cell=B-wrong-t-row image=crash_before_the_root_slot_write devices=[0,1] newest=(26, 3) t=15 p=255 F=0 next_slot_root=Some((3, 1)) today_or_bing=violated jia=violated yi_literal=violated yi_after_first_lap=violated jia_any_lap_of_next_slot=violated jia_any_unreadable_slot=violated ding_jia_with_next_record=violated ding_yi_with_next_record=violated today_red=["I-8.10","I-7.18"]
cell=B-wrong-t-row image=root_slot_written_zero devices=[0,1] newest=(26, 3) t=15 p=255 F=0 next_slot_root=None today_or_bing=violated jia=holds yi_literal=not_applicable yi_after_first_lap=not_applicable jia_any_lap_of_next_slot=holds jia_any_unreadable_slot=holds ding_jia_with_next_record=holds ding_yi_with_next_record=not_applicable today_red=["I-3.1","I-8.10","I-7.18"]
```

- 「漏算下一槽里那条最旧根」的写者（t 写 4，真值 3）：写坏的镜像上今天的判法与甲都收下——这一类今天就判不出，不是甲新放的；根槽写之前那张上全红。
- 「把旧内容当 N+1−3S」的写者（环洞形 t 写 15，真值 3，比真值高，是会回收被 txg 3 那条有效根引用的单元的方向）：写坏的镜像上今天红、甲收下；这是甲在 V1 上真新放过的一格。根槽写之前那张全红。
- 收法的界：t 在 N 那次发布时定，N+1 的根槽写只能在它之后；只要「根槽写之前崩」那张兄弟镜像在判的范围里（崩溃点重放的枚举域里它在），同一条根就会在那张上被判。格 F 是兄弟镜像不存在的那一形，见上。
- 四句：分辨臂（今天红、甲收）；系统看不到真值 3（那一槽写坏了）；满足的分句：V1「甲的「两种现算之一」在哪些镜像上会把错的 t 判成对」；改法：甲-任一圈、丁甲同样收下（量的，B-wrong-t-row 那一行），兄弟镜像兜住这一点对它们同样成立。

### V2：别的形状

- 纯掉电：根槽写撕在半截而进程不报错，与「写失败」在镜像上同形；崩溃点重放的撕裂态就是它（格 A、B 的枚举撕法那一行），结论不变。
- 撕裂域里别的槽：根槽逐槽按判定宽度对齐（`crates/singlefs-checker/invariants/02-根环.md:12`「逐槽判，不是只判第 0 个」），槽距 ≥ 4096（`crates/singlefs-checker/invariants/02-根环.md:20`「固定结构槽距 ≥ 4096 字节」），一次槽写撕不到邻槽（推的）；入库用例 `crates/singlefs-harness/tests/root_record_carries_the_reclaim_threshold_and_the_pinned_ring_slot.rs:605` 那一格在八个候选下都过（`out/regressions.txt` 的 root_record_carries 八行，9 passed）。
- 挂载时读坏：格 F。
- 整盘丢失：`cell_c`（健康环）14 张、`cell_d`（洞在丢的那块盘上、下一槽在活着的盘上）9 张。今天判法 C 里 12 张、D 里 9 张红 I-7.18；但每一张同时红 I-7.6 等七条（`today_red=["I-1.1","I-2.1","I-3.1","I-4.8","I-7.2","I-7.4","I-7.6","I-7.18"]`），checker 不把直接去掉一块盘的镜像当合法镜像判，不拿它判臂。顺带：D 那 9 张丁甲也红，只有甲-任一读不出槽绿（它会把洞那一槽的 txg 4 也认）。
- 下一槽里是被抛弃的根：没造。推的：写者不算被抛弃的根，镜像上现算与 t 相同，今天就绿；甲多收一个更低的值，不误红。

## V3：与已定条款的关系

| 条款（kb 文件:行） | 原文（摘的那一段在那一行里） | 打架的候选 | 依据 |
|---|---|---|---|
| D16 已定项 1「环里最旧有效根」 | `.claude/kb/decisions/16-发布语义.md:36`「写失败的槽按旧内容算，在飞、没持久的发布不算」 | 甲 | 甲的定义把旧内容写死成「N+1−3S 那一条根」（`research/prompts/_m3-close-i718-oldslot-r1-body.md:28`「按它的旧内容（N+1−3S 那一条根）再现算一次」），格 B 里旧内容是 N+1−2·3S，甲照字面与这一句不一致（量的）。甲-任一圈、丁甲按「那一族 txg 里任一圈」取，不与它打架 |
| 同一行的六处同取 | `.claude/kb/decisions/16-发布语义.md:36`「checker 的候选过滤、checker 的 I-3.1（已分配统计对得上） 丁-defer 门槛」 | 三个候选都不碰 | 这两处在 checker 里同样读不到写坏那一槽的旧内容；格 A、B 的写坏镜像上 I-3.1 照样绿（今天的 today_red 只有 I-7.18），是 C773 并进「钉带出来的那一截」顺带罩住的，不是这一句兑现了（量的：A、B 的 today_red） |
| I-7.18 条文 | `crates/singlefs-checker/invariants/02-根环.md:25`「这一条判得出（后者现算的门槛不带钉、比 t 高，求等红」 | 乙、甲-任一圈、甲-任一读不出槽 | 格 F：乙让两类都不适用；甲-任一圈让「p 写 255」那一类收下（量的） |
| I-7.18 条文的不适用口径 | `crates/singlefs-checker/invariants/02-根环.md:25`「最新根下面没有记账树时报不适用」 | 乙 | 乙要在这里加第二个不适用的理由；照正文字面没有第一圈的闸，第一圈几乎整段不判（格 E） |
| C560 | `.claude/kb/checks-owed.md:351`「当合法崩溃状态判成立（例如候选集并进由这条记录施加出来的那一版，推的），还是认成已知红」 | 丙就是后一支 | 丙的代价：撕裂态在崩溃点重放的枚举域里（`crates/singlefs-checker-tier/src/crash.rs:2329`），凡转满一圈、最新根建账、下一槽住着最旧根的流都要豁免；格 B 那一形同样落进「已知红」。前一支「这条记录」正是丁拿来当见证的那条 |
| C863 | `.claude/kb/checks-owed.md:548`「这两类在镜像上与合法的钉逐字节相同（读故障不留在盘上）」 | 甲-任一圈、甲-任一读不出槽、乙 | 用户在同一行定「两类不记成已认代价」；这几个候选在格 F 上各多出判不出的坏实现（量的） |
| D16 已定项 7 | `.claude/kb/decisions/16-发布语义.md:207`「一次发布的持久顺序恒为 COW 单元 / 节点 → 屏障 → journal 记录 → 屏障 → 根槽（FUA）」 | 不打架，撑丁 | N+1 的根槽写能写坏那一槽时，N+1 的记录必已持久：丁要的见证在每一张「写坏」镜像上都在（格 A、B 的写坏镜像丁甲全绿，量的） |
| I-7.19 | `crates/singlefs-checker/invariants/02-根环.md:26`「两样故障叠在一起（读坏钉着的槽，这个进程对它的写报失败而实际落了盘）时这一条会误红，超出单故障」 | 没打中 | 我造的全部合法镜像上 I-7.19 没有一次红（today_red 里没有它，量的） |

## V4：两侧判器

- GPU 走读内核的判在 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:4606`「judge(INVARIANT_I_7_18, eq64(reclaim_threshold, recomputed_threshold));」。这份文件在开工快照之后被别的会话改过（`judged-sha256.txt` 那一行此刻对不上，我的副本里是快照那一版，同一句在第 4600 行）。
- 甲、乙都要判「N+1 那一槽有没有自证过的根」：N+1 的区域与槽要一次 64 位除法（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:542`「fn divmod64(a: vec2<u32>, d: vec2<u32>) -> QuotientRemainder {」），再扫一遍根表比区域与槽（`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:393`「fn checker_root_region(index: u32) -> u32 {」），至多 3S 圈，不加缓冲区（推的）。乙照现在「不判就是不适用」的写法提前返回。
- 甲-任一圈可以写成常数步：t 等于现算就成立；否则下一槽读不出、N+1 > 3S、t 低于现算，且 t > F 时 t ≡ N+1 (mod 3S) 且 t ≤ N+1−3S，t = F 时 (N+1) mod 3S ≤ F（推的，没实现、没量）。
- 丁要 GPU 侧知道 journal 里有没有（最新根的实例，N+1）那条自证记录。内核有施加记录的那一阶段，这一判能不能直接查到、要不要另记，没看（推的，代价没量）。
- CPU 与 GPU 对拍要加的盘：格 A、B 各三张写坏镜像（写 0、对半撕、枚举撕法）连同根槽写之前那张；格 F 的合法、t 写成 F、p 写 255，各连同 N+1 在飞的两张；格 E 第一圈连同 t+1；闸的边界 N+1 = 3S 与 N+1 = 3S + 1 各一张（这两张没造，推的）。

## 改法各修哪一格

每一格都是量过的（`out/cells-condensed.txt`、`out/scan-tally.tsv`、`out/regressions.txt` 原样），只有最后一行是推的。「收」= 坏写者被判成立。

| 改法 | A 写坏（合法） | B 写坏（合法） | F（合法） | F t 写成 F | F p 写 255 | F p 写 255、N+1 在飞 | E 第一圈 | 环健康坏镜像用例 | B t 写 15（写坏那张） |
|---|---|---|---|---|---|---|---|---|---|
| 今天 / 丙 | 红 | 红 | 绿 | 红 | 红 | 红 | 判 | 过 | 红 |
| 甲 | 绿 | 红 | 绿 | 红 | 红 | 红 | 判 | 过 | 收 |
| 乙（字面） | 不适用 | 不适用 | 不适用 | 不适用 | 不适用 | 不适用 | 8/9 不适用 | 用例红 | 不适用 |
| 乙-过第一圈 | 不适用 | 不适用 | 不适用 | 不适用 | 不适用 | 不适用 | 判 | 过 | 不适用 |
| 甲-任一圈（零轮） | 绿 | 绿 | 绿 | 红 | 收 | 收 | 判 | 过 | 收 |
| 甲-任一读不出槽（零轮） | 绿 | 绿 | 绿 | 红 | 收 | 收 | 判 | 用例红 | 收 |
| 丁甲（零轮） | 绿 | 绿 | 绿 | 红 | 红 | 收 | 判 | 过 | 收 |
| 丁乙（零轮） | 不适用 | 不适用 | 绿 | 红 | 红 | 不适用 | 判 | 过 | 不适用 |
| 乙加「p 不指着下一槽」（推的） | 不适用 | 不适用 | 判、绿 | 红 | 不适用 | 不适用 | 看闸 | 看闸 | 不适用 |

丁甲在这张表上只剩两格收：一格（F N+1 在飞、p 写 255）靠记录落盘之前那张兄弟镜像兜住（推的：同一写者两张镜像写同一 t、p），一格（B t 写 15）靠根槽写之前那张兜住（量的：那张全红）。丁甲只在我的模型上量过、被攻过零轮。

## 哪一条最脆弱

甲定义里那个括注「（N+1−3S 那一条根）」：一段合法历史（格 B，环洞之后一圈正好轮到）就把它打穿，调查员补丁原样也红。乙的代价不是误红而是判别力：照字面第一圈几乎整段不判，加了闸也在钉槽那一形上把条文点名的两类坏实现一起放掉。丙不改判器，代价是已知红要在崩溃点重放的撕裂态上长期豁免，而且格 B 也落进去。三条各有一格打中，按「三轮里多数打穿才算」挂起。

## 没打中的形状

- 撕裂域里别的槽（推的，理由见 V2）。
- I-7.19 在我造的全部合法镜像上没有一次误红（量的）。
- 下一槽里是被抛弃的根（没造，推的：不误红）。
- 整盘丢失：打出了 I-7.18 红，但同一张镜像另红七条，不算合法镜像，不计。
- 取样范围：环长 12 的一种几何；实例 1 写到几那一步放开扫一圈 12 个值（12–23）；每段取写 0、对半撕、枚举撕法三种写坏，加「根槽写之前崩」一张；格 C 新池写到 13–19 各丢两块盘之一（14 张）；格 D 洞之后每一版（19–27）各一张完整、一张丢盘 1；格 E 第一圈 3–11 各一张加 t+1；格 F 钉槽轮到下一槽那一条根加两份坏写者、加 N+1 在飞两张。扫描合计 180 行（`out/scan-one-revolution.txt`），六格合计 78 张（`out/cells-condensed.txt`）。

## 这条腿自己的限度

- 几何只取环洞池一种（S = 4、R = 3、区域归盘 0 / 1 / 0、512 字节根槽）；第一版几何（S 更大）没量，格 E 的丢失量是 S 的函数，没扫。
- 下一次发布只取覆盖写（带单元、建账）；N+1 是零单元发布、正常卸载那一串、管理员回退时没造（回退那一形是题面场景本身，调查员量过）。
- 崩溃状态是我挑的三四张，没有调 `enumerate_crash_states` 把每段历史的写表整段展开：那一入口要 checker 档的判定存储与流库，两小时里搭不起来。撕裂态照 `torn_image_of_in_place_overwrite` 的文档另写了一份，结论与写 0、对半撕一致；其余子集态（单元、记录各落一部分）没枚举。这与定义第 3b 步「崩溃状态一个不落」不符，照实记在这里。
- 坏写者是改根槽字节造的，记录没跟着改，I-8.10 与部分 I-3.1 是副产物；真坏写者记录与根同写，那两条不会跟着红。
- 丁的「记录在不在」是在副本 walk.rs 调用处现扫 journal、经线程局部变量传进判里，原型写法，不是实现形态；两块盘的 journal 任一块有就算。
- 探针源码第一段一次写了 154 行，超过「每次不超过 150 行」那一条 4 行（照实记）。
- 量的数全在副本上，不是入库装置上的数；判决要引得在入库装置上重做。
- 预计挂钟：先跑 2 段估一圈 12 段约 160 秒（六个候选）；加到八个候选、每段多两张之后分 3 件、再分 7 件并行跑，每件 35–175 秒，没缩取样。

## 没做什么

- 没跑 checker 档（崩溃点重放、GPU 对拍、`crash_enumeration_*` 流）；第一圈那条流「`I-7.18=9/0`」会怎么变是推的。
- GPU 一侧一个候选都没写，V4 全是推的。
- 「乙加 p 不指着下一槽」只推了、没实现。
- 不判正推腿那几格，不替主 agent 采纳。

## 要主 agent 跑的命令

- 复跑（约 20 分钟，放后台）：`bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m3-close-i718-oldslot-r1-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs --work <新的草稿目录>`，工作目录随意；线程数默认 31（`RERUN_THREADS` 可改）；看末尾四行 `name=… same_as=…`。rerun.sh 我只分步试过拷树、两道开关与打补丁那几步（与我跑的副本逐字节相同），整条没在前台跑完。
- 核引文：`python3 scripts/agents/cite-check.py research/prompts/m3-close-i718-oldslot-r1-opus-output.md`。

## 清理

交回前删掉的（删之前 `du -sh`）：仓副本 `repo` 37M、打调查员补丁的副本 `repo-jia-patch` 37M、编译目录 `target` 3.5G、`target-jia-patch` 1.6G、对照目录 `verify` 1.1M，都在 `/home/fy5090/.local/share/singlefs/work/m3-close-i718-oldslot-r1-opus/` 下；那里只留原始输出与日志（1016K）。

## 引文核对

`python3 scripts/agents/cite-check.py research/prompts/m3-close-i718-oldslot-r1-opus-output.md` 末行（交回前现跑）：

```
  ✓ 核了 28 处引文、2 处裸行号，对上 28 处，没判 2 处（其中裸行号没有引文可比 2 处；逐处列在上面）
```
