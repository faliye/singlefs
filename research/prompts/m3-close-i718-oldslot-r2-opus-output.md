# m3-close-i718-oldslot-r2 攻方腿（Opus）报告（2026-10-23）

立场：假设丁甲站不住，造合法历史与坏写者去打。数全在我的仓副本上量（副本上的数，不是入库装置上的数）。下文我提的改法只在我的模型上量过、被攻过零轮。

## 复跑命令与文件 sha256

```
bash scripts/hosts/run-with-memory-cap.sh 24G bash research/prompts/m3-close-i718-oldslot-r2-opus-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs --work <新的草稿目录>
```

- `--frozen` 给仓根即可：开工时主工作区与 `research/prompts/m3-close-i718-oldslot-r2-snapshot/judged-sha256.txt`、`kb-sha256.txt` 逐行 OK（`sha256sum -c` 全部 OK）。rerun 只拷 `crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo`、`.claude/heavy-thread-default`，walk.rs 依次装三道开关，探针包 `crates/i718-round-two-probe` 加进副本工作区，`cargo build --release`。
- 挂钟：编译约 1–2 分钟；手挑镜像三种几何约 4 分钟；G 流全量枚举 24626 个状态切 64 段、每批 16 件并行，我跑的四批各 126–168 秒。末尾四行 `name=… same_as=…`，任一行 `DIFFERS` 就看它点名的 diff。`RERUN_PARALLEL`、`RERUN_THREADS` 可改。内存峰值没量，24G 是我按 16 件并行粗估的上限（推的）。
- rerun.sh 整条没在前台跑过（超过前台时限），各步我在草稿目录逐步跑过、产物就是 `out/` 里那几份。
- 不碰 checker 档的测试：探针是一个依赖 `singlefs-checker-tier` 的二进制包，只 `cargo build`、直接执行编出来的二进制（定义第 3b 步）。

```
c7f5c526b9984f2d9f2365eb988646f74c01a1985dbd4bd3a122c669491d285e  research/prompts/m3-close-i718-oldslot-r2-opus-model/out/cells-S4-condensed.txt
f4ee9ccb1aed8b2f06978f9a663ec9d14a2a7312d55a138c38fb65057e49c694  research/prompts/m3-close-i718-oldslot-r2-opus-model/out/cells-S4.txt
9cf3152ed2d5b0aec39984b6db1aab2e77b27129d5ed14fd4a3062e6374e19b5  research/prompts/m3-close-i718-oldslot-r2-opus-model/out/cells-S5-condensed.txt
2a728c6f9eafa9a58478d48456207a8caa72a9cc7dcaa650c9c086648769c999  research/prompts/m3-close-i718-oldslot-r2-opus-model/out/cells-S5.txt
e6b6ab2f2e7613e30d8cb991958b8b9b5f5c29d0d712877974d4c5b954f83728  research/prompts/m3-close-i718-oldslot-r2-opus-model/out/cells-S7-condensed.txt
af261fb7dec97117a07352d0e712ac8e007a5536e25a55863f33b8cc20a54c6d  research/prompts/m3-close-i718-oldslot-r2-opus-model/out/cells-S7.txt
a7acdf1f75a98d0d7d175b2b38984bd3b70871e7d352a743128f89d519a09d73  research/prompts/m3-close-i718-oldslot-r2-opus-model/out/cells-tally.txt
6ff8177de9b57f6fe1ef8019ee4acf4fc794e0e271d49046f2882c990953e930  research/prompts/m3-close-i718-oldslot-r2-opus-model/out/enumeration-G-S4-merged.txt
582b10ef56848ee98de923f6f11dba8cfe7d43d0d9096fbd1a38bc9e91ac584b  research/prompts/m3-close-i718-oldslot-r2-opus-model/out/enumeration-state-counts.txt
f26dc9f7c8fc68912df763e3a94b036b475519e501709a1051e0a985d87d5509  research/prompts/m3-close-i718-oldslot-r2-opus-model/probe/Cargo.toml
12d3b1aa1cc7d811fc5b8ad1447ca27258ea9244d8d111904942729759a2d828  research/prompts/m3-close-i718-oldslot-r2-opus-model/probe/src/main.rs
ee9f3c4b425b9339110d96c8d02cabacc4657c0626222e13f075c29082ddbc1c  research/prompts/m3-close-i718-oldslot-r2-opus-model/probe/src/ring_hole_pools.rs
60825bfb168a0abd64024921d176f5b9b0d575918c3c751425cafb045ea43e75  research/prompts/m3-close-i718-oldslot-r2-opus-model/tools/condense.py
06d54ba4b71d76492c5984b9fe78749ca85b64889e252de5c534d39e14ffc82b  research/prompts/m3-close-i718-oldslot-r2-opus-model/tools/merge.py
acdbdedc73b3642ea2513f7589f766d28502130024fb1a9b46dbe749e44b8708  research/prompts/m3-close-i718-oldslot-r2-opus-model/tools/rerun.sh
7eaa3c48a725c56bc1aa49b9c95859b38a27ca0c5d168bc48f63198abae41d02  research/prompts/m3-close-i718-oldslot-r2-opus-model/tools/tally.py
2751684b2bbbeb40d4b4be4f5553fe25e04aa128698380c2742dbd1b1a04a408  research/prompts/m3-close-i718-oldslot-r2-opus-model/tools/walk-candidate-switch-ding.py
bd90ed1e1addf6388c2ba5e613694498edcead8aa0f8ee20699cff84567fc4d9  research/prompts/m3-close-i718-oldslot-r2-opus-model/tools/walk-candidate-switch-ding-round-two.py
e3cb973e32d46786f7b1eabad4ff2d42106ed8c3e98a7326bffce76c3001e480  research/prompts/m3-close-i718-oldslot-r2-opus-model/tools/walk-candidate-switch.py
```

