#!/usr/bin/env python3
# admission: always 每一次调都判此刻被判的仓、此刻的 Verus 安装与 cargo 缓存，上一次的结论不替这一次作保
# run-condition: none 要的 Verus、cargo 与内存包装由各子命令自己判，缺了判红并给出路（缺工具直接红，不交给预判退 78）
"""门禁 checker-tier-formal-verification 的 Verus 那四格的判法（里程碑三七·5；阶段只负责把它接进门禁、给样本开关）。

被验的：登记表 .claude/gate.d/verus-pins.tsv 点名的 crate（今天是只依赖 vstd 的 crates/singlefs-replay-prefix）与它里面的核（kernel 行，今天就是它的 src/lib.rs）。
Verus 每次都直接起（不经 cargo verus：它按内容缓存，内容不变时不出判定行），对 crate 的入口文件 --crate-type=lib，带 --no-cheating、--output-json 与 --rlimit <登记的数>
（登记表 rlimit 行；没有这一行时取 DEFAULT_RLIMIT，就是 Verus 0.2026.09.20 自己的默认 10。proof、vacuity、must-red-worlds 三格同一个数），
判定取 stdout 的 JSON（verification-results 的 verified / errors / encountered-vir-error，verus 的 commit），报错位置取 stderr。
直接起的 Verus 只带它自带的 vstd：crate 的 Cargo.toml 除 vstd 之外还有依赖，proof、vacuity、must-red-worlds 三格开跑之前判红。

用法（<根> 是被判的仓根；四个判的子命令都退 0 绿、1 红、2 用法错、77 本次未跑）：
  verus-proof-check.py toolchain <根> [--verus-root 目录] [--cargo-home 目录]
      版本一致：Verus 安装里 version.json 的 version 与 commit、安装目录旁 verus.zip 的 sha256（zip 在就核，不在报一句）、
      Cargo.lock 里登记的六个包各恰好一个版本且等于登记、本机 cargo 缓存里每个包的 .cargo_vcs_info.json（sha1、path_in_vcs）
      与包源码的内容指纹（fingerprint 子命令的算法），都与登记相同。
  verus-proof-check.py proof <根> [通用选项]
      核里（去掉 // 注释之后）不许出现 external_body、assume…（含 assume_specification）、admit、verifier::external / verifier(external)、
      verifier::rlimit / verifier(rlimit …)（函数上的 rlimit 属性盖过命令行的 --rlimit，那个函数就不在登记的额度上验）；
      Verus 逐项验：核里 verus! { … } 里有证明义务的单元各一项（proof_items）：proof fn、exec fn、带 decreases 的 spec fn（终止义务）、
      derive(Clone) 的类型（Verus 验它生成的 <类型>::clone）；不带 decreases 的 spec fn 没有证明义务、不成项，随用到它的项进键。
      每项起一次 --verify-root --verify-function <名>（自由 fn 写它的名字，clone 写 <类型>::clone）：限时内出结局、没有 vir 错（作弊标记在这一层也被
      --no-cheating 拒：只验一个函数时别处的 assume 照样拒，主 agent 2026-10-15 在副本上量的）、encountered-error 不是 true、errors 0、verified 至少 1、
      跑的 Verus 的 commit 等于登记。--verify-function 下真 Verus 不给 success 键、is-verifying-entire-crate 报 false，这两样不看（verification_clean）。
      整格跑时各项 verified 之和（复用的取记下的数；按点的名字计，同名的只算一次）等于登记的数（整 crate 验的数），不等判红并点名
      verified 与静态估的义务条数不同的项（proof fn 1、exec fn 1 加每个 while / loop / for 1、带 decreases 的 spec fn 1、clone 1）与没成项的 fn。超时判红（C5：超时不当通过）。
      verus! 里两个 fn 同名、或 fn 在 impl / trait 里判红：--verify-function 按名点（有恰好同名的就取它，名字是别人子串的照常点得中），这两种点不准。
  verus-proof-check.py vacuity <根> [通用选项]
      空真：核里每一处 requires（整行以 requires 起头，fn 与闭包都算）找到它的函数体的 {（其后第一行以 { 起头、缩进不深于 requires 那一行的），
      在临时副本里 { 后面插 assert(false);，一处一份副本、一次 Verus（只验这一处所属的那个 fn：--verify-root --verify-function）；
      每一处都要报 assertion failed 且位置就是插的那一行。
      验得过 ⇒ 这一处的前置条件不可满足，判红。一次只插一处：插在外层函数入口的 assert(false) 失败之后，Verus 把其后全部当 false 已知，
      里面闭包的那一处就验得过了（2026-10-01 在草稿里实测：七处一起插只报六处）。
  verus-proof-check.py must-red-worlds <根> [通用选项]
      必红世界：登记表 must-red-worlds 行点名的表（五段制表符分隔：变异名、文件、原文、替换文、proof_errors=M；\\n 表换行），
      每行原文在文件里恰好命中一次，改坏的副本上 Verus 要出 JSON、errors ≥ 1，证明错的条数等于 M。一行都没有判红。
      证明错：输出里 error 起头的报错，除去含 Resource limit (rlimit) exceeded 的与以 aborting due to 起头的（proof_error_messages）；
      M 只数它们，不数 rlimit 用尽（Verus 把 rlimit 用尽也计进 JSON 的 errors，同一行换个 rlimit 那个数就变）。
      第五段一律写 proof_errors=<正整数>，不是这个形态的行（旧写法的输出片段、「verified, M errors」）不起 Verus、判红，出路写这条约定；
      JSON 里没有整数的 verified 与 errors 的那一行判红。
      证明错一条都没有、只剩 rlimit 用尽的那一行不作数、判红：红在算力额度上，不是证明失败（改替换文，或调高登记的 rlimit）。
      替换文里（去掉 // 注释之后）有 verifier::rlimit / verifier(rlimit …) 的那一行判红：它盖过命令行的 --rlimit。
  通用选项：--verus-root 目录（默认 $SINGLEFS_VERUS_ROOT，再默认 ${XDG_DATA_HOME:-~/.local/share}/<登记表 release 行第五列>）
            --timeout 秒（默认 $GATE_VERUS_TIMEOUT_SECONDS，再默认 600）  --threads N（默认 $SINGLEFS_THREAD_CAP，再默认登记的重活默认并行度 .claude/heavy-thread-default）
            --jobs J（空真与必红世界几处同时验，默认 $GATE_VERUS_JOBS，再默认 min(处数, max(1, N // 4))；每一处的 Verus 取 N // J 条线程，至少 1；
              N = 31 时同时至多验 7 处、每处 4 条线程起（7 处 4 条、1 处 31 条），合计不超过 N）
            --memory-cap 上限（每次 Verus 经 scripts/hosts/run-with-memory-cap.sh，默认 $GATE_VERUS_MEMORY_MAX，再默认 4G；
              限时交给它的 RUN_WITH_MEMORY_CAP_TIME_LIMIT，退 253 就是超时；退 245、250、251、252、254 是包装自己的结局，那一处判「没跑成」：
              这一格退 1，✗ 行写「没跑成：内存包装 scripts/hosts/run-with-memory-cap.sh 退 <码>」，行尾原样带上包装自己往 stderr 打的 ✗ 与 → 那几行，
              → 照包装文件头「退出码」一段、按码给，不指向证明、核或必红世界表；252 分两种成因各给一种：包装说「等多久都放不下」的（这一处要的量比总上限还大）
              给「上限写小」，别的（排队等满）给「看 --status、空出来再重跑」）
              Verus 被信号停了的（直接起时 subprocess 交负的信号号，经包装起时退 128 + 信号号、不在 250–254 里，例 137 = 外面 kill -9）：那一处判红，
              ✗ 行写「Verus 被信号停了」、这一次的输出不算判定，→ 照包装文件头「退出码」一段（撞了这一条自己的上限由包装认成 250，外面停的退 137），不写修证明
              一格里既有「没跑成」又有真红的（几处里有的没跑成、有的真没红，或核里有作弊标记而 Verus 没跑成），每一条照打，末尾另打一行
              「✗ 这一格 N 条真红、M 条没跑成：按红记」，交出的判定是红，不是没跑成
            排队时内存包装每 60 秒往 stderr 报一行（「排队（已等 …」，放行时「排了 … 秒，起跑」）：判法收包装的 stderr 判结局，
              那几行边跑边另转打到本进程的 stderr、行首带是哪一格（run_forwarding_queue_lines），判定用的输出不变
            一道里只排一次（QueueRefusal）：一格里任何一处第一次拿到包装的 252（排不上，两种成因都算），这一格还没起的那几处不建副本、不起包装、不排队，
              已经起了的照常收；没起的那几处并成一条 ✗「没跑成：内存包装排不上，没再排队（同一道里 <格>「<哪一处>」那一处退了 252…）：这一格 N 处 Verus 没起——<逐个列名>」，
              → 用先排不上那一处的出路（照包装文件头「退出码」一段）。环境里有 VERUS_PROOF_CHECK_QUEUE_REFUSED_FILE（门禁 checker-tier-formal-verification 整道跑时给一份空文件）的，
              第一次拿到 252 时往那份文件写一行「哪一处<TAB>出路」，后面几格开跑前读到它就一处都不起包装、照上面那样报；所以整道最坏只等一次等待上限
  verus-proof-check.py list-items <根> [--verdict-library 目录] [--verus-root 目录]
      逐行打可点名的项「项<TAB>格[<TAB>说明]」：toolchain（verus-toolchain-pinned）一项，proof（verus-proof 整格；说明里报成项几个、各哪一类，
      不成项的 spec fn 逐个点名）一项，证明每个成项的单元一项 proof:<--verify-function 点的名字>（说明是哪一类与它在核里的位置），
      空真每处 requires 一项 vacuity:<所属 fn 名>#<函数内第几处>（说明是函数或闭包），必红世界每行一项 must-red:<变异名>（说明是表里第几行）。
      项名只当索引。只读登记表、核与必红世界表（核经 scripts/gate/lib_function_units.py 切单元），不起 Verus、不编；核没进仓时只列 toolchain 与 proof；登记表或表写坏了判红退 1。
      给了 --verdict-library：另往 stderr 每格一行「<格>：键对不上 N 条、无记录 M 条、键相同 K 条、拒算 R 条」，末了「键对不上共 N 条」（底稿第二节第 9 条 ⑦）；
        逐项照现在的键比库（与判那一格时算的键同一份），只读库、不写库、不起 Verus（Verus 要装着：键里的工具链读它的安装）；stdout 的项名逐行不变（自证 list-items-key-mismatch-count）。
  逐项点名与按条复用（proof、vacuity、must-red-worlds 收 --item；toolchain 只有一项，给 --item 退 2；proof 那一格 --item proof 是整格）：
      --item <项>（可给多次，取并集）只判点名的项，项名写错或缺项名退 2 并列出这一格可点名的项；点名单跑一项与它在整格里跑的判定相同（自证 item-alone-equals-whole-*）；
        证明只点了几个函数时不比各项之和。
      --verdict-library <目录> 给了就按条记判定（判定库 scripts/crash/lib_verdict_library.py）：proof、vacuity、must-red-worlds 每一项判绿当场原子落盘，
        下一趟这一项的键相同就复用、不起 Verus；--no-reuse 只记不读（照样全部现判）。不给 --verdict-library 不读不写。toolchain 每趟现核、不复用。
      键分两列（判定库文件头「键」一段）：
        判法单元指纹：这一格判结局的那个函数（check_proof、check_vacuity、check_must_red_worlds）在这份脚本里的单元闭包键（判定库与 lib_function_units.py 不进）；
          变了整格每一项都现判，原因写「判法单元指纹 变了」并点名变了的单元。
        输入（逐样的单元键或摘要按名字排好再取 sha256）——每一项都带的两样：工具链（tool-versions 打的那一行：Verus、它自带的 vstd、z3、rustup 工具链）与
          rlimit（登记的数）；限时不进（超时不入库）、verus-pins.tsv 整份不进（它的注释、verified 的登记数与别的行不改任何一项的结论）。
          证明每项另加「被验函数单元闭包」：那个 fn 单元（clone 那一项是 derive 它的类型单元），连同它 requires / ensures / decreases 与函数体里用到的
            spec fn、proof fn、类型、常量、broadcast use 点名的引理（lib_function_units.py 的切法与八条闭包规则，去注释的指纹，不带路径与行号）；
            只改注释、空行、别的 fn 不改它的键；不成项的 spec fn 改了，用到它的那几项现判、红句点名它（自证 reuse-spec-function-all-users）。
          空真每处另加「所属函数单元闭包」与「函数内第几处」：身份是所属函数单元与第几处 requires，不带路径与行号（空行让 requires 挪一行不现判）。
          必红世界每行另加「受影响的函数单元」（这一行原文所在的那几个单元，与 verus! 里闭包用到它们的每个 fn，这些入口并起来的单元闭包键：
            改坏之后可能报错的就是它们，整个 crate 验、数证明错条数）与「这一行」（原文、替换文、第五段；变异名与文件路径不进，改名不现判）。
        必红世界表 Verus 不读：改表里一行只现判那一行，证明与空真照复用；表头、注释与空行不进任何一项的键
          （自证 reuse-must-red-table-row-only、reuse-must-red-table-comment-only）。核 crate 里别的文件加一个空行、被验函数体里加一行注释都不现判
          （自证 reuse-crate-other-file-no-cells、reuse-comment-in-function-body）。
      只记绿：红、没跑成、被信号停了、超时的不记；证明每项随判定记下 verified 数，复用时各项之和用记下的数。每项一行
        「· <项>：绿|红，现判（<原因>：<变了的单元>）|复用（<记下的日期与一句判定>）」，末了「· 按条：现判 N、复用 M（现判的原因：<原因> <数>…）」，
        原因是 无记录、关了复用、没给判定库、记录读不出、拒算（<哪一样>）、记录里没有 verified 数、这一行本身不对、<输入里哪几样或判法单元指纹> 变了；
        环境里有 FORMAL_ITEM_TALLY_FILE 的（门禁给的）另往那份文件追加一行「格<TAB>现判数<TAB>复用数<TAB>原因分栏」。
  verus-proof-check.py fingerprint <包目录>      打印包源码的内容指纹与文件数（填登记表用）
  verus-proof-check.py tool-versions [<根>] [--verus-root 目录]   打印一行：version.json 的 version、commit，安装目录里 verus、rust_verify、z3、vstd.vir、
      libvstd.rlib 各自的 sha256（不在的写「没有」），安装目录顶层全部普通文件的整体摘要（install_top_level_sha256：逐个「文件名 NUL 它的 sha256 换行」按文件名字节序接起来再取 sha256，
      宏库、libverus_builtin.rlib 这类 Verus 读而上面五样之外的都在里面），与 VERUS_Z3_PATH（没设写「未设」，设成空串写「空串」；
      设了写它的值、它指的文件与那个文件的 sha256：值里没有 / 的照 PATH 找到那一份，与 Verus 的找法相同，PATH 里找不到写「找不到」），
      VERUS_ROOT（rust_verify 从它读宏库、libverus_builtin.rlib、libvstd.rlib、vstd.vir：没设写「未设」，空串写「空串」，设了写它的值，
      值是目录时再接上那个目录顶层全部普通文件的整体摘要，算法同 install_top_level_sha256）与 VERUS_USE_RUSTUP（设成 0 时 verus 不经 rustup 起 rust_verify：
      没设写「未设」，空串写「空串」，设了写原值），
      以及 launcher 写死的 rustup run <version.json 的 toolchain> 那条工具链（rustup_toolchain_fields）：RUSTUP_HOME（没设写「未设」、空串写「空串」、设了写原值，
      冒号后接解析到的真实路径，没设或空串时取 rustup 的默认 ~/.rustup）、rustup_toolchain_lib_sha256（<RUSTUP_HOME>/toolchains/<工具链名> 下 lib/ 顶层全部普通文件
      与 lib/rustlib/*/lib/ 下全部普通文件的整体摘要，「相对工具链目录的路径 NUL sha256 换行」按路径字节序接起来再取 sha256，冒号后接文件数；librustc_driver、libLLVM、
      std 与 core 的 rlib / rmeta 都在里面）、rustup（PATH 里找到的那一份、它的真实路径与 sha256）；本机多哈希约 500 MB，页缓存热时多约 0.25 秒
      （门禁准入登记 stage-inputs.tsv 的 environment= 用，进复用判定与全绿标记的指纹；Verus 没装也打一行、退 0）
  verus-proof-check.py --selftest                拿假的 Verus（一段按源码里的标记出判定的 python）、假的 cargo 缓存逐格判；弄坏开关逐个打开，每个都要让至少一格判错

核没进仓（登记的核一份都不在、crate 的 Cargo.toml 不依赖 vstd、crate 的 src 里一处 verus! 都没有）时四格都退 77（本次未跑）；
核不在而 crate 已经接了 Verus（依赖 vstd 或有 verus!）判红：核被删了不许悄悄变成本次未跑。

包源码的内容指纹：包目录下全部普通文件，去掉顶层的 Cargo.toml、Cargo.toml.orig、.cargo_vcs_info.json（crates.io 打包时改写或生成的）
与 .cargo-ok（cargo 解包时写的本机标记），逐个「sha256  相对路径」按路径字节序排好、每行一个换行，整段再取 sha256。

弄坏开关 VERUS_PROOF_CHECK_BREAK=<项>[,<项>…]（只给证红用；--selftest 与门禁样本在开着时都必须判错）。
逐项写开关名、弄坏哪一处（括号里是开着时判错的门禁样本），下一行起写打开后自证哪几格判错（项名是 selftest() 的格名，括号里是那一行判错行的开头；
每一项红时都另有汇总行「✗ verus-proof-check 自证 <格数> 格里 N 格判错：<格名…>」）：
  VERUS_PROOF_CHECK_BREAK=commit-unchecked  toolchain 不比 version.json 的 commit（样本 verus-toolchain-pinned-commit-red 判绿）
      toolchain-commit（「toolchain-commit 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=fingerprint-unchecked  toolchain 不比包源码的内容指纹（样本 verus-toolchain-pinned-fingerprint-red 判绿）
      toolchain-fingerprint（「toolchain-fingerprint 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=absent-kernel-not-run  核不在一律退 77（样本 verus-proof-kernel-deleted-red 退 77）
      kernel-deleted（「kernel-deleted 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=cheating-unchecked  不 grep 作弊标记、不带 --no-cheating（样本 verus-proof-cheating-red 判绿）
      proof-cheating（「proof-cheating 这一格判错」）、proof-rlimit-attribute（「proof-rlimit-attribute 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=count-unchecked  不比各项 verified 之和与登记的数（样本 verus-proof-count-red 判绿）
      proof-count、proof-sum-from-reused-records、proof-sum-short-red-named（「… 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=timeout-as-pass  超时当绿（样本 verus-proof-timeout-red 判绿）
      proof-timeout（「proof-timeout 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=vacuity-accepts-verified  插了 assert(false) 还验得过也当没事（样本 verus-vacuity-red 判绿）
      vacuity-vacuous（「vacuity-vacuous 这一格判错」）、vacuity-closure-vacuous（「vacuity-closure-vacuous 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=vacuity-all-at-once  一份核的全部插入点插进同一份副本只跑一遍，闭包那一处没单报就当它随外层验过了（样本 verus-vacuity-closure-red 判绿）
      vacuity-closure-vacuous（「vacuity-closure-vacuous 这一格判错」）、vacuity-vacuous（「vacuity-vacuous 这一格判错」）；
      另有几格跟着判错：item-alone-equals-whole-vacuity 与空真那几格按条复用（整份一跑不按项记判定）
  VERUS_PROOF_CHECK_BREAK=mutation-accepts-green  改坏的副本上验得过也当红了（样本 verus-must-red-worlds-survives-red 判绿）
      must-red-survives（「must-red-survives 这一格判错」）、must-red-proof-error-count（「must-red-proof-error-count 这一格判错」）、
      must-red-mixed-not-run-and-red（「must-red-mixed-not-run-and-red 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=rlimit-counts-as-red  必红世界里 rlimit 用尽的报错也当证明失败（样本 verus-must-red-worlds-rlimit-red 判绿）
      must-red-rlimit（「must-red-rlimit 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=rlimit-any-line-voids  必红世界里输出夹着 rlimit 用尽的一律不作数（另有证明错也不算）（样本 verus-must-red-worlds-rlimit-with-proof-error-green 判红）
      must-red-rlimit-with-proof-error（「must-red-rlimit-with-proof-error 这一格判错」）、
      must-red-rlimit-10-counts-proof-errors-only（「must-red-rlimit-10-counts-proof-errors-only 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=rlimit-attribute-unchecked  核里不认 verifier::rlimit 属性（样本 verus-proof-rlimit-attribute-red 判绿）
      proof-rlimit-attribute（「proof-rlimit-attribute 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=mutation-rlimit-attribute-accepted  必红世界不拒替换文里的 verifier::rlimit 属性（样本 verus-must-red-worlds-rlimit-attribute-red 判绿）
      must-red-rlimit-attribute（「must-red-rlimit-attribute 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=rlimit-unpinned  Verus 命令行不带 --rlimit（假 Verus 没见到 --rlimit 就报 vir 错）
      proof-green（「proof-green 这一格判错」）；起 Verus 的格几乎都跟着判错（证明逐函数、空真、必红世界与按条复用那几格），例 must-red-green、proof-count、
      vacuity-vacuous、vacuity-closure-vacuous、must-red-survives、must-red-proof-error-count、proof-rlimit-registered、proof-rlimit-too-low、proof-rlimit-default、
      must-red-rlimit、must-red-rlimit-with-proof-error、must-red-rlimit-10-counts-proof-errors-only、must-red-rlimit-20-counts-proof-errors-only、must-red-mixed-not-run-and-red
  VERUS_PROOF_CHECK_BREAK=anchor-unchecked  不核原文恰好命中一次（样本 verus-must-red-worlds-anchor-red 报不出「恰好命中一次」）
      must-red-anchor（「must-red-anchor 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=expectation-form-unchecked  必红世界不核第五段是不是 proof_errors=<正整数>，认不出的照样起 Verus、不比条数
                                                      （样本 verus-must-red-worlds-legacy-expectation-red 判绿）
      must-red-legacy-expectation（「must-red-legacy-expectation 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=proof-error-count-unchecked  必红世界不比证明错条数与第五段的 M（样本 verus-must-red-worlds-proof-error-count-red 判绿）
      must-red-proof-error-count（「must-red-proof-error-count 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=rlimit-counted-in-proof-errors  必红世界数证明错时把 rlimit 用尽也算进去（样本 verus-must-red-worlds-rlimit-with-proof-error-green 判红）
      must-red-rlimit-10-counts-proof-errors-only（「must-red-rlimit-10-counts-proof-errors-only 这一格判错」）、
      must-red-rlimit-with-proof-error（「must-red-rlimit-with-proof-error 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=wrapper-exit-as-proof-red  包装自己退 245、250、251、252、254 照旧报成这一格的问题、给这一格自己的出路（自证里的假包装退 250 与 245）
      proof-wrapper-exit（「proof-wrapper-exit 这一格判错」）、must-red-wrapper-exit（「must-red-wrapper-exit 这一格判错」）、
      proof-wrapper-245（「proof-wrapper-245 这一格判错」）；
      另有 3 格跟着判错：must-red-mixed-not-run-and-red、must-red-wrapper-lines-carried、proof-wrapper-too-big
  VERUS_PROOF_CHECK_BREAK=tool-versions-launcher-only  tool-versions 只打 version.json 与 verus 的 sha256，不看 rust_verify、z3、vstd 与 VERUS_Z3_PATH
      tool-versions-z3（「tool-versions-z3 这一格判错」）；另有 5 格跟着判错：tool-versions-install-top-level、tool-versions-z3-path-empty、
      tool-versions-z3-bare-name、tool-versions-verus-environment、tool-versions-rustup-toolchain
  VERUS_PROOF_CHECK_BREAK=tool-versions-five-files-only  tool-versions 不打安装目录顶层的整体摘要（安装里多一份宏库）
      tool-versions-install-top-level（「tool-versions-install-top-level 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=tool-versions-empty-z3-path-as-unset  VERUS_Z3_PATH 设成空串记成「未设」
      tool-versions-z3-path-empty（「tool-versions-z3-path-empty 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=tool-versions-bare-z3-from-current-directory  VERUS_Z3_PATH 写裸名时按当前目录找、不照 PATH 找（PATH 里换一份 z3）
      tool-versions-z3-bare-name（「tool-versions-z3-bare-name 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=tool-versions-verus-environment-ignored  tool-versions 不打 VERUS_ROOT 与 VERUS_USE_RUSTUP
                                                                   （两个变量没设、空串、换值、VERUS_ROOT 指的目录换内容）
      tool-versions-verus-environment（「tool-versions-verus-environment 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=tool-versions-rustup-ignored  tool-versions 不打 RUSTUP_HOME、rustup 工具链的库摘要与 rustup 程序
                                                        （假 RUSTUP_HOME 里 librustc_driver 末尾 1 字节改了、libcore rmeta 改了、RUSTUP_HOME 换一份同内容的拷贝、PATH 里换一份 rustup）
      tool-versions-rustup-toolchain（「tool-versions-rustup-toolchain 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=queue-refused-keeps-queueing  拿到包装的 252 之后其余照排（假包装一律睡 0.5 秒再退 252，94 行必红世界、8 处同时验要排 12 次）
      queue-refused-must-red-once（「queue-refused-must-red-once 这一格判错」：包装被起 94 次、挂钟超过 3.0 秒）、queue-refused-later-cells-once（proof 格排不上之后空真与必红世界
      照排）、queue-refused-started-rows-once；另有 4 格跟着判错：queue-refused-must-red-skipped-listed、queue-refused-started-rows-collected、
      queue-refused-vacuity-skipped、queue-refused-must-red-all-skipped
  VERUS_PROOF_CHECK_BREAK=not-run-masks-red  一格里有一处「没跑成」就只报没跑成的那几条，同一格里的真红不报（一行没跑成、一行真没红）
      must-red-mixed-not-run-and-red（「must-red-mixed-not-run-and-red 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=wrapper-output-dropped  「没跑成」那一条不带包装自己打的 ✗ 与 →
      must-red-wrapper-lines-carried（「must-red-wrapper-lines-carried 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=wrapper-252-one-howto  252 不分成因，一律给「看 --status、空出来再重跑」（要的量比总上限还大）
      proof-wrapper-too-big（「proof-wrapper-too-big 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=killed-as-proof-red  Verus 被信号停了照旧报「Verus 没出判定」、给这一格自己的出路（修证明）（样本 verus-proof-killed-red 判错）
      proof-killed（假包装退 137；「proof-killed 这一格判错」）、proof-killed-direct（假 Verus kill -9 自己；「proof-killed-direct 这一格判错」）、
      proof-killed-no-proof-howto（「proof-killed-no-proof-howto 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=wrapper-queue-lines-swallowed  内存包装排队报的那几行只收进变量、不转打到本进程的 stderr（假包装报一行排队、等它被转打出来）
      wrapper-queue-line-forwarded（「wrapper-queue-line-forwarded 这一格判错」）
逐项点名与按条复用的几项：
  VERUS_PROOF_CHECK_BREAK=list-items-pipe-unhandled  list-items 不照 SIGPIPE 惯例退（接 | head 时打 BrokenPipeError 的回溯、退 1）
      list-items-closed-pipe（「list-items-closed-pipe 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=item-filter-ignored  --item 不起作用、照判全部（写错的项名也不退 2）
      item-select-one、item-unknown、item-proof-one、item-proof-unknown（「… 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=reuse-key-whole-crate  被验函数那一样换回核 crate 整个目录的原文摘要（旧判法：整 crate 一把）
      reuse-comment-in-function-body（N38：被验函数体里只加一行注释，三格全现判；「reuse-comment-in-function-body 这一格判错」）；另有几格跟着判错：
      reuse-crate-other-file-no-cells、reuse-vacuity-line-moved、reuse-timeout-changed、reuse-proof-function-only、reuse-spec-function-all-users、
      reuse-must-red-renamed、reuse-must-red-table-row-only、reuse-must-red-table-comment-only
  VERUS_PROOF_CHECK_BREAK=reuse-key-without-units  键里不带被验函数的单元闭包（改了一个函数照旧复用）
      reuse-proof-function-only（「reuse-proof-function-only 这一格判错」）；另有 1 格跟着判错：reuse-spec-function-all-users
  VERUS_PROOF_CHECK_BREAK=reuse-key-function-only  被验函数那一样只取它自己的单元，不收它 requires / ensures 与函数体用到的 spec fn、proof fn
      reuse-spec-function-all-users（改一个 spec fn，用到它的两项照旧复用；「reuse-spec-function-all-users 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=vacuity-key-by-line-number  空真每处的键另带核的路径与 requires 的行号（旧判法）
      reuse-vacuity-line-moved（N39：requires 上面加一个空行，空真两处现判；「reuse-vacuity-line-moved 这一格判错」）；
      另有 1 格跟着判错：reuse-comment-in-function-body（那一行注释让后面那个 fn 的 requires 挪了一行）
  VERUS_PROOF_CHECK_BREAK=must-red-key-with-name-and-path  必红世界「这一行」另带变异名与文件路径
      reuse-must-red-renamed（只改变异名那一行现判；「reuse-must-red-renamed 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=reuse-key-with-timeout  键里带限时（旧判法）
      reuse-timeout-changed（E11：改限时三格全现判；「reuse-timeout-changed 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=reuse-key-without-toolchain  键里不带工具链（Verus 安装里多一份库照旧复用）
      reuse-toolchain-changed（「reuse-toolchain-changed 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=reuse-key-without-judging-units  判法单元指纹一律交空串（判结局的函数改了照旧复用）
      reuse-judging-units-changed（「reuse-judging-units-changed 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=reuse-key-without-row  键里不带「这一行 / 这一处」（同一格几项共用一个键，只判过 A 就拿它的判定当 B 的）
      reuse-row-own-key（「reuse-row-own-key 这一格判错」）；另有 4 格跟着判错：reuse-red-not-recorded、reuse-resume-after-kill、reuse-must-red-table-row-only、
        prune-after-change
  VERUS_PROOF_CHECK_BREAK=reuse-ignores-key  这一项判绿过就复用，不比键
      prune-after-change（剪枝自证：改了一行之后开着复用与关掉复用逐项判定不同）；另有几格跟着判错：reuse-proof-function-only、reuse-toolchain-changed、
        reuse-must-red-table-row-only、reuse-spec-function-all-users、reuse-judging-units-changed、reuse-must-red-renamed
  VERUS_PROOF_CHECK_BREAK=reuse-records-red  红的也记成绿
      reuse-red-not-recorded（「reuse-red-not-recorded 这一格判错」）；另有 1 格跟着判错：reuse-resume-after-kill
  VERUS_PROOF_CHECK_BREAK=reuse-records-whole-cell-only  一项判完不当场落盘，整格判完、一处问题都没有才一起记（被打断或有一项红，判过的全不留）
      reuse-resume-after-kill（第一趟 C 那一处被 kill -9，第二趟同一条命令要只判 C：「reuse-resume-after-kill 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=threads-default-cpu-count  --threads 的缺省取核数（改前的样子），不取登记的重活默认并行度
      threads-default-registered（「threads-default-registered 这一格判错」）
  证明成哪几项、每项怎么判（proof_items、verification_clean、check_proof；假 Verus 照真 Verus 的形态出 JSON，自证 fake-verus-real-shape 盯着它）：
  VERUS_PROOF_CHECK_BREAK=spec-fn-as-item  不带 decreases 的 spec fn 也成项（改前的样子：真仓 90 项里 49 个 spec fn 验出 0 条）（样本 verus-must-red-worlds-list-items 判错）
      proof-spec-fn-not-item（「proof-spec-fn-not-item 这一格判错」）；另有几格跟着判错：reuse-spec-function-all-users、proof-entire-crate-false-not-red 等
  VERUS_PROOF_CHECK_BREAK=spec-decreases-not-item  带 decreases 的 spec fn 不成项（它的终止义务没人验，之和少）
      proof-spec-decreases-item（「proof-spec-decreases-item 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=derived-clone-not-item  verus! 里 derive(Clone) 的类型不成项（<类型>::clone 没人验，之和少；真仓少 8 条）
      proof-derived-clone-item（「proof-derived-clone-item 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=require-success-key-per-function  --verify-function 下 JSON 没有 success 键当没验过（改前的样子：真仓 90 项全红）
      proof-fn-verified-one-green（「proof-fn-verified-one-green 这一格判错」）；proof-green、vacuity-vacuous 等跟着判错
  VERUS_PROOF_CHECK_BREAK=require-entire-crate-per-function  按函数验也要 is-verifying-entire-crate 是 true
      proof-entire-crate-false-not-red（「proof-entire-crate-false-not-red 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=zero-verified-accepted  点的函数一条义务都没验（verified 0）也判绿
      proof-zero-verified-red（「proof-zero-verified-red 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=nonzero-errors-accepted  errors 不是 0 也当验过（不是 vir 错的）
      proof-errors-one-red（「proof-errors-one-red 这一格判错」）、proof-broken
  VERUS_PROOF_CHECK_BREAK=key-mismatch-by-item-only  list-items 数键时这一项判绿过就算键相同，不比键
      list-items-key-mismatch-count（「list-items-key-mismatch-count 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=reuse-lookup-skipped  一律不读库（键相同照样现判）
      reuse-three-questions（① 第二趟全现判）、reuse-sentinel（复用哨兵判红「复用失灵」，这一格要的就是红：照判；跟着判错的是 ① 那几格）
  REUSE_CHECKS_BREAK=rejudged-sentinel-passes（共用库 scripts/gate/lib_reuse_checks.py 的开关）  哨兵被现判不红
      reuse-sentinel（「reuse-sentinel 这一格判错」）
复用三问与复用哨兵（scripts/gate/lib_reuse_checks.py 文件头「一」「二」）：
  自证 reuse-three-questions 格三格的项一起算，打规范输出「REUSE_SELFTEST script=scripts/gate/verus-proof-check.py …」（门禁 code-tooling 的 reuse-selftests 格认它）；
  已有同义的格（reuse-first-run-records、reuse-second-run-reuses、reuse-comment-in-function-body、reuse-crate-other-file-no-cells、reuse-proof-function-only、
  reuse-spec-function-all-users）在那一格的注释里点名。
  每一趟真跑（proof、vacuity、must-red-worlds 三格各一份）开跑前从库里按键抽哨兵（候选按键文件查，lib_reuse_checks.verdict_library_candidate，与 lookup 分开），
  抽中强制现判的那一项那一行写「强制现判（复用哨兵抽中…）」；哨兵被现判、强制现判判定与库里不同，那一格判红。种子与条数取 SINGLEFS_REUSE_SENTINEL_SEED、
  SINGLEFS_REUSE_SENTINEL_COUNT；--no-reuse、SINGLEFS_GATE_FULL=1、没给库、list-items 数键的那一趟不抽。
管不到的：spec 本身写得对不对；核与调它的 singlefs-core 之间（recovery.rs 调核那一段）没被证明的部分；插入点只认「requires 整行起头、函数体的 { 单独起行或是闭包那一行的头一个 {」
  这种写法（verusfmt 与本仓的写法），别的写法认不出就判红要求改写法。
"""
import collections
import concurrent.futures
import contextlib
import glob
import io
import hashlib
import json
import os
import re
import shutil
import signal
import subprocess
import sys
import tempfile
import threading
import time
import tomllib

