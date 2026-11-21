#!/usr/bin/env bash
# admission: always 每一趟照此刻的根、变异表与判定库现算每个变体的键、现查 prune-server 里的按项判定（long_run_item_verdicts.py）；上一趟的结论只经判定库按键复用，产物与草稿目录不替这一趟作保
# run-condition: command cargo python3 nice git rsync
# E165（恢复重放有界穷举与变异杀伤界）第二段：量变异表里动 replay_journal 那五个函数的每个变体在小界内红不红、首红的 N。
# 登记：research/prompts/e165-preregistration.md 5.5 第二段、5.6 的界、第九节甲表、第十二节修订第 10、11、12 条；
#   第 1 次重跑 research/prompts/e165-r1-prereg.md 5.5 第二段、第九节（判定搬进 crates/singlefs-replay-prefix/src/lib.rs 之后，
#   选行按变异名与它改的判定、不按行号；改的文件取变异行第二列）；
#   第 2 次重跑 research/prompts/e165-r2-prereg.md 5.6 第二段、第九节 9.1–9.5（这份驱动只为 r2 档写）：基线与每个变体调装置都带
#   --profile r2；条款判据栏加 J6Qn 两行与 J7 五行；每个变体逐域报 J6 与 J6Qn 各自的首红（first_red_by_criterion，SQ2）与
#   首个文字不同的 N 上不同的判据名（first_text_red_criteria，SQ5 要的 Z5 那一格）；自造表 Z1–Z5；红例账照落，写完当场钉进钉表、当场核钉全（「钉红例」一段）。
#   r1 档那份第二段要复跑，用 git 里 r1 那份驱动与 r1 冻结的根（r2 登记第十一节「新旧产物怎么并存」）。
#   第 3 次重跑 research/prompts/e165-r3-prereg.md 9.3（第三段，加 --profile r3 模式；不给 --profile 仍是 r2 那一套，一个字不改）：
#   基线与每个变体调装置都是 --profile r3 --bounds segment1 --family root_choice；变体名单是 9.1 那六个名字（1822–1827，按名字选行，
#   改的文件取变异行第二列，原文命中处要落在 choose_root_slot 或 choose_root 里）；每个变体逐 (域, M, U) 比 KJ0–KJ3 的违例数与
#   root_choice_states 行，全跑不早停；产物每个变体一行（first_red_by_criterion、每格违例数 cells、ka_kj2 与 ka_pin）；红例账不落（9.3 第 5 条）。
#   r3 模式要 --root 指第二段冻结根（REGISTERED_R3 钉着它的 recovery.rs、lib.rs、crates/mutations.tsv 的 sha256 与两个函数的区间），
#   --measure 用的内存包装与第二段产物（TV7 的比对对象）取脚本所在的仓、不取 --root（冻结根里只有 Cargo.toml、Cargo.lock、.cargo、crates/）。
#
#   e165-replay-mutation-kill-bounds.sh --profile r3 --root <第二段冻结根> --scratch <草稿目录> --output <产物> [--item <变体>]… [--jobs <并行度>] [--threads <每个变体的线程数>（只给自证）]
#       r3 模式整轮：开头基线（核 criteria=root_choice 判决行在、与第二段产物 E7RESULT 逐行相同，不同退 6）、每个变体一份副本各编一次、收尾基线；
#       7 个变体都有行、收尾基线与开头逐字相同，才写收尾三行；不给 --ledger（给了退 2）；r3 模式不认 --recheck、--select-rows、--measure（退 2）。
#       选行 4 条（恰 1 行、文件列、原文命中 1 次、命中处落在两个函数里）有一条不满足，那一行记 outcome=not_compared（登记 TH5），别的行照做，整轮退 1。
#       变体在 KA 上 KJ2 的违例数不等于登记钉值（1823 → 18、1824 → 52、1825 → 36、1826 → 18）记 ka_pin=mismatch，整轮跑完退 1（TF8）；
#       1822、1827 的 0、18 是登记里推的，不钉，报 ka_inferred=match|mismatch，不影响退出码。
#   e165-replay-mutation-kill-bounds.sh --profile r3 --root <第二段冻结根> --list-items
#       列 7 行（基线 1 个、变体 6 个），不编不跑。
#
#   第 4 次重跑 research/prompts/e165-r4-prereg.md 9.3（第二段，加 --profile r4 模式；不给 --profile 仍是 r2 那一套，--profile r3 仍是 r3 那一套，一个字不改）：
#   基线与每个变体调装置都是 --profile r4 --bounds segment1 --family root_choice_faults；变体名单是 9.1 那六个名字（登记名 v1822–v1827，r3 的编号当名字用，在
#   根里的 crates/mutations.tsv 按名字选行，改的文件取变异行第二列，原文命中处要落在 choose_root_slot 或 choose_root 里）；每个变体逐 (域, M, U, 被判时故障,
#   历史标签) 比 KJ0、KJ1、KJ1T、KJ2、KJ3 的违例数与 root_choice_fault_states 行，全跑不早停；产物每个变体一行（first_red_by_criterion 先 KJ1、KJ1T、KJ2 再 KJ0、KJ3，
#   每格违例数合计 cells、ka_kj2 与 ka_pin、kg_pin、differs_on）；红例账不落。四个合法域任一子格上 KJ1、KJ1T、KJ2 与基线不同才记 red；只在 KG 或 KA 上与基线不同记
#   not_red_within_bounds 并报 differs_on=KG|KA|KG+KA。变体在 KA 上 KJ2 的违例数不等于登记钉值（v1823 → 18、v1824 → 52、v1825 → 36、v1826 → 18）记 ka_pin=mismatch，
#   在 KG 上每个 (M, U) 格的 KJ1、KJ1T、KJ2 合计不等于登记第七节「本地择根序」表里它那一列（v1827 取 v1823 那一列）记 kg_pin=mismatch，整轮跑完退 1（TF7）；
#   v1822、v1827 的 KA 读数 0、18 是登记里推的，不钉，报 ka_inferred=match|mismatch，KG 那一列的核不管钉没钉都做（kg_pin）。
#   r4 模式要 --root 指第一段根（R4_REGISTERED 钉着它的 recovery.rs、lib.rs、crates/mutations.tsv 的 sha256 与两个函数的区间），--measure 用的内存包装与第一段产物（TV7 的比对对象）取脚本所在的仓、
#   不取 --root；基线的 E7RESULT 行必须与第一段产物逐行相同（不同退 6）、criteria=root_choice_faults 的判决行要在（缺了退 7）；整遍峰值 RSS 约 1.7 GB，
#   同时跑的变体数 --jobs 与每个变体的线程数 --threads 见「并行」一段；内存额度整条命令交 task-server（见「内存」一段）。
#   e165-replay-mutation-kill-bounds.sh --profile r4 --root <第一段根> --scratch <草稿目录> --output <产物> [--item <变体>]… [--jobs <并行度>] [--threads <每个变体的线程数>（只给自证）]
#       r4 模式整轮：开头基线、每个变体一份副本各编一次、收尾基线；7 个变体都有行、收尾基线与开头逐字相同，才写收尾三行；不给 --ledger（给了退 2）；
#       r4 模式不认 --recheck、--select-rows、--measure（退 2）；选行 4 条有一条不满足，那一行记 outcome=not_compared（登记 TH5），别的行照做，整轮退 1。
#   e165-replay-mutation-kill-bounds.sh --profile r4 --root <第一段根> --list-items
#       列 7 行（基线 1 个、变体 6 个），不编不跑。
#   下面五项是 r4 登记 9.3 第 6 条那张表的弄坏开关，外面打开时自证同样红（自证的 r4 那几趟把外面给的这五项原样传进去，别的趟不带）：
#   E165_KILL_BOUNDS_BREAK=r4itemsmissing  r4 名单少一条
#       「r4：--list-items 应当列 7 行（基线 1、变体 6）」（实际 6 行）
#       连带红的格：「✗ 自检：r4 整轮：收尾行」、「✗ 自检：r4：KA 读数等于钉值、KG 读数等于那一列的变体记 ka_pin=match、kg_pin=match；推的 v1822、v1…」、「✗ 自检：r4：变异名选不到的那一行」
#   E165_KILL_BOUNDS_BREAK=r4firstredswap  首红那一栏里 KJ1T 与 KJ2 两列对调
#       「r4：假变体 KJ1T 在 (1, 2, 无, 罩着)、KJ2 在 (1, 0, RN, 罩着) 与基线不同应当报 KJ1T:KF=M1U2/none/inside、KJ2:KF=M1U0/RN/inside」（实际两列互换）
#   E165_KILL_BOUNDS_BREAK=r4kgcountsasred  KG 当合法域：只在 KG 与基线不同的变体也记 red
#       『r4：假变体只在 KG 与基线不同应当记 not_red_within_bounds、differs_on=KG』（实际 red）
#       连带红的格：「✗ 自检：r4：假变体只在 KG 与基线不同（读数换成它自己那一列）」、「✗ 自检：r4：KA 读数等于钉值、KG 读数等于那一列的变体记 ka_pin=match、kg_pin=match；推的 v…、v… 不钉…」、「✗ 自检：r4：变体跑到装置自己退 1（halt 行）而 KF 域 KJ0 变红」
#   E165_KILL_BOUNDS_BREAK=r4kgpinunchecked  不核 KG 钉值（读数不等于那一列也不报 mismatch、不退 1）
#       「r4：假变体的 KG 读数不等于它那一列应当报 kg_pin=mismatch 且整轮退 1」（实际退 0）
#       连带红的格：「✗ 自检：r4：KA 读数等于钉值、KG 读数等于那一列的变体记 ka_pin=match、kg_pin=match；推的 v1822、v1…」、「✗ 自检：r4：假变体只在 KG 与基线不同（读数换成它自己那一列）」、「✗ 自检：r4：变异名选不到的那一行」
#   E165_KILL_BOUNDS_BREAK=r4noprofile  调装置不带 --profile r4
#       「r4 整轮应当退 0」（实际退非 0：假装置不认没有 --profile r4 的 --family root_choice_faults）、「r4 整轮里假装置每一次被调都应当带 --profile r4」
#       连带红的格：「✗ 基线没跑完或退出码不是 0（2；日志 …）」、「✗ 自检：r4 整轮：收尾行」、「✗ 自检：r4：KA 读数等于钉值、KG 读数等于那一列的变体记 ka_pin=match、kg_pin=match；推的 v1822、v1…」、「✗ 自检：r4：假变体 KJ1T 在 (1, 2, 无, 罩着)、KJ2 在 (1, 0, RN, 罩着) 与基线不同」、「✗ 自检：r4：假变体只在 KG 与基线不同（读数换成它自己那一列）」、「✗ 自检：r4：假变体的 KG 读数不等于它那一列」、「✗ 自检：r4：变体跑到装置自己退 1（halt 行）而 KF 域 KJ0 变红」、「✗ 自检：r4：变异名选不到的那一行」、「✗ 自检：r4：基线与第一段产物不逐行相同」、「✗ 自检：r4：基线缺 criteria=root_choice_faults 判决行」、「✗ 自检：r4：第一段产物不在时 TV7 的比对」、「✗ 自检：弄坏开关 r4kgpinunchecked 开着时同一趟」
#
#   第 5 次重跑 research/prompts/e165-r5-prereg.md 9.3（第二段）与修订 1「修1.九」乙（第五段 5-乙）：
#   --pins r5   三个模式（不给 --profile、--profile r3、--profile r4）都认：开工核对改用 R5_PINS 那一套（第一段根 /tmp/claude-1000/e165-r5-segment1/root 上的
#       recovery.rs、replay-prefix lib.rs、crates/mutations.tsv 的 sha256 与函数区间；名字、标签、KA / KG 钉值、custom_keys 同旧的），V7 / TV7 的比对对象换成第一段的重跑产物；
#       自造表、标签表、--measure 用的内存包装与第一段产物一律取脚本所在的仓（第一段根里只有 Cargo.toml、Cargo.lock、.cargo、crates/）。不给 --pins 时三个模式照旧取原来三套，一个字不改。
#   --gate r2   只在 --profile r4 --pins r5 同给时认（别的组合退 2、一条都不跑）：调装置在 --bounds segment1 --family root_choice_faults 之后加 --gate r2；
#       基线 family=root_choice_faults 的 name=config 行要带 witness_gate=r2_some_slot_reports_a_read_error_and_no_device_self_verified_both_slots（5-甲 的装置在 --gate r2 时
#       实际写的闸字段与值，见 5-甲 产物第一行），缺了退 7、变体一个不编；TV7 的比对对象换成 5-甲 的产物；
#       驱动自己写的产物：config 行带 gate=r2 与根里装置的 sha256，每个 kill_bound 行与收尾判决行带 gate=r2（这几处是驱动的写法，装置不写）。
#       R5_GATE_R2_PINS 是 5-甲 的根（第一段根盖上 5-甲 的装置与变异表）上的值：变异表 sha256 与 5-甲 产物名都已填真值，两份源文件与两个函数区间同第一段根（5-甲 不改它们）。
#   下面两项是 r5 的弄坏开关（外面打开时自证的每一趟都带着它）：
#   E165_KILL_BOUNDS_BREAK=r5pinsignored  给了 --pins r5 照样用原来那套钉值
#       「r5：给了 --pins r5 应当按 r5 钉值核对、在旧钉值的假仓上停在 RH1」（三个模式各一格；实际退 0）
#       连带红的格：「✗ 自检：r5：--gate r2 整轮」、「✗ 自检：r5：--gate r2 那一套的变异表钉值与仓里的表对不上时」
#   E165_KILL_BOUNDS_BREAK=r4gateignored  给了 --gate r2 而调装置不带 --gate r2
#       「r5：--gate r2 整轮应当退 0、假装置每一次被调都带 --gate r2、产物带 gate=r2」（实际退 7：基线缺 R2 的闸字段、变体一个不编）
#   E165_KILL_BOUNDS_BREAK=defaultone  --jobs、--threads 都不给时取 1 × 1，不取登记的重活默认并行度
#       「--jobs、--threads 的缺省：…」（实际 (1, 1)）
#   E165_KILL_BOUNDS_BREAK=fixedcopylist  每个变体的副本照写死的 FIXED_COPY_ENTRIES 拷，不取共用库的闭包（没有 path 依赖的 prune-server/ 与编译期读的 .claude/heavy-thread-default）
#       「副本应当照共用库的闭包拷：…」（实际只有 Cargo.toml、Cargo.lock 与 crates/ 底下的）
#   E165_KILL_BOUNDS_BREAK=copyconfigurationunset  副本里编、跑装置不交多机配置（--root 给的根不在 git 里，照它的位置找不到配置）
#       「副本里起命令的多机配置应当取脚本所在的仓那一份：…」（实际没设得 None）
#   下面六项外面打开时自证的每一趟都带着它（drive() 照下面九项那一段的带法）：
#   E165_KILL_BOUNDS_BREAK=outputoverwritten  三套整轮的 --output 指到已经存在的文件照收（开跑之后整份换掉那一份）
#       「--output 指到已经存在的文件应当退 2、出路写换一个路径或先挪走那一份、一次不编、那一份一个字不动」（实际退 0、编 7 次）
#   E165_KILL_BOUNDS_BREAK=ledgeroverwritten  不给 --profile 那一套的 --ledger 指到已经存在的文件照收（开跑之后整份换掉那一份）
#       「--ledger 指到已经存在的文件应当退 2、出路写换一个路径或先挪走那一份、一次不编、那一份一个字不动」（实际退 0、编 7 次）
#   E165_KILL_BOUNDS_BREAK=judgeoffthetree  自证起的那几趟 judge: 另带 judge-root:..（判法闭包换到假树的上一层去算：那里没有假判法，每一项都拒算）
#       「改了假树里的判法应当每一项都现判、一项不拒算」（实际拒算 6）
#       连带红的格：「✗ 自检：续跑应当只编 5 次」、「✗ 自检：续跑：库里复用的三项」、「✗ 自检：全部复用的一趟应当一次不编」、「✗ 自检：账写完而钉表没钉时驱动当场核钉全」、
#       「✗ 自检：判过的一对重交同一条命令应当全部复用、照钉照核」、「✗ 自检：弄坏开关 reuseignored 开着时」、「✗ 自检：复用那六格」（复用格随 reuse-cells 带进去，六格都红）
#   E165_KILL_BOUNDS_BREAK=judgewholedriver  真跑的按项判定请求里判法写整份驱动（judge:<本脚本>），不写判法模块里判一个变体的那几个函数
#       「真跑的按项判定请求里判法应当是判法模块里判一个变体的函数与写基线那一行的函数（三套各两个）、judge-root 是脚本所在的仓，不是整份驱动」（实际三套都是 judge:scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh）
#   E165_KILL_BOUNDS_BREAK=pincheckskipped  钉红例时不跑 --check（当场追加之后不核钉全）
#       「账写完而钉表没钉时驱动当场核钉全应当判红、退 8、一次不编」（实际退 0）、「判过的一对重交同一条命令应当全部复用、照钉照核」（实际「核了 没核」）、
#       「整轮写完账与产物要当场钉进钉表、再核钉全」「续跑补齐之后照样当场钉、当场核」（实际成功那句不报核了几行）
#   E165_KILL_BOUNDS_BREAK=retainedmissingred  留存产物不在时不报本次未跑：V7 读不到（退非 0）、TV7 报 TV8 退 6
#       「r2：第一段产物不在时 V7 的比对应当逐项报本次未跑」「r3：第二段产物不在时 TV7 的比对应当逐项报本次未跑」「r4：第一段产物不在时 TV7 的比对应当逐项报本次未跑」
#       连带红的格：「✗ 作废条款 TV8：第一段产物读不到：…」、「✗ 作废条款 TV8：第二段产物读不到：…」
#
#   e165-replay-mutation-kill-bounds.sh --list-items
#       逐行列变体（基线 1 个、crates/mutations.tsv 里 21 种不同的替换、自造 5 条，共 27 行），不编不跑。
#   e165-replay-mutation-kill-bounds.sh --select-rows --output <产物>
#       只做登记 r1 第九节的选行（RQ4）与标签比对（RQ5、RQ6）：照 23 个变异名在今天的 crates/mutations.tsv 里找行，核 4 条（恰 1 行、文件列是 recovery.rs 或 lib.rs、
#       原文命中 1 次、命中处落在 9.2 的函数里），再把 scripts/mutation/tables/e165_replay_bounded_enumeration_tags.tsv 里每行的标签集合与登记时的集合比；
#       写 <产物>（已存在就不写、退 2），不编不跑。选不到（RQ4 / RQ6 有一行不满足）退 1，逐行比对结果照写；核对输入对不上（停机条款 RH1）退 4、不写。
#       r2 登记 9.1 不判标签（附带照报）：Z5 不在标签表里，它那一行的标签几列写 not_judged_in_r2，只按原文命中 1 次判选没选到。
#   e165-replay-mutation-kill-bounds.sh --scratch <草稿目录> --output <产物> --ledger <红例账> [--item <变体>]… [--jobs <并行度>] [--threads <每个变体的线程数>（只给自证）]
#       点名的变体（不给 --item 是全部）每个一条判定：键相同、库里判绿的取库里的行、不编不跑，别的现判、判完当场入库（「按项判定」一段）；
#       产物与红例账是这一趟的，从头写，照变体的次序落行；中途停下：挪走这一趟没收尾的产物与红例账，再重交同一条命令，只补没判的。
#       27 个变体都有行、收尾的基线与开头逐字相同，才写收尾三行（name=restored、name=verdict、name=done）。
#   e165-replay-mutation-kill-bounds.sh --scratch <草稿目录> --output <产物> --ledger <红例账> --recheck --item <变体>
#       只复跑点名的那一条，拿新的行与产物里已有的行比，不写产物、不读不写判定库；不同就退 1。
#   e165-replay-mutation-kill-bounds.sh --scratch <草稿目录> --measure <变体>
#       量一次：从改 recovery.rs 到编完 bin 的挂钟与内存峰值（冷编译，经 run-with-memory-cap.sh），再量跑 --bounds segment2 的挂钟。不写产物。
#   e165-replay-mutation-kill-bounds.sh --clean --scratch <草稿目录>
#       删掉这条脚本在草稿目录里建的 kill-bounds/（副本、编译目录、每个变体的记录），删前报大小。
#   e165-replay-mutation-kill-bounds.sh --selftest
#       判定写进临时 prune-server：只在临时服务底下跑（long_run_item_verdicts.py selftest-service，即客户端 selftest_service_problem；不在就退 2、一格都不跑），起法：
#         python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh --selftest
#       每一趟一份新的库命名空间；假仓里没有真装置的源码，装置入口不换的那几趟每一项都拒算、照跑（入口先算、算不出就停，判法的闭包不收）。
#       自证起的每一趟算键都不碰脚本所在的仓：装置入口经 LONG_RUN_ITEMS_SELFTEST_ENTRIES 换成假树里的 Python 函数，judge: 换成假树里的
#       SELFTEST_JUDGE_ENTRY（不带 judge-root:，判法闭包在假树上算；「按项判定」一段）。每判完一项打一行 E165_KILL_BOUNDS_SELFTEST_PROGRESS（立刻刷出）。
#       拿假的仓、假的 cargo 走一遍：锚点命中 0 次必须报「锚点腐化」退 3、一条都不跑；红、界内不红、无效各一条判得对；--output、--ledger 已经存在退 2、一次不编；
#       真跑的按项判定请求里判法写判法模块里判一个变体的函数与写基线那一行的函数（三套各两个），不写整份驱动；
#       续跑：判完 tsv-3 就被停下的一趟重交只补没判的、库里复用的三项不编，再交一趟全部复用、一次不编（装置入口换成假树里的 Python 函数）；
#       改了假树里的判法，同一条命令每一项都现判、一项不拒算；
#       复用那六格（long_run_item_verdicts.py reuse-cells，打 REUSE_SELFTEST script=scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh …）；
#       弄坏开关开着时这几样必须判错。
#       r2 补的格（登记 r2 9.4 第 6 条）：假变体 J6 在 N = 2、J6Qn 在 N = 3 与基线不同 ⇒ 报 (2, 3)；只有 J6Qn 不同 ⇒ J6 报 none；
#       基线里缺 J6Qn_ 行 ⇒ 报「装置没带 --profile r2」退 7、变体一个不编；自造表 Z 五行时 --list-items 列全五条。
#       r3 补的格（登记 r3 9.3 第 6 条，在自己的假仓里走，不碰上面 r2 的假仓）：--list-items 列 7 行；假变体 KJ1 在 (1, 2)、KJ2 在 (1, 0) 与基线
#       不同 ⇒ 报 KJ1:K=M1U2、KJ2:K=M1U0；假变体在 KA 上的 KJ2 读数不等于钉值 ⇒ ka_pin=mismatch 且整轮退 1；基线缺 criteria=root_choice 判决行
#       ⇒ 报「装置没带 --profile r3」退 7、变体一个不编；另有基线与第二段产物不同退 6、选行不满足记 not_compared、变体跑到装置自己退 1（TF9 的 halt 行）
#       而照常落行三格。
#   每种用法都可以加 --root <仓根>：被测的仓不是这条脚本所在的那一个时用（默认取脚本所在的仓）。
#   三套整轮的 --output、不给 --profile 那一套的 --ledger 指到已经存在的文件：退 2、不编不跑、不写产物与账（出路：换一个路径，或先挪走那一份）；
#   --recheck 读已有的产物与账，不受这一条管。
#
# 做法（每个变体一份副本、一个编译目录，仓里的 crates/ 一个字节不碰；第 3–5 步只做这一趟要现判的变体，库里复用的不编不跑，「按项判定」一段）：
#   1. 开工核对：仓里 recovery.rs、lib.rs 与 crates/mutations.tsv 的 sha256 等于 REGISTERED 里的值（r2 登记 9.4 第 1 条：第一段收尾冻结的根上的值；
#      crates/mutations.tsv 那一个在第一段收尾之前填不出，写的是占位串，开工核对一律停在 RH1）、9.2 那九个函数的行区间（从 fn 行到其后第一个只有 } 的行）
#      等于 9.2 的表，不等是停机条款 RH1（退 4，一条都不跑）。自造表的 Z 行名字要等于 REGISTERED 的 custom_keys（Z1–Z5），不等同退 4。再按 23 个变异名从 crates/mutations.tsv 选行（r1 第九节 9.1，不按行号）：每个名字恰 1 行、
#      文件列是 recovery.rs 或 lib.rs、原文在那份文件里命中 1 次、命中处的起行或末行落在 9.2 的函数里，有一个选不到就退 4、一条都不跑；
#      自造表每行的文件列也要是这两份之一。今天的表行号只用来给变体起名（tsv-<行号>）。
#   2. 每条变异的原文在它第二列点名的那份文件里恰好命中一次（\n 还原成换行）；有一条不是就报「锚点腐化」，退 3，一条都不跑。
#   3. 基线（不改的副本）先编先跑 --profile r2 --bounds segment2；每个有 J6 行的 (域, N) 都要有 J6Qn 两行，缺了就是装置没带 --profile r2，退 7、
#      变体一个不编（r2 9.4 第 6 条）；它每个 (域, N) 的行必须逐字出现在第一段 r2 档的两份产物里（作废条款 V7，r2 9.4 第 2 条、SV9），不然退 6
#      （产物有不在的，这一项本次未跑，见「留存产物」一段）。
#   4. 每个变体：照 scripts/hosts/lib_repository_copy.py 在根上现算的共用清单与闭包（path 依赖、编译期读与运行期起的仓根文件）拷成一份副本（不带 target），
#      原件另存在副本之外、核 sha256；副本里编、跑编出来的装置时，多机配置取脚本所在的仓那一份（照副本或冻结根的位置找不到它）；
#      改坏、`cargo build --release --offline -p singlefs-verification --bin e165_replay_bounded_enumeration`（编不过记「无效」）、
#      跑 --bounds segment2、逐行与基线比；红了的域在首红的 N 上找一个反例状态（两边的 --list-states 逐行比出第一个判定不同的状态），
#      再拿两边的二进制各 --replay-state 单条重放一遍，判定必须与逐态行相同；收尾把原件拷回副本、核 sha256，删掉这份副本与编译目录。
#      建副本、改坏、编译在这份驱动里；起装置、逐行比、找红例、出产物行与红例账行在判法模块 e165_replay_mutation_kill_bounds_variant_judgement.py（「按项判定」一段）。
#   5. 收尾再建一份不改的副本重编重跑基线，输出必须与开头逐字相同；仓里 recovery.rs 的 sha256 与 `git status --porcelain crates/`
#      开头与结尾相同（作废条款 V6），不然退 5、不写收尾行。
#   6. 红例账照落，写完收尾三行之后当场钉进 .claude/gate.d/e165-pinned-red-examples.tsv、当场核钉全（「钉红例」一段）。
# 钉红例：不给 --profile 的那一套（写红例账的只有它）写完一对账（--ledger）与 kill-bounds 产物（--output）、收尾三行落盘之后，
#   先跑 python3 scripts/experiment-apparatus/e165-pin-red-examples.py --root <脚本所在的仓> --kill-bounds <产物> <账>（追加：已钉的不重复，档取产物 config 行的
#   device_arguments=，即 --profile r2），再跑同一份的 --check --kill-bounds <产物> <账>（显式给这一对）；钉表与变异表取脚本所在的仓，不取 --root。
#   有一步退非 0，原样转出它的输出、整趟退 8（EXIT_PIN_INCOMPLETE）；都过，成功那句报「钉红例：钉进 N 行、核了 M 行」。
#   上一趟钉红例没过、改好之后挪走那一趟的产物与账再重跑同一条命令：判过的变体全部从判定库复用、不编不跑，产物与账照样从头写，这一步照样做。--recheck 不钉。
#   只供自证：E165_KILL_BOUNDS_SELFTEST_SKIP_PIN_APPEND 设了（且是自证起的那一趟）就不追加，造「账写完了、钉表没钉」。
# 留存产物：V7、TV7 比的对象（第一段、第二段的产物，五-甲的产物）在 research/results/ 下，那个目录可以整个删掉。比对的对象有不在的，
#   那一项本次未跑：逐个点名不在的产物打一行「! 本次未跑：作废条款 <V7|TV7> 的比对：留存产物 <路径> 不在」（环境里有 GATE_NOT_RUN_FILE 的另往那份文件追加一行），
#   不当绿、不判红，照跑完；基线那一行的比对栏（rows_found_verbatim_in_segment1_products、rows_identical_to_segment1_product、rows_identical_to_segment2_product）
#   与收尾判决行的 baseline_equals_segment1_product、baseline_equals_segment2_product 写 not_run，收尾那句接「本次未跑 N 项：…」。在的照旧比，不同照旧退 6；
#   在而一行 E7RESULT 都读不到照旧按 TV8 退 6。
# 界固定为登记 5.6 的上沿（修订第 10 条去掉了放宽规则）：每个变体整份 --bounds segment2 跑一遍，每个域的「首红的 N」取第一个与基线不同的 N，
#   与「从小往大扫、红了就停」得出的是同一个数；到上沿没红记「界内不红」。
# 「红」的口径：这个 (域, N) 上某条判据的违例数与基线不同。条款判据（J0–J4、J5a、J5b、J6、J6Qn、J7a 三格、J7b、J7c、A0、AS 三条、A-结构、A-限）与
#   实现锚点（IA1、IA2、合法历史一族里原实现报错的态数）分两栏；只有别的量（见证计数、施加过的态数、见证文字）不同的另列，不算红。
#   结局：red（有条款判据红）、red_only_by_implementation_anchor、not_red_within_bounds、invalid（编不过）。
#   first_clause_red 照上面的口径（按违例数、红了就不再往上看那个域）。r2 另报两样，口径是行的文字、整份 --bounds segment2 都比、不早停：
#   first_red_by_criterion=J6:<域>=<N>,…;J6Qn:<域>=<N>,…——J6（J6_premise_holds、J6_premise_fails 两行合看）与 J6Qn（两行合看）各自在每个域上
#   最小的 N，使变体那条判据任一行与基线同一 (域, N) 的同一行文字不同（变体缺那一行也算不同）；到上沿都相同写 none（登记里的「无」），
#   基线在那个域上一行那条判据都没有写 no_rows。
#   first_text_red_criteria=<域>:<N>:<判据>+<判据>;…——每个域上任一条判据的 name=violations 行文字与基线不同的最小 N，与那个 N 上文字不同的全部判据名
#   （SQ5 要 Z5 的这一格；每个变体都报）。
#
# 并行：--jobs 个变体同时做，每个变体的编译与装置各用 --threads 条线程；jobs × threads 不许超过线程上限 N（统一任务服务套的 capped.sh <N> 设的 SINGLEFS_THREAD_CAP）。
#   --threads 只给自证（自证拿它把假跑压到 1 条线程）；交给统一任务服务的命令里不写，写了被拒。
#   缺省：--jobs 不给是 1；--threads 不给是 N ÷ --jobs（向下取整，至少 1），N = SINGLEFS_THREAD_CAP，没设取登记的重活默认并行度（.claude/heavy-thread-default，
#   scripts/hosts/heavy_threads.py）；所以什么都不给是 1 个变体、N 条线程（登记 31 时 jobs 1 × threads 31，--jobs 4 时 4 × 7）。
# 内存：装置、基线与每个变体的二进制都直接起，脚本里不套内存包装、不设内层上限：整条命令交 task-server
#   （task-server/task submit -- bash scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh …），内存额度按记录由服务给，撞上限由服务放大重排整条命令。
#   变体的退出码一律是装置自己的退出码，照装置的退出码比，没有「包装的结局不算结果」那一支。
#   唯一的例外是 --measure：它量的就是内存峰值，量编译那一步仍经 scripts/hosts/run-with-memory-cap.sh 8G 读它写的峰值表。
#
# 退出码：0 点名的都落了行（现判的或从判定库复用的）；1 有变体没得出结果（装置没跑完）、--recheck 对不上、派出与收回的项数对不上（「每项一行与总计」一段）、
#   按项判定收尾判红（复用哨兵红、键在跑的中途变了，「按项判定」一段）、或自证判红；
#   2 用法不对（--item 写了不认识的变体名时列出可点名的全部变体）、三套整轮的 --output 或不给 --profile 那一套的 --ledger 已经存在、--selftest 不在临时 prune-server 底下；
#   3 锚点腐化；4 停机条款 H1（含自造表的 Z 行名字与登记不等）；5 作废条款 V6（收尾基线与开头不同，或仓里的 crates/ 在跑的时候变了；
#   或这一趟的基线与判定库里同一把键的基线不逐字相同）；6 作废条款 V7；
#   7 基线里缺 J6Qn_ 行（装置没带 --profile r2，r2 9.4 第 6 条）；8 钉红例没过（当场追加或 --check 退非 0，「钉红例」一段）；75 prune-server 不可用（「按项判定」一段）。
# 弄坏开关 E165_KILL_BOUNDS_BREAK=<项>（只给 --selftest 证明它会红用，逗号分隔可给几个；不认识的项当场退出）。
# 逐项写开关名、弄坏哪一处、打开后自证哪几项红（自证的例没有标签，项名直接写那条「✗ 自检：」失败行的句首，括号里写那一行接着报的）：
#   E165_KILL_BOUNDS_BREAK=progressnever  进度间隔的判断永不触发、一行进度都不打
#       「假时钟上过了进度间隔（300 秒）应当各打一行、没到不打」（实际：差一秒 None；到了 None；再过一个间隔 None）
#       「整轮里一个变体跑过进度间隔应当打出点名它的进度行」（实际退 0、进度行 []）
#   下面三项是入口脚本格式那几格（「每项一行与总计」一段），外面打开时自证的每一趟都带着它：
#   E165_KILL_BOUNDS_BREAK=itemcountsdropped  每项一行不带 judged=、red=、reused=
#       「整轮每项一行应当带判了、红了、复用了多少与秒，末行总计」、「续跑：库里复用的三项应当各打一行 reused=1」
#   E165_KILL_BOUNDS_BREAK=itemlinedropped  第一个做完的变体（基线除外）不打它那一行、不算收回（造一项收不回）
#       「整轮每项一行应当带判了、红了、复用了多少与秒，末行总计」（实际派出 6 项、收回 5 项，退 1）
#       连带红的格（这几趟都有变体做完，少收回一项就退 1）：「✗ 自检：整轮应当退 0」、「✗ 自检：续跑应当退 0」、「✗ 自检：续跑：库里复用的三项」、「✗ 自检：r2：第一段产物不在时 V7 的比对」、
#       「✗ 自检：--recheck 同一条」、「✗ 自检：整轮里一个变体跑过进度间隔」、「✗ 自检：r2 整轮」、「✗ 自检：r3 整轮」、「✗ 自检：弄坏开关 r3kapinunchecked 开着时同一趟」、
#       「✗ 自检：r4 整轮」、「✗ 自检：弄坏开关 r4kgpinunchecked 开着时同一趟」、「✗ 自检：r5：--gate r2 整轮」
#   E165_KILL_BOUNDS_BREAK=unknownitemunlisted  --item 写错时不列可点名的变体（只说「--list-items 看」）
#       「--item 写了不认识的变体名应当退 2、列出可点名的 6 项、一次不编」
#   下面四项是 r2 登记 9.5「驱动」三行加一项，外面打开时自证同样红（自证的 r2 那几趟把外面给的这四项原样传进去）：
#   E165_KILL_BOUNDS_BREAK=j6qnfromj6  J6Qn 的首红取了 J6 那两行（9.5「J6Qn 的首红取了 J6 的列」）
#       「r2：假变体 J6 在 N = 2、J6Qn 在 N = 3 与基线不同应当报 J6:L=2、J6Qn:L=3」（实际 J6Qn:L=2）
#       连带红的格：「✗ 自检：r2：只有 J6Qn 不同的假变体」
#   E165_KILL_BOUNDS_BREAK=noprofile  调装置（--bounds、--list-states、--replay-state）不带 --profile r2（9.5「基线调装置不带 --profile r2」）
#       「r2 整轮应当退 0」（实际退 7）、「r2 整轮里假装置每一次被调都应当带 --profile r2」
#       连带红的格：「✗ 自检：r2：J6 只差见证文字时按违例数的首红看不见它（L:3 才红、红的是 J6Qn），按文字的首红在 L:2、点名 J6_premi…」、「✗ 自检：r2：假变体 J6 在 N = 2、J6Qn 在 N = 3 与基线不同」、「✗ 自检：r2：只有 J6Qn 不同的假变体」、「✗ 装置没带 --profile r2：基线有 J6 行的 3 个 (域, N) 里 3 个缺 J6Qn_ 行，第一个 L:1」
#   E165_KILL_BOUNDS_BREAK=noprofilecheck  不核基线里有没有 J6Qn_ 行（9.5 同一行写的「第三格由退出非 0 转放行」）
#       「r2：基线里缺 J6Qn_ 行应当报「装置没带 --profile r2」退 7、变体一个不编」（实际退 0）
#   E165_KILL_BOUNDS_BREAK=zfirstfour  自造表只读前四行（9.5「Z 行只读前四条」）
#       「✗ 自检：r2：自造表 Z 五行时 --list-items」（应当列 8 行，实际 7 行）
#   下面四项是 r3 登记 9.3 第 6 条那张表的弄坏开关，外面打开时自证同样红（自证的 r3 那几趟把外面给的这四项原样传进去，别的趟不带）：
#   E165_KILL_BOUNDS_BREAK=r3itemsmissing  r3 名单少一条（9.3「r3 --list-items 7 行」）
#       「r3：--list-items 应当列 7 行（基线 1、变体 6）」（实际 6 行）
#       连带红的格：「✗ 自检：r3 整轮：收尾行」、「✗ 自检：r3：变异名选不到的那一行」
#   E165_KILL_BOUNDS_BREAK=r3firstredswap  首红那一栏里 KJ1 与 KJ2 两列对调
#       「r3：假变体 KJ1 在 (1, 2)、KJ2 在 (1, 0) 与基线不同应当报 KJ1:K=M1U2、KJ2:K=M1U0」（实际 KJ1:K=M1U0、KJ2:K=M1U2）
#   E165_KILL_BOUNDS_BREAK=r3kapinunchecked  不核 KA 上的钉值（读数不等于钉值也不报 mismatch、不退 1）
#       「r3：假变体的 KA 读数不等于钉值应当报 ka_pin=mismatch 且整轮退 1」（实际退 0）
#       连带红的格：「✗ 自检：r3：K 族不红而 KA 读数等于钉值的变体记 not_red_within_bounds、ka_pin=match；推的两个（1…」、「✗ 自检：r3：变异名选不到的那一行」
#   E165_KILL_BOUNDS_BREAK=r3noprofile  调装置不带 --profile r3
#       「r3 整轮应当退 0」（实际退 1：假装置不认没有 --profile r3 的 --family root_choice）、「r3 整轮里假装置每一次被调都应当带 --profile r3」
#       连带红的格：「✗ 基线没跑完或退出码不是 0（2；日志 …）」、「✗ 自检：r3 整轮：收尾行」、「✗ 自检：r3：K 族不红而 KA 读数等于钉值的变体记 not_red_within_bounds、ka_pin=match；推的两个（1…」、「✗ 自检：r3：假变体 KJ1 在 (1, 2)、KJ2 在 (1, 0) 与基线不同」、「✗ 自检：r3：假变体的 KA 读数不等于钉值」、「✗ 自检：r3：变体跑到装置自己退 1（halt 行）而 K 域 KJ0 变红」、「✗ 自检：r3：变异名选不到的那一行」、「✗ 自检：r3：基线与第二段产物不逐行相同」、「✗ 自检：r3：基线缺 criteria=root_choice 判决行」、「✗ 自检：r3：第二段产物不在时 TV7 的比对」、「✗ 自检：弄坏开关 r3kapinunchecked 开着时同一趟」
# 下面九项外面打开时，自证的每一趟都带着它（drive() 把外面给的并进那一趟自己点名的；r2 那四项照旧只由 r2 那几趟带），红在下面点名的项；
# 自证里对照那几例自己打开同一项（证明前一例的红来自那一处检查），外面打开时那几例本来就开着、不另红：
#   E165_KILL_BOUNDS_BREAK=noanchorcheck  不核锚点、命中 0 次的那条照「改坏」（什么都没改）跑
#       「锚点命中 0 次应当报「锚点腐化」退 3」（实际退 0）、「锚点腐化时一次都不许编、不许写产物」（实际编了 7 次）、
#       「弄坏开关 recoveryonly 开着时文件列是 lib.rs 的那一条应当由红转「锚点腐化」退 3（证明整轮那一格的红来自按第二列改文件）」（实际退 0）
#   E165_KILL_BOUNDS_BREAK=reuseignored  plan 答 reuse 的项也照现判（不取库里的行，判完照 record：哨兵按「键相同却现判」判红）
#       「续跑应当只编 5 次（基线 2、没判的变体 3）」（实际 7）、「续跑：库里复用的三项应当各打一行 reused=1」、「全部复用的一趟应当一次不编」（实际编 7 次、退 1）
#       连带红的格：「✗ 自检：续跑应当退 0」、「✗ 自检：续跑补齐之后要写收尾行」、「✗ 自检：续跑补齐之后照样当场钉、当场核」、「✗ 自检：账写完而钉表没钉时驱动当场核钉全」、
#       「✗ 自检：弄坏开关 pincheckskipped 开着时」、「✗ 自检：判过的一对重交同一条命令应当全部复用、照钉照核」、「✗ 自检：复用那六格」；
#       复用那六格里红的：「reuse-three-questions」「key-independent-of-directory」「reuse-sentinel」「forced-still-reused-red」
#   E165_KILL_BOUNDS_BREAK=finishignored  不认 long_run_item_verdicts.py finish 的红（复用哨兵红、键中途变了照写收尾行、照退 0）
#       「弄坏开关 reuseignored 开着时应当每个变体都重编、复用哨兵判红退 1」（实际退 0）、「复用那六格应当全过」；
#       复用那六格里红的：「sentinel-rejudged-red」「forced-verdict-differs-red」「forced-still-reused-red」
#   E165_KILL_BOUNDS_BREAK=recorddeferred  现判的项判完不当场 record，等全部判完、finish 之前一并写
#       「续跑应当只编 5 次（基线 2、没判的变体 3）」（实际 7：被停下的那一趟一项都没入库）、「续跑：库里复用的三项应当各打一行 reused=1」
#   复用那几格与续跑那几格另认 long_run_item_verdicts.py 的弄坏开关 LONG_RUN_ITEMS_BREAK（它文件头；随环境带进每一趟，打开后复用那六格红在它点名的同名格，
#       library-unread 另红续跑那几格）。
#   E165_KILL_BOUNDS_BREAK=byrownumber  选行按登记时的行号、不按变异名（r1 9.5）
#       「选行 ①：名字不在表里应当报 no_row_with_this_name、点名第 2 行、退 1、不编」（实际退 0、编 0 次）、
#       「选行 ②：同名两行应当报 more_than_one_row_with_this_name、退 1、不编」（实际退 0、编 0 次）
#       连带红的格：「✗ 有变体选行不满足（TH5）记了 not_compared，交主 agent；别的变体照判」、「✗ 自检：r3 整轮」、「✗ 自检：r3 整轮：收尾行」、「✗ 自检：r3：--list-items」、「✗ 自检：r3：K 族不红而 KA 读数等于钉值的变体记 not_red_within_bounds、ka_pin=match；推的两个（1…」、「✗ 自检：r3：假变体 KJ1 在 (1, 2)、KJ2 在 (1, 0) 与基线不同」、「✗ 自检：r3：假变体的 KA 读数不等于钉值」、「✗ 自检：r3：变体跑到装置自己退 1（halt 行）而 K 域 KJ0 变红」、「✗ 自检：r3：变异名选不到的那一行」、「✗ 自检：r4 整轮」、「✗ 自检：r4 整轮：收尾行」、「✗ 自检：r4：--list-items」、「✗ 自检：r4：KA 读数等于钉值、KG 读数等于那一列的变体记 ka_pin=match、kg_pin=match；推的 v1822、v1…」、「✗ 自检：r4：假变体 KJ1T 在 (1, 2, 无, 罩着)、KJ2 在 (1, 0, RN, 罩着) 与基线不同」、「✗ 自检：r4：假变体只在 KG 与基线不同（读数换成它自己那一列）」、「✗ 自检：r4：假变体的 KG 读数不等于它那一列」、「✗ 自检：r4：变体跑到装置自己退 1（halt 行）而 KF 域 KJ0 变红」、「✗ 自检：r4：变异名选不到的那一行」、「✗ 自检：r5：--gate r2 整轮」、「✗ 自检：弄坏开关 r3kapinunchecked 开着时同一趟」、「✗ 自检：弄坏开关 r4kgpinunchecked 开着时同一趟」、「✗ 自检：选行 ⑧：登记名那一行不成形（cargo 参数那一段空着）」
#   E165_KILL_BOUNDS_BREAK=nodupcheck  同名多行时不报、取第一行
#       「选行 ②：同名两行应当报 more_than_one_row_with_this_name、退 1、不编」（实际退 0、编 0 次）
#   E165_KILL_BOUNDS_BREAK=owntablesplit  选行回到自己切 crates/mutations.tsv（四段以上就收、不看哪一段空着），不经 scripts/mutation/crates-mutation-rows.py 的 read_table
#       「选行 ⑧：登记名那一行不成形（cargo 参数那一段空着）应当报 no_row_with_this_name、点名第 2 行、退 1、不编」（实际退 0）
#   E165_KILL_BOUNDS_BREAK=nolocationcheck  不核命中处在不在 9.2 的函数里
#       「选行 ③：原文在 9.2 的函数之外应当报 outside_the_registered_functions、点名第 5 行、退 1、不编」（实际退 0、编 0 次）
#   E165_KILL_BOUNDS_BREAK=recoveryonly  改坏时只写 recovery.rs、不看第二列
#       「整轮应当退 0」（实际 3）、「文件列是 lib.rs 的那一条应当改 lib.rs、记 outcome=red」、「自造的那一条（文件列 lib.rs）应当记 red」、
#       「收尾要打「已还原，基线与开头逐字相同」并写 name=restored 行」、「收尾行应当是 name=done emitted=10」（实际是一条 name=kill_bound 行）、
#       「整轮应当编 7 次（基线 2、变体 5）」（实际 4）、「续跑应当退 0」（实际 3）、「续跑应当只编 5 次（基线 2、没判的变体 3）」（实际 2）、
#       「续跑之后每个变体恰好一行完整的」（实际 tsv-5、Z1 各 0 行）、「续跑补齐之后要写收尾行」、「全部复用的一趟应当一次不编」（实际退 3）、
#       「弄坏开关 reuseignored 开着时应当每个变体都重编、复用哨兵判红退 1」（实际退 3）、
#       「弄坏开关 recorddeferred 开着时被停下的那一趟一项都没入库，重交应当全部重编」（实际 4）、「r2 整轮应当退 0」（实际 3）
#       连带红的格：「✗ 自检：r2：第一段产物不在时 V7 的比对」、「✗ 自检：r3 整轮：收尾行」、「✗ 自检：r3：K 族不红而 KA 读数等于钉值的变体记 not_red_within_bounds、ka_pin=match；推的两个（1…」、「✗ 自检：r3：假变体 KJ1 在 (1, 2)、KJ2 在 (1, 0) 与基线不同」、「✗ 自检：r3：假变体的 KA 读数不等于钉值」、「✗ 自检：r3：变体跑到装置自己退 1（halt 行）而 K 域 KJ0 变红」、「✗ 自检：r3：变异名选不到的那一行」、「✗ 自检：r3：第二段产物不在时 TV7 的比对」、「✗ 自检：r4 整轮：收尾行」、「✗ 自检：r4：KA 读数等于钉值、KG 读数等于那一列的变体记 ka_pin=match、kg_pin=match；推的 v1822、v1…」、「✗ 自检：r4：假变体 KJ1T 在 (1, 2, 无, 罩着)、KJ2 在 (1, 0, RN, 罩着) 与基线不同」、「✗ 自检：r4：假变体只在 KG 与基线不同（读数换成它自己那一列）」、「✗ 自检：r4：假变体的 KG 读数不等于它那一列」、「✗ 自检：r4：变体跑到装置自己退 1（halt 行）而 KF 域 KJ0 变红」、「✗ 自检：r4：变异名选不到的那一行」、「✗ 自检：r4：第一段产物不在时 TV7 的比对」、「✗ 自检：判过的一对重交同一条命令应当全部复用、照钉照核」、「✗ 自检：弄坏开关 pincheckskipped 开着时」、「✗ 自检：弄坏开关 r3kapinunchecked 开着时同一趟」、「✗ 自检：弄坏开关 r4kgpinunchecked 开着时同一趟」、「✗ 自检：整轮写完账与产物要当场钉进钉表、再核钉全：红例账里有 1 行反例，钉表」、「✗ 自检：续跑补齐之后照样当场钉、当场核：同一批红例已钉，钉表」、「✗ 自检：账写完而钉表没钉时驱动当场核钉全」、「✗ 锚点腐化：Z1 的原文在副本里的 crates/singlefs-core/src/recovery.rs 命中 0 次」、「✗ 锚点腐化：tsv-2 的原文在副本里的 crates/singlefs-core/src/recovery.rs 命中 0 次」、『✗ 锚点腐化：tsv-3 的原文在副本里的 crates/singlefs-core/src/recovery.rs 命中 0 次』、「✗ 锚点腐化：tsv-5 的原文在副本里的 crates/singlefs-core/src/recovery.rs 命中 0 次」
# 每项一行与总计（.claude/rules/test-script-discipline.md「入口脚本：不带参数就是剪枝跑全部项」）：三套整轮（不给 --profile、--profile r3、--profile r4）都往 stdout 打，
#   每个点名的变体（含基线）一行 E165_KILL_BOUNDS_ITEM item=<变体> exit=<0 得出结果、1 没得出> judged=<这一趟现判 1，否则 0> red=<那一行的结局是 red 记 1，否则 0>
#   reused=<判定库里键相同、判绿，这一趟取库里的行、没跑记 1，否则 0> seconds=<这一项的挂钟>；这一趟跑的做完时打，库里复用的开跑前打。
#   整轮收尾（中途停机退出的也算）末行 E165_KILL_BOUNDS_TOTAL dispatched=<派出几项> collected=<收回几项> items=<打了几行> judged= red= reused= failed= seconds=；
#   派出与收回对不上（.claude/singlefs-ai-sop/rules/command-safety.md「并行不许把失败吃掉」）先打 ✗ 再打总计，本来退 0 的改退 1。
#   --list-items、--select-rows、--measure、--clean 不打。判定复用经 prune-server 里的按项判定（「按项判定」一段），不设 --full。
# 按项判定（.claude/rules/test-script-discipline.md「判定可复用：按输入存，按前缀剪」）：三套整轮每个变体（含基线）一条判定，经
#   scripts/experiment-apparatus/long_run_item_verdicts.py 的 plan / record / finish 存进 prune-server 的 apparatus-items 族，种类 e165_replay_bounded_enumeration
#   （判定库只有那一份，本地不建库目录；产物与草稿目录都不是复用来源）。--recheck、--list-items、--select-rows、--measure、--clean 不读不写判定库。
#   键（long_run_item_verdicts.py 文件头「键」，即 admission.py 的 compute_unit_key）：装置入口 crates/singlefs-verification/src/bin/e165_replay_bounded_enumeration.rs::main
#   在 --root 那棵树上的单元闭包（profile:release）、工具链、闭包真引到的依赖版本；judge: 判法模块 scripts/experiment-apparatus/e165_replay_mutation_kill_bounds_variant_judgement.py
#   里这一套判一个变体的函数与写基线那一行的函数（JUDGE_FUNCTIONS：judge_variant 与 baseline_item_line，--profile r3、r4 各是带 _r3、_r4 的那两个；判法闭包在
#   脚本所在的仓上算，judge-root:）：闭包是起装置、读它的输出、按「红」的口径出产物行与红例账行的那几个函数、它们调到的单元与那份模块 import 时执行的语句；
#   建副本、改坏、编译、选行、钉红例、产物收尾与读写判定库的代码在这份驱动里，不进判法闭包，它们读的表（变异表、自造表、钉表、按项判定的登记表）也就不进判法键
#   （变异行原文经下面的 line:item= 进键）；自证起的那几趟换成假树里的 SELFTEST_JUDGE_ENTRY、不带 judge-root:，判法闭包在假树上算）；
#   line: 这一趟的档（--profile、--pins、--gate 与界参数）、这一套登记值的 sha256、这一项写进产物行与红例账的字段（变异行原文、文件、表行号、名字、标题，ITEM_KEY_FIELDS）。
#   改坏之后的源码不进闭包，变异行原文经 line: 进键；改坏那一处碰到模块集合（MODULE_SET_TOKENS：mod、#[path、include_str!、include_bytes!、include!）的变体拒算：
#   不进请求，照跑、不复用、不入库，收尾逐个报。基线是一项，留存产物（V7 / TV7 的比对对象）经 @input 按内容进它的键（不在也是一种取值）；每个变体 @after 基线。
#   值是这一项的产物行与红例账行（键排序的 JSON）；产物行里复跑命令那一段 rerun="…" 存成 rerun="<复跑命令>"，取回时照这一趟的 --output、--ledger 填回，
#   与现跑的一个字不差。得出结果的判绿（被测变体被抓到、记 outcome=red 的也是得出了结果）；驱动没得出结果的（装置没跑完而没有一个域红）record --red，
#   记红撤绿、下一趟现判。
#   一趟：开工核对与选行 → plan（每项打 LONG_RUN_ITEM 行）→ 有项要现判才编、跑开头基线（库里复用的基线也照跑一遍给变体比，与库里那一行不逐字相同退 5，不入库；
#   没点名的基线库里没有可复用的，照跑照记）→ 现判的项判完当场 record → 收尾基线 → finish（复用哨兵、键中途变了，红了退 1、产物不收尾）→
#   产物从头写：config 行、点名的变体照变体的次序落行（库里复用的取库里的）、全都有行才写收尾三行。一项都不用现判时开头与收尾的基线都不编不跑，
#   收尾行要的基线行数取库里基线那一行的 result_rows。中途停下：挪走没收尾的产物与红例账（--output、--ledger 已经存在退 2），重交同一条命令：判完的已在库里，只补没判的。
#   成功那句报「按项判定 现判 N、复用 M、拒算 R」（R 含模块集合拒算的）。服务连不上退 75。状态目录在 <草稿目录>/kill-bounds/item-verdicts/，每一趟新建、跑完删。
#   只供自证：E165_KILL_BOUNDS_SELFTEST_KILL_AFTER=<项>（自证起的那一趟才认）判完这一项、record 之后整个进程被 SIGKILL，造「中途停下」；
#   E165_KILL_BOUNDS_SELFTEST_SET_OUTPUT_ASIDE=1（自证起的那一趟才认）已经存在的 --output 与 --ledger 先挪成 <名>.previous 再判（同一条命令重交几趟的那几格）。
# 进度：跑着的时候每过 PROGRESS_INTERVAL_SECONDS 秒（300，.claude/rules/implementation-workflow.md「测试与崩溃检测优先多线程」定的进度间隔）往 stderr 打一行
#   E165_KILL_BOUNDS_PROGRESS：挂钟、变体做完几个（共几个），在做的每一项在哪一步（建副本与编译、跑 --bounds segment2、在哪个 域:N 上找红例或单条重放）、
#   这一步与这一项各跑了多久。开头与收尾的基线也算在做的项。
#   只供自证：E165_KILL_BOUNDS_PROGRESS_INTERVAL=<秒> 换掉间隔，只在自证起的那一趟（E165_KILL_BOUNDS_SELFTEST_ROOT 也设了）认。
#
# 拆分（每一份去注释不超过 2000 行，上游 code-discipline.md「一个代码文件不超过 2000 行」）：本脚本是入口，留下面的 shell 前奏（弄坏开关只在自证里认、取自己的路径）与 python 里的读参数与收尾（main）；
#   python 的其余部分是同目录两份被 import 的模块，函数、常量与注释都在那两份里（文件头的用法、口径与弄坏开关表在本脚本）：
#   e165_replay_mutation_kill_bounds_driver.py：登记表与钉值、载入变异表与选行、建副本与编译、按项判定（ItemVerdicts）、r2、r3、r4 三套整轮、--select-rows 与 --measure；
#   e165_replay_mutation_kill_bounds_selftest.py：--selftest（selftest 与三份假 cargo）。
#   弄坏开关的名单（BREAK_MODES 各表）与比较写在 driver 里，自证里读开关名的几处在 selftest 里，E165_KILL_BOUNDS_BREAK 照环境填进 BREAK 的那几行在本脚本的 python 里。
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
# 弄坏开关只在 --selftest 里认（文件头「弄坏开关」）：命令行第一个参数不是 --selftest、又不是自证起的子进程而下面的变量有一个非空（不管值是什么），
# 在参数解析之前退 2、点名它只在自证里认，一步都没做
for break_variable in E165_KILL_BOUNDS_BREAK; do
  if [[ -n "${!break_variable:-}" && "${1:-}" != --selftest && -z "${E165_KILL_BOUNDS_SELFTEST_ROOT:-}" ]]; then
    echo "  ✗ $break_variable 只在 --selftest 里认（弄坏开关只给自证证红用），这一趟不是自证，一步都没做" >&2
    echo "     → 怎么办：去掉 $break_variable 再跑；要证红就跑 $break_variable=<项> bash scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh --selftest" >&2
    exit 2
  fi
