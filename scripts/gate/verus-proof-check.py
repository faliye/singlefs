#!/usr/bin/env python3
# admission: always 每一次调都判此刻被判的仓、此刻的 Verus 安装与 cargo 缓存，上一次的结论不替这一次作保
# run-condition: none 要的 Verus 与 cargo 由各子命令自己判，缺了判红并给出路（缺工具直接红，不交给预判退 78）
"""门禁 formal-verification 的 Verus 那四格的判法（里程碑三七·5；阶段只负责把它接进门禁、给样本开关）。

被验的：登记表 .claude/gate.d/verus-pins.tsv 点名的各 crate（crate 行可以多条、各有各的 verified 登记数；今天是只依赖 vstd 的 crates/singlefs-replay-prefix，
singlefs-core 的被验区域登记成第二条）与各自里面的核（kernel 行，落在哪个 crate 的目录里就归哪个 crate；必红世界表每个 crate 一张，放在它自己的目录里）。
登记了几个 crate 时，proof、vacuity、must-red-worlds、coverage 四样逐个 crate 判（run_per_crate），--item 照各 crate 列出的项分过去；只登记一个 crate 时照一个判。
被验区域：核里 verus! { … } 包的，与属性写法（#[verus_verify]、#[verus_spec(…)]）标的条目；crate 的 src 里有这两样的文件都要登记成核。
两种起法（crate 行第四段）：
  不写第四段：Verus 每次都直接起，对 crate 的入口文件 --crate-type=lib，带 --output-json 与 --rlimit <登记的数>；只带它自带的 vstd，crate 的 Cargo.toml
    除 vstd 之外还有依赖，proof、vacuity、must-red-worlds 三格开跑之前判红。
  第四段写 cargo：经 Verus 安装里的 cargo-verus 起 cargo verus focus -p <包> --offline -- <参数>，副本是 crate 连同它经 path 连到的 crate、
    仓根的 Cargo.lock 与一份只列这几个成员的工作区 Cargo.toml（整个工作区的 cargo metadata 在离线机器上过不了）；几处同时验各占一个编译目录，vstd 与依赖在里面只编一次。
    环境里有绝对路径的 CARGO_TARGET_DIR（任务服务或整轮门禁给这一趟的编译目录）时，这些目录建在它底下的 verus-proof-check-targets/，一趟里先后几次调用共用（claim_slot_directories）、不删；
    没有时每次调用在临时目录里建、用完删。
  两种都是一处判定一份新副本：cargo 的编译指纹不看 --verify-* 参数，同一个目录里换参数再验拿到的是上一趟的结果、stdout 没有判定行。
    没有判定行的那一处判红，不当绿。
不带 --no-cheating：assume_specification、external_body、uninterp 与 external_*_specification 照信任清单
  .claude/gate.d/verus-trust-list.tsv 逐条登记（没登记、规格原文的 sha256 对不上、差分测试不在都判红），admit(…)、assume(…)、verifier::external、axiom 在核里一律判红，
  都在起 Verus 之前判（region_problems、cheating_markers）。
--rlimit 取登记表 rlimit 行；没有这一行时取 DEFAULT_RLIMIT，就是 Verus 0.2026.09.20 自己的默认 10。proof、vacuity、must-red-worlds 三格同一个数，各 crate 同一个数。
判定取 stdout 的 JSON（verification-results 的 verified / errors / encountered-vir-error，verus 的 commit），报错位置取 stderr（直接起的相对 crate 目录，cargo 那一种相对副本根）。
只验一个函数（证明逐项、空真每处）时按那个函数所在的核点模块（kernel_module）：--verify-function 只在 --verify-root 或 --verify-only-module 点的那一个模块里找。
  核是 crate 的入口文件（Cargo.toml 的 lib.path，默认 src/lib.rs）的点 --verify-root；入口所在目录下的 <名>.rs 与 <名>/mod.rs 点 --verify-only-module <名>，
  更深的用 :: 接起来（<甲>/<乙>.rs 与 <甲>/<乙>/mod.rs 是 <甲>::<乙>）。Verus 0.2026.09.20 的认法：
  模块路径认 outer::inner，不认 crate::outer、outer/inner、outer.inner（报 could not find module … specified by --verify-module or --verify-only-module）；
  --verify-module 与 --verify-function 一起给退 255（Error: When using --verify-function, use --verify-only-module instead of --verify-module）；
  --verify-root 与 --verify-only-module 一起给、两样都不给，也退 255；--verify-only-module 不含子模块，点 outer 找不到 outer::inner 里的函数。
  核不在入口所在目录下、或路径里有一段不是 Rust 标识符的，认不出模块，判红。必红世界整个 crate 验，不点函数，不点模块。

用法（<根> 是被判的仓根；四个判的子命令都退 0 绿、1 红、2 用法错、77 本次未跑）：
  verus-proof-check.py toolchain <根> [--verus-root 目录] [--cargo-home 目录] [--rustc-version-file 文件]
      版本一致：Verus 安装里 version.json 的 version 与 commit、安装目录旁 verus.zip 的 sha256（zip 在就核，不在报一句）、
      Cargo.lock 里登记的六个包各恰好一个版本且等于登记、本机 cargo 缓存里每个包的 .cargo_vcs_info.json（sha1、path_in_vcs）
      与包源码的内容指纹（fingerprint 子命令的算法），都与登记相同；Verus 与产品的 rustc 版本差钉在登记表 rustc 行（rustc_pin_problems）：
      version.json 的 toolchain 第一段、产品 rustc -vV（在被判的仓根起）的 release 与 commit-hash 都与 rustc 行相同（--rustc-version-file 给样本用：读那份文件当 rustc -vV 的输出）。
  verus-proof-check.py proof <根> [通用选项]
      核里（去掉 // 注释之后）不许出现 admit(…)、assume(…)、verifier::external / verifier(external)、
      verifier::rlimit / verifier(rlimit …)（函数上的 rlimit 属性盖过命令行的 --rlimit，那个函数就不在登记的额度上验）；
      被验区域（region_problems，起 Verus 之前判，先于切函数：按 cfg 分叉的两份同名 fn 红句先说 cfg）：axiom 一律判红；每一处可信声明（TRUST_KINDS）与 cfg 原子
      （#[cfg(…)]、#[cfg_attr(…)]、cfg!(…)，与定义里有 cfg 的 macro_rules! 宏在被验区域里的调用）在信任清单里恰好一行、规格原文 sha256 相同、差分测试在
      crates/ 下标了 #[test]，清单里这个 crate 的行在核里都找得到，vstd 恰好一行且与登记表 package vstd 行同版本同指纹；被验区域里声明的类型与 trait
      在 crate 的 src 里 verus! 外面有 impl <Trait> for <类型> 的判红（cfg(test) 下的不判；把所在文件连进来的 mod 声明带 cfg(test)、整份文件算测试代码的同样不判，
      判法在 scripts/gate/lib_verus_trait_impl_lag.py 的 in_test_only_module_file）。
      登记式放过（.claude/gate.d/verus-trait-impls-outside-lag.tsv，格式写在表头，只缩不涨）：表里一行（trait 全路径、实现类型、所在文件、哪一簇会把它挪进 verus!、
      为什么此刻放过）对上的 verus! 外面的 impl 不判红，成功句逐个点名「<trait> for <类型>（<文件>:<行>，契约没证、靠信任，归「<簇>」挪进 verus!）」、写这一格不算绿到底；
      表里这个 crate 的行，那个 impl 已挪进 verus! 或在那份文件里找不到的判红（删行）；impl 还在外面而它的 trait 与类型此刻都不在被验区域里的
      （那一簇的补丁还没打上）不红、没放过什么，成功句点名；trait 全路径与被验的那个不同判红；表与 gate 基准那一版比
      （判法与它的自证格在 scripts/gate/lib_verus_trait_impl_lag.py；.claude/gate.d/lib-lag-baseline.py 的 gate_base_of、baseline_rows），按（trait 全路径、实现类型）分组、文件照拆分族认（scripts/gate/lib_lag_split_family.py）：
      基准里点名一份入口的一行，族是它本身加按它的拆法命名的部分文件（.rs 入口加 <名>/ 下的 .rs，含 <名>/main.rs；.py、.sh 入口的三种拆法写在那一份的文件头），
      族内此刻的行数之和超过基准那一行、或文件落到族外（基准里没有这一行，同目录名字对不上拆法的文件在内）的判红，
      被判的根不在 git 仓里、基准里还没有这张表时不比、成功句写明没比，git 取不到判红。
      external_body 的规格原文取签名到函数体的 { 之前（body_open：单元是半开区间，最后一个词是 } 才有函数体），一个 impl 里最后一个 fn 也取得到签名，改签名 sha256 就变。
      uninterp、external_*_specification、带 external_body 的类型规格与 cfg 原子（规格原文是它所在的条目）取整个条目，也照半开区间 [first_token, last_token) 取，
      不连下一个条目的头一个词（whole_entry_end，几样同一种取法）。
      放过的每一处另往 $GATE_NOT_RUN_FILE 追加一行本次未跑（门禁汇总的「本次未跑」收它；这个变量不在时不写，单跑与喂样本时它不在），成功句报追加了几行。
      Verus 逐项验：核的被验区域里有证明义务的单元各一项（proof_items）：proof fn、exec fn、带 decreases 的 spec fn（终止义务）、
      derive(Clone) 的类型（Verus 验它生成的 <类型>::clone）；impl 与 trait 里的方法照样成项；不带 decreases 的 spec fn、trait 里没有函数体的声明、external_body 的 fn
      没有证明义务、不成项，随用到它的项进键（没有函数体按切出的单元认：单元以 ; 收尾，ensures 里的结构体字面量不算函数体）。另两类不是 fn、Verus 整 crate 验时照样各算条数，各成一项（里程碑四出口 5 规格底座，core 登记 16 条里有这 3 条）：
      verus! 里带值的 const（impl 里的关联 const 与 spec const 也算：真 Verus 整 crate 验时 spec const 各 1 条、点得中名；没写值的不成项）与 assume_specification。
      项名是全路径 proof:<crate>::<模块>::<名>（核是 crate 入口的没有模块那一段），
      <名> 是交给 --verify-function 的那一个：自由 fn 写它的名字，impl 里的方法写 <类型>::<方法>（trait impl 也写类型），trait 里的写 <trait>::<方法>，clone 写 <类型>::clone；
      带值的 const 项名写 <类型或 trait>::<名>（自由的写 <名>）；交给 --verify-function 的：自由 const 写 <名>，trait 里带默认值的写 <trait>::<名>，
      impl 里的关联 const（固有 impl 与 trait impl 都算）写 *::<名>（Verus 把它列成 impl&%<号>::<名>，号数这个模块里的全部 impl，verus! 外面的与 derive 生成的也数，
      从源码算不准；*::<名> 是「以 ::<名> 收尾」，不会点到以 <名> 起头的自由 const）；assume_specification 项名写 assume_specification[<点名的条目>]，交给 --verify-function 的是那个条目（只有标识符与 :: 的照写，
      例 singlefs_format::SLOT_BYTES；别的取最后一个标识符，例 <[T]>::binary_search 交 binary_search）。
      起 Verus 之前（列项也算）每一项的点名照 Verus 的点法在它那个模块的可点名函数表里点一遍（point_name_problems、verus_listing、verus_function_matches）：
      不带 * 的先找恰好同名的、没有再找包含它的，*<串> 是以它收尾、<串>* 是以它起头、*<串>* 是包含它，带 * 的点中几个 Verus 就一起验几个；
      命中不是恰好一项的判红并点名命中的那几项，出路是改名。Verus 只在 --verify-root 或 --verify-only-module 点的那一个模块里找，表按核（一个核一个模块）各列各的，
      不同模块里同名的不撞。Verus 0.2026.09.20 的点名规则：<类型>::<关联 const>、<模块>::<类型>::<名>、
      <crate>::<模块>::<类型>::<名>、crate::<模块>::<类型>::<名> 都报 could not find，只有 impl&%<号>::<名> 与 *::<名> 点得中。
      每项起一次 --verify-root 或 --verify-only-module <核所在的模块>，再 --verify-function <名>：出结局、没有 vir 错、encountered-error 不是 true、errors 0、verified 至少 1
      （assume_specification 那一项验出 0 也是绿：指向函数的规格没有义务）、跑的 Verus 的 commit 等于登记。--verify-function 下真 Verus 不给 success 键、is-verifying-entire-crate 报 false，这两样不看（verification_clean）。
      整格跑时各项 verified 之和（复用的取记下的数）等于这个 crate 登记的数（整 crate 验的数），不等判红并点名
      verified 与静态估的义务条数不同的项（proof fn 1、exec fn 1 加每个 while / loop / for 1、带 decreases 的 spec fn 1、clone 1、
      带值的 const 1、assume_specification 指向常量又写了 returns 的 1、别的 0）与没成项的 fn。
      同一个模块里两个 fn 交给 --verify-function 的名字相同（同一个类型的固有方法与 trait 方法同名）判红：Verus 按名点会把两个一起验；认不出类型的 impl 里的方法判红。
      不同模块里同名的各是各的项。整格再做一次不带过滤的 crate 验证，防止义务遗漏；再把产品构建与 Verus 构建的 MIR 逐条目比（compare_mir，不按条入库）：
      产品构建（cargo rustc --emit=mir，RUSTC_BOOTSTRAP=1 -Z trim-diagnostic-paths=false）的每个条目在 Verus 构建（verus --no-verify --compile -- --emit=mir；
      cargo 那一种 cargo verus build -- --no-verify -- --emit=mir：不再验一遍、只编，证明在前面完整 crate 那一步验过；不带 --no-verify 时 cargo verus build 连 vstd 也验，
      小 crate 上内存 1.8 GB 对 1.0 GB）里都有、整段逐字相同，对不上判红（例：宏写法产品构建返回 11、verus --compile 编出返回 12）。
      比之前两边照规整表 MIR_PATH_ALIASES 把同一个函数的两种印法写成一种（整词：前面不接标识符字符或 ::，后面接 ( 或 ::<；今天只有 std::rt::panic_fmt 写成
      core::panicking::panic_fmt），成功句与红句都报两边各改写了几处；表外的名字照红，核过是同一个函数（两份 rlib 引用的符号相同）再往表里加一行。
  verus-proof-check.py vacuity <根> [通用选项]
      空真筛查：词法枚举 requires，非独立契约行拒绝（fn 与闭包都算），核里每一处 requires 找到它的函数体的 {（其后第一行以 { 起头、缩进不深于 requires 那一行的），
      属于 assert(…) by (…) 的 requires（前面紧接 assert(…) by (…)、往后在同一层先碰到 ; 收尾，assert_by_requires_lines）不是插入点：它在那一处先证出来，不是外层函数的前提，
      外层函数入口那一处照旧插；跳过的逐处报进成功句「跳过 assert-by requires N 处：<核>:<行>…」（证明格的成功句与列项的 proof 那一行同样报）。
      往下找不到函数体的 { 又不是这种写法的（没有函数体的 fn 声明、trait 里无体的方法、挂在普通块或不带 by 的 assert 上的）照旧判红，红句带核的路径与行；
      在临时副本里 { 后面插 assert(false);，一处一份副本、一次 Verus（只验这一处所属的那个 fn：--verify-root 或 --verify-only-module <核所在的模块>，再 --verify-function）；
      每一处都要报 assertion failed 且位置就是插的那一行；失败只表示未检出矛盾，不证明前置条件可满足。关键前提另用构造见证证明。
      验得过 ⇒ 这一处的前置条件不可满足，判红。一次只插一处：插在外层函数入口的 assert(false) 失败之后，Verus 把其后全部当 false 已知，
      里面闭包的那一处就验得过了（七处一起插只报六处）。
  verus-proof-check.py coverage <根>
      静态检查每个带契约的函数与 invariant 块都有唯一锚点负例；不运行 Verus。
  verus-proof-check.py must-red-worlds <根> [通用选项]
      必红世界：登记表 must-red-worlds 行点名的表（六段制表符分隔：变异名、文件、原文、替换文、proof_errors=M、landing=<落点>；\\n 表换行），
      每行原文在文件里恰好命中一次，改坏的副本上 Verus 要出 JSON、errors ≥ 1，证明错的落点等于第六段登记的落点。一行都没有判红。
      判的是落点，不是条数（C654：条数会随求解环境、无关的 spec 改动漂，核上加一个没人用的 spec fn，同一行就从 2 条跳到 3 条）：
        落点 = 每条证明错取它 --> 位置的那一行，在改坏之后的文件文本里往上找最近一行 fn 的开头（FUNCTION_HEADING），取那个 fn 的名字，去重排序成集合
        （proof_error_landing；位置在 crate 之外或往上没有 fn 记（函数外），输出里有证明错没带位置记（无位置））。第六段 landing=<名>[,<名>…] 写这个集合，顺序随意，
        与实测的集合不相等判红（红句写实测的落点、登记的落点与出路）；同一个 fn 里多报一条循环不变式、条数跳了，落点不变就不红。
        第五段 proof_errors=M 的 M 退成「最近一次观测的条数」：与实测不同只报不判，成功句数一句「N 行证明错条数与第五段不同，只报不判」。
      证明错：输出里 error 起头的报错，除去含 Resource limit (rlimit) exceeded 的、以 aborting due to 起头的与 cargo 那一种编不过时 cargo 自己打的
        「error: could not compile `<包>` (lib) due to N previous error(s)」（proof_error_messages；那一句没有 --> 位置，算进去的话每行多一条、落点多一个（无位置））；
      条数只数它们，不数 rlimit 用尽（Verus 把 rlimit 用尽也计进 JSON 的 errors，同一行换个 rlimit 那个数就变），rlimit 用尽那条的位置不进落点。
      表一律六段，第五段一律写 proof_errors=<正整数>，第六段一律写 landing=<落点>：不是六段的行判红（报行号、要六段），第五段或第六段写成别的形态的行
      不起 Verus、判红，出路写这条约定；
      JSON 里没有整数的 verified 与 errors 的那一行判红。
      证明错一条都没有、只剩 rlimit 用尽的那一行不作数、判红：红在算力额度上，不是证明失败（改替换文，或调高登记的 rlimit）。
      替换文里（去掉 // 注释之后）有 verifier::rlimit / verifier(rlimit …) 的那一行判红：它盖过命令行的 --rlimit。
  通用选项：--verus-root 目录（默认 $SINGLEFS_VERUS_ROOT，再默认共享数据根下的 formal-tools：登记表 release 行第五列起头的 singlefs/formal-tools/ 换成
            .claude/hooks/lib_work_root.py 的 shared_data_path("formal-tools")，不经 ~、$HOME 与 XDG_DATA_HOME：第二台的家目录是另一个，两台同一个绝对路径读同一份
            Verus 与 z3（z3 在安装目录里，跟着这个目录走）；第五列是绝对路径就是它自己（门禁样本用）；取不到数据根、第五列不是这两种写法判红并给出路）
            --threads N（默认 $SINGLEFS_THREAD_CAP，再默认登记的重活默认并行度 .claude/heavy-thread-default）
            --jobs J（空真与必红世界几处同时验，默认 $GATE_VERUS_JOBS，再默认 min(处数, max(1, N // 4))；每一处的 Verus 取 N // J 条线程（--num-threads），至少 1；
              N = 31 时同时至多验 7 处、每处 4 条线程起（7 处 4 条、1 处 31 条），合计不超过 N（parallel_locations）；
              不按内存收处数：环境里的外层内存上限（RUN_WITH_MEMORY_CAP_OUTER_CAP，task-server 起任务时套的包装设的）不看，整条命令的内存额度
              由 task-server 按记录给、撞了放大重排（task-server/README.md「提交写法」）；弄坏开关 outer-memory-narrows-jobs 下设了它就收到 1 处
            每次 Verus 直接起：不套内存包装、capped.sh 与 nice，不设每个进程的内存上限，也不限时：卡住由 task-server 判
              （日志满 10 分钟不涨、CPU 也几乎不涨，或跑过记录用时 5 倍而日志不涨，停掉记 stalled，task-server/README.md），Verus 没出结局就没有判定，不当通过；
              放进它自己的进程组（start_new_session），退了之后组里还留着的（rust_verify、z3 这类）一并 KILL，stdout 与 stderr 边跑边收
              （run_in_own_process_group；弄坏开关 group-leftovers-kept 下不停留下的，它拿着输出管道时判法一直等到它自己退）
            Verus 被信号停了的（直接起时 subprocess 交负的信号号，经 bash 起时退 128 + 信号号，例 137 = 外面 kill -9）：那一处判红，
              ✗ 行写「Verus 被信号停了」、这一次的输出不算判定，→ 查是谁在那一刻停了它（KILLED_HOWTO），不写修证明；整条命令撞了内存上限由 task-server 放大重排，不在这里判
  verus-proof-check.py list-items <根> [--key-mismatches] [--verus-root 目录]
      逐行打可点名的项「项<TAB>格[<TAB>说明]」：toolchain（verus-toolchain-pinned）一项，proof（verus-proof 整格；说明里报成项几个、各哪一类，
      不成项的 spec fn 逐个点名，再接「；空真跳过 assert-by requires N 处：<核>:<行>…」）一项，证明每个成项的单元一项 proof:<全路径>（说明是哪一类与它在核里的位置），
      空真每处 requires 一项 vacuity:<所属 fn 的全路径>#<函数内第几处>（说明是函数或闭包），必红世界每行一项 must-red:<变异名>（说明是表里第几行；
      几张表之间变异名也不许重名）。登记了几个 crate 的逐个 crate 列（全路径的头一段是 crate 名）。
      项名只当索引。只读登记表、核与必红世界表（核经 scripts/gate/lib_function_units.py 切单元），不起 Verus、不编；核没进仓时只列 toolchain 与 proof；登记表或表写坏了判红退 1。
      给了 --key-mismatches：照「库放在哪」那一份库，另往 stderr 每格一行「<格>：键对不上 N 条、无记录 M 条、键相同 K 条、拒算 R 条」，末了「键对不上共 N 条」（底稿第二节第 9 条 ⑦）；
        逐项照现在的键比库（与判那一格时算的键同一份），只读库、不写库、不起 Verus（Verus 要装着：键里的工具链读它的安装）；stdout 的项名逐行不变（自证 list-items-key-mismatch-count）。
        另往 stderr 逐项一行「ITEM_KEY item=<项名> state=<键相同|键对不上|无记录|拒算>」（自证 list-items-item-key-lines；门禁 --list-items --with-keys 转成 ITEM_PLAN）。
        没比出状态的项不打：toolchain、proof（整格这一项，不按条入库）与必红世界里原文、第五段、rlimit 属性没过的行。
  verus-proof-check.py write-fake-tools <目录> <根>   往 <目录> 写假的 Verus 安装（verus、cargo、cargo-verus、version.json；就是 --selftest 用的三个假工具，
      按源码里的标记出判定，不起真 Verus 与真 cargo）；version.json 的版本与 commit 抄 <根> 登记表的 release 行。只给门禁样本档用（自证 fake-tools-written）。
  逐项点名与按条复用（proof、vacuity、must-red-worlds 收 --item；toolchain 只有一项，给 --item 退 2；proof 那一格 --item proof 是整格）：
      --item <项>（可给多次，取并集）只判点名的项，项名写错或缺项名退 2 并列出这一格可点名的项；点名单跑一项与它在整格里跑的判定相同（自证 item-alone-equals-whole-*）；
        证明只点了几个函数时不比各项之和；proof 与 proof:<名> 一起点名照整格判：点名的只核项名在不在，
        整格每一项、各项之和、完整 crate 证明与 MIR 比对照跑（自证 proof-whole-kept；弄坏开关 proof-item-narrows 下只验点名的那一个）。
      proof、vacuity、must-red-worlds 按条记判定（判定库 scripts/crash/lib_verdict_library.py，库放在哪见下面）：每一项判绿当场原子落盘，
        下一趟这一项的键相同就复用、不起 Verus。toolchain 每趟现核、不复用。
      键分两列（判定库文件头「键」一段）：
        判法单元指纹：这一格判结局的那个函数（check_proof、check_vacuity、check_must_red_worlds）在这份脚本里的单元闭包键（判定库与 lib_function_units.py 不进）；
          变了整格每一项都现判，原因写「判法单元指纹 变了」并点名变了的单元。
        输入（逐样的单元键或摘要按名字排好再取 sha256）——每一项都带的三样：工具链（tool-versions 打的那一行去掉路径的那一份，tool_versions_text 的 for_key：
          Verus、它自带的 vstd、z3、rustup 工具链的版本与内容摘要；RUSTUP_HOME、rustup 在 PATH 里的位置、VERUS_ROOT 与 VERUS_Z3_PATH 的原值不进，
          同内容换个位置不现判，自证 reuse-key-toolchain-without-paths）、rlimit（登记的数）与 crate 起法（crate_build_text：crate 行第四段 cargo 还是直接起 verus、
          Cargo.toml 的 edition 与包名，自证 reuse-key-crate-build）；
          verus-pins.tsv 整份不进（它的注释、verified 的登记数与别的行不改任何一项的结论）。
          证明、空真与必红世界每项另带「依赖版本」：单元闭包里各单元真点名的外部 crate 在 Cargo.lock 里落到的包与传递依赖（closure_dependency_lines，
          与 crates 变异行键同一份 lib_mutation_units.py 的 dependency_lines；落不到的退回这个 crate 传递引到的全部外部包，只多作废；自证 reuse-key-closure-dependencies）。
          证明每项另加「被验函数单元闭包」：那个 fn 单元（clone 那一项是 derive 它的类型单元），连同它 requires / ensures / decreases 与函数体里用到的
            spec fn、proof fn、类型、常量、broadcast use 点名的引理（lib_function_units.py 的切法与八条闭包规则，去注释的指纹，不带路径与行号）；
            只改注释、空行、别的 fn 不改它的键；不成项的 spec fn 改了，用到它的那几项现判、红句点名它（自证 reuse-spec-function-all-users）。
            带值的 const 那一项的入口是那个 const 单元，与 fn 同一种算法；assume_specification 那一项的入口是那一条 assume_specification 单元，闭包经 R-路径跨包收到
            它点名的条目（singlefs_format 的常量连同它引到的单元，改那个常量的值只现判指它的那几项；自证 reuse-assume-specification-three-questions）；
            闭包里找不到点名的条目（std 的方法这类不在工作区里的）拒算、照判，拒算行 assume-specification-target 点名那一处（assume_specification_components）。
            再加「被验入口单元」：这一项自己是闭包里的哪一个单元（入口单元与包它的 impl / trait 头一层层往外的种类与去注释指纹，entry_unit_component）。
            一个 trait impl 里的几个方法经 R-结构、R-隐式互相收进对方的闭包，闭包一模一样，没有这一样就共用一把键、复用到别的方法的 verified 数
            （自证 reuse-proof-entry-unit-own-key）。
          空真每处另加「所属函数单元闭包」「所属函数单元」（同上，所属函数的 entry_unit_component；自证 reuse-vacuity-owner-unit-own-key）与「函数内第几处」：
            身份是所属函数单元与第几处 requires，不带路径与行号（空行让 requires 挪一行不现判）。
          必红世界每行另加「受影响的函数单元」（这一行原文所在的那几个单元，与 verus! 里闭包用到它们的每个 fn，这些入口并起来的单元闭包键：
            改坏之后可能报错的就是它们，整个 crate 验、取证明错的落点）与「这一行」（原文、替换文、第六段落点；第五段的条数只报、不进键，变异名与文件路径不进，改名不现判）。
        必红世界表 Verus 不读：改表里一行只现判那一行，证明与空真照复用；表头、注释与空行不进任何一项的键
          （自证 reuse-must-red-table-row-only、reuse-must-red-table-comment-only）。核 crate 里别的文件加一个空行、被验函数体里加一行注释都不现判
          （自证 reuse-crate-other-file-no-cells、reuse-comment-in-function-body）。
      记绿、记红、撤绿：判绿的照键记绿；判红的（Verus 报错）照同一把键调共用库的 record_red 记红，
        盖掉这把键原有的绿，下一趟这一项现判（自证 reuse-red-withdraws-green）；没判成的（被信号停了，outcome_judged）不碰库。
        证明每项随判定记下 verified 数，复用时各项之和用记下的数。
      库放在哪只由被判那棵仓定，命令行不收别的：library-directory <根> 打的那一行，被判那棵树的 git common-dir 底下的 singlefs-formal-unit-verdicts/
        （共用库的 library_directory；键里只用库名，哪棵仓不进键，命名空间只在临时服务底下给）。被判的不是 git 工作树退 2（共用库的 LibraryUnavailable），一项都不判。每项一行
        「· <项>：绿|红，现判（<原因>：<变了的单元>）|复用（<记下的日期与一句判定>）」，末了「· 按条：现判 N、复用 M（现判的原因：<原因> <数>…）」，
        原因是 无记录、关了复用、记录读不出、拒算（<哪一样>）、记录里没有 verified 数、这一行本身不对、<输入里哪几样或判法单元指纹> 变了；
        环境里有 FORMAL_ITEM_TALLY_FILE 的（门禁给的）另往那份文件追加一行「格<TAB>现判数<TAB>复用数<TAB>原因分栏」。
        环境里有 FORMAL_ITEM_DONE_FILE 的（门禁给的）每项判完另追加一行「项<TAB>这一项自己的挂钟秒数<TAB>现判条数<TAB>复用条数<TAB>退出码」（record_item_done，门禁收尾转成 ITEM_DONE）：
        现判的项取 verify_in_parallel 里这一处从起到出结局的挂钟（几处同时验各算各的，不是整格的），复用的 0 秒；绿 0、红 1；
        整格这一项 proof 另记一行：秒数是它自己的活（各项之和、完整 crate 验证、MIR 比对，都不按条入库）、现判 1、复用 0，退出码是这一格有没有问题。
  verus-proof-check.py fingerprint <包目录>      打印包源码的内容指纹与文件数（填登记表用）
  verus-proof-check.py library-directory <根>    打印门禁这一趟的判定库目录（上面「库放在哪」；不读不写库，不起 Verus）
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

代码分块：本文件留 main（子命令分派）、bootstrap 与 --selftest 的守卫；别的名字写在同目录的几份模块里，本文件按名字再导出，别处按路径载入本文件取名字，名字都取得到：
  verus_proof_check_base.py                 常量与弄坏开关集合 BREAK / FORMAL_BREAK、Red / NotRun 与报告函数、登记表 verus-pins.tsv 的读法、起 Verus 与解析它的输出、产品 rustc 与信任清单的读取、词法小工具
  verus_proof_check_listing.py              MIR 文本的读法、被验区域的函数与常量列表、组成键的各项（函数组件、依赖行）、判定库的计划与记账（plan_done、lookup 等）、项的点名与汇总
  verus_proof_check_proofs.py               区域问题与 MIR 比对、判法单元 JUDGING_ENTRIES 与 judging_units、各格的判定（check_toolchain、check_proof、check_vacuity、check_must_red_worlds）、并行验证、工具版本、参数解析、逐 crate 的循环、list-items
  verus_proof_check_selftest_fixtures.py    自证的假 Verus、假 cargo 与样本核（write-fake-tools 子命令也用它）
  verus_proof_check_selftest.py             自证 selftest（main 的 --selftest 分支里才 import，模块级不 import）
下面「弄坏开关」的开关表仍写在本文件。读开关的代码在哪一份（按开关名带引号搜得到；自证里设开关的写法不在这里列）：
  verus-proof-check.py：list-items-pipe-unhandled
  verus_proof_check_base.py：absent-kernel-not-run、rlimit-unpinned、killed-as-proof-red、could-not-compile-counted、const-fn-heading-unrecognized、default-install-by-home、submodule-verify-root、trait-impl-lag-not-run-unreported、whole-entry-past-unit-end、cfg-entry-past-unit-end、group-leftovers-kept、single-crate-registry、cargo-mode-ignored、methods-refused、attribute-mode-unrecognized、no-cheating-kept、trust-list-unchecked、trust-digest-unchecked、trust-test-unchecked、trust-vstd-unchecked、axiom-accepted、cfg-atom-accepted、cfg-macro-unchecked、trait-impl-outside-accepted、mir-compare-skipped、rustc-pin-unchecked、reuse-key-without-trust-list、cargo-fingerprint-kept
  verus_proof_check_listing.py：item-filter-ignored、reuse-key-whole-crate、reuse-key-without-units、reuse-key-function-only、reuse-ignores-key、reuse-key-without-closure-dependencies、reuse-key-without-entry-unit、assume-specification-refused、assume-specification-target-unkeyed、valued-const-not-item、assume-specification-not-item、require-success-key-per-function、require-entire-crate-per-function、key-mismatch-by-item-only、unit-index-not-shared、item-done-unrecorded、reuse-lookup-skipped、body-open-past-unit-end、mir-panic-path-unnormalized、mir-alias-by-prefix、mir-alias-without-boundary、associated-const-bare-name、spec-const-not-item、methods-refused、attribute-mode-unrecognized、cfg-macro-unchecked
  verus_proof_check_proofs.py：commit-unchecked、fingerprint-unchecked、cheating-unchecked、count-unchecked、vacuity-accepts-verified、vacuity-all-at-once、mutation-accepts-green、rlimit-counts-as-red、rlimit-any-line-voids、rlimit-attribute-unchecked、mutation-rlimit-attribute-accepted、anchor-unchecked、expectation-form-unchecked、row-segments-unchecked、landing-unchecked、landing-by-count、landing-read-from-unmutated-text、landing-form-unchecked、rlimit-counted-in-proof-errors、tool-versions-launcher-only、tool-versions-five-files-only、tool-versions-empty-z3-path-as-unset、tool-versions-bare-z3-from-current-directory、tool-versions-verus-environment-ignored、tool-versions-rustup-ignored、item-filter-ignored、reuse-key-whole-crate、vacuity-key-by-line-number、must-red-key-with-name-and-path、reuse-key-without-toolchain、reuse-key-without-judging-units、reuse-key-without-row、reuse-records-red、reuse-records-whole-cell-only、red-keeps-green、proof-item-narrows、reuse-key-without-crate-build、toolchain-key-with-paths、threads-default-cpu-count、spec-fn-as-item、spec-decreases-not-item、derived-clone-not-item、zero-verified-accepted、nonzero-errors-accepted、item-key-lines-skipped、key-items-ignored、key-mismatches-share-across-crates、toolchain-digest-shared-across-installs、slot-directory-per-call、item-done-whole-cell-seconds、trait-declaration-by-brace、trait-impl-lag-matched-by-trait-only、trait-impl-lag-stale-unchecked、trait-impl-lag-passed-unnamed、mir-aliased-items-skipped、derived-clone-copy-only、point-names-by-containment、assert-by-requires-as-insertion-point、assert-by-requires-by-semicolon-line、outer-memory-narrows-jobs、trust-list-unchecked、trust-digest-unchecked、trust-test-unchecked、trust-vstd-unchecked、axiom-accepted、cfg-atom-accepted、trait-impl-outside-accepted、mir-compare-skipped、reuse-key-without-trust-list
  verus_proof_check_selftest_fixtures.py：fake-tools-commit-unpinned
  lib_verus_trait_impl_lag.py（BREAK_SWITCHES）：trait-impl-lag-growth-unchecked、trait-impl-lag-family-unrecognized、trait-impl-lag-family-main-excluded、trait-impl-lag-family-wide、trait-impl-lag-family-sum-unchecked、trait-impl-lag-family-python-split-unrecognized、trait-impl-lag-family-hook-split-unrecognized、trait-impl-lag-family-shell-split-unrecognized、trait-impl-lag-family-directory-entry-unrecognized、test-module-file-impl-judged

弄坏开关 VERUS_PROOF_CHECK_BREAK=<项>[,<项>…]（只给证红用；--selftest 与门禁样本在开着时都必须判错）。
逐项写开关名、弄坏哪一处（括号里是开着时判错的门禁样本），下一行起写打开后自证哪几格判错（项名是 selftest() 的格名，「」里是那一格判错行的开头「✗ <格名> 这一格判错」，
『』里是实测值或转述的原话、不当点名的项；「连带红的格」「红的格」那一行把打开之后实际红的其余几格点全，点名的格与实际红的格逐格相同；
每一格判错打一行以 ✗ 起头、带格名的细节行，转述的输出换行换成「 | 」不另起一行（one_line）；每一项红时都另有一句汇总行「✗ verus-proof-check 自证 <格数> 格里 N 格判错（逐格见上面…）」，
不带格名、不当一格）：
  VERUS_PROOF_CHECK_BREAK=commit-unchecked  toolchain 不比 version.json 的 commit（样本 verus-toolchain-pinned-commit-red 判绿）
      toolchain-commit（「✗ toolchain-commit 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=fingerprint-unchecked  toolchain 不比包源码的内容指纹（样本 verus-toolchain-pinned-fingerprint-red 判绿）
      toolchain-fingerprint（「✗ toolchain-fingerprint 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=absent-kernel-not-run  核不在一律退 77（样本 verus-proof-kernel-deleted-red 退 77）
      kernel-deleted（「✗ kernel-deleted 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=cheating-unchecked  不 grep 核里一律不许的标记（admit(…)、assume(…)、verifier::external）（样本 verus-proof-cheating-red、verus-proof-verifier-external-red 判绿）
      proof-cheating（「✗ proof-cheating 这一格判错」）、proof-rlimit-attribute（「✗ proof-rlimit-attribute 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=count-unchecked  不比各项 verified 之和与登记的数（样本 verus-proof-count-red 判绿）
      proof-count、proof-sum-from-reused-records、proof-sum-short-red-named（『… 这一格判错』）
      红的格：「✗ proof-count 这一格判错」、「✗ proof-sum-from-reused-records 这一格判错」、「✗ proof-sum-short-red-named 这一格判错」
  VERUS_PROOF_CHECK_BREAK=vacuity-accepts-verified  插了 assert(false) 还验得过也当没事（样本 verus-vacuity-red 判绿）
      vacuity-vacuous（「✗ vacuity-vacuous 这一格判错」）、vacuity-closure-vacuous（「✗ vacuity-closure-vacuous 这一格判错」）
      连带红的格：「✗ item-alone-equals-whole-vacuity 这一格判错」
  VERUS_PROOF_CHECK_BREAK=vacuity-all-at-once  一份核的全部插入点插进同一份副本只跑一遍，闭包那一处没单报就当它随外层验过了（样本 verus-vacuity-closure-red 判绿）
      vacuity-closure-vacuous（「✗ vacuity-closure-vacuous 这一格判错」）、vacuity-vacuous（「✗ vacuity-vacuous 这一格判错」）；
      另有几格跟着判错：item-alone-equals-whole-vacuity 与空真那几格按条复用（整份一跑不按项记判定）
      连带红的格：「✗ item-alone-equals-whole-vacuity 这一格判错」、「✗ reuse-vacuity 这一格判错」、「✗ reuse-comment-in-function-body 这一格判错」、
        「✗ reuse-vacuity-line-moved 这一格判错」、「✗ reuse-proof-function-only 这一格判错」、「✗ module-per-kernel 这一格判错」、
        「✗ list-items-key-mismatch-count 这一格判错」、「✗ reuse-must-red-table-row-only 这一格判错」、「✗ reuse-must-red-table-comment-only 这一格判错」、
        「✗ reuse-crate-other-file-no-cells 这一格判错」、「✗ reuse-three-questions 这一格判错」
  VERUS_PROOF_CHECK_BREAK=mutation-accepts-green  改坏的副本上验得过也当红了（样本 verus-must-red-worlds-survives-red 判绿）
      must-red-survives（「✗ must-red-survives 这一格判错」）、must-red-landing-moved（「✗ must-red-landing-moved 这一格判错」）
      连带红的格：「✗ must-red-landing-two-functions-one-registered 这一格判错」、「✗ item-select-one 这一格判错」、「✗ item-alone-equals-whole-must-red 这一格判错」、
        「✗ reuse-row-own-key 这一格判错」、「✗ reuse-red-not-recorded 这一格判错」、「✗ prune-after-change 这一格判错」
  VERUS_PROOF_CHECK_BREAK=rlimit-counts-as-red  必红世界里 rlimit 用尽的报错也当证明失败（样本 verus-must-red-worlds-rlimit-red 判绿）
      must-red-rlimit（「✗ must-red-rlimit 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=rlimit-any-line-voids  必红世界里输出夹着 rlimit 用尽的一律不作数（另有证明错也不算）（样本 verus-must-red-worlds-rlimit-with-proof-error-green 判红）
      must-red-rlimit-with-proof-error（「✗ must-red-rlimit-with-proof-error 这一格判错」）、
      must-red-rlimit-10-counts-proof-errors-only（「✗ must-red-rlimit-10-counts-proof-errors-only 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=rlimit-attribute-unchecked  核里不认 verifier::rlimit 属性（样本 verus-proof-rlimit-attribute-red 判绿）
      proof-rlimit-attribute（「✗ proof-rlimit-attribute 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=mutation-rlimit-attribute-accepted  必红世界不拒替换文里的 verifier::rlimit 属性（样本 verus-must-red-worlds-rlimit-attribute-red 判绿）
      must-red-rlimit-attribute（「✗ must-red-rlimit-attribute 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=rlimit-unpinned  Verus 命令行不带 --rlimit（假 Verus 没见到 --rlimit 就报 vir 错）
      proof-green（「✗ proof-green 这一格判错」）；起 Verus 的格几乎都跟着判错（证明逐函数、空真、必红世界与按条复用那几格），例 must-red-green、proof-count、
      vacuity-vacuous、vacuity-closure-vacuous、must-red-survives、must-red-proof-error-count-drift、proof-rlimit-registered、proof-rlimit-too-low、proof-rlimit-default、
      must-red-rlimit、must-red-rlimit-with-proof-error、must-red-rlimit-10-counts-proof-errors-only、must-red-rlimit-20-counts-proof-errors-only
      连带红的格：「✗ must-red-green 这一格判错」、「✗ proof-count 这一格判错」、「✗ vacuity-vacuous 这一格判错」、「✗ vacuity-closure-vacuous 这一格判错」、「✗ must-red-survives 这一格判错」、
        「✗ must-red-proof-error-count-drift 这一格判错」、「✗ must-red-landing-same-function-more-errors 这一格判错」、「✗ must-red-landing-moved 这一格判错」、
        「✗ must-red-landing-line-shift 这一格判错」、「✗ must-red-landing-two-functions 这一格判错」、「✗ must-red-landing-two-functions-one-registered 这一格判错」、
        「✗ must-red-landing-outside-function 这一格判错」、「✗ proof-rlimit-registered 这一格判错」、「✗ proof-rlimit-too-low 这一格判错」、「✗ proof-rlimit-default 这一格判错」、
        「✗ must-red-rlimit 这一格判错」、「✗ must-red-rlimit-with-proof-error 这一格判错」、「✗ must-red-rlimit-10-counts-proof-errors-only 这一格判错」、
        「✗ must-red-rlimit-20-counts-proof-errors-only 这一格判错」、
        「✗ item-proof-one 这一格判错」、「✗ reuse-first-run-records 这一格判错」、「✗ reuse-second-run-reuses 这一格判错」、「✗ reuse-proof 这一格判错」、
        「✗ reuse-judging-units-changed 这一格判错」、「✗ reuse-comment-in-function-body 这一格判错」、
        「✗ reuse-proof-function-only 这一格判错」、「✗ reuse-toolchain-changed 这一格判错」、「✗ reuse-must-red-renamed 这一格判错」、
        「✗ reuse-spec-function-all-users 这一格判错」、「✗ proof-sum-from-reused-records 这一格判错」、「✗ module-per-kernel 这一格判错」、「✗ proof-spec-fn-not-item 这一格判错」、
        「✗ proof-fn-verified-one-green 这一格判错」、「✗ proof-entire-crate-false-not-red 这一格判错」、「✗ proof-spec-decreases-item 这一格判错」、
        「✗ proof-derived-clone-item 这一格判错」、「✗ proof-sum-equals-registered-green 这一格判错」、「✗ proof-sum-short-red-named 这一格判错」、
        「✗ proof-zero-verified-red 这一格判错」、「✗ proof-errors-one-red 这一格判错」、「✗ list-items-key-mismatch-count 这一格判错」、「✗ reuse-row-own-key 这一格判错」、
        「✗ reuse-resume-after-kill 这一格判错」、「✗ reuse-must-red-table-row-only 这一格判错」、「✗ reuse-must-red-table-comment-only 这一格判错」、
        「✗ reuse-crate-other-file-no-cells 这一格判错」、「✗ reuse-three-questions 这一格判错」、「✗ reuse-sentinel 这一格判错」、
        「✗ same-name-in-two-modules 这一格判错」、「✗ proof-method-items 这一格判错」、「✗ trust-registered-green 这一格判错」、「✗ no-cheating-dropped 这一格判错」、「✗ cfg-atom-registered 这一格判错」、「✗ mir-divergence 这一格判错」、「✗ mir-green 这一格判错」、
        「✗ cargo-mode-proof 这一格判错」、「✗ cargo-mode-must-red 这一格判错」、「✗ cargo-mode-shared-build-directory 这一格判错」、「✗ cargo-mode-command 这一格判错」、「✗ multi-crate-proof 这一格判错」、「✗ multi-crate-item-routed 这一格判错」、「✗ reuse-trust-list-changed 这一格判错」
        「✗ trait-impl-lag-registered-green 这一格判错」、「✗ trait-impl-lag-baseline-green 这一格判错」、「✗ trait-impl-lag-family-moved-green 这一格判错」、
        「✗ trait-impl-lag-family-main-green 这一格判错」、「✗ proof-trait-declaration-struct-literal 这一格判错」
        「✗ proof-assert-by-requires-skip-reported 这一格判错」、「✗ proof-spec-const-items 这一格判错」
  VERUS_PROOF_CHECK_BREAK=anchor-unchecked  不核原文恰好命中一次（样本 verus-must-red-worlds-anchor-red 报不出「恰好命中一次」）
      must-red-anchor（「✗ must-red-anchor 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=expectation-form-unchecked  必红世界不核第五段是不是 proof_errors=<正整数>，认不出的照样起 Verus
                                                      （样本 verus-must-red-worlds-expectation-form-red 判绿）
      must-red-expectation-form（「✗ must-red-expectation-form 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=row-segments-unchecked  必红世界表不核段数，缺的段当空串往下走（样本 verus-must-red-worlds-not-six-segments-red 报不出「不是六段」）
      must-red-row-segments（「✗ must-red-row-segments 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=landing-unchecked  必红世界不比证明错的落点与第六段（落到别的函数照样判绿）（样本 verus-must-red-worlds-landing-moved-red 判绿）
      must-red-landing-moved、must-red-landing-two-functions-one-registered（『… 这一格判错』）
      红的格：「✗ must-red-landing-moved 这一格判错」、「✗ must-red-landing-two-functions-one-registered 这一格判错」
  VERUS_PROOF_CHECK_BREAK=landing-by-count  必红世界改回按条数判：条数与第五段不同判红、落点不比（样本 verus-must-red-worlds-proof-error-count-drift-green 判红、
                                            verus-must-red-worlds-landing-moved-red 判绿）
      must-red-proof-error-count-drift、must-red-landing-same-function-more-errors、must-red-landing-two-functions、must-red-landing-moved、
      must-red-landing-two-functions-one-registered（『… 这一格判错』）
      红的格：「✗ must-red-proof-error-count-drift 这一格判错」、「✗ must-red-landing-same-function-more-errors 这一格判错」、「✗ must-red-landing-moved 这一格判错」、
        「✗ must-red-landing-two-functions-one-registered 这一格判错」
  VERUS_PROOF_CHECK_BREAK=landing-read-from-unmutated-text  落点在改坏之前的文件文本里找（替换文比原文多几行，位置对不上）
      must-red-landing-line-shift（「✗ must-red-landing-line-shift 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=landing-form-unchecked  第六段不是 landing=<落点> 的行照样起 Verus、不比落点
      must-red-landing-form（「✗ must-red-landing-form 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=rlimit-counted-in-proof-errors  必红世界数证明错时把 rlimit 用尽也算进去：条数多一条，只报不判，所以落点相同的行照旧判绿，
      变的只有一句判定里多出『条数只报』、成功句里多一段『其中 N 行证明错条数与第五段 proof_errors= 不同，只报不判』；盯它的是成功句里没有这一段的两格，
      各是 rlimit 用尽夹一条证明错、登记 proof_errors=1 的情形（门禁样本 verus-must-red-worlds-rlimit-with-proof-error-green 的 want 只认存在的片段，这个开关下它照旧判绿，不盯它）
      must-red-rlimit-10-no-count-drift（「✗ must-red-rlimit-10-no-count-drift 这一格判错」）、
      must-red-rlimit-with-proof-error-no-count-drift（「✗ must-red-rlimit-with-proof-error-no-count-drift 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=tool-versions-launcher-only  tool-versions 只打 version.json 与 verus 的 sha256，不看 rust_verify、z3、vstd 与 VERUS_Z3_PATH
      tool-versions-z3（「✗ tool-versions-z3 这一格判错」）；另有 5 格跟着判错：tool-versions-install-top-level、tool-versions-z3-path-empty、
      tool-versions-z3-bare-name、tool-versions-verus-environment、tool-versions-rustup-toolchain
      连带红的格：「✗ reuse-toolchain-changed 这一格判错」、「✗ tool-versions-install-top-level 这一格判错」、「✗ tool-versions-z3-path-empty 这一格判错」、
        「✗ tool-versions-z3-bare-name 这一格判错」、「✗ tool-versions-verus-environment 这一格判错」、「✗ tool-versions-rustup-toolchain 这一格判错」
  VERUS_PROOF_CHECK_BREAK=tool-versions-five-files-only  tool-versions 不打安装目录顶层的整体摘要（安装里多一份宏库）
      tool-versions-install-top-level（「✗ tool-versions-install-top-level 这一格判错」）
      连带红的格：「✗ reuse-toolchain-changed 这一格判错」
  VERUS_PROOF_CHECK_BREAK=tool-versions-empty-z3-path-as-unset  VERUS_Z3_PATH 设成空串记成「未设」
      tool-versions-z3-path-empty（「✗ tool-versions-z3-path-empty 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=tool-versions-bare-z3-from-current-directory  VERUS_Z3_PATH 写裸名时按当前目录找、不照 PATH 找（PATH 里换一份 z3）
      tool-versions-z3-bare-name（「✗ tool-versions-z3-bare-name 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=tool-versions-verus-environment-ignored  tool-versions 不打 VERUS_ROOT 与 VERUS_USE_RUSTUP
                                                                   （两个变量没设、空串、换值、VERUS_ROOT 指的目录换内容）
      tool-versions-verus-environment（「✗ tool-versions-verus-environment 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=tool-versions-rustup-ignored  tool-versions 不打 RUSTUP_HOME、rustup 工具链的库摘要与 rustup 程序
                                                        （假 RUSTUP_HOME 里 librustc_driver 末尾 1 字节改了、libcore rmeta 改了、RUSTUP_HOME 换一份同内容的拷贝、PATH 里换一份 rustup）
      tool-versions-rustup-toolchain（「✗ tool-versions-rustup-toolchain 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=killed-as-proof-red  Verus 被信号停了照旧报「Verus 没出判定」、给这一格自己的出路（修证明）（样本 verus-proof-killed-red 判错）
      proof-killed-direct（假 Verus kill -9 自己；「✗ proof-killed-direct 这一格判错」）、
      proof-killed-no-proof-howto（「✗ proof-killed-no-proof-howto 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=could-not-compile-counted  cargo 那一种编不过时 cargo 自己那一句「error: could not compile `<包>` (lib) due to N previous error(s)」照算成证明错
                                                     （core 表每行必红世界多一条、落点多一个（无位置）；假 cargo-verus 照真 cargo 打这一句）
      cargo-mode-must-red（「✗ cargo-mode-must-red 这一格判错」：落点多出（无位置））
  VERUS_PROOF_CHECK_BREAK=const-fn-heading-unrecognized  空真插入点的说明（FUNCTION_LINE）不认 const fn（core 的 to_device_offset 那一处写成「fn first_file_byte」）
      vacuity-const-fn-label（「✗ vacuity-const-fn-label 这一格判错」：说明写成 fn keep_small）
逐项点名与按条复用的几项：
  VERUS_PROOF_CHECK_BREAK=default-install-by-home  不设 --verus-root 与 SINGLEFS_VERUS_ROOT 时，Verus 安装目录取 ${XDG_DATA_HOME:-~/.local/share}/<登记表 release 行第五列>（落进各台自己的家目录）
      default-install-not-by-home（「✗ default-install-not-by-home 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=list-items-pipe-unhandled  list-items 不照 SIGPIPE 惯例退（接 | head 时打 BrokenPipeError 的回溯、退 1）
      list-items-closed-pipe（「✗ list-items-closed-pipe 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=item-filter-ignored  --item 不起作用、照判全部（写错的项名也不退 2）
      item-select-one、item-unknown、item-proof-one、item-proof-unknown（『… 这一格判错』）
      红的格：「✗ item-select-one 这一格判错」、「✗ item-unknown 这一格判错」、「✗ item-proof-one 这一格判错」、「✗ item-proof-unknown 这一格判错」、
        「✗ multi-crate-item-routed 这一格判错」
  VERUS_PROOF_CHECK_BREAK=reuse-key-whole-crate  被验函数那一样换回核 crate 整个目录的原文摘要（整 crate 一把键）
      reuse-comment-in-function-body（N38：被验函数体里只加一行注释，三格全现判；「✗ reuse-comment-in-function-body 这一格判错」）；另有几格跟着判错：
      reuse-crate-other-file-no-cells、reuse-vacuity-line-moved、reuse-proof-function-only、reuse-spec-function-all-users、
      reuse-must-red-renamed、reuse-must-red-table-row-only、reuse-must-red-table-comment-only
      连带红的格：「✗ reuse-vacuity-line-moved 这一格判错」、「✗ reuse-proof-function-only 这一格判错」、「✗ reuse-must-red-renamed 这一格判错」、
        「✗ reuse-spec-function-all-users 这一格判错」、「✗ list-items-key-mismatch-count 这一格判错」、「✗ reuse-must-red-table-row-only 这一格判错」、
        「✗ reuse-must-red-table-comment-only 这一格判错」、「✗ reuse-crate-other-file-no-cells 这一格判错」、「✗ reuse-three-questions 这一格判错」
  VERUS_PROOF_CHECK_BREAK=reuse-key-without-units  键里不带被验函数的单元闭包（改了一个函数照旧复用）
      reuse-proof-function-only（「✗ reuse-proof-function-only 这一格判错」）；另有 1 格跟着判错：reuse-spec-function-all-users
      连带红的格：「✗ reuse-spec-function-all-users 这一格判错」、「✗ list-items-key-mismatch-count 这一格判错」、「✗ reuse-three-questions 这一格判错」
  VERUS_PROOF_CHECK_BREAK=reuse-key-function-only  被验函数那一样只取它自己的单元，不收它 requires / ensures 与函数体用到的 spec fn、proof fn
      reuse-spec-function-all-users（改一个 spec fn，用到它的两项照旧复用；「✗ reuse-spec-function-all-users 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=vacuity-key-by-line-number  空真每处的键另带核的路径与 requires 的行号
      reuse-vacuity-line-moved（N39：requires 上面加一个空行，空真两处现判；「✗ reuse-vacuity-line-moved 这一格判错」）；
      另有 1 格跟着判错：reuse-comment-in-function-body（那一行注释让后面那个 fn 的 requires 挪了一行）
      连带红的格：「✗ reuse-comment-in-function-body 这一格判错」、「✗ reuse-three-questions 这一格判错」
  VERUS_PROOF_CHECK_BREAK=must-red-key-with-name-and-path  必红世界「这一行」另带变异名与文件路径
      reuse-must-red-renamed（只改变异名那一行现判；「✗ reuse-must-red-renamed 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=reuse-key-without-toolchain  键里不带工具链（Verus 安装里多一份库照旧复用）
      reuse-toolchain-changed（「✗ reuse-toolchain-changed 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=reuse-key-without-judging-units  判法单元指纹一律交空串（判结局的函数改了照旧复用）
      reuse-judging-units-changed（「✗ reuse-judging-units-changed 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=reuse-key-without-row  键里不带「这一行 / 这一处」（同一格几项共用一个键，只判过 A 就拿它的判定当 B 的）
      reuse-row-own-key（「✗ reuse-row-own-key 这一格判错」）；另有 4 格跟着判错：reuse-red-not-recorded、reuse-resume-after-kill、reuse-must-red-table-row-only、
        prune-after-change
      连带红的格：「✗ reuse-red-not-recorded 这一格判错」、「✗ reuse-resume-after-kill 这一格判错」、「✗ reuse-must-red-table-row-only 这一格判错」、「✗ prune-after-change 这一格判错」
  VERUS_PROOF_CHECK_BREAK=reuse-ignores-key  这一项判绿过就复用，不比键
      prune-after-change（剪枝自证：改了一行之后开着复用与关掉复用逐项判定不同）；另有几格跟着判错：reuse-proof-function-only、reuse-toolchain-changed、
        reuse-must-red-table-row-only、reuse-spec-function-all-users、reuse-judging-units-changed、reuse-must-red-renamed
      红的格：「✗ reuse-judging-units-changed 这一格判错」、「✗ reuse-proof-function-only 这一格判错」、「✗ reuse-toolchain-changed 这一格判错」、
        「✗ reuse-must-red-renamed 这一格判错」、「✗ reuse-spec-function-all-users 这一格判错」、「✗ reuse-must-red-table-row-only 这一格判错」、
        「✗ reuse-three-questions 这一格判错」、「✗ reuse-sentinel 这一格判错」、「✗ prune-after-change 这一格判错」
  VERUS_PROOF_CHECK_BREAK=reuse-records-red  红的也记成绿
      reuse-red-not-recorded（「✗ reuse-red-not-recorded 这一格判错」）；另有 1 格跟着判错：reuse-resume-after-kill
      连带红的格：「✗ reuse-resume-after-kill 这一格判错」
  VERUS_PROOF_CHECK_BREAK=reuse-records-whole-cell-only  一项判完不当场落盘，整格判完、一处问题都没有才一起记（被打断或有一项红，判过的全不留）
      reuse-resume-after-kill（第一趟 C 那一处被 kill -9，第二趟同一条命令要只判 C：「✗ reuse-resume-after-kill 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=red-keeps-green  判红的那一项不记红（不调共用库的 record_red），同一把键原有的绿留着照复用
      reuse-red-withdraws-green（先判绿入库，同一把键只写不读地判红一趟，下一趟那一项要现判：「✗ reuse-red-withdraws-green 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=proof-item-narrows  证明那一格同时收到 proof 与 proof:<名> 时只验点名的那一个、不比之和（--item proof --item proof:X 丢了整格）
      proof-whole-kept（「✗ proof-whole-kept 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=reuse-key-without-crate-build  键里不带 crate 起法（cargo 还是直接起、Cargo.toml 的 edition 与包名）
      reuse-key-crate-build（edition 2021 / 2024、cargo 起法三种要三把键：「✗ reuse-key-crate-build 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=reuse-key-without-closure-dependencies  键里不带闭包真点名的外部 crate 在 Cargo.lock 里的版本
      reuse-key-closure-dependencies（Cargo.lock 里 vstd 换版本，点名 vstd:: 的那一项要换键：「✗ reuse-key-closure-dependencies 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=reuse-key-without-entry-unit  证明与空真的键里不带「被验入口单元」「所属函数单元」（一个 trait impl 里几个方法闭包相同、共用一把键）
      reuse-proof-entry-unit-own-key（第二趟 fill 复用到 width 那一份，之和 4 与登记的 5 不同：「✗ reuse-proof-entry-unit-own-key 这一格判错」）、
      reuse-vacuity-owner-unit-own-key（后写的绿盖掉空真那一处的红、第二趟复用成绿：「✗ reuse-vacuity-owner-unit-own-key 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=assume-specification-refused  assume_specification 那一项一律拒算、不入库
      reuse-assume-specification-three-questions（第二趟两条 assume_specification 照样现判，① 红：「✗ reuse-assume-specification-three-questions 这一格判错」）、
      reuse-assume-specification-outside-refused（拒算行不再点名 assume-specification-target：「✗ reuse-assume-specification-outside-refused 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=assume-specification-target-unkeyed  assume_specification 点名的条目不进键
      reuse-assume-specification-three-questions（改 SLOT_WIDTH 的值，指它的那一项照样复用，③ 红：「✗ reuse-assume-specification-three-questions 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=toolchain-key-with-paths  键里工具链那一样照 tool-versions 打的整行，带 RUSTUP_HOME、rustup、VERUS_ROOT、VERUS_Z3_PATH 的路径
      reuse-key-toolchain-without-paths（同内容的工具链与 rustup 换个位置键要不变：「✗ reuse-key-toolchain-without-paths 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=threads-default-cpu-count  --threads 的缺省取核数，不取登记的重活默认并行度
      threads-default-registered（「✗ threads-default-registered 这一格判错」）
  证明成哪几项、每项怎么判（proof_items、verification_clean、check_proof；假 Verus 照真 Verus 的形态出 JSON，自证 fake-verus-real-shape 盯着它）：
  VERUS_PROOF_CHECK_BREAK=spec-fn-as-item  不带 decreases 的 spec fn 也成项（真仓 90 项里 49 个 spec fn 验出 0 条）（样本 verus-must-red-worlds-list-items 判错）
      proof-spec-fn-not-item（「✗ proof-spec-fn-not-item 这一格判错」）；另有几格跟着判错：reuse-spec-function-all-users、proof-entire-crate-false-not-red 等
      连带红的格：「✗ reuse-spec-function-all-users 这一格判错」、「✗ proof-entire-crate-false-not-red 这一格判错」、「✗ proof-spec-decreases-item 这一格判错」、
        「✗ proof-derived-clone-item 这一格判错」、「✗ proof-sum-equals-registered-green 这一格判错」、「✗ proof-sum-short-red-named 这一格判错」、
        「✗ trust-registered-green 这一格判错」、「✗ no-cheating-dropped 这一格判错」
        「✗ proof-trait-declaration-struct-literal 这一格判错」
  VERUS_PROOF_CHECK_BREAK=spec-decreases-not-item  带 decreases 的 spec fn 不成项（它的终止义务没人验，之和少）
      proof-spec-decreases-item（「✗ proof-spec-decreases-item 这一格判错」）
      连带红的格：「✗ proof-spec-fn-not-item 这一格判错」、「✗ proof-entire-crate-false-not-red 这一格判错」、「✗ proof-sum-equals-registered-green 这一格判错」、
        「✗ proof-sum-short-red-named 这一格判错」
  VERUS_PROOF_CHECK_BREAK=derived-clone-not-item  verus! 里 derive(Clone) 的类型不成项（<类型>::clone 没人验，之和少；真仓少 8 条）
      proof-derived-clone-item（「✗ proof-derived-clone-item 这一格判错」）
      连带红的格：「✗ proof-spec-fn-not-item 这一格判错」、「✗ proof-entire-crate-false-not-red 这一格判错」、「✗ proof-sum-equals-registered-green 这一格判错」、
        「✗ proof-sum-short-red-named 这一格判错」
  VERUS_PROOF_CHECK_BREAK=valued-const-not-item  verus! 里带值的 const 不成项（core 的 DataUnitIndexInFile::FIRST 那一条没人验，之和少 1）
      proof-valued-const-and-assume-specification-items（「✗ proof-valued-const-and-assume-specification-items 这一格判错」：之和 3 与登记的 5 不同）
      连带红的格：「✗ list-items-associated-const-prefix 这一格判错」、「✗ proof-associated-const-prefix-green 这一格判错」、「✗ proof-associated-const-point-names 这一格判错」、
        「✗ proof-point-name-trait-default-collision-red 这一格判错」、「✗ proof-point-name-two-impls-collision-red 这一格判错」、「✗ proof-spec-const-items 这一格判错」
        （spec const 也是带值的 const）
  VERUS_PROOF_CHECK_BREAK=assume-specification-not-item  assume_specification 不成项（core 两条格式常量规格各 1 条没人验，之和少 2）
      proof-valued-const-and-assume-specification-items（「✗ proof-valued-const-and-assume-specification-items 这一格判错」：之和 4 与登记的 5 不同）
      连带红的格：「✗ trust-registered-green 这一格判错」、「✗ no-cheating-dropped 这一格判错」
  VERUS_PROOF_CHECK_BREAK=require-success-key-per-function  --verify-function 下 JSON 没有 success 键当没验过（真仓 90 项全红）
      proof-fn-verified-one-green（「✗ proof-fn-verified-one-green 这一格判错」）；proof-green、vacuity-vacuous 等跟着判错
      连带红的格：「✗ proof-green 这一格判错」、「✗ proof-count 这一格判错」、「✗ vacuity-vacuous 这一格判错」、「✗ vacuity-closure-vacuous 这一格判错」、「✗ proof-rlimit-registered 这一格判错」、
        「✗ proof-rlimit-default 这一格判错」、「✗ item-proof-one 这一格判错」、「✗ reuse-proof 这一格判错」、「✗ reuse-comment-in-function-body 这一格判错」、
        「✗ reuse-proof-function-only 这一格判错」、「✗ reuse-spec-function-all-users 这一格判错」、
        「✗ proof-sum-from-reused-records 这一格判错」、「✗ module-per-kernel 这一格判错」、「✗ proof-spec-fn-not-item 这一格判错」、
        「✗ proof-entire-crate-false-not-red 这一格判错」、「✗ proof-spec-decreases-item 这一格判错」、「✗ proof-derived-clone-item 这一格判错」、
        「✗ proof-sum-equals-registered-green 这一格判错」、「✗ proof-sum-short-red-named 这一格判错」、「✗ proof-zero-verified-red 这一格判错」、
        「✗ list-items-key-mismatch-count 这一格判错」、「✗ reuse-must-red-table-row-only 这一格判错」、「✗ reuse-must-red-table-comment-only 这一格判错」、
        「✗ reuse-crate-other-file-no-cells 这一格判错」、「✗ reuse-three-questions 这一格判错」
        「✗ trait-impl-lag-registered-green 这一格判错」、「✗ trait-impl-lag-baseline-green 这一格判错」、「✗ trait-impl-lag-family-moved-green 这一格判错」、
        「✗ trait-impl-lag-family-main-green 这一格判错」、「✗ proof-trait-declaration-struct-literal 这一格判错」
        「✗ proof-assert-by-requires-skip-reported 这一格判错」、「✗ proof-spec-const-items 这一格判错」
  VERUS_PROOF_CHECK_BREAK=require-entire-crate-per-function  按函数验也要 is-verifying-entire-crate 是 true
      proof-entire-crate-false-not-red（「✗ proof-entire-crate-false-not-red 这一格判错」）
      连带红的格：「✗ proof-green 这一格判错」、「✗ proof-count 这一格判错」、「✗ vacuity-vacuous 这一格判错」、「✗ vacuity-closure-vacuous 这一格判错」、「✗ proof-rlimit-registered 这一格判错」、
        「✗ proof-rlimit-default 这一格判错」、「✗ item-proof-one 这一格判错」、「✗ reuse-proof 这一格判错」、「✗ reuse-comment-in-function-body 这一格判错」、
        「✗ reuse-proof-function-only 这一格判错」、「✗ reuse-spec-function-all-users 这一格判错」、
        「✗ proof-sum-from-reused-records 这一格判错」、「✗ module-per-kernel 这一格判错」、「✗ proof-spec-fn-not-item 这一格判错」、「✗ proof-fn-verified-one-green 这一格判错」、
        「✗ proof-spec-decreases-item 这一格判错」、「✗ proof-derived-clone-item 这一格判错」、「✗ proof-sum-equals-registered-green 这一格判错」、
        「✗ proof-sum-short-red-named 这一格判错」、「✗ proof-zero-verified-red 这一格判错」、「✗ list-items-key-mismatch-count 这一格判错」、
        「✗ reuse-must-red-table-row-only 这一格判错」、「✗ reuse-must-red-table-comment-only 这一格判错」、「✗ reuse-crate-other-file-no-cells 这一格判错」、
        「✗ reuse-three-questions 这一格判错」
        「✗ trait-impl-lag-registered-green 这一格判错」、「✗ trait-impl-lag-baseline-green 这一格判错」、「✗ trait-impl-lag-family-moved-green 这一格判错」、
        「✗ trait-impl-lag-family-main-green 这一格判错」、「✗ proof-trait-declaration-struct-literal 这一格判错」
        「✗ proof-assert-by-requires-skip-reported 这一格判错」、「✗ proof-spec-const-items 这一格判错」
  VERUS_PROOF_CHECK_BREAK=zero-verified-accepted  点的函数一条义务都没验（verified 0）也判绿
      proof-zero-verified-red（「✗ proof-zero-verified-red 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=nonzero-errors-accepted  errors 不是 0 也当验过（不是 vir 错的）
      proof-errors-one-red（「✗ proof-errors-one-red 这一格判错」）、proof-broken
      连带红的格：「✗ proof-broken 这一格判错」、「✗ proof-rlimit-too-low 这一格判错」
  VERUS_PROOF_CHECK_BREAK=submodule-verify-root  只验一个函数时不按核所在的模块点，一律 --verify-root（真仓子模块 compaction_plan_proofs 的 20 项证明、9 处空真全红）
      module-per-kernel（根、kernel、nested、nested::deep 四份核，逐项核命令行点的模块；「✗ module-per-kernel 这一格判错」）；
      另有几格跟着判错（自证的核在子模块 src/kernel.rs 里，点根找不到函数）：proof-green、vacuity-green、proof-count 等起 Verus 逐函数验的格
      连带红的格：「✗ proof-green 这一格判错」、「✗ vacuity-green 这一格判错」、「✗ proof-count 这一格判错」、「✗ vacuity-vacuous 这一格判错」、「✗ vacuity-closure-found 这一格判错」、
        「✗ vacuity-closure-vacuous 这一格判错」、「✗ proof-rlimit-registered 这一格判错」、「✗ proof-rlimit-too-low 这一格判错」、「✗ proof-rlimit-default 这一格判错」、
        「✗ item-proof-one 这一格判错」、「✗ item-alone-equals-whole-vacuity 这一格判错」、「✗ reuse-proof 这一格判错」、「✗ reuse-vacuity 这一格判错」、
        「✗ reuse-comment-in-function-body 这一格判错」、「✗ reuse-vacuity-line-moved 这一格判错」、
        「✗ reuse-proof-function-only 这一格判错」、「✗ reuse-spec-function-all-users 这一格判错」、「✗ proof-sum-from-reused-records 这一格判错」、
        「✗ module-unrecognized-red 这一格判错」、「✗ proof-spec-fn-not-item 这一格判错」、「✗ proof-fn-verified-one-green 这一格判错」、
        「✗ proof-entire-crate-false-not-red 这一格判错」、「✗ proof-spec-decreases-item 这一格判错」、「✗ proof-derived-clone-item 这一格判错」、
        「✗ proof-sum-equals-registered-green 这一格判错」、「✗ proof-sum-short-red-named 这一格判错」、「✗ proof-zero-verified-red 这一格判错」、
        「✗ proof-errors-one-red 这一格判错」、「✗ list-items-key-mismatch-count 这一格判错」、「✗ reuse-must-red-table-row-only 这一格判错」、
        「✗ reuse-must-red-table-comment-only 这一格判错」、「✗ reuse-crate-other-file-no-cells 这一格判错」、「✗ reuse-three-questions 这一格判错」、
        「✗ item-proof-unknown 这一格判错」、「✗ list-items 这一格判错」、「✗ same-name-in-two-modules 这一格判错」、「✗ proof-method-items 这一格判错」、「✗ trust-unregistered 这一格判错」、「✗ trust-digest 这一格判错」、「✗ trust-registered-green 这一格判错」、
        「✗ no-cheating-dropped 这一格判错」、「✗ cfg-atom-red 这一格判错」、「✗ cfg-atom-registered 这一格判错」、「✗ cfg-macro-red 这一格判错」、「✗ mir-divergence 这一格判错」、「✗ mir-green 这一格判错」、「✗ cargo-mode-proof 这一格判错」、「✗ cargo-mode-vacuity 这一格判错」、「✗ cargo-mode-shared-build-directory 这一格判错」、
        「✗ cargo-mode-command 这一格判错」、「✗ multi-crate-proof 这一格判错」、「✗ reuse-trust-list-changed 这一格判错」
        「✗ trait-impl-lag-registered-green 这一格判错」、「✗ trait-impl-lag-baseline-green 这一格判错」、「✗ trait-impl-lag-family-moved-green 这一格判错」、
        「✗ trait-impl-lag-family-main-green 这一格判错」、「✗ proof-trait-declaration-struct-literal 这一格判错」
        「✗ list-items-assert-by-requires-skipped 这一格判错」、「✗ vacuity-assert-by-requires-skipped 这一格判错」、「✗ proof-assert-by-requires-skip-reported 这一格判错」、
        「✗ proof-spec-const-items 这一格判错」
  VERUS_PROOF_CHECK_BREAK=key-mismatch-by-item-only  list-items 数键时这一项判绿过就算键相同，不比键
      list-items-key-mismatch-count（「✗ list-items-key-mismatch-count 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=item-key-lines-skipped  list-items --key-mismatches 不逐项打 ITEM_KEY 行
      list-items-item-key-lines（「✗ list-items-item-key-lines 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=key-items-ignored  list-items --key-mismatches 带 --item 也比整表
      list-items-item-key-with-item（「✗ list-items-item-key-with-item 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=unit-index-not-shared  单元索引不共用：每处用到都另建一份，Rust 工作区一趟里载入多次
      key-mismatches-two-crates-shared（「✗ key-mismatches-two-crates-shared 这一格判错」：载入次数不是 1）
  VERUS_PROOF_CHECK_BREAK=key-mismatches-share-across-crates  list-items --key-mismatches 的共用组成不分 crate，第一个 crate 算的交给后面的 crate
      key-mismatches-two-crates-shared（「✗ key-mismatches-two-crates-shared 这一格判错」：second_sample::double_it 的键状态不是键相同）
  VERUS_PROOF_CHECK_BREAK=toolchain-digest-shared-across-installs  工具链摘要的共用缓存只按根存，换了 Verus 安装目录还取上一个目录的摘要
      toolchain-digest-shared（「✗ toolchain-digest-shared 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=slot-directory-per-call  cargo 那一种几处同时验的编译目录一律每次新建临时目录，不建在 CARGO_TARGET_DIR 底下、几次调用不共用
      slot-directories-shared-across-calls（「✗ slot-directories-shared-across-calls 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=item-done-unrecorded  每项判完不往 FORMAL_ITEM_DONE_FILE 记一行
      item-done-lines（「✗ item-done-lines 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=item-done-whole-cell-seconds  每项记的秒数是整格从起到收尾的挂钟，不是这一项自己的
      item-done-lines（keep_small 的秒数不再 < 0.6）
  VERUS_PROOF_CHECK_BREAK=fake-tools-commit-unpinned  write-fake-tools 写的 version.json 的 commit 不抄登记表 release 行
      fake-tools-written（「✗ fake-tools-written 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=reuse-lookup-skipped  一律不读库（键相同照样现判）
      reuse-three-questions（① 第二趟全现判）、reuse-sentinel（复用哨兵判红「复用失灵」，这一格要的就是红：照判；跟着判错的是 ① 那几格）
      连带红的格：「✗ reuse-first-run-records 这一格判错」、「✗ reuse-judging-units-changed 这一格判错」、「✗ reuse-toolchain-changed 这一格判错」、
        「✗ list-items-key-mismatch-count 这一格判错」、「✗ reuse-row-own-key 这一格判错」、「✗ reuse-three-questions 这一格判错」
  VERUS_PROOF_CHECK_BREAK=body-open-past-unit-end  external_body 的规格原文这样取：把单元的 last_token 当最后一个词（一个 impl 里最后一个 fn 取成空串、
                                                   改签名 sha256 不变；别的 fn 取成整个 fn 再多一个词）（样本 verus-proof-trust-digest-last-fn-in-impl-red 判绿）
      trust-digest-signature-only（「✗ trust-digest-signature-only 这一格判错」：impl 里最后一个 fn 是 e3b0c442… 那一个）
  VERUS_PROOF_CHECK_BREAK=trait-declaration-by-brace  trait 里的声明这样认：单元里一个 { 都没有才不成项（ensures 里的结构体字面量当成函数体）
                                                      （样本 verus-proof-list-items-trait-declaration-struct-literal 判错：Store::put 列成了一项）
      proof-trait-declaration-struct-literal（「✗ proof-trait-declaration-struct-literal 这一格判错」：Store::put 成了项、之和 4 与登记的 3 不同）
  VERUS_PROOF_CHECK_BREAK=trait-impl-lag-matched-by-trait-only  放过表的行只按所在文件与 trait 名认、不看实现类型（同一个 trait 在外面的别的 impl 也被放过）
                                                                （样本 verus-proof-trait-impl-outside-unregistered-red 判绿）
      trait-impl-lag-unregistered-red（「✗ trait-impl-lag-unregistered-red 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=trait-impl-lag-stale-unchecked  放过表里已挪进 verus! 或找不到的行不判（样本 verus-proof-trait-impl-outside-moved-red 判绿）
      trait-impl-lag-stale-rows-red（「✗ trait-impl-lag-stale-rows-red 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=trait-impl-lag-growth-unchecked  放过表不与 gate 基准那一版比（涨了也不红，成功句写「没与 gate 基准比」）
      trait-impl-lag-baseline-green（「✗ trait-impl-lag-baseline-green 这一格判错」）、trait-impl-lag-grown-red（「✗ trait-impl-lag-grown-red 这一格判错」）
      连带红的格：「✗ trait-impl-lag-registered-green 这一格判错」、「✗ trait-impl-lag-family-moved-green 这一格判错」、「✗ trait-impl-lag-family-main-green 这一格判错」、
        「✗ trait-impl-lag-family-sum-red 这一格判错」、「✗ trait-impl-lag-family-outside-red 这一格判错」、「✗ trait-impl-lag-family-python-split-green 这一格判错」、
        「✗ trait-impl-lag-family-hook-split-green 这一格判错」、「✗ trait-impl-lag-family-shell-split-green 这一格判错」、「✗ trait-impl-lag-family-script-outside-red 这一格判错」、
        「✗ trait-impl-lag-family-directory-entry-green 这一格判错」
  VERUS_PROOF_CHECK_BREAK=trait-impl-lag-family-unrecognized  放过表与基准比时不认拆分族：拆出来的新文件都当族外
      trait-impl-lag-family-moved-green（「✗ trait-impl-lag-family-moved-green 这一格判错」）、trait-impl-lag-family-main-green（「✗ trait-impl-lag-family-main-green 这一格判错」）
      连带红的格：「✗ trait-impl-lag-family-sum-red 这一格判错」、「✗ trait-impl-lag-family-python-split-green 这一格判错」、「✗ trait-impl-lag-family-hook-split-green 这一格判错」、
        「✗ trait-impl-lag-family-shell-split-green 这一格判错」、「✗ trait-impl-lag-family-directory-entry-green 这一格判错」
  VERUS_PROOF_CHECK_BREAK=trait-impl-lag-family-main-excluded  拆分族里不含 <名>/main.rs
      trait-impl-lag-family-main-green（「✗ trait-impl-lag-family-main-green 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=trait-impl-lag-family-wide  拆分族认宽：基准那一行同目录下的别的 .rs 也算族里，脚本入口同样认宽到拆法认的那几个目录里的任何 .py、.sh
      trait-impl-lag-family-outside-red（「✗ trait-impl-lag-family-outside-red 这一格判错」）、trait-impl-lag-family-script-outside-red（「✗ trait-impl-lag-family-script-outside-red 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=trait-impl-lag-family-sum-unchecked  族内同（trait、类型）的行不比之和：每个文件各自与基准那一行比
      trait-impl-lag-family-sum-red（「✗ trait-impl-lag-family-sum-red 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=trait-impl-lag-family-python-split-unrecognized  不认 Python 拆法：同目录的 <模块名>_<部分>.py 都当族外
      trait-impl-lag-family-python-split-green（「✗ trait-impl-lag-family-python-split-green 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=trait-impl-lag-family-hook-split-unrecognized  不认钩子拆法：同目录的 <名>-<部分>.sh、<名>-<部分>.py 都当族外
      trait-impl-lag-family-hook-split-green（「✗ trait-impl-lag-family-hook-split-green 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=trait-impl-lag-family-shell-split-unrecognized  不认 shell 拆法：.claude/gate.d/lib/ 下的 <名>-<部分>.sh、<名>-<部分>.py 都当族外
      trait-impl-lag-family-shell-split-green（「✗ trait-impl-lag-family-shell-split-green 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=trait-impl-lag-family-directory-entry-unrecognized  不认 main.rs、mod.rs 入口的同目录兄弟：同目录与下层的 .rs 都当族外
      trait-impl-lag-family-directory-entry-green（「✗ trait-impl-lag-family-directory-entry-green 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=test-module-file-impl-judged  verus! 外面的 impl 只看单元自己与外层的 cfg(test)，不看把所在文件连进来的 mod 声明（#[cfg(test)] mod 声明引进来的整份文件里的 impl 照产品判）
      trait-impl-test-module-file-not-judged（「✗ trait-impl-test-module-file-not-judged 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=trait-impl-lag-passed-unnamed  放过的 impl 在成功句里只报个数、不逐个点名「契约没证、靠信任」
                                                       （样本 verus-proof-trait-impl-outside-registered-green 判错）
      trait-impl-lag-registered-green（「✗ trait-impl-lag-registered-green 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=trait-impl-lag-not-run-unreported  放过的 impl 不往 $GATE_NOT_RUN_FILE 报本次未跑（只写在成功句里）
      trait-impl-lag-not-run-reported（「✗ trait-impl-lag-not-run-reported 这一格判错」：文件里 0 行）
  VERUS_PROOF_CHECK_BREAK=mir-panic-path-unnormalized  MIR 比对不照规整表 MIR_PATH_ALIASES 改写（core 103 个条目红在 std::rt::panic_fmt 对 core::panicking::panic_fmt）
                                                       （样本 verus-proof-mir-panic-path-green 判红）
      mir-panic-path-normalized-green（「✗ mir-panic-path-normalized-green 这一格判错」）
      连带红的格：「✗ mir-panic-path-divergence-red 这一格判错」、「✗ mir-unlisted-panic-path-red 这一格判错」、「✗ mir-panic-path-boundary-red 这一格判错」（红句里没有改写几处）
  VERUS_PROOF_CHECK_BREAK=mir-aliased-items-skipped  改写过的条目整个当相同、不比（同一个条目里另有真分叉也放过）
      mir-panic-path-divergence-red（「✗ mir-panic-path-divergence-red 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=mir-alias-by-prefix  规整表按路径前缀改写（std::rt::<名> 一律写成 core::panicking::<名>，表外的同类名字也放过）
      mir-unlisted-panic-path-red（「✗ mir-unlisted-panic-path-red 这一格判错」）
      连带红的格：「✗ mir-panic-path-boundary-red 这一格判错」（后面接 _with 的那一处被改写了）
  VERUS_PROOF_CHECK_BREAK=mir-alias-without-boundary  规整表的名字不卡词边界（前面接 ::、后面接标识符字符的也改写）
      mir-panic-path-boundary-red（「✗ mir-panic-path-boundary-red 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=derived-clone-copy-only  derive(Clone) 只在同时 derive(Copy) 时成项（把 Verus 那条「clone is not a copy」的警告当成没有义务）
      proof-derived-clone-not-copy-item（「✗ proof-derived-clone-not-copy-item 这一格判错」：之和 5 与登记的 6 不同）
  VERUS_PROOF_CHECK_BREAK=whole-entry-past-unit-end  uninterp、external_*_specification、带 external_body 的类型规格与 cfg 原子多取下一个条目的头一个词
      trust-digest-whole-entry-half-open（「✗ trust-digest-whole-entry-half-open 这一格判错」）
      连带红的格：「✗ trust-digest-cfg-entry-half-open 这一格判错」
  VERUS_PROOF_CHECK_BREAK=cfg-entry-past-unit-end  只 cfg 原子的「所在条目整个」多取下一个条目的头一个词
      trust-digest-cfg-entry-half-open（「✗ trust-digest-cfg-entry-half-open 这一格判错」：后接 pub 与后接 fn 的 sha256 不同）
  VERUS_PROOF_CHECK_BREAK=associated-const-bare-name  impl 里的关联 const 交给 --verify-function 的是 <名>，不是 *::<名>
                                                      （样本 verus-proof-list-items-associated-const-prefix 判错：列项退 1）
      list-items-associated-const-prefix（「✗ list-items-associated-const-prefix 这一格判错」：FIRST 在表里命中 2 项）
      连带红的格：「✗ proof-associated-const-prefix-green 这一格判错」、「✗ proof-associated-const-point-names 这一格判错」、
        「✗ proof-point-name-trait-default-collision-red 这一格判错」、「✗ proof-point-name-two-impls-collision-red 这一格判错」（红句点名的是 LIMIT、FIRST，不是 *::<名>）
  VERUS_PROOF_CHECK_BREAK=point-names-by-containment  点名不照 Verus 的点法在可点名函数表里点，只比成项的关联 const 与 assume_specification 谁是谁的一段
                                                      （样本 verus-proof-list-items-trait-default-collision-red 判错：列项退 0）
      proof-point-name-trait-default-collision-red（「✗ proof-point-name-trait-default-collision-red 这一格判错」：找不到「命中 2 项」）
      连带红的格：「✗ proof-point-name-two-impls-collision-red 这一格判错」（红句不点名撞的几项）
  VERUS_PROOF_CHECK_BREAK=assert-by-requires-as-insertion-point  属于 assert(…) by (…) 的 requires 当空真插入点（往下找不到函数体的 { 判红）
                                                                 （样本 verus-vacuity-list-items-assert-by-requires-green 判错：列项退 1）
      list-items-assert-by-requires-skipped（「✗ list-items-assert-by-requires-skipped 这一格判错」）、vacuity-assert-by-requires-skipped（「✗ vacuity-assert-by-requires-skipped 这一格判错」）、
      proof-assert-by-requires-skip-reported（「✗ proof-assert-by-requires-skip-reported 这一格判错」：成功句报跳过 0 处）
  VERUS_PROOF_CHECK_BREAK=assert-by-requires-by-semicolon-line  照「往下先碰到一行只有 ; 」认 assert-by 的 requires，不看前面接的是不是 assert(…) by (…)
                                                                （样本 verus-vacuity-list-items-requires-on-plain-block-red 判错：列项退 0）
      list-items-requires-on-plain-block-red（「✗ list-items-requires-on-plain-block-red 这一格判错」）、
      list-items-requires-on-assert-without-by-red（「✗ list-items-requires-on-assert-without-by-red 这一格判错」）
  VERUS_PROOF_CHECK_BREAK=spec-const-not-item  spec const 不成项（真 Verus 整 crate 验时每个 spec const 多 1 条，之和少）
                                               （样本 verus-proof-list-items-spec-const 判错：成项 3 个，找不到两个 spec const 那两行）
      proof-spec-const-items（「✗ proof-spec-const-items 这一格判错」：之和 2 与登记的 4 不同）
  VERUS_PROOF_CHECK_BREAK=outer-memory-narrows-jobs  环境里设了外层内存上限（RUN_WITH_MEMORY_CAP_OUTER_CAP）就把几处同时验收到 1 处
      jobs-not-narrowed-by-outer-memory（「✗ jobs-not-narrowed-by-outer-memory 这一格判错」：外层 1G、--threads 8、两处，每处起的 Verus 带 --num-threads 8）
  VERUS_PROOF_CHECK_BREAK=group-leftovers-kept  Verus 退了之后不停它的进程组里留下的
      group-leftovers-stopped（「✗ group-leftovers-stopped 这一格判错」：直接起的 bash 退了、它在后台起的 sleep 拿着输出管道，判法等到 sleep 自己退才交回）
  REUSE_CHECKS_BREAK=rejudged-sentinel-passes（共用库 scripts/gate/lib_reuse_checks.py 的开关）  哨兵被现判不红
      reuse-sentinel（「✗ reuse-sentinel 这一格判错」）
复用三问与复用哨兵（scripts/gate/lib_reuse_checks.py 文件头「一」「二」）：
  自证 reuse-three-questions 格三格的项一起算，打规范输出「REUSE_SELFTEST script=scripts/gate/verus-proof-check.py …」（门禁 code-tooling 的 reuse-selftests 格认它）；
  已有同义的格（reuse-first-run-records、reuse-second-run-reuses、reuse-comment-in-function-body、reuse-crate-other-file-no-cells、reuse-proof-function-only、
  reuse-spec-function-all-users）在那一格的注释里点名。
  每一趟真跑（proof、vacuity、must-red-worlds 三格各一份）开跑前从库里按键抽哨兵（候选按键文件查，lib_reuse_checks.verdict_library_candidate，与 lookup 分开），
  抽中强制现判的那一项那一行写「强制现判（复用哨兵抽中…）」；哨兵被现判、强制现判判定与库里不同，那一格判红。种子与条数取 SINGLEFS_REUSE_SENTINEL_SEED、
  SINGLEFS_REUSE_SENTINEL_COUNT；没给库、list-items 数键的那一趟不抽。
管不到的：spec 本身写得对不对；核与调它的 singlefs-core 之间（recovery.rs 调核那一段）没被证明的部分；插入点只认「requires 整行起头、函数体的 { 单独起行或是闭包那一行的头一个 {」
  这种写法（verusfmt 与本仓的写法），别的写法认不出就判红要求改写法。assert(…) by (…) 接 requires 再接一个证明块 { … }（不是 ; 收尾）的不当 assert-by 跳过，
  照旧往下找到那个块的 { 当插入点（replay-prefix 的 file_range_proofs.rs 有一处：插的 assert(false) 落在那个块里，判的是那几条 requires 在那一处可不可满足）。
  模块按核的文件路径认，不读 mod 声明：#[path = …] 挪了文件、或核写在 mod <名> { … } 内联块里的，点的模块与真的不同，Verus 报 could not find module / function、判红。
  差分测试只核在 crates/ 下标了 #[test]，不在这里跑（它归 验证包，随改动跑）；被验区域之外对被验类型的构造与对带 requires 的被验函数的调用
  这一份不判；必红世界逐行手写，不按 ensures 自动造；逐函数结局逐项起 Verus，不读整 crate 一趟的 func-details。
被验区域与多 crate 的弄坏开关（门禁 formal-verification 那一道的变量 FORMAL_VERIFICATION_BREAK=<项>[,<项>…]，判法这一半在这份脚本里，那一道文件头另列一份
它们让哪份门禁样本判错）；逐项写开关、弄坏哪一处，下一行起写打开后自证哪几格判错（「」里是那一格判错行的开头）：
  FORMAL_VERIFICATION_BREAK=single-crate-registry  登记表只认第一个 crate
      multi-crate-proof（「✗ multi-crate-proof 这一格判错」）、multi-crate-list-items（「✗ multi-crate-list-items 这一格判错」）、
      multi-crate-item-routed（「✗ multi-crate-item-routed 这一格判错」）
  FORMAL_VERIFICATION_BREAK=cargo-mode-ignored  crate 行第四段 cargo 不认、照直接起 verus 判（有别的依赖就红）
      cargo-mode-proof（「✗ cargo-mode-proof 这一格判错」）、cargo-mode-vacuity（「✗ cargo-mode-vacuity 这一格判错」）、
      cargo-mode-must-red（「✗ cargo-mode-must-red 这一格判错」）、cargo-mode-command（「✗ cargo-mode-command 这一格判错」）、
      cargo-mode-shared-build-directory（「✗ cargo-mode-shared-build-directory 这一格判错」）
  FORMAL_VERIFICATION_BREAK=methods-refused  impl 与 trait 里的方法判红、要挪成自由 fn
      proof-method-items（「✗ proof-method-items 这一格判错」）、proof-same-point-name（「✗ proof-same-point-name 这一格判错」）
      连带红的格：「✗ trait-impl-lag-registered-green 这一格判错」、「✗ trait-impl-lag-baseline-green 这一格判错」、「✗ trait-impl-lag-family-moved-green 这一格判错」、
        「✗ trait-impl-lag-family-main-green 这一格判错」、「✗ proof-trait-declaration-struct-literal 这一格判错」
  FORMAL_VERIFICATION_BREAK=attribute-mode-unrecognized  属性写法（#[verus_verify]、#[verus_spec]）标的文件不当核
      unregistered-attribute-kernel（「✗ unregistered-attribute-kernel 这一格判错」）
  FORMAL_VERIFICATION_BREAK=no-cheating-kept  照旧带 --no-cheating（登记了的 assume_specification、external_body 被 Verus 拒）
      trust-registered-green（「✗ trust-registered-green 这一格判错」）、no-cheating-dropped（「✗ no-cheating-dropped 这一格判错」）、
      cargo-mode-command（「✗ cargo-mode-command 这一格判错」）
  FORMAL_VERIFICATION_BREAK=trust-list-unchecked  信任清单不判（没登记的可信声明照收、表不在、vstd 行对不上、登记了却没有那一处都不红；被验区域里有 cfg 原子时 cfg 那几行照判）
      trust-unregistered（「✗ trust-unregistered 这一格判错」）、trust-digests-reported（「✗ trust-digests-reported 这一格判错」）、
      trust-stale-row（「✗ trust-stale-row 这一格判错」）、trust-list-missing（「✗ trust-list-missing 这一格判错」）、
      trust-vstd（「✗ trust-vstd 这一格判错」）、trust-digest（「✗ trust-digest 这一格判错」）、
      trust-test-missing（「✗ trust-test-missing 这一格判错」）
  FORMAL_VERIFICATION_BREAK=trust-digest-unchecked  不比登记的规格原文 sha256 与现算的
      trust-digest（「✗ trust-digest 这一格判错」）
  FORMAL_VERIFICATION_BREAK=trust-test-unchecked  不核差分测试名在 crates/ 下标了 #[test]
      trust-test-missing（「✗ trust-test-missing 这一格判错」）
  FORMAL_VERIFICATION_BREAK=trust-vstd-unchecked  不核信任清单的 vstd 行（恰好一行、与登记表 package vstd 行同版本同指纹）
      trust-vstd（「✗ trust-vstd 这一格判错」）
  FORMAL_VERIFICATION_BREAK=axiom-accepted  核里的 axiom 不判
      axiom-red（「✗ axiom-red 这一格判错」）
  FORMAL_VERIFICATION_BREAK=cfg-atom-accepted  被验区域里的 cfg 原子（连同宏里的 cfg）不认、不要登记
      cfg-atom-red（「✗ cfg-atom-red 这一格判错」）、cfg-macro-red（「✗ cfg-macro-red 这一格判错」）、cfg-feature-fork（「✗ cfg-feature-fork 这一格判错」）、
      cfg-atom-registered（「✗ cfg-atom-registered 这一格判错」）
  FORMAL_VERIFICATION_BREAK=cfg-macro-unchecked  定义里有 cfg 的宏在被验区域里的调用不认（只认写在被验区域里的 cfg）
      cfg-macro-red（「✗ cfg-macro-red 这一格判错」）
  FORMAL_VERIFICATION_BREAK=trait-impl-outside-accepted  被验的类型与 trait 在 verus! 外面的 trait impl 不判（放过表也不读）
      trait-impl-outside（「✗ trait-impl-outside 这一格判错」）
      连带红的格：「✗ trait-impl-lag-registered-green 这一格判错」、「✗ trait-impl-lag-unregistered-red 这一格判错」、「✗ trait-impl-lag-stale-rows-red 这一格判错」、
        「✗ trait-impl-lag-baseline-green 这一格判错」、「✗ trait-impl-lag-grown-red 这一格判错」、「✗ trait-impl-lag-family-moved-green 这一格判错」、
        「✗ trait-impl-lag-family-main-green 这一格判错」、「✗ trait-impl-lag-family-sum-red 这一格判错」、「✗ trait-impl-lag-family-outside-red 这一格判错」
  FORMAL_VERIFICATION_BREAK=mir-compare-skipped  不比产品构建与 Verus 构建的 MIR
      mir-divergence（「✗ mir-divergence 这一格判错」）、mir-green（「✗ mir-green 这一格判错」）、cargo-mode-proof（「✗ cargo-mode-proof 这一格判错」）、
      cargo-mode-command（「✗ cargo-mode-command 这一格判错」）
  FORMAL_VERIFICATION_BREAK=rustc-pin-unchecked  不核登记表 rustc 行（Verus 与产品的 rustc 版本差）
      rustc-pin-product（「✗ rustc-pin-product 这一格判错」）、rustc-pin-verus（「✗ rustc-pin-verus 这一格判错」）、
      rustc-pin-missing（「✗ rustc-pin-missing 这一格判错」）
  FORMAL_VERIFICATION_BREAK=reuse-key-without-trust-list  每一项的键里不带信任清单（改了一行登记照旧复用）
      reuse-trust-list-changed（「✗ reuse-trust-list-changed 这一格判错」）
  FORMAL_VERIFICATION_BREAK=cargo-fingerprint-kept  cargo 那一种起之前不删被验的 crate 在共用编译目录里的指纹（第二处起拿不到判定行）
      cargo-mode-shared-build-directory（「✗ cargo-mode-shared-build-directory 这一格判错」）
"""

