#!/usr/bin/env bash
# 不领卡（.claude/rules/verification.md「卡只归 GPU 服务，任务不拿卡」）：shard 那一步起的那一道上卡的格一格接一格、环境里的 GPU_CARDS_BY_PRIORITY 原样交，
#   不上卡的格看不到卡（scripts/mutation/crates-mutation-rows.py 文件头「上卡的格一格接一格」）；plan、merge 两步不上卡
# task-server-compiles: yes
# 门禁 crates-mutation-replay（crates 变异表复跑）分片那一套：服务把经它交的那一道拆成计划、各片、合并三步（.claude/gate.d/crates-mutation-replay.sh 文件头「分片」），
# 那一道照 TASK_SERVER_SHARD_PHASE 把这一步转到这里：本机库里没有键相同判定的行按内存定片数（每片预计内存不超过安全额度，下面「三步」plan 的 ④），
# 各片之间按用时表均分，每片各起一趟 crates-mutation-replay 只判自己那一片（GATE_MUTATION_ROW_SELECTION）；
# 各片判出的记录都经 prune-server 写进同一份判定库（.claude/rules/verification.md「判定库只有一份」），不拷库目录、不导入；
# 合并那一步只核各片判的行按本机此刻的行键在服务里读得到（scripts/mutation/crates-mutation-rows.py row-plan），之后由那一道在同一步里核账判整张。
# 这一份不自己连第二台、不给哪一片套内存包装（机器、上限、清场由服务定）。
#
#   bash scripts/mutation/mutation-shard-run.sh <树根> [--only-rows <选行文件>]
#                                      只由 crates-mutation-replay 在服务分片的三步里起（环境里有 TASK_SERVER_SHARD_PHASE 与它那一步的变量，写法在 task-server/task_server_lib.py 文件头「分片」）。
#                                      --only-rows 的选行文件一行一个变异表行号（那一道把 --item 换成它）：只在选中的行里分（③ ④ 同一套规则）。
#                                      没有 TASK_SERVER_SHARD_PHASE 退 2、什么都不分，出路写经服务交那一道；
#                                      在选行模式下被调（环境里设着 GATE_MUTATION_ROW_SELECTION）退 2；弄坏开关 MUTATION_SHARD_RUN_BREAK 设在 --selftest 之外退 2
#   mutation-shard-run.sh --selftest   自证：假 cargo、假 llvm 工具，在临时目录里的小仓上照 task-server 设的变量逐步起三步，逐格见文件末尾 mutation_shard_selftest 的说明；格数由成功行现算。
#                                      退 0 都对；退 1 有格判错
#
# 三步（TASK_SERVER_SHARD_PHASE）：
#   plan   ③ 分行（mutation_shard_split_plan）：crates-mutation-rows.py row-plan 列出每一行的行键、这一趟会不会复用、上一次现跑的用时与锚点失效没有；
#          要现判的行（拒算的不算：别的片判出来也没有键可导，留给合并那一步的核账照判；第五列是 anchor-broken 的锚点失效行不算：不跑，核账记锚点失效）
#          逐行写进 <计划目录>/pending.rows（「<行号>\t<秒>」），没记过用时的按表里记过的中位数算（一条都没记过都按 1 秒）。
#          不进各片的行（拒算、锚点失效、复用：第三列 1，库里有键相同的判定）逐行写进 <计划目录>/after-merge.rows（「<行号>\t拒算|锚点失效|复用」），
#          由合并之后那一道的核账判；点名的行（带 --only-rows 是选行文件里的，不带是 row-plan 列的每一行）有一行既不在 pending.rows、也不在 after-merge.rows 里，
#          计划不写、整步退 1，点名那几行（弄坏开关 reused-rows-unlisted）。成功句报各片几行与拒算、锚点失效、复用各几行。
#          ④ 按片估内存定片数（mutation_shard_memory_groups 把要现判的行按估值分组、定每组每片至多几行，写 <计划目录>/pending.groups；mutation_shard_assign 分）：
#          · 一片开的工作进程数 = min(每台默认重活线程数（GATE_MUTATION_WORKERS 给了照它）, 这一片的行数)（crates-mutation-rows.py 文件头「工作进程数」：工作进程数不超过要现跑的条数）；
#            这一片预计内存 = 这一片的行里每个工作进程的峰值最大的那个 × 工作进程数 + 父进程（MUTATION_SHARD_PARENT_BYTES）。一片只放同一组的行，估值只看这一组的行，不看别的组。
#          · 每一行的估值取这道门禁自己记下的量：crates-mutation-rows.py 的 measured_worker_peaks 读运行记录目录下的 worker-peaks/
#            （它文件头「内存上限」的「每个工作进程的峰值」），worker_peak_for_jobs 只取这一行点名的记录名（worker_peak_keys 只喂这一行：取测试目标那一级的 <包>--…，没有取包名；
#            目标那一级没有实测的照它文件头「内存上限」的「怎么除」取同包测试目标那一级的中位数或起步值）在那个工作进程数的编译并行度下（cargo_jobs_for，
#            整台线程数取 compile_thread_budget：每台默认重活线程数）的峰值；在那个与更高的编译并行度都没有实测的包、整份记录读不了，按保守常量 MUTATION_SHARD_UNMEASURED_WORKER_BYTES 算。
#          · 安全额度 = 多机配置 SLICE_MEMORY_CAP_BY_HOST 各台里最小的那一项（片派到哪一台由服务挑，计划照最紧的那一台算）× MUTATION_SHARD_MEMORY_SAFETY_NUMERATOR ÷ MUTATION_SHARD_MEMORY_SAFETY_DENOMINATOR；
#            配置里没写、写法不认、读不了，计划不写、整步退 1。
#          · 分组：每一行分「测试目标」（恰好点名一个测试目标，worker_peak_target_key 交得出）与「整包」（整个包跑、几个包、没点名目标）两种，整包的行估值是整个包的最大，不与测试目标的行进同一组；
#            点名同一组记录名的行每片至多几行相同：行数从 1 往上试，到要现判的行数为止，每一个都满足「预计内存 ≤ 安全额度」的最大行数；开一个工作进程就超的按每片 1 行。
#            同种类、每片至多几行相同的并成一组；同种类里按每片至多几行从少到多，相邻一组并进前一组（每片至多几行取前一组的）不多出片时就并。
#          · 片数：每组照内存要 ⌈组的行数 ÷ 组的每片至多几行⌉ 片，各组合起来不超过服务交的 TASK_SERVER_SHARD_MAXIMUM（不是正整数退 2，plan.json 不写）。
#            组比上限还多时，每片至多几行相邻的两组里合起来行最少的两组并成一组（每片至多几行取小的），并到不超过上限；之后各组先各一片，
#            余下的片一片一片给「一片约要跑多久」（组的总用时 ÷ 片数 ÷ min(一片满了开几个工作进程, 每片行数)）最长、还没到内存要的片数的那一组。
#            到了上限、每片行数比内存定的多的组，成功句前逐组写多几行：各片开几个工作进程由那一片的 crates-mutation-rows.py 照服务给的上限按内存收。行少到一片装得下就只一片。
#          · 一组分到的几片之间先按编译配置（cargo 参数在 `--` 之前的那段）归组、组内按（改的文件，行号）排，每组切成每段不超过平均每片用时（组的总用时 ÷ 组的片数）与每片行数上限（⌈组的行数 ÷ 组的片数⌉）的几段，
#            各段按用时从长到短，整段放进还放得下的片里合计用时最少的那一片（相等取行少的、再取片号小的）；放不下整段的按行数上限一片一片接着填；
#            之后最重的那一片超过平均的 1.03 倍时，与最轻那一片各拿一行对换（行数不变），换到不超过或换不动为止。
#            一片冷编的配置数、基线的批数与改的文件数因此比只按用时均分少一个量级；分片不进行键，判定与分法无关。
#          · 弄坏开关 shards-fixed-two：片数不按内存定、各组并成一组、固定分两片；shards-by-time-only：一组的几片之间只按用时均分（每行放进还没满的片里合计用时最少的那一片），不看编译配置与改的文件；
#            shard-estimate-over-all-rows：不分组，全部要现判的行一组，估值取全部行点名的记录名里最大的。
#          没有要现判的行写 0 片，不读 TASK_SERVER_SHARD_MAXIMUM。带 --only-rows 时 row-plan 带 --rows <选行文件>（mutation_shard_row_plan）只对选中的行算键、别的行只给用时（键写 -），
#          没记过用时的中位数仍取全表。各片写一份选行文件 <计划目录>/shard-<片号>.selection，plan.json 每片一项：
#          片都不钉（片里写了 pin 键服务拒，每片只写 label），各由服务照记录挑机器；各片判的记录都经 prune-server 写进同一份库。
#          不写 memory_floor、不报内存量（一片的上限由服务按记录给）。
#          row-plan 退非 0 退 1：它的标准输出写进 <计划目录>/row-plan.tsv，失败句带出它打的失败行（mutation_shard_row_plan_failure_reason），
#          一条失败行都没打写「它没打失败行」（弄坏开关 row-plan-failure-line-dropped）。
#   shard  照 TASK_SERVER_SHARD=<片号>/<片数> 跑那一片：GATE_MUTATION_ROW_SELECTION=<shard-<片号>.selection> bash <crates-mutation-replay> <树根>，退 1、3 都算这一片成了（退 0），别的照原码退；
#          各片判的记录都写进 prune-server 里同一份库（库根照被判那棵树取，两台同名）；起了门禁的那一片把它判的选行文件拷进 <产物目录>/judged-rows，合并照它核；
#          这一片本次未跑的几条由门禁追加进 <产物目录>/not-run-record（起门禁时交 GATE_MUTATION_SHARD_NOT_RUN_FILE，crates-mutation-replay.sh 文件头「分片」；每条一行
#          「<门禁文件名> 本次未跑：<表>:<行号> <变异名>：…」，点名的测试没跑到被改坏的那一处，库里这把键下没有「抓到」的记录、row-plan 第三列是 0，合并从读不到里分出它们靠这份文件），没有本次未跑的那一片没有这份文件；
#          每一片不看派到哪一台、在跑它的这一台现查 llvm 工具（记覆盖要的 llvm-profdata、llvm-cov，--version 的 LLVM 版本要以这一台 rustc -vV 的「LLVM version」起头；
#          目录取 GATE_MUTATION_LLVM_TOOLS，没设取 rustc --print sysroot 下 lib/rustlib/<host>/bin；缺了退 78、一条变异都不起，出路写 rustup component add llvm-tools），
#          再在树根跑 python3 scripts/hosts/rocksdb-prebuilt.py entry --root .（RocksDB 预编库的取值，判红退 78、一条变异都不起）；两台同一条规则；
#          变异复跑带崩溃判定库的测试各对它自己起的临时 prune-server 跑（scripts/mutation/crates-mutation-rows.py 文件头「变异复跑的崩溃判定库」），这里不设库的变量。
#   merge  照 TASK_SERVER_SHARD_OUTPUTS 读各片的 judged-rows，所有片判的行并成一份（去重），经 crates-mutation-rows.py row-plan --rows <并起来的行> --progress
#          按本机此刻的源码算行键、查服务（第三列 1 是读得到），再逐片数这一片判的行里读到几行：
#          读不到的行里，这一片自己在 not-run-record 里报了本次未跑的先分出来（一行属哪一档只看那一片自己报的，不解析日志）：另起「  · 本次未跑：N 条（…这一趟不是整张表的结论）」一行，
#          之后一条一行（第几片、台、行号、变异名、那一片打的原因），不算读不到、不让整道判红；报的记录读不出行号、或点名的行不在这一片判的行里，那几行不当本次未跑，退 1、点名那一片与头一条记录行；
#          有一片判的行（分出本次未跑的之后）一行都读不到退 1、点名那一片，出路先写它写进了别的库（连的不是同一份服务），再写比两台的源码（判的不是同一棵树：被改坏的单元与点名测试的闭包里的源码不同，行键就不同；
#          行键里没有编译器、链接器、cargo 配置与版本这类构建环境，不用比）；row-plan 退非 0 退 1；读不到的另几行逐行列出，由那一道接着的核账报；
#          成功句报各片判的行里读得到几行、读不到几行；有本次未跑的改报抓到几行、本次未跑几行、读不到几行，并写这一趟不是整张表的结论。不拷库目录、不 import：两台写的是服务里同一份库。
#          分步打进度（一步慢到看不出停在哪一步，服务按日志不涨判卡住）：每一步开始、结束各一行「  · 合并·<步名>：开始」「  · 合并·<步名>：完，用了 N 秒，…」（打 stdout 的是这一份的三步，
#          row-plan 与核账的在 stderr），按次序是：
#          · 读计划（这一份）：哪几片交了 judged-rows、各判了几行、并起来去重几行；
#          · 核锚点、现算行键、读库、读用时表（row-plan --progress，crates-mutation-rows.py 文件头「合并那一步的分步进度」）：现算行键里「点名测试的闭包」算完另打一行，
#            逐行做的步每过 GATE_MUTATION_PROGRESS_SECONDS 秒（没设 60）打一行「TASK_SERVER_HEARTBEAT 合并·<步名>：在第 N 行、共 M 行，…」，这种行不算日志在涨，真卡死照样判得出；
#          · 逐片核对（这一份）：每一片一行「第 i 片（台）判的 N 行，读到 R 行、读不到 U 行」；
#          · 核账（crates-mutation-replay 在这一步里调 run_stage，范围「合并·核账」）：开始、各逐行步、完（带退出码），在这一份返回之后打。
#          弄坏开关 merge-steps-silent 下这一份不打自己的步，row-plan 也不带 --progress。
# 真跑那一步（shard）跑一部分 crates/mutations.tsv，起 cargo，与 crates-mutation-replay 同一类，经统一任务服务起；--selftest 的登记档在 .claude/hooks/selftest-registry.tsv，改了就跑（同一份判定）。
#
# 弄坏开关 MUTATION_SHARD_RUN_BREAK=<项>（只给证红用，逗号分隔可给几个；打开时自证退 1）。只在 --selftest 起的进程里生效：自证在它的临时目录里放一份标记文件，
#   路径经 MUTATION_SHARD_RUN_SELFTEST_MARK 带给它起的驱动，文件在、内容是 MUTATION_SHARD_SELFTEST_MARK_CONTENT 那一行才认开关；别处设了它，驱动拒、退 2，什么都没分。
#   逐项写开关名、弄坏哪一处、打开后自证哪几格红
# （项名是自证 expect 的格名，一行一项；失败行是「✗ <格名>：<细节>」，括号里抄细节的句首，随跑变的路径、进程号、指纹写成 …）：
#   MUTATION_SHARD_RUN_BREAK=only-rows-ignored  驱动不看 --only-rows，照整张表分：点名的一行连同别的行都分出去
#       ㉒ 计划那一步带 --only-rows 点名一行：row-plan 只对那一行算键（算键 1 行，不是整张表的 6 行），计划只分那一行（「✗ ㉒ 计划那一步带 --only-rows 点名一行：」；row-plan 那一行写的是算了 6 行的键）
#       ⑱ ③ 分行带选行文件：只在选中的行里分，选中的行里要现判的进 pending.rows、复用的不进（「✗ ⑱ ③ 分行带选行文件：」；pending.rows 里有选行文件之外的行）
#       ㉗ 点名 N 行：计划的各片加留给合并之后的行恰好是这 N 行（「✗ ㉗ 点名 N 行：」；pending.rows 与 after-merge.rows 合起来多出点名之外的行）
#   MUTATION_SHARD_RUN_BREAK=reused-rows-unlisted  ③ 分行不把复用的行写进 after-merge.rows，也不核点名的行有没有着落：复用的行两边都不在、计划照写
#       ㉗ 点名 N 行：计划的各片加留给合并之后的行恰好是这 N 行（「✗ ㉗ 点名 N 行：」；pending.rows 与 after-merge.rows 合起来少了复用的那几行）
#   MUTATION_SHARD_RUN_BREAK=shards-fixed-two  ④ 片数不按内存定、各组并成一组、固定分两片（要现判的只有 1 行时一片）
#       ㉘ 按内存定片数：要现判 52 行、每个工作进程 3.8 GiB 时各片预计内存都在安全额度（50G × 2/3）内且不止两片，要现判 3 行只一片，片数上限 4 时恰好 4 片，各片合起来恰好是要现判的行
#          （「✗ ㉘ 按内存定片数：」；要现判 52 行那一趟写 2 片、每片 26 行，预计内存超了安全额度；3 行那一趟写 2 片；片数上限 4 那一趟写 2 片）
#       ㉜ 按片估内存：轻的 20 行（2 GiB）与整个包跑的重的 2 行（28 GiB）一起要现判……（「✗ ㉜ 按片估内存：」；两趟都只写 2 片，重的两行所在的片各有十来行，不是各单独一片）
#       ㉛ 按编译配置与改的文件分片：128 行……（「✗ ㉛ 按编译配置与改的文件分片：」；mutation_shard_assign 照开关固定写 2 片，片数与每片行数上限不同只按用时均分的那一份）
#   MUTATION_SHARD_RUN_BREAK=shard-estimate-over-all-rows  ④ 不分组：全部要现判的行一组，每片至多几行照全部行点名的记录名里最大的那个估值定
#       ㉜ 按片估内存：轻的 20 行（2 GiB）与整个包跑的重的 2 行（28 GiB）一起要现判……（「✗ ㉜ 按片估内存：」；片数上限 8 那一趟写了 8 片、不是 4 片，轻的那一组的估值不是只取轻的那几行的 2.0 GiB，上限 3 那一趟重的行与轻的行同在一片）
#   MUTATION_SHARD_RUN_BREAK=shards-by-time-only  ④ 各片只按用时均分，不看编译配置与改的文件
#       ㉛ 按编译配置与改的文件分片：128 行（8 个编译配置各 16 行，其中一个配置的行用时是别的十倍）每片至多 32 行时，各片合起来恰好是这 128 行、不相交、片数与每片行数上限同只按用时均分的一样，
#          各片合起来的不同编译配置数不到只按用时均分的四分之三、最重的一片不超过只按用时均分那一份最重的 1.1 倍（「✗ ㉛ 按编译配置与改的文件分片：」；各片合起来的编译配置数同只按用时均分的一样多）
#   MUTATION_SHARD_RUN_BREAK=row-plan-whole-table  驱动调 row-plan 不带 --rows，照整张表算键：点名一行也付整张表算键的代价
#       ㉒ 计划那一步带 --only-rows 点名一行：row-plan 只对那一行算键（算键 1 行，不是整张表的 6 行），计划只分那一行（「✗ ㉒ 计划那一步带 --only-rows 点名一行：」；row-plan 那一行写的是算了 6 行的键）
#   MUTATION_SHARD_RUN_BREAK=row-plan-selection-ignored  自证里直接起 row-plan 点名两行那一趟带 GATE_MUTATION_BREAK=rowplanselectionignored（crates-mutation-rows.py 的 row-plan 不看 --rows）
#       ㉒ row-plan 点名两行：只对两行算键（算键 2 行），别的 4 行只给用时、键写 -；不点名算整张 6 行（「✗ ㉒ row-plan 点名两行：」；点名两行那一趟写的是算了 6 行的键）
#   MUTATION_SHARD_RUN_BREAK=row-plan-failure-line-dropped  计划那一步 row-plan 退非 0 时失败句不带出它打的失败行（也不写「它没打失败行」）
#       ㉝ 计划那一步的 row-plan 退非 0：失败句带出它打的失败行，一条都没打时写它没打失败行（「✗ ㉝ 计划那一步的 row-plan 退非 0：」；打了一条失败行的那一趟，失败句里没有那一行的字）
#   MUTATION_SHARD_RUN_BREAK=anchor-broken-dispatched  ③ 分行不看 row-plan 的第五列：锚点失效、而行键算得出的那一行照样分出去
#       ⑭ ③ 分行：锚点失效的行（第五列 anchor-broken，有键没键都算）不进各片、单报几行（「✗ ⑭ ③ 分行：锚点失效的行（第五列 anchor-broken，有键没键都算）不进各片、单报几行：pending.rows「1 2 4 」」）
#   MUTATION_SHARD_RUN_BREAK=pile-pinned-local  计划给第 0 片写 pin=local：钉在本机
#       ㉔ 经 task-server 分片：计划写两片（两片都不钉、片里没有 pin 键，都不带内存下限），两片合起来是全部 6 行、不相交（「✗ ㉔ 经 task-server 分片：计划写两片」；plan.json：pin=local:False unpinned:False）
#   MUTATION_SHARD_RUN_BREAK=break-honored-anywhere  弄坏开关在 --selftest 之外也认
#       ⑨ MUTATION_SHARD_RUN_BREAK 设在 --selftest 之外（不带自证的标记）：驱动拒、退 2，说设在 --selftest 之外（「退 2；输出：」；输出里没有「设在 --selftest 之外」）
#   MUTATION_SHARD_RUN_BREAK=llvm-tools-missing-skipped  一片在这一台查出缺 llvm 工具时这一堆不判、退 0（不退 78）
#       ㉖ 每一片在跑它的这一台现查 llvm 工具：缺了退 78、一条变异都不起，出路写装哪个包；两片同一条规则（「✗ ㉖ 每一片在跑它的这一台现查 llvm 工具：」；两片退「0 0 」）
#   MUTATION_SHARD_RUN_BREAK=merge-rows-unchecked  合并不查服务，judged-rows 里的行一律当读得到
#       ㉕ 合并：另一片判的行按此刻的行键在服务里一行都读不到（写进了别的库或两台源码不同）⇒ 退 1、点名那一片，出路先写查它写进了哪份库、再写比两台的源码（「✗ ㉕ 合并：」；合并退 0）
#       ㉟ 合并认一片报的本次未跑：两片都起门禁、第 9 行点名的测试没跑到被改坏的那一处，合并退 0、另起「本次未跑：1 条」逐条列（「✗ ㉟ 合并认一片报的本次未跑：」；第 9 行被当读得到，合并输出里没有「本次未跑：1 条」）
#       ㊱ 合并：一片没报本次未跑而判的行读不到，退 1、点名那一片（「✗ ㊱ 合并：」；没报的那一片合并退 0）
#   MUTATION_SHARD_RUN_BREAK=not-run-counted-unreadable  合并不读各片的 not-run-record，本次未跑的行照读不到算
#       ㉟ 合并认一片报的本次未跑：两片都起门禁、第 9 行点名的测试没跑到被改坏的那一处，合并退 0、另起「本次未跑：1 条」逐条列（「✗ ㉟ 合并认一片报的本次未跑：」；只判了那一行的一片合并退 1，输出里没有「本次未跑：1 条」）
#       ㊱ 合并：报了本次未跑而记录读不出行号的一片也退 1（「✗ ㊱ 合并：」；那一片的输出里没有「读不出行号」，记录没读）
#   MUTATION_SHARD_RUN_BREAK=not-run-not-handed-over  各片起门禁不交 GATE_MUTATION_SHARD_NOT_RUN_FILE：本次未跑的几条到不了产物目录
#       ㉟ 合并认一片报的本次未跑：（「✗ ㉟ 合并认一片报的本次未跑：」；产物目录里没有 not-run-record、合并输出里没有「本次未跑：1 条」）
#   MUTATION_SHARD_RUN_BREAK=unreported-rows-as-not-run  合并把没报本次未跑而读不到的行也当本次未跑（那一片自己报了的行照旧带它打的原因）
#       ㉕ 合并：另一片判的行按此刻的行键在服务里一行都读不到（写进了别的库或两台源码不同）⇒ 退 1、点名那一片，出路先写查它写进了哪份库、再写比两台的源码（「✗ ㉕ 合并：」；那一片没报本次未跑，合并退 0）
#       ㊱ 合并：一片没报本次未跑而判的行读不到，退 1、点名那一片（「✗ ㊱ 合并：」；没报的那一片合并退 0、输出里写本次未跑：1 条）
#   MUTATION_SHARD_RUN_BREAK=merge-steps-silent  合并不打分步行：这一份不打读计划、逐片核对的行，row-plan 不带 --progress（核账的行由 crates-mutation-rows.py 打，不在这一份手里）
#       ㉙ 合并分步输出：读计划、现算行键、读库、逐片核对、核账各步的开始、结束行按次序都在，逐片的数对（「✗ ㉙ 合并分步输出：」；输出里按次序找不到读计划的开始行）
#       ㉚ 合并里一步慢时有心跳行：逐行的步每过一行停 0.3 秒、间隔 1 秒时，现算行键与读库有带心跳前缀的「在第 N 行、共 M 行」（「✗ ㉚ 合并里一步慢时有心跳行：」；合并·现算行键的心跳行 0 条）
#   MUTATION_SHARD_RUN_BREAK=selftest-tmpdir-outside  自证不把 TMPDIR 换成自己临时目录下的 tmp/，场景照外面那一层（自证在调用方的 TMPDIR 下另建的）建临时目录
#       ⑫ 场景都跑在自证自己的 tmp/ 下……跑完外面的临时目录……里没多出 singlefs-mutation-replay-*，自己的 tmp/ 下也一个不剩（「✗ ⑫ 场景都跑在自证自己的 tmp/ 下：」；
#          「工作目录不在自证自己的 tmp/ 下的」：本机那一侧的假 cargo 在外面那一层的 singlefs-mutation-replay-* 里起）
#   MUTATION_SHARD_RUN_BREAK=selftest-parallel-exit-dropped  自证那一层认：三个场景那三串同时跑完之后，steps 那一串的退出码文件删掉（像那一串没收回）
#       自证并行（scenes）：派出去的件都收回了退出码（「✗ 自证并行（scenes）：派出去的件都收回了退出码：」；派出去 3 件、收回 2 件）
#
# admission: always 每次调都分这一刻这棵树上没有作数记录的行；复用判定在判定库记录里，不在这里
# run-condition: command python3 rsync ssh git cargo rustc
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
mutation_shard_script_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

