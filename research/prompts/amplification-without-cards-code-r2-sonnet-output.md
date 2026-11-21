# 崩溃放量写待判行之前先问服务：代码轮第二轮辩方腿报告

日期：2026-11-18。被复核的是第一轮判决 `research/prompts/amplification-without-cards-code-r1-main-verification.md`；对着主工作区此刻的树（行号都是这一轮里 `grep -nF` 现取的，第一轮的树已不在，第一轮的行号只引腿报告里整行抄下的原文）。没写模型、没起命令，读数没跑过的一律标「推的，没量过」。

**一句话：第一轮「坐实的」七条里够得着 6 条（第 7 条够不着）；修补对第 1、2、3、5、6 条改到位 5 条（第 1 条、第 3 条各留一处残留）；八格里不成立 2 格（类型与名字、不兼容旧代码与旧格式）、推不出 0 格。**
另有三处与判决写法不同：第 10 条该升成打中；改法 1 到 5 标零轮而不做探针 A/B 的理由，字面站不住、实质上 4 条站得住；惰性取卡退出不留半截行，我自己追了一遍，可以记成没打中。

## 一、判定一览

| 判决条目 | 辩方判定 | 一句话证据（原文与行号在后面各节） |
|---|---|---|
| 坐实 1：写了核对待判行没人判，空等到被停 | 够得着（形状）；两句附带话是推的 | 服务只登记镜像判定一族；「被停」没量、「合入前开头退 78」是读 HEAD 代码推的；这一格也是正文第 5 条自述过的现状 |
| 坐实 2：只排计划那一行只数第 ① 段 | 够得着，但只够到「行少数了」，够不到「驱动凭它定跑不跑」 | 驱动只看有没有这一行；注释写错了 |
| 坐实 3：PLACEMENT 留旧栏与旧读者 | 够得着 | 两条腿整行抄下同一句 Rust 格式串；驱动与入口的假行带旧栏 |
| 坐实 4：块号、态序号、入池序号是裸 `u64` | 够得着（字面）；同样打中别的臂 | 判器那一族的待判行 HEAD 里就是同一写法，这一张照抄 |
| 坐实 5：观察者盘 CPU 判起的线程数与拿到的不是一个数 | 够得着；是 HEAD 的存量 | HEAD 第 30660、30731 行同形，不是这三批新写的 |
| 坐实 6：判段开头注释与 GPU 路线拒算时的行为不一致 | 够得着；主 agent 没逐字核，靠腿的引文，引文经 cite-check 对上 | ① 段、② 段 GPU 路线拒算都是 `Err`，注释写「照常交回」 |
| 坐实 7：正文「多机可用」一格的依据写错 | 够不着：这是正文勘误，不是被判代码的打中 | 检查单那一格的「推不出」另算，今天有一次第二台上的真跑可对 |
| 记下 8：观察者盘惰性取卡靠环境 | 不升，站得住 | 任务服务提交时拒选卡变量，通道变量不再交；代码里另有「不照多机配置挑卡」的拒绝 |
| 记下 9：写回帧没有认领键与持有者 | 不升，站得住 | 判器那一族的认领键也是服务那一侧拼的，验证包的帧类型里同样没有 |
| 记下 10：借草稿区的分支没删净 | 站不住，应升成打中 | 那一臂是删 `print_the_placement` 时留下的壳，HEAD 里它有活的第二个调用方 |
| 没打中：惰性取卡退出不留半截行 | 可以记成没打中 | 退出点在 `judge` 里、写帧之前；每一次写都是单帧 |
| 改法 1 到 5 标零轮、不做 A/B | 字面站不住（规则没有「没有两臂」这个豁免）；第 3 条的删除半句、第 4 条实质上站得住 | 规则的两臂就是「有这一改法」与「没有它」 |
| 修补对第 1 条 | 改到位；留一处窄窗与一处服务接上之后的缺口 | ② 在今天的真服务上写不到行；① 问与写之间有窗 |
| 修补对第 2 条 | 改到位 | 第 ② 段两栏与真跑同一个函数数 |
| 修补对第 3 条 | 改到位；留一个没有调用方的旧栏读者（已登记延后） | `crash_amplification_placement_rate` |
| 修补对第 5 条 | 改到位 | 段数由拿到的条数定，不会多于它 |
| 修补对第 6 条 | 改到位（注释改了，行为没改） | 24280 起那段注释 |
| 八格 | 成立 6：走不到、不新建库、多机、多线程、`--item`、新崩溃流；不成立 2：类型与名字、不兼容旧；推不出 0 | 见第七节 |

## 二、第一轮坐实的七条，逐条

### 坐实 1：写了核对待判行没人判，空等到被停——够得着（形状）