`walk-candidate-switch.py`、`walk-candidate-switch-ding.py` 从第一轮我那份模型原样拷来（sha256 与第一轮报告里记的相同）。

## 判器的几种读法（装在副本 walk.rs 上）

丁甲的正文（`research/prompts/_m3-close-i718-oldslot-r2-body.md:23` 那一句的前提「且 journal 里有点名 N+1 那次发布的记录」）没写记录要不要与最新根同一实例，第 3 种（`research/prompts/_m3-close-i718-oldslot-r2-body.md:27`「3. 被抛弃的根住在那一槽时的那一支（正推第 V1 格推的 N+2−3S）。」）没写是现算还是一个值。我各装一种读法，下文列名：

| 列名 | 是什么 |
|---|---|
| today | 今天的判法（也是丙） |
| jia | 第一轮的甲（调查员补丁同义） |
| jia_any_lap | 第一轮我提的甲-任一圈（不看记录） |
| ding_r1 | 第一轮我量过的丁甲：记录要同实例、txg = N+1，N+1 > 3S |
| ding_r2_same_instance | 第二轮丁甲，记录要同实例；N+1 ≥ 3S；第 2 种按「补进环之后的现算」（与环上别的根取小、再与 F 取大）；第 3 种按「那一槽没有根时的现算」读，与第 1 种相同 |
| ding_r2_any_instance | 同上，记录只看 txg = N+1、不看实例 |
| ding_r2_raw_values | 记录只看 txg；第 2 种按值读（t = max(F, N+1−k·3S) 就收，不与环上别的根取小），第 3 种按值读（t = max(F, N+2−3S) 就收） |

缩写：G = 成立，R = 判红，NA = 不适用；rec=xy 里 x 是「有同实例、txg N+1 的记录」，y 是「有 txg N+1 的记录（任一实例）」，t / f 是有 / 没有。七列次序同上表。

## 各格判定一览

