# m3-batch10t-code-r1 攻方腿报告（2026-10-07）

立场：找反例。攻击面：正文第三节 K1–K4 全部格，重点 K1、K2 显存账两头与 K3 第五形五段头部。
代码读冻结副本 `~/.local/share/singlefs/work/m3-batch10t-code-r1/tree/`；下文引的被判文件在主工作区与冻结副本里 sha256 逐份相同（18 份逐份比过），快照 `research/prompts/m3-batch10t-code-r1-snapshot/crates-sha256.txt` 318 行在冻结副本上 `sha256sum -c --quiet` 退 0。
副本上量的数都是副本上的数，不进 kb。

## 复跑

```
bash research/prompts/m3-batch10t-code-r1-opus-model/run.sh ~/.local/share/singlefs/work/m3-batch10t-code-r1/tree <空的草稿目录>
```

它把冻结副本拷进草稿目录，用 `fixed_ledger.patch` 从今天的 `gpu_memory_ledger.rs` 第 1–1489 行做出改法臂的拷贝，拼出扫源码原型，
经 `run-with-memory-cap.sh 8G` 与 `capped.sh 12` 跑两个 harness 档原型测试目标（`opus_ledger_attack_model`、`opus_scan_bypass_model`，名字不带 layer0），
再跑一份纯算术脚本与一份 rustc 编译核对。不上卡，账本在临时目录，不碰真账本。我从空目录用最后这一版模型跑过一遍，36 行 `MODEL` 与第一次逐行相同（`diff` 无输出）。整份模型一次 23–75 秒（看本机负载），没缩取样，也没分批。

| 文件 | sha256 |
|---|---|
| `build_scan_bypass_test.sh` | `1808ae3101e57c80108c3f093ce533e8450fd7cfa168d5f0941c238c89ff4667` |
| `compile_check_raw_c_strings.rs` | `f894e48b78776edfa37fd688c368ab5e3abb9897933180cbefae57e06b0cec25` |
| `fixed_ledger.patch` | `dbac37a24221af4e597a426b65ec21b05bbfafb634d78c3126684061bd4e91c3` |
| `opus_ledger_attack_model.rs` | `1ac701430c834a03758ce0acd275bc539ea6db00d8bf29fb5e1337c97cbb1184` |
| `run.sh` | `a53199def7f00a72bfeee0b3b38ae674fc588c7cfa8538148214ed50c1d6bf9d` |
| `s5_product_path_budget_from_the_opening_free_memory.py` | `31bc1c3b254bcd5948582edad2bd055df625e9705f14bb9995fff2224d77f0be` |
| `scan_bypass_samples.rs` | `e97d7dacffca3b7896a31c9c0110e87ca5d040d1267f49d8bc71b2277c79286d` |

模型的两个改法臂（`fixed_ledger.patch`）：
- 改法甲：每次借、每次看容量，都在那张卡的锁里现读一次空闲，照开账那一套把估计夹进 [现读, 现读 + 活进程已记]；
- 改法乙：按状态数借草稿区时照「容量 − 已记」压状态数，不照整份容量压。
两个改法都只在我的模型上量过、被攻过零轮。

## 各格判定一览

| 格 | 判定 | 依据 | 量过 / 推的 |
|---|---|---|---|
| K1-1 外人在最后一次开账之后来 | **打中** | S1：开账后外人占 20 000，进程照账借满，卡上要的比卡大 20 000（配了整卡额度）/ 11 846（没配，3/4 那一份）；改法甲 0 | 量过（模型） |
| K1-1 零轮设计「偏大不超过两者里小的那个」 | 那一形站住，「只有这一种情形」不成立 | S1b：照文件头那一形，偏大 10 000 = min(10 000, 20 000)；S1 那一形没有第二次开账，偏大的就是外人占的整份 | 量过（模型） |
| K1-1 第 ② 段与单元校验和缓冲、驱动开销 | 没打中 | 第 ② 段单线程一块一块核；第 ① 段每趟结论与回读缓冲只算一份，同时活着两趟时少记一份，几十个字每状态，相对草稿区可略 | 推的（读码） |
| K1-1 没起池的产品路径进程 | 线索，不算新打中 | 文件头写明不进账；对已经开过账的进程它就是 S1 里的外人 | 推的 |
| K1-2 先登记、后建设备三处 | 没打中 | 产品路径取卡、第 ① 段、第 ② 段与单元校验和三处都先在锁里记上才 `started_card` | 读码 |
| K1-3 锁外读估计、锁里借 | 没打中 | 锁外读的只拿来挑卡、算上限，锁里照此刻的账重判 | 读码 |
| K2-1 开账时外人在、后来走了 | **打中** | S2：卡上空 32 607，GPU 用例借 10 000 判红（比每张卡都大），产品路径草稿区被压到 3 079 / 4 983 个状态；改法甲借到 | 量过（模型） |
| K2-1 产品路径按开账那一刻的空闲装状态 | **打中** | S5：登记时克隆出来的卡空闲是开账时读的；那一刻别的记账进程占着大半张卡，之后卡空了，第 ① 段照样判红（开账空闲 2 000，或没配额度时 3 000）或压到 312 / 18 246 | 推的（读码）+ 算术 |
| K2-1 照整份容量压，别的进程借着一点就等满 | **打中** | S3：别的进程借着 1 / 4 000 / 15 000，大流产品路径等满报本次未跑，剩下的放得下 27 422 / 23 423 / 12 423 个状态；改法乙借到 | 量过（模型） |
| K2-1 两个产品路径进程只是都登记了 | **打中** | S6：两个进程先后开池、各登记一份驱动那一份，各自借草稿区都等满报本次未跑；改法乙第一个借到 | 量过（模型） |
| K2-2 取卡登记那一刻满的卡整趟不用 | **打中** | S4：5090 那一刻被借满、随即还了，这一趟只用 5060 Ti，那张上又不够整份，等满报本次未跑；5090 空着 | 量过（模型） |
| K2-2 借草稿区那一刻有 1 MiB 短借据 | **打中** | S4b：5090 不装状态，全部状态排到 5060 Ti；分派一趟只做一次，5090 整趟空着；改法乙两张都装 | 量过（模型）+ 读码 |
| K2-3 产品路径在每张卡上登记、起设备 | **打中**（压并发，不是算错） | 每个跑产品路径的进程在池里每张有空的卡上各占一份驱动那一份，用不用那张卡都占到进程退出；与 S3 合起来就是 S6 | 推的（读码）+ S6 |
| K2-3 僵尸判死、驱动开销只记一份 | 没打中 | 试了线程组长先退、PID 名字空间、同进程重开账三形，见「没打中的形状」 | 读码 |
| K3-1 五段头部三处与实现 | 没打中 | 偏移 0..28、28、29、30..32、32..34 四处逐字节一致 | 读码 |
| K3-2 行不收、链不往下走、I-9.29 门关 | 没打中 | 主走读、只读读法、引用集合、GPU 走读与扫描两处、恢复内核都做到 | 读码 |
| K3-3 I-2.4 不判 | 读法站住，kb 行少一句 | I-2.4 行把实例表链指针列为「只判不断、照样走下去」的入口，第五形那一条现在不走下去 | 读码 |
| K4-1 有卡用例分不分得出借了与没借 | **打中**（判别力缺口） | 只看交回的借的量；借了又在判之前还掉，用例照绿 | 推的（读码，没跑变异） |
| K4-2 扫源码还有哪种写法绕得过 | **打中** | 原始 C 字符串 `cr"\"`、`cr#"x"b"#` 之后的起卡调用看漏；`#[cfg(test)] mod x;` 放到另一份文件的测试整份不扫 | 量过（原型，扫法原样拷） |
| K4-3 钉值、变异行拦不拦得住改回去 | **打中** | GPU 两处第五形判法删掉 extent 偏移那一截没有用例红；S5 那一处、`started_card` 的先登记断言没有变异行 | 推的（读码） |

