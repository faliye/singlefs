#!/usr/bin/env bash
# admission: always 每次调都在这一刻的树上跑全量或点名的分项；哪些块要重判由库里的块键与判法单元指纹定，不在这里
# run-condition: command bash cargo python3
# run-condition: check source "$PREFLIGHT_SCRIPT_DIRECTORY/crash-amplification-lib.sh" && crash_amplification_run_condition_holds "$PREFLIGHT_SCRIPT_DIRECTORY/../.." :: 修好设备，或改配置：多机配置 multi-host.env 开了 GPU 而一份卡都没写、点名的卡与双机开关对不上；单跑 bash -c 'source scripts/crash/crash-amplification-lib.sh && crash_amplification_run_condition_holds' 看是哪一处
# 长活声明（判法 .claude/hooks/lib_long_run.py：scripts/hosts/detached-run.sh start --wall-clock over-2h 与 long-run-check、scripts/hosts/host-dispatch.py 文件头 ⓪ 都读这几行）：
# long-run-shards: 按登记表 ITEMS 的分项逐项跑；多机配置开着双机时，每一项由 crash-amplification-two-hosts.sh 把它的状态块按两台量到的速度分成两份（SINGLEFS_CRASH_AMPLIFICATION_SHARE=0/2、1/2），第二台那一份取回、导入本机库并成一份
# long-run-resume: 一份库装全部分项（${SINGLEFS_CRASH_AMPLIFICATION_HOME:-~/.local/share/singlefs/crash-amplification}/crash-state-verdicts，在 /tmp 之外）；重起同一条命令就是续跑：同一把判法单元指纹下判过的块全部复用、只判没判过的，第二台判到一半被停也先导入再退
# long-run-unshardable: 多机：台数只认多机配置的双机开关（至多本机与 PEER_SSH_HOST 两台，不读 crash-point-replay-shard-configuration-check.sh --emit-hosts 的清单）；跑的那一步线程取 --threads 或本机核数减一、内存上限取 --memory-cap 与 --peer-memory-cap（本机默认 38G），只有本机只编那一步与第二台那两趟的线程按 host-resources.py --json 现算
# 崩溃放量全量（用户 2026-09-29 定：其他的崩溃放量放在一个脚本里做、作为分项；以后全量就是一起跑，分项用参数跑；不指定参数就作为全量跑）：
#
#   crash-amplification.sh                                  全量：登记表里每个分项按次序跑一遍，展开深度用 FULL_EXPAND_UP_TO（见「全量的展开深度」）
#   crash-amplification.sh --item <分项> [--item <分项>]…    只跑点名的分项，给几次取并集（分项名照 --list-items 列的）；展开深度同全量，成功句不说全量
#   crash-amplification.sh --expand-up-to <N> [--item …]    缩过规模的一趟：每项按展开 N 跑（N 与 FULL_EXPAND_UP_TO 相同就是没缩）；成功句写「缩过规模：展开 N」、不说全量，并点名哪几项不穷举
#   crash-amplification.sh --list-items                     逐行列出可点名的分项：分项名、测试目标、用例函数、档位（TAB 隔开）；不清场、不跑
#   crash-amplification.sh --selftest                       用假的分项驱动与假的清场命令走一遍第 1–48 格（复用三问与上次红的清单是第 47、48 格；列项、点名、拒绝、清场、各种判红、本次未跑、复用数、派出收回、
#                                                           只供自证的变量、有红又有项退 78、按登记整批不判的数、驱动行的 red= 与全量的零态、复用数每项只取一次、两台分份的 flow= 行留的块、
#                                                           分项驱动退 77 与它报的本次未跑、几条流里有的没借到、不带参数的全量里一项不穷举红在那一项、
#                                                           只因登记的段不穷举不红而报段号与写数、没登段的流照旧红、往驱动送按段登记文件、登记文件写坏拒跑、
#                                                           登了段的流里另有展开上限盖不住又没登记的段红在那条流与那几段、驱动行没报那个字段照不穷举红、
#                                                           登记段分片枚举不红而报全展开与分片已判几态、分片段已判 0 态而那一行现判与复用都是 0 红在那条流与那一段、全部复用的一趟不红、分项驱动报判段没跑（step=judge）记本次未跑、
#                                                           登了段的行没报全或写坏登记段的域与分片进度照重编引擎红），
#                                                           末尾一轮从下面「弄坏开关」那张表现读开关、逐个打开，核它红在点名的那几格
#   其余参数原样交给每一项的分项驱动：[--threads <N>] [--memory-cap <上限>] [--peer-memory-cap <上限>] [--library <库目录>] [--label <标签>]
#
# 崩溃点重放的合表（.claude/gate.d/crash-point-replay-items.tsv，入口 scripts/crash/crash-point-replay.sh）一项一条流，每项点名它在这里的分项（第三列）：
#   ITEMS 里每一项都要在合表里恰好被一项点名（合表那一份的判法 scripts/crash/lib_crash_point_replay_items.py 核），入口照合表逐项调这里的 --item。
# 分项登记在 ITEMS：一行「分项名 <TAB> 测试目标 <TAB> 用例函数 <TAB> 档位」，一个分项就是 crates/singlefs-checker-tier/tests/ 下一条走
# crash_amplification::run_from_environment 的用例（用例函数经分项驱动的 --test-filter 点名，cargo test 的 --exact）。档位今天只有「全量」：
# 每一项都是穷举、要 checker 档的重活，只在全量里跑，不在快档（--list-items 写成「全量（不在快档）」）。
# 崩溃注入（crash_injection_campaign）不在表里：它是随机历史上的抽样、没有固定的点序列与树，是轻测，归普通 cargo test。
# 一份库装全部分项（默认 ${SINGLEFS_CRASH_AMPLIFICATION_HOME:-~/.local/share/singlefs/crash-amplification}/crash-state-verdicts，在 /tmp 之外；
# 目录名与「块键带名字的旧库引擎开库就拒、这里不删不读」见 scripts/crash/crash-amplification-lib.sh 文件头「判定库」）：
# 几条流共前缀，同一份库里前缀节点只判一次；重跑同一条命令就是续跑——同一把判法单元指纹下判过的块全部复用，判法改了旧行留作历史、这一把从头判。
# 每项交 scripts/crash/crash-amplification-item.sh 跑：配置几台就跑几台、配置开了 GPU 就由 GPU 判，做不到那一项报错
# （用户 2026-09-29 定：「配置中一台机器就要跑一台，配置中两台机器就要跑两台，配置中有gpu就一定要让GPU工作 如果不行就明确报错拒绝，不接受默认降级」）。
# 运行条件：配置上判得了的（多机配置开了 GPU 却一份卡都没写、点名的卡与双机开关对不上）由文件头 run-condition: check 在开跑之前判（同分项驱动，判法在
#   crash-amplification-lib.sh 的 crash_amplification_run_condition_holds）；跑起来才知道的退 78：开跑前清场回读没过或多机配置没写全清场那两个键、某一项的驱动退 78
#   （显卡或第二台用不上）。出路都是「修好设备，或改配置」。
# 清场（用户 2026-09-29 定「每次一定全量要清场」）：选好分项之后经 scripts/crash/gpu-quiesce-run.sh 重起自己（同一组参数）：照多机配置清场并回读
# （回读没过退 78、一项都不跑；卡上有不归清场白名单的计算进程时就是这样，等它跑完再起，这里不停它），每一项都带 --quiesced 跑（显卡用清场之后那一份先后与额度），
# 整趟跑完、跑红或被打断都不复原（那份包装打 GPU_QUIESCE step=restore result=not-restored；要复原由用户手动跑私有复原脚本，多机配置的 QUIESCE_START_COMMAND）。
# 重起的那一趟认包装交下来的 GPU_QUIESCE_HOLDER（文件在、文件名以父进程号加「-」起头），不再清。
#
# 全量的展开深度：只登记在一处，FULL_EXPAND_UP_TO（第一行代码之后那个赋值）。不带 --expand-up-to 就把它交给每一项的驱动；
#   驱动行（CRASH_AMPLIFICATION）报的 expand_up_to 与交下去的不同判红；展开深度是它时每一行都要 exhaustive=true，或 exhaustive=false 只因登记的段
#   （这一行 registered_segments= 不是 -、unregistered_segments_beyond_expansion= 是 -，见「按段登记」）；有一行两样都不是
#   （exhaustive=false 而 registered_segments= 是 -、没报或写得认不得，或 exhaustive= 没报）这一项判红、整趟退 1（全量的完成标记不认缩过规模的那一趟），
#   理由里点名不穷举又没登段的流，出路是「按段登记（.claude/gate.d/crash-point-replay-segments.tsv）或加深展开」；登了段的行另两种判红见「按段登记」。
#   给了 --expand-up-to 而值不同的是缩过规模的一趟：exhaustive=false 不判红，ITEM 行照实写出，成功句点名哪几项不穷举（见「输出」的成功句）。
#   取值：登记分项里最长一段的写数。今天是 30 写：多记录流与删除流（multi-record-publish、unlink-after-the-multi-record-publish）共有的第 16 段，
#   出处是第 32 批 Q 探针真跑出来的段形状 PROBE_SEGMENT_SIZES [2,2,1,2,2,1,2,24,2,1,2,28,4,1,2,30,6,1,2,26,2,1,2,24,2,1,2,22,2,1,2,20,2,1,2]
#   （删除流 35 段，前 19 段与多记录流逐段相同；records/2026-10-01-删除与并发事务.md 2026-10-08「三·5 量数」那一条）。展开 28 时这两项驱动行报 exhaustive=false，
#   2026-10-08 展开 30 跑这两项，驱动行都报 exhaustive=true expand_up_to=30。别的分项没有段形状的记录；一段不超过展开上限就展开全部真子集，
#   所以从 28 抬到 30 只改写数是 29、30 的段，不超过 28 写的段展开不变。
#   这里不现算：驱动行只报 exhaustive= 与 expand_up_to=、不报段的写数（crates/singlefs-checker-tier/src/crash_amplification.rs 的
#   reuse_and_scale_fields_of_the_line），段形状要编、跑 checker 档的用例才拿得到。新加的流或改了写表而出现更长的段，这里不会自己跟上：
#   不带参数的全量会在那一项判红（上面那一条），照出路把这个值改到新的最长段，或把整域展不开的段按段登记。
#
# 按段登记（判决 research/prompts/m3-crash-enumeration-merge-r3-main-verification.md 第三节 J10、H10）：表是 .claude/gate.d/crash-point-replay-segments.tsv
#   （写法在它的表头，核法 scripts/crash/lib_crash_point_replay_items.py 的 segment_problems）。选好分项之后、清场之前先核被判仓根下那一份，核不过退 2、
#   一项都不跑；核过了 export SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY=<被判仓根>/.claude/gate.d/crash-point-replay-segments.tsv（绝对路径，与
#   scripts/crash/crash-point-replay.sh 送的同一个文件），分项驱动与双机驱动从环境继承（第二台那几趟由 crash-amplification-two-hosts.sh 送快照）。
#   引擎对登记了的段按甲二 ∪ 展开 6 展开，驱动行报 exhaustive=false registered_segments=<段号>:<写数>[,…]（没登段写 -），之后报
#   unregistered_segments_beyond_expansion=<段号>:<写数>[,…]（这条流里没登记、展开上限又盖不住的段；一段都没有写 -；引擎的 exhaustive 在登了段时一律是 false）。
#   一行 exhaustive=false 而 registered_segments= 是那种写法的（登了段的行），展开深度是全量的那一个时按 unregistered_segments_beyond_expansion= 判：
#     是 -：不因不穷举判红；
#     是 <段号>:<写数>[,…]：判红，理由点名那条流与那几段（「流 <流> 段 <段号>:<写数>、…」），出路「按段登记（.claude/gate.d/crash-point-replay-segments.tsv）或加深展开」；
#     没报或写得认不得（旧引擎打的行）：照不穷举判红，理由点名那条流，出路「引擎没报 unregistered_segments_beyond_expansion=，重编引擎」。
#   按行判，同一项别的行照判。
#   分片枚举（用户 2026-10-13 定：登记段的「甲二 ∪ 展开 6」装不进 u64 时按固定次序分片枚举，每趟判 2^32 个态、判定进库，下一趟从库里接着判）：
#   引擎在 unregistered_segments_beyond_expansion= 之后报 registered_segment_domains=<段号>:<full|slice>[,…]（登记段的域是全展开还是分片；一段都没登写 -）
#   与 registered_segment_progress=<段号>:<已判态数>/<域的总态数>[,…]（只列 slice 段，没有写 -；两个数都是精确的十进制数字串，引擎的态数是 768 位宽整数
#   （用户 2026-10-13 定，6 个 u128 拼），可能到 140 位：这里只按 ^[0-9]+$ 认、不转成数、不做数值比较，成功句原样打这两个串）。登了段的行，不看展开深度：
#     两个字段缺一个（旧引擎）、registered_segments= 列的段有一段不在 registered_segment_domains= 里、slice 段没在 registered_segment_progress= 里、
#     进度里列了不是 slice 的段、或写法认不得：判红，理由点名那条流，出路「引擎没报 registered_segment_domains= / registered_segment_progress=，重编引擎」；
#     slice 段已判态数字面全是 0、而这一行的现判（states_checked_here=）与复用（states_reused=，没报照 0 算）都不是正整数：判红，理由点名那条流与那一段（「流 <流> 段 <段号>:0/<总>」），
#     出路「这一趟一个态都没判，看录入段与判段的日志」；现判或复用有一个是正整数的（全部复用的一趟就是现判 0、复用大于 0）不因它判红；
#     其余不因分片判红（分片的段没判全域，照登记段只按甲二 ∪ 展开 6 判那样，不算不穷举）。
#   ITEM 行写 registered_segments=：每段一条「<流>@<段号>:<写数>」（没带 flow= 的行只写「<段号>:<写数>」），逗号隔开、同一条流几行报的按段去重，一段都没有写 -；
#   之后写 unregistered_segments_beyond_expansion=：同一种写法，取各行（登没登段都算）报的这个字段；一段都没有而有一行没报这个字段（或写得认不得）写 unknown，否则写 -。
#   再写 registered_segment_domains= 与 registered_segment_progress=：取登了段的行报的这两个字段原样，每段前面加「<流>@」（没带 flow= 的行不加），同一条流几行报的按段去重；
#   一段都没有而有登了段的行没报全或写坏（上面判红的那种）写 unknown，否则写 -。
#   成功句末尾接「按段登记 N 段（K 项，只按甲二 ∪ 展开 6 判、没判全域）：<分项> 流 <流> 段 <段号>（<写数> 写）、…，…」，一段都没有接「按段登记：0 段」；
#   再接「登记段的域（甲二 ∪ 展开 6）N 段（K 项；全展开 F 段、分片 S 段）：<分项> 流 <流> 段 <段号>：全展开、<分项> 流 <流> 段 <段号>：分片，已判 <已判>/<总>、…」
#   （一段都没有接「登记段的域：0 段」）；
#   再接「展开上限盖不住又没登记的段 N 段（K 项，没判全域）：<分项> 流 <流> 段 <段号>（<写数> 写）、…」，一段都没有接「展开上限盖不住又没登记的段：0 段」；
#   有项 ITEM 行写 unknown 的再接「（另有 J 项取不出：<分项、…>，驱动行没报 unregistered_segments_beyond_expansion=）」。展开深度是全量的那一个时
#   这种段都已判红（登了段的照上面，没登段的照不穷举），到不了成功句；成功句里报得出段来的只有缩过规模的一趟。
#   全量展开那几段要的态数不报（H10）。
#
# 输出：每项的驱动输出边跑边转打，同时 tee 进这一项自己的文件
#   ${SINGLEFS_CRASH_AMPLIFICATION_HOME:-~/.local/share/singlefs/crash-amplification}/runs/entry-<时刻>-<进程号>/<分项>.log，跑完从那份文件取汇总，
#   打一行 CRASH_AMPLIFICATION_ITEM item=<分项> exit=<驱动退出码> states=<现判：Σ states_checked_here> reused=<复用：这一项只取一次，取法见「复用数怎么取」，取不出写 unknown>
#     red=<Σ red> lines=<CRASH_AMPLIFICATION 行数> expand_up_to=<各行报的展开深度，各行不一或有行没报写 unknown>
#     exhaustive=<每一行都是 exhaustive=true 才写 true> verdict=<green|red|not-run> seconds=<这一项的挂钟>
#     ignored_points=<按流分组、每组只取一次的 ignored_points 之和> states_skipped_by_ignore=<同样取法的 states_skipped_by_ignore 之和>（有一行没报这两个数或不是非负整数、或同一组各行报的数不同，两个都写 unknown；见「按登记整批不判」）
#     registered_segments=<按段登记的段，写法见「按段登记」；没有写 ->
#     unregistered_segments_beyond_expansion=<展开上限盖不住又没登记的段，写法见「按段登记」；没有写 -，取不出写 unknown>
#     registered_segment_domains=<登记段的域> registered_segment_progress=<分片段的进度>（写法见「按段登记」；没有写 -，取不出写 unknown）。
#   末尾一行 CRASH_AMPLIFICATION_TOTAL dispatched=<派出几项> items=<收回几项> failed=<几项红> not_run=<几项本次未跑> reuse_unknown=<几项取不出复用数>
#     states=<Σ 现判> reused_known=<Σ 取得出的那几项的复用> red=<Σ red> expand_up_to=<交给驱动的展开深度> scope=<full|items|scaled> seconds=<挂钟>
#     ignored_points=<Σ 取得出的那几项> states_skipped_by_ignore=<Σ 取得出的那几项> ignore_unknown=<几项取不出这两个数（没报、写坏或同一组各行不同）>。
#   scope：full 没点名、展开深度是 FULL_EXPAND_UP_TO；items 点了名、展开深度是它；scaled 展开深度不是它（点没点名都算）。
#   退 0 时再打一句成功句（✓）：开头「崩溃放量全量（展开 N）」（只有 scope=full 且没有一项本次未跑才这样写）、「崩溃放量（展开 N）」（scope=full、
#     有项本次未跑）、「崩溃放量点名的 K 项（展开 N）」、「崩溃放量缩过规模：展开 N」；接「K 项都退 0、0 红」，有项本次未跑时接
#     「跑完的 K 项 0 红；本次未跑 J 项：<分项（原因）、…>」；scope=scaled 时接着写穷举那一段：有驱动行的项里有不是 exhaustive=true 的，写
#     「不穷举 J 项：<分项、…>（展开 N 盖不住它们最长的一段，这几项没判完全域）」，都是的写「有驱动行的 K 项都穷举」；
#     再接「现判 N 态、<复用那一段>；派出 D 项、收回 D 项」；再接「按登记整批不判」那一段（见下），末尾接「按段登记」那三段（按段登记的段、登记段的域、展开上限盖不住又没登记的段，见上）。
# 一项怎么判（verdict）：
#   red（计进 failed，整趟退 1），有一条就是：驱动退非 0（退 78 另见下；退 248 是算法上判不了，照这一条判红、清单里写明）；red 之和大于 0（驱动退几都算，退 78 也算）；
#     有一行缺 red= 或 red 的值不是非负整数（驱动行的约定是每行带 red=<非负整数>；驱动退几都算）；
#     驱动退 0 却 0 行 CRASH_AMPLIFICATION（出路：看测试名还在不在、cargo 是不是筛出了 0 条）；展开深度是 FULL_EXPAND_UP_TO 而有一行不是 exhaustive=true、
#     又不是只因登记的段不穷举（出路：按段登记或加深展开；见「全量的展开深度」「按段登记」）；展开深度是它而登了段的行报
#     unregistered_segments_beyond_expansion= 不是 -（出路：按段登记或加深展开）或没报这个字段（出路：重编引擎；见「按段登记」）；
#     登了段的行（不看展开深度）registered_segment_domains= / registered_segment_progress= 没报全、漏段或写法认不得（出路：重编引擎），
#     或分片段已判 0 态而那一行现判与复用都是 0（出路：看录入段与判段的日志；见「按段登记」）；
#     有一行报的 expand_up_to 与交下去的不同（驱动没照给的展开深度跑）；scope=full（不带参数的全量）里一项现判 0 态、复用 0 态
#     （叫停或留了没判的块的项除外，那种照 not-run）。
#   not-run（本次未跑：不计进 failed，也不算通过，照 .claude/singlefs-ai-sop/rules/show-me-test.md「门禁不许假装通过」逐项报；环境里有 GATE_NOT_RUN_FILE
#     时每项往里追加一行）：不红，而有一行带 stopped_early=true、或有一行 unjudged_blocks>0。两台分份跑的行留着对方那一份没判是本来如此，
#     同一组（按 flow= 分，没有 flow= 的归一组，同「复用数怎么取」）里有导入行时不算——这一组没判完的块由导入行报；组里没有导入行，分份行留的块照算。
#     分份行：share=<i>/<n>、n>1 的行，与不带 share= 而 mode=two_hosts_* 的 flow= 行（tests/common_crash_points/mod.rs 的 amplify_prepared_flow 打的，
#     两台各领一份时它的 unjudged_blocks 含对方那一份）；导入行（行尾 import_run=true，或带 imported_blocks>0）不是分份行，它留的块照算。
#     本机先判着、被有意叫停的那一行（行尾 warm_up=true，双机驱动加的）也不算。
#     分项驱动退 77（一项都没跑到：排派活计划那一趟借不到显存、一块都没判，或第二台排计划那一趟借不到显存；scripts/crash/crash-amplification-item.sh 文件头「退出码」）
#     也是 not-run，不按「驱动退非 0」判红；原因写它打的 CRASH_AMPLIFICATION_NOT_RUN 行的 reason=（没打那一行的照写没打）。
#     驱动退 0 而打了 CRASH_AMPLIFICATION_NOT_RUN 行（一条用例几条流，没借到显存的那几条各一行，借到的照判）同样是 not-run，原因写几条流没判与第一行的 reason=：
#     这一项不算通过。驱动退 0 而打了 CRASH_AMPLIFICATION_NOT_RUN step=judge 行（配置写了 ENABLE_GPU=1 而二进制没编进 gpu 特性：判段这一趟不跑、录入段照录，
#     分项驱动或双机驱动转打的那几行，一条流一行）也是这一种，原因写几条流判段没跑与第一行 step=judge 的 reason=。这几种都不按「驱动退 0 却 0 行」判红，全量里也不按「现判 0 态、复用 0 态」判红；行里报了红照样记 red。
#     派出的项全是 not-run 时整趟退 77。
#   驱动退 78（显卡或第二台用不上、清场或多机配置不满足）：行里没报红（red 之和为 0、每行的 red= 都对）就记 exit=78 verdict=not-run；
#     行里报了红记 exit=78 verdict=red（判出来的红不让 78 盖掉）。两种都停在这一项、后面的项不再派。
#     整趟有红先判：别的项或这一项红就退 1，没有红才退 78；退 78 的那一项都在 ✗ 里点名。
#   green：别的。
# 复用数怎么取：登记表里每一项都走 run_from_environment 的判定库，都有判定复用这一层，没有哪一项写「不复用」。一项的复用数只取一次
#   （.claude/rules/test-script-discipline.md「判定可复用：按输入存，按前缀剪」：成功句报「现判 N、复用 M」）：
#   这一项开跑时库里这一判法单元指纹下已判、这一趟因此没判的态数。现判是各行 states_checked_here 之和（每一趟判的块不相交，照加）。
#   驱动行的 states_reused= 是那一趟（一次 run_from_environment）录入时库里已判的，叫停、导入、只排计划的那一趟都照实报
#   （crates/singlefs-checker-tier/src/crash_amplification.rs 的 reuse_and_scale_fields_of_the_line 的文档），同一项里几趟的行相加会重复计，所以按流、按行的种类取：
#   - 按流分组：行里有 flow=<名> 的按名分，没有的（fixed-script 两项的汇总行）归一组。一组只取一次：
#   - 组里有先判着的行（行尾 warm_up=true，双机驱动转打时加的）：只取组里第一行先判着的行；之后的分份行与导入行录入时，
#     先判着那一趟判过、复用过的都已在库里，它们的复用数不取。
#   - 组里没有先判着的行：取导入行（行尾 import_run=true，双机驱动转打时加的）之外各行之和——本机一台跑的那一行、被叫停的那一行，
#     或两台各领一份的两行（两份的块不相交）；导入行的复用数是导入之前本机那一份已判的，不取。组里只有导入行时这一组取不出。
#   - CRASH_AMPLIFICATION_PLACEMENT 行（只排派活计划那一趟，分项驱动与双机驱动原样转打）也带 states_reused=，它不是 CRASH_AMPLIFICATION 行，不取。
#   一行的复用数依次这样取：
#   ① 行里有 states_reused=<n> 就取它；
#   ② 本机一台跑完的整趟（share=<i>/1、带 states=、没有 stopped_early=true、imported_blocks 为 0 或没有）取 states − states_checked_here：
#      fixed-script 两项的用例打这一行之前断言过「录的 + 库里这一判法单元指纹下已判的 = states」与（一台时）「这一趟核的 = 录的」，差就是这一趟复用的状态数
#      （crates/singlefs-checker-tier/tests/crash_amplification_fixed_script_through_rollback.rs、…_through_unmount.rs 打汇总行之前那几条 assert_eq）；
#   ③ 别的行取不出：flow= 那种行（tests/common_crash_points/mod.rs 的 amplify_prepared_flow）不带 states=；两台跑的三行（share=0/2、1/2 与带
#      imported_blocks 的导入那一趟）里 states − states_checked_here 混着对方那一份、导入的块与复用的块，分不开；被叫停的行还有没判的块。
#      导入的块（imported_blocks）是第二台这一趟判的，不算复用。
#   要取的行里有一行取不出，这一项记 reused=unknown、计进 TOTAL 行的 reuse_unknown，不记 failed、不改退出码；TOTAL 行的 reused_known 只加取得出的项。
#   成功句：都取得出写「复用 M 态」；全都取不出写「复用数取不出（J 项：<分项…>，驱动行没报 states_reused=）」；一部分取不出写
#   「复用 M 态（只算取得出的 K 项；另有 J 项取不出：<分项…>，驱动行没报 states_reused=）」。有取不出的项时不写「复用 0 态」，也不写「不复用」。
# 按登记整批不判（.claude/rules/test-script-discipline.md「判定可复用：按输入存，按前缀剪」：这一类不算剪枝，成功句报跳过了多少、哪几类）：
#   驱动行的约定是每行带 ignored_points=<测试代码里预设 ignore 的点数> states_skipped_by_ignore=<这些点的段因此没展开的态数>。这两个数是那条流的测试代码预设的，
#   不随一趟变：同一条流的本机份、第二台份、导入、叫停再续的各行报的是同一份，所以和复用数一样按流分组、每组只取一次（不是各行之和）：
#   - 按流分组：行里有 flow=<名> 的按名分，没有的归一组（同「复用数怎么取」）；
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
#   2   用法错：参数认不得、--item 后面缺项名或项名认不得（这两种列出可点名的项）、--expand-up-to 不是正整数、登记表坏、分项驱动不在、
#       平常跑时带了只给自证用的变量（见「只供自证」）、按段登记文件核不过（见「按段登记」；逐处列出问题，不清场、一项都不跑）
#   77  派出的项一项都没判完（全是 not-run），也没有红
#   78  运行条件不满足（出路：修好设备，或改配置）：preflight 判的（文件头两条 run-condition；含开了 GPU 而多机配置里一份卡都没写）、
#       开跑前清场回读没过或多机配置没写全清场那两个键（scripts/crash/gpu-quiesce-run.sh 判的，它打 GPU_QUIESCE step=command result=not-started）、
#       建不了放每项输出的目录、某一项的驱动退 78（这一趟没有红时）
#   preflight 自己判不了条件（声明写坏）时退 1，见 .claude/singlefs-ai-sop/rules/preflight-discipline.md「不满足时」。没有别的退出码。
# 弄坏开关 CRASH_AMPLIFICATION_BREAK=<项>（几个用逗号连；只在自证起的子进程里认）。逐项写开关名、弄坏哪一处、打开后 --selftest 哪几格该红
#   （格号是 run_selftest 里的第 1–48 格，判错的那一格打「✗ 自证第 <号> 格：<失败行>」）。
# 写法（B 写法，样板是 scripts/mutation/mutate.sh 文件头「弄坏开关」一段）：一行「#   CRASH_AMPLIFICATION_BREAK=<开关名>  <弄坏哪一处>」，下一行起以「#       」开头
#   写点名的格「第 N 格」（、隔开；括号里是那一格失败行的句首，开关轮不读，括号外的每一段都算一格）。--selftest 末尾那一轮（开关轮）从这张表现读
#   （crash-amplification-lib.sh 的 crash_amplification_read_switch_table，与双机驱动同一份读法）：表里 CRASH_AMPLIFICATION_BREAK 的开关与代码里认的开关要相等
#   （表里另列清场包装 scripts/crash/gpu-quiesce-run.sh 的 GPU_QUIESCE_BREAK 开关，它们在那份包装里认，经重起的那一趟带进去），
#   每个逐个打开、要退 1、红的格与点名的逐格相等：
#   CRASH_AMPLIFICATION_BREAK=swallow-exit  分项驱动退非 0 也记 0
#       第 8 格（「驱动退非 0 没记成红」）、第 20 格（「驱动退 78 却没停下退 78」）、第 22 格（「有红又有一项退 78 时没退 1」）、
#       第 23 格（「驱动退 78 而行里报了红，这一项没记红」）、第 30 格（「跑完调了复原命令、或有项红时没退 1」）、
#       第 31 格（「分项驱动退 77 没记进本次未跑」）、第 48 格（「上次红的清单不对」：红的那一项没记成红）
#   CRASH_AMPLIFICATION_BREAK=skip-filter  调驱动时不带 --test-filter
#       第 4 格（「全量没把每个分项各调一次驱动」）
#   CRASH_AMPLIFICATION_BREAK=accept-unknown-item  不认得的分项名照跑
#       第 2 格（「不认得的分项名没有退 2 并列出可点名的项」）
#   CRASH_AMPLIFICATION_BREAK=no-quiesce  开跑前不经清场包装重起、每项不带 --quiesced
#       第 6 格（「没有照「开跑前清一次、每项带 --quiesced、跑完不复原」走」）、第 7 格（「清场没清成却没退 78」）、
#       第 30 格（「跑完调了复原命令、或有项红时没退 1」）
#   GPU_QUIESCE_BREAK=run-despite-failed-quiesce  清场包装清场回读没过也起命令
#       第 7 格（「清场没清成却没退 78」）
#   GPU_QUIESCE_BREAK=restore-after-run  清场包装出去时又照多机配置复原
#       第 6 格（「没有照「开跑前清一次、每项带 --quiesced、跑完不复原」走」）、第 30 格（「跑完调了复原命令、或有项红时没退 1」）
#   CRASH_AMPLIFICATION_BREAK=reuse-as-zero  取得出的复用数也记 0
#       第 10 格（「复用数取错」）、第 28 格（「复用数没有每项只取一次」）、第 29 格（「两台分份的 flow= 行留的块没照「同一组有导入行才不算」判」）、
#       第 45 格（「全部复用的一趟判了红、或 ITEM 行没照实写现判 3 复用 7」：ITEM 行复用写成 0）、第 47 格（「复用三问没过」：键相同的一趟 ITEM 行写复用 0，折成现判）
#   CRASH_AMPLIFICATION_BREAK=accept-unknown-reuse  取不出复用数的项记成 reused=0、不计进 reuse_unknown、成功句不点名
#       第 11 格（「取不出复用数的项没照实报出」）、第 12 格（「两台跑的行被拿来推复用数」）
#   CRASH_AMPLIFICATION_BREAK=reuse-across-shares  两台分份行、导入行、叫停的行也拿 states 减 states_checked_here 当复用数
#       第 12 格（「两台跑的行被拿来推复用数」）
#   CRASH_AMPLIFICATION_BREAK=skip-collection-count  不核派出收回
#       第 13 格（「少收回一项却没判红」）
#   CRASH_AMPLIFICATION_BREAK=ignore-red-count  不看 red 之和（驱动退 0 与退 78 都不看）
#       第 9 格（「驱动退 0 而 red=2 没记成红」）、第 22 格（「有红又有一项退 78 时没退 1」）、第 23 格（「驱动退 78 而行里报了红，这一项没记红」）
#   CRASH_AMPLIFICATION_BREAK=accept-zero-lines  驱动退 0、0 行 CRASH_AMPLIFICATION 不判红
#       第 14 格（「驱动退 0、0 行却没判红」）
#   CRASH_AMPLIFICATION_BREAK=accept-not-exhaustive  展开深度是全量的那一个、有行不是穷举也不判红
#       第 15 格（「全量的展开深度下有行不是穷举却没判红」）、第 33 格（「不带参数的全量里一项不穷举，没有红在那一项」）、
#       第 35 格（「一行 exhaustive=false registered_segments=- 没照旧判红在那一条流」）、
#       第 38 格（「登了段的流里另有展开上限盖不住又没登记的段，没判红在那条流与那几段」）、
#       第 39 格（「登了段的行没报 unregistered_segments_beyond_expansion=，没照不穷举判红、出路不是重编引擎」）
#   CRASH_AMPLIFICATION_BREAK=skip-not-exhaustive-in-full-scope  不带参数的全量（scope=full）里不判不穷举，点名的照判
#       第 33 格（「不带参数的全量里一项不穷举，没有红在那一项」）
#   CRASH_AMPLIFICATION_BREAK=hide-scaled-not-exhaustive  缩过规模那一趟的成功句不点名哪几项不穷举
#       第 16 格（「缩过规模的一趟没照实写出展开深度、或判了红、或成功句说了全量、没点名不穷举的项」）
#   CRASH_AMPLIFICATION_BREAK=accept-expand-mismatch  驱动行报的展开深度与交下去的不同也不判红
#       第 17 格（「驱动行的展开深度与交下去的不同却没判红」）
#   CRASH_AMPLIFICATION_BREAK=count-stopped-as-run  叫停与留了没判的块的项也记 green
#       第 18 格（「被叫停的项没按本次未跑逐项报」）、第 19 格（「派出的项全没判完却没退 77」）、第 29 格（「两台分份的 flow= 行留的块没照「同一组有导入行才不算」判」）
#   CRASH_AMPLIFICATION_BREAK=item-78-as-failure  驱动退 78 照普通失败记、接着派后面的项
#       第 20 格（「驱动退 78 却没停下退 78」）、第 22 格（「有红又有一项退 78 时没退 1」）、第 23 格（「驱动退 78 而行里报了红，这一项没记红」）
#   CRASH_AMPLIFICATION_BREAK=honor-test-variables-outside-selftest  平常跑时也认只供自证的四个变量，不拒
#       第 21 格（「平常跑时带了只供自证的变量却没拒」）
#   CRASH_AMPLIFICATION_BREAK=condition-before-red  有红又有一项驱动退 78 时整趟退 78（不让红优先）
#       第 22 格（「有红又有一项退 78 时没退 1」）、第 23 格（「驱动退 78 而行里报了红，这一项没记红」）
#   CRASH_AMPLIFICATION_BREAK=ignore-red-on-78  驱动退 78 时不看行里的红，照本次未跑记
#       第 23 格（「驱动退 78 而行里报了红，这一项没记红」）
#   CRASH_AMPLIFICATION_BREAK=skip-ignore-report  成功句不报按登记整批不判的点数与态数
#       第 24 格（「按登记整批不判的点数与态数没照实写进 ITEM、TOTAL 与成功句」）、第 25 格（「驱动行没报按登记整批不判的两个数，却没照实报取不出」）、
#       第 44 格（「按登记不判的点数与态数没按流分组、每组只取一次」：成功句没报）
#   CRASH_AMPLIFICATION_BREAK=ignore-unknown-as-zero  驱动行没报 ignored_points= / states_skipped_by_ignore= 的记成 0、不点名
#       第 25 格（「驱动行没报按登记整批不判的两个数，却没照实报取不出」）
#   CRASH_AMPLIFICATION_BREAK=accept-bad-red-field  有行缺 red= 或 red 不是非负整数也不判红
#       第 26 格（「驱动行缺 red= 或 red 不是非负整数却没判红」）
#   CRASH_AMPLIFICATION_BREAK=accept-zero-full-item  全量里一项现判 0 态、复用 0 态也不判红
#       第 27 格（「全量里一项现判 0 态、复用 0 态却没判红」）
#   CRASH_AMPLIFICATION_BREAK=sum-reuse-of-every-line  复用数按行相加（先判着之后的分份行、导入行照加），不按流只取一次
#       第 28 格（「复用数没有每项只取一次」）、第 29 格（「两台分份的 flow= 行留的块没照「同一组有导入行才不算」判」）
#   CRASH_AMPLIFICATION_BREAK=flow-share-lines-unexcused  两台分份的 flow= 行（不带 share=、mode=two_hosts_*）不认作分份行，留的块照算没判
#       第 29 格（「两台分份的 flow= 行留的块没照「同一组有导入行才不算」判」）
#   CRASH_AMPLIFICATION_BREAK=excuse-share-lines-without-import  分份行留的块不看同一组有没有导入行，一律不算没判
#       第 29 格（「两台分份的 flow= 行留的块没照「同一组有导入行才不算」判」）
#   CRASH_AMPLIFICATION_BREAK=import-line-as-share-line  导入行（import_run=true）也认作分份行，它留的块不算没判
#       第 29 格（「两台分份的 flow= 行留的块没照「同一组有导入行才不算」判」）
#   CRASH_AMPLIFICATION_BREAK=driver-not-run-as-zero-lines  不认分项驱动的 CRASH_AMPLIFICATION_NOT_RUN 行（退 0 而 0 行的照「驱动退 0 却 0 行」判红）
#       第 31 格（「分项驱动退 77 没记进本次未跑」：原因里没有那一行的 reason=）、第 32 格（「几条流里有的没借到显存，这一项却记了通过」）、
#       第 46 格（「分项驱动报判段没跑的那一项没记本次未跑」）
#   CRASH_AMPLIFICATION_BREAK=driver-77-as-failure  分项驱动退 77 照别的非 0 退出码判红（改前的判法）
#       第 31 格（「分项驱动退 77 没记进本次未跑」）
#   CRASH_AMPLIFICATION_BREAK=judge-not-run-ignored  不认分项驱动转打的 CRASH_AMPLIFICATION_NOT_RUN step=judge 行（退 0 而 0 行的照「驱动退 0 却 0 行」判红）
#       第 46 格（「分项驱动报判段没跑的那一项没记本次未跑」）
#   CRASH_AMPLIFICATION_BREAK=registered-segments-ignored  不认驱动行的 registered_segments=（改前的判法：exhaustive=false 一律算不穷举）
#       第 34 格（「一行 exhaustive=false 只因登记的段，这一项却判了红」）、第 35 格（「一行 exhaustive=false registered_segments=- 没照旧判红在那一条流」）、
#       第 38 格（「登了段的流里另有展开上限盖不住又没登记的段，没判红在那条流与那几段」）、
#       第 39 格（「登了段的行没报 unregistered_segments_beyond_expansion=，没照不穷举判红、出路不是重编引擎」）、
#       第 40 格（「登记段分片枚举判了红、或 ITEM 行没带那两个字段、或成功句没报全展开与分片已判几态」）、
#       第 41 格（「分片段这一趟已判 0 态却没判红在那条流与那一段」）、
#       第 42 格（「登了段的行 registered_segment_domains= / registered_segment_progress= 缺了或写法认不得，没判红在那条流」）、
#       第 43 格（「registered_segment_domains= 漏了登记的段、或分片段没在 registered_segment_progress= 里，没判红」）、
#       第 45 格（「全部复用的一趟判了红」：登记段不认、stub 那条照不穷举红）
#   CRASH_AMPLIFICATION_BREAK=registered-excuses-the-whole-item  一项里有一行登了段，这一项 exhaustive=false 的行就都不算不穷举（不按行判）
#       第 35 格（「一行 exhaustive=false registered_segments=- 没照旧判红在那一条流」）
#   CRASH_AMPLIFICATION_BREAK=dash-counts-as-registered  registered_segments=- 也算登了段
#       第 35 格（「一行 exhaustive=false registered_segments=- 没照旧判红在那一条流」）
#   CRASH_AMPLIFICATION_BREAK=registered-segments-unreported  ITEM 行的 registered_segments= 写 -、成功句不报按段登记那一段
#       第 34 格（「一行 exhaustive=false 只因登记的段，这一项却判了红」）、第 35 格（「一行 exhaustive=false registered_segments=- 没照旧判红在那一条流」）、
#       第 38 格（「登了段的流里另有展开上限盖不住又没登记的段，没判红在那条流与那几段」）、
#       第 39 格（「登了段的行没报 unregistered_segments_beyond_expansion=，没照不穷举判红、出路不是重编引擎」）、
#       第 40 格（「登记段分片枚举判了红、或 ITEM 行没带那两个字段、或成功句没报全展开与分片已判几态」）
#   CRASH_AMPLIFICATION_BREAK=segments-registry-not-sent  不往分项驱动送 SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY（改前的样子）
#       第 36 格（「分项驱动没收到按段登记文件的绝对路径」）
#   CRASH_AMPLIFICATION_BREAK=segments-registry-unchecked  开跑前不核按段登记文件
#       第 37 格（「按段登记文件写坏了却没整趟拒跑」）
#   CRASH_AMPLIFICATION_BREAK=beyond-expansion-ignored  不按 unregistered_segments_beyond_expansion= 判（改前的判法：登了段的行一律不算不穷举）
#       第 38 格（「登了段的流里另有展开上限盖不住又没登记的段，没判红在那条流与那几段」）、
#       第 39 格（「登了段的行没报 unregistered_segments_beyond_expansion=，没照不穷举判红、出路不是重编引擎」）
#   CRASH_AMPLIFICATION_BREAK=beyond-expansion-missing-as-dash  驱动行没报 unregistered_segments_beyond_expansion= 当成报了 -（ITEM 行写 - 不写 unknown，判法上当成没有展不开的段）
#       第 24 格（「按登记整批不判的点数与态数没照实写进 ITEM、TOTAL 与成功句」）、第 25 格（「驱动行没报按登记整批不判的两个数，却没照实报取不出」）、第 28 格（「复用数没有每项只取一次」）、
#       第 29 格（「两台分份的 flow= 行留的块没照「同一组有导入行才不算」判」）、第 34 格（「一行 exhaustive=false 只因登记的段，这一项却判了红」）、
#       第 38 格（「登了段的流里另有展开上限盖不住又没登记的段，没判红在那条流与那几段」）、
#       第 39 格（「登了段的行没报 unregistered_segments_beyond_expansion=，没照不穷举判红、出路不是重编引擎」）
#   CRASH_AMPLIFICATION_BREAK=beyond-expansion-unreported  ITEM 行的 unregistered_segments_beyond_expansion= 写 -、成功句不报展开上限盖不住又没登记的段
#       第 24 格（「按登记整批不判的点数与态数没照实写进 ITEM、TOTAL 与成功句」）、第 25 格（「驱动行没报按登记整批不判的两个数，却没照实报取不出」）、第 28 格（「复用数没有每项只取一次」）、
#       第 29 格（「两台分份的 flow= 行留的块没照「同一组有导入行才不算」判」）、第 34 格（「一行 exhaustive=false 只因登记的段，这一项却判了红」）、第 35 格（「一行 exhaustive=false registered_segments=- 没照旧判红在那一条流」）、
#       第 38 格（「登了段的流里另有展开上限盖不住又没登记的段，没判红在那条流与那几段」）、
#       第 39 格（「登了段的行没报 unregistered_segments_beyond_expansion=，没照不穷举判红、出路不是重编引擎」）、
#       第 40 格（「登记段分片枚举判了红、或 ITEM 行没带那两个字段、或成功句没报全展开与分片已判几态」）
#   CRASH_AMPLIFICATION_BREAK=slice-progress-zero-accepted  分片段已判 0 态也不判红
#       第 41 格（「分片段这一趟已判 0 态却没判红在那条流与那一段」）
#   CRASH_AMPLIFICATION_BREAK=slice-zero-despite-reuse  分片段已判 0 态不看那一行的现判与复用（改前的判法：全部复用的一趟也红）
#       第 45 格（「全部复用的一趟判了红」）
#   CRASH_AMPLIFICATION_BREAK=domains-missing-accepted  登了段的行 registered_segment_domains= / registered_segment_progress= 缺了或写法认不得，当成没有登记段的域（不判红，ITEM 行写 -）
#       第 39 格（「登了段的行没报 unregistered_segments_beyond_expansion=，没照不穷举判红、出路不是重编引擎」）、
#       第 42 格（「登了段的行 registered_segment_domains= / registered_segment_progress= 缺了或写法认不得，没判红在那条流」）
#   CRASH_AMPLIFICATION_BREAK=domain-unlisted-segment-accepted  registered_segments= 列的段不在 registered_segment_domains= 里、分片段不在 registered_segment_progress= 里都不判红
#       第 43 格（「registered_segment_domains= 漏了登记的段、或分片段没在 registered_segment_progress= 里，没判红」）
#   CRASH_AMPLIFICATION_BREAK=slice-domains-unreported  ITEM 行的 registered_segment_domains= / registered_segment_progress= 写 -、成功句不报登记段的域那一段
#       第 34 格（「一行 exhaustive=false 只因登记的段，这一项却判了红」）、第 35 格（「一行 exhaustive=false registered_segments=- 没照旧判红在那一条流」）、
#       第 38 格（「登了段的流里另有展开上限盖不住又没登记的段，没判红在那条流与那几段」）、
#       第 39 格（「登了段的行没报 unregistered_segments_beyond_expansion=，没照不穷举判红、出路不是重编引擎」）、
#       第 40 格（「登记段分片枚举判了红、或 ITEM 行没带那两个字段、或成功句没报全展开与分片已判几态」）、
#       第 41 格（「分片段这一趟已判 0 态却没判红在那条流与那一段」：ITEM 行的进度没照实写）、
#       第 42 格（「登了段的行 registered_segment_domains= / registered_segment_progress= 缺了或写法认不得，没判红在那条流」）、
#       第 43 格（「registered_segment_domains= 漏了登记的段、或分片段没在 registered_segment_progress= 里，没判红」）
#   CRASH_AMPLIFICATION_BREAK=ignore-summed-per-row  按登记不判的 ignored_points= / states_skipped_by_ignore= 对每一行相加（同一条流的本机份、第二台份、导入、叫停再续各行重复计），不按流分组只取一次、同组各行不同也不记 unknown
#       第 44 格（「按登记不判的点数与态数没按流分组、每组只取一次」）
# 只供自证（自证起的子进程里才认；平常跑时有一个非空就退 2 拒跑、拒绝句逐个点名，--list-items 与 --selftest 本身不拒）：
#   CRASH_AMPLIFICATION_BREAK=<项>（上面那张表）；CRASH_AMPLIFICATION_SELFTEST_RED_ITEM=1 每项的测试目标名后面加 -red（假驱动见了退 1）；
#   CRASH_AMPLIFICATION_SELFTEST_DROP_RESULT=<分项> 跑完那一项不打它的 CRASH_AMPLIFICATION_ITEM 行、不计进汇总（造一项收不回）；
#   CRASH_AMPLIFICATION_ITEM_DRIVER=<脚本> 换掉分项驱动（自证用假的）；
#   CRASH_AMPLIFICATION_SELFTEST_SEGMENTS_ROOT=<目录> 按段登记的被判仓根换成它（核与送的都是 <目录>/.claude/gate.d/crash-point-replay-segments.tsv，见「按段登记」）。
#   「自证起的子进程」的标记：CRASH_AMPLIFICATION_SELFTEST_MARK=<文件>，那份文件在、内容是 SELFTEST_MARK_CONTENT 那一行；只有 run_selftest 设它。
#   这六个变量读完就从环境里清掉，不传给分项驱动；经清场包装重起的那一趟由这里原样交回去（只在自证起的子进程里）。
#   清场走真的 scripts/crash/gpu-quiesce-run.sh：自证把多机配置指到临时目录里只写了假清场命令的一份、状态目录指到临时目录，不碰真的清场命令。
#   CRASH_AMPLIFICATION_SWITCH_TABLE=<文件>：--selftest 末尾的开关轮从这份文件的文件头读开关表（不设读本脚本自己的），代码里认的开关照旧从本脚本取；
#     拿一份改坏了表的拷贝（删一格、多一格、格名写错）证开关轮会红。只在 --selftest 的开关轮里读：平常跑不读它，不在上面拒跑的五个之列。
# 自证末尾那一轮（开关轮）走 scripts/crash/crash-amplification-lib.sh 的 crash_amplification_switch_wheel（与双机驱动共用，判法与并行度见那份文件头「弄坏开关轮」）：
#   并行度 CRASH_AMPLIFICATION_SELFTEST_JOBS（正整数），没给取 SINGLEFS_THREAD_CAP，再没给取登记的重活默认并行度（.claude/heavy-thread-default）；一个开关一个进程，各写各的输出文件，逐个 wait 收。
#
# gate-similar: crash-amplification-item.sh 它跑一个分项的一趟（排派活计划、定在哪跑）；这里是分项登记表与全量的循环，每项调它
# gate-similar: crash-amplification-two-hosts.sh 它跑一条用例的双机一趟，由分项驱动在要动用第二台时调；这里不直接调它
# gate-similar: crash-point-replay-shard-run.sh 它跑崩溃点重放登记的崩溃枚举用例（CPU 枚举、checker-tier-crash-point-replay 全绿标记）；这里跑的是走判定库的流水线用例
# gate-similar: gpu-quiesce-run.sh 它只管清场、回读、跑（不复原）；这里选好分项之后经它重起自己
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}

