#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓、此刻的 Verus 安装与 cargo 缓存，上一次的结论不替这一次作保
# run-condition: none 要的 Verus、cargo 与装置由各格的判法自己判，缺了判红；核还没进仓时 Verus 那四格报本次未跑
# gate-stage: 形式化验证（Verus 证明恢复重放取前缀的核，版本钉住、空真与必红世界按落点判；E165 的红例钉进门禁；两条本该独立的腿的同源性计数器）
# gate-fixture: yes
# gate-category: checker-tier 类
# gate-similar: lkmm.sh 同是起外部验证器的一道，接法相同（判法放脚本、每条判定按单元键进判定库 scripts/crash/lib_verdict_library.py）；它起 herd7 判 litmus，这里起 Verus 判 crates/ 的证明，并进去会让两件不相干的事共用一份判定库的种类与出路
# gate-similar: crates-mutation-replay.sh 它逐条改坏 crates/ 要求点名的测试红；这里的必红世界要求 Verus 报证明错，写不进它的 crates/mutations.tsv
# gate-similar: code-source-discipline.sh 它的 mutation-tables 格静态判实验二进制的变异表成形、锚点唯一；这里判 E165 钉住红例的表本身、并逐行在今天的实现上重放
# gate-similar: research-build-and-replay.sh 它的 experiment-replay 格整份复跑入库实验比产物；这里只逐行重放钉住的红例
# gate-similar: crash-point-replay.sh 它判崩溃点重放与登记的崩溃枚举用例；这里判证明与有界穷举的红例，不起崩溃枚举
# gate-similar: checker-independence-and-sync.sh 它判 checker 与实现、RefFS 与实现的依赖闭包不相交（结构上的独立）；same-origin-counter 格数的是两条腿运行时的不一致事件（统计上的独立），读 divergence-events.tsv 一张表、不看依赖，并进它会让一道结构门禁去读事件账
#
# 格名表（--list 打它，--check <格名>[,<格名>…] 只跑点名的格，不给就七格按下面的次序全跑，格名写错退 2）：
# gate-cell: verus-toolchain-pinned Verus 安装的 version 与 commit、Cargo.lock 里六个包的版本、本机 cargo 缓存里包源码的出处与内容指纹、Verus 与产品的 rustc 版本都与登记相同
# gate-cell: verus-proof Verus 逐项验登记的各 crate 被验区域里有证明义务的单元（按全路径：proof fn、exec fn、带 decreases 的 spec fn、derive(Clone) 的 clone，带值的 const 与 assume_specification 各成一项，impl 与 trait 里的方法与关联 const 也算），每项出结局、errors 0、verified 至少 1（指向函数的 assume_specification 验出 0 条也算过），各 crate 各项 verified 之和等于它登记的数，整格再完整验证 crate、产品构建与 verus --compile 构建的 MIR 逐条目相同；起 Verus 之前判被验区域：可信声明与 cfg 原子照信任清单逐条登记，axiom、作弊标记与 verus! 外面被验类型的 trait impl 判红
# gate-cell: verus-vacuity 核里每一处 requires 的函数体头插 assert(false) 都验不过（未检出入口空真；可满足性另由构造见证证明）
# gate-cell: verus-must-red-worlds 先查函数与不变式负例覆盖；必红世界表每行改坏核之后 Verus 都报错、证明错落在第六段 landing= 登记的函数里（条数只报不判），原文恰好命中一次
# gate-cell: pinned-red-examples-table 钉表 e165-pinned-red-examples.tsv 自己：每行 22 段整行抄带档、第 1 段与第 2、3、7 段重算的指纹相同、抄来的账行与自己的段对得上、第 22 段是规范写法的档，点名的变异还在它的表里；不读红例账、kill-bounds 产物与 research/ 下任何文件
# gate-cell: pinned-red-examples-replay 钉住的每一行是 22 段整行抄带档、抄来的账行与自己的段对得上，点名的变异还在它的表里，装置按档在今天的实现上逐行打的结果与原实现期望的逐态行逐字节相同
# gate-cell: same-origin-counter 同源性计数器：不一致事件表 divergence-events.tsv 里每个已立的对累计 ≥ 5 次不一致而判「spec 错」0 次即判红；未立的对报未立、不计入，条款欠一句不计入累计
# 参数解析、每格一个子 shell、汇总与退出码经共用库 lib/stage-cells.sh。
#
# 判法都在两份脚本里（判据、默认值、弄坏开关写在各自文件头），这一道只接进门禁、给样本开关：
#   Verus 四格：scripts/gate/verus-proof-check.py <toolchain|proof|vacuity|must-red-worlds> <根>，登记表 .claude/gate.d/verus-pins.tsv（crate 行可以多条：
#     replay-prefix 直接起 verus，singlefs-core 这一种第四段写 cargo、经 cargo verus focus 验），信任清单 .claude/gate.d/verus-trust-list.tsv（verus-proof 格判）。
#     核没进仓（登记的核不在、crate 不依赖 vstd、src 里没有 verus!）时四格都退 77；核不在而 crate 已经接了 Verus 判红。
#     读的环境变量：GATE_VERUS_JOBS（几处同时验，没设照判法文件头 --jobs 的默认）、
#     SINGLEFS_THREAD_CAP（线程，没设取登记的重活默认并行度 .claude/heavy-thread-default）、SINGLEFS_VERUS_ROOT（Verus 安装目录，盖过登记表）。
#     判法直接起 Verus，不套内存包装、不设每个进程的内存上限，也不按内存收几处同时验：整条命令的内存额度与线程上限由 task-server 给（task-server/README.md「提交写法」）。
#     不限时：卡住由 task-server 判、停掉记 stalled，Verus 没出结局就没有判定，不当通过。
#   红例两格：scripts/experiment-apparatus/e165-pin-red-examples.py --check-table / --replay，表的格式写在 .claude/gate.d/e165-pinned-red-examples.tsv 表头；
#     table 格只读钉表与变异表，不读红例账、kill-bounds 产物与 research/ 下任何文件（钉全由 E165 第二段驱动 scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh
#     写完一对账与产物时当场判，写法在它文件头「钉红例」一段）；每行的 ㉒ 是出那份账的那一趟的档，replay 按档分组起装置、每组带 ㉒ 的参数（例 --profile r2）。
#
# 复用：每条（证明按函数、空真每处 requires、必红世界每行、钉表每行）一条判定，键是这一条涉及的 Verus 函数单元（被验 fn、它 requires / ensures 用到的
#   spec fn 与 proof fn）去注释后的指纹、Verus 与 vstd 版本；钉表行的键是那一行档与参数重放真走到的装置函数单元闭包、状态输入、期望逐态行、变异原文与替换文；
#   空真项身份是所属函数单元指纹加函数内第几处，不带路径与行号。格不复用，不问 stage-must-run.sh，不写全绿标记：一格判绿就是它的每一条判定都判绿
#   （现判的或复用的），verus-toolchain-pinned、pinned-red-examples-table 与 same-origin-counter 每趟现判（只读几份文件，几秒）。必红世界每行的键里是改坏的原文、替换文与第六段落点（landing=），第五段条数只报、不进键。
#   每条判定的键、判法单元指纹（判结局的那几个函数的单元闭包键，判定库与 lib_function_units.py 不进）写在两份判法脚本文件头「逐项点名与按条复用」，
#   存放、组成清单与作废时报出变了的单元写在判定库 scripts/crash/lib_verdict_library.py 文件头。
#   判定库：只有 prune-server 那一份（verdict-library 族，种类 singlefs-formal-unit-verdicts）：库名在两份判法里固定，库根不进键，这一道不给库的位置；
#     取不到库名、服务连不上判法整趟退出（连不上退 75），这一道照它的码退、不当现判跑。一项判完当场写进服务，被打断之后同一条命令重跑只补没判的。
#     样本档在临时 prune-server 底下起（scripts/prune-server/prune_server_client.py 文件头「命令行」--with-temporary-service）：样本的判定另占临时服务里一份命名空间，不写真服务。
#     判红的那一项照同一把键记红、撤掉原有的绿，下一趟现判（判法两份脚本文件头「逐项点名与按条复用」的记红撤绿）。
#     整道、--check、--item 都读（键相同就复用），现判的照键写回同一份库。
#   每项一行「· <项>：绿|红，现判（<原因>：<变了的单元>）|复用」，每格一行「· 按条：现判 N、复用 M（现判的原因：…）」，整道末了一行「── 按条（…）：现判 N、复用 M」。
# 逐项点名（.claude/rules/verification.md「门禁的结构」里门禁按项点名的 --list-items / --item）：
#   --list-items 逐行打「项<TAB>格[<TAB>说明]」：toolchain、proof（整格）、table 各一项，证明每个有证明义务的单元一项 proof:<fn 名或 <类型>::clone>，空真每处 requires 一项
#     vacuity:<所属 fn>#<函数内第几处>，必红世界每行一项 must-red:<变异名>，红例钉表每行一项 pinned:<钉表行号>；只读登记表、核与两张表，不编、不起 Verus 与装置。
#   --list-items --with-keys 另逐项照现在的键比 prune-server 那一份库（只读库、不写库、不起 Verus 与装置、不编 cargo；Verus 要装着，键里的工具链读它的安装）：
#     verus-proof-check.py list-items --key-mismatches 每格一行「<格>：键对不上 N 条、无记录 M 条、键相同 K 条、拒算 R 条」与末了「键对不上共 N 条」原样接在项的后面、走 stderr
#     （stdout 项的那几行不变）；它没报键数（核没进仓）时末了说一句「列项没比键」。--item 照列出的项找格，不比键。
#     按项机读另打（stdout，接在项的那几行后面；项名就是上面那一行第一段，每项一行，条数与不带 --with-keys 的 --list-items 相等）：
#       ITEM_PLAN item=<项名> to_judge=<要现判的条数> reusable=<能复用的条数>
#     条是一条按键入库的判定。proof:*、vacuity:*、must-red:*、pinned:* 每项一条：此刻的键在库里键相同的 to_judge=0 reusable=1，键对不上 / 无记录 / 拒算的 to_judge=1 reusable=0；
#     toolchain、table 每趟现核不入库，proof（整格这一项：各项之和、完整 crate 验证、MIR 比对，都不按条入库）每趟现判：这三项一律 to_judge=1 reusable=0。
#     两份判法逐项往 stderr 打「ITEM_KEY item=<项名> state=<键相同|键对不上|无记录|拒算>」，这一道把它们折成 ITEM_PLAN。不带 --with-keys 的 --list-items 不比键、不连库；
#     --with-keys 只和 --list-items 一起用（退 2）。每项的键真算、真去库里比过才写（不照抄常数）。
#   --list-items --with-keys --item <项>（可给几次）：只比点名那几项的键、只打那几项的项行与 ITEM_PLAN（成本随点名项数走；任务服务每收到点 --item 的提交先跑这一条估用时）；
#     点名的项一份判法里一项都没有的，那份判法不连库不算键；项名写错退 2 并列出可点名的项；不带 --item 照旧整表。末行「按项机读：ITEM_PLAN N 行，与点名项数 N 相同」。
#     --item 不带 --with-keys 的 --list-items 仍退 2。
#   真跑（整道、--check、--item）每一项判完在 stdout 打一行（整道末了、按项的次序；同一项在几个 crate 里各判一次的合成一行：秒数、条数相加，退出码取大）：
#       ITEM_DONE item=<项名> seconds=<这一项自己的挂钟秒数> judged=<现判条数> reused=<复用条数> exit=<这一项的退出码>
#     条数等于这一趟跑到的项数（--item 点了几项就几行；没跑到的格不打）。几项同时验时 seconds 是那一项自己的挂钟，不是整趟的：Verus 那几格是每一处从起到出结局的挂钟
#     （verify_in_parallel 的 one），复用的 0 秒；红例重放一块里几行共用一次装置，一块里每行记那一块的秒数；proof（整格这一项）是它自己的活（各项之和、完整 crate 验证、MIR 比对）的秒数；
#     toolchain、table 是各自那一格的挂钟。judged / reused 是按键入库的判定条数，现判 1 复用 0 或现判 0 复用 1；exit 绿 0、红 1、本次未跑 77
#     （toolchain、table 取格的退出码；proof 取这一格有没有问题）。两份判法往 FORMAL_ITEM_DONE_FILE 追加「项<TAB>秒<TAB>现判<TAB>复用<TAB>退出码」，这一道收尾转成 ITEM_DONE。
#   --item <项>（可给多次，取并集）只跑点了项的那几格，证明、空真、必红世界、红例重放四格只判点名的项（proof 一项是整格）；项名写错、缺项名退 2 并列出可点名的项；
#     与 --check、--list-items 不一起用（退 2）。
#     证明那一格同时收到 proof 与 proof:<名> 时照整格判（点名的加在整格之上，各项之和、完整 crate 证明与 MIR 比对照跑；verus-proof-check.py 的 check_proof，
#     弄坏开关 VERUS_PROOF_CHECK_BREAK=proof-item-narrows 下只验点名的那一个）。
# 样本（fixtures/formal-verification.sh/<格名>-<情形>/，各放 .gate-cells 点名一格；共享门禁的 stage-selftest.sh 跑）。名字读不出判法的几份：
#   verus-toolchain-pinned-*：假的 Verus 安装与假的 cargo 缓存放在样本的 .formal-sample/ 下
#   verus-must-red-worlds-list-items：只列项；-list-items-key-counts：列项照临时服务里空的那一份比键（.formal-sample/with-keys），每格「无记录」几条、末了「键对不上共 0 条」
#     （读 Verus 的安装算工具链那一样，不起 Verus）；-list-items-with-keys：同一份核，库是空的，每项一行 ITEM_PLAN（每项 to_judge=1 reusable=0）、行数与项数相同；
#     -list-items-plan-reusable：.formal-sample/plan-after-run 先把证明、空真、必红世界三格用假工具判一趟，再比键：判过的 5 项 ITEM_PLAN 是 to_judge=0 reusable=1，
#     toolchain、proof、table 与 22 行钉表仍是 to_judge=1 reusable=0
#   verus-must-red-worlds-list-items-with-keys-one-item / -two-items：同一份核，.formal-sample/items 点名一项（证明 clamp_to_limit）/ 两项（必红世界一行、钉表 pinned:21）：
#     只打这几项的 ITEM_PLAN，Verus 那份报「1 格比了 1 项」（不算别的项的键；两项那份里钉表那一项由红例那份算、Verus 那份只比必红世界这一项）
#   verus-proof-item-done：证明那一格真跑（假 Verus），clamp_to_limit 的函数体带 SELFTEST_SLEEP_30（验到这一行睡 3 秒）、smaller_of_two 没有：每项一行 ITEM_DONE，
#     慢的那一项 seconds=3.x、快的那一项 seconds=0.x（各项自己的挂钟，不是整道的），judged=1 reused=0 exit=0，整格 proof 一行
#   verus-proof-item-done-reused：同一份核、.formal-sample/run-twice：第二趟两项全复用，ITEM_DONE 是 seconds=0.0 judged=0 reused=1 exit=0
#   verus-proof-item-done-red：核里一处 SELFTEST_BROKEN，证明错的那一项 exit=1、另一项 exit=0、整格 proof exit=1
#   verus-proof-killed-red：替身 Verus 起来就 kill -9 自己，判红，✗ 写「Verus 被信号停了」；
#   pinned-red-examples-replay-killed-red：桩装置打一行就 kill -9 自己，判红，✗ 写「装置被信号停了」；这两份另核汇总出路里「被信号停了」那一句（不改证明或实现）
#   verus-proof-kernel-absent（核没进仓）：退 77
#   pinned-red-examples-replay-rocksdb-entry-refused：没有桩，放依赖 rocksdb 的 crates/singlefs-verification/Cargo.toml：
#     起装置的 cargo build 之前 rocksdb-prebuilt.py entry 判红（样本根没有 Cargo.lock，进程环境有没有取值都红），这一格退 1、装置没编
#   verus-vacuity-closure-red：外层函数的前置条件可满足、里面闭包的不可满足
#   verus-vacuity-list-items-assert-by-requires-green（只列项）：函数体里一处 assert(…) by (nonlinear_arith) 接 requires … ;，不是空真插入点，列项退 0、
#     proof 那一行报「跳过 assert-by requires 1 处」并点名那一行（开关 VERUS_PROOF_CHECK_BREAK=assert-by-requires-as-insertion-point 下列项退 1，判错）；
#   verus-vacuity-list-items-requires-on-plain-block-red（只列项）：requires 挂在普通 proof 块后面、下一行只有 ;，照旧判红，红句点名文件与行
#     （开关 VERUS_PROOF_CHECK_BREAK=assert-by-requires-by-semicolon-line 下列项退 0、FORMAL_VERIFICATION_BREAK=list-items-red-swallowed 下红句不回显，都判错）
#   verus-proof-list-items-trait-default-collision-red（只列项）：判法的红句原样回显，want 点名撞的 Limited::LIMIT 与 Ordinal::LIMIT 两项
#   verus-proof-list-items-spec-const（只列项）：impl 里与自由的 spec const 各一个，与别的带值的 const 一样成项，成项 5 个（真 Verus 整 crate 验出 5 条；
#     开关 VERUS_PROOF_CHECK_BREAK=spec-const-not-item 下两个 spec const 不成项，判错）
#   verus-must-red-worlds-：-rlimit-red 改坏之后只剩 rlimit 用尽；-rlimit-with-proof-error-green 另有一条证明错（proof_errors=1，落点是那条证明错所在的函数）；
#     -expectation-form-red 第五段写成 proof_errors=0（不是正整数）；-proof-error-count-drift-green 一条证明错、第五段写 proof_errors=2：条数对不上而落点没变，只报不判，判绿；
#     -landing-moved-red 条数对得上而第六段落点登记成别的函数，判红；-not-six-segments-red 只有五段的行，判红、报行号、要六段（C654）
#   same-origin-counter-*：只放 .claude/gate.d/divergence-events.tsv（不起 Verus）；文件头「弄坏开关」一段点名各让哪份判错
#   pinned-red-examples-table-green：根里没有 research/，钉表三行好，判绿；-table-archived-ledger-green：research/results/ 下有一份账、同一趟的 kill-bounds 产物不在（归档了），
#     钉表好，照判绿（这一格不读账与产物）；-table-red：一行的 ① 与 ②③⑦ 重算的不同；-table-variant-red：⑥ 点名的变异不在 crates/mutations.tsv 里了；
#     -table-profile-red：一行的 ㉒ 写成「--profile r1 --table direct」（默认值也写了，不是规范写法）
#   pinned-red-examples-replay-copied-columns-red：⑳ 与 ⑦ 不同；-replay-columns-red：有一行只有 21 段（少了 ㉒），判红、报行号与要 22 段、出路是从 git 找回
#   pinned-red-examples-replay-profiles-green：r1、r2 两种 profile 各三行，桩照命令行的 --profile 打（r2 档在 counters 后面多一个 j6qn），判绿；
#   Verus 那几份样本起的是假工具，不起真 Verus 与真 cargo：样本根（登记表 samples:.formal-fake-tools 的假工具目录）里 .formal-fake-tools/ 由这一道开跑时写
#   （verus-proof-check.py write-fake-tools：verus、cargo、cargo-verus 三个假工具，就是那份脚本自证用的，按核源码里的标记出判定；version.json 抄样本登记表的 release 行），
#   --verus-root 指到它、PATH 前面放它（产品构建的 cargo rustc 也走假的）。样本自带 verus-stand-in/（verus-proof-killed-red）或 .formal-sample/verus-install/（版本一致那几份）的，
#   用样本自己的，不写 .formal-fake-tools。要真 Verus 的判别力（真证明红绿）不在样本档：整道真跑时 verus-proof-check.py 判真 Verus。
#   假 Verus 认的标记写在 scripts/gate/verus-proof-check.py 的 FAKE_VERUS 文件头（SELFTEST_BROKEN 当一处证明错、SELFTEST_VACUOUS 当空真、SELFTEST_NEEDS_RLIMIT_<n> 当要至少 n 的 rlimit、
#   SELFTEST_SLEEP_<n> 验到这一行睡 n/10 秒、SELFTEST_KILLED 起来就 kill -9 自己 等）：要验得红的样本核里带这些标记，必红世界表的替换文也带。
#   被验区域与多 crate 的那几份（要拦的写法取自 research/prompts/m4-exit5-relocation-proof-r2-opus-model/verus/ 与
#   research/prompts/m4-exit5-relocation-proof-r3-sonnet-model/verus/ 里的例子，样本核的文件头写着取自哪一份）：
#     verus-proof-trust-unregistered-red（binary_search 的 assume_specification 没登记）、-trust-registered-green（登记齐了：规格原文 sha256、差分测试名，
#     不带 --no-cheating 照验、MIR 比对相同）、-trust-digest-red（登记的 sha256 不对）、-trust-test-missing-red（差分测试不在）、-trust-vstd-red（vstd 行版本不对）、
#     -axiom-red（axiom fn）、-axiom-broadcast-red（broadcast axiom）、-cfg-feature-red（#[cfg(feature)] 分叉的两份 pick）、-cfg-debug-assertions-red（cfg!(debug_assertions)）、
#     -cfg-macro-red（宏在 verus! 外面定义、展开里带 cfg!）、-mir-divergence-red（同一个宏的 cfg 登记了，静态那一关过了，红在 MIR 比对上：产品构建 start + 1、
#     verus --compile 编出 start + span）、-trait-impl-outside-red（被验 trait 在 verus! 外面的 impl；cfg(test) 下的不判）、-external-trait-spec-red
#     （external_trait_specification 没登记）、-verifier-external-red（#[verifier::external]）、-attribute-kernel-red（#[verus_verify] 标的文件没登记成核）、
#     -core-function-green（登记一个 core 小函数：crate 行第四段 cargo、带路径依赖 singlefs-format，impl 里的方法照全路径成项，cargo verus build 的 MIR 相同）；
#     verus-must-red-worlds-list-items-two-crates（两个 crate 逐个列项、全路径不重名）；verus-toolchain-pinned-rustc-red（rustc 行的产品 rustc 版本与 .formal-sample/rustc-version 不同）
# 只供样本的开关（被判的根不是这道阶段所在的仓、也没设 GATE_STAGED_FROM 时才认；真仓与 --staged 下放了也不认）：
#   .formal-sample/verus-install/         版本一致格改读这里的 version.json（不起它）
#   .formal-sample/cargo-home/            版本一致格改读这里的 registry/src/*/
#   .formal-sample/rustc-version          版本一致格把这份文件当产品 rustc -vV 的输出（不起 rustc；样本的 rustc 行照它写）
#   .formal-sample/e165-stub.sh           replay 格改用这个桩当装置（经 bash 起）
#   .formal-sample/list-items             当成给了 --list-items（样本没法带参数）
#   .formal-sample/items                  每一行当成一个 --item（命令行给了 --item 就不看它）
#   .formal-sample/with-keys              当成给了 --with-keys（样本没法带参数；与 .formal-sample/list-items 一起放）
#   .formal-sample/plan-after-run         列项之前先把证明、空真、必红世界三格真跑一趟（假工具；判定记进临时 prune-server 里这一份样本的命名空间），再列项比键：
#                                         键相同的项 ITEM_PLAN 是 reusable=1（样本 verus-must-red-worlds-list-items-plan-reusable）
#   .formal-sample/run-twice              整道接连跑两趟（判定在临时 prune-server 里这一份样本的命名空间）：第二趟按条全复用（按条复用的样本）
# 弄坏开关：格的判法那一份的开关 VERUS_PROOF_CHECK_BREAK=<项> 与 E165_PIN_RED_EXAMPLES_BREAK=<项>（逐项让哪份样本判错写在两份脚本的文件头）；
#   汇总与 --check 的归共用库（STAGE_CELLS_BREAK=red-swallowed 让每份红样本整道退 0；marker-ignored 让每份样本七格都跑，汇总那条「跑了 1 格」找不到）。
#   逐项点名与按条复用：判法那一半的开关在两份脚本文件头「逐项点名与按条复用」；这一道自己接的那一半 FORMAL_VERIFICATION_BREAK=<项>[,<项>…]，
#   一项一行「FORMAL_VERIFICATION_BREAK=<值>  弄坏哪一处」，下一行起写打开之后判错的样本（这一半没有脚本自证，复跑方只有这一道的样本档）：
#   FORMAL_VERIFICATION_BREAK=items-unchecked  --item 写错的项名悄悄丢掉、不退 2
#       pinned-red-examples-replay-item-unknown
#   FORMAL_VERIFICATION_BREAK=item-plan-lines-skipped  --list-items --with-keys 不打 ITEM_PLAN 行
#       verus-must-red-worlds-list-items-with-keys、verus-must-red-worlds-list-items-plan-reusable（找不到 ITEM_PLAN 行）
#   FORMAL_VERIFICATION_BREAK=item-plan-unkeyed  --with-keys 不看 ITEM_KEY、每项一律 to_judge=1 reusable=0（不比键的样子）
#       verus-must-red-worlds-list-items-plan-reusable（键相同的项没报 reusable=1）
#   FORMAL_VERIFICATION_BREAK=item-plan-count-short  --with-keys 少打最后一项的 ITEM_PLAN（行数与项数对不上）
#       verus-must-red-worlds-list-items-with-keys（整道退 1，找不到「与项数 N 相同」）
#   FORMAL_VERIFICATION_BREAK=item-plan-ignores-item  --list-items --with-keys 带 --item 也比整表、打整表
#       verus-must-red-worlds-list-items-with-keys-one-item、verus-must-red-worlds-list-items-with-keys-two-items（整道退 1：ITEM_PLAN 行数对不上点名项数）
#   FORMAL_VERIFICATION_BREAK=item-done-lines-skipped  真跑收尾不打 ITEM_DONE 行（toolchain、table 两格也不记）
#       verus-toolchain-pinned-green、pinned-red-examples-table-green、verus-proof-item-done
#   FORMAL_VERIFICATION_BREAK=item-done-whole-run-seconds  ITEM_DONE 的秒数一律写整道从起到收尾的挂钟，不是各项自己的
#       verus-proof-item-done（快的那一项秒数不再是 0.x）
#   FORMAL_VERIFICATION_BREAK=rocksdb-entry-unchecked  pinned-red-examples-replay 不带 --binary 时起装置的 cargo build 之前不调 scripts/hosts/rocksdb-prebuilt.py entry
#       pinned-red-examples-replay-rocksdb-entry-refused（输出里找不到「rocksdb-prebuilt entry：」）
#   FORMAL_VERIFICATION_BREAK=list-items-red-swallowed  列项（--list-items、--item 都先列项）退非 0 时不回显判法打在 stdout 的 ✗ 与 → 行，只剩「列不出 … 退 1」（只在样本根上认，
#       echo_listing_red_lines；admission.py 的自证不盯它，靠这两份样本）：
#       verus-proof-list-items-trait-default-collision-red（找不到点名撞的那两项）、
#       verus-vacuity-list-items-requires-on-plain-block-red（找不到「第 17 行的 requires 找不到函数体的 {」）
#   被验区域与多 crate（判法在 scripts/gate/verus-proof-check.py，那份文件头「被验区域与多 crate 的弄坏开关」一段另列打开之后自证哪几格判错）的开关，
#   开关一行、下一行起写打开之后判错的样本：
#   FORMAL_VERIFICATION_BREAK=single-crate-registry  登记表只认第一个 crate
#       verus-must-red-worlds-list-items-two-crates（core 那几项列不出来）
#   FORMAL_VERIFICATION_BREAK=cargo-mode-ignored  crate 行第四段 cargo 不认、照直接起 verus 判
#       verus-proof-core-function-green（判红：除 vstd 之外还依赖 singlefs-format）
#   FORMAL_VERIFICATION_BREAK=methods-refused  impl 与 trait 里的方法判红、要挪成自由 fn
#       verus-proof-core-function-green、verus-must-red-worlds-list-items-two-crates、
#       verus-proof-trait-impl-outside-registered-green、verus-proof-list-items-trait-declaration-struct-literal（impl 与 trait 里的方法判红）
#   FORMAL_VERIFICATION_BREAK=attribute-mode-unrecognized  #[verus_verify] 标的文件不当核
#       verus-proof-attribute-kernel-red（判绿）
#   FORMAL_VERIFICATION_BREAK=no-cheating-kept  照旧带 --no-cheating
#       verus-proof-trust-registered-green（判红：Verus 拒 assume_specification）
#   FORMAL_VERIFICATION_BREAK=trust-list-unchecked  信任清单不判
#       verus-proof-trust-unregistered-red、verus-proof-external-trait-spec-red、verus-proof-trust-test-missing-red、verus-proof-trust-digest-red、verus-proof-trust-vstd-red、
#       verus-proof-trust-digest-last-fn-in-impl-red（判绿）
#   FORMAL_VERIFICATION_BREAK=trust-digest-unchecked  不比登记的规格原文 sha256
#       verus-proof-trust-digest-red、verus-proof-trust-digest-last-fn-in-impl-red（判绿）
#   FORMAL_VERIFICATION_BREAK=trust-test-unchecked  不核差分测试名
#       verus-proof-trust-test-missing-red（判绿）
#   FORMAL_VERIFICATION_BREAK=trust-vstd-unchecked  不核信任清单的 vstd 行
#       verus-proof-trust-vstd-red（判绿）
#   FORMAL_VERIFICATION_BREAK=axiom-accepted  核里的 axiom 不判
#       verus-proof-axiom-red、verus-proof-axiom-broadcast-red（找不到「核里有 axiom」）
#   FORMAL_VERIFICATION_BREAK=cfg-atom-accepted  被验区域里的 cfg 原子不认
#       verus-proof-cfg-feature-red、verus-proof-cfg-debug-assertions-red、verus-proof-cfg-macro-red（找不到「没登记进信任清单」）
#   FORMAL_VERIFICATION_BREAK=cfg-macro-unchecked  定义里有 cfg 的宏在被验区域里的调用不认
#       verus-proof-cfg-macro-red（只剩 MIR 比对那一句红，找不到「#plus_span! 没登记进信任清单」）
#   FORMAL_VERIFICATION_BREAK=trait-impl-outside-accepted  被验 trait 在 verus! 外面的 impl 不判
#       verus-proof-trait-impl-outside-red、verus-proof-trait-impl-outside-unregistered-red、verus-proof-trait-impl-outside-moved-red（判绿）、
#       verus-proof-trait-impl-outside-registered-green（找不到「都登记在…放过」）
#   FORMAL_VERIFICATION_BREAK=mir-compare-skipped  不比产品构建与 Verus 构建的 MIR
#       verus-proof-mir-divergence-red（判绿）、verus-proof-trust-registered-green、verus-proof-core-function-green、verus-proof-green、
#       verus-proof-mir-panic-path-green（找不到 MIR 那一句）
#   FORMAL_VERIFICATION_BREAK=rustc-pin-unchecked  不核登记表 rustc 行
#       verus-toolchain-pinned-rustc-red（判绿）
#   同源性计数器格（same-origin-counter；判法在本文件 cell_same_origin_counter，表是 .claude/gate.d/divergence-events.tsv）的开关，开关一行、下一行起写打开之后判错的样本：
#   FORMAL_VERIFICATION_BREAK=same-origin-zero-spec-wrong-unchecked  不判「累计 ≥ 5 次而 spec 错 0 次」
#       same-origin-counter-red（判绿）
#   FORMAL_VERIFICATION_BREAK=same-origin-clause-gap-counted  条款欠一句也计入累计
#       same-origin-counter-clause-gap-green（判红）
#   FORMAL_VERIFICATION_BREAK=same-origin-unbuilt-reported-green  未立的对当已立的对数、报绿
#       same-origin-counter-green（找不到「已立 2 对、未立 1 对」）、same-origin-counter-red（汇总里「未立 1 对」找不到）、same-origin-counter-unbuilt-event-red（判绿）
#   FORMAL_VERIFICATION_BREAK=same-origin-unbuilt-events-accepted  未立的对收了事件也不判红
#       same-origin-counter-unbuilt-event-red（判绿）
#   红例两格的开关，开关一行、下一行起写打开之后判错的样本（自证格写在 e165-pin-red-examples.py 文件头）：
#   E165_PIN_RED_EXAMPLES_BREAK=table-check-globs-research  table 格取 research/results/e165-*-red-examples.tsv、按文件名配产物判钉全
#       pinned-red-examples-table-archived-ledger-green（判红：产物不在）、pinned-red-examples-table-green（退 77：没有账）
#   E165_PIN_RED_EXAMPLES_BREAK=fingerprint-unchecked  不拿 ②③⑦ 重算 ①
#       pinned-red-examples-table-red（判绿）
#   E165_PIN_RED_EXAMPLES_BREAK=variant-unchecked  table 格与 replay 格不判变异还在不在
#       pinned-red-examples-table-variant-red、pinned-red-examples-replay-variant-red（判绿）
#   E165_PIN_RED_EXAMPLES_BREAK=profile-form-unchecked  不核 ㉒ 是不是规范写法的档
#       pinned-red-examples-table-profile-red（判绿）
#   E165_PIN_RED_EXAMPLES_BREAK=copied-columns-unchecked  不核 ⑪⑫⑬ 与 ②③④、⑳㉑ 与 ⑦⑧
#       pinned-red-examples-replay-copied-columns-red（判绿：桩照 ⑦ 打）
#   E165_PIN_RED_EXAMPLES_BREAK=replay-without-profile  replay 格起装置不带 ㉒ 的档参数
#       pinned-red-examples-replay-profiles-green（判红：桩一律按 r1 打，r2 那三行与 ⑦ 不同）
#   E165_PIN_RED_EXAMPLES_BREAK=row-columns-unchecked  段数不是 22 的行不报，当表里没有这一行
#       pinned-red-examples-replay-columns-red（判绿：桩照 ⑦ 打）
#
#   bash .claude/gate.d/formal-verification.sh [项目根]
#   bash .claude/gate.d/formal-verification.sh --list        逐行打格名与判什么，不起 Verus、不编
#   bash .claude/gate.d/formal-verification.sh --list-items  逐行打可点名的项与它属于哪一格，不起 Verus、不编
#   bash .claude/gate.d/formal-verification.sh --item <项> [--item <项>…] [项目根]   只跑点名的项
#   bash .claude/gate.d/formal-verification.sh --tool-versions   打 Verus 的版本与安装文件的 sha256、安装目录顶层的整体摘要，VERUS_Z3_PATH、VERUS_ROOT、
#     VERUS_USE_RUSTUP 三个变量的取值，以及 Verus 起 rust_verify 用的那条 rustup 工具链：RUSTUP_HOME 与它解析到的路径、工具链 lib/ 与 lib/rustlib/*/lib/ 下全部文件的
#     整体摘要、rustup 程序的 sha256（Verus 每条判定的键里「工具链」那一样就是这一行的摘要：工具链里换了一个文件，Verus 的每一条都现判；
#     逐样的写法在 scripts/gate/verus-proof-check.py 文件头 tool-versions 一段；不起 Verus；本机多哈希约 500 MB，页缓存热时多约 0.25 秒）
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
if ! declare -F preflight >/dev/null; then
  echo "  ✗ formal-verification.sh：source .claude/scripts/preflight.sh 没成（上面一行报了找不到），preflight 没有定义、参数已被清空，拒绝往下跑（不拦的话 --check 与 --list 会变成全跑）"
  echo "     → 怎么办：这棵树缺 .claude/scripts/preflight.sh：建副本时连 .claude/scripts 与 .claude/singlefs-ai-sop 一起拷（scripts/hosts/minimal-copy.py 逐条加 --with .claude/scripts=… 与 --with .claude/singlefs-ai-sop=…），或回主工作区跑"
  exit 2
