# D16（发布语义） 未定项 1 第三轮——反推腿（云端 Opus）答复

**口径**：2026-09-11（UTC），反推立场，材料 `research/prompts/_d16-item1-r3-background.md`（下称「材料」，行号指这份文件）。
标 **【我核过】** 的，要么是我当场读了原文件，要么是在 E138（按盘回退下界与推空的空间要求） 源码副本上跑出来的；标 **【我推的】** 的没有观测撑着。
除了这份文件，仓里我一个字都没改。

**副本与复跑**（都在会话 scratchpad `/tmp/claude-1000/-home-fy5090-code-singlefs/86b2ac0e-7e65-418c-97a7-f3d755730fe6/scratchpad/`，下称 `$SP`）：

- `$SP/opus-copy/research/` 是 `research/` 的拷贝（rsync 时排除了 `target`）。装置文件是 `e7-index-bench/src/bin/r3_attack.rs`：先把 `e138_per_disk_floor.rs` 原样拷过去，再用三个补丁脚本（`$SP/patch_r3.py`、`patch_r3b.py`、`patch_r3c.py`）改。每处替换都断言命中次数为 1，没命中就退出。
- **模拟器本体只做了加法**，谓词、损坏记法、候选集判法、恢复、抬 F 这几块一行没改。加的是：推空模式（原样 / 做到生效 / 一律做满 7 次）、推空次数上限可覆写、在某次准入的第 k 次根槽写上注入失败、「坏槽」（同一个槽以后每次写都失败）、空发布与非空发布的元数据开销在运行中途变大、准入可以按 `df_guaranteed` 截断、在用户视角记一次 `df`（删完对象、处置之前那一刻，字段 `false_user`）、只在准入成功时记发布数（`max_pubs_successful`）、在 ENOSPC 那一刻把环倒出来。驱动 `main` 另写，在 `$SP/r3_main.rs`、`$SP/r3_more.rs` 和 `patch_r3c.py` 里。
- **基线复现**：驱动 d1 用补丁后的装置重跑 E138 的 `near_full`，臂 A 与臂 G 的几格和 E138 产物逐数相同（强制发布 ×100 分别是 886 / 899 / 3622 / 3928，臂 G 在 c = 5 时 `wad_failed=24`），说明补丁没有改动基线行为。
- 复跑：`cd $SP/opus-copy/research && cargo build --release --bin r3_attack && ./target/release/r3_attack d1`（依次换成 d2 / d2b / d3 / d3b / d4 / d5 / d6 / d7 / d8），原始输出在 `$SP/out_d*.txt`。这是确定性模型，只跑了一遍；证据强度来自每组里的对照格，不来自轮数。

---

## 一、判据 3-1（正确性）：两条臂都没打中

**试了什么**（全部【我核过】，下面各格的 `fakes` 与 `unrecoverable` 都是 0）：

| 驱动 | 构造 | 臂 A | 臂 G / G′ | 对照：判别力 |
|---|---|---|---|---|
| d4 | 活化区间里崩溃、**不丢盘**（E138 答不了的第 8 条）→ 恢复 → 再跑 1..4 个窗口 → 分配完、写根之前崩溃，并丢盘 0 或盘 1；16 种写失败组合 × 8 个崩溃窗口 × 8 个种子 | 0 / 8192 个 case | G：0 / 8192 | G_count 52 个 case 出假候选，臂 D 119 个 ⇒ 这个世界抓得到按次数活化、按 txg 活化那两种错 |
| d5 | **先写失败跳号、再抬 F**（E138 答不了的第 9 条）：在区域 1 连续失败 1 / 2 / 3 次之后，把 F 抬到上限，再照 E138 的 `activation_faults` 做全枚举 | 0 | G：0，保留违例也是 0 | **G_txgcap 保留违例 2096**（E138 在这一格是 0，判「未验」）⇒ 在副本里，「按盘取上限」那一半第一次被测到，臂 G 过了 |
| d2 / d2b | 近满盘上，推空期间注入一次根槽写失败（第 0..N 次都试），坏槽再重复 0..2 次 | 0 | G、G′ 两种读法：0 | — |
| d7 | 占用 70% 时让一个根槽从此写不进（见第二节 R2） | 0 | G、G′：0 | — |

⇒ 臂 A 在这个模型里零假候选，理由和第二轮 H3 一样：界与候选集取自同一组根，所以零多半是构造出来的，不是经验发现。坏槽之下它照样是 0，因为那个陈旧的根照样受保护——**可空间代价全落到了 3-2 / 3-3 上**（第二节）。
⇒ G′ 与 G 只在推空那一段不同，活化与恢复的逻辑完全一样，所以 d4、d5 的结论对 G′ 同样成立。

