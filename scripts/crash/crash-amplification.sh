#!/usr/bin/env bash
# task-server-shards: *
# task-server-cargo-family: -p singlefs-verification --release --features gpu
# admission: always 每次调都在这一刻的树上跑全部项或点名的分项；哪些块要重判由库里的块键与判法单元指纹定，不在这里
# run-condition: command bash cargo python3
# run-condition: check source "$PREFLIGHT_SCRIPT_DIRECTORY/crash-amplification-lib.sh" && crash_amplification_run_condition_holds "$PREFLIGHT_SCRIPT_DIRECTORY/../.." :: 修好设备，或改配置：多机配置 multi-host.env 开了 GPU 而一份卡都没写、点名的卡与双机开关对不上；单跑 bash -c 'source scripts/crash/crash-amplification-lib.sh && crash_amplification_run_condition_holds' 看是哪一处
# 崩溃放量（用户 2026-09-29 定：其他的崩溃放量放在一个脚本里做、作为分项；以后一起跑，分项用参数跑；不指定参数就跑全部项）：
#
#   crash-amplification.sh                                  全部项：登记表里每个分项按次序跑一遍，展开深度只有一处（见「展开深度」）
#   crash-amplification.sh --item <分项> [--item <分项>]…    只跑点名的分项，给几次取并集（分项名照 --list-items 列的）；展开深度同上，成功句不说全部项
#   crash-amplification.sh [--item <分项>]… --crash-expand-up-to <n>
#                                                           缩域：把引擎里放量的展开上限缩到不超过 n（<n> 是非负整数，只往小缩，大于引擎常量的不放宽；见「展开深度」）；不给就是全量域。
#                                                           交给每一项的分项驱动的环境变量 SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO；环境里已设了这个变量的当作给了同一个值，两样都给取参数；
#                                                           值不是非负整数退 2、一项都不跑。缩过域的那一趟 TOTAL 行接 expand_up_to=<n>、成功句写「缩过域、不算门禁通过」
#                                                           指定范围的崩溃域上限：带了 --item、又没缩域（没给 --crash-expand-up-to，环境里也没设 SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO；
#                                                           值不小于引擎展开上限常量的不算缩域）时，开跑之前先估点名的分项这一趟要现判多少崩溃状态（只数键对不上的），超过 200000 态
#                                                           或估不出就拒：不编、不跑，退 79，说明里给两条出路——带 --crash-expand-up-to <n>（算得出就写出能压到上限以内的 n），
#                                                           或留给提交时不指定范围的那一趟。不带 --item 的不管上限。估法、记录来源、常量 SCOPED_CRASH_STATES_LIMIT 与退出码只登记在
#                                                           scripts/crash/lib_crash_point_replay_items.py（文件头「指定范围的崩溃域上限」），崩溃点重放入口共用。经服务按项分片的，这一判在计划那一步做，
#                                                           各片与合并不再判；崩溃点重放入口起这里时交 SINGLEFS_CRASH_SCOPED_DOMAIN_CHECKED=1（它已按合表项判过），这里就不再判
#   crash-amplification.sh --list-items                     逐行列出可点名的分项：分项名、测试目标、用例函数（TAB 隔开）；不清场、不跑
#   crash-amplification.sh --selftest                       用假的分项驱动走一遍第 1–66 格（指定范围的崩溃域上限是第 61–66 格：超上限判拒、点名项全部可复用放行、缩过域放行、估不出判拒、缩过域的记录不当全域、已判过的标记不再判；复用三问与上次红的清单是第 45、46 格，RocksDB 取值没判过不起是第 48 格，不在临时 prune-server 底下起的自证退 2 是第 53 格，缩域参数交变量是第 59 格、值写坏退 2 是第 60 格；列项、点名、拒绝、不清场、认不出的选项退 2、各种判红、本次未跑、复用数、派出收回、
#                                                           只供自证的变量、有红又有项退 78、按登记整批不判的数、驱动行的 red= 与不带参数时的零态、复用数是各行之和、几条流各留的块相加、
#                                                           分项驱动退 77 与它报的本次未跑、几条流里有的没借到、不带参数时一项不穷举红在那一项、
#                                                           只因登记的段不穷举不红而报段号与写数、没登段的流照旧红、往驱动送按段登记文件、登记文件写坏拒跑、
#                                                           登了段的流里另有展开上限盖不住又没登记的段红在那条流与那几段、驱动行没报那个字段照不穷举红、
#                                                           登记段分片枚举不红而报全展开与分片已判几态、分片段已判 0 态而那一行现判与复用都是 0 红在那条流与那一段、全部复用的一趟不红、分项驱动报判段没跑（step=judge）记本次未跑、
#                                                           登了段的行没报全或写坏登记段的域与分片进度照重编引擎红），
#                                                           末尾一轮从下面「弄坏开关」那张表现读开关、逐个打开，核它红在点名的那几格
#   其余参数原样交给每一项的分项驱动：[--label <标签>]；认不出的选项退 2（用法错）
#
# 崩溃点重放的合表（.claude/gate.d/crash-point-replay-items.tsv，入口 scripts/crash/crash-point-replay.sh）一项一条流，每项点名它在这里的分项（第三列）：
#   ITEMS 里每一项都要在合表里恰好被一项点名（合表那一份的判法 scripts/crash/lib_crash_point_replay_items.py 核），入口照合表逐项调这里的 --item。
# 分项登记在 ITEMS：一行「分项名 <TAB> 测试目标 <TAB> 用例函数」，一个分项就是 crates/singlefs-verification/tests/ 下一条走
# crash_amplification::run_from_environment 的用例（用例函数经分项驱动的 --test-filter 点名，cargo test 的 --exact）。每一项都是穷举、要 验证包的重活。
# 崩溃注入（crash_injection_campaign）不在表里：它是随机历史上的抽样、没有固定的点序列与树，归崩溃点重放合表里的 crash-injection-default-batch 一项。
# 全部分项的判定都在 prune-server 里那一份判定库（scripts/crash/crash-amplification-lib.sh 文件头「判定库」：引擎经客户端直接读写，这里与分项驱动都不拼库目录、不送库的变量）：
# 几条流共前缀，同一份库里前缀节点只判一次；重跑同一条命令就是续跑——同一把判法单元指纹下判过的块全部复用，判法改了旧行留作历史、这一把从头判。
# 每项交 scripts/crash/crash-amplification-item.sh 在本机跑：配置开了 GPU 就由 GPU 判，做不到那一项报错
# （用户 2026-09-29 定：「配置中一台机器就要跑一台，配置中两台机器就要跑两台，配置中有gpu就一定要让GPU工作 如果不行就明确报错拒绝，不接受默认降级」）。
# 运行条件：配置上判得了的（多机配置开了 GPU 却一份卡都没写、点名的卡与双机开关对不上）由文件头 run-condition: check 在开跑之前判（同分项驱动，判法在
#   crash-amplification-lib.sh 的 crash_amplification_run_condition_holds）；跑起来才知道的退 78：某一项的驱动退 78
#   （显卡或第二台用不上）。出路都是「修好设备，或改配置」。
# 清场与分卡由 task-server 管（records/2026-10-30-编译绕过内存总上限.md 收尾批 M03）：服务认出要卡的命令，经 scripts/crash/gpu-quiesce-run.sh 套着起这一趟、
#   经环境 GPU_CARDS_BY_PRIORITY 交它挑的卡；这里不清场、不重起自己、不挑卡，环境原样交给每一项的分项驱动。
# 经 task-server 按项分片（用户：「崩溃放量每次执行大约 30个小时……剩下的15个小时是空闲的」）：文件头「# task-server-shards: *」让服务把这一条拆成计划、各片、合并
#   （task-server/task_server_lib.py 文件头「分片」），一项一片，哪台空出来哪台接下一片；这一份照环境里的 TASK_SERVER_SHARD_PHASE 只跑其中一步：
#   plan   选好分项、核过按段登记之后，往计划目录写 rows.tsv（这一趟要跑的登记行，一行一项、次序照跑的次序）与 plan.json（一项一片，label 是分项名），退 0；不起驱动。
#   shard  照 TASK_SERVER_SHARD=<片号>/<片数> 只跑 rows.tsv 第 <片号> 行那一项，判法与输出同不分片；那一项的全量输出与驱动退出码另存进产物目录
#          （TASK_SERVER_SHARD_OUTPUT_DIRECTORY 下的 item.log、item.exit）。这一项判红（整趟退 1）或本次未跑（退 77）这一片退 0：结论交合并那一步判，
#          别的片照跑；退 78 与别的非 0 照原样退（服务照那一片的结局收尾）。
#   merge  照 rows.tsv 的次序逐项、照 TASK_SERVER_SHARD_OUTPUTS 找到那一片的产物目录，把存下的输出与退出码当成驱动的输出重放一遍（不起驱动、不编），
#          判法、ITEM 行、汇总、成功句与退出码都同不分片的那一趟。有一项找不到产物（产物表里没有那一片、或缺 item.log / item.exit）退 1、一项都不重放。
#   读完这几个变量就从环境里清掉（TASK_SERVER_SHARD_*），不传给分项驱动：别的入口在服务的分片里调到这里时，照不分片跑。
#   TASK_SERVER_SHARD_PHASE 是别的值退 2。弄坏开关 shard-phase-ignored 下不认这几个变量，三步都照不分片跑。
#   带 --no-shard 的不分（用户：「默认应该就是分片的 不分片才要加参数」）：计划写一片、不起驱动，那一片照不分片整趟跑、退出码照整趟，合并什么都不做。弄坏开关 no-shard-ignored 下照分片。
# 编译目录（用户 2026-10-23 定案「自己用完自己删」，不跨趟复用）：建好 <放量家目录>/runs/entry-<时刻>-<进程号>/ 之后，调用方没设 CARGO_TARGET_DIR 时
#   在里面建 target、export 给每一项的分项驱动（分项驱动看到设了就不另建）；正常结束、有项红、被 TERM / INT 停（等手里那一项的驱动退出）都在 EXIT 上删
#   （scripts/hosts/run-build-directory.sh；删之前核本机没有进程开着里面的文件，有就不删、报出进程，整趟退非 0）。调用方设了的照它、不删。
#
# 展开深度：全量域只有一处，引擎里放量的展开上限常量（crates/singlefs-verification/src/crash_amplification.rs，登记在 .claude/gate.d/crash-knobs.tsv）；
#   这里不写这个数；不指定范围、不缩域的那一趟不交给分项驱动任何展开深度。缩域（--crash-expand-up-to <n>）是唯一的改法：把 SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=<n>
#   交给每一项的分项驱动（环境里已设了这个变量的当作给了同一个值，参数优先），引擎取它与常量里小的那个，只往小缩。缩过域的那一趟不算门禁通过：TOTAL 行接 expand_up_to=<n>，
#   成功句写「缩过域、不算门禁通过」与实际的值，上次红的清单照写。驱动行（CRASH_AMPLIFICATION）报的 expand_up_to 是引擎用的那个数，ITEM 行照实写出；
#   每一行都要 exhaustive=true，或 exhaustive=false 只因登记的段（这一行 registered_segments= 不是 -、unregistered_segments_beyond_expansion= 是 -，见「按段登记」）；
#   有一行两样都不是（exhaustive=false 而 registered_segments= 是 -、没报或写得认不得，或 exhaustive= 没报）这一项判红、整趟退 1，
#   理由里点名不穷举又没登段的流，出路是「按段登记（.claude/gate.d/crash-point-replay-segments.tsv）或加深展开」；登了段的行另两种判红见「按段登记」。
#   取值：登记分项里最长一段的写数。今天是 30 写：多记录流与删除流（multi-record-publish、unlink-after-the-multi-record-publish）共有的第 16 段，
#   出处是第 32 批 Q 探针真跑出来的段形状 PROBE_SEGMENT_SIZES [2,2,1,2,2,1,2,24,2,1,2,28,4,1,2,30,6,1,2,26,2,1,2,24,2,1,2,22,2,1,2,20,2,1,2]
#   （删除流 35 段，前 19 段与多记录流逐段相同；records/2026-10-01-删除与并发事务.md 2026-10-08「三·5 量数」那一条）。展开 28 时这两项驱动行报 exhaustive=false，
#   2026-10-08 展开 30 跑这两项，驱动行都报 exhaustive=true expand_up_to=30。别的分项没有段形状的记录；一段不超过展开上限就展开全部真子集，
#   所以从 28 抬到 30 只改写数是 29、30 的段，不超过 28 写的段展开不变。
#   这里不现算：驱动行只报 exhaustive= 与 expand_up_to=、不报段的写数（crates/singlefs-verification/src/crash_amplification.rs 的
#   reuse_and_scale_fields_of_the_line），段形状要编、跑 验证包的用例才拿得到。新加的流或改了写表而出现更长的段，这里不会自己跟上：
#   这一项会判红（上面那一条），照出路把引擎常量改到新的最长段，或把整域展不开的段按段登记。
#
# 按段登记（判决 research/prompts/m3-crash-enumeration-merge-r3-main-verification.md 第三节 J10、H10）：表是 .claude/gate.d/crash-point-replay-segments.tsv
#   （写法在它的表头，核法 scripts/crash/lib_crash_point_replay_items.py 的 segment_problems）。选好分项之后、清场之前先核被判仓根下那一份，核不过退 2、
#   一项都不跑；核过了 export SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY=<被判仓根>/.claude/gate.d/crash-point-replay-segments.tsv（绝对路径，与
#   scripts/crash/crash-point-replay.sh 送的同一个文件），分项驱动从环境继承。
#   引擎对登记了的段按展开深度展开（写数不超过它的展开全部真子集，更长的取甲二），驱动行报 exhaustive=false registered_segments=<段号>:<写数>[,…]（没登段写 -），之后报
#   unregistered_segments_beyond_expansion=<段号>:<写数>[,…]（这条流里没登记、展开上限又盖不住的段；一段都没有写 -；引擎的 exhaustive 在登了段时一律是 false）。
#   一行 exhaustive=false 而 registered_segments= 是那种写法的（登了段的行），按 unregistered_segments_beyond_expansion= 判：
#     是 -：不因不穷举判红；
#     是 <段号>:<写数>[,…]：判红，理由点名那条流与那几段（「流 <流> 段 <段号>:<写数>、…」），出路「按段登记（.claude/gate.d/crash-point-replay-segments.tsv）或加深展开」；
#     没报或写得认不得（旧引擎打的行）：照不穷举判红，理由点名那条流，出路「引擎没报 unregistered_segments_beyond_expansion=，重编引擎」。
#   按行判，同一项别的行照判。
#   分片枚举（用户 2026-10-13 定：登记段的域装不进 u64 时按固定次序分片枚举，判定进库；不指定范围那一趟判完全部片，片长只当分批）：
#   引擎在 unregistered_segments_beyond_expansion= 之后报 registered_segment_domains=<段号>:<full|slice>[,…]（登记段的域是全展开还是分片；一段都没登写 -）
#   与 registered_segment_progress=<段号>:<已判态数>/<域的总态数>[,…]（只列 slice 段，没有写 -；两个数都是精确的十进制数字串，引擎的态数是 768 位宽整数
#   （用户 2026-10-13 定，6 个 u128 拼），可能到 140 位：这里只按 ^[0-9]+$ 认、不转成数、不做数值比较，成功句原样打这两个串）。登了段的行，不看展开深度：
#     两个字段缺一个（旧引擎）、registered_segments= 列的段有一段不在 registered_segment_domains= 里、slice 段没在 registered_segment_progress= 里、
#     进度里列了不是 slice 的段、或写法认不得：判红，理由点名那条流，出路「引擎没报 registered_segment_domains= / registered_segment_progress=，重编引擎」；
#     slice 段已判态数字面全是 0、而这一行的现判（states_checked_here=）与复用（states_reused=，没报照 0 算）都不是正整数：判红，理由点名那条流与那一段（「流 <流> 段 <段号>:0/<总>」），
#     出路「这一趟一个态都没判，看录入段与判段的日志」；现判或复用有一个是正整数的（全部复用的一趟就是现判 0、复用大于 0）不因它判红；
#     其余不因分片判红（片长只当分批：不指定范围那一趟判完全部片；缩过域的那一趟按缩过的展开判，不算门禁通过）。
#   ITEM 行写 registered_segments=：每段一条「<流>@<段号>:<写数>」（没带 flow= 的行只写「<段号>:<写数>」），逗号隔开、同一条流几行报的按段去重，一段都没有写 -；
#   之后写 unregistered_segments_beyond_expansion=：同一种写法，取各行（登没登段都算）报的这个字段；一段都没有而有一行没报这个字段（或写得认不得）写 unknown，否则写 -。
#   再写 registered_segment_domains= 与 registered_segment_progress=：取登了段的行报的这两个字段原样，每段前面加「<流>@」（没带 flow= 的行不加），同一条流几行报的按段去重；
#   一段都没有而有登了段的行没报全或写坏（上面判红的那种）写 unknown，否则写 -。
#   成功句末尾接「按段登记 N 段（K 项，只按展开深度内的域判、没判全域）：<分项> 流 <流> 段 <段号>（<写数> 写）、…，…」，一段都没有接「按段登记：0 段」；
#   再接「登记段的域 N 段（K 项；全展开 F 段、分片 S 段）：<分项> 流 <流> 段 <段号>：全展开、<分项> 流 <流> 段 <段号>：分片，已判 <已判>/<总>、…」
#   （一段都没有接「登记段的域：0 段」）；
#   再接「展开上限盖不住又没登记的段 N 段（K 项，没判全域）：<分项> 流 <流> 段 <段号>（<写数> 写）、…」，一段都没有接「展开上限盖不住又没登记的段：0 段」；
#   有项 ITEM 行写 unknown 的再接「（另有 J 项取不出：<分项、…>，驱动行没报 unregistered_segments_beyond_expansion=）」。这种段都已判红
#   （登了段的照上面，没登段的照不穷举），到不了成功句，所以成功句里这一段恒是「0 段」。
#   全量展开那几段要的态数不报（H10）。
#
# 输出：每项的驱动输出边跑边转打，同时 tee 进这一项自己的文件
#   ${SINGLEFS_CRASH_AMPLIFICATION_HOME:-<工作根>/crash-amplification}/runs/entry-<时刻>-<进程号>/<分项>.log（共用库 crash_amplification_home_directory，两台同一份），跑完从那份文件取汇总，
#   打一行 CRASH_AMPLIFICATION_ITEM item=<分项> exit=<驱动退出码> states=<现判：Σ states_checked_here> reused=<复用：这一项只取一次，取法见「复用数怎么取」，取不出写 unknown>
#     red=<Σ red> lines=<CRASH_AMPLIFICATION 行数> expand_up_to=<各行报的展开深度，各行不一或有行没报写 unknown>
#     exhaustive=<每一行都是 exhaustive=true 才写 true> verdict=<green|red|not-run> seconds=<这一项的挂钟>
#     ignored_points=<按流分组、每组只取一次的 ignored_points 之和> states_skipped_by_ignore=<同样取法的 states_skipped_by_ignore 之和>（有一行没报这两个数或不是非负整数、或同一组各行报的数不同，两个都写 unknown；见「按登记整批不判」）
#     registered_segments=<按段登记的段，写法见「按段登记」；没有写 ->
#     unregistered_segments_beyond_expansion=<展开上限盖不住又没登记的段，写法见「按段登记」；没有写 -，取不出写 unknown>
#     registered_segment_domains=<登记段的域> registered_segment_progress=<分片段的进度>（写法见「按段登记」；没有写 -，取不出写 unknown）。
#   末尾一行 CRASH_AMPLIFICATION_TOTAL dispatched=<派出几项> items=<收回几项> failed=<几项红> not_run=<几项本次未跑> reuse_unknown=<几项取不出复用数>
#     states=<Σ 现判> reused_known=<Σ 取得出的那几项的复用> red=<Σ red> scope=<whole|items> seconds=<挂钟>
#     ignored_points=<Σ 取得出的那几项> states_skipped_by_ignore=<Σ 取得出的那几项> ignore_unknown=<几项取不出这两个数（没报、写坏或同一组各行不同）>
#     [expand_up_to=<n>]（只有缩过域的那一趟接：--crash-expand-up-to 或环境里已设的缩域变量给的值，原样写出）。
#   scope：whole 没点名（全部项）；items 点了名。
#   退 0 时再打一句成功句（✓）：开头「崩溃放量全部项」（只有 scope=whole 且没有一项本次未跑才这样写）、「崩溃放量」（scope=whole、
#     有项本次未跑）、「崩溃放量点名的 K 项」；接「K 项都退 0、0 红」，有项本次未跑时接
#     「跑完的 K 项 0 红；本次未跑 J 项：<分项（原因）、…>」；
#     再接「现判 N 态、<复用那一段>；派出 D 项、收回 D 项」；再接「按登记整批不判」那一段（见下），末尾接「按段登记」那三段（按段登记的段、登记段的域、展开上限盖不住又没登记的段，见上）。
#     缩过域的那一趟，句首不写「崩溃放量全部项」，写「崩溃放量」（scope=whole）或「崩溃放量点名的 K 项」，其后紧接「（缩过域、不算门禁通过：展开深度上限缩到 <n>，只往小缩）」。
# 一项怎么判（verdict）：
#   red（计进 failed，整趟退 1），有一条就是：驱动退非 0（退 78 另见下；退 248 是算法上判不了，照这一条判红、清单里写明）；red 之和大于 0（驱动退几都算，退 78 也算）；
#     有一行缺 red= 或 red 的值不是非负整数（驱动行的约定是每行带 red=<非负整数>；驱动退几都算）；
#     驱动退 0 却 0 行 CRASH_AMPLIFICATION（出路：看测试名还在不在、cargo 是不是筛出了 0 条）；有一行不是 exhaustive=true、
#     又不是只因登记的段不穷举（出路：按段登记或加深展开；见「展开深度」「按段登记」）；登了段的行报
#     unregistered_segments_beyond_expansion= 不是 -（出路：按段登记或加深展开）或没报这个字段（出路：重编引擎；见「按段登记」）；
#     登了段的行（不看展开深度）registered_segment_domains= / registered_segment_progress= 没报全、漏段或写法认不得（出路：重编引擎），
#     或分片段已判 0 态而那一行现判与复用都是 0（出路：看录入段与判段的日志；见「按段登记」）；
#     scope=whole（不带参数）里一项现判 0 态、复用 0 态
#     （叫停或留了没判的块的项除外，那种照 not-run）。
#   not-run（本次未跑：不计进 failed，也不算通过，照 .claude/singlefs-ai-sop/rules/show-me-test.md「门禁不许假装通过」逐项报；环境里有 GATE_NOT_RUN_FILE
#     时每项往里追加一行）：不红，而有一行带 stopped_early=true、或有一行 unjudged_blocks>0（没判的块是各行之和）。入口只交分项驱动、一律本机整份跑
#     （文件头 gate-similar 那几行），收到的每一行都是本机一台判整份的那一趟打的，没有哪一行留的块该由别的行补上。
#     分项驱动退 77（一项都没跑到：排派活计划那一趟借不到显存、一块都没判，或第二台排计划那一趟借不到显存，或判段跑的过程中源码变了；
#     scripts/crash/crash-amplification-item.sh 文件头「退出码」）也是 not-run，不按「驱动退非 0」判红；原因照它打的第一行 CRASH_AMPLIFICATION_NOT_RUN
#     的 step= 与 reason= 写：step=placement（排计划那一趟）才写借不到显存，别的 step（step=judge 的跑的过程中源码变了这类）只写没判成与 reason=（没打那一行的照写没打）。
#     驱动退 0 而打了 CRASH_AMPLIFICATION_NOT_RUN 行（一条用例几条流，没借到显存的那几条各一行，借到的照判）同样是 not-run，原因写几条流没判与第一行的 reason=：
#     这一项不算通过。驱动退 0 而打了 CRASH_AMPLIFICATION_NOT_RUN step=judge 行（配置写了 ENABLE_GPU=1 而二进制没编进 gpu 特性：判段这一趟不跑、录入段照录，
#     分项驱动或双机驱动转打的那几行，一条流一行）也是这一种，原因写几条流判段没跑与第一行 step=judge 的 reason=。这几种都不按「驱动退 0 却 0 行」判红，不带参数的跑法里也不按「现判 0 态、复用 0 态」判红；行里报了红照样记 red。
#     派出的项全是 not-run 时整趟退 77。
#   驱动退 78（显卡用不上或多机配置不满足）：行里没报红（red 之和为 0、每行的 red= 都对）就记 exit=78 verdict=not-run；
#     行里报了红记 exit=78 verdict=red（判出来的红不让 78 盖掉）。两种都停在这一项、后面的项不再派。
#     整趟有红先判：别的项或这一项红就退 1，没有红才退 78；退 78 的那一项都在 ✗ 里点名。
#   green：别的。
# 复用数怎么取：登记表里每一项都走 run_from_environment 的判定库，都有判定复用这一层，没有哪一项写「不复用」。一项的复用数只取一次
#   （.claude/rules/test-script-discipline.md「判定可复用：按输入存，按前缀剪」：成功句报「现判 N、复用 M」）：
#   这一项开跑时库里这一判法单元指纹下已判、这一趟因此没判的态数。现判是各行 states_checked_here 之和（每一趟判的块不相交，照加）。
#   驱动行的 states_reused= 是那一趟（一次 run_from_environment）录入时库里已判的，叫停与只排计划的那一趟都照实报
#   （crates/singlefs-verification/src/crash_amplification.rs 的 reuse_and_scale_fields_of_the_line 的文档）。入口收到的行都是本机整份跑的，一条流一趟、
#   各趟判的块不相交，所以一项的复用数是各行的复用数之和：
#   - CRASH_AMPLIFICATION_PLACEMENT 行（只排派活计划那一趟，分项驱动原样转打）也带 states_reused=，它不是 CRASH_AMPLIFICATION 行，不取。
#   一行的复用数依次这样取：
#   ① 行里有 states_reused=<n> 就取它；
#   ② 跑完的整趟（带 states=、没有 stopped_early=true）取 states − states_checked_here：
#      fixed-script 两项的用例打这一行之前断言过「录的 + 库里这一判法单元指纹下已判的 = states」与「这一趟核的 = 录的」，差就是这一趟复用的状态数
#      （crates/singlefs-verification/tests/crash_amplification_fixed_script_through_rollback.rs、…_through_unmount.rs 打汇总行之前那几条 assert_eq）；
#   ③ 别的行取不出：flow= 那种行（tests/common_crash_points/mod.rs 的 amplify_prepared_flow）不带 states=；被叫停的行还有没判的块，差里混着没判的。
#   要取的行里有一行取不出，这一项记 reused=unknown、计进 TOTAL 行的 reuse_unknown，不记 failed、不改退出码；TOTAL 行的 reused_known 只加取得出的项。
#   成功句：都取得出写「复用 M 态」；全都取不出写「复用数取不出（J 项：<分项…>，驱动行没报 states_reused=）」；一部分取不出写
#   「复用 M 态（只算取得出的 K 项；另有 J 项取不出：<分项…>，驱动行没报 states_reused=）」。有取不出的项时不写「复用 0 态」，也不写「不复用」。
# 按登记整批不判（.claude/rules/test-script-discipline.md「判定可复用：按输入存，按前缀剪」：这一类不算剪枝，成功句报跳过了多少、哪几类）：
#   驱动行的约定是每行带 ignored_points=<测试代码里预设 ignore 的点数> states_skipped_by_ignore=<这些点的段因此没展开的态数>。这两个数是那条流的测试代码预设的，
#   不随一趟变：同一条流打的几行（叫停再续这类）报的是同一份，所以按流分组、每组只取一次（不是各行之和）：
#   - 按流分组：行里有 flow=<名> 的按名分，没有的（fixed-script 两项的汇总行）归一组；
#   - 组里各行报的两个数相同就取那一份；不同就这一项两个数写 unknown、点名那条流，不悄悄取最大或最小；
#   - 不同的流各取一次相加，一项的两个数是各组取到的数之和。
#   成功句在「按段登记」那一段之前接「按登记整批不判：N 个点、M 态」；N 或 M 大于 0 时接着写类名：驱动行只报点数与态数、不报类名，写「类名取不出」并点名有跳过的分项。
#   有项取不出（有一行没报这两个数或不是非负整数，或同一组各行报的数不同）：这一项两个数写 unknown、计进 TOTAL 行的 ignore_unknown，不红；成功句写
#   「按登记整批不判：取不出（J 项：…）」或「取得出的 K 项 N 个点、M 态；另有 J 项取不出：…」，不把取不出的记成 0；各行报的数不同的，J 项里该项后面括号点名那条流，
#   标准错误在 ITEM 行之后另打一行点名分项与流。
# 派出收回：派出的是这一趟起了驱动的项数，收回的是打出 CRASH_AMPLIFICATION_ITEM 行的项数，对不上整趟判红（command-safety.md「并行不许把失败吃掉」）。
#   某一项退 78 而停下时，后面没派的项不算派出，在 ✗ 里报出几项没派。
# 退出码：
#   0   跑到的项都不红、派出收回相等（有项本次未跑也是 0，成功句与标准错误里逐项点名）
#   1   有红（上面 red 的几条）、或收回的项数与派出的不等；有项退 78 时也是 1（有红先判，退 78 的那一项照样在 ✗ 里点名、后面的项照样不派）
#   2   用法错：参数认不得、--item 后面缺项名或项名认不得（这两种列出可点名的项）、登记表坏、分项驱动不在、
#       平常跑时带了只给自证用的变量（见「只供自证」）、按段登记文件核不过（见「按段登记」；逐处列出问题，不清场、一项都不跑）
#   77  派出的项一项都没判完（全是 not-run），也没有红
#   79  指定范围的崩溃域超上限：带了 --item、没缩域，点名的分项这一趟要现判约多少崩溃状态超过 200000 态，或估不出来（一项都没起；出路：带 --crash-expand-up-to <n> 缩域，或留给提交时不指定范围的那一趟）
#   78  运行条件不满足（出路：修好设备，或改配置）：preflight 判的（文件头两条 run-condition；含开了 GPU 而多机配置里一份卡都没写）、
#       建不了放每项输出的目录、某一项的驱动退 78（这一趟没有红时）、
#       起任何一项之前 scripts/hosts/rocksdb-prebuilt.py entry 判红（这台没装预编库、此刻进程环境的取值缺了或对不上；出路是开新会话，
#       或命令前 eval "$(python3 scripts/hosts/rocksdb-prebuilt.py env)"）
#   preflight 自己判不了条件（声明写坏）时退 1，见 .claude/singlefs-ai-sop/rules/preflight-discipline.md「不满足时」。没有别的退出码。
# 项与项之间串行（判过并行这一条路走不通）：点名的项一项一项交分项驱动，不同时起几项。理由：开了 GPU 时各项照 task-server 交的同一张卡与额度、
#   --test-threads 1 用卡（GPU 对拍的测试各占卡的显存，不并行起）。每一项里引擎照登记的线程数（SINGLEFS_CRASH_AMPLIFICATION_THREADS）并行判。
# 弄坏开关 CRASH_AMPLIFICATION_BREAK=<项>（几个用逗号连；只在自证起的子进程里认）。逐项写开关名、弄坏哪一处、打开后 --selftest 哪几格该红
#   （格号是 run_selftest 里的第 1–66 格，判错的那一格打「✗ 自证第 <号> 格：<失败行>」）。
# 写法（B 写法，样板是 scripts/mutation/mutate.sh 文件头「弄坏开关」一段）：一行「#   CRASH_AMPLIFICATION_BREAK=<开关名>  <弄坏哪一处>」，下一行起以「#       」开头
#   写点名的格「第 N 格」（、隔开；括号里是那一格失败行的句首，开关轮不读，括号外的每一段都算一格）。--selftest 末尾那一轮（开关轮）从这张表现读
#   （crash-amplification-lib.sh 的 crash_amplification_read_switch_table，与双机驱动同一份读法）：表里 CRASH_AMPLIFICATION_BREAK 的开关与代码里认的开关要相等
#   每个逐个打开、要退 1、红的格与点名的逐格相等：
#   CRASH_AMPLIFICATION_BREAK=swallow-exit  分项驱动退非 0 也记 0
#       第 8 格（「驱动退非 0 没记成红」）、第 18 格（「驱动退 78 却没停下退 78」）、第 20 格（「有红又有一项退 78 时没退 1」；这一格带出的分项驱动输出里另有一行「✗ 点名的卡用不了（假驱动）」）、
#       第 21 格（「驱动退 78 而行里报了红，这一项没记红」）、第 28 格（「照常跑完应当退 0、有项红退 1，两趟都不清场」）、
#       第 29 格（「分项驱动退 77 没记进本次未跑」）、第 46 格（「上次红的清单不对」：红的那一项没记成红）、
#       第 50 格（「跑完运行目录里不剩它建的编译目录（驱动退 1）」：这一项没记红、整趟退 0）、
#       第 54 格（「分项驱动退 77 而 NOT_RUN 行写的是跑的过程中源码变了…」：ITEM 行记成 exit=0）、
#       第 57 格（「按项分片的合并应当……」：重放那一项的退 1 被吞成 0、整趟退 0）、第 58 格（「带 --no-shard 应当计划写一片……」：那一片的退 1 被吞成 0）
#   CRASH_AMPLIFICATION_BREAK=logs-single  每一项驱动的输出只写一份全量日志（改前的样子：tee，不经 two-logs.sh，没有 .problems.log）
#       第 47 格（「每一项的输出应当出两份日志…」）
#   CRASH_AMPLIFICATION_BREAK=skip-filter  调驱动时不带 --test-filter
#       第 4 格（「全量没把每个分项各调一次驱动」）、第 6 格（「不该清场：各项应当带 --test-filter、不经清场包装重起」）
#   CRASH_AMPLIFICATION_BREAK=accept-unknown-item  不认得的分项名照跑
#       第 2 格（「不认得的分项名没有退 2 并列出可点名的项」）
#   CRASH_AMPLIFICATION_BREAK=unknown-option-accepted  认不出的选项也照收、当没给（不拒）
#       第 7 格（「认不出的选项应当用法错退 2、一项都不跑」）
#   CRASH_AMPLIFICATION_BREAK=expand-up-to-not-passed  给了 --crash-expand-up-to 不把变量交给分项驱动（TOTAL 行与成功句照写缩过域）
#       第 59 格（「给了 --crash-expand-up-to 6 应当把 SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=6 交给每一项的分项驱动…」：驱动收到的是 unset）
#   CRASH_AMPLIFICATION_BREAK=narrowed-run-unlabelled  缩过域的那一趟 TOTAL 行不接 expand_up_to=、成功句不写「缩过域、不算门禁通过」
#       第 59 格（「给了 --crash-expand-up-to 6 应当把 SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=6 交给每一项的分项驱动…」：TOTAL 行没有 expand_up_to=6、成功句没有缩过域那一句）
#   CRASH_AMPLIFICATION_BREAK=expand-up-to-unchecked  --crash-expand-up-to 的值与环境里的变量不核是不是非负整数
#       第 60 格（「--crash-expand-up-to 的值不是非负整数时应当退 2、一项都不跑」）
#   CRASH_AMPLIFICATION_BREAK=scoped-domain-unbounded  带 --item、没缩域也不判指定范围的崩溃域上限（改前的样子）
#       第 61 格（「估数超过 200000 态应当退 79、一项都不起」）、第 64 格（「估不出应当退 79」）、第 65 格（「只有缩过域的记录算不出全域态数，应当退 79」）
#   CRASH_AMPLIFICATION_BREAK=scoped-domain-narrowed-refused  缩了域（--crash-expand-up-to 小于引擎常量）也照没缩判：缩域值不交给判法
#       第 63 格（「缩过域的放行，不读库、不判上限」）
#   CRASH_AMPLIFICATION_BREAK=scoped-domain-reusable-counted  库里录下的块键都对得上、全部可复用的分项也按上一趟全域真跑的态数算（上一趟有记录才算）
#       第 62 格（「点名项全部可复用的放行，不论上一趟的态数多大」）
#   CRASH_AMPLIFICATION_BREAK=scoped-domain-unknown-passed  估不出现判多少态的分项按 0 态放行
#       第 64 格（「估不出应当退 79」）、第 65 格（「只有缩过域的记录算不出全域态数，应当退 79」）
#   CRASH_AMPLIFICATION_BREAK=scoped-domain-suggestion-dropped  拒绝说明里不写算得出的 n
#       第 61 格（「说明里应当写能压到上限以内的 --crash-expand-up-to 6」）
#   CRASH_AMPLIFICATION_BREAK=scoped-domain-shrunk-records-used  估上一趟全域态数时不看展开深度，取最新的一份记录（缩过域的那趟也当全域）
#       第 65 格（「只有缩过域的记录算不出全域态数，应当退 79」）
#   CRASH_AMPLIFICATION_BREAK=scoped-domain-handshake-ignored  崩溃点重放入口交了 SINGLEFS_CRASH_SCOPED_DOMAIN_CHECKED=1 也照判一遍
#       第 66 格（「收到已判过的标记就不再判上限」）
#   CRASH_AMPLIFICATION_BREAK=reuse-as-zero  取得出的复用数也记 0
#       第 10 格（「复用数取错」）、第 26 格（「复用数没照各行之和取」）、第 27 格（「几条流各留了块，没判的块没照各行之和报本次未跑」：ITEM 行复用写成 0）、
#       第 43 格（「全部复用的一趟…判了红、或 ITEM 行没照实写现判 3 复用 7」：ITEM 行复用写成 0）、第 45 格（「复用三问没过」：键相同的一趟 ITEM 行写复用 0，折成现判）
#   CRASH_AMPLIFICATION_BREAK=accept-unknown-reuse  取不出复用数的项记成 reused=0、不计进 reuse_unknown、成功句不点名
#       第 11 格（「取不出复用数的项没照实报出」）、第 12 格（「叫停的行被拿来推复用数」）
#   CRASH_AMPLIFICATION_BREAK=reuse-from-stopped-line  叫停的行（stopped_early=true）没报 states_reused= 也拿 states 减 states_checked_here 当复用数
#       第 12 格（「叫停的行被拿来推复用数」）
#   CRASH_AMPLIFICATION_BREAK=skip-collection-count  不核派出收回
#       第 13 格（「少收回一项却没判红」）
#   CRASH_AMPLIFICATION_BREAK=ignore-red-count  不看 red 之和（驱动退 0 与退 78 都不看）
#       第 9 格（「驱动退 0 而 red=2 没记成红」）、第 20 格（「有红又有一项退 78 时没退 1」；这一格带出的分项驱动输出里另有一行「✗ 点名的卡用不了（假驱动）」）、第 21 格（「驱动退 78 而行里报了红，这一项没记红」）
#   CRASH_AMPLIFICATION_BREAK=accept-zero-lines  驱动退 0、0 行 CRASH_AMPLIFICATION 不判红
#       第 14 格（「驱动退 0、0 行却没判红」）
#   CRASH_AMPLIFICATION_BREAK=accept-not-exhaustive  有行不是穷举也不判红
#       第 15 格（「有行不是穷举却没判红」）、第 31 格（「不带参数时一项不穷举，没有红在那一项」）、
#       第 33 格（「一行 exhaustive=false registered_segments=- 没照旧判红在那一条流」）、
#       第 36 格（「登了段的流里另有展开上限盖不住又没登记的段，没判红在那条流与那几段」）、
#       第 37 格（「登了段的行没报 unregistered_segments_beyond_expansion=，没照不穷举判红、出路不是重编引擎」）
#   CRASH_AMPLIFICATION_BREAK=skip-not-exhaustive-in-whole-scope  不带参数（scope=whole）时不判不穷举，点名的照判
#       第 31 格（「不带参数时一项不穷举，没有红在那一项」）
#   CRASH_AMPLIFICATION_BREAK=count-stopped-as-run  叫停与留了没判的块的项也记 green
#       第 16 格（「被叫停的项没按本次未跑逐项报」）、第 17 格（「派出的项全没判完却没退 77」）、第 27 格（「几条流各留了块，没判的块没照各行之和报本次未跑」）
#   CRASH_AMPLIFICATION_BREAK=item-78-as-failure  驱动退 78 照普通失败记、接着派后面的项
#       第 18 格（「驱动退 78 却没停下退 78」）、第 20 格（「有红又有一项退 78 时没退 1」；这一格带出的分项驱动输出里另有一行「✗ 点名的卡用不了（假驱动）」）、第 21 格（「驱动退 78 而行里报了红，这一项没记红」）
#   CRASH_AMPLIFICATION_BREAK=honor-test-variables-outside-selftest  平常跑时也认只供自证的变量，不拒；关守卫项，整值比，只在它是 CRASH_AMPLIFICATION_BREAK 唯一一项时认，与别的项同写平常跑照拒并点名
#       第 19 格（「平常跑时带了只供自证的变量却没拒」）
#   CRASH_AMPLIFICATION_BREAK=condition-before-red  有红又有一项驱动退 78 时整趟退 78（不让红优先）
#       第 20 格（「有红又有一项退 78 时没退 1」；这一格带出的分项驱动输出里另有一行「✗ 点名的卡用不了（假驱动）」）、第 21 格（「驱动退 78 而行里报了红，这一项没记红」）
#   CRASH_AMPLIFICATION_BREAK=ignore-red-on-78  驱动退 78 时不看行里的红，照本次未跑记
#       第 21 格（「驱动退 78 而行里报了红，这一项没记红」）
#   CRASH_AMPLIFICATION_BREAK=skip-ignore-report  成功句不报按登记整批不判的点数与态数
#       第 22 格（「按登记整批不判的点数与态数没照实写进 ITEM、TOTAL 与成功句」）、第 23 格（「驱动行没报按登记整批不判的两个数，却没照实报取不出」）、
#       第 42 格（「按登记不判的点数与态数没按流分组、每组只取一次」：成功句没报）
#   CRASH_AMPLIFICATION_BREAK=ignore-unknown-as-zero  驱动行没报 ignored_points= / states_skipped_by_ignore= 的记成 0、不点名
#       第 23 格（「驱动行没报按登记整批不判的两个数，却没照实报取不出」）
#   CRASH_AMPLIFICATION_BREAK=accept-bad-red-field  有行缺 red= 或 red 不是非负整数也不判红
#       第 24 格（「驱动行缺 red= 或 red 不是非负整数却没判红」）
#   CRASH_AMPLIFICATION_BREAK=accept-zero-whole-item  不带参数时一项现判 0 态、复用 0 态也不判红
#       第 25 格（「不带参数时一项现判 0 态、复用 0 态却没判红」）
#   CRASH_AMPLIFICATION_BREAK=driver-not-run-as-zero-lines  不认分项驱动的 CRASH_AMPLIFICATION_NOT_RUN 行（退 0 而 0 行的照「驱动退 0 却 0 行」判红）
#       第 29 格（「分项驱动退 77 没记进本次未跑」：原因里没有那一行的 reason=）、第 30 格（「几条流里有的没借到显存，这一项却记了通过」）、
#       第 44 格（「分项驱动报判段没跑…的那一项没记本次未跑」）、第 54 格（「分项驱动退 77 而 NOT_RUN 行写的是跑的过程中源码变了…」：原因写成了没打那一行）
#   CRASH_AMPLIFICATION_BREAK=driver-77-as-failure  分项驱动退 77 照别的非 0 退出码判红（改前的判法）
#       第 29 格（「分项驱动退 77 没记进本次未跑」）、第 54 格（「分项驱动退 77 而 NOT_RUN 行写的是跑的过程中源码变了…」：记成了红）
#   CRASH_AMPLIFICATION_BREAK=judge-not-run-ignored  不认分项驱动转打的 CRASH_AMPLIFICATION_NOT_RUN step=judge 行（退 0 而 0 行的照「驱动退 0 却 0 行」判红）
#       第 44 格（「分项驱动报判段没跑…的那一项没记本次未跑」）、第 54 格（「分项驱动退 77 而 NOT_RUN 行写的是跑的过程中源码变了…」：原因写成了没打那一行）
#   CRASH_AMPLIFICATION_BREAK=not-run-cause-always-vram  分项驱动退 77 而打了 CRASH_AMPLIFICATION_NOT_RUN 行时，原因前半句不看 step= 一律写借不到显存（改前的写法）
#       第 54 格（「分项驱动退 77 而 NOT_RUN 行写的是跑的过程中源码变了…」）
#   CRASH_AMPLIFICATION_BREAK=registered-segments-ignored  不认驱动行的 registered_segments=（改前的判法：exhaustive=false 一律算不穷举）
#       第 32 格（「一行 exhaustive=false 只因登记的段…这一项却判了红」）、第 33 格（「一行 exhaustive=false registered_segments=- 没照旧判红在那一条流」）、
#       第 36 格（「登了段的流里另有展开上限盖不住又没登记的段，没判红在那条流与那几段」）、
#       第 37 格（「登了段的行没报 unregistered_segments_beyond_expansion=，没照不穷举判红、出路不是重编引擎」）、
#       第 38 格（「登记段分片枚举…判了红、或 ITEM 行没带…或成功句没报全展开与分片已判几态」）、
#       第 39 格（「分片段这一趟已判 0 态却没判红在那条流与那一段」）、
#       第 40 格（「登了段的行 registered_segment_domains= / registered_segment_progress= 缺了或写法认不得，没判红在那条流」）、
#       第 41 格（「registered_segment_domains= 漏了登记的段、或分片段没在 registered_segment_progress= 里，没判红」）、
#       第 43 格（「全部复用的一趟…判了红」：登记段不认、stub 那条照不穷举红）
#   CRASH_AMPLIFICATION_BREAK=registered-excuses-the-whole-item  一项里有一行登了段，这一项 exhaustive=false 的行就都不算不穷举（不按行判）
#       第 33 格（「一行 exhaustive=false registered_segments=- 没照旧判红在那一条流」）
#   CRASH_AMPLIFICATION_BREAK=dash-counts-as-registered  registered_segments=- 也算登了段
#       第 33 格（「一行 exhaustive=false registered_segments=- 没照旧判红在那一条流」）
#   CRASH_AMPLIFICATION_BREAK=registered-segments-unreported  ITEM 行的 registered_segments= 写 -、成功句不报按段登记那一段
#       第 32 格（「一行 exhaustive=false 只因登记的段…这一项却判了红」）、第 33 格（「一行 exhaustive=false registered_segments=- 没照旧判红在那一条流」）、
#       第 36 格（「登了段的流里另有展开上限盖不住又没登记的段，没判红在那条流与那几段」）、
#       第 37 格（「登了段的行没报 unregistered_segments_beyond_expansion=，没照不穷举判红、出路不是重编引擎」）、
#       第 38 格（「登记段分片枚举…判了红、或 ITEM 行没带…或成功句没报全展开与分片已判几态」）
#   CRASH_AMPLIFICATION_BREAK=segments-registry-not-sent  不往分项驱动送 SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY（改前的样子）
#       第 34 格（「分项驱动没收到按段登记文件的绝对路径」）
#   CRASH_AMPLIFICATION_BREAK=segments-registry-unchecked  开跑前不核按段登记文件
#       第 35 格（「按段登记文件写坏了却没整趟拒跑」）
#   CRASH_AMPLIFICATION_BREAK=beyond-expansion-ignored  不按 unregistered_segments_beyond_expansion= 判（改前的判法：登了段的行一律不算不穷举）
#       第 36 格（「登了段的流里另有展开上限盖不住又没登记的段，没判红在那条流与那几段」）、
#       第 37 格（「登了段的行没报 unregistered_segments_beyond_expansion=，没照不穷举判红、出路不是重编引擎」）
#   CRASH_AMPLIFICATION_BREAK=beyond-expansion-missing-as-dash  驱动行没报 unregistered_segments_beyond_expansion= 当成报了 -（ITEM 行写 - 不写 unknown，判法上当成没有展不开的段）
#       第 22 格（「按登记整批不判的点数与态数没照实写进 ITEM、TOTAL 与成功句」）、第 23 格（「驱动行没报按登记整批不判的两个数，却没照实报取不出」）、第 26 格（「复用数没照各行之和取」）、
#       第 27 格（「几条流各留了块，没判的块没照各行之和报本次未跑」）、第 32 格（「一行 exhaustive=false 只因登记的段…这一项却判了红」）、
#       第 36 格（「登了段的流里另有展开上限盖不住又没登记的段，没判红在那条流与那几段」）、
#       第 37 格（「登了段的行没报 unregistered_segments_beyond_expansion=，没照不穷举判红、出路不是重编引擎」）
#   CRASH_AMPLIFICATION_BREAK=beyond-expansion-unreported  ITEM 行的 unregistered_segments_beyond_expansion= 写 -、成功句不报展开上限盖不住又没登记的段
#       第 22 格（「按登记整批不判的点数与态数没照实写进 ITEM、TOTAL 与成功句」）、第 23 格（「驱动行没报按登记整批不判的两个数，却没照实报取不出」）、第 26 格（「复用数没照各行之和取」）、
#       第 27 格（「几条流各留了块，没判的块没照各行之和报本次未跑」）、第 32 格（「一行 exhaustive=false 只因登记的段…这一项却判了红」）、第 33 格（「一行 exhaustive=false registered_segments=- 没照旧判红在那一条流」）、
#       第 36 格（「登了段的流里另有展开上限盖不住又没登记的段，没判红在那条流与那几段」）、
#       第 37 格（「登了段的行没报 unregistered_segments_beyond_expansion=，没照不穷举判红、出路不是重编引擎」）、
#       第 38 格（「登记段分片枚举…判了红、或 ITEM 行没带…或成功句没报全展开与分片已判几态」）
#   CRASH_AMPLIFICATION_BREAK=slice-progress-zero-accepted  分片段已判 0 态也不判红
#       第 39 格（「分片段这一趟已判 0 态却没判红在那条流与那一段」）
#   CRASH_AMPLIFICATION_BREAK=slice-zero-despite-reuse  分片段已判 0 态不看那一行的现判与复用（改前的判法：全部复用的一趟也红）
#       第 43 格（「全部复用的一趟…判了红」）
#   CRASH_AMPLIFICATION_BREAK=domains-missing-accepted  登了段的行 registered_segment_domains= / registered_segment_progress= 缺了或写法认不得，当成没有登记段的域（不判红，ITEM 行写 -）
#       第 37 格（「登了段的行没报 unregistered_segments_beyond_expansion=，没照不穷举判红、出路不是重编引擎」）、
#       第 40 格（「登了段的行 registered_segment_domains= / registered_segment_progress= 缺了或写法认不得，没判红在那条流」）
#   CRASH_AMPLIFICATION_BREAK=domain-unlisted-segment-accepted  registered_segments= 列的段不在 registered_segment_domains= 里、分片段不在 registered_segment_progress= 里都不判红
#       第 41 格（「registered_segment_domains= 漏了登记的段、或分片段没在 registered_segment_progress= 里，没判红」）
#   CRASH_AMPLIFICATION_BREAK=slice-domains-unreported  ITEM 行的 registered_segment_domains= / registered_segment_progress= 写 -、成功句不报登记段的域那一段
#       第 32 格（「一行 exhaustive=false 只因登记的段…这一项却判了红」）、第 33 格（「一行 exhaustive=false registered_segments=- 没照旧判红在那一条流」）、
#       第 36 格（「登了段的流里另有展开上限盖不住又没登记的段，没判红在那条流与那几段」）、
#       第 37 格（「登了段的行没报 unregistered_segments_beyond_expansion=，没照不穷举判红、出路不是重编引擎」）、
#       第 38 格（「登记段分片枚举…判了红、或 ITEM 行没带…或成功句没报全展开与分片已判几态」）、
#       第 39 格（「分片段这一趟已判 0 态却没判红在那条流与那一段」：ITEM 行的进度没照实写）、
#       第 40 格（「登了段的行 registered_segment_domains= / registered_segment_progress= 缺了或写法认不得，没判红在那条流」）、
#       第 41 格（「registered_segment_domains= 漏了登记的段、或分片段没在 registered_segment_progress= 里，没判红」）
#   CRASH_AMPLIFICATION_BREAK=ignore-summed-per-row  按登记不判的 ignored_points= / states_skipped_by_ignore= 对每一行相加（同一条流叫停再续这类几行重复计），不按流分组只取一次、同组各行不同也不记 unknown
#       第 42 格（「按登记不判的点数与态数没按流分组、每组只取一次」）
#   CRASH_AMPLIFICATION_BREAK=rocksdb-entry-unchecked  清场与起分项之前不调 scripts/hosts/rocksdb-prebuilt.py entry（改前的样子）
#       第 48 格（「验证包依赖 rocksdb 而进程环境里没有预编库的取值，应当在清场与起任何一项之前退 78…」）
#   CRASH_AMPLIFICATION_BREAK=shard-phase-ignored  不认 TASK_SERVER_SHARD_PHASE 这几个变量，计划、一片、合并都照不分片跑（文件头「经 task-server 按项分片」）
#       第 55 格（「按项分片的计划那一步应当……」）、第 56 格（「按项分片的一片应当……」）、第 57 格（「按项分片的合并应当……」）、第 58 格（「带 --no-shard 应当计划写一片……」：起了驱动）
#   CRASH_AMPLIFICATION_BREAK=no-shard-ignored  不认 --no-shard，照按项分片
#       第 58 格（「带 --no-shard 应当计划写一片……」）
#   CRASH_AMPLIFICATION_BREAK=run-target-kept  跑完不删这一趟自己建的编译目录（改前的样子：分项在仓根照仓根的 target 编、跨趟留着）
#       第 49 格（「跑完运行目录里不剩它建的编译目录（正常退 0）」）、第 50 格（「跑完运行目录里不剩它建的编译目录（驱动退 1）」）、
#       第 51 格（「跑完运行目录里不剩它建的编译目录（被 TERM 停）」）
# 只供自证（自证起的子进程里才认；平常跑时有一个非空就在参数解析之前退 2 拒跑、拒绝句逐个点名，只有 --selftest 本身不拒，--list-items 与认不得的选项也先看这个）：
#   CRASH_AMPLIFICATION_BREAK=<项>（上面那张表）；CRASH_AMPLIFICATION_SELFTEST_RED_ITEM=1 每项的测试目标名后面加 -red（假驱动见了退 1）；
#   CRASH_AMPLIFICATION_SELFTEST_DROP_RESULT=<分项> 跑完那一项不打它的 CRASH_AMPLIFICATION_ITEM 行、不计进汇总（造一项收不回）；
#   CRASH_AMPLIFICATION_ITEM_DRIVER=<脚本> 换掉分项驱动（自证用假的）；
#   CRASH_AMPLIFICATION_SELFTEST_SEGMENTS_ROOT=<目录> 按段登记的被判仓根换成它（核与送的都是 <目录>/.claude/gate.d/crash-point-replay-segments.tsv，见「按段登记」）；
#     起 cargo 之前 rocksdb-prebuilt.py entry 判的树同样换成它（小仓里没有依赖 rocksdb 的清单，entry 报不判；第 48 格往里写一份依赖 rocksdb 的清单证红）。
#   「自证起的子进程」的标记：CRASH_AMPLIFICATION_SELFTEST_MARK=<文件>，那份文件在、内容是 SELFTEST_MARK_CONTENT 那一行；只有 run_selftest 设它。
#   这几个变量读完就从环境里清掉，不传给分项驱动。
#   CRASH_AMPLIFICATION_SWITCH_TABLE=<文件>：--selftest 末尾的开关轮从这份文件的文件头读开关表（不设读本脚本自己的），代码里认的开关照旧从本脚本取；
#     拿一份改坏了表的拷贝（删一格、多一格、格名写错）证开关轮会红。只在 --selftest 的开关轮里读：平常跑不读它，不在上面拒跑的五个之列。
# 自证末尾那一轮（开关轮）走 scripts/crash/crash-amplification-lib.sh 的 crash_amplification_switch_wheel（与双机驱动共用，判法与并行度见那份文件头「弄坏开关轮」）：
#   并行度 CRASH_AMPLIFICATION_SELFTEST_JOBS（正整数），没给取 SINGLEFS_THREAD_CAP，再没给取登记的重活默认并行度（.claude/heavy-thread-default）；一个开关一个进程，各写各的输出文件，逐个 wait 收。
#
# gate-similar: crash-amplification-item.sh 它跑一个分项的一趟（排派活计划、定在哪跑）；这里是分项登记表与全部项的循环，每项调它
# gate-similar: crash-amplification-two-hosts.sh 它是分片驱动，经 task-server 分片协议跑一条用例的计划、各片与合并；这里与分项驱动都不调它
# gate-similar: crash-point-replay-shard-run.sh 它跑崩溃点重放登记的崩溃枚举用例（CPU 枚举、crash-point-replay 全绿标记）；这里跑的是走判定库的流水线用例
# gate-similar: gpu-quiesce-run.sh 它只管清场、回读、跑（不复原），由 task-server 套在要卡的命令外面；这里不经它、不清场
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}