done
E165_KILL_BOUNDS_SELF_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
export E165_KILL_BOUNDS_SELF_PATH
exec python3 - "$@" <<'PYTHON'
import os
import re
import shutil
import subprocess
import sys

sys.dont_write_bytecode = True   # 下面载入的几份模块不往仓里写 __pycache__
SELF_PATH = os.environ["E165_KILL_BOUNDS_SELF_PATH"]
SCRIPT_DIRECTORY = os.path.dirname(SELF_PATH)
sys.path.insert(0, SCRIPT_DIRECTORY)
from e165_replay_mutation_kill_bounds_driver import (  # noqa: E402
    BREAK, BREAK_MODES, DEFAULT_ROOT, EXIT_OK, EXIT_USAGE, ITEM_LINES, Setup, Stop, WORK_DIRECTORY_NAME, default_jobs_and_threads, load_items,
    load_items_r3, load_items_r4, main_run, main_run_r3, main_run_r4, measure, refuse_existing_output, select_rows_action)
from e165_replay_mutation_kill_bounds_selftest import selftest  # noqa: E402
BREAK.update(filter(None, os.environ.get("E165_KILL_BOUNDS_BREAK", "").split(",")))   # 判法模块里的那一份（两边共用），照环境填
for mode in BREAK:
    if mode not in BREAK_MODES:
        sys.exit(f"  ✗ 不认识的弄坏开关：{mode}\n  → 怎么办：E165_KILL_BOUNDS_BREAK 只认 {'、'.join(BREAK_MODES)}")
    print(f"  ! 弄坏开关开着：{mode}（只给自证用，这一次的结果不算数）", file=sys.stderr)