# 全量的展开深度：只在这里登记（文件头「全量的展开深度」）。30 是登记分项里最长一段的写数（第 32 批 Q 探针 PROBE_SEGMENT_SIZES，出处见文件头那一段）
FULL_EXPAND_UP_TO=30

crash_amplification_script_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repository_root="$(cd "$crash_amplification_script_directory/../.." && pwd)"
# 共用库：这里只用它的弄坏开关轮（读表的 crash_amplification_read_switch_table 与逐个打开的 crash_amplification_switch_wheel，--selftest 末尾那一轮，与双机驱动共用）
source "$crash_amplification_script_directory/crash-amplification-lib.sh"

# 分项登记表：分项名 <TAB> 测试目标 <TAB> 用例函数 <TAB> 档位（次序就是全量的跑序：先短的流，长的流复用它们的前缀）
ITEMS=(
  $'fixed-script-through-rollback\tcrash_amplification_fixed_script_through_rollback\tthe_fixed_script_through_the_rollback_is_recorded_checked_in_parallel_and_reused\t全量'
  $'fixed-script-through-unmount\tcrash_amplification_fixed_script_through_unmount\tthe_whole_fixed_script_through_the_unmount_is_recorded_checked_in_parallel_and_reused\t全量'
  $'unlink-after-the-fixed-script\tcrash_enumeration_unlink_after_the_fixed_script_stream\tcrash_amplification_of_the_unlink_after_the_fixed_script_stream_is_clean\t全量'
  $'unlink-merging-inode-leaf-containers\tcrash_enumeration_unlink_merging_inode_leaf_containers_stream\tcrash_amplification_of_the_unlink_merging_inode_leaf_containers_stream_is_clean\t全量'
  $'multi-record-publish\tcrash_enumeration_multi_record_publish_stream\tcrash_amplification_of_the_multi_record_publish_stream_is_clean\t全量'
  $'unlink-after-the-multi-record-publish\tcrash_enumeration_unlink_after_the_multi_record_publish_stream\tcrash_amplification_of_the_unlink_after_the_multi_record_publish_stream_is_clean\t全量'
  $'tree-split-streams\tcrash_enumeration_tree_split_streams\tcrash_amplification_of_every_tree_split_stream_is_clean\t全量'
  $'position-addressed-trees\tcrash_enumeration_position_addressed_trees\tcrash_amplification_of_every_position_addressed_tree_stream_is_clean\t全量'
  $'floor-raise-pushed-by-the-session\tcrash_enumeration_floor_raise_pushed_by_admission\tcrash_amplification_of_the_floor_raise_pushed_by_the_session_is_clean\t全量'
  $'sigma-of-the-misaligned-reuse\trecord_checker_judges_absence_by_the_persisted_set\tcrash_amplification_of_sigma_on_the_misaligned_reuse_history_is_clean\t全量'
  $'compaction-relocation\tcrash_enumeration_compaction_relocation_stream\tcrash_amplification_of_the_compaction_relocation_stream_is_clean\t全量'
  $'compaction-relocating-a-block-shared-with-a-snapshot\tcrash_enumeration_compaction_relocating_a_block_shared_with_a_snapshot_stream\tcrash_amplification_of_the_compaction_relocating_a_block_shared_with_a_snapshot_stream_is_clean\t全量'
  $'lifecycle-closure\tcrash_enumeration_lifecycle_closure_stream\tcrash_amplification_of_the_lifecycle_closure_stream_is_clean\t全量'
  $'unlink-under-a-snapshot\tcrash_enumeration_unlink_under_a_snapshot_stream\tcrash_amplification_of_the_unlink_under_a_snapshot_stream_is_clean\t全量'
  $'truncate-to-zero\tcrash_enumeration_truncate_to_zero_stream\tcrash_amplification_of_the_truncate_to_zero_stream_is_clean\t全量'
  $'truncate-into-a-unit-then-extend\tcrash_enumeration_truncate_into_a_unit_then_extend_stream\tcrash_amplification_of_the_truncate_into_a_unit_then_extend_stream_is_clean\t全量'
  $'truncate-under-a-snapshot\tcrash_enumeration_truncate_under_a_snapshot_stream\tcrash_amplification_of_the_truncate_under_a_snapshot_stream_is_clean\t全量'
  $'snapshot-creation\tcrash_enumeration_snapshot_creation_stream\tcrash_amplification_of_the_snapshot_creation_stream_is_clean\t全量'
  $'overwrite-and-unlink-under-a-snapshot\tcrash_enumeration_overwrite_and_unlink_under_a_snapshot_stream\tcrash_amplification_of_the_overwrite_and_unlink_under_a_snapshot_stream_is_clean\t全量'
  $'snapshot-destruction-resumed-after-a-crash\tcrash_enumeration_snapshot_destruction_resumed_after_a_crash_stream\tcrash_amplification_of_the_snapshot_destruction_resumed_after_a_crash_stream_is_clean\t全量'
  $'one-file-overwritten-across-two-writable-remounts\tcrash_enumeration_one_file_overwritten_across_two_writable_remounts_stream\tcrash_amplification_of_one_file_overwritten_across_two_writable_remounts_is_clean\t全量'
  $'several-inodes-overwritten-in-turn\tcrash_enumeration_several_inodes_overwritten_in_turn_stream\tcrash_amplification_of_several_inodes_overwritten_in_turn_is_clean\t全量'
  $'large-file-overwritten-across-lower-leaves\tcrash_enumeration_large_file_overwritten_across_lower_leaves_stream\tcrash_amplification_of_a_large_file_overwritten_across_lower_leaves_is_clean\t全量'
  $'unlink-of-two-files-holding-data\tcrash_enumeration_unlink_of_two_files_holding_data_streams\tcrash_amplification_of_both_unlink_streams_is_clean\t全量'
)