crash_amplification_script_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repository_root="$(cd "$crash_amplification_script_directory/../.." && pwd)"
# 共用库：这里只用它的弄坏开关轮（读表的 crash_amplification_read_switch_table 与逐个打开的 crash_amplification_switch_wheel，--selftest 末尾那一轮，与双机驱动共用）
source "$crash_amplification_script_directory/crash-amplification-lib.sh"

# 分项登记表：分项名 <TAB> 测试目标 <TAB> 用例函数（次序就是不带参数时的跑序：先短的流，长的流复用它们的前缀）
ITEMS=(
  $'fixed-script-through-rollback\tcrash_amplification_fixed_script_through_rollback\tthe_fixed_script_through_the_rollback_is_recorded_checked_in_parallel_and_reused'
  $'fixed-script-through-unmount\tcrash_amplification_fixed_script_through_unmount\tthe_whole_fixed_script_through_the_unmount_is_recorded_checked_in_parallel_and_reused'
  $'unlink-after-the-fixed-script\tcrash_enumeration_unlink_after_the_fixed_script_stream\tcrash_amplification_of_the_unlink_after_the_fixed_script_stream_is_clean'
  $'unlink-merging-inode-leaf-containers\tcrash_enumeration_unlink_merging_inode_leaf_containers_stream\tcrash_amplification_of_the_unlink_merging_inode_leaf_containers_stream_is_clean'
  $'multi-record-publish\tcrash_enumeration_multi_record_publish_stream\tcrash_amplification_of_the_multi_record_publish_stream_is_clean'
  $'unlink-after-the-multi-record-publish\tcrash_enumeration_unlink_after_the_multi_record_publish_stream\tcrash_amplification_of_the_unlink_after_the_multi_record_publish_stream_is_clean'
  $'tree-split-streams\tcrash_enumeration_tree_split_streams\tcrash_amplification_of_every_tree_split_stream_is_clean'
  $'position-addressed-trees\tcrash_enumeration_position_addressed_trees\tcrash_amplification_of_every_position_addressed_tree_stream_is_clean'
  $'floor-raise-pushed-by-the-session\tcrash_enumeration_floor_raise_pushed_by_admission\tcrash_amplification_of_the_floor_raise_pushed_by_the_session_is_clean'
  $'truncate-to-zero-on-a-nearly-full-pool\tcrash_enumeration_truncate_to_zero_on_a_nearly_full_pool_stream\tcrash_amplification_of_the_truncate_to_zero_on_a_nearly_full_pool_stream_is_clean'
  $'sigma-of-the-misaligned-reuse\trecord_checker_judges_absence_by_the_persisted_set\tcrash_amplification_of_sigma_on_the_misaligned_reuse_history_is_clean'
  $'compaction-relocation\tcrash_enumeration_compaction_relocation_stream\tcrash_amplification_of_the_compaction_relocation_stream_is_clean'
  $'compaction-relocating-a-block-shared-with-a-snapshot\tcrash_enumeration_compaction_relocating_a_block_shared_with_a_snapshot_stream\tcrash_amplification_of_the_compaction_relocating_a_block_shared_with_a_snapshot_stream_is_clean'
  $'lifecycle-closure\tcrash_enumeration_lifecycle_closure_stream\tcrash_amplification_of_the_lifecycle_closure_stream_is_clean'
  $'unlink-under-a-snapshot\tcrash_enumeration_unlink_under_a_snapshot_stream\tcrash_amplification_of_the_unlink_under_a_snapshot_stream_is_clean'
  $'truncate-to-zero\tcrash_enumeration_truncate_to_zero_stream\tcrash_amplification_of_the_truncate_to_zero_stream_is_clean'
  $'truncate-into-a-unit-then-extend\tcrash_enumeration_truncate_into_a_unit_then_extend_stream\tcrash_amplification_of_the_truncate_into_a_unit_then_extend_stream_is_clean'
  $'truncate-under-a-snapshot\tcrash_enumeration_truncate_under_a_snapshot_stream\tcrash_amplification_of_the_truncate_under_a_snapshot_stream_is_clean'
  $'snapshot-creation\tcrash_enumeration_snapshot_creation_stream\tcrash_amplification_of_the_snapshot_creation_stream_is_clean'
  $'overwrite-and-unlink-under-a-snapshot\tcrash_enumeration_overwrite_and_unlink_under_a_snapshot_stream\tcrash_amplification_of_the_overwrite_and_unlink_under_a_snapshot_stream_is_clean'
  $'snapshot-destruction-resumed-after-a-crash\tcrash_enumeration_snapshot_destruction_resumed_after_a_crash_stream\tcrash_amplification_of_the_snapshot_destruction_resumed_after_a_crash_stream_is_clean'
  $'one-file-overwritten-across-two-writable-remounts\tcrash_enumeration_one_file_overwritten_across_two_writable_remounts_stream\tcrash_amplification_of_one_file_overwritten_across_two_writable_remounts_is_clean'
  $'several-inodes-overwritten-in-turn\tcrash_enumeration_several_inodes_overwritten_in_turn_stream\tcrash_amplification_of_several_inodes_overwritten_in_turn_is_clean'
  $'large-file-overwritten-across-lower-leaves\tcrash_enumeration_large_file_overwritten_across_lower_leaves_stream\tcrash_amplification_of_a_large_file_overwritten_across_lower_leaves_is_clean'
  $'unlink-of-two-files-holding-data\tcrash_enumeration_unlink_of_two_files_holding_data_streams\tcrash_amplification_of_both_unlink_streams_is_clean'
  $'compaction-deadlist-reclamation\tcrash_enumeration_compaction_deadlist_reclamation_stream\tcrash_amplification_of_the_compaction_deadlist_reclamation_stream_is_clean'
  $'three-generation-mount-fork\tcrash_enumeration_three_generation_mount_fork_stream\tcrash_amplification_of_the_three_generation_mount_fork_stream_is_clean'
  $'tree-split-and-shrink\tcrash_enumeration_tree_split_and_shrink_stream\tcrash_amplification_of_the_tree_split_and_shrink_stream_is_clean'
  $'last-fitting-unit-then-refusal-and-unmount-on-a-nearly-full-pool\tcrash_enumeration_last_fitting_unit_then_refusal_and_unmount_on_a_nearly_full_pool_stream\tcrash_amplification_of_the_last_fitting_unit_then_refusal_and_unmount_on_a_nearly_full_pool_stream_is_clean'
)

# 只供自证的变量（文件头「只供自证」）：先存下来、判是不是自证起的子进程，标记读完就清；四个变量在平常跑的那一支拒或清
selftest_mark_file="${CRASH_AMPLIFICATION_SELFTEST_MARK:-}"
SELFTEST_MARK_CONTENT="crash-amplification.sh --selftest 起的子进程"
crash_amplification_break_switches="${CRASH_AMPLIFICATION_BREAK:-}"
selftest_red_item="${CRASH_AMPLIFICATION_SELFTEST_RED_ITEM:-}"
selftest_drop_result="${CRASH_AMPLIFICATION_SELFTEST_DROP_RESULT:-}"
test_item_driver="${CRASH_AMPLIFICATION_ITEM_DRIVER:-}"
selftest_segments_root="${CRASH_AMPLIFICATION_SELFTEST_SEGMENTS_ROOT:-}"

running_under_selftest=0
if [[ -n "${CRASH_AMPLIFICATION_SELFTEST_MARK:-}" && -f "${CRASH_AMPLIFICATION_SELFTEST_MARK:-}" ]] \
   && [[ "$(cat -- "$CRASH_AMPLIFICATION_SELFTEST_MARK")" == "$SELFTEST_MARK_CONTENT" ]]; then
  running_under_selftest=1
fi
unset CRASH_AMPLIFICATION_SELFTEST_MARK

crash_amplification_break_is() { [[ ",$crash_amplification_break_switches," == *",$1,"* ]]; }
# 关守卫项 honor-test-variables-outside-selftest 整值比：只在它是 CRASH_AMPLIFICATION_BREAK 唯一一项时认（与别的项同写，平常跑照拒；开关轮按「crash_amplification_break_is <开关>」数代码里认的开关，这一句要写出它的名字）
crash_amplification_guard_off_honored() { [[ "$crash_amplification_break_switches" == honor-test-variables-outside-selftest ]] && crash_amplification_break_is honor-test-variables-outside-selftest; }