import collections
import concurrent.futures
import contextlib
import fcntl
import glob
import io
import hashlib
import importlib.util
import json
import os
import queue
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
from lib_verdict_library import key_of, refused_component  # noqa: E402
import lib_verdict_library  # noqa: E402  library_root（库根）、locked_dependency_lines（闭包的依赖版本行算不出时退回整个 crate 的）
import lib_reuse_checks as reuse_checks  # noqa: E402
from lib_function_units import rust_lex  # noqa: E402
import lib_verus_trait_impl_lag as trait_impl_lag  # noqa: E402  放过表只缩不涨的判法与它的自证格（trait_impl_lag_growth、selftest_cells）

sys.path.insert(0, os.path.dirname(os.path.realpath(__file__)))
from verus_proof_check_base import (
     SCRIPT_DIRECTORY, REPOSITORY_ROOT, REGISTRY, SCRIPT_KEY, EXIT_GREEN, EXIT_RED, EXIT_USAGE, EXIT_NOT_RUN, DEFAULT_RLIMIT, RLIMIT_EXHAUSTED,
     EXPECTED_PROOF_ERRORS, EXPECTATION_HOWTO, OUTER_MEMORY_CAP_VARIABLE, KILLED_HOWTO, TOOL_VERSION_FILES, RUSTUP_DEFAULT_HOME,
     EXCLUDED_PACKAGE_FILES, CHEATING_MARKER, TRUST_LIST, TRUST_KINDS, CFG_KIND, TRUST_ROW_HOWTO, TRAIT_IMPLS_OUTSIDE_LAG, LAG_BASELINE_LIBRARY,
     TRAIT_IMPL_OUTSIDE_HOWTO, RLIMIT_ATTRIBUTE, FUNCTION_LINE, LANDING_OUTSIDE, LANDING_NO_LOCATION, LANDING_TOKEN, EXPECTED_LANDING, LANDING_HOWTO,
     FUNCTION_HEADING, DIAGNOSTIC_LOCATION, BREAK, CARGO_GAVE_UP, FORMAL_BREAK, FORMAL_BREAK_ITEMS, ITEM_TALLY_FILE_VARIABLE, ITEM_DONE_FILE_VARIABLE,
     ITEM_KEY_PREFIX, Red, NotRun, report_red, one_line, report_cell, strip_line_comment, Registry, CRATE_FIELDS, REGISTRY_WIDTHS, per_crate_view,
     crate_identifier, read_registry, kernel_files, INSTALL_SHARED_ENTRY, INSTALL_PREFIX, work_root_library, verus_directory, read_version_json,
     crate_manifest, check_dependencies, path_dependencies, copy_crate, RUST_IDENTIFIER, kernel_module, verus_arguments, forget_crate_fingerprints,
     verus_program, verus_program_arguments, crate_package_name, diagnostic_path, located_file, kernel_modules, run_in_own_process_group, run_verus,
     diagnostic_blocks, proof_error_messages, integer_counts, outcome_problem, outcome_judged, problem_howto, killed_signal, product_rustc_version,
     rustc_pin_problems, cheating_markers, rlimit_attributes, read_trust_list, _LAG_BASELINE, lag_baseline_library, read_trait_impls_outside_lag,
     NOT_RUN_FILE_VARIABLE, report_passed_not_run, trust_list_text, TEST_FUNCTION, test_functions, innermost_unit, joined_tokens, statement_end,
     body_open, body_open_past_unit_end, whole_entry_end, assume_specification_parts,
)  # noqa: E402,F401
from verus_proof_check_listing import (
     unit_path, region_spans, trusted_declarations, MIR_ITEM_HEAD, MIR_PATH_ALIASES, mir_path_pattern, normalized_mir, mir_alias_text, mir_items,
     newest_mir, verus_executable, crate_registry, prepared, UsageError, unit_index, verus_ranges, VerusFunction, impl_parts, unit_attribute_words,
     attribute_verified, in_verified_region, verus_point_name, verus_functions, DERIVED_VERIFIED_TRAITS, PROOF_FN, EXEC_FN, SPEC_FN_WITH_DECREASES,
     DERIVED_CLONE, VALUED_CONST, ASSUME_SPECIFICATION, PLAIN_PATH, DeclarationSite, ProofItem, leading_words, function_mode, derived_traits,
     valued_const_entries, valued_const_point_name, verus_listing, top_level_segments, enum_arrow_names, verus_function_matches,
     assume_specification_entries, owner_function, crate_build_text, _MUTATION_UNITS, mutation_units_module, closure_dependency_lines,
     dependency_component, function_components, entry_unit_component, proof_entry_unit, ASSUME_TARGET_KINDS, assume_specification_components,
     FORMAL_LIBRARY_DIRECTORY_NAME, formal_library_directory, FAKE_TOOLCHAIN, library_directory_command, PLAN_STATES, peek, plan_done, lookup,
     judged_how, plan_sentinel, sentinel_lookup, selected_items, record_item_done, report_item, cell_tally, finish_tally, merged_tally, proof_item,
     results_text, verification_clean, item_kinds_text, listed_names,
)  # noqa: E402,F401
from verus_proof_check_proofs import (
     check_toolchain, region_problems, compare_mir, JUDGING_ENTRIES, _JUDGING_UNITS, judging_units, proof_items, point_name_problems,
     point_names_by_containment, toolchain_digest, shared_components, verdict_library, check_proof, assert_by_requires_lines,
     assert_by_requires_sites, assert_by_skipped_text, requires_insertion_points, parallel_locations, SLOT_DIRECTORY_NAME, claim_slot_directories,
     verify_in_parallel, ItemJudging, with_assert_false, vacuity_points, vacuity_item_problems, check_vacuity, must_red_rows, must_red_item,
     function_keys, must_red_components, enclosing_function, parse_landing, proof_error_landing, must_red_row_problems, prepare_must_red_row,
     negative_coverage_gaps, check_negative_coverage, check_must_red_worlds, package_source_fingerprint, streamed_sha256, rustup_toolchain_fields,
     tool_versions, tool_versions_text, default_threads, parse_options, CHECKS, PER_CRATE_CHECKS, run_check, crate_item_lines, run_per_crate,
     list_items, report_key_mismatches, selftest_break_caught, selftest_break_miss_detail,
)  # noqa: E402,F401