**journal 重放窗口，没打中【我推的】**：所选根 R 之后的记录点名的单元生在 u > R；要让它被复用，得先释放在 Q ≤ 界 ≤ R 的某一代，而 R < u ≤ Q，两者矛盾。臂 A、G′ 都一样。第二轮 H11 说的「重放期间要分配」那个洞各臂都有，归 C284（施加一条记录在指针层上做什么没有定义），不分臂。

**挂载内实例切换**：模型只建了「txg 推进一格、写失败的槽留着旧根」这一半。按 D23（journal 的角色与格式） 已定项 14 索引行，切换要写的行 `(旧实例, 最后发布的 txg, W)` 只会切掉更新的根，陈旧根的有效性不变，所以这一半的结论照搬得过去【我推的】。切换自己那份预留见第二节 R4。

**d6（推空中途撕根、崩溃）不采纳**：臂 A 与 G′ 都有「假性 ENOSPC」，但每个 run 约 1 次，数的正是崩溃那一刻还在飞的那次写。崩溃之后这次写本来就不会返回，这是装置的人工痕迹，不是发现。

---

## 二、判据 3-2 / 3-3（D3（空间分配） 已定项 9 的两条）

### R1　推空期间一次根槽写失败 ⇒ 臂 A 报 ENOSPC 时 `df` 还有 11 块；界 N 不成立，要 2N − 1【我核过】

**机制**：D23（journal 的角色与格式） 已定项 14 规定「根槽写失败重发时 checkpoint_txg 推进一格再发」；C282（环里最旧根没有定义） 的定案形态逐字是「**写失败的槽按旧内容算**」（`.claude/kb/checks-owed.md:274`）。
设推空里第 j 次空发布（txg t + j）的根槽写失败，那个槽里就留着 t + j − N（< t）这个自证合法、实例有效的根，直到 txg t + j + N 把它覆盖。臂 A 的界是「环里已持久的最旧有效根」，于是被卡在 t + j − N，t 那一代的释放放不回来。
E138 的推空上限是 N − 1 次空发布（源码 `push_after_commit`），做满也放不回，只能报 ENOSPC；而 `df_guaranteed` 只扣一个常数，照样报 ≥ 8。

**具体一例**（d8a：臂 A，S = 4，c = 5，种子 0，第 50 个窗口之后的第一次准入里，第 1 次空发布的根槽写失败）。整行抄 `$SP/out_d8.txt`：

```text
DUMP enospc: df_user=11 df_guaranteed=11 df_raw=131 pinned=118 reusable=13 reserve=65 residue=55 reuse_bound=820 newest_txg=843 publications_this_admission=12 ring=[0:840 1:843 2:834 3:837 4:841 5:820 6:835 7:838 8:842 9:833 10:836 11:839]
D8a A s=4 c=5 seed=0 offset=1 false_guaranteed=1 false_user=1 true_enospc=1 wad_failed=1 max_pubs=12 stalls=0 halted_before_counting=0 fakes=0 unrecoverable=0 retention=0 max_pinned_after_push=118 injected=1 forced_per_window_x100=391 checks=1252 min_candidates=12 avg_nonempty_x100=360 max_pubs_successful=12
```

槽 5 里是 820，其余槽都在 833..843 之间，`reuse_bound=820` 就是它；用户那一刻看到 `df` 11 块（要 8 块），写却失败了。这个窗口先删了一个对象，所以 3-3 同时违反。

**扫一遍落点**（d2，每个 offset 4 个种子）。臂 A 默认形态下，失败落在第 1..N − 1 次空发布时，每个种子都中；落在第 0 次（发布正在攒的窗口本身）或第 N 次（已经在推空之外）不中。整行抄 `$SP/out_d2.txt`：

```text
D2 cfg=A s=4 c=5 repeats=0 offsets=8 seeds=4 bad_runs=24/32 worst_offset=0 worst_pubs=12 false_guaranteed=24 false_user=24 true_enospc=32 wad_failed=24 max_pubs=12 stalls=0 halted_before_counting=0 fakes=0 unrecoverable=0 retention=0 max_pinned_after_push=118 injected=32 forced_per_window_x100=437 checks=43804 min_candidates=12 avg_nonempty_x100=327
D2 cfg=A s=16 c=5 repeats=0 offsets=8 seeds=4 bad_runs=24/32 worst_offset=0 worst_pubs=48 false_guaranteed=24 false_user=24 true_enospc=32 wad_failed=24 max_pubs=48 stalls=0 halted_before_counting=0 fakes=0 unrecoverable=0 retention=0 max_pinned_after_push=478 injected=32 forced_per_window_x100=2161 checks=151912 min_candidates=48 avg_nonempty_x100=349
```