# 一项的汇总（判法见文件头「一项怎么判」「复用数怎么取」「按登记整批不判」「按段登记」）：读这一项的输出文件，打一行十四个字段，空格隔开：
# 行数 现判 红 复用（或 unknown） exhaustive（true|false） 展开深度（或 unknown） 叫停的行数 没判的块（各行之和）
# red= 坏了的行数（缺 red= 或值不是非负整数） 按登记不判的点数（或 unknown） 按登记不判的态数（或 unknown）
# 不穷举又没按段登记的行数 那几行的流名（、隔开，没带 flow= 的写「没带flow=的行」；没有写 -） 按段登记的段（[<流>@]<段号>:<写数>，逗号隔开；没有写 -）
# 登了段而另有展开上限盖不住又没登记的段的行数 那几段（[<流>@]<段号>:<写数>，逗号隔开；没有写 -）
# 登了段、exhaustive=false 而没报 unregistered_segments_beyond_expansion=（或写得认不得）的行数 那几行的流名（写法同上；没有写 -）
# ITEM 行的 unregistered_segments_beyond_expansion=（各行报的段，[<流>@]<段号>:<写数> 逗号隔开；没有写 -，取不出写 unknown）
# ITEM 行的 registered_segment_domains=（[<流>@]<段号>:<full|slice> 逗号隔开）与 registered_segment_progress=（[<流>@]<段号>:<已判>/<总>）：没有写 -，取不出写 unknown
# 登了段而 registered_segment_domains= / registered_segment_progress= 缺了、没列全登记的段或写法认不得的行数 那几行的流名（写法同上；没有写 -）
# 分片段已判 0 态的行数 那几段（[<流>@]<段号>:<已判>/<总>，逗号隔开；没有写 -）
# 按登记不判的两个数同一组（按流分，没带 flow= 的归一组）各行报的数不同的流名（、隔开，没带 flow= 的写「没带flow=的行」；没有写 -）
summarize_the_item() { # <这一项的输出文件>
  local stopped_reuse=0 as_zero=0 ignore_as_zero=0 output_file="$1" ignore_per_row=0
  crash_amplification_break_is ignore-summed-per-row && ignore_per_row=1
  local registered_ignored=0 registered_whole_item=0 dash_registered=0 beyond_ignored=0 beyond_missing_as_dash=0
  local slice_zero_accepted=0 slice_zero_despite_reuse=0 domains_missing_accepted=0 domain_unlisted_accepted=0
  crash_amplification_break_is reuse-from-stopped-line && stopped_reuse=1
  crash_amplification_break_is reuse-as-zero && as_zero=1
  crash_amplification_break_is ignore-unknown-as-zero && ignore_as_zero=1
  crash_amplification_break_is registered-segments-ignored && registered_ignored=1
  crash_amplification_break_is registered-excuses-the-whole-item && registered_whole_item=1
  crash_amplification_break_is dash-counts-as-registered && dash_registered=1
  crash_amplification_break_is beyond-expansion-ignored && beyond_ignored=1
  crash_amplification_break_is beyond-expansion-missing-as-dash && beyond_missing_as_dash=1
  crash_amplification_break_is slice-progress-zero-accepted && slice_zero_accepted=1
  crash_amplification_break_is slice-zero-despite-reuse && slice_zero_despite_reuse=1
  crash_amplification_break_is domains-missing-accepted && domains_missing_accepted=1
  crash_amplification_break_is domain-unlisted-segment-accepted && domain_unlisted_accepted=1
  [[ -f "$output_file" ]] || output_file=/dev/null
  # 复用数与没判的块都是各行之和（文件头「复用数怎么取」「一项怎么判」not-run）：入口收到的行都是本机整份跑的，各行判的块不相交。
  # 按段登记（文件头「按段登记」）：exhaustive=false 而 registered_segments= 是 <段号>:<写数>[,…] 的行，unregistered_segments_beyond_expansion= 是 - 才不算不穷举；
  #   不是 - 的那几段、没报这个字段的那几行另记；那几段与各行报的这个字段都按流去重记下。
  #   登了段的行还要报 registered_segment_domains= / registered_segment_progress=（文件头「按段登记」的分片枚举）：缺、漏段、认不得的那几行另记，分片段已判 0 态的那几段另记
  awk -v stopped_reuse="$stopped_reuse" -v as_zero="$as_zero" -v ignore_as_zero="$ignore_as_zero" \
      -v registered_ignored="$registered_ignored" -v registered_whole_item="$registered_whole_item" -v dash_registered="$dash_registered" \
      -v beyond_ignored="$beyond_ignored" -v beyond_missing_as_dash="$beyond_missing_as_dash" \
      -v slice_zero_accepted="$slice_zero_accepted" -v slice_zero_despite_reuse="$slice_zero_despite_reuse" -v domains_missing_accepted="$domains_missing_accepted" -v domain_unlisted_accepted="$domain_unlisted_accepted" \
      -v ignore_per_row="$ignore_per_row" '
    BEGIN {
      # 分片段的进度：<段号>:<已判>/<总>，两个数都是十进制数字串（可能到 140 位）：只按写法认，不转成数、不做数值比较
      progress_entry = "[0-9]+:[0-9]+/[0-9]+"
      progress_list = "^" progress_entry "(," progress_entry ")*$"
    }
    function remember_one(kind, entry) {
      # 按 kind（domain、progress、zero）去重追加进 listed[kind]
      if ((kind, entry) in entry_seen) return
      entry_seen[kind, entry] = 1
      listed[kind] = listed[kind] (listed[kind] == "" ? "" : ",") entry
    }
    function domains_problem_of_the_line(    count, pieces, piece_index, at, number) {
      # 这一行的 registered_segment_domains= / registered_segment_progress=：0 对；1 缺了一个、写法认不得、或进度里有不是分片的段；2 登记的段没在域里、或分片段没在进度里。
      #   对时段号的域记进 line_kind、分片段的「已判/总」记进 line_progress（弄坏开关 domain-unlisted-segment-accepted 下不查 2 那两样）
      split("", line_kind); split("", line_progress)
      if (!domains_seen || !progress_seen) return 1
      if (domains != "-" && domains !~ /^[0-9]+:(full|slice)(,[0-9]+:(full|slice))*$/) return 1
      if (progress != "-" && progress !~ progress_list) return 1
      if (domains != "-") {
        count = split(domains, pieces, ",")
        for (piece_index = 1; piece_index <= count; piece_index++) {
          at = index(pieces[piece_index], ":"); line_kind[substr(pieces[piece_index], 1, at - 1)] = substr(pieces[piece_index], at + 1)
        }
      }
      if (progress != "-") {
        count = split(progress, pieces, ",")
        for (piece_index = 1; piece_index <= count; piece_index++) {
          at = index(pieces[piece_index], ":"); number = substr(pieces[piece_index], 1, at - 1)
          if (!(number in line_kind) || line_kind[number] != "slice") return 1
          line_progress[number] = substr(pieces[piece_index], at + 1)
        }
      }
      if (domain_unlisted_accepted) return 0
      count = split(registered, pieces, ",")
      for (piece_index = 1; piece_index <= count; piece_index++) {
        number = pieces[piece_index]; sub(/:.*/, "", number)
        if (number != "-" && !(number in line_kind)) return 2
      }
      for (number in line_kind) if (line_kind[number] == "slice" && !(number in line_progress)) return 2
      return 0
    }
    function segment_entries_of(list, flow_name,    count, pieces, piece_index, text) {
      # <段号>:<写数>[,…] → [<流>@]<段号>:<写数>[,…]
      count = split(list, pieces, ","); text = ""
      for (piece_index = 1; piece_index <= count; piece_index++) text = text (text == "" ? "" : ",") (flow_name == "" ? "" : flow_name "@") pieces[piece_index]
      return text
    }
    function remember_entries(entries, kind,    count, pieces, piece_index) {
      # 按 kind 去重追加：kind 是 "red" 记进 beyond_red_text，"report" 记进 beyond_report_text
      count = split(entries, pieces, ",")
      for (piece_index = 1; piece_index <= count; piece_index++) {
        if ((kind, pieces[piece_index]) in entry_seen) continue
        entry_seen[kind, pieces[piece_index]] = 1
        if (kind == "red") beyond_red_text = beyond_red_text (beyond_red_text == "" ? "" : ",") pieces[piece_index]
        else beyond_report_text = beyond_report_text (beyond_report_text == "" ? "" : ",") pieces[piece_index]
      }
    }
    /^CRASH_AMPLIFICATION / {
      lines++
      explicit = ""; states = ""; checked = ""; stopped = 0; unjudged = 0; exhaustive = ""; expand = ""
      red_seen = 0; ignored_here = ""; skipped_here = ""; flow = ""; registered = ""; beyond = ""; beyond_seen = 0
      domains = ""; domains_seen = 0; progress = ""; progress_seen = 0
      for (field = 2; field <= NF; field++) {
        key = $field; sub(/=.*/, "", key); value = substr($field, length(key) + 2)
        if (key == "states_reused") explicit = value
        else if (key == "flow") flow = value
        else if (key == "states") states = value
        else if (key == "states_checked_here") checked = value
        else if (key == "stopped_early" && value == "true") stopped = 1
        else if (key == "unjudged_blocks") unjudged = value + 0
        else if (key == "red") { red_seen = 1; if (value ~ /^[0-9]+$/) red += value; else bad_red++ }
        else if (key == "ignored_points") ignored_here = value
        else if (key == "states_skipped_by_ignore") skipped_here = value
        else if (key == "exhaustive") exhaustive = value
        else if (key == "expand_up_to") expand = value
        else if (key == "registered_segments") registered = value
        else if (key == "unregistered_segments_beyond_expansion") { beyond = value; beyond_seen = 1 }
        else if (key == "registered_segment_domains") { domains = value; domains_seen = 1 }
        else if (key == "registered_segment_progress") { progress = value; progress_seen = 1 }
      }
      # unregistered_segments_beyond_expansion=：- 或 <段号>:<写数>[,…] 才认得；没报或写得认不得算取不出（弄坏开关 beyond-expansion-missing-as-dash 下当成 -）
      beyond_known = (beyond_seen && (beyond == "-" || beyond ~ /^[0-9]+:[0-9]+(,[0-9]+:[0-9]+)*$/))
      if (!beyond_known && beyond_missing_as_dash) { beyond = "-"; beyond_known = 1 }
      if (!beyond_known) beyond_unknown = 1
      else if (beyond != "-") remember_entries(segment_entries_of(beyond, flow), "report")
      if (!red_seen) bad_red++
      # 按登记不判的两个数（文件头「按登记整批不判」）：按流分组、每组只取一次；同一组各行报的不同记进 ignore_conflict（弄坏开关 ignore-summed-per-row 下逐行相加）
      if (ignored_here ~ /^[0-9]+$/ && skipped_here ~ /^[0-9]+$/) {
        if (ignore_per_row) { ignored += ignored_here; skipped += skipped_here }
        else {
          ignore_points_text = ignored_here; sub(/^0+/, "", ignore_points_text); if (ignore_points_text == "") ignore_points_text = "0"
          ignore_states_text = skipped_here; sub(/^0+/, "", ignore_states_text); if (ignore_states_text == "") ignore_states_text = "0"
          if (!(flow in ignore_seen)) {
            ignore_seen[flow] = 1; ignore_flow_order[++ignore_flow_count] = flow
            ignore_points_of[flow] = ignore_points_text; ignore_states_of[flow] = ignore_states_text
          } else if (ignore_points_of[flow] != ignore_points_text || ignore_states_of[flow] != ignore_states_text) ignore_conflict[flow] = 1
        }
      }
      else if (!ignore_as_zero) ignore_unknown = 1
      if (checked ~ /^[0-9]+$/) states_checked += checked
      if (exhaustive != "true") not_exhaustive = 1
      # 按段登记：这一行登了段（registered_segments= 不是 - 而写得认得）就把段按流记下；exhaustive=false 只因登记段的行不算不穷举
      line_registered = (!registered_ignored && (registered ~ /^[0-9]+:[0-9]+(,[0-9]+:[0-9]+)*$/ || (dash_registered && registered == "-")))
      flow_label = (flow == "" ? "没带flow=的行" : flow)
      if (line_registered) {
        any_registered = 1
        segment_count = split(registered, segment_list, ",")
        for (segment_index = 1; segment_index <= segment_count; segment_index++) {
          if (segment_list[segment_index] == "-") continue
          entry = (flow == "" ? "" : flow "@") segment_list[segment_index]
          if (!(entry in registered_seen)) { registered_seen[entry] = 1; registered_text = registered_text (registered_text == "" ? "" : ",") entry }
        }
        # 登记段的域与分片进度（文件头「按段登记」）：缺、漏段、认不得记下流名（弄坏开关 domains-missing-accepted 下缺与认不得当成没有登记段的域）；
        #   对的照报的次序按流去重记下，分片段已判数字面全是 0 的另记那几段（弄坏开关 slice-progress-zero-accepted 下不记）
        domains_problem = domains_problem_of_the_line()
        if (domains_problem == 1 && domains_missing_accepted) { domains_problem = 0; domains = "-"; progress = "-"; split("", line_kind); split("", line_progress) }
        if (domains_problem) {
          domain_bad_lines++; domain_unknown = 1
          if (!(flow_label in domain_bad_seen)) { domain_bad_seen[flow_label] = 1; domain_bad_flows = domain_bad_flows (domain_bad_flows == "" ? "" : "、") flow_label }
        } else {
          if (domains != "-") {
            segment_count = split(domains, segment_list, ",")
            for (segment_index = 1; segment_index <= segment_count; segment_index++) remember_one("domain", (flow == "" ? "" : flow "@") segment_list[segment_index])
          }
          # 分片段已判 0 态只在这一行现判与复用都是 0 时记（文件头「按段登记」）：states_checked_here= 与 states_reused= 有一个是正整数就不记（全复用的一趟不红）；
          #   没报 states_reused= 的照 0 算。弄坏开关 slice-zero-despite-reuse 下不看这两个数
          line_idle = ((checked !~ /^0*[1-9][0-9]*$/ && explicit !~ /^0*[1-9][0-9]*$/) || slice_zero_despite_reuse)
          line_zero = 0
          if (progress != "-") {
            segment_count = split(progress, segment_list, ",")
            for (segment_index = 1; segment_index <= segment_count; segment_index++) {
              entry = (flow == "" ? "" : flow "@") segment_list[segment_index]
              remember_one("progress", entry)
              judged_here = segment_list[segment_index]; sub(/^[0-9]+:/, "", judged_here); sub(/\/.*/, "", judged_here)
              if (judged_here ~ /^0+$/ && !slice_zero_accepted && line_idle) { line_zero = 1; remember_one("zero", entry) }
            }
          }
          if (line_zero) zero_lines++
        }
      }
      if (exhaustive != "true" && !(exhaustive == "false" && line_registered)) {
        not_covered++; not_covered_label[not_covered] = flow_label; not_covered_false[not_covered] = (exhaustive == "false")
      } else if (exhaustive == "false" && line_registered && !beyond_ignored) {
        # 登了段的行：另有展开上限盖不住又没登记的段判红（记下那几段），没报这个字段照不穷举判红（记下流名）；是 - 的不算不穷举
        if (!beyond_known) {
          missing_lines++
          if (!(flow_label in missing_seen)) { missing_seen[flow_label] = 1; missing_flows = missing_flows (missing_flows == "" ? "" : "、") flow_label }
        } else if (beyond != "-") {
          beyond_red_lines++; remember_entries(segment_entries_of(beyond, flow), "red")
        }
      }
      if (expand == "") expand = "?"
      if (!(expand in expands)) { expands[expand] = 1; distinct_expands++; only_expand = expand }
      if (stopped) stopped_lines++
      if (unjudged > 0) unjudged_blocks += unjudged
      # 这一行的复用数（文件头「复用数怎么取」①②③）：取不出时这一项记 unknown（弄坏开关 reuse-from-stopped-line 下叫停的行也拿差当复用数）
      if (explicit ~ /^[0-9]+$/) reused += explicit
      else if (states ~ /^[0-9]+$/ && checked ~ /^[0-9]+$/ && states + 0 >= checked + 0 && (!stopped || stopped_reuse)) reused += states - checked
      else reuse_unknown = 1
    }
    END {
      if (lines == 0 || reuse_unknown) reused_text = "unknown"; else if (as_zero) reused_text = "0"; else reused_text = sprintf("%.0f", reused)
      exhaustive_text = (lines > 0 && !not_exhaustive) ? "true" : "false"
      expand_text = (distinct_expands == 1 && only_expand != "?") ? only_expand : "unknown"
      # 按登记不判的两个数：各组取到的数相加；有组各行报的不同，这一项两个数写 unknown 并点名那条流
      ignore_conflict_flows = ""
      for (ignore_index = 1; ignore_index <= ignore_flow_count; ignore_index++) {
        ignore_flow = ignore_flow_order[ignore_index]
        if (ignore_flow in ignore_conflict) {
          ignore_unknown = 1
          ignore_conflict_flows = ignore_conflict_flows (ignore_conflict_flows == "" ? "" : "、") (ignore_flow == "" ? "没带flow=的行" : ignore_flow)
        } else { ignored += ignore_points_of[ignore_flow]; skipped += ignore_states_of[ignore_flow] }
      }
      if (lines == 0 || ignore_unknown) { ignored_text = "unknown"; skipped_text = "unknown" }
      else { ignored_text = sprintf("%.0f", ignored); skipped_text = sprintf("%.0f", skipped) }
      # 不穷举又没按段登记的行（弄坏开关 registered-excuses-the-whole-item 下，这一项有一行登了段就把 exhaustive=false 的行都算登记了）
      uncovered_lines = 0; uncovered_flows = ""
      for (line_index = 1; line_index <= not_covered; line_index++) {
        if (registered_whole_item && any_registered && not_covered_false[line_index]) continue
        uncovered_lines++; flow_label = not_covered_label[line_index]
        if (!(flow_label in uncovered_seen)) { uncovered_seen[flow_label] = 1; uncovered_flows = uncovered_flows (uncovered_flows == "" ? "" : "、") flow_label }
      }
      report_text = (beyond_report_text != "" ? beyond_report_text : ((lines == 0 || beyond_unknown) ? "unknown" : "-"))
      domain_text = (listed["domain"] != "" ? listed["domain"] : (domain_unknown ? "unknown" : "-"))
      progress_text = (listed["progress"] != "" ? listed["progress"] : (domain_unknown ? "unknown" : "-"))
      printf "%d %.0f %.0f %s %s %s %d %.0f %d %s %s %d %s %s %d %s %d %s %s %s %s %d %s %d %s %s\n", lines, states_checked, red, reused_text, exhaustive_text, expand_text, stopped_lines, unjudged_blocks, bad_red, ignored_text, skipped_text,
        uncovered_lines, (uncovered_flows == "" ? "-" : uncovered_flows), (registered_text == "" ? "-" : registered_text),
        beyond_red_lines, (beyond_red_text == "" ? "-" : beyond_red_text), missing_lines, (missing_flows == "" ? "-" : missing_flows), report_text,
        domain_text, progress_text, domain_bad_lines, (domain_bad_flows == "" ? "-" : domain_bad_flows), zero_lines, (listed["zero"] == "" ? "-" : listed["zero"]),
        (ignore_conflict_flows == "" ? "-" : ignore_conflict_flows)
    }' "$output_file"
}

item_fields() { # <登记行> <字段号 1..4>
  printf '%s' "$1" | cut -f"$2"
}

segments_of_the_item_for_the_success_line() { # <分项> <[<流>@]<段号>:<写数>[,…]>：打「段数<TAB><分项> 流 <流> 段 <段号>（<写数> 写）、…，…」（成功句用）
  awk -v item="$1" -v list="$2" 'BEGIN {
    count = split(list, entries, ",")
    for (entry_index = 1; entry_index <= count; entry_index++) {
      entry = entries[entry_index]; flow = ""
      at = index(entry, "@"); if (at) { flow = substr(entry, 1, at - 1); entry = substr(entry, at + 1) }
      split(entry, numbers, ":")
      if (!(flow in flow_seen)) { flow_seen[flow] = 1; flow_order[++flows] = flow }
      segments_of[flow] = segments_of[flow] (segments_of[flow] == "" ? "" : "、") "段 " numbers[1] "（" numbers[2] " 写）"
      total++
    }
    text = ""
    for (flow_index = 1; flow_index <= flows; flow_index++) {
      flow = flow_order[flow_index]
      text = text (text == "" ? "" : "，") item (flow == "" ? "" : " 流 " flow) " " segments_of[flow]
    }
    printf "%d\t%s\n", total, text
  }'
}

domains_of_the_item_for_the_success_line() { # <分项> <[<流>@]<段号>:<full|slice>[,…]> <[<流>@]<段号>:<已判>/<总>[,…] 或 ->：
  #   打「段数<TAB>全展开段数<TAB>分片段数<TAB><分项> 流 <流> 段 <段号>：全展开、<分项> 流 <流> 段 <段号>：分片，已判 <已判>/<总>、…」（成功句用；同一段几行报的进度用「与」连）
  awk -v item="$1" -v list="$2" -v progress="$3" 'BEGIN {
    count = split(progress, pieces, ",")
    for (piece_index = 1; piece_index <= count; piece_index++) {
      if (pieces[piece_index] == "-") continue
      at = index(pieces[piece_index], "/"); head = substr(pieces[piece_index], 1, at - 1); colon = 0
      for (position = length(head); position > 0; position--) if (substr(head, position, 1) == ":") { colon = position; break }
      key = substr(head, 1, colon - 1); fraction = substr(pieces[piece_index], colon + 1)
      progress_of[key] = progress_of[key] (progress_of[key] == "" ? "" : " 与 ") fraction
    }
    count = split(list, entries, ",")
    for (entry_index = 1; entry_index <= count; entry_index++) {
      entry = entries[entry_index]; flow = ""; key_part = entry; sub(/:[a-z]+$/, "", key_part); kind = substr(entry, length(key_part) + 2)
      segment = key_part; at = index(segment, "@"); if (at) { flow = substr(segment, 1, at - 1); segment = substr(segment, at + 1) }
      label = item (flow == "" ? "" : " 流 " flow) " 段 " segment
      if (kind == "full") { label = label "：全展开"; full++ }
      else { label = label "：分片，已判 " (key_part in progress_of ? progress_of[key_part] : "取不出"); sliced++ }
      text = text (text == "" ? "" : "、") label
      total++
    }
    printf "%d\t%d\t%d\t%s\n", total, full, sliced, text
  }'
}

segments_by_flow() { # <[<流>@]<段号>:<写数>[,…]>：打「流 <流> 段 <段号>:<写数>、…；流 …」（红因用；没带 flow= 的行只写「段 …」）
  awk -v list="$1" 'BEGIN {
    count = split(list, entries, ",")
    for (entry_index = 1; entry_index <= count; entry_index++) {
      entry = entries[entry_index]; flow = ""
      at = index(entry, "@"); if (at) { flow = substr(entry, 1, at - 1); entry = substr(entry, at + 1) }
      if (!(flow in flow_seen)) { flow_seen[flow] = 1; flow_order[++flows] = flow }
      segments_of[flow] = segments_of[flow] (segments_of[flow] == "" ? "" : "、") entry
    }
    text = ""
    for (flow_index = 1; flow_index <= flows; flow_index++) {
      flow = flow_order[flow_index]
      text = text (text == "" ? "" : "；") (flow == "" ? "" : "流 " flow " ") "段 " segments_of[flow]
    }
    printf "%s", text
  }'
}

list_items() { # 分项名、测试目标、用例函数
  local row
  for row in "${ITEMS[@]}"; do
    printf '%s\t%s\t%s\n' "$(item_fields "$row" 1)" "$(item_fields "$row" 2)" "$(item_fields "$row" 3)"
  done
}

rocksdb_entry_checked=0   # 指定范围的崩溃域上限起读库程序之前已判过 RocksDB 预编库的取值，开跑前那一判就不再判
# 指定范围的崩溃域上限借判法 lib_crash_point_replay_items.py 的 --defect 开关，每个弄坏开关一个值，没开打空
scoped_domain_defect() {
  if crash_amplification_break_is scoped-domain-reusable-counted; then printf '%s' count-reusable
  elif crash_amplification_break_is scoped-domain-unknown-passed; then printf '%s' unknown-as-zero
  elif crash_amplification_break_is scoped-domain-suggestion-dropped; then printf '%s' suggestion-dropped
  elif crash_amplification_break_is scoped-domain-shrunk-records-used; then printf '%s' shrunk-records-used
  fi
}

# 指定范围的崩溃域上限：<ITEMS 的行…> 是点名的分项。估法、上限 200000 态与退出码 79 都在 lib_crash_point_replay_items.py 的 scoped-domain guard（崩溃点重放入口共用）；
#   放行交回 0，79 超上限或估不出（说明已打在标准错误），78 建不了这一趟的编译目录。读库程序要编，编译目录归这一趟：调用方没设 CARGO_TARGET_DIR 时在运行目录下建一个，
#   判完就删，在子 shell 里做（不把 CARGO_TARGET_DIR 带给后面分项驱动自己的编译目录）。弄坏开关 scoped-domain-unbounded 下不判；
#   scoped-domain-narrowed-refused 下缩了域也照没缩判（缩域值不交给判法）
scoped_domain_guard() {
  local row home configuration defect gpu=0 flow_arguments=() narrowed_arguments=() defect_arguments=() environment_arguments=()
  crash_amplification_break_is scoped-domain-unbounded && return 0
  # 读库程序要编（验证包依赖 rocksdb）：起 cargo 之前先判这台 RocksDB 预编库的取值，判红退 78；判过了，后面开跑前那一判（同一句）不再判一遍。弄坏开关 rocksdb-entry-unchecked 下不判
  if ! crash_amplification_break_is rocksdb-entry-unchecked; then
    if ! python3 "$crash_amplification_script_directory/../hosts/rocksdb-prebuilt.py" entry --root "$segments_root" >&2; then
      echo "  ✗ 指定范围的崩溃域上限要起读库程序，起 cargo 之前 RocksDB 预编库的取值没判过（上面 rocksdb-prebuilt.py entry 的红句），一项都没跑（退 78）" >&2
      echo "  → 怎么办：照上面那几行的出路修（开新会话，或命令前 eval \"\$(python3 scripts/hosts/rocksdb-prebuilt.py env)\"），再跑同一条命令" >&2
      return 78
    fi
    rocksdb_entry_checked=1
  fi
  home="$(crash_amplification_home_directory)" || return 2
  configuration="$(crash_amplification_multi_host_configuration_file "$segments_root")"
  [[ "$(crash_amplification_configuration_value "$configuration" ENABLE_GPU)" == 1 ]] && gpu=1
  [[ -f "$configuration" ]] && environment_arguments+=("SINGLEFS_MULTI_HOST_CONFIG=$(cd "$(dirname "$configuration")" && pwd)/$(basename "$configuration")")
  for row in "$@"; do flow_arguments+=(--flow "$(item_fields "$row" 1):$(item_fields "$row" 2)"); done
  if [[ -n "$crash_expand_up_to" ]] && ! crash_amplification_break_is scoped-domain-narrowed-refused; then narrowed_arguments=(--expand-up-to "$crash_expand_up_to"); fi
  defect="$(scoped_domain_defect)"
  if [[ -n "$defect" ]]; then defect_arguments=(--defect "$defect"); fi
  (
    source "$crash_amplification_script_directory/../hosts/run-build-directory.sh"
    guard_directory="$home/runs/scoped-domain-$(date -u +%Y%m%dT%H%M%SZ)-$$"
    run_build_directory_begin "$guard_directory" || exit 78
    env "BINDGEN_EXTRA_CLANG_ARGS=$CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS" ${environment_arguments[@]+"${environment_arguments[@]}"} \
      python3 "$crash_amplification_script_directory/lib_crash_point_replay_items.py" scoped-domain guard --root "$segments_root" --runs "$home/runs" --gpu "$gpu" \
      ${narrowed_arguments[@]+"${narrowed_arguments[@]}"} ${defect_arguments[@]+"${defect_arguments[@]}"} "${flow_arguments[@]}"
    guard_exit=$?
    if ! run_build_directory_end; then (( guard_exit != 0 )) || guard_exit=1; fi
    rmdir -- "$guard_directory" 2>/dev/null
    exit "$guard_exit"
  )
}

refuse_listing_the_items() { # <哪里不对>：打原因、出路与可点名的项，退 2
  echo "  ✗ $1" >&2
  echo "  → 怎么办：分项名照下面列的写（--item <分项>，给几次取并集）；不带 --item 就是全部项" >&2
  list_items | sed 's/^/    /' >&2
  exit 2
}

usage() { # <哪里不对>
  echo "  ✗ 用法不对：$1。用法：$0 [--no-shard] [--item <分项>]… [--crash-expand-up-to <n>] [--list-items] [--selftest] [--label <标签>]" >&2
  echo "  → 怎么办：不带参数就是剪枝跑全部项；分项名照 --list-items 列的写；经服务交的默认按项分片，--no-shard 才不分；--crash-expand-up-to 后面写非负整数（缩域，只往小缩）" >&2
  exit 2
}

# 平常跑：只供自证的变量有一个非空就拒，排在一切参数解析之前（认不得的选项、--list-items 也先看这个）；自证起的子进程与命令行第一个参数就是 --selftest 的除外（文件头「只供自证」）。
# 关守卫项 honor-test-variables-outside-selftest 整值比：只在它是 CRASH_AMPLIFICATION_BREAK 唯一一项时认；与别的项同写，平常跑照拒并点名 CRASH_AMPLIFICATION_BREAK。
if (( ! running_under_selftest )) && [[ "${1:-}" != --selftest ]] && ! crash_amplification_guard_off_honored; then
  test_only_given=()
  [[ -n "$crash_amplification_break_switches" ]] && test_only_given+=(CRASH_AMPLIFICATION_BREAK)
  [[ -n "$selftest_red_item" ]] && test_only_given+=(CRASH_AMPLIFICATION_SELFTEST_RED_ITEM)
  [[ -n "$selftest_drop_result" ]] && test_only_given+=(CRASH_AMPLIFICATION_SELFTEST_DROP_RESULT)
  [[ -n "$test_item_driver" ]] && test_only_given+=(CRASH_AMPLIFICATION_ITEM_DRIVER)
  [[ -n "$selftest_segments_root" ]] && test_only_given+=(CRASH_AMPLIFICATION_SELFTEST_SEGMENTS_ROOT)

  if (( ${#test_only_given[@]} > 0 )); then
    echo "  ✗ 平常跑时带了只给自证用的变量：${test_only_given[*]}；这几个变量只在自证里认，拒跑，一项都没跑" >&2
    echo "  → 怎么办：unset ${test_only_given[*]} 再跑；要证红就跑 $0 --selftest（它起的子进程才认这几个变量）" >&2
    exit 2
  fi
fi

selected=(); driver_arguments=(); list_only=0; selftest=0; no_shard=0; crash_expand_up_to_argument=""; crash_expand_up_to_given=0
while (($# > 0)); do
  case "$1" in
    --item)
      if (( $# < 2 )) || [[ -z "$2" || "$2" == -* ]]; then refuse_listing_the_items "--item 后面缺项名"; fi
      selected+=("$2"); shift 2 ;;
    --list-items) list_only=1; shift ;;
    --no-shard) no_shard=1; shift ;;
    --selftest) selftest=1; shift ;;
    --label)
      if (( $# < 2 )) || [[ -z "$2" || "$2" == -* ]]; then usage "$1 要一个值"; fi
      driver_arguments+=("$1" "$2"); shift 2 ;;
    --crash-expand-up-to)
      # 缩域（文件头「展开深度」）：值在循环之后统一核
      if (( $# < 2 )); then usage "$1 后面要跟一个非负整数（展开深度上限，只往小缩）"; fi
      crash_expand_up_to_argument="$2"; crash_expand_up_to_given=1; shift 2 ;;
    *)
      # 弄坏开关 unknown-option-accepted：认不出的选项也照收、当没给（不拒）
      if crash_amplification_break_is unknown-option-accepted; then shift; continue; fi
      usage "认不得的参数「$1」" ;;
  esac
done

# 缩域（文件头「展开深度」）：参数给了取参数，没给取环境里已设的变量；值要是非负整数（弄坏开关 expand-up-to-unchecked：不核），写坏退 2、一项都不跑。
#   自证（--selftest）不看这个参数与调用方环境里的变量
crash_expand_up_to=""; crash_expand_up_to_origin=""
if (( ! selftest )); then
  if (( crash_expand_up_to_given )); then
    crash_expand_up_to="$crash_expand_up_to_argument"; crash_expand_up_to_origin="--crash-expand-up-to"
  elif [[ -n "${SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO+x}" ]]; then
    crash_expand_up_to="$SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO"; crash_expand_up_to_origin="环境变量 SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO"; crash_expand_up_to_given=1
  fi
  if (( crash_expand_up_to_given )) && [[ ! "$crash_expand_up_to" =~ ^[0-9]+$ ]] && ! crash_amplification_break_is expand-up-to-unchecked; then
    usage "$crash_expand_up_to_origin 的值「$crash_expand_up_to」不是非负整数（展开深度上限，只往小缩），一项都没跑"
  fi
  # 缩过域：把变量交给每一项的分项驱动（引擎取它与自己的常量里小的那个，只往小缩）；弄坏开关 expand-up-to-not-passed：不交（环境里已设的也清掉）
  if (( crash_expand_up_to_given )); then
    if crash_amplification_break_is expand-up-to-not-passed; then unset SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO
    else export SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO="$crash_expand_up_to"; fi
  fi
fi

if (( list_only )); then list_items; exit 0; fi

# 登记表自检：三段都非空、分项名不重
declare -A seen=()
for row in "${ITEMS[@]}"; do
  name="$(item_fields "$row" 1)"; target="$(item_fields "$row" 2)"; function_name="$(item_fields "$row" 3)"
  if [[ -z "$name" || -z "$target" || -z "$function_name" ]]; then echo "  ✗ 登记表有一行不全：$row" >&2; echo "  → 怎么办：三段都填上（分项名、测试目标、用例函数）" >&2; exit 2; fi
  if [[ -n "${seen[$name]:-}" ]]; then echo "  ✗ 分项名重了：$name" >&2; echo "  → 怎么办：分项名一个只登一次" >&2; exit 2; fi
  seen["$name"]=1
done

run_selftest() {
  local work stub log nested="$running_under_selftest" service_problem
  # 只在临时 prune-server 上跑（文件头「用法」--selftest），不是就退 2、一格都不跑
  service_problem="$(crash_amplification_selftest_service_problem)"
  if [[ -n "$service_problem" ]]; then
    echo "  ✗ crash-amplification.sh 自证没跑：$service_problem" >&2
    echo "     → 怎么办：照 scripts/prune-server/prune_server_client.py 文件头「命令行」--with-temporary-service 那一条，在临时服务底下起这一份自证（带 --server-binary 就不编）" >&2
    exit 2
  fi
  work="$(mktemp -d "${TMPDIR:-/tmp}/crash-amplification-selftest.XXXXXX")"
  trap 'rm -rf -- "${work:?}"' EXIT
  # 自证起的子进程的标记：只有这里设（文件头「只供自证」）
  printf '%s\n' "$SELFTEST_MARK_CONTENT" > "$work/mark"
  export CRASH_AMPLIFICATION_SELFTEST_MARK="$work/mark"
  # 子进程的运行条件、每项输出都落在这里：多机配置指到不在的文件（两个开关都关），本次未跑不写进门禁的汇总
  export SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host.env" SINGLEFS_CRASH_AMPLIFICATION_HOME="$work/home" SINGLEFS_LAST_RUNS_DIRECTORY="$work/last-runs"
  unset CRASH_AMPLIFICATION_SELFTEST_RED_ITEM CRASH_AMPLIFICATION_SELFTEST_DROP_RESULT CRASH_AMPLIFICATION_ITEM_SWITCHES CRASH_AMPLIFICATION_ITEM_CARDS GATE_NOT_RUN_FILE SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO STUB_EXPAND_LOG
  unset SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY CRASH_AMPLIFICATION_SELFTEST_SEGMENTS_ROOT
  unset TASK_SERVER_SHARD_PHASE TASK_SERVER_SHARD TASK_SERVER_SHARD_PLAN_DIRECTORY TASK_SERVER_SHARD_OUTPUT_DIRECTORY TASK_SERVER_SHARD_OUTPUTS TASK_SERVER_SHARD_HOST
  # 按段登记的仓根（文件头「按段登记」）：合表照抄仓里那一份，按段登记文件只写一行核得过的（第 35 格临时换成写坏的一份、跑完换回）
  local segments_root="$work/segments-root" segments_registry
  segments_registry="$segments_root/.claude/gate.d/crash-point-replay-segments.tsv"
  mkdir -p "$segments_root/.claude/gate.d"
  cp -- "$repository_root/.claude/gate.d/crash-point-replay-items.tsv" "$segments_root/.claude/gate.d/crash-point-replay-items.tsv"
  printf '%s\n' "# 自证的按段登记" "position-addressed-trees	stub	3	300	$(printf '0123456789abcdef%.0s' 1 2 3 4)" >"$segments_registry"
  stub="$work/driver.sh"
  cat >"$stub" <<'STUB'
#!/usr/bin/env bash
# 假的分项驱动：记下参数、收到的按段登记文件路径（给了 STUB_EXPAND_LOG 的另记收到的缩域变量）；形态按 STUB_SHAPE_<测试目标>（没给按 STUB_SHAPE，再没给 single-host）打几行 CRASH_AMPLIFICATION（照真驱动的几种行），
# 每行的 expand_up_to 固定报 7（假的引擎常量，不收展开深度的参数）；测试目标名以 -red 结尾的退 1
printf 'driver %s segments=[%s]\n' "$*" "${SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY-unset}" >> "${STUB_LOG:?}"
# 缩域那几格（第 59、60 格）：记下收到的缩域变量（没设记 unset）
[[ -z "${STUB_EXPAND_LOG:-}" ]] || printf '%s\n' "${SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO-unset}" >> "$STUB_EXPAND_LOG"
# 编译目录那几格（第 49–52 格）：记下收到的 CARGO_TARGET_DIR、往里写一份产物；给了 STUB_SLEEP 睡这么多秒（被 TERM 停那一格）
if [[ -n "${STUB_TARGET_LOG:-}" ]]; then
  printf '%s\n' "${CARGO_TARGET_DIR-unset}" >> "$STUB_TARGET_LOG"
  [[ -z "${CARGO_TARGET_DIR:-}" ]] || { mkdir -p "$CARGO_TARGET_DIR/release" && echo built > "$CARGO_TARGET_DIR/release/artifact"; }
fi
[[ -z "${STUB_SLEEP:-}" ]] || sleep "$STUB_SLEEP"
target="$1"; expand=7
shape_variable="STUB_SHAPE_$(printf '%s' "${target%-red}" | tr -c 'A-Za-z0-9_' '_')"
shape="${!shape_variable:-${STUB_SHAPE:-single-host}}"
case "$shape" in
  single-host) # 本机一台跑完的 fixed-script 汇总行：复用 10 − 7 = 3
    echo "CRASH_AMPLIFICATION mode=single_host_cpu share=0/1 cards=0 states=10 states_checked_here=7 blocks_checked_here=2 unjudged_blocks=0 red=0 expand_up_to=$expand exhaustive=true" ;;
  derived-and-explicit) # 一行推得出（复用 3）、一行带 states_reused=5：这一项复用 8，现判 7 + 2 = 9
    echo "CRASH_AMPLIFICATION mode=single_host_cpu share=0/1 cards=0 states=10 states_checked_here=7 blocks_checked_here=2 unjudged_blocks=0 red=0 expand_up_to=$expand exhaustive=true"
    echo "CRASH_AMPLIFICATION flow=stub mode=single_host_cpu states_checked_here=2 states_reused=5 blocks_checked_here=1 unjudged_blocks=0 red=0 expand_up_to=$expand exhaustive=true" ;;
  flow) # amplify_prepared_flow 打的那种行：不带 states=，取不出复用数
    echo "CRASH_AMPLIFICATION flow=stub mode=single_host_cpu states_checked_here=2 blocks_checked_here=1 unjudged_blocks=0 red=0 violations_in_the_library=0 expand_up_to=$expand exhaustive=true" ;;
  library) # 复用三问那一格（第 45 格）：照 STUB_SERVICE 指的目录（演 prune-server 里那一份判定库）按这一项测试目标的单元文件（$STUB_UNITS/<测试目标>）内容摘要记判过没有：
    #   判过的这一趟现判 0 态、复用 10 态，没判过的现判 10 态并记下（真驱动的判定在服务里按块键记，这里只演「按键存、键相同复用」）
    unit_key="$(sha256sum "${STUB_UNITS:?}/$target" | cut -c1-64)"
    record="${STUB_SERVICE:?}/$target.$unit_key"
    if [[ -f "$record" ]]; then
      echo "CRASH_AMPLIFICATION mode=single_host_cpu share=0/1 cards=0 states=10 states_checked_here=0 blocks_checked_here=0 unjudged_blocks=0 red=0 expand_up_to=$expand exhaustive=true"
    else
      mkdir -p "$STUB_SERVICE" && : >"$record"
      echo "CRASH_AMPLIFICATION mode=single_host_cpu share=0/1 cards=0 states=10 states_checked_here=10 blocks_checked_here=2 unjudged_blocks=0 red=0 expand_up_to=$expand exhaustive=true"
    fi ;;
  silent) ;; # cargo 筛出 0 条：退 0、一行都不打
  placement-not-run) # 排派活计划那一趟借不到显存、一项都没跑到：照真驱动打 DECISION 与 CRASH_AMPLIFICATION_NOT_RUN 行、退 77，不打 CRASH_AMPLIFICATION 行
    echo "CRASH_AMPLIFICATION_DECISION where=not-run reason=no-card-of-the-pool-had-room-in-time across_machines=0"
    echo "CRASH_AMPLIFICATION_NOT_RUN step=placement test=crash_amplification::run_from_environment reason=等了 1800 秒，池里没有一张卡空得出 2560 MiB（假驱动）"
    echo "  ! 本次未跑（这不是通过）：排派活计划那一趟借不到显存，一块都没判（假驱动）" >&2
    exit 77 ;;
  partial-not-run) # 一条用例三条流、第 2 条借不到显存：借到的照判（一行判完的汇总行），没借到的那一条照真驱动打 CRASH_AMPLIFICATION_NOT_RUN 行，退 0
    echo "CRASH_AMPLIFICATION_NOT_RUN step=placement test=crash_amplification::run_from_environment flow=2/3 reason=等了 1800 秒，池里没有一张卡空得出 5120 MiB（假驱动）"
    echo "CRASH_AMPLIFICATION mode=single_host_cpu share=0/1 cards=0 states=10 states_checked_here=7 blocks_checked_here=2 unjudged_blocks=0 red=0 expand_up_to=$expand exhaustive=true" ;;
  source-changed-not-run) # 判段跑的过程中源码变了：照真分项驱动打 CRASH_AMPLIFICATION_NOT_RUN step=judge … reason=跑的过程中源码变了…、退 77，不打 CRASH_AMPLIFICATION 行
    echo "CRASH_AMPLIFICATION_NOT_RUN step=judge test=crash_amplification::run_from_environment reason=跑的过程中源码变了，判段没在录入段那份源码上判完（假驱动）"
    echo "  ! 本次未跑（这不是通过）：跑的过程中源码变了，判段没在录入段那份源码上判完（假驱动）" >&2
    exit 77 ;;
  judge-not-run) # 不带 gpu 特性的二进制遇到多机配置 ENABLE_GPU=1：判段这一趟不跑、录入段照录，分项驱动转打那一条流的 CRASH_AMPLIFICATION_NOT_RUN step=judge 行、退 0，没有 CRASH_AMPLIFICATION 行
    echo "CRASH_AMPLIFICATION_NOT_RUN step=judge test=crash_amplification::run_from_environment flow=stub reason=配置写了 ENABLE_GPU=1，这份二进制没有编进 gpu 特性：判段这一趟不跑（假驱动）" ;;
  not-exhaustive) # 展开深度不够把每一段展开完
    echo "CRASH_AMPLIFICATION mode=single_host_cpu share=0/1 cards=0 states=10 states_checked_here=7 blocks_checked_here=2 unjudged_blocks=0 red=0 expand_up_to=$expand exhaustive=false" ;;
  not-exhaustive-flow) # 一条用例两条流：stub 那条有一段长过展开上限（exhaustive=false），other 那条穷举
    echo "CRASH_AMPLIFICATION flow=stub mode=single_host_cpu states_checked_here=5 states_reused=1 blocks_checked_here=1 unjudged_blocks=0 red=0 violations_in_the_library=0 expand_up_to=$expand exhaustive=false"
    echo "CRASH_AMPLIFICATION flow=other mode=single_host_cpu states_checked_here=3 states_reused=0 blocks_checked_here=1 unjudged_blocks=0 red=0 violations_in_the_library=0 expand_up_to=$expand exhaustive=true" ;;
  registered-flow) # 一条用例两条流：stub 那条登了第 3 段（300 写，exhaustive=false 只因它：别的段都盖得住），other 那条穷举、没登段
    echo "CRASH_AMPLIFICATION flow=stub mode=single_host_cpu states_checked_here=5 blocks_checked_here=1 unjudged_blocks=0 red=0 violations_in_the_library=0 states_reused=1 exhaustive=false expand_up_to=$expand ignored_points=0 states_skipped_by_ignore=0 registered_segments=3:300 unregistered_segments_beyond_expansion=- registered_segment_domains=3:full registered_segment_progress=-"
    echo "CRASH_AMPLIFICATION flow=other mode=single_host_cpu states_checked_here=3 blocks_checked_here=1 unjudged_blocks=0 red=0 violations_in_the_library=0 states_reused=0 exhaustive=true expand_up_to=$expand ignored_points=0 states_skipped_by_ignore=0 registered_segments=- unregistered_segments_beyond_expansion=- registered_segment_domains=- registered_segment_progress=-" ;;
  registered-and-unregistered) # 一条用例两条流：stub 那条登了第 3 段（300 写）；other 那条也不穷举（第 0 段 40 写），却一段都没登（registered_segments=-）
    echo "CRASH_AMPLIFICATION flow=stub mode=single_host_cpu states_checked_here=5 blocks_checked_here=1 unjudged_blocks=0 red=0 violations_in_the_library=0 states_reused=1 exhaustive=false expand_up_to=$expand ignored_points=0 states_skipped_by_ignore=0 registered_segments=3:300 unregistered_segments_beyond_expansion=- registered_segment_domains=3:full registered_segment_progress=-"
    echo "CRASH_AMPLIFICATION flow=other mode=single_host_cpu states_checked_here=3 blocks_checked_here=1 unjudged_blocks=0 red=0 violations_in_the_library=0 states_reused=0 exhaustive=false expand_up_to=$expand ignored_points=0 states_skipped_by_ignore=0 registered_segments=- unregistered_segments_beyond_expansion=0:40 registered_segment_domains=- registered_segment_progress=-" ;;
  registered-beyond) # 一条用例两条流：stub 那条登了第 0 段（322 写），第 1 段（290 写）没登、展开上限盖不住；other 那条穷举
    echo "CRASH_AMPLIFICATION flow=stub mode=single_host_cpu states_checked_here=5 blocks_checked_here=1 unjudged_blocks=0 red=0 violations_in_the_library=0 states_reused=1 exhaustive=false expand_up_to=$expand ignored_points=0 states_skipped_by_ignore=0 registered_segments=0:322 unregistered_segments_beyond_expansion=1:290 registered_segment_domains=0:full registered_segment_progress=-"
    echo "CRASH_AMPLIFICATION flow=other mode=single_host_cpu states_checked_here=3 blocks_checked_here=1 unjudged_blocks=0 red=0 violations_in_the_library=0 states_reused=0 exhaustive=true expand_up_to=$expand ignored_points=0 states_skipped_by_ignore=0 registered_segments=- unregistered_segments_beyond_expansion=- registered_segment_domains=- registered_segment_progress=-" ;;
  registered-old-engine) # 旧引擎打的行：stub 那条登了第 3 段（exhaustive=false），没报 unregistered_segments_beyond_expansion=；other 那条穷举
    echo "CRASH_AMPLIFICATION flow=stub mode=single_host_cpu states_checked_here=5 blocks_checked_here=1 unjudged_blocks=0 red=0 violations_in_the_library=0 states_reused=1 exhaustive=false expand_up_to=$expand ignored_points=0 states_skipped_by_ignore=0 registered_segments=3:300"
    echo "CRASH_AMPLIFICATION flow=other mode=single_host_cpu states_checked_here=3 blocks_checked_here=1 unjudged_blocks=0 red=0 violations_in_the_library=0 states_reused=0 exhaustive=true expand_up_to=$expand ignored_points=0 states_skipped_by_ignore=0 registered_segments=-" ;;
  registered-tail) # 一条用例两条流：stub 那条登了段、exhaustive=false，登记段那几个字段整段取 STUB_REGISTERED_TAIL（各格照要造的驱动行给）；other 那条穷举、没登段
    #   stub 那条的现判态数、现判块数与复用态数取 STUB_TAIL_CHECKED、STUB_TAIL_BLOCKS、STUB_TAIL_REUSED（没给是 5、1、1）
    echo "CRASH_AMPLIFICATION flow=stub mode=single_host_cpu states_checked_here=${STUB_TAIL_CHECKED:-5} blocks_checked_here=${STUB_TAIL_BLOCKS:-1} unjudged_blocks=0 red=0 violations_in_the_library=0 states_reused=${STUB_TAIL_REUSED:-1} exhaustive=false expand_up_to=$expand ignored_points=0 states_skipped_by_ignore=0 ${STUB_REGISTERED_TAIL:?}"
    echo "CRASH_AMPLIFICATION flow=other mode=single_host_cpu states_checked_here=3 blocks_checked_here=1 unjudged_blocks=0 red=0 violations_in_the_library=0 states_reused=0 exhaustive=true expand_up_to=$expand ignored_points=0 states_skipped_by_ignore=0 registered_segments=- unregistered_segments_beyond_expansion=- registered_segment_domains=- registered_segment_progress=-" ;;
  stopped) # 被叫停：判了一部分，留着 2 块没判
    echo "CRASH_AMPLIFICATION mode=single_host_cpu share=0/1 cards=0 states=10 states_checked_here=4 blocks_checked_here=1 unjudged_blocks=2 red=0 expand_up_to=$expand exhaustive=true stopped_early=true" ;;
  unjudged-flow) # 一台跑的流留着 3 块没判
    echo "CRASH_AMPLIFICATION flow=stub mode=single_host_cpu states_checked_here=2 blocks_checked_here=1 unjudged_blocks=3 red=0 violations_in_the_library=0 expand_up_to=$expand exhaustive=true" ;;
  reddish) # 驱动退 0，行里却报了 2 处红
    echo "CRASH_AMPLIFICATION mode=single_host_cpu share=0/1 cards=0 states=10 states_checked_here=7 blocks_checked_here=2 unjudged_blocks=0 red=2 expand_up_to=$expand exhaustive=true" ;;
  device) # 显卡用不上：驱动退 78，不打汇总行
    echo "  ✗ 点名的卡用不了（假驱动）" >&2
    exit 78 ;;
  red-then-device) # 双机驱动本机那一份失败时的样子：先打出已判的行（报了 2 处红），再照日志分类退 78
    echo "CRASH_AMPLIFICATION mode=single_host_cpu share=0/1 cards=0 states=10 states_checked_here=7 blocks_checked_here=2 unjudged_blocks=0 red=2 expand_up_to=$expand exhaustive=true"
    echo "GPU_CARD_NOT_USABLE index=1（假驱动）"
    exit 78 ;;
  ignored) # 照驱动行的约定带全五个字段：复用 3、按登记整批不判 2 个点 50 态
    echo "CRASH_AMPLIFICATION mode=single_host_cpu share=0/1 cards=0 states=10 states_checked_here=7 blocks_checked_here=2 unjudged_blocks=0 red=0 states_reused=3 exhaustive=true expand_up_to=$expand ignored_points=2 states_skipped_by_ignore=50" ;;
  no-red-field) # 行里没有 red=（驱动行的约定坏了）
    echo "CRASH_AMPLIFICATION mode=single_host_cpu share=0/1 cards=0 states=10 states_checked_here=7 blocks_checked_here=2 unjudged_blocks=0 red_states=3 violations=3 expand_up_to=$expand exhaustive=true" ;;
  red-not-a-number) # red= 的值不是数
    echo "CRASH_AMPLIFICATION mode=single_host_cpu share=0/1 cards=0 states=10 states_checked_here=7 blocks_checked_here=2 unjudged_blocks=0 red=? expand_up_to=$expand exhaustive=true" ;;
  red-negative) # red= 的值是负数
    echo "CRASH_AMPLIFICATION mode=single_host_cpu share=0/1 cards=0 states=10 states_checked_here=7 blocks_checked_here=2 unjudged_blocks=0 red=-1 expand_up_to=$expand exhaustive=true" ;;
  zero) # 一态都没判到：现判 0、复用 0
    echo "CRASH_AMPLIFICATION flow=stub mode=single_host_cpu states_checked_here=0 states_reused=0 blocks_checked_here=0 unjudged_blocks=0 red=0 expand_up_to=$expand exhaustive=true" ;;
  placement-then-whole) # 排派活计划那一趟的计划行带 states_reused=（它不是 CRASH_AMPLIFICATION 行，不取），之后本机整份那一趟判 60 态、复用 40 态：这一项现判 60、复用 40
    echo "CRASH_AMPLIFICATION_PLACEMENT host=local judge=gpu pending_blocks=6 pending_states=60 states_reused=40 tables_mebibytes=1 cards=local:0:100:100 states_per_second_on_this_host=10 estimated_seconds_on_this_host=6"
    echo "CRASH_AMPLIFICATION mode=single_host_cpu_with_gpus share=0/1 cards=1 states=100 states_checked_here=60 blocks_checked_here=6 unjudged_blocks=0 red=0 states_reused=40 exhaustive=true expand_up_to=$expand ignored_points=0 states_skipped_by_ignore=0" ;;
  two-flows-with-placement) # 两条流各一趟、各带一行计划行：a 判 14 复用 16、b 判 8 复用 2；计划行的 states_reused= 不取。这一项现判 22、复用 18（连计划行一起加是 36）
    echo "CRASH_AMPLIFICATION_PLACEMENT host=local judge=gpu pending_blocks=2 pending_states=14 states_reused=16 tables_mebibytes=1 cards=local:0:100:100 states_per_second_on_this_host=7 estimated_seconds_on_this_host=2"
    echo "CRASH_AMPLIFICATION_PLACEMENT host=local judge=gpu pending_blocks=1 pending_states=8 states_reused=2 tables_mebibytes=1 cards=local:0:100:100 states_per_second_on_this_host=8 estimated_seconds_on_this_host=1"
    echo "CRASH_AMPLIFICATION flow=a mode=single_host_cpu states_checked_here=14 blocks_checked_here=2 unjudged_blocks=0 red=0 violations_in_the_library=0 states_reused=16 exhaustive=true expand_up_to=$expand ignored_points=0 states_skipped_by_ignore=0"
    echo "CRASH_AMPLIFICATION flow=b mode=single_host_cpu states_checked_here=8 blocks_checked_here=1 unjudged_blocks=0 red=0 violations_in_the_library=0 states_reused=2 exhaustive=true expand_up_to=$expand ignored_points=0 states_skipped_by_ignore=0" ;;
  two-flows-unjudged) # 两条流各一趟、各留了块：a 判 4 复用 0 留 2 块、b 判 7 复用 1 留 1 块：这一项现判 11、复用 1、留 3 块没判，本次未跑
    echo "CRASH_AMPLIFICATION flow=a mode=single_host_cpu states_checked_here=4 blocks_checked_here=1 unjudged_blocks=2 red=0 violations_in_the_library=0 states_reused=0 exhaustive=true expand_up_to=$expand ignored_points=0 states_skipped_by_ignore=0"
    echo "CRASH_AMPLIFICATION flow=b mode=single_host_cpu states_checked_here=7 blocks_checked_here=2 unjudged_blocks=1 red=0 violations_in_the_library=0 states_reused=1 exhaustive=true expand_up_to=$expand ignored_points=0 states_skipped_by_ignore=0" ;;
  ignored-two-flows) # 两条流各一份（a 报 1 个点 10 态、b 报 2 个点 20 态），a 流有两行（叫停再续这类）报同一份：每组取一次，合计 3 个点、30 态
    echo "CRASH_AMPLIFICATION flow=a mode=single_host_cpu states_checked_here=8 blocks_checked_here=1 unjudged_blocks=0 red=0 violations_in_the_library=0 states_reused=12 exhaustive=true expand_up_to=$expand ignored_points=1 states_skipped_by_ignore=10"
    echo "CRASH_AMPLIFICATION flow=b mode=single_host_cpu states_checked_here=5 blocks_checked_here=1 unjudged_blocks=0 red=0 violations_in_the_library=0 states_reused=0 exhaustive=true expand_up_to=$expand ignored_points=2 states_skipped_by_ignore=20"
    echo "CRASH_AMPLIFICATION flow=a mode=single_host_cpu states_checked_here=6 blocks_checked_here=1 unjudged_blocks=0 red=0 violations_in_the_library=0 states_reused=4 exhaustive=true expand_up_to=$expand ignored_points=1 states_skipped_by_ignore=10" ;;
  ignored-no-flow-twice) # 两行都不带 flow=（归一组），各报 2 个点 50 态：2 个点、50 态（按行相加是 4 个点、100 态）
    echo "CRASH_AMPLIFICATION mode=single_host_cpu share=0/1 cards=0 states=10 states_checked_here=7 blocks_checked_here=2 unjudged_blocks=0 red=0 states_reused=3 exhaustive=true expand_up_to=$expand ignored_points=2 states_skipped_by_ignore=50"
    echo "CRASH_AMPLIFICATION mode=single_host_cpu share=0/1 cards=0 states=10 states_checked_here=7 blocks_checked_here=2 unjudged_blocks=0 red=0 states_reused=3 exhaustive=true expand_up_to=$expand ignored_points=2 states_skipped_by_ignore=50" ;;
  ignored-flow-differs) # a 流两行报的不同（1 个点 10 态与 1 个点 11 态），b 流一份 2 个点 20 态：这一项两个数记 unknown、点名 a 流，不悄悄取最大或最小
    echo "CRASH_AMPLIFICATION flow=a mode=single_host_cpu states_checked_here=8 blocks_checked_here=1 unjudged_blocks=0 red=0 violations_in_the_library=0 states_reused=12 exhaustive=true expand_up_to=$expand ignored_points=1 states_skipped_by_ignore=10"
    echo "CRASH_AMPLIFICATION flow=b mode=single_host_cpu states_checked_here=5 blocks_checked_here=1 unjudged_blocks=0 red=0 violations_in_the_library=0 states_reused=0 exhaustive=true expand_up_to=$expand ignored_points=2 states_skipped_by_ignore=20"
    echo "CRASH_AMPLIFICATION flow=a mode=single_host_cpu states_checked_here=6 blocks_checked_here=1 unjudged_blocks=0 red=0 violations_in_the_library=0 states_reused=4 exhaustive=true expand_up_to=$expand ignored_points=1 states_skipped_by_ignore=11" ;;
