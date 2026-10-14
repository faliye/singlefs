# 根环实现第 10t–10w 批代码轮第一轮：正推腿报告（Sonnet，K1–K4 全部格）

日期 2026-10-07。立场：正推。先从判决（`research/prompts/m3-batch10o-code-r1-main-verification.md` 第三节、第五节）与用户原话推出「应该是什么」，再读冻结副本与代码逐格比：一样写一样，不一样把两边整行并排抄出来。
引代码写冻结副本里的路径（根 `~/.local/share/singlefs/work/m3-batch10t-code-r1/tree/`，下文 `crates/...` 都指它）。
冻结副本核过：`sha256sum -c research/prompts/m3-batch10t-code-r1-snapshot/crates-sha256.txt` 在副本根下 318 行全 OK。
腿自己的探针与输出在 `research/prompts/m3-batch10t-code-r1-sonnet-model/`（README.md 写怎么重建草稿副本）。全部只跑 harness 档、经 `scripts/hosts/run-with-memory-cap.sh`，没有跑任何 checker 档包的测试二进制，没有上卡。
用户原话（背景材料正文第一节转抄，代码注释同引）：「仅仅管会不会对指定的电脑造成cpu或者gpu的oom」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:7`）、「不压并发」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:7`）。

判定词：一致 / 冲突 / 规则没说。冲突与新形都标明「读码」或「量过」；量过的是账本这一层的纯 harness 单测，真卡上的行为（驱动、分配器）一律标「推的」。

## K1　显存账会不会让一台机器 OOM（只防 OOM 那一头）

### K1-1　有没有一形，账给出的容量大于卡上此刻真能用的

**(1) 估计的夹法 [现读, 现读 + 活进程已记]。**

- 一致：夹法与文件头写的一样。没有旧估计取现读的、有旧估计夹进区间：「Some(previous) => previous.clamp(lowest, highest),」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:119`）；没有活着的记账进程时 K = 0、区间收成一点，估计就是现读的 F：「没有活着的记账进程时 K = 0，估计就是现读的 F。」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:40`）。
- 一致：纯函数的四个点（无旧估计 / 外人占 / 外人走 / 订满）有单测，我复跑 30 条账本单测全绿：「test result: ok. 30 passed; 0 failed; 0 ignored; 0 measured; 87 filtered out; finished in 2.02s」（`research/prompts/m3-batch10t-code-r1-sonnet-model/rerun-quick.output.txt:2`）。
- 冲突（字面）：判决写的是「不沿用旧基线」，代码沿用旧估计、只是夹进区间。两边并排：
  - 判决：「每次起池都现读空闲」（`research/prompts/m3-batch10o-code-r1-main-verification.md:52`）；「不沿用旧基线」（`research/prompts/m3-batch10o-code-r1-main-verification.md:52`）。
  - 代码：「Some(previous) => previous.clamp(lowest, highest),」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:119`）。
  - 判决那句「现读空闲 + 活进程已借、还没实际占的那部分」里「还没实际占的那部分」读不到（A 读不到，文件头第 42 行自己写了），所以只能夹；夹出来的值在区间里取旧估计，旧估计没被外人推翻时与「现读 + 已记」不同。这一条不是错，是实现对判决字面的改写，写进来让主 agent 知道判决第三节第 3 条第一句没有被逐字实现。
  - 什么会推翻它：把判决那句改成「夹进 [现读, 现读 + 已记]」之后两边就一致。

**(2) 活进程「已记而还没实际占」的那部分。**

- 一致：上界 F + K 的 K 含每一笔借出去的与每个登记了的进程一份驱动开销：「records.booked_mebibytes_by_every_live_process(this_process, driver),」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:896`）。K − A 里 A 读不到，真值落在区间里。实际占的比记的多（A > K，例如驱动开销大于常数 2560）时上界偏小、只会少借：「驱动开销按常数记：真开销比常数大时上界偏小，只会偏保守。」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:44`）。
- 没找到打中这一项的形。

**(3) 外人在两次开账之间来去。**

- 一致（声明过的那一形）：「夹出来的估计在一种情形下大于真值：外人在两次开账之间占了显存，」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:42`）。算术：旧估计 30 000，外人占 8 000，后开账读 F = 22 000、已记 K = 22 560、没有实际占 A = 0，夹出来仍是 30 000，真值 F + A = 22 000，大 8 000 = min(外人 8 000, K − A 22 560)（纯算术，没跑）。
- **冲突（新形，量过）：开账之后一个进程就不再读空闲。** 空闲只在 `open_reading_the_free_memory_inside_each_card_lock` 里读一次：「let free_now = read_free_memory_mebibytes(position)?;」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:891`）。`grep -n 'read_free_memory_mebibytes(' crates/singlefs-harness/src/gpu_memory_ledger.rs` 的命中是第 868 行（文档注释）与第 891 行两处，没有第三处。此后每一次借都用账本里此刻的估计：「之后每一次借都照账本里此刻的估计算容量（[`GpuMemoryLedger::book_waiting_for_room`]），」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:40`）；只有别的进程再开账才会把它改小：「别的进程开账时把估计改小了，早开账的进程下一笔也照小的借。」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:41`）。
  - 量过（探针 A，假读数）：开账读 30 000，之后外人占走 25 000（假读数改成 5 000），没有第二个进程开账；账的容量还是开账那一刻的，借 20 000 当场成功。
    「PROBE_A reads_of_the_free_memory=1 fake_free_now=5000 ledger_capacity=[27440] booking_of_20000=Ok(20000)」（`research/prompts/m3-batch10t-code-r1-sonnet-model/k_forward_ledger_probe.run-output.txt:5`）
  - 这一形里账给的容量 27 440 比卡上此刻真空闲 5 000 大 22 440，与 K − A 无关（K = 0）。文件头第 42–43 行只写了「K − A 把上界撑开」那一形，这一形没写。
  - 并排：用户「仅仅管会不会对指定的电脑造成cpu或者gpu的oom」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:7`）；判决第三节第 3 条出口「出口：攻方量到的两个方向都由用例钉住，红在点名的格。」（`research/prompts/m3-batch10o-code-r1-main-verification.md:55`）。两个方向的用例（`gpu_memory_ledger.rs` 第 1792、1836 行的两条）都是「另一个进程开账」才改估计，单进程不刷新这一形没有用例钉。
  - 什么会推翻它：在 `gpu_memory_ledger.rs` 里找到开账之外再调 `read_free_memory_mebibytes` 或刷新估计的路径；或产品里每个进程开账之后一定还有别的进程开账（运行形态不允许单进程长跑）。
  - 另一类：不经账的调用方（分项驱动、起池之前的 `run_from_environment` 用例）照旧不进账，文件头写明：「没起过卡池的进程照旧不进账」（`crates/singlefs-checker-tier/src/crash_amplification.rs:37`）。判决第三节四条没有点它、用户原话的「指定的电脑」覆盖它，归「规则没说」，不算这一轮的错。