mutation_shard_break_is() { [[ ",${MUTATION_SHARD_RUN_BREAK:-}," == *",$1,"* ]]; }
# clip_text <字节数>：标准输入每行取前 <字节数> 个字节，再丢掉被切在中间的半个字符，输出总是合法的 UTF-8（GNU 的 cut -c 与 cut -b 都按字节切，三个字节的汉字会被切断；给人看的失败行与日志摘录用它截）
clip_text() { cut -b "1-$1" | iconv -c -f UTF-8 -t UTF-8 2>/dev/null; }

# 按内存定片数的常量（文件头「三步」plan 的 ④）
# 安全额度 = 各台可用上限里最小的 × 2/3：服务撞上限每次把上限乘 1.5、乘完到那台总上限就打回（task-server/task_server_lib.py 的 grown_cap 与 retry_decision），
#   一片能拿到的最高一档落在（总上限 ÷ 1.5，总上限）之间；预计内存在总上限的 2/3 以内，从哪一档放大起都给得到
MUTATION_SHARD_MEMORY_SAFETY_NUMERATOR=2
MUTATION_SHARD_MEMORY_SAFETY_DENOMINATOR=3
# 起一片的父进程（crates-mutation-replay 的 bash 与 crates-mutation-rows.py 那一层）：任务日志里的底座读数（MUTATION_BASE_READING 的 parent_bytes）都不到 1 GiB
MUTATION_SHARD_PARENT_BYTES=$((1 << 30))
# 没有实测的包每个工作进程按这么多算：共享根运行记录 worker-peaks/ 里编译并行度 10 及以下最重的包（singlefs-verification）约 5.2 GiB，向上取到 6 GiB
MUTATION_SHARD_UNMEASURED_WORKER_BYTES=$((6 << 30))
# 变异表在被判那棵树里的位置（同 crates-mutation-rows.py 的 TABLE）：分片按它读每一行的编译配置与改的文件
MUTATION_SHARD_TABLE="crates/mutations.tsv"

# 自证起的进程的标记（文件头「弄坏开关」那一段）
MUTATION_SHARD_SELFTEST_MARK_CONTENT="mutation-shard-run.sh --selftest 起的进程"
mutation_shard_inside_selftest() {
  [[ -n "${MUTATION_SHARD_RUN_SELFTEST_MARK:-}" && -f "$MUTATION_SHARD_RUN_SELFTEST_MARK" ]] \
    && [[ "$(head -n 1 -- "$MUTATION_SHARD_RUN_SELFTEST_MARK")" == "$MUTATION_SHARD_SELFTEST_MARK_CONTENT" ]]
}

mutation_shard_llvm_check_command() { # 打在跑这一片的这一台查 llvm 工具的那一段：llvm-profdata、llvm-cov 在、llvm-cov 的 LLVM 版本以这一台 rustc 的起头（文件头「三步」shard 那一步）；查得过退 0 打一句版本，查不过退 3 打为什么
  printf 'directory=%q\n' "${GATE_MUTATION_LLVM_TOOLS:-}"
  cat <<'LLVM_CHECK'
if [[ -z "$directory" ]]; then directory="$(rustc --print sysroot)/lib/rustlib/$(rustc -vV | sed -n 's/^host: //p')/bin"; fi
for tool in llvm-profdata llvm-cov; do [[ -x "$directory/$tool" ]] || { echo "$tool 不在或不能执行"; exit 3; }; done
wanted="$(rustc -vV | sed -n 's/^LLVM version: //p')"
[[ -n "$wanted" ]] || { echo "这一台 rustc -vV 读不出 LLVM 版本"; exit 3; }
found="$("$directory/llvm-cov" --version | sed -n 's/.*LLVM version \([^ ]*\).*/\1/p' | head -n 1)"
case "$found" in
  "$wanted"|"$wanted"-*) echo "llvm-cov 的 LLVM $found，rustc 的 $wanted" ;;
  *) echo "llvm-cov 的 LLVM 版本「${found:-读不出}」与这一台 rustc 的「${wanted:-读不出}」对不上"; exit 3 ;;
esac
LLVM_CHECK
}

mutation_shard_row_plan() { # <crates-mutation-rows.py> <树根> <输出文件> [<选行文件>]：③ 列每一行的行键、这一趟复用不复用、上一次的用时，写进输出文件；
  # 带选行文件时只对选中的行算键（row-plan --rows）：点名一行不付整张表算键的代价，别的行只给用时（分行取没记过用时的中位数仍取全表）
  local selection_arguments=()
  if [[ -n "${4:-}" ]]; then
    if mutation_shard_break_is row-plan-whole-table || mutation_shard_break_is only-rows-ignored; then
      echo "  ! 弄坏开关 MUTATION_SHARD_RUN_BREAK=row-plan-whole-table（或 only-rows-ignored）：row-plan 不带 --rows，整张表算键（改前的样子）" >&2
    else
      selection_arguments=(--rows "$4")
    fi
  fi
  python3 "$1" row-plan "$2" ${selection_arguments[@]+"${selection_arguments[@]}"} > "$3"
}

mutation_shard_row_plan_failure_reason() { # <row-plan 的输出文件>：打接在「row-plan 退非 0」后面的一句。row-plan 的失败行（「  ✗ …」）打在标准输出、写进了输出文件，不在日志里：带出头一条，一条都没打写「它没打失败行」
  # 弄坏开关 row-plan-failure-line-dropped：什么都不打，失败句里看不到 row-plan 自己的原因
  mutation_shard_break_is row-plan-failure-line-dropped && return 0
  # 有几条取头一条（row-plan 先打的是它自己的原因，后面是出路）
  local failure_line
  failure_line="$(grep -m 1 '✗' -- "$1" 2>/dev/null | sed 's/^[[:space:]]*//' | clip_text 400)"
  if [[ -n "$failure_line" ]]; then
    printf '，它打的失败行：%s' "$failure_line"
  else
    printf '，它没打失败行'
  fi
}

mutation_shard_split_plan() { # <row-plan 的输出> <要现判的行文件> <留给合并之后的行文件> <键文件> [<只分这几行的选行文件>]：
  # ③ 分行：写要现判的行文件（pending.rows）、留给合并之后的行文件与键文件，stdout 打一行汇总
  # 汇总：<表里的行数> <要现判的> <记过用时的> <没记过的按几秒> <拒算的> <锚点失效的> <复用的> <没着落的行（逗号隔开，没有写 -）>
  # 要现判 = 不复用、有键、第五列不是 anchor-broken（锚点失效的行这一趟不跑，crates-mutation-rows.py 文件头「锚点失效」），一行「<行号>\t<秒>」，按行号排；
  # 留给合并之后的 = 锚点失效、拒算（没键）、复用（第三列 1：库里有键相同的判定）的行，一行「<行号>\t锚点失效|拒算|复用」，合并之后那一道的核账判它们；
  # 没着落 = 点名的行（带选行文件时是选行文件里的，不带时是 row-plan 列的每一行）不在要现判的里、也不在留给合并之后的行里，由调用方整步判红
  local anchor_dispatched="" only_rows_file="${5:-}" reused_unlisted=""
  mutation_shard_break_is anchor-broken-dispatched && anchor_dispatched=1
  mutation_shard_break_is only-rows-ignored && only_rows_file=""   # 弄坏开关 only-rows-ignored：不看选行文件，照整张表分
  mutation_shard_break_is reused-rows-unlisted && reused_unlisted=1
  MUTATION_SHARD_ANCHOR_DISPATCHED="$anchor_dispatched" MUTATION_SHARD_ONLY_ROWS_FILE="$only_rows_file" MUTATION_SHARD_REUSED_UNLISTED="$reused_unlisted" \
    python3 - "${@:1:4}" <<'PY_SPLIT'
import os, statistics, sys
plan_path, pending_path, after_merge_path, keys_path = sys.argv[1:]
only_rows_path = os.environ.get("MUTATION_SHARD_ONLY_ROWS_FILE", "")   # 只分这几行（--only-rows）：空是整张表
anchor_dispatched = os.environ.get("MUTATION_SHARD_ANCHOR_DISPATCHED") == "1"   # 弄坏开关 anchor-broken-dispatched：不看第五列
reused_unlisted = os.environ.get("MUTATION_SHARD_REUSED_UNLISTED") == "1"   # 弄坏开关 reused-rows-unlisted：复用的行不写进留给合并之后的行、不核没着落的行
with open(plan_path, encoding="utf-8") as handle:
    rows = [line.rstrip("\n").split("\t") for line in handle if line.strip()]
with open(keys_path, "w", encoding="utf-8") as handle:
    handle.write("".join(row[1] + "\n" for row in rows))
def anchor_broken(row):
    return not anchor_dispatched and len(row) > 4 and row[4] == "anchor-broken"
known = [float(row[3]) for row in rows if row[3] != "-"]
default = statistics.median(known) if known else 1.0
named = {int(row[0]) for row in rows}
if only_rows_path:
    with open(only_rows_path, encoding="utf-8") as handle:
        named = {int(text) for text in (line.strip() for line in handle) if text.isdigit()}
    rows = [row for row in rows if int(row[0]) in named]
pending = sorted((int(row[0]), float(row[3]) if row[3] != "-" else default) for row in rows if row[2] == "0" and row[1] != "-" and not anchor_broken(row))
with open(pending_path, "w", encoding="utf-8") as handle:
    handle.write("".join(f"{line_number}\t{seconds:.3f}\n" for line_number, seconds in pending))
after_merge = []
for row in rows:
    if anchor_broken(row):
        after_merge.append((int(row[0]), "锚点失效"))
    elif row[1] == "-":
        after_merge.append((int(row[0]), "拒算"))
    elif row[2] == "1" and not reused_unlisted:
        after_merge.append((int(row[0]), "复用"))
with open(after_merge_path, "w", encoding="utf-8") as handle:
    handle.write("".join(f"{line_number}\t{why}\n" for line_number, why in sorted(after_merge)))
placed = {line_number for line_number, _seconds in pending} | {line_number for line_number, _why in after_merge}
unaccounted = [] if reused_unlisted else sorted(named - placed)
broken = sum(1 for row in rows if anchor_broken(row))
refused = sum(1 for row in rows if row[1] == "-" and not anchor_broken(row))
reused = sum(1 for row in rows if row[1] != "-" and row[2] == "1" and not anchor_broken(row))
print(len(rows), len(pending), len(known), f"{default:.1f}", refused, broken, reused, ",".join(str(line_number) for line_number in unaccounted) or "-")
PY_SPLIT
}

