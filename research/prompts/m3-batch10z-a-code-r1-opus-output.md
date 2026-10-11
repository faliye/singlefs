# 根环实现第 10z 批第一块（显存账）代码轮第一轮：攻方腿报告（2026-10-08）

<!-- doc-lint:not-numbers K1 K2 K3 G1 P1 P2 -->

轮名 m3-batch10z-a-code-r1，攻方（Opus）。被判代码引冻结副本 `~/.local/share/singlefs/work/m3-batch10z-a-code-r1/tree/` 下的路径；
`crates/singlefs-harness/src/gpu_memory_ledger.rs`、`crates/singlefs-checker-tier/src/gpu_unit_checks.rs`、`crates/singlefs-checker-tier/src/crash_amplification.rs`、
`crates/singlefs-checker-tier/src/crash_judge_dispatch.rs` 四份与仓里此刻逐字节相同（`cmp` 过），下文引文写仓内路径，行号两边一样。
文中所有数都是攻方模型在草稿副本上量的（副本上的数，不进 kb）；「推的」是读码推的、没跑。

## 复跑

模型目录 `research/prompts/m3-batch10z-a-code-r1-opus-model/`。仓根执行（副本先 `rsync -a --exclude target --exclude .git` 从冻结副本拷）：

```
bash scripts/hosts/run-with-memory-cap.sh 12G bash scripts/hosts/capped.sh 15 bash research/prompts/m3-batch10z-a-code-r1-opus-model/run.sh <副本> <编译目录>
bash scripts/hosts/run-with-memory-cap.sh 12G bash scripts/hosts/capped.sh 15 bash research/prompts/m3-batch10z-a-code-r1-opus-model/pre10z.sh <空目录> <编译目录>
bash scripts/hosts/run-with-memory-cap.sh 12G bash scripts/hosts/capped.sh 15 python3 -B research/prompts/m3-batch10z-a-code-r1-opus-model/mutate_k3.py <副本> <编译目录> <变异名…>
ATTACK_MODEL=1 （同上一条，副本的 crates/singlefs-harness/tests/ 先放进两份模型）
```

```
573272994fd3f541153e19b5236e9306a5b7d4df944bced0a13fc6f9dfb1840c  ledger_attack_10z.rs
242a7563483eeda28894bc985c58872ff1dd35a1f5ba6c07c86b3a8e61a7701d  p2_blocks_10z.rs
29c1af2fb58dd90d6f581edfc1c2bfce8a092a18cea7df61c57757c34fc50cc3  run.sh
52cea529d0d165e2e4644a3d9210deb9c55b5972d60809971459e5ec7e0643d2  pre10z.sh
91ed1dd2868f172f4339450b2e5b627b11fd769683545e2f2824edb7d7f98d67  mutate_k3.py
e34934da65726d3e5f5811025dc1ffae4a0d6927c8948c46ce87505a41b48cfb  output-10z.txt
f7867f6e572c0a4defd8adb6db00364b4ddf6d9a0edaa0591ab95322393730f1  output-pre10z-copy.txt
b8eb645a550d778d5524fd4a7e3793df0097776530824de5a43fe9722e54d5e2  output-mutants-lib.txt
6074171363a7fcc684d8b983e0afd0bce22123f90ae3109692d82c4672212c7c  output-mutants-model.txt
```