# 只供自证的变量（文件头「只供自证」）：先存下来、判是不是自证起的子进程，标记读完就清；四个变量在平常跑的那一支拒或清
entry_original_arguments=("$@")
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

# 一项的汇总（判法见文件头「一项怎么判」「复用数怎么取」「按登记整批不判」「按段登记」）：读这一项的输出文件，打一行十四个字段，空格隔开：
# 行数 现判 红 复用（或 unknown） exhaustive（true|false） 展开深度（或 unknown） 叫停的行数（不算 warm_up 行） 没判的块（不算 warm_up 行；两台分份行的只在同一组没有导入行时算）
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
  local across=0 as_zero=0 ignore_as_zero=0 every_line=0 flow_unexcused=0 excuse_without_import=0 import_as_share=0 output_file="$1" ignore_per_row=0
  crash_amplification_break_is ignore-summed-per-row && ignore_per_row=1
  local registered_ignored=0 registered_whole_item=0 dash_registered=0 beyond_ignored=0 beyond_missing_as_dash=0
  local slice_zero_accepted=0 slice_zero_despite_reuse=0 domains_missing_accepted=0 domain_unlisted_accepted=0
  crash_amplification_break_is reuse-across-shares && across=1
  crash_amplification_break_is reuse-as-zero && as_zero=1
  crash_amplification_break_is ignore-unknown-as-zero && ignore_as_zero=1
  crash_amplification_break_is sum-reuse-of-every-line && every_line=1
  crash_amplification_break_is flow-share-lines-unexcused && flow_unexcused=1
  crash_amplification_break_is excuse-share-lines-without-import && excuse_without_import=1
  crash_amplification_break_is import-line-as-share-line && import_as_share=1
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
  # 复用数按流分组、每组只取一次（文件头「复用数怎么取」）：组里有先判着的行只取第一行先判着的；没有就取导入行之外各行之和。
  # 没判的块同样按流分组（文件头「一项怎么判」not-run）：分份行留的块只在同一组没有导入行时算。
  # 按段登记（文件头「按段登记」）：exhaustive=false 而 registered_segments= 是 <段号>:<写数>[,…] 的行，unregistered_segments_beyond_expansion= 是 - 才不算不穷举；
  #   不是 - 的那几段、没报这个字段的那几行另记；那几段与各行报的这个字段都按流去重记下。
  #   登了段的行还要报 registered_segment_domains= / registered_segment_progress=（文件头「按段登记」的分片枚举）：缺、漏段、认不得的那几行另记，分片段已判 0 态的那几段另记
  awk -v across="$across" -v as_zero="$as_zero" -v ignore_as_zero="$ignore_as_zero" -v every_line="$every_line" \
      -v flow_unexcused="$flow_unexcused" -v excuse_without_import="$excuse_without_import" -v import_as_share="$import_as_share" \
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
      explicit = ""; states = ""; checked = ""; share_count = ""; stopped = 0; imported = 0; unjudged = 0; warm = 0; exhaustive = ""; expand = ""
      red_seen = 0; ignored_here = ""; skipped_here = ""; flow = ""; import_run = 0; mode = ""; registered = ""; beyond = ""; beyond_seen = 0
      domains = ""; domains_seen = 0; progress = ""; progress_seen = 0
      for (field = 2; field <= NF; field++) {
        key = $field; sub(/=.*/, "", key); value = substr($field, length(key) + 2)
        if (key == "states_reused") explicit = value
        else if (key == "flow") flow = value
        else if (key == "mode") mode = value
        else if (key == "import_run" && value == "true") import_run = 1
        else if (key == "states") states = value
        else if (key == "states_checked_here") checked = value
        else if (key == "share") { share_count = value; sub(/^[0-9]+\//, "", share_count) }
        else if (key == "stopped_early" && value == "true") stopped = 1
        else if (key == "imported_blocks" && value + 0 > 0) imported = 1
        else if (key == "unjudged_blocks") unjudged = value + 0
        else if (key == "red") { red_seen = 1; if (value ~ /^[0-9]+$/) red += value; else bad_red++ }
        else if (key == "ignored_points") ignored_here = value
        else if (key == "states_skipped_by_ignore") skipped_here = value
        else if (key == "warm_up" && value == "true") warm = 1
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
      # 分份行：share=<i>/<n>、n>1，或不带 share= 而 mode=two_hosts_* 的 flow= 行；导入行不是
      two_hosts_flow_line = (share_count == "" && mode ~ /^two_hosts_/ && !flow_unexcused)
      share_line = ((share_count ~ /^[0-9]+$/ && share_count + 0 > 1) || two_hosts_flow_line) && !imported && (!import_run || import_as_share)
      if (import_run || imported) import_seen[flow] = 1
      if (stopped && !warm) stopped_lines++
      if (unjudged > 0 && !warm) { if (share_line) share_unjudged[flow] += unjudged; else unjudged_blocks += unjudged }
      # 这一行的复用数：line_reuse，取不出时 line_unknown=1
      line_reuse = 0; line_unknown = 0
      if (explicit ~ /^[0-9]+$/) line_reuse = explicit + 0
      else {
        derivable = (states ~ /^[0-9]+$/ && checked ~ /^[0-9]+$/ && states + 0 >= checked + 0)
        if (derivable && (across || (share_count == "1" && !stopped && !imported))) line_reuse = states - checked
        else line_unknown = 1
      }
      if (every_line) { if (line_unknown) reuse_unknown = 1; else reused += line_reuse; next }
      if (!(flow in seen_flow)) { seen_flow[flow] = 1; flow_order[++flow_count] = flow }
      if (warm) {
        if (!(flow in warm_reuse)) { warm_reuse[flow] = line_reuse; warm_unknown[flow] = line_unknown }
      } else if (!import_run) {
        other_lines[flow]++; other_reuse[flow] += line_reuse; if (line_unknown) other_unknown[flow] = 1
      }
    }
    END {
      for (flow in share_unjudged) if (!excuse_without_import && !(flow in import_seen)) unjudged_blocks += share_unjudged[flow]
      if (!every_line) {
        for (index_of_flow = 1; index_of_flow <= flow_count; index_of_flow++) {
          flow = flow_order[index_of_flow]
          if (flow in warm_reuse) { if (warm_unknown[flow]) reuse_unknown = 1; else reused += warm_reuse[flow] }
          else if (other_lines[flow] == 0 || other_unknown[flow]) reuse_unknown = 1
          else reused += other_reuse[flow]
        }
      }
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

list_items() { # 分项名、测试目标、用例函数、档位
  local row tier
  for row in "${ITEMS[@]}"; do
    tier="$(item_fields "$row" 4)"
    [[ "$tier" == 全量 ]] && tier="全量（不在快档）"
    printf '%s\t%s\t%s\t%s\n' "$(item_fields "$row" 1)" "$(item_fields "$row" 2)" "$(item_fields "$row" 3)" "$tier"
  done
}

refuse_listing_the_items() { # <哪里不对>：打原因、出路与可点名的项，退 2
  echo "  ✗ $1" >&2
  echo "  → 怎么办：分项名照下面列的写（--item <分项>，给几次取并集）；不带 --item 就是全量" >&2
  list_items | sed 's/^/    /' >&2
  exit 2
}

usage() { # <哪里不对>
  echo "  ✗ 用法不对：$1。用法：$0 [--item <分项>]… [--expand-up-to <N>] [--list-items] [--selftest] [--threads <N>] [--memory-cap <上限>] [--peer-memory-cap <上限>] [--library <库目录>] [--label <标签>]" >&2
  echo "  → 怎么办：不带参数就是剪枝跑全部项；分项名照 --list-items 列的写；--expand-up-to 给正整数（与全量的展开深度不同就是缩过规模的一趟）" >&2
  exit 2
}

selected=(); driver_arguments=(); list_only=0; selftest=0; expand_up_to_given=""
while (($# > 0)); do
  case "$1" in
    --item)
      if (( $# < 2 )) || [[ -z "$2" || "$2" == -* ]]; then refuse_listing_the_items "--item 后面缺项名"; fi
      selected+=("$2"); shift 2 ;;
    --list-items) list_only=1; shift ;;
    --selftest) selftest=1; shift ;;
    --expand-up-to)
      if (( $# < 2 )) || ! [[ "$2" =~ ^[1-9][0-9]*$ ]]; then usage "--expand-up-to 要一个正整数"; fi
      expand_up_to_given="$2"; shift 2 ;;
    --threads|--memory-cap|--peer-memory-cap|--library|--label)
      if (( $# < 2 )) || [[ -z "$2" || "$2" == -* ]]; then usage "$1 要一个值"; fi
      driver_arguments+=("$1" "$2"); shift 2 ;;
    *) usage "认不得的参数「$1」" ;;
  esac
done

if (( list_only )); then list_items; exit 0; fi

# 登记表自检：四段都非空、档位认得、分项名不重
declare -A seen=()
for row in "${ITEMS[@]}"; do
  name="$(item_fields "$row" 1)"; target="$(item_fields "$row" 2)"; function_name="$(item_fields "$row" 3)"; tier="$(item_fields "$row" 4)"
  if [[ -z "$name" || -z "$target" || -z "$function_name" || -z "$tier" ]]; then echo "  ✗ 登记表有一行不全：$row" >&2; echo "  → 怎么办：四段都填上（分项名、测试目标、用例函数、档位）" >&2; exit 2; fi
  if [[ "$tier" != 全量 ]]; then echo "  ✗ 登记表里 $name 的档位「$tier」认不得" >&2; echo "  → 怎么办：档位今天只有「全量」（不在快档）；要加快档先给入口加 --quick 再登记" >&2; exit 2; fi
  if [[ -n "${seen[$name]:-}" ]]; then echo "  ✗ 分项名重了：$name" >&2; echo "  → 怎么办：分项名一个只登一次" >&2; exit 2; fi
  seen["$name"]=1
done

run_selftest() {
  local work stub log nested="$running_under_selftest"
  work="$(mktemp -d "${TMPDIR:-/tmp}/crash-amplification-selftest.XXXXXX")"
  trap 'rm -rf -- "${work:?}"' EXIT
  # 自证起的子进程的标记：只有这里设（文件头「只供自证」）
  printf '%s\n' "$SELFTEST_MARK_CONTENT" > "$work/mark"
  export CRASH_AMPLIFICATION_SELFTEST_MARK="$work/mark"
  # 子进程的运行条件、每项输出都落在这里：多机配置指到不在的文件（两个开关都关），退役的私有显卡配置变量清掉，本次未跑不写进门禁的汇总
  export SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host.env" SINGLEFS_CRASH_AMPLIFICATION_HOME="$work/home" SINGLEFS_LAST_RUNS_DIRECTORY="$work/last-runs"
  unset SINGLEFS_GPU_CARDS_CONFIG CRASH_AMPLIFICATION_SELFTEST_RED_ITEM CRASH_AMPLIFICATION_SELFTEST_DROP_RESULT CRASH_AMPLIFICATION_ITEM_SWITCHES CRASH_AMPLIFICATION_ITEM_CARDS GATE_NOT_RUN_FILE
  unset SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY CRASH_AMPLIFICATION_SELFTEST_SEGMENTS_ROOT
  # 按段登记的仓根（文件头「按段登记」）：合表照抄仓里那一份，按段登记文件只写一行核得过的（第 37 格临时换成写坏的一份、跑完换回）
  local segments_root="$work/segments-root" segments_registry
  segments_registry="$segments_root/.claude/gate.d/crash-point-replay-segments.tsv"
  mkdir -p "$segments_root/.claude/gate.d"
  cp -- "$repository_root/.claude/gate.d/crash-point-replay-items.tsv" "$segments_root/.claude/gate.d/crash-point-replay-items.tsv"
  printf '%s\n' "# 自证的按段登记" "position-addressed-trees	stub	3	300	$(printf '0123456789abcdef%.0s' 1 2 3 4)" >"$segments_registry"
  stub="$work/driver.sh"
  cat >"$stub" <<'STUB'
#!/usr/bin/env bash
# 假的分项驱动：记下参数与收到的按段登记文件路径；形态按 STUB_SHAPE_<测试目标>（没给按 STUB_SHAPE，再没给 single-host）打几行 CRASH_AMPLIFICATION（照真驱动的几种行），
# 每行的 expand_up_to 照交进来的 --expand-up-to（给了 STUB_EXPAND_OVERRIDE 就报它：造一个不照参数跑的驱动）；测试目标名以 -red 结尾的退 1
printf 'driver %s segments=[%s]\n' "$*" "${SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY-unset}" >> "${STUB_LOG:?}"
target="$1"; expand=""; previous=""
for argument in "$@"; do
  if [[ "$previous" == --expand-up-to ]]; then expand="$argument"; fi
  previous="$argument"
done
expand="${STUB_EXPAND_OVERRIDE:-$expand}"
shape_variable="STUB_SHAPE_$(printf '%s' "${target%-red}" | tr -c 'A-Za-z0-9_' '_')"
shape="${!shape_variable:-${STUB_SHAPE:-single-host}}"
case "$shape" in
  single-host) # 本机一台跑完的 fixed-script 汇总行：复用 10 − 7 = 3
    echo "CRASH_AMPLIFICATION mode=single_host_cpu share=0/1 cards=0 states=10 states_checked_here=7 blocks_checked_here=2 imported_blocks=0 unjudged_blocks=0 red=0 expand_up_to=$expand exhaustive=true" ;;
  derived-and-explicit) # 一行推得出（复用 3）、一行带 states_reused=5：这一项复用 8，现判 7 + 2 = 9
    echo "CRASH_AMPLIFICATION mode=single_host_cpu share=0/1 cards=0 states=10 states_checked_here=7 blocks_checked_here=2 imported_blocks=0 unjudged_blocks=0 red=0 expand_up_to=$expand exhaustive=true"
    echo "CRASH_AMPLIFICATION flow=stub mode=single_host_cpu states_checked_here=2 states_reused=5 blocks_checked_here=1 unjudged_blocks=0 red=0 expand_up_to=$expand exhaustive=true" ;;
  flow) # amplify_prepared_flow 打的那种行：不带 states=，取不出复用数
    echo "CRASH_AMPLIFICATION flow=stub mode=single_host_cpu states_checked_here=2 blocks_checked_here=1 unjudged_blocks=0 red=0 violations_in_the_library=0 expand_up_to=$expand exhaustive=true" ;;
  two-hosts) # 两台跑的三行：本机那一份、第二台那一份、导入那一趟；分份行留着对方的块没判是本来如此，混着对方那一份与导入的块，取不出复用数
    echo "CRASH_AMPLIFICATION mode=two_hosts_cpu share=0/2 cards=0 states=10 states_checked_here=4 blocks_checked_here=1 imported_blocks=0 unjudged_blocks=3 red=0 expand_up_to=$expand exhaustive=true"
    echo "CRASH_AMPLIFICATION mode=two_hosts_cpu share=1/2 cards=0 states=10 states_checked_here=6 blocks_checked_here=1 imported_blocks=0 unjudged_blocks=2 red=0 expand_up_to=$expand exhaustive=true"
    echo "CRASH_AMPLIFICATION mode=two_hosts_cpu share=0/2 cards=0 states=10 states_checked_here=0 blocks_checked_here=0 imported_blocks=3 unjudged_blocks=0 red=0 expand_up_to=$expand exhaustive=true" ;;
  library) # 复用三问那一格（第 47 格）：照 $SINGLEFS_CRASH_AMPLIFICATION_HOME/stub-library 按这一项测试目标的单元文件（$STUB_UNITS/<测试目标>）内容摘要记判过没有：
    #   判过的这一趟现判 0 态、复用 10 态，没判过的现判 10 态并记下（真驱动的库在引擎里按块键记，这里只演「按键存、键相同复用」）
    unit_key="$(sha256sum "${STUB_UNITS:?}/$target" | cut -c1-64)"
    record="$SINGLEFS_CRASH_AMPLIFICATION_HOME/stub-library/$target.$unit_key"
    if [[ -f "$record" ]]; then
      echo "CRASH_AMPLIFICATION mode=single_host_cpu share=0/1 cards=0 states=10 states_checked_here=0 blocks_checked_here=0 imported_blocks=0 unjudged_blocks=0 red=0 expand_up_to=$expand exhaustive=true"
    else
      mkdir -p "$SINGLEFS_CRASH_AMPLIFICATION_HOME/stub-library" && : >"$record"
      echo "CRASH_AMPLIFICATION mode=single_host_cpu share=0/1 cards=0 states=10 states_checked_here=10 blocks_checked_here=2 imported_blocks=0 unjudged_blocks=0 red=0 expand_up_to=$expand exhaustive=true"
    fi ;;
  silent) ;; # cargo 筛出 0 条：退 0、一行都不打
  placement-not-run) # 排派活计划那一趟借不到显存、一项都没跑到：照真驱动打 DECISION 与 CRASH_AMPLIFICATION_NOT_RUN 行、退 77，不打 CRASH_AMPLIFICATION 行
    echo "CRASH_AMPLIFICATION_DECISION where=not-run reason=no-card-of-the-pool-had-room-in-time across_machines=0 quiesced=1"
    echo "CRASH_AMPLIFICATION_NOT_RUN step=placement test=crash_amplification::run_from_environment reason=等了 1800 秒，池里没有一张卡空得出 2560 MiB（假驱动）"
    echo "  ! 本次未跑（这不是通过）：排派活计划那一趟借不到显存，一块都没判（假驱动）" >&2
    exit 77 ;;
  partial-not-run) # 一条用例三条流、第 2 条借不到显存：借到的照判（一行判完的汇总行），没借到的那一条照真驱动打 CRASH_AMPLIFICATION_NOT_RUN 行，退 0
    echo "CRASH_AMPLIFICATION_NOT_RUN step=placement test=crash_amplification::run_from_environment flow=2/3 reason=等了 1800 秒，池里没有一张卡空得出 5120 MiB（假驱动）"
    echo "CRASH_AMPLIFICATION mode=single_host_cpu share=0/1 cards=0 states=10 states_checked_here=7 blocks_checked_here=2 imported_blocks=0 unjudged_blocks=0 red=0 expand_up_to=$expand exhaustive=true" ;;
  judge-not-run) # 不带 gpu 特性的二进制遇到多机配置 ENABLE_GPU=1：判段这一趟不跑、录入段照录，分项驱动转打那一条流的 CRASH_AMPLIFICATION_NOT_RUN step=judge 行、退 0，没有 CRASH_AMPLIFICATION 行
    echo "CRASH_AMPLIFICATION_NOT_RUN step=judge test=crash_amplification::run_from_environment flow=stub reason=配置写了 ENABLE_GPU=1，这份二进制没有编进 gpu 特性：判段这一趟不跑（假驱动）" ;;
  not-exhaustive) # 展开深度不够把每一段展开完
    echo "CRASH_AMPLIFICATION mode=single_host_cpu share=0/1 cards=0 states=10 states_checked_here=7 blocks_checked_here=2 imported_blocks=0 unjudged_blocks=0 red=0 expand_up_to=$expand exhaustive=false" ;;
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
    echo "CRASH_AMPLIFICATION mode=single_host_cpu share=0/1 cards=0 states=10 states_checked_here=4 blocks_checked_here=1 imported_blocks=0 unjudged_blocks=2 red=0 expand_up_to=$expand exhaustive=true stopped_early=true" ;;
  unjudged-flow) # 一台跑的流留着 3 块没判
    echo "CRASH_AMPLIFICATION flow=stub mode=single_host_cpu states_checked_here=2 blocks_checked_here=1 unjudged_blocks=3 red=0 violations_in_the_library=0 expand_up_to=$expand exhaustive=true" ;;
  reddish) # 驱动退 0，行里却报了 2 处红
    echo "CRASH_AMPLIFICATION mode=single_host_cpu share=0/1 cards=0 states=10 states_checked_here=7 blocks_checked_here=2 imported_blocks=0 unjudged_blocks=0 red=2 expand_up_to=$expand exhaustive=true" ;;
  device) # 显卡用不上：驱动退 78，不打汇总行
    echo "  ✗ 点名的卡用不了（假驱动）" >&2
    exit 78 ;;
  red-then-device) # 双机驱动本机那一份失败时的样子：先打出已判的行（报了 2 处红），再照日志分类退 78
    echo "CRASH_AMPLIFICATION mode=single_host_cpu share=0/1 cards=0 states=10 states_checked_here=7 blocks_checked_here=2 imported_blocks=0 unjudged_blocks=0 red=2 expand_up_to=$expand exhaustive=true"
    echo "GPU_CARD_NOT_USABLE index=1（假驱动）"
    exit 78 ;;
  ignored) # 照驱动行的约定带全五个字段：复用 3、按登记整批不判 2 个点 50 态
    echo "CRASH_AMPLIFICATION mode=single_host_cpu share=0/1 cards=0 states=10 states_checked_here=7 blocks_checked_here=2 imported_blocks=0 unjudged_blocks=0 red=0 states_reused=3 exhaustive=true expand_up_to=$expand ignored_points=2 states_skipped_by_ignore=50" ;;
  no-red-field) # 行里没有 red=（驱动行的约定坏了）
    echo "CRASH_AMPLIFICATION mode=single_host_cpu share=0/1 cards=0 states=10 states_checked_here=7 blocks_checked_here=2 imported_blocks=0 unjudged_blocks=0 red_states=3 violations=3 expand_up_to=$expand exhaustive=true" ;;
  red-not-a-number) # red= 的值不是数
    echo "CRASH_AMPLIFICATION mode=single_host_cpu share=0/1 cards=0 states=10 states_checked_here=7 blocks_checked_here=2 imported_blocks=0 unjudged_blocks=0 red=? expand_up_to=$expand exhaustive=true" ;;
  red-negative) # red= 的值是负数
    echo "CRASH_AMPLIFICATION mode=single_host_cpu share=0/1 cards=0 states=10 states_checked_here=7 blocks_checked_here=2 imported_blocks=0 unjudged_blocks=0 red=-1 expand_up_to=$expand exhaustive=true" ;;
  zero) # 一态都没判到：现判 0、复用 0
    echo "CRASH_AMPLIFICATION flow=stub mode=single_host_cpu states_checked_here=0 states_reused=0 blocks_checked_here=0 unjudged_blocks=0 red=0 expand_up_to=$expand exhaustive=true" ;;
  warm-up-then-shares) # 双机先判着、两台分份、导入，各趟照实报录入时的复用数（L1）：一共 100 态，开跑时库里已判 40 态（本机那一份 25、第二台那一份 15）；
    #   先判着判 10 态（落在本机那一份 6、第二台那一份 4）被叫停；分份 0/2 判 19（复用 25 + 6）、1/2 判 31（复用 15 + 4）；导入那一趟判 0、复用 50（本机那一份全部）。
    #   计划行带 states_reused= 不取。这一项现判 10 + 19 + 31 + 0 = 60、复用只取先判着那一行 40（60 + 40 = 100）；按行相加是 40 + 31 + 19 + 50 = 140
    echo "CRASH_AMPLIFICATION_PLACEMENT host=local judge=gpu pending_blocks=6 pending_states=60 states_reused=40 tables_mebibytes=1 cards=local:0:100:100,peer:0:100:100 states_placed_on_local=30 states_placed_on_peer=30 states_per_second_on_this_host=10 estimated_seconds_on_this_host=6"
    echo "CRASH_AMPLIFICATION mode=two_hosts_cpu_with_gpus share=0/1 states=100 states_checked_here=10 blocks_checked_here=1 unjudged_blocks=5 red=0 stopped_early=true states_reused=40 exhaustive=true expand_up_to=$expand ignored_points=0 states_skipped_by_ignore=0 warm_up=true"
    echo "CRASH_AMPLIFICATION_PLACEMENT host=peer judge=gpu pending_blocks=5 pending_states=50 states_reused=50 tables_mebibytes=1 cards=local:0:100:100,peer:0:100:100 states_placed_on_local=25 states_placed_on_peer=25 states_per_second_on_this_host=10 estimated_seconds_on_this_host=5"
    echo "CRASH_AMPLIFICATION mode=two_hosts_cpu_with_gpus share=0/2 cards=1 states=100 states_checked_here=19 blocks_checked_here=2 imported_blocks=0 unjudged_blocks=3 red=0 states_reused=31 exhaustive=true expand_up_to=$expand ignored_points=0 states_skipped_by_ignore=0"
    echo "CRASH_AMPLIFICATION mode=two_hosts_cpu_with_gpus share=1/2 cards=1 states=100 states_checked_here=31 blocks_checked_here=3 imported_blocks=0 unjudged_blocks=2 red=0 states_reused=19 exhaustive=true expand_up_to=$expand ignored_points=0 states_skipped_by_ignore=0"
    echo "CRASH_AMPLIFICATION mode=two_hosts_cpu_with_gpus share=0/2 cards=1 states=100 states_checked_here=0 blocks_checked_here=0 imported_blocks=3 unjudged_blocks=0 red=0 states_reused=50 exhaustive=true expand_up_to=$expand ignored_points=0 states_skipped_by_ignore=0 import_run=true" ;;
  two-flows-two-hosts) # 两条流（flow= 行不带 share=）、两台分份、导入，没有先判着的：本机那一份 a 判 8 复用 12、b 判 5 复用 0；第二台那一份 a 判 6 复用 4、b 判 3 复用 2；
    #   导入那一趟 a、b 各判 0、复用 20、5（本机那一份全部）。计划行带 states_reused= 不取。这一项现判 22、复用 12 + 4 + 0 + 2 = 18；按行相加是 18 + 25 = 43
    echo "CRASH_AMPLIFICATION_PLACEMENT host=local judge=gpu pending_blocks=2 pending_states=14 states_reused=16 tables_mebibytes=1 cards=local:0:100:100,peer:0:100:100 states_placed_on_local=7 states_placed_on_peer=7 states_per_second_on_this_host=7 estimated_seconds_on_this_host=2"
    echo "CRASH_AMPLIFICATION_PLACEMENT host=local judge=gpu pending_blocks=1 pending_states=8 states_reused=2 tables_mebibytes=1 cards=local:0:100:100,peer:0:100:100 states_placed_on_local=4 states_placed_on_peer=4 states_per_second_on_this_host=8 estimated_seconds_on_this_host=1"
    echo "CRASH_AMPLIFICATION flow=a mode=two_hosts_cpu states_checked_here=8 blocks_checked_here=1 unjudged_blocks=0 red=0 violations_in_the_library=0 states_reused=12 exhaustive=true expand_up_to=$expand ignored_points=0 states_skipped_by_ignore=0"
    echo "CRASH_AMPLIFICATION flow=b mode=two_hosts_cpu states_checked_here=5 blocks_checked_here=1 unjudged_blocks=0 red=0 violations_in_the_library=0 states_reused=0 exhaustive=true expand_up_to=$expand ignored_points=0 states_skipped_by_ignore=0"
    echo "CRASH_AMPLIFICATION flow=a mode=two_hosts_cpu states_checked_here=6 blocks_checked_here=1 unjudged_blocks=0 red=0 violations_in_the_library=0 states_reused=4 exhaustive=true expand_up_to=$expand ignored_points=0 states_skipped_by_ignore=0"
    echo "CRASH_AMPLIFICATION flow=b mode=two_hosts_cpu states_checked_here=3 blocks_checked_here=1 unjudged_blocks=0 red=0 violations_in_the_library=0 states_reused=2 exhaustive=true expand_up_to=$expand ignored_points=0 states_skipped_by_ignore=0"
    echo "CRASH_AMPLIFICATION flow=a mode=two_hosts_cpu states_checked_here=0 blocks_checked_here=0 unjudged_blocks=0 red=0 violations_in_the_library=0 states_reused=20 exhaustive=true expand_up_to=$expand ignored_points=0 states_skipped_by_ignore=0 import_run=true"
    echo "CRASH_AMPLIFICATION flow=b mode=two_hosts_cpu states_checked_here=0 blocks_checked_here=0 unjudged_blocks=0 red=0 violations_in_the_library=0 states_reused=5 exhaustive=true expand_up_to=$expand ignored_points=0 states_skipped_by_ignore=0 import_run=true" ;;
  flow-shares-then-import) # 两条流两台分份：flow= 分份行留着对方那一份（a 本机 2、第二台 1，b 各 1），导入之后两组对账都为空：这一项判完了。
    #   现判 8 + 5 + 6 + 3 = 22、复用 12 + 0 + 4 + 2 = 18；分份行留的 5 块照算就成了「本次未跑」
    echo "CRASH_AMPLIFICATION flow=a mode=two_hosts_cpu states_checked_here=8 blocks_checked_here=1 unjudged_blocks=2 red=0 violations_in_the_library=0 states_reused=12 exhaustive=true expand_up_to=$expand ignored_points=0 states_skipped_by_ignore=0"
    echo "CRASH_AMPLIFICATION flow=b mode=two_hosts_cpu states_checked_here=5 blocks_checked_here=1 unjudged_blocks=1 red=0 violations_in_the_library=0 states_reused=0 exhaustive=true expand_up_to=$expand ignored_points=0 states_skipped_by_ignore=0"
    echo "CRASH_AMPLIFICATION flow=a mode=two_hosts_cpu states_checked_here=6 blocks_checked_here=2 unjudged_blocks=1 red=0 violations_in_the_library=0 states_reused=4 exhaustive=true expand_up_to=$expand ignored_points=0 states_skipped_by_ignore=0"
    echo "CRASH_AMPLIFICATION flow=b mode=two_hosts_cpu states_checked_here=3 blocks_checked_here=1 unjudged_blocks=1 red=0 violations_in_the_library=0 states_reused=2 exhaustive=true expand_up_to=$expand ignored_points=0 states_skipped_by_ignore=0"
    echo "CRASH_AMPLIFICATION flow=a mode=two_hosts_cpu states_checked_here=0 blocks_checked_here=0 unjudged_blocks=0 red=0 violations_in_the_library=0 states_reused=20 exhaustive=true expand_up_to=$expand ignored_points=0 states_skipped_by_ignore=0 import_run=true"
    echo "CRASH_AMPLIFICATION flow=b mode=two_hosts_cpu states_checked_here=0 blocks_checked_here=0 unjudged_blocks=0 red=0 violations_in_the_library=0 states_reused=5 exhaustive=true expand_up_to=$expand ignored_points=0 states_skipped_by_ignore=0 import_run=true" ;;
  flow-shares-import-missing) # 两条流两台分份：a 没有导入行（分份行留的 2 + 1 块照算），b 有导入行而导入之后还留 1 块（分份行留的 1 + 1 块不算、导入行的 1 块照算）：
    #   这一项留了 4 块没判、本次未跑；现判 4 + 3 + 2 + 2 + 0 = 11、复用 a 0 + 1、b 0 + 0 = 1
    echo "CRASH_AMPLIFICATION flow=a mode=two_hosts_cpu states_checked_here=4 blocks_checked_here=1 unjudged_blocks=2 red=0 violations_in_the_library=0 states_reused=0 exhaustive=true expand_up_to=$expand ignored_points=0 states_skipped_by_ignore=0"
    echo "CRASH_AMPLIFICATION flow=b mode=two_hosts_cpu states_checked_here=2 blocks_checked_here=1 unjudged_blocks=1 red=0 violations_in_the_library=0 states_reused=0 exhaustive=true expand_up_to=$expand ignored_points=0 states_skipped_by_ignore=0"
    echo "CRASH_AMPLIFICATION flow=a mode=two_hosts_cpu states_checked_here=3 blocks_checked_here=1 unjudged_blocks=1 red=0 violations_in_the_library=0 states_reused=1 exhaustive=true expand_up_to=$expand ignored_points=0 states_skipped_by_ignore=0"
    echo "CRASH_AMPLIFICATION flow=b mode=two_hosts_cpu states_checked_here=2 blocks_checked_here=1 unjudged_blocks=1 red=0 violations_in_the_library=0 states_reused=0 exhaustive=true expand_up_to=$expand ignored_points=0 states_skipped_by_ignore=0"
    echo "CRASH_AMPLIFICATION flow=b mode=two_hosts_cpu states_checked_here=0 blocks_checked_here=0 unjudged_blocks=1 red=0 violations_in_the_library=0 states_reused=2 exhaustive=true expand_up_to=$expand ignored_points=0 states_skipped_by_ignore=0 import_run=true" ;;
  ignored-three-runs) # σ 项那种形态：一条流 sigma，双机本机份、第二台份、导入三趟，每行都报同一份 1 个点 869269834 态。按登记不判的数每组只取一次：1 个点、869269834 态（按行相加是 3 个点、2607809502 态）
    echo "CRASH_AMPLIFICATION flow=sigma mode=two_hosts_cpu states_checked_here=8 blocks_checked_here=1 unjudged_blocks=0 red=0 violations_in_the_library=0 states_reused=12 exhaustive=true expand_up_to=$expand ignored_points=1 states_skipped_by_ignore=869269834"
    echo "CRASH_AMPLIFICATION flow=sigma mode=two_hosts_cpu states_checked_here=6 blocks_checked_here=1 unjudged_blocks=0 red=0 violations_in_the_library=0 states_reused=4 exhaustive=true expand_up_to=$expand ignored_points=1 states_skipped_by_ignore=869269834"
    echo "CRASH_AMPLIFICATION flow=sigma mode=two_hosts_cpu states_checked_here=0 blocks_checked_here=0 unjudged_blocks=0 red=0 violations_in_the_library=0 states_reused=20 exhaustive=true expand_up_to=$expand ignored_points=1 states_skipped_by_ignore=869269834 import_run=true" ;;
  ignored-resumed-four) # 叫停过再续的四行，同一条流 sigma：先判着被叫停的一行、两台分份两行、导入一行，每行都报同一份 1 个点 869269834 态：1 个点、869269834 态（按行相加是 4 个点）
    echo "CRASH_AMPLIFICATION flow=sigma mode=two_hosts_cpu states_checked_here=3 blocks_checked_here=1 unjudged_blocks=5 red=0 violations_in_the_library=0 stopped_early=true states_reused=12 exhaustive=true expand_up_to=$expand ignored_points=1 states_skipped_by_ignore=869269834 warm_up=true"
    echo "CRASH_AMPLIFICATION flow=sigma mode=two_hosts_cpu states_checked_here=8 blocks_checked_here=1 unjudged_blocks=0 red=0 violations_in_the_library=0 states_reused=12 exhaustive=true expand_up_to=$expand ignored_points=1 states_skipped_by_ignore=869269834"
    echo "CRASH_AMPLIFICATION flow=sigma mode=two_hosts_cpu states_checked_here=6 blocks_checked_here=1 unjudged_blocks=0 red=0 violations_in_the_library=0 states_reused=4 exhaustive=true expand_up_to=$expand ignored_points=1 states_skipped_by_ignore=869269834"
    echo "CRASH_AMPLIFICATION flow=sigma mode=two_hosts_cpu states_checked_here=0 blocks_checked_here=0 unjudged_blocks=0 red=0 violations_in_the_library=0 states_reused=20 exhaustive=true expand_up_to=$expand ignored_points=1 states_skipped_by_ignore=869269834 import_run=true" ;;
  ignored-two-flows) # 两条流各一份（a 报 1 个点 10 态、b 报 2 个点 20 态），a 流有两行（本机份与第二台份）报同一份：每组取一次，合计 3 个点、30 态
    echo "CRASH_AMPLIFICATION flow=a mode=two_hosts_cpu states_checked_here=8 blocks_checked_here=1 unjudged_blocks=0 red=0 violations_in_the_library=0 states_reused=12 exhaustive=true expand_up_to=$expand ignored_points=1 states_skipped_by_ignore=10"
    echo "CRASH_AMPLIFICATION flow=b mode=single_host_cpu states_checked_here=5 blocks_checked_here=1 unjudged_blocks=0 red=0 violations_in_the_library=0 states_reused=0 exhaustive=true expand_up_to=$expand ignored_points=2 states_skipped_by_ignore=20"
    echo "CRASH_AMPLIFICATION flow=a mode=two_hosts_cpu states_checked_here=6 blocks_checked_here=1 unjudged_blocks=0 red=0 violations_in_the_library=0 states_reused=4 exhaustive=true expand_up_to=$expand ignored_points=1 states_skipped_by_ignore=10" ;;
  ignored-no-flow-twice) # 两行都不带 flow=（归一组），各报 2 个点 50 态：2 个点、50 态（按行相加是 4 个点、100 态）
    echo "CRASH_AMPLIFICATION mode=single_host_cpu share=0/1 cards=0 states=10 states_checked_here=7 blocks_checked_here=2 imported_blocks=0 unjudged_blocks=0 red=0 states_reused=3 exhaustive=true expand_up_to=$expand ignored_points=2 states_skipped_by_ignore=50"
    echo "CRASH_AMPLIFICATION mode=single_host_cpu share=0/1 cards=0 states=10 states_checked_here=7 blocks_checked_here=2 imported_blocks=0 unjudged_blocks=0 red=0 states_reused=3 exhaustive=true expand_up_to=$expand ignored_points=2 states_skipped_by_ignore=50" ;;
  ignored-flow-differs) # a 流两行报的不同（1 个点 10 态与 1 个点 11 态），b 流一份 2 个点 20 态：这一项两个数记 unknown、点名 a 流，不悄悄取最大或最小
    echo "CRASH_AMPLIFICATION flow=a mode=two_hosts_cpu states_checked_here=8 blocks_checked_here=1 unjudged_blocks=0 red=0 violations_in_the_library=0 states_reused=12 exhaustive=true expand_up_to=$expand ignored_points=1 states_skipped_by_ignore=10"
    echo "CRASH_AMPLIFICATION flow=b mode=single_host_cpu states_checked_here=5 blocks_checked_here=1 unjudged_blocks=0 red=0 violations_in_the_library=0 states_reused=0 exhaustive=true expand_up_to=$expand ignored_points=2 states_skipped_by_ignore=20"
    echo "CRASH_AMPLIFICATION flow=a mode=two_hosts_cpu states_checked_here=6 blocks_checked_here=1 unjudged_blocks=0 red=0 violations_in_the_library=0 states_reused=4 exhaustive=true expand_up_to=$expand ignored_points=1 states_skipped_by_ignore=11" ;;