import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
sys.path.insert(0, SCRIPT_DIRECTORY)
sys.path.insert(0, os.path.join(os.path.dirname(os.path.realpath(__file__)), "..", "crash"))
sys.path.insert(0, os.path.join(SCRIPT_DIRECTORY, "..", "hosts"))
from heavy_threads import default_heavy_threads  # noqa: E402
from lib_verdict_library import FunctionUnitIndex, Tally, UnitKey, VerdictLibrary, digest_text, digest_tree, judging_unit_key  # noqa: E402
from lib_verdict_library import key_of, read_json, refused_component  # noqa: E402
import lib_reuse_checks as reuse_checks  # noqa: E402
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
REGISTRY = ".claude/gate.d/verus-pins.tsv"
# 复用三问的规范输出与复用哨兵认的脚本名（scripts/gate/lib_reuse_checks.py 文件头「一」「二」）
SCRIPT_KEY = "scripts/gate/verus-proof-check.py"
EXIT_GREEN, EXIT_RED, EXIT_USAGE, EXIT_NOT_RUN = 0, 1, 2, 77
WRAPPER_TIMEOUT_EXIT = 253
# 登记表没有 rlimit 行时 Verus 命令行上写的数：Verus 0.2026.09.20 自己的默认值（登记表 rlimit 行的依据写在那一行上面的注释里）
DEFAULT_RLIMIT = 10
RLIMIT_EXHAUSTED = "Resource limit (rlimit) exceeded"
# 必红世界表第五段的唯一形态：proof_errors=<证明错条数>（proof_error_messages 数的那些，不含 rlimit 用尽）
EXPECTED_PROOF_ERRORS = re.compile(r"proof_errors=([1-9][0-9]*)")
EXPECTATION_HOWTO = ("必红世界表第五段一律写 proof_errors=M：M 是改坏的副本在登记的 rlimit 上 Verus 报的证明错条数"
                     "（error 起头的报错，除去含 Resource limit (rlimit) exceeded 的与以 aborting due to 起头的），至少 1；"
                     "不写输出片段、不写「verified, M errors」（Verus 把 rlimit 用尽也计进 errors，换个 rlimit 那个数就变）")
WRAPPER_OWN_EXITS = {245: "显存不够（起止之间内核报了 NV_ERR_NO_MEMORY）", 250: "撞了这一条自己的内存上限", 251: "包装用法错或起不了 scope", 252: "内存不够排不上",
                     254: "被总上限挤掉"}
MEMORY_CAP_RUNNER = os.path.join(SCRIPT_DIRECTORY, "..", "hosts", "run-with-memory-cap.sh")
# 包装自己的结局（WRAPPER_OWN_EXITS）那一处没跑成时给的出路，照 run-with-memory-cap.sh 文件头「退出码」一段、按码给（wrapper_howto）
WRAPPER_HOWTO_HEAD = "照 scripts/hosts/run-with-memory-cap.sh 文件头「退出码」一段办："
WRAPPER_HOWTO_TAIL = "；不拿掉包装跑，不改证明、核或必红世界表"
WRAPPER_HOWTO_BY_EXIT = {
    250: "250 是这一处要的比上限多，把 GATE_VERUS_MEMORY_MAX（或 --memory-cap）设大再跑",
    251: "251 是包装用法错或起不了 scope：照包装自己打的 → 修（systemctl --user status 看用户级 systemd 在不在），修好再重跑这一格",
    252: "252 是排队等满还放不下：先跑 bash scripts/hosts/run-with-memory-cap.sh --status 看 slice 被谁占着，空出来再重跑这一格",
    254: "254 是 slice 满了被挤掉：先跑 bash scripts/hosts/run-with-memory-cap.sh --status 看 slice 被谁占着，空出来再重跑这一格",
    245: "245 是这一处退了非 0 而同一段时间内核报了显存不够（Verus 不上卡，多半是别的进程撞的）：nvidia-smi 看谁在用卡，等它跑完再重跑这一格",
}
# 252 的另一种成因：包装打「等多久都放不下」（这一处要的量比 slice 的总上限还大，不等），重跑多少次都一样
WRAPPER_TOO_BIG_MARK = "等多久都放不下"
WRAPPER_TOO_BIG_HOWTO = ("252 是这一处要的量比 slice 的总上限还大、等多久都放不下：把 GATE_VERUS_MEMORY_MAX（或 --memory-cap）写小到总上限以内再跑"
                         "（总上限看 bash scripts/hosts/run-with-memory-cap.sh --status）；真要这么多，先照包装文件头「余量」一段调 RUN_WITH_MEMORY_CAP_RESERVE")
# Verus 被信号停了（不在 250–254 里的 128 + 信号号，或直接起时负的信号号）那一处的出路：照包装文件头，不写修证明（killed_signal）
KILLED_HOWTO = ("照 scripts/hosts/run-with-memory-cap.sh 文件头「退出码」一段办：撞了这一条自己的上限由包装认成 250，外面 kill -9 同样退 137——这一次是外面停的："
                "查是谁在那一刻停了它（同一时刻的 ps、journalctl --user、看门狗的记录），那件事不再发生之后重跑这一格；不拿掉包装跑，不改证明、核或必红世界表")
# 一处没跑成那一条 ✗ 的开头：一格里有它又有别的 ✗ 的，按红记（report_cell）
NOT_RUN_MARK = "没跑成：内存包装 scripts/hosts/run-with-memory-cap.sh 退 "
# 同一道里已有一处拿到内存包装的 252（排不上），这一处没起包装、没排队（QueueRefusal）；report_cell 把这几处并成一条 ✗
QUEUE_SKIPPED_MARK = "没跑成：内存包装排不上，没再排队"
# 门禁给的一份文件：一格里第一处拿到 252 就写一行「哪一处<TAB>出路」，后面几格开跑前读到就不再排队（QueueRefusal）
QUEUE_REFUSED_FILE_VARIABLE = "VERUS_PROOF_CHECK_QUEUE_REFUSED_FILE"
# tool-versions 除 verus 外另哈希的安装文件：verus 只是起 rust_verify 的外壳，判定还取决于 rust_verify、z3 与 vstd
TOOL_VERSION_FILES = ("rust_verify", "z3", "vstd.vir", "libvstd.rlib")
# Verus 的 launcher 写死 rustup run <version.json 的 toolchain> 起 rust_verify；rust_verify 从那条工具链的 lib/ 读 librustc_driver 与 libLLVM，
# 从 lib/rustlib/<三元组>/lib/ 读 std、core 这些 crate 的 rlib 与 rmeta（rustup_toolchain_fields）
RUSTUP_DEFAULT_HOME = os.path.join(os.path.expanduser("~"), ".rustup")
EXCLUDED_PACKAGE_FILES = {"Cargo.toml", "Cargo.toml.orig", ".cargo_vcs_info.json", ".cargo-ok"}
CHEATING_MARKER = re.compile(r"\b(external_body|external_fn_specification|assume\w*|admit)\b|verifier\s*::\s*external\b|verifier\s*\(\s*external\s*\)")
# 函数（或模块）上的 rlimit 属性：#[verifier::rlimit(N)] 与旧写法 #[verifier(rlimit(N))] 都盖过命令行的 --rlimit（Verus 0.2026.09.20 实测两种写法都认、--no-cheating 不拒）
RLIMIT_ATTRIBUTE = re.compile(r"verifier\s*::\s*rlimit\b|verifier\s*\(\s*rlimit\b")
FUNCTION_LINE = re.compile(r"^\s*(pub(\([^)]*\))?\s+)?(open\s+|closed\s+)?(proof\s+|spec\s+|exec\s+)?fn\s+(\w+)")
DIAGNOSTIC_LOCATION = re.compile(r"^\s*-->\s+(\S+?):(\d+):(\d+)")
BREAK = {item for item in os.environ.get("VERUS_PROOF_CHECK_BREAK", "").split(",") if item}
# 门禁给的一份文件：每格判完往里追加一行「格<TAB>现判数<TAB>复用数<TAB>原因分栏」（lib_verdict_library.Tally.append_to），门禁整道相加
ITEM_TALLY_FILE_VARIABLE = "FORMAL_ITEM_TALLY_FILE"


class Red(Exception):
    """这一格判红：message 是卡在哪，howto 是下一步。"""

    def __init__(self, message, howto):
        super().__init__(message)
        self.message, self.howto = message, howto


class NotRun(Exception):
    """这一格本次未跑（核还没进仓）。"""


def report_red(message, howto):
    print(f"  ✗ {message}")
    print(f"     → 怎么办：{howto}")


def report_cell(problems):
    """一格的问题逐条打（(卡在哪, 下一步)）；既有「没跑成」又有真红的，末尾另打一行按红记。弄坏开关 not-run-masks-red 下有没跑成就只打没跑成的那几条。
    没再排队的那几处（QUEUE_SKIPPED_MARK）并成一条，放在最后：逐个列出是哪几处、同一道里先排不上的是哪一处。"""
    not_run = [problem for problem in problems if NOT_RUN_MARK in problem[0] or QUEUE_SKIPPED_MARK in problem[0]]
    real = [problem for problem in problems if NOT_RUN_MARK not in problem[0] and QUEUE_SKIPPED_MARK not in problem[0]]
    skipped = [problem for problem in problems if QUEUE_SKIPPED_MARK in problem[0]]
    if skipped:
        labels = [problem[0].split(QUEUE_SKIPPED_MARK, 1)[0].rstrip("：") or "Verus 那一次" for problem in skipped]
        reason = skipped[0][0].split(QUEUE_SKIPPED_MARK, 1)[1]
        problems = [problem for problem in problems if QUEUE_SKIPPED_MARK not in problem[0]] + [
            (f"{QUEUE_SKIPPED_MARK}{reason}：这一格 {len(skipped)} 处 Verus 没起——{'、'.join(labels)}", skipped[0][1])]
    if not_run and "not-run-masks-red" in BREAK:
        problems, real = [problem for problem in problems if NOT_RUN_MARK in problem[0] or QUEUE_SKIPPED_MARK in problem[0]], []
    for message, howto in problems:
        report_red(message, howto)
    if not_run and real:
        report_red(f"这一格 {len(real)} 条真红、{len(not_run)} 条没跑成：按红记，不是没跑成（没跑成的那几处重跑不替真红作保）",
                   "先照真红那几条各自的 → 修；没跑成的那几处照它们的 → 空出来再重跑")


def strip_line_comment(line):
    position = line.find("//")
    return line if position < 0 else line[:position]


# ── 登记表 ──

class Registry:
    def __init__(self):
        self.tag = self.commit = self.zip_sha256 = self.install_relative = None
        self.packages = []
        self.crate_directory = None
        self.verified_count = None
        self.kernels = []
        self.must_red_table = None
        self.rlimit = None


def read_registry(root):
    path = os.path.join(root, REGISTRY)
    if not os.path.isfile(path):
        raise Red(f"登记表 {REGISTRY} 不在 {root} 里", "它随仓走；被删了从 git 找回，格式写在它自己的文件头")
    registry = Registry()
    problems = []
    for number, raw in enumerate(open(path, encoding="utf-8").read().split("\n"), 1):
        if not raw.strip() or raw.startswith("#"):
            continue
        fields = raw.split("\t")
        kind, width = fields[0], {"release": 5, "package": 6, "crate": 3, "kernel": 2, "must-red-worlds": 2, "rlimit": 2}.get(fields[0])
        if width is None or len(fields) != width:
            problems.append(f"第 {number} 行认不出（第一段 {kind!r}、{len(fields)} 段）")
            continue
        if kind == "release":
            if registry.tag is not None:
                problems.append(f"第 {number} 行：release 行不止一行")
            registry.tag, registry.commit, registry.zip_sha256, registry.install_relative = fields[1:]
            if not re.fullmatch(r"[0-9a-f]{40}", registry.commit) or not re.fullmatch(r"[0-9a-f]{64}", registry.zip_sha256):
                problems.append(f"第 {number} 行：commit 要 40 位、zip 的 sha256 要 64 位小写十六进制")
        elif kind == "package":
            name, version, sha1, path_in_vcs, fingerprint = fields[1:]
            if not re.fullmatch(r"[0-9a-f]{40}", sha1) or not re.fullmatch(r"[0-9a-f]{64}", fingerprint):
                problems.append(f"第 {number} 行：sha1 要 40 位、内容指纹要 64 位小写十六进制")
            registry.packages.append({"name": name, "version": version, "sha1": sha1, "path_in_vcs": path_in_vcs, "fingerprint": fingerprint})
        elif kind == "crate":
            if registry.crate_directory is not None:
                problems.append(f"第 {number} 行：crate 行不止一行")
            registry.crate_directory = fields[1].rstrip("/")
            if not fields[2].isdigit() or int(fields[2]) == 0:
                problems.append(f"第 {number} 行：verified 的登记数要是正整数，读到 {fields[2]!r}")
            else:
                registry.verified_count = int(fields[2])
        elif kind == "kernel":
            registry.kernels.append(fields[1])
        elif kind == "rlimit":
            if registry.rlimit is not None:
                problems.append(f"第 {number} 行：rlimit 行不止一行")
            if not fields[1].isdigit() or int(fields[1]) == 0:
                problems.append(f"第 {number} 行：rlimit 要是正整数，读到 {fields[1]!r}")
            else:
                registry.rlimit = int(fields[1])
        else:
            if registry.must_red_table is not None:
                problems.append(f"第 {number} 行：must-red-worlds 行不止一行")
            registry.must_red_table = fields[1]
    if registry.tag is None or registry.crate_directory is None or not registry.kernels or registry.must_red_table is None or not registry.packages:
        problems.append("release、crate、must-red-worlds 各要恰好一行，package 与 kernel 至少一行")
    for kernel in registry.kernels:
        if registry.crate_directory and not kernel.startswith(registry.crate_directory + "/"):
            problems.append(f"核 {kernel} 不在登记的 crate {registry.crate_directory} 里")
    if problems:
        raise Red(f"登记表 {REGISTRY} 写坏了：" + "；".join(problems), "照它文件头的格式改；一行一条、制表符分隔")
    if registry.rlimit is None:
        registry.rlimit = DEFAULT_RLIMIT
    return registry


def kernel_files(root, registry):
    """登记的核都在就交回它们；核没进仓 NotRun；核不在而 crate 已经接了 Verus 判红。"""
    missing = [kernel for kernel in registry.kernels if not os.path.isfile(os.path.join(root, kernel))]
    if not missing:
        return [os.path.join(root, kernel) for kernel in registry.kernels]
    crate = os.path.join(root, registry.crate_directory)
    evidence = []
    manifest = os.path.join(crate, "Cargo.toml")
    if os.path.isfile(manifest) and re.search(r"^\s*vstd\s*=", open(manifest, encoding="utf-8").read(), re.M):
        evidence.append(f"{registry.crate_directory}/Cargo.toml 依赖 vstd")
    for directory, _subdirectories, names in os.walk(os.path.join(crate, "src")):
        for name in sorted(names):
            if name.endswith(".rs") and "verus!" in open(os.path.join(directory, name), encoding="utf-8", errors="replace").read():
                evidence.append(os.path.relpath(os.path.join(directory, name), root) + " 里有 verus!")
    if (not evidence and len(missing) == len(registry.kernels)) or "absent-kernel-not-run" in BREAK:
        raise NotRun(f"登记的核 {' '.join(missing)} 还没进仓（crate 不依赖 vstd、src 里没有 verus!）：Verus 这一格本次无对象可判")
    raise Red(f"登记的核 {' '.join(missing)} 不在，而 crate 已经接了 Verus（{'；'.join(evidence) or '另一份登记的核在'}）",
              f"核被删了或改了名：找回它，或者照新路径改 {REGISTRY} 的 kernel 行；核的删除不许让这一格变成本次未跑")


# ── Verus 安装、crate、外部依赖 ──

def verus_directory(registry, override):
    if override:
        return os.path.abspath(override)
    if os.environ.get("SINGLEFS_VERUS_ROOT"):
        return os.path.abspath(os.environ["SINGLEFS_VERUS_ROOT"])
    data_home = os.environ.get("XDG_DATA_HOME") or os.path.join(os.path.expanduser("~"), ".local", "share")
    return os.path.join(data_home, registry.install_relative)


def read_version_json(directory):
    path = os.path.join(directory, "version.json")
    if not os.path.isfile(path):
        return None
    try:
        return json.load(open(path, encoding="utf-8")).get("verus", {})
    except (ValueError, OSError):
        return None


def crate_manifest(root, registry):
    path = os.path.join(root, registry.crate_directory, "Cargo.toml")
    if not os.path.isfile(path):
        raise Red(f"{registry.crate_directory}/Cargo.toml 不在", f"crate 目录改了就照新路径改 {REGISTRY} 的 crate 行")
    try:
        return tomllib.load(open(path, "rb"))
    except tomllib.TOMLDecodeError as error:
        raise Red(f"{registry.crate_directory}/Cargo.toml 读不出：{error}", "先让 cargo build 认得这份清单")


def check_dependencies(registry, manifest):
    """直接起的 Verus 只带它自带的 vstd：被验的 crate 除 vstd 之外还有依赖就编不过，开跑之前判红、说清下一步。"""
    others = sorted(name for name in manifest.get("dependencies", {}) if name != "vstd")
    if others:
        raise Red(f"{registry.crate_directory}/Cargo.toml 除 vstd 之外还依赖 {' '.join(others)}：直接起的 Verus 只带它自带的 vstd，编不了别的 crate",
                  "核的 crate 只依赖 vstd：要用的东西搬进核，或留在调用核的那一侧；确要加依赖，先给 scripts/gate/verus-proof-check.py "
                  "补上用 Verus 钉的工具链编依赖、交给 --extern 的一步（旧写法 git log -S prepare_externs 找得到）")


def copy_crate(root, registry, work):
    source = os.path.join(root, registry.crate_directory)
    destination = os.path.join(work, registry.crate_directory)
    shutil.copytree(source, destination, ignore=shutil.ignore_patterns("target"), symlinks=True)
    return destination


def verus_arguments(registry, manifest, threads, function=None):
    """Verus 的命令行；function 给了就只验这一个函数（--verify-root --verify-function <名>：证明逐函数一项、空真只验它所属的那一个函数）。
    Verus 0.2026.09.20 按子串点名、有恰好同名的就取它（主 agent 2026-10-15 在副本上量的：名字是别人子串的四个都退 0、没报 more than one match），
    所以 verus! 里的 fn 同名判红（verus_functions），名字是别人子串的照常用。"""
    package = manifest.get("package", {})
    entry = manifest.get("lib", {}).get("path", "src/lib.rs")
    arguments = ["--crate-type=lib", f"--edition={package.get('edition', '2021')}", "--crate-name", package.get("name", "crate").replace("-", "_"),
                 "--num-threads", str(threads), "--output-json"]
    if "rlimit-unpinned" not in BREAK:
        arguments += ["--rlimit", str(registry.rlimit)]
    if "cheating-unchecked" not in BREAK:
        arguments.insert(0, "--no-cheating")
    if function:
        arguments += ["--verify-root", "--verify-function", function]
    return arguments + [entry]


# 内存包装排队时报的那几行（run-with-memory-cap.sh：「排队（已等 …」每 60 秒一行，「排了 … 秒，起跑」放行时一行）
WRAPPER_QUEUE_LINE = re.compile(r"^run-with-memory-cap: 排(队（|了 )")


def run_forwarding_queue_lines(command, label, timeout=None, sink=None, **popen_keywords):
    """跑一条命令、收它的 stdout 与 stderr，交回 (退出码, stdout, stderr)；过了 timeout 秒（None 不限）停掉它，退出码记 WRAPPER_TIMEOUT_EXIT、stdout 记空。
    stderr 里内存包装排队报的那几行（WRAPPER_QUEUE_LINE）边跑边转打到 sink（默认本进程的 stderr），行首带 label；判定用的 stdout 与 stderr 不变。"""
    sink = sink if sink is not None else sys.stderr
    forward = "wrapper-queue-lines-swallowed" not in BREAK
    process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, errors="replace", **popen_keywords)
    stdout_parts, stderr_lines = [], []

    def pump_stderr():
        for line in process.stderr:
            stderr_lines.append(line)
            if forward and WRAPPER_QUEUE_LINE.match(line):
                print(f"  … verus-proof-check {label}：{line.rstrip()}", file=sink, flush=True)

    readers = [threading.Thread(target=lambda: stdout_parts.append(process.stdout.read()), daemon=True),
               threading.Thread(target=pump_stderr, daemon=True)]
    for reader in readers:
        reader.start()
    try:
        process.wait(timeout=timeout)
    except subprocess.TimeoutExpired:
        process.kill()   # 只点自己起的这一个进程
        process.wait()
        for reader in readers:
            reader.join()
        return WRAPPER_TIMEOUT_EXIT, "", "".join(stderr_lines)
    for reader in readers:
        reader.join()
    return process.returncode, "".join(stdout_parts), "".join(stderr_lines)


class QueueRefusal:
    """这一道里第一处拿到内存包装 252（排不上）的记录：(哪一处, 出路)。run_check 每格新建一份，几处同时验的共用它；
    环境里有 QUEUE_REFUSED_FILE_VARIABLE 的（门禁给的），前面几格写进那份文件的也认，自己第一次拿到 252 也写进去。
    记下之后这一格与后面几格还没起的 Verus 都不起包装、不排队（skipped_outcome），已经起了的照常收。弄坏开关 queue-refused-keeps-queueing 下一律照排。"""

    def __init__(self):
        self.lock = threading.Lock()
        self.first = None
        self.path = os.environ.get(QUEUE_REFUSED_FILE_VARIABLE) or None

    def refused(self):
        if "queue-refused-keeps-queueing" in BREAK:
            return None
        with self.lock:
            if self.first is not None:
                return self.first
        if self.path and os.path.isfile(self.path):
            with open(self.path, encoding="utf-8") as handle:
                where, _tab, howto = handle.readline().rstrip("\n").partition("\t")
            if where:
                return where, howto or wrapper_howto({"exit": 252, "stderr": ""})
        return None

    def record(self, key, label, outcome):
        own = wrapper_own_lines(outcome)
        where = f"{key} 格{'「' + label + '」' if label else ''}那一处退了 252" + (f"，包装说：{own[0][:200]}" if own else "")
        with self.lock:
            if self.first is not None:
                return
            self.first = (where, wrapper_howto(outcome))
        if self.path:
            with open(self.path, "a", encoding="utf-8") as handle:
                handle.write(f"{where}\t{self.first[1]}\n")