fi
STAGE_REPOSITORY="$(cd "$(dirname "$0")/../.." && pwd)"
VERUS_CHECK="$STAGE_REPOSITORY/scripts/gate/verus-proof-check.py"
PIN_CHECK="$STAGE_REPOSITORY/scripts/experiment-apparatus/e165-pin-red-examples.py"
source "$(dirname "${BASH_SOURCE[0]}")/lib/stage-cells.sh"
stage_cell verus-toolchain-pinned cell_verus_toolchain_pinned "Verus 安装的 version 与 commit、Cargo.lock 里六个包的版本、本机 cargo 缓存里包源码的出处与内容指纹、Verus 与产品的 rustc 版本都与登记相同" \
  "装登记的 Verus 发布或用 SINGLEFS_VERUS_ROOT 指到它；缓存不对就删掉那个包目录再 cargo fetch --locked；要换版本先照 .claude/gate.d/verus-pins.tsv 的注释复核同源再改登记"
stage_cell verus-proof cell_verus_proof "Verus 逐项验登记的各 crate 被验区域里有证明义务的单元（按全路径：proof fn、exec fn、带 decreases 的 spec fn、derive(Clone) 的 clone，带值的 const 与 assume_specification 各成一项，impl 与 trait 里的方法与关联 const 也算），每项出结局、errors 0、verified 至少 1（指向函数的 assume_specification 验出 0 条也算过），各 crate 各项 verified 之和等于它登记的数，整格再完整验证 crate、产品构建与 verus --compile 构建的 MIR 逐条目相同；起 Verus 之前判被验区域：可信声明与 cfg 原子照信任清单逐条登记，axiom、作弊标记与 verus! 外面被验类型的 trait impl 判红" \
  "照 Verus 报错的位置修证明或实现；verified 数变了而确是加减了证明，就改 .claude/gate.d/verus-pins.tsv 的 crate 行并写明加减了哪几个；✗ 行写「被信号停了」的是外面停了它：查是谁在那一刻停了它（同一时刻的 ps、journalctl --user、看门狗的记录），这一次的输出不算判定，不改证明或实现（整条命令撞了内存上限由 task-server 放大重排，不在这一道判）"
