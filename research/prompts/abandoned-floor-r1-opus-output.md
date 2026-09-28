# abandoned-floor-r1 云端攻方腿（Opus）报告：X1 候选在复现与相邻历史上的结局

日期 2026-09-28。攻击面只有 X1。所有数都出自攻方副本（冻结副本 `/tmp/claude-1000/abandoned-floor-r1/tree/crates/` 拷出来、打上调查员的探针 diff 与我加的候选开关），不是入库装置上的数。重型测试一条没跑。

## 复跑

模型目录 `research/prompts/abandoned-floor-r1-opus-model/`：

| 文件 | sha256 |
|---|---|
| `abandoned_floor_candidates.rs`（副本里的 harness 档用例，E1–E11） | `0d96867b15df7fe93d74f045f087758bacbeb501c77936883cf62614b18d7781` |
| `copy-vs-frozen.diff`（副本相对冻结副本的全部改动：调查员的探针 diff、候选开关、两份用例） | `faa88600353270bc97c536d21c981ed0c085eaf23bea1739b30a1c9c76c73a93` |
| `patch_walk.py`（checker 侧：甲、丁、F 算不算被抛弃根） | `fca64b46d9f86b0859f7a8ff2589e939f1241f4bc54dad6294f71b4224a67a9a` |
| `patch_core.py`（实现侧：乙、丙、F 算不算被抛弃根） | `3cb3b7440fa30934e77bcfbb4ecbb94412bef632c89c053f4f964ee92769dc5f` |
| `batch1.sh` / `batch2.sh`（逐个候选顺序跑） | `a5059aadbdd5f49d9d880fc383c2685fa4df997e86648719bf70951280bd0f0d` / `c5de7adf1f935153ab1f2c966632a7d565bb9d118e0cebe01d2fb20a014769bf` |
| `tabulate.py` / `compare.py` / `diffcells.py`（汇表） | `306588b98d7e99752a4717ad050fc8b9859482390dacaf9f3b48f5895c67b040` / `242744e0e5ca9f3872b136f376a0501fdbf00b0f65c013793f3e8bcace6caecd` / `b801712a47ec94ffb3f0cf543aa20b643ac3c8894c98c7b0b563fc59de12aa38` |
| `cellview.py`（把一格压成一行，报告里贴的原样输出都是它出的） | `bf93f1752aa7d1094e6c17b1f1b14541c76068df871fddb7171ab0cbb3277dbf` |
| `runs/`：批 1（`b1-*.log`，11 份）、批 2（`b2-*.log`，11 份）、E11（`b3-1-today-e11.log`）的原样输出 | 见文件本身 |

复跑步骤：

```
mkdir -p <草稿>/repo && cp Cargo.toml Cargo.lock <草稿>/repo/ && cp -r .cargo <草稿>/repo/
rsync -a --exclude target /tmp/claude-1000/abandoned-floor-r1/tree/crates <草稿>/repo/
cd <草稿>/repo/crates && patch -p1 < <仓>/research/prompts/abandoned-floor-r1-opus-model/copy-vs-frozen.diff
cd <草稿>/repo && CARGO_TARGET_DIR=<草稿>/target nice -n 19 bash <仓>/research/scripts/run-with-memory-cap.sh 8G \
  bash <仓>/research/scripts/capped.sh 16 cargo build --offline --release -p singlefs-harness --test abandoned_floor_candidates
# 一个候选一遍（候选名见下表「开关」列；AF_ABANDONED_F=count 是第 ② 行「算进」）：
AF_CANDIDATE=<开关> AF_ABANDONED_F=<not-counted|count> nice -n 19 bash <仓>/research/scripts/run-with-memory-cap.sh 8G \
  bash <仓>/research/scripts/capped.sh 16 <草稿>/target/release/deps/abandoned_floor_candidates-<hash> --nocapture --test-threads 16 [e9_ e10_ | e11_]
python3 compare.py <today 的日志> <候选的日志> …     # 每个候选「抬前绿、之后红」的格数与红的种类
python3 diffcells.py <today 的日志> <候选的日志>    # 与 today 签名不同的格，按实验与变化归组
```

批 1 跑的二进制里还没有 E9–E11（那三组后加），批 2 只跑 `e9_ e10_`，E11 只跑了 today。用模型目录里的最终用例整份重跑，批 1 的每份日志会多出 E9–E11 的行，E1–E8 的行应逐行相同（用例是确定性的：内存盘、固定内容、固定次序）。

## 各格判定一览

候选在副本里的开关（`AF_CANDIDATE`）与实现：

| 候选 | 开关 | 副本里怎么做的 |
|---|---|---|
| 今天 | `today` | 不改 |
| 甲 | `jia` | checker 候选集的下界改成「txg ≤ F_生效 的最大一条没被抛弃的根」（调查员的 `walk_nearest_root_below_floor` 同一句判法，只看根环里的根） |
| 乙-顺延 | `yi_adjust` | 抬 F 的目标 txg 上的根全属于被抛弃实例时，改取下一个有没被抛弃的根的 txg（不超过上限） |
| 乙-拒 | `yi_refuse` | 同一条件下拒这次抬 F（副本借 `RollbackFloorAboveCeiling` 报，ceiling 写成 u64::MAX 作记号） |
| 丙-只在抬 F | `bing` | 抬 F 时回收门槛从 max(新 F, 环里最旧有效根) 改成 max(「txg ≥ 新 F 的最低一条没被抛弃的根」, 环里最旧有效根)：只被 F 之下的根与被抛弃根引用、已释放的槽回收；被抛弃根引用的照影子账隔离 |
| 丙-抬 F 与挂载 | `bing_all` | 同一门槛，可写挂载重建分配器时也用 |
| 丁-并隔离集 | `ding_isolated` | I-3.1 的「遍历」一边并上「被抛弃根引用 − 候选集引用 − 当前账里仍分配」 |
| 丁-并隔离集中账里已释放的 | `ding_isolated_released` | 同上，只取当前账里有已释放记录的槽 |
| 丁-并 F 之下仍在 defer 的 | `ding_defer_below_floor` | I-3.1 的「遍历」一边并上「F 之下、没被抛弃的根引用、当前账里已释放且释放代 > F_生效」的槽 |
| 第 ② 行「算进」 | `AF_ABANDONED_F=count` | checker 与实现两边算 F_生效 时被抛弃根带的 F 也取大（与 `today`、`jia` 各配一遍） |

结局（「抬前绿之后红」= 抬 F 之前 checker 全绿、抬 F 之后或其后任一段用户后缀里出现红；格数由 `compare.py` 数）：