**把推空上限放开，界是多少**（d2b，上限 5N，只在准入成功时记发布数）。整行抄 `$SP/out_d2b.txt`：

```text
D2b cfg=A_cap5N s=4 n=12 c=5 repeats=0 offsets=13 seeds=2 worst_offset=11 false_guaranteed=0 false_user=0 true_enospc=26 wad_failed=0 max_pubs=61 stalls=0 halted_before_counting=0 fakes=0 unrecoverable=0 retention=0 max_pinned_after_push=55 injected=26 forced_per_window_x100=590 checks=45762 min_candidates=12 avg_nonempty_x100=258 max_pubs_successful=23
D2b cfg=A_cap5N s=16 n=48 c=5 repeats=0 offsets=49 seeds=2 worst_offset=47 false_guaranteed=0 false_user=0 true_enospc=98 wad_failed=0 max_pubs=241 stalls=0 halted_before_counting=0 fakes=0 unrecoverable=0 retention=0 max_pinned_after_push=235 injected=98 forced_per_window_x100=2868 checks=610836 min_candidates=48 avg_nonempty_x100=270 max_pubs_successful=95
```

⇒ **一次写失败时，一次准入最多要 2N − 1 次发布**（N = 12 时 23，N = 48 时 95），最坏的落点是第 N − 1 次空发布，与上面的机制推导逐数相符。（`max_pubs=61/241` 是填满那一刻真 ENOSPC 时推到上限的次数，不算数。）
**同一个槽再失败一次**（repeats = 1，下一圈写到它时又失败）：放开上限的臂 A 也 **26 / 26、98 / 98 个 run 全部卡死**（`stalls=26`、`stalls=98`）——跑前的保留池只够 N − 1 次空发布。

**G′ 的对照**（同一个 d2 / d2b）：「做满 7 次」在上限 6 次空发布时同样会中（S = 4、c = 5 时 8 / 36 个 run，`Gp_full_seven` 那一行 `false_guaranteed=8 … wad_failed=8`）。放开上限之后，1..3 次失败要 8 或 9 次发布，与 N 无关：

```text
D2b cfg=Gp_full_seven_cap40 s=16 n=48 c=1 repeats=2 offsets=9 seeds=2 worst_offset=0 false_guaranteed=0 false_user=0 true_enospc=18 wad_failed=0 max_pubs=9 stalls=0 halted_before_counting=0 fakes=0 unrecoverable=0 retention=0 max_pinned_after_push=12 injected=54 forced_per_window_x100=327 checks=20550 min_candidates=5 avg_nonempty_x100=717 max_pubs_successful=9
D2b cfg=Gp_full_seven_cap40 s=16 n=48 c=5 repeats=2 offsets=9 seeds=2 worst_offset=3 false_guaranteed=0 false_user=0 true_enospc=18 wad_failed=0 max_pubs=8 stalls=4 halted_before_counting=0 fakes=0 unrecoverable=0 retention=0 max_pinned_after_push=88 injected=50 forced_per_window_x100=314 checks=18400 min_candidates=5 avg_nonempty_x100=758 max_pubs_successful=8
```

（c = 5 那一格还有 4 / 18 个 run 卡死：保留池 10 + 6c 装不下第 8 次发布，要加一两个 c。）
⇒ **两条臂写下来的界都被一次写失败打破**；G′ 的补法是有界的常数（+2 次发布、+约 2c 的保留池），臂 A 的补法每容忍一次失败就要多 N − 1 次发布，外加相应的保留池。

⚠️ **射程**：如果主 agent 认定「故障期间报出的 ENOSPC 不算假性」，R1 这一格降为「记下」。但 D23（journal 的角色与格式） 已定项 14 把瞬时写失败放进实例切换，「不转只读、不等下次挂载」，按条款它对用户是不可见的；报 ENOSPC 的是后面那次正常的用户写，不是出错的那次 I/O。R2 不受这条射程影响。

### R2　一个根槽从此写不进 ⇒ 盘只占 70%，臂 A 在 `df` 报 1171 块时报 ENOSPC，下一个窗口卡死；G / G′ 跑满 1000 个窗口不受影响【我核过】

**为什么这个世界在条款允许的范围之内**（引文【我核过】，`.claude/kb/decisions/23-journal的角色与格式.md:681`）：
- 判别瞬时还是持续失败，靠的是「对**目标设备的固定落点**做一次探针写——写得进去 ⇒ 瞬时失败，走实例切换」。探针写的是固定落点，不是出错的那个槽，所以槽所在扇区坏了、别处都好的时候，每次都会判成瞬时失败。
- 同一行逐字：「⚠️ 由此『同一个根槽连续失败』**不再是一条判据**——槽位随 txg 轮转，N_switch ≤ R 时它永远够不着」。坏槽每 N 次发布才被写到一次，中间隔着 N − 1 次成功的发布，连续切换计数够不着 3。
- 写失败的槽「按旧内容算」（C282（环里最旧根没有定义））；实例表的行只会切掉更新的根。⇒ **槽里最后一次写成功的那个根，永远自证合法、实例有效，永远是臂 A 候选集里最旧的那个**——而把它放进候选集的正是 D23（journal 的角色与格式） 已定项 14（用户定案）的原话「候选集 = 根环里按实例表判仍然有效的根」。