stage_cell verus-vacuity cell_verus_vacuity "核里每一处 requires 的函数体头插 assert(false) 都验不过（未检出入口空真；可满足性另由构造见证证明）" \
  "验得过的那一处前置条件不可满足：改前置条件让它可满足；插入点认错了就按 scripts/gate/verus-proof-check.py 的 requires_insertion_points 改写法；✗ 行写「被信号停了」的是外面停了它：查是谁在那一刻停了它，这一次的输出不算判定，不改前置条件"
stage_cell verus-must-red-worlds cell_verus_must_red_worlds "先查函数与不变式负例覆盖；必红世界表每行改坏核之后 Verus 都报错、证明错落在第六段 landing= 登记的函数里（条数只报不判），原文恰好命中一次" \
  "锚点腐化就照今天的核改那一行的原文；改坏了还验得过就补证明或收紧 spec；落点换了先确认红的还是这条变异要盯的那一处（落到别的函数多半是证明或 spec 改过，改证明或换改法），确是本意再照红句里的实测改第六段 landing=<函数名>[,<函数名>…]；条数跳了不用管，第五段 proof_errors=<证明错条数> 只报不判；表不是六段制表符分隔的整张判红；表在 .claude/gate.d/verus-pins.tsv 的 must-red-worlds 行点名的路径；✗ 行写「被信号停了」的是外面停了它：查是谁在那一刻停了它，这一次的输出不算判定，不改表或证明"