| 候选 | 批 1（E1–E8，每候选 978 格） | 批 2（E9–E10，每候选 276 格） | 复用被抛弃根或候选版本引用的单元 | 判定 |
|---|---|---|---|---|
| 今天 | 26 格红（I-3.1；另 1 格是共有的 C554 形） | 6 格红（I-3.1） | 没有新的 | 第 43 行那一形在空档里照样红 |
| 甲 | 5 格红：E7 单个根槽永久读不出 4 格（I-3.1），共有 1 格 | 8 格红：E9「只由记录施加出来的那一版」I-3.1 | 没有（红是 checker 多算） | **打中**：一次崩溃、不带任何故障的历史上 I-3.1 红 |
| 乙-顺延 / 乙-拒 | 各 1 格（共有那格） | 各 6 格红，与今天相同 | 没有 | **打中**：F 那个 txg 上一条根都没有的空档，条件不成立，照样红 |
| 丙-只在抬 F | 27 格红 | 14 格红，含 I-2.1/I-4.8/I-7.4 | **有** | **打中**：再挂载就回到红；E9 上复用候选版本的单元 |
| 丙-抬 F 与挂载 | 5 格红（同甲） | 8 格红，含 I-2.1/I-3.10/I-4.8/I-7.4；E10 另 2 格 I-2.1/I-4.8/I-5.1/I-7.4 | **有** | **打中**：复用只由记录施加出来的那一版引用的单元；候选根一时读不出时复用它的单元 |
| 丁-并隔离集 | 抬前就红：978 格里只剩 88 格抬前绿 | 6 格红 | 没有 | **打中**：隔离位不进已分配，并进去就多算 |
| 丁-并隔离集中已释放的 | 152 格红 | 6 格红 | 没有 | **打中**：最小复现差的 10 槽里 8 槽只被 B 引用，不在任何隔离集里 |
| 丁-并 F 之下仍在 defer 的 | 1 格（共有那格） | 0 格 | 没有 | 没打中（我的全部历史上） |
| 第 ② 行 | 造出了被抛弃根带着更高 F（E4，F=8，系统配置里 0） | — | — | **两个候选结局不同**：不算 ⇒ 这份盘面上 I-7.12 红、回退到 3–7 做得成；算进 ⇒ I-7.12 绿、回退到 3–7 与抬到 8 以下都被拒 |
| 第 60 行 C379 | 整段只被被抛弃根独占：184 格 0 格（E5） | — | — | 整段独占没造出；**段里部分隔离 + 同段别的单元回收**，产品入口上自然造得出两个口径不一致（E11，60 格里 44 格） |

## 一、历史与取样（X1 的全部历史）

全在内存稀疏盘上（`SparseBlockDevice`，4 GiB 两盘，参数同 `tests/common` 的 `parameters()`）。每个起点状态建一次，之后每格从内存里的那一份 `fork` 出来；崩溃状态按录制流前缀取（一段历史取一个崩溃点，不是层 0 枚举，见「这条腿自己的限度」）。每一格：抬 F（只供测试强制进入的那一档，或产品入口）之后，**用户后缀放开扫**：覆盖写 2 次；再可写挂载后覆盖写 2 次；正常卸载、再挂载、覆盖写 1 次；回退到环里每一条没被抛弃、低于现行版本的根各一次、再覆盖写 1 次。每一步之后都跑池级 checker。

| 组 | 起点（只固定前缀与故障） | 扫的旋钮 |
|---|---|---|
| E1 | A(3)→B(4)→C(5)；C 的轮换没落盘就崩（前缀到 C 的根槽 FUA），可写挂载时 C 的根槽与数据单元一时读不出（调查员的乙）；或 C 整次落盘、另把见证 C 的系统配置槽每盘一槽一时读不出（调查员的甲） | 两种造法 × 再挂载与否 × 恢复后覆盖写 0–6 次 × F=3–9 |
| E3 | 被抛弃实例两条根：A→B→C(5)→D(6)→E(7)，E 的轮换没落盘就崩；挂载时 D、E 的根槽与数据单元、见证 D 的系统配置槽每盘一槽一时读不出；落到 C | 再挂载与否 × 覆盖写 0–6 × F=3–11（空档是 6、7 两格） |
| E4 | 被抛弃根带 F：实例 1 在 C(8) 上抬 F（强制到 4、5，或正常卸载抬到 8），系统配置先写（SysPre）、第一条空发布 txg 9 的根槽 FUA 之后崩；挂载时 txg 9 的根槽与单元一时读不出；`hide_syspre` 时 SysPre 写的那一槽每盘也一时读不出（挂载的取号写正好落回那一槽，F 从系统配置里没了） | 3 种抬法 × 藏不藏 SysPre（去掉两格重复，共 5 组）× 覆盖写 0–4 × F=3–12 与准入抬 F |
| E5 | E1 的造法，C 的文件 2500 或 30000 字节，C 之前预填 0–45 次覆盖写 | 看记账行「全空聚簇段数」与分配器能开的段（C379） |
| E6 | E1（两种）、E3、E4 的起点 | 只走产品入口：准入抬 F（抬到上限）与正常卸载，覆盖写 0–12 |
| E7 | A→B→C(5)→D(6)，C 的根槽永久读不出；或连 C 的 journal 记录、或连 C 的数据单元一起永久读不出 | 覆盖写 0–6 × F=3–9 |
| E8 | E1 乙造法再挂载 | 覆盖写 0–34（根环转过 B、C）× F=5、6 |
| E9 | A→B→C(5)，**崩在 C 的根槽 FUA 之前**（C 的单元与 journal 记录已落盘）；`hide_data=false`：什么都不藏，恢复施加 C 的记录，txg 5 是一版没有根槽的有效版本；`hide_data=true`：挂载时 C 的数据单元一时读不出，记录验点名单元失败不施加，txg 5 上没有任何版本 | 再挂载与否 × 覆盖写 0–6 × F=3–9 与准入抬 F |
| E10 | E1 乙造法，进程重开时 txg 6 或 7（实例 2 的有效根）的根槽从挂载到抬 F 都读回全 0，之后读得出 | 覆盖写 3–6 × F=5、6、7 |
| E11 | E1 两种造法（再挂载与否）、E3 | 覆盖写 0–14 后准入抬 F，再覆盖写 3 次、再挂载，看 C379 两个口径 |

每候选格数（`compare.py` 数的，不含 `-setup` 行与 E5）：批 1 978 格，批 2 276 格；E5 184 行、E11 60 行只在 today 下跑（它们看的是记账行与分配器，与第 43 行候选无关）。

## 二、第 43 行：各候选的反例