mutation_shard_memory_groups() { # <crates-mutation-rows.py> <树根> <要现判的行文件> <分组文件>：④ 按内存把要现判的行分组、每组每片至多几行（文件头「三步」plan 的 ④）；
  # 分组文件一行一个要现判的行「<行号>\t<组名>\t<每片至多几行>\t<一片满了开几个工作进程>」；stdout 头一行是安全额度与线程数，之后每组一行「<组名>\t<行数>\t<怎么算的>」；
  # 定不了（变异表里没有要现判的那一行、运行记录目录取不到、线程数取不到、多机配置没写或写坏 SLICE_MEMORY_CAP_BY_HOST）stderr 写一句原因、退 1。
  # 弄坏开关 shard-estimate-over-all-rows：不分组，全部要现判的行一组，估值取全部行点名的记录名里最大的
  MUTATION_SHARD_MEMORY_SAFETY_NUMERATOR="$MUTATION_SHARD_MEMORY_SAFETY_NUMERATOR" MUTATION_SHARD_MEMORY_SAFETY_DENOMINATOR="$MUTATION_SHARD_MEMORY_SAFETY_DENOMINATOR" \
  MUTATION_SHARD_PARENT_BYTES="$MUTATION_SHARD_PARENT_BYTES" MUTATION_SHARD_UNMEASURED_WORKER_BYTES="$MUTATION_SHARD_UNMEASURED_WORKER_BYTES" \
  MUTATION_SHARD_ESTIMATE_OVER_ALL_ROWS="$(mutation_shard_break_is shard-estimate-over-all-rows && echo 1)" \
    python3 - "$@" <<'PY_ROWS_PER_SHARD'
import importlib.util, os, sys
rows_module_path, tree_root, pending_path, groups_path = sys.argv[1:]
environment = dict(os.environ)
specification = importlib.util.spec_from_file_location("crates_mutation_rows_for_shard_plan", rows_module_path)
rows_module = importlib.util.module_from_spec(specification)
specification.loader.exec_module(rows_module)
import task_memory   # crates-mutation-rows.py 载入时已把 scripts/hosts 放进 sys.path

def refuse(reason):
    print(reason, file=sys.stderr)
    sys.exit(1)

numerator, denominator = int(environment["MUTATION_SHARD_MEMORY_SAFETY_NUMERATOR"]), int(environment["MUTATION_SHARD_MEMORY_SAFETY_DENOMINATOR"])
parent_bytes, unmeasured_bytes = int(environment["MUTATION_SHARD_PARENT_BYTES"]), int(environment["MUTATION_SHARD_UNMEASURED_WORKER_BYTES"])
size = rows_module.format_size
with open(pending_path, encoding="utf-8") as handle:
    pending = [int(line.split("\t", 1)[0]) for line in handle if line.strip()]
table, _malformed = rows_module.read_table(os.path.join(tree_root, rows_module.TABLE), environment)
by_line = {row.line_number: row for row in table}
missing = [line_number for line_number in pending if line_number not in by_line]
if missing:
    refuse(f"要现判的第 {'、'.join(map(str, missing))} 行在 {rows_module.TABLE} 里不是成形的一行（row-plan 之后表变了），读不出它点名的包")
try:
    records = rows_module.run_records_directory(tree_root, environment)
except rows_module.admission_module().InputManifestError as error:
    refuse(f"运行记录目录取不到（{error}），读不了每个工作进程的峰值")
peaks, peaks_text = rows_module.measured_worker_peaks(records, environment)
try:
    bound_name, threads, _bound_text = rows_module.worker_threads_bound(environment)
    thread_budget = rows_module.compile_thread_budget(environment)
except rows_module.HeavyThreadDefaultError as error:
    refuse(f"取不到每台默认重活线程数（{error}；{error.howto}）")
except ValueError as error:
    refuse(f"GATE_MUTATION_WORKERS 写的不是整数（{error}）")
if threads < 1:
    refuse(f"GATE_MUTATION_WORKERS 写成「{threads}」，不是正整数")
threads_text = f"每台默认重活线程数 {threads}" if bound_name == "每台默认重活线程数" else f"{bound_name} 给 {threads} 个"
configuration = task_memory.test_runs_multi_host_configuration(environment)[0]
try:
    by_host_text = task_memory.admission_module().configuration_value(configuration, task_memory.SLICE_MEMORY_CAP_KEY)
except Exception as error:   # InputManifestError：文件在而读不了
    refuse(f"多机配置 {configuration} 的 {task_memory.SLICE_MEMORY_CAP_KEY} 读不了（{error}）")
caps = []
for item in by_host_text.split():
    address, _separator, value = item.partition("=")
    item_bytes = task_memory.bytes_of_size(value)
    if item_bytes is None:
        refuse(f"多机配置 {configuration} 的 {task_memory.SLICE_MEMORY_CAP_KEY} 里 {address} 那一项写成「{value}」，不是正整数加 K / M / G / T")
    caps.append((item_bytes, address, value))
if not caps:
    refuse(f"多机配置 {configuration} 没写 {task_memory.SLICE_MEMORY_CAP_KEY}，定不了每片能用多少内存")
smallest, smallest_address, smallest_value = min(caps)
budget = smallest * numerator // denominator

def estimate(keys, workers):
    """点名 keys 这几个记录名的行，开 workers 个工作进程的一片预计要多少内存：(字节, 怎么算的)。"""
    jobs = rows_module.cargo_jobs_for(environment, workers, thread_budget)
    peak, basis = rows_module.worker_peak_for_jobs(peaks, keys, jobs, unmeasured_bytes, environment)
    unmeasured = rows_module.unmeasured_packages_at(peaks, keys, jobs)
    if unmeasured and peak == unmeasured_bytes:
        basis = (f"包 {'、'.join(map(rows_module.peak_key_label, unmeasured))} 在编译并行度 {jobs} 与更高的都没有实测，按保守常量 {size(unmeasured_bytes)} 算"
                 + ("" if peaks is not None else f"（{peaks_text}）"))
    needed = workers * peak + parent_bytes
    return needed, f"开 {workers} 个工作进程、编译并行度 {jobs}，每个工作进程的峰值 {size(peak)}（{basis}），{workers} × {size(peak)} + 父进程 {size(parent_bytes)} = {size(needed)}"

def limit_for(keys):
    """只点名 keys 这几个记录名的行每片至多几行：行数从 1 往上试，每一个都满足「预计内存 ≤ 安全额度」的最大行数；工作进程数到了线程数之后再多的行也开这么多个，
    记成要现判的行数；开一个工作进程就超的按每片 1 行。交 (每片至多几行, 怎么算的)。"""
    limit, fitting_text, over_text = 0, "", ""
    for count in range(1, len(pending) + 1):
        needed, text = estimate(keys, min(count, threads))
        if needed > budget:
            over_text = f"；{count} 行要{text}，超了"
            break
        limit, fitting_text = count, text
        if count >= threads:
            limit = len(pending)
            break
    if limit == 0:
        return 1, f"一个工作进程就超了安全额度（{estimate(keys, 1)[1]}），按每片 1 行"
    return limit, f"{limit} 行{fitting_text}{over_text}"

# 每一行点名的记录名（worker_peak_keys 只喂这一行）与种类：恰好点名一个测试目标的（worker_peak_target_key 交得出）是「测试目标」，别的（整个包跑、几个包、没点名目标）是「整包」。
#   同种类、同一组记录名的行每片至多几行相同；整包那几行估值取整个包的最大，不与测试目标的行进同一组
by_keys = {}
if environment.get("MUTATION_SHARD_ESTIMATE_OVER_ALL_ROWS") == "1":
    by_keys[("全部", rows_module.worker_peak_keys([by_line[line_number].arguments for line_number in pending]))] = list(pending)
else:
    for line_number in pending:
        arguments = by_line[line_number].arguments
        kind = "测试目标" if rows_module.worker_peak_target_key(arguments) is not None else "整包"
        by_keys.setdefault((kind, rows_module.worker_peak_keys([arguments])), []).append(line_number)
by_limit = {}
for (kind, keys), lines in sorted(by_keys.items()):
    limit, text = limit_for(keys)
    entry = by_limit.setdefault((kind, limit), {"lines": [], "keys": [], "text": text})
    entry["lines"] += lines
    entry["keys"].append(keys)
# 同种类里按每片至多几行从少到多，相邻的一组并进前一组（每片至多几行取前一组的）不多出片时就并：几行一组的零头不各占一片
def shards_needed(line_count, limit):
    return -(-line_count // limit)
groups = []
for kind, limit in sorted(by_limit, key=lambda item: (item[0], item[1])):
    entry = by_limit[(kind, limit)]
    last = groups[-1] if groups and groups[-1]["kind"] == kind else None
    if last is not None and (shards_needed(len(last["lines"]) + len(entry["lines"]), last["limit"])
                             <= shards_needed(len(last["lines"]), last["limit"]) + shards_needed(len(entry["lines"]), limit)):
        last["lines"] += entry["lines"]
        last["keys"] += entry["keys"]
        continue
    groups.append({"kind": kind, "limit": limit, "lines": list(entry["lines"]), "keys": list(entry["keys"]), "text": entry["text"]})
groups.sort(key=lambda group: (group["limit"], group["kind"]))
with open(groups_path, "w", encoding="utf-8") as handle:
    handle.write("".join(f"{line_number}\t{group['kind']}·每片至多{group['limit']}行\t{group['limit']}\t{min(group['limit'], threads)}\n"
                         for group in groups for line_number in sorted(group["lines"])))
print(f"安全额度 {size(budget)}（多机配置 {task_memory.SLICE_MEMORY_CAP_KEY} 各台里最小的 {smallest_address}={smallest_value} × {numerator}/{denominator}）；{threads_text}；"
      f"分 {len(groups)} 组，每组只按组里那几行点名的记录名估")
for group in groups:
    governing = "、".join(map(rows_module.peak_key_label, group["keys"][0]))
    print(f"{group['kind']}·每片至多{group['limit']}行\t{len(group['lines'])}\t"
          f"{len(group['keys'])} 种记录名，每片至多 {group['limit']} 行照其中最紧的 {governing}：{group['text']}")
PY_ROWS_PER_SHARD
}

mutation_shard_assign() { # <要现判的行文件> <分组文件> <片数上限> <计划目录> <变异表>：④ 分片，各片写 <计划目录>/shard-<片号>.selection（文件头「三步」plan 的 ④）；
  # <分组文件> 是 mutation_shard_memory_groups 写的（一行「<行号>\t<组名>\t<每片至多几行>\t<一片满了开几个工作进程>」），每个要现判的行都要在里面；一片只放同一组的行。
  # stdout 头一行「<片数> <各组每片的行数上限（按组的片数均摊）里最大的> <各片行数，逗号隔开> <各片约几秒，逗号隔开>」，之后每个片数到了上限、每片行数比内存定的多的组一行
  # 「over\t<组名>\t<每片行数>\t<内存定的每片至多几行>」。<变异表> 是被判那棵树的 crates/mutations.tsv：按行号读每一行的编译配置（cargo 参数在 `--` 之前的）与改的文件。
  # 弄坏开关 shards-fixed-two：片数固定两片（各组并成一组）；shards-by-time-only：各片只按用时均分、不看编译配置与改的文件。MUTATION_SHARD_ASSIGN_REFERENCE=1 只给自证（㉛）起这份参照用，不看开关就走只按用时均分那一支
  MUTATION_SHARD_FIXED_TWO="$(mutation_shard_break_is shards-fixed-two && echo 1)" \
  MUTATION_SHARD_BY_TIME_ONLY="$({ mutation_shard_break_is shards-by-time-only || [[ "${MUTATION_SHARD_ASSIGN_REFERENCE:-}" == 1 ]]; } && echo 1)" python3 - "$@" <<'PY_ASSIGN'
import os, sys
pending_path, groups_path, maximum_text, directory, table_path = sys.argv[1:]
with open(pending_path, encoding="utf-8") as handle:
    pending = [(int(line_number), float(seconds)) for line_number, seconds in (line.rstrip("\n").split("\t") for line in handle if line.strip())]
seconds_of = dict(pending)
maximum = int(maximum_text)
groups = {}   # 组名 → [每片至多几行, 一片满了开几个工作进程, [行号]]
with open(groups_path, encoding="utf-8") as handle:
    for line in handle:
        if line.strip():
            line_number, name, limit_text, workers_text = line.rstrip("\n").split("\t")
            groups.setdefault(name, [int(limit_text), int(workers_text), []])[2].append(int(line_number))
ungrouped = sorted(set(seconds_of) - {line_number for group in groups.values() for line_number in group[2]})
if ungrouped:
    sys.exit(f"要现判的第 {'、'.join(map(str, ungrouped[:20]))} 行在分组文件 {groups_path} 里没有组，定不了它每片至多几行")
order = sorted(groups, key=lambda name: (groups[name][0], name))
fixed_two = os.environ.get("MUTATION_SHARD_FIXED_TWO") == "1"
if fixed_two:
    order = ["全部"]
    groups = {"全部": [min(group[0] for group in groups.values()), min(group[1] for group in groups.values()), sorted(seconds_of)]}
# 组比片数上限还多：每片行数上限相邻的两组里合起来行最少的那两组并成一组（每片至多几行取小的），并到不超过上限
while len(order) > maximum:
    index = min(range(len(order) - 1), key=lambda position: (len(groups[order[position]][2]) + len(groups[order[position + 1]][2]), position))
    tighter, looser = order[index], order[index + 1]
    groups[f"{tighter}+{looser}"] = [min(groups[tighter][0], groups[looser][0]), min(groups[tighter][1], groups[looser][1]), groups[tighter][2] + groups[looser][2]]
    del groups[tighter], groups[looser]
    order[index:index + 2] = [f"{tighter}+{looser}"]
# 各组几片：先各一片；照内存要的片数（⌈行数 ÷ 每片至多几行⌉）都给得下就照给；给不下的，余下的片一片一片给「这一组一片约要跑多久」最长的那一组
#   （组的总用时 ÷ 片数 ÷ 一片同时开的工作进程数：一片满了开几个与每片行数的小者），到了内存要的片数的组不再给
counts = {name: (min(2, len(seconds_of)) if fixed_two else 1) for name in order}
needed = {name: -(-len(groups[name][2]) // groups[name][0]) for name in order}
def shard_wall(name):
    _limit, workers, lines = groups[name]
    return sum(seconds_of[line_number] for line_number in lines) / counts[name] / max(1, min(workers, -(-len(lines) // counts[name])))
spare = 0 if fixed_two else maximum - len(order)
while spare > 0:
    growing = [name for name in order if counts[name] < needed[name]]
    if not growing:
        break
    chosen = max(growing, key=lambda name: (shard_wall(name), -order.index(name)))
    counts[chosen] += 1
    spare -= 1

def assign_group(pending, count):
    """一组的行（[(行号, 秒)]，按行号排）分成 count 片：交 [[合计秒, [行号]]]，每片至多 ⌈行数 ÷ count⌉ 行。"""
    capacity = -(-len(pending) // count)
    shards = [[0.0, []] for _index in range(count)]
    def least_loaded(room):   # 还放得下 room 行的片里合计用时最少的那一片（相等取行少的、再取片号小的）；一片都放不下交 None
        fits = [index for index, shard in enumerate(shards) if capacity - len(shard[1]) >= room]
        return min(fits, key=lambda index: (shards[index][0], len(shards[index][1]), index)) if fits else None
    def put(index, rows):
        for line_number, seconds in rows:
            shards[index][0] += seconds
            shards[index][1].append(line_number)
    if os.environ.get("MUTATION_SHARD_BY_TIME_ONLY") == "1":
        pending.sort(key=lambda item: (-item[1], item[0]))   # 用时从长到短，一行一行放
        for row in pending:
            put(least_loaded(1), [row])
        return shards
    # 同一个编译配置的行、同一个文件的行放在同一片：一片冷编的配置数与改的文件数少了，基线的批也少。
    # 每个编译配置的行按（改的文件，行号）排好，切成每段不超过平均每片用时与行数上限的几段，各段连着放进还放得下的片里用时最少的那一片；放不下整段的按行数上限一片一片接着填。
    by_configuration = {}
    for row in pending:
        by_configuration.setdefault(details[row[0]][0], []).append(row)
    target_seconds = sum(seconds for _line_number, seconds in pending) / count
    parts = []
    for configuration, rows in by_configuration.items():
        rows.sort(key=lambda row: (details[row[0]][1], row[0]))
        piece, piece_seconds = [], 0.0
        for row in rows:
            if piece and (len(piece) >= capacity or piece_seconds + row[1] > target_seconds):
                parts.append((piece_seconds, configuration, piece))
                piece, piece_seconds = [], 0.0
            piece.append(row)
            piece_seconds += row[1]
        if piece:
            parts.append((piece_seconds, configuration, piece))
    parts.sort(key=lambda part: (-part[0], part[1], part[2][0][0]))   # 用时从长到短
    for _seconds, _configuration, piece in parts:
        index = least_loaded(len(piece))
        while index is None and piece:
            index = least_loaded(1)
            room = capacity - len(shards[index][1])
            put(index, piece[:room])
            piece = piece[room:]
            index = None
        if index is not None:
            put(index, piece)
    # 行数上限之下各片的用时仍差得多时，把最重那一片里的一行与最轻那一片里的一行对换（两片行数不变），直到最重的不超过平均的 1.03 倍或换不动
    rows_of = {row[0]: row for row in pending}
    average = sum(shard[0] for shard in shards) / count
    for _attempt in range(4 * count):
        heavy = max(range(count), key=lambda index: (shards[index][0], -index))
        light = min(range(count), key=lambda index: (shards[index][0], index))
        gap = shards[heavy][0] - shards[light][0]
        if shards[heavy][0] <= average * 1.03 or gap <= 0:
            break
        best = None
        for heavy_line in shards[heavy][1]:
            for light_line in shards[light][1]:
                moved = rows_of[heavy_line][1] - rows_of[light_line][1]
                if 0 < moved < gap:
                    badness = abs(gap - 2 * moved)
                    if best is None or badness < best[0]:
                        best = (badness, heavy_line, light_line, moved)
        if best is None:
            break
        _badness, heavy_line, light_line, moved = best
        shards[heavy][1].remove(heavy_line)
        shards[heavy][1].append(light_line)
        shards[light][1].remove(light_line)
        shards[light][1].append(heavy_line)
        shards[heavy][0] -= moved
        shards[light][0] += moved
    return shards

details = {}
if os.environ.get("MUTATION_SHARD_BY_TIME_ONLY") != "1":
    with open(table_path, encoding="utf-8") as handle:
        for line_number, line in enumerate(handle, 1):
            fields = line.rstrip("\n").split("\t")
            if line.strip() and not line.startswith("#") and len(fields) >= 6:
                words = fields[4].split()
                details[line_number] = (" ".join(words[:words.index("--")] if "--" in words else words), fields[1])
    unlisted = sorted(line_number for line_number in seconds_of if line_number not in details)
    if unlisted:
        sys.exit(f"要现判的第 {'、'.join(map(str, unlisted[:20]))} 行在 {table_path} 里不是成形的一行，读不出它的编译配置与改的文件")
shards, capacities, over = [], [], []
for name in order:
    limit, _workers, lines = groups[name]
    capacity = -(-len(lines) // counts[name])
    capacities.append(capacity)
    shards += assign_group([(line_number, seconds_of[line_number]) for line_number in sorted(lines)], counts[name])
    if capacity > limit:
        over.append(f"over\t{name}\t{capacity}\t{limit}")
for index, (_total, lines) in enumerate(shards):
    with open(os.path.join(directory, f"shard-{index}.selection"), "w", encoding="utf-8") as handle:
        handle.write("".join(f"{line_number}\n" for line_number in sorted(lines)))
print(len(shards), max(capacities), ",".join(str(len(lines)) for _total, lines in shards), ",".join(f"{total:.0f}" for total, _lines in shards))
print("".join(f"{line}\n" for line in over), end="")
PY_ASSIGN
}

mutation_shard_fail() { # <原因> <出路>
  echo "  ✗ 分片：$1"
  echo "     → 怎么办：$2；修好之后经服务重交 crates-mutation-replay（task-server/task submit -- bash .claude/gate.d/crates-mutation-replay.sh）"
  exit 1
}

# ── 经 task-server 分片（文件头「三步」）───────────────────────────────
mutation_shard_write_plan() { # <计划目录> <一句说明> [<各片行数>]…：写 plan.json（先写临时文件再改名）；各片都不钉（文件头「三步」plan），弄坏开关 pile-pinned-local 下第 0 片钉
  MUTATION_SHARD_PILE_PINNED="$(mutation_shard_break_is pile-pinned-local && echo 1)" python3 - "$@" <<'PY_PLAN'
import json, os, sys
directory, note, *counts = sys.argv[1:]
pinned = os.environ.get("MUTATION_SHARD_PILE_PINNED") == "1"
shards = []
for index, count in enumerate(counts):
    shard = {"label": f"第 {index} 片 {count} 行（服务挑台）"}
    if pinned and index == 0:
        shard["pin"] = "local"
    shards.append(shard)
temporary = os.path.join(directory, "plan.json.partial")
with open(temporary, "w", encoding="utf-8") as handle:
    json.dump({"shards": shards, "note": note}, handle, ensure_ascii=False)
os.replace(temporary, os.path.join(directory, "plan.json"))
PY_PLAN
}

mutation_shard_plan_pending() { # <crates-mutation-rows.py> <树根> <计划目录> <片数上限>：④ 照计划目录里的 pending.rows 按内存定片数、分片、写 plan.json（文件头「三步」plan 的 ④）；
  # 成功句要的片数与各片行数、秒数写进调用方的局部变量 shard_summary
  local rows_module_path="$1" root="$2" plan_directory="$3" maximum="$4" memory_lines group_name group_rows group_text group_count assign_summary shard_count shard_capacity shard_rows shard_seconds
  local over_tag over_name over_capacity over_limit
  memory_lines="$(mutation_shard_memory_groups "$rows_module_path" "$root" "$plan_directory/pending.rows" "$plan_directory/pending.groups" 2> "$plan_directory/memory-groups.err")" \
    || mutation_shard_fail "定不了每片至多几行（$(tail -n 1 "$plan_directory/memory-groups.err")）" \
         "照括号里的原因修（多机配置写上 SLICE_MEMORY_CAP_BY_HOST、NFS_SHARED_ROOTS，或修好 .claude/heavy-thread-default）；整段报错在 $plan_directory/memory-groups.err"
  assign_summary="$(mutation_shard_assign "$plan_directory/pending.rows" "$plan_directory/pending.groups" "$maximum" "$plan_directory" "$root/$MUTATION_SHARD_TABLE")" \
    || mutation_shard_fail "分片那一段 python 没跑成" "看上面 python 的报错"
  read -r shard_count shard_capacity shard_rows shard_seconds <<< "$assign_summary"
  echo "  · ④ 按内存定片数：${memory_lines%%$'\n'*}"
  group_count=0
  while IFS=$'\t' read -r group_name group_rows group_text; do
    group_count=$((group_count + 1))
    echo "  · ④ $group_name 那一组 $group_rows 行：$group_text"
  done < <(tail -n +2 <<< "$memory_lines")
  while IFS=$'\t' read -r over_tag over_name over_capacity over_limit; do
    [[ "$over_tag" == over ]] || continue
    echo "  · ④ $over_name 那一组每片 $over_capacity 行，比内存定的每片至多 $over_limit 行多 $((over_capacity - over_limit)) 行（片数到了服务交的上限 $maximum 片）：各片开几个工作进程由那一片的 crates-mutation-rows.py 照服务给的上限按内存收"
  done < <(tail -n +2 <<< "$assign_summary")
  # 各片行数逗号隔开，拆成一片一个参数
  # shellcheck disable=SC2086
  mutation_shard_write_plan "$plan_directory" "$shard_count 片（$group_count 组各按组里那几行估内存，每片至多 $shard_capacity 行）：各片 ${shard_rows//,/、} 行、约 ${shard_seconds//,/、} 秒" ${shard_rows//,/ } \
    || mutation_shard_fail "写不了 $plan_directory/plan.json" "看计划目录可不可写"
  shard_summary="$shard_count 片、各片 ${shard_rows//,/、} 行约 ${shard_seconds//,/、} 秒"
}

mutation_shard_task_server_plan() { # 文件头「三步」的 plan
  local plan_directory="${TASK_SERVER_SHARD_PLAN_DIRECTORY:-}" maximum="${TASK_SERVER_SHARD_MAXIMUM:-}" split_summary shard_summary=""
  [[ -n "$plan_directory" && -d "$plan_directory" ]] \
    || mutation_shard_fail "计划那一步没给在的计划目录（TASK_SERVER_SHARD_PLAN_DIRECTORY=${plan_directory:-没设}）" "这一步只由 task-server 按分片协议起"
  mutation_shard_row_plan "$rows_module" "$tree_root" "$plan_directory/row-plan.tsv" "$only_rows_file" \
    || mutation_shard_fail "本机列不出每一行的记录与用时（crates-mutation-rows.py row-plan 退非 0$(mutation_shard_row_plan_failure_reason "$plan_directory/row-plan.tsv")）" "它的完整输出在 $plan_directory/row-plan.tsv；要重现就单跑 python3 scripts/mutation/crates-mutation-rows.py row-plan <树根> [--rows <选行文件>]"
  split_summary="$(mutation_shard_split_plan "$plan_directory/row-plan.tsv" "$plan_directory/pending.rows" "$plan_directory/after-merge.rows" "$plan_directory/keys" "$only_rows_file")" \
    || mutation_shard_fail "分行那一段 python 没跑成" "看上面 python 的报错"
  local table_rows pending_rows timed_rows default_seconds refused_rows anchor_broken_rows reused_rows unaccounted_rows
  read -r table_rows pending_rows timed_rows default_seconds refused_rows anchor_broken_rows reused_rows unaccounted_rows <<< "$split_summary"
  # 点名的每一行要么要现判（pending.rows）、要么留给合并之后的核账（after-merge.rows）：有行两边都不在，计划不写、整步判红，不让它悄悄漏掉
  [[ "$unaccounted_rows" == "-" ]] \
    || mutation_shard_fail "点名的第 ${unaccounted_rows//,/、} 行（变异表行号）不在要现判的行（$plan_directory/pending.rows）里、也不在留给合并之后的行（$plan_directory/after-merge.rows）里：这几行这一趟谁都不判" \
         "看 $plan_directory/row-plan.tsv 里这几行的第二、三、五列（键、复用、锚点失效）是不是 row-plan 认得的写法，再看 mutation_shard_split_plan 的分法"
  if (( pending_rows == 0 )); then
    echo "  · ③ $table_rows 行里没有要现判的（拒算的 $refused_rows 行、锚点失效的 $anchor_broken_rows 行、复用的 $reused_rows 行不分，逐行在 $plan_directory/after-merge.rows）：计划写 0 片，合并那一步核账"
    mutation_shard_write_plan "$plan_directory" "没有要现判的行" || mutation_shard_fail "写不了 $plan_directory/plan.json" "看计划目录可不可写"
    return 0
  fi
  if [[ ! "$maximum" =~ ^[1-9][0-9]*$ ]]; then
    echo "  ✗ 分片：环境里的 TASK_SERVER_SHARD_MAXIMUM 不是正整数（${maximum:-没给}），定不了至多分几片，plan.json 没写（退 2）"
    echo "     → 怎么办：这一步只由 task-server 照分片协议起（task-server/task_server_lib.py 文件头「分片」①），服务交这个变量；经服务交那一道：task-server/task submit -- bash .claude/gate.d/crates-mutation-replay.sh"
    exit 2
  fi
  mutation_shard_plan_pending "$rows_module" "$tree_root" "$plan_directory" "$maximum"
  echo "  · ③ $table_rows 行里要现判的 $pending_rows 行按内存分 $shard_summary（拒算的 $refused_rows 行、锚点失效的 $anchor_broken_rows 行、复用的 $reused_rows 行留给合并那一步的核账，逐行在 $plan_directory/after-merge.rows）"
}

mutation_shard_task_server_shard() { # 文件头「三步」的 shard
  local plan_directory="${TASK_SERVER_SHARD_PLAN_DIRECTORY:-}" output="${TASK_SERVER_SHARD_OUTPUT_DIRECTORY:-}" index="${TASK_SERVER_SHARD%%/*}"
  local selection stage_exit=0 llvm_report
  local -a stage_environment=(env -u TASK_SERVER_SHARD_PHASE -u GATE_MUTATION_SHARDED_ONLY)   # 一片起的那一道是选行模式：不再分、照现判
  [[ -n "$plan_directory" && -d "$plan_directory" && -n "$output" && "$index" =~ ^[0-9]+$ ]] \
    || mutation_shard_fail "这一片缺 task-server 给的变量（计划目录 ${plan_directory:-没设}、产物目录 ${output:-没设}、片号 ${TASK_SERVER_SHARD:-没设}）" "这一步只由 task-server 起"
  mkdir -p -- "$output" || mutation_shard_fail "建不了产物目录 $output" "看它可不可写"
  output="$(cd "$output" && pwd)"
  plan_directory="$(cd "$plan_directory" && pwd)"
  selection="$plan_directory/shard-$index.selection"
  [[ -s "$selection" ]] || mutation_shard_fail "计划目录里没有第 $index 片的选行文件" "计划那一步没写全：看计划那一步的日志"
  # 不看派到哪一台：每一片在跑它的这一台现查 llvm 工具与 RocksDB 预编库的取值（文件头「三步」shard），缺了退 78。弄坏开关 llvm-tools-missing-skipped 下缺 llvm 工具不判、退 0
  if ! llvm_report="$(bash -c "$(mutation_shard_llvm_check_command)" 2>&1)"; then
    if mutation_shard_break_is llvm-tools-missing-skipped; then
      echo "  · 这一台缺 llvm-tools（${llvm_report##*$'\n'}）：这一堆不判"
      return 0
    fi
    echo "  ✗ 这一台缺记覆盖要的 llvm 工具（${llvm_report##*$'\n'}），这一片一条变异都没起（退 78）"
    echo "  → 怎么办：在这一台装 llvm-tools：rustup component add llvm-tools（装进 rustc --print sysroot 下 lib/rustlib/<host>/bin）；工具放在别处的设 GATE_MUTATION_LLVM_TOOLS=<目录>；装好再重投"
    return 78
  fi
  if ! ( cd "$tree_root" && python3 scripts/hosts/rocksdb-prebuilt.py entry --root . ); then
    echo "  ✗ 这一片起 cargo 之前 RocksDB 预编库的取值没判过（上面 rocksdb-prebuilt.py entry 的红句），一条变异都没起（退 78）"
    echo "  → 怎么办：在这一台上照上面的出路修（没装就跑 python3 scripts/hosts/rocksdb-prebuilt.py install；装了而 ssh 起的命令拿不到取值，看 ~/.bashrc 最前面那一段），再重投"
    return 78
  fi
  echo "  · 第 $TASK_SERVER_SHARD 片（这一台，判的记录写进 prune-server 里同一份判定库）：$(grep -c '[0-9]' "$selection") 行"
  # 这一片本次未跑的几条由门禁追加进产物目录的 not-run-record（crates-mutation-replay.sh 文件头「分片」），合并那一步照它列；弄坏开关 not-run-not-handed-over 下不交这个变量
  rm -f -- "$output/not-run-record" || mutation_shard_fail "删不掉上一次留在产物目录里的 $output/not-run-record" "看产物目录可不可写"
  if mutation_shard_break_is not-run-not-handed-over; then
    echo "  ! 弄坏开关 MUTATION_SHARD_RUN_BREAK=not-run-not-handed-over：起门禁不带 GATE_MUTATION_SHARD_NOT_RUN_FILE" >&2
  else
    stage_environment+=(GATE_MUTATION_SHARD_NOT_RUN_FILE="$output/not-run-record")
  fi
  if ( cd "$tree_root" && exec "${stage_environment[@]}" GATE_MUTATION_ROW_SELECTION="$selection" bash "$stage_script" "$tree_root" ); then stage_exit=0; else stage_exit=$?; fi
  # 起了门禁（退 0、1、3）的这一片把它判的行交给合并去核（文件头「三步」merge）；别的退出码这一片没成，不交
  case "$stage_exit" in
    0|1|3) cp -- "$selection" "$output/judged-rows" || mutation_shard_fail "拷不了选行文件到 $output/judged-rows" "看产物目录可不可写" ;;
  esac
  case "$stage_exit" in
    1|3) echo "  · 这一片的 crates-mutation-replay 退 $stage_exit（3 是选中的都抓到、1 是有红或没判完；判定以合并那一步的核账为准）"; return 0 ;;
    *) echo "  ✗ 这一片的 crates-mutation-replay 退 $stage_exit"; echo "  → 怎么办：看这一片的日志（task-server/task log <号>）"; return "$stage_exit" ;;
  esac
}

mutation_shard_rows_in_service() { # <row-plan 的输出> <judged-rows> <读不到的行写进哪个文件> <这一片的 not-run-record> <本次未跑的行写进哪个文件> <读不出的记录行写进哪个文件>：
  # stdout 打「<读得到的行数> <读不到的行数> <本次未跑的行数> <读不出的记录行数>」（文件头「三步」merge）。
  # 一行是哪一档：row-plan 第三列是 1 的读得到；不是 1 而这一片的 not-run-record 里有它的，本次未跑（「<行号>\t<门禁写的那一句>」逐行写进第五个文件）；两样都不是的读不到。
  # not-run-record 每条一行「<门禁文件名> 本次未跑：<表>:<行号> <变异名>：…」（crates-mutation-replay.sh 写的）：不在这一片判的行里、认不出这个形状的行，
  # 整行写进第六个文件、不当本次未跑（它点名的行照读不到算），由调用方判红；文件不在就是这一片没报本次未跑的行
  MUTATION_SHARD_MERGE_UNCHECKED="$(mutation_shard_break_is merge-rows-unchecked && echo 1)" \
  MUTATION_SHARD_NOT_RUN_IGNORED="$(mutation_shard_break_is not-run-counted-unreadable && echo 1)" \
  MUTATION_SHARD_UNREPORTED_AS_NOT_RUN="$(mutation_shard_break_is unreported-rows-as-not-run && echo 1)" \
  MUTATION_SHARD_RECORD_TABLE="$MUTATION_SHARD_TABLE" python3 - "$@" <<'PY_ROWS'
import os, re, sys
plan_path, judged_path, unreadable_path, record_path, not_run_path, malformed_path = sys.argv[1:]
with open(judged_path, encoding="utf-8") as handle:
    judged = sorted({int(text) for text in (line.strip() for line in handle) if text.isdigit()})
with open(plan_path, encoding="utf-8") as handle:
    readable_rows = {int(fields[0]) for fields in (line.rstrip("\n").split("\t") for line in handle if line.strip()) if len(fields) > 2 and fields[2] == "1"}
if os.environ.get("MUTATION_SHARD_MERGE_UNCHECKED") == "1":   # 弄坏开关 merge-rows-unchecked：不查服务，一律当读得到
    readable_rows = set(judged)
reported, malformed = {}, []
if os.environ.get("MUTATION_SHARD_NOT_RUN_IGNORED") != "1" and os.path.isfile(record_path):   # 弄坏开关 not-run-counted-unreadable：不读这一片报的本次未跑
    shape = re.compile(r"^\S+ 本次未跑：(" + re.escape(os.environ["MUTATION_SHARD_RECORD_TABLE"]) + r":([0-9]+) .+)$")
    with open(record_path, encoding="utf-8") as handle:
        for line in (text.rstrip("\n") for text in handle):
            if not line.strip():
                continue
            found = shape.match(line)
            if found is None or int(found.group(2)) not in judged:
                malformed.append(line)
            else:
                reported.setdefault(int(found.group(2)), found.group(1))
if os.environ.get("MUTATION_SHARD_UNREPORTED_AS_NOT_RUN") == "1":   # 弄坏开关 unreported-rows-as-not-run：没报本次未跑、读不到的行也当本次未跑（报了的行照旧带那一片打的原因）
    reported.update({line_number: f"{os.environ['MUTATION_SHARD_RECORD_TABLE']}:{line_number} 没报本次未跑" for line_number in judged if line_number not in readable_rows and line_number not in reported})
readable = [line_number for line_number in judged if line_number in readable_rows]
not_run = [line_number for line_number in judged if line_number not in readable_rows and line_number in reported]
unreadable = [line_number for line_number in judged if line_number not in readable_rows and line_number not in reported]
with open(unreadable_path, "w", encoding="utf-8") as handle:
    handle.write("".join(f"{line_number}\n" for line_number in unreadable))
with open(not_run_path, "w", encoding="utf-8") as handle:
    handle.write("".join(f"{line_number}\t{reported[line_number]}\n" for line_number in not_run))
with open(malformed_path, "w", encoding="utf-8") as handle:
    handle.write("".join(f"{line}\n" for line in malformed))
print(len(readable), len(unreadable), len(not_run), len(malformed))
PY_ROWS
}

mutation_shard_merge_line() { # <步名> <正文>：合并那一步的分步行（文件头「三步」merge）：「  · 合并·<步名>：<正文>」，打 stdout；弄坏开关 merge-steps-silent 下不打
  mutation_shard_break_is merge-steps-silent || echo "  · 合并·$1：$2"
}

mutation_shard_task_server_merge() { # 文件头「三步」的 merge：只核各片判的行按本机此刻的行键在 prune-server 里读得到（各片自己报了本次未跑的行另算）；不拷库目录、不 import
  local outputs="${TASK_SERVER_SHARD_OUTPUTS:-}" index host directory scratch counts readable unreadable not_run malformed merged=0 total_readable=0 total_unreadable=0 total_not_run=0
  local step_started position judged_rows grep_exit union_rows shard_summary="" not_run_note="" not_run_entry
  local -a problems=() unreadable_notes=() not_run_lines=() shard_indexes=() shard_hosts=() shard_directories=() judged_files=() row_plan_progress=(--progress)
  [[ -n "$outputs" && -f "$outputs" ]] || mutation_shard_fail "合并那一步没给在的各片产物表（TASK_SERVER_SHARD_OUTPUTS=${outputs:-没设}）" "这一步只由 task-server 起"
  scratch="$(mktemp -d)" || mutation_shard_fail "建不了放 row-plan 输出的临时目录" "看 \$TMPDIR 满没满、有没有写权限"
  # 读计划：各片的产物表里哪几片交了 judged-rows，各判了几行；所有片判的行并成一份（去重、从小到大），row-plan 只算一趟
  step_started=$SECONDS
  mutation_shard_merge_line 读计划 开始
  while IFS=$'\t' read -r index host directory; do
    [[ -n "$directory" && -s "$directory/judged-rows" ]] || continue
    judged_rows="$(grep -c '[0-9]' "$directory/judged-rows")" || judged_rows=0
    shard_indexes+=("$index"); shard_hosts+=("$host"); shard_directories+=("$directory"); judged_files+=("$directory/judged-rows")
    shard_summary+="${shard_summary:+、}第 $index 片（$host）$judged_rows 行"
  done < "$outputs"
  : > "$scratch/judged-all.rows"
  if (( ${#judged_files[@]} > 0 )); then
    grep -h -E '^[0-9]+$' -- "${judged_files[@]}" > "$scratch/judged-all.rows"; grep_exit=$?
    (( grep_exit <= 1 )) || mutation_shard_fail "读不了各片的 judged-rows（grep 退 $grep_exit）" "看产物目录读不读得到：${judged_files[*]}"
  fi
  sort -nu -- "$scratch/judged-all.rows" > "$scratch/judged-union.rows" || mutation_shard_fail "各片判的行并不起来（sort 退非 0）" "看 \$TMPDIR 满没满、有没有写权限"
  union_rows="$(grep -c . "$scratch/judged-union.rows")" || union_rows=0
  mutation_shard_merge_line 读计划 "完，用了 $((SECONDS - step_started)) 秒，${#shard_indexes[@]} 片交了判的行，并起来去重 $union_rows 行（${shard_summary:-没有}）"
  # 现算行键、读库（row-plan --progress 自己分步打进度，写法在 crates-mutation-rows.py 文件头「合并那一步的分步进度」）：并起来的行按本机此刻的源码算键、查服务（第三列 1 是读得到）
  mutation_shard_break_is merge-steps-silent && row_plan_progress=()
  if (( union_rows > 0 )); then
    if ! python3 "$rows_module" row-plan "$tree_root" --rows "$scratch/judged-union.rows" ${row_plan_progress[@]+"${row_plan_progress[@]}"} > "$scratch/row-plan.tsv"; then
      problems+=("crates-mutation-rows.py row-plan 退非 0（原因在上面它打的行：$(grep -m 1 '✗' "$scratch/row-plan.tsv")）")
    fi
  else
    : > "$scratch/row-plan.tsv"
  fi
  # 逐片核：这一片判的几行里读到几行
  if (( ${#problems[@]} == 0 )); then
    step_started=$SECONDS
    mutation_shard_merge_line 逐片核对 开始
    for position in "${!shard_indexes[@]}"; do
      index="${shard_indexes[position]}" host="${shard_hosts[position]}" directory="${shard_directories[position]}"
      counts="$(mutation_shard_rows_in_service "$scratch/row-plan.tsv" "$directory/judged-rows" "$scratch/unreadable-$index" "$directory/not-run-record" "$scratch/not-run-$index" "$scratch/malformed-$index")" \
        || { problems+=("第 $index 片（$host）：核读得到读不到的那一段 python 没跑成"); continue; }
      read -r readable unreadable not_run malformed <<< "$counts"
      merged=$((merged + 1))
      total_readable=$((total_readable + readable)); total_unreadable=$((total_unreadable + unreadable)); total_not_run=$((total_not_run + not_run))
      not_run_note=""
      (( not_run == 0 )) || not_run_note="、本次未跑 $not_run 行"
      mutation_shard_merge_line 逐片核对 "第 $index 片（$host）判的 $((readable + unreadable + not_run)) 行，读到 $readable 行、读不到 $unreadable 行$not_run_note"
      while IFS= read -r not_run_entry; do not_run_lines+=("第 $index 片（$host）${not_run_entry#*$'\t'}"); done < "$scratch/not-run-$index"
      if (( malformed > 0 )); then
        problems+=("第 $index 片（$host）交的本次未跑记录（$directory/not-run-record）有 $malformed 行读不出行号、或点名的行不在这一片判的行里，这几行不当本次未跑，头一行：$(head -n 1 -- "$scratch/malformed-$index" | clip_text 300)")
      fi
      if (( readable == 0 && unreadable > 0 )); then
        problems+=("第 $index 片（$host）判的 $unreadable 行按此刻的行键在 prune-server 里一行都读不到（先查它是不是写进了别的库：那一片连的是不是同一份服务；再比两台的源码：被判的是不是同一棵树，被改坏的单元与点名测试的闭包里的源码不同，行键就不同；行键里没有编译器、链接器、cargo 配置与版本这类构建环境，不用比）")
      elif (( unreadable > 0 )); then
        unreadable_notes+=("第 $index 片（$host）读不到的 $unreadable 行：$(tr '\n' ' ' < "$scratch/unreadable-$index")")
      fi
    done
    mutation_shard_merge_line 逐片核对 "完，用了 $((SECONDS - step_started)) 秒"
  fi
  rm -rf -- "${scratch:?}"
  # 各片自己报了本次未跑的行逐条列（门禁的写法：「本次未跑：N 条」一行，之后一条一行），不算读不到、不让整道判红
  if (( total_not_run > 0 )); then
    echo "  · 本次未跑：$total_not_run 条（各片起的门禁报的：点名的测试在那一片所在的台上没跑到被改坏的那一处，不算抓到、不算没红，这一趟不是整张表的结论）"
    printf '    · %s\n' "${not_run_lines[@]}"
  fi
  (( ${#problems[@]} == 0 )) || mutation_shard_fail "$(IFS='；'; echo "${problems[*]}")" "照括号里点名的那一片先查它写进了哪份库，再比两台的源码；row-plan 退非 0 的单跑 python3 scripts/mutation/crates-mutation-rows.py row-plan <树根> --rows <某一片的 judged-rows> --progress 看它报什么"
  (( ${#unreadable_notes[@]} == 0 )) || echo "  · 合并：$(IFS='；'; echo "${unreadable_notes[*]}")（没判成或不算数的结局，由接着的核账报）"
  [[ -z "${TASK_SERVER_SHARD_MISSING:-}" ]] || echo "  · 合并：缺第 ${TASK_SERVER_SHARD_MISSING} 片（人手单独取消的），它那一堆的行没判，接着的核账报它们"
  if (( total_not_run == 0 )); then
    echo "  ✓ 分片合并：$merged 片判的 $((total_readable + total_unreadable)) 行里 $total_readable 行按此刻的行键在 prune-server 里读得到、$total_unreadable 行读不到；接着由 crates-mutation-replay 在这一步核账"
  else
    echo "  ✓ 分片合并：$merged 片判的 $((total_readable + total_unreadable + total_not_run)) 行里 $total_readable 行抓到（按此刻的行键在 prune-server 里读得到）、$total_not_run 行本次未跑（逐条见上，不算抓到、不算没红）、$total_unreadable 行读不到；这一趟不是整张表的结论；接着由 crates-mutation-replay 在这一步核账"
  fi
}

mutation_shard_main() {
  if [[ -n "${GATE_MUTATION_ROW_SELECTION:-}" ]]; then
    echo "  ✗ 分片驱动在选行模式下被调了（GATE_MUTATION_ROW_SELECTION 设着）：各片起的 crates-mutation-replay 不许再调驱动"
    echo "     → 怎么办：这一份只由 crates-mutation-replay 在服务分片的三步里起；选行模式下 crates-mutation-replay 自己不调，这里被调说明调用方写错了"
    exit 2
  fi
  tree_root="${1:-}"
  only_rows_file=""
  if (( $# >= 1 )); then shift; fi
  while (( $# )); do
    case "$1" in
      --only-rows)
        if (( $# < 2 )) || [[ ! -s "$2" ]]; then
          echo "  ✗ --only-rows 后面要给一份非空的选行文件（一行一个变异表行号）：读到「${2:-}」"
          echo "     → 怎么办：点名几行经服务交 task-server/task submit -- bash .claude/gate.d/crates-mutation-replay.sh --item <行号或变异名>，那一道把 --item 换成选行文件"
          exit 2
        fi
        only_rows_file="$2"; shift 2 ;;
      *)
        echo "  ✗ 认不出的参数 $1"
        echo "     → 怎么办：只认一个树根，后面可跟 --only-rows <选行文件>；或单独的 --selftest"
        exit 2 ;;
    esac
  done
  if [[ -z "$tree_root" || ! -d "$tree_root" ]]; then
    echo "  ✗ 用法：mutation-shard-run.sh <树根> [--only-rows <选行文件>]，或 --selftest"
    echo "     → 怎么办：经服务交那一道：task-server/task submit -- bash .claude/gate.d/crates-mutation-replay.sh [--item <行号或变异名>]"
    exit 2
  fi
  tree_root="$(cd "$tree_root" && pwd)"
  stage_repository="$(cd "$mutation_shard_script_directory/../.." && pwd)"
  stage_script="$stage_repository/.claude/gate.d/crates-mutation-replay.sh"
  rows_module="$stage_repository/scripts/mutation/crates-mutation-rows.py"
  # 只跑服务拆的那一步（文件头「三步」）；不是服务按分片协议起的一概不分
  case "${TASK_SERVER_SHARD_PHASE:-}" in
    plan) mutation_shard_task_server_plan; exit $? ;;
    shard) mutation_shard_task_server_shard; exit $? ;;
    merge) mutation_shard_task_server_merge; exit $? ;;
    "")
      echo "  ✗ mutation-shard-run.sh 只由 crates-mutation-replay 在服务分片的三步里起（环境里要有 TASK_SERVER_SHARD_PHASE）：它不自己连第二台、不给哪一片套内存包装，这一次什么都没分（退 2）"
      echo "     → 怎么办：在仓根经服务交那一道：task-server/task submit -- bash .claude/gate.d/crates-mutation-replay.sh [--item <行号或变异名>]；服务拆成计划、各片、合并三步，合并那一步核账判整张"
      exit 2 ;;
    *)
      echo "  ✗ TASK_SERVER_SHARD_PHASE=$TASK_SERVER_SHARD_PHASE 认不出（只认 plan、shard、merge）"
      echo "     → 怎么办：这个变量只由 task-server 设；去掉它再跑"
      exit 2 ;;
  esac
}

# ── 自证 ──────────────────────────────────────────────────────────────────────
# 在临时目录里搭一棵小仓（本仓的 .claude/scripts、crates-mutation-replay 与登记表、scripts 整个目录、.claude/hooks/lib_work_root.py（memory_peaks_lib.py 的自证按路径载入它）、规范副本的符号链接，加一个 crates/tiny 与六行变异表），
# 配置写在临时目录里（SINGLEFS_MULTI_HOST_CONFIG）。
# 假 cargo（往自己目录下的 invocations.log 记起了哪几条）照 lib.rs 里「// TEST <名> NEEDS <原文>」判 ok / FAILED。逐格：
#   ① 没经分片判过的行，门禁不现判：库是空的场景里直接起 crates-mutation-replay（核账），退 1、报 6 行在判定库里没有键相同的记录，假 cargo 一次测试都没起
#      （GATE_MUTATION_BREAK=unsharded-rows-judged 下照现判、退 0，这一格判错）
#   ⑦ 选行模式下调驱动：退 2 拒
#   ⑨ 弄坏开关设在自证之外（起时去掉自证的标记）：驱动拒、退 2
#   ㉔ 经 task-server 分片的三步：场景里照 task-server 设的变量逐步起计划、两片、合并，计划写两片（按内存定片数：场景的峰值记录还没有，6 行按保守常量每片至多 5 行）、都不带内存下限，
#      两片判的记录都写进场景那一份库（服务里同一份，不拷库目录）、各片的产物目录里有 judged-rows，合并按此刻的行键在服务里读得到判的每一行，
#      之后门禁核账全部复用、判绿；不是服务按分片协议起的（没有 TASK_SERVER_SHARD_PHASE）退 2、提示经 task-server 提交
#   ㉕ 合并只认服务里读得到的：两片起之前就合并，另一片的 judged-rows 点名的行还没判进服务（同「两台源码不同」「写进了别的库」）⇒ 合并退 1、点名那一片，
#      出路先写查它写进了哪份库、再写比两台的源码（弄坏开关 merge-rows-unchecked）
#   ㉟ 合并认一片报的本次未跑：场景 notrun 照计划、两片、合并三步走一遍，两台的假 cargo 都把第 9 行点名的测试当没卡的机器上的用例（自己目录下的 not-run-tests，打 NOT_RUN_NO_GPU_ADAPTER 后返回成功），
#      报了的那一片的产物目录里有 not-run-record；合并退 0，另起「本次未跑：1 条」一行、之后逐条列（行号、变异名、那一片打的原因），成功句报抓到几行、本次未跑 1 行、读不到 0 行与「这一趟不是整张表的结论」；
#      只判了那一行的一片（一行都读不到、但报了本次未跑）合并同样退 0（弄坏开关 not-run-counted-unreadable、not-run-not-handed-over）
#   ㊱ 合并：一片没报本次未跑而判的行读不到，退 1、点名那一片、不写本次未跑；报了本次未跑而记录读不出行号的也退 1、不当本次未跑（弄坏开关 unreported-rows-as-not-run）
#   ㉒ 计划那一步带 --only-rows 点名一行：row-plan 只算那一行的键（弄坏开关 row-plan-whole-table、only-rows-ignored），计划只分那一行；
#      直接起 row-plan 点名两行算两行、别的行键写 -、不点名算整张 6 行（弄坏开关 row-plan-selection-ignored）
#   ㉝ 计划那一步的 row-plan 退非 0：假的 row-plan 打一条失败行、退 2，失败句（退 1）里有那一条失败行的字；另一个假的什么都不打、退 2，失败句写「它没打失败行」、没有空括号（弄坏开关 row-plan-failure-line-dropped）
#   ⑭ ③ 分行（mutation_shard_split_plan）喂一份写好的 row-plan 输出：锚点失效的两行（一行没键、一行有键）与拒算、复用的不进 pending.rows，汇总单报锚点失效 2 行
#   ⑱ ③ 分行喂 --only-rows 的选行文件：只在选中的行里分；选中的行里要现判的进 pending.rows、复用的不进
#   ㉗ 点名 N 行：pending.rows 加 after-merge.rows 恰好是点名的 N 行、不相交（喂 ⑭ 那份点名五行与不点名整张两趟；㉔ 判完之后计划那一步点名三行复用的：0 片、三行都记复用）
#      （弄坏开关 reused-rows-unlisted、only-rows-ignored）
#   ㉘ 按内存定片数（文件头「三步」plan 的 ④；弄坏开关 shards-fixed-two）：一棵 52 行变异表的小仓（行都点名包 tiny），峰值记录里包 tiny 只在编译并行度 1024 记 3.8 GiB
#      （任何一台的编译并行度都借它），多机配置两台 64G、50G（安全额度照小的那台：50G × 2/3），工作进程数给 31，三趟同时跑：
#      要现判 52 行、片数上限 64：各片的行数 c 都满足 min(c, 31) × 3.8 GiB + 1 GiB ≤ 50G × 2/3，不止两片；要现判 3 行：只一片；要现判 52 行、片数上限 4：恰好 4 片；
#      三趟各片合起来都恰好是要现判的那几行、不相交
#   ㉜ 按片估内存（文件头「三步」plan 的 ④；弄坏开关 shard-estimate-over-all-rows、shards-fixed-two）：另一棵小仓，20 行轻的（-p tiny --lib，峰值记录 tiny--lib 2 GiB）、
#      2 行整个包跑的重的（-p tiny，包 tiny 28 GiB），安全额度与工作进程数同 ㉘，两趟同 ㉘ 那一批同时跑：
#      片数上限 8：恰好 4 片，重的两行各单独一片，只有轻的行的片每片不过 16 行，计划的 ④ 写轻的那一组每个工作进程的峰值 2.0 GiB；
#      片数上限 3：恰好 3 片，重的两行仍各单独一片，轻的 20 行一片、写比内存定的每片至多 16 行多 4 行；两趟各片合起来都恰好是要现判的那几行、不相交
#   ㉛ 按编译配置与改的文件分片（文件头「三步」plan 的 ④；弄坏开关 shards-by-time-only）：mutation_shard_assign 直接喂 128 行（8 个编译配置各 16 行，一个配置的行用时是别的十倍）、每片至多 32 行（分组文件写一组），
#      同只按用时均分的参照（MUTATION_SHARD_ASSIGN_REFERENCE=1）比：片数与行数上限一样、各片合起来恰好是这 128 行、不相交，不同编译配置数不到参照的四分之三，最重的一片不超过参照最重的 1.1 倍
#   ㉙ 合并分步输出（文件头「三步」merge）：照服务的合并起门禁（驱动先合并、之后核账），标准输出与标准错误并进一个文件（像任务日志）：读计划、现算行键、读库、逐片核对、核账各步的开始、结束行按次序都在，
#      逐片两行的数与各片 judged-rows 的行数对，核账完退出码 0（弄坏开关 merge-steps-silent）
#   ㉚ 合并里一步慢时有心跳行：逐行的步每过一行停 0.3 秒（GATE_MUTATION_PROGRESS_ROW_PAUSE_SECONDS，假的慢读库）、间隔 1 秒（GATE_MUTATION_PROGRESS_SECONDS），驱动的 row-plan 里与核账里的现算行键、读库
#      都有带 TASK_SERVER_HEARTBEAT 前缀的「在第 N 行、共 M 行」，这种行都带前缀、开始与结束行不带（弄坏开关 merge-steps-silent）
#   ⑫ 场景都跑在 TMPDIR=<自证的临时目录>/tmp 下（command-safety.md「测试镜像一律放临时目录」）：本机那一侧的假 cargo（往自己目录下的 working-directories.log 记工作目录）
#      在 singlefs-mutation-replay-* 里起的每一次都在自证自己的 tmp/ 下、至少一次；跑完外面的临时目录里没多出 singlefs-mutation-replay-*，
#      「外面的临时目录」是自证在调用方的 TMPDIR 下另建的一层（自证的临时目录就在它里面）：同一个 TMPDIR 下并行的几份自证互不数到对方建的目录，
#      自证自己的 tmp/ 下也一个不剩（删不删由 crates-mutation-rows.py 的 run_pool 管）
# 并行（.claude/rules/test-script-discipline.md「编译与算法都要快」）：三个场景（fresh、steps、notrun）各一串，三串同时跑；串里互不依赖的几件再同时跑（run_together），
#   一批至多每台默认重活线程数件（scripts/hosts/heavy-threads.sh 的 heavy_thread_default，设了 SINGLEFS_THREAD_CAP 照它；取不到一件一件跑）。
#   每件的输出与退出码落 <临时目录>/parallel/<批名>/ 下它自己的文件，照派活的次序逐个 wait 它的进程号；派出去的件没留下退出码文件的，逐个点名、记一格判错（collect_together）。
#   各格记进各串自己的文件（子 shell 里的计数传不回来），收齐之后照登记次序回放、数格数；成功句报「并行 N 路」（各批同时跑的最多件数）。
#   有次序的留在串里照次序跑：⑦ ⑨ 用 ① 搭好的场景；steps 里计划那一步在前，㉕ 的合并要在两片判进库之前，
#   两片判完才合并、核账、再点名复用的行；⑭ ⑱ ㉗ ⑫ 在三串都收回之后跑（㉗ 读 steps 那一串留下的文件，⑫ 数三个场景留下的记录）；㉘ 的三趟与 ㉜ 的两趟在那之后同时跑（run_together memory-plan）。
# 弄坏开关各让对应的格判错（见文件头「弄坏开关」）。
# 场景里的门禁经 prune-server 写判定库：只在临时服务上跑（scripts/prune-server/prune_server_client.py 文件头「命令行」--with-temporary-service），不是就退 2、一格都不跑，不写真服务。
mutation_shard_selftest() {
  local repository_root work template checked=0 failures=0 service_problem
  if ! service_problem="$(python3 -c 'import sys; sys.path.insert(0, sys.argv[1]); import prune_server_client; print(prune_server_client.selftest_service_problem())' "$mutation_shard_script_directory/../prune-server" 2>&1)" || [[ -n "$service_problem" ]]; then
    echo "  ✗ mutation-shard-run.sh 自证没跑：${service_problem:-读不了 scripts/prune-server/prune_server_client.py}"
    echo "     → 怎么办：照 scripts/prune-server/prune_server_client.py 文件头「命令行」--with-temporary-service 那一条，在临时服务底下起这一份自证（带 --server-binary 就不编）"
    return 2
  fi
  repository_root="$(cd "$mutation_shard_script_directory/../.." && pwd)"
  # 自证自己的外层临时根：调用方的 TMPDIR 下另建一层，自证的工作目录与 ⑫ 比的「外面的临时目录」都在它里面。
  #   同一个 TMPDIR 下并行起的几份自证（弄坏开关 selftest-tmpdir-outside 下那一份把场景建在外面）互不数到对方建的 singlefs-mutation-replay-*
  local own_temporary
  own_temporary="$(mktemp -d)"
  export TMPDIR="$own_temporary"
  work="$(mktemp -d)"
  mutation_shard_selftest_work="$own_temporary"
  # 自证起的驱动认弄坏开关的标记：场景里起的都带着它
  printf '%s\n' "$MUTATION_SHARD_SELFTEST_MARK_CONTENT" > "$work/selftest-mark"
  export MUTATION_SHARD_RUN_SELFTEST_MARK="$work/selftest-mark"
  # 复用哨兵（scripts/gate/lib_reuse_checks.py 文件头「二」）：场景里 crates-mutation-replay 每一趟都抽；这里比的是复用与现跑的条数，
  #   键相同的全当哨兵、不抽强制现判（哨兵数比行数多），哨兵与强制现判本身在 crates-mutation-rows.py 的自证 reuse-sentinel 那一格判
  export SINGLEFS_REUSE_SENTINEL_COUNT=1000
  unset SINGLEFS_REUSE_SENTINEL_SEED
  trap 'echo "  · 自证的临时目录（$(du -sh "$mutation_shard_selftest_work" 2>/dev/null | cut -f1)）删掉"; rm -rf -- "${mutation_shard_selftest_work:?}"' EXIT
  # 场景起的 crates-mutation-replay 与它的工作进程都照 TMPDIR 建临时目录：换成自证自己目录下的 tmp/，跑完随自证的临时目录一起删；
  # 开跑前先记下外面那个临时目录里已有的 singlefs-mutation-replay-*，⑫ 比的是自证跑完多出来的
  local outside_temporary="${TMPDIR:-/tmp}" outside_before
  outside_before="$(compgen -G "$outside_temporary/singlefs-mutation-replay-*" | sort || true)"
  mkdir -p "$work/tmp"
  if ! mutation_shard_break_is selftest-tmpdir-outside; then
    export TMPDIR="$work/tmp"
  fi
  # 一格一行记进 $expect_records（各场景那一串在自己的子 shell 里跑，各记各的文件；收尾照登记次序回放、数格数：子 shell 里的计数传不回来）
  local expect_records="$work/expect-selftest.tsv"
  expect() { # expect <格名> <判据的退出码：0 为过> <不过时的细节>
    local detail="${3//$'\n'/ }"
    printf '%s\t%s\t%s\n' "$2" "$1" "${detail//$'\t'/ }" >> "$expect_records"
  }
  # 互不依赖的几件同时跑（.claude/rules/test-script-discipline.md「编译与算法都要快」），一批至多每台默认重活线程数件（scripts/hosts/heavy-threads.sh 的 heavy_thread_default，
  #   设了 SINGLEFS_THREAD_CAP 照它；取不到一件一件跑）；每件的退出码写自己的文件，照派活的次序逐个 wait 它的进程号（command-safety.md「并行不许把失败吃掉」）
  local selftest_jobs
  # shellcheck source=../hosts/heavy-threads.sh
  source "$mutation_shard_script_directory/../hosts/heavy-threads.sh"
  if ! selftest_jobs="$(heavy_thread_default)"; then
    echo "  · 取不到每台默认重活线程数，自证里互不依赖的几件一件一件跑"
    selftest_jobs=1
  fi
  run_together() { # <批名> <件名> <函数> <参数> [<件名> <函数> <参数>]…：几件同时跑；每件的标准输出与标准错误落 $work/parallel/<批名>/<件名>.out、
    #   退出码落 <件名>.exit，这一批同时跑的最多件数落 width
    local directory="$work/parallel/$1" item item_function argument pid window=() width=0
    shift
    mkdir -p "$directory"
    rm -f -- "$directory"/*.exit
    while (( $# >= 3 )); do
      item="$1" item_function="$2" argument="$3"
      shift 3
      { "$item_function" "$argument" > "$directory/$item.out" 2>&1; echo "$?" > "$directory/$item.exit"; } &
      window+=("$!")
      (( ${#window[@]} > width )) && width=${#window[@]}
      if (( ${#window[@]} >= selftest_jobs )); then
        for pid in "${window[@]}"; do wait "$pid"; done
        window=()
      fi
    done
    for pid in ${window[@]+"${window[@]}"}; do wait "$pid"; done
    echo "$width" > "$directory/width"
  }
  collect_together() { # <批名> <件名>…：照派活的次序逐件打出它留下的输出（有的话），派出去的每一件都留下了退出码文件才算收齐，记一格；没收回的逐个点名
    local batch="$1" item missing=()
    shift
    for item in "$@"; do
      [[ -s "$work/parallel/$batch/$item.out" ]] && sed "s/^/      （$batch $item）/" "$work/parallel/$batch/$item.out"
      [[ -f "$work/parallel/$batch/$item.exit" ]] || missing+=("$item")
    done
    expect "自证并行（$batch）：派出去的件都收回了退出码" "$(( ${#missing[@]} > 0 ))" "派出去 $# 件、收回 $(( $# - ${#missing[@]} )) 件；没收回的：${missing[*]}"
  }
  exit_of() { # <批名> <件名>：打那一件的退出码（没收回打「没收回」）
    if [[ -f "$work/parallel/$1/$2.exit" ]]; then cat -- "$work/parallel/$1/$2.exit"; else echo 没收回; fi
  }
  lines_in_order() { # <文件> <正则>…：每条正则从上一条命中的下一行往后找，都找得到交 0；有找不到的，打头一条找不到的正则、交 1
    local file="$1" pattern found previous=0
    shift
    for pattern in "$@"; do
      found="$(awk -v previous="$previous" -v pattern="$pattern" 'NR > previous && $0 ~ pattern { print NR; exit }' "$file")"
      if [[ -z "$found" ]]; then
        echo "$pattern"
        return 1
      fi
      previous="$found"
    done
  }

  # 小仓的模板
  template="$work/template"
  mkdir -p "$template/.claude/gate.d" "$template/crates/tiny/src" "$template/research"
  rsync -a --exclude __pycache__ "$repository_root/.claude/scripts/" "$template/.claude/scripts/"
  cp -- "$repository_root/.claude/gate.d/crates-mutation-replay.sh" "$repository_root/.claude/gate.d/stage-inputs.tsv" "$template/.claude/gate.d/"
  cp -- "$repository_root/.claude/heavy-thread-default" "$template/.claude/"   # 算闭包开几个子进程照每台默认重活线程数的登记（scripts/hosts/heavy_threads.py）
  rsync -a --exclude __pycache__ "$repository_root/.claude/gate.d/lib/" "$template/.claude/gate.d/lib/"   # crates-mutation-replay source 的格共用库
  rsync -a --exclude __pycache__ --exclude /experiment-apparatus/target/ "$repository_root/scripts/" "$template/scripts/"   # 装置的编译目录几十 G，自证用不到
  ln -s "$repository_root/.claude/singlefs-ai-sop" "$template/.claude/singlefs-ai-sop"
  # memory_peaks_lib.py 的自证按路径载入 .claude/hooks/lib_work_root.py：场景小仓里没有它，那一格载不进。只拷这一份，不拷整个 .claude/hooks/
  mkdir -p "$template/.claude/hooks"
  cp -- "$repository_root/.claude/hooks/lib_work_root.py" "$template/.claude/hooks/"
  printf 'target/\n__pycache__/\n/multi-host.env\n/.claude/singlefs-ai-sop\n' > "$template/.gitignore"
  printf '[workspace]\nmembers = ["crates/tiny"]\n' > "$template/Cargo.toml"
  printf '[package]\nname = "tiny"\nversion = "0.1.0"\n' > "$template/crates/tiny/Cargo.toml"
  cat > "$template/crates/tiny/src/lib.rs" <<'SAMPLE_SOURCE'
// 分片那一套自证的样本 crate（假 cargo 按下面几行判；下面的 #[test] 函数只给行键认点名的测试是哪一个单元）
// TEST adds NEEDS left + right
// TEST doubles NEEDS value * 2
// TEST negates NEEDS 0 - value
// TEST halves NEEDS value / 2
// TEST triples NEEDS value * 3
// TEST squares NEEDS value * value
pub fn add(left: u32, right: u32) -> u32 { left + right }
pub fn double(value: u32) -> u32 { value * 2 }
pub fn negate(value: i32) -> i32 { 0 - value }
pub fn halve(value: u32) -> u32 { value / 2 }
pub fn triple(value: u32) -> u32 { value * 3 }
pub fn square(value: u32) -> u32 { value * value }

#[cfg(test)]
mod tests {
    #[test]
    fn adds() { assert_eq!(super::add(2, 3), 5); }
    #[test]
    fn doubles() { assert_eq!(super::double(3), 6); }
    #[test]
    fn negates() { assert_eq!(super::negate(3), -3); }
    #[test]
    fn halves() { assert_eq!(super::halve(6), 3); }
    #[test]
    fn triples() { assert_eq!(super::triple(2), 6); }
    #[test]
    fn squares() { assert_eq!(super::square(3), 9); }
}
SAMPLE_SOURCE
  {
    printf '# 每条变异的内存上限：256M\n# 每条变异的超时秒数：120\n# 变异名\t文件\t原文\t替换文\tcargo test 参数\t必须红的测试名\n'
    printf 'add-becomes-sub\tcrates/tiny/src/lib.rs\t{ left + right }\t{ left - right }\t-p tiny --lib -- adds\tadds\n'
    printf 'double-becomes-quintuple\tcrates/tiny/src/lib.rs\t{ value * 2 }\t{ value * 5 }\t-p tiny --lib -- doubles\tdoubles\n'
    printf 'negate-becomes-identity\tcrates/tiny/src/lib.rs\t{ 0 - value }\t{ value }\t-p tiny --lib -- negates\tnegates\n'
    printf 'halve-becomes-third\tcrates/tiny/src/lib.rs\t{ value / 2 }\t{ value / 3 }\t-p tiny --lib -- halves\thalves\n'
    printf 'triple-becomes-add\tcrates/tiny/src/lib.rs\t{ value * 3 }\t{ value + 3 }\t-p tiny --lib -- triples\ttriples\n'
    printf 'square-becomes-double\tcrates/tiny/src/lib.rs\t{ value * value }\t{ value + value }\t-p tiny --lib -- squares\tsquares\n'
  } > "$template/crates/mutations.tsv"
  mkdir -p "$work/fake"
  cat > "$work/fake/cargo" <<'FAKE_CARGO'
#!/usr/bin/env python3
# rocksdb-prebuilt-entry: selftest-fake-cargo（起 cargo 之前那几道 rocksdb-prebuilt.py entry 认这一行、不判）
# 分片那一套自证的假 cargo：-V 打版本；test 照 crates/tiny/src/lib.rs 里「// TEST <名> NEEDS <原文>」判，原文在就 ok、不在就 FAILED；
# 每起一次往自己目录下的 invocations.log 记「<过滤词>\t<lib.rs 的 sha256 前 12 位>」；自己目录下的 not-run-tests 点名的用例不看源码、一律 ok，带 --show-output 时再打一行 NOT_RUN_NO_GPU_ADAPTER。
# 记覆盖那两次（照 crates-mutation-rows.py 自证的假 cargo）：开头的 --config <值> 跳过；test --no-run 在 $CARGO_TARGET_DIR/debug/deps/ 下写假的可执行文件
# （JSON：副本的根与 crates/tiny/src 下的 .rs）与它的 dep-info、打一行 compiler-artifact；设了 LLVM_PROFILE_FILE 的把跑到的用例执行的函数
# （定义行里有那段原文的）写进那份 .profraw；假 llvm 工具是 crates-mutation-rows.py 的 FAKE_LLVM_TOOL
import hashlib, json, os, re, sys
here = os.path.dirname(os.path.abspath(__file__))
arguments = sys.argv[1:]
while arguments[:1] == ["--config"]:
    arguments = arguments[2:]
if arguments[:1] in (["-V"], ["--version"]):
    print("cargo 0.0.0 (mutation-shard-run selftest)")
    sys.exit(0)
if arguments[:1] != ["test"]:
    sys.exit(2)
# 自证 ⑫：每起一次 test 往自己目录下的 working-directories.log 记它的工作目录（经 crates-mutation-rows.py 起的在它 $TMPDIR 下的 singlefs-mutation-replay-* 里）
with open(os.path.join(here, "working-directories.log"), "a", encoding="utf-8") as seen:
    seen.write(os.getcwd() + "\n")
source = open("crates/tiny/src/lib.rs", encoding="utf-8").read()
if "--no-run" in arguments:
    deps = os.path.join(os.environ["CARGO_TARGET_DIR"], "debug", "deps")
    os.makedirs(deps, exist_ok=True)
    executable = os.path.join(deps, "tiny-0000000000000000")
    with open(executable, "w", encoding="utf-8") as handle:
        json.dump({"root": os.getcwd(), "sources": ["crates/tiny/src/lib.rs"]}, handle)
    with open(executable + ".d", "w", encoding="utf-8") as handle:
        handle.write(f"{executable}: crates/tiny/src/lib.rs\n\n")
    print(json.dumps({"reason": "compiler-artifact", "manifest_path": os.path.join(os.getcwd(), "crates", "tiny", "Cargo.toml"),
                      "target": {"kind": ["lib"], "name": "tiny"}, "profile": {"test": True}, "filenames": [executable], "executable": executable}))
    sys.exit(0)
after = arguments[arguments.index("--") + 1:] if "--" in arguments else []
filters = [word for word in after if not word.startswith("-")]
with open(os.path.join(here, "invocations.log"), "a", encoding="utf-8") as log:
    log.write(" ".join(filters) + "\t" + hashlib.sha256(source.encode()).hexdigest()[:12] + "\n")
failed, executed, shown = False, set(), []
definitions = re.findall(r"(?m)^pub fn (\w+).*$", source)
lines = source.splitlines()
# 自己目录下的 not-run-tests 点名的用例（一行一个）照没卡的机器上的 GPU 对拍用例：返回成功，带 --show-output 时交出一行 NOT_RUN_NO_GPU_ADAPTER
not_run_path = os.path.join(here, "not-run-tests")
not_run_names = set(open(not_run_path, encoding="utf-8").read().split()) if os.path.isfile(not_run_path) else set()
for line in lines:
    words = line.split(" ", 4)
    if len(words) < 5 or words[:2] != ["//", "TEST"]:
        continue
    name, what = words[2], words[4]
    if filters and not any(word in "tests::" + name for word in filters):
        continue
    if name in not_run_names:
        print(f"test tests::{name} ... ok")
        if "--show-output" in arguments:
            shown.append(f"---- tests::{name} stdout ----\nNOT_RUN_NO_GPU_ADAPTER test={name} reason=自证：本机没有 GPU 适配器\n")
        continue
    executed |= {match.group(1) for definition in lines for match in [re.match(r"^pub fn (\w+)", definition)] if match and what in definition}
    ok = what in source.replace(line, "")
    failed = failed or not ok
    print(f"test tests::{name} ... {'ok' if ok else 'FAILED'}")
for text in shown:
    print(text, end="")
if os.environ.get("LLVM_PROFILE_FILE"):
    with open(os.environ["LLVM_PROFILE_FILE"].replace("%p", str(os.getpid())).replace("%m", "0"), "w", encoding="utf-8") as handle:
        json.dump({"executed": sorted(executed)}, handle)
sys.exit(101 if failed else 0)
FAKE_CARGO
  chmod +x "$work/fake/cargo"
  # 假 llvm 工具：取 crates-mutation-rows.py 里那一份（FAKE_LLVM_TOOL），两个名字各一份
  mkdir -p "$work/llvm"
  python3 - "$repository_root/scripts/mutation/crates-mutation-rows.py" "$work/llvm" <<'FAKE_LLVM' || { echo "  ✗ 写不出假 llvm 工具"; echo "     → 怎么办：单跑 python3 scripts/mutation/crates-mutation-rows.py --selftest 看共用模块读不读得进来"; return 1; }
import importlib.util, os, sys
specification = importlib.util.spec_from_file_location("crates_mutation_rows_for_shard_selftest", sys.argv[1])
module = importlib.util.module_from_spec(specification)
specification.loader.exec_module(module)
for tool in module.LLVM_TOOLS:
    path = os.path.join(sys.argv[2], tool)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(module.FAKE_LLVM_TOOL)
    os.chmod(path, 0o755)
FAKE_LLVM
  local own_repository_sample_line
  own_repository_sample_line="$(sed -n 's/^STAGE_CELLS_OWN_REPOSITORY_SAMPLE_LINE="\(.*\)"$/\1/p' "$template/.claude/gate.d/lib/stage-cells.sh")"
  if [[ -z "$own_repository_sample_line" ]]; then
    echo "  ✗ 自证读不到 .claude/gate.d/lib/stage-cells.sh 的 STAGE_CELLS_OWN_REPOSITORY_SAMPLE_LINE（场景小仓的 .gate-break-sample 第一行要写它）"
    echo "     → 怎么办：看那份共用库里这一行还在不在、是不是还写成 STAGE_CELLS_OWN_REPOSITORY_SAMPLE_LINE=\"…\" 一整行；改了名字就把这里的 sed 一起改"
    return 1
  fi

  # 每一格一个场景目录：repo（被判的树，git init）、local-bin 与 peer-bin（两台 PATH 上的假 cargo）、local-home 与 peer-home（配置里两台在 BUILD_ROOT_BY_HOST 里的那一项）、配置
  prepare_scene() { # <场景名>
    local scene="$work/$1"
    mkdir -p "$scene/local-bin" "$scene/peer-bin" "$scene/local-home" "$scene/peer-home" "$scene/shared-root"
    rsync -a "$template/" "$scene/repo/"
    git -C "$scene/repo" init -q
    # 门禁共用库认这棵小仓是自证搭的（.claude/gate.d/lib/stage-cells.sh「弄坏开关只在样本根上认」）：场景里那一道就在这棵树里，
    #   .gate-break-sample 第一行写那一句接这棵树的 pwd -P，弄坏开关才进得了经 crates-mutation-replay 跑的那几格
    printf '%s%s\n' "$own_repository_sample_line" "$(cd "$scene/repo" && pwd -P)" > "$scene/repo/.gate-break-sample"
    cp -- "$work/fake/cargo" "$scene/local-bin/cargo"
    cp -- "$work/fake/cargo" "$scene/peer-bin/cargo"
    {
      echo "HOSTS=127.0.0.1 192.0.2.64"   # 主机数组（判法文件头「主机数组」）：127.0.0.1 在每一台的网卡上是自己，192.0.2.64（文档用的地址段）是第二台
      # 按台的三个键（判法文件头「按台的三个键」）两台各一项；门禁的并发数读这一台那一项的 slice 上限，写大些不压并发
      echo "CARGO_BIN_DIRECTORY_BY_HOST=127.0.0.1=$scene/local-bin 192.0.2.64=$scene/peer-bin"
      echo "BUILD_ROOT_BY_HOST=127.0.0.1=$scene/local-home 192.0.2.64=$scene/peer-home"
      echo "SLICE_MEMORY_CAP_BY_HOST=127.0.0.1=50G 192.0.2.64=50G"
      echo "ENABLE_ACROSS_MACHINES=1"
      echo "ENABLE_GPU=0"   # 两个开关都要明确写（配置判法不兜底）
      # 配置判法把它当必写的键（每段要是本机已在的目录）；crates-mutation-rows.py 的用时表与占盘记录放它点名的、不含被判的树也不在树里的头一段
      #   （run_records_directory）：写场景自己的 shared-root（上面 mkdir 过），不写含着 repo 的场景目录，不碰真共享根
      echo "NFS_SHARED_ROOTS=$scene/shared-root"
      echo "QUIESCE_STOP_COMMAND=touch $scene/quiesce-marker-selftest"
      echo "QUIESCE_STOPPED_CHECK_COMMAND=test -e $scene/quiesce-marker-selftest"
      echo "QUIESCE_START_COMMAND=rm -f $scene/quiesce-marker-selftest"
      echo "QUIESCE_STARTED_CHECK_COMMAND=test ! -e $scene/quiesce-marker-selftest"
    } > "$scene/multi-host.env"
  }
  scene_command_for() { # <场景名>：scene_command 设成在这个场景的环境里起命令的 env 前缀（PATH 先找假 cargo；crates-mutation-replay 读的变量从调用方的环境里清掉，弄坏开关留着）
    local scene="$work/$1"
    scene_command=(env -u GATE_MUTATION_ROW_SELECTION -u SINGLEFS_REUSE_HOURS -u GATE_MUTATION_WORKERS
      -u RUN_WITH_MEMORY_CAP_RESERVE
      -u GPU_QUIESCE_ORIGIN_READ_BACK -u GPU_QUIESCE_HOLDER
      -u CARGO_TARGET_DIR -u GPU_CARDS_BY_PRIORITY
      PATH="$scene/local-bin:$PATH" SINGLEFS_MULTI_HOST_CONFIG="$scene/multi-host.env"
      GATE_MUTATION_TARGET_DIR="$scene/local-target" RUN_WITH_MEMORY_CAP_PEAKS="$scene/local-peaks.tsv"
      GATE_MUTATION_LLVM_TOOLS="$work/llvm")
  }
  scene_environment() { # <场景名> <命令…>：在这个场景的环境里跑命令
    scene_command_for "$1"
    shift
    "${scene_command[@]}" "$@"
  }
  run_stage_in() { # <场景名> [变量=值…]：在这个场景的 repo 上跑 crates-mutation-replay，stdout、stderr、退出码各写场景目录一个文件
    local scene="$work/$1" exit_code
    shift
    if scene_environment "${scene##*/}" env "$@" bash "$scene/repo/.claude/gate.d/crates-mutation-replay.sh" "$scene/repo" > "$scene/stdout" 2> "$scene/stderr"; then
      exit_code=0
    else
      exit_code=$?
    fi
    echo "$exit_code" > "$scene/exit"
  }
  # 三个场景各是一串，三串同时跑（run_together scenes）：fresh 那一串是 ① ⑦ ⑨，steps 那一串是 ㉒ ㉕ ㉖ ㉔ 与 ㉗ 要的计划那一步，notrun 那一串是 ㉟ ㊱。
  #   串里有次序：⑦ ⑨ 用 ① 搭好的场景；steps 里计划那一步在前，㉕ 的合并要在两片判进库之前，两片判完才合并、核账、再点名复用的行。
  #   串里互不依赖的几件再同时跑：计划之后的 ㉒ 计划点名一行、㉒ 直接起的两趟 row-plan、㉕ 的合并、㉖ 的两片（都不往库里写）；两片判进库那两件
  #   （两台同时判同一份库就是分片的正常用法）。⑭ ⑱ ㉗ ⑫ 等三串都收回之后在这一层跑：㉗ 读 steps 那一串计划那一步留下的文件，⑫ 数三个场景留下的记录
  scene_fresh() { # ① ⑦ ⑨：场景 fresh 里照次序跑，格记进 expect-fresh.tsv
    local expect_records="$work/expect-fresh.tsv"
    # ① 没经分片判过的行，门禁不现判（核账，crates-mutation-replay.sh 文件头「分片」）：库是空的场景里直接起 crates-mutation-replay，
    #   退 1、报 6 行在判定库里没有键相同的记录，假 cargo 一次测试都没起（invocations.log 只记 test）。
    #   找那一句直接让 grep 读两份输出文件：经 cat 接管道、grep -q 匹配就退时，cat 还要往管道写第二份文件就吃 SIGPIPE，set -o pipefail 下整条管道判非零，这一格随输出的先后判错
    prepare_scene fresh
    run_stage_in fresh
    expect "① 没经分片判过的行，门禁不现判：库是空的场景里直接起 crates-mutation-replay 退 1、报 6 行在判定库里没有键相同的记录，假 cargo 一次测试都没起" \
      "$([[ "$(cat "$work/fresh/exit")" == 1 && ! -s "$work/fresh/local-bin/invocations.log" ]] && grep -q '6 行在判定库里没有键相同的记录，这一趟不现判' "$work/fresh/stdout" "$work/fresh/stderr"; echo $?)" \
      "退 $(cat "$work/fresh/exit")；假 cargo 起了 $(grep -c . "$work/fresh/local-bin/invocations.log" 2>/dev/null || echo 0) 次测试；退回的头一句：$(grep -h -m1 '^  ✗ ' "$work/fresh/stdout" "$work/fresh/stderr" | clip_text 400 | tr '\n' ' ')；stderr 尾部：$(tail -n 2 "$work/fresh/stderr" | clip_text 300 | tr '\n' ' ')"

    # ⑦ 选行模式下调驱动
    local selection_exit
    if scene_environment fresh env GATE_MUTATION_ROW_SELECTION="$work/fresh/stdout" bash "$work/fresh/repo/scripts/mutation/mutation-shard-run.sh" "$work/fresh/repo" > "$work/fresh/driver.out" 2>&1; then
      selection_exit=0
    else
      selection_exit=$?
    fi
    expect "⑦ 选行模式下调驱动：退 2 拒" \
      "$([[ "$selection_exit" == 2 ]] && grep -q '选行模式下被调了' "$work/fresh/driver.out"; echo $?)" "退 $selection_exit"

    # ⑨ 弄坏开关设在自证之外（起时去掉自证的标记，弄坏开关照调用方的再加一项）：驱动拒、退 2
    local outside_driver_exit
    if scene_environment fresh env -u MUTATION_SHARD_RUN_SELFTEST_MARK MUTATION_SHARD_RUN_BREAK="${MUTATION_SHARD_RUN_BREAK:+$MUTATION_SHARD_RUN_BREAK,}anchor-broken-dispatched" \
         bash "$work/fresh/repo/scripts/mutation/mutation-shard-run.sh" "$work/fresh/repo" > "$work/fresh/outside-driver.out" 2>&1; then
      outside_driver_exit=0
    else
      outside_driver_exit=$?
    fi
    expect "⑨ MUTATION_SHARD_RUN_BREAK 设在 --selftest 之外（不带自证的标记）：驱动拒、退 2，说设在 --selftest 之外" \
      "$([[ "$outside_driver_exit" == 2 ]] && grep -q '设在 --selftest 之外' "$work/fresh/outside-driver.out"; echo $?)" \
      "退 $outside_driver_exit；输出：$(tail -3 "$work/fresh/outside-driver.out" | tr '\n' ' ')"
  }

  # steps 那一串里同时跑的几件（run_together 起：输出各落各的文件，退出码由 run_together 记）；$steps、$steps_only_rows 是 scene_steps 的局部变量，照调用链看得到
  steps_plan_only_step() { # ㉒ 计划那一步带 --only-rows 点名一行
    scene_environment steps env TASK_SERVER_TASK=selftest-plan-only TASK_SERVER_SHARD_PHASE=plan TASK_SERVER_SHARD_PLAN_DIRECTORY="$steps/plan-only" TASK_SERVER_SHARD_MAXIMUM=64 \
      bash "$steps/repo/scripts/mutation/mutation-shard-run.sh" "$steps/repo" --only-rows "$steps_only_rows" > "$steps/plan-only.out" 2>&1
  }
  steps_merge_lost_step() { # ㉕ 两片起之前就合并
    scene_environment steps env TASK_SERVER_TASK=selftest-merge-lost TASK_SERVER_SHARD_PHASE=merge TASK_SERVER_SHARD_PLAN_DIRECTORY="$steps/plan" TASK_SERVER_SHARD_OUTPUTS="$steps/outputs-lost.tsv" \
      bash "$steps/repo/scripts/mutation/mutation-shard-run.sh" "$steps/repo" > "$steps/merge-lost.out" 2>&1
  }
  steps_no_llvm_shard_step() { # <片号>：㉖ llvm 工具目录指到一个空目录起那一片
    scene_environment steps env TASK_SERVER_TASK="selftest-shard-no-llvm-$1" TASK_SERVER_SHARD_PHASE=shard TASK_SERVER_SHARD="$1/2" GATE_MUTATION_LLVM_TOOLS="$steps/no-llvm-tools" \
      TASK_SERVER_SHARD_PLAN_DIRECTORY="$steps/plan" TASK_SERVER_SHARD_OUTPUT_DIRECTORY="$steps/output-no-llvm-$1" \
      bash "$steps/repo/scripts/mutation/mutation-shard-run.sh" "$steps/repo" > "$steps/shard-no-llvm-$1.out" 2>&1
  }
  steps_shard_step() { # <片号>：㉔ 照计划起那一片，判进服务里同一份库
    scene_environment steps env TASK_SERVER_TASK="selftest-shard-$1" TASK_SERVER_SHARD_PHASE=shard TASK_SERVER_SHARD="$1/2" \
      TASK_SERVER_SHARD_PLAN_DIRECTORY="$steps/plan" TASK_SERVER_SHARD_OUTPUT_DIRECTORY="$steps/output-$1" \
      bash "$steps/repo/scripts/mutation/mutation-shard-run.sh" "$steps/repo" > "$steps/shard-$1.out" 2>&1
  }
  steps_row_plan_two_rows_step() { # ㉒ 直接起 row-plan 点名两行：stderr 末行落 row-plan-two.stderr-tail
    local row_plan_break=()
    mutation_shard_break_is row-plan-selection-ignored && row_plan_break=(GATE_MUTATION_BREAK=rowplanselectionignored)
    scene_environment steps env ${row_plan_break[@]+"${row_plan_break[@]}"} bash -c 'python3 "$1" row-plan "$2" --rows "$3" 2>&1 >"$4" | tail -n 1' _ \
      "$steps/repo/scripts/mutation/crates-mutation-rows.py" "$steps/repo" "$work/split-only/row-plan-two-rows" "$work/split-only/row-plan-two.tsv" > "$work/split-only/row-plan-two.stderr-tail"
  }
  steps_row_plan_whole_table_step() { # ㉒ 直接起 row-plan 不点名：stderr 末行落 row-plan-whole.stderr-tail
    scene_environment steps bash -c 'python3 "$1" row-plan "$2" 2>&1 >"$3" | tail -n 1' _ \
      "$steps/repo/scripts/mutation/crates-mutation-rows.py" "$steps/repo" "$work/split-only/row-plan-whole.tsv" > "$work/split-only/row-plan-whole.stderr-tail"
  }
  scene_steps() { # ㉒ ㉕ ㉖ ㉔ 与 ㉗ 的计划那一步：场景 steps 里跑，格记进 expect-steps.tsv
    local expect_records="$work/expect-steps.tsv"
    # ㉔ 经 task-server 分片的三步（文件头「三步」）：场景里照 task-server 设的变量逐步起——计划写两片（两片都不钉，
    #   两片不相交、合起来是全部 6 行，都不带 memory_floor）；两片都判进场景那一份库（服务里同一份），各写 judged-rows；合并按此刻的行键核两片判的行在服务里读得到；
    #   之后门禁核账全部复用、判绿。另核不是服务按分片协议起的（没有 TASK_SERVER_SHARD_PHASE）：退 2、提示经 task-server 提交
    #   两片是按内存定的（文件头「三步」plan 的 ④）：场景的峰值记录这时还没有，包 tiny 按保守常量 6 GiB 算，多机配置两台都是 50G、安全额度约 33.3 GiB，
    #   每片至多 5 行（5 × 6 GiB + 1 GiB），6 行分两片；GATE_MUTATION_WORKERS=6 让 6 行开 6 个工作进程，不随调用方的线程数变
    prepare_scene steps
    local steps="$work/steps" steps_plan_exit steps_merge_exit steps_direct_exit step_index steps_shard_exits="" steps_piles steps_plan_view
    mkdir -p "$steps/plan"
    if scene_environment steps env TASK_SERVER_TASK=selftest-plan TASK_SERVER_SHARD_PHASE=plan TASK_SERVER_SHARD_PLAN_DIRECTORY="$steps/plan" TASK_SERVER_SHARD_MAXIMUM=64 GATE_MUTATION_WORKERS=6 \
         bash "$steps/repo/scripts/mutation/mutation-shard-run.sh" "$steps/repo" > "$steps/plan.out" 2>&1; then
      steps_plan_exit=0
    else
      steps_plan_exit=$?
    fi
    # 计划那一步之后同时跑的几件（文件头那一段「两个场景各是一串」）：先把它们要的输入备好
    # ㉒ 计划那一步带 --only-rows 点名一行（在两片与合并之前，那一行还要现判）：row-plan 只对那一行算键（row-plan 自己打的「算了 N 行的键」），计划只分那一行
    local steps_only_rows="$steps/only-one.selection" steps_only_plan_exit steps_only_piles
    mkdir -p "$steps/plan-only"
    printf '7\n' > "$steps_only_rows"   # 变异表前三行是表头注释：第 7 行 halve-becomes-third
    # ㉕ 合并只认服务里读得到的（文件头「三步」merge）：两片起之前就合并，另一片的 judged-rows 点名的行还没判进服务（与两台源码不同、写进了别的库同形）⇒ 合并退 1、点名那一片，
    #    出路先写查它写进了哪份库，再写比两台的源码
    local steps_lost_exit
    mkdir -p "$steps/output-lost"
    cp -- "$steps/plan/shard-1.selection" "$steps/output-lost/judged-rows" 2>/dev/null
    printf '1\tpeer\t%s\n' "$steps/output-lost" > "$steps/outputs-lost.tsv"
    # ㉖ 每一片在跑它的这一台现查 llvm 工具（文件头「三步」shard）：llvm 工具目录指到一个空目录，两片都退 78、一条变异都不起（没写 judged-rows），出路写装哪个包
    local missing_tools_exits="" missing_tools_problems=""
    mkdir -p "$steps/no-llvm-tools"
    local row_plan_selection="$work/split-only/row-plan-two-rows" row_plan_selected_stderr row_plan_whole_stderr row_plan_selected_lines row_plan_keyed_lines
    mkdir -p "$work/split-only"
    printf '4\n7\n' > "$row_plan_selection"   # 变异表前三行是表头注释：第 4 行 add-becomes-sub、第 7 行 halve-becomes-third
    run_together steps-after-plan plan-only steps_plan_only_step "" merge-lost steps_merge_lost_step "" no-llvm-0 steps_no_llvm_shard_step 0 \
      no-llvm-1 steps_no_llvm_shard_step 1 row-plan-two steps_row_plan_two_rows_step "" row-plan-whole steps_row_plan_whole_table_step ""
    collect_together steps-after-plan plan-only merge-lost no-llvm-0 no-llvm-1 row-plan-two row-plan-whole
    steps_only_plan_exit="$(exit_of steps-after-plan plan-only)"
    steps_only_piles="$(cat "$steps/plan-only"/shard-*.selection 2>/dev/null | tr '\n' ' ')"
    expect "㉒ 计划那一步带 --only-rows 点名一行：row-plan 只对那一行算键（算键 1 行，不是整张表的 6 行），计划只分那一行" \
      "$([[ "$steps_only_plan_exit" == 0 && "$steps_only_piles" == "7 " ]] && grep -q 'row-plan：算了 1 行的键（表里成形的 6 行' "$steps/plan-only.out"; echo $?)" \
      "计划那一步退 $steps_only_plan_exit；各片里的行「$steps_only_piles」；row-plan 那一行：$(grep 'row-plan：' "$steps/plan-only.out" | tail -1 | cut -c1-120)；计划那一步输出尾部：$(tail -3 "$steps/plan-only.out" | cut -c1-300 | tr '\n' ' ')"
    steps_plan_view="$(python3 -c 'import json, sys; plan = json.load(open(sys.argv[1], encoding="utf-8")); print(" ".join(("pin=" + shard["pin"] if "pin" in shard else "unpinned") + ":" + str("memory_floor" in shard) for shard in plan["shards"]))' "$steps/plan/plan.json" 2>&1)"
    steps_piles="$(cat "$steps/plan/shard-0.selection" "$steps/plan/shard-1.selection" 2>/dev/null | grep -c '[0-9]') $(cat "$steps/plan/shard-0.selection" "$steps/plan/shard-1.selection" 2>/dev/null | sort -u | grep -c '[0-9]')"
    steps_lost_exit="$(exit_of steps-after-plan merge-lost)"
    expect "㉕ 合并：另一片判的行按此刻的行键在服务里一行都读不到（写进了别的库或两台源码不同）⇒ 退 1、点名那一片，出路先写查它写进了哪份库、再写比两台的源码" \
      "$([[ "$steps_lost_exit" == 1 && -s "$steps/output-lost/judged-rows" ]] \
           && grep -q '第 1 片（peer）判的 [1-9][0-9]* 行按此刻的行键在 prune-server 里一行都读不到（先查它是不是写进了别的库：.*再比两台的源码' "$steps/merge-lost.out" \
           && grep -q '→ 怎么办：照括号里点名的那一片先查它写进了哪份库，再比两台的源码' "$steps/merge-lost.out"; echo $?)" \
      "合并退 $steps_lost_exit；输出尾部：$(tail -3 "$steps/merge-lost.out" | tr '\n' ' ')"
    for step_index in 0 1; do
      missing_tools_exits+="$(exit_of steps-after-plan "no-llvm-$step_index") "
      [[ ! -e "$steps/output-no-llvm-$step_index/judged-rows" ]] && grep -q 'rustup component add llvm-tools' "$steps/shard-no-llvm-$step_index.out" \
        || missing_tools_problems+="第 $step_index 片写了 judged-rows 或出路里没写 rustup component add llvm-tools（$(tail -2 "$steps/shard-no-llvm-$step_index.out" | tr '\n' ' ')）；"
    done
    expect "㉖ 每一片在跑它的这一台现查 llvm 工具：缺了退 78、一条变异都不起，出路写装哪个包；两片同一条规则" \
      "$([[ "$missing_tools_exits" == "78 78 " && -z "$missing_tools_problems" ]]; echo $?)" "两片退「$missing_tools_exits」；$missing_tools_problems"
    run_together steps-shards shard-0 steps_shard_step 0 shard-1 steps_shard_step 1
    collect_together steps-shards shard-0 shard-1
    steps_shard_exits="$(exit_of steps-shards shard-0) $(exit_of steps-shards shard-1) "
    printf '0\tlocal\t%s\n1\tpeer\t%s\n' "$steps/output-0" "$steps/output-1" > "$steps/outputs.tsv"
    if scene_environment steps env TASK_SERVER_TASK=selftest-merge TASK_SERVER_SHARD_PHASE=merge TASK_SERVER_SHARD_PLAN_DIRECTORY="$steps/plan" TASK_SERVER_SHARD_OUTPUTS="$steps/outputs.tsv" \
         bash "$steps/repo/scripts/mutation/mutation-shard-run.sh" "$steps/repo" > "$steps/merge.out" 2>&1; then
      steps_merge_exit=0
    else
      steps_merge_exit=$?
    fi
    run_stage_in steps
    expect "㉔ 经 task-server 分片：计划写两片（两片都不钉、片里没有 pin 键，都不带内存下限），两片合起来是全部 6 行、不相交" \
      "$([[ "$steps_plan_exit" == 0 && "$steps_plan_view" == "unpinned:False unpinned:False" && "$steps_piles" == "6 6" ]]; echo $?)" \
      "计划那一步退 $steps_plan_exit；plan.json：$steps_plan_view；两片合起来的行数与去重之后的行数：$steps_piles；输出尾部：$(tail -2 "$steps/plan.out" | tr '\n' ' ')"
    expect "㉔ 经 task-server 分片：两片各退 0、各写 judged-rows（不拷库目录），合并按此刻的行键在服务里读得到两片判的每一行" \
      "$([[ "$steps_shard_exits" == "0 0 " && -s "$steps/output-0/judged-rows" && -s "$steps/output-1/judged-rows" && ! -e "$steps/output-1/library-path" && ! -d "$steps/output-1/records-home" && "$steps_merge_exit" == 0 ]] \
         && grep -q '合并：2 片判的 [1-9][0-9]* 行里 [1-9][0-9]* 行按此刻的行键在 prune-server 里读得到、0 行读不到' "$steps/merge.out"; echo $?)" \
      "两片退「$steps_shard_exits」；合并退 $steps_merge_exit；合并输出尾部：$(tail -3 "$steps/merge.out" | tr '\n' ' ')；第二片尾部：$(tail -2 "$steps/shard-1.out" | tr '\n' ' ')"
    expect "㉔ 经 task-server 分片：之后门禁核账全部复用（复用抓到 6 条、现跑变异至多 0 条）、判绿" \
      "$([[ "$(cat "$steps/exit")" == 0 ]] \
         && grep -q '判定库记录：复用抓到 6 条.*现跑变异至多 0 条' "$steps/stderr"; echo $?)" \
      "退 $(cat "$steps/exit")；判定库记录行：$(grep '判定库记录：' "$steps/stderr" | tail -1)"
    # ㉙ ㉚ 合并那一步的分步输出（文件头「三步」merge）：照服务的合并起门禁（驱动先合并、之后核账），标准输出与标准错误并进一个文件（像任务日志）；
    #   逐行的步每过一行停 0.3 秒当假的慢读库、心跳间隔 1 秒（GATE_MUTATION_PROGRESS_ROW_PAUSE_SECONDS 只在临时 prune-server 底下认）
    local steps_merge_steps_exit steps_judged_local steps_judged_peer steps_judged_union steps_merge_steps_missing steps_heartbeat_counts steps_heartbeat_missing=""
    local -a steps_heartbeat_fields
    if scene_environment steps env TASK_SERVER_TASK=selftest-merge-steps TASK_SERVER_SHARD_PHASE=merge TASK_SERVER_SHARD_PLAN_DIRECTORY="$steps/plan" TASK_SERVER_SHARD_OUTPUTS="$steps/outputs.tsv" \
         GATE_MUTATION_PROGRESS_SECONDS=1 GATE_MUTATION_PROGRESS_ROW_PAUSE_SECONDS=0.3 \
         bash "$steps/repo/.claude/gate.d/crates-mutation-replay.sh" "$steps/repo" > "$steps/merge-steps.out" 2>&1; then
      steps_merge_steps_exit=0
    else
      steps_merge_steps_exit=$?
    fi
    steps_judged_local="$(grep -c '[0-9]' "$steps/output-0/judged-rows")"
    steps_judged_peer="$(grep -c '[0-9]' "$steps/output-1/judged-rows")"
    steps_judged_union="$(cat "$steps/output-0/judged-rows" "$steps/output-1/judged-rows" | sort -u | grep -c '[0-9]')"
    steps_merge_steps_missing="$(lines_in_order "$steps/merge-steps.out" \
      '^  · 合并·读计划：开始$' \
      "^  · 合并·读计划：完，用了 [0-9]+ 秒，2 片交了判的行，并起来去重 $steps_judged_union 行（第 0 片（local）$steps_judged_local 行、第 1 片（peer）$steps_judged_peer 行）\$" \
      "^  · 合并·现算行键：开始，共 $steps_judged_union 行\$" \
      "^  · 合并·现算行键：完，用了 [0-9]+ 秒，算了 $steps_judged_union 行的键\$" \
      "^  · 合并·读库：开始，共 $steps_judged_union 行\$" \
      "^  · 合并·读库：完，用了 [0-9]+ 秒，读到键相同的判定 $steps_judged_union 行、读不到 0 行\$" \
      '^  · 合并·逐片核对：开始$' \
      "^  · 合并·逐片核对：第 0 片（local）判的 $steps_judged_local 行，读到 $steps_judged_local 行、读不到 0 行\$" \
      "^  · 合并·逐片核对：第 1 片（peer）判的 $steps_judged_peer 行，读到 $steps_judged_peer 行、读不到 0 行\$" \
      '^  · 合并·逐片核对：完，用了 [0-9]+ 秒$' \
      '^  · 合并·核账：开始$' \
      '^  · 合并·核账·现算行键：完，用了 [0-9]+ 秒，算了 [0-9]+ 行的键$' \
      '^  · 合并·核账·读库：完，用了 [0-9]+ 秒，读到键相同的判定 [0-9]+ 行、读不到 0 行$' \
      '^  · 合并·核账：完，用了 [0-9]+ 秒，退出码 0$')" || true
    expect "㉙ 合并分步输出：读计划、现算行键、读库、逐片核对、核账各步的开始、结束行按次序都在，逐片两行的数与各片 judged-rows 的行数对，核账完退出码 0" \
      "$([[ "$steps_merge_steps_exit" == 0 && -z "$steps_merge_steps_missing" ]]; echo $?)" \
      "门禁的合并退 $steps_merge_steps_exit；按次序找不到的第一条：「${steps_merge_steps_missing:-都找到了}」；输出头尾：$(head -3 "$steps/merge-steps.out" | cut -c1-200 | tr '\n' ' ') … $(tail -3 "$steps/merge-steps.out" | cut -c1-200 | tr '\n' ' ')"
    steps_heartbeat_counts="$(awk '
      /^TASK_SERVER_HEARTBEAT 合并·现算行键：在第 [0-9]+ 行、共 [0-9]+ 行/ { row_keys++ }
      /^TASK_SERVER_HEARTBEAT 合并·读库：在第 [0-9]+ 行、共 [0-9]+ 行/ { library_reads++ }
      /^TASK_SERVER_HEARTBEAT 合并·核账·现算行键：在第 [0-9]+ 行、共 [0-9]+ 行/ { accounting_row_keys++ }
      /^TASK_SERVER_HEARTBEAT 合并·核账·读库：在第 [0-9]+ 行、共 [0-9]+ 行/ { accounting_library_reads++ }
      /在第 [0-9]+ 行、共 [0-9]+ 行/ && !/^TASK_SERVER_HEARTBEAT / { unprefixed++ }
      /^TASK_SERVER_HEARTBEAT .*(：开始|：完，)/ { prefixed_boundaries++ }
      END { printf "%d %d %d %d %d %d\n", row_keys, library_reads, accounting_row_keys, accounting_library_reads, unprefixed, prefixed_boundaries }' "$steps/merge-steps.out")"
    read -r -a steps_heartbeat_fields <<< "$steps_heartbeat_counts"
    (( ${steps_heartbeat_fields[0]:-0} > 0 )) || steps_heartbeat_missing+="合并·现算行键的心跳行 0 条；"
    (( ${steps_heartbeat_fields[1]:-0} > 0 )) || steps_heartbeat_missing+="合并·读库的心跳行 0 条；"
    (( ${steps_heartbeat_fields[2]:-0} > 0 )) || steps_heartbeat_missing+="合并·核账·现算行键的心跳行 0 条；"
    (( ${steps_heartbeat_fields[3]:-0} > 0 )) || steps_heartbeat_missing+="合并·核账·读库的心跳行 0 条；"
    (( ${steps_heartbeat_fields[4]:-1} == 0 )) || steps_heartbeat_missing+="「在第 N 行、共 M 行」有 ${steps_heartbeat_fields[4]:-?} 行不带 TASK_SERVER_HEARTBEAT 前缀；"
    (( ${steps_heartbeat_fields[5]:-1} == 0 )) || steps_heartbeat_missing+="开始或结束行有 ${steps_heartbeat_fields[5]:-?} 行带了心跳前缀；"
    expect "㉚ 合并里一步慢时有心跳行：逐行的步每过一行停 0.3 秒、间隔 1 秒，现算行键与读库（驱动的 row-plan 里与核账里各一份）有带 TASK_SERVER_HEARTBEAT 前缀的「在第 N 行、共 M 行」，开始与结束行不带这个前缀" \
      "$([[ -z "$steps_heartbeat_missing" ]]; echo $?)" "${steps_heartbeat_missing}各类心跳行的条数「${steps_heartbeat_counts}」（顺序：合并·现算行键、合并·读库、核账·现算行键、核账·读库、不带前缀的进度行、带前缀的开始或结束行）"
    # ㉗ 的计划那一步（点名的行都已判进库、复用）：库里 6 行都有键相同的判定之后，计划那一步带 --only-rows 点名第 4、6、9 行：
    #   一片都不写，这三行逐行写进 after-merge.rows（复用），成功句报复用的 3 行
    local reused_plan_exit reused_plan_rows reused_plan_shards
    mkdir -p "$steps/plan-reused"
    printf '4\n6\n9\n' > "$steps/only-reused.selection"
    if scene_environment steps env TASK_SERVER_TASK=selftest-plan-reused TASK_SERVER_SHARD_PHASE=plan TASK_SERVER_SHARD_PLAN_DIRECTORY="$steps/plan-reused" \
         bash "$steps/repo/scripts/mutation/mutation-shard-run.sh" "$steps/repo" --only-rows "$steps/only-reused.selection" > "$steps/plan-reused.out" 2>&1; then
      reused_plan_exit=0
    else
      reused_plan_exit=$?
    fi
    echo "$reused_plan_exit" > "$steps/plan-reused.exit"   # ㉗ 在这一层之外判，读它
    if scene_environment steps env -u MUTATION_SHARD_RUN_SELFTEST_MARK -u TASK_SERVER_TASK -u MUTATION_SHARD_RUN_BREAK bash "$steps/repo/scripts/mutation/mutation-shard-run.sh" "$steps/repo" > "$steps/direct.out" 2>&1; then
      steps_direct_exit=0
    else
      steps_direct_exit=$?
    fi
    expect "㉔ 不是服务按分片协议起的（没有 TASK_SERVER_SHARD_PHASE）：退 2、提示经 task-server 提交，第二台这一趟的目录没建" \
      "$([[ "$steps_direct_exit" == 2 && ! -d "$steps/peer-home/mutation-runs" ]] && grep -q 'task-server/task submit -- bash .claude/gate.d/crates-mutation-replay.sh' "$steps/direct.out"; echo $?)" \
      "退 $steps_direct_exit；输出：$(tail -2 "$steps/direct.out" | tr '\n' ' ')"

    row_plan_selected_stderr="$(cat -- "$work/split-only/row-plan-two.stderr-tail")"
    row_plan_whole_stderr="$(cat -- "$work/split-only/row-plan-whole.stderr-tail")"
    row_plan_selected_lines="$(wc -l < "$work/split-only/row-plan-two.tsv")"
    row_plan_keyed_lines="$(awk -F'\t' '$2 ~ /^[0-9a-f]{64}$/ { count++ } END { print count + 0 }' "$work/split-only/row-plan-two.tsv")"
    expect "㉒ row-plan 点名两行：只对两行算键（算键 2 行），别的 4 行只给用时、键写 -；不点名算整张 6 行" \
      "$([[ "$row_plan_selected_stderr" == *'算了 2 行的键（表里成形的 6 行'* && "$row_plan_selected_lines" == 6 && "$row_plan_keyed_lines" == 2 && "$row_plan_whole_stderr" == *'算了 6 行的键'* ]]; echo $?)" \
      "点名两行那一趟：「${row_plan_selected_stderr:0:100}」（输出 $row_plan_selected_lines 行、有键的 $row_plan_keyed_lines 行：$(head -2 "$work/split-only/row-plan-two.tsv" | cut -c1-400 | tr '\n' ' ')）；整张那一趟：「${row_plan_whole_stderr:0:100}」"
  }

  # notrun 那一串（㉟ ㊱）：计划、两片、合并三步照 task-server 设的变量走一遍，两台的假 cargo 都把第 9 行（square-becomes-double）点名的测试 squares 当没卡的机器上的 GPU 对拍用例
  #   （自己目录下的 not-run-tests 点名：返回成功、带 --show-output 打 NOT_RUN_NO_GPU_ADAPTER），那一行判成本次未跑；两片起的门禁把它记进产物目录的 not-run-record。
  #   ㉟ 合并认各片报的本次未跑：合并退 0，另起「本次未跑：1 条」逐条列，成功句报抓到几条、本次未跑几条；只判了那一行的一片同样退 0。
  #   ㊱ 没报本次未跑而行读不到的一片退 1；报了而读不出行号的也退 1、不当本次未跑
  notrun_shard_step() { # <片号>：照计划起那一片
    scene_environment notrun env TASK_SERVER_TASK="selftest-not-run-shard-$1" TASK_SERVER_SHARD_PHASE=shard TASK_SERVER_SHARD="$1/2" \
      TASK_SERVER_SHARD_PLAN_DIRECTORY="$notrun/plan" TASK_SERVER_SHARD_OUTPUT_DIRECTORY="$notrun/output-$1" \
      bash "$notrun/repo/scripts/mutation/mutation-shard-run.sh" "$notrun/repo" > "$notrun/shard-$1.out" 2>&1
  }
  notrun_merge() { # <名> <产物表>：起合并那一步，输出落 $notrun/<名>.out，在标准输出打它的退出码
    local merge_exit
    if scene_environment notrun env TASK_SERVER_TASK="selftest-not-run-merge-$1" TASK_SERVER_SHARD_PHASE=merge TASK_SERVER_SHARD_PLAN_DIRECTORY="$notrun/plan" TASK_SERVER_SHARD_OUTPUTS="$2" \
         bash "$notrun/repo/scripts/mutation/mutation-shard-run.sh" "$notrun/repo" > "$notrun/$1.out" 2>&1; then
      merge_exit=0
    else
      merge_exit=$?
    fi
    echo "$merge_exit"
  }
  scene_notrun() { # ㉟ ㊱：场景 notrun 里跑，格记进 expect-notrun.tsv
    local expect_records="$work/expect-notrun.tsv"
    prepare_scene notrun
    local notrun="$work/notrun" notrun_plan_exit notrun_shard_exits notrun_reporters="" notrun_reporter="" notrun_host="" notrun_index notrun_record_line notrun_union
    local notrun_merge_exit notrun_lonely_exit notrun_unreported_exit notrun_garbled_exit notrun_problems="" notrun_unreported_problems=""
    local notrun_row_text='crates/mutations.tsv:9 square-becomes-double：本次未跑——点名的测试 squares 打了 NOT_RUN_NO_GPU_ADAPTER（自证：本机没有 GPU 适配器）后返回成功'
    mkdir -p "$notrun/plan"
    printf 'squares\n' > "$notrun/local-bin/not-run-tests"
    printf 'squares\n' > "$notrun/peer-bin/not-run-tests"
    if scene_environment notrun env TASK_SERVER_TASK=selftest-not-run-plan TASK_SERVER_SHARD_PHASE=plan TASK_SERVER_SHARD_PLAN_DIRECTORY="$notrun/plan" TASK_SERVER_SHARD_MAXIMUM=64 GATE_MUTATION_WORKERS=6 \
         bash "$notrun/repo/scripts/mutation/mutation-shard-run.sh" "$notrun/repo" > "$notrun/plan.out" 2>&1; then
      notrun_plan_exit=0
    else
      notrun_plan_exit=$?
    fi
    run_together notrun-shards shard-0 notrun_shard_step 0 shard-1 notrun_shard_step 1
    collect_together notrun-shards shard-0 shard-1
    notrun_shard_exits="$(exit_of notrun-shards shard-0) $(exit_of notrun-shards shard-1) "
    for notrun_index in 0 1; do
      [[ -s "$notrun/output-$notrun_index/not-run-record" ]] && notrun_reporters+="$notrun_index "
    done
    notrun_reporter="${notrun_reporters%% *}"
    [[ "$notrun_reporter" == 0 ]] && notrun_host=local
    [[ "$notrun_reporter" == 1 ]] && notrun_host=peer
    notrun_record_line=""
    [[ -n "$notrun_reporter" ]] && notrun_record_line="$(grep -h -m 1 'crates/mutations.tsv:9 ' "$notrun/output-$notrun_reporter/not-run-record")"
    printf '0\tlocal\t%s\n1\tpeer\t%s\n' "$notrun/output-0" "$notrun/output-1" > "$notrun/outputs.tsv"
    notrun_merge_exit="$(notrun_merge merge "$notrun/outputs.tsv")"
    notrun_union="$(cat "$notrun/output-0/judged-rows" "$notrun/output-1/judged-rows" 2>/dev/null | sort -u | grep -c '[0-9]')"
    # 只判了那一行本次未跑的一片（一片一行、读不到的就是那一行）：报了本次未跑的退 0，没报的退 1、写读不到的那句，报的记录读不出行号的退 1
    mkdir -p "$notrun/lonely-reported" "$notrun/lonely-unreported" "$notrun/lonely-garbled"
    printf '9\n' > "$notrun/lonely-reported/judged-rows"
    printf '9\n' > "$notrun/lonely-unreported/judged-rows"
    printf '9\n' > "$notrun/lonely-garbled/judged-rows"
    printf '%s\n' "$notrun_record_line" > "$notrun/lonely-reported/not-run-record"
    printf 'crates-mutation-replay.sh 本次未跑：这一行没有变异表行号\n' > "$notrun/lonely-garbled/not-run-record"
    printf '1\tpeer\t%s\n' "$notrun/lonely-reported" > "$notrun/outputs-lonely-reported.tsv"
    printf '1\tpeer\t%s\n' "$notrun/lonely-unreported" > "$notrun/outputs-lonely-unreported.tsv"
    printf '1\tpeer\t%s\n' "$notrun/lonely-garbled" > "$notrun/outputs-lonely-garbled.tsv"
    notrun_lonely_exit="$(notrun_merge lonely-reported "$notrun/outputs-lonely-reported.tsv")"
    notrun_unreported_exit="$(notrun_merge lonely-unreported "$notrun/outputs-lonely-unreported.tsv")"
    notrun_garbled_exit="$(notrun_merge lonely-garbled "$notrun/outputs-lonely-garbled.tsv")"
    [[ "$notrun_plan_exit" == 0 && "$notrun_shard_exits" == "0 0 " ]] || notrun_problems+="计划退 $notrun_plan_exit、两片退「$notrun_shard_exits」；"
    [[ "$notrun_reporters" == "0 " || "$notrun_reporters" == "1 " ]] || notrun_problems+="产物目录里有 not-run-record 的片是「${notrun_reporters:-没有}」，要恰好一片（那一行只在一片里判）；"
    [[ "$notrun_record_line" == *"$notrun_row_text"* ]] || notrun_problems+="not-run-record 里没有第 9 行那一句（${notrun_record_line:-没找到}）；"
    [[ "$notrun_merge_exit" == 0 ]] || notrun_problems+="合并退 $notrun_merge_exit；"
    grep -qF -- "  · 本次未跑：1 条（" "$notrun/merge.out" || notrun_problems+="合并输出里没有「本次未跑：1 条」那一行；"
    grep -qF -- "    · 第 $notrun_reporter 片（$notrun_host）$notrun_row_text" "$notrun/merge.out" || notrun_problems+="合并输出里没有逐条列第 9 行（行号、变异名、那一片打的原因）；"
    grep -qF -- "✓ 分片合并：2 片判的 $notrun_union 行里 $((notrun_union - 1)) 行抓到（按此刻的行键在 prune-server 里读得到）、1 行本次未跑（逐条见上，不算抓到、不算没红）、0 行读不到；这一趟不是整张表的结论" "$notrun/merge.out" \
      || notrun_problems+="成功句没写抓到 $((notrun_union - 1)) 行、本次未跑 1 行、读不到 0 行与「这一趟不是整张表的结论」；"
    ! grep -q '一行都读不到' "$notrun/merge.out" || notrun_problems+="合并输出里有「一行都读不到」；"
    [[ "$notrun_lonely_exit" == 0 ]] && grep -qF -- "  · 本次未跑：1 条（" "$notrun/lonely-reported.out" && grep -qF -- "    · 第 1 片（peer）$notrun_row_text" "$notrun/lonely-reported.out" \
      && grep -qF -- "✓ 分片合并：1 片判的 1 行里 0 行抓到" "$notrun/lonely-reported.out" \
      || notrun_problems+="只判了第 9 行的一片（报了本次未跑）合并退 $notrun_lonely_exit、没按本次未跑列（${notrun_record_line:-没有记录}）；"
    expect "㉟ 合并认一片报的本次未跑：两片都起门禁、第 9 行点名的测试在两台上都没跑到被改坏的那一处，报了的那一片在产物目录留下 not-run-record，合并退 0、另起「本次未跑：1 条」逐条列（行号、变异名、那一片打的原因），成功句写抓到几条、本次未跑几条与「这一趟不是整张表的结论」，只判了那一行的一片同样退 0" \
      "$([[ -z "$notrun_problems" ]]; echo $?)" "${notrun_problems}合并输出尾部：$(tail -4 "$notrun/merge.out" | clip_text 400 | tr '\n' ' ')；只判那一行的一片：$(tail -2 "$notrun/lonely-reported.out" | clip_text 300 | tr '\n' ' ')"
    [[ "$notrun_unreported_exit" == 1 ]] && grep -qF -- '第 1 片（peer）判的 1 行按此刻的行键在 prune-server 里一行都读不到（先查它是不是写进了别的库' "$notrun/lonely-unreported.out" \
      && ! grep -qF -- '本次未跑：1 条' "$notrun/lonely-unreported.out" \
      || notrun_unreported_problems+="没报本次未跑、判的一行读不到的一片合并退 $notrun_unreported_exit（要退 1、点名那一片、不写本次未跑）；"
    [[ "$notrun_garbled_exit" == 1 ]] && grep -qF -- '交的本次未跑记录' "$notrun/lonely-garbled.out" && grep -qF -- '读不出行号' "$notrun/lonely-garbled.out" \
      && ! grep -qF -- '本次未跑：1 条' "$notrun/lonely-garbled.out" \
      || notrun_unreported_problems+="本次未跑记录读不出行号的一片合并退 $notrun_garbled_exit（要退 1、点名读不出行号、不写本次未跑）；"
    expect "㊱ 合并：一片没报本次未跑而判的行读不到，退 1、点名那一片，出路先写查它写进了哪份库、再写比两台的源码；报了本次未跑而记录读不出行号的也退 1，不当本次未跑" \
      "$([[ -z "$notrun_unreported_problems" ]]; echo $?)" "${notrun_unreported_problems}没报的那一趟尾部：$(tail -2 "$notrun/lonely-unreported.out" | clip_text 300 | tr '\n' ' ')；读不出行号的那一趟尾部：$(tail -2 "$notrun/lonely-garbled.out" | clip_text 300 | tr '\n' ' ')"
  }

  run_together scenes fresh scene_fresh "" steps scene_steps "" notrun scene_notrun ""
  # 弄坏开关 selftest-parallel-exit-dropped：steps 那一串的退出码文件删掉，像那一串没收回
  mutation_shard_break_is selftest-parallel-exit-dropped && rm -f -- "$work/parallel/scenes/steps.exit"
  collect_together scenes fresh steps notrun
  local steps="$work/steps" reused_plan_exit reused_plan_rows reused_plan_shards
  reused_plan_exit="$(cat -- "$steps/plan-reused.exit" 2>&1)"
  reused_plan_rows="$(tr '\t\n' ': ' < "$steps/plan-reused/after-merge.rows" 2>/dev/null)"
  reused_plan_shards="$(compgen -G "$steps/plan-reused/shard-*.selection" | wc -l)"

  # ⑭ ③ 分行：喂一份 row-plan 的输出——两行要现判、第 3 行锚点失效没键、第 4 行锚点失效而键算得出（数据文件那一类）、第 5 行拒算、第 6 行复用：
  #    pending.rows 只有第 1、2 行，汇总报拒算 1 行、锚点失效 2 行（弄坏开关 anchor-broken-dispatched 下第 4 行进了 pending.rows）
  mkdir -p "$work/split"
  printf '1\t%s\t0\t5.000\t-\n2\t%s\t0\t4.000\t-\n3\t-\t0\t-\tanchor-broken\n4\t%s\t0\t3.000\tanchor-broken\n5\t-\t0\t-\t-\n6\t%s\t1\t2.000\t-\n' \
    "$(printf 'row-one' | sha256sum | cut -d ' ' -f1)" "$(printf 'row-two' | sha256sum | cut -d ' ' -f1)" "$(printf 'row-four' | sha256sum | cut -d ' ' -f1)" \
    "$(printf 'row-six' | sha256sum | cut -d ' ' -f1)" > "$work/split/plan.tsv"
  local split_line split_pending
  split_line="$(mutation_shard_split_plan "$work/split/plan.tsv" "$work/split/pending.rows" "$work/split/after-merge.rows" "$work/split/keys")"
  split_pending="$(cut -f1 "$work/split/pending.rows" 2>/dev/null | sort -n | tr '\n' ' ')"
  expect "⑭ ③ 分行：锚点失效的行（第五列 anchor-broken，有键没键都算）不进各片、单报几行" \
    "$([[ "$split_pending" == "1 2 " && "$split_line" == "6 2 4 3.5 1 2 1 -" ]]; echo $?)" \
    "pending.rows「$split_pending」；汇总「$split_line」"

  # ⑱ 分行带选行文件：喂一份写好的 row-plan 输出（同 ⑭），选行文件点名第 2、6 行（第 6 行复用）与第 1、2 行（两行都要现判）
  mkdir -p "$work/split-only"
  printf '1\t%s\t0\t5.000\t-\n2\t%s\t0\t4.000\t-\n3\t-\t0\t-\tanchor-broken\n4\t%s\t0\t3.000\tanchor-broken\n5\t-\t0\t-\t-\n6\t%s\t1\t2.000\t-\n' \
    "$(printf 'row-one' | sha256sum | cut -d ' ' -f1)" "$(printf 'row-two' | sha256sum | cut -d ' ' -f1)" "$(printf 'row-four' | sha256sum | cut -d ' ' -f1)" \
    "$(printf 'row-six' | sha256sum | cut -d ' ' -f1)" > "$work/split-only/plan.tsv"
  printf '2\n6\n' > "$work/split-only/only-one"
  printf '1\n2\n' > "$work/split-only/only-two"
  local only_line_one only_line_two only_pending_one only_pending_two
  only_line_one="$(mutation_shard_split_plan "$work/split-only/plan.tsv" "$work/split-only/pending-one" "$work/split-only/after-merge-one" \
    "$work/split-only/keys-one" "$work/split-only/only-one")"
  only_line_two="$(mutation_shard_split_plan "$work/split-only/plan.tsv" "$work/split-only/pending-two" "$work/split-only/after-merge-two" \
    "$work/split-only/keys-two" "$work/split-only/only-two")"
  only_pending_one="$(cut -f1 "$work/split-only/pending-one" 2>/dev/null | sort -n | tr '\n' ' ')"
  only_pending_two="$(cut -f1 "$work/split-only/pending-two" 2>/dev/null | sort -n | tr '\n' ' ')"
  expect "⑱ ③ 分行带选行文件：只在选中的行里分，选中的行里要现判的进 pending.rows、复用的不进" \
    "$([[ "$only_line_one" == "2 1 4 3.5 0 0 1 -" && "$only_pending_one" == "2 " && "$only_line_two" == "2 2 4 3.5 0 0 0 -" && "$only_pending_two" == "1 2 " ]]; echo $?)" \
    "选中第 2、6 行：汇总「$only_line_one」pending.rows「$only_pending_one」；选中第 1、2 行：汇总「$only_line_two」pending.rows「$only_pending_two」"

  # ㉝ 计划那一步的 row-plan 退非 0：row-plan 的失败行打在标准输出、写进计划目录的 row-plan.tsv，失败句要带出头一条。
  #    两个假的 row-plan（python 文件，一个打一条失败行、一个什么都不打，都退 2）各在一个子 shell 里起计划那一步（子 shell 里 mutation_shard_fail 的 exit 只退这一层）
  local plan_failure="$work/plan-failure" failure_with_line failure_with_line_exit failure_without_line failure_without_line_exit
  mkdir -p "$plan_failure/with-line" "$plan_failure/without-line"
  printf '%s\n' 'import sys' 'print("  ✗ 假的 row-plan 打的失败行：selftest-row-plan-failure")' 'print("     → 怎么办：假的出路")' 'sys.exit(2)' > "$plan_failure/failing-with-line.py"
  printf '%s\n' 'import sys' 'sys.exit(2)' > "$plan_failure/failing-without-line.py"
  failure_with_line="$(rows_module="$plan_failure/failing-with-line.py" tree_root="$plan_failure" only_rows_file="" TASK_SERVER_SHARD_PLAN_DIRECTORY="$plan_failure/with-line" \
    mutation_shard_task_server_plan 2>&1)"
  failure_with_line_exit=$?
  failure_without_line="$(rows_module="$plan_failure/failing-without-line.py" tree_root="$plan_failure" only_rows_file="" TASK_SERVER_SHARD_PLAN_DIRECTORY="$plan_failure/without-line" \
    mutation_shard_task_server_plan 2>&1)"
  failure_without_line_exit=$?
  expect "㉝ 计划那一步的 row-plan 退非 0：失败句带出它打的失败行，一条都没打时写它没打失败行" \
    "$([[ "$failure_with_line_exit" == 1 && "$failure_without_line_exit" == 1 \
       && "$failure_with_line" == *"row-plan 退非 0，它打的失败行：✗ 假的 row-plan 打的失败行：selftest-row-plan-failure）"* \
       && "$failure_without_line" == *"row-plan 退非 0，它没打失败行）"* ]]; echo $?)" \
    "打了一条失败行的那一趟退 $failure_with_line_exit：「$(head -n 1 <<< "$failure_with_line" | clip_text 300)」；什么都没打的那一趟退 $failure_without_line_exit：「$(head -n 1 <<< "$failure_without_line" | clip_text 300)」"

  # ㉗ 点名 N 行：要现判的行（pending.rows）加留给合并之后的行（after-merge.rows）恰好是点名的这 N 行、互不相交。三样：
  #   喂 ⑭ 那份 row-plan 输出、点名第 1、3、4、5、6 行（要现判、锚点失效没键、锚点失效有键、拒算、复用各一）；同一份不点名（整张 6 行）；
  #   上面计划那一步在库里都有判定之后点名第 4、6、9 行：一片都不写、三行都在 after-merge.rows 里记复用、成功句报复用的 3 行
  local cover_named cover_whole cover_named_line cover_whole_line
  printf '1\n3\n4\n5\n6\n' > "$work/split/only-cover"
  cover_named_line="$(mutation_shard_split_plan "$work/split/plan.tsv" "$work/split/cover-pending" "$work/split/cover-after-merge" \
    "$work/split/cover-keys" "$work/split/only-cover")"
  cover_named="$(cut -f1 "$work/split/cover-pending" "$work/split/cover-after-merge" 2>/dev/null | sort -n | tr '\n' ' ')"
  cover_whole_line="$(mutation_shard_split_plan "$work/split/plan.tsv" "$work/split/whole-pending" "$work/split/whole-after-merge" "$work/split/whole-keys")"
  cover_whole="$(cut -f1 "$work/split/whole-pending" "$work/split/whole-after-merge" 2>/dev/null | sort -n | tr '\n' ' ')"
  expect "㉗ 点名 N 行：计划的各片加留给合并之后的行恰好是这 N 行（要现判的进各片，复用、拒算、锚点失效的进 after-merge.rows，不多不少、不相交）" \
    "$([[ "$cover_named" == "1 3 4 5 6 " && "$cover_whole" == "1 2 3 4 5 6 " && "${cover_named_line##* }" == "-" && "${cover_whole_line##* }" == "-" \
       && "$reused_plan_exit" == 0 && "$reused_plan_shards" == 0 && "$reused_plan_rows" == "4:复用 6:复用 9:复用 " ]] \
       && grep -q '复用的 3 行不分，逐行在' "$steps/plan-reused.out"; echo $?)" \
    "点名第 1、3、4、5、6 行：pending.rows 加 after-merge.rows「$cover_named」（汇总「$cover_named_line」）；不点名：「$cover_whole」（汇总「$cover_whole_line」）；库里都有判定之后点名第 4、6、9 行：计划那一步退 $reused_plan_exit、写了 $reused_plan_shards 片、after-merge.rows「$reused_plan_rows」，输出尾部：$(tail -2 "$steps/plan-reused.out" | cut -c1-300 | tr '\n' ' ')"

  # ㉘ 按内存定片数（文件头「三步」plan 的 ④；本段开头那一节 ㉘ 的说明）：小仓、峰值记录、多机配置三趟共用，每趟照计划那一步的走法
  #   （mutation_shard_split_plan → mutation_shard_plan_pending）写自己目录下的 shard-<片号>.selection 与 plan.json
  local memory="$work/memory-plan" memory_records memory_records_exit=0 memory_index
  mkdir -p "$memory/root/crates"
  git -C "$memory/root" init -q
  {
    printf '# 每条变异的内存上限：256M\n# 每条变异的超时秒数：120\n# 变异名\t文件\t原文\t替换文\tcargo test 参数\t必须红的测试名\n'
    for memory_index in $(seq 1 52); do
      printf 'case-%d\tcrates/tiny/src/lib.rs\tleft\tright\t-p tiny --lib -- case_%d\tcase_%d\n' "$memory_index" "$memory_index" "$memory_index"
    done
  } > "$memory/root/crates/mutations.tsv"
  for memory_index in $(seq 1 52); do   # 变异表前三行是表头注释：第 4 到 55 行，用时 1 到 7 秒轮着
    printf '%d\t%s\t0\t%d.000\t-\n' "$((memory_index + 3))" "$(printf 'memory-%d' "$memory_index" | sha256sum | cut -d ' ' -f1)" "$((memory_index % 7 + 1))"
  done > "$memory/plan.tsv"
  printf 'SLICE_MEMORY_CAP_BY_HOST=127.0.0.1=64G 192.0.2.64=50G\n' > "$memory/multi-host.env"
  memory_peak_records() { # <记录的底目录> <树根>：打 crates-mutation-rows.py 的 run_records_directory 照 GATE_MUTATION_RUN_RECORDS_DIRECTORY=<记录的底目录> 取的那一层（带临时服务底下的命名空间）；
    #   取不到、或取到的不在 <记录的底目录>/ 底下（没照它取：只往自证自己目录下写），打原因、退 1
    local found
    found="$(GATE_MUTATION_RUN_RECORDS_DIRECTORY="$1" python3 -c 'import importlib.util, os, sys
specification = importlib.util.spec_from_file_location("crates_mutation_rows_for_shard_memory_cell", sys.argv[1])
module = importlib.util.module_from_spec(specification)
specification.loader.exec_module(module)
print(module.run_records_directory(sys.argv[2], dict(os.environ)))' "$repository_root/scripts/mutation/crates-mutation-rows.py" "$2" 2>&1)" \
      || { tail -n 1 <<< "$found"; return 1; }
    [[ "$found" == "$1/"* ]] || { echo "取到的 $found 不在 $1/ 底下"; return 1; }
    echo "$found"
  }
  memory_records="$(memory_peak_records "$memory/records" "$memory/root")" || memory_records_exit=1
  if (( memory_records_exit == 0 )); then
    mkdir -p "$memory_records/worker-peaks/tiny/1024"
    printf '4080218932\n' > "$memory_records/worker-peaks/tiny/1024/00000000000000000001-1-selftest"   # 3.8 GiB 向上取整到字节
  fi
  # ㉜ 的小仓：变异表前三行是表头注释，第 4 到 23 行是 20 行轻的（-p tiny --lib，测试目标那一级 tiny--lib 记 2 GiB），第 24、25 行是两行整个包跑的（-p tiny，包 tiny 记 28 GiB）；
  #   轻的每行 5 秒、重的每行 300 秒
  local mixed="$work/memory-plan-mixed" mixed_records mixed_records_exit=0
  mkdir -p "$mixed/root/crates"
  git -C "$mixed/root" init -q
  {
    printf '# 每条变异的内存上限：256M\n# 每条变异的超时秒数：120\n# 变异名\t文件\t原文\t替换文\tcargo test 参数\t必须红的测试名\n'
    for memory_index in $(seq 1 20); do
      printf 'light-%d\tcrates/tiny/src/lib.rs\tleft\tright\t-p tiny --lib -- case_%d\tcase_%d\n' "$memory_index" "$memory_index" "$memory_index"
    done
    printf 'whole-%d\tcrates/tiny/src/lib.rs\tleft\tright\t-p tiny -- whole_%d\twhole_%d\n' 1 1 1 2 2 2
  } > "$mixed/root/crates/mutations.tsv"
  for memory_index in $(seq 4 25); do
    printf '%d\t%s\t0\t%d.000\t-\n' "$memory_index" "$(printf 'mixed-%d' "$memory_index" | sha256sum | cut -d ' ' -f1)" "$((memory_index < 24 ? 5 : 300))"
  done > "$mixed/plan.tsv"
  mixed_records="$(memory_peak_records "$mixed/records" "$mixed/root")" || mixed_records_exit=1
  if (( mixed_records_exit == 0 )); then
    mkdir -p "$mixed_records/worker-peaks/tiny/1024" "$mixed_records/worker-peaks/tiny--lib/1024"
    printf '%d\n' $((28 << 30)) > "$mixed_records/worker-peaks/tiny/1024/00000000000000000001-1-selftest"
    printf '%d\n' $((2 << 30)) > "$mixed_records/worker-peaks/tiny--lib/1024/00000000000000000001-1-selftest"
  fi
  memory_plan_case() { # <趟名 whole|few|capped|mixed|mixed-capped>：㉘ ㉜ 一趟，输出落 $memory/<趟名>/
    #   ㉘：whole 要现判 52 行、片数上限 64；few 要现判 3 行；capped 要现判 52 行、片数上限 4。㉜ 喂 $mixed 那一棵：mixed 片数上限 8；mixed-capped 片数上限 3
    local directory="$memory/$1" plan_rows=52 maximum=64 source="$memory" shard_summary=""
    case "$1" in
      few) plan_rows=3 ;;
      capped) maximum=4 ;;
      mixed) source="$mixed" plan_rows=22 maximum=8 ;;
      mixed-capped) source="$mixed" plan_rows=22 maximum=3 ;;
    esac
    mkdir -p "$directory"
    head -n "$plan_rows" "$source/plan.tsv" > "$directory/plan.tsv"
    export GATE_MUTATION_RUN_RECORDS_DIRECTORY="$source/records" SINGLEFS_MULTI_HOST_CONFIG="$memory/multi-host.env" GATE_MUTATION_WORKERS=31
    # 子 shell 里跑：mutation_shard_fail 的 exit 只退这一层，run_together 照样收得到退出码
    ( mutation_shard_split_plan "$directory/plan.tsv" "$directory/pending.rows" "$directory/after-merge.rows" "$directory/keys" > "$directory/split" \
        && mutation_shard_plan_pending "$repository_root/scripts/mutation/crates-mutation-rows.py" "$source/root" "$directory" "$maximum" > "$directory/plan.out" )
  }
  run_together memory-plan whole memory_plan_case whole few memory_plan_case few capped memory_plan_case capped \
    mixed memory_plan_case mixed mixed-capped memory_plan_case mixed-capped
  collect_together memory-plan whole few capped mixed mixed-capped
  local memory_budget=$(((50 << 30) * 2 / 3)) memory_peak=4080218932 memory_parent=$((1 << 30)) memory_problems="" memory_case memory_shards memory_union memory_expected memory_file memory_count memory_workers
  (( memory_records_exit == 0 )) || memory_problems+="峰值记录的目录取不到（$memory_records）；"
  for memory_case in whole few capped; do
    memory_shards="$(compgen -G "$memory/$memory_case/shard-*.selection" | wc -l)"
    memory_union="$(cat "$memory/$memory_case"/shard-*.selection 2>/dev/null | sort -n | tr '\n' ' ')"
    memory_expected="$(cut -f1 "$memory/$memory_case/plan.tsv" | sort -n | tr '\n' ' ')"
    [[ "$(exit_of memory-plan "$memory_case")" == 0 && "$memory_union" == "$memory_expected" ]] \
      || memory_problems+="$memory_case 那一趟退 $(exit_of memory-plan "$memory_case")、各片合起来 $(wc -w <<< "$memory_union") 行，要现判的是 $(wc -w <<< "$memory_expected") 行（$(tail -n 2 "$memory/$memory_case/plan.out" "$memory/$memory_case/memory-groups.err" 2>/dev/null | tr '\n' ' ' | cut -c1-300)）；"
    case "$memory_case" in
      whole)
        (( memory_shards > 2 )) || memory_problems+="要现判 52 行只写了 $memory_shards 片；"
        for memory_file in "$memory/whole"/shard-*.selection; do
          [[ -f "$memory_file" ]] || continue
          memory_count="$(grep -c . "$memory_file")"
          memory_workers=$((memory_count < 31 ? memory_count : 31))
          (( memory_workers * memory_peak + memory_parent <= memory_budget )) \
            || memory_problems+="${memory_file##*/} 有 $memory_count 行，预计 $((memory_workers * memory_peak + memory_parent)) 字节，超了安全额度 $memory_budget 字节；"
        done ;;
      few) (( memory_shards == 1 )) || memory_problems+="要现判 3 行写了 $memory_shards 片，不是一片；" ;;
      capped) (( memory_shards == 4 )) || memory_problems+="片数上限 4 时写了 $memory_shards 片，不是 4 片；" ;;
    esac
  done
  expect "㉘ 按内存定片数：要现判 52 行、每个工作进程 3.8 GiB 时各片预计内存都在安全额度（50G × 2/3）内且不止两片，要现判 3 行只一片，片数上限 4 时恰好 4 片，各片合起来恰好是要现判的行" \
    "$([[ -z "$memory_problems" ]]; echo $?)" \
    "${memory_problems}52 行那一趟：$(grep -h '④ 按内存定片数' "$memory/whole/plan.out" 2>/dev/null | cut -c1-500)"

  # ㉜ 按片估内存（文件头「三步」plan 的 ④；本段开头那一节 ㉜ 的说明；弄坏开关 shard-estimate-over-all-rows、shards-fixed-two）：
  #   轻的 20 行（tiny--lib 2 GiB：min(c, 31) × 2 GiB + 1 GiB ≤ 50G × 2/3 ⇒ 每片至多 16 行）、重的 2 行（包 tiny 28 GiB ⇒ 每片至多 1 行）。
  #   mixed（上限 8）：恰好 4 片，重的两行各单独一片，别的片只有轻的行、每片不过 16 行，轻的那一组估值写 2.0 GiB；
  #   mixed-capped（上限 3）：恰好 3 片，重的两行仍各单独一片（重的一组一片约要跑 600 秒，比轻的 20 行一片的 100 ÷ 16 秒长，多出的那一片给它），轻的 20 行一片，写比内存定的多 4 行
  local mixed_problems="" mixed_case mixed_expected mixed_shards mixed_union mixed_heavy_alone mixed_light_text
  (( mixed_records_exit == 0 )) || mixed_problems+="峰值记录的目录取不到（$mixed_records）；"
  for mixed_case in mixed mixed-capped; do
    mixed_shards="$(compgen -G "$memory/$mixed_case/shard-*.selection" | wc -l)"
    mixed_union="$(cat "$memory/$mixed_case"/shard-*.selection 2>/dev/null | sort -n | tr '\n' ' ')"
    mixed_expected="$(cut -f1 "$memory/$mixed_case/plan.tsv" | sort -n | tr '\n' ' ')"
    [[ "$(exit_of memory-plan "$mixed_case")" == 0 && "$mixed_union" == "$mixed_expected" ]] \
      || mixed_problems+="$mixed_case 那一趟退 $(exit_of memory-plan "$mixed_case")、各片合起来 $(wc -w <<< "$mixed_union") 行，要现判的是 $(wc -w <<< "$mixed_expected") 行（$(tail -n 2 "$memory/$mixed_case/plan.out" "$memory/$mixed_case/memory-groups.err" 2>/dev/null | tr '\n' ' ' | cut -c1-300)）；"
    # 重的第 24、25 行各在一片里、那一片只有它一行
    mixed_heavy_alone="$(for memory_file in "$memory/$mixed_case"/shard-*.selection; do
        [[ -f "$memory_file" ]] && grep -qx -e 24 -e 25 "$memory_file" && grep -c . "$memory_file"
      done | tr '\n' ' ')"
    [[ "$mixed_heavy_alone" == "1 1 " ]] || mixed_problems+="$mixed_case 那一趟重的两行所在的片各有「$mixed_heavy_alone」行，不是各单独一片；"
    case "$mixed_case" in
      mixed)
        (( mixed_shards == 4 )) || mixed_problems+="片数上限 8 时写了 $mixed_shards 片，不是 4 片（轻的 2 片、重的各 1 片）；"
        for memory_file in "$memory/mixed"/shard-*.selection; do
          [[ -f "$memory_file" ]] || continue
          grep -qx -e 24 -e 25 "$memory_file" && continue
          memory_count="$(grep -c . "$memory_file")"
          (( memory_count <= 16 )) || mixed_problems+="${memory_file##*/} 只有轻的行，却有 $memory_count 行，超了内存定的每片至多 16 行；"
        done
        mixed_light_text="$(grep -h '④ 测试目标·每片至多16行 那一组 20 行：' "$memory/mixed/plan.out" 2>/dev/null)"
        [[ "$mixed_light_text" == *"每个工作进程的峰值 2.0 GiB"* ]] \
          || mixed_problems+="轻的那一组的估值不是只取轻的那几行的 2.0 GiB（④ 那几行：$(grep -h '④' "$memory/mixed/plan.out" 2>/dev/null | tr '\n' ' ' | cut -c1-400)）；" ;;
      mixed-capped)
        (( mixed_shards == 3 )) || mixed_problems+="片数上限 3 时写了 $mixed_shards 片，不是 3 片；"
        grep -q '测试目标·每片至多16行 那一组每片 20 行，比内存定的每片至多 16 行多 4 行' "$memory/mixed-capped/plan.out" \
          || mixed_problems+="片数到了上限 3，没写轻的那一组每片 20 行比内存定的多 4 行；" ;;
    esac
  done
  expect "㉜ 按片估内存：轻的 20 行（2 GiB）与整个包跑的重的 2 行（28 GiB）一起要现判，重的各单独一片，轻的那一组估值只取轻的、按它定每片至多 16 行，片数上限 8 时 4 片（不被重的那一行拖到上限），上限 3 时 3 片、重的仍各单独一片" \
    "$([[ -z "$mixed_problems" ]]; echo $?)" \
    "${mixed_problems}片数上限 8 那一趟：$(grep -h '④' "$memory/mixed/plan.out" 2>/dev/null | tr '\n' ' ' | cut -c1-600)"

  # ㉛ 按编译配置与改的文件分片（文件头「三步」plan 的 ④；弄坏开关 shards-by-time-only）：mutation_shard_assign 直接喂一份造好的要现判的行与变异表：
  #   8 个编译配置（-p c<i> --lib）各 16 行（4 个文件各 4 行），第 2 个配置的行用时 100 秒、别的 10 秒；每片至多 32 行、片数上限 64 ⇒ 4 片。
  #   参照是只按用时均分的那一支（MUTATION_SHARD_ASSIGN_REFERENCE=1）：片数与每片行数上限同参照一样，各片合起来恰好是这 128 行、不相交，
  #   各片合起来的不同编译配置数不到参照的四分之三，最重的一片不超过参照最重的 1.1 倍
  local assign_directory="$work/assign-cell" assign_problems="" assign_new assign_reference
  mkdir -p "$assign_directory/new" "$assign_directory/reference"
  python3 - "$assign_directory" <<'PY_ASSIGN_DATA' || assign_problems+="造数据的 python 没跑成；"
import sys
directory = sys.argv[1]
table = ["# 变异名\t文件\t原文\t替换文\tcargo test 参数\t必须红的测试名"]
pending = []
for configuration in range(8):
    for file_index in range(4):
        for item in range(4):
            number = len(table) + 1
            table.append(f"m{number}\tcrates/c{configuration}/src/f{file_index}.rs\ta\tb\t-p c{configuration} --lib -- t{item}\tt{item}")
            pending.append((number, 100 if configuration == 1 else 10))
with open(f"{directory}/mutations.tsv", "w", encoding="utf-8") as handle:
    handle.write("\n".join(table) + "\n")
with open(f"{directory}/pending.rows", "w", encoding="utf-8") as handle:
    handle.write("".join(f"{number}\t{seconds}\n" for number, seconds in pending))
with open(f"{directory}/pending.groups", "w", encoding="utf-8") as handle:   # 全部一组、每片至多 32 行
    handle.write("".join(f"{number}\t测试目标·每片至多32行\t32\t32\n" for number, _seconds in pending))
PY_ASSIGN_DATA
  assign_new="$(mutation_shard_assign "$assign_directory/pending.rows" "$assign_directory/pending.groups" 64 "$assign_directory/new" "$assign_directory/mutations.tsv" 2>&1)" \
    || assign_problems+="分片那一段 python 没跑成（$(tail -n 1 <<< "$assign_new")）；"
  assign_reference="$(MUTATION_SHARD_ASSIGN_REFERENCE=1 mutation_shard_assign "$assign_directory/pending.rows" "$assign_directory/pending.groups" 64 "$assign_directory/reference" "$assign_directory/mutations.tsv" 2>&1)" \
    || assign_problems+="参照那一支没跑成（$(tail -n 1 <<< "$assign_reference")）；"
  assign_problems+="$(python3 - "$assign_directory" "$assign_new" "$assign_reference" <<'PY_ASSIGN_JUDGE'
import glob, sys
directory, new_summary, reference_summary = sys.argv[1:]
details = {}
with open(f"{directory}/mutations.tsv", encoding="utf-8") as handle:
    for number, line in enumerate(handle, 1):
        if not line.startswith("#"):
            details[number] = line.rstrip("\n").split("\t")[4].split(" --")[0]
seconds = {}
with open(f"{directory}/pending.rows", encoding="utf-8") as handle:
    for line in handle:
        number, value = line.split("\t")
        seconds[int(number)] = float(value)
def read(kind):
    shards = []
    for path in sorted(glob.glob(f"{directory}/{kind}/shard-*.selection")):
        with open(path, encoding="utf-8") as handle:
            shards.append([int(text) for text in handle.read().split()])
    return shards
new, reference = read("new"), read("reference")
problems = []
for name, shards in (("按编译配置分的", new), ("只按用时分的参照", reference)):
    flat = sorted(number for shard in shards for number in shard)
    if flat != sorted(seconds):
        problems.append(f"{name}各片合起来是 {len(flat)} 行，不是恰好那 {len(seconds)} 行、不相交")
    if any(len(shard) > 32 for shard in shards):
        problems.append(f"{name}有一片超过 32 行（{[len(shard) for shard in shards]}）")
if new_summary.split()[:2] != reference_summary.split()[:2]:
    problems.append(f"片数与每片行数上限同参照对不上（{new_summary.split()[:2]} 与 {reference_summary.split()[:2]}）")
def distinct(shards):
    return sum(len({details[number] for number in shard}) for shard in shards)
def heaviest(shards):
    return max(sum(seconds[number] for number in shard) for shard in shards)
if not distinct(new) * 4 < distinct(reference) * 3:
    problems.append(f"各片合起来的不同编译配置数 {distinct(new)} 不到参照 {distinct(reference)} 的四分之三")
if heaviest(new) > heaviest(reference) * 1.1:
    problems.append(f"最重的一片 {heaviest(new):.0f} 秒超过参照最重的 {heaviest(reference):.0f} 秒的 1.1 倍")
print("；".join(problems), end="")
PY_ASSIGN_JUDGE
)" || assign_problems+="判的那一段 python 没跑成；"
  expect "㉛ 按编译配置与改的文件分片：128 行（8 个编译配置各 16 行，其中一个配置的行用时是别的十倍）每片至多 32 行时，各片合起来恰好是这 128 行、不相交、片数与每片行数上限同只按用时均分的一样，各片合起来的不同编译配置数不到只按用时均分的四分之三、最重的一片不超过只按用时均分那一份最重的 1.1 倍" \
    "$([[ -z "$assign_problems" ]]; echo $?)" "${assign_problems}新分法 ${assign_new##*$'\n'}；参照 ${assign_reference##*$'\n'}"

  # ⑫ 场景跑在自证自己的 tmp/ 下、跑完不留：本机那一侧的假 cargo 在 singlefs-mutation-replay-* 里起的每一次，工作目录都在自证自己的 tmp/ 下（至少一次）；
  #   外面的临时目录里没多出 singlefs-mutation-replay-*，自证自己的 tmp/ 下也一个不剩。前一样看的是跑的时候在哪：TMPDIR 没换过来时运行目录建在外面那一层，
  #   跑完照样删掉、后两样看不出来（弄坏开关 selftest-tmpdir-outside 单开就红在这一格）；第二台那一侧的假 cargo 照第二台的 TMPDIR，不比
  local outside_after outside_new inside_left launch_logs=() launch_counts launched_total launched_outside launched_example
  outside_after="$(compgen -G "$outside_temporary/singlefs-mutation-replay-*" | sort || true)"
  outside_new="$(comm -13 <(printf '%s\n' "$outside_before" | sed '/^$/d') <(printf '%s\n' "$outside_after" | sed '/^$/d'))"
  inside_left="$(compgen -G "$work/tmp/singlefs-mutation-replay-*" || true)"
  mapfile -t launch_logs < <(compgen -G "$work/*/local-bin/working-directories.log" || true)
  launch_counts="$(MUTATION_SHARD_OWN_RUNS_PREFIX="$(cd "$work/tmp" && pwd -P)/singlefs-mutation-replay-" awk '
      index($0, "/singlefs-mutation-replay-") { total++; if (index($0, ENVIRON["MUTATION_SHARD_OWN_RUNS_PREFIX"]) != 1) { outside++; if (example == "") example = $0 } }
      END { printf "%d\t%d\t%s\n", total, outside, example }' /dev/null ${launch_logs[@]+"${launch_logs[@]}"})"
  IFS=$'\t' read -r launched_total launched_outside launched_example <<< "$launch_counts"
  expect "⑫ 场景都跑在自证自己的 tmp/ 下：跑完外面的临时目录（$outside_temporary）里没多出 singlefs-mutation-replay-*，自己的 tmp/ 下也一个不剩" \
    "$([[ "${launched_total:-0}" -gt 0 && "${launched_outside:-0}" == 0 && -z "$outside_new" && -z "$inside_left" ]]; echo $?)" \
    "本机那一侧的假 cargo 在 singlefs-mutation-replay-* 里起了 ${launched_total:-0} 次，工作目录不在自证自己的 tmp/ 下的 ${launched_outside:-0} 次（${launched_example:-没有}）；外面多出来的「$(tr '\n' ' ' <<< "$outside_new")」；自证自己的 tmp/ 里剩下的「$(tr '\n' ' ' <<< "$inside_left")」"
  # 弄坏开关下外面多出来的，是这一趟自证自己建的：照名字逐个删，不留给别人
  local outside_entry
  while IFS= read -r outside_entry; do
    [[ -n "$outside_entry" && "$outside_entry" == "$outside_temporary"/singlefs-mutation-replay-* ]] && rm -rf -- "${outside_entry:?}"
  done <<< "$outside_new"

  # 照登记次序回放各格（fresh 那一串、steps 那一串、notrun 那一串、这一层），数格数与判错数
  local records passed label detail parallel_ways=1 width_file width
  for records in "$work/expect-fresh.tsv" "$work/expect-steps.tsv" "$work/expect-notrun.tsv" "$work/expect-selftest.tsv"; do
    [[ -f "$records" ]] || continue
    while IFS=$'\t' read -r passed label detail; do
      checked=$((checked + 1))
      if [[ "$passed" == 0 ]]; then
        echo "  ✓ $label"
      else
        echo "  ✗ $label：$detail"  # gate-lint:detail
        failures=$((failures + 1))
      fi
    done < "$records"
  done
  # 并行了几路：各批同时跑的最多件数（run_together 落的 width）里的最大
  for width_file in "$work"/parallel/*/width; do
    [[ -f "$width_file" ]] || continue
    width="$(< "$width_file")"
    (( width > parallel_ways )) && parallel_ways=$width
  done
  if (( failures > 0 )); then
    echo "  ✗ mutation-shard-run.sh 自证有 $failures 格判错（共 $checked 格）"
    echo "     → 怎么办：设了 MUTATION_SHARD_RUN_BREAK 的，这就是要的结果（那个开关把对应的一步弄坏了）；没设的，照上面那几格去看驱动的计划、各片、合并三步"
    return 1
  fi
  echo "  ✓ mutation-shard-run.sh 自证 $checked 格都对，并行 $parallel_ways 路（经 task-server 分片的计划、各片、合并三步：计划按片估内存定片数（每片预计内存在安全额度内、整个包跑的重行不与轻的行同片、轻的那几片只按轻的估、行少只一片、片数不超过服务交的上限）、各片不相交且不带内存下限、两片判进服务里同一份库、合并按此刻的行键核两片判的行读得到（读不到的那一片退 1；一片自己报了本次未跑的行不算读不到、另起「本次未跑」逐条列、不让整道判红）、合并与核账分步打开始结束行、一步慢时有心跳行、之后门禁核账全部复用、判绿；没经分片判过的行门禁不现判；不是服务按分片协议起的退 2、选行模式下被调退 2、弄坏开关设在自证之外退 2；--only-rows 只对选中的行算键、只在选中的行里分；计划那一步 row-plan 退非 0 时失败句带出它打的失败行；锚点失效的行不分；点名的行要么进一堆、要么在 after-merge.rows 里留给合并之后的核账；场景跑完不留 singlefs-mutation-replay-*）"
}

if [[ "${1:-}" == --selftest ]]; then
  mutation_shard_selftest
  exit $?
fi
if [[ -n "${MUTATION_SHARD_RUN_BREAK:-}" ]] && ! mutation_shard_break_is break-honored-anywhere && ! mutation_shard_inside_selftest; then
  echo "  ✗ MUTATION_SHARD_RUN_BREAK=$MUTATION_SHARD_RUN_BREAK 设在 --selftest 之外：弄坏开关只在自证起的进程里生效，这一次什么都没分（退 2）"
  echo "     → 怎么办：去掉它再跑（env -u MUTATION_SHARD_RUN_BREAK …）；要证红就跑 MUTATION_SHARD_RUN_BREAK=<项> bash scripts/mutation/mutation-shard-run.sh --selftest"
  exit 2
fi
mutation_shard_main "$@"