**构造**（d7）：占用 70%，预热 2N + 10 个窗口，之后下一次根槽写失败，而且那个槽此后每次写都失败；再做 1000 个「删一个、建一个」的窗口。臂 A 的界从此冻住，每个窗口的 13 块全被扣住。整行抄 `$SP/out_d7.txt`：

```text
D7 cfg=A s=4 c=1 fill=70% windows=1000 seeds=8 first_enospc_window=[79, 79, 79, 79, 79, 79, 79, 79] false_guaranteed=8 false_user=8 true_enospc=0 wad_failed=8 max_pubs=12 stalls=8 halted_before_counting=0 fakes=0 unrecoverable=0 retention=0 max_pinned_after_push=1216 injected=72 forced_per_window_x100=20 checks=768 min_candidates=12 avg_nonempty_x100=1079 max_pubs_successful=0 | seed0_end: live_objects=348 pinned=1216 df_raw=1216 df_guaranteed=1180 reuse_bound=23 newest_txg=139 halted=true
D7 cfg=A s=16 c=5 fill=70% windows=1000 seeds=8 first_enospc_window=[26, 26, 26, 26, 26, 26, 26, 26] false_guaranteed=8 false_user=8 true_enospc=0 wad_failed=8 max_pubs=48 stalls=8 halted_before_counting=0 fakes=0 unrecoverable=0 retention=0 max_pinned_after_push=1215 injected=16 forced_per_window_x100=175 checks=600 min_candidates=48 avg_nonempty_x100=3234 max_pubs_successful=0 | seed0_end: live_objects=348 pinned=1215 df_raw=1216 df_guaranteed=736 reuse_bound=59 newest_txg=183 halted=true
D7 cfg=A_cap5N s=16 c=5 fill=70% windows=1000 seeds=8 first_enospc_window=[18446744073709551615, 18446744073709551615, 18446744073709551615, 18446744073709551615, 18446744073709551615, 18446744073709551615, 18446744073709551615, 18446744073709551615] false_guaranteed=0 false_user=0 true_enospc=0 wad_failed=0 max_pubs=49 stalls=8 halted_before_counting=0 fakes=0 unrecoverable=0 retention=0 max_pinned_after_push=1207 injected=16 forced_per_window_x100=181 checks=600 min_candidates=48 avg_nonempty_x100=3233 max_pubs_successful=0 | seed0_end: live_objects=349 pinned=1207 df_raw=1208 df_guaranteed=728 reuse_bound=59 newest_txg=183 halted=true
D7 cfg=G s=16 c=5 fill=70% windows=1000 seeds=8 first_enospc_window=[18446744073709551615, 18446744073709551615, 18446744073709551615, 18446744073709551615, 18446744073709551615, 18446744073709551615, 18446744073709551615, 18446744073709551615] false_guaranteed=0 false_user=0 true_enospc=0 wad_failed=0 max_pubs=7 stalls=0 halted_before_counting=0 fakes=0 unrecoverable=0 retention=0 max_pinned_after_push=30 injected=192 forced_per_window_x100=8 checks=8656 min_candidates=5 avg_nonempty_x100=3490 max_pubs_successful=7 | seed0_end: live_objects=350 pinned=35 df_raw=1195 df_guaranteed=1125 reuse_bound=1204 newest_txg=1212 halted=false
D7 cfg=Gp_full_seven s=16 c=5 fill=70% windows=1000 seeds=8 first_enospc_window=[18446744073709551615, 18446744073709551615, 18446744073709551615, 18446744073709551615, 18446744073709551615, 18446744073709551615, 18446744073709551615, 18446744073709551615] false_guaranteed=0 false_user=0 true_enospc=0 wad_failed=0 max_pubs=7 stalls=0 halted_before_counting=0 fakes=0 unrecoverable=0 retention=0 max_pinned_after_push=30 injected=192 forced_per_window_x100=8 checks=8672 min_candidates=5 avg_nonempty_x100=3484 max_pubs_successful=7 | seed0_end: live_objects=350 pinned=35 df_raw=1195 df_guaranteed=1125 reuse_bound=1204 newest_txg=1214 halted=false
```