stage_cell pinned-red-examples-table cell_pinned_red_examples_table "钉表 e165-pinned-red-examples.tsv 自己：每行 22 段整行抄带档、第 1 段与第 2、3、7 段重算的指纹相同、抄来的账行与自己的段对得上、第 22 段是规范写法的档，点名的变异还在它的表里；不读红例账、kill-bounds 产物与 research/ 下任何文件" \
  "表只由 scripts/experiment-apparatus/e165-pin-red-examples.py 写：段数不是 22、指纹、抄来的段或档的写法对不上是手改坏了，从 git 找回那一行；变异行不在了就找回那一行，确是改名就照新名改这张表（要写明为什么）；新出的红例由 E165 第二段驱动写完账当场钉，这一格不钉新红例"
stage_cell pinned-red-examples-replay cell_pinned_red_examples_replay "钉住的每一行是 22 段整行抄带档、抄来的账行与自己的段对得上，点名的变异还在它的表里，装置按档在今天的实现上逐行打的结果与原实现期望的逐态行逐字节相同" \
  "段数不是 22、抄来的账行与自己的段对不上是手改坏了，从 git 找回再钉；变异行不在了就找回那一行；装置逐行对不上是今天的实现在钉住的输入上与原实现不同，按回归修实现；装置报指纹漂了先查它的枚举次序与状态编码；✗ 行写「被信号停了」的是外面停了它：查是谁在那一刻停了它（同一时刻的 ps、journalctl --user、看门狗的记录），这一次的输出不算判定，不改变异行或实现（整条命令撞了内存上限由 task-server 放大重排，不在这一道判）"