## K1　显存账会不会让一台机器 OOM

### K1-1 S1：外人在这个进程唯一一次开账之后才来（打中）

被判的读法：估计只在开账时现读一次空闲，「之后每一次借都照账本里此刻的估计算容量」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:40`）；
开账在锁里读空闲的就是这一句「let free_now = read_free_memory_mebibytes(position)?;」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:891`），
借的时候锁里只拿账本里那一份估计算容量，不再读卡（`book_on_the_card_if_it_still_fits`，同文件第 995 行起）。
文件头自己承认的偏大只有一种情形：「推的（没上卡量过）：A 读不到，只知道它在 [0, K] 里。夹出来的估计在一种情形下大于真值：外人在两次开账之间占了显存，」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:42`）。

历史（一个进程、一张 5090、总量 32 607）：① 进程 100 开池，读到空 32 607；② 一个不经账的 GPU 用户占了 20 000（别的会话的实验、或没起池的崩溃放量分项驱动，文件头写明它们不进账：「没起过卡池的进程照旧不进账」（`crates/singlefs-checker-tier/src/crash_amplification.rs:37`））；
③ 进程 100 的下一条 GPU 用例照账借满。这个进程之后没有第二次开账（卡池一个进程只起一次），外人它永远看不见。

命令（草稿目录副本）与原样输出：
```
CARGO_TARGET_DIR=$D/target nice -n 19 bash scripts/hosts/run-with-memory-cap.sh 8G bash scripts/hosts/capped.sh 12 cargo test --offline -p singlefs-harness --test opus_ledger_attack_model -- --nocapture --test-threads 1
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 22.99s
MODEL scenario=s1 configured=Some(32607) arm=today capacity_at_open=30047 outsider=20000 outcome=booked=30047 card_demand=52607 card_total=32607 over_mib=20000
MODEL scenario=s1 configured=None arm=today capacity_at_open=21893 outsider=20000 outcome=booked=21893 card_demand=44453 card_total=32607 over_mib=11846
MODEL scenario=s1 configured=Some(32607) arm=fix_a_re_read_at_every_booking capacity_at_open=30047 outsider=20000 outcome=red_need_larger_than_every_card(capacity=10047);then_capacity=10047;booked=10047 card_demand=32607 card_total=32607 over_mib=0
MODEL scenario=s1 configured=None arm=fix_a_re_read_at_every_booking capacity_at_open=21893 outsider=20000 outcome=red_need_larger_than_every_card(capacity=6893);then_capacity=6893;booked=6893 card_demand=29453 card_total=32607 over_mib=0
```
（`$D` 是草稿目录 `~/.local/share/singlefs/work/m3-batch10t-code-r1-opus`，在它的 `tree/` 里跑。）

四句：
1. 分辨臂：分。今天的臂卡上要的比卡大 20 000 / 11 846，改法甲 0。
2. 系统当时看不看得到：看得到。开账读空闲用的是 `nvidia-smi`（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs` 的 `reported_cards_by_pci_bus`），借的那一刻同样能读；代价是每次借起一次 `nvidia-smi`，没量。
3. 满足的分句：K1-1「有没有一形，账给出的容量大于卡上此刻真能用的？」。不是「外人在两次开账之间来去」那一项：这一形只有一次开账。
4. 跑前条款的改法（10o 判决第三节第 3 条「每次起池都现读空闲」）在这一格上还中：一个进程只起一次池。