| 格 | 判定 | 量的还是推的 | 一句话 | 什么现象会推翻它 |
|---|---|---|---|---|
| W2-前提的实例读法（新）：满环之后另一实例可写挂载，写行那次发布 N+1 的根槽写坏 | **打中丁甲（同实例读法）** | 量的 | 合法、单故障；记录是新实例写的，同实例读法前提不成立、照今天判红：S = 4、5、7 各扫一圈 N，12 / 15 / 21 段全红，写 0 与撕两种都红；不看实例的读法全绿 | 写行那次发布的记录带的实例代号与最新根相同；或这一形在入库装置上 I-7.18 不红 |
| W2-前提的实例读法：同一挂载里写行落了、第一次暖机空发布写坏 | 没打中 | 量的 | 写行那条根与暖机那条记录同实例，丁甲各读法全绿（3 × 一圈） | — |
| W1-第 3 种按值读、第 2 种按值读 | **打中按值读** | 量的 | 洞在别的槽（仍是最旧有效根）、下一槽写坏：按一圈窗口滤根的写者（t = N+1−3S）与 t = N+2−3S 的写者，按值读收下，按现算读判红；三种几何同向 | 第 3 种定成现算（那时它与第 1 种相同、是多余的一句） |
| W1-第 2 种「任一圈」只看镜像定不定得出 | 定不出哪一圈，等于收下这一族里任一个值 | 量的 | 两圈之后 t 早一圈的写者（t − 3S）在写坏的镜像上，jia_any_lap 与四种丁甲全收、甲与今天红；根槽写之前那张全红 | 判器能从镜像上读出那一槽住过哪一圈 |
| W1-兄弟镜像在崩溃点重放里兜不兜 | **前提不成立（撤回我第一轮的一句）** | 量的 + 读代码 | 崩溃点重放的枚举走不到「下一槽读不出」这一态：根槽写是一个扇区，不取撕裂第三态；G 流全量 24626 个状态里下一槽一次都没读不出。这一态只在写失败的给定状态里出现，那里兄弟镜像不一定一起交出去 | 根槽写长于一个扇区的几何进了崩溃点重放 |
| W2-持久顺序保证记录在 | 没打中 | 推的（读条款与代码） | 写失败只在根槽写那一步，记录已过屏障；镜像两盘各一份 | 一张「下一槽写坏、两盘都没有 N+1 的记录」的单故障镜像 |
| W2-CPU、GPU 的交接 | CPU 现成，GPU 没有现成 | 读代码（推的） | CPU 同一函数里已有逐盘记录表；GPU 的记录表是逐盘重填的草稿区，判 I-7.18 时不保证还在 | — |
| W3-满环、最旧根那一槽事后读坏（读故障、没有记录） | 丁甲红，甲与甲-任一圈绿 | 量的 | 三种几何同向；要它绿就得去掉记录那一条前提，回到第一轮格 F「p 写 255」被收下那一形 | — |
| W3-环没满、最旧根事后读坏 | 全部候选红 | 量的 | 三种几何同向 | — |
| W3-零单元发布、无文件版本 | 没打中 | 量的 | 只反复挂载、不建文件的池转过两圈后写行那次写坏：七列全 NA | — |
| W4-换几何 S = 5、7 | 结论不翻 | 量的 | 上面每一格三种几何同向 | — |
| W4-整段展开 | 量了一条流 | 量的 | G 流全量 24626 个状态，七类状态全绿；覆盖写那条流 1048588 个状态超过 10⁶，按定义没跑 | — |

## W2：同实例读法在合法镜像上误红（G 格）

### 历史（每一步许可它的那一句）

1. 实例 1 覆盖写到 N（N 放开扫一圈：S = 4 时 24–35），环满、无洞，下一槽 N+1 里住着最旧根 N+1−3S。进程停（崩溃，不卸载；卸载会抬 F，这一形就不出现）。
2. 下一个实例可写挂载。写行那次发布的 txg 由挂载现算：`.claude/kb/decisions/16-发布语义.md:224`「写行那次发布的 txg 取 D23（journal 的角色与格式） 已定项 14 注 3 那个下界本身（max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1）」——这里就是 N+1，根槽正落在最旧根那一槽。
3. 写行那次的记录先落盘（`.claude/kb/decisions/16-发布语义.md:207`「一次发布的持久顺序恒为 COW 单元 / 节点 → 屏障 → journal 记录 → 屏障 → 根槽（FUA）」），根槽写写成 0 或撕开，进程在重发之前崩。写失败的处置同题面那一形（D16 已定项 1「写失败的槽按旧内容算」，`.claude/kb/decisions/16-发布语义.md:36`）。

这一形不稀奇：环转满之后的每一次可写挂载，第一次根槽写都落在最旧根那一槽上。

### 量的数

`out/cells-tally.txt` 原样三行（S = 4 一圈 12 段）：

```
S4-G-row-publish                                   crash_before_the_root_slot_write     rec=ft n=12  | G G G G G G G
S4-G-row-publish                                   root_slot_torn_like_the_enumeration  rec=ft n=12  | R G G R R G G
S4-G-row-publish                                   root_slot_written_zero               rec=ft n=12  | R G G R R G G
```