### 甲（checker 下界取「txg ≤ F_生效 的最大一条没被抛弃的根」）

**反例 1（一次崩溃，不带任何故障）**：E9 `hide_data=false`。A(3)→B(4)→C(5)，崩在 C 的根槽 FUA 之前；恢复施加 C 那条记录，落到 txg 5（实例 2，写行 6）；再可写挂载；覆盖写 4 次；抬 F 到 5。txg 5 那一版有效、只是没有根槽。甲按根环找「≤ 5 的最大一条没被抛弃的根」，找到的是 B(4)，把 B 放回遍历；B 引用、被 C 换下（释放代 5）的槽在抬 F 时按 max(5, 环里最旧有效根) 合法回收了，于是遍历多算、I-3.1 红。今天与丁-并 defer 在同一格全绿。

```
$ python3 cellview.py runs/b2-1-today-not-counted.log "exp=E9 remount=true hide_data=false k=4 F=5 " | cut -c1-330
cand=today abF=not-counted exp=E9 remount=true hide_data=false k=4 F=5 txg_before=14 roots_at_F=0 all_abandoned=false before=[] raise=ok(F=5 reclaimed 25 ceiling 11)=>[] post=[ow2:[]>[] | remount_ow2:[]>[]>[] | unmount_remount_ow1:[]>[] | rb3:rollback:TargetNotACandidate | rb4:rollback:TargetNotACandidate | rb6_ow1:[]>[] | rb7_o
$ python3 cellview.py runs/b2-2-jia-not-counted.log "exp=E9 remount=true hide_data=false k=4 F=5 " | cut -c1-330
cand=jia abF=not-counted exp=E9 remount=true hide_data=false k=4 F=5 txg_before=14 roots_at_F=0 all_abandoned=false before=[] raise=ok(F=5 reclaimed 25 ceiling 11)=>[I-3.1] post=[ow2:[I-3.1]>[I-3.1] | remount_ow2:[I-3.1]>[I-3.1]>[I-3.1] | unmount_remount_ow1:[]>[] | rb3:rollback:TargetNotACandidate | rb4:rollback:TargetNotACandi
$ grep -h "E9-setup remount=true hide_data=false" runs/b2-1-today-not-counted.log | cut -c1-260
AF-CELL cand=today abF=not-counted exp=E9-setup remount=true hide_data=false C=txg 5 (no root slot, data units hidden false) chosen txg 5 new instance 2 row txg 6 isolated [0, 0] rewritten_hidden 0 | chosen txg 7 new instance 3 row txg 8 isolated [0, 0] rewrit
```

同一组里甲命中的全部格（`hide_data=false`，再挂载与否 × 覆盖写 3–6 × F=5）：8 格，每格抬后与每段用户后缀都红，只有正常卸载那一段绿（卸载把 F 抬到现行版本，B 出了候选集）。

**反例 2（单点介质故障）**：E7 `RootSlot`。A→B→C(5)→D(6)，C 的根槽一个扇区永久读不出，可写挂载落到 D；覆盖写 3–6 次、抬 F 到 5：今天 4 格全绿（checker 把 C 那一版按「由记录施加出来的那一版」并进候选集），甲 4 格 I-3.1 红，机理同反例 1。

```
$ python3 cellview.py runs/b1-1-today-not-counted.log "exp=E7 what=RootSlot k=4 F=5 " | cut -c1-260
cand=today abF=not-counted exp=E7 what=RootSlot k=4 F=5 txg_before=12 roots_at_F=0 all_abandoned=false before=[] raise=ok(F=5 reclaimed 25 ceiling 9)=>[] post=[ow2:[]>[] | remount_ow2:[]>[]>[] | unmount_remount_ow1:[]>[] | rb3:rollback:TargetNotACandidate | rb
$ python3 cellview.py runs/b1-2-jia-not-counted.log "exp=E7 what=RootSlot k=4 F=5 " | cut -c1-260
cand=jia abF=not-counted exp=E7 what=RootSlot k=4 F=5 txg_before=12 roots_at_F=0 all_abandoned=false before=[] raise=ok(F=5 reclaimed 25 ceiling 9)=>[I-3.1] post=[ow2:[I-3.1]>[I-3.1] | remount_ow2:[I-3.1]>[I-3.1]>[I-3.1] | unmount_remount_ow1:[]>[] | rb3:rollb
```

四句：
1. 分辨臂：分辨。今天、乙、丁-并 defer 在这些格上绿，甲与丙两种红。
2. 被判的系统看不看得到：看得到。checker 已经在读 journal 里「由记录施加出来、根槽从没落盘的那一版」（`versions_applied_only_by_records`），甲的「最近一条」只数根环里的根，没数它。
3. 满足判据哪一句：背景材料「五、跑前写死的判据」第 43 行那句「一个候选在 X1 的全部历史上 I-3.1 与 I-7.4 都不在合法状态上红」的前半句（I-3.1 在合法状态上红）。反例 1 的盘面是录制流的一个前缀、之后只有正常操作，合法；反例 2 带一个永久坏扇区，合不合法按介质故障的口径另判。没有复用：这里的红是 checker 多算（遍历 > 记账），盘上没有单元被复用。
4. 改法在这几格上中不中：甲按字面在这 12 格上照样中。我提的改法「最近一条」把由记录施加出来的版本也算进去（≤ F 的最大一版，不只根）：反例 1、2 里 txg 5 那一版就在，下界停在 5，B 不进遍历——**推的**，没实现、没跑，被攻过零轮；它在 E9 `hide_data=true`（txg 5 上没有任何版本）上仍然把 B 放回遍历，那一格该绿，也没跑。

### 乙（抬 F 的目标 txg 上的根全属于被抛弃实例时顺延或拒）

**反例（一次崩溃加一时读不出，不坏系统配置）**：E9 `hide_data=true`。A→B→C(5)，崩在 C 的根槽 FUA 之前；可写挂载时 C 的数据单元一时读不出，C 的记录不施加，落到 B（实例 2，写行 6）；再挂载；覆盖写 4 次；抬 F 到 5。txg 5 上**一条根都没有**，也没有任何版本。乙的条件「目标 txg 上的根全属于被抛弃实例」在空集上不成立（副本里按「有根且全被抛弃」写，与调查员匹配函数 `raised_floor_lands_only_on_abandoned_roots` 同一个判法），抬 F 照做，I-3.1 红，机理与最小复现相同：B 独占、释放代 6 的槽在记账里、不在遍历里。乙-顺延、乙-拒与今天逐格相同（6 格红）；甲、丙-抬 F 与挂载、丁-并 defer 在这 6 格上绿。