（`18446744073709551615` 表示一次 ENOSPC 都没有。）S = 4、16 与 c = 1、5 共四格，臂 A 全都是 8 / 8 个种子先报一次假性 ENOSPC、再卡死；把推空上限放开的臂 A 不报 ENOSPC，直接卡死；G、G′ 四格全是 0，每个种子吸收了 23..98 次坏槽写失败（`injected=184..784`）。
单个种子的时间线（d8b，臂 A，S = 4，c = 1）。整行抄 `$SP/out_d8.txt`：

```text
DUMP enospc: df_user=1167 df_guaranteed=1171 df_raw=1207 pinned=1198 reusable=9 reserve=21 residue=15 reuse_bound=23 newest_txg=134 publications_this_admission=12 ring=[0:132 1:123 2:126 3:129 4:133 5:124 6:127 7:130 8:134 9:125 10:128 11:23]
D8b window=79 outcome=NoSpace pinned_before=1170 pinned_after=1198 df_guaranteed=1171 live_objects=349 bad_slot=Some(11)
D8b window=80 outcome=Halted pinned_before=1198 pinned_after=1216 df_guaranteed=1180 live_objects=348 bad_slot=Some(11)
```

槽 11 里一直是 txg 23，其余槽已经到了 123..134。盘上用户数据只有 348 × 8 块（约 70%），`df` 报 1171 块（容量的 29%），写 8 块报 ENOSPC；下一个窗口连发布正在攒的窗口都分配不到，卡死——那是 D23（journal 的角色与格式） 的死锁 2，比 ENOSPC 还糟。
⇒ **3-2、3-3 双输，而且没有任何有限的 `df` 扣除量或推空上限救得回来**：只要那个槽不恢复，被扣住的块就随写入量无界增长。

**臂 A 能不能自己修【我推的】**：唯一的出路是把那个陈旧的根从候选集里拿掉。这有两重代价：① 改的正是 D23（journal 的角色与格式） 已定项 14 的候选集那一句（用户定案，3-5 要用户重判）；② 崩溃之后恢复得知道它被拿掉了，就要一个落盘的标记——那就是动态回退下界的 F，8 字节。修完之后，材料倾向臂 A 的理由 ②「不改盘上字节」、③「不碰 D23 已定项 14」一起失效。G / G′ 靠用户已经授权的 P2（盘紧时候选集可以一直缩）把陈旧根挤到 F 以下，所以不受影响。

### R3　空发布开销在挂载之后变大：两条臂都输，但臂 A 的暴露面按 N 放大【我核过】

**构造**（d3）：保留池与残留按挂载时的 c0 算，填满之后第 100 个窗口起，空发布开销与非空发布的元数据各加 b 块（b ∈ {1, 3, 5}，模拟树高一层）。E81（提交的固定点） 给出的量级是：3 层树、12 单元的 fsync，聚簇之后写 5 个元数据块，只聚簇元数据时 28 块，所以 c 在 5 附近、树每长一层加一块左右是有依据的量级【口径我核过，外推是我推的】。整行抄 `$SP/out_d3.txt`：

```text
D3 cfg=A s=16 c0=5 bonus=5 formula=at_mount mount_reserve=245 mount_residue=235 seeds=8 false_guaranteed=232 false_user=240 true_enospc=16 wad_failed=240 max_pubs=48 stalls=0 halted_before_counting=0 fakes=0 unrecoverable=0 retention=0 max_pinned_after_push=470 injected=0 forced_per_window_x100=3704 checks=118456 min_candidates=48 avg_nonempty_x100=220
D3 cfg=A s=16 c0=1 bonus=3 formula=at_mount mount_reserve=57 mount_residue=51 seeds=8 false_guaranteed=0 false_user=0 true_enospc=8 wad_failed=0 max_pubs=48 stalls=8 halted_before_counting=0 fakes=0 unrecoverable=0 retention=0 max_pinned_after_push=235 injected=0 forced_per_window_x100=2131 checks=42472 min_candidates=48 avg_nonempty_x100=324
D3 cfg=Gp_full_seven s=16 c0=5 bonus=5 formula=at_mount mount_reserve=40 mount_residue=30 seeds=8 false_guaranteed=0 false_user=0 true_enospc=8 wad_failed=0 max_pubs=7 stalls=8 halted_before_counting=0 fakes=0 unrecoverable=0 retention=0 max_pinned_after_push=88 injected=0 forced_per_window_x100=305 checks=7832 min_candidates=5 avg_nonempty_x100=783
```