S = 5（15 段）、S = 7（21 段）同向：`S5-G-row-publish … rec=ft n=15  | R G G R R G G`、`S7-G-row-publish … rec=ft n=21  | R G G R R G G`（写 0、撕两行都是）。一张原样（`out/cells-S4-condensed.txt`）：

```
S4-G-row-publish-N25                                 root_slot_written_zero               N=(25, 1)   t=14  p=255 next=None           rec=ft | R G G R R G G | red=["I-7.18"]
```

最新根是实例 1 的 txg 25，t = 14 = N+1−3S（写者对）；记录只有实例 2 的 txg 26，同实例读法（ding_r1、ding_r2_same_instance）照今天判红，不看实例的读法绿。对照：同一次挂载里写行落了、第一次暖机空发布写坏（G2），记录与最新根同实例，四种丁甲全绿（`S4-G2-first-warm-up-publish … rec=tt n=12  | R G G G G G G` 两行）。

### 四句

- 分不分辨臂：分。同实例读法红，不看实例的读法、甲、甲-任一圈绿，今天（丙）红。
- 被判的系统当时看不看得到：看得到。记录的实例代号与 txg 都在镜像上；同实例读法是自己不去用。
- 满足的分句：正文第四节 W2「前提「journal 里有点名 N+1 的记录」：…这一前提是不是还判得出」，以及第五节「丁甲在合法镜像上误红的历史」。
- 跑前条款的改法：正文第三节没给改法。把前提写成「有 txg = N+1 的自证记录，不论实例」这一格就不中（量的，上面三种几何全绿）；它多放过什么，见下一节 W1 与 W2 的「兄弟镜像」那一段（不看实例时，另一实例的记录只能出在最新根之后，量不到更多的逃逸，推的）。

## W1：丁甲多放过的错 t、错 p

### 第 2 种「任一圈」：判器定不出哪一圈

判器在写坏的镜像上看得到的只有 t、N+1 与环上别的槽；那一槽住过哪一圈的根，镜像上没有。所以第 2 种落到实现里只能是「t 落在 N+1 那一族、按补进去现算对得上就收」，哪一圈都收。坏写者「t 早一圈」（满环无洞、N = 2·3S+1，t 改成 t − 3S，仍 ≡ N+1 (mod 3S)，比真值低）原样（`out/cells-tally.txt`）：

```
S4-A-bad-t-one-lap-older                             crash_before_the_root_slot_write     rec=tt n=1   | R R R R R R R
S4-A-bad-t-one-lap-older                             root_slot_torn_like_the_enumeration  rec=tt n=1   | R R G G G G G
S4-A-bad-t-one-lap-older                             root_slot_written_zero               rec=tt n=1   | R R G G G G G
```

S = 5、7 同向。第一轮我量过的丁甲（ding_r1）同样收，第二轮四种读法比它没有多放这一格；甲不收（它只认 N+1−3S）。这一类比真值低，是不回收的方向（漏空间），I-7.18 求等照字面要判红。

### 第 2、3 种按值读：放过按一圈窗口滤根的写者