def skipped_outcome(earlier):
    """同一道里已经排不上（QueueRefusal.refused 交回的 (哪一处, 出路)）：这一处不起包装，记成没再排队。"""
    return {"exit": None, "timed_out": False, "wrapper": None, "json": None, "diagnostics": "", "stderr": "", "queue_skipped": earlier}


def queue_refusal(options):
    """经内存包装起 Verus 时（launcher 是 None）这一格共用的 QueueRefusal；直接起的（自证的 launcher）没有包装、不判排队，交 None。"""
    return options.get("queue_refusal") if options.get("launcher") is None else None


def run_verus(verus, crate_copy, arguments, options, key, label=None):
    """起一次 Verus；交回 {exit, timed_out, wrapper, json, diagnostics}。同一道里已经排不上的不起，交 skipped_outcome；这一次拿到 252 就记进 QueueRefusal。"""
    state = queue_refusal(options)
    earlier = state.refused() if state is not None else None
    if earlier:
        return skipped_outcome(earlier)
    command = [verus, *arguments]
    environment = dict(os.environ)
    timeout = options["timeout"]
    if options.get("launcher") is None:
        command = ["bash", MEMORY_CAP_RUNNER, options["memory_cap"],
                   "bash", os.path.join(SCRIPT_DIRECTORY, "..", "hosts", "capped.sh"), str(options["threads"]), "nice", "-n", "19", *command]
        environment.update(RUN_WITH_MEMORY_CAP_TIME_LIMIT=str(timeout), RUN_WITH_MEMORY_CAP_KEY=f"verus-proof-check {key}",
                           RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="门禁的上限按整个 crate 最坏的一处给，变异副本与插了断言的副本比平常大",
                           RUN_WITH_MEMORY_CAP_PARALLEL_REASON="门禁里空真与必红世界几处同时验，每处各一份副本与各自的上限")
        subprocess_timeout = None
    else:
        command = [*options["launcher"], *command]
        subprocess_timeout = timeout
    exit_code, stdout, stderr = run_forwarding_queue_lines(command, key, timeout=subprocess_timeout, cwd=crate_copy, env=environment)
    outcome = {"exit": exit_code, "timed_out": exit_code == WRAPPER_TIMEOUT_EXIT, "wrapper": WRAPPER_OWN_EXITS.get(exit_code) if options.get("launcher") is None else None,
               "json": None, "diagnostics": stderr + "\n" + stdout, "stderr": stderr}
    try:
        outcome["json"] = json.loads(stdout) if stdout.strip().startswith("{") else None
    except ValueError:
        outcome["json"] = None
    if state is not None and exit_code == 252:
        state.record(key, label, outcome)
    return outcome


def diagnostic_blocks(text):
    """stderr 里的「error: <说什么>」与它下面第一处「--> 文件:行:列」配成一对。"""
    blocks, pending = [], None
    for line in text.split("\n"):
        if line.startswith("error"):
            pending = line.split(":", 1)[1].strip() if ":" in line else line
            if pending.startswith("aborting due to"):
                pending = None
            continue
        match = DIAGNOSTIC_LOCATION.match(line)
        if match and pending is not None:
            blocks.append((pending, match.group(1), int(match.group(2))))
            pending = None
    return blocks


def proof_error_messages(text, count_rlimit=False):
    """输出里 error 起头的报错，除去含 rlimit 用尽的与以「aborting due to」起头的之后剩下的（证明错）；必红世界表第五段的 M 数的就是它们，
    只剩 rlimit 用尽的那一行不作数。count_rlimit 只给弄坏开关 rlimit-counted-in-proof-errors 用：rlimit 用尽也算进去。"""
    messages = []
    for line in text.split("\n"):
        if not line.startswith("error"):
            continue
        message = line.split(":", 1)[1].strip() if ":" in line else line
        if (RLIMIT_EXHAUSTED in message and not count_rlimit) or message.startswith("aborting due to"):
            continue
        messages.append(message)
    return messages


def integer_counts(results):
    """JSON 的 verified 与 errors 都是整数才交 True：缺一个是 vir 错没走到验证，或 Verus 换了 JSON 的写法。"""
    return all(isinstance(results.get(key), int) and not isinstance(results.get(key), bool) for key in ("verified", "errors"))


def outcome_problem(outcome, options):
    """这一次 Verus 没得出可判的结局时交回一句（没再排队、超时、包装结局、没出 JSON），得出了交 None。"""
    if outcome.get("queue_skipped"):
        return f"{QUEUE_SKIPPED_MARK}（同一道里 {outcome['queue_skipped'][0]}）"
    if outcome["timed_out"]:
        if "timeout-as-pass" in BREAK:
            return None
        return f"超过 {options['timeout']} 秒没出结局，按红记（C5：超时不当通过）"
    if outcome["wrapper"]:
        if "wrapper-exit-as-proof-red" in BREAK:
            return f"内存包装自己的结局（退 {outcome['exit']}，{outcome['wrapper']}），这一次不算结果"
        own_lines = [] if "wrapper-output-dropped" in BREAK else wrapper_own_lines(outcome)
        carried = "；包装自己打的：" + "".join(f"「{line}」" for line in own_lines) if own_lines else ""
        return (f"{NOT_RUN_MARK}{outcome['exit']}（{outcome['wrapper']}，上限 {options['memory_cap']}），"
                f"这一次的输出不算判定{carried}")
    signal_number = killed_signal(outcome)
    if signal_number is not None:
        tail = " | ".join(line for line in outcome["diagnostics"].strip().split("\n")[-4:] if line.strip())
        return f"Verus 被信号停了（退 {outcome['exit']}，信号 {signal_number}），这一次的输出不算判定{'（' + tail[:300] + '）' if tail else ''}"
    if outcome["json"] is None or "verification-results" not in outcome["json"]:
        tail = " | ".join(line for line in outcome["diagnostics"].strip().split("\n")[-6:] if line.strip())
        return f"Verus 没出判定（退 {outcome['exit']}，编不过或起不来）：{tail}"
    return None


def wrapper_own_lines(outcome):
    """内存包装自己往 stderr 打的 ✗ 与 → 那几行（去掉行首空白，原样）：outcome_problem 把它们带进「没跑成」那一条。"""
    return [line.strip() for line in outcome.get("stderr", "").split("\n") if line.strip().startswith(("✗", "→"))]


def wrapper_howto(outcome):
    """包装自己的结局那一处的出路：照包装文件头「退出码」一段、按码给；252 看包装打的是不是「等多久都放不下」（要的量比总上限还大），两种成因各一种。"""
    code = outcome["exit"]
    middle = WRAPPER_HOWTO_BY_EXIT.get(code, f"{code} 不在包装文件头列的结局里：照它自己打的 → 办")
    if code == 252 and "wrapper-252-one-howto" not in BREAK and any(WRAPPER_TOO_BIG_MARK in line for line in wrapper_own_lines(outcome)):
        middle = WRAPPER_TOO_BIG_HOWTO
    return WRAPPER_HOWTO_HEAD + middle + WRAPPER_HOWTO_TAIL


def problem_howto(outcome, howto):
    """outcome_problem 报了问题时给的出路：没再排队的给先排不上那一处的出路，包装自己的结局给包装文件头那一段（wrapper_howto），别的给这一格自己的 howto。"""
    if outcome.get("queue_skipped"):
        return outcome["queue_skipped"][1]
    if outcome["wrapper"] and "wrapper-exit-as-proof-red" not in BREAK:
        return wrapper_howto(outcome)
    if killed_signal(outcome) is not None:
        return KILLED_HOWTO
    return howto


def killed_signal(outcome):
    """Verus 被信号停了就交信号号：直接起时 subprocess 交负的信号号，经包装或 bash 起时退 128 + 信号号（129–192，碰不到包装自己的 250–254）；
    超时（包装的 253、本进程限时停掉的）与别的退出码交 None。弄坏开关 killed-as-proof-red 下一律交 None。"""
    code = outcome["exit"]
    if outcome["timed_out"] or outcome["wrapper"] or "killed-as-proof-red" in BREAK:
        return None
    if code < 0:
        return -code
    return code - 128 if 128 < code <= 128 + 64 else None


# ── 四格 ──

def check_toolchain(root, options):
    registry = read_registry(root)
    kernel_files(root, registry)
    problems = []
    directory = verus_directory(registry, options.get("verus_root"))
    version = read_version_json(directory)
    if version is None:
        problems.append((f"Verus 安装 {directory} 里没有读得出的 version.json", f"装登记的发布 {registry.tag}，或用 SINGLEFS_VERUS_ROOT 指到它的安装目录"))
    else:
        if version.get("version") != registry.tag:
            problems.append((f"Verus 安装的 version 是 {version.get('version')}，登记的是 {registry.tag}", f"换装 {registry.tag}；要升级先改 {REGISTRY} 并复核同源"))
        if version.get("commit") != registry.commit and "commit-unchecked" not in BREAK:
            problems.append((f"Verus 安装的 commit 是 {version.get('commit')}，登记的是 {registry.commit}", f"换装 {registry.tag}；要升级先改 {REGISTRY} 并复核同源"))
    zip_path = os.path.join(os.path.dirname(directory), "verus.zip")
    zip_note = "安装目录旁没有 verus.zip，没核它的 sha256"
    if os.path.isfile(zip_path):
        digest = hashlib.sha256(open(zip_path, "rb").read()).hexdigest()
        zip_note = f"verus.zip 的 sha256 与登记相同"
        if digest != registry.zip_sha256:
            problems.append((f"{zip_path} 的 sha256 是 {digest}，登记的是 {registry.zip_sha256}", "重新下载登记的发布包，或把改过的那份挪走"))
    lock_path = os.path.join(root, "Cargo.lock")
    locked = {}
    if not os.path.isfile(lock_path):
        problems.append(("仓根没有 Cargo.lock", "cargo generate-lockfile 之后照登记钉版本"))
    else:
        for package in tomllib.load(open(lock_path, "rb")).get("package", []):
            locked.setdefault(package.get("name"), []).append(package.get("version"))
    cargo_home = options.get("cargo_home") or os.environ.get("CARGO_HOME") or os.path.join(os.path.expanduser("~"), ".cargo")
    for package in registry.packages:
        name, version_text = package["name"], package["version"]
        if os.path.isfile(lock_path) and locked.get(name) != [version_text]:
            problems.append((f"Cargo.lock 里 {name} 的版本是 {locked.get(name) or '没有'}，登记的是 [{version_text}]", f"依赖写成 {name} = \"={version_text}\"（或经 vstd 带进来）再 cargo update -p {name} --precise {version_text}"))
        directories = sorted(glob.glob(os.path.join(cargo_home, "registry", "src", "*", f"{name}-{version_text}")))
        if not directories:
            problems.append((f"本机 cargo 缓存（{cargo_home}/registry/src/*/）里没有 {name}-{version_text}", "在仓根跑 cargo fetch --locked"))
        for package_directory in directories:
            vcs_path = os.path.join(package_directory, ".cargo_vcs_info.json")
            try:
                vcs = json.load(open(vcs_path, encoding="utf-8"))
            except (OSError, ValueError):
                vcs = {}
            if vcs.get("git", {}).get("sha1") != package["sha1"] or vcs.get("path_in_vcs") != package["path_in_vcs"]:
                problems.append((f"{package_directory} 的 .cargo_vcs_info.json 是 sha1={vcs.get('git', {}).get('sha1')} path_in_vcs={vcs.get('path_in_vcs')}，"
                                 f"登记的是 {package['sha1']} {package['path_in_vcs']}", "缓存被别的来源顶掉了：删掉这个目录再 cargo fetch --locked"))
            fingerprint, _count = package_source_fingerprint(package_directory)
            if fingerprint != package["fingerprint"] and "fingerprint-unchecked" not in BREAK:
                problems.append((f"{package_directory} 的源码内容指纹是 {fingerprint}，登记的是 {package['fingerprint']}",
                                 "缓存里的源码被改过或换了来源：删掉这个目录再 cargo fetch --locked；确要换包就先改登记并复核同源"))
    if problems:
        for message, howto in problems:
            report_red(message, howto)
        return EXIT_RED
    print(f"  ✓ Verus {registry.tag}（commit {registry.commit[:12]}…）与登记相同，{zip_note}；Cargo.lock 与本机缓存里 {len(registry.packages)} 个包的版本、出处与源码指纹都与登记相同；按条：toolchain 一项，不复用（每趟现核）")
    return EXIT_GREEN


def cheating_markers(root, kernels):
    found = []
    for kernel in kernels:
        for number, line in enumerate(open(kernel, encoding="utf-8").read().split("\n"), 1):
            match = CHEATING_MARKER.search(strip_line_comment(line))
            if match:
                found.append(f"{os.path.relpath(kernel, root)}:{number}：{match.group(0)}")
    return found


def rlimit_attributes(root, kernels):
    found = []
    for kernel in kernels:
        for number, line in enumerate(open(kernel, encoding="utf-8").read().split("\n"), 1):
            match = RLIMIT_ATTRIBUTE.search(strip_line_comment(line))
            if match:
                found.append(f"{os.path.relpath(kernel, root)}:{number}：{match.group(0)}")
    return found


def verus_executable(registry, options):
    directory = verus_directory(registry, options.get("verus_root"))
    executable = os.path.join(directory, "verus")
    version = read_version_json(directory)
    if not os.access(executable, os.X_OK) or version is None:
        raise Red(f"找不到 Verus（{executable} 不可执行或旁边没有 version.json）",
                  f"装登记的发布 {registry.tag}（{REGISTRY} 的 release 行），或用 SINGLEFS_VERUS_ROOT 指到它的安装目录")
    return executable, version


def prepared(root, options):
    registry = read_registry(root)
    kernels = kernel_files(root, registry)
    manifest = crate_manifest(root, registry)
    check_dependencies(registry, manifest)
    executable, _version = verus_executable(registry, options)
    return registry, kernels, manifest, executable


class UsageError(Exception):
    """--item 点名了没有的项、或给了这一格不收的 --item：退 2，列出可点名的项。"""

    def __init__(self, message, available):
        super().__init__(message)
        self.message, self.available = message, available


# ── 按条判定：键与复用（判定库 lib_verdict_library.py；判据在文件头「逐项点名与按条复用」一段）──

# 判法单元：三格各自判结局的那个函数（连同它调到的；判定库与 lib_function_units.py 两份不进），取这份脚本所在的仓
JUDGING_ENTRIES = {"proof": "check_proof", "vacuity": "check_vacuity", "must-red-worlds": "check_must_red_worlds"}
_JUDGING_UNITS = {}


def judging_units(cell):
    """判法单元指纹：这一格判结局的函数（JUDGING_ENTRIES）在这份脚本里的单元闭包键。自证给 options["judging_units"] 时用那一个（造「判法单元指纹变了」）；
    弄坏开关 reuse-key-without-judging-units 下一律交空串（判法变了照旧复用）。"""
    if "reuse-key-without-judging-units" in BREAK:
        return ""
    if cell not in _JUDGING_UNITS:
        try:
            _JUDGING_UNITS[cell] = judging_unit_key(REPOSITORY_ROOT, [f"{os.path.relpath(os.path.realpath(__file__), REPOSITORY_ROOT)}::{JUDGING_ENTRIES[cell]}"])
        except ValueError as error:
            _JUDGING_UNITS[cell] = UnitKey(None, (), (f"refused\tjudging-units\t{os.path.basename(__file__)}:1\t{error}",))
    return _JUDGING_UNITS[cell]


def unit_index(root, options):
    """被判的仓的函数单元索引（lib_function_units.py 的切法，Rust 工作区读一次）：一格里各项共用，存进 options。"""
    if options.get("unit_index") is None:
        options["unit_index"] = FunctionUnitIndex(root)
    return options["unit_index"]


def verus_ranges(rust_file):
    """一份核里每个 verus! { … } 的词号范围（{ 与配对的 }）。"""
    return [(position + 2, rust_file.pairs[position + 2]) for position in range(rust_file.count - 2)
            if rust_file.texts[position] == "verus" and rust_file.texts[position + 1] == "!" and rust_file.texts[position + 2] == "{"
            and position + 2 in rust_file.pairs]


def verus_functions(root, registry, kernels, options):
    """核里 verus! { … } 里的每个 fn 单元（spec、proof、exec 都算），按核与行号的次序：[(名, 核的相对路径, 单元)]。空真认所属函数、必红世界认受影响的函数用它；
    证明成哪几项另由 proof_items 从里面挑（spec fn 没有证明义务的不成项）。
    同一份核里两个 fn 同名、或 fn 在 impl / trait 里（--verify-function 按名点，这两种点不准）判红，出路是改名或挪成自由 fn。"""
    index = unit_index(root, options)
    functions = []
    try:
        workspace = index.workspace()
        for kernel in kernels:
            relative = os.path.relpath(kernel, root).replace(os.sep, "/")
            rust_file = workspace.file(relative)
            ranges = verus_ranges(rust_file)
            names = {}
            for unit in rust_file.units:
                if unit.kind != "fn" or unit.first_token is None or not any(low < unit.first_token < high for low, high in ranges):
                    continue
                if unit.container is not None:
                    raise Red(f"{relative}:{unit.first_line} 的 {unit.name} 在 verus! 里的 impl 或 trait 里：--verify-function 按名点不准它",
                              "把它挪成 verus! 里的自由 fn（或给 verus-proof-check.py 的 verus_functions 补上带容器的点法再用）")
                if unit.name in names:
                    raise Red(f"{relative} 的 verus! 里有两个 fn 叫 {unit.name}（第 {names[unit.name].first_line} 行与第 {unit.first_line} 行）：--verify-function 按名点不准",
                              "给其中一个改名，同步改 .claude/gate.d/verus-pins.tsv 与必红世界表里引它的行")
                names[unit.name] = unit
                functions.append((unit.name, relative, unit))
    except (OSError, KeyError, ValueError) as error:
        raise Red(f"切不出核里的函数单元（scripts/gate/lib_function_units.py）：{error}", "先让 python3 scripts/gate/lib_function_units.py dump <根> --language rust 在这棵树上跑通")
    return functions


# verus! 里的 derive 只有 Clone 生成 Verus 要验的函数（<类型>::clone）。依据是主 agent 2026-10-15 的真 Verus 探针（0.2026.09.20，逐函数 verified 数在
# research/prompts/pruning-survey-2026-10-15/verus-probe.log）：--verify-function 点不中时 Verus 列的可点名函数是 90 个自由 fn、8 个 <类型>::clone
# （核里 derive(Clone, …) 的正好 8 个类型）与 2 个 CoreFailure::arrow_*（spec，没有义务）；Copy、Debug、PartialEq、Eq 的 derive 不在表里。
# 整 crate 验出 56 条 = 逐函数验出的 48 条 + 这 8 个 clone 各 1 条（推的：clone 那 8 条没有逐个量过）。
DERIVED_VERIFIED_TRAITS = ("Clone",)
PROOF_FN, EXEC_FN, SPEC_FN_WITH_DECREASES, DERIVED_CLONE = "proof fn", "exec fn", "带 decreases 的 spec fn", "derive(Clone) 生成的 clone"


class ProofItem(collections.namedtuple("ProofItem", "item verus_name relative unit kind entry estimate")):
    """证明的一项：项名 proof:<verus_name>、交给 --verify-function 的名字、核的相对路径、单元、哪一类（PROOF_FN 这几样）、
    算键的入口（<核>::<单元名>：fn 是它自己，clone 是那个类型）、静态估的义务条数（只给计数之和对不上时点名用，不判每一项）。"""


def leading_words(rust_file, unit, stop):
    """单元开头到 stop 那个词之前的词（外属性 #[…] 整段跳过）。"""
    words, index = [], unit.first_token
    while index < unit.last_token and rust_file.texts[index] != stop:
        if rust_file.texts[index] == "#" and index + 1 < rust_file.count and rust_file.texts[index + 1] == "[" and index + 1 in rust_file.pairs:
            index = rust_file.pairs[index + 1] + 1
            continue
        words.append(rust_file.texts[index])
        index += 1
    return words


def function_mode(rust_file, unit):
    """verus! 里一个 fn 的模式：spec、proof 或 exec（fn 前面没写模式的是 exec）。"""
    words = leading_words(rust_file, unit, "fn")
    return "spec" if "spec" in words else "proof" if "proof" in words else "exec"


def derived_traits(rust_file, unit):
    """类型单元开头的 #[derive(…)] 里点名的 trait。"""
    traits, index = [], unit.first_token
    while index + 1 < unit.last_token and rust_file.texts[index] == "#" and rust_file.texts[index + 1] == "[" and index + 1 in rust_file.pairs:
        close = rust_file.pairs[index + 1]
        if rust_file.texts[index + 2] == "derive":
            traits += [text for text in rust_file.texts[index + 3:close] if text not in ("(", ")", ",", ":")]
        index = close + 1
    return traits


def proof_items(root, registry, kernels, functions, options):
    """证明这一格的项：有证明义务的单元各一项，按核与行号的次序交 [ProofItem]，与没成项的 [(名, 核的相对路径)]。
    成项的四类：proof fn、exec fn（各一条，exec fn 里每个 while / loop / for 另一条：Verus 把循环体单独验）、带 decreases 的 spec fn（终止义务一条）、
    verus! 里 derive(Clone) 的类型（Verus 验它生成的 <类型>::clone，一条）。spec fn 不带 decreases 的没有证明义务（真 Verus 逐函数验报 verified 0）：
    不成项，随用到它的项进那一项的单元闭包键。项名 proof:<--verify-function 点的名字>；名字都是 Verus 可点名函数表里恰好那一个（自由 fn 的名字、<类型>::clone），
    Verus 有恰好同名的就取它，不会点到别人。弄坏开关：spec-fn-as-item 下 spec fn 一律成项（改前的样子），spec-decreases-not-item 下带 decreases 的 spec fn
    不成项，derived-clone-not-item 下 derive(Clone) 不成项。"""
    workspace = unit_index(root, options).workspace()
    by_kernel = {}
    for name, relative, unit in functions:
        by_kernel.setdefault(relative, []).append((name, unit))
    items, left_out = [], []
    for kernel in kernels:
        relative = os.path.relpath(kernel, root).replace(os.sep, "/")
        rust_file = workspace.file(relative)
        ranges = verus_ranges(rust_file)
        entries = []
        for name, unit in by_kernel.get(relative, []):
            mode = function_mode(rust_file, unit)
            body = rust_file.texts[unit.first_token:unit.last_token]
            if mode == "proof":
                kind, estimate = PROOF_FN, 1
            elif mode == "exec":
                kind, estimate = EXEC_FN, 1 + sum(1 for text in body if text in ("while", "loop", "for"))
            elif "decreases" in body and "spec-decreases-not-item" not in BREAK:
                kind, estimate = SPEC_FN_WITH_DECREASES, 1
            elif "spec-fn-as-item" in BREAK:
                kind, estimate = "spec fn", 0
            else:
                left_out.append((name, relative))
                continue
            entries.append((unit.first_token, ProofItem(proof_item(name), name, relative, unit, kind, f"{relative}::{name}", estimate)))
        if "derived-clone-not-item" not in BREAK:
            for unit in rust_file.units:
                if unit.kind not in ("struct", "enum", "union") or unit.first_token is None or unit.container is not None \
                        or not any(low < unit.first_token < high for low, high in ranges):
                    continue
                for trait in DERIVED_VERIFIED_TRAITS:
                    if trait in derived_traits(rust_file, unit):
                        verus_name = f"{unit.short}::{trait.lower()}"
                        entries.append((unit.first_token, ProofItem(proof_item(verus_name), verus_name, relative, unit, DERIVED_CLONE, f"{relative}::{unit.short}", 1)))
        items += [item for _position, item in sorted(entries, key=lambda pair: pair[0])]
    return items, left_out


def owner_function(functions, relative, line):
    """核 relative 第 line 行落在哪个 verus! 里的 fn 单元里，交 (名, 单元)；不在任何一个里交 (None, None)。"""
    for name, kernel, unit in functions:
        if kernel == relative and unit.first_line <= line <= unit.last_line:
            return name, unit
    return None, None


def shared_components(root, registry, options):
    """Verus 每一项都带的几样：工具链（tool-versions 那一行：Verus、它自带的 vstd、z3、rustup 工具链）、rlimit（登记的数，改变结论）。
    限时不进（超时不入库；弄坏开关 reuse-key-with-timeout 下进）；verus-pins.tsv 整份不进（登记表的注释、verified 的登记数与别的行不改任何一项的结论）。
    算一次存进 options；没给判定库时不算（交空表，lookup 也不看它）。"""
    if not options.get("verdict_library"):
        return []
    if options.get("shared_components") is None:
        components = []
        if "reuse-key-without-toolchain" not in BREAK:
            components.append(("工具链", digest_text(tool_versions_text(root, options.get("verus_root")))))
        components.append(("rlimit", digest_text(str(registry.rlimit))))
        if "reuse-key-with-timeout" in BREAK:
            components.append(("限时", digest_text(str(options["timeout"]))))
        options["shared_components"] = components
    return options["shared_components"]


def function_components(root, registry, options, entries, label):
    """被验函数那一样：entries（<核>::<fn 名>）的单元闭包键（UnitKey：那个 fn 单元，连同它 requires / ensures / decreases 与函数体里用到的
    spec fn、proof fn、类型、常量、broadcast use 点名的引理，lib_function_units.py 的八条闭包规则）。没给判定库交空表。
    弄坏开关：reuse-key-whole-crate 换回核 crate 整个目录的原文摘要（旧判法）；reuse-key-function-only 只取入口那几个单元自己的指纹；
    reuse-key-without-units 这一样不进。"""
    if not options.get("verdict_library") or "reuse-key-without-units" in BREAK:
        return []
    if "reuse-key-whole-crate" in BREAK:
        return [("核 crate", digest_tree(os.path.join(root, registry.crate_directory)))]
    try:
        key = unit_index(root, options).key(entries)
    except ValueError as error:
        raise Red(f"算不出 {'、'.join(entries)} 的单元闭包：{error}", "先让 python3 scripts/gate/lib_function_units.py closure <根> <入口> 在这棵树上跑通")
    if "reuse-key-function-only" in BREAK:
        names = {entry.rsplit("::", 1)[1] for entry in entries}
        own = tuple(line for line in key.units if line.split("\t")[1] in names)
        return [(label, UnitKey(digest_text("\n".join(line.split("\t")[0] + "\t" + line.split("\t")[3] for line in own)), own, ()))]
    return [(label, key)]


def verdict_library(options, kind, cell):
    version = judging_units(cell) if "reuse-key-without-judging-units" in BREAK else (options.get("judging_units") or judging_units(cell))
    store = VerdictLibrary(options.get("verdict_library"), kind, version, reuse=options.get("reuse", True))
    store.plan = collections.Counter() if options.get("plan") is not None else None
    return store


PLAN_STATES = ("键对不上", "无记录", "键相同", "拒算")


def peek(store, item, components):
    """只数键（list-items 给了判定库）：这一项照现在的键在库里是哪一种，不写库：键相同（会复用）、键对不上（这一项判绿过，键变了）、无记录、拒算。"""
    if refused_component(components, store.judging_units) is not None:
        return "拒算"
    if store.directory is None:
        return "无记录"
    if "key-mismatch-by-item-only" in BREAK:   # 弄坏开关：这一项判绿过就算键相同，不比键
        return "键相同" if read_json(store._item_path(item)) is not None else "无记录"
    if os.path.exists(store._key_path(key_of(components, store.judging_units))):
        return "键相同"
    return "键对不上" if read_json(store._item_path(item)) is not None else "无记录"


def plan_done(options, cell, store):
    """只数键的那一趟：这一格各项的键都比过了，记下各几条，不起 Verus。"""
    options["plan"].append((cell, store.plan))
    return EXIT_GREEN


def lookup(store, item, components):
    if getattr(store, "plan", None) is not None:
        store.plan[peek(store, item, components)] += 1
        return False, "只数键"
    if "reuse-ignores-key" in BREAK and store.directory is not None and store.reuse:
        return store.lookup_by_item_only(item)
    return store.lookup(item, components)


def judged_how(store, item, reason, sentinel=None):
    """现判那一行的括号：原因，连同判定库报的变了的单元（作废时点名是哪几个单元变了）；复用哨兵抽中强制现判的那一项写「强制现判」。"""
    if sentinel is not None and sentinel.forced(item):
        return f"{reuse_checks.FORCED}（复用哨兵抽中：键相同，这一趟不读库）"
    detail = store.detail(item)
    return f"现判（{reason}{'：' + detail if detail else ''}）"


