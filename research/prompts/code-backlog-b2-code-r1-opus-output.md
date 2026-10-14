# code-backlog-b2-code-r1 攻方腿（Opus）报告（2026-10-14）

攻击面：正文「三、格」K1–K4。被判代码只取冻结副本 `~/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/`（`crates/singlefs-core/src/mount.rs` 的 sha256 975e77af…，与 `research/prompts/code-backlog-b2-code-r1-snapshot/crates-sha256.txt` 那一行相同）。下面的代码行号都是冻结副本的。本报告里的数都是在冻结副本的拷贝上量的，不是入库装置上的数。

## 复跑

模型目录 `research/prompts/code-backlog-b2-code-r1-opus-model/`。整条约 10 分钟，超过子 agent 前台 4 分钟的上限，所以我是分段跑的；复跑交给主 agent，命令写在末尾「要主 agent 跑的命令」一节：

```
bash scripts/hosts/run-with-memory-cap.sh 16G bash research/prompts/code-backlog-b2-code-r1-opus-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs \
  --frozen /home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo \
  --work <新的空草稿目录>
```

跑完最后一行是 `name=rerun_compare tally_same=<yes|no> lines_same=<yes|no>`：它把 `<work>/tally.txt` 与 `out/tally.txt` 逐行比，把 `<work>/opus-lines.txt` 与 `out/opus-lines.txt` 逐行比。`OPUS_THREADS` 不设时按 31 个线程跑。

模型目录里每个文件的 sha256：

```
dd59b855dc235ea19b6a2551b2470dd00df5e7b4af7f043c07f0101646699ecf  ./out/anchor-counts.txt
840d583202daeacbf5741d89d2c01c20faeb929e2decad212ce9c37b0c8581de  ./out/k1-main.log
a8e0cd2bc1f4652e4e13b64bee0221432c6fda3a2f9aa9da38bdc19753822853  ./out/k2-k3.log
6f28e1f66ab15bbf5b2a97cbb783d4019ba25f287679e70504b7a98e3ab318da  ./out/opus-lines.txt
9a0038f75ce4e072cbab60e2435de1196f10df1cf3b79480d5bab6f8d92c454f  ./out/swap-core-lib.log
c3643a33e6f8bd90255d73e6835a3becf24c92ad2eee60cdf1aadf59d8c92ad9  ./out/tally.txt
3ab93f340a799c1f643dd79b45f488e74ef4276921ba5cec6a3d94ebe448e1f4  ./src/anchor_counts.py
27377b2c34d1c5ca5dd60526da32e9d1235b355e4900ddea82fd83334220bf37  ./src/harness-selected.txt
c1ab8402f1f4b88db5b06bdfd1e323b4ce024bc279459572cde1258661f94d04  ./src/instrument.py
a5ada62d9ec385c91a0d13684c6fb8e9e7d58cc2f427988e2efd9a9e7e4d6f59  ./src/k1-tests.rs
41e283366753ea18d46bf1c3d7fe6586a35404b9b50248e7435b8ee9752ceae0  ./src/opus_k2_isolation_made_by_a_rollback_release.rs
b743a19e7b122672f5b9a34c81e3752f5ec5f119d5df722445397a23afb2bfc2  ./src/opus_k3_refusal_order_and_allocator_state.rs
7a6e4ecadcf39adcf2c1bc69b80df12b9c53389d608f576f888427e70f5f8a22  ./src/swap.py
f4051ab55cf0913a005cda4cf7602992813684421a78024185f97955a09137c4  ./tools/rerun.sh
```

各文件做什么：
- `src/instrument.py`：改冻结副本的拷贝，加三样东西。一是每次回退打一行 `OPUS-K2`，列出隔离单元清单。二是影子账每算一次打一行 `OPUS-B3`：按槽（今天）与按起点槽（改之前）各算一遍交给分配器的槽，打出两边的差。它只用函数里已经算出的落点，不另读盘。三是把 `src/k1-tests.rs` 的三条直调用例接进 `shadow_ledger_exempts_and_isolates_slot_by_slot_tests` 模块。
- `src/swap.py`：次序互换变异，把 `slots_of_a_placement_to_isolate_now` 改成先判重、后看豁免。
- `src/opus_k2_*.rs`、`src/opus_k3_*.rs`：经公开入口走 `roll_back_by_a_forward_publish` 的两份 harness 原型。
- `src/anchor_counts.py`：只读，数冻结副本 `crates/mutations.tsv` 里点名的那几行原文在源码里命中几次。
- `out/tally.txt`：每段一行计数。`out/opus-lines.txt`：三条直调用例与两份 harness 原型打出的结论行。

## 各格判定一览