```
$ python3 cellview.py runs/b2-4-yi_refuse-not-counted.log "exp=E9 remount=true hide_data=true k=4 F=5 " | cut -c1-300
cand=yi_refuse abF=not-counted exp=E9 remount=true hide_data=true k=4 F=5 txg_before=14 roots_at_F=0 all_abandoned=false before=[] raise=ok(F=5 reclaimed 13 ceiling 11)=>[I-3.1] post=[ow2:[I-3.1]>[I-3.1] | remount_ow2:[I-3.1]>[I-3.1]>[I-3.1] | unmount_remount_ow1:[]>[] | rb3:rollback:TargetNotACandi
$ python3 cellview.py runs/b2-2-jia-not-counted.log "exp=E9 remount=true hide_data=true k=4 F=5 " | cut -c1-300
cand=jia abF=not-counted exp=E9 remount=true hide_data=true k=4 F=5 txg_before=14 roots_at_F=0 all_abandoned=false before=[] raise=ok(F=5 reclaimed 13 ceiling 11)=>[] post=[ow2:[]>[] | remount_ow2:[]>[]>[] | unmount_remount_ow1:[]>[] | rb3:rollback:TargetNotACandidate | rb4:rollback:TargetNotACandid
```

这一形在随机历史里也不会按已知红归类：`crates/singlefs-harness/src/history.rs` 的注释写「（`raised_floor_lands_only_on_abandoned_roots`）；别的步、读不出、那个 txg 上没有根，都是 None。」（`/tmp/claude-1000/abandoned-floor-r1/tree/crates/singlefs-harness/src/history.rs:1484`），所以它会报成新发现。

四句：
1. 分辨臂：分辨（乙两种红，甲、丙-抬 F 与挂载、丁-并 defer 绿）。
2. 看不看得到：看得到。抬 F 那一刻读得到根环（txg 5 上没根）与 journal（txg 5 那条记录没施加、落到的是 B）。
3. 判据分句：同甲那一条，I-3.1 在合法状态上红。
4. 改法：乙按字面（「根全属于被抛弃实例」）在这 6 格上照样中；把条件改成「目标 txg 上没有有效版本（没有没被抛弃的根，也没有由记录施加出来的版本）」就罩住它——**推的**，没跑，零轮。

乙在我的其余历史上：E1、E3、E4、E8 里 F 落在「根全被抛弃」那一格的全部格，乙-顺延抬到下一个有效根、全绿；乙-拒拒掉、盘面不变、全绿（`diffcells.py` 归组：E1 12 格、E3 12 格、E4 5 格、E8 15 格）。

### 丙（抬 F 时把只被 F 之下的根与被抛弃根引用、已释放的槽回收）

副本里的丙：回收门槛取 max(「txg ≥ 新 F 的最低一条没被抛弃的根」, 环里最旧有效根)；被抛弃根引用的槽照旧由影子账在回收之前按新 F 补隔离，所以「回收」只清分配位、隔离位还挡着——这是我对「写明它与影子账相容」的读法。最小复现（E1 乙、再挂载、覆盖写 4、F=5）上：今天回收 13 个落点，丙两种都回收 22 个（多出 50176 那一条跨 2 与 50257–50264 那 8 条，按盘 0 报），抬后绿；50176 同时被被抛弃的 C 引用，隔离位留着，之后没有被发出去（被抛弃根那一半 I-7.4 没红）。

**反例 1（丙只在抬 F 时做）**：同一格，抬 F 之后再可写挂载：挂载从盘上重建分配器，回收门槛回到 max(F_生效, 环里最旧有效根) = 5，那几个释放代 6 的槽又回到 defer，I-3.1 再红。

```
$ python3 cellview.py runs/b1-5-bing-not-counted.log "exp=E1 variant=CrashBeforeRotation remount=true k=4 F=5 " | cut -c1-330
cand=bing abF=not-counted exp=E1 variant=CrashBeforeRotation remount=true k=4 F=5 txg_before=14 roots_at_F=1 all_abandoned=true before=[] raise=ok(F=5 reclaimed 22 ceiling 11)=>[] post=[ow2:[]>[] | remount_ow2:[I-3.1]>[I-3.1]>[I-3.1] | unmount_remount_ow1:[]>[] | rb3:rollback:TargetNotACandidate | rb4:rollback:TargetNotACandidat
```

所以丙要成立，挂载重建与挂载之内的回收都得用同一个门槛，也就是改 D16（发布语义） 已定项 1 的可再分配谓词本身（那一行原文在 `.claude/kb/decisions/16-发布语义.md:33`：「| 可再分配 | 已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根) |」）；这一半归 X2，我只报丙-抬 F 与挂载在副本上的结局。

**反例 2（丙-抬 F 与挂载，一次崩溃、不带故障，复用候选版本的单元）**：E9 `hide_data=false`（与甲的反例 1 同一段历史）。txg 5 是一版只由记录施加出来、没有根槽的有效版本；丙的门槛只看根环，「txg ≥ 5 的最低一条没被抛弃的根」是 6，于是把 txg 5 那一版引用、释放代 6 的单元回收了；之后的覆盖写把它们发出去，txg 5 那一版被写坏：I-2.1、I-3.10、I-4.8、I-7.4 一起红。这是真复用，不只是 checker 多算。

```
$ python3 cellview.py runs/b2-6-bing_all-not-counted.log "exp=E9 remount=true hide_data=false k=4 F=5 " | cut -c1-420
cand=bing_all abF=not-counted exp=E9 remount=true hide_data=false k=4 F=5 txg_before=14 roots_at_F=0 all_abandoned=false before=[] raise=ok(F=5 reclaimed 34 ceiling 11)=>[I-3.1] post=[ow2:[I-2.1,I-3.1,I-3.10,I-4.8,I-7.4]>[I-2.1,I-3.1,I-3.10,I-4.8,I-7.4] | remount_ow2:[I-3.1]>[I-2.1,I-3.1,I-3.10,I-4.8,I-7.4]>[I-2.1,I-3.1,I-3.10,I-4.8,I-7.4] | unmount_remount_ow1:[]>[] | rb3:rollback:TargetNotACandidate | rb4:rollback:
```

**反例 3（丙-抬 F 与挂载，候选根一时读不出）**：E10。E1 乙造法（不再挂载）之后进程重开，txg 7（实例 2 的一条有效根）的根槽从挂载到抬 F 都读回全 0（挂载那一刻就读坏的槽当没有根，D16 已定项 1「根槽这一次读坏」那一行），覆盖写 4 或 5 次、抬 F 到 7：丙把「≥ 7 的最低一条可读的有效根」取成 8，txg 7 引用的单元被回收、再被覆盖写发出去；txg 7 的根槽读得出之后 I-2.1、I-4.8、I-5.1、I-7.4 红，再挂载照样红。今天在同一格全绿（按 F 回收只到释放代 ≤ 7，与哪条根读得出无关）。