改法甲自己的副作用（量过）：外人来了之后，按旧容量算出来的那一笔立刻交「比每张卡都大」，GPU 用例那一路照它 panic（判红），见上面 fix_a 两行的 `red_need_larger_than_every_card`。
今天判红这一支只看估计（「if need_mebibytes > largest_capacity_mebibytes {」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:151`）），估计随外人变小，红就跟着来。
所以改法甲要连带改一句：要的不超过配置给卡的额度（或卡的总量）时，估计小了就等，不判红（推的，没实现）。

推翻条件：在入库装置上，一个进程开池之后、借之前让一个不经账的进程占显存，下一笔借照旧照开账时的估计借到，且实际建缓冲时卡上要的超过总量。若开池之外还有别的地方在借前重读空闲（我没找到），这一格不成立。

### K1-1 S1b：零轮设计那一形（对照，站住）

主 agent 定的 10t 第 4 条：外人在两次开账之间占、同时有借了没建缓冲的借据，偏大不超过两者里小的那个。
历史：进程 100 开账、借 20 000 而只起了设备没建缓冲；外人占 10 000；进程 200 开账、借满它看到的剩余。
```
MODEL scenario=s1b arm=today second_capacity=30047 second_booked_room=7487 outcome=booked=7487 card_demand=42607 card_total=32607 over_mib=10000 documented_bound=min(10000,20000)=10000
```
偏大 10 000，正好等于界。这一形的界站住。推广（推的）：上一次开账已经偏大 X 时，下一次开账夹出来的偏大不超过 min(X + 新外人, K − A)，界会累加，文件头没写。

### K1-1 其余几样（没打中）

- 第 ② 段：`verify_blocks_from_these_facts` 一块接一块在一个线程上核，同时活着的趟数照 `PASSES_ALIVE_AT_ONCE_ON_EACH_THREAD` 记两趟，够。
- 单元校验和：核对线程数 × 2 趟，与 `crc32c_of_ranges` 每次建的五个缓冲对得上。
- 第 ① 段判器：`mebibytes_a_judge_needs` 把每状态的结论与回读算一份（「u64::try_from(SCRATCH_WORDS * 4 + 2 * VERDICT_WORDS * 4).expect("装得进 u64");」（`crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:156`）），
  每趟派活另建结论与暂存缓冲、派完就丢，同时活着两趟时少记一趟的那几十个字每状态；草稿区每状态 1 261 504 字节，少记的可略（推的）。
- 驱动开销 2560 没上卡量，正文列为已知没做，不算。

### K1-2 先登记、后建设备（没打中，读码）

- 产品路径取卡：`register_this_process_on_the_cards_with_room_waiting_only_when_none_has_room` 在锁里记上，之后才 `started_card`（「.map(|(card_position, _)| self.started_card(card_position).cloned())」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1345`））；
- 第 ① 段与第 ② 段、单元校验和：借据记上之后 `card_lent_for_the_booking` 才起设备，`started_card` 里断言已登记。
- 残留（推的，不算打中）：卡起不来时登记那一份驱动开销留在账上直到进程退出，偏保守。

### K1-3 锁外读估计、锁里借（没打中，读码）

锁外读到的容量只用来挑卡（`book_waiting`）和算每张卡最多装几个（`states_each_card_of_the_pool_lends`）；真记账都在 `book_on_the_card_if_it_still_fits` 的锁里重读重判。
锁外读得偏大，锁里判不下就重来；偏小只会少借。没找到锁外读、锁里不重判就记的一处。

## K2　显存账会不会白白压并发

### K2-1 S2：开账那一刻外人在、之后走了（打中，判红）

估计只在开账时夹一次，下界就是那一刻读的空闲；没有记账进程时 K = 0，估计就等于那一刻的空闲。外人那时占着，估计就低，之后不再抬。
GPU 用例借的量大于这个低估计时，`book_waiting` 照锁外读到的容量判成比每张卡都大（`decide_booking_by_priority` 那一处，`crates/singlefs-harness/src/gpu_memory_ledger.rs` 第 1316 行），
卡池照它 panic：「Err(BookingRefusal::NeedLargerThanEveryCard(refusal)) => panic!(」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1133`）。
历史：外人占 25 000 时进程 100 开池；外人走了，卡上空 32 607；进程 100 的一条 GPU 用例借 10 000，产品路径借草稿区。
```
MODEL scenario=s2 configured=Some(24576) arm=today free_now=32607 capacity_now=5047 gpu_test_lease_10000=red_need_larger_than_every_card(capacity=5047) product_path_scratch=scratch_booked_states=[Some(4983)]
MODEL scenario=s2 configured=None arm=today free_now=32607 capacity_now=3143 gpu_test_lease_10000=red_need_larger_than_every_card(capacity=3143) product_path_scratch=scratch_booked_states=[Some(3079)]
MODEL scenario=s2 configured=Some(24576) arm=fix_a_re_read_at_every_booking free_now=20047 capacity_now=22016 gpu_test_lease_10000=booked=10000 product_path_scratch=scratch_booked_states=[Some(10000)]
MODEL scenario=s2 configured=None arm=fix_a_re_read_at_every_booking free_now=20047 capacity_now=21893 gpu_test_lease_10000=booked=10000 product_path_scratch=scratch_booked_states=[Some(10000)]
```
（fix_a 那两行的 `free_now` 是借完、建了缓冲之后读的。）
同一段代码的跨进程变形（推的，没单跑）：进程 200 在外人短暂占卡那一刻开账，把账本里的估计压低；外人随后走了，早已开账的进程 100 下一笔照低估计判，同样当场判红。一个进程开账，别的进程的用例红。

四句：分辨臂（今天红，改法甲借到）；看得到（同 S1，借时读 `nvidia-smi`）；满足 K2-1「卡上放得下、账却让它……判红」；10o 第 3 条「每次起池都现读空闲」只管新开账的进程，这里不中。
这是 10o 判决 K4-3 第二个方向（「起池时卡被占着，卡空了以后反而判红或等满」）的外人那一半：10t 只修了「被先起的记账进程订满」那一半（文件头那一句「卡被先起的进程订满：……后起的进程等着借，不当成卡不够」）。

推翻条件：入库装置上，不经账的进程占着卡时开池、它退出之后借一笔大于开池时估计、小于卡上空闲的，不判红。

### K2-1 S5：产品路径第 ① 段按开账那一刻的空闲装状态（打中，判红 / 压状态数）

起过卡池的产品路径拿到的卡是登记时克隆出来的那张，空闲照开账那一刻读的写死：
「started.memory_quota_mebibytes = card.configured_card_quota_mebibytes;」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1078`）、
「card.free_memory_read_when_the_ledger_opened_mebibytes;」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1080`）。
第 ① 段每张卡装几个状态先照这张卡的额度算（`states_in_flight_of_each_card`：「|| card.memory_budget_mebibytes()」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2083`），额度小于表加驱动那一份就是 0），
再与账能借的取小（「let ledger_capacities = pool.card_capacities_mebibytes_of(&card_indices)?;」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2186`））；
每张都是 0 就不借、直接报错：「let scratch = if capacities.iter().all(|capacity| *capacity == 0) {」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2400`），
报的是「"没有一张卡装得下这条流的表（{} MiB）与一个状态的草稿区：看配置里每张卡的显存额度与卡上此刻的空闲显存",」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2434`）。
额度照 `memory_budget_mebibytes_of`：「Some(quota) => quota.min(free_memory_mebibytes),」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:271`）、
没配额度「free_memory_mebibytes / denominator * numerator」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:274`）。

历史：进程开池那一刻，另一个记账进程在 5090 上借着大半张卡（账本估计照夹法留在整卡，账能借的不受影响）；之后那个进程还了，卡空着；这个进程跑产品路径。
命令与原样输出（纯算术，公式逐条照上面几行抄，草稿区每状态 315 376 × 4 字节、表 64 MiB）：
```
nice -n 19 python3 research/prompts/m3-batch10t-code-r1-opus-model/s5_product_path_budget_from_the_opening_free_memory.py
MODEL scenario=s5 configured=24576 free_at_open=2000 card_free_now=32607 ledger_capacity=22016 states_by_the_opening_free=0 states_the_ledger_allows=18246 outcome=red_no_card_holds_the_tables_and_one_state
MODEL scenario=s5 configured=24576 free_at_open=3000 card_free_now=32607 ledger_capacity=22016 states_by_the_opening_free=312 states_the_ledger_allows=18246 outcome=states_per_card=312
MODEL scenario=s5 configured=24576 free_at_open=10000 card_free_now=32607 ledger_capacity=22016 states_by_the_opening_free=6131 states_the_ledger_allows=18246 outcome=states_per_card=6131
MODEL scenario=s5 configured=24576 free_at_open=20000 card_free_now=32607 ledger_capacity=22016 states_by_the_opening_free=14443 states_the_ledger_allows=18246 outcome=states_per_card=14443
MODEL scenario=s5 configured=24576 free_at_open=32607 card_free_now=32607 ledger_capacity=22016 states_by_the_opening_free=18246 states_the_ledger_allows=18246 outcome=states_per_card=18246
MODEL scenario=s5 configured=None free_at_open=2000 card_free_now=32607 ledger_capacity=21893 states_by_the_opening_free=0 states_the_ledger_allows=18144 outcome=red_no_card_holds_the_tables_and_one_state
MODEL scenario=s5 configured=None free_at_open=3000 card_free_now=32607 ledger_capacity=21893 states_by_the_opening_free=0 states_the_ledger_allows=18144 outcome=red_no_card_holds_the_tables_and_one_state
MODEL scenario=s5 configured=None free_at_open=10000 card_free_now=32607 ledger_capacity=21893 states_by_the_opening_free=4052 states_the_ledger_allows=18144 outcome=states_per_card=4052
MODEL scenario=s5 configured=None free_at_open=20000 card_free_now=32607 ledger_capacity=21893 states_by_the_opening_free=10287 states_the_ledger_allows=18144 outcome=states_per_card=10287
MODEL scenario=s5 configured=None free_at_open=32607 card_free_now=32607 ledger_capacity=21893 states_by_the_opening_free=18144 states_the_ledger_allows=18144 outcome=states_per_card=18144
```
四句：分辨臂（改法丙：产品路径装状态只照账的容量，不照克隆卡的空闲；推的，没实现）；系统看得到（账本里的估计就在手边，S5 的历史里它一直是整卡）；
满足 K2-1「判红」与 K2 题面「不压并发」；10o 第 3 条「后起的进程不再拿实时空闲滤卡，改按账判」10t 在卡池那一层做到了（「/// 起池不看卡上此刻空闲多少（被先起的进程订满的卡照样进池），能不能借由账判；这个进程第一次在一张卡上借（连它的驱动那一份）」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:981`）），产品路径这一层没跟上，这一格照中。
这一形与流的大小无关：今天池里跑产品路径的几份测试都走得到。没上卡跑，是读码加算术。
推翻条件：入库装置上，让别的记账进程借满卡时起池，还掉之后跑产品路径，第 ① 段不报错、`CRASH_AMPLIFICATION_GPU_PLACEMENT` 那一行的 `states_in_flight_capacity` 等于照账算的数。

### K2-1 S3、S6：按状态数借草稿区时照整份容量压（打中，等满报本次未跑）

锁里压状态数用的是整份容量，不是剩下的：「what.need_within_the_capacity(seen.capacity_mebibytes);」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1011`），
记不记得上照「已记 + 要的 ≤ 容量」：「.position(|(capacity, booked)| booked.saturating_add(need_mebibytes) <= *capacity)」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:159`）。
一条大流（要判的状态多于一张卡装得下的）要的是整份容量，卡上只要有别人记着一点，这一笔就「此刻放不下」，一直等到别人全还。
S3 历史：一张 5090、配整卡额度；进程 100 借着 `held` 不还；进程 200 的大流走产品路径。
```
MODEL scenario=s3 held_by_other=1 arm=today states_each_card_holds=29983 room_mib=27486 states_that_fit_the_room=27422 outcome=not_run_waited_past_the_longest_wait(need=30047)
MODEL scenario=s3 held_by_other=4000 arm=today states_each_card_holds=29983 room_mib=23487 states_that_fit_the_room=23423 outcome=not_run_waited_past_the_longest_wait(need=30047)
MODEL scenario=s3 held_by_other=15000 arm=today states_each_card_holds=29983 room_mib=12487 states_that_fit_the_room=12423 outcome=not_run_waited_past_the_longest_wait(need=30047)
MODEL scenario=s3 held_by_other=1 arm=fix_b_compress_to_room states_each_card_holds=29983 room_mib=27486 states_that_fit_the_room=27422 outcome=booked_states_per_card=[Some(27422)]
MODEL scenario=s3 held_by_other=4000 arm=fix_b_compress_to_room states_each_card_holds=29983 room_mib=23487 states_that_fit_the_room=23423 outcome=booked_states_per_card=[Some(23423)]
MODEL scenario=s3 held_by_other=15000 arm=fix_b_compress_to_room states_each_card_holds=29983 room_mib=12487 states_that_fit_the_room=12423 outcome=booked_states_per_card=[Some(12423)]
```
S6 历史：一张 5090、没配额度；两个进程先后开池，各自取卡登记（只记驱动那一份、起设备），各自借草稿区。一笔借据都没有，只是对方登记着。
```
MODEL scenario=s6 arm=today p100:free_at_open=32607,capacity=21893,holds=21829,not_run_waited_past_the_longest_wait(need=21893) p200:free_at_open=30047,capacity=21893,holds=19909,not_run_waited_past_the_longest_wait(need=19973)
MODEL scenario=s6 arm=fix_b_compress_to_room p100:free_at_open=32607,capacity=21893,holds=21829,booked_states_per_card=[Some(19269)] p200:free_at_open=30047,capacity=21893,holds=19909,not_run_waited_past_the_longest_wait(need=65)
```
模型里等满按比例缩成 300 毫秒（今天 30 分钟）；这几格里账在等的那段时间不变，等多久结局都一样。两个进程真并发时，各自登记的那一份要到进程退出才去掉，两边都等满、都报本次未跑，卡上空着二万多。改法乙下第一个借到，第二个等第一个判完，这是真的卡满了。

四句：分辨臂（今天等满，改法乙借到）；看得到（锁里就有「已记」）；满足 K2-1「卡上放得下、账却让它等满 30 分钟报本次未跑」；
跑前的改法 10w「估计变小时在账本锁里重压」压的照样是整份容量，这一格照中。
射程（推的）：只有要判的状态多于这张卡装得下的那一趟会撞上；新的有卡用例跑的是「新池新建文件那条流展开的十段共几十个状态，秒级。」（`crates/singlefs-checker-tier/tests/crash_amplification_product_path_books_its_scratch_on_the_card_pool.rs:30`）的小流，撞不上。大流的分项驱动今天不起池、不进账。
改法乙的代价（推的）：后来的那一趟只拿到剩下的，一次派活的状态少，挂钟变长；GPU 用例在产品路径拿满剩余之后等它判完，今天卡空着时本来就这样。
推翻条件：入库装置上，另一个进程借着 1 MiB 时起一趟状态数大于整卡的产品路径，30 分钟内借到草稿区。

### K2-2 S4、S4b：取卡登记与借草稿区各只试一次，这一刻满的卡整趟不用（打中）

取卡只登记此刻有空的卡（10w 第 2 条，主 agent 定的零轮设计），产品路径这一趟的卡就是登记上的那几张（「return match pool.cards_registered_waiting_only_when_none_has_room()? {」（`crates/singlefs-checker-tier/src/crash_amplification.rs:4823`）），
借草稿区只在这几张里试（「let card_indices: Vec<u32> = cards.iter().map(|card| card.index).collect();」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2279`）），
借到之后按块领活，这一趟不再重排；等的时候也只等这几张。
S4 历史：取卡那一刻进程 300 借满 5090（随即还了），进程 100 在 5060 Ti 上借着 4 000 不还；进程 200 取卡、借大流的草稿区。
S4b 历史：两张都登记上了；进程 200 借草稿区那一刻，5090 上有一笔 1 MiB 的短借据（借完马上还）。
```
MODEL scenario=s4 arm=today registered=[false, true] capacities_after_the_5090_was_returned=[21893, 9671] outcome_on_the_registered_cards=not_run_waited_past_the_longest_wait(need=9671)
MODEL scenario=s4 arm=fix_b_compress_to_room registered=[false, true] capacities_after_the_5090_was_returned=[21893, 9671] outcome_on_the_registered_cards=booked_states_per_card=[Some(3047)]
MODEL scenario=s4b arm=today registered=[true, true] holds=[21829, 9607] booked_states_per_card=[None, Some(9607)]
MODEL scenario=s4b arm=fix_b_compress_to_room registered=[true, true] holds=[21829, 9607] booked_states_per_card=[Some(19268), Some(9607)]
```
S4：今天等满报本次未跑，整个过程中 5090 是空的；改法乙只在 5060 Ti 上借到 3 047 个状态，5090 照样不用。
S4b：今天 5090 一个状态都不装，全部状态排在 5060 Ti 上判完这一趟；改法乙两张都装。

四句：分辨臂（S4 要的改法是「等的时候连没登记上的卡一起再试」或「登记推到借草稿区那一步」，推的，没实现；S4b 改法乙就分开了）；看得到（账本每张卡都读得到）；
满足 K2-2「把本可以分到几张卡的状态全压到一张上，卡空着不用」，S4 同时满足 K2-1「等满 30 分钟报本次未跑」；被攻的就是 10w 第 2 条与「借不到的卡不装状态」这两条零轮设计本身。
推翻条件：入库装置上，取卡那一刻借满第一张卡、随即还掉，这一趟的 `CRASH_AMPLIFICATION_GPU_PLACEMENT` 行里那张卡 `states_placed` 不为 0。

### K2-3 产品路径在池里每张有空的卡上登记、起设备（打中，压并发）

取卡登记点的是池里全部的卡（「let every_card: Vec<usize> = (0..self.cards.len()).collect();」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1306`）），登记上的每张都起设备（第 1345 行，见 K1-2）。
一条几十个状态的小流只装到第一张卡上，它在别的卡上也各占一份驱动那一份，直到进程退出（推的，读码）。驱动那一份是真起了设备，所以不算算错；
但它进了账，别的进程看那张卡就少了 2 560，S3 的整份容量压法下一份登记就够把大流挡到等满（S6）。
5060 Ti 没配额度时一个进程看到的容量是 9 671（S4 那一行），同时四个进程登记就占掉 7 680（另三份），那张卡上别人的 GPU 用例几乎借不到（推的）。
四句：分辨臂（改法丁：登记推到借草稿区或借缓冲那一步、只在用到的卡上起设备；判器的绑定上限改读适配器，池里已有 `smallest_max_storage_buffer_binding_size` 那一读法；推的）；看得到；满足 K2 题面「不压并发」与 K2-2；跑前改法没有碰这一处。

### K2-3 僵尸判死、驱动开销只记一份（没打中）

- 线程组长先退（主线程 `pthread_exit`、别的线程还在用卡）：`/proc/<pid>/stat` 第 3 项是组长的 `Z`，会判死、少记。Rust 测试二进制的主线程不先退，够不着，只记成理论形。
- PID 名字空间：账本按 `/proc/<pid>` 判活，同一个工作根被另一个名字空间的进程写会误判。本机我这条腿的 `ps` 看得到别的会话的进程号，同一个名字空间，够不着。
- 同进程重开账：卡池 `OnceLock`，一个进程只开一次；开账时自己没登记，`booked_mebibytes_by_every_live_process` 不算自己那一份，K 不重。
- 先后开账、后者读到的空闲里已经少了前者驱动那一份：上界 F + K 补回来，S1b 的数也照这个算对了。

## K3　I-3.14 第五形在各读法之间是不是同一个

### K3-1 五段头部（没打中，读码）

| 段 | 实现 | CPU checker | GPU 三处共用 |
|---|---|---|---|
| MAC 16 + nonce 12 | `pointer.rs` `PointerHeadAsRead::judged` 先判 28 字节全 0 | `image.rs` 偏移 0..28 | `crash_judge_common.wgsl` `all_zero(base, o, 28u)` |
| 算法类型 | 偏移 28 一字节 = 0 | 偏移 28 | `rd8(base, o + 28u)` |
| 压缩算法码 | 偏移 29 = 0 | 偏移 29 | `rd8(base, o + 29u)` |
| 压后长度 | 偏移 30..32 = 0（与压缩码无关地判） | 偏移 30 起两字节 | `rd16(base, o + 30u)` |
| extent 偏移 | 偏移 32..34 = 0 | 「const POINTER_EXTENT_OFFSET_FIELD: std::ops::Range<usize> = 32..34;」（`crates/singlefs-checker/src/image.rs:1341`） | `pointer_extent_offset_is_zero`（偏移 32 两字节） |

实现判法入口「fn judged(self) -> Result<PointerHead, PointerHeadFieldOutsideTheFirstVersion> {」（`crates/singlefs-core/src/pointer.rs:209`），链指针记录经「NodePointer::read_with_head_fields_judgement_from(」（`crates/singlefs-core/src/instance_table.rs:224`）。
CPU 三处读法共用「fn chain_record_view(record: &[u8]) -> ChainRecordView {」（`crates/singlefs-checker/src/walk.rs:749`），五段在「fn pointer_head_fields_take_the_first_version_values(pointer: &PointerView) -> bool {」（`crates/singlefs-checker/src/walk.rs:735`）。
GPU 走读与扫描两处经 `w_chain_record_kind`（MAC 到压后长度那四段在「return all_zero(base, o, 28u) && rd8(base, o + 28u) == 0u && rd8(base, o + 29u) == 0u && rd16(base, o + 30u) == 0u;」（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:545`）），
恢复内核经 `next.head_ok && next.extent_offset_zero`，`head_ok` 就是同一个函数。「全零」四处都按 86 字节判，与实现 `pointer_is_zero`（88 字节记录里第 2 字节起）同一段。
没找到一段漏判或口径不同。整段按小端读与按字节判 0 等价。

### K3-2 行不收、链不往下走、I-9.29 门关（没打中，读码）

- CPU 主走读：坏的那一片不 `pages.push`、`break`，「walk_stopped_at_a_page_that_does_not_decode = true;」（`crates/singlefs-checker/src/walk.rs:3307`）；末片标志只在 `LastPage` 那一支设，I-9.29 的门关着。
- CPU 引用集合（第 4828 行那一次 `chain_record_view`）：记成数不全、那条指针不记；数不全只让 I-3.9 跳过，不会多红。
- CPU 只读读法（第 6312 行）：整张表交 `None`。
- GPU 走读：「if (chain == 2u) {」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1705`）在收行之前 `break`，末片标志不设；
  GPU 扫描两处（`crash_judge_checker_scan.wgsl` 第 520、1170 行）与恢复内核（`crash_judge_recovery.wgsl` 第 279 行）同样数不全或整表读不出。
差别（已知、不新）：主走读收了坏片之前各片的行判 I-3.8，只读读法与恢复内核整张表都不要。这是 10o 判决记的 K2-2 / K2-4 欠账那一类，不算打中。

### K3-3 第五形那条指针 I-2.4 不判（读法站住，kb 行少一句）

I-2.4 行实现列有一般句「走读不跟随的指针 I-2.4 不判」（`.claude/kb/invariants.md:133`），第五形那条指针现在走读不跟随，照这句不判，站得住。
同一行又把实例表链指针列在「只判不断、指针指向的单元照样走下去」的五个入口里：「实例表链指针（`walk_instance_table_chain`，walk.rs:2836 起，根记录里的实例表指针是链的第 0 片）」（`.claude/kb/invariants.md:133`）。
第五形那一条既不照样走下去，I-2.4 也不判；那一行「走读不跟随」举的例子只有旧根的树表根指针。读者照 I-2.4 行会以为链指针头部坏了是 I-2.4 红、照样往下走，实际是 I-3.14 红、不走。
另外（推的）：链上第 k ≥ 1 片的指针过了 `chain_record_view` 才会被跟随，循环开头那三格 I-2.4（「judge_pointer_mac_and_nonce_are_zero(&mut self.judgements, &pointer, &pointer_what);」（`crates/singlefs-checker/src/walk.rs:3224`）一类）对 k ≥ 1 恒绿，只有第 0 片的指针还判得红。
改法：I-2.4 行在「走读不跟随的指针 I-2.4 不判」后补一句第五形那一条（归 I-3.14），实例表链指针那个入口注明第 k ≥ 1 片的头部已由 I-3.14 拦在跟随之前。kb 写回不归我。

## K4　测试与扫源码的判别力

### K4-1 新的有卡用例分不分得出「借了」与「没借」（打中，判别力缺口；推的）

用例看的是这一趟交回的借的量：「let booked = card.scratch_booked_mebibytes.unwrap_or_else(|| {」（`crates/singlefs-checker-tier/tests/crash_amplification_product_path_books_its_scratch_on_the_card_pool.rs:191`），
第 ② 段与单元校验和看「run.unit_checksum_buffers_booked_mebibytes」（同文件第 223 行）那几个数是不是 `Some` 且大于 0。
借据要活到判完才还，这一点用例分不出：把「scratch.leases.push(lease);」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2319`）改成 `drop(lease);`，
或把单元校验和那一处「lease: Some(lease),」（`crates/singlefs-checker-tier/src/crash_amplification.rs:4970`）改成 `lease: None,`，借的量照旧报出来、判器照旧用借来的那张卡的额度，用例照绿，账上判的那段时间却没有这一笔。
变异表里第 10w 批那三行（`crates/mutations.tsv` 第 3117–3119 行）改的是「改回不借」，不是「借了提前还」。
没跑这两条变异（checker 档，子 agent 不跑）。补法（推的）：用例在判器建好之后读一次账本（或把借据活着时的账本行数交回），比对这一笔还在。

### K4-2 扫源码检查还有哪种写法绕得过（打中，原型量过）

扫法原样拷进 harness 档原型（`build_scan_bypass_test.sh` 照冻结副本第 21–30、62–68、89–424 行拼），命令与原样输出：
```
CARGO_TARGET_DIR=$D/target nice -n 19 bash scripts/hosts/run-with-memory-cap.sh 4G bash scripts/hosts/capped.sh 12 cargo test --offline -p singlefs-harness --test opus_scan_bypass_model -- --nocapture
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
MODEL scenario=k4-2 sample=plain_call lines_found=[2]
MODEL scenario=k4-2 sample=raw_c_string_with_a_backslash lines_found=[]
MODEL scenario=k4-2 sample=raw_c_string_with_hashes lines_found=[]
MODEL scenario=k4-2 sample=out_of_line_test_module_file test_items_collected=0
MODEL scenario=k4-2 sample=out_of_line_test_module_file lines_found=[]
```
1. 原始 C 字符串：剥离只在 `r`、`b` 不跟在名字字符后面时认原始串（「('r' | 'b', Some(_)) if !follows_an_identifier => {」（`crates/singlefs-checker-tier/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs:190`）），
   `cr"…"` 的 `r` 跟在 `c` 后面，于是当普通字符串读、把 `\` 当转义，`cr"\"` 之后一直吞到下一个 `"`，同一行的起卡调用看不见。`cr#"x"b"#` 同理。
   两个样本是合法的 Rust：`rustc --edition 2021` 编得过，跑出来是 `cards=3 more=3`（`compile_check_raw_c_strings.rs`，经 `run-with-memory-cap.sh 1G` 跑）。
2. 库源码的测试放到另一份文件：库源码只收 `#[cfg(…test…)]` 属性下的那一项（「fn test_code_lines_of_a_library_source(lines: &[(usize, String)]) -> (Vec<(usize, String)>, usize) {」（同文件第 306 行））。
   `#[cfg(test)] mod gpu_tests;` 那一项是一行就收尾的 `mod`，`gpu_tests.rs` 整份没有 cfg 属性，按产品代码不扫（样本 `test_items_collected=0`、`lines_found=[]`）。今天 `src/` 下没有这种写法，是常见写法，会红的门槛低。
3. 不在名字表里的取卡路子（读码，不算词法漏洞）：用例自己 `wgpu::Instance` / `request_device` 起设备，或调 `crash_amplification::run_from_environment` 在没起池的进程里走 `DevicesStartedForThisRun`，名字表里都没有。前者今天 `tests/` 下没有，后者文件头写明是准许的几种调用方。
补法（推的）：剥离认 `cr` 前缀；库源码里 `#[cfg(test)] mod x;` 收进来时把 `x.rs`（或 `x/mod.rs`）整份当测试代码扫。

### K4-3 钉值、变异行拦不拦得住改回去（打中，推的）

- GPU 第五形只有 MAC 那一条对拍流：「fn the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_whose_chain_pointer_mac_is_not_zero_does_not_decode(」（`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:8003`）。
  变异表第 3093、3094 行整句删掉头部判法，MAC 流抓得到；只删 extent 偏移那一截——走读与扫描的「&& pointer_extent_offset_is_zero(base, o + 2u)) {」（`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:618`），
  恢复内核「if (flag == 1u && !next.all_zero && next.head_ok && next.extent_offset_zero) {」（`crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl:279`）里的 `&& next.extent_offset_zero`——MAC 流照绿，没有一条对拍流红。
  CPU 那一截有 extent 偏移的坏镜像与变异第 3092 行；GPU 少的是同一条 extent 偏移流。正文「已知没做」只列了另三段，extent 偏移不在里面。
- S5 那一处（产品路径照开账时的空闲装状态）今天是错的，也就没有用例钉它；改好之后要一条「起池时卡被记账进程占着、之后空了，第 ① 段照账装满」的用例。
- `started_card` 里「先登记、后建设备」那一句断言（`gpu_unit_checks.rs` 第 1057 行起）没有变异行：把起设备挪回开账（10s 的次序），harness 档的「借到了也不登记」那一行（第 3073 行）不覆盖 checker 档这一侧。
- 账这一侧 10t–10w 的变异行（`crates/mutations.tsv` 里文件列是 `gpu_memory_ledger.rs`、名字带第 10t–10w 批的，`awk` 数得 22 行）我没逐行核，也没跑；上面 S1–S6 那几形今天都是错的行为，变异表里没有对应的行。

## 几个改法各修哪一格（都只在我的模型上量过或只是推的，被攻过零轮）

| 改法 | S1 OOM | S2 判红 | S5 判红 | S3 / S6 等满 | S4 | S4b | K2-3 |
|---|---|---|---|---|---|---|---|
| 甲：每次借在锁里现读空闲、重夹估计 | 修（量过：over 20 000 → 0） | 修（量过：借到 10 000） | 不碰（推的） | 不碰（推的） | 不碰（推的） | 不碰（推的） | 不碰 |
| 甲 的连带：要的不超过配置额度时估计小了只等不红 | 修甲自己在 S1 引出的红（推的） | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 |
| 乙：按「容量 − 已记」压状态数 | 不碰 | 不碰 | 不碰 | 修（量过：S3 三档都借到；S6 第一个借到） | 半修（量过：只在 5060 Ti 上借到 3 047，5090 照样不用） | 修（量过：两张都装） | 不碰 |
| 丙：产品路径装状态只照账的容量 | 不碰 | 不碰 | 修（推的） | 不碰 | 不碰 | 不碰 | 不碰 |
| 丁：只在用到的卡上登记、起设备（登记推到借草稿区或借缓冲那一步） | 不碰 | 不碰 | 不碰 | S6 修（推的） | 修（推的：借草稿区时每张卡都试） | 不碰 | 修（推的） |
| 10o 第 3 条「每次起池都现读空闲」（今天已做） | 不中 | 不中 | 不中 | 不中 | 不中 | 不中 | 不中 |
| 10w「估计变小时在账本锁里重压」（今天已做） | 不中 | 不中 | 不中 | 不中（压的仍是整份容量） | 不中 | 不中 | 不中 |

甲的代价没量：每次借、每次看容量多一次 `nvidia-smi`（几十到几百毫秒，没量过）。乙让后来的那一趟只拿剩下的、一次派活状态少，挂钟会长，没量。

## 没打中的形状

- K1-1：第 ② 段与单元校验和的缓冲量（读了两段代码的建缓冲次数，对得上）；第 ① 段结论与回读缓冲少记一趟（量级可略）；驱动开销常数（正文列为已知）。
- K1-2：三处先登记后建设备，读码都成立；卡起不来时登记的那一份留到进程退出（偏保守，不算）。
- K1-3：锁外读、锁里不重判就记的，一处没找到。
- K2-3：线程组长先退、PID 名字空间、同进程重开账、后开账的驱动那一份重记（见 K2-3 那一节）。
- K3-1 / K3-2：五段 × 四处逐段对过；三件事在六处读法里逐处对过。
- 取样范围：模型只取了一张 5090、一张 5060 Ti、驱动那一份 2 560、表 64 MiB、每状态 1 MiB；外人 20 000 / 25 000、别人借着 1 / 4 000 / 15 000；配置整卡额度、24 G、没配三种。等满缩成 300 毫秒。
  没扫额度变量（`SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES`）设了的那一支，没扫三张卡以上。
- 用户动作放开扫：S1–S6 里由用户（别的进程）决定的那几步——外人占多少、别的进程借多少、借着多久——都取了几档；结论在每一档上方向相同（S1 两档配置、S2 两档配置、S3 三档都中）。S4 只取了一档。

## 这条腿自己的限度

- 都没上卡。S1–S4b、S6 是在 harness 档用今天的账（与一份改法拷贝）对一张假卡跑的；「卡上要的比卡大」是假卡按借到就建缓冲算的，真驱动的开销、wgpu 的延迟回收没进模型。
- S5 是读码加算术，产品路径那一层（checker 档、要 wgpu）没跑。
- K4-1、K4-3 的变异都是推的，没跑（checker 档）。
- 崩溃状态：这一腿的原型不枚举崩溃状态，没调 `closed_form_state_count`，跑的状态数 0（共用原型限额 10⁷ 一个没用）。
- 只抽了一次样；「没打中」那几格照规则一次不算数。
- 模型源码 `opus_ledger_attack_model.rs` 第一段写进文件时是 151 行，比每次 150 行的上限多 1 行；报告各段都在 150 行以内。

## 没做什么

- 没跑任何 checker 档测试、变异、门禁阶段；没上卡。
- 没判正推、辩方那几格；没替主 agent 采纳；副本上的数不算入库装置上的数。
- 没改 kb（K3-3 的补句交主 agent）。
- 别的会话此刻在本机跑 checker 档 GPU 测试（`ps` 看到 pid 2285875 一条 `cargo test … -p singlefs-checker-tier --features verdict-store,gpu …`），用的是真账本；我的模型账本在临时目录，没碰它。

## 草稿与编译目录

草稿目录 `~/.local/share/singlefs/work/m3-batch10t-code-r1-opus/`：仓的拷贝 `tree/`、编译目录 `target/`、复跑用的 `replay/`、日志。交回前删掉 `tree/`、`target/`、`replay/`，日志留着（`run-final.log`、`run-scan.log`、`replay.out`）。