esac
if [[ "$target" == *-red ]]; then exit 1; fi
exit 0
STUB
  chmod +x "$stub"
  # 假 cargo（自证用）：入口自己不起 cargo，只有指定范围的崩溃域上限起读库程序 crash_library_reader 的 item 子命令（lib_crash_point_replay_items.py 的 scoped-domain）时才起。
  #   按 STUB_ITEM_RECORDED、STUB_ITEM_TO_JUDGE、STUB_ITEM_REUSABLE 答块账（不给是录下的块全部可复用），设点文件记进 STUB_ITEM_CALLS；只问版本的答版本；别的一律退 99。
  #   不写 rocksdb-prebuilt.py 认的假 cargo 标记：第 48 格要它照常判（PATH 最前面放它，后面所有格起的入口、读库程序都找得到它）
  mkdir -p "$work/bin"
  cat >"$work/bin/cargo" <<'STUB'
#!/usr/bin/env bash
[[ "${1:-}" == -V || "${1:-}" == --version ]] && { echo "cargo 0.0.0-selftest"; exit 0; }
if [[ " $* " == *" --bin crash_library_reader -- item "* ]]; then
  stub_item_name=""; previous=""
  for argument in "$@"; do [[ "$previous" == item ]] && { stub_item_name="$argument"; break; }; previous="$argument"; done
  [[ -z "${STUB_ITEM_CALLS:-}" ]] || printf 'item %s\n' "${*#* -- item }" >>"$STUB_ITEM_CALLS"
  printf 'CRASH_LIBRARY_ITEM\titem=%s\trecorded_blocks=%s\tto_judge=%s\treusable=%s\n' "$stub_item_name" "${STUB_ITEM_RECORDED:-1}" "${STUB_ITEM_TO_JUDGE:-0}" "${STUB_ITEM_REUSABLE:-1}"
  exit 0