def main(arguments):
    if arguments[:1] == ["--selftest"]:
        from verus_proof_check_selftest import selftest
        return selftest()
    if arguments[:1] == ["list-items"]:
        try:
            positional, options = parse_options(arguments[1:])
        except ValueError as error:
            positional, options = [], {"error": str(error)}
        extra = [flag for flag in ("cargo_home",) if options.get(flag)]
        if len(positional) != 1 or not os.path.isdir(positional[0]) or options.get("error") or extra:
            report_red(f"list-items 要恰好一个仓根，另外只收 --key-mismatches 与 --verus-root 目录{'（' + options['error'] + '）' if options.get('error') else ''}",
                       "例：verus-proof-check.py list-items /path/to/singlefs [--key-mismatches]")
            return EXIT_USAGE
        if "list-items-pipe-unhandled" not in BREAK:
            # 接 | head 时读的一头先关了：照 SIGPIPE 的惯例安静退出（被 SIGPIPE 停，shell 里退 141），不打 BrokenPipeError 的回溯
            signal.signal(signal.SIGPIPE, signal.SIG_DFL)
        return list_items(positional[0], formal_library_directory(os.path.abspath(positional[0])) if options.get("key_mismatches") else None, options["verus_root"], options["items"])
    if arguments[:1] == ["write-fake-tools"]:
        if len(arguments) != 3 or not os.path.isdir(arguments[2]):
            report_red("write-fake-tools 要恰好两个参数：假工具目录与被判的仓根", "例：verus-proof-check.py write-fake-tools <样本根>/.formal-fake-tools <样本根>")
            return EXIT_USAGE
        from verus_proof_check_selftest_fixtures import write_fake_tools
        return write_fake_tools(arguments[1], os.path.abspath(arguments[2]))
    if arguments[:1] == ["library-directory"]:
        if len(arguments) != 2 or not os.path.isdir(arguments[1]):
            report_red("library-directory 要恰好一个仓根", "例：verus-proof-check.py library-directory /path/to/singlefs")
            return EXIT_USAGE
        return library_directory_command(arguments[1])
    if not arguments or arguments[0] not in (*CHECKS, "fingerprint", "tool-versions"):
        report_red("用法：verus-proof-check.py <toolchain|proof|vacuity|must-red-worlds> <根> [选项] | list-items <根> [--key-mismatches] | fingerprint <包目录> | tool-versions [<根>] | --selftest",
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
    if arguments[0] != "toolchain":
        options["verdict_library"] = formal_library_directory(os.path.abspath(positional[0]))   # 文件头「库放在哪」：只由被判那棵仓定，不是 git 工作树退 2
    return run_check(arguments[0], positional[0], options)


BREAK_VARIABLES = ("FORMAL_VERIFICATION_BREAK", "VERUS_PROOF_CHECK_BREAK")


def inside_selftest():
    """往上数这个进程的祖先，有一个的命令行里带 --selftest（自证起的子进程，含别的脚本的自证起它的）就是 True。"""
    pid = os.getppid()
    while pid > 1:
        try:
            with open(f"/proc/{pid}/cmdline", "rb") as handle:
                if b"--selftest" in handle.read().split(b"\0"):
                    return True
            with open(f"/proc/{pid}/stat", "rb") as handle:
                pid = int(handle.read().rsplit(b")", 1)[1].split()[1])
        except (OSError, ValueError, IndexError):
            return False
    return False


def settle_break_switches(arguments):
    """弄坏开关只在自证里认（checks-owed.md C622），参数解析之前先过这一道。
    放行：命令行以 --selftest 起头；门禁样本根（当前目录有 .gate-cells 或 .gate-break-sample、没设 GATE_STAGED_FROM）里由阶段转来的；祖先里有自证（inside_selftest）。
    都不是而 BREAK_VARIABLES 里有设了的：点名变量、说它只在自证里认，交回退出码 2；否则交回 None。"""
    setting = [name for name in BREAK_VARIABLES if os.environ.get(name)]
    sample_root = (os.path.exists(".gate-cells") or os.path.exists(".gate-break-sample")) and not os.environ.get("GATE_STAGED_FROM")
    if not setting or arguments[:1] == ["--selftest"] or sample_root or inside_selftest():
        return None
    print(f"  ✗ {'、'.join(setting)} 是弄坏开关，只在自证（--selftest 本身与它起的子进程）和门禁样本根里认；平常跑设了就拒，这一次什么都没做", file=sys.stderr)
    print(f"     → 怎么办：unset {' '.join(setting)} 再来；要看开关红在哪一项，跑 {setting[0]}=<项> python3 scripts/gate/verus-proof-check.py --selftest", file=sys.stderr)
    return 2


if __name__ == "__main__":
    refused = settle_break_switches(sys.argv[1:])
    if refused is not None:
        sys.exit(refused)
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