stage_cell same-origin-counter cell_same_origin_counter "同源性计数器：不一致事件表 divergence-events.tsv 里每个已立的对累计 ≥ 5 次不一致而判「spec 错」0 次即判红；未立的对报未立、不计入，条款欠一句不计入累计" \
  "红的那一对累计 ≥ 5 次不一致而 spec 错 0 次，说明两条腿实际同源：D13 已定项 13 定的处置是把它们合并、预算移给穷举，这是设计决定，交主 agent 问用户，不靠改表销账；表里确有一次判错了（该是 spec 错）就改那一行的判谁错、出处写重判的记录；表写坏了（日期、对名、判谁错、出处，或未立的对收了事件）照红行点名的那一行改，格式在 .claude/gate.d/divergence-events.tsv 文件头"
stage_cells_parse "$@"; set -- ${STAGE_CELLS_REST[@]+"${STAGE_CELLS_REST[@]}"}
root_argument=""
tool_versions_requested=0
list_items_requested=0
with_keys_requested=0
requested_items=()
while (($#)); do
  case "$1" in
    --tool-versions) tool_versions_requested=1; shift ;;
    --list-items) list_items_requested=1; shift ;;
    --with-keys) with_keys_requested=1; shift ;;
    --item|--item=*)
      if [[ "$1" == --item ]]; then
        if (($# < 2)) || [[ -z "$2" ]]; then
          echo "  ✗ --item 后面缺项名"
          echo "     → 怎么办：写成 --item <项>（可给多次）；可点名的项用 bash .claude/gate.d/$(basename "$0") --list-items 列。"
          exit 2
        fi
        requested_items+=("$2"); shift 2
      else
        requested_items+=("${1#--item=}"); shift
      fi ;;
    -*)
      echo "  ✗ 认不出的选项 $1"
      echo "     → 怎么办：只认 --list、--check <格名>[,<格名>…]、--list-items [--with-keys]、--item <项>、--tool-versions 与一个项目根；缩小范围用 --item <项>（--list-items 列可点的项）。"
      exit 2 ;;
    *) root_argument="$1"; shift ;;
  esac
done
ROOT="${root_argument:-$STAGE_REPOSITORY}"
cd "$ROOT" 2>/dev/null || { echo "  ✗ 进不了项目根 $ROOT"; echo "     → 怎么办：第一个参数给被判的仓根（不给就判这道阶段所在的仓）。"; exit 2; }
ROOT="$(pwd)"
if ((tool_versions_requested)); then
  exec python3 "$VERUS_CHECK" tool-versions "$ROOT"
fi

sample_mode=0
if [[ "$(pwd -P)" != "$(cd "$STAGE_REPOSITORY" && pwd -P)" && -z "${GATE_STAGED_FROM:-}" ]]; then sample_mode=1; fi
# 只供样本的点名开关：.formal-sample/list-items 在就当成给了 --list-items，.formal-sample/items 的每一行当成一个 --item（命令行给了的不看它）
if ((sample_mode)) && [[ -f .formal-sample/list-items ]]; then list_items_requested=1; fi
if ((sample_mode)) && [[ -f .formal-sample/with-keys ]]; then with_keys_requested=1; fi
if ((with_keys_requested && !list_items_requested)); then
  echo "  ✗ --with-keys 只和 --list-items 一起用"
  echo "     → 怎么办：写成 --list-items --with-keys；要真跑就去掉 --with-keys（真跑每项判完另打 ITEM_DONE 行）。"
  exit 2