fi
echo "假 cargo 只答读库程序的 item 子命令，收到：$*" >&2
exit 99
STUB
  chmod +x "$work/bin/cargo"
  export PATH="$work/bin:$PATH"
  if [[ "$(type -P cargo)" != "$work/bin/cargo" ]]; then
    echo "  ✗ 自证 setup：PATH 最前面放了假 cargo，解到的却不是它（$(type -P cargo)），不起子进程（起了就是真 cargo）" >&2
    echo "  → 怎么办：看 $work/bin/cargo 可不可执行、TMPDIR 是不是挂成了 noexec" >&2
    exit 1
  fi
  # 入口不清场（文件头「清场」）：多机配置里只写假的清场命令，清场与复原各往 QUIESCE_LOG 记一笔，
  #   第 1、2、6、28 格与拒跑那几格核 QUIESCE_LOG 一笔都没有。包装的登记与本机日志都放这里的临时目录，不落进真工作根
  printf '%s\n' \
    "QUIESCE_STOP_COMMAND='printf \"quiesce stop\\\\n\" >> \"\$QUIESCE_LOG\"'" \
    "QUIESCE_STOPPED_CHECK_COMMAND='[[ \"\${QUIESCE_STOPPED_CHECK:-ok}\" == ok ]]'" \
    "QUIESCE_START_COMMAND='printf \"quiesce start\\\\n\" >> \"\$QUIESCE_LOG\"'" \
    "QUIESCE_STARTED_CHECK_COMMAND='[[ \"\${QUIESCE_STARTED_CHECK:-ok}\" == ok ]]'" >"$SINGLEFS_MULTI_HOST_CONFIG"
  log="$work/calls.log"
  export QUIESCE_LOG="$work/quiesce.log" GPU_QUIESCE_STATE_DIRECTORY="$work/gpu-quiesce-state" GPU_QUIESCE_LOG_DIRECTORY="$work/gpu-quiesce-logs"
  : > "$log"; : > "$QUIESCE_LOG"
  local failures=0 cells=0
  selftest_cell_failed() { # <这一格判错的一句> <出路>
    echo "  ✗ 自证第 $cells 格：$1" >&2
    echo "     → 怎么办：$2" >&2
    failures=$((failures + 1))
  }
  entry() { CRASH_AMPLIFICATION_ITEM_DRIVER="$stub" STUB_LOG="$log" CRASH_AMPLIFICATION_SELFTEST_SEGMENTS_ROOT="$segments_root" bash "$0" "$@"; } # 用假驱动起一趟入口
  local output errors exit_code first_item
  errors="$work/errors"
  first_item="$(item_fields "${ITEMS[0]}" 1)"
  # 第 1 格：--list-items 每个登记的分项一行、三段，不清场
  cells=$((cells + 1)); : > "$QUIESCE_LOG"
  output="$(bash "$0" --list-items)"; exit_code=$?
  if [[ "$exit_code" != 0 || "$(grep -c . <<<"$output")" != "${#ITEMS[@]}" ]] \
     || [[ "$(awk -F'\t' 'NF == 3 && $1 != "" && $2 != "" && $3 != ""' <<<"$output" | wc -l)" != "${#ITEMS[@]}" ]] || [[ -s "$QUIESCE_LOG" ]]; then
    selftest_cell_failed "--list-items 没有逐行列出登记的分项（退 $exit_code），或它清了场" "看 list_items：每行三段（分项名、测试目标、用例函数），在清场之前就退"
  fi
  # 第 2 格：不认得的分项名：退 2、列出可点名的项，不清场、不起驱动
  cells=$((cells + 1)); : > "$log"; : > "$QUIESCE_LOG"
  entry --item no-such-item >/dev/null 2>"$errors"; exit_code=$?
  if [[ "$exit_code" != 2 ]] || ! grep -q -- "$first_item" "$errors" || [[ -s "$log" || -s "$QUIESCE_LOG" ]]; then
    selftest_cell_failed "不认得的分项名没有退 2 并列出可点名的项（退 $exit_code）" "看选分项那一段：登记表里找不到的名字交 refuse_listing_the_items"
  fi
  # 第 3 格：--item 后面缺项名：退 2、列出可点名的项，不起驱动
  cells=$((cells + 1)); : > "$log"
  entry --item >/dev/null 2>"$errors"; exit_code=$?
  if [[ "$exit_code" != 2 ]] || ! grep -q -- "$first_item" "$errors" || [[ -s "$log" ]]; then
    selftest_cell_failed "--item 后面缺项名没有退 2 并列出可点名的项（退 $exit_code）" "看参数那一段 --item 的分支：缺项名交 refuse_listing_the_items"
  fi
  # 第 4 格：不带参数：每个分项各调一次驱动，每次带 --test-filter、不带展开深度的参数；成功句写「崩溃放量全部项」
  cells=$((cells + 1)); : > "$log"
  output="$(entry 2>/dev/null)"; exit_code=$?
  if [[ "$exit_code" != 0 ]] || [[ "$(wc -l <"$log")" != "${#ITEMS[@]}" ]] || [[ "$(grep -c -- '--test-filter' "$log")" != "${#ITEMS[@]}" ]] \
     || grep -q -- ' --expand' "$log" \
     || ! grep -q "崩溃放量全部项" <<<"$output" || ! grep -q ' scope=whole ' <<<"$output"; then
    selftest_cell_failed "不带参数没把每个分项各调一次驱动、带 --test-filter、不带展开深度的参数，或没退 0、成功句没写全部项（退 $exit_code，调了 $(wc -l <"$log") 次）" \
      "看主循环：不带参数跑 ITEMS 的每一行，filter_arguments 带 --test-filter，driver_arguments 不带展开深度"
  fi
  # 第 5 格：点名一项（给两次取并集）：只调一次、目标对
  cells=$((cells + 1)); : > "$log"
  entry --item tree-split-streams --item tree-split-streams >/dev/null 2>&1
  if [[ "$(wc -l <"$log")" != 1 ]] || ! grep -q 'crash_enumeration_tree_split_streams' "$log"; then
    selftest_cell_failed "--item 没有只跑点名的那一项、或同名给两次跑了两遍（调了 $(wc -l <"$log") 次）" "看选分项那一段：点名的分项只取登记表里同名那一行，重名只取一次"
  fi
  # 第 6 格：不清场（清场由 task-server 管，文件头「清场」）：点名三项各调一次驱动、每次带 --test-filter，不经清场包装重起（QUIESCE_LOG 一笔都没有），退 0
  cells=$((cells + 1)); : > "$log"
  output="$(QUIESCE_LOG="$log" entry --item fixed-script-through-rollback --item multi-record-publish --item tree-split-streams 2>/dev/null)"; exit_code=$?
  local expected_calls="driver-filtered driver-filtered driver-filtered " actual_calls
  actual_calls="$(sed -E 's/^driver .* --test-filter .*$/driver-filtered/; s/^driver .*$/driver-plain/; s/^quiesce (.*)$/quiesce-\1/' "$log" | tr '\n' ' ')"
  if [[ "$exit_code" != 0 || "$actual_calls" != "$expected_calls" ]] || ! grep -q 'CRASH_AMPLIFICATION_TOTAL dispatched=3 items=3 failed=0 not_run=0 ' <<<"$output"; then
    selftest_cell_failed "不该清场：各项应当带 --test-filter、不经清场包装重起（退 $exit_code，调用次序「$actual_calls」）" "看主循环：入口不清场，filter_arguments 带 --test-filter；清场由 task-server 起"
  fi
  # 第 7 格：认不出的选项（用一个从没用过的名字）：用法错退 2、一项都不跑
  cells=$((cells + 1)); : > "$log"
  entry --item tree-split-streams --never-used-option 8 >/dev/null 2>"$work/unknown-option.err"; exit_code=$?
  if [[ "$exit_code" != 2 ]] || [[ -s "$log" ]] || ! grep -qF '认不得的参数「--never-used-option」' "$work/unknown-option.err"; then
    selftest_cell_failed "认不出的选项应当用法错退 2、点名它、一项都不跑（退 $exit_code，驱动调了 $(grep -c . "$log") 次）" \
      "看参数那一段最后的 * 分支：认不出的选项交 usage（退 2）"
  fi
  # 第 8 格：驱动退非 0：这一项红、整趟退 1
  cells=$((cells + 1))
  output="$(CRASH_AMPLIFICATION_SELFTEST_RED_ITEM=1 entry --item tree-split-streams 2>/dev/null)"; exit_code=$?
  if [[ "$exit_code" != 1 ]] || ! grep -q 'CRASH_AMPLIFICATION_TOTAL dispatched=1 items=1 failed=1 ' <<<"$output"; then
    selftest_cell_failed "驱动退非 0 没记成红（退 $exit_code）：$output" "看主循环里 item_exit 非 0 时 reasons 有没有加一条、末尾退 1"
  fi
  # 第 9 格：驱动退 0、行里 red=2：这一项红、整趟退 1
  cells=$((cells + 1))
  output="$(STUB_SHAPE=reddish entry --item tree-split-streams 2>/dev/null)"; exit_code=$?
  if [[ "$exit_code" != 1 ]] || ! grep -q 'CRASH_AMPLIFICATION_TOTAL dispatched=1 items=1 failed=1 .* red=2 ' <<<"$output"; then
    selftest_cell_failed "驱动退 0 而 red=2 没记成红（退 $exit_code）：$output" "看主循环里 item_red 大于 0 那一条"
  fi
  # 第 10 格：复用数的取法：一行推得出（10 − 7 = 3）、一行带 states_reused=5，这一项 reused=8；成功句「现判 9 态、复用 8 态」
  cells=$((cells + 1))
  output="$(STUB_SHAPE=derived-and-explicit entry --item tree-split-streams 2>/dev/null)"; exit_code=$?
  if [[ "$exit_code" != 0 ]] || ! grep -q '^CRASH_AMPLIFICATION_ITEM item=tree-split-streams exit=0 states=9 reused=8 red=0 ' <<<"$output" \
     || ! grep -q 'CRASH_AMPLIFICATION_TOTAL .* reuse_unknown=0 states=9 reused_known=8 red=0 ' <<<"$output" \
     || ! grep -q '现判 9 态、复用 8 态；派出 1 项、收回 1 项' <<<"$output"; then
    selftest_cell_failed "复用数取错（退 $exit_code）：$output" "看 summarize_the_item：states_reused= 照取、本机一台跑完的行取 states − states_checked_here，几行相加"
  fi
  # 第 11 格：flow= 那种行不带 states=：reused=unknown、计进 reuse_unknown，不红、退 0；成功句点名它，不写「复用 0 态」
  cells=$((cells + 1))
  output="$(STUB_SHAPE=flow entry --item multi-record-publish 2>/dev/null)"; exit_code=$?
  if [[ "$exit_code" != 0 ]] || ! grep -q '^CRASH_AMPLIFICATION_ITEM item=multi-record-publish exit=0 states=2 reused=unknown ' <<<"$output" \
     || ! grep -q 'CRASH_AMPLIFICATION_TOTAL .* reuse_unknown=1 states=2 reused_known=0 ' <<<"$output" \
     || ! grep -q '现判 2 态、复用数取不出（1 项：multi-record-publish，驱动行没报 states_reused=）；派出 1 项、收回 1 项' <<<"$output" \
     || grep -q '复用 0 态' <<<"$output"; then
    selftest_cell_failed "取不出复用数的项没照实报出（退 $exit_code）：$output" "看主循环：reused 是 unknown 的项计进 items_reuse_unknown 与 reuse_unknown_items、不红，成功句点名它；不许记成 0"
  fi
  # 第 12 格：叫停的行（stopped_early=true、带 states= 与 states_checked_here=、没报 states_reused=）里 states − states_checked_here 混着没判的块，不拿来当复用数：
  #   点名两项，tree-split-streams 叫停、另一项本机整份跑完：叫停那一项 reused=unknown、TOTAL 的 reuse_unknown=1，退 0、成功句点名它取不出
  cells=$((cells + 1))
  output="$(STUB_SHAPE_crash_enumeration_tree_split_streams=stopped entry --item fixed-script-through-rollback --item tree-split-streams 2>/dev/null)"; exit_code=$?
  if [[ "$exit_code" != 0 ]] || ! grep -q '^CRASH_AMPLIFICATION_ITEM item=tree-split-streams exit=0 states=4 reused=unknown ' <<<"$output" \
     || ! grep -q 'CRASH_AMPLIFICATION_TOTAL .* reuse_unknown=1 states=11 ' <<<"$output" \
     || ! grep -q '另有 1 项取不出：tree-split-streams，驱动行没报 states_reused=）' <<<"$output"; then
    selftest_cell_failed "叫停的行被拿来推复用数、或取不出却没照实报出（退 $exit_code）：$output" \
      "看 summarize_the_item：行里没报 states_reused= 时只有带 states= 又没叫停（stopped_early=true）的行才取 states − states_checked_here"
  fi
  # 第 13 格：派出收回：点名两项、造一项收不回，TOTAL 报 dispatched=2 items=1、整趟退 1
  cells=$((cells + 1))
  output="$(CRASH_AMPLIFICATION_SELFTEST_DROP_RESULT=tree-split-streams entry --item fixed-script-through-rollback --item tree-split-streams 2>"$errors")"; exit_code=$?
  if [[ "$exit_code" != 1 ]] || ! grep -q 'CRASH_AMPLIFICATION_TOTAL dispatched=2 items=1 failed=0 ' <<<"$output" || ! grep -q '派出 2 项、收回 1 项' "$errors"; then
    selftest_cell_failed "少收回一项却没判红（退 $exit_code）：$output" "看主循环之后派出收回那一段：items_run 与 items_dispatched 不等要报出两个数、退 1"
  fi
  # 第 14 格：驱动退 0 却 0 行 CRASH_AMPLIFICATION：这一项红、整趟退 1，出路点到测试名与 cargo 筛出 0 条
  cells=$((cells + 1))
  output="$(STUB_SHAPE=silent entry --item tree-split-streams 2>"$errors")"; exit_code=$?
  if [[ "$exit_code" != 1 ]] || ! grep -q 'CRASH_AMPLIFICATION_TOTAL dispatched=1 items=1 failed=1 ' <<<"$output" || ! grep -q '看测试名还在不在' "$errors"; then
    selftest_cell_failed "驱动退 0、0 行却没判红（退 $exit_code）：$output" "看主循环里 item_lines 为 0 那一条"
  fi
  # 第 15 格：行里 exhaustive=false：这一项红、整趟退 1
  cells=$((cells + 1))
  output="$(STUB_SHAPE=not-exhaustive entry --item tree-split-streams 2>"$errors")"; exit_code=$?
  if [[ "$exit_code" != 1 ]] || ! grep -q 'CRASH_AMPLIFICATION_TOTAL dispatched=1 items=1 failed=1 ' <<<"$output" || ! grep -q '不是 exhaustive=true' "$errors"; then
    selftest_cell_failed "有行不是穷举却没判红（退 $exit_code）：$output" "看主循环里不穷举那一条与 summarize_the_item 的不穷举行数：每一行都要 exhaustive=true，或 exhaustive=false 只因登记的段"
  fi
  local success_line
  # 第 16 格：两项里一项被叫停（stopped_early=true、留 2 块）：那一项 verdict=not-run、逐项报本次未跑，整趟退 0；成功句点名它、不说「都退 0、0 红」
  cells=$((cells + 1))
  output="$(STUB_SHAPE_crash_enumeration_tree_split_streams=stopped entry --item fixed-script-through-rollback --item tree-split-streams 2>"$errors")"; exit_code=$?
  if [[ "$exit_code" != 0 ]] || ! grep -q '^CRASH_AMPLIFICATION_ITEM item=tree-split-streams .* verdict=not-run ' <<<"$output" \
     || ! grep -q 'CRASH_AMPLIFICATION_TOTAL dispatched=2 items=2 failed=0 not_run=1 ' <<<"$output" \
     || ! grep -q '本次未跑（这不是通过）：崩溃放量分项 tree-split-streams' "$errors" \
     || ! grep -q '本次未跑 1 项：tree-split-streams' <<<"$output" || grep -q '都退 0、0 红' <<<"$output"; then
    selftest_cell_failed "被叫停的项没按本次未跑逐项报、或成功句还说都退 0、0 红（退 $exit_code）：$output" "看主循环里 item_stopped / item_unjudged 那一支与成功句的 run_part"
  fi
  # 第 17 格：唯一的一项留了没判的块（一台跑的流，unjudged_blocks=3）：一项都没判完，整趟退 77
  cells=$((cells + 1))
  output="$(STUB_SHAPE=unjudged-flow entry --item multi-record-publish 2>/dev/null)"; exit_code=$?
  if [[ "$exit_code" != 77 ]] || ! grep -q 'CRASH_AMPLIFICATION_TOTAL dispatched=1 items=1 failed=0 not_run=1 ' <<<"$output"; then
    selftest_cell_failed "派出的项全没判完却没退 77（退 $exit_code）：$output" "看末尾 items_not_run 等于 items_run 那一支"
  fi
  # 第 18 格：第一项的驱动退 78（卡用不上）：停在这一项、后面不派，整趟退 78，出路写「修好设备，或改配置」
  cells=$((cells + 1)); : > "$log"
  output="$(STUB_SHAPE_crash_amplification_fixed_script_through_rollback=device entry --item fixed-script-through-rollback --item tree-split-streams 2>"$errors")"; exit_code=$?
  if [[ "$exit_code" != 78 ]] || [[ "$(wc -l <"$log")" != 1 ]] || ! grep -q 'CRASH_AMPLIFICATION_TOTAL dispatched=1 items=1 ' <<<"$output" || ! grep -q '修好设备，或改配置' "$errors"; then
    selftest_cell_failed "驱动退 78 却没停下退 78（退 $exit_code，调了 $(wc -l <"$log") 次驱动）：$output" "看主循环里 item_exit 为 78 那一支与末尾 stopped_by_condition"
  fi
  # 第 19 格：平常跑（不是自证起的子进程）时带了只供自证的变量：退 2、不起驱动，拒绝句点名带了的每一个。这一格只判入口拒不拒、
  #   不往下起清场与驱动（没拒的那一趟——弄坏开关 honor-test-variables-outside-selftest、或拒的那一段被删了——也一样）：换驱动的变量指到
  #   一份不在的文件，没拒时入口在「分项驱动不在」那一步就退 2，到不了经清场包装重起那一步；点名的分项也是登记表里没有的，
  #   选分项那一步同样在清场之前退 2。拒没拒只认拒绝句。换驱动的变量每一次都带着、每一次都核它被点名；另外四个各试一次
  cells=$((cells + 1))
  local variable value named refusal refused_all=1 missing_driver="$work/no-such-item-driver.sh"
  for variable in CRASH_AMPLIFICATION_BREAK CRASH_AMPLIFICATION_SELFTEST_RED_ITEM CRASH_AMPLIFICATION_SELFTEST_DROP_RESULT CRASH_AMPLIFICATION_SELFTEST_SEGMENTS_ROOT; do
    value=1
    [[ "$variable" == CRASH_AMPLIFICATION_BREAK ]] && value="${CRASH_AMPLIFICATION_BREAK:-swallow-exit}"   # 外面开着关守卫项时原样交给子进程（它只在唯一一项时认）
    [[ "$variable" == CRASH_AMPLIFICATION_SELFTEST_DROP_RESULT ]] && value=tree-split-streams
    [[ "$variable" == CRASH_AMPLIFICATION_SELFTEST_SEGMENTS_ROOT ]] && value="$segments_root"
    : > "$log"; : > "$QUIESCE_LOG"
    env -u CRASH_AMPLIFICATION_SELFTEST_MARK CRASH_AMPLIFICATION_ITEM_DRIVER="$missing_driver" STUB_LOG="$log" "$variable=$value" \
      bash "$0" --item no-such-item-for-cell-21 >/dev/null 2>"$errors"; exit_code=$?
    if [[ "$exit_code" != 2 ]] || [[ -s "$log" || -s "$QUIESCE_LOG" ]] || grep -q 'gpu-quiesce-run.sh 清场' "$errors"; then refused_all=0; fi
    refusal="$(grep '平常跑时带了只给自证用的变量' "$errors")"
    for named in "$variable" CRASH_AMPLIFICATION_ITEM_DRIVER; do
      grep -qE "(：| )$named( |；)" <<<"$refusal" || refused_all=0
    done
  done
  if (( ! refused_all )); then
    selftest_cell_failed "平常跑时带了只供自证的变量却没拒、或拒绝句没点名带了的那几个（最后一次退 $exit_code）" "看参数解析之前拒只供自证变量的那一段：running_under_selftest 为 0 且第一个参数不是 --selftest 时六个变量有一个非空就逐个点名、退 2"
  fi
  # 第 20 格：前一项红（驱动退 0、red=2）、后一项驱动退 78：停在退 78 那一项、后面不派，退 78 的那一项在 ✗ 里点名，整趟退 1（有红先判）
  cells=$((cells + 1)); : > "$log"
  output="$(STUB_SHAPE_crash_amplification_fixed_script_through_rollback=reddish STUB_SHAPE_crash_amplification_fixed_script_through_unmount=device \
    entry --item fixed-script-through-rollback --item fixed-script-through-unmount --item tree-split-streams 2>"$errors")"; exit_code=$?
  if [[ "$exit_code" != 1 ]] || [[ "$(wc -l <"$log")" != 2 ]] \
     || ! grep -q '^CRASH_AMPLIFICATION_ITEM item=fixed-script-through-unmount exit=78 .* verdict=not-run ' <<<"$output" \
     || ! grep -q 'CRASH_AMPLIFICATION_TOTAL dispatched=2 items=2 failed=1 not_run=1 ' <<<"$output" \
     || ! grep -q '分项 fixed-script-through-unmount 的驱动退 78.*后面 1 项没派' "$errors"; then
    selftest_cell_failed "有红又有一项退 78 时没退 1、或退 78 那一项没点名、或后面的项照派了（退 $exit_code，调了 $(wc -l <"$log") 次驱动）：$output" \
      "看末尾 stopped_by_condition 那一段：出路句照打，有红（run_verdict）就退 1，没有红才退 78"
  fi
  # 第 21 格：驱动打了一行 red=2 之后退 78：这一项记红（不让 78 盖掉），停在这一项、后面不派，整趟退 1
  cells=$((cells + 1)); : > "$log"
  output="$(STUB_SHAPE_crash_amplification_fixed_script_through_rollback=red-then-device entry --item fixed-script-through-rollback --item tree-split-streams 2>"$errors")"; exit_code=$?
  if [[ "$exit_code" != 1 ]] || [[ "$(wc -l <"$log")" != 1 ]] \
     || ! grep -q '^CRASH_AMPLIFICATION_ITEM item=fixed-script-through-rollback exit=78 .* red=2 .* verdict=red ' <<<"$output" \
     || ! grep -q 'CRASH_AMPLIFICATION_TOTAL dispatched=1 items=1 failed=1 not_run=0 ' <<<"$output" \
     || ! grep -q '分项 fixed-script-through-rollback 的驱动退 78' "$errors"; then
    selftest_cell_failed "驱动退 78 而行里报了红，这一项没记红、或没停在这一项（退 $exit_code，调了 $(wc -l <"$log") 次驱动）：$output" \
      "看主循环里 item_exit 为 78 那一支：red_reasons 不空就记 red，照样停在这一项"
  fi
  # 第 22 格：全量、每一行都带 ignored_points=2 states_skipped_by_ignore=50：ITEM 与 TOTAL 照实写出，
  #   成功句报「按登记整批不判：<点数> 个点、<态数> 态」，写明类名取不出、点名有跳过的分项
  cells=$((cells + 1))
  local item_count="${#ITEMS[@]}"
  output="$(STUB_SHAPE=ignored entry 2>/dev/null)"; exit_code=$?
  success_line="$(grep '✓' <<<"$output")"
  if [[ "$exit_code" != 0 ]] || [[ "$(grep -c '^CRASH_AMPLIFICATION_ITEM .* ignored_points=2 states_skipped_by_ignore=50 registered_segments=- unregistered_segments_beyond_expansion=unknown registered_segment_domains=- registered_segment_progress=-$' <<<"$output")" != "$item_count" ]] \
     || ! grep -q "CRASH_AMPLIFICATION_TOTAL .* ignored_points=$((2 * item_count)) states_skipped_by_ignore=$((50 * item_count)) ignore_unknown=0\$" <<<"$output" \
     || ! grep -q "按登记整批不判：$((2 * item_count)) 个点、$((50 * item_count)) 态（类名取不出" <<<"$success_line" \
     || ! grep -q "有跳过的分项：$first_item" <<<"$success_line"; then
    selftest_cell_failed "按登记整批不判的点数与态数没照实写进 ITEM、TOTAL 与成功句（退 $exit_code）：$output" \
      "看 summarize_the_item 里 ignored_points / states_skipped_by_ignore 两个键、主循环的 total_ignored_points 与成功句的 ignore_part"
  fi
  # 第 23 格：点名两项，一项带这两个数、一项的驱动行没报：没报的那一项写 unknown、计进 ignore_unknown，不红、退 0；成功句点名它，不记成 0
  cells=$((cells + 1))
  output="$(STUB_SHAPE_crash_amplification_fixed_script_through_rollback=ignored entry --item fixed-script-through-rollback --item tree-split-streams 2>/dev/null)"; exit_code=$?
  success_line="$(grep '✓' <<<"$output")"
  if [[ "$exit_code" != 0 ]] || ! grep -q '^CRASH_AMPLIFICATION_ITEM item=tree-split-streams .* verdict=green .* ignored_points=unknown states_skipped_by_ignore=unknown registered_segments=- unregistered_segments_beyond_expansion=unknown registered_segment_domains=- registered_segment_progress=-$' <<<"$output" \
     || ! grep -q 'CRASH_AMPLIFICATION_TOTAL .* ignored_points=2 states_skipped_by_ignore=50 ignore_unknown=1$' <<<"$output" \
     || ! grep -q '按登记整批不判：取得出的 1 项 2 个点、50 态（类名取不出.*）；另有 1 项取不出：tree-split-streams' <<<"$success_line"; then
    selftest_cell_failed "驱动行没报按登记整批不判的两个数，却没照实报取不出（退 $exit_code）：$output" \
      "看 summarize_the_item：两个数有一行没报就整项写 unknown；主循环计进 items_ignore_unknown，成功句点名它"
  fi
  # 第 24 格：驱动行缺 red=、red=?、red=-1：各判红、整趟退 1，出路点到驱动行的约定
  cells=$((cells + 1))
  local shape bad_red_all=1
  for shape in no-red-field red-not-a-number red-negative; do
    output="$(STUB_SHAPE="$shape" entry --item tree-split-streams 2>"$errors")"; exit_code=$?
    if [[ "$exit_code" != 1 ]] || ! grep -q 'CRASH_AMPLIFICATION_TOTAL dispatched=1 items=1 failed=1 ' <<<"$output" || ! grep -q '缺 red= 或 red 不是非负整数' "$errors"; then
      bad_red_all=0
    fi
  done
  if (( ! bad_red_all )); then
    selftest_cell_failed "驱动行缺 red= 或 red 不是非负整数却没判红（最后一次退 $exit_code）：$output" "看 summarize_the_item 的 bad_red 与主循环里 item_bad_red 那一条"
  fi
  # 第 25 格：不带参数，每一项现判 0 态、复用 0 态：每项判红、整趟退 1
  cells=$((cells + 1))
  output="$(STUB_SHAPE=zero entry 2>"$errors")"; exit_code=$?
  if [[ "$exit_code" != 1 ]] || ! grep -q "CRASH_AMPLIFICATION_TOTAL dispatched=$item_count items=$item_count failed=$item_count .* states=0 reused_known=0 " <<<"$output" \
     || ! grep -q '现判 0 态、复用 0 态' "$errors"; then
    selftest_cell_failed "不带参数时一项现判 0 态、复用 0 态却没判红（退 $exit_code）：$output" "看主循环里 scope 为 whole、item_states 为 0、item_reused 为 0 那一条"
  fi
  # 第 26 格：复用数是各行之和，计划行不取（文件头「复用数怎么取」）。点名两项：一项一行计划行（带 states_reused=40）加本机整份那一趟（判 60 复用 40）；
  #   一项两条流各一趟、各带一行计划行（a 判 14 复用 16、b 判 8 复用 2）。两项与 TOTAL 逐个字段钉绝对值：现判 60 复用 40、现判 22 复用 18，合计现判 82 复用 58，都绿、退 0
  cells=$((cells + 1))
  output="$(STUB_SHAPE_crash_amplification_fixed_script_through_rollback=placement-then-whole STUB_SHAPE_crash_enumeration_tree_split_streams=two-flows-with-placement \
    entry --item fixed-script-through-rollback --item tree-split-streams 2>/dev/null)"; exit_code=$?
  if [[ "$exit_code" != 0 ]] \
     || ! grep -qE "^CRASH_AMPLIFICATION_ITEM item=fixed-script-through-rollback exit=0 states=60 reused=40 red=0 lines=1 expand_up_to=7 exhaustive=true verdict=green seconds=[0-9]+ ignored_points=0 states_skipped_by_ignore=0 registered_segments=- unregistered_segments_beyond_expansion=unknown registered_segment_domains=- registered_segment_progress=-\$" <<<"$output" \
     || ! grep -qE "^CRASH_AMPLIFICATION_ITEM item=tree-split-streams exit=0 states=22 reused=18 red=0 lines=2 expand_up_to=7 exhaustive=true verdict=green seconds=[0-9]+ ignored_points=0 states_skipped_by_ignore=0 registered_segments=- unregistered_segments_beyond_expansion=unknown registered_segment_domains=- registered_segment_progress=-\$" <<<"$output" \
     || ! grep -qE "^CRASH_AMPLIFICATION_TOTAL dispatched=2 items=2 failed=0 not_run=0 reuse_unknown=0 states=82 reused_known=58 red=0 scope=items seconds=[0-9]+ ignored_points=0 states_skipped_by_ignore=0 ignore_unknown=0\$" <<<"$output" \
     || ! grep -q '2 项都退 0、0 红；现判 82 态、复用 58 态；派出 2 项、收回 2 项' <<<"$output"; then
    selftest_cell_failed "复用数没照各行之和取：有一行的复用数没加上、或计划行的复用数被加了进去（退 $exit_code）：$output" \
      "看 summarize_the_item：只读 CRASH_AMPLIFICATION 行（CRASH_AMPLIFICATION_PLACEMENT 行不取），各行的复用数相加"
  fi
  # 第 27 格：几条流各留了块（文件头「一项怎么判」not-run）。点名两项：一项两条流各留块（a 留 2、b 留 1），记 not-run、留 3 块；另一项两条流都判完，记 green。
  #   逐字段钉绝对值，整趟退 0
  cells=$((cells + 1))
  output="$(STUB_SHAPE_crash_enumeration_tree_split_streams=two-flows-with-placement STUB_SHAPE_crash_enumeration_multi_record_publish_stream=two-flows-unjudged \
    entry --item multi-record-publish --item tree-split-streams 2>"$errors")"; exit_code=$?
  if [[ "$exit_code" != 0 ]] \
     || ! grep -qE "^CRASH_AMPLIFICATION_ITEM item=multi-record-publish exit=0 states=11 reused=1 red=0 lines=2 expand_up_to=7 exhaustive=true verdict=not-run seconds=[0-9]+ ignored_points=0 states_skipped_by_ignore=0 registered_segments=- unregistered_segments_beyond_expansion=unknown registered_segment_domains=- registered_segment_progress=-\$" <<<"$output" \
     || ! grep -qE "^CRASH_AMPLIFICATION_ITEM item=tree-split-streams exit=0 states=22 reused=18 red=0 lines=2 expand_up_to=7 exhaustive=true verdict=green seconds=[0-9]+ ignored_points=0 states_skipped_by_ignore=0 registered_segments=- unregistered_segments_beyond_expansion=unknown registered_segment_domains=- registered_segment_progress=-\$" <<<"$output" \
     || ! grep -qE "^CRASH_AMPLIFICATION_TOTAL dispatched=2 items=2 failed=0 not_run=1 reuse_unknown=0 states=33 reused_known=19 red=0 scope=items " <<<"$output" \
     || ! grep -q '本次未跑（这不是通过）：崩溃放量分项 multi-record-publish：留了 3 块没判（unjudged_blocks）' "$errors" \
     || grep -q '崩溃放量分项 tree-split-streams' "$errors"; then
    selftest_cell_failed "几条流各留了块，没判的块没照各行之和报本次未跑：留了块的项没记 not-run、块数不对，或判完的项被报成本次未跑（退 $exit_code）：$output" \
      "看 summarize_the_item：各行的 unjudged_blocks 相加；主循环里 item_unjudged 大于 0 的项记 not-run"
  fi
  # 第 28 格：照常跑完退 0、有项红（驱动退 1）整趟退 1；两趟都不清场：QUIESCE_LOG 一笔都没有、输出里没有 GPU_QUIESCE 行
  local not_restored_exit not_restored_calls
  cells=$((cells + 1)); : > "$log"
  QUIESCE_LOG="$log" entry --item tree-split-streams >/dev/null 2>"$errors"; not_restored_exit=$?
  QUIESCE_LOG="$log" CRASH_AMPLIFICATION_SELFTEST_RED_ITEM=1 entry --item tree-split-streams >/dev/null 2>>"$errors"; exit_code=$?
  not_restored_calls="$(sed -E 's/^driver .*$/driver/; s/^quiesce (.*)$/quiesce-\1/' "$log" | tr '\n' ' ')"
  if [[ "$not_restored_exit" != 0 || "$exit_code" != 1 || "$not_restored_calls" != "driver driver " ]] || grep -q '^GPU_QUIESCE ' "$errors"; then
    selftest_cell_failed "照常跑完应当退 0、有项红退 1，两趟都不清场（照常那一趟退 $not_restored_exit、有项红那一趟退 $exit_code，调用次序「$not_restored_calls」，GPU_QUIESCE 行 $(grep -c '^GPU_QUIESCE ' "$errors") 行）" \
      "看主循环收尾的退出码：清场由 task-server 管，这里不经 gpu-quiesce-run.sh"
  fi
  # 第 29 格：分项驱动排派活计划那一趟借不到显存、一项都没跑到（CRASH_AMPLIFICATION_NOT_RUN 行、退 77、0 行 CRASH_AMPLIFICATION）：
  #   两项里这一项 exit=77 verdict=not-run、不红，逐项报本次未跑（标准错误与 GATE_NOT_RUN_FILE 各一笔，原因带那一行的 reason=），整趟退 0、成功句点名它；
  #   只点名这一项时一项都没判完，整趟退 77
  local not_run_file="$work/gate-not-run" only_not_run_exit
  cells=$((cells + 1)); : > "$not_run_file"
  output="$(GATE_NOT_RUN_FILE="$not_run_file" STUB_SHAPE_crash_enumeration_tree_split_streams=placement-not-run \
    entry --item fixed-script-through-rollback --item tree-split-streams 2>"$errors")"; exit_code=$?
  STUB_SHAPE=placement-not-run entry --item tree-split-streams >/dev/null 2>&1; only_not_run_exit=$?
  if [[ "$exit_code" != 0 || "$only_not_run_exit" != 77 ]] \
     || ! grep -q '^CRASH_AMPLIFICATION_ITEM item=tree-split-streams exit=77 states=0 .* lines=0 .* verdict=not-run ' <<<"$output" \
     || ! grep -q '^CRASH_AMPLIFICATION_ITEM item=fixed-script-through-rollback exit=0 .* verdict=green ' <<<"$output" \
     || ! grep -q 'CRASH_AMPLIFICATION_TOTAL dispatched=2 items=2 failed=0 not_run=1 ' <<<"$output" \
     || ! grep -q '本次未跑（这不是通过）：崩溃放量分项 tree-split-streams：分项驱动退 77（一项都没跑到），报本次未跑（CRASH_AMPLIFICATION_NOT_RUN step=placement）：借不到显存，这一项没判成（等了 1800 秒' "$errors" \
     || ! grep -q '^崩溃放量分项 tree-split-streams：分项驱动退 77（一项都没跑到）' "$not_run_file" \
     || ! grep -q '本次未跑 1 项：tree-split-streams（分项驱动退 77' <<<"$output"; then
    selftest_cell_failed "分项驱动退 77 没记进本次未跑：没逐项报、被判了红、原因里没有那一行的 reason=、成功句没点名，或只点名这一项时没退 77（两项那一趟退 $exit_code、只点名这一项退 $only_not_run_exit）：$output" \
      "看主循环里 item_exit_not_run 与 item_driver_not_run 那几处：驱动退 77 不按「驱动退非 0」判红、记 not-run，原因取 CRASH_AMPLIFICATION_NOT_RUN 行的 step= 与 reason="
  fi
  # 第 30 格：一条用例几条流，有的借不到显存（分项驱动退 0，打一行判完的 CRASH_AMPLIFICATION 与一行 flow=2/3 的 CRASH_AMPLIFICATION_NOT_RUN）：
  #   这一项 verdict=not-run、不红也不算通过，原因写 1 条流没判；两项那一趟退 0、成功句点名它；只点名这一项时没有一项判完，整趟退 77
  cells=$((cells + 1)); : > "$not_run_file"
  output="$(GATE_NOT_RUN_FILE="$not_run_file" STUB_SHAPE_crash_enumeration_tree_split_streams=partial-not-run \
    entry --item fixed-script-through-rollback --item tree-split-streams 2>"$errors")"; exit_code=$?
  STUB_SHAPE=partial-not-run entry --item tree-split-streams >/dev/null 2>&1; only_not_run_exit=$?
  if [[ "$exit_code" != 0 || "$only_not_run_exit" != 77 ]] \
     || ! grep -q '^CRASH_AMPLIFICATION_ITEM item=tree-split-streams exit=0 states=7 .* lines=1 .* verdict=not-run ' <<<"$output" \
     || ! grep -q 'CRASH_AMPLIFICATION_TOTAL dispatched=2 items=2 failed=0 not_run=1 ' <<<"$output" \
     || ! grep -q '本次未跑（这不是通过）：崩溃放量分项 tree-split-streams：分项驱动报 1 条流本次未跑（CRASH_AMPLIFICATION_NOT_RUN）：借不到显存没判，借到的照判了（等了 1800 秒' "$errors" \
     || ! grep -q '^崩溃放量分项 tree-split-streams：分项驱动报 1 条流本次未跑' "$not_run_file" \
     || ! grep -q '本次未跑 1 项：tree-split-streams（分项驱动报 1 条流本次未跑' <<<"$output"; then
    selftest_cell_failed "几条流里有的没借到显存，这一项却记了通过：没记 not-run、没逐项报、成功句没点名，或只点名这一项时没退 77（两项那一趟退 $exit_code、只点名这一项退 $only_not_run_exit）：$output" \
      "看主循环里 item_driver_not_run_lines：驱动退 0 而打了 CRASH_AMPLIFICATION_NOT_RUN 行，这一项记 not-run，原因写几条流没判"
  fi
  # 第 31 格：不带参数（scope=whole），只有 multi-record-publish 的驱动行报 exhaustive=false（flow=stub）：
  #   红在这一项（ITEM 行 exhaustive=false verdict=red，✗ 点名它与不穷举的流、出路写「按段登记（.claude/gate.d/crash-point-replay-segments.tsv）或加深展开」，重跑命令只点名它），
  #   别的每一项 verdict=green、✗ 里不点名；TOTAL 报派出、收回都是登记的项数、failed=1、scope=whole，整趟退 1、不打成功句
  local not_exhaustive_greens
  cells=$((cells + 1)); : > "$log"
  output="$(STUB_SHAPE_crash_enumeration_multi_record_publish_stream=not-exhaustive-flow entry 2>"$errors")"; exit_code=$?
  not_exhaustive_greens="$(grep -cE "^CRASH_AMPLIFICATION_ITEM item=[^ ]+ exit=0 .* expand_up_to=7 exhaustive=true verdict=green " <<<"$output")"
  if [[ "$exit_code" != 1 ]] || [[ "$(wc -l <"$log")" != "$item_count" ]] \
     || ! grep -qE "^CRASH_AMPLIFICATION_ITEM item=multi-record-publish exit=0 .* expand_up_to=7 exhaustive=false verdict=red " <<<"$output" \
     || [[ "$not_exhaustive_greens" != "$((item_count - 1))" ]] \
     || ! grep -qE "^CRASH_AMPLIFICATION_TOTAL dispatched=$item_count items=$item_count failed=1 not_run=0 .* scope=whole " <<<"$output" \
     || ! grep -q "^  ✗ 分项 multi-record-publish：有 1 行驱动行不是 exhaustive=true、也没按段登记（不穷举的流：stub）" "$errors" \
     || [[ "$(grep -c '^  ✗ 分项 ' "$errors")" != 1 ]] \
     || ! grep -q '按段登记（.claude/gate.d/crash-point-replay-segments.tsv）或加深展开' "$errors" \
     || ! grep -qE -- '只重跑判红的那几项：.* --item multi-record-publish$' "$errors" \
     || grep -q '✓' <<<"$output"; then
    selftest_cell_failed "不带参数时一项不穷举，没有红在那一项：那一项没记红、别的项被连带判红、整趟没退 1、✗ 没点名不穷举的流或没给出路（退 $exit_code，绿项 $not_exhaustive_greens 个）：$output" \
      "看主循环里不穷举那一条：有一行不是 exhaustive=true 又没按段登记就记这一项红，理由点名 item_not_exhaustive_flows、出路写按段登记或加深展开"
  fi
  # 第 32 格：不带参数的全量里一项的一行 exhaustive=false registered_segments=3:300 unregistered_segments_beyond_expansion=-（只因登记的段不穷举，flow=stub），
  #   同一项另一条流穷举：这一项不红（ITEM 行照实写 exhaustive=false、verdict=green、registered_segments=stub@3:300、unregistered_segments_beyond_expansion=-），
  #   别的项 registered_segments=- unregistered_segments_beyond_expansion=unknown（假驱动那几种行没报这个字段），整趟退 0、成功句说全量，
  #   末尾报「按段登记 1 段（1 项…）：position-addressed-trees 流 stub 段 3（300 写）」与「展开上限盖不住又没登记的段：0 段（另有 7 项取不出：…）」
  local registered_greens
  cells=$((cells + 1))
  output="$(STUB_SHAPE_crash_enumeration_position_addressed_trees=registered-flow entry 2>"$errors")"; exit_code=$?
  success_line="$(grep '✓' <<<"$output")"
  registered_greens="$(grep -cE "^CRASH_AMPLIFICATION_ITEM item=[^ ]+ exit=0 .* exhaustive=true verdict=green .* registered_segments=- unregistered_segments_beyond_expansion=unknown registered_segment_domains=- registered_segment_progress=-\$" <<<"$output")"
  if [[ "$exit_code" != 0 ]] \
     || ! grep -qE "^CRASH_AMPLIFICATION_ITEM item=position-addressed-trees exit=0 .* expand_up_to=7 exhaustive=false verdict=green .* registered_segments=stub@3:300 unregistered_segments_beyond_expansion=- registered_segment_domains=stub@3:full registered_segment_progress=-\$" <<<"$output" \
     || [[ "$registered_greens" != "$((item_count - 1))" ]] \
     || ! grep -q "崩溃放量全部项" <<<"$success_line" \
     || ! grep -q "；按段登记 1 段（1 项，只按展开深度内的域判、没判全域）：position-addressed-trees 流 stub 段 3（300 写）；登记段的域1 段（1 项；全展开 1 段、分片 0 段）：position-addressed-trees 流 stub 段 3：全展开；展开上限盖不住又没登记的段：0 段（另有 $((item_count - 1)) 项取不出：.*，驱动行没报 unregistered_segments_beyond_expansion=）\$" <<<"$success_line"; then
    selftest_cell_failed "一行 exhaustive=false 只因登记的段（registered_segments=3:300），这一项却判了红、或 ITEM 行与成功句没报段号与写数（退 $exit_code，别的项绿 $registered_greens 个）：$output" \
      "看 summarize_the_item 里 registered_segments 那一段（exhaustive=false 而登了段的行不算不穷举、段按流记下）与成功句的 registered_part"
  fi
  # 第 33 格：点名一项：stub 那条流登了第 3 段（exhaustive=false），other 那条也不穷举而 registered_segments=-：
  #   这一项照旧红在 other（✗ 只点名 other，出路写按段登记或加深展开），ITEM 行照样报 stub 登的段，整趟退 1、不打成功句
  cells=$((cells + 1))
  output="$(STUB_SHAPE_crash_enumeration_position_addressed_trees=registered-and-unregistered entry --item position-addressed-trees 2>"$errors")"; exit_code=$?
  if [[ "$exit_code" != 1 ]] \
     || ! grep -qE "^CRASH_AMPLIFICATION_ITEM item=position-addressed-trees exit=0 .* exhaustive=false verdict=red .* registered_segments=stub@3:300 unregistered_segments_beyond_expansion=other@0:40 registered_segment_domains=stub@3:full registered_segment_progress=-\$" <<<"$output" \
     || ! grep -q "^  ✗ 分项 position-addressed-trees：有 1 行驱动行不是 exhaustive=true、也没按段登记（不穷举的流：other）" "$errors" \
     || ! grep -q '按段登记（.claude/gate.d/crash-point-replay-segments.tsv）或加深展开' "$errors" \
     || grep -q '✓' <<<"$output"; then
    selftest_cell_failed "一行 exhaustive=false registered_segments=- 没照旧判红在那一条流：没记红、把没登段的流也放过了、或 ✗ 点名了登了段的流（退 $exit_code）：$output" \
      "看 summarize_the_item：只有 exhaustive=false 而本行 registered_segments= 是 <段号>:<写数>[,…] 的行才不算不穷举，- 与没报的照算，按行判、不按项判"
  fi
  # 第 34 格：每一趟分项驱动都收到按段登记文件的绝对路径（SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY=<被判仓根>/.claude/gate.d/crash-point-replay-segments.tsv，
  #   假驱动记进调用记录），标准错误点名送了哪一份
  local registry_calls
  cells=$((cells + 1)); : > "$log"
  entry --item fixed-script-through-rollback --item tree-split-streams >/dev/null 2>"$errors"; exit_code=$?
  registry_calls="$(grep -cF " segments=[$segments_registry]" "$log")"
  if [[ "$exit_code" != 0 || "$(wc -l <"$log")" != 2 || "$registry_calls" != 2 ]] \
     || ! grep -qF "往引擎送按段登记文件 SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY=$segments_registry（1 行登记）" "$errors"; then
    selftest_cell_failed "分项驱动没收到按段登记文件的绝对路径（退 $exit_code，2 趟里收到 $registry_calls 趟；调用记录 $(tr '\n' '|' <"$log")）" \
      "看选分项之后「按段登记」那一段：核过之后 export SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY=<被判仓根>/.claude/gate.d/crash-point-replay-segments.tsv"
  fi
  # 第 35 格：按段登记文件写坏一行（摘要写成大写十六进制，lib_crash_point_replay_items.py 的 segment_problems 认的一种坏法）：整趟退 2、不清场、不起驱动，
  #   标准错误逐处列出（那一处的原话）并点名那份文件
  cells=$((cells + 1)); : > "$log"; : > "$QUIESCE_LOG"
  cp -- "$segments_registry" "$work/segments-saved.tsv"
  printf '%s\n' "# 自证的按段登记（写坏的）" "position-addressed-trees	stub	3	300	$(printf '0123456789ABCDEF%.0s' 1 2 3 4)" >"$segments_registry"
  entry --item tree-split-streams >/dev/null 2>"$errors"; exit_code=$?
  cp -- "$work/segments-saved.tsv" "$segments_registry"
  if [[ "$exit_code" != 2 ]] || [[ -s "$log" || -s "$QUIESCE_LOG" ]] || ! grep -q '不是 64 位小写十六进制' "$errors" \
     || ! grep -qF "按段登记文件 $segments_registry 核不过" "$errors"; then
    selftest_cell_failed "按段登记文件写坏了却没整趟拒跑（退 $exit_code，驱动调用 $(wc -l <"$log") 次，清场记录 $(wc -l <"$QUIESCE_LOG") 笔）" \
      "看选分项之后「按段登记」那一段：开跑前调 lib_crash_point_replay_items.py segments <被判仓根>，核不过打它逐处列的问题、退 2，在清场之前"
  fi
  # 第 36 格：stub 那条流登了第 0 段（322 写），驱动行报 unregistered_segments_beyond_expansion=1:290（第 1 段没登、展开上限盖不住）：
  #   点名这一项：红在这一项，✗ 点名流 stub 与 1:290、出路写按段登记或加深展开，ITEM 行报 registered_segments=stub@0:322
  #   unregistered_segments_beyond_expansion=stub@1:290，整趟退 1、不打成功句
  cells=$((cells + 1))
  output="$(STUB_SHAPE_crash_enumeration_position_addressed_trees=registered-beyond entry --item position-addressed-trees 2>"$errors")"; exit_code=$?
  if [[ "$exit_code" != 1 ]] \
     || ! grep -qE "^CRASH_AMPLIFICATION_ITEM item=position-addressed-trees exit=0 .* exhaustive=false verdict=red .* registered_segments=stub@0:322 unregistered_segments_beyond_expansion=stub@1:290 registered_segment_domains=stub@0:full registered_segment_progress=-\$" <<<"$output" \
     || ! grep -q "^  ✗ 分项 position-addressed-trees：有 1 行驱动行登了段，可那条流里另有没登记、展开上限盖不住的段（流 stub 段 1:290）" "$errors" \
     || ! grep -q '按段登记（.claude/gate.d/crash-point-replay-segments.tsv）或加深展开' "$errors" \
     || grep -q '✓' <<<"$output"; then
    selftest_cell_failed "登了段的流里另有展开上限盖不住又没登记的段，没判红在那条流与那几段：没记红、✗ 没点名流与段号写数、没给出路（退 $exit_code）：$output" \
      "看 summarize_the_item 里 unregistered_segments_beyond_expansion 那一段（登了段的行这个字段不是 - 就记下那几段）与主循环里 item_beyond_lines 那一条红因、成功句的 beyond_part"
  fi
  # 第 37 格：旧引擎打的行：stub 那条登了第 3 段、exhaustive=false，没报 unregistered_segments_beyond_expansion=：点名这一项：
  #   照不穷举红在这一项，✗ 点名流 stub，出路写「引擎没报 unregistered_segments_beyond_expansion=，重编引擎」，ITEM 行这个字段写 unknown，整趟退 1
  cells=$((cells + 1))
  output="$(STUB_SHAPE_crash_enumeration_position_addressed_trees=registered-old-engine entry --item position-addressed-trees 2>"$errors")"; exit_code=$?
  if [[ "$exit_code" != 1 ]] \
     || ! grep -qE "^CRASH_AMPLIFICATION_ITEM item=position-addressed-trees exit=0 .* exhaustive=false verdict=red .* registered_segments=stub@3:300 unregistered_segments_beyond_expansion=unknown registered_segment_domains=unknown registered_segment_progress=unknown\$" <<<"$output" \
     || ! grep -q "^  ✗ 分项 position-addressed-trees：有 1 行驱动行登了段、exhaustive=false，却没报 unregistered_segments_beyond_expansion=（流：stub）" "$errors" \
     || ! grep -q '引擎没报 unregistered_segments_beyond_expansion=，重编引擎' "$errors" \
     || ! grep -q '^  → 登了段的行没报 unregistered_segments_beyond_expansion= 的项（position-addressed-trees）：' "$errors" \
     || grep -q '✓' <<<"$output"; then
    selftest_cell_failed "登了段的行没报 unregistered_segments_beyond_expansion=，没照不穷举判红、出路不是重编引擎（退 $exit_code）：$output" \
      "看 summarize_the_item：登了段、exhaustive=false 的行没报这个字段（或写得认不得）记进没报的行数与流名，主循环里 item_beyond_missing_lines 那一条红因与「重编引擎」那一句出路"
  fi
  # 第 38–41 格：登记段装不进 u64 时的分片枚举（文件头「按段登记」），都用假驱动形态 registered-tail，stub 那条流登记段的几个字段由 STUB_REGISTERED_TAIL 给
  # 总态数：3^290-1 的十进制（139 位）与一个 141 位的十进制串，都超过 u64（引擎的态数是 768 位宽整数，驱动行照十进制数字串报）
  local slice_total=2318269218083089908234280468459894595445540070540355238386428052606631732586480323016481331702297098820479338186895528168360391078014797448
  local slice_total_141=100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000515377520732011331036461129765621272702107522001
  local tail_sliced="registered_segments=0:322,1:290 unregistered_segments_beyond_expansion=- registered_segment_domains=0:full,1:slice registered_segment_progress=1:4294967296/$slice_total"
  local tail_forms="registered_segments=0:322,2:300 unregistered_segments_beyond_expansion=- registered_segment_domains=0:slice,2:slice registered_segment_progress=0:7/18446744073709551615,2:5/$slice_total_141"
  local slice_item_line slice_forms_output slice_forms_exit slice_forms_item_line slice_forms_success slice_wrong slice_tail slice_run
  local domain_bad_text='分项 position-addressed-trees：有 1 行驱动行登了段，registered_segment_domains= / registered_segment_progress= 缺了、没列全登记的段（或分片段没报进度）、或写法认不得（流：stub）'
  tail_entry() { # <STUB_REGISTERED_TAIL> <入口参数…>：position-addressed-trees 用 registered-tail 形态起一趟入口
    local tail="$1"; shift
    STUB_SHAPE_crash_enumeration_position_addressed_trees=registered-tail STUB_REGISTERED_TAIL="$tail" entry "$@"
  }
  # 第 38 格：stub 那条流登了第 0 段（322 写，域全展开）与第 1 段（290 写，域分片：这一趟为止已判 4294967296 态、域共 3^290-1 态，照十进制 139 位报），别的段都盖得住：
  #   点名这一项，不红、退 0；ITEM 行带两个字段（registered_segment_domains=stub@0:full,stub@1:slice
  #   registered_segment_progress=stub@1:4294967296/<那 139 位>）；成功句报「段 0：全展开」「段 1：分片，已判 4294967296/<那 139 位>」。
  #   第二趟换总态数的另两种长度（装得进 u64 的 18446744073709551615、141 位的十进制串）：照样不红，ITEM 行与成功句照原样报
  cells=$((cells + 1))
  output="$(tail_entry "$tail_sliced" --item position-addressed-trees 2>"$errors")"; exit_code=$?
  success_line="$(grep '✓' <<<"$output")"
  slice_item_line="$(grep '^CRASH_AMPLIFICATION_ITEM item=position-addressed-trees ' <<<"$output")"
  slice_forms_output="$(tail_entry "$tail_forms" --item position-addressed-trees 2>/dev/null)"; slice_forms_exit=$?
  slice_forms_item_line="$(grep '^CRASH_AMPLIFICATION_ITEM item=position-addressed-trees ' <<<"$slice_forms_output")"
  slice_forms_success="$(grep '✓' <<<"$slice_forms_output")"
  if [[ "$exit_code" != 0 ]] || grep -q '✗' "$errors" \
     || [[ "$slice_item_line" != *" exhaustive=false verdict=green "* ]] \
     || [[ "$slice_item_line" != *" registered_segments=stub@0:322,stub@1:290 unregistered_segments_beyond_expansion=- registered_segment_domains=stub@0:full,stub@1:slice registered_segment_progress=stub@1:4294967296/$slice_total" ]] \
     || [[ "$success_line" != *"崩溃放量点名的 1 项"* ]] \
     || [[ "$success_line" != *"；登记段的域2 段（1 项；全展开 1 段、分片 1 段）：position-addressed-trees 流 stub 段 0：全展开、position-addressed-trees 流 stub 段 1：分片，已判 4294967296/$slice_total；"* ]] \
     || [[ "$slice_forms_exit" != 0 ]] \
     || [[ "$slice_forms_item_line" != *" verdict=green "*" registered_segment_domains=stub@0:slice,stub@2:slice registered_segment_progress=stub@0:7/18446744073709551615,stub@2:5/$slice_total_141" ]] \
     || [[ "$slice_forms_success" != *"：position-addressed-trees 流 stub 段 0：分片，已判 7/18446744073709551615、position-addressed-trees 流 stub 段 2：分片，已判 5/$slice_total_141；"* ]]; then
    selftest_cell_failed "登记段分片枚举（全展开一段、分片一段，总态数是超过 u64 的十进制串）判了红、或 ITEM 行没带 registered_segment_domains= / registered_segment_progress=、或成功句没报全展开与分片已判几态（退 $exit_code，另两种写法那一趟退 $slice_forms_exit）：$output｜$slice_forms_output" \
      "看 summarize_the_item 的 domains_problem_of_the_line（两个字段的写法、进度的两个数只认十进制数字串）与主循环里 domains_of_the_item_for_the_success_line、成功句的 domain_part"
  fi
  # 第 39 格：分片段的进度是 1:0/<那 139 位>，stub 那条这一趟现判 0 态、复用 0 态（一个态都没判，也没复用）：点名这一项，红在这一项，
  #   ✗ 点名流 stub 与段 1，出路写「这一趟一个态都没判，看录入段与判段的日志」，整趟退 1、不打成功句
  cells=$((cells + 1))
  slice_tail="registered_segments=0:322,1:290 unregistered_segments_beyond_expansion=- registered_segment_domains=0:full,1:slice registered_segment_progress=1:0/$slice_total"
  output="$(STUB_TAIL_CHECKED=0 STUB_TAIL_BLOCKS=0 STUB_TAIL_REUSED=0 tail_entry "$slice_tail" --item position-addressed-trees 2>"$errors")"; exit_code=$?
  slice_item_line="$(grep '^CRASH_AMPLIFICATION_ITEM item=position-addressed-trees ' <<<"$output")"
  if [[ "$exit_code" != 1 ]] \
     || [[ "$slice_item_line" != *" verdict=red "*" registered_segment_progress=stub@1:0/$slice_total" ]] \
     || ! grep -qF "分项 position-addressed-trees：有 1 行驱动行的分片段已判 0 态（流 stub 段 1:0/$slice_total）：这一趟一个态都没判，看录入段与判段的日志" "$errors" \
     || ! grep -q '^  → 分片段这一趟已判 0 态的项（position-addressed-trees）：这一趟一个态都没判，看录入段与判段的日志' "$errors" \
     || grep -q '✓' <<<"$output"; then
    selftest_cell_failed "分片段这一趟已判 0 态却没判红在那条流与那一段、或出路不是看录入段与判段的日志（退 $exit_code）：$output" \
      "看 summarize_the_item 里分片进度那一段（已判是 0 的段记进 zero）与主循环里 item_zero_lines 那一条红因（不看展开深度）、slice_zero_failed_names 那一句出路"
  fi
  # 第 40 格：登了段的行，两个字段没报全或写法认不得（四趟：两个都缺，即分片之前的引擎；只缺 registered_segment_progress=；域写成 1:partial；
  #   总态数写成幂的形态 3^290-1，不是十进制数字串）：
  #   每一趟都红在这一项，✗ 点名流 stub，出路写「引擎没报 registered_segment_domains= / registered_segment_progress=，重编引擎」，ITEM 行两个字段写 unknown，整趟退 1
  cells=$((cells + 1)); slice_wrong=""; slice_run=0
  for slice_tail in 'registered_segments=0:322,1:290 unregistered_segments_beyond_expansion=-' \
                    'registered_segments=0:322,1:290 unregistered_segments_beyond_expansion=- registered_segment_domains=0:full,1:slice' \
                    'registered_segments=0:322,1:290 unregistered_segments_beyond_expansion=- registered_segment_domains=0:full,1:partial registered_segment_progress=-' \
                    'registered_segments=0:322,1:290 unregistered_segments_beyond_expansion=- registered_segment_domains=0:full,1:slice registered_segment_progress=1:4294967296/3^290-1'; do
    slice_run=$((slice_run + 1))
    output="$(tail_entry "$slice_tail" --item position-addressed-trees 2>"$errors")"; exit_code=$?
    slice_item_line="$(grep '^CRASH_AMPLIFICATION_ITEM item=position-addressed-trees ' <<<"$output")"
    if [[ "$exit_code" != 1 ]] || [[ "$slice_item_line" != *" verdict=red "*" registered_segment_domains=unknown registered_segment_progress=unknown" ]] \
       || ! grep -qF "$domain_bad_text" "$errors" || ! grep -qF '引擎没报 registered_segment_domains= / registered_segment_progress=，重编引擎' "$errors" \
       || ! grep -q '^  → 登了段的行没报全 registered_segment_domains= / registered_segment_progress= 的项（position-addressed-trees）：' "$errors" \
       || grep -q '✓' <<<"$output"; then
      slice_wrong="${slice_wrong}第 $slice_run 趟退 $exit_code：$output｜"
    fi
  done
  if [[ -n "$slice_wrong" ]]; then
    selftest_cell_failed "登了段的行 registered_segment_domains= / registered_segment_progress= 缺了或写法认不得，没判红在那条流、出路不是重编引擎：$slice_wrong" \
      "看 summarize_the_item 的 domains_problem_of_the_line：两个字段都要报、写法认得（返回 1 的那几种），记进 domain_bad 的行数与流名；主循环里 item_domain_bad_lines 那一条红因与 domain_field_missing_failed_names 那一句出路"
  fi
  # 第 41 格：登了第 0、1 段，registered_segment_domains= 漏了段 1（0:full），或段 1 是分片而 registered_segment_progress= 没列它（-）：
  #   两趟都红在这一项，✗ 与出路同第 40 格，ITEM 行两个字段写 unknown，整趟退 1
  cells=$((cells + 1)); slice_wrong=""; slice_run=0
  for slice_tail in 'registered_segments=0:322,1:290 unregistered_segments_beyond_expansion=- registered_segment_domains=0:full registered_segment_progress=-' \
                    'registered_segments=0:322,1:290 unregistered_segments_beyond_expansion=- registered_segment_domains=0:full,1:slice registered_segment_progress=-'; do
    slice_run=$((slice_run + 1))
    output="$(tail_entry "$slice_tail" --item position-addressed-trees 2>"$errors")"; exit_code=$?
    slice_item_line="$(grep '^CRASH_AMPLIFICATION_ITEM item=position-addressed-trees ' <<<"$output")"
    if [[ "$exit_code" != 1 ]] || [[ "$slice_item_line" != *" verdict=red "*" registered_segment_domains=unknown registered_segment_progress=unknown" ]] \
       || ! grep -qF "$domain_bad_text" "$errors" || ! grep -qF '引擎没报 registered_segment_domains= / registered_segment_progress=，重编引擎' "$errors" \
       || grep -q '✓' <<<"$output"; then
      slice_wrong="${slice_wrong}第 $slice_run 趟退 $exit_code：$output｜"
    fi
  done
  if [[ -n "$slice_wrong" ]]; then
    selftest_cell_failed "registered_segment_domains= 漏了登记的段、或分片段没在 registered_segment_progress= 里，没判红：$slice_wrong" \
      "看 summarize_the_item 的 domains_problem_of_the_line：registered_segments= 列的每一段都要在域里、域里每个分片段都要在进度里（返回 2 的那两种）"
  fi
  # 第 42 格：按登记不判的 ignored_points= / states_skipped_by_ignore= 按流分组、每组只取一次（文件头「按登记整批不判」），点名 tree-split-streams 一项，三趟：
  #   两条流各一份（1/10 与 2/20，a 流两行同报一份）→ 3 个点、30 态；两行都不带 flow=（归一组，各报 2/50）→ 2 个点、50 态。
  #   每趟 ITEM 行与 TOTAL 行钉绝对值，退 0、成功句报同样的数。末一趟同流两行报的不同（a 流 1/10 与 1/11，b 流 2/20）：ITEM 两个数写 unknown、
  #   TOTAL 的 ignore_unknown=1 且不加这一项，标准错误与成功句点名 a 流，退 0（不红）
  local ignore_wrong="" ignore_run=0 ignore_shape ignore_expected_points ignore_expected_states
  cells=$((cells + 1))
  for ignore_shape in 'ignored-two-flows 3 30' 'ignored-no-flow-twice 2 50'; do
    read -r ignore_shape ignore_expected_points ignore_expected_states <<<"$ignore_shape"
    ignore_run=$((ignore_run + 1))
    output="$(STUB_SHAPE="$ignore_shape" entry --item tree-split-streams 2>"$errors")"; exit_code=$?
    success_line="$(grep '✓' <<<"$output")"
    if [[ "$exit_code" != 0 ]] \
       || ! grep -qE " ignored_points=$ignore_expected_points states_skipped_by_ignore=$ignore_expected_states registered_segments=" <<<"$(grep '^CRASH_AMPLIFICATION_ITEM item=tree-split-streams ' <<<"$output")" \
       || ! grep -qE "^CRASH_AMPLIFICATION_TOTAL dispatched=1 items=1 failed=0 .* ignored_points=$ignore_expected_points states_skipped_by_ignore=$ignore_expected_states ignore_unknown=0\$" <<<"$output" \
       || ! grep -q "按登记整批不判：$ignore_expected_points 个点、$ignore_expected_states 态（类名取不出" <<<"$success_line"; then
      ignore_wrong="${ignore_wrong}$ignore_shape 退 $exit_code（要 $ignore_expected_points 个点、$ignore_expected_states 态）：$(grep -E '^CRASH_AMPLIFICATION_(ITEM|TOTAL) ' <<<"$output" | tr '\n' '|')｜"
    fi
  done
  ignore_run=$((ignore_run + 1))
  output="$(STUB_SHAPE=ignored-flow-differs entry --item tree-split-streams 2>"$errors")"; exit_code=$?
  success_line="$(grep '✓' <<<"$output")"
  if [[ "$exit_code" != 0 ]] \
     || ! grep -q ' ignored_points=unknown states_skipped_by_ignore=unknown registered_segments=' <<<"$(grep '^CRASH_AMPLIFICATION_ITEM item=tree-split-streams ' <<<"$output")" \
     || ! grep -qE '^CRASH_AMPLIFICATION_TOTAL dispatched=1 items=1 failed=0 .* ignored_points=0 states_skipped_by_ignore=0 ignore_unknown=1$' <<<"$output" \
     || ! grep -q '分项 tree-split-streams：同一条流各行报的 ignored_points= / states_skipped_by_ignore= 不同（流：a）' "$errors" \
     || ! grep -q '按登记整批不判：取不出（1 项：tree-split-streams（同一条流各行报的数不同，流：a），' <<<"$success_line"; then
    ignore_wrong="${ignore_wrong}ignored-flow-differs 退 $exit_code（要 unknown 并点名 a 流）：$(grep -E '^CRASH_AMPLIFICATION_(ITEM|TOTAL) ' <<<"$output" | tr '\n' '|')｜"
  fi
  if [[ -n "$ignore_wrong" ]]; then
    selftest_cell_failed "按登记不判的点数与态数没按流分组、每组只取一次：同流各行重复计、不同的流没各取一次相加、没带 flow= 的行没归一组，或同流各行报的不同却没记 unknown 并点名那条流（共 $ignore_run 趟）：$ignore_wrong" \
      "看 summarize_the_item 里 ignored_points / states_skipped_by_ignore 那一段：按 flow= 分组（没有的归一组），组里数相同取那一份、不同记 ignore_conflict 并让两个数写 unknown，各组相加；主循环 item_ignore_conflict_flows 那几处点名流"
  fi
  # 第 43 格：全部复用的一趟（文件头「按段登记」）：stub 那条流这一趟现判 0 态、0 块，复用 7 态，分片段的进度是 1:0/<那 139 位>：
  #   分片段已判 0 态只在现判与复用都是 0 时红，这一趟不红：点名这一项，ITEM 行 states=3（只有 other 那条现判）
  #   reused=7 verdict=green，标准错误里没有「分片段已判 0 态」，整趟退 0、打成功句
  cells=$((cells + 1))
  slice_tail="registered_segments=0:322,1:290 unregistered_segments_beyond_expansion=- registered_segment_domains=0:full,1:slice registered_segment_progress=1:0/$slice_total"
  output="$(STUB_TAIL_CHECKED=0 STUB_TAIL_BLOCKS=0 STUB_TAIL_REUSED=7 tail_entry "$slice_tail" --item position-addressed-trees 2>"$errors")"; exit_code=$?
  slice_item_line="$(grep '^CRASH_AMPLIFICATION_ITEM item=position-addressed-trees ' <<<"$output")"
  if [[ "$exit_code" != 0 ]] || [[ "$slice_item_line" != *" states=3 reused=7 red=0 "*" verdict=green "* ]] \
     || grep -q '分片段已判 0 态' "$errors" || ! grep -q '✓' <<<"$output"; then
    selftest_cell_failed "全部复用的一趟（stub 那条现判 0、复用 7，分片段进度 1:0/…）判了红、或 ITEM 行没照实写现判 3 复用 7（退 $exit_code）：$output" \
      "看 summarize_the_item 里 line_idle 那一句：分片段已判 0 态只在这一行 states_checked_here= 与 states_reused= 都不是正整数时记进 zero"
  fi
  # 第 44 格：分项驱动退 0、打了一行 CRASH_AMPLIFICATION_NOT_RUN step=judge（不带 gpu 特性的二进制遇到多机配置 ENABLE_GPU=1：判段这一趟不跑、录入段照录）、
  #   没有 CRASH_AMPLIFICATION 行：两项里这一项 exit=0 lines=0 verdict=not-run、不红（不按「驱动退 0 却 0 行」判红），原因写 1 条流判段没跑与那一行的 reason=，
  #   标准错误与 GATE_NOT_RUN_FILE 各一笔，整趟退 0、成功句写「本次未跑 1 项」；只点名这一项时一项都没判完，整趟退 77
  cells=$((cells + 1)); : > "$not_run_file"
  output="$(GATE_NOT_RUN_FILE="$not_run_file" STUB_SHAPE_crash_enumeration_tree_split_streams=judge-not-run \
    entry --item fixed-script-through-rollback --item tree-split-streams 2>"$errors")"; exit_code=$?
  STUB_SHAPE=judge-not-run entry --item tree-split-streams >/dev/null 2>&1; only_not_run_exit=$?
  if [[ "$exit_code" != 0 || "$only_not_run_exit" != 77 ]] \
     || ! grep -q '^CRASH_AMPLIFICATION_ITEM item=tree-split-streams exit=0 states=0 .* lines=0 .* verdict=not-run ' <<<"$output" \
     || ! grep -q 'CRASH_AMPLIFICATION_TOTAL dispatched=2 items=2 failed=0 not_run=1 ' <<<"$output" \
     || ! grep -q '本次未跑（这不是通过）：崩溃放量分项 tree-split-streams：分项驱动报 1 条流判段没跑（CRASH_AMPLIFICATION_NOT_RUN step=judge）：配置写了 ENABLE_GPU=1 而二进制没编进 gpu 特性，录入段照录、判段这一趟没判（配置写了 ENABLE_GPU=1，这份二进制没有编进 gpu 特性：判段这一趟不跑（假驱动））' "$errors" \
     || ! grep -q '^崩溃放量分项 tree-split-streams：分项驱动报 1 条流判段没跑' "$not_run_file" \
     || ! grep -q '本次未跑 1 项：tree-split-streams（分项驱动报 1 条流判段没跑' <<<"$output"; then
    selftest_cell_failed "分项驱动报判段没跑（CRASH_AMPLIFICATION_NOT_RUN step=judge）的那一项没记本次未跑：判了红、没逐项报、原因没写判段没跑与那一行的 reason=、成功句没点名，或只点名这一项时没退 77（两项那一趟退 $exit_code、只点名这一项退 $only_not_run_exit）：$output" \
      "看主循环里 item_judge_not_run_lines：驱动退 0 而打了 CRASH_AMPLIFICATION_NOT_RUN step=judge 行，这一项记 not-run，原因写几条流判段没跑"
  fi
  # 第 45 格：复用三问（scripts/gate/lib_reuse_checks.py 文件头「一」）：假驱动 library 形态照 $STUB_UNITS/<测试目标> 的内容摘要按键记判过没有，
  #   点名三项各跑四趟：头一趟、同一份输入、加一份哪个测试目标都不读的单元文件（闭包外）、改 fixed-script-through-rollback 那个测试目标的单元文件（闭包内）；
  #   每一趟从 ITEM 行取这一项复用还是现判（states=0 而 reused 是正整数是复用），照三问判、打规范输出。入口自己不判复用（判定在引擎里）：
  #   这一格核的是它每一趟都不清掉服务里判过的（入口不碰判定库）、不因闭包外的改动重判，并照实把驱动报的现判与复用写进 ITEM 行
  cells=$((cells + 1))
  local three_units="$work/stub-units" three_name three_items=(fixed-script-through-rollback multi-record-publish tree-split-streams) three_arguments=()
  mkdir -p "$three_units"
  for entry_row in "${ITEMS[@]}"; do printf 'unit of %s\n' "$(item_fields "$entry_row" 2)" >"$three_units/$(item_fields "$entry_row" 2)"; done
  for three_name in "${three_items[@]}"; do three_arguments+=(--item "$three_name"); done
  three_round() { # <名>：跑一趟，ITEM 行折成「分项<TAB>复用|现判」写进 $work/three-<名>.tsv
    STUB_SHAPE=library STUB_UNITS="$three_units" STUB_SERVICE="$work/stub-service" entry "${three_arguments[@]}" >"$work/three-$1.out" 2>/dev/null
    awk '/^CRASH_AMPLIFICATION_ITEM / { item = ""; states = ""; reused = ""
           for (i = 2; i <= NF; i++) { split($i, pair, "="); if (pair[1] == "item") item = pair[2]; if (pair[1] == "states") states = pair[2]; if (pair[1] == "reused") reused = pair[2] }
           print item "\t" ((states == "0" && reused ~ /^[1-9][0-9]*$/) ? "复用" : "现判") }' "$work/three-$1.out" >"$work/three-$1.tsv"
  }
  three_round first; three_round second
  printf '哪个测试目标都不读\n' >"$three_units/not-read-by-any-target"; three_round unrelated
  printf 'unit of crash_amplification_fixed_script_through_rollback，改过\n' >"$three_units/crash_amplification_fixed_script_through_rollback"; three_round unit
  if ! python3 "$repository_root/scripts/gate/lib_reuse_checks.py" three-questions --script scripts/crash/crash-amplification.sh --first "$work/three-first.tsv" \
         --second "$work/three-second.tsv" --unrelated "$work/three-unrelated.tsv" --unit-change "$work/three-unit.tsv" --affected fixed-script-through-rollback >"$work/three.log" 2>&1; then
    selftest_cell_failed "复用三问没过：$(tr '\n' ' ' <"$work/three.log")" "照规范输出那一行 red 的那一问查：ITEM 行的 states=、reused= 取法（文件头「复用数怎么取」），与入口有没有清掉驱动判过的东西"
  fi
  grep '^REUSE_SELFTEST ' "$work/three.log"
  # 第 46 格：上次红的清单（scripts/gate/lib_reuse_checks.py 文件头「三」）：点名 multi-record-publish 红一趟，清单是它；再点名 fixed-script-through-unmount 绿一趟，清单照留它；
  #   再点名 multi-record-publish 绿一趟，清单成空文件（这两项都没登段，按段登记那几个开关碰不到它们）
  cells=$((cells + 1))
  local last_run_file="$work/last-runs/scripts__crash__crash-amplification.sh.tsv" last_red last_kept last_cleared
  rm -f -- "$last_run_file"   # 前面几格留下的清单不算：这一格从空起
  CRASH_AMPLIFICATION_SELFTEST_RED_ITEM=1 entry --item multi-record-publish >/dev/null 2>&1; last_red="$(cat "$last_run_file" 2>/dev/null)"
  entry --item fixed-script-through-unmount >/dev/null 2>&1; last_kept="$(cat "$last_run_file" 2>/dev/null)"
  entry --item multi-record-publish >/dev/null 2>&1; last_cleared="$(cat "$last_run_file" 2>/dev/null)"
  if [[ "$last_red" != multi-record-publish || "$last_kept" != multi-record-publish || -n "$last_cleared" || ! -f "$last_run_file" ]]; then
    selftest_cell_failed "上次红的清单不对：红一趟之后「$last_red」、点名别的项绿一趟之后「$last_kept」、那一项绿了之后「$last_cleared」" \
      "看主循环收尾写 last-run 那一段：红的分项一行一个，点名了分项的只换跑到的那几项（scripts/gate/lib_reuse_checks.py last-run --ran）"
  fi
  # 第 47 格：每一项的输出出两份日志（scripts/hosts/two-logs.sh）：全量 <分项>.log，旁边同名的 <分项>.problems.log 带行号收「本次未跑」那一行、末行写收了几行全量几行；
  #   输出点名这份日志的路径。假驱动退 77 并在 stderr 打「! 本次未跑」那一行（placement-not-run 形态）
  cells=$((cells + 1))
  local two_logs_output two_logs_directory two_logs_problems two_logs_last_line
  two_logs_output="$(STUB_SHAPE=placement-not-run entry --item tree-split-streams 2>&1)"
  two_logs_directory="$(ls -td "$work/home/runs"/entry-* 2>/dev/null | head -n 1)"
  two_logs_problems="$two_logs_directory/tree-split-streams.problems.log"
  two_logs_last_line="$(tail -n 1 "$two_logs_problems" 2>/dev/null)"   # 管道收尾的 grep -q 在 pipefail 下会把命中读成没命中：先落到变量
  if [[ ! -f "$two_logs_directory/tree-split-streams.log" || ! -f "$two_logs_problems" ]] || ! grep -qE '^[0-9]+:  ! 本次未跑（这不是通过）' "$two_logs_problems" \
     || ! grep -qE '收了 [0-9]+ 行，全量日志共 [0-9]+ 行，被包的命令退出码 77' <<<"$two_logs_last_line" || ! grep -qF "错误警告 $two_logs_problems" <<<"$two_logs_output"; then
    selftest_cell_failed "每一项的输出应当出两份日志（tree-split-streams.log 全量；旁边的 tree-split-streams.problems.log 带行号收「本次未跑」那一行、末行写收了几行全量几行、退出码 77），输出里点名错误警告日志的路径 $two_logs_problems；实际目录里有「$(ls "$two_logs_directory" 2>/dev/null | tr '\n' ' ')」" \
      "看主循环里起分项驱动那一段：经 scripts/hosts/two-logs.sh 起，名取 \${item_output_file%.log}"
  fi
  # 第 48 格：小仓（按段登记的仓根）的 验证包依赖 rocksdb、进程环境里四个取值是空的 ⇒ 清场与起任何一项之前 rocksdb-prebuilt.py entry 判红，退 78，
  #   不清场、分项驱动一次都没起（别的格小仓里没有这份清单，entry 报不判）
  cells=$((cells + 1)); : > "$log"; : > "$QUIESCE_LOG"
  mkdir -p "$segments_root/crates/singlefs-verification"
  printf '%s\n' '[package]' 'name = "singlefs-verification"' 'version = "0.0.0"' '[dependencies]' 'rocksdb = { version = "=0.25.0", optional = true }' \
    >"$segments_root/crates/singlefs-verification/Cargo.toml"
  ROCKSDB_LIB_DIR= ROCKSDB_STATIC= SNAPPY_LIB_DIR= SNAPPY_STATIC= entry >/dev/null 2>"$errors"; exit_code=$?
  if [[ "$exit_code" != 78 ]] || [[ -s "$log" || -s "$QUIESCE_LOG" ]] || ! grep -qF "rocksdb-prebuilt entry：" "$errors" || ! grep -qF "这一趟不起 cargo" "$errors"; then
    selftest_cell_failed "验证包依赖 rocksdb 而进程环境里没有预编库的取值，应当在清场与起任何一项之前退 78、分项驱动一次都没起（退 $exit_code；驱动记录 $(wc -l <"$log") 行）" \
      "看按段登记之后 rocksdb-prebuilt.py entry 那一段"
  fi
  rm -rf -- "${segments_root:?}/crates"
  # 第 49–51 格：跑完运行目录里不剩它建的编译目录（文件头「编译目录」）：调用方没设 CARGO_TARGET_DIR，点名一项；假驱动记下收到的编译目录、往里写一份产物。
  #   第 49 格正常退 0、第 50 格驱动退 1（这一项红、整趟退 1）、第 51 格被 TERM 停（驱动睡 3 秒时只给入口发 TERM，入口等驱动退出后走 EXIT，退 143）：
  #   收到的编译目录在 $SINGLEFS_CRASH_AMPLIFICATION_HOME/runs/entry-*/target，跑完不在了
  # 逐格点名：被 TERM 停的那一格只给自己起的入口那一趟发一次，三格逐条调用、不在循环里发信号
  selftest_target_case() { # <格号:要的退出码>
    local target_case="$1" target_seen target_entry target_attempt
    cells=$((cells + 1)); : > "$log"; : > "$work/targets.log"
    if [[ "${target_case%%:*}" == 53 ]]; then
      env -u CARGO_TARGET_DIR STUB_TARGET_LOG="$work/targets.log" STUB_SLEEP=3 CRASH_AMPLIFICATION_ITEM_DRIVER="$stub" STUB_LOG="$log" CRASH_AMPLIFICATION_SELFTEST_SEGMENTS_ROOT="$segments_root" \
        bash "$0" --item tree-split-streams >/dev/null 2>"$errors" &
      target_entry=$!
      for (( target_attempt = 0; target_attempt < 600; target_attempt++ )); do [[ -s "$work/targets.log" ]] && break; sleep 0.1; done
      # TERM 发给建了运行目录的那一趟（目录名 entry-<时刻>-<进程号> 里的进程号；入口不清场，就是上面起的这一趟）
      target_seen="$(head -n 1 "$work/targets.log" 2>/dev/null)"; target_seen="${target_seen%/target}"
      [[ "${target_seen##*-}" =~ ^[0-9]+$ ]] && kill -TERM "${target_seen##*-}" 2>/dev/null
      if wait "$target_entry"; then exit_code=0; else exit_code=$?; fi
    elif [[ "${target_case%%:*}" == 52 ]]; then
      env -u CARGO_TARGET_DIR STUB_TARGET_LOG="$work/targets.log" CRASH_AMPLIFICATION_SELFTEST_RED_ITEM=1 CRASH_AMPLIFICATION_ITEM_DRIVER="$stub" STUB_LOG="$log" \
        CRASH_AMPLIFICATION_SELFTEST_SEGMENTS_ROOT="$segments_root" bash "$0" --item tree-split-streams >/dev/null 2>"$errors"; exit_code=$?
    else
      env -u CARGO_TARGET_DIR STUB_TARGET_LOG="$work/targets.log" CRASH_AMPLIFICATION_ITEM_DRIVER="$stub" STUB_LOG="$log" CRASH_AMPLIFICATION_SELFTEST_SEGMENTS_ROOT="$segments_root" \
        bash "$0" --item tree-split-streams >/dev/null 2>"$errors"; exit_code=$?
    fi
    target_seen="$(head -n 1 "$work/targets.log" 2>/dev/null)"
    if [[ "$exit_code" != "${target_case#*:}" || "$target_seen" != "$SINGLEFS_CRASH_AMPLIFICATION_HOME"/runs/entry-*/target || -e "$target_seen" ]]; then
      selftest_cell_failed "跑完运行目录里不剩它建的编译目录（$([[ "${target_case%%:*}" == 51 ]] && echo 正常退 0 || { [[ "${target_case%%:*}" == 52 ]] && echo 驱动退 1 || echo 被 TERM 停; })）：驱动收到「${target_seen:-没收到}」，退 $exit_code（要 ${target_case#*:}），跑完$([[ -n "$target_seen" && -e "$target_seen" ]] && echo 还在 || echo 不在)；stderr 尾部：$(tail -n 2 "$errors" | tr '\n' ' ')" \
        "看主循环之前 run_build_directory_begin 与 crash_amplification_entry_finish（EXIT 上 run_build_directory_end），TERM / INT 的 trap 退 143 / 130、等手里那一步退出再走 EXIT"
    fi
  }
  selftest_target_case 51:0
  selftest_target_case 52:1
  selftest_target_case 53:143
  # 第 52 格：调用方设了 CARGO_TARGET_DIR：驱动照收到它、跑完不删
  cells=$((cells + 1)); : > "$log"; : > "$work/targets.log"; mkdir -p "$work/caller-target"
  CARGO_TARGET_DIR="$work/caller-target" STUB_TARGET_LOG="$work/targets.log" entry --item tree-split-streams >/dev/null 2>"$errors"; exit_code=$?
  if [[ "$exit_code" != 0 || "$(head -n 1 "$work/targets.log" 2>/dev/null)" != "$work/caller-target" || ! -f "$work/caller-target/release/artifact" ]]; then
    selftest_cell_failed "调用方设了 CARGO_TARGET_DIR 时驱动应当照收到它、跑完不删（退 $exit_code，驱动收到「$(head -n 1 "$work/targets.log" 2>/dev/null)」）" \
      "看 run_build_directory_begin：设了 CARGO_TARGET_DIR 就不建、不删"
  fi
  # 第 53 格：自证只在临时 prune-server 上跑（文件头「用法」--selftest）：摘掉临时服务的标记再起一份自证 ⇒ 退 2、说自证没跑，一格都不跑
  cells=$((cells + 1))
  env -u PRUNE_SERVER_TEMPORARY_SERVICE_SOCKET bash "$0" --selftest >"$work/bare-selftest.out" 2>&1; exit_code=$?
  if [[ "$exit_code" != 2 ]] || ! grep -q "crash-amplification.sh 自证没跑：" "$work/bare-selftest.out" || grep -q "✗ 自证第" "$work/bare-selftest.out"; then
    selftest_cell_failed "不在临时 prune-server 底下起的自证应当退 2、说自证没跑、一格都不跑（退 $exit_code，输出 $(head -n 2 "$work/bare-selftest.out" | tr "\n" "|")）" \
      "看 run_selftest 开头调共用库 crash_amplification_selftest_service_problem 那一段"
  fi
  # 第 54 格：分项驱动退 77、打的 CRASH_AMPLIFICATION_NOT_RUN 是 step=judge … reason=跑的过程中源码变了（不是排计划那一趟借不到显存）：
  #   两项里这一项 exit=77 verdict=not-run，原因照那一行的 step= 与 reason= 写「…step=judge）：这一项没判成（跑的过程中源码变了…」，整趟标准错误里不出现借不到显存
  cells=$((cells + 1)); : > "$not_run_file"
  output="$(GATE_NOT_RUN_FILE="$not_run_file" STUB_SHAPE_crash_enumeration_tree_split_streams=source-changed-not-run \
    entry --item fixed-script-through-rollback --item tree-split-streams 2>"$errors")"; exit_code=$?
  if [[ "$exit_code" != 0 ]] \
     || ! grep -q '^CRASH_AMPLIFICATION_ITEM item=tree-split-streams exit=77 states=0 .* verdict=not-run ' <<<"$output" \
     || ! grep -q '本次未跑（这不是通过）：崩溃放量分项 tree-split-streams：分项驱动退 77（一项都没跑到），报本次未跑（CRASH_AMPLIFICATION_NOT_RUN step=judge）：这一项没判成（跑的过程中源码变了' "$errors" \
     || grep -q '借不到显存' "$errors"; then
    selftest_cell_failed "分项驱动退 77 而 NOT_RUN 行写的是跑的过程中源码变了，原因不该写借不到显存：这一项没记本次未跑、原因没照 step= 与 reason= 写，或标准错误里写了借不到显存（退 $exit_code）：$output" \
      "看主循环里 item_driver_not_run_cause：只有 step=placement 才写借不到显存，别的 step 照那一行的 reason= 写"
  fi
  # 第 55–57 格：经 task-server 按项分片（文件头「经 task-server 按项分片」）：计划一项一片、不起驱动；一片只跑它那一项、存下输出与退出码、判红也退 0；
  #   合并照存下的输出重放、不起驱动，ITEM 行与退出码同不分片
  local shard_root="$work/shards" shard_target shard_driver_lines
  rm -rf -- "${shard_root:?}"; mkdir -p -- "$shard_root"
  shard_target="$(for row in "${ITEMS[@]}"; do [[ "$(item_fields "$row" 1)" == tree-split-streams ]] && item_fields "$row" 2; done)"
  cells=$((cells + 1)); : > "$log"
  output="$(TASK_SERVER_SHARD_PHASE=plan TASK_SERVER_SHARD_PLAN_DIRECTORY="$shard_root/plan" entry --item fixed-script-through-rollback --item tree-split-streams 2>"$errors")"; exit_code=$?
  if [[ "$exit_code" != 0 || -s "$log" || "$(grep -c . "$shard_root/plan/rows.tsv" 2>/dev/null)" != 2 ]] \
     || ! python3 -c 'import json, sys; plan = json.load(open(sys.argv[1])); sys.exit(0 if [s["label"] for s in plan["shards"]] == ["fixed-script-through-rollback", "tree-split-streams"] else 1)' "$shard_root/plan/plan.json" 2>/dev/null; then
    selftest_cell_failed "按项分片的计划那一步应当一项一片写 plan.json 与 rows.tsv、退 0、不起驱动（退 $exit_code，驱动被起 $(grep -c '^driver ' "$log") 次）：$output $(tail -n 2 "$errors" | tr '\n' ' ')" \
      "看主流程「经 task-server 按项分片」那一段的 plan 分支"
  fi
  cells=$((cells + 1)); : > "$log"
  output="$(CRASH_AMPLIFICATION_SELFTEST_RED_ITEM=1 TASK_SERVER_SHARD_PHASE=shard TASK_SERVER_SHARD=1/2 TASK_SERVER_SHARD_PLAN_DIRECTORY="$shard_root/plan" TASK_SERVER_SHARD_OUTPUT_DIRECTORY="$shard_root/output-1" \
    entry --item fixed-script-through-rollback --item tree-split-streams 2>"$errors")"; exit_code=$?
  shard_driver_lines="$(grep '^driver ' "$log")"
  TASK_SERVER_SHARD_PHASE=shard TASK_SERVER_SHARD=0/2 TASK_SERVER_SHARD_PLAN_DIRECTORY="$shard_root/plan" TASK_SERVER_SHARD_OUTPUT_DIRECTORY="$shard_root/output-0" \
    entry --item fixed-script-through-rollback --item tree-split-streams >/dev/null 2>&1
  if [[ "$exit_code" != 0 || "$(grep -c . <<<"$shard_driver_lines")" != 1 ]] || ! grep -q "^driver ${shard_target}-red " <<<"$shard_driver_lines" \
     || [[ "$(cat "$shard_root/output-1/item.exit" 2>/dev/null)" != 1 ]] || ! grep -q '^CRASH_AMPLIFICATION ' "$shard_root/output-1/item.log" 2>/dev/null \
     || [[ "$(cat "$shard_root/output-0/item.exit" 2>/dev/null)" != 0 ]]; then
    selftest_cell_failed "按项分片的一片应当只跑它那一项、把输出与驱动退出码存进产物目录、这一项判红也退 0（退 $exit_code，驱动被起 $(grep -c . <<<"$shard_driver_lines") 次，item.exit=$(cat "$shard_root/output-1/item.exit" 2>/dev/null)）：$(tail -n 2 "$errors" | tr '\n' ' ')" \
      "看主流程 shard 分支、主循环里存产物那一处与 crash_amplification_entry_finish 里改退出码那一处"
  fi
  cells=$((cells + 1)); : > "$log"
  printf '0\t127.0.0.1\t%s\n1\t127.0.0.1\t%s\n' "$shard_root/output-0" "$shard_root/output-1" > "$shard_root/outputs.tsv"
  output="$(TASK_SERVER_SHARD_PHASE=merge TASK_SERVER_SHARD_PLAN_DIRECTORY="$shard_root/plan" TASK_SERVER_SHARD_OUTPUTS="$shard_root/outputs.tsv" \
    entry --item fixed-script-through-rollback --item tree-split-streams 2>"$errors")"; exit_code=$?
  if [[ "$exit_code" != 1 || -s "$log" ]] || ! grep -q '^CRASH_AMPLIFICATION_ITEM item=fixed-script-through-rollback exit=0 .* verdict=green ' <<<"$output" \
     || ! grep -q '^CRASH_AMPLIFICATION_ITEM item=tree-split-streams exit=1 .* verdict=red ' <<<"$output"; then
    selftest_cell_failed "按项分片的合并应当照各片存下的输出重放、不起驱动，两项的 ITEM 行与整趟退 1 同不分片（退 $exit_code，驱动被起 $(grep -c '^driver ' "$log") 次）：$output" \
      "看主循环里 shard_phase == merge 时的 item_command 与主流程 merge 分支读产物表那一段"
  fi
  # 第 58 格：带 --no-shard 不分：计划写一片、不起驱动；那一片两项都跑、判红照退 1；合并不起驱动、退 0
  cells=$((cells + 1)); : > "$log"
  rm -rf -- "${shard_root:?}"; mkdir -p -- "$shard_root"
  local no_shard_wrong=""
  output="$(TASK_SERVER_SHARD_PHASE=plan TASK_SERVER_SHARD_PLAN_DIRECTORY="$shard_root/plan" entry --no-shard --item fixed-script-through-rollback --item tree-split-streams 2>"$errors")"; exit_code=$?
  if [[ "$exit_code" != 0 || -s "$log" ]] || ! python3 -c 'import json, sys; sys.exit(0 if len(json.load(open(sys.argv[1]))["shards"]) == 1 else 1)' "$shard_root/plan/plan.json" 2>/dev/null; then
    no_shard_wrong+="计划退 $exit_code、驱动被起 $(grep -c '^driver ' "$log") 次；"
  fi
  : > "$log"
  output="$(CRASH_AMPLIFICATION_SELFTEST_RED_ITEM=1 TASK_SERVER_SHARD_PHASE=shard TASK_SERVER_SHARD=0/1 TASK_SERVER_SHARD_PLAN_DIRECTORY="$shard_root/plan" TASK_SERVER_SHARD_OUTPUT_DIRECTORY="$shard_root/output-0" \
    entry --no-shard --item fixed-script-through-rollback --item tree-split-streams 2>"$errors")"; exit_code=$?
  shard_driver_lines="$(grep -c '^driver ' "$log")"
  if [[ "$exit_code" != 1 || "$shard_driver_lines" != 2 || ! -d "$shard_root/output-0" ]]; then
    no_shard_wrong+="那一片退 $exit_code、驱动被起 $shard_driver_lines 次；"
  fi
  : > "$log"
  printf '0\t127.0.0.1\t%s\n' "$shard_root/output-0" > "$shard_root/outputs.tsv"
  output="$(TASK_SERVER_SHARD_PHASE=merge TASK_SERVER_SHARD_PLAN_DIRECTORY="$shard_root/plan" TASK_SERVER_SHARD_OUTPUTS="$shard_root/outputs.tsv" \
    entry --no-shard --item fixed-script-through-rollback --item tree-split-streams 2>"$errors")"; exit_code=$?
  [[ "$exit_code" == 0 && ! -s "$log" ]] || no_shard_wrong+="合并退 $exit_code；"
  [[ -z "$no_shard_wrong" ]] || selftest_cell_failed "带 --no-shard 应当计划写一片、那一片整趟跑（判红照退 1）、合并什么都不做（$no_shard_wrong）：$output $(tail -n 2 "$errors" | tr '\n' ' ')" \
    "看主流程「带 --no-shard 不分」那一段"
  # 第 59 格：缩域（文件头「展开深度」）：给了 --crash-expand-up-to 6 ⇒ 每一项的分项驱动都收到 SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=6，TOTAL 行接 expand_up_to=6，
  #   成功句写「缩过域、不算门禁通过」与 6、句首不写「全部项」；只在环境里设了 4 ⇒ 当作给了 4；两样都给取参数（6）；都不给 ⇒ 驱动收到的是 unset、TOTAL 行不接、成功句写全部项
  cells=$((cells + 1))
  local expand_wrong="" expand_case expand_want expand_text
  for expand_case in "argument:6:" "environment:4:4" "both:6:4" "none:unset:"; do
    : > "$log"; : > "$work/expand.log"
    case "${expand_case%%:*}" in
      argument) output="$(STUB_EXPAND_LOG="$work/expand.log" entry --item tree-split-streams --item fixed-script-through-rollback --crash-expand-up-to 6 2>"$errors")"; exit_code=$? ;;
      environment) output="$(STUB_EXPAND_LOG="$work/expand.log" SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=4 entry --item tree-split-streams --item fixed-script-through-rollback 2>"$errors")"; exit_code=$? ;;
      both) output="$(STUB_EXPAND_LOG="$work/expand.log" SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=4 entry --item tree-split-streams --item fixed-script-through-rollback --crash-expand-up-to 6 2>"$errors")"; exit_code=$? ;;
      none) output="$(STUB_EXPAND_LOG="$work/expand.log" entry --item tree-split-streams --item fixed-script-through-rollback 2>"$errors")"; exit_code=$? ;;
    esac
    expand_want="${expand_case#*:}"; expand_want="${expand_want%%:*}"
    expand_text="$(tr '\n' ' ' <"$work/expand.log")"
    if [[ "$exit_code" != 0 || "$expand_text" != "$expand_want $expand_want " ]]; then
      expand_wrong+="${expand_case%%:*}：退 $exit_code、两个驱动收到的缩域变量「$expand_text」（应当是「$expand_want $expand_want 」）；"
    elif [[ "$expand_want" == unset ]]; then
      if grep -q ' expand_up_to=[0-9]*$' <<<"$(grep '^CRASH_AMPLIFICATION_TOTAL ' <<<"$output")" || ! grep -q '✓ 崩溃放量点名的 2 项' <<<"$output" || grep -q '缩过域' <<<"$output"; then
        expand_wrong+="none：没缩域，TOTAL 行不该接 expand_up_to=、成功句不该写缩过域；"
      fi
    elif ! grep -q "^CRASH_AMPLIFICATION_TOTAL .* expand_up_to=$expand_want\$" <<<"$output" \
         || ! grep -qF "崩溃放量点名的 2 项（缩过域、不算门禁通过：展开深度上限缩到 $expand_want，只往小缩）" <<<"$output"; then
      expand_wrong+="${expand_case%%:*}：TOTAL 行没有 expand_up_to=$expand_want、或成功句没有「缩过域、不算门禁通过：展开深度上限缩到 $expand_want」；"
    fi
  done
  : > "$log"; : > "$work/expand.log"
  output="$(STUB_EXPAND_LOG="$work/expand.log" entry --crash-expand-up-to 6 2>"$errors")"; exit_code=$?
  if [[ "$exit_code" != 0 ]] || ! grep -qF "✓ 崩溃放量（缩过域、不算门禁通过：展开深度上限缩到 6，只往小缩）" <<<"$output" || grep -q '崩溃放量全部项' <<<"$output"; then
    expand_wrong+="不点名分项、缩了域：退 $exit_code、成功句不该写全部项、该写缩过域；"
  fi
  [[ -z "$expand_wrong" ]] || selftest_cell_failed "给了 --crash-expand-up-to 6 应当把 SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=6 交给每一项的分项驱动、TOTAL 行接 expand_up_to=6、成功句写缩过域、不算门禁通过（环境里设了的当作给了同一个值，两样都给取参数，都不给什么都不交）：$expand_wrong" \
    "看参数解析之后缩域那一段（核值、export 变量）与收尾的 TOTAL 行、成功句"
  # 第 60 格：缩域的值写坏退 2、一项都不跑：参数写 abc、-1、空串，参数后面没有值，环境里的变量写 abc 与空串
  cells=$((cells + 1)); : > "$log"
  local expand_bad_wrong="" expand_bad_case
  for expand_bad_case in "argument:abc" "argument:-1" "argument:" "bare:" "environment:abc" "environment:"; do
    case "${expand_bad_case%%:*}" in
      argument) entry --item tree-split-streams --crash-expand-up-to "${expand_bad_case#*:}" >/dev/null 2>"$errors"; exit_code=$? ;;
      bare) entry --item tree-split-streams --crash-expand-up-to >/dev/null 2>"$errors"; exit_code=$? ;;
      environment) SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO="${expand_bad_case#*:}" entry --item tree-split-streams >/dev/null 2>"$errors"; exit_code=$? ;;
    esac
    if [[ "$exit_code" != 2 ]] || [[ -s "$log" ]]; then expand_bad_wrong+="「$expand_bad_case」退 $exit_code、驱动调了 $(grep -c . "$log") 次；"; fi
  done
  [[ -z "$expand_bad_wrong" ]] || selftest_cell_failed "--crash-expand-up-to 的值（或环境里缩域变量的值）不是非负整数时应当退 2、一项都不跑：$expand_bad_wrong" \
    "看参数解析之后缩域那一段：值要匹配 ^[0-9]+\$，不是就交 usage（退 2）"
  # 第 61–66 格：指定范围的崩溃域上限（文件头 usage 一段「指定范围的崩溃域上限」；估法与记录来源在 lib_crash_point_replay_items.py 文件头「指定范围的崩溃域上限」）。
  #   点名 tree-split-streams。运行记录放这几格自己的运行目录 $scoped_home/runs/entry-scoped-*/<分项>.log（每格先清空，不混进前面各格假驱动留下的记录）；
  #   引擎展开上限常量（读它认哪份记录是全域）写进小仓；假 cargo 的读库程序 item 子命令按 STUB_ITEM_RECORDED、STUB_ITEM_TO_JUDGE、STUB_ITEM_REUSABLE 答块账
  local scoped_home="$work/scoped-home" scoped_wrong
  scoped_record() { # <目录名> <分项> <展开深度> <态数> <几天前>：放一份放量分项的运行记录（一行 CRASH_AMPLIFICATION，修改时刻照几天前定，新旧次序靠它）
    mkdir -p -- "$scoped_home/runs/$1"
    printf 'CRASH_AMPLIFICATION flow=%s mode=single_host_cpu states_checked_here=0 blocks_checked_here=0 unjudged_blocks=0 red=0 states_reused=%s exhaustive=true expand_up_to=%s\n' \
      "$2" "$4" "$3" >"$scoped_home/runs/$1/$2.log"
    touch -d "$5 days ago" -- "$scoped_home/runs/$1/$2.log"
  }
  scoped_entry() { SINGLEFS_CRASH_AMPLIFICATION_HOME="$scoped_home" entry "$@"; }
  mkdir -p "$segments_root/crates/singlefs-verification/src"
  printf '%s\n' 'pub const EXPAND_UP_TO_WRITES_PER_SEGMENT_OF_THE_AMPLIFICATION: usize = 30;' >"$segments_root/crates/singlefs-verification/src/crash_amplification.rs"
  # 第 61 格：上一趟不缩域真跑 300000 态（展开深度 30，较新）、更早缩到 6 的那一趟 225 态，库里 4 块键都对不上 ⇒ 估 300000 态，超上限退 79，
  #   说明写能压到上限以内的 n（6）与两条出路，一项驱动都没起
  cells=$((cells + 1)); : > "$log"; scoped_wrong=""; rm -rf -- "${scoped_home:?}"
  scoped_record entry-scoped-old tree-split-streams 6 225 2
  scoped_record entry-scoped-new tree-split-streams 30 300000 1
  STUB_ITEM_RECORDED=4 STUB_ITEM_TO_JUDGE=4 STUB_ITEM_REUSABLE=0 scoped_entry --item tree-split-streams >/dev/null 2>"$errors"; exit_code=$?
  [[ "$exit_code" == 79 ]] || scoped_wrong+="估 300000 态超上限应当退 79（退 $exit_code）；"
  grep -qF "300000" "$errors" || scoped_wrong+="说明里应当写估的态数；"
  grep -qF -- "--crash-expand-up-to 6" "$errors" || scoped_wrong+="说明里应当写能压到上限以内的 --crash-expand-up-to 6；"
  grep -qF "留给提交时不指定范围的那一趟" "$errors" || scoped_wrong+="说明里应当写第二条出路（留给提交时不指定范围的那一趟）；"
  [[ ! -s "$log" ]] || scoped_wrong+="拒了之后不该起分项驱动（调了 $(grep -c . "$log") 次）；"
  [[ -z "$scoped_wrong" ]] || selftest_cell_failed "带 --item、没缩域，估数超过 200000 态应当退 79、说明里写 --crash-expand-up-to 6 与留给提交时那一趟、一项驱动都不起：$scoped_wrong" \
    "看 scoped_domain_guard 与 lib_crash_point_replay_items.py 的 run_scoped_domain、refusal_text"
  # 第 62 格：同样的记录，库里 4 块键都对得上 ⇒ 现判 0 态，放行，驱动照起
  cells=$((cells + 1)); : > "$log"; scoped_wrong=""
  STUB_ITEM_RECORDED=4 STUB_ITEM_TO_JUDGE=0 STUB_ITEM_REUSABLE=4 scoped_entry --item tree-split-streams >"$work/scoped.out" 2>"$errors"; exit_code=$?
  [[ "$exit_code" == 0 ]] || scoped_wrong+="点名项全部可复用应当放行（退 $exit_code）；"
  grep -qF "全部复用，现判 0 态" "$work/scoped.out" || scoped_wrong+="应当写出这一项全部复用、现判 0 态；"
  [[ "$(grep -c . "$log")" == 1 ]] || scoped_wrong+="放行之后驱动应当照起一次（调了 $(grep -c . "$log") 次）；"
  [[ -z "$scoped_wrong" ]] || selftest_cell_failed "库里录下的块键都对得上，不论上一趟的态数多大都应当放行、现判 0 态：$scoped_wrong" \
    "看 lib_crash_point_replay_items.py 的 estimate_subject：录下的块大于 0 且键对不上的块是 0 就是现判 0 态"
  # 第 63 格：给了 --crash-expand-up-to 6（小于引擎常量 30）⇒ 缩过域，放行，不起读库程序
  cells=$((cells + 1)); : > "$log"; scoped_wrong=""; : > "$work/item-calls"
  STUB_ITEM_RECORDED=4 STUB_ITEM_TO_JUDGE=4 STUB_ITEM_REUSABLE=0 STUB_ITEM_CALLS="$work/item-calls" scoped_entry --item tree-split-streams --crash-expand-up-to 6 >"$work/scoped.out" 2>"$errors"; exit_code=$?
  [[ "$exit_code" == 0 ]] || scoped_wrong+="缩过域应当放行（退 $exit_code）；"
  grep -qF "缩过域" "$work/scoped.out" || scoped_wrong+="应当写缩过域；"
  [[ ! -s "$work/item-calls" ]] || scoped_wrong+="缩过域不判上限，不该起读库程序（起了：$(tr '\n' '|' <"$work/item-calls" | cut -c1-200)）；"
  [[ -z "$scoped_wrong" ]] || selftest_cell_failed "带了 --crash-expand-up-to 6 应当放行、不读库、不判上限：$scoped_wrong" \
    "看 scoped_domain_guard 交 --expand-up-to 与 run_scoped_domain 开头缩域那一支"
  # 第 64 格：没有任何运行记录、库里一块都没录 ⇒ 估不出，不放行，退 79
  cells=$((cells + 1)); : > "$log"; scoped_wrong=""; rm -rf -- "${scoped_home:?}"
  STUB_ITEM_RECORDED=0 STUB_ITEM_TO_JUDGE=0 STUB_ITEM_REUSABLE=0 scoped_entry --item tree-split-streams >/dev/null 2>"$errors"; exit_code=$?
  [[ "$exit_code" == 79 ]] || scoped_wrong+="估不出应当退 79（退 $exit_code）；"
  grep -qF "算不出" "$errors" || scoped_wrong+="说明里应当写算不出；"
  grep -qF -- "--crash-expand-up-to" "$errors" || scoped_wrong+="说明里应当写 --crash-expand-up-to 这条出路；"
  [[ ! -s "$log" ]] || scoped_wrong+="拒了之后不该起分项驱动；"
  [[ -z "$scoped_wrong" ]] || selftest_cell_failed "库里没录这一项的块、也没有上一趟的运行记录，应当退 79、说明写算不出与出路、一项驱动都不起：$scoped_wrong" \
    "看 lib_crash_point_replay_items.py 的 estimate_subject 算不出那一支与 refusal_text"
  # 第 65 格：只剩缩到 6 的那一趟记录（225 态）、库里 4 块键都对不上 ⇒ 缩过域的记录不当全域，估不出，退 79
  cells=$((cells + 1)); : > "$log"; scoped_wrong=""; rm -rf -- "${scoped_home:?}"
  scoped_record entry-scoped-old tree-split-streams 6 225 2
  STUB_ITEM_RECORDED=4 STUB_ITEM_TO_JUDGE=4 STUB_ITEM_REUSABLE=0 scoped_entry --item tree-split-streams >/dev/null 2>"$errors"; exit_code=$?
  [[ "$exit_code" == 79 ]] || scoped_wrong+="只有缩过域的记录应当退 79（退 $exit_code）；"
  grep -qF "没有不缩域那一趟的真跑记录" "$errors" || scoped_wrong+="说明里应当写没有不缩域那一趟的真跑记录；"
  [[ -z "$scoped_wrong" ]] || selftest_cell_failed "缩过域那一趟的态数少，拿来当全域会估小，只有它时应当算不出、退 79：$scoped_wrong" \
    "看 lib_crash_point_replay_items.py 的 full_prior：只认展开深度等于引擎常量的记录"
  # 第 66 格：崩溃点重放入口起这里时交 SINGLEFS_CRASH_SCOPED_DOMAIN_CHECKED=1（它已按合表项判过）⇒ 记录与块账明明会拒，这里也不再判，驱动照起
  cells=$((cells + 1)); : > "$log"; scoped_wrong=""; rm -rf -- "${scoped_home:?}"
  scoped_record entry-scoped-new tree-split-streams 30 300000 1
  SINGLEFS_CRASH_SCOPED_DOMAIN_CHECKED=1 STUB_ITEM_RECORDED=4 STUB_ITEM_TO_JUDGE=4 STUB_ITEM_REUSABLE=0 scoped_entry --item tree-split-streams >/dev/null 2>"$errors"; exit_code=$?
  [[ "$exit_code" == 0 ]] || scoped_wrong+="收到已判过的标记应当不再判（退 $exit_code）；"
  [[ "$(grep -c . "$log")" == 1 ]] || scoped_wrong+="驱动应当照起一次（调了 $(grep -c . "$log") 次）；"
  [[ -z "$scoped_wrong" ]] || selftest_cell_failed "收到 SINGLEFS_CRASH_SCOPED_DOMAIN_CHECKED=1 就不再判指定范围的上限：$scoped_wrong" \
    "看 scoped_domain_guard 前面那个 if：变量是 1 就不调"
  rm -rf -- "${scoped_home:?}" "${work:?}/item-calls" "${work:?}/scoped.out" "${segments_root:?}/crates"
  if (( failures > 0 )); then
    echo "  ✗ --selftest：$cells 格里 $failures 格判错" >&2
    echo "     → 怎么办：照上面每一格给的出路改，再跑 $0 --selftest" >&2
    exit 1
  fi
  if (( nested )); then echo "  ✓ --selftest（开关轮里起的一份）：$cells 格都对"; exit 0; fi

  # 开关轮：从文件头「弄坏开关」那张表现读每个开关与它该红的格（共用库的 crash_amplification_read_switch_table，与双机驱动同一份读法），
  #   逐个打开再跑一遍 --selftest，红的格要与表里写的相等
  local table assignment named round_failures=0 switch_table_source="${CRASH_AMPLIFICATION_SWITCH_TABLE:-$0}"
  [[ "$switch_table_source" == "$0" ]] || echo "  · 开关轮：开关表读自 $switch_table_source（CRASH_AMPLIFICATION_SWITCH_TABLE），代码里认的开关取自 $0"
  switch_wheel_assignments=(); switch_wheel_expected=()
  table="$(crash_amplification_read_switch_table "$switch_table_source" CRASH_AMPLIFICATION_BREAK)"
  while IFS=$'\t' read -r assignment named; do
    [[ -n "$assignment" ]] || continue
    switch_wheel_assignments+=("$assignment"); switch_wheel_expected+=("$named")
  done <<<"$table"
  if (( ${#switch_wheel_assignments[@]} == 0 )); then
    echo "  ✗ 开关轮：文件头「弄坏开关」那张表一个开关都没读到" >&2
    echo "     → 怎么办：表里一行「#   CRASH_AMPLIFICATION_BREAK=<开关名>  <弄坏哪一处>」，下一行起以「#       」开头写点名的格「第 N 格」（文件头「弄坏开关」一段的写法）" >&2
    exit 1
  fi
  local in_code in_table
  in_code="$(grep -oE 'crash_amplification_break_is [a-z0-9-]+' "$0" | awk '{ print $2 }' | LC_ALL=C sort -u)"
  in_table="$(printf '%s\n' "${switch_wheel_assignments[@]}" | sed -n 's/^CRASH_AMPLIFICATION_BREAK=//p' | LC_ALL=C sort -u)"
  if [[ "$in_code" != "$in_table" ]]; then
    echo "  ✗ 开关轮：代码里认的开关与文件头表里列的不一样（代码里：$(tr '\n' ' ' <<<"$in_code")；表里：$(tr '\n' ' ' <<<"$in_table")）" >&2
    echo "     → 怎么办：代码里每一处 crash_amplification_break_is 认的开关都在表里写一行，表里每一行都在代码里有对应的一处" >&2
    round_failures=$((round_failures + 1))
  fi
  # 逐个打开、收回、比格：共用库的 crash_amplification_switch_wheel（与双机驱动同一份判法），格号从「✗ 自证第 <号> 格」取
  crash_amplification_switch_wheel "$work" '✗ 自证第 ([0-9]+) 格' 0 -- bash "$0" --selftest
  round_failures=$((round_failures + switch_wheel_failures))
  if (( round_failures > 0 )); then
    echo "  ✗ --selftest：$cells 格都对，开关轮有 $round_failures 处对不上" >&2
    echo "     → 怎么办：红的格与表里写的不同，要么改那一处判法让开关红在点名的格，要么照实改文件头那一行；单看一个开关：CRASH_AMPLIFICATION_BREAK=<开关> $0 --selftest" >&2
    exit 1
  fi
  echo "  ✓ --selftest：列项、拒不认得的名与缺项名、不带参数不交展开深度、点名取并集、不清场（清场由 task-server 管）、认不出的选项退 2、给了 --crash-expand-up-to 把变量交给每一项的分项驱动并在 TOTAL 行与成功句写缩过域（环境里设了的当作给了同一个值、两样都给取参数）、缩域的值写坏退 2、照常退 0 有项红退 1、驱动退非 0 与 red>0 判红、复用数的取法与取不出照实报、叫停的行不推复用数、派出收回、0 行判红、不穷举判红（不带参数时红在不穷举的那一项）、叫停与没判完报本次未跑与 77、驱动退 78 停下、平常跑拒只供自证的四个变量并逐个点名、有红又有项退 78 时退 1、退 78 而行里报红记红、按登记整批不判报数与取不出、按登记整批不判的两个数按流分组每组只取一次（同流各行重复报不重复计，不同的流各取一次相加，同流各行不同记取不出并点名流）、red= 缺或坏判红、不带参数时零态判红、复用数是各行之和（计划行不取）、几条流各留的块相加报本次未跑、分项驱动退 77 记本次未跑（全是它时退 77）、几条流里有的没借到显存记本次未跑不算通过、只因登记的段不穷举不判红并报段号与写数（没登段的流照旧红）、每一趟往驱动送按段登记文件、登记文件写坏整趟拒跑、登了段的流里另有展开上限盖不住又没登记的段红在那条流与那几段（缩过规模时成功句报那几段）、驱动行没报那个字段照不穷举红（出路重编引擎）、登记段分片枚举不红而 ITEM 行与成功句报全展开与分片已判几态（总态数是超过 u64 的十进制串）、分片段已判 0 态而现判与复用都是 0 红在那条流与那一段、登了段的行登记段的域与分片进度没报全漏段或写坏照重编引擎红、全部复用的一趟不红、分项驱动报判段没跑（step=judge）记本次未跑、分项驱动退 77 而 NOT_RUN 行是跑的过程中源码变了时原因照 reason= 写不写借不到显存、点名两项以上当场声明项与项之间串行（一项不声明），$cells 格都对；开关轮 ${#switch_wheel_assignments[@]} 个开关（文件头现读）逐个打开，派出 ${#switch_wheel_assignments[@]} 个、收回 $switch_wheel_collected 个，各红在点名的那几格"
  exit 0
}
if (( selftest )); then run_selftest; fi

unset CRASH_AMPLIFICATION_BREAK CRASH_AMPLIFICATION_SELFTEST_RED_ITEM CRASH_AMPLIFICATION_SELFTEST_DROP_RESULT CRASH_AMPLIFICATION_ITEM_DRIVER CRASH_AMPLIFICATION_SELFTEST_SEGMENTS_ROOT

driver="${test_item_driver:-$crash_amplification_script_directory/crash-amplification-item.sh}"
[[ -f "$driver" ]] || { echo "  ✗ 分项驱动不在：$driver" >&2; echo "  → 怎么办：scripts/crash/crash-amplification-item.sh 要在；自证起的子进程用 CRASH_AMPLIFICATION_ITEM_DRIVER 指假的" >&2; exit 2; }

# 选分项：没点名就是全量；点名的每个都要在登记表里，重名只取一次
rows_to_run=()
if (( ${#selected[@]} == 0 )); then
  rows_to_run=("${ITEMS[@]}")
else
  declare -A selected_already=()
  for name in "${selected[@]}"; do
    [[ -n "${selected_already[$name]:-}" ]] && continue
    selected_already["$name"]=1
    found=""
    for row in "${ITEMS[@]}"; do [[ "$(item_fields "$row" 1)" == "$name" ]] && found="$row"; done
    if [[ -z "$found" ]]; then
      if crash_amplification_break_is accept-unknown-item; then found="$name"$'\t'"$name"$'\t'"$name"; else
        refuse_listing_the_items "不认得的分项：$name"
      fi
    fi
    rows_to_run+=("$found")
  done
fi

# 这一趟的范围
if (( ${#selected[@]} > 0 )); then scope=items; else scope=whole; fi

# 按段登记（文件头「按段登记」）：开跑前经 lib_crash_point_replay_items.py segments 核被判仓根下那一份，核不过退 2、一项都不跑、清场也不起；
#   核过了往引擎送它的绝对路径（与 scripts/crash/crash-point-replay.sh 送的同一个文件）。自证起的子进程里仓根换成 CRASH_AMPLIFICATION_SELFTEST_SEGMENTS_ROOT
segments_root="${selftest_segments_root:-$repository_root}"
segments_registry="$segments_root/.claude/gate.d/crash-point-replay-segments.tsv"   # 路径同 lib_crash_point_replay_items.py 的 SEGMENTS_TABLE
if ! crash_amplification_break_is segments-registry-unchecked; then
  if ! segments_listing="$(python3 "$crash_amplification_script_directory/lib_crash_point_replay_items.py" segments "$segments_root" 2>&1)"; then
    printf '%s\n' "$segments_listing" >&2
    echo "  ✗ 按段登记文件 $segments_registry 核不过（lib_crash_point_replay_items.py segments），一项都没跑" >&2
    echo "  → 怎么办：照上面逐处列的改 .claude/gate.d/crash-point-replay-segments.tsv（写法在它的表头），再跑同一条命令" >&2
    exit 2
  fi
fi
if crash_amplification_break_is segments-registry-not-sent; then
  unset SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY
else
  export SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY="$segments_registry"
fi

# 经 task-server 按项分片（文件头「经 task-server 按项分片」）：读完就从环境里清掉，不传给分项驱动
shard_phase="${TASK_SERVER_SHARD_PHASE:-}"; shard_position="${TASK_SERVER_SHARD:-}"; shard_plan_directory="${TASK_SERVER_SHARD_PLAN_DIRECTORY:-}"
shard_output_directory="${TASK_SERVER_SHARD_OUTPUT_DIRECTORY:-}"; shard_outputs="${TASK_SERVER_SHARD_OUTPUTS:-}"
unset TASK_SERVER_SHARD_PHASE TASK_SERVER_SHARD TASK_SERVER_SHARD_PLAN_DIRECTORY TASK_SERVER_SHARD_OUTPUT_DIRECTORY TASK_SERVER_SHARD_OUTPUTS TASK_SERVER_SHARD_HOST
crash_amplification_break_is shard-phase-ignored && shard_phase=""
shard_replay_directories=()
shard_refuse() { # <卡在哪> <下一步> <退出码>
  echo "  ✗ 按项分片的 $shard_phase 那一步：$1" >&2
  echo "  → 怎么办：$2" >&2
  exit "$3"
}
# 带 --no-shard 不分（文件头「经 task-server 按项分片」）：计划写一片，那一片整趟跑，合并什么都不做
if [[ -n "$shard_phase" ]] && (( no_shard )) && ! crash_amplification_break_is no-shard-ignored; then
  case "$shard_phase" in
    plan)
      [[ -n "$shard_plan_directory" ]] || shard_refuse "环境里没有 TASK_SERVER_SHARD_PLAN_DIRECTORY" "这一步只由 task-server 照分片协议起（task-server/task_server_lib.py 文件头「分片」）" 2
      mkdir -p -- "$shard_plan_directory" || shard_refuse "建不了计划目录 $shard_plan_directory" "看共享工作根可不可写" 78
      printf '{"shards": [{"label": "整趟"}], "note": "不分片：--no-shard，整趟一片"}\n' > "$shard_plan_directory/plan.json"
      echo "  ✓ 不分片（--no-shard）：计划写一片，那一片整趟跑"
      exit 0 ;;
    shard)
      mkdir -p -- "$shard_output_directory" || shard_refuse "建不了产物目录 ${shard_output_directory:-（没给）}" "看共享工作根可不可写" 78
      shard_phase="" ;;
    merge)
      echo "  · 不分片（--no-shard）：结论在那一片里，合并什么都不做"
      exit 0 ;;
  esac
fi
# 指定范围的崩溃域上限（文件头 usage 一段「指定范围的崩溃域上限」）：带了 --item、没缩域才判；经服务按项分片的在计划那一步判（各片与合并不再判），崩溃点重放入口起这里时交
#   SINGLEFS_CRASH_SCOPED_DOMAIN_CHECKED=1（它已按合表项判过）就不再判。超上限或估不出退 79，一项都没起。弄坏开关 scoped-domain-handshake-ignored 下收到那个变量也照判
if (( ${#selected[@]} > 0 )) && [[ "$shard_phase" != shard && "$shard_phase" != merge ]] \
   && { [[ "${SINGLEFS_CRASH_SCOPED_DOMAIN_CHECKED:-}" != 1 ]] || crash_amplification_break_is scoped-domain-handshake-ignored; }; then
  scoped_domain_guard "${rows_to_run[@]}" || exit $?
fi
case "$shard_phase" in
  "") ;;
  plan)
    [[ -n "$shard_plan_directory" ]] || shard_refuse "环境里没有 TASK_SERVER_SHARD_PLAN_DIRECTORY" "这一步只由 task-server 照分片协议起（task-server/task_server_lib.py 文件头「分片」）" 2
    mkdir -p -- "$shard_plan_directory" || shard_refuse "建不了计划目录 $shard_plan_directory" "看共享工作根可不可写" 78
    printf '%s\n' "${rows_to_run[@]}" > "$shard_plan_directory/rows.tsv"
    python3 - "$shard_plan_directory/rows.tsv" "$shard_plan_directory/plan.json" <<'PYTHON' || shard_refuse "写不成 plan.json" "看计划目录 $shard_plan_directory" 1
import json, sys
rows = [line.rstrip("\n") for line in open(sys.argv[1], encoding="utf-8") if line.strip()]
plan = {"shards": [{"label": row.split("\t")[0]} for row in rows], "note": f"崩溃放量按项分片：{len(rows)} 项、一项一片"}
with open(sys.argv[2], "w", encoding="utf-8") as handle:
    json.dump(plan, handle, ensure_ascii=False)
PYTHON
    echo "  ✓ 按项分片的计划：${#rows_to_run[@]} 项、一项一片，写进 $shard_plan_directory（rows.tsv、plan.json）"
    exit 0 ;;
  shard|merge)
    [[ -f "$shard_plan_directory/rows.tsv" ]] || shard_refuse "计划目录里没有 rows.tsv（${shard_plan_directory:-没给计划目录}）" "这一步只由 task-server 照分片协议在计划那一步之后起" 2
    mapfile -t shard_plan_rows < "$shard_plan_directory/rows.tsv"
    if [[ "$shard_phase" == shard ]]; then
      [[ "$shard_position" =~ ^([0-9]+)/([0-9]+)$ ]] && (( BASH_REMATCH[1] < ${#shard_plan_rows[@]} )) \
        || shard_refuse "TASK_SERVER_SHARD=$shard_position 对不上计划的 ${#shard_plan_rows[@]} 项" "照计划那一步写的 plan.json 的片数起" 2
      [[ -n "$shard_output_directory" ]] && mkdir -p -- "$shard_output_directory" || shard_refuse "建不了产物目录 ${shard_output_directory:-（没给）}" "看共享工作根可不可写" 78
      rows_to_run=("${shard_plan_rows[${BASH_REMATCH[1]}]}")
      echo "  · 按项分片：第 $shard_position 片，只跑 $(item_fields "${rows_to_run[0]}" 1)"
    else
      [[ -f "$shard_outputs" ]] || shard_refuse "环境里的 TASK_SERVER_SHARD_OUTPUTS 不是一份文件（${shard_outputs:-没给}）" "这一步只由 task-server 在各片都成了之后起" 2
      while IFS=$'\t' read -r shard_index _ shard_directory; do
        [[ "$shard_index" =~ ^[0-9]+$ ]] && shard_replay_directories[shard_index]="$shard_directory"
      done < "$shard_outputs"
      for (( shard_index = 0; shard_index < ${#shard_plan_rows[@]}; shard_index++ )); do
        shard_directory="${shard_replay_directories[shard_index]:-}"
        [[ -n "$shard_directory" && -f "$shard_directory/item.log" && -f "$shard_directory/item.exit" ]] \
          || shard_refuse "第 $shard_index 片（$(item_fields "${shard_plan_rows[shard_index]}" 1)）的产物不全：产物表 $shard_outputs 里$([[ -n "$shard_directory" ]] && echo "指到 $shard_directory，缺 item.log 或 item.exit" || echo "没有这一片")，一项都没重放" \
            "看那一片的日志（task-server/task status <号> 列各片）；重交整件，判过的块照单元键复用" 1
      done
      rows_to_run=("${shard_plan_rows[@]}")
      echo "  · 按项分片的合并：${#rows_to_run[@]} 项照各片存下的输出重放（不起驱动）"
    fi ;;
  *) shard_refuse "TASK_SERVER_SHARD_PHASE=$shard_phase 认不得（只认 plan、shard、merge）" "照 task-server/task_server_lib.py 文件头「分片」的协议起" 2 ;;
esac

# RocksDB 预编库的取值（scripts/hosts/rocksdb-prebuilt.py 文件头「entry」）：起任何一项（分项驱动里起 cargo）之前判一次这台装了、此刻进程环境的取值对得上，
#   判红退 78、一项都不跑。判的树是被判的仓根（自证起的子进程里同样换成 CRASH_AMPLIFICATION_SELFTEST_SEGMENTS_ROOT 那一棵小仓：没有依赖 rocksdb 的清单，entry 报不判）。
#   弄坏开关 rocksdb-entry-unchecked 下不判（第 48 格判错）
if (( ! rocksdb_entry_checked )) && ! crash_amplification_break_is rocksdb-entry-unchecked && ! python3 "$crash_amplification_script_directory/../hosts/rocksdb-prebuilt.py" entry --root "$segments_root" >&2; then
  echo "  ✗ 起 cargo 之前 RocksDB 预编库的取值没判过（上面 rocksdb-prebuilt.py entry 的红句），一项都没跑（退 78）" >&2
  echo "  → 怎么办：照上面那几行的出路修（开新会话，或命令前 eval \"\$(python3 scripts/hosts/rocksdb-prebuilt.py env)\"），再跑同一条命令" >&2
  exit 78
fi

# 每项的输出另存一份：边跑边转打，跑完从文件取汇总
crash_amplification_home="$(crash_amplification_home_directory)" || exit 2
entry_run_directory="$crash_amplification_home/runs/entry-$(date -u +%Y%m%dT%H%M%SZ)-$$"
if ! mkdir -p -- "$entry_run_directory"; then
  echo "  ✗ 建不了这一趟放每项输出的目录 $entry_run_directory，一项都没跑" >&2
  echo "  → 怎么办：修好设备，或改配置：看 $crash_amplification_home 可不可写，或把 SINGLEFS_CRASH_AMPLIFICATION_HOME 指到可写的目录（退 78）" >&2
  exit 78
fi
# 这一趟的编译目录（文件头「编译目录」）：调用方没设 CARGO_TARGET_DIR 时建 <这一趟的目录>/target、export 给每一项的驱动；正常结束、红了、被 TERM / INT 停都在 EXIT 上删
#   （删之前核没有进程开着里面的文件，有就整趟退非 0）。弄坏开关 run-target-kept：不删
source "$crash_amplification_script_directory/../hosts/run-build-directory.sh"
run_build_directory_begin "$entry_run_directory" || exit 78
crash_amplification_entry_finish() {
  local finish_exit=$?
  trap - EXIT
  if ! run_build_directory_end $(crash_amplification_break_is run-target-kept && echo keep); then
    (( finish_exit != 0 )) || finish_exit=1
  fi
  # 按项分片的一片：这一项判红或本次未跑都退 0，结论交合并那一步判（文件头「经 task-server 按项分片」）
  if [[ "$shard_phase" == shard ]] && (( finish_exit == 1 || finish_exit == 77 )) && [[ -f "$shard_output_directory/item.exit" ]]; then
    echo "  · 按项分片的一片：这一项整趟退 $finish_exit，输出已存进 $shard_output_directory，结论交合并那一步判；这一片退 0"
    finish_exit=0
  fi
  exit "$finish_exit"
}
trap crash_amplification_entry_finish EXIT
trap 'exit 143' TERM
trap 'exit 130' INT
if [[ -n "${SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY:-}" ]]; then
  echo "  · 往引擎送按段登记文件 SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY=$SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY（$(grep -cvE '^(#|[[:space:]]*$)' "$SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY") 行登记）" >&2
fi

# 缩过域（文件头「展开深度」）：TOTAL 行接 expand_up_to=<n>、成功句写缩过域、不算门禁通过；弄坏开关 narrowed-run-unlabelled：两处都不写
narrowed_total_part=""; narrowed_success_part=""
if [[ -n "$crash_expand_up_to" ]] && ! crash_amplification_break_is narrowed-run-unlabelled; then
  narrowed_total_part=" expand_up_to=$crash_expand_up_to"
  narrowed_success_part="（缩过域、不算门禁通过：展开深度上限缩到 $crash_expand_up_to，只往小缩）"
fi

started_at=$(date +%s)
items_dispatched=0; items_run=0; items_failed=0; items_not_run=0; items_reuse_unknown=0
reuse_unknown_items=""; not_run_items=""; not_run_entries=(); failed_entries=(); failed_names=(); last_run_ran=()
total_states=0; total_reused=0; total_red=0; stopped_by_condition=""
total_ignored_points=0; total_states_skipped_by_ignore=0; items_ignore_unknown=0; ignore_unknown_items=""; ignored_items=""; ignore_conflict_seen=0
not_exhaustive_failed_names=(); engine_field_missing_failed_names=()
registered_items=0; registered_segment_count=0; registered_entries=""
beyond_items=0; beyond_segment_count=0; beyond_entries=""; beyond_unknown_items=0; beyond_unknown_names=""
domain_field_missing_failed_names=(); slice_zero_failed_names=()
domain_segment_count=0; domain_full_count=0; domain_slice_count=0; domain_items=0; domain_entries=""
row_index=-1
for row in "${rows_to_run[@]}"; do
  row_index=$((row_index + 1))
  name="$(item_fields "$row" 1)"; target="$(item_fields "$row" 2)"; function_name="$(item_fields "$row" 3)"
  # 自证用：把测试目标改成带 -red 的名字，让假驱动退 1
  [[ -n "$selftest_red_item" ]] && target="${target}-red"
  filter_arguments=(--test-filter "$function_name")
  crash_amplification_break_is skip-filter && filter_arguments=()
  item_output_file="$entry_run_directory/$name.log"
  echo "== 分项 $name（$target::$function_name）$(date -u +%FT%TZ)；输出另存 $item_output_file，错误警告 ${item_output_file%.log}.problems.log"
  item_started_at=$(date +%s)
  items_dispatched=$((items_dispatched + 1))
  # 驱动的输出边跑边转打，同时出两份日志（scripts/hosts/two-logs.sh：$item_output_file 是全量，同名的 .problems.log 只收错误与警告，行前带全量日志里的行号，一般只看它）；
  #   退出码是驱动的。弄坏开关 logs-single：照改前只写一份全量日志（tee）
  #   按项分片的合并那一步（文件头「经 task-server 按项分片」）：不起驱动，照那一片存下的输出与退出码重放
  if [[ "$shard_phase" == merge ]]; then
    item_command=(bash -c 'cat -- "$1"; exit "$(cat -- "$2")"' shard-replay "${shard_replay_directories[row_index]}/item.log" "${shard_replay_directories[row_index]}/item.exit")
  else
    item_command=(bash "$driver" "$target" "${filter_arguments[@]+"${filter_arguments[@]}"}" "${driver_arguments[@]+"${driver_arguments[@]}"}")
  fi
  if crash_amplification_break_is logs-single; then
    "${item_command[@]}" 2>&1 | tee -- "$item_output_file"
    pipe_status=("${PIPESTATUS[@]}"); item_exit="${pipe_status[0]}"
  elif bash "$crash_amplification_script_directory/../hosts/two-logs.sh" "${item_output_file%.log}" -- "${item_command[@]}" 2>&1; then
    item_exit=0
  else
    item_exit=$?
  fi
  # 按项分片的一片：这一项的全量输出与驱动退出码存进产物目录，交合并那一步重放
  if [[ "$shard_phase" == shard ]]; then
    cp -- "$item_output_file" "$shard_output_directory/item.log" && printf '%s\n' "$item_exit" > "$shard_output_directory/item.exit" \
      || { echo "  ✗ 存不进这一片的产物目录 $shard_output_directory" >&2; echo "  → 怎么办：看共享工作根可不可写，再重交整件" >&2; exit 78; }
  fi
  crash_amplification_break_is swallow-exit && item_exit=0
  read -r item_lines item_states item_red item_reused item_exhaustive item_expand item_stopped item_unjudged item_bad_red item_ignored item_skipped \
    item_not_exhaustive_lines item_not_exhaustive_flows item_registered_segments item_beyond_lines item_beyond_segments item_beyond_missing_lines item_beyond_missing_flows \
    item_beyond_report item_domains item_progress item_domain_bad_lines item_domain_bad_flows item_zero_lines item_zero_segments item_ignore_conflict_flows <<<"$(summarize_the_item "$item_output_file")"
  item_lines="${item_lines:-0}"; item_states="${item_states:-0}"; item_red="${item_red:-0}"; item_reused="${item_reused:-unknown}"
  item_exhaustive="${item_exhaustive:-false}"; item_expand="${item_expand:-unknown}"; item_stopped="${item_stopped:-0}"; item_unjudged="${item_unjudged:-0}"
  item_bad_red="${item_bad_red:-0}"; item_ignored="${item_ignored:-unknown}"; item_skipped="${item_skipped:-unknown}"
  # 不穷举又没按段登记的行数与流名（文件头「展开深度」「按段登记」）、按段登记的段
  item_not_exhaustive_lines="${item_not_exhaustive_lines:-0}"; item_not_exhaustive_flows="${item_not_exhaustive_flows:--}"; item_registered_segments="${item_registered_segments:--}"
  # 登了段的行里另有展开上限盖不住又没登记的段、没报那个字段的（文件头「按段登记」），与 ITEM 行报的那个字段
  item_beyond_lines="${item_beyond_lines:-0}"; item_beyond_segments="${item_beyond_segments:--}"
  item_beyond_missing_lines="${item_beyond_missing_lines:-0}"; item_beyond_missing_flows="${item_beyond_missing_flows:--}"; item_beyond_report="${item_beyond_report:-unknown}"
  # 登记段的域与分片进度（文件头「按段登记」）：ITEM 行报的两个字段、缺漏认不得的行与流名、分片段已判 0 态的行与那几段
  item_domains="${item_domains:-unknown}"; item_progress="${item_progress:-unknown}"
  item_domain_bad_lines="${item_domain_bad_lines:-0}"; item_domain_bad_flows="${item_domain_bad_flows:--}"
  item_zero_lines="${item_zero_lines:-0}"; item_zero_segments="${item_zero_segments:--}"
  # 按登记不判的两个数同一组各行报的不同的流名（文件头「按登记整批不判」）
  item_ignore_conflict_flows="${item_ignore_conflict_flows:--}"
  if [[ "$item_reused" == unknown ]] && crash_amplification_break_is accept-unknown-reuse; then item_reused=0; fi
  # 分项驱动报的本次未跑（文件头「一项怎么判」not-run）：驱动退 77（一项都没跑到），或打了 CRASH_AMPLIFICATION_NOT_RUN 行（几条流里没借到的那几条）；
  #   原因取第一行的 reason=。弄坏开关 driver-77-as-failure 下退 77 照别的非 0 判红，driver-not-run-as-zero-lines 下不认那几行
  #   判段没跑的流（CRASH_AMPLIFICATION_NOT_RUN step=judge：配置写了 ENABLE_GPU=1 而二进制没编进 gpu 特性）另数一份，原因照它写；弄坏开关 judge-not-run-ignored 下不认那几行
  item_driver_not_run=0; item_driver_not_run_reason=""; item_driver_not_run_step=""; item_driver_not_run_lines=0; item_exit_not_run=0; item_judge_not_run_lines=0
  if ! crash_amplification_break_is driver-not-run-as-zero-lines; then
    item_driver_not_run_lines="$(grep -c '^CRASH_AMPLIFICATION_NOT_RUN ' "$item_output_file" 2>/dev/null)"
    item_driver_not_run_lines="${item_driver_not_run_lines:-0}"
    item_judge_not_run_lines="$(grep -c '^CRASH_AMPLIFICATION_NOT_RUN step=judge ' "$item_output_file" 2>/dev/null)"
    item_judge_not_run_lines="${item_judge_not_run_lines:-0}"
    if crash_amplification_break_is judge-not-run-ignored; then
      item_driver_not_run_lines=$((item_driver_not_run_lines - item_judge_not_run_lines)); item_judge_not_run_lines=0
    fi
    item_driver_not_run_reason="$(awk '/^CRASH_AMPLIFICATION_NOT_RUN / { at = index($0, " reason="); print (at ? substr($0, at + 8) : ""); exit }' "$item_output_file")"
    item_driver_not_run_step="$(awk '/^CRASH_AMPLIFICATION_NOT_RUN / { if (match($0, / step=[^ ]+/)) print substr($0, RSTART + 6, RLENGTH - 6); exit }' "$item_output_file")"
  fi
  if [[ "$item_exit" == 77 ]] && ! crash_amplification_break_is driver-77-as-failure; then item_exit_not_run=1; fi
  if (( item_driver_not_run_lines > 0 || item_exit_not_run )); then item_driver_not_run=1; fi
  # 自证用：跑完点名那一项不打它的结果行、不计进汇总，造一项收不回
  if [[ -n "$selftest_drop_result" && "$name" == "$selftest_drop_result" ]]; then continue; fi
  # 判这一项（文件头「一项怎么判」）：行里报的红先收，驱动退几都算（退 78 也不让它盖掉）
  reasons=(); red_reasons=(); verdict=green; not_run_reason=""; item_stopped_by_condition=0
  if [[ "$item_exit" == 78 ]] && ! crash_amplification_break_is item-78-as-failure; then item_stopped_by_condition=1; fi
  if (( item_red > 0 )) && ! crash_amplification_break_is ignore-red-count; then red_reasons+=("CRASH_AMPLIFICATION 行报了 $item_red 处红"); fi
  if (( item_bad_red > 0 )) && ! crash_amplification_break_is accept-bad-red-field; then
    red_reasons+=("有 $item_bad_red 行 CRASH_AMPLIFICATION 缺 red= 或 red 不是非负整数（驱动行的约定是每行带 red=<非负整数>）：看驱动与用例打这一行的那一处")
  fi
  if (( item_stopped_by_condition )); then
    stopped_by_condition="$name"
    if (( ${#red_reasons[@]} > 0 )) && ! crash_amplification_break_is ignore-red-on-78; then
      verdict=red; reasons=("驱动退 78（显卡或第二台用不上），可它退出之前打的行里报了红：判出来的红不让 78 盖掉" "${red_reasons[@]}")
    else
      verdict=not-run; not_run_reason="驱动退 78：显卡或第二台用不上"
    fi
  else
    if [[ "$item_exit" != 0 ]] && (( ! item_exit_not_run )); then
      if [[ "$item_exit" == 248 ]]; then reasons+=("驱动退 248：算法上判不了（有状态 GPU 判不了、内核展开超预算、建不出表），不是设备的事，照它的输出改判器")
      else reasons+=("驱动退 $item_exit"); fi
    fi
    reasons+=(${red_reasons[@]+"${red_reasons[@]}"})
    if [[ "$scope" == whole && "$item_states" == 0 && "$item_reused" == 0 ]] && (( item_stopped == 0 && item_unjudged == 0 && item_driver_not_run == 0 )) \
       && ! crash_amplification_break_is accept-zero-whole-item; then
      reasons+=("不带参数时这一项现判 0 态、复用 0 态：一态都没判到不是通过；看 --test-filter $function_name 筛中的用例是不是空跑，驱动行的 states_checked_here 与 states_reused 有没有照实报")
    fi
    if [[ "$item_exit" == 0 ]] && (( item_lines == 0 && item_driver_not_run == 0 )) && ! crash_amplification_break_is accept-zero-lines; then
      reasons+=("驱动退 0 却 0 行 CRASH_AMPLIFICATION：看测试名还在不在（$target）、cargo 是不是筛出了 0 条（--test-filter $function_name）")
    fi
    # 不穷举（文件头「展开深度」「按段登记」）：有一行不是 exhaustive=true、又不是「exhaustive=false 只因登记了段」
    #   （registered_segments= 不是 -、unregistered_segments_beyond_expansion= 是 -），这一项红。
    #   弄坏开关 skip-not-exhaustive-in-whole-scope 下不带参数的跑法不判（点名的照判）
    exhaustive_judged=0
    if (( item_lines > 0 )) && ! crash_amplification_break_is accept-not-exhaustive \
       && ! { [[ "$scope" == whole ]] && crash_amplification_break_is skip-not-exhaustive-in-whole-scope; }; then
      exhaustive_judged=1
    fi
    if (( exhaustive_judged && item_not_exhaustive_lines > 0 )); then
      reasons+=("有 $item_not_exhaustive_lines 行驱动行不是 exhaustive=true、也没按段登记（不穷举的流：$item_not_exhaustive_flows）：每一行都要穷举或只因登记的段不穷举，这一项没判完全域；按段登记（.claude/gate.d/crash-point-replay-segments.tsv）或加深展开")
      not_exhaustive_failed_names+=("$name")
    fi
    # 登了段的行（文件头「按段登记」）：unregistered_segments_beyond_expansion= 不是 - 判红、点名流与那几段；没报（旧引擎）照不穷举判红，出路重编引擎
    if (( exhaustive_judged && item_beyond_lines > 0 )); then
      reasons+=("有 $item_beyond_lines 行驱动行登了段，可那条流里另有没登记、展开上限盖不住的段（$(segments_by_flow "$item_beyond_segments")）：那几段没判全域；按段登记（.claude/gate.d/crash-point-replay-segments.tsv）或加深展开")
      [[ " ${not_exhaustive_failed_names[*]-} " == *" $name "* ]] || not_exhaustive_failed_names+=("$name")
    fi
    if (( exhaustive_judged && item_beyond_missing_lines > 0 )); then
      reasons+=("有 $item_beyond_missing_lines 行驱动行登了段、exhaustive=false，却没报 unregistered_segments_beyond_expansion=（流：$item_beyond_missing_flows）：判不出登记段之外还有没有展开上限盖不住的段，照不穷举判；引擎没报 unregistered_segments_beyond_expansion=，重编引擎")
      engine_field_missing_failed_names+=("$name")
    fi
    # 登记段的域与分片进度（文件头「按段登记」）：不看展开深度。登了段的行缺两个字段之一、没列全登记的段（或分片段没报进度）、写法认不得，出路重编引擎；
    #   分片段这一趟已判 0 态，出路看录入段与判段的日志；分片本身不判红
    if (( item_domain_bad_lines > 0 )); then
      reasons+=("有 $item_domain_bad_lines 行驱动行登了段，registered_segment_domains= / registered_segment_progress= 缺了、没列全登记的段（或分片段没报进度）、或写法认不得（流：$item_domain_bad_flows）：判不出登记段是全展开还是分片、分片判到了哪；引擎没报 registered_segment_domains= / registered_segment_progress=，重编引擎")
      domain_field_missing_failed_names+=("$name")
    fi
    if (( item_zero_lines > 0 )); then
      reasons+=("有 $item_zero_lines 行驱动行的分片段已判 0 态（$(segments_by_flow "$item_zero_segments")）：这一趟一个态都没判，看录入段与判段的日志")
      slice_zero_failed_names+=("$name")
    fi
    if (( ${#reasons[@]} > 0 )); then
      verdict=red
    elif (( item_driver_not_run )); then
      verdict=not-run
      if (( item_exit_not_run && item_driver_not_run_lines > 0 )); then
        # 原因照第一行 NOT_RUN 的 step= 与 reason= 写：只有排计划那一趟（step=placement）是借不到显存，step=judge 的跑的过程中源码变了这类只照 reason= 写
        item_driver_not_run_cause="这一项没判成"
        if [[ "$item_driver_not_run_step" == placement ]] || crash_amplification_break_is not-run-cause-always-vram; then
          item_driver_not_run_cause="借不到显存，这一项没判成"
        fi
        not_run_reason="分项驱动退 77（一项都没跑到），报本次未跑（CRASH_AMPLIFICATION_NOT_RUN step=${item_driver_not_run_step:-没写}）：${item_driver_not_run_cause}（${item_driver_not_run_reason:-那一行没写 reason=}）"
      elif (( item_exit_not_run )); then
        not_run_reason="分项驱动退 77（一项都没跑到），没打 CRASH_AMPLIFICATION_NOT_RUN 行：原因看这一项的输出"
      elif (( item_judge_not_run_lines > 0 )); then
        item_judge_not_run_reason="$(awk '/^CRASH_AMPLIFICATION_NOT_RUN step=judge / { at = index($0, " reason="); print (at ? substr($0, at + 8) : ""); exit }' "$item_output_file")"
        not_run_reason="分项驱动报 $item_judge_not_run_lines 条流判段没跑（CRASH_AMPLIFICATION_NOT_RUN step=judge）：配置写了 ENABLE_GPU=1 而二进制没编进 gpu 特性，录入段照录、判段这一趟没判（${item_judge_not_run_reason:-那一行没写 reason=}）"
        if (( item_driver_not_run_lines > item_judge_not_run_lines )); then
          not_run_reason="${not_run_reason}；另有 $((item_driver_not_run_lines - item_judge_not_run_lines)) 条流借不到显存没判"
        fi
      else
        not_run_reason="分项驱动报 $item_driver_not_run_lines 条流本次未跑（CRASH_AMPLIFICATION_NOT_RUN）：借不到显存没判，借到的照判了（${item_driver_not_run_reason:-那一行没写 reason=}）"
      fi
      if (( item_stopped > 0 )); then not_run_reason="${not_run_reason}；另有 $item_stopped 行叫停（stopped_early=true）"; fi
    elif (( item_stopped > 0 || item_unjudged > 0 )) && ! crash_amplification_break_is count-stopped-as-run; then
      verdict=not-run
      (( item_stopped > 0 )) && not_run_reason="有 $item_stopped 行叫停（stopped_early=true）"
      (( item_unjudged > 0 )) && not_run_reason="${not_run_reason:+${not_run_reason}、}留了 $item_unjudged 块没判（unjudged_blocks）"
    fi
  fi
  # 按段登记的段（文件头「按段登记」）：ITEM 行照实写出，成功句逐项、逐条流报段号与写数；弄坏开关 registered-segments-unreported 下两处都不报
  crash_amplification_break_is registered-segments-unreported && item_registered_segments=-
  # 展开上限盖不住又没登记的段（文件头「按段登记」）：ITEM 行照实写出，成功句逐项、逐条流报段号与写数；弄坏开关 beyond-expansion-unreported 下两处都不报
  crash_amplification_break_is beyond-expansion-unreported && item_beyond_report=-
  # 登记段的域与分片进度（文件头「按段登记」）：ITEM 行照实写出，成功句逐段报全展开或分片已判几态；弄坏开关 slice-domains-unreported 下两处都不报
  if crash_amplification_break_is slice-domains-unreported; then item_domains=-; item_progress=-; fi
  echo "CRASH_AMPLIFICATION_ITEM item=$name exit=$item_exit states=$item_states reused=$item_reused red=$item_red lines=$item_lines expand_up_to=$item_expand exhaustive=$item_exhaustive verdict=$verdict seconds=$(( $(date +%s) - item_started_at )) ignored_points=$item_ignored states_skipped_by_ignore=$item_skipped registered_segments=$item_registered_segments unregistered_segments_beyond_expansion=$item_beyond_report registered_segment_domains=$item_domains registered_segment_progress=$item_progress"
  # 同一条流各行报的 ignored_points= / states_skipped_by_ignore= 不同：ITEM 行两个数写了 unknown，这里点名分项与流（不悄悄取最大或最小）
  if [[ "$item_ignore_conflict_flows" != - ]]; then
    echo "  ! 分项 $name：同一条流各行报的 ignored_points= / states_skipped_by_ignore= 不同（流：$item_ignore_conflict_flows），这一项两个数记 unknown；这两个数是那条流的测试代码预设的、各趟该相同，看这几条流各趟驱动行报的数为什么不一样（输出另存在 $item_output_file）" >&2
  fi
  if [[ "$item_domains" != - && "$item_domains" != unknown ]]; then
    item_domain_text="$(domains_of_the_item_for_the_success_line "$name" "$item_domains" "$item_progress")"
    IFS=$'\t' read -r item_domain_total item_domain_full item_domain_sliced item_domain_entries <<<"$item_domain_text"
    domain_items=$((domain_items + 1)); domain_segment_count=$((domain_segment_count + item_domain_total))
    domain_full_count=$((domain_full_count + item_domain_full)); domain_slice_count=$((domain_slice_count + item_domain_sliced))
    domain_entries="${domain_entries:+${domain_entries}、}${item_domain_entries}"
  fi
  if [[ "$item_registered_segments" != - ]]; then
    registered_items=$((registered_items + 1))
    item_registered_text="$(segments_of_the_item_for_the_success_line "$name" "$item_registered_segments")"
    registered_segment_count=$((registered_segment_count + ${item_registered_text%%$'\t'*}))
    registered_entries="${registered_entries:+${registered_entries}，}${item_registered_text#*$'\t'}"
  fi
  if [[ "$item_beyond_report" == unknown ]]; then
    beyond_unknown_items=$((beyond_unknown_items + 1)); beyond_unknown_names="${beyond_unknown_names:+${beyond_unknown_names}、}${name}"
  elif [[ "$item_beyond_report" != - ]]; then
    beyond_items=$((beyond_items + 1))
    item_beyond_text="$(segments_of_the_item_for_the_success_line "$name" "$item_beyond_report")"
    beyond_segment_count=$((beyond_segment_count + ${item_beyond_text%%$'\t'*}))
    beyond_entries="${beyond_entries:+${beyond_entries}，}${item_beyond_text#*$'\t'}"
  fi
  items_run=$((items_run + 1)); last_run_ran+=("$name")
  total_states=$((total_states + item_states)); total_red=$((total_red + item_red))
  if [[ "$item_ignored" == unknown || "$item_skipped" == unknown ]]; then
    items_ignore_unknown=$((items_ignore_unknown + 1))
    if [[ "$item_ignore_conflict_flows" != - ]]; then
      ignore_unknown_items="${ignore_unknown_items:+${ignore_unknown_items}、}${name}（同一条流各行报的数不同，流：${item_ignore_conflict_flows}）"; ignore_conflict_seen=1
    else
      ignore_unknown_items="${ignore_unknown_items:+${ignore_unknown_items}、}${name}"
    fi
  else
    total_ignored_points=$((total_ignored_points + item_ignored)); total_states_skipped_by_ignore=$((total_states_skipped_by_ignore + item_skipped))
    if (( item_ignored > 0 || item_skipped > 0 )); then ignored_items="${ignored_items:+${ignored_items}、}${name}"; fi
  fi
  if [[ "$item_reused" == unknown ]]; then
    items_reuse_unknown=$((items_reuse_unknown + 1)); reuse_unknown_items="${reuse_unknown_items:+${reuse_unknown_items}、}${name}"
  else
    total_reused=$((total_reused + item_reused))
  fi
  case "$verdict" in
    red)
      reason_text=""; for reason in "${reasons[@]}"; do reason_text="${reason_text:+${reason_text}；}${reason}"; done
      items_failed=$((items_failed + 1)); failed_names+=("$name"); failed_entries+=("$name：$reason_text") ;;
    not-run)
      items_not_run=$((items_not_run + 1)); not_run_entries+=("$name：$not_run_reason")
      not_run_items="${not_run_items:+${not_run_items}、}${name}（${not_run_reason}）" ;;
  esac
  [[ -n "$stopped_by_condition" ]] && break
done
# 上次红的清单（scripts/gate/lib_reuse_checks.py 文件头「三」，.claude/hooks/run-scope-guard.sh 的拒绝说明读它）：红的分项一行一个；点名了分项的只换跑到的那几项
last_run_red_file="$(mktemp)" && last_run_ran_file="$(mktemp)" && {
  printf '%s\n' ${failed_names[@]+"${failed_names[@]}"} >"$last_run_red_file"
  printf '%s\n' ${last_run_ran[@]+"${last_run_ran[@]}"} >"$last_run_ran_file"
  if (( ${#selected[@]} > 0 )); then
    python3 "$repository_root/scripts/gate/lib_reuse_checks.py" last-run scripts/crash/crash-amplification.sh "$last_run_red_file" --ran "$last_run_ran_file" >/dev/null
  else
    python3 "$repository_root/scripts/gate/lib_reuse_checks.py" last-run scripts/crash/crash-amplification.sh "$last_run_red_file" >/dev/null
  fi || echo "  ! 上次红的清单没写成（scripts/gate/lib_reuse_checks.py last-run），这一趟的判定不受影响" >&2
  rm -f -- "$last_run_red_file" "$last_run_ran_file"
}
echo "CRASH_AMPLIFICATION_TOTAL dispatched=$items_dispatched items=$items_run failed=$items_failed not_run=$items_not_run reuse_unknown=$items_reuse_unknown states=$total_states reused_known=$total_reused red=$total_red scope=$scope seconds=$(( $(date +%s) - started_at )) ignored_points=$total_ignored_points states_skipped_by_ignore=$total_states_skipped_by_ignore ignore_unknown=$items_ignore_unknown${narrowed_total_part}"
run_verdict=0
if (( items_run != items_dispatched )) && ! crash_amplification_break_is skip-collection-count; then
  echo "  ✗ 派出 $items_dispatched 项、收回 $items_run 项，对不上：这一趟的汇总不算数" >&2
  echo "  → 怎么办：上面哪一项只有「== 分项」那一行、没有 CRASH_AMPLIFICATION_ITEM 行，就是没收回的那一项；查主循环在打那一行之前为什么跳过或退出了它，修好之后整趟重跑同一条命令" >&2
  run_verdict=1
fi
if (( items_failed > 0 )); then
  for entry in "${failed_entries[@]}"; do
    echo "  ✗ 分项 $entry" >&2 # gate-lint:detail
  done
  echo "  ✗ $items_failed 个分项判红" >&2 # gate-lint:summary
  if (( ${#not_exhaustive_failed_names[@]} > 0 )); then
    not_exhaustive_failed_text="$(printf '%s、' "${not_exhaustive_failed_names[@]}")"
    echo "  → 不穷举的项（${not_exhaustive_failed_text%、}）：按段登记（.claude/gate.d/crash-point-replay-segments.tsv）或加深展开。展开上限盖不住、整域又展不开的段（几百写的段）登进那份表，那几段只按展开深度内的域判、成功句报段号与写数（登哪几段由用户点头）；盖得住的把引擎里放量的展开上限常量改到那个写数再整趟跑（登记在 .claude/gate.d/crash-knobs.tsv）；或查 ✗ 里点名的流是哪一段的写数超过展开上限（写表是不是改长了）" >&2
  fi
  if (( ${#engine_field_missing_failed_names[@]} > 0 )); then
    engine_field_missing_text="$(printf '%s、' "${engine_field_missing_failed_names[@]}")"
    echo "  → 登了段的行没报 unregistered_segments_beyond_expansion= 的项（${engine_field_missing_text%、}）：引擎没报 unregistered_segments_beyond_expansion=，重编引擎（驱动行要在 registered_segments= 之后报这条流里没登记、展开上限盖不住的段，crates/singlefs-verification/src/crash_amplification.rs 打驱动行那一处）；重编之后只重跑这几项：$0$(printf ' --item %s' "${engine_field_missing_failed_names[@]}")" >&2
  fi
  if (( ${#domain_field_missing_failed_names[@]} > 0 )); then
    domain_field_missing_text="$(printf '%s、' "${domain_field_missing_failed_names[@]}")"
    echo "  → 登了段的行没报全 registered_segment_domains= / registered_segment_progress= 的项（${domain_field_missing_text%、}）：引擎没报 registered_segment_domains= / registered_segment_progress=，重编引擎（驱动行要在 unregistered_segments_beyond_expansion= 之后报每个登记段的域 <段号>:<full|slice> 与每个分片段的 <段号>:<已判态数>/<域的总态数>，crates/singlefs-verification/src/crash_amplification.rs 打驱动行那一处）；重编之后只重跑这几项：$0$(printf ' --item %s' "${domain_field_missing_failed_names[@]}")" >&2
  fi
  if (( ${#slice_zero_failed_names[@]} > 0 )); then
    slice_zero_text="$(printf '%s、' "${slice_zero_failed_names[@]}")"
    echo "  → 分片段这一趟已判 0 态的项（${slice_zero_text%、}）：这一趟一个态都没判，看录入段与判段的日志（那一项的输出 $entry_run_directory/<分项>.log（错误警告在同名的 .problems.log）里录入段与判段那几趟）；修好之后只重跑这几项：$0$(printf ' --item %s' "${slice_zero_failed_names[@]}")" >&2
  fi
  echo "  → 怎么办：看那一项的错误警告日志（$entry_run_directory/<分项>.problems.log，行前带全量日志里的行号）、全量输出（另存在 $entry_run_directory/<分项>.log）与它驱动日志目录里的 placement.log / run.log（动用了第二台的是 share-0.log / share-1.log / import.log）；修好之后只重跑判红的那几项：$0$(printf ' --item %s' "${failed_names[@]}")" >&2
  run_verdict=1
fi
if [[ -n "$stopped_by_condition" ]]; then
  echo "  ✗ 分项 $stopped_by_condition 的驱动退 78（显卡用不上，或多机配置不满足）：这一趟停在这一项，后面 $(( ${#rows_to_run[@]} - items_dispatched )) 项没派" >&2
  echo "  → 怎么办：修好设备，或改配置：看那一项的错误警告日志（$entry_run_directory/$stopped_by_condition.problems.log）与全量输出（$entry_run_directory/$stopped_by_condition.log）里拒绝与出路那几行；修好之后重跑同一条命令（判过的块在库里，接着判）" >&2
  # 有红先判（文件头「一项怎么判」）：这一趟有红就退 1，退 78 的那一项照上面点名
  if (( ! run_verdict )) || crash_amplification_break_is condition-before-red; then exit 78; fi
  echo "  · 这一趟有红，退 1（有红先判）；退 78 的那一项照上面的出路修好设备，与判红的几项一起重跑" >&2
fi
if (( run_verdict )); then exit 1; fi
# 本次未跑的项逐项报（不是通过）；门禁起的时候往它的汇总里也记一笔
for entry in ${not_run_entries[@]+"${not_run_entries[@]}"}; do
  echo "  ! 本次未跑（这不是通过）：崩溃放量分项 $entry" >&2
  if [[ -n "${GATE_NOT_RUN_FILE:-}" ]]; then printf '崩溃放量分项 %s\n' "$entry" >> "$GATE_NOT_RUN_FILE"; fi
done
if (( items_not_run > 0 && items_not_run == items_run )); then
  echo "  · 本次未跑：派出的 $items_run 项都没判完（见上），这一趟不算跑过；重跑同一条命令接着判（判过的块在库里）" >&2
  exit 77
fi
case "$scope" in
  whole) if (( items_not_run > 0 )) || [[ -n "$crash_expand_up_to" ]]; then success_head="崩溃放量"; else success_head="崩溃放量全部项"; fi ;;
  *) success_head="崩溃放量点名的 $items_run 项" ;;
esac
success_head="${success_head}${narrowed_success_part}"
if (( items_not_run > 0 )); then
  run_part="跑完的 $(( items_run - items_not_run )) 项 0 红；本次未跑 $items_not_run 项：$not_run_items"
else
  run_part="$items_run 项都退 0、0 红"
fi
if (( items_reuse_unknown == 0 )); then
  reuse_part="复用 $total_reused 态"
elif (( items_reuse_unknown == items_run )); then
  reuse_part="复用数取不出（$items_reuse_unknown 项：${reuse_unknown_items}，驱动行没报 states_reused=）"
else
  reuse_part="复用 $total_reused 态（只算取得出的 $(( items_run - items_reuse_unknown )) 项；另有 $items_reuse_unknown 项取不出：${reuse_unknown_items}，驱动行没报 states_reused=）"
fi
# 按登记整批不判的一段（文件头「按登记整批不判」）
ignore_classes=""
if (( total_ignored_points > 0 || total_states_skipped_by_ignore > 0 )); then
  ignore_classes="（类名取不出：驱动行只报点数与态数、不报类名；有跳过的分项：$ignored_items）"
fi
ignore_unknown_reason="驱动行没报 ignored_points= / states_skipped_by_ignore="
if (( ignore_conflict_seen )); then ignore_unknown_reason="驱动行没报 ignored_points= / states_skipped_by_ignore=，或同一条流各行报的数不同（括号里点名流）"; fi
if (( items_ignore_unknown == 0 )); then
  ignore_part="按登记整批不判：$total_ignored_points 个点、$total_states_skipped_by_ignore 态$ignore_classes"
elif (( items_ignore_unknown == items_run )); then
  ignore_part="按登记整批不判：取不出（$items_ignore_unknown 项：${ignore_unknown_items}，${ignore_unknown_reason}）"
else
  ignore_part="按登记整批不判：取得出的 $(( items_run - items_ignore_unknown )) 项 $total_ignored_points 个点、$total_states_skipped_by_ignore 态$ignore_classes；另有 $items_ignore_unknown 项取不出：${ignore_unknown_items}，${ignore_unknown_reason}"
fi
crash_amplification_break_is skip-ignore-report && ignore_part=""
# 按段登记的一段（文件头「按段登记」）：登了段的项逐项、逐条流报段号与写数（那几段只按展开深度内的域判，全量展开的态数不报）
if (( registered_segment_count > 0 )); then
  registered_part="按段登记 $registered_segment_count 段（$registered_items 项，只按展开深度内的域判、没判全域）：$registered_entries"
else
  registered_part="按段登记：0 段"
fi
crash_amplification_break_is registered-segments-unreported && registered_part=""
# 登记段的域的一段（文件头「按段登记」）：逐段报全展开，或分片、这一趟为止已判几态 / 域的总态数（两个数都是十进制数字串，原样打）
if (( domain_segment_count > 0 )); then
  domain_part="登记段的域$domain_segment_count 段（$domain_items 项；全展开 $domain_full_count 段、分片 $domain_slice_count 段）：$domain_entries"
else
  domain_part="登记段的域：0 段"
fi
crash_amplification_break_is slice-domains-unreported && domain_part=""
# 展开上限盖不住又没登记的段的一段（文件头「按段登记」）：逐项、逐条流报段号与写数，ITEM 行写 unknown 的项点名取不出
if (( beyond_segment_count > 0 )); then
  beyond_part="展开上限盖不住又没登记的段 $beyond_segment_count 段（$beyond_items 项，没判全域）：$beyond_entries"
else
  beyond_part="展开上限盖不住又没登记的段：0 段"
fi
if (( beyond_unknown_items > 0 )); then
  beyond_part="${beyond_part}（另有 $beyond_unknown_items 项取不出：${beyond_unknown_names}，驱动行没报 unregistered_segments_beyond_expansion=）"
fi
crash_amplification_break_is beyond-expansion-unreported && beyond_part=""
echo "  ✓ $success_head：$run_part；现判 $total_states 态、$reuse_part；派出 $items_dispatched 项、收回 $items_run 项${ignore_part:+；$ignore_part}${registered_part:+；$registered_part}${domain_part:+；$domain_part}${beyond_part:+；$beyond_part}"
exit 0