```
$ grep -h "exp=E10 hidden_txg=7 k=4 F=7 " runs/b2-1-today-not-counted.log runs/b2-6-bing_all-not-counted.log | grep -o "cand=[a-z_]* .*readable_again=[^ ]*" | cut -c1-400
cand=bing_all abF=not-counted exp=E10 hidden_txg=7 k=4 F=7 instance=4 steps=[] raise=ok(F=7 reclaimed 39 ceiling 11) after_raise_while_hidden=[I-3.1] after_3ow_while_hidden=[I-2.1,I-3.1,I-4.8,I-5.1,I-7.4] suffix=["ow", "ow", "ow"] restored=true readable_again=[I-2.1,I-3.1,I-4.8,I-5.1,I-7.4]
cand=today abF=not-counted exp=E10 hidden_txg=7 k=4 F=7 instance=4 steps=[] raise=ok(F=7 reclaimed 30 ceiling 11) after_raise_while_hidden=[] after_3ow_while_hidden=[] suffix=["ow", "ow", "ow"] restored=true readable_again=[]
```

另外，丙两种在 E7 `RootSlot` 上与甲同样 4 格 I-3.1 红（txg 5 那一版只剩 journal 记录，丙按根环把它引用的槽回收）。

四句（反例 2、3）：
1. 分辨臂：分辨（今天、乙、丁-并 defer 在这些格上绿）。
2. 看不看得到：反例 2 看得到（journal 里那条施加过的记录）；反例 3 在抬 F 那一刻看不到 txg 7（读回全 0，按 D16 那一行当没有根）——判别子在那一刻观测不到，所以「只被 F 之下的根引用」这句话本身在抬 F 时判不出来，按「判别子观测不到」那一形记：丙要的判断依赖一条此刻读不出的根。今天的回收门槛只用 F 与根环里最旧有效根，不需要知道 F 之上每条根引用什么，所以不受它影响。
3. 判据分句：I-7.4 在合法状态上红，并且复用了候选版本引用的单元（背景材料「五」第 43 行那句的两半都中）。
4. 改法：丙按字面在这几格上中。把「最低一条根」换成「最低一版（含由记录施加出来的）」能罩住反例 2（推的，零轮）；反例 3 罩不住，除非抬 F 时有根槽读不出就不按丙回收（推的，零轮）。

### 丁（改 I-3.1 口径）

**丁-并影子账隔离集**：影子账隔离的槽不进已分配（`/tmp/claude-1000/abandoned-floor-r1/tree/crates/singlefs-core/src/allocator.rs:573`：「隔离一个只被被抛弃根引用的落点：不进已分配、不进 defer、不动空闲计数」），把它们并进「遍历」一边就是多算。批 1 里 978 格只剩 88 格抬 F 之前是绿的；最小复现在抬 F 之前就红。
**丁-只并隔离集里当前账有已释放记录的**：最小复现差的 10 槽里 50257–50264 那 8 槽只被 B 引用（调查员 2.1 的明细，我这里同一格抬后仍红），不在任何被抛弃根的引用里，并不进来；批 1 152 格红。
**丁-并 F 之下仍在 defer 的（释放代 > F_生效、被 F 之下没被抛弃的根引用）**：批 1、批 2 全部 1254 格里，抬前绿的格没有一格因它变红，第 43 行那一形的全部格（E1、E3、E4、E8、E9 `hide_data=true`、E10 里 F=5 那些）都变绿。没打中。它的限度写在「没打中的形状」。

### 第 43 行那一形走不走得到产品入口

只供测试强制进入的那一档（`raise_rollback_floor` 直接给 F）是我、调查员与随机历史生成器进空档的唯一入口。产品入口两条：准入抬 F 抬到上限，上限按 `/tmp/claude-1000/abandoned-floor-r1/tree/crates/singlefs-core/src/mount.rs:1830` 「Some(newest_on_every_device.min(fourth_newest))」取，两个量都是某条有效根的 txg；正常卸载抬到现行版本的 txg。E6 里 52 格准入抬 F 与 52 格正常卸载、E4 与 E9 里每格的准入抬 F，F 落到的 txg 上没有一格是「根全被抛弃」或「没有根」；E9 里准入抬 F 落在 8、11 这类有效根上，全绿。这一条是观测到的，另有一句推的：上限的两个量按定义是有效根的 txg，所以准入抬 F 落不进空档，F 进空档只剩测试档与「F 落下之后那条根才被抛弃」一条路，后一条我在 E4 里试了（见第 ② 行），F 带进去的都是那次恢复落到的根的 txg 或更低，没造出 F 落在被抛弃 txg 上的盘面。

## 三、第 ② 行：被抛弃根带着更高的 F

**造出来了**（调查员没造出的那一形）：E4 `mode=Unmount hide_syspre=true`。实例 1：A(3)，预填 4–6，B(7)，C(8)；正常卸载，SysPre 先把 F=8 写进每块盘一个系统配置槽、过屏障，第一条空发布 txg 9（带 F=8）根槽 FUA 落盘后、它的轮换之前崩。可写挂载时 txg 9 的根槽与单元、以及 SysPre 写的那两槽一时读不出（与调查员甲那一形同类：每盘一个系统配置槽读不出）；择根落到 C(8)，实例 2 取号的系统配置写正好落回 SysPre 那一槽，F=8 从系统配置里没了。之后再可写挂载（实例 3），txg 9 读得出、按实例表判被抛弃，带着 F=8；别的根与系统配置都是 F=0。

```
$ grep -h "E4-setup mode=Unmount hide_syspre=true" runs/b1-1-today-not-counted.log | grep -o "F_eff_impl=.*" | cut -c1-330
F_eff_impl=0 ring=[(0, 0, 0, false), (1, 1, 0, false), (1, 2, 0, false), (1, 3, 0, false), (1, 4, 0, false), (1, 5, 0, false), (1, 6, 0, false), (1, 7, 0, false), (1, 8, 0, false), (1, 9, 8, true), (2, 10, 0, false), (2, 11, 0, false), (3, 12, 0, false), (3, 13, 0, false)]
$ grep -h "E4-setup mode=Unmount hide_syspre=true" runs/b1-10-today-count.log | grep -o "F_eff_impl=[0-9]*"
F_eff_impl=8
```