| 格 | 子问 | 判定 | 量过 / 推的 |
|---|---|---|---|
| K1 | 「豁免先于判重」这个次序换了会不会改结局 | **打中（用例缺口）**：换了就会漏隔离一槽；六条 B3 用例、core 全库 255 条、46 个 harness 目标里的 389 条一条都不红 | 量过 |
| K1 | 经公开入口能不能走到 B3 那几形 | 46 个 harness 目标与随机历史四段里共算了 61,973 次影子账（`out/tally.txt` 里 main 开头四行的 b3_calls 之和），按槽与按起点槽交给分配器的槽一次都没分开。今天没有一条经公开入口的用例罩着这次改动 | 量过（没造出经公开入口的形，不支撑「走不到」） |
| K1 | 判重跨强弱两种隔离调法 | 隔离位随读出次序变；回收时按根环表补隔离，结局兜得住 → 没打中 | 隔离位量过；兜住是推的 |
| K1 | `slots_covered_by_a_placement` 的 `expect` 前提在五处来路上成不成立 | 成立 → 没打中 | 推的（逐处读码） |
| K1 | 六条用例的三形与上一轮攻方三形是不是同一件事 | 是 | 推的（读码对照） |
| K2 | 有没有多验（不是隔离来的单元被当成隔离单元） | 没打中：23,670 次回退里隔离单元清单只有 2 次非空，都出自实现员那份 C572 用例（有隔离）；没有隔离的历史一次都没有 | 量过 |
| K2 | 有没有漏验；回退自己释放时造出的隔离认不认 | 没打中：回退那次发布释放时造出的隔离，再回退回去照样认、照样逐盘验，坏盘 0、坏盘 1 都拒，写回原样之后照做 | 量过 |
| K2 | 两条位置项不同槽 | 合法历史里造不出；坏镜像上会按「任一槽在 cur 里」放过，条款没管这一格 → 没打中 | 推的 |
| K3 | 拒之前有没有写盘或动分配器 | 没打中：拒之后分配器整份 Debug 串、现行版本、盘与录制流都不变 | 量过 |
| K3 | 两种拒因同时成立时报哪个 | 报复活集那一条（`ResurrectedUnitCopyUnreadableOrMismatched`）；条款没定次序 → 没打中 | 量过 |
| K3 | `first_copy_that_does_not_read_back` 抽出来之后复活集那一判变没变 | 读法、次序、拒因不变 → 没打中 | 推的（对 diff） |
| K4 | harness 补的两臂 | 新拒因在模型里是 `Unexplained`，按「该成却拒了」判分歧；23,670 次回退里只出现在 C572 用例里 → 没打中 | 量过（出现次数）；判法是推的 |
| K4 | 14 行变异的锚点在冻结副本里各命中几次 | 都是 1 次 → 没打中 | 量过 |
| K4 | 换一个取样点还红不红 | 次序互换那个变异全绿（就是 K1 第一行），追加的 6 行罩不到它 | 量过 |

## 取样、分批与估时

- 没调崩溃枚举函数，这一腿全部原型崩溃状态数为 0（三格都不靠崩溃状态判：K1 量的是影子账算出什么，K2、K3 量的是回退那一判）。
- 随机历史先跑一段 100 段、`rollback` 比重、384 槽盘，挂钟 4.9 秒；按这个估，两段各 2000 段每段约 100 秒，没有缩。实测两段 100 秒、113 秒。
- 46 个 harness 目标：插桩那一份 debug 下不带忽略的跑了 144 秒；变异那一份 release 下带忽略的跑了 98 秒（编译另花 77 秒）。
- 没用内存稀疏盘 `SparseBlockDevice`：K1 直调用例沿用实现员模块里的内存稀疏盘 `SparseDeviceInMemory`；harness 原型沿用 `common::build_pool`，也是内存镜像，没在磁盘上建镜像文件。
- 每条编译、测试都经 `scripts/hosts/run-with-memory-cap.sh 16G` 与 `scripts/hosts/capped.sh 31` 跑，同一时刻只起一条 cargo。开跑时 `ps` 看到别的会话在跑 `cargo test --release -p singlefs-checker-tier …`，没看到性能测量，没等锁。

## K1　B3 五处按槽展开

### K1-1　打中：「先看豁免、再判重」这个次序是承重的，却没有用例钉住它（量过）

`slots_of_a_placement_to_isolate_now` 的注释写了这个次序：`crates/singlefs-core/src/mount.rs:1742`「与改按槽之前的判重同一个次序：先看豁免，再判重」。实现在 `crates/singlefs-core/src/mount.rs:1752`「.filter(|covered| !exempt.contains(covered) && isolated.insert(*covered))」。五处共用一个判重集合（`crates/singlefs-core/src/mount.rs:1596`「let mut isolated: BTreeSet<(u32, u64)> = BTreeSet::new();」），各圈的豁免集合却不一样大：被抛弃根那一圈与根槽读坏那一圈豁免「候选根引用的 ∪ 现行账里的」，C 那一圈只豁免现行账里的（`crates/singlefs-core/src/mount.rs:1705`「&referenced_by_the_current_account,」）。要是先判重、后看豁免，前面那一圈豁免掉的槽也会记进判重集合，C 那一圈再见到它就判重跳过。C 那一圈不豁免候选根引用的槽本来就是有意为之，见 `crates/singlefs-core/src/mount.rs:1690`「// C 那一份不豁免候选根引用的槽」。