esac
if [[ "$target" == *-red ]]; then exit 1; fi
exit 0
STUB
  chmod +x "$stub"
  # 清场走真的 gpu-quiesce-run.sh，多机配置里只写假的清场命令：清场与复原各往 QUIESCE_LOG 记一笔（包装不调复原，那一笔只在它的弄坏开关 restore-after-run 下出现）；
  #   QUIESCE_STOPPED_CHECK 写成 fail 时清场回读不过。包装的登记与本机日志都放这里的临时目录，不落进真工作根
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
  # 第 1 格：--list-items 每个登记的分项一行、四段、档位写「全量（不在快档）」，不清场
  cells=$((cells + 1)); : > "$QUIESCE_LOG"
  output="$(bash "$0" --list-items)"; exit_code=$?
  if [[ "$exit_code" != 0 || "$(grep -c . <<<"$output")" != "${#ITEMS[@]}" ]] \
     || [[ "$(awk -F'\t' 'NF == 4 && $4 == "全量（不在快档）"' <<<"$output" | wc -l)" != "${#ITEMS[@]}" ]] || [[ -s "$QUIESCE_LOG" ]]; then
    selftest_cell_failed "--list-items 没有逐行列出登记的分项与档位（退 $exit_code），或它清了场" "看 list_items：每行四段、档位「全量」写成「全量（不在快档）」，在清场之前就退"
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
  # 第 4 格：全量：每个分项各调一次驱动，每次带 --test-filter 与全量的展开深度；成功句写「崩溃放量全量（展开 N）」
  cells=$((cells + 1)); : > "$log"
  output="$(entry 2>/dev/null)"; exit_code=$?
  if [[ "$exit_code" != 0 ]] || [[ "$(wc -l <"$log")" != "${#ITEMS[@]}" ]] || [[ "$(grep -c -- '--test-filter' "$log")" != "${#ITEMS[@]}" ]] \
     || [[ "$(grep -cE -- "--expand-up-to $FULL_EXPAND_UP_TO( |\$)" "$log")" != "${#ITEMS[@]}" ]] \
     || ! grep -q "崩溃放量全量（展开 $FULL_EXPAND_UP_TO）" <<<"$output" || ! grep -q ' scope=full ' <<<"$output"; then
    selftest_cell_failed "全量没把每个分项各调一次驱动、带 --test-filter 与 --expand-up-to $FULL_EXPAND_UP_TO，或没退 0、成功句没写全量（退 $exit_code，调了 $(wc -l <"$log") 次）" \
      "看主循环：不带参数跑 ITEMS 的每一行，filter_arguments 带 --test-filter，driver_arguments 带 FULL_EXPAND_UP_TO"
  fi
  # 第 5 格：点名一项（给两次取并集）：只调一次、目标对
  cells=$((cells + 1)); : > "$log"
  entry --item tree-split-streams --item tree-split-streams >/dev/null 2>&1
  if [[ "$(wc -l <"$log")" != 1 ]] || ! grep -q 'crash_enumeration_tree_split_streams' "$log"; then
    selftest_cell_failed "--item 没有只跑点名的那一项、或同名给两次跑了两遍（调了 $(wc -l <"$log") 次）" "看选分项那一段：点名的分项只取登记表里同名那一行，重名只取一次"
  fi
  # 第 6 格：清场：开跑前清一次、每一项都带 --quiesced、跑完不复原（点名的分项也一样）
  cells=$((cells + 1)); : > "$log"
  output="$(QUIESCE_LOG="$log" entry --item fixed-script-through-rollback --item multi-record-publish --item tree-split-streams 2>/dev/null)"; exit_code=$?
  local expected_calls="quiesce-stop driver-quiesced driver-quiesced driver-quiesced " actual_calls
  actual_calls="$(sed -E 's/^driver .* --quiesced( .*)?$/driver-quiesced/; s/^driver .*$/driver-plain/; s/^quiesce (.*)$/quiesce-\1/' "$log" | tr '\n' ' ')"
  if [[ "$exit_code" != 0 || "$actual_calls" != "$expected_calls" ]] || ! grep -q 'CRASH_AMPLIFICATION_TOTAL dispatched=3 items=3 failed=0 not_run=0 ' <<<"$output"; then
    selftest_cell_failed "没有照「开跑前清一次、每项带 --quiesced、跑完不复原」走（退 $exit_code，调用次序「$actual_calls」）" "看主循环之前经 gpu-quiesce-run.sh 重起的那一段与 quiesced_arguments（重起的那一趟认 GPU_QUIESCE_HOLDER）"
  fi
  # 第 7 格：清场没清成：退 78，一项都不跑
  cells=$((cells + 1)); : > "$log"
  QUIESCE_STOPPED_CHECK=fail entry --item tree-split-streams >/dev/null 2>&1; exit_code=$?
  if [[ "$exit_code" != 78 ]] || [[ -s "$log" ]]; then
    selftest_cell_failed "清场没清成却没退 78、或照跑了分项（退 $exit_code）" "看主循环之前经 gpu-quiesce-run.sh 重起的那一段：清场回读没过它退 78，不起这一趟"
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
  # 第 12 格：两台跑的三行：分份行留着对方的块不算本次未跑；states − states_checked_here 不拿来当复用数，reused=unknown、退 0、成功句点名它
  cells=$((cells + 1))
  output="$(STUB_SHAPE=two-hosts entry --item fixed-script-through-rollback 2>/dev/null)"; exit_code=$?
  if [[ "$exit_code" != 0 ]] || ! grep -q '^CRASH_AMPLIFICATION_ITEM item=fixed-script-through-rollback exit=0 states=10 reused=unknown .* verdict=green ' <<<"$output" \
     || ! grep -q 'CRASH_AMPLIFICATION_TOTAL .* not_run=0 reuse_unknown=1 states=10 reused_known=0 ' <<<"$output" \
     || ! grep -q '复用数取不出（1 项：fixed-script-through-rollback，驱动行没报 states_reused=）' <<<"$output"; then
    selftest_cell_failed "两台跑的行被拿来推复用数、被报成本次未跑、或取不出却没照实报出（退 $exit_code）：$output" \
      "看 summarize_the_item：只有 share=<i>/1、没叫停、没导入的行才取 states − states_checked_here；share_line 的 unjudged_blocks 不算"
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
  # 第 15 格：展开深度是全量的那一个，行里 exhaustive=false：这一项红、整趟退 1
  cells=$((cells + 1))
  output="$(STUB_SHAPE=not-exhaustive entry --item tree-split-streams 2>"$errors")"; exit_code=$?
  if [[ "$exit_code" != 1 ]] || ! grep -q 'CRASH_AMPLIFICATION_TOTAL dispatched=1 items=1 failed=1 ' <<<"$output" || ! grep -q '不是 exhaustive=true' "$errors"; then
    selftest_cell_failed "全量的展开深度下有行不是穷举却没判红（退 $exit_code）：$output" "看主循环里不穷举那一条与 summarize_the_item 的不穷举行数：展开深度等于 FULL_EXPAND_UP_TO 时要每一行 exhaustive=true，或 exhaustive=false 只因登记的段"
  fi
  # 第 16 格：缩过规模（--expand-up-to 6）：交给驱动的是 6，exhaustive=false 不红、退 0；ITEM 与 TOTAL 写出 6，成功句写「缩过规模：展开 6」、不说全量；
  #   点名两项，tree-split-streams 不穷举、fixed-script-through-rollback 穷举：成功句只点名不穷举的那一项
  cells=$((cells + 1)); : > "$log"
  output="$(STUB_SHAPE_crash_enumeration_tree_split_streams=not-exhaustive entry --expand-up-to 6 --item fixed-script-through-rollback --item tree-split-streams 2>/dev/null)"; exit_code=$?
  local success_line
  success_line="$(grep '✓' <<<"$output")"
  if [[ "$exit_code" != 0 ]] || [[ "$(grep -cE -- '--expand-up-to 6( |$)' "$log")" != 2 ]] \
     || ! grep -q '^CRASH_AMPLIFICATION_ITEM item=tree-split-streams .* expand_up_to=6 exhaustive=false verdict=green ' <<<"$output" \
     || ! grep -q '^CRASH_AMPLIFICATION_ITEM item=fixed-script-through-rollback .* expand_up_to=6 exhaustive=true verdict=green ' <<<"$output" \
     || ! grep -q 'CRASH_AMPLIFICATION_TOTAL .* expand_up_to=6 scope=scaled ' <<<"$output" \
     || ! grep -q '缩过规模：展开 6' <<<"$success_line" || grep -q '全量' <<<"$success_line" \
     || ! grep -q '；不穷举 1 项：tree-split-streams（展开 6 盖不住它们最长的一段' <<<"$success_line"; then
    selftest_cell_failed "缩过规模的一趟没照实写出展开深度、或判了红、或成功句说了全量、没点名不穷举的项（退 $exit_code）：$output" \
      "看 requested_expand_up_to、scope、成功句的开头与 scaled_part（不穷举的项记进 not_exhaustive_items）"
  fi
  # 第 17 格：驱动行报的展开深度（6）与交下去的（全量的那一个）不同：这一项红、整趟退 1
  cells=$((cells + 1))
  output="$(STUB_EXPAND_OVERRIDE=6 entry --item tree-split-streams 2>"$errors")"; exit_code=$?
  if [[ "$exit_code" != 1 ]] || ! grep -q 'CRASH_AMPLIFICATION_TOTAL dispatched=1 items=1 failed=1 ' <<<"$output" || ! grep -q '驱动没照给的展开深度跑' "$errors"; then
    selftest_cell_failed "驱动行的展开深度与交下去的不同却没判红（退 $exit_code）：$output" "看主循环里 item_expand 与 requested_expand_up_to 比的那一条"
  fi
  # 第 18 格：两项里一项被叫停（stopped_early=true、留 2 块）：那一项 verdict=not-run、逐项报本次未跑，整趟退 0；成功句点名它、不说「都退 0、0 红」
  cells=$((cells + 1))
  output="$(STUB_SHAPE_crash_enumeration_tree_split_streams=stopped entry --item fixed-script-through-rollback --item tree-split-streams 2>"$errors")"; exit_code=$?
  if [[ "$exit_code" != 0 ]] || ! grep -q '^CRASH_AMPLIFICATION_ITEM item=tree-split-streams .* verdict=not-run ' <<<"$output" \
     || ! grep -q 'CRASH_AMPLIFICATION_TOTAL dispatched=2 items=2 failed=0 not_run=1 ' <<<"$output" \
     || ! grep -q '本次未跑（这不是通过）：崩溃放量分项 tree-split-streams' "$errors" \
     || ! grep -q '本次未跑 1 项：tree-split-streams' <<<"$output" || grep -q '都退 0、0 红' <<<"$output"; then
    selftest_cell_failed "被叫停的项没按本次未跑逐项报、或成功句还说都退 0、0 红（退 $exit_code）：$output" "看主循环里 item_stopped / item_unjudged 那一支与成功句的 run_part"
  fi
  # 第 19 格：唯一的一项留了没判的块（一台跑的流，unjudged_blocks=3）：一项都没判完，整趟退 77
  cells=$((cells + 1))
  output="$(STUB_SHAPE=unjudged-flow entry --item multi-record-publish 2>/dev/null)"; exit_code=$?
  if [[ "$exit_code" != 77 ]] || ! grep -q 'CRASH_AMPLIFICATION_TOTAL dispatched=1 items=1 failed=0 not_run=1 ' <<<"$output"; then
    selftest_cell_failed "派出的项全没判完却没退 77（退 $exit_code）：$output" "看末尾 items_not_run 等于 items_run 那一支"
  fi
  # 第 20 格：第一项的驱动退 78（卡用不上）：停在这一项、后面不派，整趟退 78，出路写「修好设备，或改配置」
  cells=$((cells + 1)); : > "$log"
  output="$(STUB_SHAPE_crash_amplification_fixed_script_through_rollback=device entry --item fixed-script-through-rollback --item tree-split-streams 2>"$errors")"; exit_code=$?
  if [[ "$exit_code" != 78 ]] || [[ "$(wc -l <"$log")" != 1 ]] || ! grep -q 'CRASH_AMPLIFICATION_TOTAL dispatched=1 items=1 ' <<<"$output" || ! grep -q '修好设备，或改配置' "$errors"; then
    selftest_cell_failed "驱动退 78 却没停下退 78（退 $exit_code，调了 $(wc -l <"$log") 次驱动）：$output" "看主循环里 item_exit 为 78 那一支与末尾 stopped_by_condition"
  fi
  # 第 21 格：平常跑（不是自证起的子进程）时带了只供自证的变量：退 2、不起驱动，拒绝句点名带了的每一个。这一格只判入口拒不拒、
  #   不往下起清场与驱动（没拒的那一趟——弄坏开关 honor-test-variables-outside-selftest、或拒的那一段被删了——也一样）：换驱动的变量指到
  #   一份不在的文件，没拒时入口在「分项驱动不在」那一步就退 2，到不了经清场包装重起那一步；点名的分项也是登记表里没有的，
  #   选分项那一步同样在清场之前退 2。拒没拒只认拒绝句。换驱动的变量每一次都带着、每一次都核它被点名；另外四个各试一次
  cells=$((cells + 1))
  local variable value named refusal refused_all=1 missing_driver="$work/no-such-item-driver.sh"
  for variable in CRASH_AMPLIFICATION_BREAK CRASH_AMPLIFICATION_SELFTEST_RED_ITEM CRASH_AMPLIFICATION_SELFTEST_DROP_RESULT CRASH_AMPLIFICATION_SELFTEST_SEGMENTS_ROOT; do
    value=1
    [[ "$variable" == CRASH_AMPLIFICATION_BREAK ]] && value="${CRASH_AMPLIFICATION_BREAK:+$CRASH_AMPLIFICATION_BREAK,}swallow-exit"
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
    selftest_cell_failed "平常跑时带了只供自证的变量却没拒、或拒绝句没点名带了的那几个（最后一次退 $exit_code）" "看参数之后拒只供自证变量的那一段：running_under_selftest 为 0 时五个变量有一个非空就逐个点名、退 2"
  fi
  # 第 22 格：前一项红（驱动退 0、red=2）、后一项驱动退 78：停在退 78 那一项、后面不派，退 78 的那一项在 ✗ 里点名，整趟退 1（有红先判）
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
  # 第 23 格：驱动打了一行 red=2 之后退 78：这一项记红（不让 78 盖掉），停在这一项、后面不派，整趟退 1
  cells=$((cells + 1)); : > "$log"
  output="$(STUB_SHAPE_crash_amplification_fixed_script_through_rollback=red-then-device entry --item fixed-script-through-rollback --item tree-split-streams 2>"$errors")"; exit_code=$?
  if [[ "$exit_code" != 1 ]] || [[ "$(wc -l <"$log")" != 1 ]] \
     || ! grep -q '^CRASH_AMPLIFICATION_ITEM item=fixed-script-through-rollback exit=78 .* red=2 .* verdict=red ' <<<"$output" \
     || ! grep -q 'CRASH_AMPLIFICATION_TOTAL dispatched=1 items=1 failed=1 not_run=0 ' <<<"$output" \
     || ! grep -q '分项 fixed-script-through-rollback 的驱动退 78' "$errors"; then
    selftest_cell_failed "驱动退 78 而行里报了红，这一项没记红、或没停在这一项（退 $exit_code，调了 $(wc -l <"$log") 次驱动）：$output" \
      "看主循环里 item_exit 为 78 那一支：red_reasons 不空就记 red，照样停在这一项"
  fi
  # 第 24 格：全量、每一行都带 ignored_points=2 states_skipped_by_ignore=50：ITEM 与 TOTAL 照实写出，
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
  # 第 25 格：点名两项，一项带这两个数、一项的驱动行没报：没报的那一项写 unknown、计进 ignore_unknown，不红、退 0；成功句点名它，不记成 0
  cells=$((cells + 1))
  output="$(STUB_SHAPE_crash_amplification_fixed_script_through_rollback=ignored entry --item fixed-script-through-rollback --item tree-split-streams 2>/dev/null)"; exit_code=$?
  success_line="$(grep '✓' <<<"$output")"
  if [[ "$exit_code" != 0 ]] || ! grep -q '^CRASH_AMPLIFICATION_ITEM item=tree-split-streams .* verdict=green .* ignored_points=unknown states_skipped_by_ignore=unknown registered_segments=- unregistered_segments_beyond_expansion=unknown registered_segment_domains=- registered_segment_progress=-$' <<<"$output" \
     || ! grep -q 'CRASH_AMPLIFICATION_TOTAL .* ignored_points=2 states_skipped_by_ignore=50 ignore_unknown=1$' <<<"$output" \
     || ! grep -q '按登记整批不判：取得出的 1 项 2 个点、50 态（类名取不出.*）；另有 1 项取不出：tree-split-streams' <<<"$success_line"; then
    selftest_cell_failed "驱动行没报按登记整批不判的两个数，却没照实报取不出（退 $exit_code）：$output" \
      "看 summarize_the_item：两个数有一行没报就整项写 unknown；主循环计进 items_ignore_unknown，成功句点名它"
  fi
  # 第 26 格：驱动行缺 red=、red=?、red=-1：各判红、整趟退 1，出路点到驱动行的约定
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
  # 第 27 格：不带参数的全量，每一项现判 0 态、复用 0 态：每项判红、整趟退 1
  cells=$((cells + 1))
  output="$(STUB_SHAPE=zero entry 2>"$errors")"; exit_code=$?
  if [[ "$exit_code" != 1 ]] || ! grep -q "CRASH_AMPLIFICATION_TOTAL dispatched=$item_count items=$item_count failed=$item_count .* states=0 reused_known=0 " <<<"$output" \
     || ! grep -q '现判 0 态、复用 0 态' "$errors"; then
    selftest_cell_failed "全量里一项现判 0 态、复用 0 态却没判红（退 $exit_code）：$output" "看主循环里 scope 为 full、item_states 为 0、item_reused 为 0 那一条"
  fi
  # 第 28 格：复用数每项只取一次（文件头「复用数怎么取」）。点名两项：一项是双机先判着（叫停行）、两台分份、导入（导入行），夹着两行计划行；
  #   一项是两条流的 flow= 行、两台分份、导入。两项与 TOTAL 逐个字段钉绝对值：现判 60 复用 40、现判 22 复用 18，合计现判 82 复用 58，都绿、退 0
  cells=$((cells + 1))
  output="$(STUB_SHAPE_crash_amplification_fixed_script_through_rollback=warm-up-then-shares STUB_SHAPE_crash_enumeration_tree_split_streams=two-flows-two-hosts \
    entry --item fixed-script-through-rollback --item tree-split-streams 2>/dev/null)"; exit_code=$?
  if [[ "$exit_code" != 0 ]] \
     || ! grep -qE "^CRASH_AMPLIFICATION_ITEM item=fixed-script-through-rollback exit=0 states=60 reused=40 red=0 lines=4 expand_up_to=$FULL_EXPAND_UP_TO exhaustive=true verdict=green seconds=[0-9]+ ignored_points=0 states_skipped_by_ignore=0 registered_segments=- unregistered_segments_beyond_expansion=unknown registered_segment_domains=- registered_segment_progress=-\$" <<<"$output" \
     || ! grep -qE "^CRASH_AMPLIFICATION_ITEM item=tree-split-streams exit=0 states=22 reused=18 red=0 lines=6 expand_up_to=$FULL_EXPAND_UP_TO exhaustive=true verdict=green seconds=[0-9]+ ignored_points=0 states_skipped_by_ignore=0 registered_segments=- unregistered_segments_beyond_expansion=unknown registered_segment_domains=- registered_segment_progress=-\$" <<<"$output" \
     || ! grep -qE "^CRASH_AMPLIFICATION_TOTAL dispatched=2 items=2 failed=0 not_run=0 reuse_unknown=0 states=82 reused_known=58 red=0 expand_up_to=$FULL_EXPAND_UP_TO scope=items seconds=[0-9]+ ignored_points=0 states_skipped_by_ignore=0 ignore_unknown=0\$" <<<"$output" \
     || ! grep -q '2 项都退 0、0 红；现判 82 态、复用 58 态；派出 2 项、收回 2 项' <<<"$output"; then
    selftest_cell_failed "复用数没有每项只取一次：先判着之后的分份行、导入行或计划行的复用数被加了进去（退 $exit_code）：$output" \
      "看 summarize_the_item：按 flow= 分组，组里有 warm_up=true 的只取第一行先判着的，没有就取 import_run=true 之外各行之和；CRASH_AMPLIFICATION_PLACEMENT 行不取"
  fi
  # 第 29 格：两台分份的 flow= 行（不带 share=）留的块（文件头「一项怎么判」not-run）。点名两项：一项两条流都有导入行、导入之后对账为空，
  #   分份行留的块不算、记 green；一项 a 流没有导入行、b 流的导入行还留 1 块，记 not-run、留 4 块（a 分份行 3 块 + b 导入行 1 块）。逐字段钉绝对值，整趟退 0
  cells=$((cells + 1))
  output="$(STUB_SHAPE_crash_enumeration_tree_split_streams=flow-shares-then-import STUB_SHAPE_crash_enumeration_multi_record_publish_stream=flow-shares-import-missing \
    entry --item multi-record-publish --item tree-split-streams 2>"$errors")"; exit_code=$?
  if [[ "$exit_code" != 0 ]] \
     || ! grep -qE "^CRASH_AMPLIFICATION_ITEM item=multi-record-publish exit=0 states=11 reused=1 red=0 lines=5 expand_up_to=$FULL_EXPAND_UP_TO exhaustive=true verdict=not-run seconds=[0-9]+ ignored_points=0 states_skipped_by_ignore=0 registered_segments=- unregistered_segments_beyond_expansion=unknown registered_segment_domains=- registered_segment_progress=-\$" <<<"$output" \
     || ! grep -qE "^CRASH_AMPLIFICATION_ITEM item=tree-split-streams exit=0 states=22 reused=18 red=0 lines=6 expand_up_to=$FULL_EXPAND_UP_TO exhaustive=true verdict=green seconds=[0-9]+ ignored_points=0 states_skipped_by_ignore=0 registered_segments=- unregistered_segments_beyond_expansion=unknown registered_segment_domains=- registered_segment_progress=-\$" <<<"$output" \
     || ! grep -qE "^CRASH_AMPLIFICATION_TOTAL dispatched=2 items=2 failed=0 not_run=1 reuse_unknown=0 states=33 reused_known=19 red=0 expand_up_to=$FULL_EXPAND_UP_TO scope=items " <<<"$output" \
     || ! grep -q '本次未跑（这不是通过）：崩溃放量分项 multi-record-publish：留了 4 块没判（unjudged_blocks）' "$errors" \
     || grep -q '崩溃放量分项 tree-split-streams' "$errors"; then
    selftest_cell_failed "两台分份的 flow= 行留的块没照「同一组有导入行才不算」判：有导入行、对账为空的被报成本次未跑，或没有导入行、导入行留了块的没照实报（退 $exit_code）：$output" \
      "看 summarize_the_item：分份行（share=<i>/<n> n>1，或不带 share= 而 mode=two_hosts_* 的 flow= 行，导入行除外）留的块记进 share_unjudged，END 里同一组没有导入行才加进没判的块"
  fi
  # 第 30 格：跑完不调复原命令（多机配置里是假的清场命令，记进 QUIESCE_LOG）：照常跑完退 0、有项红（驱动退 1）整趟退 1，
  #   两趟都只清一次场、跑完不复原，包装打 GPU_QUIESCE step=restore result=not-restored
  local not_restored_exit not_restored_calls
  cells=$((cells + 1)); : > "$log"
  QUIESCE_LOG="$log" entry --item tree-split-streams >/dev/null 2>"$errors"; not_restored_exit=$?
  QUIESCE_LOG="$log" CRASH_AMPLIFICATION_SELFTEST_RED_ITEM=1 entry --item tree-split-streams >/dev/null 2>>"$errors"; exit_code=$?
  not_restored_calls="$(sed -E 's/^driver .*$/driver/; s/^quiesce (.*)$/quiesce-\1/' "$log" | tr '\n' ' ')"
  if [[ "$not_restored_exit" != 0 || "$exit_code" != 1 || "$not_restored_calls" != "quiesce-stop driver quiesce-stop driver " ]] \
     || [[ "$(grep -c '^GPU_QUIESCE step=restore result=not-restored ' "$errors")" != 2 ]]; then
    selftest_cell_failed "跑完调了复原命令、或有项红时没退 1（照常那一趟退 $not_restored_exit、有项红那一趟退 $exit_code，调用次序「$not_restored_calls」，not-restored 行 $(grep -c '^GPU_QUIESCE step=restore result=not-restored ' "$errors") 行）" \
      "看主循环之前经 gpu-quiesce-run.sh 重起的那一段与那份包装的 gpu_quiesce_leave：重起的那一趟退几，包装都不复原"
  fi
  # 第 31 格：分项驱动排派活计划那一趟借不到显存、一项都没跑到（CRASH_AMPLIFICATION_NOT_RUN 行、退 77、0 行 CRASH_AMPLIFICATION）：
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
  # 第 32 格：一条用例几条流，有的借不到显存（分项驱动退 0，打一行判完的 CRASH_AMPLIFICATION 与一行 flow=2/3 的 CRASH_AMPLIFICATION_NOT_RUN）：
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
  # 第 33 格：不带参数的全量（scope=full、展开深度是 FULL_EXPAND_UP_TO），只有 multi-record-publish 的驱动行报 exhaustive=false（flow=stub）：
  #   红在这一项（ITEM 行 exhaustive=false verdict=red，✗ 点名它与不穷举的流、出路写「按段登记（.claude/gate.d/crash-point-replay-segments.tsv）或加深展开」，重跑命令只点名它），
  #   别的每一项 verdict=green、✗ 里不点名；TOTAL 报派出、收回都是登记的项数、failed=1、scope=full，整趟退 1、不打成功句
  local not_exhaustive_greens
  cells=$((cells + 1)); : > "$log"
  output="$(STUB_SHAPE_crash_enumeration_multi_record_publish_stream=not-exhaustive-flow entry 2>"$errors")"; exit_code=$?
  not_exhaustive_greens="$(grep -cE "^CRASH_AMPLIFICATION_ITEM item=[^ ]+ exit=0 .* expand_up_to=$FULL_EXPAND_UP_TO exhaustive=true verdict=green " <<<"$output")"
  if [[ "$exit_code" != 1 ]] || [[ "$(grep -cE -- "--expand-up-to $FULL_EXPAND_UP_TO( |\$)" "$log")" != "$item_count" ]] \
     || ! grep -qE "^CRASH_AMPLIFICATION_ITEM item=multi-record-publish exit=0 .* expand_up_to=$FULL_EXPAND_UP_TO exhaustive=false verdict=red " <<<"$output" \
     || [[ "$not_exhaustive_greens" != "$((item_count - 1))" ]] \
     || ! grep -qE "^CRASH_AMPLIFICATION_TOTAL dispatched=$item_count items=$item_count failed=1 not_run=0 .* expand_up_to=$FULL_EXPAND_UP_TO scope=full " <<<"$output" \
     || ! grep -q "^  ✗ 分项 multi-record-publish：展开 $FULL_EXPAND_UP_TO 是全量的展开深度，却有 1 行驱动行不是 exhaustive=true、也没按段登记（不穷举的流：stub）" "$errors" \
     || [[ "$(grep -c '^  ✗ 分项 ' "$errors")" != 1 ]] \
     || ! grep -q '按段登记（.claude/gate.d/crash-point-replay-segments.tsv）或加深展开' "$errors" \
     || ! grep -qE -- '只重跑判红的那几项：.* --item multi-record-publish$' "$errors" \
     || grep -q '✓' <<<"$output"; then
    selftest_cell_failed "不带参数的全量里一项不穷举，没有红在那一项：那一项没记红、别的项被连带判红、整趟没退 1、✗ 没点名不穷举的流或没给出路（退 $exit_code，绿项 $not_exhaustive_greens 个）：$output" \
      "看主循环里不穷举那一条：展开深度等于 FULL_EXPAND_UP_TO 时（scope 是 full 也算）有一行不是 exhaustive=true 又没按段登记就记这一项红，理由点名 item_not_exhaustive_flows、出路写按段登记或加深展开"
  fi
  # 第 34 格：不带参数的全量里一项的一行 exhaustive=false registered_segments=3:300 unregistered_segments_beyond_expansion=-（只因登记的段不穷举，flow=stub），
  #   同一项另一条流穷举：这一项不红（ITEM 行照实写 exhaustive=false、verdict=green、registered_segments=stub@3:300、unregistered_segments_beyond_expansion=-），
  #   别的项 registered_segments=- unregistered_segments_beyond_expansion=unknown（假驱动那几种行没报这个字段），整趟退 0、成功句说全量，
  #   末尾报「按段登记 1 段（1 项…）：position-addressed-trees 流 stub 段 3（300 写）」与「展开上限盖不住又没登记的段：0 段（另有 7 项取不出：…）」
  local registered_greens
  cells=$((cells + 1))
  output="$(STUB_SHAPE_crash_enumeration_position_addressed_trees=registered-flow entry 2>"$errors")"; exit_code=$?
  success_line="$(grep '✓' <<<"$output")"
  registered_greens="$(grep -cE "^CRASH_AMPLIFICATION_ITEM item=[^ ]+ exit=0 .* exhaustive=true verdict=green .* registered_segments=- unregistered_segments_beyond_expansion=unknown registered_segment_domains=- registered_segment_progress=-\$" <<<"$output")"
  if [[ "$exit_code" != 0 ]] \
     || ! grep -qE "^CRASH_AMPLIFICATION_ITEM item=position-addressed-trees exit=0 .* expand_up_to=$FULL_EXPAND_UP_TO exhaustive=false verdict=green .* registered_segments=stub@3:300 unregistered_segments_beyond_expansion=- registered_segment_domains=stub@3:full registered_segment_progress=-\$" <<<"$output" \
     || [[ "$registered_greens" != "$((item_count - 1))" ]] \
     || ! grep -q "崩溃放量全量（展开 $FULL_EXPAND_UP_TO）" <<<"$success_line" \
     || ! grep -q "；按段登记 1 段（1 项，只按甲二 ∪ 展开 6 判、没判全域）：position-addressed-trees 流 stub 段 3（300 写）；登记段的域（甲二 ∪ 展开 6）1 段（1 项；全展开 1 段、分片 0 段）：position-addressed-trees 流 stub 段 3：全展开；展开上限盖不住又没登记的段：0 段（另有 $((item_count - 1)) 项取不出：.*，驱动行没报 unregistered_segments_beyond_expansion=）\$" <<<"$success_line"; then
    selftest_cell_failed "一行 exhaustive=false 只因登记的段（registered_segments=3:300），这一项却判了红、或 ITEM 行与成功句没报段号与写数（退 $exit_code，别的项绿 $registered_greens 个）：$output" \
      "看 summarize_the_item 里 registered_segments 那一段（exhaustive=false 而登了段的行不算不穷举、段按流记下）与成功句的 registered_part"
  fi
  # 第 35 格：点名一项、展开深度是全量的那一个：stub 那条流登了第 3 段（exhaustive=false），other 那条也不穷举而 registered_segments=-：
  #   这一项照旧红在 other（✗ 只点名 other，出路写按段登记或加深展开），ITEM 行照样报 stub 登的段，整趟退 1、不打成功句
  cells=$((cells + 1))
  output="$(STUB_SHAPE_crash_enumeration_position_addressed_trees=registered-and-unregistered entry --item position-addressed-trees 2>"$errors")"; exit_code=$?
  if [[ "$exit_code" != 1 ]] \
     || ! grep -qE "^CRASH_AMPLIFICATION_ITEM item=position-addressed-trees exit=0 .* exhaustive=false verdict=red .* registered_segments=stub@3:300 unregistered_segments_beyond_expansion=other@0:40 registered_segment_domains=stub@3:full registered_segment_progress=-\$" <<<"$output" \
     || ! grep -q "^  ✗ 分项 position-addressed-trees：展开 $FULL_EXPAND_UP_TO 是全量的展开深度，却有 1 行驱动行不是 exhaustive=true、也没按段登记（不穷举的流：other）" "$errors" \
     || ! grep -q '按段登记（.claude/gate.d/crash-point-replay-segments.tsv）或加深展开' "$errors" \
     || grep -q '✓' <<<"$output"; then
    selftest_cell_failed "一行 exhaustive=false registered_segments=- 没照旧判红在那一条流：没记红、把没登段的流也放过了、或 ✗ 点名了登了段的流（退 $exit_code）：$output" \
      "看 summarize_the_item：只有 exhaustive=false 而本行 registered_segments= 是 <段号>:<写数>[,…] 的行才不算不穷举，- 与没报的照算，按行判、不按项判"
  fi
  # 第 36 格：每一趟分项驱动都收到按段登记文件的绝对路径（SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY=<被判仓根>/.claude/gate.d/crash-point-replay-segments.tsv，
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
  # 第 37 格：按段登记文件写坏一行（摘要写成大写十六进制，lib_crash_point_replay_items.py 的 segment_problems 认的一种坏法）：整趟退 2、不清场、不起驱动，
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
  # 第 38 格：stub 那条流登了第 0 段（322 写），驱动行报 unregistered_segments_beyond_expansion=1:290（第 1 段没登、展开上限盖不住）：
  #   点名这一项、展开深度是全量的那一个：红在这一项，✗ 点名流 stub 与 1:290、出路写按段登记或加深展开，ITEM 行报 registered_segments=stub@0:322
  #   unregistered_segments_beyond_expansion=stub@1:290，整趟退 1、不打成功句；
  #   同一形态缩过规模（--expand-up-to 6，再点名一项不报这个字段的 tree-split-streams）：不红、退 0，成功句报这一段与取不出的那一项
  local beyond_scaled_output beyond_scaled_exit
  cells=$((cells + 1))
  output="$(STUB_SHAPE_crash_enumeration_position_addressed_trees=registered-beyond entry --item position-addressed-trees 2>"$errors")"; exit_code=$?
  beyond_scaled_output="$(STUB_SHAPE_crash_enumeration_position_addressed_trees=registered-beyond entry --expand-up-to 6 --item position-addressed-trees --item tree-split-streams 2>/dev/null)"
  beyond_scaled_exit=$?
  if [[ "$exit_code" != 1 ]] \
     || ! grep -qE "^CRASH_AMPLIFICATION_ITEM item=position-addressed-trees exit=0 .* exhaustive=false verdict=red .* registered_segments=stub@0:322 unregistered_segments_beyond_expansion=stub@1:290 registered_segment_domains=stub@0:full registered_segment_progress=-\$" <<<"$output" \
     || ! grep -q "^  ✗ 分项 position-addressed-trees：展开 $FULL_EXPAND_UP_TO 是全量的展开深度，有 1 行驱动行登了段，可那条流里另有没登记、展开上限盖不住的段（流 stub 段 1:290）" "$errors" \
     || ! grep -q '按段登记（.claude/gate.d/crash-point-replay-segments.tsv）或加深展开' "$errors" \
     || grep -q '✓' <<<"$output" \
     || [[ "$beyond_scaled_exit" != 0 ]] \
     || ! grep -qE "^CRASH_AMPLIFICATION_ITEM item=position-addressed-trees exit=0 .* verdict=green .* unregistered_segments_beyond_expansion=stub@1:290 registered_segment_domains=stub@0:full registered_segment_progress=-\$" <<<"$beyond_scaled_output" \
     || ! grep -q '；展开上限盖不住又没登记的段 1 段（1 项，没判全域）：position-addressed-trees 流 stub 段 1（290 写）（另有 1 项取不出：tree-split-streams，驱动行没报 unregistered_segments_beyond_expansion=）$' <<<"$beyond_scaled_output"; then
    selftest_cell_failed "登了段的流里另有展开上限盖不住又没登记的段，没判红在那条流与那几段：没记红、✗ 没点名流与段号写数、没给出路，或缩过规模的一趟 ITEM 行与成功句没报那几段（退 $exit_code，缩过规模那一趟退 $beyond_scaled_exit）：$output｜$beyond_scaled_output" \
      "看 summarize_the_item 里 unregistered_segments_beyond_expansion 那一段（登了段的行这个字段不是 - 就记下那几段）与主循环里 item_beyond_lines 那一条红因、成功句的 beyond_part"
  fi
  # 第 39 格：旧引擎打的行：stub 那条登了第 3 段、exhaustive=false，没报 unregistered_segments_beyond_expansion=：点名这一项、展开深度是全量的那一个：
  #   照不穷举红在这一项，✗ 点名流 stub，出路写「引擎没报 unregistered_segments_beyond_expansion=，重编引擎」，ITEM 行这个字段写 unknown，整趟退 1
  cells=$((cells + 1))
  output="$(STUB_SHAPE_crash_enumeration_position_addressed_trees=registered-old-engine entry --item position-addressed-trees 2>"$errors")"; exit_code=$?
  if [[ "$exit_code" != 1 ]] \
     || ! grep -qE "^CRASH_AMPLIFICATION_ITEM item=position-addressed-trees exit=0 .* exhaustive=false verdict=red .* registered_segments=stub@3:300 unregistered_segments_beyond_expansion=unknown registered_segment_domains=unknown registered_segment_progress=unknown\$" <<<"$output" \
     || ! grep -q "^  ✗ 分项 position-addressed-trees：展开 $FULL_EXPAND_UP_TO 是全量的展开深度，有 1 行驱动行登了段、exhaustive=false，却没报 unregistered_segments_beyond_expansion=（流：stub）" "$errors" \
     || ! grep -q '引擎没报 unregistered_segments_beyond_expansion=，重编引擎' "$errors" \
     || ! grep -q '^  → 登了段的行没报 unregistered_segments_beyond_expansion= 的项（position-addressed-trees）：' "$errors" \
     || grep -q '✓' <<<"$output"; then
    selftest_cell_failed "登了段的行没报 unregistered_segments_beyond_expansion=，没照不穷举判红、出路不是重编引擎（退 $exit_code）：$output" \
      "看 summarize_the_item：登了段、exhaustive=false 的行没报这个字段（或写得认不得）记进没报的行数与流名，主循环里 item_beyond_missing_lines 那一条红因与「重编引擎」那一句出路"
  fi
  # 第 40–43 格：登记段装不进 u64 时的分片枚举（文件头「按段登记」），都用假驱动形态 registered-tail，stub 那条流登记段的几个字段由 STUB_REGISTERED_TAIL 给
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
  # 第 40 格：stub 那条流登了第 0 段（322 写，域全展开）与第 1 段（290 写，域分片：这一趟为止已判 4294967296 态、域共 3^290-1 态，照十进制 139 位报），别的段都盖得住：
  #   点名这一项、展开深度是全量的那一个，不红、退 0；ITEM 行带两个字段（registered_segment_domains=stub@0:full,stub@1:slice
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
     || [[ "$success_line" != *"崩溃放量点名的 1 项（展开 $FULL_EXPAND_UP_TO）"* ]] \
     || [[ "$success_line" != *"；登记段的域（甲二 ∪ 展开 6）2 段（1 项；全展开 1 段、分片 1 段）：position-addressed-trees 流 stub 段 0：全展开、position-addressed-trees 流 stub 段 1：分片，已判 4294967296/$slice_total；"* ]] \
     || [[ "$slice_forms_exit" != 0 ]] \
     || [[ "$slice_forms_item_line" != *" verdict=green "*" registered_segment_domains=stub@0:slice,stub@2:slice registered_segment_progress=stub@0:7/18446744073709551615,stub@2:5/$slice_total_141" ]] \
     || [[ "$slice_forms_success" != *"：position-addressed-trees 流 stub 段 0：分片，已判 7/18446744073709551615、position-addressed-trees 流 stub 段 2：分片，已判 5/$slice_total_141；"* ]]; then
    selftest_cell_failed "登记段分片枚举（全展开一段、分片一段，总态数是超过 u64 的十进制串）判了红、或 ITEM 行没带 registered_segment_domains= / registered_segment_progress=、或成功句没报全展开与分片已判几态（退 $exit_code，另两种写法那一趟退 $slice_forms_exit）：$output｜$slice_forms_output" \
      "看 summarize_the_item 的 domains_problem_of_the_line（两个字段的写法、进度的两个数只认十进制数字串）与主循环里 domains_of_the_item_for_the_success_line、成功句的 domain_part"
  fi
  # 第 41 格：分片段的进度是 1:0/<那 139 位>，stub 那条这一趟现判 0 态、复用 0 态（一个态都没判，也没复用）：点名这一项，展开深度是全量的那一个与缩过规模（--expand-up-to 6）都红在这一项，
  #   ✗ 点名流 stub 与段 1，出路写「这一趟一个态都没判，看录入段与判段的日志」，整趟退 1、不打成功句
  local slice_zero_scaled_exit
  cells=$((cells + 1))
  slice_tail="registered_segments=0:322,1:290 unregistered_segments_beyond_expansion=- registered_segment_domains=0:full,1:slice registered_segment_progress=1:0/$slice_total"
  output="$(STUB_TAIL_CHECKED=0 STUB_TAIL_BLOCKS=0 STUB_TAIL_REUSED=0 tail_entry "$slice_tail" --item position-addressed-trees 2>"$errors")"; exit_code=$?
  STUB_TAIL_CHECKED=0 STUB_TAIL_BLOCKS=0 STUB_TAIL_REUSED=0 tail_entry "$slice_tail" --expand-up-to 6 --item position-addressed-trees >/dev/null 2>&1; slice_zero_scaled_exit=$?
  slice_item_line="$(grep '^CRASH_AMPLIFICATION_ITEM item=position-addressed-trees ' <<<"$output")"
  if [[ "$exit_code" != 1 || "$slice_zero_scaled_exit" != 1 ]] \
     || [[ "$slice_item_line" != *" verdict=red "*" registered_segment_progress=stub@1:0/$slice_total" ]] \
     || ! grep -qF "分项 position-addressed-trees：有 1 行驱动行的分片段已判 0 态（流 stub 段 1:0/$slice_total）：这一趟一个态都没判，看录入段与判段的日志" "$errors" \
     || ! grep -q '^  → 分片段这一趟已判 0 态的项（position-addressed-trees）：这一趟一个态都没判，看录入段与判段的日志' "$errors" \
     || grep -q '✓' <<<"$output"; then
    selftest_cell_failed "分片段这一趟已判 0 态却没判红在那条流与那一段、或出路不是看录入段与判段的日志（退 $exit_code，缩过规模那一趟退 $slice_zero_scaled_exit）：$output" \
      "看 summarize_the_item 里分片进度那一段（已判是 0 的段记进 zero）与主循环里 item_zero_lines 那一条红因（不看展开深度）、slice_zero_failed_names 那一句出路"
  fi
  # 第 42 格：登了段的行，两个字段没报全或写法认不得（四趟：两个都缺，即分片之前的引擎；只缺 registered_segment_progress=；域写成 1:partial；
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
  # 第 43 格：登了第 0、1 段，registered_segment_domains= 漏了段 1（0:full），或段 1 是分片而 registered_segment_progress= 没列它（-）：
  #   两趟都红在这一项，✗ 与出路同第 42 格，ITEM 行两个字段写 unknown，整趟退 1
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
  # 第 44 格：按登记不判的 ignored_points= / states_skipped_by_ignore= 按流分组、每组只取一次（文件头「按登记整批不判」），点名 tree-split-streams 一项，五趟：
  #   同一条流 sigma 的本机份、第二台份、导入三行（各报 1 个点 869269834 态）→ 1 个点、869269834 态；叫停再续的四行（同流）→ 同样 1 个点、869269834 态；
  #   两条流各一份（1/10 与 2/20，a 流两行同报一份）→ 3 个点、30 态；两行都不带 flow=（归一组，各报 2/50）→ 2 个点、50 态。
  #   每趟 ITEM 行与 TOTAL 行钉绝对值，退 0、成功句报同样的数。第五趟同流两行报的不同（a 流 1/10 与 1/11，b 流 2/20）：ITEM 两个数写 unknown、
  #   TOTAL 的 ignore_unknown=1 且不加这一项，标准错误与成功句点名 a 流，退 0（不红）
  local ignore_wrong="" ignore_run=0 ignore_shape ignore_expected_points ignore_expected_states
  cells=$((cells + 1))
  for ignore_shape in 'ignored-three-runs 1 869269834' 'ignored-resumed-four 1 869269834' 'ignored-two-flows 3 30' 'ignored-no-flow-twice 2 50'; do
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
  # 第 45 格：全部复用的一趟（文件头「按段登记」）：stub 那条流这一趟现判 0 态、0 块，复用 7 态，分片段的进度是 1:0/<那 139 位>：
  #   分片段已判 0 态只在现判与复用都是 0 时红，这一趟不红：点名这一项、展开深度是全量的那一个，ITEM 行 states=3（只有 other 那条现判）
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
  # 第 46 格：分项驱动退 0、打了一行 CRASH_AMPLIFICATION_NOT_RUN step=judge（不带 gpu 特性的二进制遇到多机配置 ENABLE_GPU=1：判段这一趟不跑、录入段照录）、
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
  # 第 47 格：复用三问（scripts/gate/lib_reuse_checks.py 文件头「一」）：假驱动 library 形态照 $STUB_UNITS/<测试目标> 的内容摘要按键记判过没有，
  #   点名三项各跑四趟：头一趟、同一份输入、加一份哪个测试目标都不读的单元文件（闭包外）、改 fixed-script-through-rollback 那个测试目标的单元文件（闭包内）；
  #   每一趟从 ITEM 行取这一项复用还是现判（states=0 而 reused 是正整数是复用），照三问判、打规范输出。入口自己不判复用（判定在引擎里）：
  #   这一格核的是它每一趟把同一份库交给驱动、不因闭包外的改动清库，并照实把驱动报的现判与复用写进 ITEM 行
  cells=$((cells + 1))
  local three_units="$work/stub-units" three_name three_items=(fixed-script-through-rollback multi-record-publish tree-split-streams) three_arguments=()
  mkdir -p "$three_units"
  for entry_row in "${ITEMS[@]}"; do printf 'unit of %s\n' "$(item_fields "$entry_row" 2)" >"$three_units/$(item_fields "$entry_row" 2)"; done
  for three_name in "${three_items[@]}"; do three_arguments+=(--item "$three_name"); done
  three_round() { # <名>：跑一趟，ITEM 行折成「分项<TAB>复用|现判」写进 $work/three-<名>.tsv
    STUB_SHAPE=library STUB_UNITS="$three_units" entry "${three_arguments[@]}" >"$work/three-$1.out" 2>/dev/null
    awk '/^CRASH_AMPLIFICATION_ITEM / { item = ""; states = ""; reused = ""
           for (i = 2; i <= NF; i++) { split($i, pair, "="); if (pair[1] == "item") item = pair[2]; if (pair[1] == "states") states = pair[2]; if (pair[1] == "reused") reused = pair[2] }
           print item "\t" ((states == "0" && reused ~ /^[1-9][0-9]*$/) ? "复用" : "现判") }' "$work/three-$1.out" >"$work/three-$1.tsv"
  }
  three_round first; three_round second
  printf '哪个测试目标都不读\n' >"$three_units/not-read-by-any-target"; three_round unrelated
  printf 'unit of crash_amplification_fixed_script_through_rollback，改过\n' >"$three_units/crash_amplification_fixed_script_through_rollback"; three_round unit
  if ! python3 "$repository_root/scripts/gate/lib_reuse_checks.py" three-questions --script scripts/crash/crash-amplification.sh --first "$work/three-first.tsv" \
         --second "$work/three-second.tsv" --unrelated "$work/three-unrelated.tsv" --unit-change "$work/three-unit.tsv" --affected fixed-script-through-rollback >"$work/three.log" 2>&1; then
    selftest_cell_failed "复用三问没过：$(tr '\n' ' ' <"$work/three.log")" "照规范输出那一行 red 的那一问查：ITEM 行的 states=、reused= 取法（文件头「复用数怎么取」），与每一趟交给驱动的库家目录"
  fi
  grep '^REUSE_SELFTEST ' "$work/three.log"
  # 第 48 格：上次红的清单（scripts/gate/lib_reuse_checks.py 文件头「三」）：点名 multi-record-publish 红一趟，清单是它；再点名 fixed-script-through-unmount 绿一趟，清单照留它；
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
  table="$(crash_amplification_read_switch_table "$switch_table_source" CRASH_AMPLIFICATION_BREAK GPU_QUIESCE_BREAK)"
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
  echo "  ✓ --selftest：列项带档位、拒不认得的名与缺项名、全量带展开深度、点名取并集、清场次序、清场失败退 78、跑完与有项红都不复原、驱动退非 0 与 red>0 判红、复用数的取法与取不出照实报、两台的行、派出收回、0 行判红、全量不穷举判红（不带参数的全量里红在不穷举的那一项）、缩过规模照实写并点名不穷举的项、展开深度对不上判红、叫停与没判完报本次未跑与 77、驱动退 78 停下、平常跑拒只供自证的四个变量并逐个点名、有红又有项退 78 时退 1、退 78 而行里报红记红、按登记整批不判报数与取不出、按登记整批不判的两个数按流分组每组只取一次（同流各行重复报不重复计，不同的流各取一次相加，同流各行不同记取不出并点名流）、red= 缺或坏判红、全量零态判红、复用数每项只取一次（先判着、导入、计划行不重复计）、两台分份的 flow= 行留的块同一组有导入行才不算、分项驱动退 77 记本次未跑（全是它时退 77）、几条流里有的没借到显存记本次未跑不算通过、只因登记的段不穷举不判红并报段号与写数（没登段的流照旧红）、每一趟往驱动送按段登记文件、登记文件写坏整趟拒跑、登了段的流里另有展开上限盖不住又没登记的段红在那条流与那几段（缩过规模时成功句报那几段）、驱动行没报那个字段照不穷举红（出路重编引擎）、登记段分片枚举不红而 ITEM 行与成功句报全展开与分片已判几态（总态数是超过 u64 的十进制串）、分片段已判 0 态而现判与复用都是 0 红在那条流与那一段、登了段的行登记段的域与分片进度没报全漏段或写坏照重编引擎红、全部复用的一趟不红、分项驱动报判段没跑（step=judge）记本次未跑，$cells 格都对；开关轮 ${#switch_wheel_assignments[@]} 个开关（文件头现读）逐个打开，派出 ${#switch_wheel_assignments[@]} 个、收回 $switch_wheel_collected 个，各红在点名的那几格"
  exit 0
}
if (( selftest )); then run_selftest; fi