两个候选的结局（同一段历史，覆盖写 0–4 次；原样输出只贴覆盖写 0 次那几格，4 次那几格见 `runs/b1-1-today-not-counted.log`、`runs/b1-10-today-count.log` 里 `k=4`）：

| | 不算（今天） | 算进 |
|---|---|---|
| F_生效 | 0 | 8 |
| 这份盘面上 checker | **I-7.12 红**：系统配置 F 0 低于同盘被抛弃根 txg 9 带的 8（`/tmp/claude-1000/abandoned-floor-r1/tree/crates/singlefs-checker/src/walk.rs:4132` 起按盘上每条根取 F 最大的那条，不滤被抛弃的） | 绿（新实例写系统配置时带的就是 8） |
| 回退到 3–7 | 做得成，之后覆盖写仍 I-7.12 红 | 拒（`TargetNotACandidate`） |
| 强制抬 F 到 3–7 | 上限之内的那几格做得成（覆盖写 0 次时上限 5，4 次时上限 14），抬后到 7 为止仍 I-7.12 红，抬到 8 起绿 | 拒（`RequestedFloorBelowTheEffectiveFloorWhoseRaiseIsUndecided`） |
| 准入抬 F | 抬到上限 5，仍 I-7.12 红 | 已在上限 8，不抬 |
| 正常卸载再挂载 | 绿 | 绿 |
| 强制抬 F 到 9（txg 9 空档） | I-3.1 红（覆盖写 4 次那一格） | I-3.1 红（同一格） |

```
$ python3 cellview.py runs/b1-1-today-not-counted.log "exp=E4 mode=Unmount hide_syspre=true k=0 F=none " | cut -c1-300
cand=today abF=not-counted exp=E4 mode=Unmount hide_syspre=true k=0 F=none txg_before=13 F_eff_impl=0 before=[I-7.12] raise=- post=[ow2:[I-7.12]>[I-7.12] | remount_ow2:[I-7.12]>[I-7.12]>[I-7.12] | unmount_remount_ow1:[]>[] | rb3_ow1:[I-7.12]>[I-7.12] | rb4_ow1:[I-7.12]>[I-7.12] | rb5_ow1:[I-7.12]>[I
$ python3 cellview.py runs/b1-1-today-not-counted.log "exp=E4 mode=Unmount hide_syspre=true k=0 F=5 " | cut -c1-300
cand=today abF=not-counted exp=E4 mode=Unmount hide_syspre=true k=0 F=5 txg_before=13 roots_at_F=1 all_abandoned=false before=[I-7.12] raise=ok(F=5 reclaimed 25 ceiling 5)=>[I-7.12] post=[ow2:[I-7.12]>[I-7.12] | remount_ow2:[I-7.12]>[I-7.12]>[I-7.12] | unmount_remount_ow1:[]>[] | rb3:rollback:Target
$ python3 cellview.py runs/b1-1-today-not-counted.log "exp=E4 mode=Unmount hide_syspre=true k=0 F=8 " | cut -c1-300
cand=today abF=not-counted exp=E4 mode=Unmount hide_syspre=true k=0 F=8 txg_before=13 roots_at_F=1 all_abandoned=false before=[I-7.12] raise=raise:RollbackFloorAboveCeiling=>[I-7.12] post=[]
$ python3 cellview.py runs/b1-10-today-count.log "exp=E4 mode=Unmount hide_syspre=true k=0 F=none " | cut -c1-300
cand=today abF=count exp=E4 mode=Unmount hide_syspre=true k=0 F=none txg_before=13 F_eff_impl=8 before=[] raise=- post=[ow2:[]>[] | remount_ow2:[]>[]>[] | unmount_remount_ow1:[]>[] | rb3:rollback:TargetNotACandidate | rb4:rollback:TargetNotACandidate | rb5:rollback:TargetNotACandidate | rb6:rollback
$ python3 cellview.py runs/b1-10-today-count.log "exp=E4 mode=Unmount hide_syspre=true k=0 F=5 " | cut -c1-300
cand=today abF=count exp=E4 mode=Unmount hide_syspre=true k=0 F=5 txg_before=13 roots_at_F=1 all_abandoned=false before=[] raise=raise:RequestedFloorBelowTheEffectiveFloorWhoseRaiseIsUndecided=>[] post=[]
$ python3 cellview.py runs/b1-10-today-count.log "exp=E4 mode=Unmount hide_syspre=true k=0 F=8 " | cut -c1-300
cand=today abF=count exp=E4 mode=Unmount hide_syspre=true k=0 F=8 txg_before=13 roots_at_F=1 all_abandoned=false before=[] raise=ok(F=8 reclaimed 0 ceiling 8)=>[] post=[ow2:[]>[] | remount_ow2:[]>[]>[] | unmount_remount_ow1:[]>[] | rb3:rollback:TargetNotACandidate | rb4:rollback:TargetNotACandidate 
```

两边的依据都在 D16（发布语义） 已定项 1「生效」取 SysPre 那一段：F=8 在崩溃前没落满每块盘（txg 9 只在盘 1，第二条空发布没发），按「生效」它没生效；unmount 在第一次发布之前回收并扣住释放代 ≤ 8 的槽，这些槽在崩溃后的新实例里回到 txg 8 那一版的账，没有被发出去——所以「不算」让 3–7 回到回退候选集，在我这段历史上没有复用（I-7.4 没红），但 I-7.12 的读法（盘上每条根、含被抛弃的）与 F_生效 的读法（只数有效根）不一致，checker 在合法盘面上报红。「算进」让 checker 一致，代价是新实例拿不到 3–7 这几个它本来能退的状态。

四句：
1. 分辨臂：分辨（两个候选在 I-7.12、回退、抬 F 三处结局不同）。
2. 看不看得到：看得到，被抛弃根读得出、带着 F=8。
3. 判据分句：背景材料「五」第 ② 行「X1 造出「被抛弃根带着更高 F」的历史、两个候选结局不同 ⇒ 按结局交用户」。另外「不算」在这份盘面上 I-7.12 红：这是 checker 与 F_生效 两个读法之间的口径差，不在第 ② 行的判据里，是一格新的；改法有两条：I-7.12 滤掉被抛弃的根（推的，没跑），或第 ② 行取「算进」（量过，上表）。
4. 这两条改法各修哪一格：见「第 43 行几个改法各修哪一格」表末两行。

没造出的：被抛弃根带的 F 高于恢复落到的那条根的 txg。推的理由：SysPre 写的那一槽与之前那次轮换写的那一槽都见证 ≥ 抬 F 时现行版本的 txg，两槽都读不出时挂载没有系统配置；只藏一槽，另一槽见证的 txg 就逼着择根落在不低于 F 的地方（C554 乙）。所以在我能造的历史上，「算进」得到的 F_生效 总有一条有效根兑现得了（E4 三种抬法：F_生效 分别是 4、5、8，C 在 txg 8、有效）。