fi
if ((sample_mode)) && ((${#requested_items[@]} == 0)) && [[ -f .formal-sample/items ]]; then
  while IFS= read -r sample_item || [[ -n "$sample_item" ]]; do
    [[ -n "$sample_item" && "$sample_item" != \#* ]] && requested_items+=("$sample_item")
  done < .formal-sample/items
fi

# 判定库：只有 prune-server 那一份（库名在两份判法里固定，这一道不给位置）；键相同就复用。
#   样本根放了 .formal-sample/run-twice 的那份整道接连跑两趟（第二趟应当全复用；判定在临时 prune-server 里这一份样本的命名空间）。
verdict_rounds=1
if ((sample_mode)) && [[ -f .formal-sample/run-twice ]] && ((list_items_requested == 0)); then verdict_rounds=2; fi

# 交给判法的选项：样本根的点名开关（文件头「只供样本的开关」）与样本的假工具。样本不起真 Verus 与真 cargo（文件头「Verus 那几份样本起的是假工具」）：
#   样本根没有自带 Verus（verus-stand-in/、.formal-sample/verus-install/）而有登记表时，往样本根的 .formal-fake-tools/ 写假的 Verus 安装
#   （verus-proof-check.py write-fake-tools），--verus-root 指到它、PATH 前面放它（产品构建的 cargo rustc 也走假的）；写不成整道退 1
toolchain_options=()
verus_options=()
replay_options=()
if ((sample_mode)); then
  [[ -d .formal-sample/verus-install ]] && toolchain_options+=(--verus-root "$ROOT/.formal-sample/verus-install")
  [[ -d .formal-sample/cargo-home ]] && toolchain_options+=(--cargo-home "$ROOT/.formal-sample/cargo-home")
  [[ -f .formal-sample/rustc-version ]] && toolchain_options+=(--rustc-version-file "$ROOT/.formal-sample/rustc-version")
  [[ -f .formal-sample/e165-stub.sh ]] && replay_options+=(--binary "$ROOT/.formal-sample/e165-stub.sh")
  if [[ ! -d .formal-sample/verus-install && ! -d verus-stand-in && -f .claude/gate.d/verus-pins.tsv ]]; then
    fake_tools_directory="$ROOT/.formal-fake-tools"
    if ! python3 "$VERUS_CHECK" write-fake-tools "$fake_tools_directory" "$ROOT" >/dev/null; then
      echo "  ✗ 写不出样本的假 Verus 安装 $fake_tools_directory（verus-proof-check.py write-fake-tools 没成）"
      echo "     → 怎么办：样本根的 .claude/gate.d/verus-pins.tsv 要有 release 行；样本根要可写。"
      exit 1
    fi
    verus_options+=(--verus-root "$fake_tools_directory")
    export PATH="$fake_tools_directory:$PATH"
  fi
fi

# 可点名的项（--list-items 打它、--item 照它核）：两份判法各列自己那几格的，「项<TAB>格[<TAB>说明]」；只读登记表、核、必红世界表与钉表，不编、不起 Verus 与装置。
#   Verus 那份往 stderr 报的每格「键对不上 N 条…」与末了「键对不上共 N 条」（照 prune-server 那一份库比）收进 formal_items_report_file，
#   --list-items 那一路列完项之后原样接在后面（stderr，项的那几行 stdout 照旧只有「项<TAB>格[<TAB>说明]」）
formal_items_file=""
formal_items_report_file=""
# 列项退非 0 时，判法的红句（以 ✗、→ 起头的行）打在 stdout、进了 formal_items_file：原样回显到 stdout（stderr 那一份另由调用处 cat 回显），
#   不然红在哪被吞掉、只剩「列不出 … 退 1」。弄坏开关 FORMAL_VERIFICATION_BREAK=list-items-red-swallowed 只在样本根上认：不回显
echo_listing_red_lines() {
  if ((sample_mode)) && [[ ",${FORMAL_VERIFICATION_BREAK:-}," == *,list-items-red-swallowed,* ]]; then return 0; fi
  grep -E '^[[:space:]]*(✗|→)' "$formal_items_file"
  return 0
}
list_formal_items() {
  formal_items_file="$(mktemp)" || { echo "  ✗ 建不了临时文件"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑。"; exit 1; }
  formal_items_report_file="$(mktemp)" || { echo "  ✗ 建不了临时文件"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑。"; rm -f "${formal_items_file:?}"; exit 1; }
  local listing_exit
  # 比键：with_keys_requested 时才比。带 --item 的（requested_items）只比点名的项：verus_key_items / pin_key_items 是点名里属于各自一份判法的那几项，
  #   点名里没有它的项就不让它比（key_options 空），成本随点名项数走。弄坏开关 item-plan-ignores-item 下不分、整表都比
  local -a verus_key_options=() pin_key_options=() verus_key_items=() pin_key_items=()
  local named_item
  if ((with_keys_requested)); then
    if ((${#requested_items[@]})) && [[ ",${FORMAL_VERIFICATION_BREAK:-}," != *,item-plan-ignores-item,* ]]; then
      for named_item in "${requested_items[@]}"; do
        case "$named_item" in
          pinned:*) pin_key_items+=(--item "$named_item") ;;
          proof:*|vacuity:*|must-red:*) verus_key_items+=(--item "$named_item") ;;
        esac
      done
      ((${#verus_key_items[@]})) && verus_key_options=(--key-mismatches "${verus_key_items[@]}")
      ((${#pin_key_items[@]})) && pin_key_options=(--key-mismatches "${pin_key_items[@]}")
    else
      verus_key_options=(--key-mismatches); pin_key_options=(--key-mismatches)
    fi
  fi
  if python3 "$VERUS_CHECK" list-items "$ROOT" ${verus_key_options[@]+"${verus_key_options[@]}"} ${verus_options[@]+"${verus_options[@]}"} > "$formal_items_file" 2> "$formal_items_report_file"; then listing_exit=0; else listing_exit=$?; fi
  if ((listing_exit != 0)); then
    echo_listing_red_lines
    cat "$formal_items_report_file" >&2
    echo "  ✗ 列不出 Verus 那几格的项：scripts/gate/verus-proof-check.py list-items 退 $listing_exit"
    echo "     → 怎么办：照它上面打的红行（判法的 ✗ 与 →，原样回显在上面）修登记表 .claude/gate.d/verus-pins.tsv、必红世界表或红行点名的核里那一处写法；列项只读文件，不起 Verus。"
    rm -f "${formal_items_file:?}" "${formal_items_report_file:?}"; exit 1
  fi
  if python3 "$PIN_CHECK" --list-items --root "$ROOT" ${pin_key_options[@]+"${pin_key_options[@]}"} ${replay_options[@]+"${replay_options[@]}"} >> "$formal_items_file" 2>> "$formal_items_report_file"; then listing_exit=0; else listing_exit=$?; fi
  if ((listing_exit != 0)); then
    echo_listing_red_lines
    cat "$formal_items_report_file" >&2
    echo "  ✗ 列不出红例两格的项：scripts/experiment-apparatus/e165-pin-red-examples.py --list-items 退 $listing_exit"
    echo "     → 怎么办：照它上面打的红行修钉表 .claude/gate.d/e165-pinned-red-examples.tsv；列项只读文件，不编装置。"
    rm -f "${formal_items_file:?}" "${formal_items_report_file:?}"; exit 1
  fi
}
# --list-items --with-keys 的按项机读：项（formal_items_file 的每一行）逐个折成 ITEM_PLAN（文件头「--list-items --with-keys」）。
#   键状态取两份判法往 formal_items_report_file 打的 ITEM_KEY 行；toolchain、proof、table 三项每趟现判、不按条入库，一律 to_judge=1 reusable=0；
#   别的项键相同的 to_judge=0 reusable=1，其余（键对不上、无记录、拒算、没比出状态）to_judge=1 reusable=0。打完核一遍行数等于项数，不等整道退 1。
#   弄坏开关（FORMAL_VERIFICATION_BREAK=<项>，样本根上认）：item-plan-unkeyed 不看键、一律现判；item-plan-lines-skipped 不打 ITEM_PLAN；item-plan-count-short 少打最后一项
emit_item_plan() {
  local break_text=",${FORMAL_VERIFICATION_BREAK:-}," unkeyed=0 plan_lines item_count plan_count
  [[ "$break_text" == *,item-plan-lines-skipped,* ]] && return 0
  [[ "$break_text" == *,item-plan-unkeyed,* ]] && unkeyed=1
  plan_lines="$(awk -F'\t' -v keys="$formal_items_report_file" -v unkeyed="$unkeyed" '
    BEGIN {
      while ((getline line < keys) > 0) {
        if (line !~ /^ITEM_KEY item=/) continue
        sub(/^ITEM_KEY item=/, "", line)
        at = match(line, / state=[^ ]*$/)
        if (at > 0) state[substr(line, 1, at - 1)] = substr(line, at + 7)
      }
    }
    $1 != "" {
      fresh = ($1 == "toolchain" || $1 == "proof" || $1 == "table")
      reusable = (!fresh && !unkeyed && state[$1] == "键相同") ? 1 : 0
      printf "ITEM_PLAN item=%s to_judge=%d reusable=%d\n", $1, 1 - reusable, reusable
    }' "$formal_items_file")"
  if [[ "$break_text" == *,item-plan-count-short,* ]]; then plan_lines="$(sed '$d' <<<"$plan_lines")"; fi
  item_count="$(grep -c . "$formal_items_file")"
  plan_count="$(grep -c '^ITEM_PLAN ' <<<"$plan_lines")"
  printf '%s\n' "$plan_lines"
  if ((plan_count != item_count)); then
    echo "  ✗ ITEM_PLAN 打了 $plan_count 行，项有 $item_count 个：每项一行才对"
    echo "     → 怎么办：看 emit_item_plan（.claude/gate.d/$(basename "$0")）：项名里有没有换行或制表符以外的东西把一行切成了两行。"
    rm -f "${formal_items_file:?}" "${formal_items_report_file:?}"; exit 1
  fi
  if ((${#requested_items[@]})); then
    local named_count="${#requested_items[@]}"
    if ((plan_count != named_count)); then
      echo "  ✗ ITEM_PLAN 打了 $plan_count 行，点名了 $named_count 项：带 --item 只打点名的项"
      echo "     → 怎么办：看 list_formal_items 里 --item 怎么分给两份判法、emit_item_plan 怎么只留点名的项。"
      rm -f "${formal_items_file:?}" "${formal_items_report_file:?}"; exit 1
    fi
    echo "  · 按项机读：ITEM_PLAN $plan_count 行，与点名项数 $named_count 相同"
  else
    echo "  · 按项机读：ITEM_PLAN $plan_count 行，与项数 $item_count 相同"
  fi
}
if ((list_items_requested)); then
  if ((STAGE_CELLS_CHECK_GIVEN)) || { ((${#requested_items[@]})) && ((! with_keys_requested)); }; then
    echo "  ✗ --list-items 只列项，不与 --check 一起用；带 --item 只和 --with-keys 一起用（只比点名那几项的键）"
    echo "     → 怎么办：先单独跑 --list-items 看项名，再用 --item <项> 点名跑；要按点名项估用时写 --list-items --with-keys --item <项>。"
    exit 2
  fi
  # 带 --item：先不比键把项列一遍，核点名的名字（写错退 2 并列出可点名的项），再只比点名的那几项的键
  if ((${#requested_items[@]})); then
    saved_with_keys="$with_keys_requested"; with_keys_requested=0
    list_formal_items
    with_keys_requested="$saved_with_keys"
    unknown_listed=()
    for wanted_item in "${requested_items[@]}"; do
      cut -f1 "$formal_items_file" | grep -qxF -- "$wanted_item" || unknown_listed+=("$wanted_item")
    done
    if ((${#unknown_listed[@]})); then
      echo "  ✗ 没有这些项：${unknown_listed[*]}"
      echo "     → 怎么办：项名照下面这张表写（第一列是项、第二列是格），给几次 --item 取并集："
      cut -f1,2 "$formal_items_file" | sed 's/^/       /'   # gate-lint:detail
      rm -f "${formal_items_file:?}" "${formal_items_report_file:?}"
      exit 2
    fi
    rm -f "${formal_items_file:?}" "${formal_items_report_file:?}"
  fi
  # 样本 .formal-sample/plan-after-run：列项比键之前先把证明、空真、必红世界三格真跑一趟（假工具），判过的项才有键相同可比
  if ((sample_mode)) && [[ -f .formal-sample/plan-after-run ]]; then
    for pre_judged_check in proof vacuity must-red-worlds; do
      python3 "$VERUS_CHECK" "$pre_judged_check" "$ROOT" ${verus_options[@]+"${verus_options[@]}"} >/dev/null 2>&1 || true
    done
  fi
  list_formal_items
  # 带 --item：项的那几行也只留点名的（弄坏开关 item-plan-ignores-item 下留整表，ITEM_PLAN 的行数核对会红）
  if ((${#requested_items[@]})) && [[ ",${FORMAL_VERIFICATION_BREAK:-}," != *,item-plan-ignores-item,* ]]; then
    printf '%s\n' "${requested_items[@]}" | awk -F'\t' 'NR == FNR { wanted[$0] = 1; next } $1 in wanted' - "$formal_items_file" > "$formal_items_file.named" && mv "$formal_items_file.named" "$formal_items_file"
  fi
  cat "$formal_items_file"
  # 键对不上的条数接在项后面、走 stderr；Verus 那份没报（没比键，或核没进仓、没有要比的项）时这份是空的，另说一句没比
  if [[ -s "$formal_items_report_file" ]]; then
    grep -v '^ITEM_KEY ' "$formal_items_report_file" >&2 || true
  elif ((with_keys_requested)); then
    echo "  · verus-proof-check.py list-items 没报键数（核没进仓）：列项没比键" >&2
  else
    echo "  · 没比键（要按项看键相同还是对不上：--list-items --with-keys）" >&2
  fi
  ((with_keys_requested)) && emit_item_plan
  rm -f "${formal_items_file:?}" "${formal_items_report_file:?}"
  exit 0
fi
# --item：点名的项各属哪一格照列出的表找；格只跑点了项的那几格（次序照登记），vacuity、must-red、pinned 那几格只判点名的项（转给判法的 --item）
declare -A item_options_of_cell=()
if ((${#requested_items[@]})); then
  if ((STAGE_CELLS_CHECK_GIVEN)); then
    echo "  ✗ --item 与 --check 不一起用：--item 已经定了跑哪几格"
    echo "     → 怎么办：只写 --item <项>（可给多次）；要整格跑就只写 --check <格名>。"
    exit 2
  fi
  list_formal_items
  declare -A cell_of_item=()
  while IFS=$'\t' read -r listed_item listed_cell _listed_note; do
    [[ -n "$listed_item" ]] && cell_of_item[$listed_item]="$listed_cell"
  done < "$formal_items_file"
  unknown_items=()
  declare -A cells_wanted=()
  for wanted_item in "${requested_items[@]}"; do
    if [[ -z "${cell_of_item[$wanted_item]+set}" ]]; then
      [[ ",${FORMAL_VERIFICATION_BREAK:-}," == *,items-unchecked,* ]] || unknown_items+=("$wanted_item")
      continue
    fi
    cells_wanted[${cell_of_item[$wanted_item]}]=1
    case "${cell_of_item[$wanted_item]}" in
      verus-proof|verus-vacuity|verus-must-red-worlds|pinned-red-examples-replay)
        item_options_of_cell[${cell_of_item[$wanted_item]}]+="$wanted_item"$'\n' ;;
    esac
  done
  if ((${#unknown_items[@]})); then
    echo "  ✗ 没有这些项：${unknown_items[*]}"
    echo "     → 怎么办：项名照下面这张表写（第一列是项、第二列是格），给几次 --item 取并集："
    cut -f1,2 "$formal_items_file" | sed 's/^/       /'   # gate-lint:detail
    rm -f "${formal_items_file:?}" "${formal_items_report_file:?}"
    exit 2
  fi
  rm -f "${formal_items_file:?}" "${formal_items_report_file:?}"
  STAGE_CELLS_SELECTED=()
  for cell_name in "${STAGE_CELLS_NAMES[@]}"; do
    [[ -n "${cells_wanted[$cell_name]+set}" ]] && STAGE_CELLS_SELECTED+=("$cell_name")
  done
  STAGE_CELLS_CHECK_GIVEN=1
  STAGE_CELLS_SELECTION_SOURCE="--item 点名 ${#requested_items[@]} 项"
fi
# 把这一格点名的项交成判法的 --item 参数（一项一对），交到 item_arguments 数组里
item_arguments_of() {
  item_arguments=()
  local item_line
  while IFS= read -r item_line; do
    [[ -n "$item_line" ]] && item_arguments+=(--item "$item_line")
  done <<<"${item_options_of_cell[$1]:-}"
}

# 按项的原料（文件头「真跑每一项判完…ITEM_DONE」）：toolchain、table 两格不经判法脚本的逐项记录，由这一道自己按格的挂钟记一行「项<TAB>秒<TAB>现判<TAB>复用<TAB>退出码」
record_gate_item_done() {
  [[ -n "${FORMAL_ITEM_DONE_FILE:-}" && ",${FORMAL_VERIFICATION_BREAK:-}," != *,item-done-lines-skipped,* ]] || return 0
  awk -v item="$1" -v start="$2" -v stop="$EPOCHREALTIME" -v code="$3" 'BEGIN { printf "%s\t%.1f\t1\t0\t%d\n", item, stop - start, code }' >> "$FORMAL_ITEM_DONE_FILE"
}
cell_verus_toolchain_pinned() {
  local started="$EPOCHREALTIME" code
  if python3 "$VERUS_CHECK" toolchain "$ROOT" ${toolchain_options[@]+"${toolchain_options[@]}"}; then code=0; else code=$?; fi
  record_gate_item_done toolchain "$started" "$code"
  return "$code"
}
cell_verus_proof() { item_arguments_of verus-proof; python3 "$VERUS_CHECK" proof "$ROOT" ${verus_options[@]+"${verus_options[@]}"} ${item_arguments[@]+"${item_arguments[@]}"}; }
cell_verus_vacuity() { item_arguments_of verus-vacuity; python3 "$VERUS_CHECK" vacuity "$ROOT" ${verus_options[@]+"${verus_options[@]}"} ${item_arguments[@]+"${item_arguments[@]}"}; }
cell_verus_must_red_worlds() { python3 "$VERUS_CHECK" coverage "$ROOT" || return $?; item_arguments_of verus-must-red-worlds; python3 "$VERUS_CHECK" must-red-worlds "$ROOT" ${verus_options[@]+"${verus_options[@]}"} ${item_arguments[@]+"${item_arguments[@]}"}; }
cell_pinned_red_examples_table() {
  local started="$EPOCHREALTIME" code
  if python3 "$PIN_CHECK" --check-table --root "$ROOT"; then code=0; else code=$?; fi
  record_gate_item_done table "$started" "$code"
  return "$code"
}
cell_pinned_red_examples_replay() {
  item_arguments_of pinned-red-examples-replay
  # RocksDB 预编库的取值（scripts/hosts/rocksdb-prebuilt.py 文件头「entry」）：不带 --binary（样本的桩）时 e165-pin-red-examples.py --replay 起
  #   cargo build -p singlefs-verification（它依赖 prune-server，经它编到 librocksdb-sys）：起它之前判一次这台装了、此刻进程环境的取值对得上，判红这一格退 1、装置不编。
  #   树里没有依赖 rocksdb 的清单（别的样本）、PATH 上是带标记的假 cargo，entry 报不判。弄坏开关 FORMAL_VERIFICATION_BREAK=rocksdb-entry-unchecked 下不判
  #   （样本 pinned-red-examples-replay-rocksdb-entry-refused 判错）。这一份被拷到不带 scripts/hosts/ 的树里时报一行没判、往下走
  if [[ " ${replay_options[*]-} " != *" --binary "* && ",${FORMAL_VERIFICATION_BREAK:-}," != *,rocksdb-entry-unchecked,* ]] \
     && [[ ! -f "$STAGE_REPOSITORY/scripts/hosts/rocksdb-prebuilt.py" ]]; then
    echo "  ! RocksDB 预编库的取值没判：$STAGE_REPOSITORY/scripts/hosts/rocksdb-prebuilt.py 不在（这一份阶段被拷到了不带 scripts/hosts/ 的树里）"
  elif [[ " ${replay_options[*]-} " != *" --binary "* && ",${FORMAL_VERIFICATION_BREAK:-}," != *,rocksdb-entry-unchecked,* ]] \
     && ! python3 "$STAGE_REPOSITORY/scripts/hosts/rocksdb-prebuilt.py" entry --root "$ROOT"; then
    echo "  ✗ 起 cargo 编装置之前 RocksDB 预编库的取值没判过（上面 rocksdb-prebuilt.py entry 的红句），这一格没编装置、没起重放"
    echo "     → 怎么办：照上面的出路修（开新会话，或命令前 eval \"\$(python3 scripts/hosts/rocksdb-prebuilt.py env)\"），再只跑这一格：bash .claude/gate.d/formal-verification.sh --check pinned-red-examples-replay"
    return 1
  fi
  python3 "$PIN_CHECK" --replay --root "$ROOT" ${replay_options[@]+"${replay_options[@]}"} ${item_arguments[@]+"${item_arguments[@]}"}
}

# ── 格 same-origin-counter：还 checks-owed.md C191（同源性计数器只盖了一对）。D13 已定项 13「冲突 2」的判据逐字：同源性要被观测，统计 spec 与 model 不一致的事件数
# 及每次判定哪一方错，累计 ≥ 5 次不一致而判「spec 错」的次数为 0，说明两者实际同源，处置是把两条腿合并、预算移给穷举。
# 读被判的根里 .claude/gate.d/divergence-events.tsv（格式在那张表自己的文件头）：「对」行登记哪一对、哪边算 spec、已立还是未立，「事件」行是一次不一致（日期、对名、判谁错、出处、说明）。
#   已立的对：累计 = spec 错 + model 错的行数；累计 ≥ 5 而 spec 错 0 次判红；条款欠一句的行先记账，不计入累计、不计入 spec 错（D13 第三格）；
#             每一对都把累计次数报出来，0 次只说「还没有事件进表」，不当作验过独立。
#   未立的对：报「未立」，不报绿、不计入任何一格；表里有点名它的事件行判红（未立的对不收事件）。
#   表写坏了判红：不是「对」「事件」行、段数不对、对名重复或不合写法、状态不是已立 / 未立、日期不是 YYYY-MM-DD、事件点名没登记的对、判谁错不是 spec错 / model错 / 条款欠一句、出处不是仓内路径起头。
#   一对已立的都没有：退 77（本次未跑），不当通过。
# 样本：same-origin-counter-red（一对已立的对 5 次不一致全判 model 错，判红）、same-origin-counter-green（一对 5 次里有 1 次 spec 错、一对 2 次 model 错加 2 次条款欠一句，
#   一对未立；判绿，报「已立 2 对、未立 1 对」）、same-origin-counter-clause-gap-green（4 次 model 错加 3 次条款欠一句：条款欠一句不计入累计，判绿）、
#   same-origin-counter-unbuilt-event-red（未立的对收了事件，判红）、same-origin-counter-verdict-red（判谁错写成表外的词，判红）。
# 弄坏开关 FORMAL_VERIFICATION_BREAK=same-origin-zero-spec-wrong-unchecked / same-origin-clause-gap-counted / same-origin-unbuilt-reported-green /
#   same-origin-unbuilt-events-accepted，各让哪份样本判错写在本文件头「弄坏开关」一段。
cell_same_origin_counter() {
  python3 - "$ROOT" "${FORMAL_VERIFICATION_BREAK:-}" <<'PY'
import datetime, os, re, sys
root, broken = sys.argv[1], {item for item in sys.argv[2].split(",") if item}
TABLE = ".claude/gate.d/divergence-events.tsv"
THRESHOLD = 5
VERDICTS = ("spec错", "model错", "条款欠一句")
STATES = ("已立", "未立")
path = os.path.join(root, TABLE)
if not os.path.isfile(path):
    print(f"  ✗ 同源性计数器的不一致事件表 {TABLE} 不在")
    print("     → 怎么办：它随仓走（checks-owed.md C191）；被删了从 git 找回，格式写在表自己的文件头")
    sys.exit(1)
pairs, events, problems = {}, [], []
for number, raw in enumerate(open(path, encoding="utf-8").read().split("\n"), 1):
    if not raw.strip() or raw.startswith("#"):
        continue
    fields = raw.split("\t")
    where = f"{TABLE} 第 {number} 行"
    if fields[0] == "对":
        if len(fields) != 6 or not all(fields[1:]):
            problems.append(f"{where}：「对」行要六段（对、对名、spec 一方、model 一方、已立|未立、说明），都不许空，读到 {len(fields)} 段")
        elif not re.fullmatch(r"[a-z0-9][a-z0-9-]*", fields[1]):
            problems.append(f"{where}：对名「{fields[1]}」只许小写字母、数字、连字符")
        elif fields[1] in pairs:
            problems.append(f"{where}：对名「{fields[1]}」登记了两次（第 {pairs[fields[1]]['number']} 行）")
        elif fields[4] not in STATES:
            problems.append(f"{where}：状态「{fields[4]}」要写已立或未立")
        else:
            pairs[fields[1]] = {"number": number, "spec": fields[2], "model": fields[3], "state": fields[4], "note": fields[5]}
    elif fields[0] == "事件":
        if len(fields) != 6 or not all(fields[1:]):
            problems.append(f"{where}：「事件」行要六段（事件、日期、对名、判谁错、出处、说明），都不许空，读到 {len(fields)} 段")
            continue
        try:
            if not re.fullmatch(r"\d{4}-\d{2}-\d{2}", fields[1]):
                raise ValueError
            datetime.date.fromisoformat(fields[1])
        except ValueError:
            problems.append(f"{where}：日期「{fields[1]}」不是 YYYY-MM-DD")
            continue
        if fields[3] not in VERDICTS:
            problems.append(f"{where}：判谁错「{fields[3]}」要写 {' | '.join(VERDICTS)} 之一")
            continue
        if not re.match(r"[A-Za-z0-9_.-]+/\S+", fields[4]):
            problems.append(f"{where}：出处「{fields[4]}」要以仓内相对路径起头（records/…、research/…、.claude/kb/… 这类，可后接行号）")
            continue
        events.append({"number": number, "date": fields[1], "pair": fields[2], "verdict": fields[3]})
    else:
        problems.append(f"{where}：第一段「{fields[0]}」不是「对」或「事件」")
if not pairs and not problems:
    problems.append(f"{TABLE} 里一个「对」都没有：没有要观测的对")
for event in events:
    if event["pair"] not in pairs:
        problems.append(f"{TABLE} 第 {event['number']} 行：事件点名的对「{event['pair']}」没有「对」行登记")
built, unbuilt, lines = [], [], []
for name, pair in pairs.items():
    mine = [event for event in events if event["pair"] == name]
    if pair["state"] == "未立" and "same-origin-unbuilt-reported-green" not in broken:
        unbuilt.append(name)
        lines.append(f"  · {name}：未立（{pair['note']}）——不报绿、不计入任何一格")
        if mine and "same-origin-unbuilt-events-accepted" not in broken:
            problems.append(f"{TABLE} 第 {mine[0]['number']} 行：对「{name}」还是未立，不收事件（共 {len(mine)} 行点名了它）")
        continue
    built.append(name)
    spec_wrong = sum(1 for event in mine if event["verdict"] == "spec错")
    model_wrong = sum(1 for event in mine if event["verdict"] == "model错")
    clause_gap = sum(1 for event in mine if event["verdict"] == "条款欠一句")
    counted = spec_wrong + model_wrong
    judged_count = counted + (clause_gap if "same-origin-clause-gap-counted" in broken else 0)
    state = ("还没有事件进表，累计 0 次只说明没观测到，不说明两边独立" if not mine
             else f"未满 {THRESHOLD} 次，不判同源" if judged_count < THRESHOLD
             else f"已满 {THRESHOLD} 次而 spec 错 0 次，实际同源" if spec_wrong == 0
             else f"已满 {THRESHOLD} 次，spec 错 {spec_wrong} 次，不是同源的样子")
    lines.append(f"  · {name}（{pair['spec']} × {pair['model']}）：已立，累计不一致 {counted} 次（spec 错 {spec_wrong}、model 错 {model_wrong}），"
                 f"条款欠一句 {clause_gap} 次先记账、不计入；{state}")
    if judged_count >= THRESHOLD and spec_wrong == 0 and "same-origin-zero-spec-wrong-unchecked" not in broken:
        problems.append(f"对「{name}」（{pair['spec']} × {pair['model']}）：累计 {judged_count} 次不一致而判「spec 错」0 次（model 错 {model_wrong} 次）：两条腿实际同源")
for line in lines:
    print(line)  # gate-lint:detail
if problems:
    for problem in problems:
        print(f"  ✗ {problem}")  # gate-lint:detail
    print(f"  ✗ 同源性计数器：{len(problems)} 处不对（已立 {len(built)} 对、未立 {len(unbuilt)} 对、事件 {len(events)} 行）")  # gate-lint:summary
    print("     → 怎么办：累计 ≥ 5 次而 spec 错 0 次的，是两条腿实际同源：D13 已定项 13 定的处置是把它们合并、预算移给穷举，这是设计决定，交主 agent 问用户，不靠改表销账；"
          "表里确有一次判错了（该是 spec 错）就改那一行的判谁错、出处写重判的记录；表写坏了照上面点名的行改，格式在 .claude/gate.d/divergence-events.tsv 文件头")
    sys.exit(1)
if not built:
    print(f"  ! {TABLE} 里一对已立的都没有（未立 {len(unbuilt)} 对），这一格无对象可判")
    sys.exit(77)
print(f"  ✓ 同源性计数器：已立 {len(built)} 对、未立 {len(unbuilt)} 对（未立的不报绿、不计入）；事件 {len(events)} 行，已立的每一对都没有累计 ≥ {THRESHOLD} 次而 spec 错 0 次")
PY
}

# 格不复用（文件头「复用」一段）：每一格照跑，格里每一条按自己的单元键查判定库。

# 本次未跑的格先收进自己的文件，再原样转给 $GATE_NOT_RUN_FILE
not_run_record="$(mktemp)" || { echo "  ✗ 建不了临时文件"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑。"; exit 1; }
# 按条的现判、复用数：判法每格往这份文件追加一行「格<TAB>现判数<TAB>复用数<TAB>原因分栏」（FORMAL_ITEM_TALLY_FILE），跑完整道相加
item_tally_record="$(mktemp)" || { echo "  ✗ 建不了临时文件"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑。"; rm -f "${not_run_record:?}"; exit 1; }
# 按项的挂钟、条数、退出码：判法每项判完往这份文件追加一行「项<TAB>秒<TAB>现判<TAB>复用<TAB>退出码」（FORMAL_ITEM_DONE_FILE），跑完整道转成 ITEM_DONE 行（文件头「真跑每一项判完…」）
item_done_record="$(mktemp)" || { echo "  ✗ 建不了临时文件"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑。"; rm -f "${not_run_record:?}" "${item_tally_record:?}"; exit 1; }
stage_started="$EPOCHREALTIME"
for ((verdict_round = 1; verdict_round <= verdict_rounds; verdict_round++)); do
  ((verdict_rounds > 1)) && echo "── 第 $verdict_round 趟（样本 .formal-sample/run-twice：判定在临时 prune-server 里，第二趟应当全复用）"
  : > "$item_tally_record"
  : > "$item_done_record"
  if ( export GATE_NOT_RUN_FILE="$not_run_record" FORMAL_ITEM_TALLY_FILE="$item_tally_record" FORMAL_ITEM_DONE_FILE="$item_done_record"; stage_cells_run "$ROOT" ); then stage_exit=0; else stage_exit=$?; fi
done
if [[ -s "$item_tally_record" ]]; then
  judged_total=0; reused_total=0; tally_cells=0
  while IFS=$'\t' read -r tally_cell tally_judged tally_reused tally_reasons; do
    tally_cells=$((tally_cells + 1))
    judged_total=$((judged_total + tally_judged)); reused_total=$((reused_total + tally_reused))
    echo "  $tally_cell：现判 $tally_judged、复用 $tally_reused${tally_reasons:+（现判的原因：$tally_reasons）}"
  done < "$item_tally_record"
  echo "── 按条（$tally_cells 格有逐项判定；verus-toolchain-pinned 与 pinned-red-examples-table 每趟现判、不复用）：现判 $judged_total、复用 $reused_total；判定在 prune-server 那一份（verdict-library 族 singlefs-formal-unit-verdicts）"
fi
rm -f "${item_tally_record:?}"
# ITEM_DONE：同一项在几个 crate 里各判一次的合成一行（秒数、条数相加，退出码取大），次序照第一次出现；
#   弄坏开关 FORMAL_VERIFICATION_BREAK=item-done-lines-skipped 不打；item-done-whole-run-seconds 每项的秒数都写整道从起到收尾的挂钟（不是各项自己的）
if [[ -s "$item_done_record" && ",${FORMAL_VERIFICATION_BREAK:-}," != *,item-done-lines-skipped,* ]]; then
  whole_run_seconds=""
  [[ ",${FORMAL_VERIFICATION_BREAK:-}," == *,item-done-whole-run-seconds,* ]] && whole_run_seconds="$(awk -v start="$stage_started" -v stop="$EPOCHREALTIME" 'BEGIN { printf "%.1f", stop - start }')"
  awk -F'\t' -v whole="$whole_run_seconds" '
    { if (!($1 in seen)) { seen[$1] = 1; order[++count] = $1 } seconds[$1] += $2; judged[$1] += $3; reused[$1] += $4; if ($5 + 0 > code[$1] + 0) code[$1] = $5 + 0 }
    END { for (index_ = 1; index_ <= count; index_++) { item = order[index_]
            printf "ITEM_DONE item=%s seconds=%.1f judged=%d reused=%d exit=%d\n", item, (whole != "" ? whole : seconds[item]), judged[item], reused[item], code[item] } }' "$item_done_record"
fi
rm -f "${item_done_record:?}"
if [[ -n "${GATE_NOT_RUN_FILE:-}" && -s "$not_run_record" ]]; then cat "$not_run_record" >> "$GATE_NOT_RUN_FILE"; fi
rm -f "${not_run_record:?}"
exit "$stage_exit"