模型怎么搭：`ledger_attack_10z.rs` 原样调冻结副本的 `GpuMemoryLedger`（一行不改），读数器接一张假 5090（空卡 32 110、D = 2 560、驱动实占 600），
卡上这个进程占着的由模型按「每笔借据建了多少、销了账而卡上还没还的、驱动实占、丢了的旧设备」记，每次读数同一刻出空闲与占着的。
判据「真能再借的」= 空闲 − 每笔活着的借据还没建的那一截 − 驱动还要长的那一截（起着是 D − 实占，没起是 D）；账给的比它大是多给，借不到而它放得下是多报本次未跑。
`p2_blocks_10z.rs` 一字不差抄 `BlocksForTheCards`，卡线程那一圈与按块序写库照原码换成按挂钟排的事件。
`pre10z.sh` 是对照臂：把改动全文里账那一段反着打回去，同一份模型跑在 10z 之前的账上（断言照 10z 的数写，在对照臂上红是预期的，只读 ATTACK 行）。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K2-1 第 1 条 乙附 | 多给：没打中；少给：打中（窄） | 另有不走账设备或驱动不按进程报时，建满的借据与驱动那一份都再扣一遍：长借据 12 000 建满后账给 4 950、真能再借 17 550；10z 比 10z 前多扣 2 560（对照臂 7 510） |
| K2-1 第 2 条 P1 的「见到的不变小」 | **打中，多给（量过）** | 活着的借据一趟派完丢了缓冲、另一笔销了账卡上还没还：账给 11 550、真能再借 8 550，照账借满之后卡上超 3 000；随机历史里读到多给最多 2 543。10z 之前同一段历史多给 0（分辨臂） |
| K2-1 第 3 条 重起 +D | 多给：没打中；少给：打中 | 外人占的从 0 扫到 20 000 共 2 001 格，重起放行而放不下 0 格（对照臂 60 格）；重起被拒而放得下 196 格（宽 1 960 MiB）；重起之后旧设备还了，少给 1 960 或 4 600，再读十次不变 |
| K2-1 第 4 条 OOM 归属残差 | 推的，没量 | 编内核那一段（冷编几分钟）里，同一设备上不经那一层的线程报的 Out of Memory 都算这段活的，这段活真正的 panic（内核校验错这类）报成显存不够 |
| K2-1 第 5 条 P2 交回的块 | 打中（窄，量过在模型上） | 别的卡收工之后才交回的块没人判；那一块后面别的卡判完的块因按块序写库也不入库（快慢 3:1 时判完 2 块不入库）；状态全装在第一张卡上时一报就本次未跑，第二张卡闲着 |
| K1-3 / K2-2 P1 修补本身 | **打中，压不够（量过）** | 一笔借据在另一条线程读数那一刻活着而没建满，销账时从「见到的」里减它整笔，减多的那截永远压不掉：三线程交错十轮之后账给 9 550（与第 10y 批 P1 第 10 笔之后同一个数）、真能再借 19 550，要 18 550 借不到 |
| K1-3 按线程认 | 推的，没量 | 一段活在自己范围里接到的 Out of Memory 要等整段活收完才记进设备（第 2167 行）；同一块的另一段若因设备丢了先失败、先查，查不到「这段活开始之后记过」，判红 |
| K1-3 P2 判两遍 | 没打中 | 交回只在 Out of Memory 那一支、那一块没发去写库；模型三组快慢、每组 20 格，判完的块数与写进库的块数都不超过块数，没有一块写两次 |
| K3 改回去照绿 | **打中（量过）** | 账上 10 处改坏 8 处照绿；其中两处（`return_a_lease` 不从见到的里减、`take_in_a_reading` 不换那一刻活着的借据）在模型随机历史里多给最多 5 126、33 次多借 |

收到主 agent 消息（写完第一段之后）：线程上限改成 10。之后没有再起编译或测试；前面的跑都是 `capped.sh 15`。

## 一、K2 主攻：实现员自定的五条取法

### 1. 乙附（分不出时驱动实占按 0 算）

- 多给：没打中。没列这个进程、或另有不走账设备时那一臂恒 0：`crates/singlefs-harness/src/gpu_memory_ledger.rs:199`：「OccupiedByThisProcess::NotListed | OccupiedByThisProcess::AlsoOutsideTheBooks { .. } => 0,」。账给的是空闲 − 借着的 − D，空闲里已经扣掉建上的，只会少。随机历史三档 9 000 段也没读到一次多给（见第 2 条的表）。
- 少给（K2-2）：打中，窄。建满的借据与已经在卡上的驱动那一份各扣两遍。模型「庚」：长借据 12 000 建满，再要 12 000。
  `research/prompts/m3-batch10z-a-code-r1-opus-model/output-10z.txt:42`：「ATTACK scenario=yi-fu-also-outside step=long-12000-built ledger_room=4950 truly_lendable=17550 difference=-12600」；
  对照臂 `research/prompts/m3-batch10z-a-code-r1-opus-model/output-pre10z-copy.txt:35`：「ledger_room=7510 truly_lendable=17550 difference=-10040」。
  借着的那 12 000 两臂都重扣，10z 多扣的是 D（2 560）。