# 平常跑：只供自证的五个变量有一个非空就拒（自证起的子进程除外，文件头「只供自证」）
if (( ! running_under_selftest )) && ! crash_amplification_break_is honor-test-variables-outside-selftest; then
  test_only_given=()
  [[ -n "$crash_amplification_break_switches" ]] && test_only_given+=(CRASH_AMPLIFICATION_BREAK)
  [[ -n "$selftest_red_item" ]] && test_only_given+=(CRASH_AMPLIFICATION_SELFTEST_RED_ITEM)
  [[ -n "$selftest_drop_result" ]] && test_only_given+=(CRASH_AMPLIFICATION_SELFTEST_DROP_RESULT)
  [[ -n "$test_item_driver" ]] && test_only_given+=(CRASH_AMPLIFICATION_ITEM_DRIVER)
  [[ -n "$selftest_segments_root" ]] && test_only_given+=(CRASH_AMPLIFICATION_SELFTEST_SEGMENTS_ROOT)

  if (( ${#test_only_given[@]} > 0 )); then
    echo "  ✗ 平常跑时带了只给自证用的变量：${test_only_given[*]}；拒跑，一项都没跑" >&2
    echo "  → 怎么办：unset ${test_only_given[*]} 再跑；要证红就跑 $0 --selftest（它起的子进程才认这几个变量）" >&2
    exit 2
  fi
fi
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
      if crash_amplification_break_is accept-unknown-item; then found="$name"$'\t'"$name"$'\t'"$name"$'\t'全量; else
        refuse_listing_the_items "不认得的分项：$name"
      fi
    fi
    rows_to_run+=("$found")
  done
fi

# 展开深度与这一趟的范围（文件头「全量的展开深度」）
requested_expand_up_to="${expand_up_to_given:-$FULL_EXPAND_UP_TO}"
if [[ "$requested_expand_up_to" != "$FULL_EXPAND_UP_TO" ]]; then scope=scaled
elif (( ${#selected[@]} > 0 )); then scope=items
else scope=full; fi
driver_arguments+=(--expand-up-to "$requested_expand_up_to")

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

# 清场（文件头「清场」）：在清场包装底下（GPU_QUIESCE_HOLDER 指到包装登记的文件，文件名以父进程号加「-」起头）每项带 --quiesced；
#   不在就经它带同一组参数重起自己，交回它的退出码（清场回读没过 78，别的是重起那一趟的）；跑完不复原
quiesced_arguments=()
if [[ -n "${GPU_QUIESCE_HOLDER:-}" && -e "${GPU_QUIESCE_HOLDER:-}" && "${GPU_QUIESCE_HOLDER##*/}" == "$PPID-"* ]]; then
  quiesced_arguments=(--quiesced)
  echo "  · 清了场（经 scripts/crash/gpu-quiesce-run.sh）：每一项带 --quiesced 跑，跑完不复原（要复原手动跑私有复原脚本，多机配置的 QUIESCE_START_COMMAND）"
elif ! crash_amplification_break_is no-quiesce; then
  reexec_environment=()
  if (( running_under_selftest )); then
    reexec_environment+=("CRASH_AMPLIFICATION_SELFTEST_MARK=$selftest_mark_file")
    [[ -n "$crash_amplification_break_switches" ]] && reexec_environment+=("CRASH_AMPLIFICATION_BREAK=$crash_amplification_break_switches")
    [[ -n "$selftest_red_item" ]] && reexec_environment+=("CRASH_AMPLIFICATION_SELFTEST_RED_ITEM=$selftest_red_item")
    [[ -n "$selftest_drop_result" ]] && reexec_environment+=("CRASH_AMPLIFICATION_SELFTEST_DROP_RESULT=$selftest_drop_result")
    [[ -n "$test_item_driver" ]] && reexec_environment+=("CRASH_AMPLIFICATION_ITEM_DRIVER=$test_item_driver")
    [[ -n "$selftest_segments_root" ]] && reexec_environment+=("CRASH_AMPLIFICATION_SELFTEST_SEGMENTS_ROOT=$selftest_segments_root")
  fi
  reexec_arguments=(${entry_original_arguments[@]+"${entry_original_arguments[@]}"})
  if [[ "${PREFLIGHT_FORCE_GIVEN:-0}" == 1 ]]; then reexec_arguments+=(--force); fi
  echo "  · 经 scripts/crash/gpu-quiesce-run.sh 清场，带同一组参数重起这一趟，跑完不复原" >&2
  exec env ${reexec_environment[@]+"${reexec_environment[@]}"} bash "$crash_amplification_script_directory/gpu-quiesce-run.sh" -- bash "$0" ${reexec_arguments[@]+"${reexec_arguments[@]}"}
fi

# 每项的输出另存一份：边跑边转打，跑完从文件取汇总
crash_amplification_home="${SINGLEFS_CRASH_AMPLIFICATION_HOME:-$HOME/.local/share/singlefs/crash-amplification}"
entry_run_directory="$crash_amplification_home/runs/entry-$(date -u +%Y%m%dT%H%M%SZ)-$$"
if ! mkdir -p -- "$entry_run_directory"; then
  echo "  ✗ 建不了这一趟放每项输出的目录 $entry_run_directory，一项都没跑" >&2
  echo "  → 怎么办：修好设备，或改配置：看 $crash_amplification_home 可不可写，或把 SINGLEFS_CRASH_AMPLIFICATION_HOME 指到可写、开机不清空的目录（退 78）" >&2
  exit 78
fi
if [[ -n "${SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY:-}" ]]; then
  echo "  · 往引擎送按段登记文件 SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY=$SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY（$(grep -cvE '^(#|[[:space:]]*$)' "$SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY") 行登记）" >&2
fi

started_at=$(date +%s)
items_dispatched=0; items_run=0; items_failed=0; items_not_run=0; items_reuse_unknown=0
reuse_unknown_items=""; not_run_items=""; not_run_entries=(); failed_entries=(); failed_names=(); last_run_ran=()
total_states=0; total_reused=0; total_red=0; stopped_by_condition=""
total_ignored_points=0; total_states_skipped_by_ignore=0; items_ignore_unknown=0; ignore_unknown_items=""; ignored_items=""; ignore_conflict_seen=0
items_with_lines=0; items_not_exhaustive=0; not_exhaustive_items=""; not_exhaustive_failed_names=(); engine_field_missing_failed_names=()
registered_items=0; registered_segment_count=0; registered_entries=""
beyond_items=0; beyond_segment_count=0; beyond_entries=""; beyond_unknown_items=0; beyond_unknown_names=""
domain_field_missing_failed_names=(); slice_zero_failed_names=()
domain_segment_count=0; domain_full_count=0; domain_slice_count=0; domain_items=0; domain_entries=""
for row in "${rows_to_run[@]}"; do
  name="$(item_fields "$row" 1)"; target="$(item_fields "$row" 2)"; function_name="$(item_fields "$row" 3)"
  # 自证用：把测试目标改成带 -red 的名字，让假驱动退 1
  [[ -n "$selftest_red_item" ]] && target="${target}-red"
  filter_arguments=(--test-filter "$function_name")
  crash_amplification_break_is skip-filter && filter_arguments=()
  item_output_file="$entry_run_directory/$name.log"
  echo "== 分项 $name（$target::$function_name，展开 $requested_expand_up_to）$(date -u +%FT%TZ)；输出另存 $item_output_file"
  item_started_at=$(date +%s)
  items_dispatched=$((items_dispatched + 1))
  bash "$driver" "$target" "${filter_arguments[@]+"${filter_arguments[@]}"}" "${driver_arguments[@]+"${driver_arguments[@]}"}" "${quiesced_arguments[@]+"${quiesced_arguments[@]}"}" 2>&1 \
    | tee -- "$item_output_file"
  pipe_status=("${PIPESTATUS[@]}"); item_exit="${pipe_status[0]}"
  crash_amplification_break_is swallow-exit && item_exit=0
  read -r item_lines item_states item_red item_reused item_exhaustive item_expand item_stopped item_unjudged item_bad_red item_ignored item_skipped \
    item_not_exhaustive_lines item_not_exhaustive_flows item_registered_segments item_beyond_lines item_beyond_segments item_beyond_missing_lines item_beyond_missing_flows \
    item_beyond_report item_domains item_progress item_domain_bad_lines item_domain_bad_flows item_zero_lines item_zero_segments item_ignore_conflict_flows <<<"$(summarize_the_item "$item_output_file")"
  item_lines="${item_lines:-0}"; item_states="${item_states:-0}"; item_red="${item_red:-0}"; item_reused="${item_reused:-unknown}"
  item_exhaustive="${item_exhaustive:-false}"; item_expand="${item_expand:-unknown}"; item_stopped="${item_stopped:-0}"; item_unjudged="${item_unjudged:-0}"
  item_bad_red="${item_bad_red:-0}"; item_ignored="${item_ignored:-unknown}"; item_skipped="${item_skipped:-unknown}"
  # 不穷举又没按段登记的行数与流名（文件头「全量的展开深度」「按段登记」）、按段登记的段
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
    if [[ "$scope" == full && "$item_states" == 0 && "$item_reused" == 0 ]] && (( item_stopped == 0 && item_unjudged == 0 && item_driver_not_run == 0 )) \
       && ! crash_amplification_break_is accept-zero-full-item; then
      reasons+=("不带参数的全量里这一项现判 0 态、复用 0 态：一态都没判到不是通过；看 --test-filter $function_name 筛中的用例是不是空跑，驱动行的 states_checked_here 与 states_reused 有没有照实报")
    fi
    if [[ "$item_exit" == 0 ]] && (( item_lines == 0 && item_driver_not_run == 0 )) && ! crash_amplification_break_is accept-zero-lines; then
      reasons+=("驱动退 0 却 0 行 CRASH_AMPLIFICATION：看测试名还在不在（$target）、cargo 是不是筛出了 0 条（--test-filter $function_name）")
    fi
    # 不穷举（文件头「全量的展开深度」「按段登记」）：展开深度是全量的那一个（不带参数的全量、点名都算）时有一行不是 exhaustive=true、
    #   又不是「exhaustive=false 只因登记了段」（registered_segments= 不是 -、unregistered_segments_beyond_expansion= 是 -），这一项红。
    #   弄坏开关 skip-not-exhaustive-in-full-scope 下不带参数的全量不判（点名的照判）
    full_depth_judged=0
    if (( item_lines > 0 )) && [[ "$requested_expand_up_to" == "$FULL_EXPAND_UP_TO" ]] && ! crash_amplification_break_is accept-not-exhaustive \
       && ! { [[ "$scope" == full ]] && crash_amplification_break_is skip-not-exhaustive-in-full-scope; }; then
      full_depth_judged=1
    fi
    if (( full_depth_judged && item_not_exhaustive_lines > 0 )); then
      reasons+=("展开 $FULL_EXPAND_UP_TO 是全量的展开深度，却有 $item_not_exhaustive_lines 行驱动行不是 exhaustive=true、也没按段登记（不穷举的流：$item_not_exhaustive_flows）：全量要每一行都穷举或只因登记的段不穷举，这一项没判完全域；按段登记（.claude/gate.d/crash-point-replay-segments.tsv）或加深展开")
      not_exhaustive_failed_names+=("$name")
    fi
    # 登了段的行（文件头「按段登记」）：unregistered_segments_beyond_expansion= 不是 - 判红、点名流与那几段；没报（旧引擎）照不穷举判红，出路重编引擎
    if (( full_depth_judged && item_beyond_lines > 0 )); then
      reasons+=("展开 $FULL_EXPAND_UP_TO 是全量的展开深度，有 $item_beyond_lines 行驱动行登了段，可那条流里另有没登记、展开上限盖不住的段（$(segments_by_flow "$item_beyond_segments")）：那几段没判全域；按段登记（.claude/gate.d/crash-point-replay-segments.tsv）或加深展开")
      [[ " ${not_exhaustive_failed_names[*]-} " == *" $name "* ]] || not_exhaustive_failed_names+=("$name")
    fi
    if (( full_depth_judged && item_beyond_missing_lines > 0 )); then
      reasons+=("展开 $FULL_EXPAND_UP_TO 是全量的展开深度，有 $item_beyond_missing_lines 行驱动行登了段、exhaustive=false，却没报 unregistered_segments_beyond_expansion=（流：$item_beyond_missing_flows）：判不出登记段之外还有没有展开上限盖不住的段，照不穷举判；引擎没报 unregistered_segments_beyond_expansion=，重编引擎")
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
    if (( item_lines > 0 )) && [[ "$item_expand" != "$requested_expand_up_to" ]] && ! crash_amplification_break_is accept-expand-mismatch; then
      reasons+=("驱动行报的展开深度是 $item_expand，交下去的是 $requested_expand_up_to：驱动没照给的展开深度跑")
    fi
    if (( ${#reasons[@]} > 0 )); then
      verdict=red
    elif (( item_driver_not_run )); then
      verdict=not-run
      if (( item_exit_not_run && item_driver_not_run_lines > 0 )); then
        not_run_reason="分项驱动退 77（一项都没跑到），报本次未跑（CRASH_AMPLIFICATION_NOT_RUN step=${item_driver_not_run_step:-没写}）：借不到显存，这一项没判成（${item_driver_not_run_reason:-那一行没写 reason=}）"
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
  if (( item_lines > 0 )); then
    items_with_lines=$((items_with_lines + 1))
    if [[ "$item_exhaustive" != true ]]; then items_not_exhaustive=$((items_not_exhaustive + 1)); not_exhaustive_items="${not_exhaustive_items:+${not_exhaustive_items}、}${name}"; fi
  fi
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
echo "CRASH_AMPLIFICATION_TOTAL dispatched=$items_dispatched items=$items_run failed=$items_failed not_run=$items_not_run reuse_unknown=$items_reuse_unknown states=$total_states reused_known=$total_reused red=$total_red expand_up_to=$requested_expand_up_to scope=$scope seconds=$(( $(date +%s) - started_at )) ignored_points=$total_ignored_points states_skipped_by_ignore=$total_states_skipped_by_ignore ignore_unknown=$items_ignore_unknown"
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
    echo "  → 不穷举的项（${not_exhaustive_failed_text%、}）：按段登记（.claude/gate.d/crash-point-replay-segments.tsv）或加深展开。展开上限盖不住、整域又展不开的段（几百写的段）登进那份表，那几段只按甲二 ∪ 展开 6 判、成功句报段号与写数（登哪几段由用户点头）；盖得住的先试 $0 --expand-up-to <比 $FULL_EXPAND_UP_TO 大的 N>$(printf ' --item %s' "${not_exhaustive_failed_names[@]}")（缩过规模的一趟，成功句点名还不穷举的项），都穷举了把文件头登记的 FULL_EXPAND_UP_TO 改成那个 N 再整趟跑；或查 ✗ 里点名的流是哪一段的写数超过 $FULL_EXPAND_UP_TO（写表是不是改长了）" >&2
  fi
  if (( ${#engine_field_missing_failed_names[@]} > 0 )); then
    engine_field_missing_text="$(printf '%s、' "${engine_field_missing_failed_names[@]}")"
    echo "  → 登了段的行没报 unregistered_segments_beyond_expansion= 的项（${engine_field_missing_text%、}）：引擎没报 unregistered_segments_beyond_expansion=，重编引擎（驱动行要在 registered_segments= 之后报这条流里没登记、展开上限盖不住的段，crates/singlefs-checker-tier/src/crash_amplification.rs 打驱动行那一处）；重编之后只重跑这几项：$0$(printf ' --item %s' "${engine_field_missing_failed_names[@]}")" >&2
  fi
  if (( ${#domain_field_missing_failed_names[@]} > 0 )); then
    domain_field_missing_text="$(printf '%s、' "${domain_field_missing_failed_names[@]}")"
    echo "  → 登了段的行没报全 registered_segment_domains= / registered_segment_progress= 的项（${domain_field_missing_text%、}）：引擎没报 registered_segment_domains= / registered_segment_progress=，重编引擎（驱动行要在 unregistered_segments_beyond_expansion= 之后报每个登记段的域 <段号>:<full|slice> 与每个分片段的 <段号>:<已判态数>/<域的总态数>，crates/singlefs-checker-tier/src/crash_amplification.rs 打驱动行那一处）；重编之后只重跑这几项：$0$(printf ' --item %s' "${domain_field_missing_failed_names[@]}")" >&2
  fi
  if (( ${#slice_zero_failed_names[@]} > 0 )); then
    slice_zero_text="$(printf '%s、' "${slice_zero_failed_names[@]}")"
    echo "  → 分片段这一趟已判 0 态的项（${slice_zero_text%、}）：这一趟一个态都没判，看录入段与判段的日志（那一项的输出 $entry_run_directory/<分项>.log 里录入段与判段那几趟）；修好之后只重跑这几项：$0$(printf ' --item %s' "${slice_zero_failed_names[@]}")${expand_up_to_given:+ --expand-up-to $expand_up_to_given}" >&2
  fi
  echo "  → 怎么办：看那一项的输出（另存在 $entry_run_directory/<分项>.log）与它驱动日志目录里的 placement.log / run.log（动用了第二台的是 share-0.log / share-1.log / import.log）；修好之后只重跑判红的那几项：$0$(printf ' --item %s' "${failed_names[@]}")${expand_up_to_given:+ --expand-up-to $expand_up_to_given}" >&2
  run_verdict=1
fi
if [[ -n "$stopped_by_condition" ]]; then
  echo "  ✗ 分项 $stopped_by_condition 的驱动退 78（显卡或第二台用不上，或清场、多机配置不满足）：这一趟停在这一项，后面 $(( ${#rows_to_run[@]} - items_dispatched )) 项没派" >&2
  echo "  → 怎么办：修好设备，或改配置：看那一项的输出（$entry_run_directory/$stopped_by_condition.log）里拒绝与出路那几行；修好之后重跑同一条命令（判过的块在库里，接着判）" >&2
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
  full) if (( items_not_run > 0 )); then success_head="崩溃放量（展开 $requested_expand_up_to）"; else success_head="崩溃放量全量（展开 $requested_expand_up_to）"; fi ;;
  items) success_head="崩溃放量点名的 $items_run 项（展开 $requested_expand_up_to）" ;;
  *) success_head="崩溃放量缩过规模：展开 $requested_expand_up_to" ;;
esac
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
# 缩过规模的一趟点名哪几项不穷举（文件头「全量的展开深度」与「输出」的成功句）；别的范围里不穷举已判红，到不了这里
scaled_part=""
if [[ "$scope" == scaled ]] && ! crash_amplification_break_is hide-scaled-not-exhaustive; then
  if (( items_not_exhaustive > 0 )); then
    scaled_part="不穷举 $items_not_exhaustive 项：${not_exhaustive_items}（展开 $requested_expand_up_to 盖不住它们最长的一段，这几项没判完全域）"
  else
    scaled_part="有驱动行的 $items_with_lines 项都穷举"
  fi
fi
# 按段登记的一段（文件头「按段登记」）：登了段的项逐项、逐条流报段号与写数（那几段只按甲二 ∪ 展开 6 判，全量展开的态数不报）
if (( registered_segment_count > 0 )); then
  registered_part="按段登记 $registered_segment_count 段（$registered_items 项，只按甲二 ∪ 展开 6 判、没判全域）：$registered_entries"
else
  registered_part="按段登记：0 段"
fi
crash_amplification_break_is registered-segments-unreported && registered_part=""
# 登记段的域的一段（文件头「按段登记」）：逐段报全展开，或分片、这一趟为止已判几态 / 域的总态数（两个数都是十进制数字串，原样打）
if (( domain_segment_count > 0 )); then
  domain_part="登记段的域（甲二 ∪ 展开 6）$domain_segment_count 段（$domain_items 项；全展开 $domain_full_count 段、分片 $domain_slice_count 段）：$domain_entries"
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
echo "  ✓ $success_head：$run_part${scaled_part:+；$scaled_part}；现判 $total_states 态、$reuse_part；派出 $items_dispatched 项、收回 $items_run 项${ignore_part:+；$ignore_part}${registered_part:+；$registered_part}${domain_part:+；$domain_part}${beyond_part:+；$beyond_part}"
exit 0