**(4) 第 ② 段与单元校验和的缓冲。**

- 一致（逐项对）：第 ② 段每趟建的三个缓冲是头、结论、回读：「let header_buffer = wgpu::util::DeviceExt::create_buffer_init(」（`crates/singlefs-checker-tier/src/crash_verify_gpu.rs:423`）、「let verdicts = wgpu::util::DeviceExt::create_buffer_init(」（`crates/singlefs-checker-tier/src/crash_verify_gpu.rs:432`）、「let staging = device.create_buffer(&wgpu::BufferDescriptor {」（`crates/singlefs-checker-tier/src/crash_verify_gpu.rs:440`）；记账的字节数是「bytes_of_one_pass: (FACTS_VERIFIER_HEADER_WORDS + 2 * states_in_the_largest_block)」（`crates/singlefs-checker-tier/src/crash_amplification.rs:1532`）：12 个字 + 每状态两个字，对得上。六张事实表按字数 ×4、空表一个字，同文件 1519–1531 行与 `crash_verify_gpu.rs` 里 `storage` 闭包（空表给 4 字节）对得上。
- 量（算术，python 现算）：一块 65 536 个状态，一趟 524 336 字节，同时活 2 趟，加上一份 20 万字节的表，第 ② 段记 2 MiB；单元校验和 40 次单元写、16 条核对线程记 81 MiB，400 次写 16 线程 801 MiB。
- 规则没说 / 推的（没上卡量过，要上卡才能核）：这两样按缓冲字节记，不含分配器的块粒度。设备按 `MemoryHints::Performance` 起：「memory_hints: wgpu::MemoryHints::Performance,」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:188`）；wgpu-hal 30.0.1 `src/lib.rs` 第 446–447 行 Performance 档的设备内存块下限 128 MiB、上限 256 MiB；gpu-allocator 0.28.0 `src/vulkan/mod.rs` 第 670–673 行释放时「最后一个通用块」不销毁（读 `~/.cargo/registry/src/*/` 里这两份源码看到的，没引成原文）。所以一个进程在一张卡上建过小缓冲之后，卡上多占的可以是一个 128–256 MiB 的块，记账只记了 2 MiB。这个量是否被「驱动那一份 2 560」吃掉，要上卡量；驱动那一份只在 5090 上量过一次（背景材料正文「已知没做」第 41 行写了，不算打中）。
- 什么会推翻它：上卡量一次「起设备 + 建第 ② 段缓冲 + 放掉」前后的 `nvidia-smi` 读数，进程多占不超过 2 560 + 记的量。

**(5) 当场起设备的驱动开销。**

- 一致：常数 2 560 记一份、每个进程在每张卡一份，开账读到的 F 里已经少了先登记进程那一份时，K 把它补回来、只记一份（单测与变异行 `第10t批 显存账 K4-2 开账的上界不算登记进程的驱动那一份…` 钉着）。
- **冲突（新形，量过）：卡上空闲不够一份驱动开销时，登记驱动那一份照样登记得上。**
  - 容量在 `saturating_sub` 里夹成 0：「None => budget_mebibytes.saturating_sub(mebibytes_left_for_the_driver),」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:88`）、「None => budget_mebibytes.saturating_sub(mebibytes_left_for_the_driver),」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:102`）。只登记驱动那一份要的是 0：「已记 0 + 要的 0 ≤ 容量 0」成立，「booked.saturating_add(need_mebibytes) <= *capacity」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:159`）判得过。
  - 量过（探针 G，没有别的进程登记、K = 0，开账读到 1 000 与 0）：登记成功、`is_this_process_registered_on_the_card` 为真，借 1 MiB 才被拒。
    「PROBE_G free=1000 capacity=[0] registered=Ok(()) registered_flag=Ok(true) booking_of_1=Err(NeedLargerThanEveryCard(NeedLargerThanEveryCard { need_mebibytes: 1, largest_capacity_mebibytes: 0 }))」（`research/prompts/m3-batch10t-code-r1-sonnet-model/run-g.output.txt:1`）
  - 并排：判决「先登记、后建设备，堵上中间那个窗口。」（`research/prompts/m3-batch10o-code-r1-main-verification.md:54`）；单测注释写的是「只登记驱动那一份的，放得下才登记，」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1908`），那条用例里第三个进程「放不下」靠的是已有登记使 K > 0、估计被旧值撑在 10 000，没有覆盖 K = 0 而 F 小于驱动那一份。
  - 后果（推的，没上卡）：登记上了，调用方随后就在这张卡上起设备（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1345` 对每张登记上的卡调 `started_card`）；这张卡空闲不到一份驱动开销，起设备要么失败（产品路径整趟判红，见 K2-1），要么把别人剩下的挤光。
  - 什么会推翻它：在真卡（空闲小于 2 560）上经产品路径起一次，设备起得来且不占超过空闲；或登记要求估计不小于一份驱动开销的判法被加上。

### K1-2　先登记、后建设备在三处都成立吗

- 产品路径第 ① 段：一致。取卡先登记：「return match pool.cards_registered_waiting_only_when_none_has_room()? {」（`crates/singlefs-checker-tier/src/crash_amplification.rs:4823`），池里先对每张卡登记再逐张起设备：「.map(|(card_position, _)| self.started_card(card_position).cloned())」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1345`）；借草稿区的那一笔在同一把锁里把进程登记上：「.push(self.settings.this_process);」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1038`）。起设备前有断言：「这个进程在它的账上登记之前就要起设备（先登记、后建设备）」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1069`）。
- 第 ② 段与单元校验和：一致。两处都经 `lease_the_first_card_with_room_waiting`（`crates/singlefs-checker-tier/src/crash_amplification.rs:4963`、`crates/singlefs-checker-tier/src/crash_amplification.rs:5111`）→ `book_waiting_for_room` → 记账时顺带登记 → `card_lent_for_the_booking` 才取 `started_card`。借的卡可以是这个进程没登记过的那张（候选是池里全部卡，已知没做的第三条），登记与借是同一把锁里的同一笔，没有窗口。
- 取卡登记：条件成立（先登记后建设备），但登记的判法有 K1-1(5) 那个洞。
- 两个不在这一问里、顺手看到的事实：
  - 那条断言只在违例时才响，把「先登记」改成「不登记就起」没有任何用例红（见 K4-3）。
  - 起卡池的进程同时在 `#[ignore]` 的量速度用例里经 `first_usable_card_or_report_not_run` / `gpu_cards_by_priority` 起不进账的设备（`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:218`、`:2907`），人工才跑，不算。

### K1-3　重压与借在锁里一气做完，有没有一形在锁外读了估计、在锁里借

- 产品路径按状态数借：一致，读容量、压状态数、判放得下、记账都在同一把 flock 里：「lock.lock()」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:832`）下，`book_on_the_card_if_it_still_fits` 里「let seen = Self::card_seen_by_this_process(」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1004`）、「what.need_within_the_capacity(seen.capacity_mebibytes);」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1011`）、之后同一个闭包里推借据。
- 有两处在锁外读容量，但在锁里重判，不是 K1-3 要找的形：
  - `book_waiting`（单笔借）先在锁外 `cards_seen_by_this_process` 读、挑卡，再进锁重判：「let cards = self」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1311`）。要的量固定，放不放得下由锁里那一次定。
  - `states_each_card_of_the_pool_lends` 在锁外读容量：「let ledger_capacities = pool.card_capacities_mebibytes_of(&card_indices)?;」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2186`），只当「每张卡最多装几个」的上界：「let states_to_place = states_left_to_place.min(*states_the_card_holds);」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:226`），锁里再压一次。读小了只会少装，读大了被锁里那次压住，不会借出超过此刻容量的量。
- 量过（探针 E）：三个假进程各两条线程，同一张卡（估计 40 000，三份驱动），各 150 轮按状态数借了就还；同一时刻借着的合计最大 32 000，不超过上限 32 320（40 000 − 3 × 2 560）。
  「PROBE_E highest_leases_held_at_once=32000 limit=32320」（`research/prompts/m3-batch10t-code-r1-sonnet-model/k_forward_ledger_probe.run-output.txt:14`）
- 判：一致，没找到锁外读估计、锁里借的形。
- 什么会推翻它：探针 E 在代码上跑出合计超过上限；或读码看到 `book_on_the_card_if_it_still_fits` 之外有一处记账不在 `with_the_card_ledger_locked` 里。
- 单测 `an_estimate_lowered_between_compressing_and_booking_is_compressed_again_and_the_booking_goes_through` 钉的是锁里重压（变异行 `第10w批 显存账 估计在压与借之间变小也不在锁里重压` 点名它）。

## K2　显存账会不会白白压并发（不压并发那一头）

### K2-1　有没有一形，卡上放得下、账却让它等满 30 分钟报本次未跑，或者判红

三形，都量过（账本层）。

**形 a：按状态数借时，状态数压到「整张卡的容量」，不压到「卡上剩下的空」；别的进程占着一部分就整笔等。**

- 代码：锁里先按容量压状态数、再判「已记 + 要的 ≤ 容量」：「what.need_within_the_capacity(seen.capacity_mebibytes);」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1011`）压的上界是容量；判放得下的是「booked.saturating_add(need_mebibytes) <= *capacity」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:159`）。压得比剩下的空大，这一笔整笔放不下。
- 量过（探针 C，假读数）：一张卡容量 20 000，别的进程已记 12 000 加驱动 2 560，空 5 440；这个进程要 30 个状态、每个状态 1 000 MiB。对照：直接借 5 000 的一笔当场借到（卡上放得下）。按状态数借则压成 20 个（20 000），放不下，等到上限、报等满，借 5 个状态本来放得下。
  「PROBE_C capacity=[20000] room_left=5440 control_lease_of_5000=Ok(5000) states_booking=WaitedPastTheLongestWait need=20000 waited_ms=300」（`research/prompts/m3-batch10t-code-r1-sonnet-model/k_forward_ledger_probe.run-output.txt:10`）
- 真账里等的上限是「pub const LONGEST_WAIT_FOR_GPU_MEMORY: Duration = Duration::from_secs(30 * 60);」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:435`）；产品路径等满报本次未跑：「ScratchBookingRefusalOutcome::NotRun」那一支（`crates/singlefs-harness/src/gpu_memory_ledger.rs:407`）。所以第二个进程在第一个进程长跑（以小时计）时，明明有空也干等 30 分钟报本次未跑。
- 并排：用户「不压并发」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:7`）；代码：只要别的进程占了任何一部分，后来的进程要的量压不到「剩下的空」就整笔让位。
- 分级：这不是 OOM 方向的错（只会少借、不会多借），是并发方向。判决三个格没有点这一形，也没有用例钉「压到剩下的空」。
- 什么会推翻它：真账上第二个进程在第一个进程占着一部分时，按状态数借到的状态数不为 0。或把压的上界改成「容量 − 已记」之后探针 C 借到 5 个。

**形 b：开账时外人占着，之后外人走了，同一个进程仍按小估计，红。**

- 代码：同 K1-1(3)，空闲只在开账读一次。量过（探针 B）：开账读 5 000，之后读数变 30 000；同一个进程借 10 000 得「比每张卡的容量都大」（红），别的进程开账读到 30 000 之后同一个账本对象的这一笔借到：
  「PROBE_B same_process_after_the_outsider_left=Err(NeedLargerThanEveryCard(NeedLargerThanEveryCard { need_mebibytes: 10000, largest_capacity_mebibytes: 2440 }))」（`research/prompts/m3-batch10t-code-r1-sonnet-model/k_forward_ledger_probe.run-output.txt:7`）
  「PROBE_B after_another_process_opened_and_read_30000=Ok(10000)」（`research/prompts/m3-batch10t-code-r1-sonnet-model/k_forward_ledger_probe.run-output.txt:8`）
- 并排：判决对 K4-3 第二个方向的描述「起池时卡被占着，卡空了以后反而判红或等满」（`research/prompts/m3-batch10o-code-r1-main-verification.md:42`）与第 55 行的出口要求「攻方量到的两个方向都由用例钉住」；今天的代码在「另有进程开账」时修好了这个方向，单个进程自己看不到卡空了。
- 红的出口：GPU 用例 `lease_card` 对「比每张卡的账都大」直接 panic（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1133`）；产品路径经 `print_the_not_run_line_or_refuse` 返回 Err、整趟判红（`crates/singlefs-checker-tier/src/crash_amplification.rs:2212`）。
- 什么会推翻它：存在开账之后再读空闲刷新估计的代码路径；或每个进程起池之后一定有别的进程开账（长跑单进程不成立）。

**形 c：登记驱动那一份在卡上空闲不够时照样登记，随后起设备失败，整趟判红（推的）。**

- 前半量过：K1-1(5) 探针 G。后半读码：`cards_registered_waiting_only_when_none_has_room` 对每张登记上的卡都起设备、任一张起不来整个 `collect::<Result<Vec<GpuCard>, String>>()` 返回 Err：「.map(|(card_position, _)| self.started_card(card_position).cloned())」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1345`）。就是说另一张卡完全够用，也会因为被外人占满的那张卡起不了设备而判红，不是「借不到的卡不装状态」。
- 卡空闲小于一份驱动开销时 `start_the_card` 是否失败、失败成什么样没上卡量：复核不了。
- 什么会推翻它：在一张被外人占满的卡与一张空卡的机器上跑一次产品路径，结果是只用空卡、不红。

- 没找到的形：账本整体放得下的 `NeverFits`、锁文件、判活等造成的虚假等满；30 分钟计时本身（`wait_once_more_unless_the_longest_wait_is_over`，`crates/singlefs-harness/src/gpu_memory_ledger.rs:1091`）读码没有问题。

### K2-2　「借不到的卡不装状态、重排到借到的卡上」会不会把本可分到几张卡的状态全压到一张上、卡空着不用

**形 d（读码 + 量过一半）：每一趟只试一次，没借到的卡这一整趟都不用。**

- 读码：装卡一次定下：`judge_on_one_card_and_start_the_next` 只从 `states_placed` 大于 0 的卡里挑下一张：「(position + 1..work.cards.len()).find(|candidate| work.states_placed[*candidate] > 0);」（`crates/singlefs-checker-tier/src/crash_amplification.rs:1966`）；没借到的卡在这个函数里不重试（一趟只调一次 `book_the_scratch_of_the_cards_waiting_only_when_none_has_room`，`crates/singlefs-checker-tier/src/crash_amplification.rs:2285`）。开头那一刻别的进程占着一点，这张卡就整趟（可以是几小时）不用，之后卡空了也不回头。
- 量过（探针 D，假读数）：两张卡，第 0 张被别的进程记了 12 000（空 5 440），第 1 张空着；要装 30 个状态、每张最多 30：第 0 张得 0 个（形 a 的结果），第 1 张按容量压成 20 个，剩下 10 个排队等前面判完。
  「PROBE_D states_per_card=[None, Some(20)] (card 0 had room for 5 states, 30 states wanted)」（`research/prompts/m3-batch10t-code-r1-sonnet-model/k_forward_ledger_probe.run-output.txt:12`）
- 并排：用户「不压并发」；代码里第 0 张卡「此刻放不下」判成这一趟不装：「/// 此刻放不下（别的借据占着）：这张卡这一趟不装状态，不等它；`need_mebibytes` 是压过之后要借的量。」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:205`）。
- 判：冲突，与形 a 叠加；单张卡完全被订满时这样处理是对的（判决第三节与 10w 报告 2 的原意），压的是「还有空但没压到剩下的空」的那一档。
- 什么会推翻它：长跑里没借到的卡在后面被重新借；或形 a 修了之后探针 D 里第 0 张得到 5 个状态。

**一致的部分**：「借不到的卡不装状态」的重排本身，`place_states_on_the_cards_that_book_now` 把没借到的那一份留给后面的卡：「let states_to_place = states_left_to_place.min(*states_the_card_holds);」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:226`）；借到几个就减几个；一张都没借到才等：「return ScratchPlacementDecision::JudgeOnTheCardsBooked;」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:270`）。单测两条（`a_second_card_booked_full_leaves_every_state_to_the_first_card_without_waiting`、`the_states_a_card_did_not_book_go_to_the_next_card`）我复跑都过（基线 30 条里）。

### K2-3　僵尸判死、驱动开销只记一份，有没有让账算重或算漏的形状

- 僵尸判死：一致。`/proc/<pid>/stat` 状态位 Z、X 判成不在：「Some("Z" | "X") => false,」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:519`）；认不出状态或启动时刻的当它还在（宁可多等）；读的时候出了别的错也当它还在。真子进程被杀而没收尸的用例我复跑过（基线 30 条里 `a_killed_process_that_its_parent_has_not_reaped_yet_is_judged_not_alive`）。判活在每一次进锁时做：「.without_the_records_of_dead_processes(&settings.is_alive);」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:843`）。
- 驱动开销只记一份：一致（K1-1(5) 前半；单测 `a_process_whose_free_memory_reading_already_lacks_the_driver_share_of_a_registered_process_counts_that_share_once` 在基线里过，变异行点名它）。
- 记账进程死了之后估计不动：F 涨多少、A 掉多少，F + A 不变（推的算术：F_again + A_again = F + A − A₁ + A₁），不会让后来的进程算漏或算重。
- 算重的形（读码，量级小）：登记上之后起设备失败的进程，驱动那一份留在账里直到这个进程退出（`registered_processes` 没有注销路径）。只在失败那种进程里发生，对别的进程是偏保守。
- 算漏 / 算重的主要来源是 K1-1(3) 与 K1-1(4) 那两形（单进程不刷新估计、分配器块粒度），不是这一格点名的两样。
- 没找到打中这一格的形。什么会推翻它：真子进程被杀之后账里仍有它的借据；或探针里两个先后开账的进程驱动那一份被记两次。

## K3　I-3.14 第五形在各读法之间是不是同一个

### K3-1　五段头部判法，三处读法与实现逐段一致吗

- 实现的判法（`crates/singlefs-core/src/instance_table.rs:192` 起的文档与 `:224` 起的读法，指针头部在 `crates/singlefs-core/src/pointer.rs:190` 起读、`:209` 起判）：MAC 16 + nonce 12 全 0、算法类型 0、压缩算法码 0、压后长度 0、extent 偏移 0，任一不过得 `None`。
  - 「有下一片而位置指针头部有一段不是第一版的取值（MAC / nonce、算法类型、压缩算法码与」（`crates/singlefs-core/src/instance_table.rs:192`）
  - 「NodePointer::read_with_head_fields_judgement_from(」（`crates/singlefs-core/src/instance_table.rs:224`）
- CPU `chain_record_view`：一致。偏移（指针内）MAC / nonce 0..28、算法类型 28、压缩算法码 29、压后长度 30..32、extent 偏移 32..34 与实现读的顺序对上：「let is_every_mac_and_nonce_byte_zero = reader.take(16 + 12).iter().all(|byte| *byte == 0);」（`crates/singlefs-core/src/pointer.rs:190`）；checker 读法「fn pointer_head_fields_take_the_first_version_values(pointer: &PointerView) -> bool {」（`crates/singlefs-checker/src/walk.rs:735`）与「1 if !pointer.all_zero && pointer_head_fields_take_the_first_version_values(&pointer) => {」（`crates/singlefs-checker/src/walk.rs:756`）；「全零」是 86 字节：「all_zero: bytes[..86].iter().all(|byte| *byte == 0),」（`crates/singlefs-checker/src/image.rs:1364`），实现是 `bytes[2..]` 也是 86 字节。
  - 量过（差分，不共用代码：checker 的 `chain_record_view` 是 checker 自己按字段表写的一份，实现是 core 的解析）：把两边喂同一条 88 字节记录，从一条「无下一片」、一条「有下一片」出发，逐字节改成 6 个代表值、前 36 字节里任两处同改、长度 0/87/89/176、20 万条固定种子的随机记录，共 203 579 条，两边的三态（最后一片 / 有下一片 / 解不开）一致：
    「PROBE_K3_CPU cases=203579 disagreements=0」（`research/prompts/m3-batch10t-code-r1-sonnet-model/rerun-quick.output.txt:13`）
  - 差分会红（弄坏 checker：extent 偏移不看）：
    「PROBE_K3_CPU cases=203579 disagreements=25」（`research/prompts/m3-batch10t-code-r1-sonnet-model/rerun-quick.output.txt:17`）
- GPU `w_chain_record_kind`：读码一致（WGSL 跑不了）。逐段对：宽不是 88 或 kind 不是 1 → 坏：「fn w_chain_record_kind(base: u32, o: u32, width: u32) -> u32 {」（`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:608`）；「有下一片」要位置指针不全零，加五段：「if (flag == 1u && !pointer_zero && pointer_encryption_and_compression_fields_are_the_first_version(base, o + 2u)」（`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:617`）；MAC / nonce 28 字节 + 算法类型@28 + 压缩算法码@29 + 压后长度@30：「return all_zero(base, o, 28u) && rd8(base, o + 28u) == 0u && rd8(base, o + 29u) == 0u && rd16(base, o + 30u) == 0u;」（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:545`）；extent 偏移@32：「return rd16(base, o + POINTER_EXTENT_OFFSET_FIELD_OFFSET) == 0u;」（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:538`）。偏移与 CPU 相同；指针起点是记录偏移 + 2，CPU 也是 `record[2..]`。
- 恢复内核：读码一致。「if (flag == 1u && !next.all_zero && next.head_ok && next.extent_offset_zero) {」（`crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl:279`）；`Ptr.head_ok` 与 `extent_offset_zero` 就是上面那两个函数的结果。公共内核里另有一份 `parse_instance_table_page`（`InstanceTablePage::parse` 的镜像）也判这五段：「if (pointer_zero || !pointer_encryption_and_compression_fields_are_the_first_version(p.base, last + 2u)) {」（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:2788`）。这一份的调用方我没追。
- 判：一致。什么会推翻它：GPU 两条读法在某条 88 字节记录上与 CPU 不同——要把 WGSL 经 naga 的 CPU 解释器或上卡对拍跑同一组 20 万条（我跑不了，见「要主 agent 跑的命令」第 1 条）。
- 另：五段里只有 MAC、extent 偏移两段有坏镜像与变异行，其余三段没有（背景材料「已知没做」第一条已写，不算打中）。

### K3-2　解不开之后「行不收、链不往下走、I-9.29 门关」三件事，第五形在每处都做到了吗

读码，按处：

| 处 | 行不收 | 链不往下走 | I-9.29 门关（没走到末片） |
|---|---|---|---|
| CPU 主走读 `walk_instance_table_chain` | `Malformed` 那一支不 `pages.push`：「ChainRecordView::Malformed => {」（`crates/singlefs-checker/src/walk.rs:3326`），上面 `LastPage`、`NextPage` 才推 | 同一支 `walk_stopped_at_a_page_that_does_not_decode = true;` 后 break（`crates/singlefs-checker/src/walk.rs:3327`） | 候选集滤过的标记只在 `LastPage` 一支设：「AbandonedRootsFilteringOfTheCandidateSet::FilteredByTheNewestRootsInstanceTable;」（`crates/singlefs-checker/src/walk.rs:3316`） |
| CPU 引用集合 `note_instance_table_pages_after_the_first` | 无行 | 第 4836 行那支 `is_complete = false` 后 return，不 `note` 下一片 | `is_complete = false` |
| CPU 只读不判读法 `instance_table_rows_of_root_without_judging` | `Malformed => return None`，连前面几片的行也不交（第 6318 行） | 同左 | 交 `None`，调用方当没有这张表 |
| GPU 走读内核 | 判红后 break，在「行那一半的账」之前：「if (chain == 2u) {」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1705`） | 同一 break | 「w_newest_instance_table_chain_reached_its_last_page = true;」只在 `chain == 0u` 设（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1739`） |
| GPU 扫描内核（引用集合） | 无行 | 「let chain = w_chain_record_kind(packed.base, last, packed.record_width);」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:520`）后 `chain == 2u` → `r_complete = false` return | `r_complete = false` |
| GPU 扫描内核（只读读法） | 「let chain = w_chain_record_kind(s.base, last, width);」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1170`）后 `chain == 2u` → `return false`，行不交 | 同左 | 交 false |
| GPU 恢复内核 | 末条不过 `return false`：「let flag = rd8(base, last + 1u);」（`crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl:273`）起往下走到第 286 行 | 同左 | `have_table` 为假，`abandoned_by_rows` 不被调（第 311、336 行） |

- 判：一致，每处都做到了。CPU 主走读红的是 I-3.14 那一格「chain != ChainRecordView::Malformed,」（`crates/singlefs-checker/src/walk.rs:3294`），GPU 走读对应「w_judge(INVARIANT_I_3_14, chain != 2u);」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1700`）。
- 已有的钉：主走读的「三件事」由 `a_chain_record_naming_a_next_page_through_a_pointer_whose_mac_is_not_zero_on_the_newest_instance_table_does_not_decode` 钉（判红集合精确等于 `VIOLATED_ONCE_THE_NEWEST_INSTANCE_TABLE_PAGE_DOES_NOT_DECODE`、I-3.8 报不适用、I-9.29 报部分没判，`crates/singlefs-harness/tests/checker_known_bad_images.rs:4362` 起）。GPU 各处由对拍流钉（我跑不了）。
- 什么会推翻它：对任一处造一张第五形的镜像，看到那一处收了这一片的行、沿那条指针往下走、或 I-9.29 报判完；GPU 部分要上卡。

### K3-3　I-2.4 不判这条读法，与 `.claude/kb/invariants.md` I-2.4 行站不站得住

- 条款：I-2.4 行写着走读跟随的每条指针判三格，同一行状态列写明不跟随的不判：「走读不跟随的指针 I-2.4 不判」（`.claude/kb/invariants.md:133`）。I-3.14 行写第五形算这一片解不开、用户 2026-10-06 定：「头部那一形用户 2026-10-06 定算第五种形态」（`.claude/kb/invariants.md:156`）。
- 代码：第五形那条指针 `chain_record_view` 交 `Malformed`、不构造 `NextPage`，主走读不把它当下一片的指针跟随；`pointer = next_page;` 只在 `NextPage` 一支赋（`crates/singlefs-checker/src/walk.rs:3322`）。所以不跟随、I-2.4 不判，与 I-2.4 行状态列那一句一致：站得住。
- 站得住之外两个事实，写给主 agent 判要不要写回 kb：
  - I-2.4 行状态列把「实例表链指针」列为五个判的入口之一，而第五形这一例是 I-3.14 红、I-2.4 不红；这一行没有点这一例（行里没有「第五」字样：`awk 'NR==133' .claude/kb/invariants.md | grep -c '第五'` 输出 0；I-3.14 那一行 `NR==156` 输出 1）。读 I-2.4 行的人会预期链指针头部非 0 红 I-2.4。
  - `walk_instance_table_chain` 每片开头仍对 `pointer` 判 I-2.4 三格：「judge_pointer_mac_and_nonce_are_zero(&mut self.judgements, &pointer, &pointer_what);」（`crates/singlefs-checker/src/walk.rs:3224`）、「judge_pointer_extent_offset_is_zero(&mut self.judgements, &pointer, &pointer_what);」（`crates/singlefs-checker/src/walk.rs:3230`）。第 0 片的指针来自根记录，第 k ≥ 1 片的指针只能来自 `NextPage`，而 `NextPage` 要求这五段都是第一版取值，所以对 k ≥ 1 的片这三格现在恒绿（读码，推的）。不是错，是这几格对链指针变成空判。
- 并排（口径）：I-2.4 行写「加密关着时恒 0、读者遇到非 0 一律判该结构损坏」，第五形的结构是这一片，由 I-3.14 红来判，口径一致，红的编号不同。
- 什么会推翻它：存在一条路径把第五形的指针当 `NextPage` 跟随（那时 I-2.4 该红而不红）；这条路径我读码没找到，CPU 差分 20 万条没有一条两边判得不同。

## K4　测试与扫源码的判别力

### K4-1　新的有卡用例，能不能分出「产品路径借了」与「没借」

读码（有卡的用例我不能跑）：

- 一致：两条用例把产品路径交回的「借的量」当判据，借了与没借分得开。第一条要求每张用到的卡的草稿区借的量是 `Some`、判器额度等于借的量加驱动那一份、判器开的草稿区不超过借的：「let booked = card.scratch_booked_mebibytes.unwrap_or_else(|| {」（`crates/singlefs-checker-tier/tests/crash_amplification_product_path_books_its_scratch_on_the_card_pool.rs:191`）、「card.memory_budget_mebibytes,」（`crates/singlefs-checker-tier/tests/crash_amplification_product_path_books_its_scratch_on_the_card_pool.rs:198`）。借的量由产品路径在拿到借据时填：「scratch.scratch_booked_mebibytes[card_slot] = Some(lease.mebibytes());」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2318`）；不进账那一支是 `None`。变异表里 10w 追加的三行（第 ① 段草稿区、单元校验和、第 ② 段）把三处改回 `DevicesStartedForThisRun`，红在这两条。
- 判别不到的（读码）：
  1. 判据是产品路径自己填的报告字段，不是账本文件。借据登记了、但借完立刻还了，或借在别的卡上，字段仍是 `Some`（第一条用 `memory_budget == 借的量 + 驱动` 间接钉住了用的是借来的那张卡，没钉借据活到判完）。
  2. 第二条只要求「借的量大于 0」：「.is_some_and(|booked| booked > 0),」（`crates/singlefs-checker-tier/tests/crash_amplification_product_path_books_its_scratch_on_the_card_pool.rs:224`），借多少不钉；量由 `crash_amplification.rs` 里的几条单测钉（`unit_checksums_book_two_ranges_per_unit_write_for_two_passes_on_every_checking_thread` 等，checker 档，我没跑）。
  3. 只排计划那一趟（`print_the_placement`）借不借：这两条用例不走；`crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs:962` 的 `a_placement_only_run_reports_the_states_judged_at_recording_as_reused` 只靠 10t 追加的最后一行（起过卡池的进程照样另起不记账的设备）间接管它，借草稿区那一步（`crates/singlefs-checker-tier/src/crash_amplification.rs:5193` 起）没有一条把它改回不借的变异行。
  4. 没卡、或账上被订满等满的机器上两条都报本次未跑：「if run.stopped_early {」（`crates/singlefs-checker-tier/tests/crash_amplification_product_path_books_its_scratch_on_the_card_pool.rs:159`）。这是设计，不是错，但这一格在没卡的机器上空过。
- 判：一致（能分出借与没借），限度如上；什么会推翻它：把借据登记后立即还（`leases` 不留到判完）而两条用例仍绿。

### K4-2　扫源码检查改成词法剥离之后，还有哪种写法绕得过

量过。把冻结副本里的扫源码测试文件（checker 档，我没跑它）拷成 `scan_copy.rs`（只给顶格 `fn`、`struct` 与三个字段加 `pub`，别的一字不动，做法见模型目录 README），在 harness 档测试二进制里拿它的 `code_lines`、`test_code_lines_of_a_library_source`、`lines_mentioning` 喂样本。对照样本（普通写法）扫得出。五种写法扫不出：

1. **C 原始字符串 `cr#"…"#`**：词法只认 `r#"`、`br#"`，`cr#` 的 `"` 当普通字符串起头，在内部的 `"` 提前收尾，后面的调用被当字符串内容吃掉。样本第 2 行 `let _a = cr#"x" "#; let _c = usable_gpu_cards(); let _b = cr#"y"#;`：
   「PROBE_EVASION_1 hits=[] (真实 Rust 里第 2 行调了 usable_gpu_cards)」（`research/prompts/m3-batch10t-code-r1-sonnet-model/rerun-quick.output.txt:27`）
   这一行是合法 Rust：`rustc --edition 2021` 编过并跑出 `"x\" " "y" 7`，`rustc 1.98.0 (88d9e12ae 2026-08-18)`（`research/prompts/m3-batch10t-code-r1-sonnet-model/crstr.rs`，原样输出见本节末的命令）。
2. **条目收尾在最外层遇 `,` 就停**：`#[cfg(test)] fn helper<A, B>() { … usable_gpu_cards() … }` 里泛型参数的逗号让条目在签名里就收了，函数体没进「测试代码行」；`where T: Copy,` 同样：
   「PROBE_EVASION_2 hits=[]」（`research/prompts/m3-batch10t-code-r1-sonnet-model/rerun-quick.output.txt:28`）、「PROBE_EVASION_2b hits=[]」（`research/prompts/m3-batch10t-code-r1-sonnet-model/rerun-quick.output.txt:29`）
3. **库源码里不带 `#[cfg(test)]` 的 `#[test]` 函数**（只在测试编译里存在）：
   「PROBE_EVASION_3 hits=[]」（`research/prompts/m3-batch10t-code-r1-sonnet-model/rerun-quick.output.txt:30`）
4. **`cfg(test)` 的几种不以「行首 `#[cfg(`」起头的写法**：文件级 `#![cfg(test)]`、`#[cfg(test)] mod x;` 而 `x.rs` 里没有 cfg、与别的属性写在同一行、`#[cfg (test)]` 带空格，四种都没收：
   「PROBE_EVASION_4 form=inner hits=[]」（`research/prompts/m3-batch10t-code-r1-sonnet-model/rerun-quick.output.txt:31`）、「PROBE_EVASION_4 form=own_file hits=[]」（`research/prompts/m3-batch10t-code-r1-sonnet-model/rerun-quick.output.txt:32`）、「PROBE_EVASION_4 form=same_line hits=[]」（`research/prompts/m3-batch10t-code-r1-sonnet-model/rerun-quick.output.txt:33`）、「PROBE_EVASION_4 form=spaced hits=[]」（`research/prompts/m3-batch10t-code-r1-sonnet-model/rerun-quick.output.txt:34`）
5. **不用那两个名字、直接用 wgpu 起设备**：名字表里只有 `usable_gpu_cards`、`gpu_cards_by_priority`（改走卡池的文件另加三个），没有 `request_device`、`wgpu::Instance`：
   「PROBE_EVASION_5 hits=[]」（`research/prompts/m3-batch10t-code-r1-sonnet-model/rerun-quick.output.txt:35`）
   今天 `grep -rn 'request_device' crates/singlefs-checker-tier/src crates/singlefs-checker-tier/tests` 只命中 `src/gpu_unit_checks.rs:181` 一处，现在没人这样写。
- 还有（读码，没造样本）：扫的范围只有 `tests/` 与 `src/` 里 cfg(test) 的项，`benches/`、`examples/`、别的 crate、`include!` 与 `#[path]` 引进来的目录外文件不扫；产品代码里加一个包装函数再从测试调它，名字不在扫的表里。
- 判：冲突（背景材料说词法剥离堵住了别名、字符串里的 `//`、非顶格 cfg 三种，那三种堵住了；以上五种另有）。数量上 1–4 都是 Rust 里正常能写出的写法，不要刻意。
- 什么会推翻它：扫源码测试在上面五个样本上红了；或样本经 `rustc` 编不过（第 1 项我编过了，2–4 是普通属性写法）。

C 原始字符串的编译与运行（原样，源文件 `research/prompts/m3-batch10t-code-r1-sonnet-model/crstr.rs`）：

```
$ rustc --edition 2021 -o tmp/crstr tmp/crstr.rs; tmp/crstr; rustc --version
"x\" " "y" 7
rustc 1.98.0 (88d9e12ae 2026-08-18)
```

### K4-3　钉值、变异行能不能拦住这几批里任何一处被改回去

量过一处，读表几处。

**量过：容量第三项（「估计 − 驱动」）被拿掉，30 条账本单测全绿，变异表里没有点名它的行。**

- 代码：设了额度变量而配置给卡的额度有值时，容量取 `quota.min(配置 − 驱动).min(默认份).min(估计 − 驱动)`；这时 `budget_mebibytes` 不进最小值、估计只在最后一项里出现：「.saturating_sub(mebibytes_left_for_the_driver),」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:99`）。这是 K4-3 那组「外人占卡」修复在「额度变量 + 配置额度」这个最常见组合下唯一起作用的地方。
- 变异（草稿副本，只改这一项成 `.min(u64::MAX)`）：
  「test result: ok. 30 passed; 0 failed; 0 ignored; 0 measured; 87 filtered out; finished in 2.81s」（`research/prompts/m3-batch10t-code-r1-sonnet-model/rerun-quick.output.txt:6`）
  我的容量上界格子探针在同一份变异代码上红：
  「test probe_f_the_capacity_never_exceeds_any_of_its_bounds ... FAILED」（`research/prompts/m3-batch10t-code-r1-sonnet-model/rerun-quick.output.txt:8`）
  还原后 `gpu_memory_ledger.rs` 的 sha256 与快照一致：「fb56c7c09ba90377130c0d2fadc8a2cb35c5b91d7251f9f27de1d79722f3e15a」（`research/prompts/m3-batch10t-code-r1-sonnet-model/rerun-quick.output.txt:11`）。
- 为什么活下来：K4-1 的单测（`an_environment_quota_larger_than_the_configured_card_quota_stays_within_the_configured_card_quota`）里估计 30 000、配置额度 8 192，第三项（27 440）不是最小的，从不绑定；K4-3 的四条外人用例用的是不设额度变量的容量闭包（`ledger_reading_the_free_memory`，估计直接当预算）。`grep -n 'memory_all_registered_processes_may_hold_mebibytes' crates/mutations.tsv` 零命中。
- 后果：额度变量设了、外人占卡，容量不跟着空闲走，是 OOM 方向的形（读码 + 单测层量过，上卡没量）。
- 什么会推翻它：变异表里有一行点名这一项且它的目标用例红；或我的变异（`rerun-quick.sh` B）在单测里红。

**读表：下面这些「改回去」，现有变异行与钉值拦不住**（`grep` 变异表 `crates/mutations.tsv`，3 121 行，里面 `gpu_memory_ledger.rs` 的行 32 条）：

1. `started_card` 里「先登记」的断言（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1061` 起）：没有一行点名它，也没有用例会在不登记起设备时红（断言只在违例时响）。
2. 只排计划那一趟的借（`print_the_placement` → `scratch_of_each_card_booked_by_where_the_cards_came_from`，`crates/singlefs-checker-tier/src/crash_amplification.rs:5193` 起）：`grep -n 'print_the_placement\|NotRunTheScratchWasNotBookedInTime\|placement_only' crates/mutations.tsv` 命中两行，一行是「只排计划那一趟复用数报 0」，一行是 10t 的「照样另起不记账的设备」，借那一步没有。
3. 10u 的五条变异行里 GPU 两条：改 `w_chain_record_kind`（公共，管走读与扫描两处调用）与恢复内核一处；扫描内核两处调用点各自被换成别的写法（不经 `w_chain_record_kind`）没有行点名。
4. 常数：`LONGEST_WAIT_FOR_GPU_MEMORY`、驱动那一份 2 560（`MEBIBYTES_LEFT_FOR_THE_DRIVER`）没有钉值也没有变异行（`grep -n 'LONGEST_WAIT_FOR_GPU_MEMORY' crates/mutations.tsv` 零命中；单测里的驱动那一份是各自写的常数 `DRIVER`，不引池里的）。
5. 外人占卡与单进程不刷新（K1-1(3)、K2-1 形 b）、压状态数不压到剩下的空（K2-1 形 a）：不是「改回去」，是今天的行为没有用例钉，上面几节有量。

**拦得住的**（读表、复跑）：10t 的额度三者最小（配置那一项）、僵尸判死、驱动只记一份、估计的上界 / 下界 / 取现读 / 照抄旧的四个变异、登记；10v 的 3/4 并进最小值、起过卡池的草稿区进账、没装状态不借、等满报本次未跑、记号、点名一张卡的借；10w 的一张借到不等、压过的卡照要装的扣、一张都没借到才等、锁里重压、登记不等订满的那张、缓冲量乘趟数。这些变异的目标单测在基线 30 条里都过（我没复跑变异本身，复跑变异整表归主 agent）。
钉值：三个内核的展开表达式数在 `crash_judge_dispatch.rs` 里按合入后的 319 137 / 381 545 / 329 390 钉（背景材料正文第二节已列），改回 10u 前的 WGSL 会动这三个数；Rust 一侧的改回不动它们。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1-1(1) 估计的夹法 | 一致；字面冲突 | 夹法与文件头一致；判决写「不沿用旧基线」，代码沿用旧估计夹进区间 |
| K1-1(2) 已记未占 | 一致 | 上界含已记；A 超过 K 只偏保守 |
| K1-1(3) 外人来去 | 声明过的一形一致；**新形冲突（量过）** | 开账之后不再读空闲，单进程里外人后来占的进不了估计（探针 A：容量 27 440 对真空闲 5 000） |
| K1-1(4) 第 ② 段与单元校验和缓冲 | 缓冲逐项一致；块粒度规则没说（推的） | 记 2 MiB / 81 MiB，不含分配器 128–256 MiB 的块，要上卡量 |
| K1-1(5) 驱动开销 | 常数一致；**新形冲突（量过）** | 卡上空闲不到一份驱动开销时登记驱动那一份照样登记（探针 G） |
| K1-2 先登记后建设备 | 三处一致 | 取卡登记、第 ①、第 ② 段都先登记；登记的判法有 K1-1(5) 的洞 |
| K1-3 锁内一气做完 | 一致 | 读容量、压、记账同一把 flock；探针 E 并发借合计 32 000 ≤ 32 320 |
| K2-1 白等或判红 | **冲突（量过）三形** | a 压到整张容量不压到剩下的空；b 外人走了单进程仍红；c 登记后起设备失败整趟红（推的） |
| K2-2 重排 | **冲突（读码 + 量过一半）** | 没借到的卡整趟不用、不回头；叠加形 a 时有空的卡得 0 个状态（探针 D） |
| K2-3 僵尸与驱动一份 | 一致 | 没找到算重算漏的形 |
| K3-1 五段判法 | 一致 | CPU 对实现差分 203 579 条零不同、差分会红；GPU 读码一致 |
| K3-2 三件事 | 一致 | 七处都做到；CPU 主走读有坏镜像钉，GPU 靠对拍（没跑） |
| K3-3 I-2.4 不判 | 站得住 | 条款写「走读不跟随的指针 I-2.4 不判」；I-2.4 行没点第五形这一例，对链指针 k≥1 的三格现在恒绿 |
| K4-1 有卡用例判别力 | 一致，有限度 | 分得出借与没借；判据是产品自填字段、只排计划那趟没钉 |
| K4-2 扫源码绕法 | **冲突（量过）五种** | C 原始字符串、泛型逗号、裸 `#[test]`、非行首 cfg(test) 写法、直接 wgpu |
| K4-3 钉值与变异行 | **冲突（量过一处）** | 容量「估计 − 驱动」那一项被拿掉 30 条单测全绿、表里没有行；另列几处无行 |

## 没做什么

- 没有上卡、没有跑 checker 档任何测试二进制、没有跑变异整表、没有跑 `gate`；GPU 三个内核（WGSL）的第五形只读码，没有对拍。第 ② 段与单元校验和缓冲的分配器块粒度、驱动那一份在小空闲卡上的行为，都是推的。
- 不判别的腿的格（攻方腿的切入角）没碰；不替主 agent 采纳或出判决；没有读禁读清单里的任何文件。
- 草稿副本里 `walk.rs` 末尾加的垫片、`scan_copy.rs`、变异都只在草稿目录，没有入库、没有写仓内别的路径（写了的只有模型目录与这份报告）。

## 要主 agent 跑的命令（我这边跑不了的）

1. K3-1 的 GPU 两条读法与 CPU 同一组 20 万条记录对拍：把 `k3_chain_record_differential_probe.rs` 的记录集喂 `w_chain_record_kind` 与恢复内核读实例表的 WGSL（要上卡或 naga 解释器）。归 checker 档 / GPU，主 agent 在清场之后按 `.claude/skills/crash-test/commit-stages.md` 的 GPU 阶段跑；看 `crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state` 那条第五形用例与 K3 差分记录集是否逐条一致。
2. K1-1(4) / K1-1(5) 的上卡量：一张空闲小于 2 560 MiB 的卡、一张空卡，各起一次产品路径，读 `nvidia-smi` 前后；再量「起设备 + 建第 ② 段缓冲 + 放掉」之后进程在卡上多占多少。上卡，需用户或主 agent 提交时批准。
3. 变异：把本报告 K4-3 的变异行（容量「估计 − 驱动」换成 `u64::MAX`，目标用例用探针 F 或新写的账本单测）加进 `crates/mutations.tsv` 之后由主 agent 在提交时跑变异整表。