- 按挂载时的 c 算：臂 A 12 格里 8 格出假性 ENOSPC、另 4 格 8 / 8 个种子卡死；G′（做满 7）12 格里 4 格出假性 ENOSPC、10 格卡死（有 2 格两样都有）。**两条臂都输。**
- 按 c_max 算，并让准入按 `df_guaranteed` 截断（d3b，要是不截断，按 c_max 算的臂 A 在 12 格里都有删了再写失败，见 d3 的 `formula=at_max` 各行）：假性 ENOSPC 两条臂都归 0，删了再写还剩零星失败（每格 0、8 或 16 次，8 个种子），两条臂都有，形状相同（`$SP/out_d3b.txt`），机制我没查。
- ⇒ **这一格不分臂，但两臂付的价不同**：`df` 要按 c_max 扣，臂 A 少报 15 + (2N − 3) × c_max，G′ 少报 15 + 11 × c_max。而 c_max 今天没有口径（C83（提交固定点没人回答） 还欠着运行时读数）。

### R4　两条臂的 `df` 都漏扣了实例切换的预留【我推的，引文我核过】

D23（journal 的角色与格式） 已定项 14：「切换要用的块在挂载准入时预留……预留量按 **N_switch × 一次切换的最坏量**……『一次固定点重做的最坏量』今天没有可算的口径」。材料第 35 行臂 A 的 `df` 式子与 E138 的 `df_guaranteed` 都没扣这一项，E138「它答不了的」第 4 条也承认没量。
这份预留若由同一池空间出，`df` 报的就比兑现得了的多出这一截；按 R1，每次切换还要让臂 A 多推 N − 1 次，所以臂 A 要为切换扣的量是 N_switch × (切换本身 + (N − 1) × c)，G′ 是 N_switch × (切换本身 + 约 2c)。

---

## 三、判据 3-4：「扣掉保留池与推空最坏残留」违背用户原意的最强版本

用户逐字：「我们的目标是 避免 btrfs ENOSPC， 我们绝对不能接受 ENOSPC」。[pitfalls.md] 第 1 条记的症状是「`df` 有空间但写不进、删文件也报没空间」。

1. **少报的量不是常数，其中一种情形下它没有上界**【R2 我核过，结论是我推的】。臂 A 在坏槽之下只有两种 `df` 可报：
   - 照材料的式子扣一个常数 ⇒ `df` 报 1171 块而写报 ENOSPC（R2，就是症状的前一半）；
   - 老实扣掉真正扣住的块 ⇒ `df` 随每次覆写单调往下掉，删了文件也不涨（被扣住的块随写入量无界增长，d8b 里 `pinned` 从 1170 涨到 1198 再到 1216）——就是症状的后一半。
   G′ 在同一个世界里 `df` 不动。
2. **常数本身按 N 放大，还压着一个今天没有口径的量**【我推的，数我核过】：按 R1、R3、R4 补齐之后，臂 A 要扣的是 15 + (2N − 3) × c_max，加上每容忍一次切换或写失败的 N_switch × (N − 1) × c_max，再加切换预留本身；G′ 是 15 + 11 × c_max 加 N_switch × 约 2c_max 加切换预留。在 4000 块的模型里这占比很大，搬到真盘上是 MiB 级（S = 16、c_max = 10 时约 15 MiB），**用字节算不上用户说的那种毛病**。我不拿字节当主论据。
3. **最强的一条是近满盘的发布风暴，材料第五节「反过来」第三句的触发条件就是它**【数我核过，折算是我推的】。E138 产物 S = 16、c = 5 那格 `forced_per_window_x100=3928`，即每个用户窗口 39.28 次强制发布，加上它自己那一次约 40 次；按 D16（发布语义） 已定项 7，每次发布三个序点（两道 FLUSH + 根槽 FUA），**一次 8 块的写约带 121 个序点**，而且是串行的（每次空发布都要等上一次的根持久，界才前进）。G′（做满 7）在 d1 里是 `forced_per_window_x100=524`，约 6.2 次、19 个序点，与 S 无关。
   btrfs 近满盘时 flush 状态机反复 `COMMIT_TRANS`（C283（准入失败时不先推发布就报 ENOSPC） 已记，本机树 `fs/btrfs/space-info.c`，未在本项目验证），写请求被拖成秒级，这是 btrfs 近满盘另一半出了名的症状：错误码没了，延迟还在。臂 A 在 N = 48 时比 G′ 深约 6 倍，而且随 S 线性加深；再叠上 R1，一次写失败会让那次准入变成 2N − 1 次串行发布。
4. **可退的历史近满盘时名义上 48、实际约 2**【数我核过】：E138 产物臂 A 在 S = 16 近满盘时 `avg_nonempty_x100=230 / 221`，也就是候选集里改过用户状态的根平均 2.2..2.3 个，低于 P2（用户定案）「最少保留 4 个持久根」的数，只是字面上用「根」而不用「状态」，所以不算违反。G 是 4.3 / 4.0。这一格该交用户确认：「4 个根」指的是 4 个根，还是 4 个可退到的状态。