历史：实例 1 写到 3S+2；下一个实例的可写挂载崩在写行那次根槽写之前，那一槽留着上一圈的根，成了洞（`crates/singlefs-harness/tests/common_root_ring_holes/mod.rs:207`「pub fn crash_a_mount_before_its_row_root(image: &MemoryPool) -> (MemoryPool, u64) {」，我的探针原样拷了这份模块、只把 S 改成从环境变量取）；再下一个实例写到 N = 写行 txg + 4，洞还在环里、是最旧有效根，但不在下一槽；N+1 的根槽写坏。合法写者 t = 洞里那条根的 txg，七列全绿。两份坏写者（只改最新根的 t）：

- 按一圈窗口滤根：t = N+1−3S（窗口 (N−3S, N] 把洞里那条更旧的根滤掉）。这正是 D16 已定项 1 废掉的写法：`.claude/kb/decisions/16-发布语义.md:36`「六处都不再按一圈窗口滤根，窗口之外的有效根（崩在记录之后、根槽之前那一形留在槽里的旧根）也在候选集里、也压着门槛」。比真值高，是会回收洞里那条有效根还引用着的单元的方向。
- t = N+2−3S：第 3 种按值读收的那个值。

原样（S = 4）：

```
S4-I-hole-elsewhere-bad-window-filter-t-N+1-3S       root_slot_written_zero               N=(19, 3)   t=8   p=255 next=None           rec=tt | R R R R R R G | red=["I-8.10", "I-7.18"]
S4-I-hole-elsewhere-bad-t-N+2-3S                     root_slot_written_zero               N=(19, 3)   t=9   p=255 next=None           rec=tt | R R R R R R G | red=["I-8.10", "I-7.18"]
```

撕裂那张同样只有 ding_r2_raw_values 绿；根槽写之前那张七列全红；S = 5、7 同向（`out/cells-tally.txt` 的 `S5-I-…`、`S7-I-…` 行）。按「补进环之后的现算」读的三种丁甲都红。I-8.10 跟着红是我只改根槽、没改记录的副产物，判臂只看 I-7.18。

四句：分辨读法（按值读收、按现算读红）；系统看得到洞里那条根（它读得出），按值读自己不用；满足的分句是第五节「找丁甲放过的错 t」；改法是把第 2、3 种都写成现算，第 3 种因此与第 1 种相同、可以删掉（量的：按现算读的那三列这一格全红）。

### 「兄弟镜像兜住」在崩溃点重放里没有载体（撤回我第一轮的一句）

第一轮判决把两格（F「p 写 255、N+1 在飞」、B「t 写 15」）交给兄弟镜像兜：同一条根在「根槽写之前」那张上判红。第二轮要问的是崩溃点重放的枚举里兄弟镜像是不是一定也被判。量下来，问题的前提不成立：枚举根本走不到「下一槽读不出」这一态。

- 读代码：根槽写是一个扇区，枚举不给它取撕裂第三态，`crates/singlefs-checker-tier/src/crash.rs:2193`「    /// 当成又一条根槽写）。根槽写是一个物理块，崩溃点重放的池都按 512 字节的物理块建，走不到。」；判它能不能撕的规则在同一份文件的 `is_tearable_in_place_overwrite`（长不过一个扇区的写不撕）。
- 量的：G 流（S = 4，满环之后另一实例可写挂载，57 次写、14 段）照全量展开（`full_expansion`，经 `sample_crash_states_on_the_cpu` 把序号 0 到 24625 全走一遍，切 64 段、首尾相接，合并核过 `contiguous=True states_counted=24626`），七类状态的下一槽全读得出、三列全绿。`out/enumeration-G-S4-merged.txt` 头三行原样：

```
shares=64 first=(0, 384) last=(24241, 24626) contiguous=True states_counted=24626
count=16400 newest=(25, 1) t=14 p=255 F=0 next_slot=Some((14, 1)) record_same_instance=false record_any_instance=false today=holds ding_r2_same_instance=holds ding_r2_any_instance=holds today_red=[]
count=4104 newest=(26, 2) t=15 p=255 F=0 next_slot=Some((15, 1)) record_same_instance=false record_any_instance=false today=holds ding_r2_same_instance=holds ding_r2_any_instance=holds today_red=[]
```

所以：

1. 「下一槽写坏」这一态只来自写失败（设备报错），不是崩溃态。它进 checker 的路是写失败的给定状态，例如 `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 交出的回退场景；那个场景交出的是「崩在重发之前、恢复之前」那一张（`crates/singlefs-harness/tests/common_images_for_the_checker_tier/rollback_by_a_forward_publish.rs:1024`「        &format!("{mode:?}：崩在重发之前、恢复之前"),」），同一场景里没有交「记录落了、根槽没写」那张兄弟镜像（我读了 951–1075 行，推的：别处可能另有）。给定状态里，丁甲收下的错 t 没有兄弟镜像兜。
2. 坏写者在崩溃点重放里照样会被判：它写坏 t 的那条根在「下一槽还读得出」的状态里（重放里大量出现）今天就红，丁甲在那些状态上与今天相同。丁甲改变的只是给定状态那一类镜像上的判定。
3. 撤回我第一轮的两句：`research/prompts/m3-close-i718-oldslot-r1-opus-output.md` 格 A 那一段「崩溃点重放的枚举本来就产出这张撕裂态」与 V2「纯掉电：……崩溃点重放的撕裂态就是它」。我当时引的 `crash.rs` 那一行讲的是原地覆写的撕裂镜像，根槽写不在其内。连带第一轮我对丙的代价说法（「撕裂态在崩溃点重放的枚举域里，凡转满一圈……的流都要豁免」）也不成立：丙的已知红只落在写失败的给定状态与故障注入上，不落在崩溃点重放上（推的，故障注入那一类没量）。

四句：不分辨臂（今天、丙、各丁甲在枚举里一样全绿），这是方法上的发现，不拿它判臂；判别子不涉及；满足的是 W1 第三问与 W4「手挑崩溃状态」那一问；改法：要让兄弟镜像的论证有载体，给定状态那一类要连同「记录落了、根槽没写」那张一起交出（推的，没改、没量）。

## W2：前提判不判得出

- 持久顺序：根槽写能写坏那一槽时，同一次发布的记录已过屏障（`.claude/kb/decisions/16-发布语义.md:207`，上文引过）；journal 环两盘各写一份。单故障下，下一槽被这次写写坏的镜像上 N+1 的记录必在（推的；量的那一面是第一轮格 A、B 的写坏镜像 rec 全为 t，本轮 A、B、I 三种几何同样 rec=tt）。G 格是反过来那一面：记录在，但是另一实例写的，见上节。
- 记录读坏：那是第二个故障（写失败之外再读坏记录），超出单故障；要读坏的是两盘各一份，再多一个故障（推的，没造）。
- journal 环转过一圈：恢复之前的镜像上 N+1 的记录是最新的那条，后面没有写会盖它；恢复之后的新实例从 N+2 起写，那时最新根已换成新实例的根，前提问的是新根的下一槽（推的，没造）。
- 另一实例的记录能不能冒充：txg 比最新根大的记录只来自最新根之后起的那次发布（同实例在飞，或下一次挂载的写行），写行那次的 txg 取环里全部根与记录的 txg 的最大值加一（D16 已定项 8，原句上文 G 格引过），所以「点名 N+1 而不是最新根之后那一次发布」的记录造不出来（推的）。这是不看实例的读法没多放出东西的理由，也只是推的。
- CPU 交接：池级 checker 在同一个函数里已经扫过逐盘记录表，`crates/singlefs-checker/src/walk.rs:18223`「        scanned_journal_records_by_device(reader, &geometry, filesystem_identifier_low);」，I-7.18 的调用在同一函数的 `crates/singlefs-checker/src/walk.rs:18979`「    judge_the_reclaim_threshold_and_the_pinned_ring_slot_of_the_newest_root(」，传一个布尔进去即可（我的原型为了少动调用处，在调用处现扫了一遍，不是实现形态）。
- GPU 交接：记录表是逐盘重填的草稿区，`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:389`「    record_count = 0u;」在 `checker_scan_journal_of_device` 开头清零，逐盘那一圈在 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:5555`「        checker_scan_journal_of_device(hdr(12u + d), fsid_low);」；判 I-7.18 在走读阶段 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:5163`「            judge_the_reclaim_threshold_and_the_pinned_ring_slot_of_the_newest_root(newest_root, g);」，那时表里留的是最后一块盘（或「由记录施加出来的那一版」那一支重扫的）记录，不保证覆盖两盘。丁甲在 GPU 上要么在判之前按位置表重扫一遍记录槽（O(记录槽数)，与 `checker_scan_journal_as_the_recovery_reads` 同一个做法），要么在逐盘那一圈里顺手记一个「见过 txg = N+1 的记录」的标志——但那时还没择根，N 未知，只能记「每块盘上记录的最大 txg」再比（推的，没写、没过 naga）。记录头里 checkpoint_txg 在偏移 26（同一份文件第 480 行注释），实例代号在表里第 3 个字。

## W3：从未被看过的形

| 形 | 原样（`out/cells-tally.txt`，S = 4；S = 5、7 同向） | 判定 |
|---|---|---|
| 满环、最旧根那一槽（就是下一槽）事后读坏，没有在飞的发布 | `S4-R-full-ring-oldest-read-fault                   stable                               rec=ff n=1   \| R G G R R R R` | 今天与丁甲红，甲、甲-任一圈绿 |
| 环没满（新池写到 txg 5），最旧根事后读坏 | `S4-R-ring-not-full-oldest-read-fault-oldest0       stable                               rec=ff n=1   \| R R R R R R R` | 全红 |
| 只挂载、不建文件的池转过两圈，写行那次根槽写坏 | `S4-K-no-file-pool-row-publish                      root_slot_written_zero               rec=ft n=1   \| NA NA NA NA NA NA NA` | 全不适用（最新根没有记账树），不误红 |

- 读故障那一形该不该绿：满环无洞时，它与题面那张写坏镜像只差 journal 里有没有 N+1 的记录。丁甲靠的就是这一条记录把两者分开，所以它罩不住读故障；要罩住就得去掉记录那一条前提，回到甲-任一圈，第一轮格 F「p 写 255 而还钉着」12 / 12 被收下（`research/prompts/m3-close-i718-oldslot-r1-opus-output.md` 里 `F	bad_p_255` 那一行），与 C863 的用户立场打架（`.claude/kb/checks-owed.md:548`「这两类在镜像上与合法的钉逐字节相同（读故障不留在盘上）」那一行）。读故障是盘上不留痕的那一类，我判它留在 C863 的射程里，丁甲不该为它放宽（推的，判断）。
- 零单元发布、无文件版本上的复制根：I-7.18 条文本来就不判（`crates/singlefs-checker/invariants/02-根环.md:25`「零单元发布与无文件版本上的复制根，t、p 抄自上一版，环转满之后求等会误红，不判」那一句），与这一态叠在一起照样不判，量的。
- 撕裂域里的邻槽：本轮没新造；根槽写是一个扇区（上文 W1 末节引的 `crash.rs` 那一行），一次写碰不到邻槽，第一轮的推断不变。

## W4：换几何、换历史、换取样方法

- 几何：S = 4、5、7（环长 12、15、21），每一格三种几何同向（`out/cells-tally.txt` 87 行）。S 的合法区间是 4 到 16，S = 8 以上没量。
- 历史：下一次发布换成写行（G）、第一次暖机空发布（G2）、无文件池的写行（K）；G、G2 放开扫一圈 N（用户决定的那一步），每段写 0 与撕两张、加根槽写之前一张。
- 取样方法：照定义第 3b 步把下一次发布那段写表整段展开。覆盖写那条流（25 次写、4 段）全量 1048588 个状态，超过约 10⁶，按定义没跑；G 流 24626 个状态全量跑完（上文）。两条原样（`out/enumeration-state-counts.txt`）：

```
stream=i718-r2-S4-A-legit writes=25 segments=4 states=1048588
stream=i718-r2-S4-G-mount-after-a-full-ring writes=57 segments=14 states=24626
```

结论：换几何、换历史不翻第一轮与本轮的任何一格；换取样方法翻掉的是「兄弟镜像在崩溃点重放里兜」那一条论证的前提（上文 W1 末节）。

## 几个改法各修哪一格（我提的，只在我的模型上量过、被攻过零轮）

「量过」= 上面贴的副本输出；「推的」= 按代码推、没实现没跑。「收」= 坏写者被判成立。

| 改法 | G 写行写坏（合法） | G2 暖机写坏（合法） | A t 早一圈（写坏那张） | I 窗口滤根写者（写坏那张） | R 满环读故障（合法） | 第一轮 F p 写 255、没在飞 |
|---|---|---|---|---|---|---|
| 今天 / 丙 | 红（量过） | 红（量过） | 红（量过） | 红（量过） | 红（量过） | 红（第一轮量过） |
| 丁甲，同实例读法 | **红（量过）** | 绿（量过） | 收（量过） | 红（量过） | 红（量过） | 红（推的，与 ding_r1 同：第一轮量过 12 / 12 红） |
| 丁甲，不看实例 | 绿（量过） | 绿（量过） | 收（量过） | 红（量过） | 红（量过） | 红（推的：那张镜像没有 txg N+1 的记录） |
| 丁甲，按值读 | 绿（量过） | 绿（量过） | 收（量过） | **收（量过）** | 红（量过） | 红（推的） |
| 丁甲不看实例、第 2 种按现算、删去第 3 种 | 绿（推的：第 3 种按现算与第 1 种相同） | 绿（推的） | 收（推的） | 红（推的） | 红（推的） | 红（推的） |

「A t 早一圈」那一格四种丁甲都收，只有甲不收；它比真值低，写坏那张之外的状态今天就红（量过：根槽写之前那张七列全红）。要收紧它，第 2 种得限成「N+1−3S 与洞那一族之间实际可能住过的那一圈」，判器看不到，我想不出只看镜像的办法（推的）。

## 没打中的形状

- 同一挂载里写行落了、暖机空发布写坏（G2）：四种丁甲全绿（量的，三种几何各一圈）。
- 无文件池（K）：全不适用（量的）。
- 不看实例的读法多放出另一实例的记录：造不出「点名 N+1、却不是最新根之后那一次发布」的记录（推的，理由在 W2）。
- 持久顺序漏洞（下一槽写坏而两盘都没有 N+1 的记录）：没造出（推的；我造的写坏镜像 rec 第二位全是 t）。
- 取样范围：几何 S = 4、5、7；A、I、B、R 每种几何各一段历史（A 取 N = 2·3S+1，B 取实例 1 写到 3S+2，I 取同一个洞之后写行 + 4），G、G2 各放开扫一圈 N（12 / 15 / 21 段）；每段写 0、照枚举撕法撕、根槽写之前三张；坏写者三份（t 早一圈、窗口滤根、t = N+2−3S）。手挑镜像合计 S = 4 的 95 行、S = 5 的 113 行、S = 7 的 151 行（`wc -l out/cells-S*-condensed.txt`）；枚举一条流 24626 个状态。

## 这条腿自己的限度

- 覆盖写那条流（A）、洞的那条流（I）全量都超过约 10⁶ 个状态（A 量出 1048588，I 没算），按定义没跑；只跑了 G 一条流。本腿原型跑的全量合计 24626 个状态（另有试跑一段 123 个状态与只算状态数的两次，不判）。
- 坏写者是改根槽字节造的，记录没跟着改，I-8.10 与部分 I-3.1 跟着红是副产物；判臂只看 I-7.18。
- 丁甲的「记录在不在」在副本 walk.rs 的调用处现扫 journal、经线程局部变量传进判里，原型写法；两块盘任一块有就算。
- GPU 一侧一种读法都没写，W2 的 GPU 那一条全是读代码推的。
- 正文没定的两处（实例、第 3 种是值还是现算）我各装一种读法量；哪一种是正文本意由主 agent 定。
- 一个写法问题照实记：我这份定义里没有 Write / Edit，改已有文件应当只用 `replace-once.py`；开工时我用 python 就地改过一次探针源码（`main.rs` 的 `recorded_*` 两个函数，经同目录临时文件再 `mv` 换上），之后都改用 `replace-once.py`。探针源码第一段一次写了 151 行，超过「每次不超过 150 行」1 行；`out/cells-S7*.txt` 是产物原样，一次拷进去 151 行。
- 资源：本机当时被别的会话的 checker 档重活占满（提醒里 loadavg 38–52），我的跑法都是单线程或 16 件 `nice -n 19` 并行、每件内存上限 1G（低于默认 8G，按跑一段的实况给的，没超）；没用第二台。
- 量的数全在副本上，不是入库装置上的数。

## 没做什么

- 没跑 checker 档的测试（崩溃点重放入口、GPU 对拍、给定状态用例）；`rollback_by_a_forward_publish.rs` 那张「崩在重发之前、恢复之前」镜像的期望写的是 `&["I-3.1"]`（`crates/singlefs-harness/tests/common_images_for_the_checker_tier/rollback_by_a_forward_publish.rs:1020`「            &["I-3.1"]」），第一轮正推量过这一形今天红在 I-7.18；那条给定状态用例今天绿不绿我没跑、没核，交主 agent。
- 没写 GPU 一侧的丁甲，没过 naga。
- rerun.sh 整条没在前台跑完（超时限），各步分开跑过。
- 不判辩方那几格，不替主 agent 采纳。

## 要主 agent 跑的命令

- 复跑（约 15 分钟，放后台）：上面「复跑命令」那一条，工作目录随意；看末尾四行 `name=… same_as=…`。
- 核引文：`python3 scripts/agents/cite-check.py research/prompts/m3-close-i718-oldslot-r2-opus-output.md`。

## 清理

交回前删掉的（删之前 `du -sh`）：仓副本 `repo` 37M、编译目录 `target` 1.7G、崩溃库 `lib-g-0` 到 `lib-g-63` 与试跑库合计 519M，都在 `/home/fy5090/.local/share/singlefs/work/m3-close-i718-oldslot-r2-opus/` 下；那里只留日志、退出码文件与几个辅助脚本（780K）。

## 引文核对

`python3 scripts/agents/cite-check.py research/prompts/m3-close-i718-oldslot-r2-opus-output.md` 末行（交回前现跑）：

```
  ✓ 核了 15 处引文、3 处裸行号，对上 15 处，没判 3 处（其中裸行号没有引文可比 3 处；逐处列在上面）
```