- 总线号只增不减：`crates/singlefs-harness/src/gpu_memory_ledger.rs:1458`：「只增不减」。一个进程只要在一条总线上起过一次不走账的设备（起不来也算），这一趟进程剩下的时间里每笔借都按上面这样扣。
  今天能到的进程（读码）：`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 里带 `#[ignore]` 的那条吞吐用例走 `first_usable_card_or_report_not_run`，同一个二进制里别的用例走卡池，其中五处用 `lease_card_waiting_for_room`，借不到就 panic。所以 `--ignored` 一起跑时，放得下的会判红（推的，没跑）。

### 2. P1 假设「一次读数见到的那一份在那几笔借据销账之前不变小」

打中，多给，量过。这一形是实现员第二节第 2 条自认的残差，这里给出能走到的样子与量。
- 能走到的样子：第 ② 段核对那一笔借据，活着的时候一趟一趟建了又丢：`crates/singlefs-checker-tier/src/crash_amplification.rs:5962`：「借据活到这一段核完（核对的缓冲在下面那一次调用里建、丢）」。
  销了账、卡上还没还的那一份是 10x 判决认过的现象；借据丢掉时只做一次不等待的 poll：`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1211`：「if let Err(error) = self.card.device().poll(wgpu::PollType::Poll) {」。
- 模型「丁」：长借据 10 000、C 8 000、V 3 000 都建满 → 别的线程读一次 → C 销账，卡上还没还 → V 这一趟派完，建上的变成 0 → 再读。
  压的那一行 `crates/singlefs-harness/src/gpu_memory_ledger.rs:217`：「.min(mebibytes.saturating_sub(seen_on_the_card_at_least_mebibytes))」 把 R 从 8 000 压到 5 000。
  `research/prompts/m3-batch10z-a-code-r1-opus-model/output-10z.txt:40`：「ATTACK scenario=P1-assumption-broken step=after-the-pass ledger_room=11550 truly_lendable=8550 difference=3000」。
  照账借满 11 550，V 再长回 3 000、驱动长到 D 之后：`research/prompts/m3-batch10z-a-code-r1-opus-model/output-10z.txt:41`：「ATTACK scenario=P1-assumption-broken step=all-grown on_the_card=35110 empty=32110 over=3000」。
- 对照臂同一段历史：`research/prompts/m3-batch10z-a-code-r1-opus-model/output-pre10z-copy.txt:34`：「ledger_room=8550 truly_lendable=8550 difference=0」。这一形是 10z 引进来的，分辨臂。
- 放开用户动作的随机历史（只固定起点：长借据 10 000 建满；每段 80 步，每步从借、往上建、读、丢、卡上还掉里挑；「缩」那一档还许活着的借据往下缩）：

| 档 | 段数 | 读到多给最大 | 借到而真放不下 | 放得下却借不到 |
|---|---|---|---|---|
| 对照：读与借都只在每笔活着的借据建满时 | 3 000 | 0 | 0 | 0 |
| 只往上建，丢了马上还 | 3 000 | 0 | 0 | 1 841 |
| 只往上建，丢了可能没还 | 3 000 | 0 | 0 | 1 976 |
| 活着的会缩，丢了可能没还 | 3 000 | 2 543 | 0 | 1 833 |

  出处 `research/prompts/m3-batch10z-a-code-r1-opus-model/output-10z.txt:39`：「mode=shrinking-live-leases-returns-left-on-the-card histories=3000 steps_each=80 overgive_at_reading_max=2543 overgive_bookings=0」。
  「借到而真放不下」那一列是 0，因为随机借的量没落进多给的那一截；多给是在读数上量到的，丁那一形的借是有意落进去的。

### 3. 重起时旧设备那一份 D 记进「销了账而卡上可能还没还的」

- 多给：没打中，量过。模型「戊」：a 10 000 建满，设备丢了，旧上下文 600 与 a 的缓冲还在卡上（a 的持有者还拿着旧设备的克隆）；n = 4 000 在丢了的设备上借到；起设备之前重新登记。外人占的从 0 扫到 20 000，步长 10：
  `research/prompts/m3-batch10z-a-code-r1-opus-model/output-10z.txt:5`：「class=restart-allowed-and-fits cells=1300 others_from=0 others_to=12990」，
  `research/prompts/m3-batch10z-a-code-r1-opus-model/output-10z.txt:7`：「class=restart-refused-but-fits cells=196 others_from=13000 others_to=14950」，
  没有一格「重起放行而放不下」。对照臂（10z 之前不重新登记）有 60 格：`research/prompts/m3-batch10z-a-code-r1-opus-model/output-pre10z-copy.txt:4`：「class=restart-allowed-but-does-not-fit cells=60 others_from=14960 others_to=15550」。K1-3 要修的那一形修住了。
- 少给（K2-2）：打中。
  - 被拒的一带宽 1 960 MiB，就是 D − 旧上下文实占：R 记了 D，卡上只有 600。拒的那一句在 `crates/singlefs-harness/src/gpu_memory_ledger.rs:1310`：「if capacity_mebibytes < booked.leases_mebibytes {」。
  - 重起放行之后，旧设备连同 a 一起还给驱动，少给的那一截压不掉：
    `research/prompts/m3-batch10z-a-code-r1-opus-model/output-10z.txt:9`：「step=reading_between=true-ten-more-readings ledger_room=23590 truly_lendable=25550 difference=-1960」；
    n 建满到 a 丢之间没有别的读数时更多：`research/prompts/m3-batch10z-a-code-r1-opus-model/output-10z.txt:11`：「step=reading_between=false-ten-more-readings ledger_room=20950 truly_lendable=25550 difference=-4600」。
  - 少得更多的那一种：重起那一刻 S 归 0、R 加了 D，随后的读数把「实际占着的减 R」记成见到的，旧借据 a 销账时整笔 10 000 从见到的里减，减到 0，连 n 与新驱动那一份都被当成没还的。机理同第二节。
- 只在旧设备晚于重起才还的那一形上量过；旧设备在重起之前已经还了的，R 被压到读数里这个进程占着的，没扫。

### 4. Out of Memory 按线程认的残差（实现员第 4 条）

推的，没量，因为要真卡。
- 编内核那一段在设备上登记着「在给这段活编」；判器三个内核里两个在另起的线程上编：`crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:454`：「.spawn(|| compute_pipeline(card, "crash-judge-recovery", &kernel_sources.recovery));」。
  冷编要几分钟（同文件注释），这几分钟里同一设备上任何不经那一层的线程（卡池里的 GPU 用例线程）报的 Out of Memory 都记在这段活上：`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:114`：「condition.record_an_out_of_memory(format!("没接住的 {error}"));」。
- 这段活之后若真正 panic（WGSL 校验错走同一个没接住的错误回调、或展开预算断言），会被说成显存不够，报本次未跑而不是判红。
  窗口按冷编算是分钟级，按盘上缓存算是秒级；同进程两段活同时编（不同摘要）时，两段互相说成显存不够。

### 5. P2 交回的块（实现员第 5 条）

打中，窄。模型是抄来的那一份，不是原码在跑。
- 卡线程报了 Out of Memory 就交回、退出：`crates/singlefs-checker-tier/src/crash_amplification.rs:2263`：「work.blocks.give_back(pending_index);」。
  别的卡领不到就收工：`crates/singlefs-checker-tier/src/crash_amplification.rs:2251`：「let Some(pending_index) = work.blocks.take_a_block() else {」。
  写库按块序：`crates/singlefs-checker-tier/src/crash_amplification.rs:2003`：「while let Some(block) = waiting.remove(&received.blocks_stored) {」。
- 20 块，卡 0 在它领的第 j 块末尾报，三组快慢各扫 j：每组恰有 1 格本次未跑，都是卡 0 的最后一块。
  卡 0 比卡 1 慢三倍时，那一块之后卡 1 判完的两块也没入库：`research/prompts/m3-batch10z-a-code-r1-opus-model/output-10z.txt:68`：「ATTACK scenario=P2 seconds=[3, 1] oom_at_block_of_card_0=5 stored=17 judged=19 judged_not_stored=2 outcome=not-run」。
  比实现员第 5 条多出来的是「判完不入库」这一截：卡 1 判完的块在 `waiting` 里等那一块，那一块永远不来。结局是本次未跑，不是判错。
- 另一形，读码，并量过模型：只起装了状态的卡：`crates/singlefs-checker-tier/src/crash_amplification.rs:2405`：「(position + 1..work.cards.len()).find(|candidate| work.states_placed[*candidate] > 0);」。
  「前一张放满了才用下一张」时，小流全部装在卡 0 上，卡 1 没借草稿区、不起；卡 0 一报就本次未跑：`research/prompts/m3-batch10z-a-code-r1-opus-model/output-10z.txt:70`：「ATTACK scenario=P2-one-card-placed oom_at_block_of_card_0=5 stored=4 judged=4 outcome=not-run」。
  所以 P2 只在流大到溢出第一张卡时起作用。
- 判两遍：没打中。交回只在那一支，交回的块没发 `Block`；模型三组共 60 格（外加单卡一格），`judged` 都不超过 20，写库没有重的。

## 二、修补本身：K1-3、K2-2

### P1 修补压不够（K2-2 放得下却报本次未跑），量过

机理：读数见到的比记着的多，就把「那一刻活着的借据」整组记下：`crates/singlefs-harness/src/gpu_memory_ledger.rs:872`：「self.leases_seen_on_the_card = self.leases.keys().copied().collect();」。
那一组里任一笔销账时，从见到的里减它借的整笔：`crates/singlefs-harness/src/gpu_memory_ledger.rs:903`：「if self.leases_seen_on_the_card.remove(&lease_identifier) {」，减法在第 906 行。
那一刻没建满（还没建、建到一半、按 MiB 向上取整）的借据，减的比它在见到的里占的多。见到的从此低于卡上真有的，R 就停在多出来那一截上：以后每次读数「见到的 = 实际占着的 − R」，再压也压回原处。
每次借都在锁里现读一次：`crates/singlefs-harness/src/gpu_memory_ledger.rs:1110`：「let rooms = self.rooms_of_the_cards(&mut books, card_positions)?;」。同一个进程里几条用例并发借同一池卡是这个池的用法：`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:516`：「几十条用例共用一张卡时」。所以「别的线程读的那一刻这一笔还没建满」是常态。

| 历史（起点：长借据 10 000 建满，真能再借 19 550） | 十轮（丙为一千轮）之后账给 | 出处 |
|---|---|---|
| 甲：三条线程，A、C 各借 1 000，A 建满时第三条线程读一次，C 那时还没建；A、C 各丢，卡上马上还 | 9 550，要 18 550 借不到 | `research/prompts/m3-batch10z-a-code-r1-opus-model/output-10z.txt:22`：「ATTACK scenario=P1-three-threads step=round=10 ledger_room=9550 truly_lendable=19550 difference=-10000」 |
| 乙：两条线程，A 建到一半时另一条读一次 | 14 550 | `research/prompts/m3-batch10z-a-code-r1-opus-model/output-10z.txt:33`：「ATTACK scenario=P1-half-built step=round=10 ledger_room=14550 truly_lendable=19550 difference=-5000」 |
| 丙：一条线程，借 2 建 1（向上取整），建满后有一次读 | 18 550 | `research/prompts/m3-batch10z-a-code-r1-opus-model/output-10z.txt:34`：「ATTACK scenario=P1-rounded step=rounds=1000 ledger_room=18550 truly_lendable=19550 difference=-1000」 |
| 对照：借据活着时没人读（10z 库单测那一形） | 19 550 | `research/prompts/m3-batch10z-a-code-r1-opus-model/output-10z.txt:35`：「ATTACK scenario=P1-control step=rounds=10 ledger_room=19550 truly_lendable=19550 difference=0」 |

甲第 10 轮之后的 9 550，与第 10y 批判决 K2-2 那一格（正推 P1）第 10 笔之后的数相同：10z 修住的是顺次的那一形，并发的那一形照旧一笔一笔累积。
随机历史里放得下却借不到 1 841 次（第一节第 2 条的表），对照档 0 次，所以这些次数都出在「读的那一刻有借据没建满」上。

另一形「壬」，暂时少给，不累积：a 建满被见到，之后借的 b 建满，a 丢、卡上马上还。压销了账那一份时，「实际占着的减见到的」正好是 b 建上的，b 被当成 a 没还的，直到 b 也丢：
`research/prompts/m3-batch10z-a-code-r1-opus-model/output-10z.txt:2`：「ATTACK scenario=P1-overlap step=book-15550 outcome=Err("NoCardHasRoomNow { need_mebibytes: 15550, card_rooms_mebibytes: [13550] }")」，b 丢之后回到 19 550（同一份第 3 行）。这一形只要一条线程里两笔借据前后重叠就能走到。

对照臂（10z 之前）同几段历史少给更多：甲、乙、对照都是 −8 040，随机档放得下却借不到 3 827 次（`research/prompts/m3-batch10z-a-code-r1-opus-model/output-pre10z-copy.txt:31`：「mode=grow-only-returns-freed histories=3000 steps_each=80 overgive_at_reading_max=0 overgive_bookings=0 false_refusals=3827」）。
10z 把少给减了一半左右，没减到 0；换来的是第一节第 2 条那一形多给。

### K1-3 按线程认之后，真的显存不够被判红（推的）

同一块的几段在几条核对线程上并发。一段在自己的错误范围里接到 Out of Memory，要等整段活收完（`catch_unwind` 返回、范围弹出）才记进设备：
`crates/singlefs-checker-tier/src/crash_amplification.rs:2158`：「let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(work));」，之后 `crates/singlefs-checker-tier/src/crash_amplification.rs:2167`：「if let Some(caught) = &out_of_memory_in_the_scope {」 才记。
若这一次 Out of Memory 让 wgpu 丢了设备（提交这类调用上的，`gpu_memory_ledger.rs` 第 1665 行注释自己这样写），另一段在设备丢了之后先失败、先查。它查不到「这段活开始之后记过」的那一条，丢了那一支要两样都有：`crates/singlefs-harness/src/gpu_memory_ledger.rs:1686`：「(Some(lost), Some(report)) => {」。查不到就走 `resume_unwind`，panic 经 scope 传出去，整趟判红。
两段活谁先查是时序，没上卡量；wgpu 把提交上的 Out of Memory 交给范围还是交给没接住的回调，也没核。没接住的那一路在出错那一刻就记了（`gpu_unit_checks.rs` 第 114 行），所以这一形只在走范围的那一路上成立。

### K1-3 单元校验和那一段的结局

读码：报了的那一块不写、不领下一块：`crates/singlefs-checker-tier/src/crash_amplification.rs:1445`：「if let Some(reason) = reason_the_gpu_memory_was_short_in_one_of_the_pieces(&judged) {」。之前的块已写库。没打中。

## 三、K3：改回去照绿

在草稿副本上逐条改坏 `gpu_memory_ledger.rs` 一处，跑 `cargo test -p singlefs-harness --lib -- gpu_memory_ledger::tests`（54 条），跑完还原，`cmp` 过已还原。
10 处里 8 处照绿（`research/prompts/m3-batch10z-a-code-r1-opus-model/output-mutants-lib.txt` 第 1–10 行）。照绿的 4 处再拿模型随机历史跑（`output-mutants-model.txt`）：

| 改坏 | 库单测 | 模型上的后果 | 判 |
|---|---|---|---|
| 见到的那一组借据销账时不减（第 906 行 `saturating_sub(0)`） | 照绿 | 丢了可能没还那一档，多给最多 5 126，33 次借到而真放不下 | **该补**：`research/prompts/m3-batch10z-a-code-r1-opus-model/output-mutants-model.txt:3`：「overgive_at_reading_max=5126 overgive_bookings=33 false_refusals=206」 |
| 见到的变多时不换那一刻活着的借据（第 872 行） | 照绿 | 同上一行（等价：那一组恒空） | **该补**：同一份第 8 行 |
| 见到的相等也换（第 870 行 `>=`） | 照绿 | 只更保守：放得下却借不到 1 841 → 3 325 | 少给方向，可不补 |
| 重起不清那一组借据（第 884 行） | 照绿 | 随机历史里没有重起，与基线逐数相同；没单造重起一形 | 没判 |
| 重起恰好放得下判成放不下（第 1310 行 `<=`） | 照绿 | 边界差 1 MiB | 少给 1 MiB，可补 |
| 编内核的计数头一处编完就抹（第 1587 行） | 照绿 | 今天的调用方没有同一段活嵌套编（读码） | 今天等价 |
| 线程上的活丢掉时不换回原来那段（第 1549 行） | 照绿 | 今天的调用方没有嵌套的那一层（读码） | 今天等价 |
| 说明取第一句不取最后一句（去掉 `.rev()`） | 照绿 | 只改原因文字 | 不影响结局 |
| 重起不清见到的（第 883 行） | 红 2 条 | — | 拦住 |
| 重起的判拿空卡上限（永不拒） | 红 1 条 | — | 拦住 |

两行该补的变异都是多给方向，第一节第 2 条那一形也是从这一对行出来的。要补的用例：一笔借据被见到之后销账、卡上没还，再读时它不该被当成还活着建着（模型「丁」的前四步就是一条，不要卡）。

checker 档几处只能读码（checker 档测试不跑）：
- 第 1445–1447 行那一支整个删掉：库单测只测了合并函数 `reason_the_gpu_memory_was_short_in_one_of_the_pieces`，接线没测；删了之后，报了的那一块会照不满的判定写进库（推的）。
- 第 2263 行 `give_back` 删掉、第 2217 行建判器报了之后不起下一张：都要卡，没有不要卡的用例。
- 第 2265 行 `return` 改成接着领：会领回自己刚交回的那一块，再报、再交，原地打转（推的）。
- 这几处都能抽成纯函数不要卡地测：卡线程那一圈的「领、判、报了交回」与写库按块序，正是 `p2_blocks_10z.rs` 抄出来跑的那一段。

## 四、打中之后的四句

| 打中 | 分不分辨臂 | 被判的系统那一刻看不看得到 | 满足哪一个分句 | 跑前给的改法在这几格上 |
|---|---|---|---|---|
| P1 修补压不够（甲乙丙壬） | 分辨 10z 与「读数时见到的按借据各自记」这一类；不分辨 10z 与 10z 之前（两臂都少给，10z 之前更多） | **看不到**：读数只给这个进程的合计，分不出那一刻哪笔借据建了多少。属「判别子观测不到」，要换判据 | K2-2「卡放得下却报本次未跑」 | 第 10y 批判决给的改法就是这一批的 P1，在甲乙丙壬上照中 |
| 丁（第 2 条假设被破） | 分辨：10z 多给 3 000，10z 之前 0 | 看不到：同上，账看不到活着的借据在卡上缩了 | K2-1「账给的量大于此刻真能用的」 | 这一形是 P1 改法自己带进来的，改法就是病根 |
| 重起少给 | 分辨：10z 0 格多给 196 格少给，10z 之前 60 格多给 | 部分看得到：旧设备还没还的那一份在读数里，分不出是旧上下文还是旧借据 | K2-2 | 判决给的改法（重起重新登记驱动那一份）修住了多给，带进来这一截少给 |
| 乙附少给 | 不分辨乙附与丙形；分辨 10z 与 10z 之前（多扣 D） | 看不到（驱动不按进程报、或分不出哪个设备占的） | K2-2 | 实现员自认的代价 |
| P2 交回的块 | 不分辨（10z 之前整趟停，更差） | 看得到：写库那条线程知道哪一块交回、哪几块在等 | K2-2，以及题面「判完的块没入库」 | 判决给的改法（只停那一张）在这两格上照中 |
| K3 两行照绿 | — | — | K3「改回去照绿」 | 补用例即可 |

## 五、改法（都只在我的模型上量过或只是推的，被攻过零轮）

| 改法 | 修哪一格 | 标 |
|---|---|---|
| A：借据丢掉时先等设备收拾完（`poll` 改成等待）再销账，销了账的那一份不再进 R，见到的那一套整个拿掉 | P1 压不够、丁、K3 那两行一起没了；代价是丢借据的那条线程要等卡上的活做完 | 推的：没实现，也没量等多久 |
| B：保留见到的，销账时减的量取「它借的」与「上一次读数之后见到的增量」里小的 | 甲（还没建的那笔被算进去时增量为 0）；乙、丙只修一部分 | 推的：没实现；对丁不起作用（丁是缩，不是销账） |
| C：活着的借据会缩的那一路（第 ② 段核对那一笔）改成整段活只借、只建一次缓冲，不一趟一趟建了又丢 | 丁 | 推的；对甲乙丙壬不起作用 |
| D：卡线程报了 Out of Memory 交回之后，若别的卡都已收工、队里还有，就由收尾那一步重起一张卡判剩下的；或者写库那条线程不再等交回的那一块，后面的块先写 | P2 漏判与判完不入库 | 推的 |
| E：重起时 R 加的是旧设备此刻读得出的那一份（读数里这个进程占着的减活着借据的下界），不是整份 D | 重起少给 1 960 的那一截 | 推的；4 600 那一截要连 A 或 B 一起才修得掉 |
| F：补两行变异与一条不要卡的用例（第三节） | K3 | 推的：用例没写，变异在 `mutate_k3.py` 里跑过、照绿是量过的 |

A 与「留着见到的」是一个岔路：前者把 P1 的问题变成等待，后者每一形都要再找下界。主 agent 定之前要有等多久的数，那要上卡。

## 没打中的形状

- 乙附多给：随机历史三档 9 000 段，加上「庚」两种读数（没列、另有不走账设备），没有一次多给。
- 重起多给：2 001 格扫描，0 格。
- P2 判两遍：三组快慢各 20 格，加单卡一格，没有一块写两次。
- 不走账设备「起不来照样记着会不会多借」：只增不减，只会少借，读码。
- 单元校验和那一段：报了的那一块不写、之后的不领、之前的在库里，读码。
- 没试的：真卡上的 Out of Memory 与丢设备（造不出，不在这一轮）、两个进程的竞态（用户已定）、一份报告里约 1.3 毫秒的窗口（用户已定）。
- 取样范围：随机历史每段 80 步、借的量 1..=4 000 MiB、只一张卡、只一个起点（长借据 10 000 建满、驱动实占 600）。驱动实占取 D（2 560）的那一档没扫，10z 之前那一臂在那一档上的数会不同。

## 这条腿自己的限度

- 判据「真能再借的」是我定的：活着的借据都可能长到借的量，驱动会长到 D，销了账没还的不回来。换一个判据，多给、少给的量会变。
- 模型里卡上的占用由我按事件记，没上卡；wgpu 什么时候把丢了的缓冲还给驱动、分配器按块拿多出来的那一截，都没进模型。
- P2 的模型是抄来的那一段换成事件，不是原码在跑；抄的那几行的行号写在文件头。
- 「对照臂」是把改动全文反着打回去得到的 10z 之前那一份账，没与当时入库的文件逐字节核。
- 崩溃状态：这一轮没有崩溃枚举，没调 `enumerate_layer0`，状态数 0，不涉及 10⁶ / 10⁷ 的额度。
- 随机历史只抽了一个种子，三档各 3 000 段；「没打中」那几格照规则要第二次抽样才算数。

## 没做什么

- checker 档测试一条没跑（定义与派发提示都不许）；checker 档几处的 K3 只读码。
- 没上卡，没占卡；nvidia-smi 也没跑（钩子自己跑的资源报告除外）。
- 不判正推那几格，不替主 agent 采纳；副本上的数不算入库装置上的数。
- 第 4 条（编内核窗口）与 K1-3 的判红时序只读码推，没造。

## 草稿目录

`~/.local/share/singlefs/work/m3-batch10z-a-code-r1-opus/`。交回前删了自己建的四份仓副本（`tree/`、`tree-mut/`、`tree-10y/`、`pre10z-copy/`）与三个编译目录（`target/`、`target-mut/`、`target-10y/`）。
留下日志（`*.log`）与 `ledger-10z.patch`。复跑照「复跑」一节重新拷。