⇒ 按材料失败条款第 2 条，交用户时写在最前面的应当是：**臂 A 的 `df` 少报量在一个根槽持续写失败时没有上界；平时的少报量随 N × c_max 线性增长；近满盘每次写带约 40 次串行发布（S = 16）**。

---

## 四、判据 3-7：界的算术

| 臂 | 写下来的界 | 反例 | 实际 |
|---|---|---|---|
| A | N 次发布 | 推空第 N − 1 次空发布的根槽写失败（R1）【我核过】 | 一次失败 **2N − 1**（S = 4 时 23、S = 16 时 95，`max_pubs_successful`）；同一个槽再失败一次，保留池不够、卡死；坏槽 **无界**（R2） |
| G′（做满 7） | 7 次发布 | 推空里任一次根槽写失败【我核过】 | 1..3 次失败要 **8..9**（d2b），与 N 无关；坏槽不影响（d7：`max_pubs_successful=7`） |
| G′（字面：做到生效） | 7 次发布 | 不需要故障 | 发布数 6 或 7，残留 5 + 4c 或 5 + 5c，**不是常数**（E138 产物 `crit1_push arm=G_floor_per_disk s=4 empty_cost=5 publications=6/7 … max_pinned=30`）；d1 里 c = 5 时删了再写失败 24 次，与原臂 G 相同 |

臂 A 2N − 1 的推导【我推的，数与它逐一相符】：失败的 txg 是 t + j，槽里留着 t + j − N，要到 t + j + N 才被覆盖；j 最大取 N − 1，t..t + 2N − 1 共 2N 个 txg，减去跳掉的那一个，是 2N − 1 次发布。

---

## 五、材料自己的错（带行号）

| # | 错在哪 | 出处 | 怎么核的 / 为什么是错的 |
|---|---|---|---|
| M1 | **G′ 的定义自相矛盾**【我核过】 | 第 36 行：「一律做到带新 F 的根在每块盘上都持久（最多 7 次发布），让残留恒为 5 + 5 × 空发布开销」 | 做到生效只要 6 或 7 次（E138 产物第 16–19 行 `publications=6/7`），6 次时残留是 5 + 4c，所以不恒定。按字面读（副本 `Gp_until_active`），它在无故障的近满盘上 c = 5 时照样删了再写失败 24 次（d1），与被判出局的 G 同一个数。只有「不管生效没有，一律做满 7 次」才过。两种读法结论相反，三条腿拿到的是哪一种，定义里没说 |
| M2 | **P5 说判据 1 把两个式子都钉成了绝对值，其实只钉了一个**【我核过】 | 第 20 行 | E138 判据 1（源码 `main` 里的 `crit1_*` 与 `expected_push`）钉的是推空后的残留与发布数，**保留池式子没有任何单测或判据钉住**，只在扫描里间接出现。而臂 A 扫描里「不卡死的最小保留池 51 / 239」（第 681 行）恰好等于扫描的下沿：式子 57 / 245 减去 `RESERVE_SWEEP_BELOW = 6`（源码第 44 行）。臂 A 真正要多少保留池没有量到；G 的 14 / 35 在扫描区间里面，才是量到的数 |
| M3 | **P4 与 P9 的「假性 ENOSPC 都是 0」「界 N 次发布」脱了口径：只在没有故障的近满盘上成立**【我核过】 | 第 19、24 行；D3（空间分配） 已定项 9（第 520–521 行） | E138 的 `run_near_full`（源码第 1085–1112 行）不注入任何故障；带故障的世界全在 70% 稳态，那里不走推空。E138「它答不了的」十条（第 743–752 行）没有「近满盘与故障的组合」这一条。一加上一次根槽写失败，臂 A 就出假性 ENOSPC、界变成 2N − 1（R1） |
| M4 | **同一个量有两个式子**【我核过】 | 第 24 行 P9 引 C283「扣掉保留池与 N × 空发布开销」；第 59 行倾向 ④「`df` 少报 N × 空发布开销 + 保留池」；第 428 行 D16 原文同样；而第 35 行臂 A 的定义与 E138（第 656 行）写的是「保留池 + 5 + (N − 2) × 空发布开销」 | c = 1、N = 48 时一个是 48，一个是 51。按 kb-discipline 第 4 条，这是矛盾，不是约数 |
| M5 | **E138 正文把 4.86 截成 4.8**，P6 写的 4.9 是对的【我核过】 | 第 733 行「臂 G 约 4.8 次」对 P6（第 21 行）「约 4.9 次」 | 产物 `forced_per_window_x100=486 / 485`。小事，但正文与产物对不上 |
| M6 | **`df` 口径漏扣实例切换的预留**【引文我核过】 | 第 35 行臂 A 的 `df` 列；第 45 行判据 3-2 自己点了「实例切换的预留与保留池叠加」，而候选臂的式子里没有这一项 | D23（journal 的角色与格式） 已定项 14 的预留量「N_switch × 一次切换的最坏量」今天没有口径（R4） |
| M7 | **倾向 ② ③ 的前提在 R2 下不成立**【我推的】 | 第 58 行 | 臂 A「不改盘上字节、不碰 D23 已定项 14」，这只在根槽写失败都是一次性的时候成立；遇到坏槽，要修就得改候选集并落盘一个下界，两条理由一起失效 |