**输入**（`src/k1-tests.rs`，直调被测函数；盘是 mkfs 写成的两块 4 GiB 内存稀疏盘）：s = 单元区起点。候选根取 mkfs 第 0 代根，它的实例表是 (s, 跨 2)。一条根环表里记着的被抛弃根引用盘 0 上的 (s, 跨 1)；改成根槽这一遍读坏那一圈记着 (s, 跨 1)，是同一形的另一处。水位之上没施加的记录点名盘 0 上的 (s, 跨 1)。现行账是空的。每一步许可它的那一句：
- C 那一圈的落点只豁免现行账：`crates/singlefs-core/src/mount.rs:1690`「// C 那一份不豁免候选根引用的槽」。
- 被抛弃根那一圈豁免候选根引用的槽：条款窄读法 `.claude/kb/decisions/23-journal的角色与格式.md:390`「分配器多查环里每一个可读根的账、只隔离其中只被被抛弃根引用的槽」。

**今天的代码**（`out/opus-lines.txt`，原样）：

```
OPUS-K1-ORDER via=abandoned_ring slot_s isolated_and_free=(true, false)
OPUS-K1-ORDER via=ring_slot_read_bad slot_s isolated_and_free=(true, false)
```

**次序互换变异**（`src/swap.py`，同一份输出文件，原样）：

```
OPUS-K1-ORDER via=abandoned_ring slot_s isolated_and_free=(false, true)
OPUS-K1-ORDER via=ring_slot_read_bad slot_s isolated_and_free=(false, true)
```

意思是：s 只被候选根与水位之上的记录引用，C 那一圈本该把它隔离；互换之后 s 没隔离、发得出去。

**现有用例都抓不到**（`out/tally.txt`，原样）：

```
name=swap-core-lib tests_passed=255 tests_failed=2 b3_calls=11 b3_new_only_nonempty=4 b3_old_only_nonempty=4 b3_with_abandoned=8 b3_strong=3 b3_named=3 b3_ring_bad=2 k2_rollbacks=0 k2_isolated_nonempty=0
name=swap-harness-46-with-ignored tests_passed=389 tests_failed=0 b3_calls=0 b3_new_only_nonempty=0 b3_old_only_nonempty=0 b3_with_abandoned=0 b3_strong=0 b3_named=0 b3_ring_bad=0 k2_rollbacks=0 k2_isolated_nonempty=0
```

core 全库红的那 2 条就是我加的两条（`out/swap-core-lib.log` 里 `FAILED` 的两行）；实现员的六条 B3 用例在变异上全绿。46 个 harness 目标是名字里带 abandon、unapplied、shadow、isolat、floor、newer、witness、unreadable 等的全部目标，另加回退、unlink、随机历史几个目标，清单在 `src/harness-selected.txt`，带忽略的用例一起跑，389 条全绿。追加的 6 行变异里，B3 第三、第四、第五处那三行把豁免与判重一起换回按起点槽；它们改的是「按什么认」，不是「先后」，罩不到这一形。

**经公开入口走不走得到（推的，没造出）**：要一个槽 y 同时满足四条：被一条被抛弃根（或根槽读坏的被抛弃根）引用；被一条候选根引用；水位之上没施加的记录点名它；现行账里没有它。合起来就是：所选那一版之后、没施加的那次发布 R 自己是候选根（`crates/singlefs-core/src/mount.rs:1690` 那一段注释说的就是这种情形），而 R 之后同一实例的根 R2 被后来某个实例写的行判成被抛弃、并且照抄着 R 的单元。我没在入库装置的盘面上造出这段历史，所以它只是变异层面的缺口：今天的代码次序是对的，只是没有会红的检查钉住它。

**四句**（`.claude/singlefs-ai-sop/rules/evidence-discipline.md@判据自己也会写错：打中之后先判是哪一种`）：
1. 分不分辨臂：这一格没有两条候选臂之争。它分开的是「今天的次序」与「互换次序」：今天的不中，互换的中。
2. 被判的系统看不看得到判别它的东西：看得到。豁免集合、判重集合与三圈的落点都在函数内存里，不读盘。
3. 满足的是判据字面的哪一个分句：正文 K1 那一句「豁免先于判重的次序换了会不会改结局」，答案是会改结局；也落在正文「四、分工」攻方立场「变异改坏照绿」那一分句上。条款上对应 `.claude/kb/decisions/23-journal的角色与格式.md:390`「分配器多查环里每一个可读根的账、只隔离其中只被被抛弃根引用的槽」：互换之后 C 那一圈要护的槽没隔离。今天的代码不违反这一句，打中的是用例。
4. 跑前条款给的改法在打中的格上还中不中：正文没给改法。我提的改法见本节末尾那张表。