## 四、第 60 行：C379 在自然历史里

**整段只被被抛弃根独占：没造出**。E5：两种造法 × C 的文件 2500 或 30000 字节（一个数据单元装得下的上限附近；70000 字节在 `publish_overwrite` 报 `ContentExceedsDataUnit`）× 预填 0–45 次，共 184 行；首次挂载后、再挂载后（影子账隔离了 C 的单元，每盘 14 槽）、再覆盖写一次后三个时刻，「分配位全空而开不出来的段」一段都没有：

```
$ grep -h "exp=E5 " runs/b1-1-today-not-counted.log | grep -o "empty_but_blocked=[0-9]* wholly_blocked=[0-9]*" | sort | uniq -c
   1104 empty_but_blocked=0 wholly_blocked=0
```

一次崩溃恢复只抛弃一条根（被抛弃实例的单元每盘 14 槽，两条根 28 槽，E3 再挂载那一行 `isolated [28, 28]`），装不满 64 槽的一段；两条以上要藏两个系统配置槽，挂载就没有系统配置可用（推的，见第 ② 行末段）。

**段里一部分被隔离、同段别的单元回收之后：造出来了，走产品入口，不注入**。E11：E1 乙造法、再挂载（影子账隔离 C 的单元，每盘 14 槽；抬 F 时按新 F 重算成 16 槽，其中 12 槽在 [50240, 50304) 那一段），覆盖写 4 次起，准入抬 F（抬到上限 11）把同段里别的单元回收：那一段分配位全空，记账行「全空聚簇段数」把它数进去（3309），分配器因为隔离位不开它，能开的最低段是 50304。之后的覆盖写照样写这个数进记账行。

```
$ grep -h "exp=E11 variant=CrashBeforeRotation remount=true k=4 " runs/b3-1-today-e11.log | grep -o "raise=[^|]*| after_raise: dev0 [^;]*"
raise=ok(F=11 reclaimed 69) v=[] | after_raise: dev0 row_empty_segments=3309 lowest_openable=Some(50304) empty_but_blocked=1 wholly_blocked=0 isolated=16 examples=["50240:12"] 
$ grep -h "exp=E11 variant=CrashBeforeRotation remount=true k=3 " runs/b3-1-today-e11.log | grep -o "raise=[^|]*| after_raise: dev0 [^;]*"
raise=ok(F=4 reclaimed 13) v=[] | after_raise: dev0 row_empty_segments=3307 lowest_openable=Some(50496) empty_but_blocked=0 wholly_blocked=0 isolated=14 examples=[] 
$ grep -h "exp=E11 " runs/b3-1-today-e11.log | grep -c "after_raise: dev0 [^;]*empty_but_blocked=1"; grep -c "exp=E11 " runs/b3-1-today-e11.log
44
60
```

（`empty_but_blocked` = 分配位全空、但段里有隔离位或扣住位的段数，用例里现数，不是 checker 的判定；盘 1 逐项相同。）四组起点（E1 乙再挂载、E1 甲再挂载、E1 乙不再挂载、E3 两条根）× 覆盖写 0–14：60 格里 44 格在准入抬 F 之后两个口径差 1 段，四组都从覆盖写 4 次起；差 1 段的那一段在四组里都是 [50240, 50304)。checker 不判「全空聚簇段数」这一行，所以这 44 格 checker 都不报。

按背景材料「五」第 60 行那句「X1 自然历史里造不出整段只被被抛弃根独占 ⇒ C379 记「只在注入下走得到」……造得出 ⇒ 按结局判」：整段独占这一形我没造出；两个口径不一致这件事本身在自然历史里造得出来，不要注入。这是字面上的两个分句之间的缝：判据写的是「整段独占」，C379 的毛病是「两个口径给同一件事不同的数」，后者部分隔离就够。

三条改法在这 44 格上（都没实现，**推的**，按同一格的 `empty_but_blocked` 算）：「记账行改看隔离位与扣住位」这一行少数 1 段、与分配器相同；「两个量分成两个名字各自登记」数不变、口径差登记下来；「不动」照旧差 1 段。改的字节：记账树里 `(11, 0, 设备, 代)` 那两条的 8 字节值（`.claude/kb/layout/01-first-txn.md:297`：「全空聚簇段数 (11, 0, 0, 3)、(11, 0, 1, 3)」），key 不变；第一个事务那一版没有隔离位与扣住位，值仍是 3310（`.claude/kb/layout/01-first-txn.md:298`：「全空聚簇段数 3310 / 3310（3312 段减 [50176, 50240) 与开放段 [50240, 50304)）」）——只有带隔离或扣住的盘面上值变。

## 五、第 43 行几个改法各修哪一格

「绿 / 红」= 那一格抬 F 之后与全部用户后缀里 checker 的结局；「量过」= 副本上跑过、日志在 `runs/`；「推的」= 按代码推、没实现没跑。我自己提的三条（甲-含记录版本、乙-无有效版本、丙-含记录版本）只在我的推理里，**被攻过零轮**。

| 格（历史） | 今天 | 甲 | 乙-顺延 / 乙-拒 | 丙-抬 F 与挂载 | 丁-并 F 之下仍在 defer 的 | 甲-含记录版本（我提的） | 乙-无有效版本（我提的） |
|---|---|---|---|---|---|---|---|
| 最小复现与同形（E1、E3、E4、E8 里 F 落在根全被抛弃的 txg） | 红（量过） | 绿（量过） | 绿（量过） | 绿（量过） | 绿（量过） | 绿（推的） | 绿（推的） |
| F 那个 txg 上没有根、也没有版本（E9 `hide_data=true`） | 红（量过） | 绿（量过） | 红（量过） | 绿（量过） | 绿（量过） | 绿（推的） | 绿（推的） |
| F 那个 txg 上只有由记录施加出来的版本（E9 `hide_data=false`，一次崩溃不带故障） | 绿（量过） | 红 I-3.1（量过） | 绿（量过） | 红，复用（量过） | 绿（量过） | 绿（推的） | 绿（推的） |
| C 的根槽永久读不出、记录在（E7 `RootSlot`） | 绿（量过） | 红 I-3.1（量过） | 绿（量过） | 红 I-3.1（量过） | 绿（量过） | 绿（推的） | 绿（推的） |
| F 之上一条有效根在抬 F 时读不出（E10） | F=5 那几格红，F=6、7 绿（量过） | 同今天，F=5 绿（量过） | 同甲（量过） | F=7 两格复用红（量过） | F=5 绿，其余绿（量过） | 同甲（推的） | 同乙（推的） |
| 抬 F 之后再挂载（E1 等的后缀） | 红（量过） | 绿（量过） | 绿（量过） | 丙-只在抬 F：红；丙-抬 F 与挂载：绿（量过） | 绿（量过） | 绿（推的） | 绿（推的） |
| 第 ② 行 E4 `hide_syspre=true` 的 I-7.12 | 红（量过） | 红（量过，甲不碰 I-7.12） | 红（量过） | 红（量过，丙不碰） | 红（量过） | 红（推的） | 红（推的） |