- 没人判这一半撑得起：服务今天只登记镜像判定一族，「family: IMAGE_JUDGEMENT_FAMILY.to_string(),」（`gpu-verification-server/src/library_pending_pool.rs:152`）；`grep -rn 'facts_verifier_pending' gpu-verification-server/src` 今天 0 行（`grep -c` 数的），`grep -rn 'facts_verifier\|facts-verifier\|FACTS_VERIFIER'` 只命中 `service_process.rs` 的 305、313 两行，在第 282 行起的 `#[cfg(test)]` 里。
- 空等的出口是设计出来的：「块数不动就不打：服务答收这一族这把版本键却一直不判时日志不涨，交统一任务服务照卡住停」（`crates/singlefs-verification/src/crash_amplification.rs:33839`）。
- 判决写成事实、其实是推的两句：（a）「一直等到被任务服务照卡住停」。判决量的只是控制命令一次的 CPU（约 0.12 CPU 秒），自己写明等待圈自己的读库开销没量；卡住判法的阈值是「STALL_CPU_CORES = 0.01」（`task-server/task_server.py:836`），窗口里平均不到 1% 个核才算 CPU 没涨，等待圈每 2 秒一批点读加每 30 秒一次控制命令会不会把它顶过，没人量过；所以「被停」与「挂到四小时」哪个是结局，是推的，没量过。（b）「合入之前这类分项开头就退 78」。只有攻方腿读 HEAD 推过（HEAD 里第 ② 段要进程里有卡：「(Some(ComputedOn::Gpu), Some(cards)) => match verify_my_blocks_on_the_first_card(」，`head-crash_amplification.rs:23981`，这是 HEAD 的副本，在我的草稿目录里），没人跑过，是推的，没量过。
- 有没有判错：没有。判决自己把「前一半（服务没接之前先问）」与「根（服务接这一族）」分开记；第一轮正文第 5 条也自述过这个现状：「GPU 服务那一侧还不扫 `0x1F`」（`research/prompts/_amplification-without-cards-code-r1-body.md:37`）。所以被判的一方按设计写行、等服务，没做错；打中的是「写行的一方在另一方没接上时照写不误」这个组合。这一条不是被判一方本来的做法判错了。
- 推翻它的现象：第一轮那棵树里 `gpu-verification-server/src` 已经有读核对待判行的代码，或第 ② 段在写行之前就问了服务。

### 坐实 2：只排计划那一行只数第 ① 段——够得着，但够到的比判决写的窄

- 撑得起的：那一行只有判器那一段的块与状态，第 ② 段在核对键下还没核过的块不在里面（今天补上的两栏证明原来缺：「verifier_pending_blocks: verifier.not_yet_verified.len(),」，`crates/singlefs-verification/src/crash_amplification.rs:34753`）。
- 够不着的：判决从注释推出「分项驱动读 `pending_states` 定跑不跑」，写明「主 agent 没读驱动那一段」。驱动今天的判法：只看有没有这一行，`pending` 只在没有任何计划行时为空，各栏的值不拿来定跑不跑：「pending="$(crash_amplification_placement_total "$placement_log" pending_states)"」（`scripts/crash/crash-amplification-item.sh:194`），后面 `if [[ -z "$pending" ]]` 才分支。注释错了（改后的注释：「跑不跑只看排计划那一趟的日志里有没有这一行（一条用例几条流各一行；一行都没有就不跑、照日志分类退），不看各栏的值；」，`crates/singlefs-verification/src/crash_amplification.rs:34684`）；实现员在调度记录里自己核到「分项驱动第 244–245 行只看有没有这一行就定跑不跑」（`records/2026-11-13-全量提交门禁前收尾调度.md` 批 95 验证包那一件交回那一行，转述）。所以「第 ① 段都复用、第 ② 段没核过时这一行报 0、真跑却要写核对行」这一句成立，「驱动因此少跑」不成立（攻方腿也只说「0 照跑」）。
- 推翻它的现象：驱动里有一处用 `pending_states` 的值（不是有没有行）决定跑不跑。

### 坐实 3：PLACEMENT 留着旧栏与旧读者——够得着

- 两条腿整行抄下同一句 Rust 格式串，含「states_placed_on_this_host=0 states_placed_on_other_hosts=0」（`research/prompts/amplification-without-cards-code-r1-sonnet-output.md:172`，这是第一轮腿的原话；第一轮的树已不在，那一行 Rust 我读不到）。今天的七栏：「"{PLACEMENT_LINE_PREFIX} judge=gpu pending_blocks={} pending_states={} states_reused={} tables_mebibytes={} verifier_pending_blocks={} verifier_pending_states={}",」（`crates/singlefs-verification/src/crash_amplification.rs:34709`）。
- 驱动认旧栏的证据是腿的引文，没有主 agent 自己读的；今天的树里那个读者还在：「seconds += int(fields.get("estimated_seconds_on_this_host", 0))」（`scripts/crash/crash-amplification-lib.sh:692`），所以这条不是凭空打中。
- 有没有判错：没有。

### 坐实 4：块号、态序号、入池序号是裸 `u64`——够得着（字面），同样打中别的臂

- 字面成立：「pub pool_entry_number: u64,」（`crates/singlefs-verification/src/verdict_store.rs:362`）、「pub state_ordinals: std::ops::Range<u64>,」（`crates/singlefs-verification/src/verdict_store.rs:358`），块号在好几处是 `block_number: u64`。检查单这一格的判据：`.claude/skills/three-way/SKILL.md@代码轮正文的检查单`：「同一个量不同语义不共用裸整数（newtype）」。
- 同形先例：判器那一族的待判行同样写法，「pub pool_entry_number: u64,」（`crates/singlefs-verification/src/verdict_store.rs:222`）、「pub state_ordinals: std::ops::Range<u64>,」（`crates/singlefs-verification/src/verdict_store.rs:215`）；`git show HEAD:crates/singlefs-verification/src/verdict_store.rs` 里同一对字段在第 204、211 行（现查）。所以这一条打中的是仓里已有的写法，新写的核对待判行照抄；改法第 4 条「两张表一起换」是对的，只换新表会留一半。腿与判决没有一条把它写成这三批新引入的回归，判决也没有。
- 推翻它的现象：读到块号、态序号、入池序号各有自己的类型。

### 坐实 5：观察者盘 CPU 判起的线程数与拿到的不是一个数——够得着，是存量

- HEAD 里就是这样：拿的「checking_threads().min(images.len()),」（`head-crash_amplification.rs:30660`），起的「let threads = checking_threads().max(1);」（`head-crash_amplification.rs:30731`）。判决写了「这一处合入之前就有」，对。
- 打中的是被判文件里的存量，不是这三批的改动；它让检查单「测试优先多线程」里「里层并行拿线程经 `InnerThreadsTaken::take`」那句话在这一处不成立，撑得起。
- 推翻它的现象：HEAD 里起的线程数取自 `InnerThreadsTaken::threads()`。

### 坐实 6：判段开头注释与 GPU 路线拒算时的行为不一致——够得着；主 agent 没逐字核

- 判决自己写了「主 agent 没逐字核这两行」。腿的引文进了 cite-check 对上（判决第一节「核了 250 处引文」）。今天两处行为都是 `Err`：「"点 {point_index} 块 {local:?} 的结论版本照判不入库（判法单元拒算）：GPU 批处理服务写不进这个进程里的行，不写待判行"」（`crates/singlefs-verification/src/crash_amplification.rs:34048`）、「"这一把核对键照判不入库（GPU 核对器的判法单元拒算）：GPU 批处理服务写不进这个进程里的行，不写核对待判行"」（`crates/singlefs-verification/src/crash_amplification.rs:34961`）。腿原文的「照常交回（不退 78）」那句，今天还在，但已限定在「没开 GPU 的组」：「// 照常交回（不退 78）；那一组要写待判行交 GPU 批处理服务的（开了 GPU 的第 ① 段判器、第 ② 段核对），服务写不进这个进程里的行，」（`crates/singlefs-verification/src/crash_amplification.rs:24280`）。
- 同形：① 段与 ② 段一样，所以这条在两个臂上同样成立；没有只打中一边。
- 推翻它的现象：GPU 路线拒算的组在交错之前被记进 `refused_blocks` 照常交回。

### 坐实 7：正文「多机可用」一格依据写错——够不着（不是代码的打中）

- 两条腿指出的是背景材料正文的一句话写错（正文写「判定库里服务的在跑标记」，代码起的是控制命令子进程）。这是正文勘误，不是被判代码违反了哪条条款；判决已经把它换成检查单那一格的「推不出」，那一格另判，见第七节。
- 推翻它的现象：条款「所有任务都要多机可用」被这一句正文的依据直接违反（它是说明，不是行为）。

## 三、记下、没当打中的第 8、9、10 条

### 第 8 条（观察者盘惰性取卡靠环境）——不升，站得住

- 条款：「经 task-server/task submit -- <命令> 交的任务只跑录入、往判定库写待判行、读结论，不领卡、不分卡、不清场」（`.claude/rules/verification.md:50`）。条款管的是交给任务服务的任务。
- 判决说「任务服务交的任务环境里没有卡串与领卡通道」，今天能核：选卡变量任务带了提交时拒、服务自己环境里带的不交给尝试：「CARD_CHOICE_VARIABLES = ("GPU_CARDS_BY_PRIORITY", "CUDA_VISIBLE_DEVICES")」（`task-server/task_server_lib.py:2020`）；领卡通道的变量不再交：`grep -rn 'TASK_SERVER_CARD_LEASE' task-server/*.py task-server/task` 只命中自证里断言「不该有」的那一行：「CARD_CHANNEL_PREFIX = "TASK_SERVER_CARD_LEASE"」（`task-server/task_server_selftest.py:11057`）。
- 判决说「靠环境成立、不靠代码」，只对一半：代码里还有显式的拒绝，没经服务给卡而配置开了 GPU 时不挑卡、退 78：「不照多机配置自己挑卡，也不照本机空闲显存挑卡，清场与分卡由服务做（运行条件不满足，退 78）。」（`crates/singlefs-verification/src/gpu_card_configuration.rs:654`），在 `(None, …)` 那一支（第 643 行）。能起卡池的只剩环境里真有卡串的进程，任务服务造不出这样的任务。
- 站不稳的一处，明说：环境里有卡串时代码照样起卡池、借显存、判盘（手跑、别的起法），旧的领卡通道读者也还在（「pub struct CardLeaseChannelOfTheCaller {」，`crates/singlefs-verification/src/gpu_card_configuration.rs:370`），而它的发送方已不在，成了没有上游的旧机制残留。这不在被判的六份文件里，也不是这三批写的；观察者盘那一族交服务后整支删（改法 6）。
- 推翻它的现象：经任务服务交的任务进程环境里出现 `GPU_CARDS_BY_PRIORITY`、`CUDA_VISIBLE_DEVICES` 或 `TASK_SERVER_CARD_LEASE` 开头的变量。

### 第 9 条（写回帧没有认领键与持有者）——不升，站得住

- 验证包的帧类型里没有认领键：「pub struct FactsVerifierRowsForOneFrame {」（`crates/singlefs-verification/src/verdict_store.rs:453`），到第 461 行只有核对行守卫与待判行守卫；`grep -c 'claim_key' crates/singlefs-verification/src/verdict_store.rs` 打 0，判器那一族的写回帧同样没有。
- 判器那一族的认领键是服务一侧拼的：「claim_key: gpu_claim_key(&self.block.stream_key, self.block.block_number),」（`prune-server/src/gpu_judging_rows.rs:406`），协议要求非空：「if view.claim_key.is_empty() {」（`prune-server/src/protocol.rs:1170`）。核对那一族接上时同样由服务一侧拼，不是验证包这三批的缺口；并进改法 1 是对的。
- 推翻它的现象：判器那一族的认领键是验证包的帧类型带进来的（今天 0 处）。

### 第 10 条（借草稿区的分支没删净）——站不住，应升成打中

- 判决写成「死分支」「存量」的口气，实际是这三批自己删调用方留下的壳。被删的那一臂是 `card_capacities_of_this_run` 里的 `GpuCardsSource::TheCardPoolOfThisProcess`：「GpuCardsSource::TheCardPoolOfThisProcess => cards_of_the_pool_with_what_each_lends_now(」（`head-crash_amplification.rs:4851`）。HEAD 里它有第二个调用方，不是「只有一个调用方」：`print_the_placement`（「fn print_the_placement(」，`head-crash_amplification.rs:33622`）里「let local_cards = card_capacities_of_this_run(cards_taken_from, cards, tables)?;」（`head-crash_amplification.rs:33650`），`cards_taken_from` 在产品路径上正是 `gpu_cards_in_use` 交的 `TheCardPoolOfThisProcess`。`print_the_placement` 与 `PlacementOfThisRun` 是第一轮那几批删掉的（今天 `grep -c 'print_the_placement\|PlacementOfThisRun' crash_amplification.rs` 打 0）；删了调用方、留下被调用方的一臂，就成了壳。
- 第一轮正文检查单「不兼容旧代码」写的是「删掉的函数与类型（`verify_my_blocks_on_the_first_card`、`print_the_placement`、`PlacementOfThisRun`、`BufferLeaseOnTheCardPool` 这几样）没有留壳」（`research/prompts/_amplification-without-cards-code-r1-body.md` 那一格），实际留了一个。所以第 10 条该升成打中；它不改变那一格的结局（那一格已经因第 3 条不成立）。
- 同形、仍在的两处是 HEAD 的存量，不是这三批的壳：`judge_my_blocks_on_gpu` 只造 `Judged`（「Ok(GpuJudgingOfMyBlocks::Judged { tally, outcome })」，`crates/singlefs-verification/src/crash_amplification.rs:34410`），而两处产品消费方还留着显存不够那一臂：「GpuJudgingOfMyBlocks::NotRunTheGpuMemoryWasShort { tally, reason: _ } => {」（`crates/singlefs-verification/src/crash_amplification.rs:24984`）与「GpuJudgingOfMyBlocks::NotRunTheGpuMemoryWasShort { tally, reason } => Err(format!(」（`crates/singlefs-verification/src/crash_state_enumeration.rs:2757`）；造这一臂的只有 `judge_my_blocks_on_the_cards_started_for_this_run`（第 4957 行），它只被对拍用的 `judge_my_blocks_on_gpu_cards` 调。HEAD 里这两处消费臂与 `Judged` 同样都在（`head-crash_amplification.rs:33194`、`head-crash_amplification.rs:23913`）。这两个臂在产品路径上今天走不到；该不该把枚举拆成两个类型，交主 agent 判，不在我这一腿的范围。
- 推翻它的现象：`print_the_placement` 在 HEAD 里的 `cards_taken_from` 恒为 `DevicesStartedForThisRun`（那一臂当时就没有活的调用）。

## 四、惰性取卡退出时不留半截行——可以记成没打中

我从头追了一遍，没有照攻方腿的话走。
- 取卡与退出在这里：「.get_or_init(|| match (self.take_the_cards)() {」（`crates/singlefs-verification/src/crash_amplification.rs:31605`），取不到走「std::process::exit(IMAGES_AFTER_THE_OBSERVERS_WITHOUT_A_CARD_EXIT_CODE);」（`crates/singlefs-verification/src/crash_amplification.rs:31639`）。它只在 `pool_checker.judge` 里被头一张要判的盘触发：「let judged = pool_checker.judge(library, &images)?;」（`crates/singlefs-verification/src/crash_amplification.rs:33015`），上一行是「if !to_judge.is_empty() {」（第 32998 行），一张盘都不交回的块不进来。
- 退出之前这一批盘的判定还没编、没写；写它的是下一行起的「.store_observed_image_judgements(」（`crates/singlefs-verification/src/crash_amplification.rs:33019`），函数里把这一批的行放进同一个 `LibraryWriteBatch`、一次写出：「let mut batch = LibraryWriteBatch::default();」（`crates/singlefs-verification/src/verdict_store.rs:5136`）、「self.write_batch_keeping_the_kept_out_rows_in_this_process(batch)」（`crates/singlefs-verification/src/verdict_store.rs:5149`）。一块的观察者结论在这一块所有盘判完之后才写（`store_conclusion_block`，第 33158 行）。所以库里留下的只有「完整的前几批、前几块」，没有半帧；退出的这一块没有观察者结论行，下一趟 `observer_conclusion_is_reused` 判它没存过、重做，已存的盘判定按复用走。
- 崩溃枚举用例那一侧走同一个 `judge`（`crates/singlefs-verification/src/crash_state_enumeration.rs:2699` 的 `pool_checker_of_the_images_on_the_cards` 建同一个惰性成员），结论相同。
- `process::exit` 不跑析构，留下的只会是进程自己的临时文件（攻方腿说客户端追加日志的一次性目录；推的，没量过），不是库里的行。
- 判：可以记成没打中。按 `.claude/rules/three-way-inference.md@一条腿只抽一次样不算一次观测——否定结论尤其不算` 的字面，「只有一条腿覆盖到」要同立场腿再抽一次；我是辩方腿、不是攻方 s2 腿，我的追路算一次独立观测，要不要再派一条攻方 s2 腿，主 agent 定。
- 推翻它的现象：观察者阶段里有一次写不经单个 `LibraryWriteBatch`（先写标记行、再写本体）而退出点落在两者之间；今天我没找到。

## 五、改法 1 到 5 标零轮而不先做探针 A/B 的理由

规则：`.claude/rules/three-way-inference.md@判决由主 agent 做，不由投票做`：「有这一改法与没有它两臂喂同一批输入，量它在被打中的那一支上换不换得到东西（写进或出路），换不到的撤回。」。
判决写的理由：「它们不是在被打中的那一支上换一种算法，是补上缺的一方（第 1 条）、删掉没人走的路（第 3 条）、换类型（第 4 条）与让两处用同一个数（第 2、5 条），没有两臂可比」。

- 字面站不住：规则的两臂不是「两种算法」，就是「有这一改法」与「没有它」，量的是被打中那一支上的写进与出路。第 1 条恰好有可量的量：同一个「服务不收这一族」的输入，没有改法时库里多出一批核对待判行、一趟空等，有改法时库里 0 行、退 78。第 2 条的量是计划行的两栏，第 5 条的量是起的线程数与拿到的条数，都量得出。规则没有给「没有两臂」这个豁免，「补上缺的一方」恰是第 1 条要 A/B 的那种改法。
- 实质上的站得住：第 4 条是换类型，被打中的那一支没有运行时的量，A/B 量不出东西；第 3 条的删除那半句（借不到显存退 77 的分支、没有调用方的函数）删的是没人走的路，同样没有量。这两处不做 A/B 站得住。
- 第 1、2、5 条的实际替代：判决写的「先做成会红的测试再改」就是同一个输入上的有无两臂，只是缩在单测尺度，且发生在写进产品代码之后。今天能看到的：第 ① 段问到不收的单测断言两张待判行表都是空的：「"一行待判行都没写：两张待判行表都是空的"」（`crates/singlefs-verification/src/crash_amplification.rs:14464`，现查）。
- 另一个落差：规则要「A/B 读数由主 agent 经任务服务跑出，任务号写进判决「改法」一节那一条」，第一轮判决「三、改法」一节没有任务号。我看不到派实现员的提示，不知道派发闸「A/B 读数：」那一行是怎么过的（`.claude/hooks/runner-dispatch-guard.sh` 判行首的「A/B 读数：」，提示里有「零轮」才判）；这一点主 agent 自己对一下。
- 推翻它的现象：第 1、2、5 条的任一条在「没有改法」那一臂上量不出和「有改法」那一臂的差别（那样 A/B 会撤回它，而这三条的红测试写过「没有改法」会红，不会出现）。

## 六、修补是不是改到了被打中的那一格（今天的树上追原来那条路）

### 第 1 条：写了核对待判行之后空等到被停、库里留行

先列全：往库里写待判行的调用点今天一共四处，两处是头一次写、两处是等的时候补写。头一次写：① 段「for (point_index, local, block_number, stream, row) in rows_to_write {」（`crates/singlefs-verification/src/crash_amplification.rs:34087`）、② 段「for block in blocks.not_yet_verified {」（`crates/singlefs-verification/src/crash_amplification.rs:34993`），都在问服务收不收之后：「if !rows_to_write.is_empty() {」（第 34078 行）、「if !blocks.not_yet_verified.is_empty() {」（第 34984 行），问到不收就「return Ok(WaitingForTheFactsVerifierEnded::ServerDoesNotServe { line });」（第 34989 行），外层退 78（「std::process::exit(GPU_VERIFICATION_SERVER_DOES_NOT_SERVE_EXIT_CODE);」，第 34318 行）。补写：「// 待判行不在了却还没判过：照不在才写补写，接着等」（第 34142 行）、「// 核对待判行不在了却还没核过：照不在才写补写，接着等」（第 35051 行）。

原来那条路（② 段、服务不收核对这一族、写了行、空等）今天的结局：

- **② 段在今天的真服务上写不到行，构造不出。** 要写行，`python3 <控制命令> serves facts-verifier <键>` 得退 0，控制命令照服务「在跑」那一行里 `families=` 栏答；栏里只有镜像判定一族：「family: IMAGE_JUDGEMENT_FAMILY.to_string(),」（`gpu-verification-server/src/library_pending_pool.rs:152`），写这一栏的只有这一处，`facts-verifier` 只出现在服务单测的造行里（`gpu-verification-server/src/service_process.rs:305`）。查过的三处：`gpu-verification-server/src` 全部命中（见第二节）、控制命令 `serves` 的收法（「至少一台收退 0，一台都不收退 78」，`gpu-verification-server/control:32`）、验证包里所有写待判行的调用点（上面四处）。真环境一趟的结局对得上：任务 19423 那一趟的运行日志里，第 ② 段打了「CRASH_AMPLIFICATION_GPU_VERIFICATION_SERVER_DOES_NOT_SERVE family=facts-verifier」，同一行后面是「served=no host.this-host=running/unregistered」（`run-of-task-19423.log:45`，这是那一趟运行目录里 `run.log` 的副本，在我的草稿目录里；那一行里还有另一台的地址，我没抄）；分项汇总行是「CRASH_AMPLIFICATION_ITEM item=fixed-script-through-rollback exit=78」（`task19423.txt:191`）。库里有没有核对待判行，这一趟没量过；单测断言的是两张表为空（`crates/singlefs-verification/src/crash_amplification.rs:15238` 起那条用例，推的，没跑）。
- **① 段走得到的一条窄路：问到收之后服务换了键或停了。** 问只问一次，等的那一圈只问「在不在跑」、不再问「收不收」：「if let Some(line) = (waiting.server_not_running_line)() {」（第 34157 行）问的是 `check`。`check` 退非 0（含版本对不上退 1）一律走 `ServerNotRunning`、退 78；版本对不上看的是服务自己打的行：「有以 GPU_VERIFICATION_SERVER_VERSION_MISMATCH 起头的（src/library_pending_pool.rs 的 VERSION_MISMATCH_LINE_PREFIX）就退 1，」（`gpu-verification-server/control:86`）。所以 ① 段这条窄路的结局是「至多隔 15 圈（约 30 秒）再加服务打那一行所需的时间，退 78」，不是空等到被停（推的，没量过）；库里仍留着写下的行（设计如此：已写的行留给下一趟接着等），所以「库里留行」这一半在 ① 段仍走得到，留下的是有版本的、等对的服务来判的行，不是第一轮那种永远没人判的行。
- **② 段将来服务登记了这一族之后，这条路会回来一部分（推的，没量过）。** 等的那一圈对 ② 段同样只问 `check`，而 `check` 判版本对不上只认判器那一族的四个键：「VERSION_MISMATCH_KEYS = ("expected_gpu_judge", "this_gpu_judge", "expected_conclusion_version", "this_conclusion_versions")」（`gpu-verification-server/control:235`）。服务接上核对这一族之后，问到收、写了行、服务又被换成不登记这一族的二进制，没有任何一行信号让 `check` 退非 0，会回到空等。这是服务接核对族那一件要一并定的收口信号：要么服务对核对待判行也打版本对不上的行并让 `check` 读，要么等的那一圈每隔一段改问 `serves`。
- 补写（第 34144、35053 行）不问 `serves`：只在待判行已经不在、结论（核对行）还没有时才走；真服务一帧里写结论同时删待判行，造不出这个状态，要行被外力删掉。
- 单测构建里编的转接「&|_family, _expected_units| None,」（`crates/singlefs-verification/src/crash_amplification.rs:33960`）恒答收，只在 `#[cfg(test)]`，产品路径不经它。
- 判：改到位（原来那条路走不到）；留两处：① 段问与写之间的窗口（结局是 30 秒级退 78，库里留行），② 段服务接上之后的收口信号（现在没有，推的）。
- 推翻它的现象：在真服务上跑 ② 段，库里扫出这一趟写下的核对待判行（即某台服务的 `families=` 栏里已有 `facts-verifier`）；或 ① 段问到收之后换键重起服务，那一趟没在约 30 秒的量级内退 78。

### 第 2 条：只排计划那一行只数第 ① 段

- 原来的路（计划行 `pending_states=0`、真跑要写核对待判行）走不到：真跑决定写哪些核对待判行用的是「let blocks = blocks_not_yet_verified_under_the_verifier_row_key(」（`crates/singlefs-verification/src/crash_amplification.rs:34971`），计划行数第 ② 段用的是同一个函数：「let verifier = blocks_not_yet_verified_under_the_verifier_row_key(」（第 34738 行）；两处的分工都是「&|block| selected.contains(&block),」（第 24855 行、第 25048 行），核对键都是读回的事实表摘要加 GPU 核对器组键（计划那一处在第 24858 行）。
- 真环境一趟的计划行：「verifier_pending_blocks=32 verifier_pending_states=141」（`task19423.txt:172`，任务 19423 日志的副本，在我的草稿目录里），第 ② 段确实问不收，两者对得上。
- 判：改到位。残留：计划行是第 ① 段开跑之前数的，第 ② 段真跑在第 ① 段之后，两者之间别的进程核过的块不进计划行（只是估算，不影响判结局）。

### 第 3 条：PLACEMENT 留旧栏与旧读者

- Rust 与驱动：七栏（`crates/singlefs-verification/src/crash_amplification.rs:34709`）；驱动的 DECISION 只取 `pending_states` 与 `verifier_pending_states`（`scripts/crash/crash-amplification-item.sh:194`、同文件第 219 行）；驱动里 `host=` 零命中；入口的假行换成新栏：「echo "CRASH_AMPLIFICATION_PLACEMENT judge=gpu pending_blocks=6 pending_states=60 states_reused=40 tables_mebibytes=1 verifier_pending_blocks=2 verifier_pending_states=20"」（`scripts/crash/crash-amplification.sh:997`）；驱动自证的假行同样：「echo "CRASH_AMPLIFICATION_PLACEMENT judge=gpu pending_blocks=4 pending_states=$1 states_reused=0 tables_mebibytes=10 verifier_pending_blocks=2 verifier_pending_states=$2"」（`scripts/crash/crash-amplification-item.sh:278`）。「借不到显存退 77」那一支删了：自证里改成断言这样的行「没有计划行，不特判，照认不出原因退 247，不退 77」（`scripts/crash/crash-amplification-item.sh:513`）。原来出事的路都走不到。
- 残留两处，都在这一轮被判的六份文件之外：（a）没有调用方的「crash_amplification_placement_rate() { # <日志>」（`scripts/crash/crash-amplification-lib.sh:682`），里面读「seconds += int(fields.get("estimated_seconds_on_this_host", 0))」（第 692 行），`grep -rn crash_amplification_placement_rate scripts .claude/gate.d .claude/hooks` 只命中定义；已登记延后（`records/2026-11-17-subagent账本自上而下清欠.md` 第二百四十五节，转述）。（b）驱动自证里造一行已不会再有的旧原因：「printf '%s\n' "running 1 test" "NOT_RUN_NO_GPU_ADAPTER test=crash_amplification::run_from_environment reason=等了 1800 秒，池里没有一张卡空得出 2560 MiB" \」（`scripts/crash/crash-amplification-item.sh:526`），Rust 里「等了 N 秒，池里没有一张卡」零命中，这是造旧形态输入的测试，为证明「不再特判」留的。
- 判：改到位；残留（a）已登记、（b）我没见到登记。

### 第 5 条：观察者盘 CPU 判起的线程数与拿到的不是一个数

- 原来的路（拿「核对线程数与盘数里小的」那么多条，起的是没截的核对线程数）走不到：拿的是「crate::crash_library::InnerThreadsTaken::take(checking_threads().min(image_count).max(1));」（`crates/singlefs-verification/src/crash_amplification.rs:31746`），段长「let piece_length = image_count.div_ceil(threads_taken.max(1)).max(1);」（第 31758 行）按拿到的条数算，段数是盘数除以段长向上取整，不会多于拿到的条数；起线程的函数是「fn judge_the_images_after_the_observers_on_the_cpu(」（第 31767 行），一段一条线程，条数就是段数。
- 边界：盘数为 0 时段数为 0、拿的却是 1 条，产品调用方先过「if !to_judge.is_empty() {」（第 32998 行），到不了；盘数大于拿到的条数、不整除时（31 张盘 17 条切 16 段，前 15 段各 2 张、末段 1 张）段连着、按次序罩住每张盘，结果按段次序拼回，所以「结论按盘的次序交回」在不整除时成立；这个例子被单测钉住（`crates/singlefs-verification/src/crash_amplification.rs:14625`，读到的，没跑）。
- 判：改到位。钉住的只是切段函数，起了几条线程由 `pieces.len()` 定、靠读代码，没有单测数线程。份额 6 下里层下限 2 加一份临时服务 7 条合 9 的超订，判决里没有算打中，这一条也没动它。
- 推翻它的现象：`judge_the_images_after_the_observers_on_the_cpu` 起的线程数大于 `InnerThreadsTaken::threads()`。

### 第 6 条：判段开头注释与 GPU 路线拒算时的行为

- 注释改成了与行为一致：「// 照常交回（不退 78）；那一组要写待判行交 GPU 批处理服务的（开了 GPU 的第 ① 段判器、第 ② 段核对），服务写不进这个进程里的行，」（`crates/singlefs-verification/src/crash_amplification.rs:24280`）后面是「verify_my_blocks_through_the_facts_verifier_pending_rows），这一趟交错、不交回」（第 24282 行）。两处 `Err` 在问服务之前、写任何一行之前返回（第 34048 行早于第 34078 行，第 34961 行早于第 34984 行）。
- 判：改到位（注释改了、行为没改）。「拒算的组整趟交错」这个行为本身该不该改成记进 `refused_blocks` 照常交回，不是这一条要判的。
- 推翻它的现象：注释里还有一句说 GPU 路线拒算照常交回。

## 七、八格检查单

| 格 | 判 | 依据（细节在下面） |
|---|---|---|
| 走不到的理由二次查实 | 成立（实现员写的理由不是对的那一条，结论对） | 被删的那一臂 HEAD 里有第二个调用方，已随 `print_the_placement` 删掉 |
| 类型与名字 | 不成立 | 裸整数：第 4 条那一处加上两个新结构里的 `block_number: u64`；`_ =>` 零处、`expect` 都写了不变量 |
| 不兼容旧代码与旧格式 | 不成立 | 单测构建里的旧签名转接；边缘三处在下面 |
| 不新建判定库 | 成立 | 问服务读退出码不落盘，计划行只打印 |
| 所有任务多机可用 | 成立（一次第二台上的真跑为凭） | 任务 19423 在第二台跑，问到了两台 |
| 测试优先多线程 | 成立 | 切段照拿到的条数；份额 6 的实跑仍是推的，没量过 |
| 带 --item 的命令与项 | 成立 | 没有新项；17 行变异抓到，其中现判 1 条、复用 16 条 |
| 新崩溃流多罩了什么 | 成立 | 与已有 8 条同一个小流，只多一种库况 |

**走不到的理由二次查实：成立。**
- 实现员写的理由是「`judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from` 只有 `judge_my_blocks_on_gpu_cards` 一个调用方、卡的来路恒为 `DevicesStartedForThisRun`」。对着 HEAD 追：那个函数确实只有一个调用方，传「GpuCardsSource::DevicesStartedForThisRun,」（`head-crash_amplification.rs:4939`）。但被删的那一臂住在 `card_capacities_of_this_run` 里，HEAD 里它另有调用方 `print_the_placement`（第三节第 10 条已列）。所以「对着 HEAD 追」得不到「没有第二个调用方」，对的理由是：第二个调用方已随 `print_the_placement` 在第一轮那几批里删掉，今天只剩一处调用：「card_capacities_of_this_run(&cards_of_the_round, tables)」（`crates/singlefs-verification/src/crash_amplification.rs:5010`）。结论成立，理由要改。第一轮结束时的树已不在，我对不了那棵树，靠 HEAD 的调用方加今天的 grep 得出。
- 新加的产品代码里 `todo!`、`unreachable!`、`assert!`、`panic!` 零处：我在第 31740 到 31830 行与第 33660 到 35357 行里逐行找过，只有 `expect`，都带不变量，例如「.expect("要判的块都出自这份计划：上面照同一份计划给每一块编了号");」（`crates/singlefs-verification/src/crash_amplification.rs:34059`）。新的 `Err` 都在问服务与写行之前返回，问与写用同一份期待键。
- 推翻它的现象：`card_capacities_of_this_run` 今天有第二个调用方（`grep -n 'card_capacities_of_this_run(' crash_amplification.rs` 命中不止一处调用）。

**类型与名字：不成立。**
- 判据：`.claude/skills/three-way/SKILL.md@代码轮正文的检查单`：「同一个量不同语义不共用裸整数（newtype）、不缩写、不写 `_ =>`、`expect` 写清依赖哪条不变量，有一处不是就是打中」。
- 新加的结构里还是裸整数：`struct BlockVerifiedOnTheServer {`（`crates/singlefs-verification/src/crash_amplification.rs:34834`）、`struct BlockNotYetVerified {`（第 34841 行）各带 `block_number: u64`；`PlacementOfTheRun` 的块数是「pub pending_blocks: usize,」（`crates/singlefs-verification/src/crash_amplification.rs:34693`）、状态数是 `u64`，也是裸整数（正文已写明）；这些是第 4 条那一处的延伸，那一条延后。命名与 `_ =>`：`git diff HEAD -- crates/singlefs-verification/src | grep '^+' | grep -c '_ =>'` 打 0。

**不兼容旧代码与旧格式：不成立。**
- 一处够：「pub(crate) fn judge_my_blocks_through_the_pending_rows(」（`crates/singlefs-verification/src/crash_amplification.rs:33941`）是只在单测构建里编的转接，恒答收：「&|_family, _expected_units| None,」（第 33960 行）；它存在的理由是让 `crash_state_enumeration.rs` 里一条单测照旧参数调：「let ended = crate::crash_amplification::judge_my_blocks_through_the_pending_rows(」（`crates/singlefs-verification/src/crash_state_enumeration.rs:4849`）。条款：「改的文件里碰到的兼容代码连同它的测试与变异行一并删掉」（`.claude/agent-common.md:41`）。判：算兼容旧代码，打中；主 agent 已登记删它（`records/2026-11-17-subagent账本自上而下清欠.md:560`）。要删的话 `crash_amplification.rs` 里还有四条单测靠它的「恒答收」，要改成自己递一个答收的闭包。
- 边缘三处，条款字面都能数，归哪一轮交主 agent：（一）`a_control_command_that_does_not_know_the_serves_question_counts_as_not_serving`（`crates/singlefs-verification/src/crash_amplification.rs:14554`）与变异行「问服务收不收：控制命令不认 serves 退 2 也当收」（`crates/mutations.tsv:8162`）造的是「没有这个子命令的控制命令」，旧形态输入；它同时钉着「退 2 当不收」，改名成退出码 2 就不再是旧形态。（二）服务那一侧认没有 `families=` 栏的旧「在跑」行：「None => Vec::new(),」（`gpu-verification-server/src/service_process.rs:268`），文档注释写着「没有 `families=` 这一栏的行认成一族都不收」（第 6 行），这是只为旧行放宽的读法；不在六份被判文件里，正文已写明是服务那一侧的事。（三）脚本两处残留，见第 3 条。

**不新建判定库：成立。**
- 问服务读控制命令的退出码，`.output()` 不落盘：「.arg("serves")」（`crates/singlefs-verification/src/crash_amplification.rs:33761`）；只排计划那一行只 `println!`。`git diff HEAD -- crates/singlefs-verification/src | grep '^+' | grep -nE 'DB::open|rocksdb::|sqlite|create_dir|File::create|OpenOptions|fs::write'` 只有两处，都是测试里写替身控制命令与多机配置，不是判定存放处。

**所有任务多机可用：成立（一次样本）。**
- 正文说的「任务派到第二台时那一支没量过」今天有一次：任务 19423 的日志写「⑤ 第二台开跑（脱离 ssh 会话起，在同一个绝对路径」（`task19423.txt:7`），那一趟里 `serves` 问到了两台，两台各有一栏，都是 `running/unregistered`（`run-of-task-19423.log:45`：「served=no host.this-host=running/unregistered」，另一台那一栏含地址、我没抄），没有 `unreachable`。
- 保留：只有一次；`serves` 点名了卡的另一台时要经 ssh，成败取决于这台的 ssh 配置：「ssh -o BatchMode=yes {host} true 要退 0（免密；用户名照这台的 ~/.ssh/config）；修好再跑同一条命令」（`gpu-verification-server/control:1064`）。这是 `check` 早就有的机制，ssh 不通时那一台记 `unreachable`、不算收，结局有字可读，不是无声的。
- 推翻它的现象：任务落在第二台、那台到另一台的 ssh 不通而另一台恰是唯一点名了卡的，`serves` 退 78 而服务其实在收。

**测试优先多线程：成立。** 切段照拿到的条数起线程（第 5 条）；问服务与等服务是单线程，等的是别的进程；新单测没有按参数循环重建池。份额 6 下实跑仍是推的，没量过。推翻它的现象：份额 6 的一片里那两条「一块的各核对线程共用一份枚举表」单测仍红。

**带 --item 的命令与项：成立。** 没有新登记的崩溃枚举用例与放量分项；任务 19409 的日志写「点名的 17 条变异都抓到了」，同一份日志的复用栏是「现判 1 条、复用 16 条、拒算 0 条」（读到的，不在仓里），所以 17 行里 16 行沿用的是库里键相同的旧判定，不是这一趟重判；这符合「按单元键复用」，但「这一趟抓到 17 条」的说法要连着这一句读。

**新崩溃流多罩了什么：成立。** 新用例的流名是「"placement-facts-verifier",」（`crates/singlefs-verification/src/crash_point_replay_provenance.rs:723`），流用的是与别的用例同一个 helper：「record_new_pool_file_creation("crash-amplification-own-blocks", false, file_content());」（`crates/singlefs-verification/tests/crash_amplification_pipeline_records_checks_and_reuses.rs:355`）与新用例的「"crash-amplification-placement-facts-verifier",」（第 1483 行）。多罩的不是崩溃状态，是库况，与正文写的一致；准则「与已有流相同的只加小用例钉住「相同」」（`.claude/skills/three-way/SKILL.md:89`）。

## 八、顺带看到的，不在我这一腿的格里（交主 agent 与攻方腿的结论对）

- 控制命令的 `serves` 退 2（族名不认得、缺参数）、起不来（`python3` 不在、路径取错）一律归成「不收」、退 78：「exit_code: Some(0),」（`crates/singlefs-verification/src/crash_amplification.rs:33713`）之外的所有结局都走同一条出口。这一侧拼错族名这类缺陷会显示成环境问题；今天两个族名都被真环境验过（任务 19423：镜像判定一族问到收，核对一族问到不收，退出码是 78 而不是 2），日志行里有 `exit_code=` 与 `said=`，认得出，但不红。是否要把退 2 与起不来分出去当红，是设计取舍，我没判。
- `said=` 的拼法：标准输出里「.find(|line| line.starts_with(GPU_VERIFICATION_SERVER_SERVES_LINE_PREFIX))」（`crates/singlefs-verification/src/crash_amplification.rs:33726`）那一行原样带进去，没有这一行时取标准错误并把换行换成 ` / `（第 33728 行）；`said=` 是那一行 `exit_code=` 之后的最后一栏，后面紧跟全角冒号，行里带空格与等号时仍按「第一个 `said=` 之后到全角冒号」读得出（读代码，推的，没跑）。

## 九、要主 agent 跑的命令

每一条都是读数，没有一条是修什么。没起模型，所以没有 `rerun.sh`、没有文件 sha256。

1. 单测：问到不收一行都不写（两族各一条）、退 2 当不收、切段不超过拿到的条数、计划行逐字。
   - 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的 --lib 里的 5 条测试; 范围=5 个测试; 为什么=复核辩方腿对第 1、2、5 条的追路：问到不收一行都不写、控制命令退 2 当不收、切段不超过拿到的条数、计划行逐字' task-server/task submit --cwd /home/fy5090/code/singlefs -- cargo test -p singlefs-verification --lib -- crash_amplification::tests::no_pending_row_is_written_when_the_gpu_verification_server_does_not_serve_the_image_judgement crash_amplification::tests::no_facts_verifier_pending_row_is_written_when_the_gpu_verification_server_does_not_serve_the_facts_verifier crash_amplification::tests::a_control_command_that_does_not_know_the_serves_question_counts_as_not_serving crash_amplification::tests::images_after_the_observers_judged_on_the_cpu_are_cut_into_no_more_pieces_than_threads_were_taken crash_amplification::tests::the_placement_line_is_spelled_out_word_for_word --exact`
   - 在哪起：仓根 `/home/fy5090/code/singlefs`。
   - 看哪几行：`TASK_SERVER_RESULT` 的 `exit=0`；`task-server/task log <号>` 里 `5 passed`（跑到 0 条也退 0，要核「5 passed」；这几条要带 `gpu` 特性才编进来的话读到 0 passed，改成加 `--features gpu` 重交）。推的，没量过。
2. 单测：只排计划那一行数第 ② 段（库里第 ① 段结论都在、第 ② 段一块都没核过）。
   - 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 crash_amplification_pipeline_records_checks_and_reuses 里的 1 条测试; 范围=1 个测试; 为什么=复核辩方腿对第 2 条的追路：计划行第 ② 段两栏等于整条流' task-server/task submit --cwd /home/fy5090/code/singlefs -- env SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=2 cargo test -p singlefs-verification --release --features gpu --test crash_amplification_pipeline_records_checks_and_reuses -- a_placement_only_run_counts_every_state_for_the_facts_verifier_when_the_judge_conclusions_are_all_stored --exact`
   - 在哪起：仓根。
   - 看哪几行：`exit=0`、`1 passed`。被范围声明闸拒了，照它的说明在缩域与全域理由里二选一改。推的，没量过。
3. 第二台上 `serves` 的另一次样本（任务 19423 只有一次）：问一把不存在的键，答案必是 78，要看的是那一行 `GPU_VERIFICATION_SERVER_SERVES` 里另一台那一栏是 `running/unregistered` 还是 `unreachable`。
   - 命令：`task-server/task submit --cwd /home/fy5090/code/singlefs -- python3 gpu-verification-server/control serves image-judgement 0000000000000000000000000000000000000000000000000000000000000000`
   - 在哪起：仓根。
   - 看哪几行：`task-server/task status <号>` 读它跑在哪台；`task-server/task log <号>` 里 `GPU_VERIFICATION_SERVER_SERVES` 那一行的两个 `host.` 栏；`exit=78` 是预期，两台各落一次（多交几次，直到两台各有一条）；任一栏是 `unreachable` 就是第 5 格要翻成不成立。

## 十、没做什么

- 没替主 agent 撤销或维持判决，只交复核结果；没往统一任务服务交任务；判决要引的数没复跑（命令在第九节）；读数没跑过的都标了「推的，没量过」。
- 没写模型、没建仓副本、没起样本进程，没起任何 `cargo`；没读禁读清单里的两处。
- 没量：等的那一圈自己读库的开销（会不会让卡住判法不触发，第 1 条的「被停」）；份额 6 下的实跑；② 段真服务接上之后的收口信号；任务 19409 的日志不在仓里，我读的是它的末段。
- 第一轮的树已不在，第一轮的行号我只引了腿报告里整行抄下的原文；对不上今天树的地方都改成今天的行号，并标了「HEAD」或「今天」。
- 只看了六份被判文件里与我这一腿的问题相关的部分；第 7 条之外，判决里别的「没打中」格（核对待判行四种时序、复用数三种库况、里层份额与锁、空枚举）我没重追。
- 草稿目录里留着：`find.py`（找行号）、`head-crash_amplification.rs`（HEAD 的副本，第三节第 10 条与第七节引它）、`run-of-task-19423.log`（任务 19423 那一趟运行目录里 `run.log` 的副本，里面有另一台的地址，不进仓）、`task19423.txt`（任务 19423 的日志）。都是单个文件，没有仓副本、编译目录、库拷贝；删不删交主 agent。