def main(arguments):
    options = {"jobs": None, "threads": None}   # 没给的由 default_jobs_and_threads 补
    item_names = []
    scratch = output = ledger = measure_name = None
    root = DEFAULT_ROOT
    profile = "r2"
    pins = gate = None
    action = "run"
    recheck = False
    position = 0
    flags = {"--list-items": "list", "--selftest": "selftest", "--clean": "clean", "--select-rows": "select"}
    while position < len(arguments):
        argument = arguments[position]
        if argument in flags:
            action = flags[argument]
            position += 1
            continue
        if argument == "--recheck":
            recheck = True
            position += 1
            continue
        if position + 1 >= len(arguments) or argument not in ("--scratch", "--output", "--ledger", "--item", "--jobs", "--threads", "--measure", "--root", "--profile",
                                                              "--pins", "--gate"):
            raise Stop(EXIT_USAGE, f"  ✗ 不认识的参数或缺值：{argument}\n  → 怎么办：用法见文件头")
        value = arguments[position + 1]
        position += 2
        if argument == "--pins":
            if value != "r5":
                raise Stop(EXIT_USAGE, f"  ✗ --pins 只认 r5，收到「{value}」\n  → 怎么办：不给 --pins 是原来那三套钉值；第 5 次重跑第二段用 --pins r5")
            pins = value
        elif argument == "--gate":
            if value != "r2":
                raise Stop(EXIT_USAGE, f"  ✗ --gate 只认 r2，收到「{value}」\n  → 怎么办：不给 --gate 是取甲；第 5 次重跑第五段 5-乙 用 --profile r4 --pins r5 --gate r2")
            gate = value
        elif argument == "--profile":
            if value not in ("r2", "r3", "r4"):
                raise Stop(EXIT_USAGE, f"  ✗ --profile 只认 r2、r3 与 r4，收到「{value}」\n  → 怎么办：不给 --profile 是 r2 那一套；第三段用 --profile r3；第 4 次重跑第二段用 --profile r4")
            profile = value
        elif argument == "--scratch":
            scratch = os.path.abspath(value)
        elif argument == "--output":
            output = value
        elif argument == "--ledger":
            ledger = value
        elif argument == "--item":
            item_names.append(value)
        elif argument == "--root":
            root = os.path.abspath(value)
        elif argument == "--measure":
            action, measure_name = "measure", value
        else:
            if not re.match(r"^[1-9][0-9]*$", value):
                raise Stop(EXIT_USAGE, f"  ✗ {argument} 要正整数，收到「{value}」\n  → 怎么办：写成 {argument} 4")
            options["jobs" if argument == "--jobs" else "threads"] = int(value)
    options["jobs"], options["threads"] = default_jobs_and_threads(options["jobs"], options["threads"])
    if action == "selftest":
        return selftest()
    if gate is not None and (profile != "r4" or pins != "r5"):
        raise Stop(EXIT_USAGE, f"  ✗ --gate r2 只在 --profile r4 --pins r5 同给时认，这一次是 --profile {profile}、--pins {pins or '（没给）'}（登记 r5 修1.九乙第 1 条）\n"
                               "  → 怎么办：写成 --profile r4 --pins r5 --gate r2；这一次一条都没跑")
    selftest_root = os.environ.get("E165_KILL_BOUNDS_SELFTEST_ROOT")
    setup = Setup(selftest_root or root, selftest_root is not None, profile, pins, gate)
    options["pins_text"] = "".join(" " + word for word in setup.pins_arguments())
    options["gate_field"] = f" gate={gate}" if gate else ""
    if profile in ("r3", "r4") and (action in ("select", "measure") or recheck or ledger is not None):
        raise Stop(EXIT_USAGE, f"  ✗ {profile} 模式不认 --select-rows、--measure、--recheck 与 --ledger（登记 {profile} 9.3：红例账不落，选行在整轮开头做）\n"
                               f"  → 怎么办：{profile} 模式只有 --list-items、--clean 与整轮（--scratch、--output、--item、--jobs；每个变体的线程数不写 --threads，取每台默认重活线程数 ÷ --jobs，经 task-server/task submit 交时 --threads 当场被拒）")
    if action == "list":
        items = load_items_r4(setup)[0] if profile == "r4" else load_items_r3(setup)[0] if profile == "r3" else load_items(setup)[0]
        for item in items:
            print(f"{item['name']}\t{item['table']}\t{'+'.join(str(row) for row in item['rows']) or '-'}\t{' ‖ '.join(item['titles'])}")
        return EXIT_OK
    if action == "select":
        if output is None:
            raise Stop(EXIT_USAGE, "  ✗ --select-rows 要 --output <产物>\n  → 怎么办：产物进 research/results/，名字照登记 5.4")
        output_path = os.path.abspath(output)
        shown_path = os.path.relpath(output_path, setup.root) if os.path.commonpath([output_path, setup.root]) == setup.root else os.path.basename(output_path)
        return select_rows_action(setup, output_path, shown_path)
    if scratch is None or os.path.commonpath([scratch, setup.root]) == setup.root:
        raise Stop(EXIT_USAGE, "  ✗ 缺 --scratch，或草稿目录在仓里\n  → 怎么办：--scratch 给一个仓外的目录（副本与编译目录建在它下面的 kill-bounds/）")
    options["work"] = os.path.join(scratch, WORK_DIRECTORY_NAME)
    if action == "clean":
        if os.path.exists(options["work"]):
            print(subprocess.run(["du", "-sh", options["work"]], stdout=subprocess.PIPE, text=True).stdout.strip())
            shutil.rmtree(options["work"])
        # gate-lint:nocount 这一句是收尾删工作目录，判了多少写在各段的 E7RESULT 行与汇总里
        print(f"  ✓ 已删 {options['work']}")
        return EXIT_OK
    cap = os.environ.get("SINGLEFS_THREAD_CAP")
    if cap and cap.isdigit() and options["jobs"] * options["threads"] > int(cap):
        raise Stop(EXIT_USAGE, f"  ✗ jobs × threads = {options['jobs'] * options['threads']} 超过线程上限 {cap}\n  → 怎么办：压小 --jobs、去掉 --threads（不写时取 {cap} ÷ --jobs；经 task-server/task submit 交时 --threads 当场被拒）")
    os.makedirs(options["work"], exist_ok=True)
    if action == "measure":
        return measure(setup, options, measure_name)
    if profile == "r3":
        if output is None:
            raise Stop(EXIT_USAGE, "  ✗ 缺 --output\n  → 怎么办：产物进 research/results/，名字照登记 r3 5.8（前缀 e165-replay-bounded-enumeration-<日期>-r3-segment3-）")
        options["output"] = os.path.abspath(output)
        options["output_shown"] = os.path.relpath(options["output"], setup.root) if os.path.commonpath([options["output"], setup.root]) == setup.root else os.path.basename(options["output"])
        refuse_existing_output(setup, options["output"], options["output_shown"])
        return main_run_r3(setup, options, item_names)
    if profile == "r4":
        if output is None:
            raise Stop(EXIT_USAGE, "  ✗ 缺 --output\n  → 怎么办：产物进 research/results/，名字照登记 r4 5.7（前缀 e165-replay-bounded-enumeration-<日期>-r4-segment2-）")
        options["output"] = os.path.abspath(output)
        options["output_shown"] = os.path.relpath(options["output"], setup.root) if os.path.commonpath([options["output"], setup.root]) == setup.root else os.path.basename(options["output"])
        refuse_existing_output(setup, options["output"], options["output_shown"])
        return main_run_r4(setup, options, item_names)
    if output is None or ledger is None:
        raise Stop(EXIT_USAGE, "  ✗ 缺 --output 或 --ledger\n  → 怎么办：两个都给；产物进 research/results/，名字照登记 5.5")
    if recheck and not item_names:
        raise Stop(EXIT_USAGE, "  ✗ --recheck 要用 --item 点名\n  → 怎么办：--recheck --item <变体>")
    options["output"], options["ledger"] = os.path.abspath(output), os.path.abspath(ledger)

    def shown(path):
        return os.path.relpath(path, setup.root) if os.path.commonpath([path, setup.root]) == setup.root else os.path.basename(path)

    options["output_shown"], options["ledger_shown"] = shown(options["output"]), shown(options["ledger"])
    if not recheck:   # --recheck 拿新跑的行与已有的产物比，产物与账本来就该在
        refuse_existing_output(setup, options["output"], options["output_shown"], options["ledger"], options["ledger_shown"])
    return main_run(setup, options, item_names, recheck)


try:
    exit_code = main(sys.argv[1:])
except Stop as stop:
    print(str(stop))
    exit_code = stop.code
if ITEM_LINES["report"] is not None:
    exit_code = ITEM_LINES["report"].close(exit_code)
sys.exit(exit_code)
PYTHON