def plan_sentinel(options, store, kind, cell, entries):
    """复用哨兵（scripts/gate/lib_reuse_checks.py 文件头「二」）：entries 是 [(项, 组成)]，候选是此刻键相同、库里有判绿记录的项
    （lib_reuse_checks.verdict_library_candidate 按键文件查，与 lookup 分开）；只数键的那一趟（list-items 给了库）不抽。"""
    sentinel = reuse_checks.ReuseSentinel(SCRIPT_KEY, cell)
    if options.get("plan") is not None:
        return sentinel
    reuse = store.directory is not None and store.reuse
    candidates = {item: "绿" for item, components in entries
                  if reuse and components and reuse_checks.verdict_library_candidate(options.get("verdict_library"), kind, components, store.judging_units)}
    sentinel.plan(candidates, reuse=reuse, off_reason=reuse_checks_off_reason(store))
    return sentinel


def reuse_checks_off_reason(store):
    return "没给判定库" if store.directory is None else "关了复用（--no-reuse 或 SINGLEFS_GATE_FULL=1）"


def sentinel_lookup(store, sentinel, item, components):
    """抽中强制现判的那一项不读库；别的照 lookup。弄坏开关 reuse-lookup-skipped 下一律不读库（复用哨兵该判红「复用失灵」）。"""
    if sentinel.forced(item):
        return False, "复用哨兵抽中强制现判"
    if "reuse-lookup-skipped" in BREAK and store.directory is not None and store.reuse:
        return False, "无记录（弄坏开关 reuse-lookup-skipped：没读库）"
    return lookup(store, item, components)


def selected_items(options, available, cell):
    """--item 点名的项（给几次取并集），按 available 的次序交回；没给交 available；点了没有的抛 UsageError。弄坏开关 item-filter-ignored 下一律交 available。"""
    wanted = options.get("items") or []
    if not wanted or "item-filter-ignored" in BREAK:
        return list(available)
    unknown = [item for item in wanted if item not in available]
    if unknown:
        raise UsageError(f"{cell} 格里没有点名的项：{'、'.join(unknown)}", available)
    return [item for item in available if item in wanted]


def report_item(item, verdict, how, seconds=None):
    """每项一行：项名、判定（绿 / 红）、现判还是复用、现判的用了几秒。"""
    print(f"  · {item}：{verdict}，{how}" + (f"，{seconds:.1f} 秒" if seconds is not None else ""))  # gate-lint:detail


def finish_tally(tally, cell):
    print(f"  · 按条：{tally.text()}")  # gate-lint:detail
    tally.append_to(os.environ.get(ITEM_TALLY_FILE_VARIABLE), cell)


def proof_item(name):
    return f"proof:{name}"


def results_text(results):
    """JSON 的 verification-results 照原样报几样：verified、errors、encountered-error、vir 错，有 success 键时另报它（--verify-function 下 Verus 不给）。"""
    def shown(value):
        return "没有这个键" if value is None else json.dumps(value)
    pieces = [f"verified {shown(results.get('verified'))}", f"errors {shown(results.get('errors'))}",
              f"encountered-error {shown(results.get('encountered-error'))}", f"vir 错 {shown(results.get('encountered-vir-error'))}"]
    if "success" in results:
        pieces.append(f"success {shown(results.get('success'))}")
    return "、".join(pieces)


def verification_clean(results, function_mode):
    """Verus 这一次验过了、一条错都没有：没有 vir 错、encountered-error 不是 true、errors 是整数 0，有 success 键时它是 true。
    --verify-function 下真 Verus（0.2026.09.20）不给 success 键、is-verifying-entire-crate 报 false（主 agent 2026-10-15 的探针：整 crate 那一趟
    给 success true 与 is-verifying-entire-crate true，逐函数那几趟只有 encountered-error、encountered-vir-error、verified、errors、is-verifying-entire-crate）：
    两样都不当成没验过。弄坏开关：require-success-key-per-function 下没有 success 键当没验过（改前的样子），
    require-entire-crate-per-function 下按函数验也要 is-verifying-entire-crate 是 true。"""
    if results.get("encountered-vir-error") or results.get("encountered-error"):
        return False
    errors = results.get("errors")
    if not isinstance(errors, int) or isinstance(errors, bool) or errors != 0:
        return False
    if "success" in results:
        if results["success"] is not True:
            return False
    elif "require-success-key-per-function" in BREAK:
        return False
    if function_mode and "require-entire-crate-per-function" in BREAK and results.get("is-verifying-entire-crate") is not True:
        return False
    return True


def item_kinds_text(items):
    """成项的几类各几个，次序固定：proof fn、exec fn、带 decreases 的 spec fn、derive(Clone) 生成的 clone（弄坏开关 spec-fn-as-item 下另有 spec fn）。"""
    kinds = collections.Counter(entry.kind for entry in items)
    return "、".join(f"{kind} {kinds[kind]}" for kind in (PROOF_FN, EXEC_FN, SPEC_FN_WITH_DECREASES, DERIVED_CLONE, "spec fn") if kinds[kind])


def listed_names(names, limit=20):
    """点名用：至多 limit 个名字，多的报个数。"""
    names = list(names)
    return "、".join(names[:limit]) + (f" 等 {len(names)} 个" if len(names) > limit else "") if names else "没有"


def check_proof(root, options):
    """证明逐项：有证明义务的单元各一项（proof_items：proof fn、exec fn、带 decreases 的 spec fn、derive(Clone) 的 clone；spec fn 不带 decreases 的不成项），
    各验一次（--verify-root --verify-function <名>）。每项判据：限时内出结局、没有 vir 错、errors 0（verification_clean：--verify-function 下不看 success 与
    is-verifying-entire-crate）、verified 至少 1（0 是 Verus 在点的函数上一条义务都没验：点错了函数，或它不该成项）、跑的 Verus 的 commit 等于登记。
    整格跑时各项 verified 之和（复用的取记下的数；按 --verify-function 点的名字计，同名的只算一次）要等于登记的数（verus-pins.tsv 的 crate 行，整 crate 验的数），
    不等判红并点名：verified 与静态估的义务条数不同的项、没成项的 verus! fn。--item proof 或不给 --item 是整格；--item proof:<名> 只验点名的，不比之和。"""
    registry, kernels, manifest, executable = prepared(root, options)
    functions = verus_functions(root, registry, kernels, options)
    items, left_out = proof_items(root, registry, kernels, functions, options)
    if not items:
        raise Red(f"{len(kernels)} 份核的 verus! 里一个有证明义务的单元都没有（verus! 里 {len(functions)} 个 fn，{len(left_out)} 个是不带 decreases 的 spec fn）：证明这一格没有项",
                  "核里的证明写在 verus! { … } 里的 proof fn 与 exec fn 上；核挪了地方就改 .claude/gate.d/verus-pins.tsv 的 kernel 行")
    available = [entry.item for entry in items]
    wanted = [item for item in (options.get("items") or []) if item != "proof"]
    whole = not wanted or "item-filter-ignored" in BREAK
    chosen = selected_items(dict(options, items=wanted), ["proof", *available], "verus-proof") if wanted else available
    selected = [entry for entry in items if entry.item in chosen]
    store, tally = verdict_library(options, "proof-function", "proof"), Tally()
    shared = shared_components(root, registry, options)
    problems = []
    if "cheating-unchecked" not in BREAK:
        for marker in cheating_markers(root, kernels):
            problems.append((f"核里有不许的标记 {marker}", "证明里不许 assume / admit / external_body / verifier::external：把它换成真证明，证不出就照实报，不绕过"))
        if "rlimit-attribute-unchecked" not in BREAK:
            for marker in rlimit_attributes(root, kernels):
                problems.append((f"核里有不许的标记 {marker}（函数上的 rlimit 属性盖过命令行的 --rlimit {registry.rlimit}，这个函数不在登记的额度上验）",
                                 f"删掉这个属性；证明确要更多额度，照 {REGISTRY} rlimit 行上面的注释在新数上把核与必红世界表整张重验一遍，再调高登记的数"))
    counts, pending = {}, []
    keyed = [(entry, shared + function_components(root, registry, options, [entry.entry], "被验函数单元闭包") if shared else []) for entry in selected]
    sentinel = plan_sentinel(options, store, "proof-function", "verus-proof", [(entry.item, components) for entry, components in keyed])
    for entry, components in keyed:
        reused, reason = sentinel_lookup(store, sentinel, entry.item, components)
        verified = ((store.record_of(entry.item) or {}).get("details") or {}).get("verified") if reused else None
        if reused and not str(verified).isdigit():
            reused, reason = False, "记录里没有 verified 数"
        tally.count(reused, reason)
        if reused:
            counts[entry.verus_name] = int(verified)
            report_item(entry.item, "绿", f"复用（{reason}）")
            sentinel.observe(entry.item, reuse_checks.REUSED, "绿")
        else:
            pending.append((entry, components, reason))
    if options.get("plan") is not None:
        return plan_done(options, "verus-proof", store)

    def judge_one(index, outcome):
        entry = pending[index][0]
        item = entry.item
        problem = outcome_problem(outcome, options)
        if problem:
            return [(f"{item}：{problem}", problem_howto(outcome, "看上面 Verus 的报错；超时先看是哪条查询慢（--profile），确要更长的时限设 GATE_VERUS_TIMEOUT_SECONDS 并写明为什么"))], "", None
        if outcome["timed_out"]:
            return [], "超时（弄坏开关 timeout-as-pass）", None
        results = outcome["json"]["verification-results"]
        commit = outcome["json"].get("verus", {}).get("commit")
        clean = verification_clean(results, True)
        if not clean and "nonzero-errors-accepted" in BREAK and not results.get("encountered-vir-error") and integer_counts(results):
            clean = True   # 弄坏开关：errors 不是 0 也当验过（只要不是 vir 错）
        if not clean:
            blocks = diagnostic_blocks(outcome["diagnostics"])
            tail = " | ".join(line for line in outcome.get("stderr", "").strip().split("\n")[-6:] if line.strip())
            listed = "；".join(f"{path}:{line} {message}" for message, path, line in blocks[:8]) or tail[-400:] or "stderr 里没有报错"
            return [(f"{item}：Verus 没验过（{entry.kind} {entry.verus_name}）：{results_text(results)}：{listed}",
                     "证明或实现有一处对不上：照报错的位置修；反例是真的就先修实现")], "", None
        if not integer_counts(results):
            return [(f"{item}：Verus 的 JSON 里没有整数的 verified 与 errors（读到 verified {results.get('verified')!r}），各项之和比不了",
                     "Verus 换了 JSON 的写法：照新写法改 scripts/gate/verus-proof-check.py 的 integer_counts")], "", None
        if commit != registry.commit:
            return [(f"{item}：跑的 Verus 报的 commit 是 {commit}，登记的是 {registry.commit}", "SINGLEFS_VERUS_ROOT 指到登记的那份安装")], "", None
        if results["verified"] < 1 and "zero-verified-accepted" not in BREAK:
            return [(f"{item}：Verus 在 --verify-function {entry.verus_name} 上一条义务都没验（{results_text(results)}；它是 {entry.kind}，静态估 {entry.estimate} 条）",
                     "点到的不是要验的那个函数，或它本没有证明义务：看 scripts/gate/verus-proof-check.py 的 proof_items 怎么认它这一类"
                     "（不带 decreases 的 spec fn 不成项），名字与别的函数撞了就改名")], "", None
        counts[entry.verus_name] = results["verified"]
        return [], f"{results['verified']} verified、0 errors", {"verified": str(results["verified"])}

    judging = ItemJudging(store, judge_one, "verus-proof", f"verus --verify-root --verify-function <名> --no-cheating --rlimit {registry.rlimit}（{registry.tag}）",
                          sentinel=sentinel)
    started = time.monotonic()
    verify_in_parallel(root, registry, manifest, executable, options, [(entry.item, {}, entry.verus_name) for entry, _components, _reason in pending], "proof",
                       judged=lambda index, outcome, seconds: judging(index, outcome, seconds, pending[index][0].item, pending[index][1],
                                                                      judged_how(store, pending[index][0].item, pending[index][2], sentinel)))
    problems += judging.finish(len(pending))
    problems += sentinel.finish()
    total = sum(counts.values())
    breakdown = item_kinds_text(selected)
    if whole and not problems and total != registry.verified_count and "count-unchecked" not in BREAK:
        off = [f"{entry.verus_name} 验出 {counts[entry.verus_name]}、静态估 {entry.estimate}" for entry in selected
               if entry.verus_name in counts and counts[entry.verus_name] != entry.estimate]
        problems.append((f"各项 verified 之和 {total} 与登记的 {registry.verified_count} 不同（{'少' if total < registry.verified_count else '多'} "
                         f"{abs(registry.verified_count - total)} 条；成项 {len(selected)} 个：{breakdown}，静态估之和 {sum(entry.estimate for entry in selected)}）："
                         f"verified 与静态估不同的项：{listed_names(off)}；没成项的 verus! fn（不带 decreases 的 spec fn，静态估 0 条）{len(left_out)} 个："
                         f"{listed_names(name for name, _relative in left_out)}",
                         "少了是整 crate 验出的义务有一部分不落在成项的单元上：在副本上整 crate 验一次（不带 --verify-function），再逐个 --verify-function 没成项的那几个，"
                         "验出非 0 的那一类补进 scripts/gate/verus-proof-check.py 的 proof_items；多了先看 verified 与静态估不同的那几项；"
                         f"确是加减了证明就照整 crate 验出的数改 {REGISTRY} 的 crate 行，并在改动说明里写加减了哪几个"))
    finish_tally(tally, "verus-proof")
    if problems:
        report_cell(problems)
        return EXIT_RED
    scope = (f"--item 点名的 {len(selected)} 项（不比之和）" if not whole
             else f"成项 {len(selected)} 个（{breakdown}），各项 verified 之和 {total}、0 errors（登记 {registry.verified_count}）；"
                  f"不成项的 spec fn {len(left_out)} 个（没有证明义务，随用到它们的项进键）")
    print(f"  ✓ Verus 逐项验 {registry.crate_directory}：{scope}，--no-cheating，--rlimit {registry.rlimit}，"
          f"{len(kernels)} 份核里 0 处作弊标记，限时 {options['timeout']} 秒内出结局；按条{tally.text()}（{time.monotonic() - started:.1f} 秒）")
    return EXIT_GREEN


def requires_insertion_points(text):
    """每一处 requires 的函数体头：[(requires 行号, { 所在行的下标, 说明)]；认不出函数体的抛 Red。"""
    lines = text.split("\n")
    points = []
    for index, line in enumerate(lines):
        stripped = strip_line_comment(line).strip()
        if stripped != "requires" and not stripped.startswith("requires "):
            continue
        indent = len(line) - len(line.lstrip())
        brace = None
        for later in range(index + 1, len(lines)):
            code = strip_line_comment(lines[later])
            if FUNCTION_LINE.match(code):
                break
            if code.strip().startswith("{") and len(code) - len(code.lstrip()) <= indent:
                brace = later
                break
        if brace is None:
            raise Red(f"第 {index + 1} 行的 requires 找不到函数体的 {{（没有函数体，或 {{ 没单独起行）",
                      "把函数体的 { 单独起一行（verusfmt 的写法）；确是没有函数体的声明，把判法补进 scripts/gate/verus-proof-check.py 的 requires_insertion_points")
        owner = next((FUNCTION_LINE.match(strip_line_comment(lines[back])).group(5) for back in range(index, -1, -1)
                      if FUNCTION_LINE.match(strip_line_comment(lines[back]))), "?")
        inline = lines[brace].split("{", 1)[1].strip()
        label = f"{owner} 里第 {index + 1} 行起的闭包" if inline else f"fn {owner}"
        points.append((index + 1, brace, label))
    return points