### K1-2　经公开入口：61,973 次影子账里按槽与按起点槽一次都没分开（量过）

插桩只读函数里已经算出的落点，算出按起点槽（改之前）会交给分配器的槽，与今天按槽交的比，差打进 `new_only` / `old_only`（`src/instrument.py`）。`out/tally.txt` 原样：

```
name=main-harness-46 tests_passed=335 tests_failed=0 b3_calls=838 b3_new_only_nonempty=0 b3_old_only_nonempty=0 b3_with_abandoned=119 b3_strong=9 b3_named=269 b3_ring_bad=4 k2_rollbacks=45 k2_isolated_nonempty=2
name=main-random-fast-and-ignored tests_passed=75 tests_failed=0 b3_calls=2968 b3_new_only_nonempty=0 b3_old_only_nonempty=0 b3_with_abandoned=5 b3_strong=0 b3_named=9 b3_ring_bad=0 k2_rollbacks=776 k2_isolated_nonempty=0
name=main-large-rollback-unit-area-384 tests_passed=1 tests_failed=0 b3_calls=22199 b3_new_only_nonempty=0 b3_old_only_nonempty=0 b3_with_abandoned=0 b3_strong=0 b3_named=0 b3_ring_bad=0 k2_rollbacks=19911 k2_isolated_nonempty=0
name=main-large-reuse-unit-area-240 tests_passed=1 tests_failed=0 b3_calls=35968 b3_new_only_nonempty=0 b3_old_only_nonempty=0 b3_with_abandoned=0 b3_strong=0 b3_named=0 b3_ring_bad=0 k2_rollbacks=2938 k2_isolated_nonempty=0
```

同一套插桩在实现员的直调用例上是分得开的（`k1-main` 那一行 `b3_new_only_nonempty=4 b3_old_only_nonempty=2`），所以 0 不是插桩看不见造成的。读法：经公开入口的 harness 历史里，带被抛弃根的影子账有 124 次，带 C 落点的有 278 次，可从没出现过「同一起点、跨度不同」或「一份落点罩住另一份的后一槽」。随机历史两段大档 4000 段里一条被抛弃根都没有（`b3_with_abandoned=0`）：随机历史不造崩溃恢复抛弃根这一形。这说明正文第二节记的「B3 用例只直调私有函数」不只是用例写法上的事，今天整个 harness 档也没有一条历史走到这次改动改了行为的那几格。它不支撑「公开入口走不到」：上一轮攻方与这一轮都没在盘面上造出这一形。

### K1-3　判重跨强弱两种隔离调法：隔离位随读出次序变，回收时兜住（隔离位量过，兜住是推的）