**核过、没有错的**：P1、P2、P3 与原文一致；P6 的各个数与产物第 242–253 行逐一相符；P7「十条」数过；P8 的 72 = 3 个 S × 24（产物第 128–145 行）；P10 与 D16 未定项 1 第 365 行一致。
P4 的「S = 4、16 各 24 / 24 个种子」光看产物读不出来（`write_after_delete_failed=24` 是 24 个种子的合计），我在副本里逐个种子数过，每个种子恰好 1 次，**这句是对的**：

```text
D8c G s=4 c=5 write_after_delete_failed_per_seed=[1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1] seeds_with_failure=24
D8c G s=16 c=5 write_after_delete_failed_per_seed=[1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1] seeds_with_failure=24
```

---

## 六、判据 × 臂

| 判据 | 臂 A（整环 + 推空） | G′ 字面读法（做到生效） | G′ 做满 7 次 |
|---|---|---|---|
| 3-1 正确性 | **赢**：d2 / d4 / d5 / d7 全部 0 假候选、0 不可恢复；C281（回退后第一次发布会复用被抛弃时间线还引用的块） 各臂都有，记下 | **赢**：同上（d4、d5 只测了 G，活化逻辑与 G′ 相同） | **赢**：同上 |
| 3-2 第 1 条 | **输**：一次根槽写失败 ⇒ `df` 报 11 块时写失败（R1，d2 里 24 / 32 个 run）；坏槽 ⇒ 占用 70% 时 `df` 报 1171 块时写失败，随后卡死（R2，8 / 8 个种子，四格全中）；开销变大而式子按挂载时算 ⇒ 12 格里 8 格出假性 ENOSPC、另 4 格卡死（R3） | 同右列（推空只多不少） | **写下来的形态输一次**：上限 6 次空发布时，一次写失败 ⇒ 8 / 36 个 run（S = 4，c = 5）；上限放到约 9 次、保留池加约 2c 之后是 0（d2b，有少量卡死）。坏槽 0。开销变大按挂载时算 ⇒ 输（R3，与臂 A 同因） |
| 3-3 第 2 条 | **输**：同 R1、R2，都是删了之后的那次写失败 | **输**：无故障、c = 5 时删了再写失败 24 次（d1，与 G 相同） | 无故障时**赢**（d1 全 0）；一次写失败时按写下来的界输（d2）；把界补到 9 之后赢（d2b） |
| 3-4 `df` 口径 | **需用户确认**，三条写在最前面：坏槽之下少报量无界；平时少报 15 + (2N − 3) × c_max；近满盘每次写约 40 次串行发布、约 121 个序点（S = 16） | 需用户确认：少报 15 + 11 × c_max，与 S 无关 | 同左 |
| 3-5 用户定案 | 字面上不动（**记下**）；但修 R2 就得改 D23（journal 的角色与格式） 已定项 14 的候选集 ⇒ 真要修时**需用户重判**；非空根 2.2 < P2 的 4，问用户「4 个」指根还是状态 | 候选集加「txg ≥ F」，P2 已授权 ⇒ 记下，D23 已定项 14 的正文要同步 | 同左 |
| 3-6 代价 | 强制发布每窗口 36.2 / 39.3（S = 16，E138）；`df` 少报 108 / 480 块，外加 R3 / R4 那几项；0 格式字节 | 约 4.8 次每窗口（d1）；根记录 +8 字节 | 约 5.2 次每窗口（d1 `forced_per_window_x100=515..524`）；+8 字节 |
| 3-7 界的算术 | **界不成立**：一次失败 2N − 1；坏槽无界 | **界不成立**：残留不恒定（M1） | **界不成立但有界**：1..3 次失败要 8..9 |

**没做到的与做不到的**：全部结果都出自 E138 的计数模型，不建字节、不建 journal 记录，实例切换只建了「txg 推进一格」这一半。坏槽在真设备上多常见，本仓没有数：固件重映射会吃掉一部分，D23 的探针写吃不掉。d3 在 c 变大那一刻出现的零星删了再写失败，机制我没查。d6 是装置的人工痕迹，不采纳。