def verify_in_parallel(root, registry, manifest, executable, options, edits, key, judged=None):
    """edits：[(说明, {相对仓根的文件: 新内容}[, 只验的函数名])]；每一处一份副本、一次 Verus（给了函数名就 --verify-root --verify-function 只验它）。交回与 edits 同序的 outcome。
    judged(下标, outcome, 秒数) 给了就在那一处出结局时当场调（按条判定在这里判完就落盘，打断之后判过的不丢）。
    一处拿到内存包装的 252 之后，还没起的那几处不建副本、不起包装（QueueRefusal），交 skipped_outcome；已经起了的照常收。"""
    if not edits:
        return []
    jobs = options["jobs"] or min(len(edits), max(1, options["threads"] // 4))
    threads = max(1, options["threads"] // jobs)
    state = queue_refusal(options)

    def one(index, edit):
        started = time.monotonic()
        earlier = state.refused() if state is not None else None
        if earlier:
            outcome = skipped_outcome(earlier)
        else:
            with tempfile.TemporaryDirectory(prefix="verus-proof-check-") as work:
                crate_copy = copy_crate(root, registry, work)
                for relative, content in edit[1].items():
                    with open(os.path.join(work, relative), "w", encoding="utf-8") as handle:
                        handle.write(content)
                outcome = run_verus(executable, crate_copy, verus_arguments(registry, manifest, threads, edit[2] if len(edit) > 2 else None), options, key, label=edit[0])
        if judged is not None:
            judged(index, outcome, time.monotonic() - started)
        return outcome

    with concurrent.futures.ThreadPoolExecutor(max_workers=jobs) as pool:
        futures = [pool.submit(one, index, edit) for index, edit in enumerate(edits)]
        return [future.result() for future in futures]


class ItemJudging:
    """一格里逐项判：每项出结局时（verify_in_parallel 的 judged）判出这一项的问题，判绿就当场记进判定库；问题按项的次序交回。
    弄坏开关 reuse-records-whole-cell-only 下不当场记，等整格判完、一处问题都没有才一起记（被打断或有一项红，判过的全不留）；
    reuse-records-red 下红的也记成绿。"""

    def __init__(self, store, judge_one, cell, run="", sentinel=None):
        self.store, self.judge_one, self.cell, self.run, self.sentinel = store, judge_one, cell, run, sentinel
        self.lock = threading.Lock()
        self.problems, self.lines, self.summaries, self.pending = {}, {}, {}, []

    def __call__(self, index, outcome, seconds, item, components, how):
        """how 是这一项那一行的「现判（原因…）」整句（judged_how）；judge_one 交 (问题, 一句判定[, 随判定存的数])。"""
        judged = self.judge_one(index, outcome)
        problems, summary = judged[0], judged[1]
        details = judged[2] if len(judged) > 2 else None
        failed_record = None
        if not problems or "reuse-records-red" in BREAK:
            if "reuse-records-whole-cell-only" in BREAK:
                with self.lock:
                    self.pending.append((item, components, summary, details))
            else:
                failed_record = self.store.record(item, components, summary, run=self.run, details=details)
        with self.lock:
            self.problems[index] = problems
            self.summaries[index] = summary
            self.lines[index] = (item, "红" if problems else "绿", how, seconds, failed_record)

    def finish(self, count):
        """按项的次序打每项那一行、交回全部问题；whole-cell-only 下整格没问题才把攒着的一起记。"""
        problems = []
        for index in range(count):
            if index in self.lines:
                item, verdict, how, seconds, failed_record = self.lines[index]
                report_item(item, verdict, how, seconds)
                if self.sentinel is not None:
                    self.sentinel.observe(item, reuse_checks.FORCED if self.sentinel.forced(item) else reuse_checks.JUDGED, verdict)
                if failed_record:
                    print(f"  ! {failed_record}：下一趟这一项照判")
            problems += self.problems.get(index, [])
        if not problems:
            for item, components, summary, details in self.pending:
                self.store.record(item, components, summary, run=self.run, details=details)
        return problems


def with_assert_false(line):
    head, tail = line.split("{", 1)
    return head + "{ assert(false);" + tail


def vacuity_points(root, registry, kernels, functions):
    """空真的每一处：[(项名, 说明, {相对仓根的文件: 插了断言的内容}, (相对 crate 的文件, 插的那一行, requires 那一行), 所属函数名, 函数内第几处)]
    与每份核全部插进去的那一份（弄坏开关用）。项名 vacuity:<所属函数名>#<函数内第几处 requires>，只当索引：这一处的身份是所属函数单元与第几处，
    不带路径与行号（空行让 requires 挪一行，项名与键都不变）。functions 是 verus_functions 交的；requires 不在其中任何一个 fn 里判红。"""
    points, all_at_once = [], []
    for kernel in kernels:
        text = open(kernel, encoding="utf-8").read()
        relative = os.path.relpath(kernel, root).replace(os.sep, "/")
        lines = text.split("\n")
        every_point = list(lines)
        ordinals = {}
        for requires_line, brace, label in requires_insertion_points(text):
            owner, _unit = owner_function(functions, relative, requires_line)
            if owner is None:
                raise Red(f"{relative} 第 {requires_line} 行的 requires 不在 verus! 里的任何 fn 里：认不出它属于哪个函数",
                          "requires 只写在 verus! 里的 fn 与它里面的闭包上；确是新写法，把判法补进 scripts/gate/verus-proof-check.py 的 vacuity_points")
            ordinals[owner] = ordinals.get(owner, 0) + 1
            changed = list(lines)
            changed[brace] = with_assert_false(changed[brace])
            every_point[brace] = with_assert_false(every_point[brace])
            points.append((f"vacuity:{owner}#{ordinals[owner]}", f"{relative} {label}", {relative: "\n".join(changed)},
                           (os.path.relpath(kernel, os.path.join(root, registry.crate_directory)), brace + 1, requires_line), owner, ordinals[owner]))
        all_at_once.append((relative, {relative: "\n".join(every_point)}))
    return points, all_at_once


def vacuity_item_problems(label, expected, outcome, options):
    """空真一处的问题（(卡在哪, 下一步)），交回 (问题, 一句判定)。"""
    path, line, requires_line = expected
    problem = outcome_problem(outcome, options)
    if problem:
        return [(f"{label}（第 {requires_line} 行的 requires）：{problem}",
                 problem_howto(outcome, "插断言的副本判不出来：看报错，修好再跑；超时见 GATE_VERUS_TIMEOUT_SECONDS"))], ""
    if outcome["timed_out"]:
        return [], "超时（弄坏开关 timeout-as-pass）"
    results = outcome["json"]["verification-results"]
    if verification_clean(results, True):
        if "vacuity-accepts-verified" not in BREAK:
            return [(f"{label}：函数体头插 assert(false) 照样验得过——第 {requires_line} 行起的前置条件不可满足（空真），这个函数的 ensures 什么也没证",
                     "改前置条件让它可满足（常见：两条互相矛盾、范围写反）；改完这一格插的 assert(false) 要在那一行报 assertion failed")], ""
        return [], "验得过（弄坏开关 vacuity-accepts-verified）"
    blocks = diagnostic_blocks(outcome["diagnostics"])
    if not any("assertion failed" in message and path == block_path and line == block_line for message, block_path, block_line in blocks):
        if "vacuity-all-at-once" in BREAK and "闭包" in label:
            return [], ""
        listed = "；".join(f"{block_path}:{block_line} {message}" for message, block_path, block_line in blocks[:6])
        return [(f"{label}：插的 assert(false) 在 {path}:{line}，Verus 报的错不在那一行（{listed or '没有配上位置的报错'}）",
                 "插入点认错了或副本编不过：看 requires_insertion_points 认的 {{ 是不是函数体的那一个")], ""
    return [], f"{path}:{line} 报 assertion failed"


def check_vacuity(root, options):
    """空真每处 requires 一项：插了 assert(false) 的副本上只验它所属的那一个函数（--verify-root --verify-function），键是所属函数的单元闭包与函数内第几处。"""
    registry, kernels, manifest, executable = prepared(root, options)
    functions = verus_functions(root, registry, kernels, options)
    points, all_at_once = vacuity_points(root, registry, kernels, functions)
    if not points:
        raise NotRun(f"{len(kernels)} 份核里一处 requires 都没有：没有前置条件，空真这一格无对象可判")
    chosen = selected_items(options, [point[0] for point in points], "verus-vacuity")
    points = [point for point in points if point[0] in chosen]
    store, tally = verdict_library(options, "vacuity-requires", "vacuity"), Tally()
    shared = shared_components(root, registry, options)
    pending, keyed = [], []
    for item, label, change, expected, owner, ordinal in points:
        components = []
        if shared:
            relative = label.split(" ", 1)[0]
            components = shared + function_components(root, registry, options, [f"{relative}::{owner}"], "所属函数单元闭包")
            if "reuse-key-without-row" not in BREAK:
                components.append(("函数内第几处", digest_text(str(ordinal))))
            if "vacuity-key-by-line-number" in BREAK:
                components.append(("这一处", digest_text(f"{relative}:{expected[2]}")))
        keyed.append(components)
    sentinel = plan_sentinel(options, store, "vacuity-requires", "verus-vacuity", [(point[0], components) for point, components in zip(points, keyed)])
    for (item, label, change, expected, owner, ordinal), components in zip(points, keyed):
        reused, reason = sentinel_lookup(store, sentinel, item, components)
        tally.count(reused, reason)
        if reused:
            report_item(item, "绿", f"复用（{reason}）")
            sentinel.observe(item, reuse_checks.REUSED, "绿")
        else:
            pending.append((item, label, change, expected, components, reason, owner))
    if options.get("plan") is not None:
        return plan_done(options, "verus-vacuity", store)
    if "vacuity-all-at-once" in BREAK:
        # 弄坏：一份核的全部插入点插进同一份副本只跑一遍、验整个 crate。外层函数入口的 assert(false) 一失败，里面闭包那一处就验得过、不单报
        together = dict(zip((relative for relative, _change in all_at_once),
                            verify_in_parallel(root, registry, manifest, executable, options, all_at_once, "vacuity")))
        problems = []
        for item, label, _change, expected, _components, reason, _owner in pending:
            item_problems, _summary = vacuity_item_problems(label, expected, together[label.split(" ", 1)[0]], options)
            report_item(item, "红" if item_problems else "绿", f"现判（{reason}）")
            problems += item_problems
    else:
        judging = ItemJudging(store, lambda index, outcome: vacuity_item_problems(pending[index][1], pending[index][3], outcome, options), "verus-vacuity",
                              f"verus --verify-root --verify-function <所属函数> --no-cheating --rlimit {registry.rlimit}（{registry.tag}），函数体头插 assert(false)",
                              sentinel=sentinel)
        verify_in_parallel(root, registry, manifest, executable, options, [(label, change, owner) for _item, label, change, _expected, _components, _reason, owner in pending],
                           "vacuity", judged=lambda index, outcome, seconds: judging(index, outcome, seconds, pending[index][0], pending[index][4],
                                                                                     judged_how(store, pending[index][0], pending[index][5], sentinel)))
        problems = judging.finish(len(pending))
        problems += sentinel.finish()
    finish_tally(tally, "verus-vacuity")
    if problems:
        report_cell(problems)
        return EXIT_RED
    named = f"（--item 点名的 {len(points)} 处）" if options.get("items") else ""
    print(f"  ✓ 空真：{len(kernels)} 份核里 {len(points)} 处 requires{named}，每一处在函数体头插 assert(false)、只验它所属的函数，都在插的那一行报 assertion failed（前置条件可满足）；"
          f"按条{tally.text()}")
    return EXIT_GREEN


def must_red_rows(root, registry):
    path = os.path.join(root, registry.must_red_table)
    if not os.path.isfile(path):
        raise Red(f"必红世界表 {registry.must_red_table} 不在", "核进了仓就要带它（五段制表符分隔：变异名、文件、原文、替换文、proof_errors=<证明错条数>）；路径改了就改登记表的 must-red-worlds 行")
    rows, problems = [], []
    for number, raw in enumerate(open(path, encoding="utf-8").read().split("\n"), 1):
        if not raw.strip() or raw.startswith("#"):
            continue
        fields = raw.split("\t")
        if len(fields) != 5 or not all(fields[index] for index in (0, 1, 2, 4)):
            problems.append(f"第 {number} 行不是五段（或变异名、文件、原文、第五段 proof_errors= 有空的）")
            continue
        rows.append({"number": number, "name": fields[0], "file": fields[1], "original": fields[2].replace("\\n", "\n"),
                     "replacement": fields[3].replace("\\n", "\n"), "expected": fields[4]})
    if problems:
        raise Red(f"{registry.must_red_table} 写坏了：" + "；".join(problems), "照表头的格式改")
    if not rows:
        raise Red(f"{registry.must_red_table} 一行都没有：核没有一个必须报红的世界", "每一层证明至少一条改回去 Verus 就红的变异（show-me-test.md「设计论证同样照办」）")
    return rows


def must_red_item(row):
    return f"must-red:{row['name']}"


def function_keys(root, options, functions):
    """verus! 里每个 fn 的单元闭包键（{名: UnitKey}），一格里算一次存进 options。"""
    if options.get("function_keys") is None:
        index = unit_index(root, options)
        keys = {}
        try:
            for name, relative, _unit in functions:
                keys[name] = index.key([f"{relative}::{name}"])
        except ValueError as error:
            raise Red(f"算不出核里函数的单元闭包：{error}", "先让 python3 scripts/gate/lib_function_units.py closure <根> <入口> 在这棵树上跑通")
        options["function_keys"] = keys
    return options["function_keys"]


def must_red_components(root, registry, options, row, text, functions):
    """必红世界一行的键：这一行改坏的单元（原文所在的那几个单元）与 verus! 里闭包用到它们的每个 fn（改坏之后可能报错的就是这几个），
    这些入口并起来的单元闭包键；加原文、替换文与第五段（变异名与文件路径不进：改名、挪表不重判）。没给判定库交空表。
    弄坏开关：must-red-key-with-name-and-path 下变异名与文件进「这一行」；reuse-key-without-row 下「这一行」不进；reuse-key-whole-crate 下单元那一样换回核 crate 整个目录。"""
    if not options.get("verdict_library"):
        return []
    shared = shared_components(root, registry, options)
    pieces = [row["original"], row["replacement"], row["expected"]]
    if "must-red-key-with-name-and-path" in BREAK:
        pieces = [row["name"], row["file"]] + pieces
    row_component = [] if "reuse-key-without-row" in BREAK else [("这一行", digest_text("\t".join(pieces)))]
    if "reuse-key-whole-crate" in BREAK:
        return shared + [("核 crate", digest_tree(os.path.join(root, registry.crate_directory)))] + row_component
    index = unit_index(root, options)
    position = text.index(row["original"])
    first_line = text.count("\n", 0, position) + 1
    last_line = first_line + row["original"].count("\n")
    try:
        rust_file = index.workspace().file(row["file"])
    except (OSError, ValueError) as error:
        raise Red(f"第 {row['number']} 行「{row['name']}」的文件 {row['file']} 切不出单元：{error}", "先让 python3 scripts/gate/lib_function_units.py dump <根> --language rust 在这棵树上跑通")
    mutated = [unit for unit in rust_file.units if unit.first_line <= last_line and unit.last_line >= first_line and not unit.data]
    identities = {(unit.kind, unit.name, unit.file) for unit in mutated}
    keys = function_keys(root, options, functions)
    affected = []
    for name, relative, _unit in functions:
        members = {(line.split("\t")[0], line.split("\t")[1], line.split("\t")[2].rsplit(":", 1)[0]) for line in keys[name].units}
        if identities & members:
            affected.append(f"{relative}::{name}")
    entries = affected + [f"{row['file']}::{unit.name}" for unit in mutated if f"{row['file']}::{unit.name}" not in affected]
    if not entries:
        return shared + [("受影响的函数单元", digest_text("原文不在任何单元里"))] + row_component
    try:
        units = index.key(entries)
    except ValueError as error:
        raise Red(f"第 {row['number']} 行「{row['name']}」算不出受影响单元的闭包：{error}", "原文所在的单元名字不唯一时让原文只落在一个 fn 里")
    return shared + [("受影响的函数单元", units)] + row_component


def must_red_row_problems(row, outcome, options, registry):
    """必红世界一行改坏之后的结局判出的问题，交回 (问题, 一句判定)；一句判定里带「rlimit 用尽夹证明错」时成功句把它数进去。"""
    problem = outcome_problem(outcome, options)
    if problem:
        return [(f"第 {row['number']} 行「{row['name']}」：{problem}",
                 problem_howto(outcome, "必红世界要红在证明上：改坏之后编不过的变异不算，换一处只改判定的改法"))], ""
    if outcome["timed_out"]:
        return [], "超时（弄坏开关 timeout-as-pass）"
    results = outcome["json"]["verification-results"]
    if not integer_counts(results):
        return [(f"第 {row['number']} 行「{row['name']}」：Verus 的 JSON 里没有整数的 verified 与 errors（读到 verified {results.get('verified')!r}、"
                 f"errors {results.get('errors')!r}、vir 错 {results.get('encountered-vir-error')}），判不出这一行红没红",
                 "vir 错是改坏的副本没走到验证（类型或模式错）：换一处只改判定的改法；没有 vir 错就是 Verus 换了 JSON 的写法："
                 "照新写法改 scripts/gate/verus-proof-check.py 的 integer_counts")], ""
    rlimit_exhausted = RLIMIT_EXHAUSTED in outcome["diagnostics"]
    proof_errors = proof_error_messages(outcome["diagnostics"])
    counted = proof_error_messages(outcome["diagnostics"], count_rlimit="rlimit-counted-in-proof-errors" in BREAK)
    if rlimit_exhausted and (not proof_errors or "rlimit-any-line-voids" in BREAK) and "rlimit-counts-as-red" not in BREAK:
        listed = "；".join(f"{path}:{line} {message}" for message, path, line in diagnostic_blocks(outcome["diagnostics"])[:6])
        return [(f"第 {row['number']} 行「{row['name']}」：Verus 输出里有 {RLIMIT_EXHAUSTED}，除它与 aborting due to 之外没有别的报错（--rlimit {registry.rlimit}；"
                 f"verified {results.get('verified')}、errors {results.get('errors')}：{listed}），这一行的红不作数",
                 f"这一行红在算力额度上，不是证明失败：改替换文或调高登记的 rlimit（{REGISTRY} 的 rlimit 行）")], ""
    summary = f"证明错 {len(counted)} 条" + ("，rlimit 用尽夹证明错" if rlimit_exhausted else "")
    if results.get("errors", 0) < 1 and "mutation-accepts-green" not in BREAK:
        return [(f"第 {row['number']} 行「{row['name']}」改坏之后 Verus 照样验过（verified {results.get('verified')}、errors 0）：证明没盯住这一处",
                 "补证明或收紧 spec，让这一处改坏就验不过；确是等价变异就从表里删掉并写明为什么")], summary
    if (row["proof_errors"] is not None and len(counted) != row["proof_errors"]
            and "proof-error-count-unchecked" not in BREAK and "mutation-accepts-green" not in BREAK):
        listed = "；".join(f"{path}:{line} {message}" for message, path, line in diagnostic_blocks(outcome["diagnostics"])[:6])
        return [(f"第 {row['number']} 行「{row['name']}」红了，但证明错 {len(counted)} 条，表里写的是 proof_errors={row['proof_errors']}"
                 f"（--rlimit {registry.rlimit}；verified {results.get('verified')}、errors {results.get('errors')}；报的是：{listed}）",
                 "先看红的是不是这条变异要盯的那一处（证明换了一条路红的就改证明或换改法）；确是它，照这一次的证明错条数改第五段。"
                 "条数只数 error 起头的证明错，含 Resource limit (rlimit) exceeded 的与 aborting due to 那一句不计")], summary
    return [], summary


def check_must_red_worlds(root, options):
    registry, kernels, manifest, executable = prepared(root, options)
    functions = verus_functions(root, registry, kernels, options)
    rows = must_red_rows(root, registry)
    chosen = selected_items(options, [must_red_item(row) for row in rows], "verus-must-red-worlds")
    rows = [row for row in rows if must_red_item(row) in chosen]
    store, tally = verdict_library(options, "must-red-world-row", "must-red-worlds"), Tally()
    problems, edits, judged, texts = [], [], [], {}
    rlimit_with_proof_error = 0
    for row in rows:
        target = os.path.join(root, row["file"])
        if not row["file"].startswith(registry.crate_directory + "/") or not os.path.isfile(target):
            problems.append((f"第 {row['number']} 行「{row['name']}」的文件 {row['file']} 不在登记的 crate 里或不存在", "变异只许改被验的那个 crate 里的文件"))
            continue
        form = EXPECTED_PROOF_ERRORS.fullmatch(row["expected"])
        row["proof_errors"] = int(form.group(1)) if form else None
        if form is None and "expectation-form-unchecked" not in BREAK:
            problems.append((f"第 {row['number']} 行「{row['name']}」的第五段「{row['expected']}」不是 proof_errors=<证明错条数>（没起 Verus）", EXPECTATION_HOWTO))
            continue
        attributes = [match.group(0) for match in (RLIMIT_ATTRIBUTE.search(strip_line_comment(line)) for line in row["replacement"].split("\n")) if match]
        if attributes and "mutation-rlimit-attribute-accepted" not in BREAK:
            problems.append((f"第 {row['number']} 行「{row['name']}」的替换文里有 {attributes[0]}：函数上的 rlimit 属性盖过命令行的 --rlimit {registry.rlimit}，改坏的副本不在登记的额度上验",
                             "删掉替换文里的 rlimit 属性；这一行要更多额度才红得出来，就换一处改法（必红世界要在登记的 rlimit 上红）"))
            continue
        text = open(target, encoding="utf-8").read()
        hits = text.count(row["original"])
        if hits != 1 and "anchor-unchecked" not in BREAK:
            problems.append((f"第 {row['number']} 行「{row['name']}」的原文在 {row['file']} 里命中 {hits} 次，要恰好命中一次（锚点腐化）",
                             "照今天的代码改这一行的原文，让它只命中要改坏的那一处"))
            continue
        edits.append((row["name"], {row["file"]: text.replace(row["original"], row["replacement"], 1)}))
        judged.append(row)
        texts[row["number"]] = text
    # 原文、第五段、rlimit 属性没过的行没起 Verus，按项各打一行红
    passed = {row["number"] for row in judged}
    for row in rows:
        if row["number"] not in passed:
            report_item(must_red_item(row), "红", "现判（没起 Verus：这一行本身不对）")
            tally.count(False, "这一行本身不对")
    pending = []
    keyed = [must_red_components(root, registry, options, row, texts[row["number"]], functions) for row in judged]
    sentinel = plan_sentinel(options, store, "must-red-world-row", "verus-must-red-worlds",
                             [(must_red_item(row), components) for row, components in zip(judged, keyed)])
    for row, edit, components in zip(judged, edits, keyed):
        item = must_red_item(row)
        reused, reason = sentinel_lookup(store, sentinel, item, components)
        tally.count(reused, reason)
        if reused:
            report_item(item, "绿", f"复用（{reason}）")
            sentinel.observe(item, reuse_checks.REUSED, "绿")
        else:
            pending.append((row, edit, item, components, reason))
    if options.get("plan") is not None:
        return plan_done(options, "verus-must-red-worlds", store)
    judging = ItemJudging(store, lambda index, outcome: must_red_row_problems(pending[index][0], outcome, options, registry), "verus-must-red-worlds",
                          f"verus --no-cheating --rlimit {registry.rlimit}（{registry.tag}），整个 crate、原文换成替换文的副本", sentinel=sentinel)
    verify_in_parallel(root, registry, manifest, executable, options, [edit for _row, edit, _item, _components, _reason in pending], "must-red-worlds",
                       judged=lambda index, outcome, seconds: judging(index, outcome, seconds, pending[index][2], pending[index][3],
                                                                      judged_how(store, pending[index][2], pending[index][4], sentinel)))
    problems += judging.finish(len(pending))
    problems += sentinel.finish()
    rlimit_with_proof_error = sum(1 for summary in judging.summaries.values() if "rlimit 用尽夹证明错" in summary)
    finish_tally(tally, "verus-must-red-worlds")
    if problems:
        report_cell(problems)
        return EXIT_RED
    named = f"（--item 点名的 {len(rows)} 行）" if options.get("items") else ""
    print(f"  ✓ 必红世界：{registry.must_red_table} 的 {len(rows)} 行{named}每行改坏之后 Verus 都报错，证明错条数与第五段 proof_errors= 相同；原文都恰好命中一次；"
          f"没有一行只红在 rlimit 用尽上（--rlimit {registry.rlimit}；{rlimit_with_proof_error} 行输出里夹着 rlimit 用尽、另有证明错，红计入，rlimit 用尽不计入证明错条数）；"
          f"按条{tally.text()}")
    return EXIT_GREEN


# ── 指纹与工具版本 ──

def package_source_fingerprint(directory):
    entries = []
    for current, subdirectories, names in os.walk(directory):
        subdirectories.sort()
        for name in names:
            full = os.path.join(current, name)
            relative = os.path.relpath(full, directory)
            if relative in EXCLUDED_PACKAGE_FILES:
                continue
            content = ("symlink:" + os.readlink(full)).encode() if os.path.islink(full) else open(full, "rb").read()
            entries.append((relative.encode("utf-8", "surrogateescape"), hashlib.sha256(content).hexdigest()))
    entries.sort()
    text = b"".join(digest.encode() + b"  " + relative + b"\n" for relative, digest in entries)
    return hashlib.sha256(text).hexdigest(), len(entries)


def streamed_sha256(path):
    """文件内容的 sha256，按 1 MiB 一块读（工具链里的 libLLVM 约 200 MB，不整份读进内存）。"""
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def rustup_toolchain_fields(toolchain_text):
    """tool-versions 里 rustup 那几段（前面带空格）：
    RUSTUP_HOME=<未设|空串|原值>:<解析到的真实路径>（没设或空串时照 rustup 取 ~/.rustup）；
    rustup_toolchain_lib_sha256=<摘要>:<文件数>，工具链目录 <RUSTUP_HOME>/toolchains/<version.json 的 toolchain 第一个词> 下
      lib/ 顶层全部普通文件与 lib/rustlib/*/lib/ 下全部普通文件（逐层往下），逐个「相对工具链目录的路径 NUL 它的 sha256 换行」按路径字节序接起来再取 sha256；
      version.json 里没有工具链名写「不知道工具链名」，目录不在写「没有」；
    rustup=<PATH 里找到的那一份>:<它的真实路径>:<sha256>（launcher 照 PATH 找 rustup），找不到写「找不到」。"""
    value = os.environ.get("RUSTUP_HOME")
    shown = "未设" if value is None else "空串" if value == "" else value
    home = os.path.realpath(os.path.abspath(value) if value else RUSTUP_DEFAULT_HOME)
    fields = f" RUSTUP_HOME={shown}:{home}"
    name = toolchain_text.split(" ")[0]
    if not name:
        fields += " rustup_toolchain_lib_sha256=不知道工具链名"
    else:
        toolchain = os.path.join(home, "toolchains", name)
        if not os.path.isdir(toolchain):
            fields += " rustup_toolchain_lib_sha256=没有"
        else:
            relatives = []
            library = os.path.join(toolchain, "lib")
            if os.path.isdir(library):
                relatives += [os.path.join("lib", entry) for entry in os.listdir(library) if os.path.isfile(os.path.join(library, entry))]
            for target_library in glob.glob(os.path.join(library, "rustlib", "*", "lib")):
                for current, _subdirectories, names in os.walk(target_library):
                    relatives += [os.path.relpath(os.path.join(current, entry), toolchain) for entry in names if os.path.isfile(os.path.join(current, entry))]
            relatives.sort(key=lambda relative: relative.encode("utf-8", "surrogateescape"))
            every = hashlib.sha256()
            for relative in relatives:
                every.update(relative.encode("utf-8", "surrogateescape") + b"\0" + streamed_sha256(os.path.join(toolchain, relative)).encode() + b"\n")
            fields += f" rustup_toolchain_lib_sha256={every.hexdigest()}:{len(relatives)}"
    located = shutil.which("rustup")
    fields += f" rustup={located}:{os.path.realpath(located)}:{streamed_sha256(located)}" if located else " rustup=找不到"
    return fields


def tool_versions(root, override):
    print(tool_versions_text(root, override))
    return EXIT_GREEN


def tool_versions_text(root, override):
    """tool-versions 打的那一行（不带换行）；按条判定的「工具链」一样就是它的摘要。"""
    try:
        registry = read_registry(root)
    except Red:
        registry = Registry()
        registry.install_relative = "singlefs/formal-tools/verus"
    directory = verus_directory(registry, override)
    version = read_version_json(directory)

    def file_sha256(path):
        return streamed_sha256(path) if os.path.isfile(path) else "没有"

    def top_level_sha256(path):
        """目录顶层全部普通文件：逐个「文件名 NUL 它的 sha256 换行」按文件名字节序接起来再取 sha256；目录不在时是空串的 sha256。"""
        every = hashlib.sha256()
        if os.path.isdir(path):
            for name in sorted(os.listdir(path), key=lambda entry: entry.encode("utf-8", "surrogateescape")):
                full = os.path.join(path, name)
                if os.path.isfile(full):
                    every.update(name.encode("utf-8", "surrogateescape") + b"\0" + file_sha256(full).encode() + b"\n")
        return every.hexdigest()

    digest = file_sha256(os.path.join(directory, "verus"))
    extra = ""
    if "tool-versions-launcher-only" not in BREAK:
        extra = "".join(f" {name}_sha256={file_sha256(os.path.join(directory, name))}" for name in TOOL_VERSION_FILES)
        if "tool-versions-five-files-only" not in BREAK:
            # 安装目录顶层全部普通文件：Verus 还读五样之外的（宏库、libverus_builtin.rlib），换了任何一份这一行都变
            extra += f" install_top_level_sha256={top_level_sha256(directory)}"
        if "tool-versions-verus-environment-ignored" not in BREAK:
            # rust_verify 读的 VERUS_ROOT（宏库、vstd 从那里读）与 launcher 读的 VERUS_USE_RUSTUP（0 时不经 rustup 起 rust_verify）：没设与空串分开
            for variable in ("VERUS_ROOT", "VERUS_USE_RUSTUP"):
                value = os.environ.get(variable)
                shown = "未设" if value is None else "空串" if value == "" else value
                if variable == "VERUS_ROOT" and value and os.path.isdir(value):
                    shown += f":top_level_sha256={top_level_sha256(value)}"
                extra += f" {variable}={shown}"
        if "tool-versions-rustup-ignored" not in BREAK:
            # launcher 经 rustup 起 rust_verify：那条工具链里 rust_verify 读的库、RUSTUP_HOME、rustup 程序本身，换了任何一样这一行都变
            extra += rustup_toolchain_fields((version or {}).get("toolchain") or "")
        z3_override = os.environ.get("VERUS_Z3_PATH")
        if z3_override is None or (z3_override == "" and "tool-versions-empty-z3-path-as-unset" in BREAK):
            extra += " VERUS_Z3_PATH=未设"
        elif z3_override == "":
            extra += " VERUS_Z3_PATH=空串"   # Verus 不把空串当没设：照它去起 z3、起不来
        else:
            bare = "/" not in z3_override and "tool-versions-bare-z3-from-current-directory" not in BREAK
            located = (shutil.which(z3_override) or "") if bare else z3_override
            extra += f" VERUS_Z3_PATH={z3_override}:{located or '找不到'}:{file_sha256(located) if located else '没有'}"
    if version is None:
        return f"verus 没装（{directory} 里没有 version.json） verus_sha256={digest}{extra}"
    return f"verus version={version.get('version')} commit={version.get('commit')} toolchain={(version.get('toolchain') or '').split(' ')[0]} verus_sha256={digest}{extra}"


# ── 命令行 ──

def default_threads(environment=os.environ):
    """--threads 的缺省：SINGLEFS_THREAD_CAP 设了取它，否则登记的重活默认并行度（.claude/heavy-thread-default）。"""
    if "threads-default-cpu-count" in BREAK:   # 弄坏开关：改前的样子
        return int(environment.get("SINGLEFS_THREAD_CAP") or os.cpu_count() or 1)
    return default_heavy_threads(environment)


def parse_options(arguments):
    options = {"verus_root": None, "cargo_home": None, "launcher": None,
               "timeout": int(os.environ.get("GATE_VERUS_TIMEOUT_SECONDS") or 600),
               "threads": default_threads(),
               "jobs": int(os.environ.get("GATE_VERUS_JOBS") or 0),
               "memory_cap": os.environ.get("GATE_VERUS_MEMORY_MAX") or "4G",
               "items": [], "verdict_library": None, "reuse": True}
    positional = []
    flags = {"--verus-root": "verus_root", "--cargo-home": "cargo_home", "--timeout": "timeout", "--threads": "threads",
             "--jobs": "jobs", "--memory-cap": "memory_cap", "--verdict-library": "verdict_library"}
    position = 0
    while position < len(arguments):
        argument = arguments[position]
        if argument == "--item":
            if position + 1 >= len(arguments) or not arguments[position + 1]:
                raise ValueError("--item 后面缺项名（可点名的项：verus-proof-check.py list-items <根>）")
            options["items"].append(arguments[position + 1])
            position += 2
        elif argument == "--no-reuse":
            options["reuse"] = False
            position += 1
        elif argument in flags:
            if position + 1 >= len(arguments):
                raise ValueError(f"{argument} 后面缺值")
            value = arguments[position + 1]
            options[flags[argument]] = int(value) if flags[argument] in ("timeout", "threads", "jobs") else value
            position += 2
        elif argument.startswith("--"):
            raise ValueError(f"认不出的选项 {argument}")
        else:
            positional.append(argument)
            position += 1
    if options["threads"] < 1 or options["timeout"] < 1 or options["jobs"] < 0:
        raise ValueError("--threads、--timeout 要正整数，--jobs 不许负")
    return positional, options


CHECKS = {"toolchain": check_toolchain, "proof": check_proof, "vacuity": check_vacuity, "must-red-worlds": check_must_red_worlds}


def run_check(name, root, options):
    options = dict(options, queue_refusal=QueueRefusal(), shared_components=None, unit_index=None, function_keys=None)
    try:
        if name == "toolchain" and options.get("items"):
            raise UsageError("verus-toolchain-pinned 格只有一项（toolchain），不收 --item", ["toolchain"])
        return CHECKS[name](os.path.abspath(root), options)
    except UsageError as usage:
        report_red(usage.message, "照下面列的项名写（verus-proof-check.py list-items <根> 也列），给几次 --item 取并集")
        for item in usage.available:
            print(f"       {item}")  # gate-lint:detail
        return EXIT_USAGE
    except Red as red:
        report_red(red.message, red.howto)
        return EXIT_RED
    except NotRun as not_run:
        print(f"  ! {not_run}")
        return EXIT_NOT_RUN


def list_items(root, verdict_library=None, verus_root=None):
    """逐行打「项<TAB>格[<TAB>说明]」：toolchain、proof（整格；说明里报成项几个、各哪一类，不成项的 spec fn 逐个点名）各一项，证明每个成项的单元一项
    proof:<名>（说明是哪一类与它在核里的位置），空真每处 requires 一项 vacuity:<所属函数>#<第几处>，必红世界每行一项。
    只读登记表、核与表（核经 lib_function_units.py 切单元），不起 Verus、不编。给了 verdict_library（命令行的 --verdict-library）另往 stderr 报每格键对不上的条数
    （report_key_mismatches）。"""
    root = os.path.abspath(root)
    lines = ["toolchain\tverus-toolchain-pinned", "proof\tverus-proof\t整格（有证明义务的单元各一项）"]
    if not os.path.isfile(os.path.join(root, REGISTRY)):
        # 登记表不在（只摆了红例两格的样本根）：只列两项固定的；真要判时那几格自己报登记表不在、判红
        print("\n".join(lines))
        return EXIT_GREEN
    try:
        registry = read_registry(root)
        try:
            kernels = kernel_files(root, registry)
        except NotRun:
            kernels = []
        options = {"unit_index": None}
        functions = verus_functions(root, registry, kernels, options) if kernels else []
        items, left_out = proof_items(root, registry, kernels, functions, options) if kernels else ([], [])
        if kernels:
            lines[1] = (f"proof\tverus-proof\t整格（成项 {len(items)} 个：{item_kinds_text(items)}" +
                        f"；不成项的 spec fn {len(left_out)} 个：{listed_names((name for name, _relative in left_out), limit=len(left_out))}）")
        lines += [f"{entry.item}\tverus-proof\t{entry.kind} {entry.relative}:{entry.unit.first_line}" for entry in items]
        points, _all_at_once = vacuity_points(root, registry, kernels, functions)
        lines += [f"{point[0]}\tverus-vacuity\t{point[1]}" for point in points]
        if kernels:
            lines += [f"{must_red_item(row)}\tverus-must-red-worlds\t第 {row['number']} 行" for row in must_red_rows(root, registry)]
    except Red as red:
        report_red(f"列不出可点名的项：{red.message}", red.howto)
        return EXIT_RED
    for line in lines:
        print(line)
    if verdict_library and kernels:
        report_key_mismatches(root, verdict_library, verus_root)
    return EXIT_GREEN


def report_key_mismatches(root, verdict_library, verus_root):
    """list-items 给了 --verdict-library：证明、空真、必红世界三格逐项照现在的键比判定库，每格往 stderr 打一行各几条（键对不上、无记录、键相同、拒算），
    末了一行合计（底稿第二节第 9 条 ⑦：各道 --list-items 报每项键对不上的判定条数）。只读库、不写库，不起 Verus（要装着：键里的工具链那一样读它的安装）；
    项名逐行的 stdout 不变。必红世界里原文、第五段、rlimit 属性没过的行不起 Verus、不进库，不数。"""
    plan, unreadable = [], []
    _positional, defaults = parse_options([])
    for cell in ("proof", "vacuity", "must-red-worlds"):
        buffer = io.StringIO()
        try:
            with contextlib.redirect_stdout(buffer):
                CHECKS[cell](root, dict(defaults, verdict_library=verdict_library, verus_root=verus_root, plan=plan, queue_refusal=None, shared_components=None,
                                         unit_index=None, function_keys=None))
        except (Red, NotRun, UsageError) as error:
            unreadable.append(f"{cell}：{getattr(error, 'message', str(error))}")
    totals = collections.Counter()
    for cell, counts in plan:
        totals.update(counts)
        print(f"  · {cell}：" + "、".join(f"{state} {counts[state]} 条" for state in PLAN_STATES), file=sys.stderr)  # gate-lint:detail
    for line in unreadable:
        print(f"  ! 这一格的键数不出：{line}", file=sys.stderr)
    print(f"  · 键对不上共 {totals['键对不上']} 条（{len(plan)} 格比了 {sum(totals.values())} 项，判定库 {verdict_library}）", file=sys.stderr)


def main(arguments):
    if arguments[:1] == ["--selftest"]:
        return selftest()
    if arguments[:1] == ["list-items"]:
        try:
            positional, options = parse_options(arguments[1:])
        except ValueError as error:
            positional, options = [], {"error": str(error)}
        extra = [flag for flag in ("items", "launcher", "cargo_home") if options.get(flag)]
        if len(positional) != 1 or not os.path.isdir(positional[0]) or options.get("error") or extra:
            report_red(f"list-items 要恰好一个仓根，另外只收 --verdict-library 目录与 --verus-root 目录{'（' + options['error'] + '）' if options.get('error') else ''}",
                       "例：verus-proof-check.py list-items /path/to/singlefs [--verdict-library <判定库目录>]")
            return EXIT_USAGE
        if "list-items-pipe-unhandled" not in BREAK:
            # 接 | head 时读的一头先关了：照 SIGPIPE 的惯例安静退出（被 SIGPIPE 停，shell 里退 141），不打 BrokenPipeError 的回溯
            signal.signal(signal.SIGPIPE, signal.SIG_DFL)
        return list_items(positional[0], options["verdict_library"], options["verus_root"])
    if not arguments or arguments[0] not in (*CHECKS, "fingerprint", "tool-versions"):
        report_red("用法：verus-proof-check.py <toolchain|proof|vacuity|must-red-worlds> <根> [选项] | list-items <根> [--verdict-library 目录] | fingerprint <包目录> | tool-versions [<根>] | --selftest",
                   "看文件头的用法一节")
        return EXIT_USAGE
    try:
        positional, options = parse_options(arguments[1:])
    except ValueError as error:
        report_red(f"参数不对：{error}", "看文件头的用法一节")
        return EXIT_USAGE
    if arguments[0] == "fingerprint":
        if len(positional) != 1 or not os.path.isdir(positional[0]):
            report_red("fingerprint 要一个包目录", "例：verus-proof-check.py fingerprint ~/.cargo/registry/src/<索引>/vstd-<版本>")
            return EXIT_USAGE
        fingerprint, count = package_source_fingerprint(positional[0])
        print(f"{fingerprint}\t{count}")
        return EXIT_GREEN
    if arguments[0] == "tool-versions":
        return tool_versions(os.path.abspath(positional[0]) if positional else os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY)), options["verus_root"])
    if len(positional) != 1 or not os.path.isdir(positional[0]):
        report_red(f"{arguments[0]} 要恰好一个仓根", "例：verus-proof-check.py proof /path/to/singlefs")
        return EXIT_USAGE
    return run_check(arguments[0], positional[0], options)


# ── 自证 ──

FAKE_VERUS = r'''#!/usr/bin/env python3
# 假的 Verus：按源码里的标记出判定。只给 verus-proof-check.py --selftest 用。
# 输出照真 Verus 0.2026.09.20 的形态（主 agent 2026-10-15 的探针）：整 crate 验时 verification-results 有 success、is-verifying-entire-crate true；
# --verify-function 下没有 success 键、is-verifying-entire-crate false；vir 错只有 encountered-error 与 encountered-vir-error 两样。
# 义务条数照真 Verus 量到的：proof fn 1 条，exec fn 1 条另加每个 while 1 条，spec fn 带 decreases 的 1 条、不带的 0 条，
# derive(Clone) 的类型另有一个 <类型>::clone（1 条）；函数里有 SELFTEST_ZERO_VERIFIED 的那个函数验出 0 条。
# --verify-function 照真 Verus 点：有恰好同名的取它，否则唯一包含它的那一个，几个都包含它报 more than one match，一个都没有报 could not find。
import json, os, re, sys, time
here = os.path.dirname(os.path.realpath(__file__))
commit = json.load(open(os.path.join(here, "version.json")))["verus"]["commit"]
no_cheating = "--no-cheating" in sys.argv
rlimit = float(sys.argv[sys.argv.index("--rlimit") + 1]) if "--rlimit" in sys.argv[:-1] else None
only = sys.argv[sys.argv.index("--verify-function") + 1] if "--verify-function" in sys.argv[:-1] else None
function_line = re.compile(r"^\s*(pub(\([^)]*\))?\s+)?(open\s+|closed\s+)?(broadcast\s+)?(proof\s+|spec\s+|exec\s+)?fn\s+(\w+)")
type_line = re.compile(r"^\s*(pub(\([^)]*\))?\s+)?(struct|enum)\s+(\w+)")
if os.environ.get("SELFTEST_VERUS_TRACE"):
    open(os.environ["SELFTEST_VERUS_TRACE"], "a").write("起了一次\n")
files = []
for directory, _s, names in os.walk("src"):
    for name in sorted(names):
        if name.endswith(".rs"):
            files.append(os.path.join(directory, name))
obligations, order = {}, []
for path in sorted(files):
    current, mode, derive_clone = None, None, False
    for line in open(path).read().split("\n"):
        heading, declared = function_line.match(line), type_line.match(line)
        if heading:
            current, mode = heading.group(6), (heading.group(5) or "exec ").strip()
            obligations[current] = 0 if mode == "spec" else 1
            order.append(current)
        elif declared and derive_clone:
            obligations[declared.group(4) + "::clone"] = 1
            order.append(declared.group(4) + "::clone")
        derive_clone = line.lstrip().startswith("#[derive(") and "Clone" in line
        if current is not None and not heading:
            if mode == "spec" and re.search(r"\bdecreases\b", line) and obligations[current] == 0:
                obligations[current] = 1
            if mode == "exec" and re.match(r"^\s*while\b", line):
                obligations[current] += 1
        if current is not None and "SELFTEST_ZERO_VERIFIED" in line:
            obligations[current] = -10 ** 6
selected = None
if only is not None:
    if only in obligations:
        selected = only
    else:
        matched = [name for name in order if only in name]
        if len(matched) > 1:
            sys.stderr.write(f"error: more than one match found for --verify-function {only}, consider using wildcard *{only}* to verify all matched results,\n"
                             "       or specify a unique substring for the desired function, matched results are:\n"
                             + "".join(f"         - {name}\n" for name in matched) + "\nerror: aborting due to 1 previous error\n\n")
            print(json.dumps({"verification-results": {"encountered-error": True, "encountered-vir-error": True}, "verus": {"commit": commit}}))
            sys.exit(1)
        selected = matched[0] if matched else None
functions, errors, vir, found = 0, [], [], selected is not None
for name in order:
    if only is None or name == selected:
        functions += max(obligations[name], 0)
for path in sorted(files):
    lines = open(path).read().split("\n")
    owner_vacuous = owner_failed = closure_vacuous = False
    current = None
    for number, line in enumerate(lines, 1):
        if "SELFTEST_SLEEP" in line:
            time.sleep(30)
        if "SELFTEST_KILLED" in line or (os.environ.get("SELFTEST_KILL_MARK") and os.environ["SELFTEST_KILL_MARK"] in line):
            os.kill(os.getpid(), 9)
        if no_cheating and re.search(r"\bassume\(", line.split("//")[0]):
            vir.append(("assume/admit not allowed with --no-cheating", path, number, 1))
        heading = function_line.match(line)
        if heading:
            current = heading.group(6)
        if only is not None and current != selected:
            continue
        if heading:
            owner_vacuous = "SELFTEST_VACUOUS" in line
            owner_failed = False
        if "SELFTEST_CLOSURE_VACUOUS" in line:
            closure_vacuous = True
        if "assert(false);" in line:
            if closure_vacuous:
                closure_vacuous = False
            elif not owner_vacuous and not owner_failed:
                errors.append(("assertion failed", path, number, line.index("assert(false)") + 1))
                owner_failed = True
        if "SELFTEST_BROKEN" in line:
            errors.append(("postcondition not satisfied", path, number, 1))
        needs = re.search(r"SELFTEST_NEEDS_RLIMIT_(\d+)", line)
        if needs and rlimit is not None and rlimit < int(needs.group(1)):
            errors.append(("while loop: Resource limit (rlimit) exceeded; consider rerunning with --profile for more details", path, number, 1))
if only is not None and not found:
    vir.append((f"could not find function {only} specified by --verify-function", "src/lib.rs", 1, 1))
if rlimit is None:
    vir.append(("SELFTEST: --rlimit not on the command line", "src/lib.rs", 1, 1))
for message, path, number, column in errors + vir:
    sys.stderr.write(f"error: {message}\n   --> {path}:{number}:{column}\n    |\n\n")
if vir:
    vir_results = {"encountered-error": True, "encountered-vir-error": True}
    if only is None:
        vir_results["success"] = False
    print(json.dumps({"verification-results": vir_results, "verus": {"commit": commit}}))
    sys.exit(1)
failed = len({(p, n) for _m, p, n, _c in errors})
results = {"encountered-error": bool(errors), "encountered-vir-error": False}
if only is None:
    results["success"] = not errors
results.update({"verified": max(functions - failed, 0), "errors": failed, "is-verifying-entire-crate": only is None})
if any("SELFTEST_NO_COUNTS" in open(path).read() for path in files):
    del results["verified"], results["errors"]
print(json.dumps({"verification-results": results, "verus": {"commit": commit}}))
sys.exit(1 if errors else 0)
'''

SELFTEST_KERNEL = """use vstd::prelude::*;
verus! {
// 这里的 assume 只在注释里，不算作弊标记
pub fn clamp_to_limit(value: u64, limit: u64) -> (result: u64)
    requires
        limit > 0,
    ensures
        result <= limit,
{
    if value > limit { limit } else { value }
}

pub fn keep_small(value: u64) -> (result: u64)
    ensures
        result <= 10,
{
    let pick = |candidate: u64| -> (picked: u64)
        requires
            candidate <= 10,
        { candidate };
    if value > 10 { pick(10) } else { pick(value) }
}
} // verus!
"""


SELFTEST_SHARED_SPEC_KERNEL = """use vstd::prelude::*;
verus! {
pub open spec fn small_bound() -> u64 {
    10
}

pub fn clamp_to_limit(value: u64, limit: u64) -> (result: u64)
    requires
        limit > 0,
    ensures
        result <= limit,
{
    if value > limit { limit } else { value }
}

pub fn keep_small(value: u64) -> (result: u64)
    ensures
        result <= small_bound(),
{
    if value > 10 { 10 } else { value }
}

pub fn keep_tiny(value: u64) -> (result: u64)
    ensures
        result <= small_bound(),
{
    if value > 3 { 3 } else { value }
}
} // verus!
"""


# 证明成项的四类各一个、加一个不成项的 spec fn：义务条数（假 Verus 照真 Verus 量到的数）Pair::clone 1、sum_to 1、lemma_within_itself 1、count_up 2（一个 while），
# within 0（不成项）；之和 5
SELFTEST_ITEMS_KERNEL = """use vstd::prelude::*;
verus! {
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pair {
    pub low: u64,
    pub high: u64,
}

pub open spec fn within(value: u64, limit: u64) -> bool {
    value <= limit
}

pub open spec fn sum_to(n: nat) -> nat
    decreases n,
{
    if n == 0 { 0 } else { n + sum_to((n - 1) as nat) }
}

pub proof fn lemma_within_itself(value: u64)
    ensures
        within(value, value),
{
}

pub fn count_up(limit: u64) -> (result: u64)
    ensures
        result == limit,
{
    let mut index: u64 = 0;
    while index < limit
        invariant
            index <= limit,
        decreases limit - index,
    {
        index = index + 1;
    }
    index
}
} // verus!
"""


# 自证的开关轮几趟同时起（每趟各用各的临时目录、假 Verus 至多两条线程）
SELFTEST_BREAK_WORKERS = 6


def selftest():
    global MEMORY_CAP_RUNNER
    os.environ.pop(QUEUE_REFUSED_FILE_VARIABLE, None)   # 外面门禁给的那份不进自证：跨格那几格自己建一份
    # 复用哨兵：别的格比的是现判与复用的条数，键相同的全当哨兵、不抽强制现判（哨兵数比项数多）；reuse-three-questions 与 reuse-sentinel 两格自己设
    os.environ[reuse_checks.SENTINEL_COUNT_VARIABLE] = "1000"
    cases, wrong = 0, []
    with tempfile.TemporaryDirectory(prefix="verus-proof-check-selftest-") as work:
        commit, other_commit = "a" * 40, "b" * 40
        install = os.path.join(work, "tools", "verus-x86-linux")
        os.makedirs(install)
        with open(os.path.join(install, "verus"), "w") as handle:
            handle.write(FAKE_VERUS)
        os.chmod(os.path.join(install, "verus"), 0o755)
        zip_path = os.path.join(work, "tools", "verus.zip")
        open(zip_path, "wb").write(b"zip")
        zip_digest = hashlib.sha256(b"zip").hexdigest()
        cargo_home = os.path.join(work, "cargo-home")
        package_directory = os.path.join(cargo_home, "registry", "src", "index-x", "vstd-0.0.1")
        os.makedirs(os.path.join(package_directory, "src"))
        open(os.path.join(package_directory, "src", "lib.rs"), "w").write("// vstd\n")
        open(os.path.join(package_directory, "Cargo.toml"), "w").write("[package]\n")
        open(os.path.join(package_directory, ".cargo-ok"), "w").write('{"v":1}')
        json.dump({"git": {"sha1": "c" * 40}, "path_in_vcs": "source/vstd"}, open(os.path.join(package_directory, ".cargo_vcs_info.json"), "w"))
        fingerprint, _count = package_source_fingerprint(package_directory)

        def make_root(name, kernel=SELFTEST_KERNEL, count=2, must_red=None, version_commit=commit, lock_version="0.0.1", with_kernel=True, wired=True, extra_dependency="",
                      rlimit=7):
            root = os.path.join(work, name)
            os.makedirs(os.path.join(root, ".claude", "gate.d"))
            os.makedirs(os.path.join(root, "crates", "core", "src"))
            json.dump({"verus": {"version": "0.9.9.tag", "commit": version_commit, "toolchain": "stable"}}, open(os.path.join(install, "version.json"), "w"))
            open(os.path.join(root, REGISTRY), "w").write(
                "# 自证用\n"
                f"release\t0.9.9.tag\t{commit}\t{zip_digest}\tunused\n"
                f"package\tvstd\t0.0.1\t{'c' * 40}\tsource/vstd\t{fingerprint}\n"
                "crate\tcrates/core\t%d\n" % count +
                "kernel\tcrates/core/src/kernel.rs\n"
                "must-red-worlds\tcrates/core/verus-must-red-worlds.tsv\n" + (f"rlimit\t{rlimit}\n" if rlimit is not None else ""))
            open(os.path.join(root, "Cargo.lock"), "w").write(f'version = 4\n\n[[package]]\nname = "vstd"\nversion = "{lock_version}"\n')
            open(os.path.join(root, "crates", "core", "Cargo.toml"), "w").write(
                '[package]\nname = "core-sample"\nedition = "2021"\n\n[dependencies]\n' + ('vstd = "=0.0.1"\n' if wired else "") + extra_dependency)
            open(os.path.join(root, "crates", "core", "src", "lib.rs"), "w").write("pub mod kernel;\n" if with_kernel else "")
            if with_kernel:
                open(os.path.join(root, "crates", "core", "src", "kernel.rs"), "w").write(kernel)
            rows = must_red if must_red is not None else ["收紧改松\tcrates/core/src/kernel.rs\tif value > limit { limit } else { value }\tif value > limit { limit } else { value } // SELFTEST_BROKEN\tproof_errors=1"]
            open(os.path.join(root, "crates", "core", "verus-must-red-worlds.tsv"), "w").write("# 自证用\n" + "".join(row + "\n" for row in rows))
            return root

        options = {"verus_root": install, "cargo_home": cargo_home, "launcher": [], "timeout": 5, "threads": 2, "jobs": 2,
                   "memory_cap": "1G"}

        def expect(label, check, root, want_exit, want_text):
            nonlocal cases
            cases += 1
            import io
            import contextlib
            buffer = io.StringIO()
            with contextlib.redirect_stdout(buffer):
                got = run_check(check, root, options)
            output = buffer.getvalue()
            want_texts = want_text if isinstance(want_text, tuple) else (want_text,)
            if got != want_exit or any(text not in output for text in want_texts):
                wrong.append(label)
                print(f"    {label} 这一格判错：期望退 {want_exit}、输出含「{'」「'.join(want_texts)}」，实测退 {got}：{output.strip()[-300:]}")  # gate-lint:detail
            return output

        green = make_root("green")
        expect("toolchain-green", "toolchain", green, 0, "个包的版本、出处与源码指纹都与登记相同")
        expect("proof-green", "proof", green, 0, "成项 2 个（exec fn 2），各项 verified 之和 2、0 errors（登记 2）；不成项的 spec fn 0 个")
        expect("vacuity-green", "vacuity", green, 0, "2 处 requires")
        expect("must-red-green", "must-red-worlds", green, 0, "1 行每行改坏之后")
        expect("toolchain-commit", "toolchain", make_root("commit", version_commit=other_commit), 1, "Verus 安装的 commit 是")
        expect("toolchain-lock", "toolchain", make_root("lock", lock_version="0.0.2"), 1, "Cargo.lock 里 vstd 的版本")
        json.dump({"verus": {"version": "0.9.9.tag", "commit": commit, "toolchain": "stable"}}, open(os.path.join(install, "version.json"), "w"))
        tampered = make_root("tampered")
        open(os.path.join(package_directory, "src", "lib.rs"), "a").write("// 改过\n")
        expect("toolchain-fingerprint", "toolchain", tampered, 1, "源码内容指纹是")
        open(os.path.join(package_directory, "src", "lib.rs"), "w").write("// vstd\n")
        expect("kernel-not-wired", "proof", make_root("absent", with_kernel=False, wired=False), 77, "还没进仓")
        expect("kernel-deleted", "proof", make_root("deleted", with_kernel=False, wired=True), 1, "而 crate 已经接了 Verus")
        expect("proof-broken", "proof", make_root("broken", kernel=SELFTEST_KERNEL.replace("result <= limit,", "result <= limit, // SELFTEST_BROKEN")), 1, "Verus 没验过")
        # 计数之和不等判红：两个函数各验出 1 条，登记的是 3（开关 count-unchecked 下判错）
        expect("proof-count", "proof", make_root("count", count=3), 1, "各项 verified 之和 2 与登记的 3 不同")
        expect("proof-cheating", "proof", make_root("cheating", kernel=SELFTEST_KERNEL.replace("if value > limit { limit }", "assume(false);\n    if value > limit { limit }")), 1, "不许的标记")
        expect("proof-timeout", "proof", make_root("timeout", kernel=SELFTEST_KERNEL + "// SELFTEST_SLEEP\n"), 1, "没出结局，按红记")
        # Verus 被信号停了：输出不算判定、出路照包装文件头，不写修证明（开关 killed-as-proof-red 下这一格与 proof-killed 判错）
        expect("proof-killed-direct", "proof", make_root("killeddirect", kernel=SELFTEST_KERNEL + "// SELFTEST_KILLED\n"), 1,
               ("Verus 被信号停了（退 -9，信号 9）", "外面 kill -9 同样退 137"))
        vacuous = SELFTEST_KERNEL.replace("pub fn clamp_to_limit(", "pub fn clamp_to_limit( /* SELFTEST_VACUOUS */")
        expect("vacuity-vacuous", "vacuity", make_root("vacuous", kernel=vacuous), 1, "空真")
        expect("vacuity-closure-found", "vacuity", green, 0, "2 处 requires")
        closure_vacuous = SELFTEST_KERNEL.replace("pub fn keep_small(value: u64) -> (result: u64)\n",
                                                  "pub fn keep_small(value: u64) -> (result: u64)\n    requires\n        value < 100,\n").replace(
            "            candidate <= 10,", "            candidate <= 10, // SELFTEST_CLOSURE_VACUOUS")
        expect("vacuity-closure-vacuous", "vacuity", make_root("closurevacuous", kernel=closure_vacuous), 1, "keep_small 里第 20 行起的闭包：函数体头插 assert(false) 照样验得过")
        expect("crate-extra-dependency", "proof", make_root("extradependency", extra_dependency='singlefs-format = { path = "../format" }\n'), 1, "除 vstd 之外还依赖 singlefs-format")
        expect("must-red-survives", "must-red-worlds", make_root("survives", must_red=["改注释\tcrates/core/src/kernel.rs\tif value > limit { limit } else { value }\tif value > limit { limit } else { value } // 只是注释\tproof_errors=1"]), 1, "照样验过")
        expect("must-red-anchor", "must-red-worlds", make_root("anchor", must_red=["不在的原文\tcrates/core/src/kernel.rs\tnot present anywhere\tx\tproof_errors=1"]), 1, "恰好命中一次")
        # 证明错条数与第五段 proof_errors= 不同：判红（开关 proof-error-count-unchecked 下判错）
        expect("must-red-proof-error-count", "must-red-worlds", make_root("proofcount", must_red=["条数不对\tcrates/core/src/kernel.rs\tif value > limit { limit } else { value }\tif value > limit { limit } else { value } // SELFTEST_BROKEN\tproof_errors=2"]),
               1, "证明错 1 条，表里写的是 proof_errors=2")
        expect("must-red-empty", "must-red-worlds", make_root("empty", must_red=[]), 1, "一行都没有")
        # rlimit：登记的数写上 Verus 命令行（假 Verus 见到的 --rlimit 小于标记要的数就报 rlimit 用尽）；没有 rlimit 行取 DEFAULT_RLIMIT；
        # 必红世界里只因 rlimit 用尽才报错的那一行不作数、判红（rlimit 用尽不计入证明错条数）
        needs_seven = SELFTEST_KERNEL.replace("result <= limit,", "result <= limit, // SELFTEST_NEEDS_RLIMIT_7")
        expect("proof-rlimit-registered", "proof", make_root("rlimitregistered", kernel=needs_seven, rlimit=7), 0, "--rlimit 7")
        expect("proof-rlimit-too-low", "proof", make_root("rlimitlow", kernel=needs_seven, rlimit=3), 1, "Resource limit (rlimit) exceeded")
        expect("proof-rlimit-default", "proof", make_root("rlimitdefault", kernel=SELFTEST_KERNEL.replace("result <= limit,", f"result <= limit, // SELFTEST_NEEDS_RLIMIT_{DEFAULT_RLIMIT}"),
                                                          rlimit=None), 0, f"--rlimit {DEFAULT_RLIMIT}")
        expect("registry-rlimit-zero", "proof", make_root("rlimitzero", rlimit=0), 1, "rlimit 要是正整数")
        expect("must-red-rlimit", "must-red-worlds", make_root("mustredrlimit", must_red=["只撞 rlimit\tcrates/core/src/kernel.rs\tif value > limit { limit } else { value }\t"
                                                                                        "if value > limit { limit } else { value } // SELFTEST_NEEDS_RLIMIT_99\tproof_errors=1"]),
               1, "这一行红在算力额度上，不是证明失败")
        # 输出里夹着 rlimit 用尽、另有一条真的证明错：红计入（只剩 rlimit 用尽的才不作数）
        expect("must-red-rlimit-with-proof-error", "must-red-worlds", make_root("mustredrlimitproof", must_red=[
            "撞 rlimit 又有证明错\tcrates/core/src/kernel.rs\tif value > limit { limit } else { value }\t"
            "if value > limit { limit } else { value } // SELFTEST_NEEDS_RLIMIT_99 SELFTEST_BROKEN\tproof_errors=1"]),
               0, "1 行输出里夹着 rlimit 用尽、另有证明错，红计入")
        # 代码轮第三轮判决 J3 那四行的情形：同一行在 rlimit 10 上输出一条 rlimit 用尽加一条证明错（Verus 把 rlimit 用尽也计进 errors，errors 2），
        # 在 rlimit 20 上只输出同一条证明错（errors 1）；第五段 proof_errors=1 只数证明错，两份都判绿（开关 rlimit-counted-in-proof-errors 下 rlimit 10 那份判错）
        j3_row = ("rlimit 用尽另记\tcrates/core/src/kernel.rs\tif value > limit { limit } else { value }\t"
                  "if value > limit { limit } else { value } // SELFTEST_NEEDS_RLIMIT_15\\n    // SELFTEST_BROKEN\t")
        expect("must-red-rlimit-10-counts-proof-errors-only", "must-red-worlds", make_root("j3rlimit10", must_red=[j3_row + "proof_errors=1"], rlimit=10),
               0, ("1 行输出里夹着 rlimit 用尽、另有证明错，红计入", "证明错条数与第五段 proof_errors= 相同"))
        expect("must-red-rlimit-20-counts-proof-errors-only", "must-red-worlds", make_root("j3rlimit20", must_red=[j3_row + "proof_errors=1"], rlimit=20),
               0, ("0 行输出里夹着 rlimit 用尽", "证明错条数与第五段 proof_errors= 相同"))
        # 第五段写成旧形态（Verus 判定行的「verified, 2 errors」，rlimit 10 上它正好对得上）：不是 proof_errors=<正整数> 的行判红、出路写约定
        # （开关 expectation-form-unchecked 下判错）
        expect("must-red-legacy-expectation", "must-red-worlds", make_root("legacyexpectation", must_red=[j3_row + "verified, 2 errors"], rlimit=10),
               1, ("的第五段「verified, 2 errors」不是 proof_errors=<证明错条数>", "必红世界表第五段一律写 proof_errors=M"))
        # 函数上的 rlimit 属性盖过命令行的 --rlimit：核里有它，proof 判红；必红世界的替换文里有它，那一行判红
        expect("proof-rlimit-attribute", "proof", make_root("rlimitattribute", kernel=SELFTEST_KERNEL.replace(
            "pub fn clamp_to_limit(", "#[verifier::rlimit(20)]\npub fn clamp_to_limit(")), 1, "verifier::rlimit（函数上的 rlimit 属性盖过命令行的 --rlimit 7")
        expect("must-red-rlimit-attribute", "must-red-worlds", make_root("mustredrlimitattribute", must_red=[
            "加 rlimit 属性\tcrates/core/src/kernel.rs\tpub fn clamp_to_limit(\t#[verifier::rlimit(20)] // SELFTEST_BROKEN\\npub fn clamp_to_limit(\tproof_errors=1"]),
               1, "的替换文里有 verifier::rlimit")
        # JSON 里缺整数的 verified 与 errors（vir 错没走到验证、或 Verus 换了 JSON 的写法）：那一行判红
        expect("must-red-counts-missing", "must-red-worlds", make_root("mustrednocounts", must_red=[
            "JSON 缺数\tcrates/core/src/kernel.rs\tif value > limit { limit } else { value }\t"
            "if value > limit { limit } else { value } // SELFTEST_BROKEN SELFTEST_NO_COUNTS\tproof_errors=1"]),
               1, "Verus 的 JSON 里没有整数的 verified 与 errors")
        # ── 逐项点名与按条复用（文件头「逐项点名与按条复用」）──
        import io
        import contextlib
        item_line = re.compile(r"^  · (\S.*?)：(绿|红)，(现判|复用)", re.M)

        def run_items(check, root, environment=None, **changes):
            """跑一格交回 (退出码, 输出, {项: (绿/红, 现判/复用)})；changes 盖进 options（items、verdict_library、judging_units、reuse、timeout），environment 临时设进 os.environ。"""
            saved_environment = {name: os.environ.get(name) for name in (environment or {})}
            os.environ.update(environment or {})
            buffer = io.StringIO()
            try:
                with contextlib.redirect_stdout(buffer):
                    got = run_check(check, root, dict(options, **changes))
            finally:
                for name, value in saved_environment.items():
                    if value is None:
                        os.environ.pop(name, None)
                    else:
                        os.environ[name] = value
            output = buffer.getvalue()
            return got, output, {match.group(1): (match.group(2), match.group(3)) for match in item_line.finditer(output)}

        def judge_cell(label, passed, detail):
            nonlocal cases
            cases += 1
            if not passed:
                wrong.append(label)
                print(f"    {label} 这一格判错：{detail}")  # gate-lint:detail

        green_row = "改坏{0}\tcrates/core/src/kernel.rs\tif value > limit {{ limit }} else {{ value }}\tif value > limit {{ limit }} else {{ value }} // SELFTEST_BROKEN {0}\tproof_errors=1"
        survives_row = "改注释{0}\tcrates/core/src/kernel.rs\tif value > limit {{ limit }} else {{ value }}\tif value > limit {{ limit }} else {{ value }} // 只是注释 {0}\tproof_errors=1"
        # 三行必红世界改的是同一处原文：原文要恰好命中一次，三行各自改一份副本
        mixed_root = make_root("itemsmixed", must_red=[green_row.format("A"), survives_row.format("B"), green_row.format("C")])
        vacuity_lines = [requires_line for requires_line, _brace, _label in requires_insertion_points(SELFTEST_KERNEL)]
        listed_items = ["toolchain", "proof", "proof:clamp_to_limit", "proof:keep_small", "vacuity:clamp_to_limit#1", "vacuity:keep_small#1",
                        "must-red:改坏A", "must-red:改注释B", "must-red:改坏C"]
        judge_cell("vacuity-points-in-kernel", len(vacuity_lines) == 2, f"自证核里该有 2 处 requires，认出 {vacuity_lines}")
        trace_path = os.path.join(work, "verus-trace.txt")
        open(trace_path, "w").close()
        os.environ["SELFTEST_VERUS_TRACE"] = trace_path
        buffer = io.StringIO()
        with contextlib.redirect_stdout(buffer):
            listed_exit = list_items(mixed_root)
        os.environ.pop("SELFTEST_VERUS_TRACE", None)
        listed = [line.split("\t")[0] for line in buffer.getvalue().strip().split("\n")]
        judge_cell("list-items", listed_exit == 0 and listed == listed_items, f"期望退 0、逐行 {listed_items}，实测退 {listed_exit}、{listed}")
        judge_cell("list-items-no-verus", open(trace_path).read() == "", f"list-items 起了 Verus：{open(trace_path).read()!r}")
        # 读的一头先关了（接 | head 的样子）：被 SIGPIPE 安静停掉，stderr 一个字都没有
        read_end, write_end = os.pipe()
        os.close(read_end)
        closed_pipe = subprocess.run([sys.executable, os.path.realpath(__file__), "list-items", mixed_root], stdout=write_end, stderr=subprocess.PIPE, text=True)
        os.close(write_end)
        judge_cell("list-items-closed-pipe", closed_pipe.returncode == -signal.SIGPIPE and closed_pipe.stderr == "",
                   f"期望被 SIGPIPE 停（{-signal.SIGPIPE}）、stderr 空，实测 {closed_pipe.returncode}：{closed_pipe.stderr.strip()[-200:]!r}")
        got, output, verdicts = run_items("must-red-worlds", mixed_root, items=["must-red:改注释B"])
        judge_cell("item-select-one", got == 1 and list(verdicts) == ["must-red:改注释B"], f"--item must-red:改注释B 期望只判这一行（退 1），实测退 {got}、判了 {list(verdicts)}")
        got, output, _verdicts = run_items("must-red-worlds", mixed_root, items=["must-red:没有这一行"])
        judge_cell("item-unknown", got == EXIT_USAGE and "must-red:改坏A" in output, f"点名没有的项期望退 2、列出可点名的项，实测退 {got}：{output.strip()[-300:]}")
        got, output, verdicts = run_items("proof", mixed_root, items=["proof:clamp_to_limit"])
        judge_cell("item-proof-one", got == 0 and list(verdicts) == ["proof:clamp_to_limit"] and "不比之和" in output,
                   f"--item proof:clamp_to_limit 期望只验这一个函数、不比之和（退 0），实测退 {got}、判了 {list(verdicts)}：{output.strip()[-200:]}")
        got, output, _verdicts = run_items("proof", mixed_root, items=["proof:没有这个函数"])
        judge_cell("item-proof-unknown", got == EXIT_USAGE and "proof:keep_small" in output, f"点名没有的函数期望退 2、列出可点名的项，实测退 {got}")
        # 点名单跑一项与它在整格里跑的判定相同（必红世界一绿一红一绿；空真里外层可满足、闭包不可满足）
        _got, _output, whole = run_items("must-red-worlds", mixed_root)
        alone = {item: run_items("must-red-worlds", mixed_root, items=[item])[2].get(item) for item in whole}
        judge_cell("item-alone-equals-whole-must-red", len(whole) == 3 and alone == whole and whole["must-red:改注释B"][0] == "红",
                   f"整格 {whole}，逐项单跑 {alone}")
        closure_root = make_root("itemsclosure", kernel=closure_vacuous)
        _got, _output, whole = run_items("vacuity", closure_root)
        alone = {item: run_items("vacuity", closure_root, items=[item])[2].get(item) for item in whole}
        judge_cell("item-alone-equals-whole-vacuity", len(whole) == 3 and alone == whole and sorted(verdict for verdict, _how in whole.values()) == ["红", "绿", "绿"],
                   f"整格 {whole}，逐项单跑 {alone}")
        # 按条复用：头一趟全现判、记下；第二趟全复用；关了复用全现判；核 crate、工具链变了全现判，原因分栏点名
        store_directory = os.path.join(work, "verdict-library")
        reuse_root = make_root("reuse", must_red=[green_row.format("A"), green_row.format("B"), green_row.format("C")])
        got, output, _verdicts = run_items("must-red-worlds", reuse_root, verdict_library=store_directory)
        judge_cell("reuse-first-run-records", got == 0 and "现判 3、复用 0（现判的原因：无记录 3）" in output, f"退 {got}：{output.strip()[-300:]}")
        got, output, verdicts = run_items("must-red-worlds", reuse_root, verdict_library=store_directory)
        judge_cell("reuse-second-run-reuses", got == 0 and "现判 0、复用 3" in output and all(how == "复用" for _verdict, how in verdicts.values()),
                   f"退 {got}：{output.strip()[-300:]}")
        got, output, _verdicts = run_items("must-red-worlds", reuse_root, verdict_library=store_directory, reuse=False)
        judge_cell("reuse-off", got == 0 and "现判 3、复用 0（现判的原因：关了复用 3）" in output, f"退 {got}：{output.strip()[-300:]}")
        for check, count in (("proof", 2), ("vacuity", 2)):
            run_items(check, reuse_root, verdict_library=store_directory)
            got, output, _verdicts = run_items(check, reuse_root, verdict_library=store_directory)
            judge_cell(f"reuse-{check}", got == 0 and f"现判 0、复用 {count}" in output, f"第二趟期望全复用，实测退 {got}：{output.strip()[-300:]}")
        # 判法单元指纹是另一列：输入一样都没变、只换判法单元指纹，整格全现判、原因点名它；再照真的跑一趟记回去，后面几格比的是真判法单元指纹
        got, output, _verdicts = run_items("must-red-worlds", reuse_root, verdict_library=store_directory, judging_units="自证造的另一份判法单元")
        judge_cell("reuse-judging-units-changed", got == 0 and "现判 3、复用 0（现判的原因：判法单元指纹 变了 3）" in output, f"退 {got}：{output.strip()[-300:]}")
        run_items("must-red-worlds", reuse_root, verdict_library=store_directory)
        # N38：被验函数体里只加一行注释，三格一项都不现判（开关 reuse-key-whole-crate 下判错：核 crate 整个目录的原文变了）
        kernel_path = os.path.join(reuse_root, "crates", "core", "src", "kernel.rs")

        def edit_kernel(path, old, new):
            with open(path, encoding="utf-8") as handle:
                kernel_text = handle.read()
            assert kernel_text.count(old) == 1, old
            with open(path + ".new", "w", encoding="utf-8") as handle:
                handle.write(kernel_text.replace(old, new))
            os.replace(path + ".new", path)

        edit_kernel(kernel_path, "    if value > limit { limit } else { value }\n", "    // 函数体里只加一行注释\n    if value > limit { limit } else { value }\n")
        after = {check: run_items(check, reuse_root, verdict_library=store_directory) for check in ("proof", "vacuity", "must-red-worlds")}
        judge_cell("reuse-comment-in-function-body", all(got == 0 for got, _output, _verdicts in after.values())
                   and "现判 0、复用 2" in after["proof"][1] and "现判 0、复用 2" in after["vacuity"][1] and "现判 0、复用 3" in after["must-red-worlds"][1],
                   "期望三格全复用，实测 " + "；".join(f"{check} 退 {got}：{output.strip()[-200:]}" for check, (got, output, _verdicts) in after.items()))
        # N39：requires 上面加一个空行（行号挪一行），空真两处都复用（开关 vacuity-key-by-line-number 下判错）
        edit_kernel(kernel_path, "    requires\n        limit > 0,", "\n    requires\n        limit > 0,")
        got, output, _verdicts = run_items("vacuity", reuse_root, verdict_library=store_directory)
        judge_cell("reuse-vacuity-line-moved", got == 0 and "现判 0、复用 2" in output, f"期望空真两处都复用，实测退 {got}：{output.strip()[-300:]}")
        # 限时不进键：改限时，三格全复用（开关 reuse-key-with-timeout 下判错）
        after = {check: run_items(check, reuse_root, verdict_library=store_directory, timeout=6) for check in ("proof", "vacuity", "must-red-worlds")}
        judge_cell("reuse-timeout-changed", all(got == 0 for got, _output, _verdicts in after.values())
                   and "现判 0、复用 2" in after["proof"][1] and "现判 0、复用 2" in after["vacuity"][1] and "现判 0、复用 3" in after["must-red-worlds"][1],
                   "期望三格全复用，实测 " + "；".join(f"{check} 退 {got}：{output.strip()[-200:]}" for check, (got, output, _verdicts) in after.items()))
        # 改一个函数只重验那一项：clamp_to_limit 的 ensures 改一个字，证明与空真只现判它那一项、keep_small 的复用；必红世界三行改的都在它里面，全现判
        # （开关 reuse-key-without-units 下全复用、reuse-key-whole-crate 下全现判，这一格都判错）
        edit_kernel(kernel_path, "        result <= limit,\n", "        result <= limit + 0,\n")
        after = {check: run_items(check, reuse_root, verdict_library=store_directory) for check in ("proof", "vacuity", "must-red-worlds")}
        judge_cell("reuse-proof-function-only", all(got == 0 for got, _output, _verdicts in after.values())
                   and after["proof"][2].get("proof:clamp_to_limit", ("", ""))[1] == "现判" and after["proof"][2].get("proof:keep_small", ("", ""))[1] == "复用"
                   and after["vacuity"][2].get("vacuity:clamp_to_limit#1", ("", ""))[1] == "现判" and after["vacuity"][2].get("vacuity:keep_small#1", ("", ""))[1] == "复用"
                   and "现判 3、复用 0" in after["must-red-worlds"][1] and "改了 fn clamp_to_limit" in after["proof"][1],
                   "期望只现判 clamp_to_limit 那几项、点名它变了，实测 " + "；".join(f"{check} 退 {got}：{output.strip()[-300:]}" for check, (got, output, _verdicts) in after.items()))
        with open(os.path.join(install, "libextra.so"), "w") as handle:
            handle.write("安装目录里多一份库\n")
        try:
            got, output, _verdicts = run_items("must-red-worlds", reuse_root, verdict_library=store_directory)
        finally:
            os.remove(os.path.join(install, "libextra.so"))
        judge_cell("reuse-toolchain-changed", got == 0 and "现判 3、复用 0（现判的原因：工具链 变了 3）" in output, f"退 {got}：{output.strip()[-300:]}")
        # 必红世界一行只改变异名：键里不带名字与文件，照复用（开关 must-red-key-with-name-and-path 下判错）
        renamed_store = os.path.join(work, "verdict-library-renamed")
        renamed_root = make_root("reuserenamed", must_red=[green_row.format("A"), green_row.format("B"), green_row.format("C")])
        run_items("must-red-worlds", renamed_root, verdict_library=renamed_store)
        table_of_renamed = os.path.join(renamed_root, "crates", "core", "verus-must-red-worlds.tsv")
        edit_kernel(table_of_renamed, "改坏B\t", "改坏乙\t")
        got, output, verdicts = run_items("must-red-worlds", renamed_root, verdict_library=renamed_store)
        judge_cell("reuse-must-red-renamed", got == 0 and "现判 0、复用 3" in output and verdicts.get("must-red:改坏乙", ("", ""))[1] == "复用",
                   f"期望改名那一行照复用，实测退 {got}：{output.strip()[-300:]}")
        # 改一个 spec fn：用到它的两个函数都重验，没用它的那个复用；spec fn 自己不带 decreases、不成项（开关 reuse-key-function-only 下判错：只看函数自己的指纹）
        spec_store = os.path.join(work, "verdict-library-spec")
        spec_root = make_root("reusespec", kernel=SELFTEST_SHARED_SPEC_KERNEL, count=3)
        first_spec = run_items("proof", spec_root, verdict_library=spec_store)
        edit_kernel(os.path.join(spec_root, "crates", "core", "src", "kernel.rs"), "{\n    10\n}", "{\n    11\n}")
        got, output, verdicts = run_items("proof", spec_root, verdict_library=spec_store)
        judge_cell("reuse-spec-function-all-users", first_spec[0] == 0 and got == 0
                   and {item: how for item, (_verdict, how) in verdicts.items()}
                   == {"proof:clamp_to_limit": "复用", "proof:keep_small": "现判", "proof:keep_tiny": "现判"}
                   and "改了 fn small_bound" in output,
                   f"第一趟退 {first_spec[0]}，期望用到 small_bound 的两项现判、点名它变了，clamp_to_limit 复用，实测退 {got}、{verdicts}：{output.strip()[-300:]}")
        # 计数之和用复用的记录：判绿记下之后把登记数改成 3，两项都复用、之和 2 照比、判红（开关 count-unchecked 下判错）
        sum_store = os.path.join(work, "verdict-library-sum")
        sum_root = make_root("reusesum")
        run_items("proof", sum_root, verdict_library=sum_store)
        edit_kernel(os.path.join(sum_root, REGISTRY), "crate\tcrates/core\t2\n", "crate\tcrates/core\t3\n")
        got, output, verdicts = run_items("proof", sum_root, verdict_library=sum_store)
        judge_cell("proof-sum-from-reused-records", got == 1 and "现判 0、复用 2" in output and "各项 verified 之和 2 与登记的 3 不同" in output,
                   f"期望两项复用、之和 2 与登记的 3 不同判红，实测退 {got}：{output.strip()[-300:]}")
        # ── 证明成哪几项、每项怎么判（check_proof、proof_items、verification_clean；假 Verus 出真 Verus 那样的 JSON）──
        # 假 Verus 自己的形态：--verify-function 下没有 success 键、is-verifying-entire-crate false，整 crate 下两样都有（照真 Verus 的探针）
        items_root = make_root("proofitems", kernel=SELFTEST_ITEMS_KERNEL, count=5)
        crate_of_items = os.path.join(items_root, "crates", "core")
        shapes = {}
        for label, extra in (("按函数", ["--verify-root", "--verify-function", "count_up"]), ("整 crate", [])):
            ran = subprocess.run([os.path.join(install, "verus"), "--no-cheating", "--rlimit", "7", "--output-json", *extra, "src/lib.rs"],
                                 cwd=crate_of_items, capture_output=True, text=True)
            shapes[label] = json.loads(ran.stdout)["verification-results"] if ran.stdout.strip().startswith("{") else {"读不出": ran.stdout[-200:]}
        judge_cell("fake-verus-real-shape",
                   shapes["按函数"] == {"encountered-error": False, "encountered-vir-error": False, "verified": 2, "errors": 0, "is-verifying-entire-crate": False}
                   and shapes["整 crate"].get("success") is True and shapes["整 crate"].get("is-verifying-entire-crate") is True and shapes["整 crate"].get("verified") == 5,
                   f"假 Verus 的输出不像真 Verus：{shapes}")
        got, output, verdicts = run_items("proof", items_root)
        want_items = ["proof:Pair::clone", "proof:sum_to", "proof:lemma_within_itself", "proof:count_up"]
        # spec fn 不带 decreases 的不成项（开关 spec-fn-as-item 下 within 成项、验出 0 条判红，这一格判错）
        judge_cell("proof-spec-fn-not-item", got == 0 and list(verdicts) == want_items and "不成项的 spec fn 1 个" in output,
                   f"期望退 0、逐项 {want_items}、报不成项的 spec fn 1 个，实测退 {got}、{list(verdicts)}：{output.strip()[-300:]}")
        # proof fn 按函数验出 verified 1（JSON 里没有 success 键）判绿（开关 require-success-key-per-function 下判错）
        judge_cell("proof-fn-verified-one-green", verdicts.get("proof:lemma_within_itself") == ("绿", "现判") and "proof:lemma_within_itself：绿" in output,
                   f"期望 proof:lemma_within_itself 现判判绿，实测 {verdicts}：{output.strip()[-300:]}")
        # 按函数验时 is-verifying-entire-crate 报 false 不判红（开关 require-entire-crate-per-function 下判错）
        judge_cell("proof-entire-crate-false-not-red", got == 0 and all(verdict == "绿" for verdict, _how in verdicts.values()) and "is-verifying-entire-crate" not in output,
                   f"期望逐项都绿，实测退 {got}、{verdicts}：{output.strip()[-300:]}")
        # 带 decreases 的 spec fn 有终止义务，成项（开关 spec-decreases-not-item 下判错）
        judge_cell("proof-spec-decreases-item", verdicts.get("proof:sum_to", ("",))[0] == "绿" and "带 decreases 的 spec fn 1" in output,
                   f"期望 proof:sum_to 成项判绿，实测 {verdicts}：{output.strip()[-300:]}")
        # verus! 里 derive(Clone) 的类型：Verus 验它生成的 <类型>::clone，成项（开关 derived-clone-not-item 下判错）
        judge_cell("proof-derived-clone-item", verdicts.get("proof:Pair::clone", ("",))[0] == "绿" and "derive(Clone) 生成的 clone 1" in output,
                   f"期望 proof:Pair::clone 成项判绿，实测 {verdicts}：{output.strip()[-300:]}")
        # 各项之和等于登记的数判绿（真仓的 56 那一格的小样：1 + 1 + 1 + 2 = 5）
        judge_cell("proof-sum-equals-registered-green", got == 0 and "各项 verified 之和 5、0 errors（登记 5）" in output,
                   f"期望之和 5 等于登记的 5、判绿，实测退 {got}：{output.strip()[-300:]}")
        # 之和比登记的少一条判红，点名没成项的 fn 与验出数和静态估不同的项（开关 count-unchecked 下判错）
        short_root = make_root("proofitemsshort", kernel=SELFTEST_ITEMS_KERNEL, count=6)
        got, output, _verdicts = run_items("proof", short_root)
        judge_cell("proof-sum-short-red-named", got == 1 and "各项 verified 之和 5 与登记的 6 不同（少 1 条" in output
                   and "没成项的 verus! fn（不带 decreases 的 spec fn，静态估 0 条）1 个：within" in output and "verified 与静态估不同的项：没有" in output,
                   f"期望之和 5 与登记的 6 不同判红、点名 within，实测退 {got}：{output.strip()[-400:]}")
        # 选中的函数一条义务都没验（verified 0）判红（开关 zero-verified-accepted 下判错）；--item 只点它，不比之和
        zero_root = make_root("proofzero", kernel=SELFTEST_ITEMS_KERNEL.replace("        within(value, value),\n{\n", "        within(value, value),\n{\n    // SELFTEST_ZERO_VERIFIED\n"), count=5)
        got, output, verdicts = run_items("proof", zero_root, items=["proof:lemma_within_itself"])
        judge_cell("proof-zero-verified-red", got == 1 and "Verus 在 --verify-function lemma_within_itself 上一条义务都没验（verified 0" in output,
                   f"期望 verified 0 判红，实测退 {got}、{verdicts}：{output.strip()[-300:]}")
        # errors 1 判红，红句照原样报 JSON 的几样（开关 nonzero-errors-accepted 下判错）；--item 只点它，不比之和
        broken_root = make_root("prooferrorsone", kernel=SELFTEST_KERNEL.replace("result <= limit,", "result <= limit, // SELFTEST_BROKEN"))
        got, output, verdicts = run_items("proof", broken_root, items=["proof:clamp_to_limit"])
        judge_cell("proof-errors-one-red", got == 1 and "Verus 没验过（exec fn clamp_to_limit）：verified 0、errors 1、encountered-error true、vir 错 false" in output
                   and "postcondition not satisfied" in output,
                   f"期望 errors 1 判红、红句带 JSON 的几样与报错位置，实测退 {got}、{verdicts}：{output.strip()[-300:]}")
        # list-items 给了判定库：每格报键对不上几条（底稿第二节第 9 条 ⑦）；只读库、不写库、不起 Verus。三格记过之后改 clamp_to_limit 的 ensures：
        # 证明 1 项、空真 1 处、必红世界 3 行（都改在它里面）键对不上，共 5 条（开关 key-mismatch-by-item-only 下全报键相同，判错）
        listing_store = os.path.join(work, "verdict-library-listing")
        listing_root = make_root("listingkeys", must_red=[green_row.format("A"), green_row.format("B"), green_row.format("C")])
        for check in ("proof", "vacuity", "must-red-worlds"):
            run_items(check, listing_root, verdict_library=listing_store)

        def listing_counts():
            snapshot = sorted((path, os.path.getmtime(os.path.join(path, name))) for path, _directories, names in os.walk(listing_store) for name in names)
            open(trace_path, "w").close()
            os.environ["SELFTEST_VERUS_TRACE"] = trace_path
            out, err = io.StringIO(), io.StringIO()
            try:
                with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
                    listed_exit = list_items(listing_root, listing_store, install)
            finally:
                os.environ.pop("SELFTEST_VERUS_TRACE", None)
            after = sorted((path, os.path.getmtime(os.path.join(path, name))) for path, _directories, names in os.walk(listing_store) for name in names)
            return listed_exit, err.getvalue(), snapshot == after and open(trace_path).read() == ""

        before_exit, before_text, before_untouched = listing_counts()
        edit_kernel(os.path.join(listing_root, "crates", "core", "src", "kernel.rs"), "        result <= limit,\n", "        result <= limit + 0,\n")
        after_exit, after_text, after_untouched = listing_counts()
        judge_cell("list-items-key-mismatch-count",
                   before_exit == 0 and after_exit == 0 and before_untouched and after_untouched
                   and "verus-proof：键对不上 0 条、无记录 0 条、键相同 2 条、拒算 0 条" in before_text and "键对不上共 0 条" in before_text
                   and "verus-proof：键对不上 1 条、无记录 0 条、键相同 1 条" in after_text and "verus-vacuity：键对不上 1 条、无记录 0 条、键相同 1 条" in after_text
                   and "verus-must-red-worlds：键对不上 3 条、无记录 0 条、键相同 0 条" in after_text and "键对不上共 5 条（3 格比了 7 项" in after_text,
                   f"改之前退 {before_exit}（库与 Verus 没动 {before_untouched}）：{before_text.strip()!r}；改之后退 {after_exit}（没动 {after_untouched}）：{after_text.strip()!r}")
        # 每一行的键里有这一行自己：只判过 A，B 不能拿 A 的判定
        own_store = os.path.join(work, "verdict-library-own")
        own_root = make_root("reuseown", must_red=[green_row.format("A"), survives_row.format("B")])
        run_items("must-red-worlds", own_root, verdict_library=own_store, items=["must-red:改坏A"])
        got, output, verdicts = run_items("must-red-worlds", own_root, verdict_library=own_store)
        judge_cell("reuse-row-own-key", got == 1 and verdicts.get("must-red:改注释B") == ("红", "现判") and "现判 1、复用 1" in output,
                   f"期望 A 复用、B 现判判红，实测退 {got}、{verdicts}")
        # 红的不记：同一份输入第二趟照判、照红
        red_store = os.path.join(work, "verdict-library-red")
        run_items("must-red-worlds", own_root, verdict_library=red_store)
        got, output, verdicts = run_items("must-red-worlds", own_root, verdict_library=red_store)
        judge_cell("reuse-red-not-recorded", got == 1 and verdicts.get("must-red:改注释B") == ("红", "现判"), f"第二趟期望 B 照判照红，实测退 {got}、{verdicts}")
        # 一项判完当场落盘：第一趟 C 那一处被外面 kill -9（不在键里的环境），A、B 已记；第二趟同一条命令只判 C
        resume_store = os.path.join(work, "verdict-library-resume")
        resume_root = make_root("resume", must_red=[green_row.format("A"), green_row.format("B"), green_row.format("C SELFTEST_KILLME")])
        first, _output, _verdicts = run_items("must-red-worlds", resume_root, environment={"SELFTEST_KILL_MARK": "SELFTEST_KILLME"}, verdict_library=resume_store)
        got, output, verdicts = run_items("must-red-worlds", resume_root, verdict_library=resume_store)
        judge_cell("reuse-resume-after-kill", first == 1 and got == 0 and "现判 1、复用 2" in output, f"第一趟退 {first}，第二趟退 {got}：{output.strip()[-300:]}")
        # 必红世界表不进共用键：改表里一行的替换文只现判那一行，证明与空真全复用；表头加一行注释一项都不现判；核 crate 里别的文件改了三格全现判
        table_store = os.path.join(work, "verdict-library-table")
        table_root = make_root("reusetable", must_red=[green_row.format("A"), green_row.format("B"), green_row.format("C")])
        table_cells = ("proof", "vacuity", "must-red-worlds")
        for check in table_cells:
            run_items(check, table_root, verdict_library=table_store)
        table_path = os.path.join(table_root, "crates", "core", "verus-must-red-worlds.tsv")
        with open(table_path, encoding="utf-8") as handle:
            table_text = handle.read()
        with open(table_path, "w", encoding="utf-8") as handle:
            handle.write(table_text.replace("// SELFTEST_BROKEN B", "// SELFTEST_BROKEN B 换了替换文"))
        after = {check: run_items(check, table_root, verdict_library=table_store) for check in table_cells}
        judge_cell("reuse-must-red-table-row-only",
                   all(got == 0 for got, _output, _verdicts in after.values())
                   and "现判 1、复用 2（现判的原因：这一行 变了 1）" in after["must-red-worlds"][1] and after["must-red-worlds"][2].get("must-red:改坏B") == ("绿", "现判")
                   and "现判 0、复用 2" in after["proof"][1] and "现判 0、复用 2" in after["vacuity"][1],
                   "期望必红世界只现判 B、证明与空真全复用，实测 " + "；".join(f"{check} 退 {got}：{output.strip()[-200:]}" for check, (got, output, _verdicts) in after.items()))
        with open(table_path, encoding="utf-8") as handle:
            table_text = handle.read()
        with open(table_path, "w", encoding="utf-8") as handle:
            handle.write("# 表头另加的一行注释\n\n" + table_text)
        after = {check: run_items(check, table_root, verdict_library=table_store) for check in table_cells}
        judge_cell("reuse-must-red-table-comment-only",
                   all(got == 0 for got, _output, _verdicts in after.values())
                   and "现判 0、复用 3" in after["must-red-worlds"][1] and "现判 0、复用 2" in after["proof"][1] and "现判 0、复用 2" in after["vacuity"][1],
                   "期望三格全复用，实测 " + "；".join(f"{check} 退 {got}：{output.strip()[-200:]}" for check, (got, output, _verdicts) in after.items()))
        with open(os.path.join(table_root, "crates", "core", "src", "lib.rs"), "a", encoding="utf-8") as handle:
            handle.write("\n")
        after = {check: run_items(check, table_root, verdict_library=table_store) for check in table_cells}
        # 核 crate 里别的文件加一个空行：三格一项都不现判（旧判法按核 crate 整个目录取原文，全现判；开关 reuse-key-whole-crate 下判错）
        judge_cell("reuse-crate-other-file-no-cells",
                   all(got == 0 for got, _output, _verdicts in after.values())
                   and "现判 0、复用 3" in after["must-red-worlds"][1] and "现判 0、复用 2" in after["proof"][1] and "现判 0、复用 2" in after["vacuity"][1],
                   "期望三格全复用，实测 " + "；".join(f"{check} 退 {got}：{output.strip()[-200:]}" for check, (got, output, _verdicts) in after.items()))
        # 复用三问（格名 reuse-three-questions，判法与规范输出在 scripts/gate/lib_reuse_checks.py 文件头「一」）：三格（证明、空真、必红世界）的项一起算，
        # 头一趟（空库）、同一份输入第二趟、被验函数体里只加一行注释（闭包外：注释不进单元指纹）、clamp_to_limit 的 ensures 改一个字（闭包内）各跑一趟；
        # affected 是闭包含 clamp_to_limit 的五项（证明与空真各一，必红世界三行改的都在它里面），keep_small 那两项要复用。开着复用哨兵（种子 5、哨兵 1 条），
        # 抽中强制现判的那一条按复用算。已有同义的格在这一格里点名、不另留：①＝reuse-first-run-records、reuse-second-run-reuses；
        # ②＝reuse-comment-in-function-body、reuse-crate-other-file-no-cells；③＝reuse-proof-function-only、reuse-spec-function-all-users
        # （开关 reuse-key-whole-crate 下 ② 现判、reuse-key-without-units 下 ③ 少现判、reuse-lookup-skipped 下 ① 全现判，这一格都判错）
        three_store = os.path.join(work, "verdict-library-three-questions")
        three_root = make_root("reusethree", must_red=[green_row.format("A"), green_row.format("B"), green_row.format("C")])
        three_kernel = os.path.join(three_root, "crates", "core", "src", "kernel.rs")
        sentinel_environment = {reuse_checks.SENTINEL_COUNT_VARIABLE: "1", reuse_checks.SENTINEL_SEED_VARIABLE: "5"}

        def three_round():
            states, exits = {}, []
            for check in ("proof", "vacuity", "must-red-worlds"):
                got, output, _verdicts = run_items(check, three_root, environment=sentinel_environment, verdict_library=three_store)
                exits.append(got)
                states.update(reuse_checks.states_from_output(output))
            return states, exits

        rounds = [three_round()]
        rounds.append(three_round())
        edit_kernel(three_kernel, "    if value > limit { limit } else { value }\n", "    // 函数体里只加一行注释\n    if value > limit { limit } else { value }\n")
        rounds.append(three_round())
        edit_kernel(three_kernel, "        result <= limit,\n", "        result <= limit + 0,\n")
        rounds.append(three_round())
        three_affected = ["proof:clamp_to_limit", "vacuity:clamp_to_limit#1", "must-red:改坏A", "must-red:改坏B", "must-red:改坏C"]
        three_passed, three_problems = reuse_checks.report_three_questions(SCRIPT_KEY, *[states for states, _exits in rounds], three_affected,
                                                                           sentinels=3)
        three_exits = [got for _states, exits in rounds for got in exits]
        judge_cell(reuse_checks.CELL_NAME, three_passed and all(got == 0 for got in three_exits),
                   f"四趟各格退 {three_exits}；{'；'.join(three_problems)}")
        # 复用哨兵（reuse-sentinel，lib_reuse_checks.py 文件头「二」）：上面四趟之后库里三行必红世界都有判绿记录；同一个种子照常跑判绿、抽中的那一条报「强制现判」，
        # 开着弄坏开关 reuse-lookup-skipped（一律不读库）跑，哨兵要判红「复用失灵」（共用库的开关 REUSE_CHECKS_BREAK=rejudged-sentinel-passes 下这一格判错）
        got_green, output_green, _verdicts = run_items("must-red-worlds", three_root, environment=sentinel_environment, verdict_library=three_store)
        skipped_added = "reuse-lookup-skipped" not in BREAK
        BREAK.add("reuse-lookup-skipped")
        try:
            got_red, output_red, _verdicts = run_items("must-red-worlds", three_root, environment=sentinel_environment, verdict_library=three_store)
        finally:
            if skipped_added:
                BREAK.discard("reuse-lookup-skipped")
        judge_cell(reuse_checks.SENTINEL_CELL_NAME, got_green == 0 and "复用哨兵（verus-must-red-worlds）：种子 5" in output_green
                   and "，强制现判（复用哨兵抽中" in output_green and got_red == 1 and "复用失灵：" in output_red,
                   f"照常那一趟退 {got_green}：{output_green.strip()[-300:]}；不读库那一趟退 {got_red}：{output_red.strip()[-300:]}")
        # 剪枝自证：小输入上开着复用与关掉复用，逐项判定相同；输入没变（全复用）与改了一行（只现判那一行、B 转红）各比一次
        prune_store = os.path.join(work, "verdict-library-prune")
        prune_root = make_root("prune", must_red=[green_row.format("A"), green_row.format("B"), green_row.format("C")])
        for check in ("proof", "vacuity", "must-red-worlds"):
            run_items(check, prune_root, verdict_library=prune_store)
        on = {check: run_items(check, prune_root, verdict_library=prune_store)[2] for check in ("proof", "vacuity", "must-red-worlds")}
        off = {check: run_items(check, prune_root)[2] for check in ("proof", "vacuity", "must-red-worlds")}
        judge_cell("prune-unchanged-input", {check: {item: verdict for item, (verdict, _how) in items.items()} for check, items in on.items()}
                   == {check: {item: verdict for item, (verdict, _how) in items.items()} for check, items in off.items()} and len(on["must-red-worlds"]) == 3,
                   f"开着复用 {on}，关掉 {off}")
        table_path = os.path.join(prune_root, "crates", "core", "verus-must-red-worlds.tsv")
        with open(table_path, encoding="utf-8") as handle:
            table_text = handle.read()
        with open(table_path + ".new", "w", encoding="utf-8") as handle:
            handle.write(table_text.replace("// SELFTEST_BROKEN B", "// 只是注释 B"))
        os.replace(table_path + ".new", table_path)
        on = run_items("must-red-worlds", prune_root, verdict_library=prune_store)[2]
        off = run_items("must-red-worlds", prune_root)[2]
        judge_cell("prune-after-change", {item: verdict for item, (verdict, _how) in on.items()} == {item: verdict for item, (verdict, _how) in off.items()}
                   and off.get("must-red:改坏B", ("?",))[0] == "红", f"开着复用 {on}，关掉 {off}")
        # 内存包装自己退 250（撞了自己的上限）：那一处判「没跑成」，出路照包装文件头，不指向证明或必红世界表（开关 wrapper-exit-as-proof-red 下这两格判错）
        fake_wrapper = os.path.join(work, "fake-memory-cap-wrapper.sh")
        with open(fake_wrapper, "w", encoding="utf-8") as handle:
            handle.write("#!/usr/bin/env bash\n# 自证用的假内存包装：什么都不起，退 250（撞了这一条自己的上限）\nexit 250\n")
        saved_runner, saved_launcher = MEMORY_CAP_RUNNER, options["launcher"]
        MEMORY_CAP_RUNNER, options["launcher"] = fake_wrapper, None
        try:
            expect("proof-wrapper-exit", "proof", green, 1, "没跑成：内存包装 scripts/hosts/run-with-memory-cap.sh 退 250")
            expect("must-red-wrapper-exit", "must-red-worlds", green, 1, "→ 怎么办：照 scripts/hosts/run-with-memory-cap.sh 文件头「退出码」一段办")
        finally:
            MEMORY_CAP_RUNNER, options["launcher"] = saved_runner, saved_launcher
        # 内存包装退 245（显存不够）：同样判「没跑成」、点名显存不够，不报成证明的问题（开关 wrapper-exit-as-proof-red 下这一格同样判错）
        gpu_wrapper = os.path.join(work, "gpu-memory-cap-wrapper.sh")
        with open(gpu_wrapper, "w", encoding="utf-8") as handle:
            handle.write("#!/usr/bin/env bash\n# 自证用的假内存包装：什么都不起，退 245（显存不够）\nexit 245\n")
        saved_runner, saved_launcher = MEMORY_CAP_RUNNER, options["launcher"]
        MEMORY_CAP_RUNNER, options["launcher"] = gpu_wrapper, None
        try:
            expect("proof-wrapper-245", "proof", green, 1, "没跑成：内存包装 scripts/hosts/run-with-memory-cap.sh 退 245（显存不够")
        finally:
            MEMORY_CAP_RUNNER, options["launcher"] = saved_runner, saved_launcher
        # 经包装起、命令被外面 kill -9（包装退 137，不在 250–254 里）：判红，✗ 写 Verus 被信号停了，出路照包装文件头（开关 killed-as-proof-red 下判错）
        killed_wrapper = os.path.join(work, "killed-memory-cap-wrapper.sh")
        with open(killed_wrapper, "w", encoding="utf-8") as handle:
            handle.write("#!/usr/bin/env bash\n# 自证用的假内存包装：什么都不起，退 137（命令被外面 kill -9）\nexit 137\n")
        MEMORY_CAP_RUNNER, options["launcher"] = killed_wrapper, None
        try:
            killed = expect("proof-killed", "proof", green, 1, ("Verus 被信号停了（退 137，信号 9）", "→ 怎么办：照 scripts/hosts/run-with-memory-cap.sh 文件头「退出码」一段办"))
            cases += 1
            if "照 Verus 报错的位置修" in killed or "看上面 Verus 的报错" in killed:
                wrong.append("proof-killed-no-proof-howto")
                print(f"    proof-killed-no-proof-howto 这一格判错：被信号停了的那一条不该给修证明的出路：{killed.strip()[-300:]}")  # gate-lint:detail
        finally:
            MEMORY_CAP_RUNNER, options["launcher"] = saved_runner, saved_launcher
        # 假包装按副本源码里的标记出结局：SELFTEST_WRAPPER_QUEUED 照排队等满那样、SELFTEST_WRAPPER_TOO_BIG 照要的量比总上限还大那样往 stderr 打自己的 ✗ 与 →、退 252；
        # 都没有就剥掉上限与 capped.sh 照样起那条命令（nice 里的假 Verus）
        marking_wrapper = os.path.join(work, "marking-memory-cap-wrapper.sh")
        with open(marking_wrapper, "w", encoding="utf-8") as handle:
            handle.write("#!/usr/bin/env bash\n# 自证用的假内存包装（见上一行注释）\n"
                         "if grep -rqs SELFTEST_WRAPPER_QUEUED src; then\n"
                         "  echo '  ✗ 内存不够排不上：等了 0 秒（上限 RUN_WITH_MEMORY_CAP_WAIT_SECONDS=0），slice 已占 3.9 GiB；命令没跑。账上占着的：' >&2\n"
                         "  echo '  → 怎么办：等上面那几条跑完再跑这一条' >&2\n  exit 252\nfi\n"
                         "if grep -rqs SELFTEST_WRAPPER_TOO_BIG src; then\n"
                         "  echo '  ✗ 内存不够排不上：这一条要 9.0 GiB（上限），比 slice 的总上限 4.0 GiB 还大，等多久都放不下；命令没跑' >&2\n"
                         "  echo '  → 怎么办：上限写小一点再跑' >&2\n  exit 252\nfi\n"
                         "shift 4\nexec \"$@\"\n")
        MEMORY_CAP_RUNNER, options["launcher"] = marking_wrapper, None
        try:
            # 一格里一行没跑成（包装退 252）、一行改坏了照样验过（真红）：这一格判红，两条都报，末尾另打一行按红记（开关 not-run-masks-red 下判错）；
            # 没跑成那一条原样带着包装自己打的 ✗ 与 →（开关 wrapper-output-dropped 下 must-red-wrapper-lines-carried 判错）
            mixed = make_root("mixed", must_red=[
                "改注释\tcrates/core/src/kernel.rs\tif value > limit { limit } else { value }\tif value > limit { limit } else { value } // 只是注释\tproof_errors=1",
                "这一行排不上\tcrates/core/src/kernel.rs\t            candidate <= 10,\t            candidate <= 10, // SELFTEST_WRAPPER_QUEUED\tproof_errors=1"])
            expect("must-red-mixed-not-run-and-red", "must-red-worlds", mixed, 1,
                   ("改坏之后 Verus 照样验过", "这一格 1 条真红、1 条没跑成：按红记", "252 是排队等满还放不下"))
            expect("must-red-wrapper-lines-carried", "must-red-worlds", mixed, 1, "包装自己打的：「✗ 内存不够排不上：等了 0 秒")
            # 252 的另一种成因（要的量比总上限还大）给「上限写小」，不给「空出来再重跑」（开关 wrapper-252-one-howto 下判错）；整格只有没跑成，不打按红记那一行
            too_big = expect("proof-wrapper-too-big", "proof", make_root("toobig", kernel=SELFTEST_KERNEL + "// SELFTEST_WRAPPER_TOO_BIG\n"), 1,
                             ("没跑成：内存包装 scripts/hosts/run-with-memory-cap.sh 退 252", "等多久都放不下：把 GATE_VERUS_MEMORY_MAX"))
            cases += 1
            if "按红记" in too_big:
                wrong.append("proof-wrapper-too-big-only-not-run")
                print(f"    proof-wrapper-too-big-only-not-run 这一格判错：整格只有没跑成，不该打按红记那一行：{too_big.strip()[-300:]}")  # gate-lint:detail
        finally:
            MEMORY_CAP_RUNNER, options["launcher"] = saved_runner, saved_launcher
        # tool-versions 换了 z3、设了 VERUS_Z3_PATH 打的那一行都要变（开关 tool-versions-launcher-only 下这一格判错）
        import io
        import contextlib

        def tool_versions_line():
            buffer = io.StringIO()
            with contextlib.redirect_stdout(buffer):
                tool_versions(green, install)
            return buffer.getvalue()

        cases += 1
        saved_z3_path = os.environ.pop("VERUS_Z3_PATH", None)
        try:
            without_z3 = tool_versions_line()
            with open(os.path.join(install, "z3"), "w", encoding="utf-8") as handle:
                handle.write("# 自证用的假 z3\n")
            with_z3 = tool_versions_line()
            os.environ["VERUS_Z3_PATH"] = zip_path
            with_override = tool_versions_line()
        finally:
            os.environ.pop("VERUS_Z3_PATH", None)
            if saved_z3_path is not None:
                os.environ["VERUS_Z3_PATH"] = saved_z3_path
        if len({without_z3, with_z3, with_override}) != 3:
            wrong.append("tool-versions-z3")
            print(f"    tool-versions-z3 这一格判错：换 z3、设 VERUS_Z3_PATH 之后打的行应当各不相同，实测：{without_z3.strip()} | {with_z3.strip()} | {with_override.strip()}")  # gate-lint:detail
        # 安装目录顶层多一份五样之外的文件（宏库）、VERUS_Z3_PATH 没设与设成空串、裸名 z3 而 PATH 里换一份：打的行都要变
        # （开关 tool-versions-five-files-only、tool-versions-empty-z3-path-as-unset、tool-versions-bare-z3-from-current-directory 下各自那一格判错）
        saved_path = os.environ.get("PATH", "")
        saved_z3_path = os.environ.pop("VERUS_Z3_PATH", None)
        z3_lines = {}
        try:
            before_macros = tool_versions_line()
            with open(os.path.join(install, "libverus_builtin_macros.so"), "wb") as handle:
                handle.write(b"selftest macros")
            z3_lines["install-top-level"] = (before_macros, tool_versions_line())
            unset_line = tool_versions_line()
            os.environ["VERUS_Z3_PATH"] = ""
            z3_lines["z3-path-empty"] = (unset_line, tool_versions_line())
            bare_lines = []
            for variant in ("a", "b"):
                directory = os.path.join(work, f"path-z3-{variant}")
                os.makedirs(directory)
                with open(os.path.join(directory, "z3"), "w", encoding="utf-8") as handle:
                    handle.write(f"#!/bin/sh\n# 自证用的 PATH 里的 z3（{variant}）\n")
                os.chmod(os.path.join(directory, "z3"), 0o755)
                os.environ["VERUS_Z3_PATH"] = "z3"
                os.environ["PATH"] = directory + os.pathsep + saved_path
                bare_lines.append(tool_versions_line())
            z3_lines["z3-bare-name"] = tuple(bare_lines)
        finally:
            os.environ["PATH"] = saved_path
            os.environ.pop("VERUS_Z3_PATH", None)
            if saved_z3_path is not None:
                os.environ["VERUS_Z3_PATH"] = saved_z3_path
        for name, (first, second) in z3_lines.items():
            cases += 1
            if first == second:
                wrong.append(f"tool-versions-{name}")
                print(f"    tool-versions-{name} 这一格判错：前后两种环境打的行应当不同，实测都是：{first.strip()}")  # gate-lint:detail
        # VERUS_ROOT 与 VERUS_USE_RUSTUP：没设、空串、换值、VERUS_ROOT 指的目录换了一份文件，打的行两两不同（开关 tool-versions-verus-environment-ignored 下判错）
        cases += 1
        saved_environment = {variable: os.environ.pop(variable, None) for variable in ("VERUS_ROOT", "VERUS_USE_RUSTUP")}
        verus_root_copy = os.path.join(work, "verus-root-copy")
        os.makedirs(verus_root_copy)
        with open(os.path.join(verus_root_copy, "libverus_builtin_macros.so"), "wb") as handle:
            handle.write(b"selftest macros copy")
        environment_lines = {}
        try:
            environment_lines["两个都没设"] = tool_versions_line()
            os.environ["VERUS_ROOT"] = ""
            environment_lines["VERUS_ROOT 空串"] = tool_versions_line()
            os.environ["VERUS_ROOT"] = verus_root_copy
            environment_lines["VERUS_ROOT 指到一份拷贝"] = tool_versions_line()
            with open(os.path.join(verus_root_copy, "libverus_builtin_macros.so"), "wb") as handle:
                handle.write(b"selftest macros copy, truncated")
            environment_lines["VERUS_ROOT 指的拷贝换了宏库"] = tool_versions_line()
            os.environ.pop("VERUS_ROOT")
            os.environ["VERUS_USE_RUSTUP"] = ""
            environment_lines["VERUS_USE_RUSTUP 空串"] = tool_versions_line()
            os.environ["VERUS_USE_RUSTUP"] = "0"
            environment_lines["VERUS_USE_RUSTUP=0"] = tool_versions_line()
        finally:
            for variable, value in saved_environment.items():
                os.environ.pop(variable, None)
                if value is not None:
                    os.environ[variable] = value
        if len(set(environment_lines.values())) != len(environment_lines):
            wrong.append("tool-versions-verus-environment")
            print(f"    tool-versions-verus-environment 这一格判错：{len(environment_lines)} 种环境打的行应当两两不同，实测只有 {len(set(environment_lines.values()))} 种："  # gate-lint:detail
                  + " | ".join(f"{label}：{line.strip()[-120:]}" for label, line in environment_lines.items()))
        # rustup 工具链（C632）：假的 RUSTUP_HOME 里 toolchains/<version.json 的 toolchain> 的 lib/ 与 lib/rustlib/<三元组>/lib/ 各换一个文件、
        # RUSTUP_HOME 指到另一份同内容的拷贝、PATH 里换一份 rustup，打的行两两不同（开关 tool-versions-rustup-ignored 下判错）
        cases += 1
        saved_rustup_home = os.environ.pop("RUSTUP_HOME", None)
        rustup_homes = [os.path.join(work, f"rustup-home-{variant}") for variant in ("a", "b")]
        driver_relative = os.path.join("toolchains", "stable", "lib", "librustc_driver-selftest.so")
        core_relative = os.path.join("toolchains", "stable", "lib", "rustlib", "x86_64-unknown-linux-gnu", "lib", "libcore-selftest.rmeta")
        for home in rustup_homes:
            for relative, content in ((driver_relative, b"selftest driver\x00"), (core_relative, b"selftest core rmeta")):
                os.makedirs(os.path.dirname(os.path.join(home, relative)), exist_ok=True)
                with open(os.path.join(home, relative), "wb") as handle:
                    handle.write(content)
        rustup_lines = {}
        try:
            for variant in ("a", "b"):
                directory = os.path.join(work, f"path-rustup-{variant}")
                os.makedirs(directory)
                with open(os.path.join(directory, "rustup"), "w", encoding="utf-8") as handle:
                    handle.write(f"#!/bin/sh\n# 自证用的 PATH 里的 rustup（{variant}）\n")
                os.chmod(os.path.join(directory, "rustup"), 0o755)
            os.environ["PATH"] = os.path.join(work, "path-rustup-a") + os.pathsep + saved_path
            os.environ["RUSTUP_HOME"] = rustup_homes[0]
            rustup_lines["拷贝 a"] = tool_versions_line()
            with open(os.path.join(rustup_homes[0], driver_relative), "r+b") as handle:
                handle.seek(-1, 2)
                handle.write(b"\xff")
            rustup_lines["拷贝 a 的 librustc_driver 末尾 1 字节改了"] = tool_versions_line()
            with open(os.path.join(rustup_homes[0], core_relative), "ab") as handle:
                handle.write(b" changed")
            rustup_lines["拷贝 a 的 libcore rmeta 也改了"] = tool_versions_line()
            os.environ["RUSTUP_HOME"] = rustup_homes[1]
            rustup_lines["RUSTUP_HOME 指到同内容的拷贝 b"] = tool_versions_line()
            os.environ["PATH"] = os.path.join(work, "path-rustup-b") + os.pathsep + saved_path
            rustup_lines["PATH 里换一份 rustup"] = tool_versions_line()
        finally:
            os.environ["PATH"] = saved_path
            os.environ.pop("RUSTUP_HOME", None)
            if saved_rustup_home is not None:
                os.environ["RUSTUP_HOME"] = saved_rustup_home
        if len(set(rustup_lines.values())) != len(rustup_lines):
            wrong.append("tool-versions-rustup-toolchain")
            print(f"    tool-versions-rustup-toolchain 这一格判错：{len(rustup_lines)} 种 rustup 工具链与 rustup 打的行应当两两不同，实测只有 {len(set(rustup_lines.values()))} 种："  # gate-lint:detail
                  + " | ".join(f"{label}：{line.strip()[-160:]}" for label, line in rustup_lines.items()))
        # 内存包装排不上（C631）：假包装一律睡 queue_wait 秒（照等满等待上限）再打 ✗ 与 →、退 252，每被起一次记一行；源码里有 SELFTEST_WRAPPER_ADMITTED 的
        # 睡 2 倍再照样起那条命令。一处拿到 252 之后，这一格与后面几格还没起的都不再排队，已经起了的照常收（开关 queue-refused-keeps-queueing 下这几格判错）
        queue_wait = 0.5
        queue_count = os.path.join(work, "refusing-wrapper-count")
        refusing_wrapper = os.path.join(work, "refusing-memory-cap-wrapper.sh")
        with open(refusing_wrapper, "w", encoding="utf-8") as handle:
            handle.write("#!/usr/bin/env bash\n# 自证用的假内存包装（见上面两行注释）\n"
                         f"echo queued >> '{queue_count}'\n"
                         f"if grep -rqs SELFTEST_WRAPPER_ADMITTED src; then sleep {2 * queue_wait}; shift 4; exec \"$@\"; fi\n"
                         f"sleep {queue_wait}\n"
                         f"echo '  ✗ 内存不够排不上：等了 {queue_wait} 秒（自证的假包装），slice 已占满；命令没跑' >&2\n"
                         "echo '  → 怎么办：等上面那几条跑完再跑这一条' >&2\nexit 252\n")
        queue_kernel = SELFTEST_KERNEL + "".join(f"// queue anchor {index:03d}\n" for index in range(94))
        queue_rows = [f"排队 {index:03d}\tcrates/core/src/kernel.rs\t// queue anchor {index:03d}\t// queue changed {index:03d}\tproof_errors=1" for index in range(94)]
        queue_root = make_root("queuerefused", kernel=queue_kernel, must_red=queue_rows)
        collected_root = make_root("queuecollected", kernel=queue_kernel, must_red=[
            "起了的照收\tcrates/core/src/kernel.rs\tif value > limit { limit } else { value }\tif value > limit { limit } else { value } // 只是注释 SELFTEST_WRAPPER_ADMITTED\tproof_errors=1"]
            + queue_rows[:9])

        def queue_counted():
            return len(open(queue_count, encoding="utf-8").read().split()) if os.path.exists(queue_count) else 0

        def timed_cells(label, checks, most_wrapper_runs, time_limited=True):
            """逐个跑 checks（[(标签, 格, 根, 期望的退出码, 期望含的字)]），包装至多被起 most_wrapper_runs 次、time_limited 时整串的挂钟不超过
            一次等待上限加 2.5 秒（照排的话 94 行 8 处同时是 12 次等待），否则 label 判错。"""
            nonlocal cases
            if os.path.exists(queue_count):
                os.remove(queue_count)
            started = time.monotonic()
            for check in checks:
                expect(*check)
            elapsed, runs = time.monotonic() - started, queue_counted()
            cases += 1
            if (time_limited and elapsed > queue_wait + 2.5) or not 1 <= runs <= most_wrapper_runs:
                wrong.append(label)
                print(f"    {label} 这一格判错：应当只排一次（包装 1 到 {most_wrapper_runs} 次"  # gate-lint:detail
                      f"{f'、挂钟不超过 {queue_wait + 2.5:.1f} 秒' if time_limited else ''}），实测挂钟 {elapsed:.1f} 秒、包装被起 {runs} 次")

        saved_jobs = options["jobs"]
        refused_file = os.path.join(work, "queue-refused.txt")
        MEMORY_CAP_RUNNER, options["launcher"] = refusing_wrapper, None
        try:
            # 一格 94 行、8 处同时验：头一批 8 处排不上之后，其余 86 处不起包装，并成一条 ✗ 逐个列出
            options["jobs"] = 8
            timed_cells("queue-refused-must-red-once", [
                ("queue-refused-must-red-skipped-listed", "must-red-worlds", queue_root, 1,
                 (QUEUE_SKIPPED_MARK, "这一格 86 处 Verus 没起", "第 95 行「排队 093」", "252 是排队等满还放不下"))], 8)
            # 已经起了的照常收：两处同时起，一处被放行（2 倍等待之后出结局、改坏了照样验过 = 真红），一处排不上；其余 8 处不起，整格按红记
            options["jobs"] = 2
            timed_cells("queue-refused-started-rows-once", [
                ("queue-refused-started-rows-collected", "must-red-worlds", collected_root, 1,
                 ("改坏之后 Verus 照样验过", "这一格 8 处 Verus 没起", "这一格 1 条真红、9 条没跑成：按红记"))], 2, time_limited=False)
            # 跨格：门禁给的文件里记下 proof 格排不上，后面空真与必红世界两格一处都不起包装
            options["jobs"] = 8
            open(refused_file, "w").close()
            os.environ[QUEUE_REFUSED_FILE_VARIABLE] = refused_file
            timed_cells("queue-refused-later-cells-once", [
                ("queue-refused-proof-not-run", "proof", green, 1, f"{NOT_RUN_MARK}252"),
                ("queue-refused-vacuity-skipped", "vacuity", green, 1, (QUEUE_SKIPPED_MARK, "proof 格「proof:", "这一格 2 处 Verus 没起")),
                ("queue-refused-must-red-all-skipped", "must-red-worlds", queue_root, 1, ("这一格 94 处 Verus 没起", "252 是排队等满还放不下"))], 2)
        finally:
            os.environ.pop(QUEUE_REFUSED_FILE_VARIABLE, None)
            MEMORY_CAP_RUNNER, options["launcher"], options["jobs"] = saved_runner, saved_launcher, saved_jobs

    # 包装排队报的那一行边跑边转打到 stderr（带是哪一格）：假包装报一行排队，等它出现在 sink 里（至多 10 秒，等不到退 9）；判定用的输出不变
    with tempfile.TemporaryDirectory(prefix="verus-proof-check-queue-line-") as queue_work:
        queueing_wrapper = os.path.join(queue_work, "queueing-wrapper.sh")
        with open(queueing_wrapper, "w", encoding="utf-8") as handle:
            handle.write("echo 'run-with-memory-cap: 排队（已等 60 秒，至多 3600 秒）：自证造的一行' >&2\n"
                         "for _ in $(seq 1 200); do\n"
                         "  if grep -q '自证造的一行' \"$QUEUE_LINE_SINK\" 2>/dev/null; then echo '{}'; echo 'Verus 自己的 stderr' >&2; exit 0; fi\n"
                         "  sleep 0.05\n"
                         "done\n"
                         "exit 9\n")
        sink_path = os.path.join(queue_work, "sink.txt")
        with open(sink_path, "w", encoding="utf-8", buffering=1) as sink:
            queued = run_forwarding_queue_lines(["bash", queueing_wrapper], "proof", sink=sink, env=dict(os.environ, QUEUE_LINE_SINK=sink_path))
        with open(sink_path, encoding="utf-8") as handle:
            forwarded = handle.read()
    cases += 1
    if queued != (0, "{}\n", "run-with-memory-cap: 排队（已等 60 秒，至多 3600 秒）：自证造的一行\nVerus 自己的 stderr\n") \
            or forwarded != "  … verus-proof-check proof：run-with-memory-cap: 排队（已等 60 秒，至多 3600 秒）：自证造的一行\n":
        wrong.append("wrapper-queue-line-forwarded")
        print(f"    wrapper-queue-line-forwarded 这一格判错：交回 {queued!r}（退 9 是假包装等了 10 秒没见到转打），转打出来的 {forwarded!r}")  # gate-lint:detail

    # --threads 的缺省：没设 SINGLEFS_THREAD_CAP 取登记文件第一条非注释行的数，设了取它
    with open(os.path.join(SCRIPT_DIRECTORY, "..", "..", ".claude", "heavy-thread-default"), encoding="utf-8") as handle:
        registered_default = next((int(line.split()[0]) for line in handle if line.strip() and not line.lstrip().startswith("#")), None)
    cases += 1
    unset_threads, capped_threads = default_threads({}), default_threads({"SINGLEFS_THREAD_CAP": "7"})
    if registered_default is None or unset_threads != registered_default or capped_threads != 7:
        wrong.append("threads-default-registered")
        print(f"    threads-default-registered 这一格判错：登记 {registered_default}，没设上限得 {unset_threads}，上限 7 得 {capped_threads}")  # gate-lint:detail

    breaks_missed, breaks_proven = [], 0
    if not wrong and not BREAK and not reuse_checks.BREAK:
        items = ("commit-unchecked", "fingerprint-unchecked", "absent-kernel-not-run", "cheating-unchecked", "count-unchecked", "timeout-as-pass",
                     "vacuity-accepts-verified", "vacuity-all-at-once", "mutation-accepts-green", "anchor-unchecked", "rlimit-counts-as-red", "rlimit-unpinned",
                     "rlimit-any-line-voids", "rlimit-attribute-unchecked", "mutation-rlimit-attribute-accepted",
                     "expectation-form-unchecked", "proof-error-count-unchecked", "rlimit-counted-in-proof-errors",
                     "wrapper-exit-as-proof-red", "tool-versions-launcher-only", "tool-versions-five-files-only", "tool-versions-empty-z3-path-as-unset",
                     "tool-versions-bare-z3-from-current-directory", "not-run-masks-red", "wrapper-output-dropped", "wrapper-252-one-howto",
                     "wrapper-queue-lines-swallowed", "tool-versions-verus-environment-ignored", "killed-as-proof-red",
                     "tool-versions-rustup-ignored", "queue-refused-keeps-queueing",
                     "item-filter-ignored", "reuse-key-without-units", "reuse-key-whole-crate", "reuse-key-function-only",
                     "vacuity-key-by-line-number", "must-red-key-with-name-and-path", "reuse-key-with-timeout", "reuse-key-without-judging-units",
                     "reuse-key-without-toolchain", "reuse-key-without-row",
                     "reuse-ignores-key",
                     "reuse-records-red", "reuse-records-whole-cell-only", "threads-default-cpu-count", "list-items-pipe-unhandled",
                     "spec-fn-as-item", "spec-decreases-not-item", "derived-clone-not-item", "require-success-key-per-function",
                     "require-entire-crate-per-function", "zero-verified-accepted", "nonzero-errors-accepted", "key-mismatch-by-item-only",
                     "reuse-lookup-skipped", "REUSE_CHECKS_BREAK=rejudged-sentinel-passes")
        # 各开关的那一趟互不依赖、各用各的临时目录：几趟同时起（至多 SELFTEST_BREAK_WORKERS 趟），按开关的次序收退出码、数份数；
        # 写成「变量=项」的是共用库 lib_reuse_checks.py 的开关（复用哨兵被现判不红：reuse-sentinel 那一格判错）
        def run_break(item):
            variable, _equals, value = item.rpartition("=") if "=" in item else ("VERUS_PROOF_CHECK_BREAK", "=", item)
            return item, subprocess.run([sys.executable, os.path.realpath(__file__), "--selftest"], capture_output=True, text=True,
                                        env=dict(os.environ, **{variable: value})).returncode

        with concurrent.futures.ThreadPoolExecutor(max_workers=max(1, min(SELFTEST_BREAK_WORKERS, int(default_heavy_threads(os.environ))))) as pool:
            outcomes = list(pool.map(run_break, items))
        if len(outcomes) != len(items):
            breaks_missed.append(f"开关轮收回 {len(outcomes)} 趟、派出 {len(items)} 趟")
        for item, returncode in outcomes:
            if returncode == 0:
                breaks_missed.append(item)
            else:
                breaks_proven += 1
    if wrong:
        report_red(f"verus-proof-check 自证 {cases} 格里 {len(wrong)} 格判错：{' '.join(wrong)}", "按判错那几格的标签找对应的 check_* 函数修；判据写在文件头")
        return EXIT_RED
    if breaks_missed:
        report_red(f"弄坏开关打开之后自证照样全对：{' '.join(breaks_missed)}", "那几个开关管的那一段没有自证盯着：补一格会被它弄错的自证，或修开关")
        return EXIT_RED
    suffix = f"；弄坏开关 {breaks_proven} 个，每个都让至少一格判错" if not BREAK else "（开着弄坏开关，没重跑开关）"
    print(f"  ✓ verus-proof-check 自证 {cases} 格都判对{suffix}")
    return EXIT_GREEN


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