末行那一格只有两条改法碰得到：第 ② 行取「算进」（量过：`today/count`、`jia/count` 在那一组抬前都绿），或 I-7.12 滤掉被抛弃的根（推的，没实现）。

## 六、没打中的形状

| 试过的形状 | 取样 | 结局 |
|---|---|---|
| 丁-并 F 之下仍在 defer 的 | 批 1 978 格、批 2 276 格，全部历史与后缀 | 没有一格因它变红，第 43 行那一形全部变绿。它是 checker 单边放宽：那几个槽仍在记账的已分配里，要等 F 抬过释放代或根环转走 F 之下那条根才回收（E8 里今天在覆盖写 15 次起 F=5 那一格自己变绿）；放宽的是「F_生效 之下仍在 defer、释放代 > F_生效」这一类，这一类只在 F 那个 txg 上没有有效版本时才非空（推的：有一版 T=F 时，被 F 之下的根引用、释放代 > F 的槽也被 T 引用，已在遍历里）。它会不会盖住「实现该回收而没回收」那种错，要拿「抬 F 一个都不回收」的变异试（代码三方第二轮攻方腿试过的那种），我没试 |
| 被抛弃根引用的单元被复用（被抛弃根那一半 I-7.4） | 全部候选 × 全部格 | 抬前绿的格里只有 1 格红（E3 不再挂载、不抬 F、恢复后直接覆盖写：C554 那一形，所有候选相同，不分辨）；丙回收 50176 那一跨 2 之后隔离位挡住，没有发出去 |
| 产品入口进空档 | E6 104 格、E4 25 格、E9 28 格准入抬 F 与卸载 | 0 格落在根全被抛弃或没有根的 txg 上 |
| 候选根一时读不出（E10）对甲、乙、丁 | 24 格 × 11 个开关 | 与各自在 E1 上的结局相同，只有丙两种多出复用 |
| 根环转圈（E8） | 覆盖写 0–34 × F=5、6 | 今天 F=5 那一格覆盖写 4–14 次红、15 次起绿；乙、甲、丙-抬 F 与挂载、丁-并 defer 全绿 |
| 被抛弃根带的 F 高于恢复落到的那条根 | E4 三种抬法 × 藏不藏 SysPre | 没造出，理由见第 ② 行末段（推的） |
| 整段只被被抛弃根独占 | E5 184 行 | 没造出 |
| E7 另两形：C 的根槽与 journal 记录都永久读不出；C 的根槽与数据单元都永久读不出 | 各 7 × 8 格 | 抬 F 之前今天就红（前者 I-3.1，后者 I-2.1、I-4.8、I-7.4），不是抬前绿的格，不算进任何候选的打中；前者甲与丙-抬 F 与挂载在 F=5 上变绿。两个永久坏处算不算合法状态我没判，交主 agent |

## 七、这条腿自己的限度

- **崩溃状态没有按层 0 的枚举域取**。定义要的 `enumerate_layer0` / `enumerate_layer0_versions` 在 checker 档包 `singlefs-checker-tier` 的 `crash.rs` 里；harness 档的测试引不到它（checker 档依赖 harness，反过来会成环），跑 checker 档包的测试是重型测试，这一轮不跑。所以每段历史我只取了一个手挑的崩溃点（录制流前缀到某次根槽 FUA 为止、或到它的前一步），再加一时读不出。`closed_form_state_count` 没算，枚举状态数 0。每段历史的其余崩溃状态一个没跑——这是欠的，交主 agent：要补就在 checker 档里给 E1、E3、E4、E9 的起点历史各录一条流、全枚举、每个状态恢复后先跑 checker、再接这里的用户后缀。
- 取样没缩：挂钟先跑了一小段（pilot，today，18 个用例 118 秒），估全量 11 个开关约 25 分钟，在 40 分钟之内，照原样跑。每批的件数与用时在草稿目录 `progress.md`：批 1 11 件 114–178 秒一件，批 2 11 件 73–95 秒一件，E11 一件 7 秒。批不设限时。内存峰值 pilot 那一遍 1017884 kB（`/usr/bin/time -v`）。
- 乙-拒在副本里借 `RollbackFloorAboveCeiling` 报错（ceiling 写 u64::MAX 作记号），真做时要一个自己的错误成员；它的结局只看「拒了、盘面不变、checker 绿」，不受这个借用影响。
- 丙的副本只改了挂载重建与抬 F 两处回收门槛，挂载之内根环转圈时的回收（`record_root_written_by_this_process`）仍按 F_生效；E8 上丙-抬 F 与挂载全绿，这一处在我的历史上没显出差别，没单独试。
- 丁的三种都只改了 I-3.1 的「遍历」一边，I-3.11、I-5.2 没动。
- 「算进」两边（checker 的 `walk.rs` 与实现的 `recovery.rs`）一起改；只改一边的组合没跑。
- 用户后缀没放开的：两次抬 F 连着做、抬 F 之后再崩溃、多次回退连着做、准入推空发布在写路径里自己触发（我只直接调准入抬 F 的入口）。
- 我的用例在副本上跑，副本就是冻结副本加 `copy-vs-frozen.diff`；这些数不是入库装置上的数，要引得在入库的装置上重做。

## 八、没做什么

- 没判 X2、X3 两格，没判候选与已定分项相不相容（丙改可再分配谓词那一句只是指出它要改哪一行，相容不相容归 X2）。
- 没跑任何重型测试，没跑 checker 档包，没跑门禁。
- 没改仓里的 `crates/`；模型目录里只有 diff、用例、汇表脚本与原样日志，没有编译目录。
- 第 ② 行 E4 那一形要每块盘一个系统配置槽一时读不出，E7 要永久坏扇区；它们算不算合法状态按哪个故障口径判，我没判，只照实写了造法。
- 我提的三条改法（甲-含记录版本、乙-无有效版本、丙-含记录版本）与「I-7.12 滤掉被抛弃的根」都没实现、没跑，被攻过零轮。