被抛弃根那一圈按来路分两种调法：根环表里记着它的走弱调法，只隔离分配位没置着的槽，判在 `crates/singlefs-core/src/allocator.rs:606`「if !self.allocated[index] {」；抬 F 那一串读盘得来、表里没记着的走整段隔离（`crates/singlefs-core/src/mount.rs:1660`「allocator.isolate_abandoned(device, slot_to_isolate, SPAN_OF_ONE_SLOT);」，理由见 `crates/singlefs-core/src/mount.rs:1610`「抬 F 那一串读盘得来、表里没记着的，回收时没有一份补得上，照旧整段隔离。」）。判重集合不分调法：先到的弱调法把槽记进判重集合，哪怕分配位置着、一槽都没隔离；后到的强调法再见到它就跳过。

输入（`src/k1-tests.rs` 第三条，只打印不判）：A1 是表里记着的被抛弃根，引用盘 0 上的 (s, 跨 2)；A2 是表里没记着、从盘上读账的被抛弃根，取 mkfs 那一版；盘 0 上 s、s+1 的分配位置着（模拟现行账里已释放、还在 defer 窗口里的槽）。`out/opus-lines.txt` 原样：

```
OPUS-K1-STRENGTH order=disk_read_root_earlier dev0_s_isolated=true dev0_s1_isolated=true dev1_s_isolated=true
OPUS-K1-STRENGTH order=recorded_root_earlier dev0_s_isolated=false dev0_s1_isolated=false dev1_s_isolated=true
```

隔离位随根环读出次序变，正是上一轮形一那种「结局随读出次序变」，只是换到了强弱两种调法之间。改之前按起点槽判重时同一组落点结局也一样（插桩那一行 `calls_new=6 calls_old=6 new_only=[] old_only=[]`），所以等起点的这一形是旧的，不是这次改动带进来的。跨度不同、起点也不同而互相罩住的那一形（A1 (s+1, 跨 2) 记着、A2 (s, 跨 2) 与 (s+2, 跨 1) 读盘），改之前两边起点不同、不判重，强调法照样隔离；按槽判重之后 s+1、s+2 被 A1 先记走。这一形是这次改动带进来的，推的、没跑。

为什么判「没打中」（推的）：A1 在表里，回收 s 时按表补隔离（`crates/singlefs-core/src/allocator.rs:1307`「let mut protected = self.placements_referenced_by_abandoned_roots();」）。A1 离开根环时，清隔离位只看表（`crates/singlefs-core/src/allocator.rs:2501`「fn clear_isolation_nothing_still_protected_backs(」），A2 不在表里，所以即使当初强调法置了位，这时也照样被清掉。两种次序最终可观测的结局一样，判重丢掉的那次强隔离没有可观测的后果。

**顺带的线索（不在这一轮的格里，推的，没量）**：上一段推理的最后一句本身是个旧洞。抬 F 那一串读盘得来、表里没记着的被抛弃根 A2，算完影子账之后它的落点没有记进根环表：抬 F 只取了计数，见 `crates/singlefs-core/src/mount.rs:4574`「Ok(computed) => computed.abandoned_roots_unreadable,」。之后共用同一槽、表里记着的 A1 离开根环时，那一槽的隔离位被清掉，而 A2 还在环里。这碰的是 `.claude/kb/decisions/23-journal的角色与格式.md:390` 那一句「它们引用的单元不许重新分配也不许抹头」。造法与频率没量，交主 agent 判要不要另立一题。

### K1-4　`slots_covered_by_a_placement` 的 `expect` 前提（推的，没打中）

`crates/singlefs-core/src/mount.rs:1735`「let end_slot = slot.0.checked_add(span_slots).expect(」依赖进影子账的落点都判过落在单元区里。五处的来路我逐个读了：
- 现行账：可写挂载那一遍来自盘上读的账，过 `crates/singlefs-core/src/recovery.rs:1942`「pub(crate) fn allocation_records_fit_the_pool_geometry(」；抬 F 那一遍是内存里的现行账，记录是分配器自己发的。
- 候选根与读盘的被抛弃根：`placements_referenced_by_root` 两臂都过单元区判定（`crates/singlefs-core/src/mount.rs:2555` 那一处，与分配记录树那一臂的同一个判定）。
- 映射现算那一支：`crates/singlefs-core/src/mount.rs:2481` 那一处逐条判。
- 记着的落点：根环表与根槽读坏那一份都是挂载时从上面两种来路记下的。
- C 那一份：记录点名的那一部分过 `crates/singlefs-core/src/mount.rs:1908` 与 `crates/singlefs-core/src/mount.rs:1915`「.is_ok()」；账、较新可读根、回退复活那几份走 `placements_referenced_by_root` 或现行账。

槽号上界：`crates/singlefs-core/src/recovery.rs:2003`「// 槽号是盘上的 6 字节（< 2⁴⁸）、跨度 ≤ 2¹⁶ 由调用方的字段宽给出，两个加法都装得进 u64。」所以 `checked_add` 溢出走不到。改之前落点按跨度整段交给分配器，现在按槽展开进 `BTreeSet`，集合大小随跨度涨；跨度上界由同一个判定挡着，不会炸。没造出反例。

### K1-5　六条用例的三形与上一轮攻方三形（推的）

逐条对 `research/prompts/m3-c554-c572-r1-opus-output.md` 第 150–181 行：形一（两条被抛弃根同起点、跨 1 与跨 2）对应用例 `two_abandoned_roots_sharing_a_start_slot_isolate_both_slots_of_the_wider_placement_in_either_read_order`，形二对应 `…leaves_the_second_slot_of_an_abandoned_roots_wider_placement_isolated`，形三对应 `…leaves_the_second_slot_of_a_placement_named_by_unapplied_records_isolated`，落点、跨度与分配器状态都相同；另两条罩的是两份豁免集合。是同一件事。缺的是 K1-1 那一格：三圈之间豁免集合大小不同时的先后。

### K1 的改法（只在我的模型上量过、被攻过零轮）

| 改法 | 修哪一格 | 要动的地方 | 标 |
|---|---|---|---|
| 把 `src/k1-tests.rs` 前两条（`opus_k1_exempt_before_dedup_…`）收进 `shadow_ledger_exempts_and_isolates_slot_by_slot_tests`，另在 `crates/mutations.tsv` 加一行次序互换变异（原文与替换文同 `src/swap.py`） | K1-1 | `crates/singlefs-core/src/mount.rs` 的测试模块、`crates/mutations.tsv` | 量过：今天两条绿，变异上两条红（`out/opus-lines.txt`、`out/swap-core-lib.log`） |
| 判重集合只记真隔离了的槽，或按调法分开记 | K1-3 那一形（今天兜得住） | `slots_of_a_placement_to_isolate_now` 与它的三处调用 | 推的；照 K1-3 的推理，不改也兜得住，不建议为它动代码 |
| 抬 F 那一串把读盘得来的被抛弃根的落点也记进根环表，或清隔离位时另看这一遍算出的被抛弃根 | K1-3 末尾那条线索 | `crates/singlefs-core/src/mount.rs` 抬 F 那一串、`crates/singlefs-core/src/allocator.rs` 的清隔离位 | 推的；没造盘面、没量 |

## K2　C572 隔离单元怎么认

**判定：没打中（量过）。**

认法：`crates/singlefs-core/src/mount.rs:8211`「current.units.iter().map(|unit| unit.slot).collect();」与 `crates/singlefs-core/src/mount.rs:7818`「.any(|location| slots_of_the_units_of_the_current_version.contains(&location.slot));」。

**多验（量过）**：每次回退把隔离单元清单打成 `OPUS-K2`。`out/tally.txt` 里 main 开头四行的 `k2_rollbacks` 合计 23,670 次回退（45 + 776 + 19,911 + 2,938），`k2_isolated_nonempty` 合计 2 次。这 2 次都出自实现员那份 `rollback_by_a_forward_publish_verifies_every_copy_of_an_isolated_unit_the_target_references.rs`：日志里它那一段的 `Running` 行之后紧跟着这两行 `target=(1, 3) … isolated_units=[("Data(DataUnitIndexInFile(0))", 50180, 50180)]`，那份用例本来就造了隔离。其余历史里没有隔离，清单也从没非空。随机历史含覆盖写、回退、抬 F、卸载重挂（重挂之后 `current` 是从盘上重建的那一版，K2 问的「units 漏了 cur 还引用的单元」最可能出在这里）；`unlink_in_histories_judged_by_the_model` 的 25 条含 `deleted_inodes` 队列没走完的版本。都没多验。
读码佐证：从盘上重建的版本里 `units` 收了数据单元、extent 树节点、inode 叶容器与根、分配记录树、删除那几棵树、记账树、映射树、树表、实例表，见 `crates/singlefs-core/src/recovery.rs` 里 `units.push(unit(` 那几处；unlink 每一批当场释放尾部单元（`crates/singlefs-core/src/unlink.rs` 文件头），所以队列里的 inode 还剩的单元仍在 extent 树里，也就仍在 `units` 里。

**漏验与另一个来路的隔离（量过）**：隔离除了覆盖写释放时造出来，还有一个来路是回退那次发布自己释放 cur 的用户可见单元时核出对不上（`roll_back_by_a_forward_publish` 释放集那一段）。原型 `src/opus_k2_isolation_made_by_a_rollback_release.rs` 的历史：A（txg 3）→ 覆盖写 B（txg 4）→ 把 B 的数据单元在盘 d 上那一份改坏 → 回退到 A（txg 5，释放 B 的数据单元时隔离）→ 回退到 B。`out/opus-lines.txt` 原样（d = 1 那一组；d = 0 那一组同形，拒在盘 0）：

```
OPUS-K2-ROLLBACK-RELEASE bad_device=1 quarantined_copies=2 released=4
OPUS-K2-ROLLBACK-RELEASE record device=1 is_released=false generation=4
OPUS-K2-BACK-TO-B bad_device=1 slot_of_b=50182 outcome=Err("IsolatedUnitCopyUnreadableOrMismatched { unit: Data(DataUnitIndexInFile(0)), device: DeviceIdentity(1), slot: SlotNumber(50182) }") disk_unchanged=true records_unchanged=true
OPUS-K2-BACK-TO-B-REPAIRED bad_device=1 outcome=Ok(RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(4) }) cold_reads_b=true
```

这一来路的隔离也是条款说的那种：`.claude/kb/decisions/19-块指针的结构与宽度预算.md:107`「核出对不上时隔离那个槽，发布照成：逻辑上照样释放（映射条目去掉），物理槽不还回空闲池，记进隔离并计数报出」。新那一步认得出、验得了，写回原样之后照做、冷启动读回 B。

**漏验为什么造不出（推的）**：隔离单元在现行账里每块盘上都仍分配，它的槽发不出去，cur 不可能在同一槽上另有一个单元；cur 要是还引用它，它就不是隔离来的，而是两版共用的，条款明说那种不验（`.claude/kb/decisions/23-journal的角色与格式.md:388`「其余 cur 的账里仍分配的单元分配器不会发出去，不在逐盘验之内」）。认法只比槽号、不比盘：第一版每个单元两块盘同槽，所以只比槽号够用。

**两条位置项不同槽（推的，没打中）**：回退目标那一版的映射条目只经 `refuse_locations_that_do_not_name_two_pool_devices` 判盘，不判同槽；释放判定路径那一侧另判同槽，回退目标这一侧没判。坏镜像上，一条同槽检查通过、两项不同槽的映射条目，只要它的某一项槽号恰好是 cur 的一个单元的槽，就会被当成共用单元，不进逐盘验。合法历史造不出这种条目。条款（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:113`「位置项指向一块不在池里的盘，或两条位置项指同一块盘，当映射条目损坏：在任何写之前拒绝，盘上不变」）没列「不同槽」，不算打中。

## K3　C572 拒在什么时候

**判定：没打中（量过）。**

原型 `src/opus_k3_refusal_order_and_allocator_state.rs`：K2 那段历史，回退到 B 之前另可选把 B 的 inode 根在盘 0 上那一份改坏（它在回退到 A 时正常释放，回退到 B 时进复活集）。拒之前与拒之后，比较分配器整份 Debug 串、现行版本、盘与录制流。实现员的用例只比 `records()`，见 `crates/singlefs-harness/tests/rollback_by_a_forward_publish_verifies_every_copy_of_an_isolated_unit_the_target_references.rs:179`「pool.allocator.records(),」。`out/opus-lines.txt` 原样（行尾截掉的字段见文件）：

```
OPUS-K3 also_corrupt_a_resurrected_unit=false data_slot_of_b=50182 inode_root_slot_of_b=50256 outcome=Err("IsolatedUnitCopyUnreadableOrMismatched { unit: Data(DataUnitIndexInFile(0)), device: DeviceIdentity(1), slot: SlotNumber(50182) }") disk_unchanged=true allocator_debug_unchanged=true output_unchanged=true
OPUS-K3 also_corrupt_a_resurrected_unit=true data_slot_of_b=50182 inode_root_slot_of_b=50256 outcome=Err("ResurrectedUnitCopyUnreadableOrMismatched { unit: InodeRoot, device: DeviceIdentity(0), slot: SlotNumber(50256) }") disk_unchanged=true allocator_debug_unchanged=true output_unchanged=true
```

- 拒之前什么都没动，连隔离位、空闲图、回收集一起比都相同。读码：动分配器的第一步在 `crates/singlefs-core/src/mount.rs:8290`「let allocator_before_the_rollback = allocator.clone();」之后，前面对分配器与盘都只有不可变借用。
- 两种拒因同时成立时，报复活集那一条（复活集的逐盘验排在前面，`crates/singlefs-core/src/mount.rs:8223`「every_copy_of_the_resurrected_units_reads_back(&*devices, &units_to_check_on_every_device)?;」）。条款那一格把几种拒因并列，没定次序（`.claude/kb/decisions/23-journal的角色与格式.md:388`「同样逐盘验，有一份读不出或对不上就拒（用户定案，原话在变更史）」），所以不算打中。
- 抽出 `first_copy_that_does_not_read_back` 之后（对 diff，推的）：读的字节数、单元次序、位置项次序、遇到第一份坏的就停，与改之前逐字等价；拒因成员与字段不变。

## K4　harness 补的两臂与用例、变异

**判定：两臂与锚点没打中（量过）；变异罩不到次序那一格（量过，就是 K1-1）。**

- **两臂**：`crates/singlefs-harness/src/model_comparison.rs:797`「| RollbackError::IsolatedUnitCopyUnreadableOrMismatched { .. }」把新拒因映射成 `Unexplained`。模型那一侧没有对应理由，按「该成却拒了」判分歧：`crates/singlefs-harness/src/model.rs:2903`「ObservedRefusalReason::Unexplained => None,」之后落到 `crates/singlefs-harness/src/model.rs:2907`「ModelDisagreementAspect::RefusedWhenModelRequiresSuccess」。所以只要随机历史或故障注入里出现「隔离单元 + 一份坏副本 + 回退回去」，这一步就会被记成新分歧，与复活集那条拒因今天的待遇相同。实测出现次数：K2 那 23,670 次回退里，隔离单元清单非空只出现在实现员那份用例里；随机历史不造坏副本，故障注入快档（`fault_injection_fast_tier`）不走回退。所以今天不会误报，以后加上「改坏一份副本」这类注入时会报成新分歧（推的）。
- **新用例造的隔离**：覆盖写释放之前读盘核出盘 1 那一份对不上，正是 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:107`「核出对不上时隔离那个槽，发布照成：逻辑上照样释放（映射条目去掉），物理槽不还回空闲池，记进隔离并计数报出」那一种。核「盘上不动」用的是 `disk_snapshot`（系统配置槽、读得出的根、录制流步数）加分配器的 `records()`；我在 K3 原型里补比了分配器整份 Debug 串，也不变。
- **锚点**：`src/anchor_counts.py` 只读冻结副本，追加 6 行与题面点名的整行替换 8 行，原文都恰好命中 1 次、替换文都与原文不同（`out/anchor-counts.txt`，14 行都是 `hits=1 replacement_differs=True`）。
- **换一个取样点**：追加 6 行里 B3 那 5 行各有一条按名字点到的用例，实现员报告说 `prove-red` 都抓到，我没重跑。它们罩的是「按什么认」；「先后」那一格靠它们罩不到，见 K1-1 的变异输出。

## 没打中的形状

- K1：等起点、强弱两种调法之间的判重（K1-3），隔离位随次序变，最终结局由回收时按表补隔离兜住。`expect` 的五处来路（K1-4）。跨度上界撑大 `BTreeSet`（K1-4）。
- K1 经公开入口：46 个 harness 目标（不带忽略 335 条）、随机历史快档与带忽略的用例 75 条，加两段大档各 2000 段（`rollback` 比重 384 槽盘、`reuse` 比重 240 槽盘，种子从 7100000000 起）。合计 61,973 次影子账，按槽与按起点槽交的槽 0 次不同。
- K2：同一批取样共 23,670 次回退，没有隔离的历史里隔离单元清单 0 次非空；回退自己释放造出的隔离，坏盘 0、坏盘 1 两形都被认出、都拒，写回原样之后照做。
- K3：拒之前分配器整份、现行版本、盘与录制流都不变（两形）；两种拒因同时成立时报复活集那一条。
- K4：两臂在上述取样里不触发；14 行锚点各命中 1 次。
- 没试的：把回退目标的映射条目造成两项不同槽的坏镜像（K2 那一条只读了码）；崩溃枚举（这一轮三格都不靠它）；入库装置上的复跑。

## 这条腿自己的限度

- 只抽了一次样，各条「没打中」照 `.claude/rules/three-way-inference.md@一条腿只抽一次样不算一次观测——否定结论尤其不算` 不算数。
- K1-1 只在变异层面打中，今天的代码次序是对的；经公开入口能造出那一形是推的，没造。
- K1-2 的「0 次不同」只说明这些历史走不到，不说明公开入口走不到。随机历史本来就不造被抛弃根。
- 插桩在被测函数里多了两份集合和一行打印，没另读盘；打印在多线程测试里会与测试名混在同一行，计数按子串数，不受影响。
- harness 原型沿用 `common::build_pool`（4 GiB 内存镜像），只取了一个几何。

## 没做什么

- 没跑任何门禁阶段，没跑 checker 档测试，没起 GPU，没跑崩溃枚举。
- 没在入库装置上重做。这里的数都是冻结副本拷贝上的，照 `.claude/rules/three-way-inference.md@判决由主 agent 做，不由投票做` 不进 kb。
- 没整条跑 `tools/rerun.sh`：它约 10 分钟，超过子 agent 前台 4 分钟的上限。我把它的每一步分开跑过，`out/` 里的数出自那几次分段跑，计数行用的是 rerun.sh 里同一份 `tally` 函数。
- 没重跑实现员那 6 行变异的 `prove-red`。
- 不判正推那一格，也不替主 agent 采纳。

## 要主 agent 跑的命令

1. 复跑模型（整条约 10 分钟，后台跑）：工作目录是仓根，环境变量不用设（`OPUS_THREADS` 不设时取 31），内存上限 16G。

   ```
   bash scripts/hosts/run-with-memory-cap.sh 16G bash research/prompts/code-backlog-b2-code-r1-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo --work <新的空草稿目录>
   ```

   看最后一行 `name=rerun_compare tally_same=yes lines_same=yes`。不是 yes 的，比 `<work>/tally.txt` 与 `out/tally.txt`。

## 草稿目录

`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1-opus/`：交回前删了冻结副本的两份拷贝（`repo/`、`repo-swap/`）与两个编译目录（`target-main/`、`target-swap/`）。留着的是日志（`out/`，含两段大档的整份日志，各约 4 MB）、改代码的脚本（`tools/`）、计数行（`tally-work/`），以及认领文件。它们没进仓的原因：大日志只留了计数行（`out/tally.txt`），其余都已拷进模型目录。

## 引文核对

`python3 scripts/agents/cite-check.py research/prompts/code-backlog-b2-code-r1-opus-output.md --root /home/fy5090/code/singlefs` 末行（追加这一节之前跑的）：

```
  ✓ 核了 31 处引文、5 处裸行号，对上 31 处，没判 5 处（其中裸行号没有引文可比 5 处；逐处列在上面）
```
